import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";

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
  "delta_fold_rebuild_rejects_a_non_fold_verification_target",
  "every_parity_dimension_has_a_flush_pipeline_falsifier",
  "four_projection_arena_settles_without_external_effects",
  "interface_projection_distinguishes_fanout_from_body_only_edits",
  "observer_enabled_and_disabled_paths_emit_identical_lsp_values",
  "proposed_authority_reduction_accepts_clean_observation",
  "proposed_authority_reduction_rejects_snapshot_read_side_effect",
  "proposed_authority_reduction_rejects_stale_live_demand",
  "proposed_authority_reduction_rejects_torn_corpus_revision",
  "seeded_event_stream_matches_all_parity_dimensions",
  "stamp_state_round_trip_preserves_all_fields",
  "target_projection_rejects_an_unplanned_reported_uri",
  "taxonomy_allows_only_reviewed_transient_timing_differences",
  "wiring_check_routes_each_report_check_to_its_summary_field",
] as const;

const reactiveShadowTestSource = readFileSync(
  "rust/crates/omena-lsp-server/src/reactive_shadow_tests.rs",
  "utf8",
);
const pipelineFalsifierTable = reactiveShadowTestSource.match(
  /const PARITY_PIPELINE_FALSIFIERS:[\s\S]*?= \[(?<body>[\s\S]*?)\n\];/u,
);
assert.ok(
  pipelineFalsifierTable?.groups?.body,
  "reactive shadow parity gate could not locate the pipeline-falsifier table",
);
const mappedPipelineFalsifiers = [
  ...pipelineFalsifierTable.groups.body.matchAll(/"([^"]+)"/gu),
].map((match) => match[1]);
assert.equal(
  new Set(mappedPipelineFalsifiers).size,
  mappedPipelineFalsifiers.length,
  "reactive shadow parity dimensions must map to distinct test names",
);
for (const testName of mappedPipelineFalsifiers) {
  assert.ok(
    (requiredTests as readonly string[]).includes(testName),
    `pipeline-falsifier table names a test outside the required set: ${testName}`,
  );
}

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
for (const testName of mappedPipelineFalsifiers) {
  assert.match(
    listedTests,
    new RegExp(`^reactive_shadow_tests::${testName}: test$`, "mu"),
    `pipeline-falsifier table names an undiscoverable test: ${testName}`,
  );
}
const discoveredTests = [...listedTests.matchAll(/^reactive_shadow_tests::([^:]+): test$/gmu)].map(
  (match) => match[1],
);
assert.deepEqual(
  discoveredTests.toSorted(),
  [...requiredTests].toSorted(),
  "reactive shadow parity gate required-test set drifted from the discovered contract suite",
);

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
