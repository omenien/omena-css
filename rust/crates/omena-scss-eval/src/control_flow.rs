use std::collections::BTreeMap;

use omena_abstract_value::{
    AbstractCssValueV0, BoundedJoinFixpointNodeV0, MAX_FLOW_ANALYSIS_ITERATIONS,
    abstract_css_value_from_text, analyze_bounded_join_fixpoint, join_abstract_css_values,
};
use omena_parser::{
    LexedToken, ParsedSassSymbolFact, ParsedSassSymbolFactKind, ParsedVariableFact,
    ParsedVariableFactKind, StyleDialect, collect_style_facts, lex,
};
use omena_syntax::SyntaxKind;
use omena_transform_cst::StableNodeKeyV0;

use crate::{
    abstract_css_value_kind,
    scss_metadata::reduce_static_scss_metadata_with_context,
    static_loop_frames::parse_static_scss_each_loop_binding_frames,
    value_eval::{reduce_static_scss_value, static_scss_literal_truthiness},
};

mod analysis_model;
mod arguments;
mod model;
mod oracle_corpus;
mod tokens;

use analysis_model::{
    ScssBranchBlock, ScssCallBoundReturnActivity, ScssCallLocalBindingScope,
    ScssCallReturnCandidate, ScssCallReturnResolutionContext, ScssGlobalVariableDeclaration,
    ScssLoopReturnResolution, ScssReturnCondition,
};
use arguments::{
    scss_named_value_from_text, split_scss_call_arguments, static_scss_argument_abstract_value,
};
use tokens::{
    declaration_end_token_index, matching_right_brace_token_index,
    matching_right_paren_token_index, next_non_trivia_token_index, token_range_end,
    token_range_start, tokens_between_are_trivia,
};

pub use model::{
    OmenaScssEvalCallArgumentValueV0, OmenaScssEvalCallLocalBindingV0,
    OmenaScssEvalCallParameterValueV0, OmenaScssEvalCallReturnEdgeV0,
    OmenaScssEvalCallReturnIrSummaryV0, OmenaScssEvalCallReturnNodeV0,
    OmenaScssEvalControlFlowBindingValueV0, OmenaScssEvalControlFlowBlockV0,
    OmenaScssEvalControlFlowIrSummaryV0, OmenaScssEvalControlFlowValueAnalysisV0,
    OmenaScssEvalControlFlowValueBlockV0,
};
pub use oracle_corpus::{
    OmenaScssEvalControlFlowOracleCorpusFixtureReportV0,
    OmenaScssEvalControlFlowOracleCorpusReportV0, summarize_scss_control_flow_oracle_corpus,
};

const SCSS_CALL_RETURN_RECURSION_LIMIT: usize = 32;

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
    let global_variable_declarations =
        collect_scss_global_variable_declarations(source, &facts.variables);
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
        return_inside_loop_control_flow: false,
        return_loop_header_text: None,
        return_loop_header_texts: Vec::new(),
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
            let return_inside_loop_control_flow =
                scss_return_is_inside_loop_control_flow(tokens, index);
            let return_loop_header_texts = scss_return_loop_header_texts(source, tokens, index);
            let return_loop_header_text = return_loop_header_texts.last().cloned();
            let return_value = if return_inside_loop_control_flow {
                Some(AbstractCssValueV0::Top)
            } else {
                return_text
                    .as_deref()
                    .map(static_scss_return_abstract_value)
            };
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
                return_inside_loop_control_flow,
                return_loop_header_text,
                return_loop_header_texts,
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

fn scss_return_is_inside_loop_control_flow(tokens: &[LexedToken], return_index: usize) -> bool {
    !enclosing_scss_loop_blocks(tokens, return_index).is_empty()
}

fn scss_return_loop_header_texts(
    source: &str,
    tokens: &[LexedToken],
    return_index: usize,
) -> Vec<String> {
    enclosing_scss_loop_blocks(tokens, return_index)
        .into_iter()
        .map(|block| control_flow_header_text(source, tokens, block.at_rule_index))
        .filter(|header| !header.is_empty())
        .collect()
}

fn enclosing_scss_loop_blocks(tokens: &[LexedToken], return_index: usize) -> Vec<ScssBranchBlock> {
    let mut blocks = tokens
        .iter()
        .enumerate()
        .filter_map(|(index, token)| {
            if token.kind != SyntaxKind::AtKeyword
                || !matches!(
                    token.text.to_ascii_lowercase().as_str(),
                    "@for" | "@each" | "@while"
                )
            {
                return None;
            }
            let body_start_index = tokens
                .iter()
                .enumerate()
                .skip(index + 1)
                .find(|(_, candidate)| candidate.kind == SyntaxKind::LeftBrace)
                .map(|(candidate_index, _)| candidate_index)?;
            let body_end_index = matching_right_brace_token_index(tokens, body_start_index)?;
            (body_start_index < return_index && return_index < body_end_index).then(|| {
                ScssBranchBlock {
                    at_rule_index: index,
                    at_rule_name: token.text.to_ascii_lowercase(),
                    condition_text: None,
                    body_start_index,
                    body_end_index,
                }
            })
        })
        .collect::<Vec<_>>();
    blocks.sort_by(|left, right| {
        let left_span = left.body_end_index.saturating_sub(left.body_start_index);
        let right_span = right.body_end_index.saturating_sub(right.body_start_index);
        right_span.cmp(&left_span)
    });
    blocks
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

fn static_scss_return_abstract_value_with_context(
    value: &str,
    position: usize,
    nodes: &[OmenaScssEvalCallReturnNodeV0],
    bindings: Option<&BTreeMap<String, AbstractCssValueV0>>,
    global_variable_declarations: &[ScssGlobalVariableDeclaration],
) -> AbstractCssValueV0 {
    let reduced = reduce_static_scss_metadata_with_context(
        value,
        |name| scss_visible_function_declaration_exists(name, position, nodes).then_some(true),
        |name| scss_visible_mixin_declaration_exists(name, position, nodes).then_some(true),
        |name| {
            bindings
                .map(|bindings| bindings.contains_key(canonical_scss_variable_name(name).as_str()))
        },
        |name| scss_global_variable_metadata_exists(name, position, global_variable_declarations),
    );
    let value = match reduced {
        Some(reduced) => reduced,
        None if static_scss_metadata_exists_call_may_need_resolution(value) => {
            return AbstractCssValueV0::Top;
        }
        None => value.to_string(),
    };
    static_scss_return_abstract_value(value.as_str())
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
    let mut scope_stack = Vec::<ScssCallLocalBindingScope>::new();
    let Some(function_scope_start) = tokens
        .get(body_start)
        .map(token_range_start)
        .or_else(|| tokens.get(body_end).map(token_range_start))
    else {
        return Vec::new();
    };
    let function_scope_end = tokens
        .get(body_end)
        .map(token_range_start)
        .unwrap_or(function_scope_start);
    let mut index = body_start;
    while index < body_end {
        while scope_stack
            .last()
            .is_some_and(|scope| index > scope.end_index)
        {
            scope_stack.pop();
        }
        let Some(token) = tokens.get(index) else {
            break;
        };
        match token.kind {
            SyntaxKind::LeftBrace => {
                let Some(scope_end_index) = matching_right_brace_token_index(tokens, index) else {
                    index += 1;
                    continue;
                };
                scope_stack.push(ScssCallLocalBindingScope {
                    end_index: scope_end_index,
                    span_start: token_range_end(token),
                    span_end: token_range_start(&tokens[scope_end_index]),
                });
                index += 1;
                continue;
            }
            SyntaxKind::RightBrace => {
                if scope_stack
                    .last()
                    .is_some_and(|scope| scope.end_index == index)
                {
                    scope_stack.pop();
                }
                index += 1;
                continue;
            }
            SyntaxKind::ScssVariable => {
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
                    let (scope_span_start, scope_span_end) = scope_stack
                        .last()
                        .map(|scope| (scope.span_start, scope.span_end))
                        .unwrap_or((function_scope_start, function_scope_end));
                    bindings.push(OmenaScssEvalCallLocalBindingV0 {
                        name: token.text.clone(),
                        source_span_start: token.range.start().into(),
                        source_span_end: token.range.end().into(),
                        scope_span_start,
                        scope_span_end,
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
        return_inside_loop_control_flow: candidate.return_inside_loop_control_flow,
        return_loop_header_text: candidate.return_loop_header_text,
        return_loop_header_texts: candidate.return_loop_header_texts,
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

fn stamp_contextual_return_values(
    nodes: &mut [OmenaScssEvalCallReturnNodeV0],
    global_variable_declarations: &[ScssGlobalVariableDeclaration],
) {
    let values = nodes
        .iter()
        .enumerate()
        .filter_map(|(index, node)| {
            (node.kind == "functionReturn").then_some((
                index,
                static_scss_return_abstract_value_with_context(
                    node.return_text.as_deref()?,
                    node.source_span_start,
                    nodes,
                    None,
                    global_variable_declarations,
                ),
            ))
        })
        .collect::<Vec<_>>();

    for (index, value) in values {
        if let Some(node) = nodes.get_mut(index) {
            node.return_value_kind = Some(abstract_css_value_kind(&value));
            node.return_value = Some(value);
        }
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
    global_variable_declarations: &[ScssGlobalVariableDeclaration],
) {
    let call_graph = declaration_call_graph(nodes, edges);
    let resolutions = edges
        .iter()
        .filter(|edge| edge.kind == "functionCall")
        .filter_map(|edge| {
            let call_index = nodes
                .iter()
                .position(|node| node.node_key == edge.source_node_key)?;
            let value = call_resolved_return_value_for_edge(
                nodes,
                &call_graph,
                edge,
                global_variable_declarations,
            )?;
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
    global_variable_declarations: &[ScssGlobalVariableDeclaration],
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
        global_variable_declarations,
    )
}

fn call_resolved_return_value_for_call(
    nodes: &[OmenaScssEvalCallReturnNodeV0],
    call_graph: &BTreeMap<String, Vec<String>>,
    declaration_node: &OmenaScssEvalCallReturnNodeV0,
    argument_values: &[OmenaScssEvalCallArgumentValueV0],
    active_stack: &[String],
    global_variable_declarations: &[ScssGlobalVariableDeclaration],
) -> Option<AbstractCssValueV0> {
    if declaration_node.kind != "functionDeclaration" {
        return None;
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
        global_variable_declarations,
    };
    let Some(argument_bindings) = call_bound_argument_bindings(
        declaration_node,
        argument_values,
        declaration_node.name.as_deref(),
        Some(&context),
    ) else {
        return Some(AbstractCssValueV0::Top);
    };
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

    for node in return_nodes {
        let mut bindings = argument_bindings.clone();
        apply_call_bound_local_bindings_before(
            &mut bindings,
            declaration_node,
            node.source_span_start,
            declaration_node.name.as_deref(),
            Some(&context),
        );
        if node.return_inside_loop_control_flow {
            match call_bound_loop_return_resolution(
                node,
                bindings,
                declaration_node.name.as_deref(),
                Some(&context),
            ) {
                ScssLoopReturnResolution::Active(value) => return Some(value),
                ScssLoopReturnResolution::Inactive => continue,
                ScssLoopReturnResolution::Unknown => return Some(AbstractCssValueV0::Top),
            }
        }
        match call_bound_return_activity(node, &bindings, Some(&context)) {
            ScssCallBoundReturnActivity::Active => {}
            ScssCallBoundReturnActivity::Inactive => continue,
            ScssCallBoundReturnActivity::Unknown => return Some(AbstractCssValueV0::Top),
        }
        return Some(
            node.return_text
                .as_deref()
                .map(|text| {
                    call_bound_return_value(
                        text,
                        &bindings,
                        declaration_node.name.as_deref(),
                        Some(&context),
                        Some(node.source_span_start),
                    )
                })
                .unwrap_or(AbstractCssValueV0::Top),
        );
    }
    Some(AbstractCssValueV0::Top)
}

fn call_bound_loop_return_resolution(
    node: &OmenaScssEvalCallReturnNodeV0,
    bindings: BTreeMap<String, AbstractCssValueV0>,
    function_name: Option<&str>,
    context: Option<&ScssCallReturnResolutionContext<'_>>,
) -> ScssLoopReturnResolution {
    let header_texts = if node.return_loop_header_texts.is_empty() {
        match node.return_loop_header_text.as_deref() {
            Some(header) => vec![header.to_string()],
            None => return ScssLoopReturnResolution::Unknown,
        }
    } else {
        node.return_loop_header_texts.clone()
    };
    let Some(return_text) = node.return_text.as_deref() else {
        return ScssLoopReturnResolution::Unknown;
    };
    let Some(frames) = loop_carried_binding_frames_for_headers(header_texts.as_slice(), &bindings)
    else {
        return ScssLoopReturnResolution::Unknown;
    };
    if frames.is_empty() {
        return ScssLoopReturnResolution::Inactive;
    }

    let mut resolved = AbstractCssValueV0::Bottom;
    let mut active = false;
    for frame in frames {
        let mut frame_bindings = bindings.clone();
        for binding in frame {
            insert_static_scss_binding(&mut frame_bindings, binding.name.as_str(), binding.value);
        }
        match call_bound_return_activity(node, &frame_bindings, context) {
            ScssCallBoundReturnActivity::Active => {}
            ScssCallBoundReturnActivity::Inactive => continue,
            ScssCallBoundReturnActivity::Unknown => return ScssLoopReturnResolution::Unknown,
        }
        let value =
            single_variable_return_value(return_text, &frame_bindings).unwrap_or_else(|| {
                call_bound_return_value(
                    return_text,
                    &frame_bindings,
                    function_name,
                    context,
                    Some(node.source_span_start),
                )
            });
        if matches!(value, AbstractCssValueV0::Top) {
            return ScssLoopReturnResolution::Unknown;
        }
        resolved = join_abstract_css_values(&resolved, &value);
        active = true;
    }
    if active {
        ScssLoopReturnResolution::Active(resolved)
    } else {
        ScssLoopReturnResolution::Inactive
    }
}

fn single_variable_return_value(
    return_text: &str,
    bindings: &BTreeMap<String, AbstractCssValueV0>,
) -> Option<AbstractCssValueV0> {
    let return_text = return_text.trim();
    let names = variable_names_in_text(return_text);
    if names.len() != 1 || names.first().is_none_or(|name| name != return_text) {
        return None;
    }
    static_scss_binding_value(bindings, return_text).cloned()
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
        let value =
            call_bound_return_value(value_text.as_str(), &bindings, function_name, context, None);
        insert_static_scss_binding(&mut bindings, parameter.name.as_str(), value);
    }
    if argument_texts.is_empty() {
        Some(bindings)
    } else {
        None
    }
}

fn apply_call_bound_local_bindings_before(
    bindings: &mut BTreeMap<String, AbstractCssValueV0>,
    declaration_node: &OmenaScssEvalCallReturnNodeV0,
    position: usize,
    function_name: Option<&str>,
    context: Option<&ScssCallReturnResolutionContext<'_>>,
) {
    for local_binding in &declaration_node.local_binding_values {
        if local_binding.source_span_start >= position
            || position < local_binding.scope_span_start
            || position >= local_binding.scope_span_end
        {
            continue;
        }
        let value = call_bound_return_value(
            local_binding.value_text.as_str(),
            bindings,
            function_name,
            context,
            Some(local_binding.source_span_start),
        );
        insert_static_scss_binding(bindings, local_binding.name.as_str(), value);
    }
}

fn call_bound_return_activity(
    node: &OmenaScssEvalCallReturnNodeV0,
    bindings: &BTreeMap<String, AbstractCssValueV0>,
    context: Option<&ScssCallReturnResolutionContext<'_>>,
) -> ScssCallBoundReturnActivity {
    for condition in &node.return_negated_condition_texts {
        match call_bound_condition_truthiness(condition, bindings, node.source_span_start, context)
        {
            Some(true) => return ScssCallBoundReturnActivity::Inactive,
            Some(false) => {}
            None => return ScssCallBoundReturnActivity::Unknown,
        }
    }
    match node.return_condition_text.as_deref() {
        Some(condition) => {
            match call_bound_condition_truthiness(
                condition,
                bindings,
                node.source_span_start,
                context,
            ) {
                Some(true) => ScssCallBoundReturnActivity::Active,
                Some(false) => ScssCallBoundReturnActivity::Inactive,
                None => ScssCallBoundReturnActivity::Unknown,
            }
        }
        None => ScssCallBoundReturnActivity::Active,
    }
}

fn call_bound_condition_truthiness(
    condition: &str,
    bindings: &BTreeMap<String, AbstractCssValueV0>,
    position: usize,
    context: Option<&ScssCallReturnResolutionContext<'_>>,
) -> Option<bool> {
    let condition = if variable_names_in_text(condition).is_empty() {
        condition.to_string()
    } else {
        substitute_static_scss_header_variables(condition, bindings)?
    };
    let condition = match context {
        Some(context) => {
            let reduced = reduce_static_scss_metadata_with_context(
                condition.as_str(),
                |name| {
                    scss_visible_function_declaration_exists(name, position, context.nodes)
                        .then_some(true)
                },
                |name| {
                    scss_visible_mixin_declaration_exists(name, position, context.nodes)
                        .then_some(true)
                },
                |name| Some(bindings.contains_key(canonical_scss_variable_name(name).as_str())),
                |name| {
                    scss_global_variable_metadata_exists(
                        name,
                        position,
                        context.global_variable_declarations,
                    )
                },
            );
            match reduced {
                Some(reduced) => reduced,
                None if static_scss_metadata_exists_call_may_need_resolution(
                    condition.as_str(),
                ) =>
                {
                    return None;
                }
                None => condition,
            }
        }
        None => condition,
    };
    let reduced = reduce_static_scss_value(condition);
    static_scss_literal_truthiness(reduced.as_str())
}

fn call_bound_return_value(
    return_text: &str,
    bindings: &BTreeMap<String, AbstractCssValueV0>,
    function_name: Option<&str>,
    context: Option<&ScssCallReturnResolutionContext<'_>>,
    position: Option<usize>,
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
    match (context, position) {
        (Some(context), Some(position)) => static_scss_return_abstract_value_with_context(
            value_text.as_str(),
            position,
            context.nodes,
            Some(bindings),
            context.global_variable_declarations,
        ),
        _ => static_scss_return_abstract_value(value_text.as_str()),
    }
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
            context.global_variable_declarations,
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

fn scss_visible_function_declaration_exists(
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

fn scss_visible_mixin_declaration_exists(
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
    let source_span_end =
        control_flow_body_span_end(tokens, token_index).unwrap_or_else(|| token.range.end().into());
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

fn control_flow_body_span_end(tokens: &[LexedToken], token_index: usize) -> Option<usize> {
    let mut left_brace_index = None;
    for (index, token) in tokens.iter().enumerate().skip(token_index + 1) {
        match token.kind {
            SyntaxKind::LeftBrace => {
                left_brace_index = Some(index);
                break;
            }
            SyntaxKind::Semicolon | SyntaxKind::SassOptionalSemicolon => return None,
            _ => {}
        }
    }
    let left_brace_index = left_brace_index?;
    let right_brace_index = matching_right_brace_token_index(tokens, left_brace_index)?;
    tokens
        .get(right_brace_index)
        .map(|token| token.range.end().into())
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
            let assigned_bindings = while_loop_body_assignment_names(source, block);
            let bindings = while_loop_carried_binding_values(
                block.header_text.as_str(),
                &contextual_bindings,
                assigned_bindings.as_slice(),
            );
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
        let assigned_bindings = while_loop_body_assignment_names(source, block);
        while_loop_carried_binding_values(
            block.header_text.as_str(),
            lexical_bindings,
            assigned_bindings.as_slice(),
        )
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

fn collect_lexical_scss_bindings(source: &str, dialect: StyleDialect) -> LexicalScssBindings {
    let lexed = lex(source, dialect);
    let tokens = lexed.tokens();
    let Some(scopes) = collect_lexical_scss_scopes(source) else {
        return LexicalScssBindings::new(Vec::new());
    };
    let facts = collect_style_facts(source, dialect);
    let mut bindings = LexicalScssBindings::new(scopes);
    for symbol in &facts.sass_symbols {
        match symbol.kind {
            ParsedSassSymbolFactKind::FunctionDeclaration => bindings.push_callable(
                LexicalScssCallableKind::Function,
                symbol.name.as_str(),
                symbol.range.start().into(),
            ),
            ParsedSassSymbolFactKind::MixinDeclaration => bindings.push_callable(
                LexicalScssCallableKind::Mixin,
                symbol.name.as_str(),
                symbol.range.start().into(),
            ),
            ParsedSassSymbolFactKind::FunctionCall
            | ParsedSassSymbolFactKind::MixinInclude
            | ParsedSassSymbolFactKind::VariableDeclaration
            | ParsedSassSymbolFactKind::VariableReference => {}
        }
    }
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
            let declaration_start = token.range.start().into();
            let Some(scope_id) =
                lexical_scss_scope_for_position(&bindings.scopes, declaration_start)
            else {
                continue;
            };
            bindings.push(
                token.text.as_str(),
                declaration_start,
                scope_id,
                static_scss_header_abstract_value(value),
            );
        }
    }
    bindings
}

fn collect_scss_global_variable_declarations(
    source: &str,
    variable_facts: &[ParsedVariableFact],
) -> Vec<ScssGlobalVariableDeclaration> {
    let Some(scopes) = collect_lexical_scss_scopes(source) else {
        return Vec::new();
    };
    variable_facts
        .iter()
        .filter(|fact| fact.kind == ParsedVariableFactKind::ScssDeclaration)
        .filter_map(|fact| {
            let declaration_start = fact.range.start().into();
            let scope_id = lexical_scss_scope_for_position(&scopes, declaration_start)?;
            (scope_id == 0).then(|| ScssGlobalVariableDeclaration {
                name: canonical_scss_variable_name(fact.name.as_str()),
                declaration_start,
            })
        })
        .collect()
}

fn scss_global_variable_metadata_exists(
    name: &str,
    position: usize,
    declarations: &[ScssGlobalVariableDeclaration],
) -> Option<bool> {
    let canonical_name = canonical_scss_variable_name(name);
    if declarations.iter().any(|declaration| {
        declaration.name == canonical_name && declaration.declaration_start <= position
    }) {
        return Some(true);
    }
    if declarations.iter().any(|declaration| {
        declaration.name == canonical_name && declaration.declaration_start > position
    }) {
        return None;
    }
    Some(false)
}

fn static_scss_metadata_exists_call_may_need_resolution(value: &str) -> bool {
    const NAMES: [&str; 8] = [
        "meta.function-exists(",
        "function-exists(",
        "meta.mixin-exists(",
        "mixin-exists(",
        "meta.variable-exists(",
        "variable-exists(",
        "meta.global-variable-exists(",
        "global-variable-exists(",
    ];
    let lower = value.to_ascii_lowercase();
    NAMES.iter().any(|name| lower.contains(name))
}

fn collect_lexical_scss_scopes(source: &str) -> Option<Vec<LexicalScssScope>> {
    let mut scopes = vec![LexicalScssScope {
        parent_id: None,
        body_start: 0,
        end: source.len(),
    }];
    let mut stack = vec![0usize];
    let mut index = 0usize;
    let mut quote: Option<char> = None;
    let bytes = source.as_bytes();

    while index < source.len() {
        let ch = source[index..].chars().next()?;
        if let Some(quote_ch) = quote {
            index += ch.len_utf8();
            if ch == '\\' {
                if let Some(escaped) = source[index..].chars().next() {
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
        if bytes.get(index..index + 2) == Some(b"/*") {
            let end = source.get(index + 2..)?.find("*/")?;
            index += end + 4;
            continue;
        }
        if bytes.get(index..index + 2) == Some(b"//") {
            let line_end = source
                .get(index + 2..)?
                .find('\n')
                .map(|offset| index + 2 + offset)
                .unwrap_or(source.len());
            index = line_end;
            continue;
        }

        match ch {
            '{' => {
                let parent_id = *stack.last()?;
                let scope_id = scopes.len();
                scopes.push(LexicalScssScope {
                    parent_id: Some(parent_id),
                    body_start: index + ch.len_utf8(),
                    end: source.len(),
                });
                stack.push(scope_id);
            }
            '}' => {
                let scope_id = stack.pop()?;
                if scope_id == 0 {
                    return None;
                }
                scopes.get_mut(scope_id)?.end = index;
            }
            _ => {}
        }
        index += ch.len_utf8();
    }

    (stack.len() == 1).then_some(scopes)
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
struct LexicalScssBindings {
    bindings: Vec<LexicalScssBinding>,
    callables: Vec<LexicalScssCallableDeclaration>,
    scopes: Vec<LexicalScssScope>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct LexicalScssBinding {
    name: String,
    declaration_start: usize,
    scope_id: usize,
    value: AbstractCssValueV0,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LexicalScssCallableKind {
    Function,
    Mixin,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct LexicalScssCallableDeclaration {
    kind: LexicalScssCallableKind,
    name: String,
    declaration_start: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct LexicalScssScope {
    parent_id: Option<usize>,
    body_start: usize,
    end: usize,
}

impl LexicalScssBindings {
    fn new(scopes: Vec<LexicalScssScope>) -> Self {
        Self {
            bindings: Vec::new(),
            callables: Vec::new(),
            scopes,
        }
    }

    fn push(
        &mut self,
        name: &str,
        declaration_start: usize,
        scope_id: usize,
        value: AbstractCssValueV0,
    ) {
        self.bindings.push(LexicalScssBinding {
            name: canonical_scss_variable_name(name),
            declaration_start,
            scope_id,
            value,
        });
    }

    fn push_callable(
        &mut self,
        kind: LexicalScssCallableKind,
        name: &str,
        declaration_start: usize,
    ) {
        self.callables.push(LexicalScssCallableDeclaration {
            kind,
            name: canonical_scss_callable_name(name),
            declaration_start,
        });
    }

    fn visible_function_metadata_exists(&self, name: &str, position: usize) -> Option<bool> {
        self.visible_callable_metadata_exists(LexicalScssCallableKind::Function, name, position)
    }

    fn visible_mixin_metadata_exists(&self, name: &str, position: usize) -> Option<bool> {
        self.visible_callable_metadata_exists(LexicalScssCallableKind::Mixin, name, position)
    }

    fn visible_callable_metadata_exists(
        &self,
        kind: LexicalScssCallableKind,
        name: &str,
        position: usize,
    ) -> Option<bool> {
        let canonical_name = canonical_scss_callable_name(name);
        self.callables
            .iter()
            .any(|callable| {
                callable.kind == kind
                    && callable.name == canonical_name
                    && callable.declaration_start <= position
            })
            .then_some(true)
    }

    fn visible_at(&self, position: usize) -> BTreeMap<String, AbstractCssValueV0> {
        let Some(scope_id) = lexical_scss_scope_for_position(&self.scopes, position) else {
            return BTreeMap::new();
        };
        let mut visible = BTreeMap::new();
        for binding in self.bindings.iter() {
            if binding.declaration_start > position {
                continue;
            }
            if lexical_scss_scope_is_ancestor_or_self(&self.scopes, binding.scope_id, scope_id) {
                visible.insert(binding.name.clone(), binding.value.clone());
            } else {
                visible.insert(binding.name.clone(), AbstractCssValueV0::Top);
            }
        }
        visible
    }

    fn visible_variable_metadata_exists(&self, name: &str, position: usize) -> Option<bool> {
        let canonical_name = canonical_scss_variable_name(name);
        let scope_id = lexical_scss_scope_for_position(&self.scopes, position)?;
        if self.bindings.iter().any(|binding| {
            binding.name == canonical_name
                && binding.declaration_start <= position
                && lexical_scss_scope_is_ancestor_or_self(&self.scopes, binding.scope_id, scope_id)
        }) {
            return Some(true);
        }
        if self.bindings.iter().any(|binding| {
            binding.name == canonical_name
                && binding.declaration_start > position
                && lexical_scss_scope_is_ancestor_or_self(&self.scopes, binding.scope_id, scope_id)
        }) {
            return None;
        }
        Some(false)
    }

    fn global_variable_metadata_exists(&self, name: &str, position: usize) -> Option<bool> {
        let canonical_name = canonical_scss_variable_name(name);
        if self.bindings.iter().any(|binding| {
            binding.name == canonical_name
                && binding.scope_id == 0
                && binding.declaration_start <= position
        }) {
            return Some(true);
        }
        if self.bindings.iter().any(|binding| {
            binding.name == canonical_name
                && binding.scope_id == 0
                && binding.declaration_start > position
        }) {
            return None;
        }
        Some(false)
    }
}

fn lexical_scss_scope_for_position(scopes: &[LexicalScssScope], position: usize) -> Option<usize> {
    scopes
        .iter()
        .enumerate()
        .rev()
        .find_map(|(scope_id, scope)| {
            (position >= scope.body_start && position < scope.end).then_some(scope_id)
        })
}

fn lexical_scss_scope_is_ancestor_or_self(
    scopes: &[LexicalScssScope],
    ancestor_id: usize,
    mut scope_id: usize,
) -> bool {
    loop {
        if scope_id == ancestor_id {
            return true;
        }
        let Some(parent_id) = scopes.get(scope_id).and_then(|scope| scope.parent_id) else {
            return false;
        };
        scope_id = parent_id;
    }
}

fn scss_header_value(
    header: &str,
    lexical_bindings: &LexicalScssBindings,
    position: usize,
) -> AbstractCssValueV0 {
    let visible_bindings = lexical_bindings.visible_at(position);
    scss_header_value_with_bindings(header, lexical_bindings, position, &visible_bindings)
}

fn scss_header_value_with_bindings(
    header: &str,
    lexical_bindings: &LexicalScssBindings,
    position: usize,
    visible_bindings: &BTreeMap<String, AbstractCssValueV0>,
) -> AbstractCssValueV0 {
    let reduced_header = reduce_static_scss_metadata_with_context(
        header,
        |name| lexical_bindings.visible_function_metadata_exists(name, position),
        |name| lexical_bindings.visible_mixin_metadata_exists(name, position),
        |name| lexical_bindings.visible_variable_metadata_exists(name, position),
        |name| lexical_bindings.global_variable_metadata_exists(name, position),
    );
    match reduced_header {
        Some(header) => scss_header_value_from_bindings(header.as_str(), visible_bindings),
        None if static_scss_metadata_exists_call_may_need_resolution(header) => {
            AbstractCssValueV0::Top
        }
        None => scss_header_value_from_bindings(header, visible_bindings),
    }
}

fn scss_header_value_from_bindings(
    header: &str,
    lexical_bindings: &BTreeMap<String, AbstractCssValueV0>,
) -> AbstractCssValueV0 {
    let variables = variable_names_in_text(header);
    if variables.is_empty() {
        return static_scss_header_abstract_value(header);
    }
    if let Some(value) = scss_header_value_from_binding_combinations(header, lexical_bindings) {
        return value;
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

fn scss_header_value_from_binding_combinations(
    header: &str,
    lexical_bindings: &BTreeMap<String, AbstractCssValueV0>,
) -> Option<AbstractCssValueV0> {
    let variables = variable_names_in_text(header);
    if variables.is_empty() {
        return Some(static_scss_header_abstract_value(header));
    }
    let mut combinations = vec![BTreeMap::<String, String>::new()];
    for variable in variables {
        let values = static_scss_binding_value(lexical_bindings, variable.as_str())?;
        let values = static_scss_header_value_texts(values)?;
        if values.is_empty() {
            return None;
        }
        let mut next = Vec::new();
        for combination in combinations {
            for value in &values {
                let mut combination = combination.clone();
                combination.insert(
                    canonical_scss_variable_name(variable.as_str()),
                    value.clone(),
                );
                next.push(combination);
                if next.len() > 64 {
                    return None;
                }
            }
        }
        combinations = next;
    }
    combinations
        .into_iter()
        .map(|combination| substitute_static_scss_header_variable_combination(header, &combination))
        .collect::<Option<Vec<_>>>()
        .map(|headers| {
            headers
                .into_iter()
                .map(|header| static_scss_header_abstract_value(header.as_str()))
                .fold(AbstractCssValueV0::Bottom, |acc, value| {
                    join_abstract_css_values(&acc, &value)
                })
        })
}

fn static_scss_header_value_texts(value: &AbstractCssValueV0) -> Option<Vec<String>> {
    match value {
        AbstractCssValueV0::Exact { value } | AbstractCssValueV0::Raw { value } => {
            Some(vec![value.clone()])
        }
        AbstractCssValueV0::FiniteSet { values } => Some(values.clone()),
        AbstractCssValueV0::Bottom | AbstractCssValueV0::Top => None,
    }
}

fn substitute_static_scss_header_variable_combination(
    header: &str,
    bindings: &BTreeMap<String, String>,
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
        let value = bindings.get(canonical_scss_variable_name(name).as_str())?;
        output.push_str(value);
        index = name_end.max(index + ch.len_utf8());
    }
    Some(output)
}

fn static_scss_header_abstract_value(value: &str) -> AbstractCssValueV0 {
    let reduced = reduce_static_scss_value(value.to_string());
    let trimmed = reduced.trim();
    if static_scss_header_is_boolean_expression(trimmed)
        && let Some(truthy) = static_scss_literal_truthiness(trimmed)
    {
        return abstract_css_value_from_text(if truthy { "true" } else { "false" });
    }
    abstract_css_value_from_text(trimmed)
}

fn static_scss_header_is_boolean_expression(value: &str) -> bool {
    let trimmed = value.trim();
    let lower = trimmed.to_ascii_lowercase();
    lower == "true"
        || lower == "false"
        || lower == "null"
        || lower.starts_with("not ")
        || lower.contains(" and ")
        || lower.contains(" or ")
        || ["==", "!=", "<=", ">=", "<", ">"]
            .iter()
            .any(|operator| trimmed.contains(operator))
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
        .unwrap_or_else(|| scss_header_value_from_bindings(header, lexical_bindings))
}

fn loop_carried_binding_values(
    header: &str,
    lexical_bindings: &BTreeMap<String, AbstractCssValueV0>,
) -> Vec<ScssControlFlowBindingValue> {
    if let Some(values) = static_each_loop_binding_values(header, lexical_bindings) {
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

fn loop_carried_binding_frames(
    header: &str,
    lexical_bindings: &BTreeMap<String, AbstractCssValueV0>,
) -> Option<Vec<Vec<ScssControlFlowBindingValue>>> {
    if header
        .trim_start()
        .to_ascii_lowercase()
        .starts_with("@while")
    {
        return static_while_loop_binding_frames(header, lexical_bindings);
    }
    static_for_loop_binding_frames(header, lexical_bindings)
        .or_else(|| static_each_loop_binding_frames(header, lexical_bindings))
        .or_else(|| static_while_loop_binding_frames(header, lexical_bindings))
}

fn loop_carried_binding_frames_for_headers(
    headers: &[String],
    lexical_bindings: &BTreeMap<String, AbstractCssValueV0>,
) -> Option<Vec<Vec<ScssControlFlowBindingValue>>> {
    if headers.is_empty() {
        return None;
    }

    let mut frames = vec![Vec::<ScssControlFlowBindingValue>::new()];
    for header in headers {
        let mut next_frames = Vec::new();
        for frame in frames {
            let mut frame_bindings = lexical_bindings.clone();
            for binding in &frame {
                insert_static_scss_binding(
                    &mut frame_bindings,
                    binding.name.as_str(),
                    binding.value.clone(),
                );
            }
            let header_frames = loop_carried_binding_frames(header, &frame_bindings)?;
            for header_frame in header_frames {
                let mut combined = frame.clone();
                combined.extend(header_frame);
                next_frames.push(combined);
                if next_frames.len() > 64 {
                    return None;
                }
            }
        }
        frames = next_frames;
    }

    Some(frames)
}

fn static_for_loop_binding_frames(
    header: &str,
    lexical_bindings: &BTreeMap<String, AbstractCssValueV0>,
) -> Option<Vec<Vec<ScssControlFlowBindingValue>>> {
    let bindings = loop_carried_bindings(header);
    if bindings.len() != 1 {
        return None;
    }
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
        return None;
    }
    let value_count = if includes_end {
        i64::from(end) - i64::from(start) + 1
    } else {
        i64::from(end) - i64::from(start)
    };
    if !(0..=64).contains(&value_count) {
        return None;
    }
    let last = if includes_end {
        end
    } else {
        end.saturating_sub(1)
    };
    let frames = if value_count == 0 {
        Vec::new()
    } else {
        (start..=last)
            .map(|value| {
                vec![ScssControlFlowBindingValue {
                    name: bindings[0].clone(),
                    value: abstract_css_value_from_text(value.to_string().as_str()),
                }]
            })
            .collect()
    };
    Some(frames)
}

fn static_while_loop_binding_frames(
    header: &str,
    lexical_bindings: &BTreeMap<String, AbstractCssValueV0>,
) -> Option<Vec<Vec<ScssControlFlowBindingValue>>> {
    let bindings = loop_carried_bindings(header);
    if bindings.len() != 1 {
        return None;
    }
    let values =
        static_while_condition_loop_binding_values(header, lexical_bindings, bindings.as_slice())?;
    if values.len() != 1 {
        return None;
    }
    let binding = values.into_iter().next()?;
    match binding.value {
        AbstractCssValueV0::Bottom => Some(Vec::new()),
        AbstractCssValueV0::FiniteSet { values } => Some(
            values
                .into_iter()
                .map(|value| {
                    vec![ScssControlFlowBindingValue {
                        name: binding.name.clone(),
                        value: abstract_css_value_from_text(value.as_str()),
                    }]
                })
                .collect(),
        ),
        AbstractCssValueV0::Exact { .. }
        | AbstractCssValueV0::Raw { .. }
        | AbstractCssValueV0::Top => None,
    }
}

fn static_each_loop_binding_frames(
    header: &str,
    lexical_bindings: &BTreeMap<String, AbstractCssValueV0>,
) -> Option<Vec<Vec<ScssControlFlowBindingValue>>> {
    parse_static_scss_each_loop_binding_frames(header, |source| {
        static_each_source_text(source.trim(), lexical_bindings).map(str::to_string)
    })
    .map(|frames| {
        frames
            .into_iter()
            .map(|frame| {
                frame
                    .into_iter()
                    .map(|(name, value)| ScssControlFlowBindingValue {
                        name,
                        value: abstract_css_value_from_text(value.as_str()),
                    })
                    .collect()
            })
            .collect()
    })
}

fn static_each_loop_binding_values(
    header: &str,
    lexical_bindings: &BTreeMap<String, AbstractCssValueV0>,
) -> Option<Vec<ScssControlFlowBindingValue>> {
    let frames = static_each_loop_binding_frames(header, lexical_bindings)?;
    if frames.len() <= 1 {
        return None;
    }

    let mut values = Vec::<ScssControlFlowBindingValue>::new();
    for frame in frames {
        for binding in frame {
            if let Some(existing) = values.iter_mut().find(|existing| {
                canonical_scss_variable_name(existing.name.as_str())
                    == canonical_scss_variable_name(binding.name.as_str())
            }) {
                existing.value = join_abstract_css_values(&existing.value, &binding.value);
            } else {
                values.push(binding);
            }
        }
    }
    (!values.is_empty()).then_some(values)
}

fn while_loop_carried_binding_values(
    header: &str,
    lexical_bindings: &BTreeMap<String, AbstractCssValueV0>,
    assigned_bindings: &[String],
) -> Vec<ScssControlFlowBindingValue> {
    let binding_names = while_loop_binding_names(header, assigned_bindings);
    if let Some(values) = static_while_condition_loop_binding_values(
        header,
        lexical_bindings,
        binding_names.as_slice(),
    ) {
        return values;
    }
    binding_names
        .into_iter()
        .map(|name| ScssControlFlowBindingValue {
            name,
            value: AbstractCssValueV0::Top,
        })
        .collect()
}

fn while_loop_binding_names(header: &str, assigned_bindings: &[String]) -> Vec<String> {
    let header_bindings = loop_carried_bindings(header);
    if assigned_bindings.is_empty() {
        return header_bindings;
    }
    let filtered = header_bindings
        .iter()
        .filter(|name| {
            assigned_bindings.iter().any(|assigned| {
                canonical_scss_variable_name(name) == canonical_scss_variable_name(assigned)
            })
        })
        .cloned()
        .collect::<Vec<_>>();
    if filtered.is_empty() {
        header_bindings
    } else {
        filtered
    }
}

fn while_loop_body_assignment_names(
    source: &str,
    block: &OmenaScssEvalControlFlowBlockV0,
) -> Vec<String> {
    let Some(body) = control_flow_block_body_text(source, block) else {
        return Vec::new();
    };
    let lexed = lex(body, StyleDialect::Scss);
    let tokens = lexed.tokens();
    let mut names: Vec<String> = Vec::new();
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
        let name = token.text.to_string();
        if !names.iter().any(|existing| {
            canonical_scss_variable_name(existing.as_str())
                == canonical_scss_variable_name(name.as_str())
        }) {
            names.push(name);
        }
    }
    names
}

fn control_flow_block_body_text<'a>(
    source: &'a str,
    block: &OmenaScssEvalControlFlowBlockV0,
) -> Option<&'a str> {
    let block_text = source.get(block.source_span_start..block.source_span_end)?;
    let open = block_text.find('{')?;
    let close = block_text.rfind('}')?;
    (open < close)
        .then(|| block_text.get(open + '{'.len_utf8()..close))
        .flatten()
}

fn static_while_condition_loop_binding_values(
    header: &str,
    lexical_bindings: &BTreeMap<String, AbstractCssValueV0>,
    binding_names: &[String],
) -> Option<Vec<ScssControlFlowBindingValue>> {
    if binding_names.len() != 1 {
        return None;
    }
    let binding_name = binding_names[0].as_str();
    let (left, operator, right) = split_static_while_inequality(header)?;
    let start = static_while_integer_binding_value(binding_name, lexical_bindings)?;

    let (operator, bound) = if static_scss_side_is_binding(left, binding_name) {
        (
            operator,
            static_while_integer_operand(right, lexical_bindings)?,
        )
    } else if static_scss_side_is_binding(right, binding_name) {
        (
            operator.inverted_for_right_hand_binding(),
            static_while_integer_operand(left, lexical_bindings)?,
        )
    } else {
        return None;
    };
    let value = static_while_integer_domain(start, operator, bound)?;

    Some(vec![ScssControlFlowBindingValue {
        name: binding_names[0].clone(),
        value,
    }])
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StaticWhileInequalityOperator {
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
}

impl StaticWhileInequalityOperator {
    const fn inverted_for_right_hand_binding(self) -> Self {
        match self {
            Self::LessThan => Self::GreaterThan,
            Self::LessThanOrEqual => Self::GreaterThanOrEqual,
            Self::GreaterThan => Self::LessThan,
            Self::GreaterThanOrEqual => Self::LessThanOrEqual,
        }
    }
}

fn split_static_while_inequality(
    value: &str,
) -> Option<(&str, StaticWhileInequalityOperator, &str)> {
    let mut comparison = None;
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;
    let mut quote: Option<char> = None;
    let mut index = 0usize;

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
            '<' | '>' if paren_depth == 0 && bracket_depth == 0 => {
                let (operator, width) = static_while_inequality_operator_at(value, index)?;
                let left = value.get(..index)?.trim();
                let right = value.get(index + width..)?.trim();
                if left.is_empty() || right.is_empty() || comparison.is_some() {
                    return None;
                }
                comparison = Some((left, operator, right));
                index += width;
                continue;
            }
            _ => {}
        }
        index += ch.len_utf8();
    }
    if quote.is_some() || paren_depth != 0 || bracket_depth != 0 {
        return None;
    }
    comparison
}

fn static_while_inequality_operator_at(
    value: &str,
    index: usize,
) -> Option<(StaticWhileInequalityOperator, usize)> {
    let suffix = value.get(index..)?;
    if suffix.starts_with("<=") {
        return Some((StaticWhileInequalityOperator::LessThanOrEqual, 2));
    }
    if suffix.starts_with(">=") {
        return Some((StaticWhileInequalityOperator::GreaterThanOrEqual, 2));
    }
    if suffix.starts_with('<') {
        return Some((StaticWhileInequalityOperator::LessThan, 1));
    }
    if suffix.starts_with('>') {
        return Some((StaticWhileInequalityOperator::GreaterThan, 1));
    }
    None
}

fn static_scss_side_is_binding(value: &str, binding_name: &str) -> bool {
    let value = value.trim();
    value.starts_with('$')
        && variable_name_end(value, '$'.len_utf8()) == value.len()
        && canonical_scss_variable_name(value) == canonical_scss_variable_name(binding_name)
}

fn static_while_integer_binding_value(
    binding_name: &str,
    lexical_bindings: &BTreeMap<String, AbstractCssValueV0>,
) -> Option<i32> {
    static_scss_binding_value(lexical_bindings, binding_name)
        .and_then(single_static_scss_header_value_text)
        .and_then(parse_static_while_integer_text)
}

fn static_while_integer_operand(
    value: &str,
    lexical_bindings: &BTreeMap<String, AbstractCssValueV0>,
) -> Option<i32> {
    let reduced = scss_header_value_from_bindings(value, lexical_bindings);
    single_static_scss_header_value_text(&reduced).and_then(parse_static_while_integer_text)
}

fn parse_static_while_integer_text(value: &str) -> Option<i32> {
    let reduced = reduce_static_scss_value(value.trim().to_string());
    reduced.trim().parse::<i32>().ok()
}

fn static_while_integer_domain(
    start: i32,
    operator: StaticWhileInequalityOperator,
    bound: i32,
) -> Option<AbstractCssValueV0> {
    let (first, last) = match operator {
        StaticWhileInequalityOperator::LessThan => {
            if start >= bound {
                return Some(AbstractCssValueV0::Bottom);
            }
            (start, bound.saturating_sub(1))
        }
        StaticWhileInequalityOperator::LessThanOrEqual => {
            if start > bound {
                return Some(AbstractCssValueV0::Bottom);
            }
            (start, bound)
        }
        StaticWhileInequalityOperator::GreaterThan => {
            if start <= bound {
                return Some(AbstractCssValueV0::Bottom);
            }
            (bound.saturating_add(1), start)
        }
        StaticWhileInequalityOperator::GreaterThanOrEqual => {
            if start < bound {
                return Some(AbstractCssValueV0::Bottom);
            }
            (bound, start)
        }
    };
    let value_count = i64::from(last) - i64::from(first) + 1;
    if !(1..=64).contains(&value_count) {
        return None;
    }
    Some(
        (first..=last).fold(AbstractCssValueV0::Bottom, |acc, value| {
            let value = abstract_css_value_from_text(value.to_string().as_str());
            join_abstract_css_values(&acc, &value)
        }),
    )
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
    let reduced = match scss_header_value_from_bindings(value, lexical_bindings) {
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
    fn control_flow_value_analysis_uses_loop_carried_bindings_for_nested_branch_conditions() {
        let source = "@for $i from 1 through 3 { @if $i == 2 { .item { order: $i; } } }";
        let report = analyze_scss_control_flow_values(source, StyleDialect::Scss);
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.block_count, 2);
        assert_eq!(report.blocks[0].transfer_kind, "loopCarriedBindings");
        assert_eq!(report.blocks[0].loop_carried_bindings, vec!["$i"]);
        assert_eq!(report.blocks[1].transfer_kind, "branchCondition");
        assert_eq!(report.blocks[1].transfer_value_kind, Some("finiteSet"));
        assert_eq!(
            report.blocks[1].transfer_value,
            Some(AbstractCssValueV0::FiniteSet {
                values: vec!["false".to_string(), "true".to_string()]
            })
        );
        assert_eq!(report.blocks[1].transfer_truthiness, None);
    }

    #[test]
    fn control_flow_value_analysis_does_not_leak_loop_bindings_after_loop_body() {
        let source = "@for $i from 1 through 3 { .item { order: $i; } } @if $i == 2 { .leak { order: $i; } }";
        let report = analyze_scss_control_flow_values(source, StyleDialect::Scss);
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.block_count, 2);
        assert_eq!(report.blocks[0].loop_carried_bindings, vec!["$i"]);
        assert_eq!(report.blocks[1].transfer_kind, "branchCondition");
        assert_eq!(report.blocks[1].transfer_value_kind, Some("top"));
        assert_eq!(report.blocks[1].output_value_kind, "top");
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
    fn control_flow_value_analysis_reduces_static_if_variable_bindings() {
        let source = "$enabled: if(true, true, false); @if $enabled { .on { color: green; } }";
        let report = analyze_scss_control_flow_values(source, StyleDialect::Scss);
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.block_count, 1);
        assert_eq!(report.blocks[0].transfer_kind, "branchCondition");
        assert_eq!(report.blocks[0].transfer_value_kind, Some("raw"));
        assert_eq!(report.blocks[0].transfer_truthiness, Some("truthy"));
    }

    #[test]
    fn control_flow_value_analysis_reduces_numeric_variable_bindings() {
        let source = "$gap: 1px + 2px; @if $gap == 3px { .on { color: green; } }";
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
    fn control_flow_value_analysis_reports_sass_variable_metadata_branch_truthiness() {
        let source = "$enabled: true; @if variable-exists(\"enabled\") and not variable-exists(\"missing\") { .on { color: green; } }";
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
    fn control_flow_value_analysis_reports_sass_global_variable_metadata_branch_truthiness() {
        let source = "$theme: dark; @if global-variable-exists(\"theme\") and not meta.global-variable-exists(\"missing\") { .on { color: green; } }";
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
    fn control_flow_value_analysis_reports_sass_function_metadata_branch_truthiness() {
        let source = "@function present() { @return 1px; } @if function-exists(\"present\") and not function-exists(\"missing\") { .on { color: green; } }";
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
    fn control_flow_value_analysis_reports_sass_builtin_function_metadata_branch_truthiness() {
        let source = "@if meta.function-exists(\"scale-color\") { .on { color: green; } }";
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
    fn control_flow_value_analysis_preserves_function_exists_declaration_order() {
        let source = "@if function-exists(\"later\") { .on { color: green; } } @function later() { @return 1px; }";
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
    fn control_flow_value_analysis_reports_sass_mixin_metadata_branch_truthiness() {
        let source = "@mixin present { color: red; } @if mixin-exists(\"present\") and not meta.mixin-exists(\"missing\") { .on { color: green; } }";
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
    fn control_flow_value_analysis_preserves_mixin_exists_declaration_order() {
        let source =
            "@if mixin-exists(\"later\") { .on { color: green; } } @mixin later { color: red; }";
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
    fn control_flow_value_analysis_keeps_future_global_variable_metadata_top() {
        let source =
            "@if global-variable-exists(\"theme\") { .on { color: green; } } $theme: dark;";
        let report = analyze_scss_control_flow_values(source, StyleDialect::Scss);
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.block_count, 1);
        assert_eq!(report.blocks[0].transfer_kind, "branchCondition");
        assert_eq!(report.blocks[0].transfer_value_kind, Some("top"));
        assert_eq!(report.blocks[0].transfer_truthiness, None);
    }

    #[test]
    fn control_flow_value_analysis_does_not_treat_local_binding_as_global_metadata() {
        let source = ".scope { $theme: dark; @if global-variable-exists(\"theme\") { .on { color: green; } } }";
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
    fn control_flow_value_analysis_respects_declaration_order_for_branch_headers() {
        let source = "@if $enabled { .on { color: green; } } $enabled: true;";
        let report = analyze_scss_control_flow_values(source, StyleDialect::Scss);
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.block_count, 1);
        assert_eq!(report.blocks[0].transfer_kind, "branchCondition");
        assert_eq!(report.blocks[0].transfer_value_kind, Some("top"));
        assert_eq!(report.blocks[0].transfer_truthiness, None);
    }

    #[test]
    fn control_flow_value_analysis_does_not_leak_sibling_block_bindings() {
        let source = "@if true { $enabled: true; } @if $enabled { .on { color: green; } }";
        let report = analyze_scss_control_flow_values(source, StyleDialect::Scss);
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.block_count, 2);
        assert_eq!(report.blocks[1].transfer_kind, "branchCondition");
        assert_eq!(report.blocks[1].transfer_value_kind, Some("top"));
        assert_eq!(report.blocks[1].transfer_truthiness, None);
    }

    #[test]
    fn control_flow_value_analysis_marks_sibling_block_reassignment_top() {
        let source =
            "$enabled: false; @if true { $enabled: true; } @if $enabled { .on { color: green; } }";
        let report = analyze_scss_control_flow_values(source, StyleDialect::Scss);
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.block_count, 2);
        assert_eq!(report.blocks[1].transfer_kind, "branchCondition");
        assert_eq!(report.blocks[1].transfer_value_kind, Some("top"));
        assert_eq!(report.blocks[1].transfer_truthiness, None);
    }

    #[test]
    fn control_flow_value_analysis_uses_enclosing_scope_bindings() {
        let source = "$enabled: true; .scope { @if $enabled { .on { color: green; } } }";
        let report = analyze_scss_control_flow_values(source, StyleDialect::Scss);
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.block_count, 1);
        assert_eq!(report.blocks[0].transfer_kind, "branchCondition");
        assert_eq!(report.blocks[0].transfer_value_kind, Some("raw"));
        assert_eq!(report.blocks[0].transfer_truthiness, Some("truthy"));
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
    fn control_flow_value_analysis_respects_declaration_order_for_loop_bounds() {
        let source = "@for $i from $start through $end { .n { order: $i; } } $start: 1; $end: 3;";
        let report = analyze_scss_control_flow_values(source, StyleDialect::Scss);
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.block_count, 1);
        assert_eq!(report.blocks[0].transfer_kind, "loopCarriedBindings");
        assert_eq!(report.blocks[0].transfer_value_kind, Some("top"));
        assert_eq!(report.blocks[0].loop_carried_bindings, vec!["$i"]);
        assert_eq!(
            report.blocks[0].loop_carried_binding_values[0].value_kind,
            "top"
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
            report.blocks[0].loop_carried_binding_values[0].value,
            AbstractCssValueV0::FiniteSet {
                values: vec!["0".to_string(), "1".to_string(), "2".to_string()]
            }
        );
    }

    #[test]
    fn control_flow_value_analysis_tracks_reversed_while_condition_loop_bindings() {
        let source = "$i: 3; @while 0 < $i { $i: $i - 1; .n { order: $i; } }";
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
            report.blocks[0].loop_carried_binding_values[0].value,
            AbstractCssValueV0::FiniteSet {
                values: vec!["1".to_string(), "2".to_string(), "3".to_string()]
            }
        );
    }

    #[test]
    fn control_flow_value_analysis_tracks_while_bound_variable_bindings() {
        let source = "$end: 3; $i: 0; @while $i < $end { $i: $i + 1; .n { order: $i; } }";
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
        assert_eq!(report.blocks[0].loop_carried_bindings, vec!["$i"]);
        assert_eq!(
            report.blocks[0].loop_carried_binding_values[0].value,
            AbstractCssValueV0::FiniteSet {
                values: vec!["0".to_string(), "1".to_string(), "2".to_string()]
            }
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
    fn call_return_ir_resolves_local_bindings_after_prior_branch() {
        let source = "@function pick($enabled) { @if $enabled { @return 3px; } $after: 1px + 1px; @return $after; } .a { width: pick(false); }";
        let report = summarize_scss_call_return_ir(source, StyleDialect::Scss);
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };
        let declaration = report.nodes.iter().find(|node| {
            node.kind == "functionDeclaration" && node.name.as_deref() == Some("pick")
        });
        assert!(declaration.is_some());
        let Some(declaration) = declaration else {
            return;
        };
        assert_eq!(declaration.local_binding_values.len(), 1);
        assert_eq!(declaration.local_binding_values[0].name, "$after");

        let function_call = report
            .nodes
            .iter()
            .find(|node| node.kind == "functionCall" && node.name.as_deref() == Some("pick"));
        assert!(function_call.is_some());
        let Some(function_call) = function_call else {
            return;
        };

        assert_eq!(function_call.call_resolved_return_value_kind, Some("exact"));
        assert_eq!(
            function_call.call_resolved_return_value,
            Some(AbstractCssValueV0::Exact {
                value: "2px".to_string()
            })
        );
    }

    #[test]
    fn call_return_ir_resolves_branch_local_bindings() {
        let source = "@function pick($enabled) { @if $enabled { $inside: 1px + 1px; @return $inside; } @return 1px; } .a { width: pick(true); }";
        let report = summarize_scss_call_return_ir(source, StyleDialect::Scss);
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };
        let declaration = report.nodes.iter().find(|node| {
            node.kind == "functionDeclaration" && node.name.as_deref() == Some("pick")
        });
        assert!(declaration.is_some());
        let Some(declaration) = declaration else {
            return;
        };
        assert_eq!(declaration.local_binding_values.len(), 1);
        assert_eq!(declaration.local_binding_values[0].name, "$inside");

        let function_call = report
            .nodes
            .iter()
            .find(|node| node.kind == "functionCall" && node.name.as_deref() == Some("pick"));
        assert!(function_call.is_some());
        let Some(function_call) = function_call else {
            return;
        };

        assert_eq!(function_call.call_resolved_return_value_kind, Some("exact"));
        assert_eq!(
            function_call.call_resolved_return_value,
            Some(AbstractCssValueV0::Exact {
                value: "2px".to_string()
            })
        );
    }

    #[test]
    fn call_return_ir_does_not_leak_sibling_branch_local_bindings() {
        let source = "@function pick($enabled) { @if $enabled { @return $other; } @else { $other: 1px; @return $other; } } .a { width: pick(true); }";
        let report = summarize_scss_call_return_ir(source, StyleDialect::Scss);
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };
        let declaration = report.nodes.iter().find(|node| {
            node.kind == "functionDeclaration" && node.name.as_deref() == Some("pick")
        });
        assert!(declaration.is_some());
        let Some(declaration) = declaration else {
            return;
        };
        assert_eq!(declaration.local_binding_values.len(), 1);
        assert_eq!(declaration.local_binding_values[0].name, "$other");

        let function_call = report
            .nodes
            .iter()
            .find(|node| node.kind == "functionCall" && node.name.as_deref() == Some("pick"));
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
    fn call_return_ir_keeps_future_local_bindings_out_of_active_return() {
        let source = "@function pick($enabled) { @if $enabled { @return $after; } $after: 1px; @return $after; } .a { width: pick(true); }";
        let report = summarize_scss_call_return_ir(source, StyleDialect::Scss);
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };
        let declaration = report.nodes.iter().find(|node| {
            node.kind == "functionDeclaration" && node.name.as_deref() == Some("pick")
        });
        assert!(declaration.is_some());
        let Some(declaration) = declaration else {
            return;
        };
        assert_eq!(declaration.local_binding_values.len(), 1);
        assert_eq!(declaration.local_binding_values[0].name, "$after");

        let function_call = report
            .nodes
            .iter()
            .find(|node| node.kind == "functionCall" && node.name.as_deref() == Some("pick"));
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
    fn call_return_ir_respects_first_active_return_before_fallback() {
        let source = "@function tone($enabled) { @if $enabled { @return red; } @return blue; } .a { color: tone(true); }";
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
    fn call_return_ir_resolves_static_for_loop_body_returns() {
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
        assert_eq!(report.finite_set_call_resolved_return_value_count, 1);
        assert_eq!(
            function_call.call_resolved_return_value_kind,
            Some("finiteSet")
        );
        assert_eq!(
            function_call.call_resolved_return_value,
            Some(AbstractCssValueV0::FiniteSet {
                values: vec!["1".to_string(), "2".to_string(), "3".to_string()]
            })
        );
    }

    #[test]
    fn call_return_ir_resolves_static_each_loop_body_returns() {
        let source = "@function tones() { @each $tone in red, blue { @return $tone; } } .a { color: tones(); }";
        let report = summarize_scss_call_return_ir(source, StyleDialect::Scss);
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };
        let function_call = report
            .nodes
            .iter()
            .find(|node| node.kind == "functionCall" && node.name.as_deref() == Some("tones"));
        assert!(function_call.is_some());
        let Some(function_call) = function_call else {
            return;
        };

        assert_eq!(report.call_resolved_return_value_count, 1);
        assert_eq!(report.finite_set_call_resolved_return_value_count, 1);
        assert_eq!(
            function_call.call_resolved_return_value_kind,
            Some("finiteSet")
        );
        assert_eq!(
            function_call.call_resolved_return_value,
            Some(AbstractCssValueV0::FiniteSet {
                values: vec!["#00f".to_string(), "red".to_string()]
            })
        );
    }

    #[test]
    fn call_return_ir_resolves_static_while_loop_body_returns() {
        let source = "@function collect() { $i: 0; @while $i < 3 { @return $i; $i: $i + 1; } } .a { width: collect(); }";
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
        assert_eq!(report.finite_set_call_resolved_return_value_count, 1);
        assert_eq!(
            function_call.call_resolved_return_value_kind,
            Some("finiteSet")
        );
        assert_eq!(
            function_call.call_resolved_return_value,
            Some(AbstractCssValueV0::FiniteSet {
                values: vec!["0".to_string(), "1".to_string(), "2".to_string()]
            })
        );
    }

    #[test]
    fn call_return_ir_filters_static_while_conditional_returns() {
        let source = "@function collect() { $i: 0; @while $i < 3 { @if $i == 2 { @return $i; } $i: $i + 1; } @return 0; } .a { width: collect(); }";
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
        assert_eq!(report.exact_call_resolved_return_value_count, 1);
        assert_eq!(function_call.call_resolved_return_value_kind, Some("exact"));
        assert_eq!(
            function_call.call_resolved_return_value,
            Some(AbstractCssValueV0::Exact {
                value: "2".to_string()
            })
        );
    }

    #[test]
    fn call_return_ir_uses_call_arguments_in_static_while_conditional_returns() {
        let source = "@function collect($target) { $i: 0; @while $i < 3 { @if $i == $target { @return $i + 1; } $i: $i + 1; } @return 0; } .a { width: collect(2); }";
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
        assert_eq!(report.exact_call_resolved_return_value_count, 1);
        assert_eq!(function_call.call_resolved_return_value_kind, Some("exact"));
        assert_eq!(
            function_call.call_resolved_return_value,
            Some(AbstractCssValueV0::Exact {
                value: "3".to_string()
            })
        );
    }

    #[test]
    fn call_return_ir_filters_static_for_loop_conditional_returns() {
        let source = "@function collect($target) { @for $i from 1 through 3 { @if $i == $target { @return $i; } } @return 0; } .a { width: collect(2); }";
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
        assert_eq!(report.exact_call_resolved_return_value_count, 1);
        assert_eq!(function_call.call_resolved_return_value_kind, Some("exact"));
        assert_eq!(
            function_call.call_resolved_return_value,
            Some(AbstractCssValueV0::Exact {
                value: "2".to_string()
            })
        );
    }

    #[test]
    fn call_return_ir_continues_after_inactive_static_loop_returns() {
        let source = "@function collect($target) { @for $i from 1 through 3 { @if $i == $target { @return $i; } } @return 0; } .a { width: collect(4); }";
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
        assert_eq!(report.exact_call_resolved_return_value_count, 1);
        assert_eq!(function_call.call_resolved_return_value_kind, Some("exact"));
        assert_eq!(
            function_call.call_resolved_return_value,
            Some(AbstractCssValueV0::Exact {
                value: "0".to_string()
            })
        );
    }

    #[test]
    fn call_return_ir_resolves_nested_static_loop_body_returns() {
        let source = "@function collect($target) { @for $i from 1 through 2 { @for $j from 1 through 2 { @if $i == $target { @return $i + $j; } } } @return 0; } .a { width: collect(2); }";
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

        assert!(report.nodes.iter().any(|node| {
            node.kind == "functionReturn" && node.return_loop_header_texts.len() == 2
        }));
        assert_eq!(report.call_resolved_return_value_count, 1);
        assert_eq!(report.finite_set_call_resolved_return_value_count, 1);
        assert_eq!(
            function_call.call_resolved_return_value_kind,
            Some("finiteSet")
        );
        assert_eq!(
            function_call.call_resolved_return_value,
            Some(AbstractCssValueV0::FiniteSet {
                values: vec!["3".to_string(), "4".to_string()]
            })
        );
    }

    #[test]
    fn call_return_ir_continues_after_inactive_nested_static_loop_returns() {
        let source = "@function collect($target) { @for $i from 1 through 2 { @for $j from 1 through 2 { @if $i == $target { @return $i + $j; } } } @return 0; } .a { width: collect(3); }";
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
        assert_eq!(report.exact_call_resolved_return_value_count, 1);
        assert_eq!(function_call.call_resolved_return_value_kind, Some("exact"));
        assert_eq!(
            function_call.call_resolved_return_value,
            Some(AbstractCssValueV0::Exact {
                value: "0".to_string()
            })
        );
    }

    #[test]
    fn call_return_ir_filters_static_each_map_conditional_returns() {
        let source = "@function tone($target) { @each $name, $tone in (primary: red, secondary: blue) { @if $name == $target { @return $tone; } } @return black; } .a { color: tone(secondary); }";
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
    fn call_return_ir_keeps_dynamic_loop_body_returns_top() {
        let source = "@function collect($count) { @for $i from 1 through $count { @return $i; } } .a { width: collect(var(--count)); }";
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
    fn call_return_ir_resolves_return_after_static_loop() {
        let source = "@function collect() { @for $i from 1 through 3 { $seen: $i; } @return 2px; } .a { width: collect(); }";
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
        assert_eq!(report.exact_call_resolved_return_value_count, 1);
        assert_eq!(function_call.call_resolved_return_value_kind, Some("exact"));
        assert_eq!(
            function_call.call_resolved_return_value,
            Some(AbstractCssValueV0::Exact {
                value: "2px".to_string()
            })
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
    fn call_return_ir_reports_nested_static_scss_map_lookup_values() {
        let source = "@function font-weight() { @return if(map.has-key((font: (weights: (regular: 400, medium: 500))), font, weights, medium), map.get((font: (weights: (regular: 400, medium: 500))), font, weights, medium), 0); } .a { font-weight: font-weight(); }";
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
                value: "500".to_string()
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
    fn call_return_ir_reports_static_scss_type_metadata_values() {
        let source = "@function metadata() { @return if(meta.type-of(1px) == number and type-of(red) == color and meta.type-of(color.mix(red, blue)) == color and meta.type-of(transparentize(red, .25)) == color and meta.type-of(hue(red)) == number and meta.type-of(color.channel(color.mix(red, blue), \"red\", $space: rgb)) == number and meta.type-of(red(red)) == number and meta.type-of(oklch(100% 0 0)) == color and meta.type-of((dense: true)) == map and feature-exists(\"at-error\") and meta.feature-exists(custom-property) and not meta.feature-exists(\"unknown\"), 3px, 4px); } .a { margin: metadata(); }";
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
    fn call_return_ir_reports_static_scss_calculation_metadata_values() {
        let source = "@function metadata() { @return if(meta.calc-name(clamp(1px, 2px, 3px)) == \"clamp\" and meta.type-of(calc(100% - 1px)) == calculation and list.nth(meta.calc-args(clamp(1px, 2px, 3px)), 2) == 2px and list.length(meta.calc-args(min(4px, 5px))) == 2, 3px, 4px); } .a { margin: metadata(); }";
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
    fn call_return_ir_reports_static_scss_function_metadata_values() {
        let source = "@function metadata() { @return if(meta.function-exists(\"scale-color\") and function-exists(\"hue\") and not function-exists(\"not-defined-here\"), 3px, 4px); } .a { margin: metadata(); }";
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
        let function_call = report
            .nodes
            .iter()
            .find(|node| node.kind == "functionCall" && node.name.as_deref() == Some("metadata"));
        assert!(function_call.is_some());
        let Some(function_call) = function_call else {
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
        assert_eq!(function_call.call_resolved_return_value_kind, Some("exact"));
        assert_eq!(
            function_call.call_resolved_return_value,
            Some(AbstractCssValueV0::Exact {
                value: "3px".to_string()
            })
        );
    }

    #[test]
    fn call_return_ir_preserves_function_exists_declaration_order() {
        let source = "@function gate() { @return if(function-exists(\"later\"), 2px, 1px); } @function later() { @return 2px; } .a { margin: gate(); }";
        let report = summarize_scss_call_return_ir(source, StyleDialect::Scss);
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };
        let function_call = report
            .nodes
            .iter()
            .find(|node| node.kind == "functionCall" && node.name.as_deref() == Some("gate"));
        assert!(function_call.is_some());
        let Some(function_call) = function_call else {
            return;
        };

        assert_eq!(function_call.call_resolved_return_value_kind, Some("exact"));
        assert_eq!(
            function_call.call_resolved_return_value,
            Some(AbstractCssValueV0::Exact {
                value: "1px".to_string()
            })
        );
    }

    #[test]
    fn call_return_ir_reports_static_scss_variable_metadata_values() {
        let source = "@function metadata($input) { $local: 1px; @return if(meta.variable-exists(\"input\") and variable-exists(\"local\") and not variable-exists(\"missing\"), 3px, 4px); } .a { margin: metadata(2px); }";
        let report = summarize_scss_call_return_ir(source, StyleDialect::Scss);
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };
        let function_call = report
            .nodes
            .iter()
            .find(|node| node.kind == "functionCall" && node.name.as_deref() == Some("metadata"));
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
    fn call_return_ir_reports_static_scss_global_variable_metadata_values() {
        let source = "$theme: dark; @function metadata() { @return if(global-variable-exists(\"theme\") and not meta.global-variable-exists(\"missing\"), 3px, 4px); } .a { margin: metadata(); }";
        let report = summarize_scss_call_return_ir(source, StyleDialect::Scss);
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };
        let function_call = report
            .nodes
            .iter()
            .find(|node| node.kind == "functionCall" && node.name.as_deref() == Some("metadata"));
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
    fn call_return_ir_keeps_future_global_variable_metadata_unknown() {
        let source = "@function metadata() { @return if(global-variable-exists(\"theme\"), 3px, 4px); } $theme: dark; .a { margin: metadata(); }";
        let report = summarize_scss_call_return_ir(source, StyleDialect::Scss);
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };
        let function_call = report
            .nodes
            .iter()
            .find(|node| node.kind == "functionCall" && node.name.as_deref() == Some("metadata"));
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
    fn call_return_ir_does_not_treat_local_binding_as_global_metadata() {
        let source = "@function metadata() { $theme: dark; @return if(global-variable-exists(\"theme\"), 3px, 4px); } .a { margin: metadata(); }";
        let report = summarize_scss_call_return_ir(source, StyleDialect::Scss);
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };
        let function_call = report
            .nodes
            .iter()
            .find(|node| node.kind == "functionCall" && node.name.as_deref() == Some("metadata"));
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
    fn call_return_ir_reports_static_scss_mixin_metadata_values() {
        let source = "@mixin present { color: red; } @function metadata() { @return if(meta.mixin-exists(\"present\") and not mixin-exists(\"not-defined-here\"), 3px, 4px); } .a { margin: metadata(); }";
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
        let function_call = report
            .nodes
            .iter()
            .find(|node| node.kind == "functionCall" && node.name.as_deref() == Some("metadata"));
        assert!(function_call.is_some());
        let Some(function_call) = function_call else {
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
        assert_eq!(function_call.call_resolved_return_value_kind, Some("exact"));
        assert_eq!(
            function_call.call_resolved_return_value,
            Some(AbstractCssValueV0::Exact {
                value: "3px".to_string()
            })
        );
    }

    #[test]
    fn call_return_ir_preserves_mixin_exists_declaration_order() {
        let source = "@function gate() { @return if(mixin-exists(\"later\"), 2px, 1px); } @mixin later { color: red; } .a { margin: gate(); }";
        let report = summarize_scss_call_return_ir(source, StyleDialect::Scss);
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };
        let function_call = report
            .nodes
            .iter()
            .find(|node| node.kind == "functionCall" && node.name.as_deref() == Some("gate"));
        assert!(function_call.is_some());
        let Some(function_call) = function_call else {
            return;
        };

        assert_eq!(function_call.call_resolved_return_value_kind, Some("exact"));
        assert_eq!(
            function_call.call_resolved_return_value,
            Some(AbstractCssValueV0::Exact {
                value: "1px".to_string()
            })
        );
    }

    #[test]
    fn call_return_ir_reports_static_scss_string_metadata_values() {
        let source = "@function index() { @return if(string.index(\"Helvetica Neue\", \"Neue\") == 11, string.length(\"Helvetica Neue\"), 0); } .a { z-index: index(); }";
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
                value: "14".to_string()
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
    fn call_return_ir_reports_static_scss_map_key_and_value_lists() {
        let source = "@function map-value() { @return list.nth(map.values((default: 1px, dense: 2px)), list.length(map.keys((default: 1px, dense: 2px)))); } .a { margin: map-value(); }";
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
    fn call_return_ir_reports_static_scss_map_merge_values() {
        let source = "@function gap() { @return map.get(map.merge((default: 1px, dense: 2px), (dense: 3px, compact: 4px)), dense); } .a { margin: gap(); }";
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
    fn call_return_ir_reports_nested_static_scss_map_merge_values() {
        let source = "@function gap() { @return map.get(map.merge((theme: (spacing: (sm: 4px))), theme, spacing, (md: 8px)), theme, spacing, md); } .a { margin: gap(); }";
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
                value: "8px".to_string()
            })
        );
    }

    #[test]
    fn call_return_ir_reports_static_scss_map_deep_merge_values() {
        let source = "@function gap() { @return map.get(map.deep-merge((theme: (spacing: (sm: 4px))), (theme: (spacing: (md: 8px)))), theme, spacing, md); } .a { margin: gap(); }";
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
                value: "8px".to_string()
            })
        );
    }

    #[test]
    fn call_return_ir_reports_static_scss_map_remove_values() {
        let source = "@function count() { @return list.length(map.keys(map.remove((default: 1px, dense: 2px), dense))); } .a { z-index: count(); }";
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
                value: "1".to_string()
            })
        );
    }

    #[test]
    fn call_return_ir_reports_nested_static_scss_map_deep_remove_values() {
        let source = "@function gap() { @return map.get(map.deep-remove((theme: (spacing: (sm: 4px, md: 8px))), theme, spacing, sm), theme, spacing, md); } .a { margin: gap(); }";
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
                value: "8px".to_string()
            })
        );
    }

    #[test]
    fn call_return_ir_reports_static_scss_map_set_values() {
        let source = "@function weight() { @return map.get(map.set((regular: 400), bold, 700), bold); } .a { font-weight: weight(); }";
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
                value: "700".to_string()
            })
        );
    }

    #[test]
    fn call_return_ir_reports_nested_static_scss_map_set_values() {
        let source = "@function tone() { @return map.get(map.set((theme: blue), theme, colors, primary, red), theme, colors, primary); } .a { color: tone(); }";
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
                value: "red".to_string()
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
    fn call_return_ir_reports_static_scss_math_constant_returns() {
        let source = "@function pi() { @return math.$pi; } .a { --pi: pi(); }";
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
                value: "3.141593".to_string()
            })
        );
    }

    #[test]
    fn call_return_ir_reports_static_scss_math_constant_argument_returns() {
        let source = "@function enabled() { @return if(math.is-unitless(math.$pi), 1px, 2px); } .a { margin: enabled(); }";
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
