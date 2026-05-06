import type { ProviderDeps } from "../../engine-core-ts/src/provider-deps";
import { pathToFileUrl } from "../../engine-core-ts/src/core/util/text-utils";
import type { ResolvedReferenceSite } from "../../engine-core-ts/src/core/query/find-references";
import type {
  SelectorReferenceRewritePolicy,
  SelectorRewriteSafetySummary,
} from "../../engine-core-ts/src/core/query/read-selector-rewrite-safety";
import {
  resolveSelectedQueryBackendKind,
  usesRustStyleSemanticGraphBackend,
} from "./selected-query-backend";
import {
  resolveRustStyleSemanticGraphForWorkspaceTargetAsync,
  resolveRustStyleSemanticGraphForWorkspaceTarget,
  type StyleSemanticGraphCache,
  type StyleSemanticGraphQueryOptions,
  type StyleSemanticGraphSummaryV0,
  type StyleSemanticGraphSelectorReferenceSummaryV0,
} from "./style-semantic-graph-query-backend";
import type { SelectorUsageRenderSummary } from "./selector-usage-query-backend";

export interface StyleSelectorReferenceQueryOptions extends Pick<
  StyleSemanticGraphQueryOptions,
  | "engineInput"
  | "sourceDocuments"
  | "styleFiles"
  | "styleSemanticGraphCache"
  | "styleSemanticGraphBatchOutputCache"
  | "runRustSelectedQueryBackendJson"
  | "runRustSelectedQueryBackendJsonAsync"
> {
  readonly env?: NodeJS.ProcessEnv;
  readonly readRustStyleSemanticGraphForWorkspaceTarget?: typeof resolveRustStyleSemanticGraphForWorkspaceTarget;
  readonly readRustStyleSemanticGraphForWorkspaceTargetAsync?: typeof resolveRustStyleSemanticGraphForWorkspaceTargetAsync;
}

type StyleSelectorReferenceDeps = Pick<
  ProviderDeps,
  | "analysisCache"
  | "settings"
  | "styleDocumentForPath"
  | "typeResolver"
  | "workspaceRoot"
  | "readStyleFile"
> & {
  readonly styleSemanticGraphCache?: StyleSemanticGraphCache;
};

export function resolveRustStyleSelectorReferenceSummaryForWorkspaceTarget(
  args: {
    readonly filePath: string;
    readonly canonicalName: string;
  },
  deps: StyleSelectorReferenceDeps,
  options: StyleSelectorReferenceQueryOptions = {},
): StyleSemanticGraphSelectorReferenceSummaryV0 | null {
  if (!usesRustStyleSemanticGraphBackend(resolveSelectedQueryBackendKind(options.env))) {
    return null;
  }

  const graph = safeResolveRustStyleSemanticGraphForWorkspaceTarget(args.filePath, deps, options);
  if (!graph) return null;
  if (!hasSourceBackedSelectorReferenceEvidence(graph)) return null;

  return (
    graph.selectorReferenceEngine.selectors.find(
      (selector) => selector.localName === args.canonicalName,
    ) ?? null
  );
}

export async function resolveRustStyleSelectorReferenceSummaryForWorkspaceTargetAsync(
  args: {
    readonly filePath: string;
    readonly canonicalName: string;
  },
  deps: StyleSelectorReferenceDeps,
  options: StyleSelectorReferenceQueryOptions = {},
): Promise<StyleSemanticGraphSelectorReferenceSummaryV0 | null> {
  if (!usesRustStyleSemanticGraphBackend(resolveSelectedQueryBackendKind(options.env))) {
    return null;
  }

  const graph = await safeResolveRustStyleSemanticGraphForWorkspaceTargetAsync(
    args.filePath,
    deps,
    options,
  );
  if (!graph) return null;
  if (!hasSourceBackedSelectorReferenceEvidence(graph)) return null;

  return (
    graph.selectorReferenceEngine.selectors.find(
      (selector) => selector.localName === args.canonicalName,
    ) ?? null
  );
}

export function resolveRustStyleSelectorReferenceSummariesForWorkspaceTarget(
  args: {
    readonly filePath: string;
  },
  deps: StyleSelectorReferenceDeps,
  options: StyleSelectorReferenceQueryOptions = {},
): readonly StyleSemanticGraphSelectorReferenceSummaryV0[] | null {
  if (!usesRustStyleSemanticGraphBackend(resolveSelectedQueryBackendKind(options.env))) {
    return null;
  }

  const graph = safeResolveRustStyleSemanticGraphForWorkspaceTarget(args.filePath, deps, options);
  if (!graph) return null;
  if (!hasSourceBackedSelectorReferenceEvidence(graph)) return null;

  return graph.selectorReferenceEngine.selectors;
}

export async function resolveRustStyleSelectorReferenceSummariesForWorkspaceTargetAsync(
  args: {
    readonly filePath: string;
  },
  deps: StyleSelectorReferenceDeps,
  options: StyleSelectorReferenceQueryOptions = {},
): Promise<readonly StyleSemanticGraphSelectorReferenceSummaryV0[] | null> {
  if (!usesRustStyleSemanticGraphBackend(resolveSelectedQueryBackendKind(options.env))) {
    return null;
  }

  const graph = await safeResolveRustStyleSemanticGraphForWorkspaceTargetAsync(
    args.filePath,
    deps,
    options,
  );
  if (!graph) return null;
  if (!hasSourceBackedSelectorReferenceEvidence(graph)) return null;

  return graph.selectorReferenceEngine.selectors;
}

function safeResolveRustStyleSemanticGraphForWorkspaceTarget(
  filePath: string,
  deps: StyleSelectorReferenceDeps,
  options: StyleSelectorReferenceQueryOptions,
): StyleSemanticGraphSummaryV0 | null {
  const queryOptions =
    options.styleSemanticGraphCache || !deps.styleSemanticGraphCache
      ? options
      : { ...options, styleSemanticGraphCache: deps.styleSemanticGraphCache };
  try {
    return (
      options.readRustStyleSemanticGraphForWorkspaceTarget ??
      resolveRustStyleSemanticGraphForWorkspaceTarget
    )(
      {
        workspaceRoot: deps.workspaceRoot,
        classnameTransform: deps.settings.scss.classnameTransform,
        pathAlias: deps.settings.pathAlias,
      },
      deps,
      filePath,
      queryOptions,
    );
  } catch {
    return null;
  }
}

async function safeResolveRustStyleSemanticGraphForWorkspaceTargetAsync(
  filePath: string,
  deps: StyleSelectorReferenceDeps,
  options: StyleSelectorReferenceQueryOptions,
): Promise<StyleSemanticGraphSummaryV0 | null> {
  const queryOptions =
    options.styleSemanticGraphCache || !deps.styleSemanticGraphCache
      ? options
      : { ...options, styleSemanticGraphCache: deps.styleSemanticGraphCache };
  try {
    return await (
      options.readRustStyleSemanticGraphForWorkspaceTargetAsync ??
      resolveRustStyleSemanticGraphForWorkspaceTargetAsync
    )(
      {
        workspaceRoot: deps.workspaceRoot,
        classnameTransform: deps.settings.scss.classnameTransform,
        pathAlias: deps.settings.pathAlias,
      },
      deps,
      filePath,
      queryOptions,
    );
  } catch {
    return null;
  }
}

function hasSourceBackedSelectorReferenceEvidence(graph: StyleSemanticGraphSummaryV0): boolean {
  if (graph.selectorReferenceEngine.totalReferenceSites > 0) return true;

  const evidence = graph.sourceInputEvidence;
  if (!isObjectRecord(evidence)) return false;
  const referenceSiteIdentity = evidence.referenceSiteIdentity;
  return (
    isObjectRecord(referenceSiteIdentity) &&
    referenceSiteIdentity.status === "ready" &&
    typeof referenceSiteIdentity.referenceSiteCount === "number"
  );
}

function isObjectRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}

export function buildSelectorReferenceRenderSummaryFromRustGraph(
  selector: StyleSemanticGraphSelectorReferenceSummaryV0,
): SelectorUsageRenderSummary {
  return {
    totalReferences: selector.totalReferences,
    directReferenceCount: selector.directReferenceCount,
    hasExpandedReferences: selector.hasExpandedReferences,
    hasStyleDependencyReferences: selector.hasStyleDependencyReferences,
    hasAnyReferences: selector.hasAnyReferences,
  };
}

export function buildSelectorReferenceEditableDirectSitesFromRustGraph(
  selector: StyleSemanticGraphSelectorReferenceSummaryV0,
): readonly ResolvedReferenceSite[] {
  return selector.editableDirectSites.map((site) => ({
    uri: pathToFileUrl(site.filePath),
    range: site.range,
    className: site.className,
    selectorCertainty: "exact",
    expansion: "direct",
    referenceKind: "source",
  }));
}

export function buildSelectorReferenceRewriteSafetyFromRustGraph(
  base: SelectorRewriteSafetySummary,
  selector: StyleSemanticGraphSelectorReferenceSummaryV0,
): SelectorRewriteSafetySummary {
  const editableDirectSites = buildSelectorReferenceEditableDirectSitesFromRustGraph(selector);
  const hasBlockingStyleDependencyReferences = selector.hasStyleDependencyReferences;
  const hasBlockingExpandedReferences = selector.hasExpandedReferences;
  const referenceRewritePolicy: SelectorReferenceRewritePolicy =
    hasBlockingStyleDependencyReferences
      ? "blockedByStyleDependencies"
      : hasBlockingExpandedReferences
        ? "blockedByExpandedReferences"
        : "directOnly";

  return {
    ...base,
    usage: {
      ...base.usage,
      editableDirectSites,
      totalReferences: selector.totalReferences,
      directReferenceCount: selector.directReferenceCount,
      hasExpandedReferences: selector.hasExpandedReferences,
      hasStyleDependencyReferences: selector.hasStyleDependencyReferences,
      hasAnyReferences: selector.hasAnyReferences,
    },
    directSites: editableDirectSites,
    referenceRewritePolicy,
    hasBlockingExpandedReferences,
    hasBlockingStyleDependencyReferences,
  };
}
