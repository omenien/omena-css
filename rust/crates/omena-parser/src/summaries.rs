use omena_interner::NameKind;
use omena_syntax::{StyleDialect, SyntaxKind};
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParserBoundarySummary {
    pub product: &'static str,
    pub tree_model: &'static str,
    pub parser_track: &'static str,
    pub dialect_count: usize,
    pub shared_name_kind_count: usize,
    pub ready_surfaces: Vec<&'static str>,
    pub not_ready_surfaces: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParserSemanticNameConsumptionSummaryV0 {
    pub product: &'static str,
    pub dialect: StyleDialect,
    pub semantic_name_count: usize,
    pub interned_name_count: usize,
    pub invalid_name_count: usize,
    pub class_name_count: usize,
    pub css_ident_count: usize,
    pub property_name_count: usize,
    pub selector_key_count: usize,
    pub custom_property_name_count: usize,
    pub keyframes_name_count: usize,
    pub mixin_name_count: usize,
    pub file_path_count: usize,
    pub ready_surfaces: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParserCstEquivalenceSummaryV0 {
    pub product: &'static str,
    pub dialect: StyleDialect,
    pub root_kind: SyntaxKind,
    pub parser_node_count: usize,
    pub parser_token_count: usize,
    pub typed_wrapper_count: usize,
    pub source_text_round_trip_ready: bool,
    pub syntax_kind_round_trip_ready: bool,
    pub zero_unknown_kind_ready: bool,
    pub typed_cst_wrapper_ready: bool,
    pub ready_surfaces: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParserPrattValueCoverageSummaryV0 {
    pub product: &'static str,
    pub infix_operator_kinds: Vec<SyntaxKind>,
    pub prefix_operator_kinds: Vec<SyntaxKind>,
    pub value_expression_node_kinds: Vec<SyntaxKind>,
    pub specialized_function_family_count: usize,
    pub css_values_l4_math_function_count: usize,
    pub css_color_function_count: usize,
    pub ready_surfaces: Vec<&'static str>,
    pub next_surfaces: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParserRecursiveDescentCoverageSummaryV0 {
    pub product: &'static str,
    pub dialect_count: usize,
    pub entry_point_count: usize,
    pub selector_surface_count: usize,
    pub at_rule_surface_count: usize,
    pub dialect_extension_surface_count: usize,
    pub recovery_surface_count: usize,
    pub ready_surfaces: Vec<&'static str>,
    pub next_surfaces: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ParserSemanticNameCandidateV0 {
    pub(crate) kind: NameKind,
    pub(crate) text: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaParserStyleFactsSummaryV0 {
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
    pub css_module_value_import_edges: Vec<OmenaParserCssModuleValueImportEdgeFactV0>,
    pub css_module_value_definition_edges: Vec<OmenaParserCssModuleValueDefinitionEdgeFactV0>,
    pub css_module_composes_target_names: Vec<String>,
    pub css_module_composes_import_sources: Vec<String>,
    pub css_module_composes_edges: Vec<OmenaParserCssModuleComposesEdgeFactV0>,
    pub icss_export_names: Vec<String>,
    pub icss_import_local_names: Vec<String>,
    pub icss_import_remote_names: Vec<String>,
    pub icss_import_sources: Vec<String>,
    pub icss_import_edges: Vec<OmenaParserIcssImportEdgeFactV0>,
    pub icss_export_edges: Vec<OmenaParserIcssExportEdgeFactV0>,
    pub variable_names: Vec<String>,
    pub sass_symbol_declaration_names: Vec<String>,
    pub sass_symbol_reference_names: Vec<String>,
    pub sass_symbol_facts: Vec<OmenaParserSassSymbolFactV0>,
    pub sass_symbol_resolution: OmenaParserSassSymbolResolutionV0,
    pub sass_module_use_sources: Vec<String>,
    pub sass_module_forward_sources: Vec<String>,
    pub sass_module_import_sources: Vec<String>,
    pub sass_module_edges: Vec<OmenaParserSassModuleEdgeFactV0>,
    pub custom_property_names: Vec<String>,
    pub custom_property_decl_names: Vec<String>,
    pub custom_property_ref_names: Vec<String>,
    pub at_rule_names: Vec<String>,
    pub parser_error_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaParserCssModuleValueImportEdgeFactV0 {
    pub remote_name: String,
    pub local_name: String,
    pub import_source: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaParserCssModuleValueDefinitionEdgeFactV0 {
    pub definition_name: String,
    pub reference_names: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaParserCssModuleComposesEdgeFactV0 {
    pub kind: &'static str,
    pub owner_selector_names: Vec<String>,
    pub target_names: Vec<String>,
    pub import_source: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaParserIcssImportEdgeFactV0 {
    pub local_name: String,
    pub remote_name: String,
    pub import_source: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaParserIcssExportEdgeFactV0 {
    pub export_name: String,
    pub reference_names: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaParserSassSymbolFactV0 {
    pub kind: &'static str,
    pub symbol_kind: &'static str,
    pub name: String,
    pub role: &'static str,
    pub namespace: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaParserSassModuleEdgeFactV0 {
    pub kind: &'static str,
    pub source: String,
    pub namespace_kind: Option<&'static str>,
    pub namespace: Option<String>,
    pub visibility_filter_kind: Option<&'static str>,
    pub visibility_filter_names: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaParserSassSymbolResolutionV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub resolution_scope: &'static str,
    pub declaration_count: usize,
    pub reference_count: usize,
    pub resolved_reference_count: usize,
    pub unresolved_reference_count: usize,
    pub edges: Vec<OmenaParserSassSymbolResolutionEdgeV0>,
    pub capabilities: OmenaParserSassSymbolResolutionCapabilitiesV0,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaParserSassSymbolResolutionEdgeV0 {
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
pub struct OmenaParserSassSymbolResolutionCapabilitiesV0 {
    pub same_file_lexical_resolution_ready: bool,
    pub declaration_before_reference_ready: bool,
    pub unresolved_reference_reporting_ready: bool,
    pub cross_file_module_resolution_ready: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaParserLexSummaryV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub dialect: &'static str,
    pub tokens: Vec<OmenaParserLexTokenV0>,
    pub parser_error_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaParserLexTokenV0 {
    pub kind: String,
    pub text: String,
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaParserParityLiteSummaryV0 {
    pub schema_version: &'static str,
    pub language: &'static str,
    pub selector_names: Vec<String>,
    pub keyframes_names: Vec<String>,
    pub value_decl_names: Vec<String>,
    pub diagnostic_count: usize,
    pub rule_count: usize,
    pub declaration_count: usize,
    pub grouped_selector_count: usize,
    pub max_nesting_depth: usize,
    pub at_rule_kind_counts: OmenaParserAtRuleKindCountsV0,
    pub declaration_kind_counts: OmenaParserDeclarationKindCountsV0,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaParserAtRuleKindCountsV0 {
    pub media: usize,
    pub supports: usize,
    pub layer: usize,
    pub keyframes: usize,
    pub value: usize,
    pub at_root: usize,
    pub generic: usize,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaParserDeclarationKindCountsV0 {
    pub composes: usize,
    pub animation: usize,
    pub animation_name: usize,
    pub generic: usize,
}
