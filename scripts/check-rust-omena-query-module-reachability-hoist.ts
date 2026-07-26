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

process.stdout.write(
  `${JSON.stringify(
    {
      schemaVersion: "0",
      product: "rust.omena-query.module-reachability-hoist",
      projectionSummaryEvaluationCountPerRun: 1,
      closedWorldBundleConstructionCountPerRun: 1,
      fixtureCount: testNames.length,
      tests: testNames,
    },
    null,
    2,
  )}\n`,
);
