use std::collections::{BTreeMap, BTreeSet};

use omena_parser::{LexedToken, StyleDialect};
use omena_syntax::SyntaxKind;

use super::{
    OmenaScssEvalResolvedReplacementV0, STATIC_STYLESHEET_VALUE_RESOLUTION_FUEL_LIMIT,
    StaticScssFunctionResolutionContext, StaticStylesheetEvaluationEdit,
    StaticStylesheetResolutionOutcome, StaticStylesheetVariableKind,
    apply_static_stylesheet_evaluation_edits, canonical_static_scss_function_name,
    canonical_static_scss_variable_name, collect_static_stylesheet_variable_references,
    resolve_static_scss_function_value_with_bindings, resolved_replacement_value,
    static_stylesheet_matching_token_index, static_stylesheet_position_is_inside_ranges,
    static_stylesheet_skip_trivia_tokens, static_stylesheet_token_end,
    static_stylesheet_token_start, static_stylesheet_value_end_token_until,
    tokens::static_stylesheet_block_kinds_for_dialect,
};
use crate::static_loop_frames::{
    parse_static_scss_each_loop_binding_frames, parse_static_scss_for_loop_header,
    static_scss_for_loop_values,
};

#[derive(Debug, Clone)]
pub(super) struct StaticScssLoopEvaluationEdits {
    pub(super) edits: Vec<StaticStylesheetEvaluationEdit>,
    pub(super) replacements: Vec<OmenaScssEvalResolvedReplacementV0>,
    pub(super) preserved_dynamic_loop_count: usize,
}

pub(super) fn collect_static_scss_loop_evaluation_edits(
    source: &str,
    dialect: StyleDialect,
    tokens: &[LexedToken],
    excluded_ranges: &[(usize, usize)],
    context: StaticScssFunctionResolutionContext<'_>,
) -> Option<StaticScssLoopEvaluationEdits> {
    if !static_scss_loop_dialect_is_supported(dialect)
        || !static_scss_source_contains_loop_candidate(source)
    {
        return Some(StaticScssLoopEvaluationEdits {
            edits: Vec::new(),
            replacements: Vec::new(),
            preserved_dynamic_loop_count: 0,
        });
    }

    let mut edits = Vec::new();
    let mut replacements = Vec::new();
    let mut preserved_dynamic_loop_count = 0usize;
    let mut index = 0usize;
    while index < tokens.len() {
        let token = &tokens[index];
        if !static_scss_token_is_loop_at_keyword(token) {
            index += 1;
            continue;
        }
        let start = static_stylesheet_token_start(token);
        if static_stylesheet_position_is_inside_ranges(start, excluded_ranges) {
            index += 1;
            continue;
        }

        let Some(loop_block) = static_scss_loop_block(source, dialect, tokens, index) else {
            preserved_dynamic_loop_count += 1;
            index += 1;
            continue;
        };
        let Some(rendered) = render_static_scss_loop_block(&loop_block, context) else {
            preserved_dynamic_loop_count += 1;
            index = loop_block.next_index;
            continue;
        };
        replacements.extend(rendered.replacements);
        edits.push(StaticStylesheetEvaluationEdit {
            start: loop_block.start,
            end: loop_block.end,
            replacement: rendered.replacement,
        });
        index = loop_block.next_index;
    }

    Some(StaticScssLoopEvaluationEdits {
        edits,
        replacements,
        preserved_dynamic_loop_count,
    })
}

pub(super) fn collect_static_scss_loop_candidate_ranges(
    source: &str,
    dialect: StyleDialect,
    tokens: &[LexedToken],
    excluded_ranges: &[(usize, usize)],
) -> Vec<(usize, usize)> {
    if !static_scss_loop_dialect_is_supported(dialect)
        || !static_scss_source_contains_loop_candidate(source)
    {
        return Vec::new();
    }
    let mut ranges = Vec::new();
    let mut index = 0usize;
    while index < tokens.len() {
        let token = &tokens[index];
        if !static_scss_token_is_loop_at_keyword(token) {
            index += 1;
            continue;
        }
        let start = static_stylesheet_token_start(token);
        if static_stylesheet_position_is_inside_ranges(start, excluded_ranges) {
            index += 1;
            continue;
        }
        if let Some(loop_block) = static_scss_loop_block(source, dialect, tokens, index) {
            ranges.push((loop_block.start, loop_block.end));
            index = loop_block.next_index;
        } else {
            index += 1;
        }
    }
    ranges
}

pub(super) fn render_static_scss_mixin_loop_control_flow_body(
    body: &str,
    dialect: StyleDialect,
    argument_values: &BTreeMap<String, String>,
    continuation_indent: &str,
    call_position: usize,
    context: StaticScssFunctionResolutionContext<'_>,
) -> Option<String> {
    if !static_scss_loop_dialect_is_supported(dialect)
        || !static_scss_source_contains_loop_candidate(body)
    {
        return Some(body.to_string());
    }
    render_static_scss_mixin_loop_control_flow_body_with_fuel(
        body,
        dialect,
        argument_values,
        continuation_indent,
        call_position,
        context,
        32,
    )
}

fn render_static_scss_mixin_loop_control_flow_body_with_fuel(
    body: &str,
    dialect: StyleDialect,
    argument_values: &BTreeMap<String, String>,
    continuation_indent: &str,
    call_position: usize,
    context: StaticScssFunctionResolutionContext<'_>,
    fuel: usize,
) -> Option<String> {
    if fuel == 0 {
        return None;
    }
    let lexed = omena_parser::lex(body, dialect);
    let tokens = lexed.tokens();
    let mut edits = Vec::new();
    let mut index = 0usize;
    while index < tokens.len() {
        let token = &tokens[index];
        if !static_scss_token_is_loop_at_keyword(token) {
            index += 1;
            continue;
        }
        let loop_block = static_scss_loop_block(body, dialect, tokens, index)?;
        let rendered = render_static_scss_loop_block_with_bindings(
            &loop_block,
            argument_values,
            call_position,
            context,
        )?;
        let replacement = static_sass_mixin_loop_replacement_with_line_indent(
            dialect,
            continuation_indent,
            rendered.replacement.as_str(),
        );
        edits.push(StaticStylesheetEvaluationEdit {
            start: loop_block.start,
            end: loop_block.end,
            replacement,
        });
        index = loop_block.next_index;
    }

    if edits.is_empty() {
        return Some(body.to_string());
    }
    let rendered = apply_static_stylesheet_evaluation_edits(body, edits)?;
    if static_scss_source_contains_loop_candidate(rendered.as_str()) {
        return render_static_scss_mixin_loop_control_flow_body_with_fuel(
            rendered.as_str(),
            dialect,
            argument_values,
            continuation_indent,
            call_position,
            context,
            fuel - 1,
        );
    }
    Some(rendered)
}

fn static_sass_mixin_loop_replacement_with_line_indent(
    dialect: StyleDialect,
    continuation_indent: &str,
    replacement: &str,
) -> String {
    if dialect != StyleDialect::Sass {
        return replacement.to_string();
    }
    if continuation_indent.is_empty() || replacement.is_empty() {
        return replacement.to_string();
    }
    let replacement = replacement.trim_start_matches([' ', '\t']);
    let mut output = String::with_capacity(replacement.len() + continuation_indent.len());
    for (index, line) in replacement.split_inclusive('\n').enumerate() {
        if index > 0 && !line.is_empty() {
            output.push_str(continuation_indent);
        }
        output.push_str(line);
    }
    output
}

#[derive(Debug, Clone)]
struct StaticScssLoopBlock {
    at_rule_name: String,
    dialect: StyleDialect,
    start: usize,
    end: usize,
    next_index: usize,
    header: String,
    body: String,
    body_start: usize,
}

#[derive(Debug, Clone)]
struct StaticScssLoopRender {
    replacement: String,
    replacements: Vec<OmenaScssEvalResolvedReplacementV0>,
}

fn static_scss_loop_block(
    source: &str,
    dialect: StyleDialect,
    tokens: &[LexedToken],
    loop_index: usize,
) -> Option<StaticScssLoopBlock> {
    let (body_open_kind, body_close_kind) = static_stylesheet_block_kinds_for_dialect(dialect);
    let body_open_index =
        (loop_index + 1..tokens.len()).find(|index| tokens[*index].kind == body_open_kind)?;
    let body_close_index = static_stylesheet_matching_token_index(
        tokens,
        body_open_index,
        body_open_kind,
        body_close_kind,
    )?;
    let start = static_stylesheet_token_start(&tokens[loop_index]);
    let end = static_stylesheet_token_end(&tokens[body_close_index]);
    let header_start = static_stylesheet_token_end(&tokens[loop_index]);
    let header_end = static_stylesheet_token_start(&tokens[body_open_index]);
    let body_start = static_stylesheet_token_end(&tokens[body_open_index]);
    let body_end = static_stylesheet_token_start(&tokens[body_close_index]);
    Some(StaticScssLoopBlock {
        at_rule_name: tokens[loop_index].text.clone(),
        dialect,
        start,
        end,
        next_index: body_close_index + 1,
        header: source.get(header_start..header_end)?.trim().to_string(),
        body: source.get(body_start..body_end)?.to_string(),
        body_start,
    })
}

fn render_static_scss_loop_block(
    loop_block: &StaticScssLoopBlock,
    context: StaticScssFunctionResolutionContext<'_>,
) -> Option<StaticScssLoopRender> {
    render_static_scss_loop_block_with_bindings(
        loop_block,
        &BTreeMap::new(),
        loop_block.start,
        context,
    )
}

fn render_static_scss_loop_block_with_bindings(
    loop_block: &StaticScssLoopBlock,
    argument_values: &BTreeMap<String, String>,
    call_position: usize,
    context: StaticScssFunctionResolutionContext<'_>,
) -> Option<StaticScssLoopRender> {
    if static_scss_loop_body_contains_known_function_call(
        loop_block.body.as_str(),
        loop_block.dialect,
        context,
    ) {
        return None;
    }
    if loop_block.at_rule_name.eq_ignore_ascii_case("@while") {
        return render_static_scss_while_loop_block_with_bindings(
            loop_block,
            argument_values,
            call_position,
            context,
        );
    }
    if loop_block.at_rule_name.eq_ignore_ascii_case("@each") {
        return render_static_scss_each_loop_block_with_bindings(
            loop_block,
            argument_values,
            call_position,
            context,
        );
    }
    let header = parse_static_scss_for_loop_header(loop_block.header.as_str())?;
    let start = resolve_static_scss_for_loop_bound_with_bindings(
        header.start_bound,
        argument_values,
        call_position,
        context,
    )?;
    let end = resolve_static_scss_for_loop_bound_with_bindings(
        header.end_bound,
        argument_values,
        call_position,
        context,
    )?;
    let values = static_scss_for_loop_values(start, end, header.includes_end)?;

    let mut output = String::with_capacity(loop_block.body.len().saturating_mul(values.len()));
    let mut replacements = Vec::new();
    for value in values {
        let binding_name = canonical_static_scss_variable_name(header.binding.as_str());
        let mut frame_values = argument_values.clone();
        frame_values.insert(binding_name, value.to_string());
        let rendered = render_static_scss_loop_body(
            loop_block.body.as_str(),
            loop_block.body_start,
            &frame_values,
            context,
            &[],
        )?;
        replacements.extend(rendered.replacements);
        append_static_scss_loop_rendered_body(
            &mut output,
            rendered.replacement.as_str(),
            loop_block.dialect,
        );
    }

    if !static_scss_loop_replacement_is_static_css_subset(output.as_str()) {
        return None;
    }
    Some(StaticScssLoopRender {
        replacement: output,
        replacements,
    })
}

fn render_static_scss_each_loop_block_with_bindings(
    loop_block: &StaticScssLoopBlock,
    argument_values: &BTreeMap<String, String>,
    call_position: usize,
    context: StaticScssFunctionResolutionContext<'_>,
) -> Option<StaticScssLoopRender> {
    let frames =
        parse_static_scss_each_loop_binding_frames(loop_block.header.as_str(), |source| {
            resolve_static_scss_loop_source_value_with_bindings(
                source,
                argument_values,
                call_position,
                context,
            )
        })?;
    let mut output = String::with_capacity(loop_block.body.len().saturating_mul(frames.len()));
    let mut replacements = Vec::new();
    for frame in frames {
        let mut frame_values = argument_values.clone();
        for (name, value) in frame {
            frame_values.insert(canonical_static_scss_variable_name(name.as_str()), value);
        }
        let rendered = render_static_scss_loop_body(
            loop_block.body.as_str(),
            loop_block.body_start,
            &frame_values,
            context,
            &[],
        )?;
        replacements.extend(rendered.replacements);
        append_static_scss_loop_rendered_body(
            &mut output,
            rendered.replacement.as_str(),
            loop_block.dialect,
        );
    }
    if !static_scss_loop_replacement_is_static_css_subset(output.as_str()) {
        return None;
    }
    Some(StaticScssLoopRender {
        replacement: output,
        replacements,
    })
}

fn render_static_scss_while_loop_block_with_bindings(
    loop_block: &StaticScssLoopBlock,
    argument_values: &BTreeMap<String, String>,
    call_position: usize,
    context: StaticScssFunctionResolutionContext<'_>,
) -> Option<StaticScssLoopRender> {
    let assignments =
        collect_static_scss_while_body_assignments(loop_block.body.as_str(), loop_block.dialect)?;
    if assignments.is_empty()
        || !static_scss_while_body_references_follow_assignments(
            loop_block.body.as_str(),
            assignments.as_slice(),
        )?
    {
        return None;
    }
    let assignment_ranges = assignments
        .iter()
        .map(|assignment| (assignment.start, assignment.end))
        .collect::<Vec<_>>();
    let mut current_values = argument_values.clone();
    let mut output = String::with_capacity(loop_block.body.len());
    let mut replacements = Vec::new();

    for _ in 0..64 {
        if !static_scss_while_condition_is_active(
            loop_block.header.as_str(),
            &current_values,
            call_position,
            context,
        )? {
            if !static_scss_loop_replacement_is_static_css_subset(output.as_str()) {
                return None;
            }
            return Some(StaticScssLoopRender {
                replacement: output,
                replacements,
            });
        }

        let next_values = static_scss_while_next_frame_values(
            loop_block.body.as_str(),
            assignments.as_slice(),
            &current_values,
            loop_block.body_start,
            context,
        )?;
        if next_values == current_values {
            return None;
        }
        let rendered = render_static_scss_loop_body(
            loop_block.body.as_str(),
            loop_block.body_start,
            &next_values,
            context,
            assignment_ranges.as_slice(),
        )?;
        replacements.extend(rendered.replacements);
        append_static_scss_loop_rendered_body(
            &mut output,
            rendered.replacement.as_str(),
            loop_block.dialect,
        );
        current_values = next_values;
    }

    None
}

fn resolve_static_scss_for_loop_bound_with_bindings(
    value: &str,
    argument_values: &BTreeMap<String, String>,
    position: usize,
    context: StaticScssFunctionResolutionContext<'_>,
) -> Option<i32> {
    let resolution = resolve_static_scss_function_value_with_bindings(
        value,
        argument_values,
        position,
        STATIC_STYLESHEET_VALUE_RESOLUTION_FUEL_LIMIT,
        context,
    );
    if resolution.outcome != StaticStylesheetResolutionOutcome::Resolved {
        return None;
    }
    resolution.rendered_value?.trim().parse::<i32>().ok()
}

fn resolve_static_scss_loop_source_value_with_bindings(
    source: &str,
    argument_values: &BTreeMap<String, String>,
    position: usize,
    context: StaticScssFunctionResolutionContext<'_>,
) -> Option<String> {
    let resolution = resolve_static_scss_function_value_with_bindings(
        source,
        argument_values,
        position,
        STATIC_STYLESHEET_VALUE_RESOLUTION_FUEL_LIMIT,
        context,
    );
    (resolution.outcome == StaticStylesheetResolutionOutcome::Resolved)
        .then_some(resolution.rendered_value)
        .flatten()
}

fn render_static_scss_loop_body(
    body: &str,
    source_body_start: usize,
    argument_values: &BTreeMap<String, String>,
    context: StaticScssFunctionResolutionContext<'_>,
    removed_ranges: &[(usize, usize)],
) -> Option<StaticScssLoopRender> {
    let references =
        collect_static_stylesheet_variable_references(body, StaticStylesheetVariableKind::Scss)?;
    let mut edits = Vec::new();
    let mut replacements = Vec::new();
    for (start, end) in removed_ranges {
        edits.push(StaticStylesheetEvaluationEdit {
            start: *start,
            end: *end,
            replacement: String::new(),
        });
    }
    for reference in references {
        if static_stylesheet_position_is_inside_ranges(reference.start, removed_ranges) {
            continue;
        }
        let original_start = source_body_start + reference.start;
        let original_end = source_body_start + reference.end;
        let resolution = resolve_static_scss_function_value_with_bindings(
            reference.name.as_str(),
            argument_values,
            original_start,
            STATIC_STYLESHEET_VALUE_RESOLUTION_FUEL_LIMIT,
            context,
        );
        if resolution.outcome != StaticStylesheetResolutionOutcome::Resolved {
            return None;
        }
        let rendered_value = resolution.rendered_value?;
        replacements.push(resolved_replacement_value(
            reference.name.as_str(),
            original_start,
            original_end,
            rendered_value.as_str(),
        ));
        edits.push(StaticStylesheetEvaluationEdit {
            start: reference.start,
            end: reference.end,
            replacement: rendered_value,
        });
    }
    Some(StaticScssLoopRender {
        replacement: apply_static_stylesheet_evaluation_edits(body, edits)?,
        replacements,
    })
}

fn append_static_scss_loop_rendered_body(
    output: &mut String,
    rendered: &str,
    dialect: StyleDialect,
) {
    if dialect == StyleDialect::Sass && !output.is_empty() && !output.ends_with('\n') {
        output.push('\n');
    }
    output.push_str(rendered);
}

fn static_scss_source_contains_loop_candidate(source: &str) -> bool {
    let lower = source.to_ascii_lowercase();
    lower.contains("@for") || lower.contains("@each") || lower.contains("@while")
}

fn static_scss_token_is_loop_at_keyword(token: &LexedToken) -> bool {
    token.kind == SyntaxKind::AtKeyword
        && (token.text.eq_ignore_ascii_case("@for")
            || token.text.eq_ignore_ascii_case("@each")
            || token.text.eq_ignore_ascii_case("@while"))
}

const fn static_scss_loop_dialect_is_supported(dialect: StyleDialect) -> bool {
    matches!(dialect, StyleDialect::Scss | StyleDialect::Sass)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StaticScssWhileAssignmentOperator {
    Assign,
    PlusAssign,
    MinusAssign,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct StaticScssWhileAssignment {
    name: String,
    start: usize,
    end: usize,
    value_start: usize,
    value_end: usize,
    operator: StaticScssWhileAssignmentOperator,
}

fn collect_static_scss_while_body_assignments(
    body: &str,
    dialect: StyleDialect,
) -> Option<Vec<StaticScssWhileAssignment>> {
    let lexed = omena_parser::lex(body, dialect);
    let tokens = lexed.tokens();
    let mut assignments = Vec::new();
    let mut block_depth = 0usize;
    for (index, token) in tokens.iter().enumerate() {
        match token.kind {
            SyntaxKind::LeftBrace | SyntaxKind::SassIndent => {
                block_depth += 1;
                continue;
            }
            SyntaxKind::RightBrace | SyntaxKind::SassDedent => {
                block_depth = block_depth.checked_sub(1)?;
                continue;
            }
            _ => {}
        }
        if static_scss_while_body_effective_block_depth(block_depth, dialect) != 0
            || token.kind != SyntaxKind::ScssVariable
        {
            continue;
        }
        let operator_index = static_stylesheet_skip_trivia_tokens(tokens, index + 1);
        let operator = match tokens.get(operator_index)?.kind {
            SyntaxKind::Colon => StaticScssWhileAssignmentOperator::Assign,
            SyntaxKind::PlusEquals => StaticScssWhileAssignmentOperator::PlusAssign,
            SyntaxKind::MinusEquals => StaticScssWhileAssignmentOperator::MinusAssign,
            _ => continue,
        };
        let value_start_index = static_stylesheet_skip_trivia_tokens(tokens, operator_index + 1);
        let assignment_end = static_scss_while_assignment_end(tokens, value_start_index, dialect)?;
        assignments.push(StaticScssWhileAssignment {
            name: token.text.clone(),
            start: static_stylesheet_token_start(token),
            end: assignment_end.removal_end,
            value_start: static_stylesheet_token_start(&tokens[value_start_index]),
            value_end: assignment_end.value_end,
            operator,
        });
    }
    Some(assignments)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct StaticScssWhileAssignmentEnd {
    value_end: usize,
    removal_end: usize,
}

fn static_scss_while_assignment_end(
    tokens: &[LexedToken],
    value_start_index: usize,
    dialect: StyleDialect,
) -> Option<StaticScssWhileAssignmentEnd> {
    if dialect == StyleDialect::Sass {
        return static_scss_sass_while_assignment_end(tokens, value_start_index);
    }
    let end_index =
        static_stylesheet_value_end_token_until(tokens, value_start_index, tokens.len())?;
    Some(StaticScssWhileAssignmentEnd {
        value_end: static_stylesheet_token_start(&tokens[end_index]),
        removal_end: static_stylesheet_token_end(&tokens[end_index]),
    })
}

fn static_scss_sass_while_assignment_end(
    tokens: &[LexedToken],
    mut index: usize,
) -> Option<StaticScssWhileAssignmentEnd> {
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;
    while index < tokens.len() {
        match tokens[index].kind {
            SyntaxKind::LeftParen => paren_depth += 1,
            SyntaxKind::RightParen => paren_depth = paren_depth.checked_sub(1)?,
            SyntaxKind::LeftBracket => bracket_depth += 1,
            SyntaxKind::RightBracket => bracket_depth = bracket_depth.checked_sub(1)?,
            SyntaxKind::SassIndentedNewline
            | SyntaxKind::SassOptionalSemicolon
            | SyntaxKind::SassDedent
                if paren_depth == 0 && bracket_depth == 0 =>
            {
                return Some(StaticScssWhileAssignmentEnd {
                    value_end: static_stylesheet_token_start(&tokens[index]),
                    removal_end: static_stylesheet_token_end(&tokens[index]),
                });
            }
            _ => {}
        }
        index += 1;
    }
    tokens.last().map(|token| StaticScssWhileAssignmentEnd {
        value_end: static_stylesheet_token_end(token),
        removal_end: static_stylesheet_token_end(token),
    })
}

const fn static_scss_while_body_effective_block_depth(
    block_depth: usize,
    dialect: StyleDialect,
) -> usize {
    if matches!(dialect, StyleDialect::Sass) {
        block_depth.saturating_sub(1)
    } else {
        block_depth
    }
}

fn static_scss_while_body_references_follow_assignments(
    body: &str,
    assignments: &[StaticScssWhileAssignment],
) -> Option<bool> {
    let references =
        collect_static_stylesheet_variable_references(body, StaticStylesheetVariableKind::Scss)?;
    for reference in references {
        let Some(assignment) = assignments.iter().find(|assignment| {
            canonical_static_scss_variable_name(assignment.name.as_str())
                == canonical_static_scss_variable_name(reference.name.as_str())
        }) else {
            continue;
        };
        if static_stylesheet_position_is_inside_ranges(
            reference.start,
            &[(assignment.start, assignment.end)],
        ) {
            continue;
        }
        if reference.start < assignment.end {
            return Some(false);
        }
    }
    Some(true)
}

fn static_scss_while_next_frame_values(
    body: &str,
    assignments: &[StaticScssWhileAssignment],
    current_values: &BTreeMap<String, String>,
    source_body_start: usize,
    context: StaticScssFunctionResolutionContext<'_>,
) -> Option<BTreeMap<String, String>> {
    let mut next_values = current_values.clone();
    for assignment in assignments {
        let value = body
            .get(assignment.value_start..assignment.value_end)?
            .trim();
        let expression = match assignment.operator {
            StaticScssWhileAssignmentOperator::Assign => value.to_string(),
            StaticScssWhileAssignmentOperator::PlusAssign => {
                format!("{} + {}", assignment.name, value)
            }
            StaticScssWhileAssignmentOperator::MinusAssign => {
                format!("{} - {}", assignment.name, value)
            }
        };
        let resolution = resolve_static_scss_function_value_with_bindings(
            expression.as_str(),
            &next_values,
            source_body_start + assignment.value_start,
            STATIC_STYLESHEET_VALUE_RESOLUTION_FUEL_LIMIT,
            context,
        );
        if resolution.outcome != StaticStylesheetResolutionOutcome::Resolved {
            return None;
        }
        next_values.insert(
            canonical_static_scss_variable_name(assignment.name.as_str()),
            resolution.rendered_value?,
        );
    }
    Some(next_values)
}

fn static_scss_while_condition_is_active(
    condition: &str,
    argument_values: &BTreeMap<String, String>,
    position: usize,
    context: StaticScssFunctionResolutionContext<'_>,
) -> Option<bool> {
    let resolution = resolve_static_scss_function_value_with_bindings(
        condition,
        argument_values,
        position,
        STATIC_STYLESHEET_VALUE_RESOLUTION_FUEL_LIMIT,
        context,
    );
    if resolution.outcome == StaticStylesheetResolutionOutcome::Top {
        return None;
    }
    (context.truthiness_evaluator)(resolution.rendered_value?.as_str())
}

fn static_scss_loop_body_contains_known_function_call(
    body: &str,
    dialect: StyleDialect,
    context: StaticScssFunctionResolutionContext<'_>,
) -> bool {
    let declaration_names = context
        .declarations
        .iter()
        .map(|declaration| canonical_static_scss_function_name(declaration.name.as_str()))
        .collect::<BTreeSet<_>>();
    if declaration_names.is_empty() {
        return false;
    }
    let lexed = omena_parser::lex(body, dialect);
    let tokens = lexed.tokens();
    tokens.iter().enumerate().any(|(index, token)| {
        token.kind == SyntaxKind::Ident
            && declaration_names.contains(&canonical_static_scss_function_name(token.text.as_str()))
            && static_stylesheet_skip_trivia_tokens(tokens, index + 1) < tokens.len()
            && tokens[static_stylesheet_skip_trivia_tokens(tokens, index + 1)].kind
                == SyntaxKind::LeftParen
    })
}

fn static_scss_loop_replacement_is_static_css_subset(replacement: &str) -> bool {
    let lower = replacement.to_ascii_lowercase();
    !replacement.contains('$')
        && !lower.contains("@mixin")
        && !lower.contains("@function")
        && !lower.contains("@return")
        && !lower.contains("@include")
        && !lower.contains("@content")
        && !lower.contains("@if")
        && !lower.contains("@for")
        && !lower.contains("@each")
        && !lower.contains("@while")
}
