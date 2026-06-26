use omena_abstract_value::AbstractCssValueV0;
use omena_parser::{StyleDialect, collect_style_facts, lex, parse};

use super::{
    SCSS_CALL_RETURN_RECURSION_LIMIT,
    blocks::{control_flow_block_from_token, control_flow_blocks_from_cst},
    call_resolution::max_call_stack_depth_observed,
    call_return_nodes::{
        call_return_node_from_candidate, call_return_node_is_call, call_return_node_is_declaration,
        stamp_containing_declarations, stamp_containing_declarations_from_cst,
    },
    call_return_resolution::{
        build_call_return_edges, stamp_call_resolved_return_values, stamp_contextual_return_values,
    },
    dialect_label,
    lexical::collect_scss_global_variable_declarations,
    model::{
        OmenaScssEvalCallReturnIrSummaryV0, OmenaScssEvalControlFlowBlockV0,
        OmenaScssEvalControlFlowIrSummaryV0,
    },
    return_candidates::{collect_scss_return_candidates, collect_scss_return_candidates_from_cst},
    symbol_candidates::call_return_candidate_from_sass_symbol,
};

pub fn summarize_scss_control_flow_ir(
    source: &str,
    dialect: StyleDialect,
) -> Option<OmenaScssEvalControlFlowIrSummaryV0> {
    if !matches!(
        dialect,
        StyleDialect::Css | StyleDialect::Scss | StyleDialect::Sass
    ) {
        return None;
    }
    let parsed = parse(source, dialect);
    let syntax = parsed.syntax();
    let blocks = control_flow_blocks_from_cst(source, &syntax, dialect);
    summarize_scss_control_flow_ir_from_blocks(dialect, blocks)
}

#[doc(hidden)]
pub fn summarize_scss_control_flow_ir_scanner_oracle(
    source: &str,
    dialect: StyleDialect,
) -> Option<OmenaScssEvalControlFlowIrSummaryV0> {
    if !matches!(
        dialect,
        StyleDialect::Css | StyleDialect::Scss | StyleDialect::Sass
    ) {
        return None;
    }
    let lexed = lex(source, dialect);
    let tokens = lexed.tokens();
    let blocks = tokens
        .iter()
        .enumerate()
        .filter_map(|(index, token)| {
            control_flow_block_from_token(source, tokens, index, token, dialect)
        })
        .collect::<Vec<_>>();
    summarize_scss_control_flow_ir_from_blocks(dialect, blocks)
}

fn summarize_scss_control_flow_ir_from_blocks(
    dialect: StyleDialect,
    blocks: Vec<OmenaScssEvalControlFlowBlockV0>,
) -> Option<OmenaScssEvalControlFlowIrSummaryV0> {
    if dialect == StyleDialect::Css && blocks.is_empty() {
        return None;
    }
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
        flat_css_cfg_built: true,
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
    summarize_scss_call_return_ir_with_path(source, dialect, false)
}

#[doc(hidden)]
pub fn summarize_scss_call_return_ir_scanner_oracle(
    source: &str,
    dialect: StyleDialect,
) -> Option<OmenaScssEvalCallReturnIrSummaryV0> {
    summarize_scss_call_return_ir_with_path(source, dialect, true)
}

fn summarize_scss_call_return_ir_with_path(
    source: &str,
    dialect: StyleDialect,
    use_legacy_path: bool,
) -> Option<OmenaScssEvalCallReturnIrSummaryV0> {
    if !matches!(dialect, StyleDialect::Scss | StyleDialect::Sass) {
        return None;
    }

    let facts = collect_style_facts(source, dialect);
    let lexed = use_legacy_path.then(|| lex(source, dialect));
    let tokens = lexed.as_ref().map(|lexed| lexed.tokens());
    let parsed = (!use_legacy_path).then(|| parse(source, dialect));
    let syntax = parsed.as_ref().map(|parsed| parsed.syntax());
    let global_variable_declarations = collect_scss_global_variable_declarations(source, dialect);
    let mut candidates = facts
        .sass_symbols
        .iter()
        .filter_map(|symbol| {
            call_return_candidate_from_sass_symbol(source, tokens, symbol, syntax.as_ref())
        })
        .chain(if let Some(syntax) = syntax.as_ref() {
            collect_scss_return_candidates_from_cst(source, syntax, dialect)
        } else {
            collect_scss_return_candidates(source, tokens.unwrap_or(&[]))
        })
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
    if let Some(syntax) = syntax.as_ref() {
        stamp_containing_declarations_from_cst(&mut nodes, syntax);
    } else {
        stamp_containing_declarations(&mut nodes, tokens.unwrap_or(&[]));
    }
    stamp_contextual_return_values(&mut nodes, &global_variable_declarations);

    let edges = build_call_return_edges(&nodes);
    stamp_call_resolved_return_values(&mut nodes, &edges, &global_variable_declarations);
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
