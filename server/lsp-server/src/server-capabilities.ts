import {
  DidChangeWatchedFilesNotification,
  type Connection,
  type InitializeParams,
  type InitializeResult,
} from "vscode-languageserver/node";
import { buildStyleFileWatcherGlob } from "../../engine-core-ts/src/core/scss/lang-registry";
import { COMPLETION_TRIGGER_CHARACTERS } from "./providers/completion";

export interface ClientRuntimeCapabilities {
  readonly dynamicWatchers: boolean;
  readonly codeLensRefresh: boolean;
  readonly workspaceFolders: boolean;
}

export function resolveClientRuntimeCapabilities(
  params: InitializeParams,
): ClientRuntimeCapabilities {
  return {
    dynamicWatchers:
      params.capabilities.workspace?.didChangeWatchedFiles?.dynamicRegistration ?? false,
    codeLensRefresh: params.capabilities.workspace?.codeLens?.refreshSupport ?? false,
    workspaceFolders: params.capabilities.workspace?.workspaceFolders ?? false,
  };
}

export function buildServerCapabilities(): InitializeResult["capabilities"] {
  return {
    textDocumentSync: 2,
    definitionProvider: true,
    hoverProvider: true,
    completionProvider: {
      triggerCharacters: [...COMPLETION_TRIGGER_CHARACTERS],
      resolveProvider: false,
    },
    codeActionProvider: {
      codeActionKinds: ["quickfix", "refactor.extract"],
      resolveProvider: false,
    },
    referencesProvider: true,
    codeLensProvider: { resolveProvider: false },
    renameProvider: { prepareProvider: true },
    workspace: {
      workspaceFolders: {
        supported: true,
        changeNotifications: true,
      },
    },
  };
}

export function registerDynamicFileWatchers(
  connection: Connection,
  dynamicWatchers: boolean,
): Promise<{ dispose(): void }> | null {
  if (!dynamicWatchers) return null;
  return connection.client
    .register(DidChangeWatchedFilesNotification.type, {
      watchers: [
        { globPattern: buildStyleFileWatcherGlob() },
        { globPattern: "**/*.{ts,tsx,js,jsx,mts,cts,mjs,cjs,d.ts}" },
        { globPattern: "**/tsconfig*.json" },
        { globPattern: "**/jsconfig*.json" },
      ],
    })
    .catch(() => ({ dispose: () => {} }));
}
