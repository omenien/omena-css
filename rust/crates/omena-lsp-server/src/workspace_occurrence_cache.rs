use crate::protocol::file_uri_to_path;
use omena_query::{OmenaQueryStyleResolutionInputsV0, OmenaWorkspaceOccurrenceV0};
use omena_sif::{compute_omena_sif_leaf_hash_v1, write_omena_canonical_json_bytes_v1};
use serde::Serialize;
use serde_json::{Value, json};
use std::{collections::BTreeSet, fs, path::PathBuf};

const WORKSPACE_OCCURRENCE_SHARD_SCHEMA_VERSION: &str = "0";
const WORKSPACE_OCCURRENCE_SHARD_PRODUCT: &str = "omena-lsp-server.workspace-occurrence-shard";
const WORKSPACE_OCCURRENCE_SHARD_KEY_PRODUCT: &str =
    "omena-lsp-server.workspace-occurrence-shard-key";
const WORKSPACE_OCCURRENCE_SHARD_DIR: &str = "workspace-occurrence-shards-v0";

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct WorkspaceOccurrenceShardKeyInputV0<'a> {
    schema_version: &'a str,
    crate_version: &'a str,
    product: &'a str,
    workspace_folder_uri: Option<&'a str>,
    document_uri: &'a str,
    language_id: &'a str,
    text_hash: &'a str,
    dependency_digest: Option<&'a str>,
    resolution_inputs: &'a OmenaQueryStyleResolutionInputsV0,
}

#[derive(Debug, Clone)]
pub(crate) struct LspWorkspaceOccurrenceShardLoadV0 {
    pub(crate) occurrences: Vec<OmenaWorkspaceOccurrenceV0>,
}

pub(crate) fn load_workspace_occurrence_shard(
    workspace_folder_uri: Option<&str>,
    document_uri: &str,
    language_id: &str,
    text_hash: &str,
    dependency_digest: Option<&str>,
    resolution_inputs: &OmenaQueryStyleResolutionInputsV0,
) -> Option<LspWorkspaceOccurrenceShardLoadV0> {
    let key = workspace_occurrence_shard_key(
        workspace_folder_uri,
        document_uri,
        language_id,
        text_hash,
        dependency_digest,
        resolution_inputs,
    )?;
    let path = workspace_occurrence_shard_path(workspace_folder_uri, key.as_str())?;
    let bytes = fs::read(path).ok()?;
    let shard: Value = serde_json::from_slice(bytes.as_slice()).ok()?;
    if shard.pointer("/schemaVersion").and_then(Value::as_str)
        != Some(WORKSPACE_OCCURRENCE_SHARD_SCHEMA_VERSION)
        || shard.pointer("/product").and_then(Value::as_str)
            != Some(WORKSPACE_OCCURRENCE_SHARD_PRODUCT)
        || shard.pointer("/key").and_then(Value::as_str) != Some(key.as_str())
        || shard.pointer("/documentUri").and_then(Value::as_str) != Some(document_uri)
        || shard.pointer("/workspaceFolderUri").and_then(Value::as_str) != workspace_folder_uri
        || shard.pointer("/languageId").and_then(Value::as_str) != Some(language_id)
        || shard.pointer("/textHash").and_then(Value::as_str) != Some(text_hash)
        || shard.pointer("/containsText").and_then(Value::as_bool) != Some(false)
    {
        return None;
    }
    let payload = shard.pointer("/payload")?;
    let payload_digest = workspace_occurrence_shard_digest(payload)?;
    if shard.pointer("/payloadDigest").and_then(Value::as_str) != Some(payload_digest.as_str()) {
        return None;
    }
    let occurrences = payload
        .get("occurrences")?
        .as_array()?
        .iter()
        .map(|value| serde_json::from_value(value.clone()).ok())
        .collect::<Option<Vec<_>>>()?;
    let occurrence_count = payload.get("occurrenceCount")?.as_u64()? as usize;
    if occurrence_count != occurrences.len() {
        return None;
    }
    let moniker_count = occurrences
        .iter()
        .map(|occurrence: &OmenaWorkspaceOccurrenceV0| occurrence.moniker.as_str())
        .collect::<BTreeSet<_>>()
        .len();
    if payload.get("monikerCount")?.as_u64()? as usize != moniker_count {
        return None;
    }
    Some(LspWorkspaceOccurrenceShardLoadV0 { occurrences })
}

pub(crate) fn store_workspace_occurrence_shard(
    workspace_folder_uri: Option<&str>,
    document_uri: &str,
    language_id: &str,
    text_hash: &str,
    dependency_digest: Option<&str>,
    resolution_inputs: &OmenaQueryStyleResolutionInputsV0,
    occurrences: &[OmenaWorkspaceOccurrenceV0],
) {
    let Some(key) = workspace_occurrence_shard_key(
        workspace_folder_uri,
        document_uri,
        language_id,
        text_hash,
        dependency_digest,
        resolution_inputs,
    ) else {
        return;
    };
    let Some(path) = workspace_occurrence_shard_path(workspace_folder_uri, key.as_str()) else {
        return;
    };
    let Some(dir) = path.parent() else {
        return;
    };
    if fs::create_dir_all(dir).is_err() {
        return;
    }
    let payload = json!({
        "occurrences": occurrences,
        "occurrenceCount": occurrences.len(),
        "monikerCount": occurrences
            .iter()
            .map(|occurrence| occurrence.moniker.as_str())
            .collect::<BTreeSet<_>>()
            .len(),
    });
    let Some(payload_digest) = workspace_occurrence_shard_digest(&payload) else {
        return;
    };
    let shard = json!({
        "schemaVersion": WORKSPACE_OCCURRENCE_SHARD_SCHEMA_VERSION,
        "product": WORKSPACE_OCCURRENCE_SHARD_PRODUCT,
        "key": key,
        "documentUri": document_uri,
        "workspaceFolderUri": workspace_folder_uri,
        "languageId": language_id,
        "textHash": text_hash,
        "containsText": false,
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

pub(crate) fn workspace_occurrence_dependency_digest<T: Serialize>(value: &T) -> Option<String> {
    let bytes = write_omena_canonical_json_bytes_v1(value).ok()?;
    Some(
        compute_omena_sif_leaf_hash_v1(bytes.as_slice())
            .as_str()
            .to_string(),
    )
}

fn workspace_occurrence_shard_key(
    workspace_folder_uri: Option<&str>,
    document_uri: &str,
    language_id: &str,
    text_hash: &str,
    dependency_digest: Option<&str>,
    resolution_inputs: &OmenaQueryStyleResolutionInputsV0,
) -> Option<String> {
    let input = WorkspaceOccurrenceShardKeyInputV0 {
        schema_version: WORKSPACE_OCCURRENCE_SHARD_SCHEMA_VERSION,
        crate_version: env!("CARGO_PKG_VERSION"),
        product: WORKSPACE_OCCURRENCE_SHARD_KEY_PRODUCT,
        workspace_folder_uri,
        document_uri,
        language_id,
        text_hash,
        dependency_digest,
        resolution_inputs,
    };
    let bytes = write_omena_canonical_json_bytes_v1(&input).ok()?;
    Some(
        compute_omena_sif_leaf_hash_v1(bytes.as_slice())
            .as_str()
            .to_string(),
    )
}

fn workspace_occurrence_shard_path(
    workspace_folder_uri: Option<&str>,
    key: &str,
) -> Option<PathBuf> {
    let workspace_folder_uri = workspace_folder_uri?;
    let root = file_uri_to_path(workspace_folder_uri)?;
    let hex = key.strip_prefix("blake3:")?;
    if hex.is_empty() || !hex.chars().all(|character| character.is_ascii_hexdigit()) {
        return None;
    }
    Some(
        root.join(".cache")
            .join("omena")
            .join(WORKSPACE_OCCURRENCE_SHARD_DIR)
            .join(format!("{hex}.json")),
    )
}

fn workspace_occurrence_shard_digest(value: &Value) -> Option<String> {
    let bytes = write_omena_canonical_json_bytes_v1(value).ok()?;
    Some(
        compute_omena_sif_leaf_hash_v1(bytes.as_slice())
            .as_str()
            .to_string(),
    )
}
