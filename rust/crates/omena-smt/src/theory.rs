use serde::Serialize;

use crate::{SMT_FEATURE_GATE_V0, SMT_LAYER_MARKER_V0, SMT_SCHEMA_VERSION_V0};

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
