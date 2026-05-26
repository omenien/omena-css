use omena_refinement_trait::RefinementVerdictV0;
use serde::Serialize;

use crate::{
    CanonicalSmtInputV0, SMT_FEATURE_GATE_V0, SMT_LAYER_MARKER_V0, SMT_SCHEMA_VERSION_V0,
    SmtBackendCheckV0, SmtBackendKindV0, SmtBackendSatResultV0, SmtBackendV0,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum SmtVerdictV0 {
    Accepted,
    Rejected,
    Unknown,
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
    pub canonical_input: CanonicalSmtInputV0,
    pub solver_check: SmtBackendCheckV0,
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

pub(crate) fn cascade_smt_proof_v0<B: SmtBackendV0>(
    canonical_input: CanonicalSmtInputV0,
    backend: &B,
    l1_primitive: &'static str,
    l1_accepted: Option<bool>,
) -> CascadeSMTProofV0 {
    let solver_check = backend.check_canonical_input_v0(&canonical_input);
    CascadeSMTProofV0 {
        schema_version: SMT_SCHEMA_VERSION_V0,
        product: "omena-smt.cascade-proof",
        layer_marker: SMT_LAYER_MARKER_V0,
        feature_gate: SMT_FEATURE_GATE_V0,
        obligation_id: canonical_input.obligation_id.clone(),
        backend: backend.backend_kind(),
        verdict: smt_verdict_from_backend_check_v0(solver_check.sat_result),
        l1_primitive,
        l1_accepted,
        canonical_input,
        solver_check,
        refinement_verdict: None,
        cascade_spec_digest: cascade_spec_digest_v0(),
    }
}

fn smt_verdict_from_backend_check_v0(sat_result: SmtBackendSatResultV0) -> SmtVerdictV0 {
    match sat_result {
        SmtBackendSatResultV0::Sat => SmtVerdictV0::Accepted,
        SmtBackendSatResultV0::Unsat => SmtVerdictV0::Rejected,
        SmtBackendSatResultV0::Unknown => SmtVerdictV0::Unknown,
    }
}

pub const fn cascade_spec_digest_v0() -> [u8; 32] {
    *b"omena-cascade-smt-spec-v0-------"
}
