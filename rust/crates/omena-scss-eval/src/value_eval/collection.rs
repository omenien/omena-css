use omena_value_lattice::css_values_canonically_equal;

pub(super) fn static_scss_named_argument(value: &str) -> Option<Option<(&str, &str)>> {
    let Some(index) = static_scss_top_level_separator_index(value, ':')? else {
        return Some(None);
    };
    let name = value.get(..index)?.trim().strip_prefix('$')?;
    let argument_value = value.get(index + ':'.len_utf8()..)?.trim();
    if name.is_empty() || argument_value.is_empty() {
        return None;
    }
    Some(Some((name, argument_value)))
}

pub(super) fn static_scss_named_argument_value<'a>(
    value: &'a str,
    name: &str,
) -> Option<Option<&'a str>> {
    match static_scss_named_argument(value)? {
        Some((argument_name, argument_value)) if argument_name == name => {
            Some(Some(argument_value))
        }
        Some(_) => None,
        None => Some(None),
    }
}

pub(super) fn static_scss_render_comma_list(items: Vec<String>) -> Option<String> {
    Some(if items.is_empty() {
        "()".to_string()
    } else {
        format!("({})", items.join(", "))
    })
}

pub(super) fn static_scss_render_map_entries(entries: Vec<(String, String)>) -> Option<String> {
    Some(if entries.is_empty() {
        "()".to_string()
    } else {
        let entries = entries
            .into_iter()
            .map(|(key, value)| format!("{key}: {value}"))
            .collect::<Vec<_>>();
        format!("({})", entries.join(", "))
    })
}

pub(super) fn static_scss_list_separator(value: &str) -> Option<&'static str> {
    let source = strip_static_scss_outer_container(value.trim()).unwrap_or_else(|| value.trim());
    if source.is_empty() {
        return None;
    }
    if split_static_scss_top_level(source, ',').is_some_and(|items| items.len() > 1) {
        return Some("comma");
    }
    if split_static_scss_top_level(source, '/').is_some_and(|items| items.len() > 1) {
        return Some("slash");
    }
    if split_static_scss_top_level_whitespace(source).is_some_and(|items| items.len() > 1) {
        return Some("space");
    }
    static_scss_collection_member_is_static(source).then_some("space")
}

pub(super) fn parse_static_scss_map_entries(value: &str) -> Option<Vec<(String, String)>> {
    let source = strip_static_scss_outer_container(value.trim())?;
    if source.is_empty() {
        return Some(Vec::new());
    }
    let entries = split_static_scss_top_level(source, ',')?;
    let mut pairs = Vec::with_capacity(entries.len());
    for entry in entries {
        let colon_index = static_scss_top_level_separator_index(entry.as_str(), ':')??;
        let key = entry.get(..colon_index)?.trim();
        let value = entry.get(colon_index + ':'.len_utf8()..)?.trim();
        if key.is_empty()
            || value.is_empty()
            || key.contains('$')
            || !static_scss_collection_member_is_static(value)
        {
            return None;
        }
        pairs.push((key.to_string(), value.to_string()));
    }
    Some(pairs)
}

pub(super) fn static_scss_update_nested_map_entries<F>(
    mut entries: Vec<(String, String)>,
    path: &[String],
    update: F,
) -> Option<Vec<(String, String)>>
where
    F: FnOnce(Vec<(String, String)>) -> Option<Vec<(String, String)>>,
{
    let Some((key, remaining_path)) = path.split_first() else {
        return update(entries);
    };
    let canonical_key = canonical_static_scss_map_key(key)?;
    let existing_index = static_scss_map_entry_index(entries.as_slice(), canonical_key.as_str())?;
    let child_entries = match existing_index {
        Some(index) => static_scss_nested_map_child_entries(entries[index].1.as_str())?,
        None => Vec::new(),
    };
    let updated_child_entries =
        static_scss_update_nested_map_entries(child_entries, remaining_path, update)?;
    let updated_child_value = static_scss_render_map_entries(updated_child_entries)?;
    if let Some(index) = existing_index {
        entries[index].1 = updated_child_value;
    } else {
        entries.push((key.trim().to_string(), updated_child_value));
    }
    Some(entries)
}

fn static_scss_nested_map_child_entries(value: &str) -> Option<Vec<(String, String)>> {
    if let Some(entries) = parse_static_scss_map_entries(value) {
        return Some(entries);
    }
    static_scss_collection_member_is_static(value).then(Vec::new)
}

pub(super) fn static_scss_existing_nested_map_child_entries(
    value: &str,
) -> Option<Option<Vec<(String, String)>>> {
    if let Some(entries) = parse_static_scss_map_entries(value) {
        return Some(Some(entries));
    }
    static_scss_collection_member_is_static(value).then_some(None)
}

pub(super) fn static_scss_map_entry_index(
    entries: &[(String, String)],
    canonical_key: &str,
) -> Option<Option<usize>> {
    for (index, (key, _)) in entries.iter().enumerate() {
        if canonical_static_scss_map_key(key.as_str())? == canonical_key {
            return Some(Some(index));
        }
    }
    Some(None)
}

pub(super) fn static_scss_map_entry_value(map: &str, key: &str) -> Option<String> {
    parse_static_scss_map_entries(map)?
        .into_iter()
        .find_map(|(candidate_key, candidate_value)| {
            canonical_static_scss_map_key(candidate_key.as_str())
                .is_some_and(|candidate| candidate == key)
                .then_some(candidate_value)
        })
}

pub(super) fn static_scss_map_contains_key(map: &str, key: &str) -> bool {
    parse_static_scss_map_entries(map).is_some_and(|entries| {
        entries.into_iter().any(|(candidate_key, _)| {
            canonical_static_scss_map_key(candidate_key.as_str())
                .is_some_and(|candidate| candidate == key)
        })
    })
}

pub(super) fn canonical_static_scss_map_key(value: &str) -> Option<String> {
    let value = value.trim();
    if value.is_empty()
        || value.contains('$')
        || static_scss_top_level_separator_index(value, ':')?.is_some()
    {
        return None;
    }
    Some(strip_static_scss_quotes(value).unwrap_or(value).to_string())
}

pub(super) fn static_scss_comparable_collection_value(value: &str) -> Option<String> {
    let value = value.trim();
    if !static_scss_collection_member_is_static(value) {
        return None;
    }
    Some(strip_static_scss_quotes(value).unwrap_or(value).to_string())
}

pub(super) fn static_scss_collection_values_equal(left: &str, right: &str) -> bool {
    left == right || css_values_canonically_equal(left, right)
}

pub(super) fn static_scss_collection_member_is_static(value: &str) -> bool {
    !value.trim().is_empty()
        && !value.contains('$')
        && static_scss_top_level_separator_index(value, ':').is_some_and(|index| index.is_none())
}

pub(super) fn strip_static_scss_quotes(value: &str) -> Option<&str> {
    let quote = value.chars().next()?;
    if !matches!(quote, '"' | '\'') || !value.ends_with(quote) || value.len() < 2 {
        return None;
    }
    value.get(quote.len_utf8()..value.len().saturating_sub(quote.len_utf8()))
}

pub(super) fn strip_static_scss_outer_container(value: &str) -> Option<&str> {
    let trimmed = value.trim();
    if trimmed.len() < 2 {
        return None;
    }
    let (open, close) = match trimmed.chars().next()? {
        '(' => ('(', ')'),
        '[' => ('[', ']'),
        _ => return None,
    };
    let end = static_scss_balanced_value_end(trimmed, 0, open, close)?;
    if end != trimmed.len() {
        return None;
    }
    trimmed
        .get(open.len_utf8()..trimmed.len().saturating_sub(close.len_utf8()))
        .map(str::trim)
}

pub(super) fn split_static_scss_top_level(source: &str, separator: char) -> Option<Vec<String>> {
    let mut values = Vec::new();
    let mut cursor = 0usize;
    let mut index = 0usize;
    while index < source.len() {
        let ch = source[index..].chars().next()?;
        if ch == separator {
            let value = source.get(cursor..index)?.trim();
            if value.is_empty() {
                return None;
            }
            values.push(value.to_string());
            cursor = index + ch.len_utf8();
        }
        index = static_scss_next_value_index(source, index)?;
    }
    let value = source.get(cursor..)?.trim();
    if value.is_empty() {
        return None;
    }
    values.push(value.to_string());
    Some(values)
}

pub(super) fn split_static_scss_top_level_whitespace(source: &str) -> Option<Vec<String>> {
    let mut values = Vec::new();
    let mut cursor = 0usize;
    let mut index = 0usize;
    while index < source.len() {
        let ch = source[index..].chars().next()?;
        if ch.is_ascii_whitespace() {
            let value = source.get(cursor..index)?.trim();
            if !value.is_empty() {
                values.push(value.to_string());
            }
            index += ch.len_utf8();
            while index < source.len() {
                let Some(next_ch) = source[index..].chars().next() else {
                    break;
                };
                if !next_ch.is_ascii_whitespace() {
                    break;
                }
                index += next_ch.len_utf8();
            }
            cursor = index;
            continue;
        }
        index = static_scss_next_value_index(source, index)?;
    }
    let value = source.get(cursor..)?.trim();
    if !value.is_empty() {
        values.push(value.to_string());
    }
    Some(values)
}

pub(super) fn static_scss_top_level_separator_index(
    source: &str,
    separator: char,
) -> Option<Option<usize>> {
    let mut index = 0usize;
    while index < source.len() {
        let ch = source[index..].chars().next()?;
        if ch == separator {
            return Some(Some(index));
        }
        index = static_scss_next_value_index(source, index)?;
    }
    Some(None)
}

fn static_scss_next_value_index(source: &str, index: usize) -> Option<usize> {
    let ch = source[index..].chars().next()?;
    match ch {
        '"' | '\'' => static_scss_quoted_value_end(source, index, ch),
        '(' => static_scss_balanced_value_end(source, index, '(', ')'),
        '[' => static_scss_balanced_value_end(source, index, '[', ']'),
        ')' | ']' => None,
        _ => Some(index + ch.len_utf8()),
    }
}

pub(super) fn static_scss_quoted_value_end(
    source: &str,
    start: usize,
    quote: char,
) -> Option<usize> {
    let mut index = start + quote.len_utf8();
    while index < source.len() {
        let ch = source[index..].chars().next()?;
        index += ch.len_utf8();
        if ch == '\\' {
            if let Some(escaped) = source[index..].chars().next() {
                index += escaped.len_utf8();
            }
        } else if ch == quote {
            return Some(index);
        }
    }
    None
}

fn static_scss_balanced_value_end(
    source: &str,
    start: usize,
    open: char,
    close: char,
) -> Option<usize> {
    let mut depth = 0usize;
    let mut index = start;
    while index < source.len() {
        let ch = source[index..].chars().next()?;
        match ch {
            '"' | '\'' => index = static_scss_quoted_value_end(source, index, ch)?,
            _ if ch == open => {
                depth += 1;
                index += ch.len_utf8();
                continue;
            }
            _ if ch == close => {
                depth = depth.checked_sub(1)?;
                index += ch.len_utf8();
                if depth == 0 {
                    return Some(index);
                }
                continue;
            }
            _ => index += ch.len_utf8(),
        }
    }
    None
}
