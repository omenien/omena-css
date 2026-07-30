use std::borrow::Cow;

/// A CSS class name with authored and decoded spellings.
///
/// Equality is intentionally available only through [`ClassNameV0::same_as`].
/// This keeps raw spelling equality from becoming an accidental join key.
///
/// ```compile_fail,E0369
/// use omena_syntax::ident::ClassNameV0;
///
/// fn raw_structural_equality(left: &ClassNameV0, right: &ClassNameV0) -> bool {
///     left == right
/// }
/// ```
#[derive(Debug, Clone)]
pub struct ClassNameV0 {
    raw: String,
    decoded: String,
}

impl ClassNameV0 {
    pub fn new(raw: impl Into<String>) -> Self {
        let raw = raw.into();
        let decoded = decode_css_identifier_escapes(&raw).into_owned();
        Self { raw, decoded }
    }

    pub fn raw(&self) -> &str {
        &self.raw
    }

    pub fn decoded(&self) -> &str {
        &self.decoded
    }

    pub fn into_raw(self) -> String {
        self.raw
    }

    pub fn same_as(&self, other: &Self) -> bool {
        self.decoded == other.decoded
    }

    pub fn canonical_key(&self) -> CanonicalClassKeyV0 {
        CanonicalClassKeyV0(self.decoded.clone(), CanonicalClassKeySealV0(()))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct CanonicalClassKeySealV0(());

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CanonicalClassKeyV0(String, CanonicalClassKeySealV0);

impl CanonicalClassKeyV0 {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClassSelectorPositionV0 {
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone)]
pub struct ClassSelectorNameV0 {
    pub name: ClassNameV0,
    pub position: ClassSelectorPositionV0,
}

/// Returns whether a decoded character can start a CSS identifier name.
pub fn is_css_name_start(ch: char) -> bool {
    ch == '-' || ch == '_' || ch.is_ascii_alphabetic() || !ch.is_ascii()
}

/// Returns whether a decoded character can continue a CSS identifier name.
pub fn is_css_name_continue(ch: char) -> bool {
    is_css_name_start(ch) || ch.is_ascii_digit()
}

/// Returns whether a character belongs to the deliberately narrow ASCII word
/// used by completion and hover cursor boundaries.
///
/// This is not the CSS identifier grammar. Widening it would change which word
/// an editor request claims under the cursor.
pub fn is_ascii_word_continue(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-')
}

pub fn is_safe_css_identifier(value: &str) -> bool {
    let mut characters = value.chars();
    let Some(first) = characters.next() else {
        return false;
    };
    match first {
        character
            if character == '_' || character.is_ascii_alphabetic() || !character.is_ascii() => {}
        '-' => {
            let Some(second) = characters.next() else {
                return false;
            };
            if !(second == '-'
                || second == '_'
                || second.is_ascii_alphabetic()
                || !second.is_ascii())
            {
                return false;
            }
        }
        _ => return false,
    }
    characters.all(is_css_name_continue)
}

pub fn decode_css_identifier_escapes(text: &str) -> Cow<'_, str> {
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
        if is_css_newline(next) {
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
            output.push(
                codepoint
                    .filter(|value| *value != 0)
                    .and_then(char::from_u32)
                    .unwrap_or(char::REPLACEMENT_CHARACTER),
            );
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

pub fn class_selector_name_end(text: &str, start: usize) -> Option<usize> {
    let first = text.get(start..)?.chars().next()?;
    let mut index = if first == '\\' {
        css_identifier_escape_sequence_end(text, start)?
    } else if is_css_name_start(first) {
        start + first.len_utf8()
    } else {
        return None;
    };

    while index < text.len() {
        let Some(ch) = text[index..].chars().next() else {
            break;
        };
        if ch == '\\' {
            let Some(end) = css_identifier_escape_sequence_end(text, index) else {
                break;
            };
            index = end;
        } else if is_css_name_continue(ch) {
            index += ch.len_utf8();
        } else {
            break;
        }
    }
    Some(index)
}

pub fn class_selector_names(selector: &str) -> Vec<ClassSelectorNameV0> {
    let mut names = Vec::new();
    let mut index = 0usize;
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;
    let mut quote = None;

    while index < selector.len() {
        let Some(ch) = selector[index..].chars().next() else {
            break;
        };
        if ch == '\\' {
            index = css_identifier_escape_sequence_end(selector, index)
                .unwrap_or(index + ch.len_utf8());
            continue;
        }
        if let Some(active_quote) = quote {
            if ch == active_quote {
                quote = None;
            }
            index += ch.len_utf8();
            continue;
        }
        match ch {
            '"' | '\'' => quote = Some(ch),
            '(' => paren_depth += 1,
            ')' => paren_depth = paren_depth.saturating_sub(1),
            '[' => bracket_depth += 1,
            ']' => bracket_depth = bracket_depth.saturating_sub(1),
            '.' if paren_depth == 0 && bracket_depth == 0 => {
                let start = index + ch.len_utf8();
                if let Some(end) = class_selector_name_end(selector, start) {
                    names.push(ClassSelectorNameV0 {
                        name: ClassNameV0::new(&selector[start..end]),
                        position: ClassSelectorPositionV0 { start, end },
                    });
                    index = end;
                    continue;
                }
            }
            _ => {}
        }
        index += ch.len_utf8();
    }

    names
}

fn css_identifier_escape_sequence_end(text: &str, slash_index: usize) -> Option<usize> {
    if text[slash_index..].chars().next()? != '\\' {
        return None;
    }
    let mut index = slash_index + '\\'.len_utf8();
    let next = text[index..].chars().next()?;
    if is_css_newline(next) {
        return None;
    }
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

fn is_css_newline(ch: char) -> bool {
    matches!(ch, '\n' | '\r' | '\u{c}')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_css_escapes_without_changing_plain_names() {
        // These assertions fail on the supplied escape spellings, all of which
        // the public decoder accepts directly from CSS source.
        assert!(matches!(
            decode_css_identifier_escapes("plain"),
            Cow::Borrowed("plain")
        ));
        assert_eq!(decode_css_identifier_escapes(r"a\.b"), "a.b");
        assert_eq!(decode_css_identifier_escapes(r"\31 23"), "123");
        assert_eq!(decode_css_identifier_escapes(r"\0"), "\u{fffd}");
        assert_eq!(decode_css_identifier_escapes("\\\n"), "\\\n");
    }

    #[test]
    fn class_name_identity_is_decoded_but_raw_text_is_preserved() {
        let escaped = ClassNameV0::new(r"a\.b");
        let plain = ClassNameV0::new("a.b");

        // A decoder or key regression makes these source-producible spellings
        // unequal or mutates the raw spelling retained for egress.
        assert!(escaped.same_as(&plain));
        assert_eq!(escaped.raw(), r"a\.b");
        assert_eq!(escaped.canonical_key().as_str(), "a.b");
    }

    #[test]
    fn extracts_top_level_class_names_with_byte_positions() {
        let selector = r#".card .title[data-x="a.b"]:is(.nested).a\.b.\31 23.카드.café"#;
        let names = class_selector_names(selector);
        let raw = names
            .iter()
            .map(|entry| entry.name.raw())
            .collect::<Vec<_>>();

        // The single selector exercises every branch and is itself a valid
        // scanner input, so omitting or splitting any name falsifies the row.
        assert_eq!(
            raw,
            vec!["card", "title", r"a\.b", r"\31 23", "카드", "café"]
        );
        assert_eq!(
            names
                .iter()
                .find(|entry| entry.name.raw() == "café")
                .map(|entry| entry.name.decoded().chars().count()),
            Some(4)
        );
        for entry in names {
            assert_eq!(
                &selector[entry.position.start..entry.position.end],
                entry.name.raw()
            );
        }
    }

    #[test]
    fn distinguishes_css_name_and_ascii_word_boundaries() {
        // Each character and identifier is accepted directly by the relevant
        // public predicate; swapping or merging the two grammars falsifies it.
        assert!(is_css_name_start('카'));
        assert!(is_css_name_continue('é'));
        assert!(!is_ascii_word_continue('카'));
        assert!(is_ascii_word_continue('9'));
        assert!(is_safe_css_identifier("카드"));
        assert!(is_safe_css_identifier("--token"));
        assert!(!is_safe_css_identifier("-9token"));
        assert!(!is_safe_css_identifier("9token"));
    }
}
