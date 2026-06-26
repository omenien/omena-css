use std::collections::{BTreeMap, BTreeSet};

use omena_abstract_value::{
    AbstractCssValueV0, abstract_css_value_from_text, join_abstract_css_values,
};
use omena_parser::{StyleDialect, parse};
use omena_syntax::SyntaxKind;

use crate::{
    static_loop_frames::{
        parse_static_scss_each_loop_binding_frames, parse_static_scss_each_single_values,
        parse_static_scss_for_loop_header, static_scss_for_loop_values,
    },
    value_eval::reduce_static_scss_value,
};

use super::{
    header_values::{scss_header_value_from_bindings, single_static_scss_header_value_text},
    model::OmenaScssEvalControlFlowBlockV0,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StaticWhileAssignmentStep {
    Known(i32),
    Unknown,
    Unspecified,
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
    let parsed = parse(body, StyleDialect::Sass);
    let root = parsed.syntax();
    let Some(tokens) = cst_token_ranges(&root) else {
        return Vec::new();
    };
    let mut names: Vec<String> = Vec::new();
    for (index, token) in tokens.iter().enumerate() {
        if token.kind != SyntaxKind::ScssVariable {
            continue;
        }
        let Some(colon_index) = cst_next_non_trivia_token_index(tokens.as_slice(), index + 1)
        else {
            continue;
        };
        if !matches!(
            tokens[colon_index].kind,
            SyntaxKind::Colon | SyntaxKind::PlusEquals | SyntaxKind::MinusEquals
        ) {
            continue;
        }
        let name = token.text.clone();
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
    let step = binding_names
        .first()
        .map_or(StaticWhileAssignmentStep::Unspecified, |binding_name| {
            static_while_assignment_step(body_text, binding_name, lexical_bindings)
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

pub(super) fn control_flow_block_body_text<'a>(
    source: &'a str,
    block: &OmenaScssEvalControlFlowBlockV0,
) -> Option<&'a str> {
    let block_text = source.get(block.source_span_start..block.source_span_end)?;
    if let Some(body) = control_flow_brace_block_body_text(block_text) {
        return Some(body);
    }
    control_flow_sass_indented_block_body_text(block_text)
}

fn control_flow_brace_block_body_text(block_text: &str) -> Option<&str> {
    let open = block_text.find('{')?;
    let close = block_text.rfind('}')?;
    (open < close)
        .then(|| block_text.get(open + '{'.len_utf8()..close))
        .flatten()
}

fn control_flow_sass_indented_block_body_text(block_text: &str) -> Option<&str> {
    let parsed = parse(block_text, StyleDialect::Sass);
    let root = parsed.syntax();
    let tokens = cst_token_ranges(&root)?;
    let block_start_index = cst_next_sass_indent_token_index(tokens.as_slice(), 0)?;
    let block_end_index =
        cst_matching_sass_dedent_token_index(tokens.as_slice(), block_start_index)?;
    let body_start = tokens.get(block_start_index)?.end;
    let body_end = tokens.get(block_end_index)?.start;
    (body_start <= body_end)
        .then(|| block_text.get(body_start..body_end))
        .flatten()
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CstTokenRange {
    kind: SyntaxKind,
    start: usize,
    end: usize,
    text: String,
}

fn cst_token_ranges(root: &cstree::syntax::SyntaxNode<SyntaxKind>) -> Option<Vec<CstTokenRange>> {
    root.descendants_with_tokens()
        .filter_map(|element| element.into_token())
        .map(|token| {
            Some(CstTokenRange {
                kind: token.kind(),
                start: u32::from(token.text_range().start()) as usize,
                end: u32::from(token.text_range().end()) as usize,
                text: cst_token_text(token)?,
            })
        })
        .collect()
}

fn cst_token_text(token: &cstree::syntax::SyntaxToken<SyntaxKind>) -> Option<String> {
    if let Some(resolver) = token.resolver() {
        Some(token.resolve_text(&**resolver).to_string())
    } else {
        token.static_text().map(str::to_string)
    }
}

fn cst_next_sass_indent_token_index(tokens: &[CstTokenRange], mut index: usize) -> Option<usize> {
    while index < tokens.len() {
        match tokens[index].kind {
            SyntaxKind::SassIndent => return Some(index),
            SyntaxKind::Semicolon | SyntaxKind::SassOptionalSemicolon => return None,
            _ => index += 1,
        }
    }
    None
}

fn cst_next_non_trivia_token_index(tokens: &[CstTokenRange], mut index: usize) -> Option<usize> {
    while tokens
        .get(index)
        .is_some_and(|token| cst_is_trivia_token(token.kind))
    {
        index += 1;
    }
    (index < tokens.len()).then_some(index)
}

fn cst_declaration_end_token_index(tokens: &[CstTokenRange], mut index: usize) -> Option<usize> {
    let mut paren_depth = 0usize;
    while index < tokens.len() {
        match tokens[index].kind {
            SyntaxKind::LeftParen => paren_depth += 1,
            SyntaxKind::RightParen => paren_depth = paren_depth.checked_sub(1)?,
            SyntaxKind::Semicolon | SyntaxKind::SassOptionalSemicolon | SyntaxKind::SassIndent
                if paren_depth == 0 =>
            {
                return Some(index);
            }
            _ => {}
        }
        index += 1;
    }
    None
}

const fn cst_is_trivia_token(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::Whitespace
            | SyntaxKind::LineComment
            | SyntaxKind::BlockComment
            | SyntaxKind::ScssSilentComment
            | SyntaxKind::SassIndentedNewline
    )
}

fn cst_matching_sass_dedent_token_index(
    tokens: &[CstTokenRange],
    sass_indent_index: usize,
) -> Option<usize> {
    let mut depth = 0usize;
    for (index, token) in tokens.iter().enumerate().skip(sass_indent_index) {
        match token.kind {
            SyntaxKind::SassIndent => depth += 1,
            SyntaxKind::SassDedent => {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    return Some(index);
                }
            }
            _ => {}
        }
    }
    None
}

fn static_while_assignment_step(
    body_text: Option<&str>,
    binding_name: &str,
    lexical_bindings: &BTreeMap<String, AbstractCssValueV0>,
) -> StaticWhileAssignmentStep {
    body_text.map_or(StaticWhileAssignmentStep::Unspecified, |body| {
        while_loop_body_assignment_step(body, binding_name, lexical_bindings)
    })
}

fn while_loop_body_assignment_step(
    body: &str,
    binding_name: &str,
    lexical_bindings: &BTreeMap<String, AbstractCssValueV0>,
) -> StaticWhileAssignmentStep {
    let parsed = parse(body, StyleDialect::Sass);
    let root = parsed.syntax();
    let Some(tokens) = cst_token_ranges(&root) else {
        return StaticWhileAssignmentStep::Unknown;
    };
    let mut brace_depth = 0usize;
    let mut total_step = 0i32;
    let mut saw_assignment = false;
    for (index, token) in tokens.iter().enumerate() {
        match token.kind {
            SyntaxKind::LeftBrace => {
                brace_depth += 1;
                continue;
            }
            SyntaxKind::RightBrace => {
                let Some(next_depth) = brace_depth.checked_sub(1) else {
                    return StaticWhileAssignmentStep::Unknown;
                };
                brace_depth = next_depth;
                continue;
            }
            _ => {}
        }
        if token.kind != SyntaxKind::ScssVariable
            || canonical_scss_variable_name(token.text.as_str())
                != canonical_scss_variable_name(binding_name)
        {
            continue;
        }
        let Some(operator_index) = cst_next_non_trivia_token_index(tokens.as_slice(), index + 1)
        else {
            continue;
        };
        let delta = match tokens[operator_index].kind {
            SyntaxKind::PlusEquals => static_while_integer_expression_after(
                body,
                tokens.as_slice(),
                operator_index,
                lexical_bindings,
            ),
            SyntaxKind::MinusEquals => static_while_integer_expression_after(
                body,
                tokens.as_slice(),
                operator_index,
                lexical_bindings,
            )
            .map(i32::saturating_neg),
            SyntaxKind::Colon => static_while_self_assignment_step(
                body,
                tokens.as_slice(),
                operator_index,
                binding_name,
                lexical_bindings,
            ),
            _ => continue,
        };
        saw_assignment = true;
        if brace_depth != 0 {
            return StaticWhileAssignmentStep::Unknown;
        }
        let Some(delta) = delta else {
            return StaticWhileAssignmentStep::Unknown;
        };
        let Some(next_step) = total_step.checked_add(delta) else {
            return StaticWhileAssignmentStep::Unknown;
        };
        total_step = next_step;
    }
    if saw_assignment {
        StaticWhileAssignmentStep::Known(total_step)
    } else {
        StaticWhileAssignmentStep::Unknown
    }
}

fn static_while_self_assignment_step(
    body: &str,
    tokens: &[CstTokenRange],
    colon_index: usize,
    binding_name: &str,
    lexical_bindings: &BTreeMap<String, AbstractCssValueV0>,
) -> Option<i32> {
    let variable_index = cst_next_non_trivia_token_index(tokens, colon_index + 1)?;
    let variable = tokens.get(variable_index)?;
    if variable.kind != SyntaxKind::ScssVariable
        || canonical_scss_variable_name(variable.text.as_str())
            != canonical_scss_variable_name(binding_name)
    {
        return None;
    }
    let operator_index = cst_next_non_trivia_token_index(tokens, variable_index + 1)?;
    match tokens.get(operator_index)?.kind {
        SyntaxKind::Plus => {
            static_while_integer_expression_after(body, tokens, operator_index, lexical_bindings)
        }
        SyntaxKind::Minus => {
            static_while_integer_expression_after(body, tokens, operator_index, lexical_bindings)
                .map(i32::saturating_neg)
        }
        _ => None,
    }
}

fn static_while_integer_expression_after(
    source: &str,
    tokens: &[CstTokenRange],
    operator_index: usize,
    lexical_bindings: &BTreeMap<String, AbstractCssValueV0>,
) -> Option<i32> {
    let value_start_index = cst_next_non_trivia_token_index(tokens, operator_index + 1)?;
    let declaration_end_index = cst_declaration_end_token_index(tokens, value_start_index)?;
    let value_start = tokens.get(value_start_index)?.start;
    let value_end = tokens.get(declaration_end_index)?.start;
    let expression = source.get(value_start..value_end)?.trim();
    parse_static_while_integer_expression(expression, lexical_bindings)
}

fn parse_static_while_integer_expression(
    expression: &str,
    lexical_bindings: &BTreeMap<String, AbstractCssValueV0>,
) -> Option<i32> {
    let reduced = scss_header_value_from_bindings(expression, lexical_bindings);
    single_static_scss_header_value_text(&reduced).and_then(parse_static_while_integer_text)
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
    let for_header = parse_static_scss_for_loop_header(header)?;
    let start_values =
        parse_static_for_loop_bound_values(for_header.start_bound, lexical_bindings)?;
    let end_values = parse_static_for_loop_bound_values(for_header.end_bound, lexical_bindings)?;
    Some(
        static_for_loop_value_set(start_values, end_values, for_header.includes_end)?
            .into_iter()
            .map(|value| {
                vec![ScssControlFlowBindingValue {
                    name: for_header.binding.clone(),
                    value: abstract_css_value_from_text(value.to_string().as_str()),
                }]
            })
            .collect(),
    )
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
    let step = static_while_assignment_step(body_text, bindings[0].as_str(), lexical_bindings);
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
        AbstractCssValueV0::FiniteSet { values, .. } => Some(
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
        static_each_source_text(source.trim(), lexical_bindings)
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
    step: StaticWhileAssignmentStep,
) -> Option<Vec<ScssControlFlowBindingValue>> {
    if binding_names.len() != 1 {
        return None;
    }
    let binding_name = binding_names[0].as_str();
    let (left, operator, right) = split_static_while_inequality(header)?;
    let start_values = static_while_integer_binding_values(binding_name, lexical_bindings)?;

    let (operator, bound_values) = if static_scss_side_is_binding(left, binding_name) {
        (
            operator,
            static_while_integer_operand_values(right, lexical_bindings)?,
        )
    } else if static_scss_side_is_binding(right, binding_name) {
        (
            operator.inverted_for_right_hand_binding(),
            static_while_integer_operand_values(left, lexical_bindings)?,
        )
    } else {
        return None;
    };
    let value = static_while_integer_domain_set(start_values, operator, bound_values, step)?;

    Some(vec![ScssControlFlowBindingValue {
        name: binding_names[0].clone(),
        value,
    }])
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StaticWhileInequalityOperator {
    Equal,
    NotEqual,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
}

impl StaticWhileInequalityOperator {
    const fn inverted_for_right_hand_binding(self) -> Self {
        match self {
            Self::Equal => Self::Equal,
            Self::NotEqual => Self::NotEqual,
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
            '=' | '!' | '<' | '>' if paren_depth == 0 && bracket_depth == 0 => {
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
    if suffix.starts_with("==") {
        return Some((StaticWhileInequalityOperator::Equal, 2));
    }
    if suffix.starts_with("!=") {
        return Some((StaticWhileInequalityOperator::NotEqual, 2));
    }
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

fn static_while_integer_binding_values(
    binding_name: &str,
    lexical_bindings: &BTreeMap<String, AbstractCssValueV0>,
) -> Option<Vec<i32>> {
    static_scss_binding_value(lexical_bindings, binding_name).and_then(static_while_integer_values)
}

fn static_while_integer_operand_values(
    value: &str,
    lexical_bindings: &BTreeMap<String, AbstractCssValueV0>,
) -> Option<Vec<i32>> {
    let reduced = scss_header_value_from_bindings(value, lexical_bindings);
    static_while_integer_values(&reduced)
}

fn static_while_integer_values(value: &AbstractCssValueV0) -> Option<Vec<i32>> {
    match value {
        AbstractCssValueV0::Exact { value, .. } | AbstractCssValueV0::Raw { value } => {
            Some(vec![parse_static_while_integer_text(value.as_str())?])
        }
        AbstractCssValueV0::FiniteSet { values, .. } => values
            .iter()
            .map(|value| parse_static_while_integer_text(value.as_str()))
            .collect(),
        AbstractCssValueV0::Bottom | AbstractCssValueV0::Top => None,
    }
}

fn parse_static_while_integer_text(value: &str) -> Option<i32> {
    let reduced = reduce_static_scss_value(value.trim().to_string());
    reduced.trim().parse::<i32>().ok()
}

fn static_while_integer_domain_set(
    start_values: Vec<i32>,
    operator: StaticWhileInequalityOperator,
    bound_values: Vec<i32>,
    step: StaticWhileAssignmentStep,
) -> Option<AbstractCssValueV0> {
    if start_values.is_empty() || bound_values.is_empty() {
        return Some(AbstractCssValueV0::Bottom);
    }
    let mut values = BTreeSet::new();
    for start in start_values {
        for bound in &bound_values {
            for value in static_while_integer_domain_values(start, operator, *bound, step)? {
                values.insert(value);
                if values.len() > 64 {
                    return None;
                }
            }
        }
    }
    Some(abstract_css_value_from_static_integers(
        values.into_iter().collect(),
    ))
}

fn static_while_integer_domain_values(
    start: i32,
    operator: StaticWhileInequalityOperator,
    bound: i32,
    step: StaticWhileAssignmentStep,
) -> Option<Vec<i32>> {
    match step {
        StaticWhileAssignmentStep::Known(step) => {
            return static_while_integer_progression_domain_values(start, operator, bound, step);
        }
        StaticWhileAssignmentStep::Unknown => {
            return (!static_while_integer_condition_holds(start, operator, bound)).then(Vec::new);
        }
        StaticWhileAssignmentStep::Unspecified => {}
    }

    let (first, last) = match operator {
        StaticWhileInequalityOperator::Equal | StaticWhileInequalityOperator::NotEqual => {
            return None;
        }
        StaticWhileInequalityOperator::LessThan => {
            if start >= bound {
                return Some(Vec::new());
            }
            (start, bound.saturating_sub(1))
        }
        StaticWhileInequalityOperator::LessThanOrEqual => {
            if start > bound {
                return Some(Vec::new());
            }
            (start, bound)
        }
        StaticWhileInequalityOperator::GreaterThan => {
            if start <= bound {
                return Some(Vec::new());
            }
            (bound.saturating_add(1), start)
        }
        StaticWhileInequalityOperator::GreaterThanOrEqual => {
            if start < bound {
                return Some(Vec::new());
            }
            (bound, start)
        }
    };
    let value_count = i64::from(last) - i64::from(first) + 1;
    if !(1..=64).contains(&value_count) {
        return None;
    }
    Some((first..=last).collect())
}

fn static_while_integer_progression_domain_values(
    start: i32,
    operator: StaticWhileInequalityOperator,
    bound: i32,
    step: i32,
) -> Option<Vec<i32>> {
    if step == 0 {
        return None;
    }
    let mut current = start;
    let mut values = Vec::new();
    for _ in 0..64 {
        if !static_while_integer_condition_holds(current, operator, bound) {
            return Some(values);
        }
        values.push(current);
        current = current.checked_add(step)?;
    }
    if static_while_integer_condition_holds(current, operator, bound) {
        None
    } else {
        Some(values)
    }
}

fn abstract_css_value_from_static_integers(values: Vec<i32>) -> AbstractCssValueV0 {
    values
        .into_iter()
        .fold(AbstractCssValueV0::Bottom, |acc, value| {
            let value = abstract_css_value_from_text(value.to_string().as_str());
            join_abstract_css_values(&acc, &value)
        })
}

const fn static_while_integer_condition_holds(
    value: i32,
    operator: StaticWhileInequalityOperator,
    bound: i32,
) -> bool {
    match operator {
        StaticWhileInequalityOperator::Equal => value == bound,
        StaticWhileInequalityOperator::NotEqual => value != bound,
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
    let for_header = parse_static_scss_for_loop_header(header)?;
    let start_values =
        parse_static_for_loop_bound_values(for_header.start_bound, lexical_bindings)?;
    let end_values = parse_static_for_loop_bound_values(for_header.end_bound, lexical_bindings)?;
    let Some(values) = static_for_loop_value_set(start_values, end_values, for_header.includes_end)
    else {
        return Some(AbstractCssValueV0::Top);
    };
    if values.is_empty() {
        return Some(AbstractCssValueV0::Bottom);
    }
    Some(
        values
            .into_iter()
            .fold(AbstractCssValueV0::Bottom, |acc, value| {
                let value = abstract_css_value_from_text(value.to_string().as_str());
                join_abstract_css_values(&acc, &value)
            }),
    )
}

fn parse_static_for_loop_bound_values(
    value: &str,
    lexical_bindings: &BTreeMap<String, AbstractCssValueV0>,
) -> Option<Vec<i32>> {
    match scss_header_value_from_bindings(value, lexical_bindings) {
        AbstractCssValueV0::Exact { value, .. } | AbstractCssValueV0::Raw { value } => {
            Some(vec![parse_static_for_loop_integer(value.as_str())?])
        }
        AbstractCssValueV0::FiniteSet { values, .. } => values
            .into_iter()
            .map(|value| parse_static_for_loop_integer(value.as_str()))
            .collect(),
        AbstractCssValueV0::Bottom | AbstractCssValueV0::Top => None,
    }
}

fn parse_static_for_loop_integer(value: &str) -> Option<i32> {
    value.parse::<i32>().ok()
}

fn static_for_loop_value_set(
    start_values: Vec<i32>,
    end_values: Vec<i32>,
    includes_end: bool,
) -> Option<Vec<i32>> {
    if start_values.is_empty() || end_values.is_empty() {
        return Some(Vec::new());
    }
    let mut values = BTreeSet::new();
    for start in start_values {
        for end in &end_values {
            for value in static_scss_for_loop_values(start, *end, includes_end)? {
                values.insert(value);
                if values.len() > 64 {
                    return None;
                }
            }
        }
    }
    Some(values.into_iter().collect())
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
    let source_text = static_each_source_text(source, lexical_bindings);
    let Some(source_text) = source_text else {
        if source_variables.is_empty() {
            return None;
        }
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
    };
    let values = parse_static_scss_each_single_values(source_text.as_str())?;
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

fn static_each_source_text(
    source: &str,
    lexical_bindings: &BTreeMap<String, AbstractCssValueV0>,
) -> Option<String> {
    if source_is_single_static_variable(source) {
        return static_scss_binding_value(lexical_bindings, source)
            .and_then(single_static_scss_header_value_text)
            .map(str::to_string);
    }
    let reduced = scss_header_value_from_bindings(source, lexical_bindings);
    single_static_scss_header_value_text(&reduced).map(str::to_string)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn static_for_loop_range_resolves_finite_set_bounds() {
        let bindings = finite_end_bindings();
        let value = parse_static_for_loop_range("@for $i from 1 through $end", &bindings);

        assert_eq!(
            value,
            Some(AbstractCssValueV0::FiniteSet {
                typed: None,
                values: vec![
                    "1".to_string(),
                    "2".to_string(),
                    "3".to_string(),
                    "4".to_string(),
                ],
            })
        );
    }

    #[test]
    fn static_for_loop_binding_frames_resolve_finite_set_bounds() {
        let bindings = finite_end_bindings();
        let frames = static_for_loop_binding_frames("@for $i from 1 through $end", &bindings);
        assert!(frames.is_some());
        let Some(frames) = frames else {
            return;
        };

        let values = frames
            .into_iter()
            .filter_map(|frame| frame.into_iter().next())
            .map(|binding| binding.value)
            .collect::<Vec<_>>();

        assert_eq!(
            values,
            vec![
                AbstractCssValueV0::Exact {
                    typed: None,
                    value: "1".to_string(),
                },
                AbstractCssValueV0::Exact {
                    typed: None,
                    value: "2".to_string(),
                },
                AbstractCssValueV0::Exact {
                    typed: None,
                    value: "3".to_string(),
                },
                AbstractCssValueV0::Exact {
                    typed: None,
                    value: "4".to_string(),
                },
            ]
        );
    }

    #[test]
    fn static_while_loop_binding_frames_resolve_finite_set_bounds() {
        let bindings = finite_limit_bindings();
        let values = while_loop_carried_binding_values_from_parts(
            "$i < $limit",
            &bindings,
            &["$i".to_string()],
            Some("$i: $i + 1;"),
        );

        assert_eq!(values.len(), 1);

        assert_eq!(
            values[0].value,
            AbstractCssValueV0::FiniteSet {
                typed: None,
                values: vec![
                    "0".to_string(),
                    "1".to_string(),
                    "2".to_string(),
                    "3".to_string(),
                ],
            }
        );
    }

    #[test]
    fn static_while_loop_binding_frames_resolve_finite_set_start_values() {
        let bindings = finite_start_bindings();
        let values = while_loop_carried_binding_values_from_parts(
            "$i < 4",
            &bindings,
            &["$i".to_string()],
            Some("$i: $i + 1;"),
        );

        assert_eq!(values.len(), 1);

        assert_eq!(
            values[0].value,
            AbstractCssValueV0::FiniteSet {
                typed: None,
                values: vec![
                    "0".to_string(),
                    "1".to_string(),
                    "2".to_string(),
                    "3".to_string(),
                ],
            }
        );
    }

    fn finite_start_bindings() -> BTreeMap<String, AbstractCssValueV0> {
        BTreeMap::from([(
            "$i".to_string(),
            AbstractCssValueV0::FiniteSet {
                typed: None,
                values: vec!["0".to_string(), "2".to_string()],
            },
        )])
    }

    #[test]
    fn static_while_loop_value_resolves_finite_set_bounds() {
        let values = static_while_integer_domain_set(
            vec![0],
            StaticWhileInequalityOperator::LessThan,
            vec![2, 4],
            StaticWhileAssignmentStep::Known(1),
        );

        assert_eq!(
            values,
            Some(AbstractCssValueV0::FiniteSet {
                typed: None,
                values: vec![
                    "0".to_string(),
                    "1".to_string(),
                    "2".to_string(),
                    "3".to_string(),
                ],
            })
        );
    }

    fn finite_end_bindings() -> BTreeMap<String, AbstractCssValueV0> {
        BTreeMap::from([(
            "$end".to_string(),
            AbstractCssValueV0::FiniteSet {
                typed: None,
                values: vec!["2".to_string(), "4".to_string()],
            },
        )])
    }

    fn finite_limit_bindings() -> BTreeMap<String, AbstractCssValueV0> {
        BTreeMap::from([
            (
                "$i".to_string(),
                AbstractCssValueV0::Exact {
                    typed: None,
                    value: "0".to_string(),
                },
            ),
            (
                "$limit".to_string(),
                AbstractCssValueV0::FiniteSet {
                    typed: None,
                    values: vec!["2".to_string(), "4".to_string()],
                },
            ),
        ])
    }
}
