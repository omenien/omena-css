use omena_parser::StyleDialect;
use omena_transform_cst::{
    IrEditRegionV0, IrNodeIdV0, IrNodeKindV0, IrTransactionV0, lower_transform_ir_from_source,
    print_transform_ir_css, reset_transform_ir_transaction_cost_telemetry,
    transform_ir_transaction_cost_telemetry_snapshot,
};
use serde::Serialize;
use std::{hint::black_box, time::Instant};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct TransactionProfileV0 {
    schema_version: &'static str,
    product: &'static str,
    measurement_pin: String,
    release_build: bool,
    operating_system: &'static str,
    architecture: &'static str,
    samples: Vec<TransactionProfileSampleV0>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct TransactionProfileSampleV0 {
    source_bytes: usize,
    node_count: usize,
    iterations: usize,
    legacy_clone_transaction_ns_p50: u128,
    edit_log_transaction_ns_p50: u128,
    edit_log_over_legacy_ratio: f64,
    output_bytes_equal: bool,
    snapshotted_node_count: u64,
    shared_node_fraction: f64,
    whole_validation_outer_node_visit_count: u64,
    delta_validation_node_visit_count: u64,
}

fn main() -> Result<(), String> {
    let mut samples = Vec::new();
    for (rule_count, iterations) in [(96usize, 40usize), (384, 20), (1_536, 10)] {
        let source = (0..rule_count)
            .map(|index| format!(".rule-{index} {{ color: red; margin: {index}px; }}\n"))
            .collect::<String>();
        let base = lower_transform_ir_from_source(
            source.as_str(),
            StyleDialect::Css,
            format!("profile-{rule_count}.css"),
        );
        let value = red_value_node(&base, source.as_str())?;
        let mut legacy_samples = Vec::with_capacity(iterations);
        let mut edit_log_samples = Vec::with_capacity(iterations);
        let mut outputs_equal = true;

        reset_transform_ir_transaction_cost_telemetry();
        for _ in 0..iterations {
            let legacy_base = base.clone();
            let legacy_started = Instant::now();
            let mut legacy_working = black_box(legacy_base.clone());
            rewrite_one_value(&mut legacy_working, value)?;
            legacy_samples.push(legacy_started.elapsed().as_nanos());

            let mut edit_log_working = base.clone();
            let edit_log_started = Instant::now();
            rewrite_one_value(&mut edit_log_working, value)?;
            edit_log_samples.push(edit_log_started.elapsed().as_nanos());

            outputs_equal &= print_transform_ir_css(&legacy_working)
                .map_err(|error| format!("legacy print failed: {error:?}"))?
                == print_transform_ir_css(&edit_log_working)
                    .map_err(|error| format!("edit-log print failed: {error:?}"))?;
        }
        legacy_samples.sort_unstable();
        edit_log_samples.sort_unstable();
        let legacy_p50 = legacy_samples[legacy_samples.len() / 2];
        let edit_log_p50 = edit_log_samples[edit_log_samples.len() / 2];
        let telemetry = transform_ir_transaction_cost_telemetry_snapshot();
        samples.push(TransactionProfileSampleV0 {
            source_bytes: source.len(),
            node_count: base.nodes.len(),
            iterations,
            legacy_clone_transaction_ns_p50: legacy_p50,
            edit_log_transaction_ns_p50: edit_log_p50,
            edit_log_over_legacy_ratio: edit_log_p50 as f64 / legacy_p50.max(1) as f64,
            output_bytes_equal: outputs_equal,
            snapshotted_node_count: telemetry.snapshotted_node_count,
            shared_node_fraction: telemetry.shared_node_fraction(),
            whole_validation_outer_node_visit_count: telemetry
                .whole_validation_outer_node_visit_count,
            delta_validation_node_visit_count: telemetry.delta_validation_node_visit_count,
        });
    }
    let profile = TransactionProfileV0 {
        schema_version: "0",
        product: "omena-benchmarks.transform-ir-transaction-profile",
        measurement_pin: std::env::var("OMENA_MEASUREMENT_PIN")
            .unwrap_or_else(|_| "unrecorded".to_string()),
        release_build: !cfg!(debug_assertions),
        operating_system: std::env::consts::OS,
        architecture: std::env::consts::ARCH,
        samples,
    };
    println!(
        "{}",
        serde_json::to_string_pretty(&profile)
            .map_err(|error| format!("profile serialization failed: {error}"))?
    );
    Ok(())
}

fn red_value_node(
    ir: &omena_transform_cst::TransformIrV0,
    source: &str,
) -> Result<IrNodeIdV0, String> {
    ir.nodes
        .iter()
        .find(|node| {
            node.kind == IrNodeKindV0::Value
                && source
                    .get(node.source_span_start..node.source_span_end)
                    .is_some_and(|text| text.trim() == "red")
        })
        .map(|node| node.node_id)
        .ok_or_else(|| "profile fixture must contain a red value".to_string())
}

fn rewrite_one_value(
    ir: &mut omena_transform_cst::TransformIrV0,
    value: IrNodeIdV0,
) -> Result<(), String> {
    let source_byte_len = ir.source_byte_len;
    let mut transaction =
        IrTransactionV0::new(ir, "profile-rewrite", IrEditRegionV0::full(source_byte_len));
    transaction
        .rewrite_value(value, "blue")
        .map_err(|error| format!("profile rewrite failed: {error:?}"))?;
    transaction
        .commit()
        .map_err(|error| format!("profile commit failed: {error:?}"))
}
