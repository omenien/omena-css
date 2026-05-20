use omena_cascade::{
    LayerFlattenInputV0, LayerFlattenProofV0, ScopeFlattenInputV0, ScopeFlattenProofV0,
    prove_layer_flatten_candidate, prove_scope_flatten_candidate,
};
use omena_parser::{StyleDialect, lex};
use omena_syntax::SyntaxKind;

use crate::helpers::{
    blocks::at_rule_block_indexes,
    identifiers::css_identifier_text_is_plain,
    rules::{collect_top_level_ordinary_rule_slices, is_ordinary_top_level_rule_prelude},
    source_rewrite::replace_source_ranges,
    tokens::{token_end, token_start},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ScopeFlattenProofCandidateV0 {
    pub(crate) source_span_start: usize,
    pub(crate) source_span_end: usize,
    pub(crate) proof: ScopeFlattenProofV0,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct LayerFlattenProofCandidateV0 {
    pub(crate) source_span_start: usize,
    pub(crate) source_span_end: usize,
    pub(crate) proof: LayerFlattenProofV0,
}

pub(crate) fn flatten_css_scopes_with_lexer(
    source: &str,
    dialect: StyleDialect,
) -> (String, usize) {
    let lexed = lex(source, dialect);
    let tokens = lexed.tokens();
    let top_level_scope_count = count_top_level_at_rules(tokens, "@scope");
    let competing_unscoped_rule_count =
        collect_top_level_ordinary_rule_slices(source, tokens).len();
    let mut replacements = Vec::new();
    let mut depth = 0usize;
    let mut index = 0;

    while index < tokens.len() {
        match tokens[index].kind {
            SyntaxKind::AtKeyword
                if depth == 0 && tokens[index].text.eq_ignore_ascii_case("@scope") =>
            {
                let Some((block_start_index, block_end_index)) =
                    at_rule_block_indexes(tokens, index)
                else {
                    index += 1;
                    continue;
                };
                let prelude = source
                    [token_end(&tokens[index])..token_start(&tokens[block_start_index])]
                    .trim();
                let Some((root_selector, limit_selector)) = parse_scope_flatten_prelude(prelude)
                else {
                    index = block_end_index + 1;
                    continue;
                };
                let scoped_rule_count = count_direct_ordinary_rules_in_block(
                    tokens,
                    block_start_index,
                    block_end_index,
                );
                let proof = prove_scope_flatten_candidate(ScopeFlattenInputV0 {
                    root_selector,
                    limit_selector,
                    scoped_rule_count,
                    peer_scope_count: top_level_scope_count.saturating_sub(1),
                    competing_unscoped_rule_count,
                    inside_layer: false,
                });
                if proof.accepted {
                    let replacement = source[token_end(&tokens[block_start_index])
                        ..token_start(&tokens[block_end_index])]
                        .trim()
                        .to_string();
                    replacements.push((
                        token_start(&tokens[index]),
                        token_end(&tokens[block_end_index]),
                        replacement,
                    ));
                }
                index = block_end_index + 1;
                continue;
            }
            SyntaxKind::LeftBrace => depth += 1,
            SyntaxKind::RightBrace => depth = depth.saturating_sub(1),
            _ => {}
        }
        index += 1;
    }

    replace_source_ranges(source, &replacements)
}

pub(crate) fn collect_scope_flatten_proof_candidates_with_lexer(
    source: &str,
    dialect: StyleDialect,
) -> Vec<ScopeFlattenProofCandidateV0> {
    let lexed = lex(source, dialect);
    let tokens = lexed.tokens();
    let top_level_scope_count = count_top_level_at_rules(tokens, "@scope");
    let competing_unscoped_rule_count =
        collect_top_level_ordinary_rule_slices(source, tokens).len();
    let mut candidates = Vec::new();
    let mut depth = 0usize;
    let mut index = 0;

    while index < tokens.len() {
        match tokens[index].kind {
            SyntaxKind::AtKeyword
                if depth == 0 && tokens[index].text.eq_ignore_ascii_case("@scope") =>
            {
                let Some((block_start_index, block_end_index)) =
                    at_rule_block_indexes(tokens, index)
                else {
                    index += 1;
                    continue;
                };
                let prelude = source
                    [token_end(&tokens[index])..token_start(&tokens[block_start_index])]
                    .trim();
                let Some((root_selector, limit_selector)) = parse_scope_flatten_prelude(prelude)
                else {
                    index = block_end_index + 1;
                    continue;
                };
                let proof = prove_scope_flatten_candidate(ScopeFlattenInputV0 {
                    root_selector,
                    limit_selector,
                    scoped_rule_count: count_direct_ordinary_rules_in_block(
                        tokens,
                        block_start_index,
                        block_end_index,
                    ),
                    peer_scope_count: top_level_scope_count.saturating_sub(1),
                    competing_unscoped_rule_count,
                    inside_layer: false,
                });
                candidates.push(ScopeFlattenProofCandidateV0 {
                    source_span_start: token_start(&tokens[index]),
                    source_span_end: token_end(&tokens[block_end_index]),
                    proof,
                });
                index = block_end_index + 1;
                continue;
            }
            SyntaxKind::LeftBrace => depth += 1,
            SyntaxKind::RightBrace => depth = depth.saturating_sub(1),
            _ => {}
        }
        index += 1;
    }

    candidates
}

pub(crate) fn flatten_css_layers_with_lexer(
    source: &str,
    dialect: StyleDialect,
    closed_bundle: bool,
) -> (String, usize) {
    let lexed = lex(source, dialect);
    let tokens = lexed.tokens();
    let top_level_layer_count = count_top_level_at_rules(tokens, "@layer");
    let unlayered_rule_count = collect_top_level_ordinary_rule_slices(source, tokens).len();
    let mut replacements = Vec::new();
    let mut depth = 0usize;
    let mut index = 0;

    while index < tokens.len() {
        match tokens[index].kind {
            SyntaxKind::AtKeyword
                if depth == 0 && tokens[index].text.eq_ignore_ascii_case("@layer") =>
            {
                let Some((block_start_index, block_end_index)) =
                    at_rule_block_indexes(tokens, index)
                else {
                    index += 1;
                    continue;
                };
                let prelude = source
                    [token_end(&tokens[index])..token_start(&tokens[block_start_index])]
                    .trim();
                let layer_name = parse_single_layer_name(prelude);
                let important_declaration_count = tokens[block_start_index + 1..block_end_index]
                    .iter()
                    .filter(|token| token.kind == SyntaxKind::Important)
                    .count();
                let proof = prove_layer_flatten_candidate(LayerFlattenInputV0 {
                    layer_name,
                    layer_rule_count: count_direct_ordinary_rules_in_block(
                        tokens,
                        block_start_index,
                        block_end_index,
                    ),
                    peer_layer_count: top_level_layer_count.saturating_sub(1),
                    unlayered_rule_count,
                    important_declaration_count,
                    closed_bundle,
                });
                if proof.accepted {
                    let replacement = source[token_end(&tokens[block_start_index])
                        ..token_start(&tokens[block_end_index])]
                        .trim()
                        .to_string();
                    replacements.push((
                        token_start(&tokens[index]),
                        token_end(&tokens[block_end_index]),
                        replacement,
                    ));
                }
                index = block_end_index + 1;
                continue;
            }
            SyntaxKind::LeftBrace => depth += 1,
            SyntaxKind::RightBrace => depth = depth.saturating_sub(1),
            _ => {}
        }
        index += 1;
    }

    replace_source_ranges(source, &replacements)
}

pub(crate) fn collect_layer_flatten_proof_candidates_with_lexer(
    source: &str,
    dialect: StyleDialect,
    closed_bundle: bool,
) -> Vec<LayerFlattenProofCandidateV0> {
    let lexed = lex(source, dialect);
    let tokens = lexed.tokens();
    let top_level_layer_count = count_top_level_at_rules(tokens, "@layer");
    let unlayered_rule_count = collect_top_level_ordinary_rule_slices(source, tokens).len();
    let mut candidates = Vec::new();
    let mut depth = 0usize;
    let mut index = 0;

    while index < tokens.len() {
        match tokens[index].kind {
            SyntaxKind::AtKeyword
                if depth == 0 && tokens[index].text.eq_ignore_ascii_case("@layer") =>
            {
                let Some((block_start_index, block_end_index)) =
                    at_rule_block_indexes(tokens, index)
                else {
                    index += 1;
                    continue;
                };
                let prelude = source
                    [token_end(&tokens[index])..token_start(&tokens[block_start_index])]
                    .trim();
                let proof = prove_layer_flatten_candidate(LayerFlattenInputV0 {
                    layer_name: parse_single_layer_name(prelude),
                    layer_rule_count: count_direct_ordinary_rules_in_block(
                        tokens,
                        block_start_index,
                        block_end_index,
                    ),
                    peer_layer_count: top_level_layer_count.saturating_sub(1),
                    unlayered_rule_count,
                    important_declaration_count: tokens[block_start_index + 1..block_end_index]
                        .iter()
                        .filter(|token| token.kind == SyntaxKind::Important)
                        .count(),
                    closed_bundle,
                });
                candidates.push(LayerFlattenProofCandidateV0 {
                    source_span_start: token_start(&tokens[index]),
                    source_span_end: token_end(&tokens[block_end_index]),
                    proof,
                });
                index = block_end_index + 1;
                continue;
            }
            SyntaxKind::LeftBrace => depth += 1,
            SyntaxKind::RightBrace => depth = depth.saturating_sub(1),
            _ => {}
        }
        index += 1;
    }

    candidates
}

fn count_top_level_at_rules(tokens: &[omena_parser::LexedToken], at_rule: &str) -> usize {
    let mut count = 0;
    let mut depth = 0usize;
    for token in tokens {
        match token.kind {
            SyntaxKind::AtKeyword if depth == 0 && token.text.eq_ignore_ascii_case(at_rule) => {
                count += 1;
            }
            SyntaxKind::LeftBrace => depth += 1,
            SyntaxKind::RightBrace => depth = depth.saturating_sub(1),
            _ => {}
        }
    }
    count
}

fn count_direct_ordinary_rules_in_block(
    tokens: &[omena_parser::LexedToken],
    block_start_index: usize,
    block_end_index: usize,
) -> usize {
    let mut count = 0;
    let mut depth = 0usize;
    let mut index = block_start_index + 1;
    while index < block_end_index {
        match tokens[index].kind {
            SyntaxKind::LeftBrace => {
                if depth == 0
                    && is_ordinary_top_level_rule_prelude(tokens, block_start_index + 1, index)
                {
                    count += 1;
                }
                depth += 1;
            }
            SyntaxKind::RightBrace => depth = depth.saturating_sub(1),
            _ => {}
        }
        index += 1;
    }
    count
}

fn parse_scope_flatten_prelude(prelude: &str) -> Option<(String, Option<String>)> {
    let prelude = prelude.trim();
    let (root, limit) = match prelude.split_once(" to ") {
        Some((root, limit)) => (root, Some(limit)),
        None => (prelude, None),
    };
    let root = strip_wrapping_parentheses(root.trim())?.trim().to_string();
    let limit = match limit {
        Some(limit) => Some(strip_wrapping_parentheses(limit.trim())?.trim().to_string()),
        None => None,
    };
    Some((root, limit))
}

fn strip_wrapping_parentheses(text: &str) -> Option<&str> {
    let text = text.trim();
    text.strip_prefix('(')
        .and_then(|value| value.strip_suffix(')'))
        .or(Some(text))
}

fn parse_single_layer_name(prelude: &str) -> Option<String> {
    let prelude = prelude.trim();
    if prelude.is_empty() || prelude.contains(',') || !css_identifier_text_is_plain(prelude) {
        return None;
    }
    Some(prelude.to_string())
}
