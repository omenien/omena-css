pub(crate) fn strip_ascii_prefix_ignore_case<'a>(text: &'a str, prefix: &str) -> Option<&'a str> {
    text.get(..prefix.len())?
        .eq_ignore_ascii_case(prefix)
        .then(|| &text[prefix.len()..])
}

pub(crate) fn starts_with_ascii_case_insensitive(text: &str, prefix: &str) -> bool {
    let text_bytes = text.as_bytes();
    let prefix_bytes = prefix.as_bytes();
    text_bytes.len() >= prefix_bytes.len()
        && text_bytes
            .iter()
            .take(prefix_bytes.len())
            .zip(prefix_bytes)
            .all(|(left, right)| left.eq_ignore_ascii_case(right))
}

pub(crate) fn ascii_css_identifier_end(text: &str, start: usize) -> usize {
    let bytes = text.as_bytes();
    let mut end = start;
    while end < bytes.len() && css_identifier_byte_is_plain(bytes[end]) {
        end += 1;
    }
    end
}

fn css_identifier_byte_is_plain(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-')
}

pub(crate) fn normalize_ascii_whitespace(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}
