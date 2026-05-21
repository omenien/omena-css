import type { WorkspaceRegistry } from "../workspace/workspace-registry";
import {
  collectWatchedFileChangeInputs,
  type RuntimeOpenDocumentLookup,
} from "./watched-file-changes";
import { createRuntimeDependencySnapshot, snapshotOpenDocuments } from "./dependency-snapshot";
import { planWatchedFileInvalidation, type RuntimeFileEvent } from "./invalidation-planner";
import { isPackageManifestPath } from "./shared-runtime-caches";

export interface WatchedFileApplicationDocuments extends RuntimeOpenDocumentLookup {
  all(): readonly { readonly uri: string }[];
}

export interface ApplyWatchedFileChangesArgs {
  readonly registry: WorkspaceRegistry;
  readonly documents: WatchedFileApplicationDocuments;
  readonly events: readonly RuntimeFileEvent[];
}

export interface WatchedFileApplicationResult {
  readonly affectedStyleUris: readonly string[];
  readonly affectedSourceUris: readonly string[];
}

export function applyWatchedFileChanges(
  args: ApplyWatchedFileChangesArgs,
): WatchedFileApplicationResult {
  const snapshot = createRuntimeDependencySnapshot(
    args.registry.allDeps(),
    snapshotOpenDocuments({
      documents: args.documents,
      getWorkspaceRoot: (uri) => args.registry.getDeps(uri)?.workspaceRoot ?? null,
    }),
  );
  const changes = collectWatchedFileChangeInputs(
    args.events,
    {
      documents: args.documents,
      getDepsForFilePath: (filePath) => args.registry.getDepsForFilePath(filePath),
    },
    snapshot,
  );
  const plan = planWatchedFileInvalidation(changes, snapshot.openDocuments);

  const affectedDeps = args.registry
    .allDeps()
    .filter(
      (deps) =>
        plan.aliasRebuildRoots.includes(deps.workspaceRoot) ||
        plan.typeResolverInvalidationRoots.includes(deps.workspaceRoot),
    );

  for (const change of changes) {
    if (change.kind === "source" && isPackageManifestPath(change.filePath)) {
      const deps = args.registry.getDepsForFilePath(change.filePath);
      deps?.invalidatePackageManifestCache?.(change.filePath);
      continue;
    }
    if (change.kind !== "style" || !change.semanticsChanged) continue;
    const deps = args.registry.getDepsForFilePath(change.filePath);
    if (!deps) continue;
    deps.clearStyleSemanticGraphCache?.();
    if (plan.stylePathsToInvalidate.includes(change.filePath)) {
      deps.invalidateStyle(change.filePath);
    }
    if (plan.stylePathsToPush.includes(change.filePath)) {
      deps.pushStyleFile(change.filePath);
    }
  }

  for (const deps of affectedDeps) {
    if (plan.aliasRebuildRoots.includes(deps.workspaceRoot)) {
      deps.rebuildAliasResolver(deps.settings.pathAlias);
    }
    if (plan.typeResolverInvalidationRoots.includes(deps.workspaceRoot)) {
      deps.typeResolver.invalidate(deps.workspaceRoot);
    }
    deps.clearStyleSemanticGraphCache?.();
  }

  for (const uri of plan.affectedSourceUris) {
    const deps = args.registry.getDeps(uri);
    if (!deps) continue;
    deps.semanticReferenceIndex.forget(uri);
    deps.analysisCache.invalidate(uri);
    deps.clearStyleSemanticGraphCache?.();
  }

  const affectedStyleUris: string[] = [];
  const affectedSourceUris: string[] = [];
  for (const doc of snapshot.openDocuments) {
    const rootAffected =
      doc.workspaceRoot !== null && plan.affectedWorkspaceRoots.includes(doc.workspaceRoot);
    const sourceAffected = plan.affectedSourceUris.includes(doc.uri);
    if (!rootAffected && !sourceAffected) continue;
    if (doc.isStyle) {
      if (rootAffected) affectedStyleUris.push(doc.uri);
      continue;
    }
    if (sourceAffected) affectedSourceUris.push(doc.uri);
  }

  return {
    affectedStyleUris,
    affectedSourceUris,
  };
}
