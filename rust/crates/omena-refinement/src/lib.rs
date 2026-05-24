//! Refinement type system contracts for cascade analysis.
//!
//! The crate keeps legacy abstract property values wire-compatible by adding a
//! strict-superset wrapper and delegating cascade checks to the byte-stable
//! `omena-cascade` proof primitives.

use std::marker::PhantomData;

use omena_abstract_value::AbstractPropertyValueV0;
use omena_cascade::{
    CascadeDeclaration, CascadeRefinementContextV0,
    refine_declaration_in_context as refine_cascade_declaration_in_context,
};
use omena_refinement_trait::{
    PropertyIndexV0, REFINEMENT_FEATURE_GATE_V0, REFINEMENT_LAYER_MARKER_V0,
    REFINEMENT_SCHEMA_VERSION_V0, RefinementPredicateV0, RefinementVerdictV0, RefinementWitnessV0,
    refinement_provenance_v0, refinement_witness_v0,
};
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum AbstractValueShapeV0 {
    Bottom,
    Exact,
    FiniteSet,
    CustomPropertyReference,
    Top,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TopPredicateV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
}

impl Default for TopPredicateV0 {
    fn default() -> Self {
        Self {
            schema_version: REFINEMENT_SCHEMA_VERSION_V0,
            product: "omena-refinement.top-predicate",
            layer_marker: REFINEMENT_LAYER_MARKER_V0,
            feature_gate: REFINEMENT_FEATURE_GATE_V0,
        }
    }
}

impl RefinementPredicateV0 for TopPredicateV0 {
    const PREDICATE_ID: &'static str = "top";
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnyPropertyIndexV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
}

impl Default for AnyPropertyIndexV0 {
    fn default() -> Self {
        Self {
            schema_version: REFINEMENT_SCHEMA_VERSION_V0,
            product: "omena-refinement.any-property-index",
            layer_marker: REFINEMENT_LAYER_MARKER_V0,
            feature_gate: REFINEMENT_FEATURE_GATE_V0,
        }
    }
}

impl PropertyIndexV0 for AnyPropertyIndexV0 {
    const PROPERTY_NAME: &'static str = "*";
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase", bound = "")]
pub struct RefinedAbstractPropertyValueV0<P: PropertyIndexV0, R: RefinementPredicateV0> {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub property_name: &'static str,
    pub predicate_id: &'static str,
    pub value_shape: AbstractValueShapeV0,
    pub legacy_value: AbstractPropertyValueV0,
    pub strict_superset_of_legacy_v0: bool,
    #[serde(skip)]
    marker: PhantomData<(P, R)>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum RefinementPropertyPredicateV0 {
    Any,
    ExactValue {
        property_name: String,
        value: String,
    },
    OneOfValues {
        property_name: String,
        values: Vec<String>,
    },
    CustomPropertyReference {
        property_name: String,
        custom_property_name: String,
    },
    And {
        predicates: Vec<RefinementPropertyPredicateV0>,
    },
    Or {
        predicates: Vec<RefinementPropertyPredicateV0>,
    },
    Not {
        predicate: Box<RefinementPropertyPredicateV0>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RefinementPredicateEvaluationV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub predicate_expression_id: String,
    pub value_shape: AbstractValueShapeV0,
    pub verdict: RefinementVerdictV0,
    pub matched_clause_count: usize,
    pub witness: RefinementWitnessV0,
}

pub fn project_legacy_to_refined_v0<P, R>(
    legacy_value: AbstractPropertyValueV0,
) -> RefinedAbstractPropertyValueV0<P, R>
where
    P: PropertyIndexV0,
    R: RefinementPredicateV0,
{
    RefinedAbstractPropertyValueV0 {
        schema_version: REFINEMENT_SCHEMA_VERSION_V0,
        product: "omena-refinement.refined-abstract-property-value",
        layer_marker: REFINEMENT_LAYER_MARKER_V0,
        feature_gate: REFINEMENT_FEATURE_GATE_V0,
        property_name: P::PROPERTY_NAME,
        predicate_id: R::PREDICATE_ID,
        value_shape: abstract_property_value_shape_v0(&legacy_value),
        legacy_value,
        strict_superset_of_legacy_v0: true,
        marker: PhantomData,
    }
}

pub fn project_refined_to_legacy_v0<P, R>(
    refined: &RefinedAbstractPropertyValueV0<P, R>,
) -> AbstractPropertyValueV0
where
    P: PropertyIndexV0,
    R: RefinementPredicateV0,
{
    refined.legacy_value.clone()
}

pub fn evaluate_refinement_property_predicate_v0(
    predicate: &RefinementPropertyPredicateV0,
    value: &AbstractPropertyValueV0,
) -> RefinementPredicateEvaluationV0 {
    let verdict = evaluate_refinement_predicate_verdict_v0(predicate, value);
    let matched_clause_count = count_satisfied_refinement_clauses_v0(predicate, value);
    let predicate_expression_id = refinement_predicate_expression_id_v0(predicate);
    let witness = refinement_witness_v0(
        "property-grammar",
        verdict,
        vec![refinement_provenance_v0("property-grammar", None)],
    );

    RefinementPredicateEvaluationV0 {
        schema_version: REFINEMENT_SCHEMA_VERSION_V0,
        product: "omena-refinement.property-predicate-evaluation",
        layer_marker: REFINEMENT_LAYER_MARKER_V0,
        feature_gate: REFINEMENT_FEATURE_GATE_V0,
        predicate_expression_id,
        value_shape: abstract_property_value_shape_v0(value),
        verdict,
        matched_clause_count,
        witness,
    }
}

pub fn refine_declaration_in_context(
    declaration: &CascadeDeclaration,
    context: &CascadeRefinementContextV0,
) -> RefinementWitnessV0 {
    refine_cascade_declaration_in_context(declaration, context)
}

pub fn abstract_property_value_shape_v0(value: &AbstractPropertyValueV0) -> AbstractValueShapeV0 {
    match value {
        AbstractPropertyValueV0::Bottom { .. } => AbstractValueShapeV0::Bottom,
        AbstractPropertyValueV0::Exact { .. } => AbstractValueShapeV0::Exact,
        AbstractPropertyValueV0::FiniteSet { .. } => AbstractValueShapeV0::FiniteSet,
        AbstractPropertyValueV0::CustomPropertyReference { .. } => {
            AbstractValueShapeV0::CustomPropertyReference
        }
        AbstractPropertyValueV0::Top { .. } => AbstractValueShapeV0::Top,
    }
}

fn evaluate_refinement_predicate_verdict_v0(
    predicate: &RefinementPropertyPredicateV0,
    value: &AbstractPropertyValueV0,
) -> RefinementVerdictV0 {
    match predicate {
        RefinementPropertyPredicateV0::Any => RefinementVerdictV0::SatisfiedAll,
        RefinementPropertyPredicateV0::ExactValue {
            property_name,
            value: expected,
        } => evaluate_exact_value_predicate_v0(property_name, expected, value),
        RefinementPropertyPredicateV0::OneOfValues {
            property_name,
            values,
        } => evaluate_one_of_values_predicate_v0(property_name, values, value),
        RefinementPropertyPredicateV0::CustomPropertyReference {
            property_name,
            custom_property_name,
        } => evaluate_custom_property_reference_predicate_v0(
            property_name,
            custom_property_name,
            value,
        ),
        RefinementPropertyPredicateV0::And { predicates } => combine_and_refinement_verdicts_v0(
            &predicates
                .iter()
                .map(|predicate| evaluate_refinement_predicate_verdict_v0(predicate, value))
                .collect::<Vec<_>>(),
        ),
        RefinementPropertyPredicateV0::Or { predicates } => combine_or_refinement_verdicts_v0(
            &predicates
                .iter()
                .map(|predicate| evaluate_refinement_predicate_verdict_v0(predicate, value))
                .collect::<Vec<_>>(),
        ),
        RefinementPropertyPredicateV0::Not { predicate } => {
            match evaluate_refinement_predicate_verdict_v0(predicate, value) {
                RefinementVerdictV0::SatisfiedAll => RefinementVerdictV0::Unsatisfiable,
                RefinementVerdictV0::Unsatisfiable => RefinementVerdictV0::SatisfiedAll,
                RefinementVerdictV0::SatisfiedSome | RefinementVerdictV0::Unknown => {
                    RefinementVerdictV0::Unknown
                }
            }
        }
    }
}

fn evaluate_exact_value_predicate_v0(
    property_name: &str,
    expected: &str,
    value: &AbstractPropertyValueV0,
) -> RefinementVerdictV0 {
    match value {
        AbstractPropertyValueV0::Exact {
            property_name: actual_property,
            value: actual_value,
            ..
        } if actual_property == property_name && actual_value == expected => {
            RefinementVerdictV0::SatisfiedAll
        }
        AbstractPropertyValueV0::FiniteSet {
            property_name: actual_property,
            values,
            ..
        } if actual_property == property_name && values.iter().any(|value| value == expected) => {
            if values.len() == 1 {
                RefinementVerdictV0::SatisfiedAll
            } else {
                RefinementVerdictV0::SatisfiedSome
            }
        }
        AbstractPropertyValueV0::Top {
            property_name: actual_property,
        }
        | AbstractPropertyValueV0::CustomPropertyReference {
            property_name: actual_property,
            ..
        } if actual_property == property_name => RefinementVerdictV0::Unknown,
        _ => RefinementVerdictV0::Unsatisfiable,
    }
}

fn evaluate_one_of_values_predicate_v0(
    property_name: &str,
    expected_values: &[String],
    value: &AbstractPropertyValueV0,
) -> RefinementVerdictV0 {
    match value {
        AbstractPropertyValueV0::Exact {
            property_name: actual_property,
            value: actual_value,
            ..
        } if actual_property == property_name => {
            if expected_values.contains(actual_value) {
                RefinementVerdictV0::SatisfiedAll
            } else {
                RefinementVerdictV0::Unsatisfiable
            }
        }
        AbstractPropertyValueV0::FiniteSet {
            property_name: actual_property,
            values,
            ..
        } if actual_property == property_name => {
            let matched = values
                .iter()
                .filter(|value| expected_values.contains(*value))
                .count();
            if matched == values.len() {
                RefinementVerdictV0::SatisfiedAll
            } else if matched > 0 {
                RefinementVerdictV0::SatisfiedSome
            } else {
                RefinementVerdictV0::Unsatisfiable
            }
        }
        AbstractPropertyValueV0::Top {
            property_name: actual_property,
        }
        | AbstractPropertyValueV0::CustomPropertyReference {
            property_name: actual_property,
            ..
        } if actual_property == property_name => RefinementVerdictV0::Unknown,
        _ => RefinementVerdictV0::Unsatisfiable,
    }
}

fn evaluate_custom_property_reference_predicate_v0(
    property_name: &str,
    expected_custom_property: &str,
    value: &AbstractPropertyValueV0,
) -> RefinementVerdictV0 {
    match value {
        AbstractPropertyValueV0::CustomPropertyReference {
            property_name: actual_property,
            custom_property_name,
            ..
        } if actual_property == property_name
            && custom_property_name == expected_custom_property =>
        {
            RefinementVerdictV0::SatisfiedAll
        }
        AbstractPropertyValueV0::Top {
            property_name: actual_property,
        } if actual_property == property_name => RefinementVerdictV0::Unknown,
        _ => RefinementVerdictV0::Unsatisfiable,
    }
}

fn combine_and_refinement_verdicts_v0(verdicts: &[RefinementVerdictV0]) -> RefinementVerdictV0 {
    if verdicts.is_empty()
        || verdicts
            .iter()
            .all(|verdict| *verdict == RefinementVerdictV0::SatisfiedAll)
    {
        RefinementVerdictV0::SatisfiedAll
    } else if verdicts.contains(&RefinementVerdictV0::Unsatisfiable) {
        RefinementVerdictV0::Unsatisfiable
    } else if verdicts.contains(&RefinementVerdictV0::SatisfiedAll)
        || verdicts.contains(&RefinementVerdictV0::SatisfiedSome)
    {
        RefinementVerdictV0::SatisfiedSome
    } else {
        RefinementVerdictV0::Unknown
    }
}

fn combine_or_refinement_verdicts_v0(verdicts: &[RefinementVerdictV0]) -> RefinementVerdictV0 {
    if verdicts.is_empty() || verdicts.contains(&RefinementVerdictV0::SatisfiedAll) {
        RefinementVerdictV0::SatisfiedAll
    } else if verdicts.contains(&RefinementVerdictV0::SatisfiedSome) {
        RefinementVerdictV0::SatisfiedSome
    } else if verdicts
        .iter()
        .all(|verdict| *verdict == RefinementVerdictV0::Unsatisfiable)
    {
        RefinementVerdictV0::Unsatisfiable
    } else {
        RefinementVerdictV0::Unknown
    }
}

fn count_satisfied_refinement_clauses_v0(
    predicate: &RefinementPropertyPredicateV0,
    value: &AbstractPropertyValueV0,
) -> usize {
    match predicate {
        RefinementPropertyPredicateV0::And { predicates }
        | RefinementPropertyPredicateV0::Or { predicates } => predicates
            .iter()
            .map(|predicate| count_satisfied_refinement_clauses_v0(predicate, value))
            .sum(),
        RefinementPropertyPredicateV0::Not { predicate } => usize::from(matches!(
            evaluate_refinement_predicate_verdict_v0(predicate, value),
            RefinementVerdictV0::Unsatisfiable
        )),
        _ => usize::from(matches!(
            evaluate_refinement_predicate_verdict_v0(predicate, value),
            RefinementVerdictV0::SatisfiedAll | RefinementVerdictV0::SatisfiedSome
        )),
    }
}

fn refinement_predicate_expression_id_v0(predicate: &RefinementPropertyPredicateV0) -> String {
    match predicate {
        RefinementPropertyPredicateV0::Any => "any".to_string(),
        RefinementPropertyPredicateV0::ExactValue {
            property_name,
            value,
        } => format!("exact:{property_name}:{value}"),
        RefinementPropertyPredicateV0::OneOfValues {
            property_name,
            values,
        } => format!("one-of:{property_name}:{}", values.join("|")),
        RefinementPropertyPredicateV0::CustomPropertyReference {
            property_name,
            custom_property_name,
        } => format!("custom-ref:{property_name}:{custom_property_name}"),
        RefinementPropertyPredicateV0::And { predicates } => format!(
            "and({})",
            predicates
                .iter()
                .map(refinement_predicate_expression_id_v0)
                .collect::<Vec<_>>()
                .join(",")
        ),
        RefinementPropertyPredicateV0::Or { predicates } => format!(
            "or({})",
            predicates
                .iter()
                .map(refinement_predicate_expression_id_v0)
                .collect::<Vec<_>>()
                .join(",")
        ),
        RefinementPropertyPredicateV0::Not { predicate } => {
            format!("not({})", refinement_predicate_expression_id_v0(predicate))
        }
    }
}

#[cfg(feature = "refinement-smt")]
pub fn refinement_smt_backend_available_v0() -> bool {
    let _ = omena_smt::cascade_theory_signature_v0();
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn refined_value_round_trips_to_legacy_without_mutating_v0() {
        let top = TopPredicateV0::default();
        let any = AnyPropertyIndexV0::default();
        assert_eq!(top.schema_version, "0");
        assert_eq!(any.layer_marker, "refinement-cascade");

        let legacy = AbstractPropertyValueV0::Top {
            property_name: "color".to_string(),
        };
        let refined =
            project_legacy_to_refined_v0::<AnyPropertyIndexV0, TopPredicateV0>(legacy.clone());
        assert_eq!(refined.schema_version, "0");
        assert_eq!(refined.layer_marker, "refinement-cascade");
        assert_eq!(project_refined_to_legacy_v0(&refined), legacy);
    }

    #[test]
    fn refinement_property_grammar_evaluates_exact_and_one_of_values() {
        let exact = AbstractPropertyValueV0::Exact {
            property_name: "display".to_string(),
            value: "grid".to_string(),
            pseudo_state: None,
        };
        let predicate = RefinementPropertyPredicateV0::OneOfValues {
            property_name: "display".to_string(),
            values: vec!["grid".to_string(), "flex".to_string()],
        };
        let evaluation = evaluate_refinement_property_predicate_v0(&predicate, &exact);

        assert_eq!(evaluation.schema_version, "0");
        assert_eq!(
            evaluation.product,
            "omena-refinement.property-predicate-evaluation"
        );
        assert_eq!(evaluation.value_shape, AbstractValueShapeV0::Exact);
        assert_eq!(evaluation.verdict, RefinementVerdictV0::SatisfiedAll);
        assert_eq!(evaluation.matched_clause_count, 1);
        assert_eq!(evaluation.witness.predicate_id, "property-grammar");
        assert!(evaluation.witness.legacy_proofs_byte_untouched);
    }

    #[test]
    fn refinement_predicate_composition_tracks_partial_and_negative_witnesses() {
        let finite = AbstractPropertyValueV0::FiniteSet {
            property_name: "color".to_string(),
            values: vec!["red".to_string(), "blue".to_string()],
            pseudo_states: Vec::new(),
        };
        let predicate = RefinementPropertyPredicateV0::And {
            predicates: vec![
                RefinementPropertyPredicateV0::OneOfValues {
                    property_name: "color".to_string(),
                    values: vec!["red".to_string()],
                },
                RefinementPropertyPredicateV0::Not {
                    predicate: Box::new(RefinementPropertyPredicateV0::ExactValue {
                        property_name: "color".to_string(),
                        value: "green".to_string(),
                    }),
                },
            ],
        };
        let evaluation = evaluate_refinement_property_predicate_v0(&predicate, &finite);

        assert_eq!(evaluation.value_shape, AbstractValueShapeV0::FiniteSet);
        assert_eq!(evaluation.verdict, RefinementVerdictV0::SatisfiedSome);
        assert_eq!(evaluation.matched_clause_count, 2);
        assert_eq!(
            evaluation.predicate_expression_id,
            "and(one-of:color:red,not(exact:color:green))"
        );
        assert_eq!(evaluation.witness.provenance[0].source, "property-grammar");
    }

    #[test]
    fn refinement_custom_property_reference_predicate_is_not_wrapper_only() {
        let reference = AbstractPropertyValueV0::CustomPropertyReference {
            property_name: "color".to_string(),
            custom_property_name: "--brand".to_string(),
            pseudo_state: None,
        };
        let predicate = RefinementPropertyPredicateV0::CustomPropertyReference {
            property_name: "color".to_string(),
            custom_property_name: "--brand".to_string(),
        };
        let evaluation = evaluate_refinement_property_predicate_v0(&predicate, &reference);

        assert_eq!(evaluation.verdict, RefinementVerdictV0::SatisfiedAll);
        assert_eq!(
            evaluation.predicate_expression_id,
            "custom-ref:color:--brand"
        );
    }
}
