//! Replica-overlap ensemble contracts over Omena cross-file facts.
//!
//! The crate is a leaf M4-beta lane: it consumes cascade outcomes and a caller-
//! supplied module graph read-only, then emits additive V0 contracts. Keeping the
//! graph an input (rather than reaching into `omena-query`) leaves the crate a DAG
//! leaf, so the real query/LSP diagnostic path can depend on it without a cycle.
//!
//! claim_level: lab-only cross-file consistency hint substrate, not a default
//! product decision mechanism.

pub mod types;

#[cfg(feature = "replica-ensemble")]
pub mod overlap;
#[cfg(feature = "replica-ensemble")]
pub mod sbm;

pub use types::*;

#[cfg(feature = "replica-ensemble")]
pub use overlap::*;
#[cfg(feature = "replica-ensemble")]
pub use sbm::*;

#[cfg(all(test, feature = "replica-ensemble"))]
mod tests;
