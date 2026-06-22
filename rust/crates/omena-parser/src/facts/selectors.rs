//! Parser facts for selector definitions and references.
//!
//! Selector facts expose the local class, id, and custom property names
//! required by diagnostics, rename, references, and transform reachability.

use cstree::text::TextRange;
use omena_syntax::SyntaxKind;
use std::collections::BTreeSet;

use crate::{
    ParseResult, Token, find_block_after_header, is_selector_combinator_kind,
    matching_right_paren_from_range, next_non_trivia_token_after_range,
    next_non_trivia_token_until, previous_non_trivia_token, selector_component_can_end,
    selector_component_can_start, skip_statement, skip_trivia_tokens, style_wrapper_at_rule,
    token_index_by_range,
};

use super::tokens_from_cst;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedSelectorFact {
    pub kind: ParsedSelectorFactKind,
    pub name: String,
    pub range: TextRange,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ParsedSelectorFactKind {
    Class,
    Id,
    Placeholder,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SelectorBranch {
    pub(crate) name: String,
    pub(crate) range: TextRange,
    pub(crate) bare_suffix_base: bool,
}

#[cfg(feature = "internal-oracle")]
pub(crate) fn collect_selector_facts_from_tokens(tokens: &[Token<'_>]) -> Vec<ParsedSelectorFact> {
    selector_facts_from_token_view(tokens)
}

pub(crate) fn collect_selector_facts_from_cst(
    text: &str,
    parsed: &ParseResult,
) -> Vec<ParsedSelectorFact> {
    let tokens = tokens_from_cst(text, parsed);
    selector_facts_from_token_view(&tokens)
}

fn selector_facts_from_token_view(tokens: &[Token<'_>]) -> Vec<ParsedSelectorFact> {
    let mut selectors = Vec::new();
    let mut seen = BTreeSet::new();
    collect_selector_facts_in_range(
        tokens,
        0,
        tokens.len(),
        &[],
        None,
        &mut seen,
        &mut selectors,
    );
    selectors
}

fn collect_selector_facts_in_range(
    tokens: &[Token<'_>],
    start: usize,
    end: usize,
    parent_branches: &[SelectorBranch],
    css_module_scope: Option<&'static str>,
    seen: &mut BTreeSet<(ParsedSelectorFactKind, String, u32, u32)>,
    selectors: &mut Vec<ParsedSelectorFact>,
) {
    let mut index = start;
    while index < end {
        index = skip_trivia_tokens(tokens, index, end);
        if index >= end {
            break;
        }

        if tokens[index].kind == SyntaxKind::AtKeyword {
            let block = find_block_after_header(tokens, index, end);
            if let Some((open, close)) = block {
                if tokens[index].text == "@nest" {
                    if css_module_scope == Some("global") {
                        collect_selector_facts_in_range(
                            tokens,
                            open + 1,
                            close,
                            &[],
                            css_module_scope,
                            seen,
                            selectors,
                        );
                    } else {
                        let branches =
                            resolve_selector_header(tokens, index + 1, open, parent_branches);
                        push_class_selector_facts_from_header(
                            selectors,
                            seen,
                            tokens,
                            index + 1,
                            open,
                        );
                        for branch in &branches {
                            push_selector_fact(
                                selectors,
                                seen,
                                ParsedSelectorFactKind::Class,
                                branch.name.clone(),
                                branch.range,
                            );
                        }
                        collect_selector_facts_in_range(
                            tokens,
                            open + 1,
                            close,
                            &branches,
                            css_module_scope,
                            seen,
                            selectors,
                        );
                    }
                } else if style_wrapper_at_rule(tokens[index].text) {
                    collect_selector_facts_in_range(
                        tokens,
                        open + 1,
                        close,
                        parent_branches,
                        css_module_scope,
                        seen,
                        selectors,
                    );
                }
                index = close + 1;
            } else {
                index = skip_statement(tokens, index, end);
            }
            continue;
        }

        let Some((open, close)) = find_block_after_header(tokens, index, end) else {
            index = skip_statement(tokens, index, end);
            continue;
        };

        let effective_scope = css_module_scope
            .or_else(|| css_module_block_scope_marker_in_header(tokens, index, open));
        if effective_scope == Some("global") {
            collect_selector_facts_in_range(
                tokens,
                open + 1,
                close,
                &[],
                effective_scope,
                seen,
                selectors,
            );
        } else {
            let branches = resolve_selector_header(tokens, index, open, parent_branches);
            push_class_selector_facts_from_header(selectors, seen, tokens, index, open);
            for branch in &branches {
                push_selector_fact(
                    selectors,
                    seen,
                    ParsedSelectorFactKind::Class,
                    branch.name.clone(),
                    branch.range,
                );
            }
            for id in collect_id_selector_facts_from_header(tokens, index, open)
                .into_iter()
                .chain(collect_local_function_id_selector_facts_from_header(
                    tokens, index, open,
                ))
            {
                push_selector_fact(selectors, seen, ParsedSelectorFactKind::Id, id.0, id.1);
            }
            for placeholder in collect_placeholder_selector_facts_from_header(tokens, index, open) {
                push_selector_fact(
                    selectors,
                    seen,
                    ParsedSelectorFactKind::Placeholder,
                    placeholder.0,
                    placeholder.1,
                );
            }

            collect_selector_facts_in_range(
                tokens,
                open + 1,
                close,
                &branches,
                effective_scope,
                seen,
                selectors,
            );
        }
        index = close + 1;
    }
}

fn push_class_selector_facts_from_header(
    selectors: &mut Vec<ParsedSelectorFact>,
    seen: &mut BTreeSet<(ParsedSelectorFactKind, String, u32, u32)>,
    tokens: &[Token<'_>],
    start: usize,
    end: usize,
) {
    for (name, range) in collect_class_selector_names_from_header(tokens, start, end) {
        push_selector_fact(selectors, seen, ParsedSelectorFactKind::Class, name, range);
    }
}

fn push_selector_fact(
    selectors: &mut Vec<ParsedSelectorFact>,
    seen: &mut BTreeSet<(ParsedSelectorFactKind, String, u32, u32)>,
    kind: ParsedSelectorFactKind,
    name: String,
    range: TextRange,
) {
    if seen.insert((
        kind,
        name.clone(),
        u32::from(range.start()),
        u32::from(range.end()),
    )) {
        selectors.push(ParsedSelectorFact { kind, name, range });
    }
}

pub(crate) fn resolve_selector_header(
    tokens: &[Token<'_>],
    start: usize,
    end: usize,
    parent_branches: &[SelectorBranch],
) -> Vec<SelectorBranch> {
    split_selector_groups(tokens, start, end)
        .into_iter()
        .flat_map(|(group_start, group_end)| {
            resolve_selector_group(tokens, group_start, group_end, parent_branches)
        })
        .collect()
}

fn resolve_selector_group(
    tokens: &[Token<'_>],
    start: usize,
    end: usize,
    parent_branches: &[SelectorBranch],
) -> Vec<SelectorBranch> {
    if let Some(mut local_names) = collect_local_function_selector_names(tokens, start, end) {
        local_names.extend(collect_class_selector_names_from_header(tokens, start, end));
        let bare_suffix_base = parent_branches.is_empty() && local_names.len() == 1;
        return local_names
            .into_iter()
            .map(|(name, range)| SelectorBranch {
                name,
                range,
                bare_suffix_base,
            })
            .collect();
    }

    let (tail_start, tail_end) = selector_group_tail_range(tokens, start, end);
    let tail_start = skip_trivia_tokens(tokens, tail_start, tail_end);

    if let Some((suffix, range)) = ampersand_suffix_selector(tokens, tail_start, tail_end) {
        let bases: Vec<&SelectorBranch> = if parent_branches.is_empty() {
            Vec::new()
        } else {
            parent_branches
                .iter()
                .filter(|parent| parent.bare_suffix_base)
                .collect()
        };
        return bases
            .into_iter()
            .map(|parent| SelectorBranch {
                name: format!("{}{}", parent.name, suffix),
                range,
                bare_suffix_base: parent.bare_suffix_base,
            })
            .collect();
    }

    let class_names = collect_class_selector_names_from_header(tokens, tail_start, tail_end);
    if class_names.is_empty() {
        return Vec::new();
    }

    let bare_suffix_base = parent_branches.is_empty()
        && class_names.len() == 1
        && is_bare_class_selector_group(tokens, tail_start, tail_end);
    class_names
        .into_iter()
        .map(|(name, range)| SelectorBranch {
            name,
            range,
            bare_suffix_base,
        })
        .collect()
}

fn is_bare_class_selector_group(tokens: &[Token<'_>], start: usize, end: usize) -> bool {
    let dot_index = skip_trivia_tokens(tokens, start, end);
    if tokens.get(dot_index).map(|token| token.kind) != Some(SyntaxKind::Dot) {
        return false;
    }
    let name_index = skip_trivia_tokens(tokens, dot_index + 1, end);
    if !tokens.get(name_index).is_some_and(|token| {
        matches!(
            token.kind,
            SyntaxKind::Ident | SyntaxKind::CustomPropertyName
        )
    }) {
        return false;
    }
    skip_trivia_tokens(tokens, name_index + 1, end) >= end
}

pub(crate) fn split_selector_groups(
    tokens: &[Token<'_>],
    start: usize,
    end: usize,
) -> Vec<(usize, usize)> {
    let mut groups = Vec::new();
    let mut group_start = start;
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;
    let mut index = start;
    while index < end {
        match tokens[index].kind {
            SyntaxKind::LeftParen => paren_depth += 1,
            SyntaxKind::RightParen => paren_depth = paren_depth.saturating_sub(1),
            SyntaxKind::LeftBracket => bracket_depth += 1,
            SyntaxKind::RightBracket => bracket_depth = bracket_depth.saturating_sub(1),
            SyntaxKind::Comma if paren_depth == 0 && bracket_depth == 0 => {
                groups.push((group_start, index));
                group_start = index + 1;
            }
            _ => {}
        }
        index += 1;
    }
    groups.push((group_start, end));
    groups
}

fn selector_group_tail_range(tokens: &[Token<'_>], start: usize, end: usize) -> (usize, usize) {
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;
    let mut tail_start = start;
    let mut index = start;
    while index < end {
        match tokens[index].kind {
            SyntaxKind::LeftParen => paren_depth += 1,
            SyntaxKind::RightParen => paren_depth = paren_depth.saturating_sub(1),
            SyntaxKind::LeftBracket => bracket_depth += 1,
            SyntaxKind::RightBracket => bracket_depth = bracket_depth.saturating_sub(1),
            kind if paren_depth == 0 && bracket_depth == 0 && is_selector_combinator_kind(kind) => {
                tail_start = index + 1;
            }
            SyntaxKind::Whitespace if paren_depth == 0 && bracket_depth == 0 => {
                let previous = previous_non_trivia_token(tokens, start, index);
                let next = next_non_trivia_token_until(tokens, index + 1, end);
                if previous.is_some_and(|token| selector_component_can_end(token.kind))
                    && next.is_some_and(|token| selector_component_can_start(token.kind))
                {
                    tail_start = index + 1;
                }
            }
            _ => {}
        }
        index += 1;
    }
    (tail_start, end)
}

fn ampersand_suffix_selector(
    tokens: &[Token<'_>],
    start: usize,
    end: usize,
) -> Option<(String, TextRange)> {
    let ampersand_index = skip_trivia_tokens(tokens, start, end);
    if tokens.get(ampersand_index)?.kind != SyntaxKind::Ampersand {
        return None;
    }
    let suffix = next_non_trivia_token_until(tokens, ampersand_index + 1, end)?;
    if matches!(
        suffix.kind,
        SyntaxKind::Ident | SyntaxKind::CustomPropertyName
    ) {
        return Some((suffix.text.to_string(), suffix.range));
    }
    None
}

pub(crate) fn collect_class_selector_names_from_header(
    tokens: &[Token<'_>],
    start: usize,
    end: usize,
) -> Vec<(String, TextRange)> {
    let mut names = Vec::new();
    let mut index = start;
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;
    while index < end {
        match tokens[index].kind {
            SyntaxKind::LeftParen => paren_depth += 1,
            SyntaxKind::RightParen => paren_depth = paren_depth.saturating_sub(1),
            SyntaxKind::LeftBracket => bracket_depth += 1,
            SyntaxKind::RightBracket => bracket_depth = bracket_depth.saturating_sub(1),
            _ => {}
        }
        if paren_depth == 0
            && bracket_depth == 0
            && tokens[index].kind == SyntaxKind::Dot
            && let Some(name) = next_non_trivia_token_until(tokens, index + 1, end)
            && matches!(
                name.kind,
                SyntaxKind::Ident | SyntaxKind::CustomPropertyName
            )
        {
            names.push((name.text.to_string(), name.range));
        }
        index += 1;
    }
    names
}

fn collect_local_function_selector_names(
    tokens: &[Token<'_>],
    start: usize,
    end: usize,
) -> Option<Vec<(String, TextRange)>> {
    let colon_index = skip_trivia_tokens(tokens, start, end);
    if tokens.get(colon_index)?.kind != SyntaxKind::Colon {
        return None;
    }
    let ident = next_non_trivia_token_until(tokens, colon_index + 1, end)?;
    if ident.kind != SyntaxKind::Ident || ident.text != "local" {
        return None;
    }
    let open_index = skip_trivia_tokens(tokens, colon_index + 2, end);
    if tokens.get(open_index)?.kind != SyntaxKind::LeftParen {
        return None;
    }
    Some(collect_class_selector_names_from_header(
        tokens,
        open_index + 1,
        end.saturating_sub(1),
    ))
}

fn collect_local_function_id_selector_facts_from_header(
    tokens: &[Token<'_>],
    start: usize,
    end: usize,
) -> Vec<(String, TextRange)> {
    let mut ids = Vec::new();
    let mut index = start;
    while index < end {
        if tokens[index].kind == SyntaxKind::Colon
            && let Some(scope) = next_non_trivia_token_until(tokens, index + 1, end)
            && scope.kind == SyntaxKind::Ident
            && scope.text == "local"
            && let Some(open) = next_non_trivia_token_after_range(tokens, scope.range, end)
            && open.kind == SyntaxKind::LeftParen
            && let Some(close) = matching_right_paren_from_range(tokens, open.range, end)
        {
            ids.extend(collect_id_selector_facts_from_header(
                tokens,
                token_index_by_range(tokens, open.range).map_or(index + 1, |value| value + 1),
                close,
            ));
            index = close.saturating_add(1);
            continue;
        }
        index += 1;
    }
    ids
}

pub(crate) fn css_module_block_scope_marker_in_header(
    tokens: &[Token<'_>],
    start: usize,
    end: usize,
) -> Option<&'static str> {
    if next_non_trivia_token_until(tokens, start, end)
        .is_some_and(|token| token.kind == SyntaxKind::AtKeyword)
    {
        return None;
    }

    css_module_scope_marker_after_colon(tokens, start, end)
        .filter(|_| !css_module_scope_marker_is_function(tokens, start, end))
}

pub(crate) fn css_module_header_is_global_only(
    tokens: &[Token<'_>],
    start: usize,
    end: usize,
) -> bool {
    if next_non_trivia_token_until(tokens, start, end)
        .is_some_and(|token| token.kind == SyntaxKind::AtKeyword)
    {
        return false;
    }
    css_module_header_contains_scope(tokens, start, end, "global")
        && collect_class_selector_names_from_header(tokens, start, end).is_empty()
        && collect_local_function_selector_names(tokens, start, end)
            .map(|names| names.is_empty())
            .unwrap_or(true)
}

fn css_module_header_contains_scope(
    tokens: &[Token<'_>],
    start: usize,
    end: usize,
    expected_scope: &str,
) -> bool {
    let mut index = start;
    while index < end {
        if tokens[index].kind == SyntaxKind::Colon
            && let Some(scope) = next_non_trivia_token_until(tokens, index + 1, end)
            && scope.kind == SyntaxKind::Ident
            && scope.text == expected_scope
        {
            return true;
        }
        index += 1;
    }
    false
}

fn css_module_scope_marker_after_colon(
    tokens: &[Token<'_>],
    start: usize,
    end: usize,
) -> Option<&'static str> {
    let colon = skip_trivia_tokens(tokens, start, end);
    if tokens.get(colon)?.kind != SyntaxKind::Colon {
        return None;
    }
    let scope = next_non_trivia_token_until(tokens, colon + 1, end)?;
    if scope.kind != SyntaxKind::Ident {
        return None;
    }
    match scope.text {
        "global" => Some("global"),
        "local" => Some("local"),
        _ => None,
    }
}

fn css_module_scope_marker_is_function(tokens: &[Token<'_>], start: usize, end: usize) -> bool {
    let colon = skip_trivia_tokens(tokens, start, end);
    let mut index = colon + 1;
    let Some(scope) = next_non_trivia_token_until(tokens, index, end) else {
        return false;
    };
    while index < end {
        if tokens[index].range == scope.range {
            break;
        }
        index += 1;
    }
    let Some(next) = next_non_trivia_token_until(tokens, index + 1, end) else {
        return false;
    };
    scope.kind == SyntaxKind::Ident && next.kind == SyntaxKind::LeftParen
}

fn collect_id_selector_facts_from_header(
    tokens: &[Token<'_>],
    start: usize,
    end: usize,
) -> Vec<(String, TextRange)> {
    let mut names = Vec::new();
    let mut index = start;
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;
    while index < end {
        match tokens[index].kind {
            SyntaxKind::LeftParen => paren_depth += 1,
            SyntaxKind::RightParen => paren_depth = paren_depth.saturating_sub(1),
            SyntaxKind::LeftBracket => bracket_depth += 1,
            SyntaxKind::RightBracket => bracket_depth = bracket_depth.saturating_sub(1),
            _ => {}
        }
        let token = tokens[index];
        if paren_depth == 0 && bracket_depth == 0 && token.kind == SyntaxKind::Hash {
            names.push((token.text.trim_start_matches('#').to_string(), token.range));
        }
        index += 1;
    }
    names
}

fn collect_placeholder_selector_facts_from_header(
    tokens: &[Token<'_>],
    start: usize,
    end: usize,
) -> Vec<(String, TextRange)> {
    let mut names = Vec::new();
    let mut index = start;
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;
    while index < end {
        match tokens[index].kind {
            SyntaxKind::LeftParen => paren_depth += 1,
            SyntaxKind::RightParen => paren_depth = paren_depth.saturating_sub(1),
            SyntaxKind::LeftBracket => bracket_depth += 1,
            SyntaxKind::RightBracket => bracket_depth = bracket_depth.saturating_sub(1),
            _ => {}
        }
        let token = tokens[index];
        if paren_depth == 0 && bracket_depth == 0 && token.kind == SyntaxKind::ScssPlaceholder {
            names.push((token.text.trim_start_matches('%').to_string(), token.range));
        }
        index += 1;
    }
    names
}
