use serde::Serialize;

use super::{SmtBackendKindV0, SmtBackendV0};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Z3SmtBackendV0;

impl SmtBackendV0 for Z3SmtBackendV0 {
    fn backend_kind(&self) -> SmtBackendKindV0 {
        SmtBackendKindV0::Z3
    }

    fn quantifier_elimination_tactic(&self) -> Option<&'static str> {
        Some("bit-blast")
    }
}
