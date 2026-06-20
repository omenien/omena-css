use std::collections::{BTreeMap, BTreeSet};

use omena_parser::StyleDialect;

use super::{
    declarations::collect_static_scss_variable_declarations,
    names::static_scss_public_module_variable_name, scopes::collect_static_stylesheet_scopes,
    scss_variables::resolve_static_scss_variable_value_in_scope,
};

pub fn derive_static_scss_stylesheet_module_variable_exports(
    style_source: &str,
) -> BTreeMap<String, String> {
    let facts = omena_parser::collect_style_facts(style_source, StyleDialect::Scss);
    let scopes = match collect_static_stylesheet_scopes(style_source) {
        Some(scopes) => scopes,
        None => return BTreeMap::new(),
    };
    let declarations = match collect_static_scss_variable_declarations(
        style_source,
        StyleDialect::Scss,
        &facts.variables,
        &scopes,
    ) {
        Some(declarations) => declarations,
        None => return BTreeMap::new(),
    };

    let mut exports = BTreeMap::new();
    for declaration in declarations
        .iter()
        .filter(|declaration| declaration.scope_id == 0)
    {
        let Some(public_name) = static_scss_public_module_variable_name(declaration.name.as_str())
        else {
            continue;
        };
        let mut stack = BTreeSet::new();
        if let Some(value) = resolve_static_scss_variable_value_in_scope(
            declaration.name.as_str(),
            0,
            usize::MAX,
            &scopes,
            &declarations,
            &mut stack,
        ) {
            exports.insert(public_name, value);
        }
    }
    exports
}

pub fn derive_static_scss_stylesheet_module_configurable_variable_names(
    style_source: &str,
) -> BTreeSet<String> {
    let facts = omena_parser::collect_style_facts(style_source, StyleDialect::Scss);
    let scopes = match collect_static_stylesheet_scopes(style_source) {
        Some(scopes) => scopes,
        None => return BTreeSet::new(),
    };
    let declarations = match collect_static_scss_variable_declarations(
        style_source,
        StyleDialect::Scss,
        &facts.variables,
        &scopes,
    ) {
        Some(declarations) => declarations,
        None => return BTreeSet::new(),
    };

    declarations
        .iter()
        .filter(|declaration| declaration.scope_id == 0)
        .filter(|declaration| declaration.declaration.is_default)
        .filter_map(|declaration| {
            static_scss_public_module_variable_name(declaration.name.as_str())
        })
        .collect()
}
