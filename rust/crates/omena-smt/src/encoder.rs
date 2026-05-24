use serde::Serialize;

use crate::{SMT_FEATURE_GATE_V0, SMT_LAYER_MARKER_V0, SMT_SCHEMA_VERSION_V0};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CanonicalSmtInputV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub obligation_id: String,
    pub l1_primitive: &'static str,
    pub canonical_terms: Vec<String>,
}

pub fn canonical_smt_input_v0(
    obligation_id: impl Into<String>,
    l1_primitive: &'static str,
    canonical_terms: Vec<String>,
) -> CanonicalSmtInputV0 {
    CanonicalSmtInputV0 {
        schema_version: SMT_SCHEMA_VERSION_V0,
        product: "omena-smt.canonical-input",
        layer_marker: SMT_LAYER_MARKER_V0,
        feature_gate: SMT_FEATURE_GATE_V0,
        obligation_id: obligation_id.into(),
        l1_primitive,
        canonical_terms,
    }
}
