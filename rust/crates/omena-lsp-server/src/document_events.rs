use serde_json::Value;

use omena_query::invalidate_omena_resolver_style_identity_cache;

use crate::{
    LspShellState, LspTextDocumentState, LspWatchedFileChangeState,
    StyleExternalDependencySnapshot, admit_foreign_style_dependencies_for_style_uri,
    byte_offset_for_parser_position, index_workspace_style_files, insert_workspace_folder,
    invalidate_file_uri_identity_cache, is_resolution_config_document_uri, is_style_document_uri,
    lsp_range_from_value, lsp_text_document_state, refresh_document_reusable_indexes,
    refresh_external_sifs_for_state, refresh_source_indexes_for_resolution_config_change,
    refresh_source_indexes_for_style_document_change,
    refresh_source_type_fact_candidates_for_document,
    refresh_style_external_inputs_after_document_removal,
    refresh_style_external_inputs_for_document_event, refresh_workspace_resolution_inputs,
    reload_indexed_source_document_from_disk, reload_indexed_style_document_from_disk,
    resolution_inputs_for_workspace_uri, resolve_workspace_folder_uri,
    style_external_dependency_snapshot,
};

pub(crate) fn did_open_text_document(state: &mut LspShellState, params: Option<&Value>) {
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
        let started = std::time::Instant::now();
        refresh_style_external_inputs_for_document_event(state, uri, None);
        let external_ms = started.elapsed().as_millis();
        let started = std::time::Instant::now();
        refresh_source_indexes_for_style_document_change(state, uri);
        crate::loop_trace!(
            "did-open style uri={} external_ms={} source_index_ms={}",
            uri,
            external_ms,
            started.elapsed().as_millis()
        );
    } else {
        let started = std::time::Instant::now();
        refresh_source_type_fact_candidates_for_document(state, uri);
        crate::loop_trace!(
            "did-open source uri={} type_fact_ms={}",
            uri,
            started.elapsed().as_millis()
        );
    }
}

pub(crate) fn did_change_text_document(state: &mut LspShellState, params: Option<&Value>) {
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

pub(crate) fn did_close_text_document(state: &mut LspShellState, params: Option<&Value>) {
    let Some(uri) = params
        .and_then(|value| value.get("textDocument"))
        .and_then(|value| value.get("uri"))
        .and_then(Value::as_str)
    else {
        return;
    };
    invalidate_file_uri_identity_cache();
    invalidate_omena_resolver_style_identity_cache();
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

pub(crate) fn did_change_workspace_folders(
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
    crate::refresh_document_workspace_owners(state);
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
    let file_identity = state.file_identity.clone();
    state.documents.retain(|file_id, document| {
        if open_document_uris.contains(file_id) {
            return true;
        }
        let uri = file_identity
            .storage_uri_for_file_id(*file_id)
            .unwrap_or(document.uri.as_str());
        let owned_by_removed_workspace =
            document
                .workspace_folder_uri
                .as_deref()
                .is_some_and(|workspace_uri| {
                    removed_workspace_uris
                        .iter()
                        .any(|removed_uri| removed_uri == workspace_uri)
                });
        !owned_by_removed_workspace || workspace_runtime_registry.resolve_owner_uri(uri).is_some()
    });
}

pub(crate) fn did_change_watched_files(state: &mut LspShellState, params: Option<&Value>) {
    let Some(changes) = params
        .and_then(|value| value.get("changes"))
        .and_then(Value::as_array)
    else {
        return;
    };
    invalidate_file_uri_identity_cache();
    invalidate_omena_resolver_style_identity_cache();
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
