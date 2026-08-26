//! Aggregated parser fact surface.
//!
//! This module re-exports the syntax-derived fact records that are safe for
//! query, bridge, LSP, and transform consumers to share.

mod animations;
mod at_rules;
mod css_modules;
mod emission_selectors;
mod icss;
mod sass;
mod selectors;
mod variables;

#[cfg(test)]
use cstree::syntax::SyntaxNode;
use cstree::{
    green::GreenNode,
    text::{TextRange, TextSize},
    util::NodeOrToken,
};
use omena_syntax::{StyleDialect, SyntaxKind};

use crate::{DialectExtension, ParseResult, Parser, Token, tokenize};

pub(crate) const STYLE_FACT_FAMILIES: &[&str] = &[
    "selectors",
    "variables",
    "sass-symbols",
    "sass-includes",
    "sass-module-edges",
    "sass-placeholder-definitions",
    "extend-targets",
    "animations",
    "css-module-values",
    "css-module-value-import-edges",
    "css-module-value-definition-edges",
    "css-module-composes",
    "css-module-composes-edges",
    "icss",
    "icss-import-edges",
    "icss-export-edges",
    "at-rules",
    "emission-selectors",
];

/// Shared event index for parser fact handlers.
///
/// Construction performs the only CST descendant walk. Fact-family handlers
/// consume the resulting node events and the single source-token view instead
/// of rematerializing or retraversing the tree independently.
pub(crate) struct StyleFactSink<'text> {
    text: &'text str,
    dialect: StyleDialect,
    error_count: usize,
    tokens: Vec<Token<'text>>,
    nodes: Vec<StyleFactNodeEvent>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct StyleFactNodeEvent {
    pub(crate) index: usize,
    pub(crate) kind: SyntaxKind,
    pub(crate) range: TextRange,
    pub(crate) parent: Option<usize>,
    pub(crate) is_top_level: bool,
}

impl<'text> StyleFactSink<'text> {
    pub(crate) fn from_cst(text: &'text str, parsed: &ParseResult) -> Self {
        crate::record_omena_parser_fact_collection_traversal("style-fact-sink");
        crate::record_omena_parser_fact_collection_registrations(STYLE_FACT_FAMILIES);
        let mut tokens = Vec::with_capacity(parsed.token_count());
        let mut nodes = Vec::new();
        collect_style_fact_events_from_green(
            parsed.green(),
            TextSize::from(0),
            None,
            text,
            &mut tokens,
            &mut nodes,
        );
        Self {
            text,
            dialect: parsed.dialect(),
            error_count: parsed.errors().len(),
            tokens,
            nodes,
        }
    }

    pub(crate) fn text(&self) -> &'text str {
        self.text
    }

    pub(crate) fn dialect(&self) -> StyleDialect {
        self.dialect
    }

    pub(crate) fn error_count(&self) -> usize {
        self.error_count
    }

    pub(crate) fn tokens(&self) -> &[Token<'text>] {
        self.tokens.as_slice()
    }

    pub(crate) fn nodes(&self) -> impl Iterator<Item = &StyleFactNodeEvent> {
        self.nodes.iter()
    }

    pub(crate) fn node_tokens(&self, node: &StyleFactNodeEvent) -> &[Token<'text>] {
        let node_range = node.range;
        let start = self
            .tokens
            .partition_point(|token| token.range.start() < node_range.start());
        let end = self.tokens[start..]
            .partition_point(|token| token.range.start() < node_range.end())
            + start;
        &self.tokens[start..end]
    }

    pub(crate) fn has_token_kind(&self, kind: SyntaxKind) -> bool {
        self.tokens.iter().any(|token| token.kind == kind)
    }

    pub(crate) fn ancestors_inclusive(
        &self,
        node: &StyleFactNodeEvent,
    ) -> Vec<&StyleFactNodeEvent> {
        let mut ancestors = Vec::new();
        let mut current = Some(node.index);
        while let Some(index) = current {
            let ancestor = &self.nodes[index];
            ancestors.push(ancestor);
            current = ancestor.parent;
        }
        ancestors
    }

    pub(crate) fn has_intervening_ancestor_kind(
        &self,
        node: &StyleFactNodeEvent,
        stop: &StyleFactNodeEvent,
        kinds: &[SyntaxKind],
    ) -> bool {
        let mut current = node.parent;
        while let Some(index) = current {
            if index == stop.index {
                return false;
            }
            let ancestor = &self.nodes[index];
            if kinds.contains(&ancestor.kind) {
                return true;
            }
            current = ancestor.parent;
        }
        false
    }
}

fn collect_style_fact_events_from_green<'text>(
    node: &GreenNode,
    start: TextSize,
    parent: Option<usize>,
    text: &'text str,
    tokens: &mut Vec<Token<'text>>,
    nodes: &mut Vec<StyleFactNodeEvent>,
) {
    let mut offset = start;
    for child in node.children() {
        match child {
            NodeOrToken::Node(child_node) => {
                let kind =
                    SyntaxKind::from_raw_kind(child_node.kind().0).unwrap_or(SyntaxKind::Unknown);
                let index = nodes.len();
                let is_top_level = parent.is_some_and(|parent| {
                    matches!(
                        nodes[parent].kind,
                        SyntaxKind::Stylesheet
                            | SyntaxKind::ScssStylesheet
                            | SyntaxKind::LessStylesheet
                    )
                });
                nodes.push(StyleFactNodeEvent {
                    index,
                    kind,
                    range: TextRange::at(offset, child_node.text_len()),
                    parent,
                    is_top_level,
                });
                collect_style_fact_events_from_green(
                    child_node,
                    offset,
                    Some(index),
                    text,
                    tokens,
                    nodes,
                );
                offset += child_node.text_len();
            }
            NodeOrToken::Token(token) => {
                let kind = SyntaxKind::from_raw_kind(token.kind().0).unwrap_or(SyntaxKind::Unknown);
                let range = TextRange::at(offset, token.text_len());
                let token_start = u32::from(range.start()) as usize;
                let token_end = u32::from(range.end()) as usize;
                tokens.push(Token {
                    kind,
                    text: text.get(token_start..token_end).unwrap_or_default(),
                    range,
                });
                offset += token.text_len();
            }
        }
    }
}

pub(crate) use animations::collect_animation_facts_from_sink;
pub use animations::{ParsedAnimationFact, ParsedAnimationFactKind};
pub use at_rules::ParsedAtRuleFact;
pub(crate) use at_rules::collect_at_rule_facts_from_sink;
pub use css_modules::{
    ParsedCssModuleComposesEdgeFact, ParsedCssModuleComposesEdgeKind, ParsedCssModuleComposesFact,
    ParsedCssModuleComposesFactKind, ParsedCssModuleValueDefinitionEdgeFact,
    ParsedCssModuleValueFact, ParsedCssModuleValueFactKind, ParsedCssModuleValueImportEdgeFact,
};
pub(crate) use css_modules::{
    collect_css_module_composes_edge_facts_from_sink, collect_css_module_composes_facts_from_sink,
    collect_css_module_value_definition_edge_facts_from_sink,
    collect_css_module_value_definition_edge_names, collect_css_module_value_facts_from_sink,
    collect_css_module_value_import_edge_facts_from_sink,
    css_module_value_reference_token_can_be_name, css_module_value_source_name,
    css_module_value_statement_end, declaration_colon_index,
};
pub(crate) use emission_selectors::collect_emission_selector_facts_from_sink;
pub use emission_selectors::{
    ParsedEmissionSelectorFactKindV0, ParsedEmissionSelectorFactV0, ParsedEmissionSelectorFactsV0,
    collect_emission_selector_facts_from_cst,
};
pub use icss::{
    ParsedIcssExportEdgeFact, ParsedIcssFact, ParsedIcssFactKind, ParsedIcssImportEdgeFact,
    collect_icss_export_values_from_cst,
};
pub(crate) use icss::{
    collect_icss_export_edge_facts_from_sink, collect_icss_facts_from_sink,
    collect_icss_import_edge_facts_from_sink,
};
pub use sass::{
    ParsedExtendTargetFact, ParsedExtendTargetFactKind, ParsedSassCallableParameterFact,
    ParsedSassCallableSignatureFact, ParsedSassIncludeFact, ParsedSassModuleEdgeFact,
    ParsedSassModuleEdgeFactKind, ParsedSassPlaceholderDefinitionFact, ParsedSassSymbolFact,
    ParsedSassSymbolFactKind,
};
pub(crate) use sass::{
    collect_extend_target_facts_from_sink, collect_sass_include_facts_from_sink,
    collect_sass_module_edge_facts_from_sink, collect_sass_placeholder_definition_facts_from_sink,
    collect_sass_symbol_facts_from_sink,
};
pub use selectors::{ParsedSelectorFact, ParsedSelectorFactKind};
pub(crate) use selectors::{
    SelectorBranch, collect_class_selector_names_from_header, collect_selector_facts_from_sink,
    css_module_block_scope_marker_in_header, css_module_header_is_global_only,
    resolve_selector_header, split_selector_groups,
};
pub use variables::{ParsedVariableFact, ParsedVariableFactKind, ParsedVariableFactNameV0};
pub(crate) use variables::{collect_variable_facts_from_sink, scss_variable_token_is_declaration};

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
    pub sass_placeholder_definition_count: usize,
    pub sass_placeholder_definitions: Vec<ParsedSassPlaceholderDefinitionFact>,
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

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct ParsedStyleFactCollectionV0 {
    pub facts: ParsedStyleFacts,
    pub emission_selectors: ParsedEmissionSelectorFactsV0,
}

struct ProductFacts(ParsedStyleFacts);

impl From<ParsedStyleFacts> for ProductFacts {
    fn from(facts: ParsedStyleFacts) -> Self {
        let ParsedStyleFacts {
            product,
            dialect,
            selector_count,
            selectors,
            variable_count,
            variables,
            sass_symbol_count,
            sass_symbols,
            sass_include_count: _,
            sass_includes: _,
            sass_module_edge_count,
            sass_module_edges,
            sass_placeholder_definition_count,
            sass_placeholder_definitions,
            extend_target_count: _,
            extend_targets: _,
            animation_count,
            animations,
            css_module_value_count,
            css_module_values,
            css_module_value_import_edge_count,
            css_module_value_import_edges,
            css_module_value_definition_edge_count,
            css_module_value_definition_edges,
            css_module_composes_count,
            css_module_composes,
            css_module_composes_edge_count,
            css_module_composes_edges,
            icss_count: _,
            icss: _,
            icss_import_edge_count: _,
            icss_import_edges: _,
            icss_export_edge_count: _,
            icss_export_edges: _,
            at_rule_count: _,
            at_rules: _,
            error_count,
        } = facts;
        let include_sass_declarations = matches!(dialect, StyleDialect::Scss | StyleDialect::Sass);
        let (
            sass_symbol_count,
            sass_symbols,
            sass_module_edge_count,
            sass_module_edges,
            sass_placeholder_definition_count,
            sass_placeholder_definitions,
        ) = if include_sass_declarations {
            (
                sass_symbol_count,
                sass_symbols,
                sass_module_edge_count,
                sass_module_edges,
                sass_placeholder_definition_count,
                sass_placeholder_definitions,
            )
        } else {
            (0, Vec::new(), 0, Vec::new(), 0, Vec::new())
        };

        Self(ParsedStyleFacts {
            product,
            dialect,
            selector_count,
            selectors,
            variable_count,
            variables,
            sass_symbol_count,
            sass_symbols,
            sass_include_count: 0,
            sass_includes: Vec::new(),
            sass_module_edge_count,
            sass_module_edges,
            sass_placeholder_definition_count,
            sass_placeholder_definitions,
            extend_target_count: 0,
            extend_targets: Vec::new(),
            animation_count,
            animations,
            css_module_value_count,
            css_module_values,
            css_module_value_import_edge_count,
            css_module_value_import_edges,
            css_module_value_definition_edge_count,
            css_module_value_definition_edges,
            css_module_composes_count,
            css_module_composes,
            css_module_composes_edge_count,
            css_module_composes_edges,
            icss_count: 0,
            icss: Vec::new(),
            icss_import_edge_count: 0,
            icss_import_edges: Vec::new(),
            icss_export_edge_count: 0,
            icss_export_edges: Vec::new(),
            at_rule_count: 0,
            at_rules: Vec::new(),
            error_count,
        })
    }
}

impl From<ProductFacts> for ParsedStyleFacts {
    fn from(facts: ProductFacts) -> Self {
        facts.0
    }
}

pub fn collect_style_facts_with_extension(
    text: &str,
    extension: &impl DialectExtension,
) -> ParsedStyleFacts {
    let parsed = parse_style_fact_source(text, extension);
    facts_from_cst(text, &parsed)
}

pub fn collect_style_fact_collection_with_extension(
    text: &str,
    extension: &impl DialectExtension,
) -> ParsedStyleFactCollectionV0 {
    let parsed = parse_style_fact_source(text, extension);
    let sink = StyleFactSink::from_cst(text, &parsed);
    ParsedStyleFactCollectionV0 {
        facts: facts_from_sink(&sink),
        emission_selectors: collect_emission_selector_facts_from_sink(&sink),
    }
}

fn parse_style_fact_source(text: &str, extension: &impl DialectExtension) -> ParseResult {
    let (tokens, lex_errors) = tokenize(text, extension);
    let token_count = tokens.len();
    let mut parser = Parser::new(tokens, lex_errors, extension.dialect());
    crate::record_omena_parser_parse_materialization(token_count);
    let (green, interner) = parser.parse();
    let errors = parser.into_errors();
    ParseResult::new(green, interner, errors, token_count, extension.dialect())
}

pub fn facts_from_cst(text: &str, parsed: &ParseResult) -> ParsedStyleFacts {
    let sink = StyleFactSink::from_cst(text, parsed);
    facts_from_sink(&sink)
}

fn facts_from_sink(sink: &StyleFactSink<'_>) -> ParsedStyleFacts {
    let selectors = collect_selector_facts_from_sink(sink);
    let variables = collect_variable_facts_from_sink(sink);
    let sass_symbols = collect_sass_symbol_facts_from_sink(sink);
    let sass_includes = collect_sass_include_facts_from_sink(sink);
    let sass_module_edges = collect_sass_module_edge_facts_from_sink(sink);
    let sass_placeholder_definitions = collect_sass_placeholder_definition_facts_from_sink(sink);
    let extend_targets = collect_extend_target_facts_from_sink(sink);
    let animations = collect_animation_facts_from_sink(sink);
    let css_module_values = collect_css_module_value_facts_from_sink(sink);
    let css_module_value_import_edges = collect_css_module_value_import_edge_facts_from_sink(sink);
    let css_module_value_definition_edges =
        collect_css_module_value_definition_edge_facts_from_sink(sink);
    let css_module_composes = collect_css_module_composes_facts_from_sink(sink);
    let css_module_composes_edges = collect_css_module_composes_edge_facts_from_sink(sink);
    let icss = collect_icss_facts_from_sink(sink);
    let icss_import_edges = collect_icss_import_edge_facts_from_sink(sink);
    let icss_export_edges = collect_icss_export_edge_facts_from_sink(sink);
    let at_rules = collect_at_rule_facts_from_sink(sink);

    ParsedStyleFacts {
        product: "omena-parser.style-facts",
        dialect: sink.dialect(),
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
        sass_placeholder_definition_count: sass_placeholder_definitions.len(),
        sass_placeholder_definitions,
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
        error_count: sink.error_count(),
    }
}

pub(crate) fn product_facts_from_cst(text: &str, parsed: &ParseResult) -> ParsedStyleFacts {
    ProductFacts::from(facts_from_cst(text, parsed)).into()
}

#[cfg(test)]
mod product_facts_authority_tests;

#[cfg(test)]
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

    const FULL_FACT_IDENTITY_FIXTURE: &str = r#"@use "./tokens" as t;
@forward "./theme";
@import "./legacy";
$brand: red !default;
@mixin paint($tone: red) { @content; color: $tone; }
@function scale($value) { @return $value; }
@value spacing: 1rem;
@value gap: spacing;
@value remoteTone as tone from "./tokens.css";
:import("./dep.css") { localName: remoteName; }
:export { exported: localName; }
%shared { color: $brand; }
#app, div[data-state="open"]::before { --accent: blue; }
.button {
  @include paint($brand) { color: scale(1); }
  @extend %shared !optional;
  composes: base global(shell) from "./base.css";
  animation: pulse 1s ease;
  padding: gap;
}
@keyframes pulse { from { opacity: 0; } to { opacity: 1; } }
"#;

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

    #[test]
    fn combined_fact_sink_preserves_the_full_pre_change_fact_bytes() {
        let (collection, instrumentation) =
            crate::instrumentation::with_omena_parser_fact_collection_instrumentation(|| {
                crate::collect_style_fact_collection(FULL_FACT_IDENTITY_FIXTURE, StyleDialect::Scss)
            });

        assert_eq!(instrumentation.traversal_entry_count, 1);
        assert_eq!(instrumentation.families, ["style-fact-sink"]);
        assert_eq!(instrumentation.registered_family_count, 18);
        assert_eq!(instrumentation.registered_families, STYLE_FACT_FAMILIES);

        let facts = &collection.facts;
        assert!(
            facts.selector_count > 0,
            "selectors handler produced no rows"
        );
        assert!(
            facts.variable_count > 0,
            "variables handler produced no rows"
        );
        assert!(
            facts.sass_symbol_count > 0,
            "sass-symbols handler produced no rows"
        );
        assert!(
            facts.sass_include_count > 0,
            "sass-includes handler produced no rows"
        );
        assert!(
            facts.sass_module_edge_count > 0,
            "sass-module-edges handler produced no rows"
        );
        assert!(
            facts.sass_placeholder_definition_count > 0,
            "sass-placeholder-definitions handler produced no rows"
        );
        assert!(
            facts.extend_target_count > 0,
            "extend-targets handler produced no rows"
        );
        assert!(
            facts.animation_count > 0,
            "animations handler produced no rows"
        );
        assert!(
            facts.css_module_value_count > 0,
            "css-module-values handler produced no rows"
        );
        assert!(
            facts.css_module_value_import_edge_count > 0,
            "css-module-value-import-edges handler produced no rows"
        );
        assert!(
            facts.css_module_value_definition_edge_count > 0,
            "css-module-value-definition-edges handler produced no rows"
        );
        assert!(
            facts.css_module_composes_count > 0,
            "css-module-composes handler produced no rows"
        );
        assert!(
            facts.css_module_composes_edge_count > 0,
            "css-module-composes-edges handler produced no rows"
        );
        assert!(facts.icss_count > 0, "icss handler produced no rows");
        assert!(
            facts.icss_import_edge_count > 0,
            "icss-import-edges handler produced no rows"
        );
        assert!(
            facts.icss_export_edge_count > 0,
            "icss-export-edges handler produced no rows"
        );
        assert!(facts.at_rule_count > 0, "at-rules handler produced no rows");
        assert!(
            !collection.emission_selectors.selectors.is_empty(),
            "emission-selectors handler produced no rows"
        );

        let bytes = format!("{collection:#?}");
        let fingerprint = bytes.bytes().fold(0xcbf29ce484222325_u64, |hash, byte| {
            (hash ^ u64::from(byte)).wrapping_mul(0x100000001b3)
        });
        assert_eq!(fingerprint, 0x60bac74d4ed97b4c);
    }
}
