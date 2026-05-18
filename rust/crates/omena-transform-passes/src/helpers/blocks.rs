use omena_parser::LexedToken;
use omena_syntax::SyntaxKind;

use super::tokens::{is_comment_token, token_start};

pub(crate) fn at_rule_block_start(tokens: &[LexedToken], start_index: usize) -> Option<usize> {
    let mut index = start_index;
    while index < tokens.len() {
        match tokens[index].kind {
            SyntaxKind::LeftBrace => return Some(index),
            SyntaxKind::RightBrace | SyntaxKind::Semicolon => return None,
            _ => index += 1,
        }
    }
    None
}

pub(crate) fn at_rule_prelude_end_index(tokens: &[LexedToken], start: usize) -> Option<usize> {
    (start..tokens.len()).find(|index| {
        matches!(
            tokens[*index].kind,
            SyntaxKind::LeftBrace | SyntaxKind::RightBrace | SyntaxKind::Semicolon
        )
    })
}

pub(crate) fn previous_significant_token_kind(
    tokens: &[LexedToken],
    index: usize,
    lower_bound: usize,
) -> Option<SyntaxKind> {
    tokens[lower_bound..index]
        .iter()
        .rev()
        .find(|token| token.kind != SyntaxKind::Whitespace && !is_comment_token(token.kind))
        .map(|token| token.kind)
}

pub(crate) fn rule_block_token_indexes(
    tokens: &[LexedToken],
    block_start: usize,
    block_end: usize,
) -> Option<(usize, usize)> {
    let start_index = tokens
        .iter()
        .position(|token| token_start(token) == block_start)?;
    let end_index = tokens
        .iter()
        .position(|token| token_start(token) == block_end)?;
    Some((start_index, end_index))
}
