use std::borrow::Borrow;
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
    decoded: Option<String>,
}

impl ClassNameV0 {
    pub fn new(raw: impl Into<String>) -> Self {
        let raw = raw.into();
        let decoded = match decode_css_identifier_escapes(&raw) {
            Cow::Borrowed(_) => None,
            Cow::Owned(decoded) => Some(decoded),
        };
        Self { raw, decoded }
    }

    pub fn raw(&self) -> &str {
        &self.raw
    }

    pub fn decoded(&self) -> &str {
        self.decoded.as_deref().unwrap_or(&self.raw)
    }

    pub fn into_raw(self) -> String {
        self.raw
    }

    pub fn same_as(&self, other: &Self) -> bool {
        self.decoded() == other.decoded()
    }

    pub fn canonical_key(self) -> CanonicalClassKeyV0 {
        let decoded = self.decoded.unwrap_or(self.raw);
        CanonicalClassKeyV0(decoded, CanonicalClassKeySealV0(()))
    }

    fn from_plain(raw: &str) -> Self {
        Self {
            raw: raw.to_owned(),
            decoded: None,
        }
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

/// Whether a declaration name belongs to the standard-property or custom-property
/// identity domain.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PropertyNameKindV0 {
    Standard,
    Custom,
}

/// A CSS property name with an authored spelling and one sealed canonical identity.
///
/// Structural equality is deliberately unavailable. Callers compare property
/// identity through [`PropertyNameV0::same_as`] or carry the sealed key returned by
/// [`PropertyNameV0::canonical_key`].
///
/// ```compile_fail,E0369
/// use omena_syntax::ident::{PropertyNameKindV0, PropertyNameV0};
///
/// fn raw_structural_equality(left: &PropertyNameV0, right: &PropertyNameV0) -> bool {
///     left == right
/// }
///
/// let _ = PropertyNameV0::new("color", PropertyNameKindV0::Standard);
/// ```
#[derive(Debug, Clone)]
pub enum PropertyNameV0 {
    Standard {
        authored: String,
        decoded: String,
        canonical: CanonicalStandardPropertyNameV0,
    },
    Custom {
        authored: String,
        decoded: String,
        canonical: CanonicalCustomPropertyNameV0,
    },
}

impl PropertyNameV0 {
    /// Classifies a property name after CSS-escape decoding, then applies the
    /// corresponding canonical identity rules.
    pub fn from_authored(authored: impl Into<String>) -> Self {
        let authored = authored.into();
        let decoded = decode_css_identifier_escapes(authored.trim());
        let kind = if decoded.starts_with("--") {
            PropertyNameKindV0::Custom
        } else {
            PropertyNameKindV0::Standard
        };
        Self::new(authored, kind)
    }

    pub fn new(authored: impl Into<String>, kind: PropertyNameKindV0) -> Self {
        let authored = authored.into();
        let authored = authored.trim().to_string();
        let decoded = decode_css_identifier_escapes(&authored).into_owned();
        match kind {
            PropertyNameKindV0::Standard => Self::Standard {
                canonical: CanonicalStandardPropertyNameV0(
                    decoded.to_ascii_lowercase(),
                    CanonicalStandardPropertyNameSealV0(()),
                ),
                authored,
                decoded,
            },
            PropertyNameKindV0::Custom => Self::Custom {
                canonical: CanonicalCustomPropertyNameV0(
                    decoded.clone(),
                    CanonicalCustomPropertyNameSealV0(()),
                ),
                authored,
                decoded,
            },
        }
    }

    pub fn standard(authored: impl Into<String>) -> Self {
        Self::new(authored, PropertyNameKindV0::Standard)
    }

    pub fn custom(authored: impl Into<String>) -> Self {
        Self::new(authored, PropertyNameKindV0::Custom)
    }

    pub fn kind(&self) -> PropertyNameKindV0 {
        match self {
            Self::Standard { .. } => PropertyNameKindV0::Standard,
            Self::Custom { .. } => PropertyNameKindV0::Custom,
        }
    }

    pub fn authored(&self) -> &str {
        match self {
            Self::Standard { authored, .. } | Self::Custom { authored, .. } => authored,
        }
    }

    pub fn decoded(&self) -> &str {
        match self {
            Self::Standard { decoded, .. } | Self::Custom { decoded, .. } => decoded,
        }
    }

    pub fn canonical_name(&self) -> &str {
        match self {
            Self::Standard { canonical, .. } => canonical.as_str(),
            Self::Custom { canonical, .. } => canonical.as_str(),
        }
    }

    pub fn same_as(&self, other: &Self) -> bool {
        match (self, other) {
            (
                Self::Standard {
                    canonical: left, ..
                },
                Self::Standard {
                    canonical: right, ..
                },
            ) => left == right,
            (
                Self::Custom {
                    canonical: left, ..
                },
                Self::Custom {
                    canonical: right, ..
                },
            ) => left == right,
            _ => false,
        }
    }

    pub fn canonical_key(&self) -> CanonicalPropertyKeyV0 {
        match self {
            Self::Standard { canonical, .. } => CanonicalPropertyKeyV0::Standard(canonical.clone()),
            Self::Custom { canonical, .. } => CanonicalPropertyKeyV0::Custom(canonical.clone()),
        }
    }

    pub fn as_custom_key(&self) -> Option<CanonicalCustomPropertyNameV0> {
        match self {
            Self::Custom { canonical, .. } => Some(canonical.clone()),
            Self::Standard { .. } => None,
        }
    }

    pub fn canonical_custom_key(authored: impl Into<String>) -> CanonicalCustomPropertyNameV0 {
        match Self::custom(authored) {
            Self::Custom { canonical, .. } => canonical,
            Self::Standard { .. } => unreachable!("custom constructor returned a standard name"),
        }
    }

    pub fn canonical_standard_key(authored: impl Into<String>) -> CanonicalStandardPropertyNameV0 {
        match Self::standard(authored) {
            Self::Standard { canonical, .. } => canonical,
            Self::Custom { .. } => unreachable!("standard constructor returned a custom name"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct CanonicalStandardPropertyNameSealV0(());

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CanonicalStandardPropertyNameV0(String, CanonicalStandardPropertyNameSealV0);

impl CanonicalStandardPropertyNameV0 {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Borrow<str> for CanonicalStandardPropertyNameV0 {
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl serde::Serialize for CanonicalStandardPropertyNameV0 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct CanonicalCustomPropertyNameSealV0(());

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CanonicalCustomPropertyNameV0(String, CanonicalCustomPropertyNameSealV0);

impl CanonicalCustomPropertyNameV0 {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Borrow<str> for CanonicalCustomPropertyNameV0 {
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl serde::Serialize for CanonicalCustomPropertyNameV0 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CanonicalPropertyKeyV0 {
    Standard(CanonicalStandardPropertyNameV0),
    Custom(CanonicalCustomPropertyNameV0),
}

impl CanonicalPropertyKeyV0 {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Standard(name) => name.as_str(),
            Self::Custom(name) => name.as_str(),
        }
    }

    pub fn kind(&self) -> PropertyNameKindV0 {
        match self {
            Self::Standard(_) => PropertyNameKindV0::Standard,
            Self::Custom(_) => PropertyNameKindV0::Custom,
        }
    }

    pub fn as_custom(&self) -> Option<&CanonicalCustomPropertyNameV0> {
        match self {
            Self::Custom(name) => Some(name),
            Self::Standard(_) => None,
        }
    }

    pub fn as_standard(&self) -> Option<&CanonicalStandardPropertyNameV0> {
        match self {
            Self::Standard(name) => Some(name),
            Self::Custom(_) => None,
        }
    }
}

impl serde::Serialize for CanonicalPropertyKeyV0 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

/// Compares two authored property spellings through the sole property-name
/// identity authority.
pub fn property_names_same(left: &str, right: &str) -> bool {
    PropertyNameV0::from_authored(left).same_as(&PropertyNameV0::from_authored(right))
}

/// Classifies an authored property spelling through the sole property-name
/// authority, including escaped leading hyphens.
pub fn is_custom_property_name(authored: &str) -> bool {
    PropertyNameV0::from_authored(authored).kind() == PropertyNameKindV0::Custom
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
            output.push(char::REPLACEMENT_CHARACTER);
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
    if let Some(names) = ascii_class_selector_names(selector) {
        return names;
    }
    general_class_selector_names(selector)
}

fn ascii_class_selector_names(selector: &str) -> Option<Vec<ClassSelectorNameV0>> {
    let bytes = selector.as_bytes();
    let mut names = Vec::new();
    let mut index = 0usize;
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;
    let mut quote = None;

    while index < bytes.len() {
        let byte = bytes[index];
        if !byte.is_ascii() || byte == b'\\' {
            return None;
        }
        if let Some(active_quote) = quote {
            if byte == active_quote {
                quote = None;
            }
            index += 1;
            continue;
        }
        match byte {
            b'"' | b'\'' => quote = Some(byte),
            b'(' => paren_depth += 1,
            b')' => paren_depth = paren_depth.saturating_sub(1),
            b'[' => bracket_depth += 1,
            b']' => bracket_depth = bracket_depth.saturating_sub(1),
            b'.' if paren_depth == 0 && bracket_depth == 0 => {
                let start = index + 1;
                let Some(first) = bytes.get(start).copied() else {
                    index += 1;
                    continue;
                };
                if !ascii_css_name_start(first) {
                    index += 1;
                    continue;
                }
                let mut end = start + 1;
                while end < bytes.len() && ascii_css_name_continue(bytes[end]) {
                    end += 1;
                }
                names.push(ClassSelectorNameV0 {
                    name: ClassNameV0::from_plain(&selector[start..end]),
                    position: ClassSelectorPositionV0 { start, end },
                });
                index = end;
                continue;
            }
            _ => {}
        }
        index += 1;
    }
    Some(names)
}

fn ascii_css_name_start(byte: u8) -> bool {
    matches!(byte, b'-' | b'_') || byte.is_ascii_alphabetic()
}

fn ascii_css_name_continue(byte: u8) -> bool {
    ascii_css_name_start(byte) || byte.is_ascii_digit()
}

fn general_class_selector_names(selector: &str) -> Vec<ClassSelectorNameV0> {
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

/// Returns the byte immediately after a valid CSS identifier escape.
///
/// A newline or end-of-input after the reverse solidus is not a valid escape.
pub fn css_identifier_escape_sequence_end(text: &str, slash_index: usize) -> Option<usize> {
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
        assert_eq!(decode_css_identifier_escapes("\\"), "\u{fffd}");
        assert_eq!(decode_css_identifier_escapes("\\\n"), "\\\n");
    }

    #[test]
    fn identifier_escape_boundaries_reject_newline_and_end_of_input() {
        assert_eq!(css_identifier_escape_sequence_end(r"\31 23", 0), Some(4));
        assert_eq!(css_identifier_escape_sequence_end(r"\:", 0), Some(2));
        assert_eq!(css_identifier_escape_sequence_end("\\\n", 0), None);
        assert_eq!(css_identifier_escape_sequence_end("\\", 0), None);
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
    fn property_name_identity_preserves_custom_case_and_standard_case_folding() {
        let custom_upper = PropertyNameV0::custom("--FOO");
        let custom_lower = PropertyNameV0::custom("--foo");
        let standard_upper = PropertyNameV0::standard("COLOR");
        let standard_lower = PropertyNameV0::standard("color");

        assert!(!custom_upper.same_as(&custom_lower));
        assert!(standard_upper.same_as(&standard_lower));
        assert_eq!(custom_upper.authored(), "--FOO");
        assert_eq!(custom_upper.canonical_name(), "--FOO");
        assert_eq!(standard_upper.authored(), "COLOR");
        assert_eq!(standard_upper.canonical_name(), "color");
    }

    #[test]
    fn custom_property_identity_decodes_escapes_without_destroying_authored_spelling() {
        let escaped = PropertyNameV0::custom(r"--f\6f o");
        let plain = PropertyNameV0::custom("--foo");

        assert!(escaped.same_as(&plain));
        assert_eq!(escaped.authored(), r"--f\6f o");
        assert_eq!(escaped.decoded(), "--foo");
        assert_eq!(escaped.canonical_name(), "--foo");
        assert_eq!(
            escaped
                .as_custom_key()
                .as_ref()
                .map(CanonicalCustomPropertyNameV0::as_str),
            Some("--foo")
        );
    }

    #[test]
    fn property_name_kind_is_classified_after_escape_decoding() {
        let property = PropertyNameV0::from_authored(r"\2d\2d FOO");

        assert_eq!(property.kind(), PropertyNameKindV0::Custom);
        assert_eq!(property.authored(), r"\2d\2d FOO");
        assert_eq!(property.canonical_name(), "--FOO");
        assert!(is_custom_property_name(r"\2d\2d FOO"));
        assert!(!is_custom_property_name("COLOR"));
    }

    #[test]
    fn ascii_class_scanner_matches_the_general_authority() {
        let selector = r#".card .title[data-x="a.b"]:is(.nested).plain"#;
        let summarize = |names: Vec<ClassSelectorNameV0>| {
            names
                .into_iter()
                .map(|entry| {
                    (
                        entry.name.into_raw(),
                        entry.position.start,
                        entry.position.end,
                    )
                })
                .collect::<Vec<_>>()
        };

        let fast = ascii_class_selector_names(selector);
        assert!(fast.is_some(), "fixture must stay on the fast path");
        if let Some(fast) = fast {
            assert_eq!(
                summarize(fast),
                summarize(general_class_selector_names(selector))
            );
        }
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
