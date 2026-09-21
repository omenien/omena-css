use std::collections::BTreeMap;

use serde::Deserialize;

use crate::{
    EngineInputV2, IncrementalRevisionV0, OmenaBundlerHostResolveModuleRequestV0,
    OmenaBundlerHostResolveModuleResponseV0, OmenaError, OmenaErrorClassV0, OmenaErrorContextV0,
    OmenaErrorRecoverabilityV0, OmenaErrorSeverityV0, OmenaQueryBuildVerificationProfileV0,
    OmenaQueryConsumerBuildOptionsV0, OmenaQueryExplainInputV0,
    OmenaQuerySourceDiagnosticsForFileV0, OmenaQueryStylePackageManifestV0,
    OmenaQueryStyleResolutionInputsV0, OmenaQueryStyleSourceInputV0,
    OmenaQueryTransformStrictPolicyEventV0, OmenaQueryTransformStrictPolicyReasonV0,
    OmenaQueryTransformStrictPolicySummaryV0, OmenaSdkBuildRequestV0, OmenaSdkBuildResponseV0,
    OmenaSdkBuildVerificationEventV0, OmenaSdkBuildVerificationProfileV0,
    OmenaSdkBuildVerificationReasonV0, OmenaSdkBuildVerificationSummaryV0,
    OmenaSdkDiagnosticsRequestV0, OmenaSdkDiagnosticsResponseV0, OmenaSdkExplainRequestV0,
    OmenaSdkExplainResponseV0, OmenaSdkQueryRequestV0, OmenaSdkQueryResponseV0,
    OmenaSdkResponsePartitionV0, OmenaSdkSnapshotRequestV0, OmenaSdkSnapshotResponseV0,
    OmenaSdkSourceDiagnosticsRequestV0, OmenaWorkspaceSnapshotIdV0, ParserPositionV0,
    attach_omena_query_consumer_build_source_map_v3,
    execute_omena_query_consumer_build_style_source_with_context_and_options,
    execute_omena_sdk_diagnostics_workflow, explain_omena_query,
    read_omena_query_cascade_at_position, resolve_omena_bundler_host_module_v0,
    summarize_omena_query_consumer_check_style_source,
    summarize_omena_query_source_diagnostics_for_workspace_file_with_resolution_inputs,
    summarize_omena_query_style_document, summarize_omena_query_style_hover_candidates,
};

#[derive(Debug, Clone)]
pub struct OmenaSdkWorkspaceV0 {
    workspace_root: String,
    style_sources: BTreeMap<String, String>,
    style_resolution_inputs: OmenaQueryStyleResolutionInputsV0,
    revision: IncrementalRevisionV0,
    bound_snapshot: Option<BoundWorkspaceSnapshot>,
}

#[derive(Debug)]
struct BoundWorkspaceSnapshot {
    binding: crate::OmenaWorkspaceSnapshotBindingV0,
    reader: crate::OmenaWorkspaceSnapshotReaderV0,
    owner: Option<crate::OmenaWorkspaceSnapshotPublisherV0>,
    imported_utility: Option<std::sync::Arc<crate::OmenaQueryUtilityClassIntelligenceReportV0>>,
    styles: Vec<OmenaQueryStyleSourceInputV0>,
    sources: Vec<crate::OmenaQuerySourceDocumentInputV0>,
    languages: BTreeMap<String, String>,
    providers: BTreeMap<String, crate::OmenaWorkspaceSourceProviderInputsV0>,
    transfer: crate::OmenaWorkspaceSnapshotTransferV0,
    external_sifs: Vec<crate::OmenaQueryExternalSifInputV0>,
    trust_records: BTreeMap<String, crate::OmenaQueryExternalSifTrustV1>,
    resolution_edges: Vec<crate::OmenaQueryExternalSifResolutionEdgeV0>,
}

impl Clone for BoundWorkspaceSnapshot {
    fn clone(&self) -> Self {
        Self {
            binding: self.binding.clone(),
            reader: self.reader.clone(),
            owner: None,
            imported_utility: self.imported_utility.clone(),
            styles: self.styles.clone(),
            sources: self.sources.clone(),
            languages: self.languages.clone(),
            providers: self.providers.clone(),
            transfer: self.transfer.clone(),
            external_sifs: self.external_sifs.clone(),
            trust_records: self.trust_records.clone(),
            resolution_edges: self.resolution_edges.clone(),
        }
    }
}

impl BoundWorkspaceSnapshot {
    fn inputs<'a>(&'a self, root: &'a str) -> crate::OmenaWorkspaceSnapshotInputsV0<'a> {
        crate::OmenaWorkspaceSnapshotInputsV0 {
            workspace_root: root,
            style_sources: &self.styles,
            source_documents: &self.sources,
            source_language_ids: &self.languages,
            source_provider_inputs: &self.providers,
            package_manifests: &self.transfer.package_manifests,
            external_sifs: &self.external_sifs,
            external_sif_trust_records: &self.trust_records,
            external_sif_resolution_edges: &self.resolution_edges,
            resolution_inputs: &self.transfer.resolution_inputs,
            settings: &self.transfer.settings,
            source_corpus_complete: self.transfer.source_corpus_complete,
        }
    }
}

impl OmenaSdkWorkspaceV0 {
    pub fn open(
        request: OmenaSdkSnapshotRequestV0,
        style_sources: impl IntoIterator<Item = OmenaQueryStyleSourceInputV0>,
    ) -> Result<Self, OmenaError> {
        Self::open_with_resolution_inputs(
            request,
            style_sources,
            OmenaQueryStyleResolutionInputsV0::default(),
        )
    }

    pub fn open_with_resolution_inputs(
        request: OmenaSdkSnapshotRequestV0,
        style_sources: impl IntoIterator<Item = OmenaQueryStyleSourceInputV0>,
        style_resolution_inputs: OmenaQueryStyleResolutionInputsV0,
    ) -> Result<Self, OmenaError> {
        Self::open_at_snapshot_with_resolution_inputs(
            request,
            style_sources,
            OmenaWorkspaceSnapshotIdV0::from_revision(IncrementalRevisionV0 { value: 1 }),
            style_resolution_inputs,
        )
    }

    pub fn open_at_snapshot(
        request: OmenaSdkSnapshotRequestV0,
        style_sources: impl IntoIterator<Item = OmenaQueryStyleSourceInputV0>,
        snapshot_id: OmenaWorkspaceSnapshotIdV0,
    ) -> Result<Self, OmenaError> {
        Self::open_at_snapshot_with_resolution_inputs(
            request,
            style_sources,
            snapshot_id,
            OmenaQueryStyleResolutionInputsV0::default(),
        )
    }

    pub fn open_at_snapshot_with_resolution_inputs(
        request: OmenaSdkSnapshotRequestV0,
        style_sources: impl IntoIterator<Item = OmenaQueryStyleSourceInputV0>,
        snapshot_id: OmenaWorkspaceSnapshotIdV0,
        style_resolution_inputs: OmenaQueryStyleResolutionInputsV0,
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
            style_resolution_inputs,
            revision: snapshot_id.revision(),
            bound_snapshot: None,
        })
    }

    /// Import an explicit binding after reconstructing source facts from actual
    /// receiver-owned text. External trust claims are not a wire input: receivers
    /// independently regenerate local bridge facts through the existing trust path.
    pub fn open_imported_snapshot(
        request: OmenaSdkSnapshotRequestV0,
        styles: Vec<OmenaQueryStyleSourceInputV0>,
        transfer: crate::OmenaWorkspaceSnapshotTransferV0,
        binding: crate::OmenaWorkspaceSnapshotBindingV0,
        utility: &crate::OmenaQueryUtilityClassIntelligenceReportV0,
    ) -> Result<Self, OmenaError> {
        let sources = crate::reconstruct_omena_workspace_snapshot_sources_v0(
            &request.workspace_root,
            &transfer.sources,
            &styles,
            &transfer.resolution_inputs,
            utility,
        )?;
        let mut owner = crate::OmenaWorkspaceSnapshotPublisherV0::default();
        let languages = transfer
            .sources
            .iter()
            .map(|source| (source.source_path.clone(), source.language_id.clone()))
            .collect();
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
        let mut styles = styles;
        styles.sort_by(|left, right| left.style_path.cmp(&right.style_path));
        let (external_sifs, trust_records, resolution_edges) = admit_snapshot_external_sifs(
            &request.workspace_root,
            &styles,
            &transfer.resolution_inputs,
        )?;
        owner.import(
            &binding,
            crate::OmenaWorkspaceSnapshotInputsV0 {
                workspace_root: &request.workspace_root,
                style_sources: &styles,
                source_documents: &sources,
                source_language_ids: &languages,
                source_provider_inputs: &providers,
                package_manifests: &transfer.package_manifests,
                external_sifs: &external_sifs,
                external_sif_trust_records: &trust_records,
                external_sif_resolution_edges: &resolution_edges,
                resolution_inputs: &transfer.resolution_inputs,
                settings: &transfer.settings,
                source_corpus_complete: transfer.source_corpus_complete,
            },
        )?;
        let mut workspace = Self::open_owned_snapshot(
            request,
            styles,
            transfer,
            sources,
            external_sifs,
            trust_records,
            resolution_edges,
            binding,
            owner.reader(),
        )?;
        if let Some(bound) = &mut workspace.bound_snapshot {
            bound.owner = Some(owner);
            bound.imported_utility = Some(std::sync::Arc::new(utility.clone()));
        }
        Ok(workspace)
    }

    /// Attach the existing owner's admitted input view. The publisher validates
    /// every family; this does not create or replace LSP committed/editor state.
    pub fn open_owned_snapshot(
        request: OmenaSdkSnapshotRequestV0,
        mut styles: Vec<OmenaQueryStyleSourceInputV0>,
        transfer: crate::OmenaWorkspaceSnapshotTransferV0,
        sources: Vec<crate::OmenaQuerySourceDocumentInputV0>,
        external_sifs: Vec<crate::OmenaQueryExternalSifInputV0>,
        trust_records: BTreeMap<String, crate::OmenaQueryExternalSifTrustV1>,
        resolution_edges: Vec<crate::OmenaQueryExternalSifResolutionEdgeV0>,
        binding: crate::OmenaWorkspaceSnapshotBindingV0,
        reader: crate::OmenaWorkspaceSnapshotReaderV0,
    ) -> Result<Self, OmenaError> {
        styles.sort_by(|left, right| left.style_path.cmp(&right.style_path));
        let mut workspace = Self::open_at_snapshot_with_resolution_inputs(
            request,
            styles.clone(),
            binding.snapshot_id(),
            transfer.resolution_inputs.clone(),
        )?;
        let bound = BoundWorkspaceSnapshot {
            binding,
            reader,
            owner: None,
            imported_utility: None,
            styles,
            sources,
            languages: transfer
                .sources
                .iter()
                .map(|source| (source.source_path.clone(), source.language_id.clone()))
                .collect(),
            providers: transfer
                .sources
                .iter()
                .filter_map(|source| {
                    source
                        .provider_inputs
                        .clone()
                        .map(|provider| (source.source_path.clone(), provider))
                })
                .collect(),
            transfer,
            external_sifs,
            trust_records,
            resolution_edges,
        };
        bound
            .reader
            .read_view(&bound.binding, bound.inputs(&workspace.workspace_root))?;
        workspace.bound_snapshot = Some(bound);
        Ok(workspace)
    }

    pub fn snapshot_binding(&self) -> Option<&crate::OmenaWorkspaceSnapshotBindingV0> {
        self.bound_snapshot.as_ref().map(|bound| &bound.binding)
    }

    pub fn export_snapshot(&self) -> Result<serde_json::Value, OmenaError> {
        let bound = self.bound_snapshot.as_ref().ok_or_else(|| {
            sdk_error(
                OmenaErrorClassV0::Workspace,
                "snapshot export requires an admitted owner view",
                "workspace.snapshot-binding-required",
                OmenaErrorRecoverabilityV0::Retry,
            )
        })?;
        bound.reader.with_current_binding(&bound.binding, || serde_json::json!({
            "snapshot": self.snapshot(), "styleSources": bound.styles, "snapshotInputs": bound.transfer,
        }))
    }

    pub fn snapshot_read_view(
        &self,
    ) -> Result<crate::OmenaWorkspaceSnapshotReadViewV0<'_>, OmenaError> {
        let bound = self.bound_snapshot.as_ref().ok_or_else(|| {
            sdk_error(
                OmenaErrorClassV0::Workspace,
                "workspace has no admitted bound snapshot",
                "workspace.snapshot-binding-required",
                OmenaErrorRecoverabilityV0::Retry,
            )
        })?;
        bound
            .reader
            .read_view(&bound.binding, bound.inputs(&self.workspace_root))
    }

    pub fn ensure_snapshot_binding(
        &self,
        binding: &crate::OmenaWorkspaceSnapshotBindingV0,
    ) -> Result<(), OmenaError> {
        let bound = self.bound_snapshot.as_ref().ok_or_else(|| {
            sdk_error(
                OmenaErrorClassV0::Workspace,
                "a bound request requires an imported or owner snapshot",
                "workspace.snapshot-binding-required",
                OmenaErrorRecoverabilityV0::Retry,
            )
        })?;
        bound.reader.with_current_binding(binding, || ())?;
        if &bound.binding != binding {
            return Err(sdk_error(
                OmenaErrorClassV0::Workspace,
                "request binding differs from this snapshot view",
                "workspace.snapshot-mismatch",
                OmenaErrorRecoverabilityV0::Retry,
            ));
        }
        Ok(())
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
        if let Some(bound) = &self.bound_snapshot {
            bound.reader.with_current_binding(&bound.binding, || ())?;
        }
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
            let next = self
                .revision
                .value
                .checked_add(1)
                .ok_or_else(snapshot_revision_exhausted)?;
            if let Some(bound) = &mut self.bound_snapshot {
                bound.reader.with_current_binding(&bound.binding, || ())?;
                if bound.owner.is_none() {
                    return Err(sdk_error(
                        OmenaErrorClassV0::Workspace,
                        "a request clone must submit mutations to its existing workspace owner",
                        "workspace.snapshot-owner-required",
                        OmenaErrorRecoverabilityV0::Retry,
                    ));
                }
                let next_styles = replacement
                    .iter()
                    .map(|(path, source)| OmenaQueryStyleSourceInputV0 {
                        style_path: path.clone(),
                        style_source: source.clone(),
                    })
                    .collect::<Vec<_>>();
                let utility = bound.imported_utility.as_deref().ok_or_else(|| {
                    sdk_error(
                        OmenaErrorClassV0::Workspace,
                        "source replacement requires the owner's admitted utility inputs",
                        "workspace.snapshot-full-replacement-required",
                        OmenaErrorRecoverabilityV0::Retry,
                    )
                })?;
                // Resolve imports and provider projections against the replacement
                // styles before changing any current owner inputs or publication.
                let next_sources = crate::reconstruct_omena_workspace_snapshot_sources_v0(
                    &self.workspace_root,
                    &bound.transfer.sources,
                    &next_styles,
                    &bound.transfer.resolution_inputs,
                    utility,
                )?;
                let (next_sifs, next_trust, next_edges) = admit_snapshot_external_sifs(
                    &self.workspace_root,
                    &next_styles,
                    &bound.transfer.resolution_inputs,
                )?;
                let mut owner = bound.owner.take().ok_or_else(|| {
                    sdk_error(
                        OmenaErrorClassV0::Workspace,
                        "a request clone must submit mutations to its existing workspace owner",
                        "workspace.snapshot-owner-required",
                        OmenaErrorRecoverabilityV0::Retry,
                    )
                })?;
                let result = (|| {
                    owner.begin_mutation(&bound.binding)?;
                    bound.styles = next_styles;
                    bound.sources = next_sources;
                    bound.external_sifs = next_sifs;
                    bound.trust_records = next_trust;
                    bound.resolution_edges = next_edges;
                    self.style_sources = replacement;
                    bound.binding = owner.publish(
                        bound.inputs(&self.workspace_root),
                        OmenaWorkspaceSnapshotIdV0::from_revision(IncrementalRevisionV0 {
                            value: next,
                        }),
                    )?;
                    self.revision = bound.binding.snapshot_id().revision();
                    Ok(())
                })();
                bound.owner = Some(owner);
                result?;
            } else {
                self.style_sources = replacement;
                self.revision.value = next;
            }
        }
        Ok(self.snapshot())
    }

    pub fn replace_style_resolution_inputs(
        &mut self,
        style_resolution_inputs: OmenaQueryStyleResolutionInputsV0,
    ) -> Result<OmenaSdkSnapshotResponseV0, OmenaError> {
        if style_resolution_inputs != self.style_resolution_inputs {
            if self.bound_snapshot.is_some() {
                return Err(sdk_error(
                    OmenaErrorClassV0::Workspace,
                    "resolver mutation requires replacement of the complete admitted source-fact view",
                    "workspace.snapshot-full-replacement-required",
                    OmenaErrorRecoverabilityV0::Retry,
                ));
            }
            let next = self
                .revision
                .value
                .checked_add(1)
                .ok_or_else(snapshot_revision_exhausted)?;
            self.style_resolution_inputs = style_resolution_inputs;
            self.revision.value = next;
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

    pub fn execute_snapshot_source_diagnostics(
        &self,
        request: OmenaSdkSourceDiagnosticsRequestV0,
    ) -> Result<OmenaQuerySourceDiagnosticsForFileV0, OmenaError> {
        self.ensure_snapshot(request.snapshot_id, "source diagnostics")?;
        let bound = self.bound_snapshot.as_ref().ok_or_else(|| {
            sdk_error(
                OmenaErrorClassV0::Workspace,
                "source snapshot diagnostics require admitted source facts",
                "workspace.snapshot-binding-required",
                OmenaErrorRecoverabilityV0::Retry,
            )
        })?;
        let document = bound
            .sources
            .iter()
            .find(|source| source.source_path == request.source_path)
            .ok_or_else(|| {
                sdk_error(
                    OmenaErrorClassV0::Workspace,
                    "source document is absent from the snapshot",
                    "workspace.source-not-found",
                    OmenaErrorRecoverabilityV0::Retry,
                )
            })?;
        let index = document.source_syntax_index.as_ref().ok_or_else(|| {
            sdk_error(
                OmenaErrorClassV0::Workspace,
                "source facts have not been admitted",
                "workspace.source-provider-admission",
                OmenaErrorRecoverabilityV0::Retry,
            )
        })?;
        let definitions =
            crate::style::summarize_omena_query_style_selector_definitions(&bound.styles);
        Ok(if bound.transfer.settings.deep_analysis {
            crate::summarize_omena_query_source_diagnostics_for_workspace_file_with_source_syntax_index_and_definitions(
                &document.source_path, &document.source_source, index, &definitions, &bound.styles,
            )
        } else {
            crate::summarize_omena_query_source_baseline_diagnostics_for_workspace_file_with_source_syntax_index_and_definitions(
                &document.source_path, &document.source_source, index, &definitions, &bound.styles,
            )
        })
    }

    pub fn execute_source_diagnostics(
        &self,
        snapshot_id: OmenaWorkspaceSnapshotIdV0,
        source_path: &str,
        source: &str,
        package_manifests: &[OmenaQueryStylePackageManifestV0],
    ) -> Result<OmenaQuerySourceDiagnosticsForFileV0, OmenaError> {
        self.ensure_snapshot(snapshot_id, "source diagnostics")?;
        if let Some(bound) = &self.bound_snapshot {
            if !bound.sources.iter().any(|document| {
                document.source_path == source_path && document.source_source == source
            }) || bound.transfer.package_manifests != package_manifests
            {
                return Err(sdk_error(
                    OmenaErrorClassV0::Workspace,
                    "source diagnostics inputs differ from the bound snapshot",
                    "workspace.source-input-mismatch",
                    OmenaErrorRecoverabilityV0::Retry,
                ));
            }
            return self.execute_snapshot_source_diagnostics(OmenaSdkSourceDiagnosticsRequestV0 {
                snapshot_id,
                source_path: source_path.to_string(),
            });
        }
        let style_sources = self.style_source_inputs();
        Ok(
            summarize_omena_query_source_diagnostics_for_workspace_file_with_resolution_inputs(
                source_path,
                source,
                style_sources.as_slice(),
                package_manifests,
                &self.style_resolution_inputs,
            ),
        )
    }

    pub fn execute_bundler_resolve(
        &self,
        snapshot_id: OmenaWorkspaceSnapshotIdV0,
        style_path: String,
        package_manifests: Vec<OmenaQueryStylePackageManifestV0>,
    ) -> Result<OmenaBundlerHostResolveModuleResponseV0, OmenaError> {
        self.ensure_snapshot(snapshot_id, "bundler resolve")?;
        if self
            .bound_snapshot
            .as_ref()
            .is_some_and(|bound| bound.transfer.package_manifests != package_manifests)
        {
            return Err(sdk_error(
                OmenaErrorClassV0::Workspace,
                "bundler manifests differ from the bound snapshot",
                "workspace.manifest-input-mismatch",
                OmenaErrorRecoverabilityV0::Retry,
            ));
        }
        Ok(resolve_omena_bundler_host_module_v0(
            OmenaBundlerHostResolveModuleRequestV0 {
                snapshot_id: self.snapshot_id(),
                workspace_root: self.workspace_root.clone(),
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
        if let Some(bound) = &self.bound_snapshot {
            bound.reader.with_current_binding(&bound.binding, || ())?;
        }
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
        OmenaQueryTransformStrictPolicyReasonV0::ClosedWorldEvidenceIncomplete { .. }
        | OmenaQueryTransformStrictPolicyReasonV0::LivenessNotClosed { .. }
        | OmenaQueryTransformStrictPolicyReasonV0::EvidenceUnavailable
        | OmenaQueryTransformStrictPolicyReasonV0::OwnershipNotSeparable { .. } => {
            // Admission-tier events are serialized through closedWorldAdmission,
            // never through the strict-policy SDK summary.
            OmenaSdkBuildVerificationReasonV0::ClosedWorldEvidenceUnavailable
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

fn admit_snapshot_external_sifs(
    workspace_root: &str,
    styles: &[OmenaQueryStyleSourceInputV0],
    resolution: &OmenaQueryStyleResolutionInputsV0,
) -> Result<
    (
        Vec<crate::OmenaQueryExternalSifInputV0>,
        BTreeMap<String, crate::OmenaQueryExternalSifTrustV1>,
        Vec<crate::OmenaQueryExternalSifResolutionEdgeV0>,
    ),
    OmenaError,
> {
    // Independently admit each document's resolved targets, as the LSP owner
    // does. Another document's equal raw alias cannot suppress this admission.
    // The bridge reads actual sources and supplies trust on import/mutation;
    // transported lock/trust assertions never enter this path.
    let storage = crate::OmenaQueryExternalSifStorageV0::for_process_workspace_root(workspace_root);
    let mut external_sifs = Vec::new();
    let mut trust_records = BTreeMap::new();
    let mut resolution_edges = Vec::new();
    for style in styles {
        let admitted = match storage.as_ref() {
            Some(storage) => crate::resolve_omena_query_bridge_external_sifs_for_style_sources_with_cache_storage_and_trust(
                std::slice::from_ref(style), &[], resolution, storage,
            ),
            None => crate::resolve_omena_query_bridge_external_sifs_for_style_sources_with_trust(
                std::slice::from_ref(style), &[], resolution,
            ),
        };
        resolution_edges.extend(admitted.resolution_edges);
        external_sifs.extend(admitted.resolution.external_sifs);
        trust_records.extend(
            admitted
                .trust_records
                .into_iter()
                .map(|record| (record.canonical_url.clone(), record)),
        );
    }
    crate::select_omena_workspace_snapshot_external_sifs_v0(
        styles,
        resolution,
        &external_sifs,
        &trust_records,
        &resolution_edges,
    )
}

fn snapshot_revision_exhausted() -> OmenaError {
    sdk_error(
        OmenaErrorClassV0::Workspace,
        "workspace snapshot revision space is exhausted",
        "workspace.snapshot-revision-exhausted",
        OmenaErrorRecoverabilityV0::Retry,
    )
}
