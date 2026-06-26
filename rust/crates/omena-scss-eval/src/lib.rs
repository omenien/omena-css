//! Native SCSS/Less value evaluator with retained oracle evidence.
//!
//! This crate is the native evaluator entry point above parser facts and the
//! shared abstract value vocabulary. Product runtime paths consume the native
//! edit output when available, while the legacy `evaluated_css` string stays in
//! the model as retained byte evidence. Covered fixture slices are checked by
//! internal value oracles and the external differential gate before native edits
//! are widened.

mod control_flow;
mod eval_mode;
mod native_css;
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
    OmenaScssEvalControlFlowBlockIdV0, OmenaScssEvalControlFlowBlockV0,
    OmenaScssEvalControlFlowEdgeV0, OmenaScssEvalControlFlowGraphBlockV0,
    OmenaScssEvalControlFlowGraphV0, OmenaScssEvalControlFlowIrSummaryV0,
    OmenaScssEvalControlFlowOracleCorpusFixtureReportV0,
    OmenaScssEvalControlFlowOracleCorpusReportV0, OmenaScssEvalControlFlowPruneReachabilityV0,
    OmenaScssEvalControlFlowValueAnalysisV0, OmenaScssEvalControlFlowValueBlockV0,
    OmenaScssEvalControlFlowWideningWitnessV0, OmenaScssEvalTypedValueKindCountV0,
    OmenaScssEvalTypedValueLatticeWitnessV0, analyze_scss_control_flow_values,
    build_scss_control_flow_graph, summarize_scss_call_return_ir, summarize_scss_control_flow_ir,
    summarize_scss_control_flow_oracle_corpus, summarize_scss_control_flow_prune_reachability,
    summarize_typed_value_lattice_witness,
};
pub use eval_mode::with_legacy_scss_eval_scanner_path;
pub use native_css::{
    OmenaScssEvalNativeCssFunctionCallArgumentV0,
    OmenaScssEvalNativeCssFunctionCallEvaluationSurfaceV0,
    OmenaScssEvalNativeCssFunctionCallEvaluationV0, OmenaScssEvalNativeCssFunctionParameterV0,
    OmenaScssEvalNativeCssFunctionResultV0, OmenaScssEvalNativeCssFunctionSurfaceV0,
    OmenaScssEvalNativeCssFunctionV0, OmenaScssEvalNativeCssIfFunctionBranchV0,
    OmenaScssEvalNativeCssIfFunctionDecisionSurfaceV0, OmenaScssEvalNativeCssIfFunctionDecisionV0,
    OmenaScssEvalNativeCssStaticEditPlanV0, OmenaScssEvalNativeCssStaticEditV0,
    summarize_native_css_function_call_evaluations, summarize_native_css_function_surface,
    summarize_native_css_if_function_decisions, summarize_native_css_static_edit_plan,
};
pub use static_stylesheet::{
    OmenaScssEvalResolvedReplacementV0, OmenaScssEvalStaticStylesheetEvaluationV0,
    OmenaScssEvalStaticStylesheetNativeEditV0,
    OmenaScssEvalStaticStylesheetOracleCorpusFixtureReportV0,
    OmenaScssEvalStaticStylesheetOracleCorpusReportV0, OmenaScssEvalStaticValueResolutionReportV0,
    OmenaScssEvalStaticValueResolutionV0, canonical_static_scss_variable_name,
    derive_static_scss_stylesheet_module_configurable_variable_names,
    derive_static_scss_stylesheet_module_variable_exports,
    derive_static_stylesheet_module_evaluation, static_scss_variable_names_equal,
    summarize_static_stylesheet_oracle_corpus, summarize_static_stylesheet_value_resolution,
};
pub use value_eval::{
    OmenaScssEvalTruthinessCstEquivalenceFixtureReportV0,
    OmenaScssEvalTruthinessCstEquivalenceReportV0, summarize_scss_eval_truthiness_cst_equivalence,
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

/// Value-WELL-FORMEDNESS self-check on the candidate native-edit output.
///
/// This function does not invoke an external SCSS/Less compiler. In the product path
/// `candidate_evaluated_css` is the native-edit output (source with native edits applied), so
/// `divergence_count == 0` / `all_legacy_declaration_values_preserved` mean "every native-emitted
/// declaration value canonically round-trips", not "native agrees with an external evaluator".
/// The separate `externalDifferential` gate compares covered fixture slices against pinned
/// dart-sass/lessc versions; this self-check remains the cheap inner oracle for every evaluated
/// candidate. The `legacy*` vocabulary is retained for the serialized contract and denotes the
/// retained product-output string, not an independent ground truth.
pub fn summarize_omena_scss_eval_oracle(
    source: &str,
    dialect: StyleDialect,
    candidate_evaluated_css: &str,
) -> OmenaScssEvalOracleReportV0 {
    let source_facts = collect_style_facts(source, dialect);
    let source_variable_reference_count = source_facts
        .variables
        .iter()
        .filter(|fact| parsed_variable_fact_kind_is_reference(fact.kind))
        .count();
    let values = collect_legacy_declaration_values(candidate_evaluated_css, dialect)
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
        AbstractCssValueV0::Exact { value, .. } | AbstractCssValueV0::Raw { value } => {
            value.clone()
        }
        AbstractCssValueV0::FiniteSet { values, .. } => values.join(" | "),
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
        AbstractCssValueV0::FiniteSet { values, .. } => values
            .iter()
            .any(|value| abstract_css_values_canonically_equal(legacy_value, value)),
        AbstractCssValueV0::Raw { .. } => legacy_value == rendered_value,
        AbstractCssValueV0::Top => false,
    }
}

pub(crate) fn abstract_css_value_reflected_in_legacy_css(
    legacy_evaluated_css: &str,
    dialect: StyleDialect,
    rendered_value: &str,
    abstract_value: &AbstractCssValueV0,
) -> bool {
    collect_legacy_declaration_values(legacy_evaluated_css, dialect)
        .into_iter()
        .any(|declaration| {
            let legacy_value = declaration.value.trim();
            abstract_css_value_matches_legacy(legacy_value, rendered_value, abstract_value)
                || legacy_declaration_value_contains_rendered_value(legacy_value, rendered_value)
        })
}

fn legacy_declaration_value_contains_rendered_value(
    legacy_value: &str,
    rendered_value: &str,
) -> bool {
    let rendered_value = rendered_value.trim();
    if rendered_value.is_empty() {
        return legacy_value.trim().is_empty();
    }
    legacy_value
        .match_indices(rendered_value)
        .any(|(start, _)| {
            let end = start + rendered_value.len();
            css_value_fragment_left_boundary(legacy_value, start)
                && css_value_fragment_right_boundary(legacy_value, end)
        })
}

fn css_value_fragment_left_boundary(source: &str, byte_index: usize) -> bool {
    if byte_index == 0 {
        return true;
    }
    source
        .get(..byte_index)
        .and_then(|text| text.chars().next_back())
        .is_some_and(|character| !css_value_fragment_char_is_ident_like(character))
}

fn css_value_fragment_right_boundary(source: &str, byte_index: usize) -> bool {
    if byte_index == source.len() {
        return true;
    }
    source
        .get(byte_index..)
        .and_then(|text| text.chars().next())
        .is_some_and(|character| !css_value_fragment_char_is_ident_like(character))
}

fn css_value_fragment_char_is_ident_like(character: char) -> bool {
    character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '%' | '.' | '#')
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
        if let Some(close_index) = matching_declaration_block_end_index(tokens, index) {
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
        if let Some(close_index) = matching_declaration_block_end_index(tokens, index) {
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
            SyntaxKind::Semicolon | SyntaxKind::SassOptionalSemicolon => {
                return build_declaration_value(property_name, value_tokens, index + 1);
            }
            SyntaxKind::LeftBrace
            | SyntaxKind::RightBrace
            | SyntaxKind::SassIndent
            | SyntaxKind::SassDedent => return None,
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

fn matching_declaration_block_end_index(
    tokens: &[LexedToken],
    start_index: usize,
) -> Option<usize> {
    match tokens.get(start_index)?.kind {
        SyntaxKind::LeftBrace => matching_block_end_index(
            tokens,
            start_index,
            SyntaxKind::LeftBrace,
            SyntaxKind::RightBrace,
        ),
        SyntaxKind::SassIndent => matching_block_end_index(
            tokens,
            start_index,
            SyntaxKind::SassIndent,
            SyntaxKind::SassDedent,
        ),
        _ => None,
    }
}

fn matching_block_end_index(
    tokens: &[LexedToken],
    start_index: usize,
    open_kind: SyntaxKind,
    close_kind: SyntaxKind,
) -> Option<usize> {
    let mut depth = 0usize;
    for (index, token) in tokens.iter().enumerate().skip(start_index) {
        if token.kind == open_kind {
            depth += 1;
        } else if token.kind == close_kind {
            depth = depth.checked_sub(1)?;
            if depth == 0 {
                return Some(index);
            }
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

    #[test]
    fn oracle_collects_declaration_values_inside_sass_indented_rules() {
        let report = summarize_omena_scss_eval_oracle(
            ".button\n  color: red",
            StyleDialect::Sass,
            ".button\n  color: red",
        );

        assert_eq!(report.legacy_declaration_value_count, 1);
        assert_eq!(report.divergence_count, 0);
        assert_eq!(
            report
                .values
                .first()
                .map(|value| (value.property_name.as_str(), value.legacy_value.as_str())),
            Some(("color", "red"))
        );
    }
}
