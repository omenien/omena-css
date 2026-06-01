import { spawn } from "node:child_process";
import { strict as assert } from "node:assert";
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import os from "node:os";
import path from "node:path";
import { performance } from "node:perf_hooks";
import { pathToFileURL } from "node:url";
import {
  createProtocolConnection,
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

const RUNTIME_LOOP_PROBE_REQUEST = "omena/runtimeLoopProbe";
const SELECTOR_COUNT = parsePositiveInteger(process.env.OMENA_LSP_RUNTIME_LOOP_SELECTORS, 50);
const PROBE_INTERVAL_MS = parsePositiveInteger(
  process.env.OMENA_LSP_RUNTIME_LOOP_PROBE_INTERVAL_MS,
  20,
);
const PROBE_DURATION_MS = parsePositiveInteger(
  process.env.OMENA_LSP_RUNTIME_LOOP_PROBE_DURATION_MS,
  1_200,
);
const MAX_PROBE_MS = parsePositiveInteger(process.env.OMENA_LSP_RUNTIME_LOOP_MAX_MS, 400);
const P95_PROBE_MS = parsePositiveInteger(process.env.OMENA_LSP_RUNTIME_LOOP_P95_MS, 150);
const REQUEST_TIMEOUT_MS = parsePositiveInteger(
  process.env.OMENA_LSP_RUNTIME_LOOP_REQUEST_TIMEOUT_MS,
  10_000,
);

interface RuntimeProbeResponse {
  readonly now: number;
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
  const sourcePath = path.join(srcDir, "App.tsx");
  const stylePath = path.join(srcDir, "App.module.scss");
  const sourceUri = pathToFileURL(sourcePath).toString();
  const styleUri = pathToFileURL(stylePath).toString();
  const sourceText = buildSourceText(SELECTOR_COUNT);
  const styleText = buildStyleText(SELECTOR_COUNT);
  const invocation = resolveOmenaLspServerInvocation();
  const diagnostics: unknown[] = [];

  mkdirSync(srcDir, { recursive: true });
  writeFileSync(sourcePath, sourceText);
  writeFileSync(stylePath, styleText);

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
    assert.equal(initialized.serverInfo?.name, "css-module-explainer-rust");

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
        uri: sourceUri,
        languageId: "typescriptreact",
        version: 1,
        text: sourceText,
      },
    });

    await requestWithTimeout(
      connection.sendRequest("textDocument/hover", sourceHoverParams(sourceUri, sourceText, 0)),
      "warmup hover",
    );

    const probePromise = collectProbeLatencies(connection);
    const loadResults = await Promise.all(
      buildHotRequestLoad(connection, sourceUri, sourceText, styleUri),
    );
    const probeLatencies = await probePromise;

    assertHotRequestResults(loadResults);
    assertProbeMetrics(probeLatencies);

    await requestWithTimeout(connection.sendRequest(ShutdownRequest.type), "shutdown");
    connection.sendNotification("exit");
    const exitCode = await waitForExit(child);
    assert.equal(exitCode, 0, `omena-lsp-server exited with ${exitCode}\n${stderr.join("")}`);

    process.stdout.write(
      [
        "omena-lsp-server runtime loop ok:",
        `command=${invocation.command}`,
        `selectors=${SELECTOR_COUNT}`,
        `requests=${loadResults.length}`,
        `probes=${probeLatencies.length}`,
        `diagnosticNotifications=${diagnostics.length}`,
        `p95=${percentile(probeLatencies, 95).toFixed(2)}ms`,
        `max=${Math.max(...probeLatencies).toFixed(2)}ms`,
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
  if (p95 > P95_PROBE_MS || max > MAX_PROBE_MS) {
    throw new Error(
      [
        "omena-lsp-server runtime loop probe exceeded budget",
        `p95=${p95.toFixed(2)}ms budget=${P95_PROBE_MS}ms`,
        `max=${max.toFixed(2)}ms budget=${MAX_PROBE_MS}ms`,
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
