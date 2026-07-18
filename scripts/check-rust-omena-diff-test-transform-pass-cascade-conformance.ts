import { spawnSync } from "node:child_process";
import { strict as assert } from "node:assert";
import { readFileSync } from "node:fs";

const result = spawnSync(
  "cargo",
  [
    "run",
    "--manifest-path",
    "rust/Cargo.toml",
    "-p",
    "omena-diff-test",
    "--bin",
    "omena-diff-test-boundary",
    "--quiet",
  ],
  {
    encoding: "utf8",
    maxBuffer: 128 * 1024 * 1024,
  },
);

assert.equal(
  result.status,
  0,
  `omena-diff-test boundary failed\nstdout=${result.stdout}\nstderr=${result.stderr}`,
);

const summary = JSON.parse(result.stdout) as {
  readonly product: string;
  readonly transformPassCascadeConformanceRecordCount: number;
  readonly transformPassCascadeConformanceModelConformantCount: number;
  readonly transformPassCascadeConformanceDivergentCount: number;
  readonly transformPassCascadeConformanceNotExercisedCount: number;
  readonly transformPassCascadeConformanceMeasuredComparisonCount: number;
  readonly allTransformPassCascadeConformancePassesAccountedFor: boolean;
  readonly allTransformPassCascadeConformanceRecordsHaveOneVerdict: boolean;
  readonly allTransformPassCascadeConformanceOracleBaselinesMatch: boolean;
  readonly allTransformPassCascadeConformanceVerdictsMatchMeasurements: boolean;
  readonly allTransformPassCascadeConformanceDivergencesReasoned: boolean;
  readonly allTransformPassCascadeConformanceFamiliesNonVacuousOrNamedGap: boolean;
  readonly transformPassCascadeConformanceReport: {
    readonly product: string;
    readonly passCount: number;
    readonly caseCount: number;
    readonly recordCount: number;
    readonly modelConformantCount: number;
    readonly divergentCount: number;
    readonly notExercisedCount: number;
    readonly measuredComparisonCount: number;
    readonly aggregatePolicy: "observationalCoverageSnapshot";
    readonly allOracleBaselinesMatch: boolean;
    readonly allVerdictsMatchMeasurements: boolean;
    readonly allDivergencesReasoned: boolean;
    readonly propertyCorpusWitnessEarned: boolean;
    readonly propertyCorpusWitness?: {
      readonly guarantee: string;
      readonly earnedVia: string;
      readonly provenance: readonly string[];
    };
    readonly familyReports: readonly {
      readonly passClass: string;
      readonly passCount: number;
      readonly exercisedRecordCount: number;
      readonly namedGap?: string;
    }[];
    readonly records: readonly {
      readonly recordKey: string;
      readonly passId: string;
      readonly passClass: string;
      readonly oracle: string;
      readonly fixtureId: string;
      readonly property: string;
      readonly comparedFacts: readonly string[];
      readonly runtimeStatus: string;
      readonly mutationCount: number;
      readonly oracleBaselineMatch?: boolean;
      readonly comparisonPerformed: boolean;
      readonly oracleMatch?: boolean;
      readonly verdict: "modelConformant" | "divergentWithReason" | "notExercised";
      readonly reason?: string;
    }[];
  };
};

const divergenceBaseline = JSON.parse(
  readFileSync(
    "rust/crates/omena-diff-test/sass-spec-corpus/transform-pass-cascade-divergences.json",
    "utf8",
  ),
) as {
  readonly schemaVersion: string;
  readonly product: string;
  readonly entries: readonly {
    readonly recordKey: string;
    readonly reason: string;
  }[];
};

assert.equal(summary.product, "omena-diff-test.boundary");
assert.equal(
  summary.transformPassCascadeConformanceReport.product,
  "omena-diff-test.transform-pass-cascade-conformance",
);
assert.equal(summary.transformPassCascadeConformanceReport.passCount, 44);
assert.ok(summary.transformPassCascadeConformanceReport.caseCount >= 1);
assert.equal(
  summary.transformPassCascadeConformanceRecordCount,
  summary.transformPassCascadeConformanceReport.recordCount,
);
assert.equal(
  summary.transformPassCascadeConformanceRecordCount,
  summary.transformPassCascadeConformanceReport.passCount *
    summary.transformPassCascadeConformanceReport.caseCount,
);
assert.ok(summary.transformPassCascadeConformanceModelConformantCount >= 1);
assert.ok(summary.transformPassCascadeConformanceNotExercisedCount >= 1);
assert.ok(summary.transformPassCascadeConformanceMeasuredComparisonCount >= 1);
assert.equal(
  summary.transformPassCascadeConformanceRecordCount,
  summary.transformPassCascadeConformanceModelConformantCount +
    summary.transformPassCascadeConformanceDivergentCount +
    summary.transformPassCascadeConformanceNotExercisedCount,
  "observational verdict buckets must partition every record",
);
assert.equal(
  summary.transformPassCascadeConformanceMeasuredComparisonCount,
  summary.transformPassCascadeConformanceModelConformantCount +
    summary.transformPassCascadeConformanceDivergentCount,
  "measured comparisons must equal the two exercised verdict buckets",
);
assert.equal(
  summary.transformPassCascadeConformanceReport.aggregatePolicy,
  "observationalCoverageSnapshot",
);
assert.ok(summary.allTransformPassCascadeConformancePassesAccountedFor);
assert.ok(summary.allTransformPassCascadeConformanceRecordsHaveOneVerdict);
assert.ok(summary.allTransformPassCascadeConformanceOracleBaselinesMatch);
assert.ok(summary.allTransformPassCascadeConformanceVerdictsMatchMeasurements);
assert.ok(summary.allTransformPassCascadeConformanceDivergencesReasoned);
assert.ok(summary.allTransformPassCascadeConformanceFamiliesNonVacuousOrNamedGap);
assert.equal(
  summary.transformPassCascadeConformanceMeasuredComparisonCount,
  summary.transformPassCascadeConformanceReport.measuredComparisonCount,
);
assert.equal(
  summary.transformPassCascadeConformanceDivergentCount,
  summary.transformPassCascadeConformanceReport.divergentCount,
);
assert.ok(summary.transformPassCascadeConformanceReport.allVerdictsMatchMeasurements);
assert.ok(summary.transformPassCascadeConformanceReport.allDivergencesReasoned);
assert.ok(summary.transformPassCascadeConformanceReport.allOracleBaselinesMatch);
assert.ok(summary.transformPassCascadeConformanceReport.propertyCorpusWitnessEarned);
assert.equal(
  summary.transformPassCascadeConformanceReport.propertyCorpusWitness?.earnedVia,
  "propertyCorpusWitness",
);
assert.equal(
  summary.transformPassCascadeConformanceReport.propertyCorpusWitness?.guarantee,
  "metricInputFixtureWitness",
);
assert.ok(
  summary.transformPassCascadeConformanceReport.propertyCorpusWitness?.provenance.includes(
    "property-corpus-witness:transform-pass-cascade-conformance",
  ),
);

const measuredRecords = summary.transformPassCascadeConformanceReport.records.filter(
  (record) => record.comparisonPerformed,
);
const divergentRecords = summary.transformPassCascadeConformanceReport.records.filter(
  (record) => record.verdict === "divergentWithReason",
);
assert.ok(
  summary.transformPassCascadeConformanceReport.records.every(
    (record) => record.oracleBaselineMatch !== false,
  ),
);
assert.equal(
  measuredRecords.length,
  summary.transformPassCascadeConformanceReport.measuredComparisonCount,
);
assert.equal(divergentRecords.length, summary.transformPassCascadeConformanceReport.divergentCount);
for (const record of summary.transformPassCascadeConformanceReport.records) {
  if (!record.comparisonPerformed) {
    assert.equal(record.oracleMatch, undefined, record.recordKey);
    assert.equal(record.verdict, "notExercised", record.recordKey);
    continue;
  }
  assert.ok(record.comparedFacts.length >= 1, record.recordKey);
  assert.equal(record.runtimeStatus, "applied", record.recordKey);
  assert.ok(record.mutationCount >= 1, record.recordKey);
  assert.equal(typeof record.oracleMatch, "boolean", record.recordKey);
  assert.equal(
    record.verdict,
    record.oracleMatch ? "modelConformant" : "divergentWithReason",
    record.recordKey,
  );
  if (!record.oracleMatch) {
    assert.ok(record.reason?.trim(), record.recordKey);
  }
}

assert.equal(divergenceBaseline.schemaVersion, "0");
assert.equal(
  divergenceBaseline.product,
  "omena-diff-test.transform-pass-cascade-divergence-baseline",
);
const baselineKeys = new Set<string>();
for (const entry of divergenceBaseline.entries) {
  assert.ok(entry.recordKey.trim());
  assert.ok(entry.reason.trim());
  assert.ok(
    !baselineKeys.has(entry.recordKey),
    `duplicate divergence baseline: ${entry.recordKey}`,
  );
  baselineKeys.add(entry.recordKey);
}
const unreviewedDivergences = divergentRecords
  .filter((record) => !baselineKeys.has(record.recordKey))
  .map((record) => ({ recordKey: record.recordKey, reason: record.reason }));
assert.deepEqual(
  unreviewedDivergences,
  [],
  `new transform-pass cascade divergences require a reasoned baseline entry:\n${JSON.stringify(
    unreviewedDivergences,
    null,
    2,
  )}`,
);
assert.deepEqual(
  summary.transformPassCascadeConformanceReport.familyReports.map((family) => family.passClass),
  ["structural", "textLocal", "moduleEvaluation", "emission"],
);
assert.ok(
  summary.transformPassCascadeConformanceReport.familyReports.every(
    (family) => family.passCount >= 1 && (family.exercisedRecordCount > 0 || family.namedGap),
  ),
);
const familyByClass = new Map(
  summary.transformPassCascadeConformanceReport.familyReports.map((family) => [
    family.passClass,
    family,
  ]),
);
assert.ok(
  (familyByClass.get("structural")?.exercisedRecordCount ?? 0) > 0,
  "structural transform passes must stay exercised by the cascade oracle corpus",
);
assert.ok(
  (familyByClass.get("textLocal")?.exercisedRecordCount ?? 0) > 0,
  "textLocal transform passes must stay exercised by the cascade oracle corpus",
);

process.stdout.write(
  `${JSON.stringify(
    {
      schemaVersion: "0",
      product: "omena-diff-test.transform-pass-cascade-conformance",
      recordCount: summary.transformPassCascadeConformanceRecordCount,
      modelConformantCount: summary.transformPassCascadeConformanceModelConformantCount,
      divergentCount: summary.transformPassCascadeConformanceDivergentCount,
      notExercisedCount: summary.transformPassCascadeConformanceNotExercisedCount,
      measuredComparisonCount: summary.transformPassCascadeConformanceMeasuredComparisonCount,
      aggregatePolicy: summary.transformPassCascadeConformanceReport.aggregatePolicy,
      remainingBaselineDivergenceCount: divergenceBaseline.entries.filter(
        (entry) => !divergentRecords.some((record) => record.recordKey === entry.recordKey),
      ).length,
      complete: true,
    },
    null,
    2,
  )}\n`,
);
