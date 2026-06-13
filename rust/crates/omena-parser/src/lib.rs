//! Green-field parser substrate for omena-css.
//!
//! This crate owns the cstree parser track and publishes parser facts for the
//! product query, bridge, LSP, and transform consumers.

pub use omena_syntax::StyleDialect;
#[cfg(test)]
pub(crate) use omena_syntax::SyntaxKind;

mod cst;
mod extension;
mod facts;
mod language;
mod lex;
mod parse;
// R1 narrow public surface: `public_product` is private; only this curated set
// of V0 contract types + summary fns is re-exported (no wildcard). Reuse of
// omena-parser as a building block goes through these names — keep the list
// explicit and minimal rather than widening to `pub use public_product::*`.
mod public_product;
mod recovery;
mod spans;
mod summaries;
mod syntax_helpers;
mod value_names;
pub use cst::{
    AtRuleCstNode, BogusCstNode, CommaSeparatedComponentValueListCstNode, ComponentValueCstNode,
    ComponentValueListCstNode, CustomPropertyValueCstNode, DeclarationCstNode,
    DeclarationListCstNode, ParsedCst, RuleCstNode, SelectorCstNode, SimpleBlockCstNode,
    StylesheetCstNode, TypedCstNode, ValueCstNode, is_at_rule_node_kind,
};
pub use extension::{BuiltinDialectExtension, DialectExtension};
pub(crate) use extension::{at_rule_spec, scss_at_rule_spec};
pub use facts::{
    ParsedAnimationFact, ParsedAnimationFactKind, ParsedAtRuleFact,
    ParsedCssModuleComposesEdgeFact, ParsedCssModuleComposesEdgeKind, ParsedCssModuleComposesFact,
    ParsedCssModuleComposesFactKind, ParsedCssModuleValueDefinitionEdgeFact,
    ParsedCssModuleValueFact, ParsedCssModuleValueFactKind, ParsedCssModuleValueImportEdgeFact,
    ParsedExtendTargetFact, ParsedExtendTargetFactKind, ParsedIcssExportEdgeFact, ParsedIcssFact,
    ParsedIcssFactKind, ParsedIcssImportEdgeFact, ParsedSassIncludeFact, ParsedSassModuleEdgeFact,
    ParsedSassModuleEdgeFactKind, ParsedSassSymbolFact, ParsedSassSymbolFactKind,
    ParsedSelectorFact, ParsedSelectorFactKind, ParsedStyleFacts, ParsedVariableFact,
    ParsedVariableFactKind, collect_style_facts_with_extension,
};
pub(crate) use facts::{
    SelectorBranch, collect_class_selector_names_from_header,
    css_module_block_scope_marker_in_header, css_module_header_is_global_only,
    resolve_selector_header, split_selector_groups,
};
pub(crate) use facts::{
    collect_css_module_value_definition_edge_names, css_module_value_reference_token_can_be_name,
    css_module_value_source_name, css_module_value_statement_end, declaration_colon_index,
};
pub use language::StyleLanguage;
pub use lex::{LexResult, LexedToken};
pub(crate) use lex::{Token, Tokenizer, public_token_text};
pub use parse::{
    ParseEntryPoint, ParseError, ParseErrorCode, ParseResult, collect_style_facts, lex,
    lex_with_extension, parse, parse_entry_point, parse_entry_point_with_extension,
    parse_with_extension,
};
pub(crate) use parse::{Parser, tokenize};
pub use public_product::{
    ParserCanonicalCandidateBundleV0, ParserCanonicalProducerSignalV0, ParserEvaluatorCandidatesV0,
    ParserIndexSummaryV0, dialect_for_path, summarize_css_modules_intermediate,
    summarize_parser_canonical_candidate, summarize_parser_canonical_producer_signal,
    summarize_parser_evaluator_candidates,
};
pub use recovery::{RECOVERY_DECLARATION, RECOVERY_SELECTOR, RECOVERY_TOP, TokenSet};
pub use spans::{ParserByteSpanV0, ParserPositionV0, ParserRangeV0};
pub use summaries::{
    OmenaParserAtRuleKindCountsV0, OmenaParserCssModuleComposesEdgeFactV0,
    OmenaParserCssModuleValueDefinitionEdgeFactV0, OmenaParserCssModuleValueImportEdgeFactV0,
    OmenaParserDeclarationKindCountsV0, OmenaParserIcssExportEdgeFactV0,
    OmenaParserIcssImportEdgeFactV0, OmenaParserLexSummaryV0, OmenaParserLexTokenV0,
    OmenaParserParityLiteSummaryV0, OmenaParserSassModuleEdgeFactV0, OmenaParserSassSymbolFactV0,
    OmenaParserSassSymbolResolutionCapabilitiesV0, OmenaParserSassSymbolResolutionEdgeV0,
    OmenaParserSassSymbolResolutionV0, OmenaParserStyleFactsSummaryV0, ParserBoundarySummary,
    ParserCstEquivalenceSummaryV0, ParserPrattValueCoverageSummaryV0,
    ParserRecursiveDescentCoverageSummaryV0, ParserSemanticNameConsumptionSummaryV0,
    summarize_omena_parser_lex, summarize_omena_parser_parity_lite,
    summarize_omena_parser_style_facts, summarize_parser_boundary,
    summarize_parser_cst_equivalence, summarize_parser_semantic_name_consumption,
    summarize_pratt_value_parser_coverage, summarize_recursive_descent_parser_coverage,
};
pub(crate) use syntax_helpers::{
    at_rule_prelude_head_is_custom_ident, at_rule_prelude_head_is_custom_property_name,
    attribute_name_token_can_continue, attribute_name_token_can_start,
    attribute_value_token_can_start, bracketed_value_recovery,
    comma_separated_component_value_list_item_recovery, containing_at_rule_header_name,
    css_module_scope_function_kind, find_block_after_header, function_argument_count_is_valid,
    function_argument_recovery, function_requires_filled_top_level_arguments, infix_binding_power,
    interpolation_end_kind, is_at_rule_prelude_boundary, is_attribute_matcher, is_combinator,
    is_component_value_atom_start, is_css_module_from_source_token,
    is_dynamic_function_argument_head, is_interpolation_start, is_nth_pseudo_class,
    is_scss_control_rule_kind, is_scss_module_namespace_token, is_scss_module_source_token,
    is_scss_module_visibility_name_token, is_selector_boundary, is_selector_boundary_until,
    is_selector_combinator_kind, is_selector_list_pseudo_class, is_statement_end,
    keyframe_selector_token_is_valid, language_tag_token_can_start, matches_ignore_ascii_case,
    matching_right_brace, matching_right_paren_from_range, matching_simple_block_close,
    namespace_selector_target_can_start, next_non_trivia_token, next_non_trivia_token_after_range,
    next_non_trivia_token_index_until, next_non_trivia_token_until, previous_non_trivia_token,
    previous_non_trivia_token_index, selector_component_can_end, selector_component_can_start,
    selector_item_token_is_recoverable, simple_block_recovery, skip_statement, skip_trivia_tokens,
    specialized_function_kind, style_wrapper_at_rule, token_index_by_range,
    top_level_token_kind_index, top_level_token_text_index, value_list_item_recovery,
    variable_declaration_node_kind,
};

#[cfg(test)]
mod tests;
