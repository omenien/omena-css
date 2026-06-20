use std::collections::BTreeSet;

use omena_parser::{ParsedVariableFact, ParsedVariableFactKind, StyleDialect, lex};

use super::{
    declarations::{
        collect_static_scss_variable_declarations,
        static_scss_function_declaration_ranges_from_declarations,
        static_scss_mixin_declaration_ranges_from_declarations,
    },
    edits::apply_static_stylesheet_evaluation_edits,
    model::{
        OmenaScssEvalStaticStylesheetEvaluationV0, StaticScssFunctionResolutionContext,
        StaticStylesheetEvaluationEdit, StaticStylesheetVariableKind,
    },
    reports::{
        build_static_stylesheet_evaluation_report,
        build_static_stylesheet_preserved_evaluation_report_if_explained,
        resolved_replacement_value,
    },
    scopes::{
        collect_static_stylesheet_scopes, static_stylesheet_position_is_inside_scss_declaration,
    },
    scss_calls::collect_static_scss_function_calls,
    scss_declarations::{
        collect_static_scss_function_declarations, collect_static_scss_mixin_declarations,
    },
    scss_function_edits::collect_static_scss_function_evaluation_edits,
    scss_mixin_control_flow::collect_static_scss_control_flow_evaluation_edits,
    scss_mixin_edits::collect_static_scss_mixin_evaluation_edits,
    scss_variables::resolve_static_scss_variable_value_at_position,
    tokens::{parser_text_size_to_usize, static_stylesheet_position_is_inside_ranges},
    variable_references::static_stylesheet_position_is_scss_module_member_reference,
};

pub(super) fn derive_static_scss_stylesheet_module_evaluation(
    style_source: &str,
    dialect: StyleDialect,
    variable_facts: &[ParsedVariableFact],
) -> Option<OmenaScssEvalStaticStylesheetEvaluationV0> {
    let lexed = lex(style_source, dialect);
    let tokens = lexed.tokens();
    let function_declarations =
        collect_static_scss_function_declarations(style_source, dialect, tokens)?;
    let mixin_declarations = collect_static_scss_mixin_declarations(style_source, dialect, tokens)?;
    if !variable_facts
        .iter()
        .any(|fact| fact.kind == ParsedVariableFactKind::ScssDeclaration)
        && function_declarations.is_empty()
        && mixin_declarations.is_empty()
    {
        return None;
    }
    let scopes = collect_static_stylesheet_scopes(style_source)?;
    let function_declaration_ranges =
        static_scss_function_declaration_ranges_from_declarations(function_declarations.as_slice());
    let mixin_declaration_ranges =
        static_scss_mixin_declaration_ranges_from_declarations(mixin_declarations.as_slice());
    let function_call_ranges =
        collect_static_scss_function_calls(style_source, tokens, function_declarations.as_slice())
            .map(|calls| {
                calls
                    .into_iter()
                    .map(|call| (call.start, call.end))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
    let declarations =
        collect_static_scss_variable_declarations(style_source, dialect, variable_facts, &scopes)?
            .into_iter()
            .filter(|declaration| {
                !static_stylesheet_position_is_inside_ranges(
                    declaration.declaration.span_start,
                    &function_declaration_ranges,
                ) && !static_stylesheet_position_is_inside_ranges(
                    declaration.declaration.span_start,
                    &mixin_declaration_ranges,
                ) && !static_stylesheet_position_is_inside_ranges(
                    declaration.declaration.span_start,
                    &function_call_ranges,
                )
            })
            .collect::<Vec<_>>();

    let mut edits = Vec::new();
    let mut resolved_replacements = Vec::new();
    for declaration in &declarations {
        for (start, end) in &declaration.removal_spans {
            edits.push(StaticStylesheetEvaluationEdit {
                start: *start,
                end: *end,
                replacement: String::new(),
            });
        }
    }
    let mut control_flow_excluded_ranges = function_declaration_ranges.clone();
    control_flow_excluded_ranges.extend(mixin_declaration_ranges.iter().copied());
    let active_functions = BTreeSet::new();
    let control_flow_context = StaticScssFunctionResolutionContext {
        dialect,
        declarations: &function_declarations,
        mixin_declarations: &mixin_declarations,
        scopes: &scopes,
        variable_declarations: &declarations,
        active_functions: &active_functions,
    };
    let control_flow_edits = collect_static_scss_control_flow_evaluation_edits(
        style_source,
        dialect,
        tokens,
        &control_flow_excluded_ranges,
        control_flow_context,
    )?;
    if control_flow_edits.preserved_dynamic_branch_count > 0 {
        return build_static_stylesheet_preserved_evaluation_report_if_explained(
            style_source,
            dialect,
            StaticStylesheetVariableKind::Scss,
        );
    }
    let control_flow_ranges = control_flow_edits
        .edits
        .iter()
        .map(|edit| (edit.start, edit.end))
        .collect::<Vec<_>>();
    for fact in variable_facts {
        if fact.kind != ParsedVariableFactKind::ScssReference {
            continue;
        }
        let reference_start = parser_text_size_to_usize(fact.range.start().into());
        if static_stylesheet_position_is_scss_module_member_reference(style_source, reference_start)
        {
            continue;
        }
        if static_stylesheet_position_is_inside_scss_declaration(&declarations, reference_start)
            || static_stylesheet_position_is_inside_ranges(
                reference_start,
                &function_declaration_ranges,
            )
            || static_stylesheet_position_is_inside_ranges(
                reference_start,
                &mixin_declaration_ranges,
            )
            || static_stylesheet_position_is_inside_ranges(reference_start, &function_call_ranges)
            || static_stylesheet_position_is_inside_ranges(reference_start, &control_flow_ranges)
        {
            continue;
        }
        let mut stack = BTreeSet::new();
        let Some(replacement) = resolve_static_scss_variable_value_at_position(
            fact.name.as_str(),
            reference_start,
            &scopes,
            &declarations,
            &mut stack,
        ) else {
            return build_static_stylesheet_preserved_evaluation_report_if_explained(
                style_source,
                dialect,
                StaticStylesheetVariableKind::Scss,
            );
        };
        let reference_end = parser_text_size_to_usize(fact.range.end().into());
        resolved_replacements.push(resolved_replacement_value(
            fact.name.as_str(),
            reference_start,
            reference_end,
            replacement.as_str(),
        ));
        edits.push(StaticStylesheetEvaluationEdit {
            start: reference_start,
            end: reference_end,
            replacement,
        });
    }
    let mut preserved_scss_evaluation_count = 0usize;
    if let Some(function_edits) = collect_static_scss_function_evaluation_edits(
        style_source,
        tokens,
        control_flow_context,
        &control_flow_ranges,
    ) {
        if function_edits.preserved_raw_call_count > 0 {
            return build_static_stylesheet_preserved_evaluation_report_if_explained(
                style_source,
                dialect,
                StaticStylesheetVariableKind::Scss,
            );
        }
        edits.extend(function_edits.edits);
        resolved_replacements.extend(function_edits.replacements);
    }
    if let Some(mixin_edits) = collect_static_scss_mixin_evaluation_edits(
        style_source,
        tokens,
        control_flow_context,
        &control_flow_ranges,
    ) {
        preserved_scss_evaluation_count += mixin_edits.preserved_raw_include_count;
        edits.extend(mixin_edits.edits);
    }
    resolved_replacements.extend(control_flow_edits.replacements);
    edits.extend(control_flow_edits.edits);

    let evaluated_css = apply_static_stylesheet_evaluation_edits(style_source, edits.clone())?;
    if evaluated_css == style_source && preserved_scss_evaluation_count == 0 {
        return None;
    }
    build_static_stylesheet_evaluation_report(
        style_source,
        dialect,
        StaticStylesheetVariableKind::Scss,
        evaluated_css,
        edits,
        resolved_replacements,
    )
}
