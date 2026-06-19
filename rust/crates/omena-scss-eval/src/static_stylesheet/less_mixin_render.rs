use std::collections::{BTreeMap, BTreeSet};

use omena_parser::StyleDialect;
use omena_syntax::SyntaxKind;

use super::{
    StaticLessDetachedRulesetDeclaration, StaticStylesheetEvaluationEdit,
    StaticStylesheetPropertyDeclaration, StaticStylesheetScope,
    StaticStylesheetVariableDeclaration, StaticStylesheetVariableKind,
    apply_static_stylesheet_evaluation_edits,
    collect_static_stylesheet_variable_references_with_options,
    less_mixin_values::{
        collect_static_less_mixin_body_local_declarations, static_less_mixin_body_scoped_values,
    },
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
