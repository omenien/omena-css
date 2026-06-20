use std::collections::{BTreeMap, BTreeSet};

use omena_parser::{LexedToken, StyleDialect, lex};
use omena_syntax::SyntaxKind;

use crate::value_eval::static_scss_literal_truthiness;

use super::{
    OmenaScssEvalResolvedReplacementV0, STATIC_STYLESHEET_VALUE_RESOLUTION_FUEL_LIMIT,
    StaticScssFunctionCall, StaticScssFunctionDeclaration, StaticScssFunctionResolutionContext,
    StaticStylesheetEvaluationEdit, StaticStylesheetResolutionOutcome,
    StaticStylesheetVariableKind, apply_static_stylesheet_evaluation_edits,
    canonical_static_scss_function_name, collect_static_stylesheet_variable_references,
    extend_static_scss_used_function_dependencies,
    resolve_static_scss_function_call_abstract_value,
    resolve_static_scss_function_value_with_bindings, resolved_replacement_value,
    split_static_scss_function_arguments, static_stylesheet_matching_token_index,
    static_stylesheet_position_is_inside_ranges, static_stylesheet_skip_trivia_tokens,
    static_stylesheet_token_end, static_stylesheet_token_start,
    tokens::static_stylesheet_block_kinds_for_dialect,
};

const STATIC_SCSS_MIXIN_CONTROL_FLOW_RENDER_LIMIT: usize = 32;

pub(super) fn render_static_scss_mixin_control_flow_body(
    body: &str,
    dialect: StyleDialect,
    argument_values: &BTreeMap<String, String>,
    call_position: usize,
    context: StaticScssFunctionResolutionContext<'_>,
) -> Option<String> {
    if !static_scss_control_flow_dialect_is_supported(dialect)
        || !body.to_ascii_lowercase().contains("@if")
    {
        return Some(body.to_string());
    }
    render_static_scss_mixin_control_flow_body_with_fuel(
        body,
        dialect,
        argument_values,
        call_position,
        context,
        STATIC_SCSS_MIXIN_CONTROL_FLOW_RENDER_LIMIT,
    )
}

pub(super) fn collect_static_scss_control_flow_evaluation_edits(
    source: &str,
    dialect: StyleDialect,
    tokens: &[LexedToken],
    excluded_ranges: &[(usize, usize)],
    context: StaticScssFunctionResolutionContext<'_>,
) -> Option<StaticScssControlFlowEvaluationEdits> {
    if !static_scss_control_flow_dialect_is_supported(dialect)
        || !source.to_ascii_lowercase().contains("@if")
    {
        return Some(StaticScssControlFlowEvaluationEdits {
            edits: Vec::new(),
            replacements: Vec::new(),
            preserved_dynamic_branch_count: 0,
        });
    }

    let argument_values = BTreeMap::new();
    let mut edits = Vec::new();
    let mut replacements = Vec::new();
    let mut used_function_declaration_names = BTreeSet::new();
    let mut preserved_dynamic_branch_count = 0usize;
    let mut index = 0usize;
    while index < tokens.len() {
        let token = &tokens[index];
        if token.kind != SyntaxKind::AtKeyword || !token.text.eq_ignore_ascii_case("@if") {
            index += 1;
            continue;
        }
        let start = static_stylesheet_token_start(token);
        if static_stylesheet_position_is_inside_ranges(start, excluded_ranges) {
            index += 1;
            continue;
        }

        let chain = static_scss_mixin_control_flow_chain(source, dialect, tokens, index)?;
        let Some(replacement) = static_scss_mixin_selected_control_flow_body(
            &chain,
            &argument_values,
            chain.start,
            context,
        ) else {
            preserved_dynamic_branch_count += 1;
            index = chain.next_index;
            continue;
        };
        let selected = replacement.unwrap_or_else(StaticScssSelectedControlFlowBody::empty);
        let StaticScssControlFlowReplacementRender {
            replacement,
            replacements: branch_replacements,
            used_function_declaration_names: branch_used_function_declaration_names,
        } = render_static_scss_control_flow_replacement_values(
            selected.body.as_str(),
            selected.body_start,
            context,
        )?;
        let replacement = render_static_scss_mixin_control_flow_body(
            replacement.as_str(),
            dialect,
            &argument_values,
            chain.start,
            context,
        )?;
        if !static_scss_control_flow_replacement_is_static_css_subset(replacement.as_str()) {
            preserved_dynamic_branch_count += 1;
            index = chain.next_index;
            continue;
        }
        replacements.extend(branch_replacements);
        used_function_declaration_names.extend(branch_used_function_declaration_names);
        edits.push(StaticStylesheetEvaluationEdit {
            start: chain.start,
            end: chain.end,
            replacement,
        });
        index = chain.next_index;
    }

    if preserved_dynamic_branch_count == 0 {
        extend_static_scss_used_function_dependencies(
            &mut used_function_declaration_names,
            context.declarations,
        );
        for declaration in context.declarations.iter().filter(|declaration| {
            used_function_declaration_names.contains(&canonical_static_scss_function_name(
                declaration.name.as_str(),
            ))
        }) {
            edits.push(StaticStylesheetEvaluationEdit {
                start: declaration.span_start,
                end: declaration.span_end,
                replacement: String::new(),
            });
        }
    }

    Some(StaticScssControlFlowEvaluationEdits {
        edits,
        replacements,
        preserved_dynamic_branch_count,
    })
}

pub(super) struct StaticScssControlFlowEvaluationEdits {
    pub(super) edits: Vec<StaticStylesheetEvaluationEdit>,
    pub(super) replacements: Vec<OmenaScssEvalResolvedReplacementV0>,
    pub(super) preserved_dynamic_branch_count: usize,
}

fn render_static_scss_mixin_control_flow_body_with_fuel(
    body: &str,
    dialect: StyleDialect,
    argument_values: &BTreeMap<String, String>,
    call_position: usize,
    context: StaticScssFunctionResolutionContext<'_>,
    fuel: usize,
) -> Option<String> {
    if fuel == 0 {
        return None;
    }
    let lexed = lex(body, dialect);
    let tokens = lexed.tokens();
    let mut edits = Vec::new();
    let mut index = 0usize;
    while index < tokens.len() {
        let token = &tokens[index];
        if token.kind != SyntaxKind::AtKeyword || !token.text.eq_ignore_ascii_case("@if") {
            index += 1;
            continue;
        }
        let chain = static_scss_mixin_control_flow_chain(body, dialect, tokens, index)?;
        let replacement = static_scss_mixin_selected_control_flow_body(
            &chain,
            argument_values,
            call_position,
            context,
        )?
        .map(|selected| selected.body)
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
            dialect,
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
    body_start: usize,
}

struct StaticScssSelectedControlFlowBody {
    body: String,
    body_start: usize,
}

struct StaticScssControlFlowReplacementRender {
    replacement: String,
    replacements: Vec<OmenaScssEvalResolvedReplacementV0>,
    used_function_declaration_names: BTreeSet<String>,
}

impl StaticScssSelectedControlFlowBody {
    fn empty() -> Self {
        Self {
            body: String::new(),
            body_start: 0,
        }
    }
}

fn static_scss_mixin_control_flow_chain(
    source: &str,
    dialect: StyleDialect,
    tokens: &[LexedToken],
    if_index: usize,
) -> Option<StaticScssMixinControlFlowChain> {
    let (condition, body_open_index, body_close_index) =
        static_scss_mixin_control_flow_header_and_body(source, dialect, tokens, if_index)?;
    let start = static_stylesheet_token_start(&tokens[if_index]);
    let mut end = static_stylesheet_token_end(&tokens[body_close_index]);
    let mut next_index = body_close_index + 1;
    let mut branches = vec![StaticScssMixinControlFlowBranch {
        condition: Some(condition),
        body_start: static_stylesheet_token_end(&tokens[body_open_index]),
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
            static_scss_mixin_control_flow_header_and_body(source, dialect, tokens, else_index)?;
        let condition = static_scss_mixin_else_if_condition(header.as_str()).map(ToOwned::to_owned);
        branches.push(StaticScssMixinControlFlowBranch {
            condition,
            body_start: static_stylesheet_token_end(&tokens[else_body_open_index]),
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
    dialect: StyleDialect,
    tokens: &[LexedToken],
    control_index: usize,
) -> Option<(String, usize, usize)> {
    let (body_open_kind, body_close_kind) = static_stylesheet_block_kinds_for_dialect(dialect);
    let body_open_index =
        (control_index + 1..tokens.len()).find(|index| tokens[*index].kind == body_open_kind)?;
    let body_close_index = static_stylesheet_matching_token_index(
        tokens,
        body_open_index,
        body_open_kind,
        body_close_kind,
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

const fn static_scss_control_flow_dialect_is_supported(dialect: StyleDialect) -> bool {
    matches!(dialect, StyleDialect::Scss | StyleDialect::Sass)
}

fn static_scss_mixin_selected_control_flow_body(
    chain: &StaticScssMixinControlFlowChain,
    argument_values: &BTreeMap<String, String>,
    call_position: usize,
    context: StaticScssFunctionResolutionContext<'_>,
) -> Option<Option<StaticScssSelectedControlFlowBody>> {
    for branch in &chain.branches {
        let Some(condition) = branch.condition.as_ref() else {
            return Some(Some(StaticScssSelectedControlFlowBody {
                body: branch.body.clone(),
                body_start: branch.body_start,
            }));
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
            return Some(Some(StaticScssSelectedControlFlowBody {
                body: branch.body.clone(),
                body_start: branch.body_start,
            }));
        }
    }
    Some(None)
}

fn render_static_scss_control_flow_replacement_values(
    replacement: &str,
    source_body_start: usize,
    context: StaticScssFunctionResolutionContext<'_>,
) -> Option<StaticScssControlFlowReplacementRender> {
    let branch_lexed = lex(replacement, StyleDialect::Scss);
    let function_calls = collect_static_scss_control_flow_function_calls(
        replacement,
        branch_lexed.tokens(),
        context.declarations,
    )?;
    let function_call_ranges = function_calls
        .iter()
        .map(|call| (call.start, call.end))
        .collect::<Vec<_>>();
    let references = collect_static_stylesheet_variable_references(
        replacement,
        StaticStylesheetVariableKind::Scss,
    )?;

    let mut edits = Vec::new();
    let argument_values = BTreeMap::new();
    let mut replacements = Vec::new();
    let mut used_function_declaration_names = BTreeSet::new();
    for call in function_calls {
        let mut original_call = call.clone();
        original_call.start = source_body_start + call.start;
        original_call.end = source_body_start + call.end;
        let resolution = resolve_static_scss_function_call_abstract_value(
            &original_call,
            context.dialect,
            context.declarations,
            context.mixin_declarations,
            context.scopes,
            context.variable_declarations,
            STATIC_STYLESHEET_VALUE_RESOLUTION_FUEL_LIMIT,
        );
        if resolution.outcome != StaticStylesheetResolutionOutcome::Resolved {
            return None;
        }
        let rendered_value = resolution.rendered_value?;
        used_function_declaration_names.insert(canonical_static_scss_function_name(
            original_call.name.as_str(),
        ));
        replacements.push(resolved_replacement_value(
            format!("function:{}", original_call.name).as_str(),
            original_call.start,
            original_call.end,
            rendered_value.as_str(),
        ));
        edits.push(StaticStylesheetEvaluationEdit {
            start: call.start,
            end: call.end,
            replacement: rendered_value,
        });
    }

    for reference in references {
        if static_stylesheet_position_is_inside_ranges(reference.start, &function_call_ranges) {
            continue;
        }
        let original_start = source_body_start + reference.start;
        let original_end = source_body_start + reference.end;
        let resolution = resolve_static_scss_function_value_with_bindings(
            reference.name.as_str(),
            &argument_values,
            original_start,
            STATIC_STYLESHEET_VALUE_RESOLUTION_FUEL_LIMIT,
            context,
        );
        if resolution.outcome != StaticStylesheetResolutionOutcome::Resolved {
            return None;
        }
        let rendered_value = resolution.rendered_value?;
        replacements.push(resolved_replacement_value(
            reference.name.as_str(),
            original_start,
            original_end,
            rendered_value.as_str(),
        ));
        edits.push(StaticStylesheetEvaluationEdit {
            start: reference.start,
            end: reference.end,
            replacement: rendered_value,
        });
    }

    Some(StaticScssControlFlowReplacementRender {
        replacement: apply_static_stylesheet_evaluation_edits(replacement, edits)?,
        replacements,
        used_function_declaration_names,
    })
}

fn collect_static_scss_control_flow_function_calls(
    source: &str,
    tokens: &[LexedToken],
    declarations: &[StaticScssFunctionDeclaration],
) -> Option<Vec<StaticScssFunctionCall>> {
    let declaration_names = declarations
        .iter()
        .map(|declaration| canonical_static_scss_function_name(declaration.name.as_str()))
        .collect::<BTreeSet<_>>();
    let mut calls = Vec::new();
    for (name_index, token) in tokens.iter().enumerate() {
        if token.kind != SyntaxKind::Ident
            || !declaration_names
                .contains(&canonical_static_scss_function_name(token.text.as_str()))
        {
            continue;
        }
        let open_index = static_stylesheet_skip_trivia_tokens(tokens, name_index + 1);
        if tokens
            .get(open_index)
            .is_none_or(|token| token.kind != SyntaxKind::LeftParen)
        {
            continue;
        }
        let close_index = static_stylesheet_matching_token_index(
            tokens,
            open_index,
            SyntaxKind::LeftParen,
            SyntaxKind::RightParen,
        )?;
        let arguments = split_static_scss_function_arguments(source.get(
            static_stylesheet_token_end(&tokens[open_index])
                ..static_stylesheet_token_start(&tokens[close_index]),
        )?)?;
        calls.push(StaticScssFunctionCall {
            name: token.text.clone(),
            start: static_stylesheet_token_start(token),
            end: static_stylesheet_token_end(&tokens[close_index]),
            arguments,
        });
    }
    calls.sort_by_key(|call| (call.start, call.end));
    Some(calls)
}

fn static_scss_control_flow_replacement_is_static_css_subset(replacement: &str) -> bool {
    let lower = replacement.to_ascii_lowercase();
    !replacement.contains('$')
        && !lower.contains("@mixin")
        && !lower.contains("@function")
        && !lower.contains("@return")
        && !lower.contains("@include")
        && !lower.contains("@content")
        && !lower.contains("@for")
        && !lower.contains("@each")
        && !lower.contains("@while")
}
