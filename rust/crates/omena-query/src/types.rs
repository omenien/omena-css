use super::*;
use omena_evidence_graph::{
    EvidenceAnalysisPrecisionV0, EvidenceDemandEdgeV0, EvidenceNodeKeyV0, EvidenceNodeSeedV0,
    GuaranteeKindV0, build_evidence_graph_from_edges_v0,
};
use omena_sif::OmenaSifV1;
use std::collections::BTreeMap;

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

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryStyleDocumentSummaryV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub language: &'static str,
    pub selector_names: Vec<String>,
    pub custom_property_decl_names: Vec<String>,
    pub custom_property_ref_names: Vec<String>,
    pub sass_module_use_sources: Vec<String>,
    pub sass_module_forward_sources: Vec<String>,
    pub diagnostic_count: usize,
}

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

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryCustomPropertyAnnotationV0 {
    pub name: String,
    pub declaration_count: usize,
    pub reference_count: usize,
    pub annotation_kind: &'static str,
    pub participates_in_fixed_point: bool,
}

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
    #[default]
    ImportInlineLegacy,
    LinkedOrder,
}

impl OmenaQueryBundleEmissionPathV0 {
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

#[cfg(feature = "lawvere-trace")]
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryLawvereTransformExecuteSummaryV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub product_scope: &'static str,
    pub default_product_mechanism: bool,
    pub global_transform_theorem_claimed: bool,
    pub execution: OmenaQueryTransformExecuteSummaryV0,
    pub lawvere_trace: OmenaQueryLawvereModelTraceV0,
    pub parallel_plan: OmenaQueryLawvereTransformPassParallelPlanV0,
    pub reorderability_certificates: Vec<OmenaQueryLawvereReorderabilityCertificateV0>,
    pub differential_witnesses: Vec<OmenaQueryLawvereDifferentialCommutativityWitnessV0>,
    pub ready_surfaces: Vec<&'static str>,
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
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
    pub custom_property_names: Vec<String>,
    pub custom_property_decl_names: Vec<String>,
    pub custom_property_ref_names: Vec<String>,
    pub at_rule_names: Vec<String>,
    pub parser_error_count: usize,
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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryStyleHoverCandidateV0 {
    pub kind: &'static str,
    pub name: String,
    pub range: ParserRangeV0,
    pub source: &'static str,
    pub namespace: Option<String>,
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryCascadeNarrowingEvidenceV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub selector: String,
    pub selector_class_names: Vec<String>,
    pub property_name: String,
    pub condition_context: Vec<String>,
    pub declaration_ids: Vec<String>,
    pub element_class_iteration: ReducedClassValueProductIterationV0,
    pub property_value_narrowing: AbstractPropertyValueNarrowingV0,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runtime_state: Option<OmenaQueryRuntimeStateScenarioEvidenceV0>,
}

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OmenaQueryRuntimeStateScenarioEvidenceV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub selector: String,
    pub selector_class_names: Vec<String>,
    pub property_name: String,
    pub scenario_join_kind: &'static str,
    pub confidence_tier: &'static str,
    pub confidence_tier_within_modeled_environment: &'static str,
    pub static_boundary: OmenaQueryRuntimeStateStaticBoundaryV0,
    pub driver_summaries: Vec<OmenaQueryRuntimeStateDriverSummaryV0>,
    pub scenarios: Vec<OmenaQueryRuntimeStateScenarioV0>,
    pub static_condition_pruning: Vec<OmenaQueryStaticConditionPruningEvidenceV0>,
    pub inline_style_overrides: Vec<OmenaQueryInlineStyleRuntimeOverrideV0>,
}

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

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryInlineStyleRuntimeOverrideV0 {
    pub source_path: String,
    pub range: ParserRangeV0,
    pub property_name: String,
    pub value: Option<String>,
    pub cascade_tier: &'static str,
    pub static_value: bool,
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
    pub shorthand_property: String,
    pub longhand_properties: Vec<String>,
    pub values: Vec<String>,
    pub combined_value: String,
    pub declaration_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryCascadeInsightV0 {
    pub relationship: &'static str,
    pub selector: String,
    pub property: String,
    pub related_selector: Option<String>,
    pub related_property: Option<String>,
    pub source_order: u32,
    pub related_source_order: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
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
    pub winner_declaration_layer_rank: Option<i32>,
    pub winner_declaration_layer_name: Option<String>,
    pub candidate_declaration_count: usize,
    pub shadowed_declaration_source_orders: Vec<usize>,
    pub referenced_declaration_property: Option<String>,
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryCreateCustomPropertyActionV0 {
    pub uri: String,
    pub range: ParserRangeV0,
    pub new_text: String,
    pub property_name: String,
}

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
    value_domain: &str,
    flow_sensitivity: &str,
    context_sensitivity: &str,
) -> OmenaQueryAnalysisPrecisionV0 {
    let precision =
        source_diagnostic_precision_node(value_domain, flow_sensitivity, context_sensitivity);
    OmenaQueryAnalysisPrecisionV0 {
        product: precision.product,
        value_domain: precision.value_domain,
        flow_sensitivity: precision.flow_sensitivity,
        context_sensitivity: precision.context_sensitivity,
        revision_axis: precision.revision_axis,
    }
}

pub fn fact_precision_from_evidence_analysis_precision(
    precision: &EvidenceAnalysisPrecisionV0,
) -> omena_query_core::FactPrecision {
    omena_query_core::fact_precision_from_analysis_precision(&OmenaQueryAnalysisPrecisionV0 {
        product: precision.product.clone(),
        value_domain: precision.value_domain.clone(),
        flow_sensitivity: precision.flow_sensitivity.clone(),
        context_sensitivity: precision.context_sensitivity.clone(),
        revision_axis: precision.revision_axis.clone(),
    })
}

pub(crate) const OMENA_QUERY_TYPE_ORACLE_UNKNOWN_VALUE_DOMAIN: &str = "unknown";
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

fn source_diagnostic_precision_node(
    value_domain: &str,
    flow_sensitivity: &str,
    context_sensitivity: &str,
) -> EvidenceAnalysisPrecisionV0 {
    let precision = EvidenceAnalysisPrecisionV0::new(
        "omena-query.analysis-precision",
        value_domain,
        flow_sensitivity,
        context_sensitivity,
        "OmenaQuerySourceDiagnosticsForFileV0.input",
    );
    let Some(node) = project_omena_query_evidence_node(
        "sourceDiagnosticPrecision",
        value_domain,
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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryCustomPropertyOccurrenceV0 {
    pub uri: String,
    pub name: String,
    pub range: ParserRangeV0,
    pub byte_span: ParserByteSpanV0,
    pub kind: &'static str,
    pub has_fallback: bool,
    pub source: &'static str,
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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
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
}

impl OmenaWorkspaceOccurrenceKindV0 {
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
            Self::SassFunctionDeclaration => "sassFunctionDeclaration",
            Self::SassFunctionCall => "sassFunctionCall",
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
    fn source_diagnostic_precision_projects_byte_identical_shape() -> Result<(), serde_json::Error>
    {
        let precision = source_diagnostic_precision(
            "classValueResolution",
            "sourceSyntaxIndex",
            "perSourceReference",
        );
        let serialized = serde_json::to_value(&precision)?;

        assert_eq!(
            serialized,
            serde_json::json!({
                "product": "omena-query.analysis-precision",
                "valueDomain": "classValueResolution",
                "flowSensitivity": "sourceSyntaxIndex",
                "contextSensitivity": "perSourceReference",
                "revisionAxis": "OmenaQuerySourceDiagnosticsForFileV0.input"
            })
        );
        Ok(())
    }
}
