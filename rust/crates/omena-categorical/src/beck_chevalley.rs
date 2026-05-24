use omena_cascade::LayerFlattenProofV0;
use serde::Serialize;

use crate::{
    CATEGORICAL_FEATURE_GATE_V0, CATEGORICAL_LAYER_MARKER_V0, CATEGORICAL_SCHEMA_VERSION_V0,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BeckChevalleyDatumV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub square_id: String,
    pub origin_preserved: bool,
    pub layer_order_preserved: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OriginInversionMorphismV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub layer_name: Option<String>,
    pub important_declarations_invert_origin: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BeckChevalleyCheckV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub datum: BeckChevalleyDatumV0,
    pub origin_inversion: OriginInversionMorphismV0,
    pub accepted: bool,
    pub witness: String,
}

pub fn beck_chevalley_from_layer_flatten_proof_v0(
    proof: &LayerFlattenProofV0,
) -> BeckChevalleyCheckV0 {
    BeckChevalleyCheckV0 {
        schema_version: CATEGORICAL_SCHEMA_VERSION_V0,
        product: "omena-categorical.beck-chevalley-check",
        layer_marker: CATEGORICAL_LAYER_MARKER_V0,
        feature_gate: CATEGORICAL_FEATURE_GATE_V0,
        datum: BeckChevalleyDatumV0 {
            schema_version: CATEGORICAL_SCHEMA_VERSION_V0,
            product: "omena-categorical.beck-chevalley-datum",
            layer_marker: CATEGORICAL_LAYER_MARKER_V0,
            feature_gate: CATEGORICAL_FEATURE_GATE_V0,
            square_id: "layer-flatten-origin-square".to_string(),
            origin_preserved: proof.provenance_preserved,
            layer_order_preserved: proof.accepted,
        },
        origin_inversion: OriginInversionMorphismV0 {
            schema_version: CATEGORICAL_SCHEMA_VERSION_V0,
            product: "omena-categorical.origin-inversion-morphism",
            layer_marker: CATEGORICAL_LAYER_MARKER_V0,
            feature_gate: CATEGORICAL_FEATURE_GATE_V0,
            layer_name: proof.layer_name.clone(),
            important_declarations_invert_origin: proof
                .blocked_reason
                .is_some_and(|reason| reason.contains("important")),
        },
        accepted: proof.accepted,
        witness: proof.cascade_safe_witness.clone(),
    }
}
