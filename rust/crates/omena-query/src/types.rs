use super::*;
use omena_sif::OmenaSifV1;

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
    pub cme_coupled_surfaces: Vec<&'static str>,
    pub next_decoupling_targets: Vec<&'static str>,
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
pub struct OmenaQueryCrossFileSummaryV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub status: &'static str,
    pub summary_scope: &'static str,
    pub style_count: usize,
    pub summary_edge_count: usize,
    pub edge_kind_counts: Vec<OmenaQueryCrossFileSummaryEdgeKindCountV0>,
    pub summary_hash: String,
    pub edges: Vec<OmenaQueryCrossFileSummaryEdgeV0>,
    pub capabilities: OmenaQueryCrossFileSummaryCapabilitiesV0,
    pub next_priorities: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryCrossFileSummaryEdgeKindCountV0 {
    pub edge_kind: &'static str,
    pub count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryCrossFileSummaryEdgeV0 {
    pub edge_id: String,
    pub edge_kind: &'static str,
    pub from_kind: &'static str,
    pub from_path: String,
    pub target_kind: Option<&'static str>,
    pub target_path: Option<String>,
    pub source: Option<String>,
    pub owner_selector_name: Option<String>,
    pub local_name: Option<String>,
    pub remote_name: Option<String>,
    pub target_names: Vec<String>,
    pub status: &'static str,
    pub provenance: Vec<&'static str>,
    pub linear_provenance: OmenaQueryLinearProvenanceV0,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryCrossFileSummaryCapabilitiesV0 {
    pub css_modules_composes_edges_ready: bool,
    pub css_modules_value_edges_ready: bool,
    pub css_modules_icss_edges_ready: bool,
    pub sass_module_edges_ready: bool,
    pub style_design_token_reference_edges_ready: bool,
    pub source_selector_reference_edges_ready: bool,
    pub stable_summary_hash_ready: bool,
    pub linear_provenance_ready: bool,
    pub linear_provenance_round_trip_ready: bool,
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
    pub import_source_resolution_ready: bool,
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
    pub namespace_kind: Option<&'static str>,
    pub namespace: Option<String>,
    pub forward_prefix: Option<String>,
    pub visibility_filter_kind: Option<&'static str>,
    pub visibility_filter_names: Vec<String>,
    pub resolved_style_path: Option<String>,
    pub status: &'static str,
    pub resolution_kind: &'static str,
    pub candidate_count: usize,
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
    pub target_query: Option<OmenaQueryTransformTargetQueryPlanV0>,
    pub unknown_pass_ids: Vec<String>,
    pub execution: TransformExecutionSummaryV0,
    pub semantic_removal_count: usize,
    pub ready_surfaces: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryTransformPassSummaryV0 {
    pub id: &'static str,
    pub title: &'static str,
    pub reads_semantic_graph: bool,
    pub reads_cascade_model: bool,
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
    pub closed_style_world: bool,
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
    pub render_source: &'static str,
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
}

pub type OmenaQueryLinearProvenanceV0 = LinearProvenanceV0<Lin01ProvenanceSemiringV0>;

pub fn summarize_omena_query_linear_provenance(
    provenance: &[&'static str],
) -> OmenaQueryLinearProvenanceV0 {
    OmenaQueryLinearProvenanceV0::from_static_labels(provenance)
}

pub fn round_trip_omena_query_linear_provenance_labels(
    linear_provenance: &OmenaQueryLinearProvenanceV0,
) -> Vec<&'static str> {
    linear_provenance.labels()
}

impl OmenaQueryCrossFileSummaryEdgeV0 {
    pub fn linear_provenance_round_trips_legacy_labels(&self) -> bool {
        round_trip_omena_query_linear_provenance_labels(&self.linear_provenance) == self.provenance
    }
}

impl OmenaQueryCrossFileSummaryV0 {
    pub fn linear_provenance_round_trips_legacy_labels(&self) -> bool {
        self.edges
            .iter()
            .all(OmenaQueryCrossFileSummaryEdgeV0::linear_provenance_round_trips_legacy_labels)
    }
}

impl OmenaQueryStyleDiagnosticV0 {
    pub fn linear_provenance(&self) -> OmenaQueryLinearProvenanceV0 {
        summarize_omena_query_linear_provenance(self.provenance.as_slice())
    }
}

pub(crate) fn apply_omena_query_checker_product_gate_to_style_diagnostics(
    diagnostics: &mut [OmenaQueryStyleDiagnosticV0],
) {
    for diagnostic in diagnostics {
        extend_omena_query_checker_product_gate_provenance(
            diagnostic.code,
            &mut diagnostic.provenance,
        );
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
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryCompletionCandidateV0 {
    pub file_uri: String,
    pub name: String,
    pub kind: &'static str,
    pub range: ParserRangeV0,
    pub source: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryCompletionItemV0 {
    pub label: String,
    pub insert_text: String,
    pub sort_text: String,
    pub detail: &'static str,
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
    pub referenced_declaration_computed_value_derivation_steps: Vec<&'static str>,
    pub custom_property_fixed_point_iteration_count: usize,
    pub custom_property_fixed_point_guaranteed_invalid_count: usize,
    pub reference_custom_property_fixed_point_status: Option<&'static str>,
    pub reference_custom_property_fixed_point_value: Option<String>,
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
    pub create_selector: Option<OmenaQueryCreateSelectorActionV0>,
}

impl OmenaQuerySourceDiagnosticV0 {
    pub fn linear_provenance(&self) -> OmenaQueryLinearProvenanceV0 {
        summarize_omena_query_linear_provenance(self.provenance.as_slice())
    }
}

pub(crate) fn apply_omena_query_checker_product_gate_to_source_diagnostics(
    diagnostics: &mut [OmenaQuerySourceDiagnosticV0],
) {
    for diagnostic in diagnostics {
        extend_omena_query_checker_product_gate_provenance(
            diagnostic.code,
            &mut diagnostic.provenance,
        );
    }
}

fn extend_omena_query_checker_product_gate_provenance(
    product_diagnostic_code: &str,
    provenance: &mut Vec<&'static str>,
) {
    let gate =
        omena_query_checker_orchestrator::gate_omena_query_checker_product_diagnostic_code_v0(
            product_diagnostic_code,
        );
    if !gate.enforcement_passed {
        provenance.push("omena-query-checker-orchestrator.product-diagnostic-gate-failed");
        return;
    }
    for label in gate.provenance {
        if !provenance.contains(&label) {
            provenance.push(label);
        }
    }
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
