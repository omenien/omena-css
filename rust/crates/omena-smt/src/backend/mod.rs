use serde::Serialize;

use crate::CanonicalSmtInputV0;

#[cfg(feature = "smt-z3")]
pub mod z3;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum SmtBackendKindV0 {
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

    fn check_canonical_input_v0(&self, input: &CanonicalSmtInputV0) -> SmtBackendCheckV0;
}
