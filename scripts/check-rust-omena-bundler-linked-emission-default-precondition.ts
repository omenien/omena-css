import { strict as assert } from "node:assert";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { existsSync, readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { loadCheckManifest } from "../packages/check-orchestrator/src/manifest/index.ts";

interface DifferentialBaselineV0 {
  readonly schemaVersion: "0";
  readonly product: "omena-bundler.linked-emission-byte-differential-baseline";
  readonly coverageScope: "boundedMultiModuleFixtures" | "fullCorpus";
  readonly fullCorpusCoverage: boolean;
  readonly maximumUnexpectedDivergenceCount: number;
}

interface DifferentialReportV0 {
  readonly fixtureCount: number;
  readonly unexpectedDivergenceCount: number;
}

interface DifferentialCensusV0 {
  readonly coverageScope: "boundedMultiModuleFixtures" | "fullCorpus";
  readonly fullCorpusCoverage: boolean;
  readonly fixtureCount: number;
  readonly populationCount: number;
  readonly coveredShapeCount: number;
  readonly notCoveredShapeCount: number;
  readonly shapes: ReadonlyArray<{
    readonly fixtureIds: readonly string[];
  }>;
  readonly notCovered: ReadonlyArray<unknown>;
}

interface DifferentialEnvelopeV0 {
  readonly schemaVersion: "0";
  readonly product: "omena-diff-test.linked-emission-byte-differential-envelope";
  readonly report: DifferentialReportV0;
  readonly census: DifferentialCensusV0;
}

interface PreconditionOwnerRefV0 {
  readonly goalRef: string;
  readonly artifactPath: string;
  readonly gateId: string;
}

interface FlipPreconditionV0 {
  readonly schemaVersion: "0";
  readonly product: "omena-bundler.linked-emission-default-precondition";
  readonly defaultEmissionPath: "importInlineLegacy";
  readonly candidateEmissionPath: "linkedOrder";
  readonly maintenanceScope: string;
  readonly conditions: {
    readonly fullCorpusDifferential: {
      readonly requiredCoverageScope: "fullCorpus";
      readonly owner: string;
      readonly ownerRef: PreconditionOwnerRefV0;
      readonly reason: string;
    };
    readonly majorVersionBoundary: {
      readonly minimumMajorVersion: number;
      readonly owner: string;
      readonly ownerRef: PreconditionOwnerRefV0;
      readonly reason: string;
    };
    readonly unexpectedDivergenceCensus: {
      readonly maximumCount: number;
      readonly owner: string;
      readonly ownerRef: PreconditionOwnerRefV0;
      readonly reason: string;
    };
  };
  readonly expectedEvaluation: {
    readonly fullCorpusDifferential: boolean;
    readonly majorVersionBoundary: boolean;
    readonly unexpectedDivergenceCensus: boolean;
    readonly decision: "ready" | "notReady";
  };
  readonly residuals: {
    readonly emissionOrderLedgerPath: string;
    readonly requiredEmissionOrderIds: readonly string[];
    readonly carriedFollowUps: ReadonlyArray<{
      readonly id: string;
      readonly subject: string;
      readonly owner: string;
    }>;
  };
}

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const contract = readJson<FlipPreconditionV0>(
  "rust/omena-linked-emission-default-precondition.json",
);
const baseline = readJson<DifferentialBaselineV0>(
  "rust/omena-linked-emission-byte-differential-baseline.json",
);
const packageManifest = readJson<{ readonly version: string }>("package.json");
const residualLedger = readJson<{
  readonly entries: ReadonlyArray<{
    readonly id: string;
    readonly owner: string;
    readonly reason: string;
  }>;
}>(contract.residuals.emissionOrderLedgerPath);
const personaPresets = readJson<{
  readonly presets: ReadonlyArray<{
    readonly missingCapabilities: ReadonlyArray<{ readonly id: string; readonly owner: string }>;
  }>;
}>("rust/crates/omena-cli/persona-presets.json");

assert.equal(contract.schemaVersion, "0");
assert.equal(contract.product, "omena-bundler.linked-emission-default-precondition");
assert.equal(contract.defaultEmissionPath, "importInlineLegacy");
assert.equal(contract.candidateEmissionPath, "linkedOrder");
assert.equal(
  contract.maintenanceScope,
  "This artifact records prerequisites only and does not authorize changing the default emission path.",
);
const checkManifest = loadCheckManifest(repoRoot);
const declaredGateIds = new Set(checkManifest.gates.map((gate) => gate.id));
for (const [conditionKey, condition] of Object.entries(contract.conditions)) {
  assert.ok(condition.owner.length > 0, `${conditionKey}.owner is empty`);
  assert.ok(condition.reason.length > 0, `${conditionKey}.reason is empty`);
  assert.ok(
    condition.ownerRef.goalRef.length > 0,
    `${conditionKey}.ownerRef.goalRef is unresolvable: ${condition.ownerRef.goalRef}`,
  );
  assert.ok(
    existsSync(path.resolve(repoRoot, condition.ownerRef.artifactPath)),
    `${conditionKey}.ownerRef.artifactPath is unresolvable: ${condition.ownerRef.artifactPath}`,
  );
  assert.ok(
    declaredGateIds.has(condition.ownerRef.gateId),
    `${conditionKey}.ownerRef.gateId is unresolvable: ${condition.ownerRef.gateId}`,
  );
}
assert.deepEqual(contract.residuals.requiredEmissionOrderIds.toSorted(), [
  "cascade-optimal-ordering",
  "cross-chunk-css-order",
  "dialect-import-semantics",
]);
assert.deepEqual(
  residualLedger.entries.map((entry) => entry.id).toSorted(),
  contract.residuals.requiredEmissionOrderIds.toSorted(),
);
assert.ok(
  residualLedger.entries.every((entry) => entry.owner.length > 0 && entry.reason.length > 0),
);

const semanticEmissionOrder = personaPresets.presets
  .flatMap((preset) => preset.missingCapabilities)
  .find((entry) => entry.id === "semantic-emission-order");
assert.deepEqual(semanticEmissionOrder, {
  id: "semantic-emission-order",
  owner: "bundle-engine",
  availabilityCondition:
    "Emission order is represented by a typed plan with a legacy-byte-compatible default.",
});
const expectedFollowUps = [
  {
    id: fingerprint("persona-missing-capability", "semantic-emission-order"),
    subject: "semantic-emission-order",
    owner: "bundle-engine",
  },
  {
    id: "c09ae163a6496bc83d2c959d553a6d817d4c41a1c2b6af77e418c4d479fcb559",
    subject: "linked-order-emission-authority",
    owner: "product surface program",
  },
];
assert.deepEqual(contract.residuals.carriedFollowUps, expectedFollowUps);

const differential = spawnSync(
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
  ],
  { encoding: "utf8" },
);
assert.equal(
  differential.status,
  0,
  [differential.stdout, differential.stderr].filter(Boolean).join("\n"),
);
const envelope = JSON.parse(differential.stdout) as DifferentialEnvelopeV0;
assert.equal(envelope.schemaVersion, "0");
assert.equal(envelope.product, "omena-diff-test.linked-emission-byte-differential-envelope");
const { report, census } = envelope;
assert.equal(census.fixtureCount, report.fixtureCount);
assert.equal(census.populationCount, census.shapes.length);
assert.equal(census.coveredShapeCount + census.notCoveredShapeCount, census.populationCount);
const derivedFullCorpusCoverage =
  census.notCovered.length === 0 && census.shapes.every((entry) => entry.fixtureIds.length > 0);
assert.equal(census.fullCorpusCoverage, derivedFullCorpusCoverage);
assert.equal(
  census.coverageScope,
  derivedFullCorpusCoverage ? "fullCorpus" : "boundedMultiModuleFixtures",
);
assert.equal(baseline.coverageScope, census.coverageScope);
assert.equal(baseline.fullCorpusCoverage, census.fullCorpusCoverage);
const currentMajorVersion = Number.parseInt(packageManifest.version.split(".")[0] ?? "", 10);
assert.ok(Number.isSafeInteger(currentMajorVersion));

const actualEvaluation = {
  fullCorpusDifferential:
    census.fullCorpusCoverage &&
    census.coverageScope === contract.conditions.fullCorpusDifferential.requiredCoverageScope,
  majorVersionBoundary:
    currentMajorVersion >= contract.conditions.majorVersionBoundary.minimumMajorVersion,
  unexpectedDivergenceCensus:
    report.unexpectedDivergenceCount <=
      contract.conditions.unexpectedDivergenceCensus.maximumCount &&
    report.unexpectedDivergenceCount <= baseline.maximumUnexpectedDivergenceCount,
};
const ready = Object.values(actualEvaluation).every(Boolean);
const expectedEvaluation = process.argv.includes("--fabricate-ready")
  ? {
      fullCorpusDifferential: true,
      majorVersionBoundary: true,
      unexpectedDivergenceCensus: true,
      decision: "ready" as const,
    }
  : contract.expectedEvaluation;
assert.deepEqual(expectedEvaluation, {
  ...actualEvaluation,
  decision: ready ? "ready" : "notReady",
});

const queryTypes = readFileSync("rust/crates/omena-query/src/types.rs", "utf8");
assert.match(
  queryTypes,
  /pub enum OmenaQueryBundleEmissionPathV0\s*\{\s*#\[default\]\s*ImportInlineLegacy,/u,
);

console.log(
  JSON.stringify(
    {
      schemaVersion: "0",
      product: "omena-bundler.linked-emission-default-precondition-evaluation",
      decision: ready ? "ready" : "notReady",
      currentMajorVersion,
      requiredMinimumMajorVersion: contract.conditions.majorVersionBoundary.minimumMajorVersion,
      observedCoverageScope: census.coverageScope,
      observedFixtureCount: report.fixtureCount,
      observedUnexpectedDivergenceCount: report.unexpectedDivergenceCount,
      conditions: actualEvaluation,
      residualEntryCount:
        contract.residuals.requiredEmissionOrderIds.length +
        contract.residuals.carriedFollowUps.length,
    },
    null,
    2,
  ),
);

function readJson<T>(filePath: string): T {
  return JSON.parse(readFileSync(filePath, "utf8")) as T;
}

function fingerprint(kind: string, key: string): string {
  return createHash("sha256").update(`${kind}\0${key}`).digest("hex");
}
