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

impl PatternIntentV0 {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Bem => "bem",
            Self::Utility => "utility",
            Self::Atomic => "atomic",
            Self::Hybrid => "hybrid",
            Self::AdHoc => "adHoc",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum PatternPriorKindV0 {
    UniformDirichlet,
    CorpusCalibrated,
    Bespoke,
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
    pub kind: PatternPriorKindV0,
    pub prior_kind: &'static str,
    pub dirichlet_alpha: Vec<PatternPriorAlphaV0>,
    pub concentration_bits: f64,
    pub corpus_calibration: Option<PatternPriorCalibrationV0>,
    pub rg_universality_class: Option<RgUniversalityClassRefV0>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PatternPriorAlphaV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub intent: PatternIntentV0,
    pub alpha_bits: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PatternPriorCalibrationV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub corpus_fingerprint: String,
    pub axis_a_schema_version: &'static str,
    pub fixture_count: usize,
    pub generated_at_epoch: u64,
    pub human_review_gate_passed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RgUniversalityClassRefV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub class_label: &'static str,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EmissionLikelihoodV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub selector_name: String,
    pub factor_count: usize,
    pub factors: Vec<EmissionLikelihoodFactorV0>,
    pub log_likelihood_bits: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EmissionLikelihoodFactorV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub source: &'static str,
    pub factor_name: &'static str,
    pub contribution_bits: f64,
    pub log_likelihood_bits: f64,
    pub reason: Option<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProvenancePosteriorAnnotationV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub node_count: usize,
    pub annotations: Vec<ProvenancePosteriorNodeV0>,
    pub provenance: Option<AbstractClassValueProvenanceV0>,
    pub annotation_id: String,
    pub mutates_existing_provenance_enum: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProvenancePosteriorNodeV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub provenance: AbstractClassValueProvenanceV0,
    pub posterior_logit_bits: f64,
    pub likelihood_factor_bits: f64,
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

pub fn uniform_pattern_prior_v0(corpus_fingerprint: impl Into<String>) -> PatternPriorV0 {
    PatternPriorV0 {
        schema_version: VARIATIONAL_SCHEMA_VERSION_V0,
        product: "omena-variational.pattern-prior",
        layer_marker: VARIATIONAL_LAYER_MARKER_V0,
        feature_gate: VARIATIONAL_FEATURE_GATE_V0,
        kind: PatternPriorKindV0::UniformDirichlet,
        prior_kind: "uniformDirichlet",
        dirichlet_alpha: [
            PatternIntentV0::Bem,
            PatternIntentV0::Utility,
            PatternIntentV0::Atomic,
            PatternIntentV0::Hybrid,
            PatternIntentV0::AdHoc,
        ]
        .into_iter()
        .map(|intent| PatternPriorAlphaV0 {
            schema_version: VARIATIONAL_SCHEMA_VERSION_V0,
            product: "omena-variational.pattern-prior-alpha",
            layer_marker: VARIATIONAL_LAYER_MARKER_V0,
            feature_gate: VARIATIONAL_FEATURE_GATE_V0,
            intent,
            alpha_bits: 1.0,
        })
        .collect(),
        concentration_bits: 5.0,
        corpus_calibration: Some(PatternPriorCalibrationV0 {
            schema_version: VARIATIONAL_SCHEMA_VERSION_V0,
            product: "omena-variational.pattern-prior-calibration",
            layer_marker: VARIATIONAL_LAYER_MARKER_V0,
            feature_gate: VARIATIONAL_FEATURE_GATE_V0,
            corpus_fingerprint: corpus_fingerprint.into(),
            axis_a_schema_version: "0",
            fixture_count: 0,
            generated_at_epoch: 0,
            human_review_gate_passed: false,
        }),
        rg_universality_class: None,
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

pub fn emission_likelihood_v0(
    selector_name: impl Into<String>,
    factors: Vec<EmissionLikelihoodFactorV0>,
) -> EmissionLikelihoodV0 {
    let log_likelihood_bits = factors
        .iter()
        .map(|factor| factor.contribution_bits)
        .sum::<f64>();
    EmissionLikelihoodV0 {
        schema_version: VARIATIONAL_SCHEMA_VERSION_V0,
        product: "omena-variational.emission-likelihood",
        layer_marker: VARIATIONAL_LAYER_MARKER_V0,
        feature_gate: VARIATIONAL_FEATURE_GATE_V0,
        selector_name: selector_name.into(),
        factor_count: factors.len(),
        factors,
        log_likelihood_bits,
    }
}

pub fn emission_likelihood_factor_v0(
    source: &'static str,
    contribution_bits: f64,
    reason: Option<&'static str>,
) -> EmissionLikelihoodFactorV0 {
    EmissionLikelihoodFactorV0 {
        schema_version: VARIATIONAL_SCHEMA_VERSION_V0,
        product: "omena-variational.emission-likelihood-factor",
        layer_marker: VARIATIONAL_LAYER_MARKER_V0,
        feature_gate: VARIATIONAL_FEATURE_GATE_V0,
        source,
        factor_name: source,
        contribution_bits,
        log_likelihood_bits: contribution_bits,
        reason,
    }
}

pub fn provenance_posterior_annotation_v0(
    annotation_id: impl Into<String>,
    annotations: Vec<ProvenancePosteriorNodeV0>,
) -> ProvenancePosteriorAnnotationV0 {
    let provenance = annotations.first().map(|node| node.provenance);
    ProvenancePosteriorAnnotationV0 {
        schema_version: VARIATIONAL_SCHEMA_VERSION_V0,
        product: "omena-variational.provenance-posterior-annotation",
        layer_marker: VARIATIONAL_LAYER_MARKER_V0,
        feature_gate: VARIATIONAL_FEATURE_GATE_V0,
        node_count: annotations.len(),
        annotations,
        provenance,
        annotation_id: annotation_id.into(),
        mutates_existing_provenance_enum: false,
    }
}

pub fn provenance_posterior_node_v0(
    provenance: AbstractClassValueProvenanceV0,
    posterior_logit_bits: f64,
    likelihood_factor_bits: f64,
) -> ProvenancePosteriorNodeV0 {
    ProvenancePosteriorNodeV0 {
        schema_version: VARIATIONAL_SCHEMA_VERSION_V0,
        product: "omena-variational.provenance-posterior-node",
        layer_marker: VARIATIONAL_LAYER_MARKER_V0,
        feature_gate: VARIATIONAL_FEATURE_GATE_V0,
        provenance,
        posterior_logit_bits,
        likelihood_factor_bits,
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

    #[test]
    fn uniform_dirichlet_prior_covers_all_pattern_intents_in_bits() {
        let prior = uniform_pattern_prior_v0("fixture-corpus-sha256");
        assert_eq!(prior.schema_version, "0");
        assert_eq!(prior.kind, PatternPriorKindV0::UniformDirichlet);
        assert_eq!(
            prior
                .dirichlet_alpha
                .iter()
                .map(|alpha| alpha.intent.as_str())
                .collect::<Vec<_>>(),
            vec!["bem", "utility", "atomic", "hybrid", "adHoc"]
        );
        assert_eq!(prior.concentration_bits, 5.0);
        assert_eq!(
            prior
                .corpus_calibration
                .as_ref()
                .map(|calibration| calibration.axis_a_schema_version),
            Some("0")
        );
    }

    #[test]
    fn likelihood_and_vfe_stay_at_bits_boundary() {
        let likelihood = emission_likelihood_v0(
            ".button",
            vec![
                emission_likelihood_factor_v0("cascadeProof", -1.0, Some("proof accepted")),
                emission_likelihood_factor_v0("specificityFit", -2.5, None),
            ],
        );
        let energy = variational_free_energy_v0(8.0, 3.5);

        assert_eq!(likelihood.factor_count, 2);
        assert_eq!(likelihood.log_likelihood_bits, -3.5);
        assert_eq!(energy.free_energy_bits, 4.5);
        assert_eq!(unit::bits_to_nats(unit::nats_to_bits(2.0)), 2.0);
    }

    #[test]
    fn posterior_annotation_is_sidecar_only() {
        let annotation = provenance_posterior_annotation_v0(
            "annotation",
            vec![provenance_posterior_node_v0(
                AbstractClassValueProvenanceV0::FiniteSetWideningChars,
                -0.25,
                -1.5,
            )],
        );

        assert_eq!(annotation.node_count, 1);
        assert_eq!(
            annotation.provenance,
            Some(AbstractClassValueProvenanceV0::FiniteSetWideningChars)
        );
        assert!(!annotation.mutates_existing_provenance_enum);
    }
}
