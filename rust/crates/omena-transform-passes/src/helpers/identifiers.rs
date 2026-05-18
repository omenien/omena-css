pub(crate) fn normalize_custom_property_name(name: &str) -> Option<&str> {
    let name = name.trim();
    if name.starts_with("--") && name.len() > 2 {
        return Some(name);
    }
    None
}

pub(crate) fn css_identifier_text_is_plain(text: &str) -> bool {
    text.chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_'))
}

pub(crate) fn is_css_ident_start(ch: char) -> bool {
    ch == '-' || ch == '_' || ch.is_ascii_alphabetic()
}

pub(crate) fn is_css_ident_continue(ch: char) -> bool {
    is_css_ident_start(ch) || ch.is_ascii_digit()
}
