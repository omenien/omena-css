use super::*;
use omena_evidence_graph::{
    EvidenceAnalysisPrecisionV0, EvidenceDemandEdgeV0, EvidenceNodeKeyV0, EvidenceNodeSeedV0,
    GuaranteeKindV0, build_evidence_graph_from_edges_v0,
};
use omena_query_transform_runner::normalize_omena_transform_bundle_path;
use omena_sif::OmenaSifV1;
use omena_syntax::ident::{
    AuthoredPropertyTextV0, CanonicalClassKeyV0, CanonicalCustomPropertyNameV0,
    CanonicalPropertyKeyV0, CanonicalStandardPropertyNameV0, PropertyNameV0,
};
use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};

mod runtime_state_serialization;
#[cfg(test)]
pub(crate) use runtime_state_serialization::runtime_state_result_certainty_labels;
pub(crate) use runtime_state_serialization::runtime_state_unknown_activation_declaration_id;

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DesignSystemMinimumDescriptionV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub model_bits: f64,
    pub residual_bits: f64,
    pub total_bits: f64,
    pub unit: &'static str,
    pub model_class: ModelClassV0,
    pub rule_count: usize,
    pub observation_count: usize,
    pub canonical_form_present: bool,
    pub cascade_proof_obligation_count: usize,
    pub sass_namespace_partition: SassNamespaceBitsV0,
    pub generated_at_iso: &'static str,
    pub source_pin: SourcePinV0,
    pub weights_calibration_pin: &'static str,
    pub weights_version: &'static str,
    pub semiring_instance: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ModelClassV0 {
    TwoPartUniform,
    TwoPartMultinomial,
    Nml,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SassNamespaceBitsV0 {
    pub namespace_count: usize,
    pub partition_count: usize,
    pub deterministic_partition: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourcePinV0 {
    pub source_uri: String,
    pub source_hash: String,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CanonicalFormV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub pass_id: &'static str,
    pub before: String,
    pub canonical_after: String,
    pub fallback_after: String,
    pub canonical_matches_fallback: bool,
    pub mdl_bits: f64,
    pub ast_size_bits: f64,
    pub bits_saved_vs_fallback: f64,
    pub unit: &'static str,
    pub iteration_count: usize,
    pub eclass_count: usize,
    pub enode_count: usize,
    pub cascade_safe_witness: &'static str,
    pub egg_analysis_witness: &'static str,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryBoundarySummaryV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub query_engine_name: &'static str,
    pub schema_version_policy: OmenaQuerySchemaVersionPolicyV0,
    pub input_version: String,
    pub abstract_value_domain: AbstractValueDomainSummaryV0,
    pub selected_query_adapter_capabilities: SelectedQueryAdapterCapabilitiesV0,
    pub delegated_fragment_products: Vec<&'static str>,
    pub expression_semantics_query_count: usize,
    pub source_resolution_query_count: usize,
    pub selector_usage_query_count: usize,
    pub total_query_count: usize,
    pub ready_surfaces: Vec<&'static str>,
    pub style_completion_consumer_decisions: Vec<OmenaQueryStyleCompletionConsumerDecisionV0>,
    pub cme_coupled_surfaces: Vec<&'static str>,
    pub next_decoupling_targets: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryStyleCompletionConsumerDecisionV0 {
    pub consumer: &'static str,
    pub surface: &'static str,
    pub decision: &'static str,
    pub rationale: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQuerySassModuleConformanceReportV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub claim_level: &'static str,
    pub theorem_claimed: bool,
    pub normative_source: &'static str,
    pub modeled_count: usize,
    pub gap_count: usize,
    pub decided_out_count: usize,
    pub policy_count: usize,
    pub rows: Vec<OmenaQuerySassModuleConformanceRowV0>,
    pub ready_surfaces: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQuerySassModuleConformanceRowV0 {
    pub key: &'static str,
    pub category: &'static str,
    pub status: &'static str,
    pub normative_anchor: &'static str,
    pub implementation: &'static str,
    pub witness: &'static str,
    pub decision: &'static str,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryEvaluationRuntimeSummaryV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub input_version: String,
    pub selected_query_adapter_capabilities: SelectedQueryAdapterCapabilitiesV0,
    pub runtime_products: Vec<&'static str>,
    pub source_resolution_expression_count: usize,
    pub source_resolution_unresolved_expression_count: usize,
    pub expression_domain_revision: u64,
    pub expression_domain_graph_count: usize,
    pub expression_domain_dirty_graph_count: usize,
    pub expression_domain_reused_graph_count: usize,
    pub style_document_summary_source: &'static str,
    pub ready_surfaces: Vec<&'static str>,
    pub retired_couplings: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SelectedQueryAdapterCapabilitiesV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub default_candidate_backend: &'static str,
    pub schema_version_policy: OmenaQuerySchemaVersionPolicyV0,
    pub schema_version_checks: Vec<OmenaQuerySchemaVersionCheckV0>,
    pub backend_kinds: Vec<SelectedQueryBackendCapabilityV0>,
    pub runner_commands: Vec<SelectedQueryRunnerCommandV0>,
    pub expression_semantics_payload_contracts: Vec<&'static str>,
    pub required_input_contracts: Vec<&'static str>,
    pub adapter_readiness: Vec<&'static str>,
    pub routing_status: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQuerySchemaVersionPolicyV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub current_version: &'static str,
    pub current_version_label: &'static str,
    pub accepted_versions: Vec<&'static str>,
    pub deprecated_versions: Vec<&'static str>,
    pub rejected_version_policy: &'static str,
    pub missing_version_policy: &'static str,
    pub migration_policy: Vec<&'static str>,
    pub compatibility_gate: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQuerySchemaVersionCheckV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub requested_version: Option<String>,
    pub current_version: &'static str,
    pub accepted: bool,
    pub status: &'static str,
    pub migration_action: &'static str,
    pub reason: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SelectedQueryBackendCapabilityV0 {
    pub backend_kind: &'static str,
    pub source_resolution: bool,
    pub expression_semantics: bool,
    pub selector_usage: bool,
    pub style_semantic_graph: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SelectedQueryRunnerCommandV0 {
    pub surface: &'static str,
    pub command: &'static str,
    pub input_contract: &'static str,
    pub output_product: &'static str,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryStyleSemanticGraphBatchOutputV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub cross_file_summary: OmenaQueryCrossFileSummaryV0,
    pub css_modules_resolution: OmenaQueryCssModulesCrossFileResolutionV0,
    pub sass_module_resolution: OmenaQuerySassModuleCrossFileResolutionV0,
    pub graphs: Vec<OmenaQueryStyleSemanticGraphBatchEntryV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryCategoricalDesignSystemCrossProjectSummaryV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub claim_scope: &'static str,
    pub source_product: &'static str,
    pub theory_product: &'static str,
    pub project_count: usize,
    pub product_path_evidence_ready: bool,
    pub models: Vec<OmenaQueryCategoricalDesignSystemModelV0>,
    pub invariant_summary: OmenaQueryCategoricalDesignSystemInvariantSummaryV0,
    pub deferred_residuals: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryM4AxisCReadinessSummaryV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub status: &'static str,
    pub required_edge_kind_count: usize,
    pub required_edge_kind_counts: Vec<OmenaQueryCrossFileSummaryEdgeKindCountV0>,
    pub workspace_edge_count: usize,
    pub issue_63_provenance_round_trip_ready: bool,
    pub issue_65_summary_edge_equivalence_ready: bool,
    pub summary_hash_invalidation_ready: bool,
    pub summary_hash_samples: OmenaQueryM4AxisCSummaryHashSamplesV0,
    pub checked_surfaces: Vec<&'static str>,
    pub next_priorities: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryM4AxisCSummaryHashSamplesV0 {
    pub baseline: String,
    pub source_selector_change: String,
    pub style_edge_change: String,
    pub package_manifest_change: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryStyleSemanticGraphBatchEntryV0 {
    pub style_path: String,
    pub graph: Option<StyleSemanticGraphSummaryV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryCssModulesCrossFileResolutionV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub status: &'static str,
    pub resolution_scope: &'static str,
    pub style_count: usize,
    pub import_edge_count: usize,
    pub resolved_import_edge_count: usize,
    pub unresolved_import_edge_count: usize,
    pub matched_name_count: usize,
    pub edges: Vec<OmenaQueryCssModulesImportEdgeResolutionV0>,
    pub composes_closure_edge_count: usize,
    pub value_closure_edge_count: usize,
    pub icss_closure_edge_count: usize,
    pub composes_cycle_count: usize,
    pub value_cycle_count: usize,
    pub icss_cycle_count: usize,
    pub composes_closure_edges: Vec<OmenaQueryCssModulesComposesClosureEdgeV0>,
    pub value_closure_edges: Vec<OmenaQueryCssModulesValueClosureEdgeV0>,
    pub icss_closure_edges: Vec<OmenaQueryCssModulesIcssClosureEdgeV0>,
    pub cycles: Vec<OmenaQueryCssModulesCycleV0>,
    pub capabilities: OmenaQueryCssModulesCrossFileResolutionCapabilitiesV0,
    pub next_priorities: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryCssModulesImportEdgeResolutionV0 {
    pub from_style_path: String,
    pub import_kind: &'static str,
    pub source: String,
    pub resolved_style_path: Option<String>,
    pub status: &'static str,
    pub import_graph_distance: Option<usize>,
    pub import_graph_order: Option<usize>,
    pub imported_names: Vec<String>,
    pub exported_names: Vec<String>,
    pub matched_names: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryCssModulesComposesClosureEdgeV0 {
    pub from_style_path: String,
    pub owner_selector_name: String,
    pub target_style_path: String,
    pub target_selector_name: String,
    pub depth: usize,
    pub path: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryCssModulesValueClosureEdgeV0 {
    pub from_style_path: String,
    pub value_name: String,
    pub target_style_path: String,
    pub target_value_name: String,
    pub depth: usize,
    pub path: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryCssModulesIcssClosureEdgeV0 {
    pub from_style_path: String,
    pub name: String,
    pub target_style_path: String,
    pub target_name: String,
    pub depth: usize,
    pub path: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryCssModulesCycleV0 {
    pub kind: &'static str,
    pub path: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryCssModulesCrossFileResolutionCapabilitiesV0 {
    pub semantic_layer_owned: bool,
    pub import_source_resolution_ready: bool,
    pub cross_file_resolution_ready: bool,
    pub composes_closure_ready: bool,
    pub composes_name_match_ready: bool,
    pub value_name_match_ready: bool,
    pub icss_name_match_ready: bool,
    pub transitive_closure_ready: bool,
    pub value_graph_closure_ready: bool,
    pub icss_export_import_closure_ready: bool,
    pub cycle_detection_ready: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQuerySassModuleCrossFileResolutionV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub status: &'static str,
    pub resolution_scope: &'static str,
    pub style_count: usize,
    pub module_edge_count: usize,
    pub resolved_module_edge_count: usize,
    pub unresolved_module_edge_count: usize,
    pub external_module_edge_count: usize,
    pub symlink_chain_edge_count: usize,
    pub symlink_chain_link_count: usize,
    pub configured_module_instance_count: usize,
    pub edges: Vec<OmenaQuerySassModuleEdgeResolutionV0>,
    pub graph_closure_edge_count: usize,
    pub cycle_count: usize,
    pub visibility_filter_count: usize,
    pub graph_closure_edges: Vec<OmenaQuerySassModuleGraphClosureEdgeV0>,
    pub cycles: Vec<OmenaQuerySassModuleCycleV0>,
    pub capabilities: OmenaQuerySassModuleCrossFileResolutionCapabilitiesV0,
    pub next_priorities: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQuerySassModuleEdgeResolutionV0 {
    pub from_style_path: String,
    pub edge_kind: &'static str,
    pub source: String,
    pub rule_ordinal: usize,
    pub namespace_kind: Option<&'static str>,
    pub namespace: Option<String>,
    pub forward_prefix: Option<String>,
    pub visibility_filter_kind: Option<&'static str>,
    pub visibility_filter_names: Vec<String>,
    pub resolved_style_path: Option<String>,
    pub status: &'static str,
    pub resolution_kind: &'static str,
    pub candidate_count: usize,
    pub symlink_chain_link_count: usize,
    pub symlink_chain_links: Vec<OmenaQuerySymlinkChainLinkV0>,
    pub configuration_signature: String,
    pub configuration_variable_count: usize,
    pub invalid_configuration_variable_names: Vec<String>,
    pub module_instance_identity_key: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQuerySymlinkChainLinkV0 {
    pub link_path: String,
    pub target_path: String,
    pub target_was_absolute: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQuerySassModuleGraphClosureEdgeV0 {
    pub from_style_path: String,
    pub target_style_path: String,
    pub edge_kind: &'static str,
    pub depth: usize,
    pub path: Vec<String>,
    pub namespace_kind: Option<&'static str>,
    pub namespace: Option<String>,
    pub forward_prefix: Option<String>,
    pub visibility_filter_kind: Option<&'static str>,
    pub visibility_filter_names: Vec<String>,
    pub configuration_signature: String,
    pub configuration_variable_count: usize,
    pub invalid_configuration_variable_names: Vec<String>,
    pub module_instance_identity_key: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQuerySassModuleCycleV0 {
    pub path: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQuerySassModuleCrossFileResolutionCapabilitiesV0 {
    pub omena_parser_module_edge_consumption_ready: bool,
    pub resolver_backed_source_resolution_ready: bool,
    pub package_manifest_resolution_ready: bool,
    pub external_module_filtering_ready: bool,
    pub graph_closure_ready: bool,
    pub cycle_detection_ready: bool,
    pub namespace_show_hide_filter_ready: bool,
    pub configured_module_instance_identity_ready: bool,
    pub symlink_chain_metadata_ready: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryStyleDocumentSummaryV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub language: &'static str,
    pub selector_names: Vec<String>,
    pub custom_property_decl_names: Vec<AuthoredPropertyTextV0>,
    pub custom_property_ref_names: Vec<AuthoredPropertyTextV0>,
    pub sass_module_use_sources: Vec<String>,
    pub sass_module_forward_sources: Vec<String>,
    pub diagnostic_count: usize,
}

impl PartialEq for OmenaQueryStyleDocumentSummaryV0 {
    fn eq(&self, other: &Self) -> bool {
        self.schema_version == other.schema_version
            && self.product == other.product
            && self.language == other.language
            && self.selector_names == other.selector_names
            && authored_custom_property_sequences_same(
                &self.custom_property_decl_names,
                &other.custom_property_decl_names,
            )
            && authored_custom_property_sequences_same(
                &self.custom_property_ref_names,
                &other.custom_property_ref_names,
            )
            && self.sass_module_use_sources == other.sass_module_use_sources
            && self.sass_module_forward_sources == other.sass_module_forward_sources
            && self.diagnostic_count == other.diagnostic_count
    }
}

impl Eq for OmenaQueryStyleDocumentSummaryV0 {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FastFactsV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub tier: &'static str,
    pub style_path: String,
    pub language: &'static str,
    pub selector_count: usize,
    pub custom_property_count: usize,
    pub sass_symbol_count: usize,
    pub module_edge_count: usize,
    pub parser_error_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalyzedGraphV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub tier: &'static str,
    pub style_path: String,
    pub fast_facts: FastFactsV0,
    pub graph_kinds: Vec<&'static str>,
    pub node_count: usize,
    pub edge_count: usize,
    pub cycle_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StyleEditDistanceSummaryV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub tier: &'static str,
    pub metric_kind: &'static str,
    pub claim_level: &'static str,
    pub public_safety_claim_ready: bool,
    pub left_style_path: String,
    pub right_style_path: String,
    pub left_fast_facts: FastFactsV0,
    pub right_fast_facts: FastFactsV0,
    pub left_analyzed_graph: AnalyzedGraphV0,
    pub right_analyzed_graph: AnalyzedGraphV0,
    pub selector_delta: usize,
    pub custom_property_delta: usize,
    pub sass_symbol_delta: usize,
    pub module_edge_delta: usize,
    pub parser_error_delta: usize,
    pub graph_node_delta: usize,
    pub graph_edge_delta: usize,
    pub graph_cycle_delta: usize,
    pub total_distance: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StyleEditDistanceCascadeMarginBridgeV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub bridge_kind: &'static str,
    pub claim_level: &'static str,
    pub theorem_claimed: bool,
    pub public_safety_claim_ready: bool,
    pub metric_product: &'static str,
    pub metric_kind: &'static str,
    pub margin_product: &'static str,
    pub margin_kind: &'static str,
    pub dominant_axis: &'static str,
    pub edit_distance_total: usize,
    pub cascade_margin_signed_distance: i64,
    pub cascade_margin_abs_distance: u64,
    pub lipschitz_constant_name: &'static str,
    pub lipschitz_constant: Option<u64>,
    pub lipschitz_bound: Option<u64>,
    pub checked: bool,
    pub calibration_stage: &'static str,
    pub incremental_priority_input: IncrementalEditDistancePriorityInputV0,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryFragileGuardedWinnerDiagnosticV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub diagnostic_kind: &'static str,
    pub claim_level: &'static str,
    pub baseline_winner_declaration_id: String,
    pub robustness_radius: u32,
    pub fragile_threshold: u32,
    pub witness: Vec<omena_cascade::GuardedCascadePerturbationV0>,
    pub calibration_stage: &'static str,
    pub public_safety_claim_ready: bool,
    pub false_alarm_boundary: &'static str,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryCascadeConfidenceV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub feature_gate: &'static str,
    pub confidence_kind: &'static str,
    pub claim_level: &'static str,
    pub theorem_claimed: bool,
    pub public_safety_claim_ready: bool,
    pub calibration_stage: &'static str,
    pub margin_product: &'static str,
    pub margin_kind: &'static str,
    pub dominant_axis: &'static str,
    pub dominant_axis_weight_basis_points: u16,
    pub sigmoid_temperature_basis_points: u16,
    pub signed_distance: i64,
    pub abs_distance: u64,
    pub confidence_score_basis_points: u16,
    pub confidence_bucket: &'static str,
    pub winner_declaration_id: String,
    pub challenger_declaration_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryCustomPropertyAnnotationSummaryV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub style_path: String,
    pub annotation_count: usize,
    pub annotations: Vec<OmenaQueryCustomPropertyAnnotationV0>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryCustomPropertyAnnotationV0 {
    pub name: AuthoredPropertyTextV0,
    pub property_key: CanonicalCustomPropertyNameV0,
    pub declaration_count: usize,
    pub reference_count: usize,
    pub annotation_kind: &'static str,
    pub participates_in_fixed_point: bool,
}

impl PartialEq for OmenaQueryCustomPropertyAnnotationV0 {
    fn eq(&self, other: &Self) -> bool {
        self.property_key == other.property_key
            && self.declaration_count == other.declaration_count
            && self.reference_count == other.reference_count
            && self.annotation_kind == other.annotation_kind
            && self.participates_in_fixed_point == other.participates_in_fixed_point
    }
}

impl Eq for OmenaQueryCustomPropertyAnnotationV0 {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryStyleContextIndexV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub style_path: String,
    pub language: &'static str,
    pub context_index_source: &'static str,
    pub context_index: StyleContextIndexV0,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryConsumerCheckSummaryV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub style_path: String,
    pub dialect: &'static str,
    pub token_count: usize,
    pub parser_error_count: usize,
    pub class_selector_count: usize,
    pub custom_property_count: usize,
    pub keyframe_count: usize,
    pub ready_surfaces: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryConsumerBuildSummaryV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub style_path: String,
    pub dialect: &'static str,
    pub requested_pass_ids: Vec<String>,
    pub effective_pass_ids: Vec<String>,
    pub target_query: Option<OmenaQueryTransformTargetQueryPlanV0>,
    pub unknown_pass_ids: Vec<String>,
    pub execution: TransformExecutionSummaryV0,
    pub semantic_removal_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bundle: Option<TransformBundleSourceSummaryV0>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bundle_emission_path: Option<OmenaQueryBundleEmissionPathV0>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_map_v3: Option<OmenaQueryTransformSourceMapV3V0>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub open_world_snapshot: Option<OmenaQueryOpenWorldSnapshotV0>,
    pub ready_surfaces: Vec<&'static str>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum OmenaQueryBuildVerificationProfileV0 {
    #[default]
    Descriptive,
    Strict,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum OmenaQueryBundleEmissionPathV0 {
    #[deprecated(
        since = "0.5.0",
        note = "legacy import-inline bundle emission is scheduled for removal before 1.0"
    )]
    ImportInlineLegacy,
    #[default]
    LinkedOrder,
}

#[allow(deprecated)]
impl OmenaQueryBundleEmissionPathV0 {
    #[deprecated(
        since = "0.5.0",
        note = "legacy import-inline bundle emission is scheduled for removal before 1.0"
    )]
    pub const fn legacy_compatibility() -> Self {
        Self::ImportInlineLegacy
    }

    pub const fn as_wire_label(self) -> &'static str {
        match self {
            Self::ImportInlineLegacy => "importInlineLegacy",
            Self::LinkedOrder => "linkedOrder",
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryBuildAdmissionRequirementsV0 {
    pub refuse_unknown_pass_ids: bool,
    pub require_closed_world_evidence: bool,
    pub require_complete_decisions: bool,
}

impl OmenaQueryBuildAdmissionRequirementsV0 {
    pub const fn strict() -> Self {
        Self {
            refuse_unknown_pass_ids: true,
            require_closed_world_evidence: true,
            require_complete_decisions: true,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(default, rename_all = "camelCase")]
pub struct OmenaQueryConsumerBuildOptionsV0 {
    pub verification_profile: OmenaQueryBuildVerificationProfileV0,
    pub bundle_emission_path: OmenaQueryBundleEmissionPathV0,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase", tag = "kind")]
pub enum OmenaQueryClosedWorldBlockerV0 {
    EmptyEntrypoints,
    MissingEntrypoint {
        source_path: String,
    },
    AmbiguousModulePath {
        source_path: String,
    },
    MissingDependency {
        source_path: String,
        import_source: String,
    },
    MissingModuleInstance {
        module: omena_parser::ModuleInstanceKeyV0,
    },
    MissingModuleDependency {
        module: omena_parser::ModuleInstanceKeyV0,
        dependency: omena_parser::ModuleInstanceKeyV0,
    },
    ClosedWorldPassUnavailable {
        requested_pass_ids: Vec<String>,
    },
    UnsupportedDialectEmissionCycle {
        dialect: OmenaQueryEmissionCycleDialectV0,
        class: OmenaQueryEmissionCycleClassV0,
        #[serde(rename = "edgeKinds")]
        edge_kinds: Vec<TransformBundleEdgeKind>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase", tag = "status")]
pub enum OmenaQueryClosedWorldOutcomeV0 {
    Closed {
        bundle: Box<omena_parser::ClosedWorldBundleV0>,
    },
    Open {
        blockers: Vec<OmenaQueryClosedWorldBlockerV0>,
    },
}

impl OmenaQueryClosedWorldOutcomeV0 {
    pub fn bundle(&self) -> Option<&omena_parser::ClosedWorldBundleV0> {
        match self {
            Self::Closed { bundle } => Some(bundle.as_ref()),
            Self::Open { .. } => None,
        }
    }

    pub fn blockers(&self) -> &[OmenaQueryClosedWorldBlockerV0] {
        match self {
            Self::Closed { .. } => &[],
            Self::Open { blockers } => blockers,
        }
    }

    pub fn is_open(&self) -> bool {
        matches!(self, Self::Open { .. })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryBundleArtifactV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub style_path: String,
    pub emission_path: OmenaQueryBundleEmissionPathV0,
    pub output_css: String,
    pub bundle: TransformBundleSourceSummaryV0,
    pub source_map_v3: OmenaQueryTransformSourceMapV3V0,
    pub code_split_outputs: Vec<OmenaQueryBundleCodeSplitWorkspacePlanOutputV0>,
    pub asset_rewrites: Vec<TransformBundleAssetUrlRewriteSummaryV0>,
    pub per_pass_provenance: Vec<TransformPassExecutionOutcomeV0>,
    /// Compatibility projection that retains entry-scoped execution fields.
    ///
    /// Linked consumers should read `BundleExecutionSummaryV0` from
    /// `OmenaQueryBundleExecutionScopeEvidenceV0::bundle_execution`. This field
    /// remains serialized unchanged; deprecation does not alter the wire shape,
    /// and removal is reserved for a future major release.
    pub execution: TransformExecutionSummaryV0,
    pub ready_surfaces: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryBundleResultV0 {
    pub artifact: OmenaQueryBundleArtifactV0,
    pub closed_world_outcome: OmenaQueryClosedWorldOutcomeV0,
    pub closed_world_decision_parity: OmenaQueryClosedWorldDecisionParityV0,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct OmenaQueryBundleTokenOwnershipResultV0 {
    pub bundle_result: OmenaQueryBundleResultV0,
    pub ownership_census: omena_query_transform_runner::CssModuleTokenOwnershipCensusV0,
}

impl OmenaQueryBundleTokenOwnershipResultV0 {
    pub(crate) fn new(
        bundle_result: OmenaQueryBundleResultV0,
        ownership_census: omena_query_transform_runner::CssModuleTokenOwnershipCensusV0,
    ) -> Self {
        Self {
            bundle_result,
            ownership_census,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub enum OmenaQueryExecutionEvidenceScopeV0 {
    Entry,
    Bundle,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct OmenaQueryExecutionFieldScopeV0 {
    pub field_name: &'static str,
    pub scope: OmenaQueryExecutionEvidenceScopeV0,
    pub derivation: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct OmenaQueryBundleModuleExecutionByteFactsV0 {
    pub module_instance: omena_parser::ModuleInstanceKeyV0,
    pub input_byte_len: usize,
    pub output_byte_len: usize,
    pub generated_start: usize,
    pub generated_end: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct OmenaQueryBundleCompositeExecutionByteFactsV0 {
    pub module_count: usize,
    pub summed_module_input_byte_len: usize,
    pub summed_module_output_byte_len: usize,
    pub inter_module_separator_byte_len: usize,
    pub materialized_output_byte_len: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub enum OmenaQueryLinkedSourceMapGranularityV0 {
    CstAnchors,
    WholeModuleFallback,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct OmenaQueryLinkedSourceMapDispositionV0 {
    pub module_instance: omena_parser::ModuleInstanceKeyV0,
    pub granularity: OmenaQueryLinkedSourceMapGranularityV0,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fallback_reason: Option<&'static str>,
    pub segment_count: usize,
}

/// One linked module's complete transform execution.
///
/// Per-module truth remains available because most transform execution fields
/// have no defensible bundle-level fold.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct BundleModuleExecutionV0 {
    pub module_instance: omena_parser::ModuleInstanceKeyV0,
    pub execution: TransformExecutionSummaryV0,
}

/// Region and count evidence owned by linked bundle materialization.
///
/// This type deliberately carries neither CSS text nor a byte total. The
/// materialized CSS remains on the bundle artifact, and
/// `OmenaQueryBundleCompositeExecutionByteFactsV0` remains the sole byte
/// accounting authority.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct BundleEmissionExecutionV0 {
    pub module_regions: Vec<LinkedEmissionModuleRegionV0>,
    pub order_entry_regions: Vec<LinkedEmissionOrderEntryRegionV0>,
    pub emitted_module_count: usize,
    pub global_order_entry_count: usize,
}

/// Bundle-level transform execution for the linked emission path.
///
/// The aggregate fields are intentionally narrower than
/// `TransformExecutionSummaryV0`: each one has an authored fold and a product
/// run where the folded value differs from the entry module. This summary says
/// nothing about the legacy emission path, and fields without a defensible fold
/// remain available only through `module_executions`. The current product
/// corpus bounds which `aggregate_*` fields have witnesses; it is not a claim
/// that the set is exhaustive. Fold tokens document the intended operation,
/// but witnesses remain load-bearing: the current corpus distinguishes the
/// refusal-count sum, while mutation and semantic-removal values happen to make
/// `sum` and `max` agree.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct BundleExecutionSummaryV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub entry_module_instance: omena_parser::ModuleInstanceKeyV0,
    pub module_executions: Vec<BundleModuleExecutionV0>,
    pub emission_execution: BundleEmissionExecutionV0,
    /// Fold: `sum` over each module execution's mutation count.
    pub aggregate_mutation_count: usize,
    /// Fold: `orderedUnion` over executed pass identifiers in module order.
    pub aggregate_executed_pass_ids: Vec<&'static str>,
    /// Fold: `sum` over each module execution's semantic removal count.
    pub aggregate_semantic_removal_count: usize,
    /// Fold: `sum` over each module execution's closed-world refusal count.
    pub aggregate_closed_world_refusal_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct OmenaQueryBundleExecutionScopeEvidenceV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub entry_module_instance: omena_parser::ModuleInstanceKeyV0,
    pub field_scopes: Vec<OmenaQueryExecutionFieldScopeV0>,
    pub module_executions: Vec<OmenaQueryBundleModuleExecutionByteFactsV0>,
    pub bundle_composite: OmenaQueryBundleCompositeExecutionByteFactsV0,
    pub bundle_execution: BundleExecutionSummaryV0,
    pub source_map_dispositions: Vec<OmenaQueryLinkedSourceMapDispositionV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct OmenaQueryBundleExecutionScopeResultV0 {
    pub bundle_result: OmenaQueryBundleResultV0,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub execution_scope: Option<OmenaQueryBundleExecutionScopeEvidenceV0>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reachability_attribution: Option<OmenaQueryModuleReachabilityAttributionReportV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct OmenaQueryModuleAttributedBundleResultV0 {
    bundle_result: OmenaQueryBundleResultV0,
    reachability_attribution: OmenaQueryModuleReachabilityAttributionReportV0,
}

impl OmenaQueryModuleAttributedBundleResultV0 {
    pub(crate) fn new(
        bundle_result: OmenaQueryBundleResultV0,
        reachability_attribution: OmenaQueryModuleReachabilityAttributionReportV0,
    ) -> Self {
        Self {
            bundle_result,
            reachability_attribution,
        }
    }

    pub fn bundle_result(&self) -> &OmenaQueryBundleResultV0 {
        &self.bundle_result
    }

    pub fn reachability_attribution(&self) -> &OmenaQueryModuleReachabilityAttributionReportV0 {
        &self.reachability_attribution
    }

    pub fn into_bundle_result(self) -> OmenaQueryBundleResultV0 {
        self.bundle_result
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryClosedWorldDecisionParityV0 {
    pub legacy_open_decision: bool,
    pub typed_outcome_open: bool,
    pub equivalent: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryBundleEvidenceGateV0 {
    pub name: &'static str,
    pub passed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryBundleReachabilityEvidenceV0 {
    pub guarantee: GuaranteeKindV0,
    pub interpretation: &'static str,
    pub module_instances: Vec<omena_parser::ModuleInstanceKeyV0>,
    pub closure_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryBundleEvidenceManifestV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub style_path: String,
    pub outcome_status: &'static str,
    pub reachability: Option<OmenaQueryBundleReachabilityEvidenceV0>,
    pub gates: Vec<OmenaQueryBundleEvidenceGateV0>,
    pub blockers: Vec<OmenaQueryClosedWorldBlockerV0>,
    pub interface_hashes: Vec<omena_parser::ClosedWorldInterfaceHashEntryV0>,
    pub source_precision: Option<omena_parser::ClosedWorldSourcePrecisionSummaryV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryBundleWithEvidenceV0 {
    #[serde(flatten)]
    pub artifact: OmenaQueryBundleArtifactV0,
    pub closed_world_outcome: OmenaQueryClosedWorldOutcomeV0,
    pub closed_world_decision_parity: OmenaQueryClosedWorldDecisionParityV0,
    pub evidence: OmenaQueryBundleEvidenceManifestV0,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryBundleCodeSplitWorkspacePlanV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub primary_entry_style_path: String,
    pub configured_entry_count: usize,
    pub output_count: usize,
    pub shared_boundary_count: usize,
    pub outputs: Vec<OmenaQueryBundleCodeSplitWorkspacePlanOutputV0>,
    pub ready_surfaces: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryBundleCodeSplitWorkspacePlanOutputV0 {
    pub source_path: String,
    pub is_entry: bool,
    pub split_boundary: &'static str,
    pub reachable_from_entries: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryTransformPassSummaryV0 {
    pub id: &'static str,
    pub title: &'static str,
    pub reads_semantic_graph: bool,
    pub reads_cascade_model: bool,
    pub explicit_opt_in_required: bool,
    pub dialect_restriction: Option<&'static str>,
    pub spec_snapshot: Option<&'static str>,
    pub opt_in_policy: Option<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryTransformPlanSummaryV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub style_path: String,
    pub dialect: &'static str,
    pub bundle: TransformBundleSourceSummaryV0,
    pub target: TransformTargetPlanV0,
    pub target_query: Option<OmenaQueryTransformTargetQueryPlanV0>,
    pub egg: TransformEggPlanV0,
    pub egg_witnesses: Vec<EggRewriteSourceWitnessV0>,
    pub custom_property_fixed_point: OmenaQueryCustomPropertyLeastFixedPointSummaryV0,
    pub print: TransformPrintArtifactV0,
    pub execution: TransformExecutionSummaryV0,
    pub semantic_removal_count: usize,
    pub combined_plan: TransformPassPlanV0,
    pub combined_pass_ids: Vec<&'static str>,
    pub combined_violated_dag_edge_count: usize,
    pub ready_surfaces: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryTransformExecuteSummaryV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub style_path: String,
    pub requested_pass_ids: Vec<String>,
    pub unknown_pass_ids: Vec<String>,
    pub execution: TransformExecutionSummaryV0,
    pub semantic_removal_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub open_world_snapshot: Option<OmenaQueryOpenWorldSnapshotV0>,
    pub ready_surfaces: Vec<&'static str>,
}

#[cfg(feature = "transform-catalog-trace")]
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryTransformCatalogTransformExecuteSummaryV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub product_scope: &'static str,
    pub default_product_mechanism: bool,
    pub global_transform_theorem_claimed: bool,
    pub execution: OmenaQueryTransformExecuteSummaryV0,
    /// Compatibility field owned by `omena-query` maintainers. Remove not
    /// before 1.0, after downstream migration and zero audited non-compat uses.
    #[deprecated(
        since = "0.4.0",
        note = "use transform_catalog_trace(); removal is not before 1.0 and requires downstream migration plus zero audited non-compatibility uses"
    )]
    pub lawvere_trace: OmenaQueryTransformCatalogModelTraceV0,
    pub parallel_plan: OmenaQueryTransformCatalogTransformPassParallelPlanV0,
    pub reorderability_certificates: Vec<OmenaQueryTransformCatalogReorderabilityCertificateV0>,
    pub differential_witnesses: Vec<OmenaQueryTransformCatalogDifferentialCommutativityWitnessV0>,
    pub ready_surfaces: Vec<&'static str>,
}

/// Pre-1.0 nominal compatibility summary for the former trace surface.
/// Owner: `omena-query` maintainers. Removal condition: not before 1.0,
/// after downstream migration and zero audited in-repo non-compatibility uses.
#[cfg(feature = "transform-catalog-trace")]
#[allow(deprecated)]
#[deprecated(
    since = "0.4.0",
    note = "use OmenaQueryTransformCatalogTransformExecuteSummaryV0; removal is not before 1.0 and requires downstream migration plus zero audited non-compatibility uses"
)]
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryLawvereTransformExecuteSummaryV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub product_scope: &'static str,
    pub default_product_mechanism: bool,
    pub global_transform_theorem_claimed: bool,
    pub execution: OmenaQueryTransformExecuteSummaryV0,
    pub lawvere_trace: omena_query_transform_runner::LawvereModelTraceV0,
    pub parallel_plan: omena_query_transform_runner::TransformPassParallelPlanV0,
    pub reorderability_certificates: Vec<omena_query_transform_runner::ReorderabilityCertificateV0>,
    pub differential_witnesses:
        Vec<omena_query_transform_runner::LawvereDifferentialCommutativityWitnessV0>,
    pub ready_surfaces: Vec<&'static str>,
}

#[cfg(feature = "transform-catalog-trace")]
impl OmenaQueryTransformCatalogTransformExecuteSummaryV0 {
    #[allow(deprecated)]
    pub fn transform_catalog_trace(&self) -> &OmenaQueryTransformCatalogModelTraceV0 {
        transform_catalog_trace_from_legacy_field_v0(self)
    }
}

#[cfg(feature = "transform-catalog-trace")]
#[allow(deprecated)]
#[deprecated(
    since = "0.4.0",
    note = "compatibility field adapter owned by omena-query maintainers; removal is not before 1.0 and requires downstream migration plus zero audited non-compatibility uses"
)]
fn transform_catalog_trace_from_legacy_field_v0(
    summary: &OmenaQueryTransformCatalogTransformExecuteSummaryV0,
) -> &OmenaQueryTransformCatalogModelTraceV0 {
    &summary.lawvere_trace
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryTransformContextFromSourcesSummaryV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub target_style_path: String,
    pub style_count: usize,
    pub context: TransformExecutionContextV0,
    pub import_inline_count: usize,
    pub class_name_rewrite_count: usize,
    pub css_module_composes_resolution_count: usize,
    pub css_module_value_resolution_count: usize,
    pub design_token_route_count: usize,
    pub reachable_class_name_count: usize,
    pub reachable_keyframe_name_count: usize,
    pub reachable_value_name_count: usize,
    pub reachable_custom_property_name_count: usize,
    pub ready_surfaces: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryTransformContextFromEngineInputSummaryV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub input_version: String,
    pub target_style_path: String,
    pub closed_world_requested: bool,
    pub style_source_count: usize,
    pub projection_count: usize,
    pub selected_projection_count: usize,
    pub import_inline_count: usize,
    pub class_name_rewrite_count: usize,
    pub css_module_composes_resolution_count: usize,
    pub css_module_value_resolution_count: usize,
    pub design_token_route_count: usize,
    pub reachable_class_name_count: usize,
    pub reachable_keyframe_name_count: usize,
    pub reachable_value_name_count: usize,
    pub reachable_custom_property_name_count: usize,
    pub reachability_sources: Vec<OmenaQuerySemanticReachabilitySourceV0>,
    pub context: TransformExecutionContextV0,
    pub ready_surfaces: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQuerySemanticReachabilitySourceV0 {
    pub graph_id: String,
    pub file_path: String,
    pub node_id: String,
    pub target_style_paths: Vec<String>,
    pub value_kind: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reduced_product: Option<ReducedClassValueProductV0>,
    pub selector_names: Vec<String>,
    pub certainty: SelectorProjectionCertaintyV0,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub enum OmenaQueryModuleReachabilityAttributionKindV0 {
    TargetedProjection,
    UnattributedProjectionFanout,
    TargetedAndUnattributedProjection,
    NoApplicableProjection,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct OmenaQueryModuleReachabilityAttributionV0 {
    style_path: String,
    class_names: Vec<String>,
    targeted_projection_count: usize,
    unattributed_projection_count: usize,
    attribution_kind: OmenaQueryModuleReachabilityAttributionKindV0,
}

impl OmenaQueryModuleReachabilityAttributionV0 {
    pub(crate) fn new(
        style_path: String,
        class_names: Vec<String>,
        targeted_projection_count: usize,
        unattributed_projection_count: usize,
    ) -> Self {
        let attribution_kind = match (
            targeted_projection_count > 0,
            unattributed_projection_count > 0,
        ) {
            (true, true) => {
                OmenaQueryModuleReachabilityAttributionKindV0::TargetedAndUnattributedProjection
            }
            (true, false) => OmenaQueryModuleReachabilityAttributionKindV0::TargetedProjection,
            (false, true) => {
                OmenaQueryModuleReachabilityAttributionKindV0::UnattributedProjectionFanout
            }
            (false, false) => OmenaQueryModuleReachabilityAttributionKindV0::NoApplicableProjection,
        };
        Self {
            style_path,
            class_names,
            targeted_projection_count,
            unattributed_projection_count,
            attribution_kind,
        }
    }

    pub fn style_path(&self) -> &str {
        self.style_path.as_str()
    }

    pub fn class_names(&self) -> &[String] {
        self.class_names.as_slice()
    }

    pub fn targeted_projection_count(&self) -> usize {
        self.targeted_projection_count
    }

    pub fn unattributed_projection_count(&self) -> usize {
        self.unattributed_projection_count
    }

    pub fn attribution_kind(&self) -> OmenaQueryModuleReachabilityAttributionKindV0 {
        self.attribution_kind
    }

    pub fn was_attempted(&self) -> bool {
        self.targeted_projection_count > 0 || self.unattributed_projection_count > 0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct OmenaQueryEngineInputModuleAttributionV0 {
    declared_class_names_by_style_path: BTreeMap<String, Vec<String>>,
    targeted_class_names_by_style_path: BTreeMap<String, Vec<String>>,
    targeted_projection_count_by_style_path: BTreeMap<String, usize>,
}

impl OmenaQueryEngineInputModuleAttributionV0 {
    pub(crate) fn new(
        declared_class_names_by_style_path: BTreeMap<String, Vec<String>>,
        targeted_class_names_by_style_path: BTreeMap<String, Vec<String>>,
        targeted_projection_count_by_style_path: BTreeMap<String, usize>,
    ) -> Self {
        Self {
            declared_class_names_by_style_path,
            targeted_class_names_by_style_path,
            targeted_projection_count_by_style_path,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct OmenaQueryEngineInputModuleReachabilityV0 {
    summary: OmenaQueryTransformContextFromEngineInputSummaryV0,
    known_style_paths: Vec<String>,
    declared_class_names_by_style_path: BTreeMap<String, Vec<String>>,
    targeted_class_names_by_style_path: BTreeMap<String, Vec<String>>,
    targeted_projection_count_by_style_path: BTreeMap<String, usize>,
    unattributed_class_names: Vec<String>,
    unattributed_projection_count: usize,
    projected_class_names: Vec<String>,
    projection_summary_evaluation_count: usize,
}

impl OmenaQueryEngineInputModuleReachabilityV0 {
    pub(crate) fn new(
        summary: OmenaQueryTransformContextFromEngineInputSummaryV0,
        known_style_paths: Vec<String>,
        module_attribution: OmenaQueryEngineInputModuleAttributionV0,
        unattributed_class_names: Vec<String>,
        unattributed_projection_count: usize,
        projected_class_names: Vec<String>,
    ) -> Self {
        let OmenaQueryEngineInputModuleAttributionV0 {
            declared_class_names_by_style_path,
            targeted_class_names_by_style_path,
            targeted_projection_count_by_style_path,
        } = module_attribution;
        Self {
            summary,
            known_style_paths,
            declared_class_names_by_style_path,
            targeted_class_names_by_style_path,
            targeted_projection_count_by_style_path,
            unattributed_class_names,
            unattributed_projection_count,
            projected_class_names,
            projection_summary_evaluation_count: 1,
        }
    }

    pub fn summary(&self) -> &OmenaQueryTransformContextFromEngineInputSummaryV0 {
        &self.summary
    }

    pub fn context(&self) -> &TransformExecutionContextV0 {
        &self.summary.context
    }

    pub fn into_summary(self) -> OmenaQueryTransformContextFromEngineInputSummaryV0 {
        self.summary
    }

    pub fn projected_class_names(&self) -> &[String] {
        self.projected_class_names.as_slice()
    }

    pub fn projection_summary_evaluation_count(&self) -> usize {
        self.projection_summary_evaluation_count
    }

    pub fn module_attribution(
        &self,
        style_path: &str,
    ) -> OmenaQueryModuleReachabilityAttributionV0 {
        let style_path = resolve_omena_query_style_path_against_known(
            style_path,
            self.known_style_paths.as_slice(),
        )
        .unwrap_or_else(|| normalize_omena_query_style_path(style_path));
        let mut class_names = self.unattributed_class_names.clone();
        if let Some(targeted_class_names) = self
            .targeted_class_names_by_style_path
            .get(style_path.as_str())
        {
            class_names.extend(targeted_class_names.iter().cloned());
        }
        class_names.sort();
        class_names.dedup();
        OmenaQueryModuleReachabilityAttributionV0::new(
            style_path.clone(),
            class_names,
            self.targeted_projection_count_by_style_path
                .get(style_path.as_str())
                .copied()
                .unwrap_or_default(),
            self.unattributed_projection_count,
        )
    }

    pub(crate) fn targeted_style_paths(&self) -> impl Iterator<Item = &str> {
        self.targeted_projection_count_by_style_path
            .keys()
            .map(String::as_str)
    }

    pub(crate) fn flat_class_names_for_style_paths<'a>(
        &self,
        style_paths: impl IntoIterator<Item = &'a str>,
        candidate_class_names: &[String],
    ) -> Vec<String> {
        let style_paths =
            module_attribution_domain_style_paths(style_paths, self.known_style_paths.as_slice());

        candidate_class_names
            .iter()
            .filter(|class_name| {
                let mut declared_owner_paths = self
                    .declared_class_names_by_style_path
                    .iter()
                    .filter(|(_, names)| {
                        class_name_collection_contains(names.iter().map(String::as_str), class_name)
                    })
                    .map(|(style_path, _)| style_path);
                let Some(first_owner_path) = declared_owner_paths.next() else {
                    return true;
                };
                style_paths.contains(first_owner_path)
                    || declared_owner_paths.any(|style_path| style_paths.contains(style_path))
            })
            .cloned()
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect()
    }
}

pub(crate) fn normalize_omena_query_style_path(style_path: &str) -> String {
    normalize_omena_transform_bundle_path(style_path)
}

pub(crate) fn resolve_omena_query_style_path_against_known(
    style_path: &str,
    known_style_paths: &[String],
) -> Option<String> {
    let normalized = normalize_omena_query_style_path(style_path);
    unique_style_path_match(known_style_paths, |known| {
        normalize_omena_query_style_path(known) == normalized
    })
    .or_else(|| {
        unique_style_path_match(known_style_paths, |known| {
            normalize_omena_query_style_path(known).eq_ignore_ascii_case(normalized.as_str())
        })
    })
    .or_else(|| {
        unique_style_path_match(known_style_paths, |known| {
            style_path_component_suffix_matches(
                normalize_omena_query_style_path(known).as_str(),
                normalized.as_str(),
                false,
            )
        })
    })
    .or_else(|| {
        unique_style_path_match(known_style_paths, |known| {
            style_path_component_suffix_matches(
                normalize_omena_query_style_path(known).as_str(),
                normalized.as_str(),
                true,
            )
        })
    })
}

fn attribution_domain_style_paths(style_path: &str, known_style_paths: &[String]) -> Vec<String> {
    let normalized = normalize_omena_query_style_path(style_path);
    for matches in [
        matching_style_paths(known_style_paths, |known| {
            normalize_omena_query_style_path(known) == normalized
        }),
        matching_style_paths(known_style_paths, |known| {
            normalize_omena_query_style_path(known).eq_ignore_ascii_case(normalized.as_str())
        }),
        matching_style_paths(known_style_paths, |known| {
            style_path_component_suffix_matches(
                normalize_omena_query_style_path(known).as_str(),
                normalized.as_str(),
                false,
            )
        }),
        matching_style_paths(known_style_paths, |known| {
            style_path_component_suffix_matches(
                normalize_omena_query_style_path(known).as_str(),
                normalized.as_str(),
                true,
            )
        }),
    ] {
        if !matches.is_empty() {
            return matches;
        }
    }
    vec![normalized]
}

// Admission and placement share this domain so every admitted name either
// fans out as ownerless or has at least one matching entry to receive it.
fn module_attribution_domain_style_paths<'a>(
    style_paths: impl IntoIterator<Item = &'a str>,
    known_style_paths: &[String],
) -> BTreeSet<String> {
    style_paths
        .into_iter()
        .flat_map(|style_path| attribution_domain_style_paths(style_path, known_style_paths))
        .collect()
}

fn matching_style_paths(
    known_style_paths: &[String],
    matches: impl Fn(&String) -> bool,
) -> Vec<String> {
    known_style_paths
        .iter()
        .filter(|known| matches(known))
        .map(|known| normalize_omena_query_style_path(known))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn unique_style_path_match(
    known_style_paths: &[String],
    matches: impl Fn(&String) -> bool,
) -> Option<String> {
    let mut matching = known_style_paths.iter().filter(|known| matches(known));
    let first = matching.next()?;
    matching.next().is_none().then(|| first.clone())
}

fn style_path_component_suffix_matches(left: &str, right: &str, ignore_case: bool) -> bool {
    let left = left.trim_matches('/');
    let right = right.trim_matches('/');
    let (left, right) = if ignore_case {
        (left.to_ascii_lowercase(), right.to_ascii_lowercase())
    } else {
        (left.to_string(), right.to_string())
    };
    left == right
        || left
            .strip_suffix(right.as_str())
            .is_some_and(|prefix| prefix.ends_with('/'))
        || right
            .strip_suffix(left.as_str())
            .is_some_and(|prefix| prefix.ends_with('/'))
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct OmenaQueryModuleReachabilityAttributionReportV0 {
    entries: Vec<OmenaQueryModuleReachabilityAttributionV0>,
    projection_summary_evaluation_count: usize,
    projected_class_names: Vec<String>,
    unattributed_class_names: Vec<String>,
    flat_class_names: Vec<String>,
    attributed_class_names: Vec<String>,
    lost_class_names: Vec<String>,
    unmatched_target_style_paths: Vec<String>,
    attempted_module_count: usize,
    attributed_empty_module_count: usize,
}

impl OmenaQueryModuleReachabilityAttributionReportV0 {
    pub(crate) fn from_style_paths<'a>(
        attribution: &OmenaQueryEngineInputModuleReachabilityV0,
        style_paths: impl IntoIterator<Item = &'a str>,
        flat_class_names: &[String],
    ) -> Self {
        let mut style_paths = module_attribution_domain_style_paths(
            style_paths,
            attribution.known_style_paths.as_slice(),
        )
        .into_iter()
        .collect::<Vec<_>>();
        style_paths.sort();
        style_paths.dedup();
        let mut entries = style_paths
            .iter()
            .map(|style_path| attribution.module_attribution(style_path))
            .collect::<Vec<_>>();
        entries.sort_by(|left, right| left.style_path().cmp(right.style_path()));
        entries.dedup_by(|left, right| left.style_path() == right.style_path());

        let mut flat_class_names = flat_class_names.to_vec();
        flat_class_names.sort();
        flat_class_names.dedup();
        let directly_attributed_class_names = entries
            .iter()
            .flat_map(|entry| entry.class_names.iter().cloned())
            .collect::<BTreeSet<_>>();
        for class_name in &flat_class_names {
            if class_name_collection_contains(
                directly_attributed_class_names.iter().map(String::as_str),
                class_name,
            ) {
                continue;
            }
            let declared_owner_paths = attribution
                .declared_class_names_by_style_path
                .iter()
                .filter(|(_, names)| {
                    class_name_collection_contains(names.iter().map(String::as_str), class_name)
                })
                .map(|(style_path, _)| normalize_omena_query_style_path(style_path))
                .collect::<BTreeSet<_>>();
            for entry in &mut entries {
                if declared_owner_paths.is_empty()
                    || declared_owner_paths
                        .contains(&normalize_omena_query_style_path(entry.style_path.as_str()))
                {
                    entry.class_names.push(class_name.clone());
                    entry.class_names.sort();
                    entry.class_names.dedup();
                }
            }
        }

        let style_path_set = entries
            .iter()
            .map(OmenaQueryModuleReachabilityAttributionV0::style_path)
            .collect::<BTreeSet<_>>();
        let attributed_class_names = entries
            .iter()
            .flat_map(|entry| entry.class_names.iter().cloned())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        let unmatched_target_style_paths = attribution
            .targeted_style_paths()
            .filter(|path| !style_path_set.contains(path))
            .map(str::to_string)
            .collect::<Vec<_>>();
        let attempted_module_count = entries.iter().filter(|entry| entry.was_attempted()).count();
        let attributed_empty_module_count = entries
            .iter()
            .filter(|entry| entry.was_attempted() && entry.class_names.is_empty())
            .count();
        Self {
            entries,
            projection_summary_evaluation_count: attribution.projection_summary_evaluation_count(),
            projected_class_names: attribution.projected_class_names.clone(),
            unattributed_class_names: attribution.unattributed_class_names.clone(),
            flat_class_names,
            attributed_class_names,
            lost_class_names: Vec::new(),
            unmatched_target_style_paths,
            attempted_module_count,
            attributed_empty_module_count,
        }
    }

    pub fn entries(&self) -> &[OmenaQueryModuleReachabilityAttributionV0] {
        self.entries.as_slice()
    }

    pub fn entry_for_style_path(
        &self,
        style_path: &str,
    ) -> Option<&OmenaQueryModuleReachabilityAttributionV0> {
        let known_style_paths = self
            .entries
            .iter()
            .map(|entry| entry.style_path().to_string())
            .collect::<Vec<_>>();
        let style_path =
            resolve_omena_query_style_path_against_known(style_path, known_style_paths.as_slice())
                .unwrap_or_else(|| normalize_omena_query_style_path(style_path));
        self.entries
            .binary_search_by(|entry| entry.style_path().cmp(style_path.as_str()))
            .ok()
            .map(|index| &self.entries[index])
    }

    pub fn projection_summary_evaluation_count(&self) -> usize {
        self.projection_summary_evaluation_count
    }

    pub fn projected_class_names(&self) -> &[String] {
        self.projected_class_names.as_slice()
    }

    /// Names from source projections that could not be assigned to one style module.
    pub fn unattributed_class_names(&self) -> &[String] {
        self.unattributed_class_names.as_slice()
    }

    /// Class names whose declared owners intersect the current build-source domain.
    pub fn flat_class_names(&self) -> &[String] {
        self.flat_class_names.as_slice()
    }

    pub fn attributed_class_names(&self) -> &[String] {
        self.attributed_class_names.as_slice()
    }

    /// Always-empty compatibility view for the shared admission/placement domain.
    ///
    /// This does not establish whether linked output retained every live declaration;
    /// emission-level checks own that separate guarantee.
    pub fn lost_class_names(&self) -> &[String] {
        self.lost_class_names.as_slice()
    }

    pub fn unmatched_target_style_paths(&self) -> &[String] {
        self.unmatched_target_style_paths.as_slice()
    }

    pub fn attempted_module_count(&self) -> usize {
        self.attempted_module_count
    }

    pub fn attributed_empty_module_count(&self) -> usize {
        self.attributed_empty_module_count
    }
}

fn class_name_collection_contains<'a>(
    names: impl IntoIterator<Item = &'a str>,
    candidate: &str,
) -> bool {
    let candidate = omena_syntax::ident::ClassNameV0::new(candidate);
    names
        .into_iter()
        .any(|name| omena_syntax::ident::ClassNameV0::new(name).same_as(&candidate))
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryStyleSourceInputV0 {
    pub style_path: String,
    pub style_source: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryExternalSifInputV0 {
    pub canonical_url: String,
    pub sif: OmenaSifV1,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQuerySourceDocumentInputV0 {
    pub source_path: String,
    pub source_source: String,
    /// Precomputed source syntax facts from the LSP workspace/source index. When
    /// present, query consumers can avoid reparsing source text while preserving
    /// the existing text-backed fallback for non-indexed callers.
    #[serde(default, skip_deserializing, skip_serializing_if = "Option::is_none")]
    pub source_syntax_index: Option<OmenaQuerySourceSyntaxIndexV0>,
    #[serde(default)]
    pub has_unresolved_style_import: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryOmenaParserStyleFactsV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub dialect: &'static str,
    pub class_selector_names: Vec<String>,
    pub id_selector_names: Vec<String>,
    pub placeholder_selector_names: Vec<String>,
    pub keyframe_names: Vec<String>,
    pub animation_reference_names: Vec<String>,
    pub css_module_value_definition_names: Vec<String>,
    pub css_module_value_reference_names: Vec<String>,
    pub css_module_value_import_sources: Vec<String>,
    pub css_module_value_import_edges: Vec<OmenaQueryCssModuleValueImportEdgeFactV0>,
    pub css_module_value_definition_edges: Vec<OmenaQueryCssModuleValueDefinitionEdgeFactV0>,
    pub css_module_composes_target_names: Vec<String>,
    pub css_module_composes_import_sources: Vec<String>,
    pub css_module_composes_edges: Vec<OmenaQueryCssModuleComposesEdgeFactV0>,
    pub icss_export_names: Vec<String>,
    pub icss_import_local_names: Vec<String>,
    pub icss_import_remote_names: Vec<String>,
    pub icss_import_sources: Vec<String>,
    pub icss_import_edges: Vec<OmenaQueryIcssImportEdgeFactV0>,
    pub icss_export_edges: Vec<OmenaQueryIcssExportEdgeFactV0>,
    pub variable_names: Vec<String>,
    pub sass_symbol_declaration_names: Vec<String>,
    pub sass_symbol_reference_names: Vec<String>,
    pub sass_symbol_facts: Vec<OmenaQuerySassSymbolFactV0>,
    pub sass_symbol_resolution: OmenaQuerySassSymbolResolutionV0,
    pub sass_module_use_sources: Vec<String>,
    pub sass_module_forward_sources: Vec<String>,
    pub sass_module_import_sources: Vec<String>,
    pub sass_module_edges: Vec<OmenaQuerySassModuleEdgeFactV0>,
    pub custom_property_names: Vec<AuthoredPropertyTextV0>,
    pub custom_property_decl_names: Vec<AuthoredPropertyTextV0>,
    pub custom_property_ref_names: Vec<AuthoredPropertyTextV0>,
    pub at_rule_names: Vec<String>,
    pub parser_error_count: usize,
}

impl PartialEq for OmenaQueryOmenaParserStyleFactsV0 {
    fn eq(&self, other: &Self) -> bool {
        self.schema_version == other.schema_version
            && self.product == other.product
            && self.dialect == other.dialect
            && self.class_selector_names == other.class_selector_names
            && self.id_selector_names == other.id_selector_names
            && self.placeholder_selector_names == other.placeholder_selector_names
            && self.keyframe_names == other.keyframe_names
            && self.animation_reference_names == other.animation_reference_names
            && self.css_module_value_definition_names == other.css_module_value_definition_names
            && self.css_module_value_reference_names == other.css_module_value_reference_names
            && self.css_module_value_import_sources == other.css_module_value_import_sources
            && self.css_module_value_import_edges == other.css_module_value_import_edges
            && self.css_module_value_definition_edges == other.css_module_value_definition_edges
            && self.css_module_composes_target_names == other.css_module_composes_target_names
            && self.css_module_composes_import_sources == other.css_module_composes_import_sources
            && self.css_module_composes_edges == other.css_module_composes_edges
            && self.icss_export_names == other.icss_export_names
            && self.icss_import_local_names == other.icss_import_local_names
            && self.icss_import_remote_names == other.icss_import_remote_names
            && self.icss_import_sources == other.icss_import_sources
            && self.icss_import_edges == other.icss_import_edges
            && self.icss_export_edges == other.icss_export_edges
            && self.variable_names == other.variable_names
            && self.sass_symbol_declaration_names == other.sass_symbol_declaration_names
            && self.sass_symbol_reference_names == other.sass_symbol_reference_names
            && self.sass_symbol_facts == other.sass_symbol_facts
            && self.sass_symbol_resolution == other.sass_symbol_resolution
            && self.sass_module_use_sources == other.sass_module_use_sources
            && self.sass_module_forward_sources == other.sass_module_forward_sources
            && self.sass_module_import_sources == other.sass_module_import_sources
            && self.sass_module_edges == other.sass_module_edges
            && authored_custom_property_sequences_same(
                &self.custom_property_names,
                &other.custom_property_names,
            )
            && authored_custom_property_sequences_same(
                &self.custom_property_decl_names,
                &other.custom_property_decl_names,
            )
            && authored_custom_property_sequences_same(
                &self.custom_property_ref_names,
                &other.custom_property_ref_names,
            )
            && self.at_rule_names == other.at_rule_names
            && self.parser_error_count == other.parser_error_count
    }
}

impl Eq for OmenaQueryOmenaParserStyleFactsV0 {}

fn authored_custom_property_sequences_same(
    left: &[AuthoredPropertyTextV0],
    right: &[AuthoredPropertyTextV0],
) -> bool {
    left.len() == right.len()
        && left
            .iter()
            .zip(right)
            .all(|(left, right)| left.to_custom_key() == right.to_custom_key())
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQuerySassSymbolFactV0 {
    pub kind: &'static str,
    pub symbol_kind: &'static str,
    pub name: String,
    pub role: &'static str,
    pub namespace: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQuerySassSymbolResolutionV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub resolution_scope: &'static str,
    pub declaration_count: usize,
    pub reference_count: usize,
    pub resolved_reference_count: usize,
    pub unresolved_reference_count: usize,
    pub edges: Vec<OmenaQuerySassSymbolResolutionEdgeV0>,
    pub capabilities: OmenaQuerySassSymbolResolutionCapabilitiesV0,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQuerySassSymbolResolutionEdgeV0 {
    pub symbol_kind: &'static str,
    pub name: String,
    pub namespace: Option<String>,
    pub reference_kind: &'static str,
    pub reference_role: &'static str,
    pub reference_source_order: usize,
    pub declaration_kind: Option<&'static str>,
    pub declaration_source_order: Option<usize>,
    pub status: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQuerySassSymbolResolutionCapabilitiesV0 {
    pub same_file_lexical_resolution_ready: bool,
    pub declaration_before_reference_ready: bool,
    pub unresolved_reference_reporting_ready: bool,
    pub cross_file_module_resolution_ready: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQuerySassModuleEdgeFactV0 {
    pub kind: &'static str,
    pub source: String,
    pub namespace_kind: Option<&'static str>,
    pub namespace: Option<String>,
    pub forward_prefix: Option<String>,
    pub visibility_filter_kind: Option<&'static str>,
    pub visibility_filter_names: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQuerySassModuleSourceEdgeV0 {
    pub kind: &'static str,
    pub source: String,
    pub byte_span: ParserByteSpanV0,
    pub namespace_kind: Option<&'static str>,
    pub namespace: Option<String>,
    pub forward_prefix: Option<String>,
    pub visibility_filter_kind: Option<&'static str>,
    pub visibility_filter_names: Vec<String>,
    pub media_qualified: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryCssModuleValueImportEdgeFactV0 {
    pub remote_name: String,
    pub local_name: String,
    pub import_source: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryCssModuleValueDefinitionEdgeFactV0 {
    pub definition_name: String,
    pub reference_names: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryCssModuleComposesEdgeFactV0 {
    pub kind: &'static str,
    pub owner_selector_names: Vec<String>,
    pub target_names: Vec<String>,
    pub import_source: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryIcssImportEdgeFactV0 {
    pub local_name: String,
    pub remote_name: String,
    pub import_source: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryIcssExportEdgeFactV0 {
    pub export_name: String,
    pub reference_names: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryStyleHoverCandidateV0 {
    pub kind: &'static str,
    pub name: AuthoredPropertyTextV0,
    #[serde(skip)]
    pub selector_key: Option<CanonicalClassKeyV0>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub property_key: Option<CanonicalCustomPropertyNameV0>,
    pub range: ParserRangeV0,
    pub source: &'static str,
    pub namespace: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum OmenaQueryStyleHoverCandidateIdentityRefV0<'candidate> {
    Selector(Option<&'candidate CanonicalClassKeyV0>),
    CustomProperty(Option<&'candidate CanonicalCustomPropertyNameV0>),
    Other,
}

impl OmenaQueryStyleHoverCandidateV0 {
    fn identity(&self) -> OmenaQueryStyleHoverCandidateIdentityRefV0<'_> {
        if self.kind == "selector" {
            OmenaQueryStyleHoverCandidateIdentityRefV0::Selector(self.selector_key.as_ref())
        } else if matches!(
            self.kind,
            "customPropertyDeclaration" | "customPropertyReference"
        ) {
            OmenaQueryStyleHoverCandidateIdentityRefV0::CustomProperty(self.property_key.as_ref())
        } else {
            OmenaQueryStyleHoverCandidateIdentityRefV0::Other
        }
    }
}

impl PartialEq for OmenaQueryStyleHoverCandidateV0 {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind
            && self.identity() == other.identity()
            && self.range == other.range
            && self.source == other.source
            && self.namespace == other.namespace
    }
}

impl Eq for OmenaQueryStyleHoverCandidateV0 {}

impl Ord for OmenaQueryStyleHoverCandidateV0 {
    fn cmp(&self, other: &Self) -> Ordering {
        (
            self.kind,
            self.identity(),
            self.range,
            self.source,
            &self.namespace,
        )
            .cmp(&(
                other.kind,
                other.identity(),
                other.range,
                other.source,
                &other.namespace,
            ))
    }
}

impl PartialOrd for OmenaQueryStyleHoverCandidateV0 {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryStyleHoverCandidatesV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub language: &'static str,
    pub candidates: Vec<OmenaQueryStyleHoverCandidateV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryStyleHoverRenderPartsV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub snippet: String,
    pub value: Option<String>,
    pub signature: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub property_value_narrowings: Vec<AbstractPropertyValueNarrowingV0>,
    pub render_source: &'static str,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryCascadeNarrowingEvidenceV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub selector: String,
    pub selector_class_names: Vec<String>,
    pub property_name: AuthoredPropertyTextV0,
    pub property_key: CanonicalPropertyKeyV0,
    pub condition_context: Vec<String>,
    pub declaration_ids: Vec<String>,
    pub element_class_iteration: ReducedClassValueProductIterationV0,
    pub property_value_narrowing: AbstractPropertyValueNarrowingV0,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runtime_state: Option<OmenaQueryRuntimeStateScenarioEvidenceV0>,
}

impl PartialEq for OmenaQueryCascadeNarrowingEvidenceV0 {
    fn eq(&self, other: &Self) -> bool {
        self.schema_version == other.schema_version
            && self.product == other.product
            && self.selector == other.selector
            && self.selector_class_names.eq(&other.selector_class_names)
            && self.property_key == other.property_key
            && self.condition_context == other.condition_context
            && self.declaration_ids == other.declaration_ids
            && self.element_class_iteration == other.element_class_iteration
            && self.property_value_narrowing == other.property_value_narrowing
            && self.runtime_state == other.runtime_state
    }
}

impl Eq for OmenaQueryCascadeNarrowingEvidenceV0 {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryStaticConditionPruningEvidenceV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub condition_context: Vec<String>,
    pub assumption: &'static str,
    pub verdict: &'static str,
    pub pruned: bool,
    pub anchor_context: bool,
}

/// Why a cascade result could not use a complete named-layer ordering.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryCascadeLayerTopologyIncompleteV0 {
    /// Number of unresolved layer-topology facts observed by the semantic index.
    pub unresolved_count: usize,
}

#[derive(Debug, Clone)]
pub struct OmenaQueryRuntimeStateScenarioEvidenceV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub selector: String,
    pub selector_class_names: Vec<String>,
    pub property_name: AuthoredPropertyTextV0,
    pub scenario_join_kind: &'static str,
    pub confidence_tier: &'static str,
    pub confidence_tier_within_modeled_environment: &'static str,
    pub static_boundary: OmenaQueryRuntimeStateStaticBoundaryV0,
    pub driver_summaries: Vec<OmenaQueryRuntimeStateDriverSummaryV0>,
    pub scenarios: Vec<OmenaQueryRuntimeStateScenarioV0>,
    pub static_condition_pruning: Vec<OmenaQueryStaticConditionPruningEvidenceV0>,
    pub inline_style_overrides: Vec<OmenaQueryInlineStyleRuntimeOverrideV0>,
    pub cascade_layer_topology_incomplete: Option<OmenaQueryCascadeLayerTopologyIncompleteV0>,
    pub guarded_winner_authority: Option<omena_cascade::GuardedCascadeWinnerAuthorityV0>,
    pub fragile_guarded_winner_diagnostics: Vec<OmenaQueryFragileGuardedWinnerDiagnosticV0>,
}

impl PartialEq for OmenaQueryRuntimeStateScenarioEvidenceV0 {
    fn eq(&self, other: &Self) -> bool {
        self.schema_version == other.schema_version
            && self.product == other.product
            && self.selector == other.selector
            && self.selector_class_names == other.selector_class_names
            && self
                .property_name
                .to_property_name()
                .same_as(&other.property_name.to_property_name())
            && self.scenario_join_kind == other.scenario_join_kind
            && self.confidence_tier == other.confidence_tier
            && self.confidence_tier_within_modeled_environment
                == other.confidence_tier_within_modeled_environment
            && self.static_boundary == other.static_boundary
            && self.driver_summaries == other.driver_summaries
            && self.scenarios == other.scenarios
            && self.static_condition_pruning == other.static_condition_pruning
            && self.inline_style_overrides == other.inline_style_overrides
            && self.cascade_layer_topology_incomplete == other.cascade_layer_topology_incomplete
            && self.guarded_winner_authority == other.guarded_winner_authority
            && self.fragile_guarded_winner_diagnostics == other.fragile_guarded_winner_diagnostics
    }
}

impl Eq for OmenaQueryRuntimeStateScenarioEvidenceV0 {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryRuntimeStateStaticBoundaryV0 {
    pub boundary_kind: &'static str,
    pub static_value_assuming_no_runtime_override: bool,
    pub tracks_dom_mutation: bool,
    pub tracks_class_list_mutation: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryRuntimeStateDriverSummaryV0 {
    pub driver: &'static str,
    pub status: &'static str,
    pub scenario_count: usize,
    pub provenance: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OmenaQueryRuntimeStateScenarioV0 {
    pub scenario_kind: &'static str,
    pub pseudo_state: Option<String>,
    pub condition_context: Vec<String>,
    pub declaration_ids: Vec<String>,
    pub winner_declaration_id: Option<String>,
    pub winner_value: Option<String>,
    pub property_value_narrowing: AbstractPropertyValueNarrowingV0,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryInlineStyleRuntimeOverrideV0 {
    pub source_path: String,
    pub range: ParserRangeV0,
    pub property_name: AuthoredPropertyTextV0,
    pub value: Option<String>,
    pub cascade_tier: &'static str,
    /// Whether the static source text ended with a CSS `!important` suffix.
    ///
    /// This is a source-text observation, not a claim about browser setter behavior.
    pub important: bool,
    pub static_value: bool,
}

impl PartialEq for OmenaQueryInlineStyleRuntimeOverrideV0 {
    fn eq(&self, other: &Self) -> bool {
        self.source_path == other.source_path
            && self.range == other.range
            && self
                .property_name
                .to_property_name()
                .same_as(&other.property_name.to_property_name())
            && self.value == other.value
            && self.cascade_tier == other.cascade_tier
            && self.important == other.important
            && self.static_value == other.static_value
    }
}

impl Eq for OmenaQueryInlineStyleRuntimeOverrideV0 {}

impl OmenaQueryInlineStyleRuntimeOverrideV0 {
    /// Whether the originating static source text ended with `!important`.
    ///
    /// The flag describes source syntax, not browser setter behavior.
    pub fn important_suffix_present(&self) -> bool {
        self.important
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryStyleDiagnosticV0 {
    pub code: &'static str,
    pub severity: &'static str,
    pub provenance: Vec<&'static str>,
    pub range: ParserRangeV0,
    pub message: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<u8>,
    pub create_custom_property: Option<OmenaQueryCreateCustomPropertyActionV0>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cascade_narrowing: Option<OmenaQueryCascadeNarrowingEvidenceV0>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cascade_confidence: Option<OmenaQueryCascadeConfidenceV0>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub polynomial_provenance: Option<OmenaQueryPolynomialProvenanceV0>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cross_file_scc: Option<OmenaQueryCrossFileSccEvidenceV0>,
}

pub type OmenaQueryLinearProvenanceV0 = LinearProvenanceV0<NaturalCountProvenanceSemiringV0>;
pub type OmenaQueryPolynomialProvenanceV0 = PolynomialProvenanceV0;

pub fn summarize_omena_query_linear_provenance(
    provenance: &[&'static str],
) -> OmenaQueryLinearProvenanceV0 {
    let labels = project_omena_query_diagnostic_provenance_from_evidence_graph(
        "linearProvenance",
        provenance.to_vec(),
    );
    summarize_omena_query_linear_provenance_with_support_count(labels.as_slice(), 1)
}

pub fn summarize_omena_query_linear_provenance_with_support_count(
    provenance: &[&'static str],
    support_count: u8,
) -> OmenaQueryLinearProvenanceV0 {
    let path = if support_count == 0 {
        LinearProvenancePathV0::unsupported(provenance)
    } else {
        LinearProvenancePathV0::supported(provenance, support_count)
    };
    OmenaQueryLinearProvenanceV0::from_composed_paths(&[path])
}

pub fn summarize_omena_query_polynomial_provenance(
    provenance: &[&'static str],
) -> OmenaQueryPolynomialProvenanceV0 {
    let labels = project_omena_query_diagnostic_provenance_from_evidence_graph(
        "polynomialProvenance",
        provenance.to_vec(),
    );
    let linear_provenance = summarize_omena_query_linear_provenance(labels.as_slice());
    summarize_polynomial_provenance_from_linear_v0(&linear_provenance, "diagnosticDefaultThreeTier")
}

pub fn summarize_omena_query_linear_provenance_semiring_laws() -> ProvenanceSemiringLawReportV0 {
    verify_provenance_semiring_laws_on_fixtures(
        &NaturalCountProvenanceSemiringV0::new(),
        &[0, 1, 2, 3],
    )
}

pub fn round_trip_omena_query_linear_provenance_labels(
    linear_provenance: &OmenaQueryLinearProvenanceV0,
) -> Vec<&'static str> {
    linear_provenance.labels()
}

impl OmenaQueryStyleDiagnosticV0 {
    pub fn linear_provenance(&self) -> OmenaQueryLinearProvenanceV0 {
        summarize_omena_query_linear_provenance(self.provenance.as_slice())
    }

    pub fn polynomial_provenance(&self) -> OmenaQueryPolynomialProvenanceV0 {
        summarize_omena_query_polynomial_provenance(self.provenance.as_slice())
    }
}

pub(crate) fn apply_omena_query_checker_product_gate_to_style_diagnostics(
    diagnostics: &mut [OmenaQueryStyleDiagnosticV0],
) {
    for diagnostic in diagnostics {
        populate_omena_query_checker_product_gate_provenance_from_evidence_graph(
            diagnostic.code,
            &mut diagnostic.provenance,
        );
        diagnostic.polynomial_provenance = Some(diagnostic.polynomial_provenance());
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryStyleDiagnosticsForFileV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub file_uri: String,
    pub file_kind: &'static str,
    pub diagnostic_count: usize,
    pub diagnostics: Vec<OmenaQueryStyleDiagnosticV0>,
    pub ready_surfaces: Vec<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suppression_summary: Option<OmenaQueryDiagnosticSuppressionSummaryV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryDiagnosticSuppressionSummaryV0 {
    pub original_diagnostic_count: usize,
    pub emitted_diagnostic_count: usize,
    pub suppressed_diagnostic_count: usize,
    pub unused_expect_error_count: usize,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub suppression_reasons: Vec<OmenaQueryDiagnosticSuppressionReasonV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryDiagnosticSuppressionReasonV0 {
    pub directive_kind: &'static str,
    pub codes: Vec<String>,
    pub reason: String,
    pub range: ParserRangeV0,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OmenaQueryDiagnosticSuppressionModeV0 {
    Apply,
    ReportOnly,
}

impl OmenaQueryDiagnosticSuppressionModeV0 {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Apply => "apply",
            Self::ReportOnly => "reportOnly",
        }
    }

    pub const fn suppresses_diagnostics(self) -> bool {
        matches!(self, Self::Apply)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryCompletionCandidateV0 {
    pub file_uri: String,
    pub name: String,
    pub kind: &'static str,
    pub range: ParserRangeV0,
    pub source: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub documentation: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryCompletionItemV0 {
    pub label: String,
    pub insert_text: String,
    pub sort_text: String,
    pub detail: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub documentation: Option<String>,
    pub item_kind: &'static str,
    pub ranking_source: &'static str,
    pub source: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryCompletionAtPositionV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub file_uri: String,
    pub file_kind: &'static str,
    pub query_position: ParserPositionV0,
    pub context_kind: &'static str,
    pub prefix: Option<String>,
    pub is_incomplete: bool,
    pub item_count: usize,
    pub items: Vec<OmenaQueryCompletionItemV0>,
    pub ready_surfaces: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryReferenceLocationV0 {
    pub uri: String,
    pub range: ParserRangeV0,
    pub name: String,
    pub role: &'static str,
    pub source: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryRefsForClassV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub selector_name: String,
    pub target_style_uri: Option<String>,
    pub include_declaration: bool,
    pub location_count: usize,
    pub locations: Vec<OmenaQueryReferenceLocationV0>,
    pub ready_surfaces: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryRenamePlanV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub selector_name: String,
    pub new_name: String,
    pub target_style_uri: Option<String>,
    pub edit_count: usize,
    pub edits: Vec<OmenaQueryWorkspaceTextEditV0>,
    pub ready_surfaces: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryCodeActionPlanV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub file_uri: String,
    pub file_kind: &'static str,
    pub action_count: usize,
    pub actions: Vec<OmenaQueryCodeActionV0>,
    pub ready_surfaces: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryCodeActionV0 {
    pub title: String,
    pub kind: &'static str,
    pub edits: Vec<OmenaQueryWorkspaceTextEditV0>,
    pub source: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryStyleInsightsV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub style_uri: String,
    pub insight_count: usize,
    pub insights: Vec<OmenaQueryInsightV0>,
    pub ready_surfaces: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryInsightV0 {
    pub kind: &'static str,
    pub title: String,
    pub message: String,
    pub range: ParserRangeV0,
    pub confidence: &'static str,
    pub scope: &'static str,
    pub source: &'static str,
    pub provenance: Vec<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub primary_edit: Option<OmenaQueryWorkspaceTextEditV0>,
    pub shorthand_combinable: Option<OmenaQueryShorthandCombinableV0>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cascade_insight: Option<OmenaQueryCascadeInsightV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryShorthandCombinableV0 {
    pub shorthand_property: CanonicalStandardPropertyNameV0,
    pub longhand_properties: Vec<CanonicalStandardPropertyNameV0>,
    pub values: Vec<String>,
    pub combined_value: String,
    pub declaration_count: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryCascadeInsightV0 {
    pub relationship: &'static str,
    pub selector: String,
    pub property: AuthoredPropertyTextV0,
    pub related_selector: Option<String>,
    pub related_property: Option<AuthoredPropertyTextV0>,
    pub source_order: u32,
    pub related_source_order: Option<u32>,
}

impl PartialEq for OmenaQueryCascadeInsightV0 {
    fn eq(&self, other: &Self) -> bool {
        self.relationship == other.relationship
            && self.selector == other.selector
            && self
                .property
                .to_property_name()
                .same_as(&other.property.to_property_name())
            && self.related_selector == other.related_selector
            && match (&self.related_property, &other.related_property) {
                (Some(left), Some(right)) => {
                    left.to_property_name().same_as(&right.to_property_name())
                }
                (None, None) => true,
                (Some(_), None) | (None, Some(_)) => false,
            }
            && self.source_order == other.source_order
            && self.related_source_order == other.related_source_order
    }
}

impl Eq for OmenaQueryCascadeInsightV0 {}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryCascadeAtPositionV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub style_path: String,
    pub query_position: ParserPositionV0,
    pub status: &'static str,
    pub cascade_engine: &'static str,
    pub reference_name: Option<String>,
    pub reference_range: Option<ParserRangeV0>,
    pub winner_declaration_source_order: Option<usize>,
    pub winner_declaration_file_path: Option<String>,
    pub winner_declaration_range: Option<ParserRangeV0>,
    pub winner_context_kind: Option<&'static str>,
    /// Opaque cascade ordering token; consumers must not interpret it as a layer-count magnitude.
    pub winner_declaration_layer_rank: Option<i32>,
    pub winner_declaration_layer_name: Option<String>,
    pub candidate_declaration_count: usize,
    pub shadowed_declaration_source_orders: Vec<usize>,
    pub referenced_declaration_property: Option<AuthoredPropertyTextV0>,
    pub referenced_declaration_value: Option<String>,
    pub referenced_declaration_computed_value_status: Option<&'static str>,
    pub referenced_declaration_computed_value: Option<String>,
    pub referenced_declaration_invalid_at_computed_value_time: bool,
    pub referenced_declaration_computed_value_indeterminate: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub referenced_declaration_computed_value_indeterminate_reason: Option<&'static str>,
    pub referenced_declaration_computed_value_derivation_steps: Vec<&'static str>,
    pub custom_property_fixed_point_iteration_count: usize,
    pub custom_property_fixed_point_guaranteed_invalid_count: usize,
    pub reference_custom_property_fixed_point_status: Option<&'static str>,
    pub reference_custom_property_fixed_point_value: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refinement_evidence: Option<CascadeDimensionalRefinementBridgeV0>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub categorical_evidence:
        Option<omena_query_checker_orchestrator::CategoricalCascadeEvidenceV0>,
}

impl PartialEq for OmenaQueryCascadeAtPositionV0 {
    fn eq(&self, other: &Self) -> bool {
        let referenced_property_same = match (
            &self.referenced_declaration_property,
            &other.referenced_declaration_property,
        ) {
            (Some(left), Some(right)) => left.to_property_name().same_as(&right.to_property_name()),
            (None, None) => true,
            (Some(_), None) | (None, Some(_)) => false,
        };
        self.schema_version == other.schema_version
            && self.product == other.product
            && self.style_path == other.style_path
            && self.query_position == other.query_position
            && self.status == other.status
            && self.cascade_engine == other.cascade_engine
            && self.reference_name == other.reference_name
            && self.reference_range == other.reference_range
            && self.winner_declaration_source_order == other.winner_declaration_source_order
            && self.winner_declaration_file_path == other.winner_declaration_file_path
            && self.winner_declaration_range == other.winner_declaration_range
            && self.winner_context_kind == other.winner_context_kind
            && self.winner_declaration_layer_rank == other.winner_declaration_layer_rank
            && self.winner_declaration_layer_name == other.winner_declaration_layer_name
            && self.candidate_declaration_count == other.candidate_declaration_count
            && self.shadowed_declaration_source_orders == other.shadowed_declaration_source_orders
            && referenced_property_same
            && self.referenced_declaration_value == other.referenced_declaration_value
            && self.referenced_declaration_computed_value_status
                == other.referenced_declaration_computed_value_status
            && self.referenced_declaration_computed_value
                == other.referenced_declaration_computed_value
            && self.referenced_declaration_invalid_at_computed_value_time
                == other.referenced_declaration_invalid_at_computed_value_time
            && self.referenced_declaration_computed_value_indeterminate
                == other.referenced_declaration_computed_value_indeterminate
            && self.referenced_declaration_computed_value_indeterminate_reason
                == other.referenced_declaration_computed_value_indeterminate_reason
            && self.referenced_declaration_computed_value_derivation_steps
                == other.referenced_declaration_computed_value_derivation_steps
            && self.custom_property_fixed_point_iteration_count
                == other.custom_property_fixed_point_iteration_count
            && self.custom_property_fixed_point_guaranteed_invalid_count
                == other.custom_property_fixed_point_guaranteed_invalid_count
            && self.reference_custom_property_fixed_point_status
                == other.reference_custom_property_fixed_point_status
            && self.reference_custom_property_fixed_point_value
                == other.reference_custom_property_fixed_point_value
            && self.refinement_evidence == other.refinement_evidence
            && self.categorical_evidence == other.categorical_evidence
    }
}

impl Eq for OmenaQueryCascadeAtPositionV0 {}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryCreateCustomPropertyActionV0 {
    pub uri: String,
    pub range: ParserRangeV0,
    pub new_text: String,
    pub property_name: AuthoredPropertyTextV0,
    pub property_key: CanonicalCustomPropertyNameV0,
}

impl PartialEq for OmenaQueryCreateCustomPropertyActionV0 {
    fn eq(&self, other: &Self) -> bool {
        self.uri == other.uri
            && self.range == other.range
            && self.new_text == other.new_text
            && self.property_key == other.property_key
    }
}

impl Eq for OmenaQueryCreateCustomPropertyActionV0 {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQuerySourceDiagnosticV0 {
    pub code: &'static str,
    pub severity: &'static str,
    pub provenance: Vec<&'static str>,
    pub range: ParserRangeV0,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub precision: Option<OmenaQueryAnalysisPrecisionV0>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggestion: Option<String>,
    pub create_selector: Option<OmenaQueryCreateSelectorActionV0>,
}

impl OmenaQuerySourceDiagnosticV0 {
    pub fn linear_provenance(&self) -> OmenaQueryLinearProvenanceV0 {
        summarize_omena_query_linear_provenance(self.provenance.as_slice())
    }
}

pub(crate) fn source_diagnostic_precision(
    value_domain: ValueDomainPrecisionV1,
    flow: FlowPrecisionV1,
    context: ContextPrecisionV1,
    unresolved_provider_count: usize,
    closed_world: bool,
) -> OmenaQueryAnalysisPrecisionV0 {
    let axes = AnalysisPrecisionV1 {
        value_domain,
        flow,
        context,
        provider_completeness: ProviderCompletenessV1::from_unresolved_count(
            unresolved_provider_count,
        ),
        world_assumption: WorldAssumptionV1::from_closed_world(closed_world),
        revision: RevisionIdentityV1::QuerySourceDiagnosticsInput,
    };
    let precision = source_diagnostic_precision_node(axes);
    OmenaQueryAnalysisPrecisionV0::new(precision.product, precision.axes)
}

pub fn fact_precision_from_evidence_analysis_precision(
    precision: &EvidenceAnalysisPrecisionV0,
) -> omena_query_core::FactPrecision {
    omena_query_core::fact_precision_from_analysis_precision(&OmenaQueryAnalysisPrecisionV0::new(
        precision.product.clone(),
        precision.axes,
    ))
}

pub(crate) const OMENA_QUERY_TSGO_PROVIDER_UNAVAILABLE_PROVENANCE: &str =
    "tsgo-provider.unavailable->unknown-precision";

pub(crate) fn apply_omena_query_checker_product_gate_to_source_diagnostics(
    diagnostics: &mut [OmenaQuerySourceDiagnosticV0],
) {
    for diagnostic in diagnostics {
        populate_omena_query_checker_product_gate_provenance_from_evidence_graph(
            diagnostic.code,
            &mut diagnostic.provenance,
        );
    }
}

pub(crate) fn project_omena_query_provenance_from_evidence_graph(
    provenance: &[&'static str],
) -> Vec<&'static str> {
    let input_identity = provenance.first().copied().unwrap_or("emptyProvenance");
    project_omena_query_diagnostic_provenance_from_evidence_graph(
        input_identity,
        provenance.to_vec(),
    )
}

fn populate_omena_query_checker_product_gate_provenance_from_evidence_graph(
    product_diagnostic_code: &str,
    provenance: &mut Vec<&'static str>,
) {
    let gate =
        omena_query_checker_orchestrator::gate_omena_query_checker_product_diagnostic_code_v0(
            product_diagnostic_code,
        );
    if !gate.enforcement_passed {
        provenance.push("omena-query-checker-orchestrator.product-diagnostic-gate-failed");
    } else {
        for label in gate.provenance {
            if !provenance.contains(&label) {
                provenance.push(label);
            }
        }
    }
    *provenance = project_omena_query_diagnostic_provenance_from_evidence_graph(
        product_diagnostic_code,
        provenance.clone(),
    );
}

fn source_diagnostic_precision_node(axes: AnalysisPrecisionV1) -> EvidenceAnalysisPrecisionV0 {
    let precision = EvidenceAnalysisPrecisionV0::new("omena-query.analysis-precision", axes);
    let input_identity = format!("{:?}", axes.value_domain);
    let Some(node) = project_omena_query_evidence_node(
        "sourceDiagnosticPrecision",
        input_identity.as_str(),
        &[],
        Some(precision.clone()),
    ) else {
        return precision;
    };
    node.precision.unwrap_or(precision)
}

fn project_omena_query_diagnostic_provenance_from_evidence_graph(
    input_identity: &str,
    provenance: Vec<&'static str>,
) -> Vec<&'static str> {
    let Some(node) = project_omena_query_evidence_node(
        "diagnosticProvenance",
        input_identity,
        provenance.as_slice(),
        None,
    ) else {
        return provenance;
    };
    node.provenance
        .iter()
        .filter_map(|label| {
            provenance
                .iter()
                .copied()
                .find(|candidate| *candidate == label.as_str())
        })
        .collect()
}

fn project_omena_query_evidence_node(
    query_identity: &str,
    input_identity: &str,
    provenance: &[&'static str],
    precision: Option<EvidenceAnalysisPrecisionV0>,
) -> Option<omena_evidence_graph::EvidenceNodeV0> {
    let key = EvidenceNodeKeyV0::new(query_identity, input_identity);
    let Ok(graph) = build_evidence_graph_from_edges_v0(
        [EvidenceNodeSeedV0::with_precision(
            key.clone(),
            provenance
                .iter()
                .map(|label| (*label).to_string())
                .collect(),
            precision,
            GuaranteeKindV0::for_label_less_family(),
        )],
        [EvidenceDemandEdgeV0::new(
            query_identity,
            key,
            "diagnostic-evidence",
        )],
    ) else {
        return None;
    };
    graph.nodes.into_iter().next()
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryCreateSelectorActionV0 {
    pub uri: String,
    pub range: ParserRangeV0,
    pub new_text: String,
    pub selector_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQuerySourceMissingSelectorDiagnosticCandidateV0 {
    pub target_style_uri: String,
    pub target_style_source: String,
    pub selector_name: String,
    pub source_reference_range: ParserRangeV0,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQuerySourceDiagnosticsForFileV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub file_uri: String,
    pub file_kind: &'static str,
    pub diagnostic_count: usize,
    pub diagnostics: Vec<OmenaQuerySourceDiagnosticV0>,
    pub ready_surfaces: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQuerySourceSelectorCandidateV0 {
    pub kind: &'static str,
    pub name: String,
    pub range: ParserRangeV0,
    pub source: &'static str,
    pub target_style_uri: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQuerySourceSelectorReferenceCandidateV0 {
    pub uri: String,
    pub kind: &'static str,
    pub name: String,
    pub range: ParserRangeV0,
    pub source: &'static str,
    pub target_style_uri: Option<String>,
}

impl OmenaQuerySourceSelectorReferenceCandidateV0 {
    pub fn projection_surface(&self) -> OmenaQuerySourceSelectorReferenceSurfaceV0 {
        match self.source {
            "omenaTsgoTypeFactProjection" => {
                OmenaQuerySourceSelectorReferenceSurfaceV0::OmenaTsgoTypeFactProjection
            }
            _ => OmenaQuerySourceSelectorReferenceSurfaceV0::OmenaQuerySourceSyntaxIndex,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryStyleSelectorDefinitionV0 {
    pub uri: String,
    pub name: String,
    pub range: ParserRangeV0,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQuerySourceProviderCandidateResolutionV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub matched: Vec<OmenaQuerySourceSelectorCandidateV0>,
    pub unresolved: Vec<OmenaQuerySourceSelectorCandidateV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQuerySourceSelectorReferenceEditTargetV0 {
    pub uri: String,
    pub name: String,
    pub range: ParserRangeV0,
    pub target_style_uri: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQuerySourceSelectorOccurrenceIndexV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub moniker_count: usize,
    pub occurrence_count: usize,
    pub workspace_index: OmenaWorkspaceOccurrenceIndexV0,
    pub occurrences: Vec<OmenaQuerySourceSelectorOccurrenceV0>,
    pub ready_surfaces: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQuerySourceSelectorOccurrenceV0 {
    pub moniker: String,
    pub uri: String,
    pub selector_name: String,
    pub range: ParserRangeV0,
    pub kind: OmenaWorkspaceOccurrenceKindV0,
    pub role: OmenaWorkspaceOccurrenceRoleV0,
    pub source: OmenaWorkspaceOccurrenceSurfaceV0,
    pub target_style_uri: Option<String>,
    pub rename_target: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryCustomPropertyOccurrenceIndexV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub occurrence_count: usize,
    pub occurrences: Vec<OmenaQueryCustomPropertyOccurrenceV0>,
    pub ready_surfaces: Vec<&'static str>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryCustomPropertyOccurrenceV0 {
    pub uri: String,
    pub name: AuthoredPropertyTextV0,
    pub range: ParserRangeV0,
    pub byte_span: ParserByteSpanV0,
    pub kind: &'static str,
    pub has_fallback: bool,
    pub source: &'static str,
    pub property_key: CanonicalCustomPropertyNameV0,
}

impl PartialEq for OmenaQueryCustomPropertyOccurrenceV0 {
    fn eq(&self, other: &Self) -> bool {
        self.uri == other.uri
            && self.property_key == other.property_key
            && self.range == other.range
            && self.byte_span == other.byte_span
            && self.kind == other.kind
            && self.has_fallback == other.has_fallback
            && self.source == other.source
    }
}

impl Eq for OmenaQueryCustomPropertyOccurrenceV0 {}

impl Ord for OmenaQueryCustomPropertyOccurrenceV0 {
    fn cmp(&self, other: &Self) -> Ordering {
        (
            &self.uri,
            &self.property_key,
            self.range,
            self.byte_span,
            self.kind,
            self.has_fallback,
            self.source,
        )
            .cmp(&(
                &other.uri,
                &other.property_key,
                other.range,
                other.byte_span,
                other.kind,
                other.has_fallback,
                other.source,
            ))
    }
}

impl PartialOrd for OmenaQueryCustomPropertyOccurrenceV0 {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaWorkspaceOccurrenceIndexV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub moniker_count: usize,
    pub occurrence_count: usize,
    pub by_moniker: BTreeMap<String, Vec<OmenaWorkspaceOccurrenceV0>>,
    pub by_file: BTreeMap<String, Vec<String>>,
    pub ready_surfaces: Vec<&'static str>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaWorkspaceOccurrenceV0 {
    pub moniker: String,
    pub uri: String,
    pub name: String,
    pub range: ParserRangeV0,
    pub kind: OmenaWorkspaceOccurrenceKindV0,
    pub role: OmenaWorkspaceOccurrenceRoleV0,
    pub surface: OmenaWorkspaceOccurrenceSurfaceV0,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub family: Option<OmenaWorkspaceOccurrenceFamilyV0>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub namespace: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_style_uri: Option<String>,
    pub rename_target: bool,
}

impl PartialEq for OmenaWorkspaceOccurrenceV0 {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl Eq for OmenaWorkspaceOccurrenceV0 {}

impl Ord for OmenaWorkspaceOccurrenceV0 {
    fn cmp(&self, other: &Self) -> Ordering {
        self.moniker
            .cmp(&other.moniker)
            .then_with(|| self.uri.cmp(&other.uri))
            .then_with(|| workspace_occurrence_name_cmp(self, other))
            .then_with(|| self.range.cmp(&other.range))
            .then_with(|| self.kind.cmp(&other.kind))
            .then_with(|| self.role.cmp(&other.role))
            .then_with(|| self.surface.cmp(&other.surface))
            .then_with(|| self.family.cmp(&other.family))
            .then_with(|| self.namespace.cmp(&other.namespace))
            .then_with(|| self.target_style_uri.cmp(&other.target_style_uri))
            .then_with(|| self.rename_target.cmp(&other.rename_target))
    }
}

impl PartialOrd for OmenaWorkspaceOccurrenceV0 {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn workspace_occurrence_name_cmp(
    left: &OmenaWorkspaceOccurrenceV0,
    right: &OmenaWorkspaceOccurrenceV0,
) -> Ordering {
    let left_is_custom_property =
        left.kind.family() == OmenaWorkspaceOccurrenceFamilyV0::CustomProperty;
    let right_is_custom_property =
        right.kind.family() == OmenaWorkspaceOccurrenceFamilyV0::CustomProperty;
    let identity_domain_order = left_is_custom_property.cmp(&right_is_custom_property);
    if identity_domain_order != Ordering::Equal {
        return identity_domain_order;
    }
    if left_is_custom_property {
        return PropertyNameV0::canonical_custom_key(left.name.clone())
            .cmp(&PropertyNameV0::canonical_custom_key(right.name.clone()));
    }
    left.name.cmp(&right.name)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum OmenaWorkspaceOccurrenceKindV0 {
    SourceSelectorReference,
    SourceSelectorPrefixReference,
    CustomPropertyDeclaration,
    CustomPropertyReference,
    SassVariableDeclaration,
    SassVariableReference,
    SassMixinDeclaration,
    SassMixinInclude,
    SassFunctionDeclaration,
    SassFunctionCall,
    SassMixinReference,
    SassFunctionReference,
    SassSymbolDeclaration,
    SassSymbolReference,
}

impl OmenaWorkspaceOccurrenceKindV0 {
    pub fn family(self) -> OmenaWorkspaceOccurrenceFamilyV0 {
        match self {
            Self::SourceSelectorReference | Self::SourceSelectorPrefixReference => {
                OmenaWorkspaceOccurrenceFamilyV0::CssModuleSelector
            }
            Self::CustomPropertyDeclaration | Self::CustomPropertyReference => {
                OmenaWorkspaceOccurrenceFamilyV0::CustomProperty
            }
            Self::SassVariableDeclaration | Self::SassVariableReference => {
                OmenaWorkspaceOccurrenceFamilyV0::Variable
            }
            Self::SassMixinDeclaration | Self::SassMixinInclude | Self::SassMixinReference => {
                OmenaWorkspaceOccurrenceFamilyV0::Mixin
            }
            Self::SassFunctionDeclaration
            | Self::SassFunctionCall
            | Self::SassFunctionReference => OmenaWorkspaceOccurrenceFamilyV0::Function,
            Self::SassSymbolDeclaration | Self::SassSymbolReference => {
                OmenaWorkspaceOccurrenceFamilyV0::Symbol
            }
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::SourceSelectorReference => "sourceSelectorReference",
            Self::SourceSelectorPrefixReference => "sourceSelectorPrefixReference",
            Self::CustomPropertyDeclaration => "customPropertyDeclaration",
            Self::CustomPropertyReference => "customPropertyReference",
            Self::SassVariableDeclaration => "sassVariableDeclaration",
            Self::SassVariableReference => "sassVariableReference",
            Self::SassMixinDeclaration => "sassMixinDeclaration",
            Self::SassMixinInclude => "sassMixinInclude",
            Self::SassMixinReference => "sassMixinReference",
            Self::SassFunctionDeclaration => "sassFunctionDeclaration",
            Self::SassFunctionCall => "sassFunctionCall",
            Self::SassFunctionReference => "sassFunctionReference",
            Self::SassSymbolDeclaration => "sassSymbolDeclaration",
            Self::SassSymbolReference => "sassSymbolReference",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum OmenaWorkspaceOccurrenceRoleV0 {
    Definition,
    Reference,
}

impl OmenaWorkspaceOccurrenceRoleV0 {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Definition => "definition",
            Self::Reference => "reference",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum OmenaWorkspaceOccurrenceSurfaceV0 {
    OmenaQuerySourceSyntaxIndex,
    OmenaLspStyleIndex,
}

impl OmenaWorkspaceOccurrenceSurfaceV0 {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::OmenaQuerySourceSyntaxIndex => "omenaQuerySourceSyntaxIndex",
            Self::OmenaLspStyleIndex => "omenaLspStyleIndex",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum OmenaWorkspaceOccurrenceFamilyV0 {
    CssModuleSelector,
    CustomProperty,
    Variable,
    Mixin,
    Function,
    Symbol,
}

impl OmenaWorkspaceOccurrenceFamilyV0 {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::CssModuleSelector => "cssModuleSelector",
            Self::CustomProperty => "customProperty",
            Self::Variable => "variable",
            Self::Mixin => "mixin",
            Self::Function => "function",
            Self::Symbol => "symbol",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryWorkspaceTextEditV0 {
    pub uri: String,
    pub range: ParserRangeV0,
    pub new_text: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQuerySassModuleUseEdgeV0 {
    pub source: String,
    pub namespace_kind: &'static str,
    pub namespace: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQuerySassModuleSourcesV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub module_use_edges: Vec<OmenaQuerySassModuleUseEdgeV0>,
    pub module_forward_sources: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryStylePackageManifestV0 {
    pub package_json_path: String,
    pub package_json_source: String,
}

#[cfg(test)]
mod evidence_graph_projection_tests {
    use super::*;

    fn custom_property_workspace_occurrence(name: &str) -> OmenaWorkspaceOccurrenceV0 {
        OmenaWorkspaceOccurrenceV0 {
            moniker: "custom-property:file:///workspace/app.css#--token".to_string(),
            uri: "file:///workspace/app.css".to_string(),
            name: name.to_string(),
            range: ParserRangeV0 {
                start: ParserPositionV0 {
                    line: 0,
                    character: 0,
                },
                end: ParserPositionV0 {
                    line: 0,
                    character: 7,
                },
            },
            kind: OmenaWorkspaceOccurrenceKindV0::CustomPropertyDeclaration,
            role: OmenaWorkspaceOccurrenceRoleV0::Definition,
            surface: OmenaWorkspaceOccurrenceSurfaceV0::OmenaLspStyleIndex,
            family: Some(OmenaWorkspaceOccurrenceFamilyV0::CustomProperty),
            namespace: None,
            target_style_uri: None,
            rename_target: true,
        }
    }

    #[test]
    fn workspace_occurrence_identity_uses_custom_property_keys_without_changing_wire_name() {
        let escaped = custom_property_workspace_occurrence(r"--to\6b en");
        let plain = custom_property_workspace_occurrence("--token");

        assert_eq!(escaped, plain);
        let mut non_custom = plain.clone();
        non_custom.kind = OmenaWorkspaceOccurrenceKindV0::SourceSelectorReference;
        non_custom.family = Some(OmenaWorkspaceOccurrenceFamilyV0::CssModuleSelector);
        assert_eq!(escaped.cmp(&non_custom), plain.cmp(&non_custom));

        let mut occurrences = vec![escaped.clone(), plain];
        occurrences.sort();
        occurrences.dedup();
        assert_eq!(occurrences.len(), 1);
        let serialized = serde_json::to_value(&escaped).unwrap_or_default();
        assert_eq!(serialized["name"], r"--to\6b en");
    }

    #[test]
    fn diagnostic_provenance_projection_preserves_legacy_labels() {
        let labels = vec![
            "omena-query.source-syntax-index",
            "omena-query.style-selector-definitions",
        ];

        assert_eq!(
            project_omena_query_diagnostic_provenance_from_evidence_graph(
                "missingSelector",
                labels.clone(),
            ),
            labels
        );
    }

    #[test]
    fn checker_product_gate_projection_matches_legacy_extension() {
        let code = "missingSelector";
        let mut expected = vec![
            "omena-query.source-syntax-index",
            "omena-query.style-selector-definitions",
        ];
        let gate =
            omena_query_checker_orchestrator::gate_omena_query_checker_product_diagnostic_code_v0(
                code,
            );
        if !gate.enforcement_passed {
            expected.push("omena-query-checker-orchestrator.product-diagnostic-gate-failed");
        } else {
            for label in gate.provenance {
                if !expected.contains(&label) {
                    expected.push(label);
                }
            }
        }

        let mut actual = vec![
            "omena-query.source-syntax-index",
            "omena-query.style-selector-definitions",
        ];
        populate_omena_query_checker_product_gate_provenance_from_evidence_graph(code, &mut actual);

        assert_eq!(actual, expected);
    }

    #[test]
    fn source_diagnostic_precision_projects_typed_wire_shape() -> Result<(), serde_json::Error> {
        let precision = source_diagnostic_precision(
            ValueDomainPrecisionV1::ClassValueResolution,
            FlowPrecisionV1::SourceSyntaxIndex,
            ContextPrecisionV1::PerSourceReference,
            0,
            true,
        );
        let serialized = serde_json::to_value(&precision)?;

        assert_eq!(
            serialized,
            serde_json::json!({
                "product": "omena-query.analysis-precision",
                "valueDomain": "classValueResolution",
                "flowSensitivity": "sourceSyntaxIndex",
                "contextSensitivity": "perSourceReference",
                "providerCompleteness": "complete",
                "worldAssumption": "closed",
                "revisionAxis": "OmenaQuerySourceDiagnosticsForFileV0.input"
            })
        );
        Ok(())
    }
}
