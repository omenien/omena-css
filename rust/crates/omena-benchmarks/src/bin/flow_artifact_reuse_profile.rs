use std::hint::black_box;
use std::time::{Duration, Instant};

use omena_abstract_value::{
    ClassBoundaryEffectV0, ClassValueFlowGraphV0, ClassValueFlowNodeV0, ClassValueFlowTransferV0,
    ExternalStringTypeFactsV0, analyze_class_value_flow_incremental,
    analyze_class_value_flow_incremental_with_artifact,
    analyze_class_value_flow_incremental_with_reuse,
};
use serde::Serialize;

const SAMPLE_BATCH_COUNT: usize = 7;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct FlowArtifactReuseProfileV0 {
    schema_version: &'static str,
    product: &'static str,
    measurement_pin: String,
    release_build: bool,
    sample_batch_count: usize,
    samples: Vec<FlowArtifactReuseSampleV0>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct FlowArtifactReuseSampleV0 {
    node_count: usize,
    iterations_per_batch: usize,
    fresh_rebuild_median_nanoseconds: u128,
    legacy_verified_reuse_median_nanoseconds: u128,
    sealed_digest_reuse_median_nanoseconds: u128,
    sealed_to_fresh_ratio_milli: u128,
    sealed_to_legacy_ratio_milli: u128,
    read_set_digest_check_count: usize,
    read_set_digest_node_count: usize,
    analysis_rebuild_count: usize,
    analysis_bytes_equal: bool,
}

fn main() -> Result<(), String> {
    let samples = [8, 32, 128]
        .into_iter()
        .map(measure_sample)
        .collect::<Result<Vec<_>, _>>()?;
    let profile = FlowArtifactReuseProfileV0 {
        schema_version: "0",
        product: "omena-benchmarks.flow-artifact-reuse-profile",
        measurement_pin: std::env::var("OMENA_MEASUREMENT_PIN")
            .unwrap_or_else(|_| "unrecorded".to_string()),
        release_build: !cfg!(debug_assertions),
        sample_batch_count: SAMPLE_BATCH_COUNT,
        samples,
    };
    println!(
        "{}",
        serde_json::to_string_pretty(&profile)
            .map_err(|error| format!("profile serialization failed: {error}"))?
    );
    Ok(())
}

fn measure_sample(node_count: usize) -> Result<FlowArtifactReuseSampleV0, String> {
    let graph = chain_graph(node_count);
    let legacy_first = analyze_class_value_flow_incremental(&graph, None, 1);
    let sealed_first = analyze_class_value_flow_incremental_with_artifact(&graph, None, 1)
        .map_err(|refusal| format!("initial artifact construction refused: {refusal:?}"))?;
    let iterations_per_batch = iterations_for_size(node_count);

    warm_up(|| {
        black_box(analyze_class_value_flow_incremental(&graph, None, 2));
    });
    warm_up(|| {
        black_box(analyze_class_value_flow_incremental_with_reuse(
            &graph,
            Some(&legacy_first.next_snapshot),
            Some(&legacy_first.analysis),
            2,
        ));
    });
    warm_up(|| {
        let _ = black_box(analyze_class_value_flow_incremental_with_artifact(
            &graph,
            Some(&sealed_first.next_artifact),
            2,
        ));
    });

    let fresh = median_batch_duration(iterations_per_batch, || {
        black_box(analyze_class_value_flow_incremental(&graph, None, 2));
    });
    let legacy = median_batch_duration(iterations_per_batch, || {
        black_box(analyze_class_value_flow_incremental_with_reuse(
            &graph,
            Some(&legacy_first.next_snapshot),
            Some(&legacy_first.analysis),
            2,
        ));
    });
    let sealed = median_batch_duration(iterations_per_batch, || {
        let _ = black_box(analyze_class_value_flow_incremental_with_artifact(
            &graph,
            Some(&sealed_first.next_artifact),
            2,
        ));
    });

    let fresh_result = analyze_class_value_flow_incremental(&graph, None, 2);
    let sealed_result = analyze_class_value_flow_incremental_with_artifact(
        &graph,
        Some(&sealed_first.next_artifact),
        2,
    )
    .map_err(|refusal| format!("sealed artifact reuse refused: {refusal:?}"))?;
    let fresh_bytes = serde_json::to_vec(&fresh_result.analysis)
        .map_err(|error| format!("fresh analysis serialization failed: {error}"))?;
    let sealed_bytes = serde_json::to_vec(&sealed_result.analysis)
        .map_err(|error| format!("sealed analysis serialization failed: {error}"))?;
    let fresh_ns = per_iteration_nanoseconds(fresh, iterations_per_batch);
    let legacy_ns = per_iteration_nanoseconds(legacy, iterations_per_batch);
    let sealed_ns = per_iteration_nanoseconds(sealed, iterations_per_batch);

    Ok(FlowArtifactReuseSampleV0 {
        node_count,
        iterations_per_batch,
        fresh_rebuild_median_nanoseconds: fresh_ns,
        legacy_verified_reuse_median_nanoseconds: legacy_ns,
        sealed_digest_reuse_median_nanoseconds: sealed_ns,
        sealed_to_fresh_ratio_milli: ratio_milli(sealed_ns, fresh_ns),
        sealed_to_legacy_ratio_milli: ratio_milli(sealed_ns, legacy_ns),
        read_set_digest_check_count: sealed_result.read_set_digest_check_count,
        read_set_digest_node_count: sealed_result.read_set_digest_node_count,
        analysis_rebuild_count: sealed_result.analysis_rebuild_count,
        analysis_bytes_equal: fresh_bytes == sealed_bytes,
    })
}

fn chain_graph(node_count: usize) -> ClassValueFlowGraphV0 {
    let mut nodes = Vec::with_capacity(node_count.max(1));
    nodes.push(ClassValueFlowNodeV0 {
        id: "node-0".to_string(),
        predecessors: Vec::new(),
        boundary_effect: ClassBoundaryEffectV0::UnknownBoundary,
        transfer: ClassValueFlowTransferV0::AssignFacts(exact_facts("seed")),
    });
    for index in 1..node_count.max(1) {
        nodes.push(ClassValueFlowNodeV0 {
            id: format!("node-{index}"),
            predecessors: vec![format!("node-{}", index - 1)],
            boundary_effect: ClassBoundaryEffectV0::UnknownBoundary,
            transfer: ClassValueFlowTransferV0::Join,
        });
    }
    ClassValueFlowGraphV0 {
        context_key: Some(format!("chain-{node_count}")),
        nodes,
    }
}

fn exact_facts(value: &str) -> ExternalStringTypeFactsV0 {
    ExternalStringTypeFactsV0 {
        kind: "exact".to_string(),
        constraint_kind: None,
        values: Some(vec![value.to_string()]),
        prefix: None,
        suffix: None,
        min_len: None,
        max_len: None,
        char_must: None,
        char_may: None,
        may_include_other_chars: None,
    }
}

fn warm_up(mut operation: impl FnMut()) {
    for _ in 0..2 {
        operation();
    }
}

fn median_batch_duration(iterations: usize, mut operation: impl FnMut()) -> Duration {
    let mut samples = Vec::with_capacity(SAMPLE_BATCH_COUNT);
    for _ in 0..SAMPLE_BATCH_COUNT {
        let started = Instant::now();
        for _ in 0..iterations {
            operation();
        }
        samples.push(started.elapsed());
    }
    samples.sort_unstable();
    samples[SAMPLE_BATCH_COUNT / 2]
}

const fn iterations_for_size(node_count: usize) -> usize {
    if node_count <= 8 {
        50
    } else if node_count <= 32 {
        10
    } else {
        2
    }
}

fn per_iteration_nanoseconds(duration: Duration, iterations: usize) -> u128 {
    duration.as_nanos() / iterations as u128
}

fn ratio_milli(numerator: u128, denominator: u128) -> u128 {
    numerator.saturating_mul(1_000) / denominator.max(1)
}
