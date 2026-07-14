import { strict as assert } from "node:assert";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { existsSync, readdirSync, readFileSync, statSync } from "node:fs";
import { join, relative, sep } from "node:path";

const BOUNDARY_PRODUCT = "omena-diff-test.boundary";
const SLOPE_PRODUCT = "omena-benchmarks.z5-perf-complexity-slope";
const SETTLE_SOAK_PRODUCT = "omena-streaming-ifds.settle-soak-report";
const GATE_PRODUCT = "omena-streaming-ifds.relocation-gate";
const SANCTIONED_DEMAND_SWITCHES = [
  {
    file: "rust/crates/engine-shadow-runner/src/main.rs",
    argument: "demand_route_green",
  },
] as const;

interface BoundarySummary {
  readonly product: string;
  readonly allReachabilityFactKeysFourWayEqual: boolean;
  readonly deletionStaleReuseAllDemandProjectedEqual: boolean;
  readonly deletionStaleReuseReadyForRelocationConsumer: boolean;
}

interface SlopeReport {
  readonly product: string;
  readonly comparisons: readonly {
    readonly lane: string;
    readonly multiplier: number;
    readonly threshold: number;
  }[];
}

interface GateArtifactVerdict {
  readonly green: boolean;
  readonly sourceProduct: string;
  readonly artifactSha256: string;
}

interface RelocationApprovalArtifact {
  readonly product: string;
  readonly boundary: {
    readonly source: string;
    readonly product: string;
    readonly artifactSha256: string;
  };
  readonly slope: {
    readonly source: string;
    readonly product: string;
    readonly artifactSha256: string;
  };
  readonly settle: {
    readonly source: string;
    readonly product: string;
    readonly artifactSha256: string;
  };
  readonly conjuncts: {
    readonly factKeyGateGreen: boolean;
    readonly deletionCorpusGreen: boolean;
    readonly complexitySlopeGreen: boolean;
    readonly settleAllEqual: boolean;
  };
  readonly switchAuthorization: {
    readonly sanctionedCount: number;
    readonly unsanctionedCount: number;
  };
  readonly verdictKind: string;
}

interface SettleSoakReport {
  readonly product: string;
  readonly requestedRevisionCount: number;
  readonly minRevisionCount: number;
  readonly distinctRevisionCount: number;
  readonly divergenceCount: number;
  readonly allRevisionsEqual: boolean;
  readonly hasInSccEdgeRemoval: boolean;
}

interface RunnerSummary {
  readonly demandFactKeyGateGreen: boolean;
  readonly demandFactKeyGateSourceProduct: string;
  readonly demandFactKeyGateArtifactSha256: string;
  readonly demandFactKeyGateRefusal: string | null;
  readonly demandDeletionCorpusGreen: boolean;
  readonly demandDeletionCorpusSourceProduct: string;
  readonly demandDeletionCorpusArtifactSha256: string;
  readonly demandDeletionCorpusRefusal: string | null;
  readonly demandComplexitySlopeGreen: boolean;
  readonly demandComplexitySlopeSourceProduct: string;
  readonly demandComplexitySlopeArtifactSha256: string;
  readonly demandComplexitySlopeRefusal: string | null;
  readonly demandRelocationApprovalGreen: boolean;
  readonly demandRelocationApprovalSourceProduct: string;
  readonly demandRelocationApprovalArtifactSha256: string;
  readonly demandRelocationApprovalRefusal: string | null;
  readonly demandRouteEquivalenceProduct: string;
  readonly demandRouteComparisonKind: string;
  readonly demandRouteDemandFactKeyCount: number;
  readonly demandRouteEagerFactKeyCount: number;
  readonly demandRouteDemandFactKeySha256: string;
  readonly demandRouteEagerFactKeySha256: string;
  readonly demandRouteEquivalentToEager: boolean;
  readonly demandRouteRefusal: string | null;
  readonly demandSettleRequestedCount: number;
  readonly demandSettleDistinctRevisionCount: number;
  readonly demandSettleMinRevisionCount: number;
  readonly demandSettleHasInSccEdgeRemoval: boolean;
  readonly demandSettleAllEqual: boolean;
  readonly demandPrimaryReady: boolean;
  readonly factKeyRouteEngine: string;
  readonly factKeyRouteRelocationGateGreen: boolean;
}

const summaryPath = flagValue("--summary-path");
const slopeReportPath = flagValue("--slope-report-path");
const approvalReportPath = flagValue("--approval-report-path");
const requireSlope = process.argv.includes("--require-slope");
const injectDigestMismatch = process.argv.includes("--inject-digest-mismatch");
const injectSwitchCensusLiteral = process.argv.includes("--inject-switch-census-literal");
const injectSwitchCensusVariable = process.argv.includes("--inject-switch-census-variable");

const boundaryArtifact = summaryPath
  ? readArtifact(summaryPath, "override")
  : runBoundaryArtifact();
const boundarySummary = parseJson<BoundarySummary>(boundaryArtifact.bytes, "boundary summary");
assert.equal(boundarySummary.product, BOUNDARY_PRODUCT);

const boundaryDigest = sha256(boundaryArtifact.bytes);
const factKeyDigest = injectDigestMismatch ? "0".repeat(64) : boundaryDigest;
const factKeyVerdict = artifactVerdict(
  boundarySummary.allReachabilityFactKeysFourWayEqual,
  BOUNDARY_PRODUCT,
  factKeyDigest,
);
const deletionVerdict = artifactVerdict(
  boundarySummary.deletionStaleReuseReadyForRelocationConsumer &&
    boundarySummary.deletionStaleReuseAllDemandProjectedEqual,
  BOUNDARY_PRODUCT,
  boundaryDigest,
);
assert.ok(factKeyVerdict.green, "fact-key boundary artifact must be green");
assert.ok(deletionVerdict.green, "deletion corpus boundary artifact must be green");
assert.equal(factKeyVerdict.artifactSha256, boundaryDigest);
assert.equal(deletionVerdict.artifactSha256, boundaryDigest);

const slopeArtifact = slopeReportPath ? readArtifact(slopeReportPath, "slope-report") : undefined;
if (requireSlope && !slopeArtifact) {
  throw new Error("slope report is required for bound relocation gate mode");
}
const slopeVerdict = slopeArtifact ? slopeArtifactVerdict(slopeArtifact.bytes) : undefined;
const settleArtifact = runSettleSoakArtifact();
const settleReport = parseJson<SettleSoakReport>(settleArtifact.bytes, "settle soak report");
assert.equal(settleReport.product, SETTLE_SOAK_PRODUCT);
assert.ok(settleReport.allRevisionsEqual, "settle soak artifact must be green");
assert.ok(
  settleReport.distinctRevisionCount >= settleReport.minRevisionCount,
  "settle soak artifact must satisfy the distinct revision floor",
);
assert.ok(
  settleReport.hasInSccEdgeRemoval,
  "settle soak artifact must include an in-SCC edge-removal revision",
);
const settleArtifactSha256 = sha256(settleArtifact.bytes);
const switchCensus = collectSwitchAuthorizationCensus({
  injectLiteral: injectSwitchCensusLiteral,
  injectVariable: injectSwitchCensusVariable,
});
assert.equal(
  switchCensus.sanctioned.length,
  SANCTIONED_DEMAND_SWITCHES.length,
  `expected ${SANCTIONED_DEMAND_SWITCHES.length} sanctioned demand-primary switch(es), got ${switchCensus.sanctioned.length}`,
);
assert.equal(
  switchCensus.unsanctioned.length,
  0,
  `unsanctioned demand-primary switch callers: ${switchCensus.unsanctioned
    .map((call) => `${call.file}:${call.line}:${call.argument}`)
    .join(", ")}`,
);

const approvalArtifact =
  approvalReportPath && existsSync(approvalReportPath)
    ? readArtifact(approvalReportPath, "approval-report")
    : undefined;
const relocationApprovalVerdict = approvalArtifact
  ? relocationApprovalArtifactVerdict(approvalArtifact.bytes, {
      boundaryArtifactSha256: boundaryDigest,
      slopeArtifactSha256: slopeVerdict?.artifactSha256,
      settleArtifactSha256,
    })
  : undefined;

const runnerSummary = runRunner({
  factKeyGateVerdict: factKeyVerdict,
  deletionCorpusVerdict: deletionVerdict,
  ...(slopeVerdict ? { complexitySlopeVerdict: slopeVerdict } : {}),
  ...(relocationApprovalVerdict ? { relocationApprovalVerdict } : {}),
});

assertRunnerEcho(
  runnerSummary,
  factKeyVerdict,
  deletionVerdict,
  slopeVerdict,
  relocationApprovalVerdict,
);
assert.equal(runnerSummary.demandFactKeyGateGreen, factKeyVerdict.green);
assert.equal(runnerSummary.demandDeletionCorpusGreen, deletionVerdict.green);
assert.equal(runnerSummary.demandComplexitySlopeGreen, slopeVerdict?.green ?? false);
assert.equal(
  runnerSummary.demandRelocationApprovalGreen,
  relocationApprovalVerdict?.green ?? false,
);
assert.equal(runnerSummary.demandSettleRequestedCount, settleReport.requestedRevisionCount);
assert.equal(runnerSummary.demandSettleDistinctRevisionCount, settleReport.distinctRevisionCount);
assert.equal(runnerSummary.demandSettleMinRevisionCount, settleReport.minRevisionCount);
assert.equal(runnerSummary.demandSettleHasInSccEdgeRemoval, settleReport.hasInSccEdgeRemoval);
assert.equal(runnerSummary.demandSettleAllEqual, settleReport.allRevisionsEqual);
assert.equal(
  runnerSummary.demandRouteEquivalenceProduct,
  "omena-streaming-ifds.demand-eager-equivalence",
);
assert.equal(runnerSummary.demandRouteComparisonKind, "demandVsIndependentProjectedBatch");
assert.equal(runnerSummary.demandRouteEquivalentToEager, true);
assert.equal(
  runnerSummary.demandRouteDemandFactKeyCount,
  runnerSummary.demandRouteEagerFactKeyCount,
);
assert.equal(
  runnerSummary.demandRouteDemandFactKeySha256,
  runnerSummary.demandRouteEagerFactKeySha256,
);

const redRunnerSummary = runRunner({
  factKeyGateVerdict: artifactVerdict(false, BOUNDARY_PRODUCT, boundaryDigest),
  deletionCorpusVerdict: deletionVerdict,
  ...(slopeVerdict ? { complexitySlopeVerdict: slopeVerdict } : {}),
  ...(relocationApprovalVerdict ? { relocationApprovalVerdict } : {}),
});
assert.equal(redRunnerSummary.demandFactKeyGateGreen, false);
assert.equal(redRunnerSummary.demandPrimaryReady, false);
const boundaryAuthoritative = boundaryArtifact.source === "boundary-binary";

if (summaryPath) {
  assert.notEqual(
    factKeyVerdict.green,
    undefined,
    "summary override must still pass through the same derivation path",
  );
}

if (boundaryArtifact.source === "boundary-binary" && boundaryArtifact.exitCode !== 0) {
  throw new Error(
    `boundary summary parsed but producer exited red: exitCode=${boundaryArtifact.exitCode}`,
  );
}

const gateSummary = {
  schemaVersion: "0",
  product: GATE_PRODUCT,
  boundary: {
    source: boundaryArtifact.source,
    product: BOUNDARY_PRODUCT,
    artifactSha256: boundaryDigest,
  },
  slope: slopeArtifact
    ? {
        source: slopeArtifact.source,
        product: SLOPE_PRODUCT,
        artifactSha256: slopeVerdict?.artifactSha256,
      }
    : {
        source: "absent",
        product: SLOPE_PRODUCT,
        artifactSha256: "",
      },
  settle: {
    source: settleArtifact.source,
    product: SETTLE_SOAK_PRODUCT,
    artifactSha256: settleArtifactSha256,
  },
  approval: approvalArtifact
    ? {
        source: approvalArtifact.source,
        product: relocationApprovalVerdict?.sourceProduct ?? "",
        artifactSha256: relocationApprovalVerdict?.artifactSha256 ?? "",
        green: runnerSummary.demandRelocationApprovalGreen,
        refusal: runnerSummary.demandRelocationApprovalRefusal,
      }
    : {
        source: "absent",
        product: GATE_PRODUCT,
        artifactSha256: "",
        green: false,
        refusal: runnerSummary.demandRelocationApprovalRefusal,
      },
  conjuncts: {
    factKeyGateGreen: factKeyVerdict.green,
    deletionCorpusGreen: deletionVerdict.green,
    complexitySlopeGreen: slopeVerdict?.green ?? false,
    settleAllEqual: settleReport.allRevisionsEqual,
  },
  switchAuthorization: {
    sanctionedCount: switchCensus.sanctioned.length,
    unsanctionedCount: switchCensus.unsanctioned.length,
    sanctionedFiles: switchCensus.sanctioned.map((call) => call.file),
  },
  demandPrimaryReady: runnerSummary.demandPrimaryReady,
  route: {
    engine: runnerSummary.factKeyRouteEngine,
    relocationGateGreen: runnerSummary.factKeyRouteRelocationGateGreen,
  },
  equivalence: {
    product: runnerSummary.demandRouteEquivalenceProduct,
    comparisonKind: runnerSummary.demandRouteComparisonKind,
    demandFactKeyCount: runnerSummary.demandRouteDemandFactKeyCount,
    eagerFactKeyCount: runnerSummary.demandRouteEagerFactKeyCount,
    demandFactKeySha256: runnerSummary.demandRouteDemandFactKeySha256,
    eagerFactKeySha256: runnerSummary.demandRouteEagerFactKeySha256,
    equivalent: runnerSummary.demandRouteEquivalentToEager,
  },
  verdictKind: slopeVerdict && boundaryAuthoritative ? "bound" : "partial",
};

console.log(JSON.stringify(gateSummary, null, 2));

function flagValue(name: string): string | undefined {
  const index = process.argv.indexOf(name);
  if (index === -1) return undefined;
  const value = process.argv[index + 1];
  assert.ok(value && !value.startsWith("--"), `${name} requires a value`);
  return value;
}

function runBoundaryArtifact(): {
  readonly bytes: string;
  readonly source: string;
  readonly exitCode: number;
} {
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
      cwd: process.cwd(),
      encoding: "utf8",
      maxBuffer: 1024 * 1024 * 20,
    },
  );
  if (!result.stdout.trim()) {
    throw new Error(
      `boundary summary producer emitted no JSON\nstatus=${result.status}\nstderr=${result.stderr}`,
    );
  }
  return {
    bytes: result.stdout,
    source: "boundary-binary",
    exitCode: result.status ?? 1,
  };
}

function readArtifact(
  path: string,
  source: string,
): { readonly bytes: string; readonly source: string } {
  return {
    bytes: readFileSync(path, "utf8"),
    source,
  };
}

function runSettleSoakArtifact(): { readonly bytes: string; readonly source: string } {
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
      input: JSON.stringify({ settleMode: "soak" }),
      maxBuffer: 1024 * 1024 * 10,
    },
  );
  assert.equal(
    result.status,
    0,
    `engine-shadow-runner settle soak command failed\nstdout=${result.stdout}\nstderr=${result.stderr}`,
  );
  return {
    bytes: result.stdout,
    source: "engine-shadow-runner",
  };
}

function slopeArtifactVerdict(bytes: string): GateArtifactVerdict {
  const report = parseJson<SlopeReport>(bytes, "slope report");
  assert.equal(report.product, SLOPE_PRODUCT);
  assert.ok(report.comparisons.length > 0, "slope report must contain comparisons");
  for (const comparison of report.comparisons) {
    assert.ok(
      Number.isFinite(comparison.multiplier) && Number.isFinite(comparison.threshold),
      `slope comparison ${comparison.lane} must carry numeric multiplier and threshold`,
    );
    assert.ok(
      comparison.multiplier <= comparison.threshold,
      `${comparison.lane} exceeded threshold: ${comparison.multiplier} > ${comparison.threshold}`,
    );
  }
  return artifactVerdict(true, SLOPE_PRODUCT, sha256(bytes));
}

function relocationApprovalArtifactVerdict(
  bytes: string,
  currentArtifacts: {
    readonly boundaryArtifactSha256: string;
    readonly slopeArtifactSha256: string | undefined;
    readonly settleArtifactSha256: string;
  },
): GateArtifactVerdict {
  let artifact: RelocationApprovalArtifact | undefined;
  try {
    artifact = JSON.parse(bytes) as RelocationApprovalArtifact;
  } catch {
    return artifactVerdict(false, "", sha256(bytes));
  }
  const conjuncts = artifact.conjuncts;
  const approved =
    artifact.product === GATE_PRODUCT &&
    artifact.boundary?.source === "boundary-binary" &&
    artifact.boundary.product === BOUNDARY_PRODUCT &&
    artifact.boundary.artifactSha256 === currentArtifacts.boundaryArtifactSha256 &&
    artifact.slope?.source === "slope-report" &&
    artifact.slope.product === SLOPE_PRODUCT &&
    artifact.slope.artifactSha256 === currentArtifacts.slopeArtifactSha256 &&
    artifact.settle?.source === "engine-shadow-runner" &&
    artifact.settle.product === SETTLE_SOAK_PRODUCT &&
    artifact.settle.artifactSha256 === currentArtifacts.settleArtifactSha256 &&
    artifact.verdictKind === "bound" &&
    conjuncts?.factKeyGateGreen === true &&
    conjuncts.deletionCorpusGreen === true &&
    conjuncts.complexitySlopeGreen === true &&
    conjuncts.settleAllEqual === true &&
    artifact.switchAuthorization?.sanctionedCount === SANCTIONED_DEMAND_SWITCHES.length &&
    artifact.switchAuthorization.unsanctionedCount === 0;
  return artifactVerdict(approved, artifact.product ?? "", sha256(bytes));
}

function artifactVerdict(
  green: boolean,
  sourceProduct: string,
  artifactSha256: string,
): GateArtifactVerdict {
  return {
    green,
    sourceProduct,
    artifactSha256,
  };
}

function runRunner(inputVerdicts: {
  readonly factKeyGateVerdict: GateArtifactVerdict;
  readonly deletionCorpusVerdict: GateArtifactVerdict;
  readonly complexitySlopeVerdict?: GateArtifactVerdict;
  readonly relocationApprovalVerdict?: GateArtifactVerdict;
}): RunnerSummary {
  const input = {
    updateId: "streaming-ifds-relocation-gate",
    startNodeId: "a",
    demandTargetNodeIds: ["b"],
    ...inputVerdicts,
    hyperedges: [
      { hyperedgeId: "edge-a-b", from: "a", to: "b", edgeKind: "lessImport" },
      {
        hyperedgeId: "edge-b-c",
        from: "b",
        to: "c",
        edgeKind: "lessModuleGraphClosure",
      },
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
    `engine-shadow-runner streaming IFDS command failed\nstdout=${result.stdout}\nstderr=${result.stderr}`,
  );
  return parseJson<RunnerSummary>(result.stdout, "runner summary");
}

function assertRunnerEcho(
  summary: RunnerSummary,
  factKeyVerdict: GateArtifactVerdict,
  deletionVerdict: GateArtifactVerdict,
  slopeVerdict: GateArtifactVerdict | undefined,
  relocationApprovalVerdict: GateArtifactVerdict | undefined,
): void {
  assert.equal(summary.demandFactKeyGateSourceProduct, factKeyVerdict.sourceProduct);
  assert.equal(summary.demandFactKeyGateArtifactSha256, factKeyVerdict.artifactSha256);
  assert.equal(summary.demandDeletionCorpusSourceProduct, deletionVerdict.sourceProduct);
  assert.equal(summary.demandDeletionCorpusArtifactSha256, deletionVerdict.artifactSha256);
  if (slopeVerdict) {
    assert.equal(summary.demandComplexitySlopeSourceProduct, slopeVerdict.sourceProduct);
    assert.equal(summary.demandComplexitySlopeArtifactSha256, slopeVerdict.artifactSha256);
    assert.equal(summary.demandComplexitySlopeRefusal, null);
  } else {
    assert.equal(summary.demandComplexitySlopeGreen, false);
    assert.equal(summary.demandComplexitySlopeRefusal, "absent artifact verdict");
  }
  if (relocationApprovalVerdict) {
    assert.equal(
      summary.demandRelocationApprovalSourceProduct,
      relocationApprovalVerdict.sourceProduct,
    );
    assert.equal(
      summary.demandRelocationApprovalArtifactSha256,
      relocationApprovalVerdict.artifactSha256,
    );
  } else {
    assert.equal(summary.demandRelocationApprovalGreen, false);
    assert.equal(summary.demandRelocationApprovalRefusal, "absent artifact verdict");
  }
}

function parseJson<T>(source: string, label: string): T {
  try {
    return JSON.parse(source) as T;
  } catch (error) {
    throw new Error(`failed to parse ${label}: ${(error as Error).message}`);
  }
}

function sha256(source: string): string {
  return createHash("sha256").update(source).digest("hex");
}

interface RouteCall {
  readonly file: string;
  readonly line: number;
  readonly argument: string;
}

function collectSwitchAuthorizationCensus(options: {
  readonly injectLiteral: boolean;
  readonly injectVariable: boolean;
}): { readonly sanctioned: readonly RouteCall[]; readonly unsanctioned: readonly RouteCall[] } {
  const calls = rustSources()
    .flatMap((source) => routeCallsInSource(source.file, source.text))
    .concat(injectedRouteCalls(options));
  const sanctioned = calls.filter(isSanctionedDemandSwitch);
  const unsanctioned = calls.filter(
    (call) => normalizeCallArgument(call.argument) !== "false" && !isSanctionedDemandSwitch(call),
  );
  assert.ok(
    sanctioned.some((call) => call.file === "rust/crates/engine-shadow-runner/src/main.rs"),
    "runner readiness pass-through must be present in the switch census",
  );
  return { sanctioned, unsanctioned };
}

function rustSources(): { readonly file: string; readonly text: string }[] {
  const root = join(process.cwd(), "rust", "crates");
  return collectRustFiles(root).map((path) => {
    const file = relative(process.cwd(), path).split(sep).join("/");
    return {
      file,
      text: productionRustSource(readFileSync(path, "utf8")),
    };
  });
}

function collectRustFiles(directory: string): string[] {
  return readdirSync(directory)
    .flatMap((entry) => {
      const path = join(directory, entry);
      const relativePath = relative(process.cwd(), path).split(sep).join("/");
      if (
        relativePath.includes("/target/") ||
        relativePath.includes("/tests/") ||
        relativePath.includes("/benches/") ||
        relativePath.includes("/examples/")
      ) {
        return [];
      }
      const stat = statSync(path);
      if (stat.isDirectory()) return collectRustFiles(path);
      return path.endsWith(".rs") ? [path] : [];
    })
    .sort();
}

function productionRustSource(text: string): string {
  const testModule = /#\[cfg\(test\)\]\s*\n\s*mod\s+tests\s*\{/u.exec(text);
  return testModule?.index === undefined ? text : text.slice(0, testModule.index);
}

function routeCallsInSource(file: string, text: string): RouteCall[] {
  const calls: RouteCall[] = [];
  const needle = "streaming_ifds_fact_key_route_with_gate_v0(";
  let index = text.indexOf(needle);
  while (index !== -1) {
    const before = text.slice(Math.max(0, index - 16), index);
    if (!/\bfn\s*$/.test(before)) {
      const callEnd = findMatchingParen(text, index + needle.length - 1);
      const argumentSource = text.slice(index + needle.length, callEnd);
      const args = splitTopLevelArguments(argumentSource);
      if (args.length >= 2) {
        calls.push({
          file,
          line: text.slice(0, index).split("\n").length,
          argument: args[1],
        });
      }
    }
    index = text.indexOf(needle, index + needle.length);
  }
  return calls;
}

function injectedRouteCalls(options: {
  readonly injectLiteral: boolean;
  readonly injectVariable: boolean;
}): RouteCall[] {
  const calls: RouteCall[] = [];
  if (options.injectLiteral) {
    calls.push({
      file: "synthetic/production-switch-literal.rs",
      line: 1,
      argument: "true",
    });
  }
  if (options.injectVariable) {
    calls.push({
      file: "synthetic/production-switch-variable.rs",
      line: 1,
      argument: "external_gate",
    });
  }
  return calls;
}

function isSanctionedDemandSwitch(call: RouteCall): boolean {
  const argument = normalizeCallArgument(call.argument);
  return SANCTIONED_DEMAND_SWITCHES.some(
    (sanctioned) => sanctioned.file === call.file && sanctioned.argument === argument,
  );
}

function normalizeCallArgument(argument: string): string {
  return argument.replace(/\s+/g, "");
}

function findMatchingParen(source: string, openIndex: number): number {
  let depth = 0;
  let quote: string | undefined;
  for (let index = openIndex; index < source.length; index += 1) {
    const char = source[index];
    const previous = source[index - 1];
    if (quote) {
      if (char === quote && previous !== "\\") quote = undefined;
      continue;
    }
    if (char === '"' || char === "'" || char === "`") {
      quote = char;
      continue;
    }
    if (char === "(") depth += 1;
    if (char === ")") {
      depth -= 1;
      if (depth === 0) return index;
    }
  }
  throw new Error("failed to parse route_with_gate call");
}

function splitTopLevelArguments(source: string): string[] {
  const args: string[] = [];
  let depth = 0;
  let quote: string | undefined;
  let start = 0;
  for (let index = 0; index < source.length; index += 1) {
    const char = source[index];
    const previous = source[index - 1];
    if (quote) {
      if (char === quote && previous !== "\\") quote = undefined;
      continue;
    }
    if (char === '"' || char === "'" || char === "`") {
      quote = char;
      continue;
    }
    if (char === "(" || char === "[" || char === "{") depth += 1;
    if (char === ")" || char === "]" || char === "}") depth -= 1;
    if (char === "," && depth === 0) {
      args.push(source.slice(start, index).trim());
      start = index + 1;
    }
  }
  args.push(source.slice(start).trim());
  return args;
}
