#![cfg_attr(not(feature = "hypergraph-ifds"), allow(dead_code))]

use std::collections::BTreeSet;

use serde::Serialize;

use crate::{OmenaQueryCrossFileSummaryEdgeV0, OmenaQueryCrossFileSummaryV0};

mod edge;
mod reachability;
#[cfg(feature = "hypergraph-ifds")]
mod scc;

pub use edge::*;
pub use reachability::*;
pub(in crate::style) use reachability::{
    HypergraphClosureMode, HypergraphClosurePath, collect_hypergraph_transitive_closure_paths,
    collect_hypergraph_transitive_closure_paths_with_mode,
};
#[cfg(feature = "hypergraph-ifds")]
pub use scc::*;

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

pub fn summarize_omena_query_unified_cross_file_hypergraph(
    summary: &OmenaQueryCrossFileSummaryV0,
) -> OmenaQueryUnifiedCrossFileHypergraphV0 {
    let mut builder = UnifiedCrossFileHypergraphBuilder::default();
    for edge in &summary.edges {
        builder.add_summary_edge(edge);
    }
    builder.finish()
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
        "cssModulesValueImport" | "cssModulesValueClosure" | "value" => {
            UnifiedHypergraphEdgeKindV0::Value
        }
        "cssModulesIcssImport" | "cssModulesIcssClosure" | "icss" => {
            UnifiedHypergraphEdgeKindV0::Icss
        }
        _ => UnifiedHypergraphEdgeKindV0::ForeignReference,
    }
}
