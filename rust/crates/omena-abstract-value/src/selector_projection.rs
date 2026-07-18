use std::collections::BTreeSet;

use crate::automaton::{automaton_matches_string, finite_language_values};
use crate::{
    AbstractClassValueV0, AbstractSelectorProjectionV0, SelectorProjectionCertaintyV0,
    reduce_class_value_product, reduced_class_value_product_matches_string,
};

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
        AbstractClassValueV0::Automaton { automaton, .. } => selector_universe
            .iter()
            .filter(|selector| automaton_matches_string(automaton, selector))
            .cloned()
            .collect(),
        AbstractClassValueV0::Prefix { .. }
        | AbstractClassValueV0::Suffix { .. }
        | AbstractClassValueV0::PrefixSuffix { .. }
        | AbstractClassValueV0::CharInclusion { .. }
        | AbstractClassValueV0::Composite { .. } => {
            let Some(product) = reduce_class_value_product(value) else {
                return Vec::new();
            };
            selector_universe
                .iter()
                .filter(|selector| reduced_class_value_product_matches_string(&product, selector))
                .cloned()
                .collect()
        }
        AbstractClassValueV0::Top { .. } => selector_universe.to_vec(),
    }
}

pub fn derive_selector_projection_certainty(
    value: &AbstractClassValueV0,
    matched_selector_count: usize,
    _selector_universe_count: usize,
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
        AbstractClassValueV0::Automaton { .. } => {
            let value_count = finite_language_values(value).map_or(0, |values| values.len());
            if value_count == 0 || matched_selector_count == 0 {
                SelectorProjectionCertaintyV0::Possible
            } else if matched_selector_count == value_count {
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
            } else {
                SelectorProjectionCertaintyV0::Inferred
            }
        }
        AbstractClassValueV0::Top { .. } => SelectorProjectionCertaintyV0::Possible,
    }
}

pub(crate) fn abstract_value_matches_string(value: &AbstractClassValueV0, candidate: &str) -> bool {
    match value {
        AbstractClassValueV0::Bottom => false,
        AbstractClassValueV0::Exact { value } => value == candidate,
        AbstractClassValueV0::FiniteSet { values } => values.iter().any(|value| value == candidate),
        AbstractClassValueV0::Automaton { automaton, .. } => {
            automaton_matches_string(automaton, candidate)
        }
        AbstractClassValueV0::Prefix { .. }
        | AbstractClassValueV0::Suffix { .. }
        | AbstractClassValueV0::PrefixSuffix { .. }
        | AbstractClassValueV0::CharInclusion { .. }
        | AbstractClassValueV0::Composite { .. } => reduce_class_value_product(value)
            .is_some_and(|product| reduced_class_value_product_matches_string(&product, candidate)),
        AbstractClassValueV0::Top { .. } => true,
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
