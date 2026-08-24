//! Canonical identity for CSS cascade-layer paths.

use crate::ident::decode_css_identifier_escapes;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct CanonicalLayerIdentifierKeySealV0(());

/// One escape-decoded layer identifier. The private seal prevents callers from
/// treating authored spelling as the identity key.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CanonicalLayerIdentifierKeyV0(String, CanonicalLayerIdentifierKeySealV0);

impl CanonicalLayerIdentifierKeyV0 {
    pub fn from_authored(authored: &str) -> Option<Self> {
        let decoded = decode_css_identifier_escapes(authored).into_owned();
        (!decoded.is_empty()).then_some(Self(decoded, CanonicalLayerIdentifierKeySealV0(())))
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl serde::Serialize for CanonicalLayerIdentifierKeyV0 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

/// A parser-issued path of decoded CSS identifiers.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LayerPathV0 {
    segments: Vec<CanonicalLayerIdentifierKeyV0>,
}

impl LayerPathV0 {
    pub fn from_authored_segments<'a>(segments: impl IntoIterator<Item = &'a str>) -> Option<Self> {
        let segments = segments
            .into_iter()
            .map(CanonicalLayerIdentifierKeyV0::from_authored)
            .collect::<Option<Vec<_>>>()?;
        (!segments.is_empty()).then_some(Self { segments })
    }

    pub fn segments(&self) -> &[CanonicalLayerIdentifierKeyV0] {
        self.segments.as_slice()
    }

    pub fn nesting_depth(&self) -> usize {
        self.segments.len().saturating_sub(1)
    }

    pub fn canonical_name(&self) -> String {
        self.segments
            .iter()
            .map(|segment| canonical_css_identifier(segment.as_str()))
            .collect::<Vec<_>>()
            .join(".")
    }

    pub fn prefix(&self, segment_count: usize) -> Option<Self> {
        (segment_count > 0 && segment_count <= self.segments.len()).then(|| Self {
            segments: self.segments[..segment_count].to_vec(),
        })
    }

    pub fn parent(&self) -> Option<Self> {
        self.prefix(self.segments.len().checked_sub(1)?)
    }

    pub fn local_name(&self) -> &str {
        self.segments
            .last()
            .map(CanonicalLayerIdentifierKeyV0::as_str)
            .unwrap_or_default()
    }

    pub fn joined(&self, local: &Self) -> Self {
        let mut segments = self.segments.clone();
        segments.extend(local.segments.iter().cloned());
        Self { segments }
    }
}

fn canonical_css_identifier(identifier: &str) -> String {
    let chars = identifier.chars().collect::<Vec<_>>();
    let mut canonical = String::with_capacity(identifier.len());
    for (index, character) in chars.iter().copied().enumerate() {
        let ascii_name_character = character.is_ascii_alphabetic()
            || character == '_'
            || character == '-'
            || (character.is_ascii_digit()
                && index > 0
                && !(index == 1 && chars.first() == Some(&'-')));
        if ascii_name_character || !character.is_ascii() {
            canonical.push(character);
        } else {
            canonical.push('\\');
            canonical.push_str(format!("{:06x}", u32::from(character)).as_str());
        }
    }
    canonical
}
