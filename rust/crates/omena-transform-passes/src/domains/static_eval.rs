use std::borrow::Cow;

use omena_cascade::{
    StaticSupportsAssumptionV0, StaticSupportsEvalVerdictV0, StaticSupportsEvalWitnessV0,
    evaluate_static_supports_condition,
};
use omena_parser::StyleDialect;
use omena_syntax::SyntaxKind;

use crate::runtime::lex_cache::lex_cached as lex;

use crate::{
    domains::number::{
        compress_number_prefix, format_css_number, parse_numeric_value_with_unit,
        parse_reducible_abs_value, parse_reducible_calc_value, parse_reducible_clamp_value,
        parse_reducible_exp_value, parse_reducible_hypot_value, parse_reducible_log_value,
        parse_reducible_max_value, parse_reducible_min_value, parse_reducible_mod_value,
        parse_reducible_pow_value, parse_reducible_rem_value, parse_reducible_round_value,
        parse_reducible_sign_value, parse_reducible_sqrt_value,
    },
    helpers::{
        ascii::normalize_ascii_whitespace,
        blocks::at_rule_block_indexes,
        tokens::{token_end, token_start},
        values::{
            matching_function_call_end,
            substitute_static_css_function_references_in_value_until_stable,
        },
    },
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct StaticSupportsProofCandidateV0 {
    pub(crate) source_span_start: usize,
    pub(crate) source_span_end: usize,
    pub(crate) witness: StaticSupportsEvalWitnessV0,
}

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

pub(crate) fn collect_static_supports_proof_candidates_with_lexer(
    source: &str,
    dialect: StyleDialect,
) -> Vec<StaticSupportsProofCandidateV0> {
    let lexed = lex(source, dialect);
    let tokens = lexed.tokens();
    let mut candidates = Vec::new();
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
                candidates.push(StaticSupportsProofCandidateV0 {
                    source_span_start: token_start(&tokens[index]),
                    source_span_end: token_end(&tokens[block_end_index]),
                    witness: evaluate_static_supports_condition(
                        condition,
                        StaticSupportsAssumptionV0::ModernBrowser,
                    ),
                });
                index = block_end_index + 1;
                continue;
            }
            _ => {}
        }
        index += 1;
    }

    candidates
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
            changed |= normalized_feature != feature;
        } else {
            output.push_str(&condition[open_index..=close_index]);
        }
        cursor = close_index + 1;
    }

    output.push_str(&condition[cursor..]);
    changed.then_some(output)
}

fn normalize_simple_media_range_feature(feature: &str) -> Option<String> {
    if let Some((name, value)) = feature.split_once(':') {
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

        return Some(format!("{dimension}{operator}{value}"));
    }

    if let Some(normalized) = normalize_chained_static_media_range_comparison(feature) {
        return Some(normalized);
    }

    normalize_static_media_range_comparison(feature)
}

fn normalize_chained_static_media_range_comparison(feature: &str) -> Option<String> {
    let (first_operator_index, first_operator) = find_static_media_range_operator(feature, 0)?;
    let first_right_index = first_operator_index + first_operator.len();
    let (second_operator_index, second_operator) =
        find_static_media_range_operator(feature, first_right_index)?;

    let left = feature[..first_operator_index].trim();
    let middle = feature[first_right_index..second_operator_index].trim();
    let right = feature[second_operator_index + second_operator.len()..].trim();
    let dimension = static_media_dimension_name(middle)?;
    let left_value = normalize_static_media_range_value(left);
    let right_value = normalize_static_media_range_value(right);
    if !is_simple_media_range_value(&left_value) || !is_simple_media_range_value(&right_value) {
        return None;
    }

    Some(format!(
        "{dimension}{}{left_value}) and ({dimension}{second_operator}{right_value}",
        reverse_static_media_range_operator(first_operator)
    ))
}

fn normalize_static_media_range_comparison(feature: &str) -> Option<String> {
    for operator in ["<=", ">=", "<", ">", "="] {
        let Some((left, right)) = feature.split_once(operator) else {
            continue;
        };
        let left = left.trim();
        let right = right.trim();
        if let Some(dimension) = static_media_dimension_name(left) {
            let value = normalize_static_media_range_value(right);
            if is_simple_media_range_value(&value) {
                return Some(format!("{dimension}{operator}{value}"));
            }
            return None;
        }
        if let Some(dimension) = static_media_dimension_name(right) {
            let value = normalize_static_media_range_value(left);
            if is_simple_media_range_value(&value) {
                return Some(format!(
                    "{dimension}{}{value}",
                    reverse_static_media_range_operator(operator)
                ));
            }
            return None;
        }
    }

    None
}

fn find_static_media_range_operator(text: &str, start: usize) -> Option<(usize, &'static str)> {
    for (offset, _) in text.get(start..)?.char_indices() {
        let index = start + offset;
        if let Some(operator) = static_media_range_operator_at(text, index) {
            return Some((index, operator));
        }
    }
    None
}

fn static_media_range_operator_at(text: &str, index: usize) -> Option<&'static str> {
    for operator in ["<=", ">=", "<", ">", "="] {
        if text[index..].starts_with(operator) {
            return Some(operator);
        }
    }
    None
}

fn static_media_dimension_name(text: &str) -> Option<&'static str> {
    match text.trim().to_ascii_lowercase().as_str() {
        "width" => Some("width"),
        "height" => Some("height"),
        _ => None,
    }
}

fn reverse_static_media_range_operator(operator: &str) -> &'static str {
    match operator {
        "<=" => ">=",
        ">=" => "<=",
        "<" => ">",
        ">" => "<",
        "=" => "=",
        _ => "",
    }
}

fn normalize_static_media_range_value(value: &str) -> Cow<'_, str> {
    let substituted = substitute_static_css_function_references_in_value_until_stable(
        value,
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
    )
    .map(Cow::Owned)
    .unwrap_or(Cow::Borrowed(value));

    if let Some(normalized) = normalize_static_media_numeric_value(substituted.as_ref())
        && normalized != substituted.as_ref()
    {
        return Cow::Owned(normalized);
    }
    substituted
}

fn normalize_static_media_numeric_value(value: &str) -> Option<String> {
    let parsed = parse_numeric_value_with_unit(value)?;
    if parsed.unit.is_empty() {
        return None;
    }
    let unit = parsed.unit.to_ascii_lowercase();
    if unit != "%" && !unit.chars().all(|ch| ch.is_ascii_alphabetic()) {
        return None;
    }

    Some(format!(
        "{}{unit}",
        compress_number_prefix(&format_css_number(parsed.value))
    ))
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

    if let Some(only_condition) = parse_static_media_only(condition) {
        return evaluate_static_media_condition(only_condition, options);
    }

    if let Some(parts) = parse_static_media_disjunction(condition) {
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

    match condition {
        "all" => StaticMediaEvalVerdict::AlwaysTrue,
        "not all" => StaticMediaEvalVerdict::AlwaysFalse,
        "(max-width: 0px)" | "(max-height: 0px)" | "(width<=0px)" | "(height<=0px)"
        | "(width<0px)" | "(height<0px)" => StaticMediaEvalVerdict::AlwaysFalse,
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

fn parse_static_media_only(condition: &str) -> Option<&str> {
    media_keyword_at(condition, 0, "only")
        .then(|| condition["only".len()..].trim())
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

fn parse_static_media_disjunction(condition: &str) -> Option<Vec<&str>> {
    parse_static_media_top_level_parts(condition, "or")
}

fn parse_static_media_conjunction(condition: &str) -> Option<Vec<&str>> {
    parse_static_media_top_level_parts(condition, "and")
}

#[derive(Debug, Clone, PartialEq)]
struct StaticMediaRangeBound {
    value: f64,
    unit: String,
    inclusive: bool,
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
        if let Some((dimension, bound)) = parse_static_media_range_equality(part) {
            let constraint = match dimension {
                "width" => &mut width,
                "height" => &mut height,
                _ => continue,
            };
            constraint.apply(StaticMediaRangeBoundKind::Lower, bound.clone());
            constraint.apply(StaticMediaRangeBoundKind::Upper, bound);
            if constraint.is_impossible() {
                return true;
            }
            continue;
        }

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
                    existing.unit == bound.unit
                        && (existing.value < bound.value
                            || (existing.value == bound.value
                                && existing.inclusive
                                && !bound.inclusive))
                }) {
                    self.lower = Some(bound);
                }
            }
            StaticMediaRangeBoundKind::Upper => {
                if self.upper.as_ref().is_none_or(|existing| {
                    existing.unit == bound.unit
                        && (existing.value > bound.value
                            || (existing.value == bound.value
                                && existing.inclusive
                                && !bound.inclusive))
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
        lower.unit == upper.unit
            && (lower.value > upper.value
                || (lower.value == upper.value && (!lower.inclusive || !upper.inclusive)))
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
        return parse_static_media_range_bound_value(value.trim(), true)
            .map(|bound| (dimension, kind, bound));
    }

    for (operator, kind, inclusive) in [
        (">=", StaticMediaRangeBoundKind::Lower, true),
        ("<=", StaticMediaRangeBoundKind::Upper, true),
        (">", StaticMediaRangeBoundKind::Lower, false),
        ("<", StaticMediaRangeBoundKind::Upper, false),
    ] {
        let Some((left, right)) = condition.split_once(operator) else {
            continue;
        };
        if let Some(dimension) = static_media_dimension_name(left) {
            return parse_static_media_range_bound_value(right.trim(), inclusive)
                .map(|bound| (dimension, kind, bound));
        }
        if let Some(dimension) = static_media_dimension_name(right) {
            let reverse_kind = match kind {
                StaticMediaRangeBoundKind::Lower => StaticMediaRangeBoundKind::Upper,
                StaticMediaRangeBoundKind::Upper => StaticMediaRangeBoundKind::Lower,
            };
            return parse_static_media_range_bound_value(left.trim(), inclusive)
                .map(|bound| (dimension, reverse_kind, bound));
        }
    }

    None
}

fn parse_static_media_range_equality(
    condition: &str,
) -> Option<(&'static str, StaticMediaRangeBound)> {
    let condition = strip_wrapping_media_condition_parentheses(condition).unwrap_or(condition);
    if condition.contains("<=")
        || condition.contains(">=")
        || condition.contains('<')
        || condition.contains('>')
    {
        return None;
    }
    let (left, right) = condition.split_once('=')?;
    if right.contains('=') {
        return None;
    }
    if let Some(dimension) = static_media_dimension_name(left) {
        return parse_static_media_range_bound_value(right.trim(), true)
            .map(|bound| (dimension, bound));
    }
    if let Some(dimension) = static_media_dimension_name(right) {
        return parse_static_media_range_bound_value(left.trim(), true)
            .map(|bound| (dimension, bound));
    }
    None
}

fn parse_static_media_range_bound_value(
    value: &str,
    inclusive: bool,
) -> Option<StaticMediaRangeBound> {
    let value = normalize_static_media_range_value(value);
    let parsed = parse_numeric_value_with_unit(value.as_ref())?;
    Some(StaticMediaRangeBound {
        value: parsed.value,
        unit: parsed.unit.to_string(),
        inclusive,
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
            _ if matches!(separator, "and" | "or")
                && depth == 0
                && media_keyword_at(condition, index, separator) =>
            {
                let part = condition[last_start..index].trim();
                if part.is_empty() {
                    return None;
                }
                parts.push(part);
                index += separator.len();
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
