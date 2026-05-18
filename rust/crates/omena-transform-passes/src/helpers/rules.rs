use omena_parser::LexedToken;
use omena_syntax::SyntaxKind;

use super::tokens::{
    is_comment_token, matching_right_brace_index, token_end, token_start, tokens_between_byte_range,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SimpleRuleSlice {
    pub(crate) selector: String,
    pub(crate) block: String,
    pub(crate) start: usize,
    pub(crate) end: usize,
    pub(crate) block_start: usize,
    pub(crate) block_end: usize,
    pub(crate) context_start: usize,
    pub(crate) context_end: usize,
}

pub(crate) fn collect_top_level_ordinary_rule_slices(
    source: &str,
    tokens: &[LexedToken],
) -> Vec<SimpleRuleSlice> {
    let mut rules = Vec::new();
    let mut depth = 0usize;
    let mut top_level_prelude_start = 0usize;
    let mut index = 0;

    while index < tokens.len() {
        match tokens[index].kind {
            SyntaxKind::LeftBrace => {
                if depth == 0
                    && let Some(close_index) = matching_right_brace_index(tokens, index)
                    && is_ordinary_top_level_rule_prelude(tokens, top_level_prelude_start, index)
                    && !tokens[index + 1..close_index].iter().any(|token| {
                        matches!(token.kind, SyntaxKind::LeftBrace | SyntaxKind::RightBrace)
                            || is_comment_token(token.kind)
                    })
                    && let Some(start) =
                        first_non_trivia_token_start(tokens, top_level_prelude_start, index)
                {
                    let selector = source[start..token_start(&tokens[index])]
                        .trim()
                        .to_string();
                    let block = source
                        [token_end(&tokens[index])..token_start(&tokens[close_index])]
                        .trim()
                        .to_string();
                    if !selector.is_empty() && !block.is_empty() {
                        rules.push(SimpleRuleSlice {
                            selector,
                            block,
                            start,
                            end: token_end(&tokens[close_index]),
                            block_start: token_start(&tokens[index]),
                            block_end: token_start(&tokens[close_index]),
                            context_start: 0,
                            context_end: source.len(),
                        });
                    }
                    index = close_index + 1;
                    top_level_prelude_start = index;
                    continue;
                }
                depth += 1;
            }
            SyntaxKind::RightBrace => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    top_level_prelude_start = index + 1;
                }
            }
            SyntaxKind::Semicolon if depth == 0 => {
                top_level_prelude_start = index + 1;
            }
            _ => {}
        }
        index += 1;
    }

    rules
}

pub(crate) fn collect_declaration_ordinary_rule_slices(
    source: &str,
    tokens: &[LexedToken],
) -> Vec<SimpleRuleSlice> {
    let mut rules = Vec::new();
    let mut depth = 0usize;
    let mut prelude_starts = vec![0usize];
    let mut rule_contexts = vec![(0usize, source.len())];
    let mut index = 0;

    while index < tokens.len() {
        match tokens[index].kind {
            SyntaxKind::LeftBrace => {
                let prelude_start = prelude_starts.get(depth).copied().unwrap_or(0);
                let parent_context = rule_contexts
                    .get(depth)
                    .copied()
                    .unwrap_or((0, source.len()));
                if let Some(close_index) = matching_right_brace_index(tokens, index)
                    && is_ordinary_rule_prelude(tokens, prelude_start, index)
                    && !tokens[index + 1..close_index].iter().any(|token| {
                        matches!(token.kind, SyntaxKind::LeftBrace | SyntaxKind::RightBrace)
                            || is_comment_token(token.kind)
                    })
                    && let Some(start) = first_non_trivia_token_start(tokens, prelude_start, index)
                {
                    let selector = source[start..token_start(&tokens[index])]
                        .trim()
                        .to_string();
                    let block = source
                        [token_end(&tokens[index])..token_start(&tokens[close_index])]
                        .trim()
                        .to_string();
                    if !selector.is_empty() && !block.is_empty() {
                        rules.push(SimpleRuleSlice {
                            selector,
                            block,
                            start,
                            end: token_end(&tokens[close_index]),
                            block_start: token_start(&tokens[index]),
                            block_end: token_start(&tokens[close_index]),
                            context_start: parent_context.0,
                            context_end: parent_context.1,
                        });
                    }
                }
                let child_context = matching_right_brace_index(tokens, index)
                    .map(|close_index| {
                        (token_start(&tokens[index]), token_end(&tokens[close_index]))
                    })
                    .unwrap_or((token_start(&tokens[index]), token_end(&tokens[index])));
                depth += 1;
                set_prelude_start(&mut prelude_starts, depth, index + 1);
                set_rule_context(&mut rule_contexts, depth, child_context);
            }
            SyntaxKind::RightBrace => {
                depth = depth.saturating_sub(1);
                set_prelude_start(&mut prelude_starts, depth, index + 1);
            }
            SyntaxKind::Semicolon => {
                set_prelude_start(&mut prelude_starts, depth, index + 1);
            }
            _ => {}
        }
        index += 1;
    }

    rules
}

pub(crate) fn collect_ordinary_rule_selector_slices(
    source: &str,
    tokens: &[LexedToken],
) -> Vec<SimpleRuleSlice> {
    let mut rules = Vec::new();
    let mut depth = 0usize;
    let mut prelude_starts = vec![0usize];
    let mut index = 0;

    while index < tokens.len() {
        match tokens[index].kind {
            SyntaxKind::LeftBrace => {
                let prelude_start = prelude_starts.get(depth).copied().unwrap_or(0);
                if let Some(close_index) = matching_right_brace_index(tokens, index)
                    && is_ordinary_rule_prelude(tokens, prelude_start, index)
                    && let Some(start) = first_non_trivia_token_start(tokens, prelude_start, index)
                {
                    let selector = source[start..token_start(&tokens[index])]
                        .trim()
                        .to_string();
                    if !selector.is_empty() {
                        rules.push(SimpleRuleSlice {
                            selector,
                            block: source
                                [token_end(&tokens[index])..token_start(&tokens[close_index])]
                                .trim()
                                .to_string(),
                            start,
                            end: token_end(&tokens[close_index]),
                            block_start: token_start(&tokens[index]),
                            block_end: token_start(&tokens[close_index]),
                            context_start: 0,
                            context_end: source.len(),
                        });
                    }
                }
                depth += 1;
                set_prelude_start(&mut prelude_starts, depth, index + 1);
            }
            SyntaxKind::RightBrace => {
                depth = depth.saturating_sub(1);
                set_prelude_start(&mut prelude_starts, depth, index + 1);
            }
            SyntaxKind::Semicolon => {
                set_prelude_start(&mut prelude_starts, depth, index + 1);
            }
            _ => {}
        }
        index += 1;
    }

    rules
}

pub(crate) fn rule_gap_is_whitespace_only(tokens: &[LexedToken], start: usize, end: usize) -> bool {
    tokens_between_byte_range(tokens, start, end)
        .iter()
        .all(|token| token.kind == SyntaxKind::Whitespace)
}

pub(crate) fn set_prelude_start(prelude_starts: &mut Vec<usize>, depth: usize, start: usize) {
    if prelude_starts.len() <= depth {
        prelude_starts.resize(depth + 1, start);
    }
    prelude_starts[depth] = start;
}

pub(crate) fn is_ordinary_rule_prelude(
    tokens: &[LexedToken],
    start: usize,
    end_exclusive: usize,
) -> bool {
    let prelude = &tokens[start..end_exclusive];
    prelude
        .iter()
        .any(|token| !is_comment_token(token.kind) && token.kind != SyntaxKind::Whitespace)
        && prelude
            .iter()
            .all(|token| token.kind != SyntaxKind::AtKeyword && !is_comment_token(token.kind))
}

pub(crate) fn is_ordinary_top_level_rule_prelude(
    tokens: &[LexedToken],
    start: usize,
    end_exclusive: usize,
) -> bool {
    is_ordinary_rule_prelude(tokens, start, end_exclusive)
}

pub(crate) fn first_non_trivia_token_start(
    tokens: &[LexedToken],
    start: usize,
    end_exclusive: usize,
) -> Option<usize> {
    tokens[start..end_exclusive]
        .iter()
        .find(|token| !is_comment_token(token.kind) && token.kind != SyntaxKind::Whitespace)
        .map(token_start)
}

fn set_rule_context(
    rule_contexts: &mut Vec<(usize, usize)>,
    depth: usize,
    context: (usize, usize),
) {
    if rule_contexts.len() <= depth {
        rule_contexts.resize(depth + 1, context);
    }
    rule_contexts[depth] = context;
}
