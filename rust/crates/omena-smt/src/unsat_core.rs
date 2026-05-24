use serde::Serialize;

use crate::{SMT_FEATURE_GATE_V0, SMT_LAYER_MARKER_V0, SMT_SCHEMA_VERSION_V0};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CascadeUnsatCoreLabelV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub label: String,
    pub l1_primitive: &'static str,
}

pub fn cascade_unsat_core_label_v0(
    label: impl Into<String>,
    l1_primitive: &'static str,
) -> CascadeUnsatCoreLabelV0 {
    CascadeUnsatCoreLabelV0 {
        schema_version: SMT_SCHEMA_VERSION_V0,
        product: "omena-smt.unsat-core-label",
        layer_marker: SMT_LAYER_MARKER_V0,
        feature_gate: SMT_FEATURE_GATE_V0,
        label: label.into(),
        l1_primitive,
    }
}
