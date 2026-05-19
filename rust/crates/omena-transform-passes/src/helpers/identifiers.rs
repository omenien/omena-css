use std::borrow::Cow;

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

pub(crate) fn css_identifier_names_match(left: &str, right: &str) -> bool {
    left == right || decode_css_identifier_escapes(left) == decode_css_identifier_escapes(right)
}

pub(crate) fn decode_css_identifier_escapes(text: &str) -> Cow<'_, str> {
    if !text.contains('\\') {
        return Cow::Borrowed(text);
    }

    let mut output = String::with_capacity(text.len());
    let mut index = 0usize;
    while index < text.len() {
        let Some(ch) = text[index..].chars().next() else {
            break;
        };
        if ch != '\\' {
            output.push(ch);
            index += ch.len_utf8();
            continue;
        }

        let escape_start = index;
        index += ch.len_utf8();
        let Some(next) = text[index..].chars().next() else {
            output.push('\\');
            break;
        };
        if next == '\n' || next == '\r' || next == '\u{c}' {
            output.push_str(&text[escape_start..index + next.len_utf8()]);
            index += next.len_utf8();
            continue;
        }
        if next.is_ascii_hexdigit() {
            let hex_start = index;
            let mut hex_end = index;
            let mut digit_count = 0usize;
            while hex_end < text.len() && digit_count < 6 {
                let Some(candidate) = text[hex_end..].chars().next() else {
                    break;
                };
                if !candidate.is_ascii_hexdigit() {
                    break;
                }
                hex_end += candidate.len_utf8();
                digit_count += 1;
            }
            let codepoint = u32::from_str_radix(&text[hex_start..hex_end], 16).ok();
            if let Some(decoded) = codepoint.and_then(char::from_u32) {
                output.push(decoded);
            }
            index = hex_end;
            if let Some(terminator) = text[index..].chars().next()
                && terminator.is_ascii_whitespace()
            {
                index += terminator.len_utf8();
            }
            continue;
        }

        output.push(next);
        index += next.len_utf8();
    }

    Cow::Owned(output)
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
