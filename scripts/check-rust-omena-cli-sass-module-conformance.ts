import { strict as assert } from "node:assert";
import { spawnSync } from "node:child_process";

interface SassModuleConformanceRow {
  readonly key: string;
  readonly category: string;
  readonly status: "modeled" | "gap" | "decidedOut" | "policy";
  readonly normativeAnchor: string;
  readonly implementation: string;
  readonly witness: string;
  readonly decision: string;
}

interface SassModuleConformanceReport {
  readonly schemaVersion: string;
  readonly product: string;
  readonly claimLevel: string;
  readonly theoremClaimed: boolean;
  readonly normativeSource: string;
  readonly modeledCount: number;
  readonly gapCount: number;
  readonly decidedOutCount: number;
  readonly policyCount: number;
  readonly rows: readonly SassModuleConformanceRow[];
  readonly readySurfaces: readonly string[];
}

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
    "sass-module-conformance",
    "--json",
  ],
  {
    cwd: process.cwd(),
    encoding: "utf8",
    maxBuffer: 1024 * 1024 * 16,
  },
);

if (result.error) {
  throw result.error;
}

assert.equal(
  result.status,
  0,
  `omena report sass-module-conformance failed\nstdout=${result.stdout}\nstderr=${result.stderr}`,
);

const report = JSON.parse(result.stdout) as SassModuleConformanceReport;
assert.equal(report.schemaVersion, "0");
assert.equal(report.product, "omena-query.sass-module-conformance");
assert.equal(report.claimLevel, "boundedStaticAnalysisCoverageLedger");
assert.equal(report.theoremClaimed, false);
assert.match(report.normativeSource, /sass\/sass/);
assert.equal(report.modeledCount, rowsByStatus("modeled").length);
assert.equal(report.gapCount, rowsByStatus("gap").length);
assert.equal(report.decidedOutCount, rowsByStatus("decidedOut").length);
assert.equal(report.policyCount, rowsByStatus("policy").length);
assert.ok(report.modeledCount >= 6, "expected modeled Sass module semantics rows");
assert.equal(report.gapCount, 1, "the remaining gap row must stay explicit until product witnesses close it");
assert.equal(report.policyCount, 2, "Q3 and Q5 policy rows are mandatory");

for (const surface of [
  "sassModuleConformanceLedger",
  "mandatoryPolicyRows",
  "gapRowsExplicit",
  "noSassRuntimeEquivalenceClaim",
]) {
  assert.ok(report.readySurfaces.includes(surface), `missing ready surface: ${surface}`);
}

for (const key of [
  "useNamespaceVisibility",
  "forwardPrefixShowHide",
  "configurationWithDefaultVariables",
  "forwardedConfigurationPropagation",
  "canonicalModuleInstanceIdentity",
  "reconfigurationConflict",
]) {
  assertRow(key, "modeled");
}

assertRow("importContextInterop", "gap");
assertRow("loadPathRelativeIdentityCoherence", "modeled");
assertRow("metaLoadCssRuntimeConfiguration", "decidedOut");
assertRow("yarnPnpImporterRuntime", "decidedOut");
assertRow("deprecatedSassImportPolicy", "policy");
assertRow("aliasExtractionFallbackPolicy", "policy");

assert.ok(
  report.rows.every(
    (row) => row.normativeAnchor && row.implementation && row.witness && row.decision,
  ),
  "every conformance row must carry normative, implementation, witness, and decision text",
);

console.log(
  [
    "validated omena-cli sass module conformance:",
    `modeled=${report.modeledCount}`,
    `gap=${report.gapCount}`,
    `policy=${report.policyCount}`,
    `decidedOut=${report.decidedOutCount}`,
  ].join(" "),
);

function assertRow(key: string, status: SassModuleConformanceRow["status"]): void {
  const row = report.rows.find((candidate) => candidate.key === key);
  assert.ok(row, `missing Sass module conformance row: ${key}`);
  assert.equal(row.status, status, `unexpected status for ${key}`);
}

function rowsByStatus(status: SassModuleConformanceRow["status"]): SassModuleConformanceRow[] {
  return report.rows.filter((row) => row.status === status);
}
