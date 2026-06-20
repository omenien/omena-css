use omena_parser::LexedToken;
use omena_syntax::SyntaxKind;

use super::{
    StaticScssFunctionLocalScope, StaticScssFunctionLocalVariable,
    canonical_static_scss_variable_name, static_stylesheet_matching_token_index,
    static_stylesheet_skip_trivia_tokens, static_stylesheet_token_end,
    static_stylesheet_token_start, static_stylesheet_value_end_token_until,
    static_stylesheet_variable_name_is_safe,
};

pub(super) fn collect_static_scss_function_local_variables(
    source: &str,
    tokens: &[LexedToken],
    start: usize,
    end: usize,
) -> Option<Vec<StaticScssFunctionLocalVariable>> {
    let mut variables = Vec::new();
    let mut scope_stack = Vec::<StaticScssFunctionLocalScope>::new();
    let function_scope_start = tokens
        .get(start)
        .map(static_stylesheet_token_start)
        .or_else(|| tokens.get(end).map(static_stylesheet_token_start))?;
    let function_scope_end = tokens
        .get(end)
        .map(static_stylesheet_token_start)
        .unwrap_or(function_scope_start);
    let mut index = start;
    while index < end {
        while scope_stack
            .last()
            .is_some_and(|scope| index > scope.end_index)
        {
            scope_stack.pop();
        }
        match tokens[index].kind {
            SyntaxKind::LeftBrace | SyntaxKind::SassIndent => {
                let close_kind = if tokens[index].kind == SyntaxKind::SassIndent {
                    SyntaxKind::SassDedent
                } else {
                    SyntaxKind::RightBrace
                };
                let scope_end_index = static_stylesheet_matching_token_index(
                    tokens,
                    index,
                    tokens[index].kind,
                    close_kind,
                )?;
                scope_stack.push(StaticScssFunctionLocalScope {
                    end_index: scope_end_index,
                    span_start: static_stylesheet_token_end(&tokens[index]),
                    span_end: static_stylesheet_token_start(&tokens[scope_end_index]),
                });
                index += 1;
            }
            SyntaxKind::RightBrace | SyntaxKind::SassDedent => {
                if scope_stack
                    .last()
                    .is_some_and(|scope| scope.end_index == index)
                {
                    scope_stack.pop();
                }
                index += 1;
            }
            SyntaxKind::ScssVariable => {
                let colon_index = static_stylesheet_skip_trivia_tokens(tokens, index + 1);
                if tokens
                    .get(colon_index)
                    .is_none_or(|token| token.kind != SyntaxKind::Colon)
                {
                    index += 1;
                    continue;
                }
                let value_end_index =
                    static_stylesheet_value_end_token_until(tokens, colon_index + 1, end)?;
                let name = canonical_static_scss_variable_name(tokens[index].text.as_str());
                if !static_stylesheet_variable_name_is_safe(name.as_str()) {
                    return None;
                }
                let value_start = static_stylesheet_token_end(&tokens[colon_index]);
                let value_end = static_stylesheet_token_start(&tokens[value_end_index]);
                let value = source.get(value_start..value_end)?.trim();
                let (scope_start, scope_end) = scope_stack
                    .last()
                    .map(|scope| (scope.span_start, scope.span_end))
                    .unwrap_or((function_scope_start, function_scope_end));
                variables.push(StaticScssFunctionLocalVariable {
                    name,
                    value: value.to_string(),
                    span_start: static_stylesheet_token_start(&tokens[index]),
                    scope_start,
                    scope_end,
                });
                index = value_end_index + 1;
            }
            _ => {
                index += 1;
            }
        }
    }
    Some(variables)
}
