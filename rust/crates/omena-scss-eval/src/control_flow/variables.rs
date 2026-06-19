use std::collections::BTreeMap;

use omena_abstract_value::AbstractCssValueV0;

pub(super) fn variable_names_in_text(text: &str) -> Vec<String> {
    let mut names = variable_names_in_text_preserving_order(text);
    names.sort();
    names.dedup();
    names
}

pub(super) fn variable_names_in_text_preserving_order(text: &str) -> Vec<String> {
    let mut names = Vec::new();
    let mut index = 0usize;
    while index < text.len() {
        let Some(ch) = text[index..].chars().next() else {
            break;
        };
        if ch != '$' {
            index += ch.len_utf8();
            continue;
        }
        let name_start = index + ch.len_utf8();
        let name_end = variable_name_end(text, name_start);
        if name_end > name_start
            && let Some(name) = text.get(index..name_end)
            && !names.iter().any(|candidate| candidate == name)
        {
            names.push(name.to_string());
        }
        index = name_end.max(index + ch.len_utf8());
    }
    names
}

pub(super) fn canonical_scss_variable_name(name: &str) -> String {
    let trimmed = name.trim();
    let bare = trimmed.strip_prefix('$').unwrap_or(trimmed);
    format!("${}", bare.replace('_', "-"))
}

pub(super) fn insert_static_scss_binding(
    bindings: &mut BTreeMap<String, AbstractCssValueV0>,
    name: &str,
    value: AbstractCssValueV0,
) {
    bindings.insert(canonical_scss_variable_name(name), value);
}

pub(super) fn static_scss_binding_value<'a>(
    bindings: &'a BTreeMap<String, AbstractCssValueV0>,
    name: &str,
) -> Option<&'a AbstractCssValueV0> {
    bindings.get(canonical_scss_variable_name(name).as_str())
}

pub(super) fn variable_name_end(text: &str, mut index: usize) -> usize {
    while index < text.len() {
        let Some(ch) = text[index..].chars().next() else {
            break;
        };
        if !(ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-')) {
            break;
        }
        index += ch.len_utf8();
    }
    index
}
