import { spawn } from "node:child_process";
import { strict as assert } from "node:assert";
import { mkdirSync, mkdtempSync, realpathSync, rmSync, writeFileSync } from "node:fs";
import os from "node:os";
import path from "node:path";
import { performance } from "node:perf_hooks";
import { fileURLToPath, pathToFileURL } from "node:url";
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

const DEBUG_STATE_REQUEST = "omena/rustLspState";
const RUNTIME_LOOP_PROBE_REQUEST = "omena/runtimeLoopProbe";
const REQUEST_TIMEOUT_MS = parsePositiveInteger(
  process.env.OMENA_LSP_EXTERNAL_SIF_REQUEST_TIMEOUT_MS,
  30_000,
);
const PROBE_DURATION_MS = parsePositiveInteger(
  process.env.OMENA_LSP_EXTERNAL_SIF_PROBE_DURATION_MS,
  1_200,
);
const PROBE_INTERVAL_MS = parsePositiveInteger(
  process.env.OMENA_LSP_EXTERNAL_SIF_PROBE_INTERVAL_MS,
  20,
);
const MAX_PROBE_MS = parsePositiveInteger(process.env.OMENA_LSP_EXTERNAL_SIF_MAX_MS, 1_500);
const P95_PROBE_MS = parsePositiveInteger(process.env.OMENA_LSP_EXTERNAL_SIF_P95_MS, 1_200);
const STEADY_MAX_PROBE_MS = parsePositiveInteger(
  process.env.OMENA_LSP_EXTERNAL_SIF_STEADY_MAX_MS,
  400,
);
const STEADY_P95_PROBE_MS = parsePositiveInteger(
  process.env.OMENA_LSP_EXTERNAL_SIF_STEADY_P95_MS,
  150,
);
const BLOCKING_EXTERNAL_CODES = new Set([
  "missingExternalSif",
  "missingSassSymbol",
  "staleExternalSif",
  "unresolvedExternalReference",
]);

interface PublishDiagnosticsParams {
  readonly uri: string;
  readonly diagnostics: readonly DiagnosticLike[];
}

interface DiagnosticLike {
  readonly code?: unknown;
  readonly message?: unknown;
}

interface RuntimeProbeResponse {
  readonly now: number;
}

interface DebugStateResponse {
  readonly externalSifBridgeGenerationCount?: number;
}

async function main(): Promise<void> {
  const workspaceRoot = mkdtempSync(path.join(os.tmpdir(), "omena-lsp-external-sif-runtime-"));
  const srcDir = path.join(workspaceRoot, "src");
  const appThemeDir = path.join(workspaceRoot, "node_modules", "@app", "theme");
  const designTokensDir = path.join(workspaceRoot, "node_modules", "@design", "tokens");
  const resolvedPath = path.join(srcDir, "Resolved.module.scss");
  const resolvedUri = pathToFileURL(resolvedPath).toString();
  const colorsUri = pathToFileURL(path.join(designTokensDir, "colors.scss")).toString();
  const radiusUri = pathToFileURL(path.join(appThemeDir, "_radius.scss")).toString();
  const resolvedText = `@use "@app/theme/index" as ds;
.external {
  color: ds.$ds_gray-700;
  border-radius: ds.$ds_radius-card;
}
`;
  const invocation = resolveOmenaLspServerInvocation();
  const diagnosticsByUri = new Map<string, readonly DiagnosticLike[]>();

  mkdirSync(srcDir, { recursive: true });
  mkdirSync(appThemeDir, { recursive: true });
  mkdirSync(designTokensDir, { recursive: true });
  writeFileSync(
    path.join(workspaceRoot, "package.json"),
    JSON.stringify({ name: "omena-external-sif-runtime-fixture", private: true }) + "\n",
  );
  writeFileSync(resolvedPath, resolvedText);
  writeFileSync(
    path.join(appThemeDir, "package.json"),
    JSON.stringify({
      name: "@app/theme",
      exports: {
        "./index": { sass: "./index.scss" },
      },
    }) + "\n",
  );
  writeFileSync(
    path.join(appThemeDir, "index.scss"),
    '@forward "@design/tokens/colors";\n@forward "./radius";\n',
  );
  writeFileSync(path.join(appThemeDir, "_radius.scss"), "$ds_radius-card: 12px;\n");
  writeFileSync(
    path.join(designTokensDir, "package.json"),
    JSON.stringify({
      name: "@design/tokens",
      exports: {
        "./colors": { sass: "./colors.scss" },
      },
    }) + "\n",
  );
  writeFileSync(path.join(designTokensDir, "colors.scss"), "$ds_gray-700: #374151;\n");

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
    const parsed = params as PublishDiagnosticsParams;
    diagnosticsByUri.set(parsed.uri, parsed.diagnostics ?? []);
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
    const probePromise = collectProbeLatencies(connection);
    openStyleDocument(connection, resolvedUri, resolvedText);

    await waitForDiagnostics(
      () => diagnosticsByUri.get(resolvedUri),
      cleanResolvedDiagnostics,
      "resolved external package Sass chain",
    );
    const probeLatencies = await probePromise;
    assertProbeMetrics(probeLatencies, P95_PROBE_MS, MAX_PROBE_MS, "external SIF refresh");

    const state = await requestWithTimeout(
      connection.sendRequest<DebugStateResponse>(DEBUG_STATE_REQUEST, {}),
      "debug state",
    );
    assert.ok(
      (state.externalSifBridgeGenerationCount ?? 0) > 0,
      "external package runtime must generate bridge SIFs through the LSP path",
    );

    await assertHoverContains(connection, resolvedUri, resolvedText, "$ds_gray-700", [
      "ds_gray-700",
      "#374151",
    ]);
    await assertDefinitionTargets(connection, resolvedUri, resolvedText, "$ds_gray-700", colorsUri);
    await assertDefinitionTargets(
      connection,
      resolvedUri,
      resolvedText,
      "$ds_radius-card",
      radiusUri,
    );
    const steadyProbeLatencies = await collectProbeLatencies(connection);
    assertProbeMetrics(
      steadyProbeLatencies,
      STEADY_P95_PROBE_MS,
      STEADY_MAX_PROBE_MS,
      "external SIF steady state",
    );

    await requestWithTimeout(connection.sendRequest(ShutdownRequest.type), "shutdown");
    connection.sendNotification("exit");
    const exitCode = await waitForExit(child);
    assert.equal(exitCode, 0, `omena-lsp-server exited with ${exitCode}\n${stderr.join("")}`);

    process.stdout.write(
      [
        "omena-lsp-server external SIF runtime ok:",
        `command=${invocation.command}`,
        `probes=${probeLatencies.length}`,
        `p95=${percentile(probeLatencies, 95).toFixed(2)}ms`,
        `max=${Math.max(...probeLatencies).toFixed(2)}ms`,
        `bridgeGenerations=${state.externalSifBridgeGenerationCount ?? 0}`,
        `resolvedDiagnostics=${diagnosticsByUri.get(resolvedUri)?.length ?? 0}`,
        `steadyProbes=${steadyProbeLatencies.length}`,
        `steadyP95=${percentile(steadyProbeLatencies, 95).toFixed(2)}ms`,
        `steadyMax=${Math.max(...steadyProbeLatencies).toFixed(2)}ms`,
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
    workspaceFolders: [{ uri: workspaceUri, name: "omena-external-sif-runtime" }],
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

function openStyleDocument(connection: ProtocolConnection, uri: string, text: string): void {
  connection.sendNotification(DidOpenTextDocumentNotification.type, {
    textDocument: {
      uri,
      languageId: "scss",
      version: 1,
      text,
    },
  });
}

function cleanResolvedDiagnostics(diagnostics: readonly DiagnosticLike[]): boolean {
  const codes = diagnosticCodes(diagnostics);
  return [...BLOCKING_EXTERNAL_CODES].every((code) => !codes.has(code)) && !codes.has("partialExternalSif");
}

function diagnosticCodes(diagnostics: readonly DiagnosticLike[]): Set<string> {
  return new Set(
    diagnostics
      .map((diagnostic) => diagnostic.code)
      .filter((code): code is string => typeof code === "string"),
  );
}

async function waitForDiagnostics(
  readDiagnostics: () => readonly DiagnosticLike[] | undefined,
  predicate: (diagnostics: readonly DiagnosticLike[]) => boolean,
  label: string,
): Promise<void> {
  const deadline = performance.now() + REQUEST_TIMEOUT_MS;
  while (performance.now() < deadline) {
    const diagnostics = readDiagnostics();
    if (diagnostics && predicate(diagnostics)) {
      return;
    }
    // oxlint-disable-next-line eslint/no-await-in-loop
    await sleep(25);
  }
  const diagnostics = readDiagnostics();
  throw new Error(
    [
      `${label} diagnostics did not converge`,
      `codes=${[...diagnosticCodes(diagnostics ?? [])].join(",")}`,
      `messages=${(diagnostics ?? [])
        .map((diagnostic) =>
          typeof diagnostic.message === "string" ? diagnostic.message : JSON.stringify(diagnostic),
        )
        .join(" | ")}`,
    ].join("\n"),
  );
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

function assertProbeMetrics(
  latencies: readonly number[],
  p95BudgetMs: number,
  maxBudgetMs: number,
  label: string,
): void {
  if (latencies.length < 10) {
    throw new Error(`Too few runtime probes completed: ${latencies.length}`);
  }
  const p95 = percentile(latencies, 95);
  const max = Math.max(...latencies);
  if (p95 > p95BudgetMs || max > maxBudgetMs) {
    throw new Error(
      [
        `omena-lsp-server ${label} probe exceeded budget`,
        `p95=${p95.toFixed(2)}ms budget=${p95BudgetMs}ms`,
        `max=${max.toFixed(2)}ms budget=${maxBudgetMs}ms`,
        `samples=${latencies.length}`,
      ].join("\n"),
    );
  }
}

async function assertHoverContains(
  connection: ProtocolConnection,
  uri: string,
  text: string,
  token: string,
  expected: readonly string[],
): Promise<void> {
  const hover = await requestWithTimeout(
    connection.sendRequest("textDocument/hover", {
      textDocument: { uri },
      position: positionForOffset(text, text.indexOf(token) + 1),
    }),
    `hover:${token}`,
  );
  const rendered = JSON.stringify(hover);
  for (const fragment of expected) {
    assert.ok(rendered.includes(fragment), `hover for ${token} must include ${fragment}: ${rendered}`);
  }
}

async function assertDefinitionTargets(
  connection: ProtocolConnection,
  uri: string,
  text: string,
  token: string,
  expectedUri: string,
): Promise<void> {
  const definition = await requestWithTimeout(
    connection.sendRequest("textDocument/definition", {
      textDocument: { uri },
      position: positionForOffset(text, text.indexOf(token) + 1),
    }),
    `definition:${token}`,
  );
  const locations = Array.isArray(definition) ? definition : definition ? [definition] : [];
  assert.ok(locations.length > 0, `definition for ${token} must not be empty`);
  const uris = locations
    .map((location) => (isRecord(location) ? location.uri : undefined))
    .filter((value): value is string => typeof value === "string");
  const expectedCanonicalUri = canonicalFileUriForComparison(expectedUri);
  assert.ok(
    uris.some((candidate) => canonicalFileUriForComparison(candidate) === expectedCanonicalUri),
    `definition for ${token} must target ${expectedUri}, got ${uris.join(",")}`,
  );
}

function canonicalFileUriForComparison(uri: string): string {
  if (!uri.startsWith("file:")) return uri;
  try {
    return pathToFileURL(realpathSync.native(fileURLToPath(uri))).toString();
  } catch {
    return uri;
  }
}

function positionForOffset(text: string, offset: number): { line: number; character: number } {
  assert.ok(offset >= 0, "position token must exist");
  const before = text.slice(0, offset);
  const lines = before.split("\n");
  return {
    line: lines.length - 1,
    character: [...lines.at(-1)!].length,
  };
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

function parsePositiveInteger(value: string | undefined, fallback: number): number {
  if (!value) return fallback;
  const parsed = Number.parseInt(value, 10);
  return Number.isFinite(parsed) && parsed > 0 ? parsed : fallback;
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}

void main().catch((error) => {
  console.error(error);
  process.exit(1);
});
