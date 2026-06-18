pub type StaticCssFunctionParser = fn(&str) -> Option<String>;
pub type StaticCssFunctionSpec<'a> = (&'a str, StaticCssFunctionParser);

pub fn parse_whole_function_value_arguments(
    value: &str,
    function_name: &str,
) -> Option<Vec<String>> {
    split_top_level_value_arguments(parse_whole_function_value_inner(value, function_name)?)
}

pub fn parse_whole_function_value_inner<'a>(
    value: &'a str,
    function_name: &str,
) -> Option<&'a str> {
    let value = value.trim();
    let name = value.get(..function_name.len())?;
    if !name.eq_ignore_ascii_case(function_name) {
        return None;
    }
    value
        .get(function_name.len()..)?
        .strip_prefix('(')?
        .strip_suffix(')')
}

pub fn split_top_level_value_arguments(inner: &str) -> Option<Vec<String>> {
    crate::split_top_level_value_arguments(inner, 0).map(|segments| {
        segments
            .into_iter()
            .map(|segment| segment.text.to_string())
            .collect()
    })
}

pub fn split_top_level_whitespace_value_components(value: &str) -> Option<Vec<String>> {
    crate::split_top_level_whitespace_value_components(value, 0).map(|segments| {
        segments
            .into_iter()
            .map(|segment| segment.text.to_string())
            .collect()
    })
}

pub fn matching_function_call_end(value: &str, left_paren_index: usize) -> Option<usize> {
    if value[left_paren_index..].chars().next()? != '(' {
        return None;
    }

    let mut depth = 0usize;
    let mut index = left_paren_index;
    let mut quote: Option<char> = None;

    while index < value.len() {
        let ch = value[index..].chars().next()?;

        if let Some(quote_ch) = quote {
            index += ch.len_utf8();
            if ch == '\\' {
                let escaped = value[index..].chars().next()?;
                index += escaped.len_utf8();
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
                if depth == 0 {
                    return Some(index);
                }
                index += ch.len_utf8();
            }
            _ => {
                index += ch.len_utf8();
            }
        }
    }

    None
}

pub fn substitute_static_css_function_references_in_value(
    value: &str,
    functions: &[StaticCssFunctionSpec<'_>],
) -> Option<String> {
    let mut output = String::with_capacity(value.len());
    let mut cursor = 0usize;
    let mut index = 0usize;
    let mut quote: Option<char> = None;
    let mut changed = false;

    while index < value.len() {
        let ch = value[index..].chars().next()?;

        if let Some(quote_ch) = quote {
            index += ch.len_utf8();
            if ch == '\\' {
                let escaped = value[index..].chars().next()?;
                index += escaped.len_utf8();
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
            _ => {
                let Some((function_name, parse_function_value)) =
                    static_css_function_at(value, index, functions)
                else {
                    index += ch.len_utf8();
                    continue;
                };
                let left_paren_index = index + function_name.len();
                let Some(close_index) = matching_function_call_end(value, left_paren_index) else {
                    index += ch.len_utf8();
                    continue;
                };
                if static_css_function_contains_nested_static_call(
                    value,
                    left_paren_index + '('.len_utf8(),
                    close_index,
                    functions,
                )
                .unwrap_or(true)
                {
                    index += ch.len_utf8();
                    continue;
                }
                let function_value = &value[index..close_index + ')'.len_utf8()];
                let Some(replacement_value) = parse_function_value(function_value) else {
                    index += ch.len_utf8();
                    continue;
                };
                output.push_str(&value[cursor..index]);
                output.push_str(&replacement_value);
                index = close_index + ')'.len_utf8();
                cursor = index;
                changed = true;
            }
        }
    }

    if !changed {
        return None;
    }
    output.push_str(&value[cursor..]);
    Some(output)
}

pub fn substitute_static_css_function_references_in_value_until_stable(
    value: &str,
    functions: &[StaticCssFunctionSpec<'_>],
) -> Option<String> {
    let mut current = value.to_string();
    let mut changed = false;

    for _ in 0..8 {
        let Some(next) = substitute_static_css_function_references_in_value(&current, functions)
        else {
            break;
        };
        if next == current {
            break;
        }
        current = next;
        changed = true;
    }

    changed.then_some(current)
}

fn static_css_function_at<'a>(
    value: &'a str,
    index: usize,
    functions: &'a [StaticCssFunctionSpec<'a>],
) -> Option<StaticCssFunctionSpec<'a>> {
    let tail = value.get(index..)?;
    for (function_name, parse_function_value) in functions {
        if static_css_function_left_boundary(value, index)
            && tail.len() > function_name.len()
            && tail[..function_name.len()].eq_ignore_ascii_case(function_name)
            && tail[function_name.len()..].starts_with('(')
        {
            return Some((*function_name, *parse_function_value));
        }
    }
    None
}

fn static_css_function_contains_nested_static_call(
    value: &str,
    start: usize,
    end: usize,
    functions: &[StaticCssFunctionSpec<'_>],
) -> Option<bool> {
    let mut index = start;
    let mut quote: Option<char> = None;

    while index < end {
        let ch = value[index..].chars().next()?;

        if let Some(quote_ch) = quote {
            index += ch.len_utf8();
            if ch == '\\' {
                let escaped = value[index..].chars().next()?;
                index += escaped.len_utf8();
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
            _ => {
                if let Some((function_name, _)) = static_css_function_at(value, index, functions) {
                    let left_paren_index = index + function_name.len();
                    if matching_function_call_end(value, left_paren_index)
                        .is_some_and(|close_index| close_index < end)
                    {
                        return Some(true);
                    }
                }
                index += ch.len_utf8();
            }
        }
    }

    Some(false)
}

fn static_css_function_left_boundary(value: &str, index: usize) -> bool {
    if index == 0 {
        return true;
    }
    value
        .get(..index)
        .and_then(|prefix| prefix.chars().next_back())
        .is_none_or(|ch| !ch.is_ascii_alphanumeric() && !matches!(ch, '_' | '-' | '.'))
}
