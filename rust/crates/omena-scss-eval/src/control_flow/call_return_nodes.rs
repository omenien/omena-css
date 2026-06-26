use cstree::syntax::SyntaxNode;
use omena_parser::LexedToken;
use omena_syntax::SyntaxKind;

use crate::abstract_css_value_kind;

use super::analysis_model::ScssCallReturnCandidate;
use super::blocks::scss_eval_stable_node_key;
use super::model::OmenaScssEvalCallReturnNodeV0;
use super::scanner_tokens::{matching_block_end_token_index, next_block_start_token_index};

pub(super) fn call_return_node_from_candidate(
    candidate: ScssCallReturnCandidate,
) -> OmenaScssEvalCallReturnNodeV0 {
    OmenaScssEvalCallReturnNodeV0 {
        node_key: scss_eval_stable_node_key(
            "scss-call-return",
            candidate.kind,
            candidate.source_span_start,
            candidate.source_span_end,
        ),
        kind: candidate.kind,
        symbol_kind: candidate.symbol_kind,
        role: candidate.role,
        name: candidate.name,
        namespace: candidate.namespace,
        parameter_names: candidate.parameter_names,
        parameter_values: candidate.parameter_values,
        local_binding_values: candidate.local_binding_values,
        argument_values: candidate.argument_values,
        return_value_kind: candidate.return_value.as_ref().map(abstract_css_value_kind),
        return_text: candidate.return_text,
        return_value: candidate.return_value,
        call_resolved_return_value: None,
        call_resolved_return_value_kind: None,
        body_has_control_flow: candidate.body_has_control_flow,
        body_has_loop_control_flow: candidate.body_has_loop_control_flow,
        return_inside_loop_control_flow: candidate.return_inside_loop_control_flow,
        return_loop_header_text: candidate.return_loop_header_text,
        return_loop_header_texts: candidate.return_loop_header_texts,
        return_loop_body_texts: candidate.return_loop_body_texts,
        return_condition_text: candidate.return_condition_text,
        return_negated_condition_texts: candidate.return_negated_condition_texts,
        source_span_start: candidate.source_span_start,
        source_span_end: candidate.source_span_end,
        containing_declaration_node_key: None,
    }
}

pub(super) fn stamp_containing_declarations(
    nodes: &mut [OmenaScssEvalCallReturnNodeV0],
    tokens: &[LexedToken],
) {
    let declaration_ranges = nodes
        .iter()
        .enumerate()
        .filter(|(_, node)| call_return_node_is_declaration(node))
        .map(|(index, node)| {
            let next_declaration_start = nodes
                .iter()
                .skip(index + 1)
                .find(|candidate| call_return_node_is_declaration(candidate))
                .map(|candidate| candidate.source_span_start)
                .unwrap_or(usize::MAX);
            let body_end = call_return_declaration_body_end(tokens, node)
                .unwrap_or(next_declaration_start)
                .min(next_declaration_start);
            (node.node_key.clone(), node.source_span_start, body_end)
        })
        .collect::<Vec<_>>();

    for node in nodes {
        if call_return_node_is_declaration(node) {
            continue;
        }
        node.containing_declaration_node_key = declaration_ranges
            .iter()
            .rev()
            .find(|(_, start, end)| {
                node.source_span_start >= *start && node.source_span_start < *end
            })
            .map(|(node_key, _, _)| node_key.clone());
    }
}

pub(super) fn stamp_containing_declarations_from_cst(
    nodes: &mut [OmenaScssEvalCallReturnNodeV0],
    root: &SyntaxNode<SyntaxKind>,
) {
    let declaration_ranges = nodes
        .iter()
        .enumerate()
        .filter(|(_, node)| call_return_node_is_declaration(node))
        .map(|(index, node)| {
            let next_declaration_start = nodes
                .iter()
                .skip(index + 1)
                .find(|candidate| call_return_node_is_declaration(candidate))
                .map(|candidate| candidate.source_span_start)
                .unwrap_or(usize::MAX);
            let body_end = cst_call_return_declaration_body_end(root, node)
                .unwrap_or(next_declaration_start)
                .min(next_declaration_start);
            (node.node_key.clone(), node.source_span_start, body_end)
        })
        .collect::<Vec<_>>();

    for node in nodes {
        if call_return_node_is_declaration(node) {
            continue;
        }
        node.containing_declaration_node_key = declaration_ranges
            .iter()
            .rev()
            .find(|(_, start, end)| {
                node.source_span_start >= *start && node.source_span_start < *end
            })
            .map(|(node_key, _, _)| node_key.clone());
    }
}

fn call_return_declaration_body_end(
    tokens: &[LexedToken],
    node: &OmenaScssEvalCallReturnNodeV0,
) -> Option<usize> {
    let at_rule_name = match node.kind {
        "mixinDeclaration" => "@mixin",
        "functionDeclaration" => "@function",
        _ => return None,
    };
    let declaration_name_start = node.source_span_start;
    let at_rule_index = tokens
        .iter()
        .enumerate()
        .rev()
        .find(|(_, token)| {
            let token_start: usize = token.range.start().into();
            token.kind == SyntaxKind::AtKeyword
                && token.text.eq_ignore_ascii_case(at_rule_name)
                && token_start <= declaration_name_start
        })
        .map(|(index, _)| index)?;
    let block_start_index = next_block_start_token_index(tokens, at_rule_index + 1)?;
    let block_end_index = matching_block_end_token_index(tokens, block_start_index)?;
    Some(tokens[block_end_index].range.end().into())
}

fn cst_call_return_declaration_body_end(
    root: &SyntaxNode<SyntaxKind>,
    node: &OmenaScssEvalCallReturnNodeV0,
) -> Option<usize> {
    let declaration_kind = match node.kind {
        "mixinDeclaration" => SyntaxKind::ScssMixinDeclaration,
        "functionDeclaration" => SyntaxKind::ScssFunctionDeclaration,
        _ => return None,
    };
    root.descendants()
        .filter(|candidate| candidate.kind() == declaration_kind)
        .find_map(|candidate| {
            let start = u32::from(candidate.text_range().start()) as usize;
            let end = u32::from(candidate.text_range().end()) as usize;
            (start <= node.source_span_start && node.source_span_start < end).then_some(end)
        })
}

pub(super) fn call_return_node_is_declaration(node: &OmenaScssEvalCallReturnNodeV0) -> bool {
    matches!(node.kind, "mixinDeclaration" | "functionDeclaration")
}

pub(super) fn call_return_node_is_call(node: &OmenaScssEvalCallReturnNodeV0) -> bool {
    matches!(node.kind, "mixinInclude" | "functionCall")
}
