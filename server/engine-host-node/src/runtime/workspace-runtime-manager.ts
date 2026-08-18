import type { StyleDocumentHIR } from "../../../engine-core-ts/src/core/hir/style-types";
import type { TypeResolver } from "../../../engine-core-ts/src/core/ts/type-resolver";
import type { ResourceSettings } from "../../../engine-core-ts/src/settings";
import {
  type WorkspaceFolderInfo,
  WorkspaceRegistry,
  type WorkspaceProviderDeps,
} from "../workspace/workspace-registry";
import { toWorkspaceFolderInfo } from "../workspace/workspace-folder-info";
import type { SharedRuntimeCaches } from "./shared-runtime-caches";
import type { SourceFrontendAnalysisOutcomeCounterSnapshotV0 } from "../source-frontend-analysis-provider";
import type { RuntimeSink } from "./runtime-sink";
import type { WorkspaceRuntime, WorkspaceRuntimeIO } from "./workspace-runtime";
import type { RuntimeDocumentsLike } from "./workspace-runtime-support";
import { registerWorkspaceRuntime, unregisterWorkspaceRuntime } from "./workspace-runtime-registry";

export interface WorkspaceRuntimeManagerArgs {
  readonly caches: SharedRuntimeCaches;
  readonly typeResolver: TypeResolver;
  readonly styleDocumentForPath: (path: string) => StyleDocumentHIR | null;
  readonly io: WorkspaceRuntimeIO;
  readonly sink: RuntimeSink;
  readonly fileExists: (path: string) => boolean;
  readonly serverName: string;
  readonly getModeForStylePath: (path: string) => ResourceSettings["scss"]["classnameTransform"];
}

export interface WorkspaceRuntimeManager {
  getRegistry(): WorkspaceRegistry;
  getDeps(documentUri: string): WorkspaceProviderDeps | null;
  getDepsForFilePath(filePath: string): WorkspaceProviderDeps | null;
  hasFolder(folderUri: string): boolean;
  getFolders(): readonly WorkspaceFolderInfo[];
  sourceFrontendAnalysisOutcomeSnapshot(): SourceFrontendAnalysisOutcomeCounterSnapshotV0;
  setSourcePathInventoryWatcherCoverage(available: boolean): void;
  registerInitialFolders(folders: readonly WorkspaceFolderInfo[]): void;
  applyWorkspaceFolderChange(
    event: {
      readonly removed: readonly { readonly uri: string; readonly name: string }[];
      readonly added: readonly { readonly uri: string; readonly name: string }[];
    },
    documents: RuntimeDocumentsLike,
  ): void;
  addFolder(folder: WorkspaceFolderInfo): void;
  removeFolder(folderUri: string, documents: RuntimeDocumentsLike): boolean;
  disposeAll(documents: RuntimeDocumentsLike): void;
}

export function createWorkspaceRuntimeManager(
  args: WorkspaceRuntimeManagerArgs,
): WorkspaceRuntimeManager {
  const registry = new WorkspaceRegistry();
  const runtimes = new Map<string, WorkspaceRuntime>();
  let sourcePathInventoryWatcherCoverage = false;

  return {
    getRegistry(): WorkspaceRegistry {
      return registry;
    },
    getDeps(documentUri: string): WorkspaceProviderDeps | null {
      return registry.getDeps(documentUri);
    },
    getDepsForFilePath(filePath: string): WorkspaceProviderDeps | null {
      return registry.getDepsForFilePath(filePath);
    },
    hasFolder(folderUri: string): boolean {
      return registry.getFolder(folderUri) !== null;
    },
    getFolders(): readonly WorkspaceFolderInfo[] {
      return registry.getFolders();
    },
    sourceFrontendAnalysisOutcomeSnapshot(): SourceFrontendAnalysisOutcomeCounterSnapshotV0 {
      return args.caches.sourceFrontendAnalysisOutcomes.snapshot();
    },
    setSourcePathInventoryWatcherCoverage(available): void {
      sourcePathInventoryWatcherCoverage = available;
      for (const runtime of runtimes.values()) {
        runtime.deps.setSourcePathInventoryWatcherCoverage(available);
      }
    },
    registerInitialFolders(folders: readonly WorkspaceFolderInfo[]): void {
      for (const folder of folders) {
        const runtime = registerWorkspaceRuntime({
          registry,
          runtimes,
          folder,
          workspaceFolders: folders,
          caches: args.caches,
          typeResolver: args.typeResolver,
          styleDocumentForPath: args.styleDocumentForPath,
          io: args.io,
          sink: args.sink,
          fileExists: args.fileExists,
          serverName: args.serverName,
          getModeForStylePath: args.getModeForStylePath,
        });
        runtime.deps.setSourcePathInventoryWatcherCoverage(sourcePathInventoryWatcherCoverage);
      }
    },
    applyWorkspaceFolderChange(event, documents): void {
      for (const folder of event.removed) {
        this.removeFolder(folder.uri, documents);
      }
      for (const folder of event.added) {
        if (this.hasFolder(folder.uri)) continue;
        this.addFolder(toWorkspaceFolderInfo(folder));
      }
    },
    addFolder(folder: WorkspaceFolderInfo): void {
      if (registry.getFolder(folder.uri)) return;
      const runtime = registerWorkspaceRuntime({
        registry,
        runtimes,
        folder,
        workspaceFolders: [...registry.getFolders(), folder],
        caches: args.caches,
        typeResolver: args.typeResolver,
        styleDocumentForPath: args.styleDocumentForPath,
        io: args.io,
        sink: args.sink,
        fileExists: args.fileExists,
        serverName: args.serverName,
        getModeForStylePath: args.getModeForStylePath,
      });
      runtime.deps.setSourcePathInventoryWatcherCoverage(sourcePathInventoryWatcherCoverage);
    },
    removeFolder(folderUri: string, documents: RuntimeDocumentsLike): boolean {
      return unregisterWorkspaceRuntime({
        registry,
        runtimes,
        folderUri,
        documents,
      });
    },
    disposeAll(documents: RuntimeDocumentsLike): void {
      for (const folder of registry.getFolders()) {
        this.removeFolder(folder.uri, documents);
      }
      args.typeResolver.clear();
    },
  };
}
