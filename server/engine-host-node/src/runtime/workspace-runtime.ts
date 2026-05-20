import type { ResourceSettings } from "../../../engine-core-ts/src/settings";
import type { StyleDocumentHIR } from "../../../engine-core-ts/src/core/hir/style-types";
import type { FileTask } from "../../../engine-core-ts/src/core/indexing/indexer-worker";
import type { TypeResolver } from "../../../engine-core-ts/src/core/ts/type-resolver";
import type { WorkspaceFolderInfo, WorkspaceProviderDeps } from "../workspace/workspace-registry";
import type { SharedRuntimeCaches } from "./shared-runtime-caches";
import type { RuntimeSink } from "./runtime-sink";
import { createWorkspaceProviderDeps } from "./workspace-runtime-deps";
import {
  createWorkspaceRuntimeSettingsState,
  type WorkspaceRuntimeSettingsState,
} from "./workspace-runtime-settings";
import {
  clearWorkspaceDocumentsWithinRoot,
  createOwnedStylePathMatcher,
  type RuntimeDocumentsLike,
} from "./workspace-runtime-support";
import { createWorkspaceStyleRuntime } from "./workspace-style-runtime";

export interface WorkspaceRuntimeIO {
  readonly readStyleFile: (path: string) => string | null;
  readonly readOpenDocumentText?: (path: string) => string | null;
  readonly readStyleFileAsync?: (path: string) => Promise<string | null>;
  readonly fileSupplier?: () => AsyncIterable<FileTask>;
}

export interface WorkspaceRuntimeFactoryArgs {
  readonly folder: WorkspaceFolderInfo;
  readonly workspaceFolders: readonly WorkspaceFolderInfo[];
  readonly caches: SharedRuntimeCaches;
  readonly typeResolver: TypeResolver;
  readonly styleDocumentForPath: (path: string) => StyleDocumentHIR | null;
  readonly io: WorkspaceRuntimeIO;
  readonly sink: RuntimeSink;
  readonly fileExists: (path: string) => boolean;
  readonly serverName: string;
  readonly getModeForStylePath: (path: string) => ResourceSettings["scss"]["classnameTransform"];
}

export interface WorkspaceRuntime {
  readonly folder: WorkspaceFolderInfo;
  readonly deps: WorkspaceProviderDeps;
  dispose(): void;
  clearWorkspaceDocuments(documents: RuntimeDocumentsLike): void;
}

export function createWorkspaceRuntime(args: WorkspaceRuntimeFactoryArgs): WorkspaceRuntime {
  const settingsState = createWorkspaceRuntimeSettingsState(args.folder.rootPath);
  const styleRuntime = createWorkspaceStyleRuntime({
    workspaceRoot: args.folder.rootPath,
    caches: args.caches,
    io: args.io,
    sink: args.sink,
    serverName: args.serverName,
    fileExists: args.fileExists,
    aliasResolver: () => settingsState.aliasResolver,
    getModeForStylePath: args.getModeForStylePath,
    isOwnedStylePath: createOwnedStylePathMatcher(args.workspaceFolders, args.folder.uri),
  });
  const deps: WorkspaceProviderDeps = createWorkspaceProviderDeps({
    folder: args.folder,
    caches: args.caches,
    typeResolver: args.typeResolver,
    styleDocumentForPath: args.styleDocumentForPath,
    readStyleFile: args.io.readStyleFile,
    ...(args.io.readOpenDocumentText ? { readOpenDocumentText: args.io.readOpenDocumentText } : {}),
    fileExists: args.fileExists,
    sink: args.sink,
    serverName: args.serverName,
    settingsState,
    styleRuntime,
  });

  return {
    folder: args.folder,
    deps,
    dispose() {
      deps.stopIndexer();
      deps.typeResolver.invalidate(args.folder.rootPath);
    },
    clearWorkspaceDocuments(documents) {
      clearWorkspaceDocumentsWithinRoot(args.folder.rootPath, documents, deps, args.sink);
    },
  };
}

export type { WorkspaceRuntimeSettingsState };
