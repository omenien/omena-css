use std::collections::BTreeSet;

use ascent::ascent;
use omena_cross_file_summary::UnifiedHypergraphHyperedgeV0;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DatalogReachabilityWitnessV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub start_node_id: String,
    pub reachable_node_ids: Vec<String>,
    pub edge_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DatalogSelectorEqualityWitnessV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub left: String,
    pub right: String,
    pub equal: bool,
}

ascent! {
    relation edge(String, String);
    relation seed(String);
    relation reach(String);

    reach(x.clone()) <-- seed(x);
    reach(y.clone()) <-- reach(x), edge(x, y);
}

pub fn datalog_reachable_node_ids(
    start_node_id: impl Into<String>,
    hyperedges: &[UnifiedHypergraphHyperedgeV0],
) -> Vec<String> {
    datalog_reachability_witness_v0(start_node_id, hyperedges).reachable_node_ids
}

pub fn datalog_reachability_witness_v0(
    start_node_id: impl Into<String>,
    hyperedges: &[UnifiedHypergraphHyperedgeV0],
) -> DatalogReachabilityWitnessV0 {
    let start_node_id = start_node_id.into();
    let edges = hypergraph_edges(hyperedges);
    let mut program = AscentProgram {
        edge: edges.into_iter().collect(),
        seed: vec![(start_node_id.clone(),)],
        ..Default::default()
    };
    program.run();

    let reachable_node_ids = program
        .reach
        .into_iter()
        .map(|(node_id,)| node_id)
        .filter(|node_id| node_id != &start_node_id)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();

    DatalogReachabilityWitnessV0 {
        schema_version: "0",
        product: "omena-reachability-datalog-lab.reachability-witness",
        start_node_id,
        reachable_node_ids,
        edge_count: program.edge.len(),
    }
}

pub fn selector_equal(left: &str, right: &str) -> bool {
    normalize_selector_for_equality(left) == normalize_selector_for_equality(right)
}

pub fn selector_equality_witness_v0(
    left: impl Into<String>,
    right: impl Into<String>,
) -> DatalogSelectorEqualityWitnessV0 {
    let left = left.into();
    let right = right.into();
    DatalogSelectorEqualityWitnessV0 {
        schema_version: "0",
        product: "omena-reachability-datalog-lab.selector-equality-witness",
        equal: selector_equal(left.as_str(), right.as_str()),
        left,
        right,
    }
}

fn hypergraph_edges(hyperedges: &[UnifiedHypergraphHyperedgeV0]) -> BTreeSet<(String, String)> {
    hyperedges
        .iter()
        .flat_map(|edge| {
            edge.tail_node_ids
                .iter()
                .map(|tail| (tail.clone(), edge.head_node_id.clone()))
        })
        .collect()
}

fn normalize_selector_for_equality(selector: &str) -> String {
    selector.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use omena_cross_file_summary::UnifiedHypergraphEdgeKindV0;

    #[test]
    fn datalog_reachability_derives_multi_hop_closure() {
        let hyperedges = vec![
            hyperedge("edge-a-b", "a", "b"),
            hyperedge("edge-b-c", "b", "c"),
            hyperedge("edge-c-d", "c", "d"),
        ];

        assert_eq!(
            datalog_reachable_node_ids("a", &hyperedges),
            vec!["b".to_string(), "c".to_string(), "d".to_string()]
        );
    }

    #[test]
    fn selector_equality_normalizes_whitespace_without_parsing_pseudo_elements() {
        let witness = selector_equality_witness_v0(".button::before", ".button::before");

        assert!(witness.equal);
        assert!(selector_equal(".button  ::before", ".button ::before"));
        assert!(!selector_equal(".button::before", ".button::after"));
    }

    fn hyperedge(id: &str, from: &str, to: &str) -> UnifiedHypergraphHyperedgeV0 {
        UnifiedHypergraphHyperedgeV0 {
            schema_version: "0",
            product: "test.hyperedge",
            layer_marker: "hypergraph-ifds",
            feature_gate: "hypergraph-ifds",
            hyperedge_id: id.to_string(),
            edge_kind: UnifiedHypergraphEdgeKindV0::ComposesExternal,
            source_summary_edge_id: id.to_string(),
            source_edge_kind: "composesExternal",
            source_status: "known",
            tail_node_ids: vec![from.to_string()],
            head_node_id: to.to_string(),
            order_significant_tail: false,
        }
    }
}
