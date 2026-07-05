use omena_abstract_value::AbstractClassValueV0;
use omena_cross_file_summary::{UnifiedHypergraphEdgeKindV0, UnifiedHypergraphHyperedgeV0};
use omena_streaming_ifds::{
    ExactStreamingConnectivityOracleV0, StreamingIFDSAnalysisReportV0,
    omena_streaming_ifds_batch_fact_keys_v0, run_streaming_ifds_exact_v0,
    streaming_ifds_event_input_v0,
};
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaDiffDeletionStaleReuseFixtureReportV0 {
    pub fixture_id: &'static str,
    pub fixture_kind: &'static str,
    pub initial_reachable_node_ids: Vec<String>,
    pub warm_reachable_node_ids: Vec<String>,
    pub incremental_output_node_ids: Vec<String>,
    pub batch_fact_keys: Vec<String>,
    pub precision_parity_with_batch: bool,
    pub fallback_to_batch: bool,
    pub dropped_node_absent_from_witness: bool,
    pub stale_node_retained_in_output_facts: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaDiffDeletionStaleReuseCorpusReportV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub readiness_artifact: &'static str,
    pub fixture_count: usize,
    pub deletion_divergence_fixture_count: usize,
    pub reachability_changing_cycle_deletion_fixture_count: usize,
    pub all_deletion_divergence_fixtures_keep_stale_output_facts: bool,
    pub all_cycle_deletion_fixtures_change_reachability: bool,
    pub ready_for_relocation_consumer: bool,
    pub fixtures: Vec<OmenaDiffDeletionStaleReuseFixtureReportV0>,
}

pub fn summarize_deletion_stale_reuse_corpus_v0() -> OmenaDiffDeletionStaleReuseCorpusReportV0 {
    let fixtures = vec![
        stale_incremental_reuse_fixture_report(),
        reachability_changing_cycle_deletion_fixture_report(),
    ];
    let deletion_divergence_fixture_count = fixtures
        .iter()
        .filter(|fixture| fixture.fixture_kind == "stale-reuse")
        .count();
    let reachability_changing_cycle_deletion_fixture_count = fixtures
        .iter()
        .filter(|fixture| fixture.fixture_kind == "cycle-deletion")
        .count();
    let all_deletion_divergence_fixtures_keep_stale_output_facts = fixtures
        .iter()
        .filter(|fixture| fixture.fixture_kind == "stale-reuse")
        .all(|fixture| {
            !fixture.precision_parity_with_batch
                && fixture.fallback_to_batch
                && fixture.stale_node_retained_in_output_facts
        });
    let all_cycle_deletion_fixtures_change_reachability = fixtures
        .iter()
        .filter(|fixture| fixture.fixture_kind == "cycle-deletion")
        .all(|fixture| fixture.dropped_node_absent_from_witness);

    OmenaDiffDeletionStaleReuseCorpusReportV0 {
        schema_version: "0",
        product: "omena-diff-test.deletion-stale-reuse-corpus",
        readiness_artifact: "DELETION-STALE-REUSE-CORPUS-READY",
        fixture_count: fixtures.len(),
        deletion_divergence_fixture_count,
        reachability_changing_cycle_deletion_fixture_count,
        all_deletion_divergence_fixtures_keep_stale_output_facts,
        all_cycle_deletion_fixtures_change_reachability,
        ready_for_relocation_consumer: deletion_divergence_fixture_count >= 1
            && reachability_changing_cycle_deletion_fixture_count >= 1
            && all_deletion_divergence_fixtures_keep_stale_output_facts
            && all_cycle_deletion_fixtures_change_reachability,
        fixtures,
    }
}

fn stale_incremental_reuse_fixture_report() -> OmenaDiffDeletionStaleReuseFixtureReportV0 {
    let old_graph = vec![
        hyperedge("edge-a-b", "a", "b"),
        hyperedge("edge-b-c", "b", "c"),
    ];
    let value = exact_value("button");
    let seed = vec![streaming_ifds_event_input_v0(
        "event-a",
        1,
        "a",
        value.clone(),
        None,
    )];
    let initial = run_streaming_ifds_exact_v0(
        "stale-reuse-initial",
        "a",
        &old_graph,
        &seed,
        &ExactStreamingConnectivityOracleV0::default(),
        None,
    );
    let current_graph = vec![hyperedge("edge-a-b", "a", "b")];
    let warm_event = vec![streaming_ifds_event_input_v0(
        "event-a-next",
        2,
        "a",
        value,
        None,
    )];
    let warm = run_streaming_ifds_exact_v0(
        "stale-reuse-warm",
        "a",
        &current_graph,
        &warm_event,
        &ExactStreamingConnectivityOracleV0::default(),
        Some(&initial.summary_cache),
    );
    let batch_fact_keys = omena_streaming_ifds_batch_fact_keys_v0(&current_graph, &warm_event);
    let output_node_ids = report_node_ids(&warm);

    OmenaDiffDeletionStaleReuseFixtureReportV0 {
        fixture_id: "removed-tail-stale-fact",
        fixture_kind: "stale-reuse",
        initial_reachable_node_ids: initial.witness.reachable_node_ids,
        warm_reachable_node_ids: warm.witness.reachable_node_ids.clone(),
        incremental_output_node_ids: output_node_ids.clone(),
        batch_fact_keys,
        precision_parity_with_batch: warm.precision_parity_with_batch,
        fallback_to_batch: warm.fallback_to_batch,
        dropped_node_absent_from_witness: !warm
            .witness
            .reachable_node_ids
            .contains(&"c".to_string()),
        stale_node_retained_in_output_facts: output_node_ids.contains(&"c".to_string()),
    }
}

fn reachability_changing_cycle_deletion_fixture_report()
-> OmenaDiffDeletionStaleReuseFixtureReportV0 {
    let old_graph = vec![
        hyperedge("edge-a-b", "a", "b"),
        hyperedge("edge-b-c", "b", "c"),
        hyperedge("edge-c-b", "c", "b"),
    ];
    let value = exact_value("button");
    let seed = vec![streaming_ifds_event_input_v0(
        "event-a",
        1,
        "a",
        value.clone(),
        None,
    )];
    let initial = run_streaming_ifds_exact_v0(
        "cycle-deletion-initial",
        "a",
        &old_graph,
        &seed,
        &ExactStreamingConnectivityOracleV0::default(),
        None,
    );
    let current_graph = vec![
        hyperedge("edge-a-b", "a", "b"),
        hyperedge("edge-c-b", "c", "b"),
    ];
    let warm_event = vec![streaming_ifds_event_input_v0(
        "event-b-next",
        2,
        "b",
        value,
        None,
    )];
    let warm = run_streaming_ifds_exact_v0(
        "cycle-deletion-warm",
        "a",
        &current_graph,
        &warm_event,
        &ExactStreamingConnectivityOracleV0::default(),
        Some(&initial.summary_cache),
    );
    let batch_fact_keys = omena_streaming_ifds_batch_fact_keys_v0(&current_graph, &warm_event);
    let output_node_ids = report_node_ids(&warm);

    OmenaDiffDeletionStaleReuseFixtureReportV0 {
        fixture_id: "cycle-edge-removal-drops-node",
        fixture_kind: "cycle-deletion",
        initial_reachable_node_ids: initial.witness.reachable_node_ids,
        warm_reachable_node_ids: warm.witness.reachable_node_ids.clone(),
        incremental_output_node_ids: output_node_ids,
        batch_fact_keys,
        precision_parity_with_batch: warm.precision_parity_with_batch,
        fallback_to_batch: warm.fallback_to_batch,
        dropped_node_absent_from_witness: !warm
            .witness
            .reachable_node_ids
            .contains(&"c".to_string()),
        stale_node_retained_in_output_facts: false,
    }
}

fn report_node_ids(report: &StreamingIFDSAnalysisReportV0) -> Vec<String> {
    report
        .output_facts
        .iter()
        .map(|fact| fact.node_id.clone())
        .collect()
}

fn exact_value(value: &str) -> AbstractClassValueV0 {
    AbstractClassValueV0::Exact {
        value: value.to_string(),
    }
}

fn hyperedge(id: &str, from: &str, to: &str) -> UnifiedHypergraphHyperedgeV0 {
    let edge_kind = UnifiedHypergraphEdgeKindV0::ComposesLocal;
    let source_edge_kind = edge_kind.as_wire_label();
    UnifiedHypergraphHyperedgeV0 {
        schema_version: "0",
        product: "omena-diff-test.deletion-stale-reuse-fixture",
        layer_marker: "hypergraph-ifds",
        feature_gate: "hypergraph-ifds",
        hyperedge_id: id.to_string(),
        edge_kind,
        source_summary_edge_id: id.to_string(),
        source_edge_kind,
        source_status: "known",
        tail_node_ids: vec![from.to_string()],
        head_node_id: to.to_string(),
        order_significant_tail: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deletion_stale_reuse_corpus_records_stale_incremental_reuse() {
        let report = summarize_deletion_stale_reuse_corpus_v0();
        assert!(report.deletion_divergence_fixture_count >= 1);
        assert!(report.all_deletion_divergence_fixtures_keep_stale_output_facts);
        let stale = report
            .fixtures
            .iter()
            .find(|fixture| fixture.fixture_kind == "stale-reuse");
        assert!(stale.is_some(), "stale-reuse fixture must exist");
        let Some(stale) = stale else {
            return;
        };
        assert!(!stale.precision_parity_with_batch);
        assert!(stale.fallback_to_batch);
        assert!(stale.dropped_node_absent_from_witness);
        assert!(stale.stale_node_retained_in_output_facts);
        assert_ne!(
            stale.incremental_output_node_ids,
            stale.warm_reachable_node_ids
        );
    }

    #[test]
    fn deletion_stale_reuse_corpus_includes_reachability_changing_cycle_removal() {
        let report = summarize_deletion_stale_reuse_corpus_v0();
        assert!(report.reachability_changing_cycle_deletion_fixture_count >= 1);
        assert!(report.all_cycle_deletion_fixtures_change_reachability);
        let cycle = report
            .fixtures
            .iter()
            .find(|fixture| fixture.fixture_kind == "cycle-deletion");
        assert!(cycle.is_some(), "cycle deletion fixture must exist");
        let Some(cycle) = cycle else {
            return;
        };
        assert_ne!(
            cycle.initial_reachable_node_ids,
            cycle.warm_reachable_node_ids
        );
        assert!(cycle.initial_reachable_node_ids.contains(&"c".to_string()));
        assert!(cycle.dropped_node_absent_from_witness);
    }

    #[test]
    fn deletion_stale_reuse_corpus_emits_readiness_artifact() {
        let report = summarize_deletion_stale_reuse_corpus_v0();
        assert_eq!(
            report.readiness_artifact,
            "DELETION-STALE-REUSE-CORPUS-READY"
        );
        assert!(report.ready_for_relocation_consumer);
    }
}
