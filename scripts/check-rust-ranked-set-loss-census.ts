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
  readonly measurementInvocationCount: number;
  readonly rankedSetOutcomeCount: number;
  readonly multiCandidateInexactRankedSetCount: number;
  readonly rowCount: number;
  readonly recoverableCount: number;
  readonly undecidableCount: number;
  readonly classCounts: Readonly<Record<string, number>>;
  readonly functionPopulations: Readonly<Record<string, number>>;
  readonly invocationSitePopulations: Readonly<Record<string, number>>;
  readonly decidingAxisCounts: Readonly<Record<string, number>>;
  readonly unclassifiedInvocationCount: number;
  readonly entries: readonly {
    readonly id: string;
    readonly measurementInvocationCount: number;
    readonly rankedSetOutcomeCount: number;
    readonly multiCandidateInexactRankedSetCount: number;
    readonly rowCount: number;
    readonly rows: readonly RankedSetLossCensusRowV0[];
  }[];
}

interface RankedSetLossCandidateV0 {
  readonly declarationId: string;
  readonly level: string;
  readonly layerRank: number;
  readonly scopeProximity: number;
  readonly specificityExactness: "exact" | "inexact";
}

interface RankedSetLossCensusRowV0 {
  readonly declarationIds: readonly string[];
  readonly candidateCount: number;
  readonly candidates: readonly RankedSetLossCandidateV0[];
  readonly classification: FixtureResultV0["classification"];
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
const artifactRows = artifact.entries.flatMap((entry) => entry.rows);
assert.equal(artifact.rowCount, artifactRows.length, "entry rows must equal the artifact row count");
assert.equal(
  artifact.measurementInvocationCount,
  sum(artifact.entries.map((entry) => entry.measurementInvocationCount)),
  "measurement invocation denominator must be derived from entry captures",
);
assert.equal(
  artifact.rankedSetOutcomeCount,
  sum(artifact.entries.map((entry) => entry.rankedSetOutcomeCount)),
  "RankedSet denominator must be derived from entry captures",
);
assert.equal(
  artifact.multiCandidateInexactRankedSetCount,
  sum(artifact.entries.map((entry) => entry.multiCandidateInexactRankedSetCount)),
  "multi-candidate inexact denominator must be derived from entry captures",
);
assert.ok(
  artifact.measurementInvocationCount >= artifact.rankedSetOutcomeCount &&
    artifact.rankedSetOutcomeCount >= artifact.rowCount,
  "capture denominators must narrow from invocations to RankedSet outcomes to classified rows",
);
assert.equal(
  artifact.multiCandidateInexactRankedSetCount,
  artifactRows.filter((row) => row.candidateCount > 1).length,
  "multi-candidate inexact denominator must equal eligible captured rows",
);
for (const row of artifactRows) {
  assert.equal(row.candidateCount, row.candidates.length, "candidate payload must be total");
  assert.deepEqual(
    row.declarationIds,
    row.candidates.map((candidate) => candidate.declarationId),
    "candidate identities must align with the legacy declaration-id projection",
  );
  assert.deepEqual(
    row.classification,
    classifyArtifactRow(row),
    "committed classification must be independently recomputable from axis-prefix values",
  );
}
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
  artifact.limitations.some((limitation) =>
    limitation.includes("No multi-candidate inexact RankedSet outcome"),
  ),
  "the artifact must disclose the empty eligible multi-candidate population",
);
assert.ok(
  artifact.limitations.some((limitation) =>
    limitation.includes("zero eligible product executions"),
  ),
  "the artifact must disclose that product data did not execute the classifier predicate",
);
assert.ok(
  artifact.limitations.some((limitation) => limitation.includes("no @layer")),
  "the artifact must disclose the missing cascade-layer corpus shape",
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
      measurementInvocationCount: artifact.measurementInvocationCount,
      rankedSetOutcomeCount: artifact.rankedSetOutcomeCount,
      multiCandidateInexactRankedSetCount: artifact.multiCandidateInexactRankedSetCount,
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

const cascadeLevels = [
  "userAgentNormal",
  "userNormal",
  "authorNormal",
  "inlineNormal",
  "animation",
  "authorImportant",
  "inlineImportant",
  "userImportant",
  "userAgentImportant",
  "transition",
] as const;

function classifyArtifactRow(
  row: RankedSetLossCensusRowV0,
): FixtureResultV0["classification"] {
  assert.ok(
    row.candidates.some((candidate) => candidate.specificityExactness === "inexact"),
    "census rows must contain an inexact candidate",
  );
  if (row.candidates.length === 1) return "singleInexactCandidate";

  const ranked = row.candidates.toSorted((left, right) => compareAxisPrefix(right, left));
  const winner = ranked[0];
  const runnerUp = ranked[1];
  assert.ok(winner && runnerUp, "multi-candidate row must contain a winner and runner-up");
  if (compareAxisPrefix(winner, runnerUp) !== 1) return "noStrictAxisDominance";
  if (winner.specificityExactness === "inexact") return "axisWinnerInexact";

  const axis =
    levelRank(winner.level) !== levelRank(runnerUp.level)
      ? "level"
      : winner.layerRank !== runnerUp.layerRank
        ? "layerRank"
        : "scopeProximity";
  return { recoverableAxisDominant: { axis } };
}

function compareAxisPrefix(
  left: RankedSetLossCandidateV0,
  right: RankedSetLossCandidateV0,
): -1 | 0 | 1 {
  return compareNumberTuple(
    [levelRank(left.level), left.layerRank, -left.scopeProximity],
    [levelRank(right.level), right.layerRank, -right.scopeProximity],
  );
}

function levelRank(level: string): number {
  const rank = cascadeLevels.indexOf(level as (typeof cascadeLevels)[number]);
  assert.notEqual(rank, -1, `unknown cascade level in census row: ${level}`);
  return rank;
}

function compareNumberTuple(left: readonly number[], right: readonly number[]): -1 | 0 | 1 {
  for (let index = 0; index < left.length; index += 1) {
    const delta = (left[index] ?? 0) - (right[index] ?? 0);
    if (delta < 0) return -1;
    if (delta > 0) return 1;
  }
  return 0;
}

function sum(values: readonly number[]): number {
  return values.reduce((total, value) => total + value, 0);
}
