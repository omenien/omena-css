mod boundary;
mod code_actions;
mod color_provider;
#[cfg(feature = "salsa-style-diagnostics")]
mod deferred_notification;
mod diagnostics_follow_up;
mod diagnostics_scheduler;
mod disk_cache;
mod document_events;
mod document_links;
mod document_refresh;
mod document_state;
mod engine_input_params;
mod explain;
mod external_sif_loader;
mod external_sif_symbols;
mod foreign_style_identity;
mod frame_aware_refresh;
mod lsp_output;
mod message_loop;
mod occurrence_mapping;
mod open_document_inputs;
#[cfg(feature = "parallel-style-diagnostics")]
mod parallel_style_wave;
mod protocol;
mod provider_tier_feedback;
mod query_adapter;
mod query_reuse;
mod sdk_workflow;
mod settings;
mod source_completion;
mod source_diagnostics;
mod source_document_cache;
mod source_domain_hover;
mod source_occurrence_cache;
mod source_selector_provider;
mod source_syntax_index;
mod source_type_fact_cache;
mod source_type_facts;
mod state;
mod streaming_ifds_diagnostics;
mod style_diagnostics;
mod style_diagnostics_snapshot;
mod style_hover_markdown;
mod style_symbol_monikers;
mod style_symbol_occurrence_cache;
mod style_symbol_provider;
pub mod tide;
#[cfg(feature = "parallel-style-diagnostics")]
mod tide_republish;
mod workspace_index;
mod workspace_occurrence_cache;
mod workspace_occurrences;
mod workspace_resolution;
mod workspace_runtime_registry;
mod workspace_symbols;

pub use boundary::*;
#[cfg(feature = "salsa-style-diagnostics")]
pub use deferred_notification::{
    resolve_deferred_diagnostics_notification,
    resolve_deferred_diagnostics_notification_with_reverse_refresh,
};
pub use diagnostics_follow_up::*;
pub(crate) use document_events::{
    did_change_text_document, did_change_watched_files, did_change_workspace_folders,
    did_close_text_document, did_open_text_document,
};
pub(crate) use document_refresh::{
    StyleExternalDependencySnapshot, admit_foreign_style_dependencies_for_indexed_style_documents,
    admit_foreign_style_dependencies_for_style_uri,
    admit_foreign_style_dependencies_for_style_uris, ensure_style_document_loaded_from_disk,
    is_resolution_config_document_uri, refresh_source_indexes_for_resolution_config_change,
    refresh_source_indexes_for_resolution_settings_change,
    refresh_source_indexes_for_style_document_change,
    refresh_style_external_inputs_after_document_removal,
    refresh_style_external_inputs_for_document_event, reload_indexed_source_document_from_disk,
    reload_indexed_style_document_from_disk, style_external_dependency_snapshot,
    summarize_style_document,
};
pub(crate) use document_state::{
    lsp_text_document_state, lsp_text_document_state_with_source_syntax_index,
};
use engine_input_params::query_engine_input_from_params;
pub(crate) use explain::{
    EXPLAIN_REQUEST, project_hover_trace_through_explain_egress, resolve_lsp_explain,
};
pub use external_sif_loader::{
    LspExternalSifRefreshJobV0, LspExternalSifRefreshResultV0,
    apply_deferred_external_sif_refresh_result, collect_deferred_external_sif_refresh,
    enable_deferred_external_sif_refresh, prepare_deferred_external_sif_refresh_job,
};
pub(crate) use external_sif_loader::{
    bridge_sources_for_style_uris, refresh_external_sifs_for_bridge_source_delta,
    refresh_external_sifs_for_state,
};
use external_sif_symbols::external_sif_sass_symbol_definition_location;
pub(crate) use external_sif_symbols::{
    ExternalSifSassSymbolTarget, external_sif_sass_symbol_target_for_candidate,
};
pub use frame_aware_refresh::*;
pub use lsp_output::*;
#[cfg(test)]
pub(crate) use message_loop::current_time_millis;
pub use message_loop::{
    HOVER_SUBSTRATE_WARMUP_METHOD, LspLoopTurnV0, LspQueryDispatchV0,
    complete_dispatched_query_response, dispatched_query_internal_error_response,
    dispatched_query_is_heavy, handle_lsp_message, handle_lsp_message_outputs,
    handle_lsp_message_scheduled_outputs, handle_lsp_message_scheduled_outputs_or_dispatch,
    hover_substrate_warmup_dispatch, resolve_dispatched_query_response,
    workspace_index_progress_end_output,
};
#[cfg(feature = "salsa-style-diagnostics")]
use omena_query::summarize_omena_query_target_unresolved_sass_import_diagnostics_for_workspace_paths;
use omena_query::{
    OmenaParserStyleDialect, OmenaQueryCompletionCandidateV0, OmenaQueryCompletionItemV0,
    OmenaQueryStyleDiagnosticV0, OmenaWorkspaceOccurrenceFamilyV0, OmenaWorkspaceOccurrenceIndexV0,
    OmenaWorkspaceOccurrenceRoleV0, OmenaWorkspaceOccurrenceV0, ParserByteSpanV0, ParserPositionV0,
    is_omena_query_sass_symbol_candidate_kind as is_sass_symbol_candidate_kind,
    is_omena_query_sass_symbol_declaration_kind as is_sass_symbol_declaration_kind,
    is_omena_query_sass_symbol_reference_kind as is_sass_symbol_reference_kind,
    occurrences_for_monikers,
    omena_query_sass_symbol_kind_from_candidate_kind as sass_symbol_kind_from_candidate_kind,
    read_omena_query_cascade_at_position_with_categorical_evidence,
    read_omena_query_style_context_index, resolve_omena_query_sass_forward_sources,
    resolve_omena_query_sass_module_use_sources_for_candidate,
    resolve_omena_query_sass_symbol_declarations,
    resolve_omena_query_source_candidate_selector_names,
    resolve_omena_query_style_uri_for_specifier_with_resolution_inputs,
    summarize_omena_query_omena_parser_style_facts,
    summarize_omena_query_refs_for_class_from_occurrence_index,
    summarize_omena_query_rename_plan_from_occurrence_index,
    summarize_omena_query_source_completion_at_position,
    summarize_omena_query_style_completion_candidate_documentation,
    summarize_omena_query_style_completion_candidate_documentation_for_workspace_file_with_substrate,
    summarize_omena_query_style_completion_for_workspace_file_with_substrate,
    summarize_omena_query_style_diagnostics_for_file,
    summarize_omena_query_style_diagnostics_for_file_with_deep_analysis,
    summarize_omena_query_style_hover_render_parts_for_hover_position,
    summarize_omena_query_style_hover_render_parts_for_workspace_file_hover_position_with_substrate,
};
#[cfg(not(feature = "salsa-style-diagnostics"))]
use omena_query::{
    OmenaQueryExternalModuleModeV0,
    summarize_omena_query_style_diagnostics_for_workspace_file_with_external_mode_and_sifs_and_resolution_inputs,
};
#[cfg(test)]
pub(crate) use omena_query::{
    OmenaQueryExternalSifInputV0, OmenaQuerySourceImportedStyleBindingV0 as ImportedStyleBinding,
    OmenaQuerySourceSelectorReferenceFactV0 as SourceSelectorReferenceFact,
    OmenaQuerySourceSelectorReferenceMatchKindV0 as SourceSelectorReferenceMatchKind,
    OmenaQuerySourceSelectorReferenceSurfaceV0 as SourceSelectorReferenceSurface,
    OmenaQuerySourceSyntaxIndexV0 as SourceSyntaxIndex,
};
#[cfg(test)]
pub(crate) use omena_tsgo_client::{TsgoResolvedTypeV0, TsgoTypeFactResultEntryV0};
#[cfg(feature = "salsa-style-diagnostics")]
pub(crate) use open_document_inputs::style_path_inputs_from_open_documents;
pub(crate) use open_document_inputs::{
    source_documents_from_open_documents, style_sources_for_hover_render,
    style_sources_from_open_documents,
};
use protocol::*;
use provider_tier_feedback::{
    attach_provider_tier_feedback, current_provider_tier_feedback_data,
    provider_tier_feedback_for_hover_definitions,
};
use query_adapter::*;
use query_reuse::{
    cascade_narrowing_substrate_for_style_sources, effective_style_package_manifests,
    refresh_document_reusable_indexes,
};
use serde_json::{Value, json};
pub(crate) use settings::{
    apply_diagnostic_settings, apply_feature_settings, apply_resolution_settings,
};
use source_completion::{
    source_completion_context_at_position, source_domain_option_completion_items,
};
#[cfg(feature = "salsa-style-diagnostics")]
pub(crate) use source_diagnostics::finish_source_diagnostics_value;
pub(crate) use source_diagnostics::{
    prepare_deferred_source_diagnostics_for_uri, resolve_source_diagnostics_for_uri,
};
use source_domain_hover::{
    source_domain_reference_hover_at_position, source_domain_reference_trace_at_position,
};
use source_occurrence_cache::store_source_selector_occurrence_sidecar;
pub(crate) use source_selector_provider::{
    collect_source_selector_reference_candidates, document_has_style_index,
    first_style_document_for_workspace, resolve_source_provider_candidates,
    source_selector_candidate_at_position, source_selector_candidate_for_params,
    source_selector_candidates_at_position, style_selector_definitions_for_source_candidates,
    style_selector_definitions_from_open_documents, style_selector_definitions_from_uri,
};
pub(crate) use source_syntax_index::{
    build_source_syntax_index, collect_source_imports, source_selector_candidates_from_index,
};
#[cfg(test)]
pub(crate) use source_type_facts::apply_source_type_fact_results_to_document;
pub(crate) use source_type_facts::refresh_source_type_fact_candidates_for_document;
pub use state::*;
use std::{collections::BTreeSet, fs, sync::Arc};
use streaming_ifds_diagnostics::summarize_cross_file_streaming_reachability_diagnostics_for_lsp;
#[cfg(feature = "salsa-style-diagnostics")]
pub(crate) use style_diagnostics_snapshot::LspStyleDiagnosticsRenderInputsV0;

/// Apply an off-loop reverse-dependency refresh (produced by a worker's
/// selector build, delivered through the completion channel) to the loop
/// state's memo. The straight-line build has no memo; the refresh is a
/// no-op there.
pub fn apply_reverse_dependency_refresh(
    state: &LspShellState,
    refresh: &lsp_output::LspReverseDependencyRefreshV0,
) {
    #[cfg(feature = "salsa-style-diagnostics")]
    diagnostics_scheduler::refresh_reverse_dependency_index_memo(
        state,
        refresh.revision,
        &refresh.summary,
        refresh.ledger_epoch,
    );
    #[cfg(not(feature = "salsa-style-diagnostics"))]
    {
        let _ = (state, refresh);
    }
}
#[cfg(test)]
pub(crate) use style_diagnostics::resolve_style_diagnostics_for_uri;
pub(crate) use style_diagnostics::{
    lsp_diagnostic_severity, prepare_deferred_style_diagnostics_for_uri,
    resolve_document_diagnostics_for_uri, resolve_style_diagnostics,
};
use style_symbol_monikers::render_external_sif_sass_symbol_hover_markdown;
pub(crate) use style_symbol_occurrence_cache::store_style_symbol_occurrence_sidecar;
pub(crate) use style_symbol_provider::{
    external_document_uri_for_query_uri, reference_lens_title,
    render_style_hover_candidate_markdown_for_workspace, resolve_selector_rename,
    resolve_style_symbol_rename, sass_forward_edges_for_document,
    sass_symbol_definitions_for_candidate,
    selector_reference_locations_by_name_from_open_documents,
    selector_reference_locations_from_open_documents, source_candidate_selector_names,
    style_symbol_definition_locations_from_documents,
    style_symbol_reference_locations_from_documents,
    style_symbol_workspace_occurrences_for_document, unapply_sass_forward_prefix,
};
#[cfg(feature = "parallel-style-diagnostics")]
pub use tide_republish::{
    TideWorkspaceRepublishItemV0, TideWorkspaceRepublishJobV0, TideWorkspaceRepublishResultV0,
    apply_tide_workspace_republish_item, collect_tide_workspace_republish_streaming,
    complete_tide_workspace_republish, prepare_tide_workspace_republish_job,
};
pub(crate) use workspace_index::index_workspace_style_files;
pub(crate) use workspace_index::workspace_index_language_id_for_uri;
pub use workspace_index::{
    LspWorkspaceIndexJobV0, LspWorkspaceIndexResultV0, apply_background_workspace_index_result,
    collect_background_workspace_index, prepare_background_workspace_index_continuation_job,
    prepare_background_workspace_index_job,
};
#[cfg(test)]
pub(crate) use workspace_index::{
    WorkspaceStyleIndexBudget, index_workspace_style_files_with_budget,
};
#[cfg(test)]
pub(crate) use workspace_resolution::load_lsp_workspace_style_resolution_inputs;
pub(crate) use workspace_resolution::{
    initialize_workspace_folders, insert_workspace_folder, refresh_document_workspace_owners,
    refresh_workspace_resolution_inputs, refresh_workspace_resolution_inputs_for_uri,
    resolution_inputs_for_workspace_uri, resolve_workspace_folder_uri,
};

pub const NODE_TEXT_DOCUMENT_SYNC_KIND: u8 = 2;

pub fn omena_loop_trace_enabled() -> bool {
    static ENABLED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ENABLED.get_or_init(|| std::env::var_os("OMENA_LOOP_TRACE").is_some())
}

#[macro_export]
macro_rules! loop_trace {
    ($($arg:tt)*) => {
        if $crate::omena_loop_trace_enabled() {
            eprintln!("[LOOPTRACE {:>10.3}] {}",
                std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs_f64()%100000.0).unwrap_or(0.0),
                format!($($arg)*));
        }
    };
}

pub const DEBUG_STATE_REQUEST: &str = "omena/rustLspState";
pub const RUNTIME_LOOP_PROBE_REQUEST: &str = "omena/runtimeLoopProbe";
pub const STYLE_HOVER_CANDIDATES_REQUEST: &str = "omena/rustStyleHoverCandidates";
pub const STYLE_DIAGNOSTICS_REQUEST: &str = "omena/rustStyleDiagnostics";
pub const SOURCE_DIAGNOSTICS_REQUEST: &str = "omena/rustSourceDiagnostics";
pub const CASCADE_AT_POSITION_REQUEST: &str = "omena/rustCascadeAtPosition";
pub const STYLE_CONTEXT_INDEX_REQUEST: &str = "omena/rustStyleContextIndex";
pub const EXPLAIN_HOVER_TRACE_REQUEST: &str = "omena/explainHoverTrace";
pub const SDK_WORKFLOW_REQUEST: &str = "omena/sdkWorkflow";
const CANCEL_REQUEST_METHOD: &str = "$/cancelRequest";
const REQUEST_CANCELLED_ERROR_CODE: i32 = -32800;
// Cascade docs cost a whole-corpus narrowing analysis per completion item; only the
// top-ranked items an editor list actually shows get them, so completion latency
// stays independent of the workspace selector count.
const SOURCE_COMPLETION_DOCUMENTATION_BUDGET: usize = 12;

pub(crate) use code_actions::resolve_lsp_code_actions;

#[cfg(feature = "test-support")]
pub mod test_support {
    use std::path::Path;

    pub fn file_uri_equivalent(left: &str, right: &str) -> bool {
        crate::protocol::file_uri_equivalent(left, right)
    }

    pub fn path_to_file_uri(path: &Path) -> String {
        crate::protocol::path_to_file_uri(path)
    }
}

pub fn resolve_style_hover_candidates(
    state: &dyn LspQueryReadView,
    params: Option<&Value>,
) -> LspStyleHoverCandidatesResult {
    let document_uri = document_uri_from_params(params);
    let query_position = lsp_position_from_params(params);
    let Some(document) = state.document(&document_uri) else {
        return empty_style_hover_candidates_result(document_uri, None, query_position);
    };

    let Some((language, mut candidates)) = style_hover_candidates_for_document(document) else {
        return empty_style_hover_candidates_result(
            document_uri,
            document.workspace_folder_uri.clone(),
            query_position,
        );
    };

    if let Some(position) = query_position {
        candidates.retain(|candidate| parser_range_contains_position(&candidate.range, position));
    }

    LspStyleHoverCandidatesResult {
        schema_version: "0",
        product: "omena-lsp-server.style-hover-candidates",
        document_uri,
        workspace_folder_uri: document.workspace_folder_uri.clone(),
        language: Some(language),
        query_position,
        candidate_count: candidates.len(),
        candidates,
    }
}

fn style_hover_candidates_for_document(
    document: &LspTextDocumentState,
) -> Option<(&'static str, Vec<LspStyleHoverCandidate>)> {
    let summary = document.style_summary.as_ref()?;
    Some((summary.language, document.style_candidates.clone()))
}

fn style_text_for_uri(state: &dyn LspQueryReadView, uri: &str) -> Option<String> {
    state
        .document(uri)
        .map(|document| document.text.clone())
        .or_else(|| fs::read_to_string(file_uri_to_path(uri)?).ok())
}

fn style_hover_candidates_for_uri(
    state: &dyn LspQueryReadView,
    uri: &str,
) -> Option<(&'static str, Vec<LspStyleHoverCandidate>)> {
    if let Some(document) = state.document(uri) {
        return style_hover_candidates_for_document(document);
    }
    let text = style_text_for_uri(state, uri)?;
    collect_style_hover_candidates(uri, text.as_str())
}

fn resolve_lsp_definition(state: &dyn LspQueryReadView, params: Option<&Value>) -> Value {
    let document_uri = document_uri_from_params(params);
    let Some(position) = lsp_position_from_params(params) else {
        return Value::Null;
    };
    let Some(document) = state.document(&document_uri) else {
        return Value::Null;
    };
    if !is_style_document_uri(document.uri.as_str()) {
        return resolve_source_lsp_definition(state, document, position);
    }

    let Some((_, candidates)) = style_hover_candidates_for_document(document) else {
        return Value::Null;
    };
    let Some(candidate) = candidates
        .iter()
        .find(|candidate| parser_range_contains_position(&candidate.range, position))
    else {
        return Value::Null;
    };
    if is_sass_symbol_reference_kind(candidate.kind) {
        let definitions = sass_symbol_definitions_for_candidate(state, document, candidate);
        if !definitions.is_empty() {
            return json!(
                definitions
                    .into_iter()
                    .map(|(uri, definition)| json!({ "uri": uri, "range": definition.range }))
                    .collect::<Vec<_>>()
            );
        }
        if let Some(location) =
            external_sif_sass_symbol_definition_location(state, document, candidate)
        {
            return json!([location]);
        }
        return Value::Null;
    }
    if candidate.kind == "customPropertyReference" {
        let definitions =
            style_symbol_definition_locations_from_documents(state, document, candidate);
        if !definitions.is_empty() {
            return json!(definitions);
        }
    }

    let target = candidate;

    json!([
        {
            "uri": document.uri.as_str(),
            "range": target.range,
        },
    ])
}

fn resolve_lsp_references(state: &LspShellState, params: Option<&Value>) -> Value {
    let document_uri = document_uri_from_params(params);
    let Some(position) = lsp_position_from_params(params) else {
        return Value::Null;
    };
    let Some(document) = state.document(&document_uri) else {
        return Value::Null;
    };
    if !is_style_document_uri(document.uri.as_str()) {
        return resolve_source_lsp_references(state, document, position, params);
    }

    let Some((_, candidates)) = style_hover_candidates_for_document(document) else {
        return Value::Null;
    };
    let Some(candidate) = candidates
        .iter()
        .find(|candidate| parser_range_contains_position(&candidate.range, position))
    else {
        return Value::Null;
    };
    let include_declaration = include_declaration_from_params(params);
    let mut locations: Vec<Value> = if candidate.kind.starts_with("customProperty")
        || is_sass_symbol_candidate_kind(candidate.kind)
    {
        style_symbol_reference_locations_from_documents(
            state,
            document,
            candidate,
            include_declaration,
        )
    } else if candidate.kind == "selector" {
        let mut locations = if include_declaration {
            vec![json!({ "uri": document.uri.as_str(), "range": candidate.range })]
        } else {
            Vec::new()
        };
        locations.extend(selector_reference_locations_from_open_documents(
            state,
            candidate.name.as_str(),
            document.workspace_folder_uri.as_deref(),
            Some(document.uri.as_str()),
        ));
        locations
    } else if include_declaration {
        vec![json!({ "uri": document.uri.as_str(), "range": candidate.range })]
    } else {
        Vec::new()
    };

    locations.sort_by_key(|location| {
        let line = location
            .pointer("/range/start/line")
            .and_then(Value::as_u64)
            .unwrap_or_default();
        let character = location
            .pointer("/range/start/character")
            .and_then(Value::as_u64)
            .unwrap_or_default();
        (line, character)
    });
    json!(locations)
}

fn resolve_lsp_completion(state: &LspShellState, params: Option<&Value>) -> Value {
    let document_uri = document_uri_from_params(params);
    let Some(document) = state.document(&document_uri) else {
        return Value::Null;
    };
    if !is_style_document_uri(document.uri.as_str()) {
        return resolve_source_lsp_completion(state, document, params);
    }

    let Some(position) = lsp_position_from_params(params) else {
        return Value::Null;
    };
    let style_sources = style_sources_from_open_documents(
        state,
        document.workspace_folder_uri.as_deref(),
        Some(document.uri.as_str()),
    );
    let resolution_inputs =
        resolution_inputs_for_workspace_uri(state, document.workspace_folder_uri.as_deref());
    let package_manifests = effective_style_package_manifests(state, &resolution_inputs);
    let narrowing_substrate = cascade_narrowing_substrate_for_style_sources(
        state,
        style_sources.as_slice(),
        &resolution_inputs,
    );
    let completion = summarize_omena_query_style_completion_for_workspace_file_with_substrate(
        document.uri.as_str(),
        style_sources.as_slice(),
        package_manifests.as_slice(),
        state.resolution.external_sifs.as_slice(),
        &resolution_inputs,
        &narrowing_substrate,
        position,
    );
    let provider_feedback =
        current_provider_tier_feedback_data(document, "textDocument/completion");
    let items: Vec<Value> = completion
        .items
        .into_iter()
        .map(|item| {
            lsp_completion_item_from_query(completion.file_kind, item, provider_feedback.as_ref())
        })
        .collect();

    json!({
        "isIncomplete": false,
        "items": items,
    })
}

fn lsp_completion_item_from_query(
    file_kind: &str,
    item: OmenaQueryCompletionItemV0,
    provider_feedback: Option<&Value>,
) -> Value {
    let kind = match (file_kind, item.item_kind) {
        ("style", "cssModuleSelector") => 7,
        (_, "cssModuleSelector") | (_, "cssCustomProperty") => 10,
        _ => 1,
    };
    let mut completion_item = json!({
        "label": item.label,
        "kind": kind,
        "sortText": item.sort_text,
        "detail": item.detail,
        "insertText": item.insert_text,
        "data": {
            "source": item.source,
            "rankingSource": item.ranking_source,
        },
    });
    if let Some(documentation) = item.documentation {
        completion_item["documentation"] = json!({
            "kind": "markdown",
            "value": documentation,
        });
    }
    attach_provider_tier_feedback(&mut completion_item, provider_feedback);
    completion_item
}

fn resolve_source_diagnostics(state: &LspShellState, params: Option<&Value>) -> Value {
    let document_uri = document_uri_from_params(params);
    resolve_source_diagnostics_for_uri(state, document_uri.as_str())
}

fn resolve_cascade_at_position(state: &LspShellState, params: Option<&Value>) -> Value {
    let document_uri = document_uri_from_params(params);
    let Some(position) = lsp_position_from_params(params) else {
        return Value::Null;
    };
    let Some(document) = state.document(&document_uri) else {
        return Value::Null;
    };
    if !is_style_document_uri(document.uri.as_str()) {
        return Value::Null;
    }
    let Some(engine_input) = query_engine_input_from_params(params) else {
        return Value::Null;
    };

    let include_categorical_evidence = params
        .and_then(|value| value.get("context"))
        .and_then(|value| value.get("includeCategoricalEvidence"))
        .and_then(Value::as_bool)
        .unwrap_or(false);

    read_omena_query_cascade_at_position_with_categorical_evidence(
        document.uri.as_str(),
        document.text.as_str(),
        &engine_input,
        position,
        include_categorical_evidence,
    )
    .map(|result| json!(result))
    .unwrap_or(Value::Null)
}

pub(crate) fn resolve_style_context_index(state: &LspShellState, params: Option<&Value>) -> Value {
    let document_uri = document_uri_from_params(params);
    let Some(document) = state.document(&document_uri) else {
        return Value::Null;
    };
    if !is_style_document_uri(document.uri.as_str()) {
        return Value::Null;
    }
    let Some(engine_input) = query_engine_input_from_params(params) else {
        return Value::Null;
    };

    read_omena_query_style_context_index(
        document.uri.as_str(),
        document.text.as_str(),
        &engine_input,
    )
    .map(|result| json!(result))
    .unwrap_or(Value::Null)
}

fn resolve_lsp_code_lens(state: &dyn LspQueryReadView, params: Option<&Value>) -> Value {
    let document_uri = document_uri_from_params(params);
    let Some(document) = state.document(document_uri.as_str()) else {
        return Value::Null;
    };
    let Some((_, candidates)) = style_hover_candidates_for_document(document) else {
        return Value::Null;
    };

    let mut lenses = Vec::new();
    let mut emitted_selectors = BTreeSet::new();
    let reference_locations_by_name = selector_reference_locations_by_name_from_open_documents(
        state,
        document.workspace_folder_uri.as_deref(),
        Some(document.uri.as_str()),
    );
    for candidate in candidates
        .iter()
        .filter(|candidate| candidate.kind == "selector")
    {
        if !emitted_selectors.insert(candidate.name.as_str()) {
            continue;
        }
        let locations = reference_locations_by_name
            .get(candidate.name.as_str())
            .cloned()
            .unwrap_or_default();
        if locations.is_empty() {
            continue;
        }
        let position = candidate.range.start;
        lenses.push(json!({
            "range": {
                "start": position,
                "end": position,
            },
            "command": {
                "title": reference_lens_title(locations.len()),
                "command": "editor.action.showReferences",
                "arguments": [
                    document.uri.as_str(),
                    position,
                    locations,
                ],
            },
        }));
    }
    lenses.sort_by_key(lsp_range_start_sort_key);

    if lenses.is_empty() {
        Value::Null
    } else {
        json!(lenses)
    }
}

pub(crate) fn query_style_dialect_for_uri(uri: &str) -> OmenaParserStyleDialect {
    let lower = uri.to_ascii_lowercase();
    if lower.ends_with(".sass") || lower.ends_with(".sass?module") {
        OmenaParserStyleDialect::Sass
    } else if lower.ends_with(".scss") || lower.ends_with(".module.scss") {
        OmenaParserStyleDialect::Scss
    } else if lower.ends_with(".less") || lower.ends_with(".module.less") {
        OmenaParserStyleDialect::Less
    } else {
        OmenaParserStyleDialect::Css
    }
}

pub(crate) fn resolve_lsp_style_uri_for_specifier(
    state: &dyn LspQueryReadView,
    document: &LspTextDocumentState,
    specifier: &str,
) -> Option<String> {
    let resolution_inputs =
        resolution_inputs_for_workspace_uri(state, document.workspace_folder_uri.as_deref());
    resolve_omena_query_style_uri_for_specifier_with_resolution_inputs(
        document.uri.as_str(),
        document.workspace_folder_uri.as_deref(),
        specifier,
        &resolution_inputs,
    )
}

fn resolve_lsp_prepare_rename(state: &LspShellState, params: Option<&Value>) -> Value {
    if let Some((_, candidate)) = source_selector_candidate_for_params(state, params) {
        return json!({
            "range": candidate.range,
            "placeholder": candidate.name,
        });
    }

    let Some((_, candidate, _)) = style_candidates_for_params(state, params) else {
        return Value::Null;
    };

    json!({
        "range": candidate.range,
        "placeholder": rename_placeholder(&candidate),
    })
}

fn resolve_lsp_rename(state: &LspShellState, params: Option<&Value>) -> Value {
    let Some(new_name) = params
        .and_then(|value| value.get("newName"))
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
    else {
        return Value::Null;
    };
    if let Some((document_uri, candidate)) = source_selector_candidate_for_params(state, params) {
        let workspace_folder_uri = state
            .document(document_uri.as_str())
            .and_then(|document| document.workspace_folder_uri.as_deref());
        return resolve_selector_rename(
            state,
            workspace_folder_uri,
            candidate.target_style_uri.as_deref(),
            candidate.name.as_str(),
            new_name,
        );
    }

    let Some((document_uri, candidate, _candidates)) = style_candidates_for_params(state, params)
    else {
        return Value::Null;
    };

    if candidate.kind == "selector" {
        let workspace_folder_uri = state
            .document(document_uri.as_str())
            .and_then(|document| document.workspace_folder_uri.as_deref());
        return resolve_selector_rename(
            state,
            workspace_folder_uri,
            Some(document_uri.as_str()),
            candidate.name.as_str(),
            new_name,
        );
    }

    if candidate.kind.starts_with("customProperty") || is_sass_symbol_candidate_kind(candidate.kind)
    {
        let Some(document) = state.document(document_uri.as_str()) else {
            return Value::Null;
        };
        return resolve_style_symbol_rename(state, document, &candidate, new_name);
    }

    Value::Null
}

fn style_candidates_for_params(
    state: &LspShellState,
    params: Option<&Value>,
) -> Option<(String, LspStyleHoverCandidate, Vec<LspStyleHoverCandidate>)> {
    let document_uri = document_uri_from_params(params);
    let position = lsp_position_from_params(params)?;
    let document = state.document(document_uri.as_str())?;
    let (_, candidates) = style_hover_candidates_for_document(document)?;
    let candidate = candidates
        .iter()
        .find(|candidate| parser_range_contains_position(&candidate.range, position))?
        .clone();
    Some((document_uri, candidate, candidates))
}

fn rename_placeholder(candidate: &LspStyleHoverCandidate) -> &str {
    candidate.name.as_str()
}

fn resolve_lsp_hover(state: &dyn LspQueryReadView, params: Option<&Value>) -> Value {
    let document_uri = document_uri_from_params(params);
    if let Some(document) = state.document(document_uri.as_str())
        && !is_style_document_uri(document.uri.as_str())
    {
        return resolve_source_lsp_hover(state, document, params);
    }

    let candidates = resolve_style_hover_candidates(state, params);
    let Some(candidate) = candidates.candidates.first() else {
        return Value::Null;
    };
    let Some(document) = state.document(document_uri.as_str()) else {
        return Value::Null;
    };
    if is_sass_symbol_reference_kind(candidate.kind)
        && let Some((target_uri, target)) =
            sass_symbol_definitions_for_candidate(state, document, candidate)
                .into_iter()
                .next()
        && let Some(target_text) = style_text_for_uri(state, target_uri.as_str())
    {
        let mut response = json!({
            "contents": {
                "kind": "markdown",
                "value": render_style_hover_candidate_markdown_for_workspace(
                    state,
                    target_uri.as_str(),
                    target_text.as_str(),
                    &target,
                ),
            },
            "range": candidate.range,
        });
        if let Some(target_document) = state.document(target_uri.as_str()) {
            let provider_feedback =
                current_provider_tier_feedback_data(target_document, "textDocument/hover");
            attach_provider_tier_feedback(&mut response, provider_feedback.as_ref());
        }
        return response;
    }
    if is_sass_symbol_reference_kind(candidate.kind)
        && let Some(target) =
            external_sif_sass_symbol_target_for_candidate(state, document, candidate)
    {
        return json!({
            "contents": {
                "kind": "markdown",
                "value": render_external_sif_sass_symbol_hover_markdown(&target),
            },
            "range": candidate.range,
        });
    }

    let mut response = json!({
        "contents": {
            "kind": "markdown",
            "value": render_style_hover_candidate_markdown_for_workspace(
                state,
                document.uri.as_str(),
                document.text.as_str(),
                candidate,
            ),
        },
        "range": candidate.range,
    });
    let provider_feedback = current_provider_tier_feedback_data(document, "textDocument/hover");
    attach_provider_tier_feedback(&mut response, provider_feedback.as_ref());
    response
}

fn resolve_lsp_hover_trace(state: &LspShellState, params: Option<&Value>) -> Value {
    let document_uri = document_uri_from_params(params);
    let mut trace = if let Some(position) = lsp_position_from_params(params) {
        if let Some(document) = state.document(document_uri.as_str()) {
            if is_style_document_uri(document.uri.as_str()) {
                resolve_style_lsp_hover_trace(state, document, position)
            } else {
                resolve_source_lsp_hover_trace(state, document, position)
            }
        } else {
            empty_hover_trace(
                document_uri,
                None,
                "unknown",
                Some(position),
                "documentNotIndexed",
                None,
            )
        }
    } else {
        empty_hover_trace(document_uri, None, "unknown", None, "missingPosition", None)
    };
    project_hover_trace_through_explain_egress(&mut trace);
    trace
}

fn resolve_style_lsp_hover_trace(
    state: &LspShellState,
    document: &LspTextDocumentState,
    position: ParserPositionV0,
) -> Value {
    let Some((language, candidates)) = style_hover_candidates_for_document(document) else {
        return empty_hover_trace(
            document.uri.clone(),
            document.workspace_folder_uri.clone(),
            "style",
            Some(position),
            "styleDocumentNotIndexed",
            None,
        );
    };
    let matched = candidates
        .iter()
        .filter(|candidate| parser_range_contains_position(&candidate.range, position))
        .cloned()
        .collect::<Vec<_>>();
    let Some(candidate) = matched.first() else {
        return json!({
            "schemaVersion": "0",
            "product": "omena-lsp-server.explain-hover-trace",
            "documentUri": document.uri.as_str(),
            "workspaceFolderUri": document.workspace_folder_uri.as_deref(),
            "fileKind": "style",
            "language": language,
            "queryPosition": position,
            "matched": false,
            "reason": "noStyleCandidateAtPosition",
            "candidateCount": 0,
            "definitionCount": 0,
            "candidates": [],
            "definitions": [],
            "resolutionPath": ["styleHoverCandidates"],
            "readySurfaces": ["explainHoverTraceRpc", "styleHoverCandidates"],
        });
    };
    let definitions =
        style_hover_trace_definitions(state, document, candidate, candidates.as_slice());
    let rendered_markdown =
        render_source_hover_definitions_markdown(state, definitions.as_slice()).unwrap_or_default();

    json!({
        "schemaVersion": "0",
        "product": "omena-lsp-server.explain-hover-trace",
        "documentUri": document.uri.as_str(),
        "workspaceFolderUri": document.workspace_folder_uri.as_deref(),
        "fileKind": "style",
        "language": language,
        "queryPosition": position,
        "matched": true,
        "reason": "styleCandidateResolved",
        "candidateCount": matched.len(),
        "definitionCount": definitions.len(),
        "candidates": matched,
        "definitions": hover_trace_definition_values(definitions.as_slice()),
        "renderedMarkdown": rendered_markdown,
        "resolutionPath": ["styleHoverCandidates", "styleDefinitionResolver", "hoverMarkdownRenderer"],
        "readySurfaces": ["explainHoverTraceRpc", "styleHoverCandidates", "hoverMarkdownRenderer"],
    })
}

fn resolve_source_lsp_hover_trace(
    state: &LspShellState,
    document: &LspTextDocumentState,
    position: ParserPositionV0,
) -> Value {
    if let Some(trace) = source_domain_reference_trace_at_position(state, document, position) {
        return trace;
    }

    let resolution = resolve_source_provider_candidates(state, document);
    let matched = resolution
        .matched
        .into_iter()
        .filter(|candidate| parser_range_contains_position(&candidate.range, position))
        .collect::<Vec<_>>();
    let unresolved = resolution
        .unresolved
        .into_iter()
        .filter(|candidate| parser_range_contains_position(&candidate.range, position))
        .collect::<Vec<_>>();
    if matched.is_empty() && unresolved.is_empty() {
        let type_fact_tier = source_type_fact_tier_trace(document, position);
        return empty_hover_trace(
            document.uri.clone(),
            document.workspace_folder_uri.clone(),
            "source",
            Some(position),
            "noSourceCandidateAtPosition",
            type_fact_tier,
        );
    }

    let definitions = style_selector_definitions_for_source_candidates(
        state,
        matched.as_slice(),
        document.workspace_folder_uri.as_deref(),
    );
    let rendered_markdown =
        render_source_hover_definitions_markdown(state, definitions.as_slice()).unwrap_or_default();

    let mut trace = json!({
        "schemaVersion": "0",
        "product": "omena-lsp-server.explain-hover-trace",
        "documentUri": document.uri.as_str(),
        "workspaceFolderUri": document.workspace_folder_uri.as_deref(),
        "fileKind": "source",
        "languageId": document.language_id.as_str(),
        "queryPosition": position,
        "matched": !matched.is_empty(),
        "reason": if matched.is_empty() { "sourceCandidateUnresolved" } else { "sourceCandidateResolved" },
        "matchedCandidateCount": matched.len(),
        "unresolvedCandidateCount": unresolved.len(),
        "definitionCount": definitions.len(),
        "candidates": matched,
        "unresolvedCandidates": unresolved,
        "definitions": hover_trace_definition_values(definitions.as_slice()),
        "renderedMarkdown": rendered_markdown,
        "resolutionPath": ["sourceSyntaxIndex", "sourceProviderCandidateResolution", "styleSelectorDefinitionResolver", "hoverMarkdownRenderer"],
        "readySurfaces": ["explainHoverTraceRpc", "sourceSyntaxIndex", "sourceProviderCandidateResolution", "hoverMarkdownRenderer"],
    });
    attach_type_fact_tier_trace(&mut trace, source_type_fact_tier_trace(document, position));
    trace
}

fn empty_hover_trace(
    document_uri: String,
    workspace_folder_uri: Option<String>,
    file_kind: &'static str,
    query_position: Option<ParserPositionV0>,
    reason: &'static str,
    type_fact_tier: Option<Value>,
) -> Value {
    let mut trace = json!({
        "schemaVersion": "0",
        "product": "omena-lsp-server.explain-hover-trace",
        "documentUri": document_uri,
        "workspaceFolderUri": workspace_folder_uri,
        "fileKind": file_kind,
        "queryPosition": query_position,
        "matched": false,
        "reason": reason,
        "candidateCount": 0,
        "definitionCount": 0,
        "candidates": [],
        "definitions": [],
        "resolutionPath": [],
        "readySurfaces": ["explainHoverTraceRpc"],
    });
    attach_type_fact_tier_trace(&mut trace, type_fact_tier);
    trace
}

fn attach_type_fact_tier_trace(trace: &mut Value, type_fact_tier: Option<Value>) {
    if let Some(type_fact_tier) = type_fact_tier
        && let Some(trace) = trace.as_object_mut()
    {
        trace.insert("typeFactTier".to_string(), type_fact_tier);
    }
}

fn source_type_fact_tier_trace(
    document: &LspTextDocumentState,
    position: ParserPositionV0,
) -> Option<Value> {
    let offset = byte_offset_for_parser_position(document.text.as_str(), position)?;
    let contains_offset = |span: ParserByteSpanV0| span.start <= offset && offset < span.end;
    let skipped_count = document.source_syntax_index.type_fact_target_skipped_count;

    if let Some(fact) = document
        .source_syntax_index
        .type_fact_target_skipped
        .iter()
        .find(|fact| contains_offset(fact.byte_span))
    {
        return Some(json!({
            "attempted": false,
            "outcome": "neverAttempted",
            "reason": fact.reason,
            "skippedTargetCount": skipped_count,
        }));
    }
    if let Some(fact) = document
        .source_syntax_index
        .type_fact_provider_unavailable
        .iter()
        .find(|fact| contains_offset(fact.byte_span))
    {
        return Some(json!({
            "attempted": true,
            "outcome": "unavailable",
            "reason": fact.reason,
            "skippedTargetCount": skipped_count,
        }));
    }
    document
        .source_syntax_index
        .selector_references
        .iter()
        .any(|reference| {
            contains_offset(reference.byte_span)
                && reference.surface.as_str() == "omenaTsgoTypeFactProjection"
        })
        .then(|| {
            json!({
                "attempted": true,
                "outcome": "resolved",
                "skippedTargetCount": skipped_count,
            })
        })
}

fn style_hover_trace_definitions(
    state: &LspShellState,
    document: &LspTextDocumentState,
    candidate: &LspStyleHoverCandidate,
    candidates: &[LspStyleHoverCandidate],
) -> Vec<(String, LspStyleHoverCandidate)> {
    if is_sass_symbol_reference_kind(candidate.kind) {
        return sass_symbol_definitions_for_candidate(state, document, candidate);
    }
    if candidate.kind == "customPropertyReference"
        && let Some(target) = candidates.iter().find(|target| {
            target.kind == "customPropertyDeclaration" && target.name == candidate.name
        })
    {
        return vec![(document.uri.clone(), target.clone())];
    }
    vec![(document.uri.clone(), candidate.clone())]
}

fn hover_trace_definition_values(definitions: &[(String, LspStyleHoverCandidate)]) -> Vec<Value> {
    definitions
        .iter()
        .map(|(uri, definition)| {
            json!({
                "uri": uri,
                "kind": definition.kind,
                "name": definition.name,
                "range": definition.range,
                "source": definition.source,
                "targetStyleUri": definition.target_style_uri,
                "namespace": definition.namespace,
            })
        })
        .collect()
}

fn resolve_source_lsp_hover(
    state: &dyn LspQueryReadView,
    document: &LspTextDocumentState,
    params: Option<&Value>,
) -> Value {
    let Some(position) = lsp_position_from_params(params) else {
        return Value::Null;
    };
    if let Some((range, value)) =
        source_domain_reference_hover_at_position(state, document, position)
    {
        return json!({
            "contents": {
                "kind": "markdown",
                "value": value,
            },
            "range": range,
        });
    }
    let candidates = source_selector_candidates_at_position(state, document, position);
    let Some(candidate) = candidates.first() else {
        return Value::Null;
    };
    let definitions = style_selector_definitions_for_source_candidates(
        state,
        candidates.as_slice(),
        document.workspace_folder_uri.as_deref(),
    );
    let value = render_source_hover_definitions_markdown(state, definitions.as_slice())
        .unwrap_or_else(|| format!("**`.{}`**", candidate.name));

    let mut response = json!({
        "contents": {
            "kind": "markdown",
            "value": value,
        },
        "range": candidate.range,
    });
    let provider_feedback =
        provider_tier_feedback_for_hover_definitions(state, definitions.as_slice());
    attach_provider_tier_feedback(&mut response, provider_feedback.as_ref());
    response
}

fn resolve_source_lsp_definition(
    state: &dyn LspQueryReadView,
    document: &LspTextDocumentState,
    position: ParserPositionV0,
) -> Value {
    let candidates = source_selector_candidates_at_position(state, document, position);
    if candidates.is_empty() {
        return Value::Null;
    };
    let definitions = style_selector_definitions_for_source_candidates(
        state,
        candidates.as_slice(),
        document.workspace_folder_uri.as_deref(),
    );
    if definitions.is_empty() {
        return Value::Null;
    }

    json!(
        definitions
            .into_iter()
            .map(|(uri, definition)| json!({ "uri": uri, "range": definition.range }))
            .collect::<Vec<_>>()
    )
}

fn resolve_source_lsp_references(
    state: &LspShellState,
    document: &LspTextDocumentState,
    position: ParserPositionV0,
    params: Option<&Value>,
) -> Value {
    let candidates = source_selector_candidates_at_position(state, document, position);
    if candidates.is_empty() {
        return Value::Null;
    };
    let include_declaration = include_declaration_from_params(params);
    let mut locations = Vec::new();
    if include_declaration {
        locations.extend(
            style_selector_definitions_for_source_candidates(
                state,
                candidates.as_slice(),
                document.workspace_folder_uri.as_deref(),
            )
            .into_iter()
            .map(|(uri, definition)| json!({ "uri": uri, "range": definition.range })),
        );
    }
    for candidate in candidates {
        if candidate.kind == "sourceSelectorPrefixReference" {
            let definitions = style_selector_definitions_from_open_documents(
                state,
                "",
                document.workspace_folder_uri.as_deref(),
            );
            for selector_name in source_candidate_selector_names(
                &candidate,
                definitions.as_slice(),
                candidate.target_style_uri.as_deref(),
            ) {
                locations.extend(selector_reference_locations_from_open_documents(
                    state,
                    selector_name.as_str(),
                    document.workspace_folder_uri.as_deref(),
                    candidate.target_style_uri.as_deref(),
                ));
            }
        } else {
            locations.extend(selector_reference_locations_from_open_documents(
                state,
                candidate.name.as_str(),
                document.workspace_folder_uri.as_deref(),
                candidate.target_style_uri.as_deref(),
            ));
        }
    }
    locations.sort_by_key(location_sort_key);
    locations.dedup();

    if locations.is_empty() {
        Value::Null
    } else {
        json!(locations)
    }
}

fn resolve_source_lsp_completion(
    state: &LspShellState,
    document: &LspTextDocumentState,
    params: Option<&Value>,
) -> Value {
    let Some(position) = lsp_position_from_params(params) else {
        return Value::Null;
    };
    let Some(context) = source_completion_context_at_position(state, document, position) else {
        return Value::Null;
    };
    if !context.domain_option_names.is_empty() {
        let items = source_domain_option_completion_items(
            context.domain_option_names.as_slice(),
            context.value_prefix.as_deref(),
        )
        .into_iter()
        .map(|item| lsp_completion_item_from_query("source", item, None))
        .collect::<Vec<_>>();
        return json!({
            "isIncomplete": false,
            "items": items,
        });
    }
    let inferred_target_style_uri = context.target_style_uri.clone().or_else(|| {
        source_selector_candidate_at_position(state, document, position)
            .and_then(|candidate| candidate.target_style_uri)
    });
    let target_style_uri = inferred_target_style_uri
        .as_deref()
        .map(|uri| external_document_uri_for_query_uri(state, uri));
    let style_sources = style_sources_from_open_documents(
        state,
        document.workspace_folder_uri.as_deref(),
        target_style_uri.as_deref(),
    );
    let resolution_inputs =
        resolution_inputs_for_workspace_uri(state, document.workspace_folder_uri.as_deref());

    let definitions = style_selector_definitions_from_open_documents(
        state,
        "",
        document.workspace_folder_uri.as_deref(),
    )
    .into_iter()
    .filter(|(uri, _)| {
        target_style_uri
            .as_deref()
            .is_none_or(|target_uri| file_uri_equivalent(target_uri, uri))
    })
    .collect::<Vec<_>>();
    let candidates = definitions
        .iter()
        .map(|(uri, definition)| {
            let file_uri = target_style_uri
                .as_deref()
                .filter(|target_uri| file_uri_equivalent(target_uri, uri.as_str()))
                .map(ToString::to_string)
                .unwrap_or_else(|| uri.clone());
            OmenaQueryCompletionCandidateV0 {
                file_uri,
                name: definition.name.clone(),
                kind: definition.kind,
                range: definition.range,
                source: definition.source,
                documentation: None,
            }
        })
        .collect::<Vec<_>>();
    let mut completion = summarize_omena_query_source_completion_at_position(
        document.uri.as_str(),
        position,
        candidates.as_slice(),
        target_style_uri.as_deref(),
        context.value_prefix.as_deref(),
        context.preferred_selector_names.as_slice(),
    );
    // Cascade documentation is attached lazily AFTER ranking/dedup and only for the
    // top-ranked items a completion list actually surfaces. The name-independent
    // narrowing inputs come from the memoized substrate (rfcs#63 E-ii) — fetched once
    // per request, reused across requests while the corpus is unchanged — so the
    // per-candidate work is the cheap per-name filter, not a whole-corpus collection.
    let mut narrowing_substrate = None;
    for item in completion
        .items
        .iter_mut()
        .take(SOURCE_COMPLETION_DOCUMENTATION_BUDGET)
    {
        if item.item_kind != "cssModuleSelector" || item.documentation.is_some() {
            continue;
        }
        let Some((uri, definition)) = definitions
            .iter()
            .find(|(_, definition)| definition.kind == "selector" && definition.name == item.label)
        else {
            continue;
        };
        let narrowing_substrate = narrowing_substrate.get_or_insert_with(|| {
            cascade_narrowing_substrate_for_style_sources(
                state,
                style_sources.as_slice(),
                &resolution_inputs,
            )
        });
        item.documentation = style_text_for_uri(state, uri.as_str()).and_then(|style_text| {
            summarize_omena_query_style_completion_candidate_documentation_for_workspace_file_with_substrate(
                uri.as_str(),
                style_sources.as_slice(),
                narrowing_substrate,
                definition.kind,
                definition.name.as_str(),
                definition.range.start,
            )
            .or_else(|| {
                summarize_omena_query_style_completion_candidate_documentation(
                    style_text.as_str(),
                    definition.kind,
                    definition.name.as_str(),
                    definition.range.start,
                )
            })
        });
    }
    let completion = completion;
    let provider_feedback = target_style_uri
        .as_deref()
        .and_then(|uri| state.document(uri))
        .and_then(|target_document| {
            current_provider_tier_feedback_data(target_document, "textDocument/completion")
        });
    let items: Vec<Value> = completion
        .items
        .into_iter()
        .map(|item| {
            lsp_completion_item_from_query(completion.file_kind, item, provider_feedback.as_ref())
        })
        .collect();

    json!({
        "isIncomplete": false,
        "items": items,
    })
}

fn render_source_hover_definitions_markdown(
    state: &dyn LspQueryReadView,
    definitions: &[(String, LspStyleHoverCandidate)],
) -> Option<String> {
    let parts = definitions
        .iter()
        .filter_map(|(uri, definition)| {
            style_text_for_uri(state, uri).map(|text| {
                render_style_hover_candidate_markdown_for_workspace(
                    state,
                    uri.as_str(),
                    text.as_str(),
                    definition,
                )
            })
        })
        .collect::<Vec<_>>();
    if parts.is_empty() {
        None
    } else {
        Some(parts.join("\n\n---\n\n"))
    }
}

#[cfg(test)]
mod tests;

/// Compact workspace status for the client's status surface. Change-driven:
/// the loop sends `omena/status` only when this tuple moves, so the wire
/// cost is bounded by real state transitions, not by tick rate.
pub fn workspace_status_snapshot(state: &LspShellState) -> (usize, usize, bool, usize) {
    (
        state.workspace_index_pending_file_count,
        state.document_count(),
        diagnostics_follow_up::workspace_republish_frontier_passed(state),
        state.resolution.external_sifs.len(),
    )
}

pub fn workspace_status_notification(
    (pending, indexed, settled, external_sifs): (usize, usize, bool, usize),
) -> Value {
    json!({
        "jsonrpc": "2.0",
        "method": "omena/status",
        "params": {
            "pendingFiles": pending,
            "indexedDocuments": indexed,
            "settled": settled,
            "externalTokenSources": external_sifs,
        },
    })
}
