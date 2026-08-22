//! Cascade-formal substrate for the Omena CSS track.
//!
//! The crate starts with the load-bearing algebra from the research plan:
//! lexicographic cascade keys, specificity, provenance proofs, and a finite
//! custom-property substitution function with explicit cycle handling.

mod axis_order;
pub mod class_tokens;
mod computed_value;
mod conformance;
mod custom_property;
pub mod first_witness;
mod frame_footprint;
mod fuzz;
mod grn;
mod modal;
mod model;
mod origin;
mod proofs;
mod property_metadata;
mod property_metadata_idl_generated;
mod ranked_set_loss_census;
mod ranking;
mod refinement;
mod robustness_radius;
mod selector;
mod shorthand_authority;
mod statistics;

pub use class_tokens::*;
pub use computed_value::*;
pub use conformance::*;
pub use custom_property::*;
pub use first_witness::*;
pub use frame_footprint::*;
pub use fuzz::*;
pub use grn::*;
pub use modal::*;
pub use model::*;
pub use origin::*;
pub use proofs::{evaluate_static_supports_condition, prove_layer_flatten_candidate};
pub use property_metadata::*;
pub use property_metadata_idl_generated::{
    CSS_PROPERTY_METADATA_RECORDS_V1, CSS_PROPERTY_METADATA_V1, CssCustomPropertyPolicyStaticV1,
    CssCustomPropertyPolicyV1Json, CssPropertyMetadataRecordStaticV1,
    CssPropertyMetadataRecordV1Json, CssPropertyMetadataSourceStaticV1,
    CssPropertyMetadataSourceV1Json, CssPropertyMetadataStaticV1, CssPropertyMetadataV1Json,
};
pub use ranked_set_loss_census::*;
pub use ranking::{
    cascade_margin_for_outcome, rank_cascade_items, select_cascade_winner,
    select_open_world_cascade_winner, summarize_cascade_margin_schema_v0,
};
pub use refinement::*;
pub use robustness_radius::*;
pub use selector::*;
pub use shorthand_authority::*;
pub use statistics::*;

#[track_caller]
pub fn cascade_property(
    declarations: impl IntoIterator<Item = CascadeDeclaration>,
    property: &str,
) -> CascadeOutcome {
    let outcome = ranking::cascade_property(declarations, property);
    ranked_set_loss_census::observe_cascade_outcome(
        CascadeRankedSetFunctionV0::CascadeProperty,
        std::panic::Location::caller(),
        &outcome,
    );
    outcome
}

#[track_caller]
pub fn cascade_property_open_world(
    declarations: impl IntoIterator<Item = CascadeDeclaration>,
    property: &str,
) -> CascadeOutcome {
    let outcome = ranking::cascade_property_open_world(declarations, property);
    ranked_set_loss_census::observe_cascade_outcome(
        CascadeRankedSetFunctionV0::CascadePropertyOpenWorld,
        std::panic::Location::caller(),
        &outcome,
    );
    outcome
}

pub fn prove_scope_flatten_candidate(mut input: ScopeFlattenInputV0) -> ScopeFlattenProofV0 {
    if omena_syntax::css_keyword(input.root_selector.trim()).equals(":root") {
        input.root_selector = ":root".to_string();
    }
    proofs::prove_scope_flatten_candidate(input)
}

pub fn prove_longhand_merge<S>(
    shorthand_property: &str,
    expected_longhands: &[S],
    longhands: &[LonghandMergeInputV0],
) -> LonghandMergeProofV0
where
    S: AsRef<str>,
{
    let canonical_shorthand =
        omena_syntax::ident::PropertyNameV0::from_authored(shorthand_property)
            .canonical_name()
            .to_string();
    let canonical_expected = expected_longhands
        .iter()
        .map(|property| {
            omena_syntax::ident::PropertyNameV0::from_authored(property.as_ref())
                .canonical_name()
                .to_string()
        })
        .collect::<Vec<_>>();
    let canonical_longhands = longhands
        .iter()
        .cloned()
        .map(|mut longhand| {
            longhand.property =
                omena_syntax::ident::PropertyNameV0::from_authored(&longhand.property)
                    .canonical_name()
                    .to_string();
            longhand
        })
        .collect::<Vec<_>>();
    let mut proof = proofs::prove_longhand_merge(
        &canonical_shorthand,
        &canonical_expected,
        &canonical_longhands,
    );
    restore_authored_longhand_proof_names(&mut proof, shorthand_property, longhands);
    proof
}

pub fn prove_box_shorthand_combination(
    shorthand_property: &str,
    longhands: &[BoxLonghandInputV0],
) -> ShorthandCombinationProofV0 {
    let canonical_shorthand =
        omena_syntax::ident::PropertyNameV0::from_authored(shorthand_property)
            .canonical_name()
            .to_string();
    let canonical_longhands = longhands
        .iter()
        .cloned()
        .map(|mut longhand| {
            longhand.property =
                omena_syntax::ident::PropertyNameV0::from_authored(&longhand.property)
                    .canonical_name()
                    .to_string();
            longhand
        })
        .collect::<Vec<_>>();
    let mut proof =
        proofs::prove_box_shorthand_combination(&canonical_shorthand, &canonical_longhands);
    restore_authored_longhand_proof_names(&mut proof, shorthand_property, longhands);
    proof
}

fn restore_authored_longhand_proof_names(
    proof: &mut ShorthandCombinationProofV0,
    shorthand_property: &str,
    longhands: &[BoxLonghandInputV0],
) {
    proof.shorthand_property = shorthand_property.to_string();
    proof.ordered_longhand_properties = longhands
        .iter()
        .map(|longhand| longhand.property.clone())
        .collect();
}

pub fn summarize_cascade_boundary() -> CascadeBoundarySummary {
    CascadeBoundarySummary {
        product: "omena-cascade.boundary",
        ordering_model: "lexicographicCascadeKey",
        substitution_model: "finiteCustomPropertyLeastFixedPoint",
        least_fixed_point_proof_model: "finite-env monotone custom-property substitution with cycle-to-guaranteed-invalid bottoming and env-size iteration bound",
        ready_surfaces: vec![
            "cascadeKeyOrdering",
            "specificityOrdering",
            "cascadeOutcomeProof",
            "genericCascadeWinner",
            "semanticDesignTokenRanking",
            "queryReadCascadeAtPosition",
            "selectorContextWitness",
            "selectorMatchWitness",
            "cascadeConformanceSeedCorpus",
            "customPropertySubstitution",
            "customPropertyLeastFixedPoint",
            "customPropertyLeastFixedPointProof",
            "customPropertyLeastFixedPointTrace",
            "cycleToGuaranteedInvalid",
            "computedValueResolutionSeed",
            "inheritanceInitialValueSeed",
            "shorthandCombinationProof",
            "supportsStaticEvalWitness",
            "scopeFlattenProof",
            "layerFlattenProof",
            "modalCheckWitnessV0",
            "cascadeMarginSchemaV0",
            "cascadeOrderingAxisSelfCheckCorpus",
            "spinGlassStatisticsV0",
            "grnAttractorBasinV0",
            "diagnosticFrameFootprintV0",
        ],
        not_ready_surfaces: vec!["fullInitialValueTable", "fullWptCascadeCorpus"],
    }
}

#[cfg(test)]
mod tests;
