use serde::Serialize;

use crate::{
    CATEGORICAL_FEATURE_GATE_V0, CATEGORICAL_LAYER_MARKER_V0, CATEGORICAL_SCHEMA_VERSION_V0,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CascadeCosheafV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub cosheaf_id: String,
    pub sections: Vec<CosheafSectionV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CosheafSectionV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub object_id: String,
    pub declaration_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CosheafColimitWitnessV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub cosheaf_id: String,
    pub colimit_object_id: String,
    pub compatible_section_count: usize,
    pub accepted: bool,
}

pub fn witness_cosheaf_colimit_v0(
    cosheaf_id: impl Into<String>,
    compatible_section_count: usize,
) -> CosheafColimitWitnessV0 {
    CosheafColimitWitnessV0 {
        schema_version: CATEGORICAL_SCHEMA_VERSION_V0,
        product: "omena-categorical.cosheaf-colimit-witness",
        layer_marker: CATEGORICAL_LAYER_MARKER_V0,
        feature_gate: CATEGORICAL_FEATURE_GATE_V0,
        cosheaf_id: cosheaf_id.into(),
        colimit_object_id: "cascade-outcome".to_string(),
        compatible_section_count,
        accepted: compatible_section_count > 0,
    }
}
