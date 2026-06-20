use std::collections::BTreeMap;

use omena_parser::{LexedToken, StyleDialect, lex};
use omena_syntax::SyntaxKind;

use crate::value_eval::static_scss_literal_truthiness;

use super::{
    STATIC_STYLESHEET_VALUE_RESOLUTION_FUEL_LIMIT, StaticScssFunctionResolutionContext,
    StaticStylesheetEvaluationEdit, StaticStylesheetResolutionOutcome,
    apply_static_stylesheet_evaluation_edits, resolve_static_scss_function_value_with_bindings,
    static_stylesheet_matching_token_index, static_stylesheet_skip_trivia_tokens,
    static_stylesheet_token_end, static_stylesheet_token_start,
};

const STATIC_SCSS_MIXIN_CONTROL_FLOW_RENDER_LIMIT: usize = 32;

pub(super) fn render_static_scss_mixin_control_flow_body(
    body: &str,
    dialect: StyleDialect,
    argument_values: &BTreeMap<String, String>,
    call_position: usize,
    context: StaticScssFunctionResolutionContext<'_>,
) -> Option<String> {
    if dialect != StyleDialect::Scss || !body.to_ascii_lowercase().contains("@if") {
        return Some(body.to_string());
    }
    render_static_scss_mixin_control_flow_body_with_fuel(
        body,
        argument_values,
        call_position,
        context,
        STATIC_SCSS_MIXIN_CONTROL_FLOW_RENDER_LIMIT,
    )
}

fn render_static_scss_mixin_control_flow_body_with_fuel(
    body: &str,
    argument_values: &BTreeMap<String, String>,
    call_position: usize,
    context: StaticScssFunctionResolutionContext<'_>,
    fuel: usize,
) -> Option<String> {
    if fuel == 0 {
        return None;
    }
    let lexed = lex(body, StyleDialect::Scss);
    let tokens = lexed.tokens();
    let mut edits = Vec::new();
    let mut index = 0usize;
    while index < tokens.len() {
        let token = &tokens[index];
        if token.kind != SyntaxKind::AtKeyword || !token.text.eq_ignore_ascii_case("@if") {
            index += 1;
            continue;
        }
        let chain = static_scss_mixin_control_flow_chain(body, tokens, index)?;
        let replacement = static_scss_mixin_selected_control_flow_body(
            &chain,
            argument_values,
            call_position,
            context,
        )?
        .unwrap_or_default();
        edits.push(StaticStylesheetEvaluationEdit {
            start: chain.start,
            end: chain.end,
            replacement,
        });
        index = chain.next_index;
    }

    if edits.is_empty() {
        return Some(body.to_string());
    }
    let rendered = apply_static_stylesheet_evaluation_edits(body, edits)?;
    if rendered.to_ascii_lowercase().contains("@if") {
        return render_static_scss_mixin_control_flow_body_with_fuel(
            rendered.as_str(),
            argument_values,
            call_position,
            context,
            fuel - 1,
        );
    }
    Some(rendered)
}

#[derive(Debug, Clone)]
struct StaticScssMixinControlFlowChain {
    start: usize,
    end: usize,
    next_index: usize,
    branches: Vec<StaticScssMixinControlFlowBranch>,
}

#[derive(Debug, Clone)]
struct StaticScssMixinControlFlowBranch {
    condition: Option<String>,
    body: String,
}

fn static_scss_mixin_control_flow_chain(
    source: &str,
    tokens: &[LexedToken],
    if_index: usize,
) -> Option<StaticScssMixinControlFlowChain> {
    let (condition, body_open_index, body_close_index) =
        static_scss_mixin_control_flow_header_and_body(source, tokens, if_index)?;
    let start = static_stylesheet_token_start(&tokens[if_index]);
    let mut end = static_stylesheet_token_end(&tokens[body_close_index]);
    let mut next_index = body_close_index + 1;
    let mut branches = vec![StaticScssMixinControlFlowBranch {
        condition: Some(condition),
        body: static_scss_mixin_control_flow_branch_body(
            source,
            tokens,
            body_open_index,
            body_close_index,
        )?,
    }];

    loop {
        let else_index = static_stylesheet_skip_trivia_tokens(tokens, next_index);
        let Some(token) = tokens.get(else_index) else {
            break;
        };
        if token.kind != SyntaxKind::AtKeyword || !token.text.eq_ignore_ascii_case("@else") {
            break;
        }
        let (header, else_body_open_index, else_body_close_index) =
            static_scss_mixin_control_flow_header_and_body(source, tokens, else_index)?;
        let condition = static_scss_mixin_else_if_condition(header.as_str()).map(ToOwned::to_owned);
        branches.push(StaticScssMixinControlFlowBranch {
            condition,
            body: static_scss_mixin_control_flow_branch_body(
                source,
                tokens,
                else_body_open_index,
                else_body_close_index,
            )?,
        });
        end = static_stylesheet_token_end(&tokens[else_body_close_index]);
        next_index = else_body_close_index + 1;
    }

    Some(StaticScssMixinControlFlowChain {
        start,
        end,
        next_index,
        branches,
    })
}

fn static_scss_mixin_control_flow_header_and_body(
    source: &str,
    tokens: &[LexedToken],
    control_index: usize,
) -> Option<(String, usize, usize)> {
    let body_open_index = (control_index + 1..tokens.len())
        .find(|index| tokens[*index].kind == SyntaxKind::LeftBrace)?;
    let body_close_index = static_stylesheet_matching_token_index(
        tokens,
        body_open_index,
        SyntaxKind::LeftBrace,
        SyntaxKind::RightBrace,
    )?;
    let header_start = static_stylesheet_token_end(&tokens[control_index]);
    let header_end = static_stylesheet_token_start(&tokens[body_open_index]);
    let header = source.get(header_start..header_end)?.trim().to_string();
    Some((header, body_open_index, body_close_index))
}

fn static_scss_mixin_control_flow_branch_body(
    source: &str,
    tokens: &[LexedToken],
    body_open_index: usize,
    body_close_index: usize,
) -> Option<String> {
    let body_start = static_stylesheet_token_end(&tokens[body_open_index]);
    let body_end = static_stylesheet_token_start(&tokens[body_close_index]);
    Some(source.get(body_start..body_end)?.to_string())
}

fn static_scss_mixin_else_if_condition(header: &str) -> Option<&str> {
    let trimmed = header.trim();
    if trimmed.is_empty() {
        return None;
    }
    let prefix = trimmed.get(..2)?;
    let rest = trimmed.get(2..)?;
    if !prefix.eq_ignore_ascii_case("if") || !rest.chars().next().is_some_and(char::is_whitespace) {
        return None;
    }
    Some(rest.trim()).filter(|condition| !condition.is_empty())
}

fn static_scss_mixin_selected_control_flow_body(
    chain: &StaticScssMixinControlFlowChain,
    argument_values: &BTreeMap<String, String>,
    call_position: usize,
    context: StaticScssFunctionResolutionContext<'_>,
) -> Option<Option<String>> {
    for branch in &chain.branches {
        let Some(condition) = branch.condition.as_ref() else {
            return Some(Some(branch.body.clone()));
        };
        let resolution = resolve_static_scss_function_value_with_bindings(
            condition.as_str(),
            argument_values,
            call_position,
            STATIC_STYLESHEET_VALUE_RESOLUTION_FUEL_LIMIT,
            context,
        );
        if resolution.outcome == StaticStylesheetResolutionOutcome::Top {
            return None;
        }
        let condition_value = resolution.rendered_value?;
        if static_scss_literal_truthiness(condition_value.as_str())? {
            return Some(Some(branch.body.clone()));
        }
    }
    Some(None)
}
