use crate::LspShellState;
use crate::protocol::file_uri_to_path;
use crate::state::LspSourceSelectorOccurrenceDocumentKey;
use omena_query::{OmenaQuerySourceSelectorOccurrenceIndexV0, OmenaQueryStyleSelectorDefinitionV0};
use omena_sif::compute_omena_sif_leaf_hash_v1;
use serde_json::{Value, json};
use std::fs;
use std::path::PathBuf;

const SOURCE_OCCURRENCE_SIDECAR_PRODUCT: &str = "omena-lsp-server.source-occurrence-index-sidecar";
const SOURCE_OCCURRENCE_SIDECAR_DIR: &str = "source-occurrence-index-v0";

pub(crate) fn store_source_selector_occurrence_sidecar(
    state: &LspShellState,
    workspace_folder_uri: Option<&str>,
    document_keys: &[LspSourceSelectorOccurrenceDocumentKey],
    definitions: &[OmenaQueryStyleSelectorDefinitionV0],
    index: &OmenaQuerySourceSelectorOccurrenceIndexV0,
) {
    let Some(key) = source_occurrence_sidecar_key(workspace_folder_uri, document_keys) else {
        return;
    };
    let Some(path) = source_occurrence_sidecar_path(state, workspace_folder_uri, key.as_str())
    else {
        return;
    };
    let Some(dir) = path.parent() else {
        return;
    };
    if fs::create_dir_all(dir).is_err() {
        return;
    }
    crate::disk_cache::ensure_omena_cache_root_markers(dir);
    let payload = json!({
        "definitions": definitions,
        "index": index,
    });
    let Some(payload_digest) = source_occurrence_sidecar_digest(&payload) else {
        return;
    };
    let shard = json!({
        "schemaVersion": "0",
        "product": SOURCE_OCCURRENCE_SIDECAR_PRODUCT,
        "key": key,
        "workspaceFolderUri": workspace_folder_uri,
        "documentKeys": document_keys,
        "payloadDigest": payload_digest,
        "payload": payload,
    });
    let Ok(bytes) = serde_json::to_vec(&shard) else {
        return;
    };
    let temporary_path = path.with_extension(format!("tmp-{}", std::process::id()));
    if fs::write(temporary_path.as_path(), bytes).is_ok() {
        let _ = fs::rename(temporary_path, path);
    }
}

fn source_occurrence_sidecar_key(
    workspace_folder_uri: Option<&str>,
    document_keys: &[LspSourceSelectorOccurrenceDocumentKey],
) -> Option<String> {
    let workspace_folder_uri = workspace_folder_uri?;
    let key = json!({
        "schemaVersion": "0",
        "product": "omena-lsp-server.source-occurrence-index-key",
        "workspaceFolderUri": workspace_folder_uri,
        "documentKeys": document_keys,
    });
    let bytes = serde_json::to_vec(&key).ok()?;
    Some(
        compute_omena_sif_leaf_hash_v1(bytes.as_slice())
            .as_str()
            .to_string(),
    )
}

fn source_occurrence_sidecar_path(
    state: &LspShellState,
    workspace_folder_uri: Option<&str>,
    key: &str,
) -> Option<PathBuf> {
    let workspace_folder_uri = workspace_folder_uri?;
    let root = file_uri_to_path(workspace_folder_uri)?;
    if !state
        .workspace_runtime_registry
        .folder_snapshots()
        .iter()
        .any(|folder| folder.uri == workspace_folder_uri)
    {
        return None;
    }
    let hex = key.strip_prefix("blake3:")?;
    if hex.is_empty() || !hex.chars().all(|character| character.is_ascii_hexdigit()) {
        return None;
    }
    Some(
        root.join(".cache")
            .join("omena")
            .join(SOURCE_OCCURRENCE_SIDECAR_DIR)
            .join(format!("{hex}.json")),
    )
}

fn source_occurrence_sidecar_digest(value: &Value) -> Option<String> {
    let bytes = serde_json::to_vec(value).ok()?;
    Some(
        compute_omena_sif_leaf_hash_v1(bytes.as_slice())
            .as_str()
            .to_string(),
    )
}

#[cfg(test)]
pub(crate) fn source_occurrence_sidecar_file_path_for_test(
    state: &LspShellState,
    workspace_folder_uri: Option<&str>,
    document_keys: &[LspSourceSelectorOccurrenceDocumentKey],
) -> Option<PathBuf> {
    let key = source_occurrence_sidecar_key(workspace_folder_uri, document_keys)?;
    source_occurrence_sidecar_path(state, workspace_folder_uri, key.as_str())
}
