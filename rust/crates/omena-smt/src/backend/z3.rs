use serde::Serialize;
use z3::{SatResult, Solver};

use super::{SmtBackendCheckV0, SmtBackendKindV0, SmtBackendSatResultV0, SmtBackendV0};
use crate::{SMT_LAYER_MARKER_V0, SMT_SCHEMA_VERSION_V0, encoder::canonical_input_has_unknown_v0};

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
            feature_gate: "smt-z3",
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

    fn check_canonical_input_v0(&self, input: &crate::CanonicalSmtInputV0) -> SmtBackendCheckV0 {
        let sat_result = if canonical_input_has_unknown_v0(input) {
            SmtBackendSatResultV0::Unknown
        } else {
            let solver = Solver::new();
            solver.from_string(input.smtlib2_script.as_str());
            match solver.check() {
                SatResult::Sat => SmtBackendSatResultV0::Sat,
                SatResult::Unsat => SmtBackendSatResultV0::Unsat,
                SatResult::Unknown => SmtBackendSatResultV0::Unknown,
            }
        };
        SmtBackendCheckV0 {
            schema_version: SMT_SCHEMA_VERSION_V0,
            product: "omena-smt.backend-check.z3",
            layer_marker: SMT_LAYER_MARKER_V0,
            feature_gate: self.feature_gate,
            backend: self.backend_kind(),
            obligation_id: input.obligation_id.clone(),
            formula_count: input.canonical_terms.len(),
            sat_result,
            model_available: matches!(sat_result, SmtBackendSatResultV0::Sat),
        }
    }
}
