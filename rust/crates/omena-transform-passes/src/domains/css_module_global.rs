use omena_syntax::SyntaxKind;

use crate::helpers::{
    rules::{first_non_trivia_token_start, set_prelude_start},
    tokens::{matching_right_brace_index, token_end, token_start},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CssModuleScopeBlockKind {
    Local,
    Global,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CssModuleScopeBlock {
    pub(crate) start: usize,
    pub(crate) end: usize,
    pub(crate) body_start: usize,
    pub(crate) body_end: usize,
    pub(crate) kind: CssModuleScopeBlockKind,
}

pub(crate) fn collect_css_module_scope_blocks(
    source: &str,
    tokens: &[omena_parser::LexedToken],
) -> Vec<CssModuleScopeBlock> {
    let mut blocks = Vec::new();
    let mut depth = 0usize;
    let mut prelude_starts = vec![0usize];
    let mut index = 0;

    while index < tokens.len() {
        match tokens[index].kind {
            SyntaxKind::LeftBrace => {
                let prelude_start = prelude_starts.get(depth).copied().unwrap_or(0);
                if let Some(close_index) = matching_right_brace_index(tokens, index)
                    && let Some(start) = first_non_trivia_token_start(tokens, prelude_start, index)
                {
                    let prelude = source[start..token_start(&tokens[index])].trim();
                    if let Some(kind) = css_module_scope_block_kind(prelude) {
                        blocks.push(CssModuleScopeBlock {
                            kind,
                            start,
                            end: token_end(&tokens[close_index]),
                            body_start: token_end(&tokens[index]),
                            body_end: token_start(&tokens[close_index]),
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

    blocks
}

fn css_module_scope_block_kind(prelude: &str) -> Option<CssModuleScopeBlockKind> {
    let prelude = prelude.trim();
    if prelude.eq_ignore_ascii_case(":local") {
        return Some(CssModuleScopeBlockKind::Local);
    }
    if prelude.eq_ignore_ascii_case(":global") {
        return Some(CssModuleScopeBlockKind::Global);
    }
    None
}

pub(crate) fn css_module_scope_kind_for_range(
    start: usize,
    end: usize,
    blocks: &[CssModuleScopeBlock],
) -> Option<CssModuleScopeBlockKind> {
    blocks
        .iter()
        .filter(|block| start >= block.body_start && end <= block.body_end)
        .min_by_key(|block| block.body_end.saturating_sub(block.body_start))
        .map(|block| block.kind)
}
