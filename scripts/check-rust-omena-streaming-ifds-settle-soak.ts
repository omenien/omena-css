import { strict as assert } from "node:assert";
import { spawnSync } from "node:child_process";

const SETTLE_SOAK_PRODUCT = "omena-streaming-ifds.settle-soak-report";
const BOUNDARY_PRODUCT = "omena-diff-test.boundary";
const SLOPE_PRODUCT = "omena-benchmarks.z5-perf-complexity-slope";
const VALID_SHA256 = "1".repeat(64);

interface SettleSoakReport {
  readonly product: string;
  readonly requestedRevisionCount: number;
  readonly minRevisionCount: number;
  readonly distinctRevisionCount: number;
  readonly consecutiveEqualCount: number;
  readonly divergenceCount: number;
  readonly allRevisionsEqual: boolean;
  readonly hasInSccEdgeRemoval: boolean;
  readonly revisions: readonly {
    readonly contentDigest: string;
    readonly equal: boolean;
    readonly hasInSccEdgeRemoval: boolean;
  }[];
}

interface RunnerSummary {
  readonly demandPrimaryReady: boolean;
  readonly factKeyRouteEngine: string;
  readonly demandSettleRequestedCount: number;
  readonly demandSettleDistinctRevisionCount: number;
  readonly demandSettleMinRevisionCount: number;
  readonly demandSettleDivergenceCount: number;
  readonly demandSettleAllEqual: boolean;
}

const defaultSoak = runSettleSoak({ settleMode: "soak" });
assert.equal(defaultSoak.status, 0, defaultSoak.stderr);
const defaultReport = parseJson<SettleSoakReport>(defaultSoak.stdout, "default settle soak");
assertGreenSettleSoak(defaultReport);

const repeatedRevision = repeatedSettleRevision();
const repeatedRevisions = [repeatedRevision, repeatedRevision, repeatedRevision, repeatedRevision];
const repeatedSoak = runSettleSoak({
  settleMode: "soak",
  settleRevisions: repeatedRevisions,
});
assert.notEqual(
  repeatedSoak.status,
  0,
  "same-input revision repetition must fail in hard soak mode",
);
assert.match(repeatedSoak.stderr, /streaming IFDS settle soak failed/);

const servingSummary = runStreamingEvaluationWithRepeatedRevisions(repeatedRevisions);
assert.equal(servingSummary.demandPrimaryReady, false);
assert.equal(servingSummary.factKeyRouteEngine, "batch");
assert.equal(servingSummary.demandSettleRequestedCount, 4);
assert.equal(servingSummary.demandSettleDistinctRevisionCount, 1);
assert.equal(servingSummary.demandSettleMinRevisionCount, 4);
assert.equal(servingSummary.demandSettleDivergenceCount, 0);
assert.equal(servingSummary.demandSettleAllEqual, false);

console.log(
  JSON.stringify(
    {
      product: "omena-streaming-ifds.settle-soak-check",
      settleProduct: defaultReport.product,
      requestedRevisionCount: defaultReport.requestedRevisionCount,
      distinctRevisionCount: defaultReport.distinctRevisionCount,
      minRevisionCount: defaultReport.minRevisionCount,
      hasInSccEdgeRemoval: defaultReport.hasInSccEdgeRemoval,
      sameInputHardFail: repeatedSoak.status !== 0,
      sameInputServingEngine: servingSummary.factKeyRouteEngine,
    },
    null,
    2,
  ),
);

function runSettleSoak(input: unknown): {
  readonly status: number;
  readonly stdout: string;
  readonly stderr: string;
} {
  const result = spawnSync(
    "cargo",
    [
      "run",
      "--manifest-path",
      "rust/Cargo.toml",
      "-p",
      "engine-shadow-runner",
      "--quiet",
      "--",
      "omena-checker-streaming-ifds-settle-soak",
    ],
    {
      cwd: process.cwd(),
      encoding: "utf8",
      input: JSON.stringify(input),
      maxBuffer: 1024 * 1024 * 10,
    },
  );
  return {
    status: result.status ?? 1,
    stdout: result.stdout,
    stderr: result.stderr,
  };
}

function runStreamingEvaluationWithRepeatedRevisions(
  settleRevisions: readonly unknown[],
): RunnerSummary {
  const input = {
    updateId: "streaming-ifds-settle-soak-check",
    startNodeId: "a",
    demandTargetNodeIds: ["c"],
    factKeyGateVerdict: artifactVerdict(BOUNDARY_PRODUCT),
    deletionCorpusVerdict: artifactVerdict(BOUNDARY_PRODUCT),
    complexitySlopeVerdict: artifactVerdict(SLOPE_PRODUCT),
    hyperedges: [
      { hyperedgeId: "edge-a-b", from: "a", to: "b", edgeKind: "lessImport" },
      { hyperedgeId: "edge-b-c", from: "b", to: "c", edgeKind: "lessModuleGraphClosure" },
    ],
    events: [
      {
        eventId: "event-a",
        revision: 2,
        nodeId: "a",
        value: { kind: "exact", value: "button" },
      },
    ],
    previousFactKeys: ["a|exact:button", "b|exact:button", "c|exact:button"],
    settleRevisions,
  };
  const result = spawnSync(
    "cargo",
    [
      "run",
      "--manifest-path",
      "rust/Cargo.toml",
      "-p",
      "engine-shadow-runner",
      "--quiet",
      "--",
      "omena-checker-streaming-ifds-evaluations",
    ],
    {
      cwd: process.cwd(),
      encoding: "utf8",
      input: JSON.stringify(input),
      maxBuffer: 1024 * 1024 * 10,
    },
  );
  assert.equal(
    result.status,
    0,
    `streaming IFDS serving-mode runner failed\nstdout=${result.stdout}\nstderr=${result.stderr}`,
  );
  return parseJson<RunnerSummary>(result.stdout, "streaming IFDS serving-mode summary");
}

function assertGreenSettleSoak(report: SettleSoakReport): void {
  assert.equal(report.product, SETTLE_SOAK_PRODUCT);
  assert.ok(report.minRevisionCount >= 4, "settle soak must require at least four revisions");
  assert.ok(
    report.requestedRevisionCount >= report.minRevisionCount,
    "settle soak must request enough revisions",
  );
  assert.ok(
    report.distinctRevisionCount >= report.minRevisionCount,
    "settle soak must satisfy the distinct revision floor",
  );
  assert.equal(report.divergenceCount, 0);
  assert.equal(report.consecutiveEqualCount, report.requestedRevisionCount);
  assert.equal(report.allRevisionsEqual, true);
  assert.equal(report.hasInSccEdgeRemoval, true);
  assert.ok(report.revisions.length >= report.minRevisionCount);
  assert.ok(
    report.revisions.some((revision) => revision.hasInSccEdgeRemoval),
    "settle soak must exercise an in-SCC edge-removal revision",
  );
  const digests = new Set<string>();
  for (const revision of report.revisions) {
    assert.equal(revision.equal, true);
    assert.match(revision.contentDigest, /^[0-9a-f]{64}$/);
    digests.add(revision.contentDigest);
  }
  assert.equal(digests.size, report.revisions.length);
}

function artifactVerdict(sourceProduct: string): {
  readonly green: true;
  readonly sourceProduct: string;
  readonly artifactSha256: string;
} {
  return {
    green: true,
    sourceProduct,
    artifactSha256: VALID_SHA256,
  };
}

function repeatedSettleRevision(): Record<string, unknown> {
  return {
    revisionId: "same-input",
    startNodeIds: ["a"],
    targetNodeIds: ["c"],
    hyperedges: [
      { hyperedgeId: "edge-a-b", from: "a", to: "b", edgeKind: "composesLocal" },
      { hyperedgeId: "edge-b-c", from: "b", to: "c", edgeKind: "composesLocal" },
    ],
    events: [
      {
        eventId: "event-a",
        revision: 1,
        nodeId: "a",
        value: { kind: "exact", value: "button" },
      },
    ],
    hasInSccEdgeRemoval: false,
  };
}

function parseJson<T>(source: string, label: string): T {
  try {
    return JSON.parse(source) as T;
  } catch (error) {
    throw new Error(`failed to parse ${label}: ${(error as Error).message}\n${source}`);
  }
}
