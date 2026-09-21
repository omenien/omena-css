use std::{fs, path::PathBuf};

use omena_query::{
    OmenaError, OmenaErrorClassV0, OmenaErrorContextV0, OmenaErrorRecoverabilityV0,
    OmenaErrorSeverityV0, OmenaQueryStyleSourceInputV0, OmenaSdkBuildRequestV0,
    OmenaSdkDiagnosticsRequestV0, OmenaSdkErrorEnvelopeV0, OmenaSdkExplainRequestV0,
    OmenaSdkSnapshotRequestV0, OmenaSdkWorkspaceV0,
};
use serde::Deserialize;

use crate::output::{CliOutputMetadataV0, print_json};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct OmenaCliSdkTransportRequestV0 {
    workspace_root: String,
    style_sources: Vec<OmenaQueryStyleSourceInputV0>,
    operation: String,
    #[serde(default)]
    request: serde_json::Value,
    #[serde(default)]
    contract_version: Option<String>,
    #[serde(default)]
    snapshot_binding: Option<omena_query::OmenaWorkspaceSnapshotBindingV0>,
    #[serde(default)]
    snapshot_inputs: Option<omena_query::OmenaWorkspaceSnapshotTransferV0>,
}

pub(crate) fn sdk_request(request_json: PathBuf) -> Result<(), String> {
    let source = fs::read_to_string(&request_json).map_err(|error| {
        encode_error(OmenaError::new(
            OmenaErrorClassV0::Input,
            format!("failed to read {}: {error}", request_json.display()),
            OmenaErrorContextV0 {
                code: "sdk.request-read".to_string(),
                severity: OmenaErrorSeverityV0::Error,
                recoverability: OmenaErrorRecoverabilityV0::UserAction,
                evidence: Vec::new(),
            },
        ))
    })?;
    let transport =
        serde_json::from_str::<OmenaCliSdkTransportRequestV0>(&source).map_err(|error| {
            encode_error(OmenaError::new(
                OmenaErrorClassV0::Input,
                format!("failed to parse SDK transport request: {error}"),
                OmenaErrorContextV0 {
                    code: "sdk.request-parse".to_string(),
                    severity: OmenaErrorSeverityV0::Error,
                    recoverability: OmenaErrorRecoverabilityV0::UserAction,
                    evidence: Vec::new(),
                },
            ))
        })?;
    let response = execute_transport_request(transport).map_err(encode_error)?;
    print_json(
        CliOutputMetadataV0::new("omena-cli.sdk-workflow"),
        &response,
    )
}

fn execute_transport_request(
    transport: OmenaCliSdkTransportRequestV0,
) -> Result<serde_json::Value, OmenaError> {
    if transport.contract_version.is_none()
        && (transport.snapshot_binding.is_some() || transport.snapshot_inputs.is_some())
    {
        return Err(binding_required());
    }
    let workspace = match transport.contract_version.as_deref() {
        None => OmenaSdkWorkspaceV0::open(
            OmenaSdkSnapshotRequestV0 {
                workspace_root: transport.workspace_root,
            },
            transport.style_sources,
        )?,
        Some("1") => {
            let binding = transport.snapshot_binding.ok_or_else(binding_required)?;
            let inputs = transport.snapshot_inputs.ok_or_else(binding_required)?;
            let root_path = crate::paths::cli_file_uri_to_path(&transport.workspace_root)
                .unwrap_or_else(|| PathBuf::from(&transport.workspace_root));
            let utility = omena_query::load_omena_query_workspace_utility_class_intelligence(
                &root_path, None,
            );
            OmenaSdkWorkspaceV0::open_imported_snapshot(
                OmenaSdkSnapshotRequestV0 {
                    workspace_root: transport.workspace_root,
                },
                transport.style_sources,
                inputs,
                binding,
                &utility,
            )?
        }
        Some(_) => return Err(binding_required()),
    };
    let binding = workspace.snapshot_binding().cloned();
    let response = match transport.operation.as_str() {
        "snapshot" => response_value(workspace.snapshot()),
        "exportSnapshot" => workspace.export_snapshot(),
        "query" => workspace
            .execute_query(parse_request(transport.request, "query")?)
            .and_then(response_value),
        "sourceDiagnostics" => workspace
            .execute_snapshot_source_diagnostics(parse_request(
                transport.request,
                "source diagnostics",
            )?)
            .and_then(response_value),
        "diagnostics" => workspace
            .execute_diagnostics(parse_request::<OmenaSdkDiagnosticsRequestV0>(
                transport.request,
                "diagnostics",
            )?)
            .and_then(response_value),
        "build" => workspace
            .execute_build(parse_request::<OmenaSdkBuildRequestV0>(
                transport.request,
                "build",
            )?)
            .and_then(response_value),
        "explain" => workspace
            .execute_explain(parse_request::<OmenaSdkExplainRequestV0>(
                transport.request,
                "explain",
            )?)
            .and_then(response_value),
        _ => Err(OmenaError::new(
            OmenaErrorClassV0::Unsupported,
            format!("unsupported SDK operation {:?}", transport.operation),
            OmenaErrorContextV0 {
                code: "sdk.unsupported-operation".to_string(),
                severity: OmenaErrorSeverityV0::Error,
                recoverability: OmenaErrorRecoverabilityV0::UserAction,
                evidence: Vec::new(),
            },
        )),
    }?;
    match binding {
        Some(snapshot_binding) => response_value(omena_query::OmenaWorkspaceBoundResponseV1 {
            contract_version: "1".to_string(),
            snapshot_binding,
            response,
        }),
        None => Ok(response),
    }
}

fn binding_required() -> OmenaError {
    OmenaError::new(
        OmenaErrorClassV0::Workspace,
        "snapshot contract version 1 requires a complete binding and actual transfer inputs",
        OmenaErrorContextV0 {
            code: "workspace.snapshot-binding-required".to_string(),
            severity: OmenaErrorSeverityV0::Error,
            recoverability: OmenaErrorRecoverabilityV0::Retry,
            evidence: Vec::new(),
        },
    )
}

fn response_value<T: serde::Serialize>(response: T) -> Result<serde_json::Value, OmenaError> {
    serde_json::to_value(response).map_err(|error| {
        OmenaError::new(
            OmenaErrorClassV0::Internal,
            format!("failed to serialize SDK workflow response: {error}"),
            OmenaErrorContextV0 {
                code: "sdk.response-serialization".to_string(),
                severity: OmenaErrorSeverityV0::Error,
                recoverability: OmenaErrorRecoverabilityV0::Retry,
                evidence: Vec::new(),
            },
        )
    })
}

fn parse_request<T: serde::de::DeserializeOwned>(
    value: serde_json::Value,
    operation: &str,
) -> Result<T, OmenaError> {
    serde_json::from_value(value).map_err(|error| {
        OmenaError::new(
            OmenaErrorClassV0::Input,
            format!("failed to parse {operation} request: {error}"),
            OmenaErrorContextV0 {
                code: "sdk.request-parse".to_string(),
                severity: OmenaErrorSeverityV0::Error,
                recoverability: OmenaErrorRecoverabilityV0::UserAction,
                evidence: Vec::new(),
            },
        )
    })
}

fn encode_error(error: OmenaError) -> String {
    serde_json::to_string(&OmenaSdkErrorEnvelopeV0 { error }).unwrap_or_else(|_| {
        "{\"error\":{\"class\":\"internal\",\"message\":\"failed to serialize SDK error\",\"context\":{\"code\":\"sdk.error-serialization\",\"severity\":\"error\",\"recoverability\":\"retry\"}}}".to_string()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn transport(operation: &str, request: serde_json::Value) -> OmenaCliSdkTransportRequestV0 {
        OmenaCliSdkTransportRequestV0 {
            workspace_root: "/workspace".to_string(),
            style_sources: vec![OmenaQueryStyleSourceInputV0 {
                style_path: "src/card.module.css".to_string(),
                style_source: ".card { color: red; }".to_string(),
            }],
            contract_version: None,
            snapshot_binding: None,
            snapshot_inputs: None,
            operation: operation.to_string(),
            request,
        }
    }

    #[test]
    fn executes_typed_query_through_cli_transport() -> Result<(), OmenaError> {
        let response = execute_transport_request(transport(
            "query",
            serde_json::json!({
                "snapshotId": { "value": 1 },
                "queryKind": "styleSummary",
                "input": { "stylePath": "src/card.module.css" }
            }),
        ))?;
        assert_eq!(response["snapshotId"]["value"], 1);
        assert_eq!(response["payload"]["language"], "css");
        Ok(())
    }

    #[test]
    fn preserves_typed_errors_through_cli_transport() -> Result<(), String> {
        let result = execute_transport_request(transport(
            "query",
            serde_json::json!({
                "snapshotId": { "value": 2 },
                "queryKind": "styleSummary",
                "input": { "stylePath": "src/card.module.css" }
            }),
        ));
        let error = result
            .err()
            .ok_or_else(|| "stale snapshot must fail".to_string())?;
        assert_eq!(error.class, OmenaErrorClassV0::Workspace);
        assert_eq!(error.context.code, "workspace.snapshot-mismatch");
        Ok(())
    }
}
