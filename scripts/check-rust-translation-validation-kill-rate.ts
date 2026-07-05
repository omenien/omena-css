import { strict as assert } from "node:assert";
import { spawnSync } from "node:child_process";
import { readFileSync } from "node:fs";
import path from "node:path";

interface BrokenTranslationFixture {
  readonly passId: string;
  readonly input: string;
  readonly output: string;
  readonly expectedRejected: boolean;
}

const repoRoot = process.cwd();
const corpusPath = path.join(
  repoRoot,
  "rust/crates/omena-transform-passes/fixtures/semantic-preservation/broken-simple.json",
);
const corpus = JSON.parse(readFileSync(corpusPath, "utf8")) as readonly BrokenTranslationFixture[];

assert.ok(corpus.length > 0, "semantic preservation corpus must not be empty");
const supportedPassIds = new Set(["empty-rule-removal", "rule-deduplication"]);
const expectedRejectedCount = corpus.filter((fixture) => fixture.expectedRejected).length;
assert.ok(expectedRejectedCount > 0, "semantic preservation corpus must contain rejected cases");

for (const [index, fixture] of corpus.entries()) {
  assert.ok(supportedPassIds.has(fixture.passId), `unsupported pass id at fixture ${index}`);
  assert.ok(fixture.input.length > 0, `fixture ${index} input must not be empty`);
  assert.ok(fixture.output.length > 0, `fixture ${index} output must not be empty`);
  assert.notEqual(
    fixture.input,
    fixture.output,
    `fixture ${index} must describe a changed translation`,
  );
}

const result = spawnSync(
  "cargo",
  [
    "test",
    "--manifest-path",
    "rust/Cargo.toml",
    "-p",
    "omena-transform-passes",
    "semantic_preservation_broken_translation_corpus_rejects_known_bad_outputs",
  ],
  {
    cwd: repoRoot,
    encoding: "utf8",
    maxBuffer: 1024 * 1024 * 16,
  },
);

assert.equal(
  result.status,
  0,
  `semantic preservation corpus check failed\nstdout=${result.stdout}\nstderr=${result.stderr}`,
);

process.stdout.write(
  `${JSON.stringify(
    {
      product: "omena-transform-passes.translation-validation-kill-rate.check",
      corpusPath: path.relative(repoRoot, corpusPath),
      fixtureCount: corpus.length,
      expectedRejectedCount,
      supportedPassIds: [...supportedPassIds],
      rustGatePassed: true,
    },
    null,
    2,
  )}\n`,
);
