use std::collections::{BTreeMap, BTreeSet, VecDeque};

use super::{HypergraphIFDSSummaryEdgeV0, UnifiedHypergraphHyperedgeV0};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::style) struct HypergraphClosurePath<N> {
    pub origin: N,
    pub target: N,
    pub depth: usize,
    pub path_labels: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::style) enum HypergraphClosureMode {
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
        let adjacency = build_adjacency(hyperedges);
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

pub(in crate::style) fn collect_hypergraph_transitive_closure_paths<N, F>(
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

pub(in crate::style) fn collect_hypergraph_transitive_closure_paths_with_mode<N, F>(
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
