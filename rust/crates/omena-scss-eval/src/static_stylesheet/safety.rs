use crate::value_eval::static_scss_bang_usage_is_comparison_only;

use super::less_strings::preserve_static_less_dynamic_escaped_string_value;

pub(super) fn static_stylesheet_less_declaration_value_is_removal_safe(value: &str) -> bool {
    if preserve_static_less_dynamic_escaped_string_value(value).is_some() {
        return true;
    }
    !static_stylesheet_value_contains_unquoted_char(value, |ch| matches!(ch, '{' | '}' | ';' | '!'))
}

pub(super) fn static_stylesheet_scss_declaration_value_is_removal_safe(value: &str) -> bool {
    !static_stylesheet_value_contains_unquoted_char(value, |ch| matches!(ch, '{' | '}' | ';'))
        && static_scss_bang_usage_is_comparison_only(value)
}

pub(super) fn static_stylesheet_property_name_is_safe(name: &str) -> bool {
    !name.is_empty()
        && name
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-'))
}

pub(super) fn static_stylesheet_selector_name_part_is_safe(name: &str) -> bool {
    static_stylesheet_property_name_is_safe(name)
}

pub(super) fn static_stylesheet_property_value_is_removal_safe(value: &str) -> bool {
    !static_stylesheet_value_contains_unquoted_char(value, |ch| matches!(ch, '{' | '}' | ';' | '!'))
}

pub(super) fn static_stylesheet_literal_value_is_safe(value: &str) -> bool {
    let value = value.trim();
    !value.is_empty()
        && !value
            .chars()
            .any(|ch| matches!(ch, '{' | '}' | ';' | '$' | '@'))
        && static_scss_bang_usage_is_comparison_only(value)
}

pub(super) fn static_stylesheet_variable_name_is_safe(name: &str) -> bool {
    !name.is_empty()
        && name
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-')
}

pub(super) fn static_scss_callable_name_is_safe(name: &str) -> bool {
    static_stylesheet_variable_name_is_safe(name)
}

pub(super) fn static_less_mixin_name_part_is_safe(name: &str) -> bool {
    static_stylesheet_property_name_is_safe(name)
}

pub(super) fn static_less_mixin_hash_name_is_safe(name: &str) -> bool {
    name.strip_prefix('#')
        .is_some_and(static_stylesheet_property_name_is_safe)
}

pub(super) fn static_less_variable_name_is_safe(name: &str) -> bool {
    name.strip_prefix('@')
        .is_some_and(static_stylesheet_variable_name_is_safe)
}

pub(super) fn static_less_mixin_argument_value_is_safe(value: &str) -> bool {
    !value.is_empty()
        && !value.contains("...")
        && !value.chars().any(|ch| matches!(ch, '{' | '}' | ';'))
}

pub(super) fn static_scss_mixin_body_is_static_declaration_subset(body: &str) -> bool {
    let lower = body.to_ascii_lowercase();
    !body.chars().any(|ch| matches!(ch, '{' | '}'))
        && !lower.contains("@content")
        && !lower.contains("@mixin")
        && !lower.contains("@function")
        && !lower.contains("@return")
        && !lower.contains("@if")
        && !lower.contains("@for")
        && !lower.contains("@each")
        && !lower.contains("@while")
}

pub(super) fn static_less_mixin_body_is_static_declaration_subset(body: &str) -> bool {
    let lower = body.to_ascii_lowercase();
    !body.chars().any(|ch| matches!(ch, '{' | '}'))
        && !lower.contains("when")
        && !lower.contains(":extend")
        && !lower.contains("@plugin")
        && !lower.contains("@import")
}

pub(super) fn static_stylesheet_composite_value_is_safe(value: &str) -> bool {
    let value = value.trim();
    !value.is_empty()
        && !value.chars().any(|ch| matches!(ch, '{' | '}' | ';'))
        && static_scss_bang_usage_is_comparison_only(value)
}

fn static_stylesheet_value_contains_unquoted_char(
    value: &str,
    predicate: impl Fn(char) -> bool,
) -> bool {
    let mut index = 0usize;
    let mut quote: Option<char> = None;
    while index < value.len() {
        let Some(ch) = value[index..].chars().next() else {
            break;
        };
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
        if predicate(ch) {
            return true;
        }
        index += ch.len_utf8();
    }
    false
}
