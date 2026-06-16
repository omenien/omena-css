mod boundary;
mod diagnostics_scheduler;
mod disk_cache;
mod external_sif_loader;
mod frame_aware_refresh;
mod lsp_output;
mod message_loop;
#[cfg(feature = "parallel-style-diagnostics")]
mod parallel_style_wave;
mod protocol;
mod query_adapter;
mod query_reuse;
mod settings;
mod source_document_cache;
mod source_occurrence_cache;
mod source_type_fact_cache;
mod source_type_facts;
mod state;
mod streaming_ifds_diagnostics;
mod style_symbol_occurrence_cache;
mod workspace_index;
mod workspace_occurrence_cache;
mod workspace_runtime_registry;

pub use boundary::*;
use disk_cache::disk_diagnostics_cache_slot_for_resolve;
pub use external_sif_loader::{
    LspExternalSifRefreshJobV0, LspExternalSifRefreshResultV0,
    apply_deferred_external_sif_refresh_result, collect_deferred_external_sif_refresh,
    enable_deferred_external_sif_refresh, prepare_deferred_external_sif_refresh_job,
};
pub(crate) use external_sif_loader::{
    refresh_external_sifs_for_bridge_source_delta, refresh_external_sifs_for_state,
};
pub use frame_aware_refresh::*;
pub use lsp_output::*;
#[cfg(test)]
pub(crate) use message_loop::current_time_millis;
pub use message_loop::{
    LspLoopTurnV0, LspQueryDispatchV0, dispatched_query_internal_error_response,
    handle_lsp_message, handle_lsp_message_outputs, handle_lsp_message_scheduled_outputs,
    handle_lsp_message_scheduled_outputs_or_dispatch, resolve_dispatched_query_response,
    workspace_index_progress_end_output,
};
#[cfg(feature = "salsa-style-diagnostics")]
use omena_query::summarize_omena_query_source_baseline_diagnostics_for_workspace_file_with_source_syntax_index_and_definitions;
#[cfg(feature = "salsa-style-diagnostics")]
use omena_query::summarize_omena_query_target_unresolved_sass_import_diagnostics_for_workspace_paths;
use omena_query::{
    OmenaParserStyleDialect, OmenaQueryCodeActionV0, OmenaQueryCompletionCandidateV0,
    OmenaQueryCompletionItemV0, OmenaQueryEngineInputV2, OmenaQueryExternalSifInputV0,
    OmenaQuerySourceDiagnosticV0, OmenaQuerySourceDocumentInputV0,
    OmenaQuerySourceDomainClassReferenceFactV0 as SourceDomainClassReferenceFact,
    OmenaQuerySourceImportedStyleBindingV0 as ImportedStyleBinding,
    OmenaQuerySourceMissingSelectorDiagnosticCandidateV0,
    OmenaQuerySourceSelectorOccurrenceIndexV0, OmenaQuerySourceSelectorOccurrenceV0,
    OmenaQuerySourceSelectorReferenceFactV0 as SourceSelectorReferenceFact,
    OmenaQuerySourceSelectorReferenceMatchKindV0 as SourceSelectorReferenceMatchKind,
    OmenaQuerySourceSyntaxIndexV0 as SourceSyntaxIndex, OmenaQueryStyleDiagnosticV0,
    OmenaQueryStylePackageManifestV0, OmenaQueryStyleSelectorDefinitionV0,
    OmenaQueryStyleSourceInputV0, OmenaWorkspaceMonikerInput, OmenaWorkspaceOccurrenceFamilyV0,
    OmenaWorkspaceOccurrenceIndexV0, OmenaWorkspaceOccurrenceKindV0,
    OmenaWorkspaceOccurrenceRoleV0, OmenaWorkspaceOccurrenceSurfaceV0, OmenaWorkspaceOccurrenceV0,
    ParserByteSpanV0, ParserPositionV0, ParserRangeV0, StyleLanguage,
    collect_omena_query_vue_style_module_bindings,
    is_omena_query_sass_symbol_candidate_kind as is_sass_symbol_candidate_kind,
    is_omena_query_sass_symbol_declaration_kind as is_sass_symbol_declaration_kind,
    is_omena_query_sass_symbol_reference_kind as is_sass_symbol_reference_kind,
    load_omena_query_workspace_style_resolution_inputs, occurrences_for_monikers,
    omena_query_sass_symbol_kind_from_candidate_kind as sass_symbol_kind_from_candidate_kind,
    omena_workspace_moniker, read_omena_query_cascade_at_position_with_categorical_evidence,
    read_omena_query_style_context_index, resolve_omena_query_sass_forward_sources,
    resolve_omena_query_sass_module_use_sources_for_candidate,
    resolve_omena_query_sass_symbol_declarations,
    resolve_omena_query_source_candidate_selector_names,
    resolve_omena_query_source_provider_candidates,
    resolve_omena_query_style_selector_definitions_for_source_candidate,
    resolve_omena_query_style_uri_for_specifier_with_resolution_inputs,
    summarize_omena_query_omena_parser_style_facts,
    summarize_omena_query_refs_for_class_from_occurrence_index,
    summarize_omena_query_rename_plan_from_occurrence_index,
    summarize_omena_query_sass_module_sources, summarize_omena_query_source_completion_at_position,
    summarize_omena_query_source_diagnostics_for_file,
    summarize_omena_query_source_diagnostics_for_workspace_file_with_source_syntax_index_and_definitions,
    summarize_omena_query_source_import_declarations_for_source_language,
    summarize_omena_query_source_syntax_index_for_source_language,
    summarize_omena_query_style_completion_candidate_documentation,
    summarize_omena_query_style_completion_candidate_documentation_for_workspace_file_with_substrate,
    summarize_omena_query_style_completion_for_workspace_file_with_substrate,
    summarize_omena_query_style_diagnostics_for_file,
    summarize_omena_query_style_diagnostics_for_file_with_deep_analysis,
    summarize_omena_query_style_document,
    summarize_omena_query_style_hover_render_parts_for_hover_position,
    summarize_omena_query_style_hover_render_parts_for_workspace_file_hover_position_with_substrate,
    summarize_omena_query_style_refactor_code_actions,
    summarize_omena_query_workspace_occurrence_index_from_occurrences,
};
#[cfg(not(feature = "salsa-style-diagnostics"))]
use omena_query::{
    OmenaQueryExternalModuleModeV0,
    summarize_omena_query_style_diagnostics_for_workspace_file_with_external_mode_and_sifs_and_resolution_inputs,
};
use omena_sif::compute_omena_sif_leaf_hash_v1;
#[cfg(test)]
pub(crate) use omena_tsgo_client::{TsgoResolvedTypeV0, TsgoTypeFactResultEntryV0};
use protocol::*;
use query_adapter::*;
use query_reuse::{
    cascade_narrowing_substrate_for_style_sources, effective_style_package_manifests,
    refresh_document_reusable_indexes,
};
use serde_json::{Value, json};
pub(crate) use settings::{
    apply_diagnostic_settings, apply_feature_settings, apply_resolution_settings,
};
use source_occurrence_cache::store_source_selector_occurrence_sidecar;
#[cfg(test)]
pub(crate) use source_type_facts::apply_source_type_fact_results_to_document;
pub(crate) use source_type_facts::refresh_source_type_fact_candidates_for_document;
pub use state::*;
use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    fs,
    path::{Component, Path, PathBuf},
    sync::Arc,
};
use streaming_ifds_diagnostics::summarize_cross_file_streaming_reachability_diagnostics_for_lsp;
use style_symbol_occurrence_cache::store_style_symbol_occurrence_sidecar;
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
use workspace_occurrence_cache::{
    load_workspace_occurrence_shard, store_workspace_occurrence_shard,
    workspace_occurrence_dependency_digest,
};

pub const NODE_TEXT_DOCUMENT_SYNC_KIND: u8 = 2;
pub const DEBUG_STATE_REQUEST: &str = "omena/rustLspState";
pub const RUNTIME_LOOP_PROBE_REQUEST: &str = "omena/runtimeLoopProbe";
pub const STYLE_HOVER_CANDIDATES_REQUEST: &str = "omena/rustStyleHoverCandidates";
pub const STYLE_DIAGNOSTICS_REQUEST: &str = "omena/rustStyleDiagnostics";
pub const SOURCE_DIAGNOSTICS_REQUEST: &str = "omena/rustSourceDiagnostics";
pub const CASCADE_AT_POSITION_REQUEST: &str = "omena/rustCascadeAtPosition";
pub const STYLE_CONTEXT_INDEX_REQUEST: &str = "omena/rustStyleContextIndex";
pub const EXPLAIN_HOVER_TRACE_REQUEST: &str = "omena/explainHoverTrace";
const CANCEL_REQUEST_METHOD: &str = "$/cancelRequest";
const REQUEST_CANCELLED_ERROR_CODE: i32 = -32800;
// Cascade docs cost a whole-corpus narrowing analysis per completion item; only the
// top-ranked items an editor list actually shows get them, so completion latency
// stays independent of the workspace selector count.
const SOURCE_COMPLETION_DOCUMENTATION_BUDGET: usize = 12;

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

#[derive(Debug, Clone, PartialEq, Eq)]
struct SourceProviderCandidateResolution {
    matched: Vec<LspStyleHoverCandidate>,
    unresolved: Vec<LspStyleHoverCandidate>,
}

fn initialize_workspace_folders(state: &mut LspShellState, params: Option<&Value>) {
    state.workspace_runtime_registry.clear();
    state.client_supports_work_done_progress = params
        .and_then(|value| value.pointer("/capabilities/window/workDoneProgress"))
        .and_then(Value::as_bool)
        .unwrap_or(false);
    if let Some(folders) = params
        .and_then(|value| value.get("workspaceFolders"))
        .and_then(Value::as_array)
    {
        for folder in folders {
            insert_workspace_folder(state, folder);
        }
        refresh_workspace_resolution_inputs(state);
        refresh_external_sifs_for_state(state);
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
    refresh_external_sifs_for_state(state);
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
        let inputs = load_lsp_workspace_style_resolution_inputs(
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
    let inputs = load_lsp_workspace_style_resolution_inputs(
        Some(workspace_uri.as_str()),
        state.resolution.package_manifests.as_slice(),
    );
    state
        .resolution
        .workspace_style_resolution_inputs
        .insert(workspace_uri, inputs);
}

fn load_lsp_workspace_style_resolution_inputs(
    workspace_folder_uri: Option<&str>,
    configured_package_manifests: &[omena_query::OmenaQueryStylePackageManifestV0],
) -> omena_query::OmenaQueryStyleResolutionInputsV0 {
    let mut inputs = load_omena_query_workspace_style_resolution_inputs(
        workspace_folder_uri,
        configured_package_manifests,
    );
    inputs.external_sif_cache_fingerprint =
        workspace_folder_uri.and_then(external_sif_cache_fingerprint_for_workspace_uri);
    inputs
}

fn external_sif_cache_fingerprint_for_workspace_uri(workspace_folder_uri: &str) -> Option<String> {
    const METADATA_SCAN_LIMIT: usize = 2048;
    let root = file_uri_to_path(workspace_folder_uri)?;
    let root = normalize_path(root);
    let mut identities = Vec::new();
    for relative in [
        "omena.lock",
        "pnpm-lock.yaml",
        "package-lock.json",
        "yarn.lock",
        "bun.lock",
        "bun.lockb",
        "node_modules/.modules.yaml",
    ] {
        push_file_identity(&mut identities, root.join(relative).as_path());
    }
    collect_node_modules_package_link_identities(
        root.join("node_modules").as_path(),
        &mut identities,
        METADATA_SCAN_LIMIT,
    );
    if identities.is_empty() {
        return None;
    }
    let value = json!({
        "schemaVersion": "0",
        "product": "omena-lsp.external-sif-cache-freshness",
        "workspaceRoot": root.to_string_lossy(),
        "identities": identities,
    });
    let bytes = serde_json::to_vec(&value).ok()?;
    Some(
        compute_omena_sif_leaf_hash_v1(bytes.as_slice())
            .as_str()
            .to_string(),
    )
}

fn push_file_identity(output: &mut Vec<String>, path: &Path) {
    let Ok(metadata) = fs::symlink_metadata(path) else {
        return;
    };
    let modified = metadata
        .modified()
        .ok()
        .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|duration| format!("{}.{:09}", duration.as_secs(), duration.subsec_nanos()))
        .unwrap_or_else(|| "unknownMtime".to_string());
    let file_type = if metadata.file_type().is_symlink() {
        "symlink"
    } else if metadata.is_dir() {
        "dir"
    } else if metadata.is_file() {
        "file"
    } else {
        "other"
    };
    let target = if metadata.file_type().is_symlink() {
        fs::read_link(path)
            .ok()
            .map(|target| target.to_string_lossy().to_string())
            .unwrap_or_else(|| "unknownTarget".to_string())
    } else {
        String::new()
    };
    output.push(format!(
        "{}|{file_type}|len{}|mtime{modified}|target{target}",
        normalize_path(path.to_path_buf()).to_string_lossy(),
        metadata.len()
    ));
}

fn collect_node_modules_package_link_identities(
    node_modules: &Path,
    output: &mut Vec<String>,
    limit: usize,
) {
    let Ok(entries) = fs::read_dir(node_modules) else {
        return;
    };
    let mut seen = 0usize;
    for entry in entries.flatten() {
        if seen >= limit {
            break;
        }
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if name.starts_with('.') {
            continue;
        }
        if name.starts_with('@') && path.is_dir() {
            let Ok(scoped_entries) = fs::read_dir(path.as_path()) else {
                continue;
            };
            for scoped_entry in scoped_entries.flatten() {
                if seen >= limit {
                    break;
                }
                push_file_identity(output, scoped_entry.path().as_path());
                seen = seen.saturating_add(1);
            }
            continue;
        }
        push_file_identity(output, path.as_path());
        seen = seen.saturating_add(1);
    }
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
        .unwrap_or_else(|| {
            load_lsp_workspace_style_resolution_inputs(
                workspace_folder_uri,
                state.resolution.package_manifests.as_slice(),
            )
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
    let origin = lsp_document_origin_for_uri(uri.as_str());
    let mut document = LspTextDocumentState {
        uri,
        origin,
        workspace_folder_uri,
        language_id,
        version,
        text,
        text_hash: String::new(),
        style_summary: None,
        diagnostics_schedule_count: 0,
        optimizing_tier_feedback: None,
        style_candidates: Vec::new(),
        source_syntax_index: SourceSyntaxIndex::default(),
        has_unresolved_style_import: false,
        source_selector_candidates: Vec::new(),
    };
    refresh_document_reusable_indexes(&mut document, resolution_inputs);
    document
}

fn lsp_text_document_state_with_source_syntax_index(
    uri: String,
    workspace_folder_uri: Option<String>,
    language_id: String,
    version: i64,
    text: String,
    source_syntax_index: SourceSyntaxIndex,
    has_unresolved_style_import: bool,
) -> LspTextDocumentState {
    let origin = lsp_document_origin_for_uri(uri.as_str());
    let mut document = LspTextDocumentState {
        uri,
        origin,
        workspace_folder_uri,
        language_id,
        version,
        text,
        text_hash: String::new(),
        style_summary: None,
        diagnostics_schedule_count: 0,
        optimizing_tier_feedback: None,
        style_candidates: Vec::new(),
        source_syntax_index: SourceSyntaxIndex::default(),
        has_unresolved_style_import,
        source_selector_candidates: Vec::new(),
    };
    document.text_hash = compute_omena_sif_leaf_hash_v1(document.text.as_bytes())
        .as_str()
        .to_string();
    document.source_selector_candidates =
        source_selector_candidates_from_index(&document, &source_syntax_index);
    document.source_syntax_index = source_syntax_index;
    document
}

fn lsp_document_origin_for_uri(uri: &str) -> LspDocumentOrigin {
    if is_foreign_style_document_uri(uri) {
        LspDocumentOrigin::Foreign
    } else {
        LspDocumentOrigin::Local
    }
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
        refresh_style_external_inputs_for_document_event(state, uri, None);
        refresh_source_indexes_for_style_document_change(state, uri);
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
    let previous_external_inputs = if is_style_document_uri(uri) {
        style_external_dependency_snapshot(state, uri)
    } else {
        StyleExternalDependencySnapshot::default()
    };
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
            refresh_style_external_inputs_for_document_event(
                state,
                uri,
                Some(previous_external_inputs),
            );
            refresh_source_indexes_for_style_document_change(state, uri);
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
    let previous_external_inputs = if is_style_document_uri(uri) {
        style_external_dependency_snapshot(state, uri)
    } else {
        StyleExternalDependencySnapshot::default()
    };
    if is_style_document_uri(uri) && reload_indexed_style_document_from_disk(state, uri) {
        refresh_style_external_inputs_for_document_event(
            state,
            uri,
            Some(previous_external_inputs),
        );
        refresh_source_indexes_for_style_document_change(state, uri);
        return;
    }
    state.remove_document_uri(uri);
    if is_style_document_uri(uri) {
        refresh_style_external_inputs_after_document_removal(state, previous_external_inputs);
        refresh_source_indexes_for_style_document_change(state, uri);
    }
}

fn did_change_workspace_folders(
    state: &mut LspShellState,
    params: Option<&Value>,
    index_added_workspace_folders: bool,
) -> bool {
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
    refresh_external_sifs_for_state(state);
    if added_workspace_folder && index_added_workspace_folders {
        index_workspace_style_files(state);
    }
    added_workspace_folder
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
            return;
        }
        if state.has_open_document_uri(uri) {
            return;
        }
        if change_type == 3 {
            state.remove_document_uri(uri);
            return;
        }
        let _ = reload_indexed_source_document_from_disk(state, uri);
        return;
    }
    if state.has_open_document_uri(uri) {
        return;
    }
    if change_type == 3 {
        state.remove_document_uri(uri);
        refresh_source_indexes_for_style_document_change(state, uri);
        return;
    }

    if reload_indexed_style_document_from_disk(state, uri) {
        admit_foreign_style_dependencies_for_style_uri(state, uri);
        refresh_external_sifs_for_state(state);
        refresh_source_indexes_for_style_document_change(state, uri);
    }
}

fn refresh_source_indexes_for_resolution_config_change(
    state: &mut LspShellState,
    config_uri: &str,
) {
    refresh_workspace_resolution_inputs_for_uri(state, config_uri);
    refresh_external_sifs_for_state(state);
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
    refresh_external_sifs_for_state(state);
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
    if is_package_manager_install_state_path(path.as_path()) {
        return true;
    }
    let Some(file_name) = path.file_name().and_then(|value| value.to_str()) else {
        return false;
    };
    file_name == "package.json"
        || file_name == "omena.lock"
        || file_name == "pnpm-lock.yaml"
        || file_name == "package-lock.json"
        || file_name == "yarn.lock"
        || file_name == "bun.lock"
        || file_name == "bun.lockb"
        || file_name == ".modules.yaml"
        || file_name.ends_with(".sif.json")
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

fn is_package_manager_install_state_path(path: &Path) -> bool {
    // The package-ROOT path (the `node_modules/<scope>/<pkg>` symlink itself) is what a
    // pnpm install / symlink-retarget touches. `node_modules_package_for_path` normalizes
    // that root case to subpath = "." (never ""), so matching `is_empty()` alone was dead
    // code — the package-root watched event never fired an external-SIF refresh. Match the
    // normalized "." sentinel (and "" defensively).
    node_modules_package_for_path(path)
        .is_some_and(|(_, _, subpath)| subpath.is_empty() || subpath == ".")
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

fn reload_indexed_source_document_from_disk(state: &mut LspShellState, uri: &str) -> bool {
    let Some(path) = file_uri_to_path(uri) else {
        return false;
    };
    let Some(language_id) = workspace_index_language_id_for_uri(uri) else {
        return false;
    };
    if is_style_document_uri(uri) {
        return false;
    }
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
            language_id,
            0,
            text,
            &resolution_inputs,
        ),
    );
    true
}

const FOREIGN_STYLE_DEPENDENCY_ADMISSION_LIMIT: usize = 512;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct StyleExternalDependencySnapshot {
    bridge_sources: Vec<String>,
    foreign_dependency_uris: Vec<String>,
}

fn style_external_dependency_snapshot(
    state: &LspShellState,
    uri: &str,
) -> StyleExternalDependencySnapshot {
    let Some(document) = state.document(uri) else {
        return StyleExternalDependencySnapshot::default();
    };
    let Some(summary) = document.style_summary.as_ref() else {
        return StyleExternalDependencySnapshot::default();
    };

    let mut bridge_sources = BTreeSet::new();
    let mut foreign_dependency_uris = BTreeSet::new();
    for source in summary
        .sass_module_use_sources
        .iter()
        .map(String::as_str)
        .chain(
            summary
                .sass_module_forward_sources
                .iter()
                .map(String::as_str),
        )
    {
        if source.starts_with("file://") {
            bridge_sources.insert(source.to_string());
        }
        if let Some(uri) = resolve_lsp_style_uri_for_specifier(state, document, source)
            && is_foreign_style_document_uri(uri.as_str())
        {
            bridge_sources.insert(source.to_string());
            foreign_dependency_uris.insert(uri);
        }
    }

    StyleExternalDependencySnapshot {
        bridge_sources: bridge_sources.into_iter().collect(),
        foreign_dependency_uris: foreign_dependency_uris.into_iter().collect(),
    }
}

fn refresh_style_external_inputs_for_document_event(
    state: &mut LspShellState,
    uri: &str,
    previous: Option<StyleExternalDependencySnapshot>,
) {
    let previous = previous.unwrap_or_default();
    let next = style_external_dependency_snapshot(state, uri);
    if previous == next {
        return;
    }

    if !previous.foreign_dependency_uris.is_empty() || !next.foreign_dependency_uris.is_empty() {
        admit_foreign_style_dependencies_for_style_uri(state, uri);
    }

    if previous.bridge_sources != next.bridge_sources {
        refresh_external_sifs_for_bridge_source_delta(
            state,
            previous.bridge_sources.as_slice(),
            next.bridge_sources.as_slice(),
        );
    }
}

fn refresh_style_external_inputs_after_document_removal(
    state: &mut LspShellState,
    previous: StyleExternalDependencySnapshot,
) {
    if !previous.bridge_sources.is_empty() {
        refresh_external_sifs_for_bridge_source_delta(
            state,
            previous.bridge_sources.as_slice(),
            &[],
        );
    }
}

pub(crate) fn admit_foreign_style_dependencies_for_indexed_style_documents(
    state: &mut LspShellState,
) {
    let style_uris = state
        .documents
        .values()
        .filter(|document| is_style_document_uri(document.uri.as_str()))
        .map(|document| document.uri.clone())
        .collect::<Vec<_>>();
    admit_foreign_style_dependencies_for_style_uris(state, style_uris);
}

fn admit_foreign_style_dependencies_for_style_uri(state: &mut LspShellState, uri: &str) {
    admit_foreign_style_dependencies_for_style_uris(state, vec![uri.to_string()]);
}

fn admit_foreign_style_dependencies_for_style_uris(
    state: &mut LspShellState,
    style_uris: Vec<String>,
) {
    let mut queue = style_uris.into_iter().collect::<VecDeque<_>>();
    let mut visited = BTreeSet::new();
    let mut admitted = 0usize;
    while let Some(current_uri) = queue.pop_front() {
        if admitted >= FOREIGN_STYLE_DEPENDENCY_ADMISSION_LIMIT
            || !visited.insert(current_uri.clone())
        {
            continue;
        }
        let dependency_uris = state
            .document(current_uri.as_str())
            .map(|document| style_module_dependency_target_uris(state, document))
            .unwrap_or_default();
        for dependency_uri in dependency_uris {
            if admitted >= FOREIGN_STYLE_DEPENDENCY_ADMISSION_LIMIT {
                break;
            }
            if !is_foreign_style_document_uri(dependency_uri.as_str()) {
                continue;
            }
            if !state.contains_document_uri(dependency_uri.as_str())
                && reload_indexed_style_document_from_disk(state, dependency_uri.as_str())
            {
                admitted += 1;
            }
            if state
                .document(dependency_uri.as_str())
                .is_some_and(|document| document.origin == LspDocumentOrigin::Foreign)
            {
                queue.push_back(dependency_uri);
            }
        }
    }
}

fn style_module_dependency_target_uris(
    state: &LspShellState,
    document: &LspTextDocumentState,
) -> Vec<String> {
    let Some(sources) =
        summarize_omena_query_sass_module_sources(document.uri.as_str(), document.text.as_str())
    else {
        return Vec::new();
    };
    let mut uris = Vec::new();
    let module_sources = sources
        .module_use_edges
        .iter()
        .map(|edge| edge.source.as_str())
        .chain(sources.module_forward_sources.iter().map(String::as_str));
    for source in module_sources {
        if let Some(uri) = resolve_lsp_style_uri_for_specifier(state, document, source) {
            uris.push(uri);
        }
    }
    uris.sort();
    uris.dedup();
    uris
}

fn refresh_source_indexes_for_style_document_change(state: &mut LspShellState, style_uri: &str) {
    let workspace_folder_uri = state
        .document(style_uri)
        .and_then(|document| document.workspace_folder_uri.clone())
        .or_else(|| resolve_workspace_folder_uri(state, style_uri));
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
        let document = std::sync::Arc::make_mut(document);
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
    let workspace_diagnostics_summary = {
        let mut host_slot = state.style_memo_host.borrow_mut();
        let host = host_slot.get_or_insert_with(omena_query::OmenaQueryStyleMemoHostV0::new);
        host.workspace_style_diagnostics(
            document.uri.as_str(),
            style_sources.as_slice(),
            source_documents.as_slice(),
            state.resolution.package_manifests.as_slice(),
            external_sifs,
            &resolution_inputs,
        )
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
    let diagnostics = finish_style_diagnostics_value(
        &LspStyleDiagnosticsRenderInputsV0 {
            document_uri: document.uri.as_str(),
            document_text: document.text.as_str(),
            query_candidates: query_candidates.as_slice(),
            style_sources: style_sources.as_slice(),
            source_documents: source_documents.as_slice(),
            package_manifests: state.resolution.package_manifests.as_slice(),
            deep_analysis: state.diagnostics.deep_analysis,
            configured_severity: state.diagnostics.severity,
        },
        workspace_diagnostics_summary,
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
    pub(crate) style_sources: &'inputs [OmenaQueryStyleSourceInputV0],
    pub(crate) source_documents: &'inputs [OmenaQuerySourceDocumentInputV0],
    pub(crate) package_manifests: &'inputs [OmenaQueryStylePackageManifestV0],
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
            style_sources: self.style_sources.as_slice(),
            source_documents: self.source_documents.as_slice(),
            package_manifests: self.package_manifests.as_slice(),
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
            style_sources: style_paths.as_slice(),
            source_documents: &[],
            package_manifests: state.resolution.package_manifests.as_slice(),
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
            let workspace_summary = host.workspace_style_diagnostics(
                inputs.document_uri.as_str(),
                inputs.style_sources.as_slice(),
                inputs.source_documents.as_slice(),
                inputs.package_manifests.as_slice(),
                inputs.external_sifs.as_slice(),
                &inputs.resolution_inputs,
            );
            finish_style_diagnostics_value(&inputs.borrowed(), workspace_summary)
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

#[derive(Debug, Default)]
pub struct LspDiagnosticsFollowUpEffectsV0 {
    pub outputs: Vec<ScheduledLspOutput>,
    pub deferred_diagnostics: Vec<LspDeferredDiagnosticsDispatchV0>,
}

pub fn external_sif_refresh_follow_up_diagnostics_effects(
    state: &mut LspShellState,
) -> LspDiagnosticsFollowUpEffectsV0 {
    let uris = state
        .documents
        .values()
        .filter(|document| {
            document.origin == LspDocumentOrigin::Local
                && protocol::is_style_document_uri(document.uri.as_str())
        })
        .map(|document| document.uri.clone())
        .collect::<Vec<_>>();
    if uris.is_empty() {
        return LspDiagnosticsFollowUpEffectsV0::default();
    }
    let effects = diagnostics_scheduler::run_diagnostics_schedule_effects(
        state,
        diagnostics_scheduler::DiagnosticsScheduleEvent::WatchedFiles { uris },
    );
    LspDiagnosticsFollowUpEffectsV0 {
        outputs: effects.outputs,
        deferred_diagnostics: effects.deferred_diagnostics,
    }
}

/// RFC 0009 Pillar F (rfcs#68): the worker-safe tail of the style
/// diagnostics pipeline — per-file fallback summarize, streaming-IFDS
/// extend, opt-in deep analysis, severity mapping and LSP JSON rendering.
/// Pure of its arguments, so the serial resolve and the parallel wave share
/// ONE implementation and cannot drift byte-wise.
pub(crate) fn finish_style_diagnostics_value(
    inputs: &LspStyleDiagnosticsRenderInputsV0<'_>,
    workspace_diagnostics_summary: Option<omena_query::OmenaQueryStyleDiagnosticsForFileV0>,
) -> Value {
    let mut diagnostics_summary = workspace_diagnostics_summary.unwrap_or_else(|| {
        summarize_omena_query_style_diagnostics_for_file(
            inputs.document_uri,
            inputs.document_text,
            inputs.query_candidates,
        )
    });
    diagnostics_summary.diagnostics.extend(
        summarize_cross_file_streaming_reachability_diagnostics_for_lsp(
            inputs.document_uri,
            inputs.style_sources,
            inputs.source_documents,
            inputs.package_manifests,
        ),
    );
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

pub(crate) struct LspSourceDiagnosticsRenderInputsV0<'inputs> {
    pub(crate) document_uri: &'inputs str,
    pub(crate) document_text: &'inputs str,
    pub(crate) source_syntax_index: &'inputs SourceSyntaxIndex,
    pub(crate) source_selector_candidates: &'inputs [LspStyleHoverCandidate],
    pub(crate) style_sources: &'inputs [OmenaQueryStyleSourceInputV0],
    pub(crate) query_definitions: &'inputs [OmenaQueryStyleSelectorDefinitionV0],
    pub(crate) source_selector_fallback_candidates:
        &'inputs [OmenaQuerySourceMissingSelectorDiagnosticCandidateV0],
    pub(crate) configured_severity: u8,
}

impl LspOwnedSourceDiagnosticsRenderInputsV0 {
    fn borrowed(&self) -> LspSourceDiagnosticsRenderInputsV0<'_> {
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
            configured_severity: self.configured_severity,
        }
    }
}

fn resolve_source_diagnostics_for_uri(state: &LspShellState, document_uri: &str) -> Value {
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
    let dispatch = LspDeferredDiagnosticsDispatchV0 {
        uri: document_uri.to_string(),
        coalesce_key: String::new(),
        tier_plan,
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
    let source_selector_fallback_candidates = provider_candidates
        .into_iter()
        .filter_map(|candidate| {
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

    Some(LspOwnedSourceDiagnosticsRenderInputsV0 {
        document_uri: document.uri.clone(),
        document_text: document.text.clone(),
        source_syntax_index: document.source_syntax_index.clone(),
        source_selector_candidates,
        style_sources,
        query_definitions,
        source_selector_fallback_candidates,
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
) -> Vec<omena_query::OmenaQueryStyleSelectorDefinitionV0> {
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

    actions.extend(resolve_lsp_suppression_code_actions(
        state,
        params,
        diagnostics,
    ));

    if diagnostics.is_empty() {
        actions.extend(resolve_lsp_refactor_code_actions(state, params));
    }

    if actions.is_empty() {
        Value::Null
    } else {
        json!(actions)
    }
}

fn resolve_lsp_suppression_code_actions(
    state: &LspShellState,
    params: Option<&Value>,
    diagnostics: &[Value],
) -> Vec<Value> {
    let document_uri = document_uri_from_params(params);
    let Some(document) = state.document(document_uri.as_str()) else {
        return Vec::new();
    };
    if !is_style_document_uri(document.uri.as_str()) {
        return Vec::new();
    }

    let mut actions = Vec::new();
    for (index, diagnostic) in diagnostics.iter().enumerate() {
        let Some(code) = diagnostic.get("code").and_then(Value::as_str) else {
            continue;
        };
        let Some(line) = diagnostic
            .pointer("/range/start/line")
            .and_then(Value::as_u64)
            .and_then(|line| usize::try_from(line).ok())
        else {
            continue;
        };
        let character = diagnostic
            .pointer("/range/start/character")
            .and_then(Value::as_u64)
            .and_then(|character| usize::try_from(character).ok())
            .unwrap_or(0);

        let indent = source_line_indent(document.text.as_str(), line);
        let insert_range = json!({
            "start": {
                "line": line,
                "character": 0,
            },
            "end": {
                "line": line,
                "character": 0,
            },
        });
        let mut changes = serde_json::Map::new();
        changes.insert(
            document.uri.clone(),
            json!([
                {
                    "range": insert_range,
                    "newText": format!(
                        "{indent}/* omena-ignore-next-line {code} [reason: 'TODO'] */\n"
                    ),
                },
            ]),
        );

        actions.push(json!({
            "title": "Suppress this diagnostic on the next line",
            "kind": "quickfix",
            "diagnostics": [diagnostic],
            "edit": {
                "changes": Value::Object(changes),
            },
            "data": {
                "source": "omenaLspDiagnosticSuppressionCodeAction",
                "diagnosticIndex": index,
                "code": code,
            },
        }));

        if let Some(block_line) =
            enclosing_style_block_open_line(document.text.as_str(), line, character)
        {
            let block_indent = source_line_indent(document.text.as_str(), block_line);
            let block_insert_range = json!({
                "start": {
                    "line": block_line,
                    "character": 0,
                },
                "end": {
                    "line": block_line,
                    "character": 0,
                },
            });
            let mut block_changes = serde_json::Map::new();
            block_changes.insert(
                document.uri.clone(),
                json!([
                    {
                        "range": block_insert_range,
                        "newText": format!(
                            "{block_indent}/* omena-ignore {code} [reason: 'TODO'] */\n"
                        ),
                    },
                ]),
            );

            actions.push(json!({
                "title": "Suppress diagnostics in this block",
                "kind": "quickfix",
                "diagnostics": [diagnostic],
                "edit": {
                    "changes": Value::Object(block_changes),
                },
                "data": {
                    "source": "omenaLspDiagnosticSuppressionCodeAction",
                    "diagnosticIndex": index,
                    "code": code,
                    "scope": "block",
                },
            }));
        }
    }
    actions
}

fn source_line_indent(source: &str, line: usize) -> String {
    source
        .lines()
        .nth(line)
        .map(|text| {
            text.chars()
                .take_while(|character| character.is_whitespace())
                .collect()
        })
        .unwrap_or_default()
}

fn enclosing_style_block_open_line(source: &str, line: usize, character: usize) -> Option<usize> {
    let offset =
        protocol::byte_offset_for_parser_position(source, ParserPositionV0 { line, character })?;
    let prefix = source.get(..offset)?;
    let mut block_stack = Vec::new();
    let mut current_line = 0usize;
    let mut quote: Option<char> = None;
    let mut in_block_comment = false;
    let mut characters = prefix.chars().peekable();

    while let Some(character) = characters.next() {
        if character == '\n' {
            current_line += 1;
            continue;
        }
        if in_block_comment {
            if character == '*' && characters.peek() == Some(&'/') {
                characters.next();
                in_block_comment = false;
            }
            continue;
        }
        if let Some(quote_character) = quote {
            if character == '\\' {
                if characters.peek().is_some() {
                    characters.next();
                }
            } else if character == quote_character {
                quote = None;
            }
            continue;
        }
        if character == '/' && characters.peek() == Some(&'*') {
            characters.next();
            in_block_comment = true;
            continue;
        }
        match character {
            '"' | '\'' => quote = Some(character),
            '{' => block_stack.push(current_line),
            '}' => {
                block_stack.pop();
            }
            _ => {}
        }
    }

    block_stack.last().copied()
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
    let actions = summarize_omena_query_style_refactor_code_actions(
        document.uri.as_str(),
        style_sources.as_slice(),
        document.text.as_str(),
        range,
        &[],
    )
    .actions;
    render_omena_query_lsp_code_actions(actions)
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

#[cfg(feature = "salsa-style-diagnostics")]
fn style_path_inputs_from_open_documents(
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
            style_source: String::new(),
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
            style_source: String::new(),
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
            source_syntax_index: Some(document.source_syntax_index.clone()),
            has_unresolved_style_import: document.has_unresolved_style_import,
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
    let occurrence_index =
        source_selector_occurrence_index_from_open_documents(state, workspace_folder_uri);
    let query_target_style_uri = query_target_style_uri_for_matching(target_style_uri);
    summarize_omena_query_refs_for_class_from_occurrence_index(
        selector_name,
        query_target_style_uri.as_deref(),
        false,
        occurrence_index.definitions.as_slice(),
        &occurrence_index.source_selector_index,
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
    let occurrence_index =
        source_selector_occurrence_index_from_open_documents(state, workspace_folder_uri);
    let query_target_style_uri = query_target_style_uri_for_matching(target_style_uri);
    for occurrence in occurrence_index
        .workspace_index
        .by_moniker
        .values()
        .flat_map(|occurrences| occurrences.iter())
    {
        if occurrence.family != Some(OmenaWorkspaceOccurrenceFamilyV0::CssModuleSelector)
            || !workspace_occurrence_matches_target_style(
                occurrence,
                query_target_style_uri.as_deref(),
            )
        {
            continue;
        }
        locations_by_name
            .entry(occurrence.name.clone())
            .or_default()
            .push(json!({
                "uri": occurrence.uri,
                "range": occurrence.range,
            }));
    }
    for locations in locations_by_name.values_mut() {
        locations.sort_by_key(location_sort_key);
        locations
            .dedup_by(|left, right| location_identity_key(left) == location_identity_key(right));
    }
    locations_by_name
}

#[derive(Debug, Clone)]
struct WorkspaceOccurrenceIndexes {
    definitions: Vec<omena_query::OmenaQueryStyleSelectorDefinitionV0>,
    source_selector_index: Arc<OmenaQuerySourceSelectorOccurrenceIndexV0>,
    workspace_index: Arc<OmenaWorkspaceOccurrenceIndexV0>,
}

fn workspace_occurrence_indexes_from_documents(
    state: &LspShellState,
    workspace_folder_uri: Option<&str>,
) -> WorkspaceOccurrenceIndexes {
    let source_document_keys =
        source_selector_occurrence_document_keys(state, workspace_folder_uri);
    let style_document_keys = style_symbol_occurrence_document_keys(state, workspace_folder_uri);
    let memo_workspace_folder_uri = workspace_folder_uri.map(str::to_string);
    if let Some(memo) = state.workspace_occurrence_index_memo.borrow().as_ref()
        && memo.workspace_folder_uri == memo_workspace_folder_uri
        && memo.source_document_keys == source_document_keys
        && memo.style_document_keys == style_document_keys
    {
        return WorkspaceOccurrenceIndexes {
            definitions: memo.definitions.clone(),
            source_selector_index: Arc::clone(&memo.source_selector_index),
            workspace_index: Arc::clone(&memo.workspace_index),
        };
    }
    let definitions =
        style_selector_definitions_from_open_documents(state, "", workspace_folder_uri)
            .iter()
            .map(|(uri, definition)| query_style_selector_definition_for_matching(uri, definition))
            .collect::<Vec<_>>();
    let definitions_digest = workspace_occurrence_dependency_digest(&definitions);
    let mut workspace_occurrences = Vec::new();
    let mut source_occurrences = state
        .documents
        .values()
        .filter(|document| !is_style_document_uri(document.uri.as_str()))
        .filter(|document| workspace_folder_compatible(workspace_folder_uri, document))
        .flat_map(|document| {
            let document_occurrences = source_selector_workspace_occurrences_for_document(
                state,
                document,
                definitions.as_slice(),
                definitions_digest.as_deref(),
            );
            workspace_occurrences.extend(document_occurrences.clone());
            document_occurrences
                .into_iter()
                .filter_map(source_selector_occurrence_from_workspace_occurrence_for_lsp)
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    source_occurrences.sort();
    source_occurrences.dedup();
    let style_dependency_digest = workspace_occurrence_dependency_digest(&(
        style_document_keys.as_slice(),
        &state.resolution.external_sifs,
    ));
    for document in state
        .documents
        .values()
        .filter(|document| document_has_style_index(document))
        .filter(|document| workspace_folder_compatible(workspace_folder_uri, document))
    {
        workspace_occurrences.extend(style_symbol_workspace_occurrences_for_document(
            state,
            document,
            workspace_folder_uri,
            style_dependency_digest.as_deref(),
        ));
    }
    workspace_occurrences.sort();
    workspace_occurrences.dedup();
    let workspace_index = Arc::new(
        summarize_omena_query_workspace_occurrence_index_from_occurrences(
            workspace_occurrences.as_slice(),
            vec![
                "workspaceOccurrenceIndex",
                "sourceSelectorOccurrenceIndex",
                "workspaceWideSelectorReferences",
                "workspaceWideSelectorRename",
                "styleSymbolReferences",
                "styleSymbolRename",
                "workspaceOccurrencePerFileShard",
            ],
        ),
    );
    let moniker_count = source_occurrences
        .iter()
        .map(|occurrence| occurrence.moniker.as_str())
        .collect::<BTreeSet<_>>()
        .len();
    let index = OmenaQuerySourceSelectorOccurrenceIndexV0 {
        schema_version: "0",
        product: "omena-query.source-selector-occurrence-index",
        moniker_count,
        occurrence_count: source_occurrences.len(),
        workspace_index: workspace_index.as_ref().clone(),
        occurrences: source_occurrences,
        ready_surfaces: vec![
            "sourceSelectorOccurrenceIndex",
            "workspaceWideSelectorReferences",
            "workspaceWideSelectorRename",
            "workspaceOccurrencePerFileShard",
        ],
    };
    let index = Arc::new(index);
    store_source_selector_occurrence_sidecar(
        state,
        workspace_folder_uri,
        source_document_keys.as_slice(),
        definitions.as_slice(),
        &index,
    );
    let style_occurrences = workspace_index
        .by_moniker
        .values()
        .flat_map(|occurrences| occurrences.iter())
        .filter_map(style_symbol_occurrence_from_workspace_occurrence_for_lsp)
        .collect::<Vec<_>>();
    store_style_symbol_occurrence_sidecar(
        state,
        workspace_folder_uri,
        style_document_keys.as_slice(),
        style_occurrences.as_slice(),
    );
    *state.workspace_occurrence_index_memo.borrow_mut() = Some(LspWorkspaceOccurrenceIndexMemo {
        workspace_folder_uri: memo_workspace_folder_uri,
        source_document_keys,
        style_document_keys,
        definitions: definitions.clone(),
        source_selector_index: Arc::clone(&index),
        workspace_index: Arc::clone(&workspace_index),
    });
    WorkspaceOccurrenceIndexes {
        definitions,
        source_selector_index: index,
        workspace_index,
    }
}

fn source_selector_occurrence_index_from_open_documents(
    state: &LspShellState,
    workspace_folder_uri: Option<&str>,
) -> WorkspaceOccurrenceIndexes {
    workspace_occurrence_indexes_from_documents(state, workspace_folder_uri)
}

fn source_selector_occurrence_document_keys(
    state: &LspShellState,
    workspace_folder_uri: Option<&str>,
) -> Vec<LspSourceSelectorOccurrenceDocumentKey> {
    state
        .documents
        .values()
        .filter(|document| workspace_folder_compatible(workspace_folder_uri, document))
        .map(|document| LspSourceSelectorOccurrenceDocumentKey {
            uri: document.uri.clone(),
            workspace_folder_uri: document.workspace_folder_uri.clone(),
            language_id: document.language_id.clone(),
            version: document.version,
            text_hash: document.text_hash.clone(),
        })
        .collect()
}

fn source_selector_workspace_occurrences_for_document(
    state: &LspShellState,
    document: &LspTextDocumentState,
    definitions: &[OmenaQueryStyleSelectorDefinitionV0],
    dependency_digest: Option<&str>,
) -> Vec<OmenaWorkspaceOccurrenceV0> {
    let resolution_inputs =
        resolution_inputs_for_workspace_uri(state, document.workspace_folder_uri.as_deref());
    if let Some(shard) = load_workspace_occurrence_shard(
        document.workspace_folder_uri.as_deref(),
        document.uri.as_str(),
        document.language_id.as_str(),
        document.text_hash.as_str(),
        dependency_digest,
        &resolution_inputs,
    ) {
        return shard.occurrences;
    }

    let mut occurrences = Vec::new();
    for candidate in collect_source_selector_reference_candidates(state, document) {
        let reference =
            query_source_selector_reference_candidate_for_matching(document, &candidate);
        let reference_candidate = omena_query::OmenaQuerySourceSelectorCandidateV0 {
            kind: reference.kind,
            name: reference.name.clone(),
            range: reference.range,
            source: reference.source,
            target_style_uri: reference.target_style_uri.clone(),
        };
        for selector_name in resolve_omena_query_source_candidate_selector_names(
            &reference_candidate,
            definitions,
            reference.target_style_uri.as_deref(),
        ) {
            let source_occurrence = OmenaQuerySourceSelectorOccurrenceV0 {
                moniker: omena_workspace_moniker(OmenaWorkspaceMonikerInput::CssModuleSelector {
                    target_style_uri: reference.target_style_uri.as_deref(),
                    selector_name: selector_name.as_str(),
                }),
                uri: reference.uri.clone(),
                selector_name: selector_name.clone(),
                range: reference.range,
                kind: workspace_occurrence_kind_from_source_reference_kind_for_lsp(reference.kind),
                role: OmenaWorkspaceOccurrenceRoleV0::Reference,
                source: OmenaWorkspaceOccurrenceSurfaceV0::OmenaQuerySourceSyntaxIndex,
                target_style_uri: reference.target_style_uri.clone(),
                rename_target: reference.kind == "sourceSelectorReference"
                    && reference.name == selector_name,
            };
            occurrences.push(
                workspace_occurrence_from_source_selector_occurrence_for_lsp(&source_occurrence),
            );
        }
    }
    occurrences.sort();
    occurrences.dedup();
    store_workspace_occurrence_shard(
        document.workspace_folder_uri.as_deref(),
        document.uri.as_str(),
        document.language_id.as_str(),
        document.text_hash.as_str(),
        dependency_digest,
        &resolution_inputs,
        occurrences.as_slice(),
    );
    occurrences
}

fn workspace_occurrence_from_source_selector_occurrence_for_lsp(
    occurrence: &OmenaQuerySourceSelectorOccurrenceV0,
) -> OmenaWorkspaceOccurrenceV0 {
    OmenaWorkspaceOccurrenceV0 {
        moniker: occurrence.moniker.clone(),
        uri: occurrence.uri.clone(),
        name: occurrence.selector_name.clone(),
        range: occurrence.range,
        kind: occurrence.kind,
        role: occurrence.role,
        surface: occurrence.source,
        family: Some(OmenaWorkspaceOccurrenceFamilyV0::CssModuleSelector),
        namespace: None,
        target_style_uri: occurrence.target_style_uri.clone(),
        rename_target: occurrence.rename_target,
    }
}

fn source_selector_occurrence_from_workspace_occurrence_for_lsp(
    occurrence: OmenaWorkspaceOccurrenceV0,
) -> Option<OmenaQuerySourceSelectorOccurrenceV0> {
    (occurrence.family == Some(OmenaWorkspaceOccurrenceFamilyV0::CssModuleSelector)).then_some(
        OmenaQuerySourceSelectorOccurrenceV0 {
            moniker: occurrence.moniker,
            uri: occurrence.uri,
            selector_name: occurrence.name,
            range: occurrence.range,
            kind: occurrence.kind,
            role: occurrence.role,
            source: occurrence.surface,
            target_style_uri: occurrence.target_style_uri,
            rename_target: occurrence.rename_target,
        },
    )
}

fn workspace_occurrence_kind_from_source_reference_kind_for_lsp(
    kind: &str,
) -> OmenaWorkspaceOccurrenceKindV0 {
    match kind {
        "sourceSelectorPrefixReference" => {
            OmenaWorkspaceOccurrenceKindV0::SourceSelectorPrefixReference
        }
        _ => OmenaWorkspaceOccurrenceKindV0::SourceSelectorReference,
    }
}

fn workspace_occurrence_matches_target_style(
    occurrence: &OmenaWorkspaceOccurrenceV0,
    target_style_uri: Option<&str>,
) -> bool {
    target_style_uri.is_none_or(|target_uri| {
        occurrence
            .target_style_uri
            .as_deref()
            .is_none_or(|candidate_target_uri| {
                file_uri_equivalent(candidate_target_uri, target_uri)
            })
    })
}

fn style_symbol_definition_locations_from_documents(
    state: &LspShellState,
    document: &LspTextDocumentState,
    candidate: &LspStyleHoverCandidate,
) -> Vec<Value> {
    let monikers = style_symbol_monikers_for_candidate(state, document, candidate);
    let occurrence_index = style_symbol_occurrence_index_from_documents(
        state,
        document.workspace_folder_uri.as_deref(),
    );
    let mut locations = occurrences_for_monikers(&occurrence_index, &monikers)
        .into_iter()
        .filter(|occurrence| occurrence.role == OmenaWorkspaceOccurrenceRoleV0::Definition)
        .map(|occurrence| {
            json!({
                "uri": occurrence.uri,
                "range": occurrence.range,
            })
        })
        .collect::<Vec<_>>();
    locations.sort_by_key(location_sort_key);
    locations.dedup_by(|left, right| location_identity_key(left) == location_identity_key(right));
    locations
}

fn style_symbol_reference_locations_from_documents(
    state: &LspShellState,
    document: &LspTextDocumentState,
    candidate: &LspStyleHoverCandidate,
    include_declaration: bool,
) -> Vec<Value> {
    let monikers = style_symbol_monikers_for_candidate(state, document, candidate);
    let occurrence_index = style_symbol_occurrence_index_from_documents(
        state,
        document.workspace_folder_uri.as_deref(),
    );
    let mut locations = occurrences_for_monikers(&occurrence_index, &monikers)
        .into_iter()
        .filter(|occurrence| {
            occurrence.role == OmenaWorkspaceOccurrenceRoleV0::Reference
                || (include_declaration
                    && occurrence.role == OmenaWorkspaceOccurrenceRoleV0::Definition)
        })
        .map(|occurrence| {
            json!({
                "uri": occurrence.uri,
                "range": occurrence.range,
            })
        })
        .collect::<Vec<_>>();
    if include_declaration && is_sass_symbol_candidate_kind(candidate.kind) {
        locations.extend(
            sass_symbol_definitions_for_candidate(state, document, candidate)
                .into_iter()
                .map(|(uri, definition)| {
                    json!({
                        "uri": uri,
                        "range": definition.range,
                    })
                }),
        );
    }
    locations.sort_by_key(location_sort_key);
    locations.dedup_by(|left, right| location_identity_key(left) == location_identity_key(right));
    locations
}

fn resolve_style_symbol_rename(
    state: &LspShellState,
    document: &LspTextDocumentState,
    candidate: &LspStyleHoverCandidate,
    new_name: &str,
) -> Value {
    let monikers = style_symbol_monikers_for_candidate(state, document, candidate);
    let occurrence_index = style_symbol_occurrence_index_from_documents(
        state,
        document.workspace_folder_uri.as_deref(),
    );
    let mut seen = BTreeSet::new();
    let mut changes: BTreeMap<String, Vec<Value>> = BTreeMap::new();
    for occurrence in occurrences_for_monikers(&occurrence_index, &monikers) {
        if !occurrence.rename_target {
            continue;
        }
        let key = (
            occurrence.uri.clone(),
            occurrence.range.start.line,
            occurrence.range.start.character,
            occurrence.range.end.line,
            occurrence.range.end.character,
        );
        if !seen.insert(key) {
            continue;
        }
        let edit_uri = external_document_uri_for_query_uri(state, occurrence.uri.as_str());
        changes.entry(edit_uri).or_default().push(json!({
            "range": occurrence.range,
            "newText": new_name,
        }));
    }

    if changes.is_empty() {
        return Value::Null;
    }
    for edits in changes.values_mut() {
        edits.sort_by_key(lsp_range_start_sort_key);
    }
    json!({
        "changes": Value::Object(changes.into_iter().map(|(uri, edits)| (uri, json!(edits))).collect()),
    })
}

fn style_symbol_occurrence_index_from_documents(
    state: &LspShellState,
    workspace_folder_uri: Option<&str>,
) -> Arc<OmenaWorkspaceOccurrenceIndexV0> {
    workspace_occurrence_indexes_from_documents(state, workspace_folder_uri).workspace_index
}

fn style_symbol_workspace_occurrences_for_document(
    state: &LspShellState,
    document: &LspTextDocumentState,
    workspace_folder_uri: Option<&str>,
    dependency_digest: Option<&str>,
) -> Vec<OmenaWorkspaceOccurrenceV0> {
    let resolution_inputs =
        resolution_inputs_for_workspace_uri(state, document.workspace_folder_uri.as_deref());
    if let Some(shard) = load_workspace_occurrence_shard(
        document.workspace_folder_uri.as_deref(),
        document.uri.as_str(),
        document.language_id.as_str(),
        document.text_hash.as_str(),
        dependency_digest,
        &resolution_inputs,
    ) {
        return shard.occurrences;
    }

    let mut style_occurrences = Vec::new();
    let Some((_, candidates)) = style_hover_candidates_for_document(document) else {
        return Vec::new();
    };
    for candidate in candidates {
        if candidate.kind.starts_with("customProperty") {
            style_occurrences.push(style_symbol_occurrence_for_candidate(
                style_custom_property_moniker(workspace_folder_uri, candidate.name.as_str()),
                document.uri.as_str(),
                &candidate,
                "customProperty",
                style_symbol_role_for_candidate(&candidate),
            ));
            continue;
        }
        if !is_sass_symbol_candidate_kind(candidate.kind) {
            continue;
        }
        if is_sass_symbol_declaration_kind(candidate.kind) {
            style_occurrences.push(style_symbol_occurrence_for_candidate(
                style_sass_symbol_moniker_for_document(state, document, &candidate),
                document.uri.as_str(),
                &candidate,
                sass_symbol_kind_from_candidate_kind(candidate.kind).unwrap_or("symbol"),
                "definition",
            ));
            continue;
        }
        let definitions = sass_symbol_definitions_for_candidate(state, document, &candidate);
        if definitions.is_empty() {
            let moniker = if let Some(target) =
                external_sif_sass_symbol_target_for_candidate(state, document, &candidate)
            {
                style_external_sif_sass_symbol_moniker(&target)
            } else {
                style_unresolved_sass_symbol_moniker(workspace_folder_uri, &candidate)
            };
            style_occurrences.push(style_symbol_occurrence_for_candidate(
                moniker,
                document.uri.as_str(),
                &candidate,
                sass_symbol_kind_from_candidate_kind(candidate.kind).unwrap_or("symbol"),
                "reference",
            ));
            continue;
        }
        for (definition_uri, definition) in definitions {
            style_occurrences.push(style_symbol_occurrence_for_candidate(
                style_sass_symbol_moniker_for_uri(state, definition_uri.as_str(), &definition),
                document.uri.as_str(),
                &candidate,
                sass_symbol_kind_from_candidate_kind(candidate.kind).unwrap_or("symbol"),
                "reference",
            ));
        }
    }
    style_occurrences.sort();
    style_occurrences.dedup();
    let workspace_occurrences = style_occurrences
        .iter()
        .map(|occurrence| workspace_occurrence_from_style_symbol_occurrence(document, occurrence))
        .collect::<Vec<_>>();
    store_workspace_occurrence_shard(
        document.workspace_folder_uri.as_deref(),
        document.uri.as_str(),
        document.language_id.as_str(),
        document.text_hash.as_str(),
        dependency_digest,
        &resolution_inputs,
        workspace_occurrences.as_slice(),
    );
    workspace_occurrences
}

fn style_symbol_occurrence_from_workspace_occurrence_for_lsp(
    occurrence: &OmenaWorkspaceOccurrenceV0,
) -> Option<LspStyleSymbolOccurrenceV0> {
    let family = occurrence.family?;
    if family == OmenaWorkspaceOccurrenceFamilyV0::CssModuleSelector {
        return None;
    }
    Some(LspStyleSymbolOccurrenceV0 {
        moniker: occurrence.moniker.clone(),
        uri: occurrence.uri.clone(),
        kind: occurrence.kind,
        family,
        name: occurrence.name.clone(),
        range: occurrence.range,
        role: occurrence.role,
        namespace: occurrence.namespace.clone(),
    })
}

fn workspace_occurrence_from_style_symbol_occurrence(
    document: &LspTextDocumentState,
    occurrence: &LspStyleSymbolOccurrenceV0,
) -> OmenaWorkspaceOccurrenceV0 {
    OmenaWorkspaceOccurrenceV0 {
        moniker: occurrence.moniker.clone(),
        uri: occurrence.uri.clone(),
        name: occurrence.name.clone(),
        range: occurrence.range,
        kind: occurrence.kind,
        role: occurrence.role,
        surface: OmenaWorkspaceOccurrenceSurfaceV0::OmenaLspStyleIndex,
        family: Some(occurrence.family),
        namespace: occurrence.namespace.clone(),
        target_style_uri: None,
        rename_target: document.origin == LspDocumentOrigin::Local,
    }
}

fn style_symbol_occurrence_document_keys(
    state: &LspShellState,
    workspace_folder_uri: Option<&str>,
) -> Vec<LspSourceSelectorOccurrenceDocumentKey> {
    state
        .documents
        .values()
        .filter(|document| document_has_style_index(document))
        .filter(|document| workspace_folder_compatible(workspace_folder_uri, document))
        .map(|document| LspSourceSelectorOccurrenceDocumentKey {
            uri: document.uri.clone(),
            workspace_folder_uri: document.workspace_folder_uri.clone(),
            language_id: document.language_id.clone(),
            version: document.version,
            text_hash: document.text_hash.clone(),
        })
        .collect()
}

fn style_symbol_monikers_for_candidate(
    state: &LspShellState,
    document: &LspTextDocumentState,
    candidate: &LspStyleHoverCandidate,
) -> BTreeSet<String> {
    if candidate.kind.starts_with("customProperty") {
        return BTreeSet::from([style_custom_property_moniker(
            document.workspace_folder_uri.as_deref(),
            candidate.name.as_str(),
        )]);
    }
    if is_sass_symbol_declaration_kind(candidate.kind) {
        return BTreeSet::from([style_sass_symbol_moniker_for_document(
            state, document, candidate,
        )]);
    }
    let definitions = sass_symbol_definitions_for_candidate(state, document, candidate);
    if definitions.is_empty() {
        if let Some(target) =
            external_sif_sass_symbol_target_for_candidate(state, document, candidate)
        {
            return BTreeSet::from([style_external_sif_sass_symbol_moniker(&target)]);
        }
        return BTreeSet::from([style_unresolved_sass_symbol_moniker(
            document.workspace_folder_uri.as_deref(),
            candidate,
        )]);
    }
    definitions
        .into_iter()
        .map(|(uri, definition)| {
            style_sass_symbol_moniker_for_uri(state, uri.as_str(), &definition)
        })
        .collect()
}

fn style_symbol_occurrence_for_candidate(
    moniker: String,
    uri: &str,
    candidate: &LspStyleHoverCandidate,
    family: &'static str,
    role: &'static str,
) -> LspStyleSymbolOccurrenceV0 {
    LspStyleSymbolOccurrenceV0 {
        moniker,
        uri: uri.to_string(),
        kind: workspace_occurrence_kind_from_style_symbol_kind(candidate.kind)
            .unwrap_or(OmenaWorkspaceOccurrenceKindV0::CustomPropertyReference),
        family: workspace_occurrence_family_from_style_symbol_family(family)
            .unwrap_or(OmenaWorkspaceOccurrenceFamilyV0::Symbol),
        name: candidate.name.clone(),
        range: candidate.range,
        role: workspace_occurrence_role_from_style_symbol_role(role),
        namespace: candidate.namespace.clone(),
    }
}

fn style_symbol_role_for_candidate(candidate: &LspStyleHoverCandidate) -> &'static str {
    if candidate.kind.ends_with("Declaration") {
        "definition"
    } else {
        "reference"
    }
}

fn style_custom_property_moniker(workspace_folder_uri: Option<&str>, name: &str) -> String {
    omena_workspace_moniker(OmenaWorkspaceMonikerInput::CssCustomProperty {
        workspace_folder_uri,
        name,
    })
}

fn style_sass_symbol_moniker(uri: &str, candidate: &LspStyleHoverCandidate) -> String {
    let family = sass_symbol_kind_from_candidate_kind(candidate.kind).unwrap_or("symbol");
    omena_workspace_moniker(OmenaWorkspaceMonikerInput::SassSymbol {
        definition_uri: uri,
        family,
        name: candidate.name.as_str(),
    })
}

fn style_sass_symbol_moniker_for_document(
    state: &LspShellState,
    document: &LspTextDocumentState,
    candidate: &LspStyleHoverCandidate,
) -> String {
    style_sass_symbol_moniker_for_uri(state, document.uri.as_str(), candidate)
}

fn style_sass_symbol_moniker_for_uri(
    state: &LspShellState,
    uri: &str,
    candidate: &LspStyleHoverCandidate,
) -> String {
    if let Some(moniker) = style_foreign_sass_symbol_moniker(state, uri, candidate) {
        return moniker;
    }
    style_sass_symbol_moniker(uri, candidate)
}

fn style_foreign_sass_symbol_moniker(
    state: &LspShellState,
    uri: &str,
    candidate: &LspStyleHoverCandidate,
) -> Option<String> {
    let identity = foreign_sass_package_identity_for_uri(state, uri)?;
    let family = sass_symbol_kind_from_candidate_kind(candidate.kind).unwrap_or("symbol");
    Some(format!(
        "sass-symbol-foreign:pkg:{}@{}/{}#{}:{}",
        identity.package_name, identity.version, identity.subpath, family, candidate.name
    ))
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ForeignSassPackageIdentity {
    package_name: String,
    version: String,
    subpath: String,
}

fn foreign_sass_package_identity_for_uri(
    state: &LspShellState,
    uri: &str,
) -> Option<ForeignSassPackageIdentity> {
    let path = file_uri_to_path(uri)?;
    let (package_name, package_root, subpath) = node_modules_package_for_path(path.as_path())?;
    let version = package_version_for_root(package_root.as_path()).unwrap_or_else(|| {
        state
            .document(uri)
            .map(|document| format!("leaf:{}", document.text_hash))
            .or_else(|| {
                fs::read(path.as_path()).ok().map(|bytes| {
                    format!(
                        "leaf:{}",
                        compute_omena_sif_leaf_hash_v1(bytes.as_slice()).as_str()
                    )
                })
            })
            .unwrap_or_else(|| "leaf:unknown".to_string())
    });
    Some(ForeignSassPackageIdentity {
        package_name,
        version,
        subpath,
    })
}

fn package_version_for_root(package_root: &Path) -> Option<String> {
    let source = fs::read_to_string(package_root.join("package.json")).ok()?;
    serde_json::from_str::<Value>(source.as_str())
        .ok()
        .and_then(|json| {
            json.get("version")
                .and_then(Value::as_str)
                .map(str::to_string)
        })
        .filter(|version| !version.is_empty())
}

fn is_foreign_style_document_uri(uri: &str) -> bool {
    is_style_document_uri(uri)
        && file_uri_to_path(uri)
            .as_deref()
            .and_then(node_modules_package_for_path)
            .is_some()
}

fn node_modules_package_for_path(path: &Path) -> Option<(String, PathBuf, String)> {
    let components = path.components().collect::<Vec<_>>();
    for (index, component) in components.iter().enumerate() {
        if !matches!(component, Component::Normal(name) if name.to_str() == Some("node_modules")) {
            continue;
        }
        let package_start = index + 1;
        let first = component_normal_str(components.get(package_start)?)?;
        let (package_name, package_end) = if first.starts_with('@') {
            let second = component_normal_str(components.get(package_start + 1)?)?;
            (format!("{first}/{second}"), package_start + 2)
        } else {
            (first.to_string(), package_start + 1)
        };
        let package_root =
            components[..package_end]
                .iter()
                .fold(PathBuf::new(), |mut root, component| {
                    root.push(component.as_os_str());
                    root
                });
        let subpath = components[package_end..]
            .iter()
            .filter_map(component_normal_str)
            .collect::<Vec<_>>()
            .join("/");
        return Some((
            package_name,
            package_root,
            if subpath.is_empty() {
                ".".to_string()
            } else {
                subpath
            },
        ));
    }
    None
}

fn component_normal_str<'a>(component: &'a Component<'a>) -> Option<&'a str> {
    match component {
        Component::Normal(value) => value.to_str(),
        _ => None,
    }
}

fn style_unresolved_sass_symbol_moniker(
    workspace_folder_uri: Option<&str>,
    candidate: &LspStyleHoverCandidate,
) -> String {
    let family = sass_symbol_kind_from_candidate_kind(candidate.kind).unwrap_or("symbol");
    omena_workspace_moniker(OmenaWorkspaceMonikerInput::SassUnresolvedSymbol {
        workspace_folder_uri,
        family,
        namespace: candidate.namespace.as_deref(),
        name: candidate.name.as_str(),
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ExternalSifSassSymbolTarget {
    canonical_url: String,
    interface_hash: String,
    family: &'static str,
    name: String,
    value_repr: Option<String>,
}

fn style_external_sif_sass_symbol_moniker(target: &ExternalSifSassSymbolTarget) -> String {
    format!(
        "sass-symbol-foreign:sif:{}@{}#{}:{}",
        target.canonical_url, target.interface_hash, target.family, target.name
    )
}

fn external_sif_sass_symbol_target_for_candidate(
    state: &LspShellState,
    document: &LspTextDocumentState,
    candidate: &LspStyleHoverCandidate,
) -> Option<ExternalSifSassSymbolTarget> {
    if is_sass_symbol_declaration_kind(candidate.kind) {
        return None;
    }
    let family = sass_symbol_kind_from_candidate_kind(candidate.kind)?;
    let sources =
        summarize_omena_query_sass_module_sources(document.uri.as_str(), document.text.as_str())?;
    let mut visiting = BTreeSet::new();
    for source in resolve_omena_query_sass_module_use_sources_for_candidate(
        &sources,
        candidate.namespace.as_deref(),
    ) {
        if let Some(target) = external_sif_sass_symbol_target_for_module_source(
            state,
            document,
            source.as_str(),
            family,
            candidate.name.as_str(),
            &mut visiting,
        ) {
            return Some(target);
        }
    }
    if candidate.namespace.is_some() {
        return None;
    }
    for forward_edge in sass_forward_edges_for_document(document) {
        let Some(private_candidate) =
            forward_edge.private_candidate_for_forwarded_public_candidate(candidate)
        else {
            continue;
        };
        if let Some(mut target) = external_sif_sass_symbol_target_for_module_source(
            state,
            document,
            forward_edge.source.as_str(),
            family,
            private_candidate.name.as_str(),
            &mut visiting,
        ) {
            target.name = candidate.name.clone();
            return Some(target);
        }
    }
    None
}

fn external_sif_sass_symbol_target_for_module_source(
    state: &LspShellState,
    document: &LspTextDocumentState,
    source: &str,
    family: &'static str,
    name: &str,
    visiting: &mut BTreeSet<String>,
) -> Option<ExternalSifSassSymbolTarget> {
    let external_sif = external_sif_for_module_source(state, document, source)?;
    external_sif_exported_sass_symbol_target(
        external_sif,
        state.resolution.external_sifs.as_slice(),
        family,
        name,
        visiting,
    )
}

fn external_sif_for_module_source<'a>(
    state: &'a LspShellState,
    document: &LspTextDocumentState,
    source: &str,
) -> Option<&'a OmenaQueryExternalSifInputV0> {
    let mut candidates = BTreeSet::from([source.to_string()]);
    if let Some(uri) = resolve_lsp_style_uri_for_specifier(state, document, source) {
        candidates.insert(uri);
    }
    state.resolution.external_sifs.iter().find(|external_sif| {
        candidates.iter().any(|candidate| {
            external_sif_canonical_urls_match(external_sif.canonical_url.as_str(), candidate)
                || external_sif_canonical_urls_match(
                    external_sif.sif.canonical_url.as_str(),
                    candidate,
                )
        })
    })
}

fn external_sif_exported_sass_symbol_target(
    external_sif: &OmenaQueryExternalSifInputV0,
    external_sifs: &[OmenaQueryExternalSifInputV0],
    family: &'static str,
    name: &str,
    visiting: &mut BTreeSet<String>,
) -> Option<ExternalSifSassSymbolTarget> {
    if !visiting.insert(external_sif.sif.canonical_url.clone()) {
        return None;
    }
    let direct = external_sif_direct_sass_symbol_target(external_sif, family, name);
    if direct.is_some() {
        visiting.remove(external_sif.sif.canonical_url.as_str());
        return direct;
    }
    for forward in &external_sif.sif.exports.forwards {
        let Some(private_name) = unapply_sass_forward_prefix(forward.prefix.as_deref(), name)
        else {
            continue;
        };
        if !external_sif_forward_visibility_allows(forward, family, private_name.as_str()) {
            continue;
        }
        let Some(forwarded_sif) = external_sif_for_forward(external_sif, forward, external_sifs)
        else {
            continue;
        };
        if let Some(mut target) = external_sif_exported_sass_symbol_target(
            forwarded_sif,
            external_sifs,
            family,
            private_name.as_str(),
            visiting,
        ) {
            target.name = name.to_string();
            visiting.remove(external_sif.sif.canonical_url.as_str());
            return Some(target);
        }
    }
    visiting.remove(external_sif.sif.canonical_url.as_str());
    None
}

fn external_sif_direct_sass_symbol_target(
    external_sif: &OmenaQueryExternalSifInputV0,
    family: &'static str,
    name: &str,
) -> Option<ExternalSifSassSymbolTarget> {
    let (name, value_repr) = match family {
        "variable" => external_sif
            .sif
            .exports
            .variables
            .iter()
            .find(|variable| sass_symbol_names_match(variable.name.as_str(), name))
            .map(|variable| {
                (
                    variable.name.trim_start_matches('$').to_string(),
                    variable.value_repr.clone(),
                )
            })?,
        "mixin" => external_sif
            .sif
            .exports
            .mixins
            .iter()
            .find(|mixin| sass_symbol_names_match(mixin.name.as_str(), name))
            .map(|mixin| (mixin.name.clone(), None))?,
        "function" => external_sif
            .sif
            .exports
            .functions
            .iter()
            .find(|function| sass_symbol_names_match(function.name.as_str(), name))
            .map(|function| (function.name.clone(), None))?,
        _ => return None,
    };
    Some(ExternalSifSassSymbolTarget {
        canonical_url: external_sif.sif.canonical_url.clone(),
        interface_hash: external_sif
            .sif
            .fingerprints
            .interface_hash
            .as_str()
            .to_string(),
        family,
        name,
        value_repr,
    })
}

fn external_sif_for_forward<'a>(
    external_sif: &OmenaQueryExternalSifInputV0,
    forward: &omena_sif::OmenaSifForwardExportV1,
    external_sifs: &'a [OmenaQueryExternalSifInputV0],
) -> Option<&'a OmenaQueryExternalSifInputV0> {
    external_sif_forward_canonical_url_candidates(
        external_sif.sif.canonical_url.as_str(),
        forward.canonical_url.as_str(),
    )
    .into_iter()
    .find_map(|candidate| {
        external_sifs.iter().find(|input| {
            external_sif_canonical_urls_match(input.canonical_url.as_str(), candidate.as_str())
                || external_sif_canonical_urls_match(
                    input.sif.canonical_url.as_str(),
                    candidate.as_str(),
                )
        })
    })
}

fn external_sif_forward_canonical_url_candidates(
    base_canonical_url: &str,
    source: &str,
) -> Vec<String> {
    let mut candidates = BTreeSet::from([source.to_string()]);
    if !source.starts_with("sass:")
        && !source.starts_with("http://")
        && !source.starts_with("https://")
        && !source.starts_with("file://")
        && !source.starts_with("pkg:")
        && let Some(base_file_path) = base_canonical_url.strip_prefix("file://")
    {
        let base_path = Path::new(base_file_path);
        let joined = if source.starts_with('/') {
            PathBuf::from(source)
        } else {
            base_path
                .parent()
                .unwrap_or_else(|| Path::new(""))
                .join(source)
        };
        push_external_sif_file_uri_candidates(&mut candidates, joined.as_path());
    }
    candidates.into_iter().collect()
}

fn push_external_sif_file_uri_candidates(candidates: &mut BTreeSet<String>, path: &Path) {
    if path
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some()
    {
        candidates.insert(format!(
            "file://{}",
            normalize_path(path.to_path_buf()).to_string_lossy()
        ));
        return;
    }
    for extension in ["scss", "sass", "css"] {
        let with_extension = path.with_extension(extension);
        candidates.insert(format!(
            "file://{}",
            normalize_path(with_extension).to_string_lossy()
        ));
        if let Some(file_name) = path.file_name().and_then(|file_name| file_name.to_str()) {
            let partial = path
                .with_file_name(format!("_{file_name}"))
                .with_extension(extension);
            candidates.insert(format!(
                "file://{}",
                normalize_path(partial).to_string_lossy()
            ));
        }
    }
}

fn external_sif_forward_visibility_allows(
    forward: &omena_sif::OmenaSifForwardExportV1,
    family: &'static str,
    name: &str,
) -> bool {
    let matches_filter = |filter_name: &String| {
        let exposed_name = apply_sass_forward_prefix(forward.prefix.as_deref(), name);
        filter_name == &exposed_name
            || filter_name.trim_start_matches('$') == exposed_name
            || (family != "variable" && filter_name.trim_start_matches('@') == exposed_name)
    };
    if !forward.show.is_empty() {
        return forward.show.iter().any(matches_filter);
    }
    !forward.hide.iter().any(matches_filter)
}

fn apply_sass_forward_prefix(prefix: Option<&str>, name: &str) -> String {
    match prefix {
        Some(prefix) if prefix.contains('*') => prefix.replace('*', name),
        Some(prefix) => format!("{prefix}{name}"),
        None => name.to_string(),
    }
}

fn external_sif_canonical_urls_match(left: &str, right: &str) -> bool {
    if left == right {
        return true;
    }
    let Some(left_path) = external_sif_canonical_url_path(left) else {
        return false;
    };
    let Some(right_path) = external_sif_canonical_url_path(right) else {
        return false;
    };
    normalize_path(Path::new(left_path.as_str()).to_path_buf())
        == normalize_path(Path::new(right_path.as_str()).to_path_buf())
}

fn external_sif_canonical_url_path(canonical_url: &str) -> Option<String> {
    if let Some(path) = canonical_url.strip_prefix("file://") {
        return Some(path.to_string());
    }
    Path::new(canonical_url)
        .is_absolute()
        .then(|| canonical_url.to_string())
}

fn sass_symbol_names_match(left: &str, right: &str) -> bool {
    fold_sass_symbol_name(left.trim_start_matches('$')) == fold_sass_symbol_name(right)
}

fn fold_sass_symbol_name(name: &str) -> String {
    name.replace('_', "-")
}

fn render_external_sif_sass_symbol_hover_markdown(target: &ExternalSifSassSymbolTarget) -> String {
    let label = match target.family {
        "variable" => format!("`${}`", format_args!("${}", target.name)),
        "mixin" => format!("`@mixin {}`", target.name),
        "function" => format!("`{}()`", target.name),
        _ => format!("`{}`", target.name),
    };
    let mut lines = vec![
        label,
        String::new(),
        format!("External Sass interface from `{}`.", target.canonical_url),
        "Source location is unavailable for this SIF-backed symbol.".to_string(),
    ];
    if let Some(value_repr) = target
        .value_repr
        .as_deref()
        .filter(|value| !value.is_empty())
    {
        lines.push(String::new());
        lines.push(format!("Value: `{value_repr}`"));
    }
    lines.join("\n")
}

fn external_sif_sass_symbol_definition_location(
    state: &LspShellState,
    document: &LspTextDocumentState,
    candidate: &LspStyleHoverCandidate,
) -> Option<Value> {
    let family = sass_symbol_kind_from_candidate_kind(candidate.kind)?;
    let sources =
        summarize_omena_query_sass_module_sources(document.uri.as_str(), document.text.as_str())?;
    let mut visiting = BTreeSet::new();
    let mut target = None;
    for source in resolve_omena_query_sass_module_use_sources_for_candidate(
        &sources,
        candidate.namespace.as_deref(),
    ) {
        target = external_sif_sass_symbol_target_for_module_source(
            state,
            document,
            source.as_str(),
            family,
            candidate.name.as_str(),
            &mut visiting,
        );
        if target.is_some() {
            break;
        }
    }
    let target = target?;
    let range = external_sif_sass_symbol_definition_range(state, &target).or_else(|| {
        style_text_for_uri(state, target.canonical_url.as_str()).map(|_| {
            let start = ParserPositionV0 {
                line: 0,
                character: 0,
            };
            ParserRangeV0 { start, end: start }
        })
    })?;
    Some(json!({
        "uri": target.canonical_url,
        "range": range,
    }))
}

fn external_sif_sass_symbol_definition_range(
    state: &LspShellState,
    target: &ExternalSifSassSymbolTarget,
) -> Option<ParserRangeV0> {
    let (_, candidates) = style_hover_candidates_for_uri(state, target.canonical_url.as_str())?;
    candidates
        .into_iter()
        .find(|candidate| {
            is_sass_symbol_declaration_kind(candidate.kind)
                && sass_symbol_kind_from_candidate_kind(candidate.kind) == Some(target.family)
                && sass_symbol_names_match(candidate.name.as_str(), target.name.as_str())
        })
        .map(|candidate| candidate.range)
}

fn workspace_occurrence_kind_from_style_symbol_kind(
    kind: &str,
) -> Option<OmenaWorkspaceOccurrenceKindV0> {
    match kind {
        "customPropertyDeclaration" => {
            Some(OmenaWorkspaceOccurrenceKindV0::CustomPropertyDeclaration)
        }
        "customPropertyReference" => Some(OmenaWorkspaceOccurrenceKindV0::CustomPropertyReference),
        "sassVariableDeclaration" => Some(OmenaWorkspaceOccurrenceKindV0::SassVariableDeclaration),
        "sassVariableReference" => Some(OmenaWorkspaceOccurrenceKindV0::SassVariableReference),
        "sassMixinDeclaration" => Some(OmenaWorkspaceOccurrenceKindV0::SassMixinDeclaration),
        "sassMixinInclude" => Some(OmenaWorkspaceOccurrenceKindV0::SassMixinInclude),
        "sassFunctionDeclaration" => Some(OmenaWorkspaceOccurrenceKindV0::SassFunctionDeclaration),
        "sassFunctionCall" => Some(OmenaWorkspaceOccurrenceKindV0::SassFunctionCall),
        _ => None,
    }
}

fn workspace_occurrence_role_from_style_symbol_role(role: &str) -> OmenaWorkspaceOccurrenceRoleV0 {
    if role == "definition" {
        OmenaWorkspaceOccurrenceRoleV0::Definition
    } else {
        OmenaWorkspaceOccurrenceRoleV0::Reference
    }
}

fn workspace_occurrence_family_from_style_symbol_family(
    family: &str,
) -> Option<OmenaWorkspaceOccurrenceFamilyV0> {
    match family {
        "customProperty" => Some(OmenaWorkspaceOccurrenceFamilyV0::CustomProperty),
        "variable" => Some(OmenaWorkspaceOccurrenceFamilyV0::Variable),
        "mixin" => Some(OmenaWorkspaceOccurrenceFamilyV0::Mixin),
        "function" => Some(OmenaWorkspaceOccurrenceFamilyV0::Function),
        "symbol" => Some(OmenaWorkspaceOccurrenceFamilyV0::Symbol),
        _ => None,
    }
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
    sass_symbol_declarations_for_uri_with_visited(
        state,
        target_uri,
        symbol_kind,
        candidate,
        &mut BTreeSet::new(),
    )
}

fn sass_symbol_declarations_for_uri_with_visited(
    state: &LspShellState,
    target_uri: &str,
    symbol_kind: &str,
    candidate: &LspStyleHoverCandidate,
    visited: &mut BTreeSet<String>,
) -> Vec<(String, LspStyleHoverCandidate)> {
    if let Some(target_document) = state.document(target_uri) {
        return sass_symbol_declarations_with_forwards(
            state,
            target_document,
            symbol_kind,
            candidate,
            visited,
        );
    }
    let Some(target_document) = style_document_from_disk_for_uri(state, target_uri) else {
        return Vec::new();
    };
    sass_symbol_declarations_with_forwards(state, &target_document, symbol_kind, candidate, visited)
}

fn style_document_from_disk_for_uri(
    state: &LspShellState,
    uri: &str,
) -> Option<LspTextDocumentState> {
    let text = style_text_for_uri(state, uri)?;
    let workspace_folder_uri = resolve_workspace_folder_uri(state, uri);
    let resolution_inputs =
        resolution_inputs_for_workspace_uri(state, workspace_folder_uri.as_deref());
    Some(lsp_text_document_state(
        uri.to_string(),
        workspace_folder_uri,
        StyleLanguage::from_module_path(uri)
            .map(style_language_label)
            .unwrap_or("unknown")
            .to_string(),
        0,
        text,
        &resolution_inputs,
    ))
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
    if summarize_omena_query_sass_module_sources(document.uri.as_str(), document.text.as_str())
        .is_none()
    {
        return definitions;
    }
    for forward_edge in sass_forward_edges_for_document(document) {
        let Some(uri) =
            resolve_lsp_style_uri_for_specifier(state, document, forward_edge.source.as_str())
        else {
            continue;
        };
        let Some(target_candidate) =
            forward_edge.private_candidate_for_forwarded_public_candidate(candidate)
        else {
            continue;
        };
        definitions.extend(sass_symbol_declarations_for_uri_with_visited(
            state,
            uri.as_str(),
            symbol_kind,
            &target_candidate,
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

#[derive(Debug, Clone, PartialEq, Eq)]
struct SassForwardEdgeForLsp {
    source: String,
    forward_prefix: Option<String>,
}

impl SassForwardEdgeForLsp {
    fn private_candidate_for_forwarded_public_candidate(
        &self,
        candidate: &LspStyleHoverCandidate,
    ) -> Option<LspStyleHoverCandidate> {
        let private_name =
            unapply_sass_forward_prefix(self.forward_prefix.as_deref(), &candidate.name)?;
        let mut target = candidate.clone();
        target.name = private_name;
        target.namespace = None;
        Some(target)
    }
}

fn sass_forward_edges_for_document(document: &LspTextDocumentState) -> Vec<SassForwardEdgeForLsp> {
    let facts = summarize_omena_query_omena_parser_style_facts(
        document.text.as_str(),
        query_style_dialect_for_uri(document.uri.as_str()),
    );
    facts
        .sass_module_edges
        .into_iter()
        .filter(|edge| edge.kind == "sassForward")
        .map(|edge| SassForwardEdgeForLsp {
            source: edge.source,
            forward_prefix: edge.forward_prefix,
        })
        .collect()
}

fn unapply_sass_forward_prefix(prefix: Option<&str>, exposed_name: &str) -> Option<String> {
    let Some(prefix) = prefix else {
        return Some(exposed_name.to_string());
    };
    if let Some(star_offset) = prefix.find('*') {
        let before = prefix.get(..star_offset).unwrap_or_default();
        let after = prefix
            .get(star_offset + '*'.len_utf8()..)
            .unwrap_or_default();
        let without_before = exposed_name.strip_prefix(before)?;
        let without_after = if after.is_empty() {
            without_before
        } else {
            without_before.strip_suffix(after)?
        };
        return Some(without_after.to_string());
    }
    exposed_name
        .strip_prefix(prefix)
        .map(str::to_string)
        .filter(|name| !name.is_empty())
}

fn query_style_dialect_for_uri(uri: &str) -> OmenaParserStyleDialect {
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
    let Some(position) = lsp_position_from_params(params) else {
        return empty_hover_trace(document_uri, None, "unknown", None, "missingPosition");
    };
    let Some(document) = state.document(document_uri.as_str()) else {
        return empty_hover_trace(
            document_uri,
            None,
            "unknown",
            Some(position),
            "documentNotIndexed",
        );
    };

    if is_style_document_uri(document.uri.as_str()) {
        return resolve_style_lsp_hover_trace(state, document, position);
    }
    resolve_source_lsp_hover_trace(state, document, position)
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
    if let Some(trace) = source_domain_reference_trace_at_position(document, position) {
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
        return empty_hover_trace(
            document.uri.clone(),
            document.workspace_folder_uri.clone(),
            "source",
            Some(position),
            "noSourceCandidateAtPosition",
        );
    }

    let definitions = style_selector_definitions_for_source_candidates(
        state,
        matched.as_slice(),
        document.workspace_folder_uri.as_deref(),
    );
    let rendered_markdown =
        render_source_hover_definitions_markdown(state, definitions.as_slice()).unwrap_or_default();

    json!({
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
    })
}

fn empty_hover_trace(
    document_uri: String,
    workspace_folder_uri: Option<String>,
    file_kind: &'static str,
    query_position: Option<ParserPositionV0>,
    reason: &'static str,
) -> Value {
    json!({
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
    state: &LspShellState,
    document: &LspTextDocumentState,
    params: Option<&Value>,
) -> Value {
    let Some(position) = lsp_position_from_params(params) else {
        return Value::Null;
    };
    if let Some((range, value)) = source_domain_reference_hover_at_position(document, position) {
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

fn source_domain_reference_at_position(
    document: &LspTextDocumentState,
    position: ParserPositionV0,
) -> Option<&SourceDomainClassReferenceFact> {
    let offset = byte_offset_for_parser_position(document.text.as_str(), position)?;
    document
        .source_syntax_index
        .domain_class_references
        .iter()
        .find(|reference| offset >= reference.byte_span.start && offset <= reference.byte_span.end)
}

fn source_domain_reference_trace_at_position(
    document: &LspTextDocumentState,
    position: ParserPositionV0,
) -> Option<Value> {
    let reference = source_domain_reference_at_position(document, position)?;
    let options = source_domain_reference_option_names(&document.source_syntax_index, reference);
    let current = source_domain_reference_current_option(reference);
    let validity = source_domain_reference_validity(reference, options.as_slice());
    let range = parser_range_for_byte_span(document.text.as_str(), reference.byte_span);

    Some(json!({
        "schemaVersion": "0",
        "product": "omena-lsp-server.explain-hover-trace",
        "documentUri": document.uri.as_str(),
        "workspaceFolderUri": document.workspace_folder_uri.as_deref(),
        "fileKind": "source",
        "languageId": document.language_id.as_str(),
        "queryPosition": position,
        "matched": true,
        "reason": "domainClassReferenceResolved",
        "hoverKind": "domainClassReference",
        "range": range,
        "sourceOwner": reference.owner_name,
        "domain": reference.domain,
        "axisName": reference.axis_name,
        "optionName": reference.option_name,
        "prefix": reference.prefix,
        "currentOption": current,
        "validity": validity,
        "knownOptions": options,
        "candidateCount": 1,
        "definitionCount": 0,
        "candidates": [],
        "definitions": [],
        "renderedMarkdown": render_source_domain_reference_hover_text(reference, options.as_slice()),
        "resolutionPath": ["sourceSyntaxIndex", "classValueUniverseProvider", "sourceDomainReferenceHover"],
        "readySurfaces": ["explainHoverTraceRpc", "sourceSyntaxIndex", "classValueUniverseProvider"],
    }))
}

fn source_domain_reference_hover_at_position(
    document: &LspTextDocumentState,
    position: ParserPositionV0,
) -> Option<(ParserRangeV0, String)> {
    let reference = source_domain_reference_at_position(document, position)?;
    let options = source_domain_reference_option_names(&document.source_syntax_index, reference);
    Some((
        parser_range_for_byte_span(document.text.as_str(), reference.byte_span),
        render_source_domain_reference_hover_text(reference, options.as_slice()),
    ))
}

fn source_domain_reference_current_option(reference: &SourceDomainClassReferenceFact) -> &str {
    reference
        .option_name
        .as_deref()
        .or(reference.prefix.as_deref())
        .unwrap_or("*")
}

fn source_domain_reference_validity(
    reference: &SourceDomainClassReferenceFact,
    options: &[String],
) -> &'static str {
    reference
        .option_name
        .as_ref()
        .map(|option| {
            if options.iter().any(|known| known == option) {
                "known option"
            } else {
                "unknown option"
            }
        })
        .unwrap_or("prefix option")
}

fn render_source_domain_reference_hover_text(
    reference: &SourceDomainClassReferenceFact,
    options: &[String],
) -> String {
    let current = reference
        .option_name
        .as_deref()
        .or(reference.prefix.as_deref())
        .unwrap_or("*");
    let validity = source_domain_reference_validity(reference, options);
    let known_options = if options.is_empty() {
        "No known options indexed.".to_string()
    } else {
        format!("Known options: `{}`.", options.join("`, `"))
    };
    format!(
        "**`{}.{}.{}`**\n\n{} from `{}`.\n\n{}",
        reference.owner_name,
        reference.axis_name,
        current,
        validity,
        reference.domain,
        known_options
    )
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

fn current_provider_tier_feedback_data(
    document: &LspTextDocumentState,
    provider: &'static str,
) -> Option<Value> {
    let feedback = document
        .optimizing_tier_feedback
        .as_ref()
        .filter(|feedback| feedback.document_version == document.version)?;
    Some(json!({
        "product": "omena-lsp-server.provider-tier-feedback",
        "source": feedback.product,
        "provider": provider,
        "policy": feedback.policy,
        "consumer": "hoverCompletionProviderRequest",
        "tier": feedback.analyzed_graph.tier,
        "feedback": "analyzedGraphV0HotStylePrewarm",
        "documentVersion": feedback.document_version,
        "nodeCount": feedback.analyzed_graph.node_count,
        "edgeCount": feedback.analyzed_graph.edge_count,
    }))
}

fn provider_tier_feedback_for_hover_definitions(
    state: &LspShellState,
    definitions: &[(String, LspStyleHoverCandidate)],
) -> Option<Value> {
    definitions.iter().find_map(|(uri, _)| {
        state.document(uri.as_str()).and_then(|document| {
            current_provider_tier_feedback_data(document, "textDocument/hover")
        })
    })
}

fn attach_provider_tier_feedback(response: &mut Value, provider_feedback: Option<&Value>) {
    let Some(provider_feedback) = provider_feedback else {
        return;
    };
    let Some(response_object) = response.as_object_mut() else {
        return;
    };
    let data = response_object.entry("data").or_insert_with(|| json!({}));
    if let Some(data_object) = data.as_object_mut() {
        data_object.insert(
            "providerTierFeedback".to_string(),
            provider_feedback.clone(),
        );
    }
}

struct SourceCompletionContext {
    target_style_uri: Option<String>,
    value_prefix: Option<String>,
    preferred_selector_names: Vec<String>,
    domain_option_names: Vec<String>,
}

fn source_completion_context_at_position(
    state: &LspShellState,
    document: &LspTextDocumentState,
    position: ParserPositionV0,
) -> Option<SourceCompletionContext> {
    let offset = byte_offset_for_parser_position(document.text.as_str(), position)?;
    if let Some(reference) = document
        .source_syntax_index
        .domain_class_references
        .iter()
        .find(|reference| offset >= reference.byte_span.start && offset <= reference.byte_span.end)
    {
        return Some(SourceCompletionContext {
            target_style_uri: None,
            value_prefix: source_completion_prefix_from_span(
                document.text.as_str(),
                reference.byte_span,
                offset,
            ),
            preferred_selector_names: Vec::new(),
            domain_option_names: source_domain_reference_option_names(
                &document.source_syntax_index,
                reference,
            ),
        });
    }
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
            domain_option_names: Vec::new(),
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
            domain_option_names: Vec::new(),
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
            domain_option_names: Vec::new(),
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
            domain_option_names: Vec::new(),
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
            domain_option_names: Vec::new(),
        });
    }
    None
}

fn source_domain_option_completion_items(
    option_names: &[String],
    value_prefix: Option<&str>,
) -> Vec<OmenaQueryCompletionItemV0> {
    let mut items = option_names
        .iter()
        .filter(|option| value_prefix.is_none_or(|prefix| option.starts_with(prefix)))
        .map(|option| OmenaQueryCompletionItemV0 {
            label: option.clone(),
            insert_text: option.clone(),
            sort_text: format!("00-{option}"),
            detail: "Class value option",
            documentation: None,
            item_kind: "classValueOption",
            ranking_source: "classValueUniverseProvider",
            source: "omenaLspSourceCompletion",
        })
        .collect::<Vec<_>>();
    items.sort_by_key(|item| item.label.clone());
    items.dedup_by(|left, right| left.label == right.label);
    items
}

fn source_domain_reference_option_names(
    index: &SourceSyntaxIndex,
    reference: &SourceDomainClassReferenceFact,
) -> Vec<String> {
    let mut options = index
        .class_value_universes
        .iter()
        .filter(|universe| {
            universe.plugin_id == reference.plugin_id
                && universe.domain == reference.domain
                && universe.owner_name == reference.owner_name
        })
        .flat_map(|universe| {
            universe
                .axes
                .iter()
                .filter(|axis| axis.axis_name == reference.axis_name)
                .flat_map(|axis| axis.values.iter().cloned())
        })
        .collect::<Vec<_>>();
    options.sort();
    options.dedup();
    options
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
pub(crate) struct SourceImportIndex {
    pub(crate) imported_style_bindings: Vec<ImportedStyleBinding>,
    pub(crate) classnames_bind_bindings: Vec<String>,
    pub(crate) has_unresolved_style_import: bool,
}

pub(crate) fn collect_source_imports(
    document: &LspTextDocumentState,
    resolution_inputs: &omena_query::OmenaQueryStyleResolutionInputsV0,
) -> SourceImportIndex {
    let source = document.text.as_str();
    let mut imports = SourceImportIndex {
        imported_style_bindings: Vec::new(),
        classnames_bind_bindings: Vec::new(),
        has_unresolved_style_import: false,
    };
    let summary = summarize_omena_query_source_import_declarations_for_source_language(
        document.uri.as_str(),
        source,
        Some(document.language_id.as_str()),
    );
    for import in summary.imports {
        if import.specifier == "classnames/bind" {
            imports.classnames_bind_bindings.push(import.binding);
        } else if StyleLanguage::from_module_path(import.specifier.as_str()).is_some() {
            if let Some(style_uri) =
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
            } else {
                imports.has_unresolved_style_import = true;
            }
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
        .map(|document| (document.uri.clone(), document.as_ref()))
        .next()
}

fn resolve_selector_rename(
    state: &LspShellState,
    workspace_folder_uri: Option<&str>,
    target_style_uri: Option<&str>,
    selector_name: &str,
    new_name: &str,
) -> Value {
    let occurrence_index =
        source_selector_occurrence_index_from_open_documents(state, workspace_folder_uri);
    let query_target_style_uri = query_target_style_uri_for_matching(target_style_uri);
    let rename_plan = summarize_omena_query_rename_plan_from_occurrence_index(
        selector_name,
        new_name,
        query_target_style_uri.as_deref(),
        occurrence_index.definitions.as_slice(),
        &occurrence_index.source_selector_index,
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

fn render_style_hover_candidate_markdown_for_workspace(
    state: &LspShellState,
    document_uri: &str,
    source: &str,
    candidate: &LspStyleHoverCandidate,
) -> String {
    let workspace_folder_uri = state
        .document(document_uri)
        .and_then(|document| document.workspace_folder_uri.clone())
        .or_else(|| resolve_workspace_folder_uri(state, document_uri));
    let style_sources = style_sources_for_hover_render(
        state,
        workspace_folder_uri.as_deref(),
        document_uri,
        source,
    );
    let resolution_inputs =
        resolution_inputs_for_workspace_uri(state, workspace_folder_uri.as_deref());
    let narrowing_substrate = cascade_narrowing_substrate_for_style_sources(
        state,
        style_sources.as_slice(),
        &resolution_inputs,
    );
    let render_parts =
        summarize_omena_query_style_hover_render_parts_for_workspace_file_hover_position_with_substrate(
            document_uri,
            style_sources.as_slice(),
            &narrowing_substrate,
            candidate.kind,
            candidate.name.as_str(),
            candidate.range.start,
        )
        .unwrap_or_else(|| {
            summarize_omena_query_style_hover_render_parts_for_hover_position(
                source,
                candidate.kind,
                candidate.name.as_str(),
                candidate.range.start,
            )
        });
    render_style_hover_candidate_markdown_from_parts(document_uri, candidate, &render_parts)
}

fn style_sources_for_hover_render(
    state: &LspShellState,
    workspace_folder_uri: Option<&str>,
    document_uri: &str,
    source: &str,
) -> Vec<OmenaQueryStyleSourceInputV0> {
    let mut style_sources =
        style_sources_from_open_documents(state, workspace_folder_uri, Some(document_uri));
    if !style_sources
        .iter()
        .any(|style_source| style_source.style_path == document_uri)
    {
        style_sources.push(OmenaQueryStyleSourceInputV0 {
            style_path: document_uri.to_string(),
            style_source: source.to_string(),
        });
    }
    style_sources
}

fn render_style_hover_candidate_markdown_from_parts(
    document_uri: &str,
    candidate: &LspStyleHoverCandidate,
    render_parts: &omena_query::OmenaQueryStyleHoverRenderPartsV0,
) -> String {
    let location = format!(
        "{}:{}",
        file_label_from_uri(document_uri),
        candidate.range.start.line + 1
    );
    match candidate.kind {
        "selector" => {
            let narrowing_markdown =
                render_property_value_narrowings_markdown(&render_parts.property_value_narrowings);
            format!(
                "**`.{}`** - _{}_\n\n```scss\n{}\n```{}",
                candidate.name, location, render_parts.snippet, narrowing_markdown
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
            render_sass_symbol_hover_markdown(candidate, location.as_str(), render_parts)
        }
        _ => candidate.name.clone(),
    }
}

fn render_property_value_narrowings_markdown(
    narrowings: &[omena_query::AbstractPropertyValueNarrowingV0],
) -> String {
    if narrowings.is_empty() {
        return String::new();
    }
    let lines = narrowings
        .iter()
        .take(6)
        .map(|narrowing| {
            format!(
                "- `{}`: {}{}",
                narrowing.property_name,
                render_abstract_property_value(&narrowing.value),
                render_property_value_narrowing_context(narrowing)
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    format!("\n\nCascade narrowed values:\n{lines}")
}

fn render_abstract_property_value(value: &omena_query::AbstractPropertyValueV0) -> String {
    match value {
        omena_query::AbstractPropertyValueV0::Bottom { .. } => "`<bottom>`".to_string(),
        omena_query::AbstractPropertyValueV0::Exact { value, .. } => format!("`{value}`"),
        omena_query::AbstractPropertyValueV0::FiniteSet { values, .. } => values
            .iter()
            .map(|value| format!("`{value}`"))
            .collect::<Vec<_>>()
            .join(" | "),
        omena_query::AbstractPropertyValueV0::CustomPropertyReference {
            custom_property_name,
            ..
        } => {
            format!("`var({custom_property_name})`")
        }
        omena_query::AbstractPropertyValueV0::Top { .. } => "`<top>`".to_string(),
    }
}

fn render_property_value_narrowing_context(
    narrowing: &omena_query::AbstractPropertyValueNarrowingV0,
) -> String {
    let mut context = Vec::new();
    if !narrowing.requested_condition_context.is_empty() {
        context.push(narrowing.requested_condition_context.join(" / "));
    }
    if let Some(layer_name) = narrowing.requested_layer_name.as_deref() {
        context.push(format!("@layer {layer_name}"));
    } else if narrowing.requested_layer_scope == "exactLayer" {
        context.push("unlayered".to_string());
    }
    if context.is_empty() {
        String::new()
    } else {
        format!(" ({})", context.join(", "))
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
