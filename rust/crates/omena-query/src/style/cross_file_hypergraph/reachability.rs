use std::collections::{BTreeMap, BTreeSet, VecDeque};

use super::{HypergraphIFDSSummaryEdgeV0, UnifiedHypergraphHyperedgeV0};

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
            let Some(targets) = adjacency.get(current.as_str()) else {
                continue;
            };
            for target in targets {
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
    let oracle = BatchHypergraphConnectivityOracle;
    let hyperedge_ids = hyperedges
        .iter()
        .map(|edge| edge.hyperedge_id.as_str())
        .collect::<BTreeSet<_>>();
    let mut edges = projected_edges
        .into_iter()
        .filter(|edge| hyperedge_ids.contains(edge.hyperedge_id.as_str()))
        .collect::<Vec<_>>();
    edges.sort_by_key(|edge| {
        (
            edge.projection_edge_id.clone(),
            oracle
                .reachable_node_ids(edge.from_node_id.as_str(), hyperedges)
                .len(),
        )
    });
    edges
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
