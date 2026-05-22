//! Shared Rust fixture and scenario substrate for omena-css tests.
//!
//! M4 uses this crate to move reusable fixture grammar out of product-specific
//! harnesses. The M4 substrate also locks scenario macros and snapshot
//! governance on top of the same `cme-fixture-v0` parser.

mod boundary;
pub mod fixture;
pub mod scenario;
pub mod snapshot;

pub use boundary::*;
pub use fixture::*;
pub use scenario::*;
pub use snapshot::*;
