use serde::Serialize;

mod stub;

#[cfg(feature = "smt-bitwuzla")]
pub mod bitwuzla;
#[cfg(feature = "smt-cvc5")]
pub mod cvc5;
#[cfg(feature = "smt-z3")]
pub mod z3;

pub use stub::StubSmtBackendV0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum SmtBackendKindV0 {
    Stub,
    Z3,
    Cvc5,
    Bitwuzla,
}

pub trait SmtBackendV0 {
    fn backend_kind(&self) -> SmtBackendKindV0;

    fn quantifier_elimination_tactic(&self) -> Option<&'static str> {
        None
    }
}
