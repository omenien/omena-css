import { strict as assert } from "node:assert";
import { spawnSync } from "node:child_process";
import { readFileSync } from "node:fs";
import path from "node:path";

interface BrokenTranslationFixture {
  readonly passId: string;
  readonly reachableClassNames?: readonly string[];
  readonly reachableKeyframeNames?: readonly string[];
  readonly reachableValueNames?: readonly string[];
  readonly reachableCustomPropertyNames?: readonly string[];
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
    supportedPassIds: new Set([
      "tree-shake-class",
      "tree-shake-keyframes",
      "tree-shake-value",
      "tree-shake-custom-property",
    ]),
    rustTest: "semantic_preservation_broken_shake_corpus_rejects_known_bad_outputs",
    requiresClosedWorldReachability: true,
  },
  {
    stage: "flatten-structural",
    path: "rust/crates/omena-transform-passes/fixtures/semantic-preservation/broken-flatten.json",
    supportedPassIds: new Set(["nesting-unwrap", "scope-flatten", "layer-flatten"]),
    rustTest: "semantic_preservation_broken_flatten_corpus_rejects_known_bad_outputs",
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
    if ("requiresClosedWorldReachability" in record && record.requiresClosedWorldReachability) {
      assert.ok(
        fixture.reachableClassNames?.length ||
          fixture.reachableKeyframeNames?.length ||
          fixture.reachableValueNames?.length ||
          fixture.reachableCustomPropertyNames?.length,
        `${record.stage} fixture ${index} must declare closed-world reachability roots`,
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
  assertCargoTestExecuted(result, `${record.stage} corpus check`);

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

function assertCargoTestExecuted(result: ReturnType<typeof spawnSync>, label: string): void {
  const output = `${result.stdout}\n${result.stderr}`;
  const passedCounts = [...output.matchAll(/test result: ok\. (\d+) passed;/gu)].map((match) =>
    Number(match[1]),
  );
  assert.ok(
    passedCounts.some((count) => count > 0),
    `${label} matched zero Rust tests\nstdout=${result.stdout}\nstderr=${result.stderr}`,
  );
}
