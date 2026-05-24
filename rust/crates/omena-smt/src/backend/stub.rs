use serde::Serialize;

use super::{SmtBackendKindV0, SmtBackendV0};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StubSmtBackendV0;

impl SmtBackendV0 for StubSmtBackendV0 {
    fn backend_kind(&self) -> SmtBackendKindV0 {
        SmtBackendKindV0::Stub
    }
}
