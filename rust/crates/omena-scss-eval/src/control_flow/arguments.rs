use omena_abstract_value::{AbstractCssValueV0, abstract_css_value_from_text};

use crate::value_eval::{reduce_static_scss_value, static_scss_bang_usage_is_comparison_only};

use super::variable_name_end;

pub(super) fn split_scss_call_arguments(arguments: &str) -> Option<Vec<String>> {
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
                let value = arguments.get(cursor..index)?.trim();
                if !scss_call_argument_is_safe(value) {
                    return None;
                }
                values.push(value.to_string());
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
    if !scss_call_argument_is_safe(value) {
        return None;
    }
    values.push(value.to_string());
    Some(values)
}

pub(super) fn scss_named_value_from_text(value: &str) -> Option<Option<(String, String)>> {
    let colon_index = scss_top_level_colon_index(value)?;
    let Some(colon_index) = colon_index else {
        return Some(None);
    };
    let name = value.get(..colon_index)?.trim();
    let value = value.get(colon_index + ':'.len_utf8()..)?.trim();
    if !name.starts_with('$') || value.is_empty() || !scss_call_argument_is_safe(value) {
        return None;
    }
    let name_end = variable_name_end(name, '$'.len_utf8());
    (name_end == name.len()).then(|| Some((name.to_string(), value.to_string())))
}

fn scss_top_level_colon_index(value: &str) -> Option<Option<usize>> {
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

fn scss_call_argument_is_safe(value: &str) -> bool {
    !value.is_empty()
        && !value.contains("...")
        && !value.chars().any(|ch| matches!(ch, '{' | '}' | ';'))
        && static_scss_bang_usage_is_comparison_only(value)
}

pub(super) fn static_scss_argument_abstract_value(value: &str) -> AbstractCssValueV0 {
    abstract_css_value_from_text(reduce_static_scss_value(value.to_string()).as_str())
}
