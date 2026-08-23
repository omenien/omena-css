//! Custom-property substitution and dependency-graph computation summaries.
//!
//! Resolution decomposes the canonical custom-property dependency graph into
//! strongly connected components, invalidates every cyclic member, and then
//! evaluates the acyclic remainder in dependency order. Historical
//! proof-oriented API names remain as compatibility aliases.

use omena_syntax::ident::CanonicalCustomPropertyNameV0;
use std::collections::{BTreeMap, BTreeSet};

use crate::{
    CascadeValue, CustomPropertyBoundedFixedPointComputationWitnessV0, CustomPropertyEnv,
    CustomPropertyLeastFixedPointEntryV0, CustomPropertyLeastFixedPointIterationV0,
    CustomPropertyLeastFixedPointProofV0, CustomPropertyLeastFixedPointSummaryV0,
};

pub fn substitute_custom_properties(value: &CascadeValue, env: &CustomPropertyEnv) -> CascadeValue {
    let resolved_env = resolve_custom_property_env_least_fixed_point(env);
    substitute_custom_properties_against_resolved_env(value, &resolved_env)
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
                name: name.as_str().to_string(),
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
    let dependency_graph = custom_property_dependency_graph(env);
    let components = strongly_connected_components(&dependency_graph);
    let component_schedule = dependency_ordered_components(&dependency_graph, &components);
    let mut resolved_env = CustomPropertyEnv::new();
    let mut iteration_trace = Vec::new();

    for component_index in component_schedule {
        let component = &components[component_index];
        if component_is_cyclic(component, &dependency_graph) {
            for name in component {
                resolved_env.insert(name.clone(), CascadeValue::GuaranteedInvalid);
            }
        } else {
            for name in component {
                let Some(value) = env.get(name) else {
                    continue;
                };
                let resolved =
                    substitute_custom_properties_against_resolved_env(value, &resolved_env);
                resolved_env.insert(name.clone(), resolved);
            }
        }

        let iteration = iteration_trace.len() + 1;
        iteration_trace.push(custom_property_least_fixed_point_iteration_witness(
            iteration,
            env,
            &resolved_env,
        ));
    }

    if iteration_trace.is_empty() {
        iteration_trace.push(custom_property_least_fixed_point_iteration_witness(
            1,
            env,
            &resolved_env,
        ));
    }

    assert_eq!(
        resolved_env.len(),
        env.len(),
        "the SCC schedule must evaluate every custom-property binding exactly once"
    );
    assert!(
        resolved_env
            .values()
            .all(|value| !cascade_value_contains_var_reference(value)),
        "the acyclic component schedule must eliminate every var() reference"
    );

    CustomPropertyLeastFixedPointComputation {
        resolved_env,
        iteration_count: iteration_trace.len(),
        iteration_bound: components.len().max(1),
        reached_fixed_point: true,
        iteration_trace,
    }
}

type CustomPropertyDependencyGraph =
    BTreeMap<CanonicalCustomPropertyNameV0, BTreeSet<CanonicalCustomPropertyNameV0>>;

fn custom_property_dependency_graph(env: &CustomPropertyEnv) -> CustomPropertyDependencyGraph {
    env.iter()
        .map(|(name, value)| {
            let mut references = BTreeSet::new();
            collect_custom_property_references(value, &mut references);
            references.retain(|reference| env.contains_key(reference));
            (name.clone(), references)
        })
        .collect()
}

fn collect_custom_property_references(
    value: &CascadeValue,
    references: &mut BTreeSet<CanonicalCustomPropertyNameV0>,
) {
    match value {
        CascadeValue::Var { name, fallback } => {
            references.insert(name.clone());
            if let Some(fallback) = fallback {
                collect_custom_property_references(fallback, references);
            }
        }
        CascadeValue::Composite(parts) => {
            for part in parts {
                collect_custom_property_references(part, references);
            }
        }
        CascadeValue::Literal(_)
        | CascadeValue::Initial
        | CascadeValue::Inherit
        | CascadeValue::Indeterminate
        | CascadeValue::GuaranteedInvalid
        | CascadeValue::Unset => {}
    }
}

fn strongly_connected_components(
    graph: &CustomPropertyDependencyGraph,
) -> Vec<Vec<CanonicalCustomPropertyNameV0>> {
    fn finish_visit(
        node: &CanonicalCustomPropertyNameV0,
        graph: &CustomPropertyDependencyGraph,
        visited: &mut BTreeSet<CanonicalCustomPropertyNameV0>,
        finish_order: &mut Vec<CanonicalCustomPropertyNameV0>,
    ) {
        if !visited.insert(node.clone()) {
            return;
        }
        if let Some(neighbors) = graph.get(node) {
            for neighbor in neighbors {
                finish_visit(neighbor, graph, visited, finish_order);
            }
        }
        finish_order.push(node.clone());
    }

    fn collect_reverse_component(
        node: &CanonicalCustomPropertyNameV0,
        reverse_graph: &CustomPropertyDependencyGraph,
        visited: &mut BTreeSet<CanonicalCustomPropertyNameV0>,
        component: &mut Vec<CanonicalCustomPropertyNameV0>,
    ) {
        if !visited.insert(node.clone()) {
            return;
        }
        component.push(node.clone());
        if let Some(neighbors) = reverse_graph.get(node) {
            for neighbor in neighbors {
                collect_reverse_component(neighbor, reverse_graph, visited, component);
            }
        }
    }

    let mut finish_order = Vec::with_capacity(graph.len());
    let mut visited = BTreeSet::new();
    for node in graph.keys() {
        finish_visit(node, graph, &mut visited, &mut finish_order);
    }

    let mut reverse_graph = graph
        .keys()
        .cloned()
        .map(|name| (name, BTreeSet::new()))
        .collect::<CustomPropertyDependencyGraph>();
    for (source, targets) in graph {
        for target in targets {
            reverse_graph
                .entry(target.clone())
                .or_default()
                .insert(source.clone());
        }
    }

    let mut components = Vec::new();
    visited.clear();
    while let Some(node) = finish_order.pop() {
        if visited.contains(&node) {
            continue;
        }
        let mut component = Vec::new();
        collect_reverse_component(&node, &reverse_graph, &mut visited, &mut component);
        component.sort();
        components.push(component);
    }
    components
}

fn dependency_ordered_components(
    graph: &CustomPropertyDependencyGraph,
    components: &[Vec<CanonicalCustomPropertyNameV0>],
) -> Vec<usize> {
    fn visit_component(
        component_index: usize,
        component_dependencies: &[BTreeSet<usize>],
        visited: &mut BTreeSet<usize>,
        schedule: &mut Vec<usize>,
    ) {
        if !visited.insert(component_index) {
            return;
        }
        for dependency in &component_dependencies[component_index] {
            visit_component(*dependency, component_dependencies, visited, schedule);
        }
        schedule.push(component_index);
    }

    let component_by_name = components
        .iter()
        .enumerate()
        .flat_map(|(index, component)| component.iter().cloned().map(move |name| (name, index)))
        .collect::<BTreeMap<_, _>>();
    let mut component_dependencies = vec![BTreeSet::new(); components.len()];
    for (source, targets) in graph {
        let Some(source_component) = component_by_name.get(source).copied() else {
            continue;
        };
        for target in targets {
            let Some(target_component) = component_by_name.get(target).copied() else {
                continue;
            };
            if source_component != target_component {
                component_dependencies[source_component].insert(target_component);
            }
        }
    }

    let mut schedule = Vec::with_capacity(components.len());
    let mut visited = BTreeSet::new();
    for component_index in 0..components.len() {
        visit_component(
            component_index,
            &component_dependencies,
            &mut visited,
            &mut schedule,
        );
    }
    schedule
}

fn component_is_cyclic(
    component: &[CanonicalCustomPropertyNameV0],
    graph: &CustomPropertyDependencyGraph,
) -> bool {
    component.len() > 1
        || component.first().is_some_and(|name| {
            graph
                .get(name)
                .is_some_and(|neighbors| neighbors.contains(name))
        })
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
        | CascadeValue::Indeterminate
        | CascadeValue::GuaranteedInvalid
        | CascadeValue::Unset => false,
    }
}

/// Describes the finite graph computation used by custom-property substitution.
pub fn custom_property_bounded_fixed_point_computation_witness()
-> CustomPropertyBoundedFixedPointComputationWitnessV0 {
    CustomPropertyLeastFixedPointProofV0 {
        finite_domain: "canonical custom-property environment keys form a fixed finite dependency graph",
        transfer_function: "strongly connected components are scheduled dependency-first; cyclic components become guaranteed-invalid and acyclic components substitute against memoized dependencies",
        bounded_fixed_point_computation_witness: "every strongly connected component is processed exactly once; no non-converged approximation is returned",
        monotone_witness: "the compatibility trace records a nondecreasing count of bindings settled by the component schedule",
        monotonic_progress_witness: "each scheduled component only adds finalized bindings to the resolved environment",
        iteration_bound_formula: "max(1, strongly_connected_component_count)",
        cycle_policy: "fallback references are dependency edges and every member of a cyclic strongly connected component becomes guaranteed-invalid before outer fallbacks are evaluated",
        proof_obligations: vec![
            "canonical-key dependency graph",
            "fallback-inclusive dependency edges",
            "complete strongly connected component partition",
            "whole-cycle guaranteed-invalid assignment",
            "dependency-ordered acyclic substitution",
            "no non-converged approximation return",
        ],
    }
}

/// Compatibility wrapper for the earlier proof-oriented name.
fn custom_property_least_fixed_point_proof() -> CustomPropertyLeastFixedPointProofV0 {
    custom_property_bounded_fixed_point_computation_witness()
}

fn substitute_custom_properties_against_resolved_env(
    value: &CascadeValue,
    resolved_env: &CustomPropertyEnv,
) -> CascadeValue {
    match value {
        CascadeValue::Literal(_)
        | CascadeValue::Initial
        | CascadeValue::Inherit
        | CascadeValue::Indeterminate
        | CascadeValue::GuaranteedInvalid
        | CascadeValue::Unset => value.clone(),
        CascadeValue::Composite(parts) => {
            let resolved_parts = parts
                .iter()
                .map(|part| substitute_custom_properties_against_resolved_env(part, resolved_env))
                .collect::<Vec<_>>();
            if resolved_parts.contains(&CascadeValue::GuaranteedInvalid) {
                return CascadeValue::GuaranteedInvalid;
            }
            CascadeValue::Composite(resolved_parts)
        }
        CascadeValue::Var { name, fallback } => match resolved_env.get(name) {
            Some(CascadeValue::Unset | CascadeValue::GuaranteedInvalid) | None => fallback
                .as_deref()
                .map(|fallback| {
                    substitute_custom_properties_against_resolved_env(fallback, resolved_env)
                })
                .unwrap_or(CascadeValue::GuaranteedInvalid),
            Some(value) => value.clone(),
        },
    }
}

fn cascade_value_is_resolved(value: &CascadeValue) -> bool {
    match value {
        CascadeValue::Literal(_) => true,
        CascadeValue::Composite(parts) => parts.iter().all(cascade_value_is_resolved),
        CascadeValue::Var { .. }
        | CascadeValue::Initial
        | CascadeValue::Inherit
        | CascadeValue::Indeterminate
        | CascadeValue::GuaranteedInvalid
        | CascadeValue::Unset => false,
    }
}
