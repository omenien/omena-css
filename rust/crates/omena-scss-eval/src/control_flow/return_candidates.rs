use std::collections::BTreeMap;

use cstree::syntax::SyntaxNode;
use omena_abstract_value::{AbstractCssValueV0, abstract_css_value_from_text};
use omena_parser::{LexedToken, StyleDialect, lex};
use omena_syntax::SyntaxKind;

use crate::{
    scss_metadata::reduce_static_scss_metadata_with_context, value_eval::reduce_static_scss_value,
};

use super::{
    analysis_model::{
        ScssBranchBlock, ScssCallReturnCandidate, ScssGlobalVariableDeclaration,
        ScssReturnCondition,
    },
    blocks::{
        control_flow_blocks_from_cst, control_flow_header_text, scss_else_if_header_condition,
    },
    call_resolution::{
        scss_visible_function_declaration_exists, scss_visible_mixin_declaration_exists,
    },
    lexical::{
        scss_global_variable_metadata_exists, static_scss_metadata_exists_call_may_need_resolution,
    },
    loop_values::{ScssControlFlowLoopContext, control_flow_block_body_text},
    model::OmenaScssEvalCallReturnNodeV0,
    tokens::{
        matching_block_end_token_index, next_block_start_token_index, tokens_between_are_trivia,
    },
    variables::canonical_scss_variable_name,
};

#[derive(Debug, Clone, PartialEq, Eq)]
struct ScssCstBranchBlock {
    at_rule_name: String,
    condition_text: Option<String>,
    source_span_start: usize,
    source_span_end: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ScssCstLoopBlock {
    header_text: String,
    body_text: Option<String>,
    source_span_start: usize,
    source_span_end: usize,
}

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

pub(super) fn collect_scss_return_candidates_from_cst(
    source: &str,
    root: &SyntaxNode<SyntaxKind>,
    dialect: StyleDialect,
) -> Vec<ScssCallReturnCandidate> {
    let blocks = control_flow_blocks_from_cst(source, root, dialect);
    let branch_blocks = cst_branch_blocks(blocks.as_slice());
    let loop_blocks = cst_loop_blocks(source, blocks.as_slice());
    root.descendants()
        .filter(|node| node.kind() == SyntaxKind::ScssReturnRule)
        .filter_map(|node| {
            let at_keyword = cst_first_at_keyword_token(node)?;
            let source_span_start = u32::from(at_keyword.text_range().start()) as usize;
            let source_span_end = u32::from(at_keyword.text_range().end()) as usize;
            let return_text = cst_return_text(source, node, source_span_end);
            let return_loop_contexts = cst_return_loop_contexts(&loop_blocks, source_span_start);
            let return_loop_header_texts = return_loop_contexts
                .iter()
                .map(|context| context.header_text.clone())
                .collect::<Vec<_>>();
            let return_loop_body_texts = return_loop_contexts
                .iter()
                .map(|context| context.body_text.clone().unwrap_or_default())
                .collect::<Vec<_>>();
            let return_inside_loop_control_flow = !return_loop_contexts.is_empty();
            let return_loop_header_text = return_loop_header_texts.last().cloned();
            let return_value = if return_inside_loop_control_flow {
                Some(AbstractCssValueV0::Top)
            } else {
                return_text
                    .as_deref()
                    .map(static_scss_return_abstract_value)
            };
            let return_condition =
                cst_return_condition(source, dialect, &branch_blocks, source_span_start);
            Some(ScssCallReturnCandidate {
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
                source_span_start,
                source_span_end,
            })
        })
        .collect()
}

fn cst_branch_blocks(
    blocks: &[super::model::OmenaScssEvalControlFlowBlockV0],
) -> Vec<ScssCstBranchBlock> {
    blocks
        .iter()
        .filter(|block| block.kind.starts_with("branch"))
        .map(|block| {
            let condition_text = if block.kind == "branchIf" {
                Some(block.header_text.clone()).filter(|header| !header.is_empty())
            } else {
                scss_else_if_header_condition(block.header_text.as_str()).map(ToString::to_string)
            };
            ScssCstBranchBlock {
                at_rule_name: block.at_rule_name.to_ascii_lowercase(),
                condition_text,
                source_span_start: block.source_span_start,
                source_span_end: block.source_span_end,
            }
        })
        .collect()
}

fn cst_loop_blocks(
    source: &str,
    blocks: &[super::model::OmenaScssEvalControlFlowBlockV0],
) -> Vec<ScssCstLoopBlock> {
    blocks
        .iter()
        .filter(|block| block.kind == "loop")
        .map(|block| ScssCstLoopBlock {
            header_text: block.header_text.clone(),
            body_text: control_flow_block_body_text(source, block).map(ToString::to_string),
            source_span_start: block.source_span_start,
            source_span_end: block.source_span_end,
        })
        .collect()
}

fn cst_return_loop_contexts(
    loop_blocks: &[ScssCstLoopBlock],
    return_start: usize,
) -> Vec<ScssControlFlowLoopContext> {
    let mut enclosing = loop_blocks
        .iter()
        .filter(|block| {
            block.source_span_start < return_start && return_start < block.source_span_end
        })
        .collect::<Vec<_>>();
    enclosing.sort_by(|left, right| {
        let left_span = left.source_span_end.saturating_sub(left.source_span_start);
        let right_span = right
            .source_span_end
            .saturating_sub(right.source_span_start);
        right_span.cmp(&left_span)
    });
    enclosing
        .into_iter()
        .map(|block| ScssControlFlowLoopContext {
            header_text: block.header_text.clone(),
            body_text: block.body_text.clone(),
        })
        .collect()
}

fn cst_return_condition(
    source: &str,
    dialect: StyleDialect,
    branch_blocks: &[ScssCstBranchBlock],
    return_start: usize,
) -> Option<ScssReturnCondition> {
    let (block_index, block) = branch_blocks
        .iter()
        .enumerate()
        .filter(|(_, block)| {
            block.source_span_start < return_start && return_start < block.source_span_end
        })
        .min_by_key(|(_, block)| {
            block
                .source_span_end
                .saturating_sub(block.source_span_start)
        })?;
    let negated_condition_texts =
        previous_cst_branch_condition_texts(source, dialect, branch_blocks, block_index);
    Some(ScssReturnCondition {
        condition_text: block.condition_text.clone(),
        negated_condition_texts,
    })
}

fn previous_cst_branch_condition_texts(
    source: &str,
    dialect: StyleDialect,
    branch_blocks: &[ScssCstBranchBlock],
    block_index: usize,
) -> Vec<String> {
    let Some(current_block) = branch_blocks.get(block_index) else {
        return Vec::new();
    };
    if current_block.at_rule_name != "@else" {
        return Vec::new();
    }

    let mut conditions = Vec::new();
    let mut cursor = current_block.source_span_start;
    for candidate in branch_blocks[..block_index].iter().rev() {
        if candidate.source_span_end > cursor
            || !cst_source_between_is_trivia(source, dialect, candidate.source_span_end, cursor)
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
        cursor = candidate.source_span_start;
    }
    Vec::new()
}

fn cst_source_between_is_trivia(
    source: &str,
    dialect: StyleDialect,
    start: usize,
    end: usize,
) -> bool {
    if start > end {
        return false;
    }
    let Some(text) = source.get(start..end) else {
        return false;
    };
    lex(text, dialect).tokens().iter().all(|token| {
        matches!(
            token.kind,
            SyntaxKind::Whitespace
                | SyntaxKind::LineComment
                | SyntaxKind::BlockComment
                | SyntaxKind::ScssSilentComment
                | SyntaxKind::SassIndentedNewline
                | SyntaxKind::SassDedent
        )
    })
}

fn cst_first_at_keyword_token(
    node: &SyntaxNode<SyntaxKind>,
) -> Option<cstree::syntax::SyntaxToken<SyntaxKind>> {
    node.descendants_with_tokens()
        .filter_map(|element| element.into_token())
        .find(|token| token.kind() == SyntaxKind::AtKeyword)
        .cloned()
}

fn cst_return_text(
    source: &str,
    node: &SyntaxNode<SyntaxKind>,
    value_start: usize,
) -> Option<String> {
    let value_end = node
        .descendants_with_tokens()
        .filter_map(|element| element.into_token())
        .find(|token| {
            let token_start = u32::from(token.text_range().start()) as usize;
            token_start >= value_start
                && matches!(
                    token.kind(),
                    SyntaxKind::Semicolon
                        | SyntaxKind::SassOptionalSemicolon
                        | SyntaxKind::SassIndentedNewline
                        | SyntaxKind::RightBrace
                )
        })
        .map(|token| u32::from(token.text_range().start()) as usize)
        .unwrap_or_else(|| u32::from(node.text_range().end()) as usize);
    source
        .get(value_start..value_end)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
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
