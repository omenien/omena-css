import type { TextDocuments } from "vscode-languageserver/node";
import type { Connection, InitializeParams } from "vscode-languageserver/node";
import type { TextDocument } from "vscode-languageserver-textdocument";
import type { TypeResolver } from "../../engine-core-ts/src/core/ts/type-resolver";
import type { FileTask } from "../../engine-core-ts/src/core/indexing/indexer-worker";
import {
  resolveClientRuntimeCapabilities,
  type ClientRuntimeCapabilities,
} from "./server-capabilities";
import { createRuntimeSink, readStyleTextFromOpenDocuments } from "./server-runtime-support";
import { resolveWorkspaceFolders } from "../../engine-host-node/src/workspace/workspace-folder-info";
import type { WorkspaceRegistry } from "../../engine-host-node/src/workspace/workspace-registry";
import {
  createServerRuntimeManager,
  type WorkspaceRuntimeManager,
} from "../../engine-host-node/src/runtime";

export interface ServerRuntimeSessionOptions {
  readonly typeResolver?: TypeResolver;
  readonly fileSupplier?: () => AsyncIterable<FileTask>;
  readonly sourceFileSupplier?: () => AsyncIterable<FileTask>;
  readonly readStyleFileAsync?: (path: string) => Promise<string | null>;
  readonly fileExists?: (path: string) => boolean;
}

export interface ServerRuntimeSession {
  readonly registry: WorkspaceRegistry;
  readonly runtimeManager: WorkspaceRuntimeManager;
  readonly clientCapabilities: ClientRuntimeCapabilities;
  setSourcePathInventoryWatcherCoverage(available: boolean): void;
  handleWorkspaceFolderChange(
    event: {
      readonly removed: readonly { readonly uri: string; readonly name: string }[];
      readonly added: readonly { readonly uri: string; readonly name: string }[];
    },
    documents: TextDocuments<TextDocument>,
  ): void;
  dispose(documents: TextDocuments<TextDocument>): void;
}

export interface CreateServerRuntimeSessionArgs {
  readonly params: InitializeParams;
  readonly options: ServerRuntimeSessionOptions;
  readonly connection: Connection;
  readonly documents: TextDocuments<TextDocument>;
  readonly readStyleFile: (path: string) => string | null;
  readonly serverName: string;
}

export function createServerRuntimeSession(
  args: CreateServerRuntimeSessionArgs,
): ServerRuntimeSession {
  const workspaceFolders = resolveWorkspaceFolders({
    ...(args.params.workspaceFolders ? { workspaceFolders: args.params.workspaceFolders } : {}),
    ...(args.params.rootUri ? { rootUri: args.params.rootUri } : {}),
    ...(args.params.rootPath ? { rootPath: args.params.rootPath } : {}),
  });
  const clientCapabilities = resolveClientRuntimeCapabilities(args.params);
  const sink = createRuntimeSink(args.connection, clientCapabilities.codeLensRefresh);
  const bundle = createServerRuntimeManager({
    options: args.options,
    readStyleFile: args.readStyleFile,
    readOpenDocumentText: (stylePath) => readStyleTextFromOpenDocuments(stylePath, args.documents),
    sink,
    serverName: args.serverName,
  });

  const runtimeManager = bundle.runtimeManager;
  runtimeManager.registerInitialFolders(workspaceFolders);

  return {
    registry: bundle.registry,
    runtimeManager,
    clientCapabilities,
    setSourcePathInventoryWatcherCoverage(available): void {
      runtimeManager.setSourcePathInventoryWatcherCoverage(available);
    },
    handleWorkspaceFolderChange(event, documents): void {
      runtimeManager.applyWorkspaceFolderChange(event, documents);
    },
    dispose(documents): void {
      runtimeManager.disposeAll(documents);
    },
  };
}
