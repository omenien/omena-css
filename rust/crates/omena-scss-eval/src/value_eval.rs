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
        &[("if", parse_static_scss_if_value)],
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
    let left = parse_numeric_value_with_unit(left)?;
    let right = parse_numeric_value_with_unit(right)?;
    if !left.unit.eq_ignore_ascii_case(right.unit) {
        return None;
    }
    Some(match operator {
        StaticScssComparisonOperator::LessThan => left.value < right.value,
        StaticScssComparisonOperator::LessThanOrEqual => left.value <= right.value,
        StaticScssComparisonOperator::GreaterThan => left.value > right.value,
        StaticScssComparisonOperator::GreaterThanOrEqual => left.value >= right.value,
        StaticScssComparisonOperator::Equal | StaticScssComparisonOperator::NotEqual => {
            return None;
        }
    })
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
