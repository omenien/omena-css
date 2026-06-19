use std::collections::BTreeMap;

use omena_abstract_value::{
    AbstractCssValueV0, abstract_css_value_from_text, join_abstract_css_values,
};
use omena_parser::{LexedToken, StyleDialect, lex};
use omena_syntax::SyntaxKind;

use crate::{
    static_loop_frames::parse_static_scss_each_loop_binding_frames,
    value_eval::reduce_static_scss_value,
};

use super::{
    header_values::{scss_header_value_from_bindings, single_static_scss_header_value_text},
    model::OmenaScssEvalControlFlowBlockV0,
    tokens::next_non_trivia_token_index,
    transfer::ScssControlFlowBindingValue,
    variables::{
        canonical_scss_variable_name, insert_static_scss_binding, static_scss_binding_value,
        variable_name_end, variable_names_in_text, variable_names_in_text_preserving_order,
    },
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ScssControlFlowLoopContext {
    pub(super) header_text: String,
    pub(super) body_text: Option<String>,
}

pub(super) fn loop_carried_bindings(header: &str) -> Vec<String> {
    let separator = if header
        .split_whitespace()
        .any(|part| part.eq_ignore_ascii_case("from"))
    {
        "from"
    } else {
        "in"
    };
    let before_separator = split_header_at_keyword(header, separator)
        .map(|(left, _)| left)
        .unwrap_or(header);
    variable_names_in_text_preserving_order(before_separator)
}

pub(super) fn split_header_at_keyword<'a>(
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
        if header_keyword_has_boundaries(header, index, right_start) {
            let left = header.get(..index)?;
            let right = header.get(right_start..)?;
            return Some((left, right));
        }
        search_start = right_start;
    }
    None
}

pub(super) fn while_loop_body_assignment_names(
    source: &str,
    block: &OmenaScssEvalControlFlowBlockV0,
) -> Vec<String> {
    let Some(body) = control_flow_block_body_text(source, block) else {
        return Vec::new();
    };
    let lexed = lex(body, StyleDialect::Scss);
    let tokens = lexed.tokens();
    let mut names: Vec<String> = Vec::new();
    for (index, token) in tokens.iter().enumerate() {
        if token.kind != SyntaxKind::ScssVariable {
            continue;
        }
        let Some(colon_index) = next_non_trivia_token_index(tokens, index + 1) else {
            continue;
        };
        if !matches!(
            tokens[colon_index].kind,
            SyntaxKind::Colon | SyntaxKind::PlusEquals | SyntaxKind::MinusEquals
        ) {
            continue;
        }
        let name = token.text.to_string();
        if !names.iter().any(|existing| {
            canonical_scss_variable_name(existing.as_str())
                == canonical_scss_variable_name(name.as_str())
        }) {
            names.push(name);
        }
    }
    names
}

pub(super) fn loop_carried_value(
    header: &str,
    lexical_bindings: &BTreeMap<String, AbstractCssValueV0>,
) -> AbstractCssValueV0 {
    parse_static_for_loop_range(header, lexical_bindings)
        .or_else(|| parse_static_each_loop_source_value(header, lexical_bindings))
        .unwrap_or_else(|| scss_header_value_from_bindings(header, lexical_bindings))
}

pub(super) fn loop_carried_binding_values(
    header: &str,
    lexical_bindings: &BTreeMap<String, AbstractCssValueV0>,
) -> Vec<ScssControlFlowBindingValue> {
    if let Some(values) = static_each_loop_binding_values(header, lexical_bindings) {
        return values;
    }
    let value = loop_carried_value(header, lexical_bindings);
    loop_carried_bindings(header)
        .into_iter()
        .map(|name| ScssControlFlowBindingValue {
            name,
            value: value.clone(),
        })
        .collect()
}

pub(super) fn loop_carried_binding_frames_for_contexts(
    contexts: &[ScssControlFlowLoopContext],
    lexical_bindings: &BTreeMap<String, AbstractCssValueV0>,
) -> Option<Vec<Vec<ScssControlFlowBindingValue>>> {
    if contexts.is_empty() {
        return None;
    }

    let mut frames = vec![Vec::<ScssControlFlowBindingValue>::new()];
    for context in contexts {
        let mut next_frames = Vec::new();
        for frame in frames {
            let mut frame_bindings = lexical_bindings.clone();
            for binding in &frame {
                insert_static_scss_binding(
                    &mut frame_bindings,
                    binding.name.as_str(),
                    binding.value.clone(),
                );
            }
            let header_frames = loop_carried_binding_frames(
                context.header_text.as_str(),
                context.body_text.as_deref(),
                &frame_bindings,
            )?;
            for header_frame in header_frames {
                let mut combined = frame.clone();
                combined.extend(header_frame);
                next_frames.push(combined);
                if next_frames.len() > 64 {
                    return None;
                }
            }
        }
        frames = next_frames;
    }

    Some(frames)
}

pub(super) fn while_loop_carried_binding_values(
    source: &str,
    block: &OmenaScssEvalControlFlowBlockV0,
    lexical_bindings: &BTreeMap<String, AbstractCssValueV0>,
) -> Vec<ScssControlFlowBindingValue> {
    let assigned_bindings = while_loop_body_assignment_names(source, block);
    let body_text = control_flow_block_body_text(source, block);
    while_loop_carried_binding_values_from_parts(
        block.header_text.as_str(),
        lexical_bindings,
        assigned_bindings.as_slice(),
        body_text,
    )
}

fn while_loop_carried_binding_values_from_parts(
    header: &str,
    lexical_bindings: &BTreeMap<String, AbstractCssValueV0>,
    assigned_bindings: &[String],
    body_text: Option<&str>,
) -> Vec<ScssControlFlowBindingValue> {
    let binding_names = while_loop_binding_names(header, assigned_bindings);
    let step = binding_names.first().and_then(|binding_name| {
        body_text.and_then(|body| while_loop_body_assignment_step(body, binding_name))
    });
    if let Some(values) = static_while_condition_loop_binding_values(
        header,
        lexical_bindings,
        binding_names.as_slice(),
        step,
    ) {
        return values;
    }
    binding_names
        .into_iter()
        .map(|name| ScssControlFlowBindingValue {
            name,
            value: AbstractCssValueV0::Top,
        })
        .collect()
}

fn header_keyword_has_boundaries(header: &str, start: usize, end: usize) -> bool {
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

fn control_flow_block_body_text<'a>(
    source: &'a str,
    block: &OmenaScssEvalControlFlowBlockV0,
) -> Option<&'a str> {
    let block_text = source.get(block.source_span_start..block.source_span_end)?;
    let open = block_text.find('{')?;
    let close = block_text.rfind('}')?;
    (open < close)
        .then(|| block_text.get(open + '{'.len_utf8()..close))
        .flatten()
}

fn while_loop_body_assignment_step(body: &str, binding_name: &str) -> Option<i32> {
    let lexed = lex(body, StyleDialect::Scss);
    let tokens = lexed.tokens();
    let mut brace_depth = 0usize;
    let mut step = None;
    for (index, token) in tokens.iter().enumerate() {
        match token.kind {
            SyntaxKind::LeftBrace => {
                brace_depth += 1;
                continue;
            }
            SyntaxKind::RightBrace => {
                brace_depth = brace_depth.checked_sub(1)?;
                continue;
            }
            _ => {}
        }
        if brace_depth != 0
            || token.kind != SyntaxKind::ScssVariable
            || canonical_scss_variable_name(token.text.as_str())
                != canonical_scss_variable_name(binding_name)
        {
            continue;
        }
        let Some(operator_index) = next_non_trivia_token_index(tokens, index + 1) else {
            continue;
        };
        step = match tokens[operator_index].kind {
            SyntaxKind::PlusEquals => {
                static_while_integer_token_after(tokens, operator_index).or(step)
            }
            SyntaxKind::MinusEquals => static_while_integer_token_after(tokens, operator_index)
                .map(i32::saturating_neg)
                .or(step),
            SyntaxKind::Colon => {
                static_while_self_assignment_step(tokens, operator_index, binding_name).or(step)
            }
            _ => step,
        };
    }
    step
}

fn static_while_self_assignment_step(
    tokens: &[LexedToken],
    colon_index: usize,
    binding_name: &str,
) -> Option<i32> {
    let variable_index = next_non_trivia_token_index(tokens, colon_index + 1)?;
    let variable = tokens.get(variable_index)?;
    if variable.kind != SyntaxKind::ScssVariable
        || canonical_scss_variable_name(variable.text.as_str())
            != canonical_scss_variable_name(binding_name)
    {
        return None;
    }
    let operator_index = next_non_trivia_token_index(tokens, variable_index + 1)?;
    match tokens.get(operator_index)?.kind {
        SyntaxKind::Plus => static_while_integer_token_after(tokens, operator_index),
        SyntaxKind::Minus => {
            static_while_integer_token_after(tokens, operator_index).map(i32::saturating_neg)
        }
        _ => None,
    }
}

fn static_while_integer_token_after(tokens: &[LexedToken], operator_index: usize) -> Option<i32> {
    let value_index = next_non_trivia_token_index(tokens, operator_index + 1)?;
    let value = tokens.get(value_index)?;
    (value.kind == SyntaxKind::Number)
        .then(|| value.text.parse::<i32>().ok())
        .flatten()
}

fn loop_carried_binding_frames(
    header: &str,
    body_text: Option<&str>,
    lexical_bindings: &BTreeMap<String, AbstractCssValueV0>,
) -> Option<Vec<Vec<ScssControlFlowBindingValue>>> {
    if header
        .trim_start()
        .to_ascii_lowercase()
        .starts_with("@while")
    {
        return static_while_loop_binding_frames(header, body_text, lexical_bindings);
    }
    static_for_loop_binding_frames(header, lexical_bindings)
        .or_else(|| static_each_loop_binding_frames(header, lexical_bindings))
        .or_else(|| static_while_loop_binding_frames(header, body_text, lexical_bindings))
}

fn static_for_loop_binding_frames(
    header: &str,
    lexical_bindings: &BTreeMap<String, AbstractCssValueV0>,
) -> Option<Vec<Vec<ScssControlFlowBindingValue>>> {
    let bindings = loop_carried_bindings(header);
    if bindings.len() != 1 {
        return None;
    }
    let parts = header.split_whitespace().collect::<Vec<_>>();
    let from_index = parts
        .iter()
        .position(|part| part.eq_ignore_ascii_case("from"))?;
    let to_index = parts
        .iter()
        .position(|part| part.eq_ignore_ascii_case("to") || part.eq_ignore_ascii_case("through"))?;
    let includes_end = parts[to_index].eq_ignore_ascii_case("through");
    let start = parse_static_for_loop_bound(parts.get(from_index + 1)?, lexical_bindings)?;
    let end = parse_static_for_loop_bound(parts.get(to_index + 1)?, lexical_bindings)?;
    if start > end {
        return None;
    }
    let value_count = if includes_end {
        i64::from(end) - i64::from(start) + 1
    } else {
        i64::from(end) - i64::from(start)
    };
    if !(0..=64).contains(&value_count) {
        return None;
    }
    let last = if includes_end {
        end
    } else {
        end.saturating_sub(1)
    };
    let frames = if value_count == 0 {
        Vec::new()
    } else {
        (start..=last)
            .map(|value| {
                vec![ScssControlFlowBindingValue {
                    name: bindings[0].clone(),
                    value: abstract_css_value_from_text(value.to_string().as_str()),
                }]
            })
            .collect()
    };
    Some(frames)
}

fn static_while_loop_binding_frames(
    header: &str,
    body_text: Option<&str>,
    lexical_bindings: &BTreeMap<String, AbstractCssValueV0>,
) -> Option<Vec<Vec<ScssControlFlowBindingValue>>> {
    let bindings = loop_carried_bindings(header);
    if bindings.len() != 1 {
        return None;
    }
    let step =
        body_text.and_then(|body| while_loop_body_assignment_step(body, bindings[0].as_str()));
    let values = static_while_condition_loop_binding_values(
        header,
        lexical_bindings,
        bindings.as_slice(),
        step,
    )?;
    if values.len() != 1 {
        return None;
    }
    let binding = values.into_iter().next()?;
    match binding.value {
        AbstractCssValueV0::Bottom => Some(Vec::new()),
        AbstractCssValueV0::FiniteSet { values } => Some(
            values
                .into_iter()
                .map(|value| {
                    vec![ScssControlFlowBindingValue {
                        name: binding.name.clone(),
                        value: abstract_css_value_from_text(value.as_str()),
                    }]
                })
                .collect(),
        ),
        AbstractCssValueV0::Exact { .. }
        | AbstractCssValueV0::Raw { .. }
        | AbstractCssValueV0::Top => None,
    }
}

fn static_each_loop_binding_frames(
    header: &str,
    lexical_bindings: &BTreeMap<String, AbstractCssValueV0>,
) -> Option<Vec<Vec<ScssControlFlowBindingValue>>> {
    parse_static_scss_each_loop_binding_frames(header, |source| {
        static_each_source_text(source.trim(), lexical_bindings).map(str::to_string)
    })
    .map(|frames| {
        frames
            .into_iter()
            .map(|frame| {
                frame
                    .into_iter()
                    .map(|(name, value)| ScssControlFlowBindingValue {
                        name,
                        value: abstract_css_value_from_text(value.as_str()),
                    })
                    .collect()
            })
            .collect()
    })
}

fn static_each_loop_binding_values(
    header: &str,
    lexical_bindings: &BTreeMap<String, AbstractCssValueV0>,
) -> Option<Vec<ScssControlFlowBindingValue>> {
    let frames = static_each_loop_binding_frames(header, lexical_bindings)?;
    if frames.len() <= 1 {
        return None;
    }

    let mut values = Vec::<ScssControlFlowBindingValue>::new();
    for frame in frames {
        for binding in frame {
            if let Some(existing) = values.iter_mut().find(|existing| {
                canonical_scss_variable_name(existing.name.as_str())
                    == canonical_scss_variable_name(binding.name.as_str())
            }) {
                existing.value = join_abstract_css_values(&existing.value, &binding.value);
            } else {
                values.push(binding);
            }
        }
    }
    (!values.is_empty()).then_some(values)
}

fn while_loop_binding_names(header: &str, assigned_bindings: &[String]) -> Vec<String> {
    let header_bindings = loop_carried_bindings(header);
    if assigned_bindings.is_empty() {
        return header_bindings;
    }
    let filtered = header_bindings
        .iter()
        .filter(|name| {
            assigned_bindings.iter().any(|assigned| {
                canonical_scss_variable_name(name) == canonical_scss_variable_name(assigned)
            })
        })
        .cloned()
        .collect::<Vec<_>>();
    if filtered.is_empty() {
        header_bindings
    } else {
        filtered
    }
}

fn static_while_condition_loop_binding_values(
    header: &str,
    lexical_bindings: &BTreeMap<String, AbstractCssValueV0>,
    binding_names: &[String],
    step: Option<i32>,
) -> Option<Vec<ScssControlFlowBindingValue>> {
    if binding_names.len() != 1 {
        return None;
    }
    let binding_name = binding_names[0].as_str();
    let (left, operator, right) = split_static_while_inequality(header)?;
    let start = static_while_integer_binding_value(binding_name, lexical_bindings)?;

    let (operator, bound) = if static_scss_side_is_binding(left, binding_name) {
        (
            operator,
            static_while_integer_operand(right, lexical_bindings)?,
        )
    } else if static_scss_side_is_binding(right, binding_name) {
        (
            operator.inverted_for_right_hand_binding(),
            static_while_integer_operand(left, lexical_bindings)?,
        )
    } else {
        return None;
    };
    let value = static_while_integer_domain(start, operator, bound, step)?;

    Some(vec![ScssControlFlowBindingValue {
        name: binding_names[0].clone(),
        value,
    }])
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StaticWhileInequalityOperator {
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
}

impl StaticWhileInequalityOperator {
    const fn inverted_for_right_hand_binding(self) -> Self {
        match self {
            Self::LessThan => Self::GreaterThan,
            Self::LessThanOrEqual => Self::GreaterThanOrEqual,
            Self::GreaterThan => Self::LessThan,
            Self::GreaterThanOrEqual => Self::LessThanOrEqual,
        }
    }
}

fn split_static_while_inequality(
    value: &str,
) -> Option<(&str, StaticWhileInequalityOperator, &str)> {
    let mut comparison = None;
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
            ')' => paren_depth = paren_depth.checked_sub(1)?,
            '[' => bracket_depth += 1,
            ']' => bracket_depth = bracket_depth.checked_sub(1)?,
            '<' | '>' if paren_depth == 0 && bracket_depth == 0 => {
                let (operator, width) = static_while_inequality_operator_at(value, index)?;
                let left = value.get(..index)?.trim();
                let right = value.get(index + width..)?.trim();
                if left.is_empty() || right.is_empty() || comparison.is_some() {
                    return None;
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
        return None;
    }
    comparison
}

fn static_while_inequality_operator_at(
    value: &str,
    index: usize,
) -> Option<(StaticWhileInequalityOperator, usize)> {
    let suffix = value.get(index..)?;
    if suffix.starts_with("<=") {
        return Some((StaticWhileInequalityOperator::LessThanOrEqual, 2));
    }
    if suffix.starts_with(">=") {
        return Some((StaticWhileInequalityOperator::GreaterThanOrEqual, 2));
    }
    if suffix.starts_with('<') {
        return Some((StaticWhileInequalityOperator::LessThan, 1));
    }
    if suffix.starts_with('>') {
        return Some((StaticWhileInequalityOperator::GreaterThan, 1));
    }
    None
}

fn static_scss_side_is_binding(value: &str, binding_name: &str) -> bool {
    let value = value.trim();
    value.starts_with('$')
        && variable_name_end(value, '$'.len_utf8()) == value.len()
        && canonical_scss_variable_name(value) == canonical_scss_variable_name(binding_name)
}

fn static_while_integer_binding_value(
    binding_name: &str,
    lexical_bindings: &BTreeMap<String, AbstractCssValueV0>,
) -> Option<i32> {
    static_scss_binding_value(lexical_bindings, binding_name)
        .and_then(single_static_scss_header_value_text)
        .and_then(parse_static_while_integer_text)
}

fn static_while_integer_operand(
    value: &str,
    lexical_bindings: &BTreeMap<String, AbstractCssValueV0>,
) -> Option<i32> {
    let reduced = scss_header_value_from_bindings(value, lexical_bindings);
    single_static_scss_header_value_text(&reduced).and_then(parse_static_while_integer_text)
}

fn parse_static_while_integer_text(value: &str) -> Option<i32> {
    let reduced = reduce_static_scss_value(value.trim().to_string());
    reduced.trim().parse::<i32>().ok()
}

fn static_while_integer_domain(
    start: i32,
    operator: StaticWhileInequalityOperator,
    bound: i32,
    step: Option<i32>,
) -> Option<AbstractCssValueV0> {
    if let Some(step) = step {
        return static_while_integer_progression_domain(start, operator, bound, step);
    }

    let (first, last) = match operator {
        StaticWhileInequalityOperator::LessThan => {
            if start >= bound {
                return Some(AbstractCssValueV0::Bottom);
            }
            (start, bound.saturating_sub(1))
        }
        StaticWhileInequalityOperator::LessThanOrEqual => {
            if start > bound {
                return Some(AbstractCssValueV0::Bottom);
            }
            (start, bound)
        }
        StaticWhileInequalityOperator::GreaterThan => {
            if start <= bound {
                return Some(AbstractCssValueV0::Bottom);
            }
            (bound.saturating_add(1), start)
        }
        StaticWhileInequalityOperator::GreaterThanOrEqual => {
            if start < bound {
                return Some(AbstractCssValueV0::Bottom);
            }
            (bound, start)
        }
    };
    let value_count = i64::from(last) - i64::from(first) + 1;
    if !(1..=64).contains(&value_count) {
        return None;
    }
    Some(
        (first..=last).fold(AbstractCssValueV0::Bottom, |acc, value| {
            let value = abstract_css_value_from_text(value.to_string().as_str());
            join_abstract_css_values(&acc, &value)
        }),
    )
}

fn static_while_integer_progression_domain(
    start: i32,
    operator: StaticWhileInequalityOperator,
    bound: i32,
    step: i32,
) -> Option<AbstractCssValueV0> {
    if step == 0 {
        return None;
    }
    let mut current = start;
    let mut value = AbstractCssValueV0::Bottom;
    for _ in 0..64 {
        if !static_while_integer_condition_holds(current, operator, bound) {
            return Some(value);
        }
        value = join_abstract_css_values(
            &value,
            &abstract_css_value_from_text(current.to_string().as_str()),
        );
        current = current.checked_add(step)?;
    }
    if static_while_integer_condition_holds(current, operator, bound) {
        None
    } else {
        Some(value)
    }
}

const fn static_while_integer_condition_holds(
    value: i32,
    operator: StaticWhileInequalityOperator,
    bound: i32,
) -> bool {
    match operator {
        StaticWhileInequalityOperator::LessThan => value < bound,
        StaticWhileInequalityOperator::LessThanOrEqual => value <= bound,
        StaticWhileInequalityOperator::GreaterThan => value > bound,
        StaticWhileInequalityOperator::GreaterThanOrEqual => value >= bound,
    }
}

fn parse_static_for_loop_range(
    header: &str,
    lexical_bindings: &BTreeMap<String, AbstractCssValueV0>,
) -> Option<AbstractCssValueV0> {
    let parts = header.split_whitespace().collect::<Vec<_>>();
    let from_index = parts
        .iter()
        .position(|part| part.eq_ignore_ascii_case("from"))?;
    let to_index = parts
        .iter()
        .position(|part| part.eq_ignore_ascii_case("to") || part.eq_ignore_ascii_case("through"))?;
    let includes_end = parts[to_index].eq_ignore_ascii_case("through");
    let start = parse_static_for_loop_bound(parts.get(from_index + 1)?, lexical_bindings)?;
    let end = parse_static_for_loop_bound(parts.get(to_index + 1)?, lexical_bindings)?;
    if start > end {
        return Some(AbstractCssValueV0::Top);
    }
    let value_count = if includes_end {
        i64::from(end) - i64::from(start) + 1
    } else {
        i64::from(end) - i64::from(start)
    };
    if !(0..=64).contains(&value_count) {
        return Some(AbstractCssValueV0::Top);
    }
    if value_count == 0 {
        return Some(AbstractCssValueV0::Bottom);
    }
    let last = if includes_end {
        end
    } else {
        end.saturating_sub(1)
    };
    Some(
        (start..=last).fold(AbstractCssValueV0::Bottom, |acc, value| {
            let value = abstract_css_value_from_text(value.to_string().as_str());
            join_abstract_css_values(&acc, &value)
        }),
    )
}

fn parse_static_for_loop_bound(
    value: &str,
    lexical_bindings: &BTreeMap<String, AbstractCssValueV0>,
) -> Option<i32> {
    let reduced = match scss_header_value_from_bindings(value, lexical_bindings) {
        AbstractCssValueV0::Exact { value } | AbstractCssValueV0::Raw { value } => value,
        AbstractCssValueV0::Bottom
        | AbstractCssValueV0::Top
        | AbstractCssValueV0::FiniteSet { .. } => return None,
    };
    reduced.parse::<i32>().ok()
}

fn parse_static_each_loop_source_value(
    header: &str,
    lexical_bindings: &BTreeMap<String, AbstractCssValueV0>,
) -> Option<AbstractCssValueV0> {
    let (_, source) = split_header_at_keyword(header, "in")?;
    let source = source.trim();
    if source.is_empty() {
        return None;
    }
    let source_variables = variable_names_in_text(source);
    let source_text = if source_is_single_static_variable(source) {
        static_scss_binding_value(lexical_bindings, source)
            .and_then(single_static_scss_header_value_text)
            .unwrap_or(source)
    } else if !source_variables.is_empty() {
        return Some(
            source_variables
                .iter()
                .map(|name| {
                    static_scss_binding_value(lexical_bindings, name)
                        .cloned()
                        .unwrap_or(AbstractCssValueV0::Top)
                })
                .fold(AbstractCssValueV0::Bottom, |acc, value| {
                    join_abstract_css_values(&acc, &value)
                }),
        );
    } else {
        source
    };
    let values = split_static_scss_top_level(source_text, ',')?;
    if values.len() <= 1 || values.len() > 64 {
        return None;
    }
    Some(
        values
            .into_iter()
            .fold(AbstractCssValueV0::Bottom, |acc, value| {
                let value = abstract_css_value_from_text(value.as_str());
                join_abstract_css_values(&acc, &value)
            }),
    )
}

fn source_is_single_static_variable(source: &str) -> bool {
    source.starts_with('$') && variable_name_end(source, '$'.len_utf8()) == source.len()
}

fn static_each_source_text<'a>(
    source: &'a str,
    lexical_bindings: &'a BTreeMap<String, AbstractCssValueV0>,
) -> Option<&'a str> {
    if source_is_single_static_variable(source) {
        return static_scss_binding_value(lexical_bindings, source)
            .and_then(single_static_scss_header_value_text);
    }
    Some(source)
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
