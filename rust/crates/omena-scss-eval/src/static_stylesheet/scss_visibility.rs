use std::collections::BTreeMap;

use crate::scss_metadata::reduce_static_scss_metadata_with_context;

use super::{
    model::StaticScssFunctionResolutionContext,
    names::{canonical_static_scss_function_name, canonical_static_scss_variable_name},
    scopes::static_stylesheet_scope_for_position,
    scss_variables::{
        find_static_scss_variable_declaration, find_static_scss_variable_declaration_in_scope,
    },
};

pub(super) fn reduce_static_scss_metadata_with_function_context(
    value: &str,
    argument_values: &BTreeMap<String, String>,
    position: usize,
    context: StaticScssFunctionResolutionContext<'_>,
) -> Option<String> {
    reduce_static_scss_metadata_with_context(
        value,
        |name| {
            static_scss_visible_function_declaration_exists(name, position, context).then_some(true)
        },
        |name| {
            static_scss_visible_mixin_declaration_exists(name, position, context).then_some(true)
        },
        |name| {
            Some(static_scss_visible_variable_exists(
                name,
                position,
                argument_values,
                context,
            ))
        },
        |name| {
            Some(static_scss_visible_global_variable_exists(
                name, position, context,
            ))
        },
    )
}

fn static_scss_visible_function_declaration_exists(
    name: &str,
    position: usize,
    context: StaticScssFunctionResolutionContext<'_>,
) -> bool {
    context.declarations.iter().any(|declaration| {
        declaration.span_start <= position
            && canonical_static_scss_function_name(declaration.name.as_str()) == name
    })
}

fn static_scss_visible_mixin_declaration_exists(
    name: &str,
    position: usize,
    context: StaticScssFunctionResolutionContext<'_>,
) -> bool {
    context.mixin_declarations.iter().any(|declaration| {
        declaration.span_start <= position
            && canonical_static_scss_function_name(declaration.name.as_str()) == name
    })
}

fn static_scss_visible_variable_exists(
    name: &str,
    position: usize,
    argument_values: &BTreeMap<String, String>,
    context: StaticScssFunctionResolutionContext<'_>,
) -> bool {
    argument_values.contains_key(canonical_static_scss_variable_name(name).as_str())
        || static_stylesheet_scope_for_position(context.scopes, position)
            .and_then(|scope_id| {
                find_static_scss_variable_declaration(
                    name,
                    scope_id,
                    position,
                    context.scopes,
                    context.variable_declarations,
                )
            })
            .is_some()
}

fn static_scss_visible_global_variable_exists(
    name: &str,
    position: usize,
    context: StaticScssFunctionResolutionContext<'_>,
) -> bool {
    find_static_scss_variable_declaration_in_scope(
        name,
        0,
        position,
        context.scopes,
        context.variable_declarations,
    )
    .is_some()
}
