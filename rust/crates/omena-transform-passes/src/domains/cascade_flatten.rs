use omena_cascade::{
    LayerFlattenInputV0, LayerFlattenProofV0, ScopeFlattenInputV0, ScopeFlattenProofV0,
    SelectorMatchVerdict, prove_layer_flatten_candidate, prove_scope_flatten_candidate,
    selector_co_match_verdict,
};
use omena_cascade_proof::{LayerInversionDeclarationV0, layer_inversion_declaration_v0};
use omena_parser::StyleDialect;
use omena_syntax::SyntaxKind;
use omena_transform_cst::{IrNodeKindV0, IrNodeV0, TransformIrV0, lower_transform_ir_from_source};

use crate::runtime::lex_cache::lex_cached as lex;

use crate::helpers::{
    blocks::{at_rule_block_indexes, at_rule_prelude_end_index, rule_block_token_indexes},
    declarations::collect_simple_declarations_in_block,
    identifiers::css_identifier_text_is_plain,
    ir_transaction::{
        TransformIrReplacementKindV0, TransformIrSourceReplacementErrorV0,
        TransformIrSourceReplacementV0, replace_ir_nodes_in_ir,
    },
    rules::{
        collect_declaration_ordinary_rule_slices, collect_top_level_ordinary_rule_slices,
        is_ordinary_top_level_rule_prelude,
    },
    source_rewrite::replace_source_ranges,
    tokens::{token_end, token_start},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ScopeFlattenProofCandidateV0 {
    pub(crate) source_span_start: usize,
    pub(crate) source_span_end: usize,
    pub(crate) input: ScopeFlattenInputV0,
    pub(crate) proof: ScopeFlattenProofV0,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct LayerFlattenProofCandidateV0 {
    pub(crate) source_span_start: usize,
    pub(crate) source_span_end: usize,
    pub(crate) input: LayerFlattenInputV0,
    pub(crate) proof: LayerFlattenProofV0,
}

pub(crate) fn flatten_css_scopes_with_lexer(
    source: &str,
    dialect: StyleDialect,
) -> (String, usize) {
    let replacements = collect_scope_flatten_replacements(source, dialect);
    replace_source_ranges(
        source,
        &replacements
            .iter()
            .map(|replacement| {
                (
                    replacement.source_span_start,
                    replacement.source_span_end,
                    replacement.replacement.clone(),
                )
            })
            .collect::<Vec<_>>(),
    )
}

pub(crate) fn flatten_css_scopes_with_ir_transaction(
    source: &str,
    dialect: StyleDialect,
) -> Result<(String, usize), TransformIrSourceReplacementErrorV0> {
    let mut ir =
        lower_transform_ir_from_source(source, dialect, "omena-transform-passes.scope-flatten");
    flatten_css_scopes_with_ir_transaction_on_ir(&mut ir, dialect)
}

pub(crate) fn flatten_css_scopes_with_ir_transaction_on_ir(
    ir: &mut TransformIrV0,
    _dialect: StyleDialect,
) -> Result<(String, usize), TransformIrSourceReplacementErrorV0> {
    let replacements = collect_scope_flatten_replacements_from_ir(ir);
    replace_ir_nodes_in_ir(ir, "scope-flatten", replacements.as_slice())
}

fn collect_scope_flatten_replacements(
    source: &str,
    dialect: StyleDialect,
) -> Vec<TransformIrSourceReplacementV0> {
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
                    replacements.push(TransformIrSourceReplacementV0 {
                        source_span_start: token_start(&tokens[index]),
                        source_span_end: token_end(&tokens[block_end_index]),
                        replacement,
                        kind: TransformIrReplacementKindV0::AtRule,
                    });
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

    replacements
}

fn collect_scope_flatten_replacements_from_ir(
    ir: &TransformIrV0,
) -> Vec<TransformIrSourceReplacementV0> {
    let top_level_scope_count = count_top_level_at_rules_from_ir(ir, "@scope");
    let competing_unscoped_rule_count = count_top_level_ordinary_rules_from_ir(ir);
    collect_top_level_at_rule_views_from_ir(ir, "@scope")
        .into_iter()
        .filter_map(|rule| {
            let (root_selector, limit_selector) = parse_scope_flatten_prelude(rule.prelude)?;
            let proof = prove_scope_flatten_candidate(ScopeFlattenInputV0 {
                root_selector,
                limit_selector,
                scoped_rule_count: count_direct_ordinary_rules_from_ir(ir, rule.node),
                peer_scope_count: top_level_scope_count.saturating_sub(1),
                competing_unscoped_rule_count,
                inside_layer: false,
            });
            proof.accepted.then(|| TransformIrSourceReplacementV0 {
                source_span_start: rule.source_span_start,
                source_span_end: rule.source_span_end,
                replacement: rule.body.trim().to_string(),
                kind: TransformIrReplacementKindV0::AtRule,
            })
        })
        .collect()
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
                let input = ScopeFlattenInputV0 {
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
                };
                let proof = prove_scope_flatten_candidate(input.clone());
                candidates.push(ScopeFlattenProofCandidateV0 {
                    source_span_start: token_start(&tokens[index]),
                    source_span_end: token_end(&tokens[block_end_index]),
                    input,
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
    let replacements = collect_layer_flatten_replacements(source, dialect, closed_bundle);
    replace_source_ranges(
        source,
        &replacements
            .iter()
            .map(|replacement| {
                (
                    replacement.source_span_start,
                    replacement.source_span_end,
                    replacement.replacement.clone(),
                )
            })
            .collect::<Vec<_>>(),
    )
}

pub(crate) fn flatten_css_layers_with_ir_transaction(
    source: &str,
    dialect: StyleDialect,
    closed_bundle: bool,
) -> Result<(String, usize), TransformIrSourceReplacementErrorV0> {
    let mut ir =
        lower_transform_ir_from_source(source, dialect, "omena-transform-passes.layer-flatten");
    flatten_css_layers_with_ir_transaction_on_ir(&mut ir, dialect, closed_bundle)
}

pub(crate) fn flatten_css_layers_with_ir_transaction_on_ir(
    ir: &mut TransformIrV0,
    _dialect: StyleDialect,
    closed_bundle: bool,
) -> Result<(String, usize), TransformIrSourceReplacementErrorV0> {
    let replacements = collect_layer_flatten_replacements_from_ir(ir, closed_bundle);
    replace_ir_nodes_in_ir(ir, "layer-flatten", replacements.as_slice())
}

fn collect_layer_flatten_replacements(
    source: &str,
    dialect: StyleDialect,
    closed_bundle: bool,
) -> Vec<TransformIrSourceReplacementV0> {
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
                    replacements.push(TransformIrSourceReplacementV0 {
                        source_span_start: token_start(&tokens[index]),
                        source_span_end: token_end(&tokens[block_end_index]),
                        replacement,
                        kind: TransformIrReplacementKindV0::AtRule,
                    });
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

    replacements
}

fn collect_layer_flatten_replacements_from_ir(
    ir: &TransformIrV0,
    closed_bundle: bool,
) -> Vec<TransformIrSourceReplacementV0> {
    let top_level_layer_count = count_top_level_at_rules_from_ir(ir, "@layer");
    let unlayered_rule_count = count_top_level_ordinary_rules_from_ir(ir);
    collect_top_level_at_rule_views_from_ir(ir, "@layer")
        .into_iter()
        .filter_map(|rule| {
            let proof = prove_layer_flatten_candidate(LayerFlattenInputV0 {
                layer_name: parse_single_layer_name(rule.prelude),
                layer_rule_count: count_direct_ordinary_rules_from_ir(ir, rule.node),
                peer_layer_count: top_level_layer_count.saturating_sub(1),
                unlayered_rule_count,
                important_declaration_count: count_important_declarations_in_source(rule.body),
                closed_bundle,
            });
            proof.accepted.then(|| TransformIrSourceReplacementV0 {
                source_span_start: rule.source_span_start,
                source_span_end: rule.source_span_end,
                replacement: rule.body.trim().to_string(),
                kind: TransformIrReplacementKindV0::AtRule,
            })
        })
        .collect()
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
                let input = LayerFlattenInputV0 {
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
                };
                let proof = prove_layer_flatten_candidate(input.clone());
                candidates.push(LayerFlattenProofCandidateV0 {
                    source_span_start: token_start(&tokens[index]),
                    source_span_end: token_end(&tokens[block_end_index]),
                    input,
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

/// A closed-style-world bundle of competing layered declarations, carrying the
/// real per-declaration `(layer_rank, source_order)` the SMT inversion search
/// reasons over.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct LayerInversionBundleCandidateV0 {
    pub(crate) source_span_start: usize,
    pub(crate) source_span_end: usize,
    /// Per-declaration cascade coordinates collected from the real token stream:
    /// no field is a literal inversion flag.
    pub(crate) declarations: Vec<LayerInversionDeclarationV0>,
}

/// A declaration competing for the cascade inside a layered bundle, before it is
/// reduced to the SMT-facing `(layer_rank, source_order)` coordinates.
struct CompetingLayerDeclarationV0 {
    selector: String,
    property: String,
    layer_rank: usize,
    source_order: usize,
    span_start: usize,
    span_end: usize,
}

/// Collect the cross-layer declaration coordinate pairs a closed-bundle layer
/// flatten must preserve.
///
/// Layer precedence (`layer_rank`) is taken from the real cascade rule: an
/// `@layer a, b;` pre-declaration statement fixes the order of its names, and any
/// layer first seen as a block is appended in appearance order. Higher rank wins
/// *before* flattening. Each declaration's `source_order` is its byte start in
/// the source; the later position wins *after* the layer boundary is erased. Only
/// declarations that genuinely compete — same property and selector co-match in
/// more than one layer — are returned, because a flatten is unsafe exactly when
/// the layered and flattened winners of a competition disagree. That ordering
/// inversion is the search performed by the cascade proof boundary
/// runs. The discriminating `(layer_rank, source_order)` pairs are read from the
/// tokens, never fabricated from a literal inversion flag.
pub(crate) fn collect_layer_inversion_declarations_with_lexer(
    source: &str,
    dialect: StyleDialect,
) -> Vec<LayerInversionBundleCandidateV0> {
    let lexed = lex(source, dialect);
    let tokens = lexed.tokens();

    let mut layer_ranks: Vec<String> = Vec::new();
    let mut layer_blocks: Vec<(usize, usize, usize)> = Vec::new();
    let mut depth = 0usize;
    let mut index = 0;

    while index < tokens.len() {
        match tokens[index].kind {
            SyntaxKind::AtKeyword
                if depth == 0 && tokens[index].text.eq_ignore_ascii_case("@layer") =>
            {
                match at_rule_block_indexes(tokens, index) {
                    Some((block_start_index, block_end_index)) => {
                        // `@layer name { ... }` block: register the layer (if not
                        // already pre-declared) and record its byte range.
                        let prelude = source
                            [token_end(&tokens[index])..token_start(&tokens[block_start_index])]
                            .trim();
                        let Some(layer_name) = parse_single_layer_name(prelude) else {
                            index = block_end_index + 1;
                            continue;
                        };
                        let layer_rank = layer_rank_for(&mut layer_ranks, &layer_name);
                        layer_blocks.push((
                            layer_rank,
                            token_start(&tokens[index]),
                            token_end(&tokens[block_end_index]),
                        ));
                        index = block_end_index + 1;
                        continue;
                    }
                    None => {
                        // `@layer a, b;` pre-declaration statement: fixes the
                        // precedence order of the named layers.
                        if let Some(prelude_end) = at_rule_prelude_end_index(tokens, index + 1) {
                            let prelude = &source
                                [token_end(&tokens[index])..token_start(&tokens[prelude_end])];
                            for name in prelude.split(',') {
                                let name = name.trim();
                                if !name.is_empty() && css_identifier_text_is_plain(name) {
                                    layer_rank_for(&mut layer_ranks, name);
                                }
                            }
                            index = prelude_end + 1;
                            continue;
                        }
                    }
                }
            }
            SyntaxKind::LeftBrace => depth += 1,
            SyntaxKind::RightBrace => depth = depth.saturating_sub(1),
            _ => {}
        }
        index += 1;
    }

    // The cross-layer obligation only exists for a genuine multi-layer bundle:
    // a single layer can never invert against itself.
    if layer_blocks.len() < 2 {
        return Vec::new();
    }

    // Harvest every `selector { decls }` rule and attribute each declaration to
    // the layer block whose byte range contains it.
    let mut competing = Vec::new();
    for rule in collect_declaration_ordinary_rule_slices(source, tokens) {
        let Some((layer_rank, _, _)) =
            layer_blocks
                .iter()
                .copied()
                .find(|(_, block_start, block_end)| {
                    rule.start >= *block_start && rule.end <= *block_end
                })
        else {
            continue;
        };
        let (rule_block_start, rule_block_end) =
            match rule_block_token_indexes(tokens, rule.block_start, rule.block_end) {
                Some(indexes) => indexes,
                None => continue,
            };
        for declaration in
            collect_simple_declarations_in_block(tokens, rule_block_start, rule_block_end)
        {
            competing.push(CompetingLayerDeclarationV0 {
                selector: rule.selector.clone(),
                property: declaration.property,
                layer_rank,
                source_order: declaration.start,
                span_start: declaration.start,
                span_end: declaration.end,
            });
        }
    }

    let mut bundles = Vec::new();
    for (left_index, left) in competing.iter().enumerate() {
        for right in competing.iter().skip(left_index + 1) {
            if left.property != right.property
                || left.layer_rank == right.layer_rank
                || selector_co_match_verdict(left.selector.as_str(), right.selector.as_str())
                    == SelectorMatchVerdict::No
            {
                continue;
            }

            let source_span_start = left.span_start.min(right.span_start);
            let source_span_end = left.span_end.max(right.span_end);
            let declarations = [left, right]
                .into_iter()
                .map(|declaration| {
                    layer_inversion_declaration_v0(
                        format!(
                            "{}|{}@{}",
                            declaration.selector, declaration.property, declaration.source_order
                        ),
                        declaration.layer_rank as i64,
                        declaration.source_order as i64,
                    )
                })
                .collect();

            bundles.push(LayerInversionBundleCandidateV0 {
                source_span_start,
                source_span_end,
                declarations,
            });
        }
    }

    bundles
}

/// Resolve `layer_name` to its cascade rank, registering it in appearance order
/// when first seen. Returns the index in the precedence order (higher wins).
fn layer_rank_for(layer_ranks: &mut Vec<String>, layer_name: &str) -> usize {
    if let Some(rank) = layer_ranks.iter().position(|name| name == layer_name) {
        rank
    } else {
        layer_ranks.push(layer_name.to_string());
        layer_ranks.len() - 1
    }
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

#[derive(Debug, Clone, Copy)]
struct FlattenAtRuleIrViewV0<'a> {
    node: &'a IrNodeV0,
    source_span_start: usize,
    source_span_end: usize,
    prelude: &'a str,
    body: &'a str,
}

fn collect_top_level_at_rule_views_from_ir<'a>(
    ir: &'a TransformIrV0,
    keyword: &str,
) -> Vec<FlattenAtRuleIrViewV0<'a>> {
    let mut rules = ir
        .nodes
        .iter()
        .filter(|node| {
            !node.deleted
                && node.parent.is_none()
                && node.kind == IrNodeKindV0::AtRule
                && at_rule_keyword_matches_ir(ir, node, keyword)
        })
        .filter_map(|node| flatten_at_rule_ir_view(ir, node, keyword))
        .collect::<Vec<_>>();
    rules.sort_by_key(|rule| (rule.source_span_start, rule.node.global_order));
    rules
}

fn count_top_level_at_rules_from_ir(ir: &TransformIrV0, keyword: &str) -> usize {
    ir.nodes
        .iter()
        .filter(|node| {
            !node.deleted
                && node.parent.is_none()
                && node.kind == IrNodeKindV0::AtRule
                && at_rule_keyword_matches_ir(ir, node, keyword)
        })
        .count()
}

fn count_top_level_ordinary_rules_from_ir(ir: &TransformIrV0) -> usize {
    ir.nodes
        .iter()
        .filter(|node| {
            !node.deleted && node.parent.is_none() && node.kind == IrNodeKindV0::StyleRule
        })
        .count()
}

fn count_direct_ordinary_rules_from_ir(ir: &TransformIrV0, node: &IrNodeV0) -> usize {
    node.children
        .iter()
        .filter_map(|child_id| ir.nodes.get(child_id.index()))
        .filter(|child| !child.deleted && child.kind == IrNodeKindV0::StyleRule)
        .count()
}

fn flatten_at_rule_ir_view<'a>(
    ir: &'a TransformIrV0,
    node: &'a IrNodeV0,
    keyword: &str,
) -> Option<FlattenAtRuleIrViewV0<'a>> {
    let source = ir.source_text();
    let node_source = source.get(node.source_span_start..node.source_span_end)?;
    let leading_offset = node_source
        .len()
        .saturating_sub(node_source.trim_start().len());
    let source_span_start = node.source_span_start.checked_add(leading_offset)?;
    let keyword_end = source_span_start.checked_add(keyword.len())?;
    if !source
        .get(source_span_start..keyword_end)?
        .eq_ignore_ascii_case(keyword)
    {
        return None;
    }
    let relative_block_start = node_source.get(leading_offset..)?.find('{')?;
    let relative_block_end = node_source.rfind('}')?;
    if relative_block_start >= relative_block_end {
        return None;
    }
    let block_start = node
        .source_span_start
        .checked_add(leading_offset + relative_block_start)?;
    let block_end = node.source_span_start.checked_add(relative_block_end)?;
    Some(FlattenAtRuleIrViewV0 {
        node,
        source_span_start: node.source_span_start,
        source_span_end: node.source_span_end,
        prelude: source.get(keyword_end..block_start)?.trim(),
        body: source.get(block_start + 1..block_end)?.trim(),
    })
}

fn at_rule_keyword_matches_ir(ir: &TransformIrV0, node: &IrNodeV0, keyword: &str) -> bool {
    let Some(source) = ir
        .source_text()
        .get(node.source_span_start..node.source_span_end)
    else {
        return false;
    };
    let source = source.trim_start();
    let Some(candidate) = source.get(..keyword.len()) else {
        return false;
    };
    if !candidate.eq_ignore_ascii_case(keyword) {
        return false;
    }
    source
        .as_bytes()
        .get(keyword.len())
        .is_none_or(|byte| !byte.is_ascii_alphanumeric() && *byte != b'-' && *byte != b'_')
}

fn count_important_declarations_in_source(source: &str) -> usize {
    let bytes = source.as_bytes();
    let mut count = 0usize;
    let mut index = 0usize;
    let mut quote = None;
    let mut escaped = false;
    let mut in_comment = false;

    while index < bytes.len() {
        let byte = bytes[index];
        if in_comment {
            if byte == b'*' && bytes.get(index + 1) == Some(&b'/') {
                in_comment = false;
                index += 2;
            } else {
                index += 1;
            }
            continue;
        }
        if let Some(quote_byte) = quote {
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == quote_byte {
                quote = None;
            }
            index += 1;
            continue;
        }
        if byte == b'/' && bytes.get(index + 1) == Some(&b'*') {
            in_comment = true;
            index += 2;
            continue;
        }
        if byte == b'\'' || byte == b'"' {
            quote = Some(byte);
            index += 1;
            continue;
        }
        if byte == b'!' && important_suffix_starts(source, index + 1) {
            count = count.saturating_add(1);
        }
        index += 1;
    }

    count
}

fn important_suffix_starts(source: &str, start: usize) -> bool {
    let Some(rest) = source.get(start..) else {
        return false;
    };
    let trimmed = rest.trim_start();
    let whitespace_len = rest.len().saturating_sub(trimmed.len());
    let important_start = start.saturating_add(whitespace_len);
    let important_end = important_start.saturating_add("important".len());
    source
        .get(important_start..important_end)
        .is_some_and(|candidate| candidate.eq_ignore_ascii_case("important"))
        && source
            .as_bytes()
            .get(important_end)
            .is_none_or(|byte| !byte.is_ascii_alphanumeric() && *byte != b'-' && *byte != b'_')
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
