//! Variational cascade inference contracts.
//!
//! Stochastic evidence is isolated in this crate and disabled by default. V0
//! contracts serialize log quantities in bits; any nats arithmetic stays behind
//! the public boundary.
//!
//! claim_level: product-wired posterior inference for checker evidence, not a
//! corpus-calibrated probabilistic design model.

pub mod hover;
pub mod unit;

use omena_abstract_value::AbstractClassValueProvenanceV0;
use serde::Serialize;

pub const VARIATIONAL_SCHEMA_VERSION_V0: &str = "0";
pub const VARIATIONAL_LAYER_MARKER_V0: &str = "variational-cascade";
pub const VARIATIONAL_FEATURE_GATE_V0: &str = "variational";
const DESIGNER_INTENT_BP_MAX_ITERATIONS_V0: usize = 12;
const DESIGNER_INTENT_BP_CONVERGENCE_EPSILON_BITS_V0: f64 = 0.005;
const DESIGNER_INTENT_BP_DAMPING_V0: f64 = 0.35;
const DESIGNER_INTENT_BP_FEEDBACK_WEIGHT_V0: f64 = 0.08;
const DESIGNER_INTENT_BP_MAX_FEEDBACK_BITS_V0: f64 = 0.35;

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
    pub calibration_scope: &'static str,
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DesignerIntentPosteriorInputV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub selector_name: String,
    pub declaration_count: usize,
    pub duplicate_property_tie_count: usize,
    pub custom_property_reference_count: usize,
}

pub fn designer_intent_posterior_input_v0(
    selector_name: impl Into<String>,
    declaration_count: usize,
    duplicate_property_tie_count: usize,
    custom_property_reference_count: usize,
) -> DesignerIntentPosteriorInputV0 {
    DesignerIntentPosteriorInputV0 {
        schema_version: VARIATIONAL_SCHEMA_VERSION_V0,
        product: "omena-variational.designer-intent-posterior-input",
        layer_marker: VARIATIONAL_LAYER_MARKER_V0,
        feature_gate: VARIATIONAL_FEATURE_GATE_V0,
        selector_name: selector_name.into(),
        declaration_count,
        duplicate_property_tie_count,
        custom_property_reference_count,
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DesignerIntentBeliefPropagationTraceV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub selector_name: String,
    pub factor_count: usize,
    pub iteration_count: usize,
    pub converged: bool,
    pub max_delta_bits: f64,
    pub free_energy_delta_bits: f64,
    pub free_energy: VariationalFreeEnergyV0,
    pub message_count: usize,
    pub messages: Vec<DesignerIntentBeliefPropagationMessageV0>,
    pub posterior_scores: Vec<DesignerIntentScoreV0>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum DesignerIntentMessageDirectionV0 {
    IntentToFactor,
    FactorToIntent,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DesignerIntentBeliefPropagationMessageV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub iteration_index: usize,
    pub direction: DesignerIntentMessageDirectionV0,
    pub source_factor: &'static str,
    pub target_intent: PatternIntentV0,
    pub message_bits: f64,
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

pub fn infer_designer_intent_posterior_v0(
    input: DesignerIntentPosteriorInputV0,
) -> DesignerIntentPosteriorV0 {
    let trace = designer_intent_belief_propagation_trace_v0(&input);

    DesignerIntentPosteriorV0 {
        schema_version: VARIATIONAL_SCHEMA_VERSION_V0,
        product: "omena-variational.designer-intent-posterior",
        layer_marker: VARIATIONAL_LAYER_MARKER_V0,
        feature_gate: VARIATIONAL_FEATURE_GATE_V0,
        mode: if trace.converged {
            DesignerIntentPosteriorModeV0::VciFormal
        } else {
            DesignerIntentPosteriorModeV0::PcnHierarchical
        },
        selector_name: input.selector_name,
        scores: trace.posterior_scores,
        enabled_by_default: true,
    }
}

pub fn designer_intent_belief_propagation_trace_v0(
    input: &DesignerIntentPosteriorInputV0,
) -> DesignerIntentBeliefPropagationTraceV0 {
    let selector = normalize_selector_name_for_intent_v0(&input.selector_name);
    let factors = designer_intent_evidence_factors_v0(&selector, input);
    let intents = [
        PatternIntentV0::Bem,
        PatternIntentV0::Utility,
        PatternIntentV0::Atomic,
        PatternIntentV0::Hybrid,
        PatternIntentV0::AdHoc,
    ];
    let prior_log_probability_bits = -(intents.len() as f64).log2();
    let mut factor_to_intent_messages = vec![vec![0.0; intents.len()]; factors.len()];
    let mut posterior_log_probability_bits = vec![prior_log_probability_bits; intents.len()];
    let mut messages = Vec::new();
    let mut iteration_count = 0;
    let mut converged = false;
    let mut max_delta_bits = f64::INFINITY;
    let mut free_energy_delta_bits = f64::INFINITY;
    let mut free_energy = variational_free_energy_from_beliefs_v0(
        &posterior_log_probability_bits,
        prior_log_probability_bits,
        &factor_to_intent_messages,
    );

    for iteration_index in 0..DESIGNER_INTENT_BP_MAX_ITERATIONS_V0 {
        iteration_count = iteration_index + 1;
        let mut intent_to_factor_messages = vec![vec![0.0; intents.len()]; factors.len()];

        for (factor_index, factor) in factors.iter().enumerate() {
            let mut raw_intent_messages = Vec::with_capacity(intents.len());
            for intent_index in 0..intents.len() {
                let incoming_from_other_factors = factor_to_intent_messages
                    .iter()
                    .enumerate()
                    .filter(|(candidate_index, _)| *candidate_index != factor_index)
                    .map(|(_, factor_messages)| factor_messages[intent_index])
                    .sum::<f64>();
                raw_intent_messages.push(prior_log_probability_bits + incoming_from_other_factors);
            }
            let normalization_bits = log2_sum_exp_v0(&raw_intent_messages);
            for (intent_index, intent) in intents.iter().enumerate() {
                let message_bits = raw_intent_messages[intent_index] - normalization_bits;
                intent_to_factor_messages[factor_index][intent_index] = message_bits;
                messages.push(DesignerIntentBeliefPropagationMessageV0 {
                    schema_version: VARIATIONAL_SCHEMA_VERSION_V0,
                    product: "omena-variational.designer-intent-bp-message",
                    layer_marker: VARIATIONAL_LAYER_MARKER_V0,
                    feature_gate: VARIATIONAL_FEATURE_GATE_V0,
                    iteration_index,
                    direction: DesignerIntentMessageDirectionV0::IntentToFactor,
                    source_factor: factor.source_factor,
                    target_intent: *intent,
                    message_bits,
                });
            }
        }

        let mut next_factor_to_intent_messages = factor_to_intent_messages.clone();
        for (factor_index, factor) in factors.iter().enumerate() {
            for (intent_index, intent) in intents.iter().enumerate() {
                let evidence_bits = factor.message_bits_for(*intent);
                let feedback_bits = (intent_to_factor_messages[factor_index][intent_index]
                    - prior_log_probability_bits)
                    .clamp(
                        -DESIGNER_INTENT_BP_MAX_FEEDBACK_BITS_V0,
                        DESIGNER_INTENT_BP_MAX_FEEDBACK_BITS_V0,
                    )
                    * DESIGNER_INTENT_BP_FEEDBACK_WEIGHT_V0;
                let target_message_bits = evidence_bits + feedback_bits;
                let message_bits = DESIGNER_INTENT_BP_DAMPING_V0
                    * factor_to_intent_messages[factor_index][intent_index]
                    + (1.0 - DESIGNER_INTENT_BP_DAMPING_V0) * target_message_bits;
                next_factor_to_intent_messages[factor_index][intent_index] = message_bits;
                messages.push(DesignerIntentBeliefPropagationMessageV0 {
                    schema_version: VARIATIONAL_SCHEMA_VERSION_V0,
                    product: "omena-variational.designer-intent-bp-message",
                    layer_marker: VARIATIONAL_LAYER_MARKER_V0,
                    feature_gate: VARIATIONAL_FEATURE_GATE_V0,
                    iteration_index,
                    direction: DesignerIntentMessageDirectionV0::FactorToIntent,
                    source_factor: factor.source_factor,
                    target_intent: *intent,
                    message_bits,
                });
            }
        }

        let next_posterior_log_probability_bits = posterior_log_probability_bits_from_messages_v0(
            prior_log_probability_bits,
            &next_factor_to_intent_messages,
        );
        max_delta_bits = max_abs_delta_bits_v0(
            &posterior_log_probability_bits,
            &next_posterior_log_probability_bits,
        );
        let next_free_energy = variational_free_energy_from_beliefs_v0(
            &next_posterior_log_probability_bits,
            prior_log_probability_bits,
            &next_factor_to_intent_messages,
        );
        free_energy_delta_bits =
            (free_energy.free_energy_bits - next_free_energy.free_energy_bits).abs();

        posterior_log_probability_bits = next_posterior_log_probability_bits;
        factor_to_intent_messages = next_factor_to_intent_messages;
        free_energy = next_free_energy;

        if max_delta_bits <= DESIGNER_INTENT_BP_CONVERGENCE_EPSILON_BITS_V0
            && free_energy_delta_bits <= DESIGNER_INTENT_BP_CONVERGENCE_EPSILON_BITS_V0
        {
            converged = true;
            break;
        }
    }

    let posterior_scores =
        designer_intent_scores_from_log_probabilities_v0(&intents, &posterior_log_probability_bits);

    DesignerIntentBeliefPropagationTraceV0 {
        schema_version: VARIATIONAL_SCHEMA_VERSION_V0,
        product: "omena-variational.designer-intent-belief-propagation",
        layer_marker: VARIATIONAL_LAYER_MARKER_V0,
        feature_gate: VARIATIONAL_FEATURE_GATE_V0,
        selector_name: input.selector_name.clone(),
        factor_count: factors.len(),
        iteration_count,
        converged,
        max_delta_bits,
        free_energy_delta_bits,
        free_energy,
        message_count: messages.len(),
        messages,
        posterior_scores,
    }
}

pub fn dominant_designer_intent_v0(
    posterior: &DesignerIntentPosteriorV0,
) -> Option<PatternIntentV0> {
    posterior.scores.first().map(|score| score.intent)
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
            calibration_scope: "fixtureUniformNoCorpusCalibration",
            axis_a_schema_version: "0",
            fixture_count: 0,
            generated_at_epoch: 0,
            human_review_gate_passed: false,
        }),
        rg_universality_class: None,
    }
}

fn normalize_selector_name_for_intent_v0(selector_name: &str) -> String {
    selector_name
        .trim()
        .trim_start_matches('.')
        .split([':', '[', ' ', '>', '+', '~', ','])
        .next()
        .unwrap_or(selector_name)
        .trim()
        .to_string()
}

fn bool_bits_v0(value: bool) -> f64 {
    if value { 1.0 } else { 0.0 }
}

struct DesignerIntentEvidenceFactorV0 {
    source_factor: &'static str,
    contributions: Vec<(PatternIntentV0, f64)>,
}

impl DesignerIntentEvidenceFactorV0 {
    fn message_bits_for(&self, intent: PatternIntentV0) -> f64 {
        self.contributions
            .iter()
            .find_map(|(candidate, bits)| (*candidate == intent).then_some(*bits))
            .unwrap_or(0.0)
    }
}

fn designer_intent_evidence_factors_v0(
    selector: &str,
    input: &DesignerIntentPosteriorInputV0,
) -> Vec<DesignerIntentEvidenceFactorV0> {
    let has_bem_marker = selector.contains("__") || selector.contains("--");
    let looks_utility = selector.starts_with("u-")
        || selector.starts_with("is-")
        || selector.starts_with("has-")
        || selector
            .split('-')
            .any(|part| matches!(part, "m" | "p" | "mt" | "mb" | "ml" | "mr" | "bg" | "text"));
    let looks_atomic = input.declaration_count <= 1 && selector.len() <= 8;
    let looks_hybrid = selector.matches('-').count() >= 3
        || (has_bem_marker && input.custom_property_reference_count > 0);

    vec![
        DesignerIntentEvidenceFactorV0 {
            source_factor: "selector-bem-marker",
            contributions: vec![
                (PatternIntentV0::Bem, bool_bits_v0(has_bem_marker) * 7.0),
                (
                    PatternIntentV0::Utility,
                    -bool_bits_v0(has_bem_marker) * 2.0,
                ),
                (PatternIntentV0::Hybrid, bool_bits_v0(has_bem_marker) * 1.0),
                (
                    PatternIntentV0::AdHoc,
                    bool_bits_v0(!has_bem_marker && !looks_utility) * 1.0,
                ),
            ],
        },
        DesignerIntentEvidenceFactorV0 {
            source_factor: "selector-utility-marker",
            contributions: vec![
                (PatternIntentV0::Utility, bool_bits_v0(looks_utility) * 6.5),
                (
                    PatternIntentV0::AdHoc,
                    bool_bits_v0(!has_bem_marker && !looks_utility) * 1.0,
                ),
            ],
        },
        DesignerIntentEvidenceFactorV0 {
            source_factor: "declaration-cardinality",
            contributions: vec![
                (
                    PatternIntentV0::Bem,
                    bool_bits_v0(input.declaration_count > 1) * 1.0,
                ),
                (
                    PatternIntentV0::Utility,
                    bool_bits_v0(input.declaration_count <= 2) * 1.0,
                ),
                (
                    PatternIntentV0::Atomic,
                    bool_bits_v0(looks_atomic) * 5.0
                        - bool_bits_v0(input.declaration_count > 1) * 2.0,
                ),
            ],
        },
        DesignerIntentEvidenceFactorV0 {
            source_factor: "source-order-tie",
            contributions: vec![
                (
                    PatternIntentV0::Bem,
                    -bool_bits_v0(input.duplicate_property_tie_count > 0) * 1.0,
                ),
                (
                    PatternIntentV0::AdHoc,
                    bool_bits_v0(input.duplicate_property_tie_count > 0) * 1.0,
                ),
            ],
        },
        DesignerIntentEvidenceFactorV0 {
            source_factor: "custom-property-context",
            contributions: vec![(
                PatternIntentV0::Hybrid,
                bool_bits_v0(looks_hybrid) * 4.0
                    + bool_bits_v0(input.custom_property_reference_count > 0) * 1.5,
            )],
        },
    ]
}

fn log2_sum_exp_v0(logits: &[f64]) -> f64 {
    let max_logit = logits
        .iter()
        .copied()
        .fold(f64::NEG_INFINITY, |left, right| left.max(right));
    max_logit
        + logits
            .iter()
            .map(|logit| 2_f64.powf(*logit - max_logit))
            .sum::<f64>()
            .log2()
}

fn posterior_log_probability_bits_from_messages_v0(
    prior_log_probability_bits: f64,
    factor_to_intent_messages: &[Vec<f64>],
) -> Vec<f64> {
    let intent_count = factor_to_intent_messages
        .first()
        .map(Vec::len)
        .unwrap_or_default();
    let logits = (0..intent_count)
        .map(|intent_index| {
            prior_log_probability_bits
                + factor_to_intent_messages
                    .iter()
                    .map(|factor_messages| factor_messages[intent_index])
                    .sum::<f64>()
        })
        .collect::<Vec<_>>();
    let normalization_bits = log2_sum_exp_v0(&logits);
    logits
        .into_iter()
        .map(|logit| logit - normalization_bits)
        .collect()
}

fn designer_intent_scores_from_log_probabilities_v0(
    intents: &[PatternIntentV0],
    log_probability_bits: &[f64],
) -> Vec<DesignerIntentScoreV0> {
    let mut scores = intents
        .iter()
        .zip(log_probability_bits.iter())
        .map(|(intent, log_probability_bits)| DesignerIntentScoreV0 {
            schema_version: VARIATIONAL_SCHEMA_VERSION_V0,
            product: "omena-variational.designer-intent-score",
            layer_marker: VARIATIONAL_LAYER_MARKER_V0,
            feature_gate: VARIATIONAL_FEATURE_GATE_V0,
            intent: *intent,
            log_probability_bits: *log_probability_bits,
        })
        .collect::<Vec<_>>();
    scores.sort_by(|left, right| {
        right
            .log_probability_bits
            .partial_cmp(&left.log_probability_bits)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| left.intent.as_str().cmp(right.intent.as_str()))
    });
    scores
}

fn max_abs_delta_bits_v0(left: &[f64], right: &[f64]) -> f64 {
    left.iter()
        .zip(right.iter())
        .map(|(left, right)| (left - right).abs())
        .fold(0.0, f64::max)
}

fn variational_free_energy_from_beliefs_v0(
    posterior_log_probability_bits: &[f64],
    prior_log_probability_bits: f64,
    factor_to_intent_messages: &[Vec<f64>],
) -> VariationalFreeEnergyV0 {
    let mut complexity_bits = 0.0;
    let mut accuracy_bits = 0.0;
    for (intent_index, posterior_log_probability_bits) in
        posterior_log_probability_bits.iter().enumerate()
    {
        let probability = 2_f64.powf(*posterior_log_probability_bits);
        complexity_bits +=
            probability * (posterior_log_probability_bits - prior_log_probability_bits);
        accuracy_bits += probability
            * factor_to_intent_messages
                .iter()
                .map(|factor_messages| factor_messages[intent_index])
                .sum::<f64>();
    }

    VariationalFreeEnergyV0 {
        schema_version: VARIATIONAL_SCHEMA_VERSION_V0,
        product: "omena-variational.free-energy",
        layer_marker: VARIATIONAL_LAYER_MARKER_V0,
        feature_gate: VARIATIONAL_FEATURE_GATE_V0,
        complexity_bits,
        accuracy_bits,
        free_energy_bits: complexity_bits - accuracy_bits,
        public_framing: "V0 mean-field free-energy over fixture-uniform prior",
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
        public_framing: "V0 mean-field free-energy over fixture-uniform prior",
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
    fn posterior_inference_uses_selector_and_cascade_features() {
        let bem = infer_designer_intent_posterior_v0(designer_intent_posterior_input_v0(
            ".button--primary",
            2,
            1,
            0,
        ));
        let utility = infer_designer_intent_posterior_v0(designer_intent_posterior_input_v0(
            ".u-color-red",
            2,
            1,
            0,
        ));

        assert_eq!(bem.mode, DesignerIntentPosteriorModeV0::VciFormal);
        assert!(bem.enabled_by_default);
        assert_eq!(
            dominant_designer_intent_v0(&bem),
            Some(PatternIntentV0::Bem)
        );
        assert_eq!(
            dominant_designer_intent_v0(&utility),
            Some(PatternIntentV0::Utility)
        );
        assert_ne!(
            bem.scores.first().map(|score| score.intent),
            utility.scores.first().map(|score| score.intent)
        );
    }

    #[test]
    fn belief_propagation_trace_carries_non_tautological_factor_messages() {
        let tied = designer_intent_posterior_input_v0(".button--primary", 2, 1, 0);
        let explicit = DesignerIntentPosteriorInputV0 {
            duplicate_property_tie_count: 0,
            ..tied.clone()
        };
        let tied_trace = designer_intent_belief_propagation_trace_v0(&tied);
        let explicit_trace = designer_intent_belief_propagation_trace_v0(&explicit);

        assert_eq!(tied_trace.factor_count, 5);
        assert!(tied_trace.iteration_count > 1);
        assert!(tied_trace.converged);
        assert!(
            tied_trace.max_delta_bits <= DESIGNER_INTENT_BP_CONVERGENCE_EPSILON_BITS_V0,
            "final iteration should satisfy posterior fixpoint tolerance"
        );
        assert!(
            tied_trace.free_energy_delta_bits <= DESIGNER_INTENT_BP_CONVERGENCE_EPSILON_BITS_V0,
            "free-energy objective should participate in convergence"
        );
        assert_eq!(
            tied_trace.message_count,
            tied_trace.messages.len(),
            "trace message count must reflect retained iteration evidence"
        );
        assert!(
            tied_trace.message_count > 25,
            "iterative belief propagation should retain more than one factor-to-intent sweep"
        );
        assert!(tied_trace.messages.iter().any(|message| {
            message.direction == DesignerIntentMessageDirectionV0::IntentToFactor
        }));
        assert!(tied_trace.messages.iter().any(|message| {
            message.direction == DesignerIntentMessageDirectionV0::FactorToIntent
        }));
        assert!(tied_trace.messages.iter().any(|message| {
            message.direction == DesignerIntentMessageDirectionV0::FactorToIntent
                && message.source_factor == "selector-bem-marker"
                && message.target_intent == PatternIntentV0::Bem
                && message.message_bits > 0.0
        }));
        assert!(tied_trace.messages.iter().any(|message| {
            message.direction == DesignerIntentMessageDirectionV0::FactorToIntent
                && message.source_factor == "source-order-tie"
                && message.target_intent == PatternIntentV0::Bem
                && message.message_bits < 0.0
        }));

        let single_sweep_trace = designer_intent_single_sweep_trace_for_test_v0(&tied);
        let single_sweep_bem_bits =
            score_bits_for_intent(&single_sweep_trace, PatternIntentV0::Bem);
        let tied_bem_bits = score_bits_for_intent(&tied_trace, PatternIntentV0::Bem);
        assert_ne!(
            tied_bem_bits, single_sweep_bem_bits,
            "coupled iterative messages must change the posterior relative to the previous single-sweep mechanism"
        );

        let explicit_bem_bits = score_bits_for_intent(&explicit_trace, PatternIntentV0::Bem);
        assert!(
            tied_bem_bits < explicit_bem_bits,
            "source-order tie factor must lower the BEM posterior instead of leaving the fixture tautological"
        );
    }

    fn designer_intent_single_sweep_trace_for_test_v0(
        input: &DesignerIntentPosteriorInputV0,
    ) -> DesignerIntentBeliefPropagationTraceV0 {
        let selector = normalize_selector_name_for_intent_v0(&input.selector_name);
        let factors = designer_intent_evidence_factors_v0(&selector, input);
        let intents = [
            PatternIntentV0::Bem,
            PatternIntentV0::Utility,
            PatternIntentV0::Atomic,
            PatternIntentV0::Hybrid,
            PatternIntentV0::AdHoc,
        ];
        let prior_log_probability_bits = -(intents.len() as f64).log2();
        let factor_to_intent_messages = factors
            .iter()
            .map(|factor| {
                intents
                    .iter()
                    .map(|intent| factor.message_bits_for(*intent))
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        let posterior_log_probability_bits = posterior_log_probability_bits_from_messages_v0(
            prior_log_probability_bits,
            &factor_to_intent_messages,
        );
        let posterior_scores = designer_intent_scores_from_log_probabilities_v0(
            &intents,
            &posterior_log_probability_bits,
        );

        DesignerIntentBeliefPropagationTraceV0 {
            schema_version: VARIATIONAL_SCHEMA_VERSION_V0,
            product: "omena-variational.designer-intent-belief-propagation",
            layer_marker: VARIATIONAL_LAYER_MARKER_V0,
            feature_gate: VARIATIONAL_FEATURE_GATE_V0,
            selector_name: input.selector_name.clone(),
            factor_count: factors.len(),
            iteration_count: 1,
            converged: true,
            max_delta_bits: 0.0,
            free_energy_delta_bits: 0.0,
            free_energy: variational_free_energy_from_beliefs_v0(
                &posterior_log_probability_bits,
                prior_log_probability_bits,
                &factor_to_intent_messages,
            ),
            message_count: factors.len() * intents.len(),
            messages: factors
                .iter()
                .flat_map(|factor| {
                    intents
                        .iter()
                        .map(|intent| DesignerIntentBeliefPropagationMessageV0 {
                            schema_version: VARIATIONAL_SCHEMA_VERSION_V0,
                            product: "omena-variational.designer-intent-bp-message",
                            layer_marker: VARIATIONAL_LAYER_MARKER_V0,
                            feature_gate: VARIATIONAL_FEATURE_GATE_V0,
                            iteration_index: 0,
                            direction: DesignerIntentMessageDirectionV0::FactorToIntent,
                            source_factor: factor.source_factor,
                            target_intent: *intent,
                            message_bits: factor.message_bits_for(*intent),
                        })
                })
                .collect(),
            posterior_scores,
        }
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
        let calibration = prior.corpus_calibration.as_ref();
        assert_eq!(
            calibration.map(|calibration| calibration.axis_a_schema_version),
            Some("0")
        );
        assert_eq!(
            calibration.map(|calibration| calibration.calibration_scope),
            Some("fixtureUniformNoCorpusCalibration")
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

    fn score_bits_for_intent(
        trace: &DesignerIntentBeliefPropagationTraceV0,
        intent: PatternIntentV0,
    ) -> f64 {
        trace
            .posterior_scores
            .iter()
            .find_map(|score| (score.intent == intent).then_some(score.log_probability_bits))
            .unwrap_or(f64::NEG_INFINITY)
    }
}
