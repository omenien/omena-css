use omena_parser::{LexedToken, StyleDialect, lex};
use omena_syntax::SyntaxKind;

use crate::domains::keyframes::is_keyframes_at_keyword;
use crate::helpers::{
    declarations::collect_simple_declarations_in_block,
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
    let ranges = collect_duplicate_ordinary_rule_ranges(&rules);

    if ranges.is_empty() {
        return (source.to_string(), declaration_count);
    }

    let (output, rule_count) = remove_source_ranges(source, &ranges);
    (output, declaration_count + rule_count)
}

fn remove_overridden_same_property_declarations_with_lexer(
    source: &str,
    dialect: StyleDialect,
) -> (String, usize) {
    let lexed = lex(source, dialect);
    let tokens = lexed.tokens();
    let mut ranges = Vec::new();

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
                ranges.push((declaration.start, declaration.end));
            }
        }
    }

    remove_source_ranges(source, &ranges)
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

fn collect_duplicate_ordinary_rule_ranges(rules: &[SimpleRuleSlice]) -> Vec<(usize, usize)> {
    let mut ranges = Vec::new();

    for (index, rule) in rules.iter().enumerate() {
        let has_later_duplicate = rules[index + 1..].iter().any(|candidate| {
            rule.selector == candidate.selector
                && rule.block == candidate.block
                && rule.context_start == candidate.context_start
                && rule.context_end == candidate.context_end
        });
        if has_later_duplicate {
            ranges.push((rule.start, rule.end));
        }
    }

    ranges
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
        let ranges = collect_empty_rule_ranges(tokens);
        let (next_output, removed_count) = remove_source_ranges(&output, &ranges);
        if removed_count == 0 {
            return (output, mutation_count);
        }
        output = next_output;
        mutation_count += removed_count;
    }
}

fn collect_empty_rule_ranges(tokens: &[LexedToken]) -> Vec<(usize, usize)> {
    let mut ranges = Vec::new();
    let mut depth = 0usize;
    let mut prelude_starts = vec![0usize];
    let mut keyframes_contexts = vec![false];
    let mut index = 0;

    while index < tokens.len() {
        match tokens[index].kind {
            SyntaxKind::LeftBrace => {
                let prelude_start = prelude_starts.get(depth).copied().unwrap_or(0);
                let inside_keyframes = keyframes_contexts.get(depth).copied().unwrap_or(false);
                if let Some(close_index) = matching_right_brace_index(tokens, index)
                    && is_empty_rule_block(tokens, index + 1, close_index)
                    && ((!inside_keyframes
                        && is_ordinary_rule_prelude(tokens, prelude_start, index))
                        || is_empty_group_rule_prelude(tokens, prelude_start, index))
                    && let Some(start) = first_non_trivia_token_start(tokens, prelude_start, index)
                {
                    let end = token_end(&tokens[close_index]);
                    ranges.push((start, end));
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

    ranges
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
