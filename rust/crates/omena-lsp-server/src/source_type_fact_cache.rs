use crate::LspShellState;
use crate::protocol::file_uri_to_path;
use omena_sif::{compute_omena_sif_leaf_hash_v1, write_omena_canonical_json_bytes_v1};
use omena_tsgo_client::{TsgoResolvedTypeV0, TsgoTypeFactResultEntryV0};
use serde_json::{Value, json};
use std::{fs, path::PathBuf};

const SOURCE_TYPE_FACT_SIDECAR_PRODUCT: &str = "omena-lsp-server.source-type-fact-sidecar";
const SOURCE_TYPE_FACT_SIDECAR_DIR: &str = "source-type-fact-cache-v0";

pub(crate) fn load_source_type_fact_sidecar(
    state: &LspShellState,
    workspace_folder_uri: Option<&str>,
    key: &str,
) -> Option<Vec<TsgoTypeFactResultEntryV0>> {
    let path = source_type_fact_sidecar_path(state, workspace_folder_uri, key)?;
    let bytes = fs::read(path).ok()?;
    let shard: Value = serde_json::from_slice(bytes.as_slice()).ok()?;
    if shard.pointer("/schemaVersion").and_then(Value::as_str) != Some("0")
        || shard.pointer("/product").and_then(Value::as_str)
            != Some(SOURCE_TYPE_FACT_SIDECAR_PRODUCT)
        || shard.pointer("/key").and_then(Value::as_str) != Some(key)
        || shard.pointer("/workspaceFolderUri").and_then(Value::as_str) != workspace_folder_uri
    {
        return None;
    }
    let payload = shard.pointer("/payload")?;
    let payload_digest = source_type_fact_sidecar_digest(payload)?;
    if shard.pointer("/payloadDigest").and_then(Value::as_str) != Some(payload_digest.as_str()) {
        return None;
    }
    source_type_fact_entries_from_payload(payload)
}

pub(crate) fn store_source_type_fact_sidecar(
    state: &LspShellState,
    workspace_folder_uri: Option<&str>,
    key: &str,
    entries: &[TsgoTypeFactResultEntryV0],
) {
    let Some(path) = source_type_fact_sidecar_path(state, workspace_folder_uri, key) else {
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
        "entries": entries,
        "entryCount": entries.len(),
    });
    let Some(payload_digest) = source_type_fact_sidecar_digest(&payload) else {
        return;
    };
    let shard = json!({
        "schemaVersion": "0",
        "product": SOURCE_TYPE_FACT_SIDECAR_PRODUCT,
        "key": key,
        "workspaceFolderUri": workspace_folder_uri,
        "payloadDigest": payload_digest,
        "payload": payload,
    });
    let Ok(bytes) = write_omena_canonical_json_bytes_v1(&shard) else {
        return;
    };
    let temporary_path = path.with_extension(format!("tmp-{}", std::process::id()));
    if fs::write(temporary_path.as_path(), bytes).is_ok() {
        let _ = fs::rename(temporary_path, path);
    }
}

fn source_type_fact_sidecar_path(
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
            .join(SOURCE_TYPE_FACT_SIDECAR_DIR)
            .join(format!("{hex}.json")),
    )
}

fn source_type_fact_sidecar_digest(value: &Value) -> Option<String> {
    let bytes = write_omena_canonical_json_bytes_v1(value).ok()?;
    Some(
        compute_omena_sif_leaf_hash_v1(bytes.as_slice())
            .as_str()
            .to_string(),
    )
}

fn source_type_fact_entries_from_payload(
    payload: &Value,
) -> Option<Vec<TsgoTypeFactResultEntryV0>> {
    let entries = payload
        .get("entries")?
        .as_array()?
        .iter()
        .map(source_type_fact_entry_from_value)
        .collect::<Option<Vec<_>>>()?;
    let entry_count = payload.get("entryCount")?.as_u64()? as usize;
    if entry_count != entries.len() {
        return None;
    }
    Some(entries)
}

fn source_type_fact_entry_from_value(value: &Value) -> Option<TsgoTypeFactResultEntryV0> {
    Some(TsgoTypeFactResultEntryV0 {
        file_path: value.get("filePath")?.as_str()?.to_string(),
        expression_id: value.get("expressionId")?.as_str()?.to_string(),
        resolved_type: source_type_fact_resolved_type_from_value(value.get("resolvedType")?)?,
    })
}

fn source_type_fact_resolved_type_from_value(value: &Value) -> Option<TsgoResolvedTypeV0> {
    let kind = match value.get("kind")?.as_str()? {
        "union" => "union",
        "unresolvable" => "unresolvable",
        _ => return None,
    };
    let values = value
        .get("values")?
        .as_array()?
        .iter()
        .map(|value| value.as_str().map(str::to_string))
        .collect::<Option<Vec<_>>>()?;
    Some(TsgoResolvedTypeV0 { kind, values })
}

#[cfg(test)]
pub(crate) fn source_type_fact_sidecar_file_path_for_test(
    state: &LspShellState,
    workspace_folder_uri: Option<&str>,
    key: &str,
) -> Option<PathBuf> {
    source_type_fact_sidecar_path(state, workspace_folder_uri, key)
}
