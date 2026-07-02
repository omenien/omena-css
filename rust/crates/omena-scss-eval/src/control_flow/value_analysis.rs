use std::collections::BTreeMap;

use omena_abstract_value::{
    AbstractCssTypedComparisonOperatorV0, AbstractCssTypedValueV0, AbstractCssValueV0,
    MAX_FLOW_ANALYSIS_ITERATIONS, abstract_css_typed_value_kind_label,
    abstract_css_value_from_text,
};
use omena_cascade::{
    StaticSupportsAssumptionV0, StaticSupportsEvalVerdictV0, evaluate_static_supports_condition,
};
use omena_parser::StyleDialect;

use crate::abstract_css_value_kind;

use super::{
    ScssControlFlowAnalysisNode,
    blocks::{self, scss_else_if_header_condition},
    build_scss_control_flow_graph, dialect_label,
    header_values::{scss_header_value, scss_header_value_with_bindings},
    lexical::{LexicalScssBindings, collect_lexical_scss_bindings},
    loop_values::{
        loop_carried_binding_values, loop_carried_value, while_loop_carried_binding_values,
    },
    model::{
        OmenaScssEvalControlFlowBlockIdV0, OmenaScssEvalControlFlowBlockV0,
        OmenaScssEvalControlFlowGraphV0, OmenaScssEvalControlFlowValueAnalysisV0,
        OmenaScssEvalControlFlowValueBlockV0, OmenaScssEvalControlFlowWideningWitnessV0,
        OmenaScssEvalTypedValueKindCountV0, OmenaScssEvalTypedValueLatticeWitnessV0,
    },
    oracle_corpus::scss_control_flow_oracle_corpus_fixtures,
    transfer::{
        ScssControlFlowBindingValue, ScssControlFlowTransfer, run_scss_control_flow_fixpoint,
        scss_static_truthiness_label,
    },
    typed_truthiness::{
        ScssTruthinessConsumer, TYPED_PRUNE_CONSUMER_ENABLED, production_truthiness_consumer,
        typed_comparison_truthiness, typed_truthiness_label,
    },
    variables::insert_static_scss_binding,
};

const SCSS_CONTROL_FLOW_WIDENING_WITNESS_NODE_COUNT: usize = MAX_FLOW_ANALYSIS_ITERATIONS + 8;
const TYPED_VALUE_LATTICE_WITNESS_VALUES: &[&str] =
    &["0px", "50%", "red", "true", "\"hello\"", "var(--gap)"];
const TYPED_VALUE_LATTICE_WITNESS_COMPARISONS: &[(
    &str,
    AbstractCssTypedComparisonOperatorV0,
    &str,
)] = &[
    ("1in", AbstractCssTypedComparisonOperatorV0::Equal, "96px"),
    (
        "2px",
        AbstractCssTypedComparisonOperatorV0::GreaterThan,
        "1px",
    ),
    ("1em", AbstractCssTypedComparisonOperatorV0::Equal, "16px"),
];

#[derive(Debug, Clone, Copy, Default)]
struct TypedPruningPreservationSummary {
    divergence_count: usize,
    corpus_fixture_count: usize,
    typed_decided_fixture_count: usize,
}

pub fn analyze_scss_control_flow_values(
    source: &str,
    dialect: StyleDialect,
) -> Option<OmenaScssEvalControlFlowValueAnalysisV0> {
    analyze_scss_control_flow_values_with_initial_bindings(source, dialect, &BTreeMap::new())
}

pub(crate) fn analyze_scss_control_flow_values_with_initial_bindings(
    source: &str,
    dialect: StyleDialect,
    initial_bindings: &BTreeMap<String, AbstractCssValueV0>,
) -> Option<OmenaScssEvalControlFlowValueAnalysisV0> {
    analyze_scss_control_flow_values_with_truthiness_consumer(
        source,
        dialect,
        initial_bindings,
        production_truthiness_consumer(),
    )
}

pub(super) fn analyze_scss_control_flow_values_with_truthiness_consumer(
    source: &str,
    dialect: StyleDialect,
    initial_bindings: &BTreeMap<String, AbstractCssValueV0>,
    truthiness_consumer: ScssTruthinessConsumer,
) -> Option<OmenaScssEvalControlFlowValueAnalysisV0> {
    if !matches!(
        dialect,
        StyleDialect::Css | StyleDialect::Scss | StyleDialect::Sass
    ) {
        return None;
    }
    let graph = build_scss_control_flow_graph(source, dialect)?;
    let predecessor_indices_by_block_id = control_flow_predecessor_indices_by_block_id(&graph);
    let mut lexical_bindings = collect_lexical_scss_bindings(source, dialect);
    for (name, value) in initial_bindings {
        lexical_bindings.push_root_binding(name.as_str(), value.clone());
    }
    let graph_block_payloads = graph
        .blocks
        .iter()
        .map(|block| block.block.clone())
        .collect::<Vec<_>>();
    let nodes = graph
        .blocks
        .iter()
        .enumerate()
        .map(|(index, graph_block)| {
            let block = &graph_block.block;
            let predecessor_indices = predecessor_indices_by_block_id
                .get(&graph_block.id)
                .cloned()
                .unwrap_or_default();
            ScssControlFlowAnalysisNode {
                block: block.clone(),
                predecessor_indices,
                transfer: control_flow_transfer_for_block(
                    source,
                    dialect,
                    block,
                    &graph_block_payloads[..index],
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
            let transfer_truthiness = node.transfer.transfer_truthiness(truthiness_consumer);
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
        flat_css_cfg_built: true,
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
                        typed: None,
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

pub fn summarize_typed_value_lattice_witness() -> OmenaScssEvalTypedValueLatticeWitnessV0 {
    let values = TYPED_VALUE_LATTICE_WITNESS_VALUES
        .iter()
        .map(|value| abstract_css_value_from_text(value))
        .collect::<Vec<_>>();
    let sample_value_count = values.len();
    let typed_payload_count = values
        .iter()
        .filter(|value| abstract_css_typed_payload(value).is_some())
        .count();
    let raw_value_count = values
        .iter()
        .filter(|value| matches!(value, AbstractCssValueV0::Raw { .. }))
        .count();
    let untyped_exact_or_finite_count = values
        .iter()
        .filter(|value| {
            matches!(
                value,
                AbstractCssValueV0::Exact { typed: None, .. }
                    | AbstractCssValueV0::FiniteSet { typed: None, .. }
            )
        })
        .count();
    let typed_coverage_basis_points = typed_payload_count
        .checked_mul(10_000)
        .and_then(|value| value.checked_div(sample_value_count))
        .unwrap_or(0);
    let mut type_kind_counts: Vec<OmenaScssEvalTypedValueKindCountV0> = Vec::new();
    for value in values.iter().filter_map(abstract_css_typed_payload) {
        let kind = abstract_css_typed_value_kind_label(value);
        if let Some(existing) = type_kind_counts.iter_mut().find(|entry| entry.kind == kind) {
            existing.count += 1;
        } else {
            type_kind_counts.push(OmenaScssEvalTypedValueKindCountV0 { kind, count: 1 });
        }
    }
    type_kind_counts.sort_by_key(|entry| entry.kind);
    let typed_advisory_comparisons = TYPED_VALUE_LATTICE_WITNESS_COMPARISONS
        .iter()
        .filter_map(|(left, operator, right)| {
            typed_comparison_truthiness(
                &abstract_css_value_from_text(left),
                *operator,
                &abstract_css_value_from_text(right),
            )
        })
        .collect::<Vec<_>>();
    let typed_advisory_comparison_count = typed_advisory_comparisons.len();
    let typed_advisory_true_count = typed_advisory_comparisons
        .iter()
        .filter(|comparison| **comparison)
        .count();
    let preservation = summarize_typed_pruning_preservation();

    OmenaScssEvalTypedValueLatticeWitnessV0 {
        schema_version: "1",
        product: "omena-scss-eval.typed-value-lattice-witness",
        mode: "typedPayloadConsumerPreservationWitness",
        value_type: "AbstractCssValueV0",
        payload_type: "AbstractCssTypedValueV0",
        policy: "typedPayloadConsumerStringOutputPreserved",
        sample_value_count,
        typed_payload_count,
        raw_value_count,
        untyped_exact_or_finite_count,
        typed_coverage_basis_points,
        typed_advisory_comparison_count,
        typed_advisory_true_count,
        divergence_count: preservation.divergence_count,
        corpus_fixture_count: preservation.corpus_fixture_count,
        typed_decided_fixture_count: preservation.typed_decided_fixture_count,
        typed_prune_consumer_enabled: TYPED_PRUNE_CONSUMER_ENABLED,
        type_kind_counts,
    }
}

fn summarize_typed_pruning_preservation() -> TypedPruningPreservationSummary {
    let mut summary = TypedPruningPreservationSummary::default();
    let initial_bindings = BTreeMap::new();
    for fixture in scss_control_flow_oracle_corpus_fixtures() {
        summary.corpus_fixture_count += 1;
        let string_output = analyze_scss_control_flow_values_with_truthiness_consumer(
            fixture.source,
            fixture.dialect,
            &initial_bindings,
            ScssTruthinessConsumer::StringLattice,
        );
        let typed_output = analyze_scss_control_flow_values_with_truthiness_consumer(
            fixture.source,
            fixture.dialect,
            &initial_bindings,
            ScssTruthinessConsumer::TypedPayload,
        );
        if serialized_value_analysis_bytes(&string_output)
            != serialized_value_analysis_bytes(&typed_output)
        {
            summary.divergence_count += 1;
        }
        if analysis_has_typed_truthiness_decision(typed_output.as_ref()) {
            summary.typed_decided_fixture_count += 1;
        }
    }
    summary
}

fn serialized_value_analysis_bytes(
    analysis: &Option<OmenaScssEvalControlFlowValueAnalysisV0>,
) -> Vec<u8> {
    match serde_json::to_vec(analysis) {
        Ok(bytes) => bytes,
        Err(error) => error.to_string().into_bytes(),
    }
}

fn analysis_has_typed_truthiness_decision(
    analysis: Option<&OmenaScssEvalControlFlowValueAnalysisV0>,
) -> bool {
    analysis.is_some_and(|analysis| {
        analysis.blocks.iter().any(|block| {
            block
                .transfer_value
                .as_ref()
                .and_then(typed_truthiness_label)
                .is_some()
        })
    })
}

fn abstract_css_typed_payload(value: &AbstractCssValueV0) -> Option<&AbstractCssTypedValueV0> {
    match value {
        AbstractCssValueV0::Exact { typed, .. } | AbstractCssValueV0::FiniteSet { typed, .. } => {
            typed.as_deref()
        }
        AbstractCssValueV0::Bottom | AbstractCssValueV0::Raw { .. } | AbstractCssValueV0::Top => {
            None
        }
    }
}

fn control_flow_predecessor_indices_by_block_id(
    graph: &OmenaScssEvalControlFlowGraphV0,
) -> BTreeMap<OmenaScssEvalControlFlowBlockIdV0, Vec<usize>> {
    let block_index_by_id = graph
        .blocks
        .iter()
        .enumerate()
        .map(|(index, block)| (block.id, index))
        .collect::<BTreeMap<_, _>>();
    let mut predecessor_indices_by_id = graph
        .blocks
        .iter()
        .map(|block| (block.id, Vec::new()))
        .collect::<BTreeMap<_, _>>();
    for edge in &graph.edges {
        let Some(target_block_id) = edge.target_block_id else {
            continue;
        };
        let Some(source_index) = block_index_by_id.get(&edge.source_block_id) else {
            continue;
        };
        predecessor_indices_by_id
            .entry(target_block_id)
            .or_default()
            .push(*source_index);
    }
    for predecessor_indices in predecessor_indices_by_id.values_mut() {
        predecessor_indices.sort_unstable();
        predecessor_indices.dedup();
    }
    predecessor_indices_by_id
}

fn control_flow_transfer_for_block(
    source: &str,
    dialect: StyleDialect,
    block: &OmenaScssEvalControlFlowBlockV0,
    previous_blocks: &[OmenaScssEvalControlFlowBlockV0],
    lexical_bindings: &LexicalScssBindings,
) -> ScssControlFlowTransfer {
    let contextual_bindings =
        contextual_control_flow_bindings(source, block, previous_blocks, lexical_bindings);
    match block.at_rule_name.to_ascii_lowercase().as_str() {
        "@when" if dialect == StyleDialect::Css => ScssControlFlowTransfer::BranchCondition {
            value: native_css_when_header_value(block.header_text.as_str()),
        },
        "@else" if dialect == StyleDialect::Css => ScssControlFlowTransfer::BranchCondition {
            value: AbstractCssValueV0::Top,
        },
        "if()" if dialect == StyleDialect::Css => ScssControlFlowTransfer::BranchCondition {
            value: native_css_if_function_header_value(block.header_text.as_str()),
        },
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

fn native_css_when_header_value(header: &str) -> AbstractCssValueV0 {
    if let Some(inner) = extract_named_function_inner(header, "supports") {
        let normalized_condition = normalize_supports_condition_for_native_css(inner);
        let witness = evaluate_static_supports_condition(
            &normalized_condition,
            StaticSupportsAssumptionV0::ModernBrowser,
        );
        return match witness.verdict {
            StaticSupportsEvalVerdictV0::AlwaysTrue => abstract_css_value_from_text("true"),
            StaticSupportsEvalVerdictV0::AlwaysFalse => abstract_css_value_from_text("false"),
            StaticSupportsEvalVerdictV0::Unknown => AbstractCssValueV0::Top,
        };
    }
    AbstractCssValueV0::Top
}

fn native_css_if_function_header_value(header: &str) -> AbstractCssValueV0 {
    let Some(condition) = native_css_if_function_first_condition(header) else {
        return AbstractCssValueV0::Top;
    };
    if condition.eq_ignore_ascii_case("else") {
        return abstract_css_value_from_text("true");
    }
    if let Some(inner) = extract_named_function_inner(condition, "supports") {
        let normalized_condition = normalize_supports_condition_for_native_css(inner);
        let witness = evaluate_static_supports_condition(
            &normalized_condition,
            StaticSupportsAssumptionV0::ModernBrowser,
        );
        return match witness.verdict {
            StaticSupportsEvalVerdictV0::AlwaysTrue => abstract_css_value_from_text("true"),
            StaticSupportsEvalVerdictV0::AlwaysFalse => abstract_css_value_from_text("false"),
            StaticSupportsEvalVerdictV0::Unknown => AbstractCssValueV0::Top,
        };
    }
    AbstractCssValueV0::Top
}

fn native_css_if_function_first_condition(header: &str) -> Option<&str> {
    let condition_end = first_top_level_colon_byte_index(header)?;
    header
        .get(..condition_end)
        .map(str::trim)
        .filter(|condition| !condition.is_empty())
}

fn first_top_level_colon_byte_index(value: &str) -> Option<usize> {
    let mut paren_depth = 0usize;
    for (index, ch) in value.char_indices() {
        match ch {
            '(' => paren_depth = paren_depth.saturating_add(1),
            ')' => paren_depth = paren_depth.saturating_sub(1),
            ':' if paren_depth == 0 => return Some(index),
            ';' if paren_depth == 0 => return None,
            _ => {}
        }
    }
    None
}

fn extract_named_function_inner<'a>(condition: &'a str, name: &str) -> Option<&'a str> {
    let trimmed = condition.trim();
    let prefix = trimmed.get(..name.len())?;
    if !prefix.eq_ignore_ascii_case(name) {
        return None;
    }
    let rest = trimmed[name.len()..].trim_start();
    if !rest.starts_with('(') {
        return None;
    }
    let close_index = matching_closing_paren_byte_index(rest)?;
    rest[close_index + 1..]
        .trim()
        .is_empty()
        .then_some(&rest[1..close_index])
}

fn matching_closing_paren_byte_index(value: &str) -> Option<usize> {
    let mut depth = 0usize;
    for (index, ch) in value.char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => {
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

fn normalize_supports_condition_for_native_css(condition: &str) -> String {
    let trimmed = condition.trim();
    if trimmed.starts_with('(') {
        trimmed.to_string()
    } else {
        format!("({trimmed})")
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
