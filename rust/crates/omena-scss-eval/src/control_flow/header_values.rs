use std::collections::BTreeMap;

use cstree::syntax::{SyntaxNode, SyntaxToken};
use omena_abstract_value::{
    AbstractCssValueV0, abstract_css_value_from_text, join_abstract_css_values,
};
use omena_parser::{ParseEntryPoint, StyleDialect, parse_entry_point};
use omena_syntax::SyntaxKind;

use crate::{
    scss_metadata::reduce_static_scss_metadata_with_context,
    value_eval::{reduce_static_scss_value, static_scss_literal_truthiness},
};

use super::lexical::{LexicalScssBindings, static_scss_metadata_exists_call_may_need_resolution};
use super::variables::{
    canonical_scss_variable_name, static_scss_binding_value, variable_name_end,
    variable_names_in_text,
};

pub(super) fn scss_header_value(
    header: &str,
    lexical_bindings: &LexicalScssBindings,
    position: usize,
) -> AbstractCssValueV0 {
    let visible_bindings = lexical_bindings.visible_at(position);
    scss_header_value_with_bindings(header, lexical_bindings, position, &visible_bindings)
}

pub(super) fn scss_header_value_with_bindings(
    header: &str,
    lexical_bindings: &LexicalScssBindings,
    position: usize,
    visible_bindings: &BTreeMap<String, AbstractCssValueV0>,
) -> AbstractCssValueV0 {
    let reduced_header = reduce_static_scss_metadata_with_context(
        header,
        |name| lexical_bindings.visible_function_metadata_exists(name, position),
        |name| lexical_bindings.visible_mixin_metadata_exists(name, position),
        |name| lexical_bindings.visible_variable_metadata_exists(name, position),
        |name| lexical_bindings.global_variable_metadata_exists(name, position),
    );
    match reduced_header {
        Some(header) => scss_header_value_from_bindings(header.as_str(), visible_bindings),
        None if static_scss_metadata_exists_call_may_need_resolution(header) => {
            AbstractCssValueV0::Top
        }
        None => scss_header_value_from_bindings(header, visible_bindings),
    }
}

pub(super) fn scss_header_value_from_bindings(
    header: &str,
    lexical_bindings: &BTreeMap<String, AbstractCssValueV0>,
) -> AbstractCssValueV0 {
    let variables = variable_names_in_text(header);
    if variables.is_empty() {
        return static_scss_header_abstract_value(header);
    }
    if let Some(value) = scss_header_value_from_binding_combinations(header, lexical_bindings) {
        return value;
    }
    if let Some(substituted) = substitute_static_scss_header_variables(header, lexical_bindings) {
        return static_scss_header_abstract_value(substituted.as_str());
    }
    variables
        .iter()
        .map(|name| {
            static_scss_binding_value(lexical_bindings, name)
                .cloned()
                .unwrap_or(AbstractCssValueV0::Top)
        })
        .fold(AbstractCssValueV0::Bottom, |acc, value| {
            join_abstract_css_values(&acc, &value)
        })
}

fn scss_header_value_from_binding_combinations(
    header: &str,
    lexical_bindings: &BTreeMap<String, AbstractCssValueV0>,
) -> Option<AbstractCssValueV0> {
    let variables = variable_names_in_text(header);
    if variables.is_empty() {
        return Some(static_scss_header_abstract_value(header));
    }
    let mut combinations = vec![BTreeMap::<String, String>::new()];
    for variable in variables {
        let values = static_scss_binding_value(lexical_bindings, variable.as_str())?;
        let values = static_scss_header_value_texts(values)?;
        if values.is_empty() {
            return None;
        }
        let mut next = Vec::new();
        for combination in combinations {
            for value in &values {
                let mut combination = combination.clone();
                combination.insert(
                    canonical_scss_variable_name(variable.as_str()),
                    value.clone(),
                );
                next.push(combination);
                if next.len() > 64 {
                    return None;
                }
            }
        }
        combinations = next;
    }
    combinations
        .into_iter()
        .map(|combination| substitute_static_scss_header_variable_combination(header, &combination))
        .collect::<Option<Vec<_>>>()
        .map(|headers| {
            headers
                .into_iter()
                .map(|header| static_scss_header_abstract_value(header.as_str()))
                .fold(AbstractCssValueV0::Bottom, |acc, value| {
                    join_abstract_css_values(&acc, &value)
                })
        })
}

fn static_scss_header_value_texts(value: &AbstractCssValueV0) -> Option<Vec<String>> {
    match value {
        AbstractCssValueV0::Exact { value, .. } | AbstractCssValueV0::Raw { value } => {
            Some(vec![value.clone()])
        }
        AbstractCssValueV0::FiniteSet { values, .. } => Some(values.clone()),
        AbstractCssValueV0::Bottom | AbstractCssValueV0::Top => None,
    }
}

fn substitute_static_scss_header_variable_combination(
    header: &str,
    bindings: &BTreeMap<String, String>,
) -> Option<String> {
    let mut output = String::with_capacity(header.len());
    let mut index = 0usize;
    while index < header.len() {
        let ch = header[index..].chars().next()?;
        if ch != '$' {
            output.push(ch);
            index += ch.len_utf8();
            continue;
        }
        let name_end = variable_name_end(header, index + ch.len_utf8());
        let name = header.get(index..name_end)?;
        let value = bindings.get(canonical_scss_variable_name(name).as_str())?;
        output.push_str(value);
        index = name_end.max(index + ch.len_utf8());
    }
    Some(output)
}

pub(super) fn static_scss_header_abstract_value(value: &str) -> AbstractCssValueV0 {
    let reduced = reduce_static_scss_value(value.to_string());
    let trimmed = reduced.trim();
    if static_scss_header_is_boolean_expression(trimmed)
        && let Some(truthy) = static_scss_literal_truthiness(trimmed)
    {
        return abstract_css_value_from_text(if truthy { "true" } else { "false" });
    }
    abstract_css_value_from_text(trimmed)
}

fn static_scss_header_is_boolean_expression(value: &str) -> bool {
    let trimmed = value.trim();
    let lower = trimmed.to_ascii_lowercase();
    if matches!(lower.as_str(), "true" | "false" | "null") {
        return true;
    }
    let parsed = parse_entry_point(trimmed, StyleDialect::Scss, ParseEntryPoint::Value);
    if !parsed.errors().is_empty() {
        return false;
    }
    let root = parsed.syntax();
    static_scss_header_boolean_expression_node(&root)
}

fn static_scss_header_boolean_expression_node(node: &SyntaxNode<SyntaxKind>) -> bool {
    match node.kind() {
        SyntaxKind::Root => node
            .children()
            .any(static_scss_header_boolean_expression_node),
        SyntaxKind::Value
        | SyntaxKind::ParenthesizedExpression
        | SyntaxKind::ScssCondition
        | SyntaxKind::LessCondition => static_scss_header_wrapped_boolean_expression(node),
        SyntaxKind::UnaryExpression => static_scss_header_unary_is_boolean(node),
        SyntaxKind::BinaryExpression => static_scss_header_binary_is_boolean(node),
        _ => false,
    }
}

fn static_scss_header_wrapped_boolean_expression(node: &SyntaxNode<SyntaxKind>) -> bool {
    if static_scss_header_node_starts_with_not_operator(node) {
        return true;
    }
    let expression_children = node
        .children()
        .filter(|child| static_scss_header_expression_node_kind(child.kind()))
        .collect::<Vec<_>>();
    match expression_children.as_slice() {
        [child] => static_scss_header_boolean_expression_node(child),
        [operator, _] if static_scss_header_node_is_not_operator(operator) => true,
        _ => false,
    }
}

fn static_scss_header_unary_is_boolean(node: &SyntaxNode<SyntaxKind>) -> bool {
    static_scss_header_first_non_trivia_token(node)
        .is_some_and(|token| static_scss_header_token_is_not_operator(&token))
}

fn static_scss_header_binary_is_boolean(node: &SyntaxNode<SyntaxKind>) -> bool {
    let children = node
        .children()
        .filter(|child| static_scss_header_expression_node_kind(child.kind()))
        .collect::<Vec<_>>();
    let [left, right] = children.as_slice() else {
        return false;
    };
    static_scss_header_binary_operator_is_boolean(node, left, right)
}

fn static_scss_header_binary_operator_is_boolean(
    node: &SyntaxNode<SyntaxKind>,
    left: &SyntaxNode<SyntaxKind>,
    right: &SyntaxNode<SyntaxKind>,
) -> bool {
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
            SyntaxKind::KeywordOr
            | SyntaxKind::KeywordAnd
            | SyntaxKind::DoubleAmpersand
            | SyntaxKind::ColumnCombinator
            | SyntaxKind::LessThan
            | SyntaxKind::GreaterThan => true,
            SyntaxKind::Ident => {
                static_scss_header_token_text(token)
                    .as_deref()
                    .is_some_and(|text| {
                        text.eq_ignore_ascii_case("or") || text.eq_ignore_ascii_case("and")
                    })
            }
            _ => false,
        },
        [first, second] if second.kind() == SyntaxKind::Equals => {
            matches!(
                first.kind(),
                SyntaxKind::LessThan | SyntaxKind::GreaterThan | SyntaxKind::Equals
            ) || (first.kind() == SyntaxKind::Delim
                && static_scss_header_token_text(first).as_deref() == Some("!"))
        }
        _ => false,
    }
}

fn static_scss_header_expression_node_kind(kind: SyntaxKind) -> bool {
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

fn static_scss_header_node_is_not_operator(node: &SyntaxNode<SyntaxKind>) -> bool {
    static_scss_header_first_non_trivia_token(node)
        .is_some_and(|token| static_scss_header_token_is_not_operator(&token))
}

fn static_scss_header_node_starts_with_not_operator(node: &SyntaxNode<SyntaxKind>) -> bool {
    let mut tokens = node
        .descendants_with_tokens()
        .filter_map(|element| element.into_token())
        .filter(|token| !token.kind().is_trivia());
    tokens
        .next()
        .is_some_and(static_scss_header_token_is_not_operator)
        && tokens.next().is_some()
}

fn static_scss_header_token_is_not_operator(token: &SyntaxToken<SyntaxKind>) -> bool {
    matches!(token.kind(), SyntaxKind::KeywordNot)
        || (token.kind() == SyntaxKind::Ident
            && static_scss_header_token_text(token)
                .as_deref()
                .is_some_and(|text| text.eq_ignore_ascii_case("not")))
}

fn static_scss_header_first_non_trivia_token(
    node: &SyntaxNode<SyntaxKind>,
) -> Option<SyntaxToken<SyntaxKind>> {
    node.descendants_with_tokens()
        .filter_map(|element| element.into_token())
        .find(|token| !token.kind().is_trivia())
        .cloned()
}

fn static_scss_header_token_text(token: &SyntaxToken<SyntaxKind>) -> Option<String> {
    if let Some(resolver) = token.resolver() {
        Some(token.resolve_text(&**resolver).to_string())
    } else {
        token.static_text().map(str::to_string)
    }
}

pub(super) fn substitute_static_scss_header_variables(
    header: &str,
    lexical_bindings: &BTreeMap<String, AbstractCssValueV0>,
) -> Option<String> {
    let mut output = String::with_capacity(header.len());
    let mut index = 0usize;
    while index < header.len() {
        let ch = header[index..].chars().next()?;
        if ch != '$' {
            output.push(ch);
            index += ch.len_utf8();
            continue;
        }
        let name_end = variable_name_end(header, index + ch.len_utf8());
        let name = header.get(index..name_end)?;
        let value = static_scss_binding_value(lexical_bindings, name)
            .and_then(single_static_scss_header_value_text)?;
        output.push_str(value);
        index = name_end.max(index + ch.len_utf8());
    }
    Some(output)
}

pub(super) fn single_static_scss_header_value_text(value: &AbstractCssValueV0) -> Option<&str> {
    match value {
        AbstractCssValueV0::Exact { value, .. } | AbstractCssValueV0::Raw { value } => {
            Some(value.as_str())
        }
        AbstractCssValueV0::FiniteSet { values, .. } if values.len() == 1 => {
            values.first().map(String::as_str)
        }
        AbstractCssValueV0::Bottom
        | AbstractCssValueV0::Top
        | AbstractCssValueV0::FiniteSet { .. } => None,
    }
}

#[cfg(test)]
mod tests {
    use omena_abstract_value::abstract_css_value_from_text;

    use super::*;

    #[test]
    fn static_scss_header_boolean_detection_uses_cst_expression_shape() {
        assert_eq!(
            static_scss_header_abstract_value("red"),
            abstract_css_value_from_text("red")
        );
        assert_eq!(
            static_scss_header_abstract_value("true and false"),
            abstract_css_value_from_text("false")
        );
        assert_eq!(
            static_scss_header_abstract_value("not false"),
            abstract_css_value_from_text("true")
        );
        assert_eq!(
            static_scss_header_abstract_value("1px < 2px"),
            abstract_css_value_from_text("true")
        );
    }
}
