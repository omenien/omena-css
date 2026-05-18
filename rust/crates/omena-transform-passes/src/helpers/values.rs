pub(crate) fn matching_function_call_end(value: &str, left_paren_index: usize) -> Option<usize> {
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

pub(crate) fn parse_whole_function_value_arguments(
    value: &str,
    function_name: &str,
) -> Option<Vec<String>> {
    split_top_level_value_arguments(parse_whole_function_value_inner(value, function_name)?)
}

pub(crate) fn parse_whole_function_value_inner<'a>(
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

pub(crate) fn split_top_level_value_arguments(inner: &str) -> Option<Vec<String>> {
    let mut arguments = Vec::new();
    let mut current = String::new();
    let mut depth = 0usize;
    let mut bracket_depth = 0usize;
    let mut quote: Option<char> = None;
    let mut escaped = false;

    for ch in inner.chars() {
        if let Some(active_quote) = quote {
            current.push(ch);
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == active_quote {
                quote = None;
            }
            continue;
        }

        match ch {
            '"' | '\'' => {
                quote = Some(ch);
                current.push(ch);
            }
            '(' => {
                depth += 1;
                current.push(ch);
            }
            ')' => {
                depth = depth.checked_sub(1)?;
                current.push(ch);
            }
            '[' => {
                bracket_depth += 1;
                current.push(ch);
            }
            ']' => {
                bracket_depth = bracket_depth.checked_sub(1)?;
                current.push(ch);
            }
            ',' if depth == 0 && bracket_depth == 0 => {
                let argument = current.trim().to_string();
                if argument.is_empty() {
                    return None;
                }
                arguments.push(argument);
                current.clear();
            }
            _ => current.push(ch),
        }
    }

    if quote.is_some() || depth != 0 || bracket_depth != 0 {
        return None;
    }

    let argument = current.trim().to_string();
    if argument.is_empty() {
        return None;
    }
    arguments.push(argument);
    Some(arguments)
}

pub(crate) fn split_top_level_whitespace_value_components(value: &str) -> Option<Vec<String>> {
    let mut components = Vec::new();
    let mut current = String::new();
    let mut depth = 0usize;
    let mut bracket_depth = 0usize;
    let mut quote: Option<char> = None;
    let mut escaped = false;

    for ch in value.chars() {
        if let Some(active_quote) = quote {
            current.push(ch);
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == active_quote {
                quote = None;
            }
            continue;
        }

        match ch {
            '"' | '\'' => {
                quote = Some(ch);
                current.push(ch);
            }
            '(' => {
                depth += 1;
                current.push(ch);
            }
            ')' => {
                depth = depth.checked_sub(1)?;
                current.push(ch);
            }
            '[' => {
                bracket_depth += 1;
                current.push(ch);
            }
            ']' => {
                bracket_depth = bracket_depth.checked_sub(1)?;
                current.push(ch);
            }
            ch if ch.is_ascii_whitespace() && depth == 0 && bracket_depth == 0 => {
                if !current.trim().is_empty() {
                    components.push(current.trim().to_string());
                    current.clear();
                }
            }
            _ => current.push(ch),
        }
    }

    if quote.is_some() || depth != 0 || bracket_depth != 0 {
        return None;
    }
    if !current.trim().is_empty() {
        components.push(current.trim().to_string());
    }
    (!components.is_empty()).then_some(components)
}

pub(crate) type StaticCssFunctionParser = fn(&str) -> Option<String>;
pub(crate) type StaticCssFunctionSpec<'a> = (&'a str, StaticCssFunctionParser);

pub(crate) fn substitute_static_css_function_references_in_value(
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

pub(crate) fn substitute_static_css_function_references_in_value_until_stable(
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
    value: &str,
    index: usize,
    functions: &'a [StaticCssFunctionSpec<'a>],
) -> Option<StaticCssFunctionSpec<'a>> {
    functions.iter().find_map(|(function_name, parser)| {
        let name = value.get(index..index + function_name.len())?;
        let open_paren = value[index + function_name.len()..].chars().next()?;
        (name.eq_ignore_ascii_case(function_name) && open_paren == '(')
            .then_some((*function_name, *parser))
    })
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
