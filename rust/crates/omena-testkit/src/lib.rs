//! Shared Rust fixture and scenario substrate for omena-css tests.
//!
//! This crate moves reusable fixture grammar out of product-specific harnesses
//! and locks scenario macros, snapshot governance, and property-based parser
//! checks on top of the same `omena-fixture-v0` substrate.

mod boundary;
pub mod fixture;
pub mod fixture_eval;
pub mod scenario;
pub mod snapshot;

#[cfg(test)]
mod property;

pub use boundary::*;
pub use fixture::*;
pub use fixture_eval::*;
pub use scenario::*;
pub use snapshot::*;
