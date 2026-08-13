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

#[cfg(test)]
mod band_law_tests {
    use proptest::prelude::*;

    use super::*;

    const TOKEN_ALPHABET: [&str; 4] = ["alpha", "beta", "gamma", "delta"];

    fn landed_word(indices: &[usize]) -> OrderedTokenWordV0 {
        let raw = indices
            .iter()
            .map(|index| TOKEN_ALPHABET[index % TOKEN_ALPHABET.len()])
            .collect::<Vec<_>>()
            .join(" ");
        let DomClassTokenizationV0::Known { word, .. } =
            tokenize_dom_class_attribute_v0(Some(&raw))
        else {
            unreachable!("a supplied string uses the known tokenizer arm");
        };
        word
    }

    fn token_labels(word: &OrderedTokenWordV0) -> Vec<&str> {
        word.tokens()
            .iter()
            .map(CanonicalClassKeyV0::as_str)
            .collect()
    }

    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 256,
            failure_persistence: None,
            ..ProptestConfig::default()
        })]
        #[test]
        fn landed_ordered_word_obeys_band_and_support_laws(
            u in prop::collection::vec(0usize..4, 0..10),
            v in prop::collection::vec(0usize..4, 0..10),
            w in prop::collection::vec(0usize..4, 0..10),
        ) {
            let u = landed_word(&u);
            let v = landed_word(&v);
            let w = landed_word(&w);
            let uv = u.combine_first_occurrence(&v);
            prop_assert_eq!(
                uv.combine_first_occurrence(&w),
                u.combine_first_occurrence(&v.combine_first_occurrence(&w)),
                "associativity"
            );
            prop_assert_eq!(u.combine_first_occurrence(&u), u.clone(), "idempotence");
            prop_assert_eq!(
                uv.combine_first_occurrence(&u),
                uv.clone(),
                "left-regular-band absorption"
            );
            let combined_support = token_support_v0(&uv);
            let mut expected = token_support_v0(&u).may().clone();
            expected.extend(token_support_v0(&v).may().iter().cloned());
            prop_assert_eq!(combined_support.must(), &expected);
            prop_assert_eq!(combined_support.may(), &expected);
        }

        #[test]
        fn whitespace_delimited_raw_join_is_a_token_word_homomorphism(
            left in prop::collection::vec(0usize..4, 0..10),
            right in prop::collection::vec(0usize..4, 0..10),
        ) {
            let left_raw = left
                .iter()
                .map(|index| TOKEN_ALPHABET[index % TOKEN_ALPHABET.len()])
                .collect::<Vec<_>>()
                .join(" ");
            let right_raw = right
                .iter()
                .map(|index| TOKEN_ALPHABET[index % TOKEN_ALPHABET.len()])
                .collect::<Vec<_>>()
                .join(" ");
            let joined = format!("{left_raw} {right_raw}");
            let DomClassTokenizationV0::Known { word: joined, .. } =
                tokenize_dom_class_attribute_v0(Some(&joined))
            else {
                unreachable!("a supplied string uses the known tokenizer arm");
            };
            prop_assert_eq!(joined, landed_word(&left).combine_first_occurrence(&landed_word(&right)));
        }
    }

    #[test]
    fn raw_concatenation_is_a_required_inequality_counterexample() {
        let DomClassTokenizationV0::Known {
            word: raw_concat, ..
        } = tokenize_dom_class_attribute_v0(Some("btn-large"))
        else {
            unreachable!("a supplied string uses the known tokenizer arm");
        };
        let DomClassTokenizationV0::Known { word: left, .. } =
            tokenize_dom_class_attribute_v0(Some("btn-"))
        else {
            unreachable!("a supplied string uses the known tokenizer arm");
        };
        let DomClassTokenizationV0::Known { word: right, .. } =
            tokenize_dom_class_attribute_v0(Some("large"))
        else {
            unreachable!("a supplied string uses the known tokenizer arm");
        };
        assert_eq!(token_labels(&raw_concat), vec!["btn-large"]);
        assert_eq!(
            token_labels(&left.combine_first_occurrence(&right)),
            vec!["btn-", "large"]
        );
        assert_ne!(raw_concat, left.combine_first_occurrence(&right));
    }

    #[test]
    fn product_from_keys_pins_the_last_occurrence_absorption_counterexample() {
        let alpha = ClassNameV0::new("alpha").canonical_key();
        let beta = ClassNameV0::new("beta").canonical_key();
        let u = OrderedTokenWordV0::from_keys([alpha.clone(), beta]);
        let v = OrderedTokenWordV0::from_keys([alpha]);
        let uv = u.combine_first_occurrence(&v);
        assert_eq!(token_labels(&uv), vec!["alpha", "beta"]);
        assert_eq!(
            uv.combine_first_occurrence(&u),
            uv,
            "the product constructor must retain first occurrence for absorption"
        );
    }

    #[test]
    fn generated_pairs_have_collision_and_non_commutativity_witnesses() {
        let generated = (0..TOKEN_ALPHABET.len())
            .flat_map(|left| {
                (0..TOKEN_ALPHABET.len()).map(move |right| landed_word(&[left, right]))
            })
            .collect::<Vec<_>>();
        let mut pair_count = 0usize;
        let mut collision_count = 0usize;
        let mut non_commutative_count = 0usize;
        for left in &generated {
            for right in &generated {
                pair_count += 1;
                if token_support_v0(left)
                    .may()
                    .intersection(token_support_v0(right).may())
                    .next()
                    .is_some()
                {
                    collision_count += 1;
                }
                if left.combine_first_occurrence(right) != right.combine_first_occurrence(left) {
                    non_commutative_count += 1;
                }
            }
        }
        let collision_basis_points = collision_count * 10_000 / pair_count;
        const COLLISION_FLOOR_BASIS_POINTS: usize = 2_500;
        assert!(collision_basis_points >= COLLISION_FLOOR_BASIS_POINTS);
        assert!(non_commutative_count > 0);
        eprintln!(
            "{{\"pairCount\":{pair_count},\"collisionCount\":{collision_count},\"collisionBasisPoints\":{collision_basis_points},\"collisionFloorBasisPoints\":{COLLISION_FLOOR_BASIS_POINTS},\"nonCommutativeCount\":{non_commutative_count}}}"
        );
    }

    #[test]
    fn law_suite_calls_landed_tokenizer_without_a_local_splitter() {
        let source = include_str!("class_tokens.rs");
        let tests = source
            .split("mod band_law_tests")
            .nth(1)
            .unwrap_or_default();
        assert!(tests.matches("tokenize_dom_class_attribute_v0").count() >= 5);
        assert!(!tests.contains("split_whitespace"));
        assert!(!tests.contains("is_whitespace"));
    }
}
