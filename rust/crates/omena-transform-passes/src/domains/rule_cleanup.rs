use omena_parser::{LexedToken, StyleDialect};
use omena_syntax::SyntaxKind;
use omena_transform_cst::TransformIrV0;

use crate::runtime::lex_cache::lex_cached as lex;

use crate::domains::keyframes::is_keyframes_at_keyword;
use crate::helpers::{
    declarations::collect_simple_declarations_in_block,
    ir_transaction::{
        TransformIrReplacementKindV0, TransformIrSourceReplacementErrorV0,
        TransformIrSourceReplacementV0, apply_ir_source_replacements_to_ir,
    },
    rules::{
        SimpleRuleSlice, collect_declaration_ordinary_rule_slices, first_non_trivia_token_start,
        is_ordinary_rule_prelude, set_prelude_start,
    },
    source_rewrite::remove_source_ranges,
    tokens::{is_comment_token, matching_right_brace_index, token_end, token_start},
};

pub(crate) fn dedupe_exact_css_rules_with_lexer(
    source: &str,
    dialect: StyleDialect,
) -> (String, usize) {
    let (output, declaration_count) =
        remove_overridden_same_property_declarations_with_lexer(source, dialect);
    let source = output.as_str();
    let lexed = lex(source, dialect);
    let tokens = lexed.tokens();
    let rules = collect_declaration_ordinary_rule_slices(source, tokens);
    let replacements = collect_duplicate_ordinary_rule_replacements_from_rules(&rules);

    if replacements.is_empty() {
        return (source.to_string(), declaration_count);
    }

    let (output, rule_count) = remove_source_ranges(
        source,
        &replacements
            .iter()
            .map(|replacement| (replacement.source_span_start, replacement.source_span_end))
            .collect::<Vec<_>>(),
    );
    (output, declaration_count + rule_count)
}

pub(crate) fn dedupe_exact_css_rules_with_ir_transaction(
    source: &str,
    dialect: StyleDialect,
) -> Result<(String, usize), TransformIrSourceReplacementErrorV0> {
    let mut ir = omena_transform_cst::lower_transform_ir_from_source(
        source,
        dialect,
        "omena-transform-passes.rule-deduplication",
    );
    dedupe_exact_css_rules_with_ir_transaction_on_ir(&mut ir, dialect)
}

pub(crate) fn dedupe_exact_css_rules_with_ir_transaction_on_ir(
    ir: &mut TransformIrV0,
    dialect: StyleDialect,
) -> Result<(String, usize), TransformIrSourceReplacementErrorV0> {
    let declaration_replacements =
        collect_overridden_same_property_declaration_replacements(ir.source_text(), dialect);
    let (output, declaration_count) = apply_ir_source_replacements_to_ir(
        ir,
        dialect,
        "rule-deduplication",
        declaration_replacements.as_slice(),
    )?;
    let rule_replacements = collect_duplicate_ordinary_rule_replacements(&output, dialect);
    let (output, rule_count) = apply_ir_source_replacements_to_ir(
        ir,
        dialect,
        "rule-deduplication",
        rule_replacements.as_slice(),
    )?;
    Ok((output, declaration_count + rule_count))
}

fn remove_overridden_same_property_declarations_with_lexer(
    source: &str,
    dialect: StyleDialect,
) -> (String, usize) {
    let replacements = collect_overridden_same_property_declaration_replacements(source, dialect);
    remove_source_ranges(
        source,
        &replacements
            .iter()
            .map(|replacement| (replacement.source_span_start, replacement.source_span_end))
            .collect::<Vec<_>>(),
    )
}

fn collect_overridden_same_property_declaration_replacements(
    source: &str,
    dialect: StyleDialect,
) -> Vec<TransformIrSourceReplacementV0> {
    let lexed = lex(source, dialect);
    let tokens = lexed.tokens();
    let mut replacements = Vec::new();

    for rule in collect_declaration_ordinary_rule_slices(source, tokens) {
        let selector = rule.selector.trim();
        if selector.eq_ignore_ascii_case(":export") || selector.starts_with(":import") {
            continue;
        }
        let Some(block_start_index) = tokens
            .iter()
            .position(|token| token_start(token) == rule.block_start)
        else {
            continue;
        };
        let Some(block_end_index) = matching_right_brace_index(tokens, block_start_index) else {
            continue;
        };
        let declarations =
            collect_simple_declarations_in_block(tokens, block_start_index, block_end_index);
        for (index, declaration) in declarations.iter().enumerate() {
            if declaration.property == "composes"
                || !same_property_override_can_dedupe(&declaration.property)
                || declaration_value_has_compat_fallback(&declaration.value)
            {
                continue;
            }
            let has_later_same_cascade_bucket = declarations[index + 1..].iter().any(|candidate| {
                candidate.property == declaration.property
                    && candidate.important == declaration.important
                    && candidate.property != "composes"
                    && same_property_override_can_dedupe(&candidate.property)
                    && !declaration_value_has_compat_fallback(&candidate.value)
            });
            if has_later_same_cascade_bucket {
                replacements.push(TransformIrSourceReplacementV0 {
                    source_span_start: declaration.start,
                    source_span_end: declaration.end,
                    replacement: String::new(),
                    kind: TransformIrReplacementKindV0::Declaration,
                });
            }
        }
    }

    replacements
}

fn declaration_value_has_compat_fallback(value: &str) -> bool {
    value.contains("-webkit-")
        || value.contains("-moz-")
        || value.contains("-ms-")
        || value.contains("-o-")
}

fn same_property_override_can_dedupe(property: &str) -> bool {
    // Keep opacity-family duplicates: lightningcss preserves them as compatibility fallbacks.
    !matches!(
        property,
        "opacity" | "fill-opacity" | "stroke-opacity" | "flood-opacity" | "stop-opacity"
    )
}

fn collect_duplicate_ordinary_rule_replacements(
    source: &str,
    dialect: StyleDialect,
) -> Vec<TransformIrSourceReplacementV0> {
    let lexed = lex(source, dialect);
    let tokens = lexed.tokens();
    let rules = collect_declaration_ordinary_rule_slices(source, tokens);
    collect_duplicate_ordinary_rule_replacements_from_rules(&rules)
}

fn collect_duplicate_ordinary_rule_replacements_from_rules(
    rules: &[SimpleRuleSlice],
) -> Vec<TransformIrSourceReplacementV0> {
    let mut replacements = Vec::new();

    for (index, rule) in rules.iter().enumerate() {
        let has_later_duplicate = rules[index + 1..].iter().any(|candidate| {
            rule.selector == candidate.selector
                && rule.block == candidate.block
                && rule.context_start == candidate.context_start
                && rule.context_end == candidate.context_end
        });
        if has_later_duplicate {
            replacements.push(TransformIrSourceReplacementV0 {
                source_span_start: rule.start,
                source_span_end: rule.end,
                replacement: String::new(),
                kind: TransformIrReplacementKindV0::StyleRule,
            });
        }
    }

    replacements
}

pub(crate) fn remove_empty_css_rules_with_lexer(
    source: &str,
    dialect: StyleDialect,
) -> (String, usize) {
    let mut output = source.to_string();
    let mut mutation_count = 0;

    loop {
        let lexed = lex(&output, dialect);
        let tokens = lexed.tokens();
        let replacements = collect_empty_rule_replacements(tokens);
        let (next_output, removed_count) = remove_source_ranges(
            &output,
            &replacements
                .iter()
                .map(|replacement| (replacement.source_span_start, replacement.source_span_end))
                .collect::<Vec<_>>(),
        );
        if removed_count == 0 {
            return (output, mutation_count);
        }
        output = next_output;
        mutation_count += removed_count;
    }
}

pub(crate) fn remove_empty_css_rules_with_ir_transaction(
    source: &str,
    dialect: StyleDialect,
) -> Result<(String, usize), TransformIrSourceReplacementErrorV0> {
    let mut ir = omena_transform_cst::lower_transform_ir_from_source(
        source,
        dialect,
        "omena-transform-passes.empty-rule-removal",
    );
    remove_empty_css_rules_with_ir_transaction_on_ir(&mut ir, dialect)
}

pub(crate) fn remove_empty_css_rules_with_ir_transaction_on_ir(
    ir: &mut TransformIrV0,
    dialect: StyleDialect,
) -> Result<(String, usize), TransformIrSourceReplacementErrorV0> {
    let mut output = ir.source_text().to_string();
    let mut mutation_count = 0;

    loop {
        let lexed = lex(&output, dialect);
        let tokens = lexed.tokens();
        let replacements = collect_empty_rule_replacements(tokens);
        let (next_output, removed_count) = apply_ir_source_replacements_to_ir(
            ir,
            dialect,
            "empty-rule-removal",
            replacements.as_slice(),
        )?;
        if removed_count == 0 {
            return Ok((output, mutation_count));
        }
        output = next_output;
        mutation_count += removed_count;
    }
}

fn collect_empty_rule_replacements(tokens: &[LexedToken]) -> Vec<TransformIrSourceReplacementV0> {
    let mut replacements = Vec::new();
    let mut depth = 0usize;
    let mut prelude_starts = vec![0usize];
    let mut keyframes_contexts = vec![false];
    let mut index = 0;

    while index < tokens.len() {
        match tokens[index].kind {
            SyntaxKind::LeftBrace => {
                let prelude_start = prelude_starts.get(depth).copied().unwrap_or(0);
                let inside_keyframes = keyframes_contexts.get(depth).copied().unwrap_or(false);
                let is_removable_ordinary_rule =
                    !inside_keyframes && is_ordinary_rule_prelude(tokens, prelude_start, index);
                let is_removable_group_rule =
                    is_empty_group_rule_prelude(tokens, prelude_start, index);
                if let Some(close_index) = matching_right_brace_index(tokens, index)
                    && is_empty_rule_block(tokens, index + 1, close_index)
                    && (is_removable_ordinary_rule || is_removable_group_rule)
                    && let Some(start) = first_non_trivia_token_start(tokens, prelude_start, index)
                {
                    let end = token_end(&tokens[close_index]);
                    replacements.push(TransformIrSourceReplacementV0 {
                        source_span_start: start,
                        source_span_end: end,
                        replacement: String::new(),
                        kind: if is_removable_group_rule {
                            TransformIrReplacementKindV0::AtRule
                        } else {
                            TransformIrReplacementKindV0::StyleRule
                        },
                    });
                    index = close_index + 1;
                    set_prelude_start(&mut prelude_starts, depth, index);
                    continue;
                }
                let child_inside_keyframes = inside_keyframes
                    || is_keyframes_group_rule_prelude(tokens, prelude_start, index);
                depth += 1;
                set_prelude_start(&mut prelude_starts, depth, index + 1);
                set_bool_context(&mut keyframes_contexts, depth, child_inside_keyframes);
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

    replacements
}

fn set_bool_context(contexts: &mut Vec<bool>, depth: usize, value: bool) {
    if contexts.len() <= depth {
        contexts.resize(depth + 1, false);
    }
    contexts[depth] = value;
}

fn is_empty_rule_block(tokens: &[LexedToken], start: usize, end_exclusive: usize) -> bool {
    tokens[start..end_exclusive].iter().all(|token| {
        matches!(
            token.kind,
            SyntaxKind::Whitespace | SyntaxKind::SassIndentedNewline
        )
    })
}

fn is_empty_group_rule_prelude(tokens: &[LexedToken], start: usize, end_exclusive: usize) -> bool {
    let prelude = &tokens[start..end_exclusive];
    let mut significant_tokens = prelude
        .iter()
        .filter(|token| !is_comment_token(token.kind) && token.kind != SyntaxKind::Whitespace);
    let Some(first) = significant_tokens.next() else {
        return false;
    };
    first.kind == SyntaxKind::AtKeyword && is_empty_removable_group_at_keyword(&first.text)
}

fn is_keyframes_group_rule_prelude(
    tokens: &[LexedToken],
    start: usize,
    end_exclusive: usize,
) -> bool {
    let prelude = &tokens[start..end_exclusive];
    let mut significant_tokens = prelude
        .iter()
        .filter(|token| !is_comment_token(token.kind) && token.kind != SyntaxKind::Whitespace);
    let Some(first) = significant_tokens.next() else {
        return false;
    };
    first.kind == SyntaxKind::AtKeyword && is_keyframes_at_keyword(&first.text)
}

fn is_empty_removable_group_at_keyword(text: &str) -> bool {
    matches!(
        text.to_ascii_lowercase().as_str(),
        "@container" | "@layer" | "@media" | "@scope" | "@supports"
    )
}
