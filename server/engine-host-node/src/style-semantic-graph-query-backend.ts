import path from "node:path";
import type { Range } from "@css-module-explainer/shared";
import type { EngineInputV2 } from "../../engine-core-ts/src/contracts";
import type { StyleDocumentHIR } from "../../engine-core-ts/src/core/hir/style-types";
import { parseStyleDocument } from "../../engine-core-ts/src/core/scss/scss-parser";
import type { ProviderDeps } from "../../engine-core-ts/src/provider-deps";
import { buildEngineInputV2 } from "./engine-input-v2";
import {
  collectSourceDocuments,
  resolveWorkspaceCheckFilesSync,
  type SourceDocumentSnapshot,
} from "./checker-host/workspace-check-support";
import {
  isEngineShadowRunnerCancelledError,
  SELECTED_QUERY_RUNNER_COMMANDS,
  runRustSelectedQueryBackendJson,
  runRustSelectedQueryBackendJsonAsync,
  type RustSelectedQueryBackendJsonRunnerAsync,
} from "./selected-query-backend";
import type { BuildSelectedQueryResultsV2Options } from "./engine-query-v2";

type RustJsonRunner = <T>(command: string, input: unknown) => T;
type RustJsonRunnerAsync = RustSelectedQueryBackendJsonRunnerAsync;
export type StyleSemanticGraphCache = Map<string, StyleSemanticGraphSummaryV0 | null>;

export interface StyleSemanticGraphSummaryV0 {
  readonly schemaVersion: "0";
  readonly product: "omena-semantic.style-semantic-graph";
  readonly language: string;
  readonly parserFacts: unknown;
  readonly semanticFacts: unknown;
  readonly designTokenSemantics?: StyleSemanticGraphDesignTokenSemanticsV0;
  readonly selectorIdentityEngine: StyleSemanticGraphSelectorIdentityEngineV0;
  readonly selectorReferenceEngine: StyleSemanticGraphSelectorReferenceEngineV0;
  readonly sourceInputEvidence: unknown;
  readonly promotionEvidence: unknown;
  readonly losslessCstContract: unknown;
}

export interface StyleSemanticGraphDesignTokenSemanticsV0 {
  readonly schemaVersion: "0";
  readonly product: "omena-semantic.design-token-semantics";
  readonly status: string;
  readonly resolutionScope: string;
  readonly declarationCount: number;
  readonly referenceCount: number;
  readonly resolvedReferenceCount: number;
  readonly unresolvedReferenceCount: number;
  readonly selectorsWithReferencesCount: number;
  readonly contextSignal: StyleSemanticGraphDesignTokenContextSignalV0;
  readonly resolutionSignal: StyleSemanticGraphDesignTokenResolutionSignalV0;
  readonly cascadeRankingSignal: StyleSemanticGraphDesignTokenCascadeRankingSignalV0;
  readonly capabilities: StyleSemanticGraphDesignTokenCapabilitiesV0;
  readonly blockingGaps: readonly string[];
  readonly nextPriorities: readonly string[];
}

export interface StyleSemanticGraphDesignTokenContextSignalV0 {
  readonly declarationContextSelectorCount: number;
  readonly declarationWrapperContextCount: number;
  readonly mediaContextSelectorCount: number;
  readonly supportsContextSelectorCount: number;
  readonly layerContextSelectorCount: number;
  readonly wrapperContextCount: number;
}

export interface StyleSemanticGraphDesignTokenResolutionSignalV0 {
  readonly declarationFactCount: number;
  readonly referenceFactCount: number;
  readonly sourceOrderedDeclarationCount: number;
  readonly sourceOrderedReferenceCount: number;
  readonly occurrenceResolvedReferenceCount: number;
  readonly occurrenceUnresolvedReferenceCount: number;
  readonly workspaceDeclarationFactCount?: number;
  readonly crossFileDeclarationFactCount?: number;
  readonly workspaceOccurrenceResolvedReferenceCount?: number;
  readonly workspaceOccurrenceUnresolvedReferenceCount?: number;
  readonly contextMatchedReferenceCount: number;
  readonly contextUnmatchedReferenceCount: number;
  readonly rootDeclarationCount: number;
  readonly selectorScopedDeclarationCount: number;
  readonly wrapperScopedDeclarationCount: number;
}

export interface StyleSemanticGraphDesignTokenCascadeRankingSignalV0 {
  readonly rankedReferenceCount: number;
  readonly unrankedReferenceCount: number;
  readonly sourceOrderWinnerDeclarationCount: number;
  readonly sourceOrderShadowedDeclarationCount: number;
  readonly repeatedNameDeclarationCount: number;
  readonly crossFileCandidateDeclarationCount?: number;
  readonly crossFileWinnerDeclarationCount?: number;
  readonly crossFileShadowedDeclarationCount?: number;
  readonly rankedReferences: readonly StyleSemanticGraphDesignTokenRankedReferenceV0[];
}

export interface StyleSemanticGraphDesignTokenRankedReferenceV0 {
  readonly referenceName: string;
  readonly referenceSourceOrder: number;
  readonly winnerDeclarationSourceOrder: number;
  readonly winnerDeclarationFilePath?: string;
  readonly winnerDeclarationRange?: Range;
  readonly winnerImportGraphDistance?: number;
  readonly winnerImportGraphOrder?: number;
  readonly shadowedDeclarationSourceOrders: readonly number[];
  readonly candidateDeclarationCount: number;
  readonly winnerContextKind?: string;
  readonly crossFileCandidateDeclarationCount?: number;
  readonly crossFileShadowedDeclarationCount?: number;
}

export interface StyleSemanticGraphDesignTokenCapabilitiesV0 {
  readonly sameFileResolutionReady: boolean;
  readonly wrapperContextSignalReady: boolean;
  readonly sourceOrderSignalReady: boolean;
  readonly sourceOrderCascadeRankingReady: boolean;
  readonly workspaceCascadeCandidateSignalReady?: boolean;
  readonly occurrenceResolutionSignalReady: boolean;
  readonly selectorContextResolutionReady: boolean;
  readonly themeOverrideContextSignalReady: boolean;
  readonly crossFileImportGraphReady: boolean;
  readonly crossPackageCascadeRankingReady: boolean;
  readonly themeOverrideContextReady: boolean;
}

export interface StyleSemanticGraphDesignTokenRankedReferenceReadModel {
  readonly referenceName: string;
  readonly referenceSourceOrder: number;
  readonly winnerDeclarationSourceOrder: number;
  readonly winnerDeclarationFilePath?: string;
  readonly winnerDeclarationRange?: Range;
  readonly winnerImportGraphDistance?: number;
  readonly winnerImportGraphOrder?: number;
  readonly crossFileCandidateScope?: string;
  readonly shadowedDeclarationSourceOrders: readonly number[];
  readonly candidateDeclarationCount: number;
  readonly winnerContextKind?: string;
  readonly crossFileCandidateDeclarationCount: number;
  readonly crossFileShadowedDeclarationCount: number;
  readonly reference?: StyleDocumentHIR["customPropertyRefs"][number];
  readonly winnerDeclaration?: StyleDocumentHIR["customPropertyDecls"][number];
  readonly shadowedDeclarations?: readonly StyleDocumentHIR["customPropertyDecls"][number][];
}

type MutableStyleSemanticGraphDesignTokenRankedReferenceReadModel = {
  -readonly [K in keyof StyleSemanticGraphDesignTokenRankedReferenceReadModel]: StyleSemanticGraphDesignTokenRankedReferenceReadModel[K];
};

export interface StyleSemanticGraphSelectorIdentityEngineV0 {
  readonly schemaVersion: "0";
  readonly product: "omena-semantic.selector-identity";
  readonly canonicalIdCount: number;
  readonly canonicalIds: readonly StyleSemanticGraphSelectorIdentityV0[];
  readonly rewriteSafety: {
    readonly allCanonicalIdsRewriteSafe: boolean;
    readonly safeCanonicalIds: readonly string[];
    readonly blockedCanonicalIds: readonly string[];
    readonly blockers: readonly string[];
  };
}

export interface StyleSemanticGraphSelectorIdentityV0 {
  readonly canonicalId: string;
  readonly localName: string;
  readonly identityKind: string;
  readonly rewriteSafety: "safe" | "blocked";
  readonly blockers: readonly string[];
}

export interface StyleSemanticGraphSelectorIdentityReadModel {
  readonly canonicalId: string;
  readonly canonicalName: string;
  readonly identityKind: string;
  readonly rewriteSafety: StyleSemanticGraphSelectorIdentityV0["rewriteSafety"];
  readonly blockers: readonly string[];
  readonly range: StyleDocumentHIR["selectors"][number]["range"];
  readonly ruleRange: StyleDocumentHIR["selectors"][number]["ruleRange"];
  readonly viewKind: StyleDocumentHIR["selectors"][number]["viewKind"];
}

export interface StyleSemanticGraphSelectorReferenceEngineV0 {
  readonly schemaVersion: "0";
  readonly product: "omena-semantic.selector-references";
  readonly stylePath: string | null;
  readonly selectorCount: number;
  readonly referencedSelectorCount: number;
  readonly unreferencedSelectorCount: number;
  readonly totalReferenceSites: number;
  readonly selectors: readonly StyleSemanticGraphSelectorReferenceSummaryV0[];
}

export interface StyleSemanticGraphSelectorReferenceSummaryV0 {
  readonly canonicalId: string;
  readonly filePath: string;
  readonly localName: string;
  readonly totalReferences: number;
  readonly directReferenceCount: number;
  readonly editableDirectReferenceCount: number;
  readonly exactReferenceCount: number;
  readonly inferredOrBetterReferenceCount: number;
  readonly hasExpandedReferences: boolean;
  readonly hasStyleDependencyReferences: boolean;
  readonly hasAnyReferences: boolean;
  readonly sites: readonly StyleSemanticGraphSelectorReferenceSiteV0[];
  readonly editableDirectSites: readonly StyleSemanticGraphSelectorEditableDirectSiteV0[];
}

export interface StyleSemanticGraphSelectorReferenceSiteV0 {
  readonly filePath: string;
  readonly range: Range;
  readonly expansion: string;
  readonly referenceKind: string;
}

export interface StyleSemanticGraphSelectorEditableDirectSiteV0 {
  readonly filePath: string;
  readonly range: Range;
  readonly className: string;
}

export interface StyleSemanticGraphRunnerInputV0 {
  readonly stylePath: string;
  readonly styleSource: string;
  readonly engineInput: EngineInputV2;
}

export interface StyleSemanticGraphBatchRunnerInputV0 {
  readonly styles: readonly StyleSemanticGraphBatchStyleInputV0[];
  readonly packageManifests?: readonly StyleSemanticGraphPackageManifestInputV0[];
  readonly engineInput: EngineInputV2;
}

export interface StyleSemanticGraphBatchStyleInputV0 {
  readonly stylePath: string;
  readonly styleSource: string;
}

export interface StyleSemanticGraphPackageManifestInputV0 {
  readonly packageJsonPath: string;
  readonly packageJsonSource: string;
}

export interface StyleSemanticGraphBatchRunnerOutputV0 {
  readonly schemaVersion: "0";
  readonly product: "omena-semantic.style-semantic-graph-batch";
  readonly graphs: readonly StyleSemanticGraphBatchEntryV0[];
}

export interface StyleSemanticGraphBatchEntryV0 {
  readonly stylePath: string;
  readonly graph: StyleSemanticGraphSummaryV0 | null;
}

type StyleSemanticGraphQueryBackendOptions = Pick<
  BuildSelectedQueryResultsV2Options,
  | "workspaceRoot"
  | "classnameTransform"
  | "pathAlias"
  | "sourceDocuments"
  | "styleFiles"
  | "analysisCache"
  | "styleDocumentForPath"
  | "typeResolver"
> & {
  readonly readStyleFile: ProviderDeps["readStyleFile"];
};

export interface StyleSemanticGraphQueryOptions {
  readonly runRustSelectedQueryBackendJson?: RustJsonRunner;
  readonly runRustSelectedQueryBackendJsonAsync?: RustJsonRunnerAsync;
  readonly engineInput?: EngineInputV2;
  readonly sourceDocuments?: readonly SourceDocumentSnapshot[];
  readonly styleFiles?: readonly string[];
  readonly styleSemanticGraphCache?: StyleSemanticGraphCache;
}

export function resolveRustStyleSemanticGraph(
  options: StyleSemanticGraphQueryBackendOptions,
  stylePath: string,
  queryOptions: StyleSemanticGraphQueryOptions = {},
): StyleSemanticGraphSummaryV0 | null {
  const cache = queryOptions.styleSemanticGraphCache;
  if (cache?.has(stylePath)) {
    return cache.get(stylePath) ?? null;
  }
  maybePopulateStyleSemanticGraphCacheFromBatch(options, queryOptions);
  if (cache?.has(stylePath)) {
    return cache.get(stylePath) ?? null;
  }

  const styleSource = options.readStyleFile(stylePath);
  if (styleSource === null) {
    cache?.set(stylePath, null);
    return null;
  }

  const engineInput =
    queryOptions.engineInput ??
    buildEngineInputV2({
      workspaceRoot: options.workspaceRoot,
      classnameTransform: options.classnameTransform,
      pathAlias: options.pathAlias,
      sourceDocuments: options.sourceDocuments,
      styleFiles: ensureStyleFileIncluded(options.styleFiles, stylePath),
      analysisCache: options.analysisCache,
      styleDocumentForPath: options.styleDocumentForPath,
      typeResolver: options.typeResolver,
    });

  let graph: StyleSemanticGraphSummaryV0 | null;
  try {
    graph = runRustStyleSemanticGraph(
      {
        stylePath,
        styleSource,
        engineInput,
      },
      queryOptions,
    );
  } catch (err) {
    if (!isEngineShadowRunnerCancelledError(err)) throw err;
    graph = null;
  }
  cache?.set(stylePath, graph);
  return graph;
}

export async function resolveRustStyleSemanticGraphAsync(
  options: StyleSemanticGraphQueryBackendOptions,
  stylePath: string,
  queryOptions: StyleSemanticGraphQueryOptions = {},
): Promise<StyleSemanticGraphSummaryV0 | null> {
  const cache = queryOptions.styleSemanticGraphCache;
  if (cache?.has(stylePath)) {
    return cache.get(stylePath) ?? null;
  }
  await maybePopulateStyleSemanticGraphCacheFromBatchAsync(options, queryOptions);
  if (cache?.has(stylePath)) {
    return cache.get(stylePath) ?? null;
  }

  const styleSource = options.readStyleFile(stylePath);
  if (styleSource === null) {
    cache?.set(stylePath, null);
    return null;
  }

  const engineInput =
    queryOptions.engineInput ??
    buildEngineInputV2({
      workspaceRoot: options.workspaceRoot,
      classnameTransform: options.classnameTransform,
      pathAlias: options.pathAlias,
      sourceDocuments: options.sourceDocuments,
      styleFiles: ensureStyleFileIncluded(options.styleFiles, stylePath),
      analysisCache: options.analysisCache,
      styleDocumentForPath: options.styleDocumentForPath,
      typeResolver: options.typeResolver,
    });

  let graph: StyleSemanticGraphSummaryV0 | null;
  try {
    graph = await runRustStyleSemanticGraphAsync(
      {
        stylePath,
        styleSource,
        engineInput,
      },
      queryOptions,
    );
  } catch (err) {
    if (!isEngineShadowRunnerCancelledError(err)) throw err;
    graph = null;
  }
  cache?.set(stylePath, graph);
  return graph;
}

export function resolveRustStyleSemanticGraphForWorkspaceTarget(
  args: {
    readonly workspaceRoot: string;
    readonly classnameTransform: BuildSelectedQueryResultsV2Options["classnameTransform"];
    readonly pathAlias: BuildSelectedQueryResultsV2Options["pathAlias"];
  },
  deps: Pick<
    ProviderDeps,
    "analysisCache" | "styleDocumentForPath" | "typeResolver" | "readStyleFile"
  >,
  stylePath: string,
  queryOptions: StyleSemanticGraphQueryOptions = {},
): StyleSemanticGraphSummaryV0 | null {
  const resolvedFiles =
    queryOptions.sourceDocuments && queryOptions.styleFiles
      ? null
      : resolveWorkspaceCheckFilesSync({
          workspaceRoot: args.workspaceRoot,
        });
  const sourceDocuments =
    queryOptions.sourceDocuments ??
    collectSourceDocuments(resolvedFiles?.sourceFiles ?? [], deps.analysisCache);
  const styleFiles = queryOptions.styleFiles ?? resolvedFiles?.styleFiles ?? [];
  const engineInput =
    queryOptions.engineInput ??
    (queryOptions.styleSemanticGraphCache && styleFiles.length > 1
      ? buildEngineInputV2({
          workspaceRoot: args.workspaceRoot,
          classnameTransform: args.classnameTransform,
          pathAlias: args.pathAlias,
          sourceDocuments,
          styleFiles,
          analysisCache: deps.analysisCache,
          styleDocumentForPath: deps.styleDocumentForPath,
          typeResolver: deps.typeResolver,
        })
      : undefined);
  const workspaceQueryOptions = {
    ...queryOptions,
    sourceDocuments,
    styleFiles,
    ...(engineInput ? { engineInput } : {}),
  };

  return resolveRustStyleSemanticGraph(
    {
      workspaceRoot: args.workspaceRoot,
      classnameTransform: args.classnameTransform,
      pathAlias: args.pathAlias,
      sourceDocuments,
      styleFiles,
      analysisCache: deps.analysisCache,
      styleDocumentForPath: deps.styleDocumentForPath,
      typeResolver: deps.typeResolver,
      readStyleFile: deps.readStyleFile,
    },
    stylePath,
    workspaceQueryOptions,
  );
}

export async function resolveRustStyleSemanticGraphForWorkspaceTargetAsync(
  args: {
    readonly workspaceRoot: string;
    readonly classnameTransform: BuildSelectedQueryResultsV2Options["classnameTransform"];
    readonly pathAlias: BuildSelectedQueryResultsV2Options["pathAlias"];
  },
  deps: Pick<
    ProviderDeps,
    "analysisCache" | "styleDocumentForPath" | "typeResolver" | "readStyleFile"
  >,
  stylePath: string,
  queryOptions: StyleSemanticGraphQueryOptions = {},
): Promise<StyleSemanticGraphSummaryV0 | null> {
  const resolvedFiles =
    queryOptions.sourceDocuments && queryOptions.styleFiles
      ? null
      : resolveWorkspaceCheckFilesSync({
          workspaceRoot: args.workspaceRoot,
        });
  const sourceDocuments =
    queryOptions.sourceDocuments ??
    collectSourceDocuments(resolvedFiles?.sourceFiles ?? [], deps.analysisCache);
  const styleFiles = queryOptions.styleFiles ?? resolvedFiles?.styleFiles ?? [];
  const engineInput =
    queryOptions.engineInput ??
    (queryOptions.styleSemanticGraphCache && styleFiles.length > 1
      ? buildEngineInputV2({
          workspaceRoot: args.workspaceRoot,
          classnameTransform: args.classnameTransform,
          pathAlias: args.pathAlias,
          sourceDocuments,
          styleFiles,
          analysisCache: deps.analysisCache,
          styleDocumentForPath: deps.styleDocumentForPath,
          typeResolver: deps.typeResolver,
        })
      : undefined);
  const workspaceQueryOptions = {
    ...queryOptions,
    sourceDocuments,
    styleFiles,
    ...(engineInput ? { engineInput } : {}),
  };

  return resolveRustStyleSemanticGraphAsync(
    {
      workspaceRoot: args.workspaceRoot,
      classnameTransform: args.classnameTransform,
      pathAlias: args.pathAlias,
      sourceDocuments,
      styleFiles,
      analysisCache: deps.analysisCache,
      styleDocumentForPath: deps.styleDocumentForPath,
      typeResolver: deps.typeResolver,
      readStyleFile: deps.readStyleFile,
    },
    stylePath,
    workspaceQueryOptions,
  );
}

export function runRustStyleSemanticGraph(
  input: StyleSemanticGraphRunnerInputV0,
  options: StyleSemanticGraphQueryOptions = {},
): StyleSemanticGraphSummaryV0 {
  const runJson = options.runRustSelectedQueryBackendJson ?? runRustSelectedQueryBackendJson;
  return runJson<StyleSemanticGraphSummaryV0>(
    SELECTED_QUERY_RUNNER_COMMANDS.styleSemanticGraph,
    input,
  );
}

export function runRustStyleSemanticGraphAsync(
  input: StyleSemanticGraphRunnerInputV0,
  options: StyleSemanticGraphQueryOptions = {},
): Promise<StyleSemanticGraphSummaryV0> {
  const runJson =
    options.runRustSelectedQueryBackendJsonAsync ?? runRustSelectedQueryBackendJsonAsync;
  return runJson<StyleSemanticGraphSummaryV0>(
    SELECTED_QUERY_RUNNER_COMMANDS.styleSemanticGraph,
    input,
  );
}

export function runRustStyleSemanticGraphBatch(
  input: StyleSemanticGraphBatchRunnerInputV0,
  options: StyleSemanticGraphQueryOptions = {},
): StyleSemanticGraphBatchRunnerOutputV0 {
  const runJson = options.runRustSelectedQueryBackendJson ?? runRustSelectedQueryBackendJson;
  return runJson<StyleSemanticGraphBatchRunnerOutputV0>(
    SELECTED_QUERY_RUNNER_COMMANDS.styleSemanticGraphBatch,
    input,
  );
}

export function runRustStyleSemanticGraphBatchAsync(
  input: StyleSemanticGraphBatchRunnerInputV0,
  options: StyleSemanticGraphQueryOptions = {},
): Promise<StyleSemanticGraphBatchRunnerOutputV0> {
  const runJson =
    options.runRustSelectedQueryBackendJsonAsync ?? runRustSelectedQueryBackendJsonAsync;
  return runJson<StyleSemanticGraphBatchRunnerOutputV0>(
    SELECTED_QUERY_RUNNER_COMMANDS.styleSemanticGraphBatch,
    input,
  );
}

export function buildStyleSemanticGraphSelectorIdentityReadModels(
  graph: StyleSemanticGraphSummaryV0,
  styleDocument: StyleDocumentHIR,
): readonly StyleSemanticGraphSelectorIdentityReadModel[] {
  const selectorByCanonicalName = new Map(
    styleDocument.selectors.map((selector) => [selector.canonicalName, selector] as const),
  );

  return graph.selectorIdentityEngine.canonicalIds.flatMap((identity) => {
    const selector = selectorByCanonicalName.get(identity.localName);
    if (!selector) return [];

    return [
      {
        canonicalId: identity.canonicalId,
        canonicalName: identity.localName,
        identityKind: identity.identityKind,
        rewriteSafety: identity.rewriteSafety,
        blockers: identity.blockers,
        range: selector.range,
        ruleRange: selector.ruleRange,
        viewKind: selector.viewKind,
      },
    ];
  });
}

export function buildStyleSemanticGraphDesignTokenRankedReferenceReadModels(
  graph: StyleSemanticGraphSummaryV0,
  styleDocument?: StyleDocumentHIR,
): readonly StyleSemanticGraphDesignTokenRankedReferenceReadModel[] {
  const designTokenSemantics = graph.designTokenSemantics;
  return (
    designTokenSemantics?.cascadeRankingSignal.rankedReferences.map((reference) => {
      const referenceNode = styleDocument?.customPropertyRefs[reference.referenceSourceOrder];
      const winnerDeclaration =
        reference.winnerDeclarationFilePath === undefined
          ? styleDocument?.customPropertyDecls[reference.winnerDeclarationSourceOrder]
          : undefined;
      const shadowedDeclarations = styleDocument
        ? reference.shadowedDeclarationSourceOrders.flatMap((sourceOrder) => {
            const declaration = styleDocument.customPropertyDecls[sourceOrder];
            return declaration ? [declaration] : [];
          })
        : undefined;

      const readModel: MutableStyleSemanticGraphDesignTokenRankedReferenceReadModel = {
        referenceName: reference.referenceName,
        referenceSourceOrder: reference.referenceSourceOrder,
        winnerDeclarationSourceOrder: reference.winnerDeclarationSourceOrder,
        shadowedDeclarationSourceOrders: reference.shadowedDeclarationSourceOrders,
        candidateDeclarationCount: reference.candidateDeclarationCount,
        crossFileCandidateDeclarationCount: reference.crossFileCandidateDeclarationCount ?? 0,
        crossFileShadowedDeclarationCount: reference.crossFileShadowedDeclarationCount ?? 0,
      };
      if (reference.winnerImportGraphDistance !== undefined) {
        readModel.winnerImportGraphDistance = reference.winnerImportGraphDistance;
      }
      if (reference.winnerImportGraphOrder !== undefined) {
        readModel.winnerImportGraphOrder = reference.winnerImportGraphOrder;
      }
      if (reference.winnerContextKind !== undefined) {
        readModel.winnerContextKind = reference.winnerContextKind;
      }
      if (reference.winnerDeclarationFilePath) {
        readModel.winnerDeclarationFilePath = reference.winnerDeclarationFilePath;
        readModel.crossFileCandidateScope = designTokenSemantics.resolutionScope;
      }
      if (reference.winnerDeclarationRange) {
        readModel.winnerDeclarationRange = reference.winnerDeclarationRange;
      }
      if (referenceNode) readModel.reference = referenceNode;
      if (winnerDeclaration) readModel.winnerDeclaration = winnerDeclaration;
      if (shadowedDeclarations) readModel.shadowedDeclarations = shadowedDeclarations;
      return readModel;
    }) ?? []
  );
}

function ensureStyleFileIncluded(
  styleFiles: readonly string[],
  stylePath: string,
): readonly string[] {
  return styleFiles.includes(stylePath) ? styleFiles : [...styleFiles, stylePath];
}

async function maybePopulateStyleSemanticGraphCacheFromBatchAsync(
  options: StyleSemanticGraphQueryBackendOptions,
  queryOptions: StyleSemanticGraphQueryOptions,
): Promise<void> {
  const cache = queryOptions.styleSemanticGraphCache;
  if (!cache || !queryOptions.engineInput || !queryOptions.styleFiles) return;

  const uncachedStyleFiles = queryOptions.styleFiles.filter((stylePath) => !cache.has(stylePath));
  if (uncachedStyleFiles.length <= 1) return;

  const styles: StyleSemanticGraphBatchStyleInputV0[] = [];
  for (const stylePath of uncachedStyleFiles) {
    const styleSource = options.readStyleFile(stylePath);
    if (styleSource === null) {
      cache.set(stylePath, null);
      continue;
    }
    styles.push({ stylePath, styleSource });
  }
  if (styles.length <= 1) return;
  const packageManifests = collectStyleSemanticGraphPackageManifests(styles, options.readStyleFile);

  try {
    const requestedStylePaths = new Set(styles.map((style) => style.stylePath));
    const output = await runRustStyleSemanticGraphBatchAsync(
      {
        styles,
        ...(packageManifests.length > 0 ? { packageManifests } : {}),
        engineInput: queryOptions.engineInput,
      },
      queryOptions,
    );

    for (const entry of output.graphs) {
      if (!requestedStylePaths.has(entry.stylePath)) continue;
      cache.set(entry.stylePath, entry.graph);
    }
  } catch (err) {
    if (isEngineShadowRunnerCancelledError(err)) {
      for (const style of styles) cache.set(style.stylePath, null);
    }
    // Batch is an optimization only. Preserve the single-target fallback path.
  }
}

function maybePopulateStyleSemanticGraphCacheFromBatch(
  options: StyleSemanticGraphQueryBackendOptions,
  queryOptions: StyleSemanticGraphQueryOptions,
): void {
  const cache = queryOptions.styleSemanticGraphCache;
  if (!cache || !queryOptions.engineInput || !queryOptions.styleFiles) return;

  const uncachedStyleFiles = queryOptions.styleFiles.filter((stylePath) => !cache.has(stylePath));
  if (uncachedStyleFiles.length <= 1) return;

  const styles: StyleSemanticGraphBatchStyleInputV0[] = [];
  for (const stylePath of uncachedStyleFiles) {
    const styleSource = options.readStyleFile(stylePath);
    if (styleSource === null) {
      cache.set(stylePath, null);
      continue;
    }
    styles.push({ stylePath, styleSource });
  }
  if (styles.length <= 1) return;
  const packageManifests = collectStyleSemanticGraphPackageManifests(styles, options.readStyleFile);

  try {
    const requestedStylePaths = new Set(styles.map((style) => style.stylePath));
    const output = runRustStyleSemanticGraphBatch(
      {
        styles,
        ...(packageManifests.length > 0 ? { packageManifests } : {}),
        engineInput: queryOptions.engineInput,
      },
      queryOptions,
    );

    for (const entry of output.graphs) {
      if (!requestedStylePaths.has(entry.stylePath)) continue;
      cache.set(entry.stylePath, entry.graph);
    }
  } catch (err) {
    if (isEngineShadowRunnerCancelledError(err)) {
      for (const style of styles) cache.set(style.stylePath, null);
    }
    // Batch is an optimization only. Preserve the single-target fallback path.
  }
}

function collectStyleSemanticGraphPackageManifests(
  styles: readonly StyleSemanticGraphBatchStyleInputV0[],
  readStyleFile: ProviderDeps["readStyleFile"],
): readonly StyleSemanticGraphPackageManifestInputV0[] {
  const manifests = new Map<string, StyleSemanticGraphPackageManifestInputV0>();
  for (const style of styles) {
    for (const source of collectSassModuleSources(style)) {
      const packageName = parsePackageStyleSource(source)?.packageName;
      if (!packageName) continue;
      for (const packageJsonPath of packageJsonCandidatePaths(style.stylePath, packageName)) {
        if (manifests.has(packageJsonPath)) continue;
        const packageJsonSource = readStyleFile(packageJsonPath);
        if (packageJsonSource === null) continue;
        manifests.set(packageJsonPath, { packageJsonPath, packageJsonSource });
        break;
      }
    }
  }
  return [...manifests.values()];
}

function collectSassModuleSources(style: StyleSemanticGraphBatchStyleInputV0): readonly string[] {
  const styleDocument = parseStyleDocument(style.styleSource, style.stylePath);
  return [
    ...styleDocument.sassModuleUses.map((moduleUse) => moduleUse.source),
    ...styleDocument.sassModuleForwards.map((moduleForward) => moduleForward.source),
  ];
}

function packageJsonCandidatePaths(stylePath: string, packageName: string): readonly string[] {
  const candidates: string[] = [];
  let current = path.dirname(stylePath);
  while (true) {
    candidates.push(path.join(current, "node_modules", packageName, "package.json"));
    const parent = path.dirname(current);
    if (parent === current) break;
    current = parent;
  }
  return candidates;
}

function parsePackageStyleSource(
  source: string,
): { readonly packageName: string; readonly subpath: string | null } | null {
  if (
    source.startsWith(".") ||
    source.startsWith("/") ||
    source.startsWith("sass:") ||
    source.startsWith("http://") ||
    source.startsWith("https://")
  ) {
    return null;
  }

  if (source.startsWith("@")) {
    const segments = source.split("/");
    if (segments.length < 2 || segments[0]!.length <= 1 || segments[1]!.length === 0) {
      return null;
    }
    return {
      packageName: `${segments[0]!}/${segments[1]!}`,
      subpath: segments.slice(2).join("/") || null,
    };
  }

  const [packageName, ...subpathParts] = source.split("/");
  if (!packageName) return null;
  return {
    packageName,
    subpath: subpathParts.join("/") || null,
  };
}
