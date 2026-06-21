//! Serializable parser span and range records.
//!
//! Parser summaries use these byte/line/column records at crate boundaries
//! instead of exposing cstree-specific range types.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ParserByteSpanV0 {
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ParserPositionV0 {
    pub line: usize,
    pub character: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ParserRangeV0 {
    pub start: ParserPositionV0,
    pub end: ParserPositionV0,
}
