use super::{
    ascii::{ascii_css_identifier_end, starts_with_ascii_case_insensitive},
    identifiers::{css_identifier_escape_sequence_end, css_identifier_text_is_plain},
    values::{matching_function_end, split_top_level_value_arguments},
};

pub(crate) fn split_css_selector_list(selector: &str) -> Option<Vec<String>> {
    let mut selectors = Vec::new();
    let mut segment_start = 0usize;
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;
    let mut quote = None::<char>;
    let mut escaped = false;

    for (index, character) in selector.char_indices() {
        if let Some(active_quote) = quote {
            if escaped {
                escaped = false;
                continue;
            }
            if character == '\\' {
                escaped = true;
                continue;
            }
            if character == active_quote {
                quote = None;
            }
            continue;
        }

        match character {
            '\'' | '"' => quote = Some(character),
            '(' => paren_depth += 1,
            ')' => paren_depth = paren_depth.checked_sub(1)?,
            '[' => bracket_depth += 1,
            ']' => bracket_depth = bracket_depth.checked_sub(1)?,
            ',' if paren_depth == 0 && bracket_depth == 0 => {
                let selector = selector[segment_start..index].trim();
                if selector.is_empty() {
                    return None;
                }
                selectors.push(selector.to_string());
                segment_start = index + character.len_utf8();
            }
            _ => {}
        }
    }

    if quote.is_some() || paren_depth != 0 || bracket_depth != 0 {
        return None;
    }

    let selector = selector[segment_start..].trim();
    if selector.is_empty() {
        return None;
    }
    selectors.push(selector.to_string());
    Some(selectors)
}

pub(crate) fn selector_branch_owner_class_names(selector: &str) -> Option<Vec<String>> {
    let selector = selector.trim();
    if selector.is_empty() {
        return None;
    }

    let mut index = 0usize;
    let mut quote: Option<char> = None;
    let mut bracket_depth = 0usize;
    let mut paren_depth = 0usize;

    while index < selector.len() {
        let ch = selector[index..].chars().next()?;

        if let Some(quote_ch) = quote {
            index += ch.len_utf8();
            if ch == '\\' {
                let escaped = selector[index..].chars().next()?;
                index += escaped.len_utf8();
            } else if ch == quote_ch {
                quote = None;
            }
            continue;
        }

        if bracket_depth == 0
            && paren_depth == 0
            && let Some(global_end) = global_pseudo_function_end(selector, index)
        {
            index = global_end;
            continue;
        }
        if bracket_depth == 0
            && paren_depth == 0
            && let Some(local_end) = local_pseudo_function_end(selector, index)
        {
            let inner_start = index + ":local(".len();
            let inner_end = local_end.saturating_sub(1);
            return selector_list_owner_class_names(&selector[inner_start..inner_end]);
        }
        if bracket_depth == 0
            && paren_depth == 0
            && let Some(selector_function_end) = selector_owner_pseudo_function_end(selector, index)
        {
            let inner_start = selector[index..].find('(')? + index + '('.len_utf8();
            let inner_end = selector_function_end.saturating_sub(1);
            return selector_list_owner_class_names(&selector[inner_start..inner_end]);
        }
        if bracket_depth == 0
            && paren_depth == 0
            && let Some(ignored_function_end) =
                selector_ignored_pseudo_function_end(selector, index)
        {
            index = ignored_function_end;
            continue;
        }

        match ch {
            '"' | '\'' => {
                quote = Some(ch);
                index += ch.len_utf8();
            }
            '[' => {
                bracket_depth += 1;
                index += ch.len_utf8();
            }
            ']' => {
                bracket_depth = bracket_depth.saturating_sub(1);
                index += ch.len_utf8();
            }
            '(' => {
                paren_depth += 1;
                index += ch.len_utf8();
            }
            ')' => {
                paren_depth = paren_depth.saturating_sub(1);
                index += ch.len_utf8();
            }
            '\\' => return None,
            '.' if bracket_depth == 0 && paren_depth == 0 => {
                let name_start = index + ch.len_utf8();
                let name_end = css_class_selector_name_end(selector, name_start);
                if name_end == name_start {
                    return None;
                }
                return Some(vec![selector[name_start..name_end].to_string()]);
            }
            _ => {
                index += ch.len_utf8();
            }
        }
    }

    None
}

fn selector_list_owner_class_names(selector_list: &str) -> Option<Vec<String>> {
    let branches = split_css_selector_list(selector_list)?;
    if branches.is_empty() {
        return None;
    }

    let mut owner_class_names = Vec::new();
    for branch in branches {
        let branch_owner_class_names = selector_branch_owner_class_names(&branch)?;
        for class_name in branch_owner_class_names {
            if !owner_class_names
                .iter()
                .any(|existing| existing == &class_name)
            {
                owner_class_names.push(class_name);
            }
        }
    }

    (!owner_class_names.is_empty()).then_some(owner_class_names)
}

fn selector_owner_pseudo_function_end(selector: &str, index: usize) -> Option<usize> {
    selector_named_pseudo_function_end(selector, index, "is")
        .or_else(|| selector_named_pseudo_function_end(selector, index, "where"))
}

fn selector_ignored_pseudo_function_end(selector: &str, index: usize) -> Option<usize> {
    selector_named_pseudo_function_end(selector, index, "not")
        .or_else(|| selector_named_pseudo_function_end(selector, index, "has"))
}

fn selector_named_pseudo_function_end(selector: &str, index: usize, name: &str) -> Option<usize> {
    if !selector[index..].starts_with(':') {
        return None;
    }
    let name_start = index + ':'.len_utf8();
    let name_end = name_start + name.len();
    let candidate = selector.get(name_start..name_end)?;
    if !candidate.eq_ignore_ascii_case(name) || !selector[name_end..].starts_with('(') {
        return None;
    }
    matching_function_end(selector, name_end)
}

pub(crate) fn simple_class_selector_name(selector: &str) -> Option<String> {
    let selector = selector.trim();
    if let Some(local_end) = local_pseudo_function_end(selector, 0)
        && local_end == selector.len()
    {
        let inner_start = ":local(".len();
        let inner_end = local_end.saturating_sub(1);
        return simple_class_selector_name(&selector[inner_start..inner_end]);
    }

    let name = selector.strip_prefix('.')?;
    if name.is_empty() {
        return None;
    }
    let name_end = css_class_selector_name_end(selector, '.'.len_utf8());
    if name_end != selector.len() {
        return None;
    }
    if !name.contains('\\') && !css_identifier_text_is_plain(name) {
        return None;
    }
    Some(name.to_string())
}

pub(crate) fn simple_class_selector_names(selector: &str) -> Option<Vec<String>> {
    let selector = selector.trim();
    if let Some(local_end) = local_pseudo_function_end(selector, 0)
        && local_end == selector.len()
    {
        let inner_start = ":local(".len();
        let inner_end = local_end.saturating_sub(1);
        return simple_class_selector_names(&selector[inner_start..inner_end]);
    }
    let branches = split_top_level_value_arguments(selector)?;
    if branches.is_empty() {
        return None;
    }
    branches
        .iter()
        .map(|branch| simple_class_selector_name(branch))
        .collect()
}

pub(crate) fn global_pseudo_function_end(selector: &str, index: usize) -> Option<usize> {
    const GLOBAL_PREFIX: &str = ":global(";
    if !starts_with_ascii_case_insensitive(&selector[index..], GLOBAL_PREFIX) {
        return None;
    }
    matching_function_end(selector, index + GLOBAL_PREFIX.len() - 1)
}

pub(crate) fn local_pseudo_function_end(selector: &str, index: usize) -> Option<usize> {
    const LOCAL_PREFIX: &str = ":local(";
    if !starts_with_ascii_case_insensitive(&selector[index..], LOCAL_PREFIX) {
        return None;
    }
    matching_function_end(selector, index + LOCAL_PREFIX.len() - 1)
}

pub(crate) fn css_class_selector_name_end(selector: &str, start: usize) -> usize {
    let mut end = start;
    while end < selector.len() {
        let Some(ch) = selector[end..].chars().next() else {
            break;
        };
        if ch == '\\' {
            let Some(escape_end) = css_identifier_escape_sequence_end(selector, end) else {
                break;
            };
            end = escape_end;
            continue;
        }
        let next = ascii_css_identifier_end(selector, end);
        if next == end {
            break;
        }
        end = next;
    }
    end
}
