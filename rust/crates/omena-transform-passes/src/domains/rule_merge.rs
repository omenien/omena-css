use omena_parser::StyleDialect;
use omena_syntax::SyntaxKind;
use omena_transform_cst::{TransformIrV0, lower_transform_ir_from_source};

use crate::runtime::lex_cache::lex_cached as lex;

use crate::{
    domains::selector::dedupe_selector_arguments,
    helpers::{
        blocks::at_rule_block_indexes,
        ir_transaction::{
            TransformIrReplacementKindV0, TransformIrSourceReplacementErrorV0,
            TransformIrSourceReplacementV0, apply_ir_source_replacements_to_ir,
        },
        rules::{collect_declaration_ordinary_rule_slices, rule_gap_is_whitespace_only},
        source_rewrite::replace_source_ranges,
        tokens::{token_end, token_start},
    },
};

#[derive(Debug, Clone, PartialEq, Eq)]
struct ConditionalAtRuleBlockSlice {
    at_keyword: String,
    prelude: String,
    start: usize,
    end: usize,
    block_start: usize,
    block_end: usize,
}

pub(crate) fn merge_adjacent_same_block_css_selectors_with_lexer(
    source: &str,
    dialect: StyleDialect,
) -> (String, usize) {
    let replacements = collect_adjacent_same_block_selector_replacements(source, dialect);
    replace_source_ranges(source, &source_replacement_ranges(&replacements))
}

pub(crate) fn merge_adjacent_same_block_css_selectors_with_ir_transaction(
    source: &str,
    dialect: StyleDialect,
) -> Result<(String, usize), TransformIrSourceReplacementErrorV0> {
    let mut ir =
        lower_transform_ir_from_source(source, dialect, "omena-transform-passes.selector-merging");
    merge_adjacent_same_block_css_selectors_with_ir_transaction_on_ir(&mut ir, dialect)
}

pub(crate) fn merge_adjacent_same_block_css_selectors_with_ir_transaction_on_ir(
    ir: &mut TransformIrV0,
    dialect: StyleDialect,
) -> Result<(String, usize), TransformIrSourceReplacementErrorV0> {
    let replacements = collect_adjacent_same_block_selector_replacements(ir.source_text(), dialect);
    apply_ir_source_replacements_to_ir(ir, dialect, "selector-merging", replacements.as_slice())
}

fn collect_adjacent_same_block_selector_replacements(
    source: &str,
    dialect: StyleDialect,
) -> Vec<TransformIrSourceReplacementV0> {
    let lexed = lex(source, dialect);
    let tokens = lexed.tokens();
    let rules = collect_declaration_ordinary_rule_slices(source, tokens);
    let mut replacements = Vec::new();
    let mut index = 0;

    while index < rules.len() {
        let current = &rules[index];
        let mut selectors = vec![current.selector.clone()];
        let mut run_end = index + 1;

        while run_end < rules.len() {
            let previous = &rules[run_end - 1];
            let next = &rules[run_end];
            if normalized_same_block_merge_value(&current.block)
                != normalized_same_block_merge_value(&next.block)
                || !rule_gap_is_whitespace_only(tokens, previous.end, next.start)
            {
                break;
            }
            selectors.push(next.selector.clone());
            run_end += 1;
        }

        let deduped_selectors = dedupe_selector_arguments(&selectors);
        if deduped_selectors.len() > 1 {
            let last = &rules[run_end - 1];
            replacements.push(TransformIrSourceReplacementV0 {
                source_span_start: current.start,
                source_span_end: last.end,
                replacement: format!(
                    "{}, {} {{ {} }}",
                    deduped_selectors[0],
                    deduped_selectors[1..].join(", "),
                    current.block
                ),
                kind: TransformIrReplacementKindV0::StyleRule,
            });
        } else {
            index += 1;
            continue;
        }

        index = run_end;
    }

    replacements
}

fn normalized_same_block_merge_value(block: &str) -> String {
    let block = block.trim().trim_end_matches(';').trim_end();
    let mut output = String::with_capacity(block.len());
    let mut index = 0usize;
    let mut quote: Option<char> = None;

    while index < block.len() {
        let Some(ch) = block[index..].chars().next() else {
            break;
        };
        if let Some(quote_ch) = quote {
            output.push(ch);
            index += ch.len_utf8();
            if ch == '\\' {
                if let Some(escaped) = block[index..].chars().next() {
                    output.push(escaped);
                    index += escaped.len_utf8();
                }
            } else if ch == quote_ch {
                quote = None;
            }
            continue;
        }

        match ch {
            '"' | '\'' => {
                quote = Some(ch);
                output.push(ch);
                index += ch.len_utf8();
            }
            ':' | ';' => {
                while output.chars().next_back().is_some_and(char::is_whitespace) {
                    output.pop();
                }
                output.push(ch);
                index += ch.len_utf8();
                while let Some(next) = block[index..].chars().next() {
                    if !next.is_whitespace() {
                        break;
                    }
                    index += next.len_utf8();
                }
            }
            _ => {
                output.push(ch);
                index += ch.len_utf8();
            }
        }
    }

    output
}

pub(crate) fn merge_adjacent_same_selector_css_rules_with_lexer(
    source: &str,
    dialect: StyleDialect,
) -> (String, usize) {
    let (output, ordinary_mutation_count) =
        merge_adjacent_same_selector_ordinary_css_rules_with_lexer(source, dialect);
    let (output, at_rule_mutation_count) =
        merge_adjacent_same_conditional_at_rule_blocks_with_lexer(&output, dialect);
    (output, ordinary_mutation_count + at_rule_mutation_count)
}

pub(crate) fn merge_adjacent_same_selector_css_rules_with_ir_transaction(
    source: &str,
    dialect: StyleDialect,
) -> Result<(String, usize), TransformIrSourceReplacementErrorV0> {
    let mut ir =
        lower_transform_ir_from_source(source, dialect, "omena-transform-passes.rule-merging");
    merge_adjacent_same_selector_css_rules_with_ir_transaction_on_ir(&mut ir, dialect)
}

pub(crate) fn merge_adjacent_same_selector_css_rules_with_ir_transaction_on_ir(
    ir: &mut TransformIrV0,
    dialect: StyleDialect,
) -> Result<(String, usize), TransformIrSourceReplacementErrorV0> {
    let ordinary_replacements =
        collect_adjacent_same_selector_ordinary_rule_replacements(ir.source_text(), dialect);
    let (_, ordinary_mutation_count) = apply_ir_source_replacements_to_ir(
        ir,
        dialect,
        "rule-merging",
        ordinary_replacements.as_slice(),
    )?;
    let at_rule_replacements =
        collect_adjacent_same_conditional_at_rule_block_replacements(ir.source_text(), dialect);
    let (output, at_rule_mutation_count) = apply_ir_source_replacements_to_ir(
        ir,
        dialect,
        "rule-merging",
        at_rule_replacements.as_slice(),
    )?;
    Ok((output, ordinary_mutation_count + at_rule_mutation_count))
}

fn merge_adjacent_same_selector_ordinary_css_rules_with_lexer(
    source: &str,
    dialect: StyleDialect,
) -> (String, usize) {
    let replacements = collect_adjacent_same_selector_ordinary_rule_replacements(source, dialect);
    replace_source_ranges(source, &source_replacement_ranges(&replacements))
}

fn collect_adjacent_same_selector_ordinary_rule_replacements(
    source: &str,
    dialect: StyleDialect,
) -> Vec<TransformIrSourceReplacementV0> {
    let lexed = lex(source, dialect);
    let tokens = lexed.tokens();
    let rules = collect_declaration_ordinary_rule_slices(source, tokens);
    let mut replacements = Vec::new();
    let mut index = 0;

    while index < rules.len() {
        let current = &rules[index];
        let mut blocks = vec![current.block.clone()];
        let mut run_end = index + 1;

        while run_end < rules.len() {
            let previous = &rules[run_end - 1];
            let next = &rules[run_end];
            if current.selector != next.selector
                || !rule_gap_is_whitespace_only(tokens, previous.end, next.start)
            {
                break;
            }
            blocks.push(next.block.clone());
            run_end += 1;
        }

        if blocks.len() > 1 && blocks.iter().any(|block| block != &blocks[0]) {
            let last = &rules[run_end - 1];
            replacements.push(TransformIrSourceReplacementV0 {
                source_span_start: current.start,
                source_span_end: last.end,
                replacement: format!(
                    "{} {{ {} }}",
                    current.selector,
                    join_rule_blocks_for_merge(&blocks)
                ),
                kind: TransformIrReplacementKindV0::StyleRule,
            });
        } else {
            index += 1;
            continue;
        }

        index = run_end;
    }

    replacements
}

fn join_rule_blocks_for_merge(blocks: &[String]) -> String {
    blocks
        .iter()
        .filter_map(|block| {
            let trimmed = block.trim();
            if trimmed.is_empty() {
                None
            } else if trimmed.ends_with(';') {
                Some(trimmed.to_string())
            } else {
                Some(format!("{trimmed};"))
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

pub(crate) fn merge_adjacent_same_conditional_at_rule_blocks_with_lexer(
    source: &str,
    dialect: StyleDialect,
) -> (String, usize) {
    let replacements =
        collect_adjacent_same_conditional_at_rule_block_replacements(source, dialect);
    replace_source_ranges(source, &source_replacement_ranges(&replacements))
}

fn collect_adjacent_same_conditional_at_rule_block_replacements(
    source: &str,
    dialect: StyleDialect,
) -> Vec<TransformIrSourceReplacementV0> {
    let lexed = lex(source, dialect);
    let tokens = lexed.tokens();
    let at_rules = collect_top_level_mergeable_conditional_at_rule_blocks(source, tokens);
    let mut replacements = Vec::new();
    let mut index = 0usize;

    while index < at_rules.len() {
        let current = &at_rules[index];
        let mut blocks = vec![
            source[current.block_start..current.block_end]
                .trim()
                .to_string(),
        ];
        let mut run_end = index + 1;

        while run_end < at_rules.len() {
            let previous = &at_rules[run_end - 1];
            let next = &at_rules[run_end];
            if current.at_keyword != next.at_keyword
                || current.prelude != next.prelude
                || !rule_gap_is_whitespace_only(tokens, previous.end, next.start)
            {
                break;
            }
            blocks.push(source[next.block_start..next.block_end].trim().to_string());
            run_end += 1;
        }

        if blocks.len() > 1 {
            let last = &at_rules[run_end - 1];
            let block = blocks
                .iter()
                .filter(|block| !block.is_empty())
                .cloned()
                .collect::<Vec<_>>()
                .join(" ");
            replacements.push(TransformIrSourceReplacementV0 {
                source_span_start: current.start,
                source_span_end: last.end,
                replacement: format!("{} {} {{ {} }}", current.at_keyword, current.prelude, block),
                kind: TransformIrReplacementKindV0::AtRule,
            });
        } else {
            index += 1;
            continue;
        }

        index = run_end;
    }

    replacements
}

fn source_replacement_ranges(
    replacements: &[TransformIrSourceReplacementV0],
) -> Vec<(usize, usize, String)> {
    replacements
        .iter()
        .map(|replacement| {
            (
                replacement.source_span_start,
                replacement.source_span_end,
                replacement.replacement.clone(),
            )
        })
        .collect()
}

fn collect_top_level_mergeable_conditional_at_rule_blocks(
    source: &str,
    tokens: &[omena_parser::LexedToken],
) -> Vec<ConditionalAtRuleBlockSlice> {
    let mut at_rules = Vec::new();
    let mut depth = 0usize;
    let mut index = 0usize;

    while index < tokens.len() {
        match tokens[index].kind {
            SyntaxKind::AtKeyword
                if depth == 0 && conditional_at_rule_can_merge(&tokens[index].text) =>
            {
                let Some((block_start_index, block_end_index)) =
                    at_rule_block_indexes(tokens, index)
                else {
                    index += 1;
                    continue;
                };
                let prelude = source
                    [token_end(&tokens[index])..token_start(&tokens[block_start_index])]
                    .trim()
                    .to_string();
                at_rules.push(ConditionalAtRuleBlockSlice {
                    at_keyword: tokens[index].text.to_ascii_lowercase(),
                    prelude,
                    start: token_start(&tokens[index]),
                    end: token_end(&tokens[block_end_index]),
                    block_start: token_end(&tokens[block_start_index]),
                    block_end: token_start(&tokens[block_end_index]),
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

    at_rules
}

fn conditional_at_rule_can_merge(at_keyword: &str) -> bool {
    matches!(
        at_keyword.to_ascii_lowercase().as_str(),
        "@media" | "@supports"
    )
}
