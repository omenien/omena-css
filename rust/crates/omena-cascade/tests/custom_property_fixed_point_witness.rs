use std::{collections::BTreeMap, error::Error};

use omena_cascade::{
    CascadeValue, CustomPropertyEnv, resolve_custom_property_env_least_fixed_point,
    summarize_custom_property_least_fixed_point,
};
use omena_syntax::ident::PropertyNameV0;
use serde::{Deserialize, Serialize};

const CORPUS_JSON: &str = include_str!("fixtures/custom-property-fixed-point-witness-v1.json");
const PERTURBATION_ENV: &str = "OMENA_CUSTOM_PROPERTY_WITNESS_PERTURBATION";

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WitnessCorpus {
    schema_version: String,
    product: String,
    oracle: FrozenOracleDefinition,
    cases: Vec<WitnessCase>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FrozenOracleDefinition {
    status_order: String,
    environment_order: String,
    initial_approximation: String,
    transfer: String,
    unresolved_at_fixed_point: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WitnessCase {
    id: String,
    cycle_shape: Option<String>,
    bindings: BTreeMap<String, FixtureValue>,
    expected_implementation: BTreeMap<String, FixtureValue>,
    expected_evaluator: BTreeMap<String, FixtureValue>,
    expected_implementation_iterations: usize,
    expected_evaluator_iterations: usize,
    expected_implementation_reached_fixed_point: bool,
    expected_disposition: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
enum FixtureValue {
    Literal(String),
    Composite(Vec<FixtureValue>),
    Var {
        name: String,
        fallback: Option<Box<FixtureValue>>,
    },
    Initial,
    Inherit,
    Indeterminate,
    GuaranteedInvalid,
    Unset,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum OracleStatus {
    Unresolved,
    Resolved(FixtureValue),
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct WitnessReceipt {
    schema_version: &'static str,
    product: &'static str,
    case_count: usize,
    agreement_count: usize,
    finding_count: usize,
    novel_cycle_case_count: usize,
    implementation_start: &'static str,
    evaluator_start: &'static str,
    rows: Vec<WitnessRow>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct WitnessRow {
    id: String,
    cycle_shape: Option<String>,
    implementation_iterations: usize,
    evaluator_iterations: usize,
    implementation_reached_fixed_point: bool,
    disposition: &'static str,
    implementation: BTreeMap<String, FixtureValue>,
    evaluator: BTreeMap<String, FixtureValue>,
}

#[test]
fn custom_property_fixed_point_witness_corpus_matches_frozen_oracle() -> Result<(), Box<dyn Error>>
{
    let corpus: WitnessCorpus = serde_json::from_str(CORPUS_JSON)?;
    assert_eq!(corpus.schema_version, "1");
    assert_eq!(
        corpus.product,
        "omena-cascade.custom-property-fixed-point-witness-corpus"
    );
    assert_eq!(
        corpus.oracle.status_order,
        "unresolvedBottomLeResolvedValue"
    );
    assert_eq!(corpus.oracle.environment_order, "pointwise");
    assert_eq!(corpus.oracle.initial_approximation, "allBottom");
    assert_eq!(
        corpus.oracle.transfer,
        "simultaneousOriginalBindingEvaluation"
    );
    assert_eq!(corpus.oracle.unresolved_at_fixed_point, "guaranteedInvalid");

    let perturbation = std::env::var(PERTURBATION_ENV).ok();
    assert!(
        perturbation
            .as_deref()
            .is_none_or(|value| value == "reordered-in-place"),
        "unknown independent evaluator perturbation"
    );
    let reordered_in_place = perturbation.as_deref() == Some("reordered-in-place");
    let mut rows = Vec::with_capacity(corpus.cases.len());

    for case in corpus.cases {
        let input = case
            .bindings
            .iter()
            .map(|(name, value)| {
                (
                    PropertyNameV0::canonical_custom_key(name),
                    to_cascade_value(value),
                )
            })
            .collect::<CustomPropertyEnv>();
        let implementation_env = resolve_custom_property_env_least_fixed_point(&input);
        let implementation_summary = summarize_custom_property_least_fixed_point(&input);
        let implementation = implementation_env
            .iter()
            .map(|(name, value)| (name.as_str().to_string(), from_cascade_value(value)))
            .collect::<BTreeMap<_, _>>();
        let summary_env = implementation_summary
            .entries
            .iter()
            .map(|entry| {
                (
                    entry.name.as_str().to_string(),
                    from_cascade_value(&entry.resolved),
                )
            })
            .collect::<BTreeMap<_, _>>();
        assert_eq!(
            implementation, summary_env,
            "{} summary projection",
            case.id
        );

        let (evaluator, evaluator_iterations) =
            evaluate_from_all_bottom(&case.bindings, reordered_in_place);
        let disposition = if implementation == evaluator {
            "agreement"
        } else {
            "finding"
        };
        assert_eq!(
            implementation, case.expected_implementation,
            "{} implementation projection",
            case.id
        );
        assert_eq!(
            evaluator, case.expected_evaluator,
            "{} all-bottom evaluator projection",
            case.id
        );
        assert_eq!(
            implementation_summary.iteration_count, case.expected_implementation_iterations,
            "{} implementation iterations",
            case.id
        );
        assert_eq!(
            evaluator_iterations, case.expected_evaluator_iterations,
            "{} evaluator iterations",
            case.id
        );
        assert_eq!(
            implementation_summary.reached_fixed_point,
            case.expected_implementation_reached_fixed_point,
            "{} implementation bound disposition",
            case.id
        );
        assert_eq!(
            disposition, case.expected_disposition,
            "{} disposition",
            case.id
        );

        rows.push(WitnessRow {
            id: case.id,
            cycle_shape: case.cycle_shape,
            implementation_iterations: implementation_summary.iteration_count,
            evaluator_iterations,
            implementation_reached_fixed_point: implementation_summary.reached_fixed_point,
            disposition,
            implementation,
            evaluator,
        });
    }

    let receipt = WitnessReceipt {
        schema_version: "1",
        product: "omena-cascade.custom-property-fixed-point-witness",
        case_count: rows.len(),
        agreement_count: rows
            .iter()
            .filter(|row| row.disposition == "agreement")
            .count(),
        finding_count: rows
            .iter()
            .filter(|row| row.disposition == "finding")
            .count(),
        novel_cycle_case_count: rows
            .iter()
            .filter(|row| {
                row.cycle_shape.as_deref().is_some_and(|shape| {
                    matches!(
                        shape,
                        "mutuallyRecursiveFallbackChain"
                            | "cycleThroughFallback"
                            | "threeNodeFallbackCycleEnteredMidChain"
                    )
                })
            })
            .count(),
        implementation_start: "dependencyGraphSccSchedule",
        evaluator_start: "allBottom",
        rows,
    };
    assert!(receipt.case_count >= 6);
    assert_eq!(receipt.agreement_count, receipt.case_count);
    assert_eq!(receipt.finding_count, 0);
    assert_eq!(receipt.novel_cycle_case_count, 3);
    assert_eq!(
        receipt
            .rows
            .iter()
            .filter_map(|row| row.cycle_shape.as_deref())
            .collect::<Vec<_>>(),
        vec![
            "mutualReferenceWithoutFallback",
            "mutuallyRecursiveFallbackChain",
            "cycleThroughFallback",
            "threeNodeFallbackCycleEnteredMidChain",
        ]
    );
    println!("{}", serde_json::to_string(&receipt)?);
    Ok(())
}

fn evaluate_from_all_bottom(
    bindings: &BTreeMap<String, FixtureValue>,
    reordered_in_place: bool,
) -> (BTreeMap<String, FixtureValue>, usize) {
    let mut current = bindings
        .keys()
        .map(|name| (name.clone(), OracleStatus::Unresolved))
        .collect::<BTreeMap<_, _>>();
    let iteration_bound = bindings.len().saturating_add(1).max(1);

    for iteration in 1..=iteration_bound {
        let next = if reordered_in_place {
            let mut in_place = current.clone();
            for (name, value) in bindings.iter().rev() {
                let status = evaluate_fixture_value(value, &in_place);
                in_place.insert(name.clone(), status);
            }
            in_place
        } else {
            bindings
                .iter()
                .map(|(name, value)| (name.clone(), evaluate_fixture_value(value, &current)))
                .collect::<BTreeMap<_, _>>()
        };
        if next == current {
            return (finalize_oracle_environment(next), iteration);
        }
        current = next;
    }

    (finalize_oracle_environment(current), iteration_bound)
}

fn evaluate_fixture_value(
    value: &FixtureValue,
    approximation: &BTreeMap<String, OracleStatus>,
) -> OracleStatus {
    match value {
        FixtureValue::Literal(_)
        | FixtureValue::Initial
        | FixtureValue::Inherit
        | FixtureValue::Indeterminate => OracleStatus::Resolved(value.clone()),
        FixtureValue::GuaranteedInvalid | FixtureValue::Unset => {
            OracleStatus::Resolved(FixtureValue::GuaranteedInvalid)
        }
        FixtureValue::Composite(parts) => {
            let evaluated = parts
                .iter()
                .map(|part| evaluate_fixture_value(part, approximation))
                .collect::<Vec<_>>();
            if evaluated.iter().any(|part| {
                matches!(
                    part,
                    OracleStatus::Resolved(FixtureValue::GuaranteedInvalid)
                )
            }) {
                OracleStatus::Resolved(FixtureValue::GuaranteedInvalid)
            } else if evaluated
                .iter()
                .any(|part| part == &OracleStatus::Unresolved)
            {
                OracleStatus::Unresolved
            } else {
                OracleStatus::Resolved(FixtureValue::Composite(
                    evaluated
                        .into_iter()
                        .filter_map(|part| match part {
                            OracleStatus::Resolved(value) => Some(value),
                            OracleStatus::Unresolved => None,
                        })
                        .collect(),
                ))
            }
        }
        FixtureValue::Var { name, fallback } => match approximation.get(name) {
            Some(OracleStatus::Resolved(FixtureValue::GuaranteedInvalid)) | None => {
                fallback.as_deref().map_or(
                    OracleStatus::Resolved(FixtureValue::GuaranteedInvalid),
                    |fallback| evaluate_fixture_value(fallback, approximation),
                )
            }
            Some(OracleStatus::Resolved(value)) => OracleStatus::Resolved(value.clone()),
            Some(OracleStatus::Unresolved) => OracleStatus::Unresolved,
        },
    }
}

fn finalize_oracle_environment(
    environment: BTreeMap<String, OracleStatus>,
) -> BTreeMap<String, FixtureValue> {
    environment
        .into_iter()
        .map(|(name, status)| {
            let value = match status {
                OracleStatus::Unresolved => FixtureValue::GuaranteedInvalid,
                OracleStatus::Resolved(value) => value,
            };
            (name, value)
        })
        .collect()
}

fn to_cascade_value(value: &FixtureValue) -> CascadeValue {
    match value {
        FixtureValue::Literal(value) => CascadeValue::Literal(value.clone()),
        FixtureValue::Composite(parts) => {
            CascadeValue::Composite(parts.iter().map(to_cascade_value).collect())
        }
        FixtureValue::Var { name, fallback } => CascadeValue::Var {
            name: PropertyNameV0::canonical_custom_key(name),
            fallback: fallback.as_deref().map(to_cascade_value).map(Box::new),
        },
        FixtureValue::Initial => CascadeValue::Initial,
        FixtureValue::Inherit => CascadeValue::Inherit,
        FixtureValue::Indeterminate => CascadeValue::Indeterminate,
        FixtureValue::GuaranteedInvalid => CascadeValue::GuaranteedInvalid,
        FixtureValue::Unset => CascadeValue::Unset,
    }
}

fn from_cascade_value(value: &CascadeValue) -> FixtureValue {
    match value {
        CascadeValue::Literal(value) => FixtureValue::Literal(value.clone()),
        CascadeValue::Composite(parts) => {
            FixtureValue::Composite(parts.iter().map(from_cascade_value).collect())
        }
        CascadeValue::Var { name, fallback } => FixtureValue::Var {
            name: name.as_str().to_string(),
            fallback: fallback.as_deref().map(from_cascade_value).map(Box::new),
        },
        CascadeValue::Initial => FixtureValue::Initial,
        CascadeValue::Inherit => FixtureValue::Inherit,
        CascadeValue::Indeterminate => FixtureValue::Indeterminate,
        CascadeValue::GuaranteedInvalid => FixtureValue::GuaranteedInvalid,
        CascadeValue::Unset => FixtureValue::Unset,
    }
}
