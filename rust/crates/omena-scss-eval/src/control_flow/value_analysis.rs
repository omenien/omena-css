use std::collections::BTreeMap;

use omena_abstract_value::{
    AbstractCssValueV0, MAX_FLOW_ANALYSIS_ITERATIONS, abstract_css_value_from_text,
};
use omena_parser::StyleDialect;

use crate::abstract_css_value_kind;

use super::{
    ScssControlFlowAnalysisNode,
    blocks::{self, scss_else_if_header_condition},
    dialect_label,
    header_values::{scss_header_value, scss_header_value_with_bindings},
    lexical::{LexicalScssBindings, collect_lexical_scss_bindings},
    loop_values::{
        loop_carried_binding_values, loop_carried_value, while_loop_carried_binding_values,
    },
    model::{
        OmenaScssEvalControlFlowBlockV0, OmenaScssEvalControlFlowValueAnalysisV0,
        OmenaScssEvalControlFlowValueBlockV0, OmenaScssEvalControlFlowWideningWitnessV0,
    },
    summarize_scss_control_flow_ir,
    transfer::{
        ScssControlFlowBindingValue, ScssControlFlowTransfer, run_scss_control_flow_fixpoint,
        scss_static_truthiness_label,
    },
    variables::insert_static_scss_binding,
};

const SCSS_CONTROL_FLOW_WIDENING_WITNESS_NODE_COUNT: usize = MAX_FLOW_ANALYSIS_ITERATIONS + 8;

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
                    source,
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

pub(crate) fn summarize_scss_control_flow_widening_witness()
-> OmenaScssEvalControlFlowWideningWitnessV0 {
    let nodes = (0..SCSS_CONTROL_FLOW_WIDENING_WITNESS_NODE_COUNT)
        .map(|index| {
            let source_span_start = index;
            let source_span_end = index + 1;
            let block = OmenaScssEvalControlFlowBlockV0 {
                node_key: blocks::scss_eval_stable_node_key(
                    "scss-control-widening-witness",
                    "loop",
                    source_span_start,
                    source_span_end,
                ),
                kind: "loop",
                at_rule_name: "@while".to_string(),
                header_text: "$i < $limit".to_string(),
                source_span_start,
                source_span_end,
                successor_count: 1,
                has_back_edge: true,
            };
            let predecessor_indices = (index + 1 < SCSS_CONTROL_FLOW_WIDENING_WITNESS_NODE_COUNT)
                .then_some(index + 1)
                .into_iter()
                .collect::<Vec<_>>();
            let transfer = if index + 1 == SCSS_CONTROL_FLOW_WIDENING_WITNESS_NODE_COUNT {
                ScssControlFlowTransfer::LoopCondition {
                    bindings: Vec::new(),
                    value: AbstractCssValueV0::Exact {
                        value: "1px".to_string(),
                    },
                }
            } else {
                ScssControlFlowTransfer::PassThrough
            };
            ScssControlFlowAnalysisNode {
                block,
                predecessor_indices,
                transfer,
            }
        })
        .collect::<Vec<_>>();
    let fixpoint = run_scss_control_flow_fixpoint(&nodes);
    let output_top_count = fixpoint
        .output_values
        .iter()
        .filter(|value| matches!(value, AbstractCssValueV0::Top))
        .count();

    OmenaScssEvalControlFlowWideningWitnessV0 {
        schema_version: "0",
        product: "omena-scss-eval.control-flow-widening-witness",
        mode: "oracleOnly",
        value_type: "AbstractCssValueV0",
        policy: "nonConvergedOutputsWidenToTop",
        max_iterations: MAX_FLOW_ANALYSIS_ITERATIONS,
        node_count: nodes.len(),
        converged: fixpoint.converged,
        iteration_count: fixpoint.iteration_count,
        widened_to_top_count: fixpoint.widened_to_top_count,
        output_top_count,
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
    source: &str,
    block: &OmenaScssEvalControlFlowBlockV0,
    previous_blocks: &[OmenaScssEvalControlFlowBlockV0],
    lexical_bindings: &LexicalScssBindings,
) -> ScssControlFlowTransfer {
    let contextual_bindings =
        contextual_control_flow_bindings(source, block, previous_blocks, lexical_bindings);
    match block.at_rule_name.to_ascii_lowercase().as_str() {
        "@if" => ScssControlFlowTransfer::BranchCondition {
            value: scss_header_value_with_bindings(
                block.header_text.as_str(),
                lexical_bindings,
                block.source_span_start,
                &contextual_bindings,
            ),
        },
        "@while" => {
            let bindings = while_loop_carried_binding_values(source, block, &contextual_bindings);
            let value = if bindings.is_empty() {
                scss_header_value_with_bindings(
                    block.header_text.as_str(),
                    lexical_bindings,
                    block.source_span_start,
                    &contextual_bindings,
                )
            } else {
                AbstractCssValueV0::Top
            };
            ScssControlFlowTransfer::LoopCondition { bindings, value }
        }
        "@for" | "@each" => {
            let bindings =
                loop_carried_binding_values(block.header_text.as_str(), &contextual_bindings);
            ScssControlFlowTransfer::LoopCarried {
                bindings,
                value: loop_carried_value(block.header_text.as_str(), &contextual_bindings),
            }
        }
        "@else" => {
            control_flow_transfer_for_else_block(source, block, previous_blocks, lexical_bindings)
        }
        _ => ScssControlFlowTransfer::PassThrough,
    }
}

fn contextual_control_flow_bindings(
    source: &str,
    block: &OmenaScssEvalControlFlowBlockV0,
    previous_blocks: &[OmenaScssEvalControlFlowBlockV0],
    lexical_bindings: &LexicalScssBindings,
) -> BTreeMap<String, AbstractCssValueV0> {
    let mut visible_bindings = lexical_bindings.visible_at(block.source_span_start);
    for previous in previous_blocks.iter().filter(|previous| {
        previous.kind == "loop"
            && previous.source_span_start < block.source_span_start
            && block.source_span_start < previous.source_span_end
    }) {
        for binding in control_flow_loop_carried_binding_values(source, previous, &visible_bindings)
        {
            insert_static_scss_binding(
                &mut visible_bindings,
                binding.name.as_str(),
                binding.value.clone(),
            );
        }
    }
    visible_bindings
}

fn control_flow_loop_carried_binding_values(
    source: &str,
    block: &OmenaScssEvalControlFlowBlockV0,
    lexical_bindings: &BTreeMap<String, AbstractCssValueV0>,
) -> Vec<ScssControlFlowBindingValue> {
    if block.at_rule_name.eq_ignore_ascii_case("@while") {
        while_loop_carried_binding_values(source, block, lexical_bindings)
    } else {
        loop_carried_binding_values(block.header_text.as_str(), lexical_bindings)
    }
}

fn control_flow_transfer_for_else_block(
    source: &str,
    block: &OmenaScssEvalControlFlowBlockV0,
    previous_blocks: &[OmenaScssEvalControlFlowBlockV0],
    lexical_bindings: &LexicalScssBindings,
) -> ScssControlFlowTransfer {
    if let Some(condition) = scss_else_if_header_condition(block.header_text.as_str()) {
        let contextual_bindings =
            contextual_control_flow_bindings(source, block, previous_blocks, lexical_bindings);
        return ScssControlFlowTransfer::BranchCondition {
            value: scss_header_value_with_bindings(
                condition,
                lexical_bindings,
                block.source_span_start,
                &contextual_bindings,
            ),
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ScssBranchConditionHeader<'a> {
    header: &'a str,
    source_span_start: usize,
}

fn previous_scss_branch_condition_headers(
    previous_blocks: &[OmenaScssEvalControlFlowBlockV0],
) -> Vec<ScssBranchConditionHeader<'_>> {
    let mut headers = Vec::new();
    for block in previous_blocks.iter().rev() {
        if block.at_rule_name.eq_ignore_ascii_case("@else") {
            let Some(condition) = scss_else_if_header_condition(block.header_text.as_str()) else {
                break;
            };
            headers.push(ScssBranchConditionHeader {
                header: condition,
                source_span_start: block.source_span_start,
            });
            continue;
        }
        if block.at_rule_name.eq_ignore_ascii_case("@if") {
            headers.push(ScssBranchConditionHeader {
                header: block.header_text.as_str(),
                source_span_start: block.source_span_start,
            });
        }
        break;
    }
    headers.reverse();
    headers
}

fn inverted_scss_branch_chain_value(
    headers: &[ScssBranchConditionHeader<'_>],
    lexical_bindings: &LexicalScssBindings,
) -> AbstractCssValueV0 {
    let mut saw_unknown = false;
    for header in headers {
        let value = scss_header_value(header.header, lexical_bindings, header.source_span_start);
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
