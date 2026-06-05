use std::collections::{BTreeMap, BTreeSet};

use serde::Serialize;

use crate::OmenaQueryCrossFileSccEvidenceV0;

use super::{HypergraphIFDSSummaryEdgeV0, OmenaQueryUnifiedCrossFileHypergraphV0};

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

pub fn summarize_omena_query_unified_cross_file_scc_report(
    hypergraph: &OmenaQueryUnifiedCrossFileHypergraphV0,
) -> OmenaQueryUnifiedCrossFileSccReportV0 {
    let adjacency = build_directed_projection_adjacency(&hypergraph.summary_edges);
    let mut sccs = collect_tarjan_sccs(&adjacency)
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

fn collect_tarjan_sccs(adjacency: &BTreeMap<String, BTreeSet<String>>) -> Vec<Vec<String>> {
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
    // Summary edges preserve the legacy endpoint kind, but SCC traversal needs
    // selector-bearing compose owners to meet their target selector identity.
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
