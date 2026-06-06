import { strict as assert } from "node:assert";
import { spawnSync } from "node:child_process";
import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

interface SoundinessReport {
  readonly product: string;
  readonly suppressedDiagnosticCount: number;
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
  writeFileSync(
    stylePath,
    [
      ".button { color: var(--missing); }",
      '@use "design-system/tokens" as tokens;',
      "/* omena-ignore-next-line missingSassSymbol */",
      ".token { color: tokens.$brand; }",
    ].join("\n"),
  );

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
    ],
    {
      cwd: process.cwd(),
      encoding: "utf8",
      maxBuffer: 1024 * 1024 * 64,
    },
  );

  assert.equal(
    result.status,
    0,
    `omena report soundiness failed\nstdout=${result.stdout}\nstderr=${result.stderr}`,
  );
  const report = JSON.parse(result.stdout) as SoundinessReport;
  assert.equal(report.product, "omena-cli.soundiness-report");
  assert.ok(report.suppressedDiagnosticCount >= 1, "expected suppression accounting");
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

  console.log(
    "validated omena-cli soundiness report: suppression=visible boundary=visible budget=review",
  );
} finally {
  rmSync(workspace, { force: true, recursive: true });
}
