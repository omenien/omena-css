use serde::Serialize;

use crate::OmenaQueryLinearProvenanceV0;

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
pub struct UnifiedHypergraphEdgeLabelV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub source_summary_edge_id: String,
    pub source_edge_kind: &'static str,
    pub source_status: &'static str,
    pub source: Option<String>,
    pub local_name: Option<String>,
    pub remote_name: Option<String>,
    pub target_names: Vec<String>,
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
    pub tail_node_ids: Vec<String>,
    pub head_node_id: String,
    pub label: UnifiedHypergraphEdgeLabelV0,
    pub lattice_effect: super::lattice::SummaryLatticeElementV0,
    pub order_significant_tail: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HypergraphIFDSProvenanceLabelV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub semiring_payload: HypergraphIFDSSemiringPayloadV0,
    pub legacy_labels: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase", tag = "kind")]
pub enum HypergraphIFDSSemiringPayloadV0 {
    Lin01 {
        linear_provenance: OmenaQueryLinearProvenanceV0,
    },
    Opaque {
        labels: Vec<&'static str>,
    },
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
    pub provenance: HypergraphIFDSProvenanceLabelV0,
}
