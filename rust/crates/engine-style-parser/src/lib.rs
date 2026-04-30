use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StyleLanguage {
    Css,
    Scss,
    Less,
}

impl StyleLanguage {
    pub fn from_module_path(path: &str) -> Option<Self> {
        if path.ends_with(".module.css") || path.ends_with(".css") {
            Some(Self::Css)
        } else if path.ends_with(".module.scss") || path.ends_with(".scss") {
            Some(Self::Scss)
        } else if path.ends_with(".module.less") || path.ends_with(".less") {
            Some(Self::Less)
        } else {
            None
        }
    }

    fn supports_line_comments(self) -> bool {
        matches!(self, Self::Scss | Self::Less)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TextSpan {
    pub start: usize,
    pub end: usize,
}

impl TextSpan {
    fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    Whitespace,
    Ident,
    Number,
    String,
    LineComment,
    BlockComment,
    Dot,
    Ampersand,
    Hash,
    Colon,
    Semicolon,
    Comma,
    At,
    OpenBrace,
    CloseBrace,
    OpenParen,
    CloseParen,
    OpenBracket,
    CloseBracket,
    InterpolationStart,
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: TextSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseDiagnostic {
    pub message: String,
    pub span: TextSpan,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyntaxNodeKind {
    Rule,
    AtRule,
    Declaration,
    Comment,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyntaxNode {
    pub kind: SyntaxNodeKind,
    pub span: TextSpan,
    pub header_span: Option<TextSpan>,
    pub payload: Option<SyntaxNodePayload>,
    pub children: Vec<SyntaxNode>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SyntaxNodePayload {
    Rule(RulePayload),
    AtRule(AtRulePayload),
    Declaration(DeclarationPayload),
    Comment(CommentPayload),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RulePayload {
    pub prelude: String,
    pub selector_groups: Vec<SelectorGroup>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectorGroup {
    pub raw: String,
    pub segments: Vec<SelectorSegment>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SelectorSegment {
    ClassName(String),
    Ampersand,
    BemSuffix(String),
    AmpersandSuffix(String),
    Pseudo(String),
    Combinator(String),
    Other(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AtRulePayload {
    pub kind: AtRuleKind,
    pub name: String,
    pub params: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AtRuleKind {
    Media,
    Supports,
    Layer,
    Keyframes,
    Value,
    AtRoot,
    Mixin,
    Include,
    Function,
    Use,
    Forward,
    Import,
    Generic,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeclarationPayload {
    pub property: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommentPayload {
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stylesheet {
    pub language: StyleLanguage,
    pub source: String,
    pub tokens: Vec<Token>,
    pub nodes: Vec<SyntaxNode>,
    pub diagnostics: Vec<ParseDiagnostic>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ParserParityLiteSummaryV0 {
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
    pub at_rule_kind_counts: AtRuleKindCountsV0,
    pub declaration_kind_counts: DeclarationKindCountsV0,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ParserIndexSummaryV0 {
    pub schema_version: &'static str,
    pub language: &'static str,
    pub selectors: ParserIndexSelectorFactsV0,
    pub values: ParserIndexValueFactsV0,
    pub custom_properties: ParserIndexCustomPropertyFactsV0,
    pub sass: ParserIndexSassFactsV0,
    pub keyframes: ParserIndexKeyframesFactsV0,
    pub composes: ParserIndexComposesFactsV0,
    pub wrappers: ParserIndexWrapperFactsV0,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ParserSemanticBoundarySummaryV0 {
    pub schema_version: &'static str,
    pub language: &'static str,
    pub parser_facts: ParserBoundarySyntaxFactsV0,
    pub semantic_facts: StyleSemanticFactsV0,
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StyleSemanticFactsV0 {
    pub selector_identity: StyleSelectorIdentityFactsV0,
    pub custom_properties: StyleCustomPropertySemanticFactsV0,
    pub sass: StyleSassSemanticFactsV0,
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ParserCanonicalCandidateBundleV0 {
    pub schema_version: &'static str,
    pub language: &'static str,
    pub parity_lite: ParserParityLiteSummaryV0,
    pub css_modules_intermediate: ParserIndexSummaryV0,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ParserEvaluatorCandidateV0 {
    pub kind: &'static str,
    pub selector_name: String,
    pub nested_safety_kind: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bem_suffix_parent_name: Option<String>,
    pub under_media: bool,
    pub under_supports: bool,
    pub under_layer: bool,
    pub has_value_refs: bool,
    pub has_local_value_refs: bool,
    pub has_imported_value_refs: bool,
    pub has_custom_property_refs: bool,
    pub has_animation_ref: bool,
    pub has_animation_name_ref: bool,
    pub has_composes: bool,
    pub has_local_composes: bool,
    pub has_imported_composes: bool,
    pub has_global_composes: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ParserEvaluatorCandidatesV0 {
    pub schema_version: &'static str,
    pub language: &'static str,
    pub results: Vec<ParserEvaluatorCandidateV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ParserCanonicalProducerSignalV0 {
    pub schema_version: &'static str,
    pub language: &'static str,
    pub canonical_candidate: ParserCanonicalCandidateBundleV0,
    pub evaluator_candidates: ParserEvaluatorCandidatesV0,
    pub public_product_gate: ParserPublicProductGateSignalV0,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ParserPublicProductGateSignalV0 {
    pub canonical_candidate_command: &'static str,
    pub consumer_boundary_command: &'static str,
    pub public_product_gate_command: &'static str,
    pub included_in_parser_lane: bool,
    pub included_in_rust_lane_bundle: bool,
    pub included_in_rust_release_bundle: bool,
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
    pub source_order: usize,
    pub byte_span: ParserByteSpanV0,
    pub range: ParserRangeV0,
    pub selector_contexts: Vec<String>,
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
    pub under_media: bool,
    pub under_supports: bool,
    pub under_layer: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ParserIndexSassFactsV0 {
    pub variable_decl_names: Vec<String>,
    pub symbol_decl_facts: Vec<ParserIndexSassSymbolDeclFactV0>,
    pub variable_parameter_names: Vec<String>,
    pub variable_ref_names: Vec<String>,
    pub selectors_with_variable_refs_names: Vec<String>,
    pub selectors_with_resolved_variable_refs_names: Vec<String>,
    pub selectors_with_unresolved_variable_refs_names: Vec<String>,
    pub mixin_decl_names: Vec<String>,
    pub mixin_include_names: Vec<String>,
    pub selectors_with_mixin_includes_names: Vec<String>,
    pub selectors_with_resolved_mixin_includes_names: Vec<String>,
    pub selectors_with_unresolved_mixin_includes_names: Vec<String>,
    pub function_decl_names: Vec<String>,
    pub function_call_names: Vec<String>,
    pub selectors_with_function_calls_names: Vec<String>,
    pub selector_symbol_facts: Vec<ParserIndexSassSelectorSymbolFactV0>,
    pub module_use_sources: Vec<String>,
    pub module_use_edges: Vec<ParserIndexSassModuleUseFactV0>,
    pub module_forward_sources: Vec<String>,
    pub module_import_sources: Vec<String>,
    pub same_file_resolution: ParserIndexSassSameFileResolutionFactsV0,
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

impl From<TextSpan> for ParserByteSpanV0 {
    fn from(span: TextSpan) -> Self {
        Self {
            start: span.start,
            end: span.end,
        }
    }
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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ParserIndexSassSymbolDeclFactV0 {
    pub symbol_kind: &'static str,
    pub name: String,
    pub role: &'static str,
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
pub struct AtRuleKindCountsV0 {
    pub media: usize,
    pub supports: usize,
    pub layer: usize,
    pub keyframes: usize,
    pub value: usize,
    pub at_root: usize,
    pub generic: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct DeclarationKindCountsV0 {
    pub composes: usize,
    pub animation: usize,
    pub animation_name: usize,
    pub generic: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct NestedSafetyCountsV0 {
    pub flat: usize,
    pub bem_suffix_safe: usize,
    pub nested_unsafe: usize,
}

#[derive(Debug, Default)]
struct ParityLiteAcc {
    selector_names: Vec<String>,
    keyframes_names: Vec<String>,
    value_decl_names: Vec<String>,
    rule_count: usize,
    declaration_count: usize,
    grouped_selector_count: usize,
    max_nesting_depth: usize,
    at_rule_kind_counts: AtRuleKindCountsV0,
    declaration_kind_counts: DeclarationKindCountsV0,
}

#[derive(Debug, Default)]
struct IndexSummaryAcc {
    selector_names: Vec<String>,
    selector_definition_facts: Vec<ParserIndexSelectorDefinitionFactV0>,
    bem_suffix_parent_names: Vec<String>,
    bem_suffix_safe_selector_names: Vec<String>,
    selectors_with_composes_names: Vec<String>,
    selectors_with_composes_under_media_names: Vec<String>,
    selectors_with_composes_under_supports_names: Vec<String>,
    selectors_with_composes_under_layer_names: Vec<String>,
    local_composes_selector_names: Vec<String>,
    imported_composes_selector_names: Vec<String>,
    global_composes_selector_names: Vec<String>,
    local_composes_selector_names_under_media: Vec<String>,
    local_composes_selector_names_under_supports: Vec<String>,
    local_composes_selector_names_under_layer: Vec<String>,
    imported_composes_selector_names_under_media: Vec<String>,
    imported_composes_selector_names_under_supports: Vec<String>,
    imported_composes_selector_names_under_layer: Vec<String>,
    global_composes_selector_names_under_media: Vec<String>,
    global_composes_selector_names_under_supports: Vec<String>,
    global_composes_selector_names_under_layer: Vec<String>,
    composes_import_sources: Vec<String>,
    composes_import_sources_under_media: Vec<String>,
    composes_import_sources_under_supports: Vec<String>,
    composes_import_sources_under_layer: Vec<String>,
    keyframes_names: Vec<String>,
    nested_unsafe_selector_names: Vec<String>,
    value_decl_names: Vec<String>,
    value_decl_names_with_local_refs: Vec<String>,
    value_decl_names_with_imported_refs: Vec<String>,
    value_import_names: Vec<String>,
    value_import_sources: Vec<String>,
    value_import_source_by_name: BTreeMap<String, String>,
    value_ref_names: Vec<String>,
    local_value_ref_names: Vec<String>,
    imported_value_ref_names: Vec<String>,
    imported_value_ref_sources: Vec<String>,
    declaration_value_ref_names: Vec<String>,
    declaration_imported_value_ref_sources: Vec<String>,
    value_decl_ref_names: Vec<String>,
    value_decl_imported_value_ref_sources: Vec<String>,
    custom_property_decl_names: Vec<String>,
    custom_property_decl_facts: Vec<ParserIndexCustomPropertyDeclFactV0>,
    custom_property_decl_context_selectors: Vec<String>,
    custom_property_decl_names_under_media: Vec<String>,
    custom_property_decl_names_under_supports: Vec<String>,
    custom_property_decl_names_under_layer: Vec<String>,
    custom_property_ref_names: Vec<String>,
    custom_property_ref_facts: Vec<ParserIndexCustomPropertyRefFactV0>,
    selectors_with_custom_property_refs_names: Vec<String>,
    selectors_with_custom_property_refs_under_media_names: Vec<String>,
    selectors_with_custom_property_refs_under_supports_names: Vec<String>,
    selectors_with_custom_property_refs_under_layer_names: Vec<String>,
    sass_variable_decl_names: Vec<String>,
    sass_variable_decl_facts: Vec<SassVariableDeclFact>,
    sass_symbol_decl_facts: Vec<ParserIndexSassSymbolDeclFactV0>,
    sass_variable_parameter_names: Vec<String>,
    sass_variable_ref_names: Vec<String>,
    sass_variable_ref_facts: Vec<SassVariableRefFact>,
    sass_selectors_with_variable_refs_names: Vec<String>,
    sass_selectors_with_resolved_variable_refs_names: Vec<String>,
    sass_selectors_with_unresolved_variable_refs_names: Vec<String>,
    sass_mixin_decl_names: Vec<String>,
    sass_mixin_include_names: Vec<String>,
    sass_selectors_with_mixin_includes_names: Vec<String>,
    sass_selectors_with_resolved_mixin_includes_names: Vec<String>,
    sass_selectors_with_unresolved_mixin_includes_names: Vec<String>,
    sass_function_decl_names: Vec<String>,
    sass_function_call_names: Vec<String>,
    sass_selectors_with_function_calls_names: Vec<String>,
    sass_selector_symbol_facts: Vec<ParserIndexSassSelectorSymbolFactV0>,
    sass_module_use_sources: Vec<String>,
    sass_module_use_edges: Vec<ParserIndexSassModuleUseFactV0>,
    sass_module_forward_sources: Vec<String>,
    sass_module_import_sources: Vec<String>,
    selectors_with_value_refs_names: Vec<String>,
    selectors_with_local_value_refs_names: Vec<String>,
    selectors_with_imported_value_refs_names: Vec<String>,
    selectors_with_value_refs_under_media_names: Vec<String>,
    selectors_with_value_refs_under_supports_names: Vec<String>,
    selectors_with_value_refs_under_layer_names: Vec<String>,
    selectors_with_local_value_refs_under_media_names: Vec<String>,
    selectors_with_local_value_refs_under_supports_names: Vec<String>,
    selectors_with_local_value_refs_under_layer_names: Vec<String>,
    selectors_with_imported_value_refs_under_media_names: Vec<String>,
    selectors_with_imported_value_refs_under_supports_names: Vec<String>,
    selectors_with_imported_value_refs_under_layer_names: Vec<String>,
    selectors_with_animation_ref_names: Vec<String>,
    selectors_with_animation_refs_under_media_names: Vec<String>,
    selectors_with_animation_refs_under_supports_names: Vec<String>,
    selectors_with_animation_refs_under_layer_names: Vec<String>,
    selectors_with_animation_name_ref_names: Vec<String>,
    selectors_with_animation_name_refs_under_media_names: Vec<String>,
    selectors_with_animation_name_refs_under_supports_names: Vec<String>,
    selectors_with_animation_name_refs_under_layer_names: Vec<String>,
    selectors_under_media_names: Vec<String>,
    selectors_under_supports_names: Vec<String>,
    selectors_under_layer_names: Vec<String>,
    animation_ref_names: Vec<String>,
    animation_name_ref_names: Vec<String>,
    keyframes_names_under_media: Vec<String>,
    keyframes_names_under_supports: Vec<String>,
    keyframes_names_under_layer: Vec<String>,
    value_import_alias_count: usize,
    composes_class_name_count: usize,
    local_composes_class_name_count: usize,
    imported_composes_class_name_count: usize,
    global_composes_class_name_count: usize,
    bem_suffix_count: usize,
    nested_safety_counts: NestedSafetyCountsV0,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ResolvedSelectorBranch {
    name: String,
    bare_suffix_base: bool,
}

pub fn parse_style_module(path: &str, source: &str) -> Option<Stylesheet> {
    let language = StyleLanguage::from_module_path(path)?;
    Some(parse_stylesheet(language, source))
}

pub fn parse_stylesheet(language: StyleLanguage, source: &str) -> Stylesheet {
    let (tokens, mut diagnostics) = tokenize(language, source);
    let mut parser = Parser::new(source, &tokens, &mut diagnostics);
    let nodes = parser.parse_root();
    Stylesheet {
        language,
        source: source.to_string(),
        tokens,
        nodes,
        diagnostics,
    }
}

pub fn summarize_parity_lite(sheet: &Stylesheet) -> ParserParityLiteSummaryV0 {
    let mut acc = ParityLiteAcc::default();
    collect_parity_names(&sheet.nodes, &mut acc);
    acc.selector_names.sort();
    acc.keyframes_names.sort();
    acc.keyframes_names.dedup();
    acc.value_decl_names.sort();
    acc.value_decl_names.dedup();

    ParserParityLiteSummaryV0 {
        schema_version: "0",
        language: match sheet.language {
            StyleLanguage::Css => "css",
            StyleLanguage::Scss => "scss",
            StyleLanguage::Less => "less",
        },
        selector_names: acc.selector_names,
        keyframes_names: acc.keyframes_names,
        value_decl_names: acc.value_decl_names,
        diagnostic_count: sheet.diagnostics.len(),
        rule_count: acc.rule_count,
        declaration_count: acc.declaration_count,
        grouped_selector_count: acc.grouped_selector_count,
        max_nesting_depth: acc.max_nesting_depth,
        at_rule_kind_counts: acc.at_rule_kind_counts,
        declaration_kind_counts: acc.declaration_kind_counts,
    }
}

pub fn summarize_css_modules_intermediate(sheet: &Stylesheet) -> ParserIndexSummaryV0 {
    let mut acc = IndexSummaryAcc::default();
    collect_index_names(
        &sheet.source,
        &sheet.nodes,
        &mut acc,
        &[],
        false,
        None,
        WrapperContext::default(),
    );
    let local_value_names: BTreeSet<String> = acc.value_decl_names.iter().cloned().collect();
    let imported_value_names: BTreeSet<String> = acc.value_import_names.iter().cloned().collect();
    let known_value_names: BTreeSet<String> = acc
        .value_decl_names
        .iter()
        .chain(acc.value_import_names.iter())
        .cloned()
        .collect();
    let value_ref_ctx = ValueRefContext {
        known: &known_value_names,
        local: &local_value_names,
        imported: &imported_value_names,
    };
    let known_keyframe_names: BTreeSet<String> = acc.keyframes_names.iter().cloned().collect();
    collect_index_refs_and_counts(&sheet.nodes, value_ref_ctx, &known_keyframe_names, &mut acc);
    let known_sass_function_names: BTreeSet<String> =
        acc.sass_function_decl_names.iter().cloned().collect();
    collect_sass_ref_facts(
        &sheet.nodes,
        &sheet.source,
        &known_sass_function_names,
        &mut acc,
    );
    let sass_variable_decl_facts = acc.sass_variable_decl_facts.clone();
    let sass_mixin_targets: BTreeSet<String> = acc.sass_mixin_decl_names.iter().cloned().collect();
    let sass_ref_ctx = SassRefContext {
        variable_decls: &sass_variable_decl_facts,
        mixin_targets: &sass_mixin_targets,
        function_targets: &known_sass_function_names,
    };
    let selector_attachment_ctx = SelectorAttachmentContext {
        source: &sheet.source,
        value_ref_ctx,
        known_keyframe_names: &known_keyframe_names,
        sass_ref_ctx,
    };
    collect_index_selector_attachment_facts(&sheet.nodes, selector_attachment_ctx, &mut acc, &[]);

    acc.selector_names.sort();
    acc.selector_definition_facts.sort();
    acc.selector_definition_facts.dedup();
    acc.bem_suffix_parent_names.sort();
    acc.bem_suffix_safe_selector_names.sort();
    acc.selectors_with_composes_names.sort();
    acc.selectors_with_composes_under_media_names.sort();
    acc.selectors_with_composes_under_supports_names.sort();
    acc.selectors_with_composes_under_layer_names.sort();
    acc.local_composes_selector_names.sort();
    acc.imported_composes_selector_names.sort();
    acc.global_composes_selector_names.sort();
    acc.local_composes_selector_names_under_media.sort();
    acc.local_composes_selector_names_under_supports.sort();
    acc.local_composes_selector_names_under_layer.sort();
    acc.imported_composes_selector_names_under_media.sort();
    acc.imported_composes_selector_names_under_supports.sort();
    acc.imported_composes_selector_names_under_layer.sort();
    acc.global_composes_selector_names_under_media.sort();
    acc.global_composes_selector_names_under_supports.sort();
    acc.global_composes_selector_names_under_layer.sort();
    acc.composes_import_sources.sort();
    acc.composes_import_sources_under_media.sort();
    acc.composes_import_sources_under_supports.sort();
    acc.composes_import_sources_under_layer.sort();
    acc.keyframes_names.sort();
    acc.keyframes_names.dedup();
    acc.nested_unsafe_selector_names.sort();
    acc.value_decl_names.sort();
    acc.value_decl_names.dedup();
    acc.value_decl_names_with_local_refs.sort();
    acc.value_decl_names_with_local_refs.dedup();
    acc.value_decl_names_with_imported_refs.sort();
    acc.value_decl_names_with_imported_refs.dedup();
    acc.value_import_names.sort();
    acc.value_import_names.dedup();
    acc.value_import_sources.sort();
    acc.value_ref_names.sort();
    acc.value_ref_names.dedup();
    acc.local_value_ref_names.sort();
    acc.local_value_ref_names.dedup();
    acc.imported_value_ref_names.sort();
    acc.imported_value_ref_names.dedup();
    acc.imported_value_ref_sources.sort();
    acc.declaration_value_ref_names.sort();
    acc.declaration_value_ref_names.dedup();
    acc.declaration_imported_value_ref_sources.sort();
    acc.value_decl_ref_names.sort();
    acc.value_decl_ref_names.dedup();
    acc.value_decl_imported_value_ref_sources.sort();
    acc.custom_property_decl_names.sort();
    acc.custom_property_decl_names.dedup();
    acc.custom_property_decl_facts.sort();
    acc.custom_property_decl_facts.dedup();
    acc.custom_property_decl_context_selectors.sort();
    acc.custom_property_decl_context_selectors.dedup();
    acc.custom_property_decl_names_under_media.sort();
    acc.custom_property_decl_names_under_media.dedup();
    acc.custom_property_decl_names_under_supports.sort();
    acc.custom_property_decl_names_under_supports.dedup();
    acc.custom_property_decl_names_under_layer.sort();
    acc.custom_property_decl_names_under_layer.dedup();
    acc.custom_property_ref_names.sort();
    acc.custom_property_ref_names.dedup();
    acc.custom_property_ref_facts.sort();
    acc.custom_property_ref_facts.dedup();
    acc.selectors_with_custom_property_refs_names.sort();
    acc.selectors_with_custom_property_refs_names.dedup();
    acc.selectors_with_custom_property_refs_under_media_names
        .sort();
    acc.selectors_with_custom_property_refs_under_media_names
        .dedup();
    acc.selectors_with_custom_property_refs_under_supports_names
        .sort();
    acc.selectors_with_custom_property_refs_under_supports_names
        .dedup();
    acc.selectors_with_custom_property_refs_under_layer_names
        .sort();
    acc.selectors_with_custom_property_refs_under_layer_names
        .dedup();
    acc.sass_variable_decl_names.sort();
    acc.sass_variable_decl_names.dedup();
    acc.sass_symbol_decl_facts.sort();
    acc.sass_symbol_decl_facts.dedup();
    acc.sass_variable_parameter_names.sort();
    acc.sass_variable_parameter_names.dedup();
    acc.sass_variable_ref_names.sort();
    acc.sass_variable_ref_names.dedup();
    acc.sass_selectors_with_variable_refs_names.sort();
    acc.sass_selectors_with_variable_refs_names.dedup();
    acc.sass_selectors_with_resolved_variable_refs_names.sort();
    acc.sass_selectors_with_resolved_variable_refs_names.dedup();
    acc.sass_selectors_with_unresolved_variable_refs_names
        .sort();
    acc.sass_selectors_with_unresolved_variable_refs_names
        .dedup();
    acc.sass_mixin_decl_names.sort();
    acc.sass_mixin_decl_names.dedup();
    acc.sass_mixin_include_names.sort();
    acc.sass_mixin_include_names.dedup();
    acc.sass_selectors_with_mixin_includes_names.sort();
    acc.sass_selectors_with_mixin_includes_names.dedup();
    acc.sass_selectors_with_resolved_mixin_includes_names.sort();
    acc.sass_selectors_with_resolved_mixin_includes_names
        .dedup();
    acc.sass_selectors_with_unresolved_mixin_includes_names
        .sort();
    acc.sass_selectors_with_unresolved_mixin_includes_names
        .dedup();
    acc.sass_function_decl_names.sort();
    acc.sass_function_decl_names.dedup();
    acc.sass_function_call_names.sort();
    acc.sass_function_call_names.dedup();
    acc.sass_selectors_with_function_calls_names.sort();
    acc.sass_selectors_with_function_calls_names.dedup();
    acc.sass_selector_symbol_facts.sort();
    acc.sass_selector_symbol_facts.dedup();
    acc.sass_module_use_sources.sort();
    acc.sass_module_use_sources.dedup();
    acc.sass_module_use_edges.sort();
    acc.sass_module_use_edges.dedup();
    acc.sass_module_forward_sources.sort();
    acc.sass_module_forward_sources.dedup();
    acc.sass_module_import_sources.sort();
    acc.sass_module_import_sources.dedup();
    acc.selectors_with_value_refs_names.sort();
    acc.selectors_with_local_value_refs_names.sort();
    acc.selectors_with_imported_value_refs_names.sort();
    acc.selectors_with_value_refs_under_media_names.sort();
    acc.selectors_with_value_refs_under_supports_names.sort();
    acc.selectors_with_value_refs_under_layer_names.sort();
    acc.selectors_with_local_value_refs_under_media_names.sort();
    acc.selectors_with_local_value_refs_under_supports_names
        .sort();
    acc.selectors_with_local_value_refs_under_layer_names.sort();
    acc.selectors_with_imported_value_refs_under_media_names
        .sort();
    acc.selectors_with_imported_value_refs_under_supports_names
        .sort();
    acc.selectors_with_imported_value_refs_under_layer_names
        .sort();
    acc.selectors_with_animation_ref_names.sort();
    acc.selectors_with_animation_refs_under_media_names.sort();
    acc.selectors_with_animation_refs_under_supports_names
        .sort();
    acc.selectors_with_animation_refs_under_layer_names.sort();
    acc.selectors_with_animation_name_ref_names.sort();
    acc.selectors_with_animation_name_refs_under_media_names
        .sort();
    acc.selectors_with_animation_name_refs_under_supports_names
        .sort();
    acc.selectors_with_animation_name_refs_under_layer_names
        .sort();
    acc.selectors_under_media_names.sort();
    acc.selectors_under_supports_names.sort();
    acc.selectors_under_layer_names.sort();
    acc.animation_ref_names.sort();
    acc.animation_ref_names.dedup();
    acc.animation_name_ref_names.sort();
    acc.animation_name_ref_names.dedup();
    acc.keyframes_names_under_media.sort();
    acc.keyframes_names_under_media.dedup();
    acc.keyframes_names_under_supports.sort();
    acc.keyframes_names_under_supports.dedup();
    acc.keyframes_names_under_layer.sort();
    acc.keyframes_names_under_layer.dedup();
    let selectors_with_value_refs_names = acc.selectors_with_value_refs_names.clone();
    let selectors_with_animation_ref_names = acc.selectors_with_animation_ref_names.clone();
    let selectors_with_animation_name_ref_names =
        acc.selectors_with_animation_name_ref_names.clone();
    let sass_same_file_resolution = summarize_sass_same_file_resolution(&acc);

    ParserIndexSummaryV0 {
        schema_version: "0",
        language: match sheet.language {
            StyleLanguage::Css => "css",
            StyleLanguage::Scss => "scss",
            StyleLanguage::Less => "less",
        },
        selectors: ParserIndexSelectorFactsV0 {
            names: acc.selector_names,
            definition_facts: acc.selector_definition_facts,
            bem_suffix_parent_names: acc.bem_suffix_parent_names,
            bem_suffix_safe_names: acc.bem_suffix_safe_selector_names,
            nested_unsafe_names: acc.nested_unsafe_selector_names,
            selectors_with_value_refs_names,
            selectors_with_animation_ref_names,
            selectors_with_animation_name_ref_names,
            bem_suffix_count: acc.bem_suffix_count,
            nested_safety_counts: acc.nested_safety_counts,
        },
        values: ParserIndexValueFactsV0 {
            decl_names: acc.value_decl_names,
            decl_names_with_local_refs: acc.value_decl_names_with_local_refs,
            decl_names_with_imported_refs: acc.value_decl_names_with_imported_refs,
            import_names: acc.value_import_names,
            import_sources: acc.value_import_sources,
            import_alias_count: acc.value_import_alias_count,
            ref_names: acc.value_ref_names,
            local_ref_names: acc.local_value_ref_names,
            imported_ref_names: acc.imported_value_ref_names,
            imported_ref_sources: acc.imported_value_ref_sources,
            declaration_ref_names: acc.declaration_value_ref_names,
            declaration_imported_ref_sources: acc.declaration_imported_value_ref_sources,
            value_decl_ref_names: acc.value_decl_ref_names,
            value_decl_imported_ref_sources: acc.value_decl_imported_value_ref_sources,
            selectors_with_refs_names: acc.selectors_with_value_refs_names,
            selectors_with_local_refs_names: acc.selectors_with_local_value_refs_names,
            selectors_with_imported_refs_names: acc.selectors_with_imported_value_refs_names,
            selectors_with_refs_under_media_names: acc.selectors_with_value_refs_under_media_names,
            selectors_with_refs_under_supports_names: acc
                .selectors_with_value_refs_under_supports_names,
            selectors_with_refs_under_layer_names: acc.selectors_with_value_refs_under_layer_names,
            selectors_with_local_refs_under_media_names: acc
                .selectors_with_local_value_refs_under_media_names,
            selectors_with_local_refs_under_supports_names: acc
                .selectors_with_local_value_refs_under_supports_names,
            selectors_with_local_refs_under_layer_names: acc
                .selectors_with_local_value_refs_under_layer_names,
            selectors_with_imported_refs_under_media_names: acc
                .selectors_with_imported_value_refs_under_media_names,
            selectors_with_imported_refs_under_supports_names: acc
                .selectors_with_imported_value_refs_under_supports_names,
            selectors_with_imported_refs_under_layer_names: acc
                .selectors_with_imported_value_refs_under_layer_names,
        },
        custom_properties: ParserIndexCustomPropertyFactsV0 {
            decl_names: acc.custom_property_decl_names,
            decl_facts: acc.custom_property_decl_facts,
            decl_context_selectors: acc.custom_property_decl_context_selectors,
            decl_names_under_media: acc.custom_property_decl_names_under_media,
            decl_names_under_supports: acc.custom_property_decl_names_under_supports,
            decl_names_under_layer: acc.custom_property_decl_names_under_layer,
            ref_names: acc.custom_property_ref_names,
            ref_facts: acc.custom_property_ref_facts,
            selectors_with_refs_names: acc.selectors_with_custom_property_refs_names,
            selectors_with_refs_under_media_names: acc
                .selectors_with_custom_property_refs_under_media_names,
            selectors_with_refs_under_supports_names: acc
                .selectors_with_custom_property_refs_under_supports_names,
            selectors_with_refs_under_layer_names: acc
                .selectors_with_custom_property_refs_under_layer_names,
        },
        sass: ParserIndexSassFactsV0 {
            variable_decl_names: acc.sass_variable_decl_names,
            symbol_decl_facts: acc.sass_symbol_decl_facts,
            variable_parameter_names: acc.sass_variable_parameter_names,
            variable_ref_names: acc.sass_variable_ref_names,
            selectors_with_variable_refs_names: acc.sass_selectors_with_variable_refs_names,
            selectors_with_resolved_variable_refs_names: acc
                .sass_selectors_with_resolved_variable_refs_names,
            selectors_with_unresolved_variable_refs_names: acc
                .sass_selectors_with_unresolved_variable_refs_names,
            mixin_decl_names: acc.sass_mixin_decl_names,
            mixin_include_names: acc.sass_mixin_include_names,
            selectors_with_mixin_includes_names: acc.sass_selectors_with_mixin_includes_names,
            selectors_with_resolved_mixin_includes_names: acc
                .sass_selectors_with_resolved_mixin_includes_names,
            selectors_with_unresolved_mixin_includes_names: acc
                .sass_selectors_with_unresolved_mixin_includes_names,
            function_decl_names: acc.sass_function_decl_names,
            function_call_names: acc.sass_function_call_names,
            selectors_with_function_calls_names: acc.sass_selectors_with_function_calls_names,
            selector_symbol_facts: acc.sass_selector_symbol_facts,
            module_use_sources: acc.sass_module_use_sources,
            module_use_edges: acc.sass_module_use_edges,
            module_forward_sources: acc.sass_module_forward_sources,
            module_import_sources: acc.sass_module_import_sources,
            same_file_resolution: sass_same_file_resolution,
        },
        keyframes: ParserIndexKeyframesFactsV0 {
            names: acc.keyframes_names,
            names_under_media: acc.keyframes_names_under_media,
            names_under_supports: acc.keyframes_names_under_supports,
            names_under_layer: acc.keyframes_names_under_layer,
            animation_ref_names: acc.animation_ref_names,
            animation_name_ref_names: acc.animation_name_ref_names,
            selectors_with_animation_ref_names: acc.selectors_with_animation_ref_names,
            selectors_with_animation_name_ref_names: acc.selectors_with_animation_name_ref_names,
            selectors_with_animation_refs_under_media_names: acc
                .selectors_with_animation_refs_under_media_names,
            selectors_with_animation_refs_under_supports_names: acc
                .selectors_with_animation_refs_under_supports_names,
            selectors_with_animation_refs_under_layer_names: acc
                .selectors_with_animation_refs_under_layer_names,
            selectors_with_animation_name_refs_under_media_names: acc
                .selectors_with_animation_name_refs_under_media_names,
            selectors_with_animation_name_refs_under_supports_names: acc
                .selectors_with_animation_name_refs_under_supports_names,
            selectors_with_animation_name_refs_under_layer_names: acc
                .selectors_with_animation_name_refs_under_layer_names,
        },
        composes: ParserIndexComposesFactsV0 {
            selectors_with_composes_names: acc.selectors_with_composes_names,
            selectors_with_composes_under_media_names: acc
                .selectors_with_composes_under_media_names,
            selectors_with_composes_under_supports_names: acc
                .selectors_with_composes_under_supports_names,
            selectors_with_composes_under_layer_names: acc
                .selectors_with_composes_under_layer_names,
            local_selector_names: acc.local_composes_selector_names,
            imported_selector_names: acc.imported_composes_selector_names,
            global_selector_names: acc.global_composes_selector_names,
            local_selector_names_under_media: acc.local_composes_selector_names_under_media,
            local_selector_names_under_supports: acc.local_composes_selector_names_under_supports,
            local_selector_names_under_layer: acc.local_composes_selector_names_under_layer,
            imported_selector_names_under_media: acc.imported_composes_selector_names_under_media,
            imported_selector_names_under_supports: acc
                .imported_composes_selector_names_under_supports,
            imported_selector_names_under_layer: acc.imported_composes_selector_names_under_layer,
            global_selector_names_under_media: acc.global_composes_selector_names_under_media,
            global_selector_names_under_supports: acc.global_composes_selector_names_under_supports,
            global_selector_names_under_layer: acc.global_composes_selector_names_under_layer,
            import_sources: acc.composes_import_sources,
            import_sources_under_media: acc.composes_import_sources_under_media,
            import_sources_under_supports: acc.composes_import_sources_under_supports,
            import_sources_under_layer: acc.composes_import_sources_under_layer,
            class_name_count: acc.composes_class_name_count,
            local_class_name_count: acc.local_composes_class_name_count,
            imported_class_name_count: acc.imported_composes_class_name_count,
            global_class_name_count: acc.global_composes_class_name_count,
        },
        wrappers: ParserIndexWrapperFactsV0 {
            selectors_under_media_names: acc.selectors_under_media_names,
            selectors_under_supports_names: acc.selectors_under_supports_names,
            selectors_under_layer_names: acc.selectors_under_layer_names,
        },
    }
}

pub fn summarize_parser_canonical_candidate(
    sheet: &Stylesheet,
) -> ParserCanonicalCandidateBundleV0 {
    let parity_lite = summarize_parity_lite(sheet);
    let css_modules_intermediate = summarize_css_modules_intermediate(sheet);

    ParserCanonicalCandidateBundleV0 {
        schema_version: "0",
        language: parity_lite.language,
        parity_lite,
        css_modules_intermediate,
    }
}

pub fn summarize_parser_evaluator_candidates(sheet: &Stylesheet) -> ParserEvaluatorCandidatesV0 {
    let intermediate = summarize_css_modules_intermediate(sheet);
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
            let bem_suffix_parent_name = if nested_safety_kind == "bemSuffixSafe" {
                let suffix_split_index = [selector.rfind("__"), selector.rfind("--")]
                    .into_iter()
                    .flatten()
                    .max();
                suffix_split_index.map(|index| selector[..index].to_string())
            } else {
                None
            };

            ParserEvaluatorCandidateV0 {
                kind: "selector-index-facts",
                selector_name: selector_name.clone(),
                nested_safety_kind,
                bem_suffix_parent_name,
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
    sheet: &Stylesheet,
) -> ParserCanonicalProducerSignalV0 {
    let canonical_candidate = summarize_parser_canonical_candidate(sheet);
    let evaluator_candidates = summarize_parser_evaluator_candidates(sheet);

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

pub fn summarize_index_bridge(sheet: &Stylesheet) -> ParserIndexSummaryV0 {
    summarize_css_modules_intermediate(sheet)
}

pub fn summarize_semantic_boundary(sheet: &Stylesheet) -> ParserSemanticBoundarySummaryV0 {
    let index = summarize_css_modules_intermediate(sheet);
    let ParserIndexSummaryV0 {
        schema_version: _,
        language,
        selectors,
        values,
        custom_properties,
        sass,
        keyframes,
        composes,
        wrappers,
    } = index;
    let ParserIndexSelectorFactsV0 {
        names,
        definition_facts,
        bem_suffix_parent_names,
        bem_suffix_safe_names,
        nested_unsafe_names,
        selectors_with_value_refs_names,
        selectors_with_animation_ref_names,
        selectors_with_animation_name_ref_names,
        bem_suffix_count,
        nested_safety_counts,
    } = selectors;
    let ParserIndexSassFactsV0 {
        variable_decl_names,
        symbol_decl_facts: _,
        variable_parameter_names,
        variable_ref_names,
        selectors_with_variable_refs_names: _,
        selectors_with_resolved_variable_refs_names,
        selectors_with_unresolved_variable_refs_names,
        mixin_decl_names,
        mixin_include_names,
        selectors_with_mixin_includes_names: _,
        selectors_with_resolved_mixin_includes_names,
        selectors_with_unresolved_mixin_includes_names,
        function_decl_names,
        function_call_names,
        selectors_with_function_calls_names,
        selector_symbol_facts,
        module_use_sources,
        module_use_edges,
        module_forward_sources,
        module_import_sources,
        same_file_resolution,
    } = sass;
    let custom_property_semantic_facts =
        summarize_custom_property_semantic_facts(&custom_properties);

    ParserSemanticBoundarySummaryV0 {
        schema_version: "0",
        language,
        parser_facts: ParserBoundarySyntaxFactsV0 {
            lossless_cst: summarize_lossless_cst(sheet),
            selectors: ParserIndexSelectorFactsV0 {
                names: names.clone(),
                definition_facts,
                bem_suffix_parent_names: bem_suffix_parent_names.clone(),
                bem_suffix_safe_names: bem_suffix_safe_names.clone(),
                nested_unsafe_names: nested_unsafe_names.clone(),
                selectors_with_value_refs_names,
                selectors_with_animation_ref_names,
                selectors_with_animation_name_ref_names,
                bem_suffix_count,
                nested_safety_counts: nested_safety_counts.clone(),
            },
            values,
            custom_properties,
            sass: ParserSassSyntaxFactsV0 {
                variable_decl_names,
                variable_parameter_names,
                variable_ref_names,
                mixin_decl_names,
                mixin_include_names,
                function_decl_names,
                function_call_names,
                module_use_sources,
                module_use_edges,
                module_forward_sources,
                module_import_sources,
            },
            keyframes,
            composes,
            wrappers,
        },
        semantic_facts: StyleSemanticFactsV0 {
            selector_identity: StyleSelectorIdentityFactsV0 {
                canonical_names: names,
                bem_suffix_safe_names,
                bem_suffix_parent_names,
                nested_unsafe_names,
                nested_safety_counts,
            },
            custom_properties: custom_property_semantic_facts,
            sass: StyleSassSemanticFactsV0 {
                selector_symbol_facts,
                selectors_with_resolved_variable_refs_names,
                selectors_with_unresolved_variable_refs_names,
                selectors_with_resolved_mixin_includes_names,
                selectors_with_unresolved_mixin_includes_names,
                selectors_with_function_calls_names,
                same_file_resolution,
            },
        },
    }
}

fn summarize_custom_property_semantic_facts(
    facts: &ParserIndexCustomPropertyFactsV0,
) -> StyleCustomPropertySemanticFactsV0 {
    let (resolved_ref_names, unresolved_ref_names) =
        summarize_custom_property_resolution_names(facts);

    StyleCustomPropertySemanticFactsV0 {
        decl_names: facts.decl_names.clone(),
        ref_names: facts.ref_names.clone(),
        resolved_ref_names,
        unresolved_ref_names,
        selectors_with_refs_names: facts.selectors_with_refs_names.clone(),
    }
}

fn summarize_custom_property_resolution_names(
    facts: &ParserIndexCustomPropertyFactsV0,
) -> (Vec<String>, Vec<String>) {
    if !facts.decl_facts.is_empty() && !facts.ref_facts.is_empty() {
        let mut resolved = BTreeSet::new();
        let mut unresolved = BTreeSet::new();
        for ref_fact in &facts.ref_facts {
            if facts
                .decl_facts
                .iter()
                .any(|decl_fact| custom_property_context_matches(decl_fact, ref_fact))
            {
                resolved.insert(ref_fact.name.clone());
            } else {
                unresolved.insert(ref_fact.name.clone());
            }
        }
        return (
            resolved.into_iter().collect(),
            unresolved.into_iter().collect(),
        );
    }

    let decl_names: BTreeSet<&str> = facts.decl_names.iter().map(String::as_str).collect();
    let resolved_ref_names = facts
        .ref_names
        .iter()
        .filter(|name| decl_names.contains(name.as_str()))
        .cloned()
        .collect();
    let unresolved_ref_names = facts
        .ref_names
        .iter()
        .filter(|name| !decl_names.contains(name.as_str()))
        .cloned()
        .collect();
    (resolved_ref_names, unresolved_ref_names)
}

fn custom_property_context_matches(
    decl: &ParserIndexCustomPropertyDeclFactV0,
    reference: &ParserIndexCustomPropertyRefFactV0,
) -> bool {
    if decl.name != reference.name {
        return false;
    }
    if decl.under_media && !reference.under_media {
        return false;
    }
    if decl.under_supports && !reference.under_supports {
        return false;
    }
    if decl.under_layer && !reference.under_layer {
        return false;
    }
    if decl.selector_contexts.is_empty() {
        return true;
    }
    decl.selector_contexts
        .iter()
        .any(|decl_selector| custom_property_selector_context_matches(decl_selector, reference))
}

fn custom_property_selector_context_matches(
    decl_selector: &str,
    reference: &ParserIndexCustomPropertyRefFactV0,
) -> bool {
    decl_selector == ":root"
        || reference.selector_contexts.iter().any(|ref_selector| {
            ref_selector == decl_selector || ref_selector.contains(decl_selector)
        })
}

fn summarize_lossless_cst(sheet: &Stylesheet) -> ParserLosslessCstFactsV0 {
    let source_byte_len = sheet.source.len();
    ParserLosslessCstFactsV0 {
        source_byte_len,
        token_count: sheet.tokens.len(),
        root_node_count: sheet.nodes.len(),
        diagnostic_count: sheet.diagnostics.len(),
        all_token_spans_within_source: sheet
            .tokens
            .iter()
            .all(|token| is_valid_span(token.span, source_byte_len)),
        all_node_spans_within_source: nodes_have_valid_spans(&sheet.nodes, source_byte_len),
    }
}

fn nodes_have_valid_spans(nodes: &[SyntaxNode], source_byte_len: usize) -> bool {
    nodes.iter().all(|node| {
        is_valid_span(node.span, source_byte_len)
            && node
                .header_span
                .is_none_or(|span| is_valid_span(span, source_byte_len))
            && nodes_have_valid_spans(&node.children, source_byte_len)
    })
}

fn is_valid_span(span: TextSpan, source_byte_len: usize) -> bool {
    span.start <= span.end && span.end <= source_byte_len
}

fn collect_parity_names(nodes: &[SyntaxNode], acc: &mut ParityLiteAcc) {
    collect_parity_names_with_parent(nodes, acc, &[], 0);
}

fn collect_parity_names_with_parent(
    nodes: &[SyntaxNode],
    acc: &mut ParityLiteAcc,
    parent_branches: &[ResolvedSelectorBranch],
    depth: usize,
) {
    for node in nodes {
        let mut next_parent_branches = parent_branches.to_vec();
        let mut next_depth = depth;
        match &node.payload {
            Some(SyntaxNodePayload::Rule(rule)) => {
                acc.rule_count += 1;
                next_depth = depth + 1;
                acc.max_nesting_depth = acc.max_nesting_depth.max(next_depth);
                if rule.selector_groups.len() > 1 {
                    acc.grouped_selector_count += rule.selector_groups.len();
                }
                let resolved = resolve_rule_selector_branches(rule, parent_branches);
                if !resolved.is_empty() {
                    acc.selector_names
                        .extend(resolved.iter().map(|branch| branch.name.clone()));
                    next_parent_branches = resolved;
                }
            }
            Some(SyntaxNodePayload::AtRule(at_rule)) => {
                next_depth = depth + 1;
                acc.max_nesting_depth = acc.max_nesting_depth.max(next_depth);
                increment_at_rule_kind_count(&mut acc.at_rule_kind_counts, at_rule.kind);
                match at_rule.kind {
                    AtRuleKind::Keyframes if !at_rule.params.is_empty() => {
                        acc.keyframes_names.push(at_rule.params.clone());
                    }
                    AtRuleKind::Keyframes => {}
                    AtRuleKind::Value => {
                        if let Some((name, _)) = at_rule.params.split_once(':') {
                            let trimmed = name.trim();
                            if !trimmed.is_empty() {
                                acc.value_decl_names.push(trimmed.to_string());
                            }
                        }
                    }
                    _ => {}
                }
            }
            Some(SyntaxNodePayload::Declaration(declaration)) => {
                acc.declaration_count += 1;
                increment_declaration_kind_count(
                    &mut acc.declaration_kind_counts,
                    classify_declaration_kind(&declaration.property),
                );
            }
            _ => {}
        }
        collect_parity_names_with_parent(&node.children, acc, &next_parent_branches, next_depth);
    }
}

fn increment_at_rule_kind_count(counts: &mut AtRuleKindCountsV0, kind: AtRuleKind) {
    match kind {
        AtRuleKind::Media => counts.media += 1,
        AtRuleKind::Supports => counts.supports += 1,
        AtRuleKind::Layer => counts.layer += 1,
        AtRuleKind::Keyframes => counts.keyframes += 1,
        AtRuleKind::Value => counts.value += 1,
        AtRuleKind::AtRoot => counts.at_root += 1,
        AtRuleKind::Mixin
        | AtRuleKind::Include
        | AtRuleKind::Function
        | AtRuleKind::Use
        | AtRuleKind::Forward
        | AtRuleKind::Import => counts.generic += 1,
        AtRuleKind::Generic => counts.generic += 1,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DeclarationKind {
    Composes,
    Animation,
    AnimationName,
    Generic,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NestedSafetyKind {
    Flat,
    BemSuffixSafe,
    NestedUnsafe,
}

struct RuleSelectorFacts {
    nested_safety: NestedSafetyKind,
    bem_suffix_count: usize,
}

#[derive(Debug, Default)]
struct RuleComposesFacts {
    local_class_name_count: usize,
    imported_class_name_count: usize,
    global_class_name_count: usize,
    imported_sources: Vec<String>,
}

#[derive(Debug, Default)]
struct RuleReferenceFacts {
    has_value_refs: bool,
    has_local_value_refs: bool,
    has_imported_value_refs: bool,
    has_animation_refs: bool,
    has_animation_name_refs: bool,
    has_custom_property_refs: bool,
    custom_property_ref_names: Vec<String>,
    has_sass_variable_refs: bool,
    has_resolved_sass_variable_refs: bool,
    has_unresolved_sass_variable_refs: bool,
    has_sass_mixin_includes: bool,
    has_resolved_sass_mixin_includes: bool,
    has_unresolved_sass_mixin_includes: bool,
    has_sass_function_calls: bool,
    sass_symbol_facts: Vec<RuleSassSymbolFact>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct RuleSassSymbolFact {
    symbol_kind: &'static str,
    name: String,
    namespace: Option<String>,
    role: &'static str,
    resolution: &'static str,
    byte_span: ParserByteSpanV0,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SassNameSpan {
    name: String,
    namespace: Option<String>,
    byte_span: ParserByteSpanV0,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CustomPropertyDeclNameSpan {
    name: String,
    byte_span: ParserByteSpanV0,
    range: ParserRangeV0,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SassVariableDeclFact {
    name: String,
    scope: SassVariableScope,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SassVariableRefFact {
    name: String,
    namespace: Option<String>,
    byte_span: ParserByteSpanV0,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SassVariableScope {
    File,
    Span(TextSpan),
}

#[derive(Debug, Clone, Copy, Default)]
struct WrapperContext {
    under_media: bool,
    under_supports: bool,
    under_layer: bool,
}

#[derive(Debug, Clone, Copy)]
struct ValueRefContext<'a> {
    known: &'a BTreeSet<String>,
    local: &'a BTreeSet<String>,
    imported: &'a BTreeSet<String>,
}

#[derive(Debug, Clone, Copy)]
struct SassRefContext<'a> {
    variable_decls: &'a [SassVariableDeclFact],
    mixin_targets: &'a BTreeSet<String>,
    function_targets: &'a BTreeSet<String>,
}

#[derive(Debug, Clone, Copy)]
struct SelectorAttachmentContext<'a> {
    source: &'a str,
    value_ref_ctx: ValueRefContext<'a>,
    known_keyframe_names: &'a BTreeSet<String>,
    sass_ref_ctx: SassRefContext<'a>,
}

#[derive(Debug, Clone, Copy)]
enum ValueRefOrigin {
    Declaration,
    ValueDecl,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ComposesKind {
    Local,
    Imported,
    Global,
}

#[derive(Debug)]
struct ComposesSpec {
    class_names: Vec<String>,
    kind: ComposesKind,
    from_source: Option<String>,
}

fn classify_declaration_kind(property: &str) -> DeclarationKind {
    match property.trim().to_ascii_lowercase().as_str() {
        "composes" => DeclarationKind::Composes,
        "animation" => DeclarationKind::Animation,
        "animation-name" => DeclarationKind::AnimationName,
        _ => DeclarationKind::Generic,
    }
}

fn increment_declaration_kind_count(counts: &mut DeclarationKindCountsV0, kind: DeclarationKind) {
    match kind {
        DeclarationKind::Composes => counts.composes += 1,
        DeclarationKind::Animation => counts.animation += 1,
        DeclarationKind::AnimationName => counts.animation_name += 1,
        DeclarationKind::Generic => counts.generic += 1,
    }
}

fn classify_rule_selector_facts(
    rule: &RulePayload,
    parent_branches: &[ResolvedSelectorBranch],
    parent_is_grouped: bool,
) -> RuleSelectorFacts {
    let is_nested = !parent_branches.is_empty()
        || rule
            .selector_groups
            .iter()
            .any(|group| group.raw.contains('&'));
    if !is_nested {
        return RuleSelectorFacts {
            nested_safety: NestedSafetyKind::Flat,
            bem_suffix_count: 0,
        };
    }

    let bem_suffix_safe = rule.selector_groups.len() == 1
        && parent_branches.len() == 1
        && parent_branches[0].bare_suffix_base
        && !parent_is_grouped
        && matches!(
            rule.selector_groups[0].segments.as_slice(),
            [SelectorSegment::Ampersand, SelectorSegment::BemSuffix(_)]
        );

    if bem_suffix_safe {
        RuleSelectorFacts {
            nested_safety: NestedSafetyKind::BemSuffixSafe,
            bem_suffix_count: 1,
        }
    } else {
        RuleSelectorFacts {
            nested_safety: NestedSafetyKind::NestedUnsafe,
            bem_suffix_count: 0,
        }
    }
}

fn increment_nested_safety_count(
    counts: &mut NestedSafetyCountsV0,
    kind: NestedSafetyKind,
    amount: usize,
) {
    match kind {
        NestedSafetyKind::Flat => counts.flat += amount,
        NestedSafetyKind::BemSuffixSafe => counts.bem_suffix_safe += amount,
        NestedSafetyKind::NestedUnsafe => counts.nested_unsafe += amount,
    }
}

fn nested_safety_kind_label(kind: NestedSafetyKind) -> &'static str {
    match kind {
        NestedSafetyKind::Flat => "flat",
        NestedSafetyKind::BemSuffixSafe => "bemSuffixSafe",
        NestedSafetyKind::NestedUnsafe => "nestedUnsafe",
    }
}

fn rule_selector_definition_spans(
    source: &str,
    header_span: TextSpan,
    rule: &RulePayload,
    resolved_names: &[String],
    parent_branches: &[ResolvedSelectorBranch],
) -> Vec<(String, ParserByteSpanV0)> {
    let resolved_names: BTreeSet<&str> = resolved_names.iter().map(String::as_str).collect();
    let mut seen = BTreeSet::new();
    let mut spans = Vec::new();

    for group in &rule.selector_groups {
        let include_all_class_segments = selector_group_allows_all_class_definition_facts(group);
        for segment in &group.segments {
            let SelectorSegment::ClassName(name) = segment else {
                continue;
            };
            if !include_all_class_segments && !resolved_names.contains(name.as_str()) {
                continue;
            }
            for byte_span in class_name_byte_spans_in_header(source, header_span, name) {
                if seen.insert((byte_span.start, byte_span.end, name.clone())) {
                    spans.push((name.clone(), byte_span));
                }
            }
        }

        for (name, byte_span) in
            bem_suffix_selector_definition_spans(source, header_span, group, parent_branches)
        {
            if !resolved_names.contains(name.as_str()) {
                continue;
            }
            if seen.insert((byte_span.start, byte_span.end, name.clone())) {
                spans.push((name, byte_span));
            }
        }
    }

    spans
}

fn selector_group_allows_all_class_definition_facts(group: &SelectorGroup) -> bool {
    ![":global(", ":not(", ":is(", ":where(", ":has("]
        .iter()
        .any(|marker| group.raw.contains(marker))
}

fn bem_suffix_selector_definition_spans(
    source: &str,
    header_span: TextSpan,
    group: &SelectorGroup,
    parent_branches: &[ResolvedSelectorBranch],
) -> Vec<(String, ParserByteSpanV0)> {
    let suffix = match group.segments.as_slice() {
        [
            SelectorSegment::Ampersand,
            SelectorSegment::BemSuffix(suffix),
        ]
        | [
            SelectorSegment::Ampersand,
            SelectorSegment::AmpersandSuffix(suffix),
        ] => suffix,
        segments => {
            let last_combinator = segments
                .iter()
                .enumerate()
                .rev()
                .find_map(|(index, segment)| {
                    matches!(segment, SelectorSegment::Combinator(_)).then_some(index)
                });
            let tail = last_combinator
                .map(|index| &segments[index + 1..])
                .unwrap_or(segments);
            match tail {
                [
                    SelectorSegment::Ampersand,
                    SelectorSegment::BemSuffix(suffix),
                ]
                | [
                    SelectorSegment::Ampersand,
                    SelectorSegment::AmpersandSuffix(suffix),
                ] => suffix,
                _ => return Vec::new(),
            }
        }
    };
    let Some(suffix_span) = ampersand_suffix_byte_span_in_header(source, header_span, suffix)
    else {
        return Vec::new();
    };

    parent_branches
        .iter()
        .filter(|parent| parent.bare_suffix_base)
        .map(|parent| (format!("{}{}", parent.name, suffix), suffix_span))
        .collect()
}

fn ampersand_suffix_byte_span_in_header(
    source: &str,
    header_span: TextSpan,
    suffix: &str,
) -> Option<ParserByteSpanV0> {
    let header = &source[header_span.start..header_span.end];
    let needle = format!("&{suffix}");
    let relative_match = header.find(&needle)?;
    let suffix_start = header_span.start + relative_match + 1;
    let suffix_end = suffix_start + suffix.len();
    Some(ParserByteSpanV0 {
        start: suffix_start,
        end: suffix_end,
    })
}

fn class_name_byte_spans_in_header(
    source: &str,
    header_span: TextSpan,
    name: &str,
) -> Vec<ParserByteSpanV0> {
    let header = &source[header_span.start..header_span.end];
    let needle = format!(".{name}");
    let mut spans = Vec::new();
    let mut search_offset = 0usize;

    while let Some(relative_match) = header[search_offset..].find(&needle) {
        let dot_start = header_span.start + search_offset + relative_match;
        let name_start = dot_start + 1;
        let name_end = name_start + name.len();
        if is_selector_name_boundary(source, name_end) {
            spans.push(ParserByteSpanV0 {
                start: name_start,
                end: name_end,
            });
        }
        search_offset += relative_match + needle.len();
    }

    spans
}

fn is_selector_name_boundary(source: &str, byte_offset: usize) -> bool {
    source[byte_offset..]
        .chars()
        .next()
        .is_none_or(|ch| !is_selector_ident_continue(ch))
}

fn collect_rule_composes_facts(children: &[SyntaxNode]) -> RuleComposesFacts {
    let mut facts = RuleComposesFacts::default();
    for child in children {
        if let Some(SyntaxNodePayload::Declaration(declaration)) = &child.payload
            && classify_declaration_kind(&declaration.property) == DeclarationKind::Composes
            && let Some(spec) = parse_composes_spec(&declaration.value)
        {
            match spec.kind {
                ComposesKind::Local => facts.local_class_name_count += spec.class_names.len(),
                ComposesKind::Imported => {
                    facts.imported_class_name_count += spec.class_names.len();
                    if let Some(source) = spec.from_source {
                        facts.imported_sources.push(source);
                    }
                }
                ComposesKind::Global => facts.global_class_name_count += spec.class_names.len(),
            }
        }
    }
    facts
}

fn declaration_value_span(source: &str, node: &SyntaxNode) -> TextSpan {
    let header_span = node.header_span.unwrap_or(node.span);
    let raw = &source[header_span.start..header_span.end];
    let Some(colon_index) = raw.find(':') else {
        return TextSpan::new(header_span.end, header_span.end);
    };
    trim_source_span(
        source,
        TextSpan::new(header_span.start + colon_index + 1, header_span.end),
    )
}

fn at_rule_params_span(source: &str, node: &SyntaxNode, at_rule: &AtRulePayload) -> TextSpan {
    let header_span = node.header_span.unwrap_or(node.span);
    let raw = &source[header_span.start..header_span.end];
    let search_start = raw.find('@').map_or(0, |index| index + 1);
    let Some(name_index) = raw[search_start..].find(&at_rule.name) else {
        return TextSpan::new(header_span.end, header_span.end);
    };
    let params_start = header_span.start + search_start + name_index + at_rule.name.len();
    trim_source_span(source, TextSpan::new(params_start, header_span.end))
}

fn trim_source_span(source: &str, span: TextSpan) -> TextSpan {
    let raw = &source[span.start..span.end];
    let trimmed_start = raw.len() - raw.trim_start().len();
    let trimmed_len = raw.trim().len();
    let start = span.start + trimmed_start;
    TextSpan::new(start, start + trimmed_len)
}

fn source_range_for_byte_span(source: &str, span: ParserByteSpanV0) -> ParserRangeV0 {
    ParserRangeV0 {
        start: source_position_for_byte_offset(source, span.start),
        end: source_position_for_byte_offset(source, span.end),
    }
}

fn source_position_for_byte_offset(source: &str, offset: usize) -> ParserPositionV0 {
    let clamped_offset = offset.min(source.len());
    let mut line = 0usize;
    let mut character = 0usize;

    for (byte_index, ch) in source.char_indices() {
        if byte_index >= clamped_offset {
            break;
        }
        if ch == '\n' {
            line += 1;
            character = 0;
        } else {
            character += ch.len_utf16();
        }
    }

    ParserPositionV0 { line, character }
}

fn collect_rule_reference_facts(
    children: &[SyntaxNode],
    source: &str,
    value_ref_ctx: ValueRefContext<'_>,
    known_keyframe_names: &BTreeSet<String>,
    sass_ref_ctx: SassRefContext<'_>,
) -> RuleReferenceFacts {
    let mut facts = RuleReferenceFacts::default();
    for child in children {
        match &child.payload {
            Some(SyntaxNodePayload::Declaration(declaration)) => {
                let value_span = declaration_value_span(source, child);
                match classify_declaration_kind(&declaration.property) {
                    DeclarationKind::Composes => {}
                    DeclarationKind::Animation => {
                        if !find_identifier_matches(&declaration.value, known_keyframe_names)
                            .is_empty()
                        {
                            facts.has_animation_refs = true;
                        }
                        extend_rule_value_ref_facts(&mut facts, &declaration.value, value_ref_ctx);
                    }
                    DeclarationKind::AnimationName => {
                        if !find_identifier_matches(&declaration.value, known_keyframe_names)
                            .is_empty()
                        {
                            facts.has_animation_name_refs = true;
                        }
                        extend_rule_value_ref_facts(&mut facts, &declaration.value, value_ref_ctx);
                    }
                    DeclarationKind::Generic => {
                        extend_rule_value_ref_facts(&mut facts, &declaration.value, value_ref_ctx);
                    }
                }
                extend_rule_sass_value_ref_facts(
                    &mut facts,
                    &declaration.value,
                    value_span,
                    sass_ref_ctx,
                );
                let custom_property_refs = find_css_var_ref_names(&declaration.value);
                if !custom_property_refs.is_empty() {
                    facts.has_custom_property_refs = true;
                    facts.custom_property_ref_names.extend(custom_property_refs);
                }
            }
            Some(SyntaxNodePayload::AtRule(at_rule)) => match at_rule.kind {
                AtRuleKind::Mixin | AtRuleKind::Function => {}
                AtRuleKind::Include => {
                    let params_span = at_rule_params_span(source, child, at_rule);
                    extend_rule_sass_value_ref_facts(
                        &mut facts,
                        &at_rule.params,
                        params_span,
                        sass_ref_ctx,
                    );
                    if let Some(name_span) =
                        parse_sass_callable_name_with_span(&at_rule.params, params_span.start)
                    {
                        facts.has_sass_mixin_includes = true;
                        let resolution = if name_span.namespace.is_some() {
                            "external"
                        } else if sass_ref_ctx.mixin_targets.contains(&name_span.name) {
                            facts.has_resolved_sass_mixin_includes = true;
                            "resolved"
                        } else {
                            facts.has_unresolved_sass_mixin_includes = true;
                            "unresolved"
                        };
                        facts.sass_symbol_facts.push(RuleSassSymbolFact {
                            symbol_kind: "mixin",
                            name: name_span.name,
                            namespace: name_span.namespace,
                            role: "include",
                            resolution,
                            byte_span: name_span.byte_span,
                        });
                    }
                }
                _ => {
                    let params_span = at_rule_params_span(source, child, at_rule);
                    extend_rule_sass_value_ref_facts(
                        &mut facts,
                        &at_rule.params,
                        params_span,
                        sass_ref_ctx,
                    );
                }
            },
            _ => {}
        }
    }
    facts
}

fn extend_rule_value_ref_facts(
    facts: &mut RuleReferenceFacts,
    value: &str,
    value_ref_ctx: ValueRefContext<'_>,
) {
    let value_refs = find_identifier_matches(value, value_ref_ctx.known);
    if !value_refs.is_empty() {
        facts.has_value_refs = true;
        facts.has_local_value_refs |= value_refs
            .iter()
            .any(|name| value_ref_ctx.local.contains(name));
        facts.has_imported_value_refs |= value_refs
            .iter()
            .any(|name| value_ref_ctx.imported.contains(name));
    }
}

fn extend_rule_sass_value_ref_facts(
    facts: &mut RuleReferenceFacts,
    value: &str,
    value_span: TextSpan,
    sass_ref_ctx: SassRefContext<'_>,
) {
    let variable_refs = find_sass_variable_ref_spans(value, value_span.start);
    if !variable_refs.is_empty() {
        facts.has_sass_variable_refs = true;
        for name_span in variable_refs {
            let resolution = if name_span.namespace.is_some() {
                "external"
            } else if resolve_sass_variable_ref(
                &name_span.name,
                name_span.byte_span,
                sass_ref_ctx.variable_decls,
            ) {
                facts.has_resolved_sass_variable_refs = true;
                "resolved"
            } else {
                facts.has_unresolved_sass_variable_refs = true;
                "unresolved"
            };
            facts.sass_symbol_facts.push(RuleSassSymbolFact {
                symbol_kind: "variable",
                name: name_span.name,
                namespace: name_span.namespace,
                role: "reference",
                resolution,
                byte_span: name_span.byte_span,
            });
        }
    }

    let function_calls =
        find_sass_function_call_spans(value, value_span.start, sass_ref_ctx.function_targets);
    if !function_calls.is_empty() {
        facts.has_sass_function_calls = true;
        facts
            .sass_symbol_facts
            .extend(
                function_calls
                    .into_iter()
                    .map(|name_span| RuleSassSymbolFact {
                        symbol_kind: "function",
                        name: name_span.name,
                        namespace: name_span.namespace.clone(),
                        role: "call",
                        resolution: if name_span.namespace.is_some() {
                            "external"
                        } else {
                            "resolved"
                        },
                        byte_span: name_span.byte_span,
                    }),
            );
    }
}

fn resolve_sass_variable_ref(
    name: &str,
    byte_span: ParserByteSpanV0,
    variable_decls: &[SassVariableDeclFact],
) -> bool {
    variable_decls
        .iter()
        .any(|decl| decl.name == name && sass_variable_scope_contains(decl.scope, byte_span))
}

fn sass_variable_scope_contains(scope: SassVariableScope, byte_span: ParserByteSpanV0) -> bool {
    match scope {
        SassVariableScope::File => true,
        SassVariableScope::Span(scope_span) => {
            scope_span.start <= byte_span.start && scope_span.end >= byte_span.end
        }
    }
}

fn collect_index_names(
    source: &str,
    nodes: &[SyntaxNode],
    acc: &mut IndexSummaryAcc,
    parent_branches: &[ResolvedSelectorBranch],
    parent_is_grouped: bool,
    current_sass_scope: Option<TextSpan>,
    wrapper_ctx: WrapperContext,
) {
    for node in nodes {
        let mut next_parent_branches = parent_branches.to_vec();
        let mut next_parent_is_grouped = false;
        let mut split_child_branches = false;
        let mut next_sass_scope = current_sass_scope;
        let mut child_wrapper_ctx = wrapper_ctx;
        match &node.payload {
            Some(SyntaxNodePayload::Rule(rule)) => {
                next_sass_scope = Some(node.span);
                let resolved_branches = resolve_rule_selector_branches(rule, parent_branches);
                let custom_property_decl_name_spans =
                    custom_property_decl_name_spans_in_rule(&node.children, source);
                if !custom_property_decl_name_spans.is_empty() {
                    let selector_contexts = selector_contexts_for_rule(rule, &resolved_branches);
                    let source_order_offset = acc.custom_property_decl_facts.len();
                    acc.custom_property_decl_context_selectors
                        .extend(selector_contexts.iter().cloned());
                    acc.custom_property_decl_facts.extend(
                        custom_property_decl_name_spans.into_iter().enumerate().map(
                            |(index, fact)| ParserIndexCustomPropertyDeclFactV0 {
                                name: fact.name,
                                source_order: source_order_offset + index,
                                byte_span: fact.byte_span,
                                range: fact.range,
                                selector_contexts: selector_contexts.clone(),
                                under_media: wrapper_ctx.under_media,
                                under_supports: wrapper_ctx.under_supports,
                                under_layer: wrapper_ctx.under_layer,
                            },
                        ),
                    );
                }
                if !resolved_branches.is_empty() {
                    let resolved: Vec<String> = resolved_branches
                        .iter()
                        .map(|branch| branch.name.clone())
                        .collect();
                    let selector_facts =
                        classify_rule_selector_facts(rule, parent_branches, parent_is_grouped);
                    let selector_definition_source_order_offset =
                        acc.selector_definition_facts.len();
                    let selector_definition_spans = rule_selector_definition_spans(
                        source,
                        node.header_span.unwrap_or(node.span),
                        rule,
                        &resolved,
                        parent_branches,
                    );
                    acc.selector_definition_facts.extend(
                        selector_definition_spans.into_iter().enumerate().map(
                            |(index, (name, byte_span))| ParserIndexSelectorDefinitionFactV0 {
                                name,
                                source_order: selector_definition_source_order_offset + index,
                                byte_span,
                                range: source_range_for_byte_span(source, byte_span),
                                nested_safety_kind: nested_safety_kind_label(
                                    selector_facts.nested_safety,
                                ),
                                bem_suffix_parent_name: match selector_facts.nested_safety {
                                    NestedSafetyKind::BemSuffixSafe => {
                                        parent_branches.first().map(|parent| parent.name.clone())
                                    }
                                    NestedSafetyKind::Flat | NestedSafetyKind::NestedUnsafe => None,
                                },
                                under_media: wrapper_ctx.under_media,
                                under_supports: wrapper_ctx.under_supports,
                                under_layer: wrapper_ctx.under_layer,
                            },
                        ),
                    );
                    acc.bem_suffix_count += selector_facts.bem_suffix_count;
                    increment_nested_safety_count(
                        &mut acc.nested_safety_counts,
                        selector_facts.nested_safety,
                        resolved.len(),
                    );
                    acc.selector_names.extend(resolved.iter().cloned());
                    let composes_facts = collect_rule_composes_facts(&node.children);
                    if composes_facts.local_class_name_count > 0
                        || composes_facts.imported_class_name_count > 0
                        || composes_facts.global_class_name_count > 0
                    {
                        acc.selectors_with_composes_names
                            .extend(resolved.iter().cloned());
                        let selector_multiplier = resolved.len();
                        if composes_facts.local_class_name_count > 0 {
                            acc.local_composes_selector_names
                                .extend(resolved.iter().cloned());
                            acc.local_composes_class_name_count +=
                                composes_facts.local_class_name_count * selector_multiplier;
                        }
                        if composes_facts.imported_class_name_count > 0 {
                            acc.imported_composes_selector_names
                                .extend(resolved.iter().cloned());
                            acc.imported_composes_class_name_count +=
                                composes_facts.imported_class_name_count * selector_multiplier;
                            acc.composes_import_sources.extend(
                                composes_facts.imported_sources.iter().flat_map(|source| {
                                    std::iter::repeat_n(source.clone(), selector_multiplier)
                                }),
                            );
                        }
                        if composes_facts.global_class_name_count > 0 {
                            acc.global_composes_selector_names
                                .extend(resolved.iter().cloned());
                            acc.global_composes_class_name_count +=
                                composes_facts.global_class_name_count * selector_multiplier;
                        }
                        acc.composes_class_name_count += (composes_facts.local_class_name_count
                            + composes_facts.imported_class_name_count
                            + composes_facts.global_class_name_count)
                            * selector_multiplier;
                    }
                    match selector_facts.nested_safety {
                        NestedSafetyKind::BemSuffixSafe => {
                            acc.bem_suffix_safe_selector_names
                                .extend(resolved.iter().cloned());
                            if let Some(parent) = parent_branches.first() {
                                acc.bem_suffix_parent_names.push(parent.name.clone());
                            }
                        }
                        NestedSafetyKind::NestedUnsafe => {
                            acc.nested_unsafe_selector_names
                                .extend(resolved.iter().cloned());
                        }
                        NestedSafetyKind::Flat => {}
                    }
                    next_parent_is_grouped = resolved.len() > 1;
                    next_parent_branches = resolved_branches;
                    split_child_branches = true;
                }
            }
            Some(SyntaxNodePayload::AtRule(at_rule)) => match at_rule.kind {
                AtRuleKind::Media => {
                    child_wrapper_ctx.under_media = true;
                    next_sass_scope = Some(node.span);
                }
                AtRuleKind::Supports => {
                    child_wrapper_ctx.under_supports = true;
                    next_sass_scope = Some(node.span);
                }
                AtRuleKind::Layer => {
                    child_wrapper_ctx.under_layer = true;
                    next_sass_scope = Some(node.span);
                }
                AtRuleKind::AtRoot
                | AtRuleKind::Mixin
                | AtRuleKind::Function
                | AtRuleKind::Generic => {
                    next_sass_scope = Some(node.span);
                }
                AtRuleKind::Keyframes
                | AtRuleKind::Value
                | AtRuleKind::Include
                | AtRuleKind::Use
                | AtRuleKind::Forward
                | AtRuleKind::Import => {}
            },
            _ => {}
        }

        match &node.payload {
            Some(SyntaxNodePayload::AtRule(at_rule)) => match at_rule.kind {
                AtRuleKind::Keyframes if !at_rule.params.is_empty() => {
                    acc.keyframes_names.push(at_rule.params.clone());
                }
                AtRuleKind::Keyframes => {}
                AtRuleKind::Value => {
                    if let Some(import_specs) = parse_value_import_specs(&at_rule.params) {
                        acc.value_import_alias_count += import_specs
                            .iter()
                            .filter(|spec| spec.imported_name != spec.local_name)
                            .count();
                        acc.value_import_names
                            .extend(import_specs.iter().map(|spec| spec.local_name.clone()));
                        acc.value_import_source_by_name
                            .extend(import_specs.iter().filter_map(|spec| {
                                spec.from_source
                                    .as_ref()
                                    .map(|source| (spec.local_name.clone(), source.clone()))
                            }));
                        acc.value_import_sources
                            .extend(import_specs.into_iter().filter_map(|spec| spec.from_source));
                    } else if let Some((name, _)) = parse_local_value_decl_parts(&at_rule.params) {
                        acc.value_decl_names.push(name.to_string());
                    }
                }
                AtRuleKind::Mixin => {
                    let params_span = at_rule_params_span(source, node, at_rule);
                    if let Some(name_span) =
                        parse_sass_callable_name_with_span(&at_rule.params, params_span.start)
                    {
                        acc.sass_mixin_decl_names.push(name_span.name.clone());
                        acc.sass_symbol_decl_facts
                            .push(ParserIndexSassSymbolDeclFactV0 {
                                symbol_kind: "mixin",
                                name: name_span.name,
                                role: "declaration",
                                byte_span: name_span.byte_span,
                                range: source_range_for_byte_span(source, name_span.byte_span),
                            });
                    }
                    let parameter_names = parse_sass_parameter_names(&at_rule.params);
                    acc.sass_variable_decl_facts
                        .extend(parameter_names.iter().map(|name| SassVariableDeclFact {
                            name: name.clone(),
                            scope: SassVariableScope::Span(node.span),
                        }));
                    acc.sass_variable_parameter_names.extend(parameter_names);
                }
                AtRuleKind::Include => {
                    if let Some(name_span) = parse_sass_callable_name_with_span(
                        &at_rule.params,
                        at_rule_params_span(source, node, at_rule).start,
                    ) && name_span.namespace.is_none()
                    {
                        acc.sass_mixin_include_names.push(name_span.name);
                    }
                }
                AtRuleKind::Function => {
                    let params_span = at_rule_params_span(source, node, at_rule);
                    if let Some(name_span) =
                        parse_sass_callable_name_with_span(&at_rule.params, params_span.start)
                    {
                        acc.sass_function_decl_names.push(name_span.name.clone());
                        acc.sass_symbol_decl_facts
                            .push(ParserIndexSassSymbolDeclFactV0 {
                                symbol_kind: "function",
                                name: name_span.name,
                                role: "declaration",
                                byte_span: name_span.byte_span,
                                range: source_range_for_byte_span(source, name_span.byte_span),
                            });
                    }
                    let parameter_names = parse_sass_parameter_names(&at_rule.params);
                    acc.sass_variable_decl_facts
                        .extend(parameter_names.iter().map(|name| SassVariableDeclFact {
                            name: name.clone(),
                            scope: SassVariableScope::Span(node.span),
                        }));
                    acc.sass_variable_parameter_names.extend(parameter_names);
                }
                AtRuleKind::Use => {
                    acc.sass_module_use_sources
                        .extend(parse_sass_module_sources(&at_rule.params));
                    acc.sass_module_use_edges
                        .extend(parse_sass_module_use_edges(&at_rule.params));
                }
                AtRuleKind::Forward => {
                    acc.sass_module_forward_sources
                        .extend(parse_sass_module_sources(&at_rule.params));
                }
                AtRuleKind::Import => {
                    acc.sass_module_import_sources
                        .extend(parse_sass_module_sources(&at_rule.params));
                    acc.sass_module_use_sources
                        .extend(parse_sass_module_sources(&at_rule.params));
                    acc.sass_module_use_edges
                        .extend(parse_sass_module_import_use_edges(&at_rule.params));
                }
                _ => {}
            },
            Some(SyntaxNodePayload::Declaration(declaration)) => {
                if is_css_custom_property_name(&declaration.property) {
                    acc.custom_property_decl_names
                        .push(declaration.property.clone());
                    if wrapper_ctx.under_media {
                        acc.custom_property_decl_names_under_media
                            .push(declaration.property.clone());
                    }
                    if wrapper_ctx.under_supports {
                        acc.custom_property_decl_names_under_supports
                            .push(declaration.property.clone());
                    }
                    if wrapper_ctx.under_layer {
                        acc.custom_property_decl_names_under_layer
                            .push(declaration.property.clone());
                    }
                }
                if let Some(name) = parse_sass_variable_decl_name(&declaration.property) {
                    let byte_span = ParserByteSpanV0 {
                        start: node.span.start + 1,
                        end: node.span.start + declaration.property.len(),
                    };
                    acc.sass_variable_decl_facts.push(SassVariableDeclFact {
                        name: name.clone(),
                        scope: current_sass_scope
                            .map(SassVariableScope::Span)
                            .unwrap_or(SassVariableScope::File),
                    });
                    acc.sass_symbol_decl_facts
                        .push(ParserIndexSassSymbolDeclFactV0 {
                            symbol_kind: "variable",
                            name: name.clone(),
                            role: "declaration",
                            byte_span,
                            range: source_range_for_byte_span(source, byte_span),
                        });
                    acc.sass_variable_decl_names.push(name);
                }
            }
            _ => {}
        }
        if split_child_branches {
            for parent_branch in &next_parent_branches {
                collect_index_names(
                    source,
                    &node.children,
                    acc,
                    std::slice::from_ref(parent_branch),
                    next_parent_is_grouped,
                    next_sass_scope,
                    child_wrapper_ctx,
                );
            }
        } else {
            collect_index_names(
                source,
                &node.children,
                acc,
                &next_parent_branches,
                next_parent_is_grouped,
                next_sass_scope,
                child_wrapper_ctx,
            );
        }
    }
}

fn custom_property_decl_name_spans_in_rule(
    children: &[SyntaxNode],
    source: &str,
) -> Vec<CustomPropertyDeclNameSpan> {
    children
        .iter()
        .filter_map(|child| match &child.payload {
            Some(SyntaxNodePayload::Declaration(declaration))
                if is_css_custom_property_name(&declaration.property) =>
            {
                let byte_span = ParserByteSpanV0 {
                    start: child.span.start,
                    end: child.span.start + declaration.property.len(),
                };
                Some(CustomPropertyDeclNameSpan {
                    name: declaration.property.clone(),
                    byte_span,
                    range: source_range_for_byte_span(source, byte_span),
                })
            }
            _ => None,
        })
        .collect()
}

fn selector_contexts_for_rule(
    rule: &RulePayload,
    resolved_branches: &[ResolvedSelectorBranch],
) -> Vec<String> {
    let mut contexts: Vec<String> = rule
        .selector_groups
        .iter()
        .map(|group| group.raw.clone())
        .chain(
            resolved_branches
                .iter()
                .map(|branch| format!(".{}", branch.name)),
        )
        .collect();
    contexts.sort();
    contexts.dedup();
    contexts
}

fn collect_index_refs_and_counts(
    nodes: &[SyntaxNode],
    value_ref_ctx: ValueRefContext<'_>,
    known_keyframe_names: &BTreeSet<String>,
    acc: &mut IndexSummaryAcc,
) {
    for node in nodes {
        match &node.payload {
            Some(SyntaxNodePayload::Declaration(declaration)) => {
                acc.custom_property_ref_names
                    .extend(find_css_var_ref_names(&declaration.value));
                match classify_declaration_kind(&declaration.property) {
                    DeclarationKind::Composes => {}
                    DeclarationKind::Animation => {
                        acc.animation_ref_names.extend(find_identifier_matches(
                            &declaration.value,
                            known_keyframe_names,
                        ));
                        extend_value_ref_facts(
                            acc,
                            find_identifier_matches(&declaration.value, value_ref_ctx.known),
                            value_ref_ctx,
                            ValueRefOrigin::Declaration,
                        );
                    }
                    DeclarationKind::AnimationName => {
                        acc.animation_name_ref_names.extend(find_identifier_matches(
                            &declaration.value,
                            known_keyframe_names,
                        ));
                        extend_value_ref_facts(
                            acc,
                            find_identifier_matches(&declaration.value, value_ref_ctx.known),
                            value_ref_ctx,
                            ValueRefOrigin::Declaration,
                        );
                    }
                    DeclarationKind::Generic => {
                        extend_value_ref_facts(
                            acc,
                            find_identifier_matches(&declaration.value, value_ref_ctx.known),
                            value_ref_ctx,
                            ValueRefOrigin::Declaration,
                        );
                    }
                }
            }
            Some(SyntaxNodePayload::AtRule(at_rule)) if at_rule.kind == AtRuleKind::Value => {
                if let Some((name, value)) = parse_local_value_decl_parts(&at_rule.params) {
                    let value_refs: Vec<String> =
                        find_identifier_matches(value, value_ref_ctx.known)
                            .into_iter()
                            .filter(|candidate| candidate != name)
                            .collect();
                    if value_refs
                        .iter()
                        .any(|candidate| value_ref_ctx.local.contains(candidate))
                    {
                        acc.value_decl_names_with_local_refs.push(name.to_string());
                    }
                    if value_refs
                        .iter()
                        .any(|candidate| value_ref_ctx.imported.contains(candidate))
                    {
                        acc.value_decl_names_with_imported_refs
                            .push(name.to_string());
                    }
                    extend_value_ref_facts(
                        acc,
                        value_refs,
                        value_ref_ctx,
                        ValueRefOrigin::ValueDecl,
                    );
                }
            }
            _ => {}
        }
        collect_index_refs_and_counts(&node.children, value_ref_ctx, known_keyframe_names, acc);
    }
}

fn collect_sass_ref_facts(
    nodes: &[SyntaxNode],
    source: &str,
    known_function_names: &BTreeSet<String>,
    acc: &mut IndexSummaryAcc,
) {
    for node in nodes {
        match &node.payload {
            Some(SyntaxNodePayload::Declaration(declaration)) => {
                let value_span = declaration_value_span(source, node);
                let variable_refs =
                    find_sass_variable_ref_spans(&declaration.value, value_span.start);
                acc.sass_variable_ref_names
                    .extend(variable_refs.iter().map(|span| span.name.clone()));
                acc.sass_variable_ref_facts
                    .extend(variable_refs.into_iter().map(|span| SassVariableRefFact {
                        name: span.name,
                        namespace: span.namespace,
                        byte_span: span.byte_span,
                    }));
                acc.sass_function_call_names
                    .extend(find_sass_function_calls(
                        &declaration.value,
                        known_function_names,
                    ));
            }
            Some(SyntaxNodePayload::AtRule(at_rule)) => match at_rule.kind {
                AtRuleKind::Mixin | AtRuleKind::Function => {}
                _ => {
                    let params_span = at_rule_params_span(source, node, at_rule);
                    let variable_refs =
                        find_sass_variable_ref_spans(&at_rule.params, params_span.start);
                    acc.sass_variable_ref_names
                        .extend(variable_refs.iter().map(|span| span.name.clone()));
                    acc.sass_variable_ref_facts
                        .extend(variable_refs.into_iter().map(|span| SassVariableRefFact {
                            name: span.name,
                            namespace: span.namespace,
                            byte_span: span.byte_span,
                        }));
                    acc.sass_function_call_names
                        .extend(find_sass_function_calls(
                            &at_rule.params,
                            known_function_names,
                        ));
                }
            },
            _ => {}
        }
        collect_sass_ref_facts(&node.children, source, known_function_names, acc);
    }
}

fn summarize_sass_same_file_resolution(
    acc: &IndexSummaryAcc,
) -> ParserIndexSassSameFileResolutionFactsV0 {
    let mixin_targets: BTreeSet<&str> = acc
        .sass_mixin_decl_names
        .iter()
        .map(String::as_str)
        .collect();
    let function_targets: BTreeSet<&str> = acc
        .sass_function_decl_names
        .iter()
        .map(String::as_str)
        .collect();

    let mut resolved_variable_ref_names = BTreeSet::new();
    let mut unresolved_variable_ref_names = BTreeSet::new();
    for fact in &acc.sass_variable_ref_facts {
        if fact.namespace.is_some() {
            continue;
        }
        if resolve_sass_variable_ref(&fact.name, fact.byte_span, &acc.sass_variable_decl_facts) {
            resolved_variable_ref_names.insert(fact.name.clone());
        } else {
            unresolved_variable_ref_names.insert(fact.name.clone());
        }
    }

    ParserIndexSassSameFileResolutionFactsV0 {
        resolved_variable_ref_names: resolved_variable_ref_names.into_iter().collect(),
        unresolved_variable_ref_names: unresolved_variable_ref_names.into_iter().collect(),
        resolved_mixin_include_names: names_matching(&acc.sass_mixin_include_names, &mixin_targets),
        unresolved_mixin_include_names: names_not_matching(
            &acc.sass_mixin_include_names,
            &mixin_targets,
        ),
        resolved_function_call_names: names_matching(
            &acc.sass_function_call_names,
            &function_targets,
        ),
    }
}

fn names_matching(names: &[String], targets: &BTreeSet<&str>) -> Vec<String> {
    names
        .iter()
        .filter(|name| targets.contains(name.as_str()))
        .cloned()
        .collect()
}

fn names_not_matching(names: &[String], targets: &BTreeSet<&str>) -> Vec<String> {
    names
        .iter()
        .filter(|name| !targets.contains(name.as_str()))
        .cloned()
        .collect()
}

fn extend_value_ref_facts(
    acc: &mut IndexSummaryAcc,
    value_refs: Vec<String>,
    value_ref_ctx: ValueRefContext<'_>,
    origin: ValueRefOrigin,
) {
    acc.value_ref_names.extend(value_refs.iter().cloned());

    let local_refs: Vec<String> = value_refs
        .iter()
        .filter(|name| value_ref_ctx.local.contains(*name))
        .cloned()
        .collect();
    acc.local_value_ref_names.extend(local_refs);

    let imported_refs: Vec<String> = value_refs
        .iter()
        .filter(|name| value_ref_ctx.imported.contains(*name))
        .cloned()
        .collect();
    acc.imported_value_ref_names
        .extend(imported_refs.iter().cloned());

    let imported_ref_sources: Vec<String> = imported_refs
        .iter()
        .filter_map(|name| acc.value_import_source_by_name.get(name))
        .cloned()
        .collect();
    acc.imported_value_ref_sources
        .extend(imported_ref_sources.iter().cloned());

    match origin {
        ValueRefOrigin::Declaration => {
            acc.declaration_value_ref_names.extend(value_refs);
            acc.declaration_imported_value_ref_sources
                .extend(imported_ref_sources);
        }
        ValueRefOrigin::ValueDecl => {
            acc.value_decl_ref_names.extend(value_refs);
            acc.value_decl_imported_value_ref_sources
                .extend(imported_ref_sources);
        }
    }
}

fn collect_index_selector_attachment_facts(
    nodes: &[SyntaxNode],
    ctx: SelectorAttachmentContext<'_>,
    acc: &mut IndexSummaryAcc,
    parent_branches: &[ResolvedSelectorBranch],
) {
    collect_index_selector_attachment_facts_with_context(
        nodes,
        ctx,
        acc,
        parent_branches,
        WrapperContext::default(),
    );
}

fn collect_index_selector_attachment_facts_with_context(
    nodes: &[SyntaxNode],
    ctx: SelectorAttachmentContext<'_>,
    acc: &mut IndexSummaryAcc,
    parent_branches: &[ResolvedSelectorBranch],
    wrapper_ctx: WrapperContext,
) {
    for node in nodes {
        let mut next_parent_branches = parent_branches.to_vec();
        let mut split_child_branches = false;
        let mut child_wrapper_ctx = wrapper_ctx;
        if let Some(SyntaxNodePayload::Rule(rule)) = &node.payload {
            let resolved_branches = resolve_rule_selector_branches(rule, parent_branches);
            if !resolved_branches.is_empty() {
                let resolved: Vec<String> = resolved_branches
                    .iter()
                    .map(|branch| branch.name.clone())
                    .collect();
                let ref_facts = collect_rule_reference_facts(
                    &node.children,
                    ctx.source,
                    ctx.value_ref_ctx,
                    ctx.known_keyframe_names,
                    ctx.sass_ref_ctx,
                );
                if ref_facts.has_value_refs {
                    acc.selectors_with_value_refs_names
                        .extend(resolved.iter().cloned());
                    if wrapper_ctx.under_media {
                        acc.selectors_with_value_refs_under_media_names
                            .extend(resolved.iter().cloned());
                    }
                    if wrapper_ctx.under_supports {
                        acc.selectors_with_value_refs_under_supports_names
                            .extend(resolved.iter().cloned());
                    }
                    if wrapper_ctx.under_layer {
                        acc.selectors_with_value_refs_under_layer_names
                            .extend(resolved.iter().cloned());
                    }
                }
                if ref_facts.has_local_value_refs {
                    acc.selectors_with_local_value_refs_names
                        .extend(resolved.iter().cloned());
                    if wrapper_ctx.under_media {
                        acc.selectors_with_local_value_refs_under_media_names
                            .extend(resolved.iter().cloned());
                    }
                    if wrapper_ctx.under_supports {
                        acc.selectors_with_local_value_refs_under_supports_names
                            .extend(resolved.iter().cloned());
                    }
                    if wrapper_ctx.under_layer {
                        acc.selectors_with_local_value_refs_under_layer_names
                            .extend(resolved.iter().cloned());
                    }
                }
                if ref_facts.has_imported_value_refs {
                    acc.selectors_with_imported_value_refs_names
                        .extend(resolved.iter().cloned());
                    if wrapper_ctx.under_media {
                        acc.selectors_with_imported_value_refs_under_media_names
                            .extend(resolved.iter().cloned());
                    }
                    if wrapper_ctx.under_supports {
                        acc.selectors_with_imported_value_refs_under_supports_names
                            .extend(resolved.iter().cloned());
                    }
                    if wrapper_ctx.under_layer {
                        acc.selectors_with_imported_value_refs_under_layer_names
                            .extend(resolved.iter().cloned());
                    }
                }
                if ref_facts.has_animation_refs {
                    acc.selectors_with_animation_ref_names
                        .extend(resolved.iter().cloned());
                    if wrapper_ctx.under_media {
                        acc.selectors_with_animation_refs_under_media_names
                            .extend(resolved.iter().cloned());
                    }
                    if wrapper_ctx.under_supports {
                        acc.selectors_with_animation_refs_under_supports_names
                            .extend(resolved.iter().cloned());
                    }
                    if wrapper_ctx.under_layer {
                        acc.selectors_with_animation_refs_under_layer_names
                            .extend(resolved.iter().cloned());
                    }
                }
                if ref_facts.has_animation_name_refs {
                    acc.selectors_with_animation_name_ref_names
                        .extend(resolved.iter().cloned());
                    if wrapper_ctx.under_media {
                        acc.selectors_with_animation_name_refs_under_media_names
                            .extend(resolved.iter().cloned());
                    }
                    if wrapper_ctx.under_supports {
                        acc.selectors_with_animation_name_refs_under_supports_names
                            .extend(resolved.iter().cloned());
                    }
                    if wrapper_ctx.under_layer {
                        acc.selectors_with_animation_name_refs_under_layer_names
                            .extend(resolved.iter().cloned());
                    }
                }
                if ref_facts.has_custom_property_refs {
                    let selector_contexts = selector_contexts_for_rule(rule, &resolved_branches);
                    let source_order_offset = acc.custom_property_ref_facts.len();
                    acc.selectors_with_custom_property_refs_names
                        .extend(resolved.iter().cloned());
                    acc.custom_property_ref_facts.extend(
                        ref_facts
                            .custom_property_ref_names
                            .iter()
                            .cloned()
                            .enumerate()
                            .map(|(index, name)| ParserIndexCustomPropertyRefFactV0 {
                                name,
                                source_order: source_order_offset + index,
                                selector_contexts: selector_contexts.clone(),
                                under_media: wrapper_ctx.under_media,
                                under_supports: wrapper_ctx.under_supports,
                                under_layer: wrapper_ctx.under_layer,
                            }),
                    );
                    if wrapper_ctx.under_media {
                        acc.selectors_with_custom_property_refs_under_media_names
                            .extend(resolved.iter().cloned());
                    }
                    if wrapper_ctx.under_supports {
                        acc.selectors_with_custom_property_refs_under_supports_names
                            .extend(resolved.iter().cloned());
                    }
                    if wrapper_ctx.under_layer {
                        acc.selectors_with_custom_property_refs_under_layer_names
                            .extend(resolved.iter().cloned());
                    }
                }
                if ref_facts.has_sass_variable_refs {
                    acc.sass_selectors_with_variable_refs_names
                        .extend(resolved.iter().cloned());
                }
                if ref_facts.has_resolved_sass_variable_refs {
                    acc.sass_selectors_with_resolved_variable_refs_names
                        .extend(resolved.iter().cloned());
                }
                if ref_facts.has_unresolved_sass_variable_refs {
                    acc.sass_selectors_with_unresolved_variable_refs_names
                        .extend(resolved.iter().cloned());
                }
                if ref_facts.has_sass_mixin_includes {
                    acc.sass_selectors_with_mixin_includes_names
                        .extend(resolved.iter().cloned());
                }
                if ref_facts.has_resolved_sass_mixin_includes {
                    acc.sass_selectors_with_resolved_mixin_includes_names
                        .extend(resolved.iter().cloned());
                }
                if ref_facts.has_unresolved_sass_mixin_includes {
                    acc.sass_selectors_with_unresolved_mixin_includes_names
                        .extend(resolved.iter().cloned());
                }
                if ref_facts.has_sass_function_calls {
                    acc.sass_selectors_with_function_calls_names
                        .extend(resolved.iter().cloned());
                }
                for selector_name in &resolved {
                    acc.sass_selector_symbol_facts
                        .extend(ref_facts.sass_symbol_facts.iter().map(|fact| {
                            ParserIndexSassSelectorSymbolFactV0 {
                                selector_name: selector_name.clone(),
                                symbol_kind: fact.symbol_kind,
                                name: fact.name.clone(),
                                namespace: fact.namespace.clone(),
                                role: fact.role,
                                resolution: fact.resolution,
                                byte_span: fact.byte_span,
                                range: source_range_for_byte_span(ctx.source, fact.byte_span),
                            }
                        }));
                }
                let composes_facts = collect_rule_composes_facts(&node.children);
                if composes_facts.local_class_name_count > 0
                    || composes_facts.imported_class_name_count > 0
                    || composes_facts.global_class_name_count > 0
                {
                    if wrapper_ctx.under_media {
                        acc.selectors_with_composes_under_media_names
                            .extend(resolved.iter().cloned());
                    }
                    if wrapper_ctx.under_supports {
                        acc.selectors_with_composes_under_supports_names
                            .extend(resolved.iter().cloned());
                    }
                    if wrapper_ctx.under_layer {
                        acc.selectors_with_composes_under_layer_names
                            .extend(resolved.iter().cloned());
                    }
                }
                if composes_facts.local_class_name_count > 0 {
                    if wrapper_ctx.under_media {
                        acc.local_composes_selector_names_under_media
                            .extend(resolved.iter().cloned());
                    }
                    if wrapper_ctx.under_supports {
                        acc.local_composes_selector_names_under_supports
                            .extend(resolved.iter().cloned());
                    }
                    if wrapper_ctx.under_layer {
                        acc.local_composes_selector_names_under_layer
                            .extend(resolved.iter().cloned());
                    }
                }
                if composes_facts.imported_class_name_count > 0 {
                    if wrapper_ctx.under_media {
                        acc.imported_composes_selector_names_under_media
                            .extend(resolved.iter().cloned());
                        acc.composes_import_sources_under_media.extend(
                            composes_facts.imported_sources.iter().flat_map(|source| {
                                std::iter::repeat_n(source.clone(), resolved.len())
                            }),
                        );
                    }
                    if wrapper_ctx.under_supports {
                        acc.imported_composes_selector_names_under_supports
                            .extend(resolved.iter().cloned());
                        acc.composes_import_sources_under_supports.extend(
                            composes_facts.imported_sources.iter().flat_map(|source| {
                                std::iter::repeat_n(source.clone(), resolved.len())
                            }),
                        );
                    }
                    if wrapper_ctx.under_layer {
                        acc.imported_composes_selector_names_under_layer
                            .extend(resolved.iter().cloned());
                        acc.composes_import_sources_under_layer.extend(
                            composes_facts.imported_sources.iter().flat_map(|source| {
                                std::iter::repeat_n(source.clone(), resolved.len())
                            }),
                        );
                    }
                }
                if composes_facts.global_class_name_count > 0 {
                    if wrapper_ctx.under_media {
                        acc.global_composes_selector_names_under_media
                            .extend(resolved.iter().cloned());
                    }
                    if wrapper_ctx.under_supports {
                        acc.global_composes_selector_names_under_supports
                            .extend(resolved.iter().cloned());
                    }
                    if wrapper_ctx.under_layer {
                        acc.global_composes_selector_names_under_layer
                            .extend(resolved.iter().cloned());
                    }
                }
                if wrapper_ctx.under_media {
                    acc.selectors_under_media_names
                        .extend(resolved.iter().cloned());
                }
                if wrapper_ctx.under_supports {
                    acc.selectors_under_supports_names
                        .extend(resolved.iter().cloned());
                }
                if wrapper_ctx.under_layer {
                    acc.selectors_under_layer_names
                        .extend(resolved.iter().cloned());
                }
                next_parent_branches = resolved_branches;
                split_child_branches = true;
            }
        } else if let Some(SyntaxNodePayload::AtRule(at_rule)) = &node.payload {
            if at_rule.kind == AtRuleKind::Keyframes && !at_rule.params.is_empty() {
                if wrapper_ctx.under_media {
                    acc.keyframes_names_under_media.push(at_rule.params.clone());
                }
                if wrapper_ctx.under_supports {
                    acc.keyframes_names_under_supports
                        .push(at_rule.params.clone());
                }
                if wrapper_ctx.under_layer {
                    acc.keyframes_names_under_layer.push(at_rule.params.clone());
                }
            }
            match at_rule.kind {
                AtRuleKind::Media => child_wrapper_ctx.under_media = true,
                AtRuleKind::Supports => child_wrapper_ctx.under_supports = true,
                AtRuleKind::Layer => child_wrapper_ctx.under_layer = true,
                _ => {}
            }
        }

        if split_child_branches {
            for parent_branch in &next_parent_branches {
                collect_index_selector_attachment_facts_with_context(
                    &node.children,
                    ctx,
                    acc,
                    std::slice::from_ref(parent_branch),
                    child_wrapper_ctx,
                );
            }
        } else {
            collect_index_selector_attachment_facts_with_context(
                &node.children,
                ctx,
                acc,
                &next_parent_branches,
                child_wrapper_ctx,
            );
        }
    }
}

fn parse_local_value_decl_parts(params: &str) -> Option<(&str, &str)> {
    if params.contains(" from ") {
        return None;
    }
    let (name, value) = params.split_once(':')?;
    let trimmed_name = name.trim();
    let trimmed_value = value.trim();
    if trimmed_name.is_empty() || trimmed_value.is_empty() {
        return None;
    }
    Some((trimmed_name, trimmed_value))
}

struct ValueImportSpec {
    imported_name: String,
    local_name: String,
    from_source: Option<String>,
}

fn parse_value_import_specs(params: &str) -> Option<Vec<ValueImportSpec>> {
    let (raw_specs, raw_source) = params.split_once(" from ")?;
    let from_source = parse_quoted_import_source(raw_source);
    let mut specs = Vec::new();
    for raw_spec in raw_specs.split(',') {
        let trimmed = raw_spec.trim();
        if trimmed.is_empty() {
            continue;
        }
        let imported_name = trimmed
            .split_once(" as ")
            .map(|(imported, _)| imported.trim())
            .unwrap_or(trimmed);
        let local_name = trimmed
            .split_once(" as ")
            .map(|(_, local)| local.trim())
            .unwrap_or(trimmed);
        if !imported_name.is_empty() && !local_name.is_empty() {
            specs.push(ValueImportSpec {
                imported_name: imported_name.to_string(),
                local_name: local_name.to_string(),
                from_source: from_source.clone(),
            });
        }
    }
    (!specs.is_empty()).then_some(specs)
}

fn parse_composes_spec(value: &str) -> Option<ComposesSpec> {
    let head = value
        .split_once(" from ")
        .map(|(left, _)| left)
        .unwrap_or(value);
    let class_names: Vec<String> = head
        .split_whitespace()
        .filter(|name| !name.is_empty())
        .map(ToString::to_string)
        .collect();
    if class_names.is_empty() {
        return None;
    }
    let from_source = value
        .split_once(" from ")
        .and_then(|(_, source)| parse_quoted_import_source(source));
    let kind = match value.split_once(" from ").map(|(_, source)| source.trim()) {
        Some("global") => ComposesKind::Global,
        Some(_) => ComposesKind::Imported,
        None => ComposesKind::Local,
    };
    Some(ComposesSpec {
        class_names,
        kind,
        from_source,
    })
}

fn parse_quoted_import_source(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.len() < 2 {
        return None;
    }
    let quote = trimmed.chars().next()?;
    if !matches!(quote, '"' | '\'') || !trimmed.ends_with(quote) {
        return None;
    }
    Some(trimmed[1..trimmed.len() - 1].to_string())
}

fn parse_sass_variable_decl_name(property: &str) -> Option<String> {
    let name = property.trim().strip_prefix('$')?;
    (!name.is_empty() && name.chars().all(is_sass_ident_continue)).then(|| name.to_string())
}

fn parse_sass_callable_name_with_span(params: &str, base_start: usize) -> Option<SassNameSpan> {
    let trimmed_start = params.find(|ch: char| !ch.is_whitespace())?;
    let trimmed = &params[trimmed_start..];
    let end = trimmed
        .find(|ch: char| ch.is_whitespace() || ch == '(')
        .unwrap_or(trimmed.len());
    let raw_name = &trimmed[..end];
    let (namespace, name, name_offset) = if let Some((namespace, name)) = raw_name.split_once('.') {
        if !is_valid_sass_namespace(namespace) {
            return None;
        }
        (Some(namespace.to_string()), name, namespace.len() + 1)
    } else {
        (None, raw_name, 0)
    };
    (!name.is_empty() && name.chars().all(is_sass_ident_continue)).then(|| SassNameSpan {
        name: name.to_string(),
        namespace,
        byte_span: ParserByteSpanV0 {
            start: base_start + trimmed_start + name_offset,
            end: base_start + trimmed_start + end,
        },
    })
}

fn parse_sass_parameter_names(params: &str) -> Vec<String> {
    let Some(open_index) = params.find('(') else {
        return Vec::new();
    };
    let close_index = params[open_index + 1..]
        .find(')')
        .map(|index| open_index + 1 + index)
        .unwrap_or(params.len());
    find_sass_variable_refs(&params[open_index + 1..close_index])
}

fn parse_sass_module_sources(params: &str) -> Vec<String> {
    let mut sources = Vec::new();
    let chars: Vec<char> = params.chars().collect();
    let mut index = 0usize;

    while index < chars.len() {
        let quote = chars[index];
        if !matches!(quote, '"' | '\'') {
            index += 1;
            continue;
        }
        index += 1;
        let start = index;
        while index < chars.len() {
            if chars[index] == '\\' {
                index = (index + 2).min(chars.len());
                continue;
            }
            if chars[index] == quote {
                break;
            }
            index += 1;
        }
        if start < index {
            sources.push(chars[start..index].iter().collect());
        }
        index += usize::from(index < chars.len());
    }

    sources
}

fn parse_sass_module_use_edges(params: &str) -> Vec<ParserIndexSassModuleUseFactV0> {
    let alias = parse_sass_use_alias(params);
    parse_sass_module_sources(params)
        .into_iter()
        .map(|source| match alias.as_deref() {
            Some("*") => ParserIndexSassModuleUseFactV0 {
                source,
                namespace_kind: "wildcard",
                namespace: None,
            },
            Some(namespace) if is_valid_sass_namespace(namespace) => {
                ParserIndexSassModuleUseFactV0 {
                    source,
                    namespace_kind: "alias",
                    namespace: Some(namespace.to_string()),
                }
            }
            _ => ParserIndexSassModuleUseFactV0 {
                namespace: default_sass_namespace_for_source(&source),
                source,
                namespace_kind: "default",
            },
        })
        .collect()
}

fn parse_sass_module_import_use_edges(params: &str) -> Vec<ParserIndexSassModuleUseFactV0> {
    parse_sass_module_sources(params)
        .into_iter()
        .map(|source| ParserIndexSassModuleUseFactV0 {
            source,
            namespace_kind: "wildcard",
            namespace: None,
        })
        .collect()
}

fn parse_sass_use_alias(params: &str) -> Option<String> {
    let chars: Vec<char> = params.chars().collect();
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut index = 0usize;
    let mut quote: Option<char> = None;

    while index < chars.len() {
        let ch = chars[index];
        if let Some(active_quote) = quote {
            if ch == '\\' && index + 1 < chars.len() {
                index += 2;
                continue;
            }
            if ch == active_quote {
                quote = None;
            }
            index += 1;
            continue;
        }

        if ch == '"' || ch == '\'' {
            if !current.is_empty() {
                tokens.push(std::mem::take(&mut current));
            }
            quote = Some(ch);
            index += 1;
            continue;
        }

        if ch.is_whitespace() || ch == ';' || ch == ',' {
            if !current.is_empty() {
                tokens.push(std::mem::take(&mut current));
            }
            index += 1;
            continue;
        }

        current.push(ch);
        index += 1;
    }

    if !current.is_empty() {
        tokens.push(current);
    }

    tokens
        .windows(2)
        .find_map(|window| (window[0].eq_ignore_ascii_case("as")).then(|| window[1].clone()))
}

fn default_sass_namespace_for_source(source: &str) -> Option<String> {
    let clean = source
        .split(['?', '#'])
        .next()
        .unwrap_or(source)
        .trim_end_matches('/');
    let segment = clean.rsplit('/').next().unwrap_or(clean);
    let package_segment = segment.rsplit(':').next().unwrap_or(segment);
    let stem = package_segment
        .rsplit_once('.')
        .map(|(stem, _)| stem)
        .unwrap_or(package_segment);
    let stem = stem.strip_prefix('_').unwrap_or(stem);

    is_valid_sass_namespace(stem).then(|| stem.to_string())
}

fn is_valid_sass_namespace(value: &str) -> bool {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    is_sass_ident_start(first) && chars.all(is_sass_ident_continue)
}

fn find_sass_variable_refs(raw: &str) -> Vec<String> {
    find_sass_variable_ref_spans(raw, 0)
        .into_iter()
        .map(|span| span.name)
        .collect()
}

fn find_sass_variable_ref_spans(raw: &str, base_start: usize) -> Vec<SassNameSpan> {
    let mut refs = Vec::new();
    let mut chars = raw.char_indices().peekable();
    let mut quote: Option<char> = None;

    while let Some((byte_index, ch)) = chars.next() {
        if let Some(active_quote) = quote {
            if ch == '\\' {
                chars.next();
                continue;
            }
            if ch == active_quote {
                quote = None;
            }
            continue;
        }

        if ch == '"' || ch == '\'' {
            quote = Some(ch);
            continue;
        }

        if ch == '$' {
            let namespace = sass_module_qualified_namespace_before(raw, byte_index);
            let name_start = byte_index + ch.len_utf8();
            let mut name_end = name_start;
            let mut name = String::new();
            while let Some(&(next_index, next_ch)) = chars.peek() {
                if !is_sass_ident_continue(next_ch) {
                    break;
                }
                name.push(next_ch);
                name_end = next_index + next_ch.len_utf8();
                chars.next();
            }
            if !name.is_empty() {
                refs.push(SassNameSpan {
                    name,
                    namespace,
                    byte_span: ParserByteSpanV0 {
                        start: base_start + byte_index,
                        end: base_start + name_end,
                    },
                });
            }
        }
    }

    refs
}

fn find_sass_function_calls(raw: &str, known_function_names: &BTreeSet<String>) -> Vec<String> {
    find_sass_function_call_spans(raw, 0, known_function_names)
        .into_iter()
        .filter(|span| span.namespace.is_none())
        .map(|span| span.name)
        .collect()
}

fn find_sass_function_call_spans(
    raw: &str,
    base_start: usize,
    known_function_names: &BTreeSet<String>,
) -> Vec<SassNameSpan> {
    let mut calls = Vec::new();
    let mut chars = raw.char_indices().peekable();
    let mut quote: Option<char> = None;

    while let Some((byte_index, ch)) = chars.next() {
        if let Some(active_quote) = quote {
            if ch == '\\' {
                chars.next();
                continue;
            }
            if ch == active_quote {
                quote = None;
            }
            continue;
        }

        if ch == '"' || ch == '\'' {
            quote = Some(ch);
            continue;
        }

        if is_sass_ident_start(ch) {
            let mut name = String::from(ch);
            let name_start = byte_index;
            let mut name_end = byte_index + ch.len_utf8();
            while let Some(&(next_index, next_ch)) = chars.peek() {
                if !is_sass_ident_continue(next_ch) {
                    break;
                }
                name.push(next_ch);
                name_end = next_index + next_ch.len_utf8();
                chars.next();
            }
            let mut namespace = None;
            if matches!(chars.peek(), Some(&(_, '.'))) {
                chars.next();
                let Some(&(member_start, member_first)) = chars.peek() else {
                    continue;
                };
                if !is_sass_ident_start(member_first) {
                    continue;
                }
                namespace = Some(name);
                name = String::new();
                name_end = member_start;
                while let Some(&(next_index, next_ch)) = chars.peek() {
                    if !is_sass_ident_continue(next_ch) {
                        break;
                    }
                    name.push(next_ch);
                    name_end = next_index + next_ch.len_utf8();
                    chars.next();
                }
            }
            while let Some(&(_, next_ch)) = chars.peek() {
                if !next_ch.is_whitespace() {
                    break;
                }
                chars.next();
            }
            if matches!(chars.peek(), Some(&(_, '(')))
                && (namespace.is_some() || known_function_names.contains(&name))
            {
                let span_start = namespace
                    .as_ref()
                    .map(|namespace| name_start + namespace.len() + 1)
                    .unwrap_or(name_start);
                calls.push(SassNameSpan {
                    name,
                    namespace,
                    byte_span: ParserByteSpanV0 {
                        start: base_start + span_start,
                        end: base_start + name_end,
                    },
                });
            }
        }
    }

    calls
}

fn sass_module_qualified_namespace_before(raw: &str, start: usize) -> Option<String> {
    if start <= 1 || !raw[..start].ends_with('.') {
        return None;
    }
    let namespace = raw[..start - 1]
        .rsplit(|ch: char| !is_sass_ident_continue(ch))
        .next()
        .unwrap_or("");
    is_valid_sass_namespace(namespace).then(|| namespace.to_string())
}

fn find_identifier_matches(raw: &str, known_names: &BTreeSet<String>) -> Vec<String> {
    let mut matches = Vec::new();
    let chars: Vec<char> = raw.chars().collect();
    let mut index = 0usize;
    let mut quote: Option<char> = None;
    let mut identifier_start: Option<usize> = None;

    let flush_identifier =
        |end: usize, identifier_start: &mut Option<usize>, matches: &mut Vec<String>| {
            if let Some(start) = *identifier_start {
                let candidate: String = chars[start..end].iter().collect();
                if known_names.contains(&candidate) {
                    matches.push(candidate);
                }
                *identifier_start = None;
            }
        };

    while index < chars.len() {
        let ch = chars[index];
        if let Some(active_quote) = quote {
            if ch == '\\' && index + 1 < chars.len() {
                index += 2;
                continue;
            }
            if ch == active_quote {
                quote = None;
            }
            index += 1;
            continue;
        }

        if ch == '"' || ch == '\'' {
            flush_identifier(index, &mut identifier_start, &mut matches);
            quote = Some(ch);
            index += 1;
            continue;
        }

        if is_value_ident_continue(ch) {
            if identifier_start.is_none() {
                identifier_start = Some(index);
            }
            index += 1;
            continue;
        }

        flush_identifier(index, &mut identifier_start, &mut matches);
        index += 1;
    }

    flush_identifier(chars.len(), &mut identifier_start, &mut matches);
    matches
}

fn is_value_ident_continue(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-' | '$')
}

fn find_css_var_ref_names(raw: &str) -> Vec<String> {
    let mut refs = Vec::new();
    let mut search_start = 0usize;

    while let Some(relative_start) = raw[search_start..].find("var(") {
        let function_start = search_start + relative_start;
        if !is_css_function_boundary(raw, function_start) {
            search_start = function_start + "var(".len();
            continue;
        }

        let mut cursor = function_start + "var(".len();
        while cursor < raw.len() {
            let Some(ch) = raw[cursor..].chars().next() else {
                break;
            };
            if !ch.is_whitespace() {
                break;
            }
            cursor += ch.len_utf8();
        }

        if !raw[cursor..].starts_with("--") {
            search_start = cursor;
            continue;
        }

        let name_start = cursor;
        let mut name_end = cursor;
        for (relative_index, ch) in raw[name_start..].char_indices() {
            let absolute_index = name_start + relative_index;
            if relative_index == 0 {
                name_end = absolute_index + ch.len_utf8();
                continue;
            }
            if !is_css_custom_property_ident_continue(ch) {
                break;
            }
            name_end = absolute_index + ch.len_utf8();
        }

        let candidate = &raw[name_start..name_end];
        if is_css_custom_property_name(candidate) {
            refs.push(candidate.to_string());
        }
        search_start = name_end.max(function_start + "var(".len());
    }

    refs
}

fn is_css_function_boundary(raw: &str, function_start: usize) -> bool {
    raw[..function_start]
        .chars()
        .next_back()
        .is_none_or(|ch| !ch.is_ascii_alphanumeric() && ch != '_')
}

fn is_css_custom_property_name(name: &str) -> bool {
    let Some(rest) = name.strip_prefix("--") else {
        return false;
    };
    let mut chars = rest.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    is_css_custom_property_ident_start(first) && chars.all(is_css_custom_property_ident_continue)
}

fn is_css_custom_property_ident_start(ch: char) -> bool {
    ch == '_' || ch == '-' || ch.is_ascii_alphabetic() || !ch.is_ascii()
}

fn is_css_custom_property_ident_continue(ch: char) -> bool {
    is_css_custom_property_ident_start(ch) || ch.is_ascii_digit()
}

fn is_sass_ident_start(ch: char) -> bool {
    ch.is_ascii_alphabetic() || ch == '_' || ch == '-' || !ch.is_ascii()
}

fn is_sass_ident_continue(ch: char) -> bool {
    is_sass_ident_start(ch) || ch.is_ascii_digit()
}

fn extract_simple_selector_name(prelude: &str) -> Option<String> {
    let trimmed = prelude.trim();
    let rest = trimmed.strip_prefix('.')?;
    if rest.is_empty() {
        return None;
    }
    if rest.contains([' ', ',', ':', '&', '#', '[', '>', '+', '~']) {
        return None;
    }
    Some(rest.to_string())
}

fn resolve_rule_selector_branches(
    rule: &RulePayload,
    parent_branches: &[ResolvedSelectorBranch],
) -> Vec<ResolvedSelectorBranch> {
    let mut branches = Vec::new();

    for group in &rule.selector_groups {
        let resolved = extract_group_selector_branches(group, parent_branches);
        if !resolved.is_empty() {
            branches.extend(resolved);
        } else if let Some(local_names) = extract_local_function_selector_names(&group.raw) {
            branches.extend(repeat_names_for_parent_branches(
                local_names,
                parent_branches,
                false,
            ));
        } else if let Some(name) = extract_simple_selector_name(&group.raw) {
            branches.extend(repeat_names_for_parent_branches(
                vec![name],
                parent_branches,
                true,
            ));
        }
    }

    branches
}

fn repeat_names_for_parent_branches(
    names: Vec<String>,
    parent_branches: &[ResolvedSelectorBranch],
    bare_when_top_level_single: bool,
) -> Vec<ResolvedSelectorBranch> {
    if names.is_empty() {
        return Vec::new();
    }
    if parent_branches.is_empty() {
        let bare_suffix_base = bare_when_top_level_single && names.len() == 1;
        return names
            .into_iter()
            .map(|name| ResolvedSelectorBranch {
                name,
                bare_suffix_base,
            })
            .collect();
    }
    if parent_branches.len() == 1 {
        return names
            .into_iter()
            .map(|name| ResolvedSelectorBranch {
                name,
                bare_suffix_base: false,
            })
            .collect();
    }
    let mut repeated = Vec::with_capacity(names.len() * parent_branches.len());
    for _ in parent_branches {
        repeated.extend(names.iter().cloned().map(|name| ResolvedSelectorBranch {
            name,
            bare_suffix_base: false,
        }));
    }
    repeated
}

fn extract_group_selector_branches(
    group: &SelectorGroup,
    parent_branches: &[ResolvedSelectorBranch],
) -> Vec<ResolvedSelectorBranch> {
    match group.segments.as_slice() {
        [SelectorSegment::ClassName(name)] => {
            repeat_names_for_parent_branches(vec![name.clone()], parent_branches, true)
        }
        [
            SelectorSegment::Ampersand,
            SelectorSegment::BemSuffix(suffix),
        ] => parent_branches
            .iter()
            .filter(|parent| parent.bare_suffix_base)
            .map(|parent| ResolvedSelectorBranch {
                name: format!("{}{}", parent.name, suffix),
                bare_suffix_base: true,
            })
            .collect(),
        [
            SelectorSegment::Ampersand,
            SelectorSegment::AmpersandSuffix(suffix),
        ] => parent_branches
            .iter()
            .filter(|parent| parent.bare_suffix_base)
            .map(|parent| ResolvedSelectorBranch {
                name: format!("{}{}", parent.name, suffix),
                bare_suffix_base: true,
            })
            .collect(),
        [SelectorSegment::Ampersand, SelectorSegment::ClassName(name)] => {
            repeat_names_for_parent_branches(vec![name.clone()], parent_branches, false)
        }
        segments => {
            let last_combinator = segments
                .iter()
                .enumerate()
                .rev()
                .find_map(|(index, segment)| {
                    matches!(segment, SelectorSegment::Combinator(_)).then_some(index)
                });
            let tail = last_combinator
                .map(|index| &segments[index + 1..])
                .unwrap_or(segments);
            if let [
                SelectorSegment::Ampersand,
                SelectorSegment::BemSuffix(suffix) | SelectorSegment::AmpersandSuffix(suffix),
            ] = tail
            {
                return parent_branches
                    .iter()
                    .filter(|parent| parent.bare_suffix_base)
                    .map(|parent| ResolvedSelectorBranch {
                        name: format!("{}{}", parent.name, suffix),
                        bare_suffix_base: false,
                    })
                    .collect();
            }
            let names: Vec<String> = tail
                .iter()
                .filter_map(|segment| match segment {
                    SelectorSegment::ClassName(name) => Some(name.clone()),
                    _ => None,
                })
                .collect();
            repeat_names_for_parent_branches(names, parent_branches, false)
        }
    }
}

fn extract_local_function_selector_names(raw: &str) -> Option<Vec<String>> {
    let trimmed = raw.trim();
    let inner = trimmed
        .strip_prefix(":local(")
        .and_then(|rest| rest.strip_suffix(')'))?;
    let mut names = Vec::new();
    let chars: Vec<char> = inner.chars().collect();
    let mut index = 0usize;

    while index < chars.len() {
        if chars[index] == '.' {
            index += 1;
            let start = index;
            while index < chars.len() && is_selector_ident_continue(chars[index]) {
                index += 1;
            }
            if start < index {
                names.push(chars[start..index].iter().collect());
                continue;
            }
        }
        index += 1;
    }

    (!names.is_empty()).then_some(names)
}

fn parse_selector_groups(prelude: &str) -> Vec<SelectorGroup> {
    let mut groups = Vec::new();
    let mut depth_paren = 0usize;
    let mut depth_bracket = 0usize;
    let mut start = 0usize;

    for (index, ch) in prelude.char_indices() {
        match ch {
            '(' => depth_paren += 1,
            ')' => depth_paren = depth_paren.saturating_sub(1),
            '[' => depth_bracket += 1,
            ']' => depth_bracket = depth_bracket.saturating_sub(1),
            ',' if depth_paren == 0 && depth_bracket == 0 => {
                let raw = prelude[start..index].trim();
                if !raw.is_empty() {
                    groups.push(SelectorGroup {
                        raw: raw.to_string(),
                        segments: parse_selector_segments(raw),
                    });
                }
                start = index + ch.len_utf8();
            }
            _ => {}
        }
    }

    let raw = prelude[start..].trim();
    if !raw.is_empty() {
        groups.push(SelectorGroup {
            raw: raw.to_string(),
            segments: parse_selector_segments(raw),
        });
    }

    groups
}

fn parse_selector_segments(raw: &str) -> Vec<SelectorSegment> {
    let chars: Vec<char> = raw.chars().collect();
    let mut index = 0usize;
    let mut segments = Vec::new();

    while index < chars.len() {
        match chars[index] {
            c if c.is_whitespace() => {
                let start = index;
                while index < chars.len() && chars[index].is_whitespace() {
                    index += 1;
                }
                let next = chars.get(index).copied();
                let has_prev = segments.last().is_some();
                let prev_is_combinator =
                    matches!(segments.last(), Some(SelectorSegment::Combinator(_)));
                let next_starts_selector = next.is_some_and(|ch| {
                    matches!(ch, '.' | '&' | ':' | '#' | '*' | '[') || ch.is_ascii_alphabetic()
                });
                if start > 0 && has_prev && !prev_is_combinator && next_starts_selector {
                    segments.push(SelectorSegment::Combinator(" ".to_string()));
                }
            }
            '.' => {
                index += 1;
                let start = index;
                while index < chars.len() && is_selector_ident_continue(chars[index]) {
                    index += 1;
                }
                if start < index {
                    segments.push(SelectorSegment::ClassName(
                        chars[start..index].iter().collect(),
                    ));
                } else {
                    segments.push(SelectorSegment::Other(".".to_string()));
                }
            }
            '&' => {
                segments.push(SelectorSegment::Ampersand);
                index += 1;
                if index + 1 < chars.len()
                    && ((chars[index] == '-' && chars[index + 1] == '-')
                        || (chars[index] == '_' && chars[index + 1] == '_'))
                {
                    let start = index;
                    index += 2;
                    while index < chars.len() && is_selector_ident_continue(chars[index]) {
                        index += 1;
                    }
                    segments.push(SelectorSegment::BemSuffix(
                        chars[start..index].iter().collect(),
                    ));
                } else if index < chars.len()
                    && !starts_selector_component_after_ampersand(chars[index])
                {
                    let start = index;
                    while index < chars.len() && is_selector_ident_continue(chars[index]) {
                        index += 1;
                    }
                    if start < index {
                        segments.push(SelectorSegment::AmpersandSuffix(
                            chars[start..index].iter().collect(),
                        ));
                    }
                }
            }
            ':' => {
                index += 1;
                let start = index;
                while index < chars.len() && is_selector_ident_continue(chars[index]) {
                    index += 1;
                }
                segments.push(SelectorSegment::Pseudo(
                    chars[start..index].iter().collect(),
                ));
                if index < chars.len() && chars[index] == '(' {
                    let mut depth = 1usize;
                    index += 1;
                    while index < chars.len() && depth > 0 {
                        match chars[index] {
                            '(' => depth += 1,
                            ')' => depth = depth.saturating_sub(1),
                            '\'' | '"' => {
                                let quote = chars[index];
                                index += 1;
                                while index < chars.len() {
                                    if chars[index] == '\\' {
                                        index += 2;
                                        continue;
                                    }
                                    if chars[index] == quote {
                                        break;
                                    }
                                    index += 1;
                                }
                            }
                            _ => {}
                        }
                        index += 1;
                    }
                }
            }
            '>' | '+' | '~' => {
                segments.push(SelectorSegment::Combinator(chars[index].to_string()));
                index += 1;
            }
            _ => {
                let start = index;
                index += 1;
                while index < chars.len()
                    && !chars[index].is_whitespace()
                    && !matches!(chars[index], '.' | '&' | ':' | '>' | '+' | '~' | ',')
                {
                    index += 1;
                }
                segments.push(SelectorSegment::Other(chars[start..index].iter().collect()));
            }
        }
    }

    segments
}

fn is_selector_ident_continue(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-') || !ch.is_ascii()
}

fn starts_selector_component_after_ampersand(ch: char) -> bool {
    matches!(
        ch,
        ':' | '.' | '[' | '#' | '>' | '+' | '~' | ',' | '*' | ')' | '}' | ']'
    ) || ch.is_whitespace()
}

fn tokenize(language: StyleLanguage, source: &str) -> (Vec<Token>, Vec<ParseDiagnostic>) {
    let mut tokens = Vec::new();
    let mut diagnostics = Vec::new();
    let bytes = source.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        let start = i;
        let byte = bytes[i];

        if byte.is_ascii_whitespace() {
            i += 1;
            while i < bytes.len() && bytes[i].is_ascii_whitespace() {
                i += 1;
            }
            tokens.push(Token {
                kind: TokenKind::Whitespace,
                span: TextSpan::new(start, i),
            });
            continue;
        }

        if language.supports_line_comments() && byte == b'/' && bytes.get(i + 1) == Some(&b'/') {
            i += 2;
            while i < bytes.len() && bytes[i] != b'\n' {
                i += 1;
            }
            tokens.push(Token {
                kind: TokenKind::LineComment,
                span: TextSpan::new(start, i),
            });
            continue;
        }

        if byte == b'/' && bytes.get(i + 1) == Some(&b'*') {
            i += 2;
            let mut closed = false;
            while i + 1 < bytes.len() {
                if bytes[i] == b'*' && bytes[i + 1] == b'/' {
                    i += 2;
                    closed = true;
                    break;
                }
                i += 1;
            }
            if !closed {
                i = bytes.len();
                diagnostics.push(ParseDiagnostic {
                    message: "unterminated block comment".to_string(),
                    span: TextSpan::new(start, i),
                });
            }
            tokens.push(Token {
                kind: TokenKind::BlockComment,
                span: TextSpan::new(start, i),
            });
            continue;
        }

        if byte == b'"' || byte == b'\'' {
            let quote = byte;
            i += 1;
            let mut closed = false;
            while i < bytes.len() {
                if bytes[i] == b'\\' {
                    i = (i + 2).min(bytes.len());
                    continue;
                }
                if bytes[i] == quote {
                    i += 1;
                    closed = true;
                    break;
                }
                i += 1;
            }
            if !closed {
                diagnostics.push(ParseDiagnostic {
                    message: "unterminated string literal".to_string(),
                    span: TextSpan::new(start, i),
                });
            }
            tokens.push(Token {
                kind: TokenKind::String,
                span: TextSpan::new(start, i),
            });
            continue;
        }

        if byte == b'#' && bytes.get(i + 1) == Some(&b'{') {
            i += 2;
            tokens.push(Token {
                kind: TokenKind::InterpolationStart,
                span: TextSpan::new(start, i),
            });
            continue;
        }

        if is_ident_start(byte) {
            i += 1;
            while i < bytes.len() && is_ident_continue(bytes[i]) {
                i += 1;
            }
            tokens.push(Token {
                kind: TokenKind::Ident,
                span: TextSpan::new(start, i),
            });
            continue;
        }

        if byte.is_ascii_digit() {
            i += 1;
            while i < bytes.len() && bytes[i].is_ascii_digit() {
                i += 1;
            }
            tokens.push(Token {
                kind: TokenKind::Number,
                span: TextSpan::new(start, i),
            });
            continue;
        }

        let kind = match byte {
            b'.' => TokenKind::Dot,
            b'&' => TokenKind::Ampersand,
            b'#' => TokenKind::Hash,
            b':' => TokenKind::Colon,
            b';' => TokenKind::Semicolon,
            b',' => TokenKind::Comma,
            b'@' => TokenKind::At,
            b'{' => TokenKind::OpenBrace,
            b'}' => TokenKind::CloseBrace,
            b'(' => TokenKind::OpenParen,
            b')' => TokenKind::CloseParen,
            b'[' => TokenKind::OpenBracket,
            b']' => TokenKind::CloseBracket,
            _ => TokenKind::Other,
        };
        i += 1;
        tokens.push(Token {
            kind,
            span: TextSpan::new(start, i),
        });
    }

    (tokens, diagnostics)
}

fn is_ident_start(byte: u8) -> bool {
    byte.is_ascii_alphabetic() || matches!(byte, b'_' | b'-') || byte >= 0x80
}

fn is_ident_continue(byte: u8) -> bool {
    is_ident_start(byte) || byte.is_ascii_digit()
}

struct Parser<'a> {
    source: &'a str,
    tokens: &'a [Token],
    diagnostics: &'a mut Vec<ParseDiagnostic>,
    cursor: usize,
}

impl<'a> Parser<'a> {
    fn new(
        source: &'a str,
        tokens: &'a [Token],
        diagnostics: &'a mut Vec<ParseDiagnostic>,
    ) -> Self {
        Self {
            source,
            tokens,
            diagnostics,
            cursor: 0,
        }
    }

    fn parse_root(&mut self) -> Vec<SyntaxNode> {
        self.parse_block(false)
    }

    fn parse_block(&mut self, stop_at_close_brace: bool) -> Vec<SyntaxNode> {
        let mut nodes = Vec::new();

        while self.cursor < self.tokens.len() {
            let token = &self.tokens[self.cursor];

            match token.kind {
                TokenKind::Whitespace => {
                    self.cursor += 1;
                }
                TokenKind::LineComment | TokenKind::BlockComment => {
                    nodes.push(SyntaxNode {
                        kind: SyntaxNodeKind::Comment,
                        span: token.span,
                        header_span: None,
                        payload: Some(SyntaxNodePayload::Comment(CommentPayload {
                            text: self.slice(token.span).to_string(),
                        })),
                        children: Vec::new(),
                    });
                    self.cursor += 1;
                }
                TokenKind::CloseBrace if stop_at_close_brace => {
                    self.cursor += 1;
                    return nodes;
                }
                TokenKind::CloseBrace => {
                    self.diagnostics.push(ParseDiagnostic {
                        message: "unexpected closing brace".to_string(),
                        span: token.span,
                    });
                    self.cursor += 1;
                }
                _ => nodes.push(self.parse_statement()),
            }
        }

        if stop_at_close_brace {
            let end = self.tokens.last().map_or(0, |token| token.span.end);
            self.diagnostics.push(ParseDiagnostic {
                message: "unterminated block".to_string(),
                span: TextSpan::new(end, end),
            });
        }

        nodes
    }

    fn parse_statement(&mut self) -> SyntaxNode {
        let start_index = self.cursor;
        let mut index = self.cursor;
        let mut saw_at = self.tokens[index].kind == TokenKind::At;
        let mut saw_colon = false;
        let mut first_colon_index = None;
        let mut paren_depth = 0usize;
        let mut bracket_depth = 0usize;

        while index < self.tokens.len() {
            let token = &self.tokens[index];
            match token.kind {
                TokenKind::OpenParen => paren_depth += 1,
                TokenKind::CloseParen => paren_depth = paren_depth.saturating_sub(1),
                TokenKind::OpenBracket => bracket_depth += 1,
                TokenKind::CloseBracket => bracket_depth = bracket_depth.saturating_sub(1),
                TokenKind::Colon if paren_depth == 0 && bracket_depth == 0 => {
                    saw_colon = true;
                    if first_colon_index.is_none() {
                        first_colon_index = Some(index);
                    }
                }
                TokenKind::At if index == start_index => saw_at = true,
                TokenKind::Semicolon if paren_depth == 0 && bracket_depth == 0 => {
                    let span = TextSpan::new(
                        self.tokens[start_index].span.start,
                        self.tokens[index].span.end,
                    );
                    self.cursor = index + 1;
                    return SyntaxNode {
                        kind: classify_statement_kind(saw_at, saw_colon),
                        span,
                        header_span: Some(TextSpan::new(
                            self.tokens[start_index].span.start,
                            self.tokens[index].span.start,
                        )),
                        payload: self.build_inline_payload(
                            start_index,
                            index,
                            saw_at,
                            first_colon_index,
                        ),
                        children: Vec::new(),
                    };
                }
                TokenKind::OpenBrace if paren_depth == 0 && bracket_depth == 0 => {
                    let header_span = TextSpan::new(
                        self.tokens[start_index].span.start,
                        self.tokens[index].span.start,
                    );
                    self.cursor = index + 1;
                    let children = self.parse_block(true);
                    let end = self
                        .tokens
                        .get(self.cursor.saturating_sub(1))
                        .map_or(self.tokens[index].span.end, |token| token.span.end);
                    return SyntaxNode {
                        kind: if saw_at {
                            SyntaxNodeKind::AtRule
                        } else {
                            SyntaxNodeKind::Rule
                        },
                        span: TextSpan::new(self.tokens[start_index].span.start, end),
                        header_span: Some(header_span),
                        payload: Some(if saw_at {
                            SyntaxNodePayload::AtRule(
                                self.build_at_rule_payload(start_index, index),
                            )
                        } else {
                            let prelude = self.slice_trimmed(header_span).to_string();
                            SyntaxNodePayload::Rule(RulePayload {
                                selector_groups: parse_selector_groups(&prelude),
                                prelude,
                            })
                        }),
                        children,
                    };
                }
                TokenKind::CloseBrace => break,
                _ => {}
            }
            index += 1;
        }

        let end = self
            .tokens
            .get(index.saturating_sub(1))
            .map_or(self.tokens[start_index].span.end, |token| token.span.end);
        self.cursor = index.max(start_index + 1);
        let span = TextSpan::new(self.tokens[start_index].span.start, end);
        SyntaxNode {
            kind: classify_statement_kind(saw_at, saw_colon),
            span,
            header_span: Some(span),
            payload: self.build_inline_payload(start_index, index, saw_at, first_colon_index),
            children: Vec::new(),
        }
    }

    fn build_inline_payload(
        &self,
        start_index: usize,
        end_index: usize,
        saw_at: bool,
        first_colon_index: Option<usize>,
    ) -> Option<SyntaxNodePayload> {
        if saw_at {
            return Some(SyntaxNodePayload::AtRule(
                self.build_at_rule_payload(start_index, end_index),
            ));
        }

        let colon_index = first_colon_index?;
        let property_span = TextSpan::new(
            self.tokens[start_index].span.start,
            self.tokens[colon_index].span.start,
        );
        let value_start = self.tokens[colon_index].span.end;
        let value_end = self
            .tokens
            .get(end_index.saturating_sub(1))
            .map_or(value_start, |token| token.span.end);
        Some(SyntaxNodePayload::Declaration(DeclarationPayload {
            property: self.slice_trimmed(property_span).to_string(),
            value: self
                .slice_trimmed(TextSpan::new(value_start, value_end))
                .to_string(),
        }))
    }

    fn build_at_rule_payload(&self, start_index: usize, end_index: usize) -> AtRulePayload {
        let name = self
            .tokens
            .get(start_index + 1)
            .map(|token| self.slice(token.span))
            .unwrap_or_default()
            .trim()
            .to_string();
        let params_start = self
            .tokens
            .get(start_index + 2)
            .map_or(self.tokens[start_index].span.end, |token| token.span.start);
        let params_end = self
            .tokens
            .get(end_index.saturating_sub(1))
            .map_or(params_start, |token| token.span.end);
        AtRulePayload {
            kind: classify_at_rule_kind(&name),
            name,
            params: self
                .slice_trimmed(TextSpan::new(params_start, params_end))
                .to_string(),
        }
    }

    fn slice(&self, span: TextSpan) -> &'a str {
        &self.source[span.start..span.end]
    }

    fn slice_trimmed(&self, span: TextSpan) -> &'a str {
        self.slice(span).trim()
    }
}

fn classify_statement_kind(saw_at: bool, saw_colon: bool) -> SyntaxNodeKind {
    if saw_at {
        SyntaxNodeKind::AtRule
    } else if saw_colon {
        SyntaxNodeKind::Declaration
    } else {
        SyntaxNodeKind::Unknown
    }
}

fn classify_at_rule_kind(name: &str) -> AtRuleKind {
    match name {
        "media" => AtRuleKind::Media,
        "supports" => AtRuleKind::Supports,
        "layer" => AtRuleKind::Layer,
        "keyframes" | "-webkit-keyframes" => AtRuleKind::Keyframes,
        "value" => AtRuleKind::Value,
        "at-root" => AtRuleKind::AtRoot,
        "mixin" => AtRuleKind::Mixin,
        "include" => AtRuleKind::Include,
        "function" => AtRuleKind::Function,
        "use" => AtRuleKind::Use,
        "forward" => AtRuleKind::Forward,
        "import" => AtRuleKind::Import,
        _ => AtRuleKind::Generic,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        AtRuleKind, AtRulePayload, DeclarationPayload, ParserByteSpanV0,
        ParserIndexSassModuleUseFactV0, ParserIndexSassSelectorSymbolFactV0, ParserPositionV0,
        ParserRangeV0, RulePayload, SelectorGroup, SelectorSegment, StyleLanguage, SyntaxNodeKind,
        SyntaxNodePayload, TextSpan, TokenKind, parse_stylesheet,
    };

    fn token_texts<'a>(source: &'a str, sheet: &super::Stylesheet) -> Vec<(TokenKind, &'a str)> {
        sheet
            .tokens
            .iter()
            .map(|token| (token.kind, &source[token.span.start..token.span.end]))
            .collect()
    }

    fn span_after(source: &str, anchor: &str, needle: &str) -> Result<ParserByteSpanV0, String> {
        let anchor_start = source
            .find(anchor)
            .ok_or_else(|| format!("missing anchor {anchor:?}"))?;
        let relative_start = source[anchor_start..]
            .find(needle)
            .ok_or_else(|| format!("missing needle {needle:?} after {anchor:?}"))?;
        let start = anchor_start + relative_start;
        Ok(ParserByteSpanV0 {
            start,
            end: start + needle.len(),
        })
    }

    fn range_after(source: &str, anchor: &str, needle: &str) -> Result<ParserRangeV0, String> {
        span_after(source, anchor, needle)
            .map(|span| super::source_range_for_byte_span(source, span))
    }

    #[test]
    fn detects_style_language_from_module_path() {
        assert_eq!(
            StyleLanguage::from_module_path("/x/Button.module.css"),
            Some(StyleLanguage::Css)
        );
        assert_eq!(
            StyleLanguage::from_module_path("/x/Button.module.scss"),
            Some(StyleLanguage::Scss)
        );
        assert_eq!(
            StyleLanguage::from_module_path("/x/Button.module.less"),
            Some(StyleLanguage::Less)
        );
        assert_eq!(
            StyleLanguage::from_module_path("/x/Button.css"),
            Some(StyleLanguage::Css)
        );
        assert_eq!(
            StyleLanguage::from_module_path("/x/_tokens.scss"),
            Some(StyleLanguage::Scss)
        );
        assert_eq!(
            StyleLanguage::from_module_path("/x/theme.less"),
            Some(StyleLanguage::Less)
        );
    }

    #[test]
    fn tokenizes_basic_css_rule() {
        let source = ".button { color: red; }";
        let sheet = parse_stylesheet(StyleLanguage::Css, source);
        let tokens = token_texts(source, &sheet);
        assert!(tokens.contains(&(TokenKind::Dot, ".")));
        assert!(tokens.contains(&(TokenKind::Ident, "button")));
        assert!(tokens.contains(&(TokenKind::OpenBrace, "{")));
        assert!(tokens.contains(&(TokenKind::Semicolon, ";")));
        assert!(sheet.diagnostics.is_empty());
    }

    #[test]
    fn keeps_css_double_slash_as_regular_tokens() {
        let source = ".button { // not-a-comment\n color: red; }";
        let sheet = parse_stylesheet(StyleLanguage::Css, source);
        assert!(
            !sheet
                .tokens
                .iter()
                .any(|token| matches!(token.kind, TokenKind::LineComment))
        );
    }

    #[test]
    fn parses_scss_nested_rules_and_comments() {
        let source = ".button {\n  // note\n  &--primary { color: red; }\n}\n";
        let sheet = parse_stylesheet(StyleLanguage::Scss, source);
        assert_eq!(sheet.nodes.len(), 1);
        let root_rule = &sheet.nodes[0];
        assert_eq!(root_rule.kind, SyntaxNodeKind::Rule);
        assert_eq!(
            root_rule.payload,
            Some(SyntaxNodePayload::Rule(RulePayload {
                prelude: ".button".to_string(),
                selector_groups: vec![SelectorGroup {
                    raw: ".button".to_string(),
                    segments: vec![SelectorSegment::ClassName("button".to_string())],
                }],
            }))
        );
        assert_eq!(root_rule.children.len(), 2);
        assert_eq!(root_rule.children[0].kind, SyntaxNodeKind::Comment);
        assert_eq!(
            root_rule.children[0].payload,
            Some(SyntaxNodePayload::Comment(super::CommentPayload {
                text: "// note".to_string(),
            }))
        );
        assert_eq!(root_rule.children[1].kind, SyntaxNodeKind::Rule);
        assert_eq!(
            root_rule.children[1].payload,
            Some(SyntaxNodePayload::Rule(RulePayload {
                prelude: "&--primary".to_string(),
                selector_groups: vec![SelectorGroup {
                    raw: "&--primary".to_string(),
                    segments: vec![
                        SelectorSegment::Ampersand,
                        SelectorSegment::BemSuffix("--primary".to_string()),
                    ],
                }],
            }))
        );
        assert_eq!(
            root_rule.children[1].children[0].payload,
            Some(SyntaxNodePayload::Declaration(DeclarationPayload {
                property: "color".to_string(),
                value: "red".to_string(),
            }))
        );
        assert!(sheet.diagnostics.is_empty());
    }

    #[test]
    fn parses_less_at_rule_like_variable_assignment() {
        let source = "@color: red;\n.button { color: @color; }";
        let sheet = parse_stylesheet(StyleLanguage::Less, source);
        assert_eq!(sheet.nodes[0].kind, SyntaxNodeKind::AtRule);
        assert_eq!(
            sheet.nodes[0].payload,
            Some(SyntaxNodePayload::AtRule(AtRulePayload {
                kind: AtRuleKind::Generic,
                name: "color".to_string(),
                params: ": red".to_string(),
            }))
        );
        assert_eq!(sheet.nodes[1].kind, SyntaxNodeKind::Rule);
    }

    #[test]
    fn parses_at_rule_header_and_params() {
        let source = "@media screen and (min-width: 10px) { .button { color: red; } }";
        let sheet = parse_stylesheet(StyleLanguage::Css, source);
        assert_eq!(
            sheet.nodes[0].payload,
            Some(SyntaxNodePayload::AtRule(AtRulePayload {
                kind: AtRuleKind::Media,
                name: "media".to_string(),
                params: "screen and (min-width: 10px)".to_string(),
            }))
        );
    }

    #[test]
    fn classifies_keyframes_and_value_at_rules() {
        let source = "@value brand: red;\n@keyframes fade { from { opacity: 0; } }\n";
        let sheet = parse_stylesheet(StyleLanguage::Scss, source);
        assert_eq!(
            sheet.nodes[0].payload,
            Some(SyntaxNodePayload::AtRule(AtRulePayload {
                kind: AtRuleKind::Value,
                name: "value".to_string(),
                params: "brand: red".to_string(),
            }))
        );
        assert_eq!(
            sheet.nodes[1].payload,
            Some(SyntaxNodePayload::AtRule(AtRulePayload {
                kind: AtRuleKind::Keyframes,
                name: "keyframes".to_string(),
                params: "fade".to_string(),
            }))
        );
    }

    #[test]
    fn classifies_sass_symbol_at_rules() {
        let source = r#"@use "./tokens" as tokens;
@forward "./theme";
@import "./legacy";
@mixin raised($depth) { color: red; }
@include raised($gap);
@function tone($value) { @return $value; }
"#;
        let sheet = parse_stylesheet(StyleLanguage::Scss, source);
        let kinds: Vec<AtRuleKind> = sheet
            .nodes
            .iter()
            .filter_map(|node| match &node.payload {
                Some(SyntaxNodePayload::AtRule(at_rule)) => Some(at_rule.kind),
                _ => None,
            })
            .collect();
        assert_eq!(
            kinds,
            vec![
                AtRuleKind::Use,
                AtRuleKind::Forward,
                AtRuleKind::Import,
                AtRuleKind::Mixin,
                AtRuleKind::Include,
                AtRuleKind::Function,
            ]
        );
    }

    #[test]
    fn records_unterminated_block_comment_diagnostic() {
        let source = "/* open";
        let sheet = parse_stylesheet(StyleLanguage::Scss, source);
        assert_eq!(sheet.diagnostics.len(), 1);
        assert_eq!(sheet.diagnostics[0].message, "unterminated block comment");
        assert_eq!(sheet.diagnostics[0].span, TextSpan::new(0, source.len()));
    }

    #[test]
    fn records_unterminated_block_diagnostic() {
        let source = ".button { color: red;";
        let sheet = parse_stylesheet(StyleLanguage::Scss, source);
        assert_eq!(sheet.nodes.len(), 1);
        assert_eq!(sheet.nodes[0].kind, SyntaxNodeKind::Rule);
        assert!(
            sheet
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.message == "unterminated block")
        );
    }

    #[test]
    fn splits_grouped_selectors_into_groups_and_segments() {
        let source = ".a, .b { &--c { color: red; } }";
        let sheet = parse_stylesheet(StyleLanguage::Scss, source);
        assert_eq!(
            sheet.nodes[0].payload,
            Some(SyntaxNodePayload::Rule(RulePayload {
                prelude: ".a, .b".to_string(),
                selector_groups: vec![
                    SelectorGroup {
                        raw: ".a".to_string(),
                        segments: vec![SelectorSegment::ClassName("a".to_string())],
                    },
                    SelectorGroup {
                        raw: ".b".to_string(),
                        segments: vec![SelectorSegment::ClassName("b".to_string())],
                    },
                ],
            }))
        );
    }

    #[test]
    fn parity_summary_reconstructs_bem_suffix_names() {
        let source = ".card { &__icon { &--small { color: red; } } }";
        let sheet = parse_stylesheet(StyleLanguage::Scss, source);
        let summary = super::summarize_parity_lite(&sheet);
        assert_eq!(
            summary.selector_names,
            vec!["card", "card__icon", "card__icon--small"]
        );
    }

    #[test]
    fn parity_summary_expands_grouped_parent_bem_suffixes() {
        let source = ".a, .b { &--c { color: red; } }";
        let sheet = parse_stylesheet(StyleLanguage::Scss, source);
        let summary = super::summarize_parity_lite(&sheet);
        assert_eq!(summary.selector_names, vec!["a", "a--c", "b", "b--c"]);
    }

    #[test]
    fn parity_summary_expands_grouped_parent_nested_bem_suffixes() {
        let source = ".a, .b { &__icon { &--small { color: red; } } }";
        let sheet = parse_stylesheet(StyleLanguage::Scss, source);
        let summary = super::summarize_parity_lite(&sheet);
        assert_eq!(
            summary.selector_names,
            vec![
                "a",
                "a__icon",
                "a__icon--small",
                "b",
                "b__icon",
                "b__icon--small"
            ]
        );
    }

    #[test]
    fn parity_summary_keeps_ampersand_class_as_standalone_class() {
        let source = ".a, .b { &.active { color: red; } }";
        let sheet = parse_stylesheet(StyleLanguage::Scss, source);
        let summary = super::summarize_parity_lite(&sheet);
        assert_eq!(summary.selector_names, vec!["a", "active", "active", "b"]);
    }

    #[test]
    fn index_summary_matches_nested_bem_safety_matrix() {
        let source = ".btn { &--primary {} &__icon {} &.active {} &-legacy {} &_legacy {} & + &--paired {} }";
        let sheet = parse_stylesheet(StyleLanguage::Scss, source);
        let summary = super::summarize_css_modules_intermediate(&sheet);
        assert_eq!(
            summary.selectors.names,
            vec![
                "active",
                "btn",
                "btn--paired",
                "btn--primary",
                "btn-legacy",
                "btn__icon",
                "btn_legacy"
            ]
        );
        assert_eq!(
            summary.selectors.bem_suffix_safe_names,
            vec!["btn--primary", "btn__icon"]
        );
        assert_eq!(
            summary.selectors.nested_unsafe_names,
            vec!["active", "btn--paired", "btn-legacy", "btn_legacy"]
        );
    }

    #[test]
    fn index_summary_does_not_bem_expand_non_bare_parent_suffixes() {
        let source = ".card:hover { &--primary {} }";
        let sheet = parse_stylesheet(StyleLanguage::Scss, source);
        let summary = super::summarize_css_modules_intermediate(&sheet);
        assert_eq!(summary.selectors.names, vec!["card"]);
        assert_eq!(summary.selectors.bem_suffix_count, 0);
    }

    #[test]
    fn index_summary_marks_plain_nested_selectors_unsafe() {
        let source = ".wrapper { .inner { &--mod {} } }";
        let sheet = parse_stylesheet(StyleLanguage::Scss, source);
        let summary = super::summarize_css_modules_intermediate(&sheet);
        assert_eq!(summary.selectors.names, vec!["inner", "wrapper"]);
        assert_eq!(summary.selectors.nested_unsafe_names, vec!["inner"]);
        assert_eq!(summary.selectors.nested_safety_counts.flat, 1);
        assert_eq!(summary.selectors.nested_safety_counts.nested_unsafe, 1);
    }

    #[test]
    fn index_summary_allows_unsafe_suffix_to_become_deep_bem_base() {
        let source = ".btn { &-legacy { &--x {} } }";
        let sheet = parse_stylesheet(StyleLanguage::Scss, source);
        let summary = super::summarize_css_modules_intermediate(&sheet);
        assert_eq!(
            summary.selectors.names,
            vec!["btn", "btn-legacy", "btn-legacy--x"]
        );
        assert_eq!(
            summary.selectors.bem_suffix_safe_names,
            vec!["btn-legacy--x"]
        );
        assert_eq!(summary.selectors.nested_unsafe_names, vec!["btn-legacy"]);
    }

    #[test]
    fn parity_summary_keeps_class_from_pseudo_selector() {
        let source = ".btn:hover { color: red; }";
        let sheet = parse_stylesheet(StyleLanguage::Scss, source);
        let summary = super::summarize_parity_lite(&sheet);
        assert_eq!(summary.selector_names, vec!["btn"]);
    }

    #[test]
    fn parity_summary_keeps_multiple_classes_from_compound_selector() {
        let source = ".btn.active { color: red; }";
        let sheet = parse_stylesheet(StyleLanguage::Scss, source);
        let summary = super::summarize_parity_lite(&sheet);
        assert_eq!(summary.selector_names, vec!["active", "btn"]);
    }

    #[test]
    fn parity_summary_prefers_rightmost_class_after_combinator() {
        let source = ".a > .b { color: red; }";
        let sheet = parse_stylesheet(StyleLanguage::Scss, source);
        let summary = super::summarize_parity_lite(&sheet);
        assert_eq!(summary.selector_names, vec!["b"]);
    }

    #[test]
    fn parity_summary_collects_nested_layer_rule_selectors() {
        let source = "@layer ui { .btn:hover { color: red; } }";
        let sheet = parse_stylesheet(StyleLanguage::Scss, source);
        let summary = super::summarize_parity_lite(&sheet);
        assert_eq!(summary.selector_names, vec!["btn"]);
    }

    #[test]
    fn parity_summary_prefers_rightmost_class_after_descendant_combinator() {
        let source = ".a .b { color: red; }";
        let sheet = parse_stylesheet(StyleLanguage::Scss, source);
        let summary = super::summarize_parity_lite(&sheet);
        assert_eq!(summary.selector_names, vec!["b"]);
    }

    #[test]
    fn index_summary_exposes_selector_definition_facts_for_lsp_consumers() -> Result<(), String> {
        let source = ".item { &.compact .body { color: red; } &--primary { color: blue; } }";
        let sheet = parse_stylesheet(StyleLanguage::Scss, source);
        let summary = super::summarize_css_modules_intermediate(&sheet);
        let definition_names: Vec<&str> = summary
            .selectors
            .definition_facts
            .iter()
            .map(|fact| fact.name.as_str())
            .collect();
        assert_eq!(
            definition_names,
            vec!["body", "compact", "item", "item--primary"]
        );
        assert_eq!(
            summary
                .selectors
                .definition_facts
                .iter()
                .find(|fact| fact.name == "compact")
                .map(|fact| fact.range),
            Some(range_after(source, "&.compact", "compact")?)
        );
        assert_eq!(
            summary
                .selectors
                .definition_facts
                .iter()
                .find(|fact| fact.name == "item--primary")
                .map(|fact| (fact.range, fact.bem_suffix_parent_name.as_deref())),
            Some((
                range_after(source, "&--primary", "--primary")?,
                Some("item")
            ))
        );
        Ok(())
    }

    #[test]
    fn parity_summary_ignores_classes_inside_pseudo_functions() {
        let source = ".btn:is(.active, .primary) { color: red; }";
        let sheet = parse_stylesheet(StyleLanguage::Scss, source);
        let summary = super::summarize_parity_lite(&sheet);
        assert_eq!(summary.selector_names, vec!["btn"]);
    }

    #[test]
    fn parity_summary_ignores_global_function_classes() {
        let source = ":global(.foo) { color: red; }";
        let sheet = parse_stylesheet(StyleLanguage::Scss, source);
        let summary = super::summarize_parity_lite(&sheet);
        assert!(summary.selector_names.is_empty());
    }

    #[test]
    fn parity_summary_ignores_not_function_classes() {
        let source = ".btn:not(.disabled) { color: red; }";
        let sheet = parse_stylesheet(StyleLanguage::Scss, source);
        let summary = super::summarize_parity_lite(&sheet);
        assert_eq!(summary.selector_names, vec!["btn"]);
    }

    #[test]
    fn parity_summary_keeps_local_function_class() {
        let source = ":local(.foo) { color: red; }";
        let sheet = parse_stylesheet(StyleLanguage::Scss, source);
        let summary = super::summarize_parity_lite(&sheet);
        assert_eq!(summary.selector_names, vec!["foo"]);
    }

    #[test]
    fn index_summary_collects_sass_symbol_seed_facts() -> Result<(), String> {
        let source = r#"@use "./plain";
@use "./reset" as *;
@use "./tokens" as tokens;
@use "sass:color";
@forward "./theme";
@import "./legacy";
$gap: 1rem;
@mixin raised($depth) { box-shadow: 0 0 $depth black; }
@function tone($value) { @return $value; }
.btn { color: $gap; @include raised($gap); border-color: tone($gap); }
.ghost { color: $missing; @include absent($gap); }
"#;
        let sheet = parse_stylesheet(StyleLanguage::Scss, source);
        let summary = super::summarize_css_modules_intermediate(&sheet);

        assert_eq!(summary.sass.variable_decl_names, vec!["gap"]);
        assert_eq!(
            summary.sass.variable_parameter_names,
            vec!["depth", "value"]
        );
        assert_eq!(
            summary.sass.variable_ref_names,
            vec!["depth", "gap", "missing", "value"]
        );
        assert_eq!(
            summary.sass.selectors_with_variable_refs_names,
            vec!["btn", "ghost"]
        );
        assert_eq!(
            summary.sass.selectors_with_resolved_variable_refs_names,
            vec!["btn", "ghost"]
        );
        assert_eq!(
            summary.sass.selectors_with_unresolved_variable_refs_names,
            vec!["ghost"]
        );
        assert_eq!(summary.sass.mixin_decl_names, vec!["raised"]);
        assert_eq!(summary.sass.mixin_include_names, vec!["absent", "raised"]);
        assert_eq!(
            summary.sass.selectors_with_mixin_includes_names,
            vec!["btn", "ghost"]
        );
        assert_eq!(
            summary.sass.selectors_with_resolved_mixin_includes_names,
            vec!["btn"]
        );
        assert_eq!(
            summary.sass.selectors_with_unresolved_mixin_includes_names,
            vec!["ghost"]
        );
        assert_eq!(summary.sass.function_decl_names, vec!["tone"]);
        assert_eq!(summary.sass.function_call_names, vec!["tone"]);
        assert_eq!(
            summary.sass.selectors_with_function_calls_names,
            vec!["btn"]
        );
        assert_eq!(
            summary.sass.selector_symbol_facts,
            vec![
                ParserIndexSassSelectorSymbolFactV0 {
                    selector_name: "btn".to_string(),
                    symbol_kind: "function",
                    name: "tone".to_string(),
                    namespace: None,
                    role: "call",
                    resolution: "resolved",
                    byte_span: span_after(source, "border-color:", "tone")?,
                    range: range_after(source, "border-color:", "tone")?,
                },
                ParserIndexSassSelectorSymbolFactV0 {
                    selector_name: "btn".to_string(),
                    symbol_kind: "mixin",
                    name: "raised".to_string(),
                    namespace: None,
                    role: "include",
                    resolution: "resolved",
                    byte_span: span_after(source, "@include", "raised")?,
                    range: range_after(source, "@include", "raised")?,
                },
                ParserIndexSassSelectorSymbolFactV0 {
                    selector_name: "btn".to_string(),
                    symbol_kind: "variable",
                    name: "gap".to_string(),
                    namespace: None,
                    role: "reference",
                    resolution: "resolved",
                    byte_span: span_after(source, ".btn { color", "$gap")?,
                    range: range_after(source, ".btn { color", "$gap")?,
                },
                ParserIndexSassSelectorSymbolFactV0 {
                    selector_name: "btn".to_string(),
                    symbol_kind: "variable",
                    name: "gap".to_string(),
                    namespace: None,
                    role: "reference",
                    resolution: "resolved",
                    byte_span: span_after(source, "@include raised(", "$gap")?,
                    range: range_after(source, "@include raised(", "$gap")?,
                },
                ParserIndexSassSelectorSymbolFactV0 {
                    selector_name: "btn".to_string(),
                    symbol_kind: "variable",
                    name: "gap".to_string(),
                    namespace: None,
                    role: "reference",
                    resolution: "resolved",
                    byte_span: span_after(source, "border-color: tone(", "$gap")?,
                    range: range_after(source, "border-color: tone(", "$gap")?,
                },
                ParserIndexSassSelectorSymbolFactV0 {
                    selector_name: "ghost".to_string(),
                    symbol_kind: "mixin",
                    name: "absent".to_string(),
                    namespace: None,
                    role: "include",
                    resolution: "unresolved",
                    byte_span: span_after(source, ".ghost", "absent")?,
                    range: range_after(source, ".ghost", "absent")?,
                },
                ParserIndexSassSelectorSymbolFactV0 {
                    selector_name: "ghost".to_string(),
                    symbol_kind: "variable",
                    name: "gap".to_string(),
                    namespace: None,
                    role: "reference",
                    resolution: "resolved",
                    byte_span: span_after(source, "absent(", "$gap")?,
                    range: range_after(source, "absent(", "$gap")?,
                },
                ParserIndexSassSelectorSymbolFactV0 {
                    selector_name: "ghost".to_string(),
                    symbol_kind: "variable",
                    name: "missing".to_string(),
                    namespace: None,
                    role: "reference",
                    resolution: "unresolved",
                    byte_span: span_after(source, ".ghost { color", "$missing")?,
                    range: range_after(source, ".ghost { color", "$missing")?,
                },
            ]
        );
        assert_eq!(
            summary.sass.module_use_sources,
            vec!["./legacy", "./plain", "./reset", "./tokens", "sass:color"]
        );
        assert_eq!(
            summary.sass.module_use_edges,
            vec![
                ParserIndexSassModuleUseFactV0 {
                    source: "./legacy".to_string(),
                    namespace_kind: "wildcard",
                    namespace: None,
                },
                ParserIndexSassModuleUseFactV0 {
                    source: "./plain".to_string(),
                    namespace_kind: "default",
                    namespace: Some("plain".to_string()),
                },
                ParserIndexSassModuleUseFactV0 {
                    source: "./reset".to_string(),
                    namespace_kind: "wildcard",
                    namespace: None,
                },
                ParserIndexSassModuleUseFactV0 {
                    source: "./tokens".to_string(),
                    namespace_kind: "alias",
                    namespace: Some("tokens".to_string()),
                },
                ParserIndexSassModuleUseFactV0 {
                    source: "sass:color".to_string(),
                    namespace_kind: "default",
                    namespace: Some("color".to_string()),
                },
            ]
        );
        assert_eq!(summary.sass.module_forward_sources, vec!["./theme"]);
        assert_eq!(summary.sass.module_import_sources, vec!["./legacy"]);
        assert_eq!(
            summary
                .sass
                .same_file_resolution
                .resolved_variable_ref_names,
            vec!["depth", "gap", "value"]
        );
        assert_eq!(
            summary
                .sass
                .same_file_resolution
                .unresolved_variable_ref_names,
            vec!["missing"]
        );
        assert_eq!(
            summary
                .sass
                .same_file_resolution
                .resolved_mixin_include_names,
            vec!["raised"]
        );
        assert_eq!(
            summary
                .sass
                .same_file_resolution
                .unresolved_mixin_include_names,
            vec!["absent"]
        );
        assert_eq!(
            summary
                .sass
                .same_file_resolution
                .resolved_function_call_names,
            vec!["tone"]
        );
        Ok(())
    }

    #[test]
    fn index_summary_collects_css_custom_property_seed_facts() {
        let source = r#":root { --color-gray-700: #767678; }
@media (min-width: 1px) { :root { --brand: white; } .btn { color: var(--color-gray-700); } }
@supports (display: grid) { @layer ui { [data-theme="dark"] { --surface: black; } .card { color: var(--missing); } } }
"#;
        let sheet = parse_stylesheet(StyleLanguage::Css, source);
        let summary = super::summarize_css_modules_intermediate(&sheet);
        let semantic_boundary = super::summarize_semantic_boundary(&sheet);

        assert_eq!(
            summary.custom_properties.decl_names,
            vec!["--brand", "--color-gray-700", "--surface"]
        );
        assert_eq!(
            summary
                .custom_properties
                .decl_facts
                .iter()
                .map(|fact| (
                    fact.name.as_str(),
                    fact.source_order,
                    fact.selector_contexts.as_slice(),
                    fact.under_media,
                    fact.under_supports,
                    fact.under_layer,
                ))
                .collect::<Vec<_>>(),
            vec![
                (
                    "--brand",
                    1,
                    vec![":root".to_string()].as_slice(),
                    true,
                    false,
                    false,
                ),
                (
                    "--color-gray-700",
                    0,
                    vec![":root".to_string()].as_slice(),
                    false,
                    false,
                    false,
                ),
                (
                    "--surface",
                    2,
                    vec!["[data-theme=\"dark\"]".to_string()].as_slice(),
                    false,
                    true,
                    true,
                ),
            ]
        );
        assert_eq!(
            summary.custom_properties.decl_context_selectors,
            vec![":root", "[data-theme=\"dark\"]"]
        );
        assert_eq!(
            summary.custom_properties.decl_names_under_media,
            vec!["--brand"]
        );
        assert_eq!(
            summary.custom_properties.decl_names_under_supports,
            vec!["--surface"]
        );
        assert_eq!(
            summary.custom_properties.decl_names_under_layer,
            vec!["--surface"]
        );
        assert_eq!(
            summary.custom_properties.ref_names,
            vec!["--color-gray-700", "--missing"]
        );
        assert_eq!(
            summary
                .custom_properties
                .ref_facts
                .iter()
                .map(|fact| (
                    fact.name.as_str(),
                    fact.source_order,
                    fact.selector_contexts.as_slice(),
                    fact.under_media,
                    fact.under_supports,
                    fact.under_layer,
                ))
                .collect::<Vec<_>>(),
            vec![
                (
                    "--color-gray-700",
                    0,
                    vec![".btn".to_string()].as_slice(),
                    true,
                    false,
                    false,
                ),
                (
                    "--missing",
                    1,
                    vec![".card".to_string()].as_slice(),
                    false,
                    true,
                    true,
                ),
            ]
        );
        assert_eq!(
            summary.custom_properties.selectors_with_refs_names,
            vec!["btn", "card"]
        );
        assert_eq!(
            summary
                .custom_properties
                .selectors_with_refs_under_media_names,
            vec!["btn"]
        );
        assert_eq!(
            summary
                .custom_properties
                .selectors_with_refs_under_supports_names,
            vec!["card"]
        );
        assert_eq!(
            summary
                .custom_properties
                .selectors_with_refs_under_layer_names,
            vec!["card"]
        );
        assert_eq!(
            semantic_boundary
                .semantic_facts
                .custom_properties
                .resolved_ref_names,
            vec!["--color-gray-700"]
        );
        assert_eq!(
            semantic_boundary
                .semantic_facts
                .custom_properties
                .unresolved_ref_names,
            vec!["--missing"]
        );
    }

    #[test]
    fn semantic_boundary_respects_custom_property_selector_context() {
        let source = r#".theme { --brand: #222; }
.button { color: var(--brand); }
.theme .button { border-color: var(--brand); }
:root { --surface: white; }
.card { background: var(--surface); }
"#;
        let sheet = parse_stylesheet(StyleLanguage::Css, source);
        let semantic_boundary = super::summarize_semantic_boundary(&sheet);

        assert_eq!(
            semantic_boundary
                .semantic_facts
                .custom_properties
                .resolved_ref_names,
            vec!["--brand", "--surface"]
        );
        assert_eq!(
            semantic_boundary
                .semantic_facts
                .custom_properties
                .unresolved_ref_names,
            vec!["--brand"]
        );
    }

    #[test]
    fn semantic_boundary_separates_parser_syntax_from_semantic_resolution() -> Result<(), String> {
        let source = r#"@use "./tokens" as tokens;
$gap: 1rem;
@mixin raised($depth) { box-shadow: 0 0 $depth black; }
.btn { color: $gap; @include raised($gap); }
.ghost { color: $missing; }
"#;
        let sheet = parse_stylesheet(StyleLanguage::Scss, source);
        let summary = super::summarize_semantic_boundary(&sheet);

        assert_eq!(
            summary.parser_facts.sass.module_use_edges,
            vec![ParserIndexSassModuleUseFactV0 {
                source: "./tokens".to_string(),
                namespace_kind: "alias",
                namespace: Some("tokens".to_string()),
            }]
        );
        assert_eq!(
            summary.parser_facts.sass.variable_ref_names,
            vec!["depth", "gap", "missing"]
        );
        assert_eq!(
            summary
                .semantic_facts
                .sass
                .same_file_resolution
                .resolved_variable_ref_names,
            vec!["depth", "gap"]
        );
        assert_eq!(
            summary
                .semantic_facts
                .sass
                .same_file_resolution
                .unresolved_variable_ref_names,
            vec!["missing"]
        );
        assert_eq!(
            summary.semantic_facts.selector_identity.canonical_names,
            vec!["btn", "ghost"]
        );
        assert_eq!(
            summary.semantic_facts.sass.selector_symbol_facts,
            vec![
                ParserIndexSassSelectorSymbolFactV0 {
                    selector_name: "btn".to_string(),
                    symbol_kind: "mixin",
                    name: "raised".to_string(),
                    namespace: None,
                    role: "include",
                    resolution: "resolved",
                    byte_span: span_after(source, "@include", "raised")?,
                    range: range_after(source, "@include", "raised")?,
                },
                ParserIndexSassSelectorSymbolFactV0 {
                    selector_name: "btn".to_string(),
                    symbol_kind: "variable",
                    name: "gap".to_string(),
                    namespace: None,
                    role: "reference",
                    resolution: "resolved",
                    byte_span: span_after(source, ".btn { color", "$gap")?,
                    range: range_after(source, ".btn { color", "$gap")?,
                },
                ParserIndexSassSelectorSymbolFactV0 {
                    selector_name: "btn".to_string(),
                    symbol_kind: "variable",
                    name: "gap".to_string(),
                    namespace: None,
                    role: "reference",
                    resolution: "resolved",
                    byte_span: span_after(source, "@include raised(", "$gap")?,
                    range: range_after(source, "@include raised(", "$gap")?,
                },
                ParserIndexSassSelectorSymbolFactV0 {
                    selector_name: "ghost".to_string(),
                    symbol_kind: "variable",
                    name: "missing".to_string(),
                    namespace: None,
                    role: "reference",
                    resolution: "unresolved",
                    byte_span: span_after(source, ".ghost { color", "$missing")?,
                    range: range_after(source, ".ghost { color", "$missing")?,
                },
            ]
        );
        Ok(())
    }

    #[test]
    fn semantic_boundary_reports_lossless_cst_span_contract() {
        let source = "// heading\n.card { &--primary { color: red; } }\n";
        let sheet = parse_stylesheet(StyleLanguage::Scss, source);
        let summary = super::summarize_semantic_boundary(&sheet);

        assert_eq!(
            summary.parser_facts.lossless_cst.source_byte_len,
            source.len()
        );
        assert!(summary.parser_facts.lossless_cst.token_count > 0);
        assert_eq!(summary.parser_facts.lossless_cst.diagnostic_count, 0);
        assert!(
            summary
                .parser_facts
                .lossless_cst
                .all_token_spans_within_source
        );
        assert!(
            summary
                .parser_facts
                .lossless_cst
                .all_node_spans_within_source
        );
    }

    #[test]
    fn index_summary_does_not_resolve_sass_variables_from_another_local_scope() -> Result<(), String>
    {
        let source = ".one { $gap: 1rem; }\n.two { color: $gap; }\n";
        let sheet = parse_stylesheet(StyleLanguage::Scss, source);
        let summary = super::summarize_css_modules_intermediate(&sheet);

        assert_eq!(
            summary.sass.selectors_with_resolved_variable_refs_names,
            Vec::<String>::new()
        );
        assert_eq!(
            summary.sass.selectors_with_unresolved_variable_refs_names,
            vec!["two"]
        );
        assert_eq!(
            summary
                .sass
                .same_file_resolution
                .resolved_variable_ref_names,
            Vec::<String>::new()
        );
        assert_eq!(
            summary
                .sass
                .same_file_resolution
                .unresolved_variable_ref_names,
            vec!["gap"]
        );
        assert_eq!(
            summary.sass.selector_symbol_facts,
            vec![ParserIndexSassSelectorSymbolFactV0 {
                selector_name: "two".to_string(),
                symbol_kind: "variable",
                name: "gap".to_string(),
                namespace: None,
                role: "reference",
                resolution: "unresolved",
                byte_span: span_after(source, ".two", "$gap")?,
                range: range_after(source, ".two", "$gap")?,
            }]
        );
        Ok(())
    }

    #[test]
    fn index_summary_tracks_module_qualified_sass_refs_outside_same_file_resolution()
    -> Result<(), String> {
        let source = r#"@use "./tokens" as tokens;
@mixin raised { box-shadow: none; }
@function tone($value) { @return $value; }
.button {
  color: tokens.$gap;
  @include tokens.raised;
  border-color: tokens.tone(tokens.$gap);
}
"#;
        let sheet = parse_stylesheet(StyleLanguage::Scss, source);
        let summary = super::summarize_css_modules_intermediate(&sheet);

        assert_eq!(summary.sass.variable_ref_names, vec!["gap", "value"]);
        assert_eq!(
            summary
                .sass
                .same_file_resolution
                .resolved_variable_ref_names,
            vec!["value"]
        );
        assert_eq!(
            summary
                .sass
                .same_file_resolution
                .unresolved_variable_ref_names,
            Vec::<String>::new()
        );
        assert_eq!(summary.sass.mixin_include_names, Vec::<String>::new());
        assert_eq!(summary.sass.function_call_names, Vec::<String>::new());
        assert_eq!(
            summary.sass.selectors_with_variable_refs_names,
            vec!["button"]
        );
        assert_eq!(
            summary.sass.selectors_with_mixin_includes_names,
            vec!["button"]
        );
        assert_eq!(
            summary.sass.selectors_with_function_calls_names,
            vec!["button"]
        );
        assert_eq!(
            summary.sass.selector_symbol_facts,
            vec![
                ParserIndexSassSelectorSymbolFactV0 {
                    selector_name: "button".to_string(),
                    symbol_kind: "function",
                    name: "tone".to_string(),
                    namespace: Some("tokens".to_string()),
                    role: "call",
                    resolution: "external",
                    byte_span: span_after(source, "border-color:", "tone")?,
                    range: range_after(source, "border-color:", "tone")?,
                },
                ParserIndexSassSelectorSymbolFactV0 {
                    selector_name: "button".to_string(),
                    symbol_kind: "mixin",
                    name: "raised".to_string(),
                    namespace: Some("tokens".to_string()),
                    role: "include",
                    resolution: "external",
                    byte_span: span_after(source, "@include tokens.", "raised")?,
                    range: range_after(source, "@include tokens.", "raised")?,
                },
                ParserIndexSassSelectorSymbolFactV0 {
                    selector_name: "button".to_string(),
                    symbol_kind: "variable",
                    name: "gap".to_string(),
                    namespace: Some("tokens".to_string()),
                    role: "reference",
                    resolution: "external",
                    byte_span: span_after(source, "color: tokens.", "$gap")?,
                    range: range_after(source, "color: tokens.", "$gap")?,
                },
                ParserIndexSassSelectorSymbolFactV0 {
                    selector_name: "button".to_string(),
                    symbol_kind: "variable",
                    name: "gap".to_string(),
                    namespace: Some("tokens".to_string()),
                    role: "reference",
                    resolution: "external",
                    byte_span: span_after(source, "tokens.tone(", "$gap")?,
                    range: range_after(source, "tokens.tone(", "$gap")?,
                },
            ]
        );
        assert_eq!(
            summary.sass.module_use_edges,
            vec![ParserIndexSassModuleUseFactV0 {
                source: "./tokens".to_string(),
                namespace_kind: "alias",
                namespace: Some("tokens".to_string()),
            }]
        );
        Ok(())
    }

    #[test]
    fn index_summary_reports_sass_symbol_ranges_after_non_ascii_text() -> Result<(), String> {
        let source = "$gap: 1rem;\n.btn { content: \"한🙂\"; color: $gap; }\n";
        let sheet = parse_stylesheet(StyleLanguage::Scss, source);
        let summary = super::summarize_css_modules_intermediate(&sheet);

        assert_eq!(
            summary.sass.selector_symbol_facts,
            vec![ParserIndexSassSelectorSymbolFactV0 {
                selector_name: "btn".to_string(),
                symbol_kind: "variable",
                name: "gap".to_string(),
                namespace: None,
                role: "reference",
                resolution: "resolved",
                byte_span: ParserByteSpanV0 { start: 46, end: 50 },
                range: ParserRangeV0 {
                    start: ParserPositionV0 {
                        line: 1,
                        character: 30,
                    },
                    end: ParserPositionV0 {
                        line: 1,
                        character: 34,
                    },
                },
            }]
        );

        Ok(())
    }
}
