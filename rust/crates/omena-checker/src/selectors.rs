use std::fmt;

use serde::{Deserialize, Serialize};

/// Selector text before the parser/query boundary has expanded nesting.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize)]
#[serde(transparent)]
pub struct RawSelector(String);

impl RawSelector {
    pub fn new(selector: impl Into<String>) -> Self {
        Self(selector.into())
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    pub fn into_string(self) -> String {
        self.0
    }
}

impl fmt::Display for RawSelector {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

impl AsRef<str> for RawSelector {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

/// Selector text after the parser/query boundary has expanded nesting and
/// normalized the selector enough for cascade-context comparison.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize)]
#[serde(transparent)]
pub struct CanonicalSelector(String);

impl CanonicalSelector {
    pub fn from_canonical(selector: impl Into<String>) -> Self {
        let selector = selector.into();
        debug_assert!(
            !selector.contains('&'),
            "canonical selector must not contain an unexpanded ampersand: {selector:?}"
        );
        Self(selector)
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    pub fn into_string(self) -> String {
        self.0
    }
}

impl fmt::Display for CanonicalSelector {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

impl AsRef<str> for CanonicalSelector {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
