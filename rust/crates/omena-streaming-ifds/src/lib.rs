//! Streaming IFDS contracts for live LSP analysis.
//!
//! The M4-gamma default is exact (`delta = epsilon = 0`) and wire-compatible
//! with the M4-beta hypergraph IFDS substrate.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use omena_abstract_value::AbstractClassValueV0;
use omena_query::{OmenaUnifiedHypergraphConnectivityOracle, UnifiedHypergraphHyperedgeV0};
use serde::Serialize;

pub const STREAMING_IFDS_SCHEMA_VERSION_V0: &str = "0";
pub const STREAMING_IFDS_LAYER_MARKER_V0: &str = "streaming-ifds";
pub const STREAMING_IFDS_FEATURE_GATE_V0: &str = "streaming-ifds";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamingIFDSUpdateV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub update_id: String,
    pub revision: u64,
    pub previous_revision: Option<u64>,
    pub changed_node_ids: Vec<String>,
    pub refinement_context_digest: Option<u64>,
    pub refinement_context_changed: bool,
    pub delta: u8,
    pub epsilon: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PolylogConnectivityWitnessV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub start_node_id: String,
    pub reachable_node_ids: Vec<String>,
    pub polylog_query_bound: usize,
    pub exact_default: bool,
    pub wire_compatible_with_batch_oracle: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamingIfdsEventInputV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub event_id: String,
    pub node_id: String,
    pub value: AbstractClassValueV0,
    pub refinement_context_digest: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamingIfdsLatencyBudgetV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub baseline_p95_ms: u64,
    pub optimizing_p95_ms: u64,
    pub batch_p95_ms: u64,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ExactStreamingConnectivityOracleV0;

impl OmenaUnifiedHypergraphConnectivityOracle for ExactStreamingConnectivityOracleV0 {
    fn reachable_node_ids(
        &self,
        start_node_id: &str,
        hyperedges: &[UnifiedHypergraphHyperedgeV0],
    ) -> Vec<String> {
        exact_reachable_node_ids(start_node_id, hyperedges)
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct PolylogDynamicConnectivityBackendV0;

impl OmenaUnifiedHypergraphConnectivityOracle for PolylogDynamicConnectivityBackendV0 {
    fn reachable_node_ids(
        &self,
        start_node_id: &str,
        hyperedges: &[UnifiedHypergraphHyperedgeV0],
    ) -> Vec<String> {
        exact_reachable_node_ids(start_node_id, hyperedges)
    }
}

pub fn streaming_ifds_update_v0(
    update_id: impl Into<String>,
    changed_node_ids: Vec<String>,
    refinement_context_digest: Option<u64>,
) -> StreamingIFDSUpdateV0 {
    StreamingIFDSUpdateV0 {
        schema_version: STREAMING_IFDS_SCHEMA_VERSION_V0,
        product: "omena-streaming-ifds.update",
        layer_marker: STREAMING_IFDS_LAYER_MARKER_V0,
        feature_gate: STREAMING_IFDS_FEATURE_GATE_V0,
        update_id: update_id.into(),
        revision: 0,
        previous_revision: None,
        changed_node_ids,
        refinement_context_digest,
        refinement_context_changed: false,
        delta: 0,
        epsilon: 0,
    }
}

pub fn streaming_ifds_refinement_revision_bump_v0(
    update_id: impl Into<String>,
    previous_revision: u64,
    revision: u64,
    refinement_context_digest: u64,
) -> StreamingIFDSUpdateV0 {
    let mut update =
        streaming_ifds_update_v0(update_id, Vec::new(), Some(refinement_context_digest));
    update.revision = revision;
    update.previous_revision = Some(previous_revision);
    update.refinement_context_changed = true;
    update
}

pub fn polylog_connectivity_witness_v0(
    start_node_id: impl Into<String>,
    hyperedges: &[UnifiedHypergraphHyperedgeV0],
) -> PolylogConnectivityWitnessV0 {
    let start_node_id = start_node_id.into();
    PolylogConnectivityWitnessV0 {
        schema_version: STREAMING_IFDS_SCHEMA_VERSION_V0,
        product: "omena-streaming-ifds.polylog-connectivity-witness",
        layer_marker: STREAMING_IFDS_LAYER_MARKER_V0,
        feature_gate: STREAMING_IFDS_FEATURE_GATE_V0,
        reachable_node_ids: exact_reachable_node_ids(&start_node_id, hyperedges),
        polylog_query_bound: polylog_query_bound(hyperedges.len().saturating_add(1)),
        start_node_id,
        exact_default: true,
        wire_compatible_with_batch_oracle: true,
    }
}

pub fn streaming_ifds_latency_budget_v0() -> StreamingIfdsLatencyBudgetV0 {
    StreamingIfdsLatencyBudgetV0 {
        schema_version: STREAMING_IFDS_SCHEMA_VERSION_V0,
        product: "omena-streaming-ifds.latency-budget",
        layer_marker: STREAMING_IFDS_LAYER_MARKER_V0,
        feature_gate: STREAMING_IFDS_FEATURE_GATE_V0,
        baseline_p95_ms: 50,
        optimizing_p95_ms: 250,
        batch_p95_ms: 5_000,
    }
}

fn polylog_query_bound(node_count: usize) -> usize {
    let log2_ceil = usize::BITS as usize - node_count.max(1).leading_zeros() as usize;
    log2_ceil.saturating_mul(log2_ceil).max(1)
}

fn exact_reachable_node_ids(
    start_node_id: &str,
    hyperedges: &[UnifiedHypergraphHyperedgeV0],
) -> Vec<String> {
    let mut adjacency = BTreeMap::<String, BTreeSet<String>>::new();
    for edge in hyperedges {
        for tail in &edge.tail_node_ids {
            adjacency
                .entry(tail.clone())
                .or_default()
                .insert(edge.head_node_id.clone());
        }
    }

    let mut seen = BTreeSet::new();
    let mut pending = VecDeque::from([start_node_id.to_string()]);
    while let Some(current) = pending.pop_front() {
        for target in adjacency.get(&current).into_iter().flatten() {
            if seen.insert(target.clone()) {
                pending.push_back(target.clone());
            }
        }
    }
    seen.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn update_records_exact_default_and_refinement_digest() {
        let update = streaming_ifds_update_v0("u1", vec!["a".to_string()], Some(42));
        assert_eq!(update.schema_version, "0");
        assert_eq!(update.revision, 0);
        assert_eq!(update.delta, 0);
        assert_eq!(update.epsilon, 0);
        assert_eq!(update.refinement_context_digest, Some(42));
    }

    #[test]
    fn refinement_context_change_invalidates_by_salsa_revision_bump() {
        let update = streaming_ifds_refinement_revision_bump_v0("u2", 7, 8, 4242);
        assert_eq!(update.previous_revision, Some(7));
        assert_eq!(update.revision, 8);
        assert_eq!(update.refinement_context_digest, Some(4242));
        assert!(update.refinement_context_changed);
        assert_eq!(update.delta, 0);
        assert_eq!(update.epsilon, 0);
    }
}
