use std::collections::BTreeMap;

use omena_abstract_value::{
    AbstractCssValueV0, abstract_css_value_from_text, join_abstract_css_values,
};

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
        AbstractCssValueV0::Exact { value } | AbstractCssValueV0::Raw { value } => {
            Some(vec![value.clone()])
        }
        AbstractCssValueV0::FiniteSet { values } => Some(values.clone()),
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
    lower == "true"
        || lower == "false"
        || lower == "null"
        || lower.starts_with("not ")
        || lower.contains(" and ")
        || lower.contains(" or ")
        || ["==", "!=", "<=", ">=", "<", ">"]
            .iter()
            .any(|operator| trimmed.contains(operator))
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
        AbstractCssValueV0::Exact { value } | AbstractCssValueV0::Raw { value } => {
            Some(value.as_str())
        }
        AbstractCssValueV0::FiniteSet { values } if values.len() == 1 => {
            values.first().map(String::as_str)
        }
        AbstractCssValueV0::Bottom
        | AbstractCssValueV0::Top
        | AbstractCssValueV0::FiniteSet { .. } => None,
    }
}
