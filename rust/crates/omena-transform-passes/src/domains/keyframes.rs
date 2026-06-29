use omena_parser::StyleDialect;
use omena_syntax::SyntaxKind;
use omena_transform_cst::TransformIrV0;

use crate::runtime::lex_cache::lex_cached as lex;

use crate::{
    domains::{
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
            TransformIrSourceReplacementV0, apply_ir_source_replacements,
            apply_ir_source_replacements_to_ir,
        },
        rules::collect_declaration_ordinary_rule_slices,
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
    let (replacements, removals) = collect_tree_shake_css_keyframe_replacements(
        source,
        dialect,
        reachable_keyframe_names,
        reachable_class_names,
    );
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
    let (replacements, removals) = collect_tree_shake_css_keyframe_replacements(
        source,
        dialect,
        reachable_keyframe_names,
        reachable_class_names,
    );
    let (output, _) = apply_ir_source_replacements(
        source,
        dialect,
        "omena-transform-passes.tree-shake-keyframes",
        "tree-shake-keyframes",
        replacements.as_slice(),
    )?;
    Ok((output, removals))
}

pub(crate) fn tree_shake_css_keyframes_with_ir_transaction_on_ir(
    ir: &mut TransformIrV0,
    dialect: StyleDialect,
    reachable_keyframe_names: &[String],
    reachable_class_names: &[String],
) -> Result<(String, Vec<TransformSemanticRemovalCandidate>), TransformIrSourceReplacementErrorV0> {
    let source = ir.source_text().to_string();
    let (replacements, removals) = collect_tree_shake_css_keyframe_replacements(
        source.as_str(),
        dialect,
        reachable_keyframe_names,
        reachable_class_names,
    );
    let (output, _) = apply_ir_source_replacements_to_ir(
        ir,
        dialect,
        "tree-shake-keyframes",
        replacements.as_slice(),
    )?;
    Ok((output, removals))
}

fn collect_tree_shake_css_keyframe_replacements(
    source: &str,
    dialect: StyleDialect,
    reachable_keyframe_names: &[String],
    reachable_class_names: &[String],
) -> (
    Vec<TransformIrSourceReplacementV0>,
    Vec<TransformSemanticRemovalCandidate>,
) {
    let lexed = lex(source, dialect);
    let tokens = lexed.tokens();
    let keyframes = collect_keyframes_rules(tokens);
    if keyframes.is_empty() {
        return (Vec::new(), Vec::new());
    }

    let Some(mut referenced_names) =
        collect_referenced_keyframe_names(source, tokens, reachable_class_names)
    else {
        return (Vec::new(), Vec::new());
    };
    for name in reachable_keyframe_names {
        push_unique_string(&mut referenced_names, name.clone());
    }

    let removals = keyframes
        .iter()
        .filter(|keyframe| !keyframe_name_is_reachable(&keyframe.name, &referenced_names))
        .map(|keyframe| TransformSemanticRemovalCandidate {
            symbol_kind: "keyframes",
            name: keyframe.name.clone(),
            source_span_start: keyframe.start,
            source_span_end: keyframe.end,
            reason: "keyframes name was absent from animation references and the closed-style-world reachable keyframe set",
        })
        .collect::<Vec<_>>();
    let replacements = removals
        .iter()
        .map(|removal| TransformIrSourceReplacementV0 {
            source_span_start: removal.source_span_start,
            source_span_end: removal.source_span_end,
            replacement: String::new(),
            kind: TransformIrReplacementKindV0::AtRule,
        })
        .collect::<Vec<_>>();
    (replacements, removals)
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
