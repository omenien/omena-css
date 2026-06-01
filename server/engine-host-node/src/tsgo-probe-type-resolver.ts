import { existsSync } from "node:fs";
import path from "node:path";
import { spawnSync, type SpawnSyncReturns } from "node:child_process";
import ts from "typescript";
import type { Range, ResolvedType } from "@css-module-explainer/shared";
import {
  UNRESOLVABLE_TYPE,
  UnresolvableTypeResolver,
  type ResolveTypeOptions,
  type TypeResolver,
} from "../../engine-core-ts/src/core/ts/type-resolver";

interface TsgoProbeResult {
  readonly status: number | null;
  readonly stdout: string;
  readonly stderr: string;
  readonly error?: Error;
}

interface TsgoProbeState {
  readonly configPath: string | null;
  readonly ok: boolean;
}

interface TsgoProbeInvocation {
  readonly command: string;
  readonly args: readonly string[];
  readonly cwd: string;
}

export interface TsgoProbeTypeResolverOptions {
  readonly fallbackResolver?: TypeResolver;
  readonly findConfigFile?: (workspaceRoot: string) => string | null;
  readonly runProbeCommand?: (workspaceRoot: string, configPath: string) => TsgoProbeResult;
}

const REPO_ROOT = path.resolve(__dirname, "../../..");
const BUNDLED_EXTENSION_ROOT = path.resolve(__dirname, "../..");
const TSGO_BINARY_NAME = process.platform === "win32" ? "tsgo.exe" : "tsgo";
const TSGO_WRAPPER_NAME = process.platform === "win32" ? "tsgo.CMD" : "tsgo";

export class TsgoProbeTypeResolver implements TypeResolver {
  private readonly probeStateByWorkspace = new Map<string, TsgoProbeState>();
  private readonly fallbackResolver: TypeResolver;
  private readonly findConfigFile: (workspaceRoot: string) => string | null;
  private readonly runProbeCommand: (workspaceRoot: string, configPath: string) => TsgoProbeResult;

  constructor(options: TsgoProbeTypeResolverOptions = {}) {
    this.fallbackResolver = options.fallbackResolver ?? new UnresolvableTypeResolver();
    this.findConfigFile =
      options.findConfigFile ??
      ((workspaceRoot) => ts.findConfigFile(workspaceRoot, ts.sys.fileExists) ?? null);
    this.runProbeCommand = options.runProbeCommand ?? defaultRunProbeCommand;
  }

  resolve(
    filePath: string,
    variableName: string,
    workspaceRoot: string,
    range: Range,
    options?: ResolveTypeOptions,
  ): ResolvedType {
    if (!this.ensureWorkspaceProbe(workspaceRoot)) {
      return UNRESOLVABLE_TYPE;
    }
    return this.fallbackResolver.resolve(filePath, variableName, workspaceRoot, range, options);
  }

  invalidate(workspaceRoot: string): void {
    this.probeStateByWorkspace.delete(workspaceRoot);
    this.fallbackResolver.invalidate(workspaceRoot);
  }

  clear(): void {
    this.probeStateByWorkspace.clear();
    this.fallbackResolver.clear();
  }

  private ensureWorkspaceProbe(workspaceRoot: string): boolean {
    const cached = this.probeStateByWorkspace.get(workspaceRoot);
    if (cached) {
      return cached.ok;
    }

    const configPath = this.findConfigFile(workspaceRoot);
    if (!configPath) {
      this.probeStateByWorkspace.set(workspaceRoot, {
        configPath: null,
        ok: true,
      });
      return true;
    }

    const result = this.runProbeCommand(workspaceRoot, configPath);
    if (result.status !== 0) {
      this.probeStateByWorkspace.set(workspaceRoot, {
        configPath,
        ok: false,
      });
      return false;
    }

    this.probeStateByWorkspace.set(workspaceRoot, {
      configPath,
      ok: true,
    });
    return true;
  }
}

function defaultRunProbeCommand(workspaceRoot: string, configPath: string): TsgoProbeResult {
  const invocation = shouldRunSyncWorkspaceProbe(process.env)
    ? buildTsgoProbeInvocation(workspaceRoot, configPath)
    : buildTsgoAvailabilityInvocation(workspaceRoot);
  if (!invocation) {
    return {
      status: 1,
      stdout: "",
      stderr: [
        "No extension-owned tsgo binary was found.",
        `Expected ${resolveTsgoBinaryPathForEnv(process.env)} or the repo-pinned @typescript/native-preview wrapper.`,
        "Run pnpm build before packaging, or set OMENA_TSGO_PATH to an explicit tsgo binary.",
      ].join("\n"),
    };
  }

  const child: SpawnSyncReturns<string> = spawnSync(invocation.command, invocation.args, {
    cwd: invocation.cwd,
    encoding: "utf8",
    stdio: "pipe",
    env: process.env,
  });

  return {
    status: child.status,
    stdout: child.stdout ?? "",
    stderr: child.stderr ?? "",
    ...(child.error ? { error: child.error } : {}),
  };
}

export function buildTsgoAvailabilityInvocation(
  workspaceRoot: string,
  env: NodeJS.ProcessEnv = process.env,
  fileExists: (filePath: string) => boolean = existsSync,
): TsgoProbeInvocation | null {
  const tsgoArgs = ["--version"];

  if (env.OMENA_TSGO_PATH) {
    return {
      command: path.resolve(env.OMENA_TSGO_PATH),
      args: tsgoArgs,
      cwd: workspaceRoot,
    };
  }

  const bundledBinaryPath = resolveTsgoBinaryPathForEnv(env, fileExists);
  if (fileExists(bundledBinaryPath)) {
    return {
      command: bundledBinaryPath,
      args: tsgoArgs,
      cwd: workspaceRoot,
    };
  }

  const projectRoot = resolveProjectRoot(env, fileExists);
  const repoPinnedWrapper = path.join(projectRoot, "node_modules", ".bin", TSGO_WRAPPER_NAME);
  if (fileExists(repoPinnedWrapper)) {
    return {
      command: repoPinnedWrapper,
      args: tsgoArgs,
      cwd: workspaceRoot,
    };
  }

  if (env.OMENA_TSGO_RESOLUTION === "workspace") {
    return {
      command: "pnpm",
      args: ["exec", "tsgo", ...tsgoArgs],
      cwd: workspaceRoot,
    };
  }

  return null;
}

export function buildTsgoProbeInvocation(
  workspaceRoot: string,
  configPath: string,
  env: NodeJS.ProcessEnv = process.env,
  fileExists: (filePath: string) => boolean = existsSync,
): TsgoProbeInvocation | null {
  const tsgoArgs = [
    "-p",
    configPath,
    "--pretty",
    "false",
    "--noEmit",
    ...resolveTsgoCheckerArgs(env),
  ];

  if (env.OMENA_TSGO_PATH) {
    return {
      command: path.resolve(env.OMENA_TSGO_PATH),
      args: tsgoArgs,
      cwd: workspaceRoot,
    };
  }

  const bundledBinaryPath = resolveTsgoBinaryPathForEnv(env, fileExists);
  if (fileExists(bundledBinaryPath)) {
    return {
      command: bundledBinaryPath,
      args: tsgoArgs,
      cwd: workspaceRoot,
    };
  }

  const projectRoot = resolveProjectRoot(env, fileExists);
  const repoPinnedWrapper = path.join(projectRoot, "node_modules", ".bin", TSGO_WRAPPER_NAME);
  if (fileExists(repoPinnedWrapper)) {
    return {
      command: repoPinnedWrapper,
      args: tsgoArgs,
      cwd: workspaceRoot,
    };
  }

  if (env.OMENA_TSGO_RESOLUTION === "workspace") {
    return {
      command: "pnpm",
      args: ["exec", "tsgo", ...tsgoArgs],
      cwd: workspaceRoot,
    };
  }

  return null;
}

export function resolveTsgoBinaryPathForEnv(
  env: NodeJS.ProcessEnv = process.env,
  fileExists: (filePath: string) => boolean = existsSync,
): string {
  if (env.OMENA_TSGO_PATH) return path.resolve(env.OMENA_TSGO_PATH);
  const projectRoot = resolveProjectRoot(env, fileExists);
  return path.join(
    projectRoot,
    "dist",
    "bin",
    `${process.platform}-${process.arch}`,
    TSGO_BINARY_NAME,
  );
}

function resolveProjectRoot(
  env: NodeJS.ProcessEnv = process.env,
  fileExists: (filePath: string) => boolean = existsSync,
): string {
  if (env.OMENA_PROJECT_ROOT) return path.resolve(env.OMENA_PROJECT_ROOT);

  for (const candidate of [REPO_ROOT, BUNDLED_EXTENSION_ROOT, process.cwd()]) {
    if (fileExists(path.join(candidate, "package.json"))) return candidate;
  }

  return REPO_ROOT;
}

function resolveTsgoCheckerArgs(env: NodeJS.ProcessEnv = process.env): readonly string[] {
  const value = env.OMENA_TSGO_CHECKERS?.trim();
  if (!value) {
    return [];
  }
  return ["--checkers", value];
}

function shouldRunSyncWorkspaceProbe(env: NodeJS.ProcessEnv): boolean {
  const value = env.OMENA_TSGO_SYNC_WORKSPACE_PROBE?.trim().toLowerCase();
  return value === "1" || value === "true";
}
