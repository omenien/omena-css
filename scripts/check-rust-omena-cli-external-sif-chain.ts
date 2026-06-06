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
  readonly readySurfaces: readonly string[];
}

const workspace = mkdtempSync(join(tmpdir(), "omena-cli-external-sif-"));

try {
  const tokensPath = join(workspace, "tokens.scss");
  const indexPath = join(workspace, "index.scss");
  const directAppPath = join(workspace, "direct.module.scss");
  const forwardedAppPath = join(workspace, "forwarded.module.scss");
  const tokensSifPath = join(workspace, "tokens.sif.json");
  const indexSifPath = join(workspace, "index.sif.json");
  const lockfilePath = join(workspace, "omena.lock");

  writeFileSync(tokensPath, "$brand: red !default;");
  writeFileSync(indexPath, '@forward "design-system/tokens";');
  writeFileSync(
    directAppPath,
    '@use "design-system/tokens" as remote;\n.button { color: remote.$brand; }',
  );
  writeFileSync(
    forwardedAppPath,
    '@use "design-system/index" as ds;\n.button { color: ds.$brand; }',
  );

  runOmena([
    "sif",
    "generate",
    tokensPath,
    "--canonical-url",
    "design-system/tokens",
    "--output",
    tokensSifPath,
  ]);
  runOmena([
    "sif",
    "generate",
    indexPath,
    "--canonical-url",
    "design-system/index",
    "--output",
    indexSifPath,
  ]);
  runOmena([
    "lock",
    "update",
    "--lockfile",
    lockfilePath,
    "--sif",
    tokensSifPath,
    "--sif",
    indexSifPath,
    "--json",
  ]);

  const direct = runStyleDiagnostics(directAppPath, lockfilePath);
  assertNoExternalResolutionCodes(direct, "direct bare canonicalUrl SIF");

  const forwarded = runStyleDiagnostics(forwardedAppPath, lockfilePath);
  assertNoExternalResolutionCodes(forwarded, "forwarded SIF export chain");

  console.log(
    [
      "validated omena-cli external SIF chain:",
      `directDiagnostics=${direct.diagnostics.length}`,
      `forwardedDiagnostics=${forwarded.diagnostics.length}`,
      "ready=externalSifBoundaryDiagnostics",
    ].join(" "),
  );
} finally {
  rmSync(workspace, { force: true, recursive: true });
}

function runStyleDiagnostics(stylePath: string, lockfilePath: string): StyleDiagnosticsSummary {
  const args = [
    "style-diagnostics",
    stylePath,
    "--source",
    stylePath,
    "--external",
    "sif",
    "--lockfile",
    lockfilePath,
    "--json",
  ];
  const result = runOmena(args);
  return JSON.parse(result.stdout) as StyleDiagnosticsSummary;
}

function assertNoExternalResolutionCodes(summary: StyleDiagnosticsSummary, label: string): void {
  assert.ok(
    summary.readySurfaces.includes("externalSifBoundaryDiagnostics"),
    `${label} should reach the external SIF diagnostics surface`,
  );
  const codes = summary.diagnostics.map((diagnostic) => diagnostic.code);
  assert.ok(!codes.includes("unresolvedExternalReference"), `${label} should not stay unresolved`);
  assert.ok(!codes.includes("missingExternalSif"), `${label} should find the provided SIF`);
  assert.ok(!codes.includes("missingSassSymbol"), `${label} should resolve forwarded symbols`);
}

function runOmena(args: readonly string[]): { readonly stdout: string } {
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
      "omena-cli",
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
    0,
    `omena-cli ${args.join(" ")} failed\nstdout=${result.stdout}\nstderr=${result.stderr}`,
  );
  return { stdout: result.stdout };
}
