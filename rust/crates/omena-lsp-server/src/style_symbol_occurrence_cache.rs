use crate::LspQueryReadView;
use crate::protocol::file_uri_to_path;
use crate::state::{LspSourceSelectorOccurrenceDocumentKey, LspStyleSymbolOccurrenceV0};
use omena_sif::compute_omena_sif_leaf_hash_v1;
use serde_json::{Value, json};
use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

const STYLE_SYMBOL_OCCURRENCE_SIDECAR_PRODUCT: &str =
    "omena-lsp-server.style-symbol-occurrence-index-sidecar";
const STYLE_SYMBOL_OCCURRENCE_SIDECAR_DIR: &str = "style-symbol-occurrence-index-v1";

pub(crate) fn store_style_symbol_occurrence_sidecar(
    state: &dyn LspQueryReadView,
    workspace_folder_uri: Option<&str>,
    document_keys: &[LspSourceSelectorOccurrenceDocumentKey],
    occurrences: &[LspStyleSymbolOccurrenceV0],
) {
    let Some(key) = style_symbol_occurrence_sidecar_key(workspace_folder_uri, document_keys) else {
        return;
    };
    let Some(path) = style_symbol_occurrence_sidecar_path(state, workspace_folder_uri) else {
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
        "occurrences": occurrences,
        "occurrenceCount": occurrences.len(),
        "monikerCount": occurrences
            .iter()
            .map(|occurrence| occurrence.moniker.as_str())
            .collect::<BTreeSet<_>>()
            .len(),
    });
    let Some(payload_digest) = style_symbol_occurrence_sidecar_digest(&payload) else {
        return;
    };
    let shard = json!({
        "schemaVersion": "0",
        "product": STYLE_SYMBOL_OCCURRENCE_SIDECAR_PRODUCT,
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

fn style_symbol_occurrence_sidecar_key(
    workspace_folder_uri: Option<&str>,
    document_keys: &[LspSourceSelectorOccurrenceDocumentKey],
) -> Option<String> {
    let workspace_folder_uri = workspace_folder_uri?;
    let key = json!({
        "schemaVersion": "0",
        "product": "omena-lsp-server.style-symbol-occurrence-index-key",
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

fn style_symbol_occurrence_sidecar_path(
    state: &dyn LspQueryReadView,
    workspace_folder_uri: Option<&str>,
) -> Option<PathBuf> {
    let workspace_folder_uri = workspace_folder_uri?;
    let root = file_uri_to_path(workspace_folder_uri)?;
    if !state
        .query_workspace_runtime_registry()
        .folder_snapshots()
        .iter()
        .any(|folder| folder.uri == workspace_folder_uri)
    {
        return None;
    }
    // Stable address (identity, never content): one file per logical entry,
    // overwritten in place; the content key is a load-verified shard field.
    let address = crate::disk_cache::stable_cache_shard_address(
        STYLE_SYMBOL_OCCURRENCE_SIDECAR_PRODUCT,
        &[workspace_folder_uri],
    )?;
    let hex = address.strip_prefix("blake3:")?.to_string();
    if hex.is_empty() || !hex.chars().all(|character| character.is_ascii_hexdigit()) {
        return None;
    }
    Some(
        root.join(".cache")
            .join("omena")
            .join(STYLE_SYMBOL_OCCURRENCE_SIDECAR_DIR)
            .join(format!("{hex}.json")),
    )
}

fn style_symbol_occurrence_sidecar_digest(value: &Value) -> Option<String> {
    let bytes = serde_json::to_vec(value).ok()?;
    Some(
        compute_omena_sif_leaf_hash_v1(bytes.as_slice())
            .as_str()
            .to_string(),
    )
}

#[cfg(test)]
pub(crate) fn style_symbol_occurrence_sidecar_file_path_for_test(
    state: &dyn LspQueryReadView,
    workspace_folder_uri: Option<&str>,
) -> Option<PathBuf> {
    style_symbol_occurrence_sidecar_path(state, workspace_folder_uri)
}
