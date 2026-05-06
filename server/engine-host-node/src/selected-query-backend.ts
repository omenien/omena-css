import path from "node:path";
import { existsSync } from "node:fs";
import readline from "node:readline";
import { spawn, spawnSync, type ChildProcessWithoutNullStreams } from "node:child_process";
import { buildEngineInputV2 } from "./engine-input-v2";
import type { ProviderDeps } from "../../engine-core-ts/src/provider-deps";

const REPO_ROOT = path.resolve(__dirname, "../../..");
const BUNDLED_EXTENSION_ROOT = path.resolve(__dirname, "../..");
const ENGINE_SHADOW_RUNNER_BINARY =
  process.platform === "win32" ? "engine-shadow-runner.exe" : "engine-shadow-runner";
const ENGINE_SHADOW_RUNNER_MAX_BUFFER_BYTES = 64 * 1024 * 1024;

export type SelectedQueryBackendKind =
  | "typescript-current"
  | "rust-selected-query"
  | "rust-source-resolution"
  | "rust-expression-semantics"
  | "rust-selector-usage";

export interface SelectedQueryBackendDocument {
  readonly uri: string;
  readonly content: string;
  readonly filePath: string;
  readonly version: number;
}

export const SELECTED_QUERY_RUNNER_COMMANDS = {
  sourceResolutionCanonicalProducer: "input-source-resolution-canonical-producer",
  sourceResolutionRuntime: "input-omena-resolver-source-resolution-runtime",
  expressionSemanticsCanonicalProducer: "input-expression-semantics-canonical-producer",
  expressionDomainFlowAnalysis: "input-expression-domain-flow-analysis",
  expressionDomainControlFlowAnalysis: "input-expression-domain-control-flow-analysis",
  expressionDomainIncrementalFlowAnalysis: "input-expression-domain-incremental-flow-analysis",
  expressionDomainSelectorProjection: "input-expression-domain-selector-projection",
  selectorUsageCanonicalProducer: "input-selector-usage-canonical-producer",
  omenaParserStyleFacts: "omena-parser-style-facts",
  styleSemanticGraph: "style-semantic-graph",
  readCascadeAtPosition: "read-cascade-at-position",
  styleSemanticGraphBatch: "style-semantic-graph-batch",
  transformPlan: "transform-plan",
} as const;

export function resolveSelectedQueryBackendKind(
  env: NodeJS.ProcessEnv = process.env,
  fileExists: (filePath: string) => boolean = existsSync,
): SelectedQueryBackendKind {
  const value = env.CME_SELECTED_QUERY_BACKEND?.trim();
  if (value === "auto") {
    return canUsePrebuiltEngineShadowRunner(env, fileExists)
      ? "rust-selected-query"
      : "typescript-current";
  }
  if (!value) {
    return canUsePrebuiltEngineShadowRunner(env, fileExists) &&
      isPackagedExtensionRuntime(env, fileExists)
      ? "rust-selected-query"
      : "typescript-current";
  }

  if (
    value === "typescript-current" ||
    value === "rust-selected-query" ||
    value === "rust-source-resolution" ||
    value === "rust-expression-semantics" ||
    value === "rust-selector-usage"
  ) {
    return value;
  }

  throw new Error(`Unknown selected query backend: ${value}`);
}

export function usesRustSourceResolutionBackend(kind: SelectedQueryBackendKind): boolean {
  return kind === "rust-source-resolution" || kind === "rust-selected-query";
}

export function usesRustExpressionSemanticsBackend(kind: SelectedQueryBackendKind): boolean {
  return kind === "rust-expression-semantics" || kind === "rust-selected-query";
}

export function usesRustSelectorUsageBackend(kind: SelectedQueryBackendKind): boolean {
  return kind === "rust-selector-usage" || kind === "rust-selected-query";
}

export function usesRustStyleSemanticGraphBackend(kind: SelectedQueryBackendKind): boolean {
  return kind === "rust-selected-query";
}

export function buildSelectedQueryBackendInput(
  document: SelectedQueryBackendDocument,
  scssModulePath: string,
  deps: Pick<
    ProviderDeps,
    "analysisCache" | "styleDocumentForPath" | "typeResolver" | "workspaceRoot" | "settings"
  >,
) {
  return buildEngineInputV2({
    workspaceRoot: deps.workspaceRoot,
    classnameTransform: deps.settings.scss.classnameTransform,
    pathAlias: deps.settings.pathAlias,
    sourceDocuments: [document],
    styleFiles: [scssModulePath],
    analysisCache: deps.analysisCache,
    styleDocumentForPath: deps.styleDocumentForPath,
    typeResolver: deps.typeResolver,
  });
}

export interface EngineShadowRunnerInvocation {
  readonly command: string;
  readonly args: readonly string[];
  readonly cwd: string;
}

export type RustSelectedQueryBackendJsonRunnerAsync = <T>(
  command: string,
  input: unknown,
) => Promise<T>;

const ENGINE_SHADOW_RUNNER_CANCELLATION_SIGNALS = new Set<NodeJS.Signals>(["SIGINT", "SIGTERM"]);
const DEFAULT_DAEMON_RESTART_LIMIT = 3;
const DEFAULT_DAEMON_RESTART_WINDOW_MS = 180_000;

export class EngineShadowRunnerCancelledError extends Error {
  readonly signal: NodeJS.Signals;

  constructor(signal: NodeJS.Signals, invocation: EngineShadowRunnerInvocation) {
    super(`engine-shadow-runner cancelled by ${signal}`);
    this.name = "EngineShadowRunnerCancelledError";
    this.signal = signal;
    this.message = [
      `engine-shadow-runner cancelled by ${signal}`,
      `${invocation.command} ${invocation.args.join(" ")}`,
    ].join("\n");
  }
}

export function isEngineShadowRunnerCancelledError(
  err: unknown,
): err is EngineShadowRunnerCancelledError {
  return err instanceof EngineShadowRunnerCancelledError;
}

function resolveProjectRoot(
  env: NodeJS.ProcessEnv = process.env,
  fileExists: (filePath: string) => boolean = existsSync,
): string {
  if (env.CME_PROJECT_ROOT) return path.resolve(env.CME_PROJECT_ROOT);

  for (const candidate of [REPO_ROOT, BUNDLED_EXTENSION_ROOT, process.cwd()]) {
    if (fileExists(path.join(candidate, "package.json"))) return candidate;
  }

  return REPO_ROOT;
}

export function resolveEngineShadowRunnerBinaryPath(env: NodeJS.ProcessEnv = process.env): string {
  return resolveEngineShadowRunnerBinaryPathForEnv(env);
}

export function canUsePrebuiltEngineShadowRunner(
  env: NodeJS.ProcessEnv = process.env,
  fileExists: (filePath: string) => boolean = existsSync,
): boolean {
  return fileExists(resolveEngineShadowRunnerBinaryPathForEnv(env, fileExists));
}

export function isPackagedExtensionRuntime(
  env: NodeJS.ProcessEnv = process.env,
  fileExists: (filePath: string) => boolean = existsSync,
): boolean {
  const projectRoot = resolveProjectRoot(env, fileExists);
  const hasBundledEntrypoints = fileExists(path.join(projectRoot, "dist/client/extension.js"));
  const hasSourceCheckoutMarkers =
    fileExists(path.join(projectRoot, "server/engine-host-node/src/selected-query-backend.ts")) ||
    fileExists(path.join(projectRoot, "rust/Cargo.toml"));

  return hasBundledEntrypoints && !hasSourceCheckoutMarkers;
}

function resolveEngineShadowRunnerBinaryPathForEnv(
  env: NodeJS.ProcessEnv = process.env,
  fileExists: (filePath: string) => boolean = existsSync,
): string {
  if (env.CME_ENGINE_SHADOW_RUNNER_PATH) {
    return path.resolve(env.CME_ENGINE_SHADOW_RUNNER_PATH);
  }

  const projectRoot = resolveProjectRoot(env, fileExists);
  const targetDir = env.CARGO_TARGET_DIR
    ? path.resolve(projectRoot, env.CARGO_TARGET_DIR)
    : path.join(projectRoot, "rust/target");
  const candidates = [
    path.join(
      projectRoot,
      "dist/bin",
      `${process.platform}-${process.arch}`,
      ENGINE_SHADOW_RUNNER_BINARY,
    ),
    path.join(projectRoot, "dist/bin", ENGINE_SHADOW_RUNNER_BINARY),
    path.join(targetDir, "debug", ENGINE_SHADOW_RUNNER_BINARY),
  ];

  return candidates.find(fileExists) ?? candidates[candidates.length - 1]!;
}

export function buildEngineShadowRunnerInvocation(
  command: string,
  env: NodeJS.ProcessEnv = process.env,
  fileExists: (filePath: string) => boolean = existsSync,
): EngineShadowRunnerInvocation {
  return buildEngineShadowRunnerInvocationForArgs([command], env, fileExists);
}

export function buildEngineShadowRunnerDaemonInvocation(
  env: NodeJS.ProcessEnv = process.env,
  fileExists: (filePath: string) => boolean = existsSync,
): EngineShadowRunnerInvocation {
  return buildEngineShadowRunnerInvocationForArgs(["--daemon"], env, fileExists);
}

function buildEngineShadowRunnerInvocationForArgs(
  runnerArgs: readonly string[],
  env: NodeJS.ProcessEnv = process.env,
  fileExists: (filePath: string) => boolean = existsSync,
): EngineShadowRunnerInvocation {
  const projectRoot = resolveProjectRoot(env, fileExists);
  const runnerMode = env.CME_ENGINE_SHADOW_RUNNER?.trim();
  const hasRunnerMode = env.CME_ENGINE_SHADOW_RUNNER !== undefined;
  if (
    runnerMode === "prebuilt" ||
    (!hasRunnerMode && canUsePrebuiltEngineShadowRunner(env, fileExists))
  ) {
    const runnerPath = resolveEngineShadowRunnerBinaryPathForEnv(env, fileExists);
    if (!fileExists(runnerPath)) {
      throw new Error(
        `CME_ENGINE_SHADOW_RUNNER=prebuilt requires ${runnerPath}; run pnpm check:rust-selected-query-warmup first`,
      );
    }
    return {
      command: runnerPath,
      args: runnerArgs,
      cwd: projectRoot,
    };
  }

  return {
    command: "cargo",
    args: [
      "run",
      "--manifest-path",
      path.join(projectRoot, "rust/Cargo.toml"),
      "-p",
      "engine-shadow-runner",
      "--quiet",
      "--",
      ...runnerArgs,
    ],
    cwd: projectRoot,
  };
}

export function runRustSelectedQueryBackendJson<T>(command: string, input: unknown): T {
  const invocation = buildEngineShadowRunnerInvocation(command);
  const child = spawnSync(invocation.command, invocation.args, {
    cwd: invocation.cwd,
    input: JSON.stringify(input),
    encoding: "utf8",
    maxBuffer: ENGINE_SHADOW_RUNNER_MAX_BUFFER_BYTES,
  });

  if (child.signal && ENGINE_SHADOW_RUNNER_CANCELLATION_SIGNALS.has(child.signal)) {
    throw new EngineShadowRunnerCancelledError(child.signal, invocation);
  }

  if (child.status !== 0) {
    throw new Error(
      [
        `engine-shadow-runner exited with code ${child.status ?? "unknown"}`,
        child.signal ? `signal: ${child.signal}` : "",
        child.stderr?.trim(),
      ]
        .filter(Boolean)
        .join("\n"),
    );
  }

  return JSON.parse(child.stdout) as T;
}

export function shouldUseEngineShadowRunnerDaemon(
  env: NodeJS.ProcessEnv = process.env,
  fileExists: (filePath: string) => boolean = existsSync,
): boolean {
  const value = env.CME_ENGINE_SHADOW_RUNNER_DAEMON?.trim().toLowerCase();
  if (value === "1" || value === "true" || value === "on") return true;
  if (value === "0" || value === "false" || value === "off") return false;

  return (
    resolveSelectedQueryBackendKind(env, fileExists) === "rust-selected-query" &&
    isPackagedExtensionRuntime(env, fileExists)
  );
}

export function shouldTraceEngineShadowRunnerDaemon(env: NodeJS.ProcessEnv = process.env): boolean {
  const value = env.CME_ENGINE_SHADOW_RUNNER_DAEMON_TRACE?.trim().toLowerCase();
  return value === "1" || value === "true" || value === "on";
}

export function getEngineShadowRunnerDaemonJsonRunner(
  env: NodeJS.ProcessEnv = process.env,
  fileExists: (filePath: string) => boolean = existsSync,
): RustSelectedQueryBackendJsonRunnerAsync | undefined {
  return shouldUseEngineShadowRunnerDaemon(env, fileExists)
    ? <T>(command: string, input: unknown) =>
        runRustSelectedQueryBackendJsonAsync<T>(command, input, env)
    : undefined;
}

export function runRustSelectedQueryBackendJsonAsync<T>(
  command: string,
  input: unknown,
  env: NodeJS.ProcessEnv = process.env,
): Promise<T> {
  if (!shouldUseEngineShadowRunnerDaemon(env)) {
    return runRustSelectedQueryBackendJsonOnceAsync<T>(command, input, env);
  }
  return getSharedEngineShadowRunnerDaemon(env).runJson<T>(command, input);
}

function runRustSelectedQueryBackendJsonOnceAsync<T>(
  command: string,
  input: unknown,
  env: NodeJS.ProcessEnv,
): Promise<T> {
  const invocation = buildEngineShadowRunnerInvocation(command, env);

  return new Promise<T>((resolve, reject) => {
    const child = spawn(invocation.command, invocation.args, {
      cwd: invocation.cwd,
      stdio: ["pipe", "pipe", "pipe"],
    });
    let stdout = "";
    let stderr = "";
    let settled = false;

    const rejectOnce = (err: unknown) => {
      if (settled) return;
      settled = true;
      reject(err);
    };

    child.stdout.setEncoding("utf8");
    child.stderr.setEncoding("utf8");
    child.stdout.on("data", (chunk: string) => {
      stdout += chunk;
    });
    child.stderr.on("data", (chunk: string) => {
      stderr += chunk;
    });
    child.on("error", rejectOnce);
    child.stdin.on("error", rejectOnce);
    child.on("close", (code, signal) => {
      if (settled) return;
      settled = true;
      if (signal && ENGINE_SHADOW_RUNNER_CANCELLATION_SIGNALS.has(signal)) {
        reject(new EngineShadowRunnerCancelledError(signal, invocation));
        return;
      }
      if (code !== 0) {
        reject(
          new Error(
            [
              `engine-shadow-runner exited with code ${code ?? "unknown"}`,
              signal ? `signal: ${signal}` : "",
              stderr.trim(),
            ]
              .filter(Boolean)
              .join("\n"),
          ),
        );
        return;
      }

      try {
        resolve(JSON.parse(stdout) as T);
      } catch (err) {
        reject(err);
      }
    });
    child.stdin.end(JSON.stringify(input), "utf8");
  });
}

export function shutdownEngineShadowRunnerDaemon(): void {
  sharedEngineShadowRunnerDaemon?.dispose();
  sharedEngineShadowRunnerDaemon = null;
}

interface EngineShadowRunnerDaemonRequestV0 {
  readonly id: string;
  readonly command: string;
  readonly input: unknown;
}

type EngineShadowRunnerDaemonResponseV0 =
  | {
      readonly schemaVersion: "0";
      readonly id: string | null;
      readonly ok: true;
      readonly result: unknown;
    }
  | {
      readonly schemaVersion: "0";
      readonly id: string | null;
      readonly ok: false;
      readonly error: string;
    };

interface PendingDaemonRequest {
  readonly id: string;
  readonly command: string;
  resolve(value: unknown): void;
  reject(err: unknown): void;
}

class EngineShadowRunnerDaemon {
  private child: ChildProcessWithoutNullStreams | null = null;
  private stdoutLines: readline.Interface | null = null;
  private readonly pending = new Map<string, PendingDaemonRequest>();
  private readonly restartTimestamps: number[] = [];
  private stderrTail = "";
  private requestSeq = 0;
  private readonly env: NodeJS.ProcessEnv;

  constructor(env: NodeJS.ProcessEnv = process.env) {
    this.env = env;
  }

  runJson<T>(command: string, input: unknown): Promise<T> {
    this.ensureStarted();
    const child = this.child;
    if (!child) {
      return Promise.reject(new Error("engine-shadow-runner daemon is not available"));
    }

    return new Promise<T>((resolve, reject) => {
      const id = `request:${++this.requestSeq}`;
      const pending: PendingDaemonRequest = {
        id,
        command,
        resolve: (value) => resolve(value as T),
        reject,
      };
      this.pending.set(id, pending);
      const request: EngineShadowRunnerDaemonRequestV0 = {
        id,
        command,
        input,
      };
      child.stdin.write(`${JSON.stringify(request)}\n`, "utf8", (err) => {
        if (!err) return;
        this.rejectPending(pending, err);
      });
    });
  }

  dispose(): void {
    this.stdoutLines?.close();
    this.stdoutLines = null;
    const child = this.child;
    this.child = null;
    if (child && child.exitCode === null && child.signalCode === null) {
      child.kill("SIGTERM");
    }
    this.rejectAll(new Error("engine-shadow-runner daemon disposed"));
  }

  private ensureStarted(): void {
    if (this.child && this.child.exitCode === null && this.child.signalCode === null) return;

    this.assertRestartBudget();
    const invocation = buildEngineShadowRunnerDaemonInvocation(this.env);
    const child = spawn(invocation.command, invocation.args, {
      cwd: invocation.cwd,
      stdio: ["pipe", "pipe", "pipe"],
    });
    child.stdout.setEncoding("utf8");
    child.stderr.setEncoding("utf8");
    this.child = child;
    this.stderrTail = "";

    this.stdoutLines = readline.createInterface({ input: child.stdout });
    this.stdoutLines.on("line", (line) => this.handleLine(line));
    child.stderr.on("data", (chunk: string) => {
      this.stderrTail = `${this.stderrTail}${chunk}`.slice(-8000);
    });
    if (shouldTraceEngineShadowRunnerDaemon(this.env)) {
      process.stderr.write(
        [
          "[css-module-explainer] engine-shadow-runner daemon spawned",
          `pid=${child.pid ?? "unknown"}`,
          `startsInWindow=${this.restartTimestamps.length}`,
        ].join(" ") + "\n",
      );
    }
    child.on("error", (err) => {
      this.child = null;
      this.rejectAll(err);
    });
    child.on("exit", (code, signal) => {
      this.child = null;
      this.stdoutLines?.close();
      this.stdoutLines = null;
      if (signal && ENGINE_SHADOW_RUNNER_CANCELLATION_SIGNALS.has(signal)) {
        this.rejectAll(new EngineShadowRunnerCancelledError(signal, invocation));
        return;
      }
      this.rejectAll(
        new Error(
          [
            `engine-shadow-runner daemon exited with code ${code ?? "unknown"}`,
            signal ? `signal: ${signal}` : "",
            this.stderrTail.trim(),
          ]
            .filter(Boolean)
            .join("\n"),
        ),
      );
    });
  }

  private assertRestartBudget(): void {
    const now = Date.now();
    const windowMs = parsePositiveInteger(
      this.env.CME_ENGINE_SHADOW_RUNNER_DAEMON_RESTART_WINDOW_MS,
      DEFAULT_DAEMON_RESTART_WINDOW_MS,
    );
    const restartLimit = parsePositiveInteger(
      this.env.CME_ENGINE_SHADOW_RUNNER_DAEMON_RESTART_LIMIT,
      DEFAULT_DAEMON_RESTART_LIMIT,
    );

    while (this.restartTimestamps.length > 0 && now - this.restartTimestamps[0]! > windowMs) {
      this.restartTimestamps.shift();
    }

    if (this.restartTimestamps.length >= restartLimit) {
      throw new Error(
        `engine-shadow-runner daemon restart limit exceeded: ${restartLimit} starts within ${windowMs}ms`,
      );
    }

    this.restartTimestamps.push(now);
  }

  private handleLine(line: string): void {
    let response: EngineShadowRunnerDaemonResponseV0;
    try {
      response = JSON.parse(line) as EngineShadowRunnerDaemonResponseV0;
    } catch (err) {
      this.rejectAll(err);
      return;
    }

    if (response.id === null) {
      this.rejectAll(new Error(response.ok ? "daemon response missing id" : response.error));
      return;
    }
    const pending = this.pending.get(response.id);
    if (!pending) return;
    this.pending.delete(response.id);

    if (response.ok) {
      pending.resolve(response.result);
      return;
    }
    pending.reject(new Error(response.error));
  }

  private rejectPending(pending: PendingDaemonRequest, err: unknown): void {
    this.pending.delete(pending.id);
    pending.reject(err);
  }

  private rejectAll(err: unknown): void {
    const pending = Array.from(this.pending.values());
    this.pending.clear();
    for (const request of pending) request.reject(err);
  }
}

let sharedEngineShadowRunnerDaemon: EngineShadowRunnerDaemon | null = null;

function getSharedEngineShadowRunnerDaemon(env: NodeJS.ProcessEnv): EngineShadowRunnerDaemon {
  sharedEngineShadowRunnerDaemon ??= new EngineShadowRunnerDaemon(env);
  return sharedEngineShadowRunnerDaemon;
}

function parsePositiveInteger(value: string | undefined, fallback: number): number {
  if (!value) return fallback;
  const parsed = Number.parseInt(value, 10);
  return Number.isFinite(parsed) && parsed > 0 ? parsed : fallback;
}
