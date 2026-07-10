//! Public transform planning, execution, provenance, and context contracts.
//!
//! These data models are the stable JSON-facing boundary for Omena CSS transform
//! passes. Runtime modules own mutation execution, while this module keeps the
//! pass registry, execution summaries, semantic-removal witnesses, fuzz reports,
//! and cross-file transform context shapes serializable for `omena-query`,
//! bindings, CLI runners, and release gates.

use omena_abstract_value::{AbstractCssValueV0, FactPrecision};
use omena_cascade::SupportsTargetCapabilityV0;
use omena_cascade_proof::{
    CanonicalSmtInputV0, DischargeLedgerLookupStatusV0, DischargeLedgerLookupV0,
    DischargeLedgerVerdictV0,
};
use omena_evidence_graph::{
    EvidenceDemandEdgeV0, EvidenceGraphBuildErrorV0, EvidenceGraphV0, EvidenceNodeKeyV0,
    EvidenceNodeSeedV0, GuaranteeFamilyV0, GuaranteeKindV0, build_evidence_graph_from_edges_v0,
};
use omena_incremental::{IncrementalComputationPlanV0, IncrementalSnapshotV0};
use omena_transform_cst::{
    StableNodeKeyV0, TransformBuildProfileV0, TransformDagEdgeV0, TransformPassContractV0,
    TransformPassDescriptorV0, TransformPassKind,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

const TRANSFORM_PASS_OUTCOME_EVIDENCE_QUERY_V0: &str =
    "omena-transform-passes.transform-pass-execution-outcome";
const TRANSFORM_PROVENANCE_NODE_EVIDENCE_QUERY_V0: &str =
    "omena-transform-passes.provenance-derivation-node";
const TRANSFORM_EVIDENCE_EDGE_KIND_V0: &str = "transform-evidence";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum TransformPassExecutionStatus {
    RegistryAndPlannerReady,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum TransformPassDispatchKindV0 {
    TextLocalSliceRewrite,
    StructuralIrTransaction,
    ModuleEvaluationHandler,
    EmissionBoundary,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformPassRegistryEntryV0 {
    pub contract: TransformPassContractV0,
    pub descriptor: TransformPassDescriptorV0,
    pub module_family: &'static str,
    pub query_family: &'static str,
    pub dispatch_kind: TransformPassDispatchKindV0,
    pub execution_status: TransformPassExecutionStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformPassRegistryV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub entries: Vec<TransformPassRegistryEntryV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformPassesBoundarySummaryV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub registry_entries: Vec<TransformPassRegistryEntryV0>,
    pub dag_edges: Vec<TransformDagEdgeV0>,
    pub pass_count: usize,
    pub full_catalog_registered: bool,
    pub semantic_aware_pass_count: usize,
    pub cascade_aware_pass_count: usize,
    pub structural_pass_count: usize,
    pub text_local_pass_count: usize,
    pub module_evaluation_pass_count: usize,
    pub planner_enforces_dag_edges: bool,
    pub planner_uses_pass_descriptors: bool,
    pub ordinal_has_execution_semantics: bool,
    pub execution_runtime_ready: bool,
    pub incremental_execution_runtime_ready: bool,
    pub module_evaluation_native_output_marker: &'static str,
    pub module_evaluation_requires_native_product_output: bool,
    pub module_evaluation_requires_oracle_readiness: bool,
    pub module_evaluation_legacy_output_is_oracle_only: bool,
    pub module_evaluation_preserves_source_without_native_output: bool,
    pub implemented_mutation_pass_ids: Vec<&'static str>,
    pub next_surfaces: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformPassPlanV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub build_profile: TransformBuildProfileV0,
    pub requested_pass_ids: Vec<&'static str>,
    pub ordered_pass_ids: Vec<&'static str>,
    pub satisfied_dag_edge_count: usize,
    pub violated_dag_edge_count: usize,
    pub all_requested_registered: bool,
    pub conflicting_unordered_pass_pairs: Vec<TransformPlanPassConflictV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformPlanPassConflictV0 {
    pub pass_a: &'static str,
    pub pass_b: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformStructuralIrShadowFieldReportV0 {
    pub field: &'static str,
    pub string_path_values: Vec<String>,
    pub ir_path_values: Vec<String>,
    pub typed_path_values: Vec<String>,
    pub matches: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformStructuralIrShadowFixtureReportV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub fixture: String,
    pub pass_id: &'static str,
    pub dialect: &'static str,
    pub string_path_mutation_count: Option<usize>,
    pub ir_path_mutation_count: Option<usize>,
    pub typed_path_mutation_count: Option<usize>,
    pub ir_path_transaction_commit_count: Option<u64>,
    pub typed_payload_projections_consumed: usize,
    pub typed_payload_memo_hits: usize,
    pub fields: Vec<TransformStructuralIrShadowFieldReportV0>,
    pub all_fields_match: bool,
    pub all_typed_path_fields_match: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformStructuralIrShadowEquivalenceReportV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub fixture_count: usize,
    pub compared_pass_ids: Vec<&'static str>,
    pub compared_fields: Vec<&'static str>,
    pub reports: Vec<TransformStructuralIrShadowFixtureReportV0>,
    pub all_fields_match: bool,
    pub all_typed_path_fields_match: bool,
    pub typed_payload_projections_consumed: usize,
    pub typed_payload_memo_hits: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum TransformPassRuntimeStatus {
    Applied,
    NoChange,
    PlannedOnly,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformPassExecutionOutcomeV0 {
    pub pass_id: &'static str,
    pub status: TransformPassRuntimeStatus,
    pub input_byte_len: usize,
    pub output_byte_len: usize,
    pub mutation_count: usize,
    pub provenance_preserved: bool,
    pub detail: &'static str,
}

impl TransformPassExecutionOutcomeV0 {
    pub fn evidence_node_key(&self) -> EvidenceNodeKeyV0 {
        EvidenceNodeKeyV0::new(TRANSFORM_PASS_OUTCOME_EVIDENCE_QUERY_V0, self.pass_id)
    }

    pub fn evidence_node_seed(&self) -> EvidenceNodeSeedV0 {
        EvidenceNodeSeedV0::new(
            self.evidence_node_key(),
            vec![
                ["pass:", self.pass_id].concat(),
                ["detail:", self.detail].concat(),
                ["mutationCount:", self.mutation_count.to_string().as_str()].concat(),
                [
                    "provenancePreserved:",
                    self.provenance_preserved.to_string().as_str(),
                ]
                .concat(),
            ],
            GuaranteeKindV0::for_label_less_family(),
        )
    }

    pub fn evidence_demand_edge(&self) -> EvidenceDemandEdgeV0 {
        EvidenceDemandEdgeV0::new(
            TRANSFORM_PASS_OUTCOME_EVIDENCE_QUERY_V0,
            self.evidence_node_key(),
            TRANSFORM_EVIDENCE_EDGE_KIND_V0,
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum TransformEvaluationProfileV0 {
    Scss,
    Less,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum TransformPreconditionV0 {
    EvaluatorOutput {
        profile: TransformEvaluationProfileV0,
    },
    ResolvedImportReplacements,
    CssModulesComposesResolution,
    DesignTokenRoutes,
    SelectorIdentity,
    ClosedStyleWorldBundle,
    ClosedWorldBundle,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum TransformNoChangeReasonV0 {
    NoMutation,
    EmissionBoundary,
    ProfileNotApplicable {
        profile: TransformEvaluationProfileV0,
    },
    NoMatchingSelectorRewrite,
    DialectNotApplicable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum TransformBlockedReasonV0 {
    MissingPrecondition {
        precondition: TransformPreconditionV0,
    },
    PrecisionBelowFloor {
        required: FactPrecision,
        observed: FactPrecision,
    },
    DischargeMissing {
        lookup_status: Option<DischargeLedgerLookupStatusV0>,
        verdict: Option<DischargeLedgerVerdictV0>,
    },
    PassImplementation,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum TransformRejectionReasonV0 {
    IrTransaction { pass: TransformPassKind },
    SemanticPreservation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum TransformStructuralDecisionClassV0 {
    FactConsuming { required_precision: FactPrecision },
    StaticExact,
    ObligationDischarge,
    NonRemovalRewrite,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformStructuralDecisionPolicyV0 {
    pub pass: TransformPassKind,
    pub class: TransformStructuralDecisionClassV0,
    pub reason: &'static str,
}

impl TransformStructuralDecisionPolicyV0 {
    pub const fn new(
        pass: TransformPassKind,
        class: TransformStructuralDecisionClassV0,
        reason: &'static str,
    ) -> Self {
        Self {
            pass,
            class,
            reason,
        }
    }

    pub const fn required_precision(self) -> Option<FactPrecision> {
        match self.class {
            TransformStructuralDecisionClassV0::FactConsuming { required_precision } => {
                Some(required_precision)
            }
            TransformStructuralDecisionClassV0::StaticExact
            | TransformStructuralDecisionClassV0::ObligationDischarge
            | TransformStructuralDecisionClassV0::NonRemovalRewrite => None,
        }
    }
}

pub const TRANSFORM_STRUCTURAL_DECISION_POLICIES_V0: &[TransformStructuralDecisionPolicyV0] = &[
    TransformStructuralDecisionPolicyV0::new(
        TransformPassKind::ImportInline,
        TransformStructuralDecisionClassV0::NonRemovalRewrite,
        "materializes explicitly resolved imports without reachability pruning",
    ),
    TransformStructuralDecisionPolicyV0::new(
        TransformPassKind::ResolveCssModulesComposes,
        TransformStructuralDecisionClassV0::NonRemovalRewrite,
        "materializes explicit CSS Modules composition resolution",
    ),
    TransformStructuralDecisionPolicyV0::new(
        TransformPassKind::DesignTokenRouting,
        TransformStructuralDecisionClassV0::NonRemovalRewrite,
        "rewrites values through explicit design-token routes",
    ),
    TransformStructuralDecisionPolicyV0::new(
        TransformPassKind::HashCssModuleClassNames,
        TransformStructuralDecisionClassV0::NonRemovalRewrite,
        "rewrites selectors through an explicit identity map",
    ),
    TransformStructuralDecisionPolicyV0::new(
        TransformPassKind::RuleDeduplication,
        TransformStructuralDecisionClassV0::StaticExact,
        "removes only statically equivalent duplicate rules",
    ),
    TransformStructuralDecisionPolicyV0::new(
        TransformPassKind::RuleMerging,
        TransformStructuralDecisionClassV0::NonRemovalRewrite,
        "combines adjacent declarations without reachability pruning",
    ),
    TransformStructuralDecisionPolicyV0::new(
        TransformPassKind::SelectorMerging,
        TransformStructuralDecisionClassV0::NonRemovalRewrite,
        "combines equivalent selector blocks without reachability pruning",
    ),
    TransformStructuralDecisionPolicyV0::new(
        TransformPassKind::NestingUnwrap,
        TransformStructuralDecisionClassV0::NonRemovalRewrite,
        "expands nested selectors without reachability pruning",
    ),
    TransformStructuralDecisionPolicyV0::new(
        TransformPassKind::ScopeFlatten,
        TransformStructuralDecisionClassV0::ObligationDischarge,
        "requires accepted scope-flatten obligations",
    ),
    TransformStructuralDecisionPolicyV0::new(
        TransformPassKind::LayerFlatten,
        TransformStructuralDecisionClassV0::ObligationDischarge,
        "requires accepted layer-flatten obligations",
    ),
    TransformStructuralDecisionPolicyV0::new(
        TransformPassKind::SupportsStaticEval,
        TransformStructuralDecisionClassV0::StaticExact,
        "removes only statically decided supports branches",
    ),
    TransformStructuralDecisionPolicyV0::new(
        TransformPassKind::MediaStaticEval,
        TransformStructuralDecisionClassV0::StaticExact,
        "removes only statically unsatisfiable media branches",
    ),
    TransformStructuralDecisionPolicyV0::new(
        TransformPassKind::ContainerStaticEval,
        TransformStructuralDecisionClassV0::StaticExact,
        "removes only statically unsatisfiable container branches",
    ),
    TransformStructuralDecisionPolicyV0::new(
        TransformPassKind::NativeCssStaticEval,
        TransformStructuralDecisionClassV0::StaticExact,
        "folds only statically evaluable native CSS expressions",
    ),
    TransformStructuralDecisionPolicyV0::new(
        TransformPassKind::DeadMediaBranchRemoval,
        TransformStructuralDecisionClassV0::StaticExact,
        "removes only media branches selected by explicit static policy",
    ),
    TransformStructuralDecisionPolicyV0::new(
        TransformPassKind::DeadSupportsBranchRemoval,
        TransformStructuralDecisionClassV0::StaticExact,
        "removes only statically decided supports branches",
    ),
    TransformStructuralDecisionPolicyV0::new(
        TransformPassKind::TreeShakeClass,
        TransformStructuralDecisionClassV0::FactConsuming {
            required_precision: FactPrecision::Conservative,
        },
        "removes class rules only from a closed-world reachability over-approximation",
    ),
    TransformStructuralDecisionPolicyV0::new(
        TransformPassKind::TreeShakeKeyframes,
        TransformStructuralDecisionClassV0::FactConsuming {
            required_precision: FactPrecision::Conservative,
        },
        "removes keyframes only from a closed-world reachability over-approximation",
    ),
    TransformStructuralDecisionPolicyV0::new(
        TransformPassKind::TreeShakeValue,
        TransformStructuralDecisionClassV0::FactConsuming {
            required_precision: FactPrecision::Conservative,
        },
        "removes CSS Modules values only from a closed-world reachability over-approximation",
    ),
    TransformStructuralDecisionPolicyV0::new(
        TransformPassKind::TreeShakeCustomProperty,
        TransformStructuralDecisionClassV0::FactConsuming {
            required_precision: FactPrecision::Conservative,
        },
        "removes custom properties only from a closed-world reachability over-approximation",
    ),
    TransformStructuralDecisionPolicyV0::new(
        TransformPassKind::EmptyRuleRemoval,
        TransformStructuralDecisionClassV0::StaticExact,
        "removes only structurally empty rules",
    ),
];

pub fn transform_structural_decision_policy(
    pass: TransformPassKind,
) -> Option<&'static TransformStructuralDecisionPolicyV0> {
    TRANSFORM_STRUCTURAL_DECISION_POLICIES_V0
        .iter()
        .find(|policy| policy.pass == pass)
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum RollbackScopeV0 {
    RejectPreservedInput,
    CommittedIrrecoverable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RollbackReceiptV0 {
    pub pass_id: &'static str,
    pub attempted_mutation_count: Option<usize>,
    pub input_content_signature: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_preserved_content_signature: Option<String>,
    pub restorable: RollbackScopeV0,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformDischargeEvidenceV0 {
    pub evidence_node_key: EvidenceNodeKeyV0,
    pub guarantee_family: GuaranteeFamilyV0,
    pub ledger_cell_key: String,
    pub boundedness_kind: String,
}

impl RollbackReceiptV0 {
    pub fn preserves_rejected_input(&self) -> bool {
        self.restorable == RollbackScopeV0::RejectPreservedInput
            && self.output_preserved_content_signature.as_deref()
                == Some(self.input_content_signature.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum TransformDecision {
    Applied {
        outcome: TransformPassExecutionOutcomeV0,
        rollback_receipt: RollbackReceiptV0,
        #[serde(skip_serializing_if = "Vec::is_empty")]
        discharge_evidence: Vec<TransformDischargeEvidenceV0>,
    },
    NoChange {
        reason: TransformNoChangeReasonV0,
        outcome: TransformPassExecutionOutcomeV0,
    },
    Blocked {
        reason: TransformBlockedReasonV0,
        outcome: TransformPassExecutionOutcomeV0,
    },
    Rejected {
        reason: TransformRejectionReasonV0,
        outcome: TransformPassExecutionOutcomeV0,
        rollback_receipt: RollbackReceiptV0,
    },
}

impl TransformDecision {
    pub fn compatibility_outcome(&self) -> &TransformPassExecutionOutcomeV0 {
        match self {
            Self::Applied { outcome, .. }
            | Self::NoChange { outcome, .. }
            | Self::Blocked { outcome, .. }
            | Self::Rejected { outcome, .. } => outcome,
        }
    }

    pub fn into_compatibility_outcome(self) -> TransformPassExecutionOutcomeV0 {
        match self {
            Self::Applied { outcome, .. }
            | Self::NoChange { outcome, .. }
            | Self::Blocked { outcome, .. }
            | Self::Rejected { outcome, .. } => outcome,
        }
    }

    pub fn rollback_receipt(&self) -> Option<&RollbackReceiptV0> {
        match self {
            Self::Applied {
                rollback_receipt, ..
            }
            | Self::Rejected {
                rollback_receipt, ..
            } => Some(rollback_receipt),
            Self::NoChange { .. } | Self::Blocked { .. } => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformProvenanceDerivationForestV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub root_count: usize,
    pub node_count: usize,
    pub nodes: Vec<TransformProvenanceDerivationNodeV0>,
}

impl TransformProvenanceDerivationForestV0 {
    pub fn evidence_graph(&self) -> Result<EvidenceGraphV0, EvidenceGraphBuildErrorV0> {
        build_evidence_graph_from_edges_v0(
            self.nodes
                .iter()
                .map(TransformProvenanceDerivationNodeV0::evidence_node_seed),
            self.nodes
                .iter()
                .map(TransformProvenanceDerivationNodeV0::evidence_demand_edge),
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformProvenanceDerivationNodeV0 {
    pub node_index: usize,
    pub parent_index: Option<usize>,
    pub pass_id: &'static str,
    pub status: TransformPassRuntimeStatus,
    pub input_byte_len: usize,
    pub output_byte_len: usize,
    pub source_span_start: usize,
    pub source_span_end: usize,
    pub generated_span_start: usize,
    pub generated_span_end: usize,
    pub mutation_spans: Vec<TransformProvenanceMutationSpanV0>,
    pub mutation_count: usize,
    pub provenance_preserved: bool,
    pub detail: &'static str,
}

impl TransformProvenanceDerivationNodeV0 {
    pub fn evidence_node_key(&self) -> EvidenceNodeKeyV0 {
        EvidenceNodeKeyV0::new(
            TRANSFORM_PROVENANCE_NODE_EVIDENCE_QUERY_V0,
            format!("{}#{}", self.pass_id, self.node_index),
        )
    }

    pub fn evidence_node_seed(&self) -> EvidenceNodeSeedV0 {
        EvidenceNodeSeedV0::new(
            self.evidence_node_key(),
            vec![
                ["pass:", self.pass_id].concat(),
                ["detail:", self.detail].concat(),
                ["mutationCount:", self.mutation_count.to_string().as_str()].concat(),
                [
                    "provenancePreserved:",
                    self.provenance_preserved.to_string().as_str(),
                ]
                .concat(),
            ],
            GuaranteeKindV0::for_label_less_family(),
        )
    }

    pub fn evidence_demand_edge(&self) -> EvidenceDemandEdgeV0 {
        EvidenceDemandEdgeV0::new(
            TRANSFORM_PROVENANCE_NODE_EVIDENCE_QUERY_V0,
            self.evidence_node_key(),
            TRANSFORM_EVIDENCE_EDGE_KIND_V0,
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformProvenanceMutationSpanV0 {
    pub source_span_start: usize,
    pub source_span_end: usize,
    pub generated_span_start: usize,
    pub generated_span_end: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_key: Option<StableNodeKeyV0>,
}

/// Counts incremental lex-splice outcomes inside a transform execution.
///
/// A fallback is conservative: the cache declines to reuse token ranges and the
/// next consumer re-lexes the generated source normally.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformLexCacheSpliceTelemetryV0 {
    /// Number of generated token streams inserted through bounded splicing.
    pub splice_hit_count: u64,
    /// Number of active-cache attempts that intentionally fell back to full re-lex.
    pub full_relex_fallback_count: u64,
    /// Fallbacks caused by invalid or non-projectable mutation windows.
    pub window_derivation_fallback_count: u64,
    /// Fallbacks where the safe restart window covers the full generated output.
    pub full_output_window_fallback_count: u64,
    /// Fallbacks caused by token offset arithmetic or projection failure.
    pub token_offset_fallback_count: u64,
}

/// Counts structural IR transaction outcomes that matter for String-currency
/// retirement.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformStructuralIrTransactionTelemetryV0 {
    pub transaction_commit_count: u64,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformSemanticPreservationTelemetryV0 {
    pub observed_pass_count: u64,
    pub preserved_pass_count: u64,
    pub blocked_pass_count: u64,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformDischargeLedgerTelemetryV0 {
    pub lookup_count: u64,
    pub matched_lookup_count: u64,
    pub accepted_stamp_count: u64,
    pub blocked_lookup_count: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformExecutionSummaryV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub input_byte_len: usize,
    pub output_byte_len: usize,
    pub requested_pass_ids: Vec<&'static str>,
    pub ordered_pass_ids: Vec<&'static str>,
    pub executed_pass_ids: Vec<&'static str>,
    pub planned_only_pass_ids: Vec<&'static str>,
    pub mutation_count: usize,
    pub provenance_preserved: bool,
    pub output_css: String,
    pub css_module_evaluation: Option<TransformModuleEvaluationV0>,
    pub css_import_inlines: Vec<TransformImportInlineV0>,
    pub css_module_composes_exports: Vec<TransformCssModuleComposesResolutionV0>,
    pub design_token_routes: Vec<TransformDesignTokenRouteV0>,
    pub semantic_removals: Vec<TransformSemanticRemovalV0>,
    pub cascade_proof_obligations: TransformCascadeProofObligationReportV0,
    pub provenance_derivation_forest: TransformProvenanceDerivationForestV0,
    pub structural_ir_transaction_telemetry: TransformStructuralIrTransactionTelemetryV0,
    pub semantic_preservation_telemetry: TransformSemanticPreservationTelemetryV0,
    pub discharge_ledger_telemetry: TransformDischargeLedgerTelemetryV0,
    pub decisions: Vec<TransformDecision>,
    pub outcomes: Vec<TransformPassExecutionOutcomeV0>,
    pub pass_plan: TransformPassPlanV0,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformCascadeProofObligationReportV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub obligation_count: usize,
    pub accepted_count: usize,
    pub blocked_count: usize,
    pub checked_pass_ids: Vec<&'static str>,
    pub obligations: Vec<TransformCascadeProofObligationV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformCascadeProofObligationV0 {
    pub pass_id: &'static str,
    pub proof_product: &'static str,
    pub accepted: bool,
    pub blocked_reason: Option<String>,
    pub provenance_preserved: bool,
    pub cascade_safe_witness: String,
    pub source_span_start: Option<usize>,
    pub source_span_end: Option<usize>,
    pub checked_obligations: Vec<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub canonical_smt_input: Option<CanonicalSmtInputV0>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub discharge_ledger_lookup: Option<DischargeLedgerLookupV0>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub discharge_evidence: Option<TransformDischargeEvidenceV0>,
    pub proof_payload: Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformSemanticRemovalV0 {
    pub pass_id: &'static str,
    pub symbol_kind: &'static str,
    pub name: String,
    pub source_span_start: usize,
    pub source_span_end: usize,
    pub reason: &'static str,
    pub certainty: &'static str,
    pub derivation_steps: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TransformSemanticRemovalCandidate {
    pub(crate) symbol_kind: &'static str,
    pub(crate) name: String,
    pub(crate) source_span_start: usize,
    pub(crate) source_span_end: usize,
    pub(crate) reason: &'static str,
}

impl TransformSemanticRemovalCandidate {
    pub(crate) fn into_public(self, pass_id: &'static str) -> TransformSemanticRemovalV0 {
        TransformSemanticRemovalV0 {
            pass_id,
            symbol_kind: self.symbol_kind,
            name: self.name,
            source_span_start: self.source_span_start,
            source_span_end: self.source_span_end,
            reason: self.reason,
            certainty: "high",
            derivation_steps: vec![
                "closedStyleWorld",
                "reachableRootSetComputed",
                "symbolNotMarkedReachable",
                "sourceRangeRemoved",
            ],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformIncrementalExecutionSummaryV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub incremental_engine: &'static str,
    pub query_model: &'static str,
    pub reuse_policy: &'static str,
    pub reused_previous_execution: bool,
    pub incremental_plan: IncrementalComputationPlanV0,
    pub next_snapshot: IncrementalSnapshotV0,
    pub execution: TransformExecutionSummaryV0,
    pub ready_surfaces: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformCascadeSafetyFuzzCaseV0 {
    pub seed: u64,
    pub pass_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformCascadeSafetyFuzzResultV0 {
    pub seed: u64,
    pub pass_count: usize,
    pub requested_pass_ids: Vec<&'static str>,
    pub executed_pass_ids: Vec<&'static str>,
    pub output_byte_len: usize,
    pub output_token_count: usize,
    pub output_error_count: usize,
    pub provenance_node_count: usize,
    pub passed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformFuzzSeedReportV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub case_count: usize,
    pub passed_count: usize,
    pub failed_count: usize,
    pub results: Vec<TransformCascadeSafetyFuzzResultV0>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(default, rename_all = "camelCase")]
pub struct TransformExecutionContextV0 {
    pub drop_dark_mode_media_queries: bool,
    pub supports_target_capability: Option<SupportsTargetCapabilityV0>,
    pub vendor_prefix_policy: Option<TransformVendorPrefixPolicyV0>,
    pub reachable_class_names: Vec<String>,
    pub reachable_keyframe_names: Vec<String>,
    pub reachable_value_names: Vec<String>,
    pub reachable_custom_property_names: Vec<String>,
    pub scss_module_evaluation: Option<TransformModuleEvaluationV0>,
    pub less_module_evaluation: Option<TransformModuleEvaluationV0>,
    pub import_inlines: Vec<TransformImportInlineV0>,
    pub class_name_rewrites: Vec<TransformClassNameRewriteV0>,
    pub css_module_composes_resolutions: Vec<TransformCssModuleComposesResolutionV0>,
    pub css_module_value_resolutions: Vec<TransformCssModuleValueResolutionV0>,
    pub design_token_routes: Vec<TransformDesignTokenRouteV0>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformVendorPrefixPolicyV0 {
    pub webkit: bool,
    pub moz: bool,
    pub ms: bool,
}

impl TransformVendorPrefixPolicyV0 {
    pub const fn none() -> Self {
        Self {
            webkit: false,
            moz: false,
            ms: false,
        }
    }

    pub const fn conservative() -> Self {
        Self {
            webkit: true,
            moz: true,
            ms: true,
        }
    }

    pub const fn is_empty(self) -> bool {
        !(self.webkit || self.moz || self.ms)
    }

    pub fn allows_prefix(self, prefixed_name: &str) -> bool {
        if prefixed_name.starts_with("-webkit-") {
            return self.webkit;
        }
        if prefixed_name.starts_with("-moz-") {
            return self.moz;
        }
        if prefixed_name.starts_with("-ms-") {
            return self.ms;
        }
        true
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformModuleEvaluationV0 {
    pub evaluator: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub product_output_source: Option<String>,
    pub evaluated_css: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub native_edit_output: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub native_replacements: Vec<TransformModuleEvaluationNativeReplacementV0>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub native_edits: Vec<TransformModuleEvaluationNativeEditV0>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub oracle: Option<TransformModuleEvaluationOracleV0>,
}

impl TransformModuleEvaluationV0 {
    pub fn declares_native_product_output(&self) -> bool {
        self.product_output_source
            .as_deref()
            .is_some_and(|source| source == "nativeEditOutput")
    }

    // HONESTY NOTE: `divergence_count == 0` is a value-WELL-FORMEDNESS self-check on the
    // native-edit output (every native-emitted declaration value canonically round-trips), NOT a
    // differential against an external SCSS/Less compiler. So this gate means "native output is
    // self-consistent and value-preserving", NOT "native agrees with dart-sass/lessc". External
    // agreement is witnessed separately by the `externalDifferential` gate
    // (`scripts/check-rust-omena-diff-test-external-corpus-differential.ts`, pinned dart-sass/lessc) over
    // its covered fixture slices only; this self-check stays the cheap inner oracle for every
    // evaluated candidate, and the production rail remains a self-comparison.
    pub fn oracle_allows_native_product_output(&self) -> bool {
        self.oracle.as_ref().is_some_and(|oracle| {
            oracle.mode == "oracleOnly"
                && oracle.divergence_count == 0
                && oracle.all_legacy_declaration_values_preserved
        })
    }

    pub fn may_consume_native_product_output(&self) -> bool {
        self.declares_native_product_output() && self.oracle_allows_native_product_output()
    }

    // NOTE: the "retained oracle" here is the retained product-output string (`evaluated_css`),
    // which in the production rail is itself native-derived — so this is a byte-equality
    // self-consistency check between two native-derived strings, not a comparison to an
    // independent external evaluator.
    pub fn native_output_matches_retained_oracle(&self, native_output: &str) -> bool {
        self.oracle
            .as_ref()
            .is_some_and(|_| native_output == self.evaluated_css)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformModuleEvaluationNativeReplacementV0 {
    pub name: String,
    pub start: usize,
    pub end: usize,
    pub text: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rendered_value: Option<String>,
    pub abstract_value: AbstractCssValueV0,
    pub abstract_value_kind: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformModuleEvaluationNativeEditV0 {
    pub start: usize,
    pub end: usize,
    pub replacement: String,
    pub edit_kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub abstract_value: Option<AbstractCssValueV0>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub abstract_value_kind: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(default, rename_all = "camelCase")]
pub struct TransformModuleEvaluationOracleV0 {
    pub mode: String,
    pub product_output_source: String,
    pub legacy_declaration_value_count: usize,
    pub abstract_value_count: usize,
    pub exact_value_count: usize,
    pub raw_value_count: usize,
    pub bottom_value_count: usize,
    pub top_value_count: usize,
    pub divergence_count: usize,
    pub all_legacy_declaration_values_preserved: bool,
    pub native_replacement_count: usize,
    pub native_replacement_legacy_reflection_count: usize,
    pub native_replacement_legacy_unreflected_count: usize,
    pub native_value_reference_count: usize,
    pub native_resolved_value_count: usize,
    pub native_raw_value_count: usize,
    pub native_top_value_count: usize,
    pub native_cycle_count: usize,
    pub native_fuel_exhausted_count: usize,
    pub native_unresolved_reference_count: usize,
    pub native_unsupported_dynamic_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformImportInlineV0 {
    pub import_source: String,
    pub replacement_css: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransformLessInlineLiteralPlaceholderV0 {
    pub placeholder: String,
    pub literal_css: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformClassNameRewriteV0 {
    pub original_name: String,
    pub rewritten_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformCssModuleComposesResolutionV0 {
    pub local_class_name: String,
    pub exported_class_names: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformCssModuleValueResolutionV0 {
    pub local_name: String,
    pub resolved_value: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformDesignTokenRouteV0 {
    pub token_name: String,
    pub routed_value: String,
}

#[cfg(test)]
mod evidence_graph_tests {
    use super::*;

    #[test]
    fn transform_outcome_evidence_graph_preserves_public_shape() -> Result<(), serde_json::Error> {
        let outcome = TransformPassExecutionOutcomeV0 {
            pass_id: "number-compression",
            status: TransformPassRuntimeStatus::Applied,
            input_byte_len: 32,
            output_byte_len: 28,
            mutation_count: 1,
            provenance_preserved: true,
            detail: "fixture pass",
        };

        let before = serde_json::to_value(&outcome)?;
        let node = outcome.evidence_node_seed();
        let graph = build_evidence_graph_from_edges_v0([node], [outcome.evidence_demand_edge()])
            .map_err(|_| serde::ser::Error::custom("outcome edge must target its node"))?;
        let after = serde_json::to_value(&outcome)?;

        assert_eq!(before, after);
        assert_eq!(graph.nodes.len(), 1);
        assert_eq!(graph.nodes[0].key.input_identity, "number-compression");
        assert_eq!(graph.nodes[0].guarantee, GuaranteeKindV0::Floor);
        assert!(
            graph.nodes[0]
                .provenance
                .iter()
                .any(|item| item == "mutationCount:1")
        );
        Ok(())
    }

    #[test]
    fn transform_derivation_forest_evidence_graph_preserves_public_shape()
    -> Result<(), serde_json::Error> {
        let forest = TransformProvenanceDerivationForestV0 {
            schema_version: "0",
            product: "omena-transform-passes.provenance-derivation-forest",
            root_count: 1,
            node_count: 1,
            nodes: vec![TransformProvenanceDerivationNodeV0 {
                node_index: 0,
                parent_index: None,
                pass_id: "comment-strip",
                status: TransformPassRuntimeStatus::Applied,
                input_byte_len: 48,
                output_byte_len: 36,
                source_span_start: 0,
                source_span_end: 12,
                generated_span_start: 0,
                generated_span_end: 0,
                mutation_spans: Vec::new(),
                mutation_count: 1,
                provenance_preserved: true,
                detail: "fixture derivation",
            }],
        };

        let before = serde_json::to_value(&forest)?;
        let graph = forest
            .evidence_graph()
            .map_err(|_| serde::ser::Error::custom("forest edge must target its node"))?;
        let after = serde_json::to_value(&forest)?;

        assert_eq!(before, after);
        assert_eq!(graph.nodes.len(), 1);
        assert_eq!(graph.nodes[0].key.input_identity, "comment-strip#0");
        assert_eq!(graph.nodes[0].guarantee, GuaranteeKindV0::Floor);
        Ok(())
    }
}
