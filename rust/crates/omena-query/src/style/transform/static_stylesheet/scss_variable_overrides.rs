use super::super::super::stylesheet_evaluation::derive_static_scss_stylesheet_module_configurable_variable_names;
use std::{borrow::Cow, collections::BTreeMap};

pub(super) fn apply_static_scss_module_variable_overrides<'a>(
    style_source: &'a str,
    variable_overrides: &BTreeMap<String, String>,
) -> Cow<'a, str> {
    if variable_overrides.is_empty() {
        return Cow::Borrowed(style_source);
    }
    let configurable_names =
        derive_static_scss_stylesheet_module_configurable_variable_names(style_source);
    if !variable_overrides
        .keys()
        .all(|name| configurable_names.contains(name))
    {
        return Cow::Borrowed(style_source);
    }

    let mut source = String::new();
    for (name, value) in variable_overrides {
        source.push('$');
        source.push_str(name);
        source.push_str(": ");
        source.push_str(value);
        source.push_str("; ");
    }
    source.push_str(style_source);
    Cow::Owned(source)
}
