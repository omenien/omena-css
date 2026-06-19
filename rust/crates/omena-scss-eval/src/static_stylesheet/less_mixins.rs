use omena_parser::LexedToken;
use omena_syntax::SyntaxKind;

use super::{
    less_mixin_arguments::{
        collect_static_less_mixin_parameters, split_static_less_mixin_arguments,
    },
    model::{StaticLessMixinAccessor, StaticLessMixinCall, StaticLessMixinDeclaration},
    static_less_mixin_hash_name_is_safe, static_less_mixin_name_part_is_safe,
    static_less_variable_name_is_safe, static_stylesheet_matching_token_index,
    static_stylesheet_next_token_kind_index, static_stylesheet_property_name_is_safe,
    static_stylesheet_skip_trivia_tokens, static_stylesheet_token_end,
    static_stylesheet_token_is_trivia, static_stylesheet_token_start,
};

pub(super) fn static_less_mixin_ranges_from_calls(
    calls: &[StaticLessMixinCall],
) -> Vec<(usize, usize)> {
    calls.iter().map(|call| (call.start, call.end)).collect()
}

pub(super) fn static_less_mixin_accessor_ranges_from_accessors(
    accessors: &[StaticLessMixinAccessor],
) -> Vec<(usize, usize)> {
    accessors
        .iter()
        .map(|accessor| (accessor.start, accessor.end))
        .collect()
}

pub(super) fn collect_static_less_mixin_declarations(
    source: &str,
    tokens: &[LexedToken],
) -> Option<Vec<StaticLessMixinDeclaration>> {
    let mut declarations = Vec::new();
    let mut index = 0usize;
    while index < tokens.len() {
        let Some((name, open_index)) = static_less_mixin_signature_at(tokens, index) else {
            index += 1;
            continue;
        };
        let close_index = static_stylesheet_matching_token_index(
            tokens,
            open_index,
            SyntaxKind::LeftParen,
            SyntaxKind::RightParen,
        )?;
        let Some(body_open_index) =
            static_stylesheet_next_token_kind_index(tokens, close_index + 1, SyntaxKind::LeftBrace)
        else {
            index += 1;
            continue;
        };
        let guard =
            static_less_mixin_header_guard_text(source, tokens, close_index + 1, body_open_index)?;
        let Some(body_close_index) = static_stylesheet_matching_token_index(
            tokens,
            body_open_index,
            SyntaxKind::LeftBrace,
            SyntaxKind::RightBrace,
        ) else {
            index += 1;
            continue;
        };
        let parameters =
            collect_static_less_mixin_parameters(source, tokens, open_index + 1, close_index)?;
        declarations.push(StaticLessMixinDeclaration {
            name,
            parameters,
            guard,
            span_start: static_stylesheet_token_start(&tokens[index]),
            span_end: static_stylesheet_token_end(&tokens[body_close_index]),
            body_start: static_stylesheet_token_end(&tokens[body_open_index]),
            body_end: static_stylesheet_token_start(&tokens[body_close_index]),
        });
        index = body_close_index + 1;
    }
    Some(declarations)
}

pub(super) fn collect_static_less_mixin_calls(
    source: &str,
    tokens: &[LexedToken],
) -> Option<Vec<StaticLessMixinCall>> {
    let mut calls = Vec::new();
    let mut index = 0usize;
    while index < tokens.len() {
        if static_less_mixin_call_context_is_plain(tokens, index)
            && let Some((call, semicolon_index)) =
                static_less_namespace_mixin_call_at(source, tokens, index)
        {
            calls.push(call);
            index = semicolon_index + 1;
            continue;
        }
        let Some((name, open_index)) = static_less_mixin_signature_at(tokens, index) else {
            index += 1;
            continue;
        };
        if !static_less_mixin_call_context_is_plain(tokens, index) {
            index += 1;
            continue;
        }
        let close_index = static_stylesheet_matching_token_index(
            tokens,
            open_index,
            SyntaxKind::LeftParen,
            SyntaxKind::RightParen,
        )?;
        let Some((semicolon_index, important)) =
            static_less_mixin_call_semicolon_and_importance(source, tokens, close_index)
        else {
            index += 1;
            continue;
        };
        let arguments = split_static_less_mixin_arguments(source.get(
            static_stylesheet_token_end(&tokens[open_index])
                ..static_stylesheet_token_start(&tokens[close_index]),
        )?)?;
        calls.push(StaticLessMixinCall {
            namespace: None,
            namespace_arguments: Vec::new(),
            name,
            start: static_stylesheet_token_start(&tokens[index]),
            end: static_stylesheet_token_end(&tokens[semicolon_index]),
            important,
            arguments,
        });
        index = semicolon_index + 1;
    }
    calls.sort_by_key(|call| (call.start, call.end));
    Some(calls)
}

pub(super) fn collect_static_less_unsupported_mixin_call_suffix_ranges(
    source: &str,
    tokens: &[LexedToken],
) -> Option<Vec<(usize, usize)>> {
    let mut ranges = Vec::new();
    let mut index = 0usize;
    while index < tokens.len() {
        if static_less_mixin_call_context_is_plain(tokens, index)
            && let Some(((start, end), semicolon_index)) =
                static_less_namespace_unsupported_mixin_call_suffix_at(source, tokens, index)
        {
            ranges.push((start, end));
            index = semicolon_index + 1;
            continue;
        }
        let Some((_, open_index)) = static_less_mixin_signature_at(tokens, index) else {
            index += 1;
            continue;
        };
        if !static_less_mixin_call_context_is_plain(tokens, index) {
            index += 1;
            continue;
        }
        let close_index = static_stylesheet_matching_token_index(
            tokens,
            open_index,
            SyntaxKind::LeftParen,
            SyntaxKind::RightParen,
        )?;
        let Some((semicolon_index, suffix)) =
            static_less_mixin_call_semicolon_suffix(source, tokens, close_index)
        else {
            index += 1;
            continue;
        };
        if !static_less_mixin_call_suffix_is_supported(suffix) {
            ranges.push((
                static_stylesheet_token_start(&tokens[index]),
                static_stylesheet_token_end(&tokens[semicolon_index]),
            ));
        }
        index = semicolon_index + 1;
    }
    ranges.sort();
    Some(ranges)
}

pub(super) fn collect_static_less_mixin_accessors(
    source: &str,
    tokens: &[LexedToken],
) -> Option<Vec<StaticLessMixinAccessor>> {
    let mut accessors = Vec::new();
    let mut index = 0usize;
    while index < tokens.len() {
        let Some((name, open_index)) = static_less_mixin_signature_at(tokens, index) else {
            index += 1;
            continue;
        };
        let close_index = static_stylesheet_matching_token_index(
            tokens,
            open_index,
            SyntaxKind::LeftParen,
            SyntaxKind::RightParen,
        )?;
        let bracket_open_index = static_stylesheet_skip_trivia_tokens(tokens, close_index + 1);
        if tokens
            .get(bracket_open_index)
            .is_none_or(|token| token.kind != SyntaxKind::LeftBracket)
        {
            index += 1;
            continue;
        }
        let bracket_close_index = static_stylesheet_matching_token_index(
            tokens,
            bracket_open_index,
            SyntaxKind::LeftBracket,
            SyntaxKind::RightBracket,
        )?;
        let arguments = split_static_less_mixin_arguments(source.get(
            static_stylesheet_token_end(&tokens[open_index])
                ..static_stylesheet_token_start(&tokens[close_index]),
        )?)?;
        let member = static_less_mixin_accessor_member(source.get(
            static_stylesheet_token_end(&tokens[bracket_open_index])
                ..static_stylesheet_token_start(&tokens[bracket_close_index]),
        )?)?;
        accessors.push(StaticLessMixinAccessor {
            name,
            member,
            start: static_stylesheet_token_start(&tokens[index]),
            end: static_stylesheet_token_end(&tokens[bracket_close_index]),
            arguments,
        });
        index = bracket_close_index + 1;
    }
    accessors.sort_by_key(|accessor| (accessor.start, accessor.end));
    Some(accessors)
}

pub(super) fn static_less_mixin_call_context_is_plain(tokens: &[LexedToken], index: usize) -> bool {
    tokens
        .get(..index)
        .and_then(|prefix| {
            prefix
                .iter()
                .rev()
                .find(|token| !static_stylesheet_token_is_trivia(token.kind))
        })
        .is_none_or(|token| matches!(token.kind, SyntaxKind::LeftBrace | SyntaxKind::Semicolon))
}

fn static_less_mixin_accessor_member(member: &str) -> Option<String> {
    let member = member.trim();
    if static_less_variable_name_is_safe(member) || static_stylesheet_property_name_is_safe(member)
    {
        return Some(member.to_string());
    }
    None
}

fn static_less_namespace_mixin_call_at(
    source: &str,
    tokens: &[LexedToken],
    index: usize,
) -> Option<(StaticLessMixinCall, usize)> {
    let (namespace, after_namespace_index) = static_less_namespace_name_at(tokens, index)?;
    let namespace_arguments_index =
        static_stylesheet_skip_trivia_tokens(tokens, after_namespace_index);
    let (namespace_arguments, separator_index) = if tokens
        .get(namespace_arguments_index)
        .is_some_and(|token| token.kind == SyntaxKind::LeftParen)
    {
        let namespace_arguments_close_index = static_stylesheet_matching_token_index(
            tokens,
            namespace_arguments_index,
            SyntaxKind::LeftParen,
            SyntaxKind::RightParen,
        )?;
        let arguments = split_static_less_mixin_arguments(source.get(
            static_stylesheet_token_end(&tokens[namespace_arguments_index])
                ..static_stylesheet_token_start(&tokens[namespace_arguments_close_index]),
        )?)?;
        (
            arguments,
            static_stylesheet_skip_trivia_tokens(tokens, namespace_arguments_close_index + 1),
        )
    } else {
        (Vec::new(), namespace_arguments_index)
    };
    if tokens
        .get(separator_index)
        .is_none_or(|token| token.kind != SyntaxKind::GreaterThan)
    {
        return None;
    }
    let call_index = static_stylesheet_skip_trivia_tokens(tokens, separator_index + 1);
    let (name, open_index) = static_less_mixin_signature_at(tokens, call_index)?;
    let close_index = static_stylesheet_matching_token_index(
        tokens,
        open_index,
        SyntaxKind::LeftParen,
        SyntaxKind::RightParen,
    )?;
    let (semicolon_index, important) =
        static_less_mixin_call_semicolon_and_importance(source, tokens, close_index)?;
    let arguments = split_static_less_mixin_arguments(source.get(
        static_stylesheet_token_end(&tokens[open_index])
            ..static_stylesheet_token_start(&tokens[close_index]),
    )?)?;
    Some((
        StaticLessMixinCall {
            namespace: Some(namespace),
            namespace_arguments,
            name,
            start: static_stylesheet_token_start(&tokens[index]),
            end: static_stylesheet_token_end(&tokens[semicolon_index]),
            important,
            arguments,
        },
        semicolon_index,
    ))
}

fn static_less_namespace_unsupported_mixin_call_suffix_at(
    source: &str,
    tokens: &[LexedToken],
    index: usize,
) -> Option<((usize, usize), usize)> {
    let (_, after_namespace_index) = static_less_namespace_name_at(tokens, index)?;
    let namespace_arguments_index =
        static_stylesheet_skip_trivia_tokens(tokens, after_namespace_index);
    let separator_index = if tokens
        .get(namespace_arguments_index)
        .is_some_and(|token| token.kind == SyntaxKind::LeftParen)
    {
        let namespace_arguments_close_index = static_stylesheet_matching_token_index(
            tokens,
            namespace_arguments_index,
            SyntaxKind::LeftParen,
            SyntaxKind::RightParen,
        )?;
        static_stylesheet_skip_trivia_tokens(tokens, namespace_arguments_close_index + 1)
    } else {
        namespace_arguments_index
    };
    if tokens
        .get(separator_index)
        .is_none_or(|token| token.kind != SyntaxKind::GreaterThan)
    {
        return None;
    }
    let call_index = static_stylesheet_skip_trivia_tokens(tokens, separator_index + 1);
    let (_, open_index) = static_less_mixin_signature_at(tokens, call_index)?;
    let close_index = static_stylesheet_matching_token_index(
        tokens,
        open_index,
        SyntaxKind::LeftParen,
        SyntaxKind::RightParen,
    )?;
    let (semicolon_index, suffix) =
        static_less_mixin_call_semicolon_suffix(source, tokens, close_index)?;
    (!static_less_mixin_call_suffix_is_supported(suffix)).then_some((
        (
            static_stylesheet_token_start(&tokens[index]),
            static_stylesheet_token_end(&tokens[semicolon_index]),
        ),
        semicolon_index,
    ))
}

fn static_less_namespace_name_at(tokens: &[LexedToken], index: usize) -> Option<(String, usize)> {
    let token = tokens.get(index)?;
    if token.kind == SyntaxKind::Hash {
        if !static_less_mixin_hash_name_is_safe(token.text.as_str()) {
            return None;
        }
        return Some((token.text.clone(), index + 1));
    }
    if token.kind != SyntaxKind::Dot {
        return None;
    }
    let name_index = static_stylesheet_skip_trivia_tokens(tokens, index + 1);
    let name_token = tokens.get(name_index)?;
    if !matches!(
        name_token.kind,
        SyntaxKind::Ident | SyntaxKind::CustomPropertyName
    ) || !static_less_mixin_name_part_is_safe(name_token.text.as_str())
    {
        return None;
    }
    Some((format!(".{}", name_token.text), name_index + 1))
}

fn static_less_mixin_call_semicolon_and_importance(
    source: &str,
    tokens: &[LexedToken],
    close_index: usize,
) -> Option<(usize, bool)> {
    let (index, suffix) = static_less_mixin_call_semicolon_suffix(source, tokens, close_index)?;
    if suffix.is_empty() {
        return Some((index, false));
    }
    if suffix.eq_ignore_ascii_case("!important") {
        return Some((index, true));
    }
    None
}

fn static_less_mixin_call_semicolon_suffix<'a>(
    source: &'a str,
    tokens: &[LexedToken],
    close_index: usize,
) -> Option<(usize, &'a str)> {
    let suffix_start = static_stylesheet_token_end(tokens.get(close_index)?);
    for (index, token) in tokens.iter().enumerate().skip(close_index + 1) {
        if token.kind != SyntaxKind::Semicolon {
            continue;
        }
        let suffix = source
            .get(suffix_start..static_stylesheet_token_start(token))?
            .trim();
        return Some((index, suffix));
    }
    None
}

fn static_less_mixin_call_suffix_is_supported(suffix: &str) -> bool {
    suffix.is_empty() || suffix.eq_ignore_ascii_case("!important")
}

fn static_less_mixin_signature_at(tokens: &[LexedToken], index: usize) -> Option<(String, usize)> {
    let token = tokens.get(index)?;
    if token.kind == SyntaxKind::Hash {
        if !static_less_mixin_hash_name_is_safe(token.text.as_str()) {
            return None;
        }
        let open_index = static_stylesheet_skip_trivia_tokens(tokens, index + 1);
        if tokens
            .get(open_index)
            .is_none_or(|token| token.kind != SyntaxKind::LeftParen)
        {
            return None;
        }
        return Some((token.text.clone(), open_index));
    }

    if token.kind != SyntaxKind::Dot {
        return None;
    }
    let name_index = static_stylesheet_skip_trivia_tokens(tokens, index + 1);
    let name_token = tokens.get(name_index)?;
    if !matches!(
        name_token.kind,
        SyntaxKind::Ident | SyntaxKind::CustomPropertyName
    ) || !static_less_mixin_name_part_is_safe(name_token.text.as_str())
    {
        return None;
    }
    let open_index = static_stylesheet_skip_trivia_tokens(tokens, name_index + 1);
    if tokens
        .get(open_index)
        .is_none_or(|token| token.kind != SyntaxKind::LeftParen)
    {
        return None;
    }
    Some((format!(".{}", name_token.text), open_index))
}

fn static_less_mixin_header_guard_text(
    source: &str,
    tokens: &[LexedToken],
    start: usize,
    end: usize,
) -> Option<Option<String>> {
    let first = static_stylesheet_skip_trivia_tokens(tokens, start);
    if first >= end {
        return Some(None);
    }
    let first_token = tokens.get(first)?;
    if first_token.kind != SyntaxKind::Ident || !first_token.text.eq_ignore_ascii_case("when") {
        return None;
    }
    let guard_start = static_stylesheet_token_start(first_token);
    let guard_end = tokens.get(end).map(static_stylesheet_token_start)?;
    source
        .get(guard_start..guard_end)
        .map(str::trim)
        .map(ToOwned::to_owned)
        .map(Some)
}
