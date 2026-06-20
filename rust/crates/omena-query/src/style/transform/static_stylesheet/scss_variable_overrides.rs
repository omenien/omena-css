use super::super::super::stylesheet_evaluation::canonical_static_scss_variable_name;
use super::super::super::stylesheet_evaluation::derive_static_scss_stylesheet_module_configurable_variable_names;
use super::scss_module_rules::static_scss_identifier_char;
use omena_syntax::SyntaxKind;
use std::{borrow::Cow, collections::BTreeMap};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct StaticScssModuleVariableOverride {
    pub(super) value: String,
    pub(super) is_default: bool,
}

pub(super) fn parse_static_scss_use_variable_override_list(
    content: &str,
) -> BTreeMap<String, String> {
    parse_static_scss_variable_override_list(content, false)
        .into_iter()
        .map(|(name, override_entry)| (name, override_entry.value))
        .collect()
}

pub(super) fn parse_static_scss_forward_variable_override_list(
    content: &str,
) -> BTreeMap<String, StaticScssModuleVariableOverride> {
    parse_static_scss_variable_override_list(content, true)
}

pub(super) fn apply_static_scss_module_variable_overrides<'a>(
    style_source: &'a str,
    variable_overrides: &BTreeMap<String, String>,
) -> Cow<'a, str> {
    if variable_overrides.is_empty() {
        return Cow::Borrowed(style_source);
    }
    let configurable_names =
        derive_static_scss_stylesheet_module_configurable_variable_names(style_source);
    if !variable_overrides
        .keys()
        .all(|name| configurable_names.contains(name))
    {
        return Cow::Borrowed(style_source);
    }

    let mut source = String::new();
    for (name, value) in variable_overrides {
        source.push('$');
        source.push_str(name);
        source.push_str(": ");
        source.push_str(value);
        source.push_str("; ");
    }
    source.push_str(style_source);
    Cow::Owned(source)
}

fn parse_static_scss_variable_override_list(
    content: &str,
    allow_default_flag: bool,
) -> BTreeMap<String, StaticScssModuleVariableOverride> {
    let mut overrides = BTreeMap::new();
    for entry in split_static_scss_top_level_commas(content) {
        if entry.trim().is_empty() {
            continue;
        }
        let Some((name, value)) =
            parse_static_scss_variable_override(entry.trim(), allow_default_flag)
        else {
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

fn parse_static_scss_variable_override(
    entry: &str,
    allow_default_flag: bool,
) -> Option<(String, StaticScssModuleVariableOverride)> {
    let colon_index = static_scss_top_level_colon_index(entry)?;
    let name = entry[..colon_index].trim().strip_prefix('$')?;
    if name.is_empty() || !name.chars().all(static_scss_identifier_char) {
        return None;
    }
    let (value, is_default) = split_static_scss_forward_default_flag(
        entry[colon_index + 1..].trim(),
        allow_default_flag,
    )?;
    if !static_scss_use_variable_override_value_is_safe(value) {
        return None;
    }
    Some((
        canonical_static_scss_variable_name(name),
        StaticScssModuleVariableOverride {
            value: value.to_string(),
            is_default,
        },
    ))
}

fn split_static_scss_forward_default_flag(
    value: &str,
    allow_default_flag: bool,
) -> Option<(&str, bool)> {
    if !allow_default_flag {
        return Some((value, false));
    }
    let lower = value.to_ascii_lowercase();
    let Some(before_default) = lower.strip_suffix("!default") else {
        return Some((value, false));
    };
    let value_before_default = &value[..before_default.len()];
    let stripped = value_before_default.trim_end();
    (!stripped.is_empty()).then_some((stripped, true))
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
