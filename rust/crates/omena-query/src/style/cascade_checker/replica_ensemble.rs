use std::collections::BTreeMap;

use omena_cascade::{
    CascadeDeclaration, CascadeKey, CascadeValue, LayerRank, ModuleRank, Specificity,
    cascade_level_for_origin, cascade_property, parse_simple_selector_signature,
};
use omena_query_checker_orchestrator::{
    OmenaCheckerCascadeDeclarationInputV0, REPLICA_ENSEMBLE_FEATURE_GATE_V0,
    REPLICA_ENSEMBLE_LAYER_MARKER_V0, REPLICA_ENSEMBLE_SCHEMA_VERSION_V0, ReplicaSiteOutcomeV0,
    site as replica_ensemble_site,
};

use super::collect_query_checker_cascade_declarations;

/// Compute the REAL per-`(selector, property)` cascade winners a parsed
/// stylesheet produces, projected as replica-ensemble site outcomes.
///
/// This is the cross-file replica-ensemble's per-file input: each in-graph CSS
/// module is one replica, and its "site outcomes" are the winning value of every
/// `(selector, property)` it declares. The winners are not literals — they are
/// computed by the genuine `cascade_property` ranking over the parsed
/// declarations: declarations are grouped by `(selector, property)`, each is
/// assigned a real `CascadeKey` (author-important vs author-normal level, real
/// `@layer` rank, real selector specificity from `parse_simple_selector_signature`,
/// and source order), and the lexicographic cascade key picks the winner. The
/// winning declaration's value is carried as the outcome identity (via the
/// `CascadeDeclaration.id`), so two replicas *agree* on a site iff their winning
/// value for that `(selector, property)` is identical, and *disagree* iff their
/// cascades resolve to different values. No replica snapshot is fabricated.
///
/// Custom-property declarations (`--*`) are excluded: their winner is a token
/// whose meaning depends on the whole-graph fixed point, not a directly
/// comparable per-file value.
pub(in crate::style) fn collect_query_replica_ensemble_site_outcomes(
    source: &str,
) -> Vec<ReplicaSiteOutcomeV0> {
    let declarations = collect_query_checker_cascade_declarations(source);

    let mut by_site: BTreeMap<(String, String), Vec<CascadeDeclaration>> = BTreeMap::new();
    for declaration in &declarations {
        let property = declaration.input.property.as_str();
        if property.starts_with("--") {
            continue;
        }
        let cascade_declaration = query_cascade_declaration_from_input(&declaration.input);
        by_site
            .entry((
                declaration.input.selector.as_str().to_string(),
                declaration.input.property.clone(),
            ))
            .or_default()
            .push(cascade_declaration);
    }

    by_site
        .into_iter()
        .filter_map(|((selector, property), site_declarations)| {
            let outcome = cascade_property(site_declarations, &property);
            if !matches!(outcome, omena_cascade::CascadeOutcome::Definite { .. }) {
                return None;
            }
            Some(ReplicaSiteOutcomeV0 {
                schema_version: REPLICA_ENSEMBLE_SCHEMA_VERSION_V0,
                product: "omena-ensemble.replica-site-outcome",
                layer_marker: REPLICA_ENSEMBLE_LAYER_MARKER_V0,
                feature_gate: REPLICA_ENSEMBLE_FEATURE_GATE_V0,
                site: replica_ensemble_site(selector, property),
                outcome,
                provenance: None,
            })
        })
        .collect()
}

/// Lift a parsed cascade declaration onto an `omena-cascade` `CascadeDeclaration`
/// with a real cascade key. The winning value is carried as the declaration `id`
/// so the replica-overlap projection (`definite:<id>`) keys agreement on the
/// resolved value rather than a per-file synthetic identifier.
fn query_cascade_declaration_from_input(
    input: &OmenaCheckerCascadeDeclarationInputV0,
) -> CascadeDeclaration {
    let level = cascade_level_for_origin(input.origin, input.important);
    let layer_rank = LayerRank(input.layer_order.unwrap_or(0));
    let (specificity, specificity_exactness) =
        parse_simple_selector_signature(input.selector.as_str()).map_or(
            (
                Specificity::ZERO,
                omena_cascade::SpecificityExactnessV0::Inexact,
            ),
            |signature| (signature.specificity, signature.specificity_exactness),
        );
    let value = input.value.trim().to_string();

    CascadeDeclaration {
        id: value.clone(),
        property: input.property.clone(),
        value: CascadeValue::Literal(value),
        key: CascadeKey::new(
            level,
            layer_rank,
            0,
            specificity,
            ModuleRank::ZERO,
            input.source_order,
        ),
        specificity_exactness,
    }
}
