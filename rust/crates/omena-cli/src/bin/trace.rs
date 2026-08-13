use std::{fs, path::PathBuf, process::ExitCode};

use clap::Parser;
use serde::{Serialize, Serializer};
use serde_json::Value;

const DEFAULT_TRACE_SOURCE: &str = ".trace { color: #ffffff; width: 1.0px; }";
const DEFAULT_TRACE_PASSES: [&str; 3] = ["color-compression", "number-compression", "print-css"];

#[deprecated(
    since = "0.4.0",
    note = "compatibility owner: omena-cli maintainers; removal condition: not before 1.0, and only after downstream migration and zero in-repo non-compat uses"
)]
const LEGACY_VARIATIONAL_TRACE_SURFACE_V0: &str = "variationalBeliefPropagationTrace";

#[cfg(test)]
#[deprecated(
    since = "0.4.0",
    note = "compatibility owner: omena-cli maintainers; removal condition: not before 1.0, and only after downstream migration and zero in-repo non-compat uses"
)]
const LEGACY_VARIATIONAL_TRACE_WIRE_BYTES_V0: &[u8] = br#"["variationalBeliefPropagationTrace","omena-variational.designer-intent-belief-propagation","variationalBeliefPropagationTrace"]"#;

#[deprecated(
    since = "0.4.0",
    note = "compatibility owner: omena-cli maintainers; removal condition: not before 1.0, and only after downstream migration and zero in-repo non-compat uses"
)]
const LEGACY_TRANSFORM_CATALOG_MODEL_SURFACE_V0: &str = "lawvereModelTrace";

#[deprecated(
    since = "0.4.0",
    note = "compatibility owner: omena-cli maintainers; removal condition: not before 1.0, and only after downstream migration and zero in-repo non-compat uses"
)]
const LEGACY_TRANSFORM_CATALOG_PARALLEL_PLAN_SURFACE_V0: &str = "lawvereParallelPlanTrace";

#[cfg(test)]
#[deprecated(
    since = "0.4.0",
    note = "compatibility owner: omena-cli maintainers; removal condition: not before 1.0, and only after downstream migration and zero in-repo non-compat uses"
)]
const LEGACY_TRANSFORM_CATALOG_WIRE_PRODUCT_BYTES_V0: &[u8] = br#"["lawvereTrace","omena-lawvere.model-trace","lawvereParallelPlan","omena-lawvere.transform-pass-parallel-plan"]"#;

#[cfg(test)]
#[deprecated(
    since = "0.4.0",
    note = "compatibility owner: omena-cli maintainers; removal condition: not before 1.0, and only after downstream migration and zero in-repo non-compat uses"
)]
const LEGACY_TRANSFORM_CATALOG_MODEL_KEY_V0: &str = "lawvereTrace";

#[cfg(test)]
#[deprecated(
    since = "0.4.0",
    note = "compatibility owner: omena-cli maintainers; removal condition: not before 1.0, and only after downstream migration and zero in-repo non-compat uses"
)]
const LEGACY_TRANSFORM_CATALOG_PARALLEL_PLAN_KEY_V0: &str = "lawvereParallelPlan";

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

#[derive(Debug, Clone, PartialEq)]
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
    transform_catalog_trace: Value,
    transform_catalog_parallel_plan: Value,
    variational_trace: Value,
    ready_surfaces: Vec<&'static str>,
}

impl Serialize for OmenaCliTraceV0 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        #[allow(deprecated)]
        serialize_omena_cli_trace_legacy_wire_v0(self, serializer)
    }
}

#[deprecated(
    since = "0.4.0",
    note = "compatibility serializer owned by omena-cli maintainers; removal is not before 1.0 and requires downstream migration plus zero audited non-compatibility uses"
)]
fn serialize_omena_cli_trace_legacy_wire_v0<S>(
    trace: &OmenaCliTraceV0,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct LegacyOmenaCliTraceWireV0<'a> {
        schema_version: &'a str,
        product: &'a str,
        trace_version: &'a str,
        style: &'a Option<String>,
        input_source: &'a str,
        requested_pass_ids: &'a [String],
        unknown_pass_ids: &'a [String],
        domain_count: usize,
        domains: &'a [OmenaCliTraceDomainV0],
        transform_execution: &'a Value,
        lawvere_trace: &'a Value,
        lawvere_parallel_plan: &'a Value,
        variational_trace: &'a Value,
        ready_surfaces: &'a [&'static str],
    }

    LegacyOmenaCliTraceWireV0 {
        schema_version: trace.schema_version,
        product: trace.product,
        trace_version: trace.trace_version,
        style: &trace.style,
        input_source: trace.input_source,
        requested_pass_ids: trace.requested_pass_ids.as_slice(),
        unknown_pass_ids: trace.unknown_pass_ids.as_slice(),
        domain_count: trace.domain_count,
        domains: trace.domains.as_slice(),
        transform_execution: &trace.transform_execution,
        lawvere_trace: &trace.transform_catalog_trace,
        lawvere_parallel_plan: &trace.transform_catalog_parallel_plan,
        variational_trace: &trace.variational_trace,
        ready_surfaces: trace.ready_surfaces.as_slice(),
    }
    .serialize(serializer)
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

#[allow(deprecated)]
fn summarize_omena_cli_trace_v0(
    style: Option<String>,
    source: &str,
    requested_pass_ids: Vec<String>,
    selector: String,
    input_source: &'static str,
) -> OmenaCliTraceV0 {
    let style_path = style.as_deref().unwrap_or("trace://built-in-fixture");
    #[allow(deprecated)]
    let transform_catalog_summary = legacy_omena_cli_transform_catalog_wire_summary_v0(
        style_path,
        source,
        requested_pass_ids.as_slice(),
    );
    let variational_input =
        omena_variational::designer_intent_posterior_input_v0(selector, 2, 1, 0);
    #[allow(deprecated)]
    let variational_trace =
        omena_variational::designer_intent_legacy_wire_compatibility_trace_v0(&variational_input);
    let transform_execution =
        serde_json::to_value(&transform_catalog_summary.execution).unwrap_or(Value::Null);
    #[allow(deprecated)]
    let transform_catalog_trace = serde_json::to_value(
        legacy_omena_cli_transform_catalog_model_trace_v0(&transform_catalog_summary),
    )
    .unwrap_or(Value::Null);
    let transform_catalog_parallel_plan =
        serde_json::to_value(&transform_catalog_summary.parallel_plan).unwrap_or(Value::Null);
    let variational_trace = serde_json::to_value(variational_trace).unwrap_or(Value::Null);
    #[allow(deprecated)]
    let domains = vec![
        trace_domain("transformExecution", &transform_execution),
        trace_domain(
            LEGACY_TRANSFORM_CATALOG_MODEL_SURFACE_V0,
            &transform_catalog_trace,
        ),
        trace_domain(
            LEGACY_TRANSFORM_CATALOG_PARALLEL_PLAN_SURFACE_V0,
            &transform_catalog_parallel_plan,
        ),
        trace_domain(LEGACY_VARIATIONAL_TRACE_SURFACE_V0, &variational_trace),
    ];
    #[allow(deprecated)]
    let ready_surfaces = vec![
        "traceCliHelp",
        "traceRequestShape",
        "unifiedTraceV0",
        "transformExecutionTrace",
        LEGACY_TRANSFORM_CATALOG_MODEL_SURFACE_V0,
        LEGACY_TRANSFORM_CATALOG_PARALLEL_PLAN_SURFACE_V0,
        LEGACY_VARIATIONAL_TRACE_SURFACE_V0,
    ];

    OmenaCliTraceV0 {
        schema_version: "0",
        product: "omena-cli.trace-v0",
        trace_version: "TraceV0",
        style,
        input_source,
        requested_pass_ids,
        unknown_pass_ids: transform_catalog_summary.execution.unknown_pass_ids,
        domain_count: domains.len(),
        domains,
        transform_execution,
        transform_catalog_trace,
        transform_catalog_parallel_plan,
        variational_trace,
        ready_surfaces,
    }
}

#[allow(deprecated)]
#[deprecated(
    since = "0.4.0",
    note = "compatibility adapter owned by omena-cli maintainers; removal is not before 1.0 and requires downstream migration plus zero audited non-compatibility uses"
)]
fn legacy_omena_cli_transform_catalog_wire_summary_v0(
    style_path: &str,
    source: &str,
    requested_pass_ids: &[String],
) -> omena_query::OmenaQueryLawvereTransformExecuteSummaryV0 {
    #[allow(deprecated)]
    omena_query::execute_omena_query_transform_passes_from_source_with_lawvere_trace(
        style_path,
        source,
        requested_pass_ids,
    )
}

#[allow(deprecated)]
#[deprecated(
    since = "0.4.0",
    note = "compatibility field adapter owned by omena-cli maintainers; removal is not before 1.0 and requires downstream migration plus zero audited non-compatibility uses"
)]
fn legacy_omena_cli_transform_catalog_model_trace_v0(
    summary: &omena_query::OmenaQueryLawvereTransformExecuteSummaryV0,
) -> &omena_query::OmenaQueryLawvereModelTraceV0 {
    &summary.lawvere_trace
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
    #[allow(deprecated)]
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
        let serialized = serde_json::to_value(&summary);
        assert!(serialized.is_ok(), "trace summary should serialize");
        let serialized = serialized.unwrap_or(Value::Null);
        let model_product_pointer = format!("/{LEGACY_TRANSFORM_CATALOG_MODEL_KEY_V0}/product");
        let parallel_plan_product_pointer =
            format!("/{LEGACY_TRANSFORM_CATALOG_PARALLEL_PLAN_KEY_V0}/product");
        let transform_catalog_wire_bytes = serde_json::to_vec(&(
            LEGACY_TRANSFORM_CATALOG_MODEL_KEY_V0,
            serialized
                .pointer(model_product_pointer.as_str())
                .and_then(Value::as_str)
                .unwrap_or_default(),
            LEGACY_TRANSFORM_CATALOG_PARALLEL_PLAN_KEY_V0,
            serialized
                .pointer(parallel_plan_product_pointer.as_str())
                .and_then(Value::as_str)
                .unwrap_or_default(),
        ));
        assert!(
            transform_catalog_wire_bytes.is_ok(),
            "legacy transform-catalog trace products should serialize"
        );
        assert_eq!(
            transform_catalog_wire_bytes.unwrap_or_default(),
            LEGACY_TRANSFORM_CATALOG_WIRE_PRODUCT_BYTES_V0
        );
        let variational_domain = summary
            .domains
            .iter()
            .find(|domain| domain.domain == LEGACY_VARIATIONAL_TRACE_SURFACE_V0);
        assert!(
            variational_domain.is_some(),
            "legacy variational trace domain should remain attached"
        );
        let variational_wire_bytes = serde_json::to_vec(&(
            variational_domain
                .map(|domain| domain.domain)
                .unwrap_or_default(),
            summary
                .variational_trace
                .pointer("/product")
                .and_then(Value::as_str)
                .unwrap_or_default(),
            summary.ready_surfaces.last().copied().unwrap_or_default(),
        ));
        assert!(
            variational_wire_bytes.is_ok(),
            "legacy variational trace labels should serialize"
        );
        assert_eq!(
            variational_wire_bytes.unwrap_or_default(),
            LEGACY_VARIATIONAL_TRACE_WIRE_BYTES_V0
        );
        assert_eq!(
            summary
                .transform_catalog_trace
                .pointer("/product")
                .and_then(Value::as_str),
            Some("omena-lawvere.model-trace")
        );
        assert_eq!(
            summary
                .transform_catalog_parallel_plan
                .pointer("/product")
                .and_then(Value::as_str),
            Some("omena-lawvere.transform-pass-parallel-plan")
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
