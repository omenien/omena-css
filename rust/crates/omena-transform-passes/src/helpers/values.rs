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
