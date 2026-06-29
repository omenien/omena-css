use omena_parser::StyleDialect;
use omena_syntax::SyntaxKind;
use omena_transform_cst::{TransformIrV0, lower_transform_ir_from_source};

use crate::runtime::lex_cache::lex_cached as lex;

use crate::helpers::{
    declarations::collect_simple_declarations_in_block,
    ir_transaction::{
        TransformIrReplacementKindV0, TransformIrSourceReplacementErrorV0,
        TransformIrSourceReplacementV0, apply_ir_source_replacements_to_ir,
    },
    rules::{first_non_trivia_token_start, is_ordinary_rule_prelude, set_prelude_start},
    selectors::split_css_selector_list,
    source_rewrite::replace_source_ranges,
    tokens::{is_comment_token, matching_right_brace_index, token_end, token_start},
};

pub(crate) fn unwrap_css_nesting_with_lexer(
    source: &str,
    dialect: StyleDialect,
) -> (String, usize) {
    let replacements = collect_nesting_unwrap_replacements(source, dialect);
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

pub(crate) fn unwrap_css_nesting_with_ir_transaction(
    source: &str,
    dialect: StyleDialect,
) -> Result<(String, usize), TransformIrSourceReplacementErrorV0> {
    let mut ir = lower_transform_ir_from_source(source, dialect, "omena-transform-passes.nesting");
    unwrap_css_nesting_with_ir_transaction_on_ir(&mut ir, dialect)
}

pub(crate) fn unwrap_css_nesting_with_ir_transaction_on_ir(
    ir: &mut TransformIrV0,
    dialect: StyleDialect,
) -> Result<(String, usize), TransformIrSourceReplacementErrorV0> {
    let replacements = collect_nesting_unwrap_replacements(ir.source_text(), dialect);
    apply_ir_source_replacements_to_ir(ir, dialect, "nesting-unwrap", replacements.as_slice())
}

fn collect_nesting_unwrap_replacements(
    source: &str,
    dialect: StyleDialect,
) -> Vec<TransformIrSourceReplacementV0> {
    let lexed = lex(source, dialect);
    let tokens = lexed.tokens();
    let mut replacements = Vec::new();
    let mut depth = 0usize;
    let mut prelude_starts = vec![0usize];
    let mut index = 0;

    while index < tokens.len() {
        match tokens[index].kind {
            SyntaxKind::LeftBrace => {
                let prelude_start = prelude_starts.get(depth).copied().unwrap_or(0);
                if let Some(close_index) = matching_right_brace_index(tokens, index)
                    && is_ordinary_rule_prelude(tokens, prelude_start, index)
                    && let Some(start) = first_non_trivia_token_start(tokens, prelude_start, index)
                    && let Some(replacement) =
                        unwrap_simple_nested_rule(source, tokens, start, index, close_index)
                {
                    replacements.push(TransformIrSourceReplacementV0 {
                        source_span_start: start,
                        source_span_end: token_end(&tokens[close_index]),
                        replacement,
                        kind: TransformIrReplacementKindV0::StyleRule,
                    });
                    index = close_index + 1;
                    set_prelude_start(&mut prelude_starts, depth, index);
                    continue;
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

    replacements
}

fn unwrap_simple_nested_rule(
    source: &str,
    tokens: &[omena_parser::LexedToken],
    rule_start: usize,
    block_start_index: usize,
    block_end_index: usize,
) -> Option<String> {
    if tokens[block_start_index + 1..block_end_index]
        .iter()
        .any(|token| is_comment_token(token.kind))
    {
        return None;
    }

    let parent_selector = source[rule_start..token_start(&tokens[block_start_index])]
        .trim()
        .to_string();
    if parent_selector.is_empty() || split_css_selector_list(&parent_selector).is_none() {
        return None;
    }

    let rule_texts = unwrap_nested_rule_body(
        source,
        tokens,
        &parent_selector,
        block_start_index,
        block_end_index,
        true,
    )?;
    Some(rule_texts.join(" "))
}

fn unwrap_nested_rule_body(
    source: &str,
    tokens: &[omena_parser::LexedToken],
    parent_selector: &str,
    block_start_index: usize,
    block_end_index: usize,
    require_nested_rule: bool,
) -> Option<Vec<String>> {
    let declarations =
        collect_simple_declarations_in_block(tokens, block_start_index, block_end_index);
    let nested_rules =
        collect_direct_nested_rule_slices(source, tokens, block_start_index, block_end_index)?;
    if require_nested_rule && nested_rules.is_empty() {
        return None;
    }

    let mut rule_texts = Vec::new();
    if !declarations.is_empty() {
        let declarations_text = declarations
            .iter()
            .map(|declaration| format!("{}: {};", declaration.property, declaration.value))
            .collect::<Vec<_>>()
            .join(" ");
        rule_texts.push(format!("{parent_selector} {{ {declarations_text} }}"));
    }

    for nested_rule in nested_rules {
        match nested_rule.kind {
            NestedRuleKind::Style => {
                let selector = expand_nested_selector(parent_selector, &nested_rule.selector)?;
                let nested_rule_texts = unwrap_nested_rule_body(
                    source,
                    tokens,
                    &selector,
                    nested_rule.block_start_index,
                    nested_rule.block_end_index,
                    false,
                )?;
                rule_texts.extend(nested_rule_texts);
            }
            NestedRuleKind::ConditionalGroup => {
                let nested_rule_texts = unwrap_nested_rule_body(
                    source,
                    tokens,
                    parent_selector,
                    nested_rule.block_start_index,
                    nested_rule.block_end_index,
                    false,
                )?;
                rule_texts.push(format!(
                    "{} {{ {} }}",
                    nested_rule.selector,
                    nested_rule_texts.join(" ")
                ));
            }
        }
    }

    if rule_texts.is_empty() {
        None
    } else {
        Some(rule_texts)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum NestedRuleKind {
    Style,
    ConditionalGroup,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct NestedRuleSlice {
    selector: String,
    block_start_index: usize,
    block_end_index: usize,
    kind: NestedRuleKind,
}

fn collect_direct_nested_rule_slices(
    source: &str,
    tokens: &[omena_parser::LexedToken],
    block_start_index: usize,
    block_end_index: usize,
) -> Option<Vec<NestedRuleSlice>> {
    let mut nested_rules = Vec::new();
    let mut segment_start_index = block_start_index + 1;
    let mut index = block_start_index + 1;

    while index < block_end_index {
        if tokens[index].kind == SyntaxKind::LeftBrace {
            let nested_close_index = matching_right_brace_index(tokens, index)?;
            if nested_close_index > block_end_index {
                return None;
            }
            let selector_start = first_non_trivia_token_start(tokens, segment_start_index, index)?;
            let selector = source[selector_start..token_start(&tokens[index])]
                .trim()
                .to_string();
            if selector.is_empty() {
                return None;
            }
            let (kind, selector) =
                if let Some(nest_selector) = parse_nest_at_rule_selector(&selector) {
                    (NestedRuleKind::Style, nest_selector)
                } else if selector.starts_with('@') {
                    if !is_supported_nested_conditional_group_rule(&selector) {
                        return None;
                    }
                    (NestedRuleKind::ConditionalGroup, selector)
                } else {
                    split_css_selector_list(&selector)?;
                    (NestedRuleKind::Style, selector)
                };
            if source[token_end(&tokens[index])..token_start(&tokens[nested_close_index])]
                .trim()
                .is_empty()
            {
                return None;
            }
            nested_rules.push(NestedRuleSlice {
                selector,
                block_start_index: index,
                block_end_index: nested_close_index,
                kind,
            });
            index = nested_close_index + 1;
            segment_start_index = index;
            continue;
        }
        if tokens[index].kind == SyntaxKind::Semicolon {
            segment_start_index = index + 1;
        }
        index += 1;
    }

    Some(nested_rules)
}

fn parse_nest_at_rule_selector(selector: &str) -> Option<String> {
    let selector = selector.trim_start();
    let rest = strip_ascii_prefix_ignore_case(selector, "@nest")?;
    if !rest.starts_with(char::is_whitespace) {
        return None;
    }
    let nested_selector = rest.trim();
    if !nested_selector.contains('&') {
        return None;
    }
    split_css_selector_list(nested_selector)?;
    Some(nested_selector.to_string())
}

fn strip_ascii_prefix_ignore_case<'a>(text: &'a str, prefix: &str) -> Option<&'a str> {
    let candidate = text.get(..prefix.len())?;
    if candidate.eq_ignore_ascii_case(prefix) {
        Some(&text[prefix.len()..])
    } else {
        None
    }
}

fn is_supported_nested_conditional_group_rule(selector: &str) -> bool {
    let selector = selector.trim_start().to_ascii_lowercase();
    [
        "@media",
        "@supports",
        "@container",
        "@layer",
        "@starting-style",
    ]
    .iter()
    .any(|prefix| selector.starts_with(prefix))
}

pub(crate) fn expand_nested_selector(
    parent_selector: &str,
    nested_selector: &str,
) -> Option<String> {
    let parent_selectors = split_css_selector_list(parent_selector)?;
    let nested_selectors = split_css_selector_list(nested_selector)?;
    let mut expanded_selectors = Vec::new();

    for parent in &parent_selectors {
        for nested in &nested_selectors {
            if nested.contains('&') {
                expanded_selectors.push(nested.replace('&', parent));
            } else {
                expanded_selectors.push(format!("{parent} {nested}"));
            }
        }
    }

    if expanded_selectors.is_empty() {
        None
    } else {
        Some(expanded_selectors.join(", "))
    }
}
