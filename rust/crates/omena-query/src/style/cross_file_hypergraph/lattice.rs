use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase", tag = "kind")]
pub enum SummaryLatticeElementV0 {
    Reachable {
        schema_version: &'static str,
        layer_marker: &'static str,
        feature_gate: &'static str,
    },
    Unresolved {
        schema_version: &'static str,
        layer_marker: &'static str,
        feature_gate: &'static str,
    },
}

impl SummaryLatticeElementV0 {
    pub const fn from_status(status: &str) -> Self {
        match status.as_bytes() {
            b"resolved" | b"reachable" => Self::Reachable {
                schema_version: "0",
                layer_marker: "hypergraph-ifds",
                feature_gate: "hypergraph-ifds",
            },
            _ => Self::Unresolved {
                schema_version: "0",
                layer_marker: "hypergraph-ifds",
                feature_gate: "hypergraph-ifds",
            },
        }
    }

    pub const fn is_reachable(&self) -> bool {
        matches!(self, Self::Reachable { .. })
    }
}
