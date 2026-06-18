use omena_value_lattice::{
    css_values_canonically_equal,
    number::{
        parse_reducible_abs_value, parse_reducible_calc_value, parse_reducible_clamp_value,
        parse_reducible_exp_value, parse_reducible_hypot_value, parse_reducible_log_value,
        parse_reducible_max_value, parse_reducible_min_value, parse_reducible_mod_value,
        parse_reducible_pow_value, parse_reducible_rem_value, parse_reducible_round_value,
        parse_reducible_sign_value, parse_reducible_sqrt_value, reduce_static_numeric_expression,
    },
    parse_numeric_value_with_unit, parse_whole_function_value_arguments,
    substitute_static_css_function_references_in_value_until_stable,
};

pub(crate) fn reduce_static_scss_value(value: String) -> String {
    let trimmed = value.trim();
    let value = substitute_static_css_function_references_in_value_until_stable(
        trimmed,
        &[
            ("if", parse_static_scss_if_value),
            ("nth", parse_static_scss_nth_value),
            ("list.nth", parse_static_scss_list_nth_value),
            ("map-get", parse_static_scss_map_get_value),
            ("map.get", parse_static_scss_map_get_namespaced_value),
        ],
    )
    .unwrap_or_else(|| trimmed.to_string());
    reduce_static_numeric_value(value)
}

pub(crate) fn reduce_static_numeric_value(value: String) -> String {
    let trimmed = value.trim();
    if let Some(reduced) = substitute_static_css_function_references_in_value_until_stable(
        trimmed,
        &[
            ("calc", parse_reducible_calc_value),
            ("min", parse_reducible_min_value),
            ("max", parse_reducible_max_value),
            ("clamp", parse_reducible_clamp_value),
            ("abs", parse_reducible_abs_value),
            ("sign", parse_reducible_sign_value),
            ("round", parse_reducible_round_value),
            ("mod", parse_reducible_mod_value),
            ("rem", parse_reducible_rem_value),
            ("hypot", parse_reducible_hypot_value),
            ("sqrt", parse_reducible_sqrt_value),
            ("pow", parse_reducible_pow_value),
            ("exp", parse_reducible_exp_value),
            ("log", parse_reducible_log_value),
        ],
    ) {
        return reduced;
    }
    if let Some(reduced) = reduce_static_numeric_expression(trimmed) {
        return reduced;
    }
    let Some(inner) = trimmed
        .strip_prefix('(')
        .and_then(|without_left| without_left.strip_suffix(')'))
    else {
        return value;
    };
    reduce_static_numeric_expression(inner.trim()).unwrap_or(value)
}

pub(crate) fn static_scss_bang_usage_is_comparison_only(value: &str) -> bool {
    let mut index = 0usize;
    while let Some(relative_index) = value[index..].find('!') {
        let bang_index = index + relative_index;
        if !value
            .get(bang_index + '!'.len_utf8()..)
            .is_some_and(|suffix| suffix.starts_with('='))
        {
            return false;
        }
        index = bang_index + '!'.len_utf8();
    }
    true
}

fn parse_static_scss_if_value(value: &str) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, "if")?;
    let [condition, truthy, falsey] = arguments.as_slice() else {
        return None;
    };
    let truthiness = static_scss_literal_truthiness(condition.trim())?;
    Some(if truthiness {
        truthy.trim().to_string()
    } else {
        falsey.trim().to_string()
    })
}

fn parse_static_scss_nth_value(value: &str) -> Option<String> {
    parse_static_scss_nth_value_with_name(value, "nth")
}

fn parse_static_scss_list_nth_value(value: &str) -> Option<String> {
    parse_static_scss_nth_value_with_name(value, "list.nth")
}

fn parse_static_scss_nth_value_with_name(value: &str, function_name: &str) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, function_name)?;
    let [list, index] = arguments.as_slice() else {
        return None;
    };
    let items = parse_static_scss_list_items(list)?;
    let index = parse_static_scss_list_index(index)?;
    let resolved_index = if index > 0 {
        index.checked_sub(1)? as usize
    } else {
        items.len().checked_sub(index.unsigned_abs())?
    };
    items.get(resolved_index).cloned()
}

fn parse_static_scss_map_get_value(value: &str) -> Option<String> {
    parse_static_scss_map_get_value_with_name(value, "map-get")
}

fn parse_static_scss_map_get_namespaced_value(value: &str) -> Option<String> {
    parse_static_scss_map_get_value_with_name(value, "map.get")
}

fn parse_static_scss_map_get_value_with_name(value: &str, function_name: &str) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, function_name)?;
    let [map, key] = arguments.as_slice() else {
        return None;
    };
    let key = canonical_static_scss_map_key(key)?;
    for (candidate_key, candidate_value) in parse_static_scss_map_entries(map)? {
        if canonical_static_scss_map_key(candidate_key.as_str())? == key {
            return Some(candidate_value);
        }
    }
    None
}

fn parse_static_scss_list_index(value: &str) -> Option<isize> {
    let reduced = reduce_static_numeric_value(value.trim().to_string());
    let index = reduced.trim().parse::<isize>().ok()?;
    (index != 0).then_some(index)
}

fn parse_static_scss_list_items(value: &str) -> Option<Vec<String>> {
    let source = strip_static_scss_outer_container(value.trim()).unwrap_or_else(|| value.trim());
    let items = match split_static_scss_top_level(source, ',') {
        Some(items) if items.len() > 1 => items,
        _ => split_static_scss_top_level_whitespace(source)?,
    };
    if items.is_empty()
        || items
            .iter()
            .any(|item| !static_scss_collection_member_is_static(item))
    {
        return None;
    }
    Some(items)
}

fn parse_static_scss_map_entries(value: &str) -> Option<Vec<(String, String)>> {
    let source = strip_static_scss_outer_container(value.trim())?;
    let entries = split_static_scss_top_level(source, ',')?;
    if entries.is_empty() {
        return None;
    }
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

fn canonical_static_scss_map_key(value: &str) -> Option<String> {
    let value = value.trim();
    if value.is_empty()
        || value.contains('$')
        || static_scss_top_level_separator_index(value, ':')?.is_some()
    {
        return None;
    }
    Some(strip_static_scss_quotes(value).unwrap_or(value).to_string())
}

fn static_scss_collection_member_is_static(value: &str) -> bool {
    !value.trim().is_empty()
        && !value.contains('$')
        && static_scss_top_level_separator_index(value, ':').is_some_and(|index| index.is_none())
}

fn strip_static_scss_quotes(value: &str) -> Option<&str> {
    let quote = value.chars().next()?;
    if !matches!(quote, '"' | '\'') || !value.ends_with(quote) || value.len() < 2 {
        return None;
    }
    value.get(quote.len_utf8()..value.len().saturating_sub(quote.len_utf8()))
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

pub(crate) fn static_scss_literal_truthiness(value: &str) -> Option<bool> {
    let trimmed = value.trim();
    let normalized = trimmed.to_ascii_lowercase();
    if let Some(inner) = strip_static_scss_outer_parens(trimmed) {
        return static_scss_literal_truthiness(inner);
    }
    match split_static_scss_boolean_operands(trimmed, "or") {
        Ok(Some(operands)) => return static_scss_or_truthiness(operands),
        Ok(None) => {}
        Err(()) => return None,
    }
    match split_static_scss_boolean_operands(trimmed, "and") {
        Ok(Some(operands)) => return static_scss_and_truthiness(operands),
        Ok(None) => {}
        Err(()) => return None,
    }
    if trimmed
        .get(..3)
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case("not"))
        && let Some(operand) = trimmed.get(3..)
        && operand.chars().next().is_some_and(char::is_whitespace)
    {
        return static_scss_literal_truthiness(operand.trim()).map(|truthy| !truthy);
    }
    match static_scss_comparison_truthiness(trimmed) {
        Ok(Some(truthy)) => return Some(truthy),
        Ok(None) => {}
        Err(()) => return None,
    }
    match normalized.as_str() {
        "false" | "null" => Some(false),
        "" => None,
        _ if normalized.starts_with('$') || normalized.contains('(') => None,
        _ => Some(true),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StaticScssComparisonOperator {
    Equal,
    NotEqual,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
}

fn static_scss_comparison_truthiness(value: &str) -> Result<Option<bool>, ()> {
    let Some((left, operator, right)) = split_static_scss_comparison(value)? else {
        return Ok(None);
    };
    let left = static_scss_comparable_operand(left).ok_or(())?;
    let right = static_scss_comparable_operand(right).ok_or(())?;
    let equal = left == right || css_values_canonically_equal(left.as_str(), right.as_str());
    Ok(Some(match operator {
        StaticScssComparisonOperator::Equal => equal,
        StaticScssComparisonOperator::NotEqual => !equal,
        StaticScssComparisonOperator::LessThan
        | StaticScssComparisonOperator::LessThanOrEqual
        | StaticScssComparisonOperator::GreaterThan
        | StaticScssComparisonOperator::GreaterThanOrEqual => {
            static_scss_numeric_ordering_truthiness(left.as_str(), operator, right.as_str())
                .ok_or(())?
        }
    }))
}

fn static_scss_numeric_ordering_truthiness(
    left: &str,
    operator: StaticScssComparisonOperator,
    right: &str,
) -> Option<bool> {
    let left_value = parse_numeric_value_with_unit(left)?;
    let right_value = parse_numeric_value_with_unit(right)?;
    if !left_value.unit.eq_ignore_ascii_case(right_value.unit)
        && !static_scss_zero_values_share_unitless_canonical_form(left, right)
    {
        return None;
    }
    Some(match operator {
        StaticScssComparisonOperator::LessThan => left_value.value < right_value.value,
        StaticScssComparisonOperator::LessThanOrEqual => left_value.value <= right_value.value,
        StaticScssComparisonOperator::GreaterThan => left_value.value > right_value.value,
        StaticScssComparisonOperator::GreaterThanOrEqual => left_value.value >= right_value.value,
        StaticScssComparisonOperator::Equal | StaticScssComparisonOperator::NotEqual => {
            return None;
        }
    })
}

fn static_scss_zero_values_share_unitless_canonical_form(left: &str, right: &str) -> bool {
    let Some(left_value) = parse_numeric_value_with_unit(left) else {
        return false;
    };
    let Some(right_value) = parse_numeric_value_with_unit(right) else {
        return false;
    };
    if left_value.value != 0.0 || right_value.value != 0.0 {
        return false;
    }
    if !left_value.unit.is_empty() && !right_value.unit.is_empty() {
        return false;
    }
    css_values_canonically_equal(left, right)
}

fn static_scss_comparable_operand(value: &str) -> Option<String> {
    let reduced = reduce_static_numeric_value(value.trim().to_string());
    let normalized = reduced.to_ascii_lowercase();
    if reduced.is_empty()
        || reduced.contains('$')
        || normalized.contains("var(")
        || normalized.contains("env(")
        || normalized.contains('(')
        || normalized.contains(')')
    {
        return None;
    }
    Some(reduced)
}

fn split_static_scss_comparison(
    value: &str,
) -> Result<Option<(&str, StaticScssComparisonOperator, &str)>, ()> {
    let mut comparison = None;
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;
    let mut quote: Option<char> = None;
    let mut index = 0usize;

    while index < value.len() {
        let ch = value[index..].chars().next().ok_or(())?;
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
            ')' => paren_depth = paren_depth.checked_sub(1).ok_or(())?,
            '[' => bracket_depth += 1,
            ']' => bracket_depth = bracket_depth.checked_sub(1).ok_or(())?,
            '=' | '!' | '<' | '>' if paren_depth == 0 && bracket_depth == 0 => {
                let (operator, width) = static_scss_comparison_operator_at(value, index)?;
                let left = value.get(..index).ok_or(())?.trim();
                let right = value.get(index + width..).ok_or(())?.trim();
                if left.is_empty() || right.is_empty() || comparison.is_some() {
                    return Err(());
                }
                comparison = Some((left, operator, right));
                index += width;
                continue;
            }
            _ => {}
        }
        index += ch.len_utf8();
    }
    if quote.is_some() || paren_depth != 0 || bracket_depth != 0 {
        return Err(());
    }
    Ok(comparison)
}

fn static_scss_comparison_operator_at(
    value: &str,
    index: usize,
) -> Result<(StaticScssComparisonOperator, usize), ()> {
    let suffix = value.get(index..).ok_or(())?;
    if suffix.starts_with("==") {
        return Ok((StaticScssComparisonOperator::Equal, 2));
    }
    if suffix.starts_with("!=") {
        return Ok((StaticScssComparisonOperator::NotEqual, 2));
    }
    if suffix.starts_with("<=") {
        return Ok((StaticScssComparisonOperator::LessThanOrEqual, 2));
    }
    if suffix.starts_with(">=") {
        return Ok((StaticScssComparisonOperator::GreaterThanOrEqual, 2));
    }
    if suffix.starts_with('<') {
        return Ok((StaticScssComparisonOperator::LessThan, 1));
    }
    if suffix.starts_with('>') {
        return Ok((StaticScssComparisonOperator::GreaterThan, 1));
    }
    Err(())
}

fn strip_static_scss_outer_parens(value: &str) -> Option<&str> {
    let inner_start = value.strip_prefix('(')?;
    value.strip_suffix(')')?;
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;
    let mut quote: Option<char> = None;
    let mut index = 0usize;
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
        match ch {
            '(' => paren_depth += 1,
            ')' => {
                paren_depth = paren_depth.checked_sub(1)?;
                if paren_depth == 0 && index + ch.len_utf8() != value.len() {
                    return None;
                }
            }
            '[' => bracket_depth += 1,
            ']' => bracket_depth = bracket_depth.checked_sub(1)?,
            _ => {}
        }
        index += ch.len_utf8();
    }
    (quote.is_none() && paren_depth == 0 && bracket_depth == 0)
        .then(|| inner_start.strip_suffix(')').unwrap_or(inner_start).trim())
}

fn static_scss_or_truthiness(operands: Vec<&str>) -> Option<bool> {
    let mut saw_unknown = false;
    for operand in operands {
        match static_scss_literal_truthiness(operand) {
            Some(true) => return Some(true),
            Some(false) => {}
            None => saw_unknown = true,
        }
    }
    (!saw_unknown).then_some(false)
}

fn static_scss_and_truthiness(operands: Vec<&str>) -> Option<bool> {
    let mut saw_unknown = false;
    for operand in operands {
        match static_scss_literal_truthiness(operand) {
            Some(true) => {}
            Some(false) => return Some(false),
            None => saw_unknown = true,
        }
    }
    (!saw_unknown).then_some(true)
}

fn split_static_scss_boolean_operands<'a>(
    value: &'a str,
    keyword: &str,
) -> Result<Option<Vec<&'a str>>, ()> {
    let mut operands = Vec::new();
    let mut cursor = 0usize;
    let mut index = 0usize;
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;
    let mut quote: Option<char> = None;

    while index < value.len() {
        let ch = value[index..].chars().next().ok_or(())?;
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
            ')' => paren_depth = paren_depth.checked_sub(1).ok_or(())?,
            '[' => bracket_depth += 1,
            ']' => bracket_depth = bracket_depth.checked_sub(1).ok_or(())?,
            _ => {}
        }
        if paren_depth == 0
            && bracket_depth == 0
            && static_scss_boolean_keyword_at(value, index, keyword)
        {
            let operand = value.get(cursor..index).ok_or(())?.trim();
            if operand.is_empty() {
                return Err(());
            }
            operands.push(operand);
            index += keyword.len();
            cursor = index;
            continue;
        }
        index += ch.len_utf8();
    }

    if quote.is_some() || paren_depth != 0 || bracket_depth != 0 {
        return Err(());
    }
    if operands.is_empty() {
        return Ok(None);
    }
    let operand = value.get(cursor..).ok_or(())?.trim();
    if operand.is_empty() {
        return Err(());
    }
    operands.push(operand);
    Ok(Some(operands))
}

fn static_scss_boolean_keyword_at(value: &str, index: usize, keyword: &str) -> bool {
    if !value
        .get(index..)
        .is_some_and(|suffix| suffix.starts_with(keyword))
    {
        return false;
    }
    let before_ok = value
        .get(..index)
        .and_then(|prefix| prefix.chars().next_back())
        .is_some_and(char::is_whitespace);
    let after_index = index + keyword.len();
    let after_ok = value
        .get(after_index..)
        .and_then(|suffix| suffix.chars().next())
        .is_some_and(char::is_whitespace);
    before_ok && after_ok
}
