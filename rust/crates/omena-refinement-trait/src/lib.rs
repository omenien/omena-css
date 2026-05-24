//! Interface-only refinement predicate contracts.
//!
//! This crate intentionally has no dependency on cascade or SMT crates. It is
//! the cycle breaker shared by `omena-cascade`, `omena-refinement`, and
//! `omena-smt`.

use serde::Serialize;

pub const REFINEMENT_SCHEMA_VERSION_V0: &str = "0";
pub const REFINEMENT_LAYER_MARKER_V0: &str = "refinement-cascade";
pub const REFINEMENT_FEATURE_GATE_V0: &str = "refinement-type-system";

pub trait PropertyIndexV0 {
    const PROPERTY_NAME: &'static str;
}

pub trait RefinementPredicateV0 {
    const PREDICATE_ID: &'static str;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum RefinementVerdictV0 {
    SatisfiedAll,
    SatisfiedSome,
    Unsatisfiable,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RefinementProvenanceV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub source: &'static str,
    pub legacy_proof_primitive: Option<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RefinementWitnessV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub predicate_id: &'static str,
    pub verdict: RefinementVerdictV0,
    pub provenance: Vec<RefinementProvenanceV0>,
    pub legacy_proofs_byte_untouched: bool,
}

pub fn refinement_provenance_v0(
    source: &'static str,
    legacy_proof_primitive: Option<&'static str>,
) -> RefinementProvenanceV0 {
    RefinementProvenanceV0 {
        schema_version: REFINEMENT_SCHEMA_VERSION_V0,
        product: "omena-refinement-trait.provenance",
        layer_marker: REFINEMENT_LAYER_MARKER_V0,
        feature_gate: REFINEMENT_FEATURE_GATE_V0,
        source,
        legacy_proof_primitive,
    }
}

pub fn refinement_witness_v0(
    predicate_id: &'static str,
    verdict: RefinementVerdictV0,
    provenance: Vec<RefinementProvenanceV0>,
) -> RefinementWitnessV0 {
    RefinementWitnessV0 {
        schema_version: REFINEMENT_SCHEMA_VERSION_V0,
        product: "omena-refinement-trait.witness",
        layer_marker: REFINEMENT_LAYER_MARKER_V0,
        feature_gate: REFINEMENT_FEATURE_GATE_V0,
        predicate_id,
        verdict,
        provenance,
        legacy_proofs_byte_untouched: true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interface_contracts_keep_schema_zero() {
        let witness = refinement_witness_v0(
            "top",
            RefinementVerdictV0::SatisfiedAll,
            vec![refinement_provenance_v0(
                "cascade-refinement",
                Some("evaluate_static_supports_condition"),
            )],
        );
        assert_eq!(witness.schema_version, "0");
        assert_eq!(witness.layer_marker, "refinement-cascade");
        assert!(witness.legacy_proofs_byte_untouched);
    }
}
