use cstree::syntax::SyntaxNode;
use omena_syntax::SyntaxKind;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct CstTokenRange {
    pub(super) kind: SyntaxKind,
    pub(super) start: usize,
    pub(super) end: usize,
    pub(super) text: String,
}

pub(super) fn cst_token_ranges(root: &SyntaxNode<SyntaxKind>) -> Option<Vec<CstTokenRange>> {
    root.descendants_with_tokens()
        .filter_map(|element| element.into_token())
        .map(|token| {
            Some(CstTokenRange {
                kind: token.kind(),
                start: u32::from(token.text_range().start()) as usize,
                end: u32::from(token.text_range().end()) as usize,
                text: cst_token_text(token)?,
            })
        })
        .collect()
}

pub(super) fn cst_next_non_trivia_token_index(
    tokens: &[CstTokenRange],
    mut index: usize,
) -> Option<usize> {
    while tokens
        .get(index)
        .is_some_and(|token| cst_is_trivia_token(token.kind))
    {
        index += 1;
    }
    (index < tokens.len()).then_some(index)
}

pub(super) fn cst_next_sass_indent_token_index(
    tokens: &[CstTokenRange],
    mut index: usize,
) -> Option<usize> {
    while index < tokens.len() {
        match tokens[index].kind {
            SyntaxKind::SassIndent => return Some(index),
            SyntaxKind::Semicolon | SyntaxKind::SassOptionalSemicolon => return None,
            _ => index += 1,
        }
    }
    None
}

pub(super) fn cst_matching_right_paren_token_index(
    tokens: &[CstTokenRange],
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

pub(super) fn cst_matching_sass_dedent_token_index(
    tokens: &[CstTokenRange],
    sass_indent_index: usize,
) -> Option<usize> {
    let mut depth = 0usize;
    for (index, token) in tokens.iter().enumerate().skip(sass_indent_index) {
        match token.kind {
            SyntaxKind::SassIndent => depth += 1,
            SyntaxKind::SassDedent => {
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

pub(super) fn cst_declaration_end_token_index(
    tokens: &[CstTokenRange],
    mut index: usize,
) -> Option<usize> {
    let mut paren_depth = 0usize;
    while index < tokens.len() {
        match tokens[index].kind {
            SyntaxKind::LeftParen => paren_depth += 1,
            SyntaxKind::RightParen => paren_depth = paren_depth.checked_sub(1)?,
            SyntaxKind::Semicolon | SyntaxKind::SassOptionalSemicolon | SyntaxKind::SassIndent
                if paren_depth == 0 =>
            {
                return Some(index);
            }
            _ => {}
        }
        index += 1;
    }
    None
}

fn cst_token_text(token: &cstree::syntax::SyntaxToken<SyntaxKind>) -> Option<String> {
    if let Some(resolver) = token.resolver() {
        Some(token.resolve_text(&**resolver).to_string())
    } else {
        token.static_text().map(str::to_string)
    }
}

const fn cst_is_trivia_token(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::Whitespace
            | SyntaxKind::LineComment
            | SyntaxKind::BlockComment
            | SyntaxKind::ScssSilentComment
            | SyntaxKind::SassIndentedNewline
    )
}
