import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import path from "node:path";
import { spawnSync } from "node:child_process";

import { maskRustCfgTestItems } from "./lib/rust-cfg-test-mask";

interface FixtureResultV0 {
  readonly name: string;
  classification:
    | "axisWinnerInexact"
    | "noStrictAxisDominance"
    | "singleInexactCandidate"
    | {
        readonly recoverableAxisDominant: {
          readonly axis: string;
        };
      };
}

interface CascadeKeyAxisOrderArtifactV0 {
  readonly schemaVersion: "0";
  readonly product: "omena-cascade.key-axis-order";
  readonly axisOrder: readonly string[];
  readonly rankedSetPrefixAxisVocabulary: readonly string[];
}

type CascadeKeyProducerDispositionV0 =
  | "automaticProductDerived"
  | "callerSuppliedBoundary"
  | "conformance"
  | "generated"
  | "fixture";

type CascadeScopeProximitySourceV0 =
  | "constantZero"
  | "legacySelectorContextFallback"
  | "callerSupplied"
  | "generatedValue";

interface CascadeKeyProducerV0 {
  readonly path: string;
  readonly symbol: string;
  readonly occurrence: number;
  readonly disposition: CascadeKeyProducerDispositionV0;
  readonly scopeProximitySource: CascadeScopeProximitySourceV0;
}

interface CascadeDriverCensusArtifactV0 {
  readonly schemaVersion: "0";
  readonly product: "omena-cascade.driver-census";
  readonly winnerAxes: readonly {
    readonly axis: string;
    readonly status: "driven" | "automaticProductDriver";
    readonly namedDriver?: "legacySelectorContextFallback";
  }[];
  readonly cascadeKeyProducers: readonly CascadeKeyProducerV0[];
  readonly specAxisReach: {
    readonly originAndImportance: Readonly<Record<string, string>>;
    readonly encapsulationContext: Readonly<Record<string, string>>;
    readonly styleAttribute: Readonly<Record<string, string>>;
    readonly layers: Readonly<Record<string, string>>;
    readonly specificity: Readonly<Record<string, string>>;
    readonly scopeProximity: Readonly<Record<string, string>>;
    readonly orderOfAppearance: Readonly<Record<string, string>>;
  };
}

interface RankedSetLossCensusArtifactV0 {
  readonly schemaVersion: "0";
  readonly product: "omena-diff-test.ranked-set-loss-census";
  readonly limitations: readonly string[];
  readonly entryCount: number;
  readonly captureStateRecoveryCount: number;
  readonly measurementInvocationCount: number;
  readonly rankedSetOutcomeCount: number;
  readonly multiCandidateInexactRankedSetCount: number;
  readonly layerSyntaxFileCount: number;
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
    readonly captureStateRecoveryCount: number;
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

const repoRoot = process.cwd();
let currentTrackedFiles: readonly string[] | undefined;
const repositorySourceCache = new Map<string, string>();
const artifactPath = path.join(
  repoRoot,
  "rust/crates/omena-diff-test/oss-corpus-farm/ranked-set-loss-census.json",
);
const axisOrderArtifactPath = path.join(
  repoRoot,
  "rust/crates/omena-cascade/data/cascade-key-axis-order.json",
);
const driverCensusArtifactPath = path.join(
  repoRoot,
  "rust/crates/omena-cascade/data/cascade-driver-census.json",
);
const args = new Set(process.argv.slice(2));
const sourceRef = valueAfter("--source-ref");

if (args.has("--discover-cascade-key-producers")) {
  process.stdout.write(
    `${JSON.stringify(
      discoverCascadeKeyProducers(readCascadeKeyProducerSources(sourceRef)),
      null,
      2,
    )}\n`,
  );
  process.exit(0);
}

if (args.has("--discover-axis-order-sites")) {
  process.stdout.write(
    `${JSON.stringify(discoverAxisOrderSites(readCascadeAxisOrderDomain(sourceRef)), null, 2)}\n`,
  );
  process.exit(0);
}

if (sourceRef !== undefined) {
  process.stdout.write(`${JSON.stringify(cascadeAxisOrderSiteCensus(sourceRef), null, 2)}\n`);
  process.exit(0);
}

const cascadeKeyProducerCensus = validateCascadeKeyProducerCensus();

const emittedAxisOrder = spawnSync(
  "cargo",
  [
    "run",
    "--quiet",
    "--manifest-path",
    "rust/Cargo.toml",
    "-p",
    "omena-cascade",
    "--example",
    "cascade_key_axis_order",
  ],
  {
    cwd: repoRoot,
    encoding: "utf8",
    maxBuffer: 1024 * 1024 * 4,
  },
);
assert.equal(
  emittedAxisOrder.status,
  0,
  `cascade key axis-order emitter failed:\n${emittedAxisOrder.stderr}`,
);
const axisOrderArtifact = JSON.parse(
  readFileSync(axisOrderArtifactPath, "utf8"),
) as CascadeKeyAxisOrderArtifactV0;
assert.deepEqual(
  JSON.parse(emittedAxisOrder.stdout),
  axisOrderArtifact,
  "committed cascade key axis order must equal the Rust-emitted authority projection",
);
assert.equal(axisOrderArtifact.schemaVersion, "0");
assert.equal(axisOrderArtifact.product, "omena-cascade.key-axis-order");
const specificityStart = axisOrderArtifact.axisOrder.indexOf("specificityIds");
assert.ok(specificityStart > 0, "cascade key axis order must include specificityIds");
const strictAxisPrefix = axisOrderArtifact.axisOrder.slice(0, specificityStart);
const axisOrderSiteCensus = cascadeAxisOrderSiteCensus();

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
const multiCandidateRecheckFixture: RankedSetLossCensusRowV0 = {
  declarationIds: ["exact-author", "inexact-user"],
  candidateCount: 2,
  candidates: [
    {
      declarationId: "exact-author",
      level: "authorNormal",
      layerRank: 0,
      scopeProximity: 0,
      specificityExactness: "exact",
    },
    {
      declarationId: "inexact-user",
      level: "userNormal",
      layerRank: 0,
      scopeProximity: 0,
      specificityExactness: "inexact",
    },
  ],
  classification: { recoverableAxisDominant: { axis: "level" } },
};
assert.deepEqual(
  classifyArtifactRow(multiCandidateRecheckFixture),
  multiCandidateRecheckFixture.classification,
  "the committed-row adjudicator must execute the multi-candidate axis-prefix path",
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
assert.equal(
  artifact.rowCount,
  artifactRows.length,
  "entry rows must equal the artifact row count",
);
assert.equal(
  artifact.captureStateRecoveryCount,
  sum(artifact.entries.map((entry) => entry.captureStateRecoveryCount)),
  "capture-state recovery count must be derived from entry captures",
);
assert.equal(
  artifact.captureStateRecoveryCount,
  0,
  "capture-state recovery invalidates the committed census",
);
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
  ...axisOrderArtifact.rankedSetPrefixAxisVocabulary.toSorted(),
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
assert.equal(
  artifact.limitations.some((limitation) =>
    limitation.includes("No multi-candidate inexact RankedSet outcome"),
  ),
  artifact.multiCandidateInexactRankedSetCount === 0,
  "the empty multi-candidate limitation must follow its measured denominator",
);
assert.equal(
  artifact.limitations.some((limitation) =>
    limitation.includes("zero eligible product executions"),
  ),
  artifact.multiCandidateInexactRankedSetCount === 0,
  "the classifier-execution limitation must follow its measured denominator",
);
assert.equal(
  artifact.limitations.some((limitation) => limitation.includes("no @layer")),
  artifact.layerSyntaxFileCount === 0,
  "the cascade-layer limitation must follow its measured style-file denominator",
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
      multiCandidateRecheckFixtureCount: 1,
      layerSyntaxFileCount: artifact.layerSyntaxFileCount,
      rowCount: artifact.rowCount,
      recoverableCount: artifact.recoverableCount,
      undecidableCount: artifact.undecidableCount,
      functionPopulations: artifact.functionPopulations,
      decidingAxisCounts: artifact.decidingAxisCounts,
      axisOrderSiteCount: axisOrderSiteCensus.siteCount,
      axisOrderSiteDispositionCounts: axisOrderSiteCensus.dispositionCounts,
      cascadeKeyProducerCount: cascadeKeyProducerCensus.producerCount,
      cascadeKeyProducerDispositionCounts: cascadeKeyProducerCensus.dispositionCounts,
      cascadeKeyScopeProximitySourceCounts: cascadeKeyProducerCensus.sourceCounts,
      automaticProductScopeDriver: cascadeKeyProducerCensus.automaticProductScopeDriver,
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

function classifyArtifactRow(row: RankedSetLossCensusRowV0): FixtureResultV0["classification"] {
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

  const axis = strictAxisPrefix.find(
    (candidate) => axisValue(winner, candidate) !== axisValue(runnerUp, candidate),
  );
  assert.ok(axis, "strict axis-prefix winner must have a deciding authority axis");
  return { recoverableAxisDominant: { axis } };
}

function compareAxisPrefix(
  left: RankedSetLossCandidateV0,
  right: RankedSetLossCandidateV0,
): -1 | 0 | 1 {
  return compareNumberTuple(
    strictAxisPrefix.map((axis) => axisValue(left, axis)),
    strictAxisPrefix.map((axis) => axisValue(right, axis)),
  );
}

function axisValue(candidate: RankedSetLossCandidateV0, axis: string): number {
  const valueByAxis: Readonly<Record<string, number>> = {
    level: levelRank(candidate.level),
    layerRank: candidate.layerRank,
    scopeProximity: -candidate.scopeProximity,
  };
  const value = valueByAxis[axis];
  assert.notEqual(value, undefined, `ranked-set classifier cannot project axis ${axis}`);
  return value;
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

function valueAfter(flag: string): string | undefined {
  const index = process.argv.indexOf(flag);
  if (index < 0) return undefined;
  const value = process.argv[index + 1];
  assert.ok(value, `${flag} requires a value`);
  return value;
}

function validateCascadeKeyProducerCensus(): {
  readonly producerCount: number;
  readonly dispositionCounts: Readonly<Record<CascadeKeyProducerDispositionV0, number>>;
  readonly sourceCounts: Readonly<Record<CascadeScopeProximitySourceV0, number>>;
  readonly automaticProductScopeDriver: "legacySelectorContextFallback";
} {
  const sources = readCascadeKeyProducerSources();
  if (args.has("--inject-cascade-key-producer")) {
    sources.set(
      "rust/crates/omena-query/src/style/cascade_key_census_probe.rs",
      [
        "fn injected_product_key() -> CascadeKey {",
        "    CascadeKey::new(level, layer_rank, 0, specificity, source_order)",
        "}",
      ].join("\n"),
    );
  }
  if (args.has("--inject-unclassified-caller-supplied-producer")) {
    sources.set(
      "rust/crates/omena-query/src/style/cascade_key_boundary_probe.rs",
      [
        "fn injected_product_key(scope_proximity: u32) -> CascadeKey {",
        "    CascadeKey::new(level, layer_rank, scope_proximity, specificity, source_order)",
        "}",
      ].join("\n"),
    );
  }
  if (args.has("--inject-automatic-scope-proximity-driver")) {
    const producerPath = "rust/crates/omena-query/src/style/cascade_checker/runtime_state.rs";
    const source = sources.get(producerPath);
    assert.ok(source, `missing automatic product producer ${producerPath}`);
    const changed = source.replace(
      "key: CascadeKey::new(level, layer_rank, 0, specificity, input.source_order),",
      "key: CascadeKey::new(level, layer_rank, 1, specificity, input.source_order),",
    );
    assert.notEqual(changed, source, "automatic scope-proximity falsifier needle is stale");
    sources.set(producerPath, changed);
  }
  if (args.has("--inject-producer-classification-drift")) {
    const producerPath = "rust/crates/omena-semantic/src/design_tokens.rs";
    const source = sources.get(producerPath);
    assert.ok(source, `missing named product producer ${producerPath}`);
    const changed = source.replace(
      "cascade_scope_proximity_fallback_for_selector_context_rank(",
      "unclassified_scope_proximity_source(",
    );
    assert.notEqual(changed, source, "producer-classification falsifier needle is stale");
    sources.set(producerPath, changed);
  }
  if (args.has("--inject-cascade-key-producer-removal")) {
    const producerPath = "rust/crates/engine-shadow-runner/src/main.rs";
    const source = sources.get(producerPath);
    assert.ok(source, `missing fixture producer ${producerPath}`);
    const changed = source.replace("key: CascadeKey {", "key: RemovedCascadeRecord {");
    assert.notEqual(changed, source, "producer-removal falsifier needle is stale");
    sources.set(producerPath, changed);
  }

  const discovered = discoverCascadeKeyProducers(sources);
  let artifact = JSON.parse(
    readFileSync(driverCensusArtifactPath, "utf8"),
  ) as CascadeDriverCensusArtifactV0;
  if (args.has("--inject-axis-reach-disclosure-drift")) {
    artifact = {
      ...artifact,
      specAxisReach: {
        ...artifact.specAxisReach,
        scopeProximity: { status: "modeled" },
      },
    };
  }
  assert.equal(artifact.schemaVersion, "0");
  assert.equal(artifact.product, "omena-cascade.driver-census");

  const invalidAutomaticSources = discovered.filter(
    (producer) =>
      producer.disposition === "automaticProductDerived" &&
      producer.scopeProximitySource !== "constantZero" &&
      producer.scopeProximitySource !== "legacySelectorContextFallback",
  );
  assert.deepEqual(
    invalidAutomaticSources,
    [],
    `scopeProximity gained an unclassified automatic product driver: ${invalidAutomaticSources
      .map(cascadeKeyProducerId)
      .join(", ")}`,
  );

  const artifactIds = artifact.cascadeKeyProducers.map(cascadeKeyProducerId);
  const discoveredIds = discovered.map(cascadeKeyProducerId);
  assert.equal(
    new Set(artifactIds).size,
    artifactIds.length,
    "cascade key producer census must not repeat a source site",
  );
  assert.deepEqual(
    discoveredIds.filter((id) => !artifactIds.includes(id)),
    [],
    "source scan found CascadeKey producers missing from the driver census",
  );
  assert.deepEqual(
    artifactIds.filter((id) => !discoveredIds.includes(id)),
    [],
    "driver census contains CascadeKey producers missing from the source scan",
  );
  const discoveredById = new Map(
    discovered.map((producer) => [cascadeKeyProducerId(producer), producer]),
  );
  for (const recorded of artifact.cascadeKeyProducers) {
    const id = cascadeKeyProducerId(recorded);
    assert.deepEqual(
      recorded,
      discoveredById.get(id),
      `CascadeKey producer source classification drifted at ${id}`,
    );
  }

  const automaticFallbacks = discovered.filter(
    (producer) =>
      producer.disposition === "automaticProductDerived" &&
      producer.scopeProximitySource === "legacySelectorContextFallback",
  );
  assert.ok(
    automaticFallbacks.length > 0 &&
      automaticFallbacks.every(
        (producer) => producer.path === "rust/crates/omena-semantic/src/design_tokens.rs",
      ),
    "the named automatic product driver must be the semantic design-token fallback",
  );

  const callerSuppliedIds = new Set(
    discovered
      .filter((producer) => producer.disposition === "callerSuppliedBoundary")
      .map(cascadeKeyProducerId),
  );
  const bundlerCallerId =
    "rust/crates/omena-bundler/src/lib.rs#cascade_key_with_global_source_order:1";
  const transformCallerId =
    "rust/crates/omena-transform-passes/src/runtime/winner_equality.rs#winner_for_pair:2";
  assert.deepEqual(
    [...callerSuppliedIds].toSorted(),
    [bundlerCallerId, transformCallerId].toSorted(),
    "only the validated bundler helper and transform/NAPI environment may be excluded as caller-supplied boundaries",
  );
  validateCallerSuppliedScopeSurfaces(sources);

  const scopeAxis = artifact.winnerAxes.find((entry) => entry.axis === "scopeProximity");
  assert.deepEqual(
    scopeAxis,
    {
      axis: "scopeProximity",
      status: "automaticProductDriver",
      namedDriver: "legacySelectorContextFallback",
    },
    "scopeProximity must name its automatic product driver without excluding caller-supplied surfaces",
  );
  assert.deepEqual(
    artifact.specAxisReach,
    {
      originAndImportance: { status: "modeled" },
      encapsulationContext: {
        status: "outOfFragment",
        reason: "shadowTreeEncapsulationContextUnmodeled",
      },
      styleAttribute: { status: "modeled" },
      layers: { status: "modeled" },
      specificity: { status: "modeled" },
      scopeProximity: {
        status: "notReachedByProduct",
        namedDriver: "legacySelectorContextFallback",
      },
      orderOfAppearance: { status: "modeled" },
    },
    "typed spec-axis reach disclosure must match authority-derived modeled limbs, the producer-derived scope limb, and the declared encapsulation boundary",
  );

  const dispositionCounts: Record<CascadeKeyProducerDispositionV0, number> = {
    automaticProductDerived: 0,
    callerSuppliedBoundary: 0,
    conformance: 0,
    generated: 0,
    fixture: 0,
  };
  const sourceCounts: Record<CascadeScopeProximitySourceV0, number> = {
    constantZero: 0,
    legacySelectorContextFallback: 0,
    callerSupplied: 0,
    generatedValue: 0,
  };
  for (const producer of discovered) {
    dispositionCounts[producer.disposition] += 1;
    sourceCounts[producer.scopeProximitySource] += 1;
  }
  return {
    producerCount: discovered.length,
    dispositionCounts,
    sourceCounts,
    automaticProductScopeDriver: "legacySelectorContextFallback",
  };
}

function validateCallerSuppliedScopeSurfaces(sources: ReadonlyMap<string, string>): void {
  const bundler = sources.get("rust/crates/omena-bundler/src/lib.rs") ?? "";
  assert.match(
    bundler,
    /pub fn cascade_key_with_global_source_order\([\s\S]*?scope_proximity:\s*u32[\s\S]*?CascadeKey::new\([\s\S]*?scope_proximity,/u,
    "bundler public helper must preserve its caller-supplied nonzero scope-proximity input",
  );

  const transformModel = sources.get("rust/crates/omena-transform-passes/src/model.rs") ?? "";
  assert.match(
    transformModel,
    /pub struct TransformCascadeEnvironmentDeclarationV0[\s\S]*?pub scope_proximity:\s*Option<u32>/u,
    "transform cascade environment must retain its caller-supplied scope-proximity field",
  );
  const transformRuntime =
    sources.get("rust/crates/omena-transform-passes/src/runtime/winner_equality.rs") ?? "";
  assert.match(
    transformRuntime,
    /declaration\.scope_proximity\.unwrap_or\(0\)/u,
    "transform winner selection must consume caller-supplied scope proximity",
  );
  const napiBoundary =
    sources.get("rust/crates/omena-napi/src/engine_napi_contract_idl_generated.rs") ?? "";
  assert.match(
    napiBoundary,
    /#\[napi\(js_name = "cascadeEnvironment"\)\][\s\S]*?pub cascade_environment:\s*Option<serde_json::Value>/u,
    "the NAPI boundary must carry the transform cascade environment",
  );
  const napi = sources.get("rust/crates/omena-napi/src/lib.rs") ?? "";
  assert.match(
    napi,
    /OmenaQueryTransformExecutionContextV0 as OmenaNapiTransformExecutionContextV0/u,
    "the NAPI context must deserialize into the typed query transform context",
  );
}

function readCascadeKeyProducerSources(sourceRef?: string): Map<string, string> {
  const sources = new Map<string, string>();
  const listed =
    sourceRef === undefined ? listCurrentTrackedFiles() : listTrackedFilesAt(sourceRef);
  for (const file of listed) {
    if (!file.startsWith("rust/crates/") || !file.endsWith(".rs")) continue;
    sources.set(file, readRepositorySource(file, sourceRef));
  }
  return sources;
}

interface RustFunctionItemV0 {
  readonly name: string;
  readonly open: number;
  readonly header: string;
  readonly body: string;
}

function discoverCascadeKeyProducers(
  sources: ReadonlyMap<string, string>,
): readonly CascadeKeyProducerV0[] {
  const discovered: CascadeKeyProducerV0[] = [];
  for (const [file, rawSource] of sources) {
    if (!isCascadeKeyProducerSourcePath(file)) continue;
    if (!/(?:CascadeKey::new\s*\(|CascadeKey\s*\{)/u.test(rawSource)) continue;
    const source = maskRustCfgTestItems(rawSource);
    if (!/(?:CascadeKey::new\s*\(|CascadeKey\s*\{)/u.test(source)) continue;
    const calls: { index: number; expression: string; functionItem: RustFunctionItemV0 }[] = [];

    for (const match of source.matchAll(/(?:omena_cascade::)?CascadeKey::new\s*\(/gu)) {
      const index = match.index;
      const functionItem = rustEnclosingFunctionAt(source, index);
      assert.ok(functionItem, `CascadeKey constructor is outside a function: ${file}`);
      const open = source.indexOf("(", index);
      const close = matchingRustDelimiter(source, open, "(", ")");
      const callArguments = splitRustTopLevel(source.slice(open + 1, close), ",");
      assert.ok(
        callArguments.length >= 5,
        `CascadeKey constructor has fewer than five arguments: ${file}#${functionItem.name}`,
      );
      calls.push({ index, expression: callArguments[2] ?? "", functionItem });
    }

    for (const match of source.matchAll(/(?:omena_cascade::)?CascadeKey\s*\{/gu)) {
      const index = match.index;
      const functionItem = rustEnclosingFunctionAt(source, index);
      if (functionItem === undefined) continue;
      const prefix = source.slice(Math.max(functionItem.open, index - 40), index);
      if (/(?:struct|impl|for|->)\s*$/u.test(prefix)) continue;
      const open = source.indexOf("{", index);
      const close = matchingRustDelimiter(source, open, "{", "}");
      const fields = source.slice(open + 1, close);
      const expression = rustStructFieldValue(fields, "scope_proximity");
      assert.ok(
        expression !== undefined,
        `CascadeKey literal is missing scope_proximity: ${file}#${functionItem.name}`,
      );
      calls.push({ index, expression, functionItem });
    }

    const occurrences = new Map<string, number>();
    for (const call of calls.toSorted((left, right) => left.index - right.index)) {
      const occurrence = (occurrences.get(call.functionItem.name) ?? 0) + 1;
      occurrences.set(call.functionItem.name, occurrence);
      const scopeProximitySource = classifyCascadeScopeProximitySource(
        call.expression,
        call.functionItem,
      );
      discovered.push({
        path: file,
        symbol: call.functionItem.name,
        occurrence,
        disposition: classifyCascadeKeyProducerDisposition(
          file,
          call.functionItem.name,
          scopeProximitySource,
        ),
        scopeProximitySource,
      });
    }
  }
  return discovered.toSorted((left, right) =>
    cascadeKeyProducerId(left).localeCompare(cascadeKeyProducerId(right)),
  );
}

function isCascadeKeyProducerSourcePath(file: string): boolean {
  return (
    file.startsWith("rust/crates/") &&
    file.endsWith(".rs") &&
    !file.includes("/tests/") &&
    !file.includes("/examples/") &&
    !file.endsWith("/src/tests.rs")
  );
}

function classifyCascadeKeyProducerDisposition(
  file: string,
  symbol: string,
  scopeSource: CascadeScopeProximitySourceV0,
): CascadeKeyProducerDispositionV0 {
  if (file === "rust/crates/omena-cascade/src/conformance.rs") return "conformance";
  if (file === "rust/crates/omena-cascade/src/fuzz.rs" || symbol.startsWith("generated_")) {
    return "generated";
  }
  if (
    file === "rust/crates/engine-shadow-runner/src/main.rs" ||
    file === "rust/crates/omena-categorical/src/lib.rs"
  ) {
    return "fixture";
  }
  if (
    scopeSource === "callerSupplied" &&
    ((file === "rust/crates/omena-bundler/src/lib.rs" &&
      symbol === "cascade_key_with_global_source_order") ||
      (file === "rust/crates/omena-transform-passes/src/runtime/winner_equality.rs" &&
        symbol === "winner_for_pair"))
  ) {
    return "callerSuppliedBoundary";
  }
  return "automaticProductDerived";
}

function classifyCascadeScopeProximitySource(
  expression: string,
  functionItem: RustFunctionItemV0,
): CascadeScopeProximitySourceV0 {
  const normalized = expression.replace(/\s+/gu, "");
  if (/^0(?:_?u32)?$/u.test(normalized)) return "constantZero";
  if (normalized.includes("cascade_scope_proximity_fallback_for_selector_context_rank")) {
    return "legacySelectorContextFallback";
  }
  if (/^[A-Za-z_][A-Za-z0-9_]*$/u.test(normalized)) {
    const assignment = new RegExp(
      `let\\s+(?:mut\\s+)?${escapeRegExp(normalized)}(?:\\s*:[^=;]+)?\\s*=\\s*([^;]+);`,
      "u",
    ).exec(functionItem.body)?.[1];
    if (assignment?.includes("cascade_scope_proximity_fallback_for_selector_context_rank")) {
      return "legacySelectorContextFallback";
    }
    const parameterPattern = new RegExp(`\\b${escapeRegExp(normalized)}\\s*:`, "u");
    if (parameterPattern.test(functionItem.header)) return "callerSupplied";
  }
  if (/\.scope_proximity\b/u.test(normalized)) return "callerSupplied";
  return "generatedValue";
}

function rustEnclosingFunctionAt(source: string, index: number): RustFunctionItemV0 | undefined {
  const pattern =
    /^[ \t]*(?:pub(?:\([^)]*\))?\s+)?(?:const\s+)?(?:async\s+)?fn\s+([A-Za-z_][A-Za-z0-9_]*)/gmu;
  const candidates = [...source.slice(0, index).matchAll(pattern)];
  for (const match of candidates.toReversed()) {
    const open = source.indexOf("{", match.index);
    const semicolon = source.indexOf(";", match.index);
    if (open < 0 || open >= index || (semicolon >= 0 && semicolon < open)) continue;
    return {
      name: match[1] ?? "",
      open,
      header: source.slice(match.index, open),
      body: source.slice(open + 1, index),
    };
  }
  return undefined;
}

function cascadeKeyProducerId(
  producer: Pick<CascadeKeyProducerV0, "path" | "symbol" | "occurrence">,
): string {
  return `${producer.path}#${producer.symbol}:${producer.occurrence}`;
}

function rustStructFieldValue(source: string, field: string): string | undefined {
  for (const part of splitRustTopLevel(source, ",")) {
    const separator = part.indexOf(":");
    if (separator < 0 || part.slice(0, separator).trim() !== field) continue;
    return part.slice(separator + 1).trim();
  }
  return undefined;
}

function splitRustTopLevel(source: string, separator: string): string[] {
  const parts: string[] = [];
  let start = 0;
  const delimiterStack: string[] = [];
  let quote: '"' | "'" | undefined;
  let lineComment = false;
  let blockComment = false;
  for (let index = 0; index < source.length; index += 1) {
    const current = source[index] ?? "";
    const next = source[index + 1] ?? "";
    const previous = source[index - 1] ?? "";
    if (lineComment) {
      if (current === "\n") lineComment = false;
      continue;
    }
    if (blockComment) {
      if (current === "*" && next === "/") {
        blockComment = false;
        index += 1;
      }
      continue;
    }
    if (quote !== undefined) {
      if (current === quote && previous !== "\\") quote = undefined;
      continue;
    }
    if (current === "/" && next === "/") {
      lineComment = true;
      index += 1;
      continue;
    }
    if (current === "/" && next === "*") {
      blockComment = true;
      index += 1;
      continue;
    }
    if (current === '"' || (current === "'" && rustCharLiteralEnd(source, index) > index)) {
      quote = current;
      continue;
    }
    if (current === "(" || current === "[" || current === "{") delimiterStack.push(current);
    if (current === ")" || current === "]" || current === "}") delimiterStack.pop();
    if (current === separator && delimiterStack.length === 0) {
      parts.push(source.slice(start, index).trim());
      start = index + 1;
    }
  }
  parts.push(source.slice(start).trim());
  return parts;
}

function matchingRustDelimiter(
  source: string,
  openIndex: number,
  openDelimiter: string,
  closeDelimiter: string,
): number {
  assert.equal(source[openIndex], openDelimiter, `missing ${openDelimiter} at ${openIndex}`);
  let depth = 0;
  let quote: '"' | "'" | undefined;
  let lineComment = false;
  let blockComment = false;
  for (let index = openIndex; index < source.length; index += 1) {
    const current = source[index] ?? "";
    const next = source[index + 1] ?? "";
    const previous = source[index - 1] ?? "";
    if (lineComment) {
      if (current === "\n") lineComment = false;
      continue;
    }
    if (blockComment) {
      if (current === "*" && next === "/") {
        blockComment = false;
        index += 1;
      }
      continue;
    }
    if (quote !== undefined) {
      if (current === quote && previous !== "\\") quote = undefined;
      continue;
    }
    if (current === "/" && next === "/") {
      lineComment = true;
      index += 1;
      continue;
    }
    if (current === "/" && next === "*") {
      blockComment = true;
      index += 1;
      continue;
    }
    if (current === '"' || (current === "'" && rustCharLiteralEnd(source, index) > index)) {
      quote = current;
      continue;
    }
    if (current === openDelimiter) depth += 1;
    if (current === closeDelimiter) {
      depth -= 1;
      if (depth === 0) return index;
    }
  }
  throw new Error(`unterminated Rust delimiter ${openDelimiter} at ${openIndex}`);
}

type AxisOrderSiteDisposition = "consumer" | "mirror" | "structural";

interface DiscoveredAxisOrderSite {
  readonly id: string;
  readonly path: string;
  readonly symbol: string;
  readonly authorityLinked: boolean;
  readonly axisTokens: readonly string[];
}

interface AxisOrderSiteDispositionRow {
  readonly id: string;
  readonly disposition: AxisOrderSiteDisposition;
  readonly owner?: string;
  readonly reentry?: string;
}

function cascadeAxisOrderDomainV0(): readonly string[] {
  return [
    "rust/crates/omena-cascade/src/",
    "rust/crates/omena-cascade/data/cascade-driver-census.json",
    "rust/crates/omena-cascade/examples/",
    "rust/crates/omena-query/src/style/cascade_checker/",
    "scripts/check-rust-ranked-set-loss-census.ts",
    "scripts/oss-corpus-farm.ts",
    "rust/crates/omena-diff-test/oss-corpus-farm/ranked-set-loss-census.json",
  ];
}

function cascadeAxisOrderSiteDispositionsV0(): readonly AxisOrderSiteDispositionRow[] {
  return [
    {
      id: "rust/crates/omena-cascade/src/model.rs#impl Ord for Specificity",
      disposition: "consumer",
    },
    {
      id: "rust/crates/omena-cascade/src/model.rs#impl Ord for CascadeKey",
      disposition: "consumer",
    },
    {
      id: "rust/crates/omena-cascade/src/model.rs#compare_cascade_axis_prefix",
      disposition: "consumer",
    },
    {
      id: "rust/crates/omena-cascade/src/ranking.rs#summarize_cascade_margin_schema_v0",
      disposition: "consumer",
    },
    {
      id: "rust/crates/omena-cascade/src/ranking.rs#dominant_cascade_key_margin",
      disposition: "consumer",
    },
    {
      id: "rust/crates/omena-cascade/src/ranked_set_loss_census.rs#deciding_axis",
      disposition: "consumer",
    },
    {
      id: "rust/crates/omena-cascade/src/ranked_set_loss_census.rs#strict_axis_prefix_winner",
      disposition: "consumer",
    },
    {
      id: "rust/crates/omena-cascade/data/cascade-driver-census.json#json:winnerAxes",
      disposition: "mirror",
    },
    {
      id: "rust/crates/omena-cascade/src/tests.rs#open_world_selector_matches_the_hand_written_axis_order",
      disposition: "mirror",
    },
    {
      id: "rust/crates/omena-cascade/src/tests.rs#cascade_margin_schema_is_substrate_only_until_calibrated",
      disposition: "mirror",
    },
    {
      id: "rust/crates/omena-cascade/src/tests.rs#cascade_key_order_covers_each_spec_axis",
      disposition: "mirror",
    },
    {
      id: "rust/crates/omena-cascade/src/tests.rs#open_world_strict_cascade_level_dominance_returns_definite",
      disposition: "mirror",
    },
    {
      id: "rust/crates/omena-cascade/src/tests.rs#open_world_strict_scope_dominance_uses_nearer_scope",
      disposition: "mirror",
    },
    {
      id: "rust/crates/omena-cascade/src/tests.rs#selects_definite_winner_with_proof",
      disposition: "mirror",
    },
    {
      id: "rust/crates/omena-cascade/src/tests.rs#selects_generic_winner_with_same_cascade_ordering",
      disposition: "mirror",
    },
    {
      id: "rust/crates/omena-cascade/src/tests.rs#open_world_module_provenance_remains_below_source_order",
      disposition: "structural",
      owner: "open-world evidence ordering contract",
      reentry: "open-world evidence stops following the specification-key order",
    },
    {
      id: "rust/crates/omena-cascade/src/conformance.rs#cascade_conformance_seed_cases",
      disposition: "structural",
      owner: "cascade conformance corpus",
      reentry: "the authored seed-case precedence relationships change",
    },
    {
      id: "rust/crates/omena-cascade/src/conformance.rs#cascade_ordering_axis_self_check_cases",
      disposition: "structural",
      owner: "cascade conformance corpus",
      reentry: "the ordering self-check axis family changes",
    },
    {
      id: "rust/crates/omena-query/src/style/cascade_checker/confidence.rs#query_cascade_confidence_axis_weight_basis_points",
      disposition: "consumer",
    },
    {
      id: "rust/crates/omena-query/src/style/cascade_checker/smt.rs#query_smt_layer_inversion_obligations",
      disposition: "structural",
      owner: "query SMT layer-inversion obligations",
      reentry: "the SMT obligation starts comparing cascade-key axes",
    },
    {
      id: "scripts/check-rust-ranked-set-loss-census.ts#top-level:strictAxisPrefix",
      disposition: "consumer",
    },
    {
      id: "scripts/check-rust-ranked-set-loss-census.ts#classifyArtifactRow",
      disposition: "consumer",
    },
    {
      id: "scripts/check-rust-ranked-set-loss-census.ts#compareAxisPrefix",
      disposition: "consumer",
    },
    {
      id: "scripts/oss-corpus-farm.ts#buildRankedSetLossCensus",
      disposition: "consumer",
    },
    {
      id: "rust/crates/omena-diff-test/oss-corpus-farm/ranked-set-loss-census.json#json:decidingAxisCounts",
      disposition: "mirror",
    },
  ];
}

function emittedAxisOrderSiteDispositionV0(): AxisOrderSiteDispositionRow {
  return {
    id: "rust/crates/omena-cascade/examples/cascade_key_axis_order.rs#main",
    disposition: "mirror",
  };
}

function cascadeAxisOrderSiteCensus(sourceRef?: string): {
  readonly product: "omena-cascade.key-axis-order-site-census";
  readonly sourceRef: string;
  readonly authorityPresent: boolean;
  readonly siteCount: number;
  readonly dispositionCounts: Readonly<Record<AxisOrderSiteDisposition, number>>;
  readonly domain: readonly string[];
  readonly sites: readonly {
    readonly id: string;
    readonly path: string;
    readonly disposition: AxisOrderSiteDisposition;
    readonly owner?: string;
    readonly reentry?: string;
  }[];
} {
  const sources = readCascadeAxisOrderDomain(sourceRef);
  if (args.has("--inject-unlinked-axis-literal")) {
    sources.set(
      "scripts/cascade-key-axis-order-unlinked-probe.ts",
      'const localOrder = ["level", "layerRank", "scopeProximity", "specificityIds", "specificityClasses", "specificityElements", "sourceOrder"];\n',
    );
  }
  if (args.has("--inject-axis-mirror-drift")) {
    const testPath = "rust/crates/omena-cascade/src/tests.rs";
    const source = sources.get(testPath);
    assert.ok(source, `missing injected mirror source ${testPath}`);
    const drifted = source.replace(
      '"scopeProximity",\n            "specificityIds",',
      '"specificityIds",\n            "scopeProximity",',
    );
    assert.notEqual(drifted, source, "axis mirror falsifier needle is stale");
    sources.set(testPath, drifted);
  }
  if (args.has("--inject-axis-consumer-detachment")) {
    const censusPath = "rust/crates/omena-cascade/src/ranked_set_loss_census.rs";
    const source = sources.get(censusPath);
    assert.ok(source, `missing injected consumer source ${censusPath}`);
    const detached = source.replaceAll("compare_cascade_axis_prefix", "detached_axis_prefix");
    assert.notEqual(detached, source, "axis consumer falsifier needle is stale");
    sources.set(censusPath, detached);
  }

  const sourceAxisOrder = cascadeAxisOrderFromSources(sources);
  assert.deepEqual(
    axisOrderFromSchemaOracle(sources.get("rust/crates/omena-cascade/src/tests.rs") ?? ""),
    sourceAxisOrder,
    "margin schema oracle must mirror the axis authority",
  );
  assert.deepEqual(
    axisOrderFromHandOracle(sources.get("rust/crates/omena-cascade/src/tests.rs") ?? ""),
    sourceAxisOrder,
    "hand-written open-world oracle must mirror the axis authority",
  );
  assert.deepEqual(
    axisOrderFromConfidence(
      sources.get("rust/crates/omena-query/src/style/cascade_checker/confidence.rs") ?? "",
      sourceAxisOrder,
    ),
    sourceAxisOrder,
    "query confidence ordering must consume or mirror the axis authority",
  );
  assert.deepEqual(
    axisOrderFromDriverCensus(
      sources.get("rust/crates/omena-cascade/data/cascade-driver-census.json") ?? "",
    ),
    collapseSpecificityAxis(sourceAxisOrder).map((axis) =>
      axis === "level" ? "cascadeLevel" : axis,
    ),
    "driver census winner axes must mirror the collapsed key-axis authority",
  );

  const discovered = discoverAxisOrderSites(sources);
  const authorityPresent = sources.has("rust/crates/omena-cascade/src/axis_order.rs");
  const dispositions = [...cascadeAxisOrderSiteDispositionsV0()];
  const cascadeTests = sources.get("rust/crates/omena-cascade/src/tests.rs") ?? "";
  const cascadeOrigin = sources.get("rust/crates/omena-cascade/src/origin.rs") ?? "";
  if (cascadeOrigin.includes("fn cascade_winner_axis_catalog_from_authority_v0")) {
    dispositions.push({
      id: "rust/crates/omena-cascade/src/origin.rs#cascade_winner_axis_catalog_from_authority_v0",
      disposition: "consumer",
    });
  } else {
    dispositions.push({
      id: "rust/crates/omena-cascade/src/origin.rs#cascade_driver_census_is_consistent_v0",
      disposition: "mirror",
    });
  }
  if (sources.has("rust/crates/omena-cascade/examples/cascade_key_axis_order.rs")) {
    dispositions.push(emittedAxisOrderSiteDispositionV0());
  }
  if (cascadeTests.includes("fn carries_module_rank_without_using_it_as_an_exact_order_axis")) {
    dispositions.push({
      id: "rust/crates/omena-cascade/src/tests.rs#carries_module_rank_without_using_it_as_an_exact_order_axis",
      disposition: "mirror",
    });
  }
  if (
    cascadeTests.includes(
      "fn open_world_winner_is_independent_of_input_order_when_only_module_rank_differs",
    )
  ) {
    dispositions.push({
      id: "rust/crates/omena-cascade/src/tests.rs#open_world_winner_is_independent_of_input_order_when_only_module_rank_differs",
      disposition: "structural",
      owner: "open-world evidence ordering contract",
      reentry: "the selector no longer provides stable evidence ordering",
    });
  }
  if (cascadeTests.includes("fn generated_binary_search_hits_if_and_only_if_a_key_is_equal")) {
    dispositions.push({
      id: "rust/crates/omena-cascade/src/tests.rs#generated_binary_search_hits_if_and_only_if_a_key_is_equal",
      disposition: "structural",
      owner: "CascadeKey Eq and Ord coherence contract",
      reentry: "the CascadeKey field set or total-order implementation changes",
    });
  }
  if (cascadeTests.includes("fn library_axis_order_prefers_specificity_before_scope_proximity")) {
    dispositions.push({
      id: "rust/crates/omena-cascade/src/tests.rs#library_axis_order_prefers_specificity_before_scope_proximity",
      disposition: "mirror",
    });
  }
  assert.deepEqual(
    discovered.map((site) => site.id).toSorted(),
    dispositions.map((site) => site.id).toSorted(),
    "syntactically discovered axis-order sites and disposition rows must match both ways",
  );

  const discoveredById = new Map(discovered.map((site) => [site.id, site]));
  const sites = dispositions.map(({ id, disposition, owner, reentry }) => {
    const discoveredSite = discoveredById.get(id);
    assert.ok(discoveredSite, `missing discovered axis-order site ${id}`);
    if (disposition === "consumer" && authorityPresent) {
      assert.equal(
        discoveredSite.authorityLinked,
        true,
        `axis-order consumer ${id} is not linked to the authority`,
      );
    }
    if (disposition === "structural") {
      assert.ok(owner, `structural axis-order site ${id} needs an owner`);
      assert.ok(reentry, `structural axis-order site ${id} needs a re-entry condition`);
    }
    return {
      ...discoveredSite,
      authorityLinked: authorityPresent && discoveredSite.authorityLinked,
      disposition,
      ...(owner === undefined ? {} : { owner }),
      ...(reentry === undefined ? {} : { reentry }),
    };
  });
  const dispositionCounts: Record<AxisOrderSiteDisposition, number> = {
    consumer: 0,
    mirror: 0,
    structural: 0,
  };
  for (const site of sites) dispositionCounts[site.disposition] += 1;
  return {
    product: "omena-cascade.key-axis-order-site-census",
    sourceRef: sourceRef ?? "worktree",
    authorityPresent,
    siteCount: sites.length,
    dispositionCounts,
    domain: cascadeAxisOrderDomainV0(),
    sites,
  };
}

function discoverAxisOrderSites(
  sources: ReadonlyMap<string, string>,
): readonly DiscoveredAxisOrderSite[] {
  const sites = new Map<string, DiscoveredAxisOrderSite>();
  for (const [file, source] of sources) {
    if (file === "rust/crates/omena-cascade/src/axis_order.rs") continue;
    if (file.endsWith("cascade-driver-census.json")) {
      addJsonAxisSite(sites, file, source, "winnerAxes");
      continue;
    }
    if (file.endsWith("ranked-set-loss-census.json")) {
      addJsonAxisSite(sites, file, source, "decidingAxisCounts");
      continue;
    }
    for (const symbol of sourceSymbols(file, source)) {
      const normalizedSymbol = normalizeAxisOrderSymbol(file, symbol.name);
      if (normalizedSymbol === undefined) continue;
      const axisTokens = axisTokensIn(symbol.body);
      const authorityLinked = hasAxisAuthorityLink(symbol.body);
      const semanticAxisSymbol =
        normalizedSymbol === "impl Ord for Specificity" || normalizedSymbol === "deciding_axis";
      const denseAxisLiteral = hasDenseAxisLiteral(symbol.body);
      if (
        !authorityLinked &&
        !semanticAxisSymbol &&
        !denseAxisLiteral &&
        (axisTokens.length < 2 || !hasAxisOrderForm(symbol.body))
      ) {
        continue;
      }
      const id = `${file}#${normalizedSymbol}`;
      sites.set(id, {
        id,
        path: file,
        symbol: normalizedSymbol,
        authorityLinked,
        axisTokens,
      });
    }
    if (
      file === "scripts/check-rust-ranked-set-loss-census.ts" &&
      (source.includes("const strictAxisPrefix") ||
        source.includes("Object.keys(artifact.decidingAxisCounts)"))
    ) {
      const id = `${file}#top-level:strictAxisPrefix`;
      sites.set(id, {
        id,
        path: file,
        symbol: "top-level:strictAxisPrefix",
        authorityLinked: source.includes("axisOrderArtifact.axisOrder.slice"),
        axisTokens: ["rankedSetPrefix"],
      });
    }
  }
  return [...sites.values()].toSorted((left, right) => left.id.localeCompare(right.id));
}

function addJsonAxisSite(
  sites: Map<string, DiscoveredAxisOrderSite>,
  file: string,
  source: string,
  key: string,
): void {
  const parsed = JSON.parse(source) as Readonly<Record<string, unknown>>;
  assert.ok(key in parsed, `${file} is missing ${key}`);
  const id = `${file}#json:${key}`;
  sites.set(id, {
    id,
    path: file,
    symbol: `json:${key}`,
    authorityLinked: false,
    axisTokens: axisTokensIn(JSON.stringify(parsed[key])),
  });
}

interface SourceSymbolV0 {
  readonly name: string;
  readonly body: string;
}

function sourceSymbols(file: string, source: string): readonly SourceSymbolV0[] {
  const declarationPattern = file.endsWith(".rs")
    ? /^(?:pub(?:\([^)]*\))?\s+)?(?:const\s+)?fn\s+([A-Za-z0-9_]+)|^impl\s+Ord\s+for\s+([A-Za-z0-9_]+)/gmu
    : /^(?:export\s+)?function\s+([A-Za-z0-9_]+)/gmu;
  const symbols = [...source.matchAll(declarationPattern)].flatMap((match) => {
    const name = match[1] ?? (file.endsWith(".rs") ? `impl Ord for ${match[2]}` : "");
    if (isAxisOrderDetectorInternal(file, name)) return [];
    return [
      {
        name,
        body: bracedSourceItem(source, match.index, `${file}#${name}`, file.endsWith(".rs")),
      },
    ];
  });
  if (file.endsWith(".ts")) {
    for (const match of source.matchAll(/^(?:export\s+)?const\s+([A-Za-z0-9_]+)/gmu)) {
      const name = match[1] ?? "";
      if (name !== "strictAxisPrefix" && name !== "localOrder") continue;
      const end = source.indexOf(";", match.index);
      symbols.push({
        name,
        body: source.slice(match.index, end < 0 ? source.length : end + 1),
      });
    }
  }
  return symbols;
}

function bracedSourceItem(
  source: string,
  start: number,
  label: string,
  rustSource: boolean,
): string {
  const open = source.indexOf("{", start);
  assert.ok(open >= 0, `axis-order source symbol ${label} is missing a body`);
  let depth = 0;
  let quote: '"' | "'" | "`" | undefined;
  let lineComment = false;
  let blockComment = false;
  for (let index = open; index < source.length; index += 1) {
    const current = source[index] ?? "";
    const next = source[index + 1] ?? "";
    const previous = source[index - 1] ?? "";
    if (lineComment) {
      if (current === "\n") lineComment = false;
      continue;
    }
    if (blockComment) {
      if (current === "*" && next === "/") {
        blockComment = false;
        index += 1;
      }
      continue;
    }
    if (quote !== undefined) {
      if (current === quote && previous !== "\\") quote = undefined;
      continue;
    }
    if (current === "/" && next === "/") {
      lineComment = true;
      index += 1;
      continue;
    }
    if (current === "/" && next === "*") {
      blockComment = true;
      index += 1;
      continue;
    }
    if (rustSource && current === "'") {
      const charEnd = rustCharLiteralEnd(source, index);
      if (charEnd > index) {
        index = charEnd;
        continue;
      }
    }
    if (current === '"' || (!rustSource && current === "'") || current === "`") {
      quote = current;
      continue;
    }
    if (current === "{") depth += 1;
    if (current === "}") {
      depth -= 1;
      if (depth === 0) return source.slice(start, index + 1);
    }
  }
  throw new Error(`axis-order source symbol ${label} has an unterminated body`);
}

function rustCharLiteralEnd(source: string, start: number): number {
  let escaped = false;
  for (let index = start + 1; index < Math.min(source.length, start + 12); index += 1) {
    const current = source[index] ?? "";
    if (!escaped && current === "'") return index;
    if (!escaped && current === "\\") {
      escaped = true;
    } else {
      escaped = false;
    }
  }
  return -1;
}

function normalizeAxisOrderSymbol(file: string, symbol: string): string | undefined {
  if (
    file === "rust/crates/omena-cascade/src/tests.rs" &&
    (symbol === "orders_cascade_keys_by_level_layer_scope_specificity_and_source" ||
      symbol === "orders_cascade_keys_by_level_layer_specificity_scope_and_source")
  ) {
    return "cascade_key_order_covers_each_spec_axis";
  }
  if (isAxisOrderDetectorInternal(file, symbol)) return undefined;
  if (
    file === "scripts/check-rust-ranked-set-loss-census.ts" &&
    /^(?:axisOrderArtifact|specificityStart|strictAxisPrefix|axisOrderSiteCensus)$/u.test(symbol)
  ) {
    return "top-level:strictAxisPrefix";
  }
  return symbol;
}

function isAxisOrderDetectorInternal(file: string, symbol: string): boolean {
  return (
    file === "scripts/check-rust-ranked-set-loss-census.ts" &&
    /^(?:cascadeAxisOrder|emittedAxisOrderSite|discover|addJsonAxis|sourceSymbols|bracedSourceItem|rustCharLiteralEnd|normalizeAxis|isAxisOrderDetectorInternal|axisTokensIn|hasAxis|hasDenseAxisLiteral|axisOrderFrom|collapseSpecificity|quotedAxis|axisNameFrom|escapeRegExp|readCascadeAxis|listCurrent|listTracked|readAtRef|validateCascadeKeyProducer|validateCallerSuppliedScope|readCascadeKeyProducer|isCascadeKeyProducer|classifyCascade|rustFunctionItems|enclosingRustFunction|cascadeKeyProducerId|rustStructFieldValue|splitRustTopLevel|matchingRustDelimiter|maskCfgTestRustItems)/u.test(
      symbol,
    )
  );
}

function axisTokensIn(source: string): readonly string[] {
  const patterns: readonly [string, RegExp][] = [
    ["level", /(?:CascadeLevel|cascade_level|cascadeLevel|[."']level\b|\blevel_)/u],
    ["layerRank", /(?:LayerRank|layer_rank|layerRank|\blayer_)/u],
    ["scopeProximity", /(?:scope_proximity|scopeProximity|\bscope_)/u],
    ["specificity", /(?:Specificity|\bspecificity\b|\bspecificity_)/u],
    ["specificityIds", /(?:specificity\.ids|specificityIds|ids:\s*u32)/u],
    ["specificityClasses", /(?:specificity\.classes|specificityClasses|classes:\s*u32)/u],
    ["specificityElements", /(?:specificity\.elements|specificityElements|elements:\s*u32)/u],
    ["sourceOrder", /(?:source_order|sourceOrder|\bsource_)/u],
  ];
  return patterns.filter(([, pattern]) => pattern.test(source)).map(([axis]) => axis);
}

function hasAxisAuthorityLink(source: string): boolean {
  const withoutSchemaDeclaration = source.replace(
    /fn\s+summarize_cascade_margin_schema_v0\s*\(/u,
    "fn schema_declaration(",
  );
  return /(?:compare_specificity_axes_v0|compare_cascade_key_axes_v0|compare_cascade_axis_prefix(?:_v0)?|first_deciding_cascade_key_axis_v0|cascade_key_axis_order_v0|cascade_key_axis_signed_distance_v0|summarize_cascade_margin_schema_v0|cascade-key-axis-order\.json|rankedSetPrefixAxisVocabulary|strictAxisPrefix)/u.test(
    withoutSchemaDeclaration,
  );
}

function hasAxisOrderForm(source: string): boolean {
  return /(?:\.cmp\s*\(|then_with|Reverse\s*\(|sort(?:_by(?:_key)?)?\s*\(|toSorted\s*\(|axis[_A-Z]?order|axisOrder|winnerAxes|expected_axes|decidingAxisCounts|strictAxisPrefix|compareAxisPrefix|dominant_axis|dominant_cascade|first_deciding|lexicograph|weight|-beats-|(?:fn|function)\s+[A-Za-z0-9_]*(?:beats|dominance|order)[A-Za-z0-9_]*)/u.test(
    source,
  );
}

function hasDenseAxisLiteral(source: string): boolean {
  const names = [
    ...source.matchAll(
      /["'](level|layerRank|scopeProximity|specificityIds|specificityClasses|specificityElements|sourceOrder)["']/gu,
    ),
  ].map((match) => match[1] ?? "");
  return new Set(names).size >= 5;
}

function readCascadeAxisOrderDomain(sourceRef?: string): Map<string, string> {
  const listed =
    sourceRef === undefined ? listCurrentTrackedFiles() : listTrackedFilesAt(sourceRef);
  const selected = listed.filter((file) =>
    cascadeAxisOrderDomainV0().some((entry) =>
      entry.endsWith("/") ? file.startsWith(entry) : file === entry,
    ),
  );
  const sources = new Map<string, string>();
  for (const file of selected) {
    sources.set(file, readRepositorySource(file, sourceRef));
  }
  return sources;
}

function readRepositorySource(file: string, sourceRef?: string): string {
  const cacheKey = `${sourceRef ?? "worktree"}\0${file}`;
  const cached = repositorySourceCache.get(cacheKey);
  if (cached !== undefined) return cached;
  const source =
    sourceRef === undefined
      ? readFileSync(path.join(repoRoot, file), "utf8")
      : readAtRef(sourceRef, file);
  repositorySourceCache.set(cacheKey, source);
  return source;
}

function listCurrentTrackedFiles(): readonly string[] {
  if (currentTrackedFiles !== undefined) return currentTrackedFiles;
  const result = spawnSync("git", ["ls-files", "--cached", "--others", "--exclude-standard"], {
    cwd: repoRoot,
    encoding: "utf8",
  });
  assert.equal(result.status, 0, `git ls-files failed: ${result.stderr}`);
  currentTrackedFiles = result.stdout.split(/\r?\n/u).filter(Boolean);
  return currentTrackedFiles;
}

function listTrackedFilesAt(sourceRef: string): string[] {
  const result = spawnSync("git", ["ls-tree", "-r", "--name-only", sourceRef], {
    cwd: repoRoot,
    encoding: "utf8",
  });
  assert.equal(result.status, 0, `git ls-tree failed for ${sourceRef}: ${result.stderr}`);
  return result.stdout.split(/\r?\n/u).filter(Boolean);
}

function readAtRef(sourceRef: string, file: string): string {
  const result = spawnSync("git", ["show", `${sourceRef}:${file}`], {
    cwd: repoRoot,
    encoding: "utf8",
    maxBuffer: 1024 * 1024 * 8,
  });
  assert.equal(result.status, 0, `git show failed for ${sourceRef}:${file}: ${result.stderr}`);
  return result.stdout;
}

function cascadeAxisOrderFromSources(sources: ReadonlyMap<string, string>): readonly string[] {
  const authority = sources.get("rust/crates/omena-cascade/src/axis_order.rs");
  if (authority !== undefined) {
    const start = authority.indexOf("pub(crate) const CASCADE_KEY_AXIS_ORDER_V0");
    const end = authority.indexOf("];", start);
    assert.ok(start >= 0 && end > start, "cascade key axis authority is malformed");
    const variants = [
      ...authority.slice(start, end).matchAll(/CascadeKeyAxisV0::([A-Za-z]+)/gu),
    ].map((match) => axisNameFromRustVariant(match[1] ?? ""));
    assert.equal(new Set(variants).size, variants.length, "axis authority must not repeat axes");
    return variants;
  }
  const ranking = sources.get("rust/crates/omena-cascade/src/ranking.rs") ?? "";
  const start = ranking.indexOf("axis_order: vec![");
  const end = ranking.indexOf("],", start);
  assert.ok(start >= 0 && end > start, "pre-authority margin schema axis order is missing");
  return quotedAxisNames(ranking.slice(start, end));
}

function axisOrderFromSchemaOracle(source: string): readonly string[] {
  const start = source.indexOf("schema.axis_order");
  const end = source.indexOf("]", start);
  assert.ok(start >= 0 && end > start, "margin schema oracle axis list is missing");
  return quotedAxisNames(source.slice(start, end));
}

function axisOrderFromHandOracle(source: string): readonly string[] {
  const start = source.indexOf("let oracle_key = |");
  const end = source.indexOf("};", start);
  assert.ok(start >= 0 && end > start, "hand-written cascade oracle is missing");
  const body = source.slice(start, end);
  const fields: readonly [RegExp, string][] = [
    [/key\.level/u, "level"],
    [/key\.layer_rank/u, "layerRank"],
    [/key\.scope_proximity/u, "scopeProximity"],
    [/key\.specificity\.ids/u, "specificityIds"],
    [/key\.specificity\.classes/u, "specificityClasses"],
    [/key\.specificity\.elements/u, "specificityElements"],
    [/key\.source_order/u, "sourceOrder"],
  ];
  return fields
    .map(([pattern, axis]) => ({ axis, index: body.search(pattern) }))
    .filter(({ index }) => index >= 0)
    .toSorted((left, right) => left.index - right.index)
    .map(({ axis }) => axis);
}

function axisOrderFromConfidence(
  source: string,
  sourceAxisOrder: readonly string[],
): readonly string[] {
  const start = source.indexOf("fn query_cascade_confidence_axis_weight_basis_points");
  const end = source.indexOf("\n}", start);
  assert.ok(start >= 0 && end > start, "query confidence axis-weight function is missing");
  const body = source.slice(start, end);
  if (body.includes("summarize_cascade_margin_schema_v0")) return sourceAxisOrder;
  return [...body.matchAll(/"([A-Za-z]+)"\s*=>\s*[0-9_]+/gu)].map((match) => match[1] ?? "");
}

function axisOrderFromDriverCensus(source: string): readonly string[] {
  const parsed = JSON.parse(source) as {
    readonly winnerAxes: readonly { readonly axis: string }[];
  };
  return parsed.winnerAxes.map((entry) => entry.axis);
}

function collapseSpecificityAxis(axisOrder: readonly string[]): readonly string[] {
  const collapsed: string[] = [];
  for (const axis of axisOrder) {
    const value = axis.startsWith("specificity") ? "specificity" : axis;
    if (collapsed.at(-1) !== value) collapsed.push(value);
  }
  return collapsed;
}

function discoverUnclassifiedAxisOrderLiterals(
  sources: ReadonlyMap<string, string>,
  sourceAxisOrder: readonly string[],
): readonly string[] {
  const knownLiteralPaths = new Set([
    "rust/crates/omena-cascade/src/ranking.rs",
    "rust/crates/omena-cascade/src/tests.rs",
    "rust/crates/omena-query/src/style/cascade_checker/confidence.rs",
    "scripts/check-rust-ranked-set-loss-census.ts",
  ]);
  const tokenPattern = sourceAxisOrder.map(escapeRegExp).join("[\"'][\\s\\S]{0,240}[\"']");
  const expression = new RegExp(`["']${tokenPattern}["']`, "gu");
  const unclassified: string[] = [];
  for (const [file, source] of sources) {
    if (knownLiteralPaths.has(file)) continue;
    if (expression.test(source)) unclassified.push(file);
    expression.lastIndex = 0;
  }
  return unclassified.toSorted();
}

function quotedAxisNames(source: string): readonly string[] {
  return [
    ...source.matchAll(
      /"(level|layerRank|scopeProximity|specificityIds|specificityClasses|specificityElements|sourceOrder)"/gu,
    ),
  ].map((match) => match[1] ?? "");
}

function axisNameFromRustVariant(variant: string): string {
  const names: Readonly<Record<string, string>> = {
    Level: "level",
    LayerRank: "layerRank",
    ScopeProximity: "scopeProximity",
    SpecificityIds: "specificityIds",
    SpecificityClasses: "specificityClasses",
    SpecificityElements: "specificityElements",
    SourceOrder: "sourceOrder",
  };
  const name = names[variant];
  assert.ok(name, `unknown cascade key axis authority variant ${variant}`);
  return name;
}

function escapeRegExp(value: string): string {
  return value.replace(/[.*+?^${}()|[\]\\]/gu, "\\$&");
}
