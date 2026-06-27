use super::*;

pub(crate) fn resolve_style_diagnostics(state: &LspShellState, params: Option<&Value>) -> Value {
    let document_uri = document_uri_from_params(params);
    resolve_style_diagnostics_for_uri(state, document_uri.as_str())
}

pub(crate) fn resolve_document_diagnostics_for_uri(
    state: &LspShellState,
    document_uri: &str,
) -> Value {
    if is_style_document_uri(document_uri) {
        resolve_style_diagnostics_for_uri(state, document_uri)
    } else {
        resolve_source_diagnostics_for_uri(state, document_uri)
    }
}

pub(crate) fn resolve_style_diagnostics_for_uri(
    state: &LspShellState,
    document_uri: &str,
) -> Value {
    let Some(document) = state.document(document_uri) else {
        return json!([]);
    };
    let Some((_, candidates)) = style_hover_candidates_for_document(document) else {
        return json!([]);
    };

    let query_candidates = candidates
        .iter()
        .map(query_style_hover_candidate_from_lsp)
        .collect::<Vec<_>>();
    let style_sources = style_sources_from_open_documents(
        state,
        document.workspace_folder_uri.as_deref(),
        Some(document.uri.as_str()),
    );
    let source_documents =
        source_documents_from_open_documents(state, document.workspace_folder_uri.as_deref());
    let external_sifs = state.resolution.external_sifs.as_slice();
    // RFC-0007-J (#50): pass the workspace's tsconfig/bundler path mappings so the unused-selector
    // usage collector resolves alias style imports (`@/styles/...`) the same way the reference/goto
    // path does — otherwise an alias import dims every selector as `unusedSelector`.
    let resolution_inputs =
        resolution_inputs_for_workspace_uri(state, document.workspace_folder_uri.as_deref());
    // RFC 0009 Pillar C (rfcs#66) stage 1: the persistent content-addressed
    // shard cache. The composite key chains the FULL input surface gathered
    // above (target path, every style source, every source document, package
    // manifests, external SIFs, resolution inputs, diagnostics settings) plus
    // crate/schema/arm versions, so a shard can only serve when a recompute
    // would be byte-identical by construction. Misses fall through to the
    // compute below and persist write-behind; everything is fail-soft and
    // killable via OMENA_LSP_DISK_CACHE=off.
    let disk_cache_slot = disk_diagnostics_cache_slot_for_resolve(
        state,
        document.workspace_folder_uri.as_deref(),
        document.uri.as_str(),
        style_sources.as_slice(),
        source_documents.as_slice(),
        external_sifs,
        &resolution_inputs,
    );
    if let Some(cached_diagnostics) = disk_cache_slot.as_ref().and_then(|slot| slot.load()) {
        return cached_diagnostics;
    }
    // RFC 0009 Pillar B (rfcs#65): the workspace entry point runs through the
    // salsa-memoized host (input diff-sync + tracked query) so an unchanged
    // corpus revalidates instead of recomputing. `--no-default-features`
    // preserves the straight-line call; byte-identity between the two is
    // enforced by omena-diff-test's salsaMemoizedVsFromScratchEquivalence
    // gate. Both arms use query-level per-edge external classification.
    #[cfg(feature = "salsa-style-diagnostics")]
    let (workspace_diagnostics_summary, committed_cross_file_summary) = {
        let mut host_slot = state.style_memo_host.borrow_mut();
        let host = host_slot.get_or_insert_with(omena_query::OmenaQueryStyleMemoHostV0::new);
        host.workspace_style_diagnostics_with_selector(
            document.uri.as_str(),
            style_sources.as_slice(),
            source_documents.as_slice(),
            state.resolution.package_manifests.as_slice(),
            external_sifs,
            &resolution_inputs,
        )
        .map(|resolved| {
            (
                Some(resolved.diagnostics),
                Some(resolved.selector.workspace_cross_file_summary().clone()),
            )
        })
        .unwrap_or((None, None))
    };
    #[cfg(not(feature = "salsa-style-diagnostics"))]
    let workspace_diagnostics_summary =
        summarize_omena_query_style_diagnostics_for_workspace_file_with_external_mode_and_sifs_and_resolution_inputs(
            document.uri.as_str(),
            style_sources.as_slice(),
            source_documents.as_slice(),
            state.resolution.package_manifests.as_slice(),
            None,
            OmenaQueryExternalModuleModeV0::Auto,
            external_sifs,
            &resolution_inputs,
        );
    #[cfg(not(feature = "salsa-style-diagnostics"))]
    let committed_cross_file_summary: Option<omena_query::OmenaQueryCrossFileSummaryV0> = None;
    let diagnostics = finish_style_diagnostics_value(
        &LspStyleDiagnosticsRenderInputsV0 {
            document_uri: document.uri.as_str(),
            document_text: document.text.as_str(),
            query_candidates: query_candidates.as_slice(),
            deep_analysis: state.diagnostics.deep_analysis,
            configured_severity: state.diagnostics.severity,
        },
        workspace_diagnostics_summary,
        committed_cross_file_summary.as_ref(),
    );
    // RFC 0009 Pillar C (rfcs#66): write-behind after the compute. Fail-soft —
    // io errors are swallowed and a session breaker stops retrying hot.
    if let Some(slot) = disk_cache_slot.as_ref() {
        slot.store_write_behind(state, &diagnostics);
    }
    diagnostics
}

/// The full argument surface of [`finish_style_diagnostics_value`]: plain
/// `Send` data only, by design — no `&LspShellState`.
pub(crate) struct LspStyleDiagnosticsRenderInputsV0<'inputs> {
    pub(crate) document_uri: &'inputs str,
    pub(crate) document_text: &'inputs str,
    pub(crate) query_candidates: &'inputs [omena_query::OmenaQueryStyleHoverCandidateV0],
    pub(crate) deep_analysis: bool,
    pub(crate) configured_severity: u8,
}

#[cfg(feature = "salsa-style-diagnostics")]
impl LspOwnedStyleDiagnosticsRenderInputsV0 {
    fn borrowed(&self) -> LspStyleDiagnosticsRenderInputsV0<'_> {
        LspStyleDiagnosticsRenderInputsV0 {
            document_uri: self.document_uri.as_str(),
            document_text: self.document_text.as_str(),
            query_candidates: self.query_candidates.as_slice(),
            deep_analysis: self.deep_analysis,
            configured_severity: self.configured_severity,
        }
    }
}

pub(crate) fn prepare_deferred_style_diagnostics_for_uri(
    state: &LspShellState,
    document_uri: &str,
    tier_plan: DiagnosticsPipelineTierPlanV0,
) -> Option<(Value, LspDeferredDiagnosticsDispatchV0)> {
    #[cfg(not(feature = "salsa-style-diagnostics"))]
    {
        let _ = (state, document_uri, tier_plan);
        None
    }
    #[cfg(feature = "salsa-style-diagnostics")]
    {
        let document = state.document(document_uri)?;
        let (_, candidates) = style_hover_candidates_for_document(document)?;
        let query_candidates = candidates
            .iter()
            .map(query_style_hover_candidate_from_lsp)
            .collect::<Vec<_>>();
        let style_paths = style_path_inputs_from_open_documents(
            state,
            document.workspace_folder_uri.as_deref(),
            Some(document.uri.as_str()),
        );

        let mut baseline_summary = summarize_omena_query_style_diagnostics_for_file(
            document.uri.as_str(),
            document.text.as_str(),
            query_candidates.as_slice(),
        );
        baseline_summary.diagnostics.extend(
            summarize_omena_query_target_unresolved_sass_import_diagnostics_for_workspace_paths(
                document.uri.as_str(),
                document.text.as_str(),
                style_paths.as_slice(),
                state.resolution.package_manifests.as_slice(),
            ),
        );
        baseline_summary.diagnostic_count = baseline_summary.diagnostics.len();
        let baseline_render_inputs = LspStyleDiagnosticsRenderInputsV0 {
            document_uri: document.uri.as_str(),
            document_text: document.text.as_str(),
            query_candidates: query_candidates.as_slice(),
            deep_analysis: state.diagnostics.deep_analysis,
            configured_severity: state.diagnostics.severity,
        };
        let baseline_diagnostics =
            render_style_diagnostics_summary_value(&baseline_render_inputs, baseline_summary);
        let dispatch = LspDeferredDiagnosticsDispatchV0 {
            uri: document_uri.to_string(),
            coalesce_key: String::new(),
            tier_plan,
            render_inputs: DeferredDiagnosticsRenderInputsV0::StyleSnapshot(Box::new(
                state.query_snapshot(),
            )),
        };
        Some((baseline_diagnostics, dispatch))
    }
}

#[cfg(feature = "salsa-style-diagnostics")]
fn owned_style_diagnostics_render_inputs_for_uri(
    state: &LspShellState,
    document_uri: &str,
) -> Option<LspOwnedStyleDiagnosticsRenderInputsV0> {
    let document = state.document(document_uri)?;
    let (_, candidates) = style_hover_candidates_for_document(document)?;
    let query_candidates = candidates
        .iter()
        .map(query_style_hover_candidate_from_lsp)
        .collect::<Vec<_>>();
    let style_sources = style_sources_from_open_documents(
        state,
        document.workspace_folder_uri.as_deref(),
        Some(document.uri.as_str()),
    );
    let source_documents =
        source_documents_from_open_documents(state, document.workspace_folder_uri.as_deref());
    let resolution_inputs =
        resolution_inputs_for_workspace_uri(state, document.workspace_folder_uri.as_deref());
    Some(LspOwnedStyleDiagnosticsRenderInputsV0 {
        document_uri: document.uri.clone(),
        document_text: document.text.clone(),
        query_candidates,
        style_sources,
        source_documents,
        package_manifests: state.resolution.package_manifests.clone(),
        external_sifs: state.resolution.external_sifs.clone(),
        resolution_inputs,
        deep_analysis: state.diagnostics.deep_analysis,
        configured_severity: state.diagnostics.severity,
    })
}

#[cfg(feature = "salsa-style-diagnostics")]
pub fn resolve_deferred_diagnostics_notification(
    host: &mut omena_query::OmenaQueryStyleMemoHostV0,
    dispatch: &LspDeferredDiagnosticsDispatchV0,
) -> Value {
    let diagnostics = match &dispatch.render_inputs {
        DeferredDiagnosticsRenderInputsV0::StyleSnapshot(snapshot) => {
            let Some(inputs) = owned_style_diagnostics_render_inputs_for_uri(
                snapshot.shell_state(),
                &dispatch.uri,
            ) else {
                return diagnostics_scheduler::deferred_full_diagnostics_notification(
                    dispatch.uri.as_str(),
                    json!([]),
                    dispatch.tier_plan,
                );
            };
            let (workspace_summary, committed_cross_file_summary) = host
                .workspace_style_diagnostics_with_selector(
                    inputs.document_uri.as_str(),
                    inputs.style_sources.as_slice(),
                    inputs.source_documents.as_slice(),
                    inputs.package_manifests.as_slice(),
                    inputs.external_sifs.as_slice(),
                    &inputs.resolution_inputs,
                )
                .map(|resolved| {
                    (
                        Some(resolved.diagnostics),
                        Some(resolved.selector.workspace_cross_file_summary().clone()),
                    )
                })
                .unwrap_or((None, None));
            finish_style_diagnostics_value(
                &inputs.borrowed(),
                workspace_summary,
                committed_cross_file_summary.as_ref(),
            )
        }
        DeferredDiagnosticsRenderInputsV0::Source(inputs) => {
            finish_source_diagnostics_value(&inputs.borrowed())
        }
    };
    diagnostics_scheduler::deferred_full_diagnostics_notification(
        dispatch.uri.as_str(),
        diagnostics,
        dispatch.tier_plan,
    )
}

/// RFC 0009 Pillar F (rfcs#68): the worker-safe tail of the style
/// diagnostics pipeline — per-file fallback summarize, streaming-IFDS
/// extend, opt-in deep analysis, severity mapping and LSP JSON rendering.
/// Pure of its arguments, so the serial resolve and the parallel wave share
/// ONE implementation and cannot drift byte-wise.
pub(crate) fn finish_style_diagnostics_value(
    inputs: &LspStyleDiagnosticsRenderInputsV0<'_>,
    workspace_diagnostics_summary: Option<omena_query::OmenaQueryStyleDiagnosticsForFileV0>,
    committed_cross_file_summary: Option<&omena_query::OmenaQueryCrossFileSummaryV0>,
) -> Value {
    let mut diagnostics_summary = workspace_diagnostics_summary.unwrap_or_else(|| {
        summarize_omena_query_style_diagnostics_for_file(
            inputs.document_uri,
            inputs.document_text,
            inputs.query_candidates,
        )
    });
    if let Some(committed_cross_file_summary) = committed_cross_file_summary {
        diagnostics_summary.diagnostics.extend(
            summarize_cross_file_streaming_reachability_diagnostics_for_lsp(
                inputs.document_uri,
                committed_cross_file_summary,
            ),
        );
    }
    if inputs.deep_analysis {
        diagnostics_summary
            .diagnostics
            .extend(summarize_lsp_opt_in_deep_analysis_diagnostics(
                inputs.document_uri,
                inputs.document_text,
                inputs.query_candidates,
            ));
    }
    diagnostics_summary.diagnostic_count = diagnostics_summary.diagnostics.len();
    render_style_diagnostics_summary_value(inputs, diagnostics_summary)
}

fn render_style_diagnostics_summary_value(
    inputs: &LspStyleDiagnosticsRenderInputsV0<'_>,
    diagnostics_summary: omena_query::OmenaQueryStyleDiagnosticsForFileV0,
) -> Value {
    let diagnostics = diagnostics_summary
        .diagnostics
        .into_iter()
        .map(|diagnostic| {
            let tags = diagnostic.tags;
            let query_severity = diagnostic.severity;
            let mut data = serde_json::Map::new();
            data.insert("querySeverity".to_string(), json!(query_severity));
            data.insert("provenance".to_string(), json!(diagnostic.provenance));
            if let Some(create_custom_property) = diagnostic.create_custom_property {
                data.insert(
                    "createCustomProperty".to_string(),
                    json!(create_custom_property),
                );
            }
            if let Some(cascade_narrowing) = diagnostic.cascade_narrowing {
                if let Some(runtime_state) = cascade_narrowing.runtime_state.as_ref() {
                    data.insert("runtimeState".to_string(), json!(runtime_state));
                }
                data.insert("cascadeNarrowing".to_string(), json!(cascade_narrowing));
            }
            if let Some(cascade_confidence) = diagnostic.cascade_confidence {
                data.insert("cascadeConfidence".to_string(), json!(cascade_confidence));
            }
            if let Some(polynomial_provenance) = diagnostic.polynomial_provenance {
                data.insert(
                    "polynomialProvenance".to_string(),
                    json!(polynomial_provenance),
                );
            }
            if let Some(cross_file_scc) = diagnostic.cross_file_scc {
                data.insert("crossFileScc".to_string(), json!(cross_file_scc));
            }

            let mut lsp_diagnostic = json!({
                "range": diagnostic.range,
                "severity": lsp_diagnostic_severity(query_severity, inputs.configured_severity),
                "code": diagnostic.code,
                "source": "omena-css",
                "message": diagnostic.message,
                "data": Value::Object(data),
            });
            if !tags.is_empty() {
                lsp_diagnostic["tags"] = json!(tags);
            }
            lsp_diagnostic
        })
        .collect::<Vec<_>>();

    json!(diagnostics)
}

fn summarize_lsp_opt_in_deep_analysis_diagnostics(
    document_uri: &str,
    text: &str,
    candidates: &[omena_query::OmenaQueryStyleHoverCandidateV0],
) -> Vec<OmenaQueryStyleDiagnosticV0> {
    summarize_omena_query_style_diagnostics_for_file_with_deep_analysis(
        document_uri,
        text,
        candidates,
        true,
    )
    .diagnostics
    .into_iter()
    .filter(|diagnostic| {
        matches!(
            diagnostic.code,
            "rgFlowRelevantOperator"
                | "categoricalCascadeEvidenceInconsistency"
                | "cascadeSmtViolation"
        )
    })
    .collect()
}

pub(crate) fn lsp_diagnostic_severity(query_severity: &str, configured_severity: u8) -> u8 {
    if configured_severity != 2 {
        return configured_severity;
    }
    match query_severity {
        "error" => 1,
        "warning" => 2,
        "information" => 3,
        "hint" => 4,
        _ => configured_severity,
    }
}
