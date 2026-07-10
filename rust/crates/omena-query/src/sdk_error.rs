use crate::{
    OmenaErrorClassV0, OmenaErrorContextV0, OmenaErrorRecoverabilityV0, OmenaErrorSeverityV0,
    OmenaErrorV0,
};
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
            },
        )
    }
}

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
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
