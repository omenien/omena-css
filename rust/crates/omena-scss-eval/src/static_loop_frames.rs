pub(crate) fn parse_static_scss_each_loop_binding_frames<F>(
    header: &str,
    mut resolve_dynamic_source: F,
) -> Option<Vec<Vec<(String, String)>>>
where
    F: FnMut(&str) -> Option<String>,
{
    let bindings = static_scss_loop_carried_bindings(header, "in");
    if bindings.len() == 2
        && let Some(frames) =
            static_scss_each_map_loop_binding_frames(header, bindings.as_slice(), |source| {
                resolve_static_scss_each_source_text(source, &mut resolve_dynamic_source)
            })
    {
        return Some(frames);
    }
    if bindings.len() > 1
        && let Some(frames) =
            static_scss_each_tuple_loop_binding_frames(header, bindings.as_slice(), |source| {
                resolve_static_scss_each_source_text(source, &mut resolve_dynamic_source)
            })
    {
        return Some(frames);
    }
    if bindings.len() == 1 {
        return static_scss_each_single_loop_binding_frames(
            header,
            bindings.as_slice(),
            |source| resolve_static_scss_each_source_text(source, &mut resolve_dynamic_source),
        );
    }
    None
}

pub(crate) fn static_scss_for_loop_values(
    start: i32,
    end: i32,
    includes_end: bool,
) -> Option<Vec<i32>> {
    if !includes_end && start == end {
        return Some(Vec::new());
    }

    let step = if start <= end { 1_i64 } else { -1_i64 };
    let stop = if includes_end {
        i64::from(end)
    } else {
        i64::from(end) - step
    };
    let count = ((stop - i64::from(start)) / step) + 1;
    if !(0..=64).contains(&count) {
        return None;
    }

    let mut values = Vec::with_capacity(count as usize);
    let mut current = i64::from(start);
    for _ in 0..count {
        values.push(i32::try_from(current).ok()?);
        current += step;
    }
    Some(values)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct StaticScssForLoopHeader<'a> {
    pub(crate) binding: String,
    pub(crate) start_bound: &'a str,
    pub(crate) end_bound: &'a str,
    pub(crate) includes_end: bool,
}

pub(crate) fn parse_static_scss_for_loop_header(
    header: &str,
) -> Option<StaticScssForLoopHeader<'_>> {
    let (binding_source, after_from) = static_scss_split_header_at_keyword(header, "from")?;
    let bindings = static_scss_variable_names_in_text_preserving_order(binding_source);
    if bindings.len() != 1 {
        return None;
    }

    let to_split = static_scss_split_header_at_keyword(after_from, "to")
        .map(|(start, end)| (start, end, false));
    let through_split = static_scss_split_header_at_keyword(after_from, "through")
        .map(|(start, end)| (start, end, true));
    let (start_bound, end_bound, includes_end) = match (to_split, through_split) {
        (Some(to), Some(through)) if to.0.len() <= through.0.len() => to,
        (Some(_), Some(through)) => through,
        (Some(to), None) => to,
        (None, Some(through)) => through,
        (None, None) => return None,
    };

    let start_bound = start_bound.trim();
    let end_bound = end_bound.trim();
    if bindings[0].is_empty() || start_bound.is_empty() || end_bound.is_empty() {
        return None;
    }

    Some(StaticScssForLoopHeader {
        binding: bindings[0].clone(),
        start_bound,
        end_bound,
        includes_end,
    })
}

fn static_scss_each_map_loop_binding_frames<F>(
    header: &str,
    bindings: &[String],
    mut resolve_source: F,
) -> Option<Vec<Vec<(String, String)>>>
where
    F: FnMut(&str) -> Option<String>,
{
    let (_, source) = static_scss_split_header_at_keyword(header, "in")?;
    let source = resolve_source(source.trim())?;
    let entries = parse_static_scss_each_map_entries(source.as_str())?;
    if entries.len() > 64 {
        return None;
    }
    Some(
        entries
            .into_iter()
            .map(|(key, value)| vec![(bindings[0].clone(), key), (bindings[1].clone(), value)])
            .collect(),
    )
}

fn static_scss_each_tuple_loop_binding_frames<F>(
    header: &str,
    bindings: &[String],
    mut resolve_source: F,
) -> Option<Vec<Vec<(String, String)>>>
where
    F: FnMut(&str) -> Option<String>,
{
    let (_, source) = static_scss_split_header_at_keyword(header, "in")?;
    let source = resolve_source(source.trim())?;
    let entries = parse_static_scss_each_tuple_entries(source.as_str(), bindings.len())?;
    if entries.len() > 64 {
        return None;
    }
    Some(
        entries
            .into_iter()
            .map(|entry| bindings.iter().cloned().zip(entry).collect())
            .collect(),
    )
}

fn static_scss_each_single_loop_binding_frames<F>(
    header: &str,
    bindings: &[String],
    mut resolve_source: F,
) -> Option<Vec<Vec<(String, String)>>>
where
    F: FnMut(&str) -> Option<String>,
{
    let (_, source) = static_scss_split_header_at_keyword(header, "in")?;
    let source = resolve_source(source.trim())?;
    let values = parse_static_scss_each_single_values(source.as_str())?;
    if values.is_empty() || values.len() > 64 {
        return None;
    }
    Some(
        values
            .into_iter()
            .map(|value| vec![(bindings[0].clone(), value)])
            .collect(),
    )
}

fn resolve_static_scss_each_source_text<F>(
    source: &str,
    resolve_dynamic_source: &mut F,
) -> Option<String>
where
    F: FnMut(&str) -> Option<String>,
{
    resolve_dynamic_source(source).or_else(|| Some(source.to_string()))
}

pub(crate) fn parse_static_scss_each_single_values(source: &str) -> Option<Vec<String>> {
    match split_static_scss_top_level(source, ',') {
        Some(values) if values.len() > 1 => Some(values),
        _ => split_static_scss_top_level_whitespace(source),
    }
}

fn parse_static_scss_each_map_entries(source: &str) -> Option<Vec<(String, String)>> {
    let inner = source
        .strip_prefix('(')
        .and_then(|source| source.strip_suffix(')'))?
        .trim();
    if inner.is_empty() {
        return None;
    }
    let entries = split_static_scss_top_level(inner, ',')?;
    let mut pairs = Vec::with_capacity(entries.len());
    for entry in entries {
        let (key, value) = split_static_scss_key_value(entry.as_str())?;
        pairs.push((key.to_string(), value.to_string()));
    }
    Some(pairs)
}

fn split_static_scss_key_value(entry: &str) -> Option<(&str, &str)> {
    let colon_index = static_scss_top_level_separator_index(entry, ':')??;
    let key = entry.get(..colon_index)?.trim();
    let value = entry.get(colon_index + ':'.len_utf8()..)?.trim();
    if key.is_empty() || value.is_empty() || key.contains('$') || value.contains('$') {
        return None;
    }
    Some((key, value))
}

fn parse_static_scss_each_tuple_entries(source: &str, arity: usize) -> Option<Vec<Vec<String>>> {
    let source = strip_static_scss_outer_container(source.trim()).unwrap_or_else(|| source.trim());
    let entries = split_static_scss_top_level(source, ',')?;
    if entries.is_empty() {
        return None;
    }
    let mut tuples = Vec::with_capacity(entries.len());
    for entry in entries {
        tuples.push(parse_static_scss_each_tuple_entry_values(
            entry.as_str(),
            arity,
        )?);
    }
    Some(tuples)
}

fn parse_static_scss_each_tuple_entry_values(entry: &str, arity: usize) -> Option<Vec<String>> {
    let entry = strip_static_scss_outer_container(entry.trim()).unwrap_or_else(|| entry.trim());
    let comma_values = split_static_scss_top_level(entry, ',')?;
    let values = if comma_values.len() == arity {
        comma_values
    } else {
        split_static_scss_top_level_whitespace(entry)?
    };
    if values.len() != arity
        || values
            .iter()
            .any(|value| !static_scss_each_tuple_value_is_static(value))
    {
        return None;
    }
    Some(values)
}

fn static_scss_each_tuple_value_is_static(value: &str) -> bool {
    !value.is_empty()
        && !value.contains('$')
        && static_scss_top_level_separator_index(value, ':').is_some_and(|index| index.is_none())
}

fn strip_static_scss_outer_container(value: &str) -> Option<&str> {
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

fn split_static_scss_top_level(source: &str, separator: char) -> Option<Vec<String>> {
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

fn split_static_scss_top_level_whitespace(source: &str) -> Option<Vec<String>> {
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

fn static_scss_top_level_separator_index(source: &str, separator: char) -> Option<Option<usize>> {
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

fn static_scss_quoted_value_end(source: &str, start: usize, quote: char) -> Option<usize> {
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
            '"' | '\'' => {
                index = static_scss_quoted_value_end(source, index, ch)?;
                continue;
            }
            _ if ch == open => depth += 1,
            _ if ch == close => {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    return Some(index + ch.len_utf8());
                }
            }
            _ => {}
        }
        index += ch.len_utf8();
    }
    None
}

fn static_scss_loop_carried_bindings(header: &str, separator: &str) -> Vec<String> {
    let before_separator = static_scss_split_header_at_keyword(header, separator)
        .map(|(left, _)| left)
        .unwrap_or(header);
    static_scss_variable_names_in_text_preserving_order(before_separator)
}

fn static_scss_split_header_at_keyword<'a>(
    header: &'a str,
    keyword: &str,
) -> Option<(&'a str, &'a str)> {
    let lower_header = header.to_ascii_lowercase();
    let lower_keyword = keyword.to_ascii_lowercase();
    let mut search_start = 0usize;
    while search_start < lower_header.len() {
        let relative_index = lower_header
            .get(search_start..)?
            .find(lower_keyword.as_str())?;
        let index = search_start + relative_index;
        let right_start = index + keyword.len();
        if static_scss_header_keyword_has_boundaries(header, index, right_start) {
            let left = header.get(..index)?;
            let right = header.get(right_start..)?;
            return Some((left, right));
        }
        search_start = right_start;
    }
    None
}

fn static_scss_header_keyword_has_boundaries(header: &str, start: usize, end: usize) -> bool {
    let before_ok = header.get(..start).is_none_or(|text| {
        text.chars()
            .next_back()
            .is_none_or(|ch| ch.is_ascii_whitespace())
    });
    let after_ok = header.get(end..).is_none_or(|text| {
        text.chars()
            .next()
            .is_none_or(|ch| ch.is_ascii_whitespace())
    });
    before_ok && after_ok
}

fn static_scss_variable_names_in_text_preserving_order(text: &str) -> Vec<String> {
    let mut names = Vec::new();
    let mut index = 0usize;
    while index < text.len() {
        let Some(ch) = text[index..].chars().next() else {
            break;
        };
        if ch != '$' {
            index += ch.len_utf8();
            continue;
        }
        let name_start = index + ch.len_utf8();
        let name_end = static_scss_variable_name_end(text, name_start);
        if name_end > name_start
            && let Some(name) = text.get(index..name_end)
            && !names.iter().any(|candidate| candidate == name)
        {
            names.push(name.to_string());
        }
        index = name_end.max(index + ch.len_utf8());
    }
    names
}

fn static_scss_variable_name_end(text: &str, mut index: usize) -> usize {
    while index < text.len() {
        let Some(ch) = text[index..].chars().next() else {
            break;
        };
        if !(ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-')) {
            break;
        }
        index += ch.len_utf8();
    }
    index
}
