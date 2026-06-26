use cstree::syntax::SyntaxNode;
use omena_parser::{ParseEntryPoint, StyleDialect, parse_entry_point};
use omena_syntax::SyntaxKind;
use omena_value_lattice::{css_values_canonically_equal, parse_numeric_value_with_unit};
use serde::Serialize;

#[cfg(test)]
use omena_abstract_value::AbstractCssTypedComparisonOperatorV0;

#[cfg(test)]
use super::numeric::static_scss_typed_advisory_numeric_comparison;
use super::reduce_static_scss_value;

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
        .or_else(|| static_scss_scanner_literal_truthiness(value))
}

pub fn summarize_scss_eval_truthiness_cst_equivalence()
-> OmenaScssEvalTruthinessCstEquivalenceReportV0 {
    let fixtures = SCSS_TRUTHINESS_CST_EQUIVALENCE_FIXTURES
        .iter()
        .map(|fixture| {
            let scanner_truthiness = static_scss_scanner_literal_truthiness(fixture.value);
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

fn static_scss_scanner_literal_truthiness(value: &str) -> Option<bool> {
    let trimmed = value.trim();
    if let Some(inner) = strip_static_scss_outer_parens(trimmed) {
        return static_scss_scanner_literal_truthiness(inner);
    }
    match split_static_scss_boolean_operands(trimmed, "or") {
        Ok(Some(operands)) => return static_scss_or_truthiness(operands),
        Ok(None) => {}
        Err(()) => return None,
    }
    match split_static_scss_boolean_operands(trimmed, "and") {
        Ok(Some(operands)) => return static_scss_and_truthiness(operands),
        Ok(None) => {}
        Err(()) => return None,
    }
    if trimmed
        .get(..3)
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case("not"))
        && let Some(operand) = trimmed.get(3..)
        && operand.chars().next().is_some_and(char::is_whitespace)
    {
        return static_scss_scanner_literal_truthiness(operand.trim()).map(|truthy| !truthy);
    }
    match static_scss_comparison_truthiness(trimmed) {
        Ok(Some(truthy)) => return Some(truthy),
        Ok(None) => {}
        Err(()) => return None,
    }
    static_scss_leaf_truthiness(trimmed)
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
    static_scss_cst_prefixed_not_truthiness(trimmed)
        .or_else(|| static_scss_cst_truthiness_for_node(trimmed, &root))
}

#[cfg(test)]
pub(crate) fn static_scss_typed_advisory_truthiness(value: &str) -> Option<bool> {
    let (left, operator, right) = split_static_scss_comparison(value).ok()??;
    static_scss_typed_advisory_numeric_comparison(left, typed_comparison_operator(operator), right)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StaticScssComparisonOperator {
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

fn static_scss_comparison_truthiness(value: &str) -> Result<Option<bool>, ()> {
    let Some((left, operator, right)) = split_static_scss_comparison(value)? else {
        return Ok(None);
    };
    static_scss_comparison_operands_truthiness(left, operator, right)
}

fn static_scss_comparison_operands_truthiness(
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
        | SyntaxKind::ScssCondition
        | SyntaxKind::LessCondition => static_scss_cst_wrapped_truthiness(source, node)
            .or_else(|| static_scss_leaf_truthiness(syntax_node_text(node)?.trim())),
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

fn static_scss_cst_prefixed_not_truthiness(value: &str) -> Option<bool> {
    if !value
        .get(..3)
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case("not"))
    {
        return None;
    }
    let operand = value.get(3..)?;
    if !operand.chars().next().is_some_and(char::is_whitespace) {
        return None;
    }
    static_scss_cst_literal_truthiness(operand.trim()).map(|truthy| !truthy)
}

fn static_scss_cst_wrapped_truthiness(source: &str, node: &SyntaxNode<SyntaxKind>) -> Option<bool> {
    let expression_children = node
        .children()
        .filter(|child| static_scss_cst_node_can_evaluate(child.kind()))
        .collect::<Vec<_>>();
    if expression_children.len() != 1 {
        return None;
    }
    static_scss_cst_truthiness_for_node(source, expression_children[0])
}

fn static_scss_cst_unary_truthiness(source: &str, node: &SyntaxNode<SyntaxKind>) -> Option<bool> {
    let text = syntax_node_text(node)?;
    let trimmed = text.trim_start();
    if !trimmed
        .get(..3)
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case("not"))
    {
        return None;
    }
    let operand = node
        .children()
        .find(|child| static_scss_cst_node_can_evaluate(child.kind()))?;
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
    let operator = syntax_between(source, left, right)?
        .trim()
        .to_ascii_lowercase();
    match operator.as_str() {
        "or" => static_scss_cst_or_truthiness(source, left, right),
        "and" => static_scss_cst_and_truthiness(source, left, right),
        "==" | "!=" | "<" | "<=" | ">" | ">=" => {
            let operator = static_scss_cst_comparison_operator(operator.as_str())?;
            static_scss_comparison_operands_truthiness(
                syntax_node_text(left)?.trim(),
                operator,
                syntax_node_text(right)?.trim(),
            )
            .ok()?
        }
        _ => None,
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

fn static_scss_cst_comparison_operator(value: &str) -> Option<StaticScssComparisonOperator> {
    match value {
        "==" => Some(StaticScssComparisonOperator::Equal),
        "!=" => Some(StaticScssComparisonOperator::NotEqual),
        "<" => Some(StaticScssComparisonOperator::LessThan),
        "<=" => Some(StaticScssComparisonOperator::LessThanOrEqual),
        ">" => Some(StaticScssComparisonOperator::GreaterThan),
        ">=" => Some(StaticScssComparisonOperator::GreaterThanOrEqual),
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

fn syntax_between<'source>(
    source: &'source str,
    left: &SyntaxNode<SyntaxKind>,
    right: &SyntaxNode<SyntaxKind>,
) -> Option<&'source str> {
    let start = u32::from(left.text_range().end()) as usize;
    let end = u32::from(right.text_range().start()) as usize;
    source.get(start..end)
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

fn static_scss_leaf_truthiness(value: &str) -> Option<bool> {
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

fn split_static_scss_comparison(
    value: &str,
) -> Result<Option<(&str, StaticScssComparisonOperator, &str)>, ()> {
    let mut comparison = None;
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;
    let mut quote: Option<char> = None;
    let mut index = 0usize;

    while index < value.len() {
        let ch = value[index..].chars().next().ok_or(())?;
        if let Some(quote_ch) = quote {
            index += ch.len_utf8();
            if ch == '\\' {
                if let Some(escaped) = value[index..].chars().next() {
                    index += escaped.len_utf8();
                }
            } else if ch == quote_ch {
                quote = None;
            }
            continue;
        }
        if matches!(ch, '"' | '\'') {
            quote = Some(ch);
            index += ch.len_utf8();
            continue;
        }
        match ch {
            '(' => paren_depth += 1,
            ')' => paren_depth = paren_depth.checked_sub(1).ok_or(())?,
            '[' => bracket_depth += 1,
            ']' => bracket_depth = bracket_depth.checked_sub(1).ok_or(())?,
            '=' | '!' | '<' | '>' if paren_depth == 0 && bracket_depth == 0 => {
                let (operator, width) = static_scss_comparison_operator_at(value, index)?;
                let left = value.get(..index).ok_or(())?.trim();
                let right = value.get(index + width..).ok_or(())?.trim();
                if left.is_empty() || right.is_empty() || comparison.is_some() {
                    return Err(());
                }
                comparison = Some((left, operator, right));
                index += width;
                continue;
            }
            _ => {}
        }
        index += ch.len_utf8();
    }
    if quote.is_some() || paren_depth != 0 || bracket_depth != 0 {
        return Err(());
    }
    Ok(comparison)
}

fn static_scss_comparison_operator_at(
    value: &str,
    index: usize,
) -> Result<(StaticScssComparisonOperator, usize), ()> {
    let suffix = value.get(index..).ok_or(())?;
    if suffix.starts_with("==") {
        return Ok((StaticScssComparisonOperator::Equal, 2));
    }
    if suffix.starts_with("!=") {
        return Ok((StaticScssComparisonOperator::NotEqual, 2));
    }
    if suffix.starts_with("<=") {
        return Ok((StaticScssComparisonOperator::LessThanOrEqual, 2));
    }
    if suffix.starts_with(">=") {
        return Ok((StaticScssComparisonOperator::GreaterThanOrEqual, 2));
    }
    if suffix.starts_with('<') {
        return Ok((StaticScssComparisonOperator::LessThan, 1));
    }
    if suffix.starts_with('>') {
        return Ok((StaticScssComparisonOperator::GreaterThan, 1));
    }
    Err(())
}

fn strip_static_scss_outer_parens(value: &str) -> Option<&str> {
    let inner_start = value.strip_prefix('(')?;
    value.strip_suffix(')')?;
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;
    let mut quote: Option<char> = None;
    let mut index = 0usize;
    while index < value.len() {
        let ch = value[index..].chars().next()?;
        if let Some(quote_ch) = quote {
            index += ch.len_utf8();
            if ch == '\\' {
                if let Some(escaped) = value[index..].chars().next() {
                    index += escaped.len_utf8();
                }
            } else if ch == quote_ch {
                quote = None;
            }
            continue;
        }
        if matches!(ch, '"' | '\'') {
            quote = Some(ch);
            index += ch.len_utf8();
            continue;
        }
        match ch {
            '(' => paren_depth += 1,
            ')' => {
                paren_depth = paren_depth.checked_sub(1)?;
                if paren_depth == 0 && index + ch.len_utf8() != value.len() {
                    return None;
                }
            }
            '[' => bracket_depth += 1,
            ']' => bracket_depth = bracket_depth.checked_sub(1)?,
            _ => {}
        }
        index += ch.len_utf8();
    }
    (quote.is_none() && paren_depth == 0 && bracket_depth == 0)
        .then(|| inner_start.strip_suffix(')').unwrap_or(inner_start).trim())
}

fn static_scss_or_truthiness(operands: Vec<&str>) -> Option<bool> {
    let mut saw_unknown = false;
    for operand in operands {
        match static_scss_scanner_literal_truthiness(operand) {
            Some(true) => return Some(true),
            Some(false) => {}
            None => saw_unknown = true,
        }
    }
    (!saw_unknown).then_some(false)
}

fn static_scss_and_truthiness(operands: Vec<&str>) -> Option<bool> {
    let mut saw_unknown = false;
    for operand in operands {
        match static_scss_scanner_literal_truthiness(operand) {
            Some(true) => {}
            Some(false) => return Some(false),
            None => saw_unknown = true,
        }
    }
    (!saw_unknown).then_some(true)
}

fn split_static_scss_boolean_operands<'a>(
    value: &'a str,
    keyword: &str,
) -> Result<Option<Vec<&'a str>>, ()> {
    let mut operands = Vec::new();
    let mut cursor = 0usize;
    let mut index = 0usize;
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;
    let mut quote: Option<char> = None;

    while index < value.len() {
        let ch = value[index..].chars().next().ok_or(())?;
        if let Some(quote_ch) = quote {
            index += ch.len_utf8();
            if ch == '\\' {
                if let Some(escaped) = value[index..].chars().next() {
                    index += escaped.len_utf8();
                }
            } else if ch == quote_ch {
                quote = None;
            }
            continue;
        }
        if matches!(ch, '"' | '\'') {
            quote = Some(ch);
            index += ch.len_utf8();
            continue;
        }
        match ch {
            '(' => paren_depth += 1,
            ')' => paren_depth = paren_depth.checked_sub(1).ok_or(())?,
            '[' => bracket_depth += 1,
            ']' => bracket_depth = bracket_depth.checked_sub(1).ok_or(())?,
            _ => {}
        }
        if paren_depth == 0
            && bracket_depth == 0
            && static_scss_boolean_keyword_at(value, index, keyword)
        {
            let operand = value.get(cursor..index).ok_or(())?.trim();
            if operand.is_empty() {
                return Err(());
            }
            operands.push(operand);
            index += keyword.len();
            cursor = index;
            continue;
        }
        index += ch.len_utf8();
    }

    if quote.is_some() || paren_depth != 0 || bracket_depth != 0 {
        return Err(());
    }
    if operands.is_empty() {
        return Ok(None);
    }
    let operand = value.get(cursor..).ok_or(())?.trim();
    if operand.is_empty() {
        return Err(());
    }
    operands.push(operand);
    Ok(Some(operands))
}

fn static_scss_boolean_keyword_at(value: &str, index: usize, keyword: &str) -> bool {
    if !value
        .get(index..)
        .is_some_and(|suffix| suffix.starts_with(keyword))
    {
        return false;
    }
    let before_ok = value
        .get(..index)
        .and_then(|prefix| prefix.chars().next_back())
        .is_some_and(char::is_whitespace);
    let after_index = index + keyword.len();
    let after_ok = value
        .get(after_index..)
        .and_then(|suffix| suffix.chars().next())
        .is_some_and(char::is_whitespace);
    before_ok && after_ok
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
