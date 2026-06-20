use std::collections::{BTreeMap, BTreeSet};

use super::{
    StaticStylesheetAbstractResolution, StaticStylesheetResolutionReason,
    collect_static_less_property_variable_references,
    collect_static_stylesheet_variable_references,
    collect_static_stylesheet_variable_references_with_options,
    less_predicates::{
        parse_static_less_isdefined_value_with_context,
        parse_static_less_isruleset_value_with_context,
    },
    less_strings::preserve_static_less_dynamic_escaped_string_value,
    less_values::{reduce_static_less_value, reduce_static_less_value_with_escape_flag},
    model::{
        StaticLessDetachedRulesetDeclaration, StaticLessResolvedValue,
        StaticStylesheetPropertyDeclaration, StaticStylesheetScope,
        StaticStylesheetVariableDeclaration, StaticStylesheetVariableKind,
    },
    raw_static_abstract_value, resolved_static_abstract_value,
    static_stylesheet_composite_value_is_safe, static_stylesheet_literal_value_is_safe,
    top_static_abstract_value,
};

#[allow(clippy::too_many_arguments)]
pub(super) fn resolve_static_less_variable_abstract_value_in_scope(
    name: &str,
    scope_id: usize,
    scopes: &[StaticStylesheetScope],
    declarations: &BTreeMap<(usize, String), StaticStylesheetVariableDeclaration>,
    property_declarations: &BTreeMap<(usize, String), StaticStylesheetPropertyDeclaration>,
    detached_ruleset_declarations: &[StaticLessDetachedRulesetDeclaration],
    stack: &mut BTreeSet<(usize, String)>,
    fuel: usize,
) -> StaticStylesheetAbstractResolution {
    if fuel == 0 {
        return top_static_abstract_value(StaticStylesheetResolutionReason::FuelExhausted);
    }
    let Some(declaration) =
        find_static_less_variable_declaration(name, scope_id, scopes, declarations)
    else {
        return top_static_abstract_value(StaticStylesheetResolutionReason::UnresolvedReference);
    };
    let stack_key = (scope_id, name.to_string());
    if !stack.insert(stack_key.clone()) {
        return top_static_abstract_value(StaticStylesheetResolutionReason::Cycle);
    }
    let resolved = resolve_static_less_variable_abstract_value_text(
        declaration.value.trim(),
        scope_id,
        declaration.span_start,
        scopes,
        declarations,
        property_declarations,
        detached_ruleset_declarations,
        stack,
        fuel - 1,
    );
    stack.remove(&stack_key);
    resolved
}

#[allow(clippy::too_many_arguments)]
fn resolve_static_less_variable_abstract_value_text(
    value: &str,
    scope_id: usize,
    reference_position: usize,
    scopes: &[StaticStylesheetScope],
    declarations: &BTreeMap<(usize, String), StaticStylesheetVariableDeclaration>,
    property_declarations: &BTreeMap<(usize, String), StaticStylesheetPropertyDeclaration>,
    detached_ruleset_declarations: &[StaticLessDetachedRulesetDeclaration],
    stack: &mut BTreeSet<(usize, String)>,
    fuel: usize,
) -> StaticStylesheetAbstractResolution {
    if let Some(value) = parse_static_less_isdefined_value_with_context(
        value,
        scope_id,
        reference_position,
        scopes,
        declarations,
        property_declarations,
        detached_ruleset_declarations,
    ) {
        return resolved_static_abstract_value(value.as_str());
    }
    if let Some(value) = parse_static_less_isruleset_value_with_context(
        value,
        scope_id,
        scopes,
        detached_ruleset_declarations,
    ) {
        return resolved_static_abstract_value(value.as_str());
    }
    let Some(references) = collect_static_stylesheet_variable_references_with_options(
        value,
        StaticStylesheetVariableKind::Less,
        false,
        true,
    ) else {
        return raw_static_abstract_value(
            value,
            StaticStylesheetResolutionReason::UnsupportedDynamic,
        );
    };
    let Some(property_references) = collect_static_less_property_variable_references(value) else {
        return raw_static_abstract_value(
            value,
            StaticStylesheetResolutionReason::UnsupportedDynamic,
        );
    };
    if references.is_empty() && property_references.is_empty() {
        if static_stylesheet_literal_value_is_safe(value) {
            return resolved_static_abstract_value(
                reduce_static_less_value(value.to_string()).as_str(),
            );
        }
        return raw_static_abstract_value(
            value,
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
        let resolved = resolve_static_less_variable_abstract_value_in_scope(
            reference.name.as_str(),
            scope_id,
            scopes,
            declarations,
            property_declarations,
            detached_ruleset_declarations,
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
    if !property_references.is_empty() {
        let mut property_stack = BTreeSet::new();
        let resolved = resolve_static_less_property_references_in_value(
            output.as_str(),
            scope_id,
            scopes,
            property_declarations,
            &mut property_stack,
        );
        let Some(rendered_value) = resolved.rendered_value else {
            return top_static_abstract_value(resolved.reason);
        };
        output = rendered_value;
    }
    resolved_static_abstract_value(reduce_static_less_value(output).as_str())
}

pub(super) fn resolve_static_less_variable_value_in_scope(
    name: &str,
    scope_id: usize,
    scopes: &[StaticStylesheetScope],
    declarations: &BTreeMap<(usize, String), StaticStylesheetVariableDeclaration>,
    property_declarations: &BTreeMap<(usize, String), StaticStylesheetPropertyDeclaration>,
    detached_ruleset_declarations: &[StaticLessDetachedRulesetDeclaration],
    stack: &mut BTreeSet<(usize, String)>,
) -> Option<StaticLessResolvedValue> {
    let stack_key = (scope_id, name.to_string());
    if !stack.insert(stack_key.clone()) {
        return None;
    }
    let declaration = find_static_less_variable_declaration(name, scope_id, scopes, declarations)?;
    let resolved = resolve_static_less_variable_value_text(
        declaration.value.trim(),
        scope_id,
        declaration.span_start,
        scopes,
        declarations,
        property_declarations,
        detached_ruleset_declarations,
        stack,
    );
    stack.remove(&stack_key);
    resolved.map(|resolved| {
        if resolved.escaped {
            resolved
        } else {
            reduce_static_less_value_with_escape_flag(resolved.text)
        }
    })
}

pub(super) fn find_static_less_variable_declaration<'a>(
    name: &str,
    mut scope_id: usize,
    scopes: &[StaticStylesheetScope],
    declarations: &'a BTreeMap<(usize, String), StaticStylesheetVariableDeclaration>,
) -> Option<&'a StaticStylesheetVariableDeclaration> {
    loop {
        if let Some(declaration) = declarations.get(&(scope_id, name.to_string())) {
            return Some(declaration);
        }
        scope_id = scopes.get(scope_id)?.parent_id?;
    }
}

#[allow(clippy::too_many_arguments)]
fn resolve_static_less_variable_value_text(
    value: &str,
    scope_id: usize,
    reference_position: usize,
    scopes: &[StaticStylesheetScope],
    declarations: &BTreeMap<(usize, String), StaticStylesheetVariableDeclaration>,
    property_declarations: &BTreeMap<(usize, String), StaticStylesheetPropertyDeclaration>,
    detached_ruleset_declarations: &[StaticLessDetachedRulesetDeclaration],
    stack: &mut BTreeSet<(usize, String)>,
) -> Option<StaticLessResolvedValue> {
    if let Some(value) = parse_static_less_isdefined_value_with_context(
        value,
        scope_id,
        reference_position,
        scopes,
        declarations,
        property_declarations,
        detached_ruleset_declarations,
    ) {
        return Some(StaticLessResolvedValue {
            text: value,
            escaped: false,
        });
    }
    if let Some(value) = parse_static_less_isruleset_value_with_context(
        value,
        scope_id,
        scopes,
        detached_ruleset_declarations,
    ) {
        return Some(StaticLessResolvedValue {
            text: value,
            escaped: false,
        });
    }
    let references = collect_static_stylesheet_variable_references_with_options(
        value,
        StaticStylesheetVariableKind::Less,
        false,
        true,
    )?;
    let property_references = collect_static_less_property_variable_references(value)?;
    if references.is_empty() && property_references.is_empty() {
        if let Some(preserved) = preserve_static_less_dynamic_escaped_string_value(value) {
            return Some(preserved);
        }
        return static_stylesheet_literal_value_is_safe(value)
            .then(|| reduce_static_less_value_with_escape_flag(value.to_string()));
    }
    if !static_stylesheet_composite_value_is_safe(value) {
        return None;
    }

    let mut output = String::with_capacity(value.len());
    let mut cursor = 0usize;
    let mut escaped = false;
    for reference in references {
        let resolved = resolve_static_less_variable_value_in_scope(
            reference.name.as_str(),
            scope_id,
            scopes,
            declarations,
            property_declarations,
            detached_ruleset_declarations,
            stack,
        )?;
        escaped |= resolved.escaped;
        output.push_str(&value[cursor..reference.start]);
        output.push_str(&resolved.text);
        cursor = reference.end;
    }
    output.push_str(&value[cursor..]);
    if !property_references.is_empty() {
        let mut property_stack = BTreeSet::new();
        let resolved = resolve_static_less_property_value_text(
            output.as_str(),
            scope_id,
            scopes,
            property_declarations,
            &mut property_stack,
        )?;
        escaped |= resolved.escaped;
        output = resolved.text;
    }
    Some(if escaped {
        StaticLessResolvedValue {
            text: output,
            escaped,
        }
    } else {
        reduce_static_less_value_with_escape_flag(output)
    })
}

pub(super) fn resolve_static_less_property_abstract_value_in_scope(
    name: &str,
    scope_id: usize,
    scopes: &[StaticStylesheetScope],
    declarations: &BTreeMap<(usize, String), StaticStylesheetPropertyDeclaration>,
    stack: &mut BTreeSet<(usize, String)>,
    fuel: usize,
) -> StaticStylesheetAbstractResolution {
    if fuel == 0 {
        return top_static_abstract_value(StaticStylesheetResolutionReason::FuelExhausted);
    }
    let Some(declaration) =
        find_static_less_property_declaration(name, scope_id, scopes, declarations)
    else {
        return top_static_abstract_value(StaticStylesheetResolutionReason::UnresolvedReference);
    };
    let stack_key = (scope_id, name.to_string());
    if !stack.insert(stack_key.clone()) {
        return top_static_abstract_value(StaticStylesheetResolutionReason::Cycle);
    }
    let resolved = resolve_static_less_property_abstract_value_text(
        declaration.value.trim(),
        scope_id,
        scopes,
        declarations,
        stack,
        fuel - 1,
    );
    stack.remove(&stack_key);
    resolved
}

fn resolve_static_less_property_abstract_value_text(
    value: &str,
    scope_id: usize,
    scopes: &[StaticStylesheetScope],
    declarations: &BTreeMap<(usize, String), StaticStylesheetPropertyDeclaration>,
    stack: &mut BTreeSet<(usize, String)>,
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
        if static_stylesheet_literal_value_is_safe(value) {
            return resolved_static_abstract_value(
                reduce_static_less_value(value.to_string()).as_str(),
            );
        }
        return raw_static_abstract_value(
            value,
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
        let resolved = resolve_static_less_property_abstract_value_in_scope(
            reference.name.as_str(),
            scope_id,
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
    resolved_static_abstract_value(reduce_static_less_value(output).as_str())
}

pub(super) fn resolve_static_less_property_value_in_scope(
    name: &str,
    scope_id: usize,
    scopes: &[StaticStylesheetScope],
    declarations: &BTreeMap<(usize, String), StaticStylesheetPropertyDeclaration>,
    stack: &mut BTreeSet<(usize, String)>,
) -> Option<StaticLessResolvedValue> {
    resolve_static_less_property_value_in_scope_with_position(
        name,
        scope_id,
        scopes,
        declarations,
        stack,
        None,
    )
}

fn resolve_static_less_property_value_in_scope_with_position(
    name: &str,
    scope_id: usize,
    scopes: &[StaticStylesheetScope],
    declarations: &BTreeMap<(usize, String), StaticStylesheetPropertyDeclaration>,
    stack: &mut BTreeSet<(usize, String)>,
    reference_position: Option<usize>,
) -> Option<StaticLessResolvedValue> {
    let stack_key = (scope_id, name.to_string());
    if !stack.insert(stack_key.clone()) {
        return None;
    }
    let declaration = match reference_position {
        Some(position) => find_static_less_property_declaration_before(
            name,
            scope_id,
            scopes,
            declarations,
            position,
        ),
        None => find_static_less_property_declaration(name, scope_id, scopes, declarations),
    }?;
    let resolved = resolve_static_less_property_value_text_with_position(
        declaration.value.trim(),
        scope_id,
        scopes,
        declarations,
        stack,
        reference_position,
    );
    stack.remove(&stack_key);
    resolved.map(|resolved| {
        if resolved.escaped {
            resolved
        } else {
            reduce_static_less_value_with_escape_flag(resolved.text)
        }
    })
}

pub(super) fn find_static_less_property_declaration<'a>(
    name: &str,
    mut scope_id: usize,
    scopes: &[StaticStylesheetScope],
    declarations: &'a BTreeMap<(usize, String), StaticStylesheetPropertyDeclaration>,
) -> Option<&'a StaticStylesheetPropertyDeclaration> {
    loop {
        if let Some(declaration) = declarations.get(&(scope_id, name.to_string())) {
            return Some(declaration);
        }
        scope_id = scopes.get(scope_id)?.parent_id?;
    }
}

pub(super) fn find_static_less_property_declaration_before<'a>(
    name: &str,
    scope_id: usize,
    scopes: &[StaticStylesheetScope],
    declarations: &'a BTreeMap<(usize, String), StaticStylesheetPropertyDeclaration>,
    reference_position: usize,
) -> Option<&'a StaticStylesheetPropertyDeclaration> {
    find_static_less_property_declaration(name, scope_id, scopes, declarations)
        .filter(|declaration| declaration.span_start < reference_position)
}

fn resolve_static_less_property_value_text(
    value: &str,
    scope_id: usize,
    scopes: &[StaticStylesheetScope],
    declarations: &BTreeMap<(usize, String), StaticStylesheetPropertyDeclaration>,
    stack: &mut BTreeSet<(usize, String)>,
) -> Option<StaticLessResolvedValue> {
    resolve_static_less_property_value_text_with_position(
        value,
        scope_id,
        scopes,
        declarations,
        stack,
        None,
    )
}

pub(super) fn resolve_static_less_property_value_text_with_position(
    value: &str,
    scope_id: usize,
    scopes: &[StaticStylesheetScope],
    declarations: &BTreeMap<(usize, String), StaticStylesheetPropertyDeclaration>,
    stack: &mut BTreeSet<(usize, String)>,
    reference_position: Option<usize>,
) -> Option<StaticLessResolvedValue> {
    let references =
        collect_static_stylesheet_variable_references(value, StaticStylesheetVariableKind::Scss)?;
    if references.is_empty() {
        if let Some(preserved) = preserve_static_less_dynamic_escaped_string_value(value) {
            return Some(preserved);
        }
        return static_stylesheet_literal_value_is_safe(value)
            .then(|| reduce_static_less_value_with_escape_flag(value.to_string()));
    }
    if !static_stylesheet_composite_value_is_safe(value) {
        return None;
    }

    let mut output = String::with_capacity(value.len());
    let mut cursor = 0usize;
    let mut escaped = false;
    for reference in references {
        let resolved = resolve_static_less_property_value_in_scope_with_position(
            reference.name.as_str(),
            scope_id,
            scopes,
            declarations,
            stack,
            reference_position,
        )?;
        escaped |= resolved.escaped;
        output.push_str(&value[cursor..reference.start]);
        output.push_str(&resolved.text);
        cursor = reference.end;
    }
    output.push_str(&value[cursor..]);
    Some(if escaped {
        StaticLessResolvedValue {
            text: output,
            escaped,
        }
    } else {
        reduce_static_less_value_with_escape_flag(output)
    })
}

fn resolve_static_less_property_references_in_value(
    value: &str,
    scope_id: usize,
    scopes: &[StaticStylesheetScope],
    declarations: &BTreeMap<(usize, String), StaticStylesheetPropertyDeclaration>,
    stack: &mut BTreeSet<(usize, String)>,
) -> StaticStylesheetAbstractResolution {
    let Some(references) = collect_static_less_property_variable_references(value) else {
        return raw_static_abstract_value(
            value,
            StaticStylesheetResolutionReason::UnsupportedDynamic,
        );
    };
    if references.is_empty() {
        if static_stylesheet_literal_value_is_safe(value) {
            return resolved_static_abstract_value(
                reduce_static_less_value(value.to_string()).as_str(),
            );
        }
        return raw_static_abstract_value(
            value,
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
        let resolved = resolve_static_less_property_abstract_value_in_scope(
            reference.name.as_str(),
            scope_id,
            scopes,
            declarations,
            stack,
            super::STATIC_STYLESHEET_VALUE_RESOLUTION_FUEL_LIMIT,
        );
        let Some(rendered_value) = resolved.rendered_value else {
            return top_static_abstract_value(resolved.reason);
        };
        output.push_str(&value[cursor..reference.start]);
        output.push_str(&rendered_value);
        cursor = reference.end;
    }
    output.push_str(&value[cursor..]);
    resolved_static_abstract_value(reduce_static_less_value(output).as_str())
}
