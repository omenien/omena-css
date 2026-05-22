use super::super::super::stylesheet_evaluation::canonical_static_scss_variable_name;
use super::static_scss_identifier_char;
use omena_syntax::SyntaxKind;
use std::collections::BTreeMap;

pub(super) fn parse_static_scss_use_variable_override_list(
    content: &str,
) -> BTreeMap<String, String> {
    let mut overrides = BTreeMap::new();
    for entry in split_static_scss_top_level_commas(content) {
        if entry.trim().is_empty() {
            continue;
        }
        let Some((name, value)) = parse_static_scss_use_variable_override(entry.trim()) else {
            return BTreeMap::new();
        };
        overrides.insert(name, value);
    }
    overrides
}

pub(super) fn static_scss_matching_right_paren(
    tokens: &[omena_parser::LexedToken],
    left_paren_index: usize,
) -> Option<usize> {
    let mut depth = 0usize;
    for (index, token) in tokens.iter().enumerate().skip(left_paren_index) {
        match token.kind {
            SyntaxKind::LeftParen => depth += 1,
            SyntaxKind::RightParen => {
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

fn parse_static_scss_use_variable_override(entry: &str) -> Option<(String, String)> {
    let colon_index = static_scss_top_level_colon_index(entry)?;
    let name = entry[..colon_index].trim().strip_prefix('$')?;
    if name.is_empty() || !name.chars().all(static_scss_identifier_char) {
        return None;
    }
    let value = entry[colon_index + 1..].trim();
    if !static_scss_use_variable_override_value_is_safe(value) {
        return None;
    }
    Some((canonical_static_scss_variable_name(name), value.to_string()))
}

fn split_static_scss_top_level_commas(content: &str) -> Vec<&str> {
    let mut entries = Vec::new();
    let mut start = 0usize;
    let mut delimiter_stack = Vec::<char>::new();
    let mut quote = None;
    let mut escaped = false;

    for (index, ch) in content.char_indices() {
        if let Some(quote_ch) = quote {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == quote_ch {
                quote = None;
            }
            continue;
        }

        match ch {
            '"' | '\'' => quote = Some(ch),
            '(' | '[' => delimiter_stack.push(ch),
            ')' if delimiter_stack.last() == Some(&'(') => {
                delimiter_stack.pop();
            }
            ']' if delimiter_stack.last() == Some(&'[') => {
                delimiter_stack.pop();
            }
            ',' if delimiter_stack.is_empty() => {
                entries.push(&content[start..index]);
                start = index + ch.len_utf8();
            }
            _ => {}
        }
    }
    entries.push(&content[start..]);
    entries
}

fn static_scss_top_level_colon_index(content: &str) -> Option<usize> {
    let mut delimiter_stack = Vec::<char>::new();
    let mut quote = None;
    let mut escaped = false;

    for (index, ch) in content.char_indices() {
        if let Some(quote_ch) = quote {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == quote_ch {
                quote = None;
            }
            continue;
        }

        match ch {
            '"' | '\'' => quote = Some(ch),
            '(' | '[' => delimiter_stack.push(ch),
            ')' if delimiter_stack.last() == Some(&'(') => {
                delimiter_stack.pop();
            }
            ']' if delimiter_stack.last() == Some(&'[') => {
                delimiter_stack.pop();
            }
            ':' if delimiter_stack.is_empty() => return Some(index),
            _ => {}
        }
    }
    None
}

fn static_scss_use_variable_override_value_is_safe(value: &str) -> bool {
    !value.is_empty()
        && !value
            .chars()
            .any(|ch| matches!(ch, ';' | '{' | '}' | '!' | '$' | '@'))
}
