use serde::Serialize;

use super::{SmtBackendKindV0, SmtBackendV0};
use crate::{SMT_FEATURE_GATE_V0, SMT_LAYER_MARKER_V0, SMT_SCHEMA_VERSION_V0};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Cvc5SmtBackendV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
}

impl Default for Cvc5SmtBackendV0 {
    fn default() -> Self {
        Self {
            schema_version: SMT_SCHEMA_VERSION_V0,
            product: "omena-smt.backend.cvc5",
            layer_marker: SMT_LAYER_MARKER_V0,
            feature_gate: SMT_FEATURE_GATE_V0,
        }
    }
}

impl SmtBackendV0 for Cvc5SmtBackendV0 {
    fn backend_kind(&self) -> SmtBackendKindV0 {
        SmtBackendKindV0::Cvc5
    }
}
