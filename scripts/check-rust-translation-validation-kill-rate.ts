import { strict as assert } from "node:assert";
import { spawnSync } from "node:child_process";
import { readFileSync } from "node:fs";
import path from "node:path";

interface BrokenTranslationFixture {
  readonly passId: string;
  readonly reachableClassNames?: readonly string[];
  readonly input: string;
  readonly output: string;
  readonly expectedRejected: boolean;
}

const repoRoot = process.cwd();
const corpusRecords = [
  {
    stage: "simple-structural",
    path: "rust/crates/omena-transform-passes/fixtures/semantic-preservation/broken-simple.json",
    supportedPassIds: new Set(["empty-rule-removal", "rule-deduplication"]),
    rustTest: "semantic_preservation_broken_translation_corpus_rejects_known_bad_outputs",
  },
  {
    stage: "merge-structural",
    path: "rust/crates/omena-transform-passes/fixtures/semantic-preservation/broken-merge.json",
    supportedPassIds: new Set(["rule-merging", "selector-merging"]),
    rustTest: "semantic_preservation_broken_merge_corpus_rejects_known_bad_outputs",
  },
  {
    stage: "shake-structural",
    path: "rust/crates/omena-transform-passes/fixtures/semantic-preservation/broken-shake.json",
    supportedPassIds: new Set(["tree-shake-class"]),
    rustTest: "semantic_preservation_broken_shake_corpus_rejects_known_bad_outputs",
    requiresReachableClassNames: true,
  },
] as const;

const stageReports = corpusRecords.map((record) => {
  const corpusPath = path.join(repoRoot, record.path);
  const corpus = JSON.parse(
    readFileSync(corpusPath, "utf8"),
  ) as readonly BrokenTranslationFixture[];

  assert.ok(corpus.length > 0, `${record.stage} corpus must not be empty`);
  const expectedRejectedCount = corpus.filter((fixture) => fixture.expectedRejected).length;
  assert.ok(expectedRejectedCount > 0, `${record.stage} corpus must contain rejected cases`);

  for (const [index, fixture] of corpus.entries()) {
    assert.ok(
      record.supportedPassIds.has(fixture.passId),
      `unsupported pass id at ${record.stage} fixture ${index}`,
    );
    assert.ok(fixture.input.length > 0, `${record.stage} fixture ${index} input must not be empty`);
    assert.ok(
      fixture.output.length > 0,
      `${record.stage} fixture ${index} output must not be empty`,
    );
    assert.notEqual(
      fixture.input,
      fixture.output,
      `${record.stage} fixture ${index} must describe a changed translation`,
    );
    if ("requiresReachableClassNames" in record && record.requiresReachableClassNames) {
      assert.ok(
        fixture.reachableClassNames?.length,
        `${record.stage} fixture ${index} must declare reachable class names`,
      );
    }
  }

  const result = spawnSync(
    "cargo",
    ["test", "--manifest-path", "rust/Cargo.toml", "-p", "omena-transform-passes", record.rustTest],
    {
      cwd: repoRoot,
      encoding: "utf8",
      maxBuffer: 1024 * 1024 * 16,
    },
  );

  assert.equal(
    result.status,
    0,
    `${record.stage} corpus check failed\nstdout=${result.stdout}\nstderr=${result.stderr}`,
  );

  return {
    stage: record.stage,
    corpusPath: path.relative(repoRoot, corpusPath),
    fixtureCount: corpus.length,
    expectedRejectedCount,
    supportedPassIds: [...record.supportedPassIds],
  };
});

process.stdout.write(
  `${JSON.stringify(
    {
      product: "omena-transform-passes.translation-validation-kill-rate.check",
      stageReports,
      fixtureCount: stageReports.reduce((sum, report) => sum + report.fixtureCount, 0),
      expectedRejectedCount: stageReports.reduce(
        (sum, report) => sum + report.expectedRejectedCount,
        0,
      ),
      rustGatePassed: true,
    },
    null,
    2,
  )}\n`,
);
