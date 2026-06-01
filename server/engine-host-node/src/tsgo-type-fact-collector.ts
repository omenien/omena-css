import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { existsSync, readFileSync } from "node:fs";
import ts from "typescript";
import type { ResolvedType } from "@omena/shared";
import {
  createTypeFactTableEntryV1,
  createTypeFactTableEntryV2,
  type TypeFactTableV1,
  type TypeFactTableV2,
} from "../../engine-core-ts/src/contracts";
import type { CollectTypeFactTableV1Options } from "./historical/type-fact-table-v1";
import { resolveTsgoBinaryPathForEnv } from "./tsgo-probe-type-resolver";

const UNRESOLVABLE: ResolvedType = { kind: "unresolvable", values: [] };

export interface TsgoTypeFactTarget {
  readonly filePath: string;
  readonly expressionId: string;
  readonly position: number;
}

export interface TsgoTypeFactWorkerInput {
  readonly workspaceRoot: string;
  readonly configPath: string;
  readonly targets: readonly TsgoTypeFactTarget[];
}

export interface TsgoTypeFactWorkerResultEntry {
  readonly filePath: string;
  readonly expressionId: string;
  readonly resolvedType: ResolvedType;
}

export type RunTsgoTypeFactWorker = (
  input: TsgoTypeFactWorkerInput,
) => readonly TsgoTypeFactWorkerResultEntry[];

export interface TsgoTypeFactResolvedTypesCache {
  get(key: string): Map<string, ResolvedType> | undefined;
  set(key: string, resolvedTypes: Map<string, ResolvedType>): void;
  clear(): void;
}

interface TsgoTypeFactResolvedTypesCacheEntry {
  readonly expiresAt: number;
  readonly resolvedTypes: Map<string, ResolvedType>;
}

export interface TsgoTypeFactWorkerInvocation {
  readonly command: string;
  readonly args: readonly string[];
  readonly cwd: string;
  readonly env: NodeJS.ProcessEnv;
}

export interface CollectTsgoTypeFactsOptions extends CollectTypeFactTableV1Options {
  readonly findConfigFile?: (workspaceRoot: string) => string | null;
  readonly runWorker?: RunTsgoTypeFactWorker;
  readonly workerCache?: TsgoTypeFactResolvedTypesCache;
}

export function collectTypeFactTableV1WithTsgo(
  options: CollectTsgoTypeFactsOptions,
): TypeFactTableV1 {
  let resolvedTypes: Map<string, ResolvedType> | null;
  try {
    resolvedTypes = collectTsgoResolvedTypes(options);
  } catch (error) {
    if (!isRecoverableTsgoWorkerError(error)) {
      throw error;
    }
    resolvedTypes = new Map();
  }
  if (!resolvedTypes) {
    resolvedTypes = new Map();
  }

  return options.sourceEntries
    .flatMap(({ document, analysis }) =>
      analysis.sourceDocument.classExpressions.flatMap((expression) => {
        if (expression.kind !== "symbolRef") return [];
        return [
          createTypeFactTableEntryV1(
            document.filePath,
            expression.id,
            resolvedTypes.get(typeFactKey(document.filePath, expression.id)) ?? UNRESOLVABLE,
          ),
        ];
      }),
    )
    .toSorted(
      (a, b) =>
        a.filePath.localeCompare(b.filePath) || a.expressionId.localeCompare(b.expressionId),
    );
}

export function collectTypeFactTableV2WithTsgo(
  options: CollectTsgoTypeFactsOptions,
): TypeFactTableV2 {
  let resolvedTypes: Map<string, ResolvedType> | null;
  try {
    resolvedTypes = collectTsgoResolvedTypes(options);
  } catch (error) {
    if (!isRecoverableTsgoWorkerError(error)) {
      throw error;
    }
    resolvedTypes = new Map();
  }
  if (!resolvedTypes) {
    resolvedTypes = new Map();
  }

  return options.sourceEntries
    .flatMap(({ document, analysis }) =>
      analysis.sourceDocument.classExpressions.flatMap((expression) => {
        if (expression.kind !== "symbolRef") return [];
        return [
          createTypeFactTableEntryV2(
            document.filePath,
            expression.id,
            resolvedTypes.get(typeFactKey(document.filePath, expression.id)) ?? UNRESOLVABLE,
          ),
        ];
      }),
    )
    .toSorted(
      (a, b) =>
        a.filePath.localeCompare(b.filePath) || a.expressionId.localeCompare(b.expressionId),
    );
}

function collectTsgoResolvedTypes(
  options: CollectTsgoTypeFactsOptions,
): Map<string, ResolvedType> | null {
  const findConfigFile =
    options.findConfigFile ??
    ((workspaceRoot: string) => ts.findConfigFile(workspaceRoot, ts.sys.fileExists) ?? null);
  const configPath = findConfigFile(options.workspaceRoot);
  if (!configPath) {
    return null;
  }

  const targets = options.sourceEntries.flatMap(({ document, analysis }) =>
    analysis.sourceDocument.classExpressions.flatMap((expression) => {
      if (expression.kind !== "symbolRef") return [];
      return [
        {
          filePath: document.filePath,
          expressionId: expression.id,
          position: offsetAtPosition(
            document.content,
            expression.range.start.line,
            expression.range.start.character,
          ),
        } satisfies TsgoTypeFactTarget,
      ];
    }),
  );

  if (targets.length === 0) {
    return new Map();
  }

  const runWorker = options.runWorker ?? defaultRunTsgoTypeFactWorker;
  const workerCache = options.workerCache ?? (options.runWorker ? null : defaultResolvedTypesCache);
  const cacheKey = createTsgoResolvedTypesCacheKey(
    options.workspaceRoot,
    configPath,
    options.sourceEntries,
    targets,
  );
  const cachedResolvedTypes = workerCache?.get(cacheKey);
  if (cachedResolvedTypes) {
    return cachedResolvedTypes;
  }

  const resolved = runWorker({
    workspaceRoot: options.workspaceRoot,
    configPath,
    targets,
  });
  const resolvedTypes = new Map(
    resolved.map((entry) => [typeFactKey(entry.filePath, entry.expressionId), entry.resolvedType]),
  );
  workerCache?.set(cacheKey, resolvedTypes);
  return resolvedTypes;
}

function defaultRunTsgoTypeFactWorker(
  input: TsgoTypeFactWorkerInput,
): readonly TsgoTypeFactWorkerResultEntry[] {
  const invocation = buildTsgoTypeFactWorkerInvocation(input.workspaceRoot);
  const child = spawnSync(invocation.command, invocation.args, {
    cwd: invocation.cwd,
    input: JSON.stringify(input),
    encoding: "utf8",
    stdio: ["pipe", "pipe", "pipe"],
    env: invocation.env,
  });

  if (child.status !== 0) {
    throw new Error(
      [
        "tsgo type fact worker failed",
        child.error ? `error: ${child.error.message}` : null,
        child.stderr.trim() ? `stderr: ${child.stderr.trim()}` : null,
        child.stdout.trim() ? `stdout: ${child.stdout.trim()}` : null,
      ]
        .filter(Boolean)
        .join("\n"),
    );
  }

  return JSON.parse(child.stdout) as readonly TsgoTypeFactWorkerResultEntry[];
}

export function buildTsgoTypeFactWorkerInvocation(
  workspaceRoot: string,
  env: NodeJS.ProcessEnv = process.env,
  fileExists: (filePath: string) => boolean = existsSync,
): TsgoTypeFactWorkerInvocation {
  const workerEnv = { ...env };
  const tsgoPath = resolveTsgoBinaryPathForEnv(workerEnv, fileExists);
  if (fileExists(tsgoPath)) {
    workerEnv.OMENA_TSGO_PATH = tsgoPath;
  }

  return {
    command: process.execPath,
    args: ["-e", TSGO_TYPE_FACT_WORKER_SOURCE],
    cwd: workspaceRoot,
    env: workerEnv,
  };
}

function isTsgoProjectMissError(error: unknown): boolean {
  return error instanceof Error && /\bno project found for file\b/u.test(error.message);
}

function isRecoverableTsgoWorkerError(error: unknown): boolean {
  if (isTsgoProjectMissError(error)) {
    return true;
  }
  if (!(error instanceof Error)) {
    return false;
  }
  return /\b(tsgo type fact worker failed|No extension-owned tsgo binary|ENOENT|spawn)\b/u.test(
    error.message,
  );
}

function offsetAtPosition(text: string, line: number, character: number): number {
  let offset = 0;
  let currentLine = 0;

  while (currentLine < line && offset < text.length) {
    const newline = text.indexOf("\n", offset);
    if (newline < 0) {
      return text.length;
    }
    offset = newline + 1;
    currentLine += 1;
  }

  return offset + character;
}

function typeFactKey(filePath: string, expressionId: string): string {
  return `${filePath}::${expressionId}`;
}

export function createTsgoTypeFactResolvedTypesCache(
  maxEntries = 64,
  maxAgeMs = 1_000,
  now: () => number = Date.now,
): TsgoTypeFactResolvedTypesCache {
  const entries = new Map<string, TsgoTypeFactResolvedTypesCacheEntry>();

  return {
    get(key) {
      const entry = entries.get(key);
      if (!entry) return undefined;
      if (entry.expiresAt <= now()) {
        entries.delete(key);
        return undefined;
      }
      entries.delete(key);
      entries.set(key, entry);
      return cloneResolvedTypes(entry.resolvedTypes);
    },
    set(key, resolvedTypes) {
      entries.delete(key);
      entries.set(key, {
        expiresAt: now() + maxAgeMs,
        resolvedTypes: cloneResolvedTypes(resolvedTypes),
      });
      while (entries.size > maxEntries) {
        const oldestKey = entries.keys().next().value as string | undefined;
        if (oldestKey === undefined) break;
        entries.delete(oldestKey);
      }
    },
    clear() {
      entries.clear();
    },
  };
}

function createTsgoResolvedTypesCacheKey(
  workspaceRoot: string,
  configPath: string,
  sourceEntries: CollectTsgoTypeFactsOptions["sourceEntries"],
  targets: readonly TsgoTypeFactTarget[],
): string {
  const sourceSignature = sourceEntries
    .map(({ document, analysis }) => ({
      filePath: document.filePath,
      version: document.version,
      contentHash: analysis.contentHash,
    }))
    .toSorted((a, b) => a.filePath.localeCompare(b.filePath))
    .map(({ filePath, version, contentHash }) => `${filePath}:${version}:${contentHash}`)
    .join("|");
  const targetSignature = targets
    .map(({ filePath, expressionId, position }) => `${filePath}:${expressionId}:${position}`)
    .toSorted()
    .join("|");

  return JSON.stringify({
    workspaceRoot,
    configPath,
    configHash: readFileContentHash(configPath),
    sources: sourceSignature,
    targets: targetSignature,
    workerEnv: readTsgoTypeFactWorkerEnvSignature(process.env),
  });
}

function readFileContentHash(filePath: string): string {
  try {
    return createHash("sha256").update(readFileSync(filePath)).digest("hex");
  } catch {
    return "unreadable";
  }
}

function readTsgoTypeFactWorkerEnvSignature(env: NodeJS.ProcessEnv): string {
  return JSON.stringify({
    projectRoot: env.OMENA_PROJECT_ROOT ?? "",
    tsgoCheckers: env.OMENA_TSGO_CHECKERS ?? "",
    tsgoPath: env.OMENA_TSGO_PATH ?? "",
  });
}

function cloneResolvedTypes(resolvedTypes: Map<string, ResolvedType>): Map<string, ResolvedType> {
  return new Map(
    [...resolvedTypes.entries()].map(([key, resolvedType]) => [
      key,
      cloneResolvedType(resolvedType),
    ]),
  );
}

const defaultResolvedTypesCache = createTsgoTypeFactResolvedTypesCache();

function cloneResolvedType(resolvedType: ResolvedType): ResolvedType {
  if (resolvedType.kind === "union") {
    return { kind: "union", values: [...resolvedType.values] };
  }
  return UNRESOLVABLE;
}

const TSGO_TYPE_FACT_WORKER_SOURCE = String.raw`
const { spawn } = require("node:child_process");
const { readFileSync } = require("node:fs");
const path = require("node:path");

const TYPE_FLAGS_UNION = 134217728;
const UNRESOLVABLE = { kind: "unresolvable", values: [] };

main().catch((error) => {
  console.error(error);
  process.exit(1);
});

async function main() {
  const input = JSON.parse(readFileSync(0, "utf8"));
  const tsgo = resolveTsgoInvocation(input.workspaceRoot);
  const child = spawn(tsgo.command, tsgo.args, {
    cwd: input.workspaceRoot,
    stdio: ["pipe", "pipe", "pipe"],
    env: process.env,
  });
  const stderr = [];
  child.stderr.setEncoding("utf8");
  child.stderr.on("data", (chunk) => stderr.push(chunk));

  const rpc = createJsonRpcClient(child);
  try {
    await rpc.sendRequest("initialize");
    const snapshotResponse = await rpc.sendRequest("updateSnapshot", {
      openProject: input.configPath,
    });
    const snapshot = snapshotResponse.snapshot;
    const projectByFile = new Map(
      await Promise.all(
        [...new Set(input.targets.map((target) => target.filePath))].map(async (filePath) => {
          const projectResponse = await rpc.sendRequest("getDefaultProjectForFile", {
            snapshot,
            file: filePath,
          });
          return [filePath, projectResponse.id];
        }),
      ),
    );
    const results = await Promise.all(
      input.targets.map(async (target) => {
        const typeResponse = await rpc.sendRequest("getTypeAtPosition", {
          snapshot,
          project: projectByFile.get(target.filePath),
          file: target.filePath,
          position: target.position,
        });
        const resolvedType = await extractResolvedType(rpc, snapshot, typeResponse);
        return {
          filePath: target.filePath,
          expressionId: target.expressionId,
          resolvedType,
        };
      }),
    );
    await rpc.sendRequest("release", { handle: snapshot });
    process.stdout.write(JSON.stringify(results));
  } finally {
    rpc.dispose();
    child.kill("SIGKILL");
    await waitForExit(child);
    if (child.exitCode && stderr.length > 0) {
      process.stderr.write(stderr.join(""));
    }
  }
}

function resolveTsgoInvocation(workspaceRoot) {
  const tsgoArgs = ["--api", "--async", "--cwd", workspaceRoot, ...resolveTsgoCheckerArgs()];
  if (process.env.OMENA_TSGO_PATH) {
    return {
      command: path.resolve(process.env.OMENA_TSGO_PATH),
      args: tsgoArgs,
    };
  }
  return {
    command: "pnpm",
    args: ["exec", "tsgo", ...tsgoArgs],
  };
}

function createJsonRpcClient(child) {
  let nextId = 0;
  let buffer = Buffer.alloc(0);
  const pending = new Map();

  child.stdout.on("data", (chunk) => {
    buffer = Buffer.concat([buffer, Buffer.from(chunk)]);
    drainMessages();
  });
  child.on("error", (error) => rejectAll(error));
  child.on("exit", (code, signal) => {
    rejectAll(new Error([
      "tsgo API process exited",
      "code=" + (code ?? "unknown"),
      signal ? "signal=" + signal : "",
    ].filter(Boolean).join(" ")));
  });

  function sendRequest(method, params) {
    const id = ++nextId;
    const request = { jsonrpc: "2.0", id, method };
    if (params !== undefined) request.params = params;
    const body = Buffer.from(JSON.stringify(request), "utf8");
    child.stdin.write(Buffer.from("Content-Length: " + body.length + "\r\n\r\n", "utf8"));
    child.stdin.write(body);

    return new Promise((resolve, reject) => {
      pending.set(id, { resolve, reject });
    });
  }

  function drainMessages() {
    while (true) {
      const headerEnd = buffer.indexOf("\r\n\r\n");
      if (headerEnd < 0) return;
      const header = buffer.subarray(0, headerEnd).toString("utf8");
      const match = /^Content-Length:\s*(\d+)/imu.exec(header);
      if (!match) {
        rejectAll(new Error("tsgo API response missing Content-Length"));
        return;
      }
      const length = Number(match[1]);
      const bodyStart = headerEnd + 4;
      const bodyEnd = bodyStart + length;
      if (buffer.length < bodyEnd) return;
      const body = buffer.subarray(bodyStart, bodyEnd).toString("utf8");
      buffer = buffer.subarray(bodyEnd);
      handleMessage(JSON.parse(body));
    }
  }

  function handleMessage(message) {
    if (message.id === undefined || message.id === null) return;
    const request = pending.get(message.id);
    if (!request) return;
    pending.delete(message.id);
    if (message.error) {
      request.reject(new Error(message.error.message ?? JSON.stringify(message.error)));
      return;
    }
    request.resolve(message.result);
  }

  function rejectAll(error) {
    for (const request of pending.values()) request.reject(error);
    pending.clear();
  }

  return {
    sendRequest,
    dispose() {
      child.stdin.end();
      rejectAll(new Error("tsgo API client disposed"));
    },
  };
}

async function extractResolvedType(rpc, snapshot, typeResponse) {
  if (typeof typeResponse?.value === "string") {
    return { kind: "union", values: [typeResponse.value] };
  }
  if ((Number(typeResponse?.flags ?? 0) & TYPE_FLAGS_UNION) !== 0) {
    const members = await rpc.sendRequest("getTypesOfType", {
      snapshot,
      type: typeResponse.id,
    });
    const resolvedMembers = await Promise.all(
      (members ?? []).map((member) => extractResolvedType(rpc, snapshot, member)),
    );
    const values = [];
    for (const resolved of resolvedMembers) {
      if (resolved.kind !== "union" || resolved.values.length !== 1) {
        return UNRESOLVABLE;
      }
      values.push(resolved.values[0]);
    }
    const deduped = [...new Set(values)];
    return deduped.length > 0 ? { kind: "union", values: deduped } : UNRESOLVABLE;
  }
  return UNRESOLVABLE;
}

function waitForExit(child) {
  if (child.exitCode !== null || child.signalCode !== null) {
    return Promise.resolve();
  }
  return new Promise((resolve) => {
    const timer = setTimeout(() => resolve(), 1000);
    const done = () => {
      clearTimeout(timer);
      resolve();
    };
    child.once("exit", done);
    child.once("close", done);
  });
}

function resolveTsgoCheckerArgs() {
  const value = process.env.OMENA_TSGO_CHECKERS?.trim();
  return value ? ["--checkers", value] : [];
}
`;
