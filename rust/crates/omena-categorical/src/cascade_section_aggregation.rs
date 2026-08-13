use serde::Serialize;

use crate::{
    CATEGORICAL_FEATURE_GATE_V0, CATEGORICAL_LAYER_MARKER_V0, CATEGORICAL_SCHEMA_VERSION_V0,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CascadeSectionAggregationV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    #[deprecated(
        since = "0.4.0",
        note = "use aggregation_id(); removal is not before 1.0 and requires downstream migration plus zero audited non-compatibility uses"
    )]
    pub cosheaf_id: String,
    pub sections: Vec<CascadeSectionV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CascadeSectionV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub object_id: String,
    pub declaration_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CascadeSectionAggregationWitnessV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    #[deprecated(
        since = "0.4.0",
        note = "use aggregation_id(); removal is not before 1.0 and requires downstream migration plus zero audited non-compatibility uses"
    )]
    pub cosheaf_id: String,
    pub colimit_object_id: String,
    pub compatible_section_count: usize,
    pub accepted: bool,
}

#[allow(deprecated)]
pub fn summarize_cascade_section_aggregation_v0(
    aggregation_id: impl Into<String>,
    compatible_section_count: usize,
) -> CascadeSectionAggregationWitnessV0 {
    build_cascade_section_aggregation_witness_with_legacy_field_v0(
        aggregation_id.into(),
        compatible_section_count,
    )
}

#[deprecated(
    since = "0.4.0",
    note = "constructs a retained serialized field; owned by omena-categorical maintainers; removal is not before 1.0 and requires downstream migration plus zero audited non-compatibility uses"
)]
#[allow(deprecated)]
fn build_cascade_section_aggregation_witness_with_legacy_field_v0(
    aggregation_id: String,
    compatible_section_count: usize,
) -> CascadeSectionAggregationWitnessV0 {
    CascadeSectionAggregationWitnessV0 {
        schema_version: CATEGORICAL_SCHEMA_VERSION_V0,
        product: "omena-categorical.cascade-section-aggregation-witness",
        layer_marker: CATEGORICAL_LAYER_MARKER_V0,
        feature_gate: CATEGORICAL_FEATURE_GATE_V0,
        cosheaf_id: aggregation_id,
        colimit_object_id: "cascade-outcome".to_string(),
        compatible_section_count,
        accepted: compatible_section_count > 0,
    }
}

impl CascadeSectionAggregationV0 {
    #[allow(deprecated)]
    pub fn aggregation_id(&self) -> &str {
        cascade_section_aggregation_id_from_legacy_field_v0(self)
    }
}

impl CascadeSectionAggregationWitnessV0 {
    #[allow(deprecated)]
    pub fn aggregation_id(&self) -> &str {
        cascade_section_aggregation_witness_id_from_legacy_field_v0(self)
    }
}

#[deprecated(
    since = "0.4.0",
    note = "compatibility field adapter owned by omena-categorical maintainers; removal is not before 1.0 and requires downstream migration plus zero audited non-compatibility uses"
)]
#[allow(deprecated)]
fn cascade_section_aggregation_id_from_legacy_field_v0(
    aggregation: &CascadeSectionAggregationV0,
) -> &str {
    aggregation.cosheaf_id.as_str()
}

#[deprecated(
    since = "0.4.0",
    note = "compatibility field adapter owned by omena-categorical maintainers; removal is not before 1.0 and requires downstream migration plus zero audited non-compatibility uses"
)]
#[allow(deprecated)]
fn cascade_section_aggregation_witness_id_from_legacy_field_v0(
    witness: &CascadeSectionAggregationWitnessV0,
) -> &str {
    witness.cosheaf_id.as_str()
}
