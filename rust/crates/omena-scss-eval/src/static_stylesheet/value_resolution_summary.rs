use std::collections::BTreeSet;

use omena_parser::{ParsedVariableFact, ParsedVariableFactKind, StyleDialect, lex};
use omena_syntax::SyntaxKind;

use super::{
    STATIC_STYLESHEET_VALUE_RESOLUTION_FUEL_LIMIT,
    declarations::{
        collect_static_less_property_declarations, collect_static_less_variable_declarations,
        collect_static_scss_variable_declarations,
        static_less_mixin_declaration_ranges_from_declarations,
        static_scss_function_declaration_ranges_from_declarations,
        static_scss_mixin_declaration_ranges_from_declarations,
    },
    less_detached_rulesets::{
        collect_static_less_detached_ruleset_accessors, collect_static_less_detached_ruleset_calls,
        collect_static_less_detached_ruleset_declarations,
        static_less_detached_ruleset_ranges_from_accessors,
        static_less_detached_ruleset_ranges_from_calls,
        static_less_detached_ruleset_ranges_from_declarations,
    },
    less_mixin_values::static_less_value_is_detached_ruleset_reference,
    less_mixins::{
        collect_static_less_mixin_accessors, collect_static_less_mixin_calls,
        collect_static_less_mixin_declarations, static_less_mixin_accessor_ranges_from_accessors,
        static_less_mixin_ranges_from_calls,
    },
    less_variables::{
        resolve_static_less_property_abstract_value_in_scope,
        resolve_static_less_variable_abstract_value_in_scope,
    },
    model::{OmenaScssEvalStaticValueResolutionV0, StaticStylesheetScope},
    scopes::{
        static_stylesheet_position_is_inside_scoped_declaration,
        static_stylesheet_position_is_inside_scss_declaration,
        static_stylesheet_scope_for_position,
    },
    scss_declarations::{
        collect_static_scss_function_declarations, collect_static_scss_mixin_declarations,
    },
    scss_function_edits::collect_static_scss_function_value_resolution_values,
    scss_variables::resolve_static_scss_variable_abstract_value_at_position,
    tokens::{
        parser_text_size_to_usize, static_stylesheet_position_is_inside_ranges,
        static_stylesheet_token_end, static_stylesheet_token_start,
    },
    value_resolution_model::static_value_resolution_record,
    variable_references::{
        static_stylesheet_position_is_scss_module_member_reference,
        static_stylesheet_variable_reference_is_named_argument_label,
    },
};

pub(super) fn summarize_static_scss_value_resolution_values(
    style_source: &str,
    dialect: StyleDialect,
    variable_facts: &[ParsedVariableFact],
    scopes: &[StaticStylesheetScope],
) -> Option<Vec<OmenaScssEvalStaticValueResolutionV0>> {
    let declarations =
        collect_static_scss_variable_declarations(style_source, dialect, variable_facts, scopes)?;
    let lexed = lex(style_source, dialect);
    let tokens = lexed.tokens();
    let function_declarations =
        collect_static_scss_function_declarations(style_source, dialect, tokens)?;
    let mixin_declarations = collect_static_scss_mixin_declarations(style_source, dialect, tokens)?;
    let function_declaration_ranges =
        static_scss_function_declaration_ranges_from_declarations(function_declarations.as_slice());
    let mixin_declaration_ranges =
        static_scss_mixin_declaration_ranges_from_declarations(mixin_declarations.as_slice());
    let mut values = Vec::new();
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
        {
            continue;
        }
        let reference_end = parser_text_size_to_usize(fact.range.end().into());
        let mut stack = BTreeSet::new();
        let resolution = resolve_static_scss_variable_abstract_value_at_position(
            fact.name.as_str(),
            reference_start,
            scopes,
            &declarations,
            &mut stack,
            STATIC_STYLESHEET_VALUE_RESOLUTION_FUEL_LIMIT,
        );
        values.push(static_value_resolution_record(
            fact.name.as_str(),
            reference_start,
            reference_end,
            style_source
                .get(reference_start..reference_end)
                .unwrap_or(""),
            resolution,
        ));
    }
    values.extend(collect_static_scss_function_value_resolution_values(
        style_source,
        dialect,
        tokens,
        &function_declarations,
        &mixin_declarations,
        scopes,
        &declarations,
    )?);
    Some(values)
}

pub(super) fn summarize_static_less_value_resolution_values(
    style_source: &str,
    variable_facts: &[ParsedVariableFact],
    scopes: &[StaticStylesheetScope],
) -> Option<Vec<OmenaScssEvalStaticValueResolutionV0>> {
    let lexed = lex(style_source, StyleDialect::Less);
    let tokens = lexed.tokens();
    let mixin_declarations = collect_static_less_mixin_declarations(style_source, tokens)?;
    let mixin_declaration_ranges =
        static_less_mixin_declaration_ranges_from_declarations(mixin_declarations.as_slice());
    let detached_rulesets =
        collect_static_less_detached_ruleset_declarations(style_source, tokens, scopes)?;
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
        scopes,
        &variable_excluded_ranges,
    )?;
    let property_declarations =
        collect_static_less_property_declarations(style_source, tokens, scopes)?;
    let mut values = Vec::new();
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
        let reference_scope_id = static_stylesheet_scope_for_position(scopes, reference_start)?;
        if static_stylesheet_position_is_inside_ranges(reference_start, &mixin_call_ranges)
            && static_less_value_is_detached_ruleset_reference(
                fact.name.as_str(),
                reference_scope_id,
                scopes,
                detached_rulesets.as_slice(),
            )
        {
            continue;
        }
        let mut stack = BTreeSet::new();
        let resolution = resolve_static_less_variable_abstract_value_in_scope(
            fact.name.as_str(),
            reference_scope_id,
            scopes,
            &declarations,
            &property_declarations,
            detached_rulesets.as_slice(),
            &mut stack,
            STATIC_STYLESHEET_VALUE_RESOLUTION_FUEL_LIMIT,
        );
        values.push(static_value_resolution_record(
            fact.name.as_str(),
            reference_start,
            reference_end,
            style_source
                .get(reference_start..reference_end)
                .unwrap_or(""),
            resolution,
        ));
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
        let reference_scope_id = static_stylesheet_scope_for_position(scopes, reference_start)?;
        let mut stack = BTreeSet::new();
        let resolution = resolve_static_less_property_abstract_value_in_scope(
            token.text.as_str(),
            reference_scope_id,
            scopes,
            &property_declarations,
            &mut stack,
            STATIC_STYLESHEET_VALUE_RESOLUTION_FUEL_LIMIT,
        );
        values.push(static_value_resolution_record(
            token.text.as_str(),
            reference_start,
            static_stylesheet_token_end(token),
            token.text.as_str(),
            resolution,
        ));
    }
    Some(values)
}
