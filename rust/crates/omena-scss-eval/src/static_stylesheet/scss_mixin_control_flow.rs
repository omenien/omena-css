use std::collections::{BTreeMap, BTreeSet};

use omena_abstract_value::{AbstractCssValueV0, abstract_css_value_from_text};
use omena_parser::{LexedToken, StyleDialect, lex};
use omena_syntax::SyntaxKind;

use crate::control_flow::{
    analyze_scss_control_flow_values_with_initial_bindings, build_scss_control_flow_graph,
    summarize_scss_control_flow_prune_reachability_with_initial_bindings,
};
use crate::value_eval::{static_scss_literal_truthiness, static_scss_typed_advisory_truthiness};

use super::model::StaticScssControlFlowPruneEvidenceCounts;
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
) -> Option<StaticScssControlFlowBodyRender> {
    if !static_scss_control_flow_dialect_is_supported(dialect)
        || !body.to_ascii_lowercase().contains("@if")
    {
        return Some(StaticScssControlFlowBodyRender {
            body: body.to_string(),
            prune_evidence_counts: StaticScssControlFlowPruneEvidenceCounts::default(),
        });
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
            prune_evidence_counts: StaticScssControlFlowPruneEvidenceCounts::default(),
        });
    }

    let argument_values = BTreeMap::new();
    let mut edits = Vec::new();
    let mut replacements = Vec::new();
    let mut used_function_declaration_names = BTreeSet::new();
    let mut preserved_dynamic_branch_count = 0usize;
    let mut prune_evidence_counts = StaticScssControlFlowPruneEvidenceCounts::default();
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
        let value_prune_plan = static_scss_control_flow_value_prune_plan(
            source,
            dialect,
            &chain,
            &argument_values,
            chain.start,
            context,
        );
        prune_evidence_counts.add_assign(value_prune_plan.evidence_counts);
        let Some(replacement) =
            static_scss_mixin_selected_control_flow_body(&chain, Some(&value_prune_plan))
        else {
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
        let rendered_control_flow = render_static_scss_mixin_control_flow_body(
            replacement.as_str(),
            dialect,
            &argument_values,
            chain.start,
            context,
        )?;
        prune_evidence_counts.add_assign(rendered_control_flow.prune_evidence_counts);
        let replacement = rendered_control_flow.body;
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
        prune_evidence_counts,
    })
}

pub(super) struct StaticScssControlFlowEvaluationEdits {
    pub(super) edits: Vec<StaticStylesheetEvaluationEdit>,
    pub(super) replacements: Vec<OmenaScssEvalResolvedReplacementV0>,
    pub(super) preserved_dynamic_branch_count: usize,
    pub(super) prune_evidence_counts: StaticScssControlFlowPruneEvidenceCounts,
}

pub(super) struct StaticScssControlFlowBodyRender {
    pub(super) body: String,
    pub(super) prune_evidence_counts: StaticScssControlFlowPruneEvidenceCounts,
}

fn render_static_scss_mixin_control_flow_body_with_fuel(
    body: &str,
    dialect: StyleDialect,
    argument_values: &BTreeMap<String, String>,
    call_position: usize,
    context: StaticScssFunctionResolutionContext<'_>,
    fuel: usize,
) -> Option<StaticScssControlFlowBodyRender> {
    if fuel == 0 {
        return None;
    }
    let lexed = lex(body, dialect);
    let tokens = lexed.tokens();
    let mut edits = Vec::new();
    let mut prune_evidence_counts = StaticScssControlFlowPruneEvidenceCounts::default();
    let mut index = 0usize;
    while index < tokens.len() {
        let token = &tokens[index];
        if token.kind != SyntaxKind::AtKeyword || !token.text.eq_ignore_ascii_case("@if") {
            index += 1;
            continue;
        }
        let chain = static_scss_mixin_control_flow_chain(body, dialect, tokens, index)?;
        let value_prune_plan = static_scss_control_flow_value_prune_plan(
            body,
            dialect,
            &chain,
            argument_values,
            call_position,
            context,
        );
        prune_evidence_counts.add_assign(value_prune_plan.evidence_counts);
        let replacement =
            static_scss_mixin_selected_control_flow_body(&chain, Some(&value_prune_plan))?
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
        return Some(StaticScssControlFlowBodyRender {
            body: body.to_string(),
            prune_evidence_counts,
        });
    }
    let rendered = apply_static_stylesheet_evaluation_edits(body, edits)?;
    if rendered.to_ascii_lowercase().contains("@if") {
        let mut nested = render_static_scss_mixin_control_flow_body_with_fuel(
            rendered.as_str(),
            dialect,
            argument_values,
            call_position,
            context,
            fuel - 1,
        )?;
        prune_evidence_counts.add_assign(nested.prune_evidence_counts);
        nested.prune_evidence_counts = prune_evidence_counts;
        return Some(nested);
    }
    Some(StaticScssControlFlowBodyRender {
        body: rendered,
        prune_evidence_counts,
    })
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
    control_start: usize,
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
        control_start: start,
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
            control_start: static_stylesheet_token_start(&tokens[else_index]),
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
    value_prune_plan: Option<&StaticScssControlFlowValuePrunePlan>,
) -> Option<Option<StaticScssSelectedControlFlowBody>> {
    for branch in &chain.branches {
        if value_prune_plan
            .is_some_and(|plan| !plan.control_start_is_reachable(branch.control_start))
        {
            continue;
        }
        if branch.condition.is_none() {
            return Some(Some(StaticScssSelectedControlFlowBody {
                body: branch.body.clone(),
                body_start: branch.body_start,
            }));
        }
        if value_prune_plan
            .is_some_and(|plan| plan.control_start_has_conflict(branch.control_start))
        {
            return None;
        }
        let truthy = value_prune_plan
            .and_then(|plan| plan.truthiness_by_start.get(&branch.control_start))
            .copied()?;
        if truthy {
            return Some(Some(StaticScssSelectedControlFlowBody {
                body: branch.body.clone(),
                body_start: branch.body_start,
            }));
        }
    }
    Some(None)
}

#[derive(Debug, Clone, Default)]
struct StaticScssControlFlowValuePrunePlan {
    truthiness_by_start: BTreeMap<usize, bool>,
    conflicting_control_starts: BTreeSet<usize>,
    reachable_control_starts: Option<BTreeSet<usize>>,
    evidence_counts: StaticScssControlFlowPruneEvidenceCounts,
}

impl StaticScssControlFlowValuePrunePlan {
    fn control_start_has_conflict(&self, control_start: usize) -> bool {
        self.conflicting_control_starts.contains(&control_start)
    }

    fn control_start_is_reachable(&self, control_start: usize) -> bool {
        self.reachable_control_starts
            .as_ref()
            .is_none_or(|starts| starts.contains(&control_start))
    }
}

fn static_scss_control_flow_value_prune_plan(
    source: &str,
    dialect: StyleDialect,
    chain: &StaticScssMixinControlFlowChain,
    argument_values: &BTreeMap<String, String>,
    call_position: usize,
    context: StaticScssFunctionResolutionContext<'_>,
) -> StaticScssControlFlowValuePrunePlan {
    let initial_bindings = static_scss_control_flow_initial_bindings(argument_values);
    let mut truthiness_by_start =
        static_scss_control_flow_value_truthiness_by_start(source, dialect, &initial_bindings);
    let mut evidence_counts = StaticScssControlFlowPruneEvidenceCounts {
        value_truthiness_count: truthiness_by_start.len(),
        ..Default::default()
    };
    let mut conflicting_control_starts = BTreeSet::new();
    for branch in &chain.branches {
        let Some(condition) = branch.condition.as_ref() else {
            continue;
        };
        let Some(contextual_truthiness) = static_scss_resolved_condition_truthiness(
            condition.as_str(),
            argument_values,
            call_position,
            context,
        ) else {
            continue;
        };
        match truthiness_by_start.get(&branch.control_start).copied() {
            Some(value_truthiness) if value_truthiness != contextual_truthiness => {
                conflicting_control_starts.insert(branch.control_start);
                evidence_counts.contextual_truthiness_conflict_count += 1;
            }
            Some(_) => {}
            None => {
                truthiness_by_start.insert(branch.control_start, contextual_truthiness);
                evidence_counts.contextual_truthiness_fallback_count += 1;
            }
        }
    }
    StaticScssControlFlowValuePrunePlan {
        truthiness_by_start,
        conflicting_control_starts,
        reachable_control_starts: static_scss_control_flow_reachable_starts_after_prune(
            source,
            dialect,
            &initial_bindings,
        ),
        evidence_counts,
    }
}

fn static_scss_control_flow_initial_bindings(
    argument_values: &BTreeMap<String, String>,
) -> BTreeMap<String, AbstractCssValueV0> {
    argument_values
        .iter()
        .map(|(name, value)| (name.clone(), abstract_css_value_from_text(value.as_str())))
        .collect()
}

fn static_scss_control_flow_value_truthiness_by_start(
    source: &str,
    dialect: StyleDialect,
    initial_bindings: &BTreeMap<String, AbstractCssValueV0>,
) -> BTreeMap<usize, bool> {
    let Some(graph) = build_scss_control_flow_graph(source, dialect) else {
        return BTreeMap::new();
    };
    let Some(analysis) =
        analyze_scss_control_flow_values_with_initial_bindings(source, dialect, initial_bindings)
    else {
        return BTreeMap::new();
    };
    let source_start_by_node_key = graph
        .blocks
        .iter()
        .map(|block| {
            (
                block.node_key.as_str().to_string(),
                block.block.source_span_start,
            )
        })
        .collect::<BTreeMap<_, _>>();

    analysis
        .blocks
        .iter()
        .filter_map(|block| {
            let truthy = match block.transfer_truthiness {
                Some("truthy") => true,
                Some("falsey") => false,
                _ => return None,
            };
            source_start_by_node_key
                .get(block.node_key.as_str())
                .copied()
                .map(|source_span_start| (source_span_start, truthy))
        })
        .collect()
}

fn static_scss_control_flow_reachable_starts_after_prune(
    source: &str,
    dialect: StyleDialect,
    initial_bindings: &BTreeMap<String, AbstractCssValueV0>,
) -> Option<BTreeSet<usize>> {
    let graph = build_scss_control_flow_graph(source, dialect)?;
    let report = summarize_scss_control_flow_prune_reachability_with_initial_bindings(
        source,
        dialect,
        initial_bindings,
    )?;
    if !report.converged {
        return None;
    }
    let reachable_block_ids = report
        .reachable_block_ids
        .into_iter()
        .collect::<BTreeSet<_>>();
    Some(
        graph
            .blocks
            .iter()
            .filter(|block| reachable_block_ids.contains(&block.id))
            .map(|block| block.block.source_span_start)
            .collect(),
    )
}

fn static_scss_resolved_condition_truthiness(
    condition: &str,
    argument_values: &BTreeMap<String, String>,
    call_position: usize,
    context: StaticScssFunctionResolutionContext<'_>,
) -> Option<bool> {
    let resolution = resolve_static_scss_function_value_with_bindings(
        condition,
        argument_values,
        call_position,
        STATIC_STYLESHEET_VALUE_RESOLUTION_FUEL_LIMIT,
        context,
    );
    if resolution.outcome == StaticStylesheetResolutionOutcome::Top {
        return None;
    }
    let condition_value = resolution.rendered_value?;
    let _typed_advisory_truthiness =
        static_scss_typed_advisory_truthiness(condition_value.as_str());
    static_scss_literal_truthiness(condition_value.as_str())
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn control_flow_selection_skips_unreachable_pruned_branches() {
        let chain = StaticScssMixinControlFlowChain {
            start: 0,
            end: 30,
            next_index: 3,
            branches: vec![
                StaticScssMixinControlFlowBranch {
                    control_start: 0,
                    condition: Some("false".to_string()),
                    body: ".off { color: red; }".to_string(),
                    body_start: 1,
                },
                StaticScssMixinControlFlowBranch {
                    control_start: 10,
                    condition: Some("true".to_string()),
                    body: ".unreachable { color: blue; }".to_string(),
                    body_start: 11,
                },
                StaticScssMixinControlFlowBranch {
                    control_start: 20,
                    condition: None,
                    body: ".fallback { color: green; }".to_string(),
                    body_start: 21,
                },
            ],
        };
        let plan = StaticScssControlFlowValuePrunePlan {
            truthiness_by_start: BTreeMap::from([(0, false), (10, true)]),
            conflicting_control_starts: BTreeSet::new(),
            reachable_control_starts: Some(BTreeSet::from([0, 20])),
            evidence_counts: StaticScssControlFlowPruneEvidenceCounts::default(),
        };

        let selected = static_scss_mixin_selected_control_flow_body(&chain, Some(&plan));

        assert!(selected.is_some());
        let Some(Some(selected)) = selected else {
            return;
        };
        assert_eq!(selected.body, ".fallback { color: green; }");
        assert_eq!(selected.body_start, 21);
    }

    #[test]
    fn control_flow_selection_preserves_conflicting_prune_truthiness() {
        let chain = StaticScssMixinControlFlowChain {
            start: 0,
            end: 20,
            next_index: 2,
            branches: vec![
                StaticScssMixinControlFlowBranch {
                    control_start: 0,
                    condition: Some("$enabled".to_string()),
                    body: ".on { color: green; }".to_string(),
                    body_start: 1,
                },
                StaticScssMixinControlFlowBranch {
                    control_start: 10,
                    condition: None,
                    body: ".off { color: gray; }".to_string(),
                    body_start: 11,
                },
            ],
        };
        let plan = StaticScssControlFlowValuePrunePlan {
            truthiness_by_start: BTreeMap::from([(0, true)]),
            conflicting_control_starts: BTreeSet::from([0]),
            reachable_control_starts: Some(BTreeSet::from([0, 10])),
            evidence_counts: StaticScssControlFlowPruneEvidenceCounts::default(),
        };

        let selected = static_scss_mixin_selected_control_flow_body(&chain, Some(&plan));

        assert!(selected.is_none());
    }
}
