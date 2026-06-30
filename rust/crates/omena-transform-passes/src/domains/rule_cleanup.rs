use omena_parser::{LexedToken, StyleDialect};
use omena_syntax::SyntaxKind;
use omena_transform_cst::{IrNodeIdV0, IrNodeKindV0, IrNodeV0, TransformIrV0};

use crate::runtime::lex_cache::lex_cached as lex;

use crate::domains::keyframes::is_keyframes_at_keyword;
use crate::helpers::{
    declarations::collect_simple_declarations_in_block,
    ir_transaction::{
        TransformIrReplacementKindV0, TransformIrSourceReplacementErrorV0,
        TransformIrSourceReplacementV0, delete_ir_nodes_in_ir,
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
    _dialect: StyleDialect,
) -> Result<(String, usize), TransformIrSourceReplacementErrorV0> {
    let declaration_replacements =
        collect_overridden_same_property_declaration_replacements_from_ir(ir);
    let declaration_node_ids =
        rule_dedup_deletion_node_ids(ir, declaration_replacements.as_slice())?;
    let (_, declaration_count) =
        delete_ir_nodes_in_ir(ir, "rule-deduplication", declaration_node_ids.as_slice())?;
    let rule_replacements = collect_duplicate_ordinary_rule_replacements_from_ir(ir);
    let rule_node_ids = rule_dedup_deletion_node_ids(ir, rule_replacements.as_slice())?;
    let (output, rule_count) =
        delete_ir_nodes_in_ir(ir, "rule-deduplication", rule_node_ids.as_slice())?;
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

fn collect_overridden_same_property_declaration_replacements_from_ir(
    ir: &TransformIrV0,
) -> Vec<TransformIrSourceReplacementV0> {
    let mut replacements = Vec::new();

    for rule in collect_declaration_ordinary_rule_slices_from_ir(ir) {
        let selector = rule.selector.trim();
        if selector.eq_ignore_ascii_case(":export") || selector.starts_with(":import") {
            continue;
        }
        let Some(rule_node) = ir.nodes.iter().find(|node| {
            !node.deleted
                && node.kind == IrNodeKindV0::StyleRule
                && node.source_span_start == rule.start
                && node.source_span_end == rule.end
        }) else {
            continue;
        };
        let declarations = collect_simple_declarations_from_ir(ir, rule_node);
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

fn collect_duplicate_ordinary_rule_replacements_from_ir(
    ir: &TransformIrV0,
) -> Vec<TransformIrSourceReplacementV0> {
    let rules = collect_declaration_ordinary_rule_slices_from_ir(ir);
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

#[derive(Debug, Clone, PartialEq, Eq)]
struct RuleDedupDeclarationV0 {
    property: String,
    value: String,
    important: bool,
    start: usize,
    end: usize,
}

fn collect_declaration_ordinary_rule_slices_from_ir(ir: &TransformIrV0) -> Vec<SimpleRuleSlice> {
    let mut rules = ir
        .nodes
        .iter()
        .filter(|node| !node.deleted && node.kind == IrNodeKindV0::StyleRule)
        .filter_map(|node| simple_rule_slice_from_ir(ir, node))
        .collect::<Vec<_>>();
    rules.sort_by_key(|rule| (rule.start, rule.end));
    rules
}

fn simple_rule_slice_from_ir(ir: &TransformIrV0, node: &IrNodeV0) -> Option<SimpleRuleSlice> {
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
    let block_start = node.source_span_start.checked_add(open)?;
    let block_end = node.source_span_start.checked_add(close)?;
    let (context_start, context_end) = rule_context_from_ir(ir, node);
    Some(SimpleRuleSlice {
        selector,
        block,
        start: node.source_span_start,
        end: node.source_span_end,
        block_start,
        block_end,
        context_start,
        context_end,
    })
}

fn rule_context_from_ir(ir: &TransformIrV0, node: &IrNodeV0) -> (usize, usize) {
    let Some(parent) = node
        .parent
        .and_then(|parent_id| ir.nodes.get(parent_id.index()))
    else {
        return (0, ir.source_text().len());
    };
    let Some(source) = ir
        .source_text()
        .get(parent.source_span_start..parent.source_span_end)
    else {
        return (0, ir.source_text().len());
    };
    let Some(open) = source.find('{') else {
        return (0, ir.source_text().len());
    };
    let Some(close) = source.rfind('}') else {
        return (0, ir.source_text().len());
    };
    (
        parent.source_span_start.saturating_add(open),
        parent.source_span_start.saturating_add(close + 1),
    )
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

fn collect_simple_declarations_from_ir(
    ir: &TransformIrV0,
    rule_node: &IrNodeV0,
) -> Vec<RuleDedupDeclarationV0> {
    let mut declarations = rule_node
        .children
        .iter()
        .filter_map(|child_id| ir.nodes.get(child_id.index()))
        .filter(|child| !child.deleted && child.kind == IrNodeKindV0::Declaration)
        .filter_map(|child| simple_declaration_from_ir(ir, child))
        .collect::<Vec<_>>();
    declarations.sort_by_key(|declaration| declaration.start);
    declarations
}

fn simple_declaration_from_ir(
    ir: &TransformIrV0,
    node: &IrNodeV0,
) -> Option<RuleDedupDeclarationV0> {
    let source = ir
        .source_text()
        .get(node.source_span_start..node.source_span_end)?
        .trim()
        .trim_end_matches(';')
        .trim();
    if source.is_empty() || block_contains_nested_or_comment(source) {
        return None;
    }
    let colon = declaration_colon_index(source)?;
    let property = source.get(..colon)?.trim();
    let value = source.get(colon + 1..)?.trim();
    if property.is_empty() || value.is_empty() {
        return None;
    }
    let property = if property.starts_with("--") {
        property.to_string()
    } else {
        property.to_ascii_lowercase()
    };
    Some(RuleDedupDeclarationV0 {
        property,
        value: value.to_string(),
        important: declaration_value_is_important(value),
        start: node.source_span_start,
        end: node.source_span_end,
    })
}

fn declaration_value_is_important(value: &str) -> bool {
    let bytes = value.as_bytes();
    let mut index = 0usize;
    while index < bytes.len() {
        if bytes[index] == b'!' {
            let rest = value.get(index + 1..).unwrap_or_default().trim_start();
            return rest
                .get(.."important".len())
                .is_some_and(|candidate| candidate.eq_ignore_ascii_case("important"));
        }
        index += 1;
    }
    false
}

fn declaration_colon_index(source: &str) -> Option<usize> {
    let bytes = source.as_bytes();
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

fn rule_dedup_deletion_node_ids(
    ir: &TransformIrV0,
    replacements: &[TransformIrSourceReplacementV0],
) -> Result<Vec<IrNodeIdV0>, TransformIrSourceReplacementErrorV0> {
    replacements
        .iter()
        .map(|replacement| rule_dedup_deletion_node_id(ir, replacement))
        .collect()
}

fn rule_dedup_deletion_node_id(
    ir: &TransformIrV0,
    replacement: &TransformIrSourceReplacementV0,
) -> Result<IrNodeIdV0, TransformIrSourceReplacementErrorV0> {
    let Some(expected_kind) = rule_dedup_deletion_node_kind(replacement.kind) else {
        return Err(TransformIrSourceReplacementErrorV0::MissingNode {
            source_span_start: replacement.source_span_start,
            source_span_end: replacement.source_span_end,
            kind: replacement.kind,
            candidate_spans: Vec::new(),
        });
    };
    ir.nodes
        .iter()
        .find(|node| {
            !node.deleted
                && node.kind == expected_kind
                && node.source_span_start == replacement.source_span_start
                && node.source_span_end == replacement.source_span_end
        })
        .map(|node| node.node_id)
        .ok_or_else(|| TransformIrSourceReplacementErrorV0::MissingNode {
            source_span_start: replacement.source_span_start,
            source_span_end: replacement.source_span_end,
            kind: replacement.kind,
            candidate_spans: ir
                .nodes
                .iter()
                .filter(|node| !node.deleted && node.kind == expected_kind)
                .map(|node| (node.source_span_start, node.source_span_end))
                .collect(),
        })
}

const fn rule_dedup_deletion_node_kind(kind: TransformIrReplacementKindV0) -> Option<IrNodeKindV0> {
    match kind {
        TransformIrReplacementKindV0::Declaration => Some(IrNodeKindV0::Declaration),
        TransformIrReplacementKindV0::StyleRule => Some(IrNodeKindV0::StyleRule),
        TransformIrReplacementKindV0::AtRule
        | TransformIrReplacementKindV0::Selector
        | TransformIrReplacementKindV0::CustomPropertyDeclaration
        | TransformIrReplacementKindV0::CustomPropertyReference
        | TransformIrReplacementKindV0::CssModuleValueDefinition
        | TransformIrReplacementKindV0::CssModuleValueImportSource
        | TransformIrReplacementKindV0::CssModuleComposesTarget
        | TransformIrReplacementKindV0::IcssExportName => None,
    }
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
    _dialect: StyleDialect,
) -> Result<(String, usize), TransformIrSourceReplacementErrorV0> {
    let mut mutation_count = 0;

    loop {
        let node_ids = collect_empty_rule_node_ids_from_ir(ir);
        let (next_output, removed_count) =
            delete_ir_nodes_in_ir(ir, "empty-rule-removal", node_ids.as_slice())?;
        if removed_count == 0 {
            return Ok((next_output, mutation_count));
        }
        mutation_count += removed_count;
    }
}

fn collect_empty_rule_node_ids_from_ir(ir: &TransformIrV0) -> Vec<IrNodeIdV0> {
    let source = ir.source_text();
    let mut candidates = ir
        .nodes
        .iter()
        .filter(|node| {
            !node.deleted
                && matches!(node.kind, IrNodeKindV0::StyleRule | IrNodeKindV0::AtRule)
                && node.source_span_start < node.source_span_end
        })
        .filter_map(|node| {
            let slice = source.get(node.source_span_start..node.source_span_end)?;
            let (prelude, body) = rule_prelude_and_body(slice)?;
            match node.kind {
                IrNodeKindV0::StyleRule
                    if !has_keyframes_ancestor(ir, node.parent) && rule_body_is_empty(body) =>
                {
                    Some((node.source_span_start, node.source_span_end, node.node_id))
                }
                IrNodeKindV0::AtRule
                    if rule_body_is_empty(body)
                        && first_significant_at_keyword(prelude)
                            .is_some_and(is_empty_removable_group_at_keyword) =>
                {
                    Some((node.source_span_start, node.source_span_end, node.node_id))
                }
                _ => None,
            }
        })
        .collect::<Vec<_>>();
    candidates.sort_by_key(|(start, end, _)| (*start, std::cmp::Reverse(*end)));

    let mut retained = Vec::new();
    let mut cursor = 0usize;
    for (start, end, node_id) in candidates {
        if start >= cursor {
            cursor = end;
            retained.push(node_id);
        }
    }

    retained
}

fn has_keyframes_ancestor(
    ir: &TransformIrV0,
    mut parent: Option<omena_transform_cst::IrNodeIdV0>,
) -> bool {
    while let Some(parent_id) = parent {
        let Some(node) = ir.nodes.get(parent_id.index()) else {
            return false;
        };
        if node.kind == IrNodeKindV0::AtRule
            && source_slice(ir, node.source_span_start, node.source_span_end)
                .and_then(|slice| rule_prelude_and_body(slice).map(|(prelude, _)| prelude))
                .and_then(first_significant_at_keyword)
                .is_some_and(is_keyframes_at_keyword)
        {
            return true;
        }
        parent = node.parent;
    }
    false
}

fn source_slice(
    ir: &TransformIrV0,
    source_span_start: usize,
    source_span_end: usize,
) -> Option<&str> {
    ir.source_text().get(source_span_start..source_span_end)
}

fn rule_prelude_and_body(rule_source: &str) -> Option<(&str, &str)> {
    let open = rule_source.find('{')?;
    let close = rule_source.rfind('}')?;
    if open >= close {
        return None;
    }
    Some((rule_source.get(..open)?, rule_source.get(open + 1..close)?))
}

fn rule_body_is_empty(body: &str) -> bool {
    body.chars().all(char::is_whitespace)
}

fn first_significant_at_keyword(prelude: &str) -> Option<&str> {
    let mut rest = prelude;
    loop {
        rest = rest.trim_start();
        if let Some(after_comment) = rest.strip_prefix("/*") {
            let end = after_comment.find("*/")?;
            rest = after_comment.get(end + 2..)?;
            continue;
        }
        if let Some(after_line_comment) = rest.strip_prefix("//") {
            rest = after_line_comment
                .find('\n')
                .and_then(|newline| after_line_comment.get(newline + 1..))?;
            continue;
        }
        break;
    }
    let rest = rest.strip_prefix('@')?;
    let keyword_end = rest
        .find(|ch: char| ch.is_whitespace() || matches!(ch, '{' | '(' | ';'))
        .unwrap_or(rest.len());
    prelude.get(prelude.len() - rest.len() - 1..prelude.len() - rest.len() + keyword_end)
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
