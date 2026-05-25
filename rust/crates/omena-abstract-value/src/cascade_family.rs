use crate::AbstractPropertyValueV0;
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};

/// Research-staged cascade-family substrate for M6 positioning.
///
/// This is intentionally framing-neutral: it records context-indexed value
/// families and restriction morphisms without claiming a sheaf/cosheaf theorem
/// or committing paper-stage terminology.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CascadeValueFamilyV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub framing: &'static str,
    pub claim_level: &'static str,
    pub property_name: String,
    pub supported_readings: Vec<&'static str>,
    pub context_value_count: usize,
    pub restriction_map_count: usize,
    pub property_consistent: bool,
    pub dangling_restriction_count: usize,
    pub theorem_claimed: bool,
    pub members: Vec<CascadeValueFamilyMemberV0>,
    pub restriction_maps: Vec<CascadeRestrictionMapV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CascadeContextV0 {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>,
    pub selectors: Vec<String>,
    pub conditions: Vec<String>,
    pub layers: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CascadeValueFamilyMemberV0 {
    pub context: CascadeContextV0,
    pub value: AbstractPropertyValueV0,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CascadeRestrictionMapV0 {
    pub parent_context_id: String,
    pub child_context_id: String,
    pub morphism: CascadeMorphismV0,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CascadeMorphismV0 {
    pub kind: &'static str,
    pub direction: &'static str,
    pub preserves_property_name: bool,
    pub evidence: Vec<&'static str>,
}

pub fn summarize_cascade_value_family_v0(
    property_name: impl Into<String>,
    members: Vec<CascadeValueFamilyMemberV0>,
    restriction_maps: Vec<CascadeRestrictionMapV0>,
) -> CascadeValueFamilyV0 {
    let property_name = property_name.into();
    let mut members = members;
    members.sort_by(|left, right| left.context.id.cmp(&right.context.id));
    members.dedup_by(|left, right| left.context.id == right.context.id);

    let mut restriction_maps = restriction_maps;
    restriction_maps.sort();
    restriction_maps.dedup();

    let context_ids = members
        .iter()
        .map(|member| member.context.id.as_str())
        .collect::<BTreeSet<_>>();
    let property_consistent = members
        .iter()
        .all(|member| property_value_name(&member.value) == property_name);
    let dangling_restriction_count = restriction_maps
        .iter()
        .filter(|restriction| {
            !context_ids.contains(restriction.parent_context_id.as_str())
                || !context_ids.contains(restriction.child_context_id.as_str())
        })
        .count();
    let restriction_map_count = restriction_maps.len();

    CascadeValueFamilyV0 {
        schema_version: "0",
        product: "omena-abstract-value.cascade-value-family",
        framing: "framingNeutralCascadeFamily",
        claim_level: "researchStagedSubstrate",
        property_name,
        supported_readings: vec!["presheafCompatible", "cosheafCompatible"],
        context_value_count: members.len(),
        restriction_map_count,
        property_consistent,
        dangling_restriction_count,
        theorem_claimed: false,
        members,
        restriction_maps,
    }
}

pub fn derive_cascade_restriction_maps_v0(
    members: &[CascadeValueFamilyMemberV0],
) -> Vec<CascadeRestrictionMapV0> {
    let context_ids = members
        .iter()
        .map(|member| member.context.id.as_str())
        .collect::<BTreeSet<_>>();
    let mut maps = members
        .iter()
        .filter_map(|member| {
            let parent_id = member.context.parent_id.as_deref()?;
            context_ids
                .contains(parent_id)
                .then(|| CascadeRestrictionMapV0 {
                    parent_context_id: parent_id.to_string(),
                    child_context_id: member.context.id.clone(),
                    morphism: cascade_context_refinement_morphism_v0(),
                })
        })
        .collect::<Vec<_>>();
    maps.sort();
    maps.dedup();
    maps
}

pub fn cascade_context_refinement_morphism_v0() -> CascadeMorphismV0 {
    CascadeMorphismV0 {
        kind: "contextRefinement",
        direction: "parentToChildRestriction",
        preserves_property_name: true,
        evidence: vec![
            "contextIndexedValueFamily",
            "parentChildCascadeContext",
            "noSheafTheoremClaim",
        ],
    }
}

pub fn cascade_value_for_context<'a>(
    family: &'a CascadeValueFamilyV0,
    context_id: &str,
) -> Option<&'a AbstractPropertyValueV0> {
    family
        .members
        .iter()
        .find(|member| member.context.id == context_id)
        .map(|member| &member.value)
}

pub fn cascade_family_context_values(
    family: &CascadeValueFamilyV0,
) -> BTreeMap<String, AbstractPropertyValueV0> {
    family
        .members
        .iter()
        .map(|member| (member.context.id.clone(), member.value.clone()))
        .collect()
}

fn property_value_name(value: &AbstractPropertyValueV0) -> &str {
    match value {
        AbstractPropertyValueV0::Bottom { property_name }
        | AbstractPropertyValueV0::Exact { property_name, .. }
        | AbstractPropertyValueV0::FiniteSet { property_name, .. }
        | AbstractPropertyValueV0::CustomPropertyReference { property_name, .. }
        | AbstractPropertyValueV0::Top { property_name } => property_name,
    }
}
