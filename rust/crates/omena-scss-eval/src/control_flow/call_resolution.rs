use std::collections::BTreeMap;

use omena_parser::{StyleDialect, lex};
use omena_syntax::SyntaxKind;

use super::{
    SCSS_CALL_RETURN_RECURSION_LIMIT,
    model::{OmenaScssEvalCallReturnEdgeV0, OmenaScssEvalCallReturnNodeV0},
    tokens::next_non_trivia_token_index,
};

pub(super) fn static_scss_value_contains_function_call(value: &str, function_name: &str) -> bool {
    let canonical_function_name = canonical_scss_callable_name(function_name);
    let lexed = lex(value, StyleDialect::Scss);
    let tokens = lexed.tokens();
    for (index, token) in tokens.iter().enumerate() {
        if token.kind != SyntaxKind::Ident
            || canonical_scss_callable_name(token.text.as_str()) != canonical_function_name
        {
            continue;
        }
        let Some(left_paren_index) = next_non_trivia_token_index(tokens, index + 1) else {
            continue;
        };
        if tokens
            .get(left_paren_index)
            .is_some_and(|candidate| candidate.kind == SyntaxKind::LeftParen)
        {
            return true;
        }
    }
    false
}

pub(super) fn canonical_scss_callable_name(name: &str) -> String {
    name.trim().replace('_', "-")
}

pub(super) fn scss_visible_function_declaration_exists(
    name: &str,
    position: usize,
    nodes: &[OmenaScssEvalCallReturnNodeV0],
) -> bool {
    nodes.iter().any(|node| {
        node.kind == "functionDeclaration"
            && node.source_span_start <= position
            && node
                .name
                .as_deref()
                .is_some_and(|candidate| canonical_scss_callable_name(candidate) == name)
    })
}

pub(super) fn scss_visible_mixin_declaration_exists(
    name: &str,
    position: usize,
    nodes: &[OmenaScssEvalCallReturnNodeV0],
) -> bool {
    nodes.iter().any(|node| {
        node.kind == "mixinDeclaration"
            && node.source_span_start <= position
            && node
                .name
                .as_deref()
                .is_some_and(|candidate| canonical_scss_callable_name(candidate) == name)
    })
}

pub(super) fn max_call_stack_depth_observed(
    nodes: &[OmenaScssEvalCallReturnNodeV0],
    edges: &[OmenaScssEvalCallReturnEdgeV0],
) -> usize {
    let call_graph = declaration_call_graph(nodes, edges);
    let mut max_depth = 0usize;
    for node_key in call_graph.keys() {
        let mut stack = Vec::new();
        max_depth = max_depth.max(call_stack_depth_from(node_key, &call_graph, &mut stack));
    }
    max_depth
}

pub(super) fn declaration_call_graph(
    nodes: &[OmenaScssEvalCallReturnNodeV0],
    edges: &[OmenaScssEvalCallReturnEdgeV0],
) -> BTreeMap<String, Vec<String>> {
    let containing_declarations = nodes
        .iter()
        .filter_map(|node| {
            Some((
                node.node_key.0.clone(),
                node.containing_declaration_node_key.as_ref()?.0.clone(),
            ))
        })
        .collect::<BTreeMap<_, _>>();
    let mut graph = BTreeMap::<String, Vec<String>>::new();
    for edge in edges {
        if !matches!(edge.kind, "mixinCall" | "functionCall") {
            continue;
        }
        let Some(source_declaration) = containing_declarations.get(&edge.source_node_key.0) else {
            continue;
        };
        graph
            .entry(source_declaration.clone())
            .or_default()
            .push(edge.target_node_key.0.clone());
    }
    graph
}

pub(super) fn call_stack_depth_from(
    node_key: &str,
    graph: &BTreeMap<String, Vec<String>>,
    stack: &mut Vec<String>,
) -> usize {
    if stack.iter().any(|entry| entry == node_key) {
        return SCSS_CALL_RETURN_RECURSION_LIMIT;
    }
    if stack.len() >= SCSS_CALL_RETURN_RECURSION_LIMIT {
        return SCSS_CALL_RETURN_RECURSION_LIMIT;
    }
    stack.push(node_key.to_string());
    let depth = graph
        .get(node_key)
        .into_iter()
        .flat_map(|targets| targets.iter())
        .map(|target| call_stack_depth_from(target, graph, stack))
        .max()
        .unwrap_or(stack.len());
    stack.pop();
    depth.min(SCSS_CALL_RETURN_RECURSION_LIMIT)
}
