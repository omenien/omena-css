//! Salsa-backed incremental computation substrate for omena-css.
//!
//! The crate owns graph snapshots, dirty-set planning, cancellation state, and
//! fuzzable consistency cases. Downstream parser, semantic, and transform
//! crates depend on these stable V0 payloads instead of reaching into Salsa
//! internals directly.

use std::collections::{BTreeMap, BTreeSet};

use omena_evidence_graph::{
    EvidenceDemandEdgeV0, EvidenceGraphBuildErrorV0, EvidenceGraphV0, EvidenceNodeKeyV0,
    EvidenceNodeSeedV0, GuaranteeKindV0, build_evidence_graph_from_edges_v0,
};
use salsa::Setter;
use serde::Serialize;

mod frame_invalidation;
pub use frame_invalidation::*;

#[cfg(test)]
use std::cell::RefCell;
#[cfg(test)]
thread_local! {
    static SALSA_NODE_VALUE_QUERY_RUNS_BY_ID: RefCell<BTreeMap<String, usize>> = const { RefCell::new(BTreeMap::new()) };
}

#[cfg(test)]
fn record_salsa_node_value_query_run(id: &str) {
    SALSA_NODE_VALUE_QUERY_RUNS_BY_ID.with(|runs| {
        *runs.borrow_mut().entry(id.to_string()).or_default() += 1;
    });
}

#[cfg(test)]
fn reset_salsa_node_value_query_runs() {
    SALSA_NODE_VALUE_QUERY_RUNS_BY_ID.with(|runs| runs.borrow_mut().clear());
}

#[cfg(test)]
fn salsa_node_value_query_runs(id: &str) -> usize {
    SALSA_NODE_VALUE_QUERY_RUNS_BY_ID
        .with(|runs| runs.borrow().get(id).copied().unwrap_or_default())
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaIncrementalBoundarySummaryV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub engine_name: &'static str,
    pub invalidation_model: &'static str,
    pub query_model: &'static str,
    pub dependency_propagation_policy: &'static str,
    pub maximum_dependency_propagation_iterations: &'static str,
    pub node_identity: Vec<&'static str>,
    pub dirty_reasons: Vec<&'static str>,
    pub ready_surfaces: Vec<&'static str>,
}

pub const DEFAULT_INCREMENTAL_CANCELLATION_LIMIT: usize = 128;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IncrementalRevisionV0 {
    pub value: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IncrementalGraphInputV0 {
    pub revision: IncrementalRevisionV0,
    pub nodes: Vec<IncrementalNodeInputV0>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IncrementalNodeInputV0 {
    pub id: String,
    pub digest: String,
    pub dependency_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IncrementalSnapshotV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub revision: IncrementalRevisionV0,
    pub nodes: Vec<IncrementalSnapshotNodeV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IncrementalSnapshotNodeV0 {
    pub id: String,
    pub digest: String,
    pub dependency_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IncrementalComputationPlanV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub revision: IncrementalRevisionV0,
    pub node_count: usize,
    pub dirty_node_count: usize,
    pub changed_input_count: usize,
    pub new_node_count: usize,
    pub removed_node_count: usize,
    pub dependency_dirty_count: usize,
    pub alpha_equivalence_graph_hash: IncrementalAlphaEquivalenceHashV0,
    pub shadow_delta_oracle: IncrementalShadowDeltaOracleV0,
    pub invalidation_priority_plan: IncrementalInvalidationPriorityPlanV0,
    pub nodes: Vec<IncrementalComputationNodeV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IncrementalComputationNodeV0 {
    pub id: String,
    pub digest: String,
    pub dependency_ids: Vec<String>,
    pub dirty: bool,
    pub reasons: Vec<&'static str>,
    pub changed_at: IncrementalRevisionV0,
    pub verified_at: IncrementalRevisionV0,
    pub value_equal_to_previous: bool,
    pub alpha_equivalence_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IncrementalAlphaEquivalenceHashV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub feature_gate: &'static str,
    pub claim_level: &'static str,
    pub theorem_claimed: bool,
    pub hash: String,
    pub normalized_node_count: usize,
    pub normalized_edge_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IncrementalShadowDeltaOracleV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub feature_gate: &'static str,
    pub claim_level: &'static str,
    pub theorem_claimed: bool,
    pub sampled_shadow_witness_ready: bool,
    pub incremental_dirty_ids: Vec<String>,
    pub from_scratch_dirty_ids: Vec<String>,
    pub incremental_matches_from_scratch_delta: bool,
    pub dbsp_zset_claim_ready: bool,
    pub performance_benchmark_claim_ready: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IncrementalEditDistancePriorityInputV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub feature_gate: &'static str,
    pub claim_level: &'static str,
    pub theorem_claimed: bool,
    pub node_id: String,
    pub edit_distance_total: usize,
    pub cascade_margin_abs_distance: u64,
    pub bridge_checked: bool,
    pub bridge_calibration_stage: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IncrementalInvalidationPriorityPlanV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub feature_gate: &'static str,
    pub claim_level: &'static str,
    pub theorem_claimed: bool,
    pub public_safety_claim_ready: bool,
    pub calibration_stage: &'static str,
    pub weight_profile: &'static str,
    pub metric_input_count: usize,
    pub dirty_node_count: usize,
    pub metric_consumed_count: usize,
    pub prioritized_dirty_node_ids: Vec<String>,
    pub entries: Vec<IncrementalInvalidationPriorityEntryV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IncrementalInvalidationPriorityEntryV0 {
    pub node_id: String,
    pub priority_rank: usize,
    pub priority_score: u64,
    pub priority_kind: &'static str,
    pub metric_consumed: bool,
    pub edit_distance_total: Option<usize>,
    pub cascade_margin_abs_distance: Option<u64>,
    pub bridge_checked: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IncrementalCancellationSnapshotV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub cancelled_request_count: usize,
    pub cancelled_request_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IncrementalDatabaseUpdateV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub incremental_plan: IncrementalComputationPlanV0,
    pub datalog_rule_evaluator: DatalogRuleEvaluatorV0,
    pub next_snapshot: IncrementalSnapshotV0,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IncrementalConsistencyFuzzCaseV0 {
    pub seed: u64,
    pub node_count: usize,
    pub changed_node_index: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IncrementalConsistencyFuzzResultV0 {
    pub seed: u64,
    pub node_count: usize,
    pub changed_node_id: Option<String>,
    pub dirty_node_count: usize,
    pub expected_dirty_node_count: usize,
    pub passed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IncrementalFuzzSeedReportV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub case_count: usize,
    pub passed_count: usize,
    pub failed_count: usize,
    pub results: Vec<IncrementalConsistencyFuzzResultV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DatalogRuleEvaluatorRuleV0 {
    pub name: &'static str,
    pub head: &'static str,
    pub body: Vec<&'static str>,
    pub source: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
/// V0 freeze-candidate typed contract over the incremental dirty-plan substrate.
///
/// This exposes Datalog-shaped rules and relations for auditability while the
/// product path remains the Salsa-backed fixed-point planner. It is not an
/// external Datalog host, FlowLog/Souffle/egglog binding, or Cargo 1.0 API.
pub struct DatalogRuleEvaluatorV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub evaluator_kind: &'static str,
    pub substrate: &'static str,
    pub external_host_ready: bool,
    pub revision: IncrementalRevisionV0,
    pub rule_count: usize,
    pub relation_count: usize,
    pub input_node_count: usize,
    pub dirty_node_count: usize,
    pub derived_node_count: usize,
    pub iteration_limit: usize,
    pub fixed_point_reached: bool,
    pub relations: Vec<&'static str>,
    pub rules: Vec<DatalogRuleEvaluatorRuleV0>,
    pub incremental_plan: IncrementalComputationPlanV0,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IncrementalLayerEvidenceV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub claim_level: &'static str,
    pub invalidation_layer: &'static str,
    pub real_invalidation_evidence_ready: bool,
    pub fuzz_evidence_ready: bool,
    pub salsa_reuse_evidence_ready: bool,
    pub datalog_contract_evidence_ready: bool,
    pub value_equality_backdating_ready: bool,
    pub alpha_equivalence_hash_ready: bool,
    pub shadow_delta_oracle_ready: bool,
    pub edit_distance_priority_ready: bool,
    pub benchmark_surface_ready: bool,
    pub performance_benchmark_claim_ready: bool,
    pub external_datalog_host_ready: bool,
    pub dbsp_zset_claim_ready: bool,
    pub public_safety_claim_ready: bool,
    pub benchmark_gate: &'static str,
    pub benchmark_evidence_level: &'static str,
    pub supported_claims: Vec<&'static str>,
    pub deferred_claims: Vec<&'static str>,
    pub boundary: OmenaIncrementalBoundarySummaryV0,
    pub fuzz_report: IncrementalFuzzSeedReportV0,
    pub sample_update: IncrementalDatabaseUpdateV0,
}

const INCREMENTAL_EVIDENCE_EDGE_KIND_V0: &str = "incremental-evidence";

fn incremental_guarantee_kind(claim_level: &str) -> GuaranteeKindV0 {
    GuaranteeKindV0::from_existing_label(claim_level)
        .unwrap_or_else(GuaranteeKindV0::for_label_less_family)
}

fn incremental_evidence_node_key(
    query_identity: impl Into<String>,
    input_identity: impl Into<String>,
) -> EvidenceNodeKeyV0 {
    EvidenceNodeKeyV0::new(query_identity, input_identity)
}

fn incremental_evidence_edge(
    from_query_identity: impl Into<String>,
    to_node_key: EvidenceNodeKeyV0,
) -> EvidenceDemandEdgeV0 {
    EvidenceDemandEdgeV0::new(
        from_query_identity,
        to_node_key,
        INCREMENTAL_EVIDENCE_EDGE_KIND_V0,
    )
}

impl IncrementalAlphaEquivalenceHashV0 {
    pub fn evidence_node_key(&self) -> EvidenceNodeKeyV0 {
        incremental_evidence_node_key(self.product, format!("{}:{}", self.feature_gate, self.hash))
    }

    pub fn evidence_node_seed(&self) -> EvidenceNodeSeedV0 {
        EvidenceNodeSeedV0::new(
            self.evidence_node_key(),
            vec![
                self.product.to_string(),
                self.feature_gate.to_string(),
                self.claim_level.to_string(),
            ],
            incremental_guarantee_kind(self.claim_level),
        )
    }

    pub fn evidence_demand_edge(&self) -> EvidenceDemandEdgeV0 {
        incremental_evidence_edge(self.product, self.evidence_node_key())
    }
}

impl IncrementalShadowDeltaOracleV0 {
    pub fn evidence_node_key(&self) -> EvidenceNodeKeyV0 {
        incremental_evidence_node_key(
            self.product,
            format!(
                "{}:incremental={}:fromScratch={}",
                self.feature_gate,
                self.incremental_dirty_ids.join(","),
                self.from_scratch_dirty_ids.join(",")
            ),
        )
    }

    pub fn evidence_node_seed(&self) -> EvidenceNodeSeedV0 {
        EvidenceNodeSeedV0::new(
            self.evidence_node_key(),
            vec![
                self.product.to_string(),
                self.feature_gate.to_string(),
                self.claim_level.to_string(),
            ],
            incremental_guarantee_kind(self.claim_level),
        )
    }

    pub fn evidence_demand_edge(&self) -> EvidenceDemandEdgeV0 {
        incremental_evidence_edge(self.product, self.evidence_node_key())
    }
}

impl IncrementalEditDistancePriorityInputV0 {
    pub fn evidence_node_key(&self) -> EvidenceNodeKeyV0 {
        incremental_evidence_node_key(
            self.product,
            format!("{}:{}", self.feature_gate, self.node_id),
        )
    }

    pub fn evidence_node_seed(&self) -> EvidenceNodeSeedV0 {
        EvidenceNodeSeedV0::new(
            self.evidence_node_key(),
            vec![
                self.product.to_string(),
                self.feature_gate.to_string(),
                self.claim_level.to_string(),
                self.bridge_calibration_stage.to_string(),
            ],
            incremental_guarantee_kind(self.claim_level),
        )
    }

    pub fn evidence_demand_edge(&self) -> EvidenceDemandEdgeV0 {
        incremental_evidence_edge(self.product, self.evidence_node_key())
    }
}

impl IncrementalInvalidationPriorityPlanV0 {
    pub fn evidence_node_key(&self) -> EvidenceNodeKeyV0 {
        incremental_evidence_node_key(
            self.product,
            format!(
                "{}:{}:dirty={}:metric={}",
                self.feature_gate,
                self.weight_profile,
                self.dirty_node_count,
                self.metric_consumed_count
            ),
        )
    }

    pub fn evidence_node_seed(&self) -> EvidenceNodeSeedV0 {
        EvidenceNodeSeedV0::new(
            self.evidence_node_key(),
            vec![
                self.product.to_string(),
                self.feature_gate.to_string(),
                self.claim_level.to_string(),
                self.calibration_stage.to_string(),
            ],
            incremental_guarantee_kind(self.claim_level),
        )
    }

    pub fn evidence_demand_edge(&self) -> EvidenceDemandEdgeV0 {
        incremental_evidence_edge(self.product, self.evidence_node_key())
    }
}

impl IncrementalComputationPlanV0 {
    pub fn evidence_node_seeds(&self) -> Vec<EvidenceNodeSeedV0> {
        vec![
            self.alpha_equivalence_graph_hash.evidence_node_seed(),
            self.shadow_delta_oracle.evidence_node_seed(),
            self.invalidation_priority_plan.evidence_node_seed(),
        ]
    }

    pub fn evidence_demand_edges(&self) -> Vec<EvidenceDemandEdgeV0> {
        vec![
            self.alpha_equivalence_graph_hash.evidence_demand_edge(),
            self.shadow_delta_oracle.evidence_demand_edge(),
            self.invalidation_priority_plan.evidence_demand_edge(),
        ]
    }

    pub fn evidence_graph(&self) -> Result<EvidenceGraphV0, EvidenceGraphBuildErrorV0> {
        build_evidence_graph_from_edges_v0(self.evidence_node_seeds(), self.evidence_demand_edges())
    }
}

impl IncrementalLayerEvidenceV0 {
    pub fn evidence_node_key(&self) -> EvidenceNodeKeyV0 {
        incremental_evidence_node_key(self.product, self.invalidation_layer)
    }

    pub fn evidence_node_seed(&self) -> EvidenceNodeSeedV0 {
        EvidenceNodeSeedV0::new(
            self.evidence_node_key(),
            vec![
                self.product.to_string(),
                self.claim_level.to_string(),
                self.invalidation_layer.to_string(),
                self.benchmark_evidence_level.to_string(),
            ],
            incremental_guarantee_kind(self.claim_level),
        )
    }

    pub fn evidence_demand_edge(&self) -> EvidenceDemandEdgeV0 {
        incremental_evidence_edge(self.product, self.evidence_node_key())
    }

    pub fn evidence_graph(&self) -> Result<EvidenceGraphV0, EvidenceGraphBuildErrorV0> {
        let mut seeds = vec![self.evidence_node_seed()];
        seeds.extend(self.sample_update.incremental_plan.evidence_node_seeds());
        let mut edges = vec![self.evidence_demand_edge()];
        edges.extend(self.sample_update.incremental_plan.evidence_demand_edges());
        build_evidence_graph_from_edges_v0(seeds, edges)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IncrementalCancellationRegistryV0 {
    limit: usize,
    cancelled_request_ids: BTreeSet<String>,
}

#[salsa::input(debug)]
pub struct SalsaIncrementalNodeInputV0 {
    #[returns(ref)]
    id: String,
    #[returns(ref)]
    digest: String,
    #[returns(ref)]
    dependency_ids: Vec<String>,
}

#[salsa::input(debug)]
pub struct SalsaIncrementalGraphInputV0 {
    #[returns(ref)]
    nodes: Vec<SalsaIncrementalNodeInputV0>,
}

#[salsa::input(debug)]
pub struct SalsaIncrementalFileRevisionInputV0 {
    file_id: u32,
    revision: IncrementalRevisionV0,
    #[returns(ref)]
    syntax_node_id: String,
}

#[salsa::db]
#[derive(Clone, Default)]
pub struct OmenaSalsaDatabaseV0 {
    storage: salsa::Storage<Self>,
}

#[salsa::db]
impl salsa::Database for OmenaSalsaDatabaseV0 {}

impl OmenaSalsaDatabaseV0 {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn handle(&self) -> salsa::StorageHandle<Self> {
        self.storage.clone().into_zalsa_handle()
    }

    pub fn from_handle(handle: salsa::StorageHandle<Self>) -> Self {
        Self {
            storage: handle.into_storage(),
        }
    }
}

#[derive(Default)]
pub struct OmenaIncrementalDatabaseV0 {
    db: OmenaSalsaDatabaseV0,
    node_inputs_by_id: BTreeMap<String, SalsaIncrementalNodeInputV0>,
    graph_input: Option<SalsaIncrementalGraphInputV0>,
    current_snapshot: Option<IncrementalSnapshotV0>,
}

pub fn summarize_omena_incremental_boundary() -> OmenaIncrementalBoundarySummaryV0 {
    OmenaIncrementalBoundarySummaryV0 {
        schema_version: "0",
        product: "omena-incremental.boundary",
        engine_name: "omena-incremental",
        invalidation_model: "stableNodeId+inputDigest+dependencyPropagation",
        query_model: "salsaInput+trackedQueryFieldGranularReuse",
        dependency_propagation_policy: "salsaDemandDirtySignatureReads",
        maximum_dependency_propagation_iterations: "oracleOnlyNodeCount+1",
        node_identity: vec!["id", "digest", "dependencyIds"],
        dirty_reasons: vec![
            "newNode",
            "inputDigestChanged",
            "dependencySetChanged",
            "dependencyDirty",
        ],
        ready_surfaces: vec![
            "incrementalGraphInput",
            "incrementalSnapshot",
            "incrementalComputationPlan",
            "incrementalCancellationRegistry",
            "datalogRuleEvaluatorV0",
            "salsaPersistentDatabase",
            "salsaTrackedNodeSnapshotQuery",
            "salsaFieldGranularReuse",
            "salsaPlanAndSnapshotUpdate",
            "salsaDemandDependencyReads",
            "valueEqualityBackdating",
            "alphaEquivalenceHash",
            "incrementalShadowDeltaOracle",
            "editDistanceInvalidationPriority",
        ],
    }
}

pub fn summarize_datalog_rule_evaluator_v0(
    input: &IncrementalGraphInputV0,
    previous: Option<&IncrementalSnapshotV0>,
) -> DatalogRuleEvaluatorV0 {
    let mut database = OmenaIncrementalDatabaseV0::default();
    if let Some(previous) = previous {
        database.restore_snapshot(previous);
    }
    database
        .plan_and_upsert_graph_input(input)
        .datalog_rule_evaluator
}

fn summarize_datalog_rule_evaluator_for_plan_v0(
    input: &IncrementalGraphInputV0,
    incremental_plan: IncrementalComputationPlanV0,
) -> DatalogRuleEvaluatorV0 {
    let relations = vec![
        "node(id,digest)",
        "previousNode(id,digest)",
        "dependsOn(nodeId,dependencyId)",
        "changedInput(nodeId)",
        "dirty(nodeId)",
    ];
    let rules = vec![
        DatalogRuleEvaluatorRuleV0 {
            name: "newNodeIsDirty",
            head: "dirty(Node)",
            body: vec!["node(Node,Digest)", "not previousNode(Node,_)"],
            source: "omena-incremental.computation-plan",
        },
        DatalogRuleEvaluatorRuleV0 {
            name: "changedDigestIsDirty",
            head: "dirty(Node)",
            body: vec![
                "node(Node,Digest)",
                "previousNode(Node,PreviousDigest)",
                "Digest != PreviousDigest",
            ],
            source: "omena-incremental.computation-plan",
        },
        DatalogRuleEvaluatorRuleV0 {
            name: "changedDependencySetIsDirty",
            head: "dirty(Node)",
            body: vec!["dependsOn(Node,_)", "previousDependencySetDiffers(Node)"],
            source: "omena-incremental.computation-plan",
        },
        DatalogRuleEvaluatorRuleV0 {
            name: "dependencyDirtyDemandRead",
            head: "dirty(Node)",
            body: vec!["dependsOn(Node,Dependency)", "dirty(Dependency)"],
            source: "omena-incremental.salsa-demand-plan",
        },
    ];
    let iteration_limit = dependency_propagation_iteration_limit(input.nodes.len());
    let fixed_point_reached = dirty_set_is_dependency_closed(&incremental_plan);

    DatalogRuleEvaluatorV0 {
        schema_version: "0",
        product: "omena-incremental.datalog-rule-evaluator",
        evaluator_kind: "typedContractOverSalsaDemandPlan",
        substrate: "omena-incremental.salsa-backed-computation-plan",
        external_host_ready: false,
        revision: input.revision,
        rule_count: rules.len(),
        relation_count: relations.len(),
        input_node_count: input.nodes.len(),
        dirty_node_count: incremental_plan.dirty_node_count,
        derived_node_count: incremental_plan.dependency_dirty_count,
        iteration_limit,
        fixed_point_reached,
        relations,
        rules,
        incremental_plan,
    }
}

pub fn snapshot_from_graph_input(input: &IncrementalGraphInputV0) -> IncrementalSnapshotV0 {
    IncrementalSnapshotV0 {
        schema_version: "0",
        product: "omena-incremental.snapshot",
        revision: input.revision,
        nodes: normalized_snapshot_nodes(input),
    }
}

fn graph_input_from_snapshot(snapshot: &IncrementalSnapshotV0) -> IncrementalGraphInputV0 {
    IncrementalGraphInputV0 {
        revision: snapshot.revision,
        nodes: snapshot
            .nodes
            .iter()
            .map(|node| IncrementalNodeInputV0 {
                id: node.id.clone(),
                digest: node.digest.clone(),
                dependency_ids: node.dependency_ids.clone(),
            })
            .collect(),
    }
}

pub fn summarize_incremental_shadow_delta_oracle_v0(
    input: &IncrementalGraphInputV0,
    previous: Option<&IncrementalSnapshotV0>,
    incremental_dirty_ids: BTreeSet<String>,
) -> IncrementalShadowDeltaOracleV0 {
    let from_scratch_dirty_ids = compute_from_scratch_delta_dirty_ids(input, previous);
    let incremental_dirty_ids = incremental_dirty_ids.into_iter().collect::<Vec<_>>();
    let from_scratch_dirty_ids = from_scratch_dirty_ids.into_iter().collect::<Vec<_>>();
    let incremental_matches_from_scratch_delta = incremental_dirty_ids == from_scratch_dirty_ids;

    IncrementalShadowDeltaOracleV0 {
        schema_version: "0",
        product: "omena-incremental.shadow-delta-oracle",
        feature_gate: "incremental-shadow-delta-v0",
        claim_level: "sampledFixtureWitnessNotEquivalenceProof",
        theorem_claimed: false,
        sampled_shadow_witness_ready: incremental_matches_from_scratch_delta,
        incremental_dirty_ids,
        from_scratch_dirty_ids,
        incremental_matches_from_scratch_delta,
        dbsp_zset_claim_ready: false,
        performance_benchmark_claim_ready: false,
    }
}

pub fn summarize_incremental_invalidation_priority_plan_v0(
    nodes: &[IncrementalComputationNodeV0],
    priority_inputs: &[IncrementalEditDistancePriorityInputV0],
) -> IncrementalInvalidationPriorityPlanV0 {
    let priority_inputs_by_node = priority_inputs
        .iter()
        .map(|input| (input.node_id.as_str(), input))
        .collect::<BTreeMap<_, _>>();
    let mut entries = nodes
        .iter()
        .filter(|node| node.dirty)
        .map(|node| {
            let priority_input = priority_inputs_by_node.get(node.id.as_str()).copied();
            let edit_distance_total = priority_input.map(|input| input.edit_distance_total);
            let cascade_margin_abs_distance =
                priority_input.map(|input| input.cascade_margin_abs_distance);
            let bridge_checked = priority_input.is_some_and(|input| input.bridge_checked);
            let metric_consumed = priority_input.is_some();
            let priority_score = invalidation_priority_score(priority_input);

            IncrementalInvalidationPriorityEntryV0 {
                node_id: node.id.clone(),
                priority_rank: 0,
                priority_score,
                priority_kind: if metric_consumed {
                    "editDistanceCascadeMarginWeighted"
                } else {
                    "dirtyNodeDefault"
                },
                metric_consumed,
                edit_distance_total,
                cascade_margin_abs_distance,
                bridge_checked,
            }
        })
        .collect::<Vec<_>>();
    entries.sort_by(|left, right| {
        right
            .priority_score
            .cmp(&left.priority_score)
            .then_with(|| left.node_id.cmp(&right.node_id))
    });
    for (index, entry) in entries.iter_mut().enumerate() {
        entry.priority_rank = index + 1;
    }
    let metric_consumed_count = entries.iter().filter(|entry| entry.metric_consumed).count();
    let prioritized_dirty_node_ids = entries
        .iter()
        .map(|entry| entry.node_id.clone())
        .collect::<Vec<_>>();

    IncrementalInvalidationPriorityPlanV0 {
        schema_version: "0",
        product: "omena-incremental.invalidation-priority-plan",
        feature_gate: "incremental-edit-distance-priority-v0",
        claim_level: "fixtureWitnessSchedulerPriority",
        theorem_claimed: false,
        public_safety_claim_ready: false,
        calibration_stage: "fixtureWitnessDistanceMarginWeightedV0",
        weight_profile: "editDistance10+cascadeMargin3+bridgeChecked1",
        metric_input_count: priority_inputs.len(),
        dirty_node_count: entries.len(),
        metric_consumed_count,
        prioritized_dirty_node_ids,
        entries,
    }
}

fn invalidation_priority_score(
    priority_input: Option<&IncrementalEditDistancePriorityInputV0>,
) -> u64 {
    const DIRTY_NODE_BASE_SCORE: u64 = 1_000;
    let Some(priority_input) = priority_input else {
        return DIRTY_NODE_BASE_SCORE;
    };
    DIRTY_NODE_BASE_SCORE
        + (priority_input.edit_distance_total as u64).saturating_mul(10)
        + priority_input.cascade_margin_abs_distance.saturating_mul(3)
        + u64::from(priority_input.bridge_checked)
}

pub fn run_incremental_consistency_fuzz_case(
    case: IncrementalConsistencyFuzzCaseV0,
) -> IncrementalConsistencyFuzzResultV0 {
    let node_count = case.node_count.clamp(1, 64);
    let previous_input = generated_incremental_fuzz_graph(case.seed, node_count, None);
    let previous_snapshot = snapshot_from_graph_input(&previous_input);
    let changed_index = case
        .changed_node_index
        .map(|index| index.min(node_count.saturating_sub(1)));
    let next_input = generated_incremental_fuzz_graph(case.seed, node_count, changed_index);
    let mut database = OmenaIncrementalDatabaseV0::default();
    database.restore_snapshot(&previous_snapshot);
    let plan = database
        .plan_and_upsert_graph_input(&next_input)
        .incremental_plan;
    let changed_node_id = changed_index.map(fuzz_node_id);
    let expected_dirty_ids = changed_node_id
        .as_ref()
        .map(|changed_id| transitive_dependents(&next_input, changed_id))
        .unwrap_or_default();
    let actual_dirty_ids = plan
        .nodes
        .iter()
        .filter(|node| node.dirty)
        .map(|node| node.id.clone())
        .collect::<BTreeSet<_>>();
    let expected_dirty_node_count = expected_dirty_ids.len();
    let passed = actual_dirty_ids == expected_dirty_ids
        && plan.dirty_node_count == expected_dirty_node_count
        && plan.changed_input_count == usize::from(changed_node_id.is_some());

    IncrementalConsistencyFuzzResultV0 {
        seed: case.seed,
        node_count,
        changed_node_id,
        dirty_node_count: plan.dirty_node_count,
        expected_dirty_node_count,
        passed,
    }
}

pub fn run_incremental_fuzz_seed_corpus() -> IncrementalFuzzSeedReportV0 {
    let seeds = [1, 2, 3, 5, 8, 13, 21, 34, 55, 89, 144, 233];
    let results = seeds
        .into_iter()
        .enumerate()
        .map(|(index, seed)| {
            run_incremental_consistency_fuzz_case(IncrementalConsistencyFuzzCaseV0 {
                seed,
                node_count: index + 1,
                changed_node_index: if index % 4 == 0 {
                    None
                } else {
                    Some(index / 2)
                },
            })
        })
        .collect::<Vec<_>>();
    let passed_count = results.iter().filter(|result| result.passed).count();
    let case_count = results.len();

    IncrementalFuzzSeedReportV0 {
        schema_version: "0",
        product: "omena-incremental.fuzz-seed-corpus",
        case_count,
        passed_count,
        failed_count: case_count - passed_count,
        results,
    }
}

pub fn summarize_incremental_layer_evidence_v0() -> IncrementalLayerEvidenceV0 {
    let boundary = summarize_omena_incremental_boundary();
    let fuzz_report = run_incremental_fuzz_seed_corpus();
    let mut database = OmenaIncrementalDatabaseV0::default();
    let previous_input = IncrementalGraphInputV0 {
        revision: IncrementalRevisionV0 { value: 1 },
        nodes: vec![
            IncrementalNodeInputV0 {
                id: "source".to_string(),
                digest: "source:v1".to_string(),
                dependency_ids: Vec::new(),
            },
            IncrementalNodeInputV0 {
                id: "style".to_string(),
                digest: "style:v1".to_string(),
                dependency_ids: vec!["source".to_string()],
            },
        ],
    };
    database.plan_and_upsert_graph_input(&previous_input);
    let sample_priority_inputs = vec![IncrementalEditDistancePriorityInputV0 {
        schema_version: "0",
        product: "omena-incremental.edit-distance-priority-input",
        feature_gate: "incremental-edit-distance-priority-v0",
        claim_level: "fixtureWitnessMetricInput",
        theorem_claimed: false,
        node_id: "style".to_string(),
        edit_distance_total: 3,
        cascade_margin_abs_distance: 2,
        bridge_checked: true,
        bridge_calibration_stage: "fixtureWitnessOnlyUncalibrated",
    }];
    let sample_update = database.plan_and_upsert_graph_input_with_priority_inputs(
        &IncrementalGraphInputV0 {
            revision: IncrementalRevisionV0 { value: 2 },
            nodes: vec![
                IncrementalNodeInputV0 {
                    id: "source".to_string(),
                    digest: "source:v2".to_string(),
                    dependency_ids: Vec::new(),
                },
                IncrementalNodeInputV0 {
                    id: "style".to_string(),
                    digest: "style:v1".to_string(),
                    dependency_ids: vec!["source".to_string()],
                },
            ],
        },
        &sample_priority_inputs,
    );

    IncrementalLayerEvidenceV0 {
        schema_version: "0",
        product: "omena-incremental.layer-evidence",
        claim_level: "m6IncrementalLayerEvidenceOnly",
        invalidation_layer: "stableNodeIdDigestDependencyGraph",
        real_invalidation_evidence_ready: sample_update.incremental_plan.changed_input_count == 1
            && sample_update.incremental_plan.dependency_dirty_count == 1
            && sample_update.incremental_plan.dirty_node_count == 2,
        fuzz_evidence_ready: fuzz_report.failed_count == 0,
        salsa_reuse_evidence_ready: boundary
            .ready_surfaces
            .contains(&"salsaTrackedNodeSnapshotQuery"),
        datalog_contract_evidence_ready: sample_update.datalog_rule_evaluator.fixed_point_reached
            && !sample_update.datalog_rule_evaluator.external_host_ready,
        value_equality_backdating_ready: sample_update.incremental_plan.nodes.iter().any(|node| {
            node.id == "style"
                && node.value_equal_to_previous
                && node.changed_at.value == 1
                && node.verified_at.value == 2
        }),
        alpha_equivalence_hash_ready: sample_update
            .incremental_plan
            .alpha_equivalence_graph_hash
            .feature_gate
            == "incremental-alpha-equivalence-hash-v0"
            && !sample_update
                .incremental_plan
                .alpha_equivalence_graph_hash
                .theorem_claimed,
        shadow_delta_oracle_ready: sample_update
            .incremental_plan
            .shadow_delta_oracle
            .incremental_matches_from_scratch_delta
            && !sample_update
                .incremental_plan
                .shadow_delta_oracle
                .theorem_claimed,
        edit_distance_priority_ready: sample_update
            .incremental_plan
            .invalidation_priority_plan
            .entries
            .iter()
            .any(|entry| {
                entry.node_id == "style"
                    && entry.metric_consumed
                    && entry.priority_kind == "editDistanceCascadeMarginWeighted"
            }),
        benchmark_surface_ready: true,
        performance_benchmark_claim_ready: false,
        external_datalog_host_ready: false,
        dbsp_zset_claim_ready: false,
        public_safety_claim_ready: false,
        benchmark_gate: "rust/z5-performance-baseline-readiness",
        benchmark_evidence_level: "configuredCriterionSurfaceNoTimingClaim",
        supported_claims: vec![
            "stable node id plus digest invalidation",
            "dependency dirty-set fixed point",
            "Salsa-backed tracked node snapshot reuse",
            "fuzzed dirty-set invariant corpus",
            "Datalog-shaped audit contract over the incremental plan",
            "value-equality backdating with changed_at/verified_at split",
            "alpha-equivalence-aware fixture hash",
            "sampled incremental-vs-from-scratch shadow delta oracle",
            "edit-distance weighted invalidation priority",
        ],
        deferred_claims: vec![
            "DBSP runtime",
            "Z-set differential dataflow semantics",
            "external Datalog host execution",
            "performance superiority from local timing data",
            "public safety claim",
        ],
        boundary,
        fuzz_report,
        sample_update,
    }
}

#[salsa::tracked(returns(clone))]
pub fn summarize_salsa_incremental_node_snapshot(
    db: &dyn salsa::Database,
    node: SalsaIncrementalNodeInputV0,
) -> IncrementalSnapshotNodeV0 {
    IncrementalSnapshotNodeV0 {
        id: node.id(db).clone(),
        digest: node.digest(db).clone(),
        dependency_ids: normalized_ids(node.dependency_ids(db)),
    }
}

#[salsa::tracked(returns(clone))]
pub fn read_salsa_incremental_node_digest(
    db: &dyn salsa::Database,
    node: SalsaIncrementalNodeInputV0,
) -> String {
    #[cfg(test)]
    omena_testkit::current_instrumentation_session_v0().record_salsa_digest_query_run();

    node.digest(db).clone()
}

#[salsa::tracked(returns(clone))]
pub fn read_salsa_incremental_node_dependency_ids(
    db: &dyn salsa::Database,
    node: SalsaIncrementalNodeInputV0,
) -> Vec<String> {
    #[cfg(test)]
    omena_testkit::current_instrumentation_session_v0().record_salsa_dependency_query_run();

    normalized_ids(node.dependency_ids(db))
}

#[salsa::tracked(returns(clone))]
fn read_salsa_incremental_node_dependency_edges(
    db: &dyn salsa::Database,
    node: SalsaIncrementalNodeInputV0,
) -> Vec<String> {
    normalized_ids(node.dependency_ids(db))
}

#[cfg(test)]
#[salsa::tracked(returns(clone))]
fn read_salsa_transitive_leaf(
    db: &dyn salsa::Database,
    node: SalsaIncrementalNodeInputV0,
) -> String {
    omena_testkit::current_instrumentation_session_v0().record_salsa_transitive_leaf_query_run();
    node.digest(db).clone()
}

#[cfg(test)]
#[salsa::tracked(returns(clone))]
fn read_salsa_transitive_a(db: &dyn salsa::Database, a: SalsaIncrementalNodeInputV0) -> String {
    omena_testkit::current_instrumentation_session_v0().record_salsa_transitive_a_query_run();
    format!("a={}", read_salsa_transitive_leaf(db, a))
}

#[cfg(test)]
#[salsa::tracked(returns(clone))]
fn read_salsa_transitive_b(
    db: &dyn salsa::Database,
    a: SalsaIncrementalNodeInputV0,
    b: SalsaIncrementalNodeInputV0,
) -> String {
    omena_testkit::current_instrumentation_session_v0().record_salsa_transitive_b_query_run();
    format!(
        "{}|b={}",
        read_salsa_transitive_a(db, a),
        read_salsa_transitive_leaf(db, b)
    )
}

#[cfg(test)]
#[salsa::tracked(returns(clone))]
fn read_salsa_transitive_c(
    db: &dyn salsa::Database,
    a: SalsaIncrementalNodeInputV0,
    b: SalsaIncrementalNodeInputV0,
    c: SalsaIncrementalNodeInputV0,
) -> String {
    omena_testkit::current_instrumentation_session_v0().record_salsa_transitive_c_query_run();
    format!(
        "{}|c={}",
        read_salsa_transitive_b(db, a, b),
        read_salsa_transitive_leaf(db, c)
    )
}

#[cfg(test)]
#[salsa::tracked(returns(clone))]
fn read_salsa_transitive_unrelated(
    db: &dyn salsa::Database,
    node: SalsaIncrementalNodeInputV0,
) -> String {
    omena_testkit::current_instrumentation_session_v0()
        .record_salsa_transitive_unrelated_query_run();
    format!("u={}", read_salsa_transitive_leaf(db, node))
}

#[salsa::tracked(returns(clone))]
pub fn read_salsa_file_revision_syntax_key(
    db: &dyn salsa::Database,
    input: SalsaIncrementalFileRevisionInputV0,
) -> String {
    let revision = input.revision(db);
    format!(
        "file={};revision={};syntax={}",
        input.file_id(db),
        revision.value,
        input.syntax_node_id(db)
    )
}

pub fn read_salsa_incremental_node_value(
    db: &dyn salsa::Database,
    graph: SalsaIncrementalGraphInputV0,
    node: SalsaIncrementalNodeInputV0,
) -> String {
    read_salsa_incremental_node_value_with_path(db, graph, node, String::new())
}

pub fn read_salsa_incremental_node_dirty_signature(
    db: &dyn salsa::Database,
    graph: SalsaIncrementalGraphInputV0,
    node: SalsaIncrementalNodeInputV0,
) -> String {
    read_salsa_incremental_node_dirty_signature_with_path(db, graph, node, String::new())
}

#[salsa::tracked(returns(clone))]
fn read_salsa_incremental_node_value_with_path(
    db: &dyn salsa::Database,
    graph: SalsaIncrementalGraphInputV0,
    node: SalsaIncrementalNodeInputV0,
    path_key: String,
) -> String {
    let id = node.id(db).clone();
    #[cfg(test)]
    record_salsa_node_value_query_run(id.as_str());

    if path_contains_id(path_key.as_str(), id.as_str()) {
        return format!("{id}=<cycle>");
    }

    let digest = node.digest(db).clone();
    let next_path = append_path_id(path_key.as_str(), id.as_str());
    let dependency_values = read_salsa_incremental_node_dependency_edges(db, node)
        .into_iter()
        .map(|dependency_id| {
            if path_contains_id(next_path.as_str(), dependency_id.as_str()) {
                return format!("{dependency_id}=<cycle>");
            }
            find_salsa_incremental_node_by_id(db, graph, dependency_id.as_str())
                .map(|dependency| {
                    read_salsa_incremental_node_value_with_path(
                        db,
                        graph,
                        dependency,
                        next_path.clone(),
                    )
                })
                .unwrap_or_else(|| format!("{dependency_id}=<missing>"))
        })
        .collect::<Vec<_>>();

    format!("{id}={digest};deps=[{}]", dependency_values.join(","))
}

#[salsa::tracked(returns(clone))]
fn read_salsa_incremental_node_dirty_signature_with_path(
    db: &dyn salsa::Database,
    graph: SalsaIncrementalGraphInputV0,
    node: SalsaIncrementalNodeInputV0,
    path_key: String,
) -> String {
    let id = node.id(db).clone();
    if path_contains_id(path_key.as_str(), id.as_str()) {
        return stable_hash_hex(format!("cycle:{id}").as_bytes());
    }

    let digest = node.digest(db).clone();
    let next_path = append_path_id(path_key.as_str(), id.as_str());
    let dependency_signatures = read_salsa_incremental_node_dependency_edges(db, node)
        .into_iter()
        .map(|dependency_id| {
            if path_contains_id(next_path.as_str(), dependency_id.as_str()) {
                return stable_hash_hex(format!("cycle:{dependency_id}").as_bytes());
            }
            find_salsa_incremental_node_by_id(db, graph, dependency_id.as_str())
                .map(|dependency| {
                    read_salsa_incremental_node_dirty_signature_with_path(
                        db,
                        graph,
                        dependency,
                        next_path.clone(),
                    )
                })
                .unwrap_or_else(|| stable_hash_hex(format!("missing:{dependency_id}").as_bytes()))
        })
        .collect::<Vec<_>>();
    let signature = format!("digest={digest};deps=[{}]", dependency_signatures.join(","));
    stable_hash_hex(signature.as_bytes())
}

fn find_salsa_incremental_node_by_id(
    db: &dyn salsa::Database,
    graph: SalsaIncrementalGraphInputV0,
    id: &str,
) -> Option<SalsaIncrementalNodeInputV0> {
    graph
        .nodes(db)
        .iter()
        .find(|node| node.id(db).as_str() == id)
        .copied()
}

fn path_contains_id(path_key: &str, id: &str) -> bool {
    path_key.split('\n').any(|entry| entry == id)
}

fn append_path_id(path_key: &str, id: &str) -> String {
    if path_key.is_empty() {
        id.to_string()
    } else {
        format!("{path_key}\n{id}")
    }
}

fn normalized_snapshot_nodes(input: &IncrementalGraphInputV0) -> Vec<IncrementalSnapshotNodeV0> {
    let mut nodes = input
        .nodes
        .iter()
        .map(|node| IncrementalSnapshotNodeV0 {
            id: node.id.clone(),
            digest: node.digest.clone(),
            dependency_ids: normalized_ids(&node.dependency_ids),
        })
        .collect::<Vec<_>>();
    nodes.sort_by(|left, right| left.id.cmp(&right.id));
    nodes
}

fn normalized_existing_snapshot_nodes(
    nodes: &[IncrementalSnapshotNodeV0],
) -> Vec<IncrementalSnapshotNodeV0> {
    let mut nodes = nodes
        .iter()
        .map(|node| IncrementalSnapshotNodeV0 {
            id: node.id.clone(),
            digest: node.digest.clone(),
            dependency_ids: normalized_ids(&node.dependency_ids),
        })
        .collect::<Vec<_>>();
    nodes.sort_by(|left, right| left.id.cmp(&right.id));
    nodes
}

fn normalized_ids(ids: &[String]) -> Vec<String> {
    ids.iter()
        .cloned()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn summarize_alpha_equivalence_hash(
    input: &IncrementalGraphInputV0,
) -> IncrementalAlphaEquivalenceHashV0 {
    let node_hashes = alpha_equivalence_hashes_by_node(input);
    let normalized_node_count = node_hashes.len();
    let normalized_edge_count = input
        .nodes
        .iter()
        .map(|node| normalized_ids(&node.dependency_ids).len())
        .sum();
    let mut labels = node_hashes.into_values().collect::<Vec<_>>();
    labels.sort();
    let labels = labels.join("|");
    let hash = stable_hash_hex(
        format!("nodes={normalized_node_count};edges={normalized_edge_count};labels={labels}")
            .as_bytes(),
    );

    IncrementalAlphaEquivalenceHashV0 {
        schema_version: "0",
        product: "omena-incremental.alpha-equivalence-hash",
        feature_gate: "incremental-alpha-equivalence-hash-v0",
        claim_level: "fixtureWitnessAlphaRenamingStableHash",
        theorem_claimed: false,
        hash,
        normalized_node_count,
        normalized_edge_count,
    }
}

fn alpha_equivalence_hashes_by_node(input: &IncrementalGraphInputV0) -> BTreeMap<String, String> {
    let nodes = normalized_snapshot_nodes(input);
    alpha_equivalence_hashes_for_snapshot_nodes(&nodes)
}

fn alpha_equivalence_hashes_by_snapshot_nodes(
    nodes: &[IncrementalSnapshotNodeV0],
) -> BTreeMap<String, String> {
    let nodes = normalized_existing_snapshot_nodes(nodes);
    alpha_equivalence_hashes_for_snapshot_nodes(&nodes)
}

fn alpha_equivalence_hashes_for_snapshot_nodes(
    nodes: &[IncrementalSnapshotNodeV0],
) -> BTreeMap<String, String> {
    let internal_ids = nodes
        .iter()
        .map(|node| node.id.as_str())
        .collect::<BTreeSet<_>>();
    let mut labels = nodes
        .iter()
        .map(|node| {
            (
                node.id.clone(),
                stable_hash_hex(format!("digest={}", node.digest).as_bytes()),
            )
        })
        .collect::<BTreeMap<_, _>>();

    for _ in 0..=nodes.len() {
        let next = nodes
            .iter()
            .map(|node| {
                let mut dependency_labels = node
                    .dependency_ids
                    .iter()
                    .map(|dependency_id| {
                        if internal_ids.contains(dependency_id.as_str()) {
                            labels
                                .get(dependency_id)
                                .cloned()
                                .unwrap_or_else(|| stable_hash_hex(dependency_id.as_bytes()))
                        } else {
                            format!("external:{dependency_id}")
                        }
                    })
                    .collect::<Vec<_>>();
                dependency_labels.sort();
                let signature = format!(
                    "digest={};deps={}",
                    node.digest,
                    dependency_labels.join(",")
                );
                (node.id.clone(), stable_hash_hex(signature.as_bytes()))
            })
            .collect::<BTreeMap<_, _>>();
        if next == labels {
            break;
        }
        labels = next;
    }

    labels
}

fn unique_previous_nodes_by_alpha_hash<'a>(
    nodes: &'a [IncrementalSnapshotNodeV0],
    hashes_by_id: &BTreeMap<String, String>,
) -> BTreeMap<String, &'a IncrementalSnapshotNodeV0> {
    let mut unique = BTreeMap::new();
    let mut duplicates = BTreeSet::new();
    for node in nodes {
        let Some(hash) = hashes_by_id.get(node.id.as_str()) else {
            continue;
        };
        if unique.insert(hash.clone(), node).is_some() {
            duplicates.insert(hash.clone());
        }
    }
    for duplicate in duplicates {
        unique.remove(duplicate.as_str());
    }
    unique
}

fn snapshot_node_value_matches(
    previous_node: &IncrementalSnapshotNodeV0,
    node: &IncrementalSnapshotNodeV0,
) -> bool {
    previous_node.digest == node.digest && previous_node.dependency_ids == node.dependency_ids
}

fn compute_from_scratch_delta_dirty_ids(
    input: &IncrementalGraphInputV0,
    previous: Option<&IncrementalSnapshotV0>,
) -> BTreeSet<String> {
    let normalized_nodes = normalized_snapshot_nodes(input);
    let alpha_hashes_by_id = alpha_equivalence_hashes_by_node(input);
    let previous_alpha_hashes_by_id = previous
        .map(|snapshot| alpha_equivalence_hashes_by_snapshot_nodes(&snapshot.nodes))
        .unwrap_or_default();
    let previous_by_alpha_hash = previous
        .map(|snapshot| {
            unique_previous_nodes_by_alpha_hash(&snapshot.nodes, &previous_alpha_hashes_by_id)
        })
        .unwrap_or_default();
    let previous_by_id = previous
        .map(|snapshot| {
            snapshot
                .nodes
                .iter()
                .map(|node| (node.id.as_str(), node))
                .collect::<BTreeMap<_, _>>()
        })
        .unwrap_or_default();
    let mut dirty_ids = normalized_nodes
        .iter()
        .filter_map(|node| {
            let alpha_equivalence_hash = alpha_hashes_by_id
                .get(node.id.as_str())
                .map(String::as_str)
                .unwrap_or(node.id.as_str());
            let exact_previous = previous_by_id.get(node.id.as_str()).copied();
            let alpha_previous = previous_by_alpha_hash.get(alpha_equivalence_hash).copied();
            if exact_previous
                .filter(|previous_node| snapshot_node_value_matches(previous_node, node))
                .or(alpha_previous)
                .is_some()
            {
                return None;
            }
            Some(node.id.clone())
        })
        .collect::<BTreeSet<_>>();

    let max_iterations = dependency_propagation_iteration_limit(normalized_nodes.len());
    for _ in 0..max_iterations {
        let mut changed = false;
        for node in &normalized_nodes {
            if dirty_ids.contains(node.id.as_str()) {
                continue;
            }
            if node
                .dependency_ids
                .iter()
                .any(|dependency_id| dirty_ids.contains(dependency_id.as_str()))
            {
                changed = dirty_ids.insert(node.id.clone()) || changed;
            }
        }
        if !changed {
            break;
        }
    }

    dirty_ids
}

fn stable_hash_hex(bytes: &[u8]) -> String {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("{hash:016x}")
}

fn dirty_set_is_dependency_closed(plan: &IncrementalComputationPlanV0) -> bool {
    let dirty_ids = plan
        .nodes
        .iter()
        .filter(|node| node.dirty)
        .map(|node| node.id.as_str())
        .collect::<BTreeSet<_>>();
    plan.nodes.iter().all(|node| {
        node.dirty
            || node
                .dependency_ids
                .iter()
                .all(|dependency_id| !dirty_ids.contains(dependency_id.as_str()))
    })
}

fn generated_incremental_fuzz_graph(
    seed: u64,
    node_count: usize,
    changed_index: Option<usize>,
) -> IncrementalGraphInputV0 {
    let mut state = seed ^ 0xa076_1d64_78bd_642f;
    let nodes = (0..node_count)
        .map(|index| {
            let id = fuzz_node_id(index);
            let mut digest_seed = fuzz_next(&mut state);
            if changed_index == Some(index) {
                digest_seed ^= 0xffff_ffff_ffff_ffff;
            }
            let dependency_ids = (0..index)
                .filter(|candidate| {
                    let divisor = ((*candidate + 2) as u64).max(2);
                    (seed + index as u64).is_multiple_of(divisor)
                })
                .map(fuzz_node_id)
                .collect::<Vec<_>>();
            IncrementalNodeInputV0 {
                id,
                digest: format!("digest-{index}-{digest_seed:016x}"),
                dependency_ids,
            }
        })
        .collect();

    IncrementalGraphInputV0 {
        revision: IncrementalRevisionV0 {
            value: 1 + if changed_index.is_some() { 1 } else { 0 },
        },
        nodes,
    }
}

fn transitive_dependents(input: &IncrementalGraphInputV0, changed_id: &str) -> BTreeSet<String> {
    let mut dirty_ids = BTreeSet::from([changed_id.to_string()]);
    let max_iterations = dependency_propagation_iteration_limit(input.nodes.len());
    for _ in 0..max_iterations {
        let mut changed = false;
        for node in &input.nodes {
            if dirty_ids.contains(&node.id) {
                continue;
            }
            if node
                .dependency_ids
                .iter()
                .any(|dependency_id| dirty_ids.contains(dependency_id))
            {
                changed = dirty_ids.insert(node.id.clone()) || changed;
            }
        }
        if !changed {
            break;
        }
    }
    dirty_ids
}

fn dependency_propagation_iteration_limit(node_count: usize) -> usize {
    node_count.saturating_add(1)
}

fn fuzz_node_id(index: usize) -> String {
    format!("node-{index}")
}

fn fuzz_next(state: &mut u64) -> u64 {
    *state = state
        .wrapping_mul(6_364_136_223_846_793_005)
        .wrapping_add(1_442_695_040_888_963_407);
    *state
}

impl OmenaIncrementalDatabaseV0 {
    pub fn salsa_database(&self) -> &OmenaSalsaDatabaseV0 {
        &self.db
    }

    pub fn node_input(&self, id: &str) -> Option<SalsaIncrementalNodeInputV0> {
        self.node_inputs_by_id.get(id).copied()
    }

    pub fn graph_input(&self) -> Option<SalsaIncrementalGraphInputV0> {
        self.graph_input
    }

    pub fn node_value(&self, id: &str) -> Option<String> {
        let graph = self.graph_input?;
        let node = self.node_input(id)?;
        Some(read_salsa_incremental_node_value(&self.db, graph, node))
    }

    pub fn node_dirty_signature(&self, id: &str) -> Option<String> {
        let graph = self.graph_input?;
        let node = self.node_input(id)?;
        Some(read_salsa_incremental_node_dirty_signature(
            &self.db, graph, node,
        ))
    }

    pub fn current_snapshot(&self) -> Option<&IncrementalSnapshotV0> {
        self.current_snapshot.as_ref()
    }

    pub fn restore_snapshot(&mut self, snapshot: &IncrementalSnapshotV0) {
        let input = graph_input_from_snapshot(snapshot);
        self.upsert_graph_input(&input);
        self.current_snapshot = Some(snapshot.clone());
    }

    pub fn plan_and_upsert_graph_input(
        &mut self,
        input: &IncrementalGraphInputV0,
    ) -> IncrementalDatabaseUpdateV0 {
        self.plan_and_upsert_graph_input_with_priority_inputs(input, &[])
    }

    pub fn plan_and_upsert_graph_input_with_priority_inputs(
        &mut self,
        input: &IncrementalGraphInputV0,
        priority_inputs: &[IncrementalEditDistancePriorityInputV0],
    ) -> IncrementalDatabaseUpdateV0 {
        let previous_snapshot = self.current_snapshot.clone();
        let previous_signatures = self.dirty_signatures_for_snapshot(previous_snapshot.as_ref());
        let next_snapshot = self.upsert_graph_input(input);
        let incremental_plan = self.salsa_demand_plan_with_priority_inputs(
            input,
            previous_snapshot.as_ref(),
            previous_signatures,
            priority_inputs,
        );
        let datalog_rule_evaluator =
            summarize_datalog_rule_evaluator_for_plan_v0(input, incremental_plan.clone());
        self.current_snapshot = Some(next_snapshot.clone());

        IncrementalDatabaseUpdateV0 {
            schema_version: "0",
            product: "omena-incremental.salsa-database-update",
            incremental_plan,
            datalog_rule_evaluator,
            next_snapshot,
        }
    }

    fn dirty_signatures_for_snapshot(
        &self,
        snapshot: Option<&IncrementalSnapshotV0>,
    ) -> (BTreeMap<String, String>, BTreeMap<String, String>) {
        let Some(snapshot) = snapshot else {
            return (BTreeMap::new(), BTreeMap::new());
        };
        let Some(graph) = self.graph_input else {
            return (BTreeMap::new(), BTreeMap::new());
        };
        let alpha_hashes_by_id = alpha_equivalence_hashes_by_snapshot_nodes(&snapshot.nodes);
        let unique_previous_by_alpha =
            unique_previous_nodes_by_alpha_hash(&snapshot.nodes, &alpha_hashes_by_id);
        let by_id = snapshot
            .nodes
            .iter()
            .filter_map(|node| {
                let node_input = self.node_input(node.id.as_str())?;
                Some((
                    node.id.clone(),
                    read_salsa_incremental_node_dirty_signature(&self.db, graph, node_input),
                ))
            })
            .collect::<BTreeMap<_, _>>();
        let by_alpha_hash = unique_previous_by_alpha
            .into_iter()
            .filter_map(|(alpha_hash, node)| {
                by_id
                    .get(node.id.as_str())
                    .cloned()
                    .map(|signature| (alpha_hash, signature))
            })
            .collect::<BTreeMap<_, _>>();
        (by_id, by_alpha_hash)
    }

    fn salsa_demand_plan_with_priority_inputs(
        &self,
        input: &IncrementalGraphInputV0,
        previous: Option<&IncrementalSnapshotV0>,
        previous_signatures: (BTreeMap<String, String>, BTreeMap<String, String>),
        priority_inputs: &[IncrementalEditDistancePriorityInputV0],
    ) -> IncrementalComputationPlanV0 {
        let normalized_nodes = normalized_snapshot_nodes(input);
        let alpha_hashes_by_id = alpha_equivalence_hashes_by_node(input);
        let alpha_equivalence_graph_hash = summarize_alpha_equivalence_hash(input);
        let previous_alpha_hashes_by_id = previous
            .map(|snapshot| alpha_equivalence_hashes_by_snapshot_nodes(&snapshot.nodes))
            .unwrap_or_default();
        let previous_by_alpha_hash = previous
            .map(|snapshot| {
                unique_previous_nodes_by_alpha_hash(&snapshot.nodes, &previous_alpha_hashes_by_id)
            })
            .unwrap_or_default();
        let previous_by_id = previous
            .map(|snapshot| {
                snapshot
                    .nodes
                    .iter()
                    .map(|node| (node.id.as_str(), node))
                    .collect::<BTreeMap<_, _>>()
            })
            .unwrap_or_default();
        let current_ids = normalized_nodes
            .iter()
            .map(|node| node.id.as_str())
            .collect::<BTreeSet<_>>();
        let current_alpha_hashes = alpha_hashes_by_id
            .values()
            .map(String::as_str)
            .collect::<BTreeSet<_>>();
        let removed_node_count = previous_by_id
            .keys()
            .filter(|id| {
                if current_ids.contains(**id) {
                    return false;
                }
                previous_alpha_hashes_by_id
                    .get(**id)
                    .is_none_or(|hash| !current_alpha_hashes.contains(hash.as_str()))
            })
            .count();
        let (previous_signature_by_id, previous_signature_by_alpha_hash) = previous_signatures;
        let mut dirty_ids = BTreeSet::<String>::new();
        let nodes = normalized_nodes
            .into_iter()
            .map(|node| {
                let alpha_equivalence_hash = alpha_hashes_by_id
                    .get(node.id.as_str())
                    .cloned()
                    .unwrap_or_else(|| stable_hash_hex(node.id.as_bytes()));
                let exact_previous = previous_by_id.get(node.id.as_str()).copied();
                let alpha_previous = previous_by_alpha_hash
                    .get(alpha_equivalence_hash.as_str())
                    .copied();
                let previous_value_match = exact_previous
                    .filter(|previous_node| snapshot_node_value_matches(previous_node, &node))
                    .or(alpha_previous);
                let current_signature = self.node_dirty_signature(node.id.as_str());
                let previous_signature = previous_value_match
                    .and_then(|previous_node| {
                        previous_signature_by_id.get(previous_node.id.as_str())
                    })
                    .or_else(|| {
                        exact_previous.and_then(|previous_node| {
                            previous_signature_by_id.get(previous_node.id.as_str())
                        })
                    })
                    .or_else(|| {
                        previous_signature_by_alpha_hash.get(alpha_equivalence_hash.as_str())
                    });
                let dependency_signature_equal = current_signature
                    .as_ref()
                    .zip(previous_signature)
                    .is_some_and(|(current, previous)| current == previous);
                let value_equal_to_previous = previous_value_match.is_some();
                let mut reasons = Vec::new();
                match previous_value_match.or(exact_previous) {
                    None => reasons.push("newNode"),
                    Some(previous_node) => {
                        if previous_value_match.is_none() {
                            if previous_node.digest != node.digest {
                                reasons.push("inputDigestChanged");
                            }
                            if previous_node.dependency_ids != node.dependency_ids {
                                reasons.push("dependencySetChanged");
                            }
                        }
                    }
                }
                if !dependency_signature_equal && reasons.is_empty() {
                    reasons.push("dependencyDirty");
                }
                let changed_at = if value_equal_to_previous {
                    previous
                        .map(|snapshot| snapshot.revision)
                        .unwrap_or(input.revision)
                } else {
                    input.revision
                };
                let dirty = !dependency_signature_equal;
                if dirty {
                    dirty_ids.insert(node.id.clone());
                }

                IncrementalComputationNodeV0 {
                    alpha_equivalence_hash,
                    id: node.id,
                    digest: node.digest,
                    dependency_ids: node.dependency_ids,
                    dirty,
                    reasons,
                    changed_at,
                    verified_at: input.revision,
                    value_equal_to_previous,
                }
            })
            .collect::<Vec<_>>();
        let shadow_delta_oracle =
            summarize_incremental_shadow_delta_oracle_v0(input, previous, dirty_ids);
        let invalidation_priority_plan =
            summarize_incremental_invalidation_priority_plan_v0(&nodes, priority_inputs);

        IncrementalComputationPlanV0 {
            schema_version: "0",
            product: "omena-incremental.computation-plan",
            revision: input.revision,
            node_count: nodes.len(),
            dirty_node_count: nodes.iter().filter(|node| node.dirty).count(),
            changed_input_count: nodes
                .iter()
                .filter(|node| node.reasons.contains(&"inputDigestChanged"))
                .count(),
            new_node_count: nodes
                .iter()
                .filter(|node| node.reasons.contains(&"newNode"))
                .count(),
            removed_node_count,
            dependency_dirty_count: nodes
                .iter()
                .filter(|node| node.reasons.contains(&"dependencyDirty"))
                .count(),
            alpha_equivalence_graph_hash,
            shadow_delta_oracle,
            invalidation_priority_plan,
            nodes,
        }
    }

    pub fn upsert_graph_input(&mut self, input: &IncrementalGraphInputV0) -> IncrementalSnapshotV0 {
        let normalized_nodes = normalized_snapshot_nodes(input);
        let current_ids = normalized_nodes
            .iter()
            .map(|node| node.id.as_str())
            .collect::<BTreeSet<_>>();
        self.node_inputs_by_id
            .retain(|id, _node| current_ids.contains(id.as_str()));

        for node in &normalized_nodes {
            self.upsert_node_input(node);
        }
        let graph_nodes = self.node_inputs_by_id.values().copied().collect::<Vec<_>>();
        self.sync_graph_input(graph_nodes);

        let nodes = self
            .node_inputs_by_id
            .values()
            .copied()
            .map(|node| summarize_salsa_incremental_node_snapshot(&self.db, node))
            .collect::<Vec<_>>();

        IncrementalSnapshotV0 {
            schema_version: "0",
            product: "omena-incremental.salsa-snapshot",
            revision: input.revision,
            nodes,
        }
    }

    fn upsert_node_input(&mut self, node: &IncrementalSnapshotNodeV0) {
        let Some(node_input) = self.node_inputs_by_id.get(node.id.as_str()).copied() else {
            let node_input = SalsaIncrementalNodeInputV0::new(
                &self.db,
                node.id.clone(),
                node.digest.clone(),
                node.dependency_ids.clone(),
            );
            self.node_inputs_by_id.insert(node.id.clone(), node_input);
            return;
        };

        if node_input.digest(&self.db).as_str() != node.digest.as_str() {
            node_input.set_digest(&mut self.db).to(node.digest.clone());
        }
        if node_input.dependency_ids(&self.db).as_slice() != node.dependency_ids.as_slice() {
            node_input
                .set_dependency_ids(&mut self.db)
                .to(node.dependency_ids.clone());
        }
    }

    fn sync_graph_input(&mut self, nodes: Vec<SalsaIncrementalNodeInputV0>) {
        match self.graph_input {
            Some(graph) => {
                if graph.nodes(&self.db).as_slice() != nodes.as_slice() {
                    graph.set_nodes(&mut self.db).to(nodes);
                }
            }
            None => {
                self.graph_input = Some(SalsaIncrementalGraphInputV0::new(&self.db, nodes));
            }
        }
    }
}

impl Default for IncrementalCancellationRegistryV0 {
    fn default() -> Self {
        Self::with_limit(DEFAULT_INCREMENTAL_CANCELLATION_LIMIT)
    }
}

impl IncrementalCancellationRegistryV0 {
    pub fn with_limit(limit: usize) -> Self {
        Self {
            limit: limit.max(1),
            cancelled_request_ids: BTreeSet::new(),
        }
    }

    pub fn cancel(&mut self, request_id: impl Into<String>) {
        if self.cancelled_request_ids.len() >= self.limit {
            self.cancelled_request_ids.clear();
        }
        self.cancelled_request_ids.insert(request_id.into());
    }

    pub fn take_cancelled(&mut self, request_id: &str) -> bool {
        self.cancelled_request_ids.remove(request_id)
    }

    pub fn take_cancelled_result(&mut self, request_id: &str) -> Result<(), salsa::Cancelled> {
        if self.take_cancelled(request_id) {
            Err(salsa::Cancelled::Local)
        } else {
            Ok(())
        }
    }

    pub fn len(&self) -> usize {
        self.cancelled_request_ids.len()
    }

    pub fn is_empty(&self) -> bool {
        self.cancelled_request_ids.is_empty()
    }

    pub fn snapshot(&self) -> IncrementalCancellationSnapshotV0 {
        IncrementalCancellationSnapshotV0 {
            schema_version: "0",
            product: "omena-incremental.cancellation-registry",
            cancelled_request_count: self.cancelled_request_ids.len(),
            cancelled_request_ids: self.cancelled_request_ids.iter().cloned().collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        GuaranteeKindV0, IncrementalCancellationRegistryV0, IncrementalGraphInputV0,
        IncrementalNodeInputV0, IncrementalRevisionV0, OmenaIncrementalDatabaseV0,
        OmenaSalsaDatabaseV0, SalsaIncrementalNodeInputV0,
        read_salsa_incremental_node_dependency_ids, read_salsa_incremental_node_digest,
        read_salsa_transitive_c, read_salsa_transitive_unrelated,
        reset_salsa_node_value_query_runs, salsa_node_value_query_runs, snapshot_from_graph_input,
        summarize_datalog_rule_evaluator_v0, summarize_incremental_layer_evidence_v0,
        summarize_omena_incremental_boundary,
    };
    use omena_testkit::{InstrumentationSessionV0, with_instrumentation_session};
    use salsa::Setter;
    use std::collections::BTreeSet;

    #[test]
    fn summarizes_incremental_boundary() {
        let summary = summarize_omena_incremental_boundary();

        assert_eq!(summary.product, "omena-incremental.boundary");
        assert_eq!(
            summary.query_model,
            "salsaInput+trackedQueryFieldGranularReuse"
        );
        assert_eq!(
            summary.dependency_propagation_policy,
            "salsaDemandDirtySignatureReads"
        );
        assert_eq!(
            summary.maximum_dependency_propagation_iterations,
            "oracleOnlyNodeCount+1"
        );
        assert!(summary.dirty_reasons.contains(&"dependencyDirty"));
        assert!(
            summary
                .ready_surfaces
                .contains(&"incrementalCancellationRegistry")
        );
        assert!(summary.ready_surfaces.contains(&"datalogRuleEvaluatorV0"));
        assert!(
            summary
                .ready_surfaces
                .contains(&"salsaTrackedNodeSnapshotQuery")
        );
        assert!(
            summary
                .ready_surfaces
                .contains(&"salsaDemandDependencyReads")
        );
    }

    #[test]
    fn first_plan_marks_all_nodes_dirty() {
        let input = sample_input("a:v1", "b:v1", 1);
        let plan = plan_from_database(&input, None);

        assert_eq!(plan.product, "omena-incremental.computation-plan");
        assert_eq!(plan.node_count, 2);
        assert_eq!(plan.dirty_node_count, 2);
        assert_eq!(plan.new_node_count, 2);
    }

    #[test]
    fn unchanged_second_plan_marks_nodes_clean() {
        let input = sample_input("a:v1", "b:v1", 1);
        let snapshot = snapshot_from_graph_input(&input);
        let next_input = sample_input("a:v1", "b:v1", 2);
        let plan = plan_from_database(&next_input, Some(&snapshot));

        assert_eq!(plan.dirty_node_count, 0);
        assert_eq!(plan.changed_input_count, 0);
        assert_eq!(
            plan.shadow_delta_oracle.incremental_dirty_ids,
            Vec::<String>::new()
        );
        assert!(
            plan.shadow_delta_oracle
                .incremental_matches_from_scratch_delta
        );
        let Some(b) = node_by_id(&plan, "b") else {
            assert!(plan.nodes.iter().any(|node| node.id == "b"));
            return;
        };
        assert_eq!(b.changed_at.value, 1);
        assert_eq!(b.verified_at.value, 2);
        assert!(b.value_equal_to_previous);
    }

    #[test]
    fn changed_dependency_marks_dependent_dirty() {
        let input = sample_input("a:v1", "b:v1", 1);
        let snapshot = snapshot_from_graph_input(&input);
        let next_input = sample_input("a:v2", "b:v1", 2);
        let plan = plan_from_database(&next_input, Some(&snapshot));

        assert_eq!(plan.changed_input_count, 1);
        assert_eq!(plan.dependency_dirty_count, 1);
        assert_eq!(node_reasons(&plan, "a"), vec!["inputDigestChanged"]);
        assert_eq!(node_reasons(&plan, "b"), vec!["dependencyDirty"]);
        assert_eq!(
            plan.shadow_delta_oracle.incremental_dirty_ids,
            vec!["a".to_string(), "b".to_string()]
        );
        assert_eq!(
            plan.shadow_delta_oracle.from_scratch_dirty_ids,
            plan.shadow_delta_oracle.incremental_dirty_ids
        );
        assert!(
            plan.shadow_delta_oracle
                .incremental_matches_from_scratch_delta
        );

        let Some(changed) = node_by_id(&plan, "a") else {
            assert!(plan.nodes.iter().any(|node| node.id == "a"));
            return;
        };
        assert_eq!(changed.changed_at.value, 2);
        assert_eq!(changed.verified_at.value, 2);
        assert!(!changed.value_equal_to_previous);

        let Some(backdated) = node_by_id(&plan, "b") else {
            assert!(plan.nodes.iter().any(|node| node.id == "b"));
            return;
        };
        assert_eq!(backdated.changed_at.value, 1);
        assert_eq!(backdated.verified_at.value, 2);
        assert!(backdated.value_equal_to_previous);
    }

    #[test]
    fn alpha_equivalence_hash_ignores_fixture_node_renaming() {
        let left = sample_input("root:v1", "leaf:v1", 1);
        let right = IncrementalGraphInputV0 {
            revision: IncrementalRevisionV0 { value: 1 },
            nodes: vec![
                IncrementalNodeInputV0 {
                    id: "renamed-leaf".to_string(),
                    digest: "leaf:v1".to_string(),
                    dependency_ids: vec!["renamed-root".to_string()],
                },
                IncrementalNodeInputV0 {
                    id: "renamed-root".to_string(),
                    digest: "root:v1".to_string(),
                    dependency_ids: Vec::new(),
                },
            ],
        };
        let left_plan = plan_from_database(&left, None);
        let right_plan = plan_from_database(&right, None);

        assert_eq!(
            left_plan.alpha_equivalence_graph_hash.product,
            "omena-incremental.alpha-equivalence-hash"
        );
        assert_eq!(
            left_plan.alpha_equivalence_graph_hash.feature_gate,
            "incremental-alpha-equivalence-hash-v0"
        );
        assert!(!left_plan.alpha_equivalence_graph_hash.theorem_claimed);
        assert_eq!(
            left_plan.alpha_equivalence_graph_hash.hash,
            right_plan.alpha_equivalence_graph_hash.hash
        );
    }

    #[test]
    fn preceding_sibling_insert_keeps_shifted_nodes_clean_by_alpha_hash() {
        let previous = IncrementalGraphInputV0 {
            revision: IncrementalRevisionV0 { value: 1 },
            nodes: vec![
                IncrementalNodeInputV0 {
                    id: "path:0".to_string(),
                    digest: "button:v1".to_string(),
                    dependency_ids: Vec::new(),
                },
                IncrementalNodeInputV0 {
                    id: "path:1".to_string(),
                    digest: "card:v1".to_string(),
                    dependency_ids: Vec::new(),
                },
            ],
        };
        let previous_snapshot = snapshot_from_graph_input(&previous);
        let next = IncrementalGraphInputV0 {
            revision: IncrementalRevisionV0 { value: 2 },
            nodes: vec![
                IncrementalNodeInputV0 {
                    id: "path:0".to_string(),
                    digest: "import:v1".to_string(),
                    dependency_ids: Vec::new(),
                },
                IncrementalNodeInputV0 {
                    id: "path:1".to_string(),
                    digest: "button:v1".to_string(),
                    dependency_ids: Vec::new(),
                },
                IncrementalNodeInputV0 {
                    id: "path:2".to_string(),
                    digest: "card:v1".to_string(),
                    dependency_ids: Vec::new(),
                },
            ],
        };
        let plan = plan_from_database(&next, Some(&previous_snapshot));
        let dirty_ids = plan
            .nodes
            .iter()
            .filter(|node| node.dirty)
            .map(|node| node.id.as_str())
            .collect::<Vec<_>>();

        assert_eq!(dirty_ids, vec!["path:0"]);
        assert!(node_by_id(&plan, "path:1").is_some_and(|node| node.value_equal_to_previous));
        assert!(node_by_id(&plan, "path:2").is_some_and(|node| node.value_equal_to_previous));
    }

    #[test]
    fn incremental_shadow_delta_oracle_matches_from_scratch_delta() {
        let previous = sample_input("a:v1", "b:v1", 1);
        let previous_snapshot = snapshot_from_graph_input(&previous);
        let next = sample_input("a:v2", "b:v1", 2);
        let plan = plan_from_database(&next, Some(&previous_snapshot));

        assert_eq!(
            plan.shadow_delta_oracle.product,
            "omena-incremental.shadow-delta-oracle"
        );
        assert_eq!(
            plan.shadow_delta_oracle.feature_gate,
            "incremental-shadow-delta-v0"
        );
        assert_eq!(
            plan.shadow_delta_oracle.claim_level,
            "sampledFixtureWitnessNotEquivalenceProof"
        );
        assert!(!plan.shadow_delta_oracle.theorem_claimed);
        assert_eq!(
            plan.shadow_delta_oracle.incremental_dirty_ids,
            vec!["a".to_string(), "b".to_string()]
        );
        assert_eq!(
            plan.shadow_delta_oracle.incremental_dirty_ids,
            plan.shadow_delta_oracle.from_scratch_dirty_ids
        );
        assert!(
            plan.shadow_delta_oracle
                .incremental_matches_from_scratch_delta
        );
        assert!(plan.shadow_delta_oracle.sampled_shadow_witness_ready);
        assert!(!plan.shadow_delta_oracle.dbsp_zset_claim_ready);
        assert!(!plan.shadow_delta_oracle.performance_benchmark_claim_ready);
    }

    #[test]
    fn edit_distance_priority_orders_dirty_nodes_for_scheduler() {
        let previous = three_node_input("a:v1", "b:v1", "c:v1", 1);
        let previous_snapshot = snapshot_from_graph_input(&previous);
        let next = three_node_input("a:v2", "b:v2", "c:v1", 2);
        let plan = plan_from_database_with_priority_inputs(
            &next,
            Some(&previous_snapshot),
            &[
                priority_input("a", 1, 1, true),
                priority_input("b", 8, 2, true),
            ],
        );

        assert_eq!(
            plan.invalidation_priority_plan.product,
            "omena-incremental.invalidation-priority-plan"
        );
        assert_eq!(
            plan.invalidation_priority_plan.feature_gate,
            "incremental-edit-distance-priority-v0"
        );
        assert_eq!(
            plan.invalidation_priority_plan.calibration_stage,
            "fixtureWitnessDistanceMarginWeightedV0"
        );
        assert!(!plan.invalidation_priority_plan.theorem_claimed);
        assert!(!plan.invalidation_priority_plan.public_safety_claim_ready);
        assert_eq!(plan.invalidation_priority_plan.metric_input_count, 2);
        assert_eq!(plan.invalidation_priority_plan.metric_consumed_count, 2);
        assert_eq!(
            plan.invalidation_priority_plan.prioritized_dirty_node_ids,
            vec!["b".to_string(), "a".to_string(), "c".to_string()]
        );
        let first = &plan.invalidation_priority_plan.entries[0];
        assert_eq!(first.node_id, "b");
        assert_eq!(first.priority_rank, 1);
        assert!(first.metric_consumed);
        assert_eq!(first.edit_distance_total, Some(8));
        assert_eq!(first.cascade_margin_abs_distance, Some(2));
        assert!(first.bridge_checked);
    }

    #[test]
    fn datalog_rule_evaluator_contract_matches_incremental_dirty_plan() {
        let input = sample_input("a:v1", "b:v1", 1);
        let snapshot = snapshot_from_graph_input(&input);
        let next_input = sample_input("a:v2", "b:v1", 2);
        let summary = summarize_datalog_rule_evaluator_v0(&next_input, Some(&snapshot));

        assert_eq!(summary.schema_version, "0");
        assert_eq!(summary.product, "omena-incremental.datalog-rule-evaluator");
        assert_eq!(summary.evaluator_kind, "typedContractOverSalsaDemandPlan");
        assert_eq!(
            summary.substrate,
            "omena-incremental.salsa-backed-computation-plan"
        );
        assert!(!summary.external_host_ready);
        assert_eq!(summary.rule_count, summary.rules.len());
        assert_eq!(summary.relation_count, summary.relations.len());
        assert_eq!(summary.input_node_count, 2);
        assert_eq!(summary.dirty_node_count, 2);
        assert_eq!(summary.derived_node_count, 1);
        assert_eq!(summary.iteration_limit, 3);
        assert!(summary.fixed_point_reached);
        assert_eq!(summary.incremental_plan.changed_input_count, 1);
        assert_eq!(summary.incremental_plan.dependency_dirty_count, 1);
        assert!(summary.rules.iter().any(|rule| {
            rule.name == "dependencyDirtyDemandRead"
                && rule.body == vec!["dependsOn(Node,Dependency)", "dirty(Dependency)"]
        }));
    }

    #[test]
    fn datalog_rule_evaluator_fixture_corpus_matches_incremental_fixed_point() {
        for seed in [1, 2, 3, 5, 8, 13, 21, 34] {
            let previous_input = super::generated_incremental_fuzz_graph(seed, 8, None);
            let previous_snapshot = snapshot_from_graph_input(&previous_input);
            let next_input = super::generated_incremental_fuzz_graph(seed, 8, Some(3));
            let plan = plan_from_database(&next_input, Some(&previous_snapshot));
            let summary =
                summarize_datalog_rule_evaluator_v0(&next_input, Some(&previous_snapshot));

            assert_eq!(summary.incremental_plan, plan);
            assert_eq!(summary.dirty_node_count, plan.dirty_node_count);
            assert_eq!(summary.derived_node_count, plan.dependency_dirty_count);
            assert!(summary.fixed_point_reached);
            assert!(!summary.external_host_ready);
            assert_eq!(summary.rule_count, 4);
            assert_eq!(summary.relation_count, 5);
        }
    }

    #[test]
    fn cyclic_dependency_graph_uses_bounded_dirty_propagation() {
        let input = cyclic_input("a:v1", "b:v1", 1);
        let snapshot = snapshot_from_graph_input(&input);
        let next_input = cyclic_input("a:v2", "b:v1", 2);
        let plan = plan_from_database(&next_input, Some(&snapshot));

        assert_eq!(plan.changed_input_count, 1);
        assert_eq!(plan.dirty_node_count, 2);
        assert_eq!(node_reasons(&plan, "a"), vec!["inputDigestChanged"]);
        assert_eq!(node_reasons(&plan, "b"), vec!["dependencyDirty"]);
        assert_eq!(
            super::dependency_propagation_iteration_limit(input.nodes.len()),
            input.nodes.len() + 1
        );
    }

    #[test]
    fn fuzz_seed_corpus_preserves_incremental_dirty_set_invariants() {
        let report = super::run_incremental_fuzz_seed_corpus();

        assert_eq!(report.product, "omena-incremental.fuzz-seed-corpus");
        assert_eq!(report.failed_count, 0);
        assert_eq!(report.passed_count, report.case_count);
        assert!(
            report
                .results
                .iter()
                .any(|result| result.changed_node_id.is_none())
        );
        assert!(
            report
                .results
                .iter()
                .any(|result| result.expected_dirty_node_count > 1)
        );
    }

    #[test]
    fn m6_incremental_layer_evidence_is_limited_to_real_invalidation_layer() {
        let evidence = summarize_incremental_layer_evidence_v0();

        assert_eq!(evidence.schema_version, "0");
        assert_eq!(evidence.product, "omena-incremental.layer-evidence");
        assert_eq!(evidence.claim_level, "m6IncrementalLayerEvidenceOnly");
        assert_eq!(
            evidence.invalidation_layer,
            "stableNodeIdDigestDependencyGraph"
        );
        assert!(evidence.real_invalidation_evidence_ready);
        assert!(evidence.fuzz_evidence_ready);
        assert!(evidence.salsa_reuse_evidence_ready);
        assert!(evidence.datalog_contract_evidence_ready);
        assert!(evidence.value_equality_backdating_ready);
        assert!(evidence.alpha_equivalence_hash_ready);
        assert!(evidence.shadow_delta_oracle_ready);
        assert!(evidence.edit_distance_priority_ready);
        assert!(evidence.benchmark_surface_ready);
        assert!(!evidence.performance_benchmark_claim_ready);
        assert!(!evidence.external_datalog_host_ready);
        assert!(!evidence.dbsp_zset_claim_ready);
        assert!(!evidence.public_safety_claim_ready);
        assert_eq!(
            evidence.benchmark_gate,
            "rust/z5-performance-baseline-readiness"
        );
        assert_eq!(evidence.fuzz_report.failed_count, 0);
        assert_eq!(
            evidence.sample_update.incremental_plan.changed_input_count,
            1
        );
        assert_eq!(
            evidence
                .sample_update
                .incremental_plan
                .dependency_dirty_count,
            1
        );
        assert!(
            evidence
                .supported_claims
                .contains(&"dependency dirty-set fixed point")
        );
        assert!(
            evidence
                .supported_claims
                .contains(&"value-equality backdating with changed_at/verified_at split")
        );
        assert!(
            evidence
                .supported_claims
                .contains(&"edit-distance weighted invalidation priority")
        );
        assert!(evidence.deferred_claims.contains(&"DBSP runtime"));
        assert!(
            evidence
                .deferred_claims
                .contains(&"Z-set differential dataflow semantics")
        );
    }

    #[test]
    fn incremental_claim_levels_round_trip_to_guarantee_kinds() {
        let evidence = summarize_incremental_layer_evidence_v0();
        let plan = &evidence.sample_update.incremental_plan;
        let priority_input = priority_input("style", 3, 2, true);

        for claim_level in [
            evidence.claim_level,
            plan.alpha_equivalence_graph_hash.claim_level,
            plan.shadow_delta_oracle.claim_level,
            plan.invalidation_priority_plan.claim_level,
            priority_input.claim_level,
        ] {
            assert_eq!(
                GuaranteeKindV0::from_existing_label(claim_level)
                    .and_then(GuaranteeKindV0::existing_label),
                Some(claim_level)
            );
        }
    }

    #[test]
    fn incremental_layer_evidence_graph_preserves_public_shape() -> Result<(), String> {
        let evidence = summarize_incremental_layer_evidence_v0();
        let before = serde_json::to_value(&evidence).map_err(|error| error.to_string())?;
        let graph = evidence
            .evidence_graph()
            .map_err(|error| format!("{error:?}"))?;
        let after = serde_json::to_value(&evidence).map_err(|error| error.to_string())?;

        assert_eq!(before, after);
        assert_eq!(graph.nodes.len(), 4);
        assert_eq!(graph.edges.len(), 4);
        let labels = graph
            .nodes
            .iter()
            .map(|node| node.guarantee.existing_label())
            .collect::<Vec<_>>();
        assert!(labels.contains(&Some("m6IncrementalLayerEvidenceOnly")));
        assert!(labels.contains(&Some("fixtureWitnessAlphaRenamingStableHash")));
        assert!(labels.contains(&Some("sampledFixtureWitnessNotEquivalenceProof")));
        assert!(labels.contains(&Some("fixtureWitnessSchedulerPriority")));
        Ok(())
    }

    #[test]
    fn salsa_database_reuses_digest_query_when_only_dependencies_change() {
        let session = InstrumentationSessionV0::default();
        with_instrumentation_session(session.clone(), || {
            session.reset_salsa_query_run_counts();

            let mut db = OmenaIncrementalDatabaseV0::default();
            let input = IncrementalGraphInputV0 {
                revision: IncrementalRevisionV0 { value: 1 },
                nodes: vec![IncrementalNodeInputV0 {
                    id: "a".to_string(),
                    digest: "a:v1".to_string(),
                    dependency_ids: Vec::new(),
                }],
            };
            let snapshot = db.upsert_graph_input(&input);
            assert_eq!(snapshot.product, "omena-incremental.salsa-snapshot");

            let Some(node) = db.node_input("a") else {
                return;
            };
            assert_eq!(
                read_salsa_incremental_node_digest(db.salsa_database(), node),
                "a:v1"
            );
            assert_eq!(
                read_salsa_incremental_node_dependency_ids(db.salsa_database(), node),
                Vec::<String>::new()
            );
            let counts = session.salsa_query_run_counts();
            assert_eq!(counts.digest, 1);
            assert_eq!(counts.dependency, 1);

            let next_input = IncrementalGraphInputV0 {
                revision: IncrementalRevisionV0 { value: 2 },
                nodes: vec![IncrementalNodeInputV0 {
                    id: "a".to_string(),
                    digest: "a:v1".to_string(),
                    dependency_ids: vec!["root".to_string()],
                }],
            };
            db.upsert_graph_input(&next_input);

            let Some(node) = db.node_input("a") else {
                return;
            };
            assert_eq!(
                read_salsa_incremental_node_digest(db.salsa_database(), node),
                "a:v1"
            );
            assert_eq!(
                read_salsa_incremental_node_dependency_ids(db.salsa_database(), node),
                vec!["root".to_string()]
            );
            let counts = session.salsa_query_run_counts();
            assert_eq!(counts.digest, 1);
            assert_eq!(counts.dependency, 2);
        });
    }

    #[test]
    fn salsa_transitive_query_graph_matches_planner_dirty_set() {
        let session = InstrumentationSessionV0::default();
        with_instrumentation_session(session.clone(), || {
            session.reset_salsa_query_run_counts();

            let mut db = OmenaSalsaDatabaseV0::new();
            let a = SalsaIncrementalNodeInputV0::new(
                &db,
                "a".to_string(),
                "a:v1".to_string(),
                Vec::new(),
            );
            let b = SalsaIncrementalNodeInputV0::new(
                &db,
                "b".to_string(),
                "b:v1".to_string(),
                vec!["a".to_string()],
            );
            let c = SalsaIncrementalNodeInputV0::new(
                &db,
                "c".to_string(),
                "c:v1".to_string(),
                vec!["b".to_string()],
            );
            let unrelated = SalsaIncrementalNodeInputV0::new(
                &db,
                "unrelated".to_string(),
                "u:v1".to_string(),
                Vec::new(),
            );

            assert_eq!(
                read_salsa_transitive_c(&db, a, b, c),
                "a=a:v1|b=b:v1|c=c:v1"
            );
            assert_eq!(read_salsa_transitive_unrelated(&db, unrelated), "u=u:v1");

            session.reset_salsa_query_run_counts();

            a.set_digest(&mut db).to("a:v2".to_string());

            assert_eq!(
                read_salsa_transitive_c(&db, a, b, c),
                "a=a:v2|b=b:v1|c=c:v1"
            );
            assert_eq!(read_salsa_transitive_unrelated(&db, unrelated), "u=u:v1");

            let counts = session.salsa_query_run_counts();
            assert_eq!(counts.transitive_leaf, 1);
            assert_eq!(counts.transitive_a, 1);
            assert_eq!(counts.transitive_b, 1);
            assert_eq!(counts.transitive_c, 1);
            assert_eq!(counts.transitive_unrelated, 0);

            let previous = IncrementalGraphInputV0 {
                revision: IncrementalRevisionV0 { value: 1 },
                nodes: vec![
                    IncrementalNodeInputV0 {
                        id: "a".to_string(),
                        digest: "a:v1".to_string(),
                        dependency_ids: Vec::new(),
                    },
                    IncrementalNodeInputV0 {
                        id: "b".to_string(),
                        digest: "b:v1".to_string(),
                        dependency_ids: vec!["a".to_string()],
                    },
                    IncrementalNodeInputV0 {
                        id: "c".to_string(),
                        digest: "c:v1".to_string(),
                        dependency_ids: vec!["b".to_string()],
                    },
                    IncrementalNodeInputV0 {
                        id: "unrelated".to_string(),
                        digest: "u:v1".to_string(),
                        dependency_ids: Vec::new(),
                    },
                ],
            };
            let next = IncrementalGraphInputV0 {
                revision: IncrementalRevisionV0 { value: 2 },
                nodes: vec![
                    IncrementalNodeInputV0 {
                        id: "a".to_string(),
                        digest: "a:v2".to_string(),
                        dependency_ids: Vec::new(),
                    },
                    IncrementalNodeInputV0 {
                        id: "b".to_string(),
                        digest: "b:v1".to_string(),
                        dependency_ids: vec!["a".to_string()],
                    },
                    IncrementalNodeInputV0 {
                        id: "c".to_string(),
                        digest: "c:v1".to_string(),
                        dependency_ids: vec!["b".to_string()],
                    },
                    IncrementalNodeInputV0 {
                        id: "unrelated".to_string(),
                        digest: "u:v1".to_string(),
                        dependency_ids: Vec::new(),
                    },
                ],
            };
            let previous_snapshot = snapshot_from_graph_input(&previous);
            let plan = plan_from_database(&next, Some(&previous_snapshot));
            let planner_dirty_ids = plan
                .nodes
                .iter()
                .filter(|node| node.dirty)
                .map(|node| node.id.as_str())
                .collect::<BTreeSet<_>>();
            let salsa_rerun_ids = ["a", "b", "c"].into_iter().collect::<BTreeSet<_>>();

            assert_eq!(planner_dirty_ids, salsa_rerun_ids);
        });
    }

    #[test]
    fn production_node_value_query_reads_only_transitive_dependencies() {
        let mut db = OmenaIncrementalDatabaseV0::default();
        let input = IncrementalGraphInputV0 {
            revision: IncrementalRevisionV0 { value: 1 },
            nodes: vec![
                IncrementalNodeInputV0 {
                    id: "a".to_string(),
                    digest: "a:v1".to_string(),
                    dependency_ids: Vec::new(),
                },
                IncrementalNodeInputV0 {
                    id: "b".to_string(),
                    digest: "b:v1".to_string(),
                    dependency_ids: vec!["a".to_string()],
                },
                IncrementalNodeInputV0 {
                    id: "c".to_string(),
                    digest: "c:v1".to_string(),
                    dependency_ids: vec!["b".to_string()],
                },
                IncrementalNodeInputV0 {
                    id: "unrelated".to_string(),
                    digest: "u:v1".to_string(),
                    dependency_ids: Vec::new(),
                },
            ],
        };
        db.upsert_graph_input(&input);

        assert_eq!(
            db.node_value("c"),
            Some("c=c:v1;deps=[b=b:v1;deps=[a=a:v1;deps=[]]]".to_string())
        );
        assert_eq!(
            db.node_value("unrelated"),
            Some("unrelated=u:v1;deps=[]".to_string())
        );

        reset_salsa_node_value_query_runs();
        let next = IncrementalGraphInputV0 {
            revision: IncrementalRevisionV0 { value: 2 },
            nodes: vec![
                IncrementalNodeInputV0 {
                    id: "a".to_string(),
                    digest: "a:v2".to_string(),
                    dependency_ids: Vec::new(),
                },
                IncrementalNodeInputV0 {
                    id: "b".to_string(),
                    digest: "b:v1".to_string(),
                    dependency_ids: vec!["a".to_string()],
                },
                IncrementalNodeInputV0 {
                    id: "c".to_string(),
                    digest: "c:v1".to_string(),
                    dependency_ids: vec!["b".to_string()],
                },
                IncrementalNodeInputV0 {
                    id: "unrelated".to_string(),
                    digest: "u:v1".to_string(),
                    dependency_ids: Vec::new(),
                },
            ],
        };
        db.upsert_graph_input(&next);

        assert_eq!(
            db.node_value("c"),
            Some("c=c:v1;deps=[b=b:v1;deps=[a=a:v2;deps=[]]]".to_string())
        );
        assert_eq!(
            db.node_value("unrelated"),
            Some("unrelated=u:v1;deps=[]".to_string())
        );
        assert_eq!(salsa_node_value_query_runs("a"), 1);
        assert_eq!(salsa_node_value_query_runs("b"), 1);
        assert_eq!(salsa_node_value_query_runs("c"), 1);
        assert_eq!(salsa_node_value_query_runs("unrelated"), 0);
    }

    #[test]
    fn salsa_database_update_owns_plan_and_snapshot_progression() {
        let mut db = OmenaIncrementalDatabaseV0::default();
        let input = sample_input("a:v1", "b:v1", 1);
        let first = db.plan_and_upsert_graph_input(&input);

        assert_eq!(first.product, "omena-incremental.salsa-database-update");
        assert_eq!(first.incremental_plan.dirty_node_count, 2);
        assert_eq!(
            first.next_snapshot.product,
            "omena-incremental.salsa-snapshot"
        );
        assert!(db.current_snapshot().is_some());

        let unchanged = db.plan_and_upsert_graph_input(&sample_input("a:v1", "b:v1", 2));
        assert_eq!(unchanged.incremental_plan.dirty_node_count, 0);

        let changed = db.plan_and_upsert_graph_input(&sample_input("a:v2", "b:v1", 3));
        assert_eq!(changed.incremental_plan.changed_input_count, 1);
        assert_eq!(changed.incremental_plan.dependency_dirty_count, 1);
        assert_eq!(changed.datalog_rule_evaluator.revision.value, 3);
        assert_eq!(
            changed.datalog_rule_evaluator.incremental_plan,
            changed.incremental_plan
        );
        assert_eq!(changed.datalog_rule_evaluator.dirty_node_count, 2);
        assert_eq!(changed.datalog_rule_evaluator.derived_node_count, 1);
        assert!(changed.datalog_rule_evaluator.fixed_point_reached);
        assert!(!changed.datalog_rule_evaluator.external_host_ready);
    }

    #[test]
    fn cancellation_registry_tracks_and_consumes_request_ids() {
        let mut registry = IncrementalCancellationRegistryV0::with_limit(4);

        registry.cancel("s:hover-1");

        assert_eq!(registry.len(), 1);
        assert!(matches!(
            registry.take_cancelled_result("s:hover-1"),
            Err(salsa::Cancelled::Local)
        ));
        assert!(matches!(
            registry.take_cancelled_result("s:hover-1"),
            Ok(())
        ));
        assert!(registry.is_empty());
    }

    #[test]
    fn cancellation_registry_bounds_stale_cancelled_requests() {
        let mut registry = IncrementalCancellationRegistryV0::with_limit(2);

        registry.cancel("n:1");
        registry.cancel("n:2");
        registry.cancel("n:3");

        let snapshot = registry.snapshot();
        assert_eq!(snapshot.product, "omena-incremental.cancellation-registry");
        assert_eq!(snapshot.cancelled_request_ids, vec!["n:3"]);
    }

    fn sample_input(a_digest: &str, b_digest: &str, revision: u64) -> IncrementalGraphInputV0 {
        IncrementalGraphInputV0 {
            revision: IncrementalRevisionV0 { value: revision },
            nodes: vec![
                IncrementalNodeInputV0 {
                    id: "b".to_string(),
                    digest: b_digest.to_string(),
                    dependency_ids: vec!["a".to_string()],
                },
                IncrementalNodeInputV0 {
                    id: "a".to_string(),
                    digest: a_digest.to_string(),
                    dependency_ids: Vec::new(),
                },
            ],
        }
    }

    fn cyclic_input(a_digest: &str, b_digest: &str, revision: u64) -> IncrementalGraphInputV0 {
        IncrementalGraphInputV0 {
            revision: IncrementalRevisionV0 { value: revision },
            nodes: vec![
                IncrementalNodeInputV0 {
                    id: "a".to_string(),
                    digest: a_digest.to_string(),
                    dependency_ids: vec!["b".to_string()],
                },
                IncrementalNodeInputV0 {
                    id: "b".to_string(),
                    digest: b_digest.to_string(),
                    dependency_ids: vec!["a".to_string()],
                },
            ],
        }
    }

    fn three_node_input(
        a_digest: &str,
        b_digest: &str,
        c_digest: &str,
        revision: u64,
    ) -> IncrementalGraphInputV0 {
        IncrementalGraphInputV0 {
            revision: IncrementalRevisionV0 { value: revision },
            nodes: vec![
                IncrementalNodeInputV0 {
                    id: "a".to_string(),
                    digest: a_digest.to_string(),
                    dependency_ids: Vec::new(),
                },
                IncrementalNodeInputV0 {
                    id: "b".to_string(),
                    digest: b_digest.to_string(),
                    dependency_ids: Vec::new(),
                },
                IncrementalNodeInputV0 {
                    id: "c".to_string(),
                    digest: c_digest.to_string(),
                    dependency_ids: vec!["a".to_string()],
                },
            ],
        }
    }

    fn priority_input(
        node_id: &str,
        edit_distance_total: usize,
        cascade_margin_abs_distance: u64,
        bridge_checked: bool,
    ) -> super::IncrementalEditDistancePriorityInputV0 {
        super::IncrementalEditDistancePriorityInputV0 {
            schema_version: "0",
            product: "omena-incremental.edit-distance-priority-input",
            feature_gate: "incremental-edit-distance-priority-v0",
            claim_level: "fixtureWitnessMetricInput",
            theorem_claimed: false,
            node_id: node_id.to_string(),
            edit_distance_total,
            cascade_margin_abs_distance,
            bridge_checked,
            bridge_calibration_stage: "fixtureWitnessOnlyUncalibrated",
        }
    }

    fn node_by_id<'a>(
        plan: &'a super::IncrementalComputationPlanV0,
        id: &str,
    ) -> Option<&'a super::IncrementalComputationNodeV0> {
        plan.nodes.iter().find(|node| node.id == id)
    }

    fn node_reasons(plan: &super::IncrementalComputationPlanV0, id: &str) -> Vec<&'static str> {
        plan.nodes
            .iter()
            .find(|node| node.id == id)
            .map(|node| node.reasons.clone())
            .unwrap_or_default()
    }

    fn plan_from_database(
        input: &IncrementalGraphInputV0,
        previous: Option<&super::IncrementalSnapshotV0>,
    ) -> super::IncrementalComputationPlanV0 {
        plan_from_database_with_priority_inputs(input, previous, &[])
    }

    fn plan_from_database_with_priority_inputs(
        input: &IncrementalGraphInputV0,
        previous: Option<&super::IncrementalSnapshotV0>,
        priority_inputs: &[super::IncrementalEditDistancePriorityInputV0],
    ) -> super::IncrementalComputationPlanV0 {
        let mut database = OmenaIncrementalDatabaseV0::default();
        if let Some(previous) = previous {
            database.restore_snapshot(previous);
        }
        database
            .plan_and_upsert_graph_input_with_priority_inputs(input, priority_inputs)
            .incremental_plan
    }
}
