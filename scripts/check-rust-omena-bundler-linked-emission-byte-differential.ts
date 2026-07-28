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
  readonly minimumInDomainFixtureCount: number;
  readonly expectedDivergenceCount: number;
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
  readonly semanticMismatchCount: number;
  readonly authoritativeMarkerOrder: readonly string[];
  readonly legacyMarkerOrder: readonly string[];
  readonly linkedMarkerOrder: readonly string[];
  readonly authoritativeModuleOrder: readonly string[];
  readonly linkedOutputModuleOrder: readonly string[];
  readonly linkedOutputModuleOrderMatchesAuthority: boolean;
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
  readonly unknownStructuralSelectorCount: number;
  readonly unknownAtRuleCount: number;
  readonly moduleTokenCollisionScope: "boundedFixtureRegressionTripwire";
  readonly moduleTokenCollisionCount: number;
  readonly moduleTokenCollisions: ReadonlyArray<{
    readonly fixtureId: string;
    readonly emittedToken: string;
    readonly modulePaths: readonly string[];
    readonly originalNames: readonly string[];
    readonly observedEmissionPaths: ReadonlyArray<"importInlineLegacy" | "linkedOrder">;
    readonly pathScope: "bothPaths" | "importInlineLegacyOnly" | "linkedOrderOnly";
    readonly reason: string;
  }>;
  readonly ordinalSkewSharedModelCollisionCount: number;
  readonly ordinalSkewPathSplitCollisionCount: number;
  readonly tokenModelByEmissionPath: Readonly<{
    readonly importInlineLegacy: "entryRewriteTable";
    readonly linkedOrder: "moduleLocalRewriteTable";
  }>;
  readonly unmodeledDeclarations: ReadonlyArray<{
    readonly fixtureId: string;
    readonly emissionPath: "importInlineLegacy" | "linkedOrder";
    readonly modulePath: string;
    readonly originalName: string;
    readonly modeledToken: string;
  }>;
  readonly reachabilityInputFixtureIds: readonly string[];
  readonly inDomainFixtureIds: readonly string[];
  readonly liveDeclarationFixtures: ReadonlyArray<{
    readonly fixtureId: string;
    readonly reachabilityReferenceCount: number;
    readonly engineInputStyleSourceCount: number;
    readonly composesResolutionCount: number;
    readonly declarationPreservingPassIds: readonly string[];
    readonly modules: ReadonlyArray<{
      readonly modulePath: string;
      readonly declaredClassNames: readonly string[];
      readonly liveDeclaredClassNames: readonly string[];
      readonly authoredLivenessExpectationCount: number;
      readonly authoredLiveClassNames: readonly string[];
    }>;
  }>;
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

interface LinkedEmissionExpectedDivergenceLedgerV0 {
  readonly schemaVersion: "0";
  readonly product: "omena-diff-test.linked-emission-expected-divergence-ledger";
  readonly entries: ReadonlyArray<{
    readonly fixtureId: string;
    readonly classificationArm:
      | "semanticPreservingKnownDifference"
      | "moduleAttributedReachabilityCorrection";
    readonly derivedReasons: readonly string[];
    readonly witnessDigest: string;
    readonly justification: string;
  }>;
}

interface LinkedEmissionExpectedCollisionLedgerV0 {
  readonly schemaVersion: "0";
  readonly product: "omena-diff-test.linked-emission-expected-collisions";
  readonly entries: LinkedEmissionCoverageCensusV0["moduleTokenCollisions"];
}

const censusPath =
  "rust/crates/omena-diff-test/oss-corpus-farm/linked-emission-coverage-census.json";
const expectedDivergenceLedger = JSON.parse(
  readFileSync(
    "rust/crates/omena-diff-test/oss-corpus-farm/linked-emission-expected-divergence-ledger.json",
    "utf8",
  ),
) as LinkedEmissionExpectedDivergenceLedgerV0;
const expectedCollisionLedger = JSON.parse(
  readFileSync(
    "rust/crates/omena-diff-test/oss-corpus-farm/linked-emission-expected-collisions.json",
    "utf8",
  ),
) as LinkedEmissionExpectedCollisionLedgerV0;
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
// FALSIFIER: id=linked-emission-gate-001 class=accounting via=--inject-unexpected-divergence producer=can-fail owner=linked-emission-instrument entry=committed-corpus-green
assert.ok(baseline.minimumFixtureCount >= 3);
// FALSIFIER: id=linked-emission-baseline-domain-floor class=shaking via=--inject-live-declaration-loss producer=can-fail owner=linked-emission-instrument entry=minimum-two-in-domain-fixtures
assert.ok(baseline.minimumInDomainFixtureCount >= 2);
// FALSIFIER: id=linked-emission-gate-002 class=accounting via=--inject-unexpected-divergence producer=can-fail owner=linked-emission-instrument entry=committed-corpus-green
assert.ok(baseline.expectedDivergenceCount > 0);
// FALSIFIER: id=linked-emission-gate-003 class=accounting via=--inject-unexpected-divergence producer=can-fail owner=linked-emission-instrument entry=committed-corpus-green
assert.ok(baseline.maximumUnexpectedDivergenceCount >= 0);
assert.equal(expectedDivergenceLedger.schemaVersion, "0");
assert.equal(
  expectedDivergenceLedger.product,
  "omena-diff-test.linked-emission-expected-divergence-ledger",
);
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
  // FALSIFIER: id=linked-emission-gate-004 class=accounting via=--inject-unexpected-divergence producer=can-fail owner=linked-emission-instrument entry=committed-corpus-green
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
// FALSIFIER: id=linked-emission-gate-005 class=accounting via=--inject-unexpected-divergence producer=can-fail owner=linked-emission-instrument entry=committed-corpus-green
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
    [
      "--inject-unexpected-divergence",
      "--force-equivalent",
      "--inject-cross-module-declaration-loss",
      "--inject-composed-declaration-loss",
      "--inject-live-declaration-loss",
      "--inject-unclaimed-linked-token",
      "--inject-composes-liveness-loss",
      "--inject-unattributed-reference",
      "--inject-authored-liveness-flip",
      "--inject-missing-fixture",
      "--inject-linked-rule-misattribution",
    ].includes(argument),
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
const censusWithOrderInjection = process.argv.includes("--inject-missing-order-census-row")
  ? {
      ...envelope.census,
      blindSpots: envelope.census.blindSpots.slice(1),
    }
  : envelope.census;
const censusWithDomainInjection = process.argv.includes("--inject-empty-reachability-domain")
  ? {
      ...censusWithOrderInjection,
      reachabilityInputFixtureIds: [
        ...censusWithOrderInjection.reachabilityInputFixtureIds,
        "empty-reachability-input",
      ],
    }
  : censusWithOrderInjection;
const census = process.argv.includes("--inject-missing-shape-row")
  ? {
      ...censusWithDomainInjection,
      shapes: censusWithDomainInjection.shapes.slice(1),
    }
  : censusWithDomainInjection;
const authoredCollisions = process.argv.includes("--inject-collision-expectation-flip")
  ? expectedCollisionLedger.entries.map((entry, index) =>
      index === 0 ? { ...entry, emittedToken: `${entry.emittedToken}-flipped` } : entry,
    )
  : process.argv.includes("--inject-collision-scope-flip")
    ? expectedCollisionLedger.entries.map((entry, index) =>
        index === 0 ? { ...entry, pathScope: "bothPaths" as const } : entry,
      )
    : expectedCollisionLedger.entries;

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
assert.equal(
  census.unknownStructuralSelectorCount,
  0,
  "the bounded linked-emission corpus projected an unknown structural selector",
);
assert.equal(
  census.unknownAtRuleCount,
  0,
  "the bounded linked-emission corpus projected an unknown at-rule",
);
assert.equal(census.moduleTokenCollisionCount, census.moduleTokenCollisions.length);
assert.equal(census.moduleTokenCollisionScope, "boundedFixtureRegressionTripwire");
assert.equal(census.ordinalSkewSharedModelCollisionCount, 0);
assert.equal(census.ordinalSkewPathSplitCollisionCount, 1);
// FALSIFIER: id=linked-emission-token-model-contract class=accounting via=--inject-collision-expectation-flip producer=can-fail owner=linked-emission-instrument entry=path-specific-models-recorded
assert.deepEqual(census.tokenModelByEmissionPath, {
  importInlineLegacy: "entryRewriteTable",
  linkedOrder: "moduleLocalRewriteTable",
});
// FALSIFIER: id=linked-emission-unmodeled-contract class=accounting via=--inject-linked-rule-misattribution producer=can-fail owner=linked-emission-instrument entry=no-unmodeled-declarations
assert.deepEqual(census.unmodeledDeclarations, []);
// FALSIFIER: id=linked-emission-domain-fixture-set class=shaking via=--inject-live-declaration-loss producer=can-fail owner=linked-emission-instrument entry=three-reachability-fixtures
assert.deepEqual(
  census.inDomainFixtureIds,
  census.reachabilityInputFixtureIds,
  "the declaration-retention arm must cover every fixture carrying reachability input",
);
// FALSIFIER: id=linked-emission-domain-floor class=shaking via=--inject-live-declaration-loss producer=can-fail owner=linked-emission-instrument entry=committed-in-domain-floor
assert.ok(
  census.inDomainFixtureIds.length >= baseline.minimumInDomainFixtureCount,
  `the declaration-retention arm domain shrank below its committed floor ${baseline.minimumInDomainFixtureCount}`,
);
// FALSIFIER: id=linked-emission-domain-census class=shaking via=--inject-live-declaration-loss producer=can-fail owner=linked-emission-instrument entry=one-census-row-per-domain-fixture
assert.deepEqual(
  census.liveDeclarationFixtures.map((fixture) => fixture.fixtureId),
  census.inDomainFixtureIds,
  "the declaration-retention census must publish every in-domain fixture",
);
for (const fixture of census.liveDeclarationFixtures) {
  // FALSIFIER: id=linked-emission-source-precondition class=shaking via=--inject-live-declaration-loss producer=can-fail owner=linked-emission-instrument entry=nonempty-engine-style-sources
  assert.ok(
    fixture.reachabilityReferenceCount > 0 && fixture.engineInputStyleSourceCount > 0,
    `${fixture.fixtureId} does not carry the source text required by liveness derivation`,
  );
  // FALSIFIER: id=linked-emission-pass-precondition class=shaking via=--inject-live-declaration-loss producer=can-fail owner=linked-emission-instrument entry=declaration-preserving-pass-set
  assert.deepEqual(
    fixture.declarationPreservingPassIds,
    ["import-inline", "print-css", "css-modules-class-hashing", "tree-shake-class"],
    `${fixture.fixtureId} does not use the declaration-preserving pass set`,
  );
  for (const module of fixture.modules) {
    assert.equal(
      module.authoredLivenessExpectationCount,
      module.declaredClassNames.length,
      `${fixture.fixtureId}:${module.modulePath} does not enumerate every declared class`,
    );
    // FALSIFIER: id=linked-emission-authored-live-set-equality class=liveness via=--inject-authored-liveness-flip producer=can-fail owner=linked-emission-instrument entry=authored-and-derived-live-sets-equal
    assert.deepEqual(
      module.authoredLiveClassNames.toSorted(),
      module.liveDeclaredClassNames.toSorted(),
      `${fixture.fixtureId}:${module.modulePath} authored and derived liveness differ`,
    );
  }
}
const composesDomainFixture = census.liveDeclarationFixtures.find(
  (fixture) => fixture.fixtureId === "module-qualified-composes-reachability",
);
// FALSIFIER: id=linked-emission-composes-source-precondition class=liveness via=--inject-composed-declaration-loss producer=can-fail owner=linked-emission-instrument entry=composes-resolution-observed
assert.ok(
  composesDomainFixture && composesDomainFixture.composesResolutionCount > 0,
  "the composes fixture must derive at least one source-backed resolution",
);
const liveDeclarationArmStart = linkedEmissionSource.indexOf(
  "fn validate_live_declared_names_survive_linked_emission_v0",
);
const liveDeclarationArmEnd = linkedEmissionSource.indexOf(
  "\nfn live_declared_names_by_module_v0",
  liveDeclarationArmStart,
);
const liveDeclarationArmSource =
  liveDeclarationArmStart >= 0 && liveDeclarationArmEnd > liveDeclarationArmStart
    ? linkedEmissionSource.slice(liveDeclarationArmStart, liveDeclarationArmEnd)
    : undefined;
// FALSIFIER: id=linked-emission-arm-source-present class=shaking via=--inject-live-declaration-loss producer=can-fail owner=linked-emission-instrument entry=arm-symbol-present
assert.ok(liveDeclarationArmSource, "the linked declaration-retention arm is absent");
// FALSIFIER: id=linked-emission-arm-no-default-oracle class=shaking via=--inject-live-declaration-loss producer=can-fail owner=linked-emission-instrument entry=linked-output-only
assert.ok(
  !liveDeclarationArmSource.includes("legacy_css") &&
    !liveDeclarationArmSource.includes("shape_classes"),
  "the linked declaration-retention arm must not read legacy bytes or shape-class literals",
);
// FALSIFIER: id=linked-emission-gate-006 class=accounting via=--inject-unexpected-divergence producer=can-fail owner=linked-emission-instrument entry=committed-corpus-green
assert.ok(
  census.moduleTokenCollisionCount > 0,
  "the bounded corpus must retain a cross-module emitted-token collision witness",
);
assert.equal(expectedCollisionLedger.schemaVersion, "0");
assert.equal(
  expectedCollisionLedger.product,
  "omena-diff-test.linked-emission-expected-collisions",
);
// FALSIFIER: id=linked-emission-collision-authored-equality class=accounting via=--inject-collision-expectation-flip producer=can-fail owner=linked-emission-instrument entry=committed-corpus-green
assert.deepEqual(
  census.moduleTokenCollisions.map(
    ({ observedEmissionPaths: _observedEmissionPaths, ...collision }) => collision,
  ),
  authoredCollisions,
  "live emitted-token collisions must equal the hand-authored collision contract",
);
assert.equal(census.placementWitnesses.length, 4);
const moduleBoundaryShapeClasses = new Set(["empty-module", "comment-only-module"]);
const moduleBoundaryBlindSpots = census.blindSpots.filter((blindSpot) =>
  blindSpot.shapeClasses.some((shapeClass) => moduleBoundaryShapeClasses.has(shapeClass)),
);
// FALSIFIER: id=linked-emission-gate-012 class=accounting via=--inject-unexpected-divergence producer=can-fail owner=linked-emission-instrument entry=committed-corpus-green
assert.deepEqual(moduleBoundaryBlindSpots.map((blindSpot) => blindSpot.fixtureId).toSorted(), [
  "comment-only-module-boundary",
  "empty-module-boundary",
]);
// FALSIFIER: id=linked-emission-gate-013 class=accounting via=--inject-unexpected-divergence producer=can-fail owner=linked-emission-instrument entry=committed-corpus-green
assert.ok(
  moduleBoundaryBlindSpots.every(
    (blindSpot) =>
      blindSpot.emissionPlanEntryCount === 1 &&
      blindSpot.factCategories.length === 0 &&
      blindSpot.markerOrdersAgree &&
      blindSpot.linkedMarkerOrderMatchesAuthority &&
      !blindSpot.semanticDifferenceObserved,
  ),
);
// FALSIFIER: id=linked-emission-gate-014 class=accounting via=--inject-unexpected-divergence producer=can-fail owner=linked-emission-instrument entry=committed-corpus-green
assert.ok(
  census.blindSpots
    .filter(
      (blindSpot) =>
        !blindSpot.shapeClasses.some((shapeClass) => moduleBoundaryShapeClasses.has(shapeClass)),
    )
    .every(
      (blindSpot) =>
        blindSpot.emissionPlanEntryCount > 0 &&
        blindSpot.outputBytesDiffer &&
        blindSpot.markerOrdersAgree &&
        blindSpot.linkedMarkerOrderMatchesAuthority &&
        !blindSpot.semanticDifferenceObserved &&
        blindSpot.differenceReasonObserved,
    ),
);
// FALSIFIER: id=linked-emission-gate-015 class=accounting via=--inject-unexpected-divergence producer=can-fail owner=linked-emission-instrument entry=committed-corpus-green
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
// FALSIFIER: id=linked-emission-gate-016 class=accounting via=--inject-unexpected-divergence producer=can-fail owner=linked-emission-instrument entry=committed-corpus-green
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
  // FALSIFIER: id=linked-emission-gate-017 class=accounting via=--inject-unexpected-divergence producer=can-fail owner=linked-emission-instrument entry=committed-corpus-green
  assert.ok(
    census.placementWitnesses.every(
      (witness) => witness.linkedWinner === witness.importGraphWinner,
    ),
    "linked emission must preserve the import-graph winner for every placement witness",
  );
}
const shapeClasses = census.shapes.map((entry) => entry.shapeClass);
assert.equal(new Set(shapeClasses).size, shapeClasses.length);
// FALSIFIER: id=linked-emission-gate-018 class=accounting via=--inject-unexpected-divergence producer=can-fail owner=linked-emission-instrument entry=committed-corpus-green
assert.ok(census.shapes.every((entry) => entry.shapeClass.length > 0));
// FALSIFIER: id=linked-emission-gate-019 class=accounting via=--inject-unexpected-divergence producer=can-fail owner=linked-emission-instrument entry=committed-corpus-green
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
// FALSIFIER: id=linked-emission-gate-020 class=accounting via=--inject-unexpected-divergence producer=can-fail owner=linked-emission-instrument entry=committed-corpus-green
assert.ok(
  report.unexpectedDivergenceCount <= baseline.maximumUnexpectedDivergenceCount,
  `unexpected linked-emission divergences grew from the committed ceiling ${baseline.maximumUnexpectedDivergenceCount} to ${report.unexpectedDivergenceCount}`,
);
assert.equal(
  report.expectedDivergenceCount,
  baseline.expectedDivergenceCount,
  "the enumerated expected linked-emission divergence count drifted",
);

const fixtureIds = new Set<string>();
for (const entry of report.cases) {
  // FALSIFIER: id=linked-emission-gate-021 class=accounting via=--inject-unexpected-divergence producer=can-fail owner=linked-emission-instrument entry=committed-corpus-green
  assert.ok(!fixtureIds.has(entry.fixtureId), `duplicate fixture id ${entry.fixtureId}`);
  fixtureIds.add(entry.fixtureId);
  // FALSIFIER: id=linked-emission-gate-022 class=accounting via=--inject-unexpected-divergence producer=can-fail owner=linked-emission-instrument entry=committed-corpus-green
  assert.ok(entry.moduleCount >= 2, `${entry.fixtureId} is not a multi-module fixture`);
  assert.equal(entry.legacyEmissionPath, "importInlineLegacy");
  assert.equal(entry.linkedEmissionPath, "linkedOrder");
  assert.equal(entry.linkedModulesEmittedOnce, true);
  // FALSIFIER: id=linked-emission-gate-023 class=accounting via=--inject-unexpected-divergence producer=can-fail owner=linked-emission-instrument entry=committed-corpus-green
  assert.deepEqual(entry.linkedMarkerOrder, entry.authoritativeMarkerOrder);
  assert.equal(entry.linkedOutputModuleOrderMatchesAuthority, true);
  // FALSIFIER: id=linked-emission-gate-024 class=accounting via=--inject-unexpected-divergence producer=can-fail owner=linked-emission-instrument entry=committed-corpus-green
  assert.deepEqual(entry.linkedOutputModuleOrder, entry.authoritativeModuleOrder);
  if (entry.differenceClass === "equivalent") {
    assert.equal(entry.byteEqual, true);
  } else if (entry.differenceClass === "expected") {
    assert.equal(entry.byteEqual, false);
    if (entry.fixtureId === "module-qualified-reachability") {
      assert.equal(entry.semanticPreserved, false);
      assert.equal(entry.semanticMismatchCount, 1);
      // FALSIFIER: id=linked-emission-gate-025 class=accounting via=--inject-unexpected-divergence producer=can-fail owner=linked-emission-instrument entry=committed-corpus-green
      assert.deepEqual(entry.legacyMarkerOrder, ["_entry-marker_1"]);
      // FALSIFIER: id=linked-emission-gate-026 class=accounting via=--inject-unexpected-divergence producer=can-fail owner=linked-emission-instrument entry=committed-corpus-green
      assert.deepEqual(entry.linkedMarkerOrder, ["_dependency-own_1", "_entry-marker_1"]);
      // FALSIFIER: id=linked-emission-gate-027 class=accounting via=--inject-unexpected-divergence producer=can-fail owner=linked-emission-instrument entry=committed-corpus-green
      assert.deepEqual(entry.authoritativeMarkerOrder, ["_dependency-own_1", "_entry-marker_1"]);
    } else if (entry.fixtureId === "module-qualified-composes-reachability") {
      // FALSIFIER: id=linked-emission-gate-028 class=accounting via=--inject-unexpected-divergence producer=can-fail owner=linked-emission-instrument entry=committed-corpus-green
      assert.ok(entry.legacyMarkerOrder.includes("_card_0"));
      // FALSIFIER: id=linked-emission-gate-029 class=accounting via=--inject-unexpected-divergence producer=can-fail owner=linked-emission-instrument entry=committed-corpus-green
      assert.ok(entry.linkedMarkerOrder.includes("_base_0"));
      // FALSIFIER: id=linked-emission-gate-030 class=accounting via=--inject-unexpected-divergence producer=can-fail owner=linked-emission-instrument entry=committed-corpus-green
      assert.ok(entry.linkedMarkerOrder.includes("_base-live_1"));
    } else if (entry.fixtureId === "module-qualified-at-rule-reachability") {
      assert.equal(entry.semanticPreserved, false);
      assert.equal(entry.semanticMismatchCount, 6);
      // FALSIFIER: id=linked-emission-gate-at-rule-legacy-markers class=accounting via=--inject-unexpected-divergence producer=can-fail owner=linked-emission-instrument entry=legacy-at-rule-marker-order
      assert.deepEqual(entry.legacyMarkerOrder, ["_card_0"]);
      // FALSIFIER: id=linked-emission-gate-at-rule-linked-markers class=accounting via=--inject-unexpected-divergence producer=can-fail owner=linked-emission-instrument entry=linked-at-rule-marker-order
      assert.deepEqual(entry.linkedMarkerOrder, ["_base_0", "_media-live_2", "_card_0"]);
      // FALSIFIER: id=linked-emission-gate-at-rule-authority-markers class=placement via=--inject-unexpected-divergence producer=can-fail owner=linked-emission-instrument entry=authoritative-at-rule-marker-order
      assert.deepEqual(entry.authoritativeMarkerOrder, ["_base_0", "_media-live_2", "_card_0"]);
    } else if (entry.fixtureId === "entry-ordinal-skew") {
      // The two paths intentionally apply different documented rewrite scopes.
      assert.equal(entry.semanticPreserved, false);
      // FALSIFIER: id=linked-emission-skew-semantic-delta class=accounting via=--inject-collision-expectation-flip producer=can-fail owner=linked-emission-instrument entry=path-model-difference-nonempty
      assert.ok(entry.semanticMismatchCount > 0);
    } else if (
      census.liveDeclarationFixtures.some((fixture) => fixture.fixtureId === entry.fixtureId)
    ) {
      assert.equal(entry.semanticPreserved, false);
      // FALSIFIER: id=linked-emission-authored-case-delta class=liveness via=--inject-authored-liveness-flip producer=can-fail owner=linked-emission-instrument entry=authored-case-has-observable-delta
      assert.ok(
        entry.semanticMismatchCount > 0,
        `${entry.fixtureId} has no observable authored-liveness correction`,
      );
    } else {
      assert.equal(entry.semanticPreserved, true);
    }
    // FALSIFIER: id=linked-emission-gate-031 class=accounting via=--inject-unexpected-divergence producer=can-fail owner=linked-emission-instrument entry=committed-corpus-green
    assert.ok(entry.reasons.length > 0, `${entry.fixtureId} has no derived divergence reason`);
  } else {
    assert.equal(entry.byteEqual, false);
  }
}

const expectedLedgerFixtureIds = expectedDivergenceLedger.entries
  .map((entry) => entry.fixtureId)
  .toSorted();
const expectedFixtureIds = report.cases
  .filter((entry) => entry.differenceClass === "expected")
  .map((entry) => entry.fixtureId)
  .toSorted();
assert.equal(new Set(expectedLedgerFixtureIds).size, expectedLedgerFixtureIds.length);
// FALSIFIER: id=linked-emission-gate-032 class=accounting via=--inject-unexpected-divergence producer=can-fail owner=linked-emission-instrument entry=committed-corpus-green
assert.deepEqual(
  expectedLedgerFixtureIds,
  expectedFixtureIds,
  "the expected-divergence ledger must exactly equal the live expected fixture set",
);
assert.equal(baseline.expectedDivergenceCount, expectedDivergenceLedger.entries.length);
assert.equal(report.expectedDivergenceCount, expectedDivergenceLedger.entries.length);

const casesByFixtureId = new Map(report.cases.map((entry) => [entry.fixtureId, entry]));
for (const entry of expectedDivergenceLedger.entries) {
  const liveCase = casesByFixtureId.get(entry.fixtureId);
  // FALSIFIER: id=linked-emission-gate-033 class=accounting via=--inject-unexpected-divergence producer=can-fail owner=linked-emission-instrument entry=committed-corpus-green
  assert.ok(liveCase, `expected-divergence fixture ${entry.fixtureId} is absent from the corpus`);
  assert.equal(liveCase.differenceClass, "expected");
  assert.equal(liveCase.byteEqual, false);
  if (entry.classificationArm === "moduleAttributedReachabilityCorrection") {
    // FALSIFIER: id=linked-emission-gate-034 class=accounting via=--inject-unexpected-divergence producer=can-fail owner=linked-emission-instrument entry=committed-corpus-green
    assert.ok(
      census.liveDeclarationFixtures.some((fixture) => fixture.fixtureId === entry.fixtureId),
      `${entry.fixtureId} is not backed by the authored liveness domain`,
    );
    if (entry.fixtureId === "module-qualified-reachability") {
      assert.equal(liveCase.semanticPreserved, false);
      assert.equal(liveCase.semanticMismatchCount, 1);
      // FALSIFIER: id=linked-emission-gate-035 class=accounting via=--inject-unexpected-divergence producer=can-fail owner=linked-emission-instrument entry=committed-corpus-green
      assert.deepEqual(liveCase.legacyMarkerOrder, ["_entry-marker_1"]);
      // FALSIFIER: id=linked-emission-gate-036 class=accounting via=--inject-unexpected-divergence producer=can-fail owner=linked-emission-instrument entry=committed-corpus-green
      assert.deepEqual(liveCase.linkedMarkerOrder, ["_dependency-own_1", "_entry-marker_1"]);
      // FALSIFIER: id=linked-emission-gate-037 class=accounting via=--inject-unexpected-divergence producer=can-fail owner=linked-emission-instrument entry=committed-corpus-green
      assert.deepEqual(liveCase.authoritativeMarkerOrder, ["_dependency-own_1", "_entry-marker_1"]);
    } else if (entry.fixtureId === "module-qualified-composes-reachability") {
      // FALSIFIER: id=linked-emission-gate-038 class=accounting via=--inject-unexpected-divergence producer=can-fail owner=linked-emission-instrument entry=committed-corpus-green
      assert.ok(liveCase.legacyMarkerOrder.includes("_card_0"));
      // FALSIFIER: id=linked-emission-gate-039 class=accounting via=--inject-unexpected-divergence producer=can-fail owner=linked-emission-instrument entry=committed-corpus-green
      assert.ok(liveCase.linkedMarkerOrder.includes("_base_0"));
      // FALSIFIER: id=linked-emission-gate-040 class=accounting via=--inject-unexpected-divergence producer=can-fail owner=linked-emission-instrument entry=committed-corpus-green
      assert.ok(liveCase.linkedMarkerOrder.includes("_base-live_1"));
    } else if (entry.fixtureId === "module-qualified-at-rule-reachability") {
      assert.equal(liveCase.semanticPreserved, false);
      assert.equal(liveCase.semanticMismatchCount, 6);
      // FALSIFIER: id=linked-emission-ledger-at-rule-legacy-markers class=accounting via=--inject-unexpected-divergence producer=can-fail owner=linked-emission-instrument entry=ledger-legacy-at-rule-marker-order
      assert.deepEqual(liveCase.legacyMarkerOrder, ["_card_0"]);
      // FALSIFIER: id=linked-emission-ledger-at-rule-linked-markers class=accounting via=--inject-unexpected-divergence producer=can-fail owner=linked-emission-instrument entry=ledger-linked-at-rule-marker-order
      assert.deepEqual(liveCase.linkedMarkerOrder, ["_base_0", "_media-live_2", "_card_0"]);
      // FALSIFIER: id=linked-emission-ledger-at-rule-authority-markers class=placement via=--inject-unexpected-divergence producer=can-fail owner=linked-emission-instrument entry=ledger-authoritative-at-rule-marker-order
      assert.deepEqual(liveCase.authoritativeMarkerOrder, ["_base_0", "_media-live_2", "_card_0"]);
    } else {
      assert.equal(liveCase.semanticPreserved, false);
      // FALSIFIER: id=linked-emission-authored-delta-observed class=liveness via=--inject-authored-liveness-flip producer=can-fail owner=linked-emission-instrument entry=authored-liveness-fixture-has-observable-delta
      assert.ok(
        liveCase.semanticMismatchCount > 0,
        `${entry.fixtureId} has no observable module-attributed correction`,
      );
    }
  } else if (entry.fixtureId === "entry-ordinal-skew") {
    assert.equal(liveCase.semanticPreserved, false);
    // FALSIFIER: id=linked-emission-ledger-skew-semantic-delta class=accounting via=--inject-collision-expectation-flip producer=can-fail owner=linked-emission-instrument entry=ledger-path-model-difference-nonempty
    assert.ok(liveCase.semanticMismatchCount > 0);
  } else {
    assert.equal(liveCase.semanticPreserved, true);
  }
  // FALSIFIER: id=linked-emission-gate-041 class=accounting via=--inject-unexpected-divergence producer=can-fail owner=linked-emission-instrument entry=committed-corpus-green
  assert.deepEqual(
    entry.derivedReasons,
    liveCase.reasons,
    `expected-divergence reasons drifted for ${entry.fixtureId}`,
  );
  assert.equal(
    entry.witnessDigest,
    divergenceWitnessDigest(entry.fixtureId, liveCase.legacySha256, liveCase.linkedSha256),
    `expected-divergence witness digest drifted for ${entry.fixtureId}`,
  );
  // FALSIFIER: id=linked-emission-gate-042 class=accounting via=--inject-unexpected-divergence producer=can-fail owner=linked-emission-instrument entry=committed-corpus-green
  assert.ok(
    entry.justification.trim().length > 0,
    `${entry.fixtureId} has no expected-difference justification`,
  );
}

const ledgerFixtureIds = divergenceLedger.entries.map((entry) => entry.fixtureId).toSorted();
const unexpectedFixtureIds = report.cases
  .filter((entry) => entry.differenceClass === "unexpected")
  .map((entry) => entry.fixtureId)
  .toSorted();
assert.equal(new Set(ledgerFixtureIds).size, ledgerFixtureIds.length);
// FALSIFIER: id=linked-emission-gate-043 class=accounting via=--inject-unexpected-divergence producer=can-fail owner=linked-emission-instrument entry=committed-corpus-green
assert.deepEqual(
  ledgerFixtureIds,
  unexpectedFixtureIds,
  "the open-divergence ledger must exactly equal the live unexpected fixture set",
);
assert.equal(baseline.maximumUnexpectedDivergenceCount, divergenceLedger.entries.length);
assert.equal(report.unexpectedDivergenceCount, divergenceLedger.entries.length);

for (const entry of divergenceLedger.entries) {
  const liveCase = casesByFixtureId.get(entry.fixtureId);
  // FALSIFIER: id=linked-emission-gate-044 class=accounting via=--inject-unexpected-divergence producer=can-fail owner=linked-emission-instrument entry=committed-corpus-green
  assert.ok(liveCase, `open-divergence fixture ${entry.fixtureId} is absent from the live corpus`);
  assert.equal(
    entry.witnessDigest,
    divergenceWitnessDigest(entry.fixtureId, liveCase.legacySha256, liveCase.linkedSha256),
    `open-divergence witness digest drifted for ${entry.fixtureId}`,
  );
  // FALSIFIER: id=linked-emission-gate-045 class=accounting via=--inject-unexpected-divergence producer=can-fail owner=linked-emission-instrument entry=committed-corpus-green
  assert.ok(entry.owningGoal.trim().length > 0, `${entry.fixtureId} has no owning goal`);
  // FALSIFIER: id=linked-emission-gate-046 class=accounting via=--inject-unexpected-divergence producer=can-fail owner=linked-emission-instrument entry=committed-corpus-green
  assert.ok(entry.note.trim().length > 0, `${entry.fixtureId} has no closure note`);
  // FALSIFIER: id=linked-emission-gate-047 class=accounting via=--inject-unexpected-divergence producer=can-fail owner=linked-emission-instrument entry=committed-corpus-green
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
