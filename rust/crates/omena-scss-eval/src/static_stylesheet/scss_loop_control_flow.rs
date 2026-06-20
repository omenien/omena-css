use std::collections::{BTreeMap, BTreeSet};

use omena_parser::{LexedToken, StyleDialect};
use omena_syntax::SyntaxKind;

use crate::static_loop_frames::{
    parse_static_scss_each_loop_binding_frames, parse_static_scss_for_loop_header,
    static_scss_for_loop_values,
};

use super::{
    OmenaScssEvalResolvedReplacementV0, STATIC_STYLESHEET_VALUE_RESOLUTION_FUEL_LIMIT,
    StaticScssFunctionResolutionContext, StaticStylesheetEvaluationEdit,
    StaticStylesheetResolutionOutcome, StaticStylesheetVariableKind,
    apply_static_stylesheet_evaluation_edits, canonical_static_scss_function_name,
    canonical_static_scss_variable_name, collect_static_stylesheet_variable_references,
    resolve_static_scss_function_value_with_bindings, resolved_replacement_value,
    static_stylesheet_matching_token_index, static_stylesheet_position_is_inside_ranges,
    static_stylesheet_skip_trivia_tokens, static_stylesheet_token_end,
    static_stylesheet_token_start,
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
    if dialect != StyleDialect::Scss || !static_scss_source_contains_loop_candidate(source) {
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

        let Some(loop_block) = static_scss_loop_block(source, tokens, index) else {
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
    if dialect != StyleDialect::Scss || !static_scss_source_contains_loop_candidate(source) {
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
        if let Some(loop_block) = static_scss_loop_block(source, tokens, index) {
            ranges.push((loop_block.start, loop_block.end));
            index = loop_block.next_index;
        } else {
            index += 1;
        }
    }
    ranges
}

#[derive(Debug, Clone)]
struct StaticScssLoopBlock {
    at_rule_name: String,
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
    tokens: &[LexedToken],
    loop_index: usize,
) -> Option<StaticScssLoopBlock> {
    let body_open_index = (loop_index + 1..tokens.len())
        .find(|index| tokens[*index].kind == SyntaxKind::LeftBrace)?;
    let body_close_index = static_stylesheet_matching_token_index(
        tokens,
        body_open_index,
        SyntaxKind::LeftBrace,
        SyntaxKind::RightBrace,
    )?;
    let start = static_stylesheet_token_start(&tokens[loop_index]);
    let end = static_stylesheet_token_end(&tokens[body_close_index]);
    let header_start = static_stylesheet_token_end(&tokens[loop_index]);
    let header_end = static_stylesheet_token_start(&tokens[body_open_index]);
    let body_start = static_stylesheet_token_end(&tokens[body_open_index]);
    let body_end = static_stylesheet_token_start(&tokens[body_close_index]);
    Some(StaticScssLoopBlock {
        at_rule_name: tokens[loop_index].text.clone(),
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
    if static_scss_loop_body_contains_known_function_call(loop_block.body.as_str(), context) {
        return None;
    }
    if loop_block.at_rule_name.eq_ignore_ascii_case("@each") {
        return render_static_scss_each_loop_block(loop_block, context);
    }
    let header = parse_static_scss_for_loop_header(loop_block.header.as_str())?;
    let argument_values = BTreeMap::new();
    let start = resolve_static_scss_for_loop_bound(header.start_bound, loop_block.start, context)?;
    let end = resolve_static_scss_for_loop_bound(header.end_bound, loop_block.start, context)?;
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
        )?;
        replacements.extend(rendered.replacements);
        output.push_str(rendered.replacement.as_str());
    }

    if !static_scss_loop_replacement_is_static_css_subset(output.as_str()) {
        return None;
    }
    Some(StaticScssLoopRender {
        replacement: output,
        replacements,
    })
}

fn render_static_scss_each_loop_block(
    loop_block: &StaticScssLoopBlock,
    context: StaticScssFunctionResolutionContext<'_>,
) -> Option<StaticScssLoopRender> {
    let frames =
        parse_static_scss_each_loop_binding_frames(loop_block.header.as_str(), |source| {
            resolve_static_scss_loop_source_value(source, loop_block.start, context)
        })?;
    let mut output = String::with_capacity(loop_block.body.len().saturating_mul(frames.len()));
    let mut replacements = Vec::new();
    for frame in frames {
        let mut frame_values = BTreeMap::new();
        for (name, value) in frame {
            frame_values.insert(canonical_static_scss_variable_name(name.as_str()), value);
        }
        let rendered = render_static_scss_loop_body(
            loop_block.body.as_str(),
            loop_block.body_start,
            &frame_values,
            context,
        )?;
        replacements.extend(rendered.replacements);
        output.push_str(rendered.replacement.as_str());
    }
    if !static_scss_loop_replacement_is_static_css_subset(output.as_str()) {
        return None;
    }
    Some(StaticScssLoopRender {
        replacement: output,
        replacements,
    })
}

fn resolve_static_scss_for_loop_bound(
    value: &str,
    position: usize,
    context: StaticScssFunctionResolutionContext<'_>,
) -> Option<i32> {
    let resolution = resolve_static_scss_function_value_with_bindings(
        value,
        &BTreeMap::new(),
        position,
        STATIC_STYLESHEET_VALUE_RESOLUTION_FUEL_LIMIT,
        context,
    );
    if resolution.outcome != StaticStylesheetResolutionOutcome::Resolved {
        return None;
    }
    resolution.rendered_value?.trim().parse::<i32>().ok()
}

fn resolve_static_scss_loop_source_value(
    source: &str,
    position: usize,
    context: StaticScssFunctionResolutionContext<'_>,
) -> Option<String> {
    let resolution = resolve_static_scss_function_value_with_bindings(
        source,
        &BTreeMap::new(),
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
) -> Option<StaticScssLoopRender> {
    let references =
        collect_static_stylesheet_variable_references(body, StaticStylesheetVariableKind::Scss)?;
    let mut edits = Vec::new();
    let mut replacements = Vec::new();
    for reference in references {
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

fn static_scss_source_contains_loop_candidate(source: &str) -> bool {
    let lower = source.to_ascii_lowercase();
    lower.contains("@for") || lower.contains("@each")
}

fn static_scss_token_is_loop_at_keyword(token: &LexedToken) -> bool {
    token.kind == SyntaxKind::AtKeyword
        && (token.text.eq_ignore_ascii_case("@for") || token.text.eq_ignore_ascii_case("@each"))
}

fn static_scss_loop_body_contains_known_function_call(
    body: &str,
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
    let lexed = omena_parser::lex(body, StyleDialect::Scss);
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
