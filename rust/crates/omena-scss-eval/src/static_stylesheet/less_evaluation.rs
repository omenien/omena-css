use std::collections::BTreeSet;

use omena_parser::{ParsedVariableFact, ParsedVariableFactKind, StyleDialect, lex};
use omena_syntax::SyntaxKind;

use super::{
    declarations::{
        collect_static_less_property_declarations, collect_static_less_variable_declarations,
        static_less_mixin_declaration_ranges_from_declarations,
    },
    edits::apply_static_stylesheet_evaluation_edits,
    less_detached_ruleset_edits::{
        collect_static_less_detached_ruleset_accessor_evaluation_edits,
        collect_static_less_detached_ruleset_evaluation_edits,
    },
    less_detached_rulesets::{
        collect_static_less_detached_ruleset_accessors, collect_static_less_detached_ruleset_calls,
        collect_static_less_detached_ruleset_declarations,
        static_less_detached_ruleset_ranges_from_accessors,
        static_less_detached_ruleset_ranges_from_calls,
        static_less_detached_ruleset_ranges_from_declarations,
    },
    less_literal_edits::collect_static_less_literal_value_edits,
    less_mixin_edits::{
        collect_static_less_mixin_accessor_evaluation_edits,
        collect_static_less_mixin_evaluation_edits,
    },
    less_mixin_values::static_less_value_is_detached_ruleset_reference,
    less_mixins::{
        collect_static_less_mixin_accessors, collect_static_less_mixin_calls,
        collect_static_less_mixin_declarations, static_less_mixin_accessor_ranges_from_accessors,
        static_less_mixin_ranges_from_calls,
    },
    less_variables::{
        resolve_static_less_property_value_in_scope, resolve_static_less_variable_value_in_scope,
    },
    model::{
        OmenaScssEvalStaticStylesheetEvaluationV0, StaticStylesheetEvaluationEdit,
        StaticStylesheetVariableKind,
    },
    reports::{build_static_stylesheet_evaluation_report, resolved_replacement_value},
    scopes::{
        collect_static_stylesheet_scopes, static_stylesheet_position_is_inside_scoped_declaration,
        static_stylesheet_scope_for_position,
    },
    tokens::{
        parser_text_size_to_usize, static_stylesheet_position_is_inside_ranges,
        static_stylesheet_token_end, static_stylesheet_token_start,
    },
    variable_references::static_stylesheet_variable_reference_is_named_argument_label,
};

pub(super) fn derive_static_less_stylesheet_module_evaluation(
    style_source: &str,
    variable_facts: &[ParsedVariableFact],
) -> Option<OmenaScssEvalStaticStylesheetEvaluationV0> {
    let scopes = collect_static_stylesheet_scopes(style_source)?;
    let lexed = lex(style_source, StyleDialect::Less);
    let tokens = lexed.tokens();
    let mixin_declarations = collect_static_less_mixin_declarations(style_source, tokens)?;
    let mixin_declaration_ranges =
        static_less_mixin_declaration_ranges_from_declarations(mixin_declarations.as_slice());
    let detached_rulesets =
        collect_static_less_detached_ruleset_declarations(style_source, tokens, &scopes)?;
    let detached_ruleset_ranges =
        static_less_detached_ruleset_ranges_from_declarations(detached_rulesets.as_slice());
    let detached_ruleset_calls = collect_static_less_detached_ruleset_calls(style_source, tokens)?;
    let detached_ruleset_call_ranges =
        static_less_detached_ruleset_ranges_from_calls(detached_ruleset_calls.as_slice());
    let detached_ruleset_accessors =
        collect_static_less_detached_ruleset_accessors(style_source, tokens)?;
    let detached_ruleset_accessor_ranges =
        static_less_detached_ruleset_ranges_from_accessors(detached_ruleset_accessors.as_slice());
    let mixin_calls = collect_static_less_mixin_calls(style_source, tokens).unwrap_or_default();
    let mixin_call_ranges = static_less_mixin_ranges_from_calls(mixin_calls.as_slice());
    let mixin_accessors = collect_static_less_mixin_accessors(style_source, tokens)?;
    let mixin_accessor_ranges = static_less_mixin_accessor_ranges_from_accessors(&mixin_accessors);
    let mut variable_excluded_ranges = mixin_declaration_ranges.clone();
    variable_excluded_ranges.extend(detached_ruleset_ranges.iter().copied());
    variable_excluded_ranges.extend(detached_ruleset_accessor_ranges.iter().copied());
    variable_excluded_ranges.extend(mixin_accessor_ranges.iter().copied());
    let declarations = collect_static_less_variable_declarations(
        style_source,
        variable_facts,
        &scopes,
        &variable_excluded_ranges,
    )?;
    let property_declarations =
        collect_static_less_property_declarations(style_source, tokens, &scopes)?;

    let mut edits = Vec::new();
    let mut preserved_less_evaluation_count = 0usize;
    let mut resolved_replacements = Vec::new();
    for declaration in declarations.values() {
        for (start, end) in &declaration.removal_spans {
            edits.push(StaticStylesheetEvaluationEdit {
                start: *start,
                end: *end,
                replacement: String::new(),
            });
        }
    }
    for fact in variable_facts {
        if fact.kind != ParsedVariableFactKind::LessReference {
            continue;
        }
        let reference_start = parser_text_size_to_usize(fact.range.start().into());
        let reference_end = parser_text_size_to_usize(fact.range.end().into());
        if static_stylesheet_variable_reference_is_named_argument_label(
            style_source,
            reference_start,
            reference_end,
        ) {
            continue;
        }
        if static_stylesheet_position_is_inside_scoped_declaration(&declarations, reference_start) {
            continue;
        }
        if static_stylesheet_position_is_inside_ranges(reference_start, &mixin_declaration_ranges) {
            continue;
        }
        if static_stylesheet_position_is_inside_ranges(reference_start, &detached_ruleset_ranges) {
            continue;
        }
        if static_stylesheet_position_is_inside_ranges(
            reference_start,
            &detached_ruleset_call_ranges,
        ) {
            continue;
        }
        if static_stylesheet_position_is_inside_ranges(
            reference_start,
            &detached_ruleset_accessor_ranges,
        ) {
            continue;
        }
        if static_stylesheet_position_is_inside_ranges(reference_start, &mixin_accessor_ranges) {
            continue;
        }
        let reference_scope_id = static_stylesheet_scope_for_position(&scopes, reference_start)?;
        if static_stylesheet_position_is_inside_ranges(reference_start, &mixin_call_ranges)
            && static_less_value_is_detached_ruleset_reference(
                fact.name.as_str(),
                reference_scope_id,
                &scopes,
                detached_rulesets.as_slice(),
            )
        {
            continue;
        }
        let mut stack = BTreeSet::new();
        let replacement = resolve_static_less_variable_value_in_scope(
            fact.name.as_str(),
            reference_scope_id,
            &scopes,
            &declarations,
            &property_declarations,
            detached_rulesets.as_slice(),
            &mut stack,
        )?;
        let replacement = replacement.text;
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
    for token in tokens {
        if token.kind != SyntaxKind::LessPropertyVariableToken {
            continue;
        }
        let reference_start = static_stylesheet_token_start(token);
        if static_stylesheet_position_is_inside_scoped_declaration(&declarations, reference_start) {
            continue;
        }
        if static_stylesheet_position_is_inside_ranges(reference_start, &mixin_declaration_ranges) {
            continue;
        }
        if static_stylesheet_position_is_inside_ranges(reference_start, &detached_ruleset_ranges) {
            continue;
        }
        if static_stylesheet_position_is_inside_ranges(
            reference_start,
            &detached_ruleset_accessor_ranges,
        ) {
            continue;
        }
        if static_stylesheet_position_is_inside_ranges(reference_start, &mixin_accessor_ranges) {
            continue;
        }
        let reference_scope_id = static_stylesheet_scope_for_position(&scopes, reference_start)?;
        let mut stack = BTreeSet::new();
        let replacement = resolve_static_less_property_value_in_scope(
            token.text.as_str(),
            reference_scope_id,
            &scopes,
            &property_declarations,
            &mut stack,
        )?;
        let replacement = replacement.text;
        resolved_replacements.push(resolved_replacement_value(
            token.text.as_str(),
            reference_start,
            static_stylesheet_token_end(token),
            replacement.as_str(),
        ));
        edits.push(StaticStylesheetEvaluationEdit {
            start: reference_start,
            end: static_stylesheet_token_end(token),
            replacement,
        });
    }
    edits.extend(collect_static_less_literal_value_edits(
        style_source,
        tokens,
        &declarations,
        &variable_excluded_ranges,
    )?);
    let detached_ruleset_accessor_evaluation_edits =
        collect_static_less_detached_ruleset_accessor_evaluation_edits(
            style_source,
            &detached_rulesets,
            &detached_ruleset_accessors,
            &mixin_declaration_ranges,
            &scopes,
            &declarations,
            &property_declarations,
        )?;
    preserved_less_evaluation_count +=
        detached_ruleset_accessor_evaluation_edits.preserved_raw_accessor_count;
    let detached_ruleset_evaluation_edits = collect_static_less_detached_ruleset_evaluation_edits(
        style_source,
        &detached_rulesets,
        &detached_ruleset_calls,
        &mixin_declarations,
        &mixin_declaration_ranges,
        &detached_ruleset_accessor_evaluation_edits.preserved_declaration_keys,
        &scopes,
        &declarations,
        &property_declarations,
    )?;
    preserved_less_evaluation_count += detached_ruleset_evaluation_edits.preserved_raw_call_count;
    edits.extend(detached_ruleset_evaluation_edits.edits);
    edits.extend(detached_ruleset_accessor_evaluation_edits.edits);
    let accessor_evaluation_edits = collect_static_less_mixin_accessor_evaluation_edits(
        style_source,
        tokens,
        &mixin_declarations,
        &mixin_declaration_ranges,
        &detached_rulesets,
        &scopes,
        &declarations,
        &property_declarations,
        &detached_ruleset_ranges,
    )?;
    preserved_less_evaluation_count += accessor_evaluation_edits.preserved_raw_accessor_count;
    edits.extend(accessor_evaluation_edits.edits);
    if let Some(mixin_evaluation_edits) = collect_static_less_mixin_evaluation_edits(
        style_source,
        tokens,
        &mixin_declarations,
        &mixin_declaration_ranges,
        &detached_rulesets,
        &scopes,
        &declarations,
        &property_declarations,
        &detached_ruleset_ranges,
    ) {
        preserved_less_evaluation_count +=
            mixin_evaluation_edits.preserved_non_rendering_call_count;
        edits.extend(mixin_evaluation_edits.edits);
    }

    let evaluated_css = apply_static_stylesheet_evaluation_edits(style_source, edits.clone())?;
    if evaluated_css == style_source && preserved_less_evaluation_count == 0 {
        return None;
    }
    build_static_stylesheet_evaluation_report(
        style_source,
        StyleDialect::Less,
        StaticStylesheetVariableKind::Less,
        evaluated_css,
        edits,
        resolved_replacements,
    )
}
