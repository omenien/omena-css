use cstree::syntax::{SyntaxNode, SyntaxToken};
use omena_parser::{ParseEntryPoint, StyleDialect, parse_entry_point};
use omena_syntax::SyntaxKind;
use omena_value_lattice::{css_values_canonically_equal, parse_numeric_value_with_unit};
use serde::Serialize;

#[cfg(test)]
use omena_abstract_value::AbstractCssTypedComparisonOperatorV0;

#[cfg(test)]
use super::numeric::static_scss_typed_advisory_numeric_comparison;
use super::reduce_static_scss_value;
use super::truthiness_scanner::scanner_literal_truthiness;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaScssEvalTruthinessCstEquivalenceFixtureReportV0 {
    pub id: &'static str,
    pub value: &'static str,
    pub scanner_truthiness: Option<bool>,
    pub cst_truthiness: Option<bool>,
    pub matches: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaScssEvalTruthinessCstEquivalenceReportV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub fixture_count: usize,
    pub matching_fixture_count: usize,
    pub all_fixtures_match: bool,
    pub closed_gates: Vec<&'static str>,
    pub fixtures: Vec<OmenaScssEvalTruthinessCstEquivalenceFixtureReportV0>,
}

pub(crate) fn static_scss_literal_truthiness(value: &str) -> Option<bool> {
    static_scss_cst_literal_truthiness(value)
}

pub(crate) fn static_scss_literal_truthiness_scanner_oracle(value: &str) -> Option<bool> {
    scanner_literal_truthiness(value)
}

pub fn summarize_scss_eval_truthiness_cst_equivalence()
-> OmenaScssEvalTruthinessCstEquivalenceReportV0 {
    let fixtures = SCSS_TRUTHINESS_CST_EQUIVALENCE_FIXTURES
        .iter()
        .map(|fixture| {
            let scanner_truthiness = scanner_literal_truthiness(fixture.value);
            let cst_truthiness = static_scss_cst_literal_truthiness(fixture.value);
            OmenaScssEvalTruthinessCstEquivalenceFixtureReportV0 {
                id: fixture.id,
                value: fixture.value,
                scanner_truthiness,
                cst_truthiness,
                matches: scanner_truthiness == cst_truthiness,
            }
        })
        .collect::<Vec<_>>();
    let matching_fixture_count = fixtures.iter().filter(|fixture| fixture.matches).count();
    OmenaScssEvalTruthinessCstEquivalenceReportV0 {
        schema_version: "0",
        product: "omena-scss-eval.truthiness-cst-equivalence",
        fixture_count: fixtures.len(),
        matching_fixture_count,
        all_fixtures_match: matching_fixture_count == fixtures.len(),
        closed_gates: vec!["scssEvalTruthinessCstEquivalence"],
        fixtures,
    }
}

fn static_scss_cst_literal_truthiness(value: &str) -> Option<bool> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }
    let parsed = parse_entry_point(trimmed, StyleDialect::Scss, ParseEntryPoint::Value);
    if !parsed.errors().is_empty() {
        return None;
    }
    let root = parsed.syntax();
    static_scss_cst_truthiness_for_node(trimmed, &root)
}

#[cfg(test)]
pub(crate) fn static_scss_typed_advisory_truthiness(value: &str) -> Option<bool> {
    let trimmed = value.trim();
    let parsed = parse_entry_point(trimmed, StyleDialect::Scss, ParseEntryPoint::Value);
    if !parsed.errors().is_empty() {
        return None;
    }
    let root = parsed.syntax();
    let binary = root
        .descendants()
        .find(|node| node.kind() == SyntaxKind::BinaryExpression)?;
    let children = binary
        .children()
        .filter(|child| static_scss_cst_node_can_evaluate(child.kind()))
        .collect::<Vec<_>>();
    let [left, right] = children.as_slice() else {
        return None;
    };
    let StaticScssCstBinaryOperator::Comparison(operator) =
        static_scss_cst_binary_operator(binary, left, right)?
    else {
        return None;
    };
    let left_text = syntax_node_text(left)?;
    let right_text = syntax_node_text(right)?;
    static_scss_typed_advisory_numeric_comparison(
        left_text.trim(),
        typed_comparison_operator(operator),
        right_text.trim(),
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum StaticScssComparisonOperator {
    Equal,
    NotEqual,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
}

#[cfg(test)]
fn typed_comparison_operator(
    operator: StaticScssComparisonOperator,
) -> AbstractCssTypedComparisonOperatorV0 {
    match operator {
        StaticScssComparisonOperator::Equal => AbstractCssTypedComparisonOperatorV0::Equal,
        StaticScssComparisonOperator::NotEqual => AbstractCssTypedComparisonOperatorV0::NotEqual,
        StaticScssComparisonOperator::LessThan => AbstractCssTypedComparisonOperatorV0::LessThan,
        StaticScssComparisonOperator::LessThanOrEqual => {
            AbstractCssTypedComparisonOperatorV0::LessThanOrEqual
        }
        StaticScssComparisonOperator::GreaterThan => {
            AbstractCssTypedComparisonOperatorV0::GreaterThan
        }
        StaticScssComparisonOperator::GreaterThanOrEqual => {
            AbstractCssTypedComparisonOperatorV0::GreaterThanOrEqual
        }
    }
}

pub(super) fn static_scss_comparison_operands_truthiness(
    left: &str,
    operator: StaticScssComparisonOperator,
    right: &str,
) -> Result<Option<bool>, ()> {
    let left = static_scss_comparable_operand(left).ok_or(())?;
    let right = static_scss_comparable_operand(right).ok_or(())?;
    let equal = left == right || css_values_canonically_equal(left.as_str(), right.as_str());
    Ok(Some(match operator {
        StaticScssComparisonOperator::Equal => equal,
        StaticScssComparisonOperator::NotEqual => !equal,
        StaticScssComparisonOperator::LessThan
        | StaticScssComparisonOperator::LessThanOrEqual
        | StaticScssComparisonOperator::GreaterThan
        | StaticScssComparisonOperator::GreaterThanOrEqual => {
            static_scss_numeric_ordering_truthiness(left.as_str(), operator, right.as_str())
                .ok_or(())?
        }
    }))
}

fn static_scss_numeric_ordering_truthiness(
    left: &str,
    operator: StaticScssComparisonOperator,
    right: &str,
) -> Option<bool> {
    let left_value = parse_numeric_value_with_unit(left)?;
    let right_value = parse_numeric_value_with_unit(right)?;
    if !left_value.unit.eq_ignore_ascii_case(right_value.unit)
        && !static_scss_zero_values_share_unitless_canonical_form(left, right)
    {
        return None;
    }
    Some(match operator {
        StaticScssComparisonOperator::LessThan => left_value.value < right_value.value,
        StaticScssComparisonOperator::LessThanOrEqual => left_value.value <= right_value.value,
        StaticScssComparisonOperator::GreaterThan => left_value.value > right_value.value,
        StaticScssComparisonOperator::GreaterThanOrEqual => left_value.value >= right_value.value,
        StaticScssComparisonOperator::Equal | StaticScssComparisonOperator::NotEqual => {
            return None;
        }
    })
}

fn static_scss_cst_truthiness_for_node(
    source: &str,
    node: &SyntaxNode<SyntaxKind>,
) -> Option<bool> {
    match node.kind() {
        SyntaxKind::Value
        | SyntaxKind::ParenthesizedExpression
        | SyntaxKind::ScssList
        | SyntaxKind::ScssCondition
        | SyntaxKind::LessCondition => static_scss_cst_wrapped_truthiness(source, node),
        SyntaxKind::UnaryExpression => static_scss_cst_unary_truthiness(source, node),
        SyntaxKind::BinaryExpression => static_scss_cst_binary_truthiness(source, node),
        SyntaxKind::IdentifierValue
        | SyntaxKind::StringValue
        | SyntaxKind::UnicodeRangeValue
        | SyntaxKind::NumberValue
        | SyntaxKind::PercentageValue
        | SyntaxKind::DimensionValue
        | SyntaxKind::ColorValue
        | SyntaxKind::UrlValue
        | SyntaxKind::ComponentValue
        | SyntaxKind::CustomPropertyValue
        | SyntaxKind::AttributeValue => static_scss_leaf_truthiness(syntax_node_text(node)?.trim()),
        _ => node
            .children()
            .find_map(|child| static_scss_cst_truthiness_for_node(source, child)),
    }
}

fn static_scss_cst_wrapped_truthiness(source: &str, node: &SyntaxNode<SyntaxKind>) -> Option<bool> {
    let expression_children = node
        .children()
        .filter(|child| static_scss_cst_node_can_evaluate(child.kind()))
        .collect::<Vec<_>>();
    match expression_children.as_slice() {
        [child] => static_scss_cst_truthiness_for_node(source, child),
        [operator, operand] if cst_node_is_not_operator(operator) => {
            static_scss_cst_truthiness_for_node(source, operand).map(|truthy| !truthy)
        }
        [] if cst_node_has_single_non_trivia_token(node) => {
            static_scss_leaf_truthiness(syntax_node_text(node)?.trim())
        }
        _ => None,
    }
}

fn static_scss_cst_unary_truthiness(source: &str, node: &SyntaxNode<SyntaxKind>) -> Option<bool> {
    if !cst_node_first_non_trivia_token(node).is_some_and(|token| cst_token_is_not_operator(&token))
    {
        return None;
    }
    let operands = node
        .children()
        .filter(|child| static_scss_cst_node_can_evaluate(child.kind()))
        .collect::<Vec<_>>();
    let operand = operands
        .iter()
        .rev()
        .find(|child| !cst_node_is_not_operator(child))?;
    static_scss_cst_truthiness_for_node(source, operand).map(|truthy| !truthy)
}

fn static_scss_cst_binary_truthiness(source: &str, node: &SyntaxNode<SyntaxKind>) -> Option<bool> {
    let children = node
        .children()
        .filter(|child| static_scss_cst_node_can_evaluate(child.kind()))
        .collect::<Vec<_>>();
    let [left, right] = children.as_slice() else {
        return None;
    };
    match static_scss_cst_binary_operator(node, left, right)? {
        StaticScssCstBinaryOperator::Or => static_scss_cst_or_truthiness(source, left, right),
        StaticScssCstBinaryOperator::And => static_scss_cst_and_truthiness(source, left, right),
        StaticScssCstBinaryOperator::Comparison(operator) => {
            static_scss_comparison_operands_truthiness(
                syntax_node_text(left)?.trim(),
                operator,
                syntax_node_text(right)?.trim(),
            )
            .ok()?
        }
    }
}

fn static_scss_cst_or_truthiness(
    source: &str,
    left: &SyntaxNode<SyntaxKind>,
    right: &SyntaxNode<SyntaxKind>,
) -> Option<bool> {
    match (
        static_scss_cst_truthiness_for_node(source, left),
        static_scss_cst_truthiness_for_node(source, right),
    ) {
        (Some(true), _) | (_, Some(true)) => Some(true),
        (Some(false), Some(false)) => Some(false),
        _ => None,
    }
}

fn static_scss_cst_and_truthiness(
    source: &str,
    left: &SyntaxNode<SyntaxKind>,
    right: &SyntaxNode<SyntaxKind>,
) -> Option<bool> {
    match (
        static_scss_cst_truthiness_for_node(source, left),
        static_scss_cst_truthiness_for_node(source, right),
    ) {
        (Some(false), _) | (_, Some(false)) => Some(false),
        (Some(true), Some(true)) => Some(true),
        _ => None,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StaticScssCstBinaryOperator {
    Or,
    And,
    Comparison(StaticScssComparisonOperator),
}

fn static_scss_cst_binary_operator(
    node: &SyntaxNode<SyntaxKind>,
    left: &SyntaxNode<SyntaxKind>,
    right: &SyntaxNode<SyntaxKind>,
) -> Option<StaticScssCstBinaryOperator> {
    let start = u32::from(left.text_range().end()) as usize;
    let end = u32::from(right.text_range().start()) as usize;
    let tokens = node
        .descendants_with_tokens()
        .filter_map(|element| element.into_token())
        .filter(|token| {
            let token_start = u32::from(token.text_range().start()) as usize;
            let token_end = u32::from(token.text_range().end()) as usize;
            start <= token_start && token_end <= end && !token.kind().is_trivia()
        })
        .collect::<Vec<_>>();

    match tokens.as_slice() {
        [token] => match token.kind() {
            SyntaxKind::KeywordOr => Some(StaticScssCstBinaryOperator::Or),
            SyntaxKind::KeywordAnd => Some(StaticScssCstBinaryOperator::And),
            SyntaxKind::Ident
                if syntax_token_text(token)
                    .as_deref()
                    .is_some_and(|text| text.eq_ignore_ascii_case("or")) =>
            {
                Some(StaticScssCstBinaryOperator::Or)
            }
            SyntaxKind::Ident
                if syntax_token_text(token)
                    .as_deref()
                    .is_some_and(|text| text.eq_ignore_ascii_case("and")) =>
            {
                Some(StaticScssCstBinaryOperator::And)
            }
            SyntaxKind::LessThan => Some(StaticScssCstBinaryOperator::Comparison(
                StaticScssComparisonOperator::LessThan,
            )),
            SyntaxKind::GreaterThan => Some(StaticScssCstBinaryOperator::Comparison(
                StaticScssComparisonOperator::GreaterThan,
            )),
            _ => None,
        },
        [first, second] if second.kind() == SyntaxKind::Equals => match first.kind() {
            SyntaxKind::LessThan => Some(StaticScssCstBinaryOperator::Comparison(
                StaticScssComparisonOperator::LessThanOrEqual,
            )),
            SyntaxKind::GreaterThan => Some(StaticScssCstBinaryOperator::Comparison(
                StaticScssComparisonOperator::GreaterThanOrEqual,
            )),
            SyntaxKind::Equals => Some(StaticScssCstBinaryOperator::Comparison(
                StaticScssComparisonOperator::Equal,
            )),
            SyntaxKind::Delim
                if syntax_token_text(first)
                    .as_deref()
                    .is_some_and(|text| text == "!") =>
            {
                Some(StaticScssCstBinaryOperator::Comparison(
                    StaticScssComparisonOperator::NotEqual,
                ))
            }
            _ => None,
        },
        _ => None,
    }
}

fn static_scss_cst_node_can_evaluate(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::Value
            | SyntaxKind::BinaryExpression
            | SyntaxKind::UnaryExpression
            | SyntaxKind::ParenthesizedExpression
            | SyntaxKind::ScssList
            | SyntaxKind::ScssCondition
            | SyntaxKind::LessCondition
            | SyntaxKind::IdentifierValue
            | SyntaxKind::StringValue
            | SyntaxKind::UnicodeRangeValue
            | SyntaxKind::NumberValue
            | SyntaxKind::PercentageValue
            | SyntaxKind::DimensionValue
            | SyntaxKind::ColorValue
            | SyntaxKind::UrlValue
            | SyntaxKind::ComponentValue
            | SyntaxKind::CustomPropertyValue
            | SyntaxKind::AttributeValue
    )
}

fn syntax_node_text(node: &SyntaxNode<SyntaxKind>) -> Option<String> {
    let mut text = String::new();
    for token in node
        .descendants_with_tokens()
        .filter_map(|element| element.into_token())
    {
        if let Some(resolver) = token.resolver() {
            text.push_str(token.resolve_text(&**resolver));
        } else if let Some(static_text) = token.static_text() {
            text.push_str(static_text);
        } else {
            return None;
        }
    }
    Some(text)
}

fn syntax_token_text(token: &SyntaxToken<SyntaxKind>) -> Option<String> {
    if let Some(resolver) = token.resolver() {
        Some(token.resolve_text(&**resolver).to_string())
    } else {
        token.static_text().map(str::to_string)
    }
}

fn cst_node_first_non_trivia_token(
    node: &SyntaxNode<SyntaxKind>,
) -> Option<SyntaxToken<SyntaxKind>> {
    node.descendants_with_tokens()
        .filter_map(|element| element.into_token())
        .find(|token| !token.kind().is_trivia())
        .cloned()
}

fn cst_node_is_not_operator(node: &SyntaxNode<SyntaxKind>) -> bool {
    cst_node_first_non_trivia_token(node).is_some_and(|token| cst_token_is_not_operator(&token))
}

fn cst_token_is_not_operator(token: &SyntaxToken<SyntaxKind>) -> bool {
    matches!(token.kind(), SyntaxKind::KeywordNot)
        || (token.kind() == SyntaxKind::Ident
            && syntax_token_text(token)
                .as_deref()
                .is_some_and(|text| text.eq_ignore_ascii_case("not")))
}

fn cst_node_has_single_non_trivia_token(node: &SyntaxNode<SyntaxKind>) -> bool {
    node.descendants_with_tokens()
        .filter_map(|element| element.into_token())
        .filter(|token| !token.kind().is_trivia())
        .take(2)
        .count()
        == 1
}

pub(super) fn static_scss_leaf_truthiness(value: &str) -> Option<bool> {
    let normalized = value.trim().to_ascii_lowercase();
    match normalized.as_str() {
        "false" | "null" => Some(false),
        "" => None,
        _ if normalized.starts_with('$') || normalized.contains('(') => None,
        _ => Some(true),
    }
}

fn static_scss_zero_values_share_unitless_canonical_form(left: &str, right: &str) -> bool {
    let Some(left_value) = parse_numeric_value_with_unit(left) else {
        return false;
    };
    let Some(right_value) = parse_numeric_value_with_unit(right) else {
        return false;
    };
    if left_value.value != 0.0 || right_value.value != 0.0 {
        return false;
    }
    if !left_value.unit.is_empty() && !right_value.unit.is_empty() {
        return false;
    }
    css_values_canonically_equal(left, right)
}

fn static_scss_comparable_operand(value: &str) -> Option<String> {
    let reduced = reduce_static_scss_value(value.trim().to_string());
    let normalized = reduced.to_ascii_lowercase();
    if reduced.is_empty()
        || reduced.contains('$')
        || normalized.contains("var(")
        || normalized.contains("env(")
        || normalized.contains('(')
        || normalized.contains(')')
    {
        return None;
    }
    Some(reduced)
}

#[derive(Debug, Clone, Copy)]
struct ScssTruthinessCstEquivalenceFixtureV0 {
    id: &'static str,
    value: &'static str,
}

const SCSS_TRUTHINESS_CST_EQUIVALENCE_FIXTURES: &[ScssTruthinessCstEquivalenceFixtureV0] = &[
    ScssTruthinessCstEquivalenceFixtureV0 {
        id: "literal.false",
        value: "false",
    },
    ScssTruthinessCstEquivalenceFixtureV0 {
        id: "literal.null",
        value: "null",
    },
    ScssTruthinessCstEquivalenceFixtureV0 {
        id: "literal.truthy-ident",
        value: "red",
    },
    ScssTruthinessCstEquivalenceFixtureV0 {
        id: "literal.unknown-variable",
        value: "$enabled",
    },
    ScssTruthinessCstEquivalenceFixtureV0 {
        id: "literal.unknown-function",
        value: "var(--enabled)",
    },
    ScssTruthinessCstEquivalenceFixtureV0 {
        id: "parenthesized.truthy",
        value: "(true)",
    },
    ScssTruthinessCstEquivalenceFixtureV0 {
        id: "unary.not-false",
        value: "not false",
    },
    ScssTruthinessCstEquivalenceFixtureV0 {
        id: "unary.not-true",
        value: "not true",
    },
    ScssTruthinessCstEquivalenceFixtureV0 {
        id: "logical.or",
        value: "false or true",
    },
    ScssTruthinessCstEquivalenceFixtureV0 {
        id: "logical.and",
        value: "true and false",
    },
    ScssTruthinessCstEquivalenceFixtureV0 {
        id: "logical.nested",
        value: "(false or true) and not null",
    },
    ScssTruthinessCstEquivalenceFixtureV0 {
        id: "comparison.equal",
        value: "1px == 1px",
    },
    ScssTruthinessCstEquivalenceFixtureV0 {
        id: "comparison.not-equal",
        value: "1px != 2px",
    },
    ScssTruthinessCstEquivalenceFixtureV0 {
        id: "comparison.less-than",
        value: "1px < 2px",
    },
    ScssTruthinessCstEquivalenceFixtureV0 {
        id: "comparison.less-than-or-equal",
        value: "1px <= 1px",
    },
    ScssTruthinessCstEquivalenceFixtureV0 {
        id: "comparison.greater-than",
        value: "2px > 1px",
    },
    ScssTruthinessCstEquivalenceFixtureV0 {
        id: "comparison.greater-than-or-equal",
        value: "2px >= 2px",
    },
    ScssTruthinessCstEquivalenceFixtureV0 {
        id: "comparison.incompatible-unit",
        value: "1em == 16px",
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cst_truthiness_matches_scanner_corpus() {
        let report = summarize_scss_eval_truthiness_cst_equivalence();

        assert_eq!(report.product, "omena-scss-eval.truthiness-cst-equivalence");
        assert_eq!(
            report.fixture_count,
            SCSS_TRUTHINESS_CST_EQUIVALENCE_FIXTURES.len()
        );
        assert!(
            report.all_fixtures_match,
            "scanner/CST truthiness diverged: {report:#?}"
        );
        assert!(
            report
                .fixtures
                .iter()
                .any(|fixture| fixture.cst_truthiness == Some(true))
        );
        assert!(
            report
                .fixtures
                .iter()
                .any(|fixture| fixture.cst_truthiness == Some(false))
        );
        assert!(
            report
                .fixtures
                .iter()
                .any(|fixture| fixture.cst_truthiness.is_none())
        );
    }

    #[test]
    fn typed_advisory_truthiness_compares_absolute_dimensions_without_consuming_prunes() {
        assert_eq!(
            static_scss_typed_advisory_truthiness("1in == 96px"),
            Some(true)
        );
        assert_eq!(
            static_scss_typed_advisory_truthiness("2px > 1px"),
            Some(true)
        );
        assert_eq!(static_scss_typed_advisory_truthiness("1em == 16px"), None);
    }
}
