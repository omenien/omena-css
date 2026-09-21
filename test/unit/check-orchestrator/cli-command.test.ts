import { describe, expect, it } from "vitest";
import { pnpmRunCommand } from "../../../packages/check-orchestrator/src/cli/commands";

describe("check orchestrator CLI command helpers", () => {
  it("uses the current pnpm CLI through node when launched from a pnpm script", () => {
    expect(
      pnpmRunCommand("test:protocol", ["--runInBand"], {
        env: { npm_execpath: "/tmp/pnpm.cjs" },
        nodeExecutable: "/usr/local/bin/node",
        platform: "win32",
      }),
    ).toEqual({
      executable: "/usr/local/bin/node",
      args: ["/tmp/pnpm.cjs", "run", "test:protocol", "--", "--runInBand"],
      display: ["pnpm", "run", "test:protocol", "--", "--runInBand"],
    });
  });

  it.each([
    ["darwin", "/opt/homebrew/bin/pnpm"],
    ["linux", "/opt/pnpm/pnpm"],
    ["win32", "C:/Program Files/pnpm/pnpm.exe"],
  ] as const)("executes standalone pnpm directly on %s", (platform, cliPath) => {
    expect(
      pnpmRunCommand("omena-check", ["inventory", "--check"], {
        env: { npm_execpath: cliPath },
        platform,
        nodeExecutable: "/node",
      }),
    ).toEqual({
      executable: cliPath,
      args: ["run", "omena-check", "--", "inventory", "--check"],
      display: ["pnpm", "run", "omena-check", "--", "inventory", "--check"],
    });
  });

  it("keeps Windows command shims on the command-shell fallback", () => {
    expect(
      pnpmRunCommand("test:protocol", [], {
        env: { npm_execpath: "C:/pnpm/pnpm.cmd" },
        platform: "win32",
      }).executable,
    ).toBe("cmd.exe");
  });

  it("falls back to cmd.exe on Windows instead of spawning a .cmd shim directly", () => {
    expect(
      pnpmRunCommand("test:protocol", [], {
        env: { ComSpec: "C:/Windows/System32/cmd.exe" },
        platform: "win32",
      }),
    ).toEqual({
      executable: "C:/Windows/System32/cmd.exe",
      args: ["/d", "/s", "/c", "pnpm", "run", "test:protocol"],
      display: ["pnpm", "run", "test:protocol"],
    });
  });

  it("uses pnpm directly on non-Windows fallback paths", () => {
    expect(pnpmRunCommand("test:protocol", [], { env: {}, platform: "darwin" })).toEqual({
      executable: "pnpm",
      args: ["run", "test:protocol"],
      display: ["pnpm", "run", "test:protocol"],
    });
  });
});
