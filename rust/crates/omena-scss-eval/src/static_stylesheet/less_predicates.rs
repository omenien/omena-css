use std::collections::BTreeMap;

use omena_value_lattice::parse_whole_function_value_arguments;

use super::less_guard::{
    static_less_guard_unit_text, static_less_guard_value_has_unit,
    static_less_guard_value_is_color, static_less_guard_value_is_keyword,
    static_less_guard_value_is_number, static_less_guard_value_is_string,
    static_less_guard_value_is_url, static_less_value_condition_matches,
};
use super::less_mixin_values::static_less_value_is_detached_ruleset_reference;
use super::model::{
    StaticLessDetachedRulesetDeclaration, StaticStylesheetPropertyDeclaration,
    StaticStylesheetScope, StaticStylesheetVariableDeclaration,
};
use super::{
    find_static_less_detached_ruleset_declaration, find_static_less_property_declaration_before,
    find_static_less_variable_declaration, static_less_variable_name_is_safe,
    static_stylesheet_literal_value_is_safe, static_stylesheet_property_name_is_safe,
};

pub(super) fn parse_static_less_if_value(value: &str) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, "if")?;
    let [condition, truthy, falsey] = arguments.as_slice() else {
        return None;
    };
    Some(
        if static_less_value_condition_matches(condition.trim())? {
            truthy
        } else {
            falsey
        }
        .trim()
        .to_string(),
    )
}

pub(super) fn parse_static_less_boolean_value(value: &str) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, "boolean")?;
    let [condition] = arguments.as_slice() else {
        return None;
    };
    Some(static_less_value_condition_matches(condition.trim())?.to_string())
}

pub(super) fn parse_static_less_isnumber_value(value: &str) -> Option<String> {
    parse_static_less_unary_predicate_value(value, "isnumber", static_less_guard_value_is_number)
}

pub(super) fn parse_static_less_iscolor_value(value: &str) -> Option<String> {
    parse_static_less_unary_predicate_value(value, "iscolor", static_less_guard_value_is_color)
}

pub(super) fn parse_static_less_isstring_value(value: &str) -> Option<String> {
    parse_static_less_unary_predicate_value(value, "isstring", static_less_guard_value_is_string)
}

pub(super) fn parse_static_less_iskeyword_value(value: &str) -> Option<String> {
    parse_static_less_unary_predicate_value(value, "iskeyword", static_less_guard_value_is_keyword)
}

pub(super) fn parse_static_less_isurl_value(value: &str) -> Option<String> {
    parse_static_less_unary_predicate_value(value, "isurl", static_less_guard_value_is_url)
}

pub(super) fn parse_static_less_isdefined_value(value: &str) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, "isdefined")?;
    let [value] = arguments.as_slice() else {
        return None;
    };
    let value = value.trim();
    (!value.starts_with('@') && !value.starts_with('$')).then_some(true.to_string())
}

pub(super) fn parse_static_less_isdefined_value_with_context(
    value: &str,
    scope_id: usize,
    reference_position: usize,
    scopes: &[StaticStylesheetScope],
    variable_declarations: &BTreeMap<(usize, String), StaticStylesheetVariableDeclaration>,
    property_declarations: &BTreeMap<(usize, String), StaticStylesheetPropertyDeclaration>,
    detached_ruleset_declarations: &[StaticLessDetachedRulesetDeclaration],
) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, "isdefined")?;
    let [value] = arguments.as_slice() else {
        return None;
    };
    static_less_isdefined_argument_matches(
        value,
        scope_id,
        reference_position,
        scopes,
        variable_declarations,
        property_declarations,
        detached_ruleset_declarations,
    )
    .map(|defined| defined.to_string())
}

fn static_less_isdefined_argument_matches(
    value: &str,
    scope_id: usize,
    reference_position: usize,
    scopes: &[StaticStylesheetScope],
    variable_declarations: &BTreeMap<(usize, String), StaticStylesheetVariableDeclaration>,
    property_declarations: &BTreeMap<(usize, String), StaticStylesheetPropertyDeclaration>,
    detached_ruleset_declarations: &[StaticLessDetachedRulesetDeclaration],
) -> Option<bool> {
    let value = value.trim();
    if value.is_empty() {
        return None;
    }
    if value.starts_with('$') {
        if !value
            .strip_prefix('$')
            .is_some_and(static_stylesheet_property_name_is_safe)
        {
            return None;
        }
        return Some(
            find_static_less_property_declaration_before(
                value,
                scope_id,
                scopes,
                property_declarations,
                reference_position,
            )
            .is_some(),
        );
    }
    if !value.starts_with('@') {
        return Some(true);
    }
    if value.starts_with("@@") || !static_less_variable_name_is_safe(value) {
        return None;
    }
    Some(
        find_static_less_variable_declaration(value, scope_id, scopes, variable_declarations)
            .is_some()
            || find_static_less_detached_ruleset_declaration(
                value,
                scope_id,
                scopes,
                detached_ruleset_declarations,
            )
            .is_some(),
    )
}

pub(super) fn parse_static_less_isruleset_value(value: &str) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, "isruleset")?;
    let [value] = arguments.as_slice() else {
        return None;
    };
    let value = value.trim();
    (!value.starts_with('@') && static_stylesheet_literal_value_is_safe(value))
        .then(|| false.to_string())
}

pub(super) fn parse_static_less_isruleset_value_with_context(
    value: &str,
    scope_id: usize,
    scopes: &[StaticStylesheetScope],
    detached_ruleset_declarations: &[StaticLessDetachedRulesetDeclaration],
) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, "isruleset")?;
    let [value] = arguments.as_slice() else {
        return None;
    };
    let value = value.trim();
    if static_less_value_is_detached_ruleset_reference(
        value,
        scope_id,
        scopes,
        detached_ruleset_declarations,
    ) {
        return Some(true.to_string());
    }
    (!value.starts_with('@') && static_stylesheet_literal_value_is_safe(value))
        .then(|| false.to_string())
}

pub(super) fn parse_static_less_ispixel_value(value: &str) -> Option<String> {
    parse_static_less_unary_predicate_value(value, "ispixel", |value| {
        static_less_guard_value_has_unit(value, "px")
    })
}

pub(super) fn parse_static_less_ispercentage_value(value: &str) -> Option<String> {
    parse_static_less_unary_predicate_value(value, "ispercentage", |value| {
        static_less_guard_value_has_unit(value, "%")
    })
}

pub(super) fn parse_static_less_isem_value(value: &str) -> Option<String> {
    parse_static_less_unary_predicate_value(value, "isem", |value| {
        static_less_guard_value_has_unit(value, "em")
    })
}

pub(super) fn parse_static_less_isunit_value(value: &str) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, "isunit")?;
    let [value, unit] = arguments.as_slice() else {
        return None;
    };
    let unit = static_less_guard_unit_text(unit.trim())?;
    Some(static_less_guard_value_has_unit(value.trim(), unit).to_string())
}

fn parse_static_less_unary_predicate_value(
    value: &str,
    function_name: &str,
    predicate: impl FnOnce(&str) -> bool,
) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, function_name)?;
    let [value] = arguments.as_slice() else {
        return None;
    };
    Some(predicate(value.trim()).to_string())
}
