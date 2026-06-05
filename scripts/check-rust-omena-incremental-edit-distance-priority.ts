import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { readFileSync } from "node:fs";

const incrementalSource = readFileSync("rust/crates/omena-incremental/src/lib.rs", "utf8");
const querySource = readFileSync("rust/crates/omena-query/src/style/substrate.rs", "utf8");

for (const needle of [
  "IncrementalEditDistancePriorityInputV0",
  "plan_incremental_computation_with_priority_inputs",
  "incremental-edit-distance-priority-v0",
  "fixtureWitnessDistanceMarginWeightedV0",
  "editDistanceCascadeMarginWeighted",
]) {
  assert.ok(
    incrementalSource.includes(needle) || querySource.includes(needle),
    `edit-distance priority boundary must include ${needle}`,
  );
}

for (const [crateName, testName] of [
  ["omena-incremental", "edit_distance_priority_orders_dirty_nodes_for_scheduler"],
  ["omena-query", "exposes_style_edit_distance_and_cascade_margin_bridge_witness"],
] as const) {
  const result = spawnSync(
    "cargo",
    ["test", "--manifest-path", "rust/Cargo.toml", "-p", crateName, testName, "--", "--nocapture"],
    {
      cwd: process.cwd(),
      encoding: "utf8",
      maxBuffer: 1024 * 1024 * 10,
    },
  );

  assert.equal(
    result.status,
    0,
    `edit-distance priority witness failed for ${crateName}:${testName}\nstdout=${result.stdout}\nstderr=${result.stderr}`,
  );
}

console.log("omena-incremental edit-distance priority ok: bridge metric consumed by scheduler");
