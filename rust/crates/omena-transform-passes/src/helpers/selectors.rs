use super::{
    ascii::{ascii_css_identifier_end, starts_with_ascii_case_insensitive},
    identifiers::css_identifier_text_is_plain,
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

pub(crate) fn selector_branch_owner_class_name(selector: &str) -> Option<String> {
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
            return selector_branch_owner_class_name(&selector[inner_start..inner_end]);
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
                let name_end = ascii_css_identifier_end(selector, name_start);
                if name_end == name_start {
                    return None;
                }
                return Some(selector[name_start..name_end].to_string());
            }
            _ => {
                index += ch.len_utf8();
            }
        }
    }

    None
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
    if name.is_empty() || !css_identifier_text_is_plain(name) {
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
