use std::collections::BTreeMap;

use omena_abstract_value::{
    AbstractCssValueV0, BoundedJoinFixpointNodeV0, MAX_FLOW_ANALYSIS_ITERATIONS,
    abstract_css_value_from_text, analyze_bounded_join_fixpoint, join_abstract_css_values,
};
use omena_parser::{
    LexedToken, ParsedSassSymbolFact, ParsedSassSymbolFactKind, StyleDialect, collect_style_facts,
    lex,
};
use omena_syntax::SyntaxKind;
use omena_transform_cst::StableNodeKeyV0;
use serde::Serialize;

use crate::{abstract_css_value_kind, value_eval::reduce_static_numeric_value};

const SCSS_CALL_RETURN_RECURSION_LIMIT: usize = 32;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaScssEvalControlFlowIrSummaryV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub mode: &'static str,
    pub dialect: &'static str,
    pub node_key_type: &'static str,
    pub flat_css_cfg_built: bool,
    pub merged_cross_file_graph: bool,
    pub block_count: usize,
    pub branch_block_count: usize,
    pub loop_block_count: usize,
    pub back_edge_count: usize,
    pub edge_count: usize,
    pub blocks: Vec<OmenaScssEvalControlFlowBlockV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaScssEvalControlFlowBlockV0 {
    pub node_key: StableNodeKeyV0,
    pub kind: &'static str,
    pub at_rule_name: String,
    pub header_text: String,
    pub source_span_start: usize,
    pub source_span_end: usize,
    pub successor_count: usize,
    pub has_back_edge: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaScssEvalControlFlowValueAnalysisV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub mode: &'static str,
    pub dialect: &'static str,
    pub value_type: &'static str,
    pub max_iterations: usize,
    pub converged: bool,
    pub iteration_count: usize,
    pub block_count: usize,
    pub back_edge_count: usize,
    pub loop_carried_binding_count: usize,
    pub widened_to_top_count: usize,
    pub flat_css_cfg_built: bool,
    pub merged_cross_file_graph: bool,
    pub blocks: Vec<OmenaScssEvalControlFlowValueBlockV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaScssEvalControlFlowValueBlockV0 {
    pub node_key: StableNodeKeyV0,
    pub kind: &'static str,
    pub transfer_kind: &'static str,
    pub transfer_value: Option<AbstractCssValueV0>,
    pub transfer_value_kind: Option<&'static str>,
    pub predecessor_node_keys: Vec<StableNodeKeyV0>,
    pub loop_carried_bindings: Vec<String>,
    pub loop_carried_binding_values: Vec<OmenaScssEvalControlFlowBindingValueV0>,
    pub input_value: AbstractCssValueV0,
    pub input_value_kind: &'static str,
    pub output_value: AbstractCssValueV0,
    pub output_value_kind: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaScssEvalControlFlowBindingValueV0 {
    pub name: String,
    pub value: AbstractCssValueV0,
    pub value_kind: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaScssEvalCallReturnIrSummaryV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub mode: &'static str,
    pub dialect: &'static str,
    pub node_key_type: &'static str,
    pub recursion_cap: usize,
    pub flat_css_cfg_built: bool,
    pub merged_cross_file_graph: bool,
    pub node_count: usize,
    pub declaration_node_count: usize,
    pub call_node_count: usize,
    pub return_node_count: usize,
    pub edge_count: usize,
    pub recursive_edge_count: usize,
    pub capped_recursive_call_count: usize,
    pub max_stack_depth_observed: usize,
    pub nodes: Vec<OmenaScssEvalCallReturnNodeV0>,
    pub edges: Vec<OmenaScssEvalCallReturnEdgeV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaScssEvalCallReturnNodeV0 {
    pub node_key: StableNodeKeyV0,
    pub kind: &'static str,
    pub symbol_kind: &'static str,
    pub role: &'static str,
    pub name: Option<String>,
    pub namespace: Option<String>,
    pub return_text: Option<String>,
    pub return_value: Option<AbstractCssValueV0>,
    pub return_value_kind: Option<&'static str>,
    pub source_span_start: usize,
    pub source_span_end: usize,
    pub containing_declaration_node_key: Option<StableNodeKeyV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaScssEvalCallReturnEdgeV0 {
    pub source_node_key: StableNodeKeyV0,
    pub target_node_key: StableNodeKeyV0,
    pub kind: &'static str,
    pub recursive: bool,
    pub capped_by_recursion_cap: bool,
}

pub fn summarize_scss_control_flow_ir(
    source: &str,
    dialect: StyleDialect,
) -> Option<OmenaScssEvalControlFlowIrSummaryV0> {
    if !matches!(dialect, StyleDialect::Scss | StyleDialect::Sass) {
        return None;
    }
    let lexed = lex(source, dialect);
    let tokens = lexed.tokens();
    let blocks = tokens
        .iter()
        .enumerate()
        .filter_map(|(index, token)| control_flow_block_from_token(source, tokens, index, token))
        .collect::<Vec<_>>();
    let branch_block_count = blocks
        .iter()
        .filter(|block| block.kind.starts_with("branch"))
        .count();
    let loop_block_count = blocks.iter().filter(|block| block.kind == "loop").count();
    let back_edge_count = blocks.iter().filter(|block| block.has_back_edge).count();
    let edge_count = blocks.iter().map(|block| block.successor_count).sum();
    Some(OmenaScssEvalControlFlowIrSummaryV0 {
        schema_version: "0",
        product: "omena-scss-eval.control-flow-ir",
        mode: "oracleOnly",
        dialect: dialect_label(dialect),
        node_key_type: "StableNodeKeyV0",
        flat_css_cfg_built: false,
        merged_cross_file_graph: false,
        block_count: blocks.len(),
        branch_block_count,
        loop_block_count,
        back_edge_count,
        edge_count,
        blocks,
    })
}

pub fn summarize_scss_call_return_ir(
    source: &str,
    dialect: StyleDialect,
) -> Option<OmenaScssEvalCallReturnIrSummaryV0> {
    if !matches!(dialect, StyleDialect::Scss | StyleDialect::Sass) {
        return None;
    }

    let facts = collect_style_facts(source, dialect);
    let lexed = lex(source, dialect);
    let mut candidates = facts
        .sass_symbols
        .iter()
        .filter_map(call_return_candidate_from_sass_symbol)
        .chain(collect_scss_return_candidates(source, lexed.tokens()))
        .collect::<Vec<_>>();
    candidates.sort_by(|left, right| {
        left.source_span_start
            .cmp(&right.source_span_start)
            .then(left.source_span_end.cmp(&right.source_span_end))
            .then(left.kind.cmp(right.kind))
            .then(left.name.cmp(&right.name))
    });

    let mut nodes = candidates
        .into_iter()
        .map(call_return_node_from_candidate)
        .collect::<Vec<_>>();
    stamp_containing_declarations(&mut nodes, lexed.tokens());

    let edges = build_call_return_edges(&nodes);
    let declaration_node_count = nodes
        .iter()
        .filter(|node| call_return_node_is_declaration(node))
        .count();
    let call_node_count = nodes
        .iter()
        .filter(|node| call_return_node_is_call(node))
        .count();
    let return_node_count = nodes
        .iter()
        .filter(|node| node.kind == "functionReturn")
        .count();
    let recursive_edge_count = edges.iter().filter(|edge| edge.recursive).count();
    let capped_recursive_call_count = edges
        .iter()
        .filter(|edge| edge.capped_by_recursion_cap)
        .count();
    let max_stack_depth_observed = max_call_stack_depth_observed(&nodes, &edges);

    Some(OmenaScssEvalCallReturnIrSummaryV0 {
        schema_version: "0",
        product: "omena-scss-eval.call-return-ir",
        mode: "oracleOnly",
        dialect: dialect_label(dialect),
        node_key_type: "StableNodeKeyV0",
        recursion_cap: SCSS_CALL_RETURN_RECURSION_LIMIT,
        flat_css_cfg_built: false,
        merged_cross_file_graph: false,
        node_count: nodes.len(),
        declaration_node_count,
        call_node_count,
        return_node_count,
        edge_count: edges.len(),
        recursive_edge_count,
        capped_recursive_call_count,
        max_stack_depth_observed,
        nodes,
        edges,
    })
}

pub fn analyze_scss_control_flow_values(
    source: &str,
    dialect: StyleDialect,
) -> Option<OmenaScssEvalControlFlowValueAnalysisV0> {
    if !matches!(dialect, StyleDialect::Scss | StyleDialect::Sass) {
        return None;
    }
    let summary = summarize_scss_control_flow_ir(source, dialect)?;
    let lexical_bindings = collect_lexical_scss_bindings(source, dialect);
    let nodes = summary
        .blocks
        .iter()
        .enumerate()
        .map(|(index, block)| {
            let predecessor_indices = control_flow_predecessor_indices(index, block);
            ScssControlFlowAnalysisNode {
                block: block.clone(),
                predecessor_indices,
                transfer: control_flow_transfer_for_block(block, &lexical_bindings),
            }
        })
        .collect::<Vec<_>>();
    let fixpoint = run_scss_control_flow_fixpoint(&nodes);
    let back_edge_count = nodes.iter().filter(|node| node.block.has_back_edge).count();
    let loop_carried_binding_count = nodes
        .iter()
        .map(|node| node.transfer.loop_carried_bindings().len())
        .sum();
    let blocks = nodes
        .iter()
        .zip(fixpoint.input_values.iter())
        .zip(fixpoint.output_values.iter())
        .map(|((node, input_value), output_value)| {
            let transfer_value = node.transfer.transfer_value();
            let transfer_value_kind = transfer_value.as_ref().map(abstract_css_value_kind);
            OmenaScssEvalControlFlowValueBlockV0 {
                node_key: node.block.node_key.clone(),
                kind: node.block.kind,
                transfer_kind: node.transfer.kind_label(),
                transfer_value,
                transfer_value_kind,
                predecessor_node_keys: node
                    .predecessor_indices
                    .iter()
                    .filter_map(|index| nodes.get(*index).map(|node| node.block.node_key.clone()))
                    .collect(),
                loop_carried_bindings: node.transfer.loop_carried_bindings(),
                loop_carried_binding_values: node.transfer.loop_carried_binding_values(),
                input_value_kind: abstract_css_value_kind(input_value),
                input_value: input_value.clone(),
                output_value_kind: abstract_css_value_kind(output_value),
                output_value: output_value.clone(),
            }
        })
        .collect::<Vec<_>>();
    Some(OmenaScssEvalControlFlowValueAnalysisV0 {
        schema_version: "0",
        product: "omena-scss-eval.control-flow-value-analysis",
        mode: "oracleOnly",
        dialect: dialect_label(dialect),
        value_type: "AbstractCssValueV0",
        max_iterations: MAX_FLOW_ANALYSIS_ITERATIONS,
        converged: fixpoint.converged,
        iteration_count: fixpoint.iteration_count,
        block_count: nodes.len(),
        back_edge_count,
        loop_carried_binding_count,
        widened_to_top_count: fixpoint.widened_to_top_count,
        flat_css_cfg_built: false,
        merged_cross_file_graph: false,
        blocks,
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ScssCallReturnCandidate {
    kind: &'static str,
    symbol_kind: &'static str,
    role: &'static str,
    name: Option<String>,
    namespace: Option<String>,
    return_text: Option<String>,
    return_value: Option<AbstractCssValueV0>,
    source_span_start: usize,
    source_span_end: usize,
}

fn call_return_candidate_from_sass_symbol(
    symbol: &ParsedSassSymbolFact,
) -> Option<ScssCallReturnCandidate> {
    let (kind, symbol_kind, role) = match symbol.kind {
        ParsedSassSymbolFactKind::MixinDeclaration => ("mixinDeclaration", "mixin", "declaration"),
        ParsedSassSymbolFactKind::MixinInclude => ("mixinInclude", "mixin", "call"),
        ParsedSassSymbolFactKind::FunctionDeclaration => {
            ("functionDeclaration", "function", "declaration")
        }
        ParsedSassSymbolFactKind::FunctionCall => ("functionCall", "function", "call"),
        ParsedSassSymbolFactKind::VariableDeclaration
        | ParsedSassSymbolFactKind::VariableReference => return None,
    };
    Some(ScssCallReturnCandidate {
        kind,
        symbol_kind,
        role,
        name: Some(symbol.name.clone()),
        namespace: symbol.namespace.clone(),
        return_text: None,
        return_value: None,
        source_span_start: symbol.range.start().into(),
        source_span_end: symbol.range.end().into(),
    })
}

fn collect_scss_return_candidates(
    source: &str,
    tokens: &[LexedToken],
) -> Vec<ScssCallReturnCandidate> {
    tokens
        .iter()
        .enumerate()
        .filter(|(_, token)| {
            token.kind == SyntaxKind::AtKeyword && token.text.eq_ignore_ascii_case("@return")
        })
        .map(|(index, token)| {
            let return_text = scss_return_text_from_token(source, tokens, index);
            let return_value = return_text
                .as_deref()
                .map(static_scss_return_abstract_value);
            ScssCallReturnCandidate {
                kind: "functionReturn",
                symbol_kind: "return",
                role: "return",
                name: None,
                namespace: None,
                return_text,
                return_value,
                source_span_start: token.range.start().into(),
                source_span_end: token.range.end().into(),
            }
        })
        .collect()
}

fn scss_return_text_from_token(
    source: &str,
    tokens: &[LexedToken],
    token_index: usize,
) -> Option<String> {
    let token = tokens.get(token_index)?;
    let value_start = token.range.end().into();
    let value_end = tokens
        .iter()
        .skip(token_index + 1)
        .find(|candidate| {
            matches!(
                candidate.kind,
                SyntaxKind::Semicolon
                    | SyntaxKind::SassOptionalSemicolon
                    | SyntaxKind::SassIndentedNewline
                    | SyntaxKind::RightBrace
            )
        })
        .map(|candidate| candidate.range.start().into())
        .unwrap_or(value_start);
    source
        .get(value_start..value_end)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
}

fn static_scss_return_abstract_value(value: &str) -> AbstractCssValueV0 {
    abstract_css_value_from_text(reduce_static_numeric_value(value.to_string()).as_str())
}

fn call_return_node_from_candidate(
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
        return_value_kind: candidate.return_value.as_ref().map(abstract_css_value_kind),
        return_text: candidate.return_text,
        return_value: candidate.return_value,
        source_span_start: candidate.source_span_start,
        source_span_end: candidate.source_span_end,
        containing_declaration_node_key: None,
    }
}

fn stamp_containing_declarations(
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
    let left_brace_index = tokens
        .iter()
        .enumerate()
        .skip(at_rule_index + 1)
        .find(|(_, token)| token.kind == SyntaxKind::LeftBrace)
        .map(|(index, _)| index)?;
    let right_brace_index = matching_right_brace_token_index(tokens, left_brace_index)?;
    Some(tokens[right_brace_index].range.end().into())
}

fn matching_right_brace_token_index(
    tokens: &[LexedToken],
    left_brace_index: usize,
) -> Option<usize> {
    let mut depth = 0usize;
    for (index, token) in tokens.iter().enumerate().skip(left_brace_index) {
        match token.kind {
            SyntaxKind::LeftBrace => depth += 1,
            SyntaxKind::RightBrace => {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    return Some(index);
                }
            }
            _ => {}
        }
    }
    None
}

fn build_call_return_edges(
    nodes: &[OmenaScssEvalCallReturnNodeV0],
) -> Vec<OmenaScssEvalCallReturnEdgeV0> {
    let declarations = nodes
        .iter()
        .filter(|node| call_return_node_is_declaration(node))
        .filter_map(|node| {
            Some((
                (node.symbol_kind, node.name.as_deref()?),
                node.node_key.clone(),
            ))
        })
        .collect::<BTreeMap<_, _>>();
    let mut edges = Vec::new();

    for node in nodes {
        match node.kind {
            "mixinInclude" | "functionCall" if node.namespace.is_none() => {
                if let Some(name) = node.name.as_deref()
                    && let Some(target_node_key) = declarations.get(&(node.symbol_kind, name))
                {
                    let recursive =
                        node.containing_declaration_node_key.as_ref() == Some(target_node_key);
                    edges.push(OmenaScssEvalCallReturnEdgeV0 {
                        source_node_key: node.node_key.clone(),
                        target_node_key: target_node_key.clone(),
                        kind: if node.kind == "mixinInclude" {
                            "mixinCall"
                        } else {
                            "functionCall"
                        },
                        recursive,
                        capped_by_recursion_cap: recursive,
                    });
                }
            }
            "functionReturn" => {
                if let Some(target_node_key) = node.containing_declaration_node_key.clone()
                    && nodes.iter().any(|candidate| {
                        candidate.node_key == target_node_key
                            && candidate.kind == "functionDeclaration"
                    })
                {
                    edges.push(OmenaScssEvalCallReturnEdgeV0 {
                        source_node_key: target_node_key,
                        target_node_key: node.node_key.clone(),
                        kind: "functionReturn",
                        recursive: false,
                        capped_by_recursion_cap: false,
                    });
                }
            }
            _ => {}
        }
    }

    edges
}

fn max_call_stack_depth_observed(
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

fn declaration_call_graph(
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

fn call_stack_depth_from(
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

fn call_return_node_is_declaration(node: &OmenaScssEvalCallReturnNodeV0) -> bool {
    matches!(node.kind, "mixinDeclaration" | "functionDeclaration")
}

fn call_return_node_is_call(node: &OmenaScssEvalCallReturnNodeV0) -> bool {
    matches!(node.kind, "mixinInclude" | "functionCall")
}

fn control_flow_block_from_token(
    source: &str,
    tokens: &[LexedToken],
    token_index: usize,
    token: &LexedToken,
) -> Option<OmenaScssEvalControlFlowBlockV0> {
    if token.kind != SyntaxKind::AtKeyword {
        return None;
    }
    let node_kind = scss_control_node_kind_from_name(token.text.as_str())?;
    let kind = scss_control_block_kind(node_kind)?;
    let has_back_edge = scss_control_block_has_back_edge(node_kind);
    let source_span_start = token.range.start().into();
    let source_span_end = token.range.end().into();
    Some(OmenaScssEvalControlFlowBlockV0 {
        node_key: scss_eval_stable_node_key(
            "scss-control",
            kind,
            source_span_start,
            source_span_end,
        ),
        kind,
        at_rule_name: token.text.to_string(),
        header_text: control_flow_header_text(source, tokens, token_index),
        source_span_start,
        source_span_end,
        successor_count: scss_control_block_successor_count(node_kind),
        has_back_edge,
    })
}

fn scss_eval_stable_node_key(
    prefix: &str,
    kind: &str,
    source_span_start: usize,
    source_span_end: usize,
) -> StableNodeKeyV0 {
    StableNodeKeyV0(format!(
        "{prefix}:{kind}@{source_span_start}..{source_span_end}"
    ))
}

fn control_flow_header_text(source: &str, tokens: &[LexedToken], token_index: usize) -> String {
    let Some(token) = tokens.get(token_index) else {
        return String::new();
    };
    let header_start = token.range.end().into();
    let header_end = tokens
        .iter()
        .skip(token_index + 1)
        .find(|candidate| {
            matches!(
                candidate.kind,
                SyntaxKind::LeftBrace
                    | SyntaxKind::Semicolon
                    | SyntaxKind::SassIndent
                    | SyntaxKind::SassOptionalSemicolon
            )
        })
        .map(|candidate| candidate.range.start().into())
        .unwrap_or(header_start);
    source
        .get(header_start..header_end)
        .unwrap_or("")
        .trim()
        .to_string()
}

fn scss_control_node_kind_from_name(name: &str) -> Option<SyntaxKind> {
    match name.to_ascii_lowercase().as_str() {
        "@if" => Some(SyntaxKind::ScssControlIf),
        "@else" => Some(SyntaxKind::ScssControlElse),
        "@for" => Some(SyntaxKind::ScssControlFor),
        "@each" => Some(SyntaxKind::ScssControlEach),
        "@while" => Some(SyntaxKind::ScssControlWhile),
        _ => None,
    }
}

fn scss_control_block_kind(kind: SyntaxKind) -> Option<&'static str> {
    match kind {
        SyntaxKind::ScssControlIf => Some("branchIf"),
        SyntaxKind::ScssControlElse => Some("branchElse"),
        SyntaxKind::ScssControlFor | SyntaxKind::ScssControlEach | SyntaxKind::ScssControlWhile => {
            Some("loop")
        }
        _ => None,
    }
}

const fn scss_control_block_has_back_edge(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::ScssControlFor | SyntaxKind::ScssControlEach | SyntaxKind::ScssControlWhile
    )
}

const fn scss_control_block_successor_count(kind: SyntaxKind) -> usize {
    match kind {
        SyntaxKind::ScssControlIf => 2,
        SyntaxKind::ScssControlElse => 1,
        SyntaxKind::ScssControlFor | SyntaxKind::ScssControlEach | SyntaxKind::ScssControlWhile => {
            2
        }
        _ => 0,
    }
}

const fn dialect_label(dialect: StyleDialect) -> &'static str {
    match dialect {
        StyleDialect::Css => "css",
        StyleDialect::Scss => "scss",
        StyleDialect::Sass => "sass",
        StyleDialect::Less => "less",
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ScssControlFlowAnalysisNode {
    block: OmenaScssEvalControlFlowBlockV0,
    predecessor_indices: Vec<usize>,
    transfer: ScssControlFlowTransfer,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ScssControlFlowTransfer {
    BranchCondition {
        value: AbstractCssValueV0,
    },
    LoopCarried {
        bindings: Vec<ScssControlFlowBindingValue>,
        value: AbstractCssValueV0,
    },
    PassThrough,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ScssControlFlowBindingValue {
    name: String,
    value: AbstractCssValueV0,
}

impl ScssControlFlowTransfer {
    const fn kind_label(&self) -> &'static str {
        match self {
            Self::BranchCondition { .. } => "branchCondition",
            Self::LoopCarried { .. } => "loopCarriedBindings",
            Self::PassThrough => "passThrough",
        }
    }

    fn loop_carried_bindings(&self) -> Vec<String> {
        match self {
            Self::LoopCarried { bindings, .. } => bindings
                .iter()
                .map(|binding| binding.name.clone())
                .collect(),
            Self::BranchCondition { .. } | Self::PassThrough => Vec::new(),
        }
    }

    fn loop_carried_binding_values(&self) -> Vec<OmenaScssEvalControlFlowBindingValueV0> {
        match self {
            Self::LoopCarried { bindings, .. } => bindings
                .iter()
                .map(|binding| OmenaScssEvalControlFlowBindingValueV0 {
                    name: binding.name.clone(),
                    value_kind: abstract_css_value_kind(&binding.value),
                    value: binding.value.clone(),
                })
                .collect(),
            Self::BranchCondition { .. } | Self::PassThrough => Vec::new(),
        }
    }

    fn transfer_value(&self) -> Option<AbstractCssValueV0> {
        match self {
            Self::BranchCondition { value } | Self::LoopCarried { value, .. } => {
                Some(value.clone())
            }
            Self::PassThrough => None,
        }
    }

    fn apply(&self, input_value: &AbstractCssValueV0) -> AbstractCssValueV0 {
        match self {
            Self::BranchCondition { value } | Self::LoopCarried { value, .. } => {
                join_abstract_css_values(input_value, value)
            }
            Self::PassThrough => input_value.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ScssControlFlowFixpointResult {
    converged: bool,
    iteration_count: usize,
    widened_to_top_count: usize,
    input_values: Vec<AbstractCssValueV0>,
    output_values: Vec<AbstractCssValueV0>,
}

fn run_scss_control_flow_fixpoint(
    nodes: &[ScssControlFlowAnalysisNode],
) -> ScssControlFlowFixpointResult {
    let flow_nodes = nodes
        .iter()
        .map(|node| BoundedJoinFixpointNodeV0 {
            id: node.block.node_key.0.clone(),
            predecessor_ids: node
                .predecessor_indices
                .iter()
                .filter_map(|index| nodes.get(*index).map(|node| node.block.node_key.0.clone()))
                .collect(),
            transfer: node.transfer.clone(),
        })
        .collect::<Vec<_>>();
    let fixpoint = analyze_bounded_join_fixpoint(
        &flow_nodes,
        MAX_FLOW_ANALYSIS_ITERATIONS,
        AbstractCssValueV0::Bottom,
        AbstractCssValueV0::Top,
        join_abstract_css_values,
        |input_value, transfer| transfer.apply(input_value),
    );
    let input_values = fixpoint
        .nodes
        .iter()
        .map(|node| node.input_value.clone())
        .collect::<Vec<_>>();
    let mut output_values = fixpoint
        .nodes
        .iter()
        .map(|node| node.output_value.clone())
        .collect::<Vec<_>>();
    let widened_to_top_count = if fixpoint.converged {
        0
    } else {
        output_values
            .iter_mut()
            .filter(|value| !matches!(value, AbstractCssValueV0::Top))
            .map(|value| {
                *value = AbstractCssValueV0::Top;
            })
            .count()
    };

    ScssControlFlowFixpointResult {
        converged: fixpoint.converged,
        iteration_count: fixpoint.iteration_count,
        widened_to_top_count,
        input_values,
        output_values,
    }
}

fn control_flow_predecessor_indices(
    index: usize,
    block: &OmenaScssEvalControlFlowBlockV0,
) -> Vec<usize> {
    let mut predecessors = Vec::new();
    if index > 0 {
        predecessors.push(index - 1);
    }
    if block.has_back_edge {
        predecessors.push(index);
    }
    predecessors
}

fn control_flow_transfer_for_block(
    block: &OmenaScssEvalControlFlowBlockV0,
    lexical_bindings: &BTreeMap<String, AbstractCssValueV0>,
) -> ScssControlFlowTransfer {
    match block.at_rule_name.to_ascii_lowercase().as_str() {
        "@if" | "@while" => ScssControlFlowTransfer::BranchCondition {
            value: scss_header_value(block.header_text.as_str(), lexical_bindings),
        },
        "@for" | "@each" => {
            let bindings =
                loop_carried_binding_values(block.header_text.as_str(), lexical_bindings);
            ScssControlFlowTransfer::LoopCarried {
                bindings,
                value: loop_carried_value(block.header_text.as_str(), lexical_bindings),
            }
        }
        "@else" => ScssControlFlowTransfer::PassThrough,
        _ => ScssControlFlowTransfer::PassThrough,
    }
}

fn collect_lexical_scss_bindings(
    source: &str,
    dialect: StyleDialect,
) -> BTreeMap<String, AbstractCssValueV0> {
    let lexed = lex(source, dialect);
    let tokens = lexed.tokens();
    let mut bindings = BTreeMap::new();
    for (index, token) in tokens.iter().enumerate() {
        if token.kind != SyntaxKind::ScssVariable {
            continue;
        }
        let Some(colon_index) = next_non_trivia_token_index(tokens, index + 1) else {
            continue;
        };
        if tokens[colon_index].kind != SyntaxKind::Colon {
            continue;
        }
        let value_start = tokens[colon_index].range.end().into();
        let Some(value_end_index) = declaration_end_token_index(tokens, colon_index + 1) else {
            continue;
        };
        let value_end = tokens[value_end_index].range.start().into();
        if let Some(value) = source.get(value_start..value_end).map(str::trim)
            && !value.is_empty()
        {
            bindings.insert(token.text.to_string(), abstract_css_value_from_text(value));
        }
    }
    bindings
}

fn scss_header_value(
    header: &str,
    lexical_bindings: &BTreeMap<String, AbstractCssValueV0>,
) -> AbstractCssValueV0 {
    let variables = variable_names_in_text(header);
    if variables.is_empty() {
        return abstract_css_value_from_text(header);
    }
    variables
        .iter()
        .map(|name| {
            lexical_bindings
                .get(name)
                .cloned()
                .unwrap_or(AbstractCssValueV0::Top)
        })
        .fold(AbstractCssValueV0::Bottom, |acc, value| {
            join_abstract_css_values(&acc, &value)
        })
}

fn loop_carried_value(
    header: &str,
    lexical_bindings: &BTreeMap<String, AbstractCssValueV0>,
) -> AbstractCssValueV0 {
    parse_static_for_loop_range(header)
        .or_else(|| parse_static_each_loop_source_value(header, lexical_bindings))
        .unwrap_or_else(|| scss_header_value(header, lexical_bindings))
}

fn loop_carried_binding_values(
    header: &str,
    lexical_bindings: &BTreeMap<String, AbstractCssValueV0>,
) -> Vec<ScssControlFlowBindingValue> {
    let value = loop_carried_value(header, lexical_bindings);
    loop_carried_bindings(header)
        .into_iter()
        .map(|name| ScssControlFlowBindingValue {
            name,
            value: value.clone(),
        })
        .collect()
}

fn parse_static_for_loop_range(header: &str) -> Option<AbstractCssValueV0> {
    let parts = header.split_whitespace().collect::<Vec<_>>();
    let from_index = parts
        .iter()
        .position(|part| part.eq_ignore_ascii_case("from"))?;
    let to_index = parts
        .iter()
        .position(|part| part.eq_ignore_ascii_case("to") || part.eq_ignore_ascii_case("through"))?;
    let start = parts.get(from_index + 1)?.parse::<i32>().ok()?;
    let end = parts.get(to_index + 1)?.parse::<i32>().ok()?;
    if start > end || end.saturating_sub(start) > 64 {
        return Some(AbstractCssValueV0::Top);
    }
    Some(
        (start..=end).fold(AbstractCssValueV0::Bottom, |acc, value| {
            let value = abstract_css_value_from_text(value.to_string().as_str());
            join_abstract_css_values(&acc, &value)
        }),
    )
}

fn parse_static_each_loop_source_value(
    header: &str,
    lexical_bindings: &BTreeMap<String, AbstractCssValueV0>,
) -> Option<AbstractCssValueV0> {
    let (_, source) = split_header_at_keyword(header, "in")?;
    let source = source.trim();
    if source.is_empty() {
        return None;
    }
    let source_variables = variable_names_in_text(source);
    if !source_variables.is_empty() {
        return Some(
            source_variables
                .iter()
                .map(|name| {
                    lexical_bindings
                        .get(name)
                        .cloned()
                        .unwrap_or(AbstractCssValueV0::Top)
                })
                .fold(AbstractCssValueV0::Bottom, |acc, value| {
                    join_abstract_css_values(&acc, &value)
                }),
        );
    }
    let values = source
        .split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>();
    if values.len() <= 1 || values.len() > 64 {
        return None;
    }
    Some(
        values
            .into_iter()
            .fold(AbstractCssValueV0::Bottom, |acc, value| {
                let value = abstract_css_value_from_text(value);
                join_abstract_css_values(&acc, &value)
            }),
    )
}

fn loop_carried_bindings(header: &str) -> Vec<String> {
    let separator = if header
        .split_whitespace()
        .any(|part| part.eq_ignore_ascii_case("from"))
    {
        "from"
    } else {
        "in"
    };
    let before_separator = split_header_at_keyword(header, separator)
        .map(|(left, _)| left)
        .unwrap_or(header);
    variable_names_in_text(before_separator)
}

fn split_header_at_keyword<'a>(header: &'a str, keyword: &str) -> Option<(&'a str, &'a str)> {
    let lower_header = header.to_ascii_lowercase();
    let lower_keyword = keyword.to_ascii_lowercase();
    let mut search_start = 0usize;
    while search_start < lower_header.len() {
        let relative_index = lower_header
            .get(search_start..)?
            .find(lower_keyword.as_str())?;
        let index = search_start + relative_index;
        let right_start = index + keyword.len();
        if header_keyword_has_boundaries(header, index, right_start) {
            let left = header.get(..index)?;
            let right = header.get(right_start..)?;
            return Some((left, right));
        }
        search_start = right_start;
    }
    None
}

fn header_keyword_has_boundaries(header: &str, start: usize, end: usize) -> bool {
    let before_ok = header.get(..start).is_none_or(|text| {
        text.chars()
            .next_back()
            .is_none_or(|ch| ch.is_ascii_whitespace())
    });
    let after_ok = header.get(end..).is_none_or(|text| {
        text.chars()
            .next()
            .is_none_or(|ch| ch.is_ascii_whitespace())
    });
    before_ok && after_ok
}

fn variable_names_in_text(text: &str) -> Vec<String> {
    let mut names = Vec::new();
    let mut index = 0usize;
    while index < text.len() {
        let Some(ch) = text[index..].chars().next() else {
            break;
        };
        if ch != '$' {
            index += ch.len_utf8();
            continue;
        }
        let name_start = index + ch.len_utf8();
        let name_end = variable_name_end(text, name_start);
        if name_end > name_start {
            names.push(text[index..name_end].to_string());
        }
        index = name_end.max(index + ch.len_utf8());
    }
    names.sort();
    names.dedup();
    names
}

fn variable_name_end(text: &str, mut index: usize) -> usize {
    while index < text.len() {
        let Some(ch) = text[index..].chars().next() else {
            break;
        };
        if !(ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-')) {
            break;
        }
        index += ch.len_utf8();
    }
    index
}

fn next_non_trivia_token_index(tokens: &[LexedToken], mut index: usize) -> Option<usize> {
    while tokens
        .get(index)
        .is_some_and(|token| is_trivia_token(token.kind))
    {
        index += 1;
    }
    (index < tokens.len()).then_some(index)
}

fn declaration_end_token_index(tokens: &[LexedToken], mut index: usize) -> Option<usize> {
    let mut paren_depth = 0usize;
    while index < tokens.len() {
        match tokens[index].kind {
            SyntaxKind::LeftParen => paren_depth += 1,
            SyntaxKind::RightParen => paren_depth = paren_depth.checked_sub(1)?,
            SyntaxKind::Semicolon | SyntaxKind::SassOptionalSemicolon if paren_depth == 0 => {
                return Some(index);
            }
            _ => {}
        }
        index += 1;
    }
    None
}

const fn is_trivia_token(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::Whitespace
            | SyntaxKind::LineComment
            | SyntaxKind::BlockComment
            | SyntaxKind::ScssSilentComment
            | SyntaxKind::SassIndentedNewline
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scss_control_flow_ir_summarizes_branch_and_loop_blocks() {
        let source = "@if $enabled { .on { color: green; } } @else { .off { color: red; } } @for $i from 1 through 3 { .n { order: $i; } } @each $k, $v in $map { .e { color: $v; } } @while $enabled { .w { color: red; } }";
        let report = summarize_scss_control_flow_ir(source, StyleDialect::Scss);
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.mode, "oracleOnly");
        assert!(!report.flat_css_cfg_built);
        assert!(!report.merged_cross_file_graph);
        assert_eq!(report.node_key_type, "StableNodeKeyV0");
        assert_eq!(report.block_count, 5);
        assert_eq!(report.branch_block_count, 2);
        assert_eq!(report.loop_block_count, 3);
        assert_eq!(report.back_edge_count, 3);
        assert!(report.blocks.iter().any(|block| {
            block
                .node_key
                .as_str()
                .starts_with("scss-control:branchIf@")
        }));
    }

    #[test]
    fn control_flow_ir_does_not_build_flat_css_cfg() {
        assert!(
            summarize_scss_control_flow_ir(".button { color: red; }", StyleDialect::Css).is_none()
        );
    }

    #[test]
    fn control_flow_value_analysis_uses_single_abstract_css_value_domain() {
        let source = "$enabled: 1; @if $enabled { .on { color: green; } } @for $i from 1 through 3 { .n { order: $i; } }";
        let report = analyze_scss_control_flow_values(source, StyleDialect::Scss);
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.mode, "oracleOnly");
        assert_eq!(report.value_type, "AbstractCssValueV0");
        assert!(!report.flat_css_cfg_built);
        assert!(!report.merged_cross_file_graph);
        assert!(report.converged);
        assert_eq!(report.block_count, 2);
        assert_eq!(report.back_edge_count, 1);
        assert_eq!(report.loop_carried_binding_count, 1);
        assert_eq!(report.widened_to_top_count, 0);
        assert_eq!(report.blocks[0].transfer_kind, "branchCondition");
        assert_eq!(report.blocks[0].transfer_value_kind, Some("exact"));
        assert_eq!(report.blocks[1].transfer_kind, "loopCarriedBindings");
        assert_eq!(report.blocks[1].transfer_value_kind, Some("finiteSet"));
        assert_eq!(report.blocks[1].loop_carried_bindings, vec!["$i"]);
        assert_eq!(report.blocks[1].loop_carried_binding_values.len(), 1);
        assert_eq!(report.blocks[1].loop_carried_binding_values[0].name, "$i");
        assert_eq!(
            report.blocks[1].loop_carried_binding_values[0].value_kind,
            "finiteSet"
        );
        assert_eq!(report.blocks[1].output_value_kind, "finiteSet");
    }

    #[test]
    fn control_flow_value_analysis_keeps_dynamic_each_loop_top() {
        let source = "@each $key, $value in $tokens { .item { color: $value; } }";
        let report = analyze_scss_control_flow_values(source, StyleDialect::Scss);
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.block_count, 1);
        assert_eq!(report.back_edge_count, 1);
        assert_eq!(
            report.blocks[0].loop_carried_bindings,
            vec!["$key", "$value"]
        );
        assert!(
            report.blocks[0]
                .loop_carried_binding_values
                .iter()
                .all(|binding| binding.value_kind == "top")
        );
        assert_eq!(report.blocks[0].output_value_kind, "top");
    }

    #[test]
    fn control_flow_value_analysis_tracks_static_each_binding_values() {
        let source = "@each $tone in red, blue { .item { color: $tone; } }";
        let report = analyze_scss_control_flow_values(source, StyleDialect::Scss);
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.block_count, 1);
        assert_eq!(report.blocks[0].loop_carried_bindings, vec!["$tone"]);
        assert_eq!(report.blocks[0].loop_carried_binding_values.len(), 1);
        assert_eq!(
            report.blocks[0].loop_carried_binding_values[0].value_kind,
            "finiteSet"
        );
        assert_eq!(report.blocks[0].output_value_kind, "finiteSet");
    }

    #[test]
    fn call_return_ir_summarizes_mixin_and_function_edges() {
        let source = r#"
@mixin tone($color) { color: $color; }
@function double($n) { @return $n * 2; }
.a { @include tone(red); width: double(2px); }
"#;
        let report = summarize_scss_call_return_ir(source, StyleDialect::Scss);
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.mode, "oracleOnly");
        assert_eq!(report.node_key_type, "StableNodeKeyV0");
        assert_eq!(report.recursion_cap, SCSS_CALL_RETURN_RECURSION_LIMIT);
        assert!(!report.flat_css_cfg_built);
        assert!(!report.merged_cross_file_graph);
        assert_eq!(report.declaration_node_count, 2);
        assert_eq!(report.call_node_count, 2);
        assert_eq!(report.return_node_count, 1);
        assert!(
            report
                .edges
                .iter()
                .any(|edge| edge.kind == "mixinCall" && !edge.recursive)
        );
        assert!(
            report
                .edges
                .iter()
                .any(|edge| edge.kind == "functionCall" && !edge.recursive)
        );
        assert!(
            report
                .edges
                .iter()
                .any(|edge| edge.kind == "functionReturn")
        );
        assert_eq!(report.recursive_edge_count, 0);
        assert!(
            report
                .nodes
                .iter()
                .all(|node| node.node_key.as_str().contains('@'))
        );
    }

    #[test]
    fn call_return_ir_reports_function_return_values_in_abstract_domain() {
        let source = "@function gap() { @return calc(1px + 2px); } .a { width: gap(); }";
        let report = summarize_scss_call_return_ir(source, StyleDialect::Scss);
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };
        let return_node = report
            .nodes
            .iter()
            .find(|node| node.kind == "functionReturn");
        assert!(return_node.is_some());
        let Some(return_node) = return_node else {
            return;
        };

        assert_eq!(return_node.return_text.as_deref(), Some("calc(1px + 2px)"));
        assert_eq!(return_node.return_value_kind, Some("exact"));
        assert_eq!(
            return_node.return_value,
            Some(AbstractCssValueV0::Exact {
                value: "3px".to_string()
            })
        );
    }

    #[test]
    fn call_return_ir_reports_recursion_cap_for_recursive_mixin() {
        let source = "@mixin again { @include again; }";
        let report = summarize_scss_call_return_ir(source, StyleDialect::Scss);
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.declaration_node_count, 1);
        assert_eq!(report.call_node_count, 1);
        assert_eq!(report.recursive_edge_count, 1);
        assert_eq!(report.capped_recursive_call_count, 1);
        assert_eq!(
            report.max_stack_depth_observed,
            SCSS_CALL_RETURN_RECURSION_LIMIT
        );
        assert!(report.edges.iter().any(|edge| edge.capped_by_recursion_cap));
    }

    #[test]
    fn call_return_ir_does_not_build_flat_css_cfg() {
        assert!(
            summarize_scss_call_return_ir(".button { color: red; }", StyleDialect::Css).is_none()
        );
    }

    #[test]
    fn control_flow_value_analysis_does_not_build_flat_css_cfg() {
        assert!(
            analyze_scss_control_flow_values(".button { color: red; }", StyleDialect::Css)
                .is_none()
        );
    }
}
