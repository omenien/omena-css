import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import os from "node:os";
import path from "node:path";
import { spawn } from "node:child_process";
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

const REPO_ROOT = process.cwd();
const SERVER_ENTRY = path.join(REPO_ROOT, "dist/server/server.js");
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
  const workspaceRoot = mkdtempSync(path.join(os.tmpdir(), "cme-lsp-runtime-loop-"));
  const srcDir = path.join(workspaceRoot, "src");
  const sourcePath = path.join(srcDir, "App.tsx");
  const stylePath = path.join(srcDir, "App.module.scss");
  const sourceUri = pathToFileURL(sourcePath).toString();
  const styleUri = pathToFileURL(stylePath).toString();
  const sourceText = buildSourceText(SELECTOR_COUNT);
  const styleText = buildStyleText(SELECTOR_COUNT);

  mkdirSync(srcDir, { recursive: true });
  writeFileSync(
    path.join(workspaceRoot, "tsconfig.json"),
    JSON.stringify(
      {
        compilerOptions: {
          target: "ES2022",
          module: "ESNext",
          moduleResolution: "Bundler",
          jsx: "react-jsx",
          strict: true,
          allowJs: false,
          noEmit: true,
        },
        include: ["src/**/*"],
      },
      null,
      2,
    ),
  );
  writeFileSync(sourcePath, sourceText);
  writeFileSync(stylePath, styleText);

  const stderr: string[] = [];
  const child = spawn(process.execPath, [SERVER_ENTRY, "--stdio"], {
    cwd: REPO_ROOT,
    env: {
      ...process.env,
      OMENA_ENGINE_SHADOW_RUNNER: "prebuilt",
      OMENA_ENGINE_SHADOW_RUNNER_DAEMON: "1",
      OMENA_ENGINE_SHADOW_RUNNER_DAEMON_TRACE: "1",
      OMENA_LSP_RUNTIME_LOOP_PROBE: "1",
      OMENA_PROJECT_ROOT: REPO_ROOT,
      OMENA_SELECTED_QUERY_BACKEND: "rust-selected-query",
    },
    stdio: ["pipe", "pipe", "pipe"],
  });
  child.stderr.setEncoding("utf8");
  child.stderr.on("data", (chunk) => stderr.push(chunk));

  const connection: ProtocolConnection = createProtocolConnection(
    new StreamMessageReader(child.stdout),
    new StreamMessageWriter(child.stdin),
  );
  connection.onRequest(
    "workspace/configuration",
    (params: { items: Array<{ section?: string; scopeUri?: string }> }) =>
      params.items.map(() => ({})),
  );
  connection.listen();

  try {
    const initialized = await requestWithTimeout(
      connection.sendRequest<InitializeResult>(
        InitializeRequest.type,
        initializeParams(workspaceRoot),
      ),
      "initialize",
    );
    if (initialized.serverInfo?.name !== "css-module-explainer") {
      throw new Error(`Unexpected server name: ${initialized.serverInfo?.name ?? "<missing>"}`);
    }

    connection.sendNotification(InitializedNotification.type, {});
    connection.sendNotification(DidOpenTextDocumentNotification.type, {
      textDocument: {
        uri: sourceUri,
        languageId: "typescriptreact",
        version: 1,
        text: sourceText,
      },
    });
    connection.sendNotification(DidOpenTextDocumentNotification.type, {
      textDocument: {
        uri: styleUri,
        languageId: "scss",
        version: 1,
        text: styleText,
      },
    });

    await requestWithTimeout(
      connection.sendRequest("textDocument/hover", hoverParams(sourceUri, sourceText, 0)),
      "warmup hover",
    );

    const probePromise = collectProbeLatencies(connection);
    const loadResults = await Promise.all(
      buildHotRequestLoad(connection, sourceUri, sourceText, styleUri),
    );
    const probeLatencies = await probePromise;

    assertHotRequestResults(loadResults);
    assertProbeMetrics(probeLatencies);
    assertDaemonSpawnCount(stderr.join(""));

    await requestWithTimeout(connection.sendRequest(ShutdownRequest.type), "shutdown");
    connection.sendNotification("exit");

    process.stdout.write(
      [
        "lsp runtime loop ok:",
        `selectors=${SELECTOR_COUNT}`,
        `requests=${loadResults.length}`,
        `probes=${probeLatencies.length}`,
        `p95=${percentile(probeLatencies, 95).toFixed(2)}ms`,
        `max=${Math.max(...probeLatencies).toFixed(2)}ms`,
        `requestLatency=[${formatRequestLatencySummaries(loadResults)}]`,
        "daemonSpawns=1",
      ].join(" ") + "\n",
    );
  } finally {
    connection.dispose();
    child.kill();
    await waitForExit(child);
    rmSync(workspaceRoot, { force: true, recursive: true });
  }
}

function initializeParams(workspaceRoot: string): InitializeParams {
  const workspaceUri = pathToFileURL(workspaceRoot).toString();
  return {
    processId: process.pid,
    rootUri: workspaceUri,
    workspaceFolders: [{ uri: workspaceUri, name: "cme-lsp-runtime-loop" }],
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
        connection.sendRequest("textDocument/hover", hoverParams(sourceUri, sourceText, index)),
        `hover:${index}`,
        "source-hover",
      ),
    );
    if (index % 2 === 0) {
      requests.push(
        timedHotRequest(
          connection.sendRequest(
            "textDocument/definition",
            hoverParams(sourceUri, sourceText, index),
          ),
          `definition:${index}`,
          "source-definition",
        ),
      );
    }
    if (index % 5 === 0) {
      requests.push(
        timedHotRequest(
          connection.sendRequest("textDocument/references", referenceParams(styleUri, index)),
          `references:${index}`,
          "style-references",
        ),
      );
      requests.push(
        timedHotRequest(
          connection.sendRequest(
            "textDocument/completion",
            completionParams(sourceUri, sourceText, index),
          ),
          `completion:${index}`,
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
        "LSP runtime loop probe exceeded budget",
        `p95=${p95.toFixed(2)}ms budget=${P95_PROBE_MS}ms`,
        `max=${max.toFixed(2)}ms budget=${MAX_PROBE_MS}ms`,
        `samples=${latencies.length}`,
      ].join("\n"),
    );
  }
}

function assertHotRequestResults(results: readonly TimedHotRequestResult[]): void {
  if (results.length === 0) {
    throw new Error("No hot LSP requests completed.");
  }
  const nullCount = results.filter(({ result }) => result === null).length;
  if (nullCount > results.length / 2) {
    throw new Error(`Too many null hot request results: ${nullCount}/${results.length}`);
  }
}

function assertDaemonSpawnCount(stderr: string): void {
  const spawnCount = [...stderr.matchAll(/engine-shadow-runner daemon spawned/g)].length;
  if (spawnCount !== 1) {
    throw new Error(`Expected exactly one daemon spawn, saw ${spawnCount}\n${stderr}`);
  }
}

function hoverParams(uri: string, text: string, tokenIndex: number) {
  return {
    textDocument: { uri },
    position: sourceTokenPosition(text, tokenIndex),
  };
}

function completionParams(uri: string, text: string, tokenIndex: number) {
  return {
    textDocument: { uri },
    position: sourceTokenPrefixPosition(text, tokenIndex),
  };
}

function referenceParams(uri: string, tokenIndex: number) {
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
