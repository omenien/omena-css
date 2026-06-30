use omena_parser::StyleDialect;
use omena_syntax::SyntaxKind;
use omena_transform_cst::{IrNodeKindV0, IrNodeV0, TransformIrV0};

use crate::runtime::lex_cache::lex_cached as lex;

use crate::{
    domains::selector::dedupe_selector_arguments,
    helpers::{
        blocks::at_rule_block_indexes,
        ir_transaction::{
            TransformIrReplacementKindV0, TransformIrSourceReplacementErrorV0,
            TransformIrSourceReplacementV0, replace_ir_node_spans_in_ir,
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

#[derive(Debug, Clone, PartialEq, Eq)]
struct SimpleRuleSliceV0 {
    selector: String,
    block: String,
    start: usize,
    end: usize,
}

pub(crate) fn merge_adjacent_same_block_css_selectors_with_lexer(
    source: &str,
    dialect: StyleDialect,
) -> (String, usize) {
    let replacements = collect_adjacent_same_block_selector_replacements(source, dialect);
    replace_source_ranges(source, &source_replacement_ranges(&replacements))
}

pub(crate) fn merge_adjacent_same_block_css_selectors_with_ir_transaction_on_ir(
    ir: &mut TransformIrV0,
    _dialect: StyleDialect,
) -> Result<usize, TransformIrSourceReplacementErrorV0> {
    let replacements = collect_adjacent_same_block_selector_replacements_from_ir(ir);
    replace_ir_node_spans_in_ir(ir, "selector-merging", replacements.as_slice())
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

fn collect_adjacent_same_block_selector_replacements_from_ir(
    ir: &TransformIrV0,
) -> Vec<TransformIrSourceReplacementV0> {
    let rules = collect_declaration_ordinary_rule_slices_from_ir(ir);
    collect_adjacent_same_block_selector_replacements_from_rules(ir.source_text(), &rules)
}

fn collect_adjacent_same_block_selector_replacements_from_rules(
    source: &str,
    rules: &[SimpleRuleSliceV0],
) -> Vec<TransformIrSourceReplacementV0> {
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
                || !source_gap_is_whitespace_only(source, previous.end, next.start)
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

pub(crate) fn merge_adjacent_same_selector_css_rules_with_ir_transaction_on_ir(
    ir: &mut TransformIrV0,
    _dialect: StyleDialect,
) -> Result<usize, TransformIrSourceReplacementErrorV0> {
    let ordinary_replacements =
        collect_adjacent_same_selector_ordinary_rule_replacements_from_ir(ir);
    let ordinary_mutation_count =
        replace_ir_node_spans_in_ir(ir, "rule-merging", ordinary_replacements.as_slice())?;
    let at_rule_replacements =
        collect_adjacent_same_conditional_at_rule_block_replacements_from_ir(ir);
    let at_rule_mutation_count =
        replace_ir_node_spans_in_ir(ir, "rule-merging", at_rule_replacements.as_slice())?;
    Ok(ordinary_mutation_count + at_rule_mutation_count)
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

fn collect_adjacent_same_selector_ordinary_rule_replacements_from_ir(
    ir: &TransformIrV0,
) -> Vec<TransformIrSourceReplacementV0> {
    let rules = collect_declaration_ordinary_rule_slices_from_ir(ir);
    collect_adjacent_same_selector_ordinary_rule_replacements_from_rules(ir.source_text(), &rules)
}

fn collect_adjacent_same_selector_ordinary_rule_replacements_from_rules(
    source: &str,
    rules: &[SimpleRuleSliceV0],
) -> Vec<TransformIrSourceReplacementV0> {
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
                || !source_gap_is_whitespace_only(source, previous.end, next.start)
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

fn collect_adjacent_same_conditional_at_rule_block_replacements_from_ir(
    ir: &TransformIrV0,
) -> Vec<TransformIrSourceReplacementV0> {
    let at_rules = collect_top_level_mergeable_conditional_at_rule_blocks_from_ir(ir);
    let mut replacements = Vec::new();
    let mut index = 0usize;

    while index < at_rules.len() {
        let current = &at_rules[index];
        let mut blocks = vec![
            ir.source_text()[current.block_start..current.block_end]
                .trim()
                .to_string(),
        ];
        let mut run_end = index + 1;

        while run_end < at_rules.len() {
            let previous = &at_rules[run_end - 1];
            let next = &at_rules[run_end];
            if current.at_keyword != next.at_keyword
                || current.prelude != next.prelude
                || !source_gap_is_whitespace_only(ir.source_text(), previous.end, next.start)
            {
                break;
            }
            blocks.push(
                ir.source_text()[next.block_start..next.block_end]
                    .trim()
                    .to_string(),
            );
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

fn collect_declaration_ordinary_rule_slices_from_ir(ir: &TransformIrV0) -> Vec<SimpleRuleSliceV0> {
    let mut rules = ir
        .nodes
        .iter()
        .filter(|node| !node.deleted && node.kind == IrNodeKindV0::StyleRule)
        .filter_map(|node| simple_rule_slice_from_ir(ir, node))
        .collect::<Vec<_>>();
    rules.sort_by_key(|rule| (rule.start, rule.end));
    rules
}

fn simple_rule_slice_from_ir(ir: &TransformIrV0, node: &IrNodeV0) -> Option<SimpleRuleSliceV0> {
    let source = ir.source_text();
    let rule_source = source.get(node.source_span_start..node.source_span_end)?;
    let open = rule_source.find('{')?;
    let close = rule_source.rfind('}')?;
    if open >= close {
        return None;
    }
    let selector = rule_source.get(..open)?.trim().to_string();
    let block = rule_source.get(open + 1..close)?.trim().to_string();
    if selector.is_empty() || block.is_empty() || block_contains_nested_or_comment(&block) {
        return None;
    }
    Some(SimpleRuleSliceV0 {
        selector,
        block,
        start: node.source_span_start,
        end: node.source_span_end,
    })
}

fn collect_top_level_mergeable_conditional_at_rule_blocks_from_ir(
    ir: &TransformIrV0,
) -> Vec<ConditionalAtRuleBlockSlice> {
    let mut at_rules = ir
        .nodes
        .iter()
        .filter(|node| !node.deleted && node.parent.is_none() && node.kind == IrNodeKindV0::AtRule)
        .filter_map(|node| conditional_at_rule_block_slice_from_ir(ir, node))
        .collect::<Vec<_>>();
    at_rules.sort_by_key(|rule| (rule.start, rule.end));
    at_rules
}

fn conditional_at_rule_block_slice_from_ir(
    ir: &TransformIrV0,
    node: &IrNodeV0,
) -> Option<ConditionalAtRuleBlockSlice> {
    let source = ir.source_text();
    let rule_source = source.get(node.source_span_start..node.source_span_end)?;
    let leading_offset = rule_source
        .len()
        .saturating_sub(rule_source.trim_start().len());
    let at_keyword_start = node.source_span_start.checked_add(leading_offset)?;
    let rest = source.get(at_keyword_start..node.source_span_end)?;
    let at_keyword_end_offset = rest
        .find(|ch: char| ch.is_whitespace() || matches!(ch, '{' | '(' | ';'))
        .unwrap_or(rest.len());
    let at_keyword_end = at_keyword_start.checked_add(at_keyword_end_offset)?;
    let at_keyword = source
        .get(at_keyword_start..at_keyword_end)?
        .to_ascii_lowercase();
    if !conditional_at_rule_can_merge(&at_keyword) {
        return None;
    }
    let relative_block_start = rule_source.get(leading_offset..)?.find('{')?;
    let relative_block_end = rule_source.rfind('}')?;
    if relative_block_start >= relative_block_end {
        return None;
    }
    let block_start = node
        .source_span_start
        .checked_add(leading_offset + relative_block_start)?;
    let block_end = node.source_span_start.checked_add(relative_block_end)?;
    Some(ConditionalAtRuleBlockSlice {
        at_keyword,
        prelude: source.get(at_keyword_end..block_start)?.trim().to_string(),
        start: node.source_span_start,
        end: node.source_span_end,
        block_start: block_start.saturating_add(1),
        block_end,
    })
}

fn source_gap_is_whitespace_only(source: &str, start: usize, end: usize) -> bool {
    source
        .get(start..end)
        .is_some_and(|gap| gap.chars().all(char::is_whitespace))
}

fn block_contains_nested_or_comment(block: &str) -> bool {
    let bytes = block.as_bytes();
    let mut index = 0usize;
    let mut quote = None;
    let mut escaped = false;
    while index < bytes.len() {
        let byte = bytes[index];
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
        if byte == b'\'' || byte == b'"' {
            quote = Some(byte);
            index += 1;
            continue;
        }
        if matches!(byte, b'{' | b'}') || (byte == b'/' && bytes.get(index + 1) == Some(&b'*')) {
            return true;
        }
        index += 1;
    }
    false
}

fn conditional_at_rule_can_merge(at_keyword: &str) -> bool {
    matches!(
        at_keyword.to_ascii_lowercase().as_str(),
        "@media" | "@supports"
    )
}
