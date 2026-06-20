use omena_value_lattice::parse_whole_function_value_arguments;

use super::{
    parse_static_scss_list_index, static_scss_quoted_value_end,
    static_scss_top_level_separator_index, strip_static_scss_quotes,
};

pub(super) fn parse_static_scss_quote_value(value: &str) -> Option<String> {
    parse_static_scss_quote_value_with_name(value, "quote")
}

pub(super) fn parse_static_scss_string_quote_value(value: &str) -> Option<String> {
    parse_static_scss_quote_value_with_name(value, "string.quote")
}

pub(super) fn parse_static_scss_unquote_value(value: &str) -> Option<String> {
    parse_static_scss_unquote_value_with_name(value, "unquote")
}

pub(super) fn parse_static_scss_string_unquote_value(value: &str) -> Option<String> {
    parse_static_scss_unquote_value_with_name(value, "string.unquote")
}

pub(super) fn parse_static_scss_str_length_value(value: &str) -> Option<String> {
    parse_static_scss_string_length_value_with_name(value, "str-length")
}

pub(super) fn parse_static_scss_string_length_value(value: &str) -> Option<String> {
    parse_static_scss_string_length_value_with_name(value, "string.length")
}

pub(super) fn parse_static_scss_str_index_value(value: &str) -> Option<String> {
    parse_static_scss_string_index_value_with_name(value, "str-index")
}

pub(super) fn parse_static_scss_string_index_value(value: &str) -> Option<String> {
    parse_static_scss_string_index_value_with_name(value, "string.index")
}

pub(super) fn parse_static_scss_str_insert_value(value: &str) -> Option<String> {
    parse_static_scss_string_insert_value_with_name(value, "str-insert")
}

pub(super) fn parse_static_scss_string_insert_value(value: &str) -> Option<String> {
    parse_static_scss_string_insert_value_with_name(value, "string.insert")
}

pub(super) fn parse_static_scss_str_slice_value(value: &str) -> Option<String> {
    parse_static_scss_string_slice_value_with_name(value, "str-slice")
}

pub(super) fn parse_static_scss_string_slice_value(value: &str) -> Option<String> {
    parse_static_scss_string_slice_value_with_name(value, "string.slice")
}

pub(super) fn parse_static_scss_to_upper_case_value(value: &str) -> Option<String> {
    parse_static_scss_string_case_value_with_name(
        value,
        "to-upper-case",
        StaticScssStringCase::Upper,
    )
}

pub(super) fn parse_static_scss_string_to_upper_case_value(value: &str) -> Option<String> {
    parse_static_scss_string_case_value_with_name(
        value,
        "string.to-upper-case",
        StaticScssStringCase::Upper,
    )
}

pub(super) fn parse_static_scss_to_lower_case_value(value: &str) -> Option<String> {
    parse_static_scss_string_case_value_with_name(
        value,
        "to-lower-case",
        StaticScssStringCase::Lower,
    )
}

pub(super) fn parse_static_scss_string_to_lower_case_value(value: &str) -> Option<String> {
    parse_static_scss_string_case_value_with_name(
        value,
        "string.to-lower-case",
        StaticScssStringCase::Lower,
    )
}

fn parse_static_scss_quote_value_with_name(value: &str, function_name: &str) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, function_name)?;
    let [string] = arguments.as_slice() else {
        return None;
    };
    static_scss_quote_string(parse_static_scss_string_argument(string)?.text.as_str())
}

fn parse_static_scss_unquote_value_with_name(value: &str, function_name: &str) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, function_name)?;
    let [string] = arguments.as_slice() else {
        return None;
    };
    Some(parse_static_scss_string_argument(string)?.text)
}

fn parse_static_scss_string_length_value_with_name(
    value: &str,
    function_name: &str,
) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, function_name)?;
    let [string] = arguments.as_slice() else {
        return None;
    };
    Some(
        parse_static_scss_string_argument(string)?
            .text
            .chars()
            .count()
            .to_string(),
    )
}

fn parse_static_scss_string_index_value_with_name(
    value: &str,
    function_name: &str,
) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, function_name)?;
    let [string, substring] = arguments.as_slice() else {
        return None;
    };
    let string = parse_static_scss_string_argument(string)?;
    let substring = parse_static_scss_string_argument(substring)?;
    if substring.text.is_empty() {
        return None;
    }
    Some(
        string
            .text
            .find(substring.text.as_str())
            .and_then(|byte_index| {
                string
                    .text
                    .get(..byte_index)
                    .map(|prefix| prefix.chars().count() + 1)
            })
            .map(|index| index.to_string())
            .unwrap_or_else(|| "null".to_string()),
    )
}

fn parse_static_scss_string_insert_value_with_name(
    value: &str,
    function_name: &str,
) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, function_name)?;
    let [string, insert, index] = arguments.as_slice() else {
        return None;
    };
    let string = parse_static_scss_string_argument(string)?;
    let insert = parse_static_scss_string_argument(insert)?;
    let index = parse_static_scss_list_index(index)?;
    let chars = string.text.chars().collect::<Vec<_>>();
    let offset = static_scss_string_insert_offset(index, chars.len())?;
    let output = chars
        .iter()
        .take(offset)
        .copied()
        .chain(insert.text.chars())
        .chain(chars.iter().skip(offset).copied())
        .collect::<String>();
    static_scss_render_string_value(output.as_str(), string.quoted)
}

fn parse_static_scss_string_slice_value_with_name(
    value: &str,
    function_name: &str,
) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, function_name)?;
    let [string, start, end @ ..] = arguments.as_slice() else {
        return None;
    };
    if end.len() > 1 {
        return None;
    }
    let string = parse_static_scss_string_argument(string)?;
    let start = parse_static_scss_list_index(start)?;
    let end = match end {
        [] => -1,
        [end] => parse_static_scss_list_index(end)?,
        _ => return None,
    };
    let chars = string.text.chars().collect::<Vec<_>>();
    let start_offset = static_scss_string_slice_start_offset(start, chars.len())?;
    let end_offset = static_scss_string_slice_end_offset(end, chars.len())?;
    let output = if start_offset >= end_offset {
        String::new()
    } else {
        chars[start_offset..end_offset].iter().collect::<String>()
    };
    static_scss_render_string_value(output.as_str(), string.quoted)
}

fn parse_static_scss_string_case_value_with_name(
    value: &str,
    function_name: &str,
    case: StaticScssStringCase,
) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, function_name)?;
    let [string] = arguments.as_slice() else {
        return None;
    };
    let string = parse_static_scss_string_argument(string)?;
    let output = match case {
        StaticScssStringCase::Upper => string.text.to_ascii_uppercase(),
        StaticScssStringCase::Lower => string.text.to_ascii_lowercase(),
    };
    static_scss_render_string_value(output.as_str(), string.quoted)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct StaticScssStringValue {
    pub(super) text: String,
    pub(super) quoted: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StaticScssStringCase {
    Upper,
    Lower,
}

pub(super) fn parse_static_scss_string_argument(value: &str) -> Option<StaticScssStringValue> {
    let value = value.trim();
    if value.contains('$') {
        return None;
    }
    if let Some(text) = static_scss_quoted_string_text(value) {
        return Some(StaticScssStringValue {
            text: text.to_string(),
            quoted: true,
        });
    }
    if value.is_empty()
        || value.contains('(')
        || value.contains(')')
        || value.contains('[')
        || value.contains(']')
        || static_scss_top_level_separator_index(value, ',')?.is_some()
    {
        return None;
    }
    Some(StaticScssStringValue {
        text: value.to_string(),
        quoted: false,
    })
}

pub(super) fn static_scss_quoted_string_text(value: &str) -> Option<&str> {
    let quote = value.chars().next()?;
    if !matches!(quote, '"' | '\'') || static_scss_quoted_value_end(value, 0, quote)? != value.len()
    {
        return None;
    }
    strip_static_scss_quotes(value)
}

fn static_scss_render_string_value(value: &str, quoted: bool) -> Option<String> {
    if quoted {
        static_scss_quote_string(value)
    } else {
        Some(value.to_string())
    }
}

pub(super) fn static_scss_quote_string(value: &str) -> Option<String> {
    let mut output = String::with_capacity(value.len() + 2);
    output.push('"');
    for ch in value.chars() {
        match ch {
            '"' | '\\' => {
                output.push('\\');
                output.push(ch);
            }
            _ if ch.is_control() => return None,
            _ => output.push(ch),
        }
    }
    output.push('"');
    Some(output)
}

fn static_scss_string_insert_offset(index: isize, len: usize) -> Option<usize> {
    let len = isize::try_from(len).ok()?;
    Some(if index > 0 {
        (index - 1).clamp(0, len) as usize
    } else {
        (len + index + 1).clamp(0, len) as usize
    })
}

fn static_scss_string_slice_start_offset(index: isize, len: usize) -> Option<usize> {
    let len = isize::try_from(len).ok()?;
    Some(if index > 0 {
        (index - 1).clamp(0, len) as usize
    } else {
        (len + index).clamp(0, len) as usize
    })
}

fn static_scss_string_slice_end_offset(index: isize, len: usize) -> Option<usize> {
    let len = isize::try_from(len).ok()?;
    Some(if index > 0 {
        index.clamp(0, len) as usize
    } else {
        (len + index + 1).clamp(0, len) as usize
    })
}
