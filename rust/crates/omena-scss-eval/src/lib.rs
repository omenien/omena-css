//! Oracle-first SCSS/Less value evaluator rail.
//!
//! This crate is the native evaluator entry point above parser facts and the
//! shared abstract value vocabulary. It does not mutate CSS output; the current
//! product path still consumes the legacy `evaluated_css` string while this rail
//! checks that resolved declaration values can be represented as
//! `AbstractCssValueV0`.

mod control_flow;
mod scss_metadata;
mod static_loop_frames;
mod static_stylesheet;
mod value_eval;

use omena_abstract_value::{
    AbstractCssValueV0, abstract_css_value_from_text, abstract_css_values_canonically_equal,
};
use omena_parser::{LexedToken, ParsedVariableFactKind, StyleDialect, collect_style_facts, lex};
use omena_syntax::SyntaxKind;
use serde::Serialize;

pub use control_flow::{
    OmenaScssEvalCallReturnEdgeV0, OmenaScssEvalCallReturnIrSummaryV0,
    OmenaScssEvalCallReturnNodeV0, OmenaScssEvalControlFlowBindingValueV0,
    OmenaScssEvalControlFlowBlockV0, OmenaScssEvalControlFlowIrSummaryV0,
    OmenaScssEvalControlFlowValueAnalysisV0, OmenaScssEvalControlFlowValueBlockV0,
    analyze_scss_control_flow_values, summarize_scss_call_return_ir,
    summarize_scss_control_flow_ir,
};
pub use static_stylesheet::{
    OmenaScssEvalResolvedReplacementV0, OmenaScssEvalStaticStylesheetEvaluationV0,
    OmenaScssEvalStaticStylesheetOracleCorpusFixtureReportV0,
    OmenaScssEvalStaticStylesheetOracleCorpusReportV0, OmenaScssEvalStaticValueResolutionReportV0,
    OmenaScssEvalStaticValueResolutionV0, canonical_static_scss_variable_name,
    derive_static_scss_stylesheet_module_configurable_variable_names,
    derive_static_scss_stylesheet_module_variable_exports,
    derive_static_stylesheet_module_evaluation, static_scss_variable_names_equal,
    summarize_static_stylesheet_oracle_corpus, summarize_static_stylesheet_value_resolution,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaScssEvalOracleReportV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub mode: &'static str,
    pub dialect: &'static str,
    pub source_variable_fact_count: usize,
    pub source_variable_reference_count: usize,
    pub legacy_declaration_value_count: usize,
    pub exact_value_count: usize,
    pub raw_value_count: usize,
    pub bottom_value_count: usize,
    pub top_value_count: usize,
    pub divergence_count: usize,
    pub all_legacy_declaration_values_preserved: bool,
    pub product_output_source: &'static str,
    pub values: Vec<OmenaScssEvalValueOracleV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaScssEvalValueOracleV0 {
    pub property_name: String,
    pub legacy_value: String,
    pub abstract_value: AbstractCssValueV0,
    pub abstract_value_kind: &'static str,
    pub rendered_value: String,
    pub matches_legacy: bool,
}

pub fn summarize_omena_scss_eval_oracle(
    source: &str,
    dialect: StyleDialect,
    legacy_evaluated_css: &str,
) -> OmenaScssEvalOracleReportV0 {
    let source_facts = collect_style_facts(source, dialect);
    let source_variable_reference_count = source_facts
        .variables
        .iter()
        .filter(|fact| parsed_variable_fact_kind_is_reference(fact.kind))
        .count();
    let values = collect_legacy_declaration_values(legacy_evaluated_css, dialect)
        .into_iter()
        .map(evaluate_legacy_declaration_value)
        .collect::<Vec<_>>();

    let exact_value_count = values
        .iter()
        .filter(|value| matches!(value.abstract_value, AbstractCssValueV0::Exact { .. }))
        .count();
    let raw_value_count = values
        .iter()
        .filter(|value| matches!(value.abstract_value, AbstractCssValueV0::Raw { .. }))
        .count();
    let bottom_value_count = values
        .iter()
        .filter(|value| matches!(value.abstract_value, AbstractCssValueV0::Bottom))
        .count();
    let top_value_count = values
        .iter()
        .filter(|value| matches!(value.abstract_value, AbstractCssValueV0::Top))
        .count();
    let divergence_count = values.iter().filter(|value| !value.matches_legacy).count();

    OmenaScssEvalOracleReportV0 {
        schema_version: "0",
        product: "omena-scss-eval.oracle",
        mode: "oracleOnly",
        dialect: dialect_label(dialect),
        source_variable_fact_count: source_facts.variable_count,
        source_variable_reference_count,
        legacy_declaration_value_count: values.len(),
        exact_value_count,
        raw_value_count,
        bottom_value_count,
        top_value_count,
        divergence_count,
        all_legacy_declaration_values_preserved: divergence_count == 0,
        product_output_source: "legacyEvaluatedCss",
        values,
    }
}

fn evaluate_legacy_declaration_value(
    declaration: LegacyDeclarationValueV0,
) -> OmenaScssEvalValueOracleV0 {
    let abstract_value = abstract_css_value_from_text(declaration.value.as_str());
    let rendered_value = render_abstract_css_value_for_oracle(&abstract_value);
    let matches_legacy = abstract_css_value_matches_legacy(
        declaration.value.as_str(),
        rendered_value.as_str(),
        &abstract_value,
    );
    OmenaScssEvalValueOracleV0 {
        property_name: declaration.property_name,
        legacy_value: declaration.value,
        abstract_value_kind: abstract_css_value_kind(&abstract_value),
        abstract_value,
        rendered_value,
        matches_legacy,
    }
}

fn render_abstract_css_value_for_oracle(value: &AbstractCssValueV0) -> String {
    match value {
        AbstractCssValueV0::Bottom => String::new(),
        AbstractCssValueV0::Exact { value } | AbstractCssValueV0::Raw { value } => value.clone(),
        AbstractCssValueV0::FiniteSet { values } => values.join(" | "),
        AbstractCssValueV0::Top => "<top>".to_string(),
    }
}

fn abstract_css_value_matches_legacy(
    legacy_value: &str,
    rendered_value: &str,
    abstract_value: &AbstractCssValueV0,
) -> bool {
    let legacy_value = legacy_value.trim();
    match abstract_value {
        AbstractCssValueV0::Bottom => legacy_value.is_empty(),
        AbstractCssValueV0::Exact { .. } => {
            legacy_value == rendered_value
                || abstract_css_values_canonically_equal(legacy_value, rendered_value)
        }
        AbstractCssValueV0::FiniteSet { values } => values
            .iter()
            .any(|value| abstract_css_values_canonically_equal(legacy_value, value)),
        AbstractCssValueV0::Raw { .. } => legacy_value == rendered_value,
        AbstractCssValueV0::Top => false,
    }
}

pub(crate) fn abstract_css_value_kind(value: &AbstractCssValueV0) -> &'static str {
    match value {
        AbstractCssValueV0::Bottom => "bottom",
        AbstractCssValueV0::Exact { .. } => "exact",
        AbstractCssValueV0::FiniteSet { .. } => "finiteSet",
        AbstractCssValueV0::Raw { .. } => "raw",
        AbstractCssValueV0::Top => "top",
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct LegacyDeclarationValueV0 {
    property_name: String,
    value: String,
}

fn collect_legacy_declaration_values(
    source: &str,
    dialect: StyleDialect,
) -> Vec<LegacyDeclarationValueV0> {
    let lexed = lex(source, dialect);
    let tokens = lexed.tokens();
    let mut values = Vec::new();
    let mut index = 0usize;

    while index < tokens.len() {
        if tokens[index].kind == SyntaxKind::LeftBrace
            && let Some(close_index) = matching_right_brace_index(tokens, index)
        {
            collect_declaration_values_in_block(tokens, index, close_index, &mut values);
            index = close_index + 1;
        } else {
            index += 1;
        }
    }

    values
}

fn collect_declaration_values_in_block(
    tokens: &[LexedToken],
    block_start: usize,
    block_end: usize,
    values: &mut Vec<LegacyDeclarationValueV0>,
) {
    let mut index = block_start + 1;
    while index < block_end {
        index = skip_trivia_tokens(tokens, index, block_end);
        if index >= block_end {
            break;
        }
        if tokens[index].kind == SyntaxKind::LeftBrace
            && let Some(close_index) = matching_right_brace_index(tokens, index)
        {
            collect_declaration_values_in_block(tokens, index, close_index, values);
            index = close_index + 1;
            continue;
        }
        if let Some((declaration, next_index)) = parse_declaration_value(tokens, index, block_end) {
            values.push(declaration);
            index = next_index;
        } else {
            index += 1;
        }
    }
}

fn parse_declaration_value(
    tokens: &[LexedToken],
    start_index: usize,
    block_end: usize,
) -> Option<(LegacyDeclarationValueV0, usize)> {
    let property_token = tokens.get(start_index)?;
    let property_name = match property_token.kind {
        SyntaxKind::Ident => property_token.text.to_ascii_lowercase(),
        SyntaxKind::CustomPropertyName => property_token.text.clone(),
        _ => return None,
    };
    let colon_index = skip_trivia_tokens(tokens, start_index + 1, block_end);
    if tokens.get(colon_index)?.kind != SyntaxKind::Colon {
        return None;
    }

    let mut value_tokens = Vec::<&LexedToken>::new();
    let mut index = colon_index + 1;
    while index < block_end {
        match tokens[index].kind {
            SyntaxKind::Semicolon => {
                return build_declaration_value(property_name, value_tokens, index + 1);
            }
            SyntaxKind::LeftBrace | SyntaxKind::RightBrace => return None,
            _ => value_tokens.push(&tokens[index]),
        }
        index += 1;
    }
    build_declaration_value(property_name, value_tokens, index)
}

fn build_declaration_value(
    property_name: String,
    value_tokens: Vec<&LexedToken>,
    next_index: usize,
) -> Option<(LegacyDeclarationValueV0, usize)> {
    if value_tokens
        .iter()
        .any(|token| is_comment_token(token.kind))
    {
        return None;
    }
    let value = value_tokens
        .iter()
        .map(|token| token.text.as_str())
        .collect::<String>()
        .trim()
        .to_string();
    (!value.is_empty()).then_some((
        LegacyDeclarationValueV0 {
            property_name,
            value,
        },
        next_index,
    ))
}

fn matching_right_brace_index(tokens: &[LexedToken], left_brace_index: usize) -> Option<usize> {
    let mut depth = 0usize;
    for (index, token) in tokens.iter().enumerate().skip(left_brace_index) {
        match token.kind {
            SyntaxKind::LeftBrace => depth += 1,
            SyntaxKind::RightBrace => {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    return Some(index);
                }
            }
            _ => {}
        }
    }
    None
}

fn skip_trivia_tokens(tokens: &[LexedToken], mut index: usize, end_exclusive: usize) -> usize {
    while index < end_exclusive && is_trivia_token(tokens[index].kind) {
        index += 1;
    }
    index
}

fn is_trivia_token(kind: SyntaxKind) -> bool {
    is_comment_token(kind)
        || matches!(
            kind,
            SyntaxKind::Whitespace | SyntaxKind::SassIndentedNewline
        )
}

fn is_comment_token(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::LineComment | SyntaxKind::BlockComment | SyntaxKind::ScssSilentComment
    )
}

fn dialect_label(dialect: StyleDialect) -> &'static str {
    match dialect {
        StyleDialect::Css => "css",
        StyleDialect::Scss => "scss",
        StyleDialect::Sass => "sass",
        StyleDialect::Less => "less",
    }
}

fn parsed_variable_fact_kind_is_reference(kind: ParsedVariableFactKind) -> bool {
    matches!(
        kind,
        ParsedVariableFactKind::ScssReference
            | ParsedVariableFactKind::LessReference
            | ParsedVariableFactKind::CustomPropertyReference
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn oracle_preserves_static_scss_values_as_abstract_css_values() {
        let report = summarize_omena_scss_eval_oracle(
            "$brand: red; .button { color: $brand; margin: 0px; }",
            StyleDialect::Scss,
            " .button { color: red; margin: 0px; }",
        );

        assert_eq!(report.mode, "oracleOnly");
        assert_eq!(report.product_output_source, "legacyEvaluatedCss");
        assert_eq!(report.source_variable_reference_count, 1);
        assert_eq!(report.legacy_declaration_value_count, 2);
        assert_eq!(report.divergence_count, 0);
        assert!(report.all_legacy_declaration_values_preserved);
        assert!(
            report
                .values
                .iter()
                .any(|value| value.property_name == "margin"
                    && value.legacy_value == "0px"
                    && value.rendered_value == "0"
                    && value.abstract_value_kind == "exact")
        );
    }

    #[test]
    fn oracle_keeps_unresolved_dynamic_values_raw() {
        let report = summarize_omena_scss_eval_oracle(
            "@brand: @missing; .button { color: @brand; }",
            StyleDialect::Less,
            ".button { color: @missing; }",
        );

        assert_eq!(report.dialect, "less");
        assert_eq!(report.raw_value_count, 1);
        assert_eq!(report.divergence_count, 0);
        assert_eq!(
            report.values.first().map(|value| value.abstract_value_kind),
            Some("raw")
        );
    }

    #[test]
    fn oracle_collects_declaration_values_inside_nested_at_rules() {
        let report = summarize_omena_scss_eval_oracle(
            "$brand: #fff; @media (min-width: 40rem) { .button { color: $brand; } }",
            StyleDialect::Scss,
            "@media (min-width: 40rem) { .button { color: #fff; } }",
        );

        assert_eq!(report.legacy_declaration_value_count, 1);
        assert_eq!(report.divergence_count, 0);
        assert_eq!(
            report
                .values
                .first()
                .map(|value| value.legacy_value.as_str()),
            Some("#fff")
        );
    }
}
