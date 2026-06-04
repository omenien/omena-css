mod boundary;
mod diagnostics_scheduler;
mod frame_aware_refresh;
mod message_loop;
mod protocol;
mod query_adapter;
mod query_reuse;
mod settings;
mod source_type_facts;
mod state;
mod workspace_index;
mod workspace_runtime_registry;

pub use boundary::*;
pub use frame_aware_refresh::*;
#[cfg(test)]
pub(crate) use message_loop::current_time_millis;
pub use message_loop::{handle_lsp_message, handle_lsp_message_outputs};
use omena_query::{
    OmenaQueryCodeActionV0, OmenaQueryCompletionCandidateV0, OmenaQueryCompletionItemV0,
    OmenaQueryEngineInputV2, OmenaQueryExternalModuleModeV0, OmenaQuerySourceDocumentInputV0,
    OmenaQuerySourceImportedStyleBindingV0 as ImportedStyleBinding,
    OmenaQuerySourceMissingSelectorDiagnosticCandidateV0,
    OmenaQuerySourceSelectorReferenceFactV0 as SourceSelectorReferenceFact,
    OmenaQuerySourceSelectorReferenceMatchKindV0 as SourceSelectorReferenceMatchKind,
    OmenaQuerySourceSyntaxIndexV0 as SourceSyntaxIndex, OmenaQueryStyleDiagnosticV0,
    OmenaQueryStylePackageManifestV0, OmenaQueryStyleSourceInputV0, ParserByteSpanV0,
    ParserPositionV0, ParserRangeV0, StyleLanguage, collect_omena_query_vue_style_module_bindings,
    is_omena_query_sass_symbol_candidate_kind as is_sass_symbol_candidate_kind,
    is_omena_query_sass_symbol_declaration_kind as is_sass_symbol_declaration_kind,
    is_omena_query_sass_symbol_reference_kind as is_sass_symbol_reference_kind,
    load_omena_query_workspace_style_resolution_inputs,
    omena_query_sass_symbol_kind_from_candidate_kind as sass_symbol_kind_from_candidate_kind,
    omena_query_sass_symbol_target_matches,
    read_omena_query_cascade_at_position_with_categorical_evidence,
    read_omena_query_style_context_index, resolve_omena_query_sass_forward_sources,
    resolve_omena_query_sass_module_use_sources_for_candidate,
    resolve_omena_query_sass_symbol_declarations,
    resolve_omena_query_source_candidate_selector_names,
    resolve_omena_query_source_provider_candidates,
    resolve_omena_query_style_selector_definitions_for_source_candidate,
    resolve_omena_query_style_uri_for_specifier_with_resolution_inputs,
    summarize_omena_query_refs_for_class, summarize_omena_query_rename_plan,
    summarize_omena_query_sass_module_sources, summarize_omena_query_source_completion_at_position,
    summarize_omena_query_source_diagnostics_for_file,
    summarize_omena_query_source_diagnostics_for_workspace_file,
    summarize_omena_query_source_import_declarations_for_source_language,
    summarize_omena_query_source_syntax_index_for_source_language,
    summarize_omena_query_style_completion_at_position,
    summarize_omena_query_style_diagnostics_for_file,
    summarize_omena_query_style_diagnostics_for_workspace_file_with_external_mode_and_sifs_and_resolution_inputs,
    summarize_omena_query_style_document, summarize_omena_query_style_extract_code_actions,
    summarize_omena_query_style_hover_render_parts,
    summarize_omena_query_style_inline_code_actions,
};
use omena_streaming_ifds::summarize_streaming_ifds_workspace_cross_file_reachability_v0;
#[cfg(test)]
pub(crate) use omena_tsgo_client::{TsgoResolvedTypeV0, TsgoTypeFactResultEntryV0};
use protocol::*;
use query_adapter::*;
use query_reuse::refresh_document_reusable_indexes;
use serde_json::{Value, json};
pub(crate) use settings::{
    apply_diagnostic_settings, apply_feature_settings, apply_resolution_settings,
};
#[cfg(test)]
pub(crate) use source_type_facts::apply_source_type_fact_results_to_document;
pub(crate) use source_type_facts::refresh_source_type_fact_candidates_for_document;
pub use state::*;
#[cfg(test)]
use std::path::Path;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
};
pub(crate) use workspace_index::index_workspace_style_files;
#[cfg(test)]
pub(crate) use workspace_index::{
    WorkspaceStyleIndexBudget, index_workspace_style_files_with_budget,
};

pub const NODE_TEXT_DOCUMENT_SYNC_KIND: u8 = 2;
pub const DEBUG_STATE_REQUEST: &str = "omena/rustLspState";
pub const RUNTIME_LOOP_PROBE_REQUEST: &str = "omena/runtimeLoopProbe";
pub const STYLE_HOVER_CANDIDATES_REQUEST: &str = "omena/rustStyleHoverCandidates";
pub const STYLE_DIAGNOSTICS_REQUEST: &str = "omena/rustStyleDiagnostics";
pub const SOURCE_DIAGNOSTICS_REQUEST: &str = "omena/rustSourceDiagnostics";
pub const CASCADE_AT_POSITION_REQUEST: &str = "omena/rustCascadeAtPosition";
pub const STYLE_CONTEXT_INDEX_REQUEST: &str = "omena/rustStyleContextIndex";
const CANCEL_REQUEST_METHOD: &str = "$/cancelRequest";
const REQUEST_CANCELLED_ERROR_CODE: i32 = -32800;
#[derive(Debug, Clone, PartialEq, Eq)]
struct SourceProviderCandidateResolution {
    matched: Vec<LspStyleHoverCandidate>,
    unresolved: Vec<LspStyleHoverCandidate>,
}

fn initialize_workspace_folders(state: &mut LspShellState, params: Option<&Value>) {
    state.workspace_runtime_registry.clear();
    if let Some(folders) = params
        .and_then(|value| value.get("workspaceFolders"))
        .and_then(Value::as_array)
    {
        for folder in folders {
            insert_workspace_folder(state, folder);
        }
        refresh_workspace_resolution_inputs(state);
        return;
    }

    if let Some(root_uri) = params
        .and_then(|value| value.get("rootUri"))
        .and_then(Value::as_str)
    {
        state
            .workspace_runtime_registry
            .insert(root_uri.to_string(), root_uri.to_string());
    }
    refresh_workspace_resolution_inputs(state);
}

fn refresh_workspace_resolution_inputs(state: &mut LspShellState) {
    let configured_package_manifests = state.resolution.package_manifests.clone();
    let workspace_uris = state
        .workspace_runtime_registry
        .folder_snapshots()
        .into_iter()
        .map(|folder| folder.uri)
        .collect::<BTreeSet<_>>();
    state
        .resolution
        .workspace_style_resolution_inputs
        .retain(|workspace_uri, _| workspace_uris.contains(workspace_uri));
    for workspace_uri in workspace_uris {
        let inputs = load_omena_query_workspace_style_resolution_inputs(
            Some(workspace_uri.as_str()),
            configured_package_manifests.as_slice(),
        );
        state
            .resolution
            .workspace_style_resolution_inputs
            .insert(workspace_uri, inputs);
    }
}

fn refresh_workspace_resolution_inputs_for_uri(state: &mut LspShellState, uri: &str) {
    let Some(workspace_uri) = resolve_workspace_folder_uri(state, uri) else {
        return;
    };
    let inputs = load_omena_query_workspace_style_resolution_inputs(
        Some(workspace_uri.as_str()),
        state.resolution.package_manifests.as_slice(),
    );
    state
        .resolution
        .workspace_style_resolution_inputs
        .insert(workspace_uri, inputs);
}

fn resolution_inputs_for_workspace_uri(
    state: &LspShellState,
    workspace_folder_uri: Option<&str>,
) -> omena_query::OmenaQueryStyleResolutionInputsV0 {
    workspace_folder_uri
        .and_then(|workspace_uri| {
            state
                .resolution
                .workspace_style_resolution_inputs
                .get(workspace_uri)
        })
        .cloned()
        .unwrap_or_else(|| omena_query::OmenaQueryStyleResolutionInputsV0 {
            package_manifests: state.resolution.package_manifests.clone(),
            ..Default::default()
        })
}

fn lsp_text_document_state(
    uri: String,
    workspace_folder_uri: Option<String>,
    language_id: String,
    version: i64,
    text: String,
    resolution_inputs: &omena_query::OmenaQueryStyleResolutionInputsV0,
) -> LspTextDocumentState {
    let mut document = LspTextDocumentState {
        uri,
        workspace_folder_uri,
        language_id,
        version,
        text,
        style_summary: None,
        style_candidates: Vec::new(),
        source_syntax_index: SourceSyntaxIndex::default(),
        source_selector_candidates: Vec::new(),
    };
    refresh_document_reusable_indexes(&mut document, resolution_inputs);
    document
}

fn did_open_text_document(state: &mut LspShellState, params: Option<&Value>) {
    let Some(document) = params.and_then(|value| value.get("textDocument")) else {
        return;
    };
    let Some(uri) = document.get("uri").and_then(Value::as_str) else {
        return;
    };

    state.insert_open_document_uri(uri);
    let workspace_folder_uri = resolve_workspace_folder_uri(state, uri);
    let resolution_inputs =
        resolution_inputs_for_workspace_uri(state, workspace_folder_uri.as_deref());
    state.insert_document(
        uri,
        lsp_text_document_state(
            uri.to_string(),
            workspace_folder_uri,
            document
                .get("languageId")
                .and_then(Value::as_str)
                .unwrap_or("unknown")
                .to_string(),
            document.get("version").and_then(Value::as_i64).unwrap_or(0),
            document
                .get("text")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string(),
            &resolution_inputs,
        ),
    );
    if is_style_document_uri(uri) {
        refresh_source_type_fact_candidates_for_referencing_documents(state, uri);
    } else {
        refresh_source_type_fact_candidates_for_document(state, uri);
    }
}

fn did_change_text_document(state: &mut LspShellState, params: Option<&Value>) {
    let Some(text_document) = params.and_then(|value| value.get("textDocument")) else {
        return;
    };
    let Some(uri) = text_document.get("uri").and_then(Value::as_str) else {
        return;
    };
    let resolution_inputs = state
        .document(uri)
        .map(|document| {
            resolution_inputs_for_workspace_uri(state, document.workspace_folder_uri.as_deref())
        })
        .unwrap_or_else(|| resolution_inputs_for_workspace_uri(state, None));
    let Some(existing) = state.document_mut(uri) else {
        return;
    };

    if let Some(version) = text_document.get("version").and_then(Value::as_i64) {
        existing.version = version;
    }
    let Some(changes) = params
        .and_then(|value| value.get("contentChanges"))
        .and_then(Value::as_array)
    else {
        return;
    };

    let mut text_changed = false;
    for change in changes {
        if apply_text_document_content_change(existing, change) {
            text_changed = true;
        }
    }
    if text_changed {
        refresh_document_reusable_indexes(existing, &resolution_inputs);
    }
    if text_changed {
        if is_style_document_uri(uri) {
            refresh_source_type_fact_candidates_for_referencing_documents(state, uri);
        } else {
            refresh_source_type_fact_candidates_for_document(state, uri);
        }
    }
}

fn apply_text_document_content_change(document: &mut LspTextDocumentState, change: &Value) -> bool {
    let Some(next_text) = change.get("text").and_then(Value::as_str) else {
        return false;
    };
    let Some(range) = change.get("range").and_then(lsp_range_from_value) else {
        document.text = next_text.to_string();
        return true;
    };
    let Some(start_offset) = byte_offset_for_parser_position(document.text.as_str(), range.start)
    else {
        return false;
    };
    let Some(end_offset) = byte_offset_for_parser_position(document.text.as_str(), range.end)
    else {
        return false;
    };
    if start_offset > end_offset {
        return false;
    }
    document
        .text
        .replace_range(start_offset..end_offset, next_text);
    true
}

fn did_close_text_document(state: &mut LspShellState, params: Option<&Value>) {
    let Some(uri) = params
        .and_then(|value| value.get("textDocument"))
        .and_then(|value| value.get("uri"))
        .and_then(Value::as_str)
    else {
        return;
    };
    state.remove_open_document_uri(uri);
    if is_style_document_uri(uri) && reload_indexed_style_document_from_disk(state, uri) {
        refresh_source_type_fact_candidates_for_referencing_documents(state, uri);
        return;
    }
    state.remove_document_uri(uri);
    if is_style_document_uri(uri) {
        refresh_source_type_fact_candidates_for_referencing_documents(state, uri);
    }
}

fn did_change_workspace_folders(state: &mut LspShellState, params: Option<&Value>) {
    let event = params.and_then(|value| value.get("event"));
    let mut removed_workspace_uris = Vec::new();
    if let Some(removed) = event
        .and_then(|value| value.get("removed"))
        .and_then(Value::as_array)
    {
        for folder in removed {
            if let Some(uri) = folder.get("uri").and_then(Value::as_str) {
                state.workspace_runtime_registry.remove(uri);
                removed_workspace_uris.push(uri.to_string());
            }
        }
    }
    let mut added_workspace_folder = false;
    if let Some(added) = event
        .and_then(|value| value.get("added"))
        .and_then(Value::as_array)
    {
        for folder in added {
            insert_workspace_folder(state, folder);
            added_workspace_folder = true;
        }
    }
    reconcile_documents_after_workspace_folder_changes(state, removed_workspace_uris.as_slice());
    refresh_workspace_resolution_inputs(state);
    if added_workspace_folder {
        index_workspace_style_files(state);
    }
}

fn reconcile_documents_after_workspace_folder_changes(
    state: &mut LspShellState,
    removed_workspace_uris: &[String],
) {
    remove_unowned_indexed_documents_for_removed_workspaces(state, removed_workspace_uris);
    refresh_document_workspace_owners(state);
}

fn remove_unowned_indexed_documents_for_removed_workspaces(
    state: &mut LspShellState,
    removed_workspace_uris: &[String],
) {
    if removed_workspace_uris.is_empty() {
        return;
    }
    let open_document_uris = state.open_document_uris.clone();
    let workspace_runtime_registry = state.workspace_runtime_registry.clone();
    state.documents.retain(|uri, document| {
        if open_document_uris.contains(uri)
            || open_document_uris
                .iter()
                .any(|open_uri| file_uri_equivalent(open_uri, uri))
        {
            return true;
        }
        let owned_by_removed_workspace =
            document
                .workspace_folder_uri
                .as_deref()
                .is_some_and(|workspace_uri| {
                    removed_workspace_uris
                        .iter()
                        .any(|removed_uri| removed_uri == workspace_uri)
                });
        !owned_by_removed_workspace
            || workspace_runtime_registry
                .resolve_owner_uri(uri.as_str())
                .is_some()
    });
}

fn did_change_watched_files(state: &mut LspShellState, params: Option<&Value>) {
    let Some(changes) = params
        .and_then(|value| value.get("changes"))
        .and_then(Value::as_array)
    else {
        return;
    };
    for change in changes {
        let Some(uri) = change.get("uri").and_then(Value::as_str) else {
            continue;
        };
        let change_type = change.get("type").and_then(Value::as_u64).unwrap_or(0);
        state.watched_file_changes.push(LspWatchedFileChangeState {
            uri: uri.to_string(),
            change_type,
        });
        apply_watched_file_change_to_index(state, uri, change_type);
    }
}

fn apply_watched_file_change_to_index(state: &mut LspShellState, uri: &str, change_type: u64) {
    if !is_style_document_uri(uri) {
        if is_resolution_config_document_uri(uri) {
            refresh_source_indexes_for_resolution_config_change(state, uri);
        }
        return;
    }
    if state.has_open_document_uri(uri) {
        return;
    }
    if change_type == 3 {
        state.remove_document_uri(uri);
        refresh_source_type_fact_candidates_for_referencing_documents(state, uri);
        return;
    }

    if reload_indexed_style_document_from_disk(state, uri) {
        refresh_source_type_fact_candidates_for_referencing_documents(state, uri);
    }
}

fn refresh_source_indexes_for_resolution_config_change(
    state: &mut LspShellState,
    config_uri: &str,
) {
    refresh_workspace_resolution_inputs_for_uri(state, config_uri);
    let workspace_folder_uri = resolve_workspace_folder_uri(state, config_uri);
    let source_uris = state
        .documents
        .values()
        .filter(|document| !is_style_document_uri(document.uri.as_str()))
        .filter(|document| {
            workspace_folder_uri.as_deref().is_none_or(|workspace_uri| {
                workspace_folder_compatible(Some(workspace_uri), document)
            })
        })
        .map(|document| document.uri.clone())
        .collect::<Vec<_>>();
    for source_uri in source_uris {
        let resolution_inputs = state
            .document(source_uri.as_str())
            .map(|document| {
                resolution_inputs_for_workspace_uri(state, document.workspace_folder_uri.as_deref())
            })
            .unwrap_or_else(|| {
                resolution_inputs_for_workspace_uri(state, workspace_folder_uri.as_deref())
            });
        if let Some(document) = state.document_mut(source_uri.as_str()) {
            refresh_document_reusable_indexes(document, &resolution_inputs);
        }
        refresh_source_type_fact_candidates_for_document(state, source_uri.as_str());
    }
}

pub(crate) fn refresh_source_indexes_for_resolution_settings_change(state: &mut LspShellState) {
    refresh_workspace_resolution_inputs(state);
    let source_uris = state
        .documents
        .values()
        .filter(|document| !is_style_document_uri(document.uri.as_str()))
        .map(|document| document.uri.clone())
        .collect::<Vec<_>>();
    for source_uri in source_uris {
        let resolution_inputs = state
            .document(source_uri.as_str())
            .map(|document| {
                resolution_inputs_for_workspace_uri(state, document.workspace_folder_uri.as_deref())
            })
            .unwrap_or_else(|| resolution_inputs_for_workspace_uri(state, None));
        if let Some(document) = state.document_mut(source_uri.as_str()) {
            refresh_document_reusable_indexes(document, &resolution_inputs);
        }
        refresh_source_type_fact_candidates_for_document(state, source_uri.as_str());
    }
}

pub(crate) fn is_resolution_config_document_uri(uri: &str) -> bool {
    let Some(path) = file_uri_to_path(uri) else {
        return false;
    };
    let Some(file_name) = path.file_name().and_then(|value| value.to_str()) else {
        return false;
    };
    file_name == "package.json"
        || file_name == "jsconfig.json"
        || (file_name.starts_with("tsconfig") && file_name.ends_with(".json"))
        || matches!(
            file_name,
            "vite.config.ts"
                | "vite.config.mts"
                | "vite.config.cts"
                | "vite.config.js"
                | "vite.config.mjs"
                | "vite.config.cjs"
                | "webpack.config.ts"
                | "webpack.config.mts"
                | "webpack.config.cts"
                | "webpack.config.js"
                | "webpack.config.mjs"
                | "webpack.config.cjs"
        )
}

pub(crate) fn ensure_style_document_loaded_from_disk(state: &mut LspShellState, uri: &str) -> bool {
    if state.contains_document_uri(uri) {
        return true;
    }
    reload_indexed_style_document_from_disk(state, uri)
}

fn reload_indexed_style_document_from_disk(state: &mut LspShellState, uri: &str) -> bool {
    let Some(path) = file_uri_to_path(uri) else {
        return false;
    };
    let Ok(text) = fs::read_to_string(path) else {
        return false;
    };
    let workspace_folder_uri = resolve_workspace_folder_uri(state, uri);
    let resolution_inputs =
        resolution_inputs_for_workspace_uri(state, workspace_folder_uri.as_deref());
    state.insert_document(
        uri,
        lsp_text_document_state(
            uri.to_string(),
            workspace_folder_uri,
            StyleLanguage::from_module_path(uri)
                .map(style_language_label)
                .unwrap_or("unknown")
                .to_string(),
            0,
            text,
            &resolution_inputs,
        ),
    );
    true
}

fn refresh_source_type_fact_candidates_for_referencing_documents(
    state: &mut LspShellState,
    style_uri: &str,
) {
    let source_uris = state
        .documents
        .values()
        .filter(|document| !is_style_document_uri(document.uri.as_str()))
        .filter(|document| document_references_style_uri(document, style_uri))
        .map(|document| document.uri.clone())
        .collect::<Vec<_>>();
    for source_uri in source_uris {
        refresh_source_type_fact_candidates_for_document(state, source_uri.as_str());
    }
}

fn document_references_style_uri(document: &LspTextDocumentState, style_uri: &str) -> bool {
    document
        .source_syntax_index
        .selector_references
        .iter()
        .any(|reference| reference.target_style_uri.as_deref() == Some(style_uri))
        || document
            .source_syntax_index
            .type_fact_targets
            .iter()
            .any(|target| target.target_style_uri.as_deref() == Some(style_uri))
}

fn insert_workspace_folder(state: &mut LspShellState, folder: &Value) {
    let Some(uri) = folder.get("uri").and_then(Value::as_str) else {
        return;
    };
    state.workspace_runtime_registry.insert(
        uri.to_string(),
        folder
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or(uri)
            .to_string(),
    );
}

fn refresh_document_workspace_owners(state: &mut LspShellState) {
    let workspace_runtime_registry = state.workspace_runtime_registry.clone();
    for document in state.documents.values_mut() {
        document.workspace_folder_uri =
            workspace_runtime_registry.resolve_owner_uri(document.uri.as_str());
    }
}

fn resolve_workspace_folder_uri(state: &LspShellState, document_uri: &str) -> Option<String> {
    state
        .workspace_runtime_registry
        .resolve_owner_uri(document_uri)
}

fn summarize_style_document(uri: &str, text: Option<&str>) -> Option<LspStyleDocumentSummary> {
    let text = text?;
    let summary = summarize_omena_query_style_document(uri, text)?;
    Some(LspStyleDocumentSummary {
        language: summary.language,
        selector_names: summary.selector_names,
        custom_property_decl_names: summary.custom_property_decl_names,
        custom_property_ref_names: summary.custom_property_ref_names,
        sass_module_use_sources: summary.sass_module_use_sources,
        sass_module_forward_sources: summary.sass_module_forward_sources,
        diagnostic_count: summary.diagnostic_count,
    })
}

pub fn resolve_style_hover_candidates(
    state: &LspShellState,
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

fn style_text_for_uri(state: &LspShellState, uri: &str) -> Option<String> {
    state
        .document(uri)
        .map(|document| document.text.clone())
        .or_else(|| fs::read_to_string(file_uri_to_path(uri)?).ok())
}

fn style_hover_candidates_for_uri(
    state: &LspShellState,
    uri: &str,
) -> Option<(&'static str, Vec<LspStyleHoverCandidate>)> {
    if let Some(document) = state.document(uri) {
        return style_hover_candidates_for_document(document);
    }
    let text = style_text_for_uri(state, uri)?;
    collect_style_hover_candidates(uri, text.as_str())
}

fn resolve_lsp_definition(state: &LspShellState, params: Option<&Value>) -> Value {
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
        if definitions.is_empty() {
            return Value::Null;
        }
        return json!(
            definitions
                .into_iter()
                .map(|(uri, definition)| json!({ "uri": uri, "range": definition.range }))
                .collect::<Vec<_>>()
        );
    }
    let target = if candidate.kind == "customPropertyReference" {
        candidates
            .iter()
            .find(|target| {
                target.kind == "customPropertyDeclaration" && target.name == candidate.name
            })
            .unwrap_or(candidate)
    } else {
        candidate
    };

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
    let mut locations: Vec<Value> = if candidate.kind.starts_with("customProperty") {
        candidates
            .iter()
            .filter(|target| {
                target.name == candidate.name
                    && (target.kind == "customPropertyReference"
                        || (include_declaration && target.kind == "customPropertyDeclaration"))
            })
            .map(|target| json!({ "uri": document.uri.as_str(), "range": target.range }))
            .collect()
    } else if is_sass_symbol_candidate_kind(candidate.kind) {
        let mut locations = Vec::new();
        if include_declaration {
            locations.extend(
                sass_symbol_definitions_for_candidate(state, document, candidate)
                    .into_iter()
                    .map(|(uri, definition)| json!({ "uri": uri, "range": definition.range })),
            );
        }
        locations.extend(
            candidates
                .iter()
                .filter(|target| sass_symbol_reference_matches(candidate, target))
                .map(|target| json!({ "uri": document.uri.as_str(), "range": target.range })),
        );
        locations
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
    let Some((_, candidates)) = style_hover_candidates_for_document(document) else {
        return Value::Null;
    };

    let query_candidates = candidates
        .iter()
        .map(query_style_hover_candidate_from_lsp)
        .collect::<Vec<_>>();
    let completion = summarize_omena_query_style_completion_at_position(
        document.uri.as_str(),
        document.text.as_str(),
        position,
        query_candidates.as_slice(),
    );
    let items: Vec<Value> = completion
        .items
        .into_iter()
        .map(|item| lsp_completion_item_from_query(completion.file_kind, item))
        .collect();

    json!({
        "isIncomplete": false,
        "items": items,
    })
}

fn lsp_completion_item_from_query(file_kind: &str, item: OmenaQueryCompletionItemV0) -> Value {
    let kind = match (file_kind, item.item_kind) {
        ("style", "cssModuleSelector") => 7,
        (_, "cssModuleSelector") | (_, "cssCustomProperty") => 10,
        _ => 1,
    };
    json!({
        "label": item.label,
        "kind": kind,
        "sortText": item.sort_text,
        "detail": item.detail,
        "insertText": item.insert_text,
        "data": {
            "source": item.source,
            "rankingSource": item.ranking_source,
        },
    })
}

fn resolve_style_diagnostics(state: &LspShellState, params: Option<&Value>) -> Value {
    let document_uri = document_uri_from_params(params);
    resolve_style_diagnostics_for_uri(state, document_uri.as_str())
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

fn resolve_style_context_index(state: &LspShellState, params: Option<&Value>) -> Value {
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

fn query_engine_input_from_params(params: Option<&Value>) -> Option<OmenaQueryEngineInputV2> {
    if let Some(engine_input) = params.and_then(|value| value.get("engineInput")) {
        return serde_json::from_value(engine_input.clone()).ok();
    }

    serde_json::from_value(json!({
        "version": "2",
        "sources": [],
        "styles": [],
        "typeFacts": [],
    }))
    .ok()
}

fn resolve_document_diagnostics_for_uri(state: &LspShellState, document_uri: &str) -> Value {
    if is_style_document_uri(document_uri) {
        resolve_style_diagnostics_for_uri(state, document_uri)
    } else {
        resolve_source_diagnostics_for_uri(state, document_uri)
    }
}

fn resolve_style_diagnostics_for_uri(state: &LspShellState, document_uri: &str) -> Value {
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
    // #35: drive the workspace path in `Sif` mode whenever external SIF artifacts are available
    // (sourced from the lock/bridge per #32/#33). That branch is what classifies the external
    // boundary lattice and parses the `@omena-strict:` sigil; with no SIFs present we fall back to
    // `Ignored`, which is byte-for-byte the legacy behaviour.
    let external_sifs = state.resolution.external_sifs.as_slice();
    let external_mode = if external_sifs.is_empty() {
        OmenaQueryExternalModuleModeV0::Ignored
    } else {
        OmenaQueryExternalModuleModeV0::Sif
    };
    // RFC-0007-J (#50): pass the workspace's tsconfig/bundler path mappings so the unused-selector
    // usage collector resolves alias style imports (`@/styles/...`) the same way the reference/goto
    // path does — otherwise an alias import dims every selector as `unusedSelector`.
    let resolution_inputs =
        resolution_inputs_for_workspace_uri(state, document.workspace_folder_uri.as_deref());
    let mut diagnostics_summary =
        summarize_omena_query_style_diagnostics_for_workspace_file_with_external_mode_and_sifs_and_resolution_inputs(
            document.uri.as_str(),
            style_sources.as_slice(),
            source_documents.as_slice(),
            state.resolution.package_manifests.as_slice(),
            None,
            external_mode,
            external_sifs,
            &resolution_inputs,
        )
        .unwrap_or_else(|| {
            summarize_omena_query_style_diagnostics_for_file(
                document.uri.as_str(),
                document.text.as_str(),
                query_candidates.as_slice(),
            )
        });
    diagnostics_summary.diagnostics.extend(
        summarize_cross_file_streaming_reachability_diagnostics_for_lsp(
            document.uri.as_str(),
            style_sources.as_slice(),
            source_documents.as_slice(),
            state.resolution.package_manifests.as_slice(),
        ),
    );
    diagnostics_summary.diagnostic_count = diagnostics_summary.diagnostics.len();
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

            let mut lsp_diagnostic = json!({
                "range": diagnostic.range,
                "severity": lsp_diagnostic_severity(query_severity, state.diagnostics.severity),
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

/// Surface streaming-IFDS cross-file reachability through the live LSP style
/// diagnostics path. The mechanism remains owned by `omena-streaming-ifds`;
/// LSP only renders the returned report as a product diagnostic.
fn summarize_cross_file_streaming_reachability_diagnostics_for_lsp(
    target_style_path: &str,
    style_sources: &[OmenaQueryStyleSourceInputV0],
    source_documents: &[OmenaQuerySourceDocumentInputV0],
    package_manifests: &[OmenaQueryStylePackageManifestV0],
) -> Vec<OmenaQueryStyleDiagnosticV0> {
    let report = summarize_streaming_ifds_workspace_cross_file_reachability_v0(
        target_style_path,
        style_sources,
        source_documents,
        package_manifests,
    );
    if report.reachable_foreign_paths.is_empty() {
        return Vec::new();
    }

    let reachable_modules = report.reachable_foreign_paths.join(", ");
    vec![OmenaQueryStyleDiagnosticV0 {
        code: "crossFileStreamingReachability",
        severity: "hint",
        provenance: vec![
            "omena-lsp-server.style-diagnostics",
            "omena-streaming-ifds.cross-file-reachability-report",
            "omena-streaming-ifds.analysis-report",
            "omena-query.unified-cross-file-hypergraph",
            "omena-query.cross-file-summary",
        ],
        range: ParserRangeV0::default(),
        message: format!(
            "cross-file dataflow reaches {} module(s) via resolved edges: {reachable_modules}",
            report.reachable_foreign_path_count
        ),
        tags: Vec::new(),
        create_custom_property: None,
    }]
}

fn resolve_source_diagnostics_for_uri(state: &LspShellState, document_uri: &str) -> Value {
    let Some(document) = state.document(document_uri) else {
        return json!([]);
    };
    if is_style_document_uri(document.uri.as_str()) {
        return json!([]);
    }

    let style_sources =
        style_sources_from_open_documents(state, document.workspace_folder_uri.as_deref(), None);
    let mut query_diagnostics = summarize_omena_query_source_diagnostics_for_workspace_file(
        document.uri.as_str(),
        document.text.as_str(),
        style_sources.as_slice(),
        state.resolution.package_manifests.as_slice(),
    )
    .diagnostics
    .into_iter()
    // The LSP source index already resolves tsconfig/bundler aliases. Keep module-resolution
    // diagnostics on that path until the workspace summary accepts the same resolution inputs.
    .filter(|diagnostic| diagnostic.code != "missingModule")
    .collect::<Vec<_>>();
    let query_resolved_source_diagnostic_keys = query_diagnostics
        .iter()
        .filter(|diagnostic| {
            matches!(
                diagnostic.code,
                "missingStaticClass"
                    | "missingTemplatePrefix"
                    | "missingResolvedClassValues"
                    | "missingResolvedClassDomain"
            )
        })
        .filter_map(|diagnostic| {
            diagnostic
                .create_selector
                .as_ref()
                .map(|create_selector| (diagnostic.range, create_selector.selector_name.clone()))
        })
        .collect::<BTreeSet<_>>();

    let candidates = resolve_source_provider_candidates(state, document)
        .unresolved
        .into_iter()
        .filter(|candidate| candidate.kind == "sourceSelectorReference")
        .filter_map(|candidate| {
            if query_resolved_source_diagnostic_keys
                .contains(&(candidate.range, candidate.name.clone()))
            {
                return None;
            }
            let (target_style_uri, target_style_document) = source_selector_diagnostic_target(
                state,
                &candidate,
                document.workspace_folder_uri.as_deref(),
            )?;
            Some(OmenaQuerySourceMissingSelectorDiagnosticCandidateV0 {
                target_style_uri,
                target_style_source: target_style_document.text.clone(),
                selector_name: candidate.name,
                source_reference_range: candidate.range,
            })
        })
        .collect::<Vec<_>>();
    query_diagnostics.extend(
        summarize_omena_query_source_diagnostics_for_file(
            document.uri.as_str(),
            candidates.as_slice(),
        )
        .diagnostics,
    );
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
            if let Some(create_selector) = diagnostic.create_selector {
                data.insert("createSelector".to_string(), json!(create_selector));
            }

            json!({
                "range": diagnostic.range,
                "severity": lsp_diagnostic_severity(query_severity, state.diagnostics.severity),
                "code": diagnostic.code,
                "source": "omena-css",
                "message": diagnostic.message,
                "data": Value::Object(data),
            })
        })
        .collect();

    json!(diagnostics)
}

fn lsp_diagnostic_severity(query_severity: &str, configured_severity: u8) -> u8 {
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

fn resolve_lsp_code_actions(state: &LspShellState, params: Option<&Value>) -> Value {
    let diagnostics = params
        .and_then(|value| value.get("context"))
        .and_then(|value| value.get("diagnostics"))
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or(&[]);

    let mut actions: Vec<Value> = diagnostics
        .iter()
        .enumerate()
        .filter_map(|(index, diagnostic)| {
            let payload = diagnostic
                .pointer("/data/createCustomProperty")
                .and_then(Value::as_object)?;
            let uri = payload.get("uri").and_then(Value::as_str)?;
            let range = payload.get("range")?;
            let new_text = payload.get("newText").and_then(Value::as_str)?;
            let property_name = payload.get("propertyName").and_then(Value::as_str)?;
            let mut changes = serde_json::Map::new();
            changes.insert(
                uri.to_string(),
                json!([
                    {
                        "range": range,
                        "newText": new_text,
                    },
                ]),
            );

            Some(json!({
                "title": format!("Add '{}' to {}", property_name, file_label_from_uri(uri)),
                "kind": "quickfix",
                "diagnostics": [diagnostic],
                "edit": {
                    "changes": Value::Object(changes),
                },
                "data": {
                    "source": "omenaQueryStyleDiagnosticsForFile",
                    "diagnosticIndex": index,
                },
            }))
        })
        .chain(diagnostics.iter().enumerate().filter_map(|(index, diagnostic)| {
            let payload = diagnostic
                .pointer("/data/createSelector")
                .and_then(Value::as_object)?;
            let uri = payload.get("uri").and_then(Value::as_str)?;
            let range = payload.get("range")?;
            let new_text = payload.get("newText").and_then(Value::as_str)?;
            let selector_name = payload.get("selectorName").and_then(Value::as_str)?;
            let mut changes = serde_json::Map::new();
            changes.insert(
                uri.to_string(),
                json!([
                    {
                        "range": range,
                        "newText": new_text,
                    },
                ]),
            );

            Some(json!({
                "title": format!("Add '.{}' to {}", selector_name, file_label_from_uri(uri)),
                "kind": "quickfix",
                "diagnostics": [diagnostic],
                "edit": {
                    "changes": Value::Object(changes),
                },
                "data": {
                    "source": "omenaQuerySourceSyntaxIndex",
                    "diagnosticIndex": index,
                },
            }))
        }))
        .collect();

    if diagnostics.is_empty() {
        actions.extend(resolve_lsp_refactor_code_actions(state, params));
    }

    if actions.is_empty() {
        Value::Null
    } else {
        json!(actions)
    }
}

fn resolve_lsp_refactor_code_actions(state: &LspShellState, params: Option<&Value>) -> Vec<Value> {
    let document_uri = document_uri_from_params(params);
    let Some(document) = state.document(document_uri.as_str()) else {
        return Vec::new();
    };
    if !is_style_document_uri(document.uri.as_str()) {
        return Vec::new();
    }
    let Some(range) = params
        .and_then(|value| value.get("range"))
        .and_then(lsp_range_from_value)
    else {
        return Vec::new();
    };

    let style_sources = style_sources_from_open_documents(
        state,
        document.workspace_folder_uri.as_deref(),
        Some(document.uri.as_str()),
    );
    let inline_actions = summarize_omena_query_style_inline_code_actions(
        document.uri.as_str(),
        style_sources.as_slice(),
        range,
        &[],
    )
    .actions;
    if !inline_actions.is_empty() {
        return render_omena_query_lsp_code_actions(inline_actions);
    }

    let extract_actions = summarize_omena_query_style_extract_code_actions(
        document.uri.as_str(),
        document.text.as_str(),
        range,
    )
    .actions;
    render_omena_query_lsp_code_actions(extract_actions)
}

fn style_sources_from_open_documents(
    state: &LspShellState,
    workspace_folder_uri: Option<&str>,
    required_document_uri: Option<&str>,
) -> Vec<OmenaQueryStyleSourceInputV0> {
    let mut sources = state
        .documents
        .values()
        .filter(|document| {
            is_style_document_uri(document.uri.as_str())
                && workspace_folder_compatible(workspace_folder_uri, document)
        })
        .map(|document| OmenaQueryStyleSourceInputV0 {
            style_path: document.uri.clone(),
            style_source: document.text.clone(),
        })
        .collect::<Vec<_>>();
    if let Some(required_document_uri) = required_document_uri
        && !sources
            .iter()
            .any(|source| source.style_path == required_document_uri)
        && let Some(document) = state.document(required_document_uri)
    {
        sources.push(OmenaQueryStyleSourceInputV0 {
            style_path: document.uri.clone(),
            style_source: document.text.clone(),
        });
    }
    sources
}

fn source_documents_from_open_documents(
    state: &LspShellState,
    workspace_folder_uri: Option<&str>,
) -> Vec<OmenaQuerySourceDocumentInputV0> {
    state
        .documents
        .values()
        .filter(|document| {
            !is_style_document_uri(document.uri.as_str())
                && workspace_folder_compatible(workspace_folder_uri, document)
        })
        .map(|document| OmenaQuerySourceDocumentInputV0 {
            source_path: document.uri.clone(),
            source_source: document.text.clone(),
        })
        .collect()
}

fn render_omena_query_lsp_code_actions(actions: Vec<OmenaQueryCodeActionV0>) -> Vec<Value> {
    actions
        .into_iter()
        .enumerate()
        .map(|(index, action)| {
            let mut changes_by_uri = BTreeMap::<String, Vec<Value>>::new();
            for edit in action.edits {
                changes_by_uri.entry(edit.uri).or_default().push(json!({
                    "range": edit.range,
                    "newText": edit.new_text,
                }));
            }

            let changes = changes_by_uri
                .into_iter()
                .map(|(uri, edits)| (uri, Value::Array(edits)))
                .collect::<serde_json::Map<_, _>>();

            json!({
                "title": action.title,
                "kind": action.kind,
                "edit": {
                    "changes": Value::Object(changes),
                },
                "data": {
                    "source": action.source,
                    "actionIndex": index,
                },
            })
        })
        .collect()
}

fn resolve_lsp_code_lens(state: &LspShellState, params: Option<&Value>) -> Value {
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

fn selector_reference_locations_from_open_documents(
    state: &LspShellState,
    selector_name: &str,
    workspace_folder_uri: Option<&str>,
    target_style_uri: Option<&str>,
) -> Vec<Value> {
    let definitions =
        style_selector_definitions_from_open_documents(state, "", workspace_folder_uri)
            .iter()
            .map(|(uri, definition)| query_style_selector_definition_for_matching(uri, definition))
            .collect::<Vec<_>>();
    let query_target_style_uri = query_target_style_uri_for_matching(target_style_uri);
    let mut references = Vec::new();
    for document in state.documents.values() {
        if is_style_document_uri(document.uri.as_str()) {
            continue;
        }
        if !workspace_folder_compatible(workspace_folder_uri, document) {
            continue;
        }
        references.extend(
            collect_source_selector_reference_candidates(state, document)
                .iter()
                .map(|candidate| {
                    query_source_selector_reference_candidate_for_matching(document, candidate)
                }),
        );
    }
    summarize_omena_query_refs_for_class(
        selector_name,
        query_target_style_uri.as_deref(),
        false,
        definitions.as_slice(),
        references.as_slice(),
    )
    .locations
    .into_iter()
    .map(|location| json!({ "uri": location.uri, "range": location.range }))
    .collect()
}

fn selector_reference_locations_by_name_from_open_documents(
    state: &LspShellState,
    workspace_folder_uri: Option<&str>,
    target_style_uri: Option<&str>,
) -> BTreeMap<String, Vec<Value>> {
    let mut locations_by_name: BTreeMap<String, Vec<Value>> = BTreeMap::new();
    let definitions =
        style_selector_definitions_from_open_documents(state, "", workspace_folder_uri);
    for document in state.documents.values() {
        if is_style_document_uri(document.uri.as_str()) {
            continue;
        }
        if !workspace_folder_compatible(workspace_folder_uri, document) {
            continue;
        }
        for candidate in collect_source_selector_reference_candidates(state, document) {
            if !source_candidate_matches_target_style(&candidate, target_style_uri) {
                continue;
            }
            for selector_name in source_candidate_selector_names(
                &candidate,
                definitions.as_slice(),
                target_style_uri,
            ) {
                locations_by_name
                    .entry(selector_name)
                    .or_default()
                    .push(json!({
                        "uri": document.uri.as_str(),
                        "range": candidate.range,
                    }));
            }
        }
    }
    for locations in locations_by_name.values_mut() {
        locations.sort_by_key(location_sort_key);
        locations
            .dedup_by(|left, right| location_identity_key(left) == location_identity_key(right));
    }
    locations_by_name
}

fn source_candidate_selector_names(
    candidate: &LspStyleHoverCandidate,
    definitions: &[(String, LspStyleHoverCandidate)],
    target_style_uri: Option<&str>,
) -> Vec<String> {
    let query_definitions = definitions
        .iter()
        .map(|(uri, definition)| query_style_selector_definition_for_matching(uri, definition))
        .collect::<Vec<_>>();
    let query_target_style_uri = query_target_style_uri_for_matching(target_style_uri);
    resolve_omena_query_source_candidate_selector_names(
        &query_source_selector_candidate_for_matching(candidate),
        query_definitions.as_slice(),
        query_target_style_uri.as_deref(),
    )
}

fn source_candidate_matches_target_style(
    candidate: &LspStyleHoverCandidate,
    target_style_uri: Option<&str>,
) -> bool {
    target_style_uri.is_none_or(|target_uri| {
        candidate
            .target_style_uri
            .as_deref()
            .is_none_or(|candidate_target_uri| {
                file_uri_equivalent(candidate_target_uri, target_uri)
            })
    })
}

fn sass_symbol_definitions_for_candidate(
    state: &LspShellState,
    document: &LspTextDocumentState,
    candidate: &LspStyleHoverCandidate,
) -> Vec<(String, LspStyleHoverCandidate)> {
    let Some(symbol_kind) = sass_symbol_kind_from_candidate_kind(candidate.kind) else {
        return Vec::new();
    };
    if is_sass_symbol_declaration_kind(candidate.kind) {
        return vec![(document.uri.clone(), candidate.clone())];
    }

    let mut definitions = if candidate.namespace.is_none() {
        sass_symbol_declarations_in_document(document, symbol_kind, candidate)
    } else {
        Vec::new()
    };
    if candidate.namespace.is_none() && !definitions.is_empty() {
        return definitions;
    }

    for target_uri in sass_module_target_uris_for_candidate(state, document, candidate) {
        definitions.extend(sass_symbol_declarations_for_uri(
            state,
            target_uri.as_str(),
            symbol_kind,
            candidate,
        ));
    }
    definitions.sort_by_key(|(uri, target)| {
        (
            uri.clone(),
            target.range.start.line,
            target.range.start.character,
        )
    });
    definitions.dedup_by(|left, right| {
        left.0 == right.0
            && left.1.kind == right.1.kind
            && left.1.name == right.1.name
            && left.1.range == right.1.range
    });
    definitions
}

fn sass_symbol_declarations_for_uri(
    state: &LspShellState,
    target_uri: &str,
    symbol_kind: &str,
    candidate: &LspStyleHoverCandidate,
) -> Vec<(String, LspStyleHoverCandidate)> {
    if let Some(target_document) = state.document(target_uri) {
        return sass_symbol_declarations_with_forwards(
            state,
            target_document,
            symbol_kind,
            candidate,
            &mut BTreeSet::new(),
        );
    }
    let Some((_, candidates)) = style_hover_candidates_for_uri(state, target_uri) else {
        return Vec::new();
    };
    let query_candidates = candidates
        .iter()
        .map(query_style_hover_candidate_from_lsp)
        .collect::<Vec<_>>();
    resolve_omena_query_sass_symbol_declarations(
        query_candidates.as_slice(),
        symbol_kind,
        candidate.name.as_str(),
    )
    .into_iter()
    .map(lsp_style_hover_candidate_from_query)
    .map(|target| (target_uri.to_string(), target))
    .collect()
}

fn sass_symbol_declarations_in_document(
    document: &LspTextDocumentState,
    symbol_kind: &str,
    candidate: &LspStyleHoverCandidate,
) -> Vec<(String, LspStyleHoverCandidate)> {
    let query_candidates = document
        .style_candidates
        .iter()
        .map(query_style_hover_candidate_from_lsp)
        .collect::<Vec<_>>();
    resolve_omena_query_sass_symbol_declarations(
        query_candidates.as_slice(),
        symbol_kind,
        candidate.name.as_str(),
    )
    .into_iter()
    .map(lsp_style_hover_candidate_from_query)
    .map(|target| (document.uri.clone(), target))
    .collect()
}

fn sass_module_target_uris_for_candidate(
    state: &LspShellState,
    document: &LspTextDocumentState,
    candidate: &LspStyleHoverCandidate,
) -> Vec<String> {
    let Some(sources) =
        summarize_omena_query_sass_module_sources(document.uri.as_str(), document.text.as_str())
    else {
        return Vec::new();
    };
    let mut uris = Vec::new();
    for source in resolve_omena_query_sass_module_use_sources_for_candidate(
        &sources,
        candidate.namespace.as_deref(),
    ) {
        if let Some(uri) = resolve_lsp_style_uri_for_specifier(state, document, source.as_str()) {
            uris.push(uri);
        }
    }
    for forward_source in resolve_omena_query_sass_forward_sources(&sources) {
        if let Some(uri) =
            resolve_lsp_style_uri_for_specifier(state, document, forward_source.as_str())
        {
            uris.push(uri.clone());
            if let Some(target_document) = state.document(uri.as_str()) {
                uris.extend(sass_forward_module_target_uris(
                    state,
                    target_document,
                    &mut BTreeSet::new(),
                ));
            }
        }
    }
    uris.sort();
    uris.dedup();
    uris
}

fn sass_symbol_declarations_with_forwards(
    state: &LspShellState,
    document: &LspTextDocumentState,
    symbol_kind: &str,
    candidate: &LspStyleHoverCandidate,
    visited: &mut BTreeSet<String>,
) -> Vec<(String, LspStyleHoverCandidate)> {
    if !visited.insert(document.uri.clone()) {
        return Vec::new();
    }
    let mut definitions = sass_symbol_declarations_in_document(document, symbol_kind, candidate);
    let Some(sources) =
        summarize_omena_query_sass_module_sources(document.uri.as_str(), document.text.as_str())
    else {
        return definitions;
    };
    for forward_source in resolve_omena_query_sass_forward_sources(&sources) {
        let Some(uri) =
            resolve_lsp_style_uri_for_specifier(state, document, forward_source.as_str())
        else {
            continue;
        };
        let Some(target_document) = state.document(uri.as_str()) else {
            continue;
        };
        definitions.extend(sass_symbol_declarations_with_forwards(
            state,
            target_document,
            symbol_kind,
            candidate,
            visited,
        ));
    }
    definitions
}

fn sass_forward_module_target_uris(
    state: &LspShellState,
    document: &LspTextDocumentState,
    visited: &mut BTreeSet<String>,
) -> Vec<String> {
    if !visited.insert(document.uri.clone()) {
        return Vec::new();
    }
    let Some(sources) =
        summarize_omena_query_sass_module_sources(document.uri.as_str(), document.text.as_str())
    else {
        return Vec::new();
    };
    let mut uris = Vec::new();
    for forward_source in resolve_omena_query_sass_forward_sources(&sources) {
        if let Some(uri) =
            resolve_lsp_style_uri_for_specifier(state, document, forward_source.as_str())
        {
            uris.push(uri.clone());
        }
    }
    uris.sort();
    uris.dedup();
    uris
}

fn resolve_lsp_style_uri_for_specifier(
    state: &LspShellState,
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

fn sass_symbol_reference_matches(
    candidate: &LspStyleHoverCandidate,
    target: &LspStyleHoverCandidate,
) -> bool {
    is_sass_symbol_reference_kind(target.kind) && sass_symbol_target_matches(candidate, target)
}

fn sass_symbol_target_matches(
    candidate: &LspStyleHoverCandidate,
    target: &LspStyleHoverCandidate,
) -> bool {
    omena_query_sass_symbol_target_matches(
        candidate.kind,
        candidate.name.as_str(),
        candidate.namespace.as_deref(),
        target.kind,
        target.name.as_str(),
        target.namespace.as_deref(),
    )
}

fn render_sass_symbol_label(candidate: &LspStyleHoverCandidate) -> String {
    let namespace_prefix = candidate
        .namespace
        .as_deref()
        .map(|namespace| format!("{namespace}."))
        .unwrap_or_default();
    match sass_symbol_kind_from_candidate_kind(candidate.kind) {
        Some("variable") => format!("{namespace_prefix}${}", candidate.name),
        Some("mixin") if is_sass_symbol_declaration_kind(candidate.kind) => {
            format!("@mixin {}", candidate.name)
        }
        Some("mixin") => format!("@include {namespace_prefix}{}", candidate.name),
        Some("function") => format!("{namespace_prefix}{}()", candidate.name),
        _ => candidate.name.clone(),
    }
}

fn reference_lens_title(count: usize) -> String {
    if count == 1 {
        "1 reference".to_string()
    } else {
        format!("{count} references")
    }
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

    let Some((document_uri, candidate, candidates)) = style_candidates_for_params(state, params)
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

    let replacement = match candidate.kind {
        "customPropertyReference" | "customPropertyDeclaration" => new_name.to_string(),
        _ => return Value::Null,
    };
    let mut targets: Vec<&LspStyleHoverCandidate> = candidates
        .iter()
        .filter(|target| rename_target_matches(&candidate, target))
        .collect();
    targets.sort_by_key(|target| {
        (
            target.range.start.line,
            target.range.start.character,
            target.range.end.line,
            target.range.end.character,
        )
    });
    let edits: Vec<Value> = targets
        .into_iter()
        .map(|target| {
            json!({
                "range": target.range,
                "newText": replacement,
            })
        })
        .collect();
    if edits.is_empty() {
        return Value::Null;
    }

    let mut changes = serde_json::Map::new();
    changes.insert(document_uri, json!(edits));
    json!({
        "changes": Value::Object(changes),
    })
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

fn rename_target_matches(
    candidate: &LspStyleHoverCandidate,
    target: &LspStyleHoverCandidate,
) -> bool {
    match candidate.kind {
        "selector" => target.kind == "selector" && target.name == candidate.name,
        "customPropertyReference" | "customPropertyDeclaration" => {
            target.name == candidate.name && target.kind.starts_with("customProperty")
        }
        kind if is_sass_symbol_candidate_kind(kind) => {
            sass_symbol_target_matches(candidate, target)
        }
        _ => false,
    }
}

fn resolve_lsp_hover(state: &LspShellState, params: Option<&Value>) -> Value {
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
        return json!({
            "contents": {
                "kind": "markdown",
                "value": render_style_hover_candidate_markdown(
                    target_uri.as_str(),
                    target_text.as_str(),
                    &target,
                ),
            },
            "range": candidate.range,
        });
    }

    json!({
        "contents": {
            "kind": "markdown",
            "value": render_style_hover_candidate_markdown(
                document.uri.as_str(),
                document.text.as_str(),
                candidate,
            ),
        },
        "range": candidate.range,
    })
}

fn resolve_source_lsp_hover(
    state: &LspShellState,
    document: &LspTextDocumentState,
    params: Option<&Value>,
) -> Value {
    let Some(position) = lsp_position_from_params(params) else {
        return Value::Null;
    };
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

    json!({
        "contents": {
            "kind": "markdown",
            "value": value,
        },
        "range": candidate.range,
    })
}

fn resolve_source_lsp_definition(
    state: &LspShellState,
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
    let inferred_target_style_uri = context.target_style_uri.clone().or_else(|| {
        source_selector_candidate_at_position(state, document, position)
            .and_then(|candidate| candidate.target_style_uri)
    });
    let target_style_uri = inferred_target_style_uri
        .as_deref()
        .map(|uri| external_document_uri_for_query_uri(state, uri));

    let candidates = style_selector_definitions_from_open_documents(
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
    .map(|(uri, definition)| {
        let file_uri = target_style_uri
            .as_deref()
            .filter(|target_uri| file_uri_equivalent(target_uri, uri.as_str()))
            .map(ToString::to_string)
            .unwrap_or(uri);
        OmenaQueryCompletionCandidateV0 {
            file_uri,
            name: definition.name,
            kind: definition.kind,
            range: definition.range,
            source: definition.source,
        }
    })
    .collect::<Vec<_>>();
    let completion = summarize_omena_query_source_completion_at_position(
        document.uri.as_str(),
        position,
        candidates.as_slice(),
        target_style_uri.as_deref(),
        context.value_prefix.as_deref(),
        context.preferred_selector_names.as_slice(),
    );
    let items: Vec<Value> = completion
        .items
        .into_iter()
        .map(|item| lsp_completion_item_from_query(completion.file_kind, item))
        .collect();

    json!({
        "isIncomplete": false,
        "items": items,
    })
}

struct SourceCompletionContext {
    target_style_uri: Option<String>,
    value_prefix: Option<String>,
    preferred_selector_names: Vec<String>,
}

fn source_completion_context_at_position(
    state: &LspShellState,
    document: &LspTextDocumentState,
    position: ParserPositionV0,
) -> Option<SourceCompletionContext> {
    let offset = byte_offset_for_parser_position(document.text.as_str(), position)?;
    if let Some(target) = document
        .source_syntax_index
        .type_fact_targets
        .iter()
        .find(|target| offset >= target.byte_span.start && offset <= target.byte_span.end)
    {
        return Some(SourceCompletionContext {
            target_style_uri: target.target_style_uri.clone(),
            value_prefix: (!target.prefix.is_empty()).then(|| target.prefix.clone()),
            preferred_selector_names: source_completion_value_domain_selectors_for_target(
                document,
                target.byte_span,
                target.target_style_uri.as_deref(),
            ),
        });
    }
    if let Some(candidate) = source_selector_candidates_at_position(state, document, position)
        .into_iter()
        .find(|candidate| {
            candidate.kind == "sourceSelectorReference"
                || candidate.kind == "sourceSelectorPrefixReference"
        })
        && let Some(span) = byte_span_for_parser_range(document.text.as_str(), candidate.range)
    {
        return Some(SourceCompletionContext {
            target_style_uri: candidate.target_style_uri.clone(),
            value_prefix: source_completion_prefix_for_terminal_offset(
                document.text.as_str(),
                span,
                offset,
            ),
            preferred_selector_names: Vec::new(),
        });
    }
    if let Some(access) = document
        .source_syntax_index
        .style_property_accesses
        .iter()
        .find(|access| offset >= access.byte_span.start && offset <= access.byte_span.end)
    {
        let target_style_uri = access
            .target_style_uri
            .clone()
            .or_else(|| source_completion_target_uri_for_span(document, access.byte_span));
        return Some(SourceCompletionContext {
            target_style_uri,
            value_prefix: source_completion_prefix_for_terminal_offset(
                document.text.as_str(),
                access.byte_span,
                offset,
            ),
            preferred_selector_names: Vec::new(),
        });
    }
    if let Some(reference) = document
        .source_syntax_index
        .selector_references
        .iter()
        .find(|reference| offset >= reference.byte_span.start && offset <= reference.byte_span.end)
    {
        let target_style_uri = reference
            .target_style_uri
            .clone()
            .or_else(|| source_completion_target_uri_for_span(document, reference.byte_span));
        return Some(SourceCompletionContext {
            target_style_uri,
            value_prefix: source_completion_prefix_for_terminal_offset(
                document.text.as_str(),
                reference.byte_span,
                offset,
            ),
            preferred_selector_names: Vec::new(),
        });
    }
    if document
        .source_syntax_index
        .class_string_literals
        .iter()
        .any(|span| offset >= span.start && offset <= span.end)
    {
        let span = document
            .source_syntax_index
            .class_string_literals
            .iter()
            .find(|span| offset >= span.start && offset <= span.end)
            .copied()?;
        return Some(SourceCompletionContext {
            target_style_uri: None,
            value_prefix: source_completion_class_token_prefix_from_span(
                document.text.as_str(),
                span,
                offset,
            ),
            preferred_selector_names: Vec::new(),
        });
    }
    None
}

fn source_completion_target_uri_for_span(
    document: &LspTextDocumentState,
    span: ParserByteSpanV0,
) -> Option<String> {
    let range = parser_range_for_byte_span(document.text.as_str(), span);
    document
        .source_selector_candidates
        .iter()
        .find(|candidate| {
            candidate.range == range
                || parser_range_contains_position(&candidate.range, range.start)
                || parser_range_contains_position(&candidate.range, range.end)
        })
        .and_then(|candidate| candidate.target_style_uri.clone())
}

fn byte_span_for_parser_range(source: &str, range: ParserRangeV0) -> Option<ParserByteSpanV0> {
    Some(ParserByteSpanV0 {
        start: byte_offset_for_parser_position(source, range.start)?,
        end: byte_offset_for_parser_position(source, range.end)?,
    })
}

fn source_completion_value_domain_selectors_for_target(
    document: &LspTextDocumentState,
    byte_span: ParserByteSpanV0,
    target_style_uri: Option<&str>,
) -> Vec<String> {
    let mut selectors = document
        .source_syntax_index
        .selector_references
        .iter()
        .filter(|reference| reference.byte_span == byte_span)
        .filter(|reference| reference.match_kind == SourceSelectorReferenceMatchKind::Exact)
        .filter(|reference| {
            target_style_uri.is_none_or(|target_uri| {
                reference
                    .target_style_uri
                    .as_deref()
                    .is_some_and(|reference_uri| file_uri_equivalent(reference_uri, target_uri))
            })
        })
        .filter_map(|reference| reference.selector_name.clone())
        .collect::<Vec<_>>();
    selectors.sort();
    selectors.dedup();
    selectors
}

fn source_completion_prefix_for_terminal_offset(
    source: &str,
    span: ParserByteSpanV0,
    offset: usize,
) -> Option<String> {
    (offset >= span.end).then(|| source_completion_prefix_from_span(source, span, offset))?
}

fn source_completion_prefix_from_span(
    source: &str,
    span: ParserByteSpanV0,
    offset: usize,
) -> Option<String> {
    let end = offset.min(span.end);
    if end < span.start {
        return None;
    }
    let prefix = source.get(span.start..end)?;
    if prefix.is_empty() {
        return None;
    }
    if prefix.chars().all(is_css_identifier_continue) {
        Some(prefix.to_string())
    } else {
        None
    }
}

fn source_completion_class_token_prefix_from_span(
    source: &str,
    span: ParserByteSpanV0,
    offset: usize,
) -> Option<String> {
    let end = offset.min(span.end);
    if end < span.start {
        return None;
    }
    let prefix = source.get(span.start..end)?;
    let token = prefix
        .rsplit(|ch: char| ch.is_ascii_whitespace())
        .next()
        .unwrap_or_default();
    if token.is_empty() {
        return None;
    }
    if token.chars().all(is_css_identifier_continue) {
        Some(token.to_string())
    } else {
        None
    }
}

fn source_selector_candidate_for_params(
    state: &LspShellState,
    params: Option<&Value>,
) -> Option<(String, LspStyleHoverCandidate)> {
    let document_uri = document_uri_from_params(params);
    let position = lsp_position_from_params(params)?;
    let document = state.document(document_uri.as_str())?;
    if is_style_document_uri(document.uri.as_str()) {
        return None;
    }
    source_selector_candidate_at_position(state, document, position)
        .map(|candidate| (document_uri, candidate))
}

fn source_selector_candidate_at_position(
    state: &LspShellState,
    document: &LspTextDocumentState,
    position: ParserPositionV0,
) -> Option<LspStyleHoverCandidate> {
    source_selector_candidates_at_position(state, document, position)
        .into_iter()
        .next()
}

fn source_selector_candidates_at_position(
    state: &LspShellState,
    document: &LspTextDocumentState,
    position: ParserPositionV0,
) -> Vec<LspStyleHoverCandidate> {
    collect_source_selector_reference_candidates(state, document)
        .into_iter()
        .filter(|candidate| parser_range_contains_position(&candidate.range, position))
        .collect()
}

fn collect_source_selector_reference_candidates(
    state: &LspShellState,
    document: &LspTextDocumentState,
) -> Vec<LspStyleHoverCandidate> {
    resolve_source_provider_candidates(state, document).matched
}

fn resolve_source_provider_candidates(
    state: &LspShellState,
    document: &LspTextDocumentState,
) -> SourceProviderCandidateResolution {
    let source_candidates = collect_source_class_reference_candidates(document);
    let mut definitions = style_selector_definitions_from_open_documents(
        state,
        "",
        document.workspace_folder_uri.as_deref(),
    );
    for candidate in &source_candidates {
        if let Some(target_uri) = candidate.target_style_uri.as_deref()
            && !definitions
                .iter()
                .any(|(uri, _)| file_uri_equivalent(uri, target_uri))
        {
            definitions.extend(style_selector_definitions_from_uri(state, target_uri));
        }
    }
    let query_definitions = definitions
        .iter()
        .map(|(uri, definition)| query_style_selector_definition_for_matching(uri, definition))
        .collect::<Vec<_>>();
    let resolution = resolve_omena_query_source_provider_candidates(
        source_candidates
            .iter()
            .map(query_source_selector_candidate_for_matching)
            .collect(),
        query_definitions.as_slice(),
    );

    SourceProviderCandidateResolution {
        matched: resolution
            .matched
            .into_iter()
            .map(lsp_source_selector_candidate_from_query)
            .collect(),
        unresolved: resolution
            .unresolved
            .into_iter()
            .map(lsp_source_selector_candidate_from_query)
            .collect(),
    }
}

fn collect_source_class_reference_candidates(
    document: &LspTextDocumentState,
) -> Vec<LspStyleHoverCandidate> {
    document.source_selector_candidates.clone()
}

fn source_selector_candidates_from_index(
    document: &LspTextDocumentState,
    index: &SourceSyntaxIndex,
) -> Vec<LspStyleHoverCandidate> {
    let mut candidates: Vec<LspStyleHoverCandidate> = index
        .selector_references
        .iter()
        .map(|reference| source_reference_candidate(document, reference))
        .collect();
    candidates.sort();
    candidates.dedup();
    candidates
}

fn build_source_syntax_index(
    document: &LspTextDocumentState,
    resolution_inputs: &omena_query::OmenaQueryStyleResolutionInputsV0,
) -> SourceSyntaxIndex {
    if is_style_document_uri(document.uri.as_str()) {
        return SourceSyntaxIndex::default();
    }

    let imports = collect_source_imports(document, resolution_inputs);
    summarize_omena_query_source_syntax_index_for_source_language(
        document.uri.as_str(),
        document.text.as_str(),
        Some(document.language_id.as_str()),
        imports.imported_style_bindings,
        imports.classnames_bind_bindings,
    )
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SourceImportIndex {
    imported_style_bindings: Vec<ImportedStyleBinding>,
    classnames_bind_bindings: Vec<String>,
}

fn collect_source_imports(
    document: &LspTextDocumentState,
    resolution_inputs: &omena_query::OmenaQueryStyleResolutionInputsV0,
) -> SourceImportIndex {
    let source = document.text.as_str();
    let mut imports = SourceImportIndex {
        imported_style_bindings: Vec::new(),
        classnames_bind_bindings: Vec::new(),
    };
    let summary = summarize_omena_query_source_import_declarations_for_source_language(
        document.uri.as_str(),
        source,
        Some(document.language_id.as_str()),
    );
    for import in summary.imports {
        if import.specifier == "classnames/bind" {
            imports.classnames_bind_bindings.push(import.binding);
        } else if StyleLanguage::from_module_path(import.specifier.as_str()).is_some()
            && let Some(style_uri) =
                resolve_omena_query_style_uri_for_specifier_with_resolution_inputs(
                    document.uri.as_str(),
                    document.workspace_folder_uri.as_deref(),
                    import.specifier.as_str(),
                    resolution_inputs,
                )
        {
            imports.imported_style_bindings.push(ImportedStyleBinding {
                binding: import.binding,
                style_uri,
            });
        }
    }
    if is_vue_document(document) && has_vue_module_style_block(source) {
        for binding in collect_omena_query_vue_style_module_bindings(
            document.uri.as_str(),
            source,
            Some(document.language_id.as_str()),
        ) {
            imports.imported_style_bindings.push(ImportedStyleBinding {
                binding,
                style_uri: document.uri.clone(),
            });
        }
    }
    imports
        .imported_style_bindings
        .sort_by(|left, right| left.binding.cmp(&right.binding));
    imports
        .imported_style_bindings
        .dedup_by(|left, right| left.binding == right.binding && left.style_uri == right.style_uri);
    imports.classnames_bind_bindings.sort();
    imports.classnames_bind_bindings.dedup();
    imports
}

fn is_vue_document(document: &LspTextDocumentState) -> bool {
    document.language_id == "vue" || document.uri.ends_with(".vue")
}

fn document_has_style_index(document: &LspTextDocumentState) -> bool {
    is_style_document_uri(document.uri.as_str()) || document.style_summary.is_some()
}

fn has_vue_module_style_block(source: &str) -> bool {
    let lower = source.to_ascii_lowercase();
    let mut cursor = 0usize;
    while let Some(relative_start) = lower[cursor..].find("<style") {
        let tag_start = cursor + relative_start;
        let Some(relative_tag_end) = lower[tag_start..].find('>') else {
            break;
        };
        let tag = &lower[tag_start..tag_start + relative_tag_end + 1];
        if tag.contains("module") {
            return true;
        }
        cursor = tag_start + relative_tag_end + 1;
    }
    false
}

fn source_reference_candidate(
    document: &LspTextDocumentState,
    reference: &SourceSelectorReferenceFact,
) -> LspStyleHoverCandidate {
    let name = reference.selector_name.clone().unwrap_or_else(|| {
        document.text[reference.byte_span.start..reference.byte_span.end].to_string()
    });
    LspStyleHoverCandidate {
        kind: match reference.match_kind {
            SourceSelectorReferenceMatchKind::Exact => "sourceSelectorReference",
            SourceSelectorReferenceMatchKind::Prefix => "sourceSelectorPrefixReference",
        },
        name,
        range: parser_range_for_byte_span(document.text.as_str(), reference.byte_span),
        source: "omenaQuerySourceSyntaxIndex",
        target_style_uri: reference.target_style_uri.clone(),
        namespace: None,
    }
}

fn style_selector_definitions_from_open_documents(
    state: &LspShellState,
    selector_name: &str,
    workspace_folder_uri: Option<&str>,
) -> Vec<(String, LspStyleHoverCandidate)> {
    let mut definitions = Vec::new();
    for document in state.documents.values() {
        if !document_has_style_index(document)
            || !workspace_folder_compatible(workspace_folder_uri, document)
        {
            continue;
        }
        let Some((_, candidates)) = style_hover_candidates_for_document(document) else {
            continue;
        };
        definitions.extend(
            candidates
                .into_iter()
                .filter(|candidate| {
                    candidate.kind == "selector"
                        && (selector_name.is_empty() || candidate.name == selector_name)
                })
                .map(|candidate| (document.uri.clone(), candidate)),
        );
    }
    definitions.sort_by_key(|(uri, candidate)| {
        (
            uri.clone(),
            candidate.range.start.line,
            candidate.range.start.character,
        )
    });
    definitions
}

fn style_selector_definitions_from_uri(
    state: &LspShellState,
    uri: &str,
) -> Vec<(String, LspStyleHoverCandidate)> {
    style_hover_candidates_for_uri(state, uri)
        .map(|(_, candidates)| {
            candidates
                .into_iter()
                .filter(|candidate| candidate.kind == "selector")
                .map(|candidate| (uri.to_string(), candidate))
                .collect()
        })
        .unwrap_or_default()
}

fn style_selector_definitions_for_source_candidate(
    state: &LspShellState,
    candidate: &LspStyleHoverCandidate,
    workspace_folder_uri: Option<&str>,
) -> Vec<(String, LspStyleHoverCandidate)> {
    let mut definitions = style_selector_definitions_from_open_documents(
        state,
        source_candidate_definition_lookup_name(candidate),
        workspace_folder_uri,
    );
    if let Some(target_uri) = candidate.target_style_uri.as_deref()
        && !definitions
            .iter()
            .any(|(uri, _)| file_uri_equivalent(uri, target_uri))
    {
        definitions.extend(style_selector_definitions_from_uri(state, target_uri));
    }
    let query_definitions = definitions
        .iter()
        .map(|(uri, definition)| query_style_selector_definition_for_matching(uri, definition))
        .collect::<Vec<_>>();
    let matched_identities = resolve_omena_query_style_selector_definitions_for_source_candidate(
        &query_source_selector_candidate_for_matching(candidate),
        query_definitions.as_slice(),
    )
    .into_iter()
    .map(|definition| {
        query_definition_identity(
            definition.uri.as_str(),
            definition.name.as_str(),
            definition.range,
        )
    })
    .collect::<BTreeSet<_>>();

    definitions
        .into_iter()
        .filter(|(uri, definition)| {
            matched_identities.contains(&query_definition_identity(
                uri.as_str(),
                definition.name.as_str(),
                definition.range,
            ))
        })
        .collect()
}

fn style_selector_definitions_for_source_candidates(
    state: &LspShellState,
    candidates: &[LspStyleHoverCandidate],
    workspace_folder_uri: Option<&str>,
) -> Vec<(String, LspStyleHoverCandidate)> {
    let mut definitions = candidates
        .iter()
        .flat_map(|candidate| {
            style_selector_definitions_for_source_candidate(state, candidate, workspace_folder_uri)
        })
        .collect::<Vec<_>>();
    definitions.sort_by_key(|(uri, definition)| {
        (
            uri.clone(),
            definition.range.start.line,
            definition.range.start.character,
            definition.name.clone(),
        )
    });
    definitions.dedup_by(|left, right| {
        left.0 == right.0 && left.1.name == right.1.name && left.1.range == right.1.range
    });
    definitions
}

fn render_source_hover_definitions_markdown(
    state: &LspShellState,
    definitions: &[(String, LspStyleHoverCandidate)],
) -> Option<String> {
    let parts = definitions
        .iter()
        .filter_map(|(uri, definition)| {
            style_text_for_uri(state, uri).map(|text| {
                render_style_hover_candidate_markdown(uri.as_str(), text.as_str(), definition)
            })
        })
        .collect::<Vec<_>>();
    if parts.is_empty() {
        None
    } else {
        Some(parts.join("\n\n---\n\n"))
    }
}

fn source_candidate_definition_lookup_name(candidate: &LspStyleHoverCandidate) -> &str {
    if candidate.kind == "sourceSelectorPrefixReference" {
        ""
    } else {
        candidate.name.as_str()
    }
}

fn first_style_document_for_workspace<'a>(
    state: &'a LspShellState,
    workspace_folder_uri: Option<&str>,
) -> Option<(String, &'a LspTextDocumentState)> {
    state
        .documents
        .values()
        .filter(|document| document_has_style_index(document))
        .filter(|document| workspace_folder_compatible(workspace_folder_uri, document))
        .map(|document| (document.uri.clone(), document))
        .next()
}

fn resolve_selector_rename(
    state: &LspShellState,
    workspace_folder_uri: Option<&str>,
    target_style_uri: Option<&str>,
    selector_name: &str,
    new_name: &str,
) -> Value {
    let query_definitions =
        style_selector_definitions_from_open_documents(state, selector_name, workspace_folder_uri)
            .iter()
            .map(|(uri, definition)| query_style_selector_definition_for_matching(uri, definition))
            .collect::<Vec<_>>();
    let query_target_style_uri = query_target_style_uri_for_matching(target_style_uri);
    let mut query_references = Vec::new();
    for document in state.documents.values() {
        if is_style_document_uri(document.uri.as_str()) {
            continue;
        }
        if !workspace_folder_compatible(workspace_folder_uri, document) {
            continue;
        }
        query_references.extend(
            collect_source_selector_reference_candidates(state, document)
                .iter()
                .map(|candidate| {
                    query_source_selector_reference_edit_target_for_matching(document, candidate)
                }),
        );
    }
    let rename_plan = summarize_omena_query_rename_plan(
        selector_name,
        new_name,
        query_target_style_uri.as_deref(),
        query_definitions.as_slice(),
        query_references.as_slice(),
    );
    if rename_plan.edits.is_empty() {
        return Value::Null;
    }

    let mut changes: BTreeMap<String, Vec<Value>> = BTreeMap::new();
    for edit in rename_plan.edits {
        let edit_uri = external_document_uri_for_query_uri(state, edit.uri.as_str());
        changes.entry(edit_uri).or_default().push(json!({
            "range": edit.range,
            "newText": edit.new_text,
        }));
    }
    for edits in changes.values_mut() {
        edits.sort_by_key(|edit| {
            let line = edit
                .pointer("/range/start/line")
                .and_then(Value::as_u64)
                .unwrap_or_default();
            let character = edit
                .pointer("/range/start/character")
                .and_then(Value::as_u64)
                .unwrap_or_default();
            (line, character)
        });
    }

    let mut response_changes = serde_json::Map::new();
    for (uri, edits) in changes {
        response_changes.insert(uri, json!(edits));
    }
    json!({
        "changes": Value::Object(response_changes),
    })
}

fn external_document_uri_for_query_uri(state: &LspShellState, uri: &str) -> String {
    state
        .document(uri)
        .map(|document| document.uri.clone())
        .unwrap_or_else(|| uri.to_string())
}

fn render_style_hover_candidate_markdown(
    document_uri: &str,
    source: &str,
    candidate: &LspStyleHoverCandidate,
) -> String {
    let location = format!(
        "{}:{}",
        file_label_from_uri(document_uri),
        candidate.range.start.line + 1
    );
    let render_parts = summarize_omena_query_style_hover_render_parts(
        source,
        candidate.kind,
        candidate.name.as_str(),
        candidate.range.start,
    );
    match candidate.kind {
        "selector" => {
            format!(
                "**`.{}`** - _{}_\n\n```scss\n{}\n```",
                candidate.name, location, render_parts.snippet
            )
        }
        "customPropertyReference" => {
            format!(
                "**`var({})`** - _{}_\n\n```scss\n{}\n```",
                candidate.name, location, render_parts.snippet
            )
        }
        "customPropertyDeclaration" => {
            format!(
                "**`{}`** - _{}_\n\n```scss\n{}\n```",
                candidate.name, location, render_parts.snippet
            )
        }
        kind if is_sass_symbol_candidate_kind(kind) => {
            render_sass_symbol_hover_markdown(candidate, location.as_str(), &render_parts)
        }
        _ => candidate.name.clone(),
    }
}

fn render_sass_symbol_hover_markdown(
    candidate: &LspStyleHoverCandidate,
    location: &str,
    render_parts: &omena_query::OmenaQueryStyleHoverRenderPartsV0,
) -> String {
    let label = render_sass_symbol_label(candidate);
    match sass_symbol_kind_from_candidate_kind(candidate.kind) {
        Some("variable") if is_sass_symbol_declaration_kind(candidate.kind) => {
            if let Some(value) = render_parts.value.as_deref() {
                return format!(
                    "**`{}`** - _{}_\n\nValue: `{}`\n\n```scss\n{}\n```",
                    label, location, value, render_parts.snippet
                );
            }
            format!(
                "**`{}`** - _{}_\n\n```scss\n{}\n```",
                label, location, render_parts.snippet
            )
        }
        Some("mixin" | "function") if is_sass_symbol_declaration_kind(candidate.kind) => {
            let rendered_label = render_parts.signature.as_deref().unwrap_or(label.as_str());
            format!(
                "**`{}`** - _{}_\n\n```scss\n{}\n```",
                rendered_label, location, render_parts.snippet
            )
        }
        _ => {
            format!(
                "**`{}`** - _{}_\n\n```scss\n{}\n```",
                label, location, render_parts.snippet
            )
        }
    }
}

#[cfg(test)]
mod tests;
