import { strict as assert } from "node:assert";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import path from "node:path";

const root = process.cwd();
const packageJson = JSON.parse(read("package.json")) as {
  readonly scripts: Record<string, string>;
};
const readinessScript = requiredScript("check:rust-m4-axis-a-readiness");

for (const target of [
  "rust/omena-diff-test-boundary",
  "rust/omena-spec-audit-boundary",
  "rust/omena-meta-macros-boundary",
  "rust/omena-transform-target/boundary",
  "rust/omena-testkit/boundary",
  "tooling/cme-checker-boundary",
  "rust/m4-axis-a-closure-audit",
] as const) {
  assertIncludes(readinessScript, target, `M4 Axis A readiness must include ${target}`);
}

const wptManifest = readJson<WptSeedManifestV0>(
  "rust/crates/omena-diff-test/wpt-corpus/manifest.json",
);
const wptPolicy = read("rust/crates/omena-diff-test/known-failures/wpt-seed-policy.toml");
const wptChunks = wptManifest.chunks.map((chunkManifest) => {
  const chunkPath = path.join("rust/crates/omena-diff-test/wpt-corpus", chunkManifest.path);
  const chunkSource = read(chunkPath);
  const chunk = JSON.parse(chunkSource) as WptSeedChunkV0;
  const sha256 = createHash("sha256").update(chunkSource).digest("hex");
  assert.equal(chunk.fixtures.length, chunkManifest.fixtureCount);
  assert.equal(sha256, chunkManifest.sha256);
  return { manifest: chunkManifest, chunk };
});
const wptBlockingChunks = wptChunks.filter((chunk) => chunk.manifest.stage === "stage2-blocking");
const wptAdvisoryFixtureCount = wptChunks
  .filter((chunk) => chunk.manifest.stage === "stage1-advisory")
  .reduce((count, chunk) => count + chunk.chunk.fixtures.length, 0);
assert.equal(wptBlockingChunks.length, 1);
const wptBlockingFixtureCount = wptBlockingChunks[0]?.chunk.fixtures.length ?? 0;
const wptPolicyStage2Blocking = readTomlBoolean(wptPolicy, "stage2_blocking");
const wptPolicyConsecutiveGreenRuns = readTomlNumber(wptPolicy, "consecutive_green_runs");
const wptPolicyRequiredGreenRuns = readTomlNumber(wptPolicy, "required_consecutive_green_runs");
const wptPolicyRequiredFixtureCount = readTomlNumber(
  wptPolicy,
  "required_min_fixture_count_for_stage2",
);
const greenRunEntries = [...wptPolicy.matchAll(/\[\[green_run\]\]/gu)].length;

assert.equal(wptManifest.stage, "stage2-blocking");
assert.equal(wptManifest.knownFailurePolicy.stage2Blocking, true);
assert.equal(wptPolicyStage2Blocking, true);
assert.equal(wptPolicyConsecutiveGreenRuns, greenRunEntries);
assert.ok(wptPolicyConsecutiveGreenRuns >= wptPolicyRequiredGreenRuns);
assert.ok(wptBlockingFixtureCount >= wptPolicyRequiredFixtureCount);
assert.ok(wptAdvisoryFixtureCount > 0, "M4 Axis A must keep a stage1 advisory WPT lane");
assert.ok(wptManifest.source.sparsePaths.includes("css/css-values"));
assert.ok(wptManifest.source.sparsePaths.includes("css/css-color"));
assert.ok(wptManifest.source.helperClasses.includes("test_valid_value"));
assert.ok(wptManifest.source.layoutDependentHelpersExcluded.includes("test_computed_value"));

const specSources = readJson<SpecSourcePinsV0>(
  "rust/crates/omena-spec-audit/data/spec-sources.json",
);
const specManifest = readJson<OmenaSpecManifestV0>(
  "rust/crates/omena-spec-audit/data/omena-spec-manifest.json",
);
const sourceNames = new Set(specSources.sources.map((source) => source.name));
for (const sourceName of [
  "webref-css",
  "browser-specs",
  "web-features",
  "mdn-browser-compat-data",
] as const) {
  assert.ok(sourceNames.has(sourceName), `spec audit must pin ${sourceName}`);
}
assert.equal(specSources.generatedDataReviewGate.humanReviewRequired, true);
assert.equal(specSources.generatedDataReviewGate.changedGeneratedDataRequiresReview, true);
assert.equal(specSources.generatedDataReviewGate.autoMergeAllowed, false);
assert.equal(specManifest.stage, "stage1-advisory");
const specSourceLinkedEntries = specManifest.entries.filter((entry) =>
  sourceNames.has(entry.sourceName),
);
const specManifestEntryIds = new Set(specManifest.entries.map((entry) => entry.id));
const specSourceCoverageNames = new Set(
  specManifest.sourceCoverage.map((coverage) => coverage.sourceName),
);
for (const sourceName of sourceNames) {
  assert.ok(
    specSourceCoverageNames.has(sourceName),
    `spec manifest source coverage must include ${sourceName}`,
  );
}
for (const coverage of specManifest.sourceCoverage) {
  assert.ok(sourceNames.has(coverage.sourceName), "source coverage must reference a pinned source");
  assert.ok(coverage.usage.trim().length > 0, "source coverage must declare usage");
  assert.ok(coverage.sourceKeys.length > 0, "source coverage must declare source keys");
  assert.ok(
    coverage.sourceKeys.every((sourceKey) => sourceKey.trim().length > 0),
    "source coverage keys must be non-empty",
  );
  assert.ok(
    coverage.entryIds.every((entryId) => specManifestEntryIds.has(entryId)),
    "source coverage entry ids must reference manifest entries",
  );
}
assert.ok(
  specManifest.entries
    .filter((entry) => entry.priority === "P0")
    .every((entry) => entry.status === "covered" || hasRationale(entry)),
  "P0 spec gaps must be covered or explicitly rationalized",
);
assert.equal(
  specSourceLinkedEntries.length,
  specManifest.entries.length,
  "every spec manifest entry must link to a pinned source",
);
assert.ok(
  specManifest.entries.every((entry) => entry.specUrl.startsWith("https://")),
  "every spec manifest entry must carry a spec URL",
);
assert.ok(
  specManifest.entries.every((entry) => entry.webrefId.length > 0),
  "every spec manifest entry must carry a webref id",
);

const metaMacroSource = read("rust/crates/omena-meta-macros/src/lib.rs");
for (const marker of [
  "pub fn spec",
  "pub fn pass",
  "validate_priority",
  "validate_ordinal",
  "reject_unknown_keys",
] as const) {
  assertIncludes(metaMacroSource, marker, `metadata macro substrate must retain ${marker}`);
}

const transformTargetSource = read("rust/crates/omena-transform-target/src/lib.rs");
const browserThresholds = read("rust/crates/omena-transform-target/data/browser-thresholds.toml");
const passFeatureBindings = read(
  "rust/crates/omena-transform-target/data/pass-feature-bindings.toml",
);
assertIncludes(
  transformTargetSource,
  "browser_data_quorum_valid",
  "target boundary must expose browser quorum",
);
assertIncludes(
  transformTargetSource,
  "browser_data_bindings_valid",
  "target boundary must expose pass-feature binding validation",
);
assert.ok(readTomlNumber(browserThresholds, "quorum_min_sources") >= 2);
assert.ok(countOccurrences(browserThresholds, "source_quorum = [") >= 2);
assertIncludes(
  passFeatureBindings,
  'pass_id = "light-dark-lowering"',
  "light-dark binding required",
);
assertIncludes(passFeatureBindings, 'pass_id = "color-mix-lowering"', "color-mix binding required");

const testkitBoundary = read("rust/crates/omena-testkit/src/boundary.rs");
const testkitFixture = read("rust/crates/omena-testkit/src/fixture.rs");
const testkitScenario = read("rust/crates/omena-testkit/src/scenario.rs");
const testkitSnapshot = read("rust/crates/omena-testkit/src/snapshot.rs");
for (const marker of [
  "sharedFixtureParserOwnedByOmenaTestkit",
  "crossLanguageFixtureGrammar",
  "fixtureHeaderMetadata",
  "fixtureMarkerOffsets",
  "lspScenarioMacro",
  "shadowOmenaVerbIntrospection",
  "snapshotGovernanceKnownFailurePolicy",
] as const) {
  assertIncludes(testkitBoundary, marker, `testkit boundary must retain ${marker}`);
}
assertIncludes(testkitFixture, "parse_cme_fixture_v0", "testkit must own the fixture parser");
assertIncludes(testkitScenario, "CmeScenarioArchetypeV0", "testkit must own scenario archetypes");
assertIncludes(
  testkitSnapshot,
  "allow_global_disable: false",
  "testkit snapshots must reject global disable",
);

const cmeCheckerSource = read("packages/cme-checker/src/testkit.ts");
const cmeCheckerGate = read("scripts/check-cme-checker-testkit-archetypes.ts");
for (const marker of [
  'bundle: "source-missing"',
  'bundle: "style-unused"',
  'bundle: "style-recovery"',
] as const) {
  assertIncludes(cmeCheckerSource, marker, `cme-checker testkit archetypes must retain ${marker}`);
}
assertIncludes(
  cmeCheckerGate,
  "parseFixtureWithOmenaTestkit",
  "cme-checker archetype gate must consume omena-testkit fixture parsing",
);
assertIncludes(
  cmeCheckerGate,
  "assertCheckerCanonicalCandidateEqual",
  "cme-checker archetype gate must compare canonical candidates",
);

process.stdout.write(
  JSON.stringify(
    {
      product: "rust.m4-axis-a-closure-audit",
      wpt: {
        stage: wptManifest.stage,
        fixtureCount: wptBlockingFixtureCount + wptAdvisoryFixtureCount,
        blockingFixtureCount: wptBlockingFixtureCount,
        advisoryFixtureCount: wptAdvisoryFixtureCount,
        chunkCount: wptChunks.length,
        consecutiveGreenRuns: wptPolicyConsecutiveGreenRuns,
        stage2Blocking: wptPolicyStage2Blocking,
      },
      specAudit: {
        stage: specManifest.stage,
        sourceCount: specSources.sources.length,
        p0EntryCount: specManifest.entries.filter((entry) => entry.priority === "P0").length,
        sourceLinkedEntryCount: specSourceLinkedEntries.length,
        sourceCoverageCount: specManifest.sourceCoverage.length,
      },
      browserData: {
        thresholdTables: uniqueTomlValues(browserThresholds, "table").length,
        passFeatureBindingCount: countOccurrences(passFeatureBindings, "[[binding]]"),
      },
      testkit: {
        fixtureGrammar: "cme-fixture-v0",
        checkerArchetypes: ["source-missing", "style-unused", "style-recovery"],
      },
    },
    null,
    2,
  ),
);
process.stdout.write("\n");

interface WptSeedManifestV0 {
  readonly stage: string;
  readonly source: {
    readonly sparsePaths: readonly string[];
    readonly helperClasses: readonly string[];
    readonly layoutDependentHelpersExcluded: readonly string[];
  };
  readonly knownFailurePolicy: {
    readonly stage2Blocking: boolean;
  };
  readonly chunks: readonly {
    readonly path: string;
    readonly stage: string;
    readonly sha256: string;
    readonly fixtureCount: number;
  }[];
}

interface WptSeedChunkV0 {
  readonly fixtures: readonly unknown[];
}

interface SpecSourcePinsV0 {
  readonly generatedDataReviewGate: {
    readonly humanReviewRequired: boolean;
    readonly changedGeneratedDataRequiresReview: boolean;
    readonly autoMergeAllowed: boolean;
  };
  readonly sources: readonly {
    readonly name: string;
  }[];
}

interface OmenaSpecManifestV0 {
  readonly stage: string;
  readonly sourceCoverage: readonly {
    readonly sourceName: string;
    readonly usage: string;
    readonly entryIds: readonly string[];
    readonly sourceKeys: readonly string[];
  }[];
  readonly entries: readonly {
    readonly id: string;
    readonly priority: string;
    readonly status: string;
    readonly rationale?: string;
    readonly sourceName: string;
    readonly specUrl: string;
    readonly webrefId: string;
  }[];
}

function read(relativePath: string): string {
  return readFileSync(path.join(root, relativePath), "utf8");
}

function readJson<T>(relativePath: string): T {
  return JSON.parse(read(relativePath)) as T;
}

function requiredScript(name: string): string {
  const script = packageJson.scripts[name];
  assert.equal(typeof script, "string", `${name} must be declared in package.json`);
  return script;
}

function assertIncludes(source: string, marker: string, message: string): void {
  assert.ok(source.includes(marker), message);
}

function readTomlNumber(source: string, key: string): number {
  const match = new RegExp(`^${key}\\s*=\\s*(\\d+)`, "mu").exec(source);
  assert.ok(match, `missing TOML number ${key}`);
  return Number(match[1]);
}

function readTomlBoolean(source: string, key: string): boolean {
  const match = new RegExp(`^${key}\\s*=\\s*(true|false)`, "mu").exec(source);
  assert.ok(match, `missing TOML boolean ${key}`);
  return match[1] === "true";
}

function countOccurrences(source: string, marker: string): number {
  return source.split(marker).length - 1;
}

function uniqueTomlValues(source: string, key: string): readonly string[] {
  const values = [...source.matchAll(new RegExp(`^${key}\\s*=\\s*"([^"]+)"`, "gmu"))].map(
    (match) => match[1]!,
  );
  return [...new Set(values)].toSorted();
}

function hasRationale(entry: { readonly rationale?: string }): boolean {
  return Boolean(entry.rationale?.trim());
}
