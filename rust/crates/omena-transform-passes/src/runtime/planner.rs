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
    TransformPassRegistryEntryV0, TransformPassRegistryV0, TransformPassesBoundarySummaryV0,
    TransformPlanPassConflictV0,
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
    let requested_pass_ids = requested.iter().map(|pass| pass.id()).collect::<Vec<_>>();
    let registry = default_transform_pass_registry();
    let dag_edges = default_transform_dag_edges();
    let requested_unique = dedupe_requested_passes(requested);
    let conflicting_unordered_pass_pairs = conflicting_unordered_pass_pairs(
        requested_unique.as_slice(),
        registry.entries.as_slice(),
        dag_edges.as_slice(),
    );
    let ordered_passes = if conflicting_unordered_pass_pairs.is_empty() {
        order_passes_by_dag(requested)
    } else {
        Vec::new()
    };
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

    TransformPassPlanV0 {
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
        all_requested_registered: requested.iter().all(pass_is_registered),
        conflicting_unordered_pass_pairs,
    }
}

pub fn plan_transform_passes_checked(
    requested: &[TransformPassKind],
) -> Result<TransformPassPlanV0, TransformPlanPassConflictV0> {
    let plan = plan_transform_passes(requested);
    if let Some(conflict) = plan.conflicting_unordered_pass_pairs.first().cloned() {
        Err(conflict)
    } else {
        Ok(plan)
    }
}

#[cfg(feature = "lawvere-trace")]
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

fn order_passes_by_dag(requested: &[TransformPassKind]) -> Vec<TransformPassKind> {
    let mut remaining = dedupe_requested_passes(requested);
    let registry = default_transform_pass_registry();
    remaining.sort_by_key(|kind| {
        descriptor_for_pass(*kind, registry.entries.as_slice())
            .map(|descriptor| (descriptor.phase, descriptor.phase_order, descriptor.id))
            .unwrap_or((u8::MAX, u16::MAX, ""))
    });

    let mut ordered = Vec::with_capacity(remaining.len());
    while !remaining.is_empty() {
        let next_index = remaining
            .iter()
            .position(|candidate| {
                !has_incoming_edge_from_remaining(
                    *candidate,
                    &remaining,
                    registry.entries.as_slice(),
                )
            })
            .unwrap_or_default();
        ordered.push(remaining.remove(next_index));
    }

    ordered
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
        descriptor.depends_on.iter().any(|dependency| {
            remaining
                .iter()
                .any(|other| other.id() == *dependency && *other != candidate)
        })
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

fn pass_is_registered(pass: &TransformPassKind) -> bool {
    default_transform_pass_registry()
        .entries
        .iter()
        .any(|entry| entry.contract.kind == *pass)
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
