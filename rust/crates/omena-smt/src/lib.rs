//! SMT-encoded cascade verification contracts.
//!
//! The default build provides a deterministic stub backend only. Z3, CVC5, and
//! Bitwuzla are represented as opt-in feature gates without pulling heavy
//! solver dependencies into the default workspace.

use omena_cascade::{
    BoxLonghandInputV0, LayerFlattenInputV0, ScopeFlattenInputV0, StaticSupportsAssumptionV0,
    evaluate_static_supports_condition, prove_box_shorthand_combination,
    prove_layer_flatten_candidate, prove_scope_flatten_candidate,
};
use omena_refinement_trait::RefinementVerdictV0;
use serde::Serialize;

pub mod backend;

pub const SMT_SCHEMA_VERSION_V0: &str = "0";
pub const SMT_LAYER_MARKER_V0: &str = "smt-cascade-verification";
pub const SMT_FEATURE_GATE_V0: &str = "smt-stub";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum SmtBackendKindV0 {
    Stub,
    Z3,
    Cvc5,
    Bitwuzla,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum SmtVerdictV0 {
    Accepted,
    Rejected,
    Unknown,
}

pub trait SmtBackendV0 {
    fn backend_kind(&self) -> SmtBackendKindV0;

    fn quantifier_elimination_tactic(&self) -> Option<&'static str> {
        None
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StubSmtBackendV0;

impl SmtBackendV0 for StubSmtBackendV0 {
    fn backend_kind(&self) -> SmtBackendKindV0 {
        SmtBackendKindV0::Stub
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CascadeTheorySignatureV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub theory_id: &'static str,
    pub cascade_key_encoding: &'static str,
    pub axiom_count: usize,
    pub l1_read_only: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CanonicalSmtInputV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub obligation_id: String,
    pub l1_primitive: &'static str,
    pub canonical_terms: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CascadeSMTProofV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub obligation_id: String,
    pub backend: SmtBackendKindV0,
    pub verdict: SmtVerdictV0,
    pub l1_primitive: &'static str,
    pub l1_accepted: Option<bool>,
    pub refinement_verdict: Option<RefinementVerdictV0>,
    pub cascade_spec_digest: [u8; 32],
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CascadeSMTProofAuditLogV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub obligation_id: String,
    pub backend: SmtBackendKindV0,
    pub solver_latency_us: Option<u64>,
    pub unsat_core_labels: Vec<String>,
}

pub fn cascade_theory_signature_v0() -> CascadeTheorySignatureV0 {
    CascadeTheorySignatureV0 {
        schema_version: SMT_SCHEMA_VERSION_V0,
        product: "omena-smt.cascade-theory-signature",
        layer_marker: SMT_LAYER_MARKER_V0,
        feature_gate: SMT_FEATURE_GATE_V0,
        theory_id: "cascade-smt-theory-v0",
        cascade_key_encoding: "196-bit-bitvector",
        axiom_count: 4,
        l1_read_only: true,
    }
}

pub fn canonical_smt_input_v0(
    obligation_id: impl Into<String>,
    l1_primitive: &'static str,
    canonical_terms: Vec<String>,
) -> CanonicalSmtInputV0 {
    CanonicalSmtInputV0 {
        schema_version: SMT_SCHEMA_VERSION_V0,
        product: "omena-smt.canonical-input",
        layer_marker: SMT_LAYER_MARKER_V0,
        feature_gate: SMT_FEATURE_GATE_V0,
        obligation_id: obligation_id.into(),
        l1_primitive,
        canonical_terms,
    }
}

pub fn smt_prove_box_shorthand_combination_v0<B: SmtBackendV0>(
    shorthand_property: &str,
    longhands: &[BoxLonghandInputV0],
    backend: &B,
) -> CascadeSMTProofV0 {
    let proof = prove_box_shorthand_combination(shorthand_property, longhands);
    cascade_smt_proof_v0(
        "box-shorthand-combination",
        backend,
        "prove_box_shorthand_combination",
        Some(proof.accepted),
    )
}

pub fn smt_prove_scope_flatten_candidate_v0<B: SmtBackendV0>(
    input: ScopeFlattenInputV0,
    backend: &B,
) -> CascadeSMTProofV0 {
    let proof = prove_scope_flatten_candidate(input);
    cascade_smt_proof_v0(
        "scope-flatten-candidate",
        backend,
        "prove_scope_flatten_candidate",
        Some(proof.accepted),
    )
}

pub fn smt_prove_layer_flatten_candidate_v0<B: SmtBackendV0>(
    input: LayerFlattenInputV0,
    backend: &B,
) -> CascadeSMTProofV0 {
    let proof = prove_layer_flatten_candidate(input);
    cascade_smt_proof_v0(
        "layer-flatten-candidate",
        backend,
        "prove_layer_flatten_candidate",
        Some(proof.accepted),
    )
}

pub fn smt_evaluate_static_supports_condition_v0<B: SmtBackendV0>(
    condition: &str,
    assumption: StaticSupportsAssumptionV0,
    backend: &B,
) -> CascadeSMTProofV0 {
    let witness = evaluate_static_supports_condition(condition, assumption);
    let l1_accepted = match witness.verdict {
        omena_cascade::StaticSupportsEvalVerdictV0::AlwaysTrue => Some(true),
        omena_cascade::StaticSupportsEvalVerdictV0::AlwaysFalse => Some(false),
        omena_cascade::StaticSupportsEvalVerdictV0::Unknown => None,
    };
    cascade_smt_proof_v0(
        "static-supports-condition",
        backend,
        "evaluate_static_supports_condition",
        l1_accepted,
    )
}

fn cascade_smt_proof_v0<B: SmtBackendV0>(
    obligation_id: impl Into<String>,
    backend: &B,
    l1_primitive: &'static str,
    l1_accepted: Option<bool>,
) -> CascadeSMTProofV0 {
    CascadeSMTProofV0 {
        schema_version: SMT_SCHEMA_VERSION_V0,
        product: "omena-smt.cascade-proof",
        layer_marker: SMT_LAYER_MARKER_V0,
        feature_gate: SMT_FEATURE_GATE_V0,
        obligation_id: obligation_id.into(),
        backend: backend.backend_kind(),
        verdict: match l1_accepted {
            Some(true) => SmtVerdictV0::Accepted,
            Some(false) => SmtVerdictV0::Rejected,
            None => SmtVerdictV0::Unknown,
        },
        l1_primitive,
        l1_accepted,
        refinement_verdict: None,
        cascade_spec_digest: cascade_spec_digest_v0(),
    }
}

pub const fn cascade_spec_digest_v0() -> [u8; 32] {
    *b"omena-cascade-smt-spec-v0-------"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_stub_backend_matches_l1_proof_verdict() {
        let backend = StubSmtBackendV0;
        let proof = smt_prove_box_shorthand_combination_v0(
            "margin",
            &[
                BoxLonghandInputV0 {
                    property: "margin-top".to_string(),
                    value: "1px".to_string(),
                    important: false,
                    source_order: 1,
                },
                BoxLonghandInputV0 {
                    property: "margin-right".to_string(),
                    value: "1px".to_string(),
                    important: false,
                    source_order: 2,
                },
                BoxLonghandInputV0 {
                    property: "margin-bottom".to_string(),
                    value: "1px".to_string(),
                    important: false,
                    source_order: 3,
                },
                BoxLonghandInputV0 {
                    property: "margin-left".to_string(),
                    value: "1px".to_string(),
                    important: false,
                    source_order: 4,
                },
            ],
            &backend,
        );
        assert_eq!(proof.schema_version, "0");
        assert_eq!(proof.verdict, SmtVerdictV0::Accepted);
        assert_eq!(proof.backend, SmtBackendKindV0::Stub);
    }
}
