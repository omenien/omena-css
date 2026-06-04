use std::{fs, path::PathBuf, process::ExitCode};

use clap::Parser;
use serde::Serialize;
use serde_json::Value;

const DEFAULT_TRACE_SOURCE: &str = ".trace { color: #ffffff; width: 1.0px; }";
const DEFAULT_TRACE_PASSES: [&str; 3] = ["color-compression", "number-compression", "print-css"];

#[derive(Debug, Parser)]
#[command(
    name = "omena-trace",
    about = "Inspect Omena CSS unified trace V0 output"
)]
struct TraceCli {
    /// CSS-family source file to trace. Uses a small built-in fixture when omitted.
    #[arg(long)]
    style: Option<PathBuf>,
    /// Transform pass id to include in the trace. Repeat to trace a pass set.
    #[arg(long = "pass")]
    passes: Vec<String>,
    /// Selector name used by the variational designer-intent trace.
    #[arg(long, default_value = ".trace")]
    selector: String,
    /// Print machine-readable JSON.
    #[arg(long)]
    json: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct OmenaCliTraceV0 {
    schema_version: &'static str,
    product: &'static str,
    trace_version: &'static str,
    style: Option<String>,
    input_source: &'static str,
    requested_pass_ids: Vec<String>,
    unknown_pass_ids: Vec<String>,
    domain_count: usize,
    domains: Vec<OmenaCliTraceDomainV0>,
    transform_execution: Value,
    lawvere_trace: Value,
    lawvere_parallel_plan: Value,
    variational_trace: Value,
    ready_surfaces: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
struct OmenaCliTraceDomainV0 {
    domain: &'static str,
    product: String,
    attached: bool,
}

fn main() -> ExitCode {
    let cli = TraceCli::parse();
    let (style_label, source, input_source) = match read_trace_source(cli.style.as_ref()) {
        Ok(read) => read,
        Err(error) => {
            eprintln!("{error}");
            return ExitCode::FAILURE;
        }
    };
    let requested_pass_ids = requested_trace_pass_ids(cli.passes);
    let summary = summarize_omena_cli_trace_v0(
        style_label,
        source.as_str(),
        requested_pass_ids,
        cli.selector,
        input_source,
    );

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
            "omena trace: product={} domains={} passes={}",
            summary.product,
            summary.domain_count,
            summary.requested_pass_ids.join(",")
        );
    }

    ExitCode::SUCCESS
}

fn read_trace_source(
    style: Option<&PathBuf>,
) -> Result<(Option<String>, String, &'static str), String> {
    let Some(path) = style else {
        return Ok((None, DEFAULT_TRACE_SOURCE.to_string(), "builtInFixture"));
    };
    let source = fs::read_to_string(path)
        .map_err(|error| format!("failed to read trace style '{}': {error}", path.display()))?;
    Ok((Some(path.to_string_lossy().into_owned()), source, "file"))
}

fn requested_trace_pass_ids(passes: Vec<String>) -> Vec<String> {
    if passes.is_empty() {
        DEFAULT_TRACE_PASSES
            .iter()
            .map(|pass_id| (*pass_id).to_string())
            .collect()
    } else {
        passes
    }
}

fn summarize_omena_cli_trace_v0(
    style: Option<String>,
    source: &str,
    requested_pass_ids: Vec<String>,
    selector: String,
    input_source: &'static str,
) -> OmenaCliTraceV0 {
    let style_path = style.as_deref().unwrap_or("trace://built-in-fixture");
    let transform_trace =
        omena_query::execute_omena_query_transform_passes_from_source_with_lawvere_trace(
            style_path,
            source,
            requested_pass_ids.as_slice(),
        );
    let variational_input =
        omena_variational::designer_intent_posterior_input_v0(selector, 2, 1, 0);
    let variational_trace =
        omena_variational::designer_intent_belief_propagation_trace_v0(&variational_input);
    let transform_execution =
        serde_json::to_value(&transform_trace.execution).unwrap_or(Value::Null);
    let lawvere_trace = serde_json::to_value(&transform_trace.lawvere_trace).unwrap_or(Value::Null);
    let lawvere_parallel_plan =
        serde_json::to_value(&transform_trace.parallel_plan).unwrap_or(Value::Null);
    let variational_trace = serde_json::to_value(variational_trace).unwrap_or(Value::Null);
    let domains = vec![
        trace_domain("transformExecution", &transform_execution),
        trace_domain("lawvereModelTrace", &lawvere_trace),
        trace_domain("lawvereParallelPlanTrace", &lawvere_parallel_plan),
        trace_domain("variationalBeliefPropagationTrace", &variational_trace),
    ];

    OmenaCliTraceV0 {
        schema_version: "0",
        product: "omena-cli.trace-v0",
        trace_version: "TraceV0",
        style,
        input_source,
        requested_pass_ids,
        unknown_pass_ids: transform_trace.execution.unknown_pass_ids,
        domain_count: domains.len(),
        domains,
        transform_execution,
        lawvere_trace,
        lawvere_parallel_plan,
        variational_trace,
        ready_surfaces: vec![
            "traceCliHelp",
            "traceRequestShape",
            "unifiedTraceV0",
            "transformExecutionTrace",
            "lawvereModelTrace",
            "lawvereParallelPlanTrace",
            "variationalBeliefPropagationTrace",
        ],
    }
}

fn trace_domain(domain: &'static str, value: &Value) -> OmenaCliTraceDomainV0 {
    OmenaCliTraceDomainV0 {
        domain,
        product: value
            .pointer("/product")
            .and_then(Value::as_str)
            .unwrap_or("unknown")
            .to_string(),
        attached: !value.is_null(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trace_summary_unifies_existing_domain_traces() {
        let summary = summarize_omena_cli_trace_v0(
            None,
            DEFAULT_TRACE_SOURCE,
            requested_trace_pass_ids(Vec::new()),
            ".trace".to_string(),
            "builtInFixture",
        );

        assert_eq!(summary.product, "omena-cli.trace-v0");
        assert_eq!(summary.trace_version, "TraceV0");
        assert_eq!(summary.domain_count, 4);
        assert_eq!(summary.unknown_pass_ids, Vec::<String>::new());
        assert!(
            summary
                .ready_surfaces
                .contains(&"variationalBeliefPropagationTrace")
        );
        assert_eq!(
            summary
                .lawvere_trace
                .pointer("/product")
                .and_then(Value::as_str),
            Some("omena-lawvere.model-trace")
        );
        assert_eq!(
            summary
                .lawvere_parallel_plan
                .pointer("/product")
                .and_then(Value::as_str),
            Some("omena-lawvere.transform-pass-parallel-plan")
        );
        assert_eq!(
            summary
                .variational_trace
                .pointer("/product")
                .and_then(Value::as_str),
            Some("omena-variational.designer-intent-belief-propagation")
        );
        assert_eq!(
            summary
                .transform_execution
                .pointer("/product")
                .and_then(Value::as_str),
            Some("omena-query.transform-execute")
        );
    }
}
