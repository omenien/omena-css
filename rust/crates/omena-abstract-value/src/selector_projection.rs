use std::collections::BTreeSet;

use crate::{AbstractClassValueV0, AbstractSelectorProjectionV0, SelectorProjectionCertaintyV0};

pub fn project_abstract_value_selectors(
    value: &AbstractClassValueV0,
    selector_universe: &[String],
) -> AbstractSelectorProjectionV0 {
    let selector_names = resolve_abstract_value_selectors(value, selector_universe);
    let certainty =
        derive_selector_projection_certainty(value, selector_names.len(), selector_universe.len());

    AbstractSelectorProjectionV0 {
        selector_names,
        certainty,
    }
}

pub fn resolve_abstract_value_selectors(
    value: &AbstractClassValueV0,
    selector_universe: &[String],
) -> Vec<String> {
    match value {
        AbstractClassValueV0::Bottom => Vec::new(),
        AbstractClassValueV0::Exact { value } => find_selectors(selector_universe, value),
        AbstractClassValueV0::FiniteSet { values } => unique_selector_names(
            values
                .iter()
                .flat_map(|value| find_selectors(selector_universe, value)),
        ),
        AbstractClassValueV0::Prefix { prefix, .. } => selector_universe
            .iter()
            .filter(|selector| selector.starts_with(prefix))
            .cloned()
            .collect(),
        AbstractClassValueV0::Suffix { suffix, .. } => selector_universe
            .iter()
            .filter(|selector| selector.ends_with(suffix))
            .cloned()
            .collect(),
        AbstractClassValueV0::PrefixSuffix { prefix, suffix, .. } => selector_universe
            .iter()
            .filter(|selector| selector.starts_with(prefix) && selector.ends_with(suffix))
            .cloned()
            .collect(),
        AbstractClassValueV0::CharInclusion {
            must_chars,
            may_chars,
            may_include_other_chars,
            ..
        } => selector_universe
            .iter()
            .filter(|selector| {
                matches_char_constraints(selector, must_chars, may_chars, *may_include_other_chars)
            })
            .cloned()
            .collect(),
        AbstractClassValueV0::Composite {
            prefix,
            suffix,
            min_length,
            must_chars,
            may_chars,
            may_include_other_chars,
            ..
        } => selector_universe
            .iter()
            .filter(|selector| {
                min_length.is_none_or(|min_length| selector.len() >= min_length)
                    && prefix
                        .as_ref()
                        .is_none_or(|prefix| selector.starts_with(prefix))
                    && suffix
                        .as_ref()
                        .is_none_or(|suffix| selector.ends_with(suffix))
                    && matches_char_constraints(
                        selector,
                        must_chars,
                        may_chars,
                        *may_include_other_chars,
                    )
            })
            .cloned()
            .collect(),
        AbstractClassValueV0::Top => selector_universe.to_vec(),
    }
}

pub fn derive_selector_projection_certainty(
    value: &AbstractClassValueV0,
    matched_selector_count: usize,
    selector_universe_count: usize,
) -> SelectorProjectionCertaintyV0 {
    match value {
        AbstractClassValueV0::Bottom => SelectorProjectionCertaintyV0::Possible,
        AbstractClassValueV0::Exact { .. } => {
            if matched_selector_count == 1 {
                SelectorProjectionCertaintyV0::Exact
            } else {
                SelectorProjectionCertaintyV0::Possible
            }
        }
        AbstractClassValueV0::FiniteSet { values } => {
            if values.is_empty() || matched_selector_count == 0 {
                SelectorProjectionCertaintyV0::Possible
            } else if matched_selector_count == values.len() {
                SelectorProjectionCertaintyV0::Exact
            } else {
                SelectorProjectionCertaintyV0::Inferred
            }
        }
        AbstractClassValueV0::Prefix { .. }
        | AbstractClassValueV0::Suffix { .. }
        | AbstractClassValueV0::PrefixSuffix { .. }
        | AbstractClassValueV0::CharInclusion { .. }
        | AbstractClassValueV0::Composite { .. } => {
            if matched_selector_count == 0 {
                SelectorProjectionCertaintyV0::Possible
            } else if matched_selector_count == selector_universe_count {
                SelectorProjectionCertaintyV0::Exact
            } else {
                SelectorProjectionCertaintyV0::Inferred
            }
        }
        AbstractClassValueV0::Top => SelectorProjectionCertaintyV0::Possible,
    }
}

pub(crate) fn abstract_value_matches_string(value: &AbstractClassValueV0, candidate: &str) -> bool {
    match value {
        AbstractClassValueV0::Bottom => false,
        AbstractClassValueV0::Exact { value } => value == candidate,
        AbstractClassValueV0::FiniteSet { values } => values.iter().any(|value| value == candidate),
        AbstractClassValueV0::Prefix { prefix, .. } => candidate.starts_with(prefix),
        AbstractClassValueV0::Suffix { suffix, .. } => candidate.ends_with(suffix),
        AbstractClassValueV0::PrefixSuffix {
            prefix,
            suffix,
            min_length,
            ..
        } => {
            candidate.len() >= *min_length
                && candidate.starts_with(prefix)
                && candidate.ends_with(suffix)
        }
        AbstractClassValueV0::CharInclusion {
            must_chars,
            may_chars,
            may_include_other_chars,
            ..
        } => matches_char_constraints(candidate, must_chars, may_chars, *may_include_other_chars),
        AbstractClassValueV0::Composite {
            prefix,
            suffix,
            min_length,
            must_chars,
            may_chars,
            may_include_other_chars,
            ..
        } => {
            min_length.is_none_or(|min_length| candidate.len() >= min_length)
                && prefix
                    .as_ref()
                    .is_none_or(|prefix| candidate.starts_with(prefix))
                && suffix
                    .as_ref()
                    .is_none_or(|suffix| candidate.ends_with(suffix))
                && matches_char_constraints(
                    candidate,
                    must_chars,
                    may_chars,
                    *may_include_other_chars,
                )
        }
        AbstractClassValueV0::Top => true,
    }
}

fn find_selectors(selector_universe: &[String], value: &str) -> Vec<String> {
    selector_universe
        .iter()
        .filter(|selector| selector.as_str() == value)
        .cloned()
        .collect()
}

fn unique_selector_names<I>(values: I) -> Vec<String>
where
    I: IntoIterator<Item = String>,
{
    values
        .into_iter()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn matches_char_constraints(
    value: &str,
    must_chars: &str,
    may_chars: &str,
    may_include_other_chars: bool,
) -> bool {
    let value_chars = value.chars().collect::<BTreeSet<_>>();
    let must_chars = must_chars.chars().collect::<BTreeSet<_>>();
    if !must_chars.iter().all(|char| value_chars.contains(char)) {
        return false;
    }
    if may_include_other_chars {
        return true;
    }
    let may_chars = may_chars.chars().collect::<BTreeSet<_>>();
    value_chars.iter().all(|char| may_chars.contains(char))
}
