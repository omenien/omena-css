//! Cascade ranking helpers shared by conformance, fuzz, and query consumers.
//!
//! The helpers implement the crate's lexicographic cascade key ordering while
//! preserving non-winning declarations as evidence for diagnostics and proof
//! reports.

use omena_syntax::ident::PropertyNameV0;
use std::cmp::{Ordering, Reverse};

use crate::{
    CascadeDeclaration, CascadeKey, CascadeMarginSchemaV0, CascadeMarginV0, CascadeOutcome,
    CascadeProof, OpenWorldTieEvidence, SpecificityExactnessV0,
    axis_order::{
        AxisComparisonV0, CascadeKeyAxisV0, cascade_key_axis_name_v0, cascade_key_axis_order_v0,
        cascade_key_axis_signed_distance_v0, compare_cascade_declaration_axes_v0,
        first_deciding_cascade_key_axis_v0,
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum InexactSpecificityAdjudicationV0 {
    Recoverable {
        winner_index: usize,
        deciding_axis: CascadeKeyAxisV0,
    },
    AxisWinnerInexact,
    NoStrictAxisDominance,
    SingleInexactCandidate,
}

pub(crate) fn adjudicate_inexact_specificity_v0(
    declarations: &[CascadeDeclaration],
) -> InexactSpecificityAdjudicationV0 {
    assert!(
        declarations.iter().any(|declaration| {
            declaration.specificity_exactness == SpecificityExactnessV0::Inexact
        }),
        "ranked-set loss classification requires an inexact declaration",
    );
    if declarations.len() == 1 {
        return InexactSpecificityAdjudicationV0::SingleInexactCandidate;
    }

    let Some((winner_index, winner)) = declarations
        .iter()
        .enumerate()
        .max_by(|(_, left), (_, right)| left.key.cmp(&right.key))
    else {
        return InexactSpecificityAdjudicationV0::SingleInexactCandidate;
    };
    let mut deciding_axis = None;
    for (challenger_index, challenger) in declarations.iter().enumerate() {
        if challenger_index == winner_index {
            continue;
        }
        let AxisComparisonV0::Decided {
            ordering: Ordering::Greater,
            axis,
        } = compare_cascade_declaration_axes_v0(winner, challenger)
        else {
            return InexactSpecificityAdjudicationV0::NoStrictAxisDominance;
        };
        if challenger.specificity_exactness == SpecificityExactnessV0::Inexact {
            deciding_axis = Some(match deciding_axis {
                Some(current) if axis_position(current) >= axis_position(axis) => current,
                Some(_) | None => axis,
            });
        }
    }

    if winner.specificity_exactness == SpecificityExactnessV0::Inexact {
        return InexactSpecificityAdjudicationV0::AxisWinnerInexact;
    }
    InexactSpecificityAdjudicationV0::Recoverable {
        winner_index,
        deciding_axis: deciding_axis
            .expect("an exact recovery winner must dominate an inexact challenger"),
    }
}

pub fn cascade_property(
    declarations: impl IntoIterator<Item = CascadeDeclaration>,
    property: &str,
) -> CascadeOutcome {
    let property_key = PropertyNameV0::from_authored(property).canonical_key();
    let mut matching: Vec<CascadeDeclaration> = declarations
        .into_iter()
        .filter(|declaration| declaration.property_key == property_key)
        .collect();

    if matching.is_empty() {
        return CascadeOutcome::Inherit;
    }

    matching.sort_by_key(|declaration| Reverse(declaration.key));
    if matching
        .iter()
        .any(|declaration| declaration.specificity_exactness == SpecificityExactnessV0::Inexact)
    {
        return match adjudicate_inexact_specificity_v0(&matching) {
            InexactSpecificityAdjudicationV0::Recoverable { winner_index, .. } => {
                definite_outcome(matching, winner_index)
            }
            InexactSpecificityAdjudicationV0::AxisWinnerInexact
            | InexactSpecificityAdjudicationV0::NoStrictAxisDominance
            | InexactSpecificityAdjudicationV0::SingleInexactCandidate => {
                CascadeOutcome::RankedSet(matching)
            }
        };
    }
    definite_outcome(matching, 0)
}

pub fn cascade_property_open_world(
    declarations: impl IntoIterator<Item = CascadeDeclaration>,
    property: &str,
) -> CascadeOutcome {
    let property_key = PropertyNameV0::from_authored(property).canonical_key();
    let mut matching: Vec<CascadeDeclaration> = declarations
        .into_iter()
        .filter(|declaration| declaration.property_key == property_key)
        .collect();

    if matching.is_empty() {
        return CascadeOutcome::Inherit;
    }

    matching.sort_by(compare_open_world_declarations);
    if matching
        .iter()
        .any(|declaration| declaration.specificity_exactness == SpecificityExactnessV0::Inexact)
    {
        return match adjudicate_inexact_specificity_v0(&matching) {
            InexactSpecificityAdjudicationV0::Recoverable { winner_index, .. } => {
                definite_outcome(matching, winner_index)
            }
            InexactSpecificityAdjudicationV0::AxisWinnerInexact
            | InexactSpecificityAdjudicationV0::NoStrictAxisDominance
            | InexactSpecificityAdjudicationV0::SingleInexactCandidate => {
                CascadeOutcome::RankedSet(matching)
            }
        };
    }
    let has_strict_base_key_winner = matching
        .get(1)
        .is_some_and(|runner_up| matching[0].key.cmp(&runner_up.key) == Ordering::Greater);
    if matching.len() == 1 || has_strict_base_key_winner {
        let winner = matching.remove(0);
        let proof = CascadeProof::from_declaration(&winner);
        return CascadeOutcome::Definite {
            winner,
            proof: Box::new(proof),
            also_considered: matching,
        };
    }

    CascadeOutcome::RankedSet(matching)
}

fn definite_outcome(
    mut declarations: Vec<CascadeDeclaration>,
    winner_index: usize,
) -> CascadeOutcome {
    let winner = declarations.remove(winner_index);
    let proof = CascadeProof::from_declaration(&winner);
    CascadeOutcome::Definite {
        winner,
        proof: Box::new(proof),
        also_considered: declarations,
    }
}

fn axis_position(axis: CascadeKeyAxisV0) -> usize {
    cascade_key_axis_order_v0()
        .iter()
        .position(|candidate| *candidate == axis)
        .expect("cascade axis must belong to the canonical order")
}

fn compare_open_world_declarations(
    left: &CascadeDeclaration,
    right: &CascadeDeclaration,
) -> Ordering {
    compare_open_world_cascade_keys(
        (left.key, left.open_world_tie_evidence),
        (right.key, right.open_world_tie_evidence),
    )
}

fn compare_open_world_cascade_keys(
    left: (CascadeKey, OpenWorldTieEvidence),
    right: (CascadeKey, OpenWorldTieEvidence),
) -> Ordering {
    right
        .0
        .cmp(&left.0)
        .then_with(|| right.1.module_rank.cmp(&left.1.module_rank))
}

pub fn rank_cascade_items<T>(
    items: impl IntoIterator<Item = T>,
    key_for: impl Fn(&T) -> CascadeKey,
) -> Vec<T> {
    let mut ranked = items.into_iter().collect::<Vec<_>>();
    ranked.sort_by_key(|item| Reverse(key_for(item)));
    ranked
}

pub fn select_cascade_winner<T>(
    items: impl IntoIterator<Item = T>,
    key_for: impl Fn(&T) -> CascadeKey,
) -> Option<(T, Vec<T>)> {
    let mut ranked = rank_cascade_items(items, key_for);
    if ranked.is_empty() {
        return None;
    }

    let winner = ranked.remove(0);
    Some((winner, ranked))
}

/// Selects an open-world winner while retaining provenance as a final tiebreak.
///
/// `CascadeKey::Ord` owns the spec-defined cascade axes. The returned
/// [`OpenWorldTieEvidence`] remains outside that order and is considered only
/// after those axes compare equal.
pub fn select_open_world_cascade_winner<T>(
    items: impl IntoIterator<Item = T>,
    key_and_evidence_for: impl Fn(&T) -> (CascadeKey, OpenWorldTieEvidence),
) -> Option<(T, Vec<T>)> {
    let mut ranked = items.into_iter().collect::<Vec<_>>();
    ranked.sort_by(|left, right| {
        compare_open_world_cascade_keys(key_and_evidence_for(left), key_and_evidence_for(right))
    });
    if ranked.is_empty() {
        return None;
    }

    let winner = ranked.remove(0);
    Some((winner, ranked))
}

pub fn summarize_cascade_margin_schema_v0() -> CascadeMarginSchemaV0 {
    CascadeMarginSchemaV0 {
        schema_version: "0",
        product: "omena-cascade.margin-schema",
        margin_kind: "lexicographicCascadeKeyDelta",
        axis_order: cascade_key_axis_order_v0()
            .iter()
            .copied()
            .map(cascade_key_axis_name_v0)
            .collect(),
        calibration_stage: "schemaOnlyUncalibrated",
        public_safety_claim_ready: false,
    }
}

pub fn cascade_margin_for_outcome(outcome: &CascadeOutcome) -> Option<CascadeMarginV0> {
    let CascadeOutcome::Definite {
        winner,
        also_considered,
        ..
    } = outcome
    else {
        return None;
    };

    let Some(challenger) = also_considered.first() else {
        return Some(CascadeMarginV0 {
            schema_version: "0",
            product: "omena-cascade.margin",
            margin_kind: "lexicographicCascadeKeyDelta",
            winner_declaration_id: winner.id.clone(),
            challenger_declaration_id: None,
            dominant_axis: "uncontested",
            signed_distance: 0,
            winner_key: winner.key,
            challenger_key: None,
            calibration_stage: "schemaOnlyUncalibrated",
            public_safety_claim_ready: false,
        });
    };

    let (dominant_axis, signed_distance) = dominant_cascade_key_margin(winner.key, challenger.key);
    Some(CascadeMarginV0 {
        schema_version: "0",
        product: "omena-cascade.margin",
        margin_kind: "lexicographicCascadeKeyDelta",
        winner_declaration_id: winner.id.clone(),
        challenger_declaration_id: Some(challenger.id.clone()),
        dominant_axis,
        signed_distance,
        winner_key: winner.key,
        challenger_key: Some(challenger.key),
        calibration_stage: "schemaOnlyUncalibrated",
        public_safety_claim_ready: false,
    })
}

fn dominant_cascade_key_margin(winner: CascadeKey, challenger: CascadeKey) -> (&'static str, i64) {
    let axis = first_deciding_cascade_key_axis_v0(&winner, &challenger)
        .unwrap_or(crate::axis_order::CascadeKeyAxisV0::SourceOrder);
    (
        cascade_key_axis_name_v0(axis),
        cascade_key_axis_signed_distance_v0(axis, &winner, &challenger),
    )
}
