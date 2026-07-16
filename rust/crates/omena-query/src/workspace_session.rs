use crate::{
    OmenaError, OmenaErrorClassV0, OmenaErrorContextV0, OmenaErrorRecoverabilityV0,
    OmenaErrorSeverityV0, OmenaSdkResponsePartitionV0, OmenaWorkspaceSessionHandshakeRequestV0,
    OmenaWorkspaceSessionHandshakeResponseV0, OmenaWorkspaceSessionResponseV0,
    OmenaWorkspaceSnapshotIdV0,
};

pub const OMENA_WORKSPACE_SESSION_PROTOCOL_VERSION_V0: &str = "0";

const WORKSPACE_SESSION_CAPABILITIES_V0: &[&str] = &[
    "cancel",
    "check",
    "diagnostics",
    "explain",
    "format",
    "lint",
    "replaceStyleSources",
    "shutdown",
];

pub fn negotiate_omena_workspace_session_v0(
    request: &OmenaWorkspaceSessionHandshakeRequestV0,
    snapshot_id: OmenaWorkspaceSnapshotIdV0,
) -> Result<OmenaWorkspaceSessionHandshakeResponseV0, OmenaError> {
    if request.protocol_version != OMENA_WORKSPACE_SESSION_PROTOCOL_VERSION_V0 {
        return Err(session_error(
            OmenaErrorClassV0::Unsupported,
            format!(
                "unsupported workspace session protocol version {:?}",
                request.protocol_version
            ),
            "workspace-session.protocol-version",
            OmenaErrorRecoverabilityV0::UserAction,
        ));
    }
    if request.workspace_root.trim().is_empty() {
        return Err(session_error(
            OmenaErrorClassV0::Input,
            "workspace session root must not be empty",
            "workspace-session.empty-root",
            OmenaErrorRecoverabilityV0::UserAction,
        ));
    }
    if request.limits.deadline_ms == 0 || request.limits.max_response_bytes == 0 {
        return Err(session_error(
            OmenaErrorClassV0::Input,
            "workspace session limits must be positive",
            "workspace-session.invalid-limits",
            OmenaErrorRecoverabilityV0::UserAction,
        ));
    }

    Ok(OmenaWorkspaceSessionHandshakeResponseV0 {
        protocol_version: OMENA_WORKSPACE_SESSION_PROTOCOL_VERSION_V0.to_string(),
        snapshot_id,
        partition: OmenaSdkResponsePartitionV0::Public,
        workspace_root: request.workspace_root.clone(),
        config_content_digest: request.config_content_digest.clone(),
        capabilities: WORKSPACE_SESSION_CAPABILITIES_V0
            .iter()
            .map(|capability| (*capability).to_string())
            .collect(),
    })
}

pub fn omena_workspace_session_success_v0(
    request_id: impl Into<String>,
    snapshot_id: OmenaWorkspaceSnapshotIdV0,
    payload: serde_json::Value,
) -> OmenaWorkspaceSessionResponseV0 {
    OmenaWorkspaceSessionResponseV0 {
        request_id: request_id.into(),
        protocol_version: OMENA_WORKSPACE_SESSION_PROTOCOL_VERSION_V0.to_string(),
        snapshot_id,
        partition: OmenaSdkResponsePartitionV0::Public,
        ok: true,
        payload: Some(payload),
        error: None,
    }
}

pub fn omena_workspace_session_failure_v0(
    request_id: impl Into<String>,
    snapshot_id: OmenaWorkspaceSnapshotIdV0,
    error: OmenaError,
) -> OmenaWorkspaceSessionResponseV0 {
    OmenaWorkspaceSessionResponseV0 {
        request_id: request_id.into(),
        protocol_version: OMENA_WORKSPACE_SESSION_PROTOCOL_VERSION_V0.to_string(),
        snapshot_id,
        partition: OmenaSdkResponsePartitionV0::Public,
        ok: false,
        payload: None,
        error: Some(error),
    }
}

fn session_error(
    class: OmenaErrorClassV0,
    message: impl Into<String>,
    code: &str,
    recoverability: OmenaErrorRecoverabilityV0,
) -> OmenaError {
    OmenaError::new(
        class,
        message,
        OmenaErrorContextV0 {
            code: code.to_string(),
            severity: OmenaErrorSeverityV0::Error,
            recoverability,
            evidence: Vec::new(),
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        IncrementalRevisionV0, OmenaQueryStyleSourceInputV0, OmenaWorkspaceSessionLimitsV0,
    };

    fn request() -> OmenaWorkspaceSessionHandshakeRequestV0 {
        OmenaWorkspaceSessionHandshakeRequestV0 {
            protocol_version: "0".to_string(),
            workspace_root: "/workspace".to_string(),
            config_content_digest: Some("digest".to_string()),
            style_sources: vec![OmenaQueryStyleSourceInputV0 {
                style_path: "/workspace/app.css".to_string(),
                style_source: ".app { color: red; }".to_string(),
            }],
            limits: OmenaWorkspaceSessionLimitsV0 {
                deadline_ms: 1_000,
                max_response_bytes: 1_048_576,
            },
        }
    }

    #[test]
    fn handshake_reuses_snapshot_identity_and_generated_wire_shape() -> Result<(), String> {
        let snapshot_id =
            OmenaWorkspaceSnapshotIdV0::from_revision(IncrementalRevisionV0 { value: 7 });
        let response = negotiate_omena_workspace_session_v0(&request(), snapshot_id)
            .map_err(|error| error.to_string())?;
        assert_eq!(response.snapshot_id, snapshot_id);
        assert_eq!(response.config_content_digest.as_deref(), Some("digest"));
        assert!(
            response
                .capabilities
                .iter()
                .any(|capability| capability == "diagnostics")
        );
        assert!(
            response
                .capabilities
                .iter()
                .any(|capability| capability == "format")
        );

        let wire = serde_json::to_value(&response).map_err(|error| error.to_string())?;
        assert_eq!(wire["protocolVersion"], "0");
        assert_eq!(wire["snapshotId"]["value"], 7);
        Ok(())
    }

    #[test]
    fn handshake_rejects_protocol_drift_and_unbounded_requests() -> Result<(), String> {
        let snapshot_id =
            OmenaWorkspaceSnapshotIdV0::from_revision(IncrementalRevisionV0 { value: 1 });
        let mut unsupported = request();
        unsupported.protocol_version = "1".to_string();
        let Err(protocol_error) = negotiate_omena_workspace_session_v0(&unsupported, snapshot_id)
        else {
            return Err("protocol drift must fail".to_string());
        };
        assert_eq!(
            protocol_error.context.code,
            "workspace-session.protocol-version"
        );

        let mut unbounded = request();
        unbounded.limits.max_response_bytes = 0;
        let Err(limit_error) = negotiate_omena_workspace_session_v0(&unbounded, snapshot_id) else {
            return Err("unbounded response must fail".to_string());
        };
        assert_eq!(limit_error.context.code, "workspace-session.invalid-limits");
        Ok(())
    }
}
