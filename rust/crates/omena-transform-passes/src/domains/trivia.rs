use omena_parser::{LexedToken, StyleDialect};
use omena_syntax::SyntaxKind;

use crate::runtime::lex_cache::lex_cached as lex;

use crate::helpers::tokens::{
    is_comment_token, next_non_comment_token_kind, previous_non_comment_token_kind,
};

pub(crate) fn normalize_css_whitespace_with_lexer(
    source: &str,
    dialect: StyleDialect,
) -> (String, usize) {
    let lexed = lex(source, dialect);
    let tokens = lexed.tokens();
    let mut output = String::with_capacity(source.len());
    let mut mutation_count = 0;

    for (index, token) in tokens.iter().enumerate() {
        if token.kind == SyntaxKind::Semicolon
            && matches!(
                next_non_comment_token_kind(tokens, index),
                Some(SyntaxKind::RightBrace)
            )
        {
            mutation_count += 1;
            continue;
        }

        if token.kind != SyntaxKind::Whitespace && token.kind != SyntaxKind::SassIndentedNewline {
            output.push_str(&token.text);
            continue;
        }

        let replacement = if whitespace_is_important_annotation_gap(tokens, index) {
            ""
        } else {
            whitespace_replacement_for_tokens(
                previous_non_comment_token_kind(tokens, index),
                next_non_comment_token_kind(tokens, index),
            )
        };
        if replacement != token.text {
            mutation_count += 1;
        }
        output.push_str(replacement);
    }

    (output, mutation_count)
}

fn whitespace_is_important_annotation_gap(tokens: &[LexedToken], index: usize) -> bool {
    let Some((previous_index, previous)) = previous_non_trivia_token(tokens, index) else {
        return false;
    };
    let Some((next_index, next)) = next_non_trivia_token(tokens, index) else {
        return false;
    };

    if next.kind == SyntaxKind::Important {
        return true;
    }

    if previous.kind == SyntaxKind::Delim
        && previous.text == "!"
        && next.kind == SyntaxKind::Ident
        && next.text.eq_ignore_ascii_case("important")
    {
        return true;
    }

    next.kind == SyntaxKind::Delim
        && next.text == "!"
        && previous_index < next_index
        && next_non_trivia_token(tokens, next_index).is_some_and(|(_, token)| {
            token.kind == SyntaxKind::Ident && token.text.eq_ignore_ascii_case("important")
        })
}

fn previous_non_trivia_token(tokens: &[LexedToken], index: usize) -> Option<(usize, &LexedToken)> {
    tokens[..index]
        .iter()
        .enumerate()
        .rev()
        .find(|(_, token)| !crate::helpers::tokens::is_trivia_token(token.kind))
}

fn next_non_trivia_token(tokens: &[LexedToken], index: usize) -> Option<(usize, &LexedToken)> {
    tokens
        .iter()
        .enumerate()
        .skip(index + 1)
        .find(|(_, token)| !crate::helpers::tokens::is_trivia_token(token.kind))
}

fn whitespace_replacement_for_tokens(
    previous: Option<SyntaxKind>,
    next: Option<SyntaxKind>,
) -> &'static str {
    match (previous, next) {
        (None, _) | (_, None) => "",
        (Some(previous), Some(next))
            if can_remove_whitespace_after(previous) || can_remove_whitespace_before(next) =>
        {
            ""
        }
        _ => " ",
    }
}

fn can_remove_whitespace_after(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::LeftBrace
            | SyntaxKind::RightBrace
            | SyntaxKind::LeftParen
            | SyntaxKind::LeftBracket
            | SyntaxKind::Comma
            | SyntaxKind::Colon
            | SyntaxKind::Semicolon
    )
}

fn can_remove_whitespace_before(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::LeftBrace
            | SyntaxKind::RightBrace
            | SyntaxKind::RightParen
            | SyntaxKind::RightBracket
            | SyntaxKind::Comma
            | SyntaxKind::Colon
            | SyntaxKind::Semicolon
    )
}

pub(crate) fn strip_css_comments_with_lexer(
    source: &str,
    dialect: StyleDialect,
) -> (String, usize) {
    let lexed = lex(source, dialect);
    let mut output = String::with_capacity(source.len());
    let mut cursor = 0;
    let mut removed_comment_count = 0;

    for token in lexed.tokens() {
        let start = u32::from(token.range.start()) as usize;
        let end = u32::from(token.range.end()) as usize;
        if start > cursor {
            output.push_str(&source[cursor..start]);
        }
        if is_comment_token(token.kind) {
            removed_comment_count += 1;
        } else {
            output.push_str(&source[start..end]);
        }
        cursor = end;
    }

    if cursor < source.len() {
        output.push_str(&source[cursor..]);
    }

    (output, removed_comment_count)
}
