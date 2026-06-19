use super::{
    StaticStylesheetVariableKind, static_stylesheet_property_name_is_safe,
    static_stylesheet_variable_name_is_safe,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct StaticStylesheetVariableReference {
    pub(super) name: String,
    pub(super) start: usize,
    pub(super) end: usize,
}

pub(super) fn collect_static_stylesheet_variable_references(
    value: &str,
    variable_kind: StaticStylesheetVariableKind,
) -> Option<Vec<StaticStylesheetVariableReference>> {
    collect_static_stylesheet_variable_references_with_options(value, variable_kind, false, false)
}

pub(super) fn collect_static_less_property_variable_references(
    value: &str,
) -> Option<Vec<StaticStylesheetVariableReference>> {
    let mut references = Vec::new();
    let mut index = 0usize;
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
        if ch != '$' {
            index += ch.len_utf8();
            continue;
        }

        let name_start = index + ch.len_utf8();
        let name_end = static_stylesheet_variable_name_end(value, name_start);
        if name_end == name_start {
            return None;
        }
        let bare_name = &value[name_start..name_end];
        if !static_stylesheet_property_name_is_safe(bare_name) {
            return None;
        }
        references.push(StaticStylesheetVariableReference {
            name: value[index..name_end].to_string(),
            start: index,
            end: name_end,
        });
        index = name_end;
    }

    Some(references)
}

pub(super) fn collect_static_stylesheet_variable_references_with_options(
    value: &str,
    variable_kind: StaticStylesheetVariableKind,
    allow_scss_include_at_keyword: bool,
    allow_less_property_variables: bool,
) -> Option<Vec<StaticStylesheetVariableReference>> {
    let prefix = variable_kind.reference_prefix();
    let other_prefix = match variable_kind {
        StaticStylesheetVariableKind::Scss => '@',
        StaticStylesheetVariableKind::Less => '$',
    };
    let mut references = Vec::new();
    let mut index = 0usize;
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
        if ch == other_prefix {
            if allow_less_property_variables && variable_kind == StaticStylesheetVariableKind::Less
            {
                let name_start = index + ch.len_utf8();
                let name_end = static_stylesheet_variable_name_end(value, name_start);
                if name_end == name_start {
                    return None;
                }
                let bare_name = &value[name_start..name_end];
                if !static_stylesheet_property_name_is_safe(bare_name) {
                    return None;
                }
                index = name_end;
                continue;
            }
            if allow_scss_include_at_keyword
                && variable_kind == StaticStylesheetVariableKind::Scss
                && static_scss_at_keyword_prefix_is_include(value, index)
            {
                index += "@include".len();
                continue;
            }
            return None;
        }
        if ch != prefix {
            index += ch.len_utf8();
            continue;
        }

        let name_start = index + ch.len_utf8();
        let name_end = static_stylesheet_variable_name_end(value, name_start);
        if name_end == name_start {
            return None;
        }
        let bare_name = &value[name_start..name_end];
        if !static_stylesheet_variable_name_is_safe(bare_name) {
            return None;
        }
        if static_stylesheet_variable_reference_is_named_argument_label(value, index, name_end) {
            index = name_end;
            continue;
        }
        if variable_kind == StaticStylesheetVariableKind::Scss
            && static_stylesheet_position_is_scss_module_member_reference(value, index)
        {
            index = name_end;
            continue;
        }
        references.push(StaticStylesheetVariableReference {
            name: value[index..name_end].to_string(),
            start: index,
            end: name_end,
        });
        index = name_end;
    }

    Some(references)
}

pub(super) fn static_stylesheet_position_is_scss_module_member_reference(
    value: &str,
    start: usize,
) -> bool {
    value
        .get(..start)
        .and_then(|prefix| prefix.chars().next_back())
        .is_some_and(|ch| ch == '.')
}

fn static_scss_at_keyword_prefix_is_include(value: &str, index: usize) -> bool {
    let Some(candidate) = value.get(index..index + "@include".len()) else {
        return false;
    };
    if !candidate.eq_ignore_ascii_case("@include") {
        return false;
    }
    value
        .get(index + "@include".len()..)
        .and_then(|suffix| suffix.chars().next())
        .is_some_and(|ch| ch.is_ascii_whitespace())
}

fn static_stylesheet_variable_name_end(value: &str, mut index: usize) -> usize {
    while index < value.len() {
        let Some(ch) = value[index..].chars().next() else {
            break;
        };
        if !(ch.is_ascii_alphanumeric() || ch == '_' || ch == '-') {
            break;
        }
        index += ch.len_utf8();
    }
    index
}

pub(super) fn static_stylesheet_variable_reference_is_named_argument_label(
    value: &str,
    start: usize,
    mut index: usize,
) -> bool {
    let Some(previous) = value.get(..start).and_then(|prefix| {
        prefix
            .chars()
            .rev()
            .find(|candidate| !candidate.is_ascii_whitespace())
    }) else {
        return false;
    };
    if !matches!(previous, '(' | ',' | ';') {
        return false;
    }
    if previous == ';' && !static_stylesheet_position_is_inside_parentheses(value, start) {
        return false;
    }
    while index < value.len() {
        let Some(ch) = value[index..].chars().next() else {
            return false;
        };
        if ch == ':' {
            return true;
        }
        if !ch.is_ascii_whitespace() {
            return false;
        }
        index += ch.len_utf8();
    }
    false
}

fn static_stylesheet_position_is_inside_parentheses(value: &str, end: usize) -> bool {
    let mut index = 0usize;
    let mut paren_depth = 0usize;
    let mut quote: Option<char> = None;
    while index < end && index < value.len() {
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
        match ch {
            '(' => paren_depth += 1,
            ')' => paren_depth = paren_depth.saturating_sub(1),
            _ => {}
        }
        index += ch.len_utf8();
    }
    paren_depth > 0
}
