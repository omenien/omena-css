use omena_resolver::{ModuleCanonicalIdV0, canonicalize_module_id_v0};
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum UnifiedHypergraphNodeKindV0 {
    StyleModule,
    SourceModule,
    StyleSymbol,
    SourceSymbol,
    ForeignSymbol,
}

impl UnifiedHypergraphNodeKindV0 {
    pub const fn as_wire_label(self) -> &'static str {
        match self {
            Self::StyleModule => "styleModule",
            Self::SourceModule => "sourceModule",
            Self::StyleSymbol => "styleSymbol",
            Self::SourceSymbol => "sourceSymbol",
            Self::ForeignSymbol => "foreignSymbol",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UnifiedHypergraphNodeKeyV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub node_kind: UnifiedHypergraphNodeKindV0,
    pub module: ModuleCanonicalIdV0,
    pub symbol_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UnifiedHypergraphNodeOriginV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub source_summary_edge_id: String,
    pub source_product: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UnifiedHypergraphNodeV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub node_id: String,
    pub key: UnifiedHypergraphNodeKeyV0,
    pub origins: Vec<UnifiedHypergraphNodeOriginV0>,
}

pub fn build_unified_hypergraph_node_id(
    node_kind: UnifiedHypergraphNodeKindV0,
    path: &str,
    symbol_name: Option<&str>,
) -> String {
    format!(
        "{}|{}|{}",
        node_kind.as_wire_label(),
        path,
        symbol_name.unwrap_or("-")
    )
}

pub fn build_unified_hypergraph_node(
    node_kind: UnifiedHypergraphNodeKindV0,
    path: impl Into<String>,
    symbol_name: Option<String>,
    source_summary_edge_id: String,
) -> UnifiedHypergraphNodeV0 {
    let path = path.into();
    let node_id =
        build_unified_hypergraph_node_id(node_kind, path.as_str(), symbol_name.as_deref());
    UnifiedHypergraphNodeV0 {
        schema_version: "0",
        product: "omena-query.unified-hypergraph-node",
        layer_marker: "hypergraph-ifds",
        feature_gate: "hypergraph-ifds",
        node_id,
        key: UnifiedHypergraphNodeKeyV0 {
            schema_version: "0",
            product: "omena-query.unified-hypergraph-node-key",
            layer_marker: "hypergraph-ifds",
            feature_gate: "hypergraph-ifds",
            node_kind,
            module: canonicalize_module_id_v0(path),
            symbol_name,
        },
        origins: vec![UnifiedHypergraphNodeOriginV0 {
            schema_version: "0",
            product: "omena-query.unified-hypergraph-node-origin",
            layer_marker: "hypergraph-ifds",
            feature_gate: "hypergraph-ifds",
            source_summary_edge_id,
            source_product: "omena-query.cross-file-summary",
        }],
    }
}
