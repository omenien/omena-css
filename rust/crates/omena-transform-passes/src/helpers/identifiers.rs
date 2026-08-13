use omena_syntax::ident::{ClassNameV0, is_css_name_continue};

pub(crate) fn normalize_custom_property_name(name: &str) -> Option<&str> {
    let name = name.trim();
    if name.starts_with("--") && name.len() > 2 {
        return Some(name);
    }
    None
}

pub(crate) fn css_identifier_text_is_plain(text: &str) -> bool {
    text.chars().all(is_css_name_continue)
}

pub(crate) fn css_identifier_names_match(left: &str, right: &str) -> bool {
    ClassNameV0::new(left).same_as(&ClassNameV0::new(right))
}
