import { strict as assert } from "node:assert";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";

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

interface FlipPreconditionV0 {
  readonly schemaVersion: "0";
  readonly product: "omena-bundler.linked-emission-default-precondition";
  readonly defaultEmissionPath: "importInlineLegacy";
  readonly candidateEmissionPath: "linkedOrder";
  readonly conditions: {
    readonly fullCorpusDifferential: {
      readonly requiredCoverageScope: "fullCorpus";
      readonly owner: string;
      readonly reason: string;
    };
    readonly majorVersionBoundary: {
      readonly minimumMajorVersion: number;
      readonly owner: string;
      readonly reason: string;
    };
    readonly unexpectedDivergenceCensus: {
      readonly maximumCount: number;
      readonly owner: string;
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
assert.ok(
  Object.values(contract.conditions).every(
    (condition) => condition.owner.length > 0 && condition.reason.length > 0,
  ),
);
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
const report = JSON.parse(differential.stdout) as DifferentialReportV0;
const currentMajorVersion = Number.parseInt(packageManifest.version.split(".")[0] ?? "", 10);
assert.ok(Number.isSafeInteger(currentMajorVersion));

const actualEvaluation = {
  fullCorpusDifferential:
    baseline.fullCorpusCoverage &&
    baseline.coverageScope === contract.conditions.fullCorpusDifferential.requiredCoverageScope,
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
      observedCoverageScope: baseline.coverageScope,
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
