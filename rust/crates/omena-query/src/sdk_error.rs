use crate::{
    OmenaErrorClassV0, OmenaErrorContextV0, OmenaErrorEvidenceReferenceV0,
    OmenaErrorRecoverabilityV0, OmenaErrorSeverityV0, OmenaErrorV0,
};
use omena_evidence_graph::{EvidenceGraphV0, EvidenceNodeKeyV0};
use std::fmt;

pub type OmenaError = OmenaErrorV0;

impl OmenaErrorV0 {
    pub fn new(
        class: OmenaErrorClassV0,
        message: impl Into<String>,
        context: OmenaErrorContextV0,
    ) -> Self {
        Self {
            class,
            message: message.into(),
            context,
        }
    }

    pub fn unknown(message: impl Into<String>, code: impl Into<String>) -> Self {
        Self::new(
            OmenaErrorClassV0::Unknown,
            message,
            OmenaErrorContextV0 {
                code: code.into(),
                severity: OmenaErrorSeverityV0::Error,
                recoverability: OmenaErrorRecoverabilityV0::UserAction,
                evidence: Vec::new(),
            },
        )
    }

    pub fn with_evidence_graph_nodes(
        mut self,
        graph: &EvidenceGraphV0,
        keys: impl IntoIterator<Item = EvidenceNodeKeyV0>,
    ) -> Result<Self, OmenaErrorEvidenceBindingErrorV0> {
        let mut keys = keys.into_iter().collect::<Vec<_>>();
        keys.sort();
        keys.dedup();
        for key in &keys {
            if !graph.nodes.iter().any(|node| &node.key == key) {
                return Err(OmenaErrorEvidenceBindingErrorV0::MissingNode(key.clone()));
            }
        }
        self.context.evidence = keys
            .into_iter()
            .map(|key| OmenaErrorEvidenceReferenceV0 {
                query_identity: key.query_identity,
                input_identity: key.input_identity,
            })
            .collect();
        Ok(self)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OmenaErrorEvidenceBindingErrorV0 {
    MissingNode(EvidenceNodeKeyV0),
}

impl fmt::Display for OmenaErrorEvidenceBindingErrorV0 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingNode(key) => write!(
                formatter,
                "evidence graph does not contain query '{}' input '{}'",
                key.query_identity, key.input_identity
            ),
        }
    }
}

impl std::error::Error for OmenaErrorEvidenceBindingErrorV0 {}

impl fmt::Display for OmenaErrorV0 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.context.code, self.message)
    }
}

impl std::error::Error for OmenaErrorV0 {}

pub fn omena_error_from_boundary_encoding(
    kind: &str,
    message: impl Into<String>,
    operation: &str,
) -> OmenaError {
    let (class, recoverability) = match kind {
        "parse-error" => (
            OmenaErrorClassV0::Input,
            OmenaErrorRecoverabilityV0::UserAction,
        ),
        "serialize-error" => (
            OmenaErrorClassV0::Internal,
            OmenaErrorRecoverabilityV0::Retry,
        ),
        "unsupported-mode" => (
            OmenaErrorClassV0::Unsupported,
            OmenaErrorRecoverabilityV0::UserAction,
        ),
        _ => (
            OmenaErrorClassV0::Unknown,
            OmenaErrorRecoverabilityV0::UserAction,
        ),
    };
    OmenaErrorV0::new(
        class,
        message,
        OmenaErrorContextV0 {
            code: format!("boundary.{operation}.{kind}"),
            severity: OmenaErrorSeverityV0::Error,
            recoverability,
            evidence: Vec::new(),
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use omena_evidence_graph::{
        EvidenceDemandEdgeV0, EvidenceNodeSeedV0, GuaranteeKindV0,
        build_evidence_graph_from_edges_v0,
    };

    #[test]
    fn boundary_error_encodings_map_to_unified_classes() {
        assert_eq!(
            omena_error_from_boundary_encoding("parse-error", "bad input", "build").class,
            OmenaErrorClassV0::Input
        );
        assert_eq!(
            omena_error_from_boundary_encoding("serialize-error", "encoding", "check").class,
            OmenaErrorClassV0::Internal
        );
        assert_eq!(
            omena_error_from_boundary_encoding("unsupported-mode", "mode", "build").class,
            OmenaErrorClassV0::Unsupported
        );
        assert_eq!(
            omena_error_from_boundary_encoding("future-kind", "unknown", "query").class,
            OmenaErrorClassV0::Unknown
        );
    }

    #[test]
    fn evidence_binding_requires_a_graph_node_and_is_absent_by_default() {
        let key = EvidenceNodeKeyV0::new("sdkDiagnostics", "src/card.css");
        let graph = build_evidence_graph_from_edges_v0(
            [EvidenceNodeSeedV0::new(
                key.clone(),
                vec!["fixture".to_string()],
                GuaranteeKindV0::for_label_less_family(),
            )],
            [EvidenceDemandEdgeV0::new(
                "sdkError",
                key.clone(),
                "supportsErrorContext",
            )],
        )
        .unwrap();
        let error = OmenaError::unknown("analysis failed", "analysis.failed");
        let default_json = serde_json::to_value(&error).unwrap();
        assert!(default_json["context"].get("evidence").is_none());

        let bound = error
            .with_evidence_graph_nodes(&graph, [key.clone()])
            .unwrap();
        assert_eq!(
            serde_json::to_value(bound).unwrap()["context"]["evidence"][0],
            serde_json::json!({
                "queryIdentity": "sdkDiagnostics",
                "inputIdentity": "src/card.css",
            }),
        );

        let missing = OmenaError::unknown("analysis failed", "analysis.failed")
            .with_evidence_graph_nodes(
                &graph,
                [EvidenceNodeKeyV0::new("sdkDiagnostics", "src/missing.css")],
            );
        assert!(matches!(
            missing,
            Err(OmenaErrorEvidenceBindingErrorV0::MissingNode(_))
        ));
    }
}
