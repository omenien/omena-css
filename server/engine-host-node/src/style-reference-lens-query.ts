import type { Position, ShowReferencesLocation } from "@css-module-explainer/shared";
import {
  listCanonicalSelectors,
  readSelectorUsageSummary,
} from "../../engine-core-ts/src/core/query";
import type { StyleDocumentHIR } from "../../engine-core-ts/src/core/hir/style-types";
import type { ProviderDeps } from "../../engine-core-ts/src/provider-deps";
import { pathToFileUrl } from "../../engine-core-ts/src/core/util/text-utils";
import {
  resolveSelectedQueryBackendKind,
  usesRustSelectorUsageBackend,
} from "./selected-query-backend";
import {
  buildSelectorUsageLocationsFromRustPayload,
  buildSelectorUsageRenderSummaryFromRustPayload,
  resolveRustSelectorUsagePayloadsForWorkspaceTargetAsync,
  resolveRustSelectorUsagePayloadsForWorkspaceTarget,
  type resolveRustSelectorUsagePayloadForWorkspaceTargetAsync,
  type resolveRustSelectorUsagePayloadForWorkspaceTarget,
  type SelectorUsageEvaluatorCandidatePayloadV0,
  type SelectorUsagePayloadCache,
  type SelectorUsageRenderSummary,
} from "./selector-usage-query-backend";
import {
  buildSelectorReferenceRenderSummaryFromRustGraph,
  resolveRustStyleSelectorReferenceSummariesForWorkspaceTargetAsync,
  resolveRustStyleSelectorReferenceSummariesForWorkspaceTarget,
  type StyleSelectorReferenceQueryOptions,
} from "./style-selector-reference-query";
import type { StyleSemanticGraphSelectorReferenceSummaryV0 } from "./style-semantic-graph-query-backend";

export interface StyleReferenceLensSummary {
  readonly position: Position;
  readonly title: string;
  readonly locations: readonly ShowReferencesLocation[];
}

export interface StyleReferenceLensQueryOptions extends StyleSelectorReferenceQueryOptions {
  readonly env?: NodeJS.ProcessEnv;
  readonly readRustSelectorUsagePayloadForWorkspaceTarget?: typeof resolveRustSelectorUsagePayloadForWorkspaceTarget;
  readonly readRustSelectorUsagePayloadsForWorkspaceTarget?: typeof resolveRustSelectorUsagePayloadsForWorkspaceTarget;
  readonly readRustSelectorUsagePayloadForWorkspaceTargetAsync?: typeof resolveRustSelectorUsagePayloadForWorkspaceTargetAsync;
  readonly readRustSelectorUsagePayloadsForWorkspaceTargetAsync?: typeof resolveRustSelectorUsagePayloadsForWorkspaceTargetAsync;
  readonly selectorUsagePayloadCache?: SelectorUsagePayloadCache;
}

export function resolveStyleReferenceLenses(
  filePath: string,
  styleDocument: StyleDocumentHIR,
  deps: Pick<
    ProviderDeps,
    | "analysisCache"
    | "semanticReferenceIndex"
    | "styleDependencyGraph"
    | "styleDocumentForPath"
    | "typeResolver"
    | "workspaceRoot"
    | "settings"
    | "readStyleFile"
  >,
  options: StyleReferenceLensQueryOptions = {},
): readonly StyleReferenceLensSummary[] {
  const depsSelectorUsageCache = (
    deps as {
      readonly selectorUsagePayloadCache?: SelectorUsagePayloadCache;
    }
  ).selectorUsagePayloadCache;
  const queryOptions =
    options.selectorUsagePayloadCache || !depsSelectorUsageCache
      ? options
      : {
          ...options,
          selectorUsagePayloadCache: depsSelectorUsageCache,
        };
  const lenses: StyleReferenceLensSummary[] = [];
  const selectedQueryBackend = resolveSelectedQueryBackendKind(queryOptions.env);
  const canUseRustSelectorUsage = usesRustSelectorUsageBackend(selectedQueryBackend);
  const readRustSelectorUsagePayload = createRustSelectorUsagePayloadReader(
    {
      workspaceRoot: deps.workspaceRoot,
      classnameTransform: deps.settings.scss.classnameTransform,
      pathAlias: deps.settings.pathAlias,
    },
    deps,
    filePath,
    queryOptions,
  );
  const rustGraphReferenceLensSummariesByName = resolveRustGraphReferenceLensSummariesByName(
    deps,
    filePath,
    queryOptions,
  );
  for (const selector of listCanonicalSelectors(styleDocument)) {
    const graphLensResolution =
      rustGraphReferenceLensSummariesByName?.get(selector.canonicalName) ?? null;
    const selectorUsageLensResolution =
      canUseRustSelectorUsage &&
      (!graphLensResolution || !graphLensResolution.usage.hasAnyReferences)
        ? resolveRustReferenceLensSummary(selector.canonicalName, readRustSelectorUsagePayload)
        : null;
    const rustLensResolution = graphLensResolution?.usage.hasAnyReferences
      ? graphLensResolution
      : (selectorUsageLensResolution ?? graphLensResolution);
    if (rustLensResolution) {
      if (!rustLensResolution.usage.hasAnyReferences) continue;
      lenses.push({
        position: selector.range.start,
        title: formatReferenceLensTitle(rustLensResolution.usage),
        locations: rustLensResolution.locations,
      });
      continue;
    }

    const currentUsage = readSelectorUsageSummary(deps, filePath, selector.canonicalName);
    if (!currentUsage.hasAnyReferences) continue;

    lenses.push({
      position: selector.range.start,
      title: formatReferenceLensTitle(currentUsage),
      locations: currentUsage.allSites.map((site) => ({
        uri: site.uri,
        range: site.range,
      })),
    });
  }
  return lenses;
}

export async function resolveStyleReferenceLensesAsync(
  filePath: string,
  styleDocument: StyleDocumentHIR,
  deps: Pick<
    ProviderDeps,
    | "analysisCache"
    | "semanticReferenceIndex"
    | "styleDependencyGraph"
    | "styleDocumentForPath"
    | "typeResolver"
    | "workspaceRoot"
    | "settings"
    | "readStyleFile"
  >,
  options: StyleReferenceLensQueryOptions = {},
): Promise<readonly StyleReferenceLensSummary[]> {
  const depsSelectorUsageCache = (
    deps as {
      readonly selectorUsagePayloadCache?: SelectorUsagePayloadCache;
    }
  ).selectorUsagePayloadCache;
  const queryOptions =
    options.selectorUsagePayloadCache || !depsSelectorUsageCache
      ? options
      : {
          ...options,
          selectorUsagePayloadCache: depsSelectorUsageCache,
        };
  const lenses: StyleReferenceLensSummary[] = [];
  const selectedQueryBackend = resolveSelectedQueryBackendKind(queryOptions.env);
  const canUseRustSelectorUsage = usesRustSelectorUsageBackend(selectedQueryBackend);
  const readRustSelectorUsagePayload = createRustSelectorUsagePayloadReaderAsync(
    {
      workspaceRoot: deps.workspaceRoot,
      classnameTransform: deps.settings.scss.classnameTransform,
      pathAlias: deps.settings.pathAlias,
    },
    deps,
    filePath,
    queryOptions,
  );
  const rustGraphReferenceLensSummariesByName =
    await resolveRustGraphReferenceLensSummariesByNameAsync(deps, filePath, queryOptions);
  const selectorLenses = await Promise.all(
    listCanonicalSelectors(styleDocument).map(async (selector) => {
      const graphLensResolution =
        rustGraphReferenceLensSummariesByName?.get(selector.canonicalName) ?? null;
      const selectorUsageLensResolution =
        canUseRustSelectorUsage &&
        (!graphLensResolution || !graphLensResolution.usage.hasAnyReferences)
          ? await resolveRustReferenceLensSummaryAsync(
              selector.canonicalName,
              readRustSelectorUsagePayload,
            )
          : null;
      const rustLensResolution = graphLensResolution?.usage.hasAnyReferences
        ? graphLensResolution
        : (selectorUsageLensResolution ?? graphLensResolution);
      if (rustLensResolution) {
        if (!rustLensResolution.usage.hasAnyReferences) return null;
        return {
          position: selector.range.start,
          title: formatReferenceLensTitle(rustLensResolution.usage),
          locations: rustLensResolution.locations,
        };
      }

      const currentUsage = readSelectorUsageSummary(deps, filePath, selector.canonicalName);
      if (!currentUsage.hasAnyReferences) return null;

      return {
        position: selector.range.start,
        title: formatReferenceLensTitle(currentUsage),
        locations: currentUsage.allSites.map((site) => ({
          uri: site.uri,
          range: site.range,
        })),
      };
    }),
  );
  lenses.push(...selectorLenses.filter((lens): lens is StyleReferenceLensSummary => Boolean(lens)));
  return lenses;
}

function createRustSelectorUsagePayloadReader(
  args: {
    readonly workspaceRoot: string;
    readonly classnameTransform: ProviderDeps["settings"]["scss"]["classnameTransform"];
    readonly pathAlias: ProviderDeps["settings"]["pathAlias"];
  },
  deps: Pick<ProviderDeps, "analysisCache" | "styleDocumentForPath" | "typeResolver">,
  filePath: string,
  options: StyleReferenceLensQueryOptions,
): (canonicalName: string) => SelectorUsageEvaluatorCandidatePayloadV0 | null {
  if (options.readRustSelectorUsagePayloadForWorkspaceTarget) {
    return (canonicalName) =>
      options.readRustSelectorUsagePayloadForWorkspaceTarget!(args, deps, filePath, canonicalName);
  }

  const readPayloads =
    options.readRustSelectorUsagePayloadsForWorkspaceTarget ??
    resolveRustSelectorUsagePayloadsForWorkspaceTarget;
  let payloadsByName: ReadonlyMap<string, SelectorUsageEvaluatorCandidatePayloadV0> | null = null;

  return (canonicalName) => {
    if (!payloadsByName) {
      payloadsByName = new Map(
        readPayloads(args, deps, filePath, options.selectorUsagePayloadCache).map((candidate) => [
          candidate.queryId,
          candidate.payload,
        ]),
      );
    }
    return payloadsByName.get(canonicalName) ?? null;
  };
}

function createRustSelectorUsagePayloadReaderAsync(
  args: {
    readonly workspaceRoot: string;
    readonly classnameTransform: ProviderDeps["settings"]["scss"]["classnameTransform"];
    readonly pathAlias: ProviderDeps["settings"]["pathAlias"];
  },
  deps: Pick<ProviderDeps, "analysisCache" | "styleDocumentForPath" | "typeResolver">,
  filePath: string,
  options: StyleReferenceLensQueryOptions,
): (canonicalName: string) => Promise<SelectorUsageEvaluatorCandidatePayloadV0 | null> {
  if (options.readRustSelectorUsagePayloadForWorkspaceTargetAsync) {
    return (canonicalName) =>
      options.readRustSelectorUsagePayloadForWorkspaceTargetAsync!(
        args,
        deps,
        filePath,
        canonicalName,
        options.selectorUsagePayloadCache,
        options.runRustSelectedQueryBackendJsonAsync,
      );
  }

  const readPayloads =
    options.readRustSelectorUsagePayloadsForWorkspaceTargetAsync ??
    resolveRustSelectorUsagePayloadsForWorkspaceTargetAsync;
  let payloadsByNamePromise: Promise<
    ReadonlyMap<string, SelectorUsageEvaluatorCandidatePayloadV0>
  > | null = null;

  return async (canonicalName) => {
    payloadsByNamePromise ??= readPayloads(
      args,
      deps,
      filePath,
      options.selectorUsagePayloadCache,
      options.runRustSelectedQueryBackendJsonAsync,
    ).then(
      (payloads) =>
        new Map(payloads.map((candidate) => [candidate.queryId, candidate.payload] as const)),
    );
    return (await payloadsByNamePromise).get(canonicalName) ?? null;
  };
}

function resolveRustGraphReferenceLensSummariesByName(
  deps: Pick<
    ProviderDeps,
    | "analysisCache"
    | "styleDocumentForPath"
    | "typeResolver"
    | "workspaceRoot"
    | "settings"
    | "readStyleFile"
  >,
  filePath: string,
  options: StyleReferenceLensQueryOptions,
): ReadonlyMap<
  string,
  {
    readonly usage: SelectorUsageRenderSummary;
    readonly locations: readonly ShowReferencesLocation[];
  }
> | null {
  const selectors = resolveRustStyleSelectorReferenceSummariesForWorkspaceTarget(
    {
      filePath,
    },
    deps,
    options,
  );
  if (!selectors) return null;
  return new Map(
    selectors.map((selector) => [selector.localName, buildRustGraphReferenceLensSummary(selector)]),
  );
}

async function resolveRustGraphReferenceLensSummariesByNameAsync(
  deps: Pick<
    ProviderDeps,
    | "analysisCache"
    | "styleDocumentForPath"
    | "typeResolver"
    | "workspaceRoot"
    | "settings"
    | "readStyleFile"
  >,
  filePath: string,
  options: StyleReferenceLensQueryOptions,
): Promise<ReadonlyMap<
  string,
  {
    readonly usage: SelectorUsageRenderSummary;
    readonly locations: readonly ShowReferencesLocation[];
  }
> | null> {
  const selectors = await resolveRustStyleSelectorReferenceSummariesForWorkspaceTargetAsync(
    {
      filePath,
    },
    deps,
    options,
  );
  if (!selectors) return null;
  return new Map(
    selectors.map((selector) => [selector.localName, buildRustGraphReferenceLensSummary(selector)]),
  );
}

function buildRustGraphReferenceLensSummary(
  selector: StyleSemanticGraphSelectorReferenceSummaryV0,
): {
  readonly usage: SelectorUsageRenderSummary;
  readonly locations: readonly ShowReferencesLocation[];
} {
  return {
    usage: buildSelectorReferenceRenderSummaryFromRustGraph(selector),
    locations: selector.sites.map((site) => ({
      uri: pathToFileUrl(site.filePath),
      range: site.range,
    })),
  };
}

function resolveRustReferenceLensSummary(
  canonicalName: string,
  readRustSelectorUsagePayload: (
    canonicalName: string,
  ) => SelectorUsageEvaluatorCandidatePayloadV0 | null,
): {
  readonly usage: SelectorUsageRenderSummary;
  readonly locations: readonly ShowReferencesLocation[];
} | null {
  const payload = readRustSelectorUsagePayload(canonicalName);
  if (!payload) return null;
  return {
    usage: buildSelectorUsageRenderSummaryFromRustPayload(payload),
    locations:
      buildSelectorUsageLocationsFromRustPayload(payload)?.map((site) => ({
        uri: pathToFileUrl(site.filePath),
        range: site.range,
      })) ?? [],
  };
}

async function resolveRustReferenceLensSummaryAsync(
  canonicalName: string,
  readRustSelectorUsagePayload: (
    canonicalName: string,
  ) => Promise<SelectorUsageEvaluatorCandidatePayloadV0 | null>,
): Promise<{
  readonly usage: SelectorUsageRenderSummary;
  readonly locations: readonly ShowReferencesLocation[];
} | null> {
  const payload = await readRustSelectorUsagePayload(canonicalName);
  if (!payload) return null;
  return {
    usage: buildSelectorUsageRenderSummaryFromRustPayload(payload),
    locations:
      buildSelectorUsageLocationsFromRustPayload(payload)?.map((site) => ({
        uri: pathToFileUrl(site.filePath),
        range: site.range,
      })) ?? [],
  };
}

function formatReferenceLensTitle(usage: SelectorUsageRenderSummary): string {
  const base = `${usage.totalReferences} reference${usage.totalReferences === 1 ? "" : "s"}`;
  const details: string[] = [];
  if (usage.totalReferences !== usage.directReferenceCount) {
    details.push(`${usage.directReferenceCount} direct`);
  }
  if (usage.hasStyleDependencyReferences) {
    details.push("composed");
  }
  if (usage.hasExpandedReferences) {
    details.push("dynamic");
  }
  return details.length > 0 ? `${base} (${details.join(", ")})` : base;
}
