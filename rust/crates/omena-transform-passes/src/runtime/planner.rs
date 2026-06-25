//! Transform pass registry, DAG planner, and public boundary summary.
//!
//! Planner code maps `omena-transform-cst` pass contracts into executable
//! registry entries, enforces default DAG ordering, and reports the mutation
//! passes that are implemented by the runtime executor.

use omena_transform_cst::{
    TRANSFORM_PASS_CATALOG_LEN, TransformDagEdgeV0, TransformLayer, TransformPassContractV0,
    TransformPassKind, all_transform_pass_kinds, default_transform_dag_edges,
    default_transform_pass_contracts,
};

use crate::{
    TransformPassExecutionStatus, TransformPassPlanV0, TransformPassRegistryEntryV0,
    TransformPassesBoundarySummaryV0,
};

pub fn summarize_omena_transform_passes_boundary() -> TransformPassesBoundarySummaryV0 {
    let registry_entries = default_transform_pass_contracts()
        .into_iter()
        .map(registry_entry_for_contract)
        .collect::<Vec<_>>();
    let pass_count = registry_entries.len();
    let semantic_aware_pass_count = registry_entries
        .iter()
        .filter(|entry| entry.contract.layer == TransformLayer::SemanticAware)
        .count();
    let cascade_aware_pass_count = registry_entries
        .iter()
        .filter(|entry| entry.contract.reads_cascade_model)
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
        planner_enforces_dag_edges: true,
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
    let ordered_passes = order_passes_by_dag(requested);
    let ordered_pass_ids = ordered_passes
        .iter()
        .map(|pass| pass.id())
        .collect::<Vec<_>>();
    let dag_edges = default_transform_dag_edges();
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
        requested_pass_ids,
        ordered_pass_ids,
        satisfied_dag_edge_count,
        violated_dag_edge_count,
        all_requested_registered: requested.iter().all(pass_is_registered),
    }
}

#[cfg(feature = "lawvere-trace")]
pub fn plan_transform_passes_parallel_lawvere_layers(
    requested: &[TransformPassKind],
) -> omena_lawvere::TransformPassParallelPlanV0 {
    omena_lawvere::plan_transform_pass_parallel_layers_v0(requested)
}

pub fn implemented_mutation_pass_ids() -> Vec<&'static str> {
    default_transform_pass_contracts()
        .into_iter()
        .filter(|contract| contract.executes_mutation)
        .map(|contract| contract.id)
        .collect()
}

fn registry_entry_for_contract(contract: TransformPassContractV0) -> TransformPassRegistryEntryV0 {
    let module_family = contract.family;
    TransformPassRegistryEntryV0 {
        module_family,
        query_family: query_family_for_pass(contract.kind),
        execution_status: TransformPassExecutionStatus::RegistryAndPlannerReady,
        contract,
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
    let contracts = default_transform_pass_contracts();
    remaining.sort_by_key(|kind| {
        (
            contract_for_pass(*kind, &contracts)
                .map(|contract| contract.execution_phase)
                .unwrap_or(u8::MAX),
            kind.ordinal(),
        )
    });

    let mut ordered = Vec::with_capacity(remaining.len());
    while !remaining.is_empty() {
        let next_index = remaining
            .iter()
            .position(|candidate| !has_incoming_edge_from_remaining(*candidate, &remaining))
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

fn has_incoming_edge_from_remaining(
    candidate: TransformPassKind,
    remaining: &[TransformPassKind],
) -> bool {
    default_transform_dag_edges().iter().any(|edge| {
        edge.to == candidate.id()
            && remaining
                .iter()
                .any(|other| other.id() == edge.from && *other != candidate)
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

fn position_of_pass_id(pass_id: &'static str, ordered_pass_ids: &[&'static str]) -> Option<usize> {
    ordered_pass_ids
        .iter()
        .position(|ordered_pass_id| *ordered_pass_id == pass_id)
}

fn pass_is_registered(pass: &TransformPassKind) -> bool {
    default_transform_pass_contracts()
        .iter()
        .any(|contract| contract.kind == *pass)
}

fn contract_for_pass(
    pass: TransformPassKind,
    contracts: &[TransformPassContractV0],
) -> Option<&TransformPassContractV0> {
    contracts.iter().find(|contract| contract.kind == pass)
}

pub(crate) fn transform_pass_kind_from_id(pass_id: &str) -> Option<TransformPassKind> {
    all_transform_pass_kinds()
        .into_iter()
        .find(|kind| kind.id() == pass_id)
}
