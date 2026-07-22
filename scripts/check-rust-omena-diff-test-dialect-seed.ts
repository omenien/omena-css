import { createHash } from "node:crypto";
import { strict as assert } from "node:assert";
import { readFileSync } from "node:fs";
import path from "node:path";

interface DialectSeedManifestV0 {
  readonly schemaVersion: string;
  readonly product: string;
  readonly stage: string;
  readonly source: {
    readonly kind: "pinned-repository";
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
  readonly sparsePathFixtureCounts: readonly SparsePathFixtureCountV0[];
  readonly chunks: readonly DialectSeedChunkManifestV0[];
}

interface DialectSeedChunkManifestV0 {
  readonly chunkId: string;
  readonly path: string;
  readonly stage: string;
  readonly sha256: string;
  readonly fixtureCount: number;
  readonly sparsePathFixtureCounts: readonly SparsePathFixtureCountV0[];
}

interface SparsePathFixtureCountV0 {
  readonly sparsePath: string;
  readonly fixtureCount: number;
}

interface DialectSeedChunkV0 {
  readonly schemaVersion: string;
  readonly product: string;
  readonly chunkId: string;
  readonly sourcePin: string;
  readonly fixtures: readonly DialectSeedFixtureV0[];
}

interface DialectSeedFixtureV0 {
  readonly id: string;
  readonly upstreamPath: string;
  readonly upstreamSourceLine: number;
  readonly subtest: string;
  readonly dialect: string;
  readonly source: string;
  readonly status: "pass" | "known-failure";
  readonly expectedBogusKinds: readonly string[];
  readonly expectedErrorCodes: readonly string[];
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
const corpusName = process.argv[2];
const config = corpusConfig(corpusName);
const corpusRoot = path.join(repoRoot, config.corpusRoot);
const manifestPath = path.join(corpusRoot, "manifest.json");
const manifest = readJson<DialectSeedManifestV0>(manifestPath);

assert.equal(manifest.schemaVersion, "0");
assert.equal(manifest.product, config.manifestProduct);
assert.equal(manifest.source.kind, "pinned-repository");
assert.equal(
  manifest.stage,
  manifest.knownFailurePolicy.stage2Blocking ? "stage2-blocking" : "stage1-advisory",
);
assert.equal(manifest.source.repository, config.repository);
assert.match(
  manifest.source.pin,
  config.pinPattern,
  `${corpusName} source pin must use a 40-character SHA`,
);
assert.ok(manifest.source.sparsePaths.length > 0, `${corpusName} must declare sparse paths`);
assert.ok(
  manifest.source.helperClasses.includes("parse_complete_tree"),
  `${corpusName} must declare the complete-tree helper`,
);
assert.equal(manifest.knownFailurePolicy.schemaVersion, "0");
assert.equal(manifest.generation.tool, "manual-seed");
assert.equal(manifest.generation.selectionPath, `${config.corpusRoot}/manifest.json`);

const policy = readKnownFailurePolicy(path.resolve(corpusRoot, manifest.knownFailurePolicy.path));
assert.equal(policy.schemaVersion, manifest.knownFailurePolicy.schemaVersion);
assert.equal(policy.corpusManifest, `../${path.basename(config.corpusRoot)}/manifest.json`);
assert.equal(policy.stage2Blocking, manifest.knownFailurePolicy.stage2Blocking);
assert.equal(policy.stage, policy.stage2Blocking ? "blocking" : "advisory");
assert.equal(policy.sourcePin, manifest.source.pin);
assert.ok(policy.reviewIntervalDays > 0);
assert.ok(policy.requiredMinFixtureCountForStage2 > 0);
assert.equal(policy.requiredConsecutiveGreenRuns, 5);
assert.equal(
  policy.consecutiveGreenRuns,
  policy.greenRuns.length,
  "consecutive green runs must be backed by explicit evidence entries",
);

const chunkRecords = manifest.chunks.map((chunkManifest) => {
  const chunkPath = path.join(corpusRoot, chunkManifest.path);
  const chunkSource = readFileSync(chunkPath, "utf8");
  const actualHash = createHash("sha256").update(chunkSource).digest("hex");
  assert.equal(actualHash, chunkManifest.sha256, `${chunkManifest.path} sha256 drift`);
  const chunk = JSON.parse(chunkSource) as DialectSeedChunkV0;
  assert.equal(chunk.schemaVersion, "0");
  assert.equal(chunk.product, config.chunkProduct);
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
assert.ok(fixtures.length > 0, `${corpusName} seed corpus must not be empty`);
assert.deepEqual(
  manifest.sparsePathFixtureCounts,
  countSparsePathFixtures(manifest.source.sparsePaths, fixtures),
  `${corpusName} manifest sparse-path counts drift`,
);
assert.ok(
  manifest.sparsePathFixtureCounts.every((count) => count.fixtureCount > 0),
  `${corpusName} every sparse path must have fixture coverage`,
);

const fixtureIds = new Set<string>();
const subtestKeys = new Set<string>();
for (const fixture of fixtures) {
  assert.ok(!fixtureIds.has(fixture.id), `${fixture.id} duplicate fixture id`);
  fixtureIds.add(fixture.id);
  subtestKeys.add(`${fixture.id}\n${fixture.subtest}`);
  assert.ok(fixture.upstreamPath.length > 0, `${fixture.id} upstream path missing`);
  assert.ok(fixture.upstreamSourceLine > 0, `${fixture.id} upstream source line missing`);
  assert.ok(
    manifest.source.sparsePaths.some(
      (sparsePath) =>
        fixture.upstreamPath.startsWith(`${sparsePath}/`) || fixture.upstreamPath === sparsePath,
    ),
    `${fixture.id} must belong to a pinned sparse path`,
  );
  assert.ok(config.dialects.includes(fixture.dialect), `${fixture.id} unsupported dialect`);
  assert.ok(fixture.source.length > 0, `${fixture.id} source missing`);
  assert.deepEqual(
    fixture.expectedBogusKinds,
    sortedUnique(fixture.expectedBogusKinds),
    `${fixture.id} bogus kinds must be sorted and unique`,
  );
  assert.deepEqual(
    fixture.expectedErrorCodes,
    sortedUnique(fixture.expectedErrorCodes),
    `${fixture.id} error codes must be sorted and unique`,
  );
  if (fixture.status === "pass") {
    assert.deepEqual(
      fixture.expectedBogusKinds,
      [],
      `${fixture.id} pass fixture cannot declare bogus kinds`,
    );
    assert.deepEqual(
      fixture.expectedErrorCodes,
      [],
      `${fixture.id} pass fixture cannot declare error codes`,
    );
  } else {
    assert.ok(
      fixture.expectedBogusKinds.length + fixture.expectedErrorCodes.length > 0,
      `${fixture.id} known-failure fixture must record a non-empty bogus/error set`,
    );
  }
}

const staleKnownFailures: string[] = [];
for (const subtest of policy.subtests) {
  const key = `${subtest.fixture}\n${subtest.name}`;
  if (!fixtureIds.has(subtest.fixture) || !subtestKeys.has(key)) staleKnownFailures.push(key);
  assert.match(subtest.status, /^(fail|implementation-defined)$/);
  assert.ok(subtest.reason.length > 0);
  assert.ok(subtest.issue.length > 0);
  assert.ok(isIsoDate(subtest.since), `${subtest.fixture} since must be an ISO date`);
  assert.ok(isIsoDate(subtest.reviewAfter), `${subtest.fixture} review_after must be an ISO date`);
}
assert.deepEqual(
  staleKnownFailures,
  [],
  `${corpusName} known-failure policy contains stale entries`,
);
assert.deepEqual(
  new Set(
    fixtures
      .filter(
        (fixture) => fixture.expectedBogusKinds.length + fixture.expectedErrorCodes.length > 0,
      )
      .map((fixture) => `${fixture.id}\n${fixture.subtest}`),
  ),
  new Set(policy.subtests.map((subtest) => `${subtest.fixture}\n${subtest.name}`)),
  `${corpusName} recorded fixture set must match policy entries`,
);

for (const run of policy.greenRuns) {
  assert.ok(isIsoDate(run.date), `${run.commit} green-run date must be an ISO date`);
  assert.ok(isCommitId(run.commit), `${run.commit} must be a commit id`);
  assert.ok(run.chunkSha256.length === 64, `${run.commit} chunk sha must be sha256`);
  assert.equal(run.criticalRegressionCount, 0);
  assert.ok(
    run.command.includes(config.checkName),
    `${run.commit} command must point at ${config.checkName}`,
  );
}

process.stdout.write(
  JSON.stringify(
    {
      product: "omena-diff-test.dialect-seed-manifest-report",
      corpus: corpusName,
      sourcePin: manifest.source.pin,
      fixtureCount: fixtures.length,
      chunkCount: chunkRecords.length,
      knownFailureCount: policy.subtests.length,
      stage2Blocking: policy.stage2Blocking,
      requiredConsecutiveGreenRuns: policy.requiredConsecutiveGreenRuns,
      consecutiveGreenRuns: policy.consecutiveGreenRuns,
    },
    null,
    2,
  ),
);
process.stdout.write("\n");

function corpusConfig(name: string | undefined): {
  readonly corpusRoot: string;
  readonly manifestProduct: string;
  readonly chunkProduct: string;
  readonly repository: string;
  readonly pinPattern: RegExp;
  readonly dialects: readonly string[];
  readonly checkName: string;
} {
  if (name === "sass-spec") {
    return {
      corpusRoot: "rust/crates/omena-diff-test/sass-spec-corpus",
      manifestProduct: "omena-diff-test.sass-spec-seed-corpus.manifest",
      chunkProduct: "omena-diff-test.sass-spec-seed-corpus.chunk",
      repository: "https://github.com/sass/sass-spec",
      pinPattern: /^sass\/sass-spec@[0-9a-f]{40}$/,
      dialects: ["scss", "sass"],
      checkName: "rust/omena-diff-test-sass-spec-seed",
    };
  }
  if (name === "less") {
    return {
      corpusRoot: "rust/crates/omena-diff-test/less-corpus",
      manifestProduct: "omena-diff-test.less-seed-corpus.manifest",
      chunkProduct: "omena-diff-test.less-seed-corpus.chunk",
      repository: "https://github.com/less/less.js",
      pinPattern: /^less\/less\.js@[0-9a-f]{40}$/,
      dialects: ["less"],
      checkName: "rust/omena-diff-test-less-seed",
    };
  }
  throw new Error(`usage: check-rust-omena-diff-test-dialect-seed.ts sass-spec|less`);
}

function readJson<T>(filePath: string): T {
  return JSON.parse(readFileSync(filePath, "utf8")) as T;
}

function countSparsePathFixtures(
  sparsePaths: readonly string[],
  fixtureSet: readonly DialectSeedFixtureV0[],
): readonly SparsePathFixtureCountV0[] {
  return sparsePaths.map((sparsePath) => ({
    sparsePath,
    fixtureCount: fixtureSet.filter(
      (fixture) =>
        fixture.upstreamPath.startsWith(`${sparsePath}/`) || fixture.upstreamPath === sparsePath,
    ).length,
  }));
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

function sortedUnique(values: readonly string[]): readonly string[] {
  return [...new Set(values)].sort();
}

function isIsoDate(value: string): boolean {
  return /^[0-9]{4}-[0-9]{2}-[0-9]{2}$/.test(value);
}

function isCommitId(value: string): boolean {
  return /^[0-9a-f]{8,40}$/.test(value);
}
