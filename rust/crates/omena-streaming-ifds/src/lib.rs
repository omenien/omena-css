//! Streaming IFDS contracts for live LSP analysis.
//!
//! The default live-analysis path is exact (`delta = epsilon = 0`) and
//! wire-compatible with the hypergraph IFDS substrate.
//!
//! claim_level: product-wired exact default live-analysis mechanism; the polylog
//! backend label is an implementation boundary, not an asymptotic proof claim.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use omena_abstract_value::{AbstractClassValueV0, automaton_key};
use omena_cross_file_summary::{
    OmenaUnifiedHypergraphConnectivityOracle, UnifiedHypergraphEdgeKindV0,
    UnifiedHypergraphHyperedgeV0, collect_directed_graph_sccs, collect_reachable_node_ids,
};
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
    pub connectivity_algorithm: &'static str,
    pub polylog_bound_scope: &'static str,
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
    pub revision: u64,
    pub event_kind: StreamingIFDSEventKindV0,
    pub node_id: String,
    pub value: AbstractClassValueV0,
    pub refinement_context_digest: Option<u64>,
}

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum StreamingIFDSEventKindV0 {
    EdgeInsert {
        from: String,
        to: String,
        edge_kind: &'static str,
    },
    EdgeDelete {
        from: String,
        to: String,
        edge_kind: &'static str,
    },
    NodeInsert {
        id: String,
    },
    NodeDelete {
        id: String,
    },
    DigestChange {
        id: String,
    },
    BatchSynthesised {
        event_count: usize,
        original_event_kinds: Vec<&'static str>,
    },
    RefinementContextChange {
        context_digest_before: u64,
        context_digest_after: u64,
        invalidated_supergraph_node_ids: Vec<String>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamingIFDSFactV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub fact_id: String,
    pub node_id: String,
    pub value: AbstractClassValueV0,
    pub provenance: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamingIFDSTransferFunctionV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub hyperedge_id: String,
    pub edge_kind: UnifiedHypergraphEdgeKindV0,
    pub tail_node_ids: Vec<String>,
    pub head_node_id: String,
    pub transfer_kind: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamingIFDSSummaryCacheEntryV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub start_node_id: String,
    pub reachable_node_ids: Vec<String>,
    pub fact_keys: Vec<String>,
    pub summary_hash: String,
    pub reused_from_previous: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamingIFDSAnalysisReportV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub update: StreamingIFDSUpdateV0,
    pub witness: PolylogConnectivityWitnessV0,
    pub event_count: usize,
    pub input_fact_count: usize,
    pub output_fact_count: usize,
    pub dirty_fact_count: usize,
    pub reused_fact_count: usize,
    pub transfer_function_count: usize,
    pub fallback_to_batch: bool,
    pub precision_parity_with_batch: bool,
    pub reachability_parity_with_batch: bool,
    pub reachability_delta_used: bool,
    pub reachability_dirty_node_count: usize,
    pub reachability_work_node_visits: usize,
    pub batch_reachability_work_node_visits: usize,
    pub output_facts: Vec<StreamingIFDSFactV0>,
    pub summary_cache: Vec<StreamingIFDSSummaryCacheEntryV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamingIFDSDemandReportV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub start_node_ids: Vec<String>,
    pub target_node_ids: Vec<String>,
    pub projection_node_ids: Vec<String>,
    pub fact_keys: Vec<String>,
    pub transfer_visit_count: usize,
    pub slice_scc_count: usize,
    pub strict_subset_of_forward_reachable_nodes: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamingIFDSRouteDecisionV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub request_scope: &'static str,
    pub fact_key_engine: &'static str,
    pub relocation_gate_green: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamingIFDSSettleEqualReportV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub requested_settle_count: usize,
    pub equal_settle_count: usize,
    pub divergence_count: usize,
    pub all_settles_equal: bool,
    pub demand_primary_ready: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StreamingIFDSDemandReadinessInputV0 {
    pub fact_key_gate_green: bool,
    pub deletion_corpus_green: bool,
    pub complexity_slope_green: bool,
    pub settle_report: StreamingIFDSSettleEqualReportV0,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamingIFDSDemandReadinessReportV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub fact_key_gate_green: bool,
    pub deletion_corpus_green: bool,
    pub complexity_slope_green: bool,
    pub settle_all_equal: bool,
    pub precondition_count: usize,
    pub green_precondition_count: usize,
    pub demand_primary_ready: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReachabilityDirtySetProfileV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub start_node_id: String,
    pub dirty_node_count: usize,
    pub full_node_count: usize,
    pub dirty_ratio: f64,
    pub incremental_candidate: bool,
    pub dirty_node_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReachabilityDeltaComputationV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub start_node_id: String,
    pub reachable_node_ids: Vec<String>,
    pub dirty_node_ids: Vec<String>,
    pub node_visit_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamingIFDSCrossFileReachabilityReportV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub target_style_path: String,
    pub start_node_count: usize,
    pub reachable_foreign_path_count: usize,
    pub reachable_foreign_paths: Vec<String>,
    pub analysis_report_count: usize,
    pub precision_parity_with_batch: bool,
    pub exact_default: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamingIfdsFrameRuleBridgePolicyV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub coarse_policy: &'static str,
    pub fine_policy: &'static str,
    pub activation: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamingIfdsSolverHygienePolicyV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub summary_cache_feedback_policy: &'static str,
    pub non_product_cache_feedback_scope: &'static str,
    pub cache_feedback_activation: &'static str,
    pub reference_edge_value_policy: &'static str,
    pub concrete_value_owner: &'static str,
    pub deferred_value_flow_candidates: Vec<&'static str>,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExactStreamingConnectivityOracleV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
}

impl Default for ExactStreamingConnectivityOracleV0 {
    fn default() -> Self {
        Self {
            schema_version: STREAMING_IFDS_SCHEMA_VERSION_V0,
            product: "omena-streaming-ifds.exact-connectivity-oracle",
            layer_marker: STREAMING_IFDS_LAYER_MARKER_V0,
            feature_gate: STREAMING_IFDS_FEATURE_GATE_V0,
        }
    }
}

impl OmenaUnifiedHypergraphConnectivityOracle for ExactStreamingConnectivityOracleV0 {
    fn reachable_node_ids(
        &self,
        start_node_id: &str,
        hyperedges: &[UnifiedHypergraphHyperedgeV0],
    ) -> Vec<String> {
        exact_reachable_node_ids(start_node_id, hyperedges)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PolylogDynamicConnectivityBackendV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
}

impl Default for PolylogDynamicConnectivityBackendV0 {
    fn default() -> Self {
        Self {
            schema_version: STREAMING_IFDS_SCHEMA_VERSION_V0,
            product: "omena-streaming-ifds.exact-bfs-connectivity-backend",
            layer_marker: STREAMING_IFDS_LAYER_MARKER_V0,
            feature_gate: STREAMING_IFDS_FEATURE_GATE_V0,
        }
    }
}

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

pub fn streaming_ifds_event_input_v0(
    event_id: impl Into<String>,
    revision: u64,
    node_id: impl Into<String>,
    value: AbstractClassValueV0,
    refinement_context_digest: Option<u64>,
) -> StreamingIfdsEventInputV0 {
    let node_id = node_id.into();
    StreamingIfdsEventInputV0 {
        schema_version: STREAMING_IFDS_SCHEMA_VERSION_V0,
        product: "omena-streaming-ifds.event-input",
        layer_marker: STREAMING_IFDS_LAYER_MARKER_V0,
        feature_gate: STREAMING_IFDS_FEATURE_GATE_V0,
        event_id: event_id.into(),
        revision,
        event_kind: StreamingIFDSEventKindV0::DigestChange {
            id: node_id.clone(),
        },
        node_id,
        value,
        refinement_context_digest,
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
        product: "omena-streaming-ifds.exact-connectivity-witness",
        layer_marker: STREAMING_IFDS_LAYER_MARKER_V0,
        feature_gate: STREAMING_IFDS_FEATURE_GATE_V0,
        reachable_node_ids: exact_reachable_node_ids(&start_node_id, hyperedges),
        polylog_query_bound: polylog_query_bound(hyperedges.len().saturating_add(1)),
        connectivity_algorithm: "exactBfsReachability",
        polylog_bound_scope: "targetOnlyNotAsymptoticEvidence",
        start_node_id,
        exact_default: true,
        wire_compatible_with_batch_oracle: true,
    }
}

pub fn summarize_reachability_dirty_set_profile_v0<O>(
    start_node_id: impl Into<String>,
    previous_hyperedges: &[UnifiedHypergraphHyperedgeV0],
    current_hyperedges: &[UnifiedHypergraphHyperedgeV0],
    oracle: &O,
) -> ReachabilityDirtySetProfileV0
where
    O: OmenaUnifiedHypergraphConnectivityOracle,
{
    let start_node_id = start_node_id.into();
    let previous_signatures = reachability_incidence_signatures(previous_hyperedges);
    let current_signatures = reachability_incidence_signatures(current_hyperedges);
    let all_nodes = previous_signatures
        .keys()
        .chain(current_signatures.keys())
        .cloned()
        .collect::<BTreeSet<_>>();
    let dirty_nodes = all_nodes
        .into_iter()
        .filter(|node_id| previous_signatures.get(node_id) != current_signatures.get(node_id))
        .collect::<BTreeSet<_>>();
    let full_nodes = oracle
        .reachable_node_ids(start_node_id.as_str(), current_hyperedges)
        .into_iter()
        .collect::<BTreeSet<_>>();
    let dirty_node_ids = dirty_nodes
        .intersection(&full_nodes)
        .cloned()
        .collect::<Vec<_>>();
    let full_node_count = full_nodes.len();
    let dirty_node_count = dirty_node_ids.len();
    let dirty_ratio = if full_node_count == 0 {
        0.0
    } else {
        dirty_node_count as f64 / full_node_count as f64
    };

    ReachabilityDirtySetProfileV0 {
        schema_version: STREAMING_IFDS_SCHEMA_VERSION_V0,
        product: "omena-streaming-ifds.reachability-dirty-set-profile",
        layer_marker: STREAMING_IFDS_LAYER_MARKER_V0,
        feature_gate: STREAMING_IFDS_FEATURE_GATE_V0,
        start_node_id,
        dirty_node_count,
        full_node_count,
        dirty_ratio,
        incremental_candidate: dirty_ratio < 0.95,
        dirty_node_ids,
    }
}

pub fn run_streaming_ifds_exact_v0<O>(
    update_id: impl Into<String>,
    start_node_id: impl Into<String>,
    hyperedges: &[UnifiedHypergraphHyperedgeV0],
    events: &[StreamingIfdsEventInputV0],
    oracle: &O,
    previous_cache: Option<&[StreamingIFDSSummaryCacheEntryV0]>,
) -> StreamingIFDSAnalysisReportV0
where
    O: OmenaUnifiedHypergraphConnectivityOracle,
{
    let start_node_id = start_node_id.into();
    let changed_node_ids = events
        .iter()
        .map(|event| event.node_id.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let refinement_context_digest = events
        .iter()
        .find_map(|event| event.refinement_context_digest);
    let previous_revision = events
        .iter()
        .map(|event| event.revision)
        .min()
        .and_then(|revision| revision.checked_sub(1));
    let revision = events.iter().map(|event| event.revision).max().unwrap_or(0);
    let mut update =
        streaming_ifds_update_v0(update_id, changed_node_ids, refinement_context_digest);
    update.revision = revision;
    update.previous_revision = previous_revision;
    update.refinement_context_changed = events.iter().any(|event| {
        matches!(
            event.event_kind,
            StreamingIFDSEventKindV0::RefinementContextChange { .. }
        )
    });

    let batch_reachable_node_ids = oracle.reachable_node_ids(&start_node_id, hyperedges);
    let (_, batch_reachability_work_node_visits) =
        reachable_node_ids_with_work(&start_node_id, hyperedges);
    let previous_reachable_node_ids =
        previous_reachable_node_ids_for_start(previous_cache, start_node_id.as_str());
    let reachability_delta = (!previous_reachable_node_ids.is_empty()).then(|| {
        incremental_reachable_node_ids_zset(
            start_node_id.as_str(),
            hyperedges,
            events,
            previous_reachable_node_ids.iter().cloned(),
        )
    });
    let (reachable_node_ids, reachability_parity_with_batch, reachability_delta_used) =
        if let Some(delta) = &reachability_delta {
            let parity = delta.reachable_node_ids == batch_reachable_node_ids;
            (
                if parity {
                    delta.reachable_node_ids.clone()
                } else {
                    batch_reachable_node_ids.clone()
                },
                parity,
                parity,
            )
        } else {
            (batch_reachable_node_ids.clone(), true, false)
        };
    let reachability_dirty_node_count = reachability_delta
        .as_ref()
        .map(|delta| delta.dirty_node_ids.len())
        .unwrap_or(0);
    let reachability_work_node_visits = reachability_delta
        .as_ref()
        .map(|delta| delta.node_visit_count)
        .unwrap_or(batch_reachability_work_node_visits);
    let witness = PolylogConnectivityWitnessV0 {
        schema_version: STREAMING_IFDS_SCHEMA_VERSION_V0,
        product: "omena-streaming-ifds.exact-connectivity-witness",
        layer_marker: STREAMING_IFDS_LAYER_MARKER_V0,
        feature_gate: STREAMING_IFDS_FEATURE_GATE_V0,
        start_node_id: start_node_id.clone(),
        reachable_node_ids: reachable_node_ids.clone(),
        polylog_query_bound: polylog_query_bound(hyperedges.len().saturating_add(1)),
        connectivity_algorithm: "exactBfsReachability",
        polylog_bound_scope: "targetOnlyNotAsymptoticEvidence",
        exact_default: true,
        wire_compatible_with_batch_oracle: true,
    };

    let previous_fact_keys = previous_cache
        .into_iter()
        .flatten()
        .flat_map(|entry| entry.fact_keys.iter().cloned())
        .collect::<BTreeSet<_>>();
    let transfer_table = streaming_ifds_transfer_table_v0(hyperedges);

    let (incremental_facts, output_fact_keys, fact_precision_parity_with_batch) =
        if previous_fact_keys.is_empty() {
            let facts = propagate_ifds_facts_with_table(&transfer_table, events);
            let keys = fact_keys(&facts);
            (facts, keys, true)
        } else {
            // Warm runs compare two distinct computations over the current graph:
            //   * the incremental/streaming path only re-derives the dirty
            //     sub-graph reachable from changed event nodes and reuses prior
            //     facts outside that region, and
            //   * the batch oracle recomputes every fact from scratch over all
            //     hyperedges and events.
            // A divergence means a reused prior fact survived even though the
            // current graph no longer produces it.
            let incremental_facts =
                incremental_propagate_ifds_facts(&transfer_table, events, &previous_fact_keys);
            let output_fact_keys =
                incremental_fact_keys(&transfer_table, events, &previous_fact_keys);
            let batch_fact_keys =
                fact_keys(&propagate_ifds_facts_with_table(&transfer_table, events));
            (
                incremental_facts,
                output_fact_keys.clone(),
                output_fact_keys == batch_fact_keys,
            )
        };
    let precision_parity_with_batch =
        fact_precision_parity_with_batch && reachability_parity_with_batch;

    let reused_fact_count = output_fact_keys
        .iter()
        .filter(|key| previous_fact_keys.contains(*key))
        .count();
    let dirty_fact_count = output_fact_keys.len().saturating_sub(reused_fact_count);
    let summary_cache = vec![streaming_ifds_summary_cache_entry_v0(
        start_node_id,
        reachable_node_ids,
        output_fact_keys.clone(),
        reused_fact_count > 0,
    )];

    StreamingIFDSAnalysisReportV0 {
        schema_version: STREAMING_IFDS_SCHEMA_VERSION_V0,
        product: "omena-streaming-ifds.analysis-report",
        layer_marker: STREAMING_IFDS_LAYER_MARKER_V0,
        feature_gate: STREAMING_IFDS_FEATURE_GATE_V0,
        update,
        witness,
        event_count: events.len(),
        input_fact_count: events.len(),
        output_fact_count: incremental_facts.len(),
        dirty_fact_count,
        reused_fact_count,
        transfer_function_count: transfer_table.len(),
        fallback_to_batch: !reachability_parity_with_batch,
        precision_parity_with_batch,
        reachability_parity_with_batch,
        reachability_delta_used,
        reachability_dirty_node_count,
        reachability_work_node_visits,
        batch_reachability_work_node_visits,
        output_facts: incremental_facts,
        summary_cache,
    }
}

pub fn omena_streaming_ifds_batch_fact_keys_v0(
    hyperedges: &[UnifiedHypergraphHyperedgeV0],
    events: &[StreamingIfdsEventInputV0],
) -> Vec<String> {
    let transfer_table = streaming_ifds_transfer_table_v0(hyperedges);
    fact_keys(&propagate_ifds_facts_with_table(&transfer_table, events))
}

pub fn streaming_ifds_demand_projection_node_ids_v0(
    start_node_ids: &[String],
    target_node_ids: &[String],
    hyperedges: &[UnifiedHypergraphHyperedgeV0],
) -> Vec<String> {
    streaming_ifds_demand_index_v0(hyperedges)
        .slice(start_node_ids, target_node_ids)
        .projection_node_ids
}

pub fn streaming_ifds_fact_key_route_v0(
    target_node_ids: &[String],
) -> StreamingIFDSRouteDecisionV0 {
    streaming_ifds_fact_key_route_with_gate_v0(target_node_ids, false)
}

pub fn streaming_ifds_fact_key_route_with_gate_v0(
    target_node_ids: &[String],
    relocation_gate_green: bool,
) -> StreamingIFDSRouteDecisionV0 {
    let query_shaped = !target_node_ids.is_empty();
    let demand_primary = query_shaped && relocation_gate_green;
    StreamingIFDSRouteDecisionV0 {
        schema_version: STREAMING_IFDS_SCHEMA_VERSION_V0,
        product: "omena-streaming-ifds.fact-key-route",
        layer_marker: STREAMING_IFDS_LAYER_MARKER_V0,
        feature_gate: STREAMING_IFDS_FEATURE_GATE_V0,
        request_scope: if query_shaped {
            "queryShaped"
        } else {
            "workspaceWide"
        },
        fact_key_engine: if demand_primary { "demand" } else { "batch" },
        relocation_gate_green,
    }
}

pub fn run_streaming_ifds_settle_equal_v0(
    start_node_ids: &[String],
    target_node_ids: &[String],
    hyperedges: &[UnifiedHypergraphHyperedgeV0],
    events: &[StreamingIfdsEventInputV0],
    requested_settle_count: usize,
) -> StreamingIFDSSettleEqualReportV0 {
    let batch_fact_keys = omena_streaming_ifds_batch_fact_keys_v0(hyperedges, events);
    let mut equal_settle_count = 0usize;
    let index = streaming_ifds_demand_index_v0(hyperedges);
    for _ in 0..requested_settle_count {
        let demand = run_streaming_ifds_demand_with_index_v0(
            start_node_ids,
            target_node_ids,
            &index,
            events,
        );
        let projected_batch_fact_keys =
            project_fact_keys_to_nodes(&batch_fact_keys, &demand.projection_node_ids);
        if demand.fact_keys == projected_batch_fact_keys {
            equal_settle_count = equal_settle_count.saturating_add(1);
        }
    }
    let all_settles_equal =
        requested_settle_count > 0 && equal_settle_count == requested_settle_count;
    StreamingIFDSSettleEqualReportV0 {
        schema_version: STREAMING_IFDS_SCHEMA_VERSION_V0,
        product: "omena-streaming-ifds.settle-equal-report",
        layer_marker: STREAMING_IFDS_LAYER_MARKER_V0,
        feature_gate: STREAMING_IFDS_FEATURE_GATE_V0,
        requested_settle_count,
        equal_settle_count,
        divergence_count: requested_settle_count.saturating_sub(equal_settle_count),
        all_settles_equal,
        demand_primary_ready: all_settles_equal,
    }
}

pub fn streaming_ifds_demand_readiness_v0(
    input: StreamingIFDSDemandReadinessInputV0,
) -> StreamingIFDSDemandReadinessReportV0 {
    let preconditions = [
        input.fact_key_gate_green,
        input.deletion_corpus_green,
        input.complexity_slope_green,
        input.settle_report.all_settles_equal,
    ];
    let green_precondition_count = preconditions.iter().filter(|&&green| green).count();
    let demand_primary_ready = green_precondition_count == preconditions.len();

    StreamingIFDSDemandReadinessReportV0 {
        schema_version: STREAMING_IFDS_SCHEMA_VERSION_V0,
        product: "omena-streaming-ifds.demand-readiness-report",
        layer_marker: STREAMING_IFDS_LAYER_MARKER_V0,
        feature_gate: STREAMING_IFDS_FEATURE_GATE_V0,
        fact_key_gate_green: input.fact_key_gate_green,
        deletion_corpus_green: input.deletion_corpus_green,
        complexity_slope_green: input.complexity_slope_green,
        settle_all_equal: input.settle_report.all_settles_equal,
        precondition_count: preconditions.len(),
        green_precondition_count,
        demand_primary_ready,
    }
}

pub fn run_streaming_ifds_demand_v0(
    start_node_ids: &[String],
    target_node_ids: &[String],
    hyperedges: &[UnifiedHypergraphHyperedgeV0],
    events: &[StreamingIfdsEventInputV0],
) -> StreamingIFDSDemandReportV0 {
    let index = streaming_ifds_demand_index_v0(hyperedges);
    run_streaming_ifds_demand_with_index_v0(start_node_ids, target_node_ids, &index, events)
}

pub fn run_streaming_ifds_demand_with_index_v0(
    start_node_ids: &[String],
    target_node_ids: &[String],
    index: &StreamingIFDSDemandIndexV0,
    events: &[StreamingIfdsEventInputV0],
) -> StreamingIFDSDemandReportV0 {
    let slice = index.slice(start_node_ids, target_node_ids);
    let projection_nodes = slice
        .projection_node_ids
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();
    let mut intern_table = StreamingIFDSRunInternTableV0::from_transfer_functions(
        slice
            .transfer_indices
            .iter()
            .map(|index_id| &index.transfer_table.transfers[*index_id]),
        events,
    );
    let mut seen = BTreeSet::<StreamingIFDSInternedFactKeyV0>::new();
    let mut pending = VecDeque::<StreamingIFDSFactV0>::new();
    let mut output = Vec::<StreamingIFDSFactV0>::new();
    let start_nodes = start_node_ids.iter().collect::<BTreeSet<_>>();
    let mut transfer_visit_count = 0usize;

    for event in events {
        if !start_nodes.contains(&event.node_id) || !projection_nodes.contains(&event.node_id) {
            continue;
        }
        let fact = streaming_ifds_fact_v0(
            event.node_id.clone(),
            event.value.clone(),
            vec![format!("event:{}", event.event_id)],
        );
        if seen.insert(intern_table.intern_fact_key(&fact.node_id, &fact.value)) {
            pending.push_back(fact.clone());
            output.push(fact);
        }
    }

    while let Some(fact) = pending.pop_front() {
        for index_id in slice.transfer_indices_for_tail(&fact.node_id) {
            let transfer = &index.transfer_table.transfers[*index_id];
            transfer_visit_count = transfer_visit_count.saturating_add(1);
            let mut provenance = fact.provenance.clone();
            provenance.push(format!("transfer:{}", transfer.hyperedge_id));
            let next_value = apply_streaming_ifds_transfer(transfer, &fact.value);
            let next_fact =
                streaming_ifds_fact_v0(transfer.head_node_id.clone(), next_value, provenance);
            if seen.insert(intern_table.intern_fact_key(&next_fact.node_id, &next_fact.value)) {
                pending.push_back(next_fact.clone());
                output.push(next_fact);
            }
        }
    }

    output.sort_by(|left, right| {
        left.node_id
            .cmp(&right.node_id)
            .then(left.fact_id.cmp(&right.fact_id))
    });
    let forward_node_count = index.forward_node_count(start_node_ids);
    let fact_keys = fact_keys(&output);
    StreamingIFDSDemandReportV0 {
        schema_version: STREAMING_IFDS_SCHEMA_VERSION_V0,
        product: "omena-streaming-ifds.demand-report",
        layer_marker: STREAMING_IFDS_LAYER_MARKER_V0,
        feature_gate: STREAMING_IFDS_FEATURE_GATE_V0,
        start_node_ids: start_node_ids.to_vec(),
        target_node_ids: target_node_ids.to_vec(),
        projection_node_ids: slice.projection_node_ids,
        fact_keys,
        transfer_visit_count,
        slice_scc_count: collect_directed_graph_sccs(&slice.adjacency).len(),
        strict_subset_of_forward_reachable_nodes: !projection_nodes.is_empty()
            && projection_nodes.len() < forward_node_count,
    }
}

/// Compute cross-file reachability over the resolved unified hypergraph.
///
/// This is the crate-owned mechanism behind the CLI/product diagnostic. The
/// caller supplies resolved cross-file hyperedges; this function seeds each node
/// owned by the target style file, runs the exact streaming IFDS oracle, and
/// reports the foreign module paths reached by propagated facts.
pub fn summarize_streaming_ifds_cross_file_reachability_v0(
    target_style_path: &str,
    hyperedges: &[UnifiedHypergraphHyperedgeV0],
) -> StreamingIFDSCrossFileReachabilityReportV0 {
    summarize_streaming_ifds_cross_file_reachability_fast_v0(target_style_path, hyperedges).report
}

/// Target-INDEPENDENT condensation of the reachability graph (rfcs#111, the
/// first C1 slice): node adjacency, SCCs, the SCC DAG, and the node-id
/// grouping by owning path, computed ONCE per wave. Per-target work then
/// reduces to a start-SCC lookup plus a BFS over the SCC DAG — the fast
/// batch arm recomputed all of this on every one of N targets.
#[derive(Debug, Clone)]
pub struct StreamingIfdsReachabilityCondensationV0 {
    sccs: Vec<Vec<String>>,
    scc_by_node: BTreeMap<String, usize>,
    scc_adjacency: BTreeMap<usize, BTreeSet<usize>>,
    node_ids_by_path: BTreeMap<String, Vec<String>>,
}

pub fn streaming_ifds_reachability_condensation_v0(
    hyperedges: &[UnifiedHypergraphHyperedgeV0],
) -> StreamingIfdsReachabilityCondensationV0 {
    let adjacency = streaming_ifds_node_adjacency(hyperedges);
    let sccs = collect_directed_graph_sccs(&adjacency);
    let mut scc_by_node = BTreeMap::<String, usize>::new();
    for (index, scc) in sccs.iter().enumerate() {
        for node_id in scc {
            scc_by_node.insert(node_id.clone(), index);
        }
    }
    let mut scc_adjacency = BTreeMap::<usize, BTreeSet<usize>>::new();
    for (tail, heads) in &adjacency {
        let Some(tail_scc) = scc_by_node.get(tail).copied() else {
            continue;
        };
        scc_adjacency.entry(tail_scc).or_default();
        for head in heads {
            let Some(head_scc) = scc_by_node.get(head).copied() else {
                continue;
            };
            if tail_scc != head_scc {
                scc_adjacency.entry(tail_scc).or_default().insert(head_scc);
            }
        }
    }
    // Mirrors `streaming_ifds_node_ids_for_path` exactly: every node id from
    // tails + heads, deduped and sorted per owning path.
    let mut grouped = BTreeMap::<String, BTreeSet<String>>::new();
    for edge in hyperedges {
        for node_id in edge
            .tail_node_ids
            .iter()
            .chain(std::iter::once(&edge.head_node_id))
        {
            if let Some(path) = streaming_ifds_node_path(node_id) {
                grouped
                    .entry(path.to_string())
                    .or_default()
                    .insert(node_id.clone());
            }
        }
    }
    StreamingIfdsReachabilityCondensationV0 {
        sccs,
        scc_by_node,
        scc_adjacency,
        node_ids_by_path: grouped
            .into_iter()
            .map(|(path, node_ids)| (path, node_ids.into_iter().collect()))
            .collect(),
    }
}

/// Per-target reachability over a prebuilt condensation. Byte-identical to
/// [`summarize_streaming_ifds_cross_file_reachability_v0`] on the same
/// hyperedges (gated by the parity test below).
pub fn summarize_streaming_ifds_cross_file_reachability_with_condensation_v0(
    target_style_path: &str,
    condensation: &StreamingIfdsReachabilityCondensationV0,
) -> StreamingIFDSCrossFileReachabilityReportV0 {
    let start_node_ids = condensation
        .node_ids_by_path
        .get(target_style_path)
        .cloned()
        .unwrap_or_default();
    let mut seen_sccs = BTreeSet::<usize>::new();
    let mut pending_sccs = VecDeque::<usize>::new();
    for start_node_id in &start_node_ids {
        if let Some(scc_index) = condensation.scc_by_node.get(start_node_id).copied()
            && seen_sccs.insert(scc_index)
        {
            pending_sccs.push_back(scc_index);
        }
    }
    while let Some(scc) = pending_sccs.pop_front() {
        for next_scc in condensation.scc_adjacency.get(&scc).into_iter().flatten() {
            if seen_sccs.insert(*next_scc) {
                pending_sccs.push_back(*next_scc);
            }
        }
    }
    let mut reachable_foreign_paths = BTreeSet::<String>::new();
    for scc in seen_sccs {
        let Some(node_ids) = condensation.sccs.get(scc) else {
            continue;
        };
        for node_id in node_ids {
            if let Some(path) = streaming_ifds_node_path(node_id)
                && path != target_style_path
            {
                reachable_foreign_paths.insert(path.to_string());
            }
        }
    }
    let reachable_foreign_paths = reachable_foreign_paths.into_iter().collect::<Vec<_>>();
    StreamingIFDSCrossFileReachabilityReportV0 {
        schema_version: STREAMING_IFDS_SCHEMA_VERSION_V0,
        product: "omena-streaming-ifds.cross-file-reachability-report",
        layer_marker: STREAMING_IFDS_LAYER_MARKER_V0,
        feature_gate: STREAMING_IFDS_FEATURE_GATE_V0,
        target_style_path: target_style_path.to_string(),
        start_node_count: start_node_ids.len(),
        reachable_foreign_path_count: reachable_foreign_paths.len(),
        reachable_foreign_paths,
        analysis_report_count: start_node_ids.len(),
        precision_parity_with_batch: true,
        exact_default: true,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct StreamingIfdsCrossFileReachabilityFastSummaryV0 {
    report: StreamingIFDSCrossFileReachabilityReportV0,
    traversal_step_count: usize,
}

fn summarize_streaming_ifds_cross_file_reachability_fast_v0(
    target_style_path: &str,
    hyperedges: &[UnifiedHypergraphHyperedgeV0],
) -> StreamingIfdsCrossFileReachabilityFastSummaryV0 {
    let start_node_ids = streaming_ifds_node_ids_for_path(target_style_path, hyperedges);
    let adjacency = streaming_ifds_node_adjacency(hyperedges);
    let sccs = collect_directed_graph_sccs(&adjacency);
    let mut scc_by_node = BTreeMap::<String, usize>::new();
    for (index, scc) in sccs.iter().enumerate() {
        for node_id in scc {
            scc_by_node.insert(node_id.clone(), index);
        }
    }
    let mut start_sccs = BTreeSet::<usize>::new();
    for start_node_id in &start_node_ids {
        if let Some(scc_index) = scc_by_node.get(start_node_id).copied() {
            start_sccs.insert(scc_index);
        }
    }
    let mut scc_adjacency = BTreeMap::<usize, BTreeSet<usize>>::new();
    let mut traversal_step_count = 0usize;
    for (tail, heads) in &adjacency {
        let Some(tail_scc) = scc_by_node.get(tail).copied() else {
            continue;
        };
        scc_adjacency.entry(tail_scc).or_default();
        for head in heads {
            traversal_step_count = traversal_step_count.saturating_add(1);
            let Some(head_scc) = scc_by_node.get(head).copied() else {
                continue;
            };
            if tail_scc != head_scc {
                scc_adjacency.entry(tail_scc).or_default().insert(head_scc);
            }
        }
    }
    let mut seen_sccs = BTreeSet::<usize>::new();
    let mut pending_sccs = VecDeque::<usize>::new();
    for scc in start_sccs {
        if seen_sccs.insert(scc) {
            pending_sccs.push_back(scc);
        }
    }
    while let Some(scc) = pending_sccs.pop_front() {
        for next_scc in scc_adjacency.get(&scc).into_iter().flatten() {
            traversal_step_count = traversal_step_count.saturating_add(1);
            if seen_sccs.insert(*next_scc) {
                pending_sccs.push_back(*next_scc);
            }
        }
    }
    let mut reachable_foreign_paths = BTreeSet::<String>::new();

    for scc in seen_sccs {
        let Some(node_ids) = sccs.get(scc) else {
            continue;
        };
        for node_id in node_ids {
            if let Some(path) = streaming_ifds_node_path(node_id)
                && path != target_style_path
            {
                reachable_foreign_paths.insert(path.to_string());
            }
        }
    }

    let reachable_foreign_paths = reachable_foreign_paths.into_iter().collect::<Vec<_>>();
    StreamingIfdsCrossFileReachabilityFastSummaryV0 {
        report: StreamingIFDSCrossFileReachabilityReportV0 {
            schema_version: STREAMING_IFDS_SCHEMA_VERSION_V0,
            product: "omena-streaming-ifds.cross-file-reachability-report",
            layer_marker: STREAMING_IFDS_LAYER_MARKER_V0,
            feature_gate: STREAMING_IFDS_FEATURE_GATE_V0,
            target_style_path: target_style_path.to_string(),
            start_node_count: start_node_ids.len(),
            reachable_foreign_path_count: reachable_foreign_paths.len(),
            reachable_foreign_paths,
            analysis_report_count: start_node_ids.len(),
            precision_parity_with_batch: true,
            exact_default: true,
        },
        traversal_step_count,
    }
}

#[cfg(test)]
fn summarize_streaming_ifds_cross_file_reachability_oracle_v0(
    target_style_path: &str,
    hyperedges: &[UnifiedHypergraphHyperedgeV0],
) -> StreamingIFDSCrossFileReachabilityReportV0 {
    let start_node_ids = streaming_ifds_node_ids_for_path(target_style_path, hyperedges);
    let oracle = ExactStreamingConnectivityOracleV0::default();
    let mut reachable_foreign_paths = BTreeSet::<String>::new();
    let mut precision_parity_with_batch = true;
    let mut analysis_report_count = 0usize;

    for start_node_id in &start_node_ids {
        let event = streaming_ifds_event_input_v0(
            format!("foreign-reference-seed:{start_node_id}"),
            0,
            start_node_id.clone(),
            AbstractClassValueV0::Top,
            None,
        );
        let report = run_streaming_ifds_exact_v0(
            format!("cross-file-reachability:{target_style_path}"),
            start_node_id.clone(),
            hyperedges,
            std::slice::from_ref(&event),
            &oracle,
            None,
        );
        analysis_report_count += 1;
        precision_parity_with_batch &= report.precision_parity_with_batch;
        for fact in &report.output_facts {
            if let Some(path) = streaming_ifds_node_path(fact.node_id.as_str())
                && path != target_style_path
            {
                reachable_foreign_paths.insert(path.to_string());
            }
        }
    }

    let reachable_foreign_paths = reachable_foreign_paths.into_iter().collect::<Vec<_>>();
    StreamingIFDSCrossFileReachabilityReportV0 {
        schema_version: STREAMING_IFDS_SCHEMA_VERSION_V0,
        product: "omena-streaming-ifds.cross-file-reachability-report",
        layer_marker: STREAMING_IFDS_LAYER_MARKER_V0,
        feature_gate: STREAMING_IFDS_FEATURE_GATE_V0,
        target_style_path: target_style_path.to_string(),
        start_node_count: start_node_ids.len(),
        reachable_foreign_path_count: reachable_foreign_paths.len(),
        reachable_foreign_paths,
        analysis_report_count,
        precision_parity_with_batch,
        exact_default: true,
    }
}

pub fn streaming_ifds_transfer_functions_v0(
    hyperedges: &[UnifiedHypergraphHyperedgeV0],
) -> Vec<StreamingIFDSTransferFunctionV0> {
    streaming_ifds_transfer_table_v0(hyperedges).transfers
}

pub fn streaming_ifds_demand_index_v0(
    hyperedges: &[UnifiedHypergraphHyperedgeV0],
) -> StreamingIFDSDemandIndexV0 {
    let transfer_table = streaming_ifds_transfer_table_v0(hyperedges);
    let mut incoming_transfers_by_head_node_id = BTreeMap::<String, Vec<usize>>::new();
    for (index, transfer) in transfer_table.transfers.iter().enumerate() {
        incoming_transfers_by_head_node_id
            .entry(transfer.head_node_id.clone())
            .or_default()
            .push(index);
    }
    StreamingIFDSDemandIndexV0 {
        transfer_table,
        incoming_transfers_by_head_node_id,
    }
}

#[derive(Debug, Clone)]
struct StreamingIFDSTransferTableV0 {
    transfers: Vec<StreamingIFDSTransferFunctionV0>,
    transfers_by_tail_node_id: BTreeMap<String, Vec<usize>>,
}

impl StreamingIFDSTransferTableV0 {
    fn len(&self) -> usize {
        self.transfers.len()
    }

    fn transfers_for_tail<'a>(
        &'a self,
        node_id: &str,
    ) -> impl Iterator<Item = &'a StreamingIFDSTransferFunctionV0> + 'a {
        self.transfers_by_tail_node_id
            .get(node_id)
            .into_iter()
            .flat_map(|indices| indices.iter())
            .map(|index| &self.transfers[*index])
    }
}

#[derive(Debug, Clone)]
pub struct StreamingIFDSDemandIndexV0 {
    transfer_table: StreamingIFDSTransferTableV0,
    incoming_transfers_by_head_node_id: BTreeMap<String, Vec<usize>>,
}

#[derive(Debug, Clone)]
struct StreamingIFDSDemandSliceV0 {
    projection_node_ids: Vec<String>,
    transfer_indices: Vec<usize>,
    transfer_indices_by_tail_node_id: BTreeMap<String, Vec<usize>>,
    adjacency: BTreeMap<String, BTreeSet<String>>,
}

impl StreamingIFDSDemandIndexV0 {
    fn slice(
        &self,
        start_node_ids: &[String],
        target_node_ids: &[String],
    ) -> StreamingIFDSDemandSliceV0 {
        let (projection_nodes, transfer_indices) = if target_node_ids.is_empty() {
            let projection_nodes = self.forward_node_ids(start_node_ids);
            let transfer_indices = self
                .transfer_table
                .transfers
                .iter()
                .enumerate()
                .filter_map(|(index, transfer)| {
                    let in_projection = projection_nodes.contains(&transfer.head_node_id)
                        && transfer
                            .tail_node_ids
                            .iter()
                            .any(|tail| projection_nodes.contains(tail));
                    in_projection.then_some(index)
                })
                .collect::<BTreeSet<_>>();
            (projection_nodes, transfer_indices)
        } else {
            self.backward_slice_from_targets(target_node_ids)
        };

        let mut transfer_indices_by_tail_node_id = BTreeMap::<String, Vec<usize>>::new();
        let mut adjacency = BTreeMap::<String, BTreeSet<String>>::new();
        for index in &transfer_indices {
            let transfer = &self.transfer_table.transfers[*index];
            if !projection_nodes.contains(&transfer.head_node_id) {
                continue;
            }
            for tail_node_id in &transfer.tail_node_ids {
                if !projection_nodes.contains(tail_node_id) {
                    continue;
                }
                transfer_indices_by_tail_node_id
                    .entry(tail_node_id.clone())
                    .or_default()
                    .push(*index);
                adjacency
                    .entry(tail_node_id.clone())
                    .or_default()
                    .insert(transfer.head_node_id.clone());
            }
        }

        StreamingIFDSDemandSliceV0 {
            projection_node_ids: projection_nodes.into_iter().collect(),
            transfer_indices: transfer_indices.into_iter().collect(),
            transfer_indices_by_tail_node_id,
            adjacency,
        }
    }

    fn forward_node_count(&self, start_node_ids: &[String]) -> usize {
        self.forward_node_ids(start_node_ids).len()
    }

    fn forward_node_ids(&self, start_node_ids: &[String]) -> BTreeSet<String> {
        let adjacency = self.forward_adjacency();
        let mut forward = BTreeSet::<String>::new();
        for start_node_id in start_node_ids {
            forward.insert(start_node_id.clone());
            forward.extend(collect_reachable_node_ids(start_node_id, &adjacency));
        }
        forward
    }

    fn backward_slice_from_targets(
        &self,
        target_node_ids: &[String],
    ) -> (BTreeSet<String>, BTreeSet<usize>) {
        let mut projection_nodes = BTreeSet::<String>::new();
        let mut transfer_indices = BTreeSet::<usize>::new();
        let mut pending = target_node_ids.iter().cloned().collect::<VecDeque<_>>();
        while let Some(node_id) = pending.pop_front() {
            if !projection_nodes.insert(node_id.clone()) {
                continue;
            }
            if let Some(indices) = self.incoming_transfers_by_head_node_id.get(&node_id) {
                for index in indices {
                    transfer_indices.insert(*index);
                    for tail_node_id in &self.transfer_table.transfers[*index].tail_node_ids {
                        pending.push_back(tail_node_id.clone());
                    }
                }
            }
        }
        (projection_nodes, transfer_indices)
    }

    fn forward_adjacency(&self) -> BTreeMap<String, BTreeSet<String>> {
        let mut adjacency = BTreeMap::<String, BTreeSet<String>>::new();
        for transfer in &self.transfer_table.transfers {
            for tail_node_id in &transfer.tail_node_ids {
                adjacency
                    .entry(tail_node_id.clone())
                    .or_default()
                    .insert(transfer.head_node_id.clone());
            }
        }
        adjacency
    }
}

impl StreamingIFDSDemandSliceV0 {
    fn transfer_indices_for_tail<'a>(
        &'a self,
        node_id: &str,
    ) -> impl Iterator<Item = &'a usize> + 'a {
        self.transfer_indices_by_tail_node_id
            .get(node_id)
            .into_iter()
            .flat_map(|indices| indices.iter())
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct StreamingIFDSPropagationStatsV0 {
    popped_fact_count: usize,
    transfer_visit_count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct StreamingIFDSInternedNodeKeyV0(u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct StreamingIFDSInternedFactKeyV0(u32);

#[derive(Debug, Default)]
struct StreamingIFDSRunInternTableV0 {
    node_ids_by_value: BTreeMap<String, StreamingIFDSInternedNodeKeyV0>,
    node_values: Vec<String>,
    fact_keys_by_value: BTreeMap<String, StreamingIFDSInternedFactKeyV0>,
    fact_key_values: Vec<String>,
}

impl StreamingIFDSRunInternTableV0 {
    fn from_inputs(
        transfer_table: &StreamingIFDSTransferTableV0,
        events: &[StreamingIfdsEventInputV0],
    ) -> Self {
        Self::from_transfer_functions(transfer_table.transfers.iter(), events)
    }

    fn from_transfer_functions<'a, I>(transfers: I, events: &[StreamingIfdsEventInputV0]) -> Self
    where
        I: IntoIterator<Item = &'a StreamingIFDSTransferFunctionV0>,
    {
        let mut table = Self::default();
        for event in events {
            table.intern_node_id(&event.node_id);
        }
        for transfer in transfers {
            table.intern_node_id(&transfer.head_node_id);
            for tail_node_id in &transfer.tail_node_ids {
                table.intern_node_id(tail_node_id);
            }
        }
        table
    }

    fn intern_node_id(&mut self, node_id: &str) -> StreamingIFDSInternedNodeKeyV0 {
        if let Some(key) = self.node_ids_by_value.get(node_id) {
            return *key;
        }
        let key = StreamingIFDSInternedNodeKeyV0(next_intern_index(self.node_values.len()));
        self.node_values.push(node_id.to_string());
        self.node_ids_by_value.insert(node_id.to_string(), key);
        key
    }

    fn intern_fact_key(
        &mut self,
        node_id: &str,
        value: &AbstractClassValueV0,
    ) -> StreamingIFDSInternedFactKeyV0 {
        self.intern_node_id(node_id);
        let key_value = fact_key(node_id, value);
        if let Some(key) = self.fact_keys_by_value.get(key_value.as_str()) {
            return *key;
        }
        let key = StreamingIFDSInternedFactKeyV0(next_intern_index(self.fact_key_values.len()));
        self.fact_key_values.push(key_value.clone());
        self.fact_keys_by_value.insert(key_value, key);
        key
    }

    #[cfg(test)]
    fn fact_key_value(&self, key: StreamingIFDSInternedFactKeyV0) -> &str {
        self.fact_key_values
            .get(key.0 as usize)
            .map(String::as_str)
            .unwrap_or("")
    }
}

fn next_intern_index(len: usize) -> u32 {
    if len > u32::MAX as usize {
        u32::MAX
    } else {
        len as u32
    }
}

fn streaming_ifds_transfer_table_v0(
    hyperedges: &[UnifiedHypergraphHyperedgeV0],
) -> StreamingIFDSTransferTableV0 {
    let transfers = hyperedges
        .iter()
        .map(|edge| StreamingIFDSTransferFunctionV0 {
            schema_version: STREAMING_IFDS_SCHEMA_VERSION_V0,
            product: "omena-streaming-ifds.transfer-function",
            layer_marker: STREAMING_IFDS_LAYER_MARKER_V0,
            feature_gate: STREAMING_IFDS_FEATURE_GATE_V0,
            hyperedge_id: edge.hyperedge_id.clone(),
            edge_kind: edge.edge_kind,
            tail_node_ids: edge.tail_node_ids.clone(),
            head_node_id: edge.head_node_id.clone(),
            transfer_kind: streaming_ifds_transfer_kind(edge.edge_kind),
        })
        .collect::<Vec<_>>();
    let mut transfers_by_tail_node_id = BTreeMap::<String, Vec<usize>>::new();
    for (index, transfer) in transfers.iter().enumerate() {
        for tail_node_id in &transfer.tail_node_ids {
            transfers_by_tail_node_id
                .entry(tail_node_id.clone())
                .or_default()
                .push(index);
        }
    }
    StreamingIFDSTransferTableV0 {
        transfers,
        transfers_by_tail_node_id,
    }
}

pub fn streaming_ifds_summary_cache_entry_v0(
    start_node_id: impl Into<String>,
    reachable_node_ids: Vec<String>,
    fact_keys: Vec<String>,
    reused_from_previous: bool,
) -> StreamingIFDSSummaryCacheEntryV0 {
    let mut canonical_parts = vec![start_node_id.into()];
    canonical_parts.extend(reachable_node_ids.iter().cloned());
    canonical_parts.extend(fact_keys.iter().cloned());
    StreamingIFDSSummaryCacheEntryV0 {
        schema_version: STREAMING_IFDS_SCHEMA_VERSION_V0,
        product: "omena-streaming-ifds.summary-cache-entry",
        layer_marker: STREAMING_IFDS_LAYER_MARKER_V0,
        feature_gate: STREAMING_IFDS_FEATURE_GATE_V0,
        start_node_id: canonical_parts.first().cloned().unwrap_or_default(),
        reachable_node_ids,
        fact_keys,
        summary_hash: format!("fnv64:{:016x}", stable_hash(&canonical_parts)),
        reused_from_previous,
    }
}

#[cfg(feature = "with-frame-rule")]
pub fn streaming_ifds_frame_rule_bridge_policy_v0() -> StreamingIfdsFrameRuleBridgePolicyV0 {
    StreamingIfdsFrameRuleBridgePolicyV0 {
        schema_version: STREAMING_IFDS_SCHEMA_VERSION_V0,
        product: "omena-streaming-ifds.frame-rule-bridge-policy",
        layer_marker: STREAMING_IFDS_LAYER_MARKER_V0,
        feature_gate: "with-frame-rule",
        coarse_policy: "frameFootprintReachability",
        fine_policy: "incidfaTouchedFactFilter",
        activation: "onlyWhenStreamingIfdsAndFrameRuleFeaturesAreEnabled",
    }
}

pub fn streaming_ifds_solver_hygiene_policy_v0() -> StreamingIfdsSolverHygienePolicyV0 {
    StreamingIfdsSolverHygienePolicyV0 {
        schema_version: STREAMING_IFDS_SCHEMA_VERSION_V0,
        product: "omena-streaming-ifds.solver-hygiene-policy",
        layer_marker: STREAMING_IFDS_LAYER_MARKER_V0,
        feature_gate: STREAMING_IFDS_FEATURE_GATE_V0,
        summary_cache_feedback_policy: "emitEvidenceCacheButDoNotFeedProductPaths",
        non_product_cache_feedback_scope: "engineShadowRunnerPrecisionEvidenceOnly",
        cache_feedback_activation: "requiresNonCountConsumerAndPrecisionParityFallback",
        reference_edge_value_policy: "identityOnReferenceAndAliasEdges",
        concrete_value_owner: "omena-sif.variable-export.value-repr",
        deferred_value_flow_candidates: vec![
            "composesFiniteSetConsumer",
            "icssValueAliasReExportChain",
        ],
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

#[cfg(test)]
fn propagate_ifds_facts(
    hyperedges: &[UnifiedHypergraphHyperedgeV0],
    events: &[StreamingIfdsEventInputV0],
) -> Vec<StreamingIFDSFactV0> {
    let transfer_table = streaming_ifds_transfer_table_v0(hyperedges);
    propagate_ifds_facts_with_table(&transfer_table, events)
}

fn propagate_ifds_facts_with_table(
    transfer_table: &StreamingIFDSTransferTableV0,
    events: &[StreamingIfdsEventInputV0],
) -> Vec<StreamingIFDSFactV0> {
    propagate_ifds_facts_with_table_and_stats(transfer_table, events).0
}

fn propagate_ifds_facts_with_table_and_stats(
    transfer_table: &StreamingIFDSTransferTableV0,
    events: &[StreamingIfdsEventInputV0],
) -> (Vec<StreamingIFDSFactV0>, StreamingIFDSPropagationStatsV0) {
    let mut intern_table = StreamingIFDSRunInternTableV0::from_inputs(transfer_table, events);
    let mut seen = BTreeSet::<StreamingIFDSInternedFactKeyV0>::new();
    let mut pending = VecDeque::<StreamingIFDSFactV0>::new();
    let mut output = Vec::<StreamingIFDSFactV0>::new();
    let mut stats = StreamingIFDSPropagationStatsV0::default();

    for event in events {
        let fact = streaming_ifds_fact_v0(
            event.node_id.clone(),
            event.value.clone(),
            vec![format!("event:{}", event.event_id)],
        );
        if seen.insert(intern_table.intern_fact_key(&fact.node_id, &fact.value)) {
            pending.push_back(fact.clone());
            output.push(fact);
        }
    }

    while let Some(fact) = pending.pop_front() {
        stats.popped_fact_count = stats.popped_fact_count.saturating_add(1);
        for transfer in transfer_table.transfers_for_tail(&fact.node_id) {
            stats.transfer_visit_count = stats.transfer_visit_count.saturating_add(1);
            let mut provenance = fact.provenance.clone();
            provenance.push(format!("transfer:{}", transfer.hyperedge_id));
            let next_value = apply_streaming_ifds_transfer(transfer, &fact.value);
            let next_fact =
                streaming_ifds_fact_v0(transfer.head_node_id.clone(), next_value, provenance);
            if seen.insert(intern_table.intern_fact_key(&next_fact.node_id, &next_fact.value)) {
                pending.push_back(next_fact.clone());
                output.push(next_fact);
            }
        }
    }

    output.sort_by(|left, right| {
        left.node_id
            .cmp(&right.node_id)
            .then(left.fact_id.cmp(&right.fact_id))
    });
    (output, stats)
}

/// Nodes that the changed event nodes can still reach over the *current* graph.
/// This is the incremental dirty sub-graph: facts at these nodes are re-derived
/// from scratch, everything else may be reused from the prior fact set.
fn incremental_dirty_nodes(
    transfer_table: &StreamingIFDSTransferTableV0,
    events: &[StreamingIfdsEventInputV0],
) -> BTreeSet<String> {
    propagate_ifds_facts_with_table(transfer_table, events)
        .into_iter()
        .map(|fact| fact.node_id)
        .collect()
}

/// Incremental/streaming IFDS fact-key set.
///
/// Distinct from [`propagate_ifds_facts`] (the batch oracle that recomputes
/// every fact from scratch): this path re-derives only the dirty sub-graph
/// reachable from the changed event nodes and reuses prior fact keys that fall
/// entirely outside it. A prior fact key is reused (not recomputed) iff its node
/// is not in the dirty region. That reuse is what makes parity with the batch
/// oracle a real check: if a reused prior fact is stale — i.e. the current graph
/// no longer produces it because a supporting edge was removed — the incremental
/// key set retains it while the batch key set drops it, so the two diverge.
fn incremental_fact_keys(
    transfer_table: &StreamingIFDSTransferTableV0,
    events: &[StreamingIfdsEventInputV0],
    previous_fact_keys: &BTreeSet<String>,
) -> Vec<String> {
    // Cold start (no prior facts): the dirty region is the whole reachable graph,
    // so the incremental path coincides with a full recompute.
    if previous_fact_keys.is_empty() {
        return fact_keys(&propagate_ifds_facts_with_table(transfer_table, events));
    }

    let dirty_nodes = incremental_dirty_nodes(transfer_table, events);
    let mut keys = fact_keys(&propagate_ifds_facts_with_table(transfer_table, events))
        .into_iter()
        .collect::<BTreeSet<_>>();
    for key in previous_fact_keys {
        let node_id = key.split_once('|').map(|(node, _)| node).unwrap_or(key);
        if !dirty_nodes.contains(node_id) {
            keys.insert(key.clone());
        }
    }
    keys.into_iter().collect()
}

/// Incremental/streaming IFDS facts whose key set equals [`incremental_fact_keys`].
/// Dirty-region facts are re-derived with full provenance; reused prior facts
/// outside the dirty region are carried forward verbatim from their key.
fn incremental_propagate_ifds_facts(
    transfer_table: &StreamingIFDSTransferTableV0,
    events: &[StreamingIfdsEventInputV0],
    previous_fact_keys: &BTreeSet<String>,
) -> Vec<StreamingIFDSFactV0> {
    let mut output = propagate_ifds_facts_with_table(transfer_table, events);
    if !previous_fact_keys.is_empty() {
        let dirty_nodes = incremental_dirty_nodes(transfer_table, events);
        let mut seen = output
            .iter()
            .map(|fact| fact_key(&fact.node_id, &fact.value))
            .collect::<BTreeSet<_>>();
        for key in previous_fact_keys {
            let node_id = key.split_once('|').map(|(node, _)| node).unwrap_or(key);
            if dirty_nodes.contains(node_id) {
                continue;
            }
            if seen.insert(key.clone()) {
                output.push(reused_fact_from_key(key));
            }
        }
    }

    output.sort_by(|left, right| {
        left.node_id
            .cmp(&right.node_id)
            .then(left.fact_id.cmp(&right.fact_id))
    });
    output
}

/// Materialize a reused prior fact directly from its `node_id|value-key` key.
/// The reconstructed value carries the verbatim value-key so the fact's own
/// `fact_key` is byte-identical to the reused key, keeping the materialized
/// fact set's key set equal to [`incremental_fact_keys`].
fn reused_fact_from_key(key: &str) -> StreamingIFDSFactV0 {
    let (node_id, value_key) = key.split_once('|').unwrap_or((key, ""));
    let value = match value_key {
        "bottom" => AbstractClassValueV0::Bottom,
        "top" => AbstractClassValueV0::Top,
        other if other.starts_with("finiteSet:") => AbstractClassValueV0::FiniteSet {
            values: other
                .trim_start_matches("finiteSet:")
                .split(',')
                .filter(|value| !value.is_empty())
                .map(str::to_string)
                .collect(),
        },
        other => AbstractClassValueV0::Exact {
            value: other.strip_prefix("exact:").unwrap_or(other).to_string(),
        },
    };
    StreamingIFDSFactV0 {
        schema_version: STREAMING_IFDS_SCHEMA_VERSION_V0,
        product: "omena-streaming-ifds.fact",
        layer_marker: STREAMING_IFDS_LAYER_MARKER_V0,
        feature_gate: STREAMING_IFDS_FEATURE_GATE_V0,
        fact_id: format!(
            "fact:{:016x}",
            stable_hash(std::slice::from_ref(&key.to_string()))
        ),
        node_id: node_id.to_string(),
        value,
        provenance: vec![format!("reused:{key}")],
    }
}

fn streaming_ifds_transfer_kind(edge_kind: UnifiedHypergraphEdgeKindV0) -> &'static str {
    match edge_kind {
        UnifiedHypergraphEdgeKindV0::ComposesLocal
        | UnifiedHypergraphEdgeKindV0::ComposesGlobal
        | UnifiedHypergraphEdgeKindV0::ComposesExternal => "composeClassSet",
        UnifiedHypergraphEdgeKindV0::Value | UnifiedHypergraphEdgeKindV0::Icss => {
            "valueAliasPreserving"
        }
        UnifiedHypergraphEdgeKindV0::SassUse
        | UnifiedHypergraphEdgeKindV0::SassForward
        | UnifiedHypergraphEdgeKindV0::SassImport
        | UnifiedHypergraphEdgeKindV0::LessImport
        | UnifiedHypergraphEdgeKindV0::LessModuleGraphClosure
        | UnifiedHypergraphEdgeKindV0::ForeignReference => "semanticReferencePreserving",
        _ => "semanticReferencePreserving",
    }
}

fn apply_streaming_ifds_transfer(
    transfer: &StreamingIFDSTransferFunctionV0,
    value: &AbstractClassValueV0,
) -> AbstractClassValueV0 {
    match transfer.edge_kind {
        UnifiedHypergraphEdgeKindV0::ComposesLocal
        | UnifiedHypergraphEdgeKindV0::ComposesGlobal
        | UnifiedHypergraphEdgeKindV0::ComposesExternal => {
            widen_class_value_with_composed_head(value, &transfer.head_node_id)
        }
        UnifiedHypergraphEdgeKindV0::Value
        | UnifiedHypergraphEdgeKindV0::Icss
        | UnifiedHypergraphEdgeKindV0::SassUse
        | UnifiedHypergraphEdgeKindV0::SassForward
        | UnifiedHypergraphEdgeKindV0::SassImport
        | UnifiedHypergraphEdgeKindV0::LessImport
        | UnifiedHypergraphEdgeKindV0::LessModuleGraphClosure
        | UnifiedHypergraphEdgeKindV0::ForeignReference => value.clone(),
        _ => value.clone(),
    }
}

fn widen_class_value_with_composed_head(
    value: &AbstractClassValueV0,
    head_node_id: &str,
) -> AbstractClassValueV0 {
    let head_token = class_token_from_node_id(head_node_id);
    match value {
        AbstractClassValueV0::Bottom | AbstractClassValueV0::Top => value.clone(),
        AbstractClassValueV0::Exact { value } => finite_class_set([value.clone(), head_token]),
        AbstractClassValueV0::FiniteSet { values } => {
            let mut widened = values.clone();
            widened.push(head_token);
            finite_class_set(widened)
        }
        _ => value.clone(),
    }
}

fn finite_class_set(values: impl IntoIterator<Item = String>) -> AbstractClassValueV0 {
    let mut values = values.into_iter().collect::<Vec<_>>();
    values.sort();
    values.dedup();
    AbstractClassValueV0::FiniteSet { values }
}

fn class_token_from_node_id(node_id: &str) -> String {
    node_id
        .rsplit(['/', '#', '.', ':', '|'])
        .find(|segment| !segment.is_empty())
        .unwrap_or(node_id)
        .to_string()
}

fn streaming_ifds_fact_v0(
    node_id: String,
    value: AbstractClassValueV0,
    provenance: Vec<String>,
) -> StreamingIFDSFactV0 {
    let key = fact_key(&node_id, &value);
    StreamingIFDSFactV0 {
        schema_version: STREAMING_IFDS_SCHEMA_VERSION_V0,
        product: "omena-streaming-ifds.fact",
        layer_marker: STREAMING_IFDS_LAYER_MARKER_V0,
        feature_gate: STREAMING_IFDS_FEATURE_GATE_V0,
        fact_id: format!("fact:{:016x}", stable_hash(std::slice::from_ref(&key))),
        node_id,
        value,
        provenance,
    }
}

fn fact_keys(facts: &[StreamingIFDSFactV0]) -> Vec<String> {
    facts
        .iter()
        .map(|fact| fact_key(&fact.node_id, &fact.value))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn project_fact_keys_to_nodes(fact_keys: &[String], node_ids: &[String]) -> Vec<String> {
    let node_ids = node_ids.iter().map(String::as_str).collect::<BTreeSet<_>>();
    fact_keys
        .iter()
        .filter(|key| {
            key.rsplit_once('|')
                .is_some_and(|(node_id, _value)| node_ids.contains(node_id))
        })
        .cloned()
        .collect()
}

fn fact_key(node_id: &str, value: &AbstractClassValueV0) -> String {
    format!("{node_id}|{}", abstract_class_value_key(value))
}

fn abstract_class_value_key(value: &AbstractClassValueV0) -> String {
    match value {
        AbstractClassValueV0::Bottom => "bottom".to_string(),
        AbstractClassValueV0::Exact { value } => format!("exact:{value}"),
        AbstractClassValueV0::FiniteSet { values } => {
            let mut values = values.clone();
            values.sort();
            format!("finiteSet:{}", values.join(","))
        }
        AbstractClassValueV0::Automaton { automaton, .. } => automaton_key(automaton),
        AbstractClassValueV0::Prefix { prefix, .. } => format!("prefix:{prefix}"),
        AbstractClassValueV0::Suffix { suffix, .. } => format!("suffix:{suffix}"),
        AbstractClassValueV0::PrefixSuffix {
            prefix,
            suffix,
            min_length,
            ..
        } => format!("prefixSuffix:{prefix}:{suffix}:{min_length}"),
        AbstractClassValueV0::CharInclusion {
            must_chars,
            may_chars,
            may_include_other_chars,
            ..
        } => format!("charInclusion:{must_chars}:{may_chars}:{may_include_other_chars}"),
        AbstractClassValueV0::Composite {
            prefix,
            suffix,
            min_length,
            must_chars,
            may_chars,
            may_include_other_chars,
            ..
        } => format!(
            "composite:{}:{}:{}:{must_chars}:{may_chars}:{may_include_other_chars}",
            prefix.as_deref().unwrap_or("-"),
            suffix.as_deref().unwrap_or("-"),
            min_length
                .map(|value| value.to_string())
                .unwrap_or_else(|| "-".to_string())
        ),
        AbstractClassValueV0::Top => "top".to_string(),
    }
}

fn stable_hash(parts: &[String]) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for part in parts {
        for byte in part.as_bytes() {
            hash ^= u64::from(*byte);
            hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
        }
        hash ^= 0xff;
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

fn polylog_query_bound(node_count: usize) -> usize {
    let log2_ceil = usize::BITS as usize - node_count.max(1).leading_zeros() as usize;
    log2_ceil.saturating_mul(log2_ceil).max(1)
}

fn streaming_ifds_node_ids_for_path(
    target_style_path: &str,
    hyperedges: &[UnifiedHypergraphHyperedgeV0],
) -> Vec<String> {
    hyperedges
        .iter()
        .flat_map(|edge| {
            edge.tail_node_ids
                .iter()
                .chain(std::iter::once(&edge.head_node_id))
        })
        .filter(|node_id| streaming_ifds_node_path(node_id) == Some(target_style_path))
        .cloned()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn streaming_ifds_node_path(node_id: &str) -> Option<&str> {
    let mut parts = node_id.splitn(3, '|');
    let _kind = parts.next()?;
    parts.next()
}

fn streaming_ifds_node_adjacency(
    hyperedges: &[UnifiedHypergraphHyperedgeV0],
) -> BTreeMap<String, BTreeSet<String>> {
    let mut adjacency = BTreeMap::<String, BTreeSet<String>>::new();
    for edge in hyperedges {
        adjacency.entry(edge.head_node_id.clone()).or_default();
        for tail in &edge.tail_node_ids {
            adjacency
                .entry(tail.clone())
                .or_default()
                .insert(edge.head_node_id.clone());
        }
    }
    adjacency
}

fn streaming_ifds_reverse_adjacency(
    hyperedges: &[UnifiedHypergraphHyperedgeV0],
) -> BTreeMap<String, BTreeSet<String>> {
    let mut reverse = BTreeMap::<String, BTreeSet<String>>::new();
    for edge in hyperedges {
        reverse.entry(edge.head_node_id.clone()).or_default();
        for tail in &edge.tail_node_ids {
            reverse
                .entry(edge.head_node_id.clone())
                .or_default()
                .insert(tail.clone());
            reverse.entry(tail.clone()).or_default();
        }
    }
    reverse
}

fn reachability_incidence_signatures(
    hyperedges: &[UnifiedHypergraphHyperedgeV0],
) -> BTreeMap<String, BTreeSet<String>> {
    let mut signatures = BTreeMap::<String, BTreeSet<String>>::new();
    for edge in hyperedges {
        signatures.entry(edge.head_node_id.clone()).or_default();
        for tail in &edge.tail_node_ids {
            signatures.entry(tail.clone()).or_default().insert(format!(
                "out:{}:{}:{}",
                edge.edge_kind.as_wire_label(),
                edge.hyperedge_id,
                edge.head_node_id
            ));
            signatures
                .entry(edge.head_node_id.clone())
                .or_default()
                .insert(format!(
                    "in:{}:{}:{}",
                    edge.edge_kind.as_wire_label(),
                    edge.hyperedge_id,
                    tail
                ));
        }
    }
    signatures
}

fn reachable_node_ids_with_work(
    start_node_id: &str,
    hyperedges: &[UnifiedHypergraphHyperedgeV0],
) -> (Vec<String>, usize) {
    let adjacency = streaming_ifds_node_adjacency(hyperedges);
    let mut seen = BTreeSet::new();
    let mut pending = VecDeque::from([start_node_id.to_string()]);
    let mut node_visit_count = 0usize;
    while let Some(current) = pending.pop_front() {
        node_visit_count = node_visit_count.saturating_add(1);
        for target in adjacency.get(current.as_str()).into_iter().flatten() {
            if seen.insert(target.clone()) {
                pending.push_back(target.clone());
            }
        }
    }
    (seen.into_iter().collect(), node_visit_count)
}

fn exact_reachable_node_ids(
    start_node_id: &str,
    hyperedges: &[UnifiedHypergraphHyperedgeV0],
) -> Vec<String> {
    // Build this crate's OWN adjacency (its node space), but share the single reachability BFS
    // loop owned by omena-cross-file-summary (SLICE-1.5; the duplicate loop is removed here).
    collect_reachable_node_ids(start_node_id, &streaming_ifds_node_adjacency(hyperedges))
}

fn previous_reachable_node_ids_for_start(
    previous_cache: Option<&[StreamingIFDSSummaryCacheEntryV0]>,
    start_node_id: &str,
) -> BTreeSet<String> {
    previous_cache
        .into_iter()
        .flatten()
        .filter(|entry| entry.start_node_id == start_node_id)
        .flat_map(|entry| entry.reachable_node_ids.iter().cloned())
        .collect()
}

pub fn incremental_reachable_node_ids_zset(
    start_node_id: impl Into<String>,
    hyperedges: &[UnifiedHypergraphHyperedgeV0],
    events: &[StreamingIfdsEventInputV0],
    previous_reachable: impl IntoIterator<Item = String>,
) -> ReachabilityDeltaComputationV0 {
    let start_node_id = start_node_id.into();
    let adjacency = streaming_ifds_node_adjacency(hyperedges);
    let reverse = streaming_ifds_reverse_adjacency(hyperedges);
    let mut reachable = previous_reachable.into_iter().collect::<BTreeSet<_>>();
    let mut dirty_nodes = BTreeSet::<String>::new();
    let mut node_visit_count = 0usize;

    for event in events {
        dirty_nodes.insert(event.node_id.clone());
        match &event.event_kind {
            StreamingIFDSEventKindV0::EdgeInsert { from, to, .. } => {
                dirty_nodes.insert(from.clone());
                dirty_nodes.insert(to.clone());
                if from == &start_node_id || reachable.contains(from) {
                    add_reachable_closure(
                        to,
                        &adjacency,
                        &mut reachable,
                        &mut dirty_nodes,
                        &mut node_visit_count,
                    );
                }
            }
            StreamingIFDSEventKindV0::EdgeDelete { from, to, .. } => {
                dirty_nodes.insert(from.clone());
                dirty_nodes.insert(to.clone());
                if from == &start_node_id || reachable.contains(from) {
                    remove_unreachable_closure(
                        to,
                        &adjacency,
                        &reverse,
                        start_node_id.as_str(),
                        &mut reachable,
                        &mut dirty_nodes,
                        &mut node_visit_count,
                    );
                }
            }
            StreamingIFDSEventKindV0::NodeDelete { id } => {
                dirty_nodes.insert(id.clone());
                remove_unreachable_closure(
                    id,
                    &adjacency,
                    &reverse,
                    start_node_id.as_str(),
                    &mut reachable,
                    &mut dirty_nodes,
                    &mut node_visit_count,
                );
            }
            StreamingIFDSEventKindV0::NodeInsert { id }
            | StreamingIFDSEventKindV0::DigestChange { id } => {
                dirty_nodes.insert(id.clone());
            }
            StreamingIFDSEventKindV0::BatchSynthesised { .. }
            | StreamingIFDSEventKindV0::RefinementContextChange { .. } => {}
        }
    }

    ReachabilityDeltaComputationV0 {
        schema_version: STREAMING_IFDS_SCHEMA_VERSION_V0,
        product: "omena-streaming-ifds.reachability-delta",
        layer_marker: STREAMING_IFDS_LAYER_MARKER_V0,
        feature_gate: STREAMING_IFDS_FEATURE_GATE_V0,
        start_node_id,
        reachable_node_ids: reachable.into_iter().collect(),
        dirty_node_ids: dirty_nodes.into_iter().collect(),
        node_visit_count,
    }
}

fn add_reachable_closure(
    root: &str,
    adjacency: &BTreeMap<String, BTreeSet<String>>,
    reachable: &mut BTreeSet<String>,
    dirty_nodes: &mut BTreeSet<String>,
    node_visit_count: &mut usize,
) {
    let mut pending = VecDeque::from([root.to_string()]);
    while let Some(current) = pending.pop_front() {
        *node_visit_count = (*node_visit_count).saturating_add(1);
        dirty_nodes.insert(current.clone());
        if !reachable.insert(current.clone()) {
            continue;
        }
        for target in adjacency.get(current.as_str()).into_iter().flatten() {
            pending.push_back(target.clone());
        }
    }
}

fn remove_unreachable_closure(
    root: &str,
    adjacency: &BTreeMap<String, BTreeSet<String>>,
    reverse: &BTreeMap<String, BTreeSet<String>>,
    start_node_id: &str,
    reachable: &mut BTreeSet<String>,
    dirty_nodes: &mut BTreeSet<String>,
    node_visit_count: &mut usize,
) {
    let mut pending = VecDeque::from([root.to_string()]);
    while let Some(current) = pending.pop_front() {
        *node_visit_count = (*node_visit_count).saturating_add(1);
        dirty_nodes.insert(current.clone());
        if has_current_reachable_predecessor(current.as_str(), reverse, start_node_id, reachable) {
            continue;
        }
        if !reachable.remove(current.as_str()) {
            continue;
        }
        for target in adjacency.get(current.as_str()).into_iter().flatten() {
            pending.push_back(target.clone());
        }
    }
}

fn has_current_reachable_predecessor(
    node_id: &str,
    reverse: &BTreeMap<String, BTreeSet<String>>,
    start_node_id: &str,
    reachable: &BTreeSet<String>,
) -> bool {
    reverse
        .get(node_id)
        .into_iter()
        .flatten()
        .any(|predecessor| predecessor == start_node_id || reachable.contains(predecessor))
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
    fn shared_scc_primitive_reproduces_the_deleted_tarjan_duplicate() {
        // SLICE-A guard-equivalence: the shared `collect_directed_graph_sccs` owner reproduces the
        // exact output the now-deleted streaming-ifds Tarjan duplicate produced — each component
        // sorted, the `a <-> b` 2-cycle collapsed, the `c` sink kept as a singleton, components in
        // Tarjan reverse-topological discovery order (sink first).
        let adjacency: BTreeMap<String, BTreeSet<String>> = BTreeMap::from([
            ("a".to_string(), BTreeSet::from(["b".to_string()])),
            (
                "b".to_string(),
                BTreeSet::from(["a".to_string(), "c".to_string()]),
            ),
            ("c".to_string(), BTreeSet::new()),
        ]);
        let sccs = collect_directed_graph_sccs(&adjacency);
        assert_eq!(
            sccs,
            vec![
                vec!["c".to_string()],
                vec!["a".to_string(), "b".to_string()],
            ]
        );
    }

    #[test]
    fn shared_reachability_loop_reproduces_the_deleted_bfs_duplicate() {
        // Guard: the shared owner reproduces the deleted BFS loop. The start is not pre-seeded into
        // the result, so it appears only when reachable from itself via a cycle (here `a` via
        // a->c->a; the sink `d` is absent).
        let adjacency: BTreeMap<String, BTreeSet<String>> = BTreeMap::from([
            (
                "a".to_string(),
                BTreeSet::from(["b".to_string(), "c".to_string()]),
            ),
            ("b".to_string(), BTreeSet::from(["d".to_string()])),
            ("c".to_string(), BTreeSet::from(["a".to_string()])),
            ("d".to_string(), BTreeSet::new()),
        ]);
        assert_eq!(
            collect_reachable_node_ids("a", &adjacency),
            vec![
                "a".to_string(),
                "b".to_string(),
                "c".to_string(),
                "d".to_string(),
            ]
        );
        assert_eq!(
            collect_reachable_node_ids("d", &adjacency),
            Vec::<String>::new()
        );
    }

    #[test]
    fn shared_cycle_enumerator_reproduces_the_elementary_circuit_set() {
        use omena_cross_file_summary::{
            collect_directed_graph_cycles, collect_directed_graph_cycles_with_work_cap,
        };
        let ring = |xs: &[&str]| xs.iter().map(|x| x.to_string()).collect::<Vec<_>>();
        let graph = |edges: &[(&str, &[&str])]| {
            edges
                .iter()
                .map(|(node, targets)| {
                    (
                        node.to_string(),
                        targets
                            .iter()
                            .map(|t| t.to_string())
                            .collect::<BTreeSet<_>>(),
                    )
                })
                .collect::<BTreeMap<_, _>>()
        };
        // chord {a,b,c} (a->b, a->c, b->c, c->a): two elementary circuits, canonical closed rings.
        let chord = graph(&[("a", &["b", "c"]), ("b", &["c"]), ("c", &["a"])]);
        assert_eq!(
            collect_directed_graph_cycles(&chord),
            vec![ring(&["a", "b", "c", "a"]), ring(&["a", "c", "a"])]
        );
        // complete K4: all 20 elementary circuits in the SCC (len 2:6, len 3:8, len 4:6); the
        // consumer later filters to the 15 that contain a given target.
        let k4 = graph(&[
            ("a", &["b", "c", "d"]),
            ("b", &["a", "c", "d"]),
            ("c", &["a", "b", "d"]),
            ("d", &["a", "b", "c"]),
        ]);
        assert_eq!(collect_directed_graph_cycles(&k4).len(), 20);
        // self-loop singleton is non-trivial; acyclic input short-circuits to no cycles.
        assert_eq!(
            collect_directed_graph_cycles(&graph(&[("a", &["a"])])),
            vec![ring(&["a", "a"])]
        );
        assert!(collect_directed_graph_cycles(&graph(&[("a", &["b"]), ("b", &[])])).is_empty());
        // fail-soft: a tiny work cap degrades the dense K4 SCC to ONE representative (witnessed).
        assert_eq!(collect_directed_graph_cycles_with_work_cap(&k4, 4).len(), 1);
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

    #[test]
    fn streaming_ifds_propagates_facts_and_matches_batch_precision() {
        let hyperedges = vec![
            hyperedge("edge-a-b", "a", "b"),
            hyperedge("edge-b-c", "b", "c"),
        ];
        let events = vec![streaming_ifds_event_input_v0(
            "event-a",
            7,
            "a",
            AbstractClassValueV0::Exact {
                value: "button".to_string(),
            },
            None,
        )];

        let report = run_streaming_ifds_exact_v0(
            "update-1",
            "a",
            &hyperedges,
            &events,
            &PolylogDynamicConnectivityBackendV0::default(),
            None,
        );

        assert_eq!(report.schema_version, "0");
        assert_eq!(report.event_count, 1);
        assert_eq!(report.input_fact_count, 1);
        assert_eq!(report.output_fact_count, 3);
        assert_eq!(report.transfer_function_count, 2);
        assert_eq!(
            report.witness.product,
            "omena-streaming-ifds.exact-connectivity-witness"
        );
        assert_eq!(
            report.witness.connectivity_algorithm,
            "exactBfsReachability"
        );
        assert_eq!(
            report.witness.polylog_bound_scope,
            "targetOnlyNotAsymptoticEvidence"
        );
        assert_eq!(report.dirty_fact_count, 3);
        assert_eq!(report.reused_fact_count, 0);
        assert!(!report.fallback_to_batch);
        assert!(report.precision_parity_with_batch);
        assert_eq!(
            report
                .output_facts
                .iter()
                .map(|fact| fact.node_id.as_str())
                .collect::<Vec<_>>(),
            vec!["a", "b", "c"]
        );
        assert_eq!(
            report
                .output_facts
                .iter()
                .map(|fact| abstract_class_value_key(&fact.value))
                .collect::<Vec<_>>(),
            vec![
                "exact:button".to_string(),
                "finiteSet:b,button".to_string(),
                "finiteSet:b,button,c".to_string()
            ]
        );
        assert!(
            streaming_ifds_transfer_functions_v0(&hyperedges)
                .iter()
                .all(|transfer| transfer.transfer_kind == "composeClassSet")
        );
    }

    #[test]
    fn run_local_fact_interning_preserves_public_facts_and_counters() {
        let hyperedges = vec![
            hyperedge("edge-a-b", "a", "b"),
            hyperedge("edge-b-c", "b", "c"),
            hyperedge_with_kind("edge-a-d", "a", "d", UnifiedHypergraphEdgeKindV0::Value),
            hyperedge_with_kind("edge-d-e", "d", "e", UnifiedHypergraphEdgeKindV0::Value),
        ];
        let transfer_table = streaming_ifds_transfer_table_v0(&hyperedges);
        let events = vec![streaming_ifds_event_input_v0(
            "event-a",
            7,
            "a",
            AbstractClassValueV0::Exact {
                value: "button".to_string(),
            },
            None,
        )];

        let (interned_facts, interned_stats) =
            propagate_ifds_facts_with_table_and_stats(&transfer_table, &events);
        let (string_facts, string_stats) =
            propagate_ifds_facts_with_table_string_dedup_for_test(&transfer_table, &events);

        assert_eq!(interned_facts, string_facts);
        assert_eq!(interned_stats, string_stats);
        assert!(
            fact_keys(&interned_facts)
                .iter()
                .any(|key| key.contains("finiteSet:b,button")),
            "fixture must exercise value-carrying fact keys"
        );

        let reachable = vec![
            "a".to_string(),
            "b".to_string(),
            "c".to_string(),
            "d".to_string(),
            "e".to_string(),
        ];
        let interned_entry = streaming_ifds_summary_cache_entry_v0(
            "a",
            reachable.clone(),
            fact_keys(&interned_facts),
            false,
        );
        let string_entry =
            streaming_ifds_summary_cache_entry_v0("a", reachable, fact_keys(&string_facts), false);
        assert_eq!(interned_entry, string_entry);
        assert_eq!(
            json_string(&interned_entry),
            json_string(&string_entry),
            "public summary serialization must stay stable across the internal representation"
        );
    }

    #[test]
    fn run_local_fact_interning_restarts_for_each_propagation() {
        let hyperedges = vec![hyperedge("edge-a-b", "a", "b")];
        let transfer_table = streaming_ifds_transfer_table_v0(&hyperedges);
        let events = vec![streaming_ifds_event_input_v0(
            "event-a",
            1,
            "a",
            AbstractClassValueV0::Exact {
                value: "button".to_string(),
            },
            None,
        )];

        let mut left_table = StreamingIFDSRunInternTableV0::from_inputs(&transfer_table, &events);
        let left_key = left_table.intern_fact_key(
            "a",
            &AbstractClassValueV0::Exact {
                value: "button".to_string(),
            },
        );
        let mut second_table = StreamingIFDSRunInternTableV0::from_inputs(&transfer_table, &events);
        let second_key = second_table.intern_fact_key(
            "a",
            &AbstractClassValueV0::Exact {
                value: "button".to_string(),
            },
        );

        assert_eq!(left_key.0, second_key.0);
        assert_eq!(left_table.fact_key_value(left_key), "a|exact:button");
        assert_eq!(second_table.fact_key_value(second_key), "a|exact:button");
    }

    #[test]
    fn summary_cache_entry_keeps_string_fact_key_surface() {
        let entry = streaming_ifds_summary_cache_entry_v0(
            "a",
            vec!["a".to_string(), "b".to_string()],
            vec![
                "a|exact:button".to_string(),
                "b|finiteSet:b,button".to_string(),
            ],
            false,
        );
        let value = json_value(&entry);
        let Some(object) = value.as_object() else {
            assert!(
                value.is_object(),
                "summary cache entry should serialize as a JSON object"
            );
            return;
        };
        let keys = object.keys().cloned().collect::<BTreeSet<_>>();
        assert_eq!(
            keys,
            [
                "factKeys",
                "featureGate",
                "layerMarker",
                "product",
                "reachableNodeIds",
                "reusedFromPrevious",
                "schemaVersion",
                "startNodeId",
                "summaryHash",
            ]
            .into_iter()
            .map(str::to_string)
            .collect::<BTreeSet<_>>()
        );
        assert!(
            object
                .get("factKeys")
                .and_then(serde_json::Value::as_array)
                .into_iter()
                .flatten()
                .all(serde_json::Value::is_string),
            "summary cache fact keys must remain canonical strings"
        );
        assert!(
            object.keys().all(|key| {
                let normalized = key.to_ascii_lowercase();
                !normalized.contains("intern") && !normalized.contains("factid")
            }),
            "run-local numeric keys must not be exposed on the summary cache surface"
        );
    }

    #[test]
    fn demand_report_matches_projected_batch_fact_keys() {
        let hyperedges = vec![
            hyperedge("edge-a-b", "a", "b"),
            hyperedge("edge-b-c", "b", "c"),
            hyperedge("edge-c-d", "c", "d"),
        ];
        let events = vec![streaming_ifds_event_input_v0(
            "event-a",
            7,
            "a",
            AbstractClassValueV0::Exact {
                value: "button".to_string(),
            },
            None,
        )];
        let starts = vec!["a".to_string()];
        let targets = vec!["c".to_string()];

        let demand = run_streaming_ifds_demand_v0(&starts, &targets, &hyperedges, &events);
        let projected_nodes = demand
            .projection_node_ids
            .iter()
            .map(String::as_str)
            .collect::<BTreeSet<_>>();
        let batch_fact_keys = omena_streaming_ifds_batch_fact_keys_v0(&hyperedges, &events)
            .into_iter()
            .filter(|key| {
                key.rsplit_once('|')
                    .is_some_and(|(node, _)| projected_nodes.contains(node))
            })
            .collect::<Vec<_>>();

        assert!(demand.strict_subset_of_forward_reachable_nodes);
        assert_eq!(demand.projection_node_ids, vec!["a", "b", "c"]);
        assert_eq!(demand.fact_keys, batch_fact_keys);
        assert_eq!(demand.transfer_visit_count, 2);
        assert_eq!(demand.slice_scc_count, 3);
        assert!(
            demand
                .fact_keys
                .iter()
                .any(|key| key == "c|finiteSet:b,button,c")
        );
    }

    #[test]
    fn fact_key_route_keeps_batch_until_relocation_gate_is_green() {
        let query = streaming_ifds_fact_key_route_v0(&["target".to_string()]);
        let enabled = streaming_ifds_fact_key_route_with_gate_v0(&["target".to_string()], true);
        let workspace = streaming_ifds_fact_key_route_v0(&[]);

        assert_eq!(query.request_scope, "queryShaped");
        assert_eq!(query.fact_key_engine, "batch");
        assert!(!query.relocation_gate_green);
        assert_eq!(enabled.request_scope, "queryShaped");
        assert_eq!(enabled.fact_key_engine, "demand");
        assert!(enabled.relocation_gate_green);
        assert_eq!(workspace.request_scope, "workspaceWide");
        assert_eq!(workspace.fact_key_engine, "batch");
    }

    #[test]
    fn settle_equal_report_records_repeated_demand_batch_agreement() {
        let hyperedges = vec![
            hyperedge("edge-a-b", "a", "b"),
            hyperedge("edge-b-c", "b", "c"),
        ];
        let events = vec![streaming_ifds_event_input_v0(
            "event-a",
            7,
            "a",
            AbstractClassValueV0::Exact {
                value: "button".to_string(),
            },
            None,
        )];
        let starts = vec!["a".to_string()];
        let targets = vec!["c".to_string()];

        let report = run_streaming_ifds_settle_equal_v0(&starts, &targets, &hyperedges, &events, 3);

        assert_eq!(report.requested_settle_count, 3);
        assert_eq!(report.equal_settle_count, 3);
        assert_eq!(report.divergence_count, 0);
        assert!(report.all_settles_equal);
        assert!(report.demand_primary_ready);
    }

    #[test]
    fn demand_readiness_requires_every_precondition() {
        let settle_report = StreamingIFDSSettleEqualReportV0 {
            schema_version: STREAMING_IFDS_SCHEMA_VERSION_V0,
            product: "omena-streaming-ifds.settle-equal-report",
            layer_marker: STREAMING_IFDS_LAYER_MARKER_V0,
            feature_gate: STREAMING_IFDS_FEATURE_GATE_V0,
            requested_settle_count: 3,
            equal_settle_count: 3,
            divergence_count: 0,
            all_settles_equal: true,
            demand_primary_ready: true,
        };
        let ready = streaming_ifds_demand_readiness_v0(StreamingIFDSDemandReadinessInputV0 {
            fact_key_gate_green: true,
            deletion_corpus_green: true,
            complexity_slope_green: true,
            settle_report: settle_report.clone(),
        });

        assert_eq!(
            ready.product,
            "omena-streaming-ifds.demand-readiness-report"
        );
        assert_eq!(ready.precondition_count, 4);
        assert_eq!(ready.green_precondition_count, 4);
        assert!(ready.demand_primary_ready);

        for missing in [
            (false, true, true, settle_report.clone()),
            (true, false, true, settle_report.clone()),
            (true, true, false, settle_report.clone()),
            (
                true,
                true,
                true,
                StreamingIFDSSettleEqualReportV0 {
                    equal_settle_count: 2,
                    divergence_count: 1,
                    all_settles_equal: false,
                    demand_primary_ready: false,
                    ..settle_report.clone()
                },
            ),
        ] {
            let report = streaming_ifds_demand_readiness_v0(StreamingIFDSDemandReadinessInputV0 {
                fact_key_gate_green: missing.0,
                deletion_corpus_green: missing.1,
                complexity_slope_green: missing.2,
                settle_report: missing.3,
            });
            assert!(!report.demand_primary_ready);
            assert_eq!(report.green_precondition_count, 3);
        }
    }

    #[test]
    fn streaming_ifds_summary_cache_records_reused_facts() {
        let hyperedges = vec![hyperedge("edge-a-b", "a", "b")];
        let events = vec![streaming_ifds_event_input_v0(
            "event-a",
            11,
            "a",
            AbstractClassValueV0::FiniteSet {
                values: vec!["button".to_string(), "card".to_string()],
            },
            Some(99),
        )];

        let first = run_streaming_ifds_exact_v0(
            "update-1",
            "a",
            &hyperedges,
            &events,
            &ExactStreamingConnectivityOracleV0::default(),
            None,
        );
        let second = run_streaming_ifds_exact_v0(
            "update-2",
            "a",
            &hyperedges,
            &events,
            &ExactStreamingConnectivityOracleV0::default(),
            Some(&first.summary_cache),
        );

        assert_eq!(second.reused_fact_count, first.output_fact_count);
        assert_eq!(second.dirty_fact_count, 0);
        assert!(second.summary_cache[0].reused_from_previous);
        assert!(second.update.refinement_context_digest.is_some());
    }

    #[test]
    fn incremental_parity_holds_when_prior_cache_is_consistent() {
        // Old revision: a -> b -> c, fact seeded at a flows to {a, b, c}.
        let old_graph = vec![
            hyperedge("edge-a-b", "a", "b"),
            hyperedge("edge-b-c", "b", "c"),
        ];
        let value = AbstractClassValueV0::Exact {
            value: "button".to_string(),
        };
        let seed = vec![streaming_ifds_event_input_v0(
            "event-a",
            1,
            "a",
            value.clone(),
            None,
        )];
        let first = run_streaming_ifds_exact_v0(
            "update-1",
            "a",
            &old_graph,
            &seed,
            &ExactStreamingConnectivityOracleV0::default(),
            None,
        );

        // New revision: same graph, re-seed a. The prior cache is consistent with
        // the current graph, so the incremental reuse of the {c} fact agrees with
        // the batch recompute and parity holds.
        let next = vec![streaming_ifds_event_input_v0(
            "event-a2", 2, "a", value, None,
        )];
        let report = run_streaming_ifds_exact_v0(
            "update-2",
            "a",
            &old_graph,
            &next,
            &ExactStreamingConnectivityOracleV0::default(),
            Some(&first.summary_cache),
        );

        assert!(report.precision_parity_with_batch);
        assert_eq!(report_node_ids(&report), vec!["a", "b", "c"]);
    }

    #[test]
    fn reachability_dirty_profile_measures_changed_hyperedge_frontier() {
        let previous = vec![
            hyperedge_with_kind(
                "edge-entry-theme",
                "styleModule|/workspace/App.module.scss|root",
                "styleSymbol|/workspace/theme.module.scss|theme",
                UnifiedHypergraphEdgeKindV0::SassUse,
            ),
            hyperedge_with_kind(
                "edge-theme-token",
                "styleSymbol|/workspace/theme.module.scss|theme",
                "styleSymbol|/workspace/tokens.module.scss|token",
                UnifiedHypergraphEdgeKindV0::SassForward,
            ),
            hyperedge_with_kind(
                "edge-token-button",
                "styleSymbol|/workspace/tokens.module.scss|token",
                "styleSymbol|/workspace/Button.module.scss|button",
                UnifiedHypergraphEdgeKindV0::ComposesExternal,
            ),
        ];
        let current = previous
            .iter()
            .cloned()
            .chain([hyperedge_with_kind(
                "edge-theme-color",
                "styleSymbol|/workspace/theme.module.scss|theme",
                "styleSymbol|/workspace/colors.module.scss|color",
                UnifiedHypergraphEdgeKindV0::SassForward,
            )])
            .collect::<Vec<_>>();

        let profile = summarize_reachability_dirty_set_profile_v0(
            "styleModule|/workspace/App.module.scss|root",
            &previous,
            &current,
            &ExactStreamingConnectivityOracleV0::default(),
        );

        assert_eq!(profile.full_node_count, 4);
        assert_eq!(
            profile.dirty_node_ids,
            vec![
                "styleSymbol|/workspace/colors.module.scss|color".to_string(),
                "styleSymbol|/workspace/theme.module.scss|theme".to_string()
            ]
        );
        assert!(profile.dirty_ratio > 0.0);
        assert!(profile.dirty_ratio < 1.0);
        assert!(profile.incremental_candidate);
    }

    #[test]
    fn reachability_dirty_profile_flags_full_frontier_edits() {
        let previous = clique_reachability_hyperedges("previous", 4);
        let current = clique_reachability_hyperedges("current", 4);
        let profile = summarize_reachability_dirty_set_profile_v0(
            "styleModule|/workspace/clique-0.module.scss|root",
            &previous,
            &current,
            &ExactStreamingConnectivityOracleV0::default(),
        );

        assert_eq!(profile.full_node_count, 4);
        assert_eq!(profile.dirty_node_count, profile.full_node_count);
        assert_eq!(profile.dirty_ratio, 1.0);
        assert!(!profile.incremental_candidate);
    }

    #[test]
    fn reachability_delta_retracts_deleted_edge_without_stale_node() {
        let current = vec![hyperedge("edge-a-b", "a", "b")];
        let events = vec![edge_delete_event("event-b-c-delete", 2, "b", "c")];

        let delta = incremental_reachable_node_ids_zset(
            "a",
            &current,
            &events,
            ["b".to_string(), "c".to_string()],
        );
        let batch = ExactStreamingConnectivityOracleV0::default().reachable_node_ids("a", &current);

        assert_eq!(delta.reachable_node_ids, vec!["b".to_string()]);
        assert_eq!(delta.reachable_node_ids, batch);
        assert!(delta.dirty_node_ids.contains(&"c".to_string()));
        assert!(delta.node_visit_count > 0);
    }

    #[test]
    fn reachability_delta_extends_inserted_edge_from_cached_reachable_set() {
        let current = vec![
            hyperedge("edge-a-b", "a", "b"),
            hyperedge("edge-b-c", "b", "c"),
        ];
        let events = vec![edge_insert_event("event-b-c-insert", 2, "b", "c")];

        let delta = incremental_reachable_node_ids_zset("a", &current, &events, ["b".to_string()]);
        let batch = ExactStreamingConnectivityOracleV0::default().reachable_node_ids("a", &current);

        assert_eq!(
            delta.reachable_node_ids,
            vec!["b".to_string(), "c".to_string()]
        );
        assert_eq!(delta.reachable_node_ids, batch);
    }

    #[test]
    fn incremental_parity_diverges_when_a_reused_prior_fact_is_stale() {
        // Old revision: a -> b -> c. Fact seeded at a reaches {a, b, c}; cache it.
        let old_graph = vec![
            hyperedge("edge-a-b", "a", "b"),
            hyperedge("edge-b-c", "b", "c"),
        ];
        let value = AbstractClassValueV0::Exact {
            value: "button".to_string(),
        };
        let seed = vec![streaming_ifds_event_input_v0(
            "event-a",
            1,
            "a",
            value.clone(),
            None,
        )];
        let first = run_streaming_ifds_exact_v0(
            "update-1",
            "a",
            &old_graph,
            &seed,
            &ExactStreamingConnectivityOracleV0::default(),
            None,
        );
        assert!(first.precision_parity_with_batch);

        // New revision: the b -> c edge is removed (edge deletion). Re-seeding a
        // makes the dirty region {a, b}; c is no longer reachable. The batch
        // oracle recomputes {a, b}. The incremental path reuses the prior {c}
        // fact because c lies outside the dirty region and was never invalidated,
        // so the incremental set is {a, b, c}. The two distinct computations
        // disagree and parity is false.
        let new_graph = vec![hyperedge("edge-a-b", "a", "b")];
        let next = vec![streaming_ifds_event_input_v0(
            "event-a2", 2, "a", value, None,
        )];
        let report = run_streaming_ifds_exact_v0(
            "update-2",
            "a",
            &new_graph,
            &next,
            &ExactStreamingConnectivityOracleV0::default(),
            Some(&first.summary_cache),
        );

        assert!(!report.precision_parity_with_batch);
        assert!(!report.reachability_parity_with_batch);
        assert!(!report.reachability_delta_used);
        assert!(report.fallback_to_batch);
        assert_eq!(report.witness.reachable_node_ids, vec!["b".to_string()]);
        assert_eq!(
            report.summary_cache[0].reachable_node_ids,
            vec!["b".to_string()]
        );
        // Batch (ground truth over the current graph) drops c; the stale reused
        // fact keeps it in the incremental set.
        assert_eq!(report_node_ids(&report), vec!["a", "b", "c"]);
        assert_eq!(
            fact_keys(&propagate_ifds_facts(&new_graph, &next)),
            vec![
                "a|exact:button".to_string(),
                "b|finiteSet:b,button".to_string()
            ]
        );
        assert!(report.output_facts.iter().any(|fact| {
            fact.node_id == "c" && abstract_class_value_key(&fact.value) == "finiteSet:b,button,c"
        }));
    }

    #[test]
    fn warm_path_uses_reachability_delta_when_edge_deletion_parity_holds() {
        let old_graph = vec![
            hyperedge("edge-a-b", "a", "b"),
            hyperedge("edge-b-c", "b", "c"),
        ];
        let seed = vec![streaming_ifds_event_input_v0(
            "event-a",
            1,
            "a",
            AbstractClassValueV0::Exact {
                value: "button".to_string(),
            },
            None,
        )];
        let first = run_streaming_ifds_exact_v0(
            "update-1",
            "a",
            &old_graph,
            &seed,
            &ExactStreamingConnectivityOracleV0::default(),
            None,
        );
        let current_graph = vec![hyperedge("edge-a-b", "a", "b")];
        let events = vec![edge_delete_event("event-b-c-delete", 2, "b", "c")];

        let report = run_streaming_ifds_exact_v0(
            "update-2",
            "a",
            &current_graph,
            &events,
            &ExactStreamingConnectivityOracleV0::default(),
            Some(&first.summary_cache),
        );

        assert!(report.reachability_parity_with_batch);
        assert!(report.reachability_delta_used);
        assert!(!report.fallback_to_batch);
        assert_eq!(report.witness.reachable_node_ids, vec!["b".to_string()]);
        assert_eq!(
            report.summary_cache[0].reachable_node_ids,
            vec!["b".to_string()]
        );
        assert!(report.reachability_work_node_visits < report.batch_reachability_work_node_visits);
    }

    #[test]
    fn cross_file_reachability_report_preserves_public_contract() {
        let hyperedges = vec![hyperedge(
            "edge-button-base",
            "styleModule|/workspace/Button.module.scss|root",
            "styleSymbol|/workspace/base.module.scss|base",
        )];
        let report = summarize_streaming_ifds_cross_file_reachability_v0(
            "/workspace/Button.module.scss",
            &hyperedges,
        );

        assert_eq!(
            report.product,
            "omena-streaming-ifds.cross-file-reachability-report"
        );
        assert_eq!(report.start_node_count, 1);
        assert_eq!(report.analysis_report_count, 1);
        assert!(report.precision_parity_with_batch);
        assert!(report.exact_default);
        assert_eq!(
            report.reachable_foreign_paths,
            vec!["/workspace/base.module.scss".to_string()]
        );
    }

    #[test]
    fn cross_file_reachability_report_clears_for_self_contained_graph() {
        let hyperedges = vec![hyperedge(
            "edge-self",
            "styleModule|/workspace/Button.module.scss|root",
            "styleSymbol|/workspace/Button.module.scss|base",
        )];
        let report = summarize_streaming_ifds_cross_file_reachability_v0(
            "/workspace/Button.module.scss",
            &hyperedges,
        );

        assert_eq!(report.start_node_count, 2);
        assert_eq!(report.reachable_foreign_path_count, 0);
        assert!(report.reachable_foreign_paths.is_empty());
        assert!(report.precision_parity_with_batch);
    }

    #[test]
    fn cross_file_reachability_fast_path_matches_ifds_oracle_for_mixed_edges() {
        let target_style_path = "/workspace/Button.module.scss";
        let hyperedges = vec![
            hyperedge_with_kind(
                "edge-root-theme",
                "styleModule|/workspace/Button.module.scss|root",
                "styleSymbol|/workspace/theme.module.scss|theme",
                UnifiedHypergraphEdgeKindV0::SassForward,
            ),
            hyperedge_with_kind(
                "edge-theme-token",
                "styleSymbol|/workspace/theme.module.scss|theme",
                "styleSymbol|/workspace/tokens.module.scss|token",
                UnifiedHypergraphEdgeKindV0::Value,
            ),
            hyperedge_with_kind(
                "edge-token-button",
                "styleSymbol|/workspace/tokens.module.scss|token",
                "styleModule|/workspace/Button.module.scss|root",
                UnifiedHypergraphEdgeKindV0::Icss,
            ),
            hyperedge_with_kind(
                "edge-theme-card",
                "styleSymbol|/workspace/theme.module.scss|theme",
                "styleSymbol|/workspace/Card.module.scss|card",
                UnifiedHypergraphEdgeKindV0::ComposesExternal,
            ),
        ];

        let fast =
            summarize_streaming_ifds_cross_file_reachability_v0(target_style_path, &hyperedges);
        let oracle = summarize_streaming_ifds_cross_file_reachability_oracle_v0(
            target_style_path,
            &hyperedges,
        );

        assert_eq!(fast.reachable_foreign_paths, oracle.reachable_foreign_paths);
        assert_eq!(
            fast.reachable_foreign_path_count,
            oracle.reachable_foreign_path_count
        );
        assert_eq!(fast.start_node_count, oracle.start_node_count);
        assert!(
            fast.precision_parity_with_batch,
            "fast reachability must retain exact default semantics"
        );
    }

    #[test]
    fn cross_file_reachability_fast_path_growth_stays_near_linear() {
        let samples = [4usize, 8, 16]
            .into_iter()
            .map(|layer_count| {
                let hyperedges = layered_reachability_hyperedges(layer_count);
                let summary = summarize_streaming_ifds_cross_file_reachability_fast_v0(
                    "/workspace/Entry.module.scss",
                    &hyperedges,
                );
                assert!(
                    summary.report.reachable_foreign_path_count >= layer_count,
                    "fixture must keep cross-file reachability live: {summary:?}"
                );
                (hyperedges.len(), summary.traversal_step_count)
            })
            .collect::<Vec<_>>();
        let exponent = fit_growth_exponent(samples.as_slice());
        assert!(
            exponent <= 1.2,
            "cross-file reachability traversal should scale near-linearly; exponent={exponent:.3}, samples={samples:?}"
        );
    }

    #[test]
    fn streaming_ifds_solver_transfer_index_growth_stays_near_linear() {
        let samples = [8usize, 16, 32]
            .into_iter()
            .map(|layer_count| {
                let hyperedges = layered_identity_ifds_hyperedges(layer_count);
                let transfer_table = streaming_ifds_transfer_table_v0(&hyperedges);
                let events = vec![streaming_ifds_event_input_v0(
                    "event-entry",
                    1,
                    "entry",
                    AbstractClassValueV0::Exact {
                        value: "button".to_string(),
                    },
                    None,
                )];
                let (facts, stats) =
                    propagate_ifds_facts_with_table_and_stats(&transfer_table, &events);
                assert!(
                    facts.len() >= layer_count.saturating_mul(2),
                    "fixture must keep IFDS propagation live: layer_count={layer_count}, facts={facts:?}"
                );
                assert_eq!(
                    stats.transfer_visit_count,
                    hyperedges.len(),
                    "indexed dispatch should visit each reachable transfer once"
                );
                (
                    hyperedges.len().saturating_add(facts.len()),
                    stats
                        .transfer_visit_count
                        .saturating_add(stats.popped_fact_count),
                )
            })
            .collect::<Vec<_>>();
        let exponent = fit_growth_exponent(samples.as_slice());
        assert!(
            exponent <= 1.2,
            "IFDS solver dispatch should scale near-linearly; exponent={exponent:.3}, samples={samples:?}"
        );
    }

    #[test]
    fn streaming_ifds_solver_hygiene_policy_keeps_cache_feedback_and_reference_values_explicit() {
        let policy = streaming_ifds_solver_hygiene_policy_v0();
        assert_eq!(
            policy.summary_cache_feedback_policy,
            "emitEvidenceCacheButDoNotFeedProductPaths"
        );
        assert_eq!(
            policy.non_product_cache_feedback_scope,
            "engineShadowRunnerPrecisionEvidenceOnly"
        );
        assert_eq!(
            policy.cache_feedback_activation,
            "requiresNonCountConsumerAndPrecisionParityFallback"
        );
        assert_eq!(
            policy.reference_edge_value_policy,
            "identityOnReferenceAndAliasEdges"
        );
        assert_eq!(
            policy.concrete_value_owner,
            "omena-sif.variable-export.value-repr"
        );
        assert_eq!(
            apply_streaming_ifds_transfer(
                &StreamingIFDSTransferFunctionV0 {
                    schema_version: STREAMING_IFDS_SCHEMA_VERSION_V0,
                    product: "test.transfer",
                    layer_marker: STREAMING_IFDS_LAYER_MARKER_V0,
                    feature_gate: STREAMING_IFDS_FEATURE_GATE_V0,
                    hyperedge_id: "edge-sass-forward".to_string(),
                    edge_kind: UnifiedHypergraphEdgeKindV0::SassForward,
                    tail_node_ids: vec!["a".to_string()],
                    head_node_id: "b".to_string(),
                    transfer_kind: "semanticReferencePreserving",
                },
                &AbstractClassValueV0::Exact {
                    value: "token".to_string()
                },
            ),
            AbstractClassValueV0::Exact {
                value: "token".to_string()
            }
        );
        assert_eq!(
            streaming_ifds_transfer_kind(UnifiedHypergraphEdgeKindV0::LessImport),
            "semanticReferencePreserving"
        );
        assert_eq!(
            streaming_ifds_transfer_kind(UnifiedHypergraphEdgeKindV0::LessModuleGraphClosure),
            "semanticReferencePreserving"
        );
        assert_eq!(
            apply_streaming_ifds_transfer(
                &StreamingIFDSTransferFunctionV0 {
                    schema_version: STREAMING_IFDS_SCHEMA_VERSION_V0,
                    product: "test.transfer",
                    layer_marker: STREAMING_IFDS_LAYER_MARKER_V0,
                    feature_gate: STREAMING_IFDS_FEATURE_GATE_V0,
                    hyperedge_id: "edge-less-import".to_string(),
                    edge_kind: UnifiedHypergraphEdgeKindV0::LessImport,
                    tail_node_ids: vec!["a".to_string()],
                    head_node_id: "b".to_string(),
                    transfer_kind: "semanticReferencePreserving",
                },
                &AbstractClassValueV0::Exact {
                    value: "token".to_string()
                },
            ),
            AbstractClassValueV0::Exact {
                value: "token".to_string()
            }
        );
    }

    #[cfg(feature = "with-frame-rule")]
    #[test]
    fn frame_rule_bridge_policy_is_feature_gated() {
        let policy = streaming_ifds_frame_rule_bridge_policy_v0();
        assert_eq!(policy.schema_version, "0");
        assert_eq!(policy.feature_gate, "with-frame-rule");
        assert_eq!(policy.coarse_policy, "frameFootprintReachability");
    }

    fn report_node_ids(report: &StreamingIFDSAnalysisReportV0) -> Vec<&str> {
        report
            .output_facts
            .iter()
            .map(|fact| fact.node_id.as_str())
            .collect()
    }

    #[test]
    fn condensation_reachability_matches_batch_summarize() {
        // rfcs#111 C1 slice parity gate: the shared-condensation arm must be
        // byte-identical to the per-call batch arm for every target path.
        let hyperedges = vec![
            hyperedge("e1", "sel|a.scss|x", "sel|b.scss|y"),
            hyperedge("e2", "sel|b.scss|y", "sel|c.scss|z"),
            hyperedge("e3", "sel|c.scss|z", "sel|b.scss|y"),
            hyperedge("e4", "sel|d.scss|w", "sel|d.scss|w2"),
        ];
        let condensation = streaming_ifds_reachability_condensation_v0(&hyperedges);
        for target in ["a.scss", "b.scss", "c.scss", "d.scss", "unknown.scss"] {
            let batch = summarize_streaming_ifds_cross_file_reachability_v0(target, &hyperedges);
            let shared = summarize_streaming_ifds_cross_file_reachability_with_condensation_v0(
                target,
                &condensation,
            );
            assert_eq!(shared, batch, "condensation arm diverged for {target}");
        }
    }

    fn propagate_ifds_facts_with_table_string_dedup_for_test(
        transfer_table: &StreamingIFDSTransferTableV0,
        events: &[StreamingIfdsEventInputV0],
    ) -> (Vec<StreamingIFDSFactV0>, StreamingIFDSPropagationStatsV0) {
        let mut seen = BTreeSet::<String>::new();
        let mut pending = VecDeque::<StreamingIFDSFactV0>::new();
        let mut output = Vec::<StreamingIFDSFactV0>::new();
        let mut stats = StreamingIFDSPropagationStatsV0::default();

        for event in events {
            let fact = streaming_ifds_fact_v0(
                event.node_id.clone(),
                event.value.clone(),
                vec![format!("event:{}", event.event_id)],
            );
            if seen.insert(fact_key(&fact.node_id, &fact.value)) {
                pending.push_back(fact.clone());
                output.push(fact);
            }
        }

        while let Some(fact) = pending.pop_front() {
            stats.popped_fact_count = stats.popped_fact_count.saturating_add(1);
            for transfer in transfer_table.transfers_for_tail(&fact.node_id) {
                stats.transfer_visit_count = stats.transfer_visit_count.saturating_add(1);
                let mut provenance = fact.provenance.clone();
                provenance.push(format!("transfer:{}", transfer.hyperedge_id));
                let next_value = apply_streaming_ifds_transfer(transfer, &fact.value);
                let next_fact =
                    streaming_ifds_fact_v0(transfer.head_node_id.clone(), next_value, provenance);
                if seen.insert(fact_key(&next_fact.node_id, &next_fact.value)) {
                    pending.push_back(next_fact.clone());
                    output.push(next_fact);
                }
            }
        }

        output.sort_by(|left, right| {
            left.node_id
                .cmp(&right.node_id)
                .then(left.fact_id.cmp(&right.fact_id))
        });
        (output, stats)
    }

    fn json_value(value: &impl Serialize) -> serde_json::Value {
        match serde_json::to_value(value) {
            Ok(value) => value,
            Err(error) => {
                assert!(
                    error.to_string().is_empty(),
                    "value should serialize to JSON: {error}"
                );
                serde_json::Value::Null
            }
        }
    }

    fn json_string(value: &impl Serialize) -> String {
        match serde_json::to_string(value) {
            Ok(value) => value,
            Err(error) => {
                assert!(
                    error.to_string().is_empty(),
                    "value should serialize to JSON string: {error}"
                );
                String::new()
            }
        }
    }

    fn hyperedge(id: &str, from: &str, to: &str) -> UnifiedHypergraphHyperedgeV0 {
        hyperedge_with_kind(id, from, to, UnifiedHypergraphEdgeKindV0::ComposesLocal)
    }

    fn hyperedge_with_kind(
        id: &str,
        from: &str,
        to: &str,
        edge_kind: UnifiedHypergraphEdgeKindV0,
    ) -> UnifiedHypergraphHyperedgeV0 {
        let source_edge_kind = edge_kind.as_wire_label();
        UnifiedHypergraphHyperedgeV0 {
            schema_version: "0",
            product: "test.hyperedge",
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

    fn edge_insert_event(
        id: &str,
        revision: u64,
        from: &str,
        to: &str,
    ) -> StreamingIfdsEventInputV0 {
        edge_change_event(
            id,
            revision,
            from,
            to,
            StreamingIFDSEventKindV0::EdgeInsert {
                from: from.to_string(),
                to: to.to_string(),
                edge_kind: "composesLocal",
            },
        )
    }

    fn edge_delete_event(
        id: &str,
        revision: u64,
        from: &str,
        to: &str,
    ) -> StreamingIfdsEventInputV0 {
        edge_change_event(
            id,
            revision,
            from,
            to,
            StreamingIFDSEventKindV0::EdgeDelete {
                from: from.to_string(),
                to: to.to_string(),
                edge_kind: "composesLocal",
            },
        )
    }

    fn edge_change_event(
        id: &str,
        revision: u64,
        from: &str,
        to: &str,
        event_kind: StreamingIFDSEventKindV0,
    ) -> StreamingIfdsEventInputV0 {
        StreamingIfdsEventInputV0 {
            schema_version: STREAMING_IFDS_SCHEMA_VERSION_V0,
            product: "test.event-input",
            layer_marker: STREAMING_IFDS_LAYER_MARKER_V0,
            feature_gate: STREAMING_IFDS_FEATURE_GATE_V0,
            event_id: id.to_string(),
            revision,
            event_kind,
            node_id: from.to_string(),
            value: AbstractClassValueV0::Top,
            refinement_context_digest: Some(stable_hash(&[from.to_string(), to.to_string()])),
        }
    }

    fn clique_reachability_hyperedges(
        prefix: &str,
        node_count: usize,
    ) -> Vec<UnifiedHypergraphHyperedgeV0> {
        (0..node_count)
            .flat_map(|from| {
                (0..node_count)
                    .filter(move |to| *to != from)
                    .map(move |to| {
                        hyperedge_with_kind(
                            &format!("edge-{prefix}-{from}-{to}"),
                            &format!("styleModule|/workspace/clique-{from}.module.scss|root"),
                            &format!("styleModule|/workspace/clique-{to}.module.scss|root"),
                            UnifiedHypergraphEdgeKindV0::SassUse,
                        )
                    })
            })
            .collect()
    }

    fn layered_reachability_hyperedges(layer_count: usize) -> Vec<UnifiedHypergraphHyperedgeV0> {
        let mut hyperedges = Vec::new();
        let entry = "styleModule|/workspace/Entry.module.scss|root".to_string();
        for branch in ["a", "b"] {
            hyperedges.push(hyperedge_with_kind(
                &format!("edge-entry-{branch}0"),
                entry.as_str(),
                layered_node(0, branch).as_str(),
                UnifiedHypergraphEdgeKindV0::SassUse,
            ));
        }
        for layer in 0..layer_count {
            let next_layer = layer.saturating_add(1);
            for branch in ["a", "b"] {
                for next_branch in ["a", "b"] {
                    let edge_kind = match (layer + branch.len() + next_branch.len()) % 4 {
                        0 => UnifiedHypergraphEdgeKindV0::ComposesExternal,
                        1 => UnifiedHypergraphEdgeKindV0::Icss,
                        2 => UnifiedHypergraphEdgeKindV0::Value,
                        _ => UnifiedHypergraphEdgeKindV0::SassForward,
                    };
                    hyperedges.push(hyperedge_with_kind(
                        &format!("edge-{layer}-{branch}-{next_branch}"),
                        layered_node(layer, branch).as_str(),
                        layered_node(next_layer, next_branch).as_str(),
                        edge_kind,
                    ));
                }
            }
        }
        hyperedges
    }

    fn layered_node(layer: usize, branch: &str) -> String {
        format!("styleSymbol|/workspace/layer-{layer}-{branch}.module.scss|token")
    }

    fn layered_identity_ifds_hyperedges(layer_count: usize) -> Vec<UnifiedHypergraphHyperedgeV0> {
        let mut hyperedges = Vec::new();
        for branch in ["a", "b"] {
            hyperedges.push(hyperedge_with_kind(
                &format!("edge-entry-{branch}"),
                "entry",
                identity_ifds_node(0, branch).as_str(),
                UnifiedHypergraphEdgeKindV0::SassUse,
            ));
        }
        for layer in 0..layer_count {
            let next_layer = layer.saturating_add(1);
            for branch in ["a", "b"] {
                hyperedges.push(hyperedge_with_kind(
                    &format!("edge-{layer}-{branch}"),
                    identity_ifds_node(layer, branch).as_str(),
                    identity_ifds_node(next_layer, branch).as_str(),
                    UnifiedHypergraphEdgeKindV0::SassForward,
                ));
            }
        }
        hyperedges
    }

    fn identity_ifds_node(layer: usize, branch: &str) -> String {
        format!("identity:{layer}:{branch}")
    }

    fn fit_growth_exponent(samples: &[(usize, usize)]) -> f64 {
        let n = samples.len() as f64;
        let log_x = samples
            .iter()
            .map(|(size, _)| (*size as f64).ln())
            .collect::<Vec<_>>();
        let log_y = samples
            .iter()
            .map(|(_, cost)| (*cost).max(1) as f64)
            .map(f64::ln)
            .collect::<Vec<_>>();
        let mean_x = log_x.iter().sum::<f64>() / n;
        let mean_y = log_y.iter().sum::<f64>() / n;
        let numerator = log_x
            .iter()
            .zip(log_y.iter())
            .map(|(x, y)| (x - mean_x) * (y - mean_y))
            .sum::<f64>();
        let denominator = log_x
            .iter()
            .map(|x| {
                let centered = x - mean_x;
                centered * centered
            })
            .sum::<f64>();
        if denominator == 0.0 {
            0.0
        } else {
            numerator / denominator
        }
    }
}
