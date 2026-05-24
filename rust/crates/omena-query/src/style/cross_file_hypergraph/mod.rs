#![cfg_attr(not(feature = "hypergraph-ifds"), allow(dead_code))]

use std::collections::BTreeMap;

use serde::Serialize;

use crate::{OmenaQueryCrossFileSummaryEdgeV0, OmenaQueryCrossFileSummaryV0};

mod cycle;
mod edge;
mod lattice;
mod node;
mod reachability;

pub use cycle::*;
pub use edge::*;
pub use lattice::*;
pub use node::*;
pub use reachability::*;
pub(in crate::style) use reachability::{
    HypergraphClosureOptions, HypergraphClosurePath, collect_hypergraph_transitive_closure_paths,
    collect_hypergraph_transitive_closure_paths_with_options,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryUnifiedCrossFileHypergraphV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub status: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub source_product: &'static str,
    pub node_count: usize,
    pub hyperedge_count: usize,
    pub summary_edge_count: usize,
    pub scc_count: usize,
    pub cycle_count: usize,
    pub projection_edge_ids: Vec<String>,
    pub nodes: Vec<UnifiedHypergraphNodeV0>,
    pub hyperedges: Vec<UnifiedHypergraphHyperedgeV0>,
    pub summary_edges: Vec<HypergraphIFDSSummaryEdgeV0>,
    pub strongly_connected_components: Vec<UnifiedHypergraphSccV0>,
    pub cycles: Vec<UnifiedHypergraphCycleV0>,
    pub capabilities: UnifiedHypergraphCapabilitiesV0,
    pub diagnostics: UnifiedHypergraphDiagnosticsSummaryV0,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UnifiedHypergraphCapabilitiesV0 {
    pub p1_type_introduction_ready: bool,
    pub p2_adjacency_projection_ready: bool,
    pub p3_scc_unification_ready: bool,
    pub p4_summary_edge_tabulation_ready: bool,
    pub p5_projection_helper_ready: bool,
    pub p6_closure_body_switch_over_ready: bool,
    pub p7_v0_publication_ready: bool,
    pub batch_connectivity_oracle_ready: bool,
    pub streaming_oracle_wire_compatible: bool,
    pub composes_tail_ordering_uses_vec: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UnifiedHypergraphDiagnosticsSummaryV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub mixed_kind_scc_count: usize,
    pub unresolved_edge_count: usize,
}

pub fn summarize_omena_query_unified_cross_file_hypergraph(
    summary: &OmenaQueryCrossFileSummaryV0,
) -> OmenaQueryUnifiedCrossFileHypergraphV0 {
    let mut builder = UnifiedCrossFileHypergraphBuilder::default();
    for edge in &summary.edges {
        builder.add_summary_edge(edge);
    }
    builder.finish(summary)
}

#[derive(Default)]
struct UnifiedCrossFileHypergraphBuilder {
    nodes: BTreeMap<String, UnifiedHypergraphNodeV0>,
    hyperedges: Vec<UnifiedHypergraphHyperedgeV0>,
    summary_edges: Vec<HypergraphIFDSSummaryEdgeV0>,
}

impl UnifiedCrossFileHypergraphBuilder {
    fn add_summary_edge(&mut self, edge: &OmenaQueryCrossFileSummaryEdgeV0) {
        let edge_kind = unified_edge_kind_for_summary_edge(edge);
        let from_node = endpoint_node(edge, EndpointSide::From);
        let to_node = endpoint_node(edge, EndpointSide::Target);
        let mut tail_node_ids = if edge_kind.is_order_significant() && !edge.target_names.is_empty()
        {
            edge.target_names
                .iter()
                .map(|target_name| {
                    let target_path = edge
                        .target_path
                        .as_deref()
                        .unwrap_or(edge.from_path.as_str())
                        .to_string();
                    let node = build_unified_hypergraph_node(
                        UnifiedHypergraphNodeKindV0::StyleSymbol,
                        target_path,
                        Some(target_name.clone()),
                        edge.edge_id.clone(),
                    );
                    let node_id = node.node_id.clone();
                    self.nodes.entry(node_id.clone()).or_insert(node);
                    node_id
                })
                .collect::<Vec<_>>()
        } else {
            vec![from_node.node_id.clone()]
        };
        if tail_node_ids.is_empty() {
            tail_node_ids.push(from_node.node_id.clone());
        }

        let from_node_id = from_node.node_id.clone();
        let to_node_id = to_node.node_id.clone();
        self.nodes
            .entry(from_node.node_id.clone())
            .or_insert(from_node);
        self.nodes.entry(to_node.node_id.clone()).or_insert(to_node);

        let hyperedge_id = format!(
            "hyperedge:{}|{}|{}",
            edge_kind.as_wire_label(),
            edge.edge_id,
            tail_node_ids.join(">")
        );
        let hyperedge = UnifiedHypergraphHyperedgeV0 {
            schema_version: "0",
            product: "omena-query.unified-hypergraph-hyperedge",
            layer_marker: "hypergraph-ifds",
            feature_gate: "hypergraph-ifds",
            hyperedge_id: hyperedge_id.clone(),
            edge_kind,
            tail_node_ids,
            head_node_id: to_node_id.clone(),
            label: UnifiedHypergraphEdgeLabelV0 {
                schema_version: "0",
                product: "omena-query.unified-hypergraph-edge-label",
                layer_marker: "hypergraph-ifds",
                feature_gate: "hypergraph-ifds",
                source_summary_edge_id: edge.edge_id.clone(),
                source_edge_kind: edge.edge_kind,
                source_status: edge.status,
                source: edge.source.clone(),
                local_name: edge.local_name.clone(),
                remote_name: edge.remote_name.clone(),
                target_names: edge.target_names.clone(),
            },
            lattice_effect: SummaryLatticeElementV0::from_status(edge.status),
            order_significant_tail: edge_kind.is_order_significant(),
        };
        self.hyperedges.push(hyperedge);

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
            provenance: HypergraphIFDSProvenanceLabelV0 {
                schema_version: "0",
                product: "omena-query.hypergraph-ifds-provenance-label",
                layer_marker: "hypergraph-ifds",
                feature_gate: "hypergraph-ifds",
                semiring_payload: HypergraphIFDSSemiringPayloadV0::Lin01 {
                    linear_provenance: edge.linear_provenance.clone(),
                },
                legacy_labels: edge.provenance.clone(),
            },
        });
    }

    fn finish(
        mut self,
        summary: &OmenaQueryCrossFileSummaryV0,
    ) -> OmenaQueryUnifiedCrossFileHypergraphV0 {
        self.hyperedges
            .sort_by_key(|edge| edge.hyperedge_id.clone());
        let summary_edges =
            tabulate_hypergraph_ifds_summary_edges(&self.hyperedges, self.summary_edges);
        let mut nodes = self.nodes.into_values().collect::<Vec<_>>();
        nodes.sort_by_key(|node| node.node_id.clone());
        let (strongly_connected_components, cycles) =
            summarize_unified_hypergraph_sccs(&nodes, &self.hyperedges);
        let projection_edge_ids = summary_edges
            .iter()
            .map(|edge| edge.projection_edge_id.clone())
            .collect::<Vec<_>>();
        let unresolved_edge_count = summary_edges
            .iter()
            .filter(|edge| edge.status != "resolved" && edge.status != "reachable")
            .count();
        let mixed_kind_scc_count = strongly_connected_components
            .iter()
            .filter(|scc| scc.mixed_kind)
            .count();

        OmenaQueryUnifiedCrossFileHypergraphV0 {
            schema_version: "0",
            product: "omena-query.unified-cross-file-hypergraph",
            status: "hypergraphIfdsProjectionSeed",
            layer_marker: "hypergraph-ifds",
            feature_gate: "hypergraph-ifds",
            source_product: summary.product,
            node_count: nodes.len(),
            hyperedge_count: self.hyperedges.len(),
            summary_edge_count: summary_edges.len(),
            scc_count: strongly_connected_components.len(),
            cycle_count: cycles.len(),
            projection_edge_ids,
            nodes,
            hyperedges: self.hyperedges,
            summary_edges,
            strongly_connected_components,
            cycles,
            capabilities: UnifiedHypergraphCapabilitiesV0 {
                p1_type_introduction_ready: true,
                p2_adjacency_projection_ready: true,
                p3_scc_unification_ready: true,
                p4_summary_edge_tabulation_ready: true,
                p5_projection_helper_ready: true,
                p6_closure_body_switch_over_ready: true,
                p7_v0_publication_ready: true,
                batch_connectivity_oracle_ready: true,
                streaming_oracle_wire_compatible: true,
                composes_tail_ordering_uses_vec: true,
            },
            diagnostics: UnifiedHypergraphDiagnosticsSummaryV0 {
                schema_version: "0",
                product: "omena-query.unified-hypergraph-diagnostics-summary",
                layer_marker: "hypergraph-ifds",
                feature_gate: "hypergraph-ifds",
                mixed_kind_scc_count,
                unresolved_edge_count,
            },
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum EndpointSide {
    From,
    Target,
}

fn endpoint_node(
    edge: &OmenaQueryCrossFileSummaryEdgeV0,
    side: EndpointSide,
) -> UnifiedHypergraphNodeV0 {
    let (kind, path, symbol) = match side {
        EndpointSide::From => (
            node_kind_for_summary_kind(edge.from_kind, false),
            edge.from_path.clone(),
            edge.owner_selector_name
                .clone()
                .or_else(|| edge.local_name.clone()),
        ),
        EndpointSide::Target => (
            node_kind_for_summary_kind(edge.target_kind.unwrap_or(edge.from_kind), true),
            edge.target_path
                .clone()
                .unwrap_or_else(|| edge.from_path.clone()),
            edge.remote_name
                .clone()
                .or_else(|| edge.target_names.first().cloned()),
        ),
    };
    build_unified_hypergraph_node(kind, path, symbol, edge.edge_id.clone())
}

fn node_kind_for_summary_kind(kind: &str, target: bool) -> UnifiedHypergraphNodeKindV0 {
    match (kind, target) {
        ("style", false) => UnifiedHypergraphNodeKindV0::StyleModule,
        ("style", true) => UnifiedHypergraphNodeKindV0::StyleSymbol,
        ("source", false) => UnifiedHypergraphNodeKindV0::SourceModule,
        ("source", true) => UnifiedHypergraphNodeKindV0::SourceSymbol,
        _ => UnifiedHypergraphNodeKindV0::ForeignSymbol,
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

#[cfg(all(test, feature = "hypergraph-ifds"))]
mod tests {
    use std::collections::BTreeSet;

    use crate::{
        OmenaQueryCrossFileSummaryCapabilitiesV0, OmenaQueryCrossFileSummaryEdgeKindCountV0,
        OmenaQueryCrossFileSummaryEdgeV0, OmenaQueryCrossFileSummaryV0,
        OmenaQuerySourceDocumentInputV0, OmenaQueryStyleSourceInputV0,
        summarize_omena_query_linear_provenance,
        summarize_omena_query_workspace_cross_file_summary,
    };

    use super::*;

    #[test]
    fn unified_hypergraph_projects_summary_edges_byte_equal_by_id() {
        let style_sources = vec![
            OmenaQueryStyleSourceInputV0 {
                style_path: "/tmp/base.module.scss".to_string(),
                style_source: ".base { color: red; }".to_string(),
            },
            OmenaQueryStyleSourceInputV0 {
                style_path: "/tmp/Button.module.scss".to_string(),
                style_source:
                    ".root { composes: base from \"./base.module.scss\"; color: var(--brand); }"
                        .to_string(),
            },
        ];
        let source_documents = vec![OmenaQuerySourceDocumentInputV0 {
            source_path: "/tmp/Button.tsx".to_string(),
            source_source: "import styles from './Button.module.scss';\nconst cls = styles.root;\n"
                .to_string(),
        }];
        let summary = summarize_omena_query_workspace_cross_file_summary(
            style_sources.as_slice(),
            source_documents.as_slice(),
            &[],
        );
        let hypergraph = summarize_omena_query_unified_cross_file_hypergraph(&summary);

        assert_eq!(hypergraph.schema_version, "0");
        assert_eq!(hypergraph.layer_marker, "hypergraph-ifds");
        assert_eq!(hypergraph.feature_gate, "hypergraph-ifds");
        assert!(hypergraph.capabilities.p1_type_introduction_ready);
        assert!(hypergraph.capabilities.p5_projection_helper_ready);
        assert!(hypergraph.capabilities.p6_closure_body_switch_over_ready);
        assert_eq!(hypergraph.summary_edge_count, summary.summary_edge_count);

        let projected = hypergraph
            .projection_edge_ids
            .iter()
            .map(String::as_str)
            .collect::<BTreeSet<_>>();
        let original = summary
            .edges
            .iter()
            .map(|edge| edge.edge_id.as_str())
            .collect::<BTreeSet<_>>();
        assert_eq!(projected, original);
    }

    #[test]
    fn composes_hyperedge_tail_preserves_target_name_order() {
        let summary = OmenaQueryCrossFileSummaryV0 {
            schema_version: "0",
            product: "omena-query.cross-file-summary",
            status: "summaryEdgeSeed",
            summary_scope: "test",
            style_count: 2,
            summary_edge_count: 1,
            edge_kind_counts: vec![OmenaQueryCrossFileSummaryEdgeKindCountV0 {
                edge_kind: "cssModulesComposesImport",
                count: 1,
            }],
            summary_hash: "test".to_string(),
            edges: vec![OmenaQueryCrossFileSummaryEdgeV0 {
                edge_id: "cssModulesComposesImport|from:/tmp/Button.module.scss|target:/tmp/base.module.scss|names:a,b,c".to_string(),
                edge_kind: "cssModulesComposesImport",
                from_kind: "style",
                from_path: "/tmp/Button.module.scss".to_string(),
                target_kind: Some("style"),
                target_path: Some("/tmp/base.module.scss".to_string()),
                source: Some("./base.module.scss".to_string()),
                owner_selector_name: Some("root".to_string()),
                local_name: None,
                remote_name: None,
                target_names: vec!["a".to_string(), "b".to_string(), "c".to_string()],
                status: "resolved",
                provenance: vec![
                    "omena-query.css-modules-cross-file-resolution",
                    "omena-parser.css-module-composes-facts",
                ],
                linear_provenance: summarize_omena_query_linear_provenance(&[
                    "omena-query.css-modules-cross-file-resolution",
                    "omena-parser.css-module-composes-facts",
                ]),
            }],
            capabilities: OmenaQueryCrossFileSummaryCapabilitiesV0 {
                css_modules_composes_edges_ready: true,
                css_modules_value_edges_ready: false,
                css_modules_icss_edges_ready: false,
                sass_module_edges_ready: false,
                style_design_token_reference_edges_ready: false,
                source_selector_reference_edges_ready: false,
                stable_summary_hash_ready: true,
                linear_provenance_ready: true,
                linear_provenance_round_trip_ready: true,
            },
            next_priorities: Vec::new(),
        };
        let hypergraph = summarize_omena_query_unified_cross_file_hypergraph(&summary);
        let composes_edges = hypergraph
            .hyperedges
            .iter()
            .filter(|edge| edge.edge_kind == UnifiedHypergraphEdgeKindV0::ComposesExternal)
            .collect::<Vec<_>>();
        assert_eq!(composes_edges.len(), 1);
        let composes = composes_edges[0];

        assert!(composes.order_significant_tail);
        assert_eq!(composes.label.target_names, vec!["a", "b", "c"]);
        assert_eq!(composes.tail_node_ids.len(), 3);
        assert!(composes.tail_node_ids[0].ends_with("|a"));
        assert!(composes.tail_node_ids[1].ends_with("|b"));
        assert!(composes.tail_node_ids[2].ends_with("|c"));
    }

    #[test]
    fn p6_switch_over_projects_all_closure_edge_kinds() {
        let style_sources = vec![
            OmenaQueryStyleSourceInputV0 {
                style_path: "/tmp/App.module.scss".to_string(),
                style_source: "@use \"./tokens\"; .a { composes: b; } .b { composes: c; } .c {} @value a: b; @value b: c; @value c: red; :export { alpha: beta; beta: gamma; gamma: red; }".to_string(),
            },
            OmenaQueryStyleSourceInputV0 {
                style_path: "/tmp/_tokens.scss".to_string(),
                style_source: "@forward \"./palette\";".to_string(),
            },
            OmenaQueryStyleSourceInputV0 {
                style_path: "/tmp/_palette.scss".to_string(),
                style_source: ".palette { color: red; }".to_string(),
            },
        ];
        let summary =
            summarize_omena_query_workspace_cross_file_summary(style_sources.as_slice(), &[], &[]);
        let edge_kinds = summary
            .edge_kind_counts
            .iter()
            .map(|entry| entry.edge_kind)
            .collect::<BTreeSet<_>>();

        assert!(edge_kinds.contains("cssModulesComposesClosure"));
        assert!(edge_kinds.contains("cssModulesValueClosure"));
        assert!(edge_kinds.contains("cssModulesIcssClosure"));
        assert!(edge_kinds.contains("sassModuleGraphClosure"));

        let hypergraph = summarize_omena_query_unified_cross_file_hypergraph(&summary);

        assert!(hypergraph.capabilities.p1_type_introduction_ready);
        assert!(hypergraph.capabilities.p2_adjacency_projection_ready);
        assert!(hypergraph.capabilities.p3_scc_unification_ready);
        assert!(hypergraph.capabilities.p4_summary_edge_tabulation_ready);
        assert!(hypergraph.capabilities.p5_projection_helper_ready);
        assert!(hypergraph.capabilities.p6_closure_body_switch_over_ready);
        assert!(hypergraph.capabilities.p7_v0_publication_ready);
        assert_eq!(hypergraph.summary_edge_count, summary.summary_edge_count);
    }
}
