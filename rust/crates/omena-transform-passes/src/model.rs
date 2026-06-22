//! Public transform planning, execution, provenance, and context contracts.
//!
//! These data models are the stable JSON-facing boundary for Omena CSS transform
//! passes. Runtime modules own mutation execution, while this module keeps the
//! pass registry, execution summaries, semantic-removal witnesses, fuzz reports,
//! and cross-file transform context shapes serializable for `omena-query`,
//! bindings, CLI runners, and release gates.

use omena_abstract_value::AbstractCssValueV0;
use omena_incremental::{IncrementalComputationPlanV0, IncrementalSnapshotV0};
use omena_smt::CanonicalSmtInputV0;
use omena_transform_cst::{StableNodeKeyV0, TransformDagEdgeV0, TransformPassContractV0};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum TransformPassExecutionStatus {
    RegistryAndPlannerReady,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformPassRegistryEntryV0 {
    pub contract: TransformPassContractV0,
    pub module_family: &'static str,
    pub query_family: &'static str,
    pub execution_status: TransformPassExecutionStatus,
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
    pub planner_enforces_dag_edges: bool,
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
    pub requested_pass_ids: Vec<&'static str>,
    pub ordered_pass_ids: Vec<&'static str>,
    pub satisfied_dag_edge_count: usize,
    pub violated_dag_edge_count: usize,
    pub all_requested_registered: bool,
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformProvenanceDerivationForestV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub root_count: usize,
    pub node_count: usize,
    pub nodes: Vec<TransformProvenanceDerivationNodeV0>,
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
    pub closed_style_world: bool,
    pub drop_dark_mode_media_queries: bool,
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
    // (`scripts/check-rust-omena-scss-eval-external-differential.ts`, pinned dart-sass/lessc) over
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
