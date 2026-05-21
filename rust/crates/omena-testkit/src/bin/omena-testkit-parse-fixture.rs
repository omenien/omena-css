use omena_testkit::parse_cme_fixture_v0;
use std::{
    io::{self, Read},
    process::ExitCode,
};

fn main() -> ExitCode {
    let mut raw = String::new();
    if let Err(error) = io::stdin().read_to_string(&mut raw) {
        eprintln!("failed to read cme-fixture-v0 from stdin: {error}");
        return ExitCode::FAILURE;
    }

    match parse_cme_fixture_v0(raw.as_str()) {
        Ok(fixture) => {
            if let Err(error) = serde_json::to_writer_pretty(io::stdout(), &fixture) {
                eprintln!("failed to write parsed cme-fixture-v0 JSON: {error}");
                return ExitCode::FAILURE;
            }
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}
