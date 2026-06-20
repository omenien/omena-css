use std::collections::BTreeSet;

use omena_parser::{LexedToken, StyleDialect};
use omena_syntax::SyntaxKind;

use super::{
    StaticScssFunctionCall, StaticScssFunctionDeclaration, StaticScssMixinDeclaration,
    StaticScssMixinIncludeCall, canonical_static_scss_function_name,
    collect_static_scss_content_parameters, split_static_scss_function_arguments,
    static_stylesheet_matching_token_index, static_stylesheet_skip_trivia_tokens,
    static_stylesheet_token_end, static_stylesheet_token_start,
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
    collect_static_scss_mixin_include_calls_with_options(
        source,
        dialect,
        tokens,
        declarations,
        true,
    )
}

pub(super) fn collect_static_scss_mixin_include_calls_without_sass_content_blocks(
    source: &str,
    dialect: StyleDialect,
    tokens: &[LexedToken],
    declarations: &[StaticScssMixinDeclaration],
) -> Option<Vec<StaticScssMixinIncludeCall>> {
    collect_static_scss_mixin_include_calls_with_options(
        source,
        dialect,
        tokens,
        declarations,
        false,
    )
}

fn collect_static_scss_mixin_include_calls_with_options(
    source: &str,
    dialect: StyleDialect,
    tokens: &[LexedToken],
    declarations: &[StaticScssMixinDeclaration],
    allow_sass_content_blocks: bool,
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
        let (content_parameters, after_using_index) =
            if tokens.get(after_arguments_index).is_some_and(|token| {
                token.kind == SyntaxKind::Ident && token.text.eq_ignore_ascii_case("using")
            }) {
                let open_index =
                    static_stylesheet_skip_trivia_tokens(tokens, after_arguments_index + 1);
                if tokens
                    .get(open_index)
                    .is_none_or(|token| token.kind != SyntaxKind::LeftParen)
                {
                    index += 1;
                    continue;
                }
                let close_index = static_stylesheet_matching_token_index(
                    tokens,
                    open_index,
                    SyntaxKind::LeftParen,
                    SyntaxKind::RightParen,
                )?;
                (
                    collect_static_scss_content_parameters(
                        source,
                        tokens,
                        open_index + 1,
                        close_index,
                    )?,
                    static_stylesheet_skip_trivia_tokens(tokens, close_index + 1),
                )
            } else {
                (Vec::new(), after_arguments_index)
            };
        let content_block_kinds = match dialect {
            StyleDialect::Sass => (SyntaxKind::SassIndent, SyntaxKind::SassDedent),
            _ => (SyntaxKind::LeftBrace, SyntaxKind::RightBrace),
        };
        let can_collect_content_block = dialect != StyleDialect::Sass || allow_sass_content_blocks;
        if can_collect_content_block
            && tokens
                .get(after_using_index)
                .is_some_and(|token| token.kind == content_block_kinds.0)
        {
            let close_index = static_stylesheet_matching_token_index(
                tokens,
                after_using_index,
                content_block_kinds.0,
                content_block_kinds.1,
            )?;
            let content_body = source
                .get(
                    static_stylesheet_token_end(&tokens[after_using_index])
                        ..static_stylesheet_token_start(&tokens[close_index]),
                )?
                .to_string();
            calls.push(StaticScssMixinIncludeCall {
                name: name_token.text.clone(),
                start: static_stylesheet_token_start(token),
                end: static_stylesheet_token_end(&tokens[close_index]),
                arguments,
                content_body: Some(content_body),
                content_parameters,
            });
            index = close_index + 1;
            continue;
        }
        let end_token = tokens.get(after_using_index)?;
        let valid_terminator = match dialect {
            StyleDialect::Sass => {
                matches!(
                    end_token.kind,
                    SyntaxKind::SassOptionalSemicolon | SyntaxKind::SassDedent
                ) || (!allow_sass_content_blocks && end_token.kind == SyntaxKind::SassIndent)
            }
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
            content_body: None,
            content_parameters,
        });
        index = after_using_index + 1;
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
