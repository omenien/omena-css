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

pub(crate) fn css_identifier_escape_sequence_end(text: &str, slash_index: usize) -> Option<usize> {
    let slash = text[slash_index..].chars().next()?;
    if slash != '\\' {
        return None;
    }
    let mut index = slash_index + slash.len_utf8();
    let next = text[index..].chars().next()?;
    if !next.is_ascii_hexdigit() {
        return Some(index + next.len_utf8());
    }

    let mut digit_count = 0usize;
    while index < text.len() && digit_count < 6 {
        let Some(candidate) = text[index..].chars().next() else {
            break;
        };
        if !candidate.is_ascii_hexdigit() {
            break;
        }
        index += candidate.len_utf8();
        digit_count += 1;
    }
    if let Some(terminator) = text[index..].chars().next()
        && terminator.is_ascii_whitespace()
    {
        index += terminator.len_utf8();
    }
    Some(index)
}
