use std::collections::BTreeSet;

use omena_parser::{LexedToken, StyleDialect};
use omena_syntax::SyntaxKind;

use super::{
    StaticScssFunctionCall, StaticScssFunctionDeclaration, StaticScssMixinDeclaration,
    StaticScssMixinIncludeCall, canonical_static_scss_function_name,
    split_static_scss_function_arguments, static_stylesheet_matching_token_index,
    static_stylesheet_skip_trivia_tokens, static_stylesheet_token_end,
    static_stylesheet_token_start,
};

pub(super) fn collect_static_scss_function_calls(
    source: &str,
    tokens: &[LexedToken],
    declarations: &[StaticScssFunctionDeclaration],
) -> Option<Vec<StaticScssFunctionCall>> {
    let declaration_names = declarations
        .iter()
        .map(|declaration| canonical_static_scss_function_name(declaration.name.as_str()))
        .collect::<BTreeSet<_>>();
    let mut calls = Vec::new();
    for (name_index, token) in tokens.iter().enumerate() {
        if token.kind != SyntaxKind::Ident
            || !declaration_names
                .contains(&canonical_static_scss_function_name(token.text.as_str()))
            || static_scss_function_position_is_inside_declaration_header(
                declarations,
                static_stylesheet_token_start(token),
            )
        {
            continue;
        }
        let open_index = static_stylesheet_skip_trivia_tokens(tokens, name_index + 1);
        if tokens
            .get(open_index)
            .is_none_or(|token| token.kind != SyntaxKind::LeftParen)
        {
            continue;
        }
        let close_index = static_stylesheet_matching_token_index(
            tokens,
            open_index,
            SyntaxKind::LeftParen,
            SyntaxKind::RightParen,
        )?;
        let arguments = split_static_scss_function_arguments(source.get(
            static_stylesheet_token_end(&tokens[open_index])
                ..static_stylesheet_token_start(&tokens[close_index]),
        )?)?;
        calls.push(StaticScssFunctionCall {
            name: token.text.clone(),
            start: static_stylesheet_token_start(token),
            end: static_stylesheet_token_end(&tokens[close_index]),
            arguments,
        });
    }
    calls.sort_by_key(|call| (call.start, call.end));
    Some(calls)
}

pub(super) fn collect_static_scss_mixin_include_calls(
    source: &str,
    dialect: StyleDialect,
    tokens: &[LexedToken],
    declarations: &[StaticScssMixinDeclaration],
) -> Option<Vec<StaticScssMixinIncludeCall>> {
    let declaration_names = declarations
        .iter()
        .map(|declaration| canonical_static_scss_function_name(declaration.name.as_str()))
        .collect::<BTreeSet<_>>();
    let mut calls = Vec::new();
    let mut index = 0usize;
    while index < tokens.len() {
        let token = &tokens[index];
        if token.kind != SyntaxKind::AtKeyword || !token.text.eq_ignore_ascii_case("@include") {
            index += 1;
            continue;
        }
        let name_index = static_stylesheet_skip_trivia_tokens(tokens, index + 1);
        let name_token = tokens.get(name_index)?;
        if name_token.kind != SyntaxKind::Ident
            || !declaration_names.contains(&canonical_static_scss_function_name(
                name_token.text.as_str(),
            ))
        {
            index += 1;
            continue;
        }

        let after_name_index = static_stylesheet_skip_trivia_tokens(tokens, name_index + 1);
        let (arguments, after_arguments_index) = if tokens
            .get(after_name_index)
            .is_some_and(|candidate| candidate.kind == SyntaxKind::LeftParen)
        {
            let close_index = static_stylesheet_matching_token_index(
                tokens,
                after_name_index,
                SyntaxKind::LeftParen,
                SyntaxKind::RightParen,
            )?;
            let argument_text = source.get(
                static_stylesheet_token_end(&tokens[after_name_index])
                    ..static_stylesheet_token_start(&tokens[close_index]),
            )?;
            (
                split_static_scss_function_arguments(argument_text)?,
                static_stylesheet_skip_trivia_tokens(tokens, close_index + 1),
            )
        } else {
            (
                Vec::new(),
                static_stylesheet_skip_trivia_tokens(tokens, name_index + 1),
            )
        };
        let end_token = tokens.get(after_arguments_index)?;
        let valid_terminator = match dialect {
            StyleDialect::Sass => matches!(
                end_token.kind,
                SyntaxKind::SassOptionalSemicolon | SyntaxKind::SassDedent
            ),
            _ => end_token.kind == SyntaxKind::Semicolon,
        };
        if !valid_terminator {
            index += 1;
            continue;
        }
        calls.push(StaticScssMixinIncludeCall {
            name: name_token.text.clone(),
            start: static_stylesheet_token_start(token),
            end: static_stylesheet_token_end(end_token),
            arguments,
        });
        index = after_arguments_index + 1;
    }
    calls.sort_by_key(|call| (call.start, call.end));
    Some(calls)
}

fn static_scss_function_position_is_inside_declaration_header(
    declarations: &[StaticScssFunctionDeclaration],
    position: usize,
) -> bool {
    declarations
        .iter()
        .any(|declaration| position >= declaration.span_start && position < declaration.body_start)
}

pub(super) fn static_scss_function_call_is_inside_declaration_body(
    call: &StaticScssFunctionCall,
    declarations: &[StaticScssFunctionDeclaration],
) -> bool {
    declarations.iter().any(|declaration| {
        call.start >= declaration.body_start && call.start < declaration.body_end
    })
}

pub(super) fn static_scss_function_call_is_inside_mixin_declaration_body(
    call: &StaticScssFunctionCall,
    declarations: &[StaticScssMixinDeclaration],
) -> bool {
    declarations.iter().any(|declaration| {
        call.start >= declaration.body_start && call.start < declaration.body_end
    })
}

pub(super) fn static_scss_mixin_include_is_inside_declaration_body(
    call: &StaticScssMixinIncludeCall,
    declarations: &[StaticScssMixinDeclaration],
) -> bool {
    declarations.iter().any(|declaration| {
        call.start >= declaration.body_start && call.start < declaration.body_end
    })
}

pub(super) fn static_scss_mixin_include_is_inside_function_declaration_body(
    call: &StaticScssMixinIncludeCall,
    declarations: &[StaticScssFunctionDeclaration],
) -> bool {
    declarations.iter().any(|declaration| {
        call.start >= declaration.body_start && call.start < declaration.body_end
    })
}
