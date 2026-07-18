import { describe, expect, it } from "vitest";
import { pnpmCliCommand } from "../../../scripts/lib/pnpm-cli";

describe("pnpm CLI command selection", () => {
  it("runs the active pnpm JavaScript entrypoint through Node", () => {
    expect(
      pnpmCliCommand(["exec", "napi", "build"], {
        env: { npm_execpath: "C:/pnpm/pnpm.cjs" },
        nodeExecutable: "C:/node/node.exe",
        platform: "win32",
      }),
    ).toEqual({
      executable: "C:/node/node.exe",
      args: ["C:/pnpm/pnpm.cjs", "exec", "napi", "build"],
    });
  });

  it("uses the command interpreter instead of spawning a Windows shim", () => {
    expect(
      pnpmCliCommand(["exec", "napi", "build"], {
        env: { ComSpec: "C:/Windows/System32/cmd.exe" },
        platform: "win32",
      }),
    ).toEqual({
      executable: "C:/Windows/System32/cmd.exe",
      args: ["/d", "/s", "/c", "pnpm", "exec", "napi", "build"],
    });
  });

  it("runs pnpm directly on non-Windows fallback paths", () => {
    expect(pnpmCliCommand(["exec", "napi", "build"], { env: {}, platform: "linux" })).toEqual({
      executable: "pnpm",
      args: ["exec", "napi", "build"],
    });
  });
});
