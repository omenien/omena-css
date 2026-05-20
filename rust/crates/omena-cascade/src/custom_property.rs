use std::collections::BTreeSet;

use crate::{
    CascadeValue, CustomPropertyEnv, CustomPropertyLeastFixedPointEntryV0,
    CustomPropertyLeastFixedPointIterationV0, CustomPropertyLeastFixedPointProofV0,
    CustomPropertyLeastFixedPointSummaryV0,
};

pub fn substitute_custom_properties(value: &CascadeValue, env: &CustomPropertyEnv) -> CascadeValue {
    let mut visiting = BTreeSet::new();
    substitute_custom_properties_inner(value, env, &mut visiting)
}

pub fn resolve_custom_property_env_least_fixed_point(env: &CustomPropertyEnv) -> CustomPropertyEnv {
    compute_custom_property_env_least_fixed_point(env).resolved_env
}

pub fn summarize_custom_property_least_fixed_point(
    env: &CustomPropertyEnv,
) -> CustomPropertyLeastFixedPointSummaryV0 {
    let computation = compute_custom_property_env_least_fixed_point(env);
    let entries = env
        .iter()
        .map(|(name, input)| {
            let resolved = computation
                .resolved_env
                .get(name)
                .cloned()
                .unwrap_or(CascadeValue::GuaranteedInvalid);
            CustomPropertyLeastFixedPointEntryV0 {
                name: name.clone(),
                input: input.clone(),
                changed: &resolved != input,
                guaranteed_invalid: resolved == CascadeValue::GuaranteedInvalid,
                resolved,
            }
        })
        .collect::<Vec<_>>();
    let resolved_count = entries
        .iter()
        .filter(|entry| cascade_value_is_resolved(&entry.resolved))
        .count();
    let guaranteed_invalid_count = entries
        .iter()
        .filter(|entry| entry.guaranteed_invalid)
        .count();

    CustomPropertyLeastFixedPointSummaryV0 {
        schema_version: "0",
        product: "omena-cascade.custom-property-least-fixed-point",
        input_count: env.len(),
        resolved_count,
        guaranteed_invalid_count,
        iteration_count: computation.iteration_count,
        iteration_bound: computation.iteration_bound,
        reached_fixed_point: computation.reached_fixed_point,
        monotone_witness_valid: custom_property_iteration_trace_is_monotone(
            &computation.iteration_trace,
        ),
        proof: custom_property_least_fixed_point_proof(),
        iteration_trace: computation.iteration_trace,
        entries,
        ready_surfaces: vec![
            "customPropertySubstitution",
            "customPropertyLeastFixedPoint",
            "customPropertyLeastFixedPointProof",
            "customPropertyLeastFixedPointTrace",
            "cycleToGuaranteedInvalid",
        ],
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CustomPropertyLeastFixedPointComputation {
    resolved_env: CustomPropertyEnv,
    iteration_count: usize,
    iteration_bound: usize,
    reached_fixed_point: bool,
    iteration_trace: Vec<CustomPropertyLeastFixedPointIterationV0>,
}

fn compute_custom_property_env_least_fixed_point(
    env: &CustomPropertyEnv,
) -> CustomPropertyLeastFixedPointComputation {
    let mut current = env.clone();
    let max_iterations = env.len().saturating_add(1).max(1);
    let mut iteration_trace = Vec::new();

    for iteration in 1..=max_iterations {
        let next = env
            .iter()
            .map(|(name, value)| (name.clone(), substitute_custom_properties(value, &current)))
            .collect::<CustomPropertyEnv>();
        iteration_trace.push(custom_property_least_fixed_point_iteration_witness(
            iteration, env, &next,
        ));
        if next == current {
            return CustomPropertyLeastFixedPointComputation {
                resolved_env: next,
                iteration_count: iteration,
                iteration_bound: max_iterations,
                reached_fixed_point: true,
                iteration_trace,
            };
        }
        current = next;
    }

    CustomPropertyLeastFixedPointComputation {
        resolved_env: current,
        iteration_count: max_iterations,
        iteration_bound: max_iterations,
        reached_fixed_point: false,
        iteration_trace,
    }
}

fn custom_property_least_fixed_point_iteration_witness(
    iteration: usize,
    input_env: &CustomPropertyEnv,
    resolved_env: &CustomPropertyEnv,
) -> CustomPropertyLeastFixedPointIterationV0 {
    let changed_count = input_env
        .iter()
        .filter(|(name, input)| {
            resolved_env
                .get(*name)
                .is_some_and(|resolved| resolved != *input)
        })
        .count();
    let settled_count = resolved_env
        .values()
        .filter(|value| !cascade_value_contains_var_reference(value))
        .count();
    let guaranteed_invalid_count = resolved_env
        .values()
        .filter(|value| **value == CascadeValue::GuaranteedInvalid)
        .count();

    CustomPropertyLeastFixedPointIterationV0 {
        iteration,
        changed_count,
        settled_count,
        guaranteed_invalid_count,
    }
}

fn custom_property_iteration_trace_is_monotone(
    trace: &[CustomPropertyLeastFixedPointIterationV0],
) -> bool {
    trace
        .windows(2)
        .all(|pair| pair[0].settled_count <= pair[1].settled_count)
}

fn cascade_value_contains_var_reference(value: &CascadeValue) -> bool {
    match value {
        CascadeValue::Var { .. } => true,
        CascadeValue::Composite(values) => values.iter().any(cascade_value_contains_var_reference),
        CascadeValue::Literal(_)
        | CascadeValue::Initial
        | CascadeValue::Inherit
        | CascadeValue::GuaranteedInvalid
        | CascadeValue::Unset => false,
    }
}

fn custom_property_least_fixed_point_proof() -> CustomPropertyLeastFixedPointProofV0 {
    CustomPropertyLeastFixedPointProofV0 {
        finite_domain: "custom-property environment keys are fixed during iteration",
        transfer_function: "each step substitutes every original binding against the previous environment approximation",
        monotone_witness: "iteration trace records a nondecreasing settled-value count across the fixed-key environment",
        iteration_bound_formula: "max(1, env.len() + 1)",
        cycle_policy: "recursive var() cycles are detected by the visiting set and collapsed to guaranteed-invalid or fallback",
        proof_obligations: vec![
            "fixed-key environment",
            "deterministic simultaneous transfer",
            "nondecreasing settled-value trace",
            "cycle-to-guaranteed-invalid bottoming",
            "finite iteration bound",
            "explicit fixed-point equality check",
        ],
    }
}

fn substitute_custom_properties_inner(
    value: &CascadeValue,
    env: &CustomPropertyEnv,
    visiting: &mut BTreeSet<String>,
) -> CascadeValue {
    match value {
        CascadeValue::Literal(_)
        | CascadeValue::Initial
        | CascadeValue::Inherit
        | CascadeValue::GuaranteedInvalid
        | CascadeValue::Unset => value.clone(),
        CascadeValue::Composite(parts) => {
            let resolved_parts = parts
                .iter()
                .map(|part| substitute_custom_properties_inner(part, env, visiting))
                .collect::<Vec<_>>();
            if resolved_parts.contains(&CascadeValue::GuaranteedInvalid) {
                return CascadeValue::GuaranteedInvalid;
            }
            CascadeValue::Composite(resolved_parts)
        }
        CascadeValue::Var { name, fallback } => {
            if !visiting.insert(name.clone()) {
                return CascadeValue::GuaranteedInvalid;
            }
            let resolved = match env.get(name) {
                Some(CascadeValue::Unset) | None => fallback
                    .as_deref()
                    .map(|fallback| substitute_custom_properties_inner(fallback, env, visiting))
                    .unwrap_or(CascadeValue::GuaranteedInvalid),
                Some(value) => {
                    let resolved = substitute_custom_properties_inner(value, env, visiting);
                    if resolved == CascadeValue::GuaranteedInvalid {
                        fallback
                            .as_deref()
                            .map(|fallback| {
                                substitute_custom_properties_inner(fallback, env, visiting)
                            })
                            .unwrap_or(CascadeValue::GuaranteedInvalid)
                    } else {
                        resolved
                    }
                }
            };
            visiting.remove(name);
            resolved
        }
    }
}

fn cascade_value_is_resolved(value: &CascadeValue) -> bool {
    match value {
        CascadeValue::Literal(_) => true,
        CascadeValue::Composite(parts) => parts.iter().all(cascade_value_is_resolved),
        CascadeValue::Var { .. }
        | CascadeValue::Initial
        | CascadeValue::Inherit
        | CascadeValue::GuaranteedInvalid
        | CascadeValue::Unset => false,
    }
}
