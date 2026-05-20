use std::{path::PathBuf, process::ExitCode};

use clap::Parser;
use serde::Serialize;

#[derive(Debug, Parser)]
#[command(
    name = "omena-trace",
    about = "Inspect planned Omena CSS transform trace inputs"
)]
struct TraceCli {
    /// CSS-family source file to trace.
    #[arg(long)]
    style: Option<PathBuf>,
    /// Transform pass id to include in the trace. Repeat to trace a pass set.
    #[arg(long = "pass")]
    passes: Vec<String>,
    /// Print machine-readable JSON.
    #[arg(long)]
    json: bool,
}

#[derive(Debug, Serialize)]
struct TraceScaffoldSummary<'a> {
    product: &'a str,
    schema_version: &'a str,
    style: Option<&'a str>,
    passes: &'a [String],
    ready_surfaces: [&'a str; 2],
}

fn main() -> ExitCode {
    let cli = TraceCli::parse();
    let style = cli.style.as_ref().and_then(|path| path.to_str());
    let summary = TraceScaffoldSummary {
        product: "omena-cli.trace",
        schema_version: "0",
        style,
        passes: &cli.passes,
        ready_surfaces: ["traceCliHelp", "traceRequestShape"],
    };

    if cli.json {
        match serde_json::to_string_pretty(&summary) {
            Ok(output) => println!("{output}"),
            Err(error) => {
                eprintln!("failed to serialize trace summary: {error}");
                return ExitCode::FAILURE;
            }
        }
    } else {
        println!(
            "omena trace scaffold: style={} passes={}",
            style.unwrap_or("<none>"),
            if cli.passes.is_empty() {
                "<default>".to_string()
            } else {
                cli.passes.join(",")
            }
        );
    }

    ExitCode::SUCCESS
}
