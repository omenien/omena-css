import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import path from "node:path";
import { spawnSync } from "node:child_process";

interface FixtureResultV0 {
  readonly name: string;
  classification:
    | "axisWinnerInexact"
    | "noStrictAxisDominance"
    | "singleInexactCandidate"
    | {
        readonly recoverableAxisDominant: {
          readonly axis: "level" | "layerRank" | "scopeProximity";
        };
      };
}

interface RankedSetLossCensusArtifactV0 {
  readonly schemaVersion: "0";
  readonly product: "omena-diff-test.ranked-set-loss-census";
  readonly limitations: readonly string[];
  readonly entryCount: number;
  readonly rowCount: number;
  readonly recoverableCount: number;
  readonly undecidableCount: number;
  readonly classCounts: Readonly<Record<string, number>>;
  readonly functionPopulations: Readonly<Record<string, number>>;
  readonly invocationSitePopulations: Readonly<Record<string, number>>;
  readonly decidingAxisCounts: Readonly<Record<string, number>>;
  readonly unclassifiedInvocationCount: number;
}

const repoRoot = process.cwd();
const artifactPath = path.join(
  repoRoot,
  "rust/crates/omena-diff-test/oss-corpus-farm/ranked-set-loss-census.json",
);
const args = new Set(process.argv.slice(2));

const fixtureRun = spawnSync(
  "cargo",
  [
    "run",
    "--quiet",
    "--manifest-path",
    "rust/Cargo.toml",
    "-p",
    "omena-cascade",
    "--example",
    "ranked_set_loss_classifier",
  ],
  {
    cwd: repoRoot,
    encoding: "utf8",
    maxBuffer: 1024 * 1024 * 4,
  },
);
assert.equal(
  fixtureRun.status,
  0,
  `ranked-set loss classifier fixture failed:\n${fixtureRun.stderr}`,
);
const actual = JSON.parse(fixtureRun.stdout) as FixtureResultV0[];

if (args.has("--inject-axis-winner-inexact-recoverable")) {
  fixture(actual, "inexactAxisWinner").classification = {
    recoverableAxisDominant: { axis: "level" },
  };
}
if (args.has("--inject-specificity-only-recoverable")) {
  fixture(actual, "specificityOnlyWinner").classification = {
    recoverableAxisDominant: { axis: "level" },
  };
}
if (args.has("--inject-single-inexact-recoverable")) {
  fixture(actual, "singleInexactCandidate").classification = {
    recoverableAxisDominant: { axis: "level" },
  };
}

const expected: readonly FixtureResultV0[] = [
  {
    name: "inexactAxisWinner",
    classification: "axisWinnerInexact",
  },
  {
    name: "exactAxisWinner",
    classification: {
      recoverableAxisDominant: {
        axis: "level",
      },
    },
  },
  {
    name: "specificityOnlyWinner",
    classification: "noStrictAxisDominance",
  },
  {
    name: "singleInexactCandidate",
    classification: "singleInexactCandidate",
  },
];
assert.deepEqual(
  actual,
  expected,
  "ranked-set loss classification must match the independently authored axis-prefix cases",
);

const artifactBytes = readFileSync(artifactPath);
const artifact = JSON.parse(artifactBytes.toString("utf8")) as RankedSetLossCensusArtifactV0;
assert.equal(artifact.schemaVersion, "0");
assert.equal(artifact.product, "omena-diff-test.ranked-set-loss-census");
assert.equal(
  artifact.entryCount,
  3,
  "the committed farm manifest has three local workspace entries",
);
assert.deepEqual(Object.keys(artifact.classCounts).sort(), [
  "axisWinnerInexact",
  "noStrictAxisDominance",
  "recoverableAxisDominant",
  "singleInexactCandidate",
]);
assert.deepEqual(Object.keys(artifact.functionPopulations).sort(), [
  "cascadeProperty",
  "cascadePropertyOpenWorld",
]);
assert.deepEqual(Object.keys(artifact.decidingAxisCounts).sort(), [
  "layerRank",
  "level",
  "scopeProximity",
]);
assert.equal(
  sum(Object.values(artifact.classCounts)),
  artifact.rowCount,
  "class counts must partition captured rows",
);
assert.equal(
  artifact.recoverableCount + artifact.undecidableCount,
  artifact.rowCount,
  "the derived recoverable and undecidable counts must partition captured rows",
);
assert.equal(
  sum(Object.values(artifact.functionPopulations)),
  artifact.rowCount,
  "per-function populations must partition captured rows",
);
assert.equal(
  sum(Object.values(artifact.invocationSitePopulations)),
  artifact.rowCount,
  "invocation-site populations must partition captured rows",
);
assert.equal(artifact.unclassifiedInvocationCount, 0);
assert.equal(
  artifact.functionPopulations.cascadePropertyOpenWorld,
  0,
  "the no-production-caller function must retain a separate zero population",
);
assert.ok(
  artifact.limitations.some((limitation) => limitation.includes("not prevalence estimates")),
  "the artifact must reject prevalence interpretation",
);
assert.ok(
  artifact.limitations.some((limitation) => limitation.includes("empty by construction")),
  "the artifact must disclose the structurally empty open-world population",
);
assert.doesNotMatch(
  artifactBytes.toString("utf8"),
  /\b(?:certified|proven|verified)\b/iu,
  "the bounded census must not claim unqualified adequacy",
);

const artifactSha256 = createHash("sha256").update(artifactBytes).digest("hex");
process.stdout.write(
  `${JSON.stringify(
    {
      product: "omena-cascade.ranked-set-loss-census-gate",
      artifactSha256,
      entryCount: artifact.entryCount,
      rowCount: artifact.rowCount,
      recoverableCount: artifact.recoverableCount,
      undecidableCount: artifact.undecidableCount,
      functionPopulations: artifact.functionPopulations,
      decidingAxisCounts: artifact.decidingAxisCounts,
    },
    null,
    2,
  )}\n`,
);

function fixture(actual: FixtureResultV0[], name: string): FixtureResultV0 {
  const result = actual.find((candidate) => candidate.name === name);
  assert.ok(result, `classifier fixture is missing ${name}`);
  return result;
}

function sum(values: readonly number[]): number {
  return values.reduce((total, value) => total + value, 0);
}
