import type { Connection } from "vscode-languageserver/node";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { DEFAULT_WINDOW_SETTINGS } from "../../server/engine-core-ts/src/settings";
import type { RustSelectedQueryBackendJsonRunnerAsync } from "../../server/engine-host-node/src/selected-query-backend";
import type { SourceCorpusFileRead } from "../../server/engine-host-node/src/runtime/workspace-source-path-inventory";
import {
  createDiagnosticsScheduler,
  type DiagnosticsSchedulerDeps,
  type DiagnosticsScheduler,
} from "../../server/lsp-server/src/diagnostics-scheduler";
import { makeBaseDeps } from "../_fixtures/test-helpers";
import { makeStyleDocumentFixture, makeTestSelector } from "../_fixtures/style-documents";

const STYLE_URI = "file:///fake/ws/styles/a.module.scss";
const STYLE_PATH = "/fake/ws/styles/a.module.scss";
const SOURCE_PATH = "/fake/ws/src/App.tsx";

describe("diagnostics scheduler source-corpus supply", () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
    vi.unstubAllEnvs();
  });

  it("pays zero source-corpus collection cost on the discarded TypeScript path", async () => {
    vi.stubEnv("OMENA_SELECTED_QUERY_BACKEND", "typescript-current");
    const completeSourcePathEnumeration = vi.fn(() => [SOURCE_PATH]);
    const readSourceFileForCorpus = vi.fn<() => SourceCorpusFileRead | null>(() => {
      throw new Error("discarded backend must not read the source corpus");
    });
    const runner = makeRunner();
    const scheduler = makeScheduler({
      completeSourcePathEnumeration,
      readSourceFileForCorpus,
      runner,
    });

    await runOneScssTick(scheduler);

    expect(completeSourcePathEnumeration).not.toHaveBeenCalled();
    expect(readSourceFileForCorpus).not.toHaveBeenCalled();
    expect(runner).not.toHaveBeenCalled();
    expect(scheduler.sourceCorpusSupplyCounters()).toEqual({
      collections: 0,
      skippedForBackend: 1,
      incompleteCollections: 0,
      suppliedFiles: 0,
      suppliedBytes: 0,
      diskReadFiles: 0,
      diskReadBytes: 0,
      cacheHitFiles: 0,
    });
  });

  it("supplies a non-empty Rust corpus and accounts cached file and byte volume", async () => {
    vi.stubEnv("OMENA_SELECTED_QUERY_BACKEND", "rust-selected-query");
    const source = "import styles from '../styles/a.module.scss';\nvoid styles.a;\n";
    let cached = false;
    const readSourceFileForCorpus = vi.fn((): SourceCorpusFileRead => {
      const read = {
        source,
        utf8Bytes: Buffer.byteLength(source, "utf8"),
        cacheHit: cached,
      };
      cached = true;
      return read;
    });
    const runner = makeRunner();
    const scheduler = makeScheduler({
      completeSourcePathEnumeration: () => [SOURCE_PATH],
      readSourceFileForCorpus,
      runner,
    });

    await runOneScssTick(scheduler);
    await runOneScssTick(scheduler);

    expect(readSourceFileForCorpus).toHaveBeenCalledTimes(2);
    expect(runner).toHaveBeenCalledTimes(2);
    expect(runner.mock.calls[0]?.[1]).toMatchObject({
      sourceDocuments: [{ sourcePath: SOURCE_PATH, sourceSource: source }],
      sourceCorpusComplete: true,
    });
    expect(scheduler.sourceCorpusSupplyCounters()).toEqual({
      collections: 2,
      skippedForBackend: 0,
      incompleteCollections: 0,
      suppliedFiles: 2,
      suppliedBytes: Buffer.byteLength(source, "utf8") * 2,
      diskReadFiles: 1,
      diskReadBytes: Buffer.byteLength(source, "utf8"),
      cacheHitFiles: 1,
    });
  });
});

function makeScheduler(args: {
  readonly completeSourcePathEnumeration: () => readonly string[] | null;
  readonly readSourceFileForCorpus: (filePath: string) => SourceCorpusFileRead | null;
  readonly runner: ReturnType<typeof makeRunner>;
}): DiagnosticsScheduler {
  const styleDocument = makeStyleDocumentFixture(STYLE_PATH, [makeTestSelector("a", 0)]);
  const providerDeps = {
    ...makeBaseDeps({
      styleDocumentForPath: (filePath) => (filePath === STYLE_PATH ? styleDocument : null),
      readStyleFile: (filePath) => (filePath === STYLE_PATH ? ".a { color: red; }\n" : null),
    }),
    completeSourcePathEnumeration: args.completeSourcePathEnumeration,
    readSourceFileForCorpus: args.readSourceFileForCorpus,
    runRustSelectedQueryBackendJsonAsync: args.runner,
  };
  const styleTextDocument = {
    uri: STYLE_URI,
    languageId: "scss",
    version: 1,
    getText: () => ".a { color: red; }\n",
  };
  const documents = {
    get: (uri: string) => (uri === STYLE_URI ? styleTextDocument : undefined),
    all: () => [styleTextDocument],
  } as unknown as DiagnosticsSchedulerDeps["documents"];
  const connection = {
    console: { error: vi.fn() },
    sendDiagnostics: vi.fn(),
  } as unknown as Connection;
  return createDiagnosticsScheduler(
    {
      connection,
      documents,
      getDeps: (uri) => (uri === STYLE_URI ? providerDeps : null),
      getAllDeps: () => [providerDeps],
    },
    DEFAULT_WINDOW_SETTINGS,
  );
}

function makeRunner() {
  return vi.fn(async () => ({
    product: "omena-query.diagnostics-for-file",
    fileKind: "style",
    diagnostics: [],
  })) as unknown as RustSelectedQueryBackendJsonRunnerAsync & ReturnType<typeof vi.fn>;
}

async function runOneScssTick(scheduler: DiagnosticsScheduler): Promise<void> {
  scheduler.ensureReadySubscribed();
  await Promise.resolve();
  scheduler.scheduleScss(STYLE_URI);
  await vi.advanceTimersByTimeAsync(250);
  await Promise.resolve();
}
