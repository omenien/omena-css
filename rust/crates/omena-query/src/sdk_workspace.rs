use std::collections::BTreeMap;

use serde::Deserialize;

use crate::{
    EngineInputV2, IncrementalRevisionV0, OmenaError, OmenaErrorClassV0, OmenaErrorContextV0,
    OmenaErrorRecoverabilityV0, OmenaErrorSeverityV0, OmenaQueryExplainInputV0,
    OmenaQueryStyleSourceInputV0, OmenaSdkBuildRequestV0, OmenaSdkBuildResponseV0,
    OmenaSdkDiagnosticsRequestV0, OmenaSdkDiagnosticsResponseV0, OmenaSdkExplainRequestV0,
    OmenaSdkExplainResponseV0, OmenaSdkQueryRequestV0, OmenaSdkQueryResponseV0,
    OmenaSdkResponsePartitionV0, OmenaSdkSnapshotRequestV0, OmenaSdkSnapshotResponseV0,
    OmenaWorkspaceSnapshotIdV0, ParserPositionV0, attach_omena_query_consumer_build_source_map_v3,
    execute_omena_query_consumer_build_style_source, execute_omena_sdk_diagnostics_workflow,
    explain_omena_query, read_omena_query_cascade_at_position,
    summarize_omena_query_consumer_check_style_source, summarize_omena_query_style_document,
    summarize_omena_query_style_hover_candidates,
};

#[derive(Debug, Clone)]
pub struct OmenaSdkWorkspaceV0 {
    workspace_root: String,
    style_sources: BTreeMap<String, String>,
    revision: IncrementalRevisionV0,
}

impl OmenaSdkWorkspaceV0 {
    pub fn open(
        request: OmenaSdkSnapshotRequestV0,
        style_sources: impl IntoIterator<Item = OmenaQueryStyleSourceInputV0>,
    ) -> Result<Self, OmenaError> {
        Self::open_at_snapshot(
            request,
            style_sources,
            OmenaWorkspaceSnapshotIdV0::from_revision(IncrementalRevisionV0 { value: 1 }),
        )
    }

    pub fn open_at_snapshot(
        request: OmenaSdkSnapshotRequestV0,
        style_sources: impl IntoIterator<Item = OmenaQueryStyleSourceInputV0>,
        snapshot_id: OmenaWorkspaceSnapshotIdV0,
    ) -> Result<Self, OmenaError> {
        if request.workspace_root.trim().is_empty() {
            return Err(sdk_error(
                OmenaErrorClassV0::Input,
                "workspace root must not be empty",
                "workspace.empty-root",
                OmenaErrorRecoverabilityV0::UserAction,
            ));
        }
        let mut sources = BTreeMap::new();
        for source in style_sources {
            let style_path = normalize_style_path(source.style_path.as_str());
            if sources
                .insert(style_path.clone(), source.style_source)
                .is_some()
            {
                return Err(sdk_error(
                    OmenaErrorClassV0::Input,
                    format!("workspace contains duplicate style path {style_path:?}"),
                    "workspace.duplicate-style-path",
                    OmenaErrorRecoverabilityV0::UserAction,
                ));
            }
        }
        Ok(Self {
            workspace_root: request.workspace_root,
            style_sources: sources,
            revision: snapshot_id.revision(),
        })
    }

    pub fn snapshot_id(&self) -> OmenaWorkspaceSnapshotIdV0 {
        OmenaWorkspaceSnapshotIdV0::from_revision(self.revision)
    }

    pub fn snapshot(&self) -> OmenaSdkSnapshotResponseV0 {
        OmenaSdkSnapshotResponseV0 {
            snapshot_id: self.snapshot_id(),
            partition: OmenaSdkResponsePartitionV0::Public,
            workspace_root: self.workspace_root.clone(),
        }
    }

    pub fn replace_style_sources(
        &mut self,
        style_sources: impl IntoIterator<Item = OmenaQueryStyleSourceInputV0>,
    ) -> Result<OmenaSdkSnapshotResponseV0, OmenaError> {
        let mut replacement = BTreeMap::new();
        for source in style_sources {
            let style_path = normalize_style_path(source.style_path.as_str());
            if replacement
                .insert(style_path.clone(), source.style_source)
                .is_some()
            {
                return Err(sdk_error(
                    OmenaErrorClassV0::Input,
                    format!("workspace contains duplicate style path {style_path:?}"),
                    "workspace.duplicate-style-path",
                    OmenaErrorRecoverabilityV0::UserAction,
                ));
            }
        }
        if replacement != self.style_sources {
            self.style_sources = replacement;
            self.revision.value = self.revision.value.saturating_add(1);
        }
        Ok(self.snapshot())
    }

    pub fn execute_query(
        &self,
        request: OmenaSdkQueryRequestV0,
    ) -> Result<OmenaSdkQueryResponseV0, OmenaError> {
        self.ensure_snapshot(request.snapshot_id, "query")?;
        let input = query_input(request.input.as_ref())?;
        let (style_path, style_source) = self.style_source(input.style_path.as_str())?;
        let payload = match request.query_kind.as_str() {
            "styleSummary" => summarize_omena_query_style_document(style_path, style_source)
                .map(|summary| serde_json::to_value(summary).map_err(serialize_error))
                .transpose()?
                .ok_or_else(|| {
                    sdk_error(
                        OmenaErrorClassV0::Analysis,
                        format!("style summary is unavailable for {style_path:?}"),
                        "query.style-summary-unavailable",
                        OmenaErrorRecoverabilityV0::Retry,
                    )
                })?,
            "hoverCandidates" => serde_json::to_value(
                summarize_omena_query_style_hover_candidates(style_path, style_source).ok_or_else(
                    || {
                        sdk_error(
                            OmenaErrorClassV0::Analysis,
                            format!("hover candidates are unavailable for {style_path:?}"),
                            "query.hover-candidates-unavailable",
                            OmenaErrorRecoverabilityV0::Retry,
                        )
                    },
                )?,
            )
            .map_err(serialize_error)?,
            _ => {
                return Err(sdk_error(
                    OmenaErrorClassV0::Unsupported,
                    format!("unsupported SDK query kind {:?}", request.query_kind),
                    "query.unsupported-kind",
                    OmenaErrorRecoverabilityV0::UserAction,
                ));
            }
        };
        Ok(OmenaSdkQueryResponseV0 {
            snapshot_id: self.snapshot_id(),
            partition: OmenaSdkResponsePartitionV0::Public,
            payload,
        })
    }

    pub fn execute_diagnostics(
        &self,
        mut request: OmenaSdkDiagnosticsRequestV0,
    ) -> Result<OmenaSdkDiagnosticsResponseV0, OmenaError> {
        self.ensure_snapshot(request.snapshot_id, "diagnostics")?;
        let (style_path, style_source) = self.style_source(request.style_path.as_str())?;
        if request.style_source != style_source {
            return Err(sdk_error(
                OmenaErrorClassV0::Workspace,
                format!("diagnostics source does not match snapshot for {style_path:?}"),
                "workspace.style-source-mismatch",
                OmenaErrorRecoverabilityV0::Retry,
            ));
        }
        request.style_path = style_path.to_string();
        execute_omena_sdk_diagnostics_workflow(request, self.snapshot_id())
    }

    pub fn execute_consumer_check(
        &self,
        snapshot_id: OmenaWorkspaceSnapshotIdV0,
        style_path: &str,
    ) -> Result<serde_json::Value, OmenaError> {
        self.ensure_snapshot(snapshot_id, "check")?;
        let (style_path, style_source) = self.style_source(style_path)?;
        serde_json::to_value(summarize_omena_query_consumer_check_style_source(
            style_path,
            style_source,
        ))
        .map_err(serialize_error)
    }

    pub fn execute_build(
        &self,
        mut request: OmenaSdkBuildRequestV0,
    ) -> Result<OmenaSdkBuildResponseV0, OmenaError> {
        self.ensure_snapshot(request.snapshot_id, "build")?;
        let (style_path, style_source) = self.style_source(request.style_path.as_str())?;
        if request.style_source != style_source {
            return Err(sdk_error(
                OmenaErrorClassV0::Workspace,
                format!("build source does not match snapshot for {style_path:?}"),
                "workspace.style-source-mismatch",
                OmenaErrorRecoverabilityV0::Retry,
            ));
        }
        request.style_path = style_path.to_string();
        let mut summary = match request.context.as_ref() {
            Some(context) => crate::execute_omena_query_consumer_build_style_source_with_context(
                style_path,
                style_source,
                request.pass_ids.as_slice(),
                context,
            ),
            None => execute_omena_query_consumer_build_style_source(
                style_path,
                style_source,
                request.pass_ids.as_slice(),
            ),
        };
        attach_omena_query_consumer_build_source_map_v3(&mut summary, style_source);
        Ok(OmenaSdkBuildResponseV0 {
            snapshot_id: self.snapshot_id(),
            partition: OmenaSdkResponsePartitionV0::Public,
            summary: serde_json::to_value(summary).map_err(serialize_error)?,
        })
    }

    pub fn execute_explain(
        &self,
        request: OmenaSdkExplainRequestV0,
    ) -> Result<OmenaSdkExplainResponseV0, OmenaError> {
        self.ensure_snapshot(request.snapshot_id, "explain")?;
        let (style_path, style_source) = self.style_source(request.style_path.as_str())?;
        let position = parser_position(request.position.line, request.position.character)?;
        let empty_input = EngineInputV2 {
            version: "2".to_string(),
            sources: Vec::new(),
            styles: Vec::new(),
            type_facts: Vec::new(),
        };
        let report = match read_omena_query_cascade_at_position(
            style_path,
            style_source,
            &empty_input,
            position,
        ) {
            Some(cascade) => {
                explain_omena_query(OmenaQueryExplainInputV0::Cascade { result: &cascade })
            }
            None => {
                let candidate_count =
                    summarize_omena_query_style_hover_candidates(style_path, style_source)
                        .map_or(0, |candidates| candidates.candidates.len());
                explain_omena_query(OmenaQueryExplainInputV0::HoverTrace {
                    document_uri: style_path,
                    position: Some(position),
                    reason_code: "style-position",
                    matched: candidate_count > 0,
                    candidate_count,
                    definition_count: 0,
                })
            }
        };
        let source_identity = serde_json::json!({
            "originalSource": style_path,
            "line": position.line,
            "character": position.character,
        });
        Ok(OmenaSdkExplainResponseV0 {
            snapshot_id: self.snapshot_id(),
            partition: OmenaSdkResponsePartitionV0::Public,
            report: serde_json::json!({
                "explanation": report,
                "sourceIdentity": source_identity,
            }),
        })
    }

    fn ensure_snapshot(
        &self,
        requested: OmenaWorkspaceSnapshotIdV0,
        operation: &str,
    ) -> Result<(), OmenaError> {
        if requested == self.snapshot_id() {
            return Ok(());
        }
        Err(sdk_error(
            OmenaErrorClassV0::Workspace,
            format!("{operation} request does not match the current workspace snapshot"),
            "workspace.snapshot-mismatch",
            OmenaErrorRecoverabilityV0::Retry,
        ))
    }

    fn style_source(&self, style_path: &str) -> Result<(&str, &str), OmenaError> {
        let style_path = normalize_style_path(style_path);
        self.style_sources
            .get_key_value(style_path.as_str())
            .map(|(path, source)| (path.as_str(), source.as_str()))
            .ok_or_else(|| {
                sdk_error(
                    OmenaErrorClassV0::Resolution,
                    format!("style path {style_path:?} is not present in the workspace snapshot"),
                    "workspace.style-path-not-found",
                    OmenaErrorRecoverabilityV0::UserAction,
                )
            })
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct OmenaSdkStyleQueryInputV0 {
    style_path: String,
}

fn query_input(input: Option<&serde_json::Value>) -> Result<OmenaSdkStyleQueryInputV0, OmenaError> {
    serde_json::from_value(input.cloned().unwrap_or(serde_json::Value::Null)).map_err(|error| {
        sdk_error(
            OmenaErrorClassV0::Input,
            format!("SDK query input is invalid: {error}"),
            "query.invalid-input",
            OmenaErrorRecoverabilityV0::UserAction,
        )
    })
}

fn parser_position(line: i32, character: i32) -> Result<ParserPositionV0, OmenaError> {
    let line = usize::try_from(line).map_err(|_| {
        sdk_error(
            OmenaErrorClassV0::Input,
            "explain line must be non-negative",
            "explain.invalid-position",
            OmenaErrorRecoverabilityV0::UserAction,
        )
    })?;
    let character = usize::try_from(character).map_err(|_| {
        sdk_error(
            OmenaErrorClassV0::Input,
            "explain character must be non-negative",
            "explain.invalid-position",
            OmenaErrorRecoverabilityV0::UserAction,
        )
    })?;
    Ok(ParserPositionV0 { line, character })
}

fn normalize_style_path(style_path: &str) -> String {
    if style_path.trim().is_empty() {
        "style.css".to_string()
    } else {
        style_path.to_string()
    }
}

fn serialize_error(error: serde_json::Error) -> OmenaError {
    sdk_error(
        OmenaErrorClassV0::Internal,
        format!("failed to serialize SDK workflow response: {error}"),
        "sdk.response-serialization",
        OmenaErrorRecoverabilityV0::Retry,
    )
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
