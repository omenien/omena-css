import { strict as assert } from "node:assert";
import { spawnSync } from "node:child_process";
import { readFileSync, writeFileSync } from "node:fs";

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

interface LinkedEmissionCoverageCensusV0 {
  readonly schemaVersion: "0";
  readonly product: "omena-diff-test.linked-emission-coverage-census";
  readonly coverageScope: "boundedMultiModuleFixtures" | "fullCorpus";
  readonly fullCorpusCoverage: boolean;
  readonly populationCount: number;
  readonly coveredShapeCount: number;
  readonly notCoveredShapeCount: number;
  readonly fixtureCount: number;
  readonly moduleCount: number;
  readonly markerObservableModuleCount: number;
  readonly blindSpotModuleCount: number;
  readonly shapes: ReadonlyArray<{
    readonly shapeClass: string;
    readonly fixtureIds: readonly string[];
  }>;
  readonly notCovered: ReadonlyArray<{
    readonly shapeClass: string;
    readonly reentry: string;
  }>;
  readonly fixtureObservability: ReadonlyArray<{
    readonly fixtureId: string;
    readonly moduleCount: number;
    readonly markerObservableModuleCount: number;
    readonly blindSpotModuleCount: number;
  }>;
  readonly blindSpots: ReadonlyArray<{
    readonly fixtureId: string;
    readonly modulePath: string;
    readonly shapeClasses: readonly string[];
  }>;
}

interface LinkedEmissionByteDifferentialEnvelopeV0 {
  readonly schemaVersion: "0";
  readonly product: "omena-diff-test.linked-emission-byte-differential-envelope";
  readonly report: LinkedEmissionByteDifferentialReportV0;
  readonly census: LinkedEmissionCoverageCensusV0;
}

const censusPath =
  "rust/crates/omena-diff-test/oss-corpus-farm/linked-emission-coverage-census.json";
const baseline = JSON.parse(
  readFileSync("rust/omena-linked-emission-byte-differential-baseline.json", "utf8"),
) as LinkedEmissionByteDifferentialBaselineV0;
assert.equal(baseline.schemaVersion, "0");
assert.equal(baseline.product, "omena-bundler.linked-emission-byte-differential-baseline");
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
const envelope = JSON.parse(run.stdout) as LinkedEmissionByteDifferentialEnvelopeV0;
const { report, census } = envelope;

assert.equal(envelope.schemaVersion, "0");
assert.equal(envelope.product, "omena-diff-test.linked-emission-byte-differential-envelope");
assert.equal(report.schemaVersion, "0");
assert.equal(report.product, "omena-diff-test.linked-emission-byte-differential");
assert.equal(census.schemaVersion, "0");
assert.equal(census.product, "omena-diff-test.linked-emission-coverage-census");
assert.equal(census.fixtureCount, report.fixtureCount);
assert.equal(census.fixtureObservability.length, report.fixtureCount);
assert.equal(census.populationCount, census.shapes.length);
assert.equal(census.coveredShapeCount + census.notCoveredShapeCount, census.populationCount);
assert.equal(census.notCoveredShapeCount, census.notCovered.length);
assert.equal(census.markerObservableModuleCount + census.blindSpotModuleCount, census.moduleCount);
assert.equal(census.blindSpotModuleCount, census.blindSpots.length);
const shapeClasses = census.shapes.map((entry) => entry.shapeClass);
assert.equal(new Set(shapeClasses).size, shapeClasses.length);
assert.ok(census.shapes.every((entry) => entry.shapeClass.length > 0));
assert.ok(
  census.notCovered.every((entry) => entry.shapeClass.length > 0 && entry.reentry.length > 0),
);
const derivedFullCorpusCoverage =
  census.notCovered.length === 0 && census.shapes.every((entry) => entry.fixtureIds.length > 0);
assert.equal(census.fullCorpusCoverage, derivedFullCorpusCoverage);
assert.equal(
  census.coverageScope,
  derivedFullCorpusCoverage ? "fullCorpus" : "boundedMultiModuleFixtures",
);
assert.equal(baseline.coverageScope, census.coverageScope);
assert.equal(baseline.fullCorpusCoverage, census.fullCorpusCoverage);
assert.equal(baseline.minimumFixtureCount, census.fixtureCount);

const serializedCensus = `${JSON.stringify(census, null, 2)}\n`;
if (process.argv.includes("--update-census")) {
  writeFileSync(censusPath, serializedCensus);
}
assert.equal(
  readFileSync(censusPath, "utf8"),
  serializedCensus,
  `linked-emission coverage census drifted; run pnpm update:rust-omena-linked-emission-coverage-census`,
);

assert.equal(report.fixtureCount, report.cases.length);
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
