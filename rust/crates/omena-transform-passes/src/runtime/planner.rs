//! Transform pass registry, DAG planner, and public boundary summary.
//!
//! Planner code maps `omena-transform-cst` pass contracts into executable
//! registry entries, enforces default DAG ordering, and reports the mutation
//! passes that are implemented by the runtime executor.

use omena_transform_cst::{
    TRANSFORM_PASS_CATALOG_LEN, TransformDagEdgeV0, TransformLayer, TransformPassClassV0,
    TransformPassContractV0, TransformPassDescriptorV0, TransformPassKind,
    all_transform_pass_kinds, default_transform_dag_edges, default_transform_pass_contracts,
    default_transform_pass_descriptors, transform_build_profile_from_passes,
};

use crate::{
    TransformPassDispatchKindV0, TransformPassExecutionStatus, TransformPassPlanV0,
    TransformPassPlanningErrorV0, TransformPassRegistryEntryV0, TransformPassRegistryV0,
    TransformPassesBoundarySummaryV0, TransformPlanDependencyCycleV0,
    TransformPlanDependencyEdgeV0, TransformPlanPassConflictV0,
};

pub fn summarize_omena_transform_passes_boundary() -> TransformPassesBoundarySummaryV0 {
    let registry = default_transform_pass_registry();
    let registry_entries = registry.entries.clone();
    let pass_count = registry_entries.len();
    let semantic_aware_pass_count = registry_entries
        .iter()
        .filter(|entry| entry.contract.layer == TransformLayer::SemanticAware)
        .count();
    let cascade_aware_pass_count = registry_entries
        .iter()
        .filter(|entry| entry.contract.reads_cascade_model)
        .count();
    let structural_pass_count = registry_entries
        .iter()
        .filter(|entry| entry.descriptor.pass_class == TransformPassClassV0::Structural)
        .count();
    let text_local_pass_count = registry_entries
        .iter()
        .filter(|entry| entry.descriptor.pass_class == TransformPassClassV0::TextLocal)
        .count();
    let module_evaluation_pass_count = registry_entries
        .iter()
        .filter(|entry| entry.descriptor.pass_class == TransformPassClassV0::ModuleEvaluation)
        .count();

    TransformPassesBoundarySummaryV0 {
        schema_version: "0",
        product: "omena-transform-passes.boundary",
        registry_entries,
        dag_edges: default_transform_dag_edges(),
        pass_count,
        full_catalog_registered: pass_count == TRANSFORM_PASS_CATALOG_LEN,
        semantic_aware_pass_count,
        cascade_aware_pass_count,
        structural_pass_count,
        text_local_pass_count,
        module_evaluation_pass_count,
        planner_enforces_dag_edges: true,
        planner_uses_pass_descriptors: true,
        ordinal_has_execution_semantics: false,
        execution_runtime_ready: true,
        incremental_execution_runtime_ready: true,
        module_evaluation_native_output_marker: "nativeEditOutput",
        module_evaluation_requires_native_product_output: true,
        module_evaluation_requires_oracle_readiness: true,
        module_evaluation_legacy_output_is_oracle_only: true,
        module_evaluation_preserves_source_without_native_output: true,
        implemented_mutation_pass_ids: implemented_mutation_pass_ids(),
        next_surfaces: Vec::new(),
    }
}

pub fn plan_transform_passes(requested: &[TransformPassKind]) -> TransformPassPlanV0 {
    let registry = default_transform_pass_registry();
    let dag_edges = default_transform_dag_edges();
    plan_transform_passes_with_registry(requested, &registry, dag_edges.as_slice()).unwrap_or_else(
        |cycle| panic!("default transform pass registry contains a dependency cycle: {cycle:?}"),
    )
}

fn plan_transform_passes_with_registry(
    requested: &[TransformPassKind],
    registry: &TransformPassRegistryV0,
    dag_edges: &[TransformDagEdgeV0],
) -> Result<TransformPassPlanV0, TransformPlanDependencyCycleV0> {
    let requested_pass_ids = requested.iter().map(|pass| pass.id()).collect::<Vec<_>>();
    let requested_unique = dedupe_requested_passes(requested);
    let conflicting_unordered_pass_pairs = conflicting_unordered_pass_pairs(
        requested_unique.as_slice(),
        registry.entries.as_slice(),
        dag_edges,
    );
    let ordered_passes = order_passes_by_registry(requested, registry.entries.as_slice())?;
    let ordered_pass_ids = ordered_passes
        .iter()
        .map(|pass| pass.id())
        .collect::<Vec<_>>();
    let satisfied_dag_edge_count = dag_edges
        .iter()
        .filter(|edge| {
            edge_applies(edge, &ordered_pass_ids) && edge_is_satisfied(edge, &ordered_pass_ids)
        })
        .count();
    let violated_dag_edge_count = dag_edges
        .iter()
        .filter(|edge| {
            edge_applies(edge, &ordered_pass_ids) && !edge_is_satisfied(edge, &ordered_pass_ids)
        })
        .count();

    Ok(TransformPassPlanV0 {
        schema_version: "0",
        product: "omena-transform-passes.plan",
        build_profile: transform_build_profile_from_passes(
            "descriptor-ordered-transform-plan",
            ordered_passes.as_slice(),
        ),
        requested_pass_ids,
        ordered_pass_ids,
        satisfied_dag_edge_count,
        violated_dag_edge_count,
        all_requested_registered: requested
            .iter()
            .all(|pass| descriptor_for_pass(*pass, registry.entries.as_slice()).is_some()),
        conflicting_unordered_pass_pairs,
    })
}

pub fn plan_transform_passes_checked(
    requested: &[TransformPassKind],
) -> Result<TransformPassPlanV0, TransformPassPlanningErrorV0> {
    let registry = default_transform_pass_registry();
    let dag_edges = default_transform_dag_edges();
    plan_transform_passes_checked_with_registry(requested, &registry, dag_edges.as_slice())
}

fn plan_transform_passes_checked_with_registry(
    requested: &[TransformPassKind],
    registry: &TransformPassRegistryV0,
    dag_edges: &[TransformDagEdgeV0],
) -> Result<TransformPassPlanV0, TransformPassPlanningErrorV0> {
    let plan = plan_transform_passes_with_registry(requested, registry, dag_edges)
        .map_err(|cycle| TransformPassPlanningErrorV0::DependencyCycle { cycle })?;
    if let Some(conflict) = plan.conflicting_unordered_pass_pairs.first().cloned() {
        Err(TransformPassPlanningErrorV0::UnorderedPassConflict { conflict })
    } else {
        Ok(plan)
    }
}

#[cfg(feature = "transform-catalog-trace")]
pub fn plan_transform_passes_parallel_transform_catalog_layers(
    requested: &[TransformPassKind],
) -> omena_lawvere::TransformCatalogTransformPassParallelPlanV0 {
    omena_lawvere::plan_transform_catalog_parallel_layers_v0(requested)
}

#[cfg(feature = "transform-catalog-trace")]
#[allow(deprecated)]
#[deprecated(
    since = "0.4.0",
    note = "use plan_transform_passes_parallel_transform_catalog_layers; removal is not before 1.0 and requires downstream migration plus zero audited non-compatibility uses"
)]
pub fn plan_transform_passes_parallel_lawvere_layers(
    requested: &[TransformPassKind],
) -> omena_lawvere::TransformPassParallelPlanV0 {
    omena_lawvere::plan_transform_pass_parallel_layers_v0(requested)
}

pub fn implemented_mutation_pass_ids() -> Vec<&'static str> {
    default_transform_pass_registry()
        .entries
        .into_iter()
        .filter(|entry| entry.contract.executes_mutation)
        .map(|entry| entry.contract.id)
        .collect()
}

pub fn default_transform_pass_registry() -> TransformPassRegistryV0 {
    let contracts = default_transform_pass_contracts();
    let entries = default_transform_pass_descriptors()
        .into_iter()
        .filter_map(|descriptor| {
            contract_for_pass(descriptor.kind, contracts.as_slice())
                .cloned()
                .map(|contract| registry_entry_for_descriptor(contract, descriptor))
        })
        .collect::<Vec<_>>();
    TransformPassRegistryV0 {
        schema_version: "0",
        product: "omena-transform-passes.pass-registry",
        entries,
    }
}

fn registry_entry_for_descriptor(
    contract: TransformPassContractV0,
    descriptor: TransformPassDescriptorV0,
) -> TransformPassRegistryEntryV0 {
    let module_family = contract.family;
    let dispatch_kind = dispatch_kind_for_descriptor(&descriptor);
    TransformPassRegistryEntryV0 {
        module_family,
        query_family: query_family_for_pass(contract.kind),
        dispatch_kind,
        execution_status: TransformPassExecutionStatus::RegistryAndPlannerReady,
        contract,
        descriptor,
    }
}

fn dispatch_kind_for_descriptor(
    descriptor: &TransformPassDescriptorV0,
) -> TransformPassDispatchKindV0 {
    match descriptor.pass_class {
        TransformPassClassV0::TextLocal => TransformPassDispatchKindV0::TextLocalSliceRewrite,
        TransformPassClassV0::Structural => TransformPassDispatchKindV0::StructuralIrTransaction,
        TransformPassClassV0::ModuleEvaluation => {
            TransformPassDispatchKindV0::ModuleEvaluationHandler
        }
        TransformPassClassV0::Emission => TransformPassDispatchKindV0::EmissionBoundary,
    }
}

fn query_family_for_pass(kind: TransformPassKind) -> &'static str {
    match kind.layer() {
        TransformLayer::SemanticAware => "semantic-aware-transform-query",
        TransformLayer::Commodity => "commodity-transform-query",
        TransformLayer::Emission => "emission-transform-query",
        TransformLayer::SemanticReadOnly => "semantic-read-only-query",
    }
}

fn order_passes_by_registry(
    requested: &[TransformPassKind],
    registry_entries: &[TransformPassRegistryEntryV0],
) -> Result<Vec<TransformPassKind>, TransformPlanDependencyCycleV0> {
    let mut remaining = dedupe_requested_passes(requested);
    remaining.sort_by_key(|kind| {
        descriptor_for_pass(*kind, registry_entries)
            .map(|descriptor| (descriptor.phase, descriptor.phase_order, descriptor.id))
            .unwrap_or((u8::MAX, u16::MAX, ""))
    });

    let mut ordered = Vec::with_capacity(remaining.len());
    while !remaining.is_empty() {
        let Some(next_index) = remaining.iter().position(|candidate| {
            !has_incoming_edge_from_remaining(*candidate, &remaining, registry_entries)
        }) else {
            let cycle = dependency_cycle_witness(remaining.as_slice(), registry_entries)
                .expect("Kahn planner no-progress state must contain a dependency cycle");
            return Err(cycle);
        };
        ordered.push(remaining.remove(next_index));
    }

    Ok(ordered)
}

#[cfg(test)]
mod planner_cycle_tests {
    use super::*;

    #[test]
    fn constructed_dependency_cycle_is_not_silently_ordered() {
        let mut registry = default_transform_pass_registry();
        registry
            .entries
            .iter_mut()
            .find(|entry| entry.descriptor.kind == TransformPassKind::ImportInline)
            .expect("import-inline descriptor")
            .descriptor
            .depends_on = vec![TransformPassKind::PrintCss.id()];
        registry
            .entries
            .iter_mut()
            .find(|entry| entry.descriptor.kind == TransformPassKind::PrintCss)
            .expect("print-css descriptor")
            .descriptor
            .depends_on = vec![TransformPassKind::ImportInline.id()];

        let error = plan_transform_passes_checked_with_registry(
            &[TransformPassKind::ImportInline, TransformPassKind::PrintCss],
            &registry,
            default_transform_dag_edges().as_slice(),
        )
        .expect_err("a cyclic transform registry must not produce an arbitrary order");
        let serialized = serde_json::to_value(&error).expect("serialize typed planner error");
        eprintln!("TRANSFORM_PLANNER_CYCLE_ERROR={serialized}");
        assert_eq!(serialized["kind"], "dependencyCycle");
        assert_eq!(
            serialized["cycle"]["dependencyEdges"]
                .as_array()
                .map(Vec::len),
            Some(2)
        );
        let TransformPassPlanningErrorV0::DependencyCycle { cycle: error } = error else {
            panic!("constructed dependency cycle returned the wrong typed planning error");
        };

        assert_eq!(
            error.cycle_path,
            vec!["import-inline", "print-css", "import-inline"]
        );
        assert_eq!(
            error.dependency_edges,
            vec![
                TransformPlanDependencyEdgeV0 {
                    prerequisite_pass_id: "print-css",
                    dependent_pass_id: "import-inline",
                },
                TransformPlanDependencyEdgeV0 {
                    prerequisite_pass_id: "import-inline",
                    dependent_pass_id: "print-css",
                },
            ]
        );
    }

    #[test]
    fn default_catalog_has_no_dependency_cycle_across_300k_deterministic_requests() {
        const PROBE_INPUT_COUNT: usize = 300_000;
        const MAX_REQUEST_WIDTH: usize = 8;

        let registry = default_transform_pass_registry();
        let dag_edges = default_transform_dag_edges();
        let catalog = all_transform_pass_kinds();
        let mut state = 0x9e37_79b9_7f4a_7c15_u64;
        let mut observed_pass_ids = std::collections::BTreeSet::new();
        let mut cycle_error_count = 0_usize;
        let mut violated_dag_edge_plan_count = 0_usize;

        for _ in 0..PROBE_INPUT_COUNT {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            let width = 1 + (state as usize % MAX_REQUEST_WIDTH);
            let mut requested = Vec::with_capacity(width);
            for _ in 0..width {
                state ^= state << 13;
                state ^= state >> 7;
                state ^= state << 17;
                let pass = catalog[state as usize % catalog.len()];
                observed_pass_ids.insert(pass.id());
                requested.push(pass);
            }
            match plan_transform_passes_with_registry(
                requested.as_slice(),
                &registry,
                dag_edges.as_slice(),
            ) {
                Ok(plan) => {
                    violated_dag_edge_plan_count += usize::from(plan.violated_dag_edge_count > 0);
                }
                Err(_) => cycle_error_count += 1,
            }
        }

        eprintln!(
            "TRANSFORM_PLANNER_PROBE={{\"inputCount\":{PROBE_INPUT_COUNT},\"maximumRequestWidth\":{MAX_REQUEST_WIDTH},\"observedPassKindCount\":{},\"cycleErrorCount\":{cycle_error_count},\"violatedDagEdgePlanCount\":{violated_dag_edge_plan_count}}}",
            observed_pass_ids.len(),
        );
        assert_eq!(observed_pass_ids.len(), catalog.len());
        assert_eq!(cycle_error_count, 0);
        assert_eq!(violated_dag_edge_plan_count, 0);
    }
}

fn dependency_cycle_witness(
    remaining: &[TransformPassKind],
    registry_entries: &[TransformPassRegistryEntryV0],
) -> Option<TransformPlanDependencyCycleV0> {
    let remaining_ids = remaining.iter().map(|pass| pass.id()).collect::<Vec<_>>();
    let mut visit_state = std::collections::BTreeMap::<&'static str, u8>::new();
    let mut stack = Vec::new();
    for pass_id in &remaining_ids {
        if visit_state.get(pass_id).copied().unwrap_or_default() == 0
            && let Some(cycle) = visit_dependency_cycle(
                pass_id,
                remaining_ids.as_slice(),
                registry_entries,
                &mut visit_state,
                &mut stack,
            )
        {
            return Some(cycle);
        }
    }
    None
}

fn visit_dependency_cycle(
    pass_id: &'static str,
    remaining_ids: &[&'static str],
    registry_entries: &[TransformPassRegistryEntryV0],
    visit_state: &mut std::collections::BTreeMap<&'static str, u8>,
    stack: &mut Vec<&'static str>,
) -> Option<TransformPlanDependencyCycleV0> {
    visit_state.insert(pass_id, 1);
    stack.push(pass_id);
    let mut dependencies = registry_entries
        .iter()
        .find(|entry| entry.descriptor.id == pass_id)
        .map(|entry| entry.descriptor.depends_on.clone())
        .unwrap_or_default();
    dependencies.sort_unstable();
    dependencies.dedup();
    for dependency in dependencies
        .into_iter()
        .filter(|dependency| remaining_ids.contains(dependency))
    {
        match visit_state.get(dependency).copied().unwrap_or_default() {
            0 => {
                if let Some(cycle) = visit_dependency_cycle(
                    dependency,
                    remaining_ids,
                    registry_entries,
                    visit_state,
                    stack,
                ) {
                    return Some(cycle);
                }
            }
            1 => {
                let cycle_start = stack
                    .iter()
                    .position(|candidate| *candidate == dependency)?;
                let mut cycle_path = stack[cycle_start..].to_vec();
                cycle_path.push(dependency);
                let dependency_edges = cycle_path
                    .windows(2)
                    .map(|pair| TransformPlanDependencyEdgeV0 {
                        prerequisite_pass_id: pair[1],
                        dependent_pass_id: pair[0],
                    })
                    .collect();
                return Some(TransformPlanDependencyCycleV0 {
                    cycle_path,
                    dependency_edges,
                });
            }
            _ => {}
        }
    }
    stack.pop();
    visit_state.insert(pass_id, 2);
    None
}

fn dedupe_requested_passes(requested: &[TransformPassKind]) -> Vec<TransformPassKind> {
    let mut unique = Vec::new();
    for pass in requested {
        if !unique.contains(pass) {
            unique.push(*pass);
        }
    }
    unique
}

fn conflicting_unordered_pass_pairs(
    requested: &[TransformPassKind],
    registry_entries: &[TransformPassRegistryEntryV0],
    dag_edges: &[TransformDagEdgeV0],
) -> Vec<TransformPlanPassConflictV0> {
    let mut conflicts = Vec::new();
    for (left_index, left) in requested.iter().enumerate() {
        for right in requested.iter().skip(left_index + 1) {
            let Some(left_descriptor) = descriptor_for_pass(*left, registry_entries) else {
                continue;
            };
            let Some(right_descriptor) = descriptor_for_pass(*right, registry_entries) else {
                continue;
            };
            let declared = left_descriptor
                .conflicts_with
                .contains(&right_descriptor.id)
                || right_descriptor
                    .conflicts_with
                    .contains(&left_descriptor.id);
            if declared
                && !dag_path_exists(left_descriptor.id, right_descriptor.id, dag_edges)
                && !dag_path_exists(right_descriptor.id, left_descriptor.id, dag_edges)
            {
                conflicts.push(TransformPlanPassConflictV0 {
                    pass_a: left_descriptor.id,
                    pass_b: right_descriptor.id,
                });
            }
        }
    }
    conflicts
}

fn has_incoming_edge_from_remaining(
    candidate: TransformPassKind,
    remaining: &[TransformPassKind],
    registry_entries: &[TransformPassRegistryEntryV0],
) -> bool {
    descriptor_for_pass(candidate, registry_entries).is_some_and(|descriptor| {
        descriptor
            .depends_on
            .iter()
            .any(|dependency| remaining.iter().any(|other| other.id() == *dependency))
    })
}

fn edge_applies(edge: &TransformDagEdgeV0, ordered_pass_ids: &[&'static str]) -> bool {
    ordered_pass_ids.contains(&edge.from) && ordered_pass_ids.contains(&edge.to)
}

fn edge_is_satisfied(edge: &TransformDagEdgeV0, ordered_pass_ids: &[&'static str]) -> bool {
    let from = position_of_pass_id(edge.from, ordered_pass_ids);
    let to = position_of_pass_id(edge.to, ordered_pass_ids);
    match (from, to) {
        (Some(from), Some(to)) => from < to,
        _ => false,
    }
}

fn dag_path_exists(from: &'static str, to: &'static str, dag_edges: &[TransformDagEdgeV0]) -> bool {
    let mut stack = vec![from];
    let mut visited = Vec::new();
    while let Some(current) = stack.pop() {
        if current == to {
            return true;
        }
        if visited.contains(&current) {
            continue;
        }
        visited.push(current);
        for edge in dag_edges.iter().filter(|edge| edge.from == current) {
            stack.push(edge.to);
        }
    }
    false
}

fn position_of_pass_id(pass_id: &'static str, ordered_pass_ids: &[&'static str]) -> Option<usize> {
    ordered_pass_ids
        .iter()
        .position(|ordered_pass_id| *ordered_pass_id == pass_id)
}

fn contract_for_pass(
    pass: TransformPassKind,
    contracts: &[TransformPassContractV0],
) -> Option<&TransformPassContractV0> {
    contracts.iter().find(|contract| contract.kind == pass)
}

fn descriptor_for_pass(
    pass: TransformPassKind,
    registry_entries: &[TransformPassRegistryEntryV0],
) -> Option<&TransformPassDescriptorV0> {
    registry_entries
        .iter()
        .find(|entry| entry.descriptor.kind == pass)
        .map(|entry| &entry.descriptor)
}

pub(crate) fn transform_pass_kind_from_id(pass_id: &str) -> Option<TransformPassKind> {
    all_transform_pass_kinds()
        .into_iter()
        .find(|kind| kind.id() == pass_id)
}
