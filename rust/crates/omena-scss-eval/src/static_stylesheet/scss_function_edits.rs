use std::collections::BTreeSet;

use omena_parser::{LexedToken, StyleDialect};

use super::{
    OmenaScssEvalStaticValueResolutionV0, STATIC_STYLESHEET_VALUE_RESOLUTION_FUEL_LIMIT,
    StaticScssFunctionDeclaration, StaticScssFunctionEvaluationEdits,
    StaticScssFunctionResolutionContext, StaticScssMixinDeclaration,
    StaticStylesheetEvaluationEdit, StaticStylesheetResolutionOutcome,
    StaticStylesheetResolutionReason, StaticStylesheetScope,
    StaticStylesheetScopedVariableDeclaration, canonical_static_scss_function_name,
    collect_static_scss_function_calls, extend_static_scss_used_function_dependencies,
    resolve_static_scss_function_call_abstract_value, resolved_replacement_value,
    static_scss_function_call_is_inside_declaration_body,
    static_scss_function_call_is_inside_mixin_declaration_body,
    static_stylesheet_position_is_inside_ranges,
    value_resolution_model::static_value_resolution_record,
};

pub(super) fn collect_static_scss_function_evaluation_edits(
    source: &str,
    tokens: &[LexedToken],
    context: StaticScssFunctionResolutionContext<'_>,
    excluded_ranges: &[(usize, usize)],
) -> Option<StaticScssFunctionEvaluationEdits> {
    let calls = collect_static_scss_function_calls(source, tokens, context.declarations)?;
    if calls.is_empty() {
        return Some(StaticScssFunctionEvaluationEdits {
            edits: Vec::new(),
            replacements: Vec::new(),
            preserved_raw_call_count: 0,
        });
    }

    let mut edits = Vec::new();
    let mut replacements = Vec::new();
    let mut used_declaration_names = BTreeSet::new();
    for call in calls.iter().filter(|call| {
        !static_scss_function_call_is_inside_declaration_body(call, context.declarations)
            && !static_scss_function_call_is_inside_mixin_declaration_body(
                call,
                context.mixin_declarations,
            )
            && !static_stylesheet_position_is_inside_ranges(call.start, excluded_ranges)
    }) {
        let resolution = resolve_static_scss_function_call_abstract_value(
            call,
            context.dialect,
            context.declarations,
            context.mixin_declarations,
            context.scopes,
            context.variable_declarations,
            STATIC_STYLESHEET_VALUE_RESOLUTION_FUEL_LIMIT,
        );
        if resolution.outcome == StaticStylesheetResolutionOutcome::Top
            && resolution.reason != StaticStylesheetResolutionReason::UnresolvedReference
        {
            return Some(StaticScssFunctionEvaluationEdits {
                edits: Vec::new(),
                replacements: Vec::new(),
                preserved_raw_call_count: 1,
            });
        }
        if resolution.outcome != StaticStylesheetResolutionOutcome::Resolved {
            return None;
        }
        let rendered_value = resolution.rendered_value?;
        used_declaration_names.insert(canonical_static_scss_function_name(call.name.as_str()));
        replacements.push(resolved_replacement_value(
            format!("function:{}", call.name).as_str(),
            call.start,
            call.end,
            rendered_value.as_str(),
        ));
        edits.push(StaticStylesheetEvaluationEdit {
            start: call.start,
            end: call.end,
            replacement: rendered_value,
        });
    }
    extend_static_scss_used_function_dependencies(
        &mut used_declaration_names,
        context.declarations,
    );

    for declaration in context.declarations.iter().filter(|declaration| {
        used_declaration_names.contains(&canonical_static_scss_function_name(
            declaration.name.as_str(),
        ))
    }) {
        edits.push(StaticStylesheetEvaluationEdit {
            start: declaration.span_start,
            end: declaration.span_end,
            replacement: String::new(),
        });
    }

    Some(StaticScssFunctionEvaluationEdits {
        edits,
        replacements,
        preserved_raw_call_count: 0,
    })
}

pub(super) fn collect_static_scss_function_value_resolution_values(
    source: &str,
    dialect: StyleDialect,
    tokens: &[LexedToken],
    declarations: &[StaticScssFunctionDeclaration],
    mixin_declarations: &[StaticScssMixinDeclaration],
    scopes: &[StaticStylesheetScope],
    variable_declarations: &[StaticStylesheetScopedVariableDeclaration],
) -> Option<Vec<OmenaScssEvalStaticValueResolutionV0>> {
    let calls = collect_static_scss_function_calls(source, tokens, declarations)?;
    let values = calls
        .into_iter()
        .filter(|call| {
            !static_scss_function_call_is_inside_declaration_body(call, declarations)
                && !static_scss_function_call_is_inside_mixin_declaration_body(
                    call,
                    mixin_declarations,
                )
        })
        .map(|call| {
            let resolution = resolve_static_scss_function_call_abstract_value(
                &call,
                dialect,
                declarations,
                mixin_declarations,
                scopes,
                variable_declarations,
                STATIC_STYLESHEET_VALUE_RESOLUTION_FUEL_LIMIT,
            );
            static_value_resolution_record(
                format!("function:{}", call.name).as_str(),
                call.start,
                call.end,
                source.get(call.start..call.end).unwrap_or(""),
                resolution,
            )
        })
        .collect();
    Some(values)
}
