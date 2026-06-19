use std::collections::{BTreeMap, BTreeSet};

use omena_value_lattice::{canonicalize_css_value, css_values_canonically_equal};

use crate::{
    AbstractCssValueV0, AbstractPropertyValueCandidateV0, AbstractPropertyValueNarrowingV0,
    AbstractPropertyValueV0,
};

pub fn abstract_css_value_from_text(value: &str) -> AbstractCssValueV0 {
    let value = value.trim();
    if value.is_empty() {
        return AbstractCssValueV0::Bottom;
    }
    match canonical_css_value_text(value) {
        Some(value) => AbstractCssValueV0::Exact { value },
        None => AbstractCssValueV0::Raw {
            value: value.to_string(),
        },
    }
}

pub fn canonical_css_value_text(value: &str) -> Option<String> {
    canonicalize_css_value(value).map(|value| value.serialized)
}

pub fn abstract_css_values_canonically_equal(left: &str, right: &str) -> bool {
    css_values_canonically_equal(left, right)
}

pub fn join_abstract_css_values(
    left: &AbstractCssValueV0,
    right: &AbstractCssValueV0,
) -> AbstractCssValueV0 {
    match (left, right) {
        (AbstractCssValueV0::Bottom, value) | (value, AbstractCssValueV0::Bottom) => value.clone(),
        (AbstractCssValueV0::Top, _) | (_, AbstractCssValueV0::Top) => AbstractCssValueV0::Top,
        (AbstractCssValueV0::Exact { value: left }, AbstractCssValueV0::Exact { value: right })
            if left == right =>
        {
            left_css_value(left)
        }
        (AbstractCssValueV0::Raw { value: left }, AbstractCssValueV0::Raw { value: right })
            if left == right =>
        {
            AbstractCssValueV0::Raw {
                value: left.clone(),
            }
        }
        _ => finite_css_value_set([left, right]),
    }
}

pub fn narrow_abstract_property_value_for_pseudo_state(
    property_name: &str,
    requested_pseudo_state: Option<&str>,
    candidates: &[AbstractPropertyValueCandidateV0],
) -> AbstractPropertyValueNarrowingV0 {
    narrow_abstract_property_value_for_cascade_branch(
        property_name,
        requested_pseudo_state,
        &[],
        None,
        None,
        false,
        candidates,
    )
}

pub fn narrow_abstract_property_value_for_cascade_branch(
    property_name: &str,
    requested_pseudo_state: Option<&str>,
    requested_condition_context: &[String],
    requested_layer_name: Option<&str>,
    requested_layer_order: Option<i32>,
    exact_layer: bool,
    candidates: &[AbstractPropertyValueCandidateV0],
) -> AbstractPropertyValueNarrowingV0 {
    let matched = candidates
        .iter()
        .filter(|candidate| candidate.property_name == property_name)
        .filter(|candidate| {
            candidate.pseudo_state.as_deref().is_none()
                || candidate.pseudo_state.as_deref() == requested_pseudo_state
        })
        .filter(|candidate| candidate.condition_context == requested_condition_context)
        .filter(|candidate| {
            !exact_layer
                || (candidate.layer_name.as_deref() == requested_layer_name
                    && candidate.layer_order == requested_layer_order)
        })
        .collect::<Vec<_>>();
    let value = abstract_property_value_from_matched_candidates(
        property_name,
        requested_pseudo_state,
        &matched,
        exact_layer,
    );
    let (display_value, display_values) =
        display_property_values_from_matched_candidates(&value, &matched, exact_layer);

    AbstractPropertyValueNarrowingV0 {
        schema_version: "0",
        product: "omena-abstract-value.property-value-narrowing",
        stylesheet_scope: "singleStylesheet",
        property_name: property_name.to_string(),
        requested_pseudo_state: requested_pseudo_state.map(str::to_string),
        requested_condition_context: requested_condition_context.to_vec(),
        requested_layer_name: requested_layer_name.map(str::to_string),
        requested_layer_order,
        requested_layer_scope: if exact_layer {
            "exactLayer"
        } else {
            "anyLayer"
        },
        candidate_count: candidates.len(),
        matched_candidate_count: matched.len(),
        display_value,
        display_values,
        value,
    }
}

fn abstract_property_value_from_matched_candidates(
    property_name: &str,
    requested_pseudo_state: Option<&str>,
    matched: &[&AbstractPropertyValueCandidateV0],
    exact_layer: bool,
) -> AbstractPropertyValueV0 {
    if exact_layer
        && matched.len() > 1
        && matched
            .iter()
            .all(|candidate| candidate.same_selector_ordering && candidate.source_order.is_some())
    {
        let winner = matched
            .iter()
            .max_by_key(|candidate| (candidate.important, candidate.source_order.unwrap_or(0)))
            .copied();
        if let Some(winner) = winner {
            return abstract_property_value_from_single_candidate(
                property_name,
                requested_pseudo_state,
                winner.value.trim(),
            );
        }
    }

    let mut values = matched
        .iter()
        .map(|candidate| canonical_property_value_text(candidate.value.trim()))
        .filter(|value| !value.is_empty())
        .collect::<BTreeSet<_>>();
    if values.is_empty() {
        return AbstractPropertyValueV0::Bottom {
            property_name: property_name.to_string(),
        };
    }

    if values.len() == 1 {
        let value = values.pop_first().unwrap_or_default();
        return abstract_property_value_from_single_candidate(
            property_name,
            requested_pseudo_state,
            value.as_str(),
        );
    }

    let pseudo_states = matched
        .iter()
        .filter_map(|candidate| candidate.pseudo_state.as_deref())
        .map(str::to_string)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();

    AbstractPropertyValueV0::FiniteSet {
        property_name: property_name.to_string(),
        values: values.into_iter().collect(),
        pseudo_states,
    }
}

fn abstract_property_value_from_single_candidate(
    property_name: &str,
    requested_pseudo_state: Option<&str>,
    value: &str,
) -> AbstractPropertyValueV0 {
    if let Some(custom_property_name) = referenced_custom_property_name(value) {
        return AbstractPropertyValueV0::CustomPropertyReference {
            property_name: property_name.to_string(),
            custom_property_name,
            pseudo_state: requested_pseudo_state.map(str::to_string),
        };
    }
    let value = canonical_property_value_text(value);
    AbstractPropertyValueV0::Exact {
        property_name: property_name.to_string(),
        value,
        pseudo_state: requested_pseudo_state.map(str::to_string),
    }
}

fn display_property_values_from_matched_candidates(
    value: &AbstractPropertyValueV0,
    matched: &[&AbstractPropertyValueCandidateV0],
    exact_layer: bool,
) -> (Option<String>, Vec<String>) {
    match value {
        AbstractPropertyValueV0::Bottom { .. } | AbstractPropertyValueV0::Top { .. } => {
            (None, Vec::new())
        }
        AbstractPropertyValueV0::Exact { value, .. } => (
            display_value_for_exact_canonical_value(value, matched, exact_layer),
            Vec::new(),
        ),
        AbstractPropertyValueV0::CustomPropertyReference {
            custom_property_name,
            ..
        } => (
            display_value_for_custom_property_reference(custom_property_name, matched),
            Vec::new(),
        ),
        AbstractPropertyValueV0::FiniteSet { values, .. } => {
            let display_values = display_values_for_canonical_values(values, matched);
            (None, display_values)
        }
    }
}

fn display_value_for_exact_canonical_value(
    canonical_value: &str,
    matched: &[&AbstractPropertyValueCandidateV0],
    exact_layer: bool,
) -> Option<String> {
    if exact_layer
        && matched.len() > 1
        && matched
            .iter()
            .all(|candidate| candidate.same_selector_ordering && candidate.source_order.is_some())
    {
        return matched
            .iter()
            .max_by_key(|candidate| (candidate.important, candidate.source_order.unwrap_or(0)))
            .and_then(|candidate| display_candidate_value(candidate));
    }

    matched
        .iter()
        .find(|candidate| canonical_property_value_text(candidate.value.trim()) == canonical_value)
        .and_then(|candidate| display_candidate_value(candidate))
        .or_else(|| Some(canonical_value.to_string()))
}

fn display_value_for_custom_property_reference(
    custom_property_name: &str,
    matched: &[&AbstractPropertyValueCandidateV0],
) -> Option<String> {
    matched
        .iter()
        .find(|candidate| {
            referenced_custom_property_name(candidate.value.trim()).as_deref()
                == Some(custom_property_name)
        })
        .and_then(|candidate| display_candidate_value(candidate))
        .or_else(|| Some(format!("var({custom_property_name})")))
}

fn display_values_for_canonical_values(
    canonical_values: &[String],
    matched: &[&AbstractPropertyValueCandidateV0],
) -> Vec<String> {
    let mut display_by_canonical_value = BTreeMap::new();
    for candidate in matched {
        let canonical_value = canonical_property_value_text(candidate.value.trim());
        if canonical_values.contains(&canonical_value)
            && let Some(display_value) = display_candidate_value(candidate)
        {
            display_by_canonical_value
                .entry(canonical_value)
                .or_insert(display_value);
        }
    }

    canonical_values
        .iter()
        .map(|canonical_value| {
            display_by_canonical_value
                .get(canonical_value)
                .cloned()
                .unwrap_or_else(|| canonical_value.clone())
        })
        .collect()
}

fn display_candidate_value(candidate: &AbstractPropertyValueCandidateV0) -> Option<String> {
    let value = candidate.value.trim();
    (!value.is_empty()).then(|| value.to_string())
}

fn canonical_property_value_text(value: &str) -> String {
    canonical_css_value_text(value).unwrap_or_else(|| value.trim().to_string())
}

fn left_css_value(value: &str) -> AbstractCssValueV0 {
    AbstractCssValueV0::Exact {
        value: value.to_string(),
    }
}

fn finite_css_value_set<const N: usize>(values: [&AbstractCssValueV0; N]) -> AbstractCssValueV0 {
    let values = values
        .into_iter()
        .flat_map(flatten_css_value_set)
        .collect::<BTreeSet<_>>();
    if values.is_empty() {
        AbstractCssValueV0::Bottom
    } else if values.len() == 1 {
        AbstractCssValueV0::Exact {
            value: values.into_iter().next().unwrap_or_default(),
        }
    } else {
        AbstractCssValueV0::FiniteSet {
            values: values.into_iter().collect(),
        }
    }
}

fn flatten_css_value_set(value: &AbstractCssValueV0) -> Vec<String> {
    match value {
        AbstractCssValueV0::Bottom => Vec::new(),
        AbstractCssValueV0::Exact { value } | AbstractCssValueV0::Raw { value } => {
            vec![value.clone()]
        }
        AbstractCssValueV0::FiniteSet { values } => values.clone(),
        AbstractCssValueV0::Top => vec!["<top>".to_string()],
    }
}

fn referenced_custom_property_name(value: &str) -> Option<String> {
    let value = value.trim();
    let name_start = value.find("var(")? + "var(".len();
    let tail = value.get(name_start..)?.trim_start();
    if !tail.starts_with("--") {
        return None;
    }
    let name_end = tail
        .find(|character: char| character == ')' || character == ',' || character.is_whitespace())
        .unwrap_or(tail.len());
    let name = tail.get(..name_end)?.trim();
    (!name.is_empty()).then(|| name.to_string())
}
