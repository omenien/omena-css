import { strict as assert } from "node:assert";
import { spawnSync } from "node:child_process";
import { readFileSync } from "node:fs";

type DifferenceClass = "equivalent" | "expected" | "unexpected";

interface LinkedEmissionByteDifferentialBaselineV0 {
  readonly schemaVersion: "0";
  readonly product: "omena-bundler.linked-emission-byte-differential-baseline";
  readonly coverageScope: "boundedMultiModuleFixtures" | "fullCorpus";
  readonly fullCorpusCoverage: boolean;
  readonly minimumFixtureCount: number;
  readonly minimumExpectedDivergenceCount: number;
  readonly maximumUnexpectedDivergenceCount: number;
}

interface LinkedEmissionByteDifferentialCaseV0 {
  readonly fixtureId: string;
  readonly moduleCount: number;
  readonly legacyEmissionPath: "importInlineLegacy";
  readonly linkedEmissionPath: "linkedOrder";
  readonly byteEqual: boolean;
  readonly semanticPreserved: boolean;
  readonly authoritativeMarkerOrder: readonly string[];
  readonly legacyMarkerOrder: readonly string[];
  readonly linkedMarkerOrder: readonly string[];
  readonly linkedModulesEmittedOnce: boolean;
  readonly differenceClass: DifferenceClass;
  readonly reasons: readonly string[];
}

interface LinkedEmissionByteDifferentialReportV0 {
  readonly schemaVersion: "0";
  readonly product: "omena-diff-test.linked-emission-byte-differential";
  readonly fixtureCount: number;
  readonly equivalentCount: number;
  readonly expectedDivergenceCount: number;
  readonly unexpectedDivergenceCount: number;
  readonly totalDivergenceCount: number;
  readonly cases: readonly LinkedEmissionByteDifferentialCaseV0[];
}

const baseline = JSON.parse(
  readFileSync("rust/omena-linked-emission-byte-differential-baseline.json", "utf8"),
) as LinkedEmissionByteDifferentialBaselineV0;
assert.equal(baseline.schemaVersion, "0");
assert.equal(baseline.product, "omena-bundler.linked-emission-byte-differential-baseline");
assert.equal(baseline.coverageScope, "boundedMultiModuleFixtures");
assert.equal(baseline.fullCorpusCoverage, false);
assert.ok(baseline.minimumFixtureCount >= 3);
assert.ok(baseline.minimumExpectedDivergenceCount > 0);
assert.ok(baseline.maximumUnexpectedDivergenceCount >= 0);

const forwardedArguments = process.argv
  .slice(2)
  .filter((argument) =>
    ["--inject-unexpected-divergence", "--force-equivalent"].includes(argument),
  );
const run = spawnSync(
  "cargo",
  [
    "run",
    "--manifest-path",
    "rust/Cargo.toml",
    "-p",
    "omena-diff-test",
    "--bin",
    "omena-linked-emission-byte-differential",
    "--quiet",
    "--",
    ...forwardedArguments,
  ],
  { encoding: "utf8" },
);
assert.equal(run.status, 0, [run.stdout, run.stderr].filter(Boolean).join("\n"));
const report = JSON.parse(run.stdout) as LinkedEmissionByteDifferentialReportV0;

assert.equal(report.schemaVersion, "0");
assert.equal(report.product, "omena-diff-test.linked-emission-byte-differential");
assert.equal(report.fixtureCount, report.cases.length);
assert.ok(report.fixtureCount >= baseline.minimumFixtureCount);
assert.equal(
  report.equivalentCount + report.expectedDivergenceCount + report.unexpectedDivergenceCount,
  report.fixtureCount,
);
assert.equal(
  report.expectedDivergenceCount + report.unexpectedDivergenceCount,
  report.totalDivergenceCount,
);
assert.ok(
  report.expectedDivergenceCount >= baseline.minimumExpectedDivergenceCount,
  "the two byte-producing authorities must retain a non-vacuous expected differential",
);
assert.ok(
  report.unexpectedDivergenceCount <= baseline.maximumUnexpectedDivergenceCount,
  `unexpected linked-emission divergences grew from the committed ceiling ${baseline.maximumUnexpectedDivergenceCount} to ${report.unexpectedDivergenceCount}`,
);

const fixtureIds = new Set<string>();
for (const entry of report.cases) {
  assert.ok(!fixtureIds.has(entry.fixtureId), `duplicate fixture id ${entry.fixtureId}`);
  fixtureIds.add(entry.fixtureId);
  assert.ok(entry.moduleCount >= 2, `${entry.fixtureId} is not a multi-module fixture`);
  assert.equal(entry.legacyEmissionPath, "importInlineLegacy");
  assert.equal(entry.linkedEmissionPath, "linkedOrder");
  assert.equal(entry.linkedModulesEmittedOnce, true);
  assert.deepEqual(entry.linkedMarkerOrder, entry.authoritativeMarkerOrder);
  if (entry.differenceClass === "equivalent") {
    assert.equal(entry.byteEqual, true);
  } else if (entry.differenceClass === "expected") {
    assert.equal(entry.byteEqual, false);
    assert.equal(entry.semanticPreserved, true);
    assert.ok(entry.reasons.length > 0, `${entry.fixtureId} has no derived divergence reason`);
  } else {
    assert.equal(entry.byteEqual, false);
  }
}

console.log(JSON.stringify(report, null, 2));
