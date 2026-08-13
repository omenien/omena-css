use serde::Serialize;

use crate::{
    CATEGORICAL_FEATURE_GATE_V0, CATEGORICAL_LAYER_MARKER_V0, CATEGORICAL_SCHEMA_VERSION_V0,
    CascadeSectionAggregationWitnessV0, summarize_cascade_section_aggregation_v0,
};

#[deprecated(
    since = "0.4.0",
    note = "use CascadeSectionAggregationV0; compatibility owner: omena-categorical maintainers; removal is not before 1.0 and requires downstream migration plus zero audited non-compatibility uses"
)]
#[allow(deprecated)]
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

#[deprecated(
    since = "0.4.0",
    note = "use CascadeSectionV0; compatibility owner: omena-categorical maintainers; removal is not before 1.0 and requires downstream migration plus zero audited non-compatibility uses"
)]
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

#[deprecated(
    since = "0.4.0",
    note = "use CascadeSectionAggregationWitnessV0; compatibility owner: omena-categorical maintainers; removal is not before 1.0 and requires downstream migration plus zero audited non-compatibility uses"
)]
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

#[deprecated(
    since = "0.4.0",
    note = "use summarize_cascade_section_aggregation_v0; compatibility owner: omena-categorical maintainers; removal is not before 1.0 and requires downstream migration plus zero audited non-compatibility uses"
)]
#[allow(deprecated)]
pub fn witness_cosheaf_colimit_v0(
    cosheaf_id: impl Into<String>,
    compatible_section_count: usize,
) -> CosheafColimitWitnessV0 {
    compatibility_witness_from_canonical_v0(summarize_cascade_section_aggregation_v0(
        cosheaf_id,
        compatible_section_count,
    ))
}

#[deprecated(
    since = "0.4.0",
    note = "compatibility owner: omena-categorical maintainers; removal is not before 1.0 and requires downstream migration plus zero audited non-compatibility uses"
)]
#[allow(deprecated)]
fn compatibility_witness_from_canonical_v0(
    witness: CascadeSectionAggregationWitnessV0,
) -> CosheafColimitWitnessV0 {
    CosheafColimitWitnessV0 {
        schema_version: CATEGORICAL_SCHEMA_VERSION_V0,
        product: "omena-categorical.cosheaf-colimit-witness",
        layer_marker: CATEGORICAL_LAYER_MARKER_V0,
        feature_gate: CATEGORICAL_FEATURE_GATE_V0,
        cosheaf_id: witness.cosheaf_id,
        colimit_object_id: witness.colimit_object_id,
        compatible_section_count: witness.compatible_section_count,
        accepted: witness.accepted,
    }
}
