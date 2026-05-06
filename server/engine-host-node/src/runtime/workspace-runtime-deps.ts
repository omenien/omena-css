import type { StyleDocumentHIR } from "../../../engine-core-ts/src/core/hir/style-types";
import type { TypeResolver } from "../../../engine-core-ts/src/core/ts/type-resolver";
import type { WorkspaceFolderInfo, WorkspaceProviderDeps } from "../workspace/workspace-registry";
import type { SharedRuntimeCaches } from "./shared-runtime-caches";
import type { RuntimeSink } from "./runtime-sink";
import { createWorkspaceAnalysisCache } from "./workspace-analysis-runtime";
import type { WorkspaceRuntimeSettingsState } from "./workspace-runtime-settings";
import type { WorkspaceStyleRuntime } from "./workspace-style-runtime";
import { getEngineShadowRunnerDaemonJsonRunner } from "../selected-query-backend";

export interface WorkspaceRuntimeDepsArgs {
  readonly folder: WorkspaceFolderInfo;
  readonly caches: SharedRuntimeCaches;
  readonly typeResolver: TypeResolver;
  readonly styleDocumentForPath: (path: string) => StyleDocumentHIR | null;
  readonly readStyleFile: (path: string) => string | null;
  readonly fileExists: (path: string) => boolean;
  readonly sink: RuntimeSink;
  readonly serverName: string;
  readonly settingsState: WorkspaceRuntimeSettingsState;
  readonly styleRuntime: WorkspaceStyleRuntime;
}

export function createWorkspaceProviderDeps(args: WorkspaceRuntimeDepsArgs): WorkspaceProviderDeps {
  const analysisCache = createWorkspaceAnalysisCache({
    caches: args.caches,
    typeResolver: args.typeResolver,
    workspaceRoot: args.folder.rootPath,
    styleDocumentForPath: args.styleDocumentForPath,
    fileExists: args.fileExists,
    aliasResolver: () => args.settingsState.aliasResolver,
    settingsKey: () => args.settingsState.settingsKey,
    onReferencesChanged: () => args.sink.requestCodeLensRefresh(),
  });
  const runRustSelectedQueryBackendJsonAsync = getEngineShadowRunnerDaemonJsonRunner();

  return {
    analysisCache,
    get aliasResolver() {
      return args.settingsState.aliasResolver;
    },
    styleDocumentForPath: args.styleDocumentForPath,
    typeResolver: args.typeResolver,
    semanticReferenceIndex: args.caches.semanticReferenceIndex,
    styleDependencyGraph: args.caches.styleDependencyGraph,
    styleSemanticGraphCache: args.caches.styleSemanticGraphCache,
    styleSemanticGraphBatchOutputCache: args.caches.styleSemanticGraphBatchOutputCache,
    selectorUsagePayloadCache: args.caches.selectorUsagePayloadCache,
    ...(runRustSelectedQueryBackendJsonAsync ? { runRustSelectedQueryBackendJsonAsync } : {}),
    clearStyleSemanticGraphCache: () => {
      args.caches.styleSemanticGraphCache.clear();
      args.caches.styleSemanticGraphBatchOutputCache.clear();
      args.caches.selectorUsagePayloadCache.clear();
    },
    workspaceRoot: args.folder.rootPath,
    workspaceFolderUri: args.folder.uri,
    logError: (message, err) => {
      const detail = err instanceof Error ? `${err.message}\n${err.stack ?? ""}` : String(err);
      args.sink.error(`[${args.serverName}] ${message}: ${detail}`);
    },
    invalidateStyle: (stylePath) => args.styleRuntime.invalidateStyle(stylePath),
    peekStyleDocument: (stylePath) =>
      args.styleRuntime.peekStyleDocument(stylePath, args.settingsState.classnameTransform),
    buildStyleDocument: (stylePath, content) =>
      args.styleRuntime.buildStyleDocument(
        stylePath,
        content,
        args.settingsState.classnameTransform,
      ),
    readStyleFile: args.readStyleFile,
    fileExists: args.fileExists,
    pushStyleFile: (stylePath) => args.styleRuntime.pushStyleFile(stylePath),
    indexerReady: args.styleRuntime.indexerReady,
    stopIndexer: () => args.styleRuntime.stop(),
    get settings() {
      return args.settingsState.get();
    },
    set settings(next) {
      args.settingsState.set(next);
    },
    rebuildAliasResolver: (pathAlias) => args.settingsState.rebuildAliasResolver(pathAlias),
    refreshCodeLens: () => args.sink.requestCodeLensRefresh(),
  };
}
