//! Cross-file summary hypergraph substrate shared by query and streaming IFDS.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use omena_abstract_value::{LinearProvenanceV0, NaturalCountProvenanceSemiringV0};
use serde::Serialize;

pub type OmenaCrossFileLinearProvenanceV0 = LinearProvenanceV0<NaturalCountProvenanceSemiringV0>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryCrossFileSummaryV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub status: &'static str,
    pub summary_scope: &'static str,
    pub style_count: usize,
    pub summary_edge_count: usize,
    pub edge_kind_counts: Vec<OmenaQueryCrossFileSummaryEdgeKindCountV0>,
    pub summary_hash: String,
    pub edges: Vec<OmenaQueryCrossFileSummaryEdgeV0>,
    pub capabilities: OmenaQueryCrossFileSummaryCapabilitiesV0,
    pub next_priorities: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryCrossFileSummaryEdgeKindCountV0 {
    pub edge_kind: &'static str,
    pub count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryCrossFileSummaryEdgeV0 {
    pub edge_id: String,
    pub edge_kind: &'static str,
    pub from_kind: &'static str,
    pub from_path: String,
    pub target_kind: Option<&'static str>,
    pub target_path: Option<String>,
    pub source: Option<String>,
    pub owner_selector_name: Option<String>,
    pub local_name: Option<String>,
    pub remote_name: Option<String>,
    pub target_names: Vec<String>,
    pub status: &'static str,
    pub provenance: Vec<&'static str>,
    pub linear_provenance: OmenaCrossFileLinearProvenanceV0,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryCrossFileSummaryCapabilitiesV0 {
    pub css_modules_composes_edges_ready: bool,
    pub css_modules_value_edges_ready: bool,
    pub css_modules_icss_edges_ready: bool,
    pub sass_module_edges_ready: bool,
    pub style_design_token_reference_edges_ready: bool,
    pub source_selector_reference_edges_ready: bool,
    pub stable_summary_hash_ready: bool,
    pub linear_provenance_ready: bool,
    pub linear_provenance_round_trip_ready: bool,
    pub linear_provenance_semiring_laws_hold: bool,
}

impl OmenaQueryCrossFileSummaryEdgeV0 {
    pub fn linear_provenance_round_trips_legacy_labels(&self) -> bool {
        self.linear_provenance.labels() == self.provenance
    }
}

impl OmenaQueryCrossFileSummaryV0 {
    pub fn linear_provenance_round_trips_legacy_labels(&self) -> bool {
        self.edges
            .iter()
            .all(OmenaQueryCrossFileSummaryEdgeV0::linear_provenance_round_trips_legacy_labels)
    }

    pub fn recompute_stable_summary_hash(&self) -> String {
        stable_omena_query_cross_file_summary_hash(self.edges.as_slice())
    }
}

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum UnifiedHypergraphEdgeKindV0 {
    ComposesLocal,
    ComposesGlobal,
    ComposesExternal,
    SassUse,
    SassForward,
    SassImport,
    LessImport,
    LessModuleGraphClosure,
    Value,
    Icss,
    ForeignReference,
}

impl UnifiedHypergraphEdgeKindV0 {
    pub const fn as_wire_label(self) -> &'static str {
        match self {
            Self::ComposesLocal => "composesLocal",
            Self::ComposesGlobal => "composesGlobal",
            Self::ComposesExternal => "composesExternal",
            Self::SassUse => "sassUse",
            Self::SassForward => "sassForward",
            Self::SassImport => "sassImport",
            Self::LessImport => "lessImport",
            Self::LessModuleGraphClosure => "lessModuleGraphClosure",
            Self::Value => "value",
            Self::Icss => "icss",
            Self::ForeignReference => "foreignReference",
        }
    }

    pub const fn is_order_significant(self) -> bool {
        matches!(
            self,
            Self::ComposesLocal | Self::ComposesGlobal | Self::ComposesExternal
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UnifiedHypergraphHyperedgeV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub hyperedge_id: String,
    pub edge_kind: UnifiedHypergraphEdgeKindV0,
    pub source_summary_edge_id: String,
    pub source_edge_kind: &'static str,
    pub source_status: &'static str,
    pub tail_node_ids: Vec<String>,
    pub head_node_id: String,
    pub order_significant_tail: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HypergraphIFDSSummaryEdgeV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub summary_edge_id: String,
    pub projection_edge_id: String,
    pub hyperedge_id: String,
    pub from_node_id: String,
    pub to_node_id: String,
    pub edge_kind: UnifiedHypergraphEdgeKindV0,
    pub status: &'static str,
    pub provenance: Vec<&'static str>,
    pub linear_provenance: OmenaCrossFileLinearProvenanceV0,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryUnifiedCrossFileHypergraphV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub status: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub node_count: usize,
    pub hyperedge_count: usize,
    pub summary_edge_count: usize,
    pub projection_edge_ids: Vec<String>,
    pub hyperedges: Vec<UnifiedHypergraphHyperedgeV0>,
    pub summary_edges: Vec<HypergraphIFDSSummaryEdgeV0>,
    pub gate_predicates: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryCrossFileSccEvidenceV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub feature_gate: &'static str,
    pub claim_level: &'static str,
    pub theorem_claimed: bool,
    pub connectivity_backend: &'static str,
    pub polylog_bound_scope: &'static str,
    pub scc_id: String,
    pub node_count: usize,
    pub directed_edge_count: usize,
    pub cross_file: bool,
    pub node_ids: Vec<String>,
    pub style_paths: Vec<String>,
    pub edge_kinds: Vec<&'static str>,
    pub summary_edge_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryUnifiedCrossFileSccReportV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub feature_gate: &'static str,
    pub claim_level: &'static str,
    pub theorem_claimed: bool,
    pub connectivity_backend: &'static str,
    pub polylog_bound_scope: &'static str,
    pub node_count: usize,
    pub directed_edge_count: usize,
    pub cyclic_scc_count: usize,
    pub sccs: Vec<OmenaQueryCrossFileSccEvidenceV0>,
    pub gate_predicates: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HypergraphClosurePath<N> {
    pub origin: N,
    pub target: N,
    pub depth: usize,
    pub path_labels: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HypergraphClosureMode {
    CanonicalFirstTarget,
    RawAllPaths,
}

pub trait OmenaUnifiedHypergraphConnectivityOracle {
    fn reachable_node_ids(
        &self,
        start_node_id: &str,
        hyperedges: &[UnifiedHypergraphHyperedgeV0],
    ) -> Vec<String>;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct BatchHypergraphConnectivityOracle;

impl OmenaUnifiedHypergraphConnectivityOracle for BatchHypergraphConnectivityOracle {
    fn reachable_node_ids(
        &self,
        start_node_id: &str,
        hyperedges: &[UnifiedHypergraphHyperedgeV0],
    ) -> Vec<String> {
        collect_reachable_node_ids(start_node_id, &build_adjacency(hyperedges))
    }
}

/// The single shared reachability BFS loop for the cross-file engine: forward closure over a
/// deterministic `BTreeMap` adjacency, returned sorted (BTreeSet order). Generic over the
/// adjacency key type so each caller keeps its OWN adjacency builder (the distinct node spaces
/// comment-2 point 4 demands) while sharing exactly one traversal loop. (The substrate diagnostics
/// reachability is a separate borrowed-`&str` DFS over inline-filtered `resolved` edges and is
/// intentionally NOT one of the two BFS impls this collapses.)
pub fn collect_reachable_node_ids<K>(
    start_node_id: &str,
    adjacency: &BTreeMap<K, BTreeSet<String>>,
) -> Vec<String>
where
    K: Ord + std::borrow::Borrow<str>,
{
    let mut seen = BTreeSet::new();
    let mut pending = VecDeque::from([start_node_id.to_string()]);
    while let Some(current) = pending.pop_front() {
        for target in adjacency.get(current.as_str()).into_iter().flatten() {
            if seen.insert(target.clone()) {
                pending.push_back(target.clone());
            }
        }
    }
    seen.into_iter().collect()
}

pub fn tabulate_hypergraph_ifds_summary_edges(
    hyperedges: &[UnifiedHypergraphHyperedgeV0],
    projected_edges: Vec<HypergraphIFDSSummaryEdgeV0>,
) -> Vec<HypergraphIFDSSummaryEdgeV0> {
    let hyperedge_ids = hyperedges
        .iter()
        .map(|edge| edge.hyperedge_id.as_str())
        .collect::<BTreeSet<_>>();
    let mut edges = projected_edges
        .into_iter()
        .filter(|edge| hyperedge_ids.contains(edge.hyperedge_id.as_str()))
        .collect::<Vec<_>>();
    edges.sort_by(|left, right| {
        left.projection_edge_id
            .cmp(&right.projection_edge_id)
            .then(left.hyperedge_id.cmp(&right.hyperedge_id))
    });
    edges
}

pub fn summarize_omena_query_unified_cross_file_scc_report(
    hypergraph: &OmenaQueryUnifiedCrossFileHypergraphV0,
) -> OmenaQueryUnifiedCrossFileSccReportV0 {
    let adjacency = build_directed_projection_adjacency(&hypergraph.summary_edges);
    let mut sccs = collect_directed_graph_sccs(&adjacency)
        .into_iter()
        .filter_map(|node_ids| summarize_cyclic_scc(&node_ids, &hypergraph.summary_edges))
        .collect::<Vec<_>>();
    sccs.sort_by(|left, right| {
        left.node_ids
            .cmp(&right.node_ids)
            .then(left.summary_edge_ids.cmp(&right.summary_edge_ids))
    });
    for (index, scc) in sccs.iter_mut().enumerate() {
        scc.scc_id = format!("exact-tarjan-scc:{}", index + 1);
    }

    OmenaQueryUnifiedCrossFileSccReportV0 {
        schema_version: "0",
        product: "omena-query.unified-cross-file-scc-report",
        feature_gate: "cross-file-scc-v0",
        claim_level: "fixtureWitnessExactTarjanScc",
        theorem_claimed: false,
        connectivity_backend: "exactTarjanScc",
        polylog_bound_scope: "notClaimedExactTraversal",
        node_count: adjacency.len(),
        directed_edge_count: hypergraph
            .summary_edges
            .iter()
            .filter(|edge| summary_edge_has_supported_target(edge.status))
            .count(),
        cyclic_scc_count: sccs.len(),
        sccs,
        gate_predicates: vec![
            "exactTarjanSccBackend",
            "theorem_claimed=false",
            "polylog_bound_scope=notClaimedExactTraversal",
        ],
    }
}

pub fn summarize_omena_query_unified_cross_file_hypergraph(
    summary: &OmenaQueryCrossFileSummaryV0,
) -> OmenaQueryUnifiedCrossFileHypergraphV0 {
    let mut builder = UnifiedCrossFileHypergraphBuilder::default();
    for edge in &summary.edges {
        builder.add_summary_edge(edge);
    }
    builder.finish()
}

pub fn collect_hypergraph_transitive_closure_paths<N, F>(
    graph: &BTreeMap<N, BTreeSet<N>>,
    mut label: F,
) -> (Vec<HypergraphClosurePath<N>>, Vec<Vec<String>>)
where
    N: Clone + Ord,
    F: FnMut(&N) -> String,
{
    collect_hypergraph_transitive_closure_paths_with_mode(
        graph,
        &mut label,
        HypergraphClosureMode::CanonicalFirstTarget,
    )
}

pub fn collect_hypergraph_transitive_closure_paths_with_mode<N, F>(
    graph: &BTreeMap<N, BTreeSet<N>>,
    label: &mut F,
    mode: HypergraphClosureMode,
) -> (Vec<HypergraphClosurePath<N>>, Vec<Vec<String>>)
where
    N: Clone + Ord,
    F: FnMut(&N) -> String,
{
    let mut closure_paths = Vec::new();
    let mut cycle_paths = Vec::new();
    let mut seen_cycles = BTreeSet::new();
    let first_target = mode == HypergraphClosureMode::CanonicalFirstTarget;

    for start in graph.keys() {
        let mut visited = BTreeSet::new();
        let mut pending = VecDeque::from([(start.clone(), vec![start.clone()])]);
        while let Some((current, path)) = pending.pop_front() {
            for target in graph.get(&current).into_iter().flatten() {
                if let Some(cycle_start) = path.iter().position(|node| node == target) {
                    let mut cycle = path[cycle_start..].to_vec();
                    cycle.push(target.clone());
                    let mut labels = cycle.iter().map(&mut *label).collect::<Vec<_>>();
                    if first_target {
                        labels = canonical_hypergraph_cycle_labels(labels);
                    }
                    if !labels.is_empty() && seen_cycles.insert(labels.clone()) {
                        cycle_paths.push(labels);
                    }
                    continue;
                }
                if first_target && !visited.insert(target.clone()) {
                    continue;
                }
                let mut edge_path = path.clone();
                edge_path.push(target.clone());
                closure_paths.push(HypergraphClosurePath {
                    origin: start.clone(),
                    target: target.clone(),
                    depth: edge_path.len().saturating_sub(1),
                    path_labels: edge_path.iter().map(&mut *label).collect(),
                });
                pending.push_back((target.clone(), edge_path));
            }
        }
    }
    (closure_paths, cycle_paths)
}

/// Generous per-SCC enumeration work budget; real module-graph SCCs are tiny (a cycle is a user
/// error to report), so the cap is a backstop that never fires in practice.
const DEFAULT_CYCLE_ENUMERATION_WORK_CAP: usize = 1 << 16;

/// All elementary directed circuits of `adjacency`, found per strongly-connected component:
/// partition with `collect_directed_graph_sccs`, then enumerate the simple cycles CONFINED to each
/// non-trivial SCC (a back-edge to the start closes a circuit). Each circuit is canonicalized via
/// `canonical_hypergraph_cycle_labels` (lex-smallest rotation, emitted CLOSED so a consumer's
/// `windows(2)` successor lookup resolves), deduped and sorted. This is the cross-file CYCLE owner,
/// decoupled from the all-paths closure scan so the latter can be replaced without touching cycles.
pub fn collect_directed_graph_cycles(
    adjacency: &BTreeMap<String, BTreeSet<String>>,
) -> Vec<Vec<String>> {
    collect_directed_graph_cycles_with_work_cap(adjacency, DEFAULT_CYCLE_ENUMERATION_WORK_CAP)
}

/// `collect_directed_graph_cycles` with an explicit per-SCC work cap (for tests). On cap-hit a
/// dense SCC degrades to its lex-smallest representative circuit — a witnessed shrink, NEVER a
/// silent drop.
pub fn collect_directed_graph_cycles_with_work_cap(
    adjacency: &BTreeMap<String, BTreeSet<String>>,
    per_scc_work_cap: usize,
) -> Vec<Vec<String>> {
    let mut circuits = BTreeSet::new();
    for scc in collect_directed_graph_sccs(adjacency) {
        let self_loop = scc.len() == 1
            && adjacency
                .get(scc[0].as_str())
                .is_some_and(|targets| targets.contains(&scc[0]));
        if scc.len() < 2 && !self_loop {
            continue;
        }
        let scc_nodes = scc.iter().map(String::as_str).collect::<BTreeSet<_>>();
        let mut found = BTreeSet::new();
        let mut work = 0usize;
        let mut capped = false;
        'starts: for start in &scc {
            // BFS over simple paths so the shortest circuits surface first (so the cap-fallback
            // representative is a real, short circuit).
            let mut pending = VecDeque::from([(start.clone(), vec![start.clone()])]);
            while let Some((current, path)) = pending.pop_front() {
                work += 1;
                if work > per_scc_work_cap {
                    capped = true;
                    break 'starts;
                }
                for target in adjacency.get(current.as_str()).into_iter().flatten() {
                    if !scc_nodes.contains(target.as_str()) {
                        continue;
                    }
                    if target == start {
                        let mut ring = path.clone();
                        ring.push(target.clone());
                        let canonical = canonical_hypergraph_cycle_labels(ring);
                        if !canonical.is_empty() {
                            found.insert(canonical);
                        }
                    } else if !path.iter().any(|node| node == target) {
                        let mut next = path.clone();
                        next.push(target.clone());
                        pending.push_back((target.clone(), next));
                    }
                }
            }
        }
        if capped {
            circuits.extend(found.into_iter().next());
        } else {
            circuits.extend(found);
        }
    }
    circuits.into_iter().collect()
}

fn canonical_hypergraph_cycle_labels(mut labels: Vec<String>) -> Vec<String> {
    if labels.len() > 1 && labels.first() == labels.last() {
        labels.pop();
    }
    if labels.is_empty() {
        return labels;
    }
    let mut best = labels.clone();
    for offset in 1..labels.len() {
        let mut rotated = labels[offset..].to_vec();
        rotated.extend_from_slice(&labels[..offset]);
        best = best.min(rotated);
    }
    best.push(best[0].clone());
    best
}

fn build_adjacency(
    hyperedges: &[UnifiedHypergraphHyperedgeV0],
) -> BTreeMap<&str, BTreeSet<String>> {
    let mut adjacency = BTreeMap::<&str, BTreeSet<String>>::new();
    for edge in hyperedges {
        for tail in &edge.tail_node_ids {
            adjacency
                .entry(tail.as_str())
                .or_default()
                .insert(edge.head_node_id.clone());
        }
    }
    adjacency
}

#[derive(Default)]
struct UnifiedCrossFileHypergraphBuilder {
    node_ids: BTreeSet<String>,
    hyperedges: Vec<UnifiedHypergraphHyperedgeV0>,
    summary_edges: Vec<HypergraphIFDSSummaryEdgeV0>,
}

impl UnifiedCrossFileHypergraphBuilder {
    fn add_summary_edge(&mut self, edge: &OmenaQueryCrossFileSummaryEdgeV0) {
        let edge_kind = unified_edge_kind_for_summary_edge(edge);
        let from_node_id = endpoint_node_id(edge, false);
        let to_node_id = endpoint_node_id(edge, true);
        let tail_node_ids = if edge_kind.is_order_significant() && !edge.target_names.is_empty() {
            edge.target_names
                .iter()
                .map(|target_name| {
                    node_id(
                        "styleSymbol",
                        edge.target_path
                            .as_deref()
                            .unwrap_or(edge.from_path.as_str()),
                        Some(target_name),
                    )
                })
                .collect::<Vec<_>>()
        } else {
            vec![from_node_id.clone()]
        };
        self.node_ids.insert(from_node_id.clone());
        self.node_ids.insert(to_node_id.clone());
        self.node_ids.extend(tail_node_ids.iter().cloned());

        let hyperedge_id = format!(
            "hyperedge:{}|{}|{}",
            edge_kind.as_wire_label(),
            edge.edge_id,
            tail_node_ids.join(">")
        );
        self.hyperedges.push(UnifiedHypergraphHyperedgeV0 {
            schema_version: "0",
            product: "omena-query.unified-hypergraph-hyperedge",
            layer_marker: "hypergraph-ifds",
            feature_gate: "hypergraph-ifds",
            hyperedge_id: hyperedge_id.clone(),
            edge_kind,
            source_summary_edge_id: edge.edge_id.clone(),
            source_edge_kind: edge.edge_kind,
            source_status: edge.status,
            tail_node_ids,
            head_node_id: to_node_id.clone(),
            order_significant_tail: edge_kind.is_order_significant(),
        });
        self.summary_edges.push(HypergraphIFDSSummaryEdgeV0 {
            schema_version: "0",
            product: "omena-query.hypergraph-ifds-summary-edge",
            layer_marker: "hypergraph-ifds",
            feature_gate: "hypergraph-ifds",
            summary_edge_id: format!("ifds-summary:{}", edge.edge_id),
            projection_edge_id: edge.edge_id.clone(),
            hyperedge_id,
            from_node_id,
            to_node_id,
            edge_kind,
            status: edge.status,
            provenance: edge.provenance.clone(),
            linear_provenance: edge.linear_provenance.clone(),
        });
    }

    fn finish(mut self) -> OmenaQueryUnifiedCrossFileHypergraphV0 {
        self.hyperedges
            .sort_by_key(|edge| edge.hyperedge_id.clone());
        let summary_edges =
            tabulate_hypergraph_ifds_summary_edges(&self.hyperedges, self.summary_edges);
        let projection_edge_ids = summary_edges
            .iter()
            .map(|edge| edge.projection_edge_id.clone())
            .collect::<Vec<_>>();

        OmenaQueryUnifiedCrossFileHypergraphV0 {
            schema_version: "0",
            product: "omena-query.unified-cross-file-hypergraph",
            status: "hypergraphIfdsProjection",
            layer_marker: "hypergraph-ifds",
            feature_gate: "hypergraph-ifds",
            node_count: self.node_ids.len(),
            hyperedge_count: self.hyperedges.len(),
            summary_edge_count: summary_edges.len(),
            projection_edge_ids,
            hyperedges: self.hyperedges,
            summary_edges,
            gate_predicates: vec![
                "P1.typeIntroduction",
                "P2.byteEqualAdjacencyProjection",
                "P3.sccUnification",
                "P4.summaryEdgeSetEquality",
                "P5.projectionHelper",
                "P6.closureBodySwitchOver",
                "P7.v0Publication",
                "batchConnectivityOracle",
                "streamingOracleWireCompatible",
                "composesTailOrderingUsesVec",
            ],
        }
    }
}

fn endpoint_node_id(edge: &OmenaQueryCrossFileSummaryEdgeV0, target: bool) -> String {
    let (kind, path, symbol) = if target {
        (
            node_kind_for_summary_kind(edge.target_kind.unwrap_or(edge.from_kind), true),
            edge.target_path
                .as_deref()
                .unwrap_or(edge.from_path.as_str()),
            edge.remote_name
                .as_deref()
                .or_else(|| edge.target_names.first().map(String::as_str)),
        )
    } else {
        (
            node_kind_for_summary_kind(edge.from_kind, false),
            edge.from_path.as_str(),
            edge.owner_selector_name
                .as_deref()
                .or(edge.local_name.as_deref()),
        )
    };
    node_id(kind, path, symbol)
}

fn node_id(kind: &'static str, path: &str, symbol: Option<&str>) -> String {
    format!("{}|{}|{}", kind, path, symbol.unwrap_or("-"))
}

fn node_kind_for_summary_kind(kind: &str, target: bool) -> &'static str {
    match (kind, target) {
        ("style", false) => "styleModule",
        ("style", true) => "styleSymbol",
        ("source", false) => "sourceModule",
        ("source", true) => "sourceSymbol",
        _ => "foreignSymbol",
    }
}

fn unified_edge_kind_for_summary_edge(
    edge: &OmenaQueryCrossFileSummaryEdgeV0,
) -> UnifiedHypergraphEdgeKindV0 {
    match edge.edge_kind {
        "composesLocal" => UnifiedHypergraphEdgeKindV0::ComposesLocal,
        "composesGlobal" => UnifiedHypergraphEdgeKindV0::ComposesGlobal,
        "cssModulesComposesImport" | "cssModulesComposesClosure" | "composesExternal" => {
            UnifiedHypergraphEdgeKindV0::ComposesExternal
        }
        "sassUse" => UnifiedHypergraphEdgeKindV0::SassUse,
        "sassForward" => UnifiedHypergraphEdgeKindV0::SassForward,
        "sassImport" => UnifiedHypergraphEdgeKindV0::SassImport,
        "lessImport" => UnifiedHypergraphEdgeKindV0::LessImport,
        "lessModuleGraphClosure" => UnifiedHypergraphEdgeKindV0::LessModuleGraphClosure,
        "cssModulesValueImport" | "cssModulesValueClosure" | "value" => {
            UnifiedHypergraphEdgeKindV0::Value
        }
        "cssModulesIcssImport" | "cssModulesIcssClosure" | "icss" => {
            UnifiedHypergraphEdgeKindV0::Icss
        }
        _ => UnifiedHypergraphEdgeKindV0::ForeignReference,
    }
}

fn build_directed_projection_adjacency(
    summary_edges: &[HypergraphIFDSSummaryEdgeV0],
) -> BTreeMap<String, BTreeSet<String>> {
    let mut adjacency = BTreeMap::<String, BTreeSet<String>>::new();
    for edge in summary_edges {
        if !summary_edge_has_supported_target(edge.status) {
            continue;
        }
        let from_node_id = canonical_scc_node_id(edge.from_node_id.as_str());
        let to_node_id = canonical_scc_node_id(edge.to_node_id.as_str());
        adjacency.entry(from_node_id.clone()).or_default();
        adjacency.entry(to_node_id.clone()).or_default();
        adjacency
            .entry(from_node_id)
            .or_default()
            .insert(to_node_id);
    }
    adjacency
}

/// The single shared strongly-connected-components primitive for the cross-file engine: exact
/// Tarjan over a deterministic `BTreeMap` adjacency, each component sorted, components in Tarjan
/// reverse-topological discovery order. Consumed by both this crate's unified SCC report and
/// `omena-streaming-ifds` (which previously carried a byte-identical duplicate).
pub fn collect_directed_graph_sccs(
    adjacency: &BTreeMap<String, BTreeSet<String>>,
) -> Vec<Vec<String>> {
    let mut state = TarjanState::default();
    for node_id in adjacency.keys() {
        if !state.indices.contains_key(node_id) {
            state.visit(node_id, adjacency);
        }
    }
    state.components
}

#[derive(Default)]
struct TarjanState {
    next_index: usize,
    stack: Vec<String>,
    on_stack: BTreeSet<String>,
    indices: BTreeMap<String, usize>,
    lowlinks: BTreeMap<String, usize>,
    components: Vec<Vec<String>>,
}

impl TarjanState {
    fn visit(&mut self, node_id: &str, adjacency: &BTreeMap<String, BTreeSet<String>>) {
        let index = self.next_index;
        self.next_index += 1;
        self.indices.insert(node_id.to_string(), index);
        self.lowlinks.insert(node_id.to_string(), index);
        self.stack.push(node_id.to_string());
        self.on_stack.insert(node_id.to_string());

        if let Some(targets) = adjacency.get(node_id) {
            for target in targets {
                if !self.indices.contains_key(target.as_str()) {
                    self.visit(target, adjacency);
                    let target_lowlink = self.lowlinks[target.as_str()];
                    let current_lowlink = self.lowlinks[node_id];
                    self.lowlinks
                        .insert(node_id.to_string(), current_lowlink.min(target_lowlink));
                } else if self.on_stack.contains(target.as_str()) {
                    let target_index = self.indices[target.as_str()];
                    let current_lowlink = self.lowlinks[node_id];
                    self.lowlinks
                        .insert(node_id.to_string(), current_lowlink.min(target_index));
                }
            }
        }

        if self.lowlinks[node_id] == self.indices[node_id] {
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

fn summarize_cyclic_scc(
    node_ids: &[String],
    summary_edges: &[HypergraphIFDSSummaryEdgeV0],
) -> Option<OmenaQueryCrossFileSccEvidenceV0> {
    let node_set = node_ids.iter().map(String::as_str).collect::<BTreeSet<_>>();
    let internal_edges = summary_edges
        .iter()
        .filter(|edge| summary_edge_has_supported_target(edge.status))
        .filter(|edge| {
            let from_node_id = canonical_scc_node_id(edge.from_node_id.as_str());
            let to_node_id = canonical_scc_node_id(edge.to_node_id.as_str());
            node_set.contains(from_node_id.as_str()) && node_set.contains(to_node_id.as_str())
        })
        .collect::<Vec<_>>();
    let has_self_loop = internal_edges.iter().any(|edge| {
        canonical_scc_node_id(edge.from_node_id.as_str())
            == canonical_scc_node_id(edge.to_node_id.as_str())
    });
    if node_ids.len() < 2 && !has_self_loop {
        return None;
    }

    let style_paths = node_ids
        .iter()
        .filter_map(|node_id| style_path_from_node_id(node_id))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let edge_kinds = internal_edges
        .iter()
        .map(|edge| edge.edge_kind.as_wire_label())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let summary_edge_ids = internal_edges
        .iter()
        .map(|edge| edge.projection_edge_id.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();

    Some(OmenaQueryCrossFileSccEvidenceV0 {
        schema_version: "0",
        product: "omena-query.cross-file-scc-evidence",
        feature_gate: "cross-file-scc-v0",
        claim_level: "fixtureWitnessExactTarjanScc",
        theorem_claimed: false,
        connectivity_backend: "exactTarjanScc",
        polylog_bound_scope: "notClaimedExactTraversal",
        scc_id: String::new(),
        node_count: node_ids.len(),
        directed_edge_count: internal_edges.len(),
        cross_file: style_paths.len() > 1,
        node_ids: node_ids.to_vec(),
        style_paths,
        edge_kinds,
        summary_edge_ids,
    })
}

fn style_path_from_node_id(node_id: &str) -> Option<String> {
    let mut parts = node_id.splitn(3, '|');
    let _kind = parts.next()?;
    let path = parts.next()?;
    Some(path.to_string())
}

fn canonical_scc_node_id(node_id: &str) -> String {
    let mut parts = node_id.splitn(3, '|');
    let Some(kind) = parts.next() else {
        return node_id.to_string();
    };
    let Some(path) = parts.next() else {
        return node_id.to_string();
    };
    let Some(symbol) = parts.next() else {
        return node_id.to_string();
    };
    if kind == "styleModule" && symbol != "-" {
        return format!("styleSymbol|{path}|{symbol}");
    }
    node_id.to_string()
}

fn summary_edge_has_supported_target(status: &str) -> bool {
    matches!(
        status,
        "resolved" | "reachable" | "localResolved" | "importResolved" | "external"
    )
}

fn stable_omena_query_cross_file_summary_hash(
    edges: &[OmenaQueryCrossFileSummaryEdgeV0],
) -> String {
    let mut hash = 0xcbf29ce484222325u64;
    stable_omena_query_hash_piece(&mut hash, "omena-query.cross-file-summary");
    stable_omena_query_hash_piece(&mut hash, "0");
    for edge in edges {
        stable_omena_query_hash_piece(&mut hash, edge.edge_id.as_str());
        stable_omena_query_hash_piece(&mut hash, edge.status);
        stable_omena_query_hash_piece(&mut hash, edge.linear_provenance.semiring_identifier());
        let term_count = edge.linear_provenance.term_count.to_string();
        stable_omena_query_hash_piece(&mut hash, term_count.as_str());
        for term in &edge.linear_provenance.terms {
            let coefficient = term.coefficient.to_string();
            stable_omena_query_hash_piece(&mut hash, coefficient.as_str());
            stable_omena_query_hash_piece(&mut hash, term.label);
        }
    }
    format!("{hash:016x}")
}

fn stable_omena_query_hash_piece(hash: &mut u64, piece: &str) {
    for byte in piece.as_bytes() {
        *hash ^= u64::from(*byte);
        *hash = hash.wrapping_mul(0x100000001b3);
    }
    *hash ^= 0xff;
    *hash = hash.wrapping_mul(0x100000001b3);
}
