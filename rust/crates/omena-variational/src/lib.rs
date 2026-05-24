//! Variational cascade inference contracts.
//!
//! Stochastic evidence is isolated in this crate and disabled by default. V0
//! contracts serialize log quantities in bits; any nats arithmetic stays behind
//! the public boundary.

pub mod hover;
pub mod unit;

use omena_abstract_value::AbstractClassValueProvenanceV0;
use serde::Serialize;

pub const VARIATIONAL_SCHEMA_VERSION_V0: &str = "0";
pub const VARIATIONAL_LAYER_MARKER_V0: &str = "variational-cascade";
pub const VARIATIONAL_FEATURE_GATE_V0: &str = "variational";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum DesignerIntentPosteriorModeV0 {
    VciFormal,
    PcnHierarchical,
    Fallback,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum PatternIntentV0 {
    Bem,
    Utility,
    Atomic,
    Hybrid,
    AdHoc,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DesignerIntentPosteriorV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub mode: DesignerIntentPosteriorModeV0,
    pub selector_name: String,
    pub scores: Vec<DesignerIntentScoreV0>,
    pub enabled_by_default: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DesignerIntentScoreV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub intent: PatternIntentV0,
    pub log_probability_bits: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VariationalFreeEnergyV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub complexity_bits: f64,
    pub accuracy_bits: f64,
    pub free_energy_bits: f64,
    pub public_framing: &'static str,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PatternPriorV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub prior_kind: &'static str,
    pub concentration_bits: f64,
    pub calibration: PatternPriorCalibrationV0,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PatternPriorCalibrationV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub corpus_fingerprint: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EmissionLikelihoodV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub selector_name: String,
    pub factors: Vec<EmissionLikelihoodFactorV0>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EmissionLikelihoodFactorV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub factor_name: &'static str,
    pub log_likelihood_bits: f64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProvenancePosteriorAnnotationV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub provenance: Option<AbstractClassValueProvenanceV0>,
    pub annotation_id: String,
    pub mutates_existing_provenance_enum: bool,
}

pub fn summarize_variational_default_posterior_v0(
    selector_name: impl Into<String>,
) -> DesignerIntentPosteriorV0 {
    DesignerIntentPosteriorV0 {
        schema_version: VARIATIONAL_SCHEMA_VERSION_V0,
        product: "omena-variational.designer-intent-posterior",
        layer_marker: VARIATIONAL_LAYER_MARKER_V0,
        feature_gate: VARIATIONAL_FEATURE_GATE_V0,
        mode: DesignerIntentPosteriorModeV0::Fallback,
        selector_name: selector_name.into(),
        scores: vec![DesignerIntentScoreV0 {
            schema_version: VARIATIONAL_SCHEMA_VERSION_V0,
            product: "omena-variational.designer-intent-score",
            layer_marker: VARIATIONAL_LAYER_MARKER_V0,
            feature_gate: VARIATIONAL_FEATURE_GATE_V0,
            intent: PatternIntentV0::AdHoc,
            log_probability_bits: 0.0,
        }],
        enabled_by_default: false,
    }
}

pub fn variational_free_energy_v0(
    complexity_bits: f64,
    accuracy_bits: f64,
) -> VariationalFreeEnergyV0 {
    VariationalFreeEnergyV0 {
        schema_version: VARIATIONAL_SCHEMA_VERSION_V0,
        product: "omena-variational.free-energy",
        layer_marker: VARIATIONAL_LAYER_MARKER_V0,
        feature_gate: VARIATIONAL_FEATURE_GATE_V0,
        complexity_bits,
        accuracy_bits,
        free_energy_bits: complexity_bits - accuracy_bits,
        public_framing: "Champion 2024 ELBO/VFE framing",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn posterior_is_default_off_and_bits_only() {
        let posterior = summarize_variational_default_posterior_v0(".button");
        assert_eq!(posterior.schema_version, "0");
        assert_eq!(posterior.layer_marker, "variational-cascade");
        assert!(!posterior.enabled_by_default);
        assert_eq!(unit::nats_to_bits(std::f64::consts::LN_2), 1.0);
    }
}
