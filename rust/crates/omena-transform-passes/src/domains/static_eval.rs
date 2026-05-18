use std::borrow::Cow;

use omena_cascade::{
    StaticSupportsAssumptionV0, StaticSupportsEvalVerdictV0, evaluate_static_supports_condition,
};
use omena_parser::{StyleDialect, lex};
use omena_syntax::SyntaxKind;

use crate::{
    domains::number::{
        parse_numeric_value_with_unit, parse_reducible_calc_value, parse_reducible_clamp_value,
        parse_reducible_max_value, parse_reducible_min_value,
    },
    helpers::{
        ascii::normalize_ascii_whitespace,
        blocks::at_rule_block_indexes,
        tokens::{token_end, token_start},
    },
    matching_function_call_end, substitute_static_css_function_references_in_value_until_stable,
};

pub(crate) fn evaluate_static_supports_rules_with_lexer(
    source: &str,
    dialect: StyleDialect,
) -> (String, usize) {
    let mut output = source.to_string();
    let mut mutation_count = 0;

    loop {
        let (next_output, next_mutation_count) =
            evaluate_static_supports_rules_once_with_lexer(&output, dialect);
        if next_mutation_count == 0 {
            return (output, mutation_count);
        }
        output = next_output;
        mutation_count += next_mutation_count;
    }
}

fn evaluate_static_supports_rules_once_with_lexer(
    source: &str,
    dialect: StyleDialect,
) -> (String, usize) {
    let lexed = lex(source, dialect);
    let tokens = lexed.tokens();
    let mut replacements = Vec::new();
    let mut index = 0;

    while index < tokens.len() {
        match tokens[index].kind {
            SyntaxKind::AtKeyword if tokens[index].text.eq_ignore_ascii_case("@supports") => {
                let Some((block_start_index, block_end_index)) =
                    at_rule_block_indexes(tokens, index)
                else {
                    index += 1;
                    continue;
                };
                let condition = source
                    [token_end(&tokens[index])..token_start(&tokens[block_start_index])]
                    .trim();
                let witness = evaluate_static_supports_condition(
                    condition,
                    StaticSupportsAssumptionV0::ModernBrowser,
                );
                let replacement = match witness.verdict {
                    StaticSupportsEvalVerdictV0::AlwaysTrue => {
                        source[token_end(&tokens[block_start_index])
                            ..token_start(&tokens[block_end_index])]
                            .trim()
                            .to_string()
                    }
                    StaticSupportsEvalVerdictV0::AlwaysFalse => String::new(),
                    StaticSupportsEvalVerdictV0::Unknown => {
                        index += 1;
                        continue;
                    }
                };
                replacements.push((
                    token_start(&tokens[index]),
                    token_end(&tokens[block_end_index]),
                    replacement,
                ));
                index = block_end_index + 1;
                continue;
            }
            _ => {}
        }
        index += 1;
    }

    if replacements.is_empty() {
        return (source.to_string(), 0);
    }

    let mut output = String::with_capacity(source.len());
    let mut cursor = 0;
    for (start, end, replacement) in &replacements {
        if *start > cursor {
            output.push_str(&source[cursor..*start]);
        }
        output.push_str(replacement);
        cursor = *end;
    }
    if cursor < source.len() {
        output.push_str(&source[cursor..]);
    }

    (output, replacements.len())
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct StaticMediaEvaluationOptions {
    pub(crate) drop_dark_mode_media_queries: bool,
}

pub(crate) fn evaluate_static_media_rules_with_lexer(
    source: &str,
    dialect: StyleDialect,
    options: StaticMediaEvaluationOptions,
) -> (String, usize) {
    let mut output = source.to_string();
    let mut mutation_count = 0;

    loop {
        let (next_output, next_mutation_count) =
            evaluate_static_media_rules_once_with_lexer(&output, dialect, options);
        if next_mutation_count == 0 {
            return (output, mutation_count);
        }
        output = next_output;
        mutation_count += next_mutation_count;
    }
}

fn evaluate_static_media_rules_once_with_lexer(
    source: &str,
    dialect: StyleDialect,
    options: StaticMediaEvaluationOptions,
) -> (String, usize) {
    let lexed = lex(source, dialect);
    let tokens = lexed.tokens();
    let mut replacements = Vec::new();
    let mut index = 0;

    while index < tokens.len() {
        match tokens[index].kind {
            SyntaxKind::AtKeyword if tokens[index].text.eq_ignore_ascii_case("@media") => {
                let Some((block_start_index, block_end_index)) =
                    at_rule_block_indexes(tokens, index)
                else {
                    index += 1;
                    continue;
                };
                let condition = normalize_ascii_whitespace(
                    source[token_end(&tokens[index])..token_start(&tokens[block_start_index])]
                        .trim(),
                )
                .to_ascii_lowercase();
                let replacement = match evaluate_static_media_condition(&condition, options) {
                    StaticMediaEvalVerdict::AlwaysTrue => {
                        source[token_end(&tokens[block_start_index])
                            ..token_start(&tokens[block_end_index])]
                            .trim()
                            .to_string()
                    }
                    StaticMediaEvalVerdict::AlwaysFalse => String::new(),
                    StaticMediaEvalVerdict::Unknown => {
                        let original_condition = source
                            [token_end(&tokens[index])..token_start(&tokens[block_start_index])]
                            .trim();
                        if let Some(normalized_condition) =
                            normalize_simple_media_range_features(original_condition)
                        {
                            replacements.push((
                                token_end(&tokens[index]),
                                token_start(&tokens[block_start_index]),
                                format!(" {normalized_condition} "),
                            ));
                            index = block_end_index + 1;
                            continue;
                        }
                        index += 1;
                        continue;
                    }
                };
                replacements.push((
                    token_start(&tokens[index]),
                    token_end(&tokens[block_end_index]),
                    replacement,
                ));
                index = block_end_index + 1;
                continue;
            }
            _ => {}
        }
        index += 1;
    }

    if replacements.is_empty() {
        return (source.to_string(), 0);
    }

    let mut output = String::with_capacity(source.len());
    let mut cursor = 0;
    for (start, end, replacement) in &replacements {
        if *start > cursor {
            output.push_str(&source[cursor..*start]);
        }
        output.push_str(replacement);
        cursor = *end;
    }
    if cursor < source.len() {
        output.push_str(&source[cursor..]);
    }

    (output, replacements.len())
}

fn normalize_simple_media_range_features(condition: &str) -> Option<String> {
    let mut output = String::with_capacity(condition.len());
    let mut cursor = 0usize;
    let mut changed = false;

    while let Some(open_offset) = condition[cursor..].find('(') {
        let open_index = cursor + open_offset;
        let Some(close_index) = matching_function_call_end(condition, open_index) else {
            break;
        };
        let feature = &condition[open_index + 1..close_index];

        output.push_str(&condition[cursor..open_index]);
        if let Some(normalized_feature) = normalize_simple_media_range_feature(feature) {
            output.push('(');
            output.push_str(&normalized_feature);
            output.push(')');
            changed = true;
        } else {
            output.push_str(&condition[open_index..=close_index]);
        }
        cursor = close_index + 1;
    }

    output.push_str(&condition[cursor..]);
    changed.then_some(output)
}

fn normalize_simple_media_range_feature(feature: &str) -> Option<String> {
    let (name, value) = feature.split_once(':')?;
    let name = name.trim().to_ascii_lowercase();
    let value = normalize_static_media_range_value(value.trim());
    if !is_simple_media_range_value(&value) {
        return None;
    }

    let (dimension, operator) = match name.as_str() {
        "min-width" => ("width", ">="),
        "max-width" => ("width", "<="),
        "min-height" => ("height", ">="),
        "max-height" => ("height", "<="),
        _ => return None,
    };

    Some(format!("{dimension}{operator}{value}"))
}

fn normalize_static_media_range_value(value: &str) -> Cow<'_, str> {
    substitute_static_css_function_references_in_value_until_stable(
        value,
        &[
            ("calc", parse_reducible_calc_value),
            ("min", parse_reducible_min_value),
            ("max", parse_reducible_max_value),
            ("clamp", parse_reducible_clamp_value),
        ],
    )
    .map(Cow::Owned)
    .unwrap_or(Cow::Borrowed(value))
}

fn is_simple_media_range_value(value: &str) -> bool {
    !value.is_empty()
        && value.bytes().any(|byte| byte.is_ascii_digit())
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'+' | b'-' | b'%'))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StaticMediaEvalVerdict {
    AlwaysTrue,
    AlwaysFalse,
    Unknown,
}

fn evaluate_static_media_condition(
    condition: &str,
    options: StaticMediaEvaluationOptions,
) -> StaticMediaEvalVerdict {
    if let Some(parts) = parse_static_media_query_list(condition) {
        let verdicts = parts
            .iter()
            .map(|part| evaluate_static_media_condition(part, options))
            .collect::<Vec<_>>();
        if verdicts.contains(&StaticMediaEvalVerdict::AlwaysTrue) {
            return StaticMediaEvalVerdict::AlwaysTrue;
        }
        if verdicts
            .iter()
            .all(|verdict| *verdict == StaticMediaEvalVerdict::AlwaysFalse)
        {
            return StaticMediaEvalVerdict::AlwaysFalse;
        }
        return StaticMediaEvalVerdict::Unknown;
    }

    if let Some(parts) = parse_static_media_conjunction(condition) {
        let verdicts = parts
            .iter()
            .map(|part| evaluate_static_media_condition(part, options))
            .collect::<Vec<_>>();
        if verdicts.contains(&StaticMediaEvalVerdict::AlwaysFalse) {
            return StaticMediaEvalVerdict::AlwaysFalse;
        }
        if verdicts
            .iter()
            .all(|verdict| *verdict == StaticMediaEvalVerdict::AlwaysTrue)
        {
            return StaticMediaEvalVerdict::AlwaysTrue;
        }
        if static_media_conjunction_is_impossible(&parts) {
            return StaticMediaEvalVerdict::AlwaysFalse;
        }
        return StaticMediaEvalVerdict::Unknown;
    }

    if let Some(negated_condition) = parse_static_media_negation(condition) {
        let direct_verdict = evaluate_static_media_condition(negated_condition, options);
        if direct_verdict != StaticMediaEvalVerdict::Unknown {
            return invert_static_media_verdict(direct_verdict);
        }
        if let Some(unwrapped_condition) =
            strip_wrapping_media_condition_parentheses(negated_condition)
        {
            return invert_static_media_verdict(evaluate_static_media_condition(
                unwrapped_condition,
                options,
            ));
        }
        return StaticMediaEvalVerdict::Unknown;
    }

    match condition {
        "all" => StaticMediaEvalVerdict::AlwaysTrue,
        "not all" => StaticMediaEvalVerdict::AlwaysFalse,
        "(max-width: 0px)" | "(max-height: 0px)" | "(width<=0px)" | "(height<=0px)" => {
            StaticMediaEvalVerdict::AlwaysFalse
        }
        "(prefers-color-scheme: dark)" if options.drop_dark_mode_media_queries => {
            StaticMediaEvalVerdict::AlwaysFalse
        }
        _ => StaticMediaEvalVerdict::Unknown,
    }
}

fn parse_static_media_negation(condition: &str) -> Option<&str> {
    media_keyword_at(condition, 0, "not")
        .then(|| condition["not".len()..].trim())
        .filter(|condition| !condition.is_empty())
}

fn invert_static_media_verdict(verdict: StaticMediaEvalVerdict) -> StaticMediaEvalVerdict {
    match verdict {
        StaticMediaEvalVerdict::AlwaysTrue => StaticMediaEvalVerdict::AlwaysFalse,
        StaticMediaEvalVerdict::AlwaysFalse => StaticMediaEvalVerdict::AlwaysTrue,
        StaticMediaEvalVerdict::Unknown => StaticMediaEvalVerdict::Unknown,
    }
}

fn strip_wrapping_media_condition_parentheses(condition: &str) -> Option<&str> {
    let condition = condition.trim();
    if !condition.starts_with('(') || !condition.ends_with(')') {
        return None;
    }
    (matching_function_call_end(condition, 0)? == condition.len() - 1)
        .then(|| condition[1..condition.len() - 1].trim())
        .filter(|condition| !condition.is_empty())
}

fn parse_static_media_query_list(condition: &str) -> Option<Vec<&str>> {
    parse_static_media_top_level_parts(condition, ",")
}

fn parse_static_media_conjunction(condition: &str) -> Option<Vec<&str>> {
    parse_static_media_top_level_parts(condition, "and")
}

#[derive(Debug, Clone, PartialEq)]
struct StaticMediaRangeBound {
    value: f64,
    unit: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StaticMediaRangeBoundKind {
    Lower,
    Upper,
}

fn static_media_conjunction_is_impossible(parts: &[&str]) -> bool {
    let mut width = StaticMediaRangeConstraint::default();
    let mut height = StaticMediaRangeConstraint::default();

    for part in parts {
        let Some((dimension, kind, bound)) = parse_static_media_range_bound(part) else {
            continue;
        };
        let constraint = match dimension {
            "width" => &mut width,
            "height" => &mut height,
            _ => continue,
        };
        constraint.apply(kind, bound);
        if constraint.is_impossible() {
            return true;
        }
    }

    false
}

#[derive(Debug, Default, Clone, PartialEq)]
struct StaticMediaRangeConstraint {
    lower: Option<StaticMediaRangeBound>,
    upper: Option<StaticMediaRangeBound>,
}

impl StaticMediaRangeConstraint {
    fn apply(&mut self, kind: StaticMediaRangeBoundKind, bound: StaticMediaRangeBound) {
        match kind {
            StaticMediaRangeBoundKind::Lower => {
                if self.lower.as_ref().is_none_or(|existing| {
                    existing.unit == bound.unit && existing.value < bound.value
                }) {
                    self.lower = Some(bound);
                }
            }
            StaticMediaRangeBoundKind::Upper => {
                if self.upper.as_ref().is_none_or(|existing| {
                    existing.unit == bound.unit && existing.value > bound.value
                }) {
                    self.upper = Some(bound);
                }
            }
        }
    }

    fn is_impossible(&self) -> bool {
        let Some(lower) = &self.lower else {
            return false;
        };
        let Some(upper) = &self.upper else {
            return false;
        };
        lower.unit == upper.unit && lower.value > upper.value
    }
}

fn parse_static_media_range_bound(
    condition: &str,
) -> Option<(
    &'static str,
    StaticMediaRangeBoundKind,
    StaticMediaRangeBound,
)> {
    let condition = strip_wrapping_media_condition_parentheses(condition).unwrap_or(condition);
    if let Some((name, value)) = condition.split_once(':') {
        let (dimension, kind) = match name.trim().to_ascii_lowercase().as_str() {
            "min-width" => ("width", StaticMediaRangeBoundKind::Lower),
            "max-width" => ("width", StaticMediaRangeBoundKind::Upper),
            "min-height" => ("height", StaticMediaRangeBoundKind::Lower),
            "max-height" => ("height", StaticMediaRangeBoundKind::Upper),
            _ => return None,
        };
        return parse_static_media_range_bound_value(value.trim())
            .map(|bound| (dimension, kind, bound));
    }

    for (operator, kind) in [
        (">=", StaticMediaRangeBoundKind::Lower),
        ("<=", StaticMediaRangeBoundKind::Upper),
    ] {
        let Some((dimension, value)) = condition.split_once(operator) else {
            continue;
        };
        let dimension = match dimension.trim().to_ascii_lowercase().as_str() {
            "width" => "width",
            "height" => "height",
            _ => continue,
        };
        return parse_static_media_range_bound_value(value.trim())
            .map(|bound| (dimension, kind, bound));
    }

    None
}

fn parse_static_media_range_bound_value(value: &str) -> Option<StaticMediaRangeBound> {
    let value = normalize_static_media_range_value(value);
    let parsed = parse_numeric_value_with_unit(value.as_ref())?;
    Some(StaticMediaRangeBound {
        value: parsed.value,
        unit: parsed.unit.to_string(),
    })
}

fn parse_static_media_top_level_parts<'a>(
    condition: &'a str,
    separator: &str,
) -> Option<Vec<&'a str>> {
    let mut parts = Vec::new();
    let mut depth = 0usize;
    let mut last_start = 0usize;
    let mut index = 0usize;
    let mut found_separator = false;

    while index < condition.len() {
        let ch = condition[index..].chars().next()?;
        match ch {
            '(' => {
                depth += 1;
                index += ch.len_utf8();
            }
            ')' => {
                depth = depth.saturating_sub(1);
                index += ch.len_utf8();
            }
            ',' if separator == "," && depth == 0 => {
                let part = condition[last_start..index].trim();
                if part.is_empty() {
                    return None;
                }
                parts.push(part);
                index += ch.len_utf8();
                last_start = index;
                found_separator = true;
            }
            _ if separator == "and" && depth == 0 && media_keyword_at(condition, index, "and") => {
                let part = condition[last_start..index].trim();
                if part.is_empty() {
                    return None;
                }
                parts.push(part);
                index += "and".len();
                last_start = index;
                found_separator = true;
            }
            _ => {
                index += ch.len_utf8();
            }
        }
    }

    if !found_separator {
        return None;
    }
    let part = condition[last_start..].trim();
    if part.is_empty() {
        return None;
    }
    parts.push(part);
    Some(parts)
}

fn media_keyword_at(text: &str, index: usize, keyword: &str) -> bool {
    text[index..]
        .get(..keyword.len())
        .is_some_and(|candidate| candidate == keyword)
        && text[..index]
            .chars()
            .next_back()
            .is_none_or(|ch| !is_media_ident_char(ch))
        && text[index + keyword.len()..]
            .chars()
            .next()
            .is_none_or(|ch| !is_media_ident_char(ch))
}

fn is_media_ident_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '-'
}
