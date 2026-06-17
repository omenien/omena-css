pub(crate) use omena_value_lattice::{
    StaticCssFunctionSpec, matching_function_call_end, parse_whole_function_value_arguments,
    parse_whole_function_value_inner,
    split_top_level_value_arguments_owned as split_top_level_value_arguments,
    split_top_level_whitespace_value_components_owned as split_top_level_whitespace_value_components,
    substitute_static_css_function_references_in_value,
    substitute_static_css_function_references_in_value_until_stable,
};

pub(crate) fn matching_function_end(text: &str, open_paren_index: usize) -> Option<usize> {
    let mut index = open_paren_index;
    let mut depth = 0usize;
    let mut quote: Option<char> = None;

    while index < text.len() {
        let ch = text[index..].chars().next()?;
        if let Some(quote_ch) = quote {
            index += ch.len_utf8();
            if ch == '\\' {
                if let Some(escaped) = text[index..].chars().next() {
                    index += escaped.len_utf8();
                }
            } else if ch == quote_ch {
                quote = None;
            }
            continue;
        }

        match ch {
            '"' | '\'' => {
                quote = Some(ch);
                index += ch.len_utf8();
            }
            '(' => {
                depth += 1;
                index += ch.len_utf8();
            }
            ')' => {
                depth = depth.checked_sub(1)?;
                index += ch.len_utf8();
                if depth == 0 {
                    return Some(index);
                }
            }
            _ => index += ch.len_utf8(),
        }
    }

    None
}

pub(crate) fn compact_adjacent_css_function_separators(value: &str) -> String {
    let mut output = String::with_capacity(value.len());
    let mut index = 0usize;
    let mut depth = 0usize;

    while index < value.len() {
        let Some(ch) = value[index..].chars().next() else {
            break;
        };
        if ch.is_ascii_whitespace()
            && depth == 0
            && output.ends_with(')')
            && next_css_function_component_starts(value, index)
        {
            while index < value.len() {
                let Some(whitespace) = value[index..].chars().next() else {
                    break;
                };
                if !whitespace.is_ascii_whitespace() {
                    break;
                }
                index += whitespace.len_utf8();
            }
            continue;
        }

        match ch {
            '(' => depth += 1,
            ')' => depth = depth.saturating_sub(1),
            _ => {}
        }
        output.push(ch);
        index += ch.len_utf8();
    }

    output
}

fn next_css_function_component_starts(value: &str, index: usize) -> bool {
    let mut cursor = index;
    while cursor < value.len() {
        let Some(ch) = value[cursor..].chars().next() else {
            return false;
        };
        if !ch.is_ascii_whitespace() {
            break;
        }
        cursor += ch.len_utf8();
    }
    let name_start = cursor;
    while cursor < value.len() {
        let Some(ch) = value[cursor..].chars().next() else {
            return false;
        };
        if !(ch.is_ascii_alphabetic() || ch == '-') {
            break;
        }
        cursor += ch.len_utf8();
    }
    cursor > name_start && value[cursor..].starts_with('(')
}

pub(crate) fn static_css_string_value(value: &str) -> Option<String> {
    let value = value.trim();
    if value.len() < 2 {
        return None;
    }
    let quote = value.as_bytes()[0];
    if !matches!(quote, b'"' | b'\'') || value.as_bytes().last().copied() != Some(quote) {
        return None;
    }
    let inner = &value[1..value.len() - 1];
    if inner.is_empty() || inner.contains(['\\', '\n', '\r', '\x0c']) {
        return None;
    }
    Some(inner.to_string())
}
