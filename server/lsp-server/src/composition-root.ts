import type { MessageReader, MessageWriter } from "vscode-languageserver/node";
import {
  createConnection,
  ProposedFeatures,
  TextDocuments,
  type Connection,
  type InitializeResult,
} from "vscode-languageserver/node";
import { TextDocument } from "vscode-languageserver-textdocument";
import type { TypeResolver } from "../../engine-core-ts/src/core/ts/type-resolver";
import type { FileTask } from "../../engine-core-ts/src/core/indexing/indexer-worker";
import { registerHandlers } from "./handler-registration";
import { buildServerCapabilities, registerDynamicFileWatchers } from "./server-capabilities";
import { createServerRuntimeSession, type ServerRuntimeSession } from "./server-runtime-session";
import type { WorkspaceRegistry } from "../../engine-host-node/src/workspace/workspace-registry";
import { defaultReadStyleFile } from "../../engine-host-node/src/runtime";
import { shutdownEngineShadowRunnerDaemon } from "../../engine-host-node/src/selected-query-backend";

const SERVER_NAME = "css-module-explainer";
const SERVER_VERSION = "4.19.0";
const RUNTIME_LOOP_PROBE_REQUEST = "cssModuleExplainer/runtimeLoopProbe";

/**
 * Transport-agnostic shared options consumed by every
 * `createServer` variant. Lives on the "auto" branch of the
 * discriminated union as the implied default — the "streams"
 * branch extends it and adds the required reader/writer pair.
 *
 * Split on `transport` so there is no stringly-typed cast at the
 * `createConnection` call site.
 */
export interface CreateServerAutoOptions {
  /**
   * `"auto"` (default): `createConnection(ProposedFeatures.all)`
   * auto-detects the transport from process.argv flags set by
   * the LanguageClient — `--node-ipc` → IPC, `--stdio` →
   * stdin/stdout.
   */
  readonly transport?: "auto";
  readonly typeResolver?: TypeResolver;
  readonly readStyleFile?: (path: string) => string | null;
  readonly fileSupplier?: () => AsyncIterable<FileTask>;
  readonly readStyleFileAsync?: (path: string) => Promise<string | null>;
  /**
   * Filesystem existence check used by the analysis cache to tag
   * style imports as resolved or missing. Defaults to
   * `fs.existsSync`. Test harnesses inject a stub.
   */
  readonly fileExists?: (path: string) => boolean;
}

/**
 * Streams transport: the caller supplies a preconstructed
 * reader/writer pair (Tier 2 test harness with PassThrough
 * streams). The discriminant narrows both fields to
 * `MessageReader` / `MessageWriter` inside the branch — no cast.
 */
export interface CreateServerStreamsOptions extends Omit<CreateServerAutoOptions, "transport"> {
  readonly transport: "streams";
  readonly reader: MessageReader;
  readonly writer: MessageWriter;
}

export type CreateServerOptions = CreateServerAutoOptions | CreateServerStreamsOptions;

export interface CreatedServer {
  readonly connection: Connection;
  readonly documents: TextDocuments<TextDocument>;
}

/**
 * Build an LSP server instance from a pair of streams plus
 * optional dependency overrides.
 *
 * Does NOT call `connection.listen()` — the caller decides when
 * the event loop starts.
 *
 * Responsibilities are split:
 *   - THIS file: DI assembly (buildBundle) + lifecycle (init/initialized)
 *   - lsp-server/handler-registration.ts: LSP request routing + diagnostics scheduler
 */
export function createServer(options: CreateServerOptions): CreatedServer {
  const connection =
    options.transport === "streams"
      ? createConnection(ProposedFeatures.all, options.reader, options.writer)
      : createConnection(ProposedFeatures.all);
  const documents = new TextDocuments<TextDocument>(TextDocument);
  const readStyleFile = options.readStyleFile ?? defaultReadStyleFile;

  let registry: WorkspaceRegistry | null = null;
  let session: ServerRuntimeSession | null = null;
  let watchedFilesDisposable: Promise<{ dispose(): void }> | null = null;

  // ── Lifecycle ──────────────────────────────────────────────

  connection.onInitialize((params): InitializeResult => {
    connection.console.info(`[${SERVER_NAME}] initialize received`);
    session = createServerRuntimeSession({
      params,
      options,
      connection,
      documents,
      readStyleFile,
      serverName: SERVER_NAME,
    });
    registry = session.registry;
    return {
      capabilities: buildServerCapabilities(),
      serverInfo: { name: SERVER_NAME, version: SERVER_VERSION },
    };
  });

  // ── Request routing (delegated to handler-registration.ts) ─

  const handlers = registerHandlers({
    connection,
    documents,
    getDeps: (uri) => registry?.getDeps(uri) ?? null,
    getRegistry: () => registry,
  });

  if (process.env.CME_LSP_RUNTIME_LOOP_PROBE === "1") {
    connection.onRequest(RUNTIME_LOOP_PROBE_REQUEST, () => ({
      now: Date.now(),
    }));
  }

  connection.onInitialized(async () => {
    connection.console.info(`[${SERVER_NAME}] initialized`);
    if (!session) return;
    if (session.clientCapabilities.workspaceFolders) {
      connection.workspace.onDidChangeWorkspaceFolders((event) => {
        session?.handleWorkspaceFolderChange(event, documents);
        handlers.refreshSettings();
      });
    }
    watchedFilesDisposable = registerDynamicFileWatchers(
      connection,
      session.clientCapabilities.dynamicWatchers,
    );
    if (!watchedFilesDisposable) {
      watchedFilesDisposable = null;
    }
    handlers.refreshSettings();
  });

  connection.onShutdown(() => {
    handlers.shutdown();
    shutdownEngineShadowRunnerDaemon();
    session?.dispose(documents);
    void watchedFilesDisposable?.then((d) => d.dispose()).catch(() => {});
    watchedFilesDisposable = null;
    session = null;
    registry = null;
  });

  documents.listen(connection);
  return { connection, documents };
}
