use crate::value_eval::static_scss_bang_usage_is_comparison_only;

use super::{
    StaticScssFunctionArgument, canonical_static_scss_variable_name,
    static_stylesheet_variable_name_is_safe,
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
