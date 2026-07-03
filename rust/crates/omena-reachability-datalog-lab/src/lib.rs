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
    relation fact_edge(String, String, String);
    relation seed(String);
    relation fact_seed(String, String);
    relation reach(String);
    relation fact(String, String);

    reach(x.clone()) <-- seed(x);
    reach(y.clone()) <-- reach(x), edge(x, y);
    fact(x.clone(), value.clone()) <-- fact_seed(x, value);
    fact(y.clone(), widen_datalog_fact_value_key(value, edge_kind, y)) <-- fact(x, value), fact_edge(x, y, edge_kind);
}

pub fn datalog_reachable_node_ids(
    start_node_id: impl Into<String>,
    hyperedges: &[UnifiedHypergraphHyperedgeV0],
) -> Vec<String> {
    datalog_reachability_witness_v0(start_node_id, hyperedges).reachable_node_ids
}

pub fn datalog_fact_keys_v0(
    start_node_id: impl Into<String>,
    seed_value_key: impl Into<String>,
    hyperedges: &[UnifiedHypergraphHyperedgeV0],
) -> Vec<String> {
    let start_node_id = start_node_id.into();
    let seed_value_key = seed_value_key.into();
    let edges = hypergraph_edges(hyperedges);
    let fact_edges = hypergraph_fact_edges(hyperedges);
    let mut program = AscentProgram {
        edge: edges.into_iter().collect(),
        fact_edge: fact_edges.into_iter().collect(),
        seed: vec![(start_node_id.clone(),)],
        fact_seed: vec![(start_node_id, seed_value_key)],
        ..Default::default()
    };
    program.run();

    program
        .fact
        .into_iter()
        .map(|(node_id, value_key)| format!("{node_id}|{value_key}"))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
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

fn hypergraph_fact_edges(
    hyperedges: &[UnifiedHypergraphHyperedgeV0],
) -> BTreeSet<(String, String, String)> {
    hyperedges
        .iter()
        .flat_map(|edge| {
            edge.tail_node_ids.iter().map(|tail| {
                (
                    tail.clone(),
                    edge.head_node_id.clone(),
                    edge.edge_kind.as_wire_label().to_string(),
                )
            })
        })
        .collect()
}

fn widen_datalog_fact_value_key(value_key: &str, edge_kind: &str, head_node_id: &str) -> String {
    if !matches!(
        edge_kind,
        "composesLocal" | "composesGlobal" | "composesExternal"
    ) {
        return value_key.to_string();
    }

    let head_token = class_token_from_node_id(head_node_id);
    if let Some(value) = value_key.strip_prefix("exact:") {
        return finite_value_key([value.to_string(), head_token]);
    }
    if let Some(values) = value_key.strip_prefix("finiteSet:") {
        return finite_value_key(
            values
                .split(',')
                .filter(|value| !value.is_empty())
                .map(str::to_string)
                .chain([head_token]),
        );
    }
    value_key.to_string()
}

fn finite_value_key(values: impl IntoIterator<Item = String>) -> String {
    let mut values = values.into_iter().collect::<Vec<_>>();
    values.sort();
    values.dedup();
    format!("finiteSet:{}", values.join(","))
}

fn class_token_from_node_id(node_id: &str) -> String {
    node_id
        .rsplit(['/', '#', '.', ':', '|'])
        .find(|segment| !segment.is_empty())
        .unwrap_or(node_id)
        .to_string()
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
    fn datalog_fact_keys_carry_compose_widened_values() {
        let hyperedges = vec![
            hyperedge(
                "edge-a-b",
                "styleModule|/workspace/Button.module.scss|button",
                "styleSymbol|/workspace/Button.module.scss|base",
            ),
            hyperedge(
                "edge-b-c",
                "styleSymbol|/workspace/Button.module.scss|base",
                "styleSymbol|/workspace/theme.module.scss|primary",
            ),
        ];

        assert_eq!(
            datalog_fact_keys_v0(
                "styleModule|/workspace/Button.module.scss|button",
                "exact:btn",
                &hyperedges,
            ),
            vec![
                "styleModule|/workspace/Button.module.scss|button|exact:btn".to_string(),
                "styleSymbol|/workspace/Button.module.scss|base|finiteSet:base,btn".to_string(),
                "styleSymbol|/workspace/theme.module.scss|primary|finiteSet:base,btn,primary"
                    .to_string(),
            ]
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
