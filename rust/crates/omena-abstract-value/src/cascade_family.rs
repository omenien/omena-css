use crate::{ABSTRACT_VALUE_CASCADE_FAMILY_CLAIM_LEVEL_V0, AbstractPropertyValueV0};
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CascadeStalkEvaluationV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub claim_level: &'static str,
    pub property_name: String,
    pub requested_context_id: String,
    pub requested_context_exists: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolved_context_id: Option<String>,
    pub restriction_path: Vec<String>,
    pub used_restriction_map_count: usize,
    pub bounded_by_context_count: usize,
    pub bounded_resolution_ready: bool,
    pub theorem_claimed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<AbstractPropertyValueV0>,
    pub resolved: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blocked_reason: Option<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CascadeRestrictionCycleSummaryV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub claim_level: &'static str,
    pub cycle_detection_model: &'static str,
    pub context_count: usize,
    pub restriction_map_count: usize,
    pub cycle_count: usize,
    pub cycles: Vec<CascadeRestrictionCycleV0>,
    pub theorem_claimed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CascadeRestrictionCycleV0 {
    pub path: Vec<String>,
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
        claim_level: ABSTRACT_VALUE_CASCADE_FAMILY_CLAIM_LEVEL_V0,
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

pub fn evaluate_cascade_stalk_v0(
    family: &CascadeValueFamilyV0,
    context_id: &str,
) -> CascadeStalkEvaluationV0 {
    let values = cascade_family_context_values(family);
    let requested_context_exists = values.contains_key(context_id);
    let parent_by_child = family
        .restriction_maps
        .iter()
        .map(|restriction| {
            (
                restriction.child_context_id.clone(),
                restriction.parent_context_id.clone(),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let mut restriction_path = Vec::new();
    let mut current = context_id.to_string();
    let bound = family.context_value_count.max(1);

    for _ in 0..=bound {
        restriction_path.push(current.clone());
        if let Some(value) = values.get(&current)
            && !property_value_is_bottom(value)
        {
            return CascadeStalkEvaluationV0 {
                schema_version: "0",
                product: "omena-abstract-value.cascade-stalk-evaluation",
                claim_level: "fixtureWitnessBoundedCascadeStalkEvaluation",
                property_name: family.property_name.clone(),
                requested_context_id: context_id.to_string(),
                requested_context_exists,
                resolved_context_id: Some(current),
                used_restriction_map_count: restriction_path.len().saturating_sub(1),
                bounded_by_context_count: bound,
                bounded_resolution_ready: true,
                theorem_claimed: false,
                value: Some(value.clone()),
                resolved: true,
                blocked_reason: None,
                restriction_path,
            };
        }
        let Some(parent) = parent_by_child.get(&current) else {
            break;
        };
        current = parent.clone();
    }

    CascadeStalkEvaluationV0 {
        schema_version: "0",
        product: "omena-abstract-value.cascade-stalk-evaluation",
        claim_level: "fixtureWitnessBoundedCascadeStalkEvaluation",
        property_name: family.property_name.clone(),
        requested_context_id: context_id.to_string(),
        requested_context_exists,
        resolved_context_id: None,
        restriction_path,
        used_restriction_map_count: 0,
        bounded_by_context_count: bound,
        bounded_resolution_ready: true,
        theorem_claimed: false,
        value: None,
        resolved: false,
        blocked_reason: Some("no non-bottom value found along bounded restriction path"),
    }
}

pub fn summarize_cascade_restriction_cycles_v0(
    family: &CascadeValueFamilyV0,
) -> CascadeRestrictionCycleSummaryV0 {
    let context_ids = family
        .members
        .iter()
        .map(|member| member.context.id.clone())
        .collect::<BTreeSet<_>>();
    let mut adjacency = BTreeMap::<String, Vec<String>>::new();
    for restriction in &family.restriction_maps {
        adjacency
            .entry(restriction.parent_context_id.clone())
            .or_default()
            .push(restriction.child_context_id.clone());
    }
    for targets in adjacency.values_mut() {
        targets.sort();
        targets.dedup();
    }

    let mut cycles = BTreeSet::<CascadeRestrictionCycleV0>::new();
    for start in &context_ids {
        collect_restriction_cycles_from(
            start,
            start,
            &adjacency,
            &mut Vec::new(),
            &mut cycles,
            family.context_value_count.max(1),
        );
    }
    let cycles = cycles.into_iter().collect::<Vec<_>>();
    CascadeRestrictionCycleSummaryV0 {
        schema_version: "0",
        product: "omena-abstract-value.cascade-restriction-cycle-summary",
        claim_level: "fixtureWitnessBoundedRestrictionCycleDetection",
        cycle_detection_model: "boundedRestrictionCycleWitnessNotCohomologyTheorem",
        context_count: family.context_value_count,
        restriction_map_count: family.restriction_map_count,
        cycle_count: cycles.len(),
        cycles,
        theorem_claimed: false,
    }
}

fn collect_restriction_cycles_from(
    start: &str,
    current: &str,
    adjacency: &BTreeMap<String, Vec<String>>,
    path: &mut Vec<String>,
    cycles: &mut BTreeSet<CascadeRestrictionCycleV0>,
    bound: usize,
) {
    if path.len() > bound {
        return;
    }
    path.push(current.to_string());
    if let Some(children) = adjacency.get(current) {
        for child in children {
            if child == start && path.len() > 1 {
                let mut cycle = path.clone();
                cycle.push(start.to_string());
                cycles.insert(CascadeRestrictionCycleV0 {
                    path: canonical_restriction_cycle(cycle),
                });
            } else if !path.contains(child) {
                collect_restriction_cycles_from(start, child, adjacency, path, cycles, bound);
            }
        }
    }
    path.pop();
}

fn canonical_restriction_cycle(mut cycle: Vec<String>) -> Vec<String> {
    if cycle.len() <= 2 {
        return cycle;
    }
    cycle.pop();
    let Some((start_index, _)) = cycle
        .iter()
        .enumerate()
        .min_by(|left, right| left.1.cmp(right.1))
    else {
        return cycle;
    };
    let mut canonical = (0..cycle.len())
        .map(|offset| cycle[(start_index + offset) % cycle.len()].clone())
        .collect::<Vec<_>>();
    canonical.push(canonical[0].clone());
    canonical
}

fn property_value_is_bottom(value: &AbstractPropertyValueV0) -> bool {
    matches!(value, AbstractPropertyValueV0::Bottom { .. })
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
