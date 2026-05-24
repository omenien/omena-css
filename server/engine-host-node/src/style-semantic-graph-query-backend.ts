import path from "node:path";
import type { Range } from "@css-module-explainer/shared";
import type { EngineInputV2 } from "../../engine-core-ts/src/contracts";
import type { StyleDocumentHIR } from "../../engine-core-ts/src/core/hir/style-types";
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
export type StyleSemanticGraphBatchOutputCache = Map<
  string,
  StyleSemanticGraphBatchRunnerOutputV0 | null
>;

export interface StyleSemanticGraphSummaryV0 {
  readonly schemaVersion: "0";
  readonly product: "omena-semantic.style-semantic-graph";
  readonly language: string;
  readonly parserFacts: unknown;
  readonly semanticFacts: unknown;
  readonly cssModulesSemantics?: StyleSemanticGraphCssModulesSemanticsV0;
  readonly designTokenSemantics?: StyleSemanticGraphDesignTokenSemanticsV0;
  readonly selectorIdentityEngine: StyleSemanticGraphSelectorIdentityEngineV0;
  readonly selectorReferenceEngine: StyleSemanticGraphSelectorReferenceEngineV0;
  readonly sourceInputEvidence: unknown;
  readonly promotionEvidence: unknown;
  readonly losslessCstContract: unknown;
}

export interface StyleSemanticGraphCssModulesSemanticsV0 {
  readonly schemaVersion: "0";
  readonly product: "omena-semantic.css-modules-semantics";
  readonly status: string;
  readonly resolutionScope: string;
  readonly classExportCount: number;
  readonly classExportNames: readonly string[];
  readonly composesEdgeSeedCount: number;
  readonly composesLocalEdgeCount: number;
  readonly composesGlobalEdgeCount: number;
  readonly composesExternalEdgeCount: number;
  readonly composesTargetNames: readonly string[];
  readonly composesImportSources: readonly string[];
  readonly valueEdgeSeedCount: number;
  readonly valueImportEdgeCount: number;
  readonly valueDefinitionEdgeCount: number;
  readonly valueDefinitionNames: readonly string[];
  readonly valueReferenceNames: readonly string[];
  readonly valueImportSources: readonly string[];
  readonly icssEdgeSeedCount: number;
  readonly icssImportEdgeCount: number;
  readonly icssExportEdgeCount: number;
  readonly icssExportNames: readonly string[];
  readonly icssImportLocalNames: readonly string[];
  readonly icssImportRemoteNames: readonly string[];
  readonly icssImportSources: readonly string[];
  readonly keyframeNames: readonly string[];
  readonly animationReferenceNames: readonly string[];
  readonly capabilities: StyleSemanticGraphCssModulesSemanticCapabilitiesV0;
  readonly nextPriorities: readonly string[];
}

export interface StyleSemanticGraphCssModulesSemanticCapabilitiesV0 {
  readonly parserFactSurfaceReady: boolean;
  readonly perFileSymbolSummaryReady: boolean;
  readonly composesEdgeSeedReady: boolean;
  readonly valueEdgeSeedReady: boolean;
  readonly icssEdgeSeedReady: boolean;
  readonly animationEdgeSeedReady: boolean;
  readonly crossFileResolutionReady: boolean;
  readonly composesClosureReady: boolean;
  readonly valueGraphResolutionReady: boolean;
  readonly cycleDetectionReady: boolean;
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
  readonly declarationCandidates?: readonly StyleSemanticGraphDesignTokenDeclarationCandidateV0[];
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

export interface StyleSemanticGraphDesignTokenDeclarationCandidateV0 {
  readonly name: string;
  readonly sourceOrder: number;
  readonly filePath: string;
  readonly range: Range;
  readonly selectorContexts: readonly string[];
  readonly underMedia: boolean;
  readonly underSupports: boolean;
  readonly underLayer: boolean;
  readonly candidateScope: string;
  readonly importGraphDistance?: number;
  readonly importGraphOrder?: number;
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

export interface OmenaParserStyleFactsV0 {
  readonly schemaVersion: "0";
  readonly product: "omena-parser.style-facts";
  readonly dialect: string;
  readonly sassModuleUseSources: readonly string[];
  readonly sassModuleForwardSources: readonly string[];
  readonly sassModuleImportSources: readonly string[];
}

export interface StyleSemanticGraphPackageManifestInputV0 {
  readonly packageJsonPath: string;
  readonly packageJsonSource: string;
}

export interface StyleSemanticGraphBatchRunnerOutputV0 {
  readonly schemaVersion: "0";
  readonly product: "omena-semantic.style-semantic-graph-batch";
  readonly crossFileSummary?: StyleSemanticGraphCrossFileSummaryV0;
  readonly cssModulesResolution?: StyleSemanticGraphCssModulesCrossFileResolutionV0;
  readonly sassModuleResolution?: StyleSemanticGraphSassModuleCrossFileResolutionV0;
  readonly graphs: readonly StyleSemanticGraphBatchEntryV0[];
}

export interface StyleSemanticGraphCrossFileSummaryV0 {
  readonly schemaVersion: "0";
  readonly product: "omena-query.cross-file-summary";
  readonly status: string;
  readonly summaryScope: string;
  readonly styleCount: number;
  readonly summaryEdgeCount: number;
  readonly edgeKindCounts: readonly StyleSemanticGraphCrossFileSummaryEdgeKindCountV0[];
  readonly summaryHash: string;
  readonly edges: readonly StyleSemanticGraphCrossFileSummaryEdgeV0[];
  readonly capabilities: StyleSemanticGraphCrossFileSummaryCapabilitiesV0;
  readonly nextPriorities: readonly string[];
}

export interface StyleSemanticGraphCrossFileSummaryEdgeKindCountV0 {
  readonly edgeKind: string;
  readonly count: number;
}

export interface StyleSemanticGraphCrossFileSummaryEdgeV0 {
  readonly edgeId: string;
  readonly edgeKind: string;
  readonly fromKind: string;
  readonly fromPath: string;
  readonly targetKind?: string | null;
  readonly targetPath?: string | null;
  readonly source?: string | null;
  readonly ownerSelectorName?: string | null;
  readonly localName?: string | null;
  readonly remoteName?: string | null;
  readonly targetNames: readonly string[];
  readonly status: string;
  readonly provenance: readonly string[];
  readonly linearProvenance: StyleSemanticGraphLinearProvenanceV0;
}

export interface StyleSemanticGraphCrossFileSummaryCapabilitiesV0 {
  readonly cssModulesComposesEdgesReady: boolean;
  readonly cssModulesValueEdgesReady: boolean;
  readonly cssModulesIcssEdgesReady: boolean;
  readonly sassModuleEdgesReady: boolean;
  readonly styleDesignTokenReferenceEdgesReady: boolean;
  readonly sourceSelectorReferenceEdgesReady: boolean;
  readonly stableSummaryHashReady: boolean;
  readonly linearProvenanceReady: boolean;
}

export interface StyleSemanticGraphLinearProvenanceV0 {
  readonly schemaVersion: "0";
  readonly product: "omena-abstract-value.linear-provenance";
  readonly layerMarker: "qtt-graded";
  readonly featureGate: "qtt-provenance";
  readonly semiringIdentifier: "lin01";
  readonly semiring: {
    readonly zero: number;
    readonly one: number;
    readonly addition: string;
    readonly multiplication: string;
    readonly idempotentAddition: boolean;
  };
  readonly termCount: number;
  readonly terms: readonly {
    readonly coefficient: number;
    readonly label: string;
  }[];
}

export interface StyleSemanticGraphCssModulesCrossFileResolutionV0 {
  readonly schemaVersion: "0";
  readonly product: "omena-query.css-modules-cross-file-resolution";
  readonly status: string;
  readonly resolutionScope: string;
  readonly styleCount: number;
  readonly importEdgeCount: number;
  readonly resolvedImportEdgeCount: number;
  readonly unresolvedImportEdgeCount: number;
  readonly matchedNameCount: number;
  readonly edges: readonly StyleSemanticGraphCssModulesImportEdgeResolutionV0[];
  readonly composesClosureEdgeCount: number;
  readonly valueClosureEdgeCount: number;
  readonly icssClosureEdgeCount: number;
  readonly composesCycleCount: number;
  readonly valueCycleCount: number;
  readonly icssCycleCount: number;
  readonly composesClosureEdges: readonly StyleSemanticGraphCssModulesComposesClosureEdgeV0[];
  readonly valueClosureEdges: readonly StyleSemanticGraphCssModulesValueClosureEdgeV0[];
  readonly icssClosureEdges: readonly StyleSemanticGraphCssModulesIcssClosureEdgeV0[];
  readonly cycles: readonly StyleSemanticGraphCssModulesCycleV0[];
  readonly capabilities: {
    readonly importSourceResolutionReady: boolean;
    readonly composesNameMatchReady: boolean;
    readonly valueNameMatchReady: boolean;
    readonly icssNameMatchReady: boolean;
    readonly transitiveClosureReady: boolean;
    readonly valueGraphClosureReady: boolean;
    readonly icssExportImportClosureReady: boolean;
    readonly cycleDetectionReady: boolean;
  };
  readonly nextPriorities: readonly string[];
}

export interface StyleSemanticGraphCssModulesImportEdgeResolutionV0 {
  readonly fromStylePath: string;
  readonly importKind: string;
  readonly source: string;
  readonly resolvedStylePath?: string | null;
  readonly status: string;
  readonly importGraphDistance?: number | null;
  readonly importGraphOrder?: number | null;
  readonly importedNames: readonly string[];
  readonly exportedNames: readonly string[];
  readonly matchedNames: readonly string[];
}

export interface StyleSemanticGraphCssModulesComposesClosureEdgeV0 {
  readonly fromStylePath: string;
  readonly ownerSelectorName: string;
  readonly targetStylePath: string;
  readonly targetSelectorName: string;
  readonly depth: number;
  readonly path: readonly string[];
}

export interface StyleSemanticGraphCssModulesValueClosureEdgeV0 {
  readonly fromStylePath: string;
  readonly valueName: string;
  readonly targetStylePath: string;
  readonly targetValueName: string;
  readonly depth: number;
  readonly path: readonly string[];
}

export interface StyleSemanticGraphCssModulesIcssClosureEdgeV0 {
  readonly fromStylePath: string;
  readonly name: string;
  readonly targetStylePath: string;
  readonly targetName: string;
  readonly depth: number;
  readonly path: readonly string[];
}

export interface StyleSemanticGraphCssModulesCycleV0 {
  readonly kind: string;
  readonly path: readonly string[];
}

export interface StyleSemanticGraphSassModuleCrossFileResolutionV0 {
  readonly schemaVersion: "0";
  readonly product: "omena-query.sass-module-cross-file-resolution";
  readonly status: string;
  readonly resolutionScope: string;
  readonly styleCount: number;
  readonly moduleEdgeCount: number;
  readonly resolvedModuleEdgeCount: number;
  readonly unresolvedModuleEdgeCount: number;
  readonly externalModuleEdgeCount: number;
  readonly edges: readonly StyleSemanticGraphSassModuleEdgeResolutionV0[];
  readonly graphClosureEdgeCount: number;
  readonly cycleCount: number;
  readonly visibilityFilterCount: number;
  readonly graphClosureEdges: readonly StyleSemanticGraphSassModuleGraphClosureEdgeV0[];
  readonly cycles: readonly StyleSemanticGraphSassModuleCycleV0[];
  readonly capabilities: StyleSemanticGraphSassModuleCrossFileResolutionCapabilitiesV0;
  readonly nextPriorities: readonly string[];
}

export interface StyleSemanticGraphSassModuleEdgeResolutionV0 {
  readonly fromStylePath: string;
  readonly edgeKind: string;
  readonly source: string;
  readonly namespaceKind?: string | null;
  readonly namespace?: string | null;
  readonly visibilityFilterKind?: string | null;
  readonly visibilityFilterNames: readonly string[];
  readonly resolvedStylePath?: string | null;
  readonly status: string;
  readonly resolutionKind: string;
  readonly candidateCount: number;
}

export interface StyleSemanticGraphSassModuleGraphClosureEdgeV0 {
  readonly fromStylePath: string;
  readonly targetStylePath: string;
  readonly edgeKind: string;
  readonly depth: number;
  readonly path: readonly string[];
  readonly namespaceKind?: string | null;
  readonly namespace?: string | null;
  readonly visibilityFilterKind?: string | null;
  readonly visibilityFilterNames: readonly string[];
}

export interface StyleSemanticGraphSassModuleCycleV0 {
  readonly path: readonly string[];
}

export interface StyleSemanticGraphSassModuleCrossFileResolutionCapabilitiesV0 {
  readonly omenaParserModuleEdgeConsumptionReady: boolean;
  readonly resolverBackedSourceResolutionReady: boolean;
  readonly packageManifestResolutionReady: boolean;
  readonly externalModuleFilteringReady: boolean;
  readonly graphClosureReady: boolean;
  readonly cycleDetectionReady: boolean;
  readonly namespaceShowHideFilterReady: boolean;
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
type StyleSemanticGraphWorkspaceTargetArgs = {
  readonly workspaceRoot: string;
  readonly classnameTransform: BuildSelectedQueryResultsV2Options["classnameTransform"];
  readonly pathAlias: BuildSelectedQueryResultsV2Options["pathAlias"];
};
type StyleSemanticGraphWorkspaceTargetDeps = Pick<
  ProviderDeps,
  "analysisCache" | "styleDocumentForPath" | "typeResolver" | "readStyleFile"
>;

export interface StyleSemanticGraphQueryOptions {
  readonly runRustSelectedQueryBackendJson?: RustJsonRunner;
  readonly runRustSelectedQueryBackendJsonAsync?: RustJsonRunnerAsync;
  readonly engineInput?: EngineInputV2;
  readonly sourceDocuments?: readonly SourceDocumentSnapshot[];
  readonly styleFiles?: readonly string[];
  readonly styleSemanticGraphCache?: StyleSemanticGraphCache;
  readonly styleSemanticGraphBatchOutputCache?: StyleSemanticGraphBatchOutputCache;
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

export function resolveRustCssModulesCrossFileResolutionForWorkspaceTarget(
  args: StyleSemanticGraphWorkspaceTargetArgs,
  deps: StyleSemanticGraphWorkspaceTargetDeps,
  stylePath: string,
  queryOptions: StyleSemanticGraphQueryOptions = {},
): StyleSemanticGraphCssModulesCrossFileResolutionV0 | null {
  const output = resolveRustStyleSemanticGraphBatchOutputForWorkspaceTarget(
    args,
    deps,
    stylePath,
    queryOptions,
  );
  return output?.cssModulesResolution ?? null;
}

export async function resolveRustCssModulesCrossFileResolutionForWorkspaceTargetAsync(
  args: StyleSemanticGraphWorkspaceTargetArgs,
  deps: StyleSemanticGraphWorkspaceTargetDeps,
  stylePath: string,
  queryOptions: StyleSemanticGraphQueryOptions = {},
): Promise<StyleSemanticGraphCssModulesCrossFileResolutionV0 | null> {
  const output = await resolveRustStyleSemanticGraphBatchOutputForWorkspaceTargetAsync(
    args,
    deps,
    stylePath,
    queryOptions,
  );
  return output?.cssModulesResolution ?? null;
}

export function resolveRustSassModuleCrossFileResolutionForWorkspaceTarget(
  args: StyleSemanticGraphWorkspaceTargetArgs,
  deps: StyleSemanticGraphWorkspaceTargetDeps,
  stylePath: string,
  queryOptions: StyleSemanticGraphQueryOptions = {},
): StyleSemanticGraphSassModuleCrossFileResolutionV0 | null {
  const output = resolveRustStyleSemanticGraphBatchOutputForWorkspaceTarget(
    args,
    deps,
    stylePath,
    queryOptions,
  );
  return output?.sassModuleResolution ?? null;
}

export async function resolveRustSassModuleCrossFileResolutionForWorkspaceTargetAsync(
  args: StyleSemanticGraphWorkspaceTargetArgs,
  deps: StyleSemanticGraphWorkspaceTargetDeps,
  stylePath: string,
  queryOptions: StyleSemanticGraphQueryOptions = {},
): Promise<StyleSemanticGraphSassModuleCrossFileResolutionV0 | null> {
  const output = await resolveRustStyleSemanticGraphBatchOutputForWorkspaceTargetAsync(
    args,
    deps,
    stylePath,
    queryOptions,
  );
  return output?.sassModuleResolution ?? null;
}

function resolveRustStyleSemanticGraphBatchOutputForWorkspaceTarget(
  args: StyleSemanticGraphWorkspaceTargetArgs,
  deps: StyleSemanticGraphWorkspaceTargetDeps,
  stylePath: string,
  queryOptions: StyleSemanticGraphQueryOptions,
): StyleSemanticGraphBatchRunnerOutputV0 | null {
  const resolvedFiles =
    queryOptions.sourceDocuments && queryOptions.styleFiles
      ? null
      : resolveWorkspaceCheckFilesSync({
          workspaceRoot: args.workspaceRoot,
        });
  const sourceDocuments =
    queryOptions.sourceDocuments ??
    collectSourceDocuments(resolvedFiles?.sourceFiles ?? [], deps.analysisCache);
  const styleFiles = ensureStyleFileIncluded(
    queryOptions.styleFiles ?? resolvedFiles?.styleFiles ?? [],
    stylePath,
  );
  const engineInput =
    queryOptions.engineInput ??
    buildEngineInputV2({
      workspaceRoot: args.workspaceRoot,
      classnameTransform: args.classnameTransform,
      pathAlias: args.pathAlias,
      sourceDocuments,
      styleFiles,
      analysisCache: deps.analysisCache,
      styleDocumentForPath: deps.styleDocumentForPath,
      typeResolver: deps.typeResolver,
    });

  return resolveRustStyleSemanticGraphBatchOutput(
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
    {
      ...queryOptions,
      sourceDocuments,
      styleFiles,
      engineInput,
    },
  );
}

async function resolveRustStyleSemanticGraphBatchOutputForWorkspaceTargetAsync(
  args: StyleSemanticGraphWorkspaceTargetArgs,
  deps: StyleSemanticGraphWorkspaceTargetDeps,
  stylePath: string,
  queryOptions: StyleSemanticGraphQueryOptions,
): Promise<StyleSemanticGraphBatchRunnerOutputV0 | null> {
  const resolvedFiles =
    queryOptions.sourceDocuments && queryOptions.styleFiles
      ? null
      : resolveWorkspaceCheckFilesSync({
          workspaceRoot: args.workspaceRoot,
        });
  const sourceDocuments =
    queryOptions.sourceDocuments ??
    collectSourceDocuments(resolvedFiles?.sourceFiles ?? [], deps.analysisCache);
  const styleFiles = ensureStyleFileIncluded(
    queryOptions.styleFiles ?? resolvedFiles?.styleFiles ?? [],
    stylePath,
  );
  const engineInput =
    queryOptions.engineInput ??
    buildEngineInputV2({
      workspaceRoot: args.workspaceRoot,
      classnameTransform: args.classnameTransform,
      pathAlias: args.pathAlias,
      sourceDocuments,
      styleFiles,
      analysisCache: deps.analysisCache,
      styleDocumentForPath: deps.styleDocumentForPath,
      typeResolver: deps.typeResolver,
    });

  return resolveRustStyleSemanticGraphBatchOutputAsync(
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
    {
      ...queryOptions,
      sourceDocuments,
      styleFiles,
      engineInput,
    },
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
  const batchStyles = expandStyleSemanticGraphBatchStyles(
    styles,
    options.readStyleFile,
    queryOptions,
  );
  const packageManifests = collectStyleSemanticGraphPackageManifests(
    batchStyles,
    options.readStyleFile,
    queryOptions,
  );

  const requestedStylePaths = new Set(batchStyles.map((style) => style.stylePath));
  try {
    const output = await runRustStyleSemanticGraphBatchAsync(
      {
        styles: batchStyles,
        ...(packageManifests.length > 0 ? { packageManifests } : {}),
        engineInput: queryOptions.engineInput,
      },
      queryOptions,
    );
    cacheStyleSemanticGraphBatchOutput(batchStyles, queryOptions, output);

    for (const entry of output.graphs) {
      if (!requestedStylePaths.has(entry.stylePath)) continue;
      cache.set(entry.stylePath, entry.graph);
    }
  } catch (err) {
    if (isEngineShadowRunnerCancelledError(err)) {
      for (const style of batchStyles) cache.set(style.stylePath, null);
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
  const batchStyles = expandStyleSemanticGraphBatchStyles(
    styles,
    options.readStyleFile,
    queryOptions,
  );
  const packageManifests = collectStyleSemanticGraphPackageManifests(
    batchStyles,
    options.readStyleFile,
    queryOptions,
  );

  const requestedStylePaths = new Set(batchStyles.map((style) => style.stylePath));
  try {
    const output = runRustStyleSemanticGraphBatch(
      {
        styles: batchStyles,
        ...(packageManifests.length > 0 ? { packageManifests } : {}),
        engineInput: queryOptions.engineInput,
      },
      queryOptions,
    );
    cacheStyleSemanticGraphBatchOutput(batchStyles, queryOptions, output);

    for (const entry of output.graphs) {
      if (!requestedStylePaths.has(entry.stylePath)) continue;
      cache.set(entry.stylePath, entry.graph);
    }
  } catch (err) {
    if (isEngineShadowRunnerCancelledError(err)) {
      for (const style of batchStyles) cache.set(style.stylePath, null);
    }
    // Batch is an optimization only. Preserve the single-target fallback path.
  }
}

function resolveRustStyleSemanticGraphBatchOutput(
  options: StyleSemanticGraphQueryBackendOptions,
  queryOptions: StyleSemanticGraphQueryOptions,
): StyleSemanticGraphBatchRunnerOutputV0 | null {
  if (!queryOptions.engineInput || !queryOptions.styleFiles) return null;

  const styles = collectReadableStyleSemanticGraphBatchStyles(
    queryOptions.styleFiles,
    options.readStyleFile,
  );
  if (styles.length === 0) return null;
  const batchStyles = expandStyleSemanticGraphBatchStyles(
    styles,
    options.readStyleFile,
    queryOptions,
  );
  const cached = readStyleSemanticGraphBatchOutputCache(batchStyles, queryOptions);
  if (cached !== undefined) return cached;
  const packageManifests = collectStyleSemanticGraphPackageManifests(
    batchStyles,
    options.readStyleFile,
    queryOptions,
  );

  try {
    const output = runRustStyleSemanticGraphBatch(
      {
        styles: batchStyles,
        ...(packageManifests.length > 0 ? { packageManifests } : {}),
        engineInput: queryOptions.engineInput,
      },
      queryOptions,
    );
    cacheStyleSemanticGraphBatchOutput(batchStyles, queryOptions, output);
    seedStyleSemanticGraphCacheFromBatchOutput(batchStyles, queryOptions, output);
    return output;
  } catch (err) {
    if (isEngineShadowRunnerCancelledError(err)) {
      cacheStyleSemanticGraphBatchOutput(batchStyles, queryOptions, null);
      for (const style of batchStyles)
        queryOptions.styleSemanticGraphCache?.set(style.stylePath, null);
    }
    return null;
  }
}

async function resolveRustStyleSemanticGraphBatchOutputAsync(
  options: StyleSemanticGraphQueryBackendOptions,
  queryOptions: StyleSemanticGraphQueryOptions,
): Promise<StyleSemanticGraphBatchRunnerOutputV0 | null> {
  if (!queryOptions.engineInput || !queryOptions.styleFiles) return null;

  const styles = collectReadableStyleSemanticGraphBatchStyles(
    queryOptions.styleFiles,
    options.readStyleFile,
  );
  if (styles.length === 0) return null;
  const batchStyles = expandStyleSemanticGraphBatchStyles(
    styles,
    options.readStyleFile,
    queryOptions,
  );
  const cached = readStyleSemanticGraphBatchOutputCache(batchStyles, queryOptions);
  if (cached !== undefined) return cached;
  const packageManifests = collectStyleSemanticGraphPackageManifests(
    batchStyles,
    options.readStyleFile,
    queryOptions,
  );

  try {
    const output = await runRustStyleSemanticGraphBatchAsync(
      {
        styles: batchStyles,
        ...(packageManifests.length > 0 ? { packageManifests } : {}),
        engineInput: queryOptions.engineInput,
      },
      queryOptions,
    );
    cacheStyleSemanticGraphBatchOutput(batchStyles, queryOptions, output);
    seedStyleSemanticGraphCacheFromBatchOutput(batchStyles, queryOptions, output);
    return output;
  } catch (err) {
    if (isEngineShadowRunnerCancelledError(err)) {
      cacheStyleSemanticGraphBatchOutput(batchStyles, queryOptions, null);
      for (const style of batchStyles)
        queryOptions.styleSemanticGraphCache?.set(style.stylePath, null);
    }
    return null;
  }
}

function collectReadableStyleSemanticGraphBatchStyles(
  styleFiles: readonly string[],
  readStyleFile: ProviderDeps["readStyleFile"],
): readonly StyleSemanticGraphBatchStyleInputV0[] {
  return styleFiles.flatMap((stylePath) => {
    const styleSource = readStyleFile(stylePath);
    return styleSource === null ? [] : [{ stylePath, styleSource }];
  });
}

function seedStyleSemanticGraphCacheFromBatchOutput(
  batchStyles: readonly StyleSemanticGraphBatchStyleInputV0[],
  queryOptions: StyleSemanticGraphQueryOptions,
  output: StyleSemanticGraphBatchRunnerOutputV0,
): void {
  const cache = queryOptions.styleSemanticGraphCache;
  if (!cache) return;
  const requestedStylePaths = new Set(batchStyles.map((style) => style.stylePath));
  for (const entry of output.graphs) {
    if (!requestedStylePaths.has(entry.stylePath)) continue;
    cache.set(entry.stylePath, entry.graph);
  }
}

function readStyleSemanticGraphBatchOutputCache(
  batchStyles: readonly StyleSemanticGraphBatchStyleInputV0[],
  queryOptions: StyleSemanticGraphQueryOptions,
): StyleSemanticGraphBatchRunnerOutputV0 | null | undefined {
  const cache = queryOptions.styleSemanticGraphBatchOutputCache;
  if (!cache || !queryOptions.engineInput) return undefined;
  const key = styleSemanticGraphBatchOutputCacheKey(batchStyles, queryOptions.engineInput);
  return cache.has(key) ? (cache.get(key) ?? null) : undefined;
}

function cacheStyleSemanticGraphBatchOutput(
  batchStyles: readonly StyleSemanticGraphBatchStyleInputV0[],
  queryOptions: StyleSemanticGraphQueryOptions,
  output: StyleSemanticGraphBatchRunnerOutputV0 | null,
): void {
  const cache = queryOptions.styleSemanticGraphBatchOutputCache;
  if (!cache || !queryOptions.engineInput) return;
  cache.set(styleSemanticGraphBatchOutputCacheKey(batchStyles, queryOptions.engineInput), output);
}

function styleSemanticGraphBatchOutputCacheKey(
  batchStyles: readonly StyleSemanticGraphBatchStyleInputV0[],
  engineInput: EngineInputV2,
): string {
  return [
    engineInput.workspace.root,
    engineInput.workspace.settingsKey,
    ...batchStyles.map((style) => style.stylePath).toSorted(),
  ].join("\0");
}

function expandStyleSemanticGraphBatchStyles(
  styles: readonly StyleSemanticGraphBatchStyleInputV0[],
  readStyleFile: ProviderDeps["readStyleFile"],
  queryOptions: StyleSemanticGraphQueryOptions,
): readonly StyleSemanticGraphBatchStyleInputV0[] {
  const byPath = new Map(styles.map((style) => [style.stylePath, style] as const));
  const pending = [...styles];

  while (pending.length > 0) {
    const style = pending.shift()!;
    for (const source of collectSassModuleSources(style, queryOptions)) {
      for (const candidate of styleModuleSourceCandidates(style.stylePath, source, readStyleFile)) {
        if (byPath.has(candidate)) continue;
        const styleSource = readStyleFile(candidate);
        if (styleSource === null) continue;
        const discoveredStyle = { stylePath: candidate, styleSource };
        byPath.set(candidate, discoveredStyle);
        pending.push(discoveredStyle);
        break;
      }
    }
  }

  return [...byPath.values()];
}

function collectStyleSemanticGraphPackageManifests(
  styles: readonly StyleSemanticGraphBatchStyleInputV0[],
  readStyleFile: ProviderDeps["readStyleFile"],
  queryOptions: StyleSemanticGraphQueryOptions,
): readonly StyleSemanticGraphPackageManifestInputV0[] {
  const manifests = new Map<string, StyleSemanticGraphPackageManifestInputV0>();
  for (const style of styles) {
    for (const source of collectSassModuleSources(style, queryOptions)) {
      if (isPackageImportStyleSource(source)) {
        collectPackageImportManifests(style.stylePath, source, readStyleFile, manifests);
        continue;
      }
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

function collectPackageImportManifests(
  stylePath: string,
  source: string,
  readStyleFile: ProviderDeps["readStyleFile"],
  manifests: Map<string, StyleSemanticGraphPackageManifestInputV0>,
): void {
  for (const packageJsonPath of currentPackageJsonCandidatePaths(stylePath)) {
    const packageJsonSource = readStyleFile(packageJsonPath);
    if (packageJsonSource === null) continue;
    manifests.set(packageJsonPath, { packageJsonPath, packageJsonSource });
    const entry = readPackageImportEntry(packageJsonSource, source);
    const externalPackageName = entry ? parsePackageStyleSource(entry)?.packageName : null;
    if (externalPackageName) {
      for (const externalPackageJsonPath of packageJsonCandidatePaths(
        stylePath,
        externalPackageName,
      )) {
        if (manifests.has(externalPackageJsonPath)) continue;
        const externalPackageJsonSource = readStyleFile(externalPackageJsonPath);
        if (externalPackageJsonSource === null) continue;
        manifests.set(externalPackageJsonPath, {
          packageJsonPath: externalPackageJsonPath,
          packageJsonSource: externalPackageJsonSource,
        });
        break;
      }
    }
    break;
  }
}

function collectSassModuleSources(
  style: StyleSemanticGraphBatchStyleInputV0,
  queryOptions: StyleSemanticGraphQueryOptions,
): readonly string[] {
  if (!hasSassModuleDirective(style.styleSource)) return [];
  const facts = runOmenaParserStyleFacts(style, queryOptions);
  return [
    ...facts.sassModuleUseSources,
    ...facts.sassModuleForwardSources,
    ...facts.sassModuleImportSources,
  ];
}

function runOmenaParserStyleFacts(
  style: StyleSemanticGraphBatchStyleInputV0,
  queryOptions: StyleSemanticGraphQueryOptions,
): OmenaParserStyleFactsV0 {
  const runJson = queryOptions.runRustSelectedQueryBackendJson ?? runRustSelectedQueryBackendJson;
  return runJson<OmenaParserStyleFactsV0>(SELECTED_QUERY_RUNNER_COMMANDS.omenaParserStyleFacts, {
    styleSource: style.styleSource,
    dialect: omenaParserDialectForStylePath(style.stylePath),
  });
}

function hasSassModuleDirective(styleSource: string): boolean {
  return (
    styleSource.includes("@use") ||
    styleSource.includes("@forward") ||
    styleSource.includes("@import")
  );
}

function omenaParserDialectForStylePath(stylePath: string): string {
  if (stylePath.endsWith(".sass")) return "sass";
  if (stylePath.endsWith(".scss")) return "scss";
  if (stylePath.endsWith(".less")) return "less";
  return "css";
}

function styleModuleSourceCandidates(
  fromStylePath: string,
  source: string,
  readStyleFile: ProviderDeps["readStyleFile"],
): readonly string[] {
  if (source.startsWith("sass:") || source.startsWith("http://") || source.startsWith("https://")) {
    return [];
  }

  const candidates: string[] = [];
  if (isPackageImportStyleSource(source)) {
    for (const packageImportBasePath of packageImportStyleModuleBaseCandidates(
      fromStylePath,
      source,
      readStyleFile,
    )) {
      pushStyleModulePathCandidates(candidates, packageImportBasePath, true);
    }
    return candidates;
  }

  const basePath = path.isAbsolute(source)
    ? source
    : path.join(path.dirname(fromStylePath), source);
  pushStyleModulePathCandidates(candidates, basePath, path.extname(source) === "");

  for (const packageEntryBasePath of packageManifestStyleModuleBaseCandidates(
    fromStylePath,
    source,
    readStyleFile,
  )) {
    pushStyleModulePathCandidates(candidates, packageEntryBasePath, true);
  }
  for (const packageBasePath of packageStyleModuleBaseCandidates(fromStylePath, source)) {
    pushStyleModulePathCandidates(candidates, packageBasePath, true);
  }

  return candidates;
}

function packageImportStyleModuleBaseCandidates(
  fromStylePath: string,
  source: string,
  readStyleFile: ProviderDeps["readStyleFile"],
): readonly string[] {
  const candidates: string[] = [];
  for (const packageJsonPath of currentPackageJsonCandidatePaths(fromStylePath)) {
    const packageJsonSource = readStyleFile(packageJsonPath);
    if (packageJsonSource === null) continue;
    const entry = readPackageImportEntry(packageJsonSource, source);
    if (!entry) continue;
    for (const candidate of packageImportEntryBaseCandidates(
      path.dirname(packageJsonPath),
      fromStylePath,
      entry,
      readStyleFile,
    )) {
      pushUniquePath(candidates, candidate);
    }
    break;
  }
  return candidates;
}

function packageImportEntryBaseCandidates(
  packageDir: string,
  fromStylePath: string,
  entry: string,
  readStyleFile: ProviderDeps["readStyleFile"],
): readonly string[] {
  if (entry.startsWith("./")) return [path.join(packageDir, normalizePackageJsonEntry(entry))];
  if (entry.startsWith("#") || isExternalStyleModuleSource(entry)) return [];
  return [
    ...packageManifestStyleModuleBaseCandidates(fromStylePath, entry, readStyleFile),
    ...packageStyleModuleBaseCandidates(fromStylePath, entry),
  ];
}

function pushStyleModulePathCandidates(
  candidates: string[],
  basePath: string,
  includeExtensionVariants: boolean,
): void {
  pushStylePathCandidate(candidates, basePath);
  pushPartialStylePathCandidate(candidates, basePath);

  if (!includeExtensionVariants) return;
  for (const extension of [
    ".module.scss",
    ".module.css",
    ".module.less",
    ".scss",
    ".css",
    ".less",
  ]) {
    const candidate = `${basePath}${extension}`;
    pushStylePathCandidate(candidates, candidate);
    pushPartialStylePathCandidate(candidates, candidate);
  }
}

function pushPartialStylePathCandidate(candidates: string[], stylePath: string): void {
  const fileName = path.basename(stylePath);
  if (fileName.startsWith("_")) return;
  pushStylePathCandidate(candidates, path.join(path.dirname(stylePath), `_${fileName}`));
}

function pushStylePathCandidate(candidates: string[], stylePath: string): void {
  const candidate = normalizeStylePath(stylePath);
  if (!candidates.includes(candidate)) candidates.push(candidate);
}

function packageManifestStyleModuleBaseCandidates(
  fromStylePath: string,
  source: string,
  readStyleFile: ProviderDeps["readStyleFile"],
): readonly string[] {
  const packageSource = parsePackageStyleSource(source);
  if (!packageSource) return [];
  const candidates: string[] = [];
  for (const packageJsonPath of packageJsonCandidatePaths(
    fromStylePath,
    packageSource.packageName,
  )) {
    const packageJsonSource = readStyleFile(packageJsonPath);
    if (packageJsonSource === null) continue;
    const entry = readPackageManifestStyleEntry(packageJsonSource, packageSource.subpath);
    if (!entry) continue;
    candidates.push(path.join(path.dirname(packageJsonPath), entry));
    break;
  }
  return candidates;
}

function packageStyleModuleBaseCandidates(
  fromStylePath: string,
  source: string,
): readonly string[] {
  const packageSource = parsePackageStyleSource(source);
  if (!packageSource) return [];
  const candidates: string[] = [];
  let current = path.dirname(fromStylePath);
  while (true) {
    const packageRoot = path.join(current, "node_modules", packageSource.packageName);
    if (packageSource.subpath) {
      pushUniquePath(candidates, path.join(packageRoot, packageSource.subpath));
      pushUniquePath(candidates, path.join(packageRoot, "src", packageSource.subpath));
    } else {
      pushUniquePath(candidates, packageRoot);
      pushUniquePath(candidates, path.join(packageRoot, "index"));
      pushUniquePath(candidates, path.join(packageRoot, "src", "index"));
    }
    const parent = path.dirname(current);
    if (parent === current) break;
    current = parent;
  }
  return candidates;
}

function pushUniquePath(candidates: string[], value: string): void {
  const normalized = normalizeStylePath(value);
  if (!candidates.includes(normalized)) candidates.push(normalized);
}

function readPackageManifestStyleEntry(
  packageJsonSource: string,
  subpath: string | null,
): string | null {
  const packageJson = safeParsePackageJson(packageJsonSource);
  if (!packageJson) return null;
  const entry = subpath
    ? readPackageExportSubpathEntry(packageJson.exports, subpath)
    : (readPackageExportEntry(packageJson.exports) ??
      readPackageJsonStringField(packageJson, "sass") ??
      readPackageJsonStringField(packageJson, "scss") ??
      readPackageJsonStringField(packageJson, "style"));
  return entry ? normalizePackageJsonEntry(entry) : null;
}

function readPackageImportEntry(packageJsonSource: string, specifier: string): string | null {
  const packageJson = safeParsePackageJson(packageJsonSource);
  if (!packageJson || !isObjectRecord(packageJson.imports)) return null;
  const exactEntry = readPackageExportEntry(packageJson.imports[specifier]);
  if (exactEntry) return exactEntry;
  for (const [key, value] of Object.entries(packageJson.imports)) {
    const patternMatch = matchPackageImportPattern(key, specifier);
    if (patternMatch === null) continue;
    const entry = readPackageExportEntry(value);
    if (!entry) continue;
    return entry.includes("*") ? entry.replaceAll("*", patternMatch) : entry;
  }
  return null;
}

function matchPackageImportPattern(patternKey: string, specifier: string): string | null {
  const [prefix, suffix, extra] = patternKey.split("*");
  if (prefix === undefined || suffix === undefined || extra !== undefined) return null;
  if (!specifier.startsWith(prefix) || !specifier.endsWith(suffix)) return null;
  return specifier.slice(prefix.length, specifier.length - suffix.length);
}

function safeParsePackageJson(packageJsonSource: string): Record<string, unknown> | null {
  try {
    const parsed: unknown = JSON.parse(packageJsonSource);
    return isObjectRecord(parsed) ? parsed : null;
  } catch {
    return null;
  }
}

function readPackageJsonStringField(
  packageJson: Record<string, unknown>,
  fieldName: string,
): string | null {
  const value = packageJson[fieldName];
  return typeof value === "string" ? value : null;
}

function readPackageExportSubpathEntry(exportsValue: unknown, subpath: string): string | null {
  if (!isObjectRecord(exportsValue)) return null;
  for (const key of packageExportSubpathKeys(subpath)) {
    const entry = readPackageExportEntry(exportsValue[key]);
    if (entry) return entry;
  }
  for (const [key, value] of Object.entries(exportsValue)) {
    const patternMatch = matchPackageExportSubpathPattern(key, subpath);
    if (patternMatch === null) continue;
    const entry = readPackageExportEntry(value);
    if (!entry) continue;
    return entry.includes("*") ? entry.replaceAll("*", patternMatch) : entry;
  }
  return null;
}

function packageExportSubpathKeys(subpath: string): readonly string[] {
  const normalized = subpath.replace(/^\.?\//u, "");
  return [`./${normalized}`, `./${normalized}.scss`, `./${normalized}.sass`, `./${normalized}.css`];
}

function matchPackageExportSubpathPattern(patternKey: string, subpath: string): string | null {
  const normalizedPattern = patternKey.replace(/^\.?\//u, "");
  const [prefix, suffix, extra] = normalizedPattern.split("*");
  if (prefix === undefined || suffix === undefined || extra !== undefined) return null;
  for (const candidateKey of packageExportSubpathKeys(subpath)) {
    const normalizedCandidate = candidateKey.replace(/^\.?\//u, "");
    if (!normalizedCandidate.startsWith(prefix) || !normalizedCandidate.endsWith(suffix)) {
      continue;
    }
    return normalizedCandidate.slice(prefix.length, normalizedCandidate.length - suffix.length);
  }
  return null;
}

function readPackageExportEntry(exportsValue: unknown): string | null {
  return readPackageExportEntryWithPolicy(exportsValue, true);
}

function readPackageExportEntryWithPolicy(
  exportsValue: unknown,
  requireStyleEntry: boolean,
): string | null {
  if (typeof exportsValue === "string") {
    return !requireStyleEntry || isPackageStyleExportEntry(exportsValue) ? exportsValue : null;
  }
  if (Array.isArray(exportsValue)) {
    for (const value of exportsValue) {
      const entry = readPackageExportEntryWithPolicy(value, requireStyleEntry);
      if (entry) return entry;
    }
    return null;
  }
  if (!isObjectRecord(exportsValue)) return null;
  const rootEntry = readPackageExportEntryWithPolicy(exportsValue["."], requireStyleEntry);
  if (rootEntry) return rootEntry;
  for (const [key, value] of Object.entries(exportsValue)) {
    const entry = isPackageStyleExportCondition(key)
      ? readPackageExportEntryWithPolicy(value, false)
      : key === "default"
        ? readPackageExportEntryWithPolicy(value, true)
        : null;
    if (entry) return entry;
  }
  return null;
}

function isPackageStyleExportCondition(key: string): boolean {
  return ["sass", "scss", "style"].includes(key);
}

function isPackageStyleExportEntry(entry: string): boolean {
  const extension = path.extname(normalizePackageJsonEntry(entry));
  return extension === "" || [".css", ".scss", ".sass", ".less"].includes(extension);
}

function normalizePackageJsonEntry(entry: string): string {
  return entry.replace(/^\.?\//u, "");
}

function normalizeStylePath(stylePath: string): string {
  return path.normalize(stylePath).replaceAll("\\", "/");
}

function isExternalStyleModuleSource(source: string): boolean {
  return (
    source.startsWith("sass:") || source.startsWith("http://") || source.startsWith("https://")
  );
}

function isObjectRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
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

function currentPackageJsonCandidatePaths(stylePath: string): readonly string[] {
  const candidates: string[] = [];
  let current = path.dirname(stylePath);
  while (true) {
    candidates.push(path.join(current, "package.json"));
    const parent = path.dirname(current);
    if (parent === current) break;
    current = parent;
  }
  return candidates;
}

function parsePackageStyleSource(
  source: string,
): { readonly packageName: string; readonly subpath: string | null } | null {
  const packageSource = source.startsWith("pkg:") ? source.slice("pkg:".length) : source;
  if (
    packageSource.startsWith(".") ||
    packageSource.startsWith("/") ||
    packageSource.startsWith("sass:") ||
    packageSource.startsWith("http://") ||
    packageSource.startsWith("https://")
  ) {
    return null;
  }

  if (packageSource.startsWith("@")) {
    const segments = packageSource.split("/");
    if (segments.length < 2 || segments[0]!.length <= 1 || segments[1]!.length === 0) {
      return null;
    }
    return {
      packageName: `${segments[0]!}/${segments[1]!}`,
      subpath: segments.slice(2).join("/") || null,
    };
  }

  const [packageName, ...subpathParts] = packageSource.split("/");
  if (!packageName) return null;
  return {
    packageName,
    subpath: subpathParts.join("/") || null,
  };
}

function isPackageImportStyleSource(source: string): boolean {
  return source.startsWith("#");
}
