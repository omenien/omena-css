use omena_cascade::{CascadeDeclaration, cascade_margin_for_outcome, cascade_property};
use omena_query_checker_orchestrator::{
    OmenaCheckerCascadeDeclarationInputV0, OmenaCheckerCascadeEvaluationV0,
};

use super::super::OmenaQueryCascadeConfidenceV0;
use super::runtime_state::query_runtime_cascade_declaration_from_input;

pub(super) fn summarize_query_cascade_confidence_for_evaluation(
    evaluation: &OmenaCheckerCascadeEvaluationV0,
    declarations: &[OmenaCheckerCascadeDeclarationInputV0],
) -> Option<OmenaQueryCascadeConfidenceV0> {
    if !matches!(
        evaluation.rule_code_name,
        "unreachable-declaration" | "dead-cascade-layer"
    ) {
        return None;
    }
    let margin = query_cascade_margin_for_evaluation(evaluation, declarations)?;
    let abs_distance = margin.signed_distance.unsigned_abs();
    let dominant_axis_weight_basis_points =
        query_cascade_confidence_axis_weight_basis_points(margin.dominant_axis);
    let sigmoid_temperature_basis_points = 1_200u16;
    let confidence_score_basis_points = query_cascade_confidence_score_basis_points(
        abs_distance,
        dominant_axis_weight_basis_points,
        sigmoid_temperature_basis_points,
    );

    Some(OmenaQueryCascadeConfidenceV0 {
        schema_version: "0",
        product: "omena-query.cascade-confidence",
        feature_gate: "cascade-confidence-v0",
        confidence_kind: "fixtureWitnessTierWeightedSigmoid",
        claim_level: "fixtureWitnessResearchHint",
        theorem_claimed: false,
        public_safety_claim_ready: false,
        calibration_stage: "fixtureWitnessTierWeightSigmoidV0",
        margin_product: margin.product,
        margin_kind: margin.margin_kind,
        dominant_axis: margin.dominant_axis,
        dominant_axis_weight_basis_points,
        sigmoid_temperature_basis_points,
        signed_distance: margin.signed_distance,
        abs_distance,
        confidence_score_basis_points,
        confidence_bucket: query_cascade_confidence_bucket(confidence_score_basis_points),
        winner_declaration_id: margin.winner_declaration_id,
        challenger_declaration_id: margin.challenger_declaration_id,
    })
}

fn query_cascade_margin_for_evaluation(
    evaluation: &OmenaCheckerCascadeEvaluationV0,
    declarations: &[OmenaCheckerCascadeDeclarationInputV0],
) -> Option<omena_cascade::CascadeMarginV0> {
    let anchor_id = evaluation.declaration_ids.first()?;
    let anchor = declarations
        .iter()
        .find(|declaration| declaration.declaration_id == *anchor_id)?;
    let site_declarations = declarations
        .iter()
        .filter(|declaration| {
            declaration.selector == anchor.selector
                && declaration.property == anchor.property
                && declaration.condition_context == anchor.condition_context
        })
        .map(query_diagnostic_cascade_declaration_from_input)
        .collect::<Vec<_>>();
    if site_declarations.len() < 2 {
        return None;
    }

    let outcome = cascade_property(site_declarations, anchor.property.as_str());
    cascade_margin_for_outcome(&outcome)
}

fn query_diagnostic_cascade_declaration_from_input(
    input: &OmenaCheckerCascadeDeclarationInputV0,
) -> CascadeDeclaration {
    let mut declaration = query_runtime_cascade_declaration_from_input(input);
    declaration.id = input.declaration_id.clone();
    declaration
}

fn query_cascade_confidence_axis_weight_basis_points(axis: &str) -> u16 {
    match axis {
        "level" => 7_000,
        "layerRank" => 6_000,
        "scopeProximity" => 5_000,
        "specificityIds" => 4_000,
        "specificityClasses" => 3_000,
        "specificityElements" => 2_000,
        "sourceOrder" => 1_000,
        _ => 500,
    }
}

fn query_cascade_confidence_score_basis_points(
    abs_distance: u64,
    axis_weight_basis_points: u16,
    sigmoid_temperature_basis_points: u16,
) -> u16 {
    let signed_input = (abs_distance as f64 * f64::from(axis_weight_basis_points))
        / f64::from(sigmoid_temperature_basis_points);
    let confidence = 1.0 / (1.0 + (-signed_input).exp());
    (confidence * 10_000.0).round().clamp(0.0, 10_000.0) as u16
}

fn query_cascade_confidence_bucket(score_basis_points: u16) -> &'static str {
    match score_basis_points {
        0..=5_999 => "narrow",
        6_000..=8_499 => "moderate",
        _ => "clear",
    }
}
