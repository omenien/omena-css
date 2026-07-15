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
pub use animations::{ParsedAnimationFact, ParsedAnimationFactKind};
pub use at_rules::ParsedAtRuleFact;
pub(crate) use at_rules::collect_at_rule_facts_from_cst;
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
pub use icss::{
    ParsedIcssExportEdgeFact, ParsedIcssFact, ParsedIcssFactKind, ParsedIcssImportEdgeFact,
    collect_icss_export_values_from_cst,
};
pub(crate) use icss::{
    collect_icss_export_edge_facts_from_cst, collect_icss_facts_from_cst,
    collect_icss_import_edge_facts_from_cst,
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
pub use selectors::{ParsedSelectorFact, ParsedSelectorFactKind};
pub(crate) use selectors::{
    SelectorBranch, collect_class_selector_names_from_header, collect_selector_facts_from_cst,
    css_module_block_scope_marker_in_header, css_module_header_is_global_only,
    resolve_selector_header, split_selector_groups,
};
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

pub(crate) fn product_facts_from_cst(text: &str, parsed: &ParseResult) -> ParsedStyleFacts {
    let has_sass_syntax = matches!(parsed.dialect(), StyleDialect::Scss | StyleDialect::Sass);

    let selectors = collect_selector_facts_from_cst(text, parsed);
    let variables = collect_variable_facts_from_cst(text, parsed);
    let sass_symbols = if has_sass_syntax {
        collect_sass_symbol_facts_from_cst(text, parsed)
    } else {
        Vec::new()
    };
    let sass_module_edges = if has_sass_syntax {
        collect_sass_module_edge_facts_from_cst(text, parsed)
    } else {
        Vec::new()
    };
    let animations = collect_animation_facts_from_cst(text, parsed);
    let css_module_values = collect_css_module_value_facts_from_cst(text, parsed);
    let css_module_value_import_edges =
        collect_css_module_value_import_edge_facts_from_cst(text, parsed);
    let css_module_value_definition_edges =
        collect_css_module_value_definition_edge_facts_from_cst(text, parsed);
    let css_module_composes = collect_css_module_composes_facts_from_cst(text, parsed);
    let css_module_composes_edges = collect_css_module_composes_edge_facts_from_cst(text, parsed);

    ParsedStyleFacts {
        product: "omena-parser.style-facts",
        dialect: parsed.dialect(),
        selector_count: selectors.len(),
        selectors,
        variable_count: variables.len(),
        variables,
        sass_symbol_count: sass_symbols.len(),
        sass_symbols,
        sass_include_count: 0,
        sass_includes: Vec::new(),
        sass_module_edge_count: sass_module_edges.len(),
        sass_module_edges,
        extend_target_count: 0,
        extend_targets: Vec::new(),
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
        icss_count: 0,
        icss: Vec::new(),
        icss_import_edge_count: 0,
        icss_import_edges: Vec::new(),
        icss_export_edge_count: 0,
        icss_export_edges: Vec::new(),
        at_rule_count: 0,
        at_rules: Vec::new(),
        error_count: parsed.errors().len(),
    }
}

#[cfg(test)]
mod product_facts_authority_tests;

pub(crate) fn tokens_from_syntax_node<'text>(
    text: &'text str,
    parsed: &ParseResult,
    node: &SyntaxNode<SyntaxKind>,
) -> Vec<Token<'text>> {
    let node_range = node.text_range();
    let tokens = parsed.syntax_token_views();
    let start_index = tokens.partition_point(|token| token.range.start() < node_range.start());
    let end_index = tokens[start_index..]
        .partition_point(|token| token.range.start() < node_range.end())
        + start_index;
    tokens[start_index..end_index]
        .iter()
        .filter(|token| token.range.end() <= node_range.end())
        .map(|token| {
            let range = token.range;
            let start = u32::from(range.start()) as usize;
            let end = u32::from(range.end()) as usize;
            Token {
                kind: token.kind,
                text: text.get(start..end).unwrap_or_default(),
                range,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{StyleDialect, parse};

    fn tokens_from_syntax_node_linear<'text>(
        text: &'text str,
        parsed: &ParseResult,
        node: &SyntaxNode<SyntaxKind>,
    ) -> Vec<Token<'text>> {
        let node_range = node.text_range();
        parsed
            .syntax_token_views()
            .iter()
            .filter(|token| token.range.start() >= node_range.start())
            .filter(|token| token.range.end() <= node_range.end())
            .map(|token| {
                let range = token.range;
                let start = u32::from(range.start()) as usize;
                let end = u32::from(range.end()) as usize;
                Token {
                    kind: token.kind,
                    text: text.get(start..end).unwrap_or_default(),
                    range,
                }
            })
            .collect()
    }

    #[test]
    fn tokens_from_syntax_node_matches_linear_scan_order() {
        let text = r#"@use "./tokens" as t;
:export { exported: local; }
.button, :global(.card) {
  --gap: 1rem;
  color: var(--brand);
  &__icon { composes: icon from "./icons.module.css"; }
}
@media (width >= 1px) {
  .button--primary { color: t.$brand; }
}"#;
        let parsed = parse(text, StyleDialect::Scss);
        let syntax = parsed.syntax();

        for node in syntax.descendants() {
            assert_eq!(
                tokens_from_syntax_node(text, &parsed, node),
                tokens_from_syntax_node_linear(text, &parsed, node),
                "token slice drift for {:?} at {:?}",
                node.kind(),
                node.text_range()
            );
        }
    }
}
