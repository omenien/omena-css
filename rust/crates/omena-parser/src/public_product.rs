//! Product-facing parser summaries and compatibility signals.
//!
//! This module is the stable reporting layer above the raw parser. It exposes
//! CSS Modules facts, canonical producer/candidate summaries, and evaluator
//! readiness payloads used by cme gates while the parser migrates toward the
//! standalone omena-css track.

use std::collections::{BTreeMap, BTreeSet};

use crate::{
    ParsedAnimationFactKind, ParsedCssModuleComposesEdgeKind, ParsedCssModuleComposesFactKind,
    ParsedCssModuleValueFactKind, ParsedSassModuleEdgeFactKind, ParsedSassSymbolFactKind,
    ParsedSelectorFactKind, ParsedStyleFacts, ParsedVariableFactKind, ParserByteSpanV0,
    ParserPositionV0, ParserRangeV0, StyleDialect, css_keyword, parse, product_facts_from_cst,
    summarize_omena_parser_parity_lite,
};
use cstree::text::TextRange;
use serde::Serialize;

mod style_blocks;
mod syntax_index;

#[cfg(test)]
mod tests;

use syntax_index::ProductSyntaxIndexV0;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ParserIndexSummaryV0 {
    schema_version: &'static str,
    language: &'static str,
    selectors: ParserIndexSelectorFactsV0,
    values: ParserIndexValueFactsV0,
    custom_properties: ParserIndexCustomPropertyFactsV0,
    sass: ParserIndexSassFactsV0,
    keyframes: ParserIndexKeyframesFactsV0,
    composes: ParserIndexComposesFactsV0,
    wrappers: ParserIndexWrapperFactsV0,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ParserCanonicalCandidateBundleV0 {
    schema_version: &'static str,
    language: &'static str,
    parity_lite: crate::OmenaParserParityLiteSummaryV0,
    css_modules_intermediate: ParserIndexSummaryV0,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
struct ParserEvaluatorCandidateV0 {
    kind: &'static str,
    selector_name: String,
    nested_safety_kind: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    bem_suffix_parent_name: Option<String>,
    under_media: bool,
    under_supports: bool,
    under_layer: bool,
    has_value_refs: bool,
    has_local_value_refs: bool,
    has_imported_value_refs: bool,
    has_custom_property_refs: bool,
    has_animation_ref: bool,
    has_animation_name_ref: bool,
    has_composes: bool,
    has_local_composes: bool,
    has_imported_composes: bool,
    has_global_composes: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ParserEvaluatorCandidatesV0 {
    schema_version: &'static str,
    language: &'static str,
    results: Vec<ParserEvaluatorCandidateV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ParserCanonicalProducerSignalV0 {
    schema_version: &'static str,
    language: &'static str,
    canonical_candidate: ParserCanonicalCandidateBundleV0,
    evaluator_candidates: ParserEvaluatorCandidatesV0,
    public_product_gate: ParserPublicProductGateSignalV0,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
struct ParserPublicProductGateSignalV0 {
    canonical_candidate_command: &'static str,
    consumer_boundary_command: &'static str,
    public_product_gate_command: &'static str,
    included_in_parser_lane: bool,
    included_in_rust_lane_bundle: bool,
    included_in_rust_release_bundle: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Default)]
#[serde(rename_all = "camelCase")]
struct ParserIndexSelectorFactsV0 {
    names: Vec<String>,
    definition_facts: Vec<ParserIndexSelectorDefinitionFactV0>,
    bem_suffix_parent_names: Vec<String>,
    bem_suffix_safe_names: Vec<String>,
    nested_unsafe_names: Vec<String>,
    selectors_with_value_refs_names: Vec<String>,
    selectors_with_animation_ref_names: Vec<String>,
    selectors_with_animation_name_ref_names: Vec<String>,
    bem_suffix_count: usize,
    nested_safety_counts: NestedSafetyCountsV0,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Default)]
#[serde(rename_all = "camelCase")]
struct ParserIndexSelectorDefinitionFactV0 {
    name: String,
    source_order: usize,
    byte_span: ParserByteSpanV0,
    range: ParserRangeV0,
    rule_byte_span: ParserByteSpanV0,
    rule_range: ParserRangeV0,
    full_selector: String,
    declarations: String,
    nested_safety_kind: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    bem_suffix_parent_name: Option<String>,
    under_media: bool,
    under_supports: bool,
    under_layer: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Default)]
#[serde(rename_all = "camelCase")]
struct ParserIndexValueFactsV0 {
    decl_names: Vec<String>,
    decl_facts: Vec<ParserIndexValueDeclFactV0>,
    decl_names_with_local_refs: Vec<String>,
    decl_names_with_imported_refs: Vec<String>,
    import_names: Vec<String>,
    import_facts: Vec<ParserIndexValueImportFactV0>,
    import_sources: Vec<String>,
    import_alias_count: usize,
    ref_names: Vec<String>,
    ref_facts: Vec<ParserIndexValueRefFactV0>,
    local_ref_names: Vec<String>,
    imported_ref_names: Vec<String>,
    imported_ref_sources: Vec<String>,
    declaration_ref_names: Vec<String>,
    declaration_imported_ref_sources: Vec<String>,
    value_decl_ref_names: Vec<String>,
    value_decl_imported_ref_sources: Vec<String>,
    selectors_with_refs_names: Vec<String>,
    selectors_with_local_refs_names: Vec<String>,
    selectors_with_imported_refs_names: Vec<String>,
    selectors_with_refs_under_media_names: Vec<String>,
    selectors_with_refs_under_supports_names: Vec<String>,
    selectors_with_refs_under_layer_names: Vec<String>,
    selectors_with_local_refs_under_media_names: Vec<String>,
    selectors_with_local_refs_under_supports_names: Vec<String>,
    selectors_with_local_refs_under_layer_names: Vec<String>,
    selectors_with_imported_refs_under_media_names: Vec<String>,
    selectors_with_imported_refs_under_supports_names: Vec<String>,
    selectors_with_imported_refs_under_layer_names: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Default)]
#[serde(rename_all = "camelCase")]
struct ParserIndexValueDeclFactV0 {
    name: String,
    value: String,
    source_order: usize,
    byte_span: ParserByteSpanV0,
    range: ParserRangeV0,
    rule_byte_span: ParserByteSpanV0,
    rule_range: ParserRangeV0,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Default)]
#[serde(rename_all = "camelCase")]
struct ParserIndexValueImportFactV0 {
    name: String,
    imported_name: String,
    from: String,
    source_order: usize,
    byte_span: ParserByteSpanV0,
    range: ParserRangeV0,
    #[serde(skip_serializing_if = "Option::is_none")]
    imported_name_byte_span: Option<ParserByteSpanV0>,
    #[serde(skip_serializing_if = "Option::is_none")]
    imported_name_range: Option<ParserRangeV0>,
    rule_byte_span: ParserByteSpanV0,
    rule_range: ParserRangeV0,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Default)]
#[serde(rename_all = "camelCase")]
struct ParserIndexValueRefFactV0 {
    name: String,
    source: &'static str,
    source_order: usize,
    byte_span: ParserByteSpanV0,
    range: ParserRangeV0,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Default)]
#[serde(rename_all = "camelCase")]
struct ParserIndexCustomPropertyFactsV0 {
    decl_names: Vec<String>,
    decl_facts: Vec<ParserIndexCustomPropertyDeclFactV0>,
    decl_context_selectors: Vec<String>,
    decl_names_under_media: Vec<String>,
    decl_names_under_supports: Vec<String>,
    decl_names_under_layer: Vec<String>,
    ref_names: Vec<String>,
    ref_facts: Vec<ParserIndexCustomPropertyRefFactV0>,
    selectors_with_refs_names: Vec<String>,
    selectors_with_refs_under_media_names: Vec<String>,
    selectors_with_refs_under_supports_names: Vec<String>,
    selectors_with_refs_under_layer_names: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Default)]
#[serde(rename_all = "camelCase")]
struct ParserIndexCustomPropertyDeclFactV0 {
    name: String,
    value: String,
    source_order: usize,
    byte_span: ParserByteSpanV0,
    range: ParserRangeV0,
    rule_byte_span: ParserByteSpanV0,
    rule_range: ParserRangeV0,
    selector_contexts: Vec<String>,
    wrapper_at_rules: Vec<ParserIndexAtRuleContextV0>,
    under_media: bool,
    under_supports: bool,
    under_layer: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Default)]
#[serde(rename_all = "camelCase")]
struct ParserIndexCustomPropertyRefFactV0 {
    name: String,
    source_order: usize,
    byte_span: ParserByteSpanV0,
    range: ParserRangeV0,
    selector_contexts: Vec<String>,
    wrapper_at_rules: Vec<ParserIndexAtRuleContextV0>,
    under_media: bool,
    under_supports: bool,
    under_layer: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Default)]
#[serde(rename_all = "camelCase")]
struct ParserIndexAtRuleContextV0 {
    name: String,
    params: String,
    byte_span: ParserByteSpanV0,
    range: ParserRangeV0,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Default)]
#[serde(rename_all = "camelCase")]
struct ParserIndexSassFactsV0 {
    variable_decl_names: Vec<String>,
    symbol_decl_facts: Vec<ParserIndexSassSymbolDeclFactV0>,
    variable_parameter_names: Vec<String>,
    variable_ref_names: Vec<String>,
    selectors_with_variable_refs_names: Vec<String>,
    selectors_with_resolved_variable_refs_names: Vec<String>,
    selectors_with_unresolved_variable_refs_names: Vec<String>,
    mixin_decl_names: Vec<String>,
    mixin_include_names: Vec<String>,
    selectors_with_mixin_includes_names: Vec<String>,
    selectors_with_resolved_mixin_includes_names: Vec<String>,
    selectors_with_unresolved_mixin_includes_names: Vec<String>,
    function_decl_names: Vec<String>,
    function_call_names: Vec<String>,
    selectors_with_function_calls_names: Vec<String>,
    selector_symbol_facts: Vec<ParserIndexSassSelectorSymbolFactV0>,
    module_use_sources: Vec<String>,
    module_use_edges: Vec<ParserIndexSassModuleUseFactV0>,
    module_forward_sources: Vec<String>,
    module_forward_edges: Vec<ParserIndexSassModuleForwardFactV0>,
    module_import_sources: Vec<String>,
    same_file_resolution: ParserIndexSassSameFileResolutionFactsV0,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Default)]
#[serde(rename_all = "camelCase")]
struct ParserIndexSassSymbolDeclFactV0 {
    symbol_kind: &'static str,
    name: String,
    role: &'static str,
    byte_span: ParserByteSpanV0,
    range: ParserRangeV0,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "camelCase")]
struct ParserIndexSassModuleUseFactV0 {
    source: String,
    namespace_kind: &'static str,
    namespace: Option<String>,
    byte_span: ParserByteSpanV0,
    range: ParserRangeV0,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "camelCase")]
struct ParserIndexSassModuleForwardFactV0 {
    source: String,
    prefix: String,
    visibility_kind: &'static str,
    visibility_members: Vec<ParserIndexSassModuleForwardMemberV0>,
    byte_span: ParserByteSpanV0,
    range: ParserRangeV0,
    rule_byte_span: ParserByteSpanV0,
    rule_range: ParserRangeV0,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "camelCase")]
struct ParserIndexSassModuleForwardMemberV0 {
    name: String,
    symbol_kind: Option<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Default)]
#[serde(rename_all = "camelCase")]
struct ParserIndexSassSameFileResolutionFactsV0 {
    resolved_variable_ref_names: Vec<String>,
    unresolved_variable_ref_names: Vec<String>,
    resolved_mixin_include_names: Vec<String>,
    unresolved_mixin_include_names: Vec<String>,
    resolved_function_call_names: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "camelCase")]
struct ParserIndexSassSelectorSymbolFactV0 {
    selector_name: String,
    symbol_kind: &'static str,
    name: String,
    namespace: Option<String>,
    role: &'static str,
    resolution: &'static str,
    byte_span: ParserByteSpanV0,
    range: ParserRangeV0,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Default)]
#[serde(rename_all = "camelCase")]
struct ParserIndexKeyframesFactsV0 {
    names: Vec<String>,
    decl_facts: Vec<ParserIndexKeyframesDeclFactV0>,
    names_under_media: Vec<String>,
    names_under_supports: Vec<String>,
    names_under_layer: Vec<String>,
    animation_ref_names: Vec<String>,
    animation_name_ref_names: Vec<String>,
    ref_facts: Vec<ParserIndexAnimationNameRefFactV0>,
    selectors_with_animation_ref_names: Vec<String>,
    selectors_with_animation_name_ref_names: Vec<String>,
    selectors_with_animation_refs_under_media_names: Vec<String>,
    selectors_with_animation_refs_under_supports_names: Vec<String>,
    selectors_with_animation_refs_under_layer_names: Vec<String>,
    selectors_with_animation_name_refs_under_media_names: Vec<String>,
    selectors_with_animation_name_refs_under_supports_names: Vec<String>,
    selectors_with_animation_name_refs_under_layer_names: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Default)]
#[serde(rename_all = "camelCase")]
struct ParserIndexKeyframesDeclFactV0 {
    name: String,
    source_order: usize,
    byte_span: ParserByteSpanV0,
    range: ParserRangeV0,
    rule_byte_span: ParserByteSpanV0,
    rule_range: ParserRangeV0,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Default)]
#[serde(rename_all = "camelCase")]
struct ParserIndexAnimationNameRefFactV0 {
    name: String,
    property: &'static str,
    source_order: usize,
    byte_span: ParserByteSpanV0,
    range: ParserRangeV0,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Default)]
#[serde(rename_all = "camelCase")]
struct ParserIndexComposesFactsV0 {
    edges: Vec<ParserIndexComposesEdgeFactV0>,
    selectors_with_composes_names: Vec<String>,
    selectors_with_composes_under_media_names: Vec<String>,
    selectors_with_composes_under_supports_names: Vec<String>,
    selectors_with_composes_under_layer_names: Vec<String>,
    local_selector_names: Vec<String>,
    imported_selector_names: Vec<String>,
    global_selector_names: Vec<String>,
    local_selector_names_under_media: Vec<String>,
    local_selector_names_under_supports: Vec<String>,
    local_selector_names_under_layer: Vec<String>,
    imported_selector_names_under_media: Vec<String>,
    imported_selector_names_under_supports: Vec<String>,
    imported_selector_names_under_layer: Vec<String>,
    global_selector_names_under_media: Vec<String>,
    global_selector_names_under_supports: Vec<String>,
    global_selector_names_under_layer: Vec<String>,
    import_sources: Vec<String>,
    import_sources_under_media: Vec<String>,
    import_sources_under_supports: Vec<String>,
    import_sources_under_layer: Vec<String>,
    class_name_count: usize,
    local_class_name_count: usize,
    imported_class_name_count: usize,
    global_class_name_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "camelCase")]
struct ParserIndexComposesEdgeFactV0 {
    kind: &'static str,
    owner_selector_names: Vec<String>,
    target_names: Vec<String>,
    import_source: Option<String>,
    class_tokens: Vec<ParserIndexComposesClassTokenV0>,
    byte_span: ParserByteSpanV0,
    range: ParserRangeV0,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "camelCase")]
struct ParserIndexComposesClassTokenV0 {
    class_name: String,
    byte_span: ParserByteSpanV0,
    range: ParserRangeV0,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Default)]
#[serde(rename_all = "camelCase")]
struct ParserIndexWrapperFactsV0 {
    selectors_under_media_names: Vec<String>,
    selectors_under_supports_names: Vec<String>,
    selectors_under_layer_names: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Default)]
#[serde(rename_all = "camelCase")]
struct NestedSafetyCountsV0 {
    flat: usize,
    bem_suffix_safe: usize,
    nested_unsafe: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SelectorBranch {
    name: String,
    name_span: ParserByteSpanV0,
    bare_suffix_base: bool,
    amp_suffix_depth: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SassVariableDeclScope {
    name: String,
    selector_names: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct StyleBlock {
    names: Vec<String>,
    context_text: Option<String>,
    start: usize,
    end: usize,
    rule_start: usize,
    rule_end: usize,
    body_start: usize,
    body_end: usize,
    header_text: Option<String>,
    under_media: bool,
    under_supports: bool,
    under_layer: bool,
    wrapper_at_rules: Vec<ParserIndexAtRuleContextV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
struct WrapperContext {
    under_media: bool,
    under_supports: bool,
    under_layer: bool,
    wrapper_at_rules: Vec<ParserIndexAtRuleContextV0>,
}

pub fn summarize_css_modules_intermediate(
    source: &str,
    dialect: StyleDialect,
) -> ParserIndexSummaryV0 {
    let line_index = SourceLineIndex::new(source);
    let parsed = parse(source, dialect);
    let facts = product_facts_from_cst(source, &parsed);
    let blocks = style_blocks::collect_style_blocks_from_cst(source, &line_index, &parsed);
    let syntax_index = ProductSyntaxIndexV0::new(source, &parsed);
    let selectors = summarize_selectors(source, &line_index, &facts, &blocks);
    let values = summarize_values(source, &line_index, &facts, &blocks, &syntax_index);
    let custom_properties =
        summarize_custom_properties(source, &line_index, &facts, &blocks, &syntax_index);
    let sass = summarize_sass(source, &line_index, &facts, &blocks, &syntax_index);
    let keyframes = summarize_keyframes(source, &line_index, &facts, &blocks, &syntax_index);
    let composes = summarize_composes(source, &line_index, &facts, &blocks);
    let wrappers = summarize_wrappers(&blocks);

    ParserIndexSummaryV0 {
        schema_version: "0",
        language: dialect_label(dialect),
        selectors: ParserIndexSelectorFactsV0 {
            selectors_with_value_refs_names: values.selectors_with_refs_names.clone(),
            selectors_with_animation_ref_names: keyframes
                .selectors_with_animation_ref_names
                .clone(),
            selectors_with_animation_name_ref_names: keyframes
                .selectors_with_animation_name_ref_names
                .clone(),
            ..selectors
        },
        values,
        custom_properties,
        sass,
        keyframes,
        composes,
        wrappers,
    }
}

pub fn summarize_parser_canonical_candidate(
    source: &str,
    dialect: StyleDialect,
) -> ParserCanonicalCandidateBundleV0 {
    let parity_lite = summarize_omena_parser_parity_lite(source, dialect);
    let css_modules_intermediate = summarize_css_modules_intermediate(source, dialect);

    ParserCanonicalCandidateBundleV0 {
        schema_version: "0",
        language: parity_lite.language,
        parity_lite,
        css_modules_intermediate,
    }
}

pub fn summarize_parser_evaluator_candidates(
    source: &str,
    dialect: StyleDialect,
) -> ParserEvaluatorCandidatesV0 {
    let intermediate = summarize_css_modules_intermediate(source, dialect);
    let bem_suffix_safe_names: BTreeSet<&str> = intermediate
        .selectors
        .bem_suffix_safe_names
        .iter()
        .map(String::as_str)
        .collect();
    let nested_unsafe_names: BTreeSet<&str> = intermediate
        .selectors
        .nested_unsafe_names
        .iter()
        .map(String::as_str)
        .collect();
    let selectors_under_media_names: BTreeSet<&str> = intermediate
        .wrappers
        .selectors_under_media_names
        .iter()
        .map(String::as_str)
        .collect();
    let selectors_under_supports_names: BTreeSet<&str> = intermediate
        .wrappers
        .selectors_under_supports_names
        .iter()
        .map(String::as_str)
        .collect();
    let selectors_under_layer_names: BTreeSet<&str> = intermediate
        .wrappers
        .selectors_under_layer_names
        .iter()
        .map(String::as_str)
        .collect();
    let selectors_with_refs_names: BTreeSet<&str> = intermediate
        .values
        .selectors_with_refs_names
        .iter()
        .map(String::as_str)
        .collect();
    let selectors_with_local_refs_names: BTreeSet<&str> = intermediate
        .values
        .selectors_with_local_refs_names
        .iter()
        .map(String::as_str)
        .collect();
    let selectors_with_imported_refs_names: BTreeSet<&str> = intermediate
        .values
        .selectors_with_imported_refs_names
        .iter()
        .map(String::as_str)
        .collect();
    let selectors_with_custom_property_refs_names: BTreeSet<&str> = intermediate
        .custom_properties
        .selectors_with_refs_names
        .iter()
        .map(String::as_str)
        .collect();
    let selectors_with_animation_ref_names: BTreeSet<&str> = intermediate
        .keyframes
        .selectors_with_animation_ref_names
        .iter()
        .map(String::as_str)
        .collect();
    let selectors_with_animation_name_ref_names: BTreeSet<&str> = intermediate
        .keyframes
        .selectors_with_animation_name_ref_names
        .iter()
        .map(String::as_str)
        .collect();
    let selectors_with_composes_names: BTreeSet<&str> = intermediate
        .composes
        .selectors_with_composes_names
        .iter()
        .map(String::as_str)
        .collect();
    let local_selector_names: BTreeSet<&str> = intermediate
        .composes
        .local_selector_names
        .iter()
        .map(String::as_str)
        .collect();
    let imported_selector_names: BTreeSet<&str> = intermediate
        .composes
        .imported_selector_names
        .iter()
        .map(String::as_str)
        .collect();
    let global_selector_names: BTreeSet<&str> = intermediate
        .composes
        .global_selector_names
        .iter()
        .map(String::as_str)
        .collect();

    let results = intermediate
        .selectors
        .names
        .iter()
        .map(|selector_name| {
            let selector = selector_name.as_str();
            let nested_safety_kind = if nested_unsafe_names.contains(selector) {
                "nestedUnsafe"
            } else if bem_suffix_safe_names.contains(selector) {
                "bemSuffixSafe"
            } else {
                "flat"
            };
            ParserEvaluatorCandidateV0 {
                kind: "selector-index-facts",
                selector_name: selector_name.clone(),
                nested_safety_kind,
                bem_suffix_parent_name: if nested_safety_kind == "bemSuffixSafe" {
                    bem_suffix_parent_name(selector)
                } else {
                    None
                },
                under_media: selectors_under_media_names.contains(selector),
                under_supports: selectors_under_supports_names.contains(selector),
                under_layer: selectors_under_layer_names.contains(selector),
                has_value_refs: selectors_with_refs_names.contains(selector),
                has_local_value_refs: selectors_with_local_refs_names.contains(selector),
                has_imported_value_refs: selectors_with_imported_refs_names.contains(selector),
                has_custom_property_refs: selectors_with_custom_property_refs_names
                    .contains(selector),
                has_animation_ref: selectors_with_animation_ref_names.contains(selector),
                has_animation_name_ref: selectors_with_animation_name_ref_names.contains(selector),
                has_composes: selectors_with_composes_names.contains(selector),
                has_local_composes: local_selector_names.contains(selector),
                has_imported_composes: imported_selector_names.contains(selector),
                has_global_composes: global_selector_names.contains(selector),
            }
        })
        .collect();

    ParserEvaluatorCandidatesV0 {
        schema_version: "0",
        language: intermediate.language,
        results,
    }
}

pub fn summarize_parser_canonical_producer_signal(
    source: &str,
    dialect: StyleDialect,
) -> ParserCanonicalProducerSignalV0 {
    let canonical_candidate = summarize_parser_canonical_candidate(source, dialect);
    let evaluator_candidates = summarize_parser_evaluator_candidates(source, dialect);

    ParserCanonicalProducerSignalV0 {
        schema_version: "0",
        language: canonical_candidate.language,
        canonical_candidate,
        evaluator_candidates,
        public_product_gate: ParserPublicProductGateSignalV0 {
            canonical_candidate_command: "pnpm check:rust-parser-canonical-candidate",
            consumer_boundary_command: "pnpm check:rust-parser-consumer-boundary",
            public_product_gate_command: "pnpm check:rust-parser-public-product",
            included_in_parser_lane: true,
            included_in_rust_lane_bundle: true,
            included_in_rust_release_bundle: true,
        },
    }
}

fn summarize_selectors(
    source: &str,
    line_index: &SourceLineIndex,
    facts: &ParsedStyleFacts,
    blocks: &[StyleBlock],
) -> ParserIndexSelectorFactsV0 {
    let mut names = Vec::new();
    let mut definition_facts = Vec::new();
    let mut bem_suffix_parent_names = Vec::new();
    let mut bem_suffix_safe_names = Vec::new();
    let mut nested_unsafe_names = Vec::new();
    let mut nested_safety_counts = NestedSafetyCountsV0::default();

    for selector in &facts.selectors {
        if selector.kind != ParsedSelectorFactKind::Class {
            continue;
        }
        let name = selector.name.clone();
        names.push(name.clone());
        let byte_span = byte_span_for_range(selector.range);
        let nested_safety_kind = nested_safety_for_selector(blocks, &name).unwrap_or("flat");
        let rule_block = selector_rule_block(blocks, &name, byte_span.start);
        let rule_byte_span = rule_block
            .map(|block| ParserByteSpanV0 {
                start: block.rule_start,
                end: block.rule_end,
            })
            .unwrap_or(byte_span);
        let full_selector = rule_block
            .and_then(|block| block.header_text.clone())
            .unwrap_or_else(|| format!(".{name}"));
        let declarations = rule_block
            .and_then(|block| source.get(block.body_start..block.body_end))
            .unwrap_or_default()
            .trim()
            .to_string();
        let bem_suffix_parent_name = if nested_safety_kind == "bemSuffixSafe" {
            bem_suffix_parent_name(&name)
        } else {
            None
        };
        match nested_safety_kind {
            "bemSuffixSafe" => {
                nested_safety_counts.bem_suffix_safe += 1;
                bem_suffix_safe_names.push(name.clone());
                if let Some(parent) = &bem_suffix_parent_name {
                    bem_suffix_parent_names.push(parent.clone());
                }
            }
            "nestedUnsafe" => {
                nested_safety_counts.nested_unsafe += 1;
                nested_unsafe_names.push(name.clone());
            }
            _ => nested_safety_counts.flat += 1,
        }
        let wrapper = wrapper_for_offset(blocks, byte_span.start);
        definition_facts.push(ParserIndexSelectorDefinitionFactV0 {
            name,
            source_order: definition_facts.len(),
            byte_span,
            range: parser_range_for_byte_span(source, line_index, byte_span),
            rule_byte_span,
            rule_range: parser_range_for_byte_span(source, line_index, rule_byte_span),
            full_selector,
            declarations,
            nested_safety_kind,
            bem_suffix_parent_name,
            under_media: wrapper.under_media,
            under_supports: wrapper.under_supports,
            under_layer: wrapper.under_layer,
        });
    }

    names.sort();
    definition_facts.sort();
    bem_suffix_parent_names.sort();
    bem_suffix_safe_names.sort();
    nested_unsafe_names.sort();

    ParserIndexSelectorFactsV0 {
        names,
        definition_facts,
        bem_suffix_count: bem_suffix_safe_names.len(),
        bem_suffix_parent_names,
        bem_suffix_safe_names,
        nested_unsafe_names,
        nested_safety_counts,
        ..ParserIndexSelectorFactsV0::default()
    }
}

fn summarize_values(
    source: &str,
    line_index: &SourceLineIndex,
    facts: &ParsedStyleFacts,
    blocks: &[StyleBlock],
    syntax_index: &ProductSyntaxIndexV0,
) -> ParserIndexValueFactsV0 {
    let imported_sources_by_name = facts
        .css_module_value_import_edges
        .iter()
        .map(|edge| (edge.local_name.clone(), edge.import_source.clone()))
        .collect::<BTreeMap<_, _>>();
    let imported_names = imported_sources_by_name
        .keys()
        .cloned()
        .collect::<BTreeSet<_>>();
    let local_decl_names = facts
        .css_module_values
        .iter()
        .filter(|value| value.kind == ParsedCssModuleValueFactKind::Definition)
        .map(|value| value.name.clone())
        .filter(|name| !imported_names.contains(name))
        .collect::<BTreeSet<_>>();
    let mut decl_facts = Vec::new();
    for value in &facts.css_module_values {
        if value.kind != ParsedCssModuleValueFactKind::Definition
            || !local_decl_names.contains(&value.name)
        {
            continue;
        }
        let byte_span = byte_span_for_range(value.range);
        let rule_byte_span = syntax_index
            .css_module_value_span_for_offset(byte_span.start)
            .unwrap_or(byte_span);
        decl_facts.push(ParserIndexValueDeclFactV0 {
            name: value.name.clone(),
            value: syntax_index
                .css_module_value_text(source, byte_span.start)
                .unwrap_or_default(),
            source_order: decl_facts.len(),
            byte_span,
            range: parser_range_for_byte_span(source, line_index, byte_span),
            rule_byte_span,
            rule_range: parser_range_for_byte_span(source, line_index, rule_byte_span),
        });
    }
    decl_facts.sort();
    decl_facts.dedup();
    let mut import_facts = Vec::new();
    for edge in &facts.css_module_value_import_edges {
        let byte_span = byte_span_for_range(edge.local_range);
        let remote_byte_span = byte_span_for_range(edge.remote_range);
        let imported_name_byte_span =
            (edge.remote_name != edge.local_name).then_some(remote_byte_span);
        let rule_byte_span = syntax_index
            .css_module_value_span_for_offset(byte_span.start)
            .unwrap_or(byte_span);
        import_facts.push(ParserIndexValueImportFactV0 {
            name: edge.local_name.clone(),
            imported_name: edge.remote_name.clone(),
            from: edge.import_source.clone(),
            source_order: import_facts.len(),
            byte_span,
            range: parser_range_for_byte_span(source, line_index, byte_span),
            imported_name_byte_span,
            imported_name_range: imported_name_byte_span
                .map(|span| parser_range_for_byte_span(source, line_index, span)),
            rule_byte_span,
            rule_range: parser_range_for_byte_span(source, line_index, rule_byte_span),
        });
    }
    import_facts.sort();
    import_facts.dedup();
    let mut ref_facts = Vec::new();
    let value_decl_ref_names = facts
        .css_module_value_definition_edges
        .iter()
        .flat_map(|edge| edge.reference_names.iter().cloned())
        .collect::<Vec<_>>();
    let mut declaration_ref_names = Vec::new();
    let mut selectors_with_refs = BTreeSet::new();
    let mut selectors_with_local_refs = BTreeSet::new();
    let mut selectors_with_imported_refs = BTreeSet::new();
    let mut selectors_with_refs_under_media = BTreeSet::new();
    let mut selectors_with_refs_under_supports = BTreeSet::new();
    let mut selectors_with_refs_under_layer = BTreeSet::new();
    let mut selectors_with_local_refs_under_media = BTreeSet::new();
    let mut selectors_with_local_refs_under_supports = BTreeSet::new();
    let mut selectors_with_local_refs_under_layer = BTreeSet::new();
    let mut selectors_with_imported_refs_under_media = BTreeSet::new();
    let mut selectors_with_imported_refs_under_supports = BTreeSet::new();
    let mut selectors_with_imported_refs_under_layer = BTreeSet::new();

    for value in &facts.css_module_values {
        if value.kind != ParsedCssModuleValueFactKind::Reference {
            continue;
        }
        if !local_decl_names.contains(&value.name) && !imported_names.contains(&value.name) {
            continue;
        }
        let offset = range_start(value.range);
        let selector_names = selector_names_for_offset(blocks, offset);
        if !selector_names.is_empty() {
            declaration_ref_names.push(value.name.clone());
            let byte_span = byte_span_for_range(value.range);
            ref_facts.push(ParserIndexValueRefFactV0 {
                name: value.name.clone(),
                source: "declaration",
                source_order: ref_facts.len(),
                byte_span,
                range: parser_range_for_byte_span(source, line_index, byte_span),
            });
            let wrapper = wrapper_for_offset(blocks, offset);
            for selector in selector_names {
                selectors_with_refs.insert(selector.clone());
                insert_by_wrapper(
                    &mut selectors_with_refs_under_media,
                    &mut selectors_with_refs_under_supports,
                    &mut selectors_with_refs_under_layer,
                    &selector,
                    &wrapper,
                );
                if local_decl_names.contains(&value.name) {
                    selectors_with_local_refs.insert(selector.clone());
                    insert_by_wrapper(
                        &mut selectors_with_local_refs_under_media,
                        &mut selectors_with_local_refs_under_supports,
                        &mut selectors_with_local_refs_under_layer,
                        &selector,
                        &wrapper,
                    );
                }
                if imported_names.contains(&value.name) {
                    selectors_with_imported_refs.insert(selector.clone());
                    insert_by_wrapper(
                        &mut selectors_with_imported_refs_under_media,
                        &mut selectors_with_imported_refs_under_supports,
                        &mut selectors_with_imported_refs_under_layer,
                        &selector,
                        &wrapper,
                    );
                }
            }
        } else {
            let byte_span = byte_span_for_range(value.range);
            ref_facts.push(ParserIndexValueRefFactV0 {
                name: value.name.clone(),
                source: "valueDecl",
                source_order: ref_facts.len(),
                byte_span,
                range: parser_range_for_byte_span(source, line_index, byte_span),
            });
        }
    }
    ref_facts.sort();
    ref_facts.dedup();

    let mut value_decl_imported_ref_sources = Vec::new();
    for name in &value_decl_ref_names {
        if let Some(source) = imported_sources_by_name.get(name) {
            value_decl_imported_ref_sources.push(source.clone());
        }
    }
    let mut declaration_imported_ref_sources = Vec::new();
    for name in &declaration_ref_names {
        if let Some(source) = imported_sources_by_name.get(name) {
            declaration_imported_ref_sources.push(source.clone());
        }
    }
    let semantic_ref_names = declaration_ref_names
        .iter()
        .chain(value_decl_ref_names.iter())
        .cloned()
        .collect::<Vec<_>>();

    ParserIndexValueFactsV0 {
        decl_names: sorted(local_decl_names.clone()),
        decl_facts,
        decl_names_with_local_refs: facts
            .css_module_value_definition_edges
            .iter()
            .filter(|edge| {
                edge.reference_names
                    .iter()
                    .any(|name| local_decl_names.contains(name))
            })
            .map(|edge| edge.definition_name.clone())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect(),
        decl_names_with_imported_refs: facts
            .css_module_value_definition_edges
            .iter()
            .filter(|edge| {
                edge.reference_names
                    .iter()
                    .any(|name| imported_names.contains(name))
            })
            .map(|edge| edge.definition_name.clone())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect(),
        import_names: facts
            .css_module_value_import_edges
            .iter()
            .map(|edge| edge.local_name.clone())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect(),
        import_facts,
        import_sources: facts
            .css_module_value_import_edges
            .iter()
            .map(|edge| edge.import_source.clone())
            .collect::<Vec<_>>()
            .tap_sort(),
        import_alias_count: facts
            .css_module_value_import_edges
            .iter()
            .filter(|edge| edge.remote_name != edge.local_name)
            .count(),
        ref_names: semantic_ref_names.clone().tap_sort(),
        ref_facts,
        local_ref_names: semantic_ref_names
            .iter()
            .filter(|name| local_decl_names.contains(*name))
            .cloned()
            .collect::<Vec<_>>()
            .tap_sort(),
        imported_ref_names: semantic_ref_names
            .iter()
            .filter(|name| imported_names.contains(*name))
            .cloned()
            .collect::<Vec<_>>()
            .tap_sort(),
        imported_ref_sources: semantic_ref_names
            .iter()
            .filter_map(|name| imported_sources_by_name.get(name).cloned())
            .collect::<Vec<_>>()
            .tap_sort(),
        declaration_ref_names: declaration_ref_names.tap_sort(),
        declaration_imported_ref_sources: declaration_imported_ref_sources.tap_sort(),
        value_decl_ref_names: value_decl_ref_names.tap_sort(),
        value_decl_imported_ref_sources: value_decl_imported_ref_sources.tap_sort(),
        selectors_with_refs_names: sorted(selectors_with_refs),
        selectors_with_local_refs_names: sorted(selectors_with_local_refs),
        selectors_with_imported_refs_names: sorted(selectors_with_imported_refs),
        selectors_with_refs_under_media_names: sorted(selectors_with_refs_under_media),
        selectors_with_refs_under_supports_names: sorted(selectors_with_refs_under_supports),
        selectors_with_refs_under_layer_names: sorted(selectors_with_refs_under_layer),
        selectors_with_local_refs_under_media_names: sorted(selectors_with_local_refs_under_media),
        selectors_with_local_refs_under_supports_names: sorted(
            selectors_with_local_refs_under_supports,
        ),
        selectors_with_local_refs_under_layer_names: sorted(selectors_with_local_refs_under_layer),
        selectors_with_imported_refs_under_media_names: sorted(
            selectors_with_imported_refs_under_media,
        ),
        selectors_with_imported_refs_under_supports_names: sorted(
            selectors_with_imported_refs_under_supports,
        ),
        selectors_with_imported_refs_under_layer_names: sorted(
            selectors_with_imported_refs_under_layer,
        ),
    }
}

fn summarize_custom_properties(
    source: &str,
    line_index: &SourceLineIndex,
    facts: &ParsedStyleFacts,
    blocks: &[StyleBlock],
    syntax_index: &ProductSyntaxIndexV0,
) -> ParserIndexCustomPropertyFactsV0 {
    let mut decl_facts = Vec::new();
    let mut ref_facts = Vec::new();
    for variable in &facts.variables {
        match variable.kind {
            ParsedVariableFactKind::CustomPropertyDeclaration => {
                let byte_span = byte_span_for_range(variable.range);
                let wrapper = wrapper_for_offset(blocks, byte_span.start);
                let rule_byte_span = style_block_for_offset(blocks, byte_span.start)
                    .map(|block| ParserByteSpanV0 {
                        start: block.rule_start,
                        end: block.rule_end,
                    })
                    .or_else(|| syntax_index.declaration_span_for_offset(byte_span.start))
                    .unwrap_or(byte_span);
                decl_facts.push(ParserIndexCustomPropertyDeclFactV0 {
                    name: variable.name.clone(),
                    value: syntax_index
                        .declaration_value_text(source, byte_span.start)
                        .unwrap_or_default(),
                    source_order: decl_facts.len(),
                    byte_span,
                    range: parser_range_for_byte_span(source, line_index, byte_span),
                    rule_byte_span,
                    rule_range: parser_range_for_byte_span(source, line_index, rule_byte_span),
                    selector_contexts: selector_contexts_for_offset(blocks, byte_span.start),
                    wrapper_at_rules: wrapper.wrapper_at_rules.clone(),
                    under_media: wrapper.under_media,
                    under_supports: wrapper.under_supports,
                    under_layer: wrapper.under_layer,
                });
            }
            ParsedVariableFactKind::CustomPropertyReference => {
                let byte_span = byte_span_for_range(variable.range);
                let wrapper = wrapper_for_offset(blocks, byte_span.start);
                ref_facts.push(ParserIndexCustomPropertyRefFactV0 {
                    name: variable.name.clone(),
                    source_order: ref_facts.len(),
                    byte_span,
                    range: parser_range_for_byte_span(source, line_index, byte_span),
                    selector_contexts: selector_contexts_for_offset(blocks, byte_span.start),
                    wrapper_at_rules: wrapper.wrapper_at_rules.clone(),
                    under_media: wrapper.under_media,
                    under_supports: wrapper.under_supports,
                    under_layer: wrapper.under_layer,
                });
            }
            _ => {}
        }
    }
    decl_facts.sort();
    decl_facts.dedup();
    ref_facts.sort();
    ref_facts.dedup();
    ParserIndexCustomPropertyFactsV0 {
        decl_names: sorted(decl_facts.iter().map(|fact| fact.name.clone()).collect()),
        decl_context_selectors: sorted(
            decl_facts
                .iter()
                .flat_map(|fact| fact.selector_contexts.iter().cloned())
                .collect(),
        ),
        decl_names_under_media: sorted(
            decl_facts
                .iter()
                .filter(|fact| fact.under_media)
                .map(|fact| fact.name.clone())
                .collect(),
        ),
        decl_names_under_supports: sorted(
            decl_facts
                .iter()
                .filter(|fact| fact.under_supports)
                .map(|fact| fact.name.clone())
                .collect(),
        ),
        decl_names_under_layer: sorted(
            decl_facts
                .iter()
                .filter(|fact| fact.under_layer)
                .map(|fact| fact.name.clone())
                .collect(),
        ),
        ref_names: sorted(ref_facts.iter().map(|fact| fact.name.clone()).collect()),
        selectors_with_refs_names: sorted(
            ref_facts
                .iter()
                .flat_map(|fact| selector_names_from_contexts(&fact.selector_contexts))
                .collect(),
        ),
        selectors_with_refs_under_media_names: sorted(
            ref_facts
                .iter()
                .filter(|fact| fact.under_media)
                .flat_map(|fact| selector_names_from_contexts(&fact.selector_contexts))
                .collect(),
        ),
        selectors_with_refs_under_supports_names: sorted(
            ref_facts
                .iter()
                .filter(|fact| fact.under_supports)
                .flat_map(|fact| selector_names_from_contexts(&fact.selector_contexts))
                .collect(),
        ),
        selectors_with_refs_under_layer_names: sorted(
            ref_facts
                .iter()
                .filter(|fact| fact.under_layer)
                .flat_map(|fact| selector_names_from_contexts(&fact.selector_contexts))
                .collect(),
        ),
        decl_facts,
        ref_facts,
    }
}

fn summarize_sass(
    source: &str,
    line_index: &SourceLineIndex,
    facts: &ParsedStyleFacts,
    blocks: &[StyleBlock],
    syntax_index: &ProductSyntaxIndexV0,
) -> ParserIndexSassFactsV0 {
    let mut variable_decl_names = BTreeSet::new();
    let mut variable_parameter_names = BTreeSet::new();
    let mut variable_ref_names = BTreeSet::new();
    let mut mixin_decl_names = BTreeSet::new();
    let mut mixin_include_names = BTreeSet::new();
    let mut function_decl_names = BTreeSet::new();
    let mut function_call_names = BTreeSet::new();
    let mut symbol_decl_facts = Vec::new();
    let mut selector_symbol_facts = Vec::new();
    let mut global_variable_decl_names = BTreeSet::new();
    let mut variable_decl_scopes = Vec::new();

    for symbol in &facts.sass_symbols {
        let byte_span = byte_span_for_range(symbol.range);
        let range = parser_range_for_byte_span(source, line_index, byte_span);
        match symbol.kind {
            ParsedSassSymbolFactKind::VariableDeclaration => {
                if symbol.role == "parameter"
                    || syntax_index.sass_parameter_list_contains(byte_span.start)
                {
                    variable_parameter_names.insert(symbol.name.clone());
                } else {
                    variable_decl_names.insert(symbol.name.clone());
                    let selector_names = selector_names_for_offset(blocks, byte_span.start);
                    if selector_names.is_empty() {
                        global_variable_decl_names.insert(symbol.name.clone());
                    }
                    variable_decl_scopes.push(SassVariableDeclScope {
                        name: symbol.name.clone(),
                        selector_names,
                    });
                }
                symbol_decl_facts.push(ParserIndexSassSymbolDeclFactV0 {
                    symbol_kind: symbol.symbol_kind,
                    name: symbol.name.clone(),
                    role: symbol.role,
                    byte_span,
                    range,
                });
            }
            ParsedSassSymbolFactKind::MixinDeclaration => {
                mixin_decl_names.insert(symbol.name.clone());
                symbol_decl_facts.push(ParserIndexSassSymbolDeclFactV0 {
                    symbol_kind: symbol.symbol_kind,
                    name: symbol.name.clone(),
                    role: symbol.role,
                    byte_span,
                    range,
                });
            }
            ParsedSassSymbolFactKind::FunctionDeclaration => {
                function_decl_names.insert(symbol.name.clone());
                symbol_decl_facts.push(ParserIndexSassSymbolDeclFactV0 {
                    symbol_kind: symbol.symbol_kind,
                    name: symbol.name.clone(),
                    role: symbol.role,
                    byte_span,
                    range,
                });
            }
            ParsedSassSymbolFactKind::VariableReference => {
                variable_ref_names.insert(symbol.name.clone());
            }
            ParsedSassSymbolFactKind::MixinInclude => {
                if symbol.namespace.is_none() {
                    mixin_include_names.insert(symbol.name.clone());
                }
            }
            ParsedSassSymbolFactKind::FunctionCall => {
                if symbol.namespace.is_none() {
                    function_call_names.insert(symbol.name.clone());
                }
            }
        }
    }

    let mut resolved_variable_ref_names = BTreeSet::new();
    let mut unresolved_variable_ref_names = BTreeSet::new();
    for symbol in &facts.sass_symbols {
        if symbol.kind != ParsedSassSymbolFactKind::VariableReference || symbol.namespace.is_some()
        {
            continue;
        }
        if is_sass_variable_reference_resolved(
            &symbol.name,
            range_start(symbol.range),
            blocks,
            &global_variable_decl_names,
            &variable_parameter_names,
            &variable_decl_scopes,
        ) {
            resolved_variable_ref_names.insert(symbol.name.clone());
        } else {
            unresolved_variable_ref_names.insert(symbol.name.clone());
        }
    }

    let same_file_resolution = ParserIndexSassSameFileResolutionFactsV0 {
        resolved_variable_ref_names: sorted(resolved_variable_ref_names),
        unresolved_variable_ref_names: sorted(unresolved_variable_ref_names),
        resolved_mixin_include_names: sorted(
            mixin_include_names
                .iter()
                .filter(|name| mixin_decl_names.contains(*name))
                .cloned()
                .collect(),
        ),
        unresolved_mixin_include_names: sorted(
            mixin_include_names
                .iter()
                .filter(|name| !mixin_decl_names.contains(*name))
                .cloned()
                .collect(),
        ),
        resolved_function_call_names: sorted(
            function_call_names
                .iter()
                .filter(|name| function_decl_names.contains(*name))
                .cloned()
                .collect(),
        ),
    };

    for symbol in &facts.sass_symbols {
        if matches!(
            symbol.kind,
            ParsedSassSymbolFactKind::VariableDeclaration
                | ParsedSassSymbolFactKind::MixinDeclaration
                | ParsedSassSymbolFactKind::FunctionDeclaration
        ) {
            continue;
        }
        let offset = range_start(symbol.range);
        let byte_span = byte_span_for_range(symbol.range);
        for selector_name in selector_names_for_offset(blocks, offset) {
            let resolution = match symbol.kind {
                ParsedSassSymbolFactKind::VariableReference if symbol.namespace.is_some() => {
                    "external"
                }
                ParsedSassSymbolFactKind::VariableReference
                    if is_sass_variable_reference_resolved(
                        &symbol.name,
                        offset,
                        blocks,
                        &global_variable_decl_names,
                        &variable_parameter_names,
                        &variable_decl_scopes,
                    ) =>
                {
                    "resolved"
                }
                ParsedSassSymbolFactKind::MixinInclude if symbol.namespace.is_some() => "external",
                ParsedSassSymbolFactKind::MixinInclude
                    if same_file_resolution
                        .resolved_mixin_include_names
                        .contains(&symbol.name) =>
                {
                    "resolved"
                }
                ParsedSassSymbolFactKind::FunctionCall if symbol.namespace.is_some() => "external",
                ParsedSassSymbolFactKind::FunctionCall
                    if same_file_resolution
                        .resolved_function_call_names
                        .contains(&symbol.name) =>
                {
                    "resolved"
                }
                _ => "unresolved",
            };
            selector_symbol_facts.push(ParserIndexSassSelectorSymbolFactV0 {
                selector_name,
                symbol_kind: symbol.symbol_kind,
                name: symbol.name.clone(),
                namespace: symbol.namespace.clone(),
                role: symbol.role,
                resolution,
                byte_span,
                range: parser_range_for_byte_span(source, line_index, byte_span),
            });
        }
    }
    selector_symbol_facts.sort();
    selector_symbol_facts.dedup();

    let mut module_use_sources = BTreeSet::new();
    let mut module_forward_sources = BTreeSet::new();
    let mut module_import_sources = BTreeSet::new();
    let mut module_use_edges = Vec::new();
    let mut module_forward_edges = Vec::new();
    for edge in &facts.sass_module_edges {
        match edge.kind {
            ParsedSassModuleEdgeFactKind::Use => {
                let byte_span = byte_span_for_range(edge.range);
                module_use_sources.insert(edge.source.clone());
                module_use_edges.push(ParserIndexSassModuleUseFactV0 {
                    source: edge.source.clone(),
                    namespace_kind: edge.namespace_kind.unwrap_or("default"),
                    namespace: edge.namespace.clone(),
                    byte_span,
                    range: parser_range_for_byte_span(source, line_index, byte_span),
                });
            }
            ParsedSassModuleEdgeFactKind::Forward => {
                let byte_span = byte_span_for_range(edge.range);
                let rule_byte_span = syntax_index
                    .scss_forward_span_for_offset(byte_span.start)
                    .unwrap_or(byte_span);
                module_forward_sources.insert(edge.source.clone());
                module_forward_edges.push(ParserIndexSassModuleForwardFactV0 {
                    source: edge.source.clone(),
                    prefix: sass_module_forward_prefix_from_statement(source, rule_byte_span),
                    visibility_kind: edge.visibility_filter_kind.unwrap_or("all"),
                    visibility_members: edge
                        .visibility_filter_names
                        .iter()
                        .map(|name| ParserIndexSassModuleForwardMemberV0 {
                            name: name.clone(),
                            symbol_kind: sass_module_forward_member_symbol_kind(
                                source,
                                rule_byte_span,
                                name,
                            ),
                        })
                        .collect(),
                    byte_span,
                    range: parser_range_for_byte_span(source, line_index, byte_span),
                    rule_byte_span,
                    rule_range: parser_range_for_byte_span(source, line_index, rule_byte_span),
                });
            }
            ParsedSassModuleEdgeFactKind::Import => {
                let byte_span = byte_span_for_range(edge.range);
                module_use_sources.insert(edge.source.clone());
                module_import_sources.insert(edge.source.clone());
                module_use_edges.push(ParserIndexSassModuleUseFactV0 {
                    source: edge.source.clone(),
                    namespace_kind: "wildcard",
                    namespace: None,
                    byte_span,
                    range: parser_range_for_byte_span(source, line_index, byte_span),
                });
            }
        }
    }
    module_use_edges.sort();
    module_use_edges.dedup();
    module_forward_edges.sort();
    module_forward_edges.dedup();

    ParserIndexSassFactsV0 {
        variable_decl_names: sorted(variable_decl_names),
        symbol_decl_facts,
        variable_parameter_names: sorted(variable_parameter_names.clone()),
        variable_ref_names: sorted(variable_ref_names),
        selectors_with_variable_refs_names: selector_names_for_variable_symbols(
            blocks,
            facts,
            &global_variable_decl_names,
            &variable_parameter_names,
            &variable_decl_scopes,
            None,
        ),
        selectors_with_resolved_variable_refs_names: selector_names_for_variable_symbols(
            blocks,
            facts,
            &global_variable_decl_names,
            &variable_parameter_names,
            &variable_decl_scopes,
            Some(true),
        ),
        selectors_with_unresolved_variable_refs_names: selector_names_for_variable_symbols(
            blocks,
            facts,
            &global_variable_decl_names,
            &variable_parameter_names,
            &variable_decl_scopes,
            Some(false),
        ),
        mixin_decl_names: sorted(mixin_decl_names),
        mixin_include_names: sorted(mixin_include_names),
        selectors_with_mixin_includes_names: selector_names_for_symbols(
            blocks,
            facts,
            ParsedSassSymbolFactKind::MixinInclude,
            None,
        ),
        selectors_with_resolved_mixin_includes_names: selector_names_for_symbols(
            blocks,
            facts,
            ParsedSassSymbolFactKind::MixinInclude,
            Some(&same_file_resolution.resolved_mixin_include_names),
        ),
        selectors_with_unresolved_mixin_includes_names: selector_names_for_symbols(
            blocks,
            facts,
            ParsedSassSymbolFactKind::MixinInclude,
            Some(&same_file_resolution.unresolved_mixin_include_names),
        ),
        function_decl_names: sorted(function_decl_names),
        function_call_names: sorted(function_call_names),
        selectors_with_function_calls_names: selector_names_for_symbols(
            blocks,
            facts,
            ParsedSassSymbolFactKind::FunctionCall,
            None,
        ),
        selector_symbol_facts,
        module_use_sources: sorted(module_use_sources),
        module_use_edges,
        module_forward_sources: sorted(module_forward_sources),
        module_forward_edges,
        module_import_sources: sorted(module_import_sources),
        same_file_resolution,
    }
}

fn summarize_keyframes(
    source: &str,
    line_index: &SourceLineIndex,
    facts: &ParsedStyleFacts,
    blocks: &[StyleBlock],
    syntax_index: &ProductSyntaxIndexV0,
) -> ParserIndexKeyframesFactsV0 {
    let mut names = Vec::new();
    let mut decl_facts = Vec::new();
    let mut names_under_media = BTreeSet::new();
    let mut names_under_supports = BTreeSet::new();
    let mut names_under_layer = BTreeSet::new();
    let mut animation_ref_names = Vec::new();
    let mut animation_name_ref_names = Vec::new();
    let mut ref_facts = Vec::new();
    let mut selectors_with_animation_ref_names = BTreeSet::new();
    let mut selectors_with_animation_name_ref_names = BTreeSet::new();
    let mut selectors_with_animation_refs_under_media_names = BTreeSet::new();
    let mut selectors_with_animation_refs_under_supports_names = BTreeSet::new();
    let mut selectors_with_animation_refs_under_layer_names = BTreeSet::new();
    let mut selectors_with_animation_name_refs_under_media_names = BTreeSet::new();
    let mut selectors_with_animation_name_refs_under_supports_names = BTreeSet::new();
    let mut selectors_with_animation_name_refs_under_layer_names = BTreeSet::new();
    let declared_keyframes = facts
        .animations
        .iter()
        .filter(|animation| animation.kind == ParsedAnimationFactKind::KeyframesDeclaration)
        .map(|animation| animation.name.clone())
        .collect::<BTreeSet<_>>();

    for animation in &facts.animations {
        let offset = range_start(animation.range);
        let wrapper = wrapper_for_offset(blocks, offset);
        match animation.kind {
            ParsedAnimationFactKind::KeyframesDeclaration => {
                let byte_span = byte_span_for_range(animation.range);
                let rule_byte_span = syntax_index
                    .keyframes_span_for_offset(byte_span.start)
                    .unwrap_or(byte_span);
                decl_facts.push(ParserIndexKeyframesDeclFactV0 {
                    name: animation.name.clone(),
                    source_order: decl_facts.len(),
                    byte_span,
                    range: parser_range_for_byte_span(source, line_index, byte_span),
                    rule_byte_span,
                    rule_range: parser_range_for_byte_span(source, line_index, rule_byte_span),
                });
                names.push(animation.name.clone());
                insert_by_wrapper(
                    &mut names_under_media,
                    &mut names_under_supports,
                    &mut names_under_layer,
                    &animation.name,
                    &wrapper,
                );
            }
            ParsedAnimationFactKind::AnimationNameReference => {
                let byte_span = byte_span_for_range(animation.range);
                let property = if syntax_index
                    .declaration_property_name_for_offset(offset)
                    .is_some_and(|name| name == "animation-name")
                {
                    "animation-name"
                } else {
                    "animation"
                };
                ref_facts.push(ParserIndexAnimationNameRefFactV0 {
                    name: animation.name.clone(),
                    property,
                    source_order: ref_facts.len(),
                    byte_span,
                    range: parser_range_for_byte_span(source, line_index, byte_span),
                });
                if !declared_keyframes.contains(&animation.name) {
                    continue;
                }
                let selectors = selector_names_for_offset(blocks, offset);
                if property == "animation-name" {
                    animation_name_ref_names.push(animation.name.clone());
                    for selector in selectors {
                        selectors_with_animation_name_ref_names.insert(selector.clone());
                        insert_by_wrapper(
                            &mut selectors_with_animation_name_refs_under_media_names,
                            &mut selectors_with_animation_name_refs_under_supports_names,
                            &mut selectors_with_animation_name_refs_under_layer_names,
                            &selector,
                            &wrapper,
                        );
                    }
                } else {
                    animation_ref_names.push(animation.name.clone());
                    for selector in selectors {
                        selectors_with_animation_ref_names.insert(selector.clone());
                        insert_by_wrapper(
                            &mut selectors_with_animation_refs_under_media_names,
                            &mut selectors_with_animation_refs_under_supports_names,
                            &mut selectors_with_animation_refs_under_layer_names,
                            &selector,
                            &wrapper,
                        );
                    }
                }
            }
        }
    }
    decl_facts.sort();
    decl_facts.dedup();
    ref_facts.sort();
    ref_facts.dedup();

    ParserIndexKeyframesFactsV0 {
        names: names.tap_sort_unique(),
        decl_facts,
        names_under_media: sorted(names_under_media),
        names_under_supports: sorted(names_under_supports),
        names_under_layer: sorted(names_under_layer),
        animation_ref_names: animation_ref_names.tap_sort_unique(),
        animation_name_ref_names: animation_name_ref_names.tap_sort_unique(),
        ref_facts,
        selectors_with_animation_ref_names: sorted(selectors_with_animation_ref_names),
        selectors_with_animation_name_ref_names: sorted(selectors_with_animation_name_ref_names),
        selectors_with_animation_refs_under_media_names: sorted(
            selectors_with_animation_refs_under_media_names,
        ),
        selectors_with_animation_refs_under_supports_names: sorted(
            selectors_with_animation_refs_under_supports_names,
        ),
        selectors_with_animation_refs_under_layer_names: sorted(
            selectors_with_animation_refs_under_layer_names,
        ),
        selectors_with_animation_name_refs_under_media_names: sorted(
            selectors_with_animation_name_refs_under_media_names,
        ),
        selectors_with_animation_name_refs_under_supports_names: sorted(
            selectors_with_animation_name_refs_under_supports_names,
        ),
        selectors_with_animation_name_refs_under_layer_names: sorted(
            selectors_with_animation_name_refs_under_layer_names,
        ),
    }
}

fn summarize_composes(
    source: &str,
    line_index: &SourceLineIndex,
    facts: &ParsedStyleFacts,
    blocks: &[StyleBlock],
) -> ParserIndexComposesFactsV0 {
    let mut summary = ParserIndexComposesFactsV0::default();
    for edge in &facts.css_module_composes_edges {
        let byte_span = byte_span_for_range(edge.range);
        summary.edges.push(ParserIndexComposesEdgeFactV0 {
            kind: match edge.kind {
                ParsedCssModuleComposesEdgeKind::Local => "local",
                ParsedCssModuleComposesEdgeKind::External => "external",
                ParsedCssModuleComposesEdgeKind::Global => "global",
            },
            owner_selector_names: edge.owner_selector_names.clone(),
            target_names: edge.target_names.clone(),
            import_source: edge.import_source.clone(),
            class_tokens: composes_class_tokens_for_edge(source, line_index, facts, edge),
            byte_span,
            range: parser_range_for_byte_span(source, line_index, byte_span),
        });
        let wrapper = wrapper_for_offset(blocks, range_start(edge.range));
        let count = edge.owner_selector_names.len() * edge.target_names.len();
        summary.class_name_count += count;
        for owner in &edge.owner_selector_names {
            summary.selectors_with_composes_names.push(owner.clone());
            insert_vec_by_wrapper(
                &mut summary.selectors_with_composes_under_media_names,
                &mut summary.selectors_with_composes_under_supports_names,
                &mut summary.selectors_with_composes_under_layer_names,
                owner,
                &wrapper,
            );
        }
        match edge.kind {
            ParsedCssModuleComposesEdgeKind::Local => {
                summary.local_class_name_count += count;
                for owner in &edge.owner_selector_names {
                    summary.local_selector_names.push(owner.clone());
                    insert_vec_by_wrapper(
                        &mut summary.local_selector_names_under_media,
                        &mut summary.local_selector_names_under_supports,
                        &mut summary.local_selector_names_under_layer,
                        owner,
                        &wrapper,
                    );
                }
            }
            ParsedCssModuleComposesEdgeKind::External => {
                summary.imported_class_name_count += count;
                for owner in &edge.owner_selector_names {
                    summary.imported_selector_names.push(owner.clone());
                    insert_vec_by_wrapper(
                        &mut summary.imported_selector_names_under_media,
                        &mut summary.imported_selector_names_under_supports,
                        &mut summary.imported_selector_names_under_layer,
                        owner,
                        &wrapper,
                    );
                    if let Some(source) = &edge.import_source {
                        summary.import_sources.push(source.clone());
                        if wrapper.under_media {
                            summary.import_sources_under_media.push(source.clone());
                        }
                        if wrapper.under_supports {
                            summary.import_sources_under_supports.push(source.clone());
                        }
                        if wrapper.under_layer {
                            summary.import_sources_under_layer.push(source.clone());
                        }
                    }
                }
            }
            ParsedCssModuleComposesEdgeKind::Global => {
                summary.global_class_name_count += count;
                for owner in &edge.owner_selector_names {
                    summary.global_selector_names.push(owner.clone());
                    insert_vec_by_wrapper(
                        &mut summary.global_selector_names_under_media,
                        &mut summary.global_selector_names_under_supports,
                        &mut summary.global_selector_names_under_layer,
                        owner,
                        &wrapper,
                    );
                }
            }
        }
    }
    sort_all_composes(&mut summary);
    summary.edges.sort();
    summary.edges.dedup();
    summary
}

fn composes_class_tokens_for_edge(
    source: &str,
    line_index: &SourceLineIndex,
    facts: &ParsedStyleFacts,
    edge: &crate::ParsedCssModuleComposesEdgeFact,
) -> Vec<ParserIndexComposesClassTokenV0> {
    let target_names = edge
        .target_names
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let edge_start = range_start(edge.range);
    let edge_end = u32::from(edge.range.end()) as usize;
    let mut class_tokens = facts
        .css_module_composes
        .iter()
        .filter(|fact| fact.kind == ParsedCssModuleComposesFactKind::Target)
        .filter(|fact| target_names.contains(fact.name.as_str()))
        .filter(|fact| {
            let token_start = range_start(fact.range);
            let token_end = u32::from(fact.range.end()) as usize;
            token_start >= edge_start && token_end <= edge_end
        })
        .map(|fact| {
            let byte_span = byte_span_for_range(fact.range);
            ParserIndexComposesClassTokenV0 {
                class_name: fact.name.clone(),
                byte_span,
                range: parser_range_for_byte_span(source, line_index, byte_span),
            }
        })
        .collect::<Vec<_>>();
    class_tokens.sort();
    class_tokens.dedup();
    class_tokens
}

fn summarize_wrappers(blocks: &[StyleBlock]) -> ParserIndexWrapperFactsV0 {
    ParserIndexWrapperFactsV0 {
        selectors_under_media_names: sorted(
            blocks
                .iter()
                .filter(|block| block.under_media)
                .flat_map(|block| {
                    block
                        .names
                        .iter()
                        .filter(|name| !name.starts_with("__selector_meta:"))
                        .cloned()
                })
                .collect(),
        ),
        selectors_under_supports_names: sorted(
            blocks
                .iter()
                .filter(|block| block.under_supports)
                .flat_map(|block| {
                    block
                        .names
                        .iter()
                        .filter(|name| !name.starts_with("__selector_meta:"))
                        .cloned()
                })
                .collect(),
        ),
        selectors_under_layer_names: sorted(
            blocks
                .iter()
                .filter(|block| block.under_layer)
                .flat_map(|block| {
                    block
                        .names
                        .iter()
                        .filter(|name| !name.starts_with("__selector_meta:"))
                        .cloned()
                })
                .collect(),
        ),
    }
}

fn resolve_selector_header_text(
    source: &str,
    header: &str,
    parent_branches: &[SelectorBranch],
) -> Vec<SelectorBranch> {
    split_selector_groups_text(header)
        .into_iter()
        .flat_map(|group| resolve_selector_group_text(source, header, group, parent_branches))
        .collect()
}

fn resolve_selector_group_text(
    source: &str,
    full_header: &str,
    group: &str,
    parent_branches: &[SelectorBranch],
) -> Vec<SelectorBranch> {
    let group = group.trim();
    if group.starts_with(":global") && !group.starts_with(":local") {
        return Vec::new();
    }
    let tail = selector_tail(group);
    if let Some(suffix) = tail.strip_prefix('&').map(str::trim)
        && is_ampersand_suffix_text(suffix)
    {
        let span = source_span_for_header_piece(source, full_header, suffix);
        return parent_branches
            .iter()
            .map(|parent| SelectorBranch {
                name: format!("{}{}", parent.name, suffix),
                name_span: span,
                bare_suffix_base: parent.bare_suffix_base,
                amp_suffix_depth: parent.amp_suffix_depth + 1,
            })
            .collect();
    }
    let names = class_names_in_selector(tail, source, full_header);
    let bare_suffix_base = parent_branches.is_empty() && names.len() == 1;
    names
        .into_iter()
        .map(|(name, name_span)| SelectorBranch {
            name,
            name_span,
            bare_suffix_base,
            amp_suffix_depth: 0,
        })
        .collect()
}

fn is_ampersand_suffix_text(suffix: &str) -> bool {
    suffix
        .chars()
        .next()
        .is_some_and(|ch| ch == '-' || ch == '_' || ch.is_ascii_alphanumeric())
}

fn classify_nested_safety(
    header: &str,
    branches: &[SelectorBranch],
    parent_branches: &[SelectorBranch],
    parent_is_grouped: bool,
) -> &'static str {
    if branches.is_empty() {
        return "flat";
    }
    let is_nested = !parent_branches.is_empty() || header.contains('&');
    if !is_nested {
        return "flat";
    }
    let header = header.trim();
    let bem_suffix_safe = branches.len() == 1
        && parent_branches.len() == 1
        && parent_branches[0].bare_suffix_base
        && !parent_is_grouped
        && header.starts_with('&')
        && (header[1..].trim_start().starts_with("__")
            || header[1..].trim_start().starts_with("--"));
    let chained_bem_modifier_safe = header.starts_with('&')
        && header[1..].trim_start().starts_with("--")
        && !parent_branches.is_empty()
        && parent_branches
            .iter()
            .all(|parent| parent.amp_suffix_depth > 0);
    if bem_suffix_safe || chained_bem_modifier_safe {
        "bemSuffixSafe"
    } else {
        "nestedUnsafe"
    }
}

fn nested_safety_for_selector(blocks: &[StyleBlock], name: &str) -> Option<&'static str> {
    blocks.iter().find_map(|block| {
        block.names.iter().find_map(|entry| {
            entry
                .strip_prefix("__selector_meta:")
                .and_then(|rest| rest.rsplit_once(':'))
                .and_then(|(entry_name, kind)| {
                    (entry_name == name).then_some(match kind {
                        "bemSuffixSafe" => "bemSuffixSafe",
                        "nestedUnsafe" => "nestedUnsafe",
                        _ => "flat",
                    })
                })
        })
    })
}

fn selector_rule_block<'a>(
    blocks: &'a [StyleBlock],
    name: &str,
    selector_offset: usize,
) -> Option<&'a StyleBlock> {
    blocks
        .iter()
        .filter(|block| block.start <= selector_offset && selector_offset < block.end)
        .filter(|block| {
            block.names.iter().any(|entry| {
                entry
                    .strip_prefix("__selector_meta:")
                    .and_then(|rest| rest.rsplit_once(':'))
                    .is_some_and(|(entry_name, _)| entry_name == name)
            })
        })
        .max_by_key(|block| block.rule_start)
}

fn style_block_for_offset(blocks: &[StyleBlock], offset: usize) -> Option<&StyleBlock> {
    blocks
        .iter()
        .filter(|block| !block.names.is_empty())
        .filter(|block| block.start <= offset && offset < block.end)
        .max_by_key(|block| block.rule_start)
}

fn split_selector_groups_text(header: &str) -> Vec<&str> {
    let mut groups = Vec::new();
    let mut start = 0usize;
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;
    for (index, byte) in header.bytes().enumerate() {
        match byte {
            b'(' => paren_depth += 1,
            b')' => paren_depth = paren_depth.saturating_sub(1),
            b'[' => bracket_depth += 1,
            b']' => bracket_depth = bracket_depth.saturating_sub(1),
            b',' if paren_depth == 0 && bracket_depth == 0 => {
                groups.push(&header[start..index]);
                start = index + 1;
            }
            _ => {}
        }
    }
    groups.push(&header[start..]);
    groups
}

fn selector_tail(group: &str) -> &str {
    let mut tail_start = 0usize;
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;
    let bytes = group.as_bytes();
    let mut index = 0usize;
    while index < bytes.len() {
        match bytes[index] {
            b'(' => paren_depth += 1,
            b')' => paren_depth = paren_depth.saturating_sub(1),
            b'[' => bracket_depth += 1,
            b']' => bracket_depth = bracket_depth.saturating_sub(1),
            b'>' | b'+' | b'~' if paren_depth == 0 && bracket_depth == 0 => tail_start = index + 1,
            byte if byte.is_ascii_whitespace() && paren_depth == 0 && bracket_depth == 0 => {
                let previous = group[..index].trim_end().as_bytes().last().copied();
                let next = group[index + 1..].trim_start().as_bytes().first().copied();
                if previous.is_some()
                    && next.is_some_and(|value| value == b'.' || value == b':' || value == b'&')
                {
                    tail_start = index + 1;
                }
            }
            _ => {}
        }
        index += 1;
    }
    group[tail_start..].trim()
}

fn class_names_in_selector(
    selector: &str,
    source: &str,
    full_header: &str,
) -> Vec<(String, ParserByteSpanV0)> {
    let mut names = Vec::new();
    let mut index = 0usize;
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;
    let bytes = selector.as_bytes();
    while index < bytes.len() {
        match bytes[index] {
            b'(' => paren_depth += 1,
            b')' => paren_depth = paren_depth.saturating_sub(1),
            b'[' => bracket_depth += 1,
            b']' => bracket_depth = bracket_depth.saturating_sub(1),
            b'.' if paren_depth == 0 && bracket_depth == 0 => {
                let start = index + 1;
                let mut end = start;
                while end < bytes.len()
                    && (bytes[end].is_ascii_alphanumeric() || matches!(bytes[end], b'_' | b'-'))
                {
                    end += 1;
                }
                if end > start {
                    let name = selector[start..end].to_string();
                    names.push((
                        name.clone(),
                        source_span_for_header_piece(source, full_header, &name),
                    ));
                }
                index = end;
                continue;
            }
            _ => {}
        }
        index += 1;
    }
    names
}

fn selector_names_for_offset(blocks: &[StyleBlock], offset: usize) -> Vec<String> {
    let Some(max_start) = blocks
        .iter()
        .filter(|block| block.start <= offset && offset < block.end && !block.names.is_empty())
        .map(|block| block.start)
        .max()
    else {
        return Vec::new();
    };
    blocks
        .iter()
        .filter(|block| block.start == max_start && block.start <= offset && offset < block.end)
        .flat_map(|block| {
            block
                .names
                .iter()
                .filter(|name| !name.starts_with("__selector_meta:"))
                .cloned()
        })
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn selector_contexts_for_offset(blocks: &[StyleBlock], offset: usize) -> Vec<String> {
    let Some(max_start) = blocks
        .iter()
        .filter(|block| block.start <= offset && offset < block.end)
        .map(|block| block.start)
        .max()
    else {
        return Vec::new();
    };
    let mut contexts = BTreeSet::new();
    for block in blocks
        .iter()
        .filter(|block| block.start == max_start && block.start <= offset && offset < block.end)
    {
        if block.names.is_empty() {
            if let Some(context) = &block.context_text {
                contexts.insert(context.clone());
            }
        } else {
            for name in &block.names {
                if !name.starts_with("__selector_meta:") {
                    contexts.insert(format!(".{name}"));
                }
            }
        }
    }
    contexts.into_iter().collect()
}

fn wrapper_for_offset(blocks: &[StyleBlock], offset: usize) -> WrapperContext {
    blocks
        .iter()
        .filter(|block| block.start <= offset && offset < block.end)
        .max_by_key(|block| block.start)
        .map(|block| WrapperContext {
            under_media: block.under_media,
            under_supports: block.under_supports,
            under_layer: block.under_layer,
            wrapper_at_rules: block.wrapper_at_rules.clone(),
        })
        .unwrap_or_default()
}

fn insert_by_wrapper(
    media: &mut BTreeSet<String>,
    supports: &mut BTreeSet<String>,
    layer: &mut BTreeSet<String>,
    value: &str,
    wrapper: &WrapperContext,
) {
    if wrapper.under_media {
        media.insert(value.to_string());
    }
    if wrapper.under_supports {
        supports.insert(value.to_string());
    }
    if wrapper.under_layer {
        layer.insert(value.to_string());
    }
}

fn insert_vec_by_wrapper(
    media: &mut Vec<String>,
    supports: &mut Vec<String>,
    layer: &mut Vec<String>,
    value: &str,
    wrapper: &WrapperContext,
) {
    if wrapper.under_media {
        media.push(value.to_string());
    }
    if wrapper.under_supports {
        supports.push(value.to_string());
    }
    if wrapper.under_layer {
        layer.push(value.to_string());
    }
}

fn selector_names_from_contexts(contexts: &[String]) -> Vec<String> {
    contexts
        .iter()
        .filter_map(|context| context.strip_prefix('.').map(ToString::to_string))
        .collect()
}

fn selector_names_for_variable_symbols(
    blocks: &[StyleBlock],
    facts: &ParsedStyleFacts,
    global_variable_decl_names: &BTreeSet<String>,
    variable_parameter_names: &BTreeSet<String>,
    variable_decl_scopes: &[SassVariableDeclScope],
    resolved_filter: Option<bool>,
) -> Vec<String> {
    facts
        .sass_symbols
        .iter()
        .filter(|symbol| symbol.kind == ParsedSassSymbolFactKind::VariableReference)
        .filter(|symbol| {
            let Some(expected_resolved) = resolved_filter else {
                return true;
            };
            if symbol.namespace.is_some() {
                return false;
            }
            is_sass_variable_reference_resolved(
                &symbol.name,
                range_start(symbol.range),
                blocks,
                global_variable_decl_names,
                variable_parameter_names,
                variable_decl_scopes,
            ) == expected_resolved
        })
        .flat_map(|symbol| selector_names_for_offset(blocks, range_start(symbol.range)))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn is_sass_variable_reference_resolved(
    name: &str,
    offset: usize,
    blocks: &[StyleBlock],
    global_variable_decl_names: &BTreeSet<String>,
    variable_parameter_names: &BTreeSet<String>,
    variable_decl_scopes: &[SassVariableDeclScope],
) -> bool {
    if global_variable_decl_names.contains(name) || variable_parameter_names.contains(name) {
        return true;
    }
    let reference_selectors = selector_names_for_offset(blocks, offset);
    !reference_selectors.is_empty()
        && variable_decl_scopes.iter().any(|scope| {
            scope.name == name
                && !scope.selector_names.is_empty()
                && scope
                    .selector_names
                    .iter()
                    .any(|selector| reference_selectors.contains(selector))
        })
}

fn selector_names_for_symbols(
    blocks: &[StyleBlock],
    facts: &ParsedStyleFacts,
    kind: ParsedSassSymbolFactKind,
    names_filter: Option<&[String]>,
) -> Vec<String> {
    facts
        .sass_symbols
        .iter()
        .filter(|symbol| symbol.kind == kind)
        .filter(|symbol| {
            names_filter
                .map(|names| names.contains(&symbol.name))
                .unwrap_or(true)
        })
        .flat_map(|symbol| selector_names_for_offset(blocks, range_start(symbol.range)))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn sass_module_forward_prefix_from_statement(source: &str, span: ParserByteSpanV0) -> String {
    let Some(statement) = source.get(span.start..span.end) else {
        return String::new();
    };
    let Some(as_index) = css_keyword(statement).find(" as ") else {
        return String::new();
    };
    let after_as = &statement[as_index + 4..];
    let Some(star_index) = after_as.find('*') else {
        return String::new();
    };
    after_as[..star_index].trim().to_string()
}

fn sass_module_forward_member_symbol_kind(
    source: &str,
    span: ParserByteSpanV0,
    name: &str,
) -> Option<&'static str> {
    let statement = source.get(span.start..span.end)?;
    statement
        .contains(&format!("${name}"))
        .then_some("variable")
}

fn sort_all_composes(summary: &mut ParserIndexComposesFactsV0) {
    sort_unique(&mut summary.selectors_with_composes_names);
    sort_unique(&mut summary.selectors_with_composes_under_media_names);
    sort_unique(&mut summary.selectors_with_composes_under_supports_names);
    sort_unique(&mut summary.selectors_with_composes_under_layer_names);
    sort_unique(&mut summary.local_selector_names);
    sort_unique(&mut summary.imported_selector_names);
    sort_unique(&mut summary.global_selector_names);
    sort_unique(&mut summary.local_selector_names_under_media);
    sort_unique(&mut summary.local_selector_names_under_supports);
    sort_unique(&mut summary.local_selector_names_under_layer);
    sort_unique(&mut summary.imported_selector_names_under_media);
    sort_unique(&mut summary.imported_selector_names_under_supports);
    sort_unique(&mut summary.imported_selector_names_under_layer);
    sort_unique(&mut summary.global_selector_names_under_media);
    sort_unique(&mut summary.global_selector_names_under_supports);
    sort_unique(&mut summary.global_selector_names_under_layer);
    summary.import_sources.sort();
    summary.import_sources_under_media.sort();
    summary.import_sources_under_supports.sort();
    summary.import_sources_under_layer.sort();
}

fn sort_unique(values: &mut Vec<String>) {
    values.sort();
    values.dedup();
}

fn source_span_for_header_piece(source: &str, full_header: &str, piece: &str) -> ParserByteSpanV0 {
    if let Some(header_offset) = source.find(full_header)
        && let Some(piece_offset) = full_header.find(piece)
    {
        let start = header_offset + piece_offset;
        return ParserByteSpanV0 {
            start,
            end: start + piece.len(),
        };
    }
    ParserByteSpanV0 {
        start: 0,
        end: piece.len(),
    }
}

fn byte_span_for_range(range: TextRange) -> ParserByteSpanV0 {
    ParserByteSpanV0 {
        start: range_start(range),
        end: u32::from(range.end()) as usize,
    }
}

fn range_start(range: TextRange) -> usize {
    u32::from(range.start()) as usize
}

struct SourceLineIndex {
    line_starts: Vec<usize>,
}

impl SourceLineIndex {
    fn new(source: &str) -> Self {
        let mut line_starts = vec![0];
        for (index, byte) in source.as_bytes().iter().enumerate() {
            if *byte == b'\n' {
                line_starts.push(index + 1);
            }
        }
        Self { line_starts }
    }

    fn position_for_byte_offset(&self, source: &str, byte_offset: usize) -> ParserPositionV0 {
        let offset = byte_offset.min(source.len());
        let line = self.line_starts.partition_point(|start| *start <= offset);
        let line_index = line.saturating_sub(1);
        let line_start = self.line_starts.get(line_index).copied().unwrap_or(0);
        ParserPositionV0 {
            line: line_index,
            character: source
                .get(line_start..offset)
                .map(|text| text.encode_utf16().count())
                .unwrap_or_else(|| offset.saturating_sub(line_start)),
        }
    }
}

fn parser_range_for_byte_span(
    source: &str,
    line_index: &SourceLineIndex,
    span: ParserByteSpanV0,
) -> ParserRangeV0 {
    ParserRangeV0 {
        start: line_index.position_for_byte_offset(source, span.start),
        end: line_index.position_for_byte_offset(source, span.end),
    }
}

fn bem_suffix_parent_name(name: &str) -> Option<String> {
    let marker = [name.rfind("__"), name.rfind("--")]
        .into_iter()
        .flatten()
        .max()?;
    (marker > 0).then(|| name[..marker].to_string())
}

fn sorted(values: BTreeSet<String>) -> Vec<String> {
    values.into_iter().collect()
}

trait SortVec {
    fn tap_sort(self) -> Self;
    fn tap_sort_unique(self) -> Self;
}

impl SortVec for Vec<String> {
    fn tap_sort(mut self) -> Self {
        self.sort();
        self
    }

    fn tap_sort_unique(mut self) -> Self {
        self.sort();
        self.dedup();
        self
    }
}

pub fn dialect_for_path(file_path: &str) -> StyleDialect {
    if file_path.ends_with(".sass") || file_path.ends_with(".module.sass") {
        StyleDialect::Sass
    } else if file_path.ends_with(".scss") || file_path.ends_with(".module.scss") {
        StyleDialect::Scss
    } else if file_path.ends_with(".less") || file_path.ends_with(".module.less") {
        StyleDialect::Less
    } else {
        StyleDialect::Css
    }
}

fn dialect_label(dialect: StyleDialect) -> &'static str {
    match dialect {
        StyleDialect::Css => "css",
        StyleDialect::Scss => "scss",
        StyleDialect::Sass => "sass",
        StyleDialect::Less => "less",
    }
}
