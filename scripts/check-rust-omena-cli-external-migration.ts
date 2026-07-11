import { strict as assert } from "node:assert";
import { spawnSync } from "node:child_process";
import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

interface StyleDiagnostic {
  readonly code: string;
}

interface StyleDiagnosticsSummary {
  readonly diagnostics: readonly StyleDiagnostic[];
}

const workspace = mkdtempSync(join(tmpdir(), "omena-cli-external-migration-"));

try {
  const appPath = join(workspace, "app.module.scss");
  const tokensPath = join(workspace, "tokens.scss");
  const explicitLockfilePath = join(workspace, "explicit.omena.lock");
  const sifPath = join(workspace, "tokens.sif.json");
  const lockfilePath = join(workspace, "omena.lock");

  writeFileSync(
    appPath,
    '@use "design-system/tokens" as tokens;\n.button { color: tokens.$brand; }',
  );
  writeFileSync(tokensPath, "$brand: red !default;");

  const noLockfile = runStyleDiagnostics([appPath, "--json"]);
  assertDiagnostic(
    noLockfile,
    "unresolvedExternalReference",
    "omitted --external without omena.lock must use Phase 2 SIF discovery",
  );

  const explicitSifWithoutLockfile = runStyleDiagnostics([appPath, "--external", "sif", "--json"]);
  assertDiagnostic(
    explicitSifWithoutLockfile,
    "unresolvedExternalReference",
    "explicit --external sif should opt into boundary diagnostics without omena.lock",
  );

  writeFileSync(explicitLockfilePath, '{"entries":[],"lockfileVersion":"1"}');
  const explicitLockfile = runStyleDiagnostics([
    appPath,
    "--lockfile",
    explicitLockfilePath,
    "--json",
  ]);
  assertDiagnostic(
    explicitLockfile,
    "unresolvedExternalReference",
    "explicit --lockfile should opt into boundary diagnostics even when no omena.lock is discovered",
  );

  writeFileSync(lockfilePath, '{"entries":[],"lockfileVersion":"1"}');

  const missing = runStyleDiagnostics([appPath, "--json"]);
  assertDiagnostic(
    missing,
    "unresolvedExternalReference",
    "lockfile presence should auto-enable external SIF boundary diagnostics",
  );

  const ignored = runStyleDiagnostics([appPath, "--external", "ignored", "--json"]);
  assertNoDiagnostic(
    ignored,
    "unresolvedExternalReference",
    "explicit --external ignored must preserve Phase 0 compatibility",
  );

  writeFileSync(lockfilePath, "{ not json");
  const invalid = runStyleDiagnostics([appPath, "--json"]);
  assertDiagnostic(
    invalid,
    "lockfileInvalid",
    "malformed auto-discovered omena.lock should surface a product diagnostic",
  );
  writeFileSync(lockfilePath, '{"entries":[],"lockfileVersion":"1"}');

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

  const resolved = runStyleDiagnostics([appPath, "--json"]);
  assertNoDiagnostic(
    resolved,
    "unresolvedExternalReference",
    "auto-discovered lockfile SIF should resolve the external boundary",
  );
  assertNoDiagnostic(
    resolved,
    "missingSassSymbol",
    "auto-discovered lockfile SIF should resolve exported Sass symbols",
  );

  const ignoredAfterResolution = runStyleDiagnostics([appPath, "--external", "ignored", "--json"]);
  assertNoExternalBoundaryDiagnostic(
    ignoredAfterResolution,
    "explicit --external ignored must remain reversible after omena.lock is populated",
  );

  console.log(
    "validated omena-cli external migration: phase2-default explicit-sif ignored lockfile-invalid resolved",
  );
} finally {
  rmSync(workspace, { force: true, recursive: true });
}

function runStyleDiagnostics(args: readonly string[]): StyleDiagnosticsSummary {
  return JSON.parse(runOmena(["style-diagnostics", ...args]).stdout) as StyleDiagnosticsSummary;
}

function assertDiagnostic(summary: StyleDiagnosticsSummary, code: string, message: string): void {
  assert.ok(
    summary.diagnostics.some((diagnostic) => diagnostic.code === code),
    `${message}: got ${summary.diagnostics.map((diagnostic) => diagnostic.code).join(",")}`,
  );
}

function assertNoDiagnostic(summary: StyleDiagnosticsSummary, code: string, message: string): void {
  assert.ok(
    summary.diagnostics.every((diagnostic) => diagnostic.code !== code),
    `${message}: got ${summary.diagnostics.map((diagnostic) => diagnostic.code).join(",")}`,
  );
}

function assertNoExternalBoundaryDiagnostic(
  summary: StyleDiagnosticsSummary,
  message: string,
): void {
  for (const code of [
    "missingExternalSif",
    "partialExternalSif",
    "staleExternalSif",
    "unresolvedExternalReference",
    "lockfileInvalid",
  ]) {
    assertNoDiagnostic(summary, code, message);
  }
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
