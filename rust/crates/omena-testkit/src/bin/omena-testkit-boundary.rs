use omena_testkit::summarize_omena_testkit_boundary;
use std::{io, process::ExitCode};

fn main() -> ExitCode {
    let summary = summarize_omena_testkit_boundary();
    if let Err(error) = serde_json::to_writer_pretty(io::stdout(), &summary) {
        eprintln!("failed to write omena-testkit boundary summary: {error}");
        return ExitCode::FAILURE;
    }
    if summary.all_fixture_seeds_parse
        && summary.snapshot_governance_report.global_disable_rejected
        && summary.snapshot_governance_report.unreferenced_reject_ready
        && summary
            .snapshot_governance_report
            .hot_snapshot_age_audit_ready
        && summary
            .snapshot_governance_report
            .known_failure_policy_ready
    {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}
