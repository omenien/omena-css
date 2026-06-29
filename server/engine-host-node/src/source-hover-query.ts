import type {
  DynamicHoverExplanation,
  SelectorStyleDependencySummary,
  SourceExpressionContext,
} from "../../engine-core-ts/src/core/query";
import type { StyleDocumentHIR } from "../../engine-core-ts/src/core/hir/style-types";
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
  type resolveRustSourceResolutionSelectorMatch,
  resolveRustSourceResolutionPayload,
  resolveRustSourceResolutionPayloadAsync,
  buildSourceResolutionSummaryFromRustPayload,
  type SourceResolutionEvaluatorCandidatePayloadV0,
  resolveSelectedQueryBackendKind,
  usesRustExpressionSemanticsBackend,
  usesRustSourceResolutionBackend,
} from "./source-resolution-query-backend";
import type { ExpressionSemanticsSummary } from "../../engine-core-ts/src/core/query/read-expression-semantics";
import type { RustSelectedQueryBackendJsonRunnerAsync } from "./selected-query-backend";
import { resolveSymbolValuesFromRustControlFlow } from "./type-fact-control-flow-graph";

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
    if (shouldUseRustHoverResult(ctx, rustResult)) return rustResult;
  }
  if (usesRustSourceResolutionBackend(backend)) {
    const rustResult = resolveHoverFromRustSourceResolution(ctx, params, deps, options);
    if (rustResult) return rustResult;
  }

  const result = resolveRefDetails(ctx, {
    styleDocumentForPath: deps.styleDocumentForPath,
    typeResolver: deps.typeResolver,
    filePath: params.filePath,
    workspaceRoot: deps.workspaceRoot,
    resolveSymbolValues: (expression) =>
      resolveSymbolValuesFromRustControlFlow({
        source: params.content,
        sourcePath: params.filePath,
        expression,
      }),
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
    if (shouldUseRustHoverResult(ctx, rustResult)) return rustResult;
  }
  if (usesRustSourceResolutionBackend(backend)) {
    const rustResult = await resolveHoverFromRustSourceResolutionAsync(ctx, params, deps, options);
    if (rustResult) return rustResult;
  }

  const result = resolveRefDetails(ctx, {
    styleDocumentForPath: deps.styleDocumentForPath,
    typeResolver: deps.typeResolver,
    filePath: params.filePath,
    workspaceRoot: deps.workspaceRoot,
    resolveSymbolValues: (expression) =>
      resolveSymbolValuesFromRustControlFlow({
        source: params.content,
        sourcePath: params.filePath,
        expression,
      }),
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

function shouldUseRustHoverResult(
  ctx: SourceExpressionContext,
  result: SourceHoverResult | null,
): result is SourceHoverResult {
  if (!result || result.selectors.length === 0) return false;
  if (ctx.expression.kind === "literal" || ctx.expression.kind === "styleAccess") return true;
  return result.dynamicExplanation !== null;
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
  const projection =
    rawPayload && rawPayload.selectorNames.length === 0 && readRustSelectorProjections
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
  const projection =
    rawPayload && rawPayload.selectorNames.length === 0 && readRustSelectorProjections
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

function resolveHoverFromRustSourceResolution(
  ctx: SourceExpressionContext,
  params: Pick<CursorParams, "documentUri" | "content" | "filePath" | "version">,
  deps: Pick<
    ProviderDeps,
    | "analysisCache"
    | "styleDependencyGraph"
    | "styleDocumentForPath"
    | "typeResolver"
    | "workspaceRoot"
    | "settings"
  >,
  options: SourceHoverQueryOptions,
): SourceHoverResult | null {
  const document = {
    uri: params.documentUri,
    content: params.content,
    filePath: params.filePath,
    version: params.version,
  };
  if (!options.readRustSourceResolutionSelectorMatch) {
    const payload = resolveRustSourceResolutionPayload(
      document,
      ctx.expression.id,
      ctx.expression.scssModulePath,
      deps,
    );
    return buildHoverFromRustSourceResolutionPayload(ctx, deps, payload);
  }

  const match = options.readRustSourceResolutionSelectorMatch(
    document,
    ctx.expression.id,
    ctx.expression.scssModulePath,
    deps,
  );
  if (!match) return null;
  const styleDocument = deps.styleDocumentForPath(match.styleFilePath);
  if (!styleDocument || match.selectorNames.length === 0) return null;

  return buildSourceHoverResult(
    ctx,
    deps,
    resolveSelectorsByNames(styleDocument, match.selectorNames),
    null,
  );
}

async function resolveHoverFromRustSourceResolutionAsync(
  ctx: SourceExpressionContext,
  params: Pick<CursorParams, "documentUri" | "content" | "filePath" | "version">,
  deps: Pick<
    ProviderDeps,
    | "analysisCache"
    | "styleDependencyGraph"
    | "styleDocumentForPath"
    | "typeResolver"
    | "workspaceRoot"
    | "settings"
  >,
  options: SourceHoverQueryOptions,
): Promise<SourceHoverResult | null> {
  const document = {
    uri: params.documentUri,
    content: params.content,
    filePath: params.filePath,
    version: params.version,
  };
  if (!options.readRustSourceResolutionSelectorMatch) {
    const payload = await resolveRustSourceResolutionPayloadAsync(
      document,
      ctx.expression.id,
      ctx.expression.scssModulePath,
      deps,
      options.runRustSelectedQueryBackendJsonAsync,
    );
    return buildHoverFromRustSourceResolutionPayload(ctx, deps, payload);
  }

  const match = await resolveRustSourceResolutionSelectorMatchAsync(
    document,
    ctx.expression.id,
    ctx.expression.scssModulePath,
    deps,
    options.runRustSelectedQueryBackendJsonAsync,
  );
  if (!match) return null;
  const styleDocument = deps.styleDocumentForPath(match.styleFilePath);
  if (!styleDocument || match.selectorNames.length === 0) return null;

  return buildSourceHoverResult(
    ctx,
    deps,
    resolveSelectorsByNames(styleDocument, match.selectorNames),
    null,
  );
}

function buildHoverFromRustSourceResolutionPayload(
  ctx: SourceExpressionContext,
  deps: Pick<ProviderDeps, "styleDependencyGraph" | "styleDocumentForPath">,
  payload: SourceResolutionEvaluatorCandidatePayloadV0 | null,
): SourceHoverResult | null {
  if (!payload || !payload.styleFilePath || payload.selectorNames.length === 0) return null;
  const styleDocument = deps.styleDocumentForPath(payload.styleFilePath);
  if (!styleDocument) return null;
  const selectors = resolveSelectorsByNames(styleDocument, payload.selectorNames);
  if (selectors.length === 0) return null;
  const resolution = buildSourceResolutionSummaryFromRustPayload(styleDocument, selectors, payload);
  const semantics: ExpressionSemanticsSummary = {
    expression: ctx.expression,
    styleDocument: resolution.styleDocument,
    selectors: resolution.selectors,
    selectorNames: resolution.selectors.map((selector) => selector.name),
    candidateNames: payload.finiteValues ?? payload.selectorNames,
    finiteValues: resolution.finiteValues,
    valueDomainKind: "top",
    ...(resolution.abstractValue ? { abstractValue: resolution.abstractValue } : {}),
    ...(resolution.valueCertainty ? { valueCertainty: resolution.valueCertainty } : {}),
    ...(resolution.reason ? { reason: resolution.reason } : {}),
    selectorCertainty: resolution.selectorCertainty,
  };
  return buildSourceHoverResult(
    ctx,
    deps,
    selectors,
    buildDynamicExpressionExplanation(ctx.expression, semantics),
  );
}

function resolveSelectorsByNames(
  styleDocument: StyleDocumentHIR,
  selectorNames: readonly string[],
): ReturnType<typeof resolveRefDetails>["selectors"] {
  return selectorNames.flatMap((name) => {
    const selectorsForName = findCanonicalSelectorsByName(styleDocument, name);
    if (selectorsForName.length > 0) return selectorsForName;
    const selector =
      styleDocument.selectors.find((candidate) => candidate.canonicalName === name) ?? null;
    return selector ? [findCanonicalSelector(styleDocument, selector)] : [];
  });
}
