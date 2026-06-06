use std::collections::BTreeSet;

use crate::{
    AbstractPropertyValueCandidateV0, AbstractPropertyValueNarrowingV0, AbstractPropertyValueV0,
};

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
    );

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
        value,
    }
}

fn abstract_property_value_from_matched_candidates(
    property_name: &str,
    requested_pseudo_state: Option<&str>,
    matched: &[&AbstractPropertyValueCandidateV0],
) -> AbstractPropertyValueV0 {
    let mut values = matched
        .iter()
        .map(|candidate| candidate.value.trim())
        .filter(|value| !value.is_empty())
        .collect::<BTreeSet<_>>();
    if values.is_empty() {
        return AbstractPropertyValueV0::Bottom {
            property_name: property_name.to_string(),
        };
    }

    if values.len() == 1 {
        let value = values.pop_first().unwrap_or_default().to_string();
        if let Some(custom_property_name) = referenced_custom_property_name(value.as_str()) {
            return AbstractPropertyValueV0::CustomPropertyReference {
                property_name: property_name.to_string(),
                custom_property_name,
                pseudo_state: requested_pseudo_state.map(str::to_string),
            };
        }
        return AbstractPropertyValueV0::Exact {
            property_name: property_name.to_string(),
            value,
            pseudo_state: requested_pseudo_state.map(str::to_string),
        };
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
        values: values.into_iter().map(str::to_string).collect(),
        pseudo_states,
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
