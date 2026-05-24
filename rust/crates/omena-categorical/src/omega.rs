use omena_cascade::CascadeOutcome;
use serde::Serialize;

use crate::{
    CATEGORICAL_FEATURE_GATE_V0, CATEGORICAL_LAYER_MARKER_V0, CATEGORICAL_SCHEMA_VERSION_V0,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum OmegaCascadeTruthValueV0 {
    Open,
    Boundary,
    Closed,
    Full,
}

impl OmegaCascadeTruthValueV0 {
    pub fn from_outcome(outcome: &CascadeOutcome) -> Self {
        match outcome {
            CascadeOutcome::Definite { .. } => Self::Closed,
            CascadeOutcome::RankedSet(_) => Self::Boundary,
            CascadeOutcome::Inherit => Self::Open,
            CascadeOutcome::Top => Self::Full,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmegaTruthMappingV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub cascade_outcome_kind: &'static str,
    pub truth_value: OmegaCascadeTruthValueV0,
}

pub fn omega_truth_mapping_v0(
    cascade_outcome_kind: &'static str,
    truth_value: OmegaCascadeTruthValueV0,
) -> OmegaTruthMappingV0 {
    OmegaTruthMappingV0 {
        schema_version: CATEGORICAL_SCHEMA_VERSION_V0,
        product: "omena-categorical.omega-truth-mapping",
        layer_marker: CATEGORICAL_LAYER_MARKER_V0,
        feature_gate: CATEGORICAL_FEATURE_GATE_V0,
        cascade_outcome_kind,
        truth_value,
    }
}
