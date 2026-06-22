//! Aggregated parser fact surface.
//!
//! This module re-exports the syntax-derived fact records that are safe for
//! query, bridge, LSP, and transform consumers to share.

mod animations;
mod at_rules;
mod css_modules;
mod icss;
mod sass;
mod selectors;
mod variables;

use cstree::syntax::SyntaxNode;
use omena_syntax::{StyleDialect, SyntaxKind};

use crate::{DialectExtension, ParseResult, Parser, Token, tokenize};

pub(crate) use animations::collect_animation_facts_from_cst;
#[cfg(feature = "internal-oracle")]
pub(crate) use animations::collect_animation_facts_from_tokens;
pub use animations::{ParsedAnimationFact, ParsedAnimationFactKind};
pub use at_rules::ParsedAtRuleFact;
pub(crate) use at_rules::collect_at_rule_facts_from_cst;
#[cfg(feature = "internal-oracle")]
pub(crate) use at_rules::collect_at_rule_facts_from_tokens;
pub use css_modules::{
    ParsedCssModuleComposesEdgeFact, ParsedCssModuleComposesEdgeKind, ParsedCssModuleComposesFact,
    ParsedCssModuleComposesFactKind, ParsedCssModuleValueDefinitionEdgeFact,
    ParsedCssModuleValueFact, ParsedCssModuleValueFactKind, ParsedCssModuleValueImportEdgeFact,
};
pub(crate) use css_modules::{
    collect_css_module_composes_edge_facts_from_cst, collect_css_module_composes_facts_from_cst,
    collect_css_module_value_definition_edge_facts_from_cst,
    collect_css_module_value_definition_edge_names, collect_css_module_value_facts_from_cst,
    collect_css_module_value_import_edge_facts_from_cst,
    css_module_value_reference_token_can_be_name, css_module_value_source_name,
    css_module_value_statement_end, declaration_colon_index,
};
#[cfg(feature = "internal-oracle")]
pub(crate) use css_modules::{
    collect_css_module_composes_edge_facts_from_tokens,
    collect_css_module_composes_facts_from_tokens,
    collect_css_module_value_definition_edge_facts_from_tokens,
    collect_css_module_value_facts_from_tokens,
    collect_css_module_value_import_edge_facts_from_tokens,
};
pub use icss::{
    ParsedIcssExportEdgeFact, ParsedIcssFact, ParsedIcssFactKind, ParsedIcssImportEdgeFact,
};
pub(crate) use icss::{
    collect_icss_export_edge_facts_from_cst, collect_icss_facts_from_cst,
    collect_icss_import_edge_facts_from_cst,
};
#[cfg(feature = "internal-oracle")]
pub(crate) use icss::{
    collect_icss_export_edge_facts_from_tokens, collect_icss_facts_from_tokens,
    collect_icss_import_edge_facts_from_tokens,
};
pub use sass::{
    ParsedExtendTargetFact, ParsedExtendTargetFactKind, ParsedSassIncludeFact,
    ParsedSassModuleEdgeFact, ParsedSassModuleEdgeFactKind, ParsedSassSymbolFact,
    ParsedSassSymbolFactKind,
};
pub(crate) use sass::{
    collect_extend_target_facts_from_cst, collect_sass_include_facts_from_cst,
    collect_sass_module_edge_facts_from_cst, collect_sass_symbol_facts_from_cst,
};
#[cfg(feature = "internal-oracle")]
pub(crate) use sass::{
    collect_extend_target_facts_from_tokens, collect_sass_include_facts_from_tokens,
    collect_sass_module_edge_facts_from_tokens, collect_sass_symbol_facts_from_tokens,
};
#[cfg(feature = "internal-oracle")]
pub(crate) use selectors::collect_selector_facts_from_tokens;
pub use selectors::{ParsedSelectorFact, ParsedSelectorFactKind};
pub(crate) use selectors::{
    SelectorBranch, collect_class_selector_names_from_header, collect_selector_facts_from_cst,
    css_module_block_scope_marker_in_header, css_module_header_is_global_only,
    resolve_selector_header, split_selector_groups,
};
#[cfg(feature = "internal-oracle")]
pub(crate) use variables::collect_variable_facts_from_tokens;
pub use variables::{ParsedVariableFact, ParsedVariableFactKind};
pub(crate) use variables::{collect_variable_facts_from_cst, scss_variable_token_is_declaration};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedStyleFacts {
    pub product: &'static str,
    pub dialect: StyleDialect,
    pub selector_count: usize,
    pub selectors: Vec<ParsedSelectorFact>,
    pub variable_count: usize,
    pub variables: Vec<ParsedVariableFact>,
    pub sass_symbol_count: usize,
    pub sass_symbols: Vec<ParsedSassSymbolFact>,
    pub sass_include_count: usize,
    pub sass_includes: Vec<ParsedSassIncludeFact>,
    pub sass_module_edge_count: usize,
    pub sass_module_edges: Vec<ParsedSassModuleEdgeFact>,
    pub extend_target_count: usize,
    pub extend_targets: Vec<ParsedExtendTargetFact>,
    pub animation_count: usize,
    pub animations: Vec<ParsedAnimationFact>,
    pub css_module_value_count: usize,
    pub css_module_values: Vec<ParsedCssModuleValueFact>,
    pub css_module_value_import_edge_count: usize,
    pub css_module_value_import_edges: Vec<ParsedCssModuleValueImportEdgeFact>,
    pub css_module_value_definition_edge_count: usize,
    pub css_module_value_definition_edges: Vec<ParsedCssModuleValueDefinitionEdgeFact>,
    pub css_module_composes_count: usize,
    pub css_module_composes: Vec<ParsedCssModuleComposesFact>,
    pub css_module_composes_edge_count: usize,
    pub css_module_composes_edges: Vec<ParsedCssModuleComposesEdgeFact>,
    pub icss_count: usize,
    pub icss: Vec<ParsedIcssFact>,
    pub icss_import_edge_count: usize,
    pub icss_import_edges: Vec<ParsedIcssImportEdgeFact>,
    pub icss_export_edge_count: usize,
    pub icss_export_edges: Vec<ParsedIcssExportEdgeFact>,
    pub at_rule_count: usize,
    pub at_rules: Vec<ParsedAtRuleFact>,
    pub error_count: usize,
}

pub fn collect_style_facts_with_extension(
    text: &str,
    extension: &impl DialectExtension,
) -> ParsedStyleFacts {
    let (tokens, lex_errors) = tokenize(text, extension);
    let token_count = tokens.len();
    let mut parser = Parser::new(tokens.clone(), lex_errors, extension.dialect());
    crate::record_omena_parser_parse_materialization(token_count);
    let (green, interner) = parser.parse();
    let errors = parser.into_errors();
    let parsed = ParseResult::new(green, interner, errors, token_count, extension.dialect());
    facts_from_cst(text, &parsed)
}

#[cfg(feature = "internal-oracle")]
pub fn collect_style_facts_with_extension_from_legacy_tokens(
    text: &str,
    extension: &impl DialectExtension,
) -> ParsedStyleFacts {
    let (tokens, lex_errors) = tokenize(text, extension);
    let mut parser = Parser::new(tokens.clone(), lex_errors, extension.dialect());
    crate::record_omena_parser_parse_materialization(tokens.len());
    let _ = parser.parse();
    let errors = parser.into_errors();
    style_facts_from_tokens(&tokens, extension.dialect(), errors.len())
}

#[cfg(feature = "internal-oracle")]
fn style_facts_from_tokens(
    tokens: &[Token<'_>],
    dialect: StyleDialect,
    error_count: usize,
) -> ParsedStyleFacts {
    let selectors = collect_selector_facts_from_tokens(tokens);
    let variables = collect_variable_facts_from_tokens(tokens);
    let sass_symbols = collect_sass_symbol_facts_from_tokens(tokens);
    let sass_includes = collect_sass_include_facts_from_tokens(tokens);
    let sass_module_edges = collect_sass_module_edge_facts_from_tokens(tokens);
    let extend_targets = collect_extend_target_facts_from_tokens(tokens);
    let animations = collect_animation_facts_from_tokens(tokens);
    let css_module_values = collect_css_module_value_facts_from_tokens(tokens);
    let css_module_value_import_edges =
        collect_css_module_value_import_edge_facts_from_tokens(tokens);
    let css_module_value_definition_edges =
        collect_css_module_value_definition_edge_facts_from_tokens(tokens);
    let css_module_composes = collect_css_module_composes_facts_from_tokens(tokens);
    let css_module_composes_edges = collect_css_module_composes_edge_facts_from_tokens(tokens);
    let icss = collect_icss_facts_from_tokens(tokens);
    let icss_import_edges = collect_icss_import_edge_facts_from_tokens(tokens);
    let icss_export_edges = collect_icss_export_edge_facts_from_tokens(tokens);
    let at_rules = collect_at_rule_facts_from_tokens(tokens, dialect);

    ParsedStyleFacts {
        product: "omena-parser.style-facts",
        dialect,
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
        error_count,
    }
}

pub fn facts_from_cst(text: &str, parsed: &ParseResult) -> ParsedStyleFacts {
    let selectors = collect_selector_facts_from_cst(text, parsed);
    let variables = collect_variable_facts_from_cst(text, parsed);
    let sass_symbols = collect_sass_symbol_facts_from_cst(text, parsed);
    let sass_includes = collect_sass_include_facts_from_cst(text, parsed);
    let sass_module_edges = collect_sass_module_edge_facts_from_cst(text, parsed);
    let extend_targets = collect_extend_target_facts_from_cst(text, parsed);
    let animations = collect_animation_facts_from_cst(text, parsed);
    let css_module_values = collect_css_module_value_facts_from_cst(text, parsed);
    let css_module_value_import_edges =
        collect_css_module_value_import_edge_facts_from_cst(text, parsed);
    let css_module_value_definition_edges =
        collect_css_module_value_definition_edge_facts_from_cst(text, parsed);
    let css_module_composes = collect_css_module_composes_facts_from_cst(text, parsed);
    let css_module_composes_edges = collect_css_module_composes_edge_facts_from_cst(text, parsed);
    let icss = collect_icss_facts_from_cst(text, parsed);
    let icss_import_edges = collect_icss_import_edge_facts_from_cst(text, parsed);
    let icss_export_edges = collect_icss_export_edge_facts_from_cst(text, parsed);
    let at_rules = collect_at_rule_facts_from_cst(text, parsed);

    ParsedStyleFacts {
        product: "omena-parser.style-facts",
        dialect: parsed.dialect(),
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
        error_count: parsed.errors().len(),
    }
}

pub(crate) fn tokens_from_syntax_node<'text>(
    text: &'text str,
    node: &SyntaxNode<SyntaxKind>,
) -> Vec<Token<'text>> {
    node.descendants_with_tokens()
        .filter_map(|element| element.into_token())
        .map(|token| {
            let range = token.text_range();
            let start = u32::from(range.start()) as usize;
            let end = u32::from(range.end()) as usize;
            Token {
                kind: token.kind(),
                text: text.get(start..end).unwrap_or_default(),
                range,
            }
        })
        .collect()
}
