use crate::LspShellState;
use crate::cache_limits::{
    DEFAULT_PERSISTENT_CACHE_LIMITS, PersistentCacheLimitsV0, ensure_cache_root_attribution,
    read_cache_shard_with_limits, write_cache_shard_atomically_with_limits,
};
use crate::protocol::file_uri_to_path;
use omena_sif::{compute_omena_sif_leaf_hash_v1, write_omena_canonical_json_bytes_v1};
use omena_tsgo_client::{TsgoResolvedTypeV0, TsgoTypeFactResultEntryV0};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};

const SOURCE_TYPE_FACT_SIDECAR_PRODUCT: &str = "omena-lsp-server.source-type-fact-sidecar";
const SOURCE_TYPE_FACT_SIDECAR_DIR: &str = "source-type-fact-cache-v1";
const SOURCE_TYPE_FACT_SIDECAR_LIMITS: PersistentCacheLimitsV0 = DEFAULT_PERSISTENT_CACHE_LIMITS;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SourceTypeFactSidecarFreshnessV0 {
    pub(crate) environment_fingerprint: String,
    pub(crate) tsgo_binary_fingerprint: String,
    pub(crate) collection_provenance: Value,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SourceTypeFactSidecarRefusalReasonV0 {
    Environment,
    Binary,
    Digest,
    Schema,
}

impl SourceTypeFactSidecarRefusalReasonV0 {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Environment => "environment",
            Self::Binary => "binary",
            Self::Digest => "digest",
            Self::Schema => "schema",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum SourceTypeFactSidecarLoadV0 {
    Miss,
    Refused(SourceTypeFactSidecarRefusalReasonV0),
    Hit(Vec<TsgoTypeFactResultEntryV0>),
}

pub(crate) fn load_source_type_fact_sidecar_with_freshness(
    state: &LspShellState,
    workspace_folder_uri: Option<&str>,
    document_uri: &str,
    key: &str,
    freshness: &SourceTypeFactSidecarFreshnessV0,
) -> SourceTypeFactSidecarLoadV0 {
    let Some(path) = source_type_fact_sidecar_path(state, workspace_folder_uri, document_uri)
    else {
        return SourceTypeFactSidecarLoadV0::Miss;
    };
    let Some(bytes) = read_source_type_fact_shard(path.as_path(), &SOURCE_TYPE_FACT_SIDECAR_LIMITS)
    else {
        return SourceTypeFactSidecarLoadV0::Miss;
    };
    let Ok(shard) = serde_json::from_slice::<Value>(bytes.as_slice()) else {
        return SourceTypeFactSidecarLoadV0::Refused(SourceTypeFactSidecarRefusalReasonV0::Schema);
    };
    if shard.pointer("/schemaVersion").and_then(Value::as_str) != Some("1")
        || shard.pointer("/product").and_then(Value::as_str)
            != Some(SOURCE_TYPE_FACT_SIDECAR_PRODUCT)
        || shard.pointer("/workspaceFolderUri").and_then(Value::as_str) != workspace_folder_uri
    {
        return SourceTypeFactSidecarLoadV0::Refused(SourceTypeFactSidecarRefusalReasonV0::Schema);
    }
    if shard
        .pointer("/environmentFingerprint")
        .and_then(Value::as_str)
        != Some(freshness.environment_fingerprint.as_str())
    {
        return SourceTypeFactSidecarLoadV0::Refused(
            SourceTypeFactSidecarRefusalReasonV0::Environment,
        );
    }
    if shard
        .pointer("/tsgoBinaryFingerprint")
        .and_then(Value::as_str)
        != Some(freshness.tsgo_binary_fingerprint.as_str())
    {
        return SourceTypeFactSidecarLoadV0::Refused(SourceTypeFactSidecarRefusalReasonV0::Binary);
    }
    if shard.pointer("/key").and_then(Value::as_str) != Some(key) {
        return SourceTypeFactSidecarLoadV0::Refused(SourceTypeFactSidecarRefusalReasonV0::Digest);
    }
    let Some(payload) = shard.pointer("/payload") else {
        return SourceTypeFactSidecarLoadV0::Refused(SourceTypeFactSidecarRefusalReasonV0::Schema);
    };
    let Some(payload_digest) = source_type_fact_sidecar_digest(payload) else {
        return SourceTypeFactSidecarLoadV0::Refused(SourceTypeFactSidecarRefusalReasonV0::Digest);
    };
    if shard.pointer("/payloadDigest").and_then(Value::as_str) != Some(payload_digest.as_str()) {
        return SourceTypeFactSidecarLoadV0::Refused(SourceTypeFactSidecarRefusalReasonV0::Digest);
    }
    let Some(entries) = source_type_fact_entries_from_payload(payload) else {
        return SourceTypeFactSidecarLoadV0::Refused(SourceTypeFactSidecarRefusalReasonV0::Schema);
    };
    SourceTypeFactSidecarLoadV0::Hit(entries)
}

pub(crate) fn store_source_type_fact_sidecar_with_freshness(
    state: &LspShellState,
    workspace_folder_uri: Option<&str>,
    document_uri: &str,
    key: &str,
    entries: &[TsgoTypeFactResultEntryV0],
    freshness: &SourceTypeFactSidecarFreshnessV0,
) -> bool {
    let Some(path) = source_type_fact_sidecar_path(state, workspace_folder_uri, document_uri)
    else {
        return false;
    };
    let payload = json!({
        "entries": entries,
        "entryCount": entries.len(),
    });
    let Some(payload_digest) = source_type_fact_sidecar_digest(&payload) else {
        return false;
    };
    let shard = json!({
        "schemaVersion": "1",
        "product": SOURCE_TYPE_FACT_SIDECAR_PRODUCT,
        "key": key,
        "workspaceFolderUri": workspace_folder_uri,
        "environmentFingerprint": freshness.environment_fingerprint,
        "tsgoBinaryFingerprint": freshness.tsgo_binary_fingerprint,
        "collectionProvenance": freshness.collection_provenance,
        "payloadDigest": payload_digest,
        "payload": payload,
    });
    let Ok(bytes) = write_omena_canonical_json_bytes_v1(&shard) else {
        return false;
    };
    if write_source_type_fact_shard(
        path.as_path(),
        bytes.as_slice(),
        &SOURCE_TYPE_FACT_SIDECAR_LIMITS,
    ) {
        if let Some(dir) = path.parent() {
            crate::disk_cache::ensure_omena_cache_root_markers(dir);
            ensure_cache_root_attribution(dir, workspace_folder_uri.unwrap_or(document_uri));
        }
        return true;
    }
    false
}

fn read_source_type_fact_shard(path: &Path, limits: &PersistentCacheLimitsV0) -> Option<Vec<u8>> {
    read_cache_shard_with_limits(path, limits)
}

fn write_source_type_fact_shard(
    path: &Path,
    bytes: &[u8],
    limits: &PersistentCacheLimitsV0,
) -> bool {
    write_cache_shard_atomically_with_limits(path, bytes, limits)
}

fn source_type_fact_sidecar_path(
    state: &LspShellState,
    workspace_folder_uri: Option<&str>,
    document_uri: &str,
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
    // Stable address (identity, never content): one file per document,
    // overwritten in place; the content key is a load-verified shard field.
    let address = crate::disk_cache::stable_cache_shard_address(
        SOURCE_TYPE_FACT_SIDECAR_PRODUCT,
        &[workspace_folder_uri, document_uri],
    )?;
    let hex = address.strip_prefix("blake3:")?.to_string();
    if hex.is_empty() || !hex.chars().all(|character| character.is_ascii_hexdigit()) {
        return None;
    }
    crate::cache_root::resolved_workspace_cache_dir(
        workspace_folder_uri,
        root.as_path(),
        SOURCE_TYPE_FACT_SIDECAR_DIR,
    )
    .map(|dir| dir.join(format!("{hex}.json")))
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
    document_uri: &str,
) -> Option<PathBuf> {
    source_type_fact_sidecar_path(state, workspace_folder_uri, document_uri)
}

#[cfg(test)]
mod cache_limit_tests {
    use super::*;

    #[test]
    fn source_type_fact_store_enforces_reachable_count_byte_and_shard_limits() {
        crate::cache_limits::assert_real_cache_store_enforces_reachable_limits(
            "source-type-fact-sidecar",
            write_source_type_fact_shard,
            read_source_type_fact_shard,
        );
    }
}
