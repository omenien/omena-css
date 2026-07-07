use omena_diff_test::summarize_omena_diff_test_boundary;
use std::{io, process::ExitCode};

fn main() -> ExitCode {
    let summary = summarize_omena_diff_test_boundary();
    if let Err(error) = serde_json::to_writer_pretty(io::stdout(), &summary) {
        eprintln!("failed to write omena-diff-test boundary summary: {error}");
        return ExitCode::FAILURE;
    }
    if summary.all_parser_legacy_fixtures_match
        && summary.all_m3_fixture_seeds_parse
        && summary.all_soundiness_metamorphic_relations_hold
        && summary.all_diagnostic_metamorphic_relations_hold
        && summary.all_parser_cst_fact_authority_values_match
        && summary.all_parser_cst_fact_authority_spans_match
        && summary.all_parser_cst_fact_authority_metamorphic_relations_hold
        && summary.all_parser_cst_context_raw_scan_fixtures_match
        && summary.all_cache_equivalence_files_identical
        && summary.all_salsa_memo_equivalence_phases_identical
        && summary.all_parallel_salsa_equivalence_phases_identical
        && summary.all_reachability_second_oracle_sets_equal
        && summary.all_reachability_streaming_matches_batch
        && summary.all_reachability_bitset_parity_equal
        && summary.all_reachability_closure_hash_bitset_parity_equal
        && summary.all_reachability_product_parity_with_batch
        && summary.all_reachability_fact_keys_three_way_equal
        && summary.all_reachability_fact_keys_four_way_equal
        && summary.all_reachability_selector_relations_equal
        && summary.all_typed_graph_summary_plane_foundation_checks_hold
        && summary.workspace_summary_plane_and_snapshot_id_green
        && summary.all_scss_eval_truthiness_cst_equivalence_fixtures_match
        && summary.all_scss_eval_public_summaries_match
        && summary
            .wpt_value_differential_report
            .all_foldable_matches_hold
    {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}
