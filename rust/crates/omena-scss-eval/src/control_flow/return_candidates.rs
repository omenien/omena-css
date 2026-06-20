use std::collections::BTreeMap;

use omena_abstract_value::{AbstractCssValueV0, abstract_css_value_from_text};
use omena_parser::LexedToken;
use omena_syntax::SyntaxKind;

use crate::{
    scss_metadata::reduce_static_scss_metadata_with_context, value_eval::reduce_static_scss_value,
};

use super::{
    analysis_model::{
        ScssBranchBlock, ScssCallReturnCandidate, ScssGlobalVariableDeclaration,
        ScssReturnCondition,
    },
    blocks::{control_flow_header_text, scss_else_if_header_condition},
    call_resolution::{
        scss_visible_function_declaration_exists, scss_visible_mixin_declaration_exists,
    },
    lexical::{
        scss_global_variable_metadata_exists, static_scss_metadata_exists_call_may_need_resolution,
    },
    loop_values::ScssControlFlowLoopContext,
    model::OmenaScssEvalCallReturnNodeV0,
    tokens::{
        matching_block_end_token_index, next_block_start_token_index, tokens_between_are_trivia,
    },
    variables::canonical_scss_variable_name,
};

pub(super) fn collect_scss_return_candidates(
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
            let return_loop_contexts = scss_return_loop_contexts(source, tokens, index);
            let return_loop_header_texts = return_loop_contexts
                .iter()
                .map(|context| context.header_text.clone())
                .collect::<Vec<_>>();
            let return_loop_body_texts = return_loop_contexts
                .iter()
                .map(|context| context.body_text.clone().unwrap_or_default())
                .collect::<Vec<_>>();
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
                return_loop_body_texts,
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

fn scss_return_loop_contexts(
    source: &str,
    tokens: &[LexedToken],
    return_index: usize,
) -> Vec<ScssControlFlowLoopContext> {
    enclosing_scss_loop_blocks(tokens, return_index)
        .into_iter()
        .filter_map(|block| {
            let header_text = control_flow_header_text(source, tokens, block.at_rule_index);
            if header_text.is_empty() {
                return None;
            }
            let body_start = tokens.get(block.body_start_index)?.range.end().into();
            let body_end = tokens.get(block.body_end_index)?.range.start().into();
            let body_text = source.get(body_start..body_end).map(str::to_string);
            Some(ScssControlFlowLoopContext {
                header_text,
                body_text,
            })
        })
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
            let body_start_index = next_block_start_token_index(tokens, index + 1)?;
            let body_end_index = matching_block_end_token_index(tokens, body_start_index)?;
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

pub(super) fn static_scss_return_abstract_value(value: &str) -> AbstractCssValueV0 {
    abstract_css_value_from_text(reduce_static_scss_value(value.to_string()).as_str())
}

pub(super) fn static_scss_return_abstract_value_with_context(
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
            let body_start_index = next_block_start_token_index(tokens, index + 1)?;
            let body_end_index = matching_block_end_token_index(tokens, body_start_index)?;
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
