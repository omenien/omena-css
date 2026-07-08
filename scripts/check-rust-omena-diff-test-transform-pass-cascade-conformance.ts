import { spawnSync } from "node:child_process";
import { strict as assert } from "node:assert";

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
  readonly allTransformPassCascadeConformancePassesAccountedFor: boolean;
  readonly allTransformPassCascadeConformanceRecordsHaveOneVerdict: boolean;
  readonly allTransformPassCascadeConformanceFamiliesNonVacuousOrNamedGap: boolean;
  readonly transformPassCascadeConformanceReport: {
    readonly product: string;
    readonly passCount: number;
    readonly caseCount: number;
    readonly recordCount: number;
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
  };
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
assert.equal(summary.transformPassCascadeConformanceDivergentCount, 0);
assert.ok(summary.transformPassCascadeConformanceNotExercisedCount >= 1);
assert.ok(summary.allTransformPassCascadeConformancePassesAccountedFor);
assert.ok(summary.allTransformPassCascadeConformanceRecordsHaveOneVerdict);
assert.ok(summary.allTransformPassCascadeConformanceFamiliesNonVacuousOrNamedGap);
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
assert.deepEqual(
  summary.transformPassCascadeConformanceReport.familyReports.map((family) => family.passClass),
  ["structural", "textLocal", "moduleEvaluation", "emission"],
);
assert.ok(
  summary.transformPassCascadeConformanceReport.familyReports.every(
    (family) => family.passCount >= 1 && (family.exercisedRecordCount > 0 || family.namedGap),
  ),
);

process.stdout.write(
  `${JSON.stringify(
    {
      schemaVersion: "0",
      product: "omena-diff-test.transform-pass-cascade-conformance",
      recordCount: summary.transformPassCascadeConformanceRecordCount,
      modelConformantCount: summary.transformPassCascadeConformanceModelConformantCount,
      notExercisedCount: summary.transformPassCascadeConformanceNotExercisedCount,
      complete: true,
    },
    null,
    2,
  )}\n`,
);
