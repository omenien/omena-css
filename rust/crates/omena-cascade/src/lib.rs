//! Cascade-formal substrate for the Omena CSS track.
//!
//! The crate starts with the load-bearing algebra from the research plan:
//! lexicographic cascade keys, specificity, provenance proofs, and a finite
//! custom-property substitution function with explicit cycle handling.

mod computed_value;
mod conformance;
mod custom_property;
mod frame_footprint;
mod fuzz;
mod grn;
mod modal;
mod model;
mod origin;
mod proofs;
mod property_metadata;
mod property_metadata_idl_generated;
mod ranking;
mod refinement;
mod selector;
mod shorthand_authority;
mod statistics;

pub use computed_value::*;
pub use conformance::*;
pub use custom_property::*;
pub use frame_footprint::*;
pub use fuzz::*;
pub use grn::*;
pub use modal::*;
pub use model::*;
pub use origin::*;
pub use proofs::{
    evaluate_static_supports_condition, prove_box_shorthand_combination,
    prove_layer_flatten_candidate, prove_longhand_merge,
};
pub use property_metadata::*;
pub use property_metadata_idl_generated::{
    CSS_PROPERTY_METADATA_RECORDS_V1, CSS_PROPERTY_METADATA_V1, CssCustomPropertyPolicyStaticV1,
    CssCustomPropertyPolicyV1Json, CssPropertyMetadataRecordStaticV1,
    CssPropertyMetadataRecordV1Json, CssPropertyMetadataSourceStaticV1,
    CssPropertyMetadataSourceV1Json, CssPropertyMetadataStaticV1, CssPropertyMetadataV1Json,
};
pub use ranking::*;
pub use refinement::*;
pub use selector::*;
pub use shorthand_authority::*;
pub use statistics::*;

pub fn prove_scope_flatten_candidate(mut input: ScopeFlattenInputV0) -> ScopeFlattenProofV0 {
    if omena_syntax::css_keyword(input.root_selector.trim()).equals(":root") {
        input.root_selector = ":root".to_string();
    }
    proofs::prove_scope_flatten_candidate(input)
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
            "wptCascadeSeedCorpus",
            "spinGlassStatisticsV0",
            "grnAttractorBasinV0",
            "diagnosticFrameFootprintV0",
        ],
        not_ready_surfaces: vec!["fullInitialValueTable", "fullWptCascadeCorpus"],
    }
}

#[cfg(test)]
mod tests;
