import type { Connection, TextDocumentPositionParams } from "vscode-languageserver/node";
import { FileChangeType } from "vscode-languageserver/node";
import type { DidChangeWatchedFilesParams } from "vscode-languageserver/node";
import type { TextDocument } from "vscode-languageserver-textdocument";
import type { TextDocuments } from "vscode-languageserver/node";
import { handleCodeAction } from "./providers/code-actions";
import { handleCompletion } from "./providers/completion";
import { handleDefinition } from "./providers/definition";
import { handleHover } from "./providers/hover";
import { handleCodeLens } from "./providers/reference-lens";
import { handleReferences } from "./providers/references";
import { handlePrepareRename, handleRename } from "./providers/rename";
import type { CursorParams, ProviderDeps } from "../../engine-core-ts/src/provider-deps";
import { fileUrlToPath } from "../../engine-core-ts/src/core/util/text-utils";
import { findLangForPath } from "../../engine-core-ts/src/core/scss/lang-registry";
import {
  DEFAULT_WINDOW_SETTINGS,
  fetchResourceSettings,
  fetchWindowSettings,
  type WindowSettings,
} from "../../engine-core-ts/src/settings";
import { createDiagnosticsScheduler, type DiagnosticsScheduler } from "./diagnostics-scheduler";
import type { WorkspaceRegistry } from "../../engine-host-node/src/workspace/workspace-registry";
import {
  type RuntimeFileEvent,
  applyWatchedFileChanges,
  applySettingsReload,
} from "../../engine-host-node/src/runtime";

export interface HandlerContext {
  readonly connection: Connection;
  readonly documents: TextDocuments<TextDocument>;
  getDeps(uri: string): ProviderDeps | null;
  getRegistry(): WorkspaceRegistry | null;
}

export interface HandlerCleanup {
  shutdown(): void;
  refreshSettings(): void;
}

interface HandlerState {
  readonly ctx: HandlerContext;
  readonly scheduler: DiagnosticsScheduler;
  windowSettings: WindowSettings;
}

type StyleSemanticGraphCacheInvalidator = ProviderDeps & {
  clearStyleSemanticGraphCache?(): void;
};

/**
 * Wire every LSP request/notification handler onto the connection.
 *
 * Split into feature-group registrars so this file owns routing
 * only — the composition root stays a thin DI shell.
 */
export function registerHandlers(ctx: HandlerContext): HandlerCleanup {
  const state: HandlerState = {
    ctx,
    scheduler: createDiagnosticsScheduler(
      {
        connection: ctx.connection,
        documents: ctx.documents,
        getDeps: ctx.getDeps,
        getAllDeps: () => ctx.getRegistry()?.allDeps() ?? [],
      },
      DEFAULT_WINDOW_SETTINGS,
    ),
    windowSettings: DEFAULT_WINDOW_SETTINGS,
  };

  const reloadSettings = registerSettingsHandler(state);
  registerCursorHandlers(state);
  registerDocumentHandlers(state);
  registerWatchedFilesHandler(state);

  return {
    shutdown() {
      for (const deps of state.ctx.getRegistry()?.allDeps() ?? []) {
        deps.stopIndexer();
      }
      state.scheduler.shutdown();
    },
    refreshSettings: reloadSettings,
  };
}

function registerSettingsHandler(state: HandlerState): () => void {
  const { connection } = state.ctx;

  function reloadSettings(): void {
    fetchWindowSettings(connection)
      .then(async (windowSettings) => {
        state.windowSettings = windowSettings;
        state.scheduler.refreshSettings(windowSettings);

        const registry = state.ctx.getRegistry();
        if (!registry) return;

        const resourceSettingsByBundle = await Promise.all(
          registry.allDeps().map(async (deps) => ({
            workspaceFolderUri: deps.workspaceFolderUri,
            resourceSettings: await fetchResourceSettings(connection, deps.workspaceFolderUri),
          })),
        );
        const result = applySettingsReload({
          registry,
          documents: state.ctx.documents,
          windowSettings,
          resourceSettingsByWorkspaceFolder: resourceSettingsByBundle,
        });
        for (const item of result.scheduledDiagnostics) {
          if (item.kind === "style") {
            state.scheduler.scheduleScss(item.uri);
            continue;
          }
          state.scheduler.scheduleTsx(item.uri);
        }
      })
      .catch((err: unknown) => safeLogError(connection, "settings fetch failed", err));
  }

  connection.onDidChangeConfiguration(reloadSettings);
  return reloadSettings;
}

function registerCursorHandlers(state: HandlerState): void {
  const { connection, documents, getDeps } = state.ctx;

  const withCursor = <T>(
    featureOn: () => boolean,
    p: TextDocumentPositionParams,
    run: (cursor: CursorParams, deps: ProviderDeps) => MaybePromise<T | null>,
  ): MaybePromise<T | null> => {
    if (!featureOn()) return null;
    const deps = getDeps(p.textDocument.uri);
    if (!deps) return null;
    const cursor = toCursorParams(p, documents);
    if (!cursor) return null;
    return run(cursor, deps);
  };

  connection.onDefinition((p) =>
    withCursor(() => state.windowSettings.features.definition, p, handleDefinition),
  );

  connection.onHover((p) =>
    withCursor(
      () => state.windowSettings.features.hover,
      p,
      (cursor, deps) => handleHover(cursor, deps, state.windowSettings.hover.maxCandidates),
    ),
  );

  connection.onCompletion((p) =>
    withCursor(() => state.windowSettings.features.completion, p, handleCompletion),
  );

  connection.onCodeAction((p) => {
    const deps = getDeps(p.textDocument.uri);
    if (!deps) return null;
    return handleCodeAction(p, deps, documents.get(p.textDocument.uri)?.getText());
  });

  connection.onReferences((p) => {
    if (!state.windowSettings.features.references) return null;
    const deps = getDeps(p.textDocument.uri);
    if (!deps) return null;
    const cursor = toCursorParams(p, documents);
    return handleReferences(p, deps, cursor ?? undefined);
  });

  connection.onCodeLens((p) => {
    if (!state.windowSettings.features.references) return null;
    const deps = getDeps(p.textDocument.uri);
    if (!deps) return null;
    return handleCodeLens(p, deps);
  });

  // Capability parity with the Rust server (the primary endpoint), which
  // resolves variable references to color swatches. This legacy server
  // advertises the capability but serves no decorations.
  connection.onDocumentColor(() => []);
  connection.onColorPresentation(() => []);

  connection.onPrepareRename((p) => {
    if (!state.windowSettings.features.rename) return null;
    const deps = getDeps(p.textDocument.uri);
    if (!deps) return null;
    const cursor = toCursorParams(p, documents);
    return handlePrepareRename(p, deps, cursor ?? undefined);
  });

  connection.onRenameRequest((p) => {
    if (!state.windowSettings.features.rename) return null;
    const deps = getDeps(p.textDocument.uri);
    if (!deps) return null;
    const cursor = toCursorParams(p, documents);
    return handleRename(p, deps, cursor ?? undefined);
  });
}

type MaybePromise<T> = T | PromiseLike<T>;

function registerDocumentHandlers(state: HandlerState): void {
  const { documents } = state.ctx;

  documents.onDidChangeContent((change) => {
    state.scheduler.ensureReadySubscribed();
    clearRuntimeStyleSemanticGraphCache(state.ctx.getDeps(change.document.uri));
    const filePath = fileUrlToPath(change.document.uri);
    if (findLangForPath(filePath)) {
      state.scheduler.scheduleScss(change.document.uri);
    } else {
      state.scheduler.scheduleTsx(change.document.uri);
    }
  });

  documents.onDidClose((change) => {
    state.scheduler.handleDocumentClose(change.document.uri);
    const deps = state.ctx.getDeps(change.document.uri);
    if (!deps) return;
    // Drop every workspace-visible trace of the closed buffer
    // before the next analyze or unused-selector check runs.
    deps.semanticReferenceIndex.forget(change.document.uri);
    deps.analysisCache.invalidate(change.document.uri);
    clearRuntimeStyleSemanticGraphCache(deps);
    deps.refreshCodeLens();
  });
}

function registerWatchedFilesHandler(state: HandlerState): void {
  const { connection } = state.ctx;

  connection.onDidChangeWatchedFiles((params: DidChangeWatchedFilesParams) => {
    const registry = state.ctx.getRegistry();
    if (!registry) return;
    const result = applyWatchedFileChanges({
      registry,
      documents: state.ctx.documents,
      events: params.changes.map(toRuntimeFileEvent),
    });
    for (const uri of result.affectedStyleUris) {
      state.scheduler.scheduleScss(uri);
    }
    for (const uri of result.affectedSourceUris) {
      state.scheduler.scheduleTsx(uri);
    }
  });
}

function toRuntimeFileEvent(
  change: DidChangeWatchedFilesParams["changes"][number],
): RuntimeFileEvent {
  return {
    uri: change.uri,
    type:
      change.type === FileChangeType.Created
        ? "created"
        : change.type === FileChangeType.Deleted
          ? "deleted"
          : "changed",
  };
}

function safeLogError(connection: Connection, context: string, err: unknown): void {
  const message = err instanceof Error ? err.message : String(err);
  try {
    connection.console.error(`[omena-css] ${context}: ${message}`);
  } catch {
    // Connection already disposed — nothing to log to.
  }
}

function clearRuntimeStyleSemanticGraphCache(deps: ProviderDeps | null): void {
  (deps as StyleSemanticGraphCacheInvalidator | null)?.clearStyleSemanticGraphCache?.();
}

function toCursorParams(
  p: TextDocumentPositionParams,
  documents: TextDocuments<TextDocument>,
): CursorParams | null {
  const doc = documents.get(p.textDocument.uri);
  if (!doc) return null;
  return {
    documentUri: p.textDocument.uri,
    content: doc.getText(),
    filePath: fileUrlToPath(p.textDocument.uri),
    line: p.position.line,
    character: p.position.character,
    version: doc.version,
  };
}
