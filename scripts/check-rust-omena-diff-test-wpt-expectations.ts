import { strict as assert } from "node:assert";
import { createHash } from "node:crypto";
import { readFileSync, readdirSync } from "node:fs";
import path from "node:path";

const EXPECTATION_REASONS = [
  "engine-validity-disagreement",
  "engine-serialization-difference",
  "unsupported-semantic-feature",
  "external-oracle-limitation",
  "unstable",
] as const;

interface WptManifestV0 {
  readonly extraction: {
    readonly sourcePin: string;
    readonly tuples: { readonly path: string };
    readonly moduleCoverage: readonly {
      readonly moduleId: string;
      readonly wptPath: string;
    }[];
  };
}

interface WptTupleArtifactV0 {
  readonly tuples: readonly {
    readonly id: string;
    readonly moduleId: string;
    readonly wptPath: string;
  }[];
}

interface WptExpectationManifestV0 {
  readonly schemaVersion: string;
  readonly product: string;
  readonly moduleId: string;
  readonly wptPath: string;
  readonly sourcePin: string;
  readonly reasonVocabulary: readonly string[];
  readonly expectations: readonly {
    readonly tupleId: string;
    readonly name: string;
    readonly status: "expected-failure" | "quarantined";
    readonly reasonCode: string;
    readonly adjudicationId: string;
    readonly reviewedAt: string;
    readonly reviewAfter: string;
  }[];
}

interface WptAdjudicationLedgerV0 {
  readonly schemaVersion: string;
  readonly product: string;
  readonly reviewPolicy: {
    readonly humanReviewRequired: boolean;
    readonly autoUpdateAllowed: boolean;
    readonly reviewerRole: string;
  };
  readonly records: readonly {
    readonly manifestPath: string;
    readonly manifestSha256: string;
    readonly reviewer: string;
    readonly reviewedAt: string;
    readonly decision: string;
    readonly note: string;
  }[];
}

interface SpecSourcesV0 {
  readonly generatedDataReviewGate: {
    readonly humanReviewRequired: boolean;
    readonly changedGeneratedDataRequiresReview: boolean;
    readonly autoMergeAllowed: boolean;
  };
  readonly sources: readonly {
    readonly name: string;
    readonly package: string;
    readonly version: string;
    readonly repository?: string;
    readonly repoPin?: string;
    readonly declaredVersionSource?: string;
    readonly modulePath?: string;
    readonly sharedPinReason?: string;
    readonly role: string;
  }[];
}

const repoRoot = process.cwd();
const corpusRoot = path.join(repoRoot, "rust/crates/omena-diff-test/wpt-corpus");
const expectationRoot = path.join(corpusRoot, "expectations");
const ledgerPath = path.join(corpusRoot, "adjudications/reviewed-expectations.json");
const specSourcesPath = path.join(repoRoot, "rust/crates/omena-spec-audit/data/spec-sources.json");
const manifest = readJson<WptManifestV0>(path.join(corpusRoot, "manifest.json"));
const tuples = readJson<WptTupleArtifactV0>(path.join(corpusRoot, manifest.extraction.tuples.path));
const ledger = readJson<WptAdjudicationLedgerV0>(ledgerPath);
const specSources = readJson<SpecSourcesV0>(specSourcesPath);

assert.equal(ledger.schemaVersion, "0");
assert.equal(ledger.product, "omena-diff-test.wpt-expectation-adjudications");
assert.equal(ledger.reviewPolicy.humanReviewRequired, true);
assert.equal(ledger.reviewPolicy.autoUpdateAllowed, false);
assert.equal(ledger.reviewPolicy.reviewerRole, "maintainer");
assert.equal(specSources.generatedDataReviewGate.humanReviewRequired, true);
assert.equal(specSources.generatedDataReviewGate.changedGeneratedDataRequiresReview, true);
assert.equal(specSources.generatedDataReviewGate.autoMergeAllowed, false);

const expectedPaths = manifest.extraction.moduleCoverage
  .map((module) => `expectations/${module.wptPath}.json`)
  .sort();
const observedPaths = listJsonFiles(expectationRoot)
  .map((filePath) => path.relative(corpusRoot, filePath).split(path.sep).join("/"))
  .sort();
assert.deepEqual(
  observedPaths,
  expectedPaths,
  "expectation manifests must mirror WPT module paths",
);

const tupleById = new Map(tuples.tuples.map((tuple) => [tuple.id, tuple] as const));
const adjudicationByPath = new Map(
  ledger.records.map((record) => [record.manifestPath, record] as const),
);
assert.equal(
  adjudicationByPath.size,
  ledger.records.length,
  "duplicate expectation adjudication path",
);
assert.deepEqual([...adjudicationByPath.keys()].sort(), expectedPaths);

const seenExpectationIds = new Set<string>();
let expectedFailureCount = 0;
let quarantinedCount = 0;
for (const module of manifest.extraction.moduleCoverage) {
  const relativePath = `expectations/${module.wptPath}.json`;
  const absolutePath = path.join(corpusRoot, relativePath);
  const source = readFileSync(absolutePath, "utf8");
  const expectation = JSON.parse(source) as WptExpectationManifestV0;
  assert.equal(expectation.schemaVersion, "0");
  assert.equal(expectation.product, "omena-diff-test.wpt-tier-zero-expectations");
  assert.equal(expectation.moduleId, module.moduleId);
  assert.equal(expectation.wptPath, module.wptPath);
  assert.equal(expectation.sourcePin, manifest.extraction.sourcePin);
  assert.deepEqual(expectation.reasonVocabulary, EXPECTATION_REASONS);

  for (const entry of expectation.expectations) {
    assert.ok(!seenExpectationIds.has(entry.tupleId), `duplicate expectation: ${entry.tupleId}`);
    seenExpectationIds.add(entry.tupleId);
    const tuple = tupleById.get(entry.tupleId);
    assert.ok(tuple, `expectation references an unknown tuple: ${entry.tupleId}`);
    assert.equal(tuple.moduleId, module.moduleId, `${entry.tupleId} module drift`);
    assert.ok(tuple.wptPath.startsWith(`${module.wptPath}/`), `${entry.tupleId} path drift`);
    assert.ok(entry.name.trim().length > 0, `${entry.tupleId} needs a name`);
    assert.ok(
      EXPECTATION_REASONS.includes(entry.reasonCode as (typeof EXPECTATION_REASONS)[number]),
    );
    assert.ok(entry.adjudicationId.trim().length > 0, `${entry.tupleId} needs adjudication`);
    assert.match(entry.reviewedAt, /^\d{4}-\d{2}-\d{2}$/u);
    assert.match(entry.reviewAfter, /^\d{4}-\d{2}-\d{2}$/u);
    assert.ok(
      entry.reviewAfter >= entry.reviewedAt,
      `${entry.tupleId} review interval is inverted`,
    );
    if (entry.reasonCode === "unstable") {
      assert.equal(
        entry.status,
        "quarantined",
        `${entry.tupleId} unstable entries are quarantined`,
      );
      quarantinedCount += 1;
    } else {
      assert.equal(entry.status, "expected-failure", `${entry.tupleId} has an invalid status`);
      expectedFailureCount += 1;
    }
  }

  const adjudication = adjudicationByPath.get(relativePath);
  assert.ok(adjudication, `${relativePath} lacks a reviewed adjudication record`);
  assert.equal(
    adjudication.manifestSha256,
    createHash("sha256").update(source).digest("hex"),
    `${relativePath} changed without a matching reviewed adjudication`,
  );
  assert.equal(adjudication.reviewer, ledger.reviewPolicy.reviewerRole);
  assert.match(adjudication.reviewedAt, /^\d{4}-\d{2}-\d{2}$/u);
  assert.equal(adjudication.decision, "reviewed-expectations");
  assert.ok(adjudication.note.trim().length > 0, `${relativePath} adjudication needs a note`);
}

assert.ok(expectedFailureCount > 0, "the expectation lane needs reviewed non-vacuity");
const wptSources = specSources.sources.filter(
  (source) => source.role === "external-wpt-corpus-module",
);
assert.equal(wptSources.length, manifest.extraction.moduleCoverage.length);
const sourceNames = new Set<string>();
for (const module of manifest.extraction.moduleCoverage) {
  const source = wptSources.find((candidate) => candidate.modulePath === module.wptPath);
  assert.ok(source, `${module.moduleId} lacks an independent WPT source row`);
  assert.ok(!sourceNames.has(source.name), `duplicate WPT source name: ${source.name}`);
  sourceNames.add(source.name);
  assert.equal(source.package, "web-platform-tests/wpt");
  assert.equal(source.version, manifest.extraction.sourcePin.split("@")[1]);
  assert.equal(source.repository, "https://github.com/web-platform-tests/wpt");
  assert.equal(source.repoPin, manifest.extraction.sourcePin);
  assert.equal(
    source.declaredVersionSource,
    `rust/crates/omena-diff-test/wpt-corpus/expectations/${module.wptPath}.json#sourcePin`,
  );
  assert.ok(
    source.sharedPinReason?.trim(),
    `${source.name} must explain a shared extraction epoch`,
  );
}

process.stdout.write(
  `${JSON.stringify({
    product: "omena-diff-test.wpt-expectation-gate",
    moduleCount: expectedPaths.length,
    expectedFailureCount,
    quarantinedCount,
    independentPinCount: wptSources.length,
    humanReviewRequired: true,
    autoUpdateAllowed: false,
  })}\n`,
);

function readJson<T>(filePath: string): T {
  return JSON.parse(readFileSync(filePath, "utf8")) as T;
}

function listJsonFiles(root: string): string[] {
  const files: string[] = [];
  for (const entry of readdirSync(root, { withFileTypes: true })) {
    const entryPath = path.join(root, entry.name);
    if (entry.isDirectory()) files.push(...listJsonFiles(entryPath));
    else if (entry.isFile() && entry.name.endsWith(".json")) files.push(entryPath);
  }
  return files;
}
