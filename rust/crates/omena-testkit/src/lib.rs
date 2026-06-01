//! Shared Rust fixture and scenario substrate for omena-css tests.
//!
//! M4 uses this crate to move reusable fixture grammar out of product-specific
//! harnesses. The M4 substrate also locks scenario macros and snapshot
//! governance on top of the same `omena-fixture-v0` parser.

mod boundary;
pub mod fixture;
pub mod fixture_eval;
pub mod scenario;
pub mod snapshot;

pub use boundary::*;
pub use fixture::*;
pub use fixture_eval::*;
pub use scenario::*;
pub use snapshot::*;
