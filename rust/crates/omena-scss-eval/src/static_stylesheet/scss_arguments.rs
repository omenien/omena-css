use std::collections::BTreeSet;

use omena_parser::LexedToken;

use crate::value_eval::static_scss_bang_usage_is_comparison_only;

use super::{
    canonical_static_scss_variable_name,
    model::{StaticScssFunctionArgument, StaticScssFunctionParameter},
    static_stylesheet_token_start, static_stylesheet_variable_name_is_safe,
};

pub(super) fn split_static_scss_function_arguments(
    arguments: &str,
) -> Option<Vec<StaticScssFunctionArgument>> {
    let arguments = arguments.trim();
    if arguments.is_empty() {
        return Some(Vec::new());
    }

    let mut values = Vec::new();
    let mut cursor = 0usize;
    let mut index = 0usize;
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;
    let mut quote: Option<char> = None;
    while index < arguments.len() {
        let ch = arguments[index..].chars().next()?;
        if let Some(quote_ch) = quote {
            index += ch.len_utf8();
            if ch == '\\' {
                if let Some(escaped) = arguments[index..].chars().next() {
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
            ')' => paren_depth = paren_depth.checked_sub(1)?,
            '[' => bracket_depth += 1,
            ']' => bracket_depth = bracket_depth.checked_sub(1)?,
            ',' if paren_depth == 0 && bracket_depth == 0 => {
                values.push(parse_static_scss_function_argument(
                    arguments.get(cursor..index)?.trim(),
                )?);
                cursor = index + ch.len_utf8();
            }
            _ => {}
        }
        index += ch.len_utf8();
    }

    if quote.is_some() || paren_depth != 0 || bracket_depth != 0 {
        return None;
    }
    let value = arguments.get(cursor..)?.trim();
    values.push(parse_static_scss_function_argument(value)?);
    Some(values)
}

fn parse_static_scss_function_argument(value: &str) -> Option<StaticScssFunctionArgument> {
    let value = value.trim();
    if value.is_empty() {
        return None;
    }
    if let Some((name, argument_value)) = split_static_scss_named_function_argument(value)? {
        if !static_stylesheet_variable_name_is_safe(name.as_str())
            || !static_scss_function_argument_is_safe(argument_value.as_str())
        {
            return None;
        }
        return Some(StaticScssFunctionArgument {
            name: Some(canonical_static_scss_variable_name(name.as_str())),
            value: argument_value,
        });
    }
    if !static_scss_function_argument_is_safe(value) {
        return None;
    }
    Some(StaticScssFunctionArgument {
        name: None,
        value: value.to_string(),
    })
}

pub(super) fn collect_static_scss_function_parameters(
    source: &str,
    tokens: &[LexedToken],
    start: usize,
    end: usize,
) -> Option<Vec<StaticScssFunctionParameter>> {
    let parameter_start = tokens.get(start).map(static_stylesheet_token_start)?;
    let parameter_end = tokens
        .get(end)
        .map(static_stylesheet_token_start)
        .unwrap_or(parameter_start);
    let parameter_text = source.get(parameter_start..parameter_end)?.trim();
    if parameter_text.is_empty() {
        return Some(Vec::new());
    }

    let mut parameters = Vec::new();
    let mut names = BTreeSet::new();
    let mut saw_default = false;
    for argument in split_static_scss_function_arguments(parameter_text)? {
        let parameter = parse_static_scss_function_parameter(argument)?;
        if parameter.default_value.is_some() {
            saw_default = true;
        } else if saw_default {
            return None;
        }
        if parameter.pattern_value.is_none() && !names.insert(parameter.name.clone()) {
            return None;
        }
        parameters.push(parameter);
    }
    Some(parameters)
}

pub(super) fn collect_static_scss_content_parameters(
    source: &str,
    tokens: &[LexedToken],
    start: usize,
    end: usize,
) -> Option<Vec<String>> {
    collect_static_scss_function_parameters(source, tokens, start, end)?
        .into_iter()
        .map(|parameter| {
            (parameter.default_value.is_none()
                && !parameter.variadic
                && parameter.pattern_value.is_none())
            .then_some(parameter.name)
        })
        .collect()
}

fn parse_static_scss_function_parameter(
    argument: StaticScssFunctionArgument,
) -> Option<StaticScssFunctionParameter> {
    if let Some(name) = argument.name {
        return Some(StaticScssFunctionParameter {
            name,
            default_value: Some(argument.value),
            variadic: false,
            pattern_value: None,
        });
    }

    let name = argument.value.trim();
    let name = name.strip_prefix('$')?.trim();
    if !static_stylesheet_variable_name_is_safe(name) {
        return None;
    }
    Some(StaticScssFunctionParameter {
        name: canonical_static_scss_variable_name(name),
        default_value: None,
        variadic: false,
        pattern_value: None,
    })
}

fn split_static_scss_named_function_argument(value: &str) -> Option<Option<(String, String)>> {
    let colon_index = static_scss_top_level_colon_index(value)?;
    let Some(colon_index) = colon_index else {
        return Some(None);
    };
    let name = value.get(..colon_index)?.trim();
    let argument_value = value.get(colon_index + ':'.len_utf8()..)?.trim();
    let name = name.strip_prefix('$')?.trim();
    (!name.is_empty() && !argument_value.is_empty())
        .then(|| Some((name.to_string(), argument_value.to_string())))
}

pub(super) fn static_scss_top_level_colon_index(value: &str) -> Option<Option<usize>> {
    let mut index = 0usize;
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;
    let mut quote: Option<char> = None;
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
            ')' => paren_depth = paren_depth.checked_sub(1)?,
            '[' => bracket_depth += 1,
            ']' => bracket_depth = bracket_depth.checked_sub(1)?,
            ':' if paren_depth == 0 && bracket_depth == 0 => return Some(Some(index)),
            _ => {}
        }
        index += ch.len_utf8();
    }
    (quote.is_none() && paren_depth == 0 && bracket_depth == 0).then_some(None)
}

fn static_scss_function_argument_is_safe(value: &str) -> bool {
    !value.is_empty()
        && !value.contains("...")
        && !value.chars().any(|ch| matches!(ch, '{' | '}' | ';' | ':'))
        && static_scss_bang_usage_is_comparison_only(value)
}
