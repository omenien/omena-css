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
    #[serde(default)]
    contract_version: Option<String>,
    #[serde(default)]
    snapshot_binding: Option<omena_query::OmenaWorkspaceSnapshotBindingV0>,
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
    let bound = envelope.contract_version.as_deref() == Some("1");
    if envelope.contract_version.is_none() && envelope.snapshot_binding.is_some() {
        return Err(sdk_error(
            OmenaErrorClassV0::Workspace,
            "snapshot binding requires the explicit bound contract version",
            "workspace.snapshot-binding-required",
            OmenaErrorRecoverabilityV0::UserAction,
        ));
    }
    if envelope
        .contract_version
        .as_deref()
        .is_some_and(|version| version != "1")
    {
        return Err(sdk_error(
            OmenaErrorClassV0::Unsupported,
            "unsupported snapshot contract version",
            "workspace.snapshot-contract-version",
            OmenaErrorRecoverabilityV0::UserAction,
        ));
    }
    let workspace = if bound {
        bound_lsp_workspace(state, &envelope.workspace_root, snapshot_id)?
    } else {
        OmenaSdkWorkspaceV0::open_at_snapshot(
            OmenaSdkSnapshotRequestV0 {
                workspace_root: envelope.workspace_root.clone(),
            },
            lsp_style_sources(state, &envelope.workspace_root),
            snapshot_id,
        )?
    };
    if bound && !matches!(envelope.operation.as_str(), "snapshot" | "exportSnapshot") {
        let binding = envelope.snapshot_binding.as_ref().ok_or_else(|| {
            sdk_error(
                OmenaErrorClassV0::Workspace,
                "a bound workflow request must carry the complete snapshot binding",
                "workspace.snapshot-binding-required",
                OmenaErrorRecoverabilityV0::Retry,
            )
        })?;
        workspace.ensure_snapshot_binding(binding)?;
    }
    let binding = workspace.snapshot_binding().cloned();
    let response = match envelope.operation.as_str() {
        "snapshot" | "exportSnapshot" => {
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
            if envelope.operation == "exportSnapshot" {
                workspace.export_snapshot()
            } else {
                serialize_response(workspace.snapshot())
            }
        }
        "query" => serialize_response(
            workspace.execute_query(parse_request(envelope.request, "query request")?)?,
        ),
        "sourceDiagnostics" => serialize_response(workspace.execute_snapshot_source_diagnostics(
            parse_request(envelope.request, "source diagnostics request")?,
        )?),
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
    }?;
    match binding {
        Some(snapshot_binding) => serialize_response(omena_query::OmenaWorkspaceBoundResponseV1 {
            contract_version: "1".to_string(),
            snapshot_binding,
            response,
        }),
        None => Ok(response),
    }
}

fn bound_lsp_workspace(
    state: &LspShellState,
    root: &str,
    revision: OmenaWorkspaceSnapshotIdV0,
) -> Result<OmenaSdkWorkspaceV0, OmenaError> {
    use omena_query::{
        OmenaWorkspaceSnapshotInputsV0, OmenaWorkspaceSnapshotSettingsV0,
        OmenaWorkspaceSnapshotSourceV0, OmenaWorkspaceSnapshotTransferV0,
    };
    let styles = lsp_style_sources(state, root);
    let mut documents = state
        .documents
        .values()
        .filter(|document| {
            !is_style_document_uri(&document.uri)
                && document.workspace_folder_uri.as_deref().map_or_else(
                    || uri_is_equal_or_descendant(root, &document.uri),
                    |owner| workspace_folder_uri_equivalent(root, owner),
                )
        })
        .collect::<Vec<_>>();
    documents.sort_by(|left, right| left.uri.cmp(&right.uri));
    let sources = documents
        .iter()
        .map(|document| omena_query::OmenaQuerySourceDocumentInputV0 {
            source_path: document.uri.clone(),
            source_source: document.text.clone(),
            source_syntax_index: Some(document.source_syntax_index.clone()),
            has_unresolved_style_import: document.has_unresolved_style_import,
        })
        .collect::<Vec<_>>();
    let transfer = OmenaWorkspaceSnapshotTransferV0 {
        sources: documents
            .iter()
            .map(|document| OmenaWorkspaceSnapshotSourceV0 {
                source_path: document.uri.clone(),
                source_source: document.text.clone(),
                language_id: document.language_id.clone(),
                provider_inputs: state
                    .snapshot_source_provider_inputs
                    .get(&document.uri)
                    .filter(|provider| {
                        provider.source_digest
                            == omena_query::source_provider_snapshot_digest_v0(&document.text)
                    })
                    .cloned(),
            })
            .collect(),
        package_manifests: state.resolution.package_manifests.clone(),
        resolution_inputs: state
            .resolution
            .workspace_style_resolution_inputs
            .iter()
            .find(|(owner, _)| workspace_folder_uri_equivalent(root, owner))
            .map(|(_, inputs)| inputs.clone())
            .unwrap_or_default(),
        settings: OmenaWorkspaceSnapshotSettingsV0 {
            diagnostic_severity: state.diagnostics.severity,
            deep_analysis: state.diagnostics.deep_analysis,
            definition: state.features.definition,
            hover: state.features.hover,
            completion: state.features.completion,
            references: state.features.references,
            rename: state.features.rename,
            config_content_digest: None,
        },
        // Open-document ownership alone never proves a complete source corpus.
        source_corpus_complete: false,
    };
    let languages = transfer
        .sources
        .iter()
        .map(|source| (source.source_path.clone(), source.language_id.clone()))
        .collect();
    let utility = crate::protocol::file_uri_to_path(root)
        .map(|path| omena_query::load_omena_query_workspace_utility_class_intelligence(&path, None))
        .unwrap_or_default();
    let reconstructed = omena_query::reconstruct_omena_workspace_snapshot_sources_v0(
        root,
        &transfer.sources,
        &styles,
        &transfer.resolution_inputs,
        &utility,
    )?;
    if serialize_response(&reconstructed)? != serialize_response(&sources)? {
        return Err(sdk_error(
            OmenaErrorClassV0::Workspace,
            "admitted provider inputs do not reproduce the current owner's source facts",
            "workspace.source-provider-admission",
            OmenaErrorRecoverabilityV0::Retry,
        ));
    }
    let providers = transfer
        .sources
        .iter()
        .filter_map(|source| {
            source
                .provider_inputs
                .clone()
                .map(|provider| (source.source_path.clone(), provider))
        })
        .collect();
    let (external_sifs, external_sif_trust_records, external_sif_resolution_edges) =
        omena_query::select_omena_workspace_snapshot_external_sifs_v0(
            &styles,
            &transfer.resolution_inputs,
            &state.resolution.external_sifs,
            &state.resolution.external_sif_trust_records,
            &state.resolution.external_sif_resolution_edges,
        )?;
    let mut publishers = state.sdk_snapshot_publishers.borrow_mut();
    let publisher = publishers.entry(root.to_string()).or_default();
    let binding = publisher.publish(
        OmenaWorkspaceSnapshotInputsV0 {
            workspace_root: root,
            style_sources: &styles,
            source_documents: &sources,
            source_language_ids: &languages,
            source_provider_inputs: &providers,
            package_manifests: &transfer.package_manifests,
            external_sifs: &external_sifs,
            external_sif_trust_records: &external_sif_trust_records,
            external_sif_resolution_edges: &external_sif_resolution_edges,
            resolution_inputs: &transfer.resolution_inputs,
            settings: &transfer.settings,
            source_corpus_complete: transfer.source_corpus_complete,
        },
        revision,
    )?;
    OmenaSdkWorkspaceV0::open_owned_snapshot(
        OmenaSdkSnapshotRequestV0 {
            workspace_root: root.to_string(),
        },
        styles,
        transfer,
        sources,
        external_sifs,
        external_sif_trust_records,
        external_sif_resolution_edges,
        binding,
        publisher.reader(),
    )
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
