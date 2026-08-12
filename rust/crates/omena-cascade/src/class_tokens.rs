use std::collections::BTreeSet;

use omena_syntax::ident::{CanonicalClassKeyV0, ClassNameV0};
use serde::{Serialize, Serializer, ser::SerializeStruct};

/// A duplicate-free class-token word in first-occurrence order.
///
/// Elements are sealed canonical class keys rather than authored spellings.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct OrderedTokenWordV0 {
    tokens: Vec<CanonicalClassKeyV0>,
}

impl OrderedTokenWordV0 {
    pub fn from_keys(keys: impl IntoIterator<Item = CanonicalClassKeyV0>) -> Self {
        let mut seen = BTreeSet::new();
        let tokens = keys
            .into_iter()
            .filter(|key| seen.insert(key.clone()))
            .collect();
        Self { tokens }
    }

    pub fn tokens(&self) -> &[CanonicalClassKeyV0] {
        &self.tokens
    }

    pub fn combine_first_occurrence(&self, right: &Self) -> Self {
        Self::from_keys(self.tokens.iter().chain(&right.tokens).cloned())
    }
}

impl Serialize for OrderedTokenWordV0 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.tokens
            .iter()
            .map(CanonicalClassKeyV0::as_str)
            .collect::<Vec<_>>()
            .serialize(serializer)
    }
}

/// Must/may token support with the invariant `must` is a subset of `may`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TokenSupportV0 {
    must: BTreeSet<CanonicalClassKeyV0>,
    may: BTreeSet<CanonicalClassKeyV0>,
}

impl TokenSupportV0 {
    pub fn new(
        must: impl IntoIterator<Item = CanonicalClassKeyV0>,
        may: impl IntoIterator<Item = CanonicalClassKeyV0>,
    ) -> Option<Self> {
        let must = must.into_iter().collect::<BTreeSet<_>>();
        let may = may.into_iter().collect::<BTreeSet<_>>();
        must.is_subset(&may).then_some(Self { must, may })
    }

    pub fn must(&self) -> &BTreeSet<CanonicalClassKeyV0> {
        &self.must
    }

    pub fn may(&self) -> &BTreeSet<CanonicalClassKeyV0> {
        &self.may
    }
}

impl Serialize for TokenSupportV0 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let must = self
            .must
            .iter()
            .map(CanonicalClassKeyV0::as_str)
            .collect::<Vec<_>>();
        let may = self
            .may
            .iter()
            .map(CanonicalClassKeyV0::as_str)
            .collect::<Vec<_>>();
        let mut state = serializer.serialize_struct("TokenSupportV0", 2)?;
        state.serialize_field("must", &must)?;
        state.serialize_field("may", &may)?;
        state.end()
    }
}

pub fn token_support_v0(word: &OrderedTokenWordV0) -> TokenSupportV0 {
    let keys = word.tokens.iter().cloned().collect::<BTreeSet<_>>();
    TokenSupportV0 {
        must: keys.clone(),
        may: keys,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DomClassTokenSpanV0 {
    pub start: usize,
    pub end: usize,
    pub word_index: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum DomClassTokenizationUnknownCauseV0 {
    InputUnavailable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(
    tag = "outcome",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum DomClassTokenizationV0 {
    Known {
        word: OrderedTokenWordV0,
        token_spans: Vec<DomClassTokenSpanV0>,
    },
    Unknown {
        cause: DomClassTokenizationUnknownCauseV0,
    },
}

pub const fn is_dom_class_ascii_whitespace_v0(character: char) -> bool {
    matches!(
        character,
        '\u{0009}' | '\u{000a}' | '\u{000c}' | '\u{000d}' | '\u{0020}'
    )
}

/// Tokenizes a DOM class attribute using only HTML ASCII whitespace.
///
/// Splitting precedes CSS identifier decoding. Canonically equal duplicate
/// spellings retain only their first span and first-occurrence position.
pub fn tokenize_dom_class_attribute_v0(value: Option<&str>) -> DomClassTokenizationV0 {
    let Some(value) = value else {
        return DomClassTokenizationV0::Unknown {
            cause: DomClassTokenizationUnknownCauseV0::InputUnavailable,
        };
    };

    let mut keys = Vec::new();
    let mut token_spans = Vec::new();
    let mut seen = BTreeSet::new();
    let mut start = None;

    let mut finish_token = |start: usize, end: usize| {
        let key = ClassNameV0::new(&value[start..end]).canonical_key();
        if seen.insert(key.clone()) {
            token_spans.push(DomClassTokenSpanV0 {
                start,
                end,
                word_index: keys.len(),
            });
            keys.push(key);
        }
    };

    for (offset, character) in value.char_indices() {
        if is_dom_class_ascii_whitespace_v0(character) {
            if let Some(token_start) = start.take() {
                finish_token(token_start, offset);
            }
        } else if start.is_none() {
            start = Some(offset);
        }
    }
    if let Some(token_start) = start {
        finish_token(token_start, value.len());
    }

    DomClassTokenizationV0::Known {
        word: OrderedTokenWordV0 { tokens: keys },
        token_spans,
    }
}
