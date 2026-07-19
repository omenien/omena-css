//! Cascade ranking helpers shared by conformance, fuzz, and query consumers.
//!
//! The helpers implement the crate's lexicographic cascade key ordering while
//! preserving non-winning declarations as evidence for diagnostics and proof
//! reports.

use std::cmp::{Ordering, Reverse};

use crate::{
    CascadeDeclaration, CascadeKey, CascadeLevel, CascadeMarginSchemaV0, CascadeMarginV0,
    CascadeOutcome, CascadeProof,
};

pub fn cascade_property(
    declarations: impl IntoIterator<Item = CascadeDeclaration>,
    property: &str,
) -> CascadeOutcome {
    let mut matching: Vec<CascadeDeclaration> = declarations
        .into_iter()
        .filter(|declaration| declaration.property == property)
        .collect();

    if matching.is_empty() {
        return CascadeOutcome::Inherit;
    }

    matching.sort_by_key(|declaration| Reverse(declaration.key));
    let winner = matching.remove(0);
    let proof = CascadeProof::from_declaration(&winner);
    CascadeOutcome::Definite {
        winner,
        proof: Box::new(proof),
        also_considered: matching,
    }
}

pub fn cascade_property_open_world(
    declarations: impl IntoIterator<Item = CascadeDeclaration>,
    property: &str,
) -> CascadeOutcome {
    let mut matching: Vec<CascadeDeclaration> = declarations
        .into_iter()
        .filter(|declaration| declaration.property == property)
        .collect();

    if matching.is_empty() {
        return CascadeOutcome::Inherit;
    }

    matching.sort_by(compare_open_world_declarations);
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

fn compare_open_world_declarations(
    left: &CascadeDeclaration,
    right: &CascadeDeclaration,
) -> Ordering {
    right
        .key
        .cmp(&left.key)
        .then_with(|| right.key.module_rank.cmp(&left.key.module_rank))
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

pub fn summarize_cascade_margin_schema_v0() -> CascadeMarginSchemaV0 {
    CascadeMarginSchemaV0 {
        schema_version: "0",
        product: "omena-cascade.margin-schema",
        margin_kind: "lexicographicCascadeKeyDelta",
        axis_order: vec![
            "level",
            "layerRank",
            "scopeProximity",
            "specificityIds",
            "specificityClasses",
            "specificityElements",
            "sourceOrder",
        ],
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
    let level_delta = cascade_level_rank(winner.level) - cascade_level_rank(challenger.level);
    if level_delta != 0 {
        return ("level", level_delta);
    }

    let layer_delta = i64::from(winner.layer_rank.0) - i64::from(challenger.layer_rank.0);
    if layer_delta != 0 {
        return ("layerRank", layer_delta);
    }

    let scope_delta = i64::from(challenger.scope_proximity) - i64::from(winner.scope_proximity);
    if scope_delta != 0 {
        return ("scopeProximity", scope_delta);
    }

    let specificity_id_delta =
        i64::from(winner.specificity.ids) - i64::from(challenger.specificity.ids);
    if specificity_id_delta != 0 {
        return ("specificityIds", specificity_id_delta);
    }

    let specificity_class_delta =
        i64::from(winner.specificity.classes) - i64::from(challenger.specificity.classes);
    if specificity_class_delta != 0 {
        return ("specificityClasses", specificity_class_delta);
    }

    let specificity_element_delta =
        i64::from(winner.specificity.elements) - i64::from(challenger.specificity.elements);
    if specificity_element_delta != 0 {
        return ("specificityElements", specificity_element_delta);
    }

    (
        "sourceOrder",
        i64::from(winner.source_order) - i64::from(challenger.source_order),
    )
}

fn cascade_level_rank(level: CascadeLevel) -> i64 {
    match level {
        CascadeLevel::UserAgentNormal => 0,
        CascadeLevel::UserNormal => 1,
        CascadeLevel::AuthorNormal => 2,
        CascadeLevel::InlineNormal => 3,
        CascadeLevel::Animation => 4,
        CascadeLevel::AuthorImportant => 5,
        CascadeLevel::UserImportant => 6,
        CascadeLevel::UserAgentImportant => 7,
        CascadeLevel::Transition => 8,
    }
}
