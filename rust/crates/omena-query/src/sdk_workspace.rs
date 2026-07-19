use std::collections::BTreeMap;

use serde::Deserialize;

use crate::{
    EngineInputV2, IncrementalRevisionV0, OmenaBundlerHostResolveModuleRequestV0,
    OmenaBundlerHostResolveModuleResponseV0, OmenaError, OmenaErrorClassV0, OmenaErrorContextV0,
    OmenaErrorRecoverabilityV0, OmenaErrorSeverityV0, OmenaQueryBuildVerificationProfileV0,
    OmenaQueryConsumerBuildOptionsV0, OmenaQueryExplainInputV0,
    OmenaQuerySourceDiagnosticsForFileV0, OmenaQueryStylePackageManifestV0,
    OmenaQueryStyleSourceInputV0, OmenaQueryTransformStrictPolicyEventV0,
    OmenaQueryTransformStrictPolicyReasonV0, OmenaQueryTransformStrictPolicySummaryV0,
    OmenaSdkBuildRequestV0, OmenaSdkBuildResponseV0, OmenaSdkBuildVerificationEventV0,
    OmenaSdkBuildVerificationProfileV0, OmenaSdkBuildVerificationReasonV0,
    OmenaSdkBuildVerificationSummaryV0, OmenaSdkDiagnosticsRequestV0,
    OmenaSdkDiagnosticsResponseV0, OmenaSdkExplainRequestV0, OmenaSdkExplainResponseV0,
    OmenaSdkQueryRequestV0, OmenaSdkQueryResponseV0, OmenaSdkResponsePartitionV0,
    OmenaSdkSnapshotRequestV0, OmenaSdkSnapshotResponseV0, OmenaWorkspaceSnapshotIdV0,
    ParserPositionV0, attach_omena_query_consumer_build_source_map_v3,
    execute_omena_query_consumer_build_style_source_with_context_and_options,
    execute_omena_sdk_diagnostics_workflow, explain_omena_query,
    read_omena_query_cascade_at_position, resolve_omena_bundler_host_module_v0,
    summarize_omena_query_consumer_check_style_source,
    summarize_omena_query_source_diagnostics_for_workspace_file,
    summarize_omena_query_style_document, summarize_omena_query_style_hover_candidates,
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

    pub fn execute_source_diagnostics(
        &self,
        snapshot_id: OmenaWorkspaceSnapshotIdV0,
        source_path: &str,
        source: &str,
        package_manifests: &[OmenaQueryStylePackageManifestV0],
    ) -> Result<OmenaQuerySourceDiagnosticsForFileV0, OmenaError> {
        self.ensure_snapshot(snapshot_id, "source diagnostics")?;
        let style_sources = self.style_source_inputs();
        Ok(summarize_omena_query_source_diagnostics_for_workspace_file(
            source_path,
            source,
            style_sources.as_slice(),
            package_manifests,
        ))
    }

    pub fn execute_bundler_resolve(
        &self,
        snapshot_id: OmenaWorkspaceSnapshotIdV0,
        style_path: String,
        package_manifests: Vec<OmenaQueryStylePackageManifestV0>,
    ) -> Result<OmenaBundlerHostResolveModuleResponseV0, OmenaError> {
        self.ensure_snapshot(snapshot_id, "bundler resolve")?;
        Ok(resolve_omena_bundler_host_module_v0(
            OmenaBundlerHostResolveModuleRequestV0 {
                snapshot_id: self.snapshot_id(),
                style_path,
                style_sources: self.style_source_inputs(),
                package_manifests,
            },
        ))
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
        let build_options = OmenaQueryConsumerBuildOptionsV0 {
            verification_profile: match request.verification_profile {
                Some(OmenaSdkBuildVerificationProfileV0::Strict) => {
                    OmenaQueryBuildVerificationProfileV0::Strict
                }
                Some(OmenaSdkBuildVerificationProfileV0::Descriptive) | None => {
                    OmenaQueryBuildVerificationProfileV0::Descriptive
                }
            },
            ..OmenaQueryConsumerBuildOptionsV0::default()
        };
        let default_context = crate::OmenaQueryTransformExecutionContextV0::default();
        let context = request.context.as_ref().unwrap_or(&default_context);
        let mut summary = execute_omena_query_consumer_build_style_source_with_context_and_options(
            style_path,
            style_source,
            request.pass_ids.as_slice(),
            context,
            &build_options,
        );
        attach_omena_query_consumer_build_source_map_v3(&mut summary, style_source);
        let verification = sdk_build_verification_summary(&summary.execution.strict_policy);
        Ok(OmenaSdkBuildResponseV0 {
            snapshot_id: self.snapshot_id(),
            partition: OmenaSdkResponsePartitionV0::Public,
            verification,
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

    fn style_source_inputs(&self) -> Vec<OmenaQueryStyleSourceInputV0> {
        self.style_sources
            .iter()
            .map(|(style_path, style_source)| OmenaQueryStyleSourceInputV0 {
                style_path: style_path.clone(),
                style_source: style_source.clone(),
            })
            .collect()
    }
}

fn sdk_build_verification_summary(
    summary: &OmenaQueryTransformStrictPolicySummaryV0,
) -> OmenaSdkBuildVerificationSummaryV0 {
    OmenaSdkBuildVerificationSummaryV0 {
        profile_id: summary.profile_id.clone(),
        refused_count: summary.refused_count as u64,
        rolled_back_count: summary.rolled_back_count as u64,
        refusal_reasons: summary
            .refusal_reasons
            .iter()
            .map(sdk_build_verification_event)
            .collect(),
        rollback_reasons: summary
            .rollback_reasons
            .iter()
            .map(sdk_build_verification_event)
            .collect(),
    }
}

fn sdk_build_verification_event(
    event: &OmenaQueryTransformStrictPolicyEventV0,
) -> OmenaSdkBuildVerificationEventV0 {
    OmenaSdkBuildVerificationEventV0 {
        pass_id: event.pass_id.clone(),
        reasons: event
            .reasons
            .iter()
            .map(sdk_build_verification_reason)
            .collect(),
    }
}

fn sdk_build_verification_reason(
    reason: &OmenaQueryTransformStrictPolicyReasonV0,
) -> OmenaSdkBuildVerificationReasonV0 {
    match reason {
        OmenaQueryTransformStrictPolicyReasonV0::RequiredAxisUnavailable { .. } => {
            OmenaSdkBuildVerificationReasonV0::RequiredAxisUnavailable
        }
        OmenaQueryTransformStrictPolicyReasonV0::CascadeEnvironmentUnavailable => {
            OmenaSdkBuildVerificationReasonV0::CascadeEnvironmentUnavailable
        }
        OmenaQueryTransformStrictPolicyReasonV0::WinnerChanged { .. } => {
            OmenaSdkBuildVerificationReasonV0::WinnerChanged
        }
        OmenaQueryTransformStrictPolicyReasonV0::ObservationUnavailable { .. } => {
            OmenaSdkBuildVerificationReasonV0::ObservationUnavailable
        }
        OmenaQueryTransformStrictPolicyReasonV0::UnknownPass => {
            OmenaSdkBuildVerificationReasonV0::UnknownPass
        }
        OmenaQueryTransformStrictPolicyReasonV0::ClosedWorldEvidenceUnavailable => {
            OmenaSdkBuildVerificationReasonV0::ClosedWorldEvidenceUnavailable
        }
        OmenaQueryTransformStrictPolicyReasonV0::DecisionCoverageIncomplete => {
            OmenaSdkBuildVerificationReasonV0::DecisionCoverageIncomplete
        }
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
