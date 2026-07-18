import { strict as assert } from "node:assert";
import { spawnSync } from "node:child_process";

const expectedTests = [
  "style::salsa_memo::tests::source_element_computed_value_inherits_across_files_without_unrelated_reads",
  "style::salsa_memo::tests::source_element_parent_chain_and_scope_proximity_cross_files_without_unrelated_reads",
].toSorted();

const result = spawnSync(
  "cargo",
  [
    "test",
    "--manifest-path",
    "rust/Cargo.toml",
    "-p",
    "omena-query",
    "--features",
    "salsa-memo",
    "without_unrelated_reads",
    "--",
    "--nocapture",
  ],
  {
    encoding: "utf8",
    maxBuffer: 128 * 1024 * 1024,
  },
);

process.stdout.write(result.stdout);
process.stderr.write(result.stderr);

assert.equal(
  result.status,
  0,
  `demand read-set invariant tests failed\nstdout=${result.stdout}\nstderr=${result.stderr}`,
);

const selectedTests = [...result.stdout.matchAll(/^test (.+without_unrelated_reads) \.\.\. ok$/gmu)]
  .map((match) => match[1])
  .toSorted();

assert.deepEqual(
  selectedTests,
  expectedTests,
  "the per-file invariant lane must execute both demand-shaped unrelated-read firewall tests",
);

process.stdout.write(`validated demand read-set invariants: tests=${selectedTests.length}\n`);
