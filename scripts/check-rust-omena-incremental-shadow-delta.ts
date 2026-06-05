import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { readFileSync } from "node:fs";

const source = readFileSync("rust/crates/omena-incremental/src/lib.rs", "utf8");

for (const needle of [
  "changed_at",
  "verified_at",
  "alpha_equivalence_graph_hash",
  "incremental_matches_from_scratch_delta",
  "dbsp_zset_claim_ready: false",
  "performance_benchmark_claim_ready: false",
  "sampledFixtureWitnessNotEquivalenceProof",
]) {
  assert.ok(
    source.includes(needle),
    `omena-incremental shadow-delta boundary must include ${needle}`,
  );
}

for (const testName of [
  "incremental_shadow_delta_oracle_matches_from_scratch_delta",
  "alpha_equivalence_hash_ignores_fixture_node_renaming",
]) {
  const result = spawnSync(
    "cargo",
    [
      "test",
      "--manifest-path",
      "rust/Cargo.toml",
      "-p",
      "omena-incremental",
      testName,
      "--",
      "--nocapture",
    ],
    {
      cwd: process.cwd(),
      encoding: "utf8",
      maxBuffer: 1024 * 1024 * 10,
    },
  );

  assert.equal(
    result.status,
    0,
    `omena-incremental focused witness failed for ${testName}\nstdout=${result.stdout}\nstderr=${result.stderr}`,
  );
}

console.log(
  "omena-incremental shadow delta ok: changed_at/verified_at + alpha hash + shadow oracle",
);
