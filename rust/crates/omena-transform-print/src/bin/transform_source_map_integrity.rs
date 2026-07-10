use omena_transform_print::summarize_transform_source_map_integrity_v0;
use std::{io, process::ExitCode};

fn main() -> ExitCode {
    let report = summarize_transform_source_map_integrity_v0();
    if let Err(error) = serde_json::to_writer_pretty(io::stdout(), &report) {
        eprintln!("failed to write transform source-map integrity report: {error}");
        return ExitCode::FAILURE;
    }
    if report.complete {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}
