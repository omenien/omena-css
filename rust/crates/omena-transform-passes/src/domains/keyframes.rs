use omena_parser::StyleDialect;
use omena_syntax::SyntaxKind;
use omena_transform_cst::{IrNodeIdV0, IrNodeKindV0, IrNodeV0, TransformIrV0};

use crate::runtime::lex_cache::lex_cached as lex;

use crate::{
    domains::{
        css_module_global::{CssModuleScopeBlock, CssModuleScopeBlockKind},
        number::parse_numeric_value_with_unit,
        reachability::rule_slice_matches_reachable_class_context,
    },
    helpers::{
        blocks::rule_block_token_indexes,
        collections::push_unique_string,
        declarations::collect_simple_declarations_in_block,
        identifiers::{css_identifier_escape_sequence_end, css_identifier_names_match},
        ir_transaction::{
            TransformIrReplacementKindV0, TransformIrSourceReplacementErrorV0,
            TransformIrSourceReplacementV0, delete_ir_nodes_in_ir,
        },
        rules::{SimpleRuleSlice, collect_declaration_ordinary_rule_slices},
        source_rewrite::remove_source_ranges,
        tokens::{matching_right_brace_index, skip_whitespace_tokens, token_end, token_start},
        values::{
            split_top_level_value_arguments, split_top_level_whitespace_value_components,
            static_css_string_value,
        },
    },
    model::TransformSemanticRemovalCandidate,
};

use super::css_module_global::collect_css_module_scope_blocks;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct KeyframesRuleSlice {
    pub(crate) name: String,
    pub(crate) start: usize,
    pub(crate) end: usize,
}

pub(crate) fn tree_shake_css_keyframes_with_lexer(
    source: &str,
    dialect: StyleDialect,
    reachable_keyframe_names: &[String],
    reachable_class_names: &[String],
) -> (String, Vec<TransformSemanticRemovalCandidate>) {
    let removals = collect_tree_shake_css_keyframe_removals(
        source,
        dialect,
        reachable_keyframe_names,
        reachable_class_names,
    );
    let replacements = keyframe_removal_replacements(removals.as_slice());
    let ranges = replacements
        .iter()
        .map(|replacement| (replacement.source_span_start, replacement.source_span_end))
        .collect::<Vec<_>>();
    let (output, _) = remove_source_ranges(source, &ranges);
    (output, removals)
}

pub(crate) fn tree_shake_css_keyframes_with_ir_transaction(
    source: &str,
    dialect: StyleDialect,
    reachable_keyframe_names: &[String],
    reachable_class_names: &[String],
) -> Result<(String, Vec<TransformSemanticRemovalCandidate>), TransformIrSourceReplacementErrorV0> {
    let mut ir = omena_transform_cst::lower_transform_ir_from_source(
        source,
        dialect,
        "omena-transform-passes.tree-shake-keyframes",
    );
    tree_shake_css_keyframes_with_ir_transaction_on_ir(
        &mut ir,
        dialect,
        reachable_keyframe_names,
        reachable_class_names,
    )
}

pub(crate) fn tree_shake_css_keyframes_with_ir_transaction_on_ir(
    ir: &mut TransformIrV0,
    _dialect: StyleDialect,
    reachable_keyframe_names: &[String],
    reachable_class_names: &[String],
) -> Result<(String, Vec<TransformSemanticRemovalCandidate>), TransformIrSourceReplacementErrorV0> {
    let removals = collect_tree_shake_css_keyframe_removals_from_ir(
        ir,
        reachable_keyframe_names,
        reachable_class_names,
    );
    let node_ids = keyframe_removal_node_ids(ir, removals.as_slice())?;
    let (output, _) = delete_ir_nodes_in_ir(ir, "tree-shake-keyframes", node_ids.as_slice())?;
    Ok((output, removals))
}

fn collect_tree_shake_css_keyframe_removals(
    source: &str,
    dialect: StyleDialect,
    reachable_keyframe_names: &[String],
    reachable_class_names: &[String],
) -> Vec<TransformSemanticRemovalCandidate> {
    let lexed = lex(source, dialect);
    let tokens = lexed.tokens();
    let keyframes = collect_keyframes_rules(tokens);
    if keyframes.is_empty() {
        return Vec::new();
    }

    let Some(mut referenced_names) =
        collect_referenced_keyframe_names(source, tokens, reachable_class_names)
    else {
        return Vec::new();
    };
    for name in reachable_keyframe_names {
        push_unique_string(&mut referenced_names, name.clone());
    }

    keyframes
        .iter()
        .filter(|keyframe| !keyframe_name_is_reachable(&keyframe.name, &referenced_names))
        .map(|keyframe| TransformSemanticRemovalCandidate {
            symbol_kind: "keyframes",
            name: keyframe.name.clone(),
            source_span_start: keyframe.start,
            source_span_end: keyframe.end,
            reason: "keyframes name was absent from animation references and the closed-style-world reachable keyframe set",
        })
        .collect::<Vec<_>>()
}

fn collect_tree_shake_css_keyframe_removals_from_ir(
    ir: &TransformIrV0,
    reachable_keyframe_names: &[String],
    reachable_class_names: &[String],
) -> Vec<TransformSemanticRemovalCandidate> {
    let keyframes = collect_keyframes_rules_from_ir(ir);
    if keyframes.is_empty() {
        return Vec::new();
    }

    let Some(mut referenced_names) =
        collect_referenced_keyframe_names_from_ir(ir, reachable_class_names)
    else {
        return Vec::new();
    };
    for name in reachable_keyframe_names {
        push_unique_string(&mut referenced_names, name.clone());
    }

    keyframes
        .iter()
        .filter(|keyframe| !keyframe_name_is_reachable(&keyframe.name, &referenced_names))
        .map(|keyframe| TransformSemanticRemovalCandidate {
            symbol_kind: "keyframes",
            name: keyframe.name.clone(),
            source_span_start: keyframe.start,
            source_span_end: keyframe.end,
            reason: "keyframes name was absent from animation references and the closed-style-world reachable keyframe set",
        })
        .collect::<Vec<_>>()
}

fn keyframe_removal_replacements(
    removals: &[TransformSemanticRemovalCandidate],
) -> Vec<TransformIrSourceReplacementV0> {
    removals
        .iter()
        .map(|removal| TransformIrSourceReplacementV0 {
            source_span_start: removal.source_span_start,
            source_span_end: removal.source_span_end,
            replacement: String::new(),
            kind: TransformIrReplacementKindV0::AtRule,
        })
        .collect::<Vec<_>>()
}

fn keyframe_removal_node_ids(
    ir: &TransformIrV0,
    removals: &[TransformSemanticRemovalCandidate],
) -> Result<Vec<IrNodeIdV0>, TransformIrSourceReplacementErrorV0> {
    removals
        .iter()
        .map(|removal| keyframe_removal_node_id(ir, removal))
        .collect()
}

fn keyframe_removal_node_id(
    ir: &TransformIrV0,
    removal: &TransformSemanticRemovalCandidate,
) -> Result<IrNodeIdV0, TransformIrSourceReplacementErrorV0> {
    ir.nodes
        .iter()
        .find(|node| {
            !node.deleted
                && node.kind == IrNodeKindV0::AtRule
                && node.source_span_start == removal.source_span_start
                && node.source_span_end == removal.source_span_end
        })
        .map(|node| node.node_id)
        .ok_or_else(|| TransformIrSourceReplacementErrorV0::MissingNode {
            source_span_start: removal.source_span_start,
            source_span_end: removal.source_span_end,
            kind: TransformIrReplacementKindV0::AtRule,
            candidate_spans: ir
                .nodes
                .iter()
                .filter(|node| !node.deleted && node.kind == IrNodeKindV0::AtRule)
                .map(|node| (node.source_span_start, node.source_span_end))
                .collect(),
        })
}

pub(crate) fn collect_keyframes_rules(
    tokens: &[omena_parser::LexedToken],
) -> Vec<KeyframesRuleSlice> {
    let mut rules = Vec::new();
    let mut index = 0;

    while index < tokens.len() {
        if tokens[index].kind == SyntaxKind::AtKeyword
            && is_keyframes_at_keyword(&tokens[index].text)
            && let Some((rule, next_index)) = parse_keyframes_rule(tokens, index)
        {
            rules.push(rule);
            index = next_index;
            continue;
        }
        index += 1;
    }

    rules
}

pub(crate) fn collect_keyframes_rules_from_ir(ir: &TransformIrV0) -> Vec<KeyframesRuleSlice> {
    let mut rules = ir
        .nodes
        .iter()
        .filter(|node| !node.deleted && node.kind == IrNodeKindV0::AtRule)
        .filter_map(|node| keyframes_rule_from_ir(ir, node))
        .collect::<Vec<_>>();
    rules.sort_by_key(|rule| (rule.start, rule.end));
    rules
}

fn keyframes_rule_from_ir(ir: &TransformIrV0, node: &IrNodeV0) -> Option<KeyframesRuleSlice> {
    let source = ir.source_text();
    let node_source = source.get(node.source_span_start..node.source_span_end)?;
    let leading_offset = node_source
        .len()
        .saturating_sub(node_source.trim_start().len());
    let at_keyword_start = node.source_span_start.checked_add(leading_offset)?;
    let at_rule_source = source.get(at_keyword_start..node.source_span_end)?;
    let keyword_len = at_rule_source
        .find(|ch: char| ch.is_whitespace() || matches!(ch, '{' | '(' | ';'))
        .unwrap_or(at_rule_source.len());
    let keyword_end = at_keyword_start.checked_add(keyword_len)?;
    if !is_keyframes_at_keyword(source.get(at_keyword_start..keyword_end)?) {
        return None;
    }
    let block_start = find_keyframes_block_start(source, keyword_end, node.source_span_end)?;
    let name = keyframes_name_from_ir_prelude(source.get(keyword_end..block_start)?)?;
    Some(KeyframesRuleSlice {
        name,
        start: at_keyword_start,
        end: node.source_span_end,
    })
}

fn find_keyframes_block_start(source: &str, start: usize, end: usize) -> Option<usize> {
    let bytes = source.as_bytes();
    let mut index = start;
    let mut quote = None;
    let mut escaped = false;
    let mut in_comment = false;
    while index < end {
        let byte = *bytes.get(index)?;
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
        if byte == b'{' {
            return Some(index);
        }
        if byte == b';' {
            return None;
        }
        index += 1;
    }
    None
}

fn keyframes_name_from_ir_prelude(prelude: &str) -> Option<String> {
    let name = prelude.trim();
    if name.is_empty() {
        return None;
    }
    static_css_string_value(name).or_else(|| Some(name.to_string()))
}

pub(crate) fn is_keyframes_at_keyword(text: &str) -> bool {
    matches!(
        text.to_ascii_lowercase().as_str(),
        "@keyframes" | "@-webkit-keyframes"
    )
}

fn parse_keyframes_rule(
    tokens: &[omena_parser::LexedToken],
    at_keyframes_index: usize,
) -> Option<(KeyframesRuleSlice, usize)> {
    let name_index = skip_whitespace_tokens(tokens, at_keyframes_index + 1, tokens.len());
    let name_token = tokens.get(name_index)?;
    let name = static_keyframe_name_from_rule_name_token(name_token)?;
    let mut index = name_index + 1;
    while index < tokens.len() {
        match tokens[index].kind {
            SyntaxKind::Semicolon => return None,
            SyntaxKind::LeftBrace => {
                let close_index = matching_right_brace_index(tokens, index)?;
                return Some((
                    KeyframesRuleSlice {
                        name,
                        start: token_start(&tokens[at_keyframes_index]),
                        end: token_end(&tokens[close_index]),
                    },
                    close_index + 1,
                ));
            }
            _ => index += 1,
        }
    }

    None
}

fn static_keyframe_name_from_rule_name_token(token: &omena_parser::LexedToken) -> Option<String> {
    match token.kind {
        SyntaxKind::Ident => Some(token.text.clone()),
        SyntaxKind::String => static_css_string_value(&token.text),
        _ => None,
    }
}

pub(crate) fn collect_referenced_keyframe_names(
    source: &str,
    tokens: &[omena_parser::LexedToken],
    reachable_class_names: &[String],
) -> Option<Vec<String>> {
    let mut names = Vec::new();
    let scope_blocks = collect_css_module_scope_blocks(source, tokens);
    for rule in collect_declaration_ordinary_rule_slices(source, tokens) {
        if !rule_slice_matches_reachable_class_context(&rule, &scope_blocks, reachable_class_names)
        {
            continue;
        };
        let Some((block_start_index, block_end_index)) =
            rule_block_token_indexes(tokens, rule.block_start, rule.block_end)
        else {
            continue;
        };
        for declaration in
            collect_simple_declarations_in_block(tokens, block_start_index, block_end_index)
        {
            match declaration.property.as_str() {
                "animation-name" => {
                    if declaration.value.contains("var(") {
                        return None;
                    }
                    for name in split_top_level_value_arguments(&declaration.value)? {
                        if let Some(candidate) = static_animation_name_candidate(&name)
                            && (candidate.quoted || !candidate.name.eq_ignore_ascii_case("none"))
                        {
                            push_unique_string(&mut names, candidate.name);
                        }
                    }
                }
                "animation" => {
                    if declaration.value.contains("var(") {
                        return None;
                    }
                    for name in extract_animation_shorthand_name_candidates(&declaration.value)? {
                        push_unique_string(&mut names, name);
                    }
                }
                _ => {}
            }
        }
    }

    Some(names)
}

pub(crate) fn collect_referenced_keyframe_names_from_ir(
    ir: &TransformIrV0,
    reachable_class_names: &[String],
) -> Option<Vec<String>> {
    let mut names = Vec::new();
    let scope_blocks = collect_css_module_scope_blocks_from_ir(ir);
    for rule in collect_declaration_ordinary_rule_slices_from_ir(ir) {
        if !rule_slice_matches_reachable_class_context(&rule, &scope_blocks, reachable_class_names)
        {
            continue;
        }
        for declaration in collect_simple_declarations_from_ir(ir, &rule) {
            match declaration.property.as_str() {
                "animation-name" => {
                    if declaration.value.contains("var(") {
                        return None;
                    }
                    for name in split_top_level_value_arguments(&declaration.value)? {
                        if let Some(candidate) = static_animation_name_candidate(&name)
                            && (candidate.quoted || !candidate.name.eq_ignore_ascii_case("none"))
                        {
                            push_unique_string(&mut names, candidate.name);
                        }
                    }
                }
                "animation" => {
                    if declaration.value.contains("var(") {
                        return None;
                    }
                    for name in extract_animation_shorthand_name_candidates(&declaration.value)? {
                        push_unique_string(&mut names, name);
                    }
                }
                _ => {}
            }
        }
    }

    Some(names)
}

fn collect_css_module_scope_blocks_from_ir(ir: &TransformIrV0) -> Vec<CssModuleScopeBlock> {
    let mut blocks = ir
        .nodes
        .iter()
        .filter(|node| !node.deleted && node.kind == IrNodeKindV0::StyleRule)
        .filter_map(|node| css_module_scope_block_from_ir(ir, node))
        .collect::<Vec<_>>();
    blocks.sort_by_key(|block| (block.start, block.end));
    blocks
}

fn css_module_scope_block_from_ir(
    ir: &TransformIrV0,
    node: &IrNodeV0,
) -> Option<CssModuleScopeBlock> {
    let selector = style_rule_selector_from_ir(ir, node)?;
    let kind = if selector.eq_ignore_ascii_case(":local") {
        CssModuleScopeBlockKind::Local
    } else if selector.eq_ignore_ascii_case(":global") {
        CssModuleScopeBlockKind::Global
    } else {
        return None;
    };
    let (body_start, body_end) = style_rule_body_bounds_from_ir(ir.source_text(), node)?;
    Some(CssModuleScopeBlock {
        start: node.source_span_start,
        end: node.source_span_end,
        body_start,
        body_end,
        kind,
    })
}

fn collect_declaration_ordinary_rule_slices_from_ir(ir: &TransformIrV0) -> Vec<SimpleRuleSlice> {
    let mut rules = ir
        .nodes
        .iter()
        .filter(|node| !node.deleted && node.kind == IrNodeKindV0::StyleRule)
        .filter_map(|node| declaration_ordinary_rule_slice_from_ir(ir, node))
        .collect::<Vec<_>>();
    rules.sort_by_key(|rule| (rule.start, rule.end));
    rules
}

fn declaration_ordinary_rule_slice_from_ir(
    ir: &TransformIrV0,
    node: &IrNodeV0,
) -> Option<SimpleRuleSlice> {
    if node.children.iter().any(|child_id| {
        ir.nodes.get(child_id.index()).is_some_and(|child| {
            !child.deleted && matches!(child.kind, IrNodeKindV0::StyleRule | IrNodeKindV0::AtRule)
        })
    }) {
        return None;
    }
    let source = ir.source_text();
    let selector = style_rule_selector_from_ir(ir, node)?.trim().to_string();
    let (body_start, body_end) = style_rule_body_bounds_from_ir(source, node)?;
    let block = source.get(body_start..body_end)?.trim().to_string();
    if selector.is_empty() || block.is_empty() || source_text_contains_comment(&block) {
        return None;
    }
    let (context_start, context_end) = style_rule_context_from_ir(ir, node);
    Some(SimpleRuleSlice {
        selector,
        block,
        start: node.source_span_start,
        end: node.source_span_end,
        block_start: body_start.saturating_sub(1),
        block_end: body_end,
        context_start,
        context_end,
    })
}

fn style_rule_context_from_ir(ir: &TransformIrV0, node: &IrNodeV0) -> (usize, usize) {
    let Some(parent_id) = node.parent else {
        return (0, ir.source_text().len());
    };
    let Some(parent) = ir.nodes.get(parent_id.index()) else {
        return (0, ir.source_text().len());
    };
    let Some((body_start, body_end)) = style_rule_body_bounds_from_ir(ir.source_text(), parent)
    else {
        return (0, ir.source_text().len());
    };
    (body_start.saturating_sub(1), body_end.saturating_add(1))
}

struct KeyframeDeclarationIrViewV0 {
    source_span_start: usize,
    property: String,
    value: String,
}

fn collect_simple_declarations_from_ir(
    ir: &TransformIrV0,
    rule: &SimpleRuleSlice,
) -> Vec<KeyframeDeclarationIrViewV0> {
    let mut declarations = ir
        .nodes
        .iter()
        .filter(|node| {
            !node.deleted
                && node.kind == IrNodeKindV0::Declaration
                && node.source_span_start >= rule.block_start
                && node.source_span_end <= rule.block_end
        })
        .filter_map(|node| simple_declaration_from_ir(ir, node))
        .collect::<Vec<_>>();
    declarations.sort_by_key(|declaration| declaration.source_span_start);
    declarations
}

fn simple_declaration_from_ir(
    ir: &TransformIrV0,
    node: &IrNodeV0,
) -> Option<KeyframeDeclarationIrViewV0> {
    let source = ir
        .source_text()
        .get(node.source_span_start..node.source_span_end)?
        .trim()
        .trim_end_matches(';')
        .trim();
    if source.is_empty() || source_text_contains_comment(source) {
        return None;
    }
    let colon = source.find(':')?;
    let property = source.get(..colon)?.trim();
    let value = source.get(colon + 1..)?.trim();
    if property.is_empty() || value.is_empty() {
        return None;
    }
    Some(KeyframeDeclarationIrViewV0 {
        source_span_start: node.source_span_start,
        property: property.to_ascii_lowercase(),
        value: value.to_string(),
    })
}

fn style_rule_selector_from_ir<'source>(
    ir: &'source TransformIrV0,
    node: &IrNodeV0,
) -> Option<&'source str> {
    let source = ir.source_text();
    let rule_source = source.get(node.source_span_start..node.source_span_end)?;
    let open = rule_source.find('{')?;
    source
        .get(node.source_span_start..node.source_span_start + open)
        .map(str::trim)
}

fn style_rule_body_bounds_from_ir(source: &str, node: &IrNodeV0) -> Option<(usize, usize)> {
    let rule_source = source.get(node.source_span_start..node.source_span_end)?;
    let open = rule_source.find('{')?;
    let close = rule_source.rfind('}')?;
    if open >= close {
        return None;
    }
    Some((
        node.source_span_start.checked_add(open + 1)?,
        node.source_span_start.checked_add(close)?,
    ))
}

fn source_text_contains_comment(source: &str) -> bool {
    source.as_bytes().windows(2).any(|bytes| bytes == b"/*")
}

pub(crate) fn keyframe_name_is_reachable(name: &str, reachable_keyframe_names: &[String]) -> bool {
    reachable_keyframe_names
        .iter()
        .any(|reachable| css_identifier_names_match(reachable, name))
}

fn extract_animation_shorthand_name_candidates(value: &str) -> Option<Vec<String>> {
    let mut candidates = Vec::new();
    for branch in split_top_level_value_arguments(value)? {
        for part in split_top_level_whitespace_value_components(&branch)? {
            let candidate = part.trim();
            if let Some(candidate) = static_animation_name_candidate(candidate)
                && (candidate.quoted || !is_known_animation_shorthand_keyword(&candidate.name))
            {
                push_unique_string(&mut candidates, candidate.name);
            }
        }
    }
    Some(candidates)
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct StaticAnimationNameCandidate {
    name: String,
    quoted: bool,
}

fn static_animation_name_candidate(value: &str) -> Option<StaticAnimationNameCandidate> {
    let value = value.trim();
    if value.is_empty() {
        return None;
    }
    if let Some(name) = static_css_string_value(value) {
        return Some(StaticAnimationNameCandidate { name, quoted: true });
    }
    if value.contains(['(', ')', '"', '\'', '/'])
        || parse_numeric_value_with_unit(value).is_some()
        || !static_animation_custom_ident_value_is_safe(value)
    {
        return None;
    }
    Some(StaticAnimationNameCandidate {
        name: value.to_string(),
        quoted: false,
    })
}

fn static_animation_custom_ident_value_is_safe(value: &str) -> bool {
    let mut index = 0usize;
    while index < value.len() {
        let Some(ch) = value[index..].chars().next() else {
            return false;
        };
        if ch == '\\' {
            let Some(end) = css_identifier_escape_sequence_end(value, index) else {
                return false;
            };
            index = end;
            continue;
        }
        if ch.is_ascii_whitespace() || matches!(ch, ',' | ';' | '{' | '}' | '[' | ']') {
            return false;
        }
        index += ch.len_utf8();
    }
    true
}

fn is_known_animation_shorthand_keyword(value: &str) -> bool {
    matches!(
        value.to_ascii_lowercase().as_str(),
        "alternate"
            | "alternate-reverse"
            | "backwards"
            | "both"
            | "ease"
            | "ease-in"
            | "ease-in-out"
            | "ease-out"
            | "forwards"
            | "infinite"
            | "linear"
            | "none"
            | "normal"
            | "paused"
            | "reverse"
            | "running"
            | "step-end"
            | "step-start"
    )
}
