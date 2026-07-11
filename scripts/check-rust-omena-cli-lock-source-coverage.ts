import { strict as assert } from "node:assert";
import { spawnSync } from "node:child_process";
import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { parseOmenaCliResponse } from "./lib/omena-cli-response";

const workspace = mkdtempSync(join(tmpdir(), "omena-cli-lock-source-coverage-"));

try {
  const sourcePath = join(workspace, "app.module.scss");
  const tokensPath = join(workspace, "tokens.scss");
  const sifPath = join(workspace, "tokens.sif.json");
  const lockfilePath = join(workspace, "omena.lock");

  writeFileSync(
    sourcePath,
    [
      '@use "sass:map";',
      '@use "./local" as local;',
      '@use "design-system/tokens" as tokens;',
      ".button { color: tokens.$brand; }",
    ].join("\n"),
  );
  writeFileSync(tokensPath, "$brand: red !default;");
  writeFileSync(lockfilePath, '{"entries":[],"lockfileVersion":"1"}');

  const missing = runOmena(
    ["lock", "verify", "--frozen", "--lockfile", lockfilePath, "--source", sourcePath, "--json"],
    1,
  );
  assert.match(missing.stdout, /sourceSifMissingFromLock/);
  assert.match(missing.stdout, /design-system\/tokens/);

  runOmena([
    "sif",
    "generate",
    tokensPath,
    "--canonical-url",
    "design-system/tokens",
    "--output",
    sifPath,
  ]);
  runOmena(["lock", "update", "--lockfile", lockfilePath, "--sif", sifPath, "--json"]);

  const covered = runOmena([
    "lock",
    "verify",
    "--frozen",
    "--lockfile",
    lockfilePath,
    "--source",
    sourcePath,
    "--json",
  ]);
  const coveredReport = parseOmenaCliResponse<{ readonly verified: boolean }>(
    covered.stdout,
    "omena-cli.lock-verify",
  );
  assert.equal(coveredReport.verified, true);
  assert.ok(!covered.stdout.includes("sourceSifMissingFromLock"));

  console.log("validated omena-cli lock source coverage: missing=failed covered=passed");
} finally {
  rmSync(workspace, { force: true, recursive: true });
}

function runOmena(args: readonly string[], expectedStatus = 0): { readonly stdout: string } {
  const result = spawnSync(
    "cargo",
    [
      "run",
      "--quiet",
      "--manifest-path",
      "rust/Cargo.toml",
      "-p",
      "omena-cli",
      "--bin",
      "omena",
      "--",
      ...args,
    ],
    {
      cwd: process.cwd(),
      encoding: "utf8",
      maxBuffer: 1024 * 1024 * 64,
    },
  );

  if (result.error) {
    throw result.error;
  }
  assert.equal(
    result.status,
    expectedStatus,
    `omena-cli ${args.join(" ")} exited ${result.status}, expected ${expectedStatus}\nstdout=${result.stdout}\nstderr=${result.stderr}`,
  );
  return { stdout: result.stdout };
}
