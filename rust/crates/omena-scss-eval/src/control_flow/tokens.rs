use omena_parser::LexedToken;
use omena_syntax::SyntaxKind;

pub(super) fn tokens_between_are_trivia(tokens: &[LexedToken], start: usize, end: usize) -> bool {
    start <= end
        && end <= tokens.len()
        && tokens[start..end]
            .iter()
            .all(|token| is_trivia_token(token.kind))
}

pub(super) fn token_range_start(token: &LexedToken) -> usize {
    token.range.start().into()
}

pub(super) fn token_range_end(token: &LexedToken) -> usize {
    token.range.end().into()
}

pub(super) fn matching_right_paren_token_index(
    tokens: &[LexedToken],
    left_paren_index: usize,
) -> Option<usize> {
    let mut depth = 0usize;
    for (index, token) in tokens.iter().enumerate().skip(left_paren_index) {
        match token.kind {
            SyntaxKind::LeftParen => depth += 1,
            SyntaxKind::RightParen => {
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

pub(super) fn matching_right_brace_token_index(
    tokens: &[LexedToken],
    left_brace_index: usize,
) -> Option<usize> {
    let mut depth = 0usize;
    for (index, token) in tokens.iter().enumerate().skip(left_brace_index) {
        match token.kind {
            SyntaxKind::LeftBrace => depth += 1,
            SyntaxKind::RightBrace => {
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

pub(super) fn next_non_trivia_token_index(
    tokens: &[LexedToken],
    mut index: usize,
) -> Option<usize> {
    while tokens
        .get(index)
        .is_some_and(|token| is_trivia_token(token.kind))
    {
        index += 1;
    }
    (index < tokens.len()).then_some(index)
}

pub(super) fn declaration_end_token_index(
    tokens: &[LexedToken],
    mut index: usize,
) -> Option<usize> {
    let mut paren_depth = 0usize;
    while index < tokens.len() {
        match tokens[index].kind {
            SyntaxKind::LeftParen => paren_depth += 1,
            SyntaxKind::RightParen => paren_depth = paren_depth.checked_sub(1)?,
            SyntaxKind::Semicolon | SyntaxKind::SassOptionalSemicolon if paren_depth == 0 => {
                return Some(index);
            }
            _ => {}
        }
        index += 1;
    }
    None
}

const fn is_trivia_token(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::Whitespace
            | SyntaxKind::LineComment
            | SyntaxKind::BlockComment
            | SyntaxKind::ScssSilentComment
            | SyntaxKind::SassIndentedNewline
    )
}
