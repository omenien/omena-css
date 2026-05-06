use super::*;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryBoundarySummaryV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub query_engine_name: &'static str,
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
pub struct OmenaQueryFragmentBundleV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub input_version: String,
    pub expression_semantics: ExpressionSemanticsQueryFragmentsV0,
    pub source_resolution: SourceResolutionQueryFragmentsV0,
    pub selector_usage: SelectorUsageQueryFragmentsV0,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SelectedQueryAdapterCapabilitiesV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub default_candidate_backend: &'static str,
    pub backend_kinds: Vec<SelectedQueryBackendCapabilityV0>,
    pub runner_commands: Vec<SelectedQueryRunnerCommandV0>,
    pub expression_semantics_payload_contracts: Vec<&'static str>,
    pub required_input_contracts: Vec<&'static str>,
    pub adapter_readiness: Vec<&'static str>,
    pub routing_status: &'static str,
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
    pub css_modules_resolution: OmenaQueryCssModulesCrossFileResolutionV0,
    pub graphs: Vec<OmenaQueryStyleSemanticGraphBatchEntryV0>,
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
    pub composes_cycle_count: usize,
    pub composes_closure_edges: Vec<OmenaQueryCssModulesComposesClosureEdgeV0>,
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
    pub cycle_detection_ready: bool,
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
    pub css_module_composes_target_names: Vec<String>,
    pub css_module_composes_import_sources: Vec<String>,
    pub css_module_composes_edges: Vec<OmenaQueryCssModuleComposesEdgeFactV0>,
    pub icss_export_names: Vec<String>,
    pub icss_import_local_names: Vec<String>,
    pub icss_import_remote_names: Vec<String>,
    pub icss_import_sources: Vec<String>,
    pub icss_import_edges: Vec<OmenaQueryIcssImportEdgeFactV0>,
    pub variable_names: Vec<String>,
    pub custom_property_names: Vec<String>,
    pub at_rule_names: Vec<String>,
    pub parser_error_count: usize,
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
    pub range: ParserRangeV0,
    pub message: String,
    pub create_custom_property: Option<OmenaQueryCreateCustomPropertyActionV0>,
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
    pub candidate_declaration_count: usize,
    pub shadowed_declaration_source_orders: Vec<usize>,
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
    pub range: ParserRangeV0,
    pub message: String,
    pub create_selector: Option<OmenaQueryCreateSelectorActionV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryCreateSelectorActionV0 {
    pub uri: String,
    pub range: ParserRangeV0,
    pub new_text: String,
    pub selector_name: String,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OmenaQueryStylePackageManifestV0 {
    pub package_json_path: String,
    pub package_json_source: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryExpressionDomainIncrementalFlowAnalysisV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub input_version: String,
    pub revision: u64,
    pub graph_count: usize,
    pub dirty_graph_count: usize,
    pub reused_graph_count: usize,
    pub analyses: Vec<OmenaQueryExpressionDomainIncrementalFlowAnalysisEntryV0>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryExpressionDomainIncrementalFlowAnalysisEntryV0 {
    pub graph_id: String,
    pub file_path: String,
    pub analysis: ClassValueFlowIncrementalAnalysisV0,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryExpressionDomainSelectorProjectionV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub input_version: String,
    pub projection_count: usize,
    pub projections: Vec<OmenaQueryExpressionDomainSelectorProjectionEntryV0>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryExpressionDomainSelectorProjectionEntryV0 {
    pub graph_id: String,
    pub file_path: String,
    pub node_id: String,
    pub target_style_paths: Vec<String>,
    pub value_kind: &'static str,
    pub selector_names: Vec<String>,
    pub certainty: SelectorProjectionCertaintyV0,
}

#[derive(Default)]
pub struct OmenaQueryExpressionDomainFlowRuntimeV0 {
    pub(crate) revision: u64,
    pub(crate) databases_by_graph_id: BTreeMap<String, OmenaIncrementalDatabaseV0>,
    pub(crate) previous_analyses_by_graph_id: BTreeMap<String, ClassValueFlowAnalysisV0>,
}
