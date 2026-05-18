use omena_parser::{StyleDialect, lex};
use omena_syntax::SyntaxKind;

use crate::helpers::{
    blocks::at_rule_block_indexes,
    rules::rule_gap_is_whitespace_only,
    source_rewrite::replace_source_ranges,
    tokens::{token_end, token_start},
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
