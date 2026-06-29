import type { Range } from "@omena/shared";
import type {
  KeyframesDeclHIR,
  SelectorDeclHIR,
  ValueDeclHIR,
} from "../../engine-core-ts/src/core/hir/style-types";
import {
  findCanonicalSelector,
  findCanonicalSelectorsByName,
  readSourceExpressionResolution,
  type SourceExpressionContext,
} from "../../engine-core-ts/src/core/query";
import type { CursorParams, ProviderDeps } from "../../engine-core-ts/src/provider-deps";
import {
  resolveRustSourceResolutionSelectorMatchAsync,
  resolveRustSourceResolutionSelectorMatch,
  resolveSelectedQueryBackendKind,
  usesRustSourceResolutionBackend,
} from "./source-resolution-query-backend";
import type { RustSelectedQueryBackendJsonRunnerAsync } from "./selected-query-backend";
import { resolveSymbolValuesFromRustControlFlow } from "./type-fact-control-flow-graph";

export interface SourceDefinitionTarget {
  readonly originRange: Range;
  readonly targetFilePath: string;
  readonly targetRange: Range;
  readonly targetSelectionRange: Range;
}

export interface SourceDefinitionQueryOptions {
  readonly env?: NodeJS.ProcessEnv;
  readonly readRustSourceResolutionSelectorMatch?: typeof resolveRustSourceResolutionSelectorMatch;
  readonly runRustSelectedQueryBackendJsonAsync?: RustSelectedQueryBackendJsonRunnerAsync;
}

export function resolveSourceExpressionDefinitionTargets(
  ctx: SourceExpressionContext,
  params: Pick<CursorParams, "documentUri" | "content" | "filePath" | "version">,
  deps: Pick<
    ProviderDeps,
    "analysisCache" | "styleDocumentForPath" | "typeResolver" | "workspaceRoot" | "settings"
  >,
  options: SourceDefinitionQueryOptions = {},
): readonly SourceDefinitionTarget[] {
  const backend = resolveSelectedQueryBackendKind(options.env);
  const localExactTargets = resolveExactSourceDefinitionTargetsFromTypescript(ctx, params, deps);
  if (localExactTargets.length > 0) return localExactTargets;

  if (usesRustSourceResolutionBackend(backend)) {
    const rustTargets = resolveSourceDefinitionTargetsFromRust(
      ctx,
      params,
      deps,
      options.readRustSourceResolutionSelectorMatch ?? resolveRustSourceResolutionSelectorMatch,
    );
    if (rustTargets.length > 0) return rustTargets;
  }

  return resolveSourceDefinitionTargetsFromTypescript(ctx, params, deps);
}

export async function resolveSourceExpressionDefinitionTargetsAsync(
  ctx: SourceExpressionContext,
  params: Pick<CursorParams, "documentUri" | "content" | "filePath" | "version">,
  deps: Pick<
    ProviderDeps,
    "analysisCache" | "styleDocumentForPath" | "typeResolver" | "workspaceRoot" | "settings"
  >,
  options: SourceDefinitionQueryOptions = {},
): Promise<readonly SourceDefinitionTarget[]> {
  const backend = resolveSelectedQueryBackendKind(options.env);
  const localExactTargets = resolveExactSourceDefinitionTargetsFromTypescript(ctx, params, deps);
  if (localExactTargets.length > 0) return localExactTargets;

  if (usesRustSourceResolutionBackend(backend)) {
    const rustTargets = await resolveSourceDefinitionTargetsFromRustAsync(
      ctx,
      params,
      deps,
      options.runRustSelectedQueryBackendJsonAsync,
    );
    if (rustTargets.length > 0) return rustTargets;
  }

  return resolveSourceDefinitionTargetsFromTypescript(ctx, params, deps);
}

function resolveExactSourceDefinitionTargetsFromTypescript(
  ctx: SourceExpressionContext,
  params: Pick<CursorParams, "content" | "filePath">,
  deps: Pick<ProviderDeps, "styleDocumentForPath" | "typeResolver" | "workspaceRoot">,
): readonly SourceDefinitionTarget[] {
  if (ctx.expression.kind !== "literal" && ctx.expression.kind !== "styleAccess") return [];
  return resolveSourceDefinitionTargetsFromTypescript(ctx, params, deps);
}

function resolveSourceDefinitionTargetsFromTypescript(
  ctx: SourceExpressionContext,
  params: Pick<CursorParams, "content" | "filePath">,
  deps: Pick<ProviderDeps, "styleDocumentForPath" | "typeResolver" | "workspaceRoot">,
): readonly SourceDefinitionTarget[] {
  const resolution = readSourceExpressionResolution(
    {
      expression: ctx.expression,
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
      resolveSymbolValues: (expression) =>
        resolveSymbolValuesFromRustControlFlow({
          source: params.content,
          sourcePath: params.filePath,
          expression,
        }),
    },
  );
  const styleDocument = resolution.styleDocument;
  if (!styleDocument || resolution.selectors.length === 0) return [];
  return resolution.selectors.map((selector) =>
    toSourceDefinitionTarget(ctx.expression.range, styleDocument.filePath, selector),
  );
}

async function resolveSourceDefinitionTargetsFromRustAsync(
  ctx: SourceExpressionContext,
  params: Pick<CursorParams, "documentUri" | "content" | "filePath" | "version">,
  deps: Pick<
    ProviderDeps,
    "analysisCache" | "styleDocumentForPath" | "typeResolver" | "workspaceRoot" | "settings"
  >,
  runJson?: RustSelectedQueryBackendJsonRunnerAsync,
): Promise<readonly SourceDefinitionTarget[]> {
  const match = await resolveRustSourceResolutionSelectorMatchAsync(
    {
      uri: params.documentUri,
      content: params.content,
      filePath: params.filePath,
      version: params.version,
    },
    ctx.expression.id,
    ctx.expression.scssModulePath,
    deps,
    runJson,
  );
  if (!match) return [];
  const styleDocument = deps.styleDocumentForPath(match.styleFilePath);
  if (!styleDocument || match.selectorNames.length === 0) return [];
  return match.selectorNames
    .flatMap((name) => {
      const selectors = findCanonicalSelectorsByName(styleDocument, name);
      if (selectors.length > 0) return selectors;
      const selector =
        styleDocument.selectors.find((candidate) => candidate.canonicalName === name) ?? null;
      return selector ? [findCanonicalSelector(styleDocument, selector)] : [];
    })
    .map((selector) =>
      toSourceDefinitionTarget(ctx.expression.range, match.styleFilePath, selector),
    );
}

function resolveSourceDefinitionTargetsFromRust(
  ctx: SourceExpressionContext,
  params: Pick<CursorParams, "documentUri" | "content" | "filePath" | "version">,
  deps: Pick<
    ProviderDeps,
    "analysisCache" | "styleDocumentForPath" | "typeResolver" | "workspaceRoot" | "settings"
  >,
  readRustSelectorMatch: typeof resolveRustSourceResolutionSelectorMatch,
): readonly SourceDefinitionTarget[] {
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
  if (!match) return [];
  const styleDocument = deps.styleDocumentForPath(match.styleFilePath);
  if (!styleDocument || match.selectorNames.length === 0) return [];
  return match.selectorNames
    .flatMap((name) => {
      const selectors = findCanonicalSelectorsByName(styleDocument, name);
      if (selectors.length > 0) return selectors;
      const selector =
        styleDocument.selectors.find((candidate) => candidate.canonicalName === name) ?? null;
      return selector ? [findCanonicalSelector(styleDocument, selector)] : [];
    })
    .map((selector) =>
      toSourceDefinitionTarget(ctx.expression.range, match.styleFilePath, selector),
    );
}

function toSourceDefinitionTarget(
  originRange: Range,
  targetFilePath: string,
  target: SelectorDeclHIR | KeyframesDeclHIR | ValueDeclHIR,
): SourceDefinitionTarget {
  return {
    originRange,
    targetFilePath,
    targetRange: target.ruleRange,
    targetSelectionRange: target.range,
  };
}
