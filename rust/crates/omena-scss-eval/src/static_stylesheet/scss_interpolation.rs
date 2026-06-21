use std::collections::BTreeSet;

use omena_parser::LexedToken;
use omena_syntax::SyntaxKind;

use super::{
    OmenaScssEvalResolvedReplacementV0, StaticStylesheetEvaluationEdit, StaticStylesheetScope,
    StaticStylesheetScopedVariableDeclaration, resolved_replacement_value,
    scss_variables::resolve_static_scss_variable_value_at_position,
    static_stylesheet_position_is_inside_ranges, static_stylesheet_scope_for_position,
    static_stylesheet_skip_trivia_tokens, static_stylesheet_token_end,
    static_stylesheet_token_is_trivia, static_stylesheet_token_start,
    static_stylesheet_variable_name_is_safe,
};

pub(super) struct StaticScssInterpolationEvaluationEdits {
    pub(super) edits: Vec<StaticStylesheetEvaluationEdit>,
    pub(super) replacements: Vec<OmenaScssEvalResolvedReplacementV0>,
    pub(super) preserved_dynamic_interpolation_count: usize,
}

pub(super) fn collect_static_scss_interpolation_ranges(
    tokens: &[LexedToken],
) -> Vec<(usize, usize)> {
    let mut ranges = Vec::new();
    let mut index = 0usize;
    while index < tokens.len() {
        if tokens[index].kind != SyntaxKind::ScssInterpolationStart {
            index += 1;
            continue;
        }
        let Some(end_index) = static_scss_interpolation_end_token_index(tokens, index) else {
            index += 1;
            continue;
        };
        ranges.push((
            static_stylesheet_token_start(&tokens[index]),
            static_stylesheet_token_end(&tokens[end_index]),
        ));
        index = end_index + 1;
    }
    ranges
}

pub(super) fn collect_static_scss_interpolation_evaluation_edits(
    tokens: &[LexedToken],
    scopes: &[StaticStylesheetScope],
    declarations: &[StaticStylesheetScopedVariableDeclaration],
    excluded_ranges: &[(usize, usize)],
) -> Option<StaticScssInterpolationEvaluationEdits> {
    let mut edits = Vec::new();
    let mut replacements = Vec::new();
    let mut preserved_dynamic_interpolation_count = 0usize;
    let mut index = 0usize;
    while index < tokens.len() {
        if tokens[index].kind != SyntaxKind::ScssInterpolationStart {
            index += 1;
            continue;
        }
        let interpolation_start = static_stylesheet_token_start(&tokens[index]);
        let Some(end_index) = static_scss_interpolation_end_token_index(tokens, index) else {
            preserved_dynamic_interpolation_count += 1;
            index += 1;
            continue;
        };
        let interpolation_end = static_stylesheet_token_end(&tokens[end_index]);
        if static_stylesheet_position_is_inside_ranges(interpolation_start, excluded_ranges) {
            index = end_index + 1;
            continue;
        }
        if !static_scss_interpolation_is_inside_declaration_value(tokens, interpolation_start) {
            preserved_dynamic_interpolation_count += 1;
            index = end_index + 1;
            continue;
        }
        let Some(variable_token_index) =
            static_scss_interpolation_single_variable_token_index(tokens, index, end_index)
        else {
            preserved_dynamic_interpolation_count += 1;
            index = end_index + 1;
            continue;
        };
        let variable_name = tokens[variable_token_index].text.as_str();
        if !variable_name.starts_with('$')
            || !static_stylesheet_variable_name_is_safe(&variable_name[1..])
        {
            preserved_dynamic_interpolation_count += 1;
            index = end_index + 1;
            continue;
        }
        let _scope_id = static_stylesheet_scope_for_position(scopes, interpolation_start)?;
        let mut stack = BTreeSet::new();
        let Some(replacement) = resolve_static_scss_variable_value_at_position(
            variable_name,
            interpolation_start,
            scopes,
            declarations,
            &mut stack,
        ) else {
            preserved_dynamic_interpolation_count += 1;
            index = end_index + 1;
            continue;
        };
        replacements.push(resolved_replacement_value(
            variable_name,
            interpolation_start,
            interpolation_end,
            replacement.as_str(),
        ));
        edits.push(StaticStylesheetEvaluationEdit {
            start: interpolation_start,
            end: interpolation_end,
            replacement,
        });
        index = end_index + 1;
    }
    Some(StaticScssInterpolationEvaluationEdits {
        edits,
        replacements,
        preserved_dynamic_interpolation_count,
    })
}

fn static_scss_interpolation_end_token_index(
    tokens: &[LexedToken],
    interpolation_start_index: usize,
) -> Option<usize> {
    let mut depth = 0usize;
    for (index, token) in tokens.iter().enumerate().skip(interpolation_start_index) {
        match token.kind {
            SyntaxKind::ScssInterpolationStart => depth += 1,
            SyntaxKind::ScssInterpolationEnd => {
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

fn static_scss_interpolation_single_variable_token_index(
    tokens: &[LexedToken],
    start_index: usize,
    end_index: usize,
) -> Option<usize> {
    let variable_index = static_stylesheet_skip_trivia_tokens(tokens, start_index + 1);
    if variable_index >= end_index || tokens[variable_index].kind != SyntaxKind::ScssVariable {
        return None;
    }
    let after_variable_index = static_stylesheet_skip_trivia_tokens(tokens, variable_index + 1);
    (after_variable_index == end_index).then_some(variable_index)
}

fn static_scss_interpolation_is_inside_declaration_value(
    tokens: &[LexedToken],
    position: usize,
) -> bool {
    let Some(mut index) = tokens.iter().position(|token| {
        position >= static_stylesheet_token_start(token)
            && position < static_stylesheet_token_end(token)
    }) else {
        return false;
    };
    while index > 0 {
        index -= 1;
        match tokens[index].kind {
            kind if static_stylesheet_token_is_trivia(kind) => {}
            SyntaxKind::Colon => return true,
            SyntaxKind::LeftBrace | SyntaxKind::RightBrace | SyntaxKind::Semicolon => return false,
            _ => {}
        }
    }
    false
}
