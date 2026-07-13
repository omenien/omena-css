import { strict as assert } from "node:assert";
import { spawnSync } from "node:child_process";

export interface DartSassRunResult {
  readonly status: number | null;
  readonly stdout: string;
  readonly stderr: string;
}

export function assertPinnedDartSassVersion(cwd: string): string {
  const result = runPinnedDartSass(["--version"], cwd);
  assert.equal(
    result.status,
    0,
    `pnpm exec sass --version failed\nstdout=${result.stdout}\nstderr=${result.stderr}`,
  );
  const version = result.stdout.trim();
  assert.match(
    version,
    /^1\.101\.0\b/u,
    `dart-sass oracle must resolve to 1.101.0, got ${version}`,
  );
  return version;
}

export function runPinnedDartSass(args: readonly string[], cwd: string): DartSassRunResult {
  const result = spawnSync("pnpm", ["exec", "sass", ...args], {
    cwd,
    encoding: "utf8",
    maxBuffer: 1024 * 1024 * 16,
  });
  return {
    status: result.status,
    stdout: result.stdout,
    stderr: result.stderr,
  };
}
