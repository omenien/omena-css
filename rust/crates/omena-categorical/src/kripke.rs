use serde::Serialize;

use crate::{
    CATEGORICAL_FEATURE_GATE_V0, CATEGORICAL_LAYER_MARKER_V0, CATEGORICAL_SCHEMA_VERSION_V0,
    OmegaCascadeTruthValueV0,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KripkeFrameV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub frame_id: String,
    pub worlds: Vec<String>,
    pub edges: Vec<KripkeEdgeV0>,
    pub valuations: Vec<KripkeValuationV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KripkeEdgeV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub from_world: String,
    pub to_world: String,
    pub relation: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KripkeValuationV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub world: String,
    pub atom: String,
    pub truth_value: OmegaCascadeTruthValueV0,
}

pub fn empty_s4_kripke_frame_v0(frame_id: impl Into<String>) -> KripkeFrameV0 {
    KripkeFrameV0 {
        schema_version: CATEGORICAL_SCHEMA_VERSION_V0,
        product: "omena-categorical.kripke-frame",
        layer_marker: CATEGORICAL_LAYER_MARKER_V0,
        feature_gate: CATEGORICAL_FEATURE_GATE_V0,
        frame_id: frame_id.into(),
        worlds: Vec::new(),
        edges: Vec::new(),
        valuations: Vec::new(),
    }
}
