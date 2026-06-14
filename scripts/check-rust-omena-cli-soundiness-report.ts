import { strict as assert } from "node:assert";
import { spawnSync } from "node:child_process";
import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

interface SoundinessReport {
  readonly product: string;
  readonly originalDiagnosticCount: number;
  readonly emittedDiagnosticCount: number;
  readonly suppressedDiagnosticCount: number;
  readonly unusedExpectErrorCount: number;
  readonly diagnosticSuppressionMode: string;
  readonly suppressionReasons: readonly {
    readonly directiveKind: string;
    readonly codes: readonly string[];
    readonly reason: string;
  }[];
  readonly boundaryDiagnostics: {
    readonly unresolvedExternalReference: number;
  };
  readonly noiseBudget: {
    readonly perPrSuppressedDiagnosticRatio: { readonly status: string };
    readonly perFileSuppressedDensity: { readonly status: string };
    readonly projectSuppressionRate: { readonly status: string };
    readonly withinBudget: boolean;
  };
  readonly readySurfaces: readonly string[];
}

const workspace = mkdtempSync(join(tmpdir(), "omena-cli-soundiness-report-"));

try {
  const stylePath = join(workspace, "app.module.scss");
  // The unresolved `@use` surfaces an `unresolvedExternalReference` boundary
  // diagnostic; a genuinely-unresolvable module no longer also yields a
  // speculative per-symbol `missingSassSymbol` (omena-cli #33: only on-disk
  // readable edges get a bridge SIF, unreadable edges surface boundary state).
  // The suppressible `missingSassSymbol` therefore comes from a LOCAL undefined
  // Sass variable (`$brand`), so this fixture still exercises real suppression
  // accounting (RED if suppression regresses) alongside the boundary diagnostic.
  writeFileSync(
    stylePath,
    [
      ".button { color: var(--missing); }",
      '@use "design-system/tokens" as tokens;',
      "/* omena-ignore-next-line missingSassSymbol [reason: 'awaiting upstream SIF'] */",
      ".token { color: $brand; }",
    ].join("\n"),
  );

  const result = runSoundinessReport(stylePath);
  const report = JSON.parse(result.stdout) as SoundinessReport;
  assert.equal(report.product, "omena-cli.soundiness-report");
  assert.ok(report.suppressedDiagnosticCount >= 1, "expected suppression accounting");
  assert.equal(report.diagnosticSuppressionMode, "apply");
  assert.ok(
    report.emittedDiagnosticCount < report.originalDiagnosticCount,
    "default report must apply suppression directives",
  );
  assert.equal(report.unusedExpectErrorCount, 0);
  assert.equal(report.suppressionReasons.length, 1, "expected suppression reason capture");
  assert.equal(report.suppressionReasons[0]?.directiveKind, "ignoreNextLine");
  assert.deepEqual(report.suppressionReasons[0]?.codes, ["missingSassSymbol"]);
  assert.equal(report.suppressionReasons[0]?.reason, "awaiting upstream SIF");
  assert.ok(
    report.boundaryDiagnostics.unresolvedExternalReference >= 1,
    "expected unresolved external boundary visibility",
  );
  assert.equal(report.noiseBudget.perPrSuppressedDiagnosticRatio.status, "review");
  assert.equal(report.noiseBudget.perFileSuppressedDensity.status, "review");
  assert.equal(report.noiseBudget.projectSuppressionRate.status, "review");
  assert.equal(report.noiseBudget.withinBudget, false);
  assert.ok(report.readySurfaces.includes("soundinessReport"));
  assert.ok(report.readySurfaces.includes("noiseBudgetVisibilityGates"));
  assert.ok(report.readySurfaces.includes("diagnosticSuppressionReasonSummary"));

  const noSuppressReport = JSON.parse(
    runSoundinessReport(stylePath, ["--no-suppress"]).stdout,
  ) as SoundinessReport;
  assert.equal(noSuppressReport.diagnosticSuppressionMode, "reportOnly");
  assert.equal(
    noSuppressReport.emittedDiagnosticCount,
    noSuppressReport.originalDiagnosticCount,
    "--no-suppress must leave matched diagnostics visible",
  );
  assert.ok(
    noSuppressReport.suppressedDiagnosticCount >= 1,
    "--no-suppress must still report suppression accounting",
  );

  const budgetFailure = runSoundinessReport(stylePath, ["--max-suppressions", "0"], 1);
  assert.match(
    budgetFailure.stderr,
    /suppression budget exceeded/,
    "max suppression audit flag must fail when the report exceeds the threshold",
  );

  const stalePath = join(workspace, "stale.module.scss");
  writeFileSync(
    stalePath,
    [
      "/* omena-expect-error missingSassSymbol [reason: 'stale fixture'] */",
      ".clean { color: red; }",
    ].join("\n"),
  );
  const staleFailure = runSoundinessReport(stalePath, ["--report-stale-suppressions"], 1);
  assert.match(
    staleFailure.stderr,
    /stale suppressions observed/,
    "stale suppression audit flag must fail on unused omena-expect-error",
  );

  console.log(
    "validated omena-cli soundiness report: suppression=visible boundary=visible budget=review",
  );
} finally {
  rmSync(workspace, { force: true, recursive: true });
}

function runSoundinessReport(
  stylePath: string,
  extraArgs: readonly string[] = [],
  expectedStatus = 0,
): { readonly stdout: string; readonly stderr: string } {
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
      "report",
      "soundiness",
      "--source",
      stylePath,
      "--external",
      "sif",
      "--json",
      ...extraArgs,
    ],
    {
      cwd: process.cwd(),
      encoding: "utf8",
      maxBuffer: 1024 * 1024 * 64,
    },
  );

  assert.equal(
    result.status,
    expectedStatus,
    `omena report soundiness status mismatch\nstdout=${result.stdout}\nstderr=${result.stderr}`,
  );
  return { stdout: result.stdout, stderr: result.stderr };
}
