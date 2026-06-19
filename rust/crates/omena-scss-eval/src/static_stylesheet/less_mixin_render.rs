use std::collections::{BTreeMap, BTreeSet};

use omena_parser::{StyleDialect, lex};
use omena_syntax::SyntaxKind;

use super::{
    StaticLessDetachedRulesetCallRenderOutcome, StaticLessDetachedRulesetDeclaration,
    StaticLessMixinCallRenderOutcome, StaticLessMixinRenderContext, StaticLessMixinRenderResult,
    StaticStylesheetEvaluationEdit, StaticStylesheetPropertyDeclaration, StaticStylesheetScope,
    StaticStylesheetVariableDeclaration, StaticStylesheetVariableKind,
    apply_static_stylesheet_evaluation_edits,
    collect_static_stylesheet_variable_references_with_options,
    less_detached_rulesets::{
        collect_static_less_detached_ruleset_calls, find_static_less_detached_ruleset_declaration,
    },
    less_mixin_values::{
        collect_static_less_mixin_body_local_declarations, static_less_mixin_body_scoped_values,
    },
    less_mixins::collect_static_less_mixin_calls,
    render_static_less_detached_ruleset_body, render_static_less_mixin_call,
    resolve_static_less_property_value_in_scope, resolve_static_less_variable_value_in_scope,
    static_stylesheet_position_is_inside_ranges, static_stylesheet_token_end,
    static_stylesheet_token_start,
};

#[allow(clippy::too_many_arguments)]
pub(super) fn render_static_less_mixin_body_variables(
    body: &str,
    call_scope_id: usize,
    argument_values: &BTreeMap<String, String>,
    captured_values: &BTreeMap<String, String>,
    scopes: &[StaticStylesheetScope],
    variable_declarations: &BTreeMap<(usize, String), StaticStylesheetVariableDeclaration>,
    property_declarations: &BTreeMap<(usize, String), StaticStylesheetPropertyDeclaration>,
    detached_ruleset_declarations: &[StaticLessDetachedRulesetDeclaration],
) -> Option<String> {
    let local_declarations = collect_static_less_mixin_body_local_declarations(body)?;
    let local_declaration_ranges = local_declarations
        .iter()
        .flat_map(|declaration| declaration.declaration.removal_spans.iter().copied())
        .collect::<Vec<_>>();
    let scoped_values = static_less_mixin_body_scoped_values(
        body,
        call_scope_id,
        argument_values,
        captured_values,
        scopes,
        variable_declarations,
        property_declarations,
        detached_ruleset_declarations,
    )?;
    let mut edits = local_declarations
        .iter()
        .flat_map(|declaration| {
            declaration
                .declaration
                .removal_spans
                .iter()
                .map(|(start, end)| StaticStylesheetEvaluationEdit {
                    start: *start,
                    end: *end,
                    replacement: String::new(),
                })
        })
        .collect::<Vec<_>>();

    let references = collect_static_stylesheet_variable_references_with_options(
        body,
        StaticStylesheetVariableKind::Less,
        false,
        true,
    )?;
    for reference in references {
        if static_stylesheet_position_is_inside_ranges(reference.start, &local_declaration_ranges) {
            continue;
        }
        let replacement = if let Some(value) = scoped_values.get(reference.name.as_str()) {
            value.clone()
        } else if let Some(value) = captured_values.get(reference.name.as_str()) {
            value.clone()
        } else {
            let mut stack = BTreeSet::new();
            resolve_static_less_variable_value_in_scope(
                reference.name.as_str(),
                call_scope_id,
                scopes,
                variable_declarations,
                property_declarations,
                detached_ruleset_declarations,
                &mut stack,
            )?
            .text
        };
        edits.push(StaticStylesheetEvaluationEdit {
            start: reference.start,
            end: reference.end,
            replacement,
        });
    }
    let body_lexed = omena_parser::lex(body, StyleDialect::Less);
    for token in body_lexed.tokens() {
        if token.kind != SyntaxKind::LessPropertyVariableToken {
            continue;
        }
        let reference_start = static_stylesheet_token_start(token);
        let mut stack = BTreeSet::new();
        let replacement = resolve_static_less_property_value_in_scope(
            token.text.as_str(),
            call_scope_id,
            scopes,
            property_declarations,
            &mut stack,
        )?
        .text;
        edits.push(StaticStylesheetEvaluationEdit {
            start: reference_start,
            end: static_stylesheet_token_end(token),
            replacement,
        });
    }
    apply_static_stylesheet_evaluation_edits(body, edits)
}

pub(super) fn render_static_less_mixin_body_nested_calls(
    body: &str,
    call_scope_id: usize,
    context: StaticLessMixinRenderContext<'_>,
    active_mixins: &mut BTreeSet<String>,
) -> Option<StaticLessMixinRenderResult> {
    let body_lexed = lex(body, StyleDialect::Less);
    let body_tokens = body_lexed.tokens();
    let calls = collect_static_less_mixin_calls(body, body_tokens)?;
    let detached_calls = collect_static_less_detached_ruleset_calls(body, body_tokens)?;
    if calls.is_empty() && detached_calls.is_empty() {
        return Some(StaticLessMixinRenderResult {
            body: body.to_string(),
            used_declaration_names: BTreeSet::new(),
        });
    }

    let mut edits = Vec::new();
    let mut used_declaration_names = BTreeSet::new();
    for call in calls {
        let Some(rendered) =
            render_static_less_mixin_call(&call, call_scope_id, context, active_mixins)?
        else {
            continue;
        };
        match rendered {
            StaticLessMixinCallRenderOutcome::Rendered(rendered) => {
                used_declaration_names.extend(rendered.used_declaration_names);
                edits.push(StaticStylesheetEvaluationEdit {
                    start: call.start,
                    end: call.end,
                    replacement: rendered.body,
                });
            }
            StaticLessMixinCallRenderOutcome::PreservedNoOutput => {}
        }
    }
    for call in detached_calls {
        let declaration = find_static_less_detached_ruleset_declaration(
            call.name.as_str(),
            call_scope_id,
            context.scopes,
            context.detached_ruleset_declarations,
        )?;
        let rendered = render_static_less_detached_ruleset_body(
            context.source,
            declaration,
            call_scope_id,
            context.scopes,
            context.variable_declarations,
            context.property_declarations,
            context.declarations,
            context.detached_ruleset_declarations,
        )?;
        match rendered {
            StaticLessDetachedRulesetCallRenderOutcome::Rendered(rendered) => {
                used_declaration_names.extend(rendered.used_declaration_names);
                edits.push(StaticStylesheetEvaluationEdit {
                    start: call.start,
                    end: call.end,
                    replacement: rendered.body,
                });
            }
            StaticLessDetachedRulesetCallRenderOutcome::PreservedRaw => {}
        }
    }

    Some(StaticLessMixinRenderResult {
        body: apply_static_stylesheet_evaluation_edits(body, edits)?,
        used_declaration_names,
    })
}
