pub(super) fn static_scss_public_module_variable_name(name: &str) -> Option<String> {
    let bare_name = name.strip_prefix('$')?;
    if bare_name.starts_with('-') || bare_name.starts_with('_') || bare_name.is_empty() {
        return None;
    }
    Some(canonical_static_scss_variable_name(bare_name))
}

pub fn canonical_static_scss_variable_name(name: &str) -> String {
    name.trim()
        .strip_prefix('$')
        .unwrap_or_else(|| name.trim())
        .replace('_', "-")
}

pub(super) fn canonical_static_scss_function_name(name: &str) -> String {
    name.trim().replace('_', "-")
}

pub(super) fn canonical_static_less_mixin_name(name: &str) -> String {
    name.trim().to_string()
}

pub fn static_scss_variable_names_equal(left: &str, right: &str) -> bool {
    canonical_static_scss_variable_name(left) == canonical_static_scss_variable_name(right)
}
