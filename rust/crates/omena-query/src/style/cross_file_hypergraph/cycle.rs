use std::collections::{BTreeMap, BTreeSet};

use serde::Serialize;

use super::{UnifiedHypergraphEdgeKindV0, UnifiedHypergraphHyperedgeV0, UnifiedHypergraphNodeV0};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UnifiedHypergraphSccV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub scc_id: String,
    pub node_ids: Vec<String>,
    pub kinds: Vec<UnifiedHypergraphEdgeKindV0>,
    pub mixed_kind: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UnifiedHypergraphCycleV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub cycle_id: String,
    pub node_ids: Vec<String>,
    pub kinds: Vec<UnifiedHypergraphEdgeKindV0>,
}

pub fn summarize_unified_hypergraph_sccs(
    nodes: &[UnifiedHypergraphNodeV0],
    hyperedges: &[UnifiedHypergraphHyperedgeV0],
) -> (Vec<UnifiedHypergraphSccV0>, Vec<UnifiedHypergraphCycleV0>) {
    let node_ids = nodes
        .iter()
        .map(|node| node.node_id.clone())
        .collect::<Vec<_>>();
    let adjacency = build_adjacency(hyperedges);
    let reverse_adjacency = reverse_adjacency(&adjacency);
    let mut visited = BTreeSet::new();
    let mut order = Vec::new();
    for node_id in &node_ids {
        dfs_order(node_id, &adjacency, &mut visited, &mut order);
    }

    let mut assigned = BTreeSet::new();
    let mut sccs = Vec::new();
    for node_id in order.iter().rev() {
        if assigned.contains(node_id) {
            continue;
        }
        let mut component = Vec::new();
        dfs_component(node_id, &reverse_adjacency, &mut assigned, &mut component);
        component.sort();
        let has_self_loop = component.len() == 1
            && adjacency
                .get(component[0].as_str())
                .is_some_and(|targets| targets.contains(component[0].as_str()));
        if component.len() <= 1 && !has_self_loop {
            continue;
        }
        let kinds = edge_kinds_for_component(&component, hyperedges);
        let scc_id = format!("scc:{}", component.join("|"));
        sccs.push(UnifiedHypergraphSccV0 {
            schema_version: "0",
            product: "omena-query.unified-hypergraph-scc",
            layer_marker: "hypergraph-ifds",
            feature_gate: "hypergraph-ifds",
            scc_id: scc_id.clone(),
            node_ids: component.clone(),
            mixed_kind: kinds.len() > 1,
            kinds: kinds.clone(),
        });
    }
    sccs.sort_by_key(|scc| scc.scc_id.clone());

    let cycles = sccs
        .iter()
        .map(|scc| UnifiedHypergraphCycleV0 {
            schema_version: "0",
            product: "omena-query.unified-hypergraph-cycle",
            layer_marker: "hypergraph-ifds",
            feature_gate: "hypergraph-ifds",
            cycle_id: format!("cycle:{}", scc.scc_id),
            node_ids: scc.node_ids.clone(),
            kinds: scc.kinds.clone(),
        })
        .collect();

    (sccs, cycles)
}

fn build_adjacency(
    hyperedges: &[UnifiedHypergraphHyperedgeV0],
) -> BTreeMap<String, BTreeSet<String>> {
    let mut adjacency = BTreeMap::<String, BTreeSet<String>>::new();
    for edge in hyperedges {
        for tail_node_id in &edge.tail_node_ids {
            adjacency
                .entry(tail_node_id.clone())
                .or_default()
                .insert(edge.head_node_id.clone());
        }
        adjacency.entry(edge.head_node_id.clone()).or_default();
    }
    adjacency
}

fn reverse_adjacency(
    adjacency: &BTreeMap<String, BTreeSet<String>>,
) -> BTreeMap<String, BTreeSet<String>> {
    let mut reverse = BTreeMap::<String, BTreeSet<String>>::new();
    for (from, targets) in adjacency {
        reverse.entry(from.clone()).or_default();
        for target in targets {
            reverse
                .entry(target.clone())
                .or_default()
                .insert(from.clone());
        }
    }
    reverse
}

fn dfs_order(
    node_id: &str,
    adjacency: &BTreeMap<String, BTreeSet<String>>,
    visited: &mut BTreeSet<String>,
    order: &mut Vec<String>,
) {
    if !visited.insert(node_id.to_string()) {
        return;
    }
    if let Some(targets) = adjacency.get(node_id) {
        for target in targets {
            dfs_order(target, adjacency, visited, order);
        }
    }
    order.push(node_id.to_string());
}

fn dfs_component(
    node_id: &str,
    adjacency: &BTreeMap<String, BTreeSet<String>>,
    assigned: &mut BTreeSet<String>,
    component: &mut Vec<String>,
) {
    if !assigned.insert(node_id.to_string()) {
        return;
    }
    component.push(node_id.to_string());
    if let Some(targets) = adjacency.get(node_id) {
        for target in targets {
            dfs_component(target, adjacency, assigned, component);
        }
    }
}

fn edge_kinds_for_component(
    component: &[String],
    hyperedges: &[UnifiedHypergraphHyperedgeV0],
) -> Vec<UnifiedHypergraphEdgeKindV0> {
    let component_set = component
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let mut kinds = BTreeSet::new();
    for edge in hyperedges {
        if !component_set.contains(edge.head_node_id.as_str()) {
            continue;
        }
        if edge
            .tail_node_ids
            .iter()
            .any(|tail| component_set.contains(tail.as_str()))
        {
            kinds.insert(edge.edge_kind);
        }
    }
    kinds.into_iter().collect()
}
