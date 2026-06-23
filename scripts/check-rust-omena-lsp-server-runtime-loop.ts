import { spawn } from "node:child_process";
import { strict as assert } from "node:assert";
import { mkdirSync, mkdtempSync, realpathSync, rmSync, writeFileSync } from "node:fs";
import os from "node:os";
import path from "node:path";
import { performance } from "node:perf_hooks";
import { fileURLToPath, pathToFileURL } from "node:url";
import {
  createProtocolConnection,
  DidChangeTextDocumentNotification,
  DidOpenTextDocumentNotification,
  InitializedNotification,
  InitializeRequest,
  ShutdownRequest,
  type InitializeParams,
  type InitializeResult,
  type ProtocolConnection,
} from "vscode-languageserver-protocol/node";
import { StreamMessageReader, StreamMessageWriter } from "vscode-jsonrpc/node";
import { resolveOmenaLspServerInvocation } from "./omena-lsp-server-invocation";

const DEBUG_STATE_REQUEST = "omena/rustLspState";
const RUNTIME_LOOP_PROBE_REQUEST = "omena/runtimeLoopProbe";
const SELECTOR_COUNT = parsePositiveInteger(process.env.OMENA_LSP_RUNTIME_LOOP_SELECTORS, 50);
const STYLE_IMPORTER_COUNT = parsePositiveInteger(
  process.env.OMENA_LSP_RUNTIME_LOOP_STYLE_IMPORTERS,
  4,
);
const PROBE_INTERVAL_MS = parsePositiveInteger(
  process.env.OMENA_LSP_RUNTIME_LOOP_PROBE_INTERVAL_MS,
  20,
);
// Budget calibration (2026-06-12, RFC 0009 Pillar A slice A-min, rfcs#67): hover and
// definition requests now leave the loop — the server dispatches them to a query worker
// with a copy-on-write snapshot, so the probe (which stays loop-side ON PURPOSE) no
// longer measures their burst drain. What the probe measures now is the turn time of
// everything still synchronous: didOpen/didChange + diagnostics scheduling, the tsgo
// round trip, references/completion, and the O(documents) snapshot build at dispatch.
// Budgets stay at the Pillar E values (1.2s window / p95 150 / max 400): they still
// gate mutation/notification stalls, and a regression that drags hover/definition back
// onto the loop (or makes the snapshot build deep-clone the corpus) trips them again.
// Per-class request latency for the dispatched lane stays visible via the
// requestLatency summary printed below. (History: pre-Pillar-E interim values were 4s
// window / p95 1200 / max 2500; Pillar E restored the tight values; Pillar A keeps
// them with the loop no longer serving the hover/definition class.)
const PROBE_DURATION_MS = parsePositiveInteger(
  process.env.OMENA_LSP_RUNTIME_LOOP_PROBE_DURATION_MS,
  1_200,
);
const MAX_PROBE_MS = parsePositiveInteger(process.env.OMENA_LSP_RUNTIME_LOOP_MAX_MS, 400);
const SINGLE_OUTLIER_PROBE_MS = parsePositiveInteger(
  process.env.OMENA_LSP_RUNTIME_LOOP_SINGLE_OUTLIER_MS,
  750,
);
const P95_PROBE_MS = parsePositiveInteger(process.env.OMENA_LSP_RUNTIME_LOOP_P95_MS, 150);
const REQUEST_TIMEOUT_MS = parsePositiveInteger(
  process.env.OMENA_LSP_RUNTIME_LOOP_REQUEST_TIMEOUT_MS,
  10_000,
);

interface RuntimeProbeResponse {
  readonly now: number;
}

interface DebugStateResponse {
  readonly externalSifLockReadCount?: number;
  readonly externalSifBridgeGenerationCount?: number;
}

interface TimedHotRequestResult {
  readonly kind: string;
  readonly label: string;
  readonly durationMs: number;
  readonly result: unknown;
}

interface RequestLatencySummary {
  readonly kind: string;
  readonly count: number;
  readonly p50Ms: number;
  readonly p95Ms: number;
  readonly maxMs: number;
}

async function main(): Promise<void> {
  const workspaceRoot = mkdtempSync(path.join(os.tmpdir(), "cme-rust-omena-lsp-runtime-loop-"));
  const srcDir = path.join(workspaceRoot, "src");
  const vendorDir = path.join(workspaceRoot, "vendor");
  const sourcePath = path.join(srcDir, "App.tsx");
  const stylePath = path.join(srcDir, "App.module.scss");
  const peerStylePath = path.join(srcDir, "Peer.module.scss");
  const sharedPartialPath = path.join(srcDir, "_shared.scss");
  const importerStylePaths = Array.from({ length: STYLE_IMPORTER_COUNT }, (_, index) =>
    path.join(srcDir, `Importer${index}.module.scss`),
  );
  const bridgePath = path.join(vendorDir, "_tokens.scss");
  const sourceUri = pathToFileURL(sourcePath).toString();
  const styleUri = pathToFileURL(stylePath).toString();
  const peerStyleUri = pathToFileURL(peerStylePath).toString();
  const sharedPartialUri = pathToFileURL(sharedPartialPath).toString();
  const importerStyleUris = importerStylePaths.map((filePath) =>
    pathToFileURL(filePath).toString(),
  );
  const bridgeUri = pathToFileURL(bridgePath).toString();
  const sourceText = buildSourceText(SELECTOR_COUNT);
  const styleText = buildStyleText(SELECTOR_COUNT);
  const changedStyleText = buildChangedStyleText(SELECTOR_COUNT);
  const peerStyleText = buildPeerStyleText(bridgeUri);
  const sharedPartialText = buildSharedPartialText("red");
  const changedSharedPartialText = buildSharedPartialText("green");
  const invocation = resolveOmenaLspServerInvocation();
  const diagnostics: unknown[] = [];

  mkdirSync(srcDir, { recursive: true });
  mkdirSync(vendorDir, { recursive: true });
  writeFileSync(sourcePath, sourceText);
  writeFileSync(stylePath, styleText);
  writeFileSync(peerStylePath, peerStyleText);
  writeFileSync(sharedPartialPath, sharedPartialText);
  for (const [index, filePath] of importerStylePaths.entries()) {
    writeFileSync(filePath, buildImporterStyleText(index));
  }
  writeFileSync(bridgePath, "$brand: red !default;\n");
  writeFileSync(
    path.join(workspaceRoot, "omena.lock"),
    JSON.stringify({ lockfileVersion: "1", entries: [] }) + "\n",
  );

  const child = spawn(invocation.command, [...invocation.args], {
    cwd: process.cwd(),
    stdio: ["pipe", "pipe", "pipe"],
  });
  const stderr: string[] = [];
  child.stderr.setEncoding("utf8");
  child.stderr.on("data", (chunk) => stderr.push(chunk));

  const connection: ProtocolConnection = createProtocolConnection(
    new StreamMessageReader(child.stdout),
    new StreamMessageWriter(child.stdin),
  );
  connection.onNotification("textDocument/publishDiagnostics", (params) => {
    diagnostics.push(params);
  });
  connection.listen();

  try {
    const initialized = await requestWithTimeout(
      connection.sendRequest<InitializeResult>(
        InitializeRequest.type,
        initializeParams(workspaceRoot),
      ),
      "initialize",
    );
    assert.equal(initialized.serverInfo?.name, "omena-css-rust");

    connection.sendNotification(InitializedNotification.type, {});
    connection.sendNotification(DidOpenTextDocumentNotification.type, {
      textDocument: {
        uri: styleUri,
        languageId: "scss",
        version: 1,
        text: styleText,
      },
    });
    connection.sendNotification(DidOpenTextDocumentNotification.type, {
      textDocument: {
        uri: peerStyleUri,
        languageId: "scss",
        version: 1,
        text: peerStyleText,
      },
    });
    connection.sendNotification(DidOpenTextDocumentNotification.type, {
      textDocument: {
        uri: sharedPartialUri,
        languageId: "scss",
        version: 1,
        text: sharedPartialText,
      },
    });
    for (const [index, uri] of importerStyleUris.entries()) {
      connection.sendNotification(DidOpenTextDocumentNotification.type, {
        textDocument: {
          uri,
          languageId: "scss",
          version: 1,
          text: buildImporterStyleText(index),
        },
      });
    }
    connection.sendNotification(DidOpenTextDocumentNotification.type, {
      textDocument: {
        uri: sourceUri,
        languageId: "typescriptreact",
        version: 1,
        text: sourceText,
      },
    });
    await waitForDiagnosticsForUris(
      () => diagnostics,
      importerStyleUris,
      "initial shared-style importer diagnostics",
    );

    await requestWithTimeout(
      connection.sendRequest("textDocument/hover", sourceHoverParams(sourceUri, sourceText, 0)),
      "warmup hover",
    );

    const externalCountersBeforeChange = await readDebugState(
      connection,
      "debug-state:before-change",
    );
    const probePromise = collectProbeLatencies(connection);
    connection.sendNotification(DidChangeTextDocumentNotification.type, {
      textDocument: {
        uri: styleUri,
        version: 2,
      },
      contentChanges: [{ text: changedStyleText }],
    });
    const externalCountersAfterChangePromise = readDebugState(
      connection,
      "debug-state:after-change",
    );
    const loadResults = await Promise.all(
      buildHotRequestLoad(connection, sourceUri, sourceText, styleUri),
    );
    const externalCountersAfterChange = await externalCountersAfterChangePromise;
    const probeLatencies = await probePromise;

    assertExternalSifCountersStableOnPlainDidChange(
      externalCountersBeforeChange,
      externalCountersAfterChange,
    );
    assertHotRequestResults(loadResults);
    assertProbeMetrics(probeLatencies);

    const diagnosticsBeforeFanout = diagnostics.length;
    const fanoutProbePromise = collectProbeLatencies(connection);
    connection.sendNotification(DidChangeTextDocumentNotification.type, {
      textDocument: {
        uri: sharedPartialUri,
        version: 2,
      },
      contentChanges: [{ text: changedSharedPartialText }],
    });
    await waitForDiagnosticsForUris(
      () => diagnostics.slice(diagnosticsBeforeFanout),
      importerStyleUris,
      "shared style fan-out diagnostics",
    );
    const fanoutProbeLatencies = await fanoutProbePromise;
    assertProbeMetrics(fanoutProbeLatencies);

    await requestWithTimeout(connection.sendRequest(ShutdownRequest.type), "shutdown");
    connection.sendNotification("exit");
    const exitCode = await waitForExit(child);
    assert.equal(exitCode, 0, `omena-lsp-server exited with ${exitCode}\n${stderr.join("")}`);

    process.stdout.write(
      [
        "omena-lsp-server runtime loop ok:",
        `command=${invocation.command}`,
        `selectors=${SELECTOR_COUNT}`,
        `styleImporters=${STYLE_IMPORTER_COUNT}`,
        `requests=${loadResults.length}`,
        `probes=${probeLatencies.length}`,
        `fanoutProbes=${fanoutProbeLatencies.length}`,
        `diagnosticNotifications=${diagnostics.length}`,
        `p95=${percentile(probeLatencies, 95).toFixed(2)}ms`,
        `max=${Math.max(...probeLatencies).toFixed(2)}ms`,
        `fanoutP95=${percentile(fanoutProbeLatencies, 95).toFixed(2)}ms`,
        `fanoutMax=${Math.max(...fanoutProbeLatencies).toFixed(2)}ms`,
        `requestLatency=[${formatRequestLatencySummaries(loadResults)}]`,
      ].join(" ") + "\n",
    );
  } finally {
    connection.dispose();
    if (!child.killed && child.exitCode === null) {
      child.kill();
    }
    rmSync(workspaceRoot, { force: true, recursive: true });
  }
}

function initializeParams(workspaceRoot: string): InitializeParams {
  const workspaceUri = pathToFileURL(workspaceRoot).toString();
  return {
    processId: process.pid,
    rootUri: workspaceUri,
    workspaceFolders: [{ uri: workspaceUri, name: "cme-rust-omena-lsp-runtime-loop" }],
    capabilities: {
      workspace: {
        configuration: true,
        workspaceFolders: true,
      },
      textDocument: {
        publishDiagnostics: {},
      },
    },
  };
}

function buildHotRequestLoad(
  connection: ProtocolConnection,
  sourceUri: string,
  sourceText: string,
  styleUri: string,
): Array<Promise<TimedHotRequestResult>> {
  const requests: Array<Promise<TimedHotRequestResult>> = [];
  for (let index = 0; index < SELECTOR_COUNT; index += 1) {
    requests.push(
      timedHotRequest(
        connection.sendRequest(
          "textDocument/hover",
          sourceHoverParams(sourceUri, sourceText, index),
        ),
        `source-hover:${index}`,
        "source-hover",
      ),
    );
    if (index % 2 === 0) {
      requests.push(
        timedHotRequest(
          connection.sendRequest(
            "textDocument/definition",
            sourceHoverParams(sourceUri, sourceText, index),
          ),
          `source-definition:${index}`,
          "source-definition",
        ),
      );
    }
    if (index % 5 === 0) {
      requests.push(
        timedHotRequest(
          connection.sendRequest("textDocument/references", styleReferenceParams(styleUri, index)),
          `style-references:${index}`,
          "style-references",
        ),
      );
      requests.push(
        timedHotRequest(
          connection.sendRequest(
            "textDocument/completion",
            sourceCompletionParams(sourceUri, sourceText, index),
          ),
          `source-completion:${index}`,
          "source-completion",
        ),
      );
    }
  }
  return requests;
}

async function collectProbeLatencies(connection: ProtocolConnection): Promise<readonly number[]> {
  const latencies: number[] = [];
  const deadline = performance.now() + PROBE_DURATION_MS;
  let seq = 0;
  while (performance.now() < deadline) {
    const started = performance.now();
    // oxlint-disable-next-line eslint/no-await-in-loop
    await requestWithTimeout(
      connection.sendRequest<RuntimeProbeResponse>(RUNTIME_LOOP_PROBE_REQUEST, { seq: ++seq }),
      `runtime-probe:${seq}`,
    );
    latencies.push(performance.now() - started);
    // oxlint-disable-next-line eslint/no-await-in-loop
    await sleep(PROBE_INTERVAL_MS);
  }
  return latencies;
}

function assertProbeMetrics(latencies: readonly number[]): void {
  if (latencies.length < 10) {
    throw new Error(`Too few runtime probes completed: ${latencies.length}`);
  }

  const p95 = percentile(latencies, 95);
  const max = Math.max(...latencies);
  const overBudgetCount = latencies.filter((latency) => latency > MAX_PROBE_MS).length;
  const hasSustainedMaxRegression =
    max > MAX_PROBE_MS && (overBudgetCount > 1 || max > SINGLE_OUTLIER_PROBE_MS);
  if (p95 > P95_PROBE_MS || hasSustainedMaxRegression) {
    throw new Error(
      [
        "omena-lsp-server runtime loop probe exceeded budget",
        `p95=${p95.toFixed(2)}ms budget=${P95_PROBE_MS}ms`,
        `max=${max.toFixed(2)}ms budget=${MAX_PROBE_MS}ms`,
        `singleOutlierBudget=${SINGLE_OUTLIER_PROBE_MS}ms`,
        `overBudgetSamples=${overBudgetCount}`,
        `samples=${latencies.length}`,
      ].join("\n"),
    );
  }
}

function assertHotRequestResults(results: readonly TimedHotRequestResult[]): void {
  if (results.length === 0) {
    throw new Error("No hot omena-lsp-server requests completed.");
  }
  const nullCount = results.filter(({ result }) => result === null).length;
  if (nullCount > results.length / 2) {
    throw new Error(`Too many null hot request results: ${nullCount}/${results.length}`);
  }
}

function sourceHoverParams(uri: string, text: string, tokenIndex: number) {
  return {
    textDocument: { uri },
    position: sourceTokenPosition(text, tokenIndex),
  };
}

function sourceCompletionParams(uri: string, text: string, tokenIndex: number) {
  return {
    textDocument: { uri },
    position: sourceTokenPrefixPosition(text, tokenIndex),
  };
}

function styleReferenceParams(uri: string, tokenIndex: number) {
  return {
    textDocument: { uri },
    position: { line: tokenIndex, character: 2 },
    context: { includeDeclaration: false },
  };
}

function sourceTokenPrefixPosition(
  text: string,
  tokenIndex: number,
): { line: number; character: number } {
  const token = `"token${tokenIndex}"`;
  const index = text.indexOf(token);
  if (index < 0) {
    throw new Error(`Unable to find ${token}`);
  }
  const before = text.slice(0, index + 1 + "token".length);
  const lines = before.split("\n");
  return {
    line: lines.length - 1,
    character: lines.at(-1)!.length,
  };
}

function sourceTokenPosition(
  text: string,
  tokenIndex: number,
): { line: number; character: number } {
  const token = `"token${tokenIndex}"`;
  const index = text.indexOf(token);
  if (index < 0) {
    throw new Error(`Unable to find ${token}`);
  }
  const before = text.slice(0, index + 1);
  const lines = before.split("\n");
  return {
    line: lines.length - 1,
    character: lines.at(-1)!.length,
  };
}

function buildSourceText(count: number): string {
  const rows = Array.from(
    { length: count },
    (_, index) => `      <span className={cx("token${index}")}>${index}</span>`,
  ).join("\n");
  return `import classNames from "classnames/bind";
import styles from "./App.module.scss";

const cx = classNames.bind(styles);

export function App() {
  return (
    <div>
${rows}
    </div>
  );
}
`;
}

function buildStyleText(count: number): string {
  return Array.from(
    { length: count },
    (_, index) => `.token${index} { color: rgb(${index % 255}, 0, 0); }`,
  ).join("\n");
}

function buildChangedStyleText(count: number): string {
  return Array.from(
    { length: count },
    (_, index) => `.token${index} { color: rgb(0, ${index % 255}, 0); }`,
  ).join("\n");
}

function buildPeerStyleText(bridgeUri: string): string {
  return `@use "${bridgeUri}" as tokens;
.peer { color: tokens.$brand; }
`;
}

function buildSharedPartialText(color: string): string {
  return `$tone: ${color};\n`;
}

function buildImporterStyleText(index: number): string {
  return `@use "./shared";
.importer${index} { width: var(--missing-importer-${index}); color: red; color: blue; }
`;
}

async function readDebugState(
  connection: ProtocolConnection,
  label: string,
): Promise<DebugStateResponse> {
  return requestWithTimeout(
    connection.sendRequest<DebugStateResponse>(DEBUG_STATE_REQUEST, {}),
    label,
  );
}

function assertExternalSifCountersStableOnPlainDidChange(
  before: DebugStateResponse,
  after: DebugStateResponse,
): void {
  const lockReadsBefore = before.externalSifLockReadCount ?? 0;
  const lockReadsAfter = after.externalSifLockReadCount ?? 0;
  const bridgeGenerationsBefore = before.externalSifBridgeGenerationCount ?? 0;
  const bridgeGenerationsAfter = after.externalSifBridgeGenerationCount ?? 0;
  assert.equal(
    lockReadsAfter - lockReadsBefore,
    0,
    "plain style didChange must not reread workspace lockfiles",
  );
  assert.equal(
    bridgeGenerationsAfter - bridgeGenerationsBefore,
    0,
    "plain style didChange must not regenerate bridge SIFs",
  );
}

async function waitForDiagnosticsForUris(
  diagnosticsSnapshot: () => readonly unknown[],
  expectedUris: readonly string[],
  label: string,
): Promise<void> {
  const deadline = performance.now() + REQUEST_TIMEOUT_MS;
  const expectedCanonicalUris = expectedUris.map(canonicalFileUriForComparison);
  while (performance.now() < deadline) {
    const seen = new Set(
      diagnosticsSnapshot()
        .map((entry) => diagnosticUri(entry))
        .filter((uri): uri is string => typeof uri === "string")
        .map(canonicalFileUriForComparison),
    );
    if (expectedCanonicalUris.every((uri) => seen.has(uri))) {
      return;
    }
    // oxlint-disable-next-line eslint/no-await-in-loop
    await sleep(25);
  }
  const seenUris = [
    ...new Set(
      diagnosticsSnapshot()
        .map((entry) => diagnosticUri(entry))
        .filter((uri): uri is string => typeof uri === "string")
        .map(canonicalFileUriForComparison),
    ),
  ].sort();
  throw new Error(
    `${label} timed out; missing=${expectedCanonicalUris
      .filter((uri) => !seenUris.includes(uri))
      .join(",")} seen=${seenUris.join(",")}`,
  );
}

function diagnosticUri(value: unknown): string | undefined {
  if (!value || typeof value !== "object" || !("uri" in value)) {
    return undefined;
  }
  const uri = (value as { uri?: unknown }).uri;
  return typeof uri === "string" ? uri : undefined;
}

function canonicalFileUriForComparison(uri: string): string {
  if (!uri.startsWith("file:")) return uri;
  try {
    return pathToFileURL(realpathSync.native(fileURLToPath(uri))).toString();
  } catch {
    return uri;
  }
}

async function requestWithTimeout<T>(promise: Promise<T>, label: string): Promise<T> {
  let timer: NodeJS.Timeout | null = null;
  try {
    return await Promise.race([
      promise,
      new Promise<T>((_, reject) => {
        timer = setTimeout(() => reject(new Error(`${label} timed out`)), REQUEST_TIMEOUT_MS);
      }),
    ]);
  } finally {
    if (timer) clearTimeout(timer);
  }
}

async function timedHotRequest<T>(
  promise: Promise<T>,
  label: string,
  kind: string,
): Promise<TimedHotRequestResult> {
  const started = performance.now();
  const result = await requestWithTimeout(promise, label);
  return {
    kind,
    label,
    durationMs: performance.now() - started,
    result,
  };
}

function waitForExit(child: ReturnType<typeof spawn>): Promise<number | null> {
  return new Promise((resolve) => {
    child.once("exit", (code) => resolve(code));
    child.once("close", (code) => resolve(code));
  });
}

function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

function percentile(values: readonly number[], p: number): number {
  const sorted = values.toSorted((left, right) => left - right);
  const index = Math.min(sorted.length - 1, Math.max(0, Math.ceil((p / 100) * sorted.length) - 1));
  return sorted[index] ?? 0;
}

function summarizeRequestLatencies(
  results: readonly TimedHotRequestResult[],
): readonly RequestLatencySummary[] {
  const byKind = new Map<string, number[]>();
  for (const result of results) {
    const values = byKind.get(result.kind);
    if (values) {
      values.push(result.durationMs);
    } else {
      byKind.set(result.kind, [result.durationMs]);
    }
  }
  return [...byKind.entries()]
    .map(([kind, values]) => ({
      kind,
      count: values.length,
      p50Ms: percentile(values, 50),
      p95Ms: percentile(values, 95),
      maxMs: Math.max(...values),
    }))
    .toSorted((left, right) => left.kind.localeCompare(right.kind));
}

function formatRequestLatencySummaries(results: readonly TimedHotRequestResult[]): string {
  return summarizeRequestLatencies(results)
    .map(
      (summary) =>
        `${summary.kind}:n=${summary.count},p50=${summary.p50Ms.toFixed(2)}ms,p95=${summary.p95Ms.toFixed(2)}ms,max=${summary.maxMs.toFixed(2)}ms`,
    )
    .join(";");
}

function parsePositiveInteger(value: string | undefined, fallback: number): number {
  if (!value) return fallback;
  const parsed = Number.parseInt(value, 10);
  return Number.isFinite(parsed) && parsed > 0 ? parsed : fallback;
}

void main().catch((error) => {
  console.error(error);
  process.exit(1);
});
