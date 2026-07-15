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
    let workspace = OmenaSdkWorkspaceV0::open(
        OmenaSdkSnapshotRequestV0 {
            workspace_root: transport.workspace_root,
        },
        transport.style_sources,
    )?;
    match transport.operation.as_str() {
        "snapshot" => response_value(workspace.snapshot()),
        "query" => workspace
            .execute_query(parse_request(transport.request, "query")?)
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
            },
        )),
    }
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
    fn preserves_typed_errors_through_cli_transport() {
        let error = execute_transport_request(transport(
            "query",
            serde_json::json!({
                "snapshotId": { "value": 2 },
                "queryKind": "styleSummary",
                "input": { "stylePath": "src/card.module.css" }
            }),
        ))
        .expect_err("stale snapshot must fail");
        assert_eq!(error.class, OmenaErrorClassV0::Workspace);
        assert_eq!(error.context.code, "workspace.snapshot-mismatch");
    }
}
