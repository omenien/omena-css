import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import path from "node:path";
import { fileURLToPath } from "node:url";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const testNames = [
  "module_reachability_producers_are_hoisted_for_two_module_bundle",
  "module_reachability_producers_are_hoisted_for_three_module_bundle",
] as const;
const result = spawnSync(
  "cargo",
  [
    "test",
    "--manifest-path",
    "rust/Cargo.toml",
    "-p",
    "omena-query",
    "--features",
    "test-support",
    "module_reachability_producers_are_hoisted_",
    "--",
    "--nocapture",
  ],
  {
    cwd: repoRoot,
    encoding: "utf8",
    maxBuffer: 16 * 1024 * 1024,
  },
);
const output = `${result.stdout ?? ""}\n${result.stderr ?? ""}`;
assert.equal(result.status, 0, `module reachability hoist tests failed\n${output}`);

for (const testName of testNames) {
  assert.match(
    output,
    new RegExp(`test [^\\n]*${testName} \\.\\.\\. ok`, "u"),
    `${testName} must execute through the registered gate`,
  );
}
const passedCount = [...output.matchAll(/test result: ok\. ([0-9]+) passed/gu)].reduce(
  (largest, match) => Math.max(largest, Number(match[1])),
  0,
);
assert.equal(
  passedCount,
  testNames.length,
  "the hoist gate must execute exactly the two product-path fixtures",
);
const observations = [
  ...output.matchAll(
    /OMENA_QUERY_MODULE_REACHABILITY_HOIST path=(\S+) projectionSummaryEvaluationCount=([0-9]+) closedWorldBundleConstructionCount=([0-9]+)/gu,
  ),
].map((match) => ({
  path: match[1],
  projectionSummaryEvaluationCount: Number(match[2]),
  closedWorldBundleConstructionCount: Number(match[3]),
}));
assert.equal(
  observations.length,
  testNames.length,
  "every product-path fixture must publish observed producer counts",
);
for (const observation of observations) {
  assert.equal(
    observation.projectionSummaryEvaluationCount,
    1,
    `${observation.path} recomputed the selector projection summary`,
  );
  assert.equal(
    observation.closedWorldBundleConstructionCount,
    1,
    `${observation.path} rebuilt the closed-world bundle or qualified index`,
  );
}

process.stdout.write(
  `${JSON.stringify(
    {
      schemaVersion: "0",
      product: "rust.omena-query.module-reachability-hoist",
      projectionSummaryEvaluationCounts: observations.map(
        (observation) => observation.projectionSummaryEvaluationCount,
      ),
      closedWorldBundleConstructionCounts: observations.map(
        (observation) => observation.closedWorldBundleConstructionCount,
      ),
      fixtureCount: testNames.length,
      tests: testNames,
      observations,
    },
    null,
    2,
  )}\n`,
);
