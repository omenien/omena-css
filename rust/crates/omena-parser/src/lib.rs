//! Green-field parser substrate for omena-css.
//!
//! This crate owns the cstree parser track and publishes parser facts for the
//! product query, bridge, LSP, and transform consumers.

use cstree::{
    Syntax,
    build::GreenNodeBuilder,
    green::GreenNode,
    interning::TokenInterner,
    syntax::SyntaxNode,
    text::{TextRange, TextSize},
};
use omena_interner::{
    NameKind, intern_class_name, intern_css_ident, intern_custom_property_name, intern_file_path,
    intern_keyframes_name, intern_mixin_name, intern_property_name, intern_selector_key,
};
pub use omena_syntax::StyleDialect;
use omena_syntax::SyntaxKind;
use serde::Serialize;
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

mod cst;
mod extension;
mod facts;
mod language;
// R1 narrow public surface: `public_product` is private; only this curated set
// of V0 contract types + summary fns is re-exported (no wildcard). Reuse of
// omena-parser as a building block goes through these names — keep the list
// explicit and minimal rather than widening to `pub use public_product::*`.
mod public_product;
mod recovery;
mod spans;
pub use cst::{
    AtRuleCstNode, BogusCstNode, CommaSeparatedComponentValueListCstNode, ComponentValueCstNode,
    ComponentValueListCstNode, CustomPropertyValueCstNode, DeclarationCstNode,
    DeclarationListCstNode, ParsedCst, RuleCstNode, SelectorCstNode, SimpleBlockCstNode,
    StylesheetCstNode, TypedCstNode, ValueCstNode, is_at_rule_node_kind,
};
use extension::{AtRuleBlockKind, AtRuleSpec, at_rule_spec, scss_at_rule_spec};
pub use extension::{BuiltinDialectExtension, DialectExtension};
pub use facts::{
    ParsedAnimationFact, ParsedAnimationFactKind, ParsedAtRuleFact,
    ParsedCssModuleComposesEdgeFact, ParsedCssModuleComposesEdgeKind, ParsedCssModuleComposesFact,
    ParsedCssModuleComposesFactKind, ParsedCssModuleValueDefinitionEdgeFact,
    ParsedCssModuleValueFact, ParsedCssModuleValueFactKind, ParsedCssModuleValueImportEdgeFact,
    ParsedExtendTargetFact, ParsedExtendTargetFactKind, ParsedIcssExportEdgeFact, ParsedIcssFact,
    ParsedIcssFactKind, ParsedIcssImportEdgeFact, ParsedSassIncludeFact, ParsedSassModuleEdgeFact,
    ParsedSassModuleEdgeFactKind, ParsedSassSymbolFact, ParsedSassSymbolFactKind,
    ParsedSelectorFact, ParsedSelectorFactKind, ParsedStyleFacts, ParsedVariableFact,
    ParsedVariableFactKind,
};
pub use language::StyleLanguage;
pub use public_product::{
    ParserCanonicalCandidateBundleV0, ParserCanonicalProducerSignalV0, ParserEvaluatorCandidatesV0,
    ParserIndexSummaryV0, dialect_for_path, summarize_css_modules_intermediate,
    summarize_parser_canonical_candidate, summarize_parser_canonical_producer_signal,
    summarize_parser_evaluator_candidates,
};
pub use recovery::{RECOVERY_DECLARATION, RECOVERY_SELECTOR, RECOVERY_TOP, TokenSet};
pub use spans::{ParserByteSpanV0, ParserPositionV0, ParserRangeV0};

const VALUES_L4_MATH_FUNCTION_NAMES: &[&str] = &[
    "min", "max", "clamp", "round", "mod", "rem", "sin", "cos", "tan", "asin", "acos", "atan",
    "atan2", "pow", "sqrt", "hypot", "log", "exp", "abs", "sign",
];

const CSS_COLOR_FUNCTION_NAMES: &[&str] = &[
    "rgb",
    "rgba",
    "hsl",
    "hsla",
    "hwb",
    "lab",
    "lch",
    "oklab",
    "oklch",
    "color",
    "color-mix",
    "device-cmyk",
    "light-dark",
    "contrast-color",
];

const CSS_GRADIENT_FUNCTION_NAMES: &[&str] = &[
    "linear-gradient",
    "radial-gradient",
    "conic-gradient",
    "repeating-linear-gradient",
    "repeating-radial-gradient",
    "repeating-conic-gradient",
];

const CSS_TRANSFORM_FUNCTION_NAMES: &[&str] = &[
    "matrix",
    "matrix3d",
    "translate",
    "translate3d",
    "translateX",
    "translateY",
    "translateZ",
    "scale",
    "scale3d",
    "scaleX",
    "scaleY",
    "scaleZ",
    "rotate",
    "rotate3d",
    "rotateX",
    "rotateY",
    "rotateZ",
    "skew",
    "skewX",
    "skewY",
    "perspective",
];

const CSS_FILTER_FUNCTION_NAMES: &[&str] = &[
    "blur",
    "brightness",
    "contrast",
    "drop-shadow",
    "grayscale",
    "hue-rotate",
    "invert",
    "opacity",
    "saturate",
    "sepia",
];

const CSS_IMAGE_FUNCTION_NAMES: &[&str] = &["image", "image-set", "cross-fade", "element", "paint"];

const CSS_SHAPE_FUNCTION_NAMES: &[&str] = &[
    "path", "shape", "ray", "inset", "circle", "ellipse", "polygon",
];

#[derive(Debug, Clone)]
pub struct ParseResult {
    green: GreenNode,
    interner: Option<Arc<TokenInterner>>,
    errors: Vec<ParseError>,
    token_count: usize,
    dialect: StyleDialect,
}

impl PartialEq for ParseResult {
    fn eq(&self, other: &Self) -> bool {
        self.green == other.green
            && self.errors == other.errors
            && self.token_count == other.token_count
            && self.dialect == other.dialect
    }
}

impl Eq for ParseResult {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LexResult {
    tokens: Vec<LexedToken>,
    errors: Vec<ParseError>,
    dialect: StyleDialect,
}

impl LexResult {
    pub fn tokens(&self) -> &[LexedToken] {
        &self.tokens
    }

    pub fn errors(&self) -> &[ParseError] {
        &self.errors
    }

    pub fn dialect(&self) -> StyleDialect {
        self.dialect
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LexedToken {
    pub kind: SyntaxKind,
    pub range: TextRange,
    pub text: String,
}

impl ParseResult {
    pub fn green(&self) -> &GreenNode {
        &self.green
    }

    pub fn syntax(&self) -> SyntaxNode<SyntaxKind> {
        if let Some(interner) = &self.interner {
            return SyntaxNode::new_root_with_resolver(self.green.clone(), Arc::clone(interner))
                .syntax()
                .clone();
        }
        SyntaxNode::new_root(self.green.clone())
    }

    pub fn source_text(&self) -> Option<String> {
        let syntax = self.syntax();
        syntax
            .try_resolved()
            .map(|resolved| resolved.text().to_string())
    }

    pub fn errors(&self) -> &[ParseError] {
        &self.errors
    }

    pub fn token_count(&self) -> usize {
        self.token_count
    }

    pub fn dialect(&self) -> StyleDialect {
        self.dialect
    }

    pub fn cst(&self) -> ParsedCst {
        ParsedCst::new(self.syntax())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    pub code: ParseErrorCode,
    pub range: TextRange,
    pub message: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseErrorCode {
    UnterminatedBlockComment,
    UnterminatedString,
    UnexpectedCharacter,
    ExpectedSelectorName,
    UnterminatedAttributeSelector,
    ExpectedValue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseEntryPoint {
    Stylesheet,
    RuleList,
    Rule,
    DeclarationList,
    Declaration,
    Value,
    ComponentValue,
    ComponentValueList,
    CommaSeparatedComponentValueList,
    SimpleBlock,
}

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
struct ParserSemanticNameCandidateV0 {
    kind: NameKind,
    text: String,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Token<'text> {
    kind: SyntaxKind,
    text: &'text str,
    range: TextRange,
}

pub fn parse(text: &str, dialect: StyleDialect) -> ParseResult {
    parse_entry_point(text, dialect, ParseEntryPoint::Stylesheet)
}

pub fn parse_entry_point(
    text: &str,
    dialect: StyleDialect,
    entry_point: ParseEntryPoint,
) -> ParseResult {
    let extension = BuiltinDialectExtension::new(dialect);
    parse_entry_point_with_extension(text, &extension, entry_point)
}

pub fn lex(text: &str, dialect: StyleDialect) -> LexResult {
    let extension = BuiltinDialectExtension::new(dialect);
    lex_with_extension(text, &extension)
}

pub fn lex_with_extension(text: &str, extension: &impl DialectExtension) -> LexResult {
    let (tokens, errors) = tokenize(text, extension);
    LexResult {
        tokens: tokens
            .into_iter()
            .map(|token| LexedToken {
                kind: token.kind,
                range: token.range,
                text: public_token_text(token.text),
            })
            .collect(),
        errors,
        dialect: extension.dialect(),
    }
}

pub fn parse_with_extension(text: &str, extension: &impl DialectExtension) -> ParseResult {
    parse_entry_point_with_extension(text, extension, ParseEntryPoint::Stylesheet)
}

pub fn parse_entry_point_with_extension(
    text: &str,
    extension: &impl DialectExtension,
    entry_point: ParseEntryPoint,
) -> ParseResult {
    let (tokens, errors) = tokenize(text, extension);
    let token_count = tokens.len();
    let mut parser = Parser::new(tokens, errors, extension.dialect());
    let (green, interner) = parser.parse_entry_point(entry_point);

    ParseResult {
        green,
        interner,
        errors: parser.into_errors(),
        token_count,
        dialect: extension.dialect(),
    }
}

pub fn collect_style_facts(text: &str, dialect: StyleDialect) -> ParsedStyleFacts {
    let extension = BuiltinDialectExtension::new(dialect);
    collect_style_facts_with_extension(text, &extension)
}

pub fn summarize_omena_parser_style_facts(
    style_source: &str,
    dialect: StyleDialect,
) -> OmenaParserStyleFactsSummaryV0 {
    let facts = collect_style_facts(style_source, dialect);
    let sass_symbol_resolution = summarize_omena_parser_sass_symbol_resolution(&facts.sass_symbols);
    let mut class_selector_names = Vec::new();
    let mut id_selector_names = Vec::new();
    let mut placeholder_selector_names = Vec::new();
    let mut keyframe_names = Vec::new();
    let mut animation_reference_names = Vec::new();
    let mut css_module_value_definition_names = BTreeSet::new();
    let mut css_module_value_reference_names = BTreeSet::new();
    let mut css_module_value_import_sources = BTreeSet::new();
    let mut css_module_composes_target_names = BTreeSet::new();
    let mut css_module_composes_import_sources = BTreeSet::new();
    let mut icss_export_names = BTreeSet::new();
    let mut icss_import_local_names = BTreeSet::new();
    let mut icss_import_remote_names = BTreeSet::new();
    let mut icss_import_sources = BTreeSet::new();
    let mut variable_names = BTreeSet::new();
    let mut sass_symbol_declaration_names = BTreeSet::new();
    let mut sass_symbol_reference_names = BTreeSet::new();
    let mut sass_module_use_sources = BTreeSet::new();
    let mut sass_module_forward_sources = BTreeSet::new();
    let mut sass_module_import_sources = BTreeSet::new();
    let mut custom_property_names = BTreeSet::new();
    let mut custom_property_decl_names = BTreeSet::new();
    let mut custom_property_ref_names = BTreeSet::new();

    for selector in facts.selectors {
        match selector.kind {
            ParsedSelectorFactKind::Class => class_selector_names.push(selector.name),
            ParsedSelectorFactKind::Id => id_selector_names.push(selector.name),
            ParsedSelectorFactKind::Placeholder => placeholder_selector_names.push(selector.name),
        }
    }

    for variable in facts.variables {
        match variable.kind {
            ParsedVariableFactKind::ScssDeclaration
            | ParsedVariableFactKind::ScssReference
            | ParsedVariableFactKind::LessDeclaration
            | ParsedVariableFactKind::LessReference => {
                variable_names.insert(variable.name);
            }
            ParsedVariableFactKind::CustomPropertyDeclaration
            | ParsedVariableFactKind::CustomPropertyReference => {
                custom_property_names.insert(variable.name.clone());
                match variable.kind {
                    ParsedVariableFactKind::CustomPropertyDeclaration => {
                        custom_property_decl_names.insert(variable.name);
                    }
                    ParsedVariableFactKind::CustomPropertyReference => {
                        custom_property_ref_names.insert(variable.name);
                    }
                    _ => {}
                }
            }
        }
    }

    for symbol in &facts.sass_symbols {
        match symbol.role {
            "declaration" => {
                sass_symbol_declaration_names.insert(symbol.name.clone());
            }
            _ => {
                sass_symbol_reference_names.insert(symbol.name.clone());
            }
        }
    }

    for edge in &facts.sass_module_edges {
        match edge.kind {
            ParsedSassModuleEdgeFactKind::Use => {
                sass_module_use_sources.insert(edge.source.clone());
            }
            ParsedSassModuleEdgeFactKind::Forward => {
                sass_module_forward_sources.insert(edge.source.clone());
            }
            ParsedSassModuleEdgeFactKind::Import => {
                sass_module_import_sources.insert(edge.source.clone());
            }
        }
    }

    for animation in facts.animations {
        match animation.kind {
            ParsedAnimationFactKind::KeyframesDeclaration => keyframe_names.push(animation.name),
            ParsedAnimationFactKind::AnimationNameReference => {
                animation_reference_names.push(animation.name);
            }
        }
    }

    for value in facts.css_module_values {
        match value.kind {
            ParsedCssModuleValueFactKind::Definition => {
                css_module_value_definition_names.insert(value.name);
            }
            ParsedCssModuleValueFactKind::Reference => {
                css_module_value_reference_names.insert(value.name);
            }
            ParsedCssModuleValueFactKind::ImportSource => {
                css_module_value_import_sources.insert(value.name);
            }
        }
    }

    for composes in facts.css_module_composes {
        match composes.kind {
            ParsedCssModuleComposesFactKind::Target => {
                css_module_composes_target_names.insert(composes.name);
            }
            ParsedCssModuleComposesFactKind::ImportSource => {
                css_module_composes_import_sources.insert(composes.name);
            }
        }
    }

    for icss in facts.icss {
        match icss.kind {
            ParsedIcssFactKind::ExportName => {
                icss_export_names.insert(icss.name);
            }
            ParsedIcssFactKind::ImportLocalName => {
                icss_import_local_names.insert(icss.name);
            }
            ParsedIcssFactKind::ImportRemoteName => {
                icss_import_remote_names.insert(icss.name);
            }
            ParsedIcssFactKind::ImportSource => {
                icss_import_sources.insert(icss.name);
            }
        }
    }

    OmenaParserStyleFactsSummaryV0 {
        schema_version: "0",
        product: "omena-parser.style-facts",
        dialect: style_dialect_label(dialect),
        class_selector_names,
        id_selector_names,
        placeholder_selector_names,
        keyframe_names,
        animation_reference_names,
        css_module_value_definition_names: css_module_value_definition_names.into_iter().collect(),
        css_module_value_reference_names: css_module_value_reference_names.into_iter().collect(),
        css_module_value_import_sources: css_module_value_import_sources.into_iter().collect(),
        css_module_value_import_edges: facts
            .css_module_value_import_edges
            .into_iter()
            .map(|edge| OmenaParserCssModuleValueImportEdgeFactV0 {
                remote_name: edge.remote_name,
                local_name: edge.local_name,
                import_source: edge.import_source,
            })
            .collect(),
        css_module_value_definition_edges: facts
            .css_module_value_definition_edges
            .into_iter()
            .map(|edge| OmenaParserCssModuleValueDefinitionEdgeFactV0 {
                definition_name: edge.definition_name,
                reference_names: edge.reference_names,
            })
            .collect(),
        css_module_composes_target_names: css_module_composes_target_names.into_iter().collect(),
        css_module_composes_import_sources: css_module_composes_import_sources
            .into_iter()
            .collect(),
        css_module_composes_edges: facts
            .css_module_composes_edges
            .into_iter()
            .map(|edge| OmenaParserCssModuleComposesEdgeFactV0 {
                kind: css_module_composes_edge_kind_label(edge.kind),
                owner_selector_names: edge.owner_selector_names,
                target_names: edge.target_names,
                import_source: edge.import_source,
            })
            .collect(),
        icss_export_names: icss_export_names.into_iter().collect(),
        icss_import_local_names: icss_import_local_names.into_iter().collect(),
        icss_import_remote_names: icss_import_remote_names.into_iter().collect(),
        icss_import_sources: icss_import_sources.into_iter().collect(),
        icss_import_edges: facts
            .icss_import_edges
            .into_iter()
            .map(|edge| OmenaParserIcssImportEdgeFactV0 {
                local_name: edge.local_name,
                remote_name: edge.remote_name,
                import_source: edge.import_source,
            })
            .collect(),
        icss_export_edges: facts
            .icss_export_edges
            .into_iter()
            .map(|edge| OmenaParserIcssExportEdgeFactV0 {
                export_name: edge.export_name,
                reference_names: edge.reference_names,
            })
            .collect(),
        variable_names: variable_names.into_iter().collect(),
        sass_symbol_declaration_names: sass_symbol_declaration_names.into_iter().collect(),
        sass_symbol_reference_names: sass_symbol_reference_names.into_iter().collect(),
        sass_symbol_facts: facts
            .sass_symbols
            .into_iter()
            .map(|symbol| OmenaParserSassSymbolFactV0 {
                kind: sass_symbol_fact_kind_label(symbol.kind),
                symbol_kind: symbol.symbol_kind,
                name: symbol.name,
                role: symbol.role,
                namespace: symbol.namespace,
            })
            .collect(),
        sass_symbol_resolution,
        sass_module_use_sources: sass_module_use_sources.into_iter().collect(),
        sass_module_forward_sources: sass_module_forward_sources.into_iter().collect(),
        sass_module_import_sources: sass_module_import_sources.into_iter().collect(),
        sass_module_edges: facts
            .sass_module_edges
            .into_iter()
            .map(|edge| OmenaParserSassModuleEdgeFactV0 {
                kind: sass_module_edge_fact_kind_label(edge.kind),
                source: edge.source,
                namespace_kind: edge.namespace_kind,
                namespace: edge.namespace,
                visibility_filter_kind: edge.visibility_filter_kind,
                visibility_filter_names: edge.visibility_filter_names,
            })
            .collect(),
        custom_property_names: custom_property_names.into_iter().collect(),
        custom_property_decl_names: custom_property_decl_names.into_iter().collect(),
        custom_property_ref_names: custom_property_ref_names.into_iter().collect(),
        at_rule_names: facts
            .at_rules
            .into_iter()
            .map(|at_rule| at_rule.name)
            .collect(),
        parser_error_count: facts.error_count,
    }
}

pub fn summarize_omena_parser_lex(source: &str, dialect: StyleDialect) -> OmenaParserLexSummaryV0 {
    let result = lex(source, dialect);
    OmenaParserLexSummaryV0 {
        schema_version: "0",
        product: "omena-parser.lex-result",
        dialect: style_dialect_label(result.dialect()),
        tokens: result
            .tokens()
            .iter()
            .map(|token| OmenaParserLexTokenV0 {
                kind: format!("{:?}", token.kind),
                text: token.text.clone(),
                start: token.range.start().into(),
                end: token.range.end().into(),
            })
            .collect(),
        parser_error_count: result.errors().len(),
    }
}

pub fn summarize_omena_parser_parity_lite(
    source: &str,
    dialect: StyleDialect,
) -> OmenaParserParityLiteSummaryV0 {
    let facts = collect_style_facts(source, dialect);
    let result = parse(source, dialect);
    let (tokens, _) = tokenize(source, &BuiltinDialectExtension::new(dialect));
    let mut structural = ParserStructuralSummary::default();
    summarize_parser_structural_range(&tokens, 0, tokens.len(), 0, &mut structural);
    let mut selector_names = collect_parity_lite_selector_names_from_tokens(&tokens);
    selector_names.sort();

    OmenaParserParityLiteSummaryV0 {
        schema_version: "0",
        language: style_dialect_label(dialect),
        selector_names,
        keyframes_names: sorted_unique(
            facts
                .animations
                .iter()
                .filter(|animation| animation.kind == ParsedAnimationFactKind::KeyframesDeclaration)
                .map(|animation| animation.name.clone()),
        ),
        value_decl_names: sorted_unique(
            facts
                .css_module_values
                .iter()
                .filter(|value| value.kind == ParsedCssModuleValueFactKind::Definition)
                .map(|value| value.name.clone()),
        ),
        diagnostic_count: result.errors().len(),
        rule_count: structural.rule_count,
        declaration_count: structural.declaration_count,
        grouped_selector_count: structural.grouped_selector_count,
        max_nesting_depth: structural.max_nesting_depth,
        at_rule_kind_counts: structural.at_rule_kind_counts,
        declaration_kind_counts: structural.declaration_kind_counts,
    }
}

fn style_dialect_label(dialect: StyleDialect) -> &'static str {
    match dialect {
        StyleDialect::Css => "css",
        StyleDialect::Scss => "scss",
        StyleDialect::Sass => "sass",
        StyleDialect::Less => "less",
    }
}

#[derive(Default)]
struct ParserStructuralSummary {
    rule_count: usize,
    declaration_count: usize,
    grouped_selector_count: usize,
    max_nesting_depth: usize,
    at_rule_kind_counts: OmenaParserAtRuleKindCountsV0,
    declaration_kind_counts: OmenaParserDeclarationKindCountsV0,
}

fn summarize_parser_structural_range(
    tokens: &[Token<'_>],
    start: usize,
    end: usize,
    depth: usize,
    summary: &mut ParserStructuralSummary,
) {
    let mut index = start;
    while index < end {
        index = skip_trivia_tokens(tokens, index, end);
        if index >= end {
            break;
        }

        if tokens[index].kind == SyntaxKind::AtKeyword {
            increment_omena_parser_at_rule_kind_count(
                &mut summary.at_rule_kind_counts,
                classify_omena_parser_at_rule_kind(tokens[index].text),
            );
            let next_depth = depth + 1;
            summary.max_nesting_depth = summary.max_nesting_depth.max(next_depth);
            if let Some((open, close)) = find_block_after_header(tokens, index, end) {
                summarize_parser_structural_range(tokens, open + 1, close, next_depth, summary);
                index = close + 1;
            } else {
                index = skip_statement(tokens, index, end);
            }
            continue;
        }

        let statement_end = css_module_value_statement_end(tokens, index);
        if is_root_less_variable_statement(tokens, index, statement_end.min(end), depth) {
            increment_omena_parser_at_rule_kind_count(
                &mut summary.at_rule_kind_counts,
                keyof_omena_parser_at_rule_kind_counts::Kind::Generic,
            );
            if statement_end >= end || tokens[statement_end].kind == SyntaxKind::RightBrace {
                break;
            }
            index = statement_end + 1;
            continue;
        }

        if statement_end < end && tokens[statement_end].kind == SyntaxKind::LeftBrace {
            summary.rule_count += 1;
            let next_depth = depth + 1;
            summary.max_nesting_depth = summary.max_nesting_depth.max(next_depth);
            let group_count = count_omena_parser_selector_groups(tokens, index, statement_end);
            if group_count > 1 {
                summary.grouped_selector_count += group_count;
            }
            if let Some(close) = matching_right_brace(tokens, statement_end, end) {
                summarize_parser_structural_range(
                    tokens,
                    statement_end + 1,
                    close,
                    next_depth,
                    summary,
                );
                index = close + 1;
            } else {
                index = statement_end + 1;
            }
            continue;
        }

        if let Some(colon_index) = declaration_colon_index(tokens, index, statement_end.min(end)) {
            summary.declaration_count += 1;
            let property = previous_non_trivia_token_index(tokens, colon_index, index)
                .map(|property| tokens[property].text)
                .unwrap_or_default();
            increment_omena_parser_declaration_kind_count(
                &mut summary.declaration_kind_counts,
                classify_omena_parser_declaration_kind(property),
            );
        }

        if statement_end >= end || tokens[statement_end].kind == SyntaxKind::RightBrace {
            break;
        }
        index = statement_end + 1;
    }
}

fn is_root_less_variable_statement(
    tokens: &[Token<'_>],
    start: usize,
    end: usize,
    depth: usize,
) -> bool {
    if depth != 0 {
        return false;
    }
    let Some(first) = next_non_trivia_token_index_until(tokens, start, end) else {
        return false;
    };
    tokens[first].kind == SyntaxKind::LessVariable
        && declaration_colon_index(tokens, first, end).is_some()
}

fn count_omena_parser_selector_groups(tokens: &[Token<'_>], start: usize, end: usize) -> usize {
    split_selector_groups(tokens, start, end)
        .into_iter()
        .filter(|(group_start, group_end)| {
            *group_start < *group_end
                && next_non_trivia_token_index_until(tokens, *group_start, *group_end).is_some()
        })
        .count()
}

fn collect_parity_lite_selector_names_from_tokens(tokens: &[Token<'_>]) -> Vec<String> {
    let mut names = Vec::new();
    collect_parity_lite_selector_names_in_range(tokens, 0, tokens.len(), &[], None, &mut names);
    names
}

fn collect_parity_lite_selector_names_in_range(
    tokens: &[Token<'_>],
    start: usize,
    end: usize,
    parent_branches: &[SelectorBranch],
    css_module_scope: Option<&'static str>,
    names: &mut Vec<String>,
) {
    let mut index = start;
    while index < end {
        index = skip_trivia_tokens(tokens, index, end);
        if index >= end {
            break;
        }

        if tokens[index].kind == SyntaxKind::AtKeyword {
            let block = find_block_after_header(tokens, index, end);
            if let Some((open, close)) = block {
                if tokens[index].text == "@nest" {
                    if css_module_scope == Some("global") {
                        collect_parity_lite_selector_names_in_range(
                            tokens,
                            open + 1,
                            close,
                            &[],
                            css_module_scope,
                            names,
                        );
                    } else {
                        let branches =
                            resolve_selector_header(tokens, index + 1, open, parent_branches);
                        names.extend(branches.iter().map(|branch| branch.name.clone()));
                        collect_grouped_ampersand_compound_selector_duplicates(
                            tokens,
                            index + 1,
                            open,
                            parent_branches.len(),
                            names,
                        );
                        collect_parity_lite_selector_names_in_range(
                            tokens,
                            open + 1,
                            close,
                            &branches,
                            css_module_scope,
                            names,
                        );
                    }
                } else if style_wrapper_at_rule(tokens[index].text) {
                    collect_parity_lite_selector_names_in_range(
                        tokens,
                        open + 1,
                        close,
                        parent_branches,
                        css_module_scope,
                        names,
                    );
                }
                index = close + 1;
            } else {
                index = skip_statement(tokens, index, end);
            }
            continue;
        }

        let Some((open, close)) = find_block_after_header(tokens, index, end) else {
            index = skip_statement(tokens, index, end);
            continue;
        };

        let effective_scope = css_module_scope
            .or_else(|| css_module_block_scope_marker_in_header(tokens, index, open));
        if effective_scope == Some("global") {
            collect_parity_lite_selector_names_in_range(
                tokens,
                open + 1,
                close,
                &[],
                effective_scope,
                names,
            );
        } else {
            let branches = resolve_selector_header(tokens, index, open, parent_branches);
            names.extend(branches.iter().map(|branch| branch.name.clone()));
            collect_grouped_ampersand_compound_selector_duplicates(
                tokens,
                index,
                open,
                parent_branches.len(),
                names,
            );
            collect_parity_lite_selector_names_in_range(
                tokens,
                open + 1,
                close,
                &branches,
                effective_scope,
                names,
            );
        }
        index = close + 1;
    }
}

fn collect_grouped_ampersand_compound_selector_duplicates(
    tokens: &[Token<'_>],
    start: usize,
    end: usize,
    parent_branch_count: usize,
    names: &mut Vec<String>,
) {
    if parent_branch_count <= 1 || !header_contains_ampersand(tokens, start, end) {
        return;
    }
    for (name, _) in collect_class_selector_names_from_header(tokens, start, end) {
        names.extend(std::iter::repeat_n(name, parent_branch_count - 1));
    }
}

fn header_contains_ampersand(tokens: &[Token<'_>], start: usize, end: usize) -> bool {
    tokens[start..end]
        .iter()
        .any(|token| token.kind == SyntaxKind::Ampersand)
}

fn classify_omena_parser_at_rule_kind(text: &str) -> keyof_omena_parser_at_rule_kind_counts::Kind {
    match text.trim_start_matches('@').to_ascii_lowercase().as_str() {
        "media" => keyof_omena_parser_at_rule_kind_counts::Kind::Media,
        "supports" => keyof_omena_parser_at_rule_kind_counts::Kind::Supports,
        "layer" => keyof_omena_parser_at_rule_kind_counts::Kind::Layer,
        "keyframes" | "-webkit-keyframes" => {
            keyof_omena_parser_at_rule_kind_counts::Kind::Keyframes
        }
        "value" => keyof_omena_parser_at_rule_kind_counts::Kind::Value,
        "at-root" => keyof_omena_parser_at_rule_kind_counts::Kind::AtRoot,
        _ => keyof_omena_parser_at_rule_kind_counts::Kind::Generic,
    }
}

fn increment_omena_parser_at_rule_kind_count(
    counts: &mut OmenaParserAtRuleKindCountsV0,
    kind: keyof_omena_parser_at_rule_kind_counts::Kind,
) {
    match kind {
        keyof_omena_parser_at_rule_kind_counts::Kind::Media => counts.media += 1,
        keyof_omena_parser_at_rule_kind_counts::Kind::Supports => counts.supports += 1,
        keyof_omena_parser_at_rule_kind_counts::Kind::Layer => counts.layer += 1,
        keyof_omena_parser_at_rule_kind_counts::Kind::Keyframes => counts.keyframes += 1,
        keyof_omena_parser_at_rule_kind_counts::Kind::Value => counts.value += 1,
        keyof_omena_parser_at_rule_kind_counts::Kind::AtRoot => counts.at_root += 1,
        keyof_omena_parser_at_rule_kind_counts::Kind::Generic => counts.generic += 1,
    }
}

fn classify_omena_parser_declaration_kind(
    property: &str,
) -> keyof_omena_parser_declaration_kind_counts::Kind {
    match property.trim().to_ascii_lowercase().as_str() {
        "composes" => keyof_omena_parser_declaration_kind_counts::Kind::Composes,
        "animation" => keyof_omena_parser_declaration_kind_counts::Kind::Animation,
        "animation-name" => keyof_omena_parser_declaration_kind_counts::Kind::AnimationName,
        _ => keyof_omena_parser_declaration_kind_counts::Kind::Generic,
    }
}

fn increment_omena_parser_declaration_kind_count(
    counts: &mut OmenaParserDeclarationKindCountsV0,
    kind: keyof_omena_parser_declaration_kind_counts::Kind,
) {
    match kind {
        keyof_omena_parser_declaration_kind_counts::Kind::Composes => counts.composes += 1,
        keyof_omena_parser_declaration_kind_counts::Kind::Animation => counts.animation += 1,
        keyof_omena_parser_declaration_kind_counts::Kind::AnimationName => {
            counts.animation_name += 1
        }
        keyof_omena_parser_declaration_kind_counts::Kind::Generic => counts.generic += 1,
    }
}

mod keyof_omena_parser_at_rule_kind_counts {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum Kind {
        Media,
        Supports,
        Layer,
        Keyframes,
        Value,
        AtRoot,
        Generic,
    }
}

mod keyof_omena_parser_declaration_kind_counts {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum Kind {
        Composes,
        Animation,
        AnimationName,
        Generic,
    }
}

fn sorted_unique(values: impl IntoIterator<Item = String>) -> Vec<String> {
    values
        .into_iter()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn css_module_composes_edge_kind_label(kind: ParsedCssModuleComposesEdgeKind) -> &'static str {
    match kind {
        ParsedCssModuleComposesEdgeKind::Local => "local",
        ParsedCssModuleComposesEdgeKind::Global => "global",
        ParsedCssModuleComposesEdgeKind::External => "external",
    }
}

fn sass_symbol_fact_kind_label(kind: ParsedSassSymbolFactKind) -> &'static str {
    match kind {
        ParsedSassSymbolFactKind::VariableDeclaration => "sassVariableDeclaration",
        ParsedSassSymbolFactKind::VariableReference => "sassVariableReference",
        ParsedSassSymbolFactKind::MixinDeclaration => "sassMixinDeclaration",
        ParsedSassSymbolFactKind::MixinInclude => "sassMixinInclude",
        ParsedSassSymbolFactKind::FunctionDeclaration => "sassFunctionDeclaration",
        ParsedSassSymbolFactKind::FunctionCall => "sassFunctionCall",
    }
}

fn sass_module_edge_fact_kind_label(kind: ParsedSassModuleEdgeFactKind) -> &'static str {
    match kind {
        ParsedSassModuleEdgeFactKind::Use => "sassUse",
        ParsedSassModuleEdgeFactKind::Forward => "sassForward",
        ParsedSassModuleEdgeFactKind::Import => "sassImport",
    }
}

fn summarize_omena_parser_sass_symbol_resolution(
    symbols: &[ParsedSassSymbolFact],
) -> OmenaParserSassSymbolResolutionV0 {
    let mut declaration_by_symbol: BTreeMap<
        (&'static str, Option<String>, String),
        (usize, &'static str),
    > = BTreeMap::new();
    let mut declaration_count = 0usize;
    let mut reference_count = 0usize;
    let mut edges = Vec::new();

    for (source_order, symbol) in symbols.iter().enumerate() {
        let kind = sass_symbol_fact_kind_label(symbol.kind);
        if sass_symbol_fact_kind_is_declaration(symbol.kind) {
            declaration_count += 1;
            declaration_by_symbol.insert(
                (
                    symbol.symbol_kind,
                    symbol.namespace.clone(),
                    symbol.name.clone(),
                ),
                (source_order, kind),
            );
            continue;
        }
        if !sass_symbol_fact_kind_is_reference(symbol.kind) {
            continue;
        }

        reference_count += 1;
        let declaration = declaration_by_symbol.get(&(
            symbol.symbol_kind,
            symbol.namespace.clone(),
            symbol.name.clone(),
        ));
        edges.push(OmenaParserSassSymbolResolutionEdgeV0 {
            symbol_kind: symbol.symbol_kind,
            name: symbol.name.clone(),
            namespace: symbol.namespace.clone(),
            reference_kind: kind,
            reference_role: symbol.role,
            reference_source_order: source_order,
            declaration_kind: declaration.map(|(_, declaration_kind)| *declaration_kind),
            declaration_source_order: declaration.map(|(declaration_order, _)| *declaration_order),
            status: if declaration.is_some() {
                "resolved"
            } else {
                "unresolved"
            },
        });
    }

    let resolved_reference_count = edges
        .iter()
        .filter(|edge| edge.status == "resolved")
        .count();

    OmenaParserSassSymbolResolutionV0 {
        schema_version: "0",
        product: "omena-parser.sass-symbol-same-file-resolution",
        resolution_scope: "same-file",
        declaration_count,
        reference_count,
        resolved_reference_count,
        unresolved_reference_count: reference_count.saturating_sub(resolved_reference_count),
        edges,
        capabilities: OmenaParserSassSymbolResolutionCapabilitiesV0 {
            same_file_lexical_resolution_ready: true,
            declaration_before_reference_ready: true,
            unresolved_reference_reporting_ready: true,
            cross_file_module_resolution_ready: false,
        },
    }
}

fn sass_symbol_fact_kind_is_declaration(kind: ParsedSassSymbolFactKind) -> bool {
    matches!(
        kind,
        ParsedSassSymbolFactKind::VariableDeclaration
            | ParsedSassSymbolFactKind::MixinDeclaration
            | ParsedSassSymbolFactKind::FunctionDeclaration
    )
}

fn sass_symbol_fact_kind_is_reference(kind: ParsedSassSymbolFactKind) -> bool {
    matches!(
        kind,
        ParsedSassSymbolFactKind::VariableReference
            | ParsedSassSymbolFactKind::MixinInclude
            | ParsedSassSymbolFactKind::FunctionCall
    )
}

pub fn summarize_parser_cst_equivalence(
    text: &str,
    dialect: StyleDialect,
) -> ParserCstEquivalenceSummaryV0 {
    let result = parse(text, dialect);
    let syntax = result.syntax();
    let cst = result.cst();

    let mut node_count = 0;
    let mut token_count = 0;
    let mut syntax_kind_round_trip_ready = true;
    let mut zero_unknown_kind_ready = true;

    for node in syntax.descendants() {
        node_count += 1;
        let kind = node.kind();
        syntax_kind_round_trip_ready &= SyntaxKind::from_raw(kind.into_raw()) == kind;
        zero_unknown_kind_ready &= SyntaxKind::ALL.contains(&kind);
    }

    for token in syntax
        .descendants_with_tokens()
        .filter_map(|element| element.into_token())
    {
        token_count += 1;
        let kind = token.kind();
        syntax_kind_round_trip_ready &= SyntaxKind::from_raw(kind.into_raw()) == kind;
        zero_unknown_kind_ready &= SyntaxKind::ALL.contains(&kind);
    }

    let typed_wrapper_count = usize::from(cst.stylesheet().is_some())
        + cst.rules().len()
        + cst.selectors().len()
        + cst.declarations().len()
        + cst.declaration_lists().len()
        + cst.values().len()
        + cst.component_values().len()
        + cst.simple_blocks().len()
        + cst.component_value_lists().len()
        + cst.comma_separated_component_value_lists().len()
        + cst.custom_property_values().len()
        + cst.at_rules().len()
        + cst.bogus_nodes().len();

    ParserCstEquivalenceSummaryV0 {
        product: "omena-parser.cst-equivalence",
        dialect,
        root_kind: syntax.kind(),
        parser_node_count: node_count,
        parser_token_count: token_count,
        typed_wrapper_count,
        source_text_round_trip_ready: result.source_text().as_deref() == Some(text),
        syntax_kind_round_trip_ready,
        zero_unknown_kind_ready,
        typed_cst_wrapper_ready: cst.stylesheet().is_some() && typed_wrapper_count > 1,
        ready_surfaces: vec![
            "parserCstEquivalence",
            "parserUsesOmenaSyntaxKind",
            "parserCstSourceTextRoundTrip",
            "typedCstWrapperEquivalence",
        ],
    }
}

pub fn collect_style_facts_with_extension(
    text: &str,
    extension: &impl DialectExtension,
) -> ParsedStyleFacts {
    let (tokens, lex_errors) = tokenize(text, extension);
    let mut parser = Parser::new(tokens.clone(), lex_errors, extension.dialect());
    let _green = parser.parse();
    let errors = parser.into_errors();
    let selectors = collect_selector_facts_from_tokens(&tokens);
    let variables = collect_variable_facts_from_tokens(&tokens);
    let sass_symbols = collect_sass_symbol_facts_from_tokens(&tokens);
    let sass_includes = collect_sass_include_facts_from_tokens(text, &tokens);
    let sass_module_edges = collect_sass_module_edge_facts_from_tokens(&tokens);
    let extend_targets = collect_extend_target_facts_from_tokens(&tokens);
    let animations = collect_animation_facts_from_tokens(&tokens);
    let css_module_values = collect_css_module_value_facts_from_tokens(&tokens);
    let css_module_value_import_edges =
        collect_css_module_value_import_edge_facts_from_tokens(&tokens);
    let css_module_value_definition_edges =
        collect_css_module_value_definition_edge_facts_from_tokens(&tokens);
    let css_module_composes = collect_css_module_composes_facts_from_tokens(&tokens);
    let css_module_composes_edges = collect_css_module_composes_edge_facts_from_tokens(&tokens);
    let icss = collect_icss_facts_from_tokens(&tokens);
    let icss_import_edges = collect_icss_import_edge_facts_from_tokens(&tokens);
    let icss_export_edges = collect_icss_export_edge_facts_from_tokens(&tokens);
    let at_rules = collect_at_rule_facts_from_tokens(&tokens, extension.dialect());

    ParsedStyleFacts {
        product: "omena-parser.style-facts",
        dialect: extension.dialect(),
        selector_count: selectors.len(),
        selectors,
        variable_count: variables.len(),
        variables,
        sass_symbol_count: sass_symbols.len(),
        sass_symbols,
        sass_include_count: sass_includes.len(),
        sass_includes,
        sass_module_edge_count: sass_module_edges.len(),
        sass_module_edges,
        extend_target_count: extend_targets.len(),
        extend_targets,
        animation_count: animations.len(),
        animations,
        css_module_value_count: css_module_values.len(),
        css_module_values,
        css_module_value_import_edge_count: css_module_value_import_edges.len(),
        css_module_value_import_edges,
        css_module_value_definition_edge_count: css_module_value_definition_edges.len(),
        css_module_value_definition_edges,
        css_module_composes_count: css_module_composes.len(),
        css_module_composes,
        css_module_composes_edge_count: css_module_composes_edges.len(),
        css_module_composes_edges,
        icss_count: icss.len(),
        icss,
        icss_import_edge_count: icss_import_edges.len(),
        icss_import_edges,
        icss_export_edge_count: icss_export_edges.len(),
        icss_export_edges,
        at_rule_count: at_rules.len(),
        at_rules,
        error_count: errors.len(),
    }
}

pub fn summarize_parser_semantic_name_consumption(
    text: &str,
    dialect: StyleDialect,
    db: &dyn salsa::Database,
) -> ParserSemanticNameConsumptionSummaryV0 {
    let facts = collect_style_facts(text, dialect);
    let candidates = parser_semantic_name_candidates(&facts);
    let interned_name_count = candidates
        .iter()
        .filter(|candidate| intern_parser_semantic_name(db, candidate.kind, &candidate.text))
        .count();
    let invalid_name_count = candidates.len().saturating_sub(interned_name_count);

    ParserSemanticNameConsumptionSummaryV0 {
        product: "omena-parser.semantic-name-consumption",
        dialect,
        semantic_name_count: candidates.len(),
        interned_name_count,
        invalid_name_count,
        class_name_count: count_parser_semantic_name_kind(&candidates, NameKind::ClassName),
        css_ident_count: count_parser_semantic_name_kind(&candidates, NameKind::CssIdent),
        property_name_count: count_parser_semantic_name_kind(&candidates, NameKind::PropertyName),
        selector_key_count: count_parser_semantic_name_kind(&candidates, NameKind::SelectorKey),
        custom_property_name_count: count_parser_semantic_name_kind(
            &candidates,
            NameKind::CustomPropertyName,
        ),
        keyframes_name_count: count_parser_semantic_name_kind(&candidates, NameKind::KeyframesName),
        mixin_name_count: count_parser_semantic_name_kind(&candidates, NameKind::MixinName),
        file_path_count: count_parser_semantic_name_kind(&candidates, NameKind::FilePath),
        ready_surfaces: vec![
            "parserSemanticNameConsumption",
            "typedInternerValidation",
            "styleFactNameKindProjection",
        ],
    }
}

fn parser_semantic_name_candidates(facts: &ParsedStyleFacts) -> Vec<ParserSemanticNameCandidateV0> {
    let mut candidates = Vec::new();

    for selector in &facts.selectors {
        let kind = match selector.kind {
            ParsedSelectorFactKind::Class => NameKind::ClassName,
            ParsedSelectorFactKind::Id | ParsedSelectorFactKind::Placeholder => {
                NameKind::SelectorKey
            }
        };
        push_parser_semantic_name_candidate(&mut candidates, kind, &selector.name);
    }

    for variable in &facts.variables {
        let kind = match variable.kind {
            ParsedVariableFactKind::CustomPropertyDeclaration
            | ParsedVariableFactKind::CustomPropertyReference => NameKind::CustomPropertyName,
            ParsedVariableFactKind::ScssDeclaration
            | ParsedVariableFactKind::ScssReference
            | ParsedVariableFactKind::LessDeclaration
            | ParsedVariableFactKind::LessReference => NameKind::CssIdent,
        };
        push_parser_semantic_name_candidate(&mut candidates, kind, &variable.name);
    }

    for symbol in &facts.sass_symbols {
        let kind = match symbol.kind {
            ParsedSassSymbolFactKind::MixinDeclaration | ParsedSassSymbolFactKind::MixinInclude => {
                NameKind::MixinName
            }
            ParsedSassSymbolFactKind::VariableDeclaration
            | ParsedSassSymbolFactKind::VariableReference
            | ParsedSassSymbolFactKind::FunctionDeclaration
            | ParsedSassSymbolFactKind::FunctionCall => NameKind::CssIdent,
        };
        push_parser_semantic_name_candidate(&mut candidates, kind, &symbol.name);
        if let Some(namespace) = &symbol.namespace {
            push_parser_semantic_name_candidate(&mut candidates, NameKind::CssIdent, namespace);
        }
    }

    for include in &facts.sass_includes {
        push_parser_semantic_name_candidate(&mut candidates, NameKind::MixinName, &include.name);
        if let Some(namespace) = &include.namespace {
            push_parser_semantic_name_candidate(&mut candidates, NameKind::CssIdent, namespace);
        }
    }

    for edge in &facts.sass_module_edges {
        push_parser_semantic_name_candidate(&mut candidates, NameKind::FilePath, &edge.source);
        if let Some(namespace) = &edge.namespace {
            push_parser_semantic_name_candidate(&mut candidates, NameKind::CssIdent, namespace);
        }
    }

    for animation in &facts.animations {
        push_parser_semantic_name_candidate(
            &mut candidates,
            NameKind::KeyframesName,
            &animation.name,
        );
    }

    for value in &facts.css_module_values {
        let kind = match value.kind {
            ParsedCssModuleValueFactKind::Definition | ParsedCssModuleValueFactKind::Reference => {
                NameKind::CssIdent
            }
            ParsedCssModuleValueFactKind::ImportSource => NameKind::FilePath,
        };
        push_parser_semantic_name_candidate(&mut candidates, kind, &value.name);
    }

    for edge in &facts.css_module_value_import_edges {
        push_parser_semantic_name_candidate(&mut candidates, NameKind::CssIdent, &edge.local_name);
        push_parser_semantic_name_candidate(&mut candidates, NameKind::CssIdent, &edge.remote_name);
        push_parser_semantic_name_candidate(
            &mut candidates,
            NameKind::FilePath,
            &edge.import_source,
        );
    }

    for edge in &facts.css_module_value_definition_edges {
        push_parser_semantic_name_candidate(
            &mut candidates,
            NameKind::CssIdent,
            &edge.definition_name,
        );
        for reference_name in &edge.reference_names {
            push_parser_semantic_name_candidate(
                &mut candidates,
                NameKind::CssIdent,
                reference_name,
            );
        }
    }

    for composes in &facts.css_module_composes {
        let kind = match composes.kind {
            ParsedCssModuleComposesFactKind::Target => NameKind::ClassName,
            ParsedCssModuleComposesFactKind::ImportSource => NameKind::FilePath,
        };
        push_parser_semantic_name_candidate(&mut candidates, kind, &composes.name);
    }

    for edge in &facts.css_module_composes_edges {
        for owner_selector_name in &edge.owner_selector_names {
            push_parser_semantic_name_candidate(
                &mut candidates,
                NameKind::ClassName,
                owner_selector_name,
            );
        }
        for target_name in &edge.target_names {
            push_parser_semantic_name_candidate(&mut candidates, NameKind::ClassName, target_name);
        }
        if let Some(import_source) = &edge.import_source {
            push_parser_semantic_name_candidate(&mut candidates, NameKind::FilePath, import_source);
        }
    }

    for icss in &facts.icss {
        let kind = match icss.kind {
            ParsedIcssFactKind::ImportSource => NameKind::FilePath,
            ParsedIcssFactKind::ExportName
            | ParsedIcssFactKind::ImportLocalName
            | ParsedIcssFactKind::ImportRemoteName => NameKind::CssIdent,
        };
        push_parser_semantic_name_candidate(&mut candidates, kind, &icss.name);
    }

    for edge in &facts.icss_import_edges {
        push_parser_semantic_name_candidate(&mut candidates, NameKind::CssIdent, &edge.local_name);
        push_parser_semantic_name_candidate(&mut candidates, NameKind::CssIdent, &edge.remote_name);
        push_parser_semantic_name_candidate(
            &mut candidates,
            NameKind::FilePath,
            &edge.import_source,
        );
    }

    for edge in &facts.icss_export_edges {
        push_parser_semantic_name_candidate(&mut candidates, NameKind::CssIdent, &edge.export_name);
        for reference_name in &edge.reference_names {
            push_parser_semantic_name_candidate(
                &mut candidates,
                NameKind::CssIdent,
                reference_name,
            );
        }
    }

    for at_rule in &facts.at_rules {
        push_parser_semantic_name_candidate(&mut candidates, NameKind::CssIdent, &at_rule.name);
    }

    candidates
}

fn push_parser_semantic_name_candidate(
    candidates: &mut Vec<ParserSemanticNameCandidateV0>,
    kind: NameKind,
    text: &str,
) {
    candidates.push(ParserSemanticNameCandidateV0 {
        kind,
        text: text.to_string(),
    });
}

fn count_parser_semantic_name_kind(
    candidates: &[ParserSemanticNameCandidateV0],
    kind: NameKind,
) -> usize {
    candidates
        .iter()
        .filter(|candidate| candidate.kind == kind)
        .count()
}

fn intern_parser_semantic_name(db: &dyn salsa::Database, kind: NameKind, text: &str) -> bool {
    match kind {
        NameKind::ClassName => intern_class_name(db, text).is_ok(),
        NameKind::CssIdent => intern_css_ident(db, text).is_ok(),
        NameKind::PropertyName => intern_property_name(db, text).is_ok(),
        NameKind::SelectorKey => intern_selector_key(db, text).is_ok(),
        NameKind::CustomPropertyName => intern_custom_property_name(db, text).is_ok(),
        NameKind::KeyframesName => intern_keyframes_name(db, text).is_ok(),
        NameKind::MixinName => intern_mixin_name(db, text).is_ok(),
        NameKind::FilePath => intern_file_path(db, text).is_ok(),
    }
}

pub fn summarize_pratt_value_parser_coverage() -> ParserPrattValueCoverageSummaryV0 {
    ParserPrattValueCoverageSummaryV0 {
        product: "omena-parser.pratt-value-coverage",
        infix_operator_kinds: vec![
            SyntaxKind::Plus,
            SyntaxKind::Minus,
            SyntaxKind::Star,
            SyntaxKind::Slash,
            SyntaxKind::Percent,
        ],
        prefix_operator_kinds: vec![SyntaxKind::Plus, SyntaxKind::Minus],
        value_expression_node_kinds: vec![
            SyntaxKind::UnaryExpression,
            SyntaxKind::BinaryExpression,
            SyntaxKind::ParenthesizedExpression,
            SyntaxKind::FunctionCall,
            SyntaxKind::FunctionArguments,
            SyntaxKind::ValueList,
            SyntaxKind::ComponentValueList,
            SyntaxKind::SimpleBlock,
            SyntaxKind::BogusValue,
        ],
        specialized_function_family_count: 10,
        css_values_l4_math_function_count: VALUES_L4_MATH_FUNCTION_NAMES.len(),
        css_color_function_count: CSS_COLOR_FUNCTION_NAMES.len(),
        ready_surfaces: vec![
            "prattValueParserCore",
            "prefixUnaryExpressions",
            "additiveMultiplicativePrecedence",
            "parenthesizedValueExpressions",
            "functionArgumentValueLists",
            "specializedCssValueFunctionFamilies",
            "valuesL4MathFunctionArityChecks",
            "varEnvAttrFunctionHeadChecks",
            "dynamicInterpolationEscapeHatches",
            "valueBogusRecovery",
        ],
        next_surfaces: vec!["fullPropertyValueGrammarRegistry"],
    }
}

pub fn summarize_recursive_descent_parser_coverage() -> ParserRecursiveDescentCoverageSummaryV0 {
    ParserRecursiveDescentCoverageSummaryV0 {
        product: "omena-parser.recursive-descent-coverage",
        dialect_count: 4,
        entry_point_count: 10,
        selector_surface_count: 12,
        at_rule_surface_count: 19,
        dialect_extension_surface_count: 17,
        recovery_surface_count: 8,
        ready_surfaces: vec![
            "recursiveDescentParserCore",
            "stylesheetRuleDeclarationEntryPoints",
            "selectorsLevelFourCstNodes",
            "registeredAtRulePreludeParsers",
            "cssNestingRuleItems",
            "scssDialectStatements",
            "sassIndentedBlocks",
            "lessDialectStatements",
            "bogusRecoverySkeleton",
            "styleFactExtractionSurface",
        ],
        next_surfaces: vec!["completeExternalSpecMirror"],
    }
}

pub fn summarize_parser_boundary() -> ParserBoundarySummary {
    ParserBoundarySummary {
        product: "omena-parser.boundary",
        tree_model: "cstree-green-root",
        parser_track: "greenFieldNextToEngineStyleParser",
        dialect_count: 4,
        shared_name_kind_count: NameKind::ALL.len(),
        ready_surfaces: vec![
            "lexResult",
            "lexedTokenTextSurface",
            "parseResult",
            "panicFreeTokenizer",
            "cstreeGreenBuilder",
            "tokenSetRecoveryScaffold",
            "dialectExtensionScaffold",
            "recursiveDescentParserCore",
            "recursiveDescentCoverageSummary",
            "selectorCstSkeleton",
            "atRuleRegistrySkeleton",
            "prattValueExpressionSkeleton",
            "prattValueParserCore",
            "prattValueCoverageSummary",
            "attributeMatcherTokenization",
            "attributeMatcherCstNodes",
            "attributeNameValueModifierCstNodes",
            "specializedValueFunctionCstNodes",
            "caseInsensitiveFunctionRegistry",
            "caseInsensitiveAtRuleRegistry",
            "valueAtomCstNodes",
            "identifierValueCstNodes",
            "stringValueCstNodes",
            "unicodeRangeValueCstNodes",
            "functionArgumentValueLists",
            "cssModuleScopeFunctionCstNodes",
            "cssModuleGlobalSelectorFactFiltering",
            "cssModuleLocalIdSelectorFacts",
            "cssModuleValueStyleFacts",
            "cssModuleValueDeclarationReferenceFacts",
            "cssModuleComposesStyleFacts",
            "icssStyleFacts",
            "animationNameStyleFacts",
            "animationShorthandStyleFacts",
            "scssStructuredBlockAtRules",
            "scssControlPreludeValidation",
            "scssControlStyleFactExtraction",
            "scssIncludeContentBlockStyleFacts",
            "scssSassModuleEdgeStyleFacts",
            "scssSassSymbolStyleFacts",
            "scssUtilityAtRules",
            "scssVariableFlagCstNodes",
            "scssNestedPropertyCstNodes",
            "scssModulePreludeSourceValidation",
            "scssModulePreludeClauseValidation",
            "scssModuleConfigCstNodes",
            "scssModuleConfigBogusRecovery",
            "scssPlaceholderSelectorCstNodes",
            "lessMixinDeclarationCstNodes",
            "lessMixinCallCstNodes",
            "lessMixinGuardCstNodes",
            "lessExtendPseudoCstNodes",
            "lessDetachedRulesetCstNodes",
            "lessNamespaceAccessCstNodes",
            "lessPropertyVariableTokenization",
            "lessPropertyVariableCstNodes",
            "lessEscapedStringTokenization",
            "lessEscapedStringValueCstNodes",
            "importantAnnotationTokenization",
            "urlTokenization",
            "urlValueCstNodes",
            "quotedUrlFunctionValueCstNodes",
            "conditionalAtRulePreludeCstNodes",
            "supportsAtRulePreludeValidation",
            "conditionalLevel5AtRuleCstNodes",
            "mediaQueryCstNodes",
            "mediaQueryListValidation",
            "importPreludeCstNodes",
            "importSourcePreludeValidation",
            "importTailPreludeValidation",
            "customMediaPreludeValidation",
            "propertyAtRuleNameValidation",
            "namedAtRulePreludeValidation",
            "containerAtRulePreludeValidation",
            "charsetNamespaceAtRulePreludeValidation",
            "keyframesAtRuleNameValidation",
            "emptyBlockAtRulePreludeValidation",
            "layerScopePreludeCstNodes",
            "layerAtRulePreludeValidation",
            "scopeAtRulePreludeValidation",
            "pageAtRulePreludeValidation",
            "pageMarginAtRuleCstNodes",
            "modernDeclarationAtRuleCstNodes",
            "fontFeatureValuesAtRuleCstNodes",
            "fontFeatureValuesPreludeValidation",
            "keyframeSelectorListValidation",
            "viewTransitionAtRuleCstNodes",
            "genericAtRulePreludeCstNodes",
            "bogusAtRulePreludeCstNodes",
            "nestingAtRuleCstNodes",
            "customMediaAtRuleCstNodes",
            "cssColorFunctionCstNodes",
            "colorFunctionArgumentChecks",
            "gradientFunctionCstNodes",
            "transformFunctionCstNodes",
            "filterFunctionCstNodes",
            "imageFunctionCstNodes",
            "shapeFunctionCstNodes",
            "envAttrFunctionCstNodes",
            "mathFunctionCstNodes",
            "mathFunctionArityChecks",
            "mathFunctionEmptyArgumentChecks",
            "varEnvAttrFunctionHeadChecks",
            "scssInterpolationTokenization",
            "scssInterpolationCstNodes",
            "lessInterpolationTokenization",
            "lessInterpolationCstNodes",
            "interpolationBogusRecovery",
            "unicodeRangeTokenization",
            "badStringTokenRecovery",
            "badStringValueBogusNodes",
            "emptyDeclarationValueRecovery",
            "emptyVariableValueRecovery",
            "missingSemicolonDeclarationRecovery",
            "coreBogusPopulationSlice",
            "dialectBogusPopulationSlice",
            "cssModuleValueCstNodes",
            "cssModuleComposesCstNodes",
            "icssModuleBlockCstNodes",
            "icssImportSourceValidation",
            "cssModuleFromClauseSourceValidation",
            "cssModuleComposesMultipleFromValidation",
            "cssModuleGlobalComposesValidation",
            "cssModuleBogusRecovery",
            "valueListCstNodes",
            "valueListBogusRecovery",
            "genericRecoveryBogusNodes",
            "sassIndentedTokenization",
            "sassIndentedBlockCstNodes",
            "sassIndentedStyleFacts",
            "differentialCorpusSeed",
            "differentialCorpus",
            "lightningCssDifferentialCorpusSlice",
            "lightningCssSelectorIdAndAtRuleDifferentialSlice",
            "midTypingNoPanicPropertySlice",
            "deterministicPanicFreeCorpus",
            "losslessCstTextRoundTripSmoke",
            "parseResultSourceTextSurface",
            "parseSourceParseRoundTripSmoke",
            "typedNumericValueAtomCstNodes",
            "bracketedValueCstNodes",
            "importantAnnotationCstNodes",
            "splitImportantAnnotationCstNodes",
            "unexpectedValueTokenBogusNodes",
            "cdoCdcTokenization",
            "cssIdentifierEscapeTokenization",
            "nullAndBomInputPreprocessingSlice",
            "hashDelimiterTokenization",
            "cssDashIdentTokenization",
            "signedNumericTokenization",
            "exponentNumericTokenization",
            "badUrlWhitespaceRecovery",
            "parserEntryPointApiSlice",
            "ruleListEntryPointApiSlice",
            "componentValueEntryPointApiSlice",
            "componentValueListEntryPointApiSlice",
            "commaSeparatedComponentValueListEntryPointApiSlice",
            "simpleBlockEntryPointApiSlice",
            "typedCstWrapperSlice",
            "parserCstEquivalence",
            "typedBogusCstWrapperSlice",
            "componentValueCstNodes",
            "simpleBlockCstNodes",
            "fullBogusPopulation",
            "componentValueListCstNodes",
            "commaSeparatedComponentValueListCstNodes",
            "customPropertyAnyValueComponentList",
            "customPropertyValueCstNodes",
            "functionalPseudoSelectorListCstNodes",
            "strictNotPseudoSelectorListCstNodes",
            "nthSelectorOfSelectorListCstNodes",
            "nthSelectorFormulaCstNodes",
            "hasRelativeSelectorListCstNodes",
            "langDirSelectorArgumentCstNodes",
            "namespaceQualifiedSelectorCstNodes",
            "selectorFunctionArgumentFactExclusion",
            "missingBlockCloseBogusTrivia",
            "initialDialectStatementNodes",
            "recoveryBogusSkeleton",
            "styleFactExtractionSurface",
            "parserSemanticNameConsumption",
            "productCutoverGate",
        ],
        not_ready_surfaces: vec![
            "completeExternalSpecMirror",
            "fullPropertyValueGrammarRegistry",
        ],
    }
}

fn tokenize<'text>(
    text: &'text str,
    extension: &impl DialectExtension,
) -> (Vec<Token<'text>>, Vec<ParseError>) {
    let mut tokenizer = Tokenizer::new(text, extension);
    tokenizer.tokenize();
    (tokenizer.tokens, tokenizer.errors)
}

struct Tokenizer<'text, 'extension, E> {
    text: &'text str,
    extension: &'extension E,
    offset: usize,
    scss_interpolation_depth: usize,
    less_interpolation_depth: usize,
    sass_indent_stack: Vec<usize>,
    tokens: Vec<Token<'text>>,
    errors: Vec<ParseError>,
}

struct Parser<'text> {
    tokens: Vec<Token<'text>>,
    position: usize,
    dialect: StyleDialect,
    builder: GreenNodeBuilder<'static, 'static, SyntaxKind>,
    errors: Vec<ParseError>,
}

impl<'text> Parser<'text> {
    fn new(tokens: Vec<Token<'text>>, errors: Vec<ParseError>, dialect: StyleDialect) -> Self {
        Self {
            tokens,
            position: 0,
            dialect,
            builder: GreenNodeBuilder::new(),
            errors,
        }
    }

    fn parse(&mut self) -> (GreenNode, Option<Arc<TokenInterner>>) {
        self.parse_entry_point(ParseEntryPoint::Stylesheet)
    }

    fn parse_entry_point(
        &mut self,
        entry_point: ParseEntryPoint,
    ) -> (GreenNode, Option<Arc<TokenInterner>>) {
        self.builder.start_node(SyntaxKind::Root);
        match entry_point {
            ParseEntryPoint::Stylesheet => {
                self.builder.start_node(SyntaxKind::Stylesheet);
                self.parse_stylesheet_items();
                self.builder.finish_node();
            }
            ParseEntryPoint::RuleList => {
                self.builder.start_node(SyntaxKind::RuleList);
                self.parse_rule_list_items();
                self.builder.finish_node();
            }
            ParseEntryPoint::Rule => self.parse_rule(),
            ParseEntryPoint::DeclarationList => {
                self.builder.start_node(SyntaxKind::DeclarationList);
                self.parse_declaration_list();
                self.builder.finish_node();
            }
            ParseEntryPoint::Declaration => self.parse_declaration(),
            ParseEntryPoint::Value => {
                self.builder.start_node(SyntaxKind::Value);
                self.parse_value_or_value_list_until(&[]);
                self.builder.finish_node();
            }
            ParseEntryPoint::ComponentValue => self.parse_component_value(&[]),
            ParseEntryPoint::ComponentValueList => self.parse_component_value_list_until(&[]),
            ParseEntryPoint::CommaSeparatedComponentValueList => {
                self.parse_comma_separated_component_value_list_until(&[])
            }
            ParseEntryPoint::SimpleBlock => self.parse_simple_block_entry_point(&[]),
        }
        self.parse_sass_indentation_bogus();
        self.parse_entry_point_trailing_bogus();
        self.builder.finish_node();

        let builder = std::mem::take(&mut self.builder);
        let (green, cache) = builder.finish();
        let interner = cache.and_then(|cache| cache.into_interner()).map(Arc::new);
        (green, interner)
    }

    fn parse_sass_indentation_bogus(&mut self) {
        if self.dialect != StyleDialect::Sass
            || !self
                .errors
                .iter()
                .any(|error| error.message == "inconsistent Sass indentation")
        {
            return;
        }
        self.builder.start_node(SyntaxKind::BogusSassIndentation);
        self.builder.finish_node();
    }

    fn parse_entry_point_trailing_bogus(&mut self) {
        self.eat_trivia();
        if self.at_end() {
            return;
        }
        self.builder.start_node(SyntaxKind::BogusRecovery);
        while !self.at_end() {
            self.token_current();
        }
        self.builder.finish_node();
    }

    fn into_errors(self) -> Vec<ParseError> {
        self.errors
    }

    fn parse_stylesheet_items(&mut self) {
        while !self.at_end() {
            self.eat_trivia();
            if self.at_end() {
                break;
            }
            match self.current_kind() {
                Some(SyntaxKind::AtKeyword) if self.current_is_css_module_value_rule() => {
                    self.parse_css_module_value_rule()
                }
                Some(SyntaxKind::AtKeyword) if self.current_dialect_at_rule_spec().is_some() => {
                    self.parse_dialect_at_rule()
                }
                Some(SyntaxKind::AtKeyword) => self.parse_at_rule(),
                Some(SyntaxKind::ScssVariable)
                    if matches!(self.dialect, StyleDialect::Scss | StyleDialect::Sass) =>
                {
                    self.parse_variable_declaration(SyntaxKind::ScssVariableDeclaration)
                }
                Some(SyntaxKind::LessVariable) if self.dialect == StyleDialect::Less => {
                    self.parse_variable_declaration(SyntaxKind::LessVariableDeclaration)
                }
                Some(SyntaxKind::Cdo | SyntaxKind::Cdc) => self.token_current(),
                Some(SyntaxKind::RightBrace | SyntaxKind::SassDedent) => self.token_current(),
                Some(SyntaxKind::Semicolon | SyntaxKind::SassOptionalSemicolon) => {
                    self.token_current()
                }
                Some(_) => self.parse_rule(),
                None => break,
            }
        }
    }

    fn parse_rule(&mut self) {
        let starts_less_mixin =
            self.dialect == StyleDialect::Less && self.current_starts_less_callable_signature();
        let has_rule_block = self.find_rule_block_open_before_recovery(&[
            SyntaxKind::Semicolon,
            SyntaxKind::SassOptionalSemicolon,
            SyntaxKind::RightBrace,
            SyntaxKind::SassDedent,
        ]);
        let kind = if let Some(kind) = self
            .current_icss_module_rule_kind()
            .filter(|_| has_rule_block)
        {
            kind
        } else if self.current_starts_less_mixin_declaration() {
            SyntaxKind::LessMixinDeclaration
        } else if starts_less_mixin {
            SyntaxKind::BogusLessMixin
        } else if has_rule_block {
            SyntaxKind::Rule
        } else {
            SyntaxKind::BogusRule
        };

        self.builder.start_node(kind);
        if kind == SyntaxKind::CssModuleImportBlock && !self.current_icss_import_has_source() {
            self.error_at_current(ParseErrorCode::ExpectedValue, "expected ICSS import source");
        }
        if kind == SyntaxKind::LessMixinDeclaration {
            self.parse_less_mixin_header();
        } else if kind == SyntaxKind::BogusLessMixin {
            self.parse_until_recovery_with_optional_less_guard(&[
                SyntaxKind::Semicolon,
                SyntaxKind::RightBrace,
                SyntaxKind::SassDedent,
            ]);
            self.error_at_current(
                ParseErrorCode::UnexpectedCharacter,
                "expected Less mixin block",
            );
        } else {
            self.parse_selector_list();
        }
        if self.current_kind() == Some(SyntaxKind::LeftBrace) {
            self.token_current();
            self.builder
                .start_node(if self.previous_left_brace_has_match() {
                    SyntaxKind::DeclarationList
                } else {
                    SyntaxKind::BogusDeclarationList
                });
            self.parse_declaration_list();
            self.builder.finish_node();
            if self.current_kind() == Some(SyntaxKind::RightBrace) {
                self.token_current();
            } else {
                self.missing_token_bogus_trivia(
                    ParseErrorCode::UnexpectedCharacter,
                    "unterminated declaration block",
                );
            }
        } else if self.current_kind() == Some(SyntaxKind::SassIndent) {
            self.builder.start_node(SyntaxKind::SassIndentedBlock);
            self.token_current();
            self.builder.start_node(SyntaxKind::DeclarationList);
            self.parse_declaration_list();
            self.builder.finish_node();
            if self.current_kind() == Some(SyntaxKind::SassDedent) {
                self.token_current();
            } else {
                self.missing_token_bogus_trivia(
                    ParseErrorCode::UnexpectedCharacter,
                    "unterminated Sass indented declaration block",
                );
            }
            self.builder.finish_node();
        } else {
            self.consume_until_recovery(&[
                SyntaxKind::Semicolon,
                SyntaxKind::SassOptionalSemicolon,
                SyntaxKind::RightBrace,
                SyntaxKind::SassDedent,
            ]);
            if self.current_kind().is_some_and(is_statement_end) {
                self.token_current();
            }
        }
        self.builder.finish_node();
    }

    fn current_icss_module_rule_kind(&self) -> Option<SyntaxKind> {
        if self.current_kind() != Some(SyntaxKind::Colon) {
            return None;
        }
        let (name_index, name_kind) = self.non_trivia_token_from(self.position + 1)?;
        if name_kind != SyntaxKind::Ident {
            return None;
        }
        match self.tokens.get(name_index)?.text {
            "export" => Some(SyntaxKind::CssModuleExportBlock),
            "import" => Some(SyntaxKind::CssModuleImportBlock),
            _ => None,
        }
    }

    fn current_icss_import_has_source(&self) -> bool {
        let Some((name_index, SyntaxKind::Ident)) = self.non_trivia_token_from(self.position + 1)
        else {
            return false;
        };
        if self
            .tokens
            .get(name_index)
            .is_none_or(|token| token.text != "import")
        {
            return false;
        }
        let Some((open_index, SyntaxKind::LeftParen)) = self.non_trivia_token_from(name_index + 1)
        else {
            return false;
        };
        let Some((_, source_kind)) = self.non_trivia_token_from(open_index + 1) else {
            return false;
        };
        matches!(
            source_kind,
            SyntaxKind::String | SyntaxKind::Url | SyntaxKind::ScssInterpolationStart
        )
    }

    fn parse_selector_list(&mut self) {
        self.parse_selector_list_until(&[]);
    }

    fn parse_selector_list_until(&mut self, recovery: &[SyntaxKind]) {
        let kind = if self.current_kind() == Some(SyntaxKind::LeftBrace) {
            SyntaxKind::BogusSelectorList
        } else {
            SyntaxKind::SelectorList
        };
        self.builder.start_node(kind);
        while !self.at_end() {
            match self.current_kind() {
                Some(SyntaxKind::Comma) => self.token_current(),
                Some(kind) if is_selector_boundary_until(kind, recovery) => break,
                Some(SyntaxKind::SassIndentedNewline) => self.token_current(),
                Some(_)
                    if recovery.contains(&SyntaxKind::RightParen)
                        && self.current_selector_item_is_bogus(recovery) =>
                {
                    self.parse_bogus_selector_until(recovery)
                }
                Some(_) => self.parse_selector_until(recovery),
                None => break,
            }
        }
        self.builder.finish_node();
    }

    fn parse_strict_selector_list_until(&mut self, recovery: &[SyntaxKind]) {
        self.builder.start_node(
            if self.selector_list_contains_bogus_item_until(recovery)
                && self.current_kind() != Some(SyntaxKind::RightParen)
            {
                SyntaxKind::BogusSelectorList
            } else {
                SyntaxKind::SelectorList
            },
        );
        while !self.at_end() {
            match self.current_kind() {
                Some(SyntaxKind::Comma) => self.token_current(),
                Some(kind) if is_selector_boundary_until(kind, recovery) => break,
                Some(SyntaxKind::SassIndentedNewline) => self.token_current(),
                Some(_)
                    if self.current_selector_item_is_bogus(recovery)
                        && self.current_kind() != Some(SyntaxKind::RightParen) =>
                {
                    self.parse_bogus_selector_until(recovery)
                }
                Some(_) => self.parse_selector_until(recovery),
                None => break,
            }
        }
        self.builder.finish_node();
    }

    fn parse_relative_selector_list_until(&mut self, recovery: &[SyntaxKind]) {
        self.builder.start_node(
            if self.current_selector_item_is_bogus(recovery)
                && self.current_kind() != Some(SyntaxKind::RightParen)
            {
                SyntaxKind::BogusSelectorList
            } else {
                SyntaxKind::RelativeSelectorList
            },
        );
        while !self.at_end() {
            match self.current_kind() {
                Some(SyntaxKind::Comma) => self.token_current(),
                Some(kind) if is_selector_boundary_until(kind, recovery) => break,
                Some(SyntaxKind::SassIndentedNewline) => self.token_current(),
                Some(_)
                    if self.current_selector_item_is_bogus(recovery)
                        && self.current_kind() != Some(SyntaxKind::RightParen) =>
                {
                    self.parse_bogus_selector_until(recovery)
                }
                Some(_) => self.parse_relative_selector_until(recovery),
                None => break,
            }
        }
        self.builder.finish_node();
    }

    fn parse_relative_selector_until(&mut self, recovery: &[SyntaxKind]) {
        self.builder.start_node(SyntaxKind::RelativeSelector);
        self.builder.start_node(SyntaxKind::ComplexSelector);
        self.parse_complex_selector_until(recovery);
        self.builder.finish_node();
        self.builder.finish_node();
    }

    fn parse_bogus_selector_until(&mut self, recovery: &[SyntaxKind]) {
        self.builder.start_node(SyntaxKind::BogusSelector);
        self.error_at_current(
            ParseErrorCode::UnexpectedCharacter,
            "invalid selector in selector list",
        );
        let mut paren_depth = 0usize;
        let mut bracket_depth = 0usize;
        while !self.at_end() {
            let Some(kind) = self.current_kind() else {
                break;
            };
            if paren_depth == 0
                && bracket_depth == 0
                && (kind == SyntaxKind::Comma || is_selector_boundary_until(kind, recovery))
            {
                break;
            }
            match kind {
                SyntaxKind::LeftParen => paren_depth += 1,
                SyntaxKind::RightParen => paren_depth = paren_depth.saturating_sub(1),
                SyntaxKind::LeftBracket => bracket_depth += 1,
                SyntaxKind::RightBracket => bracket_depth = bracket_depth.saturating_sub(1),
                _ => {}
            }
            self.token_current();
        }
        self.builder.finish_node();
    }

    fn parse_selector_until(&mut self, recovery: &[SyntaxKind]) {
        self.builder.start_node(SyntaxKind::Selector);
        self.builder.start_node(SyntaxKind::ComplexSelector);
        self.parse_complex_selector_until(recovery);
        self.builder.finish_node();
        self.builder.finish_node();
    }

    fn parse_complex_selector_until(&mut self, recovery: &[SyntaxKind]) {
        let mut has_component = false;
        while !self.at_end() {
            match self.current_kind() {
                Some(kind) if is_selector_boundary_until(kind, recovery) => break,
                Some(SyntaxKind::Whitespace) => {
                    if has_component
                        && self.next_non_trivia_kind().is_some_and(|kind| {
                            !is_selector_boundary_until(kind, recovery) && !is_combinator(kind)
                        })
                    {
                        self.parse_whitespace_combinator();
                        has_component = false;
                    } else {
                        self.token_current();
                    }
                }
                Some(SyntaxKind::SassIndentedNewline) => self.token_current(),
                Some(kind) if is_combinator(kind) => {
                    self.parse_combinator();
                    has_component = false;
                }
                Some(_) => {
                    self.parse_compound_selector_until(recovery);
                    has_component = true;
                }
                None => break,
            }
        }
    }

    fn parse_compound_selector_until(&mut self, recovery: &[SyntaxKind]) {
        let starts_valid = self.current_kind().is_some_and(|kind| {
            selector_component_can_start(kind)
                || self.current_starts_namespace_qualified_selector(kind)
                || is_interpolation_start(kind)
        });
        self.builder.start_node(if starts_valid {
            SyntaxKind::CompoundSelector
        } else {
            SyntaxKind::BogusCompoundSelector
        });
        let start = self.position;
        while !self.at_end() {
            match self.current_kind() {
                Some(kind)
                    if is_selector_boundary_until(kind, recovery)
                        || kind == SyntaxKind::Whitespace
                        || kind == SyntaxKind::SassIndentedNewline
                        || is_combinator(kind) =>
                {
                    break;
                }
                Some(SyntaxKind::Dot) => self.parse_class_selector(),
                Some(SyntaxKind::Hash) => self.parse_id_selector(),
                Some(kind) if self.current_starts_namespace_qualified_selector(kind) => {
                    self.parse_namespace_qualified_selector()
                }
                Some(SyntaxKind::Ident) => self.parse_type_selector(),
                Some(SyntaxKind::Star) => self.parse_universal_selector(),
                Some(SyntaxKind::Ampersand) => self.parse_nesting_selector(),
                Some(SyntaxKind::ScssPlaceholder) => self.parse_scss_placeholder_selector(),
                Some(kind) if is_interpolation_start(kind) => self.parse_interpolation(
                    kind,
                    &[
                        SyntaxKind::Comma,
                        SyntaxKind::LeftBrace,
                        SyntaxKind::SassIndent,
                        SyntaxKind::RightBrace,
                        SyntaxKind::SassDedent,
                        SyntaxKind::RightParen,
                        SyntaxKind::Semicolon,
                        SyntaxKind::SassOptionalSemicolon,
                    ],
                ),
                Some(SyntaxKind::LeftBracket) => self.parse_attribute_selector(),
                Some(SyntaxKind::Colon) if self.current_starts_less_extend_rule() => {
                    self.parse_less_extend_rule()
                }
                Some(SyntaxKind::Colon) => {
                    self.parse_pseudo_selector(SyntaxKind::PseudoClassSelector)
                }
                Some(SyntaxKind::DoubleColon) => {
                    self.parse_pseudo_selector(SyntaxKind::PseudoElementSelector)
                }
                Some(_) => self.token_current(),
                None => break,
            }
        }
        if self.position == start {
            self.token_current();
        }
        if !starts_valid {
            self.error_at_current(
                ParseErrorCode::UnexpectedCharacter,
                "expected selector component",
            );
        }
        self.builder.finish_node();
    }

    fn parse_class_selector(&mut self) {
        self.builder.start_node(SyntaxKind::ClassSelector);
        self.token_current();
        if matches!(
            self.current_kind(),
            Some(SyntaxKind::Ident | SyntaxKind::CustomPropertyName)
        ) {
            self.token_current();
        } else {
            self.empty_bogus_node(
                SyntaxKind::BogusSelector,
                ParseErrorCode::ExpectedSelectorName,
                "expected class selector name",
            );
        }
        self.builder.finish_node();
    }

    fn parse_id_selector(&mut self) {
        self.builder.start_node(SyntaxKind::IdSelector);
        self.token_current();
        self.builder.finish_node();
    }

    fn parse_type_selector(&mut self) {
        self.builder.start_node(SyntaxKind::TypeSelector);
        self.token_current();
        self.builder.finish_node();
    }

    fn parse_universal_selector(&mut self) {
        self.builder.start_node(SyntaxKind::UniversalSelector);
        self.token_current();
        self.builder.finish_node();
    }

    fn parse_namespace_qualified_selector(&mut self) {
        let selector_kind =
            if self.namespace_qualified_selector_target_kind() == Some(SyntaxKind::Star) {
                SyntaxKind::UniversalSelector
            } else {
                SyntaxKind::TypeSelector
            };
        self.builder.start_node(selector_kind);
        self.builder.start_node(SyntaxKind::NamespacePrefix);
        if self.current_kind() != Some(SyntaxKind::Pipe) {
            self.token_current();
        }
        self.token_current();
        self.builder.finish_node();
        if matches!(
            self.current_kind(),
            Some(SyntaxKind::Ident | SyntaxKind::CustomPropertyName | SyntaxKind::Star)
        ) {
            self.token_current();
        } else {
            self.empty_bogus_node(
                SyntaxKind::BogusSelector,
                ParseErrorCode::ExpectedSelectorName,
                "expected namespace-qualified selector name",
            );
        }
        self.builder.finish_node();
    }

    fn parse_nesting_selector(&mut self) {
        self.builder.start_node(SyntaxKind::NestingSelectorNode);
        self.token_current();
        self.builder.finish_node();
    }

    fn parse_scss_placeholder_selector(&mut self) {
        self.builder.start_node(SyntaxKind::ScssPlaceholderSelector);
        self.token_current();
        self.builder.finish_node();
    }

    fn parse_attribute_selector(&mut self) {
        let kind = if self.find_before_recovery(
            SyntaxKind::RightBracket,
            &[
                SyntaxKind::Comma,
                SyntaxKind::LeftBrace,
                SyntaxKind::RightBrace,
                SyntaxKind::Semicolon,
            ],
        ) {
            SyntaxKind::AttributeSelector
        } else {
            SyntaxKind::BogusSelector
        };
        self.builder.start_node(kind);
        self.token_current();
        let mut saw_matcher = false;
        let mut saw_value = false;
        let mut closed = false;
        while !self.at_end() {
            match self.current_kind() {
                Some(SyntaxKind::RightBracket) => {
                    self.token_current();
                    closed = true;
                    break;
                }
                Some(kind) if is_attribute_matcher(kind) => {
                    self.parse_attribute_matcher();
                    saw_matcher = true;
                }
                Some(kind) if is_selector_boundary(kind) => break,
                Some(kind) if !saw_matcher && attribute_name_token_can_start(kind) => {
                    self.parse_attribute_name()
                }
                Some(kind)
                    if saw_matcher && !saw_value && attribute_value_token_can_start(kind) =>
                {
                    self.parse_attribute_value();
                    saw_value = true;
                }
                Some(SyntaxKind::Ident | SyntaxKind::CustomPropertyName) if saw_value => {
                    self.parse_attribute_modifier()
                }
                Some(_) => self.token_current(),
                None => break,
            }
        }
        if !closed {
            self.error_at_current(
                ParseErrorCode::UnterminatedAttributeSelector,
                "unterminated attribute selector",
            );
        }
        self.builder.finish_node();
    }

    fn parse_attribute_matcher(&mut self) {
        self.builder.start_node(SyntaxKind::AttributeMatcher);
        self.token_current();
        self.builder.finish_node();
    }

    fn parse_attribute_name(&mut self) {
        self.builder.start_node(SyntaxKind::AttributeName);
        while !self.at_end() {
            match self.current_kind() {
                Some(SyntaxKind::RightBracket) => break,
                Some(kind) if is_attribute_matcher(kind) || is_selector_boundary(kind) => break,
                Some(kind) if attribute_name_token_can_continue(kind) => self.token_current(),
                Some(_) => break,
                None => break,
            }
        }
        self.builder.finish_node();
    }

    fn parse_attribute_value(&mut self) {
        self.builder.start_node(SyntaxKind::AttributeValue);
        self.token_current();
        self.builder.finish_node();
    }

    fn parse_attribute_modifier(&mut self) {
        self.builder.start_node(SyntaxKind::AttributeModifier);
        self.token_current();
        self.builder.finish_node();
    }

    fn parse_pseudo_selector(&mut self, kind: SyntaxKind) {
        self.builder.start_node(kind);
        self.token_current();
        let pseudo_name = self.current_text().map(str::to_owned);
        let css_module_scope_kind = if kind == SyntaxKind::PseudoClassSelector {
            self.current_text().and_then(css_module_scope_function_kind)
        } else {
            None
        };
        if self.current_kind() == Some(SyntaxKind::Ident) {
            if let Some(kind) = css_module_scope_kind {
                self.builder.start_node(kind);
            }
            self.token_current();
        } else {
            self.empty_bogus_node(
                SyntaxKind::BogusSelector,
                ParseErrorCode::ExpectedSelectorName,
                "expected pseudo selector name",
            );
        }
        if self.current_kind() == Some(SyntaxKind::LeftParen) {
            self.token_current();
            self.builder.start_node(SyntaxKind::PseudoSelectorArgument);
            if kind == SyntaxKind::PseudoClassSelector
                && pseudo_name
                    .as_deref()
                    .is_some_and(is_selector_list_pseudo_class)
            {
                self.parse_selector_list_until(&[SyntaxKind::RightParen]);
            } else if kind == SyntaxKind::PseudoClassSelector
                && pseudo_name.as_deref() == Some("not")
            {
                self.parse_strict_selector_list_until(&[SyntaxKind::RightParen]);
            } else if kind == SyntaxKind::PseudoClassSelector
                && pseudo_name.as_deref() == Some("has")
            {
                self.parse_relative_selector_list_until(&[SyntaxKind::RightParen]);
            } else if kind == SyntaxKind::PseudoClassSelector
                && pseudo_name.as_deref().is_some_and(is_nth_pseudo_class)
            {
                self.parse_nth_selector_argument();
            } else if kind == SyntaxKind::PseudoClassSelector
                && pseudo_name.as_deref() == Some("lang")
            {
                self.parse_language_selector_argument();
            } else if kind == SyntaxKind::PseudoClassSelector
                && pseudo_name.as_deref() == Some("dir")
            {
                self.parse_directionality_selector_argument();
            } else {
                while !self.at_end() {
                    match self.current_kind() {
                        Some(SyntaxKind::RightParen) => break,
                        Some(kind) if is_selector_boundary(kind) => break,
                        Some(_) => self.token_current(),
                        None => break,
                    }
                }
            }
            self.builder.finish_node();
            if self.current_kind() == Some(SyntaxKind::RightParen) {
                self.token_current();
            }
        }
        if css_module_scope_kind.is_some() {
            self.builder.finish_node();
        }
        self.builder.finish_node();
    }

    fn parse_nth_selector_argument(&mut self) {
        self.builder.start_node(SyntaxKind::NthSelectorArgument);
        self.builder.start_node(SyntaxKind::NthSelectorFormula);
        while !self.at_end() {
            match self.current_kind() {
                Some(SyntaxKind::RightParen) => break,
                Some(kind) if is_selector_boundary(kind) => break,
                Some(SyntaxKind::Ident) if self.current_text() == Some("of") => break,
                Some(_) => self.token_current(),
                None => break,
            }
        }
        self.builder.finish_node();

        if self.current_kind() == Some(SyntaxKind::Ident) && self.current_text() == Some("of") {
            self.builder
                .start_node(SyntaxKind::NthSelectorOfSelectorList);
            self.token_current();
            self.parse_selector_list_until(&[SyntaxKind::RightParen]);
            self.builder.finish_node();
        }

        self.builder.finish_node();
    }

    fn parse_language_selector_argument(&mut self) {
        self.builder
            .start_node(SyntaxKind::LanguageSelectorArgument);
        while !self.at_end() {
            match self.current_kind() {
                Some(SyntaxKind::RightParen) => break,
                Some(SyntaxKind::Comma) => self.token_current(),
                Some(kind) if is_selector_boundary(kind) => break,
                Some(kind) if language_tag_token_can_start(kind) => self.parse_language_tag(),
                Some(_) => self.token_current(),
                None => break,
            }
        }
        self.builder.finish_node();
    }

    fn parse_language_tag(&mut self) {
        self.builder.start_node(SyntaxKind::LanguageTag);
        self.token_current();
        self.builder.finish_node();
    }

    fn parse_directionality_selector_argument(&mut self) {
        self.builder
            .start_node(SyntaxKind::DirectionalitySelectorArgument);
        if self
            .current_kind()
            .is_some_and(language_tag_token_can_start)
        {
            self.token_current();
        }
        while !self.at_end() {
            match self.current_kind() {
                Some(SyntaxKind::RightParen) => break,
                Some(kind) if is_selector_boundary(kind) => break,
                Some(_) => self.token_current(),
                None => break,
            }
        }
        self.builder.finish_node();
    }

    fn parse_less_extend_rule(&mut self) {
        self.builder.start_node(SyntaxKind::LessExtendRule);
        if self.current_kind() == Some(SyntaxKind::Colon) {
            self.token_current();
        }
        if self.current_text() == Some("extend") {
            self.token_current();
        } else {
            self.empty_bogus_node(
                SyntaxKind::BogusSelector,
                ParseErrorCode::ExpectedSelectorName,
                "expected Less extend selector",
            );
        }
        if self.current_kind() == Some(SyntaxKind::LeftParen) {
            self.token_current();
            self.builder.start_node(SyntaxKind::PseudoSelectorArgument);
            while !self.at_end() {
                match self.current_kind() {
                    Some(SyntaxKind::RightParen) => break,
                    Some(kind) if is_selector_boundary(kind) => break,
                    Some(kind) if is_interpolation_start(kind) => self.parse_interpolation(
                        kind,
                        &[
                            SyntaxKind::RightParen,
                            SyntaxKind::Comma,
                            SyntaxKind::LeftBrace,
                            SyntaxKind::SassIndent,
                            SyntaxKind::Semicolon,
                            SyntaxKind::SassOptionalSemicolon,
                        ],
                    ),
                    Some(_) => self.token_current(),
                    None => break,
                }
            }
            self.builder.finish_node();
            if self.current_kind() == Some(SyntaxKind::RightParen) {
                self.token_current();
            }
        }
        self.builder.finish_node();
    }

    fn parse_combinator(&mut self) {
        let has_rhs = self
            .next_non_trivia_kind()
            .is_some_and(|kind| selector_component_can_start(kind) || is_interpolation_start(kind));
        self.builder.start_node(if has_rhs {
            SyntaxKind::Combinator
        } else {
            SyntaxKind::BogusCombinator
        });
        self.token_current();
        if !has_rhs {
            self.error_at_current(
                ParseErrorCode::UnexpectedCharacter,
                "expected selector after combinator",
            );
        }
        self.builder.finish_node();
    }

    fn parse_whitespace_combinator(&mut self) {
        self.builder.start_node(SyntaxKind::Combinator);
        while self.current_kind() == Some(SyntaxKind::Whitespace) {
            self.token_current();
        }
        self.builder.finish_node();
    }

    fn parse_declaration_list(&mut self) {
        while !self.at_end() {
            self.eat_trivia();
            match self.current_kind() {
                Some(SyntaxKind::RightBrace | SyntaxKind::SassDedent) | None => break,
                Some(SyntaxKind::Semicolon | SyntaxKind::SassOptionalSemicolon) => {
                    self.token_current()
                }
                Some(SyntaxKind::AtKeyword) if self.current_is_css_module_value_rule() => {
                    self.parse_css_module_value_rule()
                }
                Some(SyntaxKind::AtKeyword) if self.current_dialect_at_rule_spec().is_some() => {
                    self.parse_dialect_at_rule()
                }
                Some(SyntaxKind::AtKeyword) => self.parse_at_rule(),
                Some(_) if self.current_starts_less_namespace_access() => {
                    self.parse_less_namespace_access()
                }
                Some(_) if self.current_starts_less_mixin_call() => self.parse_less_mixin_call(),
                Some(_) if self.current_starts_scss_nested_property() => {
                    self.parse_scss_nested_property()
                }
                Some(_) if self.current_starts_nested_rule() => self.parse_rule(),
                Some(SyntaxKind::ScssVariable)
                    if matches!(self.dialect, StyleDialect::Scss | StyleDialect::Sass) =>
                {
                    self.parse_variable_declaration(SyntaxKind::ScssVariableDeclaration)
                }
                Some(SyntaxKind::LessVariable) if self.dialect == StyleDialect::Less => {
                    self.parse_variable_declaration(SyntaxKind::LessVariableDeclaration)
                }
                Some(SyntaxKind::LeftBrace) => {
                    self.builder.start_node(SyntaxKind::BogusDeclaration);
                    self.token_current();
                    self.builder.finish_node();
                }
                Some(_) => self.parse_declaration(),
            }
        }
    }

    fn parse_scss_nested_property(&mut self) {
        self.builder.start_node(SyntaxKind::ScssNestedProperty);
        self.builder.start_node(SyntaxKind::PropertyName);
        while !self.at_end() {
            match self.current_kind() {
                Some(SyntaxKind::Colon) => break,
                Some(
                    SyntaxKind::Semicolon
                    | SyntaxKind::SassOptionalSemicolon
                    | SyntaxKind::RightBrace
                    | SyntaxKind::SassDedent,
                ) => break,
                Some(kind) if is_interpolation_start(kind) => self.parse_interpolation(
                    kind,
                    &[
                        SyntaxKind::Colon,
                        SyntaxKind::Semicolon,
                        SyntaxKind::SassOptionalSemicolon,
                        SyntaxKind::RightBrace,
                        SyntaxKind::SassDedent,
                    ],
                ),
                Some(_) => self.token_current(),
                None => break,
            }
        }
        self.builder.finish_node();

        if self.current_kind() == Some(SyntaxKind::Colon) {
            self.token_current();
        }

        let block_recovery = [
            SyntaxKind::LeftBrace,
            SyntaxKind::SassIndent,
            SyntaxKind::Semicolon,
            SyntaxKind::SassOptionalSemicolon,
            SyntaxKind::RightBrace,
            SyntaxKind::SassDedent,
        ];
        if !matches!(
            self.current_kind(),
            Some(
                SyntaxKind::LeftBrace
                    | SyntaxKind::SassIndent
                    | SyntaxKind::Semicolon
                    | SyntaxKind::SassOptionalSemicolon
                    | SyntaxKind::RightBrace
                    | SyntaxKind::SassDedent
            )
        ) {
            self.builder.start_node(SyntaxKind::Value);
            self.parse_value_or_value_list_until(&block_recovery);
            self.builder.finish_node();
        }

        match self.current_kind() {
            Some(SyntaxKind::LeftBrace) => self.parse_declaration_block(),
            Some(SyntaxKind::SassIndent) => self.parse_sass_indented_nested_property_block(),
            Some(_) => self.consume_until_recovery(&[
                SyntaxKind::Semicolon,
                SyntaxKind::SassOptionalSemicolon,
                SyntaxKind::RightBrace,
                SyntaxKind::SassDedent,
            ]),
            None => {}
        }

        if self.current_kind().is_some_and(is_statement_end) {
            self.token_current();
        }
        self.builder.finish_node();
    }

    fn parse_sass_indented_nested_property_block(&mut self) {
        self.builder.start_node(SyntaxKind::SassIndentedBlock);
        if self.current_kind() == Some(SyntaxKind::SassIndent) {
            self.token_current();
        }
        self.builder.start_node(SyntaxKind::DeclarationList);
        self.parse_declaration_list();
        self.builder.finish_node();
        if self.current_kind() == Some(SyntaxKind::SassDedent) {
            self.token_current();
        } else {
            self.error_at_current(
                ParseErrorCode::UnexpectedCharacter,
                "unterminated Sass indented nested property block",
            );
        }
        self.builder.finish_node();
    }

    fn parse_variable_declaration(&mut self, kind: SyntaxKind) {
        let has_colon = self.find_before_recovery(
            SyntaxKind::Colon,
            &[
                SyntaxKind::Semicolon,
                SyntaxKind::SassOptionalSemicolon,
                SyntaxKind::RightBrace,
                SyntaxKind::SassDedent,
            ],
        );
        self.builder
            .start_node(variable_declaration_node_kind(kind, has_colon));
        self.token_current();
        if self.current_kind() == Some(SyntaxKind::Colon) {
            self.token_current();
            self.eat_value_trivia();
            let value_recovery = [
                SyntaxKind::Semicolon,
                SyntaxKind::SassOptionalSemicolon,
                SyntaxKind::RightBrace,
                SyntaxKind::SassDedent,
            ];
            if kind == SyntaxKind::LessVariableDeclaration
                && self.current_kind() == Some(SyntaxKind::LeftBrace)
            {
                self.parse_less_detached_ruleset();
            } else {
                let has_value = self
                    .non_trivia_token_from(self.position)
                    .is_some_and(|(_, kind)| !value_recovery.contains(&kind));
                self.builder.start_node(SyntaxKind::Value);
                if has_value {
                    self.parse_value_or_value_list_until(&value_recovery);
                } else {
                    self.empty_bogus_node(
                        SyntaxKind::BogusValue,
                        ParseErrorCode::ExpectedValue,
                        "expected variable value",
                    );
                }
                self.builder.finish_node();
            }
        } else {
            self.error_at_current(
                ParseErrorCode::UnexpectedCharacter,
                "expected variable declaration colon",
            );
            self.consume_until_recovery(&[
                SyntaxKind::Semicolon,
                SyntaxKind::SassOptionalSemicolon,
                SyntaxKind::RightBrace,
                SyntaxKind::SassDedent,
            ]);
        }
        if self.current_kind().is_some_and(is_statement_end) {
            self.token_current();
        }
        self.builder.finish_node();
    }

    fn parse_less_detached_ruleset(&mut self) {
        let closed = self.current_left_brace_has_match();
        self.builder.start_node(if closed {
            SyntaxKind::LessDetachedRulesetNode
        } else {
            SyntaxKind::BogusLessDetachedRuleset
        });
        if self.current_kind() == Some(SyntaxKind::LeftBrace) {
            self.token_current();
            self.builder.start_node(SyntaxKind::DeclarationList);
            self.parse_declaration_list();
            self.builder.finish_node();
        }
        if self.current_kind() == Some(SyntaxKind::RightBrace) {
            self.token_current();
        } else {
            self.error_at_current(
                ParseErrorCode::UnexpectedCharacter,
                "unterminated Less detached ruleset",
            );
        }
        self.builder.finish_node();
    }

    fn parse_declaration(&mut self) {
        let starts_composes = self.current_text() == Some("composes");
        let starts_custom_property = self.current_kind() == Some(SyntaxKind::CustomPropertyName);
        let has_colon = self.find_before_recovery(
            SyntaxKind::Colon,
            &[
                SyntaxKind::Semicolon,
                SyntaxKind::SassOptionalSemicolon,
                SyntaxKind::RightBrace,
                SyntaxKind::SassDedent,
                SyntaxKind::LeftBrace,
                SyntaxKind::SassIndent,
            ],
        );
        let kind = if starts_composes && has_colon {
            SyntaxKind::CssModuleComposesDeclaration
        } else if starts_composes {
            SyntaxKind::BogusComposesDeclaration
        } else if has_colon {
            SyntaxKind::Declaration
        } else {
            SyntaxKind::BogusDeclaration
        };
        self.builder.start_node(kind);
        if kind == SyntaxKind::CssModuleComposesDeclaration
            && self.current_css_module_scope_context() == Some("global")
        {
            self.error_at_current(
                ParseErrorCode::UnexpectedCharacter,
                "composes is not allowed inside :global scope",
            );
        }
        let property_kind = if matches!(
            self.current_kind(),
            Some(
                SyntaxKind::Colon
                    | SyntaxKind::Semicolon
                    | SyntaxKind::SassOptionalSemicolon
                    | SyntaxKind::LeftBrace
                    | SyntaxKind::SassIndent
                    | SyntaxKind::RightBrace
                    | SyntaxKind::SassDedent
            )
        ) {
            SyntaxKind::BogusPropertyName
        } else {
            SyntaxKind::PropertyName
        };
        self.builder.start_node(property_kind);
        while !self.at_end() {
            match self.current_kind() {
                Some(
                    SyntaxKind::Colon
                    | SyntaxKind::Semicolon
                    | SyntaxKind::SassOptionalSemicolon
                    | SyntaxKind::RightBrace
                    | SyntaxKind::SassDedent,
                ) => break,
                Some(kind) if is_interpolation_start(kind) => self.parse_interpolation(
                    kind,
                    &[
                        SyntaxKind::Colon,
                        SyntaxKind::Semicolon,
                        SyntaxKind::SassOptionalSemicolon,
                        SyntaxKind::RightBrace,
                        SyntaxKind::SassDedent,
                    ],
                ),
                Some(_) => self.token_current(),
                None => break,
            }
        }
        self.builder.finish_node();
        if property_kind == SyntaxKind::BogusPropertyName {
            self.error_at_current(
                ParseErrorCode::UnexpectedCharacter,
                "expected declaration property name",
            );
        }

        if self.current_kind() == Some(SyntaxKind::Colon) {
            self.token_current();
            let value_recovery = [
                SyntaxKind::Semicolon,
                SyntaxKind::SassOptionalSemicolon,
                SyntaxKind::RightBrace,
                SyntaxKind::SassDedent,
            ];
            let has_value = self
                .non_trivia_token_from(self.position)
                .is_some_and(|(_, kind)| !value_recovery.contains(&kind));
            self.builder.start_node(SyntaxKind::Value);
            if kind == SyntaxKind::CssModuleComposesDeclaration {
                self.parse_composes_value_until(&value_recovery);
            } else if starts_custom_property {
                self.builder.start_node(SyntaxKind::CustomPropertyValue);
                self.parse_component_value_list_until(&value_recovery);
                self.builder.finish_node();
            } else if !has_value {
                self.empty_bogus_node(
                    SyntaxKind::BogusValue,
                    ParseErrorCode::ExpectedValue,
                    "expected declaration value",
                );
            } else {
                self.parse_declaration_value_or_value_list_until(&value_recovery);
            }
            self.builder.finish_node();
        } else {
            self.consume_until_recovery(&[
                SyntaxKind::Semicolon,
                SyntaxKind::SassOptionalSemicolon,
                SyntaxKind::RightBrace,
                SyntaxKind::SassDedent,
            ]);
        }

        if self.current_kind().is_some_and(is_statement_end) {
            self.token_current();
        }
        self.builder.finish_node();
    }

    fn parse_composes_value_until(&mut self, recovery: &[SyntaxKind]) {
        let mut saw_target = false;
        if self.current_composes_value_has_multiple_from_clauses(recovery) {
            self.error_at_current(
                ParseErrorCode::UnexpectedCharacter,
                "multiple composes from clauses are not allowed",
            );
        }
        while !self.at_end() {
            self.eat_value_trivia();
            match self.current_kind() {
                Some(kind) if recovery.contains(&kind) => break,
                Some(SyntaxKind::Ident) if self.current_text() == Some("from") => {
                    if !saw_target {
                        self.empty_bogus_node(
                            SyntaxKind::BogusComposesTarget,
                            ParseErrorCode::UnexpectedCharacter,
                            "expected composes target before from clause",
                        );
                        saw_target = true;
                    }
                    self.parse_css_module_from_clause(recovery);
                }
                Some(SyntaxKind::Ident | SyntaxKind::CustomPropertyName) => {
                    self.builder.start_node(SyntaxKind::CssModuleComposesTarget);
                    self.token_current();
                    self.builder.finish_node();
                    saw_target = true;
                }
                Some(kind) if is_interpolation_start(kind) => {
                    self.parse_interpolation(kind, recovery)
                }
                Some(_) => self.token_current(),
                None => break,
            }
        }
        if !saw_target {
            self.empty_bogus_node(
                SyntaxKind::BogusComposesTarget,
                ParseErrorCode::UnexpectedCharacter,
                "expected composes target",
            );
        }
    }

    fn current_composes_value_has_multiple_from_clauses(&self, recovery: &[SyntaxKind]) -> bool {
        let mut index = self.position;
        let mut paren_depth = 0usize;
        let mut bracket_depth = 0usize;
        let mut brace_depth = 0usize;
        let mut from_count = 0usize;
        while let Some(token) = self.tokens.get(index) {
            if paren_depth == 0
                && bracket_depth == 0
                && brace_depth == 0
                && recovery.contains(&token.kind)
            {
                break;
            }
            match token.kind {
                SyntaxKind::LeftParen => paren_depth += 1,
                SyntaxKind::RightParen => paren_depth = paren_depth.saturating_sub(1),
                SyntaxKind::LeftBracket => bracket_depth += 1,
                SyntaxKind::RightBracket => bracket_depth = bracket_depth.saturating_sub(1),
                SyntaxKind::LeftBrace => brace_depth += 1,
                SyntaxKind::RightBrace => brace_depth = brace_depth.saturating_sub(1),
                SyntaxKind::Ident
                    if paren_depth == 0
                        && bracket_depth == 0
                        && brace_depth == 0
                        && token.text == "from" =>
                {
                    from_count += 1;
                    if from_count > 1 {
                        return true;
                    }
                }
                _ => {}
            }
            index += 1;
        }
        false
    }

    fn parse_css_module_from_clause(&mut self, recovery: &[SyntaxKind]) {
        let source = self.non_trivia_token_from(self.position + 1);
        let has_source = source.is_some_and(|(_, kind)| !recovery.contains(&kind));
        let has_valid_source = source.is_some_and(|(index, kind)| {
            self.tokens
                .get(index)
                .is_some_and(|token| is_css_module_from_source_token(kind, token.text))
        });
        self.builder.start_node(if has_valid_source {
            SyntaxKind::CssModuleFromClause
        } else {
            SyntaxKind::BogusFromClause
        });
        self.token_current();
        while !self.at_end() {
            match self.current_kind() {
                Some(kind) if recovery.contains(&kind) => break,
                Some(_) => self.token_current(),
                None => break,
            }
        }
        if !has_source {
            self.error_at_current(
                ParseErrorCode::UnexpectedCharacter,
                "expected CSS Modules from-clause source",
            );
        } else if !has_valid_source {
            self.error_at_current(
                ParseErrorCode::ExpectedValue,
                "invalid CSS Modules from-clause source",
            );
        }
        self.builder.finish_node();
    }

    fn current_css_module_scope_context(&self) -> Option<&'static str> {
        let mut open_blocks = Vec::new();
        for (index, token) in self.tokens.iter().take(self.position).enumerate() {
            match token.kind {
                SyntaxKind::LeftBrace | SyntaxKind::SassIndent => open_blocks.push(index),
                SyntaxKind::RightBrace | SyntaxKind::SassDedent => {
                    open_blocks.pop();
                }
                _ => {}
            }
        }

        if let Some(scope) = open_blocks.iter().copied().find_map(|block_start| {
            let header_start = self.header_start_for_block(block_start);
            css_module_block_scope_marker_in_header(&self.tokens, header_start, block_start)
        }) {
            return Some(scope);
        }

        let block_start = open_blocks.last().copied()?;
        let header_start = self.header_start_for_block(block_start);
        css_module_header_is_global_only(&self.tokens, header_start, block_start)
            .then_some("global")
    }

    fn header_start_for_block(&self, block_start: usize) -> usize {
        let mut index = block_start;
        while index > 0 {
            let previous = index - 1;
            if matches!(
                self.tokens[previous].kind,
                SyntaxKind::LeftBrace
                    | SyntaxKind::RightBrace
                    | SyntaxKind::SassIndent
                    | SyntaxKind::SassDedent
                    | SyntaxKind::Semicolon
                    | SyntaxKind::SassOptionalSemicolon
            ) {
                break;
            }
            index = previous;
        }
        index
    }

    fn parse_dialect_at_rule(&mut self) {
        let Some(spec) = self.current_dialect_at_rule_spec() else {
            self.parse_at_rule();
            return;
        };

        self.builder
            .start_node(self.current_dialect_at_rule_node_kind(spec));
        if self.current_kind() == Some(SyntaxKind::AtKeyword) {
            self.token_current();
        }
        if matches!(
            spec.node_kind,
            SyntaxKind::ScssUseRule | SyntaxKind::ScssForwardRule
        ) {
            self.parse_scss_module_prelude(spec.node_kind);
        }
        if is_scss_control_rule_kind(spec.node_kind)
            && !self.current_scss_control_prelude_is_valid(spec.node_kind)
        {
            self.error_at_current(
                ParseErrorCode::ExpectedValue,
                "invalid SCSS control prelude",
            );
        }
        while !self.at_end() {
            match self.current_kind() {
                Some(kind) if is_statement_end(kind) => {
                    self.token_current();
                    break;
                }
                Some(SyntaxKind::LeftBrace) => {
                    match spec.block_kind {
                        AtRuleBlockKind::GroupRuleList => self.parse_group_at_rule_block(),
                        AtRuleBlockKind::DeclarationList => self.parse_declaration_block(),
                        AtRuleBlockKind::Keyframes => self.parse_keyframes_block(),
                        AtRuleBlockKind::Raw => self.consume_balanced_block(),
                    }
                    break;
                }
                Some(SyntaxKind::SassIndent) => {
                    self.parse_sass_indented_at_rule_block(spec.block_kind);
                    break;
                }
                Some(_) => self.token_current(),
                None => break,
            }
        }
        self.builder.finish_node();
    }

    fn parse_scss_module_prelude(&mut self, node_kind: SyntaxKind) {
        self.validate_scss_module_prelude(node_kind);
        while !self.at_end() {
            match self.current_kind() {
                Some(kind)
                    if is_statement_end(kind)
                        || kind == SyntaxKind::LeftBrace
                        || kind == SyntaxKind::SassIndent =>
                {
                    break;
                }
                Some(SyntaxKind::Ident | SyntaxKind::KeywordWith)
                    if self.current_text() == Some("with")
                        && self
                            .non_trivia_token_from(self.position + 1)
                            .is_some_and(|(_, kind)| kind == SyntaxKind::LeftParen) =>
                {
                    self.parse_scss_module_config()
                }
                Some(kind) if is_interpolation_start(kind) => self.parse_interpolation(
                    kind,
                    &[
                        SyntaxKind::Semicolon,
                        SyntaxKind::SassOptionalSemicolon,
                        SyntaxKind::LeftBrace,
                        SyntaxKind::SassIndent,
                    ],
                ),
                Some(_) => self.token_current(),
                None => break,
            }
        }
    }

    fn validate_scss_module_prelude(&mut self, node_kind: SyntaxKind) {
        let recovery = [
            SyntaxKind::Semicolon,
            SyntaxKind::SassOptionalSemicolon,
            SyntaxKind::LeftBrace,
            SyntaxKind::SassIndent,
        ];
        let Some((source_index, source_kind)) = self.non_trivia_token_from(self.position) else {
            self.error_at_current(ParseErrorCode::ExpectedValue, "expected SCSS module source");
            return;
        };
        if recovery.contains(&source_kind) || !is_scss_module_source_token(source_kind) {
            let range = self
                .tokens
                .get(source_index)
                .map(|token| token.range)
                .unwrap_or_else(|| self.current_range());
            self.errors.push(ParseError {
                code: ParseErrorCode::ExpectedValue,
                range,
                message: "expected SCSS module source",
            });
        }

        let mut index = source_index;
        while let Some(token) = self.tokens.get(index).copied() {
            if recovery.contains(&token.kind) {
                break;
            }
            if token.kind == SyntaxKind::Ident {
                if token.text.eq_ignore_ascii_case("as") {
                    let next_kind = self.non_trivia_token_from(index + 1).map(|(_, kind)| kind);
                    if next_kind.is_none_or(|kind| {
                        recovery.contains(&kind) || !is_scss_module_namespace_token(kind)
                    }) {
                        self.errors.push(ParseError {
                            code: ParseErrorCode::ExpectedValue,
                            range: token.range,
                            message: "expected SCSS module namespace",
                        });
                    }
                } else if token.text.eq_ignore_ascii_case("with") {
                    let next_kind = self.non_trivia_token_from(index + 1).map(|(_, kind)| kind);
                    if next_kind != Some(SyntaxKind::LeftParen) {
                        self.errors.push(ParseError {
                            code: ParseErrorCode::ExpectedValue,
                            range: token.range,
                            message: "expected SCSS module configuration",
                        });
                    }
                } else if matches_ignore_ascii_case(token.text, &["show", "hide"]) {
                    if node_kind != SyntaxKind::ScssForwardRule {
                        self.errors.push(ParseError {
                            code: ParseErrorCode::UnexpectedCharacter,
                            range: token.range,
                            message: "unexpected SCSS module visibility clause",
                        });
                    }
                    let next_kind = self.non_trivia_token_from(index + 1).map(|(_, kind)| kind);
                    if next_kind.is_none_or(|kind| {
                        recovery.contains(&kind) || !is_scss_module_visibility_name_token(kind)
                    }) {
                        self.errors.push(ParseError {
                            code: ParseErrorCode::ExpectedValue,
                            range: token.range,
                            message: "expected SCSS module visibility name",
                        });
                    }
                }
            }
            index += 1;
        }
    }

    fn current_scss_control_prelude_is_valid(&self, node_kind: SyntaxKind) -> bool {
        let recovery = [
            SyntaxKind::LeftBrace,
            SyntaxKind::SassIndent,
            SyntaxKind::Semicolon,
            SyntaxKind::SassOptionalSemicolon,
            SyntaxKind::RightBrace,
            SyntaxKind::SassDedent,
        ];
        match node_kind {
            SyntaxKind::ScssControlIf | SyntaxKind::ScssControlWhile => self
                .non_trivia_token_from(self.position)
                .is_some_and(|(_, kind)| !recovery.contains(&kind)),
            SyntaxKind::ScssControlFor => {
                self.non_trivia_token_from(self.position)
                    .is_some_and(|(_, kind)| kind == SyntaxKind::ScssVariable)
                    && self.find_text_before_recovery("from", &recovery)
                    && (self.find_text_before_recovery("to", &recovery)
                        || self.find_text_before_recovery("through", &recovery))
            }
            SyntaxKind::ScssControlEach => {
                self.non_trivia_token_from(self.position)
                    .is_some_and(|(_, kind)| kind == SyntaxKind::ScssVariable)
                    && self.find_text_before_recovery("in", &recovery)
            }
            SyntaxKind::ScssControlElse => true,
            _ => true,
        }
    }

    fn parse_scss_module_config(&mut self) {
        let has_balanced_config = self.current_scss_module_config_has_balanced_parens();
        self.builder.start_node(if has_balanced_config {
            SyntaxKind::ScssModuleConfig
        } else {
            SyntaxKind::BogusScssModuleConfig
        });
        self.token_current();
        self.eat_trivia();
        if self.current_kind() == Some(SyntaxKind::LeftParen) {
            self.parse_balanced_parenthesized_prelude_until(
                None,
                &[
                    SyntaxKind::LeftBrace,
                    SyntaxKind::SassIndent,
                    SyntaxKind::Semicolon,
                    SyntaxKind::SassOptionalSemicolon,
                ],
            );
        }
        self.builder.finish_node();
    }

    fn parse_css_module_value_rule(&mut self) {
        let has_name = self
            .non_trivia_token_from(self.position + 1)
            .and_then(|(index, kind)| {
                self.tokens
                    .get(index)
                    .map(|token| (kind, token.text != "from"))
            })
            .is_some_and(|(kind, allowed_name)| {
                allowed_name && matches!(kind, SyntaxKind::Ident | SyntaxKind::CustomPropertyName)
            });
        let has_from = self.find_text_before_recovery(
            "from",
            &[
                SyntaxKind::Semicolon,
                SyntaxKind::SassOptionalSemicolon,
                SyntaxKind::LeftBrace,
                SyntaxKind::SassIndent,
            ],
        );
        let has_colon = self.find_before_recovery(
            SyntaxKind::Colon,
            &[
                SyntaxKind::Semicolon,
                SyntaxKind::SassOptionalSemicolon,
                SyntaxKind::LeftBrace,
                SyntaxKind::SassIndent,
            ],
        );
        let kind = if !has_name {
            SyntaxKind::BogusCssModuleBlock
        } else if has_from && !has_colon {
            SyntaxKind::CssModuleImportBlock
        } else {
            SyntaxKind::CssModuleExportBlock
        };

        self.builder.start_node(kind);
        self.token_current();
        if !has_name {
            self.error_at_current(
                ParseErrorCode::UnexpectedCharacter,
                "expected CSS Modules @value name",
            );
        }
        if has_colon {
            self.parse_css_module_value_export();
        } else {
            self.parse_css_module_value_import_or_statement();
        }
        if self.current_kind().is_some_and(is_statement_end) {
            self.token_current();
        }
        self.builder.finish_node();
    }

    fn parse_css_module_value_export(&mut self) {
        self.parse_css_module_token_definitions_until(&[
            SyntaxKind::Colon,
            SyntaxKind::Semicolon,
            SyntaxKind::SassOptionalSemicolon,
        ]);
        if self.current_kind() == Some(SyntaxKind::Colon) {
            self.token_current();
            self.builder.start_node(SyntaxKind::Value);
            self.parse_css_module_token_references_until(&[
                SyntaxKind::Semicolon,
                SyntaxKind::SassOptionalSemicolon,
            ]);
            self.builder.finish_node();
        }
    }

    fn parse_css_module_value_import_or_statement(&mut self) {
        self.parse_css_module_token_definitions_until(&[
            SyntaxKind::Semicolon,
            SyntaxKind::SassOptionalSemicolon,
        ]);
    }

    fn parse_css_module_token_definitions_until(&mut self, recovery: &[SyntaxKind]) {
        while !self.at_end() {
            match self.current_kind() {
                Some(kind) if recovery.contains(&kind) => break,
                Some(SyntaxKind::Ident) if self.current_text() == Some("from") => {
                    self.parse_css_module_from_clause(recovery);
                    break;
                }
                Some(SyntaxKind::Ident | SyntaxKind::CustomPropertyName) => {
                    self.builder.start_node(SyntaxKind::TokenDefinition);
                    self.token_current();
                    self.builder.finish_node();
                }
                Some(_) => self.token_current(),
                None => break,
            }
        }
    }

    fn parse_css_module_token_references_until(&mut self, recovery: &[SyntaxKind]) {
        while !self.at_end() {
            self.eat_value_trivia();
            match self.current_kind() {
                Some(kind) if recovery.contains(&kind) => break,
                Some(SyntaxKind::Ident | SyntaxKind::CustomPropertyName) => {
                    self.builder.start_node(SyntaxKind::TokenReference);
                    self.token_current();
                    self.builder.finish_node();
                }
                Some(kind) if is_interpolation_start(kind) => {
                    self.parse_interpolation(kind, recovery)
                }
                Some(_) => self.token_current(),
                None => break,
            }
        }
    }

    fn parse_less_mixin_header(&mut self) {
        self.builder.start_node(SyntaxKind::SelectorList);
        self.parse_until_recovery_with_optional_less_guard(&[SyntaxKind::LeftBrace]);
        self.builder.finish_node();
    }

    fn parse_less_mixin_call(&mut self) {
        self.builder.start_node(SyntaxKind::LessMixinCall);
        self.parse_until_recovery_with_optional_less_guard(&[
            SyntaxKind::Semicolon,
            SyntaxKind::SassOptionalSemicolon,
            SyntaxKind::RightBrace,
            SyntaxKind::SassDedent,
        ]);
        if self.current_kind().is_some_and(is_statement_end) {
            self.token_current();
        }
        self.builder.finish_node();
    }

    fn parse_less_namespace_access(&mut self) {
        self.builder.start_node(SyntaxKind::LessNamespaceAccess);
        while !self.at_end() {
            match self.current_kind() {
                Some(
                    SyntaxKind::Semicolon
                    | SyntaxKind::SassOptionalSemicolon
                    | SyntaxKind::RightBrace
                    | SyntaxKind::SassDedent
                    | SyntaxKind::LeftBrace
                    | SyntaxKind::SassIndent,
                ) => break,
                Some(_) if self.current_starts_less_mixin_call() => {
                    self.parse_less_mixin_call();
                    break;
                }
                Some(_) => self.token_current(),
                None => break,
            }
        }
        if self.current_kind().is_some_and(is_statement_end) {
            self.token_current();
        }
        self.builder.finish_node();
    }

    fn parse_until_recovery_with_optional_less_guard(&mut self, recovery: &[SyntaxKind]) {
        let mut guard_open = false;
        while !self.at_end() {
            match self.current_kind() {
                Some(kind) if recovery.contains(&kind) => break,
                Some(SyntaxKind::Ident) if self.current_text() == Some("when") && !guard_open => {
                    self.builder.start_node(
                        if self.current_less_guard_has_condition_before(recovery) {
                            SyntaxKind::LessMixinGuard
                        } else {
                            SyntaxKind::BogusLessGuard
                        },
                    );
                    guard_open = true;
                    self.token_current();
                }
                Some(_) => self.token_current(),
                None => break,
            }
        }
        if guard_open {
            self.builder.finish_node();
        }
    }

    fn parse_value_until(&mut self, recovery: &[SyntaxKind]) {
        while !self.at_end() {
            self.eat_value_trivia();
            if matches!(self.current_kind(), Some(kind) if recovery.contains(&kind)) {
                break;
            }
            if self.at_end() {
                break;
            }
            self.parse_value_expression(0, recovery);
        }
    }

    fn parse_value_or_value_list_until(&mut self, recovery: &[SyntaxKind]) {
        if self.current_value_has_top_level_comma_before(recovery) {
            self.parse_value_list_until(recovery);
        } else {
            self.parse_value_until(recovery);
        }
    }

    fn parse_declaration_value_or_value_list_until(&mut self, recovery: &[SyntaxKind]) {
        if self.current_value_has_top_level_comma_before(recovery) {
            self.parse_declaration_value_list_until(recovery);
        } else {
            self.parse_declaration_value_until(recovery);
        }
    }

    fn parse_declaration_value_until(&mut self, recovery: &[SyntaxKind]) {
        let mut saw_value = false;
        while !self.at_end() {
            self.eat_value_trivia();
            if matches!(self.current_kind(), Some(kind) if recovery.contains(&kind)) {
                break;
            }
            if saw_value && self.current_starts_missing_semicolon_declaration(recovery) {
                self.error_at_current(
                    ParseErrorCode::UnexpectedCharacter,
                    "expected semicolon between declarations",
                );
                break;
            }
            if self.at_end() {
                break;
            }
            self.parse_value_expression(0, recovery);
            saw_value = true;
        }
    }

    fn parse_declaration_value_list_until(&mut self, recovery: &[SyntaxKind]) {
        self.builder
            .start_node(if self.current_value_list_is_bogus(recovery) {
                SyntaxKind::BogusValueList
            } else {
                SyntaxKind::ValueList
            });
        let item_recovery = value_list_item_recovery(recovery);
        let mut saw_item = false;
        while !self.at_end() {
            self.eat_value_trivia();
            match self.current_kind() {
                Some(kind) if recovery.contains(&kind) => break,
                Some(SyntaxKind::Comma) => self.token_current(),
                Some(_)
                    if saw_item && self.current_starts_missing_semicolon_declaration(recovery) =>
                {
                    self.error_at_current(
                        ParseErrorCode::UnexpectedCharacter,
                        "expected semicolon between declarations",
                    );
                    break;
                }
                Some(_) => {
                    self.parse_value_expression(0, &item_recovery);
                    saw_item = true;
                }
                None => break,
            }
        }
        self.builder.finish_node();
    }

    fn parse_value_list_until(&mut self, recovery: &[SyntaxKind]) {
        self.builder
            .start_node(if self.current_value_list_is_bogus(recovery) {
                SyntaxKind::BogusValueList
            } else {
                SyntaxKind::ValueList
            });
        let item_recovery = value_list_item_recovery(recovery);
        while !self.at_end() {
            self.eat_value_trivia();
            match self.current_kind() {
                Some(kind) if recovery.contains(&kind) => break,
                Some(SyntaxKind::Comma) => self.token_current(),
                Some(_) => self.parse_value_expression(0, &item_recovery),
                None => break,
            }
        }
        self.builder.finish_node();
    }

    fn parse_component_value(&mut self, recovery: &[SyntaxKind]) {
        self.builder.start_node(SyntaxKind::ComponentValue);
        self.parse_component_value_inner(recovery);
        self.builder.finish_node();
    }

    fn parse_component_value_list_until(&mut self, recovery: &[SyntaxKind]) {
        self.builder.start_node(SyntaxKind::ComponentValueList);
        while !self.at_end() {
            self.eat_value_trivia();
            match self.current_kind() {
                Some(kind) if recovery.contains(&kind) => break,
                Some(_) => self.parse_component_value(recovery),
                None => break,
            }
        }
        self.builder.finish_node();
    }

    fn parse_comma_separated_component_value_list_until(&mut self, recovery: &[SyntaxKind]) {
        self.builder
            .start_node(SyntaxKind::CommaSeparatedComponentValueList);
        let item_recovery = comma_separated_component_value_list_item_recovery(recovery);
        while !self.at_end() {
            self.eat_value_trivia();
            match self.current_kind() {
                Some(kind) if recovery.contains(&kind) => break,
                Some(SyntaxKind::Comma) => self.token_current(),
                Some(_) => self.parse_component_value(&item_recovery),
                None => break,
            }
        }
        self.builder.finish_node();
    }

    fn parse_component_value_inner(&mut self, recovery: &[SyntaxKind]) {
        self.eat_value_trivia();
        match self.current_kind() {
            Some(kind) if recovery.contains(&kind) => {
                self.empty_bogus_node(
                    SyntaxKind::BogusValue,
                    ParseErrorCode::ExpectedValue,
                    "expected component value",
                );
            }
            Some(SyntaxKind::LeftBrace | SyntaxKind::LeftBracket | SyntaxKind::LeftParen) => {
                self.parse_simple_block(recovery)
            }
            Some(SyntaxKind::Ident) if self.next_kind() == Some(SyntaxKind::LeftParen) => {
                self.parse_function_call(recovery)
            }
            Some(kind) if is_component_value_atom_start(kind) => self.parse_value_prefix(recovery),
            Some(_) => self.token_current(),
            None => {
                self.empty_bogus_node(
                    SyntaxKind::BogusValue,
                    ParseErrorCode::ExpectedValue,
                    "expected component value",
                );
            }
        }
    }

    fn parse_simple_block_entry_point(&mut self, recovery: &[SyntaxKind]) {
        self.eat_value_trivia();
        match self.current_kind() {
            Some(SyntaxKind::LeftBrace | SyntaxKind::LeftBracket | SyntaxKind::LeftParen) => {
                self.parse_simple_block(recovery)
            }
            Some(_) | None => {
                self.empty_bogus_node(
                    SyntaxKind::BogusSimpleBlock,
                    ParseErrorCode::ExpectedValue,
                    "expected simple block",
                );
            }
        }
    }

    fn parse_simple_block(&mut self, recovery: &[SyntaxKind]) {
        let Some(open_kind) = self.current_kind() else {
            self.empty_bogus_node(
                SyntaxKind::BogusSimpleBlock,
                ParseErrorCode::ExpectedValue,
                "expected simple block",
            );
            return;
        };
        let Some(close_kind) = matching_simple_block_close(open_kind) else {
            self.empty_bogus_node(
                SyntaxKind::BogusSimpleBlock,
                ParseErrorCode::ExpectedValue,
                "expected simple block",
            );
            return;
        };

        let block_kind = if self.current_simple_block_has_matching_close(recovery) {
            SyntaxKind::SimpleBlock
        } else {
            SyntaxKind::BogusSimpleBlock
        };
        self.builder.start_node(block_kind);
        self.token_current();

        let block_recovery = simple_block_recovery(close_kind, recovery);
        while !self.at_end() {
            self.eat_value_trivia();
            match self.current_kind() {
                Some(kind) if kind == close_kind => break,
                Some(kind) if recovery.contains(&kind) => break,
                Some(_) => self.parse_component_value(&block_recovery),
                None => break,
            }
        }

        if self.current_kind() == Some(close_kind) {
            self.token_current();
        } else {
            self.error_at_current(
                ParseErrorCode::UnexpectedCharacter,
                "unterminated simple block",
            );
        }
        self.builder.finish_node();
    }

    fn parse_value_expression(&mut self, min_binding_power: u8, recovery: &[SyntaxKind]) {
        self.eat_value_trivia();
        let checkpoint = self.builder.checkpoint();
        self.parse_value_prefix(recovery);

        loop {
            self.eat_value_trivia();
            let Some(operator) = self.current_kind() else {
                break;
            };
            if recovery.contains(&operator) {
                break;
            }
            let Some((left_binding_power, right_binding_power)) = infix_binding_power(operator)
            else {
                break;
            };
            if left_binding_power < min_binding_power {
                break;
            }

            self.builder
                .start_node_at(checkpoint, SyntaxKind::BinaryExpression);
            self.token_current();
            self.parse_value_expression(right_binding_power, recovery);
            self.builder.finish_node();
        }
    }

    fn parse_value_prefix(&mut self, recovery: &[SyntaxKind]) {
        match self.current_kind() {
            Some(SyntaxKind::Plus | SyntaxKind::Minus) => {
                self.builder.start_node(SyntaxKind::UnaryExpression);
                self.token_current();
                self.parse_value_expression(5, recovery);
                self.builder.finish_node();
            }
            Some(SyntaxKind::Ident)
                if self
                    .current_text()
                    .is_some_and(|text| text.eq_ignore_ascii_case("url"))
                    && self.next_kind() == Some(SyntaxKind::LeftParen) =>
            {
                self.builder.start_node(SyntaxKind::UrlValue);
                self.parse_function_call(recovery);
                self.builder.finish_node();
            }
            Some(SyntaxKind::Ident) if self.next_kind() == Some(SyntaxKind::LeftParen) => {
                self.parse_function_call(recovery)
            }
            Some(SyntaxKind::Number) => {
                self.builder.start_node(SyntaxKind::NumberValue);
                self.token_current();
                self.builder.finish_node();
            }
            Some(SyntaxKind::Percentage) => {
                self.builder.start_node(SyntaxKind::PercentageValue);
                self.token_current();
                self.builder.finish_node();
            }
            Some(SyntaxKind::Dimension) => {
                self.builder.start_node(SyntaxKind::DimensionValue);
                self.token_current();
                self.builder.finish_node();
            }
            Some(SyntaxKind::Ident | SyntaxKind::CustomPropertyName) => {
                self.builder.start_node(SyntaxKind::IdentifierValue);
                self.token_current();
                self.builder.finish_node();
            }
            Some(SyntaxKind::String | SyntaxKind::LessEscapedString) => {
                self.builder.start_node(SyntaxKind::StringValue);
                self.token_current();
                self.builder.finish_node();
            }
            Some(SyntaxKind::UnicodeRange) => {
                self.builder.start_node(SyntaxKind::UnicodeRangeValue);
                self.token_current();
                self.builder.finish_node();
            }
            Some(SyntaxKind::Hash) => {
                self.builder.start_node(SyntaxKind::ColorValue);
                self.token_current();
                self.builder.finish_node();
            }
            Some(SyntaxKind::Url) => {
                self.builder.start_node(SyntaxKind::UrlValue);
                self.token_current();
                self.builder.finish_node();
            }
            Some(SyntaxKind::BadUrl) => {
                self.builder.start_node(SyntaxKind::BogusValue);
                self.token_current();
                self.builder.finish_node();
            }
            Some(SyntaxKind::BadString) => {
                self.builder.start_node(SyntaxKind::BogusValue);
                self.token_current();
                self.builder.finish_node();
            }
            Some(SyntaxKind::Important) => {
                self.builder.start_node(SyntaxKind::ImportantAnnotation);
                self.token_current();
                self.builder.finish_node();
            }
            Some(SyntaxKind::Delim) if self.current_split_important_annotation() => {
                self.parse_split_important_annotation()
            }
            Some(SyntaxKind::Delim) if self.current_scss_variable_flag_annotation() => {
                self.parse_scss_variable_flag_annotation()
            }
            Some(kind) if is_interpolation_start(kind) => self.parse_interpolation(kind, recovery),
            Some(SyntaxKind::ScssVariable) => {
                self.builder.start_node(SyntaxKind::ScssVariableReference);
                self.token_current();
                self.builder.finish_node();
            }
            Some(SyntaxKind::LessVariable) => {
                self.builder.start_node(SyntaxKind::LessVariableReference);
                self.token_current();
                self.builder.finish_node();
            }
            Some(SyntaxKind::LessPropertyVariableToken) => {
                self.builder.start_node(SyntaxKind::LessPropertyVariable);
                self.token_current();
                self.builder.finish_node();
            }
            Some(SyntaxKind::LeftBrace) => self.parse_simple_block(recovery),
            Some(SyntaxKind::LeftParen) => self.parse_parenthesized_expression(recovery),
            Some(SyntaxKind::LeftBracket) => self.parse_bracketed_value(recovery),
            Some(kind) if recovery.contains(&kind) => {
                self.empty_bogus_node(
                    SyntaxKind::BogusValue,
                    ParseErrorCode::ExpectedValue,
                    "expected value",
                );
            }
            Some(SyntaxKind::Delim) => {
                self.builder.start_node(SyntaxKind::BogusToken);
                self.token_current();
                self.builder.finish_node();
            }
            Some(_) => {
                self.builder.start_node(SyntaxKind::BogusValue);
                self.error_at_current(ParseErrorCode::ExpectedValue, "expected value");
                self.token_current();
                self.builder.finish_node();
            }
            None => {
                self.empty_bogus_node(
                    SyntaxKind::BogusValue,
                    ParseErrorCode::ExpectedValue,
                    "expected value",
                );
            }
        }
    }

    fn parse_split_important_annotation(&mut self) {
        self.builder.start_node(SyntaxKind::ImportantAnnotation);
        self.token_current();
        self.eat_value_trivia();
        if self
            .current_text()
            .is_some_and(|text| text.eq_ignore_ascii_case("important"))
        {
            self.token_current();
        }
        self.builder.finish_node();
    }

    fn parse_scss_variable_flag_annotation(&mut self) {
        self.builder.start_node(SyntaxKind::ScssVariableFlag);
        self.token_current();
        self.eat_value_trivia();
        self.token_current();
        self.builder.finish_node();
    }

    fn eat_value_trivia(&mut self) {
        while matches!(self.current_kind(), Some(kind) if kind.is_trivia()) {
            self.token_current();
        }
    }

    fn parse_function_call(&mut self, recovery: &[SyntaxKind]) {
        let function_name = self.current_text().map(str::to_owned);
        let function_range = self.current_range();
        let argument_count = self.current_function_top_level_argument_count_before(recovery);
        let has_empty_argument_slot =
            self.current_function_has_empty_top_level_argument_slot_before(recovery);
        let argument_head = self.current_function_first_argument_token_before(recovery);
        let specialized_kind = function_name.as_deref().and_then(specialized_function_kind);
        let closed = self.current_function_has_closing_paren_before(recovery);
        let function_kind = if closed {
            SyntaxKind::FunctionCall
        } else {
            SyntaxKind::BogusFunctionCall
        };
        let arguments_kind = if closed {
            SyntaxKind::FunctionArguments
        } else {
            SyntaxKind::BogusFunctionArguments
        };

        self.builder.start_node(function_kind);
        if let Some(kind) = specialized_kind {
            self.builder.start_node(kind);
        }
        self.token_current();
        if self.current_kind() == Some(SyntaxKind::LeftParen) {
            self.token_current();
            self.builder.start_node(arguments_kind);
            let argument_recovery = function_argument_recovery(recovery);
            self.parse_value_or_value_list_until(&argument_recovery);
            self.builder.finish_node();
            if self.current_kind() == Some(SyntaxKind::RightParen) {
                self.token_current();
            } else {
                self.error_at_current(
                    ParseErrorCode::UnexpectedCharacter,
                    "unterminated function call",
                );
            }
        }
        if let Some(function_name) = function_name {
            if let Some(argument_count) = argument_count {
                self.validate_function_argument_count(
                    &function_name,
                    argument_count,
                    function_range,
                );
            }
            if let Some(true) = has_empty_argument_slot {
                self.validate_function_argument_slots(&function_name, function_range);
            }
            self.validate_function_argument_head(&function_name, argument_head, function_range);
        }
        if specialized_kind.is_some() {
            self.builder.finish_node();
        }
        self.builder.finish_node();
    }

    fn current_function_top_level_argument_count_before(
        &self,
        recovery: &[SyntaxKind],
    ) -> Option<usize> {
        if self.next_kind() != Some(SyntaxKind::LeftParen) {
            return None;
        }

        let mut index = self.position + 2;
        let mut depth = 0usize;
        let mut comma_count = 0usize;
        let mut saw_argument = false;
        while let Some(token) = self.tokens.get(index) {
            match token.kind {
                kind if depth == 0 && recovery.contains(&kind) => return None,
                SyntaxKind::RightParen if depth == 0 => {
                    return Some(if saw_argument { comma_count + 1 } else { 0 });
                }
                SyntaxKind::Comma if depth == 0 => {
                    comma_count += 1;
                    saw_argument = false;
                }
                kind if kind.is_trivia() => {}
                SyntaxKind::LeftBrace | SyntaxKind::LeftBracket | SyntaxKind::LeftParen => {
                    depth += 1;
                    saw_argument = true;
                }
                SyntaxKind::RightBrace | SyntaxKind::RightBracket | SyntaxKind::RightParen => {
                    depth = depth.saturating_sub(1);
                    saw_argument = true;
                }
                _ => saw_argument = true,
            }
            index += 1;
        }
        None
    }

    fn current_function_has_empty_top_level_argument_slot_before(
        &self,
        recovery: &[SyntaxKind],
    ) -> Option<bool> {
        if self.next_kind() != Some(SyntaxKind::LeftParen) {
            return None;
        }

        let mut index = self.position + 2;
        let mut depth = 0usize;
        let mut expecting_argument = true;
        let mut saw_argument = false;
        while let Some(token) = self.tokens.get(index) {
            match token.kind {
                kind if depth == 0 && recovery.contains(&kind) => return None,
                SyntaxKind::RightParen if depth == 0 => {
                    return Some(expecting_argument && saw_argument);
                }
                SyntaxKind::Comma if depth == 0 => {
                    if expecting_argument {
                        return Some(true);
                    }
                    expecting_argument = true;
                }
                kind if kind.is_trivia() => {}
                SyntaxKind::LeftBrace | SyntaxKind::LeftBracket | SyntaxKind::LeftParen => {
                    depth += 1;
                    expecting_argument = false;
                    saw_argument = true;
                }
                SyntaxKind::RightBrace | SyntaxKind::RightBracket | SyntaxKind::RightParen => {
                    depth = depth.saturating_sub(1);
                    expecting_argument = false;
                    saw_argument = true;
                }
                _ => {
                    expecting_argument = false;
                    saw_argument = true;
                }
            }
            index += 1;
        }
        None
    }

    fn current_function_first_argument_token_before(
        &self,
        recovery: &[SyntaxKind],
    ) -> Option<Token<'text>> {
        if self.next_kind() != Some(SyntaxKind::LeftParen) {
            return None;
        }

        let mut index = self.position + 2;
        while let Some(token) = self.tokens.get(index).copied() {
            match token.kind {
                kind if recovery.contains(&kind) => return None,
                SyntaxKind::RightParen => return None,
                kind if kind.is_trivia() => {}
                _ => return Some(token),
            }
            index += 1;
        }
        None
    }

    fn validate_function_argument_count(
        &mut self,
        function_name: &str,
        argument_count: usize,
        range: TextRange,
    ) {
        if function_argument_count_is_valid(function_name, argument_count) {
            return;
        }
        self.errors.push(ParseError {
            code: ParseErrorCode::ExpectedValue,
            range,
            message: "invalid function argument count",
        });
    }

    fn validate_function_argument_slots(&mut self, function_name: &str, range: TextRange) {
        if !function_requires_filled_top_level_arguments(function_name) {
            return;
        }
        self.errors.push(ParseError {
            code: ParseErrorCode::ExpectedValue,
            range,
            message: "empty function argument",
        });
    }

    fn validate_function_argument_head(
        &mut self,
        function_name: &str,
        argument_head: Option<Token<'text>>,
        range: TextRange,
    ) {
        let head_kind = argument_head.map(|token| token.kind);
        let valid = if function_name.eq_ignore_ascii_case("var") {
            matches!(head_kind, Some(SyntaxKind::CustomPropertyName))
                || head_kind.is_some_and(is_dynamic_function_argument_head)
        } else if function_name.eq_ignore_ascii_case("env") {
            matches!(
                head_kind,
                Some(SyntaxKind::Ident | SyntaxKind::CustomPropertyName)
            ) || head_kind.is_some_and(is_dynamic_function_argument_head)
        } else if function_name.eq_ignore_ascii_case("attr") {
            matches!(head_kind, Some(SyntaxKind::Ident))
                || head_kind.is_some_and(is_dynamic_function_argument_head)
        } else if function_name.eq_ignore_ascii_case("color-mix") {
            argument_head.is_some_and(|token| token.text.eq_ignore_ascii_case("in"))
                || head_kind.is_some_and(is_dynamic_function_argument_head)
        } else {
            true
        };

        if valid {
            return;
        }
        self.errors.push(ParseError {
            code: ParseErrorCode::ExpectedValue,
            range,
            message: "invalid function argument head",
        });
    }

    fn parse_bracketed_value(&mut self, recovery: &[SyntaxKind]) {
        let closed = self.current_bracketed_value_has_closing_bracket_before(recovery);
        self.builder.start_node(if closed {
            SyntaxKind::BracketedValue
        } else {
            SyntaxKind::BogusBracketedValue
        });
        self.token_current();
        let bracket_recovery = bracketed_value_recovery(recovery);
        self.parse_value_until(&bracket_recovery);
        if self.current_kind() == Some(SyntaxKind::RightBracket) {
            self.token_current();
        } else {
            self.error_at_current(
                ParseErrorCode::UnexpectedCharacter,
                "unterminated bracketed value",
            );
        }
        self.builder.finish_node();
    }

    fn parse_parenthesized_expression(&mut self, recovery: &[SyntaxKind]) {
        self.builder.start_node(SyntaxKind::ParenthesizedExpression);
        self.token_current();
        let paren_recovery = function_argument_recovery(recovery);
        self.parse_value_until(&paren_recovery);
        if self.current_kind() == Some(SyntaxKind::RightParen) {
            self.token_current();
        }
        self.builder.finish_node();
    }

    fn parse_at_rule(&mut self) {
        let spec = self.current_text().and_then(at_rule_spec);
        let at_rule_kind = if spec.is_none() && self.current_text() == Some("@") {
            SyntaxKind::BogusAtRule
        } else {
            SyntaxKind::AtRule
        };
        self.builder.start_node(at_rule_kind);
        if at_rule_kind == SyntaxKind::BogusAtRule {
            self.error_at_current(ParseErrorCode::UnexpectedCharacter, "expected at-rule name");
        }
        if let Some(spec) = spec {
            self.builder.start_node(spec.node_kind);
        }

        if self.current_kind() == Some(SyntaxKind::AtKeyword) {
            self.token_current();
        }
        if let Some(spec) = spec {
            self.parse_at_rule_prelude(spec.node_kind);
        } else {
            self.consume_at_rule_prelude_tokens();
        }

        while !self.at_end() {
            match self.current_kind() {
                Some(kind) if is_statement_end(kind) => {
                    self.token_current();
                    break;
                }
                Some(SyntaxKind::LeftBrace) => {
                    match spec
                        .map(|spec| spec.block_kind)
                        .unwrap_or(AtRuleBlockKind::Raw)
                    {
                        AtRuleBlockKind::GroupRuleList => self.parse_group_at_rule_block(),
                        AtRuleBlockKind::DeclarationList => self.parse_declaration_block(),
                        AtRuleBlockKind::Keyframes => self.parse_keyframes_block(),
                        AtRuleBlockKind::Raw => self.consume_balanced_block(),
                    }
                    break;
                }
                Some(SyntaxKind::SassIndent) => {
                    self.parse_sass_indented_at_rule_block(
                        spec.map(|spec| spec.block_kind)
                            .unwrap_or(AtRuleBlockKind::Raw),
                    );
                    break;
                }
                Some(_) => self.token_current(),
                None => break,
            }
        }

        if spec.is_some() {
            self.builder.finish_node();
        }
        self.builder.finish_node();
    }

    fn parse_at_rule_prelude(&mut self, node_kind: SyntaxKind) {
        match node_kind {
            SyntaxKind::MediaRule => self.parse_media_query_list(),
            SyntaxKind::SupportsRule => self.parse_supports_rule_prelude(),
            SyntaxKind::ContainerRule => self.parse_container_rule_prelude(),
            SyntaxKind::ImportRule => self.parse_import_prelude(),
            SyntaxKind::CharsetRule => self.parse_charset_rule_prelude(),
            SyntaxKind::NamespaceRule => self.parse_namespace_rule_prelude(),
            SyntaxKind::KeyframesRule => self.parse_keyframes_rule_prelude(),
            SyntaxKind::PageRule => self.parse_page_rule_prelude(),
            SyntaxKind::FontFaceRule
            | SyntaxKind::StartingStyleRule
            | SyntaxKind::PageMarginRule
            | SyntaxKind::FontFeatureValuesStylisticRule
            | SyntaxKind::FontFeatureValuesStylesetRule
            | SyntaxKind::FontFeatureValuesCharacterVariantRule
            | SyntaxKind::FontFeatureValuesSwashRule
            | SyntaxKind::FontFeatureValuesOrnamentsRule
            | SyntaxKind::FontFeatureValuesAnnotationRule
            | SyntaxKind::FontFeatureValuesHistoricalFormsRule
            | SyntaxKind::ViewTransitionRule => {
                self.parse_empty_at_rule_prelude("unexpected at-rule prelude")
            }
            SyntaxKind::PropertyRule => self.parse_named_at_rule_prelude(
                at_rule_prelude_head_is_custom_property_name,
                "invalid @property name",
            ),
            SyntaxKind::FontPaletteValuesRule
            | SyntaxKind::ColorProfileRule
            | SyntaxKind::PositionTryRule => self.parse_named_at_rule_prelude(
                at_rule_prelude_head_is_custom_property_name,
                "invalid at-rule custom property name",
            ),
            SyntaxKind::CustomMediaRule => self.parse_custom_media_rule_prelude(),
            SyntaxKind::CounterStyleRule => self.parse_named_at_rule_prelude(
                at_rule_prelude_head_is_custom_ident,
                "invalid @counter-style name",
            ),
            SyntaxKind::FontFeatureValuesRule => self.parse_font_feature_values_prelude(),
            SyntaxKind::LayerRule => self.parse_layer_rule_prelude(),
            SyntaxKind::ScopeRule => self.parse_scope_rule_prelude(),
            _ => self.consume_at_rule_prelude_tokens(),
        }
    }

    fn parse_media_query_list(&mut self) {
        self.builder.start_node(SyntaxKind::MediaQueryList);
        let mut saw_query = false;
        let mut expecting_query = true;
        while !self.at_end() {
            match self.current_kind() {
                Some(kind) if is_at_rule_prelude_boundary(kind) => break,
                Some(SyntaxKind::Comma) => {
                    if expecting_query {
                        self.error_at_current(
                            ParseErrorCode::ExpectedValue,
                            "invalid @media prelude",
                        );
                        self.builder.start_node(SyntaxKind::BogusMediaQuery);
                        self.token_current();
                        self.builder.finish_node();
                    } else {
                        self.token_current();
                        expecting_query = true;
                    }
                }
                Some(_) => {
                    let valid = self.current_media_query_is_valid();
                    if !valid {
                        self.error_at_current(
                            ParseErrorCode::ExpectedValue,
                            "invalid @media prelude",
                        );
                    }
                    self.parse_media_query(valid);
                    saw_query = true;
                    expecting_query = false;
                }
                None => break,
            }
        }
        if !saw_query || expecting_query {
            self.error_at_current(ParseErrorCode::ExpectedValue, "invalid @media prelude");
            self.builder.start_node(SyntaxKind::BogusMediaQuery);
            self.builder.finish_node();
        }
        self.builder.finish_node();
    }

    fn parse_media_query(&mut self, valid: bool) {
        self.builder.start_node(if valid {
            SyntaxKind::MediaQuery
        } else {
            SyntaxKind::BogusMediaQuery
        });
        while !self.at_end() {
            match self.current_kind() {
                Some(kind) if is_at_rule_prelude_boundary(kind) || kind == SyntaxKind::Comma => {
                    break;
                }
                Some(SyntaxKind::LeftParen) => self.parse_balanced_parenthesized_prelude_until(
                    Some(SyntaxKind::MediaFeature),
                    &[
                        SyntaxKind::Comma,
                        SyntaxKind::LeftBrace,
                        SyntaxKind::Semicolon,
                    ],
                ),
                Some(kind) if is_interpolation_start(kind) => self.parse_interpolation(
                    kind,
                    &[
                        SyntaxKind::Comma,
                        SyntaxKind::LeftBrace,
                        SyntaxKind::Semicolon,
                    ],
                ),
                Some(_) => self.token_current(),
                None => break,
            }
        }
        self.builder.finish_node();
    }

    fn current_media_query_is_valid(&self) -> bool {
        let Some((first_index, first_kind)) = self.non_trivia_token_from(self.position) else {
            return false;
        };
        if is_at_rule_prelude_boundary(first_kind) || first_kind == SyntaxKind::Comma {
            return false;
        }
        if !self.current_prelude_parentheses_are_balanced_until(&[
            SyntaxKind::Comma,
            SyntaxKind::LeftBrace,
            SyntaxKind::SassIndent,
            SyntaxKind::Semicolon,
            SyntaxKind::SassOptionalSemicolon,
        ]) {
            return false;
        }
        self.media_query_starts_at(first_index, first_kind)
    }

    fn media_query_starts_at(&self, index: usize, kind: SyntaxKind) -> bool {
        match kind {
            SyntaxKind::Ident | SyntaxKind::LeftParen => true,
            SyntaxKind::KeywordNot | SyntaxKind::KeywordOnly => self
                .non_trivia_token_from(index + 1)
                .is_some_and(|(_, next_kind)| {
                    matches!(next_kind, SyntaxKind::Ident | SyntaxKind::LeftParen)
                        || is_interpolation_start(next_kind)
                }),
            kind if is_interpolation_start(kind) => true,
            _ => false,
        }
    }

    fn parse_charset_rule_prelude(&mut self) {
        if !self.charset_rule_prelude_is_valid() {
            self.error_at_current(ParseErrorCode::ExpectedValue, "invalid @charset prelude");
        }
        self.consume_at_rule_prelude_tokens();
    }

    fn charset_rule_prelude_is_valid(&self) -> bool {
        let Some((source_index, SyntaxKind::String)) = self.non_trivia_token_from(self.position)
        else {
            return false;
        };
        self.non_trivia_token_from(source_index + 1)
            .is_none_or(|(_, kind)| is_at_rule_prelude_boundary(kind))
    }

    fn parse_namespace_rule_prelude(&mut self) {
        if !self.namespace_rule_prelude_is_valid() {
            self.error_at_current(ParseErrorCode::ExpectedValue, "invalid @namespace prelude");
        }
        self.consume_at_rule_prelude_tokens();
    }

    fn parse_custom_media_rule_prelude(&mut self) {
        self.eat_trivia();
        let valid = self.custom_media_rule_prelude_is_valid();
        if !valid {
            self.error_at_current(
                ParseErrorCode::ExpectedValue,
                "invalid @custom-media prelude",
            );
        }
        self.builder.start_node(if valid {
            SyntaxKind::AtRulePrelude
        } else {
            SyntaxKind::BogusAtRulePrelude
        });
        self.consume_at_rule_prelude_tokens_without_wrapping();
        self.builder.finish_node();
    }

    fn custom_media_rule_prelude_is_valid(&self) -> bool {
        let Some((name_index, name_kind)) = self.non_trivia_token_from(self.position) else {
            return false;
        };
        if !self.current_prelude_parentheses_are_balanced_until(&[
            SyntaxKind::Semicolon,
            SyntaxKind::SassOptionalSemicolon,
        ]) {
            return false;
        }
        let tail = if name_kind == SyntaxKind::CustomPropertyName {
            self.non_trivia_token_from(name_index + 1)
        } else if is_interpolation_start(name_kind) {
            self.non_trivia_token_after_interpolation(name_index, name_kind)
        } else {
            return false;
        };
        let Some((tail_index, tail_kind)) = tail else {
            return false;
        };
        if is_at_rule_prelude_boundary(tail_kind) {
            return false;
        }
        self.media_query_starts_at(tail_index, tail_kind)
    }

    fn namespace_rule_prelude_is_valid(&self) -> bool {
        let Some((first_index, first_kind)) = self.non_trivia_token_from(self.position) else {
            return false;
        };

        if self.namespace_source_starts_at(first_index, first_kind) {
            return true;
        }
        if !matches!(
            first_kind,
            SyntaxKind::Ident | SyntaxKind::CustomPropertyName
        ) {
            return false;
        }
        self.non_trivia_token_from(first_index + 1)
            .is_some_and(|(source_index, source_kind)| {
                self.namespace_source_starts_at(source_index, source_kind)
            })
    }

    fn namespace_source_starts_at(&self, index: usize, kind: SyntaxKind) -> bool {
        matches!(kind, SyntaxKind::String | SyntaxKind::Url)
            || is_interpolation_start(kind)
            || self.token_starts_url_function(index, kind)
    }

    fn token_starts_url_function(&self, index: usize, kind: SyntaxKind) -> bool {
        kind == SyntaxKind::Ident
            && self
                .tokens
                .get(index)
                .is_some_and(|token| token.text.eq_ignore_ascii_case("url"))
            && self
                .non_trivia_token_from(index + 1)
                .is_some_and(|(_, next_kind)| next_kind == SyntaxKind::LeftParen)
    }

    fn parse_keyframes_rule_prelude(&mut self) {
        if !self.keyframes_rule_prelude_is_valid() {
            self.error_at_current(ParseErrorCode::ExpectedValue, "invalid @keyframes name");
        }
        self.consume_at_rule_prelude_tokens();
    }

    fn keyframes_rule_prelude_is_valid(&self) -> bool {
        let Some((name_index, name_kind)) = self.non_trivia_token_from(self.position) else {
            return false;
        };
        if is_interpolation_start(name_kind) {
            return true;
        }
        if !matches!(name_kind, SyntaxKind::Ident | SyntaxKind::String) {
            return false;
        }
        self.non_trivia_token_from(name_index + 1)
            .is_none_or(|(_, kind)| is_at_rule_prelude_boundary(kind))
    }

    fn parse_empty_at_rule_prelude(&mut self, message: &'static str) {
        self.eat_trivia();
        if self
            .current_kind()
            .is_some_and(|kind| !is_at_rule_prelude_boundary(kind))
        {
            self.error_at_current(ParseErrorCode::ExpectedValue, message);
            self.consume_at_rule_prelude_tokens();
        }
    }

    fn parse_font_feature_values_prelude(&mut self) {
        if !self.font_feature_values_prelude_is_valid() {
            self.error_at_current(
                ParseErrorCode::ExpectedValue,
                "invalid @font-feature-values family name",
            );
        }
        self.consume_at_rule_prelude_tokens();
    }

    fn font_feature_values_prelude_is_valid(&self) -> bool {
        self.non_trivia_token_from(self.position)
            .is_some_and(|(_, kind)| {
                matches!(kind, SyntaxKind::Ident | SyntaxKind::String)
                    || is_interpolation_start(kind)
            })
    }

    fn parse_layer_rule_prelude(&mut self) {
        self.eat_trivia();
        match self.current_kind() {
            Some(SyntaxKind::LeftBrace | SyntaxKind::SassIndent) => return,
            Some(SyntaxKind::Semicolon | SyntaxKind::SassOptionalSemicolon) | None => {
                self.empty_bogus_node(
                    SyntaxKind::BogusLayerName,
                    ParseErrorCode::ExpectedValue,
                    "invalid @layer prelude",
                );
                return;
            }
            Some(_) => {}
        }

        let valid = self.layer_rule_prelude_is_valid();
        if !valid {
            self.error_at_current(ParseErrorCode::ExpectedValue, "invalid @layer prelude");
        }
        self.builder.start_node(if valid {
            SyntaxKind::LayerName
        } else {
            SyntaxKind::BogusLayerName
        });
        self.consume_at_rule_prelude_tokens_without_wrapping();
        self.builder.finish_node();
    }

    fn layer_rule_prelude_is_valid(&self) -> bool {
        let mut saw_name = false;
        let mut expecting_segment = true;
        let mut index = self.position;

        while let Some(token) = self.tokens.get(index) {
            if token.kind.is_trivia() {
                index += 1;
                continue;
            }
            if is_at_rule_prelude_boundary(token.kind) {
                return saw_name && !expecting_segment;
            }
            if is_interpolation_start(token.kind) {
                return true;
            }
            match token.kind {
                SyntaxKind::Ident if expecting_segment => {
                    saw_name = true;
                    expecting_segment = false;
                }
                SyntaxKind::Comma if saw_name && !expecting_segment => {
                    expecting_segment = true;
                }
                SyntaxKind::Dot if saw_name && !expecting_segment => {
                    expecting_segment = true;
                }
                _ => return false,
            }
            index += 1;
        }

        saw_name && !expecting_segment
    }

    fn parse_container_rule_prelude(&mut self) {
        self.eat_trivia();
        let valid = self.container_rule_prelude_is_valid();
        if !valid {
            self.error_at_current(ParseErrorCode::ExpectedValue, "invalid @container prelude");
        }
        self.builder.start_node(if valid {
            SyntaxKind::ContainerCondition
        } else {
            SyntaxKind::BogusContainerCondition
        });
        self.consume_at_rule_prelude_tokens_without_wrapping();
        self.builder.finish_node();
    }

    fn container_rule_prelude_is_valid(&self) -> bool {
        let Some((first_index, first_kind)) = self.non_trivia_token_from(self.position) else {
            return false;
        };
        if is_at_rule_prelude_boundary(first_kind) {
            return false;
        }
        if !self.current_prelude_parentheses_are_balanced_until(&[
            SyntaxKind::LeftBrace,
            SyntaxKind::SassIndent,
            SyntaxKind::Semicolon,
            SyntaxKind::SassOptionalSemicolon,
        ]) {
            return false;
        }
        if self.container_condition_starts_at(first_index, first_kind) {
            return true;
        }
        if first_kind != SyntaxKind::Ident {
            return false;
        }
        self.non_trivia_token_from(first_index + 1).is_some_and(
            |(condition_index, condition_kind)| {
                self.container_condition_starts_at(condition_index, condition_kind)
            },
        )
    }

    fn container_condition_starts_at(&self, index: usize, kind: SyntaxKind) -> bool {
        if matches!(kind, SyntaxKind::LeftParen | SyntaxKind::KeywordNot)
            || is_interpolation_start(kind)
        {
            return true;
        }
        kind == SyntaxKind::Ident
            && self
                .non_trivia_token_from(index + 1)
                .is_some_and(|(_, next_kind)| next_kind == SyntaxKind::LeftParen)
    }

    fn parse_supports_rule_prelude(&mut self) {
        self.eat_trivia();
        let valid = self.supports_rule_prelude_is_valid();
        if !valid {
            self.error_at_current(ParseErrorCode::ExpectedValue, "invalid @supports prelude");
        }
        self.builder.start_node(if valid {
            SyntaxKind::SupportsCondition
        } else {
            SyntaxKind::BogusSupportsCondition
        });
        self.consume_at_rule_prelude_tokens_without_wrapping();
        self.builder.finish_node();
    }

    fn supports_rule_prelude_is_valid(&self) -> bool {
        let Some((first_index, first_kind)) = self.non_trivia_token_from(self.position) else {
            return false;
        };
        if is_at_rule_prelude_boundary(first_kind) {
            return false;
        }
        if !self.current_prelude_parentheses_are_balanced_until(&[
            SyntaxKind::LeftBrace,
            SyntaxKind::SassIndent,
            SyntaxKind::Semicolon,
            SyntaxKind::SassOptionalSemicolon,
        ]) {
            return false;
        }
        self.supports_condition_starts_at(first_index, first_kind)
    }

    fn supports_condition_starts_at(&self, index: usize, kind: SyntaxKind) -> bool {
        if kind == SyntaxKind::KeywordNot {
            return self
                .non_trivia_token_from(index + 1)
                .is_some_and(|(next_index, next_kind)| {
                    self.supports_condition_starts_at(next_index, next_kind)
                });
        }
        if kind == SyntaxKind::LeftParen || is_interpolation_start(kind) {
            return true;
        }
        kind == SyntaxKind::Ident
            && self
                .non_trivia_token_from(index + 1)
                .is_some_and(|(_, next_kind)| next_kind == SyntaxKind::LeftParen)
    }

    fn parse_scope_rule_prelude(&mut self) {
        self.eat_trivia();
        let valid = self.scope_rule_prelude_is_valid();
        if !valid {
            self.error_at_current(ParseErrorCode::ExpectedValue, "invalid @scope prelude");
        }
        self.builder.start_node(if valid {
            SyntaxKind::ScopeRange
        } else {
            SyntaxKind::BogusScopeRange
        });
        self.consume_at_rule_prelude_tokens_without_wrapping();
        self.builder.finish_node();
    }

    fn scope_rule_prelude_is_valid(&self) -> bool {
        let Some((start_index, start_kind)) = self.non_trivia_token_from(self.position) else {
            return false;
        };
        if is_at_rule_prelude_boundary(start_kind) {
            return false;
        }
        if !self.current_prelude_parentheses_are_balanced_until(&[
            SyntaxKind::LeftBrace,
            SyntaxKind::SassIndent,
            SyntaxKind::Semicolon,
            SyntaxKind::SassOptionalSemicolon,
        ]) {
            return false;
        }
        if is_interpolation_start(start_kind) {
            return true;
        }
        if start_kind != SyntaxKind::LeftParen {
            return false;
        }

        let Some(start_close_index) = self.parenthesized_prelude_close_index(start_index) else {
            return false;
        };
        let Some((after_start_index, after_start_kind)) =
            self.non_trivia_token_from(start_close_index + 1)
        else {
            return true;
        };
        if is_at_rule_prelude_boundary(after_start_kind) {
            return true;
        }
        if after_start_kind != SyntaxKind::Ident
            || !self
                .tokens
                .get(after_start_index)
                .is_some_and(|token| token.text.eq_ignore_ascii_case("to"))
        {
            return false;
        }

        let Some((end_index, end_kind)) = self.non_trivia_token_from(after_start_index + 1) else {
            return false;
        };
        if is_interpolation_start(end_kind) {
            return true;
        }
        if end_kind != SyntaxKind::LeftParen {
            return false;
        }
        let Some(end_close_index) = self.parenthesized_prelude_close_index(end_index) else {
            return false;
        };
        self.non_trivia_token_from(end_close_index + 1)
            .is_none_or(|(_, kind)| is_at_rule_prelude_boundary(kind))
    }

    fn parenthesized_prelude_close_index(&self, open_index: usize) -> Option<usize> {
        let mut depth = 0usize;
        for (index, token) in self.tokens.iter().enumerate().skip(open_index) {
            match token.kind {
                SyntaxKind::LeftParen => depth += 1,
                SyntaxKind::RightParen => {
                    depth = depth.saturating_sub(1);
                    if depth == 0 {
                        return Some(index);
                    }
                }
                kind if depth == 0 && is_at_rule_prelude_boundary(kind) => return None,
                _ => {}
            }
        }
        None
    }

    fn parse_page_rule_prelude(&mut self) {
        self.eat_trivia();
        if self.current_kind().is_none_or(is_at_rule_prelude_boundary) {
            return;
        }
        let valid = self.page_rule_prelude_is_valid();
        if !valid {
            self.error_at_current(ParseErrorCode::ExpectedValue, "invalid @page prelude");
        }
        self.builder.start_node(if valid {
            SyntaxKind::AtRulePrelude
        } else {
            SyntaxKind::BogusAtRulePrelude
        });
        self.consume_at_rule_prelude_tokens_without_wrapping();
        self.builder.finish_node();
    }

    fn page_rule_prelude_is_valid(&self) -> bool {
        let mut expecting_selector = true;
        let mut expecting_pseudo_name = false;
        let mut saw_selector = false;

        for token in self.tokens.iter().skip(self.position) {
            if token.kind.is_trivia() {
                continue;
            }
            if is_at_rule_prelude_boundary(token.kind) {
                return saw_selector && !expecting_selector && !expecting_pseudo_name;
            }
            if is_interpolation_start(token.kind) {
                return true;
            }
            if expecting_pseudo_name {
                if token.kind != SyntaxKind::Ident {
                    return false;
                }
                saw_selector = true;
                expecting_selector = false;
                expecting_pseudo_name = false;
                continue;
            }
            match token.kind {
                SyntaxKind::Ident if expecting_selector => {
                    saw_selector = true;
                    expecting_selector = false;
                }
                SyntaxKind::Colon => {
                    expecting_pseudo_name = true;
                }
                SyntaxKind::Comma if saw_selector && !expecting_selector => {
                    expecting_selector = true;
                }
                _ => return false,
            }
        }

        saw_selector && !expecting_selector && !expecting_pseudo_name
    }

    fn parse_import_prelude(&mut self) {
        self.eat_trivia();
        if self.dialect == StyleDialect::Less && self.current_kind() == Some(SyntaxKind::LeftParen)
        {
            self.builder.start_node(SyntaxKind::AtRulePrelude);
            self.parse_balanced_parenthesized_prelude(None);
            self.builder.finish_node();
            self.eat_trivia();
        }
        if !self.parse_import_source() {
            self.parse_bogus_import_prelude();
            return;
        }
        while !self.at_end() {
            match self.current_kind() {
                Some(kind) if is_at_rule_prelude_boundary(kind) => break,
                Some(kind) if kind.is_trivia() => self.token_current(),
                Some(SyntaxKind::Ident) if self.current_text() == Some("layer") => {
                    self.parse_import_layer_tail_node()
                }
                Some(SyntaxKind::Ident) if self.current_text() == Some("supports") => {
                    self.parse_import_supports_tail_node()
                }
                Some(_) => {
                    self.parse_media_query_list();
                    break;
                }
                None => break,
            }
        }
    }

    fn parse_import_source(&mut self) -> bool {
        match self.current_kind() {
            Some(SyntaxKind::Url) => {
                self.builder.start_node(SyntaxKind::UrlValue);
                self.token_current();
                self.builder.finish_node();
                true
            }
            Some(SyntaxKind::Ident)
                if self
                    .current_text()
                    .is_some_and(|text| text.eq_ignore_ascii_case("url"))
                    && self.next_kind() == Some(SyntaxKind::LeftParen) =>
            {
                self.builder.start_node(SyntaxKind::UrlValue);
                self.parse_function_call(&[SyntaxKind::LeftBrace, SyntaxKind::Semicolon]);
                self.builder.finish_node();
                true
            }
            Some(SyntaxKind::String) => {
                self.token_current();
                true
            }
            Some(kind) if is_interpolation_start(kind) => {
                self.parse_interpolation(kind, &[SyntaxKind::LeftBrace, SyntaxKind::Semicolon]);
                true
            }
            Some(_) | None => false,
        }
    }

    fn parse_bogus_import_prelude(&mut self) {
        self.builder.start_node(SyntaxKind::BogusAtRulePrelude);
        self.error_at_current(ParseErrorCode::ExpectedValue, "invalid @import source");
        self.consume_at_rule_prelude_tokens_without_wrapping();
        self.builder.finish_node();
    }

    fn parse_named_at_rule_prelude(
        &mut self,
        valid_head: fn(SyntaxKind) -> bool,
        message: &'static str,
    ) {
        if self.current_kind().is_none_or(is_at_rule_prelude_boundary) {
            return;
        }
        let valid_name = self
            .non_trivia_token_from(self.position)
            .is_some_and(|(_, kind)| valid_head(kind));
        if !valid_name {
            self.error_at_current(ParseErrorCode::ExpectedValue, message);
        }
        self.consume_at_rule_prelude_tokens();
    }

    fn parse_import_layer_tail_node(&mut self) {
        let valid = self.import_layer_tail_is_valid();
        if !valid {
            self.error_at_current(ParseErrorCode::ExpectedValue, "invalid @import layer tail");
        }
        self.builder.start_node(if valid {
            SyntaxKind::LayerName
        } else {
            SyntaxKind::BogusLayerName
        });
        self.token_current();
        if self.current_kind() == Some(SyntaxKind::LeftParen) {
            self.parse_balanced_parenthesized_prelude(None);
        }
        self.builder.finish_node();
    }

    fn import_layer_tail_is_valid(&self) -> bool {
        let Some((open_index, next_kind)) = self.non_trivia_token_from(self.position + 1) else {
            return true;
        };
        if next_kind != SyntaxKind::LeftParen {
            return true;
        }
        let Some(close_index) = self.parenthesized_prelude_close_index(open_index) else {
            return false;
        };
        self.layer_name_is_valid_between(open_index + 1, close_index)
    }

    fn layer_name_is_valid_between(&self, start: usize, end: usize) -> bool {
        let mut saw_name = false;
        let mut expecting_segment = true;

        for token in self.tokens[start..end]
            .iter()
            .filter(|token| !token.kind.is_trivia())
        {
            if is_interpolation_start(token.kind) {
                return true;
            }
            match token.kind {
                SyntaxKind::Ident if expecting_segment => {
                    saw_name = true;
                    expecting_segment = false;
                }
                SyntaxKind::Dot if saw_name && !expecting_segment => {
                    expecting_segment = true;
                }
                _ => return false,
            }
        }

        saw_name && !expecting_segment
    }

    fn parse_import_supports_tail_node(&mut self) {
        let valid = self.import_supports_tail_is_valid();
        if !valid {
            self.error_at_current(
                ParseErrorCode::ExpectedValue,
                "invalid @import supports tail",
            );
        }
        self.builder.start_node(if valid {
            SyntaxKind::SupportsCondition
        } else {
            SyntaxKind::BogusSupportsCondition
        });
        self.token_current();
        if self.current_kind() == Some(SyntaxKind::LeftParen) {
            self.parse_balanced_parenthesized_prelude(None);
        }
        self.builder.finish_node();
    }

    fn import_supports_tail_is_valid(&self) -> bool {
        let Some((open_index, SyntaxKind::LeftParen)) =
            self.non_trivia_token_from(self.position + 1)
        else {
            return false;
        };
        let Some(close_index) = self.parenthesized_prelude_close_index(open_index) else {
            return false;
        };
        self.non_trivia_token_from(open_index + 1)
            .is_some_and(|(inner_index, inner_kind)| {
                inner_index < close_index && inner_kind != SyntaxKind::RightParen
            })
    }

    fn consume_at_rule_prelude_tokens(&mut self) {
        if self.current_kind().is_none_or(is_at_rule_prelude_boundary) {
            return;
        }
        self.builder
            .start_node(self.current_generic_at_rule_prelude_node_kind());
        self.consume_at_rule_prelude_tokens_without_wrapping();
        self.builder.finish_node();
    }

    fn consume_at_rule_prelude_tokens_without_wrapping(&mut self) {
        while !self.at_end() {
            match self.current_kind() {
                Some(kind) if is_at_rule_prelude_boundary(kind) => break,
                Some(SyntaxKind::LeftParen) => self.parse_balanced_parenthesized_prelude(None),
                Some(kind) if is_interpolation_start(kind) => {
                    self.parse_interpolation(kind, &[SyntaxKind::LeftBrace, SyntaxKind::Semicolon])
                }
                Some(_) => self.token_current(),
                None => break,
            }
        }
    }

    fn parse_balanced_parenthesized_prelude(&mut self, node_kind: Option<SyntaxKind>) {
        self.parse_balanced_parenthesized_prelude_until(
            node_kind,
            &[SyntaxKind::LeftBrace, SyntaxKind::Semicolon],
        );
    }

    fn parse_balanced_parenthesized_prelude_until(
        &mut self,
        node_kind: Option<SyntaxKind>,
        recovery: &[SyntaxKind],
    ) {
        if let Some(kind) = node_kind {
            self.builder.start_node(kind);
        }
        let mut depth = 0usize;
        let mut closed = false;
        while !self.at_end() {
            match self.current_kind() {
                Some(SyntaxKind::LeftParen) => {
                    depth += 1;
                    self.token_current();
                }
                Some(SyntaxKind::RightParen) => {
                    self.token_current();
                    depth = depth.saturating_sub(1);
                    if depth == 0 {
                        closed = true;
                        break;
                    }
                }
                Some(kind) if recovery.contains(&kind) => break,
                Some(kind) if is_interpolation_start(kind) => {
                    self.parse_interpolation(kind, &[SyntaxKind::LeftBrace, SyntaxKind::Semicolon])
                }
                Some(_) => self.token_current(),
                None => break,
            }
        }
        if node_kind.is_some() {
            self.builder.finish_node();
        }
        if !closed {
            self.error_at_current(
                ParseErrorCode::UnexpectedCharacter,
                "unterminated parenthesized prelude",
            );
        }
    }

    fn parse_interpolation(&mut self, start_kind: SyntaxKind, recovery: &[SyntaxKind]) {
        let Some(end_kind) = interpolation_end_kind(start_kind) else {
            self.token_current();
            return;
        };
        let closed = self.find_before_recovery(end_kind, recovery);
        self.builder.start_node(if closed {
            SyntaxKind::Interpolation
        } else {
            SyntaxKind::BogusInterpolation
        });
        if self.current_kind() == Some(start_kind) {
            self.token_current();
        }
        while !self.at_end() {
            match self.current_kind() {
                Some(kind) if kind == end_kind => {
                    self.token_current();
                    break;
                }
                Some(kind) if !closed && recovery.contains(&kind) => break,
                Some(_) => self.token_current(),
                None => break,
            }
        }
        if !closed {
            self.error_at_current(
                ParseErrorCode::UnexpectedCharacter,
                "unterminated interpolation",
            );
        }
        self.builder.finish_node();
    }

    fn parse_group_at_rule_block(&mut self) {
        self.token_current();
        self.builder.start_node(SyntaxKind::RuleList);
        self.parse_rule_list_items();
        self.builder.finish_node();
        if self.current_kind() == Some(SyntaxKind::RightBrace) {
            self.token_current();
        }
    }

    fn parse_rule_list_items(&mut self) {
        while !self.at_end() {
            self.eat_trivia();
            match self.current_kind() {
                Some(SyntaxKind::RightBrace | SyntaxKind::SassDedent) | None => break,
                Some(SyntaxKind::Semicolon | SyntaxKind::SassOptionalSemicolon) => {
                    self.token_current()
                }
                Some(SyntaxKind::AtKeyword) if self.current_is_css_module_value_rule() => {
                    self.parse_css_module_value_rule()
                }
                Some(SyntaxKind::AtKeyword) if self.current_dialect_at_rule_spec().is_some() => {
                    self.parse_dialect_at_rule()
                }
                Some(SyntaxKind::AtKeyword) => self.parse_at_rule(),
                Some(_) => self.parse_rule(),
            }
        }
    }

    fn parse_declaration_block(&mut self) {
        self.token_current();
        self.builder
            .start_node(if self.previous_left_brace_has_match() {
                SyntaxKind::DeclarationList
            } else {
                SyntaxKind::BogusDeclarationList
            });
        self.parse_declaration_list();
        self.builder.finish_node();
        if self.current_kind() == Some(SyntaxKind::RightBrace) {
            self.token_current();
        } else {
            self.missing_token_bogus_trivia(
                ParseErrorCode::UnexpectedCharacter,
                "unterminated declaration block",
            );
        }
    }

    fn parse_sass_indented_at_rule_block(&mut self, block_kind: AtRuleBlockKind) {
        self.builder.start_node(SyntaxKind::SassIndentedBlock);
        if self.current_kind() == Some(SyntaxKind::SassIndent) {
            self.token_current();
        }
        match block_kind {
            AtRuleBlockKind::GroupRuleList => {
                self.builder.start_node(SyntaxKind::RuleList);
                self.parse_rule_list_items();
                self.builder.finish_node();
            }
            AtRuleBlockKind::DeclarationList | AtRuleBlockKind::Keyframes => {
                self.builder.start_node(SyntaxKind::DeclarationList);
                self.parse_declaration_list();
                self.builder.finish_node();
            }
            AtRuleBlockKind::Raw => self.consume_sass_indented_raw_body(),
        }
        if self.current_kind() == Some(SyntaxKind::SassDedent) {
            self.token_current();
        } else {
            self.error_at_current(
                ParseErrorCode::UnexpectedCharacter,
                "unterminated Sass indented at-rule block",
            );
        }
        self.builder.finish_node();
    }

    fn consume_sass_indented_raw_body(&mut self) {
        let mut depth = 0usize;
        while !self.at_end() {
            match self.current_kind() {
                Some(SyntaxKind::SassIndent) => {
                    depth += 1;
                    self.token_current();
                }
                Some(SyntaxKind::SassDedent) if depth == 0 => break,
                Some(SyntaxKind::SassDedent) => {
                    depth = depth.saturating_sub(1);
                    self.token_current();
                }
                Some(_) => self.token_current(),
                None => break,
            }
        }
    }

    fn parse_keyframes_block(&mut self) {
        self.token_current();
        while !self.at_end() {
            self.eat_trivia();
            match self.current_kind() {
                Some(SyntaxKind::RightBrace) | None => break,
                Some(_) => self.parse_keyframe_block(),
            }
        }
        if self.current_kind() == Some(SyntaxKind::RightBrace) {
            self.token_current();
        }
    }

    fn parse_keyframe_block(&mut self) {
        let has_block = self.find_before_recovery(SyntaxKind::LeftBrace, &[SyntaxKind::RightBrace]);
        self.builder.start_node(if has_block {
            SyntaxKind::KeyframeBlock
        } else {
            SyntaxKind::BogusKeyframeBlock
        });
        if has_block && !self.keyframe_selector_list_is_valid() {
            self.error_at_current(ParseErrorCode::ExpectedValue, "invalid keyframe selector");
        }
        while !self.at_end() {
            match self.current_kind() {
                Some(SyntaxKind::LeftBrace) => {
                    self.parse_declaration_block();
                    break;
                }
                Some(SyntaxKind::RightBrace) | None => break,
                Some(_) => self.token_current(),
            }
        }
        if !has_block {
            self.error_at_current(
                ParseErrorCode::UnexpectedCharacter,
                "expected keyframe declaration block",
            );
        }
        self.builder.finish_node();
    }

    fn keyframe_selector_list_is_valid(&self) -> bool {
        let mut index = self.position;
        let mut saw_selector = false;
        let mut expect_selector = true;
        loop {
            let Some((token_index, kind)) = self.non_trivia_token_from(index) else {
                return false;
            };
            if kind == SyntaxKind::LeftBrace {
                return saw_selector && !expect_selector;
            }
            if expect_selector {
                if is_interpolation_start(kind) {
                    return true;
                }
                if !keyframe_selector_token_is_valid(self.tokens[token_index]) {
                    return false;
                }
                saw_selector = true;
                expect_selector = false;
                index = token_index + 1;
                continue;
            }
            if kind != SyntaxKind::Comma {
                return false;
            }
            expect_selector = true;
            index = token_index + 1;
        }
    }

    fn consume_balanced_block(&mut self) {
        let mut depth = 0usize;
        while !self.at_end() {
            match self.current_kind() {
                Some(SyntaxKind::LeftBrace) => {
                    depth += 1;
                    self.token_current();
                }
                Some(SyntaxKind::RightBrace) => {
                    self.token_current();
                    depth = depth.saturating_sub(1);
                    if depth == 0 {
                        break;
                    }
                }
                Some(_) => self.token_current(),
                None => break,
            }
        }
    }

    fn eat_trivia(&mut self) {
        while matches!(self.current_kind(), Some(kind) if kind.is_trivia()) {
            self.token_current();
        }
    }

    fn consume_until_recovery(&mut self, recovery: &[SyntaxKind]) {
        let should_wrap = self
            .current_kind()
            .is_some_and(|kind| !recovery.contains(&kind));
        if should_wrap {
            self.builder.start_node(SyntaxKind::BogusRecovery);
        }
        while !self.at_end() {
            match self.current_kind() {
                Some(kind) if recovery.contains(&kind) => break,
                Some(_) => self.token_current(),
                None => break,
            }
        }
        if should_wrap {
            self.builder.finish_node();
        }
    }

    fn find_before_recovery(&self, target: SyntaxKind, recovery: &[SyntaxKind]) -> bool {
        let mut index = self.position;
        while let Some(token) = self.tokens.get(index) {
            if token.kind == target {
                return true;
            }
            if recovery.contains(&token.kind) {
                return false;
            }
            index += 1;
        }
        false
    }

    fn find_rule_block_open_before_recovery(&self, recovery: &[SyntaxKind]) -> bool {
        let mut index = self.position;
        while let Some(token) = self.tokens.get(index) {
            if token.kind == SyntaxKind::LeftBrace
                || (self.dialect == StyleDialect::Sass && token.kind == SyntaxKind::SassIndent)
            {
                return true;
            }
            if recovery.contains(&token.kind) {
                return false;
            }
            index += 1;
        }
        false
    }

    fn find_text_before_recovery(&self, target: &str, recovery: &[SyntaxKind]) -> bool {
        let mut index = self.position;
        while let Some(token) = self.tokens.get(index) {
            if token.text == target {
                return true;
            }
            if recovery.contains(&token.kind) {
                return false;
            }
            index += 1;
        }
        false
    }

    fn current_function_has_closing_paren_before(&self, recovery: &[SyntaxKind]) -> bool {
        let Some(open_index) = self.position.checked_add(1) else {
            return false;
        };
        if self
            .tokens
            .get(open_index)
            .is_none_or(|token| token.kind != SyntaxKind::LeftParen)
        {
            return false;
        }

        let mut depth = 0usize;
        for token in self.tokens.iter().skip(open_index) {
            match token.kind {
                SyntaxKind::LeftParen => depth += 1,
                SyntaxKind::RightParen => {
                    depth = depth.saturating_sub(1);
                    if depth == 0 {
                        return true;
                    }
                }
                kind if depth == 1 && recovery.contains(&kind) => return false,
                _ => {}
            }
        }
        false
    }

    fn current_split_important_annotation(&self) -> bool {
        self.current_text() == Some("!")
            && self
                .non_trivia_token_from(self.position + 1)
                .is_some_and(|(index, kind)| {
                    matches!(kind, SyntaxKind::Ident | SyntaxKind::KeywordImportant)
                        && self
                            .tokens
                            .get(index)
                            .is_some_and(|token| token.text.eq_ignore_ascii_case("important"))
                })
    }

    fn current_scss_variable_flag_annotation(&self) -> bool {
        matches!(self.dialect, StyleDialect::Scss | StyleDialect::Sass)
            && self.current_text() == Some("!")
            && self
                .non_trivia_token_from(self.position + 1)
                .is_some_and(|(index, kind)| {
                    kind == SyntaxKind::Ident
                        && self.tokens.get(index).is_some_and(|token| {
                            token.text.eq_ignore_ascii_case("default")
                                || token.text.eq_ignore_ascii_case("global")
                        })
                })
    }

    fn current_bracketed_value_has_closing_bracket_before(&self, recovery: &[SyntaxKind]) -> bool {
        let mut depth = 0usize;
        for token in self.tokens.iter().skip(self.position) {
            match token.kind {
                SyntaxKind::LeftBracket => depth += 1,
                SyntaxKind::RightBracket => {
                    depth = depth.saturating_sub(1);
                    if depth == 0 {
                        return true;
                    }
                }
                kind if depth == 1 && recovery.contains(&kind) => return false,
                _ => {}
            }
        }
        false
    }

    fn current_simple_block_has_matching_close(&self, recovery: &[SyntaxKind]) -> bool {
        let Some(open_kind) = self.current_kind() else {
            return false;
        };
        if matching_simple_block_close(open_kind).is_none() {
            return false;
        }

        let mut expected_closes = Vec::new();
        for token in self.tokens.iter().skip(self.position) {
            if let Some(close_kind) = matching_simple_block_close(token.kind) {
                expected_closes.push(close_kind);
                continue;
            }

            if expected_closes.last().copied() == Some(token.kind) {
                expected_closes.pop();
                if expected_closes.is_empty() {
                    return true;
                }
                continue;
            }

            if expected_closes.len() == 1 && recovery.contains(&token.kind) {
                return false;
            }
        }
        false
    }

    fn current_dialect_at_rule_node_kind(&self, spec: AtRuleSpec) -> SyntaxKind {
        if !self.find_rule_block_open_before_recovery(&[
            SyntaxKind::Semicolon,
            SyntaxKind::SassOptionalSemicolon,
            SyntaxKind::RightBrace,
            SyntaxKind::SassDedent,
        ]) {
            return match spec.node_kind {
                SyntaxKind::ScssMixinDeclaration => SyntaxKind::BogusScssMixin,
                SyntaxKind::ScssFunctionDeclaration => SyntaxKind::BogusScssFunction,
                SyntaxKind::ScssControlIf
                | SyntaxKind::ScssControlElse
                | SyntaxKind::ScssControlEach
                | SyntaxKind::ScssControlFor
                | SyntaxKind::ScssControlWhile => SyntaxKind::BogusScssControl,
                _ => spec.node_kind,
            };
        }
        spec.node_kind
    }

    fn current_less_guard_has_condition_before(&self, recovery: &[SyntaxKind]) -> bool {
        let mut index = self.position + 1;
        while let Some(token) = self.tokens.get(index) {
            if recovery.contains(&token.kind) {
                return false;
            }
            if token.kind == SyntaxKind::LeftParen {
                return true;
            }
            index += 1;
        }
        false
    }

    fn current_scss_module_config_has_balanced_parens(&self) -> bool {
        let Some((_, SyntaxKind::LeftParen)) = self.non_trivia_token_from(self.position + 1) else {
            return false;
        };
        self.current_prelude_parentheses_are_balanced_until(&[
            SyntaxKind::Semicolon,
            SyntaxKind::SassOptionalSemicolon,
            SyntaxKind::LeftBrace,
            SyntaxKind::SassIndent,
        ])
    }

    fn current_value_has_top_level_comma_before(&self, recovery: &[SyntaxKind]) -> bool {
        let mut paren_depth = 0usize;
        let mut bracket_depth = 0usize;
        for token in self.tokens.iter().skip(self.position) {
            match token.kind {
                kind if paren_depth == 0 && bracket_depth == 0 && recovery.contains(&kind) => {
                    return false;
                }
                SyntaxKind::LeftParen => paren_depth += 1,
                SyntaxKind::RightParen => paren_depth = paren_depth.saturating_sub(1),
                SyntaxKind::LeftBracket => bracket_depth += 1,
                SyntaxKind::RightBracket => bracket_depth = bracket_depth.saturating_sub(1),
                SyntaxKind::Comma if paren_depth == 0 && bracket_depth == 0 => return true,
                _ => {}
            }
        }
        false
    }

    fn current_value_list_is_bogus(&self, recovery: &[SyntaxKind]) -> bool {
        let mut paren_depth = 0usize;
        let mut bracket_depth = 0usize;
        let mut expecting_item = true;
        for token in self.tokens.iter().skip(self.position) {
            if token.kind.is_trivia() {
                continue;
            }
            match token.kind {
                kind if paren_depth == 0 && bracket_depth == 0 && recovery.contains(&kind) => {
                    return expecting_item;
                }
                SyntaxKind::LeftParen => {
                    paren_depth += 1;
                    expecting_item = false;
                }
                SyntaxKind::RightParen => {
                    paren_depth = paren_depth.saturating_sub(1);
                    expecting_item = false;
                }
                SyntaxKind::LeftBracket => {
                    bracket_depth += 1;
                    expecting_item = false;
                }
                SyntaxKind::RightBracket => {
                    bracket_depth = bracket_depth.saturating_sub(1);
                    expecting_item = false;
                }
                SyntaxKind::Comma if paren_depth == 0 && bracket_depth == 0 => {
                    if expecting_item {
                        return true;
                    }
                    expecting_item = true;
                }
                _ => expecting_item = false,
            }
        }
        expecting_item
    }

    fn current_starts_missing_semicolon_declaration(&self, recovery: &[SyntaxKind]) -> bool {
        match self.current_kind() {
            Some(SyntaxKind::Ident | SyntaxKind::CustomPropertyName) => {}
            _ => return false,
        }

        let mut index = self.position + 1;
        while let Some(token) = self.tokens.get(index) {
            if token.kind.is_trivia() {
                index += 1;
                continue;
            }
            if recovery.contains(&token.kind) {
                return false;
            }
            return token.kind == SyntaxKind::Colon;
        }
        false
    }

    fn current_selector_item_is_bogus(&self, recovery: &[SyntaxKind]) -> bool {
        self.selector_item_is_bogus_from(self.position, recovery)
    }

    fn selector_item_is_bogus_from(&self, start: usize, recovery: &[SyntaxKind]) -> bool {
        let mut paren_depth = 0usize;
        let mut bracket_depth = 0usize;
        let mut saw_selector_token = false;

        for token in self.tokens.iter().skip(start) {
            if token.kind.is_trivia() {
                continue;
            }
            if paren_depth == 0
                && bracket_depth == 0
                && (token.kind == SyntaxKind::Comma
                    || is_selector_boundary_until(token.kind, recovery))
            {
                break;
            }

            match token.kind {
                SyntaxKind::LeftParen => paren_depth += 1,
                SyntaxKind::RightParen => paren_depth = paren_depth.saturating_sub(1),
                SyntaxKind::LeftBracket => bracket_depth += 1,
                SyntaxKind::RightBracket => bracket_depth = bracket_depth.saturating_sub(1),
                _ => {}
            }

            if !selector_item_token_is_recoverable(token.kind) {
                return true;
            }
            saw_selector_token = true;
        }

        !saw_selector_token
    }

    fn selector_list_contains_bogus_item_until(&self, recovery: &[SyntaxKind]) -> bool {
        let mut index = self.position;
        while let Some(token) = self.tokens.get(index) {
            if token.kind.is_trivia() || token.kind == SyntaxKind::Comma {
                index += 1;
                continue;
            }
            if is_selector_boundary_until(token.kind, recovery) {
                return false;
            }
            if self.selector_item_is_bogus_from(index, recovery) {
                return true;
            }

            let mut paren_depth = 0usize;
            let mut bracket_depth = 0usize;
            while let Some(token) = self.tokens.get(index) {
                if paren_depth == 0
                    && bracket_depth == 0
                    && (token.kind == SyntaxKind::Comma
                        || is_selector_boundary_until(token.kind, recovery))
                {
                    break;
                }
                match token.kind {
                    SyntaxKind::LeftParen => paren_depth += 1,
                    SyntaxKind::RightParen => paren_depth = paren_depth.saturating_sub(1),
                    SyntaxKind::LeftBracket => bracket_depth += 1,
                    SyntaxKind::RightBracket => bracket_depth = bracket_depth.saturating_sub(1),
                    _ => {}
                }
                index += 1;
            }
        }
        false
    }

    fn current_generic_at_rule_prelude_node_kind(&self) -> SyntaxKind {
        if self.current_prelude_parentheses_are_balanced_until(&[
            SyntaxKind::LeftBrace,
            SyntaxKind::Semicolon,
        ]) {
            SyntaxKind::AtRulePrelude
        } else {
            SyntaxKind::BogusAtRulePrelude
        }
    }

    fn current_prelude_parentheses_are_balanced_until(&self, recovery: &[SyntaxKind]) -> bool {
        let mut depth = 0usize;
        for token in self.tokens.iter().skip(self.position) {
            match token.kind {
                kind if depth == 0 && recovery.contains(&kind) => return true,
                SyntaxKind::LeftParen => depth += 1,
                SyntaxKind::RightParen => {
                    if depth == 0 {
                        return false;
                    }
                    depth -= 1;
                }
                _ => {}
            }
        }
        depth == 0
    }

    fn previous_left_brace_has_match(&self) -> bool {
        let Some(open_index) = self.position.checked_sub(1) else {
            return false;
        };
        let Some(open) = self.tokens.get(open_index) else {
            return false;
        };
        if open.kind != SyntaxKind::LeftBrace {
            return false;
        }

        let mut depth = 0usize;
        for token in self.tokens.iter().skip(open_index) {
            match token.kind {
                SyntaxKind::LeftBrace => depth += 1,
                SyntaxKind::RightBrace => {
                    depth = depth.saturating_sub(1);
                    if depth == 0 {
                        return true;
                    }
                }
                _ => {}
            }
        }
        false
    }

    fn current_starts_nested_rule(&self) -> bool {
        matches!(
            self.current_kind(),
            Some(
                SyntaxKind::Dot
                    | SyntaxKind::Hash
                    | SyntaxKind::Ampersand
                    | SyntaxKind::Colon
                    | SyntaxKind::DoubleColon
                    | SyntaxKind::LeftBracket
            )
        ) && self.find_rule_block_open_before_recovery(&[
            SyntaxKind::Colon,
            SyntaxKind::Semicolon,
            SyntaxKind::SassOptionalSemicolon,
            SyntaxKind::RightBrace,
            SyntaxKind::SassDedent,
        ])
    }

    fn current_starts_scss_nested_property(&self) -> bool {
        if !matches!(self.dialect, StyleDialect::Scss | StyleDialect::Sass) {
            return false;
        }
        if !matches!(
            self.current_kind(),
            Some(SyntaxKind::Ident | SyntaxKind::CustomPropertyName)
        ) {
            return false;
        }

        let mut saw_colon = false;
        for token in self.tokens.iter().skip(self.position) {
            match token.kind {
                SyntaxKind::Colon => saw_colon = true,
                SyntaxKind::LeftBrace if saw_colon => return true,
                SyntaxKind::SassIndent if saw_colon && self.dialect == StyleDialect::Sass => {
                    return true;
                }
                SyntaxKind::Semicolon
                | SyntaxKind::SassOptionalSemicolon
                | SyntaxKind::RightBrace
                | SyntaxKind::SassDedent => return false,
                _ => {}
            }
        }
        false
    }

    fn current_starts_less_mixin_declaration(&self) -> bool {
        self.dialect == StyleDialect::Less
            && self.current_starts_less_callable_signature()
            && self.find_before_recovery(
                SyntaxKind::LeftBrace,
                &[SyntaxKind::Semicolon, SyntaxKind::RightBrace],
            )
    }

    fn current_starts_less_mixin_call(&self) -> bool {
        self.dialect == StyleDialect::Less
            && self.current_starts_less_callable_signature()
            && !self.find_before_recovery(
                SyntaxKind::LeftBrace,
                &[SyntaxKind::Semicolon, SyntaxKind::RightBrace],
            )
    }

    fn current_starts_less_callable_signature(&self) -> bool {
        match self.current_kind() {
            Some(SyntaxKind::Dot) => {
                let Some((index, SyntaxKind::Ident | SyntaxKind::CustomPropertyName)) =
                    self.non_trivia_token_from(self.position + 1)
                else {
                    return false;
                };
                self.non_trivia_token_from(index + 1)
                    .is_some_and(|(_, kind)| kind == SyntaxKind::LeftParen)
            }
            Some(SyntaxKind::Hash) => self
                .non_trivia_token_from(self.position + 1)
                .is_some_and(|(_, kind)| kind == SyntaxKind::LeftParen),
            _ => false,
        }
    }

    fn current_starts_less_extend_rule(&self) -> bool {
        self.dialect == StyleDialect::Less
            && self.current_kind() == Some(SyntaxKind::Colon)
            && self
                .non_trivia_token_from(self.position + 1)
                .is_some_and(|(index, kind)| {
                    kind == SyntaxKind::Ident
                        && self
                            .tokens
                            .get(index)
                            .is_some_and(|token| token.text == "extend")
                })
    }

    fn current_starts_less_namespace_access(&self) -> bool {
        self.dialect == StyleDialect::Less
            && matches!(
                self.current_kind(),
                Some(SyntaxKind::Dot | SyntaxKind::Hash)
            )
            && self.find_before_recovery(
                SyntaxKind::GreaterThan,
                &[
                    SyntaxKind::Semicolon,
                    SyntaxKind::LeftBrace,
                    SyntaxKind::RightBrace,
                ],
            )
            && self.find_before_recovery(
                SyntaxKind::LeftParen,
                &[
                    SyntaxKind::Semicolon,
                    SyntaxKind::LeftBrace,
                    SyntaxKind::RightBrace,
                ],
            )
    }

    fn current_left_brace_has_match(&self) -> bool {
        let mut depth = 0usize;
        for token in self.tokens.iter().skip(self.position) {
            match token.kind {
                SyntaxKind::LeftBrace => depth += 1,
                SyntaxKind::RightBrace => {
                    depth = depth.saturating_sub(1);
                    if depth == 0 {
                        return true;
                    }
                }
                _ => {}
            }
        }
        false
    }

    fn token_current(&mut self) {
        if let Some(token) = self.tokens.get(self.position).copied() {
            self.builder.token(token.kind, token.text);
            self.position += 1;
        }
    }

    fn empty_bogus_node(&mut self, kind: SyntaxKind, code: ParseErrorCode, message: &'static str) {
        self.builder.start_node(kind);
        self.builder.finish_node();
        self.error_at_current(code, message);
    }

    fn missing_token_bogus_trivia(&mut self, code: ParseErrorCode, message: &'static str) {
        self.builder.start_node(SyntaxKind::BogusTrivia);
        self.builder.finish_node();
        self.error_at_current(code, message);
    }

    fn error_at_current(&mut self, code: ParseErrorCode, message: &'static str) {
        self.errors.push(ParseError {
            code,
            range: self.current_range(),
            message,
        });
    }

    fn current_kind(&self) -> Option<SyntaxKind> {
        self.tokens.get(self.position).map(|token| token.kind)
    }

    fn current_range(&self) -> TextRange {
        if let Some(token) = self.tokens.get(self.position) {
            return token.range;
        }
        let end = self
            .tokens
            .last()
            .map(|token| token.range.end())
            .unwrap_or_else(|| TextSize::from(0));
        TextRange::new(end, end)
    }

    fn current_text(&self) -> Option<&'text str> {
        self.tokens.get(self.position).map(|token| token.text)
    }

    fn current_dialect_at_rule_spec(&self) -> Option<AtRuleSpec> {
        let text = self.current_text()?;
        match self.dialect {
            StyleDialect::Scss | StyleDialect::Sass => scss_at_rule_spec(text),
            StyleDialect::Css | StyleDialect::Less => None,
        }
    }

    fn current_is_css_module_value_rule(&self) -> bool {
        self.current_text() == Some("@value")
    }

    fn next_kind(&self) -> Option<SyntaxKind> {
        self.tokens.get(self.position + 1).map(|token| token.kind)
    }

    fn next_non_trivia_kind(&self) -> Option<SyntaxKind> {
        let mut index = self.position + 1;
        while let Some(token) = self.tokens.get(index) {
            if !token.kind.is_trivia() {
                return Some(token.kind);
            }
            index += 1;
        }
        None
    }

    fn non_trivia_token_from(&self, mut index: usize) -> Option<(usize, SyntaxKind)> {
        while let Some(token) = self.tokens.get(index) {
            if !token.kind.is_trivia() {
                return Some((index, token.kind));
            }
            index += 1;
        }
        None
    }

    fn non_trivia_token_after_interpolation(
        &self,
        mut index: usize,
        start_kind: SyntaxKind,
    ) -> Option<(usize, SyntaxKind)> {
        let end_kind = interpolation_end_kind(start_kind)?;
        index += 1;
        while let Some(token) = self.tokens.get(index) {
            if token.kind == end_kind {
                return self.non_trivia_token_from(index + 1);
            }
            if is_at_rule_prelude_boundary(token.kind) {
                return None;
            }
            index += 1;
        }
        None
    }

    fn current_starts_namespace_qualified_selector(&self, kind: SyntaxKind) -> bool {
        match kind {
            SyntaxKind::Ident | SyntaxKind::Star => {
                self.next_kind() == Some(SyntaxKind::Pipe)
                    && self
                        .tokens
                        .get(self.position + 2)
                        .is_some_and(|token| namespace_selector_target_can_start(token.kind))
            }
            SyntaxKind::Pipe => self
                .tokens
                .get(self.position + 1)
                .is_some_and(|token| namespace_selector_target_can_start(token.kind)),
            _ => false,
        }
    }

    fn namespace_qualified_selector_target_kind(&self) -> Option<SyntaxKind> {
        let target_index = if self.current_kind() == Some(SyntaxKind::Pipe) {
            self.position + 1
        } else {
            self.position + 2
        };
        self.tokens.get(target_index).map(|token| token.kind)
    }

    fn at_end(&self) -> bool {
        self.position >= self.tokens.len()
    }
}

impl<'text, 'extension, E> Tokenizer<'text, 'extension, E>
where
    E: DialectExtension,
{
    fn new(text: &'text str, extension: &'extension E) -> Self {
        Self {
            text,
            extension,
            offset: 0,
            scss_interpolation_depth: 0,
            less_interpolation_depth: 0,
            sass_indent_stack: vec![0],
            tokens: Vec::new(),
            errors: Vec::new(),
        }
    }

    fn tokenize(&mut self) {
        while let Some(current) = self.current_char() {
            let start = self.offset;
            match current {
                '\u{feff}' if start == 0 => self.bump_current(),
                '\r' | '\n' if self.extension.dialect() == StyleDialect::Sass => {
                    self.consume_sass_indented_newline(start)
                }
                char if char.is_whitespace() => {
                    self.consume_while(SyntaxKind::Whitespace, |c| c.is_whitespace())
                }
                '/' if self.starts_with("/*") => self.consume_block_comment(),
                '/' if self.starts_with("//") && self.extension.dialect() != StyleDialect::Css => {
                    self.consume_line_comment()
                }
                '#' if self.starts_with("#{") && self.supports_scss_interpolation() => {
                    self.consume_scss_interpolation_start(start)
                }
                '@' if self.starts_with("@{") && self.supports_less_interpolation() => {
                    self.consume_less_interpolation_start(start)
                }
                '!' if self.starts_with_ascii_keyword("!important") => {
                    self.consume_static(SyntaxKind::Important, start, "!important".len())
                }
                '<' if self.starts_with("<!--") => {
                    self.consume_static(SyntaxKind::Cdo, start, "<!--".len())
                }
                '-' if self.starts_with("-->") => {
                    self.consume_static(SyntaxKind::Cdc, start, "-->".len())
                }
                '"' | '\'' => self.consume_string(current),
                'u' | 'U' if self.starts_unicode_range() => self.consume_unicode_range(),
                '0'..='9' => self.consume_number(),
                '$' if matches!(
                    self.extension.dialect(),
                    StyleDialect::Scss | StyleDialect::Sass
                ) =>
                {
                    self.consume_prefixed_name(SyntaxKind::ScssVariable)
                }
                '@' if self.extension.dialect() == StyleDialect::Less => {
                    self.consume_less_at_name()
                }
                '@' => self.consume_at_keyword(),
                '!' => self.consume_static(SyntaxKind::Delim, start, 1),
                '.' if self.current_starts_number() => self.consume_number(),
                '.' => self.consume_static(SyntaxKind::Dot, start, 1),
                ',' => self.consume_static(SyntaxKind::Comma, start, 1),
                ':' if self.starts_with("::") => {
                    self.consume_static(SyntaxKind::DoubleColon, start, 2)
                }
                ':' => self.consume_static(SyntaxKind::Colon, start, 1),
                ';' => self.consume_static(SyntaxKind::Semicolon, start, 1),
                '{' => self.consume_static(SyntaxKind::LeftBrace, start, 1),
                '}' if self.scss_interpolation_depth > 0 => {
                    self.consume_scss_interpolation_end(start)
                }
                '}' if self.less_interpolation_depth > 0 => {
                    self.consume_less_interpolation_end(start)
                }
                '}' => self.consume_static(SyntaxKind::RightBrace, start, 1),
                '(' => self.consume_static(SyntaxKind::LeftParen, start, 1),
                ')' => self.consume_static(SyntaxKind::RightParen, start, 1),
                '[' => self.consume_static(SyntaxKind::LeftBracket, start, 1),
                ']' => self.consume_static(SyntaxKind::RightBracket, start, 1),
                '+' if self.starts_with("+=") => {
                    self.consume_static(SyntaxKind::PlusEquals, start, 2)
                }
                '+' if self.current_starts_number() => self.consume_number(),
                '+' => self.consume_static(SyntaxKind::Plus, start, 1),
                '-' if self.starts_with("-=") => {
                    self.consume_static(SyntaxKind::MinusEquals, start, 2)
                }
                '-' if self.current_starts_number() => self.consume_number(),
                '-' if self.current_starts_ident_sequence() => self.consume_ident_like(),
                '-' => self.consume_static(SyntaxKind::Minus, start, 1),
                '*' if self.starts_with("*=") => {
                    self.consume_static(SyntaxKind::SubstringMatch, start, 2)
                }
                '*' => self.consume_static(SyntaxKind::Star, start, 1),
                '/' if self.starts_with("/=") => {
                    self.consume_static(SyntaxKind::SlashEquals, start, 2)
                }
                '/' => self.consume_static(SyntaxKind::Slash, start, 1),
                '%' if self.starts_scss_placeholder() => {
                    self.consume_prefixed_name(SyntaxKind::ScssPlaceholder)
                }
                '%' => self.consume_static(SyntaxKind::Percent, start, 1),
                '=' if self.starts_with("=>") => self.consume_static(SyntaxKind::Arrow, start, 2),
                '=' => self.consume_static(SyntaxKind::Equals, start, 1),
                '~' if self.starts_less_escaped_string() => self.consume_less_escaped_string(start),
                '~' if self.starts_with("~=") => {
                    self.consume_static(SyntaxKind::IncludesMatch, start, 2)
                }
                '~' => self.consume_static(SyntaxKind::Tilde, start, 1),
                '|' if self.starts_with("|=") => {
                    self.consume_static(SyntaxKind::DashMatch, start, 2)
                }
                '|' if self.starts_with("||") => {
                    self.consume_static(SyntaxKind::ColumnCombinator, start, 2)
                }
                '|' => self.consume_static(SyntaxKind::Pipe, start, 1),
                '^' if self.starts_with("^=") => {
                    self.consume_static(SyntaxKind::PrefixMatch, start, 2)
                }
                '^' => self.consume_static(SyntaxKind::Caret, start, 1),
                '$' if self.starts_with("$=") => {
                    self.consume_static(SyntaxKind::SuffixMatch, start, 2)
                }
                '$' if self.starts_less_property_variable() => {
                    self.consume_prefixed_name(SyntaxKind::LessPropertyVariableToken)
                }
                '&' if self.starts_with("&&") => {
                    self.consume_static(SyntaxKind::DoubleAmpersand, start, 2)
                }
                '&' => self.consume_static(SyntaxKind::Ampersand, start, 1),
                '>' => self.consume_static(SyntaxKind::GreaterThan, start, 1),
                '<' => self.consume_static(SyntaxKind::LessThan, start, 1),
                '#' if self.current_hash_starts_name() => self.consume_name_like(SyntaxKind::Hash),
                '#' => self.consume_static(SyntaxKind::Delim, start, 1),
                '\\' if self.current_starts_valid_escape() => {
                    self.consume_name_like(SyntaxKind::Ident)
                }
                char if is_name_start(char) => self.consume_ident_like(),
                char => self.consume_unexpected(char),
            }
        }
        self.consume_pending_sass_dedents();
    }

    fn consume_static(&mut self, kind: SyntaxKind, start: usize, byte_len: usize) {
        self.offset += byte_len;
        self.push(kind, start, self.offset);
    }

    fn consume_while(&mut self, kind: SyntaxKind, predicate: impl Fn(char) -> bool) {
        let start = self.offset;
        while let Some(char) = self.current_char() {
            if !predicate(char) {
                break;
            }
            self.bump_char(char);
        }
        self.push(kind, start, self.offset);
    }

    fn consume_block_comment(&mut self) {
        let start = self.offset;
        self.offset += 2;
        while self.offset < self.text.len() {
            if self.starts_with("*/") {
                self.offset += 2;
                self.push(SyntaxKind::BlockComment, start, self.offset);
                return;
            }
            match self.current_char() {
                Some(char) => self.bump_char(char),
                None => break,
            }
        }
        self.push(SyntaxKind::BlockComment, start, self.offset);
        self.error(
            ParseErrorCode::UnterminatedBlockComment,
            start,
            self.offset,
            "unterminated block comment",
        );
    }

    fn consume_line_comment(&mut self) {
        let start = self.offset;
        while let Some(char) = self.current_char() {
            if char == '\n' {
                break;
            }
            if char == '\r' {
                break;
            }
            self.bump_char(char);
        }
        self.push(SyntaxKind::LineComment, start, self.offset);
    }

    fn consume_sass_indented_newline(&mut self, start: usize) {
        self.consume_line_break();
        let indent = self.consume_sass_line_indent();
        let line_start = self.offset;
        let current_indent = self.sass_indent_stack.last().copied().unwrap_or(0);

        if indent > current_indent {
            self.push(SyntaxKind::SassIndentedNewline, start, line_start);
            self.sass_indent_stack.push(indent);
            self.push(SyntaxKind::SassIndent, line_start, line_start);
            return;
        }

        if self.previous_significant_sass_token_can_end_statement() {
            self.push(SyntaxKind::SassOptionalSemicolon, start, start);
        }
        self.push(SyntaxKind::SassIndentedNewline, start, line_start);

        while self.sass_indent_stack.len() > 1
            && self
                .sass_indent_stack
                .last()
                .is_some_and(|current| indent < *current)
        {
            self.sass_indent_stack.pop();
            self.push(SyntaxKind::SassDedent, line_start, line_start);
        }

        if self
            .sass_indent_stack
            .last()
            .is_some_and(|current| indent != *current)
        {
            self.error(
                ParseErrorCode::UnexpectedCharacter,
                line_start,
                line_start,
                "inconsistent Sass indentation",
            );
        }
    }

    fn consume_line_break(&mut self) {
        if self.starts_with("\r\n") {
            self.offset += "\r\n".len();
            return;
        }
        if let Some(char @ ('\r' | '\n')) = self.current_char() {
            self.bump_char(char);
        }
    }

    fn consume_sass_line_indent(&mut self) -> usize {
        let mut indent = 0usize;
        while let Some(char) = self.current_char() {
            match char {
                ' ' => {
                    indent += 1;
                    self.bump_char(char);
                }
                '\t' => {
                    indent += 4;
                    self.bump_char(char);
                }
                _ => break,
            }
        }
        indent
    }

    fn consume_pending_sass_dedents(&mut self) {
        if self.extension.dialect() != StyleDialect::Sass {
            return;
        }
        while self.sass_indent_stack.len() > 1 {
            self.sass_indent_stack.pop();
            self.push(SyntaxKind::SassDedent, self.offset, self.offset);
        }
    }

    fn previous_significant_sass_token_can_end_statement(&self) -> bool {
        self.tokens
            .iter()
            .rev()
            .find(|token| !token.kind.is_trivia())
            .is_some_and(|token| sass_token_can_end_statement(token.kind))
    }

    fn consume_scss_interpolation_start(&mut self, start: usize) {
        self.offset += "#{".len();
        self.scss_interpolation_depth += 1;
        self.push(SyntaxKind::ScssInterpolationStart, start, self.offset);
    }

    fn consume_scss_interpolation_end(&mut self, start: usize) {
        self.offset += '}'.len_utf8();
        self.scss_interpolation_depth = self.scss_interpolation_depth.saturating_sub(1);
        self.push(SyntaxKind::ScssInterpolationEnd, start, self.offset);
    }

    fn consume_less_interpolation_start(&mut self, start: usize) {
        self.offset += "@{".len();
        self.less_interpolation_depth += 1;
        self.push(SyntaxKind::LessInterpolationStart, start, self.offset);
    }

    fn consume_less_interpolation_end(&mut self, start: usize) {
        self.offset += '}'.len_utf8();
        self.less_interpolation_depth = self.less_interpolation_depth.saturating_sub(1);
        self.push(SyntaxKind::LessInterpolationEnd, start, self.offset);
    }

    fn consume_string(&mut self, quote: char) {
        let start = self.offset;
        self.bump_char(quote);
        while let Some(char) = self.current_char() {
            self.bump_char(char);
            if matches!(char, '\n' | '\r' | '\u{000c}') {
                self.push(SyntaxKind::BadString, start, self.offset);
                self.error(
                    ParseErrorCode::UnterminatedString,
                    start,
                    self.offset,
                    "unterminated string",
                );
                return;
            }
            if char == quote {
                self.push(SyntaxKind::String, start, self.offset);
                return;
            }
            if char == '\\'
                && let Some(escaped) = self.current_char()
            {
                self.bump_char(escaped);
            }
        }
        self.push(SyntaxKind::BadString, start, self.offset);
        self.error(
            ParseErrorCode::UnterminatedString,
            start,
            self.offset,
            "unterminated string",
        );
    }

    fn consume_less_escaped_string(&mut self, start: usize) {
        self.offset += '~'.len_utf8();
        let Some(quote @ ('"' | '\'')) = self.current_char() else {
            self.push(SyntaxKind::Tilde, start, self.offset);
            return;
        };
        self.bump_char(quote);
        while let Some(char) = self.current_char() {
            self.bump_char(char);
            if matches!(char, '\n' | '\r' | '\u{000c}') {
                self.push(SyntaxKind::BadString, start, self.offset);
                self.error(
                    ParseErrorCode::UnterminatedString,
                    start,
                    self.offset,
                    "unterminated Less escaped string",
                );
                return;
            }
            if char == quote {
                self.push(SyntaxKind::LessEscapedString, start, self.offset);
                return;
            }
            if char == '\\'
                && let Some(escaped) = self.current_char()
            {
                self.bump_char(escaped);
            }
        }
        self.push(SyntaxKind::BadString, start, self.offset);
        self.error(
            ParseErrorCode::UnterminatedString,
            start,
            self.offset,
            "unterminated Less escaped string",
        );
    }

    fn consume_number(&mut self) {
        let start = self.offset;
        if matches!(self.current_char(), Some('+' | '-')) {
            self.bump_current();
        }
        self.consume_digits();
        if self.current_char() == Some('.') && self.char_after_current_is_ascii_digit() {
            self.bump_current();
            self.consume_digits();
        }
        if self.current_starts_number_exponent() {
            self.bump_current();
            if matches!(self.current_char(), Some('+' | '-')) {
                self.bump_current();
            }
            self.consume_digits();
        }
        if self.current_char() == Some('%') {
            self.offset += 1;
            self.push(SyntaxKind::Percentage, start, self.offset);
            return;
        }
        if self.current_starts_ident_sequence() {
            self.consume_name_continue_sequence();
            self.push(SyntaxKind::Dimension, start, self.offset);
            return;
        }
        self.push(SyntaxKind::Number, start, self.offset);
    }

    fn consume_unicode_range(&mut self) {
        let start = self.offset;
        self.bump_current();
        self.offset += '+'.len_utf8();
        self.consume_unicode_range_codepoints(true);
        if self.current_char() == Some('-') && self.next_char_is_hex_digit() {
            self.bump_current();
            self.consume_unicode_range_codepoints(false);
        }
        self.push(SyntaxKind::UnicodeRange, start, self.offset);
    }

    fn consume_unicode_range_codepoints(&mut self, allow_question_mark: bool) {
        let mut consumed = 0usize;
        while consumed < 6 {
            match self.current_char() {
                Some(char) if char.is_ascii_hexdigit() => {
                    self.bump_char(char);
                    consumed += 1;
                }
                Some('?') if allow_question_mark => {
                    self.bump_current();
                    consumed += 1;
                }
                _ => break,
            }
        }
    }

    fn consume_digits(&mut self) {
        while matches!(self.current_char(), Some('0'..='9')) {
            self.offset += 1;
        }
    }

    fn consume_prefixed_name(&mut self, preferred_kind: SyntaxKind) {
        let start = self.offset;
        self.bump_current();
        while matches!(self.current_char(), Some(char) if is_name_continue(char)) {
            self.bump_current();
        }
        let text = &self.text[start..self.offset];
        let kind = self
            .extension
            .classify_variable_token(text)
            .unwrap_or(preferred_kind);
        self.push(kind, start, self.offset);
    }

    fn consume_less_at_name(&mut self) {
        let start = self.offset;
        self.bump_current();
        while matches!(self.current_char(), Some(char) if is_name_continue(char)) {
            self.bump_current();
        }
        let text = &self.text[start..self.offset];
        let kind = if is_css_at_rule_name(text) {
            SyntaxKind::AtKeyword
        } else {
            self.extension
                .classify_variable_token(text)
                .unwrap_or(SyntaxKind::LessVariable)
        };
        self.push(kind, start, self.offset);
    }

    fn consume_at_keyword(&mut self) {
        let start = self.offset;
        self.bump_current();
        while matches!(self.current_char(), Some(char) if is_name_continue(char)) {
            self.bump_current();
        }
        self.push(SyntaxKind::AtKeyword, start, self.offset);
    }

    fn consume_name_like(&mut self, kind: SyntaxKind) {
        let start = self.offset;
        self.consume_name_start();
        self.consume_name_continue_sequence();
        self.push(kind, start, self.offset);
    }

    fn consume_ident_like(&mut self) {
        let start = self.offset;
        self.consume_name_continue_sequence();
        let ident = &self.text[start..self.offset];
        if ident.eq_ignore_ascii_case("url")
            && self.current_char() == Some('(')
            && !self.url_starts_with_quoted_argument()
        {
            self.consume_url_token(start);
            return;
        }
        let kind = if is_custom_property_name_text(ident) {
            SyntaxKind::CustomPropertyName
        } else {
            SyntaxKind::Ident
        };
        self.push(kind, start, self.offset);
    }

    fn consume_name_start(&mut self) {
        if self.current_starts_valid_escape() {
            self.consume_name_escape();
        } else {
            self.bump_current();
        }
    }

    fn consume_name_continue_sequence(&mut self) {
        loop {
            if self.current_starts_valid_escape() {
                self.consume_name_escape();
            } else if matches!(self.current_char(), Some(char) if is_name_continue(char)) {
                self.bump_current();
            } else {
                break;
            }
        }
    }

    fn consume_name_escape(&mut self) {
        self.bump_current();
        let mut hex_digits = 0usize;
        while hex_digits < 6
            && matches!(self.current_char(), Some(char) if char.is_ascii_hexdigit())
        {
            self.bump_current();
            hex_digits += 1;
        }
        if hex_digits > 0 {
            if matches!(self.current_char(), Some(char) if char.is_whitespace()) {
                self.bump_current();
            }
        } else if self.current_char().is_some() {
            self.bump_current();
        }
    }

    fn consume_url_token(&mut self, start: usize) {
        self.bump_current();
        while matches!(self.current_char(), Some(char) if char.is_whitespace()) {
            self.bump_current();
        }
        while let Some(char) = self.current_char() {
            match char {
                ')' => {
                    self.bump_current();
                    self.push(SyntaxKind::Url, start, self.offset);
                    return;
                }
                char if char.is_whitespace() => {
                    self.bump_current();
                    while matches!(self.current_char(), Some(char) if char.is_whitespace()) {
                        self.bump_current();
                    }
                    if self.current_char() == Some(')') {
                        self.bump_current();
                        self.push(SyntaxKind::Url, start, self.offset);
                        return;
                    }
                    self.consume_bad_url(start);
                    return;
                }
                '"' | '\'' | '(' => {
                    self.consume_bad_url(start);
                    return;
                }
                '\\' if self.current_starts_valid_escape() => {
                    self.consume_name_escape();
                }
                '\\' => {
                    self.consume_bad_url(start);
                    return;
                }
                char if is_non_printable_code_point(char) => {
                    self.consume_bad_url(start);
                    return;
                }
                _ => self.bump_current(),
            }
        }
        self.push(SyntaxKind::BadUrl, start, self.offset);
        self.error(
            ParseErrorCode::UnexpectedCharacter,
            start,
            self.offset,
            "unterminated url token",
        );
    }

    fn consume_bad_url(&mut self, start: usize) {
        while let Some(char) = self.current_char() {
            if char == ')' {
                self.bump_current();
                break;
            }
            if self.current_starts_valid_escape() {
                self.consume_name_escape();
            } else {
                self.bump_current();
            }
        }
        self.push(SyntaxKind::BadUrl, start, self.offset);
        self.error(
            ParseErrorCode::UnexpectedCharacter,
            start,
            self.offset,
            "bad url token",
        );
    }

    fn url_starts_with_quoted_argument(&self) -> bool {
        let Some(mut rest) = self.text.get(self.offset + '('.len_utf8()..) else {
            return false;
        };
        rest = rest.trim_start_matches(char::is_whitespace);
        matches!(rest.chars().next(), Some('"' | '\''))
    }

    fn starts_less_property_variable(&self) -> bool {
        self.extension.dialect() == StyleDialect::Less
            && self.text[self.offset + '$'.len_utf8()..]
                .chars()
                .next()
                .is_some_and(is_name_start)
    }

    fn starts_scss_placeholder(&self) -> bool {
        matches!(
            self.extension.dialect(),
            StyleDialect::Scss | StyleDialect::Sass
        ) && self.text[self.offset + '%'.len_utf8()..]
            .chars()
            .next()
            .is_some_and(is_name_start)
    }

    fn current_hash_starts_name(&self) -> bool {
        if self.current_char() != Some('#') {
            return false;
        }
        let next_offset = self.offset + '#'.len_utf8();
        self.text[next_offset..]
            .chars()
            .next()
            .is_some_and(is_name_continue)
            || self.escape_starts_at(next_offset)
    }

    fn consume_unexpected(&mut self, char: char) {
        let start = self.offset;
        self.bump_char(char);
        self.push(SyntaxKind::Delim, start, self.offset);
        self.error(
            ParseErrorCode::UnexpectedCharacter,
            start,
            self.offset,
            "unexpected character",
        );
    }

    fn push(&mut self, kind: SyntaxKind, start: usize, end: usize) {
        self.tokens.push(Token {
            kind,
            text: &self.text[start..end],
            range: text_range(start, end),
        });
    }

    fn error(&mut self, code: ParseErrorCode, start: usize, end: usize, message: &'static str) {
        self.errors.push(ParseError {
            code,
            range: text_range(start, end),
            message,
        });
    }

    fn starts_with(&self, pattern: &str) -> bool {
        self.text[self.offset..].starts_with(pattern)
    }

    fn current_starts_valid_escape(&self) -> bool {
        self.escape_starts_at(self.offset)
    }

    fn current_starts_number(&self) -> bool {
        self.starts_number_at(self.offset)
    }

    fn current_starts_number_exponent(&self) -> bool {
        let Some('e' | 'E') = self.current_char() else {
            return false;
        };
        let exponent_offset = self.offset + 'e'.len_utf8();
        self.char_at(exponent_offset)
            .is_some_and(|char| char.is_ascii_digit())
            || (matches!(self.char_at(exponent_offset), Some('+' | '-'))
                && self.char_after_offset_is_ascii_digit(exponent_offset))
    }

    fn starts_number_at(&self, offset: usize) -> bool {
        let Some(first) = self.char_at(offset) else {
            return false;
        };
        let second_offset = offset + first.len_utf8();
        match first {
            '+' | '-' => {
                self.char_at(second_offset)
                    .is_some_and(|char| char.is_ascii_digit())
                    || (self.char_at(second_offset) == Some('.')
                        && self.char_after_offset_is_ascii_digit(second_offset))
            }
            '.' => self.char_after_offset_is_ascii_digit(offset),
            char => char.is_ascii_digit(),
        }
    }

    fn current_starts_ident_sequence(&self) -> bool {
        self.starts_ident_sequence_at(self.offset)
    }

    fn starts_ident_sequence_at(&self, offset: usize) -> bool {
        let Some(first) = self.char_at(offset) else {
            return false;
        };
        let second_offset = offset + first.len_utf8();
        match first {
            '-' => {
                self.char_at(second_offset)
                    .is_some_and(|char| char == '-' || is_name_start(char))
                    || self.escape_starts_at(second_offset)
            }
            '\\' => self.escape_starts_at(offset),
            char => is_name_start(char),
        }
    }

    fn escape_starts_at(&self, offset: usize) -> bool {
        if !self
            .text
            .get(offset..)
            .is_some_and(|remaining| remaining.starts_with('\\'))
        {
            return false;
        }
        self.text[offset + '\\'.len_utf8()..]
            .chars()
            .next()
            .is_some_and(|char| !matches!(char, '\n' | '\r' | '\u{000c}'))
    }

    fn char_at(&self, offset: usize) -> Option<char> {
        self.text.get(offset..)?.chars().next()
    }

    fn char_after_current_is_ascii_digit(&self) -> bool {
        self.char_after_offset_is_ascii_digit(self.offset)
    }

    fn char_after_offset_is_ascii_digit(&self, offset: usize) -> bool {
        let Some(char) = self.char_at(offset) else {
            return false;
        };
        self.char_at(offset + char.len_utf8())
            .is_some_and(|char| char.is_ascii_digit())
    }

    fn starts_with_ascii_keyword(&self, keyword: &str) -> bool {
        let remaining = &self.text[self.offset..];
        let Some(prefix) = remaining.get(..keyword.len()) else {
            return false;
        };
        if !prefix.eq_ignore_ascii_case(keyword) {
            return false;
        }
        remaining[keyword.len()..]
            .chars()
            .next()
            .is_none_or(|char| !is_name_continue(char))
    }

    fn supports_scss_interpolation(&self) -> bool {
        matches!(
            self.extension.dialect(),
            StyleDialect::Scss | StyleDialect::Sass
        )
    }

    fn supports_less_interpolation(&self) -> bool {
        self.extension.dialect() == StyleDialect::Less
    }

    fn starts_less_escaped_string(&self) -> bool {
        self.extension.dialect() == StyleDialect::Less
            && (self.starts_with("~\"") || self.starts_with("~'"))
    }

    fn starts_unicode_range(&self) -> bool {
        let mut chars = self.text[self.offset..].chars();
        matches!(chars.next(), Some('u' | 'U'))
            && chars.next() == Some('+')
            && chars
                .next()
                .is_some_and(|char| char.is_ascii_hexdigit() || char == '?')
    }

    fn current_char(&self) -> Option<char> {
        self.text[self.offset..].chars().next()
    }

    fn next_char_is_hex_digit(&self) -> bool {
        let offset = self.offset + '-'.len_utf8();
        self.text
            .get(offset..)
            .and_then(|tail| tail.chars().next())
            .is_some_and(|char| char.is_ascii_hexdigit())
    }

    fn bump_current(&mut self) {
        if let Some(char) = self.current_char() {
            self.bump_char(char);
        }
    }

    fn bump_char(&mut self, char: char) {
        self.offset += char.len_utf8();
    }
}

fn public_token_text(text: &str) -> String {
    text.chars()
        .map(css_syntax_preprocessed_char)
        .collect::<String>()
}

fn css_syntax_preprocessed_char(char: char) -> char {
    if char == '\0' { '\u{fffd}' } else { char }
}

fn is_name_start(char: char) -> bool {
    let char = css_syntax_preprocessed_char(char);
    char == '_' || char == '-' || char.is_alphabetic() || !char.is_ascii()
}

fn is_name_continue(char: char) -> bool {
    is_name_start(char) || char.is_ascii_digit()
}

fn is_non_printable_code_point(char: char) -> bool {
    let char = css_syntax_preprocessed_char(char);
    matches!(char, '\u{0000}'..='\u{0008}' | '\u{000b}' | '\u{000e}'..='\u{001f}' | '\u{007f}')
}

fn is_custom_property_name_text(text: &str) -> bool {
    let Some(rest) = text.strip_prefix("--") else {
        return false;
    };
    let Some(first) = rest.chars().next() else {
        return false;
    };
    first == '-' || is_name_start(first) || starts_valid_escape_text(rest)
}

fn starts_valid_escape_text(text: &str) -> bool {
    text.starts_with('\\')
        && text['\\'.len_utf8()..]
            .chars()
            .next()
            .is_some_and(|char| !matches!(char, '\n' | '\r' | '\u{000c}'))
}

fn is_css_at_rule_name(text: &str) -> bool {
    matches_ignore_ascii_case(
        text,
        &[
            "@charset",
            "@container",
            "@font-face",
            "@font-feature-values",
            "@font-palette-values",
            "@import",
            "@keyframes",
            "@layer",
            "@media",
            "@namespace",
            "@page",
            "@property",
            "@scope",
            "@starting-style",
            "@supports",
            "@counter-style",
            "@custom-media",
            "@color-profile",
            "@nest",
            "@position-try",
            "@view-transition",
            "@stylistic",
            "@styleset",
            "@character-variant",
            "@swash",
            "@ornaments",
            "@annotation",
            "@historical-forms",
            "@when",
            "@else",
        ],
    )
}

fn is_interpolation_start(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::ScssInterpolationStart | SyntaxKind::LessInterpolationStart
    )
}

fn is_component_value_atom_start(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::Ident
            | SyntaxKind::CustomPropertyName
            | SyntaxKind::Number
            | SyntaxKind::Percentage
            | SyntaxKind::Dimension
            | SyntaxKind::String
            | SyntaxKind::LessEscapedString
            | SyntaxKind::UnicodeRange
            | SyntaxKind::Hash
            | SyntaxKind::Url
            | SyntaxKind::BadUrl
            | SyntaxKind::BadString
            | SyntaxKind::Important
            | SyntaxKind::ScssVariable
            | SyntaxKind::LessVariable
            | SyntaxKind::LessPropertyVariableToken
            | SyntaxKind::ScssInterpolationStart
            | SyntaxKind::LessInterpolationStart
    )
}

fn interpolation_end_kind(start_kind: SyntaxKind) -> Option<SyntaxKind> {
    match start_kind {
        SyntaxKind::ScssInterpolationStart => Some(SyntaxKind::ScssInterpolationEnd),
        SyntaxKind::LessInterpolationStart => Some(SyntaxKind::LessInterpolationEnd),
        _ => None,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SelectorBranch {
    name: String,
    range: TextRange,
    bare_suffix_base: bool,
}

fn collect_selector_facts_from_tokens(tokens: &[Token<'_>]) -> Vec<ParsedSelectorFact> {
    let mut selectors = Vec::new();
    let mut seen = BTreeSet::new();
    collect_selector_facts_in_range(
        tokens,
        0,
        tokens.len(),
        &[],
        None,
        &mut seen,
        &mut selectors,
    );
    selectors
}

fn collect_selector_facts_in_range(
    tokens: &[Token<'_>],
    start: usize,
    end: usize,
    parent_branches: &[SelectorBranch],
    css_module_scope: Option<&'static str>,
    seen: &mut BTreeSet<(ParsedSelectorFactKind, String, u32, u32)>,
    selectors: &mut Vec<ParsedSelectorFact>,
) {
    let mut index = start;
    while index < end {
        index = skip_trivia_tokens(tokens, index, end);
        if index >= end {
            break;
        }

        if tokens[index].kind == SyntaxKind::AtKeyword {
            let block = find_block_after_header(tokens, index, end);
            if let Some((open, close)) = block {
                if tokens[index].text == "@nest" {
                    if css_module_scope == Some("global") {
                        collect_selector_facts_in_range(
                            tokens,
                            open + 1,
                            close,
                            &[],
                            css_module_scope,
                            seen,
                            selectors,
                        );
                    } else {
                        let branches =
                            resolve_selector_header(tokens, index + 1, open, parent_branches);
                        push_class_selector_facts_from_header(
                            selectors,
                            seen,
                            tokens,
                            index + 1,
                            open,
                        );
                        for branch in &branches {
                            push_selector_fact(
                                selectors,
                                seen,
                                ParsedSelectorFactKind::Class,
                                branch.name.clone(),
                                branch.range,
                            );
                        }
                        collect_selector_facts_in_range(
                            tokens,
                            open + 1,
                            close,
                            &branches,
                            css_module_scope,
                            seen,
                            selectors,
                        );
                    }
                } else if style_wrapper_at_rule(tokens[index].text) {
                    collect_selector_facts_in_range(
                        tokens,
                        open + 1,
                        close,
                        parent_branches,
                        css_module_scope,
                        seen,
                        selectors,
                    );
                }
                index = close + 1;
            } else {
                index = skip_statement(tokens, index, end);
            }
            continue;
        }

        let Some((open, close)) = find_block_after_header(tokens, index, end) else {
            index = skip_statement(tokens, index, end);
            continue;
        };

        let effective_scope = css_module_scope
            .or_else(|| css_module_block_scope_marker_in_header(tokens, index, open));
        if effective_scope == Some("global") {
            collect_selector_facts_in_range(
                tokens,
                open + 1,
                close,
                &[],
                effective_scope,
                seen,
                selectors,
            );
        } else {
            let branches = resolve_selector_header(tokens, index, open, parent_branches);
            push_class_selector_facts_from_header(selectors, seen, tokens, index, open);
            for branch in &branches {
                push_selector_fact(
                    selectors,
                    seen,
                    ParsedSelectorFactKind::Class,
                    branch.name.clone(),
                    branch.range,
                );
            }
            for id in collect_id_selector_facts_from_header(tokens, index, open)
                .into_iter()
                .chain(collect_local_function_id_selector_facts_from_header(
                    tokens, index, open,
                ))
            {
                push_selector_fact(selectors, seen, ParsedSelectorFactKind::Id, id.0, id.1);
            }
            for placeholder in collect_placeholder_selector_facts_from_header(tokens, index, open) {
                push_selector_fact(
                    selectors,
                    seen,
                    ParsedSelectorFactKind::Placeholder,
                    placeholder.0,
                    placeholder.1,
                );
            }

            collect_selector_facts_in_range(
                tokens,
                open + 1,
                close,
                &branches,
                effective_scope,
                seen,
                selectors,
            );
        }
        index = close + 1;
    }
}

fn push_class_selector_facts_from_header(
    selectors: &mut Vec<ParsedSelectorFact>,
    seen: &mut BTreeSet<(ParsedSelectorFactKind, String, u32, u32)>,
    tokens: &[Token<'_>],
    start: usize,
    end: usize,
) {
    for (name, range) in collect_class_selector_names_from_header(tokens, start, end) {
        push_selector_fact(selectors, seen, ParsedSelectorFactKind::Class, name, range);
    }
}

fn push_selector_fact(
    selectors: &mut Vec<ParsedSelectorFact>,
    seen: &mut BTreeSet<(ParsedSelectorFactKind, String, u32, u32)>,
    kind: ParsedSelectorFactKind,
    name: String,
    range: TextRange,
) {
    if seen.insert((
        kind,
        name.clone(),
        u32::from(range.start()),
        u32::from(range.end()),
    )) {
        selectors.push(ParsedSelectorFact { kind, name, range });
    }
}

fn resolve_selector_header(
    tokens: &[Token<'_>],
    start: usize,
    end: usize,
    parent_branches: &[SelectorBranch],
) -> Vec<SelectorBranch> {
    split_selector_groups(tokens, start, end)
        .into_iter()
        .flat_map(|(group_start, group_end)| {
            resolve_selector_group(tokens, group_start, group_end, parent_branches)
        })
        .collect()
}

fn resolve_selector_group(
    tokens: &[Token<'_>],
    start: usize,
    end: usize,
    parent_branches: &[SelectorBranch],
) -> Vec<SelectorBranch> {
    if let Some(mut local_names) = collect_local_function_selector_names(tokens, start, end) {
        local_names.extend(collect_class_selector_names_from_header(tokens, start, end));
        let bare_suffix_base = parent_branches.is_empty() && local_names.len() == 1;
        return local_names
            .into_iter()
            .map(|(name, range)| SelectorBranch {
                name,
                range,
                bare_suffix_base,
            })
            .collect();
    }

    let (tail_start, tail_end) = selector_group_tail_range(tokens, start, end);
    let tail_start = skip_trivia_tokens(tokens, tail_start, tail_end);

    if let Some((suffix, range)) = ampersand_suffix_selector(tokens, tail_start, tail_end) {
        let bases: Vec<&SelectorBranch> = if parent_branches.is_empty() {
            Vec::new()
        } else {
            parent_branches
                .iter()
                .filter(|parent| parent.bare_suffix_base)
                .collect()
        };
        return bases
            .into_iter()
            .map(|parent| SelectorBranch {
                name: format!("{}{}", parent.name, suffix),
                range,
                bare_suffix_base: parent.bare_suffix_base,
            })
            .collect();
    }

    let class_names = collect_class_selector_names_from_header(tokens, tail_start, tail_end);
    if class_names.is_empty() {
        return Vec::new();
    }

    let bare_suffix_base = parent_branches.is_empty()
        && class_names.len() == 1
        && is_bare_class_selector_group(tokens, tail_start, tail_end);
    class_names
        .into_iter()
        .map(|(name, range)| SelectorBranch {
            name,
            range,
            bare_suffix_base,
        })
        .collect()
}

fn is_bare_class_selector_group(tokens: &[Token<'_>], start: usize, end: usize) -> bool {
    let dot_index = skip_trivia_tokens(tokens, start, end);
    if tokens.get(dot_index).map(|token| token.kind) != Some(SyntaxKind::Dot) {
        return false;
    }
    let name_index = skip_trivia_tokens(tokens, dot_index + 1, end);
    if !tokens.get(name_index).is_some_and(|token| {
        matches!(
            token.kind,
            SyntaxKind::Ident | SyntaxKind::CustomPropertyName
        )
    }) {
        return false;
    }
    skip_trivia_tokens(tokens, name_index + 1, end) >= end
}

fn split_selector_groups(tokens: &[Token<'_>], start: usize, end: usize) -> Vec<(usize, usize)> {
    let mut groups = Vec::new();
    let mut group_start = start;
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;
    let mut index = start;
    while index < end {
        match tokens[index].kind {
            SyntaxKind::LeftParen => paren_depth += 1,
            SyntaxKind::RightParen => paren_depth = paren_depth.saturating_sub(1),
            SyntaxKind::LeftBracket => bracket_depth += 1,
            SyntaxKind::RightBracket => bracket_depth = bracket_depth.saturating_sub(1),
            SyntaxKind::Comma if paren_depth == 0 && bracket_depth == 0 => {
                groups.push((group_start, index));
                group_start = index + 1;
            }
            _ => {}
        }
        index += 1;
    }
    groups.push((group_start, end));
    groups
}

fn selector_group_tail_range(tokens: &[Token<'_>], start: usize, end: usize) -> (usize, usize) {
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;
    let mut tail_start = start;
    let mut index = start;
    while index < end {
        match tokens[index].kind {
            SyntaxKind::LeftParen => paren_depth += 1,
            SyntaxKind::RightParen => paren_depth = paren_depth.saturating_sub(1),
            SyntaxKind::LeftBracket => bracket_depth += 1,
            SyntaxKind::RightBracket => bracket_depth = bracket_depth.saturating_sub(1),
            kind if paren_depth == 0 && bracket_depth == 0 && is_selector_combinator_kind(kind) => {
                tail_start = index + 1;
            }
            SyntaxKind::Whitespace if paren_depth == 0 && bracket_depth == 0 => {
                let previous = previous_non_trivia_token(tokens, start, index);
                let next = next_non_trivia_token_until(tokens, index + 1, end);
                if previous.is_some_and(|token| selector_component_can_end(token.kind))
                    && next.is_some_and(|token| selector_component_can_start(token.kind))
                {
                    tail_start = index + 1;
                }
            }
            _ => {}
        }
        index += 1;
    }
    (tail_start, end)
}

fn ampersand_suffix_selector(
    tokens: &[Token<'_>],
    start: usize,
    end: usize,
) -> Option<(String, TextRange)> {
    let ampersand_index = skip_trivia_tokens(tokens, start, end);
    if tokens.get(ampersand_index)?.kind != SyntaxKind::Ampersand {
        return None;
    }
    let suffix = next_non_trivia_token_until(tokens, ampersand_index + 1, end)?;
    if matches!(
        suffix.kind,
        SyntaxKind::Ident | SyntaxKind::CustomPropertyName
    ) {
        return Some((suffix.text.to_string(), suffix.range));
    }
    None
}

fn collect_class_selector_names_from_header(
    tokens: &[Token<'_>],
    start: usize,
    end: usize,
) -> Vec<(String, TextRange)> {
    let mut names = Vec::new();
    let mut index = start;
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;
    while index < end {
        match tokens[index].kind {
            SyntaxKind::LeftParen => paren_depth += 1,
            SyntaxKind::RightParen => paren_depth = paren_depth.saturating_sub(1),
            SyntaxKind::LeftBracket => bracket_depth += 1,
            SyntaxKind::RightBracket => bracket_depth = bracket_depth.saturating_sub(1),
            _ => {}
        }
        if paren_depth == 0
            && bracket_depth == 0
            && tokens[index].kind == SyntaxKind::Dot
            && let Some(name) = next_non_trivia_token_until(tokens, index + 1, end)
            && matches!(
                name.kind,
                SyntaxKind::Ident | SyntaxKind::CustomPropertyName
            )
        {
            names.push((name.text.to_string(), name.range));
        }
        index += 1;
    }
    names
}

fn collect_local_function_selector_names(
    tokens: &[Token<'_>],
    start: usize,
    end: usize,
) -> Option<Vec<(String, TextRange)>> {
    let colon_index = skip_trivia_tokens(tokens, start, end);
    if tokens.get(colon_index)?.kind != SyntaxKind::Colon {
        return None;
    }
    let ident = next_non_trivia_token_until(tokens, colon_index + 1, end)?;
    if ident.kind != SyntaxKind::Ident || ident.text != "local" {
        return None;
    }
    let open_index = skip_trivia_tokens(tokens, colon_index + 2, end);
    if tokens.get(open_index)?.kind != SyntaxKind::LeftParen {
        return None;
    }
    Some(collect_class_selector_names_from_header(
        tokens,
        open_index + 1,
        end.saturating_sub(1),
    ))
}

fn collect_local_function_id_selector_facts_from_header(
    tokens: &[Token<'_>],
    start: usize,
    end: usize,
) -> Vec<(String, TextRange)> {
    let mut ids = Vec::new();
    let mut index = start;
    while index < end {
        if tokens[index].kind == SyntaxKind::Colon
            && let Some(scope) = next_non_trivia_token_until(tokens, index + 1, end)
            && scope.kind == SyntaxKind::Ident
            && scope.text == "local"
            && let Some(open) = next_non_trivia_token_after_range(tokens, scope.range, end)
            && open.kind == SyntaxKind::LeftParen
            && let Some(close) = matching_right_paren_from_range(tokens, open.range, end)
        {
            ids.extend(collect_id_selector_facts_from_header(
                tokens,
                token_index_by_range(tokens, open.range).map_or(index + 1, |value| value + 1),
                close,
            ));
            index = close.saturating_add(1);
            continue;
        }
        index += 1;
    }
    ids
}

fn css_module_block_scope_marker_in_header(
    tokens: &[Token<'_>],
    start: usize,
    end: usize,
) -> Option<&'static str> {
    if next_non_trivia_token_until(tokens, start, end)
        .is_some_and(|token| token.kind == SyntaxKind::AtKeyword)
    {
        return None;
    }

    css_module_scope_marker_after_colon(tokens, start, end)
        .filter(|_| !css_module_scope_marker_is_function(tokens, start, end))
}

fn css_module_header_is_global_only(tokens: &[Token<'_>], start: usize, end: usize) -> bool {
    if next_non_trivia_token_until(tokens, start, end)
        .is_some_and(|token| token.kind == SyntaxKind::AtKeyword)
    {
        return false;
    }
    css_module_header_contains_scope(tokens, start, end, "global")
        && collect_class_selector_names_from_header(tokens, start, end).is_empty()
        && collect_local_function_selector_names(tokens, start, end)
            .map(|names| names.is_empty())
            .unwrap_or(true)
}

fn css_module_header_contains_scope(
    tokens: &[Token<'_>],
    start: usize,
    end: usize,
    expected_scope: &str,
) -> bool {
    let mut index = start;
    while index < end {
        if tokens[index].kind == SyntaxKind::Colon
            && let Some(scope) = next_non_trivia_token_until(tokens, index + 1, end)
            && scope.kind == SyntaxKind::Ident
            && scope.text == expected_scope
        {
            return true;
        }
        index += 1;
    }
    false
}

fn css_module_scope_marker_after_colon(
    tokens: &[Token<'_>],
    start: usize,
    end: usize,
) -> Option<&'static str> {
    let colon = skip_trivia_tokens(tokens, start, end);
    if tokens.get(colon)?.kind != SyntaxKind::Colon {
        return None;
    }
    let scope = next_non_trivia_token_until(tokens, colon + 1, end)?;
    if scope.kind != SyntaxKind::Ident {
        return None;
    }
    match scope.text {
        "global" => Some("global"),
        "local" => Some("local"),
        _ => None,
    }
}

fn css_module_scope_marker_is_function(tokens: &[Token<'_>], start: usize, end: usize) -> bool {
    let colon = skip_trivia_tokens(tokens, start, end);
    let mut index = colon + 1;
    let Some(scope) = next_non_trivia_token_until(tokens, index, end) else {
        return false;
    };
    while index < end {
        if tokens[index].range == scope.range {
            break;
        }
        index += 1;
    }
    let Some(next) = next_non_trivia_token_until(tokens, index + 1, end) else {
        return false;
    };
    scope.kind == SyntaxKind::Ident && next.kind == SyntaxKind::LeftParen
}

fn collect_id_selector_facts_from_header(
    tokens: &[Token<'_>],
    start: usize,
    end: usize,
) -> Vec<(String, TextRange)> {
    let mut names = Vec::new();
    let mut index = start;
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;
    while index < end {
        match tokens[index].kind {
            SyntaxKind::LeftParen => paren_depth += 1,
            SyntaxKind::RightParen => paren_depth = paren_depth.saturating_sub(1),
            SyntaxKind::LeftBracket => bracket_depth += 1,
            SyntaxKind::RightBracket => bracket_depth = bracket_depth.saturating_sub(1),
            _ => {}
        }
        let token = tokens[index];
        if paren_depth == 0 && bracket_depth == 0 && token.kind == SyntaxKind::Hash {
            names.push((token.text.trim_start_matches('#').to_string(), token.range));
        }
        index += 1;
    }
    names
}

fn collect_placeholder_selector_facts_from_header(
    tokens: &[Token<'_>],
    start: usize,
    end: usize,
) -> Vec<(String, TextRange)> {
    let mut names = Vec::new();
    let mut index = start;
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;
    while index < end {
        match tokens[index].kind {
            SyntaxKind::LeftParen => paren_depth += 1,
            SyntaxKind::RightParen => paren_depth = paren_depth.saturating_sub(1),
            SyntaxKind::LeftBracket => bracket_depth += 1,
            SyntaxKind::RightBracket => bracket_depth = bracket_depth.saturating_sub(1),
            _ => {}
        }
        let token = tokens[index];
        if paren_depth == 0 && bracket_depth == 0 && token.kind == SyntaxKind::ScssPlaceholder {
            names.push((token.text.trim_start_matches('%').to_string(), token.range));
        }
        index += 1;
    }
    names
}

fn collect_variable_facts_from_tokens(tokens: &[Token<'_>]) -> Vec<ParsedVariableFact> {
    let mut variables = Vec::new();
    for (index, token) in tokens.iter().enumerate() {
        let kind = match token.kind {
            SyntaxKind::ScssVariable => {
                if scss_variable_token_is_declaration(tokens, index) {
                    ParsedVariableFactKind::ScssDeclaration
                } else {
                    ParsedVariableFactKind::ScssReference
                }
            }
            SyntaxKind::LessVariable => {
                if next_non_trivia_token(tokens, index + 1)
                    .is_some_and(|candidate| candidate.kind == SyntaxKind::Colon)
                {
                    ParsedVariableFactKind::LessDeclaration
                } else {
                    ParsedVariableFactKind::LessReference
                }
            }
            SyntaxKind::CustomPropertyName => {
                if previous_non_trivia_token(tokens, 0, index).is_some_and(|candidate| {
                    matches!(candidate.kind, SyntaxKind::Ampersand | SyntaxKind::Dot)
                }) {
                    continue;
                }
                if let Some(at_rule_name) = containing_at_rule_header_name(tokens, index) {
                    if at_rule_name == "@property" {
                        ParsedVariableFactKind::CustomPropertyDeclaration
                    } else {
                        continue;
                    }
                } else if next_non_trivia_token(tokens, index + 1)
                    .is_some_and(|candidate| candidate.kind == SyntaxKind::Colon)
                {
                    ParsedVariableFactKind::CustomPropertyDeclaration
                } else {
                    ParsedVariableFactKind::CustomPropertyReference
                }
            }
            _ => continue,
        };
        let has_fallback = kind == ParsedVariableFactKind::CustomPropertyReference
            && custom_property_reference_has_var_fallback(tokens, index);
        variables.push(ParsedVariableFact {
            kind,
            name: token.text.to_string(),
            range: token.range,
            has_fallback,
        });
    }
    variables
}

/// Detect a `var(--x, fallback)` fallback for the `CustomPropertyName` at `index`.
///
/// True iff the reference is the first argument of an enclosing `var(` call *and* a
/// top-level comma follows it before that call's closing paren. Scoped per-`var()`: in
/// `var(--a, var(--b))` only `--a` carries a fallback; the nested `--b` (no fallback of
/// its own) is unaffected and stays a live `missingCustomProperty` candidate.
fn custom_property_reference_has_var_fallback(tokens: &[Token<'_>], index: usize) -> bool {
    // The reference must be the leading argument of a `var(` call: its immediate
    // non-trivia predecessor is `(`, preceded by an identifier `var`.
    let Some(open_index) = previous_non_trivia_token_index(tokens, index, 0) else {
        return false;
    };
    if tokens[open_index].kind != SyntaxKind::LeftParen {
        return false;
    }
    let Some(callee_index) = previous_non_trivia_token_index(tokens, open_index, 0) else {
        return false;
    };
    if tokens[callee_index].kind != SyntaxKind::Ident
        || !tokens[callee_index].text.eq_ignore_ascii_case("var")
    {
        return false;
    }
    // Scan forward at this call's paren depth for a top-level comma before its close.
    let mut depth = 0usize;
    let mut cursor = open_index;
    while cursor < tokens.len() {
        match tokens[cursor].kind {
            SyntaxKind::LeftParen => depth += 1,
            SyntaxKind::RightParen => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return false;
                }
            }
            SyntaxKind::Comma if depth == 1 => return true,
            _ => {}
        }
        cursor += 1;
    }
    false
}

fn scss_variable_token_is_declaration(tokens: &[Token<'_>], index: usize) -> bool {
    if scss_loop_variable_token_is_binding(tokens, index) {
        return true;
    }
    next_non_trivia_token(tokens, index + 1).is_some_and(|candidate| {
        candidate.kind == SyntaxKind::Colon
            || (matches!(candidate.kind, SyntaxKind::Comma | SyntaxKind::RightParen)
                && containing_at_rule_header_name(tokens, index).is_some_and(|name| {
                    name.eq_ignore_ascii_case("@mixin") || name.eq_ignore_ascii_case("@function")
                }))
    })
}

/// Positional guard for `@each` / `@for` loop bindings.
///
/// In `@each $k, $v in $map` the `$k`/`$v` are *bindings* (declarations), while
/// the iterable `$map` after `in` is a *reference*. In `@for $i from $start
/// through $end` the `$i` is a binding, while `$start`/`$end` after `from` are
/// references. A `$var` is a binding iff it sits in the loop header *before* the
/// top-level separator keyword (`in` for `@each`, `from` for `@for`). `@while` /
/// `@if` headers introduce no bindings and stay reference-only.
fn scss_loop_variable_token_is_binding(tokens: &[Token<'_>], index: usize) -> bool {
    let Some(header_index) = containing_at_rule_header_index(tokens, index) else {
        return false;
    };
    let separator = match () {
        _ if tokens[header_index].text.eq_ignore_ascii_case("@each") => "in",
        _ if tokens[header_index].text.eq_ignore_ascii_case("@for") => "from",
        _ => return false,
    };
    // Scan the header from just after the at-keyword up to (but excluding) the
    // variable token. If the top-level separator keyword has already appeared,
    // the variable is part of the iterable/bounds expression -> reference.
    let mut paren_depth = 0usize;
    for token in &tokens[header_index + 1..index] {
        match token.kind {
            SyntaxKind::LeftParen => paren_depth += 1,
            SyntaxKind::RightParen => paren_depth = paren_depth.saturating_sub(1),
            SyntaxKind::Ident if paren_depth == 0 && token.text.eq_ignore_ascii_case(separator) => {
                return false;
            }
            _ => {}
        }
    }
    true
}

/// Like [`containing_at_rule_header_name`] but returns the index of the
/// enclosing `@`-keyword token rather than its text.
fn containing_at_rule_header_index(tokens: &[Token<'_>], index: usize) -> Option<usize> {
    let mut current = index;
    while current > 0 {
        current -= 1;
        let token = tokens.get(current)?;
        if token.kind.is_trivia() {
            continue;
        }
        if matches!(
            token.kind,
            SyntaxKind::Semicolon
                | SyntaxKind::SassOptionalSemicolon
                | SyntaxKind::LeftBrace
                | SyntaxKind::RightBrace
                | SyntaxKind::SassIndent
                | SyntaxKind::SassDedent
        ) {
            return None;
        }
        if token.kind == SyntaxKind::AtKeyword {
            return Some(current);
        }
    }
    None
}

fn collect_sass_symbol_facts_from_tokens(tokens: &[Token<'_>]) -> Vec<ParsedSassSymbolFact> {
    let declared_functions = collect_sass_callable_declaration_names(tokens, "@function");
    let mut symbols = Vec::new();

    for (index, token) in tokens.iter().enumerate() {
        match token.kind {
            SyntaxKind::ScssVariable => {
                let kind = if scss_variable_token_is_declaration(tokens, index) {
                    ParsedSassSymbolFactKind::VariableDeclaration
                } else {
                    ParsedSassSymbolFactKind::VariableReference
                };
                let namespace = (!scss_variable_token_is_declaration(tokens, index))
                    .then(|| sass_member_namespace_before(tokens, index))
                    .flatten();
                symbols.push(ParsedSassSymbolFact {
                    kind,
                    symbol_kind: "variable",
                    name: token.text.trim_start_matches('$').to_string(),
                    role: match kind {
                        ParsedSassSymbolFactKind::VariableDeclaration => "declaration",
                        _ => "reference",
                    },
                    namespace,
                    range: sass_symbol_variable_range(token, kind),
                });
            }
            SyntaxKind::AtKeyword if token.text.eq_ignore_ascii_case("@mixin") => {
                if let Some(name) = sass_callable_name_after_at_rule(tokens, index) {
                    symbols.push(ParsedSassSymbolFact {
                        kind: ParsedSassSymbolFactKind::MixinDeclaration,
                        symbol_kind: "mixin",
                        name: name.text.to_string(),
                        role: "declaration",
                        namespace: None,
                        range: name.range,
                    });
                }
            }
            SyntaxKind::AtKeyword if token.text.eq_ignore_ascii_case("@include") => {
                if let Some((name, namespace)) = sass_include_name_after_at_rule(tokens, index) {
                    symbols.push(ParsedSassSymbolFact {
                        kind: ParsedSassSymbolFactKind::MixinInclude,
                        symbol_kind: "mixin",
                        name: name.text.to_string(),
                        role: "include",
                        namespace,
                        range: name.range,
                    });
                }
            }
            SyntaxKind::AtKeyword if token.text.eq_ignore_ascii_case("@function") => {
                if let Some(name) = sass_callable_name_after_at_rule(tokens, index) {
                    symbols.push(ParsedSassSymbolFact {
                        kind: ParsedSassSymbolFactKind::FunctionDeclaration,
                        symbol_kind: "function",
                        name: name.text.to_string(),
                        role: "declaration",
                        namespace: None,
                        range: name.range,
                    });
                }
            }
            SyntaxKind::Ident
                if (declared_functions.contains(token.text)
                    || sass_member_namespace_before(tokens, index).is_some())
                    && next_non_trivia_token(tokens, index + 1)
                        .is_some_and(|candidate| candidate.kind == SyntaxKind::LeftParen)
                    && !containing_at_rule_header_name(tokens, index)
                        .is_some_and(|name| name.eq_ignore_ascii_case("@include"))
                    && previous_non_trivia_token(tokens, 0, index).is_none_or(|candidate| {
                        !matches!(candidate.kind, SyntaxKind::AtKeyword)
                    }) =>
            {
                symbols.push(ParsedSassSymbolFact {
                    kind: ParsedSassSymbolFactKind::FunctionCall,
                    symbol_kind: "function",
                    name: token.text.to_string(),
                    role: "call",
                    namespace: sass_member_namespace_before(tokens, index),
                    range: token.range,
                });
            }
            _ => {}
        }
    }

    symbols
}

fn sass_symbol_variable_range(token: &Token<'_>, kind: ParsedSassSymbolFactKind) -> TextRange {
    if kind == ParsedSassSymbolFactKind::VariableDeclaration && token.text.starts_with('$') {
        let start = u32::from(token.range.start());
        let end = u32::from(token.range.end());
        if start < end {
            return TextRange::new(TextSize::from(start + 1), TextSize::from(end));
        }
    }
    token.range
}

fn collect_sass_callable_declaration_names(
    tokens: &[Token<'_>],
    at_keyword: &str,
) -> BTreeSet<String> {
    tokens
        .iter()
        .enumerate()
        .filter_map(|(index, token)| {
            (token.kind == SyntaxKind::AtKeyword && token.text.eq_ignore_ascii_case(at_keyword))
                .then(|| sass_callable_name_after_at_rule(tokens, index))
                .flatten()
                .map(|name| name.text.to_string())
        })
        .collect()
}

fn sass_callable_name_after_at_rule<'text>(
    tokens: &[Token<'text>],
    at_rule_index: usize,
) -> Option<Token<'text>> {
    let statement_end = css_module_value_statement_end(tokens, at_rule_index + 1);
    let name_index = next_non_trivia_token_index_until(tokens, at_rule_index + 1, statement_end)?;
    let name = tokens[name_index];
    if name.kind != SyntaxKind::Ident {
        return None;
    }
    if next_non_trivia_token_index_until(tokens, name_index + 1, statement_end)
        .is_some_and(|next| tokens[next].kind == SyntaxKind::Dot)
    {
        return None;
    }
    Some(name)
}

fn sass_include_name_after_at_rule<'text>(
    tokens: &[Token<'text>],
    at_rule_index: usize,
) -> Option<(Token<'text>, Option<String>)> {
    let statement_end = css_module_value_statement_end(tokens, at_rule_index + 1);
    let first_index = next_non_trivia_token_index_until(tokens, at_rule_index + 1, statement_end)?;
    let first = tokens[first_index];
    if first.kind != SyntaxKind::Ident {
        return None;
    }
    let Some(dot_index) = next_non_trivia_token_index_until(tokens, first_index + 1, statement_end)
    else {
        return Some((first, None));
    };
    if tokens[dot_index].kind != SyntaxKind::Dot {
        return Some((first, None));
    }
    let member_index = next_non_trivia_token_index_until(tokens, dot_index + 1, statement_end)?;
    let member = tokens[member_index];
    (member.kind == SyntaxKind::Ident).then(|| (member, Some(first.text.to_string())))
}

fn sass_member_namespace_before(tokens: &[Token<'_>], member_index: usize) -> Option<String> {
    let dot_index = previous_non_trivia_token_index(tokens, member_index, 0)?;
    if tokens[dot_index].kind != SyntaxKind::Dot {
        return None;
    }
    let namespace = tokens[previous_non_trivia_token_index(tokens, dot_index, 0)?];
    (namespace.kind == SyntaxKind::Ident).then(|| namespace.text.to_string())
}

fn collect_sass_include_facts_from_tokens(
    source: &str,
    tokens: &[Token<'_>],
) -> Vec<ParsedSassIncludeFact> {
    let mut includes = Vec::new();
    for (index, token) in tokens.iter().enumerate() {
        if token.kind != SyntaxKind::AtKeyword || !token.text.eq_ignore_ascii_case("@include") {
            continue;
        }
        let statement_end = css_module_value_statement_end(tokens, index + 1);
        let Some((name, namespace)) = sass_include_name_after_at_rule(tokens, index) else {
            continue;
        };
        let header_end = previous_non_trivia_token_index(tokens, statement_end, index + 1)
            .map(|previous| tokens[previous].range.end())
            .unwrap_or(name.range.end());
        let params = source
            .get(u32::from(name.range.end()) as usize..u32::from(header_end) as usize)
            .unwrap_or_default()
            .trim()
            .to_string();
        includes.push(ParsedSassIncludeFact {
            name: name.text.to_string(),
            namespace,
            params,
            range: TextRange::new(token.range.start(), header_end),
        });
    }
    includes
}

fn collect_sass_module_edge_facts_from_tokens(
    tokens: &[Token<'_>],
) -> Vec<ParsedSassModuleEdgeFact> {
    let mut edges = Vec::new();
    let mut seen = BTreeSet::new();

    for (index, token) in tokens.iter().enumerate() {
        if token.kind != SyntaxKind::AtKeyword {
            continue;
        }
        let Some(kind) = sass_module_edge_kind(token.text) else {
            continue;
        };
        let start = skip_trivia_tokens(tokens, index + 1, tokens.len());
        let end = css_module_value_statement_end(tokens, start);
        if kind == ParsedSassModuleEdgeFactKind::Import {
            collect_sass_import_module_edges(tokens, start, end, &mut edges, &mut seen);
            continue;
        }
        let Some(source_index) = next_non_trivia_token_index_until(tokens, start, end) else {
            continue;
        };
        let source = tokens[source_index];
        if !matches!(source.kind, SyntaxKind::String | SyntaxKind::Url) {
            continue;
        }
        let source_name = css_module_value_source_name(source);
        let (namespace_kind, namespace) = if kind == ParsedSassModuleEdgeFactKind::Use {
            sass_module_use_namespace(tokens, source_name.as_str(), source_index + 1, end)
        } else {
            (None, None)
        };
        let (visibility_filter_kind, visibility_filter_names) =
            if kind == ParsedSassModuleEdgeFactKind::Forward {
                sass_module_forward_visibility_filter(tokens, source_index + 1, end)
            } else {
                (None, Vec::new())
            };
        let forward_prefix = if kind == ParsedSassModuleEdgeFactKind::Forward {
            sass_module_forward_prefix(tokens, source_index + 1, end)
        } else {
            None
        };
        push_sass_module_edge_fact(
            &mut edges,
            &mut seen,
            ParsedSassModuleEdgeFact {
                kind,
                source: source_name,
                namespace_kind,
                namespace,
                forward_prefix,
                visibility_filter_kind,
                visibility_filter_names,
                media_qualified: false,
                range: source.range,
            },
        );
    }

    edges
}

fn sass_module_edge_kind(text: &str) -> Option<ParsedSassModuleEdgeFactKind> {
    match text {
        text if text.eq_ignore_ascii_case("@use") => Some(ParsedSassModuleEdgeFactKind::Use),
        text if text.eq_ignore_ascii_case("@forward") => {
            Some(ParsedSassModuleEdgeFactKind::Forward)
        }
        text if text.eq_ignore_ascii_case("@import") => Some(ParsedSassModuleEdgeFactKind::Import),
        _ => None,
    }
}

/// RFC-0007-E1 (#45): capture the target of each `@extend` rule. For each `@extend` keyword, the
/// statement runs to the next `;`/`}`/indent boundary (`css_module_value_statement_end`). Within
/// it we capture the FIRST simple target — a `%placeholder` (one `ScssPlaceholder` token) or a
/// `.class` (`Dot` + `Ident`) — and record whether the statement carries the `!optional` flag
/// (`!` `optional`, anywhere in the statement). A compound target (`.a.b`) records only its first
/// simple selector; dart-sass rejects compound `@extend` targets outright, so the first-simple
/// capture is sufficient for the missing-target check and never over-reports. Interpolated targets
/// (`#{...}`) produce no simple token here and are skipped (not statically checkable).
fn collect_extend_target_facts_from_tokens(tokens: &[Token<'_>]) -> Vec<ParsedExtendTargetFact> {
    let mut targets = Vec::new();

    for (index, token) in tokens.iter().enumerate() {
        if token.kind != SyntaxKind::AtKeyword || !token.text.eq_ignore_ascii_case("@extend") {
            continue;
        }
        let start = skip_trivia_tokens(tokens, index + 1, tokens.len());
        let end = css_module_value_statement_end(tokens, start);

        // `!optional` may appear after the target; scan the whole statement for it first.
        let optional = extend_statement_has_optional_flag(tokens, start, end);

        // First simple target within the statement.
        let mut cursor = start;
        let mut captured: Option<ParsedExtendTargetFact> = None;
        while cursor < end {
            let current = tokens[cursor];
            if current.kind == SyntaxKind::ScssPlaceholder {
                captured = Some(ParsedExtendTargetFact {
                    kind: ParsedExtendTargetFactKind::Placeholder,
                    name: current.text.trim_start_matches('%').to_string(),
                    optional,
                    range: current.range,
                });
                break;
            }
            if current.kind == SyntaxKind::Dot
                && let Some(name_index) = next_non_trivia_token_index_until(tokens, cursor + 1, end)
                && tokens[name_index].kind == SyntaxKind::Ident
            {
                let name_token = tokens[name_index];
                let range = TextRange::new(current.range.start(), name_token.range.end());
                captured = Some(ParsedExtendTargetFact {
                    kind: ParsedExtendTargetFactKind::Class,
                    name: name_token.text.to_string(),
                    optional,
                    range,
                });
                break;
            }
            cursor += 1;
        }

        if let Some(target) = captured {
            targets.push(target);
        }
    }

    targets
}

/// Detect a trailing `!optional` flag in an `@extend` statement span. The flag tokenizes as a
/// `Delim "!"` followed by an `Ident "optional"` (case-insensitive), matching the `!important`
/// tokenization shape observed for value flags.
fn extend_statement_has_optional_flag(tokens: &[Token<'_>], start: usize, end: usize) -> bool {
    let mut index = start;
    while index < end {
        if tokens[index].kind == SyntaxKind::Delim
            && tokens[index].text == "!"
            && let Some(next_index) = next_non_trivia_token_index_until(tokens, index + 1, end)
            && tokens[next_index].kind == SyntaxKind::Ident
            && tokens[next_index].text.eq_ignore_ascii_case("optional")
        {
            return true;
        }
        index += 1;
    }
    false
}

fn collect_sass_import_module_edges(
    tokens: &[Token<'_>],
    start: usize,
    end: usize,
    edges: &mut Vec<ParsedSassModuleEdgeFact>,
    seen: &mut BTreeSet<(ParsedSassModuleEdgeFactKind, String, u32, u32)>,
) {
    for index in start..end {
        let token = tokens[index];
        if !matches!(token.kind, SyntaxKind::String | SyntaxKind::Url) {
            continue;
        }
        // RFC-0007-D1 (#44): a trailing media qualifier (`@import "foo" screen`,
        // `@import "foo" (min-width: 100px)`) keeps the import as plain CSS. Classify
        // per comma-peer target: the next significant token after this target marks a
        // media qualifier iff it is present and is NOT the `,` comma-peer separator.
        // (`@import "a", "b" screen` qualifies only `"b"`.)
        let media_qualified = next_non_trivia_token_index_until(tokens, index + 1, end)
            .is_some_and(|next| tokens[next].kind != SyntaxKind::Comma);
        push_sass_module_edge_fact(
            edges,
            seen,
            ParsedSassModuleEdgeFact {
                kind: ParsedSassModuleEdgeFactKind::Import,
                source: css_module_value_source_name(token),
                namespace_kind: None,
                namespace: None,
                forward_prefix: None,
                visibility_filter_kind: None,
                visibility_filter_names: Vec::new(),
                media_qualified,
                range: token.range,
            },
        );
    }
}

fn sass_module_use_namespace(
    tokens: &[Token<'_>],
    source: &str,
    start: usize,
    end: usize,
) -> (Option<&'static str>, Option<String>) {
    let Some(as_index) = top_level_token_text_index(tokens, start, end, "as") else {
        return (
            Some("default"),
            sass_module_default_namespace(source).map(str::to_string),
        );
    };
    let Some(namespace_index) = next_non_trivia_token_index_until(tokens, as_index + 1, end) else {
        return (Some("invalid"), None);
    };
    let namespace = tokens[namespace_index];
    match namespace.kind {
        SyntaxKind::Star => (Some("wildcard"), None),
        SyntaxKind::Ident => (Some("alias"), Some(namespace.text.to_string())),
        _ => (Some("invalid"), None),
    }
}

fn sass_module_forward_prefix(tokens: &[Token<'_>], start: usize, end: usize) -> Option<String> {
    let as_index = top_level_token_text_index(tokens, start, end, "as")?;
    let prefix_index = next_non_trivia_token_index_until(tokens, as_index + 1, end)?;
    let prefix = tokens[prefix_index].text.trim();
    if prefix.is_empty() {
        return None;
    }
    Some(prefix.to_string())
}

fn sass_module_forward_visibility_filter(
    tokens: &[Token<'_>],
    start: usize,
    end: usize,
) -> (Option<&'static str>, Vec<String>) {
    let show_index = top_level_token_text_index(tokens, start, end, "show");
    let hide_index = top_level_token_text_index(tokens, start, end, "hide");
    let (filter_kind, filter_index) = match (show_index, hide_index) {
        (Some(show_index), Some(hide_index)) if show_index <= hide_index => ("show", show_index),
        (Some(_), Some(hide_index)) => ("hide", hide_index),
        (Some(show_index), None) => ("show", show_index),
        (None, Some(hide_index)) => ("hide", hide_index),
        (None, None) => return (None, Vec::new()),
    };
    let clause_end =
        top_level_token_text_index(tokens, filter_index + 1, end, "with").unwrap_or(end);
    (
        Some(filter_kind),
        sass_module_visibility_filter_names(tokens, filter_index + 1, clause_end),
    )
}

fn sass_module_visibility_filter_names(
    tokens: &[Token<'_>],
    start: usize,
    end: usize,
) -> Vec<String> {
    let mut names = BTreeSet::new();
    for token in &tokens[start..end] {
        match token.kind {
            SyntaxKind::Ident | SyntaxKind::ScssVariable => {
                if matches_ignore_ascii_case(token.text, &["show", "hide", "with", "as"]) {
                    continue;
                }
                let name = token.text.trim_start_matches('$');
                if !name.is_empty() {
                    names.insert(name.to_string());
                }
            }
            _ => {}
        }
    }
    names.into_iter().collect()
}

fn sass_module_default_namespace(source: &str) -> Option<&str> {
    let basename = source
        .rsplit(['/', '\\', ':'])
        .next()
        .unwrap_or(source)
        .trim_start_matches('_');
    let namespace = basename.split('.').next().unwrap_or(basename);
    (!namespace.is_empty()).then_some(namespace)
}

fn push_sass_module_edge_fact(
    edges: &mut Vec<ParsedSassModuleEdgeFact>,
    seen: &mut BTreeSet<(ParsedSassModuleEdgeFactKind, String, u32, u32)>,
    edge: ParsedSassModuleEdgeFact,
) {
    let start: u32 = edge.range.start().into();
    let end: u32 = edge.range.end().into();
    if seen.insert((edge.kind, edge.source.clone(), start, end)) {
        edges.push(edge);
    }
}

fn collect_css_module_value_facts_from_tokens(
    tokens: &[Token<'_>],
) -> Vec<ParsedCssModuleValueFact> {
    let mut values = Vec::new();
    let mut seen = BTreeSet::new();
    let value_path_aliases = collect_css_module_value_path_aliases_from_tokens(tokens);
    for (index, token) in tokens.iter().enumerate() {
        if token.kind != SyntaxKind::AtKeyword || !token.text.eq_ignore_ascii_case("@value") {
            continue;
        }

        let start = skip_trivia_tokens(tokens, index + 1, tokens.len());
        let end = css_module_value_statement_end(tokens, start);
        let colon_index = top_level_token_kind_index(tokens, start, end, SyntaxKind::Colon);
        let from_index = top_level_token_text_index(tokens, start, end, "from");

        if let Some(from_index) = from_index
            && match colon_index {
                Some(colon_index) => from_index < colon_index,
                None => true,
            }
        {
            collect_css_module_value_import_facts(
                tokens,
                start,
                from_index,
                end,
                &value_path_aliases,
                &mut values,
                &mut seen,
            );
            continue;
        }

        if let Some(colon_index) = colon_index {
            if css_module_value_path_alias_from_tokens(tokens, start, colon_index, end).is_some() {
                continue;
            }
            collect_css_module_value_definition_facts(
                tokens,
                start,
                colon_index,
                &mut values,
                &mut seen,
            );
            collect_css_module_value_reference_facts(
                tokens,
                colon_index + 1,
                end,
                &mut values,
                &mut seen,
            );
        } else {
            collect_css_module_value_definition_facts(tokens, start, end, &mut values, &mut seen);
        }
    }
    let local_value_names = values
        .iter()
        .filter(|value| value.kind == ParsedCssModuleValueFactKind::Definition)
        .map(|value| value.name.clone())
        .collect::<BTreeSet<_>>();
    collect_css_module_value_declaration_reference_facts(
        tokens,
        0,
        tokens.len(),
        &local_value_names,
        &mut values,
        &mut seen,
    );
    values
}

fn collect_css_module_value_path_aliases_from_tokens(
    tokens: &[Token<'_>],
) -> BTreeMap<String, String> {
    let mut aliases = BTreeMap::new();
    for (index, token) in tokens.iter().enumerate() {
        if token.kind != SyntaxKind::AtKeyword || !token.text.eq_ignore_ascii_case("@value") {
            continue;
        }

        let start = skip_trivia_tokens(tokens, index + 1, tokens.len());
        let end = css_module_value_statement_end(tokens, start);
        let Some(colon_index) = top_level_token_kind_index(tokens, start, end, SyntaxKind::Colon)
        else {
            continue;
        };
        if top_level_token_text_index(tokens, start, end, "from").is_some() {
            continue;
        }
        if let Some((name, target)) =
            css_module_value_path_alias_from_tokens(tokens, start, colon_index, end)
        {
            aliases.insert(name, target);
        }
    }
    aliases
}

fn css_module_value_path_alias_from_tokens(
    tokens: &[Token<'_>],
    start: usize,
    colon_index: usize,
    end: usize,
) -> Option<(String, String)> {
    let name_index = next_non_trivia_token_index_until(tokens, start, colon_index)?;
    let name_token = tokens[name_index];
    if !css_module_value_name_token_can_define(name_token) {
        return None;
    }
    let source_index = next_non_trivia_token_index_until(tokens, colon_index + 1, end)?;
    let source_token = tokens[source_index];
    if !matches!(source_token.kind, SyntaxKind::String | SyntaxKind::Url) {
        return None;
    }
    let source = css_module_value_source_name(source_token);
    css_module_value_source_looks_like_style_request(&source)
        .then(|| (name_token.text.to_string(), source))
}

fn css_module_value_statement_end(tokens: &[Token<'_>], start: usize) -> usize {
    let mut index = start;
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;
    while index < tokens.len() {
        match tokens[index].kind {
            SyntaxKind::LeftParen => paren_depth += 1,
            SyntaxKind::RightParen => paren_depth = paren_depth.saturating_sub(1),
            SyntaxKind::LeftBracket => bracket_depth += 1,
            SyntaxKind::RightBracket => bracket_depth = bracket_depth.saturating_sub(1),
            SyntaxKind::Semicolon
            | SyntaxKind::SassOptionalSemicolon
            | SyntaxKind::LeftBrace
            | SyntaxKind::RightBrace
            | SyntaxKind::SassIndent
            | SyntaxKind::SassDedent
                if paren_depth == 0 && bracket_depth == 0 =>
            {
                return index;
            }
            _ => {}
        }
        index += 1;
    }
    index
}

fn collect_css_module_value_import_facts(
    tokens: &[Token<'_>],
    start: usize,
    from_index: usize,
    end: usize,
    value_path_aliases: &BTreeMap<String, String>,
    values: &mut Vec<ParsedCssModuleValueFact>,
    seen: &mut BTreeSet<(ParsedCssModuleValueFactKind, String, u32, u32)>,
) {
    collect_css_module_value_import_names(tokens, start, from_index, values, seen);
    if let Some((source_name, source_range)) =
        css_module_value_import_edge_source(tokens, from_index + 1, end, value_path_aliases)
    {
        push_css_module_value_fact(
            values,
            seen,
            ParsedCssModuleValueFactKind::ImportSource,
            source_name,
            source_range,
        );
    }
}

fn collect_css_module_value_import_edge_facts_from_tokens(
    tokens: &[Token<'_>],
) -> Vec<ParsedCssModuleValueImportEdgeFact> {
    let mut edges = Vec::new();
    let value_path_aliases = collect_css_module_value_path_aliases_from_tokens(tokens);
    for (index, token) in tokens.iter().enumerate() {
        if token.kind != SyntaxKind::AtKeyword || !token.text.eq_ignore_ascii_case("@value") {
            continue;
        }

        let start = skip_trivia_tokens(tokens, index + 1, tokens.len());
        let end = css_module_value_statement_end(tokens, start);
        let colon_index = top_level_token_kind_index(tokens, start, end, SyntaxKind::Colon);
        let from_index = top_level_token_text_index(tokens, start, end, "from");
        let Some(from_index) = from_index else {
            continue;
        };
        if colon_index.is_some_and(|colon_index| from_index > colon_index) {
            continue;
        }
        let Some((import_source, _source_range)) =
            css_module_value_import_edge_source(tokens, from_index + 1, end, &value_path_aliases)
        else {
            continue;
        };

        collect_css_module_value_import_edges(tokens, start, from_index, import_source, &mut edges);
    }
    edges
}

fn collect_css_module_value_definition_edge_facts_from_tokens(
    tokens: &[Token<'_>],
) -> Vec<ParsedCssModuleValueDefinitionEdgeFact> {
    let mut edges = Vec::new();
    for (index, token) in tokens.iter().enumerate() {
        if token.kind != SyntaxKind::AtKeyword || !token.text.eq_ignore_ascii_case("@value") {
            continue;
        }

        let start = skip_trivia_tokens(tokens, index + 1, tokens.len());
        let end = css_module_value_statement_end(tokens, start);
        let colon_index = top_level_token_kind_index(tokens, start, end, SyntaxKind::Colon);
        let from_index = top_level_token_text_index(tokens, start, end, "from");
        let Some(colon_index) = colon_index else {
            continue;
        };
        if from_index.is_some_and(|from_index| from_index < colon_index) {
            continue;
        }

        let definition_names = collect_css_module_value_definition_edge_names(
            tokens,
            start,
            colon_index,
            |tokens, index| css_module_value_name_token_can_define(tokens[index]),
        );
        let reference_names = collect_css_module_value_definition_edge_names(
            tokens,
            colon_index + 1,
            end,
            css_module_value_reference_token_can_be_name,
        );
        if reference_names.is_empty() {
            continue;
        }
        let range_end = end
            .checked_sub(1)
            .and_then(|end| tokens.get(end))
            .map(|token| token.range.end())
            .unwrap_or_else(|| tokens[index].range.end());

        for definition_name in definition_names {
            edges.push(ParsedCssModuleValueDefinitionEdgeFact {
                definition_name,
                reference_names: reference_names.clone(),
                range: TextRange::new(tokens[index].range.start(), range_end),
            });
        }
    }
    edges
}

fn collect_css_module_value_definition_edge_names(
    tokens: &[Token<'_>],
    start: usize,
    end: usize,
    predicate: impl Fn(&[Token<'_>], usize) -> bool,
) -> Vec<String> {
    let mut names = Vec::new();
    let mut index = start;
    while index < end {
        if predicate(tokens, index) && !names.iter().any(|name| name == tokens[index].text) {
            names.push(tokens[index].text.to_string());
        }
        index += 1;
    }
    names
}

fn css_module_value_import_edge_source(
    tokens: &[Token<'_>],
    start: usize,
    end: usize,
    value_path_aliases: &BTreeMap<String, String>,
) -> Option<(String, TextRange)> {
    let source_index = next_non_trivia_token_index_until(tokens, start, end)?;
    let token = tokens[source_index];
    if matches!(token.kind, SyntaxKind::String | SyntaxKind::Url) {
        return Some((css_module_value_source_name(token), token.range));
    }
    if css_module_value_name_token_can_define(token) {
        return css_module_value_source_alias_target(token.text, token.range, value_path_aliases);
    }
    None
}

fn css_module_value_source_alias_target(
    name: &str,
    range: TextRange,
    value_path_aliases: &BTreeMap<String, String>,
) -> Option<(String, TextRange)> {
    value_path_aliases
        .get(name)
        .map(|source| (source.clone(), range))
}

fn collect_css_module_value_import_edges(
    tokens: &[Token<'_>],
    start: usize,
    end: usize,
    import_source: String,
    edges: &mut Vec<ParsedCssModuleValueImportEdgeFact>,
) {
    let mut index = start;
    while index < end {
        let token = tokens[index];
        if !css_module_value_name_token_can_define(token) {
            index += 1;
            continue;
        }
        if previous_non_trivia_token_index(tokens, index, start)
            .is_some_and(|previous| tokens[previous].text == "as")
        {
            index += 1;
            continue;
        }
        let remote_name = token.text.to_string();
        let mut local_name = remote_name.clone();
        let mut local_range = token.range;
        if let Some(as_index) = next_non_trivia_token_index_until(tokens, index + 1, end)
            && tokens[as_index].text == "as"
            && let Some(local_index) = next_non_trivia_token_index_until(tokens, as_index + 1, end)
            && css_module_value_name_token_can_define(tokens[local_index])
        {
            local_name = tokens[local_index].text.to_string();
            local_range = tokens[local_index].range;
            index = local_index + 1;
        } else {
            index += 1;
        }
        edges.push(ParsedCssModuleValueImportEdgeFact {
            remote_name,
            local_name,
            import_source: import_source.clone(),
            local_range,
            remote_range: token.range,
            range: token.range,
        });
    }
}

fn collect_css_module_value_import_names(
    tokens: &[Token<'_>],
    start: usize,
    end: usize,
    values: &mut Vec<ParsedCssModuleValueFact>,
    seen: &mut BTreeSet<(ParsedCssModuleValueFactKind, String, u32, u32)>,
) {
    let mut index = start;
    while index < end {
        let token = tokens[index];
        if css_module_value_name_token_can_define(token) {
            let previous = previous_non_trivia_token_index(tokens, index, start);
            let next = next_non_trivia_token_index_until(tokens, index + 1, end);
            let kind = if previous.is_some_and(|previous| tokens[previous].text == "as") {
                Some(ParsedCssModuleValueFactKind::Definition)
            } else if next.is_some_and(|next| tokens[next].text == "as") {
                Some(ParsedCssModuleValueFactKind::Reference)
            } else {
                Some(ParsedCssModuleValueFactKind::Definition)
            };
            if let Some(kind) = kind {
                push_css_module_value_fact(values, seen, kind, token.text.to_string(), token.range);
            }
        }
        index += 1;
    }
}

fn collect_css_module_value_definition_facts(
    tokens: &[Token<'_>],
    start: usize,
    end: usize,
    values: &mut Vec<ParsedCssModuleValueFact>,
    seen: &mut BTreeSet<(ParsedCssModuleValueFactKind, String, u32, u32)>,
) {
    let mut index = start;
    while index < end {
        let token = tokens[index];
        if css_module_value_name_token_can_define(token) {
            push_css_module_value_fact(
                values,
                seen,
                ParsedCssModuleValueFactKind::Definition,
                token.text.to_string(),
                token.range,
            );
        }
        index += 1;
    }
}

fn collect_css_module_value_reference_facts(
    tokens: &[Token<'_>],
    start: usize,
    end: usize,
    values: &mut Vec<ParsedCssModuleValueFact>,
    seen: &mut BTreeSet<(ParsedCssModuleValueFactKind, String, u32, u32)>,
) {
    let mut index = start;
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;
    while index < end {
        match tokens[index].kind {
            SyntaxKind::LeftParen => paren_depth += 1,
            SyntaxKind::RightParen => paren_depth = paren_depth.saturating_sub(1),
            SyntaxKind::LeftBracket => bracket_depth += 1,
            SyntaxKind::RightBracket => bracket_depth = bracket_depth.saturating_sub(1),
            _ => {}
        }
        if paren_depth == 0
            && bracket_depth == 0
            && css_module_value_reference_token_can_be_name(tokens, index)
        {
            push_css_module_value_fact(
                values,
                seen,
                ParsedCssModuleValueFactKind::Reference,
                tokens[index].text.to_string(),
                tokens[index].range,
            );
        }
        index += 1;
    }
}

fn collect_css_module_value_declaration_reference_facts(
    tokens: &[Token<'_>],
    start: usize,
    end: usize,
    local_value_names: &BTreeSet<String>,
    values: &mut Vec<ParsedCssModuleValueFact>,
    seen: &mut BTreeSet<(ParsedCssModuleValueFactKind, String, u32, u32)>,
) {
    if local_value_names.is_empty() {
        return;
    }

    let mut index = start;
    while index < end {
        index = skip_trivia_tokens(tokens, index, end);
        if index >= end {
            break;
        }

        if tokens[index].kind == SyntaxKind::AtKeyword {
            let block = find_block_after_header(tokens, index, end);
            if let Some((open, close)) = block {
                if style_wrapper_at_rule(tokens[index].text) {
                    collect_css_module_value_declaration_reference_facts(
                        tokens,
                        open + 1,
                        close,
                        local_value_names,
                        values,
                        seen,
                    );
                }
                index = close + 1;
            } else {
                index = skip_statement(tokens, index, end);
            }
            continue;
        }

        let statement_end = css_module_value_statement_end(tokens, index);
        if statement_end < end && tokens[statement_end].kind == SyntaxKind::LeftBrace {
            if let Some(close) = matching_right_brace(tokens, statement_end, end) {
                collect_css_module_value_declaration_reference_facts(
                    tokens,
                    statement_end + 1,
                    close,
                    local_value_names,
                    values,
                    seen,
                );
                index = close + 1;
            } else {
                index = statement_end + 1;
            }
            continue;
        }

        if let Some(colon_index) = declaration_colon_index(tokens, index, statement_end.min(end)) {
            collect_known_css_module_value_reference_facts(
                tokens,
                colon_index + 1,
                statement_end.min(end),
                local_value_names,
                values,
                seen,
            );
        }

        if statement_end >= end || tokens[statement_end].kind == SyntaxKind::RightBrace {
            break;
        }
        index = statement_end + 1;
    }
}

fn declaration_colon_index(tokens: &[Token<'_>], start: usize, end: usize) -> Option<usize> {
    let colon_index = top_level_token_kind_index(tokens, start, end, SyntaxKind::Colon)?;
    let property_index = previous_non_trivia_token_index(tokens, colon_index, start)?;
    if !matches!(
        tokens[property_index].kind,
        SyntaxKind::Ident
            | SyntaxKind::CustomPropertyName
            | SyntaxKind::ScssVariable
            | SyntaxKind::LessVariable
            | SyntaxKind::LessPropertyVariableToken
    ) {
        return None;
    }
    let value_index = next_non_trivia_token_index_until(tokens, colon_index + 1, end)?;
    if matches!(
        tokens[value_index].kind,
        SyntaxKind::LeftBrace | SyntaxKind::LeftParen | SyntaxKind::LeftBracket
    ) {
        return None;
    }
    Some(colon_index)
}

fn collect_known_css_module_value_reference_facts(
    tokens: &[Token<'_>],
    start: usize,
    end: usize,
    local_value_names: &BTreeSet<String>,
    values: &mut Vec<ParsedCssModuleValueFact>,
    seen: &mut BTreeSet<(ParsedCssModuleValueFactKind, String, u32, u32)>,
) {
    let mut index = start;
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;
    while index < end {
        match tokens[index].kind {
            SyntaxKind::LeftParen => paren_depth += 1,
            SyntaxKind::RightParen => paren_depth = paren_depth.saturating_sub(1),
            SyntaxKind::LeftBracket => bracket_depth += 1,
            SyntaxKind::RightBracket => bracket_depth = bracket_depth.saturating_sub(1),
            _ => {}
        }
        if paren_depth == 0
            && bracket_depth == 0
            && css_module_value_reference_token_can_be_name(tokens, index)
            && local_value_names.contains(tokens[index].text)
        {
            push_css_module_value_fact(
                values,
                seen,
                ParsedCssModuleValueFactKind::Reference,
                tokens[index].text.to_string(),
                tokens[index].range,
            );
        }
        index += 1;
    }
}

fn push_css_module_value_fact(
    values: &mut Vec<ParsedCssModuleValueFact>,
    seen: &mut BTreeSet<(ParsedCssModuleValueFactKind, String, u32, u32)>,
    kind: ParsedCssModuleValueFactKind,
    name: String,
    range: TextRange,
) {
    if seen.insert((
        kind,
        name.clone(),
        u32::from(range.start()),
        u32::from(range.end()),
    )) {
        values.push(ParsedCssModuleValueFact { kind, name, range });
    }
}

fn top_level_token_kind_index(
    tokens: &[Token<'_>],
    start: usize,
    end: usize,
    expected: SyntaxKind,
) -> Option<usize> {
    let mut index = start;
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;
    while index < end {
        match tokens[index].kind {
            SyntaxKind::LeftParen => paren_depth += 1,
            SyntaxKind::RightParen => paren_depth = paren_depth.saturating_sub(1),
            SyntaxKind::LeftBracket => bracket_depth += 1,
            SyntaxKind::RightBracket => bracket_depth = bracket_depth.saturating_sub(1),
            kind if kind == expected && paren_depth == 0 && bracket_depth == 0 => {
                return Some(index);
            }
            _ => {}
        }
        index += 1;
    }
    None
}

fn top_level_token_text_index(
    tokens: &[Token<'_>],
    start: usize,
    end: usize,
    expected: &str,
) -> Option<usize> {
    let mut index = start;
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;
    while index < end {
        match tokens[index].kind {
            SyntaxKind::LeftParen => paren_depth += 1,
            SyntaxKind::RightParen => paren_depth = paren_depth.saturating_sub(1),
            SyntaxKind::LeftBracket => bracket_depth += 1,
            SyntaxKind::RightBracket => bracket_depth = bracket_depth.saturating_sub(1),
            SyntaxKind::Ident
                if paren_depth == 0
                    && bracket_depth == 0
                    && tokens[index].text.eq_ignore_ascii_case(expected) =>
            {
                return Some(index);
            }
            _ => {}
        }
        index += 1;
    }
    None
}

fn previous_non_trivia_token_index(
    tokens: &[Token<'_>],
    mut index: usize,
    start: usize,
) -> Option<usize> {
    while index > start {
        index -= 1;
        if !tokens[index].kind.is_trivia() {
            return Some(index);
        }
    }
    None
}

fn css_module_value_name_token_can_define(token: Token<'_>) -> bool {
    matches!(
        token.kind,
        SyntaxKind::Ident | SyntaxKind::CustomPropertyName
    ) && !matches!(token.text, "as" | "from")
}

fn css_module_value_reference_token_can_be_name(tokens: &[Token<'_>], index: usize) -> bool {
    let token = tokens[index];
    if !matches!(
        token.kind,
        SyntaxKind::Ident | SyntaxKind::CustomPropertyName
    ) {
        return false;
    }
    if let Some(next_index) = next_non_trivia_token_index_until(tokens, index + 1, tokens.len())
        && tokens[next_index].kind == SyntaxKind::LeftParen
    {
        return false;
    }
    !css_module_value_literal_ident_is_not_reference(token.text)
}

fn css_module_value_literal_ident_is_not_reference(name: &str) -> bool {
    matches!(
        name.to_ascii_lowercase().as_str(),
        "initial"
            | "inherit"
            | "unset"
            | "revert"
            | "revert-layer"
            | "none"
            | "auto"
            | "normal"
            | "transparent"
            | "currentcolor"
            | "black"
            | "white"
            | "red"
            | "green"
            | "blue"
            | "yellow"
            | "magenta"
            | "cyan"
            | "solid"
            | "dashed"
            | "block"
            | "inline"
            | "flex"
            | "grid"
    )
}

fn css_module_value_source_name(token: Token<'_>) -> String {
    token
        .text
        .trim_matches(|character| character == '"' || character == '\'')
        .to_string()
}

fn css_module_value_source_looks_like_style_request(source: &str) -> bool {
    let lower = source.to_ascii_lowercase();
    (lower.starts_with('/') || lower.starts_with("./") || lower.starts_with("../"))
        && (lower.ends_with(".css")
            || lower.ends_with(".scss")
            || lower.ends_with(".sass")
            || lower.ends_with(".less"))
}

fn collect_css_module_composes_facts_from_tokens(
    tokens: &[Token<'_>],
) -> Vec<ParsedCssModuleComposesFact> {
    let mut composes = Vec::new();
    let mut seen = BTreeSet::new();
    for (index, token) in tokens.iter().enumerate() {
        if token.kind != SyntaxKind::Ident || !token.text.eq_ignore_ascii_case("composes") {
            continue;
        }
        let Some(colon_index) = next_non_trivia_token_index_until(tokens, index + 1, tokens.len())
        else {
            continue;
        };
        if tokens[colon_index].kind != SyntaxKind::Colon {
            continue;
        }

        let start = colon_index + 1;
        let end = css_module_value_statement_end(tokens, start);
        let from_index = top_level_token_text_index(tokens, start, end, "from");
        let target_end = from_index.unwrap_or(end);
        collect_css_module_composes_targets(tokens, start, target_end, &mut composes, &mut seen);
        if let Some(from_index) = from_index {
            collect_css_module_composes_import_source(
                tokens,
                from_index + 1,
                end,
                &mut composes,
                &mut seen,
            );
        }
    }
    composes
}

fn collect_css_module_composes_edge_facts_from_tokens(
    tokens: &[Token<'_>],
) -> Vec<ParsedCssModuleComposesEdgeFact> {
    let mut edges = Vec::new();
    collect_css_module_composes_edge_facts_in_range(tokens, 0, tokens.len(), &[], None, &mut edges);
    edges
}

fn collect_css_module_composes_edge_facts_in_range(
    tokens: &[Token<'_>],
    start: usize,
    end: usize,
    parent_branches: &[SelectorBranch],
    css_module_scope: Option<&'static str>,
    edges: &mut Vec<ParsedCssModuleComposesEdgeFact>,
) {
    let mut index = start;
    while index < end {
        index = skip_trivia_tokens(tokens, index, end);
        if index >= end {
            break;
        }

        if tokens[index].kind == SyntaxKind::AtKeyword {
            let block = find_block_after_header(tokens, index, end);
            if let Some((open, close)) = block {
                if tokens[index].text == "@nest" {
                    if css_module_scope == Some("global") {
                        collect_css_module_composes_edge_facts_in_range(
                            tokens,
                            open + 1,
                            close,
                            &[],
                            css_module_scope,
                            edges,
                        );
                    } else {
                        let branches =
                            resolve_selector_header(tokens, index + 1, open, parent_branches);
                        collect_immediate_css_module_composes_edge_facts(
                            tokens,
                            open + 1,
                            close,
                            &branches,
                            edges,
                        );
                        collect_css_module_composes_edge_facts_in_range(
                            tokens,
                            open + 1,
                            close,
                            &branches,
                            css_module_scope,
                            edges,
                        );
                    }
                } else if style_wrapper_at_rule(tokens[index].text) {
                    collect_css_module_composes_edge_facts_in_range(
                        tokens,
                        open + 1,
                        close,
                        parent_branches,
                        css_module_scope,
                        edges,
                    );
                }
                index = close + 1;
            } else {
                index = skip_statement(tokens, index, end);
            }
            continue;
        }

        let Some((open, close)) = find_block_after_header(tokens, index, end) else {
            index = skip_statement(tokens, index, end);
            continue;
        };

        let effective_scope = css_module_scope
            .or_else(|| css_module_block_scope_marker_in_header(tokens, index, open));
        if effective_scope == Some("global") {
            collect_css_module_composes_edge_facts_in_range(
                tokens,
                open + 1,
                close,
                &[],
                effective_scope,
                edges,
            );
        } else {
            let branches = resolve_selector_header(tokens, index, open, parent_branches);
            collect_immediate_css_module_composes_edge_facts(
                tokens,
                open + 1,
                close,
                &branches,
                edges,
            );
            collect_css_module_composes_edge_facts_in_range(
                tokens,
                open + 1,
                close,
                &branches,
                effective_scope,
                edges,
            );
        }
        index = close + 1;
    }
}

fn collect_immediate_css_module_composes_edge_facts(
    tokens: &[Token<'_>],
    start: usize,
    end: usize,
    owner_branches: &[SelectorBranch],
    edges: &mut Vec<ParsedCssModuleComposesEdgeFact>,
) {
    let owner_selector_names = sorted_selector_branch_names(owner_branches);
    let mut index = start;
    let mut block_depth = 0usize;
    while index < end {
        match tokens[index].kind {
            SyntaxKind::LeftBrace | SyntaxKind::SassIndent => {
                block_depth += 1;
                index += 1;
                continue;
            }
            SyntaxKind::RightBrace | SyntaxKind::SassDedent => {
                block_depth = block_depth.saturating_sub(1);
                index += 1;
                continue;
            }
            _ => {}
        }
        if block_depth > 0
            || tokens[index].kind != SyntaxKind::Ident
            || !tokens[index].text.eq_ignore_ascii_case("composes")
        {
            index += 1;
            continue;
        }
        let Some(colon_index) = next_non_trivia_token_index_until(tokens, index + 1, end) else {
            index += 1;
            continue;
        };
        if tokens[colon_index].kind != SyntaxKind::Colon {
            index += 1;
            continue;
        }

        let value_start = colon_index + 1;
        let value_end = css_module_value_statement_end(tokens, value_start).min(end);
        let from_index = top_level_token_text_index(tokens, value_start, value_end, "from");
        let target_end = from_index.unwrap_or(value_end);
        let target_names =
            collect_css_module_composes_target_names(tokens, value_start, target_end);
        if target_names.is_empty() {
            index = value_end;
            continue;
        }

        let (kind, import_source) = from_index
            .and_then(|from_index| {
                css_module_composes_import_edge_source(tokens, from_index + 1, value_end)
            })
            .map(|source| {
                if source == "global" {
                    (ParsedCssModuleComposesEdgeKind::Global, Some(source))
                } else {
                    (ParsedCssModuleComposesEdgeKind::External, Some(source))
                }
            })
            .unwrap_or((ParsedCssModuleComposesEdgeKind::Local, None));
        let range_end = value_end
            .checked_sub(1)
            .and_then(|end| tokens.get(end))
            .map(|token| token.range.end())
            .unwrap_or_else(|| tokens[index].range.end());

        edges.push(ParsedCssModuleComposesEdgeFact {
            kind,
            owner_selector_names: owner_selector_names.clone(),
            target_names,
            import_source,
            range: TextRange::new(tokens[index].range.start(), range_end),
        });
        index = value_end;
    }
}

fn sorted_selector_branch_names(branches: &[SelectorBranch]) -> Vec<String> {
    branches
        .iter()
        .map(|branch| branch.name.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn collect_css_module_composes_target_names(
    tokens: &[Token<'_>],
    start: usize,
    end: usize,
) -> Vec<String> {
    let mut names = Vec::new();
    let mut index = start;
    while index < end {
        if matches!(
            tokens[index].kind,
            SyntaxKind::Ident | SyntaxKind::CustomPropertyName
        ) && !tokens[index].text.eq_ignore_ascii_case("from")
            && !names.iter().any(|name| name == tokens[index].text)
        {
            names.push(tokens[index].text.to_string());
        }
        index += 1;
    }
    names
}

fn css_module_composes_import_edge_source(
    tokens: &[Token<'_>],
    start: usize,
    end: usize,
) -> Option<String> {
    let source_index = next_non_trivia_token_index_until(tokens, start, end)?;
    let token = tokens[source_index];
    matches!(
        token.kind,
        SyntaxKind::String | SyntaxKind::Url | SyntaxKind::Ident
    )
    .then(|| css_module_value_source_name(token))
}

fn collect_css_module_composes_targets(
    tokens: &[Token<'_>],
    start: usize,
    end: usize,
    composes: &mut Vec<ParsedCssModuleComposesFact>,
    seen: &mut BTreeSet<(ParsedCssModuleComposesFactKind, String, u32, u32)>,
) {
    let mut index = start;
    while index < end {
        if matches!(
            tokens[index].kind,
            SyntaxKind::Ident | SyntaxKind::CustomPropertyName
        ) && !tokens[index].text.eq_ignore_ascii_case("from")
        {
            push_css_module_composes_fact(
                composes,
                seen,
                ParsedCssModuleComposesFactKind::Target,
                tokens[index].text.to_string(),
                tokens[index].range,
            );
        }
        index += 1;
    }
}

fn collect_css_module_composes_import_source(
    tokens: &[Token<'_>],
    start: usize,
    end: usize,
    composes: &mut Vec<ParsedCssModuleComposesFact>,
    seen: &mut BTreeSet<(ParsedCssModuleComposesFactKind, String, u32, u32)>,
) {
    if let Some(source_index) = next_non_trivia_token_index_until(tokens, start, end) {
        let token = tokens[source_index];
        if matches!(
            token.kind,
            SyntaxKind::String | SyntaxKind::Url | SyntaxKind::Ident
        ) {
            push_css_module_composes_fact(
                composes,
                seen,
                ParsedCssModuleComposesFactKind::ImportSource,
                css_module_value_source_name(token),
                token.range,
            );
        }
    }
}

fn push_css_module_composes_fact(
    composes: &mut Vec<ParsedCssModuleComposesFact>,
    seen: &mut BTreeSet<(ParsedCssModuleComposesFactKind, String, u32, u32)>,
    kind: ParsedCssModuleComposesFactKind,
    name: String,
    range: TextRange,
) {
    if seen.insert((
        kind,
        name.clone(),
        u32::from(range.start()),
        u32::from(range.end()),
    )) {
        composes.push(ParsedCssModuleComposesFact { kind, name, range });
    }
}

fn collect_icss_facts_from_tokens(tokens: &[Token<'_>]) -> Vec<ParsedIcssFact> {
    let mut icss = Vec::new();
    let mut seen = BTreeSet::new();
    for (index, token) in tokens.iter().enumerate() {
        if token.kind != SyntaxKind::Colon {
            continue;
        }
        let Some(name_index) = next_non_trivia_token_index_until(tokens, index + 1, tokens.len())
        else {
            continue;
        };
        let name = tokens[name_index].text;
        if !matches!(tokens[name_index].kind, SyntaxKind::Ident) {
            continue;
        }
        if name.eq_ignore_ascii_case("export") {
            if let Some((open, close)) =
                find_block_after_header(tokens, name_index + 1, tokens.len())
            {
                collect_icss_export_names(tokens, open + 1, close, &mut icss, &mut seen);
            }
            continue;
        }
        if name.eq_ignore_ascii_case("import") {
            collect_icss_import_source(tokens, name_index + 1, &mut icss, &mut seen);
            if let Some((open, close)) =
                find_block_after_header(tokens, name_index + 1, tokens.len())
            {
                collect_icss_import_names(tokens, open + 1, close, &mut icss, &mut seen);
            }
        }
    }
    icss
}

fn collect_icss_import_edge_facts_from_tokens(
    tokens: &[Token<'_>],
) -> Vec<ParsedIcssImportEdgeFact> {
    let mut edges = Vec::new();
    for (index, token) in tokens.iter().enumerate() {
        if token.kind != SyntaxKind::Colon {
            continue;
        }
        let Some(name_index) = next_non_trivia_token_index_until(tokens, index + 1, tokens.len())
        else {
            continue;
        };
        if tokens[name_index].kind != SyntaxKind::Ident
            || !tokens[name_index].text.eq_ignore_ascii_case("import")
        {
            continue;
        }
        let Some(import_source) = icss_import_edge_source(tokens, name_index + 1) else {
            continue;
        };
        if let Some((open, close)) = find_block_after_header(tokens, name_index + 1, tokens.len()) {
            collect_icss_import_edges(tokens, open + 1, close, import_source, &mut edges);
        }
    }
    edges
}

fn collect_icss_export_edge_facts_from_tokens(
    tokens: &[Token<'_>],
) -> Vec<ParsedIcssExportEdgeFact> {
    let mut edges = Vec::new();
    for (index, token) in tokens.iter().enumerate() {
        if token.kind != SyntaxKind::Colon {
            continue;
        }
        let Some(name_index) = next_non_trivia_token_index_until(tokens, index + 1, tokens.len())
        else {
            continue;
        };
        if tokens[name_index].kind != SyntaxKind::Ident
            || !tokens[name_index].text.eq_ignore_ascii_case("export")
        {
            continue;
        }
        if let Some((open, close)) = find_block_after_header(tokens, name_index + 1, tokens.len()) {
            collect_icss_export_edges(tokens, open + 1, close, &mut edges);
        }
    }
    edges
}

fn collect_icss_export_edges(
    tokens: &[Token<'_>],
    start: usize,
    end: usize,
    edges: &mut Vec<ParsedIcssExportEdgeFact>,
) {
    let mut index = start;
    while index < end {
        let token = tokens[index];
        if matches!(
            token.kind,
            SyntaxKind::Ident | SyntaxKind::CustomPropertyName
        ) && let Some(colon_index) = next_non_trivia_token_index_until(tokens, index + 1, end)
            && tokens[colon_index].kind == SyntaxKind::Colon
        {
            let value_end = css_module_value_statement_end(tokens, colon_index + 1).min(end);
            let reference_names = collect_css_module_value_definition_edge_names(
                tokens,
                colon_index + 1,
                value_end,
                css_module_value_reference_token_can_be_name,
            );
            if !reference_names.is_empty() {
                let range_end = value_end
                    .checked_sub(1)
                    .and_then(|end| tokens.get(end))
                    .map(|token| token.range.end())
                    .unwrap_or_else(|| token.range.end());
                edges.push(ParsedIcssExportEdgeFact {
                    export_name: token.text.to_string(),
                    reference_names,
                    range: TextRange::new(token.range.start(), range_end),
                });
            }
            index = value_end;
            continue;
        }
        index += 1;
    }
}

fn icss_import_edge_source(tokens: &[Token<'_>], start: usize) -> Option<String> {
    let open_index = next_non_trivia_token_index_until(tokens, start, tokens.len())?;
    if tokens[open_index].kind != SyntaxKind::LeftParen {
        return None;
    }
    let source_index = next_non_trivia_token_index_until(tokens, open_index + 1, tokens.len())?;
    let token = tokens[source_index];
    matches!(
        token.kind,
        SyntaxKind::String | SyntaxKind::Url | SyntaxKind::Ident
    )
    .then(|| css_module_value_source_name(token))
}

fn collect_icss_import_edges(
    tokens: &[Token<'_>],
    start: usize,
    end: usize,
    import_source: String,
    edges: &mut Vec<ParsedIcssImportEdgeFact>,
) {
    let mut index = start;
    while index < end {
        let token = tokens[index];
        if matches!(
            token.kind,
            SyntaxKind::Ident | SyntaxKind::CustomPropertyName
        ) && let Some(colon_index) = next_non_trivia_token_index_until(tokens, index + 1, end)
            && tokens[colon_index].kind == SyntaxKind::Colon
            && let Some(remote_index) =
                next_non_trivia_token_index_until(tokens, colon_index + 1, end)
            && matches!(
                tokens[remote_index].kind,
                SyntaxKind::Ident | SyntaxKind::CustomPropertyName
            )
        {
            edges.push(ParsedIcssImportEdgeFact {
                local_name: token.text.to_string(),
                remote_name: tokens[remote_index].text.to_string(),
                import_source: import_source.clone(),
                range: token.range,
            });
            index = css_module_value_statement_end(tokens, colon_index + 1);
            continue;
        }
        index += 1;
    }
}

fn collect_icss_export_names(
    tokens: &[Token<'_>],
    start: usize,
    end: usize,
    icss: &mut Vec<ParsedIcssFact>,
    seen: &mut BTreeSet<(ParsedIcssFactKind, String, u32, u32)>,
) {
    let mut index = start;
    while index < end {
        let token = tokens[index];
        if matches!(
            token.kind,
            SyntaxKind::Ident | SyntaxKind::CustomPropertyName
        ) && let Some(colon_index) = next_non_trivia_token_index_until(tokens, index + 1, end)
            && tokens[colon_index].kind == SyntaxKind::Colon
        {
            push_icss_fact(
                icss,
                seen,
                ParsedIcssFactKind::ExportName,
                token.text.to_string(),
                token.range,
            );
            index = css_module_value_statement_end(tokens, colon_index + 1);
            continue;
        }
        index += 1;
    }
}

fn collect_icss_import_source(
    tokens: &[Token<'_>],
    start: usize,
    icss: &mut Vec<ParsedIcssFact>,
    seen: &mut BTreeSet<(ParsedIcssFactKind, String, u32, u32)>,
) {
    let Some(open_index) = next_non_trivia_token_index_until(tokens, start, tokens.len()) else {
        return;
    };
    if tokens[open_index].kind != SyntaxKind::LeftParen {
        return;
    }
    let Some(source_index) =
        next_non_trivia_token_index_until(tokens, open_index + 1, tokens.len())
    else {
        return;
    };
    let token = tokens[source_index];
    if matches!(
        token.kind,
        SyntaxKind::String | SyntaxKind::Url | SyntaxKind::Ident
    ) {
        push_icss_fact(
            icss,
            seen,
            ParsedIcssFactKind::ImportSource,
            css_module_value_source_name(token),
            token.range,
        );
    }
}

fn collect_icss_import_names(
    tokens: &[Token<'_>],
    start: usize,
    end: usize,
    icss: &mut Vec<ParsedIcssFact>,
    seen: &mut BTreeSet<(ParsedIcssFactKind, String, u32, u32)>,
) {
    let mut index = start;
    while index < end {
        let token = tokens[index];
        if matches!(
            token.kind,
            SyntaxKind::Ident | SyntaxKind::CustomPropertyName
        ) && let Some(colon_index) = next_non_trivia_token_index_until(tokens, index + 1, end)
            && tokens[colon_index].kind == SyntaxKind::Colon
        {
            push_icss_fact(
                icss,
                seen,
                ParsedIcssFactKind::ImportLocalName,
                token.text.to_string(),
                token.range,
            );
            if let Some(remote_index) =
                next_non_trivia_token_index_until(tokens, colon_index + 1, end)
                && matches!(
                    tokens[remote_index].kind,
                    SyntaxKind::Ident | SyntaxKind::CustomPropertyName
                )
            {
                push_icss_fact(
                    icss,
                    seen,
                    ParsedIcssFactKind::ImportRemoteName,
                    tokens[remote_index].text.to_string(),
                    tokens[remote_index].range,
                );
            }
            index = css_module_value_statement_end(tokens, colon_index + 1);
            continue;
        }
        index += 1;
    }
}

fn push_icss_fact(
    icss: &mut Vec<ParsedIcssFact>,
    seen: &mut BTreeSet<(ParsedIcssFactKind, String, u32, u32)>,
    kind: ParsedIcssFactKind,
    name: String,
    range: TextRange,
) {
    if seen.insert((
        kind,
        name.clone(),
        u32::from(range.start()),
        u32::from(range.end()),
    )) {
        icss.push(ParsedIcssFact { kind, name, range });
    }
}

fn collect_animation_facts_from_tokens(tokens: &[Token<'_>]) -> Vec<ParsedAnimationFact> {
    let mut animations = Vec::new();
    let mut seen = BTreeSet::new();
    for (index, token) in tokens.iter().enumerate() {
        if token.kind == SyntaxKind::AtKeyword && at_keyword_is_keyframes_rule(token.text) {
            if let Some(name_index) =
                next_non_trivia_token_index_until(tokens, index + 1, tokens.len())
                && let Some(name) = animation_name_from_token(tokens[name_index])
            {
                push_animation_fact(
                    &mut animations,
                    &mut seen,
                    ParsedAnimationFactKind::KeyframesDeclaration,
                    name,
                    tokens[name_index].range,
                );
            }
            continue;
        }

        if token.kind == SyntaxKind::Ident
            && token.text.eq_ignore_ascii_case("animation-name")
            && let Some(colon_index) =
                next_non_trivia_token_index_until(tokens, index + 1, tokens.len())
            && tokens[colon_index].kind == SyntaxKind::Colon
        {
            collect_animation_name_references_until(
                tokens,
                colon_index + 1,
                &mut animations,
                &mut seen,
            );
        }

        if token.kind == SyntaxKind::Ident
            && token.text.eq_ignore_ascii_case("animation")
            && let Some(colon_index) =
                next_non_trivia_token_index_until(tokens, index + 1, tokens.len())
            && tokens[colon_index].kind == SyntaxKind::Colon
        {
            collect_animation_shorthand_references_until(
                tokens,
                colon_index + 1,
                &mut animations,
                &mut seen,
            );
        }
    }
    animations
}

fn collect_animation_name_references_until(
    tokens: &[Token<'_>],
    start: usize,
    animations: &mut Vec<ParsedAnimationFact>,
    seen: &mut BTreeSet<(ParsedAnimationFactKind, String, u32, u32)>,
) {
    let mut index = start;
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;
    while index < tokens.len() {
        match tokens[index].kind {
            SyntaxKind::LeftParen => paren_depth += 1,
            SyntaxKind::RightParen => paren_depth = paren_depth.saturating_sub(1),
            SyntaxKind::LeftBracket => bracket_depth += 1,
            SyntaxKind::RightBracket => bracket_depth = bracket_depth.saturating_sub(1),
            SyntaxKind::Semicolon
            | SyntaxKind::SassOptionalSemicolon
            | SyntaxKind::RightBrace
            | SyntaxKind::SassDedent
                if paren_depth == 0 && bracket_depth == 0 =>
            {
                break;
            }
            _ => {}
        }

        if paren_depth == 0
            && bracket_depth == 0
            && !animation_name_token_is_interpolation_adjacent(tokens, index)
            && let Some(name) = animation_name_from_token(tokens[index])
        {
            push_animation_fact(
                animations,
                seen,
                ParsedAnimationFactKind::AnimationNameReference,
                name,
                tokens[index].range,
            );
        }
        index += 1;
    }
}

fn collect_animation_shorthand_references_until(
    tokens: &[Token<'_>],
    start: usize,
    animations: &mut Vec<ParsedAnimationFact>,
    seen: &mut BTreeSet<(ParsedAnimationFactKind, String, u32, u32)>,
) {
    let mut index = start;
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;
    while index < tokens.len() {
        match tokens[index].kind {
            SyntaxKind::LeftParen => paren_depth += 1,
            SyntaxKind::RightParen => paren_depth = paren_depth.saturating_sub(1),
            SyntaxKind::LeftBracket => bracket_depth += 1,
            SyntaxKind::RightBracket => bracket_depth = bracket_depth.saturating_sub(1),
            SyntaxKind::Semicolon
            | SyntaxKind::SassOptionalSemicolon
            | SyntaxKind::RightBrace
            | SyntaxKind::SassDedent
                if paren_depth == 0 && bracket_depth == 0 =>
            {
                break;
            }
            _ => {}
        }

        if paren_depth == 0
            && bracket_depth == 0
            && animation_shorthand_token_can_be_name(tokens, index)
            && let Some(name) = animation_name_from_token(tokens[index])
        {
            push_animation_fact(
                animations,
                seen,
                ParsedAnimationFactKind::AnimationNameReference,
                name,
                tokens[index].range,
            );
        }
        index += 1;
    }
}

fn animation_shorthand_token_can_be_name(tokens: &[Token<'_>], index: usize) -> bool {
    let token = tokens[index];
    if token.kind == SyntaxKind::String {
        return true;
    }
    if token.kind != SyntaxKind::Ident {
        return false;
    }
    // A literal fragment that is *immediately* adjacent to an interpolation boundary is part
    // of a statically-unknown name (`#{$dur}s` unit suffix, `#{$p}-spin` / `spin-#{$p}`
    // interpolated keyframes name), not a standalone animation name. Reject it so neither the
    // unit nor the literal fragment is misread as a missing `@keyframes` reference.
    if animation_name_token_is_interpolation_adjacent(tokens, index) {
        return false;
    }
    // Standalone CSS time-unit idents (`s` / `ms`) are durations, never animation names.
    if animation_shorthand_ident_is_time_unit(token.text) {
        return false;
    }
    if let Some(next_index) = next_non_trivia_token_index_until(tokens, index + 1, tokens.len())
        && tokens[next_index].kind == SyntaxKind::LeftParen
    {
        return false;
    }
    !animation_shorthand_ident_is_non_name(token.text)
}

fn animation_shorthand_ident_is_time_unit(name: &str) -> bool {
    name.eq_ignore_ascii_case("s") || name.eq_ignore_ascii_case("ms")
}

/// An ident is part of an interpolated (statically-unknown) animation name when it is
/// *immediately* adjacent to an interpolation boundary — `#{$p}-spin` (post-interpolation
/// literal fragment) or `spin-#{$p}` (pre-interpolation literal fragment). The post-`#{...}`
/// text is the trailing fragment of a dynamic name, not a real keyframes reference, so it
/// must not be flagged as `missingKeyframes`.
///
/// Adjacency is checked against the *immediate* neighbor token (no trivia skipping): a
/// fully-static name separated from an interpolation by whitespace (`#{$p} spin`, a real
/// space-delimited keyframes reference) is NOT suppressed.
fn animation_name_token_is_interpolation_adjacent(tokens: &[Token<'_>], index: usize) -> bool {
    if index > 0
        && matches!(
            tokens[index - 1].kind,
            SyntaxKind::ScssInterpolationEnd | SyntaxKind::LessInterpolationEnd
        )
    {
        return true;
    }
    if let Some(next) = tokens.get(index + 1)
        && matches!(
            next.kind,
            SyntaxKind::ScssInterpolationStart | SyntaxKind::LessInterpolationStart
        )
    {
        return true;
    }
    false
}

fn animation_shorthand_ident_is_non_name(name: &str) -> bool {
    matches!(
        name.to_ascii_lowercase().as_str(),
        "ease"
            | "ease-in"
            | "ease-out"
            | "ease-in-out"
            | "linear"
            | "step-start"
            | "step-end"
            | "infinite"
            | "normal"
            | "reverse"
            | "alternate"
            | "alternate-reverse"
            | "running"
            | "paused"
            | "forwards"
            | "backwards"
            | "both"
            | "replace"
            | "add"
            | "accumulate"
            | "auto"
    )
}

fn push_animation_fact(
    animations: &mut Vec<ParsedAnimationFact>,
    seen: &mut BTreeSet<(ParsedAnimationFactKind, String, u32, u32)>,
    kind: ParsedAnimationFactKind,
    name: String,
    range: TextRange,
) {
    if seen.insert((
        kind,
        name.clone(),
        u32::from(range.start()),
        u32::from(range.end()),
    )) {
        animations.push(ParsedAnimationFact { kind, name, range });
    }
}

fn animation_name_from_token(token: Token<'_>) -> Option<String> {
    if !matches!(token.kind, SyntaxKind::Ident | SyntaxKind::String) {
        return None;
    }
    let name = token
        .text
        .trim_matches(|character| character == '"' || character == '\'')
        .to_string();
    if name.is_empty() || animation_name_is_reserved(&name) {
        return None;
    }
    Some(name)
}

fn animation_name_is_reserved(name: &str) -> bool {
    matches!(
        name.to_ascii_lowercase().as_str(),
        "none" | "initial" | "inherit" | "unset" | "revert" | "revert-layer"
    )
}

/// Recognize an `@keyframes` at-rule prefix-insensitively.
///
/// Per CSS, `animation-name` resolves against any `@keyframes`/`@-webkit-keyframes`
/// (and other vendor prefixes) with a matching name, so a vendor-prefixed at-rule
/// must register the same bare keyframes-name fact as the unprefixed form. Strips a
/// leading `@`, then an optional `-vendor-` prefix, and compares the remainder to
/// `keyframes`.
fn at_keyword_is_keyframes_rule(text: &str) -> bool {
    let Some(rule) = text.strip_prefix('@') else {
        return false;
    };
    if rule.eq_ignore_ascii_case("keyframes") {
        return true;
    }
    // Accept a single `-vendor-` prefix (`-webkit-`, `-moz-`, `-o-`, `-ms-`, ...).
    if let Some(rest) = rule.strip_prefix('-')
        && let Some((vendor, remainder)) = rest.split_once('-')
        && !vendor.is_empty()
        && remainder.eq_ignore_ascii_case("keyframes")
    {
        return true;
    }
    false
}

fn containing_at_rule_header_name<'text>(
    tokens: &'text [Token<'text>],
    index: usize,
) -> Option<&'text str> {
    let mut current = index;
    while current > 0 {
        current -= 1;
        let token = tokens.get(current)?;
        if token.kind.is_trivia() {
            continue;
        }
        if matches!(
            token.kind,
            SyntaxKind::Semicolon
                | SyntaxKind::SassOptionalSemicolon
                | SyntaxKind::LeftBrace
                | SyntaxKind::RightBrace
                | SyntaxKind::SassIndent
                | SyntaxKind::SassDedent
        ) {
            return None;
        }
        if token.kind == SyntaxKind::AtKeyword {
            return Some(token.text);
        }
    }
    None
}

fn skip_trivia_tokens(tokens: &[Token<'_>], mut index: usize, end: usize) -> usize {
    while index < end && tokens[index].kind.is_trivia() {
        index += 1;
    }
    index
}

fn skip_statement(tokens: &[Token<'_>], mut index: usize, end: usize) -> usize {
    while index < end {
        match tokens[index].kind {
            SyntaxKind::Semicolon | SyntaxKind::SassOptionalSemicolon => return index + 1,
            SyntaxKind::RightBrace | SyntaxKind::SassDedent => return index,
            _ => index += 1,
        }
    }
    index
}

fn find_block_after_header(
    tokens: &[Token<'_>],
    start: usize,
    end: usize,
) -> Option<(usize, usize)> {
    let mut index = start;
    while index < end {
        match tokens[index].kind {
            SyntaxKind::Semicolon
            | SyntaxKind::SassOptionalSemicolon
            | SyntaxKind::RightBrace
            | SyntaxKind::SassDedent => return None,
            SyntaxKind::LeftBrace => {
                let close = matching_right_brace(tokens, index, end)?;
                return Some((index, close));
            }
            SyntaxKind::SassIndent => {
                let close = matching_sass_dedent(tokens, index, end)?;
                return Some((index, close));
            }
            _ => index += 1,
        }
    }
    None
}

fn matching_right_brace(tokens: &[Token<'_>], open: usize, end: usize) -> Option<usize> {
    let mut depth = 0usize;
    let mut index = open;
    while index < end {
        match tokens[index].kind {
            SyntaxKind::LeftBrace => depth += 1,
            SyntaxKind::RightBrace => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return Some(index);
                }
            }
            _ => {}
        }
        index += 1;
    }
    None
}

fn matching_sass_dedent(tokens: &[Token<'_>], open: usize, end: usize) -> Option<usize> {
    let mut depth = 0usize;
    let mut index = open;
    while index < end {
        match tokens[index].kind {
            SyntaxKind::SassIndent => depth += 1,
            SyntaxKind::SassDedent => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return Some(index);
                }
            }
            _ => {}
        }
        index += 1;
    }
    None
}

fn style_wrapper_at_rule(name: &str) -> bool {
    matches_ignore_ascii_case(
        name,
        &[
            "@media",
            "@supports",
            "@when",
            "@else",
            "@layer",
            "@scope",
            "@container",
            "@starting-style",
            "@if",
            "@else",
            "@for",
            "@each",
            "@while",
            "@at-root",
            "@include",
        ],
    )
}

fn is_selector_combinator_kind(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::GreaterThan
            | SyntaxKind::Plus
            | SyntaxKind::Tilde
            | SyntaxKind::ColumnCombinator
            | SyntaxKind::DoublePipe
    )
}

fn selector_component_can_start(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::Dot
            | SyntaxKind::Hash
            | SyntaxKind::Ident
            | SyntaxKind::Star
            | SyntaxKind::Ampersand
            | SyntaxKind::ScssPlaceholder
            | SyntaxKind::LeftBracket
            | SyntaxKind::Colon
            | SyntaxKind::DoubleColon
    )
}

fn namespace_selector_target_can_start(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::Ident | SyntaxKind::CustomPropertyName | SyntaxKind::Star
    )
}

fn keyframe_selector_token_is_valid(token: Token<'_>) -> bool {
    token.kind == SyntaxKind::Percentage
        || (token.kind == SyntaxKind::Ident
            && (token.text.eq_ignore_ascii_case("from") || token.text.eq_ignore_ascii_case("to")))
}

fn selector_component_can_end(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::Ident
            | SyntaxKind::CustomPropertyName
            | SyntaxKind::Hash
            | SyntaxKind::RightBracket
            | SyntaxKind::RightParen
            | SyntaxKind::Star
    )
}

fn collect_at_rule_facts_from_tokens(
    tokens: &[Token<'_>],
    dialect: StyleDialect,
) -> Vec<ParsedAtRuleFact> {
    tokens
        .iter()
        .filter(|token| token.kind == SyntaxKind::AtKeyword)
        .map(|token| {
            let css_spec = at_rule_spec(token.text);
            let node_kind = css_spec
                .or_else(|| match dialect {
                    StyleDialect::Scss | StyleDialect::Sass => scss_at_rule_spec(token.text),
                    StyleDialect::Css | StyleDialect::Less => None,
                })
                .map(|spec| spec.node_kind);
            let name = if css_spec.is_some() {
                token.text.to_ascii_lowercase()
            } else {
                token.text.to_string()
            };
            ParsedAtRuleFact {
                name,
                node_kind,
                range: token.range,
            }
        })
        .collect()
}

fn next_non_trivia_token<'text>(
    tokens: &'text [Token<'text>],
    mut index: usize,
) -> Option<Token<'text>> {
    while let Some(token) = tokens.get(index).copied() {
        if !token.kind.is_trivia() {
            return Some(token);
        }
        index += 1;
    }
    None
}

fn next_non_trivia_token_until<'text>(
    tokens: &'text [Token<'text>],
    mut index: usize,
    end: usize,
) -> Option<Token<'text>> {
    while index < end {
        let token = tokens.get(index).copied()?;
        if !token.kind.is_trivia() {
            return Some(token);
        }
        index += 1;
    }
    None
}

fn next_non_trivia_token_index_until(
    tokens: &[Token<'_>],
    mut index: usize,
    end: usize,
) -> Option<usize> {
    while index < end {
        let token = tokens.get(index)?;
        if !token.kind.is_trivia() {
            return Some(index);
        }
        index += 1;
    }
    None
}

fn next_non_trivia_token_after_range<'text>(
    tokens: &'text [Token<'text>],
    range: TextRange,
    end: usize,
) -> Option<Token<'text>> {
    let index = token_index_by_range(tokens, range)?;
    next_non_trivia_token_until(tokens, index + 1, end)
}

fn token_index_by_range(tokens: &[Token<'_>], range: TextRange) -> Option<usize> {
    tokens.iter().position(|token| token.range == range)
}

fn matching_right_paren_from_range(
    tokens: &[Token<'_>],
    open_range: TextRange,
    end: usize,
) -> Option<usize> {
    let mut depth = 0usize;
    let mut index = token_index_by_range(tokens, open_range)?;
    while index < end {
        match tokens[index].kind {
            SyntaxKind::LeftParen => depth += 1,
            SyntaxKind::RightParen => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return Some(index);
                }
            }
            _ => {}
        }
        index += 1;
    }
    None
}

fn previous_non_trivia_token<'text>(
    tokens: &'text [Token<'text>],
    start: usize,
    index: usize,
) -> Option<Token<'text>> {
    let mut current = index;
    while current > start {
        current -= 1;
        let token = tokens.get(current).copied()?;
        if !token.kind.is_trivia() {
            return Some(token);
        }
    }
    None
}

fn is_selector_boundary(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::Comma
            | SyntaxKind::LeftBrace
            | SyntaxKind::SassIndent
            | SyntaxKind::RightBrace
            | SyntaxKind::SassDedent
            | SyntaxKind::Semicolon
            | SyntaxKind::SassOptionalSemicolon
    )
}

fn is_selector_boundary_until(kind: SyntaxKind, recovery: &[SyntaxKind]) -> bool {
    is_selector_boundary(kind) || recovery.contains(&kind)
}

fn is_selector_list_pseudo_class(text: &str) -> bool {
    matches!(text, "is" | "where" | "local" | "global")
}

fn is_nth_pseudo_class(text: &str) -> bool {
    matches!(
        text,
        "nth-child" | "nth-last-child" | "nth-of-type" | "nth-last-of-type"
    )
}

fn language_tag_token_can_start(kind: SyntaxKind) -> bool {
    matches!(kind, SyntaxKind::Ident | SyntaxKind::String)
}

fn selector_item_token_is_recoverable(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::Whitespace
            | SyntaxKind::SassIndentedNewline
            | SyntaxKind::Dot
            | SyntaxKind::Comma
            | SyntaxKind::Hash
            | SyntaxKind::Ident
            | SyntaxKind::CustomPropertyName
            | SyntaxKind::String
            | SyntaxKind::Number
            | SyntaxKind::Percentage
            | SyntaxKind::Dimension
            | SyntaxKind::Star
            | SyntaxKind::Ampersand
            | SyntaxKind::ScssPlaceholder
            | SyntaxKind::LeftBracket
            | SyntaxKind::RightBracket
            | SyntaxKind::Colon
            | SyntaxKind::DoubleColon
            | SyntaxKind::LeftParen
            | SyntaxKind::RightParen
            | SyntaxKind::Equals
            | SyntaxKind::IncludesMatch
            | SyntaxKind::DashMatch
            | SyntaxKind::PrefixMatch
            | SyntaxKind::SuffixMatch
            | SyntaxKind::SubstringMatch
            | SyntaxKind::Pipe
            | SyntaxKind::ColumnCombinator
            | SyntaxKind::GreaterThan
            | SyntaxKind::Plus
            | SyntaxKind::Minus
            | SyntaxKind::Tilde
            | SyntaxKind::KeywordAnd
            | SyntaxKind::KeywordOr
            | SyntaxKind::KeywordNot
    )
}

fn is_at_rule_prelude_boundary(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::LeftBrace
            | SyntaxKind::SassIndent
            | SyntaxKind::Semicolon
            | SyntaxKind::SassOptionalSemicolon
    )
}

fn is_statement_end(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::Semicolon | SyntaxKind::SassOptionalSemicolon
    )
}

fn sass_token_can_end_statement(kind: SyntaxKind) -> bool {
    !matches!(
        kind,
        SyntaxKind::Whitespace
            | SyntaxKind::LineComment
            | SyntaxKind::BlockComment
            | SyntaxKind::SassIndentedNewline
            | SyntaxKind::SassIndent
            | SyntaxKind::SassDedent
            | SyntaxKind::SassOptionalSemicolon
            | SyntaxKind::Comma
            | SyntaxKind::Colon
            | SyntaxKind::DoubleColon
            | SyntaxKind::LeftBrace
            | SyntaxKind::LeftParen
            | SyntaxKind::LeftBracket
            | SyntaxKind::Plus
            | SyntaxKind::Minus
            | SyntaxKind::Star
            | SyntaxKind::Slash
            | SyntaxKind::GreaterThan
            | SyntaxKind::LessThan
            | SyntaxKind::Equals
            | SyntaxKind::Arrow
            | SyntaxKind::Pipe
            | SyntaxKind::Tilde
            | SyntaxKind::Caret
            | SyntaxKind::Ampersand
            | SyntaxKind::DoubleAmpersand
            | SyntaxKind::ColumnCombinator
            | SyntaxKind::IncludesMatch
            | SyntaxKind::DashMatch
            | SyntaxKind::PrefixMatch
            | SyntaxKind::SuffixMatch
            | SyntaxKind::SubstringMatch
            | SyntaxKind::PlusEquals
            | SyntaxKind::MinusEquals
            | SyntaxKind::SlashEquals
    )
}

fn function_argument_recovery(recovery: &[SyntaxKind]) -> Vec<SyntaxKind> {
    let mut kinds = vec![SyntaxKind::RightParen];
    for kind in recovery {
        if !kinds.contains(kind) {
            kinds.push(*kind);
        }
    }
    kinds
}

fn bracketed_value_recovery(recovery: &[SyntaxKind]) -> Vec<SyntaxKind> {
    let mut kinds = vec![SyntaxKind::RightBracket];
    for kind in recovery {
        if !kinds.contains(kind) {
            kinds.push(*kind);
        }
    }
    kinds
}

fn simple_block_recovery(close_kind: SyntaxKind, recovery: &[SyntaxKind]) -> Vec<SyntaxKind> {
    let mut kinds = vec![close_kind];
    for kind in recovery {
        if !kinds.contains(kind) {
            kinds.push(*kind);
        }
    }
    kinds
}

fn matching_simple_block_close(open_kind: SyntaxKind) -> Option<SyntaxKind> {
    match open_kind {
        SyntaxKind::LeftBrace => Some(SyntaxKind::RightBrace),
        SyntaxKind::LeftBracket => Some(SyntaxKind::RightBracket),
        SyntaxKind::LeftParen => Some(SyntaxKind::RightParen),
        _ => None,
    }
}

fn value_list_item_recovery(recovery: &[SyntaxKind]) -> Vec<SyntaxKind> {
    let mut kinds = vec![SyntaxKind::Comma];
    for kind in recovery {
        if !kinds.contains(kind) {
            kinds.push(*kind);
        }
    }
    kinds
}

fn comma_separated_component_value_list_item_recovery(recovery: &[SyntaxKind]) -> Vec<SyntaxKind> {
    let mut kinds = vec![SyntaxKind::Comma];
    for kind in recovery {
        if !kinds.contains(kind) {
            kinds.push(*kind);
        }
    }
    kinds
}

fn variable_declaration_node_kind(kind: SyntaxKind, has_colon: bool) -> SyntaxKind {
    if has_colon {
        return kind;
    }
    match kind {
        SyntaxKind::ScssVariableDeclaration => SyntaxKind::BogusScssVariable,
        SyntaxKind::LessVariableDeclaration => SyntaxKind::BogusLessVariable,
        _ => kind,
    }
}

fn is_attribute_matcher(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::Equals
            | SyntaxKind::IncludesMatch
            | SyntaxKind::DashMatch
            | SyntaxKind::PrefixMatch
            | SyntaxKind::SuffixMatch
            | SyntaxKind::SubstringMatch
    )
}

fn attribute_name_token_can_start(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::Ident | SyntaxKind::CustomPropertyName | SyntaxKind::Star
    )
}

fn attribute_name_token_can_continue(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::Ident
            | SyntaxKind::CustomPropertyName
            | SyntaxKind::Star
            | SyntaxKind::Pipe
            | SyntaxKind::ColumnCombinator
    )
}

fn attribute_value_token_can_start(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::Ident
            | SyntaxKind::CustomPropertyName
            | SyntaxKind::String
            | SyntaxKind::Hash
            | SyntaxKind::Number
            | SyntaxKind::Dimension
    )
}

fn is_combinator(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::GreaterThan
            | SyntaxKind::Plus
            | SyntaxKind::Tilde
            | SyntaxKind::ColumnCombinator
    )
}

fn infix_binding_power(kind: SyntaxKind) -> Option<(u8, u8)> {
    match kind {
        SyntaxKind::Plus | SyntaxKind::Minus => Some((1, 2)),
        SyntaxKind::Star | SyntaxKind::Slash | SyntaxKind::Percent => Some((3, 4)),
        _ => None,
    }
}

fn specialized_function_kind(text: &str) -> Option<SyntaxKind> {
    if text.eq_ignore_ascii_case("var") {
        return Some(SyntaxKind::VarFunction);
    }
    if text.eq_ignore_ascii_case("calc") {
        return Some(SyntaxKind::CalcFunction);
    }
    if text.eq_ignore_ascii_case("env") {
        return Some(SyntaxKind::EnvFunction);
    }
    if text.eq_ignore_ascii_case("attr") {
        return Some(SyntaxKind::AttrFunction);
    }
    if matches_ignore_ascii_case(text, VALUES_L4_MATH_FUNCTION_NAMES) {
        return Some(SyntaxKind::MathFunction);
    }
    if matches_ignore_ascii_case(text, CSS_COLOR_FUNCTION_NAMES) {
        return Some(SyntaxKind::ColorValue);
    }
    if matches_ignore_ascii_case(text, CSS_GRADIENT_FUNCTION_NAMES) {
        return Some(SyntaxKind::GradientFunction);
    }
    if matches_ignore_ascii_case(text, CSS_TRANSFORM_FUNCTION_NAMES) {
        return Some(SyntaxKind::TransformFunction);
    }
    if matches_ignore_ascii_case(text, CSS_FILTER_FUNCTION_NAMES) {
        return Some(SyntaxKind::FilterFunction);
    }
    if matches_ignore_ascii_case(text, CSS_IMAGE_FUNCTION_NAMES) {
        return Some(SyntaxKind::ImageFunction);
    }
    if matches_ignore_ascii_case(text, CSS_SHAPE_FUNCTION_NAMES) {
        return Some(SyntaxKind::ShapeFunction);
    }
    None
}

fn function_argument_count_is_valid(function_name: &str, argument_count: usize) -> bool {
    if function_name.eq_ignore_ascii_case("calc") {
        return argument_count == 1;
    }
    if matches_ignore_ascii_case(function_name, &["min", "max", "hypot"]) {
        return argument_count >= 1;
    }
    if function_name.eq_ignore_ascii_case("clamp") {
        return argument_count == 3;
    }
    if function_name.eq_ignore_ascii_case("round") {
        return (2..=3).contains(&argument_count);
    }
    if function_name.eq_ignore_ascii_case("log") {
        return (1..=2).contains(&argument_count);
    }
    if matches_ignore_ascii_case(function_name, &["mod", "rem", "pow", "atan2"]) {
        return argument_count == 2;
    }
    if matches_ignore_ascii_case(
        function_name,
        &[
            "sin", "cos", "tan", "asin", "acos", "atan", "sqrt", "exp", "abs", "sign",
        ],
    ) {
        return argument_count == 1;
    }
    if function_name.eq_ignore_ascii_case("color-mix") {
        return argument_count == 3;
    }
    if function_name.eq_ignore_ascii_case("light-dark") {
        return argument_count == 2;
    }
    if function_name.eq_ignore_ascii_case("contrast-color") {
        return argument_count == 1;
    }
    true
}

fn function_requires_filled_top_level_arguments(function_name: &str) -> bool {
    function_name.eq_ignore_ascii_case("calc")
        || matches_ignore_ascii_case(function_name, VALUES_L4_MATH_FUNCTION_NAMES)
        || matches_ignore_ascii_case(
            function_name,
            &["color-mix", "light-dark", "contrast-color"],
        )
}

fn at_rule_prelude_head_is_custom_property_name(kind: SyntaxKind) -> bool {
    kind == SyntaxKind::CustomPropertyName || is_interpolation_start(kind)
}

fn at_rule_prelude_head_is_custom_ident(kind: SyntaxKind) -> bool {
    kind == SyntaxKind::Ident || is_interpolation_start(kind)
}

fn is_dynamic_function_argument_head(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::ScssVariable
            | SyntaxKind::LessVariable
            | SyntaxKind::ScssInterpolationStart
            | SyntaxKind::LessInterpolationStart
    )
}

fn is_scss_module_source_token(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::String | SyntaxKind::Url | SyntaxKind::ScssInterpolationStart
    )
}

fn is_scss_module_namespace_token(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::Ident | SyntaxKind::Star | SyntaxKind::ScssInterpolationStart
    )
}

fn is_scss_module_visibility_name_token(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::Ident
            | SyntaxKind::ScssVariable
            | SyntaxKind::ScssPlaceholder
            | SyntaxKind::ScssInterpolationStart
    )
}

fn is_css_module_from_source_token(kind: SyntaxKind, text: &str) -> bool {
    matches!(
        kind,
        SyntaxKind::String
            | SyntaxKind::Url
            | SyntaxKind::ScssInterpolationStart
            | SyntaxKind::LessInterpolationStart
    ) || (kind == SyntaxKind::Ident && text == "global")
}

fn is_scss_control_rule_kind(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::ScssControlIf
            | SyntaxKind::ScssControlElse
            | SyntaxKind::ScssControlEach
            | SyntaxKind::ScssControlFor
            | SyntaxKind::ScssControlWhile
    )
}

fn matches_ignore_ascii_case(value: &str, candidates: &[&str]) -> bool {
    candidates
        .iter()
        .any(|candidate| value.eq_ignore_ascii_case(candidate))
}

fn css_module_scope_function_kind(text: &str) -> Option<SyntaxKind> {
    match text {
        "local" => Some(SyntaxKind::CssModuleLocalBlock),
        "global" => Some(SyntaxKind::CssModuleGlobalBlock),
        _ => None,
    }
}

fn text_range(start: usize, end: usize) -> TextRange {
    TextRange::new(TextSize::from(start as u32), TextSize::from(end as u32))
}

#[cfg(test)]
mod tests;
