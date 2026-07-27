import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";

const cargoArguments = [
  "test",
  "--manifest-path",
  "rust/Cargo.toml",
  "-p",
  "omena-lsp-server",
  "reactive_shadow_tests::",
];
const requiredTests = [
  "deferred_digest_receipt_stays_attached_to_its_scheduler_flush",
  "delivery_projection_rejects_an_inverted_writer_decision",
  "every_flush_equality_rejects_its_own_projection_drift",
  "four_projection_arena_settles_without_external_effects",
  "interface_projection_distinguishes_fanout_from_body_only_edits",
  "observer_enabled_and_disabled_paths_emit_identical_lsp_values",
  "proposed_authority_reduction_accepts_clean_observation",
  "proposed_authority_reduction_rejects_snapshot_read_side_effect",
  "proposed_authority_reduction_rejects_stale_live_demand",
  "proposed_authority_reduction_rejects_torn_corpus_revision",
  "seeded_event_stream_matches_all_flush_projections",
  "snapshot_generation_projection_uses_flush_completion_stamps",
  "target_projection_rejects_an_unplanned_reported_uri",
  "taxonomy_allows_only_reviewed_transient_timing_differences",
] as const;

const listedTests = execFileSync("cargo", [...cargoArguments, "--", "--list"], {
  cwd: process.cwd(),
  encoding: "utf8",
});
for (const testName of requiredTests) {
  assert.match(
    listedTests,
    new RegExp(`^reactive_shadow_tests::${testName}: test$`, "mu"),
    `reactive shadow parity gate did not discover ${testName}`,
  );
}

execFileSync("cargo", cargoArguments, {
  cwd: process.cwd(),
  stdio: "inherit",
});

console.log(
  JSON.stringify(
    {
      schemaVersion: "omena.reactive-shadow-parity.v0",
      product: "rust.omena-lsp-server.reactive-shadow-parity",
      discoveredContractTests: requiredTests.length,
    },
    null,
    2,
  ),
);
