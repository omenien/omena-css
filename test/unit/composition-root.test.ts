import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { PassThrough } from "node:stream";
import { afterEach, describe, expect, it, vi } from "vitest";
import { StreamMessageReader, StreamMessageWriter } from "vscode-jsonrpc/node";
import { createServer } from "../../server/lsp-server/src/composition-root";
import type { FileTask } from "../../server/engine-core-ts/src/core/indexing/indexer-worker";
import { createInProcessServer, type LspTestClient } from "../protocol/_harness/in-process-server";
import { FakeTypeResolver } from "../_fixtures/fake-type-resolver";

const { selectedQueryRunner } = vi.hoisted(() => ({
  selectedQueryRunner: vi.fn(async (_command: string, _input: unknown) => ({
    product: "omena-query.diagnostics-for-file",
    fileKind: "style",
    diagnostics: [],
  })),
}));

vi.mock("../../server/engine-host-node/src/selected-query-backend", async (importOriginal) => ({
  ...(await importOriginal()),
  getEngineShadowRunnerDaemonJsonRunner: () => selectedQueryRunner,
}));

/**
 * Empty AsyncIterable factory used to keep the indexer worker
 * idle — tests here exercise construction only, not file
 * walking.
 */
function emptySupplier(): AsyncIterable<FileTask> {
  return {
    [Symbol.asyncIterator](): AsyncIterator<FileTask> {
      return {
        next: () => Promise.resolve({ done: true, value: undefined as never }),
      };
    },
  };
}

describe("createServer transport discriminated union", () => {
  let client: LspTestClient | null = null;

  afterEach(() => {
    client?.dispose();
    client = null;
    selectedQueryRunner.mockClear();
    vi.unstubAllEnvs();
  });
  // Auto-transport construction requires the LanguageClient's
  // argv flags (`--node-ipc` / `--stdio`) to wire stdin/stdout.
  // Under vitest those flags are absent, so `createConnection`
  // throws a well-defined error. Asserting that shape documents
  // the contract — the discriminated-union branch is exercised,
  // even though the test environment cannot complete startup.
  it("routes the default shape (no transport field) through the auto branch", () => {
    expect(() =>
      createServer({
        fileSupplier: () => emptySupplier(),
        readStyleFileAsync: () => Promise.resolve(null),
      }),
    ).toThrow(/Connection input stream is not set/);
  });

  it("routes an explicit `transport: 'auto'` through the auto branch", () => {
    expect(() =>
      createServer({
        transport: "auto",
        fileSupplier: () => emptySupplier(),
        readStyleFileAsync: () => Promise.resolve(null),
      }),
    ).toThrow(/Connection input stream is not set/);
  });

  it("routes `transport: 'streams'` through the streams branch with no cast", () => {
    const serverToClient = new PassThrough();
    const clientToServer = new PassThrough();
    const reader = new StreamMessageReader(clientToServer);
    const writer = new StreamMessageWriter(serverToClient);
    const created = createServer({
      transport: "streams",
      reader,
      writer,
      fileSupplier: () => emptySupplier(),
      readStyleFileAsync: () => Promise.resolve(null),
    });
    expect(created.connection).toBeDefined();
    expect(created.documents).toBeDefined();
    created.connection.dispose();
  });

  it("downgrades source-corpus completeness when real dynamic watcher registration fails", async () => {
    vi.stubEnv("OMENA_SELECTED_QUERY_BACKEND", "rust-selected-query");
    const workspacePath = mkdtempSync(path.join(tmpdir(), "omena-watcher-coverage-"));
    try {
      mkdirSync(path.join(workspacePath, "src"));
      const sourcePath = path.join(workspacePath, "src/App.tsx");
      writeFileSync(sourcePath, "export const value = 1;\n");
      client = createInProcessServer({
        workspacePath,
        rejectDynamicWatcherRegistration: true,
        sourceFileSupplier: () => supplierFor([sourcePath]),
        readStyleFile: () => ".button { color: red; }\n",
        typeResolver: new FakeTypeResolver(),
      });
      await client.initialize({
        capabilities: {
          workspace: {
            didChangeWatchedFiles: { dynamicRegistration: true },
            workspaceFolders: true,
          },
        },
      });
      client.initialized();
      await client.waitForDynamicWatcherRegistration();
      const styleUri = "file:///fake/workspace/src/Button.module.scss";
      client.didOpen({
        textDocument: {
          uri: styleUri,
          languageId: "scss",
          version: 1,
          text: ".button { color: red; }\n",
        },
      });
      await client.waitForDiagnostics(styleUri);

      expect(selectedQueryRunner.mock.calls.map(([command]) => command)).toContain(
        "style-diagnostics-for-file",
      );
      const selectorUsageCall = selectedQueryRunner.mock.calls.find(
        ([command]) => command === "style-diagnostics-for-file",
      );
      expect(selectorUsageCall?.[1]).toMatchObject({
        sourceCorpusComplete: false,
        sourceDocuments: [],
      });
    } finally {
      client?.dispose();
      client = null;
      rmSync(workspacePath, { recursive: true, force: true });
    }
  });
});

function supplierFor(paths: readonly string[]): AsyncIterable<FileTask> {
  return {
    async *[Symbol.asyncIterator](): AsyncGenerator<FileTask> {
      for (const filePath of paths) yield { path: filePath };
    },
  };
}
