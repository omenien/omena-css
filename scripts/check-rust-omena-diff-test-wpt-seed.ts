import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { strict as assert } from "node:assert";
import { existsSync, readFileSync } from "node:fs";
import path from "node:path";
import { transform as lightningTransform } from "lightningcss";
import postcss from "postcss";

interface WptSeedManifestV0 {
  readonly schemaVersion: string;
  readonly product: string;
  readonly stage: string;
  readonly source: {
    readonly repository: string;
    readonly pin: string;
    readonly sparsePaths: readonly string[];
    readonly helperClasses: readonly string[];
    readonly layoutDependentHelpersExcluded: readonly string[];
  };
  readonly knownFailurePolicy: {
    readonly path: string;
    readonly schemaVersion: string;
    readonly stage2Blocking: boolean;
  };
  readonly generation: {
    readonly tool: string;
    readonly selectionPath: string;
  };
  readonly sparsePathFixtureCounts: readonly WptSparsePathFixtureCountV0[];
  readonly chunks: readonly WptSeedChunkManifestV0[];
  readonly extraction: {
    readonly tool: string;
    readonly sourcePin: string;
    readonly tuples: WptDerivedArtifactManifestV0;
    readonly coverage: WptDerivedArtifactManifestV0;
    readonly moduleCoverage: readonly WptTierZeroModuleCoverageV0[];
  };
  readonly expectations: {
    readonly reviewPolicy: WptDerivedArtifactManifestV0;
    readonly modules: readonly (WptDerivedArtifactManifestV0 & {
      readonly moduleId: string;
      readonly wptPath: string;
      readonly expectationCount: number;
    })[];
  };
}

interface WptDerivedArtifactManifestV0 {
  readonly path: string;
  readonly sha256: string;
  readonly recordCount: number;
}

interface WptTierZeroModuleCoverageV0 {
  readonly moduleId: string;
  readonly wptPath: string;
  readonly htmlFileCount: number;
  readonly eligibleTierZeroFileCount: number;
  readonly nonTierZeroFileCount: number;
  readonly excludedTentativeFileCount: number;
  readonly excludedOptionalFileCount: number;
  readonly extractedSubtestCount: number;
  readonly skippedDynamicCallCount: number;
  readonly skippedDynamicReasons: Readonly<Record<string, number>>;
}

interface WptTierZeroTupleArtifactV0 {
  readonly schemaVersion: string;
  readonly product: string;
  readonly source: {
    readonly repository: string;
    readonly pin: string;
    readonly extractionMode: string;
    readonly testharnessExecuted: boolean;
  };
  readonly tuples: readonly WptTierZeroTupleV0[];
}

interface WptTierZeroTupleV0 {
  readonly id: string;
  readonly moduleId: string;
  readonly wptPath: string;
  readonly wptSourceLine: number;
  readonly subtest: string;
  readonly sourceTextSha256: string;
  readonly helperClass: string;
  readonly helperCall: string;
  readonly subject: "property" | "selector" | "rule";
  readonly expectedValidity: "valid" | "invalid";
  readonly property: string;
  readonly value: string;
  readonly expectedValues: readonly string[];
  readonly specLinks: readonly string[];
}

interface WptExpectationManifestV0 {
  readonly schemaVersion: string;
  readonly product: string;
  readonly moduleId: string;
  readonly wptPath: string;
  readonly sourcePin: string;
  readonly expectations: readonly WptExpectationV0[];
}

interface WptExpectationV0 {
  readonly tupleId: string;
  readonly name: string;
  readonly status: "expected-failure" | "quarantined";
  readonly reasonCode: string;
  readonly adjudicationId: string;
}

interface WptSeedChunkManifestV0 {
  readonly chunkId: string;
  readonly path: string;
  readonly stage: string;
  readonly sha256: string;
  readonly fixtureCount: number;
  readonly sparsePathFixtureCounts: readonly WptSparsePathFixtureCountV0[];
}

interface WptSparsePathFixtureCountV0 {
  readonly sparsePath: string;
  readonly fixtureCount: number;
}

interface WptSeedChunkV0 {
  readonly schemaVersion: string;
  readonly product: string;
  readonly chunkId: string;
  readonly sourcePin: string;
  readonly fixtures: readonly WptSeedFixtureV0[];
}

interface WptSeedFixtureV0 {
  readonly id: string;
  readonly wptPath: string;
  readonly wptSourceLine: number;
  readonly subtest: string;
  readonly helper: string;
  readonly property: string;
  readonly wptValue: string;
  readonly wptExpectedValue: string;
  readonly source: string;
  readonly expectedCss: string;
  readonly status: "pass";
}

interface TransformExecuteSummaryV0 {
  readonly product: string;
  readonly unknownPassIds: readonly string[];
  readonly execution: {
    readonly product: string;
    readonly outputCss: string;
    readonly provenancePreserved: boolean;
    readonly passPlan: {
      readonly violatedDagEdgeCount: number;
      readonly allRequestedRegistered: boolean;
    };
  };
}

interface TransformExecuteBatchSummaryV0 {
  readonly schemaVersion: string;
  readonly product: string;
  readonly caseCount: number;
  readonly results: readonly TransformExecuteSummaryV0[];
}

interface KnownFailurePolicyV0 {
  readonly schemaVersion: string;
  readonly corpusManifest: string;
  readonly stage: string;
  readonly stage2Blocking: boolean;
  readonly sourcePin: string;
  readonly reviewIntervalDays: number;
  readonly requiredMinFixtureCountForStage2: number;
  readonly requiredConsecutiveGreenRuns: number;
  readonly consecutiveGreenRuns: number;
  readonly greenRuns: readonly GreenRunEvidenceV0[];
  readonly subtests: readonly KnownFailureSubtestV0[];
}

interface GreenRunEvidenceV0 {
  readonly date: string;
  readonly commit: string;
  readonly fixtureCount: number;
  readonly chunkSha256: string;
  readonly outcomeOlw: number;
  readonly criticalRegressionCount: number;
  readonly command: string;
}

interface KnownFailureSubtestV0 {
  readonly fixture: string;
  readonly name: string;
  readonly status: string;
  readonly reason: string;
  readonly issue: string;
  readonly since: string;
  readonly reviewAfter: string;
}

const repoRoot = process.cwd();
const corpusRoot = path.join(repoRoot, "rust/crates/omena-diff-test/wpt-corpus");
const manifestPath = path.join(corpusRoot, "manifest.json");
const fullExtractedCorpus = process.env.OMENA_WPT_FULL_CORPUS === "1";
const includeExtractedCases = process.env.OMENA_WPT_INCLUDE_CASES === "1";
const extractedPerModuleSampleLimit = 48;
const passIds = [
  "whitespace-strip",
  "comment-strip",
  "number-compression",
  "unit-normalization",
  "color-compression",
  "url-quote-strip",
  "string-quote-normalize",
  "selector-is-where-compression",
  "shorthand-combining",
  "rule-deduplication",
  "rule-merging",
  "selector-merging",
  "empty-rule-removal",
  "media-static-eval",
  "calc-reduction",
  "print-css",
] as const;

const manifest = readJson<WptSeedManifestV0>(manifestPath);
assert.equal(manifest.schemaVersion, "0");
assert.equal(manifest.product, "omena-diff-test.wpt-seed-corpus.manifest");
assert.ok(manifest.source.repository.endsWith("/web-platform-tests/wpt"));
assert.ok(isPinnedWptSha(manifest.source.pin), "WPT seed source pin must be a 40-character SHA");
assert.ok(manifest.source.sparsePaths.length > 0);
assert.ok(manifest.source.helperClasses.includes("test_valid_value"));
assert.ok(manifest.source.layoutDependentHelpersExcluded.includes("test_computed_value"));
assert.equal(manifest.generation.tool, "scripts/generate-rust-omena-diff-test-wpt-corpus.ts");
assert.equal(
  manifest.generation.selectionPath,
  "rust/crates/omena-diff-test/wpt-corpus/selections.json",
);
assert.equal(manifest.extraction.tool, "scripts/extract-rust-omena-diff-test-wpt-tier-zero.ts");
assert.ok(isPinnedWptSha(manifest.extraction.sourcePin));

const extractedTupleSource = readFileSync(
  path.join(corpusRoot, manifest.extraction.tuples.path),
  "utf8",
);
assert.equal(
  createHash("sha256").update(extractedTupleSource).digest("hex"),
  manifest.extraction.tuples.sha256,
  "extracted WPT tuple artifact hash drift",
);
const extractedTupleArtifact = JSON.parse(extractedTupleSource) as WptTierZeroTupleArtifactV0;
assert.equal(extractedTupleArtifact.schemaVersion, "0");
assert.equal(extractedTupleArtifact.product, "omena-diff-test.wpt-tier-zero-tuples");
assert.equal(extractedTupleArtifact.source.pin, manifest.extraction.sourcePin);
assert.equal(extractedTupleArtifact.source.testharnessExecuted, false);
assert.equal(extractedTupleArtifact.tuples.length, manifest.extraction.tuples.recordCount);
const expectationManifests = manifest.expectations.modules.map((artifact) => {
  const source = readFileSync(path.join(corpusRoot, artifact.path), "utf8");
  assert.equal(
    createHash("sha256").update(source).digest("hex"),
    artifact.sha256,
    `${artifact.path} expectation hash drift`,
  );
  const expectation = JSON.parse(source) as WptExpectationManifestV0;
  assert.equal(expectation.schemaVersion, "0");
  assert.equal(expectation.product, "omena-diff-test.wpt-tier-zero-expectations");
  assert.equal(expectation.moduleId, artifact.moduleId);
  assert.equal(expectation.wptPath, artifact.wptPath);
  assert.equal(expectation.sourcePin, manifest.extraction.sourcePin);
  assert.equal(expectation.expectations.length, artifact.expectationCount);
  assert.equal(expectation.expectations.length, artifact.recordCount);
  return expectation;
});
const expectations = expectationManifests.flatMap((manifest) => manifest.expectations);
const expectationByTupleId = new Map(expectations.map((entry) => [entry.tupleId, entry] as const));
assert.equal(expectationByTupleId.size, expectations.length, "duplicate WPT expectation tuple id");
const reviewPolicySource = readFileSync(
  path.join(corpusRoot, manifest.expectations.reviewPolicy.path),
  "utf8",
);
assert.equal(
  createHash("sha256").update(reviewPolicySource).digest("hex"),
  manifest.expectations.reviewPolicy.sha256,
  "WPT expectation review policy hash drift",
);

const extractedCoverageSource = readFileSync(
  path.join(corpusRoot, manifest.extraction.coverage.path),
  "utf8",
);
assert.equal(
  createHash("sha256").update(extractedCoverageSource).digest("hex"),
  manifest.extraction.coverage.sha256,
  "extracted WPT coverage artifact hash drift",
);

const policy = readKnownFailurePolicy(path.resolve(corpusRoot, manifest.knownFailurePolicy.path));
assert.equal(policy.schemaVersion, manifest.knownFailurePolicy.schemaVersion);
assert.equal(policy.corpusManifest, "../wpt-corpus/manifest.json");
assert.equal(policy.stage2Blocking, manifest.knownFailurePolicy.stage2Blocking);
const expectedManifestStage = policy.stage2Blocking ? "stage2-blocking" : "stage1-advisory";
const expectedPolicyStage = policy.stage2Blocking ? "blocking" : "advisory";
assert.equal(manifest.stage, expectedManifestStage);
assert.equal(policy.stage, expectedPolicyStage);
assert.equal(policy.sourcePin, manifest.source.pin);
assert.ok(policy.reviewIntervalDays > 0, "known-failure review interval must be positive");
assert.ok(
  policy.requiredMinFixtureCountForStage2 > 0,
  "Stage 2 must declare a positive fixture-count threshold",
);
assert.ok(
  policy.requiredConsecutiveGreenRuns > 0,
  "Stage 2 must declare a positive consecutive-green threshold",
);
assert.ok(policy.consecutiveGreenRuns >= 0, "consecutive green runs cannot be negative");
assert.equal(
  policy.consecutiveGreenRuns,
  policy.greenRuns.length,
  "consecutive green runs must be backed by reviewed green-run evidence",
);

const chunkRecords = manifest.chunks.map((chunkManifest) => {
  const chunkPath = path.join(corpusRoot, chunkManifest.path);
  const chunkSource = readFileSync(chunkPath, "utf8");
  const actualHash = createHash("sha256").update(chunkSource).digest("hex");
  assert.equal(actualHash, chunkManifest.sha256, `${chunkManifest.path} sha256 drift`);
  const chunk = JSON.parse(chunkSource) as WptSeedChunkV0;
  assert.equal(chunk.schemaVersion, "0");
  assert.equal(chunk.product, "omena-diff-test.wpt-seed-corpus.chunk");
  assert.equal(chunk.chunkId, chunkManifest.chunkId);
  assert.equal(chunk.sourcePin, manifest.source.pin);
  assert.equal(chunk.fixtures.length, chunkManifest.fixtureCount);
  assert.deepEqual(
    chunkManifest.sparsePathFixtureCounts,
    countSparsePathFixtures(manifest.source.sparsePaths, chunk.fixtures),
    `${chunkManifest.path} sparse-path fixture counts drift`,
  );
  return { manifest: chunkManifest, chunk };
});
const fixtures = chunkRecords.flatMap((record) => record.chunk.fixtures);
const sparsePathFixtureCounts = countSparsePathFixtures(manifest.source.sparsePaths, fixtures);
assert.deepEqual(
  manifest.sparsePathFixtureCounts,
  sparsePathFixtureCounts,
  "manifest sparse-path fixture counts drift",
);
assert.ok(
  sparsePathFixtureCounts.every((count) => count.fixtureCount > 0),
  "every pinned WPT sparse path must have fixture coverage",
);
const blockingChunkRecords = chunkRecords.filter(
  (record) => record.manifest.stage === "stage2-blocking",
);
assert.equal(blockingChunkRecords.length, 1, "WPT Stage 2 policy expects one blocking chunk");
const blockingFixtures = blockingChunkRecords.flatMap((record) => record.chunk.fixtures);
const extractedTuples = selectExtractedTuples(
  extractedTupleArtifact.tuples,
  fullExtractedCorpus ? Number.POSITIVE_INFINITY : extractedPerModuleSampleLimit,
  new Set(expectationByTupleId.keys()),
);
const engineShadowRunnerPath = prepareEngineShadowRunner();
const omenaBatch = runOmenaTransformBatch([
  ...fixtures.map((fixture) => ({
    id: fixture.id,
    source: fixture.source,
    requestedPassIds: passIds,
  })),
  ...extractedTuples.map((tuple) => ({
    id: tuple.id,
    source: sourceForExtractedTuple(tuple),
    requestedPassIds: ["print-css"] as const,
  })),
]);
assert.equal(omenaBatch.results.length, fixtures.length + extractedTuples.length);

const fixtureKeys = new Set(fixtures.map((fixture) => fixture.id));
const subtestKeys = new Set(fixtures.map((fixture) => `${fixture.id}\n${fixture.subtest}`));
const currentBlockingChunkSha256 = blockingChunkRecords[0]?.manifest.sha256;
assert.ok(currentBlockingChunkSha256, "WPT seed corpus must declare a blocking chunk");
for (const run of policy.greenRuns) {
  assert.ok(isIsoDate(run.date), `${run.commit} green-run date must be an ISO date`);
  assert.ok(isCommitId(run.commit), `${run.commit} must be a commit id`);
  assert.equal(run.fixtureCount, blockingFixtures.length, `${run.commit} fixture count drift`);
  assert.equal(run.chunkSha256, currentBlockingChunkSha256, `${run.commit} chunk sha drift`);
  assert.equal(
    run.outcomeOlw,
    blockingFixtures.length,
    `${run.commit} must be all-green for current corpus`,
  );
  assert.equal(run.criticalRegressionCount, 0, `${run.commit} must have no critical regressions`);
  assert.ok(
    run.command.includes("rust/omena-diff-test-wpt-seed"),
    `${run.commit} command must point at the WPT seed checker`,
  );
}
const staleKnownFailures: string[] = [];
for (const subtest of policy.subtests) {
  if (
    !fixtureKeys.has(subtest.fixture) ||
    !subtestKeys.has(`${subtest.fixture}\n${subtest.name}`)
  ) {
    staleKnownFailures.push(`${subtest.fixture} ${subtest.name}`);
  }
  assert.match(subtest.status, /^(fail|implementation-defined)$/);
  assert.ok(subtest.reason.length > 0);
  assert.ok(subtest.issue.length > 0);
  assert.ok(isIsoDate(subtest.since), `${subtest.fixture} since must be an ISO date`);
  assert.ok(isIsoDate(subtest.reviewAfter), `${subtest.fixture} review_after must be an ISO date`);
}
assert.deepEqual(staleKnownFailures, [], "known-failure policy contains stale entries");

const reports = fixtures.map((fixture, fixtureIndex) => {
  assert.equal(fixture.status, "pass", fixture.id);
  assert.ok(manifest.source.helperClasses.includes(fixture.helper), fixture.id);
  assert.ok(
    manifest.source.sparsePaths.some((sparsePath) => fixture.wptPath.startsWith(`${sparsePath}/`)),
    `${fixture.id} must point at a pinned sparse WPT path`,
  );
  assert.ok(fixture.wptSourceLine > 0, `${fixture.id} must record a WPT source line`);
  assert.ok(
    fixture.subtest.includes(fixture.wptValue),
    `${fixture.id} subtest must include WPT value`,
  );
  assert.ok(
    fixture.subtest.includes(fixture.wptExpectedValue),
    `${fixture.id} subtest must include WPT expected value`,
  );
  assert.ok(
    fixture.source.includes(fixture.property),
    `${fixture.id} source must include property`,
  );
  assert.ok(
    fixture.source.includes(fixture.wptValue),
    `${fixture.id} source must include WPT value`,
  );
  const omena = omenaBatch.results[fixtureIndex];
  assert.ok(omena, `${fixture.id} is missing its Omena batch result`);
  const lightning = runLightningTransform(fixture);
  const omenaPass = omena.execution.outputCss === fixture.expectedCss;
  const lightningPass = lightning === fixture.expectedCss;
  const wptExpectedPass = fixture.status === "pass" && fixture.wptExpectedValue.length > 0;
  const outcomeCell = [
    omenaPass ? "O" : "o",
    lightningPass ? "L" : "l",
    wptExpectedPass ? "W" : "w",
  ].join("");

  assert.equal(omena.product, "omena-query.transform-execute", fixture.id);
  assert.equal(omena.execution.product, "omena-transform-passes.execution", fixture.id);
  assert.deepEqual(omena.unknownPassIds, [], fixture.id);
  assert.equal(omena.execution.passPlan.violatedDagEdgeCount, 0, fixture.id);
  assert.equal(omena.execution.passPlan.allRequestedRegistered, true, fixture.id);
  assert.equal(omena.execution.provenancePreserved, true, fixture.id);

  return {
    id: fixture.id,
    wptPath: fixture.wptPath,
    subtest: fixture.subtest,
    outcomeCell,
    omenaPass,
    lightningPass,
    wptExpectedPass,
  };
});

const extractedReports = extractedTuples.map((tuple, tupleIndex) => {
  const source = sourceForExtractedTuple(tuple);
  const omena = omenaBatch.results[fixtures.length + tupleIndex];
  assert.ok(omena, `${tuple.id} is missing its Omena batch result`);
  const lightning = runLightningTransformSource(tuple.id, source, false);
  const omenaObserved = observedSerialization(tuple, omena.execution.outputCss);
  const lightningObserved =
    lightning === undefined ? undefined : observedSerialization(tuple, lightning);
  const omenaPass = serializationSatisfiesExpectation(tuple, omenaObserved);
  const lightningPass = serializationSatisfiesExpectation(tuple, lightningObserved);
  const wptExpectedPass =
    tuple.sourceTextSha256 === createHash("sha256").update(tuple.subtest).digest("hex") &&
    (tuple.expectedValidity === "valid") === tuple.expectedValues.length > 0;
  return {
    id: tuple.id,
    moduleId: tuple.moduleId,
    wptPath: tuple.wptPath,
    subtest: tuple.subtest,
    expectedValidity: tuple.expectedValidity,
    expectedValues: tuple.expectedValues,
    specLinks: tuple.specLinks,
    omenaObserved,
    lightningObserved,
    outcomeCell: [
      omenaPass ? "O" : "o",
      lightningPass ? "L" : "l",
      wptExpectedPass ? "W" : "w",
    ].join(""),
    omenaPass,
    lightningPass,
    wptExpectedPass,
    expectation: expectationByTupleId.get(tuple.id),
  };
});

for (const expectation of expectations) {
  const report = extractedReports.find((candidate) => candidate.id === expectation.tupleId);
  assert.ok(report, `${expectation.tupleId} expectation was not evaluated`);
  if (expectation.status === "expected-failure") {
    assert.equal(report.omenaPass, false, `${expectation.tupleId} expectation is stale`);
  }
}

const criticalRegressionCount = reports.filter((report) => report.outcomeCell === "oLW").length;
assert.equal(criticalRegressionCount, 0, "WPT seed corpus has omena-only failures");
const stage2PromotionBlockers = stage2Blockers({
  manifestStage: manifest.stage,
  policyStage: policy.stage,
  fixtureCount: blockingFixtures.length,
  knownFailureCount: policy.subtests.length,
  stage2Blocking: policy.stage2Blocking,
  staleKnownFailureCount: staleKnownFailures.length,
  criticalRegressionCount,
  requiredMinFixtureCountForStage2: policy.requiredMinFixtureCountForStage2,
  requiredConsecutiveGreenRuns: policy.requiredConsecutiveGreenRuns,
  consecutiveGreenRuns: policy.consecutiveGreenRuns,
});
const stage2CandidateReady = stage2PromotionBlockers.length === 0;
if (manifest.knownFailurePolicy.stage2Blocking) {
  assert.ok(stage2CandidateReady, "Stage 2 blocking cannot be enabled before readiness");
}

const outcomeCells = ["OLW", "OLw", "OlW", "Olw", "oLW", "oLw", "olW", "olw"] as const;
const outcomeCube = Object.fromEntries(outcomeCells.map((cell) => [cell, 0])) as Record<
  (typeof outcomeCells)[number],
  number
>;
for (const report of reports) {
  assert.ok(report.outcomeCell in outcomeCube, `unexpected outcome cell: ${report.outcomeCell}`);
  outcomeCube[report.outcomeCell as (typeof outcomeCells)[number]] += 1;
}
const observedOutcomeCube = reports.reduce<Record<string, number>>((counts, report) => {
  counts[report.outcomeCell] = (counts[report.outcomeCell] ?? 0) + 1;
  return counts;
}, {});
const extractedOutcomeCube = Object.fromEntries(outcomeCells.map((cell) => [cell, 0])) as Record<
  (typeof outcomeCells)[number],
  number
>;
for (const report of extractedReports) {
  assert.ok(
    report.outcomeCell in extractedOutcomeCube,
    `unexpected outcome cell: ${report.outcomeCell}`,
  );
  extractedOutcomeCube[report.outcomeCell as (typeof outcomeCells)[number]] += 1;
}
const extractedModuleOutcomes = manifest.extraction.moduleCoverage.map((module) => {
  const moduleReports = extractedReports.filter((report) => report.moduleId === module.moduleId);
  return {
    moduleId: module.moduleId,
    sourcePin: manifest.extraction.sourcePin,
    extractedSubtestCount: module.extractedSubtestCount,
    evaluatedSubtestCount: moduleReports.length,
    omenaPassCount: moduleReports.filter((report) => report.omenaPass).length,
    lightningPassCount: moduleReports.filter((report) => report.lightningPass).length,
    expectedSetWitnessCount: moduleReports.filter((report) => report.wptExpectedPass).length,
    expectedFailureCount: moduleReports.filter(
      (report) => report.expectation?.status === "expected-failure",
    ).length,
    quarantinedCount: moduleReports.filter((report) => report.expectation?.status === "quarantined")
      .length,
    unexpectedFailureCount: moduleReports.filter(
      (report) => !report.omenaPass && report.expectation === undefined,
    ).length,
    skippedDynamicCallCount: module.skippedDynamicCallCount,
    nonTierZeroFileCount: module.nonTierZeroFileCount,
  };
});

assert.equal(reports.length, fixtures.length);
assert.equal(Object.keys(outcomeCube).length, 8);
assert.ok(Object.keys(observedOutcomeCube).length > 0);

process.stdout.write(
  JSON.stringify(
    {
      product: "omena-diff-test.wpt-seed-three-way-report",
      stage: manifest.stage,
      sourcePin: manifest.source.pin,
      fixtureCount: reports.length,
      blockingFixtureCount: blockingFixtures.length,
      advisoryFixtureCount: reports.length - blockingFixtures.length,
      sparsePathFixtureCounts,
      chunkCount: chunkRecords.length,
      knownFailureCount: policy.subtests.length,
      knownFailureReviewIntervalDays: policy.reviewIntervalDays,
      stage2Blocking: policy.stage2Blocking,
      stage2CandidateReady,
      requiredMinFixtureCountForStage2: policy.requiredMinFixtureCountForStage2,
      requiredConsecutiveGreenRuns: policy.requiredConsecutiveGreenRuns,
      consecutiveGreenRuns: policy.consecutiveGreenRuns,
      greenRunEvidenceCount: policy.greenRuns.length,
      stage2PromotionBlockers,
      staleKnownFailureCount: staleKnownFailures.length,
      criticalRegressionCount,
      outcomeCellCount: Object.keys(outcomeCube).length,
      outcomeCube,
      extractedCorpusMode: fullExtractedCorpus ? "full" : "deterministic-module-sample",
      extractedSourcePin: manifest.extraction.sourcePin,
      extractedTupleCount: extractedTupleArtifact.tuples.length,
      extractedEvaluatedTupleCount: extractedReports.length,
      extractedOutcomeCube,
      extractedModuleOutcomes,
      ...(includeExtractedCases ? { extractedCases: extractedReports } : {}),
    },
    null,
    2,
  ),
);
process.stdout.write("\n");

function readJson<T>(filePath: string): T {
  return JSON.parse(readFileSync(filePath, "utf8")) as T;
}

function isPinnedWptSha(pin: string): boolean {
  return /^web-platform-tests\/wpt@[0-9a-f]{40}$/.test(pin);
}

function countSparsePathFixtures(
  sparsePaths: readonly string[],
  fixtureSet: readonly WptSeedFixtureV0[],
): readonly WptSparsePathFixtureCountV0[] {
  return sparsePaths.map((sparsePath) => ({
    sparsePath,
    fixtureCount: fixtureSet.filter((fixture) => fixture.wptPath.startsWith(`${sparsePath}/`))
      .length,
  }));
}

function runOmenaTransformBatch(
  cases: readonly {
    readonly id: string;
    readonly source: string;
    readonly requestedPassIds: readonly string[];
  }[],
): TransformExecuteBatchSummaryV0 {
  const result = spawnSync(engineShadowRunnerPath, ["transform-execute-batch"], {
    cwd: repoRoot,
    encoding: "utf8",
    input: JSON.stringify({
      cases: cases.map((entry) => ({
        stylePath: `${entry.id}.css`,
        styleSource: entry.source,
        requestedPassIds: entry.requestedPassIds,
      })),
    }),
    maxBuffer: 64 * 1024 * 1024,
  });

  assert.equal(result.status, 0, result.stderr);
  assert.equal(result.error, undefined);

  const output = JSON.parse(result.stdout) as TransformExecuteBatchSummaryV0;
  assert.equal(output.schemaVersion, "0");
  assert.equal(output.product, "engine-shadow-runner.transform-execute-batch");
  assert.equal(output.caseCount, cases.length);
  return output;
}

function runLightningTransform(fixture: WptSeedFixtureV0): string {
  const output = runLightningTransformSource(fixture.id, fixture.source, true);
  assert.notEqual(output, undefined, `${fixture.id} must be accepted by lightningcss`);
  return output;
}

function runLightningTransformSource(
  id: string,
  source: string,
  minify: boolean,
): string | undefined {
  try {
    const result = lightningTransform({
      filename: `${id}.css`,
      code: Buffer.from(source),
      minify,
    });
    return String(result.code);
  } catch {
    return undefined;
  }
}

function prepareEngineShadowRunner(): string {
  const profile = fullExtractedCorpus ? "release" : "debug";
  const profileArgs = fullExtractedCorpus ? ["--release"] : [];
  const build = spawnSync(
    "cargo",
    [
      "build",
      "--quiet",
      "--manifest-path",
      "rust/Cargo.toml",
      "-p",
      "engine-shadow-runner",
      ...profileArgs,
    ],
    { cwd: repoRoot, encoding: "utf8", maxBuffer: 8 * 1024 * 1024 },
  );
  assert.equal(build.status, 0, build.stderr);
  const metadata = spawnSync(
    "cargo",
    ["metadata", "--no-deps", "--format-version", "1", "--manifest-path", "rust/Cargo.toml"],
    { cwd: repoRoot, encoding: "utf8", maxBuffer: 8 * 1024 * 1024 },
  );
  assert.equal(metadata.status, 0, metadata.stderr);
  const targetDirectory = (JSON.parse(metadata.stdout) as { readonly target_directory: string })
    .target_directory;
  const executable = path.join(
    targetDirectory,
    profile,
    process.platform === "win32" ? "engine-shadow-runner.exe" : "engine-shadow-runner",
  );
  assert.ok(existsSync(executable), `engine-shadow-runner build output is missing: ${executable}`);
  return executable;
}

function selectExtractedTuples(
  tuples: readonly WptTierZeroTupleV0[],
  perModuleLimit: number,
  requiredTupleIds: ReadonlySet<string>,
): readonly WptTierZeroTupleV0[] {
  if (!Number.isFinite(perModuleLimit)) return tuples;
  const selected: WptTierZeroTupleV0[] = [];
  for (const moduleId of [...new Set(tuples.map((tuple) => tuple.moduleId))].sort()) {
    const moduleTuples = tuples.filter((candidate) => candidate.moduleId === moduleId);
    const required = moduleTuples
      .filter((candidate) => requiredTupleIds.has(candidate.id))
      .sort((left, right) => left.id.localeCompare(right.id));
    assert.ok(required.length <= perModuleLimit, `${moduleId} expectations exceed sample capacity`);
    selected.push(...required);
    const selectedIds = new Set(required.map((tuple) => tuple.id));
    const buckets = new Map<string, WptTierZeroTupleV0[]>();
    for (const tuple of moduleTuples) {
      if (selectedIds.has(tuple.id)) continue;
      const key = `${tuple.subject}:${tuple.expectedValidity}:${tuple.helperCall}`;
      const bucket = buckets.get(key) ?? [];
      bucket.push(tuple);
      buckets.set(key, bucket);
    }
    const orderedBuckets = [...buckets.entries()]
      .sort(([left], [right]) => left.localeCompare(right))
      .map(([, bucket]) => bucket);
    while (
      selectedIds.size < perModuleLimit &&
      orderedBuckets.some((bucket) => bucket.length > 0)
    ) {
      for (const bucket of orderedBuckets) {
        const tuple = bucket.shift();
        if (tuple) {
          selected.push(tuple);
          selectedIds.add(tuple.id);
        }
        if (selectedIds.size >= perModuleLimit) {
          break;
        }
      }
    }
  }
  return selected;
}

function sourceForExtractedTuple(tuple: WptTierZeroTupleV0): string {
  switch (tuple.subject) {
    case "property":
      return `.wpt{${tuple.property}:${tuple.value}}`;
    case "selector":
      return `${tuple.value}{}`;
    case "rule":
      return tuple.value;
  }
}

function observedSerialization(tuple: WptTierZeroTupleV0, cssSource: string): string | undefined {
  try {
    const root = postcss.parse(cssSource);
    if (tuple.subject === "property") {
      let observed: string | undefined;
      root.walkDecls((declaration) => {
        if (observed === undefined && declaration.prop === tuple.property) {
          observed = declaration.value.trim();
        }
      });
      return observed;
    }
    const first = root.first;
    if (tuple.subject === "selector") {
      return first?.type === "rule" ? first.selector.trim() : undefined;
    }
    return first?.toString().trim();
  } catch {
    return undefined;
  }
}

function serializationSatisfiesExpectation(
  tuple: WptTierZeroTupleV0,
  observed: string | undefined,
): boolean {
  if (tuple.expectedValidity === "invalid") return observed === undefined;
  return observed !== undefined && tuple.expectedValues.includes(observed);
}

function readKnownFailurePolicy(filePath: string): KnownFailurePolicyV0 {
  const source = readFileSync(filePath, "utf8");
  const topLevel = new Map<string, string | boolean | number>();
  const greenRuns: GreenRunEvidenceV0[] = [];
  const subtests: KnownFailureSubtestV0[] = [];
  let currentGreenRun = new Map<string, string | boolean | number>();
  let currentSubtest = new Map<string, string>();
  let section: "top" | "green_run" | "subtest" = "top";

  for (const rawLine of source.split(/\r?\n/)) {
    const line = rawLine.replace(/\s+#.*$/, "").trim();
    if (line === "" || line.startsWith("#")) continue;
    if (line === "[[subtest]]") {
      pushGreenRun(greenRuns, currentGreenRun);
      currentGreenRun = new Map<string, string | boolean | number>();
      pushKnownFailureSubtest(subtests, currentSubtest);
      currentSubtest = new Map<string, string>();
      section = "subtest";
      continue;
    }
    if (line === "[[green_run]]") {
      pushKnownFailureSubtest(subtests, currentSubtest);
      currentSubtest = new Map<string, string>();
      pushGreenRun(greenRuns, currentGreenRun);
      currentGreenRun = new Map<string, string | boolean | number>();
      section = "green_run";
      continue;
    }
    const match = /^([A-Za-z0-9_]+)\s*=\s*(.+)$/.exec(line);
    assert.ok(match, `unsupported TOML line: ${rawLine}`);
    const [, key, rawValue] = match;
    const value = parseTomlScalar(rawValue);
    if (section === "subtest" || isKnownFailureSubtestKey(key)) {
      assert.equal(typeof value, "string", `${key} must be a string`);
      currentSubtest.set(key, value);
    } else if (section === "green_run") {
      currentGreenRun.set(key, value);
    } else {
      topLevel.set(key, value);
    }
  }
  pushGreenRun(greenRuns, currentGreenRun);
  pushKnownFailureSubtest(subtests, currentSubtest);

  return {
    schemaVersion: expectString(topLevel, "schema_version"),
    corpusManifest: expectString(topLevel, "corpus_manifest"),
    stage: expectString(topLevel, "stage"),
    stage2Blocking: expectBoolean(topLevel, "stage2_blocking"),
    sourcePin: expectString(topLevel, "source_pin"),
    reviewIntervalDays: expectNumber(topLevel, "review_interval_days"),
    requiredMinFixtureCountForStage2: expectNumber(
      topLevel,
      "required_min_fixture_count_for_stage2",
    ),
    requiredConsecutiveGreenRuns: expectNumber(topLevel, "required_consecutive_green_runs"),
    consecutiveGreenRuns: expectNumber(topLevel, "consecutive_green_runs"),
    greenRuns,
    subtests,
  };
}

function stage2Blockers(input: {
  readonly manifestStage: string;
  readonly policyStage: string;
  readonly stage2Blocking: boolean;
  readonly fixtureCount: number;
  readonly knownFailureCount: number;
  readonly staleKnownFailureCount: number;
  readonly criticalRegressionCount: number;
  readonly requiredMinFixtureCountForStage2: number;
  readonly requiredConsecutiveGreenRuns: number;
  readonly consecutiveGreenRuns: number;
}): string[] {
  const blockers: string[] = [];
  const expectedStageForManifest = input.stage2Blocking ? "stage2-blocking" : "stage1-advisory";
  const expectedStageForPolicy = input.stage2Blocking ? "blocking" : "advisory";
  if (input.manifestStage !== expectedStageForManifest) {
    blockers.push("stageMismatch");
  }
  if (input.policyStage !== expectedStageForPolicy) {
    blockers.push("knownFailurePolicyStageMismatch");
  }
  if (input.knownFailureCount > 0) {
    blockers.push("knownFailuresPresent");
  }
  if (input.staleKnownFailureCount > 0) {
    blockers.push("staleKnownFailuresPresent");
  }
  if (input.criticalRegressionCount > 0) {
    blockers.push("criticalRegressionsPresent");
  }
  if (input.fixtureCount < input.requiredMinFixtureCountForStage2) {
    blockers.push("seedCorpusBelowStageTwoMinimum");
  }
  if (input.consecutiveGreenRuns < input.requiredConsecutiveGreenRuns) {
    blockers.push("insufficientConsecutiveGreenRuns");
  }
  return blockers;
}

function pushKnownFailureSubtest(subtests: KnownFailureSubtestV0[], values: Map<string, string>) {
  if (values.size === 0) return;
  subtests.push({
    fixture: expectString(values, "fixture"),
    name: expectString(values, "name"),
    status: expectString(values, "status"),
    reason: expectString(values, "reason"),
    issue: expectString(values, "issue"),
    since: expectString(values, "since"),
    reviewAfter: expectString(values, "review_after"),
  });
}

function pushGreenRun(
  greenRuns: GreenRunEvidenceV0[],
  values: Map<string, string | boolean | number>,
) {
  if (values.size === 0) return;
  greenRuns.push({
    date: expectString(values, "date"),
    commit: expectString(values, "commit"),
    fixtureCount: expectNumber(values, "fixture_count"),
    chunkSha256: expectString(values, "chunk_sha256"),
    outcomeOlw: expectNumber(values, "outcome_olw"),
    criticalRegressionCount: expectNumber(values, "critical_regression_count"),
    command: expectString(values, "command"),
  });
}

function parseTomlScalar(rawValue: string): string | boolean | number {
  if (rawValue === "true") return true;
  if (rawValue === "false") return false;
  if (/^[0-9]+$/.test(rawValue)) return Number(rawValue);
  const stringMatch = /^"(.*)"$/.exec(rawValue);
  assert.ok(stringMatch, `unsupported TOML scalar: ${rawValue}`);
  return stringMatch[1];
}

function isKnownFailureSubtestKey(key: string): boolean {
  return ["fixture", "name", "status", "reason", "issue", "since", "review_after"].includes(key);
}

function expectString(values: Map<string, string | boolean | number>, key: string): string {
  const value = values.get(key);
  assert.equal(typeof value, "string", `${key} must be a string`);
  return value;
}

function expectBoolean(values: Map<string, string | boolean | number>, key: string): boolean {
  const value = values.get(key);
  assert.equal(typeof value, "boolean", `${key} must be a boolean`);
  return value;
}

function expectNumber(values: Map<string, string | boolean | number>, key: string): number {
  const value = values.get(key);
  assert.equal(typeof value, "number", `${key} must be a number`);
  return value;
}

function isIsoDate(value: string): boolean {
  return /^[0-9]{4}-[0-9]{2}-[0-9]{2}$/.test(value);
}

function isCommitId(value: string): boolean {
  return /^[0-9a-f]{8,40}$/.test(value);
}
