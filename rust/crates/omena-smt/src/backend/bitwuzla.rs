use serde::Serialize;

use super::{SmtBackendKindV0, SmtBackendV0};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BitwuzlaSmtBackendV0;

impl SmtBackendV0 for BitwuzlaSmtBackendV0 {
    fn backend_kind(&self) -> SmtBackendKindV0 {
        SmtBackendKindV0::Bitwuzla
    }

    fn quantifier_elimination_tactic(&self) -> Option<&'static str> {
        Some("196-bit-cascade-key-bitvector")
    }
}
