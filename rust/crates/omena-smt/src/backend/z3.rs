use serde::Serialize;

use super::{SmtBackendKindV0, SmtBackendV0};
use crate::{SMT_FEATURE_GATE_V0, SMT_LAYER_MARKER_V0, SMT_SCHEMA_VERSION_V0};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Z3SmtBackendV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
}

impl Default for Z3SmtBackendV0 {
    fn default() -> Self {
        Self {
            schema_version: SMT_SCHEMA_VERSION_V0,
            product: "omena-smt.backend.z3",
            layer_marker: SMT_LAYER_MARKER_V0,
            feature_gate: SMT_FEATURE_GATE_V0,
        }
    }
}

impl SmtBackendV0 for Z3SmtBackendV0 {
    fn backend_kind(&self) -> SmtBackendKindV0 {
        SmtBackendKindV0::Z3
    }

    fn quantifier_elimination_tactic(&self) -> Option<&'static str> {
        Some("bit-blast")
    }
}
