use super::*;
use std::collections::{BTreeMap, BTreeSet};

pub(in crate::style::transform) fn static_scss_module_instance_identity_key(
    style_path: &str,
    variable_overrides: &BTreeMap<String, String>,
) -> String {
    let canonical_path = canonicalize_omena_resolver_style_identity_path(style_path);
    let mut key = format!("path:{}:{canonical_path}", canonical_path.len());
    key.push('|');
    key.push_str(static_scss_module_configuration_signature(variable_overrides).as_str());
    key
}

pub(in crate::style::transform) fn static_scss_module_configuration_signature(
    variable_overrides: &BTreeMap<String, String>,
) -> String {
    if variable_overrides.is_empty() {
        return "with:none".to_string();
    }
    let mut key = String::from("with");
    for (name, value) in variable_overrides {
        key.push('|');
        key.push_str(name.len().to_string().as_str());
        key.push(':');
        key.push_str(name);
        key.push('=');
        key.push_str(value.len().to_string().as_str());
        key.push(':');
        key.push_str(value);
    }
    key
}

pub(super) fn resolve_static_scss_module_effective_variable_overrides(
    style_path: &str,
    variable_overrides: &BTreeMap<String, String>,
    loaded_module_overrides_by_path: &mut BTreeMap<String, BTreeMap<String, String>>,
) -> Option<BTreeMap<String, String>> {
    let canonical_path = canonicalize_omena_resolver_style_identity_path(style_path);
    match loaded_module_overrides_by_path.get(canonical_path.as_str()) {
        Some(existing_overrides) if variable_overrides.is_empty() => {
            Some(existing_overrides.clone())
        }
        Some(existing_overrides) => {
            (existing_overrides == variable_overrides).then(|| variable_overrides.clone())
        }
        None => {
            loaded_module_overrides_by_path.insert(canonical_path, variable_overrides.clone());
            Some(variable_overrides.clone())
        }
    }
}

pub(super) fn static_scss_module_configuration_variables_are_valid(
    variable_overrides: &BTreeMap<String, String>,
    configurable_names: &BTreeSet<String>,
) -> bool {
    variable_overrides
        .keys()
        .all(|name| configurable_names.contains(name))
}
