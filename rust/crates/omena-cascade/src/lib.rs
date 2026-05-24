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
mod model;
mod proofs;
mod ranking;
mod refinement;
mod selector;
mod statistics;

pub use computed_value::*;
pub use conformance::*;
pub use custom_property::*;
pub use frame_footprint::*;
pub use fuzz::*;
pub use grn::*;
pub use model::*;
pub use proofs::*;
pub use ranking::*;
pub use refinement::*;
pub use selector::*;
pub use statistics::*;

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
