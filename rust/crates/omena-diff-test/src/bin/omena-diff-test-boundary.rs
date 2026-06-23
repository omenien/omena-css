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
        && summary
            .wpt_value_differential_report
            .all_foldable_matches_hold
    {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}
