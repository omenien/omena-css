import type { StyleDocumentHIR } from "../../engine-core-ts/src/core/hir/style-types";
import type { ProviderDeps } from "../../engine-core-ts/src/provider-deps";
import {
  resolveSelectedQueryBackendKind,
  usesRustStyleSemanticGraphBackend,
} from "./selected-query-backend";
import {
  buildStyleSemanticGraphDesignTokenRankedReferenceReadModels,
  resolveRustStyleSemanticGraphForWorkspaceTarget,
  resolveRustStyleSemanticGraphForWorkspaceTargetAsync,
  type StyleSemanticGraphCache,
  type StyleSemanticGraphDesignTokenDeclarationCandidateV0,
  type StyleSemanticGraphDesignTokenRankedReferenceReadModel,
  type StyleSemanticGraphQueryOptions,
  type StyleSemanticGraphSummaryV0,
} from "./style-semantic-graph-query-backend";

export interface StyleDesignTokenRankingQueryOptions extends Pick<
  StyleSemanticGraphQueryOptions,
  | "engineInput"
  | "sourceDocuments"
  | "styleFiles"
  | "styleSemanticGraphCache"
  | "runRustSelectedQueryBackendJson"
  | "runRustSelectedQueryBackendJsonAsync"
> {
  readonly env?: NodeJS.ProcessEnv;
  readonly readRustStyleSemanticGraphForWorkspaceTarget?: typeof resolveRustStyleSemanticGraphForWorkspaceTarget;
  readonly readRustStyleSemanticGraphForWorkspaceTargetAsync?: typeof resolveRustStyleSemanticGraphForWorkspaceTargetAsync;
}

export type StyleDesignTokenRankingDeps = Pick<
  ProviderDeps,
  | "analysisCache"
  | "styleDocumentForPath"
  | "settings"
  | "typeResolver"
  | "workspaceRoot"
  | "readStyleFile"
> & {
  readonly styleSemanticGraphCache?: StyleSemanticGraphCache;
};

export type StyleDesignTokenDeclarationCandidateReadModel =
  StyleSemanticGraphDesignTokenDeclarationCandidateV0;

export function resolveStyleDesignTokenRankingForReference(
  args: {
    readonly filePath: string;
    readonly styleDocument: StyleDocumentHIR;
    readonly customPropertyRef: StyleDocumentHIR["customPropertyRefs"][number];
  },
  deps: StyleDesignTokenRankingDeps,
  options: StyleDesignTokenRankingQueryOptions = {},
): StyleSemanticGraphDesignTokenRankedReferenceReadModel | null {
  const rankings = resolveStyleDesignTokenRankingsForDocument(args, deps, options);
  return rankings?.find((readModel) => readModel.reference === args.customPropertyRef) ?? null;
}

export function resolveStyleDesignTokenRankingsForDocument(
  args: {
    readonly filePath: string;
    readonly styleDocument: StyleDocumentHIR;
  },
  deps: StyleDesignTokenRankingDeps,
  options: StyleDesignTokenRankingQueryOptions = {},
): readonly StyleSemanticGraphDesignTokenRankedReferenceReadModel[] | null {
  const graph = resolveStyleDesignTokenGraphForDocument(args, deps, options);
  return graph
    ? buildStyleSemanticGraphDesignTokenRankedReferenceReadModels(graph, args.styleDocument)
    : null;
}

export function resolveStyleDesignTokenDeclarationCandidatesForDocument(
  args: {
    readonly filePath: string;
    readonly styleDocument: StyleDocumentHIR;
  },
  deps: StyleDesignTokenRankingDeps,
  options: StyleDesignTokenRankingQueryOptions = {},
): readonly StyleDesignTokenDeclarationCandidateReadModel[] | null {
  const graph = resolveStyleDesignTokenGraphForDocument(args, deps, options);
  return graph?.designTokenSemantics?.declarationCandidates ?? null;
}

export async function resolveStyleDesignTokenRankingForReferenceAsync(
  args: {
    readonly filePath: string;
    readonly styleDocument: StyleDocumentHIR;
    readonly customPropertyRef: StyleDocumentHIR["customPropertyRefs"][number];
  },
  deps: StyleDesignTokenRankingDeps,
  options: StyleDesignTokenRankingQueryOptions = {},
): Promise<StyleSemanticGraphDesignTokenRankedReferenceReadModel | null> {
  const rankings = await resolveStyleDesignTokenRankingsForDocumentAsync(args, deps, options);
  return rankings?.find((readModel) => readModel.reference === args.customPropertyRef) ?? null;
}

export async function resolveStyleDesignTokenRankingsForDocumentAsync(
  args: {
    readonly filePath: string;
    readonly styleDocument: StyleDocumentHIR;
  },
  deps: StyleDesignTokenRankingDeps,
  options: StyleDesignTokenRankingQueryOptions = {},
): Promise<readonly StyleSemanticGraphDesignTokenRankedReferenceReadModel[] | null> {
  const graph = await resolveStyleDesignTokenGraphForDocumentAsync(args, deps, options);
  return graph
    ? buildStyleSemanticGraphDesignTokenRankedReferenceReadModels(graph, args.styleDocument)
    : null;
}

export async function resolveStyleDesignTokenDeclarationCandidatesForDocumentAsync(
  args: {
    readonly filePath: string;
    readonly styleDocument: StyleDocumentHIR;
  },
  deps: StyleDesignTokenRankingDeps,
  options: StyleDesignTokenRankingQueryOptions = {},
): Promise<readonly StyleDesignTokenDeclarationCandidateReadModel[] | null> {
  const graph = await resolveStyleDesignTokenGraphForDocumentAsync(args, deps, options);
  return graph?.designTokenSemantics?.declarationCandidates ?? null;
}

function resolveStyleDesignTokenGraphForDocument(
  args: {
    readonly filePath: string;
  },
  deps: StyleDesignTokenRankingDeps,
  options: StyleDesignTokenRankingQueryOptions,
): StyleSemanticGraphSummaryV0 | null {
  if (!usesRustStyleSemanticGraphBackend(resolveSelectedQueryBackendKind(options.env))) {
    return null;
  }

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
      args.filePath,
      withDepsStyleSemanticGraphCache(deps, options),
    );
  } catch {
    return null;
  }
}

async function resolveStyleDesignTokenGraphForDocumentAsync(
  args: {
    readonly filePath: string;
  },
  deps: StyleDesignTokenRankingDeps,
  options: StyleDesignTokenRankingQueryOptions,
): Promise<StyleSemanticGraphSummaryV0 | null> {
  if (!usesRustStyleSemanticGraphBackend(resolveSelectedQueryBackendKind(options.env))) {
    return null;
  }

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
      args.filePath,
      withDepsStyleSemanticGraphCache(deps, options),
    );
  } catch {
    return null;
  }
}

function withDepsStyleSemanticGraphCache(
  deps: { readonly styleSemanticGraphCache?: StyleSemanticGraphCache },
  options: StyleDesignTokenRankingQueryOptions,
): StyleDesignTokenRankingQueryOptions {
  if (options.styleSemanticGraphCache || !deps.styleSemanticGraphCache) return options;
  return { ...options, styleSemanticGraphCache: deps.styleSemanticGraphCache };
}
