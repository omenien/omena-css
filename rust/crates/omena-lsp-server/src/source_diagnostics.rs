#[cfg(feature = "salsa-style-diagnostics")]
use crate::DeferredDiagnosticsRenderInputsV0;
use crate::{
    DiagnosticsPipelineTierPlanV0, LspDeferredDiagnosticsDispatchV0,
    LspOwnedSourceDiagnosticsRenderInputsV0, LspShellState, LspStyleHoverCandidate,
    LspTextDocumentState, document_has_style_index, first_style_document_for_workspace,
    lsp_diagnostic_severity,
    protocol::{file_uri_equivalent, is_style_document_uri, workspace_folder_compatible},
    query_style_selector_definition_for_matching, resolve_source_provider_candidates,
    style_selector_definitions_from_open_documents, style_selector_definitions_from_uri,
    style_sources_from_open_documents,
};
#[cfg(feature = "salsa-style-diagnostics")]
use omena_query::summarize_omena_query_source_baseline_diagnostics_for_workspace_file_with_source_syntax_index_and_definitions;
use omena_query::{
    OmenaQuerySourceDiagnosticV0, OmenaQuerySourceMissingSelectorDiagnosticCandidateV0,
    OmenaQuerySourceSyntaxIndexV0 as SourceSyntaxIndex, OmenaQueryStyleSelectorDefinitionV0,
    OmenaQueryStyleSourceInputV0, summarize_omena_query_source_diagnostics_for_file,
    summarize_omena_query_source_diagnostics_for_workspace_file_with_source_syntax_index_and_definitions,
};
use serde_json::{Value, json};

pub(crate) struct LspSourceDiagnosticsRenderInputsV0<'inputs> {
    pub(crate) document_uri: &'inputs str,
    pub(crate) document_text: &'inputs str,
    pub(crate) source_syntax_index: &'inputs SourceSyntaxIndex,
    pub(crate) source_selector_candidates: &'inputs [LspStyleHoverCandidate],
    pub(crate) style_sources: &'inputs [OmenaQueryStyleSourceInputV0],
    pub(crate) query_definitions: &'inputs [OmenaQueryStyleSelectorDefinitionV0],
    pub(crate) source_selector_fallback_candidates:
        &'inputs [OmenaQuerySourceMissingSelectorDiagnosticCandidateV0],
    pub(crate) global_class_fallthroughs: &'inputs [crate::LspGlobalClassFallthroughCandidateV0],
    pub(crate) configured_severity: u8,
}

impl LspOwnedSourceDiagnosticsRenderInputsV0 {
    pub(crate) fn borrowed(&self) -> LspSourceDiagnosticsRenderInputsV0<'_> {
        LspSourceDiagnosticsRenderInputsV0 {
            document_uri: self.document_uri.as_str(),
            document_text: self.document_text.as_str(),
            source_syntax_index: &self.source_syntax_index,
            source_selector_candidates: self.source_selector_candidates.as_slice(),
            style_sources: self.style_sources.as_slice(),
            query_definitions: self.query_definitions.as_slice(),
            source_selector_fallback_candidates: self
                .source_selector_fallback_candidates
                .as_slice(),
            global_class_fallthroughs: self.global_class_fallthroughs.as_slice(),
            configured_severity: self.configured_severity,
        }
    }
}

pub(crate) fn resolve_source_diagnostics_for_uri(
    state: &LspShellState,
    document_uri: &str,
) -> Value {
    gather_source_diagnostics_render_inputs(state, document_uri)
        .map(|inputs| finish_source_diagnostics_value(&inputs.borrowed()))
        .unwrap_or_else(|| json!([]))
}

#[cfg(feature = "salsa-style-diagnostics")]
pub(crate) fn prepare_deferred_source_diagnostics_for_uri(
    state: &LspShellState,
    document_uri: &str,
    tier_plan: DiagnosticsPipelineTierPlanV0,
) -> Option<(Value, LspDeferredDiagnosticsDispatchV0)> {
    let render_inputs = gather_source_diagnostics_render_inputs(state, document_uri)?;
    let diagnostics = finish_source_baseline_diagnostics_value(&render_inputs.borrowed());
    let ledger_epoch = state.tide_ledger.epoch();
    let dispatch = LspDeferredDiagnosticsDispatchV0 {
        ledger_epoch,
        uri: document_uri.to_string(),
        coalesce_key: String::new(),
        tier_plan,
        workspace_snapshot_id: None,
        render_inputs: DeferredDiagnosticsRenderInputsV0::Source(Box::new(render_inputs)),
    };
    Some((diagnostics, dispatch))
}

#[cfg(not(feature = "salsa-style-diagnostics"))]
pub(crate) fn prepare_deferred_source_diagnostics_for_uri(
    state: &LspShellState,
    document_uri: &str,
    tier_plan: DiagnosticsPipelineTierPlanV0,
) -> Option<(Value, LspDeferredDiagnosticsDispatchV0)> {
    let _ = (state, document_uri, tier_plan);
    None
}

fn gather_source_diagnostics_render_inputs(
    state: &LspShellState,
    document_uri: &str,
) -> Option<LspOwnedSourceDiagnosticsRenderInputsV0> {
    let document = state.document(document_uri)?;
    if is_style_document_uri(document.uri.as_str()) {
        return None;
    }

    let style_sources =
        style_sources_from_open_documents(state, document.workspace_folder_uri.as_deref(), None);
    let query_definitions = source_diagnostic_selector_definitions(state, document);
    let source_selector_candidates = document.source_selector_candidates.clone();
    let provider_candidates = resolve_source_provider_candidates(state, document)
        .unresolved
        .into_iter()
        .filter(|candidate| candidate.kind == "sourceSelectorReference")
        .collect::<Vec<_>>();
    // Two-tier universe inputs: the global class map (tier two) and the
    // property-access spans that must NEVER consult it — `styles.x` has no
    // runtime fall-through, so a module miss there stays a real miss.
    let global_class_definitions =
        global_class_definitions_for_workspace(state, document.workspace_folder_uri.as_deref());
    let property_access_ranges = document
        .source_syntax_index
        .style_property_accesses
        .iter()
        .map(|access| {
            crate::protocol::parser_range_for_byte_span(document.text.as_str(), access.byte_span)
        })
        .collect::<Vec<_>>();
    let mut global_class_fallthroughs = Vec::new();
    let source_selector_fallback_candidates = provider_candidates
        .into_iter()
        .filter_map(|candidate| {
            let (target_style_uri, target_style_document) = source_selector_diagnostic_target(
                state,
                &candidate,
                document.workspace_folder_uri.as_deref(),
            )?;
            let is_property_access = property_access_ranges
                .iter()
                .any(|range| parser_range_contains(range, &candidate.range));
            if !is_property_access
                && let Some(global_uri) = global_class_definitions.get(candidate.name.as_str())
            {
                global_class_fallthroughs.push(crate::LspGlobalClassFallthroughCandidateV0 {
                    selector_name: candidate.name,
                    global_definition_uri: global_uri.clone(),
                    target_style_uri,
                    target_style_source: target_style_document.text.clone(),
                    source_reference_range: candidate.range,
                });
                return None;
            }
            Some(OmenaQuerySourceMissingSelectorDiagnosticCandidateV0 {
                target_style_uri,
                target_style_source: target_style_document.text.clone(),
                selector_name: candidate.name,
                source_reference_range: candidate.range,
            })
        })
        .collect::<Vec<_>>();

    Some(LspOwnedSourceDiagnosticsRenderInputsV0 {
        document_uri: document.uri.clone(),
        document_text: document.text.clone(),
        source_syntax_index: document.source_syntax_index.clone(),
        source_selector_candidates,
        style_sources,
        query_definitions,
        source_selector_fallback_candidates,
        global_class_fallthroughs,
        configured_severity: state.diagnostics.severity,
    })
}

pub(crate) fn finish_source_diagnostics_value(
    inputs: &LspSourceDiagnosticsRenderInputsV0<'_>,
) -> Value {
    let query_diagnostics =
        summarize_omena_query_source_diagnostics_for_workspace_file_with_source_syntax_index_and_definitions(
            inputs.document_uri,
            inputs.document_text,
            inputs.source_syntax_index,
            inputs.query_definitions,
            inputs.style_sources,
        )
        .diagnostics
        .into_iter()
        .collect::<Vec<OmenaQuerySourceDiagnosticV0>>();
    finish_source_diagnostics_from_query_diagnostics(inputs, query_diagnostics)
}

#[cfg(feature = "salsa-style-diagnostics")]
fn finish_source_baseline_diagnostics_value(
    inputs: &LspSourceDiagnosticsRenderInputsV0<'_>,
) -> Value {
    let query_diagnostics =
        summarize_omena_query_source_baseline_diagnostics_for_workspace_file_with_source_syntax_index_and_definitions(
            inputs.document_uri,
            inputs.document_text,
            inputs.source_syntax_index,
            inputs.query_definitions,
            inputs.style_sources,
        )
        .diagnostics
        .into_iter()
        .collect::<Vec<OmenaQuerySourceDiagnosticV0>>();
    finish_source_diagnostics_from_query_diagnostics(inputs, query_diagnostics)
}

fn finish_source_diagnostics_from_query_diagnostics(
    inputs: &LspSourceDiagnosticsRenderInputsV0<'_>,
    mut query_diagnostics: Vec<OmenaQuerySourceDiagnosticV0>,
) -> Value {
    let _source_selector_candidate_count = inputs.source_selector_candidates.len();
    let fallback_diagnostics = summarize_omena_query_source_diagnostics_for_file(
        inputs.document_uri,
        inputs.source_selector_fallback_candidates,
    )
    .diagnostics;
    for fallback_diagnostic in fallback_diagnostics {
        if let Some(existing) = query_diagnostics.iter_mut().find(|diagnostic| {
            source_missing_selector_diagnostic_code(diagnostic.code)
                && diagnostic.range == fallback_diagnostic.range
        }) {
            if existing.create_selector.is_none() {
                existing.create_selector = fallback_diagnostic.create_selector;
            }
            continue;
        }
        query_diagnostics.push(fallback_diagnostic);
    }
    // Tier-two replacements: a reference the gather classified as a global
    // fall-through must not ALSO keep a module-tier miss from the syntax-
    // index arm at the same range (utility/literal references flow through
    // both arms).
    for fallthrough in inputs.global_class_fallthroughs {
        query_diagnostics.retain(|diagnostic| {
            !(source_missing_selector_diagnostic_code(diagnostic.code)
                && diagnostic.range == fallthrough.source_reference_range)
        });
        query_diagnostics.push(
            omena_query::summarize_omena_query_global_class_fallthrough_diagnostic(
                fallthrough.selector_name.as_str(),
                fallthrough.global_definition_uri.as_str(),
                fallthrough.target_style_uri.as_str(),
                fallthrough.target_style_source.as_str(),
                fallthrough.source_reference_range,
            ),
        );
    }
    query_diagnostics.sort_by_key(|diagnostic| {
        (
            diagnostic.range.start.line,
            diagnostic.range.start.character,
            diagnostic.code,
            diagnostic.message.clone(),
        )
    });
    query_diagnostics.dedup_by(|left, right| {
        left.code == right.code && left.range == right.range && left.message == right.message
    });

    let diagnostics: Vec<Value> = query_diagnostics
        .into_iter()
        .map(|diagnostic| {
            let query_severity = diagnostic.severity;
            let mut data = serde_json::Map::new();
            data.insert("querySeverity".to_string(), json!(query_severity));
            data.insert("provenance".to_string(), json!(diagnostic.provenance));
            if let Some(precision) = diagnostic.precision {
                data.insert("precision".to_string(), json!(precision));
            }
            if let Some(create_selector) = diagnostic.create_selector {
                data.insert("createSelector".to_string(), json!(create_selector));
            }

            json!({
                "range": diagnostic.range,
                "severity": lsp_diagnostic_severity(query_severity, inputs.configured_severity),
                "code": diagnostic.code,
                "source": "omena-css",
                "message": diagnostic.message,
                "data": Value::Object(data),
            })
        })
        .collect();

    json!(diagnostics)
}

fn source_missing_selector_diagnostic_code(code: &str) -> bool {
    matches!(
        code,
        "missingStaticClass"
            | "missingTemplatePrefix"
            | "missingResolvedClassValues"
            | "missingResolvedClassDomain"
    )
}

fn source_diagnostic_selector_definitions(
    state: &LspShellState,
    document: &LspTextDocumentState,
) -> Vec<OmenaQueryStyleSelectorDefinitionV0> {
    let mut definitions = style_selector_definitions_from_open_documents(
        state,
        "",
        document.workspace_folder_uri.as_deref(),
    );
    for reference in &document.source_syntax_index.selector_references {
        let Some(target_uri) = reference.target_style_uri.as_deref() else {
            continue;
        };
        if definitions
            .iter()
            .any(|(uri, _)| file_uri_equivalent(uri, target_uri))
        {
            continue;
        }
        definitions.extend(style_selector_definitions_from_uri(state, target_uri));
    }
    definitions
        .iter()
        .map(|(uri, definition)| query_style_selector_definition_for_matching(uri, definition))
        .collect()
}

fn source_selector_diagnostic_target<'a>(
    state: &'a LspShellState,
    candidate: &LspStyleHoverCandidate,
    workspace_folder_uri: Option<&str>,
) -> Option<(String, &'a LspTextDocumentState)> {
    if let Some(target_style_uri) = candidate.target_style_uri.as_deref() {
        let target_document = state.document(target_style_uri)?;
        if !document_has_style_index(target_document)
            || !workspace_folder_compatible(workspace_folder_uri, target_document)
        {
            return None;
        }
        return Some((target_document.uri.clone(), target_document));
    }

    first_style_document_for_workspace(state, workspace_folder_uri)
}

/// Tier two of the reference universe: class selector names defined by
/// indexed NON-module stylesheets (plain .scss/.css — the files a literal
/// class lands in at runtime). Reads only precomputed per-document
/// candidates; first definition wins for the disclosure's file name.
fn global_class_definitions_for_workspace(
    state: &LspShellState,
    workspace_folder_uri: Option<&str>,
) -> std::collections::BTreeMap<String, String> {
    let mut definitions = std::collections::BTreeMap::new();
    for document in state.documents.values() {
        if !is_style_document_uri(document.uri.as_str())
            || is_css_module_document_uri(document.uri.as_str())
            || !crate::workspace_folder_compatible(workspace_folder_uri, document)
        {
            continue;
        }
        for candidate in &document.style_candidates {
            if candidate.kind == "selector" {
                definitions
                    .entry(candidate.name.clone())
                    .or_insert_with(|| document.uri.clone());
            }
        }
    }
    definitions
}

fn is_css_module_document_uri(uri: &str) -> bool {
    uri.rsplit('/')
        .next()
        .is_some_and(|file_name| file_name.contains(".module."))
}

fn parser_range_contains(
    outer: &omena_query::ParserRangeV0,
    inner: &omena_query::ParserRangeV0,
) -> bool {
    let starts_before =
        (outer.start.line, outer.start.character) <= (inner.start.line, inner.start.character);
    let ends_after = (outer.end.line, outer.end.character) >= (inner.end.line, inner.end.character);
    starts_before && ends_after
}
