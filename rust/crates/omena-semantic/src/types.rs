//! Shared semantic data contracts.
//!
//! These V0 structs are the stable schema that parser, semantic, query,
//! checker, and LSP gates exchange. They intentionally stay serializable and
//! explicit so product-facing diagnostics can be checked without touching
//! internal parser state.

use omena_parser::StyleDialect;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stylesheet {
    pub path: String,
    pub language: StyleDialect,
    pub source: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ParserBoundarySyntaxFactsV0 {
    pub lossless_cst: ParserLosslessCstFactsV0,
    pub selectors: ParserIndexSelectorFactsV0,
    pub values: ParserIndexValueFactsV0,
    pub custom_properties: ParserIndexCustomPropertyFactsV0,
    pub sass: ParserSassSyntaxFactsV0,
    pub keyframes: ParserIndexKeyframesFactsV0,
    pub composes: ParserIndexComposesFactsV0,
    pub wrappers: ParserIndexWrapperFactsV0,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ParserLosslessCstFactsV0 {
    pub source_byte_len: usize,
    pub token_count: usize,
    pub root_node_count: usize,
    pub diagnostic_count: usize,
    pub all_token_spans_within_source: bool,
    pub all_node_spans_within_source: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ParserIndexSelectorFactsV0 {
    pub names: Vec<String>,
    pub definition_facts: Vec<ParserIndexSelectorDefinitionFactV0>,
    pub bem_suffix_parent_names: Vec<String>,
    pub bem_suffix_safe_names: Vec<String>,
    pub nested_unsafe_names: Vec<String>,
    pub selectors_with_value_refs_names: Vec<String>,
    pub selectors_with_animation_ref_names: Vec<String>,
    pub selectors_with_animation_name_ref_names: Vec<String>,
    pub bem_suffix_count: usize,
    pub nested_safety_counts: NestedSafetyCountsV0,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ParserIndexSelectorDefinitionFactV0 {
    pub name: String,
    pub source_order: usize,
    pub byte_span: ParserByteSpanV0,
    pub range: ParserRangeV0,
    pub nested_safety_kind: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bem_suffix_parent_name: Option<String>,
    pub under_media: bool,
    pub under_supports: bool,
    pub under_layer: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ParserIndexValueFactsV0 {
    pub decl_names: Vec<String>,
    pub decl_names_with_local_refs: Vec<String>,
    pub decl_names_with_imported_refs: Vec<String>,
    pub import_names: Vec<String>,
    pub import_sources: Vec<String>,
    pub import_alias_count: usize,
    pub ref_names: Vec<String>,
    pub local_ref_names: Vec<String>,
    pub imported_ref_names: Vec<String>,
    pub imported_ref_sources: Vec<String>,
    pub declaration_ref_names: Vec<String>,
    pub declaration_imported_ref_sources: Vec<String>,
    pub value_decl_ref_names: Vec<String>,
    pub value_decl_imported_ref_sources: Vec<String>,
    pub selectors_with_refs_names: Vec<String>,
    pub selectors_with_local_refs_names: Vec<String>,
    pub selectors_with_imported_refs_names: Vec<String>,
    pub selectors_with_refs_under_media_names: Vec<String>,
    pub selectors_with_refs_under_supports_names: Vec<String>,
    pub selectors_with_refs_under_layer_names: Vec<String>,
    pub selectors_with_local_refs_under_media_names: Vec<String>,
    pub selectors_with_local_refs_under_supports_names: Vec<String>,
    pub selectors_with_local_refs_under_layer_names: Vec<String>,
    pub selectors_with_imported_refs_under_media_names: Vec<String>,
    pub selectors_with_imported_refs_under_supports_names: Vec<String>,
    pub selectors_with_imported_refs_under_layer_names: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ParserIndexCustomPropertyFactsV0 {
    pub decl_names: Vec<String>,
    pub decl_facts: Vec<ParserIndexCustomPropertyDeclFactV0>,
    pub decl_context_selectors: Vec<String>,
    pub decl_names_under_media: Vec<String>,
    pub decl_names_under_supports: Vec<String>,
    pub decl_names_under_layer: Vec<String>,
    pub ref_names: Vec<String>,
    pub ref_facts: Vec<ParserIndexCustomPropertyRefFactV0>,
    pub selectors_with_refs_names: Vec<String>,
    pub selectors_with_refs_under_media_names: Vec<String>,
    pub selectors_with_refs_under_supports_names: Vec<String>,
    pub selectors_with_refs_under_layer_names: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ParserIndexCustomPropertyDeclFactV0 {
    pub name: String,
    pub value: String,
    pub source_order: usize,
    pub byte_span: ParserByteSpanV0,
    pub range: ParserRangeV0,
    pub selector_contexts: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub condition_context: Vec<String>,
    pub layer_names: Vec<String>,
    pub under_media: bool,
    pub under_supports: bool,
    pub under_layer: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ParserIndexCustomPropertyRefFactV0 {
    pub name: String,
    pub source_order: usize,
    pub selector_contexts: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub condition_context: Vec<String>,
    pub layer_names: Vec<String>,
    pub under_media: bool,
    pub under_supports: bool,
    pub under_layer: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ParserSassSyntaxFactsV0 {
    pub variable_decl_names: Vec<String>,
    pub variable_parameter_names: Vec<String>,
    pub variable_ref_names: Vec<String>,
    pub mixin_decl_names: Vec<String>,
    pub mixin_include_names: Vec<String>,
    pub function_decl_names: Vec<String>,
    pub function_call_names: Vec<String>,
    pub module_use_sources: Vec<String>,
    pub module_use_edges: Vec<ParserIndexSassModuleUseFactV0>,
    pub module_forward_sources: Vec<String>,
    pub module_import_sources: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ParserIndexSassModuleUseFactV0 {
    pub source: String,
    pub namespace_kind: &'static str,
    pub namespace: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ParserIndexSassSameFileResolutionFactsV0 {
    pub resolved_variable_ref_names: Vec<String>,
    pub unresolved_variable_ref_names: Vec<String>,
    pub resolved_mixin_include_names: Vec<String>,
    pub unresolved_mixin_include_names: Vec<String>,
    pub resolved_function_call_names: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ParserByteSpanV0 {
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ParserPositionV0 {
    pub line: usize,
    pub character: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ParserRangeV0 {
    pub start: ParserPositionV0,
    pub end: ParserPositionV0,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ParserIndexSassSelectorSymbolFactV0 {
    pub selector_name: String,
    pub symbol_kind: &'static str,
    pub name: String,
    pub namespace: Option<String>,
    pub role: &'static str,
    pub resolution: &'static str,
    pub byte_span: ParserByteSpanV0,
    pub range: ParserRangeV0,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ParserIndexKeyframesFactsV0 {
    pub names: Vec<String>,
    pub names_under_media: Vec<String>,
    pub names_under_supports: Vec<String>,
    pub names_under_layer: Vec<String>,
    pub animation_ref_names: Vec<String>,
    pub animation_name_ref_names: Vec<String>,
    pub selectors_with_animation_ref_names: Vec<String>,
    pub selectors_with_animation_name_ref_names: Vec<String>,
    pub selectors_with_animation_refs_under_media_names: Vec<String>,
    pub selectors_with_animation_refs_under_supports_names: Vec<String>,
    pub selectors_with_animation_refs_under_layer_names: Vec<String>,
    pub selectors_with_animation_name_refs_under_media_names: Vec<String>,
    pub selectors_with_animation_name_refs_under_supports_names: Vec<String>,
    pub selectors_with_animation_name_refs_under_layer_names: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ParserIndexComposesFactsV0 {
    pub selectors_with_composes_names: Vec<String>,
    pub selectors_with_composes_under_media_names: Vec<String>,
    pub selectors_with_composes_under_supports_names: Vec<String>,
    pub selectors_with_composes_under_layer_names: Vec<String>,
    pub local_selector_names: Vec<String>,
    pub imported_selector_names: Vec<String>,
    pub global_selector_names: Vec<String>,
    pub local_selector_names_under_media: Vec<String>,
    pub local_selector_names_under_supports: Vec<String>,
    pub local_selector_names_under_layer: Vec<String>,
    pub imported_selector_names_under_media: Vec<String>,
    pub imported_selector_names_under_supports: Vec<String>,
    pub imported_selector_names_under_layer: Vec<String>,
    pub global_selector_names_under_media: Vec<String>,
    pub global_selector_names_under_supports: Vec<String>,
    pub global_selector_names_under_layer: Vec<String>,
    pub import_sources: Vec<String>,
    pub import_sources_under_media: Vec<String>,
    pub import_sources_under_supports: Vec<String>,
    pub import_sources_under_layer: Vec<String>,
    pub class_name_count: usize,
    pub local_class_name_count: usize,
    pub imported_class_name_count: usize,
    pub global_class_name_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ParserIndexWrapperFactsV0 {
    pub selectors_under_media_names: Vec<String>,
    pub selectors_under_supports_names: Vec<String>,
    pub selectors_under_layer_names: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct NestedSafetyCountsV0 {
    pub flat: usize,
    pub bem_suffix_safe: usize,
    pub nested_unsafe: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StyleSemanticFactsV0 {
    pub selector_identity: StyleSelectorIdentityFactsV0,
    pub custom_properties: StyleCustomPropertySemanticFactsV0,
    pub sass: StyleSassSemanticFactsV0,
    pub context_index: StyleContextIndexV0,
}

/// Explicit semantic indexes for CSS wrapper contexts that affect cascade and queries.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct StyleContextIndexV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_index: StyleLayerIndexV0,
    pub container_index: StyleContainerIndexV0,
    pub scope_index: StyleScopeIndexV0,
    pub selector_context_count: usize,
    pub ready_surfaces: Vec<&'static str>,
}

/// Cascade layer declarations, blocks, and selector membership.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct StyleLayerIndexV0 {
    pub statement_layers: Vec<StyleLayerStatementV0>,
    pub block_layers: Vec<StyleContextBlockV0>,
    pub selector_memberships: Vec<StyleContextSelectorMembershipV0>,
    pub named_layer_count: usize,
    pub anonymous_layer_block_count: usize,
}

/// `@layer` statement-layer ordering fact.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StyleLayerStatementV0 {
    pub name: String,
    pub source_order: usize,
    pub byte_span: ParserByteSpanV0,
    pub range: ParserRangeV0,
}

/// Container-query blocks and selector membership.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct StyleContainerIndexV0 {
    pub containers: Vec<StyleContextBlockV0>,
    pub selector_memberships: Vec<StyleContextSelectorMembershipV0>,
    pub named_container_count: usize,
    pub anonymous_container_count: usize,
}

/// CSS `@scope` blocks and selector membership.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct StyleScopeIndexV0 {
    pub scopes: Vec<StyleContextBlockV0>,
    pub selector_memberships: Vec<StyleContextSelectorMembershipV0>,
    pub scoped_selector_count: usize,
}

/// A semantic wrapper block with normalized prelude and source range.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StyleContextBlockV0 {
    pub id: String,
    pub kind: &'static str,
    pub name: Option<String>,
    pub prelude: String,
    pub source_order: usize,
    pub byte_span: ParserByteSpanV0,
    pub range: ParserRangeV0,
}

/// Selector membership edge from a selector to an enclosing context block.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StyleContextSelectorMembershipV0 {
    pub selector_name: String,
    pub context_id: String,
    pub context_kind: &'static str,
    pub source_order: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StyleSelectorIdentityFactsV0 {
    pub canonical_names: Vec<String>,
    pub bem_suffix_safe_names: Vec<String>,
    pub bem_suffix_parent_names: Vec<String>,
    pub nested_unsafe_names: Vec<String>,
    pub nested_safety_counts: NestedSafetyCountsV0,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StyleCustomPropertySemanticFactsV0 {
    pub decl_names: Vec<String>,
    pub ref_names: Vec<String>,
    pub resolved_ref_names: Vec<String>,
    pub unresolved_ref_names: Vec<String>,
    pub selectors_with_refs_names: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StyleSassSemanticFactsV0 {
    pub selector_symbol_facts: Vec<ParserIndexSassSelectorSymbolFactV0>,
    pub selectors_with_resolved_variable_refs_names: Vec<String>,
    pub selectors_with_unresolved_variable_refs_names: Vec<String>,
    pub selectors_with_resolved_mixin_includes_names: Vec<String>,
    pub selectors_with_unresolved_mixin_includes_names: Vec<String>,
    pub selectors_with_function_calls_names: Vec<String>,
    pub same_file_resolution: ParserIndexSassSameFileResolutionFactsV0,
}
