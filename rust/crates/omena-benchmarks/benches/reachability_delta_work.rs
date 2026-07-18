use std::hint::black_box;

use iai_callgrind::{library_benchmark, library_benchmark_group, main};
use omena_abstract_value::top_class_value;
use omena_cross_file_summary::{UnifiedHypergraphEdgeKindV0, UnifiedHypergraphHyperedgeV0};
use omena_streaming_ifds::{
    ExactStreamingConnectivityOracleV0, STREAMING_IFDS_FEATURE_GATE_V0,
    STREAMING_IFDS_LAYER_MARKER_V0, STREAMING_IFDS_SCHEMA_VERSION_V0, StreamingIFDSEventKindV0,
    StreamingIfdsEventInputV0, run_streaming_ifds_exact_v0, streaming_ifds_event_input_v0,
};

#[library_benchmark]
fn reachability_delta_visits_less_work_than_batch() -> usize {
    let old_graph = vec![
        hyperedge("edge-entry-theme", "entry", "theme"),
        hyperedge("edge-theme-token", "theme", "token"),
        hyperedge("edge-token-button", "token", "button"),
        hyperedge("edge-button-icon", "button", "icon"),
    ];
    let seed = vec![streaming_ifds_event_input_v0(
        "event-entry",
        1,
        "entry",
        top_class_value(),
        None,
    )];
    let first = run_streaming_ifds_exact_v0(
        "initial-analysis",
        "entry",
        &old_graph,
        &seed,
        &ExactStreamingConnectivityOracleV0::default(),
        None,
    );
    let current_graph = vec![
        hyperedge("edge-entry-theme", "entry", "theme"),
        hyperedge("edge-theme-token", "theme", "token"),
    ];
    let events = vec![edge_delete_event(
        "event-token-button-delete",
        2,
        "token",
        "button",
    )];
    let report = run_streaming_ifds_exact_v0(
        "incremental-analysis",
        "entry",
        &current_graph,
        &events,
        &ExactStreamingConnectivityOracleV0::default(),
        Some(&first.summary_cache),
    );

    assert!(report.reachability_parity_with_batch);
    assert!(report.reachability_delta_used);
    assert!(
        report.reachability_work_node_visits < report.batch_reachability_work_node_visits,
        "incremental reachability work should stay below batch work: {report:#?}"
    );
    black_box(report.reachability_work_node_visits)
}

fn hyperedge(id: &str, from: &str, to: &str) -> UnifiedHypergraphHyperedgeV0 {
    UnifiedHypergraphHyperedgeV0 {
        schema_version: "0",
        product: "test.hyperedge",
        layer_marker: "hypergraph-ifds",
        feature_gate: "hypergraph-ifds",
        hyperedge_id: id.to_string(),
        edge_kind: UnifiedHypergraphEdgeKindV0::SassForward,
        source_summary_edge_id: id.to_string(),
        source_edge_kind: "sassForward",
        source_status: "known",
        tail_node_ids: vec![from.to_string()],
        head_node_id: to.to_string(),
        order_significant_tail: false,
    }
}

fn edge_delete_event(id: &str, revision: u64, from: &str, to: &str) -> StreamingIfdsEventInputV0 {
    StreamingIfdsEventInputV0 {
        schema_version: STREAMING_IFDS_SCHEMA_VERSION_V0,
        product: "test.event-input",
        layer_marker: STREAMING_IFDS_LAYER_MARKER_V0,
        feature_gate: STREAMING_IFDS_FEATURE_GATE_V0,
        event_id: id.to_string(),
        revision,
        event_kind: StreamingIFDSEventKindV0::EdgeDelete {
            from: from.to_string(),
            to: to.to_string(),
            edge_kind: "sassForward",
        },
        node_id: from.to_string(),
        value: top_class_value(),
        refinement_context_digest: None,
    }
}

library_benchmark_group!(
    name = reachability_delta_work;
    benchmarks = reachability_delta_visits_less_work_than_batch
);

main!(library_benchmark_groups = reachability_delta_work);
