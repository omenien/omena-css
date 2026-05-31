import path from "node:path";
import { PassThrough } from "node:stream";
import { pathToFileURL } from "node:url";
import {
  CodeActionRequest,
  CodeLensRequest,
  CodeLensRefreshRequest,
  CompletionRequest,
  createProtocolConnection,
  DefinitionRequest,
  DidChangeConfigurationNotification,
  DidChangeTextDocumentNotification,
  DidChangeWatchedFilesNotification,
  DidChangeWorkspaceFoldersNotification,
  DidCloseTextDocumentNotification,
  DidOpenTextDocumentNotification,
  ExitNotification,
  HoverRequest,
  InitializedNotification,
  InitializeRequest,
  PrepareRenameRequest,
  PublishDiagnosticsNotification,
  ReferencesRequest,
  RenameRequest,
  ShutdownRequest,
  type CodeAction,
  type CodeActionParams,
  type CodeLens,
  type CodeLensParams,
  type Command,
  type CompletionItem,
  type CompletionList,
  type CompletionParams,
  type DefinitionParams,
  type Diagnostic,
  type DidChangeTextDocumentParams,
  type DidChangeWatchedFilesParams,
  type DidChangeWorkspaceFoldersParams,
  type DidCloseTextDocumentParams,
  type DidOpenTextDocumentParams,
  type Hover,
  type HoverParams,
  type InitializeParams,
  type InitializeResult,
  type Location,
  type LocationLink,
  type PrepareRenameParams,
  type ProtocolConnection,
  type PublishDiagnosticsParams,
  type Range as LspRange,
  type ReferenceParams,
  type RenameParams,
  type WorkspaceEdit,
} from "vscode-languageserver-protocol/node";
import { StreamMessageReader, StreamMessageWriter } from "vscode-jsonrpc/node";
import {
  createServer,
  type CreateServerOptions,
} from "../../../server/lsp-server/src/composition-root";
import type { FileTask } from "../../../server/engine-core-ts/src/core/indexing/indexer-worker";

const LEGACY_WORKSPACE_URI = "file:///fake/workspace";

function mapWorkspaceUriString(value: string, from: string, to: string): string {
  return value === from || value.startsWith(`${from}/`)
    ? `${to}${value.slice(from.length)}`
    : value;
}

function remapWorkspaceUris<T>(value: T, from: string, to: string): T {
  if (typeof value === "string") {
    return mapWorkspaceUriString(value, from, to) as T;
  }
  if (Array.isArray(value)) {
    return value.map((entry) => remapWorkspaceUris(entry, from, to)) as T;
  }
  if (value && typeof value === "object") {
    const remapped = Object.entries(value).map(([key, entry]) => [
      mapWorkspaceUriString(key, from, to),
      remapWorkspaceUris(entry, from, to),
    ]);
    return Object.fromEntries(remapped) as T;
  }
  return value;
}

/**
 * An exhausted AsyncIterable — yields nothing and completes
 * immediately. Built without an `async function*` so it does
 * not trip the `require-await` rule.
 */
export function emptySupplier(): AsyncIterable<FileTask> {
  return {
    [Symbol.asyncIterator](): AsyncIterator<FileTask> {
      return {
        next: () => Promise.resolve({ done: true, value: undefined as never }),
      };
    },
  };
}

export interface InProcessServerOptions extends Omit<
  Extract<CreateServerOptions, { transport?: "auto" }>,
  "transport"
> {
  readonly workspacePath?: string;
}

export interface LspTestClient {
  initialize(overrides?: Partial<InitializeParams>): Promise<InitializeResult>;
  initialized(): void;
  didOpen(params: DidOpenTextDocumentParams): void;
  didClose(params: DidCloseTextDocumentParams): void;
  didChange(params: DidChangeTextDocumentParams): void;
  definition(params: DefinitionParams): Promise<LocationLink[] | Location[] | null>;
  hover(params: HoverParams): Promise<Hover | null>;
  completion(params: CompletionParams): Promise<CompletionItem[] | CompletionList | null>;
  codeAction(params: CodeActionParams): Promise<(Command | CodeAction)[] | null>;
  codeLens(params: CodeLensParams): Promise<CodeLens[] | null>;
  /**
   * Wait for the next publishDiagnostics notification matching
   * `uri`, or reject after `timeoutMs`. Use this to test the
   * debounced push-based diagnostics pipeline .
   */
  waitForDiagnostics(uri: string, timeoutMs?: number): Promise<Diagnostic[]>;
  waitForCodeLensRefresh(timeoutMs?: number): Promise<void>;
  prepareRename(
    params: PrepareRenameParams,
  ): Promise<{ range: LspRange; placeholder: string } | null>;
  references(params: ReferenceParams): Promise<Location[] | null>;
  rename(params: RenameParams): Promise<WorkspaceEdit | null>;
  didChangeWatchedFiles(params: DidChangeWatchedFilesParams): void;
  didChangeWorkspaceFolders(params: DidChangeWorkspaceFoldersParams): void;
  /**
   * Replace the workspace configuration the server will see on the
   * next `workspace/configuration` request. The server triggers
   * such a request via `fetchSettings` after a
   * `workspace/didChangeConfiguration` notification.
   */
  setConfiguration(section: string, value: unknown): void;
  setScopedConfiguration(section: string, scopeUri: string, value: unknown): void;
  didChangeConfiguration(): void;
  shutdown(): Promise<void>;
  exit(): void;
  dispose(): void;
}

/**
 * Build an in-process LSP server wired to an in-process client.
 *
 * Two PassThrough streams form a full-duplex pair:
 *   serverOut ──► clientIn  (server → client)
 *   clientOut ──► serverIn  (client → server)
 *
 * The server is started immediately. The returned client exposes
 * typed request helpers for the LSP methods the protocol tests
 * exercise.
 *
 * `dispose()` ends both streams and disposes both connections.
 * Tests MUST call it in afterEach to avoid resource leaks.
 */
export function createInProcessServer(options: InProcessServerOptions = {}): LspTestClient {
  const { workspacePath: customWorkspacePath, ...serverOptions } = options;
  const workspacePath = customWorkspacePath ?? path.resolve(process.cwd(), ".lsp-test-workspace");
  const workspaceUri = pathToFileURL(workspacePath).toString();
  const serverToClient = new PassThrough();
  const clientToServer = new PassThrough();

  // Pre-wrap streams in typed reader/writer BEFORE handing them to
  // createServer. vscode-languageserver/node otherwise detects a raw
  // readable stream and attaches its own `end`/`close` handlers that
  // call `process.exit()` — fine in production (the server should
  // die when the client disconnects) but catastrophic for in-process
  // tests: exiting kills the vitest worker. Wrapping hides the raw
  // `.read` method and skips the auto-exit block.
  //
  const { connection: serverConnection } = createServer({
    // Default indexer supplier → empty. Tests exercise providers
    // via didOpen + in-memory readStyleFile; the background walk
    // is not wanted because it would hit the real filesystem.
    fileSupplier: () => emptySupplier(),
    readStyleFileAsync: async () => null,
    // Default fileExists to always-true for protocol tests: their
    // fixtures use fake absolute paths that would fail a real
    // fs.existsSync check, producing spurious missing-module
    // diagnostics. Tests that exercise missing-module specifically
    // override via the `fileExists` option.
    fileExists: () => true,
    ...serverOptions,
    transport: "streams",
    reader: new StreamMessageReader(clientToServer),
    writer: new StreamMessageWriter(serverToClient),
  });
  serverConnection.listen();

  const client: ProtocolConnection = createProtocolConnection(
    new StreamMessageReader(serverToClient),
    new StreamMessageWriter(clientToServer),
  );
  const pendingDiagnostics = new Map<string, PublishDiagnosticsParams[]>();
  const diagnosticsWaiters = new Map<
    string,
    Array<{ resolve: (d: Diagnostic[]) => void; reject: (e: unknown) => void }>
  >();
  let pendingCodeLensRefreshes = 0;
  const codeLensRefreshWaiters: Array<{ resolve: () => void; reject: (e: unknown) => void }> = [];
  // Handle workspace/configuration requests from the server. The
  // server's `fetchSettings` asks for two sections
  // (`omena` + `cssModules`) in separate requests, so
  // we return a per-section object that tests can swap out between
  // didChangeConfiguration reloads.
  const configBySection: Record<string, unknown> = {};
  const scopedConfigBySection = new Map<string, unknown>();
  client.onRequest(
    "workspace/configuration",
    (params: { items: Array<{ section?: string; scopeUri?: string }> }) => {
      return params.items.map((item) => {
        if (item.section && item.scopeUri) {
          const scopedKey = `${item.section}\u0000${item.scopeUri}`;
          if (scopedConfigBySection.has(scopedKey)) {
            return scopedConfigBySection.get(scopedKey);
          }
        }
        return item.section && item.section in configBySection ? configBySection[item.section] : {};
      });
    },
  );

  client.onNotification(PublishDiagnosticsNotification.type, (params) => {
    const mapped = remapWorkspaceUris(params, workspaceUri, LEGACY_WORKSPACE_URI);
    const waiters = diagnosticsWaiters.get(mapped.uri);
    if (waiters && waiters.length > 0) {
      const waiter = waiters.shift()!;
      waiter.resolve([...mapped.diagnostics]);
      return;
    }
    const queue = pendingDiagnostics.get(mapped.uri) ?? [];
    queue.push(mapped);
    pendingDiagnostics.set(mapped.uri, queue);
  });
  client.onRequest(CodeLensRefreshRequest.type, async () => {
    if (codeLensRefreshWaiters.length > 0) {
      const waiter = codeLensRefreshWaiters.shift()!;
      waiter.resolve();
    } else {
      pendingCodeLensRefreshes += 1;
    }
    return undefined;
  });
  client.listen();

  return {
    async initialize(overrides) {
      const base: InitializeParams = {
        processId: process.pid,
        rootUri: workspaceUri,
        capabilities: {
          workspace: {
            codeLens: { refreshSupport: true },
            workspaceFolders: true,
          },
        },
        workspaceFolders: [{ uri: workspaceUri, name: "fake" }],
      };
      return client.sendRequest(InitializeRequest.type, {
        ...base,
        ...remapWorkspaceUris(overrides ?? {}, LEGACY_WORKSPACE_URI, workspaceUri),
      });
    },
    initialized() {
      client.sendNotification(InitializedNotification.type, {});
    },
    didOpen(params) {
      client.sendNotification(
        DidOpenTextDocumentNotification.type,
        remapWorkspaceUris(params, LEGACY_WORKSPACE_URI, workspaceUri),
      );
    },
    didClose(params) {
      client.sendNotification(
        DidCloseTextDocumentNotification.type,
        remapWorkspaceUris(params, LEGACY_WORKSPACE_URI, workspaceUri),
      );
    },
    didChange(params) {
      client.sendNotification(
        DidChangeTextDocumentNotification.type,
        remapWorkspaceUris(params, LEGACY_WORKSPACE_URI, workspaceUri),
      );
    },
    async definition(params) {
      const result = await client.sendRequest(
        DefinitionRequest.type,
        remapWorkspaceUris(params, LEGACY_WORKSPACE_URI, workspaceUri),
      );
      return remapWorkspaceUris(result, workspaceUri, LEGACY_WORKSPACE_URI);
    },
    async hover(params) {
      const result = await client.sendRequest(
        HoverRequest.type,
        remapWorkspaceUris(params, LEGACY_WORKSPACE_URI, workspaceUri),
      );
      return remapWorkspaceUris(result, workspaceUri, LEGACY_WORKSPACE_URI);
    },
    async completion(params) {
      const result = await client.sendRequest(
        CompletionRequest.type,
        remapWorkspaceUris(params, LEGACY_WORKSPACE_URI, workspaceUri),
      );
      return remapWorkspaceUris(result, workspaceUri, LEGACY_WORKSPACE_URI);
    },
    async codeAction(params) {
      const result = await client.sendRequest(
        CodeActionRequest.type,
        remapWorkspaceUris(params, LEGACY_WORKSPACE_URI, workspaceUri),
      );
      return remapWorkspaceUris(result, workspaceUri, LEGACY_WORKSPACE_URI);
    },
    async codeLens(params) {
      const result = await client.sendRequest(
        CodeLensRequest.type,
        remapWorkspaceUris(params, LEGACY_WORKSPACE_URI, workspaceUri),
      );
      return remapWorkspaceUris(result, workspaceUri, LEGACY_WORKSPACE_URI);
    },
    async waitForCodeLensRefresh(timeoutMs = 5000) {
      if (pendingCodeLensRefreshes > 0) {
        pendingCodeLensRefreshes -= 1;
        return;
      }
      return new Promise<void>((resolve, reject) => {
        const timer = setTimeout(() => {
          const index = codeLensRefreshWaiters.findIndex((waiter) => waiter.resolve === resolve);
          if (index >= 0) codeLensRefreshWaiters.splice(index, 1);
          reject(new Error(`waitForCodeLensRefresh timed out after ${timeoutMs}ms`));
        }, timeoutMs);
        codeLensRefreshWaiters.push({
          resolve: () => {
            clearTimeout(timer);
            resolve();
          },
          reject: (e) => {
            clearTimeout(timer);
            reject(e);
          },
        });
      });
    },
    async prepareRename(params) {
      const result = await client.sendRequest(
        PrepareRenameRequest.type,
        remapWorkspaceUris(params, LEGACY_WORKSPACE_URI, workspaceUri),
      );
      return remapWorkspaceUris(result, workspaceUri, LEGACY_WORKSPACE_URI);
    },
    async references(params) {
      const result = await client.sendRequest(
        ReferencesRequest.type,
        remapWorkspaceUris(params, LEGACY_WORKSPACE_URI, workspaceUri),
      );
      return remapWorkspaceUris(result, workspaceUri, LEGACY_WORKSPACE_URI);
    },
    async rename(params) {
      const result = await client.sendRequest(
        RenameRequest.type,
        remapWorkspaceUris(params, LEGACY_WORKSPACE_URI, workspaceUri),
      );
      return remapWorkspaceUris(result, workspaceUri, LEGACY_WORKSPACE_URI);
    },
    didChangeWatchedFiles(params) {
      client.sendNotification(
        DidChangeWatchedFilesNotification.type,
        remapWorkspaceUris(params, LEGACY_WORKSPACE_URI, workspaceUri),
      );
    },
    didChangeWorkspaceFolders(params) {
      client.sendNotification(
        DidChangeWorkspaceFoldersNotification.type,
        remapWorkspaceUris(params, LEGACY_WORKSPACE_URI, workspaceUri),
      );
    },
    setConfiguration(section, value) {
      configBySection[section] = value;
    },
    setScopedConfiguration(section, scopeUri, value) {
      scopedConfigBySection.set(`${section}\u0000${scopeUri}`, value);
    },
    didChangeConfiguration() {
      client.sendNotification(DidChangeConfigurationNotification.type, { settings: null });
    },
    async waitForDiagnostics(uri, timeoutMs = 5000) {
      const queue = pendingDiagnostics.get(uri);
      if (queue && queue.length > 0) {
        return [...queue.shift()!.diagnostics];
      }
      return new Promise<Diagnostic[]>((resolve, reject) => {
        const list = diagnosticsWaiters.get(uri) ?? [];
        const timer = setTimeout(() => {
          const filtered = list.filter((w) => w.resolve !== resolve);
          diagnosticsWaiters.set(uri, filtered);
          reject(new Error(`waitForDiagnostics(${uri}) timed out after ${timeoutMs}ms`));
        }, timeoutMs);
        list.push({
          resolve: (d) => {
            clearTimeout(timer);
            resolve(d);
          },
          reject: (e) => {
            clearTimeout(timer);
            reject(e);
          },
        });
        diagnosticsWaiters.set(uri, list);
      });
    },
    async shutdown() {
      await client.sendRequest(ShutdownRequest.type, undefined);
    },
    exit() {
      client.sendNotification(ExitNotification.type);
    },
    dispose() {
      // TODO: revisit if Tier 3 E2E surfaces stream
      // leaks. For now we deliberately do NOT destroy the
      // PassThrough pair to avoid ERR_STREAM_WRITE_AFTER_END on
      // racing shutdown acks. Each test owns a fresh pair, so GC
      // reclaims them at vitest worker shutdown.
      client.dispose();
      serverConnection.dispose();
    },
  };
}
