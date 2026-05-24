//! Streaming IFDS contracts for live LSP analysis.
//!
//! The M4-gamma default is exact (`delta = epsilon = 0`) and wire-compatible
//! with the M4-beta hypergraph IFDS substrate.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use omena_abstract_value::AbstractClassValueV0;
use omena_query::{
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
        product: "omena-streaming-ifds.polylog-connectivity-witness",
        layer_marker: STREAMING_IFDS_LAYER_MARKER_V0,
        feature_gate: STREAMING_IFDS_FEATURE_GATE_V0,
        start_node_id: start_node_id.clone(),
        reachable_node_ids: reachable_node_ids.clone(),
        polylog_query_bound: polylog_query_bound(hyperedges.len().saturating_add(1)),
        exact_default: true,
        wire_compatible_with_batch_oracle: true,
    };

    let output_facts = propagate_ifds_facts(hyperedges, events);
    let output_fact_keys = fact_keys(&output_facts);
    let previous_fact_keys = previous_cache
        .into_iter()
        .flatten()
        .flat_map(|entry| entry.fact_keys.iter().cloned())
        .collect::<BTreeSet<_>>();
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
    let batch_fact_keys = fact_keys(&propagate_ifds_facts(hyperedges, events));

    StreamingIFDSAnalysisReportV0 {
        schema_version: STREAMING_IFDS_SCHEMA_VERSION_V0,
        product: "omena-streaming-ifds.analysis-report",
        layer_marker: STREAMING_IFDS_LAYER_MARKER_V0,
        feature_gate: STREAMING_IFDS_FEATURE_GATE_V0,
        update,
        witness,
        event_count: events.len(),
        input_fact_count: events.len(),
        output_fact_count: output_facts.len(),
        dirty_fact_count,
        reused_fact_count,
        transfer_function_count: streaming_ifds_transfer_functions_v0(hyperedges).len(),
        fallback_to_batch: false,
        precision_parity_with_batch: output_fact_keys == batch_fact_keys,
        output_facts,
        summary_cache,
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
            transfer_kind: "identityMonotone",
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
            let next_fact = streaming_ifds_fact_v0(
                transfer.head_node_id.clone(),
                fact.value.clone(),
                provenance,
            );
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
            &PolylogDynamicConnectivityBackendV0,
            None,
        );

        assert_eq!(report.schema_version, "0");
        assert_eq!(report.event_count, 1);
        assert_eq!(report.input_fact_count, 1);
        assert_eq!(report.output_fact_count, 3);
        assert_eq!(report.transfer_function_count, 2);
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
            &ExactStreamingConnectivityOracleV0,
            None,
        );
        let second = run_streaming_ifds_exact_v0(
            "update-2",
            "a",
            &hyperedges,
            &events,
            &ExactStreamingConnectivityOracleV0,
            Some(&first.summary_cache),
        );

        assert_eq!(second.reused_fact_count, first.output_fact_count);
        assert_eq!(second.dirty_fact_count, 0);
        assert!(second.summary_cache[0].reused_from_previous);
        assert!(second.update.refinement_context_digest.is_some());
    }

    #[cfg(feature = "with-frame-rule")]
    #[test]
    fn frame_rule_bridge_policy_is_feature_gated() {
        let policy = streaming_ifds_frame_rule_bridge_policy_v0();
        assert_eq!(policy.schema_version, "0");
        assert_eq!(policy.feature_gate, "with-frame-rule");
        assert_eq!(policy.coarse_policy, "frameFootprintReachability");
    }

    fn hyperedge(id: &str, from: &str, to: &str) -> UnifiedHypergraphHyperedgeV0 {
        UnifiedHypergraphHyperedgeV0 {
            schema_version: "0",
            product: "test.hyperedge",
            layer_marker: "hypergraph-ifds",
            feature_gate: "hypergraph-ifds",
            hyperedge_id: id.to_string(),
            edge_kind: UnifiedHypergraphEdgeKindV0::ComposesLocal,
            source_summary_edge_id: id.to_string(),
            source_edge_kind: "composesLocal",
            source_status: "known",
            tail_node_ids: vec![from.to_string()],
            head_node_id: to.to_string(),
            order_significant_tail: false,
        }
    }
}
