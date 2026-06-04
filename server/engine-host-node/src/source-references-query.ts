import type { Range } from "@omena/shared";
import {
  findCanonicalSelector,
  readSourceExpressionResolution,
  type SourceExpressionContext,
} from "../../engine-core-ts/src/core/query";
import type {
  SelectorDeclHIR,
  StyleDocumentHIR,
} from "../../engine-core-ts/src/core/hir/style-types";
import type { CursorParams, ProviderDeps } from "../../engine-core-ts/src/provider-deps";
import { pathToFileUrl } from "../../engine-core-ts/src/core/util/text-utils";
import {
  resolveRustSourceResolutionSelectorMatchAsync,
  resolveRustSourceResolutionSelectorMatch,
  usesRustSelectorUsageBackend,
  usesRustSourceResolutionBackend,
} from "./source-resolution-query-backend";
import {
  buildSelectorUsageLocationsFromRustPayload,
  resolveRustSelectorUsagePayloadForWorkspaceTargetAsync,
  resolveRustSelectorUsagePayloadForWorkspaceTarget,
  type SelectorUsagePayloadCache,
} from "./selector-usage-query-backend";
import { resolveSelectedQueryBackendKind } from "./selected-query-backend";
import { resolveSelectorReferenceLocations } from "./selector-references-query";
import {
  resolveRustStyleSelectorReferenceSummaryForWorkspaceTargetAsync,
  resolveRustStyleSelectorReferenceSummaryForWorkspaceTarget,
  type StyleSelectorReferenceQueryOptions,
} from "./style-selector-reference-query";

export interface SourceReferenceLocation {
  readonly uri: string;
  readonly range: Range;
}

interface SourceReferenceTarget {
  readonly filePath: string;
  readonly canonicalName: string;
}

export interface SourceReferencesQueryOptions extends StyleSelectorReferenceQueryOptions {
  readonly env?: NodeJS.ProcessEnv;
  readonly readRustSourceResolutionSelectorMatch?: typeof resolveRustSourceResolutionSelectorMatch;
  readonly readRustSourceResolutionSelectorMatchAsync?: typeof resolveRustSourceResolutionSelectorMatchAsync;
  readonly readRustSelectorUsagePayloadForWorkspaceTarget?: typeof resolveRustSelectorUsagePayloadForWorkspaceTarget;
  readonly readRustSelectorUsagePayloadForWorkspaceTargetAsync?: typeof resolveRustSelectorUsagePayloadForWorkspaceTargetAsync;
  readonly selectorUsagePayloadCache?: SelectorUsagePayloadCache;
}

export function resolveSourceExpressionReferences(
  ctx: SourceExpressionContext,
  params: Pick<CursorParams, "documentUri" | "content" | "filePath" | "version">,
  deps: Pick<
    ProviderDeps,
    | "analysisCache"
    | "semanticReferenceIndex"
    | "settings"
    | "styleDependencyGraph"
    | "styleDocumentForPath"
    | "typeResolver"
    | "workspaceRoot"
    | "readStyleFile"
  >,
  options: SourceReferencesQueryOptions = {},
): readonly SourceReferenceLocation[] {
  const targets = resolveSourceReferenceTargets(ctx, params, deps, options);
  if (targets.length === 0) return [];

  const locations = targets.flatMap((target) =>
    resolveReferenceLocationsForTarget(target, deps, options),
  );
  return dedupeLocations(locations);
}

export async function resolveSourceExpressionReferencesAsync(
  ctx: SourceExpressionContext,
  params: Pick<CursorParams, "documentUri" | "content" | "filePath" | "version">,
  deps: Pick<
    ProviderDeps,
    | "analysisCache"
    | "semanticReferenceIndex"
    | "settings"
    | "styleDependencyGraph"
    | "styleDocumentForPath"
    | "typeResolver"
    | "workspaceRoot"
    | "readStyleFile"
  >,
  options: SourceReferencesQueryOptions = {},
): Promise<readonly SourceReferenceLocation[]> {
  const targets = await resolveSourceReferenceTargetsAsync(ctx, params, deps, options);
  if (targets.length === 0) return [];

  const locations = (
    await Promise.all(
      targets.map((target) => resolveReferenceLocationsForTargetAsync(target, deps, options)),
    )
  ).flat();
  return dedupeLocations(locations);
}

function resolveSourceReferenceTargets(
  ctx: SourceExpressionContext,
  params: Pick<CursorParams, "documentUri" | "content" | "filePath" | "version">,
  deps: Pick<
    ProviderDeps,
    "analysisCache" | "styleDocumentForPath" | "typeResolver" | "workspaceRoot" | "settings"
  >,
  options: SourceReferencesQueryOptions,
): readonly SourceReferenceTarget[] {
  const selectedQueryBackend = resolveSelectedQueryBackendKind(options.env);
  if (usesRustSourceResolutionBackend(selectedQueryBackend)) {
    const rustTargets = resolveReferenceTargetsFromRust(
      ctx,
      params,
      deps,
      options.readRustSourceResolutionSelectorMatch ?? resolveRustSourceResolutionSelectorMatch,
    );
    if (rustTargets.length > 0) return rustTargets;
  }

  const resolution = readSourceExpressionResolution(
    {
      expression: ctx.expression,
      sourceFile: ctx.entry.sourceFile,
      styleDocument: ctx.styleDocument,
    },
    {
      styleDocumentForPath: deps.styleDocumentForPath,
      typeResolver: deps.typeResolver,
      filePath: params.filePath,
      workspaceRoot: deps.workspaceRoot,
      sourceBinder: ctx.entry.sourceBinder,
      sourceBindingGraph: ctx.entry.sourceBindingGraph,
      classValueUniverses: ctx.entry.classValueUniverses,
    },
  );
  if (!resolution.styleDocument || resolution.selectors.length === 0) return [];
  return dedupeTargets(
    resolution.selectors.map((selector) =>
      toSourceReferenceTarget(
        resolution.styleDocument!,
        resolution.styleDocument!.filePath,
        selector,
      ),
    ),
  );
}

async function resolveSourceReferenceTargetsAsync(
  ctx: SourceExpressionContext,
  params: Pick<CursorParams, "documentUri" | "content" | "filePath" | "version">,
  deps: Pick<
    ProviderDeps,
    "analysisCache" | "styleDocumentForPath" | "typeResolver" | "workspaceRoot" | "settings"
  >,
  options: SourceReferencesQueryOptions,
): Promise<readonly SourceReferenceTarget[]> {
  const selectedQueryBackend = resolveSelectedQueryBackendKind(options.env);
  if (usesRustSourceResolutionBackend(selectedQueryBackend)) {
    const rustTargets = await resolveReferenceTargetsFromRustAsync(
      ctx,
      params,
      deps,
      options.readRustSourceResolutionSelectorMatchAsync ??
        resolveRustSourceResolutionSelectorMatchAsync,
      options,
    );
    if (rustTargets.length > 0) return rustTargets;
  }

  const resolution = readSourceExpressionResolution(
    {
      expression: ctx.expression,
      sourceFile: ctx.entry.sourceFile,
      styleDocument: ctx.styleDocument,
    },
    {
      styleDocumentForPath: deps.styleDocumentForPath,
      typeResolver: deps.typeResolver,
      filePath: params.filePath,
      workspaceRoot: deps.workspaceRoot,
      sourceBinder: ctx.entry.sourceBinder,
      sourceBindingGraph: ctx.entry.sourceBindingGraph,
      classValueUniverses: ctx.entry.classValueUniverses,
    },
  );
  if (!resolution.styleDocument || resolution.selectors.length === 0) return [];
  return dedupeTargets(
    resolution.selectors.map((selector) =>
      toSourceReferenceTarget(
        resolution.styleDocument!,
        resolution.styleDocument!.filePath,
        selector,
      ),
    ),
  );
}

function resolveReferenceTargetsFromRust(
  ctx: SourceExpressionContext,
  params: Pick<CursorParams, "documentUri" | "content" | "filePath" | "version">,
  deps: Pick<
    ProviderDeps,
    "analysisCache" | "styleDocumentForPath" | "typeResolver" | "workspaceRoot" | "settings"
  >,
  readRustSelectorMatch: typeof resolveRustSourceResolutionSelectorMatch,
): readonly SourceReferenceTarget[] {
  const match = readRustSelectorMatch(
    {
      uri: params.documentUri,
      content: params.content,
      filePath: params.filePath,
      version: params.version,
    },
    ctx.expression.id,
    ctx.expression.scssModulePath,
    deps,
  );
  if (!match || match.selectorNames.length === 0) return [];

  const styleDocument = deps.styleDocumentForPath(match.styleFilePath);
  if (!styleDocument) return [];

  return dedupeTargets(
    match.selectorNames
      .map((name) => {
        const selector =
          styleDocument.selectors.find((candidate) => candidate.canonicalName === name) ?? null;
        return selector
          ? toSourceReferenceTarget(styleDocument, match.styleFilePath, selector)
          : null;
      })
      .filter((target): target is SourceReferenceTarget => target !== null),
  );
}

async function resolveReferenceTargetsFromRustAsync(
  ctx: SourceExpressionContext,
  params: Pick<CursorParams, "documentUri" | "content" | "filePath" | "version">,
  deps: Pick<
    ProviderDeps,
    "analysisCache" | "styleDocumentForPath" | "typeResolver" | "workspaceRoot" | "settings"
  >,
  readRustSelectorMatch: typeof resolveRustSourceResolutionSelectorMatchAsync,
  options: SourceReferencesQueryOptions,
): Promise<readonly SourceReferenceTarget[]> {
  const match = await readRustSelectorMatch(
    {
      uri: params.documentUri,
      content: params.content,
      filePath: params.filePath,
      version: params.version,
    },
    ctx.expression.id,
    ctx.expression.scssModulePath,
    deps,
    options.runRustSelectedQueryBackendJsonAsync,
  );
  if (!match || match.selectorNames.length === 0) return [];

  const styleDocument = deps.styleDocumentForPath(match.styleFilePath);
  if (!styleDocument) return [];

  return dedupeTargets(
    match.selectorNames
      .map((name) => {
        const selector =
          styleDocument.selectors.find((candidate) => candidate.canonicalName === name) ?? null;
        return selector
          ? toSourceReferenceTarget(styleDocument, match.styleFilePath, selector)
          : null;
      })
      .filter((target): target is SourceReferenceTarget => target !== null),
  );
}

function toSourceReferenceTarget(
  styleDocument: StyleDocumentHIR,
  styleFilePath: string,
  selector: SelectorDeclHIR,
): SourceReferenceTarget {
  return {
    filePath: styleFilePath,
    canonicalName: findCanonicalSelector(styleDocument, selector).canonicalName,
  };
}

async function resolveReferenceLocationsForTargetAsync(
  target: SourceReferenceTarget,
  deps: Pick<
    ProviderDeps,
    | "analysisCache"
    | "semanticReferenceIndex"
    | "settings"
    | "styleDependencyGraph"
    | "styleDocumentForPath"
    | "typeResolver"
    | "workspaceRoot"
    | "readStyleFile"
  >,
  options: SourceReferencesQueryOptions,
): Promise<readonly SourceReferenceLocation[]> {
  const graphReferences = await resolveRustStyleSelectorReferenceSummaryForWorkspaceTargetAsync(
    target,
    deps,
    options,
  );
  if (graphReferences?.hasAnyReferences) {
    return graphReferences.sites.map((site) => ({
      uri: pathToFileUrl(site.filePath),
      range: site.range,
    }));
  }

  const selectedQueryBackend = resolveSelectedQueryBackendKind(options.env);
  if (usesRustSelectorUsageBackend(selectedQueryBackend)) {
    const selectorUsagePayloadCache =
      options.selectorUsagePayloadCache ??
      (
        deps as {
          readonly selectorUsagePayloadCache?: SelectorUsagePayloadCache;
        }
      ).selectorUsagePayloadCache;
    const payload = await (
      options.readRustSelectorUsagePayloadForWorkspaceTargetAsync ??
      resolveRustSelectorUsagePayloadForWorkspaceTargetAsync
    )(
      {
        workspaceRoot: deps.workspaceRoot,
        classnameTransform: deps.settings.scss.classnameTransform,
        pathAlias: deps.settings.pathAlias,
      },
      deps,
      target.filePath,
      target.canonicalName,
      selectorUsagePayloadCache,
      options.runRustSelectedQueryBackendJsonAsync,
    );
    const rustLocations =
      payload &&
      buildSelectorUsageLocationsFromRustPayload(payload)?.map((site) => ({
        uri: pathToFileUrl(site.filePath),
        range: site.range,
      }));
    if (rustLocations) return rustLocations;
  }
  if (graphReferences) return [];

  return resolveSelectorReferenceLocations(deps, target);
}

function resolveReferenceLocationsForTarget(
  target: SourceReferenceTarget,
  deps: Pick<
    ProviderDeps,
    | "analysisCache"
    | "semanticReferenceIndex"
    | "settings"
    | "styleDependencyGraph"
    | "styleDocumentForPath"
    | "typeResolver"
    | "workspaceRoot"
    | "readStyleFile"
  >,
  options: SourceReferencesQueryOptions,
): readonly SourceReferenceLocation[] {
  const graphReferences = resolveRustStyleSelectorReferenceSummaryForWorkspaceTarget(
    target,
    deps,
    options,
  );
  if (graphReferences?.hasAnyReferences) {
    return graphReferences.sites.map((site) => ({
      uri: pathToFileUrl(site.filePath),
      range: site.range,
    }));
  }

  const selectedQueryBackend = resolveSelectedQueryBackendKind(options.env);
  if (usesRustSelectorUsageBackend(selectedQueryBackend)) {
    const selectorUsagePayloadCache =
      options.selectorUsagePayloadCache ??
      (
        deps as {
          readonly selectorUsagePayloadCache?: SelectorUsagePayloadCache;
        }
      ).selectorUsagePayloadCache;
    const payload = (
      options.readRustSelectorUsagePayloadForWorkspaceTarget ??
      resolveRustSelectorUsagePayloadForWorkspaceTarget
    )(
      {
        workspaceRoot: deps.workspaceRoot,
        classnameTransform: deps.settings.scss.classnameTransform,
        pathAlias: deps.settings.pathAlias,
      },
      deps,
      target.filePath,
      target.canonicalName,
      selectorUsagePayloadCache,
    );
    const rustLocations =
      payload &&
      buildSelectorUsageLocationsFromRustPayload(payload)?.map((site) => ({
        uri: pathToFileUrl(site.filePath),
        range: site.range,
      }));
    if (rustLocations) return rustLocations;
  }
  if (graphReferences) return [];

  return resolveSelectorReferenceLocations(deps, target);
}

function dedupeTargets(
  targets: readonly SourceReferenceTarget[],
): readonly SourceReferenceTarget[] {
  const unique = new Map<string, SourceReferenceTarget>();
  for (const target of targets) {
    const key = `${target.filePath}:${target.canonicalName}`;
    if (!unique.has(key)) unique.set(key, target);
  }
  return [...unique.values()];
}

function dedupeLocations(
  locations: readonly SourceReferenceLocation[],
): readonly SourceReferenceLocation[] {
  const unique = new Map<string, SourceReferenceLocation>();
  for (const location of locations) {
    const key = [
      location.uri,
      location.range.start.line,
      location.range.start.character,
      location.range.end.line,
      location.range.end.character,
    ].join(":");
    if (!unique.has(key)) unique.set(key, location);
  }
  return [...unique.values()];
}
