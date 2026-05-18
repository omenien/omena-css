use omena_parser::{StyleDialect, lex};
use omena_syntax::SyntaxKind;

use crate::{
    domains::selector::dedupe_selector_arguments,
    helpers::{
        blocks::at_rule_block_indexes,
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
            if current.block != next.block
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
            replacements.push((
                current.start,
                last.end,
                format!(
                    "{}, {} {{ {} }}",
                    deduped_selectors[0],
                    deduped_selectors[1..].join(", "),
                    current.block
                ),
            ));
        } else {
            index += 1;
            continue;
        }

        index = run_end;
    }

    replace_source_ranges(source, &replacements)
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

fn merge_adjacent_same_selector_ordinary_css_rules_with_lexer(
    source: &str,
    dialect: StyleDialect,
) -> (String, usize) {
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
            replacements.push((
                current.start,
                last.end,
                format!(
                    "{} {{ {} }}",
                    current.selector,
                    join_rule_blocks_for_merge(&blocks)
                ),
            ));
        } else {
            index += 1;
            continue;
        }

        index = run_end;
    }

    replace_source_ranges(source, &replacements)
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
            replacements.push((
                current.start,
                last.end,
                format!("{} {} {{ {} }}", current.at_keyword, current.prelude, block),
            ));
        } else {
            index += 1;
            continue;
        }

        index = run_end;
    }

    replace_source_ranges(source, &replacements)
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
