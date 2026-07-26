import { strict as assert } from "node:assert";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { readFileSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";

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
  readonly legacySha256: string;
  readonly linkedSha256: string;
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
    readonly emissionPlanEntryCount: number;
    readonly factCategories: readonly string[];
    readonly outputBytesDiffer: boolean;
    readonly markerOrdersAgree: boolean;
    readonly linkedMarkerOrderMatchesAuthority: boolean;
    readonly semanticDifferenceObserved: boolean;
    readonly differenceReasonObserved: boolean;
  }>;
  readonly placementWitnesses: ReadonlyArray<{
    readonly witnessId: string;
    readonly selectorlessModulePaths: readonly string[];
    readonly emissionPlanEntryCount: number;
    readonly outputBytesDiffer: boolean;
    readonly markerOrdersAgree: boolean;
    readonly linkedMarkerOrderMatchesAuthority: boolean;
    readonly semanticDifferenceObserved: boolean;
    readonly differenceReasonObserved: boolean;
    readonly importGraphWinner: string;
    readonly legacyWinner: string;
    readonly linkedWinner: string;
  }>;
}

interface LinkedEmissionByteDifferentialEnvelopeV0 {
  readonly schemaVersion: "0";
  readonly product: "omena-diff-test.linked-emission-byte-differential-envelope";
  readonly report: LinkedEmissionByteDifferentialReportV0;
  readonly census: LinkedEmissionCoverageCensusV0;
}

interface LinkedEmissionOpenDivergenceLedgerV0 {
  readonly schemaVersion: "0";
  readonly product: "omena-diff-test.linked-emission-open-divergence-ledger";
  readonly entries: ReadonlyArray<{
    readonly fixtureId: string;
    readonly shapeClass: string;
    readonly owningGoal: string;
    readonly witnessDigest: string;
    readonly note: string;
  }>;
}

const censusPath =
  "rust/crates/omena-diff-test/oss-corpus-farm/linked-emission-coverage-census.json";
const divergenceLedger = JSON.parse(
  readFileSync(
    "rust/crates/omena-diff-test/oss-corpus-farm/linked-emission-open-divergence-ledger.json",
    "utf8",
  ),
) as LinkedEmissionOpenDivergenceLedgerV0;
const baseline = JSON.parse(
  readFileSync("rust/omena-linked-emission-byte-differential-baseline.json", "utf8"),
) as LinkedEmissionByteDifferentialBaselineV0;
assert.equal(baseline.schemaVersion, "0");
assert.equal(baseline.product, "omena-bundler.linked-emission-byte-differential-baseline");
assert.ok(baseline.minimumFixtureCount >= 3);
assert.ok(baseline.minimumExpectedDivergenceCount > 0);
assert.ok(baseline.maximumUnexpectedDivergenceCount >= 0);
assert.equal(divergenceLedger.schemaVersion, "0");
assert.equal(divergenceLedger.product, "omena-diff-test.linked-emission-open-divergence-ledger");

const linkedEmissionSource = readFileSync(
  "rust/crates/omena-diff-test/src/linked_emission.rs",
  "utf8",
);
const gateSources = [
  {
    path: fileURLToPath(import.meta.url),
    source: readFileSync(fileURLToPath(import.meta.url), "utf8"),
  },
  {
    path: "scripts/check-rust-omena-bundler-linked-emission-default-precondition.ts",
    source: readFileSync(
      "scripts/check-rust-omena-bundler-linked-emission-default-precondition.ts",
      "utf8",
    ),
  },
];
const workspaceWalkApis = [
  ["readdir", "Sync"].join(""),
  ["opendir", "Sync"].join(""),
  ["glob", "Sync"].join(""),
  ["walk", "Dir"].join(""),
];
const hoistCensus = {
  caseProducer: "LinkedEmissionByteDifferentialCaseV0",
  linkedStylesheetProducer: "LinkedStylesheetWithEmissionItemsV0",
  projectionInvocationCount: countMatches(
    linkedEmissionSource,
    /\bproject_omena_transform_bundle_linker_and_emission_items\s*\(/gu,
  ),
  linkerInvocationCount: countMatches(
    linkedEmissionSource,
    /\blink_omena_transform_bundle_projection_with_emission_items_and_resolved_dependencies_and_options\s*\(/gu,
  ),
  cargoSpawnCounts: Object.fromEntries(
    gateSources.map(({ path, source }) => [
      path,
      countMatches(source, /\bspawnSync\(\s*"cargo"/gu),
    ]),
  ),
  workspaceWalkApiCount: gateSources.reduce(
    (count, { source }) =>
      count + workspaceWalkApis.filter((apiName) => source.includes(apiName)).length,
    0,
  ),
};
for (const producerName of [hoistCensus.caseProducer, hoistCensus.linkedStylesheetProducer]) {
  assert.ok(
    linkedEmissionSource.includes(producerName),
    `linked-emission hoist producer is absent: ${producerName}`,
  );
}
assert.equal(
  hoistCensus.projectionInvocationCount,
  1,
  "linked-emission fixtures must share one emission-item projection",
);
assert.equal(
  hoistCensus.linkerInvocationCount,
  1,
  "linked-emission fixtures must share one linker invocation",
);
assert.deepEqual(
  Object.values(hoistCensus.cargoSpawnCounts),
  [1, 1],
  "each linked-emission gate must consume one shared differential cargo run",
);
assert.equal(
  hoistCensus.workspaceWalkApiCount,
  0,
  "linked-emission gates must not introduce a second workspace walk",
);

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
const observedEnvelope = JSON.parse(run.stdout) as LinkedEmissionByteDifferentialEnvelopeV0;
const envelope = process.argv.includes("--inject-missing-justification")
  ? {
      ...observedEnvelope,
      report: {
        ...observedEnvelope.report,
        cases: observedEnvelope.report.cases.map((entry, index) =>
          index === 0 ? { ...entry, reasons: [] } : entry,
        ),
      },
    }
  : observedEnvelope;
const report = envelope.report;
const census = process.argv.includes("--inject-missing-order-census-row")
  ? {
      ...envelope.census,
      blindSpots: envelope.census.blindSpots.slice(1),
    }
  : envelope.census;

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
assert.equal(census.placementWitnesses.length, 4);
assert.ok(
  census.blindSpots.every(
    (blindSpot) =>
      blindSpot.emissionPlanEntryCount > 0 &&
      blindSpot.outputBytesDiffer &&
      blindSpot.markerOrdersAgree &&
      blindSpot.linkedMarkerOrderMatchesAuthority &&
      !blindSpot.semanticDifferenceObserved &&
      blindSpot.differenceReasonObserved,
  ),
);
assert.ok(
  census.placementWitnesses.every(
    (witness) =>
      witness.selectorlessModulePaths.length > 0 &&
      witness.emissionPlanEntryCount > 0 &&
      witness.outputBytesDiffer &&
      witness.markerOrdersAgree &&
      witness.linkedMarkerOrderMatchesAuthority &&
      witness.linkedWinner === witness.importGraphWinner,
  ),
);
assert.deepEqual(
  Object.fromEntries(
    census.placementWitnesses.map((witness) => [
      witness.witnessId,
      {
        importGraphWinner: witness.importGraphWinner,
        legacyWinner: witness.legacyWinner,
        linkedWinner: witness.linkedWinner,
        semanticDifferenceObserved: witness.semanticDifferenceObserved,
        differenceReasonObserved: witness.differenceReasonObserved,
      },
    ]),
  ),
  {
    "element-selector-winner": {
      importGraphWinner: "red",
      legacyWinner: "red",
      linkedWinner: "red",
      semanticDifferenceObserved: false,
      differenceReasonObserved: true,
    },
    "element-selector-after-rule-bearing-module": {
      importGraphWinner: "red",
      legacyWinner: "red",
      linkedWinner: "red",
      semanticDifferenceObserved: false,
      differenceReasonObserved: true,
    },
    "path-name-independence": {
      importGraphWinner: "red",
      legacyWinner: "red",
      linkedWinner: "red",
      semanticDifferenceObserved: false,
      differenceReasonObserved: true,
    },
    "cascade-layer-declaration-order": {
      importGraphWinner: "blue",
      legacyWinner: "blue",
      linkedWinner: "blue",
      semanticDifferenceObserved: false,
      differenceReasonObserved: true,
    },
  },
);
if (process.argv.includes("--require-import-graph-winners")) {
  assert.ok(
    census.placementWitnesses.every(
      (witness) => witness.linkedWinner === witness.importGraphWinner,
    ),
    "linked emission must preserve the import-graph winner for every placement witness",
  );
}
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

const censusFormat = spawnSync("pnpm", ["exec", "oxfmt", `--stdin-filepath=${censusPath}`], {
  encoding: "utf8",
  input: JSON.stringify(census, null, 2),
});
assert.equal(
  censusFormat.status,
  0,
  `linked-emission coverage census could not be formatted: ${censusFormat.stderr}`,
);
const serializedCensus = censusFormat.stdout;
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
  report.unexpectedDivergenceCount <= baseline.maximumUnexpectedDivergenceCount,
  `unexpected linked-emission divergences grew from the committed ceiling ${baseline.maximumUnexpectedDivergenceCount} to ${report.unexpectedDivergenceCount}`,
);
assert.ok(
  report.expectedDivergenceCount >= baseline.minimumExpectedDivergenceCount,
  "the two byte-producing authorities must retain a non-vacuous expected differential",
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

const ledgerFixtureIds = divergenceLedger.entries.map((entry) => entry.fixtureId).toSorted();
const unexpectedFixtureIds = report.cases
  .filter((entry) => entry.differenceClass === "unexpected")
  .map((entry) => entry.fixtureId)
  .toSorted();
assert.equal(new Set(ledgerFixtureIds).size, ledgerFixtureIds.length);
assert.deepEqual(
  ledgerFixtureIds,
  unexpectedFixtureIds,
  "the open-divergence ledger must exactly equal the live unexpected fixture set",
);
assert.equal(baseline.maximumUnexpectedDivergenceCount, divergenceLedger.entries.length);
assert.equal(report.unexpectedDivergenceCount, divergenceLedger.entries.length);

const casesByFixtureId = new Map(report.cases.map((entry) => [entry.fixtureId, entry]));
for (const entry of divergenceLedger.entries) {
  const liveCase = casesByFixtureId.get(entry.fixtureId);
  assert.ok(liveCase, `open-divergence fixture ${entry.fixtureId} is absent from the live corpus`);
  assert.equal(
    entry.witnessDigest,
    divergenceWitnessDigest(entry.fixtureId, liveCase.legacySha256, liveCase.linkedSha256),
    `open-divergence witness digest drifted for ${entry.fixtureId}`,
  );
  assert.ok(entry.owningGoal.length > 0, `${entry.fixtureId} has no owning goal`);
  assert.ok(entry.note.length > 0, `${entry.fixtureId} has no closure note`);
  assert.ok(
    census.blindSpots.some(
      (blindSpot) =>
        blindSpot.fixtureId === entry.fixtureId &&
        blindSpot.shapeClasses.includes(entry.shapeClass),
    ),
    `${entry.fixtureId} is not backed by a live marker-blind shape`,
  );
}
console.log(JSON.stringify(report, null, 2));

function divergenceWitnessDigest(
  fixtureId: string,
  legacySha256: string,
  linkedSha256: string,
): string {
  return createHash("sha256")
    .update(`${fixtureId}\0${legacySha256}\0${linkedSha256}`)
    .digest("hex");
}

function countMatches(source: string, pattern: RegExp): number {
  return [...source.matchAll(pattern)].length;
}
