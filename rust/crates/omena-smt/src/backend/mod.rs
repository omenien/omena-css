use serde::Serialize;

use crate::{
    CanonicalSmtInputV0, SMT_FEATURE_GATE_V0, SMT_LAYER_MARKER_V0, SMT_SCHEMA_VERSION_V0,
    encoder::{canonical_input_has_unknown_v0, canonical_requirement_value_v0},
};

mod stub;

#[cfg(feature = "smt-z3")]
pub mod z3;

pub use stub::StubSmtBackendV0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum SmtBackendKindV0 {
    Stub,
    Z3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum SmtBackendSatResultV0 {
    Sat,
    Unsat,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SmtBackendCheckV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub backend: SmtBackendKindV0,
    pub obligation_id: String,
    pub formula_count: usize,
    pub sat_result: SmtBackendSatResultV0,
    pub model_available: bool,
}

pub trait SmtBackendV0 {
    fn backend_kind(&self) -> SmtBackendKindV0;

    fn quantifier_elimination_tactic(&self) -> Option<&'static str> {
        None
    }

    fn check_canonical_input_v0(&self, input: &CanonicalSmtInputV0) -> SmtBackendCheckV0 {
        let sat_result = if canonical_input_has_unknown_v0(input) {
            SmtBackendSatResultV0::Unknown
        } else if input
            .canonical_terms
            .iter()
            .all(|term| canonical_requirement_value_v0(term).unwrap_or(true))
        {
            SmtBackendSatResultV0::Sat
        } else {
            SmtBackendSatResultV0::Unsat
        };
        SmtBackendCheckV0 {
            schema_version: SMT_SCHEMA_VERSION_V0,
            product: "omena-smt.backend-check",
            layer_marker: SMT_LAYER_MARKER_V0,
            feature_gate: SMT_FEATURE_GATE_V0,
            backend: self.backend_kind(),
            obligation_id: input.obligation_id.clone(),
            formula_count: input.canonical_terms.len(),
            sat_result,
            model_available: matches!(sat_result, SmtBackendSatResultV0::Sat),
        }
    }
}
