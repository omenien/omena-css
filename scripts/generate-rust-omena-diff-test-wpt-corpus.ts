import { strict as assert } from "node:assert";
import { createHash } from "node:crypto";
import { readFileSync, writeFileSync } from "node:fs";
import path from "node:path";

interface WptSeedSelectionsV0 {
  readonly schemaVersion: string;
  readonly product: string;
  readonly sourcePin: string;
  readonly chunkId: string;
  readonly chunkPath: string;
  readonly source: WptSeedManifestV0["source"];
  readonly knownFailurePolicy: WptSeedManifestV0["knownFailurePolicy"];
  readonly fixtures: readonly WptSeedFixtureV0[];
  readonly advisoryChunks?: readonly WptSeedSelectionChunkV0[];
}

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
  readonly extraction: WptTierZeroExtractionManifestV0;
}

interface WptTierZeroExtractionManifestV0 {
  readonly tool: string;
  readonly sourcePin: string;
  readonly tuples: WptDerivedArtifactManifestV0;
  readonly coverage: WptDerivedArtifactManifestV0;
  readonly moduleCoverage: readonly WptTierZeroModuleCoverageV0[];
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
  readonly modules: readonly { readonly moduleId: string; readonly wptPath: string }[];
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

interface WptTierZeroCoverageArtifactV0 {
  readonly schemaVersion: string;
  readonly product: string;
  readonly sourcePin: string;
  readonly moduleCount: number;
  readonly extractedSubtestCount: number;
  readonly skippedDynamicCallCount: number;
  readonly modules: readonly WptTierZeroModuleCoverageV0[];
  readonly skippedDynamicCalls: readonly unknown[];
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

interface WptSeedSelectionChunkV0 {
  readonly chunkId: string;
  readonly chunkPath: string;
  readonly stage: "stage1-advisory";
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

const repoRoot = process.cwd();
const checkOnly = process.argv.includes("--check");
const corpusRoot = path.join(repoRoot, "rust/crates/omena-diff-test/wpt-corpus");
const selectionsPath = path.join(corpusRoot, "selections.json");
const manifestPath = path.join(corpusRoot, "manifest.json");
const toolPath = "scripts/generate-rust-omena-diff-test-wpt-corpus.ts";
const selectionPath = "rust/crates/omena-diff-test/wpt-corpus/selections.json";
const extractionToolPath = "scripts/extract-rust-omena-diff-test-wpt-tier-zero.ts";
const extractedTuplePath = "extracted/tier-zero-tuples.json";
const extractedCoveragePath = "extracted/tier-zero-coverage.json";

const selectionFile = readJson<WptSeedSelectionsV0>(selectionsPath);
validateSelections(selectionFile);
const extractedTupleSource = readFileSync(path.join(corpusRoot, extractedTuplePath), "utf8");
const extractedCoverageSource = readFileSync(path.join(corpusRoot, extractedCoveragePath), "utf8");
const extractedTuples = JSON.parse(extractedTupleSource) as WptTierZeroTupleArtifactV0;
const extractedCoverage = JSON.parse(extractedCoverageSource) as WptTierZeroCoverageArtifactV0;
validateExtraction(extractedTuples, extractedCoverage);

const selectedChunks = [
  {
    chunkId: selectionFile.chunkId,
    chunkPath: selectionFile.chunkPath,
    stage: selectionFile.knownFailurePolicy.stage2Blocking ? "stage2-blocking" : "stage1-advisory",
    fixtures: selectionFile.fixtures,
  },
  ...(selectionFile.advisoryChunks ?? []),
] as const;
const generatedChunks = selectedChunks.map((selectedChunk) => {
  const chunk: WptSeedChunkV0 = {
    schemaVersion: "0",
    product: "omena-diff-test.wpt-seed-corpus.chunk",
    chunkId: selectedChunk.chunkId,
    sourcePin: selectionFile.sourcePin,
    fixtures: selectedChunk.fixtures,
  };
  const source = stableJson(chunk);
  const sha256 = createHash("sha256").update(source).digest("hex");
  return {
    chunkId: selectedChunk.chunkId,
    chunkPath: selectedChunk.chunkPath,
    stage: selectedChunk.stage,
    fixtures: selectedChunk.fixtures,
    sparsePathFixtureCounts: sparsePathFixtureCounts(
      selectionFile.source.sparsePaths,
      selectedChunk.fixtures,
    ),
    source,
    sha256,
  };
});
const manifest: WptSeedManifestV0 = {
  schemaVersion: "0",
  product: "omena-diff-test.wpt-seed-corpus.manifest",
  stage: selectionFile.knownFailurePolicy.stage2Blocking ? "stage2-blocking" : "stage1-advisory",
  source: selectionFile.source,
  knownFailurePolicy: selectionFile.knownFailurePolicy,
  generation: {
    tool: toolPath,
    selectionPath,
  },
  sparsePathFixtureCounts: sparsePathFixtureCounts(
    selectionFile.source.sparsePaths,
    selectedChunks.flatMap((chunk) => chunk.fixtures),
  ),
  chunks: generatedChunks.map((chunk) => ({
    chunkId: chunk.chunkId,
    path: chunk.chunkPath,
    stage: chunk.stage,
    sha256: chunk.sha256,
    fixtureCount: chunk.fixtures.length,
    sparsePathFixtureCounts: chunk.sparsePathFixtureCounts,
  })),
  extraction: {
    tool: extractionToolPath,
    sourcePin: extractedTuples.source.pin,
    tuples: {
      path: extractedTuplePath,
      sha256: createHash("sha256").update(extractedTupleSource).digest("hex"),
      recordCount: extractedTuples.tuples.length,
    },
    coverage: {
      path: extractedCoveragePath,
      sha256: createHash("sha256").update(extractedCoverageSource).digest("hex"),
      recordCount: extractedCoverage.skippedDynamicCalls.length,
    },
    moduleCoverage: extractedCoverage.modules,
  },
};
const manifestSource = stableJson(manifest);

if (checkOnly) {
  for (const chunk of generatedChunks) {
    assert.equal(
      readFileSync(path.join(corpusRoot, chunk.chunkPath), "utf8"),
      chunk.source,
      `${chunk.chunkPath} is stale`,
    );
  }
  assert.equal(readFileSync(manifestPath, "utf8"), manifestSource, "manifest.json is stale");
} else {
  for (const chunk of generatedChunks) {
    writeFileSync(path.join(corpusRoot, chunk.chunkPath), chunk.source);
  }
  writeFileSync(manifestPath, manifestSource);
}

process.stdout.write(
  stableJson({
    product: "omena-diff-test.wpt-corpus-generator",
    mode: checkOnly ? "check" : "write",
    sourcePin: selectionFile.sourcePin,
    fixtureCount: generatedChunks.reduce((count, chunk) => count + chunk.fixtures.length, 0),
    chunkCount: generatedChunks.length,
    extractedTupleCount: extractedTuples.tuples.length,
    skippedDynamicCallCount: extractedCoverage.skippedDynamicCalls.length,
    chunks: generatedChunks.map((chunk) => ({
      chunkId: chunk.chunkId,
      stage: chunk.stage,
      fixtureCount: chunk.fixtures.length,
      sha256: chunk.sha256,
    })),
    generatedFiles: ["manifest.json", ...generatedChunks.map((chunk) => chunk.chunkPath)],
  }),
);

function validateSelections(candidate: WptSeedSelectionsV0): void {
  assert.equal(candidate.schemaVersion, "0");
  assert.equal(candidate.product, "omena-diff-test.wpt-seed-corpus.selections");
  assert.equal(candidate.source.pin, candidate.sourcePin);
  assert.ok(isPinnedWptSha(candidate.sourcePin), "sourcePin must be a full WPT SHA");
  assert.ok(candidate.source.repository.endsWith("/web-platform-tests/wpt"));
  assert.ok(candidate.source.sparsePaths.length > 0);
  assert.ok(candidate.source.helperClasses.includes("test_valid_value"));
  assert.ok(candidate.knownFailurePolicy.path.endsWith("wpt-seed-policy.toml"));
  assert.equal(candidate.knownFailurePolicy.schemaVersion, "0");
  assert.equal(typeof candidate.knownFailurePolicy.stage2Blocking, "boolean");

  const ids = new Set<string>();
  for (const chunk of [
    {
      chunkId: candidate.chunkId,
      chunkPath: candidate.chunkPath,
      stage: candidate.knownFailurePolicy.stage2Blocking ? "stage2-blocking" : "stage1-advisory",
      fixtures: candidate.fixtures,
    },
    ...(candidate.advisoryChunks ?? []),
  ] as const) {
    assert.ok(chunk.chunkId.length > 0, "chunk id must not be empty");
    assert.ok(chunk.chunkPath.endsWith(".json"), `${chunk.chunkId} chunk path must be JSON`);
    assert.match(chunk.stage, /^stage[12]-(advisory|blocking)$/u);
    if (chunk !== undefined && "stage" in chunk && chunk.stage === "stage2-blocking") {
      assert.equal(
        chunk.chunkId,
        candidate.chunkId,
        "only the primary seed chunk may be stage2-blocking",
      );
    }
    for (const fixture of chunk.fixtures) {
      assert.ok(!ids.has(fixture.id), `duplicate fixture id: ${fixture.id}`);
      ids.add(fixture.id);
      assert.equal(fixture.status, "pass", fixture.id);
      assert.ok(
        candidate.source.helperClasses.includes(fixture.helper),
        `${fixture.id} helper is not allowed by manifest source policy`,
      );
      assert.ok(
        candidate.source.sparsePaths.some((sparsePath) =>
          fixture.wptPath.startsWith(`${sparsePath}/`),
        ),
        `${fixture.id} is outside the sparse WPT path policy`,
      );
      assert.ok(fixture.wptSourceLine > 0, `${fixture.id} needs a WPT source line`);
      assert.ok(fixture.subtest.includes(fixture.wptValue), `${fixture.id} value is not sourced`);
      assert.ok(
        fixture.subtest.includes(fixture.wptExpectedValue),
        `${fixture.id} expected value is not sourced`,
      );
      assert.ok(fixture.source.includes(fixture.property), `${fixture.id} source misses property`);
      assert.ok(fixture.source.includes(fixture.wptValue), `${fixture.id} source misses WPT value`);
      assert.ok(fixture.expectedCss.length > 0, `${fixture.id} needs expected CSS`);
    }
  }
}

function validateExtraction(
  tuples: WptTierZeroTupleArtifactV0,
  coverage: WptTierZeroCoverageArtifactV0,
): void {
  assert.equal(tuples.schemaVersion, "0");
  assert.equal(tuples.product, "omena-diff-test.wpt-tier-zero-tuples");
  assert.ok(tuples.source.repository.endsWith("/web-platform-tests/wpt"));
  assert.ok(isPinnedWptSha(tuples.source.pin));
  assert.equal(tuples.source.extractionMode, "static-helper-call-sites");
  assert.equal(tuples.source.testharnessExecuted, false);
  assert.equal(coverage.schemaVersion, "0");
  assert.equal(coverage.product, "omena-diff-test.wpt-tier-zero-coverage");
  assert.equal(coverage.sourcePin, tuples.source.pin);
  assert.equal(coverage.moduleCount, tuples.modules.length);
  assert.equal(coverage.modules.length, tuples.modules.length);
  assert.equal(coverage.extractedSubtestCount, tuples.tuples.length);
  assert.equal(coverage.skippedDynamicCallCount, coverage.skippedDynamicCalls.length);
  assert.equal(
    coverage.modules.reduce((count, module) => count + module.extractedSubtestCount, 0),
    tuples.tuples.length,
  );
  assert.equal(
    coverage.modules.reduce((count, module) => count + module.skippedDynamicCallCount, 0),
    coverage.skippedDynamicCalls.length,
  );

  const moduleIds = new Set(tuples.modules.map((module) => module.moduleId));
  assert.equal(moduleIds.size, tuples.modules.length);
  for (const module of coverage.modules) {
    assert.ok(moduleIds.has(module.moduleId), `${module.moduleId} is not declared by extraction`);
    assert.equal(
      module.htmlFileCount,
      module.eligibleTierZeroFileCount +
        module.nonTierZeroFileCount +
        module.excludedTentativeFileCount +
        module.excludedOptionalFileCount,
      `${module.moduleId} extraction buckets must cover every HTML file`,
    );
    assert.equal(
      Object.values(module.skippedDynamicReasons).reduce((count, value) => count + value, 0),
      module.skippedDynamicCallCount,
      `${module.moduleId} skipped-call reasons must be total`,
    );
  }

  const ids = new Set<string>();
  for (const tuple of tuples.tuples) {
    assert.ok(!ids.has(tuple.id), `duplicate extracted tuple id: ${tuple.id}`);
    ids.add(tuple.id);
    assert.ok(moduleIds.has(tuple.moduleId), `${tuple.id} has an unknown module`);
    assert.ok(tuple.wptSourceLine > 0, `${tuple.id} needs a source line`);
    assert.ok(!/\.(?:tentative|optional)\./u.test(tuple.wptPath), `${tuple.id} is excluded`);
    assert.equal(
      tuple.sourceTextSha256,
      createHash("sha256").update(tuple.subtest).digest("hex"),
      `${tuple.id} source hash drift`,
    );
    assert.ok(tuple.helperClass.length > 0, `${tuple.id} needs a helper class`);
    assert.ok(tuple.helperCall.length > 0, `${tuple.id} needs a helper call`);
    assert.ok(tuple.property.length > 0, `${tuple.id} needs a property or subject marker`);
    assert.ok(tuple.value.length > 0, `${tuple.id} needs a source value`);
    assert.equal(
      tuple.expectedValidity === "valid",
      tuple.expectedValues.length > 0,
      `${tuple.id} expected-set cardinality does not match validity`,
    );
    assert.equal(
      new Set(tuple.expectedValues).size,
      tuple.expectedValues.length,
      `${tuple.id} expected set contains duplicates`,
    );
  }
}

function readJson<T>(filePath: string): T {
  return JSON.parse(readFileSync(filePath, "utf8")) as T;
}

function stableJson(value: unknown): string {
  return `${inlineStringArrays(JSON.stringify(value, null, 2))}\n`;
}

function inlineStringArrays(source: string): string {
  return source.replace(/\[\n((?:\s+"(?:[^"\\]|\\.)*"(?:,\n)?)+)\s+\]/g, (_match, body) => {
    const values = String(body)
      .trim()
      .split(/\n/)
      .map((line) => line.trim().replace(/,$/, ""));
    const inlineArray = `[${values.join(", ")}]`;
    return inlineArray.length <= 80 ? inlineArray : String(_match);
  });
}

function isPinnedWptSha(pin: string): boolean {
  return /^web-platform-tests\/wpt@[0-9a-f]{40}$/.test(pin);
}

function sparsePathFixtureCounts(
  sparsePaths: readonly string[],
  fixtures: readonly WptSeedFixtureV0[],
): readonly WptSparsePathFixtureCountV0[] {
  return sparsePaths.map((sparsePath) => ({
    sparsePath,
    fixtureCount: fixtures.filter((fixture) => fixture.wptPath.startsWith(`${sparsePath}/`)).length,
  }));
}
