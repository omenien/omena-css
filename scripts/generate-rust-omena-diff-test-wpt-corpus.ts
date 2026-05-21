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
  readonly chunks: readonly WptSeedChunkManifestV0[];
}

interface WptSeedChunkManifestV0 {
  readonly chunkId: string;
  readonly path: string;
  readonly sha256: string;
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

const repoRoot = process.cwd();
const checkOnly = process.argv.includes("--check");
const corpusRoot = path.join(repoRoot, "rust/crates/omena-diff-test/wpt-corpus");
const selectionsPath = path.join(corpusRoot, "selections.json");
const manifestPath = path.join(corpusRoot, "manifest.json");
const toolPath = "scripts/generate-rust-omena-diff-test-wpt-corpus.ts";
const selectionPath = "rust/crates/omena-diff-test/wpt-corpus/selections.json";

const selectionFile = readJson<WptSeedSelectionsV0>(selectionsPath);
validateSelections(selectionFile);

const chunk: WptSeedChunkV0 = {
  schemaVersion: "0",
  product: "omena-diff-test.wpt-seed-corpus.chunk",
  chunkId: selectionFile.chunkId,
  sourcePin: selectionFile.sourcePin,
  fixtures: selectionFile.fixtures,
};
const chunkSource = stableJson(chunk);
const chunkSha256 = createHash("sha256").update(chunkSource).digest("hex");
const manifest: WptSeedManifestV0 = {
  schemaVersion: "0",
  product: "omena-diff-test.wpt-seed-corpus.manifest",
  stage: "stage1-advisory",
  source: selectionFile.source,
  knownFailurePolicy: selectionFile.knownFailurePolicy,
  generation: {
    tool: toolPath,
    selectionPath,
  },
  chunks: [
    {
      chunkId: selectionFile.chunkId,
      path: selectionFile.chunkPath,
      sha256: chunkSha256,
      fixtureCount: selectionFile.fixtures.length,
    },
  ],
};
const manifestSource = stableJson(manifest);
const chunkPath = path.join(corpusRoot, selectionFile.chunkPath);

if (checkOnly) {
  assert.equal(readFileSync(chunkPath, "utf8"), chunkSource, `${selectionFile.chunkPath} is stale`);
  assert.equal(readFileSync(manifestPath, "utf8"), manifestSource, "manifest.json is stale");
} else {
  writeFileSync(chunkPath, chunkSource);
  writeFileSync(manifestPath, manifestSource);
}

process.stdout.write(
  stableJson({
    product: "omena-diff-test.wpt-corpus-generator",
    mode: checkOnly ? "check" : "write",
    sourcePin: selectionFile.sourcePin,
    fixtureCount: selectionFile.fixtures.length,
    chunkSha256,
    generatedFiles: ["manifest.json", selectionFile.chunkPath],
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
  for (const fixture of candidate.fixtures) {
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
    return `[${values.join(", ")}]`;
  });
}

function isPinnedWptSha(pin: string): boolean {
  return /^web-platform-tests\/wpt@[0-9a-f]{40}$/.test(pin);
}
