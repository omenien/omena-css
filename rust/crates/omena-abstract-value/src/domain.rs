use std::collections::BTreeSet;

use crate::{
    AbstractClassValueProvenanceV0, AbstractClassValueV0, AbstractValueDomainSummaryV0,
    CompositeClassValueInputV0, MAX_FINITE_CLASS_VALUES,
};

pub fn summarize_omena_abstract_value_domain() -> AbstractValueDomainSummaryV0 {
    AbstractValueDomainSummaryV0 {
        schema_version: "0",
        product: "omena-abstract-value.domain",
        domain_kinds: vec![
            "bottom",
            "exact",
            "finiteSet",
            "prefix",
            "suffix",
            "prefixSuffix",
            "charInclusion",
            "composite",
            "top",
        ],
        max_finite_class_values: MAX_FINITE_CLASS_VALUES,
        selector_projection_certainties: vec!["exact", "inferred", "possible"],
        provenance_tree_ready: true,
        provenance_tree_scopes: vec![
            "literal",
            "finiteSet",
            "constraint",
            "finiteSetWidening",
            "reducedProduct",
            "flowResult",
        ],
    }
}

pub fn bottom_class_value() -> AbstractClassValueV0 {
    AbstractClassValueV0::Bottom
}

pub fn top_class_value() -> AbstractClassValueV0 {
    AbstractClassValueV0::Top
}

pub fn exact_class_value(value: impl Into<String>) -> AbstractClassValueV0 {
    AbstractClassValueV0::Exact {
        value: value.into(),
    }
}

pub fn finite_set_class_value<I, S>(values: I) -> AbstractClassValueV0
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let normalized = normalize_values(values);
    match normalized.len() {
        0 => bottom_class_value(),
        1 => exact_class_value(normalized[0].clone()),
        2..=MAX_FINITE_CLASS_VALUES => AbstractClassValueV0::FiniteSet { values: normalized },
        _ => widen_large_finite_set(&normalized),
    }
}

pub fn prefix_class_value(
    prefix: impl Into<String>,
    provenance: Option<AbstractClassValueProvenanceV0>,
) -> AbstractClassValueV0 {
    AbstractClassValueV0::Prefix {
        prefix: prefix.into(),
        provenance,
    }
}

pub fn suffix_class_value(
    suffix: impl Into<String>,
    provenance: Option<AbstractClassValueProvenanceV0>,
) -> AbstractClassValueV0 {
    AbstractClassValueV0::Suffix {
        suffix: suffix.into(),
        provenance,
    }
}

pub fn prefix_suffix_class_value(
    prefix: impl Into<String>,
    suffix: impl Into<String>,
    min_length: Option<usize>,
    provenance: Option<AbstractClassValueProvenanceV0>,
) -> AbstractClassValueV0 {
    let prefix = prefix.into();
    let suffix = suffix.into();
    if prefix.is_empty() && suffix.is_empty() {
        return top_class_value();
    }
    if prefix.is_empty() {
        return suffix_class_value(suffix, provenance);
    }
    if suffix.is_empty() {
        return prefix_class_value(prefix, provenance);
    }

    AbstractClassValueV0::PrefixSuffix {
        min_length: min_length
            .unwrap_or(prefix.len() + suffix.len())
            .max(prefix.len() + suffix.len()),
        prefix,
        suffix,
        provenance,
    }
}

pub fn char_inclusion_class_value(
    must_chars: impl Into<String>,
    may_chars: impl Into<String>,
    provenance: Option<AbstractClassValueProvenanceV0>,
    may_include_other_chars: bool,
) -> AbstractClassValueV0 {
    let must_chars = normalize_char_set(must_chars.into());
    let may_chars = normalize_char_set(format!("{}{}", may_chars.into(), must_chars));

    if may_include_other_chars && must_chars.is_empty() {
        return top_class_value();
    }
    if !may_include_other_chars && may_chars.is_empty() {
        return bottom_class_value();
    }

    AbstractClassValueV0::CharInclusion {
        must_chars,
        may_chars,
        may_include_other_chars,
        provenance,
    }
}

pub fn composite_class_value(input: CompositeClassValueInputV0) -> AbstractClassValueV0 {
    let prefix = input.prefix.unwrap_or_default();
    let suffix = input.suffix.unwrap_or_default();
    let edge_chars = char_set_for_string(format!("{prefix}{suffix}"));
    let must_chars = normalize_char_set(format!("{}{}", input.must_chars, edge_chars));
    let may_chars = normalize_char_set(format!("{}{}", input.may_chars, must_chars));
    let has_char_info =
        !must_chars.is_empty() || (!input.may_include_other_chars && !may_chars.is_empty());

    if !has_char_info {
        return prefix_suffix_class_value(prefix, suffix, input.min_length, input.provenance);
    }
    if prefix.is_empty() && suffix.is_empty() {
        return char_inclusion_class_value(
            must_chars,
            may_chars,
            input.provenance,
            input.may_include_other_chars,
        );
    }

    let guaranteed_distinct_char_count = must_chars.chars().count();
    let edge_min_length = prefix.len() + suffix.len();
    let min_length = input
        .min_length
        .map(|value| value.max(edge_min_length))
        .or(Some(edge_min_length))
        .map(|value| value.max(guaranteed_distinct_char_count));

    AbstractClassValueV0::Composite {
        prefix: (!prefix.is_empty()).then_some(prefix),
        suffix: (!suffix.is_empty()).then_some(suffix),
        min_length,
        must_chars,
        may_chars,
        may_include_other_chars: input.may_include_other_chars,
        provenance: input.provenance,
    }
}

pub fn enumerate_finite_class_values(value: &AbstractClassValueV0) -> Option<Vec<String>> {
    match value {
        AbstractClassValueV0::Bottom => Some(Vec::new()),
        AbstractClassValueV0::Exact { value } => Some(vec![value.clone()]),
        AbstractClassValueV0::FiniteSet { values } => Some(values.clone()),
        _ => None,
    }
}

pub fn abstract_class_value_kind(value: &AbstractClassValueV0) -> &'static str {
    match value {
        AbstractClassValueV0::Bottom => "bottom",
        AbstractClassValueV0::Exact { .. } => "exact",
        AbstractClassValueV0::FiniteSet { .. } => "finiteSet",
        AbstractClassValueV0::Prefix { .. } => "prefix",
        AbstractClassValueV0::Suffix { .. } => "suffix",
        AbstractClassValueV0::PrefixSuffix { .. } => "prefixSuffix",
        AbstractClassValueV0::CharInclusion { .. } => "charInclusion",
        AbstractClassValueV0::Composite { .. } => "composite",
        AbstractClassValueV0::Top => "top",
    }
}

pub(crate) fn normalize_char_set(chars: impl AsRef<str>) -> String {
    chars
        .as_ref()
        .chars()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

pub(crate) fn union_char_sets(left: &str, right: &str) -> String {
    normalize_char_set(format!("{left}{right}"))
}

pub(crate) fn intersect_char_sets(left: &str, right: &str) -> String {
    let right_set = right.chars().collect::<BTreeSet<_>>();
    left.chars()
        .filter(|char| right_set.contains(char))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

pub(crate) fn char_set_for_string(value: impl AsRef<str>) -> String {
    normalize_char_set(value)
}

pub(crate) fn meaningful_longest_common_prefix(values: &[String]) -> String {
    let prefix = longest_common_prefix(values);
    if prefix.is_empty() || !is_meaningful_class_prefix(&prefix, values) {
        return String::new();
    }
    prefix
}

pub(crate) fn meaningful_longest_common_suffix(values: &[String]) -> String {
    let suffix = longest_common_suffix(values);
    if suffix.is_empty() || !is_meaningful_class_suffix(&suffix, values) {
        return String::new();
    }
    suffix
}

pub(crate) fn char_set_is_subset(left: &str, right: &str) -> bool {
    let right = right.chars().collect::<BTreeSet<_>>();
    left.chars().all(|char| right.contains(&char))
}

fn widen_large_finite_set(values: &[String]) -> AbstractClassValueV0 {
    let prefix = meaningful_longest_common_prefix(values);
    let suffix = meaningful_longest_common_suffix(values);
    let (must_chars, may_chars) = char_inclusion_from_finite_values(values);

    if !prefix.is_empty() || !suffix.is_empty() {
        return composite_class_value(CompositeClassValueInputV0 {
            prefix: (!prefix.is_empty()).then_some(prefix),
            suffix: (!suffix.is_empty()).then_some(suffix),
            min_length: values.iter().map(String::len).min(),
            must_chars,
            may_chars,
            may_include_other_chars: false,
            provenance: Some(AbstractClassValueProvenanceV0::FiniteSetWideningComposite),
        });
    }

    char_inclusion_class_value(
        must_chars,
        may_chars,
        Some(AbstractClassValueProvenanceV0::FiniteSetWideningChars),
        false,
    )
}

fn normalize_values<I, S>(values: I) -> Vec<String>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    values
        .into_iter()
        .map(Into::into)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn char_inclusion_from_finite_values(values: &[String]) -> (String, String) {
    let mut sets = values.iter().map(char_set_for_string);
    let Some(first) = sets.next() else {
        return (String::new(), String::new());
    };

    sets.fold((first.clone(), first), |(must_chars, may_chars), next| {
        (
            intersect_char_sets(&must_chars, &next),
            union_char_sets(&may_chars, &next),
        )
    })
}

fn longest_common_prefix(values: &[String]) -> String {
    let Some(first) = values.first() else {
        return String::new();
    };
    let mut prefix = first.clone();

    for value in values.iter().skip(1) {
        let mut match_length = 0usize;
        for (left, right) in prefix.chars().zip(value.chars()) {
            if left != right {
                break;
            }
            match_length += left.len_utf8();
        }
        prefix.truncate(match_length);
        if prefix.is_empty() {
            break;
        }
    }

    prefix
}

fn longest_common_suffix(values: &[String]) -> String {
    let reversed = values
        .iter()
        .map(|value| value.chars().rev().collect::<String>())
        .collect::<Vec<_>>();
    longest_common_prefix(&reversed)
        .chars()
        .rev()
        .collect::<String>()
}

fn is_meaningful_class_prefix(prefix: &str, values: &[String]) -> bool {
    if prefix.is_empty() {
        return false;
    }
    if ends_at_class_boundary(prefix) {
        return true;
    }
    values.iter().all(|value| {
        value.len() == prefix.len()
            || value[prefix.len()..]
                .chars()
                .next()
                .is_some_and(is_class_boundary_char)
    })
}

fn is_meaningful_class_suffix(suffix: &str, values: &[String]) -> bool {
    if suffix.is_empty() {
        return false;
    }
    if starts_at_class_boundary(suffix) {
        return true;
    }
    values.iter().all(|value| {
        if value.len() == suffix.len() {
            return true;
        }
        value[..value.len() - suffix.len()]
            .chars()
            .next_back()
            .is_some_and(is_class_boundary_char)
    })
}

fn ends_at_class_boundary(value: &str) -> bool {
    value
        .chars()
        .next_back()
        .is_some_and(is_class_boundary_char)
}

fn starts_at_class_boundary(value: &str) -> bool {
    value.chars().next().is_some_and(is_class_boundary_char)
}

fn is_class_boundary_char(char: char) -> bool {
    char == '-' || char == '_'
}
