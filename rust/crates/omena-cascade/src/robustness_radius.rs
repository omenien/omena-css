//! Computed guarded-winner robustness over a finite, declared edit alphabet.

use std::collections::{BTreeMap, BTreeSet};

use serde::Serialize;

use crate::{
    CascadeKey, CascadeLevel, FirstWitnessErrorV0, FirstWitnessManagerConfigV0,
    FirstWitnessManagerV0, GuardedCascadeCandidateV0, GuardedCascadeFragmentV0,
    GuardedCascadeSpecificityExactnessV0, LayerOrdinal, Specificity,
    at_rule_nesting_order_for_fragment_v0, build_guarded_cascade_winner_v0,
    evaluate_guarded_cascade_winner_v0, normalized_layer_rank,
};

pub const GUARDED_CASCADE_ROBUSTNESS_PRODUCT_V0: &str =
    "omena-cascade.guarded-winner-robustness-radius";
pub const GUARDED_CASCADE_ROBUSTNESS_CALIBRATION_STAGE_V0: &str = "schemaOnlyUncalibrated";
pub const GUARDED_CASCADE_ROBUSTNESS_MIN_PLUS_DUPLICATION_REASON_V0: &str = "the decision diagram lives in omena-cascade while the reusable tropical semiring lives downstream in omena-abstract-value, so this finite min-plus fold avoids a dependency cycle";
pub const MAX_GUARDED_CASCADE_PERTURBATIONS_V0: usize = 20;

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum GuardedCascadePerturbationKindV0 {
    AddClass,
    RemoveClass,
    ToggleImportant,
    IncreaseSpecificity,
    MoveLayer,
    MoveSourceOrder,
    ToggleCondition,
}

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum GuardedCascadePerturbationV0 {
    AddClass { declaration_id: u32 },
    RemoveClass { declaration_id: u32 },
    ToggleImportant { declaration_id: u32 },
    IncreaseSpecificity { declaration_id: u32 },
    MoveLayer { declaration_id: u32 },
    MoveSourceOrder { declaration_id: u32 },
    ToggleCondition { atom: String },
}

impl GuardedCascadePerturbationV0 {
    pub const fn kind(&self) -> GuardedCascadePerturbationKindV0 {
        match self {
            Self::AddClass { .. } => GuardedCascadePerturbationKindV0::AddClass,
            Self::RemoveClass { .. } => GuardedCascadePerturbationKindV0::RemoveClass,
            Self::ToggleImportant { .. } => GuardedCascadePerturbationKindV0::ToggleImportant,
            Self::IncreaseSpecificity { .. } => {
                GuardedCascadePerturbationKindV0::IncreaseSpecificity
            }
            Self::MoveLayer { .. } => GuardedCascadePerturbationKindV0::MoveLayer,
            Self::MoveSourceOrder { .. } => GuardedCascadePerturbationKindV0::MoveSourceOrder,
            Self::ToggleCondition { .. } => GuardedCascadePerturbationKindV0::ToggleCondition,
        }
    }
}

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GuardedCascadePerturbationCostModelV0 {
    pub add_class: u32,
    pub remove_class: u32,
    pub toggle_important: u32,
    pub increase_specificity: u32,
    pub move_layer: u32,
    pub move_source_order: u32,
    pub toggle_condition: u32,
    pub calibration_stage: &'static str,
    pub public_safety_claim_ready: bool,
}

impl GuardedCascadePerturbationCostModelV0 {
    pub const fn unit_cost_v0() -> Self {
        Self {
            add_class: 1,
            remove_class: 1,
            toggle_important: 1,
            increase_specificity: 1,
            move_layer: 1,
            move_source_order: 1,
            toggle_condition: 1,
            calibration_stage: GUARDED_CASCADE_ROBUSTNESS_CALIBRATION_STAGE_V0,
            public_safety_claim_ready: false,
        }
    }

    pub const fn cost(&self, kind: GuardedCascadePerturbationKindV0) -> u32 {
        match kind {
            GuardedCascadePerturbationKindV0::AddClass => self.add_class,
            GuardedCascadePerturbationKindV0::RemoveClass => self.remove_class,
            GuardedCascadePerturbationKindV0::ToggleImportant => self.toggle_important,
            GuardedCascadePerturbationKindV0::IncreaseSpecificity => self.increase_specificity,
            GuardedCascadePerturbationKindV0::MoveLayer => self.move_layer,
            GuardedCascadePerturbationKindV0::MoveSourceOrder => self.move_source_order,
            GuardedCascadePerturbationKindV0::ToggleCondition => self.toggle_condition,
        }
    }
}

pub const fn guarded_cascade_perturbation_cost_model_v0() -> GuardedCascadePerturbationCostModelV0 {
    GuardedCascadePerturbationCostModelV0::unit_cost_v0()
}

#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", content = "cost", rename_all = "camelCase")]
pub enum GuardedCascadeRobustnessRadiusValueV0 {
    Finite(u32),
    Infinity,
}

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GuardedCascadeConditionImplicationV0 {
    pub antecedent_atom: String,
    pub consequent_atom: String,
}

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GuardedCascadeRealisabilityModelV0 {
    pub derivation: &'static str,
    pub always_false_atoms: Vec<String>,
    pub implications: Vec<GuardedCascadeConditionImplicationV0>,
}

#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GuardedCascadeRobustnessRadiusV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub baseline_winner_declaration_id: u32,
    pub radius: GuardedCascadeRobustnessRadiusValueV0,
    pub witness: Vec<GuardedCascadePerturbationV0>,
    pub evaluated_perturbation_set_count: usize,
    pub verified_below_radius_perturbation_set_count: usize,
    pub excluded_unrealisable_assignment_count: usize,
    pub realisability: GuardedCascadeRealisabilityModelV0,
    pub calibration_stage: &'static str,
    pub public_safety_claim_ready: bool,
    pub min_plus_duplication_reason: &'static str,
}

#[non_exhaustive]
#[derive(Debug)]
pub enum GuardedCascadeRobustnessErrorV0 {
    FirstWitness(FirstWitnessErrorV0),
    MissingBaselineWinner,
    AssignmentCardinalityMismatch {
        expected: usize,
        observed: usize,
    },
    PerturbationCapacityExceeded {
        observed: usize,
        capacity: usize,
    },
    ZeroPerturbationCost {
        kind: GuardedCascadePerturbationKindV0,
    },
}

impl From<FirstWitnessErrorV0> for GuardedCascadeRobustnessErrorV0 {
    fn from(value: FirstWitnessErrorV0) -> Self {
        Self::FirstWitness(value)
    }
}

impl std::fmt::Display for GuardedCascadeRobustnessErrorV0 {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FirstWitness(error) => error.fmt(formatter),
            Self::MissingBaselineWinner => formatter
                .write_str("guarded cascade robustness requires a baseline winner declaration"),
            Self::AssignmentCardinalityMismatch { expected, observed } => write!(
                formatter,
                "guarded cascade assignment cardinality mismatch: expected {expected}, observed {observed}"
            ),
            Self::PerturbationCapacityExceeded { observed, capacity } => write!(
                formatter,
                "guarded cascade perturbation capacity exceeded: observed {observed}, capacity {capacity}"
            ),
            Self::ZeroPerturbationCost { kind } => {
                write!(
                    formatter,
                    "guarded cascade perturbation {kind:?} has zero cost"
                )
            }
        }
    }
}

impl std::error::Error for GuardedCascadeRobustnessErrorV0 {}

#[derive(Clone)]
struct EnumeratedPerturbationV0 {
    perturbation: GuardedCascadePerturbationV0,
    candidate_index: Option<usize>,
    variable_index: Option<usize>,
    cost: u32,
}

pub fn compute_guarded_cascade_robustness_radius_v0(
    fragment: &GuardedCascadeFragmentV0<CascadeKey>,
    assignment: &[bool],
    cost_model: &GuardedCascadePerturbationCostModelV0,
) -> Result<GuardedCascadeRobustnessRadiusV0, GuardedCascadeRobustnessErrorV0> {
    let order = at_rule_nesting_order_for_fragment_v0(fragment)?;
    if assignment.len() != order.atoms().len() {
        return Err(
            GuardedCascadeRobustnessErrorV0::AssignmentCardinalityMismatch {
                expected: order.atoms().len(),
                observed: assignment.len(),
            },
        );
    }
    let baseline_winner = evaluate_fragment_winner(fragment, order.clone(), assignment)?
        .ok_or(GuardedCascadeRobustnessErrorV0::MissingBaselineWinner)?;
    let realisability = derive_guarded_cascade_realisability_v0(order.atoms());
    #[cfg(test)]
    let realisability = if std::env::var_os("OMENA_G122_INJECT_IGNORE_REALISABILITY").is_some() {
        GuardedCascadeRealisabilityModelV0 {
            derivation: realisability.derivation,
            always_false_atoms: Vec::new(),
            implications: Vec::new(),
        }
    } else {
        realisability
    };
    let perturbations = enumerate_perturbations(fragment, &order, cost_model)?;
    if perturbations.len() > MAX_GUARDED_CASCADE_PERTURBATIONS_V0 {
        return Err(
            GuardedCascadeRobustnessErrorV0::PerturbationCapacityExceeded {
                observed: perturbations.len(),
                capacity: MAX_GUARDED_CASCADE_PERTURBATIONS_V0,
            },
        );
    }

    let mut best: Option<(u32, Vec<GuardedCascadePerturbationV0>)> = None;
    let mut evaluated_perturbation_set_count = 0usize;
    let mut preserving_costs = Vec::new();
    let mut excluded_unrealisable_assignment_count = 0usize;
    let upper = 1u64 << perturbations.len();
    for mask in 1..upper {
        let cost = perturbations
            .iter()
            .enumerate()
            .filter(|(index, _)| mask & (1u64 << index) != 0)
            .map(|(_, perturbation)| perturbation.cost)
            .sum::<u32>();
        if best.as_ref().is_some_and(|(best, _)| cost > *best) {
            continue;
        }
        let (candidate_keys, candidate_assignment, witness) =
            apply_perturbation_set(fragment, assignment, perturbations.as_slice(), mask);
        if !assignment_is_realisable(
            order.atoms(),
            candidate_assignment.as_slice(),
            &realisability,
        ) {
            excluded_unrealisable_assignment_count += 1;
            continue;
        }
        let Some(candidate_fragment) = fragment_with_keys(fragment, candidate_keys) else {
            continue;
        };
        evaluated_perturbation_set_count += 1;
        let winner = evaluate_fragment_winner(
            &candidate_fragment,
            order.clone(),
            candidate_assignment.as_slice(),
        )?;
        if winner != Some(baseline_winner) {
            if best.as_ref().is_none_or(|(best, _)| cost < *best) {
                best = Some((cost, witness));
            }
        } else {
            preserving_costs.push(cost);
        }
    }

    let (radius, witness) = best.map_or(
        (GuardedCascadeRobustnessRadiusValueV0::Infinity, Vec::new()),
        |(cost, witness)| (GuardedCascadeRobustnessRadiusValueV0::Finite(cost), witness),
    );
    let verified_below_radius_perturbation_set_count = preserving_costs
        .into_iter()
        .filter(|cost| match radius {
            GuardedCascadeRobustnessRadiusValueV0::Finite(radius) => *cost < radius,
            GuardedCascadeRobustnessRadiusValueV0::Infinity => true,
        })
        .count();
    Ok(GuardedCascadeRobustnessRadiusV0 {
        schema_version: "0",
        product: GUARDED_CASCADE_ROBUSTNESS_PRODUCT_V0,
        baseline_winner_declaration_id: baseline_winner,
        radius,
        witness,
        evaluated_perturbation_set_count,
        verified_below_radius_perturbation_set_count,
        excluded_unrealisable_assignment_count,
        realisability,
        calibration_stage: cost_model.calibration_stage,
        public_safety_claim_ready: cost_model.public_safety_claim_ready,
        min_plus_duplication_reason: GUARDED_CASCADE_ROBUSTNESS_MIN_PLUS_DUPLICATION_REASON_V0,
    })
}

fn evaluate_fragment_winner(
    fragment: &GuardedCascadeFragmentV0<CascadeKey>,
    order: crate::VariableOrderRegistrationV0,
    assignment: &[bool],
) -> Result<Option<u32>, GuardedCascadeRobustnessErrorV0> {
    let mut manager = FirstWitnessManagerV0::new(order, FirstWitnessManagerConfigV0::default());
    let root = build_guarded_cascade_winner_v0(&mut manager, fragment)?;
    Ok(evaluate_guarded_cascade_winner_v0(
        &manager, root, assignment,
    )?)
}

fn enumerate_perturbations(
    fragment: &GuardedCascadeFragmentV0<CascadeKey>,
    order: &crate::VariableOrderRegistrationV0,
    cost_model: &GuardedCascadePerturbationCostModelV0,
) -> Result<Vec<EnumeratedPerturbationV0>, GuardedCascadeRobustnessErrorV0> {
    let mut result = Vec::new();
    for (candidate_index, candidate) in fragment.candidates().iter().enumerate() {
        let declaration_id = candidate.declaration_id();
        let key = *candidate.cascade_key();
        for perturbation in [
            GuardedCascadePerturbationV0::AddClass { declaration_id },
            GuardedCascadePerturbationV0::IncreaseSpecificity { declaration_id },
            GuardedCascadePerturbationV0::MoveSourceOrder { declaration_id },
        ] {
            push_enumerated(
                &mut result,
                perturbation,
                Some(candidate_index),
                None,
                cost_model,
            )?;
        }
        if key.specificity.classes > 0 {
            push_enumerated(
                &mut result,
                GuardedCascadePerturbationV0::RemoveClass { declaration_id },
                Some(candidate_index),
                None,
                cost_model,
            )?;
        }
        if toggle_important_level(key.level).is_some() {
            push_enumerated(
                &mut result,
                GuardedCascadePerturbationV0::ToggleImportant { declaration_id },
                Some(candidate_index),
                None,
                cost_model,
            )?;
        }
        if moved_layer_rank(key).is_some() {
            push_enumerated(
                &mut result,
                GuardedCascadePerturbationV0::MoveLayer { declaration_id },
                Some(candidate_index),
                None,
                cost_model,
            )?;
        }
    }
    for (variable_index, atom) in order.atoms().iter().enumerate() {
        push_enumerated(
            &mut result,
            GuardedCascadePerturbationV0::ToggleCondition { atom: atom.clone() },
            None,
            Some(variable_index),
            cost_model,
        )?;
    }
    Ok(result)
}

fn push_enumerated(
    result: &mut Vec<EnumeratedPerturbationV0>,
    perturbation: GuardedCascadePerturbationV0,
    candidate_index: Option<usize>,
    variable_index: Option<usize>,
    cost_model: &GuardedCascadePerturbationCostModelV0,
) -> Result<(), GuardedCascadeRobustnessErrorV0> {
    let cost = cost_model.cost(perturbation.kind());
    #[cfg(test)]
    let cost = if std::env::var_os("OMENA_G122_INJECT_IGNORE_RADIUS_COST_MODEL").is_some() {
        1
    } else {
        cost
    };
    if cost == 0 {
        return Err(GuardedCascadeRobustnessErrorV0::ZeroPerturbationCost {
            kind: perturbation.kind(),
        });
    }
    result.push(EnumeratedPerturbationV0 {
        perturbation,
        candidate_index,
        variable_index,
        cost,
    });
    Ok(())
}

fn apply_perturbation_set(
    fragment: &GuardedCascadeFragmentV0<CascadeKey>,
    assignment: &[bool],
    perturbations: &[EnumeratedPerturbationV0],
    mask: u64,
) -> (
    Vec<CascadeKey>,
    Vec<bool>,
    Vec<GuardedCascadePerturbationV0>,
) {
    let mut keys = fragment
        .candidates()
        .iter()
        .map(|candidate| *candidate.cascade_key())
        .collect::<Vec<_>>();
    let mut assignment = assignment.to_vec();
    let mut witness = Vec::new();
    for (index, perturbation) in perturbations.iter().enumerate() {
        if mask & (1u64 << index) == 0 {
            continue;
        }
        if let Some(candidate_index) = perturbation.candidate_index {
            apply_key_perturbation(&mut keys[candidate_index], &perturbation.perturbation);
        }
        if let Some(variable_index) = perturbation.variable_index {
            assignment[variable_index] = !assignment[variable_index];
        }
        witness.push(perturbation.perturbation.clone());
    }
    (keys, assignment, witness)
}

fn apply_key_perturbation(key: &mut CascadeKey, perturbation: &GuardedCascadePerturbationV0) {
    #[cfg(test)]
    if std::env::var_os("OMENA_G122_INJECT_DISABLE_KEY_PERTURBATIONS").is_some() {
        return;
    }
    match perturbation {
        GuardedCascadePerturbationV0::AddClass { .. }
        | GuardedCascadePerturbationV0::IncreaseSpecificity { .. } => {
            key.specificity = Specificity::new(
                key.specificity.ids,
                key.specificity.classes.saturating_add(1),
                key.specificity.elements,
            );
        }
        GuardedCascadePerturbationV0::RemoveClass { .. } => {
            key.specificity = Specificity::new(
                key.specificity.ids,
                key.specificity.classes.saturating_sub(1),
                key.specificity.elements,
            );
        }
        GuardedCascadePerturbationV0::ToggleImportant { .. } => {
            if let Some(level) = toggle_important_level(key.level) {
                key.level = level;
            }
        }
        GuardedCascadePerturbationV0::MoveLayer { .. } => {
            if let Some(layer_rank) = moved_layer_rank(*key) {
                key.layer_rank = layer_rank;
            }
        }
        GuardedCascadePerturbationV0::MoveSourceOrder { .. } => {
            key.source_order = if key.source_order == u32::MAX {
                0
            } else {
                u32::MAX
            };
        }
        GuardedCascadePerturbationV0::ToggleCondition { .. } => {}
    }
}

fn toggle_important_level(level: CascadeLevel) -> Option<CascadeLevel> {
    match level {
        CascadeLevel::UserAgentNormal => Some(CascadeLevel::UserAgentImportant),
        CascadeLevel::UserNormal => Some(CascadeLevel::UserImportant),
        CascadeLevel::AuthorNormal => Some(CascadeLevel::AuthorImportant),
        CascadeLevel::InlineNormal => Some(CascadeLevel::InlineImportant),
        CascadeLevel::AuthorImportant => Some(CascadeLevel::AuthorNormal),
        CascadeLevel::InlineImportant => Some(CascadeLevel::InlineNormal),
        CascadeLevel::UserImportant => Some(CascadeLevel::UserNormal),
        CascadeLevel::UserAgentImportant => Some(CascadeLevel::UserAgentNormal),
        CascadeLevel::Animation | CascadeLevel::Transition => None,
    }
}

fn moved_layer_rank(key: CascadeKey) -> Option<crate::LayerRank> {
    let important = matches!(
        key.level,
        CascadeLevel::AuthorImportant
            | CascadeLevel::InlineImportant
            | CascadeLevel::UserImportant
            | CascadeLevel::UserAgentImportant
    );
    let rank = key.layer_rank.get();
    let ordinal = if important {
        if rank == i32::MIN {
            0
        } else {
            rank.checked_neg()?.saturating_add(1)
        }
    } else if rank == i32::MAX {
        0
    } else {
        rank.saturating_add(1)
    };
    LayerOrdinal::new(ordinal).map(|ordinal| normalized_layer_rank(important, Some(ordinal)))
}

fn fragment_with_keys(
    fragment: &GuardedCascadeFragmentV0<CascadeKey>,
    keys: Vec<CascadeKey>,
) -> Option<GuardedCascadeFragmentV0<CascadeKey>> {
    let candidates = fragment
        .candidates()
        .iter()
        .zip(keys)
        .map(|(candidate, key)| {
            GuardedCascadeCandidateV0::new(
                candidate.declaration_id(),
                candidate.element_signature(),
                candidate.property().clone(),
                key,
                GuardedCascadeSpecificityExactnessV0::Exact,
                candidate.scope_proximity(),
                candidate.conditions().to_vec(),
            )
        });
    GuardedCascadeFragmentV0::admit(fragment.condition_alphabet().iter().cloned(), candidates).ok()
}

fn derive_guarded_cascade_realisability_v0(atoms: &[String]) -> GuardedCascadeRealisabilityModelV0 {
    let parsed = atoms
        .iter()
        .filter_map(|atom| parse_width_bounds(atom).map(|bounds| (atom, bounds)))
        .collect::<Vec<_>>();
    let always_false_atoms = parsed
        .iter()
        .filter(|(_, bounds)| {
            bounds.min.is_some()
                && bounds.max.is_some()
                && bounds.min_unit == bounds.max_unit
                && bounds.min > bounds.max
        })
        .map(|(atom, _)| (*atom).clone())
        .collect::<Vec<_>>();
    let mut implications = BTreeSet::new();
    for (antecedent_atom, antecedent) in &parsed {
        let (Some(antecedent_min), Some(antecedent_unit)) =
            (antecedent.min, antecedent.min_unit.as_deref())
        else {
            continue;
        };
        for (consequent_atom, consequent) in &parsed {
            let (Some(consequent_min), Some(consequent_unit)) =
                (consequent.min, consequent.min_unit.as_deref())
            else {
                continue;
            };
            if antecedent_atom != consequent_atom
                && antecedent_unit == consequent_unit
                && antecedent_min >= consequent_min
            {
                implications.insert(((*antecedent_atom).clone(), (*consequent_atom).clone()));
            }
        }
    }
    GuardedCascadeRealisabilityModelV0 {
        derivation: "sameUnitWidthBoundaryMonotonicity",
        always_false_atoms,
        implications: implications
            .into_iter()
            .map(
                |(antecedent_atom, consequent_atom)| GuardedCascadeConditionImplicationV0 {
                    antecedent_atom,
                    consequent_atom,
                },
            )
            .collect(),
    }
}

fn assignment_is_realisable(
    atoms: &[String],
    assignment: &[bool],
    model: &GuardedCascadeRealisabilityModelV0,
) -> bool {
    let values = atoms
        .iter()
        .cloned()
        .zip(assignment.iter().copied())
        .collect::<BTreeMap<_, _>>();
    if model
        .always_false_atoms
        .iter()
        .any(|atom| values.get(atom).copied().unwrap_or(false))
    {
        return false;
    }
    model.implications.iter().all(|implication| {
        !values
            .get(implication.antecedent_atom.as_str())
            .copied()
            .unwrap_or(false)
            || values
                .get(implication.consequent_atom.as_str())
                .copied()
                .unwrap_or(false)
    })
}

#[derive(Default)]
struct WidthBoundsV0 {
    min: Option<u32>,
    min_unit: Option<String>,
    max: Option<u32>,
    max_unit: Option<String>,
}

fn parse_width_bounds(atom: &str) -> Option<WidthBoundsV0> {
    let lower = atom.to_ascii_lowercase();
    let mut bounds = WidthBoundsV0::default();
    if let Some((value, unit)) = numeric_boundary(&lower, "min-width") {
        bounds.min = Some(value);
        bounds.min_unit = Some(unit);
    }
    if let Some((value, unit)) = numeric_boundary(&lower, "max-width") {
        bounds.max = Some(value);
        bounds.max_unit = Some(unit);
    }
    (bounds.min.is_some() || bounds.max.is_some()).then_some(bounds)
}

fn numeric_boundary(source: &str, name: &str) -> Option<(u32, String)> {
    let start = source.find(name)? + name.len();
    let tail = source.get(start..)?.trim_start_matches([' ', ':']);
    let digits = tail
        .chars()
        .take_while(|character| character.is_ascii_digit())
        .collect::<String>();
    let value = digits.parse().ok()?;
    let unit = tail
        .get(digits.len()..)?
        .chars()
        .take_while(|character| character.is_ascii_alphabetic() || *character == '%')
        .collect::<String>();
    (!unit.is_empty()).then_some((value, unit))
}

#[cfg(test)]
mod tests {
    use crate::{GuardedCascadeConditionAtomV0, GuardedCascadeSpecificityExactnessV0};
    use omena_syntax::ident::AuthoredPropertyTextV0;

    use super::*;

    fn candidate(
        declaration_id: u32,
        source_order: u32,
        condition: Option<&str>,
    ) -> GuardedCascadeCandidateV0<CascadeKey> {
        GuardedCascadeCandidateV0::new(
            declaration_id,
            ".a",
            AuthoredPropertyTextV0::new("color"),
            CascadeKey::new(
                CascadeLevel::AuthorNormal,
                normalized_layer_rank(false, LayerOrdinal::new(0)),
                0,
                Specificity::new(0, 1, 0),
                source_order,
            ),
            GuardedCascadeSpecificityExactnessV0::Exact,
            0,
            condition
                .map(|condition| vec![GuardedCascadeConditionAtomV0::media(condition, [0], true)])
                .unwrap_or_default(),
        )
    }

    fn candidate_with_key(
        declaration_id: u32,
        key: CascadeKey,
    ) -> GuardedCascadeCandidateV0<CascadeKey> {
        GuardedCascadeCandidateV0::new(
            declaration_id,
            ".a",
            AuthoredPropertyTextV0::new("color"),
            key,
            GuardedCascadeSpecificityExactnessV0::Exact,
            0,
            Vec::new(),
        )
    }

    fn key(
        level: CascadeLevel,
        layer_ordinal: Option<i32>,
        classes: u32,
        source_order: u32,
    ) -> CascadeKey {
        CascadeKey::new(
            level,
            normalized_layer_rank(false, layer_ordinal.and_then(LayerOrdinal::new)),
            0,
            Specificity::new(0, classes, 0),
            source_order,
        )
    }

    fn isolated_cost_model(
        kind: GuardedCascadePerturbationKindV0,
    ) -> GuardedCascadePerturbationCostModelV0 {
        let mut model = GuardedCascadePerturbationCostModelV0 {
            add_class: 11,
            remove_class: 11,
            toggle_important: 11,
            increase_specificity: 11,
            move_layer: 11,
            move_source_order: 11,
            toggle_condition: 11,
            calibration_stage: GUARDED_CASCADE_ROBUSTNESS_CALIBRATION_STAGE_V0,
            public_safety_claim_ready: false,
        };
        match kind {
            GuardedCascadePerturbationKindV0::AddClass => model.add_class = 1,
            GuardedCascadePerturbationKindV0::RemoveClass => model.remove_class = 1,
            GuardedCascadePerturbationKindV0::ToggleImportant => model.toggle_important = 1,
            GuardedCascadePerturbationKindV0::IncreaseSpecificity => {
                model.increase_specificity = 1;
            }
            GuardedCascadePerturbationKindV0::MoveLayer => model.move_layer = 1,
            GuardedCascadePerturbationKindV0::MoveSourceOrder => model.move_source_order = 1,
            GuardedCascadePerturbationKindV0::ToggleCondition => model.toggle_condition = 1,
        }
        model
    }

    fn perturbation(
        kind: GuardedCascadePerturbationKindV0,
        declaration_id: u32,
    ) -> GuardedCascadePerturbationV0 {
        match kind {
            GuardedCascadePerturbationKindV0::AddClass => {
                GuardedCascadePerturbationV0::AddClass { declaration_id }
            }
            GuardedCascadePerturbationKindV0::RemoveClass => {
                GuardedCascadePerturbationV0::RemoveClass { declaration_id }
            }
            GuardedCascadePerturbationKindV0::ToggleImportant => {
                GuardedCascadePerturbationV0::ToggleImportant { declaration_id }
            }
            GuardedCascadePerturbationKindV0::IncreaseSpecificity => {
                GuardedCascadePerturbationV0::IncreaseSpecificity { declaration_id }
            }
            GuardedCascadePerturbationKindV0::MoveLayer => {
                GuardedCascadePerturbationV0::MoveLayer { declaration_id }
            }
            GuardedCascadePerturbationKindV0::MoveSourceOrder => {
                GuardedCascadePerturbationV0::MoveSourceOrder { declaration_id }
            }
            GuardedCascadePerturbationKindV0::ToggleCondition => {
                GuardedCascadePerturbationV0::ToggleCondition {
                    atom: String::new(),
                }
            }
        }
    }

    fn fragment(
        condition: &str,
    ) -> Result<GuardedCascadeFragmentV0<CascadeKey>, crate::GuardedCascadeFragmentRefusalV0> {
        GuardedCascadeFragmentV0::admit(
            [condition],
            [candidate(0, 0, None), candidate(1, 1, Some(condition))],
        )
    }

    #[test]
    fn radius_computes_named_cheapest_flip_and_moves_with_the_cost_table()
    -> Result<(), Box<dyn std::error::Error>> {
        let condition = "@media (min-width: 1px)";
        let fragment = fragment(condition)?;
        let unit = compute_guarded_cascade_robustness_radius_v0(
            &fragment,
            &[false],
            &guarded_cascade_perturbation_cost_model_v0(),
        )?;
        assert_eq!(
            unit.radius,
            GuardedCascadeRobustnessRadiusValueV0::Finite(1)
        );
        assert_eq!(
            unit.witness,
            vec![GuardedCascadePerturbationV0::ToggleCondition {
                atom: condition.to_string(),
            }]
        );
        assert_eq!(unit.calibration_stage, "schemaOnlyUncalibrated");
        assert!(!unit.public_safety_claim_ready);

        let mut changed_cost = guarded_cascade_perturbation_cost_model_v0();
        changed_cost.toggle_condition = 7;
        let changed =
            compute_guarded_cascade_robustness_radius_v0(&fragment, &[false], &changed_cost)?;
        assert_eq!(
            changed.radius,
            GuardedCascadeRobustnessRadiusValueV0::Finite(7)
        );
        Ok(())
    }

    #[test]
    fn every_key_perturbation_kind_has_a_cheapest_winner_flip()
    -> Result<(), Box<dyn std::error::Error>> {
        let ordinary_winner = key(CascadeLevel::AuthorNormal, Some(0), 2, 0);
        let ordinary_challenger = key(CascadeLevel::AuthorNormal, Some(0), 1, 1);
        let cases = [
            (
                GuardedCascadePerturbationKindV0::AddClass,
                ordinary_winner,
                ordinary_challenger,
                1,
            ),
            (
                GuardedCascadePerturbationKindV0::RemoveClass,
                ordinary_winner,
                ordinary_challenger,
                0,
            ),
            (
                GuardedCascadePerturbationKindV0::ToggleImportant,
                ordinary_winner,
                ordinary_challenger,
                1,
            ),
            (
                GuardedCascadePerturbationKindV0::IncreaseSpecificity,
                ordinary_winner,
                ordinary_challenger,
                1,
            ),
            (
                GuardedCascadePerturbationKindV0::MoveLayer,
                key(CascadeLevel::AuthorNormal, None, 1, 0),
                key(CascadeLevel::AuthorNormal, Some(0), 1, 1),
                0,
            ),
            (
                GuardedCascadePerturbationKindV0::MoveSourceOrder,
                key(CascadeLevel::AuthorNormal, Some(0), 1, 1),
                key(CascadeLevel::AuthorNormal, Some(0), 1, 0),
                1,
            ),
        ];
        let mut observations = Vec::new();
        for (kind, winner, challenger, target_declaration_id) in cases {
            let fragment = GuardedCascadeFragmentV0::admit(
                std::iter::empty::<&str>(),
                [
                    candidate_with_key(0, winner),
                    candidate_with_key(1, challenger),
                ],
            )?;
            let result = compute_guarded_cascade_robustness_radius_v0(
                &fragment,
                &[],
                &isolated_cost_model(kind),
            )?;
            assert_eq!(
                result.radius,
                GuardedCascadeRobustnessRadiusValueV0::Finite(1),
                "{kind:?} must provide a real cheapest winner flip"
            );
            assert_eq!(
                result.witness,
                vec![perturbation(kind, target_declaration_id)],
                "{kind:?} must name the declaration whose key crosses the winner boundary"
            );
            observations.push((kind, result.baseline_winner_declaration_id, result.witness));
        }
        eprintln!("S6_KEY_PERTURBATION_OBSERVATIONS={observations:?}");
        Ok(())
    }

    #[test]
    fn unrealisable_only_flip_has_infinite_radius() -> Result<(), Box<dyn std::error::Error>> {
        let contradictory = "@media (min-width: 1200px) and (max-width: 768px)";
        let fragment = fragment(contradictory)?;
        let radius = compute_guarded_cascade_robustness_radius_v0(
            &fragment,
            &[false],
            &guarded_cascade_perturbation_cost_model_v0(),
        )?;
        assert_eq!(
            radius.radius,
            GuardedCascadeRobustnessRadiusValueV0::Infinity
        );
        assert!(radius.witness.is_empty());
        assert!(radius.excluded_unrealisable_assignment_count > 0);
        assert!(radius.verified_below_radius_perturbation_set_count > 0);
        assert_eq!(
            radius.verified_below_radius_perturbation_set_count,
            radius.evaluated_perturbation_set_count,
            "every evaluated finite perturbation is below an infinite radius"
        );
        assert_eq!(
            radius.realisability.always_false_atoms,
            vec![contradictory.to_string()]
        );
        Ok(())
    }

    #[test]
    fn every_realisable_perturbation_below_radius_preserves_the_winner()
    -> Result<(), Box<dyn std::error::Error>> {
        let condition = "@media (min-width: 1px)";
        let fragment = fragment(condition)?;
        let mut costs = guarded_cascade_perturbation_cost_model_v0();
        costs.toggle_condition = 3;
        let radius = compute_guarded_cascade_robustness_radius_v0(&fragment, &[false], &costs)?;
        assert_eq!(
            radius.radius,
            GuardedCascadeRobustnessRadiusValueV0::Finite(3)
        );
        let order = at_rule_nesting_order_for_fragment_v0(&fragment)?;
        let perturbations = enumerate_perturbations(&fragment, &order, &costs)?;
        let baseline = independently_rederive_winner(&fragment, order.atoms(), &[false]);
        let mut independently_verified = 0usize;
        for mask in 1..(1u64 << perturbations.len()) {
            let cost = perturbations
                .iter()
                .enumerate()
                .filter(|(index, _)| mask & (1u64 << index) != 0)
                .map(|(_, perturbation)| perturbation.cost)
                .sum::<u32>();
            if cost >= 3 {
                continue;
            }
            let (keys, assignment, _) =
                apply_perturbation_set(&fragment, &[false], &perturbations, mask);
            if !assignment_is_realisable(order.atoms(), &assignment, &radius.realisability) {
                continue;
            }
            let Some(candidate_fragment) = fragment_with_keys(&fragment, keys) else {
                continue;
            };
            assert_eq!(
                independently_rederive_winner(&candidate_fragment, order.atoms(), &assignment),
                baseline,
                "an independently rederived sub-radius path changed the winner"
            );
            independently_verified += 1;
        }
        assert!(independently_verified > 0);
        assert_eq!(
            radius.verified_below_radius_perturbation_set_count, independently_verified,
            "the theorem-7 receipt must equal the independent sub-radius rederivation"
        );
        Ok(())
    }

    fn independently_rederive_winner(
        fragment: &GuardedCascadeFragmentV0<CascadeKey>,
        atoms: &[String],
        assignment: &[bool],
    ) -> Option<u32> {
        let values = atoms
            .iter()
            .map(String::as_str)
            .zip(assignment.iter().copied())
            .collect::<BTreeMap<_, _>>();
        fragment
            .candidates()
            .iter()
            .filter(|candidate| {
                candidate
                    .conditions()
                    .iter()
                    .all(|condition| values.get(condition.atom()).copied().unwrap_or(false))
            })
            .max_by_key(|candidate| *candidate.cascade_key())
            .map(GuardedCascadeCandidateV0::declaration_id)
    }
}
