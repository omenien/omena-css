use super::*;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaResolverBoundarySummaryV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub resolver_name: &'static str,
    pub input_version: String,
    pub delegated_source_resolution_products: Vec<&'static str>,
    pub resolver_owned_products: Vec<&'static str>,
    pub source_resolution_query_count: usize,
    pub source_resolution_candidate_count: usize,
    pub source_resolution_evaluator_candidate_count: usize,
    pub module_graph_module_count: usize,
    pub module_graph_source_expression_edge_count: usize,
    pub runtime_query_module_count: usize,
    pub runtime_query_ready_module_count: usize,
    pub source_resolution_runtime_expression_count: usize,
    pub source_resolution_runtime_resolved_expression_count: usize,
    pub ready_surfaces: Vec<&'static str>,
    pub cme_coupled_surfaces: Vec<&'static str>,
    pub next_decoupling_targets: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaResolverModuleGraphSummaryV0 {
    pub schema_version: String,
    pub product: String,
    pub input_version: String,
    pub module_count: usize,
    pub source_expression_edge_count: usize,
    pub type_fact_edge_count: usize,
    pub selector_count: usize,
    pub unresolved_type_fact_count: usize,
    pub modules: Vec<OmenaResolverModuleGraphModuleV0>,
    pub unresolved_type_fact_expression_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaResolverModuleGraphModuleV0 {
    pub style_file_path: String,
    pub source_expression_ids: Vec<String>,
    pub source_expression_kinds: Vec<String>,
    pub type_fact_expression_ids: Vec<String>,
    pub selector_names: Vec<String>,
    pub canonical_selector_names: Vec<String>,
    pub has_source_input: bool,
    pub has_style_input: bool,
    pub has_type_fact_input: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaResolverRuntimeQueryBoundarySummaryV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub input_product: String,
    pub input_version: String,
    pub module_query_count: usize,
    pub fully_resolvable_module_count: usize,
    pub source_only_module_count: usize,
    pub style_only_module_count: usize,
    pub unresolved_type_fact_count: usize,
    pub runtime_capabilities: Vec<&'static str>,
    pub blocking_gaps: Vec<&'static str>,
    pub module_queries: Vec<OmenaResolverRuntimeModuleQueryV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaResolverRuntimeModuleQueryV0 {
    pub style_file_path: String,
    pub source_expression_ids: Vec<String>,
    pub type_fact_expression_ids: Vec<String>,
    pub selector_names: Vec<String>,
    pub canonical_selector_names: Vec<String>,
    pub can_resolve_source_expressions: bool,
    pub can_check_type_fact_edges: bool,
    pub can_query_selector_names: bool,
    pub status: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaResolverSourceResolutionRuntimeIndexV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub input_product: &'static str,
    pub input_version: String,
    pub expression_count: usize,
    pub resolved_expression_count: usize,
    pub unresolved_expression_count: usize,
    pub blocking_gaps: Vec<&'static str>,
    pub entries: Vec<OmenaResolverSourceResolutionRuntimeEntryV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaResolverSourceResolutionRuntimeEntryV0 {
    pub query_id: String,
    pub expression_id: String,
    pub expression_kind: String,
    pub style_file_path: String,
    pub selector_names: Vec<String>,
    pub finite_values: Option<Vec<String>>,
    pub selector_certainty: String,
    pub value_certainty: Option<String>,
    pub selector_certainty_shape_kind: String,
    pub value_certainty_shape_kind: String,
    pub has_selector_match: bool,
    pub has_finite_values: bool,
    pub can_resolve_source_expression: bool,
    pub status: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaResolverStylePackageManifestV0 {
    pub package_json_path: String,
    pub package_json_source: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaResolverTsconfigPathMappingV0 {
    pub base_path: String,
    pub pattern: String,
    pub target_patterns: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaResolverBundlerPathAliasMappingV0 {
    pub pattern: String,
    pub target_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaResolverStyleModuleDiskCandidateIdentityV0 {
    pub style_path: String,
    pub metadata_identity: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OmenaResolverStyleModuleConfirmationOptionsV0<'a> {
    pub allow_disk_confirmation: bool,
    pub allow_live_disk_confirmation: bool,
    pub max_disk_candidate_count: usize,
    pub allow_unconfirmed_indexable_candidate: bool,
    pub identity_index: Option<&'a OmenaResolverStyleModuleConfirmationIdentityIndexV0>,
}

impl<'a> Default for OmenaResolverStyleModuleConfirmationOptionsV0<'a> {
    fn default() -> Self {
        Self {
            allow_disk_confirmation: false,
            allow_live_disk_confirmation: false,
            max_disk_candidate_count: 64,
            allow_unconfirmed_indexable_candidate: false,
            identity_index: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OmenaResolverStyleModuleConfirmationIdentityIndexV0 {
    pub available_by_identity: BTreeMap<String, String>,
    pub disk_by_identity: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaResolverStyleModuleCandidateConfirmationV0 {
    pub resolved_style_path: Option<String>,
    pub confirmation_kind: &'static str,
    pub disk_candidate_count: usize,
    pub candidate_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaResolverStyleModuleResolutionV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub from_style_path: String,
    pub source: String,
    pub resolved_style_path: Option<String>,
    pub candidate_count: usize,
    pub candidates: Vec<String>,
    pub resolution_kind: &'static str,
    pub symlink_chain: OmenaResolverSymlinkChainInspectionV0,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaResolverSymlinkChainInspectionV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub requested_path: String,
    pub inspected_component_count: usize,
    pub link_count: usize,
    pub links: Vec<OmenaResolverSymlinkChainLinkV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaResolverSymlinkChainLinkV0 {
    pub link_path: String,
    pub target_path: String,
    pub target_was_absolute: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaResolverSpecifierResolutionRuntimeV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub from_style_path: String,
    pub specifier_count: usize,
    pub resolved_specifier_count: usize,
    pub external_specifier_count: usize,
    pub unresolved_specifier_count: usize,
    pub entries: Vec<OmenaResolverSpecifierResolutionRuntimeEntryV0>,
    pub ready_surfaces: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaResolverSpecifierResolutionRuntimeEntryV0 {
    pub source: String,
    pub resolved_style_path: Option<String>,
    pub candidate_count: usize,
    pub resolution_kind: &'static str,
    pub status: &'static str,
}
