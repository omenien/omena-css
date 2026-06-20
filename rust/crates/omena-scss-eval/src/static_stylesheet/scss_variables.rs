use std::collections::BTreeSet;

use crate::{
    scss_metadata::reduce_static_scss_metadata_with_context, value_eval::reduce_static_scss_value,
};

use super::{
    model::{
        StaticStylesheetScope, StaticStylesheetScopedVariableDeclaration,
        StaticStylesheetVariableKind,
    },
    names::{canonical_static_scss_variable_name, static_scss_variable_names_equal},
    safety::{static_stylesheet_composite_value_is_safe, static_stylesheet_literal_value_is_safe},
    scopes::static_stylesheet_scope_for_position,
    value_resolution_model::{
        StaticStylesheetAbstractResolution, StaticStylesheetResolutionReason,
        raw_static_abstract_value, resolved_static_abstract_value_preserving_callable_raw,
        top_static_abstract_value,
    },
    variable_references::collect_static_stylesheet_variable_references,
};

pub(super) fn resolve_static_scss_variable_abstract_value_at_position(
    name: &str,
    position: usize,
    scopes: &[StaticStylesheetScope],
    declarations: &[StaticStylesheetScopedVariableDeclaration],
    stack: &mut BTreeSet<(usize, String, usize)>,
    fuel: usize,
) -> StaticStylesheetAbstractResolution {
    let Some(scope_id) = static_stylesheet_scope_for_position(scopes, position) else {
        return top_static_abstract_value(StaticStylesheetResolutionReason::UnresolvedReference);
    };
    resolve_static_scss_variable_abstract_value_in_scope(
        name,
        scope_id,
        position,
        scopes,
        declarations,
        stack,
        fuel,
    )
}

fn resolve_static_scss_variable_abstract_value_in_scope(
    name: &str,
    scope_id: usize,
    position: usize,
    scopes: &[StaticStylesheetScope],
    declarations: &[StaticStylesheetScopedVariableDeclaration],
    stack: &mut BTreeSet<(usize, String, usize)>,
    fuel: usize,
) -> StaticStylesheetAbstractResolution {
    if fuel == 0 {
        return top_static_abstract_value(StaticStylesheetResolutionReason::FuelExhausted);
    }
    let Some(declaration) =
        find_static_scss_variable_declaration(name, scope_id, position, scopes, declarations)
    else {
        return top_static_abstract_value(StaticStylesheetResolutionReason::UnresolvedReference);
    };
    let stack_key = (
        declaration.scope_id,
        canonical_static_scss_variable_name(name),
        declaration.declaration.span_start,
    );
    if !stack.insert(stack_key.clone()) {
        return top_static_abstract_value(StaticStylesheetResolutionReason::Cycle);
    }
    let resolved = resolve_static_scss_variable_abstract_value_text(
        declaration.declaration.value.trim(),
        declaration.declaration.span_start,
        scopes,
        declarations,
        stack,
        fuel - 1,
    );
    stack.remove(&stack_key);
    resolved
}

fn resolve_static_scss_variable_abstract_value_text(
    value: &str,
    position: usize,
    scopes: &[StaticStylesheetScope],
    declarations: &[StaticStylesheetScopedVariableDeclaration],
    stack: &mut BTreeSet<(usize, String, usize)>,
    fuel: usize,
) -> StaticStylesheetAbstractResolution {
    let Some(references) =
        collect_static_stylesheet_variable_references(value, StaticStylesheetVariableKind::Scss)
    else {
        return raw_static_abstract_value(
            value,
            StaticStylesheetResolutionReason::UnsupportedDynamic,
        );
    };
    if references.is_empty() {
        let metadata_reduced_value = reduce_static_scss_metadata_with_variable_context(
            value,
            position,
            scopes,
            declarations,
        )
        .unwrap_or_else(|| value.to_string());
        let reduced = reduce_static_scss_value(metadata_reduced_value.clone());
        if static_stylesheet_literal_value_is_safe(reduced.as_str()) {
            return resolved_static_abstract_value_preserving_callable_raw(value, reduced.as_str());
        }
        return raw_static_abstract_value(
            metadata_reduced_value.as_str(),
            StaticStylesheetResolutionReason::UnsupportedDynamic,
        );
    }
    if !static_stylesheet_composite_value_is_safe(value) {
        return raw_static_abstract_value(
            value,
            StaticStylesheetResolutionReason::UnsupportedDynamic,
        );
    }

    let mut output = String::with_capacity(value.len());
    let mut cursor = 0usize;
    for reference in references {
        let resolved = resolve_static_scss_variable_abstract_value_at_position(
            reference.name.as_str(),
            position,
            scopes,
            declarations,
            stack,
            fuel,
        );
        let Some(rendered_value) = resolved.rendered_value else {
            return top_static_abstract_value(resolved.reason);
        };
        output.push_str(&value[cursor..reference.start]);
        output.push_str(&rendered_value);
        cursor = reference.end;
    }
    output.push_str(&value[cursor..]);
    let output = reduce_static_scss_metadata_with_variable_context(
        output.as_str(),
        position,
        scopes,
        declarations,
    )
    .unwrap_or(output);
    let reduced_output = reduce_static_scss_value(output.clone());
    resolved_static_abstract_value_preserving_callable_raw(output.as_str(), reduced_output.as_str())
}

pub(super) fn resolve_static_scss_variable_value_at_position(
    name: &str,
    position: usize,
    scopes: &[StaticStylesheetScope],
    declarations: &[StaticStylesheetScopedVariableDeclaration],
    stack: &mut BTreeSet<(usize, String, usize)>,
) -> Option<String> {
    let scope_id = static_stylesheet_scope_for_position(scopes, position)?;
    resolve_static_scss_variable_value_in_scope(
        name,
        scope_id,
        position,
        scopes,
        declarations,
        stack,
    )
}

pub(super) fn resolve_static_scss_variable_value_in_scope(
    name: &str,
    scope_id: usize,
    position: usize,
    scopes: &[StaticStylesheetScope],
    declarations: &[StaticStylesheetScopedVariableDeclaration],
    stack: &mut BTreeSet<(usize, String, usize)>,
) -> Option<String> {
    let stack_key = (
        scope_id,
        canonical_static_scss_variable_name(name),
        position,
    );
    if !stack.insert(stack_key.clone()) {
        return None;
    }
    let declaration =
        find_static_scss_variable_declaration(name, scope_id, position, scopes, declarations)?;
    let resolved = resolve_static_scss_variable_value_text(
        declaration.declaration.value.trim(),
        declaration.declaration.span_start,
        scopes,
        declarations,
        stack,
    );
    stack.remove(&stack_key);
    resolved
}

pub(super) fn find_static_scss_variable_declaration<'a>(
    name: &str,
    mut scope_id: usize,
    position: usize,
    scopes: &[StaticStylesheetScope],
    declarations: &'a [StaticStylesheetScopedVariableDeclaration],
) -> Option<&'a StaticStylesheetScopedVariableDeclaration> {
    loop {
        if let Some(declaration) = find_static_scss_variable_declaration_in_scope(
            name,
            scope_id,
            position,
            scopes,
            declarations,
        ) {
            return Some(declaration);
        }
        scope_id = scopes.get(scope_id)?.parent_id?;
    }
}

pub(super) fn find_static_scss_variable_declaration_in_scope<'a>(
    name: &str,
    scope_id: usize,
    position: usize,
    scopes: &[StaticStylesheetScope],
    declarations: &'a [StaticStylesheetScopedVariableDeclaration],
) -> Option<&'a StaticStylesheetScopedVariableDeclaration> {
    let mut active = None;
    for declaration in declarations.iter().filter(|declaration| {
        static_scss_variable_names_equal(&declaration.name, name)
            && declaration.scope_id == scope_id
            && declaration.declaration.span_end <= position
    }) {
        if declaration.declaration.is_default {
            let has_visible_value = active.is_some()
                || scopes
                    .get(scope_id)
                    .and_then(|scope| scope.parent_id)
                    .and_then(|parent_scope_id| {
                        find_static_scss_variable_declaration(
                            name,
                            parent_scope_id,
                            declaration.declaration.span_start,
                            scopes,
                            declarations,
                        )
                    })
                    .is_some();
            if !has_visible_value {
                active = Some(declaration);
            }
            continue;
        }
        active = Some(declaration);
    }
    active
}

fn reduce_static_scss_metadata_with_variable_context(
    value: &str,
    position: usize,
    scopes: &[StaticStylesheetScope],
    declarations: &[StaticStylesheetScopedVariableDeclaration],
) -> Option<String> {
    reduce_static_scss_metadata_with_context(
        value,
        |_| None,
        |_| None,
        |name| {
            Some(
                static_stylesheet_scope_for_position(scopes, position)
                    .and_then(|scope_id| {
                        find_static_scss_variable_declaration(
                            name,
                            scope_id,
                            position,
                            scopes,
                            declarations,
                        )
                    })
                    .is_some(),
            )
        },
        |name| {
            Some(
                find_static_scss_variable_declaration_in_scope(
                    name,
                    0,
                    position,
                    scopes,
                    declarations,
                )
                .is_some(),
            )
        },
    )
}

fn resolve_static_scss_variable_value_text(
    value: &str,
    position: usize,
    scopes: &[StaticStylesheetScope],
    declarations: &[StaticStylesheetScopedVariableDeclaration],
    stack: &mut BTreeSet<(usize, String, usize)>,
) -> Option<String> {
    let references =
        collect_static_stylesheet_variable_references(value, StaticStylesheetVariableKind::Scss)?;
    if references.is_empty() {
        let value = reduce_static_scss_metadata_with_variable_context(
            value,
            position,
            scopes,
            declarations,
        )
        .unwrap_or_else(|| value.to_string());
        let reduced = reduce_static_scss_value(value);
        return static_stylesheet_literal_value_is_safe(reduced.as_str()).then_some(reduced);
    }
    if !static_stylesheet_composite_value_is_safe(value) {
        return None;
    }

    let mut output = String::with_capacity(value.len());
    let mut cursor = 0usize;
    for reference in references {
        let resolved = resolve_static_scss_variable_value_at_position(
            reference.name.as_str(),
            position,
            scopes,
            declarations,
            stack,
        )?;
        output.push_str(&value[cursor..reference.start]);
        output.push_str(&resolved);
        cursor = reference.end;
    }
    output.push_str(&value[cursor..]);
    let output = reduce_static_scss_metadata_with_variable_context(
        output.as_str(),
        position,
        scopes,
        declarations,
    )
    .unwrap_or(output);
    Some(reduce_static_scss_value(output))
}
