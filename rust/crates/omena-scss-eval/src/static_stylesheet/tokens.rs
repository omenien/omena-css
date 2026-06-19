use omena_parser::{LexedToken, StyleDialect};
use omena_syntax::SyntaxKind;

pub(super) fn static_stylesheet_matching_token_index(
    tokens: &[LexedToken],
    start: usize,
    left: SyntaxKind,
    right: SyntaxKind,
) -> Option<usize> {
    if tokens.get(start)?.kind != left {
        return None;
    }
    let mut depth = 0usize;
    for (index, token) in tokens.iter().enumerate().skip(start) {
        if token.kind == left {
            depth += 1;
        } else if token.kind == right {
            depth = depth.checked_sub(1)?;
            if depth == 0 {
                return Some(index);
            }
        }
    }
    None
}

pub(super) fn static_stylesheet_block_kinds_for_dialect(
    dialect: StyleDialect,
) -> (SyntaxKind, SyntaxKind) {
    match dialect {
        StyleDialect::Sass => (SyntaxKind::SassIndent, SyntaxKind::SassDedent),
        _ => (SyntaxKind::LeftBrace, SyntaxKind::RightBrace),
    }
}

pub(super) fn static_stylesheet_next_token_kind_index(
    tokens: &[LexedToken],
    mut index: usize,
    kind: SyntaxKind,
) -> Option<usize> {
    while index < tokens.len() {
        match tokens[index].kind {
            candidate if candidate == kind => return Some(index),
            SyntaxKind::Semicolon
            | SyntaxKind::SassOptionalSemicolon
            | SyntaxKind::RightBrace
            | SyntaxKind::SassDedent => return None,
            _ => index += 1,
        }
    }
    None
}

pub(super) fn static_stylesheet_scss_module_rule_semicolon(
    tokens: &[LexedToken],
    at_rule_index: usize,
) -> Option<usize> {
    let mut index = at_rule_index + 1;
    while index < tokens.len() {
        match tokens[index].kind {
            SyntaxKind::Semicolon | SyntaxKind::SassOptionalSemicolon => return Some(index),
            SyntaxKind::LeftBrace
            | SyntaxKind::SassIndent
            | SyntaxKind::RightBrace
            | SyntaxKind::SassDedent => return None,
            _ => index += 1,
        }
    }
    None
}

pub(super) fn static_stylesheet_position_is_inside_ranges(
    position: usize,
    ranges: &[(usize, usize)],
) -> bool {
    ranges
        .iter()
        .any(|(start, end)| *start <= position && position < *end)
}

pub(super) fn static_stylesheet_previous_token_is_body_start(
    tokens: &[LexedToken],
    index: usize,
) -> bool {
    tokens[..index]
        .iter()
        .rev()
        .all(|token| static_stylesheet_token_is_trivia(token.kind))
}

pub(super) fn static_stylesheet_declaration_value_end_token(
    tokens: &[LexedToken],
    index: usize,
) -> Option<usize> {
    static_stylesheet_value_end_token_until(tokens, index, tokens.len())
}

pub(super) fn static_stylesheet_value_end_token_until(
    tokens: &[LexedToken],
    mut index: usize,
    end: usize,
) -> Option<usize> {
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;
    while index < end {
        match tokens[index].kind {
            SyntaxKind::LeftParen => paren_depth += 1,
            SyntaxKind::RightParen => paren_depth = paren_depth.checked_sub(1)?,
            SyntaxKind::LeftBracket => bracket_depth += 1,
            SyntaxKind::RightBracket => bracket_depth = bracket_depth.checked_sub(1)?,
            SyntaxKind::Semicolon
            | SyntaxKind::SassOptionalSemicolon
            | SyntaxKind::RightBrace
            | SyntaxKind::SassDedent
                if paren_depth == 0 && bracket_depth == 0 =>
            {
                return Some(index);
            }
            _ => {}
        }
        index += 1;
    }
    None
}

pub(super) fn static_stylesheet_skip_trivia_tokens(
    tokens: &[LexedToken],
    mut index: usize,
) -> usize {
    while tokens
        .get(index)
        .is_some_and(|token| static_stylesheet_token_is_trivia(token.kind))
    {
        index += 1;
    }
    index
}

pub(super) fn static_stylesheet_token_is_trivia(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::Whitespace
            | SyntaxKind::SassIndentedNewline
            | SyntaxKind::LineComment
            | SyntaxKind::BlockComment
            | SyntaxKind::ScssSilentComment
    )
}

pub(super) fn parser_text_size_to_usize(value: u32) -> usize {
    value as usize
}

pub(super) fn static_stylesheet_token_start(token: &LexedToken) -> usize {
    parser_text_size_to_usize(token.range.start().into())
}

pub(super) fn static_stylesheet_token_end(token: &LexedToken) -> usize {
    parser_text_size_to_usize(token.range.end().into())
}
