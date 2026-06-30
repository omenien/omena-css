use omena_parser::StyleDialect;
use omena_syntax::SyntaxKind;
use omena_transform_cst::{IrNodeKindV0, IrNodeV0, TransformIrV0, lower_transform_ir_from_source};

use crate::runtime::lex_cache::lex_cached as lex;

use crate::helpers::{
    declarations::collect_simple_declarations_in_block,
    ir_transaction::{
        TransformIrReplacementKindV0, TransformIrSourceReplacementErrorV0,
        TransformIrSourceReplacementV0, replace_ir_nodes_in_ir,
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
    _dialect: StyleDialect,
) -> Result<(String, usize), TransformIrSourceReplacementErrorV0> {
    let replacements = collect_nesting_unwrap_replacements_from_ir(ir);
    replace_ir_nodes_in_ir(ir, "nesting-unwrap", replacements.as_slice())
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

fn collect_nesting_unwrap_replacements_from_ir(
    ir: &TransformIrV0,
) -> Vec<TransformIrSourceReplacementV0> {
    let mut style_nodes = ir
        .nodes
        .iter()
        .filter(|node| {
            !node.deleted
                && node.kind == IrNodeKindV0::StyleRule
                && !node_parent_is_style_rule(ir, node)
        })
        .collect::<Vec<_>>();
    style_nodes.sort_by_key(|node| (node.source_span_start, node.global_order));

    let mut replacements = Vec::new();
    let mut skip_until = 0usize;
    for node in style_nodes {
        if node.source_span_start < skip_until {
            continue;
        }
        let Some(replacement) = unwrap_simple_nested_rule_from_ir(ir, node) else {
            continue;
        };
        skip_until = node.source_span_end;
        replacements.push(TransformIrSourceReplacementV0 {
            source_span_start: node.source_span_start,
            source_span_end: node.source_span_end,
            replacement,
            kind: TransformIrReplacementKindV0::StyleRule,
        });
    }

    replacements
}

fn node_parent_is_style_rule(ir: &TransformIrV0, node: &IrNodeV0) -> bool {
    node.parent
        .and_then(|parent_id| ir.nodes.get(parent_id.index()))
        .is_some_and(|parent| !parent.deleted && parent.kind == IrNodeKindV0::StyleRule)
}

fn unwrap_simple_nested_rule_from_ir(ir: &TransformIrV0, node: &IrNodeV0) -> Option<String> {
    let (parent_selector, body) = rule_prelude_and_body(node_source(ir, node)?)?;
    let parent_selector = parent_selector.trim();
    if parent_selector.is_empty()
        || split_css_selector_list(parent_selector).is_none()
        || source_contains_css_comment(body)
    {
        return None;
    }

    let rule_texts = unwrap_nested_rule_body_from_ir(ir, parent_selector, node, true)?;
    Some(rule_texts.join(" "))
}

fn unwrap_nested_rule_body_from_ir(
    ir: &TransformIrV0,
    parent_selector: &str,
    node: &IrNodeV0,
    require_nested_rule: bool,
) -> Option<Vec<String>> {
    let declarations = direct_declaration_texts_from_ir(ir, node);
    let nested_rules = collect_direct_nested_rule_nodes_from_ir(ir, node)?;
    if require_nested_rule && nested_rules.is_empty() {
        return None;
    }

    let mut rule_texts = Vec::new();
    if !declarations.is_empty() {
        rule_texts.push(format!(
            "{parent_selector} {{ {} }}",
            declarations.join(" ")
        ));
    }

    for nested_rule in nested_rules {
        match nested_rule.kind {
            NestedRuleKind::Style => {
                let selector = expand_nested_selector(parent_selector, &nested_rule.selector)?;
                let nested_rule_texts =
                    unwrap_nested_rule_body_from_ir(ir, &selector, nested_rule.node, false)?;
                rule_texts.extend(nested_rule_texts);
            }
            NestedRuleKind::ConditionalGroup => {
                let nested_rule_texts =
                    unwrap_nested_rule_body_from_ir(ir, parent_selector, nested_rule.node, false)?;
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
struct NestedRuleNode<'a> {
    selector: String,
    node: &'a IrNodeV0,
    kind: NestedRuleKind,
}

fn collect_direct_nested_rule_nodes_from_ir<'a>(
    ir: &'a TransformIrV0,
    node: &IrNodeV0,
) -> Option<Vec<NestedRuleNode<'a>>> {
    let mut nested_rules = Vec::new();
    for child in direct_children_from_ir(ir, node) {
        match child.kind {
            IrNodeKindV0::StyleRule => {
                let selector = style_rule_selector_from_ir(ir, child)?.trim().to_string();
                if selector.is_empty() {
                    return None;
                }
                split_css_selector_list(&selector)?;
                if rule_body_from_ir(ir, child)?.trim().is_empty() {
                    return None;
                }
                nested_rules.push(NestedRuleNode {
                    selector,
                    node: child,
                    kind: NestedRuleKind::Style,
                });
            }
            IrNodeKindV0::AtRule => {
                let Some(prelude) = at_rule_prelude_from_ir(ir, child) else {
                    continue;
                };
                if rule_body_from_ir(ir, child)?.trim().is_empty() {
                    return None;
                }
                if let Some(nest_selector) = parse_nest_at_rule_selector(prelude) {
                    nested_rules.push(NestedRuleNode {
                        selector: nest_selector,
                        node: child,
                        kind: NestedRuleKind::Style,
                    });
                } else {
                    if !is_supported_nested_conditional_group_rule(prelude) {
                        return None;
                    }
                    nested_rules.push(NestedRuleNode {
                        selector: prelude.to_string(),
                        node: child,
                        kind: NestedRuleKind::ConditionalGroup,
                    });
                }
            }
            _ => {}
        }
    }

    Some(nested_rules)
}

fn direct_declaration_texts_from_ir(ir: &TransformIrV0, node: &IrNodeV0) -> Vec<String> {
    let Some((body_start, body_end)) = rule_body_bounds_from_ir(ir, node) else {
        return Vec::new();
    };
    let mut declarations = Vec::new();
    let mut cursor = body_start;

    for child in direct_children_from_ir(ir, node)
        .into_iter()
        .filter(|child| child.kind == IrNodeKindV0::StyleRule || child.kind == IrNodeKindV0::AtRule)
    {
        if child.source_span_start < body_start || child.source_span_end > body_end {
            continue;
        }
        if cursor < child.source_span_start
            && let Some(segment) = ir.source_text().get(cursor..child.source_span_start)
        {
            declarations.extend(declaration_texts_from_source_segment(segment));
        }
        cursor = cursor.max(child.source_span_end);
    }

    if cursor < body_end
        && let Some(segment) = ir.source_text().get(cursor..body_end)
    {
        declarations.extend(declaration_texts_from_source_segment(segment));
    }
    declarations
}

fn direct_children_from_ir<'a>(ir: &'a TransformIrV0, node: &IrNodeV0) -> Vec<&'a IrNodeV0> {
    let mut children = node
        .children
        .iter()
        .filter_map(|child_id| ir.nodes.get(child_id.index()))
        .filter(|child| !child.deleted)
        .collect::<Vec<_>>();
    children.sort_by_key(|child| (child.source_span_start, child.global_order));
    children
}

fn style_rule_selector_from_ir<'a>(ir: &'a TransformIrV0, node: &IrNodeV0) -> Option<&'a str> {
    let (prelude, _) = rule_prelude_and_body(node_source(ir, node)?)?;
    Some(prelude.trim())
}

fn at_rule_prelude_from_ir<'a>(ir: &'a TransformIrV0, node: &IrNodeV0) -> Option<&'a str> {
    let (prelude, _) = rule_prelude_and_body(node_source(ir, node)?)?;
    Some(prelude.trim())
}

fn rule_body_from_ir<'a>(ir: &'a TransformIrV0, node: &IrNodeV0) -> Option<&'a str> {
    let (_, body) = rule_prelude_and_body(node_source(ir, node)?)?;
    Some(body)
}

fn rule_body_bounds_from_ir(ir: &TransformIrV0, node: &IrNodeV0) -> Option<(usize, usize)> {
    let node_source = node_source(ir, node)?;
    let block_start = node_source.find('{')?;
    let block_end = node_source.rfind('}')?;
    if block_start >= block_end {
        return None;
    }
    Some((
        node.source_span_start.checked_add(block_start + 1)?,
        node.source_span_start.checked_add(block_end)?,
    ))
}

fn node_source<'a>(ir: &'a TransformIrV0, node: &IrNodeV0) -> Option<&'a str> {
    ir.source_text()
        .get(node.source_span_start..node.source_span_end)
}

fn rule_prelude_and_body(source: &str) -> Option<(&str, &str)> {
    let block_start = source.find('{')?;
    let block_end = source.rfind('}')?;
    if block_start >= block_end {
        return None;
    }
    Some((
        source.get(..block_start)?,
        source.get(block_start + 1..block_end)?,
    ))
}

fn source_contains_css_comment(source: &str) -> bool {
    let bytes = source.as_bytes();
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
        if byte == b'/' && bytes.get(index + 1) == Some(&b'*') {
            return true;
        }
        index += 1;
    }
    false
}

fn declaration_texts_from_source_segment(segment: &str) -> Vec<String> {
    split_declaration_segments(segment)
        .into_iter()
        .filter_map(format_declaration_text_from_segment)
        .collect()
}

fn split_declaration_segments(segment: &str) -> Vec<&str> {
    let bytes = segment.as_bytes();
    let mut segments = Vec::new();
    let mut start = 0usize;
    let mut index = 0usize;
    let mut quote = None;
    let mut escaped = false;
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;

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
        match byte {
            b'\'' | b'"' => quote = Some(byte),
            b'(' => paren_depth = paren_depth.saturating_add(1),
            b')' => paren_depth = paren_depth.saturating_sub(1),
            b'[' => bracket_depth = bracket_depth.saturating_add(1),
            b']' => bracket_depth = bracket_depth.saturating_sub(1),
            b';' if paren_depth == 0 && bracket_depth == 0 => {
                if let Some(declaration) = segment.get(start..index + 1) {
                    segments.push(declaration);
                }
                start = index + 1;
            }
            _ => {}
        }
        index += 1;
    }

    if start < segment.len()
        && let Some(declaration) = segment.get(start..)
    {
        segments.push(declaration);
    }
    segments
}

fn format_declaration_text_from_segment(segment: &str) -> Option<String> {
    let segment = segment.trim().trim_end_matches(';').trim();
    if segment.is_empty() || segment.contains(['{', '}']) {
        return None;
    }
    let colon = declaration_colon_index(segment)?;
    let property = segment.get(..colon)?.trim();
    let value = segment.get(colon + 1..)?.trim();
    if property.is_empty() || value.is_empty() {
        return None;
    }
    let property = if property.starts_with("--") {
        property.to_string()
    } else {
        property.to_ascii_lowercase()
    };
    Some(format!("{property}: {value};"))
}

fn declaration_colon_index(segment: &str) -> Option<usize> {
    let bytes = segment.as_bytes();
    let mut index = 0usize;
    let mut quote = None;
    let mut escaped = false;
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;

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
        match byte {
            b'\'' | b'"' => quote = Some(byte),
            b'(' => paren_depth = paren_depth.saturating_add(1),
            b')' => paren_depth = paren_depth.saturating_sub(1),
            b'[' => bracket_depth = bracket_depth.saturating_add(1),
            b']' => bracket_depth = bracket_depth.saturating_sub(1),
            b':' if paren_depth == 0 && bracket_depth == 0 => return Some(index),
            _ => {}
        }
        index += 1;
    }
    None
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
