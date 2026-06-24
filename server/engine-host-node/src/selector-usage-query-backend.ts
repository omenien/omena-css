import type { Range } from "@omena/shared";
import type { ProviderDeps } from "../../engine-core-ts/src/provider-deps";
import { buildEngineInputV2, buildEngineInputV2Async } from "./engine-input-v2";
import {
  collectSourceDocuments,
  resolveWorkspaceCheckFilesSync,
} from "./checker-host/workspace-check-support";
import {
  isEngineShadowRunnerCancelledError,
  SELECTED_QUERY_RUNNER_COMMANDS,
  runRustSelectedQueryBackendJson,
  runRustSelectedQueryBackendJsonAsync,
  type RustSelectedQueryBackendJsonRunnerAsync,
} from "./selected-query-backend";
import type { BuildSelectedQueryResultsV2Options } from "./engine-query-v2";

export type SelectorUsagePayloadCache = Map<string, readonly SelectorUsageEvaluatorCandidateV0[]>;

type SelectorUsageQueryBackendOptions = Pick<
  BuildSelectedQueryResultsV2Options,
  | "workspaceRoot"
  | "classnameTransform"
  | "pathAlias"
  | "sourceDocuments"
  | "styleFiles"
  | "analysisCache"
  | "styleDocumentForPath"
  | "typeResolver"
>;

export interface SelectorUsageEvaluatorCandidatePayloadV0 {
  readonly canonicalName: string;
  readonly totalReferences: number;
  readonly directReferenceCount: number;
  readonly editableDirectReferenceCount: number;
  readonly exactReferenceCount: number;
  readonly inferredOrBetterReferenceCount: number;
  readonly hasExpandedReferences: boolean;
  readonly hasStyleDependencyReferences: boolean;
  readonly hasAnyReferences: boolean;
  readonly allSites?: readonly SelectorUsageReferenceSiteV0[];
  readonly editableDirectSites?: readonly SelectorUsageEditableDirectSiteV0[];
}

export interface SelectorUsageReferenceSiteV0 {
  readonly filePath: string;
  readonly range: Range;
  readonly expansion: "direct" | "expanded";
  readonly referenceKind: "source" | "styleDependency";
}

export interface SelectorUsageEditableDirectSiteV0 {
  readonly filePath: string;
  readonly range: Range;
  readonly className: string;
}

export interface SelectorUsageRenderSummary {
  readonly totalReferences: number;
  readonly directReferenceCount: number;
  readonly hasExpandedReferences: boolean;
  readonly hasStyleDependencyReferences: boolean;
  readonly hasAnyReferences: boolean;
}

export interface SelectorUsageEvaluatorCandidateV0 {
  readonly kind: "selector-usage";
  readonly filePath: string;
  readonly queryId: string;
  readonly payload: SelectorUsageEvaluatorCandidatePayloadV0;
}

interface SelectorUsageCanonicalProducerSignalV0 {
  readonly evaluatorCandidates: {
    readonly results: readonly SelectorUsageEvaluatorCandidateV0[];
  };
}

export function resolveRustSelectorUsagePayloads(
  options: SelectorUsageQueryBackendOptions,
): readonly SelectorUsageEvaluatorCandidateV0[] {
  const input = buildEngineInputV2({
    workspaceRoot: options.workspaceRoot,
    classnameTransform: options.classnameTransform,
    pathAlias: options.pathAlias,
    sourceDocuments: options.sourceDocuments,
    styleFiles: options.styleFiles,
    analysisCache: options.analysisCache,
    styleDocumentForPath: options.styleDocumentForPath,
    typeResolver: options.typeResolver,
  });
  try {
    const signal = runRustSelectedQueryBackendJson<SelectorUsageCanonicalProducerSignalV0>(
      SELECTED_QUERY_RUNNER_COMMANDS.selectorUsageCanonicalProducer,
      input,
    );
    return signal.evaluatorCandidates.results;
  } catch (err) {
    if (isEngineShadowRunnerCancelledError(err)) return [];
    throw err;
  }
}

export async function resolveRustSelectorUsagePayloadsAsync(
  options: SelectorUsageQueryBackendOptions,
  runJson: RustSelectedQueryBackendJsonRunnerAsync = runRustSelectedQueryBackendJsonAsync,
): Promise<readonly SelectorUsageEvaluatorCandidateV0[]> {
  const input = await buildEngineInputV2Async({
    workspaceRoot: options.workspaceRoot,
    classnameTransform: options.classnameTransform,
    pathAlias: options.pathAlias,
    sourceDocuments: options.sourceDocuments,
    styleFiles: options.styleFiles,
    analysisCache: options.analysisCache,
    styleDocumentForPath: options.styleDocumentForPath,
    typeResolver: options.typeResolver,
  });
  try {
    const signal = await runJson<SelectorUsageCanonicalProducerSignalV0>(
      SELECTED_QUERY_RUNNER_COMMANDS.selectorUsageCanonicalProducer,
      input,
    );
    return signal.evaluatorCandidates.results;
  } catch (err) {
    if (isEngineShadowRunnerCancelledError(err)) return [];
    throw err;
  }
}

export function resolveRustSelectorUsagePayload(
  options: SelectorUsageQueryBackendOptions,
  filePath: string,
  canonicalName: string,
): SelectorUsageEvaluatorCandidatePayloadV0 | null {
  const match = resolveRustSelectorUsagePayloads(options).find(
    (candidate) => candidate.filePath === filePath && candidate.queryId === canonicalName,
  );
  return match?.payload ?? null;
}

export async function resolveRustSelectorUsagePayloadAsync(
  options: SelectorUsageQueryBackendOptions,
  filePath: string,
  canonicalName: string,
  runJson?: RustSelectedQueryBackendJsonRunnerAsync,
): Promise<SelectorUsageEvaluatorCandidatePayloadV0 | null> {
  const match = (await resolveRustSelectorUsagePayloadsAsync(options, runJson)).find(
    (candidate) => candidate.filePath === filePath && candidate.queryId === canonicalName,
  );
  return match?.payload ?? null;
}

export function resolveRustSelectorUsagePayloadsForWorkspaceTarget(
  args: {
    readonly workspaceRoot: string;
    readonly classnameTransform: BuildSelectedQueryResultsV2Options["classnameTransform"];
    readonly pathAlias: BuildSelectedQueryResultsV2Options["pathAlias"];
  },
  deps: Pick<ProviderDeps, "analysisCache" | "styleDocumentForPath" | "typeResolver">,
  filePath: string,
  cache?: SelectorUsagePayloadCache,
): readonly SelectorUsageEvaluatorCandidateV0[] {
  const cacheKey = selectorUsagePayloadCacheKey(args, filePath);
  const cached = cache?.get(cacheKey);
  if (cached) return cached;

  const { sourceFiles, styleFiles } = resolveWorkspaceCheckFilesSync({
    workspaceRoot: args.workspaceRoot,
  });
  const sourceDocuments = collectSourceDocuments(sourceFiles, deps.analysisCache);
  const payloads = resolveRustSelectorUsagePayloads({
    workspaceRoot: args.workspaceRoot,
    classnameTransform: args.classnameTransform,
    pathAlias: args.pathAlias,
    sourceDocuments,
    styleFiles,
    analysisCache: deps.analysisCache,
    styleDocumentForPath: deps.styleDocumentForPath,
    typeResolver: deps.typeResolver,
  }).filter((candidate) => candidate.filePath === filePath);
  if (payloads.length > 0) cache?.set(cacheKey, payloads);
  return payloads;
}

export async function resolveRustSelectorUsagePayloadsForWorkspaceTargetAsync(
  args: {
    readonly workspaceRoot: string;
    readonly classnameTransform: BuildSelectedQueryResultsV2Options["classnameTransform"];
    readonly pathAlias: BuildSelectedQueryResultsV2Options["pathAlias"];
  },
  deps: Pick<ProviderDeps, "analysisCache" | "styleDocumentForPath" | "typeResolver">,
  filePath: string,
  cache?: SelectorUsagePayloadCache,
  runJson?: RustSelectedQueryBackendJsonRunnerAsync,
): Promise<readonly SelectorUsageEvaluatorCandidateV0[]> {
  const cacheKey = selectorUsagePayloadCacheKey(args, filePath);
  const cached = cache?.get(cacheKey);
  if (cached) return cached;

  const { sourceFiles, styleFiles } = resolveWorkspaceCheckFilesSync({
    workspaceRoot: args.workspaceRoot,
  });
  const sourceDocuments = collectSourceDocuments(sourceFiles, deps.analysisCache);
  const payloads = (
    await resolveRustSelectorUsagePayloadsAsync(
      {
        workspaceRoot: args.workspaceRoot,
        classnameTransform: args.classnameTransform,
        pathAlias: args.pathAlias,
        sourceDocuments,
        styleFiles,
        analysisCache: deps.analysisCache,
        styleDocumentForPath: deps.styleDocumentForPath,
        typeResolver: deps.typeResolver,
      },
      runJson,
    )
  ).filter((candidate) => candidate.filePath === filePath);
  if (payloads.length > 0) cache?.set(cacheKey, payloads);
  return payloads;
}

export function resolveRustSelectorUsagePayloadForWorkspaceTarget(
  args: {
    readonly workspaceRoot: string;
    readonly classnameTransform: BuildSelectedQueryResultsV2Options["classnameTransform"];
    readonly pathAlias: BuildSelectedQueryResultsV2Options["pathAlias"];
  },
  deps: Pick<ProviderDeps, "analysisCache" | "styleDocumentForPath" | "typeResolver">,
  filePath: string,
  canonicalName: string,
  cache?: SelectorUsagePayloadCache,
): SelectorUsageEvaluatorCandidatePayloadV0 | null {
  const match = resolveRustSelectorUsagePayloadsForWorkspaceTarget(
    args,
    deps,
    filePath,
    cache,
  ).find((candidate) => candidate.queryId === canonicalName);
  return match?.payload ?? null;
}

export async function resolveRustSelectorUsagePayloadForWorkspaceTargetAsync(
  args: {
    readonly workspaceRoot: string;
    readonly classnameTransform: BuildSelectedQueryResultsV2Options["classnameTransform"];
    readonly pathAlias: BuildSelectedQueryResultsV2Options["pathAlias"];
  },
  deps: Pick<ProviderDeps, "analysisCache" | "styleDocumentForPath" | "typeResolver">,
  filePath: string,
  canonicalName: string,
  cache?: SelectorUsagePayloadCache,
  runJson?: RustSelectedQueryBackendJsonRunnerAsync,
): Promise<SelectorUsageEvaluatorCandidatePayloadV0 | null> {
  const match = (
    await resolveRustSelectorUsagePayloadsForWorkspaceTargetAsync(
      args,
      deps,
      filePath,
      cache,
      runJson,
    )
  ).find((candidate) => candidate.queryId === canonicalName);
  return match?.payload ?? null;
}

function selectorUsagePayloadCacheKey(
  args: {
    readonly workspaceRoot: string;
    readonly classnameTransform: BuildSelectedQueryResultsV2Options["classnameTransform"];
    readonly pathAlias: BuildSelectedQueryResultsV2Options["pathAlias"];
  },
  filePath: string,
): string {
  const pathAliasKey = Object.entries(args.pathAlias)
    .toSorted(([left], [right]) => left.localeCompare(right))
    .map(([key, value]) => `${key}=${value}`)
    .join("|");
  return [args.workspaceRoot, args.classnameTransform, pathAliasKey, filePath].join("\0");
}

export function buildSelectorUsageRenderSummaryFromRustPayload(
  payload: SelectorUsageEvaluatorCandidatePayloadV0,
): SelectorUsageRenderSummary {
  return {
    totalReferences: payload.totalReferences,
    directReferenceCount: payload.directReferenceCount,
    hasExpandedReferences: payload.hasExpandedReferences,
    hasStyleDependencyReferences: payload.hasStyleDependencyReferences,
    hasAnyReferences: payload.hasAnyReferences,
  };
}

export function buildSelectorUsageLocationsFromRustPayload(
  payload: SelectorUsageEvaluatorCandidatePayloadV0,
) {
  return payload.allSites?.map((site) => ({
    filePath: site.filePath,
    range: site.range,
  }));
}

export function buildSelectorUsageEditableDirectSitesFromRustPayload(
  payload: SelectorUsageEvaluatorCandidatePayloadV0,
) {
  return payload.editableDirectSites?.map((site) => ({
    filePath: site.filePath,
    range: site.range,
    className: site.className,
  }));
}
