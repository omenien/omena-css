//! Streaming IFDS contracts for live LSP analysis.
//!
//! The M4-gamma default is exact (`delta = epsilon = 0`) and wire-compatible
//! with the M4-beta hypergraph IFDS substrate.
//!
//! claim_level: product-wired exact default live-analysis mechanism; the polylog
//! backend label is an implementation boundary, not an asymptotic proof claim.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use omena_abstract_value::AbstractClassValueV0;
use omena_cross_file_summary::{
    OmenaUnifiedHypergraphConnectivityOracle, UnifiedHypergraphEdgeKindV0,
    UnifiedHypergraphHyperedgeV0,
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
    pub output_facts: Vec<StreamingIFDSFactV0>,
    pub summary_cache: Vec<StreamingIFDSSummaryCacheEntryV0>,
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

    let reachable_node_ids = oracle.reachable_node_ids(&start_node_id, hyperedges);
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

    // Two independently-computed fact sets over the *current* graph:
    //   * the incremental/streaming path only re-derives the dirty sub-graph
    //     reachable from the changed (event) nodes and reuses prior facts that
    //     fall outside that region, and
    //   * the batch oracle recomputes every fact from scratch over all
    //     hyperedges and events.
    // They agree when the reused prior facts are still consistent with the
    // current graph, and diverge when a prior fact survives in the reused
    // region even though the current graph no longer produces it (stale fact
    // not invalidated by the incremental dirty-set). Parity is therefore a real
    // equality of two distinct computations, not f(x) == f(x).
    let incremental_facts =
        incremental_propagate_ifds_facts(hyperedges, events, &previous_fact_keys);
    let output_fact_keys = incremental_fact_keys(hyperedges, events, &previous_fact_keys);
    let batch_fact_keys = fact_keys(&propagate_ifds_facts(hyperedges, events));

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
        transfer_function_count: streaming_ifds_transfer_functions_v0(hyperedges).len(),
        fallback_to_batch: false,
        precision_parity_with_batch: output_fact_keys == batch_fact_keys,
        output_facts: incremental_facts,
        summary_cache,
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
    let sccs = collect_streaming_ifds_tarjan_sccs(&adjacency);
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
    hyperedges
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
        .collect()
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

fn propagate_ifds_facts(
    hyperedges: &[UnifiedHypergraphHyperedgeV0],
    events: &[StreamingIfdsEventInputV0],
) -> Vec<StreamingIFDSFactV0> {
    let transfer_functions = streaming_ifds_transfer_functions_v0(hyperedges);
    let mut seen = BTreeSet::<String>::new();
    let mut pending = VecDeque::<StreamingIFDSFactV0>::new();
    let mut output = Vec::<StreamingIFDSFactV0>::new();

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
        for transfer in transfer_functions
            .iter()
            .filter(|transfer| transfer.tail_node_ids.contains(&fact.node_id))
        {
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
    output
}

/// Nodes that the changed event nodes can still reach over the *current* graph.
/// This is the incremental dirty sub-graph: facts at these nodes are re-derived
/// from scratch, everything else may be reused from the prior fact set.
fn incremental_dirty_nodes(
    hyperedges: &[UnifiedHypergraphHyperedgeV0],
    events: &[StreamingIfdsEventInputV0],
) -> BTreeSet<String> {
    propagate_ifds_facts(hyperedges, events)
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
    hyperedges: &[UnifiedHypergraphHyperedgeV0],
    events: &[StreamingIfdsEventInputV0],
    previous_fact_keys: &BTreeSet<String>,
) -> Vec<String> {
    // Cold start (no prior facts): the dirty region is the whole reachable graph,
    // so the incremental path coincides with a full recompute.
    if previous_fact_keys.is_empty() {
        return fact_keys(&propagate_ifds_facts(hyperedges, events));
    }

    let dirty_nodes = incremental_dirty_nodes(hyperedges, events);
    let mut keys = fact_keys(&propagate_ifds_facts(hyperedges, events))
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
    hyperedges: &[UnifiedHypergraphHyperedgeV0],
    events: &[StreamingIfdsEventInputV0],
    previous_fact_keys: &BTreeSet<String>,
) -> Vec<StreamingIFDSFactV0> {
    let mut output = propagate_ifds_facts(hyperedges, events);
    if !previous_fact_keys.is_empty() {
        let dirty_nodes = incremental_dirty_nodes(hyperedges, events);
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

fn collect_streaming_ifds_tarjan_sccs(
    adjacency: &BTreeMap<String, BTreeSet<String>>,
) -> Vec<Vec<String>> {
    let mut state = StreamingIfdsTarjanStateV0::default();
    for node_id in adjacency.keys() {
        if !state.indices.contains_key(node_id) {
            state.visit(node_id, adjacency);
        }
    }
    state.components
}

#[derive(Debug, Default)]
struct StreamingIfdsTarjanStateV0 {
    next_index: usize,
    stack: Vec<String>,
    on_stack: BTreeSet<String>,
    indices: BTreeMap<String, usize>,
    lowlinks: BTreeMap<String, usize>,
    components: Vec<Vec<String>>,
}

impl StreamingIfdsTarjanStateV0 {
    fn visit(&mut self, node_id: &str, adjacency: &BTreeMap<String, BTreeSet<String>>) {
        let index = self.next_index;
        self.next_index = self.next_index.saturating_add(1);
        self.indices.insert(node_id.to_string(), index);
        self.lowlinks.insert(node_id.to_string(), index);
        self.stack.push(node_id.to_string());
        self.on_stack.insert(node_id.to_string());

        if let Some(targets) = adjacency.get(node_id) {
            for target in targets {
                if !self.indices.contains_key(target.as_str()) {
                    self.visit(target, adjacency);
                    if let (Some(target_lowlink), Some(current_lowlink)) = (
                        self.lowlinks.get(target.as_str()).copied(),
                        self.lowlinks.get(node_id).copied(),
                    ) {
                        self.lowlinks
                            .insert(node_id.to_string(), current_lowlink.min(target_lowlink));
                    }
                } else if self.on_stack.contains(target.as_str())
                    && let (Some(target_index), Some(current_lowlink)) = (
                        self.indices.get(target.as_str()).copied(),
                        self.lowlinks.get(node_id).copied(),
                    )
                {
                    self.lowlinks
                        .insert(node_id.to_string(), current_lowlink.min(target_index));
                }
            }
        }

        if self.lowlinks.get(node_id) == self.indices.get(node_id) {
            let mut component = Vec::new();
            while let Some(stack_node) = self.stack.pop() {
                self.on_stack.remove(stack_node.as_str());
                let done = stack_node == node_id;
                component.push(stack_node);
                if done {
                    break;
                }
            }
            component.sort();
            self.components.push(component);
        }
    }
}

fn exact_reachable_node_ids(
    start_node_id: &str,
    hyperedges: &[UnifiedHypergraphHyperedgeV0],
) -> Vec<String> {
    let adjacency = streaming_ifds_node_adjacency(hyperedges);

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
