use omena_parser::{LexedToken, StyleDialect};
use omena_syntax::SyntaxKind;

use super::{
    model::{StaticScssFunctionDeclaration, StaticScssMixinDeclaration},
    safety::static_scss_callable_name_is_safe,
    scss_arguments::collect_static_scss_function_parameters,
    scss_function_locals::collect_static_scss_function_local_variables,
    scss_return_clauses::{
        collect_static_scss_function_return_clauses, static_scss_function_return_clauses_are_safe,
    },
    tokens::{
        static_stylesheet_block_kinds_for_dialect, static_stylesheet_matching_token_index,
        static_stylesheet_next_token_kind_index, static_stylesheet_skip_trivia_tokens,
        static_stylesheet_token_end, static_stylesheet_token_start,
    },
};

pub(super) fn collect_static_scss_function_declarations(
    source: &str,
    dialect: StyleDialect,
    tokens: &[LexedToken],
) -> Option<Vec<StaticScssFunctionDeclaration>> {
    let mut declarations = Vec::new();
    let mut index = 0usize;
    while index < tokens.len() {
        if tokens[index].kind != SyntaxKind::AtKeyword
            || !tokens[index].text.eq_ignore_ascii_case("@function")
        {
            index += 1;
            continue;
        }

        let name_index = static_stylesheet_skip_trivia_tokens(tokens, index + 1);
        let name_token = tokens.get(name_index)?;
        if name_token.kind != SyntaxKind::Ident
            || !static_scss_callable_name_is_safe(name_token.text.as_str())
        {
            index += 1;
            continue;
        }

        let parameter_open_index = static_stylesheet_skip_trivia_tokens(tokens, name_index + 1);
        if tokens
            .get(parameter_open_index)
            .is_none_or(|token| token.kind != SyntaxKind::LeftParen)
        {
            index += 1;
            continue;
        }
        let parameter_close_index = static_stylesheet_matching_token_index(
            tokens,
            parameter_open_index,
            SyntaxKind::LeftParen,
            SyntaxKind::RightParen,
        )?;
        let parameters = collect_static_scss_function_parameters(
            source,
            tokens,
            parameter_open_index + 1,
            parameter_close_index,
        )?;

        let body_open_index =
            static_stylesheet_skip_trivia_tokens(tokens, parameter_close_index + 1);
        let (body_open_kind, body_close_kind) = static_stylesheet_block_kinds_for_dialect(dialect);
        if tokens
            .get(body_open_index)
            .is_none_or(|token| token.kind != body_open_kind)
        {
            index += 1;
            continue;
        }
        let body_close_index = static_stylesheet_matching_token_index(
            tokens,
            body_open_index,
            body_open_kind,
            body_close_kind,
        )?;
        let return_clauses = collect_static_scss_function_return_clauses(
            source,
            tokens,
            body_open_index + 1,
            body_close_index,
        )?;
        let local_variables = collect_static_scss_function_local_variables(
            source,
            tokens,
            body_open_index + 1,
            body_close_index,
        )?;
        if !static_scss_function_return_clauses_are_safe(return_clauses.as_slice()) {
            index = body_close_index + 1;
            continue;
        }

        declarations.push(StaticScssFunctionDeclaration {
            name: name_token.text.clone(),
            parameters,
            local_variables,
            return_clauses,
            span_start: static_stylesheet_token_start(&tokens[index]),
            span_end: static_stylesheet_token_end(&tokens[body_close_index]),
            body_start: static_stylesheet_token_end(&tokens[body_open_index]),
            body_end: static_stylesheet_token_start(&tokens[body_close_index]),
        });
        index = body_close_index + 1;
    }
    Some(declarations)
}

pub(super) fn collect_static_scss_mixin_declarations(
    source: &str,
    dialect: StyleDialect,
    tokens: &[LexedToken],
) -> Option<Vec<StaticScssMixinDeclaration>> {
    let mut declarations = Vec::new();
    let mut index = 0usize;
    while index < tokens.len() {
        if tokens[index].kind != SyntaxKind::AtKeyword
            || !tokens[index].text.eq_ignore_ascii_case("@mixin")
        {
            index += 1;
            continue;
        }

        let name_index = static_stylesheet_skip_trivia_tokens(tokens, index + 1);
        let name_token = tokens.get(name_index)?;
        if name_token.kind != SyntaxKind::Ident
            || !static_scss_callable_name_is_safe(name_token.text.as_str())
        {
            index += 1;
            continue;
        }
        let after_name_index = static_stylesheet_skip_trivia_tokens(tokens, name_index + 1);
        let (parameters, body_search_index) = if tokens
            .get(after_name_index)
            .is_some_and(|token| token.kind == SyntaxKind::LeftParen)
        {
            let parameter_close_index = static_stylesheet_matching_token_index(
                tokens,
                after_name_index,
                SyntaxKind::LeftParen,
                SyntaxKind::RightParen,
            )?;
            let parameters = collect_static_scss_function_parameters(
                source,
                tokens,
                after_name_index + 1,
                parameter_close_index,
            )?;
            (parameters, parameter_close_index + 1)
        } else {
            (Vec::new(), name_index + 1)
        };
        let (body_open_kind, body_close_kind) = static_stylesheet_block_kinds_for_dialect(dialect);
        let Some(body_open_index) =
            static_stylesheet_next_token_kind_index(tokens, body_search_index, body_open_kind)
        else {
            index += 1;
            continue;
        };
        let Some(body_close_index) = static_stylesheet_matching_token_index(
            tokens,
            body_open_index,
            body_open_kind,
            body_close_kind,
        ) else {
            index += 1;
            continue;
        };
        declarations.push(StaticScssMixinDeclaration {
            name: name_token.text.clone(),
            parameters,
            span_start: static_stylesheet_token_start(&tokens[index]),
            span_end: static_stylesheet_token_end(&tokens[body_close_index]),
            body_start: static_stylesheet_token_end(&tokens[body_open_index]),
            body_end: static_stylesheet_token_start(&tokens[body_close_index]),
        });
        index = body_close_index + 1;
    }
    Some(declarations)
}
