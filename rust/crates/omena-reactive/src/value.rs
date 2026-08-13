use std::collections::{BTreeMap, BTreeSet};

/// Values carried by the static graph.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum ReactiveValueV0 {
    Unit,
    Bool(bool),
    Counter(u64),
    Text(String),
    StringSet(BTreeSet<String>),
    TextMap(BTreeMap<String, String>),
    Tuple(Vec<Self>),
    Digest([u8; 32]),
}

/// A node-local failure. Unavailability propagates as data and never poisons
/// unrelated nodes or the graph itself.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub struct ReactiveUnavailableV0 {
    pub code: String,
    pub detail: String,
}

impl ReactiveUnavailableV0 {
    pub fn new(code: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            detail: detail.into(),
        }
    }

    pub(crate) fn pending() -> Self {
        Self::new(
            "pendingInitialComputation",
            "the node has not completed its first necessary computation",
        )
    }
}

/// The complete state of one node.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum ReactiveStateV0 {
    Available(ReactiveValueV0),
    Unavailable(ReactiveUnavailableV0),
}

impl ReactiveStateV0 {
    pub fn available(value: ReactiveValueV0) -> Self {
        Self::Available(value)
    }

    pub fn unavailable(code: impl Into<String>, detail: impl Into<String>) -> Self {
        Self::Unavailable(ReactiveUnavailableV0::new(code, detail))
    }

    pub(crate) fn pending() -> Self {
        Self::Unavailable(ReactiveUnavailableV0::pending())
    }
}
