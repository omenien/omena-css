use crate::protocol::workspace_folder_uri_equivalent;
use crate::style_diagnostics_snapshot::current_style_workspace_snapshot_id;
use crate::{LspShellState, is_style_document_uri};
use omena_query::{
    OmenaError, OmenaErrorClassV0, OmenaErrorContextV0, OmenaErrorRecoverabilityV0,
    OmenaErrorSeverityV0, OmenaQueryStyleSourceInputV0, OmenaSdkSnapshotRequestV0,
    OmenaSdkWorkspaceV0, OmenaWorkspaceSnapshotIdV0,
};
use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LspSdkWorkflowRequestV0 {
    workspace_root: String,
    operation: String,
    request: Value,
}

pub(crate) fn resolve_lsp_sdk_workflow(
    state: &LspShellState,
    params: Option<&Value>,
) -> Result<Value, OmenaError> {
    let envelope = parse_request::<LspSdkWorkflowRequestV0>(
        params.cloned().unwrap_or(Value::Null),
        "workflow envelope",
    )?;
    let snapshot_id = current_style_workspace_snapshot_id(state).unwrap_or(
        OmenaWorkspaceSnapshotIdV0::from_revision(omena_query::IncrementalRevisionV0 { value: 1 }),
    );
    let workspace = OmenaSdkWorkspaceV0::open_at_snapshot(
        OmenaSdkSnapshotRequestV0 {
            workspace_root: envelope.workspace_root.clone(),
        },
        lsp_style_sources(state, envelope.workspace_root.as_str()),
        snapshot_id,
    )?;

    match envelope.operation.as_str() {
        "snapshot" => {
            let request =
                parse_request::<OmenaSdkSnapshotRequestV0>(envelope.request, "snapshot request")?;
            if request.workspace_root != envelope.workspace_root {
                return Err(sdk_error(
                    OmenaErrorClassV0::Workspace,
                    "snapshot request root does not match the LSP workspace root",
                    "workspace.root-mismatch",
                    OmenaErrorRecoverabilityV0::UserAction,
                ));
            }
            serialize_response(workspace.snapshot())
        }
        "query" => serialize_response(
            workspace.execute_query(parse_request(envelope.request, "query request")?)?,
        ),
        "diagnostics" => serialize_response(
            workspace
                .execute_diagnostics(parse_request(envelope.request, "diagnostics request")?)?,
        ),
        "build" => serialize_response(
            workspace.execute_build(parse_request(envelope.request, "build request")?)?,
        ),
        "explain" => serialize_response(
            workspace.execute_explain(parse_request(envelope.request, "explain request")?)?,
        ),
        operation => Err(sdk_error(
            OmenaErrorClassV0::Unsupported,
            format!("unsupported SDK workflow operation {operation:?}"),
            "sdk.unsupported-operation",
            OmenaErrorRecoverabilityV0::UserAction,
        )),
    }
}

fn lsp_style_sources(
    state: &LspShellState,
    workspace_root: &str,
) -> Vec<OmenaQueryStyleSourceInputV0> {
    let mut sources = state
        .documents
        .values()
        .filter(|document| is_style_document_uri(document.uri.as_str()))
        .filter(|document| {
            document.workspace_folder_uri.as_deref().map_or_else(
                || uri_is_equal_or_descendant(workspace_root, document.uri.as_str()),
                |owner| workspace_folder_uri_equivalent(workspace_root, owner),
            )
        })
        .map(|document| OmenaQueryStyleSourceInputV0 {
            style_path: document.uri.clone(),
            style_source: document.text.clone(),
        })
        .collect::<Vec<_>>();
    sources.sort_by(|left, right| left.style_path.cmp(&right.style_path));
    sources
}

fn uri_is_equal_or_descendant(root: &str, candidate: &str) -> bool {
    candidate == root
        || candidate
            .strip_prefix(root.trim_end_matches('/'))
            .is_some_and(|suffix| suffix.starts_with('/'))
}

fn parse_request<T: serde::de::DeserializeOwned>(
    value: Value,
    label: &str,
) -> Result<T, OmenaError> {
    serde_json::from_value(value).map_err(|error| {
        sdk_error(
            OmenaErrorClassV0::Input,
            format!("failed to parse {label}: {error}"),
            "sdk.request-parse",
            OmenaErrorRecoverabilityV0::UserAction,
        )
    })
}

fn serialize_response<T: serde::Serialize>(value: T) -> Result<Value, OmenaError> {
    serde_json::to_value(value).map_err(|error| {
        sdk_error(
            OmenaErrorClassV0::Internal,
            format!("failed to serialize SDK workflow response: {error}"),
            "sdk.response-serialize",
            OmenaErrorRecoverabilityV0::Retry,
        )
    })
}

fn sdk_error(
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
