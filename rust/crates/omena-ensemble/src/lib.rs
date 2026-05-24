//! Replica-overlap ensemble contracts over Omena cross-file facts.
//!
//! The crate is a leaf M4-beta lane: it consumes cascade outcomes and query
//! graph summaries read-only, then emits additive V0 contracts.

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
