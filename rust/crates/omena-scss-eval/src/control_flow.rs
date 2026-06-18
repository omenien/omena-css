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

use crate::{
    abstract_css_value_kind,
    value_eval::{
        reduce_static_scss_value, static_scss_bang_usage_is_comparison_only,
        static_scss_literal_truthiness,
    },
};

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
    pub transfer_truthiness: Option<&'static str>,
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
    pub return_value_count: usize,
    pub exact_return_value_count: usize,
    pub finite_set_return_value_count: usize,
    pub raw_return_value_count: usize,
    pub top_return_value_count: usize,
    pub bottom_return_value_count: usize,
    pub call_resolved_return_value_count: usize,
    pub exact_call_resolved_return_value_count: usize,
    pub finite_set_call_resolved_return_value_count: usize,
    pub raw_call_resolved_return_value_count: usize,
    pub top_call_resolved_return_value_count: usize,
    pub bottom_call_resolved_return_value_count: usize,
    pub call_argument_value_count: usize,
    pub exact_call_argument_value_count: usize,
    pub finite_set_call_argument_value_count: usize,
    pub raw_call_argument_value_count: usize,
    pub top_call_argument_value_count: usize,
    pub bottom_call_argument_value_count: usize,
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
    pub parameter_names: Vec<String>,
    pub parameter_values: Vec<OmenaScssEvalCallParameterValueV0>,
    pub local_binding_values: Vec<OmenaScssEvalCallLocalBindingV0>,
    pub argument_values: Vec<OmenaScssEvalCallArgumentValueV0>,
    pub return_text: Option<String>,
    pub return_value: Option<AbstractCssValueV0>,
    pub return_value_kind: Option<&'static str>,
    pub call_resolved_return_value: Option<AbstractCssValueV0>,
    pub call_resolved_return_value_kind: Option<&'static str>,
    pub body_has_control_flow: bool,
    pub body_has_loop_control_flow: bool,
    pub return_condition_text: Option<String>,
    pub return_negated_condition_texts: Vec<String>,
    pub source_span_start: usize,
    pub source_span_end: usize,
    pub containing_declaration_node_key: Option<StableNodeKeyV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaScssEvalCallParameterValueV0 {
    pub name: String,
    pub default_value_text: Option<String>,
    pub default_value: Option<AbstractCssValueV0>,
    pub default_value_kind: Option<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaScssEvalCallArgumentValueV0 {
    pub name: Option<String>,
    pub text: String,
    pub value: AbstractCssValueV0,
    pub value_kind: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaScssEvalCallLocalBindingV0 {
    pub name: String,
    pub value_text: String,
    pub value: AbstractCssValueV0,
    pub value_kind: &'static str,
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
        .filter_map(|symbol| call_return_candidate_from_sass_symbol(source, lexed.tokens(), symbol))
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
    stamp_call_resolved_return_values(&mut nodes, &edges);
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
    let return_value_count = nodes
        .iter()
        .filter(|node| node.return_value.is_some())
        .count();
    let exact_return_value_count = nodes
        .iter()
        .filter(|node| matches!(node.return_value, Some(AbstractCssValueV0::Exact { .. })))
        .count();
    let finite_set_return_value_count = nodes
        .iter()
        .filter(|node| {
            matches!(
                node.return_value,
                Some(AbstractCssValueV0::FiniteSet { .. })
            )
        })
        .count();
    let raw_return_value_count = nodes
        .iter()
        .filter(|node| matches!(node.return_value, Some(AbstractCssValueV0::Raw { .. })))
        .count();
    let top_return_value_count = nodes
        .iter()
        .filter(|node| matches!(node.return_value, Some(AbstractCssValueV0::Top)))
        .count();
    let bottom_return_value_count = nodes
        .iter()
        .filter(|node| matches!(node.return_value, Some(AbstractCssValueV0::Bottom)))
        .count();
    let call_resolved_return_values = nodes
        .iter()
        .filter(|node| node.kind == "functionCall")
        .filter_map(|node| node.call_resolved_return_value.as_ref())
        .collect::<Vec<_>>();
    let call_resolved_return_value_count = call_resolved_return_values.len();
    let exact_call_resolved_return_value_count = call_resolved_return_values
        .iter()
        .filter(|value| matches!(value, AbstractCssValueV0::Exact { .. }))
        .count();
    let finite_set_call_resolved_return_value_count = call_resolved_return_values
        .iter()
        .filter(|value| matches!(value, AbstractCssValueV0::FiniteSet { .. }))
        .count();
    let raw_call_resolved_return_value_count = call_resolved_return_values
        .iter()
        .filter(|value| matches!(value, AbstractCssValueV0::Raw { .. }))
        .count();
    let top_call_resolved_return_value_count = call_resolved_return_values
        .iter()
        .filter(|value| matches!(value, AbstractCssValueV0::Top))
        .count();
    let bottom_call_resolved_return_value_count = call_resolved_return_values
        .iter()
        .filter(|value| matches!(value, AbstractCssValueV0::Bottom))
        .count();
    let call_argument_values = nodes
        .iter()
        .flat_map(|node| node.argument_values.iter())
        .collect::<Vec<_>>();
    let call_argument_value_count = call_argument_values.len();
    let exact_call_argument_value_count = call_argument_values
        .iter()
        .filter(|argument| matches!(&argument.value, AbstractCssValueV0::Exact { .. }))
        .count();
    let finite_set_call_argument_value_count = call_argument_values
        .iter()
        .filter(|argument| matches!(&argument.value, AbstractCssValueV0::FiniteSet { .. }))
        .count();
    let raw_call_argument_value_count = call_argument_values
        .iter()
        .filter(|argument| matches!(&argument.value, AbstractCssValueV0::Raw { .. }))
        .count();
    let top_call_argument_value_count = call_argument_values
        .iter()
        .filter(|argument| matches!(&argument.value, AbstractCssValueV0::Top))
        .count();
    let bottom_call_argument_value_count = call_argument_values
        .iter()
        .filter(|argument| matches!(&argument.value, AbstractCssValueV0::Bottom))
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
        return_value_count,
        exact_return_value_count,
        finite_set_return_value_count,
        raw_return_value_count,
        top_return_value_count,
        bottom_return_value_count,
        call_resolved_return_value_count,
        exact_call_resolved_return_value_count,
        finite_set_call_resolved_return_value_count,
        raw_call_resolved_return_value_count,
        top_call_resolved_return_value_count,
        bottom_call_resolved_return_value_count,
        call_argument_value_count,
        exact_call_argument_value_count,
        finite_set_call_argument_value_count,
        raw_call_argument_value_count,
        top_call_argument_value_count,
        bottom_call_argument_value_count,
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
            let previous_blocks = &summary.blocks[..index];
            ScssControlFlowAnalysisNode {
                block: block.clone(),
                predecessor_indices,
                transfer: control_flow_transfer_for_block(
                    block,
                    previous_blocks,
                    &lexical_bindings,
                ),
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
            let transfer_truthiness = node.transfer.transfer_truthiness();
            OmenaScssEvalControlFlowValueBlockV0 {
                node_key: node.block.node_key.clone(),
                kind: node.block.kind,
                transfer_kind: node.transfer.kind_label(),
                transfer_value,
                transfer_value_kind,
                transfer_truthiness,
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
    parameter_names: Vec<String>,
    parameter_values: Vec<OmenaScssEvalCallParameterValueV0>,
    local_binding_values: Vec<OmenaScssEvalCallLocalBindingV0>,
    argument_values: Vec<OmenaScssEvalCallArgumentValueV0>,
    return_text: Option<String>,
    return_value: Option<AbstractCssValueV0>,
    body_has_control_flow: bool,
    body_has_loop_control_flow: bool,
    return_condition_text: Option<String>,
    return_negated_condition_texts: Vec<String>,
    source_span_start: usize,
    source_span_end: usize,
}

fn call_return_candidate_from_sass_symbol(
    source: &str,
    tokens: &[LexedToken],
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
        parameter_names: scss_declaration_parameter_names_from_symbol(source, tokens, symbol),
        parameter_values: scss_declaration_parameter_values_from_symbol(source, tokens, symbol),
        local_binding_values: scss_declaration_local_bindings_from_symbol(source, tokens, symbol),
        argument_values: scss_call_argument_values_from_symbol(source, tokens, symbol),
        return_text: None,
        return_value: None,
        body_has_control_flow: scss_declaration_body_has_control_flow(tokens, symbol),
        body_has_loop_control_flow: scss_declaration_body_has_loop_control_flow(tokens, symbol),
        return_condition_text: None,
        return_negated_condition_texts: Vec::new(),
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
            let return_condition = scss_return_condition_from_token(source, tokens, index);
            ScssCallReturnCandidate {
                kind: "functionReturn",
                symbol_kind: "return",
                role: "return",
                name: None,
                namespace: None,
                parameter_names: Vec::new(),
                parameter_values: Vec::new(),
                local_binding_values: Vec::new(),
                argument_values: Vec::new(),
                return_text,
                return_value,
                body_has_control_flow: false,
                body_has_loop_control_flow: false,
                return_condition_text: return_condition
                    .as_ref()
                    .and_then(|condition| condition.condition_text.clone()),
                return_negated_condition_texts: return_condition
                    .map(|condition| condition.negated_condition_texts)
                    .unwrap_or_default(),
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
    abstract_css_value_from_text(reduce_static_scss_value(value.to_string()).as_str())
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ScssReturnCondition {
    condition_text: Option<String>,
    negated_condition_texts: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ScssBranchBlock {
    at_rule_index: usize,
    at_rule_name: String,
    condition_text: Option<String>,
    body_start_index: usize,
    body_end_index: usize,
}

fn scss_return_condition_from_token(
    source: &str,
    tokens: &[LexedToken],
    return_index: usize,
) -> Option<ScssReturnCondition> {
    let branch_blocks = collect_scss_branch_blocks(source, tokens);
    let (block_index, block) = branch_blocks
        .iter()
        .enumerate()
        .filter(|(_, block)| {
            block.body_start_index < return_index && return_index < block.body_end_index
        })
        .min_by_key(|(_, block)| block.body_end_index.saturating_sub(block.body_start_index))?;
    let negated_condition_texts =
        previous_scss_branch_condition_texts(tokens, &branch_blocks, block_index);
    Some(ScssReturnCondition {
        condition_text: block.condition_text.clone(),
        negated_condition_texts,
    })
}

fn collect_scss_branch_blocks(source: &str, tokens: &[LexedToken]) -> Vec<ScssBranchBlock> {
    tokens
        .iter()
        .enumerate()
        .filter_map(|(index, token)| {
            if token.kind != SyntaxKind::AtKeyword {
                return None;
            }
            let at_rule_name = token.text.to_ascii_lowercase();
            if !matches!(at_rule_name.as_str(), "@if" | "@else") {
                return None;
            }
            let body_start_index = tokens
                .iter()
                .enumerate()
                .skip(index + 1)
                .find(|(_, candidate)| candidate.kind == SyntaxKind::LeftBrace)
                .map(|(candidate_index, _)| candidate_index)?;
            let body_end_index = matching_right_brace_token_index(tokens, body_start_index)?;
            let header_text = control_flow_header_text(source, tokens, index);
            let condition_text = if at_rule_name == "@if" {
                Some(header_text).filter(|header| !header.is_empty())
            } else {
                scss_else_if_header_condition(header_text.as_str()).map(ToString::to_string)
            };
            Some(ScssBranchBlock {
                at_rule_index: index,
                at_rule_name,
                condition_text,
                body_start_index,
                body_end_index,
            })
        })
        .collect()
}

fn previous_scss_branch_condition_texts(
    tokens: &[LexedToken],
    branch_blocks: &[ScssBranchBlock],
    block_index: usize,
) -> Vec<String> {
    let Some(current_block) = branch_blocks.get(block_index) else {
        return Vec::new();
    };
    if current_block.at_rule_name != "@else" {
        return Vec::new();
    }

    let mut conditions = Vec::new();
    let mut cursor = current_block.at_rule_index;
    for candidate in branch_blocks[..block_index].iter().rev() {
        if candidate.body_end_index >= cursor
            || !tokens_between_are_trivia(tokens, candidate.body_end_index + 1, cursor)
        {
            continue;
        }
        if let Some(condition) = candidate.condition_text.clone() {
            conditions.push(condition);
        }
        if candidate.at_rule_name == "@if" {
            conditions.reverse();
            return conditions;
        }
        cursor = candidate.at_rule_index;
    }
    Vec::new()
}

fn tokens_between_are_trivia(tokens: &[LexedToken], start: usize, end: usize) -> bool {
    start <= end
        && end <= tokens.len()
        && tokens[start..end]
            .iter()
            .all(|token| is_trivia_token(token.kind))
}

fn scss_call_argument_values_from_symbol(
    source: &str,
    tokens: &[LexedToken],
    symbol: &ParsedSassSymbolFact,
) -> Vec<OmenaScssEvalCallArgumentValueV0> {
    if !matches!(
        symbol.kind,
        ParsedSassSymbolFactKind::FunctionCall | ParsedSassSymbolFactKind::MixinInclude
    ) {
        return Vec::new();
    }
    let Some(arguments) = scss_call_argument_texts_from_symbol(source, tokens, symbol) else {
        return Vec::new();
    };
    arguments
        .into_iter()
        .filter_map(|text| scss_call_argument_value_from_text(text.as_str()))
        .collect()
}

fn scss_call_argument_value_from_text(text: &str) -> Option<OmenaScssEvalCallArgumentValueV0> {
    let (name, text) = match scss_named_value_from_text(text)? {
        Some((name, value)) => (Some(name), value),
        None => (None, text.to_string()),
    };
    let value = static_scss_argument_abstract_value(text.as_str());
    Some(OmenaScssEvalCallArgumentValueV0 {
        name,
        value_kind: abstract_css_value_kind(&value),
        text,
        value,
    })
}

fn scss_call_argument_texts_from_symbol(
    source: &str,
    tokens: &[LexedToken],
    symbol: &ParsedSassSymbolFact,
) -> Option<Vec<String>> {
    let token_index = token_index_for_symbol_range(tokens, symbol)?;
    match symbol.kind {
        ParsedSassSymbolFactKind::FunctionCall => {
            let left_paren_index = next_non_trivia_token_index(tokens, token_index + 1)?;
            if tokens.get(left_paren_index)?.kind != SyntaxKind::LeftParen {
                return None;
            }
            let right_paren_index = matching_right_paren_token_index(tokens, left_paren_index)?;
            split_scss_call_arguments(source.get(
                token_range_end(&tokens[left_paren_index])
                    ..token_range_start(&tokens[right_paren_index]),
            )?)
        }
        ParsedSassSymbolFactKind::MixinInclude => {
            let next_index = next_non_trivia_token_index(tokens, token_index + 1)?;
            if tokens.get(next_index)?.kind == SyntaxKind::LeftParen {
                let right_paren_index = matching_right_paren_token_index(tokens, next_index)?;
                return split_scss_call_arguments(source.get(
                    token_range_end(&tokens[next_index])
                        ..token_range_start(&tokens[right_paren_index]),
                )?);
            }
            let argument_start = token_range_end(&tokens[token_index]);
            let argument_end = tokens
                .iter()
                .skip(token_index + 1)
                .find(|candidate| {
                    matches!(
                        candidate.kind,
                        SyntaxKind::Semicolon
                            | SyntaxKind::SassOptionalSemicolon
                            | SyntaxKind::SassIndentedNewline
                            | SyntaxKind::LeftBrace
                            | SyntaxKind::RightBrace
                    )
                })
                .map(token_range_start)
                .unwrap_or(argument_start);
            split_scss_call_arguments(source.get(argument_start..argument_end)?)
        }
        _ => None,
    }
}

fn scss_declaration_parameter_names_from_symbol(
    source: &str,
    tokens: &[LexedToken],
    symbol: &ParsedSassSymbolFact,
) -> Vec<String> {
    if !matches!(
        symbol.kind,
        ParsedSassSymbolFactKind::FunctionDeclaration | ParsedSassSymbolFactKind::MixinDeclaration
    ) {
        return Vec::new();
    }
    let Some(parameters) = scss_declaration_parameter_texts_from_symbol(source, tokens, symbol)
    else {
        return Vec::new();
    };
    parameters
        .into_iter()
        .filter_map(|parameter| scss_parameter_name_from_text(parameter.as_str()))
        .collect()
}

fn scss_declaration_parameter_values_from_symbol(
    source: &str,
    tokens: &[LexedToken],
    symbol: &ParsedSassSymbolFact,
) -> Vec<OmenaScssEvalCallParameterValueV0> {
    if !matches!(
        symbol.kind,
        ParsedSassSymbolFactKind::FunctionDeclaration | ParsedSassSymbolFactKind::MixinDeclaration
    ) {
        return Vec::new();
    }
    let Some(parameters) = scss_declaration_parameter_texts_from_symbol(source, tokens, symbol)
    else {
        return Vec::new();
    };
    parameters
        .into_iter()
        .filter_map(|parameter| scss_parameter_value_from_text(parameter.as_str()))
        .collect()
}

fn scss_parameter_value_from_text(parameter: &str) -> Option<OmenaScssEvalCallParameterValueV0> {
    let name = scss_parameter_name_from_text(parameter)?;
    let default_value_text = scss_named_value_from_text(parameter)
        .flatten()
        .map(|(_, value)| value);
    let default_value = default_value_text
        .as_deref()
        .map(static_scss_argument_abstract_value);
    let default_value_kind = default_value.as_ref().map(abstract_css_value_kind);
    Some(OmenaScssEvalCallParameterValueV0 {
        name,
        default_value_text,
        default_value,
        default_value_kind,
    })
}

fn scss_declaration_local_bindings_from_symbol(
    source: &str,
    tokens: &[LexedToken],
    symbol: &ParsedSassSymbolFact,
) -> Vec<OmenaScssEvalCallLocalBindingV0> {
    if !matches!(symbol.kind, ParsedSassSymbolFactKind::FunctionDeclaration) {
        return Vec::new();
    }
    let Some((body_start, body_end)) = scss_declaration_body_token_range(tokens, symbol) else {
        return Vec::new();
    };
    let mut bindings = Vec::new();
    let mut index = body_start;
    let mut nested_brace_depth = 0usize;
    while index < body_end {
        let Some(token) = tokens.get(index) else {
            break;
        };
        match token.kind {
            SyntaxKind::LeftBrace => {
                nested_brace_depth += 1;
                index += 1;
                continue;
            }
            SyntaxKind::RightBrace => {
                nested_brace_depth = nested_brace_depth.saturating_sub(1);
                index += 1;
                continue;
            }
            SyntaxKind::AtKeyword
                if nested_brace_depth == 0
                    && matches!(
                        token.text.to_ascii_lowercase().as_str(),
                        "@return" | "@if" | "@else" | "@for" | "@each" | "@while"
                    ) =>
            {
                break;
            }
            SyntaxKind::ScssVariable if nested_brace_depth == 0 => {
                let Some(colon_index) = next_non_trivia_token_index(tokens, index + 1) else {
                    index += 1;
                    continue;
                };
                if tokens.get(colon_index).map(|token| token.kind) != Some(SyntaxKind::Colon) {
                    index += 1;
                    continue;
                }
                let Some(end_index) = declaration_end_token_index(tokens, colon_index + 1) else {
                    index += 1;
                    continue;
                };
                if end_index >= body_end {
                    break;
                }
                let value_start = token_range_end(&tokens[colon_index]);
                let value_end = token_range_start(&tokens[end_index]);
                if let Some(value_text) = source.get(value_start..value_end).map(str::trim)
                    && !value_text.is_empty()
                {
                    let value = static_scss_return_abstract_value(value_text);
                    bindings.push(OmenaScssEvalCallLocalBindingV0 {
                        name: token.text.clone(),
                        value_text: value_text.to_string(),
                        value_kind: abstract_css_value_kind(&value),
                        value,
                    });
                }
                index = end_index + 1;
                continue;
            }
            _ => {}
        }
        index += 1;
    }
    bindings
}

fn scss_declaration_body_token_range(
    tokens: &[LexedToken],
    symbol: &ParsedSassSymbolFact,
) -> Option<(usize, usize)> {
    let token_index = token_index_for_symbol_range(tokens, symbol)?;
    let left_brace_index = tokens
        .iter()
        .enumerate()
        .skip(token_index + 1)
        .find(|(_, token)| token.kind == SyntaxKind::LeftBrace)
        .map(|(index, _)| index)?;
    let right_brace_index = matching_right_brace_token_index(tokens, left_brace_index)?;
    Some((left_brace_index + 1, right_brace_index))
}

fn scss_declaration_body_has_control_flow(
    tokens: &[LexedToken],
    symbol: &ParsedSassSymbolFact,
) -> bool {
    scss_declaration_body_has_matching_control_flow(tokens, symbol, |name| {
        matches!(name, "@if" | "@else" | "@for" | "@each" | "@while")
    })
}

fn scss_declaration_body_has_loop_control_flow(
    tokens: &[LexedToken],
    symbol: &ParsedSassSymbolFact,
) -> bool {
    scss_declaration_body_has_matching_control_flow(tokens, symbol, |name| {
        matches!(name, "@for" | "@each" | "@while")
    })
}

fn scss_declaration_body_has_matching_control_flow(
    tokens: &[LexedToken],
    symbol: &ParsedSassSymbolFact,
    matches_name: impl Fn(&str) -> bool,
) -> bool {
    if !matches!(
        symbol.kind,
        ParsedSassSymbolFactKind::FunctionDeclaration | ParsedSassSymbolFactKind::MixinDeclaration
    ) {
        return false;
    }
    let Some((body_start, body_end)) = scss_declaration_body_token_range(tokens, symbol) else {
        return false;
    };
    tokens
        .iter()
        .skip(body_start)
        .take(body_end.saturating_sub(body_start))
        .any(|token| {
            token.kind == SyntaxKind::AtKeyword
                && matches_name(token.text.to_ascii_lowercase().as_str())
        })
}

fn scss_declaration_parameter_texts_from_symbol(
    source: &str,
    tokens: &[LexedToken],
    symbol: &ParsedSassSymbolFact,
) -> Option<Vec<String>> {
    let token_index = token_index_for_symbol_range(tokens, symbol)?;
    let left_paren_index = next_non_trivia_token_index(tokens, token_index + 1)?;
    if tokens.get(left_paren_index)?.kind != SyntaxKind::LeftParen {
        return Some(Vec::new());
    }
    let right_paren_index = matching_right_paren_token_index(tokens, left_paren_index)?;
    split_scss_call_arguments(source.get(
        token_range_end(&tokens[left_paren_index])..token_range_start(&tokens[right_paren_index]),
    )?)
}

fn scss_parameter_name_from_text(parameter: &str) -> Option<String> {
    let trimmed = parameter.trim();
    if !trimmed.starts_with('$') || trimmed.contains("...") {
        return None;
    }
    let end = variable_name_end(trimmed, '$'.len_utf8());
    (end > '$'.len_utf8())
        .then(|| trimmed.get(..end).map(ToString::to_string))
        .flatten()
}

fn token_index_for_symbol_range(
    tokens: &[LexedToken],
    symbol: &ParsedSassSymbolFact,
) -> Option<usize> {
    let start: usize = symbol.range.start().into();
    let end: usize = symbol.range.end().into();
    tokens
        .iter()
        .enumerate()
        .find_map(|(index, token)| {
            (token_range_start(token) == start && token_range_end(token) == end).then_some(index)
        })
        .or_else(|| {
            tokens.iter().enumerate().find_map(|(index, token)| {
                (token_range_start(token) <= start
                    && start < token_range_end(token)
                    && token.text.ends_with(symbol.name.as_str()))
                .then_some(index)
            })
        })
        .or_else(|| {
            tokens.iter().enumerate().find_map(|(index, token)| {
                (token_range_start(token) >= start
                    && token_range_end(token) <= end
                    && token.text.ends_with(symbol.name.as_str()))
                .then_some(index)
            })
        })
}

fn token_range_start(token: &LexedToken) -> usize {
    token.range.start().into()
}

fn token_range_end(token: &LexedToken) -> usize {
    token.range.end().into()
}

fn matching_right_paren_token_index(
    tokens: &[LexedToken],
    left_paren_index: usize,
) -> Option<usize> {
    let mut depth = 0usize;
    for (index, token) in tokens.iter().enumerate().skip(left_paren_index) {
        match token.kind {
            SyntaxKind::LeftParen => depth += 1,
            SyntaxKind::RightParen => {
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

fn split_scss_call_arguments(arguments: &str) -> Option<Vec<String>> {
    let arguments = arguments.trim();
    if arguments.is_empty() {
        return Some(Vec::new());
    }

    let mut values = Vec::new();
    let mut cursor = 0usize;
    let mut index = 0usize;
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;
    let mut quote: Option<char> = None;
    while index < arguments.len() {
        let ch = arguments[index..].chars().next()?;
        if let Some(quote_ch) = quote {
            index += ch.len_utf8();
            if ch == '\\' {
                if let Some(escaped) = arguments[index..].chars().next() {
                    index += escaped.len_utf8();
                }
            } else if ch == quote_ch {
                quote = None;
            }
            continue;
        }
        if matches!(ch, '"' | '\'') {
            quote = Some(ch);
            index += ch.len_utf8();
            continue;
        }
        match ch {
            '(' => paren_depth += 1,
            ')' => paren_depth = paren_depth.checked_sub(1)?,
            '[' => bracket_depth += 1,
            ']' => bracket_depth = bracket_depth.checked_sub(1)?,
            ',' if paren_depth == 0 && bracket_depth == 0 => {
                let value = arguments.get(cursor..index)?.trim();
                if !scss_call_argument_is_safe(value) {
                    return None;
                }
                values.push(value.to_string());
                cursor = index + ch.len_utf8();
            }
            _ => {}
        }
        index += ch.len_utf8();
    }

    if quote.is_some() || paren_depth != 0 || bracket_depth != 0 {
        return None;
    }
    let value = arguments.get(cursor..)?.trim();
    if !scss_call_argument_is_safe(value) {
        return None;
    }
    values.push(value.to_string());
    Some(values)
}

fn scss_named_value_from_text(value: &str) -> Option<Option<(String, String)>> {
    let colon_index = scss_top_level_colon_index(value)?;
    let Some(colon_index) = colon_index else {
        return Some(None);
    };
    let name = value.get(..colon_index)?.trim();
    let value = value.get(colon_index + ':'.len_utf8()..)?.trim();
    if !name.starts_with('$') || value.is_empty() || !scss_call_argument_is_safe(value) {
        return None;
    }
    let name_end = variable_name_end(name, '$'.len_utf8());
    (name_end == name.len()).then(|| Some((name.to_string(), value.to_string())))
}

fn scss_top_level_colon_index(value: &str) -> Option<Option<usize>> {
    let mut index = 0usize;
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;
    let mut quote: Option<char> = None;
    while index < value.len() {
        let ch = value[index..].chars().next()?;
        if let Some(quote_ch) = quote {
            index += ch.len_utf8();
            if ch == '\\' {
                if let Some(escaped) = value[index..].chars().next() {
                    index += escaped.len_utf8();
                }
            } else if ch == quote_ch {
                quote = None;
            }
            continue;
        }
        if matches!(ch, '"' | '\'') {
            quote = Some(ch);
            index += ch.len_utf8();
            continue;
        }
        match ch {
            '(' => paren_depth += 1,
            ')' => paren_depth = paren_depth.checked_sub(1)?,
            '[' => bracket_depth += 1,
            ']' => bracket_depth = bracket_depth.checked_sub(1)?,
            ':' if paren_depth == 0 && bracket_depth == 0 => return Some(Some(index)),
            _ => {}
        }
        index += ch.len_utf8();
    }
    (quote.is_none() && paren_depth == 0 && bracket_depth == 0).then_some(None)
}

fn scss_call_argument_is_safe(value: &str) -> bool {
    !value.is_empty()
        && !value.contains("...")
        && !value.chars().any(|ch| matches!(ch, '{' | '}' | ';'))
        && static_scss_bang_usage_is_comparison_only(value)
}

fn static_scss_argument_abstract_value(value: &str) -> AbstractCssValueV0 {
    abstract_css_value_from_text(reduce_static_scss_value(value.to_string()).as_str())
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
        return_condition_text: candidate.return_condition_text,
        return_negated_condition_texts: candidate.return_negated_condition_texts,
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
                (
                    node.symbol_kind,
                    canonical_scss_callable_name(node.name.as_deref()?),
                ),
                node.node_key.clone(),
            ))
        })
        .collect::<BTreeMap<_, _>>();
    let mut edges = Vec::new();

    for node in nodes {
        match node.kind {
            "mixinInclude" | "functionCall" if node.namespace.is_none() => {
                if let Some(name) = node.name.as_deref()
                    && let Some(target_node_key) =
                        declarations.get(&(node.symbol_kind, canonical_scss_callable_name(name)))
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

fn stamp_call_resolved_return_values(
    nodes: &mut [OmenaScssEvalCallReturnNodeV0],
    edges: &[OmenaScssEvalCallReturnEdgeV0],
) {
    let call_graph = declaration_call_graph(nodes, edges);
    let resolutions = edges
        .iter()
        .filter(|edge| edge.kind == "functionCall")
        .filter_map(|edge| {
            let call_index = nodes
                .iter()
                .position(|node| node.node_key == edge.source_node_key)?;
            let value = call_resolved_return_value_for_edge(nodes, &call_graph, edge)?;
            Some((call_index, value))
        })
        .collect::<Vec<_>>();

    for (call_index, value) in resolutions {
        if let Some(node) = nodes.get_mut(call_index) {
            node.call_resolved_return_value_kind = Some(abstract_css_value_kind(&value));
            node.call_resolved_return_value = Some(value);
        }
    }
}

fn call_resolved_return_value_for_edge(
    nodes: &[OmenaScssEvalCallReturnNodeV0],
    call_graph: &BTreeMap<String, Vec<String>>,
    edge: &OmenaScssEvalCallReturnEdgeV0,
) -> Option<AbstractCssValueV0> {
    if edge.capped_by_recursion_cap {
        return Some(AbstractCssValueV0::Top);
    }
    let call_node = nodes
        .iter()
        .find(|node| node.node_key == edge.source_node_key)?;
    let declaration_node = nodes
        .iter()
        .find(|node| node.node_key == edge.target_node_key)?;
    if declaration_node.kind != "functionDeclaration" || call_node.kind != "functionCall" {
        return None;
    }
    call_resolved_return_value_for_call(
        nodes,
        call_graph,
        declaration_node,
        &call_node.argument_values,
        &[],
    )
}

fn call_resolved_return_value_for_call(
    nodes: &[OmenaScssEvalCallReturnNodeV0],
    call_graph: &BTreeMap<String, Vec<String>>,
    declaration_node: &OmenaScssEvalCallReturnNodeV0,
    argument_values: &[OmenaScssEvalCallArgumentValueV0],
    active_stack: &[String],
) -> Option<AbstractCssValueV0> {
    if declaration_node.kind != "functionDeclaration" {
        return None;
    }
    if declaration_node.body_has_loop_control_flow {
        return Some(AbstractCssValueV0::Top);
    }
    if active_stack
        .iter()
        .any(|entry| entry == declaration_node.node_key.as_str())
    {
        return Some(AbstractCssValueV0::Top);
    }
    if call_stack_depth_from(&declaration_node.node_key.0, call_graph, &mut Vec::new())
        >= SCSS_CALL_RETURN_RECURSION_LIMIT
    {
        return Some(AbstractCssValueV0::Top);
    }
    let mut next_stack = active_stack.to_vec();
    next_stack.push(declaration_node.node_key.0.clone());
    let context = ScssCallReturnResolutionContext {
        nodes,
        call_graph,
        active_stack: &next_stack,
    };
    let Some(mut bindings) = call_bound_argument_bindings(
        declaration_node,
        argument_values,
        declaration_node.name.as_deref(),
        Some(&context),
    ) else {
        return Some(AbstractCssValueV0::Top);
    };
    apply_call_bound_local_bindings(
        &mut bindings,
        declaration_node,
        declaration_node.name.as_deref(),
        Some(&context),
    );
    let return_nodes = nodes
        .iter()
        .filter(|node| {
            node.kind == "functionReturn"
                && node.containing_declaration_node_key.as_ref() == Some(&declaration_node.node_key)
        })
        .collect::<Vec<_>>();
    if return_nodes.is_empty() {
        return None;
    }

    let mut resolved = AbstractCssValueV0::Bottom;
    let mut active_return_count = 0usize;
    for node in return_nodes {
        match call_bound_return_activity(node, &bindings) {
            ScssCallBoundReturnActivity::Active => {}
            ScssCallBoundReturnActivity::Inactive => continue,
            ScssCallBoundReturnActivity::Unknown => return Some(AbstractCssValueV0::Top),
        }
        let value = node
            .return_text
            .as_deref()
            .map(|text| {
                call_bound_return_value(
                    text,
                    &bindings,
                    declaration_node.name.as_deref(),
                    Some(&context),
                )
            })
            .unwrap_or(AbstractCssValueV0::Top);
        resolved = join_abstract_css_values(&resolved, &value);
        active_return_count += 1;
    }

    if active_return_count == 0 {
        Some(AbstractCssValueV0::Top)
    } else {
        Some(resolved)
    }
}

fn call_bound_argument_bindings(
    declaration_node: &OmenaScssEvalCallReturnNodeV0,
    argument_values: &[OmenaScssEvalCallArgumentValueV0],
    function_name: Option<&str>,
    context: Option<&ScssCallReturnResolutionContext<'_>>,
) -> Option<BTreeMap<String, AbstractCssValueV0>> {
    let mut argument_texts = BTreeMap::<String, String>::new();
    let mut positional_index = 0usize;
    let mut saw_named_argument = false;
    for argument in argument_values {
        if let Some(argument_name) = argument.name.as_ref() {
            let argument_key = canonical_scss_variable_name(argument_name);
            saw_named_argument = true;
            if !declaration_node.parameter_values.iter().any(|parameter| {
                canonical_scss_variable_name(parameter.name.as_str()) == argument_key
            }) || argument_texts
                .insert(argument_key, argument.text.clone())
                .is_some()
            {
                return None;
            }
            continue;
        }

        if saw_named_argument {
            return None;
        }
        let parameter = declaration_node.parameter_values.get(positional_index)?;
        if argument_texts
            .insert(
                canonical_scss_variable_name(parameter.name.as_str()),
                argument.text.clone(),
            )
            .is_some()
        {
            return None;
        }
        positional_index += 1;
    }

    let mut bindings = BTreeMap::new();
    for parameter in &declaration_node.parameter_values {
        let value_text = argument_texts
            .remove(canonical_scss_variable_name(parameter.name.as_str()).as_str())
            .or_else(|| parameter.default_value_text.clone())?;
        let value = call_bound_return_value(value_text.as_str(), &bindings, function_name, context);
        insert_static_scss_binding(&mut bindings, parameter.name.as_str(), value);
    }
    if argument_texts.is_empty() {
        Some(bindings)
    } else {
        None
    }
}

fn apply_call_bound_local_bindings(
    bindings: &mut BTreeMap<String, AbstractCssValueV0>,
    declaration_node: &OmenaScssEvalCallReturnNodeV0,
    function_name: Option<&str>,
    context: Option<&ScssCallReturnResolutionContext<'_>>,
) {
    for local_binding in &declaration_node.local_binding_values {
        let value = call_bound_return_value(
            local_binding.value_text.as_str(),
            bindings,
            function_name,
            context,
        );
        insert_static_scss_binding(bindings, local_binding.name.as_str(), value);
    }
}

struct ScssCallReturnResolutionContext<'a> {
    nodes: &'a [OmenaScssEvalCallReturnNodeV0],
    call_graph: &'a BTreeMap<String, Vec<String>>,
    active_stack: &'a [String],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ScssCallBoundReturnActivity {
    Active,
    Inactive,
    Unknown,
}

fn call_bound_return_activity(
    node: &OmenaScssEvalCallReturnNodeV0,
    bindings: &BTreeMap<String, AbstractCssValueV0>,
) -> ScssCallBoundReturnActivity {
    for condition in &node.return_negated_condition_texts {
        match call_bound_condition_truthiness(condition, bindings) {
            Some(true) => return ScssCallBoundReturnActivity::Inactive,
            Some(false) => {}
            None => return ScssCallBoundReturnActivity::Unknown,
        }
    }
    match node.return_condition_text.as_deref() {
        Some(condition) => match call_bound_condition_truthiness(condition, bindings) {
            Some(true) => ScssCallBoundReturnActivity::Active,
            Some(false) => ScssCallBoundReturnActivity::Inactive,
            None => ScssCallBoundReturnActivity::Unknown,
        },
        None => ScssCallBoundReturnActivity::Active,
    }
}

fn call_bound_condition_truthiness(
    condition: &str,
    bindings: &BTreeMap<String, AbstractCssValueV0>,
) -> Option<bool> {
    let condition = if variable_names_in_text(condition).is_empty() {
        condition.to_string()
    } else {
        substitute_static_scss_header_variables(condition, bindings)?
    };
    let reduced = reduce_static_scss_value(condition);
    static_scss_literal_truthiness(reduced.as_str())
}

fn call_bound_return_value(
    return_text: &str,
    bindings: &BTreeMap<String, AbstractCssValueV0>,
    function_name: Option<&str>,
    context: Option<&ScssCallReturnResolutionContext<'_>>,
) -> AbstractCssValueV0 {
    if function_name.is_some_and(|name| static_scss_value_contains_function_call(return_text, name))
    {
        return AbstractCssValueV0::Top;
    }
    let value_text = if variable_names_in_text(return_text).is_empty() {
        return_text.to_string()
    } else {
        let Some(substituted) = substitute_static_scss_header_variables(return_text, bindings)
        else {
            return AbstractCssValueV0::Top;
        };
        substituted
    };
    if function_name
        .is_some_and(|name| static_scss_value_contains_function_call(value_text.as_str(), name))
    {
        return AbstractCssValueV0::Top;
    }
    let value_text = if let Some(context) = context {
        let Some(substituted) = substitute_call_bound_function_calls(value_text.as_str(), context)
        else {
            return AbstractCssValueV0::Top;
        };
        substituted
    } else {
        value_text
    };
    static_scss_return_abstract_value(value_text.as_str())
}

fn substitute_call_bound_function_calls(
    value: &str,
    context: &ScssCallReturnResolutionContext<'_>,
) -> Option<String> {
    let lexed = lex(value, StyleDialect::Scss);
    let tokens = lexed.tokens();
    let mut output = String::with_capacity(value.len());
    let mut cursor = 0usize;
    let mut index = 0usize;
    let mut replaced = false;

    while index < tokens.len() {
        let token = &tokens[index];
        if token.kind != SyntaxKind::Ident {
            index += 1;
            continue;
        }
        let canonical_call_name = canonical_scss_callable_name(token.text.as_str());
        let Some(declaration_node) = context.nodes.iter().find(|node| {
            node.kind == "functionDeclaration"
                && node
                    .name
                    .as_deref()
                    .is_some_and(|name| canonical_scss_callable_name(name) == canonical_call_name)
        }) else {
            index += 1;
            continue;
        };
        let Some(left_paren_index) = next_non_trivia_token_index(tokens, index + 1) else {
            index += 1;
            continue;
        };
        if tokens.get(left_paren_index)?.kind != SyntaxKind::LeftParen {
            index += 1;
            continue;
        }
        let right_paren_index = matching_right_paren_token_index(tokens, left_paren_index)?;
        let call_start = token_range_start(token);
        let call_end = token_range_end(&tokens[right_paren_index]);
        let argument_source = value.get(
            token_range_end(&tokens[left_paren_index])
                ..token_range_start(&tokens[right_paren_index]),
        )?;
        let argument_values = split_scss_call_arguments(argument_source)?
            .into_iter()
            .map(|argument| scss_call_argument_value_from_text(argument.as_str()))
            .collect::<Option<Vec<_>>>()?;
        let resolved = call_resolved_return_value_for_call(
            context.nodes,
            context.call_graph,
            declaration_node,
            &argument_values,
            context.active_stack,
        )?;
        let replacement = single_static_scss_header_value_text(&resolved)?;
        output.push_str(value.get(cursor..call_start)?);
        output.push_str(replacement);
        cursor = call_end;
        index = right_paren_index + 1;
        replaced = true;
    }

    if !replaced {
        return Some(value.to_string());
    }
    output.push_str(value.get(cursor..)?);
    Some(output)
}

fn static_scss_value_contains_function_call(value: &str, function_name: &str) -> bool {
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

fn canonical_scss_callable_name(name: &str) -> String {
    name.trim().replace('_', "-")
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
    let header_text = control_flow_header_text(source, tokens, token_index);
    let successor_count = scss_control_block_successor_count(node_kind, header_text.as_str());
    Some(OmenaScssEvalControlFlowBlockV0 {
        node_key: scss_eval_stable_node_key(
            "scss-control",
            kind,
            source_span_start,
            source_span_end,
        ),
        kind,
        at_rule_name: token.text.to_string(),
        header_text,
        source_span_start,
        source_span_end,
        successor_count,
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

fn scss_control_block_successor_count(kind: SyntaxKind, header: &str) -> usize {
    match kind {
        SyntaxKind::ScssControlIf => 2,
        SyntaxKind::ScssControlElse if scss_else_if_header_condition(header).is_some() => 2,
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
    LoopCondition {
        bindings: Vec<ScssControlFlowBindingValue>,
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
            Self::LoopCondition { .. } => "loopCondition",
            Self::LoopCarried { .. } => "loopCarriedBindings",
            Self::PassThrough => "passThrough",
        }
    }

    fn loop_carried_bindings(&self) -> Vec<String> {
        match self {
            Self::LoopCondition { bindings, .. } | Self::LoopCarried { bindings, .. } => bindings
                .iter()
                .map(|binding| binding.name.clone())
                .collect(),
            Self::BranchCondition { .. } | Self::PassThrough => Vec::new(),
        }
    }

    fn loop_carried_binding_values(&self) -> Vec<OmenaScssEvalControlFlowBindingValueV0> {
        match self {
            Self::LoopCondition { bindings, .. } | Self::LoopCarried { bindings, .. } => bindings
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
            Self::BranchCondition { value }
            | Self::LoopCondition { value, .. }
            | Self::LoopCarried { value, .. } => Some(value.clone()),
            Self::PassThrough => None,
        }
    }

    fn transfer_truthiness(&self) -> Option<&'static str> {
        match self {
            Self::BranchCondition { value } | Self::LoopCondition { value, .. } => {
                scss_static_truthiness_label(value)
            }
            Self::LoopCarried { .. } | Self::PassThrough => None,
        }
    }

    fn apply(&self, input_value: &AbstractCssValueV0) -> AbstractCssValueV0 {
        match self {
            Self::BranchCondition { value }
            | Self::LoopCondition { value, .. }
            | Self::LoopCarried { value, .. } => join_abstract_css_values(input_value, value),
            Self::PassThrough => input_value.clone(),
        }
    }
}

fn scss_static_truthiness_label(value: &AbstractCssValueV0) -> Option<&'static str> {
    match value {
        AbstractCssValueV0::Exact { value } => scss_static_truthiness_label_from_text(value),
        AbstractCssValueV0::FiniteSet { values } => {
            let mut truthiness = values
                .iter()
                .filter_map(|value| {
                    scss_static_truthiness_label(&AbstractCssValueV0::Exact {
                        value: value.clone(),
                    })
                })
                .collect::<Vec<_>>();
            truthiness.sort_unstable();
            truthiness.dedup();
            (truthiness.len() == 1).then_some(truthiness[0])
        }
        AbstractCssValueV0::Raw { value } => scss_static_truthiness_label_from_text(value),
        AbstractCssValueV0::Bottom | AbstractCssValueV0::Top => None,
    }
}

fn scss_static_truthiness_label_from_text(value: &str) -> Option<&'static str> {
    static_scss_literal_truthiness(value).map(|truthy| if truthy { "truthy" } else { "falsey" })
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
    previous_blocks: &[OmenaScssEvalControlFlowBlockV0],
    lexical_bindings: &BTreeMap<String, AbstractCssValueV0>,
) -> ScssControlFlowTransfer {
    match block.at_rule_name.to_ascii_lowercase().as_str() {
        "@if" => ScssControlFlowTransfer::BranchCondition {
            value: scss_header_value(block.header_text.as_str(), lexical_bindings),
        },
        "@while" => ScssControlFlowTransfer::LoopCondition {
            bindings: while_loop_carried_binding_values(block.header_text.as_str()),
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
        "@else" => control_flow_transfer_for_else_block(block, previous_blocks, lexical_bindings),
        _ => ScssControlFlowTransfer::PassThrough,
    }
}

fn control_flow_transfer_for_else_block(
    block: &OmenaScssEvalControlFlowBlockV0,
    previous_blocks: &[OmenaScssEvalControlFlowBlockV0],
    lexical_bindings: &BTreeMap<String, AbstractCssValueV0>,
) -> ScssControlFlowTransfer {
    if let Some(condition) = scss_else_if_header_condition(block.header_text.as_str()) {
        return ScssControlFlowTransfer::BranchCondition {
            value: scss_header_value(condition, lexical_bindings),
        };
    }
    let conditions = previous_scss_branch_condition_headers(previous_blocks);
    if !conditions.is_empty() {
        return ScssControlFlowTransfer::BranchCondition {
            value: inverted_scss_branch_chain_value(conditions.as_slice(), lexical_bindings),
        };
    }
    ScssControlFlowTransfer::PassThrough
}

fn previous_scss_branch_condition_headers(
    previous_blocks: &[OmenaScssEvalControlFlowBlockV0],
) -> Vec<&str> {
    let mut headers = Vec::new();
    for block in previous_blocks.iter().rev() {
        if block.at_rule_name.eq_ignore_ascii_case("@else") {
            let Some(condition) = scss_else_if_header_condition(block.header_text.as_str()) else {
                break;
            };
            headers.push(condition);
            continue;
        }
        if block.at_rule_name.eq_ignore_ascii_case("@if") {
            headers.push(block.header_text.as_str());
        }
        break;
    }
    headers.reverse();
    headers
}

fn scss_else_if_header_condition(header: &str) -> Option<&str> {
    let trimmed = header.trim();
    let prefix = trimmed.get(..2)?;
    let rest = trimmed.get(2..)?;
    if !prefix.eq_ignore_ascii_case("if") || !rest.chars().next().is_some_and(char::is_whitespace) {
        return None;
    }
    Some(rest.trim()).filter(|condition| !condition.is_empty())
}

fn inverted_scss_branch_chain_value(
    headers: &[&str],
    lexical_bindings: &BTreeMap<String, AbstractCssValueV0>,
) -> AbstractCssValueV0 {
    let mut saw_unknown = false;
    for header in headers {
        let value = scss_header_value(header, lexical_bindings);
        match scss_static_truthiness_label(&value) {
            Some("truthy") => return abstract_css_value_from_text("false"),
            Some("falsey") => {}
            _ => saw_unknown = true,
        }
    }
    if saw_unknown {
        AbstractCssValueV0::Top
    } else {
        abstract_css_value_from_text("true")
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
            insert_static_scss_binding(
                &mut bindings,
                token.text.as_str(),
                abstract_css_value_from_text(value),
            );
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
        return static_scss_header_abstract_value(header);
    }
    if let Some(substituted) = substitute_static_scss_header_variables(header, lexical_bindings) {
        return static_scss_header_abstract_value(substituted.as_str());
    }
    variables
        .iter()
        .map(|name| {
            static_scss_binding_value(lexical_bindings, name)
                .cloned()
                .unwrap_or(AbstractCssValueV0::Top)
        })
        .fold(AbstractCssValueV0::Bottom, |acc, value| {
            join_abstract_css_values(&acc, &value)
        })
}

fn static_scss_header_abstract_value(value: &str) -> AbstractCssValueV0 {
    abstract_css_value_from_text(reduce_static_scss_value(value.to_string()).as_str())
}

fn substitute_static_scss_header_variables(
    header: &str,
    lexical_bindings: &BTreeMap<String, AbstractCssValueV0>,
) -> Option<String> {
    let mut output = String::with_capacity(header.len());
    let mut index = 0usize;
    while index < header.len() {
        let ch = header[index..].chars().next()?;
        if ch != '$' {
            output.push(ch);
            index += ch.len_utf8();
            continue;
        }
        let name_end = variable_name_end(header, index + ch.len_utf8());
        let name = header.get(index..name_end)?;
        let value = static_scss_binding_value(lexical_bindings, name)
            .and_then(single_static_scss_header_value_text)?;
        output.push_str(value);
        index = name_end.max(index + ch.len_utf8());
    }
    Some(output)
}

fn single_static_scss_header_value_text(value: &AbstractCssValueV0) -> Option<&str> {
    match value {
        AbstractCssValueV0::Exact { value } | AbstractCssValueV0::Raw { value } => {
            Some(value.as_str())
        }
        AbstractCssValueV0::FiniteSet { values } if values.len() == 1 => {
            values.first().map(String::as_str)
        }
        AbstractCssValueV0::Bottom
        | AbstractCssValueV0::Top
        | AbstractCssValueV0::FiniteSet { .. } => None,
    }
}

fn loop_carried_value(
    header: &str,
    lexical_bindings: &BTreeMap<String, AbstractCssValueV0>,
) -> AbstractCssValueV0 {
    parse_static_for_loop_range(header, lexical_bindings)
        .or_else(|| parse_static_each_loop_source_value(header, lexical_bindings))
        .unwrap_or_else(|| scss_header_value(header, lexical_bindings))
}

fn loop_carried_binding_values(
    header: &str,
    lexical_bindings: &BTreeMap<String, AbstractCssValueV0>,
) -> Vec<ScssControlFlowBindingValue> {
    if let Some(values) = static_each_map_loop_binding_values(header, lexical_bindings) {
        return values;
    }
    if let Some(values) = static_each_tuple_loop_binding_values(header, lexical_bindings) {
        return values;
    }
    let value = loop_carried_value(header, lexical_bindings);
    loop_carried_bindings(header)
        .into_iter()
        .map(|name| ScssControlFlowBindingValue {
            name,
            value: value.clone(),
        })
        .collect()
}

fn static_each_map_loop_binding_values(
    header: &str,
    lexical_bindings: &BTreeMap<String, AbstractCssValueV0>,
) -> Option<Vec<ScssControlFlowBindingValue>> {
    let bindings = loop_carried_bindings(header);
    if bindings.len() != 2 {
        return None;
    }
    let (_, source) = split_header_at_keyword(header, "in")?;
    let source = static_each_source_text(source.trim(), lexical_bindings)?;
    let entries = parse_static_each_map_entries(source)?;
    if entries.len() > 64 {
        return None;
    }

    let key_value = entries
        .iter()
        .map(|(key, _)| abstract_css_value_from_text(key.as_str()))
        .fold(AbstractCssValueV0::Bottom, |acc, value| {
            join_abstract_css_values(&acc, &value)
        });
    let item_value = entries
        .iter()
        .map(|(_, value)| abstract_css_value_from_text(value.as_str()))
        .fold(AbstractCssValueV0::Bottom, |acc, value| {
            join_abstract_css_values(&acc, &value)
        });

    Some(vec![
        ScssControlFlowBindingValue {
            name: bindings[0].clone(),
            value: key_value,
        },
        ScssControlFlowBindingValue {
            name: bindings[1].clone(),
            value: item_value,
        },
    ])
}

fn static_each_tuple_loop_binding_values(
    header: &str,
    lexical_bindings: &BTreeMap<String, AbstractCssValueV0>,
) -> Option<Vec<ScssControlFlowBindingValue>> {
    let bindings = loop_carried_bindings(header);
    if bindings.len() <= 1 {
        return None;
    }
    let (_, source) = split_header_at_keyword(header, "in")?;
    let source = static_each_source_text(source.trim(), lexical_bindings)?;
    let entries = parse_static_each_tuple_entries(source, bindings.len())?;
    if entries.len() > 64 {
        return None;
    }

    Some(
        bindings
            .into_iter()
            .enumerate()
            .map(|(index, name)| {
                let value = entries
                    .iter()
                    .map(|entry| abstract_css_value_from_text(entry[index].as_str()))
                    .fold(AbstractCssValueV0::Bottom, |acc, value| {
                        join_abstract_css_values(&acc, &value)
                    });
                ScssControlFlowBindingValue { name, value }
            })
            .collect(),
    )
}

fn while_loop_carried_binding_values(header: &str) -> Vec<ScssControlFlowBindingValue> {
    loop_carried_bindings(header)
        .into_iter()
        .map(|name| ScssControlFlowBindingValue {
            name,
            value: AbstractCssValueV0::Top,
        })
        .collect()
}

fn parse_static_for_loop_range(
    header: &str,
    lexical_bindings: &BTreeMap<String, AbstractCssValueV0>,
) -> Option<AbstractCssValueV0> {
    let parts = header.split_whitespace().collect::<Vec<_>>();
    let from_index = parts
        .iter()
        .position(|part| part.eq_ignore_ascii_case("from"))?;
    let to_index = parts
        .iter()
        .position(|part| part.eq_ignore_ascii_case("to") || part.eq_ignore_ascii_case("through"))?;
    let includes_end = parts[to_index].eq_ignore_ascii_case("through");
    let start = parse_static_for_loop_bound(parts.get(from_index + 1)?, lexical_bindings)?;
    let end = parse_static_for_loop_bound(parts.get(to_index + 1)?, lexical_bindings)?;
    if start > end {
        return Some(AbstractCssValueV0::Top);
    }
    let value_count = if includes_end {
        i64::from(end) - i64::from(start) + 1
    } else {
        i64::from(end) - i64::from(start)
    };
    if !(0..=64).contains(&value_count) {
        return Some(AbstractCssValueV0::Top);
    }
    if value_count == 0 {
        return Some(AbstractCssValueV0::Bottom);
    }
    let last = if includes_end {
        end
    } else {
        end.saturating_sub(1)
    };
    Some(
        (start..=last).fold(AbstractCssValueV0::Bottom, |acc, value| {
            let value = abstract_css_value_from_text(value.to_string().as_str());
            join_abstract_css_values(&acc, &value)
        }),
    )
}

fn parse_static_for_loop_bound(
    value: &str,
    lexical_bindings: &BTreeMap<String, AbstractCssValueV0>,
) -> Option<i32> {
    let reduced = match scss_header_value(value, lexical_bindings) {
        AbstractCssValueV0::Exact { value } | AbstractCssValueV0::Raw { value } => value,
        AbstractCssValueV0::Bottom
        | AbstractCssValueV0::Top
        | AbstractCssValueV0::FiniteSet { .. } => return None,
    };
    reduced.parse::<i32>().ok()
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
    let source_text = if source_is_single_static_variable(source) {
        static_scss_binding_value(lexical_bindings, source)
            .and_then(single_static_scss_header_value_text)
            .unwrap_or(source)
    } else if !source_variables.is_empty() {
        return Some(
            source_variables
                .iter()
                .map(|name| {
                    static_scss_binding_value(lexical_bindings, name)
                        .cloned()
                        .unwrap_or(AbstractCssValueV0::Top)
                })
                .fold(AbstractCssValueV0::Bottom, |acc, value| {
                    join_abstract_css_values(&acc, &value)
                }),
        );
    } else {
        source
    };
    let values = split_static_scss_top_level(source_text, ',')?;
    if values.len() <= 1 || values.len() > 64 {
        return None;
    }
    Some(
        values
            .into_iter()
            .fold(AbstractCssValueV0::Bottom, |acc, value| {
                let value = abstract_css_value_from_text(value.as_str());
                join_abstract_css_values(&acc, &value)
            }),
    )
}

fn source_is_single_static_variable(source: &str) -> bool {
    source.starts_with('$') && variable_name_end(source, '$'.len_utf8()) == source.len()
}

fn static_each_source_text<'a>(
    source: &'a str,
    lexical_bindings: &'a BTreeMap<String, AbstractCssValueV0>,
) -> Option<&'a str> {
    if source_is_single_static_variable(source) {
        return static_scss_binding_value(lexical_bindings, source)
            .and_then(single_static_scss_header_value_text);
    }
    Some(source)
}

fn parse_static_each_map_entries(source: &str) -> Option<Vec<(String, String)>> {
    let inner = source
        .strip_prefix('(')
        .and_then(|source| source.strip_suffix(')'))?
        .trim();
    if inner.is_empty() {
        return None;
    }
    let entries = split_static_scss_top_level(inner, ',')?;
    if entries.len() <= 1 {
        return None;
    }

    let mut pairs = Vec::with_capacity(entries.len());
    for entry in entries {
        let (key, value) = split_static_scss_key_value(entry.as_str())?;
        pairs.push((key.to_string(), value.to_string()));
    }
    Some(pairs)
}

fn split_static_scss_key_value(entry: &str) -> Option<(&str, &str)> {
    let colon_index = static_scss_top_level_separator_index(entry, ':')??;
    let key = entry.get(..colon_index)?.trim();
    let value = entry.get(colon_index + ':'.len_utf8()..)?.trim();
    if key.is_empty() || value.is_empty() || key.contains('$') || value.contains('$') {
        return None;
    }
    Some((key, value))
}

fn parse_static_each_tuple_entries(source: &str, arity: usize) -> Option<Vec<Vec<String>>> {
    let entries = split_static_scss_top_level(source.trim(), ',')?;
    if entries.len() <= 1 {
        return None;
    }

    let mut tuples = Vec::with_capacity(entries.len());
    for entry in entries {
        let values = parse_static_each_tuple_entry_values(entry.as_str(), arity)?;
        tuples.push(values);
    }
    Some(tuples)
}

fn parse_static_each_tuple_entry_values(entry: &str, arity: usize) -> Option<Vec<String>> {
    let entry = strip_static_scss_outer_container(entry.trim()).unwrap_or_else(|| entry.trim());
    let comma_values = split_static_scss_top_level(entry, ',')?;
    let values = if comma_values.len() == arity {
        comma_values
    } else {
        split_static_scss_top_level_whitespace(entry)?
    };
    if values.len() != arity
        || values
            .iter()
            .any(|value| !static_each_tuple_value_is_static(value))
    {
        return None;
    }
    Some(values)
}

fn static_each_tuple_value_is_static(value: &str) -> bool {
    !value.is_empty()
        && !value.contains('$')
        && static_scss_top_level_separator_index(value, ':').is_some_and(|index| index.is_none())
}

fn strip_static_scss_outer_container(value: &str) -> Option<&str> {
    let trimmed = value.trim();
    if trimmed.len() < 2 {
        return None;
    }
    let (open, close) = match trimmed.chars().next()? {
        '(' => ('(', ')'),
        '[' => ('[', ']'),
        _ => return None,
    };
    let end = static_scss_balanced_value_end(trimmed, 0, open, close)?;
    if end != trimmed.len() {
        return None;
    }
    trimmed
        .get(open.len_utf8()..trimmed.len().saturating_sub(close.len_utf8()))
        .map(str::trim)
}

fn split_static_scss_top_level(source: &str, separator: char) -> Option<Vec<String>> {
    let mut values = Vec::new();
    let mut cursor = 0usize;
    let mut index = 0usize;
    while index < source.len() {
        let ch = source[index..].chars().next()?;
        if ch == separator {
            let value = source.get(cursor..index)?.trim();
            if value.is_empty() {
                return None;
            }
            values.push(value.to_string());
            cursor = index + ch.len_utf8();
        }
        index = static_scss_next_value_index(source, index)?;
    }
    let value = source.get(cursor..)?.trim();
    if value.is_empty() {
        return None;
    }
    values.push(value.to_string());
    Some(values)
}

fn split_static_scss_top_level_whitespace(source: &str) -> Option<Vec<String>> {
    let mut values = Vec::new();
    let mut cursor = 0usize;
    let mut index = 0usize;
    while index < source.len() {
        let ch = source[index..].chars().next()?;
        if ch.is_ascii_whitespace() {
            let value = source.get(cursor..index)?.trim();
            if !value.is_empty() {
                values.push(value.to_string());
            }
            index += ch.len_utf8();
            while index < source.len() {
                let Some(next_ch) = source[index..].chars().next() else {
                    break;
                };
                if !next_ch.is_ascii_whitespace() {
                    break;
                }
                index += next_ch.len_utf8();
            }
            cursor = index;
            continue;
        }
        index = static_scss_next_value_index(source, index)?;
    }
    let value = source.get(cursor..)?.trim();
    if !value.is_empty() {
        values.push(value.to_string());
    }
    Some(values)
}

fn static_scss_top_level_separator_index(source: &str, separator: char) -> Option<Option<usize>> {
    let mut index = 0usize;
    while index < source.len() {
        let ch = source[index..].chars().next()?;
        if ch == separator {
            return Some(Some(index));
        }
        index = static_scss_next_value_index(source, index)?;
    }
    Some(None)
}

fn static_scss_next_value_index(source: &str, index: usize) -> Option<usize> {
    let ch = source[index..].chars().next()?;
    match ch {
        '"' | '\'' => static_scss_quoted_value_end(source, index, ch),
        '(' => static_scss_balanced_value_end(source, index, '(', ')'),
        '[' => static_scss_balanced_value_end(source, index, '[', ']'),
        ')' | ']' => None,
        _ => Some(index + ch.len_utf8()),
    }
}

fn static_scss_quoted_value_end(source: &str, start: usize, quote: char) -> Option<usize> {
    let mut index = start + quote.len_utf8();
    while index < source.len() {
        let ch = source[index..].chars().next()?;
        index += ch.len_utf8();
        if ch == '\\' {
            if let Some(escaped) = source[index..].chars().next() {
                index += escaped.len_utf8();
            }
        } else if ch == quote {
            return Some(index);
        }
    }
    None
}

fn static_scss_balanced_value_end(
    source: &str,
    start: usize,
    open: char,
    close: char,
) -> Option<usize> {
    let mut depth = 0usize;
    let mut index = start;
    while index < source.len() {
        let ch = source[index..].chars().next()?;
        match ch {
            '"' | '\'' => {
                index = static_scss_quoted_value_end(source, index, ch)?;
                continue;
            }
            _ if ch == open => depth += 1,
            _ if ch == close => {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    return Some(index + ch.len_utf8());
                }
            }
            _ => {}
        }
        index += ch.len_utf8();
    }
    None
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
    variable_names_in_text_preserving_order(before_separator)
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
    let mut names = variable_names_in_text_preserving_order(text);
    names.sort();
    names.dedup();
    names
}

fn variable_names_in_text_preserving_order(text: &str) -> Vec<String> {
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
        if name_end > name_start
            && let Some(name) = text.get(index..name_end)
            && !names.iter().any(|candidate| candidate == name)
        {
            names.push(name.to_string());
        }
        index = name_end.max(index + ch.len_utf8());
    }
    names
}

fn canonical_scss_variable_name(name: &str) -> String {
    let trimmed = name.trim();
    let bare = trimmed.strip_prefix('$').unwrap_or(trimmed);
    format!("${}", bare.replace('_', "-"))
}

fn insert_static_scss_binding(
    bindings: &mut BTreeMap<String, AbstractCssValueV0>,
    name: &str,
    value: AbstractCssValueV0,
) {
    bindings.insert(canonical_scss_variable_name(name), value);
}

fn static_scss_binding_value<'a>(
    bindings: &'a BTreeMap<String, AbstractCssValueV0>,
    name: &str,
) -> Option<&'a AbstractCssValueV0> {
    bindings.get(canonical_scss_variable_name(name).as_str())
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
    fn scss_control_flow_ir_counts_else_if_as_conditional_branch() {
        let source = "$enabled: false; $fallback: true; @if $enabled { .on { color: green; } } @else if $fallback { .fallback { color: yellow; } } @else { .off { color: red; } }";
        let report = summarize_scss_control_flow_ir(source, StyleDialect::Scss);
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.block_count, 3);
        assert_eq!(report.branch_block_count, 3);
        assert_eq!(report.edge_count, 5);
        assert_eq!(report.blocks[0].kind, "branchIf");
        assert_eq!(report.blocks[0].successor_count, 2);
        assert_eq!(report.blocks[1].kind, "branchElse");
        assert_eq!(report.blocks[1].header_text, "if $fallback");
        assert_eq!(report.blocks[1].successor_count, 2);
        assert_eq!(report.blocks[2].kind, "branchElse");
        assert_eq!(report.blocks[2].successor_count, 1);
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
        assert_eq!(report.blocks[0].transfer_truthiness, Some("truthy"));
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
    fn control_flow_value_analysis_reports_sass_branch_truthiness() {
        let source = "$enabled: false; @if $enabled { .on { color: green; } }";
        let report = analyze_scss_control_flow_values(source, StyleDialect::Scss);
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.block_count, 1);
        assert_eq!(report.blocks[0].transfer_kind, "branchCondition");
        assert_eq!(report.blocks[0].transfer_value_kind, Some("raw"));
        assert_eq!(report.blocks[0].transfer_truthiness, Some("falsey"));
    }

    #[test]
    fn control_flow_value_analysis_reports_sass_not_branch_truthiness() {
        let source = "$enabled: true; @if not $enabled { .off { color: red; } }";
        let report = analyze_scss_control_flow_values(source, StyleDialect::Scss);
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.block_count, 1);
        assert_eq!(report.blocks[0].transfer_kind, "branchCondition");
        assert_eq!(report.blocks[0].transfer_truthiness, Some("falsey"));
    }

    #[test]
    fn control_flow_value_analysis_reports_sass_boolean_branch_truthiness() {
        let source = "$enabled: true; @if $enabled and false { .off { color: red; } }";
        let report = analyze_scss_control_flow_values(source, StyleDialect::Scss);
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.block_count, 1);
        assert_eq!(report.blocks[0].transfer_kind, "branchCondition");
        assert_eq!(report.blocks[0].transfer_truthiness, Some("falsey"));
    }

    #[test]
    fn control_flow_value_analysis_reports_sass_equality_branch_truthiness() {
        let source = "$enabled: false; @if $enabled == false { .off { color: red; } }";
        let report = analyze_scss_control_flow_values(source, StyleDialect::Scss);
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.block_count, 1);
        assert_eq!(report.blocks[0].transfer_kind, "branchCondition");
        assert_eq!(report.blocks[0].transfer_truthiness, Some("truthy"));
    }

    #[test]
    fn control_flow_value_analysis_reports_sass_inequality_branch_truthiness() {
        let source = "$enabled: false; @if $enabled != true { .off { color: red; } }";
        let report = analyze_scss_control_flow_values(source, StyleDialect::Scss);
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.block_count, 1);
        assert_eq!(report.blocks[0].transfer_kind, "branchCondition");
        assert_eq!(report.blocks[0].transfer_truthiness, Some("truthy"));
    }

    #[test]
    fn control_flow_value_analysis_reports_sass_numeric_ordering_branch_truthiness() {
        let source = "$gap: 3px; @if $gap > 2px { .on { color: green; } }";
        let report = analyze_scss_control_flow_values(source, StyleDialect::Scss);
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.block_count, 1);
        assert_eq!(report.blocks[0].transfer_kind, "branchCondition");
        assert_eq!(report.blocks[0].transfer_truthiness, Some("truthy"));
    }

    #[test]
    fn control_flow_value_analysis_reports_sass_zero_numeric_ordering_branch_truthiness() {
        let source = "$gap: 0px; @if $gap >= 0 { .on { color: green; } }";
        let report = analyze_scss_control_flow_values(source, StyleDialect::Scss);
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.block_count, 1);
        assert_eq!(report.blocks[0].transfer_kind, "branchCondition");
        assert_eq!(report.blocks[0].transfer_truthiness, Some("truthy"));
    }

    #[test]
    fn control_flow_value_analysis_reduces_static_if_header_values() {
        let source = "$enabled: if(false, false, true); @if $enabled { .on { color: green; } } @else { .off { color: red; } }";
        let report = analyze_scss_control_flow_values(source, StyleDialect::Scss);
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.block_count, 2);
        assert_eq!(report.blocks[0].transfer_kind, "branchCondition");
        assert_eq!(report.blocks[0].transfer_truthiness, Some("truthy"));
        assert_eq!(report.blocks[1].transfer_kind, "branchCondition");
        assert_eq!(report.blocks[1].transfer_truthiness, Some("falsey"));
    }

    #[test]
    fn control_flow_value_analysis_reports_sass_else_branch_truthiness() {
        let source = "$enabled: false; @if $enabled { .on { color: green; } } @else { .off { color: red; } }";
        let report = analyze_scss_control_flow_values(source, StyleDialect::Scss);
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.block_count, 2);
        assert_eq!(report.blocks[0].kind, "branchIf");
        assert_eq!(report.blocks[0].transfer_kind, "branchCondition");
        assert_eq!(report.blocks[0].transfer_truthiness, Some("falsey"));
        assert_eq!(report.blocks[1].kind, "branchElse");
        assert_eq!(report.blocks[1].transfer_kind, "branchCondition");
        assert_eq!(report.blocks[1].transfer_truthiness, Some("truthy"));
    }

    #[test]
    fn control_flow_value_analysis_reports_sass_else_if_branch_truthiness() {
        let source = "$enabled: false; @if $enabled { .on { color: green; } } @else if not $enabled { .off { color: red; } }";
        let report = analyze_scss_control_flow_values(source, StyleDialect::Scss);
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.block_count, 2);
        assert_eq!(report.blocks[1].kind, "branchElse");
        assert_eq!(report.blocks[1].transfer_kind, "branchCondition");
        assert_eq!(report.blocks[1].transfer_truthiness, Some("truthy"));
    }

    #[test]
    fn control_flow_value_analysis_reports_sass_final_else_after_else_if_truthiness() {
        let source = "$enabled: false; $fallback: false; @if $enabled { .on { color: green; } } @else if $fallback { .fallback { color: yellow; } } @else { .off { color: red; } }";
        let report = analyze_scss_control_flow_values(source, StyleDialect::Scss);
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.block_count, 3);
        assert_eq!(report.blocks[2].kind, "branchElse");
        assert_eq!(report.blocks[2].transfer_kind, "branchCondition");
        assert_eq!(report.blocks[2].transfer_truthiness, Some("truthy"));
    }

    #[test]
    fn control_flow_value_analysis_final_else_observes_full_else_if_chain() {
        let source = "$enabled: true; $fallback: false; @if $enabled { .on { color: green; } } @else if $fallback { .fallback { color: yellow; } } @else { .off { color: red; } }";
        let report = analyze_scss_control_flow_values(source, StyleDialect::Scss);
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.block_count, 3);
        assert_eq!(report.blocks[2].kind, "branchElse");
        assert_eq!(report.blocks[2].transfer_kind, "branchCondition");
        assert_eq!(report.blocks[2].transfer_truthiness, Some("falsey"));
    }

    #[test]
    fn control_flow_value_analysis_reports_parenthesized_branch_truthiness() {
        let source = "$enabled: false; @if ($enabled) { .off { color: red; } }";
        let report = analyze_scss_control_flow_values(source, StyleDialect::Scss);
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.block_count, 1);
        assert_eq!(report.blocks[0].transfer_kind, "branchCondition");
        assert_eq!(report.blocks[0].transfer_truthiness, Some("falsey"));
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
    fn control_flow_value_analysis_tracks_static_each_map_pair_values() {
        let source = "@each $name, $color in (primary: red, secondary: blue) { .#{$name} { color: $color; } }";
        let report = analyze_scss_control_flow_values(source, StyleDialect::Scss);
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.block_count, 1);
        assert_eq!(
            report.blocks[0].loop_carried_bindings,
            vec!["$name", "$color"]
        );
        assert_eq!(report.blocks[0].loop_carried_binding_values.len(), 2);
        assert_eq!(
            report.blocks[0].loop_carried_binding_values[0].name,
            "$name"
        );
        assert_eq!(
            report.blocks[0].loop_carried_binding_values[0].value,
            AbstractCssValueV0::FiniteSet {
                values: vec!["primary".to_string(), "secondary".to_string()]
            }
        );
        assert_eq!(
            report.blocks[0].loop_carried_binding_values[1].name,
            "$color"
        );
        assert_eq!(
            report.blocks[0].loop_carried_binding_values[1].value,
            AbstractCssValueV0::FiniteSet {
                values: vec!["#00f".to_string(), "red".to_string()]
            }
        );
    }

    #[test]
    fn control_flow_value_analysis_tracks_static_each_map_variable_pair_values() {
        let source = "$tones: (primary: red, secondary: blue); @each $name, $color in $tones { .#{$name} { color: $color; } }";
        let report = analyze_scss_control_flow_values(source, StyleDialect::Scss);
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.block_count, 1);
        assert_eq!(
            report.blocks[0].loop_carried_bindings,
            vec!["$name", "$color"]
        );
        assert_eq!(
            report.blocks[0].loop_carried_binding_values[0].value,
            AbstractCssValueV0::FiniteSet {
                values: vec!["primary".to_string(), "secondary".to_string()]
            }
        );
        assert_eq!(
            report.blocks[0].loop_carried_binding_values[1].value,
            AbstractCssValueV0::FiniteSet {
                values: vec!["#00f".to_string(), "red".to_string()]
            }
        );
    }

    #[test]
    fn control_flow_value_analysis_tracks_static_each_tuple_pair_values() {
        let source =
            "@each $icon, $size in (save, 16px), (cancel, 24px) { .#{$icon} { width: $size; } }";
        let report = analyze_scss_control_flow_values(source, StyleDialect::Scss);
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.block_count, 1);
        assert_eq!(
            report.blocks[0].loop_carried_bindings,
            vec!["$icon", "$size"]
        );
        assert_eq!(
            report.blocks[0].loop_carried_binding_values[0].value,
            AbstractCssValueV0::FiniteSet {
                values: vec!["cancel".to_string(), "save".to_string()]
            }
        );
        assert_eq!(
            report.blocks[0].loop_carried_binding_values[1].value,
            AbstractCssValueV0::FiniteSet {
                values: vec!["16px".to_string(), "24px".to_string()]
            }
        );
        assert_eq!(report.blocks[0].output_value_kind, "finiteSet");
    }

    #[test]
    fn control_flow_value_analysis_tracks_static_each_tuple_variable_pair_values() {
        let source = "$pairs: (save, 16px), (cancel, 24px); @each $icon, $size in $pairs { .#{$icon} { width: $size; } }";
        let report = analyze_scss_control_flow_values(source, StyleDialect::Scss);
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.block_count, 1);
        assert_eq!(
            report.blocks[0].loop_carried_bindings,
            vec!["$icon", "$size"]
        );
        assert_eq!(
            report.blocks[0].loop_carried_binding_values[0].value,
            AbstractCssValueV0::FiniteSet {
                values: vec!["cancel".to_string(), "save".to_string()]
            }
        );
        assert_eq!(
            report.blocks[0].loop_carried_binding_values[1].value,
            AbstractCssValueV0::FiniteSet {
                values: vec!["16px".to_string(), "24px".to_string()]
            }
        );
    }

    #[test]
    fn control_flow_value_analysis_tracks_static_each_space_tuple_pair_values() {
        let source = "@each $icon, $size in save 16px, cancel 24px { .#{$icon} { width: $size; } }";
        let report = analyze_scss_control_flow_values(source, StyleDialect::Scss);
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(
            report.blocks[0].loop_carried_bindings,
            vec!["$icon", "$size"]
        );
        assert_eq!(
            report.blocks[0].loop_carried_binding_values[0].value,
            AbstractCssValueV0::FiniteSet {
                values: vec!["cancel".to_string(), "save".to_string()]
            }
        );
        assert_eq!(
            report.blocks[0].loop_carried_binding_values[1].value,
            AbstractCssValueV0::FiniteSet {
                values: vec!["16px".to_string(), "24px".to_string()]
            }
        );
    }

    #[test]
    fn control_flow_value_analysis_models_for_to_as_end_exclusive() {
        let source = "@for $i from 1 to 3 { .n { order: $i; } }";
        let report = analyze_scss_control_flow_values(source, StyleDialect::Scss);
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.block_count, 1);
        assert_eq!(report.blocks[0].loop_carried_bindings, vec!["$i"]);
        assert_eq!(
            report.blocks[0].loop_carried_binding_values[0].value,
            AbstractCssValueV0::FiniteSet {
                values: vec!["1".to_string(), "2".to_string()]
            }
        );
    }

    #[test]
    fn control_flow_value_analysis_resolves_static_for_loop_bounds() {
        let source = "$start: 1; $end: 3; @for $i from $start through $end { .n { order: $i; } }";
        let report = analyze_scss_control_flow_values(source, StyleDialect::Scss);
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.block_count, 1);
        assert_eq!(report.blocks[0].loop_carried_bindings, vec!["$i"]);
        assert_eq!(
            report.blocks[0].loop_carried_binding_values[0].value,
            AbstractCssValueV0::FiniteSet {
                values: vec!["1".to_string(), "2".to_string(), "3".to_string()]
            }
        );
    }

    #[test]
    fn control_flow_value_analysis_resolves_hyphen_underscore_equivalent_loop_bounds() {
        let source = "$start_value: 1; $end_value: 3; @for $i from $start-value through $end-value { .n { order: $i; } }";
        let report = analyze_scss_control_flow_values(source, StyleDialect::Scss);
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.blocks[0].loop_carried_bindings, vec!["$i"]);
        assert_eq!(
            report.blocks[0].loop_carried_binding_values[0].value,
            AbstractCssValueV0::FiniteSet {
                values: vec!["1".to_string(), "2".to_string(), "3".to_string()]
            }
        );
    }

    #[test]
    fn control_flow_value_analysis_tracks_while_condition_loop_bindings() {
        let source = "$i: 0; @while $i < 3 { $i: $i + 1; .n { order: $i; } }";
        let report = analyze_scss_control_flow_values(source, StyleDialect::Scss);
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.block_count, 1);
        assert_eq!(report.back_edge_count, 1);
        assert_eq!(report.loop_carried_binding_count, 1);
        assert_eq!(report.blocks[0].kind, "loop");
        assert_eq!(report.blocks[0].transfer_kind, "loopCondition");
        assert_eq!(report.blocks[0].transfer_truthiness, None);
        assert_eq!(report.blocks[0].loop_carried_bindings, vec!["$i"]);
        assert_eq!(
            report.blocks[0].loop_carried_binding_values[0].value_kind,
            "top"
        );
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
        assert_eq!(report.return_value_count, 1);
        assert_eq!(report.call_argument_value_count, 2);
        assert_eq!(report.exact_call_argument_value_count, 2);
        assert_eq!(report.raw_call_argument_value_count, 0);
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
        let source = "@function gap($value) { @return calc(1px + 2px); } .a { width: gap(2px); }";
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
        let function_call = report.nodes.iter().find(|node| node.kind == "functionCall");
        assert!(function_call.is_some());
        let Some(function_call) = function_call else {
            return;
        };

        assert_eq!(function_call.argument_values.len(), 1);
        assert_eq!(function_call.argument_values[0].text, "2px");
        assert_eq!(function_call.argument_values[0].value_kind, "exact");
        assert_eq!(report.return_value_count, 1);
        assert_eq!(report.call_argument_value_count, 1);
        assert_eq!(report.exact_return_value_count, 1);
        assert_eq!(report.exact_call_argument_value_count, 1);
        assert_eq!(report.raw_return_value_count, 0);
        assert_eq!(report.top_return_value_count, 0);
        assert_eq!(return_node.return_value_kind, Some("exact"));
        assert_eq!(
            return_node.return_value,
            Some(AbstractCssValueV0::Exact {
                value: "3px".to_string()
            })
        );
    }

    #[test]
    fn call_return_ir_resolves_function_call_returns_with_arguments() {
        let source = "@function double($value) { @return $value * 2; } .a { width: double(2px); }";
        let report = summarize_scss_call_return_ir(source, StyleDialect::Scss);
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };
        let declaration = report
            .nodes
            .iter()
            .find(|node| node.kind == "functionDeclaration");
        assert!(declaration.is_some());
        let Some(declaration) = declaration else {
            return;
        };
        assert_eq!(declaration.parameter_names, vec!["$value"]);

        let function_call = report
            .nodes
            .iter()
            .find(|node| node.kind == "functionCall" && node.name.as_deref() == Some("double"));
        assert!(function_call.is_some());
        let Some(function_call) = function_call else {
            return;
        };

        assert_eq!(report.call_resolved_return_value_count, 1);
        assert_eq!(report.exact_call_resolved_return_value_count, 1);
        assert_eq!(function_call.call_resolved_return_value_kind, Some("exact"));
        assert_eq!(
            function_call.call_resolved_return_value,
            Some(AbstractCssValueV0::Exact {
                value: "4px".to_string()
            })
        );
    }

    #[test]
    fn call_return_ir_resolves_function_call_returns_through_static_if() {
        let source = "@function tone($enabled) { @return if($enabled, red, blue); } .a { color: tone(false); }";
        let report = summarize_scss_call_return_ir(source, StyleDialect::Scss);
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };
        let function_call = report
            .nodes
            .iter()
            .find(|node| node.kind == "functionCall" && node.name.as_deref() == Some("tone"));
        assert!(function_call.is_some());
        let Some(function_call) = function_call else {
            return;
        };

        assert_eq!(report.call_resolved_return_value_count, 1);
        assert_eq!(report.exact_call_resolved_return_value_count, 1);
        assert_eq!(function_call.call_resolved_return_value_kind, Some("exact"));
        assert_eq!(
            function_call.call_resolved_return_value,
            Some(AbstractCssValueV0::Exact {
                value: "#00f".to_string()
            })
        );
    }

    #[test]
    fn call_return_ir_resolves_call_bound_local_variable_returns() {
        let source = "@function offset($base) { $next: $base + 1px; @return $next + 1px; } .a { width: offset(2px); }";
        let report = summarize_scss_call_return_ir(source, StyleDialect::Scss);
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };
        let declaration = report.nodes.iter().find(|node| {
            node.kind == "functionDeclaration" && node.name.as_deref() == Some("offset")
        });
        assert!(declaration.is_some());
        let Some(declaration) = declaration else {
            return;
        };
        assert_eq!(declaration.local_binding_values.len(), 1);
        assert_eq!(declaration.local_binding_values[0].name, "$next");
        assert_eq!(
            declaration.local_binding_values[0].value_text,
            "$base + 1px"
        );

        let function_call = report
            .nodes
            .iter()
            .find(|node| node.kind == "functionCall" && node.name.as_deref() == Some("offset"));
        assert!(function_call.is_some());
        let Some(function_call) = function_call else {
            return;
        };

        assert_eq!(report.call_resolved_return_value_count, 1);
        assert_eq!(report.exact_call_resolved_return_value_count, 1);
        assert_eq!(function_call.call_resolved_return_value_kind, Some("exact"));
        assert_eq!(
            function_call.call_resolved_return_value,
            Some(AbstractCssValueV0::Exact {
                value: "4px".to_string()
            })
        );
    }

    #[test]
    fn call_return_ir_resolves_hyphen_underscore_equivalent_local_bindings() {
        let source = "@function offset($base) { $next_value: $base + 1px; @return $next-value + 1px; } .a { width: offset(2px); }";
        let report = summarize_scss_call_return_ir(source, StyleDialect::Scss);
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };
        let function_call = report
            .nodes
            .iter()
            .find(|node| node.kind == "functionCall" && node.name.as_deref() == Some("offset"));
        assert!(function_call.is_some());
        let Some(function_call) = function_call else {
            return;
        };

        assert_eq!(function_call.call_resolved_return_value_kind, Some("exact"));
        assert_eq!(
            function_call.call_resolved_return_value,
            Some(AbstractCssValueV0::Exact {
                value: "4px".to_string()
            })
        );
    }

    #[test]
    fn call_return_ir_resolves_call_bound_local_variable_chains() {
        let source = "@function scale($base) { $next: $base + 1px; $double: $next * 2; @return $double; } .a { width: scale(2px); }";
        let report = summarize_scss_call_return_ir(source, StyleDialect::Scss);
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };
        let declaration = report.nodes.iter().find(|node| {
            node.kind == "functionDeclaration" && node.name.as_deref() == Some("scale")
        });
        assert!(declaration.is_some());
        let Some(declaration) = declaration else {
            return;
        };
        assert_eq!(declaration.local_binding_values.len(), 2);
        assert_eq!(declaration.local_binding_values[0].name, "$next");
        assert_eq!(declaration.local_binding_values[1].name, "$double");

        let function_call = report
            .nodes
            .iter()
            .find(|node| node.kind == "functionCall" && node.name.as_deref() == Some("scale"));
        assert!(function_call.is_some());
        let Some(function_call) = function_call else {
            return;
        };

        assert_eq!(report.call_resolved_return_value_count, 1);
        assert_eq!(report.exact_call_resolved_return_value_count, 1);
        assert_eq!(function_call.call_resolved_return_value_kind, Some("exact"));
        assert_eq!(
            function_call.call_resolved_return_value,
            Some(AbstractCssValueV0::Exact {
                value: "6px".to_string()
            })
        );
    }

    #[test]
    fn call_return_ir_resolves_named_function_arguments() {
        let source = "@function pair($left, $right) { @return $left + $right; } .a { width: pair($right: 2px, $left: 1px); }";
        let report = summarize_scss_call_return_ir(source, StyleDialect::Scss);
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };
        let function_call = report
            .nodes
            .iter()
            .find(|node| node.kind == "functionCall" && node.name.as_deref() == Some("pair"));
        assert!(function_call.is_some());
        let Some(function_call) = function_call else {
            return;
        };

        assert_eq!(function_call.argument_values.len(), 2);
        assert_eq!(
            function_call.argument_values[0].name.as_deref(),
            Some("$right")
        );
        assert_eq!(function_call.argument_values[0].text, "2px");
        assert_eq!(
            function_call.argument_values[1].name.as_deref(),
            Some("$left")
        );
        assert_eq!(function_call.argument_values[1].text, "1px");
        assert_eq!(report.call_resolved_return_value_count, 1);
        assert_eq!(report.exact_call_resolved_return_value_count, 1);
        assert_eq!(function_call.call_resolved_return_value_kind, Some("exact"));
        assert_eq!(
            function_call.call_resolved_return_value,
            Some(AbstractCssValueV0::Exact {
                value: "3px".to_string()
            })
        );
    }

    #[test]
    fn call_return_ir_resolves_hyphen_underscore_equivalent_parameter_references() {
        let source =
            "@function gap($base_value) { @return $base-value + 1px; } .a { width: gap(2px); }";
        let report = summarize_scss_call_return_ir(source, StyleDialect::Scss);
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };
        let function_call = report
            .nodes
            .iter()
            .find(|node| node.kind == "functionCall" && node.name.as_deref() == Some("gap"));
        assert!(function_call.is_some());
        let Some(function_call) = function_call else {
            return;
        };

        assert_eq!(function_call.call_resolved_return_value_kind, Some("exact"));
        assert_eq!(
            function_call.call_resolved_return_value,
            Some(AbstractCssValueV0::Exact {
                value: "3px".to_string()
            })
        );
    }

    #[test]
    fn call_return_ir_resolves_hyphen_underscore_equivalent_named_arguments() {
        let source = "@function gap($base_value) { @return $base-value + 1px; } .a { width: gap($base-value: 2px); }";
        let report = summarize_scss_call_return_ir(source, StyleDialect::Scss);
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };
        let function_call = report
            .nodes
            .iter()
            .find(|node| node.kind == "functionCall" && node.name.as_deref() == Some("gap"));
        assert!(function_call.is_some());
        let Some(function_call) = function_call else {
            return;
        };

        assert_eq!(function_call.call_resolved_return_value_kind, Some("exact"));
        assert_eq!(
            function_call.call_resolved_return_value,
            Some(AbstractCssValueV0::Exact {
                value: "3px".to_string()
            })
        );
    }

    #[test]
    fn call_return_ir_resolves_default_function_arguments() {
        let source = "@function offset($value: 1px, $extra: 2px) { @return $value + $extra; } .a { width: offset(); }";
        let report = summarize_scss_call_return_ir(source, StyleDialect::Scss);
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };
        let declaration = report.nodes.iter().find(|node| {
            node.kind == "functionDeclaration" && node.name.as_deref() == Some("offset")
        });
        assert!(declaration.is_some());
        let Some(declaration) = declaration else {
            return;
        };
        assert_eq!(declaration.parameter_values.len(), 2);
        assert_eq!(
            declaration.parameter_values[0]
                .default_value_text
                .as_deref(),
            Some("1px")
        );
        assert_eq!(
            declaration.parameter_values[1]
                .default_value_text
                .as_deref(),
            Some("2px")
        );

        let function_call = report
            .nodes
            .iter()
            .find(|node| node.kind == "functionCall" && node.name.as_deref() == Some("offset"));
        assert!(function_call.is_some());
        let Some(function_call) = function_call else {
            return;
        };

        assert_eq!(report.call_resolved_return_value_count, 1);
        assert_eq!(report.exact_call_resolved_return_value_count, 1);
        assert_eq!(function_call.call_resolved_return_value_kind, Some("exact"));
        assert_eq!(
            function_call.call_resolved_return_value,
            Some(AbstractCssValueV0::Exact {
                value: "3px".to_string()
            })
        );
    }

    #[test]
    fn call_return_ir_resolves_default_arguments_from_prior_parameters() {
        let source = "@function offset($value, $extra: $value + 1px) { @return $extra; } .a { width: offset(2px); }";
        let report = summarize_scss_call_return_ir(source, StyleDialect::Scss);
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };
        let function_call = report
            .nodes
            .iter()
            .find(|node| node.kind == "functionCall" && node.name.as_deref() == Some("offset"));
        assert!(function_call.is_some());
        let Some(function_call) = function_call else {
            return;
        };

        assert_eq!(report.call_resolved_return_value_count, 1);
        assert_eq!(report.exact_call_resolved_return_value_count, 1);
        assert_eq!(function_call.call_resolved_return_value_kind, Some("exact"));
        assert_eq!(
            function_call.call_resolved_return_value,
            Some(AbstractCssValueV0::Exact {
                value: "3px".to_string()
            })
        );
    }

    #[test]
    fn call_return_ir_resolves_composed_same_file_function_calls() {
        let source = "@function inc($value) { @return $value + 1px; } @function gap($value) { @return inc($value) + 1px; } .a { width: gap(2px); }";
        let report = summarize_scss_call_return_ir(source, StyleDialect::Scss);
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };
        let function_call = report.nodes.iter().find(|node| {
            node.kind == "functionCall"
                && node.name.as_deref() == Some("gap")
                && node.containing_declaration_node_key.is_none()
        });
        assert!(function_call.is_some());
        let Some(function_call) = function_call else {
            return;
        };

        assert_eq!(function_call.call_resolved_return_value_kind, Some("exact"));
        assert_eq!(
            function_call.call_resolved_return_value,
            Some(AbstractCssValueV0::Exact {
                value: "4px".to_string()
            })
        );
    }

    #[test]
    fn call_return_ir_resolves_hyphen_underscore_equivalent_function_calls() {
        let source = "@function inc_value($value) { @return $value + 1px; } @function gap_value($value) { @return inc-value($value) + 1px; } .a { width: gap-value(2px); }";
        let report = summarize_scss_call_return_ir(source, StyleDialect::Scss);
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };
        let function_call = report.nodes.iter().find(|node| {
            node.kind == "functionCall"
                && node.name.as_deref() == Some("gap-value")
                && node.containing_declaration_node_key.is_none()
        });
        assert!(function_call.is_some());
        let Some(function_call) = function_call else {
            return;
        };

        assert_eq!(function_call.call_resolved_return_value_kind, Some("exact"));
        assert_eq!(
            function_call.call_resolved_return_value,
            Some(AbstractCssValueV0::Exact {
                value: "4px".to_string()
            })
        );
    }

    #[test]
    fn call_return_ir_resolves_local_values_with_same_file_function_calls() {
        let source = "@function inc($value) { @return $value + 1px; } @function gap($value) { $next: inc($value); @return $next + 1px; } .a { width: gap(2px); }";
        let report = summarize_scss_call_return_ir(source, StyleDialect::Scss);
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };
        let function_call = report.nodes.iter().find(|node| {
            node.kind == "functionCall"
                && node.name.as_deref() == Some("gap")
                && node.containing_declaration_node_key.is_none()
        });
        assert!(function_call.is_some());
        let Some(function_call) = function_call else {
            return;
        };

        assert_eq!(function_call.call_resolved_return_value_kind, Some("exact"));
        assert_eq!(
            function_call.call_resolved_return_value,
            Some(AbstractCssValueV0::Exact {
                value: "4px".to_string()
            })
        );
    }

    #[test]
    fn call_return_ir_keeps_indirect_recursive_function_calls_top() {
        let source = "@function a($value) { @return b($value); } @function b($value) { @return a($value); } .x { width: a(1px); }";
        let report = summarize_scss_call_return_ir(source, StyleDialect::Scss);
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };
        let function_call = report.nodes.iter().find(|node| {
            node.kind == "functionCall"
                && node.name.as_deref() == Some("a")
                && node.containing_declaration_node_key.is_none()
        });
        assert!(function_call.is_some());
        let Some(function_call) = function_call else {
            return;
        };

        assert_eq!(function_call.call_resolved_return_value_kind, Some("top"));
        assert_eq!(
            function_call.call_resolved_return_value,
            Some(AbstractCssValueV0::Top)
        );
    }

    #[test]
    fn call_return_ir_caps_hyphen_underscore_recursive_function_calls() {
        let source = "@function again_value($value) { @return again-value($value); } .a { width: again-value(1px); }";
        let report = summarize_scss_call_return_ir(source, StyleDialect::Scss);
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };
        let function_call = report.nodes.iter().find(|node| {
            node.kind == "functionCall"
                && node.name.as_deref() == Some("again-value")
                && node.containing_declaration_node_key.is_none()
        });
        assert!(function_call.is_some());
        let Some(function_call) = function_call else {
            return;
        };

        assert_eq!(function_call.call_resolved_return_value_kind, Some("top"));
        assert_eq!(
            function_call.call_resolved_return_value,
            Some(AbstractCssValueV0::Top)
        );
    }

    #[test]
    fn call_return_ir_resolves_same_file_function_call_arguments() {
        let source = "@function inc($value) { @return $value + 1px; } @function gap($value) { @return $value + 1px; } .a { width: gap(inc(2px)); }";
        let report = summarize_scss_call_return_ir(source, StyleDialect::Scss);
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };
        let function_call = report.nodes.iter().find(|node| {
            node.kind == "functionCall"
                && node.name.as_deref() == Some("gap")
                && node.containing_declaration_node_key.is_none()
        });
        assert!(function_call.is_some());
        let Some(function_call) = function_call else {
            return;
        };

        assert_eq!(function_call.call_resolved_return_value_kind, Some("exact"));
        assert_eq!(
            function_call.call_resolved_return_value,
            Some(AbstractCssValueV0::Exact {
                value: "4px".to_string()
            })
        );
    }

    #[test]
    fn call_return_ir_resolves_named_same_file_function_call_arguments() {
        let source = "@function inc($value) { @return $value + 1px; } @function pair($left, $right) { @return $left + $right; } .a { width: pair($right: inc(2px), $left: 1px); }";
        let report = summarize_scss_call_return_ir(source, StyleDialect::Scss);
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };
        let function_call = report.nodes.iter().find(|node| {
            node.kind == "functionCall"
                && node.name.as_deref() == Some("pair")
                && node.containing_declaration_node_key.is_none()
        });
        assert!(function_call.is_some());
        let Some(function_call) = function_call else {
            return;
        };

        assert_eq!(function_call.call_resolved_return_value_kind, Some("exact"));
        assert_eq!(
            function_call.call_resolved_return_value,
            Some(AbstractCssValueV0::Exact {
                value: "4px".to_string()
            })
        );
    }

    #[test]
    fn call_return_ir_keeps_positional_after_named_arguments_top() {
        let source = "@function pair($left, $right) { @return $left + $right; } .a { width: pair($left: 1px, 2px); }";
        let report = summarize_scss_call_return_ir(source, StyleDialect::Scss);
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };
        let function_call = report
            .nodes
            .iter()
            .find(|node| node.kind == "functionCall" && node.name.as_deref() == Some("pair"));
        assert!(function_call.is_some());
        let Some(function_call) = function_call else {
            return;
        };

        assert_eq!(report.call_resolved_return_value_count, 1);
        assert_eq!(report.top_call_resolved_return_value_count, 1);
        assert_eq!(function_call.call_resolved_return_value_kind, Some("top"));
        assert_eq!(
            function_call.call_resolved_return_value,
            Some(AbstractCssValueV0::Top)
        );
    }

    #[test]
    fn call_return_ir_keeps_malformed_named_argument_top() {
        let source = "@function gap($value) { @return $value; } .a { width: gap(value: 1px); }";
        let report = summarize_scss_call_return_ir(source, StyleDialect::Scss);
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };
        let function_call = report
            .nodes
            .iter()
            .find(|node| node.kind == "functionCall" && node.name.as_deref() == Some("gap"));
        assert!(function_call.is_some());
        let Some(function_call) = function_call else {
            return;
        };

        assert_eq!(report.call_resolved_return_value_count, 1);
        assert_eq!(report.top_call_resolved_return_value_count, 1);
        assert_eq!(function_call.call_resolved_return_value_kind, Some("top"));
        assert_eq!(
            function_call.call_resolved_return_value,
            Some(AbstractCssValueV0::Top)
        );
    }

    #[test]
    fn call_return_ir_uses_local_variables_in_branch_conditions() {
        let source = "@function tone($enabled) { $flag: $enabled; @if $flag { @return red; } @else { @return blue; } } .a { color: tone(false); }";
        let report = summarize_scss_call_return_ir(source, StyleDialect::Scss);
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };
        let function_call = report
            .nodes
            .iter()
            .find(|node| node.kind == "functionCall" && node.name.as_deref() == Some("tone"));
        assert!(function_call.is_some());
        let Some(function_call) = function_call else {
            return;
        };

        assert_eq!(report.call_resolved_return_value_count, 1);
        assert_eq!(report.exact_call_resolved_return_value_count, 1);
        assert_eq!(function_call.call_resolved_return_value_kind, Some("exact"));
        assert_eq!(
            function_call.call_resolved_return_value,
            Some(AbstractCssValueV0::Exact {
                value: "#00f".to_string()
            })
        );
    }

    #[test]
    fn call_return_ir_keeps_dynamic_local_variable_branches_top() {
        let source = "@function tone() { $flag: var(--enabled); @if $flag { @return red; } @else { @return blue; } } .a { color: tone(); }";
        let report = summarize_scss_call_return_ir(source, StyleDialect::Scss);
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };
        let function_call = report
            .nodes
            .iter()
            .find(|node| node.kind == "functionCall" && node.name.as_deref() == Some("tone"));
        assert!(function_call.is_some());
        let Some(function_call) = function_call else {
            return;
        };

        assert_eq!(report.call_resolved_return_value_count, 1);
        assert_eq!(report.top_call_resolved_return_value_count, 1);
        assert_eq!(function_call.call_resolved_return_value_kind, Some("top"));
        assert_eq!(
            function_call.call_resolved_return_value,
            Some(AbstractCssValueV0::Top)
        );
    }

    #[test]
    fn call_return_ir_resolves_call_bound_if_branch_returns() {
        let source = "@function tone($enabled) { @if $enabled { @return red; } @else { @return blue; } } .a { color: tone(true); }";
        let report = summarize_scss_call_return_ir(source, StyleDialect::Scss);
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };
        let function_call = report
            .nodes
            .iter()
            .find(|node| node.kind == "functionCall" && node.name.as_deref() == Some("tone"));
        assert!(function_call.is_some());
        let Some(function_call) = function_call else {
            return;
        };

        assert_eq!(report.call_resolved_return_value_count, 1);
        assert_eq!(report.exact_call_resolved_return_value_count, 1);
        assert_eq!(function_call.call_resolved_return_value_kind, Some("exact"));
        assert!(report.nodes.iter().any(|node| {
            node.kind == "functionReturn"
                && node.return_condition_text.as_deref() == Some("$enabled")
        }));
        assert_eq!(
            function_call.call_resolved_return_value,
            Some(AbstractCssValueV0::Exact {
                value: "red".to_string()
            })
        );
    }

    #[test]
    fn call_return_ir_resolves_call_bound_else_branch_returns() {
        let source = "@function tone($enabled) { @if $enabled { @return red; } @else { @return blue; } } .a { color: tone(false); }";
        let report = summarize_scss_call_return_ir(source, StyleDialect::Scss);
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };
        let function_call = report
            .nodes
            .iter()
            .find(|node| node.kind == "functionCall" && node.name.as_deref() == Some("tone"));
        assert!(function_call.is_some());
        let Some(function_call) = function_call else {
            return;
        };

        assert_eq!(report.call_resolved_return_value_count, 1);
        assert_eq!(report.exact_call_resolved_return_value_count, 1);
        assert_eq!(function_call.call_resolved_return_value_kind, Some("exact"));
        assert!(report.nodes.iter().any(|node| {
            node.kind == "functionReturn"
                && node.return_text.as_deref() == Some("blue")
                && node.return_condition_text.is_none()
                && node.return_negated_condition_texts == vec!["$enabled".to_string()]
        }));
        assert_eq!(
            function_call.call_resolved_return_value,
            Some(AbstractCssValueV0::Exact {
                value: "#00f".to_string()
            })
        );
    }

    #[test]
    fn call_return_ir_resolves_call_bound_else_if_branch_returns() {
        let source = "@function tone($first, $second) { @if $first { @return red; } @else if $second { @return green; } @else { @return blue; } } .a { color: tone(false, true); }";
        let report = summarize_scss_call_return_ir(source, StyleDialect::Scss);
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };
        let function_call = report
            .nodes
            .iter()
            .find(|node| node.kind == "functionCall" && node.name.as_deref() == Some("tone"));
        assert!(function_call.is_some());
        let Some(function_call) = function_call else {
            return;
        };

        assert_eq!(report.call_resolved_return_value_count, 1);
        assert_eq!(report.exact_call_resolved_return_value_count, 1);
        assert_eq!(function_call.call_resolved_return_value_kind, Some("exact"));
        assert_eq!(
            function_call.call_resolved_return_value,
            Some(AbstractCssValueV0::Exact {
                value: "green".to_string()
            })
        );
    }

    #[test]
    fn call_return_ir_keeps_dynamic_branch_returns_top() {
        let source = "@function tone($enabled) { @if $enabled { @return red; } @else { @return blue; } } .a { color: tone(var(--enabled)); }";
        let report = summarize_scss_call_return_ir(source, StyleDialect::Scss);
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };
        let function_call = report
            .nodes
            .iter()
            .find(|node| node.kind == "functionCall" && node.name.as_deref() == Some("tone"));
        assert!(function_call.is_some());
        let Some(function_call) = function_call else {
            return;
        };

        assert_eq!(report.call_resolved_return_value_count, 1);
        assert_eq!(report.top_call_resolved_return_value_count, 1);
        assert_eq!(function_call.call_resolved_return_value_kind, Some("top"));
        assert_eq!(
            function_call.call_resolved_return_value,
            Some(AbstractCssValueV0::Top)
        );
    }

    #[test]
    fn call_return_ir_keeps_loop_body_returns_top() {
        let source = "@function collect($count) { @for $i from 1 through $count { @return $i; } } .a { width: collect(3); }";
        let report = summarize_scss_call_return_ir(source, StyleDialect::Scss);
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };
        let function_call = report
            .nodes
            .iter()
            .find(|node| node.kind == "functionCall" && node.name.as_deref() == Some("collect"));
        assert!(function_call.is_some());
        let Some(function_call) = function_call else {
            return;
        };

        assert_eq!(report.call_resolved_return_value_count, 1);
        assert_eq!(report.top_call_resolved_return_value_count, 1);
        assert_eq!(function_call.call_resolved_return_value_kind, Some("top"));
        assert_eq!(
            function_call.call_resolved_return_value,
            Some(AbstractCssValueV0::Top)
        );
    }

    #[test]
    fn call_return_ir_caps_recursive_function_call_return_values() {
        let source = "@function again($value) { @return again($value); } .a { width: again(1px); }";
        let report = summarize_scss_call_return_ir(source, StyleDialect::Scss);
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };
        let function_call = report.nodes.iter().find(|node| {
            node.kind == "functionCall"
                && node.name.as_deref() == Some("again")
                && node.containing_declaration_node_key.is_none()
        });
        assert!(function_call.is_some());
        let Some(function_call) = function_call else {
            return;
        };

        assert_eq!(function_call.call_resolved_return_value_kind, Some("top"));
        assert_eq!(
            function_call.call_resolved_return_value,
            Some(AbstractCssValueV0::Top)
        );
    }

    #[test]
    fn call_return_ir_reports_static_scss_if_return_values_in_abstract_domain() {
        let source = "@function gap() { @return if(false, 1px, 2px); } .a { width: gap(); }";
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

        assert_eq!(
            return_node.return_text.as_deref(),
            Some("if(false, 1px, 2px)")
        );
        assert_eq!(report.return_value_count, 1);
        assert_eq!(report.exact_return_value_count, 1);
        assert_eq!(report.raw_return_value_count, 0);
        assert_eq!(return_node.return_value_kind, Some("exact"));
        assert_eq!(
            return_node.return_value,
            Some(AbstractCssValueV0::Exact {
                value: "2px".to_string()
            })
        );
    }

    #[test]
    fn call_return_ir_reports_static_scss_nth_return_values_in_abstract_domain() {
        let source =
            "@function gap() { @return list.nth((1px, 2px, 3px), 2); } .a { width: gap(); }";
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

        assert_eq!(report.return_value_count, 1);
        assert_eq!(report.exact_return_value_count, 1);
        assert_eq!(return_node.return_value_kind, Some("exact"));
        assert_eq!(
            return_node.return_value,
            Some(AbstractCssValueV0::Exact {
                value: "2px".to_string()
            })
        );
    }

    #[test]
    fn call_return_ir_reports_static_scss_map_get_return_values_in_abstract_domain() {
        let source = "@function gap() { @return map-get((default: 2px, dense: 1px), dense); } .a { margin: gap(); }";
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

        assert_eq!(report.return_value_count, 1);
        assert_eq!(report.exact_return_value_count, 1);
        assert_eq!(return_node.return_value_kind, Some("exact"));
        assert_eq!(
            return_node.return_value,
            Some(AbstractCssValueV0::Exact {
                value: "1px".to_string()
            })
        );
    }

    #[test]
    fn call_return_ir_reports_static_scss_collection_search_values_in_abstract_domain() {
        let source =
            "@function item() { @return list.index(red blue green, green); } .a { order: item(); }";
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

        assert_eq!(report.return_value_count, 1);
        assert_eq!(report.exact_return_value_count, 1);
        assert_eq!(return_node.return_value_kind, Some("exact"));
        assert_eq!(
            return_node.return_value,
            Some(AbstractCssValueV0::Exact {
                value: "3".to_string()
            })
        );
    }

    #[test]
    fn call_return_ir_reports_static_scss_list_metadata_values() {
        let source = "@function metadata() { @return if(list.separator((1px, 2px)) == \"comma\" and list.is-bracketed([1px]), 3px, 4px); } .a { margin: metadata(); }";
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

        assert_eq!(report.return_value_count, 1);
        assert_eq!(report.exact_return_value_count, 1);
        assert_eq!(return_node.return_value_kind, Some("exact"));
        assert_eq!(
            return_node.return_value,
            Some(AbstractCssValueV0::Exact {
                value: "3px".to_string()
            })
        );
    }

    #[test]
    fn call_return_ir_reports_static_scss_map_has_key_conditions_in_abstract_domain() {
        let source = "@function gap() { @return if(map.has-key((default: 2px, dense: 1px), dense), list.length((1px, 2px)), 0); } .a { margin: gap(); }";
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

        assert_eq!(report.return_value_count, 1);
        assert_eq!(report.exact_return_value_count, 1);
        assert_eq!(return_node.return_value_kind, Some("exact"));
        assert_eq!(
            return_node.return_value,
            Some(AbstractCssValueV0::Exact {
                value: "2".to_string()
            })
        );
    }

    #[test]
    fn call_return_ir_reports_static_scss_math_return_values_in_abstract_domain() {
        let source = "@function gap() { @return math.div(6px, 3); } .a { margin: gap(); }";
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

        assert_eq!(report.return_value_count, 1);
        assert_eq!(report.exact_return_value_count, 1);
        assert_eq!(return_node.return_value_kind, Some("exact"));
        assert_eq!(
            return_node.return_value,
            Some(AbstractCssValueV0::Exact {
                value: "2px".to_string()
            })
        );
    }

    #[test]
    fn call_return_ir_reports_static_scss_math_alias_returns() {
        let source =
            "@function gap() { @return math.max(1px, math.abs(-3px)); } .a { margin: gap(); }";
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

        assert_eq!(report.return_value_count, 1);
        assert_eq!(report.exact_return_value_count, 1);
        assert_eq!(return_node.return_value_kind, Some("exact"));
        assert_eq!(
            return_node.return_value,
            Some(AbstractCssValueV0::Exact {
                value: "3px".to_string()
            })
        );
    }

    #[test]
    fn call_return_ir_reports_static_scss_extended_math_alias_returns() {
        let source = "@function gap() { @return math.hypot(3px, 4px); } .a { margin: gap(); }";
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

        assert_eq!(report.return_value_count, 1);
        assert_eq!(report.exact_return_value_count, 1);
        assert_eq!(return_node.return_value_kind, Some("exact"));
        assert_eq!(
            return_node.return_value,
            Some(AbstractCssValueV0::Exact {
                value: "5px".to_string()
            })
        );
    }

    #[test]
    fn call_return_ir_reports_static_scss_rounding_alias_returns() {
        let source = "@function gap() { @return math.round(1.5px); } .a { margin: gap(); }";
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

        assert_eq!(report.return_value_count, 1);
        assert_eq!(report.exact_return_value_count, 1);
        assert_eq!(return_node.return_value_kind, Some("exact"));
        assert_eq!(
            return_node.return_value,
            Some(AbstractCssValueV0::Exact {
                value: "2px".to_string()
            })
        );
    }

    #[test]
    fn call_return_ir_reduces_nested_static_list_conditions_in_order() {
        let source = "@function count() { @return list.length(if(false, 1px 2px, 3px 4px 5px)); } .a { z-index: count(); }";
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

        assert_eq!(report.return_value_count, 1);
        assert_eq!(report.exact_return_value_count, 1);
        assert_eq!(return_node.return_value_kind, Some("exact"));
        assert_eq!(
            return_node.return_value,
            Some(AbstractCssValueV0::Exact {
                value: "3".to_string()
            })
        );
    }

    #[test]
    fn call_return_ir_reports_static_scss_unitless_branch_returns() {
        let source = "@function gap() { @return if(unitless(2px), 1px, math.div(6px, 3)); } .a { margin: gap(); }";
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

        assert_eq!(report.return_value_count, 1);
        assert_eq!(report.exact_return_value_count, 1);
        assert_eq!(return_node.return_value_kind, Some("exact"));
        assert_eq!(
            return_node.return_value,
            Some(AbstractCssValueV0::Exact {
                value: "2px".to_string()
            })
        );
    }

    #[test]
    fn call_return_ir_reports_static_scss_unit_compatibility_returns() {
        let source = "@function unit-name() { @return if(math.compatible(1px, 2px) and not comparable(1, 1px), math.unit(4px), \"bad\"); } .a { content: unit-name(); }";
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

        assert_eq!(report.return_value_count, 1);
        assert_eq!(report.raw_return_value_count, 1);
        assert_eq!(return_node.return_value_kind, Some("raw"));
        assert_eq!(
            return_node.return_value,
            Some(AbstractCssValueV0::Raw {
                value: "\"px\"".to_string()
            })
        );
    }

    #[test]
    fn call_return_ir_reports_static_scss_if_argument_values_in_abstract_domain() {
        let source =
            "@function gap($value) { @return $value; } .a { width: gap(if(false, 1px, 2px)); }";
        let report = summarize_scss_call_return_ir(source, StyleDialect::Scss);
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };
        let function_call = report
            .nodes
            .iter()
            .find(|node| node.kind == "functionCall" && node.name.as_deref() == Some("gap"));
        assert!(function_call.is_some());
        let Some(function_call) = function_call else {
            return;
        };

        assert_eq!(function_call.argument_values.len(), 1);
        assert_eq!(function_call.argument_values[0].text, "if(false, 1px, 2px)");
        assert_eq!(function_call.argument_values[0].value_kind, "exact");
        assert_eq!(
            function_call.argument_values[0].value,
            AbstractCssValueV0::Exact {
                value: "2px".to_string()
            }
        );
    }

    #[test]
    fn call_return_ir_reports_static_scss_inequality_argument_values_in_abstract_domain() {
        let source = "@function gap($value) { @return $value; } .a { width: gap(if(1px != 2px, 1px, 2px)); }";
        let report = summarize_scss_call_return_ir(source, StyleDialect::Scss);
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };
        let function_call = report
            .nodes
            .iter()
            .find(|node| node.kind == "functionCall" && node.name.as_deref() == Some("gap"));
        assert!(function_call.is_some());
        let Some(function_call) = function_call else {
            return;
        };

        assert_eq!(function_call.argument_values.len(), 1);
        assert_eq!(
            function_call.argument_values[0].text,
            "if(1px != 2px, 1px, 2px)"
        );
        assert_eq!(function_call.argument_values[0].value_kind, "exact");
        assert_eq!(
            function_call.argument_values[0].value,
            AbstractCssValueV0::Exact {
                value: "1px".to_string()
            }
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
    fn call_return_ir_resolves_hyphen_underscore_equivalent_mixin_edges() {
        let source =
            "@mixin tone_color($color) { color: $color; } .a { @include tone-color(red); }";
        let report = summarize_scss_call_return_ir(source, StyleDialect::Scss);
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.declaration_node_count, 1);
        assert_eq!(report.call_node_count, 1);
        assert!(report.edges.iter().any(|edge| {
            edge.kind == "mixinCall" && !edge.recursive && !edge.capped_by_recursion_cap
        }));
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
