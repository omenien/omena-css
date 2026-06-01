import { describe, expect, it } from "vitest";
import type { Range, ResolvedType } from "@css-module-explainer/shared";
import type { TypeResolver } from "../../../server/engine-core-ts/src/core/ts/type-resolver";
import {
  buildTsgoAvailabilityInvocation,
  buildTsgoProbeInvocation,
  resolveTsgoBinaryPathForEnv,
  TsgoProbeTypeResolver,
} from "../../../server/engine-host-node/src/tsgo-probe-type-resolver";

const SAMPLE_RANGE: Range = {
  start: { line: 0, character: 0 },
  end: { line: 0, character: 1 },
};

describe("TsgoProbeTypeResolver", () => {
  it("builds a cheap tsgo availability probe for the request path", () => {
    const invocation = buildTsgoAvailabilityInvocation(
      "/workspace",
      {
        OMENA_TSGO_PATH: "/tools/tsgo",
        OMENA_TSGO_CHECKERS: "4",
      },
      () => false,
    );

    expect(invocation).toEqual({
      command: "/tools/tsgo",
      args: ["--version"],
      cwd: "/workspace",
    });
  });

  it("probes once per workspace and delegates to the fallback resolver", () => {
    const probeCalls: string[] = [];
    const resolveCalls: string[] = [];
    const fallbackResolver = createFakeResolver(resolveCalls);

    const resolver = new TsgoProbeTypeResolver({
      fallbackResolver,
      findConfigFile: (workspaceRoot) => `${workspaceRoot}/tsconfig.json`,
      runProbeCommand: (workspaceRoot) => {
        probeCalls.push(workspaceRoot);
        return { status: 0, stdout: "", stderr: "" };
      },
    });

    resolver.resolve("/repo/src/App.tsx", "variant", "/repo", SAMPLE_RANGE);
    resolver.resolve("/repo/src/App.tsx", "variant", "/repo", SAMPLE_RANGE);

    expect(probeCalls).toEqual(["/repo"]);
    expect(resolveCalls).toEqual(["/repo", "/repo"]);
  });

  it("returns unresolvable without an implicit current TypeScript fallback", () => {
    const resolver = new TsgoProbeTypeResolver({
      findConfigFile: (workspaceRoot) => `${workspaceRoot}/tsconfig.json`,
      runProbeCommand: () => ({ status: 0, stdout: "", stderr: "" }),
    });

    expect(resolver.resolve("/repo/src/App.tsx", "variant", "/repo", SAMPLE_RANGE)).toEqual({
      kind: "unresolvable",
      values: [],
    });
  });

  it("re-probes after invalidation", () => {
    const probeCalls: string[] = [];
    const resolver = new TsgoProbeTypeResolver({
      fallbackResolver: createFakeResolver([]),
      findConfigFile: (workspaceRoot) => `${workspaceRoot}/tsconfig.json`,
      runProbeCommand: (workspaceRoot) => {
        probeCalls.push(workspaceRoot);
        return { status: 0, stdout: "", stderr: "" };
      },
    });

    resolver.resolve("/repo/src/App.tsx", "variant", "/repo", SAMPLE_RANGE);
    resolver.invalidate("/repo");
    resolver.resolve("/repo/src/App.tsx", "variant", "/repo", SAMPLE_RANGE);

    expect(probeCalls).toEqual(["/repo", "/repo"]);
  });

  it("returns unresolvable and caches failed tsgo probes", () => {
    const probeCalls: string[] = [];
    const resolveCalls: string[] = [];
    const resolver = new TsgoProbeTypeResolver({
      fallbackResolver: createFakeResolver(resolveCalls),
      findConfigFile: (workspaceRoot) => `${workspaceRoot}/tsconfig.json`,
      runProbeCommand: (workspaceRoot) => {
        probeCalls.push(workspaceRoot);
        return {
          status: 1,
          stdout: "",
          stderr: "tsgo failed",
        };
      },
    });

    expect(resolver.resolve("/repo/src/App.tsx", "variant", "/repo", SAMPLE_RANGE)).toEqual({
      kind: "unresolvable",
      values: [],
    });
    expect(resolver.resolve("/repo/src/App.tsx", "variant", "/repo", SAMPLE_RANGE)).toEqual({
      kind: "unresolvable",
      values: [],
    });
    expect(probeCalls).toEqual(["/repo"]);
    expect(resolveCalls).toEqual([]);
  });

  it("prefers an explicit tsgo binary path", () => {
    const invocation = buildTsgoProbeInvocation(
      "/workspace",
      "/workspace/tsconfig.json",
      {
        OMENA_TSGO_PATH: "/tools/tsgo",
        OMENA_TSGO_CHECKERS: "2",
      },
      () => false,
    );

    expect(invocation).toEqual({
      command: "/tools/tsgo",
      args: ["-p", "/workspace/tsconfig.json", "--pretty", "false", "--noEmit", "--checkers", "2"],
      cwd: "/workspace",
    });
  });

  it("uses packaged extension tsgo before workspace resolution", () => {
    const env = { OMENA_PROJECT_ROOT: "/extension" };
    const packagedPath = resolveTsgoBinaryPathForEnv(env, (filePath) =>
      filePath.includes("package.json"),
    );
    const invocation = buildTsgoProbeInvocation(
      "/workspace",
      "/workspace/tsconfig.json",
      env,
      (filePath) => filePath === packagedPath || filePath.includes("package.json"),
    );

    expect(invocation?.command).toBe(packagedPath);
    expect(invocation?.args).toEqual([
      "-p",
      "/workspace/tsconfig.json",
      "--pretty",
      "false",
      "--noEmit",
    ]);
    expect(invocation?.cwd).toBe("/workspace");
  });

  it("does not fall back to a workspace pnpm tsgo unless explicitly requested", () => {
    const implicitInvocation = buildTsgoProbeInvocation(
      "/workspace",
      "/workspace/tsconfig.json",
      { OMENA_PROJECT_ROOT: "/extension" },
      (filePath) => filePath.includes("package.json"),
    );
    const explicitInvocation = buildTsgoProbeInvocation(
      "/workspace",
      "/workspace/tsconfig.json",
      {
        OMENA_PROJECT_ROOT: "/extension",
        OMENA_TSGO_RESOLUTION: "workspace",
      },
      (filePath) => filePath.includes("package.json"),
    );

    expect(implicitInvocation).toBeNull();
    expect(explicitInvocation).toEqual({
      command: "pnpm",
      args: ["exec", "tsgo", "-p", "/workspace/tsconfig.json", "--pretty", "false", "--noEmit"],
      cwd: "/workspace",
    });
  });
});

function createFakeResolver(calls: string[]): TypeResolver {
  return {
    resolve(_filePath: string, _variableName: string, workspaceRoot: string): ResolvedType {
      calls.push(workspaceRoot);
      return { kind: "unresolvable", values: [] };
    },
    invalidate() {},
    clear() {},
  };
}
