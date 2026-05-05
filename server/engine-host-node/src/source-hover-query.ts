import type {
  DynamicHoverExplanation,
  SelectorStyleDependencySummary,
  SourceExpressionContext,
} from "../../engine-core-ts/src/core/query";
import {
  buildDynamicExpressionExplanation,
  findCanonicalSelector,
  findCanonicalSelectorsByName,
  readSelectorStyleDependencySummary,
  resolveRefDetails,
} from "../../engine-core-ts/src/core/query";
import type { CursorParams, ProviderDeps } from "../../engine-core-ts/src/provider-deps";
import {
  buildExpressionSemanticsSummaryFromRustPayload,
  resolveRustExpressionSemanticsPayloadAsync,
  resolveRustExpressionSemanticsPayload,
} from "./expression-semantics-query-backend";
import {
  indexExpressionDomainSelectorProjectionsForStyle,
  resolveRustExpressionDomainSelectorProjections,
  resolveRustExpressionDomainSelectorProjectionsAsync,
  type ExpressionDomainSelectorProjectionEntryV0,
  withExpressionDomainSelectorProjection,
} from "./expression-domain-selector-projection-query-backend";
import {
  resolveRustSourceResolutionSelectorMatchAsync,
  resolveRustSourceResolutionSelectorMatch,
  resolveSelectedQueryBackendKind,
  usesRustExpressionSemanticsBackend,
  usesRustSourceResolutionBackend,
} from "./source-resolution-query-backend";
import type { RustSelectedQueryBackendJsonRunnerAsync } from "./selected-query-backend";

export interface SourceHoverQueryOptions {
  readonly env?: NodeJS.ProcessEnv;
  readonly readRustSourceResolutionSelectorMatch?: typeof resolveRustSourceResolutionSelectorMatch;
  readonly readRustExpressionSemanticsPayload?: typeof resolveRustExpressionSemanticsPayload;
  readonly readRustExpressionDomainSelectorProjections?: typeof resolveRustExpressionDomainSelectorProjections;
  readonly readRustExpressionDomainSelectorProjectionsAsync?: typeof resolveRustExpressionDomainSelectorProjectionsAsync;
  readonly runRustSelectedQueryBackendJsonAsync?: RustSelectedQueryBackendJsonRunnerAsync;
}

export interface SourceHoverResult {
  readonly selectors: ReturnType<typeof resolveRefDetails>["selectors"];
  readonly dynamicExplanation: DynamicHoverExplanation | null;
  readonly styleDependenciesBySelector: ReadonlyMap<string, SelectorStyleDependencySummary>;
}

export function resolveSourceExpressionHoverResult(
  ctx: SourceExpressionContext,
  params: Pick<CursorParams, "documentUri" | "content" | "filePath" | "version">,
  deps: Pick<
    ProviderDeps,
    | "analysisCache"
    | "styleDocumentForPath"
    | "typeResolver"
    | "workspaceRoot"
    | "styleDependencyGraph"
    | "settings"
  >,
  options: SourceHoverQueryOptions = {},
): SourceHoverResult {
  const backend = resolveSelectedQueryBackendKind(options.env);
  if (usesRustExpressionSemanticsBackend(backend)) {
    const rustResult = resolveHoverFromRustExpressionSemantics(
      ctx,
      params,
      deps,
      options.readRustExpressionSemanticsPayload ?? resolveRustExpressionSemanticsPayload,
      backend === "rust-selected-query"
        ? (options.readRustExpressionDomainSelectorProjections ??
            resolveRustExpressionDomainSelectorProjections)
        : null,
    );
    if (rustResult && rustResult.selectors.length > 0) return rustResult;
  }
  if (usesRustSourceResolutionBackend(backend)) {
    const rustSelectors = resolveSelectorsFromRustSourceResolution(
      ctx,
      params,
      deps,
      options.readRustSourceResolutionSelectorMatch ?? resolveRustSourceResolutionSelectorMatch,
    );
    if (rustSelectors) return buildSourceHoverResult(ctx, deps, rustSelectors, null);
  }

  const result = resolveRefDetails(ctx, {
    styleDocumentForPath: deps.styleDocumentForPath,
    typeResolver: deps.typeResolver,
    filePath: params.filePath,
    workspaceRoot: deps.workspaceRoot,
  });

  return buildSourceHoverResult(ctx, deps, result.selectors, result.dynamicExplanation);
}

export async function resolveSourceExpressionHoverResultAsync(
  ctx: SourceExpressionContext,
  params: Pick<CursorParams, "documentUri" | "content" | "filePath" | "version">,
  deps: Pick<
    ProviderDeps,
    | "analysisCache"
    | "styleDocumentForPath"
    | "typeResolver"
    | "workspaceRoot"
    | "styleDependencyGraph"
    | "settings"
  >,
  options: SourceHoverQueryOptions = {},
): Promise<SourceHoverResult> {
  const backend = resolveSelectedQueryBackendKind(options.env);
  if (usesRustExpressionSemanticsBackend(backend)) {
    const rustResult = await resolveHoverFromRustExpressionSemanticsAsync(
      ctx,
      params,
      deps,
      backend === "rust-selected-query"
        ? (options.readRustExpressionDomainSelectorProjectionsAsync ??
            resolveRustExpressionDomainSelectorProjectionsAsync)
        : null,
      options.runRustSelectedQueryBackendJsonAsync,
    );
    if (rustResult && rustResult.selectors.length > 0) return rustResult;
  }
  if (usesRustSourceResolutionBackend(backend)) {
    const rustSelectors = await resolveSelectorsFromRustSourceResolutionAsync(
      ctx,
      params,
      deps,
      options.runRustSelectedQueryBackendJsonAsync,
    );
    if (rustSelectors) return buildSourceHoverResult(ctx, deps, rustSelectors, null);
  }

  const result = resolveRefDetails(ctx, {
    styleDocumentForPath: deps.styleDocumentForPath,
    typeResolver: deps.typeResolver,
    filePath: params.filePath,
    workspaceRoot: deps.workspaceRoot,
  });

  return buildSourceHoverResult(ctx, deps, result.selectors, result.dynamicExplanation);
}

function buildSourceHoverResult(
  ctx: SourceExpressionContext,
  deps: Pick<ProviderDeps, "styleDependencyGraph">,
  selectors: ReturnType<typeof resolveRefDetails>["selectors"],
  dynamicExplanation: DynamicHoverExplanation | null,
): SourceHoverResult {
  return {
    selectors,
    dynamicExplanation,
    styleDependenciesBySelector: new Map(
      selectors.map((selector) => [
        selector.canonicalName,
        readSelectorStyleDependencySummary(
          deps.styleDependencyGraph,
          ctx.expression.scssModulePath,
          selector.canonicalName,
        ),
      ]),
    ),
  };
}

function resolveHoverFromRustExpressionSemantics(
  ctx: SourceExpressionContext,
  params: Pick<CursorParams, "documentUri" | "content" | "filePath" | "version">,
  deps: Pick<
    ProviderDeps,
    | "analysisCache"
    | "styleDocumentForPath"
    | "typeResolver"
    | "workspaceRoot"
    | "styleDependencyGraph"
    | "settings"
  >,
  readRustSemanticsPayload: typeof resolveRustExpressionSemanticsPayload,
  readRustSelectorProjections: typeof resolveRustExpressionDomainSelectorProjections | null,
): SourceHoverResult | null {
  const document = {
    uri: params.documentUri,
    content: params.content,
    filePath: params.filePath,
    version: params.version,
  };
  const rawPayload = readRustSemanticsPayload(
    document,
    ctx.expression.id,
    ctx.expression.scssModulePath,
    deps,
  );
  const projection = readRustSelectorProjections
    ? readExpressionProjection(
        readRustSelectorProjections(document, ctx.expression.scssModulePath, deps),
        ctx.expression.id,
        ctx.expression.scssModulePath,
      )
    : null;
  const payload = rawPayload
    ? withExpressionDomainSelectorProjection(rawPayload, projection)
    : rawPayload;
  if (!payload || !payload.styleFilePath) return null;

  const styleDocument = deps.styleDocumentForPath(payload.styleFilePath);
  if (!styleDocument) return null;

  const selectors = payload.selectorNames.flatMap((name) => {
    const selectorsForName = findCanonicalSelectorsByName(styleDocument, name);
    if (selectorsForName.length > 0) return selectorsForName;
    const selector =
      styleDocument.selectors.find((candidate) => candidate.canonicalName === name) ?? null;
    return selector ? [findCanonicalSelector(styleDocument, selector)] : [];
  });
  const semantics = buildExpressionSemanticsSummaryFromRustPayload(
    ctx.expression,
    styleDocument,
    selectors,
    payload,
  );

  return {
    selectors,
    dynamicExplanation: buildDynamicExpressionExplanation(ctx.expression, semantics),
    styleDependenciesBySelector: new Map(
      selectors.map((selector) => [
        selector.canonicalName,
        readSelectorStyleDependencySummary(
          deps.styleDependencyGraph,
          ctx.expression.scssModulePath,
          selector.canonicalName,
        ),
      ]),
    ),
  };
}

async function resolveHoverFromRustExpressionSemanticsAsync(
  ctx: SourceExpressionContext,
  params: Pick<CursorParams, "documentUri" | "content" | "filePath" | "version">,
  deps: Pick<
    ProviderDeps,
    | "analysisCache"
    | "styleDocumentForPath"
    | "typeResolver"
    | "workspaceRoot"
    | "styleDependencyGraph"
    | "settings"
  >,
  readRustSelectorProjections: typeof resolveRustExpressionDomainSelectorProjectionsAsync | null,
  runJson?: RustSelectedQueryBackendJsonRunnerAsync,
): Promise<SourceHoverResult | null> {
  const document = {
    uri: params.documentUri,
    content: params.content,
    filePath: params.filePath,
    version: params.version,
  };
  const rawPayload = await resolveRustExpressionSemanticsPayloadAsync(
    document,
    ctx.expression.id,
    ctx.expression.scssModulePath,
    deps,
    runJson,
  );
  const projection = readRustSelectorProjections
    ? readExpressionProjection(
        await readRustSelectorProjections(document, ctx.expression.scssModulePath, deps, runJson),
        ctx.expression.id,
        ctx.expression.scssModulePath,
      )
    : null;
  const payload = rawPayload
    ? withExpressionDomainSelectorProjection(rawPayload, projection)
    : rawPayload;
  if (!payload || !payload.styleFilePath) return null;

  const styleDocument = deps.styleDocumentForPath(payload.styleFilePath);
  if (!styleDocument) return null;

  const selectors = payload.selectorNames.flatMap((name) => {
    const selectorsForName = findCanonicalSelectorsByName(styleDocument, name);
    if (selectorsForName.length > 0) return selectorsForName;
    const selector =
      styleDocument.selectors.find((candidate) => candidate.canonicalName === name) ?? null;
    return selector ? [findCanonicalSelector(styleDocument, selector)] : [];
  });
  const semantics = buildExpressionSemanticsSummaryFromRustPayload(
    ctx.expression,
    styleDocument,
    selectors,
    payload,
  );

  return {
    selectors,
    dynamicExplanation: buildDynamicExpressionExplanation(ctx.expression, semantics),
    styleDependenciesBySelector: new Map(
      selectors.map((selector) => [
        selector.canonicalName,
        readSelectorStyleDependencySummary(
          deps.styleDependencyGraph,
          ctx.expression.scssModulePath,
          selector.canonicalName,
        ),
      ]),
    ),
  };
}

function readExpressionProjection(
  projections: readonly ExpressionDomainSelectorProjectionEntryV0[],
  expressionId: string,
  scssModulePath: string,
): ExpressionDomainSelectorProjectionEntryV0 | null {
  return (
    indexExpressionDomainSelectorProjectionsForStyle(projections, scssModulePath).get(
      expressionId,
    ) ?? null
  );
}

function resolveSelectorsFromRustSourceResolution(
  ctx: SourceExpressionContext,
  params: Pick<CursorParams, "documentUri" | "content" | "filePath" | "version">,
  deps: Pick<
    ProviderDeps,
    "analysisCache" | "styleDocumentForPath" | "typeResolver" | "workspaceRoot" | "settings"
  >,
  readRustSelectorMatch: typeof resolveRustSourceResolutionSelectorMatch,
): ReturnType<typeof resolveRefDetails>["selectors"] | null {
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
  if (!match) return null;
  const styleDocument = deps.styleDocumentForPath(match.styleFilePath);
  if (!styleDocument || match.selectorNames.length === 0) return null;

  return match.selectorNames.flatMap((name) => {
    const selectorsForName = findCanonicalSelectorsByName(styleDocument, name);
    if (selectorsForName.length > 0) return selectorsForName;
    const selector =
      styleDocument.selectors.find((candidate) => candidate.canonicalName === name) ?? null;
    return selector ? [findCanonicalSelector(styleDocument, selector)] : [];
  });
}

async function resolveSelectorsFromRustSourceResolutionAsync(
  ctx: SourceExpressionContext,
  params: Pick<CursorParams, "documentUri" | "content" | "filePath" | "version">,
  deps: Pick<
    ProviderDeps,
    "analysisCache" | "styleDocumentForPath" | "typeResolver" | "workspaceRoot" | "settings"
  >,
  runJson?: RustSelectedQueryBackendJsonRunnerAsync,
): Promise<ReturnType<typeof resolveRefDetails>["selectors"] | null> {
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
  if (!match) return null;
  const styleDocument = deps.styleDocumentForPath(match.styleFilePath);
  if (!styleDocument || match.selectorNames.length === 0) return null;

  return match.selectorNames.flatMap((name) => {
    const selectorsForName = findCanonicalSelectorsByName(styleDocument, name);
    if (selectorsForName.length > 0) return selectorsForName;
    const selector =
      styleDocument.selectors.find((candidate) => candidate.canonicalName === name) ?? null;
    return selector ? [findCanonicalSelector(styleDocument, selector)] : [];
  });
}
