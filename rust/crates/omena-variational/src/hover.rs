use serde::Serialize;

use crate::{
    VARIATIONAL_FEATURE_GATE_V0, VARIATIONAL_LAYER_MARKER_V0, VARIATIONAL_SCHEMA_VERSION_V0,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VariationalHoverBudgetV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub total_budget_ms: u64,
    pub fragment_budget_ms: u64,
    pub enabled_by_default: bool,
}

pub fn variational_hover_budget_v0() -> VariationalHoverBudgetV0 {
    VariationalHoverBudgetV0 {
        schema_version: VARIATIONAL_SCHEMA_VERSION_V0,
        product: "omena-variational.hover-budget",
        layer_marker: VARIATIONAL_LAYER_MARKER_V0,
        feature_gate: VARIATIONAL_FEATURE_GATE_V0,
        total_budget_ms: 25,
        fragment_budget_ms: 6,
        enabled_by_default: false,
    }
}
