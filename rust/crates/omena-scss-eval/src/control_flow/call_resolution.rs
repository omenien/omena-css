use std::collections::BTreeMap;

use omena_parser::{StyleDialect, parse};
use omena_syntax::SyntaxKind;

use super::{
    SCSS_CALL_RETURN_RECURSION_LIMIT,
    model::{OmenaScssEvalCallReturnEdgeV0, OmenaScssEvalCallReturnNodeV0},
};

pub(super) fn static_scss_value_contains_function_call(value: &str, function_name: &str) -> bool {
    let canonical_function_name = canonical_scss_callable_name(function_name);
    let parsed = parse(value, StyleDialect::Scss);
    let root = parsed.syntax();
    let Some(tokens) = cst_token_texts(&root) else {
        return false;
    };
    for (index, token) in tokens.iter().enumerate() {
        if token.kind != SyntaxKind::Ident
            || canonical_scss_callable_name(token.text.as_str()) != canonical_function_name
        {
            continue;
        }
        let Some(left_paren_index) = cst_next_non_trivia_token_index(tokens.as_slice(), index + 1)
        else {
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

#[derive(Debug, Clone, PartialEq, Eq)]
struct CstTokenText {
    kind: SyntaxKind,
    text: String,
}

fn cst_token_texts(root: &cstree::syntax::SyntaxNode<SyntaxKind>) -> Option<Vec<CstTokenText>> {
    root.descendants_with_tokens()
        .filter_map(|element| element.into_token())
        .map(|token| {
            Some(CstTokenText {
                kind: token.kind(),
                text: cst_token_text(token)?,
            })
        })
        .collect()
}

fn cst_token_text(token: &cstree::syntax::SyntaxToken<SyntaxKind>) -> Option<String> {
    if let Some(resolver) = token.resolver() {
        Some(token.resolve_text(&**resolver).to_string())
    } else {
        token.static_text().map(str::to_string)
    }
}

fn cst_next_non_trivia_token_index(tokens: &[CstTokenText], mut index: usize) -> Option<usize> {
    while tokens
        .get(index)
        .is_some_and(|token| cst_is_trivia_token(token.kind))
    {
        index += 1;
    }
    (index < tokens.len()).then_some(index)
}

const fn cst_is_trivia_token(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::Whitespace
            | SyntaxKind::LineComment
            | SyntaxKind::BlockComment
            | SyntaxKind::ScssSilentComment
            | SyntaxKind::SassIndentedNewline
    )
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
