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
    {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}
