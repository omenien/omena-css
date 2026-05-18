use omena_parser::LexedToken;
use omena_syntax::SyntaxKind;

pub(crate) fn skip_whitespace_tokens(
    tokens: &[LexedToken],
    mut index: usize,
    end_exclusive: usize,
) -> usize {
    while index < end_exclusive && tokens[index].kind == SyntaxKind::Whitespace {
        index += 1;
    }
    index
}

pub(crate) fn matching_right_brace_index(
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

pub(crate) fn matching_right_paren_index(
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

pub(crate) fn previous_non_comment_token_kind(
    tokens: &[LexedToken],
    index: usize,
) -> Option<SyntaxKind> {
    tokens[..index]
        .iter()
        .rev()
        .find(|token| !is_trivia_token(token.kind))
        .map(|token| token.kind)
}

pub(crate) fn next_non_comment_token_kind(
    tokens: &[LexedToken],
    index: usize,
) -> Option<SyntaxKind> {
    tokens
        .get(index + 1..)
        .unwrap_or_default()
        .iter()
        .find(|token| !is_trivia_token(token.kind))
        .map(|token| token.kind)
}

pub(crate) fn token_start(token: &LexedToken) -> usize {
    u32::from(token.range.start()) as usize
}

pub(crate) fn token_end(token: &LexedToken) -> usize {
    u32::from(token.range.end()) as usize
}

pub(crate) fn is_comment_token(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::LineComment | SyntaxKind::BlockComment | SyntaxKind::ScssSilentComment
    )
}

pub(crate) fn is_trivia_token(kind: SyntaxKind) -> bool {
    is_comment_token(kind)
        || matches!(
            kind,
            SyntaxKind::Whitespace | SyntaxKind::SassIndentedNewline
        )
}
