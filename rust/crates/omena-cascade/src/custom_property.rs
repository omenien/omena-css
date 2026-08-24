//! Custom-property substitution and dependency-graph computation summaries.
//!
//! Resolution decomposes the canonical custom-property dependency graph into
//! strongly connected components, invalidates every cyclic member, and then
//! evaluates the acyclic remainder in dependency order. Historical
//! proof-oriented API names remain as compatibility aliases.

use omena_syntax::ident::CanonicalCustomPropertyNameV0;
#[cfg(test)]
use std::cell::Cell;
use std::collections::{BTreeMap, HashMap, VecDeque};

use crate::{
    CascadeValue, CustomPropertyBoundedFixedPointComputationWitnessV0, CustomPropertyEnv,
    CustomPropertyGuaranteedInvalidReasonV0, CustomPropertyLeastFixedPointEntryV0,
    CustomPropertyLeastFixedPointIterationV0, CustomPropertyLeastFixedPointProofV0,
    CustomPropertyLeastFixedPointSummaryV0,
};

pub fn substitute_custom_properties(value: &CascadeValue, env: &CustomPropertyEnv) -> CascadeValue {
    let resolved_env = resolve_custom_property_env_least_fixed_point(env);
    substitute_custom_properties_against_resolved_env(value, &resolved_env)
}

pub fn resolve_custom_property_env_least_fixed_point(env: &CustomPropertyEnv) -> CustomPropertyEnv {
    compute_custom_property_env_least_fixed_point(env, TraceMode::Omit).resolved_env
}

pub fn summarize_custom_property_least_fixed_point(
    env: &CustomPropertyEnv,
) -> CustomPropertyLeastFixedPointSummaryV0 {
    let computation = compute_custom_property_env_least_fixed_point(env, TraceMode::Record);
    let entries = env
        .iter()
        .zip(computation.resolved_env.iter())
        .map(|((name, input), (resolved_name, resolved))| {
            assert_eq!(
                name, resolved_name,
                "the resolved environment must preserve the canonical input key set"
            );
            CustomPropertyLeastFixedPointEntryV0 {
                name: name.as_str().to_string(),
                input: input.clone(),
                changed: resolved != input,
                guaranteed_invalid: *resolved == CascadeValue::GuaranteedInvalid,
                guaranteed_invalid_reason: computation.invalid_reasons.get(name).copied(),
                resolved: resolved.clone(),
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
    invalid_reasons:
        BTreeMap<CanonicalCustomPropertyNameV0, CustomPropertyGuaranteedInvalidReasonV0>,
    iteration_count: usize,
    iteration_bound: usize,
    reached_fixed_point: bool,
    iteration_trace: Vec<CustomPropertyLeastFixedPointIterationV0>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TraceMode {
    Omit,
    Record,
}

fn compute_custom_property_env_least_fixed_point(
    env: &CustomPropertyEnv,
    trace_mode: TraceMode,
) -> CustomPropertyLeastFixedPointComputation {
    if env
        .values()
        .all(|value| !cascade_value_contains_var_reference(value))
    {
        let iteration_bound = env.len().max(1);
        let iteration_trace = match trace_mode {
            TraceMode::Omit => Vec::new(),
            TraceMode::Record if env.is_empty() => vec![CustomPropertyLeastFixedPointIterationV0 {
                iteration: 1,
                changed_count: 0,
                settled_count: 0,
                guaranteed_invalid_count: 0,
            }],
            TraceMode::Record => {
                let mut guaranteed_invalid_count = 0;
                env.iter()
                    .enumerate()
                    .map(|(index, (_, value))| {
                        guaranteed_invalid_count +=
                            usize::from(*value == CascadeValue::GuaranteedInvalid);
                        CustomPropertyLeastFixedPointIterationV0 {
                            iteration: index + 1,
                            changed_count: 0,
                            settled_count: index + 1,
                            guaranteed_invalid_count,
                        }
                    })
                    .collect()
            }
        };
        let invalid_reasons = env
            .iter()
            .filter(|(_, value)| **value == CascadeValue::GuaranteedInvalid)
            .map(|(name, _)| {
                (
                    name.clone(),
                    CustomPropertyGuaranteedInvalidReasonV0::InvalidDependencyWithoutFallback,
                )
            })
            .collect();
        return CustomPropertyLeastFixedPointComputation {
            resolved_env: env.clone(),
            invalid_reasons,
            iteration_count: iteration_bound,
            iteration_bound,
            reached_fixed_point: true,
            iteration_trace,
        };
    }
    let dependency_graph = custom_property_dependency_graph(env);
    let components = strongly_connected_components(&dependency_graph);
    let component_schedule = dependency_ordered_components(&dependency_graph, &components);
    let mut resolved_env = CustomPropertyEnv::new();
    let mut invalid_reasons = BTreeMap::new();
    let mut iteration_trace = match trace_mode {
        TraceMode::Omit => Vec::new(),
        TraceMode::Record => Vec::with_capacity(components.len().max(1)),
    };
    let mut changed_count = 0;
    let mut settled_count = 0;
    let mut guaranteed_invalid_count = 0;

    for component_index in component_schedule {
        let component = &components[component_index];
        if component_is_cyclic(component, &dependency_graph) {
            for node in component {
                let name = dependency_graph.names[*node];
                let input = dependency_graph.values[*node];
                let resolved = CascadeValue::GuaranteedInvalid;
                record_custom_property_settlement(
                    trace_mode,
                    input,
                    &resolved,
                    &mut changed_count,
                    &mut settled_count,
                    &mut guaranteed_invalid_count,
                );
                resolved_env.insert(name.clone(), CascadeValue::GuaranteedInvalid);
                invalid_reasons.insert(
                    name.clone(),
                    CustomPropertyGuaranteedInvalidReasonV0::CycleMember,
                );
            }
        } else {
            for node in component {
                let name = dependency_graph.names[*node];
                let value = dependency_graph.values[*node];
                let outcome = substitute_custom_properties_with_reason(
                    value,
                    &resolved_env,
                    &invalid_reasons,
                );
                if let Some(reason) = outcome.invalid_reason {
                    invalid_reasons.insert(name.clone(), reason);
                }
                record_custom_property_settlement(
                    trace_mode,
                    value,
                    &outcome.value,
                    &mut changed_count,
                    &mut settled_count,
                    &mut guaranteed_invalid_count,
                );
                resolved_env.insert(name.clone(), outcome.value);
            }
        }

        if trace_mode == TraceMode::Record {
            iteration_trace.push(CustomPropertyLeastFixedPointIterationV0 {
                iteration: iteration_trace.len() + 1,
                changed_count,
                settled_count,
                guaranteed_invalid_count,
            });
        }
    }

    if trace_mode == TraceMode::Record && iteration_trace.is_empty() {
        iteration_trace.push(CustomPropertyLeastFixedPointIterationV0 {
            iteration: 1,
            changed_count: 0,
            settled_count: 0,
            guaranteed_invalid_count: 0,
        });
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
        invalid_reasons,
        iteration_count: components.len().max(1),
        iteration_bound: components.len().max(1),
        reached_fixed_point: true,
        iteration_trace,
    }
}

struct CustomPropertyDependencyGraph<'a> {
    names: Vec<&'a CanonicalCustomPropertyNameV0>,
    values: Vec<&'a CascadeValue>,
    edges: Vec<Vec<usize>>,
}

#[cfg(test)]
thread_local! {
    static DEPENDENCY_GRAPH_BUILD_COUNT: Cell<usize> = const { Cell::new(0) };
}

#[cfg(test)]
pub(crate) fn reset_custom_property_dependency_graph_build_count() {
    DEPENDENCY_GRAPH_BUILD_COUNT.set(0);
}

#[cfg(test)]
pub(crate) fn custom_property_dependency_graph_build_count() -> usize {
    DEPENDENCY_GRAPH_BUILD_COUNT.get()
}

fn custom_property_dependency_graph(env: &CustomPropertyEnv) -> CustomPropertyDependencyGraph<'_> {
    #[cfg(test)]
    DEPENDENCY_GRAPH_BUILD_COUNT.set(DEPENDENCY_GRAPH_BUILD_COUNT.get() + 1);
    let entries = env.iter().collect::<Vec<_>>();
    let names = entries.iter().map(|(name, _)| *name).collect::<Vec<_>>();
    let values = entries.iter().map(|(_, value)| *value).collect::<Vec<_>>();
    let index_by_name = names
        .iter()
        .enumerate()
        .map(|(index, name)| (name.as_str(), index))
        .collect::<HashMap<_, _>>();
    let edges = values
        .iter()
        .map(|value| {
            let mut references = Vec::new();
            collect_custom_property_reference_indices(value, &index_by_name, &mut references);
            references.sort_unstable();
            references.dedup();
            references
        })
        .collect();
    CustomPropertyDependencyGraph {
        names,
        values,
        edges,
    }
}

#[inline]
fn record_custom_property_settlement(
    trace_mode: TraceMode,
    input: &CascadeValue,
    resolved: &CascadeValue,
    changed_count: &mut usize,
    settled_count: &mut usize,
    guaranteed_invalid_count: &mut usize,
) {
    if trace_mode == TraceMode::Omit {
        return;
    }
    *changed_count += usize::from(input != resolved);
    *settled_count += usize::from(!cascade_value_contains_var_reference(resolved));
    *guaranteed_invalid_count += usize::from(*resolved == CascadeValue::GuaranteedInvalid);
}

fn collect_custom_property_reference_indices(
    value: &CascadeValue,
    index_by_name: &HashMap<&str, usize>,
    references: &mut Vec<usize>,
) {
    match value {
        CascadeValue::Var { name, fallback } => {
            if let Some(index) = index_by_name.get(name.as_str()) {
                references.push(*index);
            }
            if let Some(fallback) = fallback {
                collect_custom_property_reference_indices(fallback, index_by_name, references);
            }
        }
        CascadeValue::Composite(parts) => {
            for part in parts {
                collect_custom_property_reference_indices(part, index_by_name, references);
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

fn strongly_connected_components(graph: &CustomPropertyDependencyGraph<'_>) -> Vec<Vec<usize>> {
    let mut finish_order = Vec::with_capacity(graph.names.len());
    let mut visited = vec![false; graph.names.len()];
    for start in 0..graph.names.len() {
        if visited[start] {
            continue;
        }
        visited[start] = true;
        let mut stack = vec![(start, 0usize)];
        while let Some((node, neighbor_index)) = stack.last_mut() {
            if let Some(neighbor) = graph.edges[*node].get(*neighbor_index).copied() {
                *neighbor_index += 1;
                if !visited[neighbor] {
                    visited[neighbor] = true;
                    stack.push((neighbor, 0));
                }
            } else {
                let node = *node;
                stack.pop();
                finish_order.push(node);
            }
        }
    }

    let mut reverse_graph = vec![Vec::new(); graph.names.len()];
    for (source, targets) in graph.edges.iter().enumerate() {
        for target in targets {
            reverse_graph[*target].push(source);
        }
    }
    for neighbors in &mut reverse_graph {
        neighbors.sort_unstable();
    }

    let mut components = Vec::new();
    visited.fill(false);
    while let Some(node) = finish_order.pop() {
        if visited[node] {
            continue;
        }
        let mut component = Vec::new();
        visited[node] = true;
        let mut stack = vec![node];
        while let Some(current) = stack.pop() {
            component.push(current);
            for neighbor in reverse_graph[current].iter().rev() {
                if !visited[*neighbor] {
                    visited[*neighbor] = true;
                    stack.push(*neighbor);
                }
            }
        }
        component.sort_unstable_by_key(|index| graph.names[*index].as_str());
        components.push(component);
    }
    components
}

fn dependency_ordered_components(
    graph: &CustomPropertyDependencyGraph<'_>,
    components: &[Vec<usize>],
) -> Vec<usize> {
    let mut component_by_node = vec![0; graph.names.len()];
    for (component_index, component) in components.iter().enumerate() {
        for node in component {
            component_by_node[*node] = component_index;
        }
    }
    let mut component_dependencies = vec![Vec::new(); components.len()];
    for (source, targets) in graph.edges.iter().enumerate() {
        let source_component = component_by_node[source];
        for target in targets {
            let target_component = component_by_node[*target];
            if source_component != target_component {
                component_dependencies[source_component].push(target_component);
            }
        }
    }
    for dependencies in &mut component_dependencies {
        dependencies.sort_unstable();
        dependencies.dedup();
    }
    let mut dependents = vec![Vec::new(); components.len()];
    for (component, dependencies) in component_dependencies.iter().enumerate() {
        for dependency in dependencies {
            dependents[*dependency].push(component);
        }
    }
    for entries in &mut dependents {
        entries.sort_unstable();
    }
    let mut remaining_dependencies = component_dependencies
        .iter()
        .map(Vec::len)
        .collect::<Vec<_>>();
    let mut ready = (0..components.len())
        .filter(|index| remaining_dependencies[*index] == 0)
        .collect::<VecDeque<_>>();
    let mut schedule = Vec::with_capacity(components.len());
    while let Some(component) = ready.pop_front() {
        schedule.push(component);
        for dependent in &dependents[component] {
            remaining_dependencies[*dependent] -= 1;
            if remaining_dependencies[*dependent] == 0 {
                ready.push_back(*dependent);
            }
        }
    }
    assert_eq!(
        schedule.len(),
        components.len(),
        "the SCC condensation graph is acyclic"
    );
    schedule
}

fn component_is_cyclic(component: &[usize], graph: &CustomPropertyDependencyGraph<'_>) -> bool {
    component.len() > 1
        || component
            .first()
            .is_some_and(|node| graph.edges[*node].contains(node))
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

#[derive(Debug, Clone, PartialEq, Eq)]
struct CustomPropertySubstitutionOutcome {
    value: CascadeValue,
    invalid_reason: Option<CustomPropertyGuaranteedInvalidReasonV0>,
}

fn substitute_custom_properties_with_reason(
    value: &CascadeValue,
    resolved_env: &CustomPropertyEnv,
    invalid_reasons: &BTreeMap<
        CanonicalCustomPropertyNameV0,
        CustomPropertyGuaranteedInvalidReasonV0,
    >,
) -> CustomPropertySubstitutionOutcome {
    match value {
        CascadeValue::Literal(_)
        | CascadeValue::Initial
        | CascadeValue::Inherit
        | CascadeValue::Indeterminate
        | CascadeValue::Unset => CustomPropertySubstitutionOutcome {
            value: value.clone(),
            invalid_reason: None,
        },
        CascadeValue::GuaranteedInvalid => CustomPropertySubstitutionOutcome {
            value: CascadeValue::GuaranteedInvalid,
            invalid_reason: Some(
                CustomPropertyGuaranteedInvalidReasonV0::InvalidDependencyWithoutFallback,
            ),
        },
        CascadeValue::Composite(parts) => {
            let mut resolved_parts = Vec::with_capacity(parts.len());
            let mut invalid_reason = None;
            for part in parts {
                let outcome =
                    substitute_custom_properties_with_reason(part, resolved_env, invalid_reasons);
                invalid_reason = invalid_reason.or(outcome.invalid_reason);
                resolved_parts.push(outcome.value);
            }
            if let Some(invalid_reason) = invalid_reason {
                CustomPropertySubstitutionOutcome {
                    value: CascadeValue::GuaranteedInvalid,
                    invalid_reason: Some(invalid_reason),
                }
            } else {
                CustomPropertySubstitutionOutcome {
                    value: CascadeValue::Composite(resolved_parts),
                    invalid_reason: None,
                }
            }
        }
        CascadeValue::Var { name, fallback } => match resolved_env.get(name) {
            Some(CascadeValue::Unset | CascadeValue::GuaranteedInvalid) => fallback
                .as_deref()
                .map(|fallback| {
                    substitute_custom_properties_with_reason(
                        fallback,
                        resolved_env,
                        invalid_reasons,
                    )
                })
                .unwrap_or(CustomPropertySubstitutionOutcome {
                    value: CascadeValue::GuaranteedInvalid,
                    invalid_reason: Some(
                        CustomPropertyGuaranteedInvalidReasonV0::InvalidDependencyWithoutFallback,
                    ),
                }),
            None => fallback
                .as_deref()
                .map(|fallback| {
                    substitute_custom_properties_with_reason(
                        fallback,
                        resolved_env,
                        invalid_reasons,
                    )
                })
                .unwrap_or(CustomPropertySubstitutionOutcome {
                    value: CascadeValue::GuaranteedInvalid,
                    invalid_reason: Some(CustomPropertyGuaranteedInvalidReasonV0::MissingReference),
                }),
            Some(resolved) => CustomPropertySubstitutionOutcome {
                value: resolved.clone(),
                invalid_reason: invalid_reasons.get(name).copied(),
            },
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
