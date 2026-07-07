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
const WORKSPACE_OCCURRENCE_SHARD_DIR: &str = "workspace-occurrence-shards-v1";

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
    let path = workspace_occurrence_shard_path(workspace_folder_uri, document_uri, language_id)?;
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
    let Some(path) =
        workspace_occurrence_shard_path(workspace_folder_uri, document_uri, language_id)
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
    document_uri: &str,
    language_id: &str,
) -> Option<PathBuf> {
    let workspace_folder_uri = workspace_folder_uri?;
    let root = file_uri_to_path(workspace_folder_uri)?;
    // Stable address (identity, never content): one file per document,
    // overwritten in place; the content key is a load-verified shard field.
    let address = crate::disk_cache::stable_cache_shard_address(
        WORKSPACE_OCCURRENCE_SHARD_PRODUCT,
        &[workspace_folder_uri, document_uri, language_id],
    )?;
    let hex = address.strip_prefix("blake3:")?.to_string();
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::path_to_file_uri;
    use omena_query::{
        OmenaQueryStyleResolutionInputsV0, OmenaWorkspaceOccurrenceFamilyV0,
        OmenaWorkspaceOccurrenceKindV0, OmenaWorkspaceOccurrenceRoleV0,
        OmenaWorkspaceOccurrenceSurfaceV0, ParserPositionV0, ParserRangeV0,
    };
    use std::{
        error::Error,
        path::Path,
        time::{SystemTime, UNIX_EPOCH},
    };

    #[test]
    fn workspace_occurrence_shard_excludes_source_text_and_roundtrips_bytes()
    -> Result<(), Box<dyn Error>> {
        let root = unique_temp_root("omena_workspace_occurrence_shard_contract")?;
        let workspace_uri = path_to_file_uri(root.as_path());
        let document_uri = path_to_file_uri(root.join("src/App.tsx").as_path());
        let source_text_sentinel = "LEAK_SENTINEL_source_text_must_not_be_serialized";
        let text_hash = compute_omena_sif_leaf_hash_v1(source_text_sentinel.as_bytes())
            .as_str()
            .to_string();
        let dependency_digest = Some("blake3:dependency-contract");
        let resolution_inputs = OmenaQueryStyleResolutionInputsV0::default();
        let occurrences = vec![fixture_occurrence(document_uri.as_str())];

        store_workspace_occurrence_shard(
            Some(workspace_uri.as_str()),
            document_uri.as_str(),
            "typescriptreact",
            text_hash.as_str(),
            dependency_digest,
            &resolution_inputs,
            occurrences.as_slice(),
        );
        let key = workspace_occurrence_shard_key(
            Some(workspace_uri.as_str()),
            document_uri.as_str(),
            "typescriptreact",
            text_hash.as_str(),
            dependency_digest,
            &resolution_inputs,
        )
        .ok_or("missing workspace occurrence shard key")?;
        let _ = key;
        let shard_path = workspace_occurrence_shard_path(
            Some(workspace_uri.as_str()),
            document_uri.as_str(),
            "typescriptreact",
        )
        .ok_or("missing workspace occurrence shard path")?;
        let first_bytes = fs::read(shard_path.as_path())?;
        let first_json: Value = serde_json::from_slice(first_bytes.as_slice())?;

        assert_eq!(
            first_json.pointer("/containsText").and_then(Value::as_bool),
            Some(false)
        );
        assert!(
            !first_bytes
                .windows(source_text_sentinel.len())
                .any(|window| window == source_text_sentinel.as_bytes()),
            "workspace occurrence shard must not serialize source text bytes"
        );

        let loaded = load_workspace_occurrence_shard(
            Some(workspace_uri.as_str()),
            document_uri.as_str(),
            "typescriptreact",
            text_hash.as_str(),
            dependency_digest,
            &resolution_inputs,
        )
        .ok_or("workspace occurrence shard should reload")?;
        assert_eq!(loaded.occurrences, occurrences);

        store_workspace_occurrence_shard(
            Some(workspace_uri.as_str()),
            document_uri.as_str(),
            "typescriptreact",
            text_hash.as_str(),
            dependency_digest,
            &resolution_inputs,
            loaded.occurrences.as_slice(),
        );
        let second_bytes = fs::read(shard_path.as_path())?;
        assert_eq!(second_bytes, first_bytes);

        let _ = fs::remove_dir_all(root);
        Ok(())
    }

    fn fixture_occurrence(document_uri: &str) -> OmenaWorkspaceOccurrenceV0 {
        OmenaWorkspaceOccurrenceV0 {
            moniker: "css-module-selector:file:///workspace/src/App.module.scss#button".to_string(),
            uri: document_uri.to_string(),
            name: "button".to_string(),
            range: ParserRangeV0 {
                start: ParserPositionV0 {
                    line: 4,
                    character: 20,
                },
                end: ParserPositionV0 {
                    line: 4,
                    character: 28,
                },
            },
            kind: OmenaWorkspaceOccurrenceKindV0::SourceSelectorReference,
            role: OmenaWorkspaceOccurrenceRoleV0::Reference,
            surface: OmenaWorkspaceOccurrenceSurfaceV0::OmenaQuerySourceSyntaxIndex,
            family: Some(OmenaWorkspaceOccurrenceFamilyV0::CssModuleSelector),
            namespace: None,
            target_style_uri: Some("file:///workspace/src/App.module.scss".to_string()),
            rename_target: true,
        }
    }

    fn unique_temp_root(prefix: &str) -> Result<PathBuf, Box<dyn Error>> {
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
        let root = std::env::temp_dir().join(format!("{prefix}_{}_{}", std::process::id(), nanos));
        fs::create_dir_all(root.as_path())?;
        Ok(normalize_test_path(root.as_path()))
    }

    fn normalize_test_path(path: &Path) -> PathBuf {
        path.canonicalize().unwrap_or_else(|_| path.to_path_buf())
    }
}
