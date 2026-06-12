use crate::LspShellState;
use crate::protocol::file_uri_to_path;
use crate::state::LspSourceSelectorOccurrenceDocumentKey;
use omena_query::{
    OmenaQuerySourceSelectorOccurrenceIndexV0, OmenaQuerySourceSelectorOccurrenceV0,
    OmenaQueryStyleSelectorDefinitionV0, ParserRangeV0,
};
use omena_sif::compute_omena_sif_leaf_hash_v1;
use serde_json::{Value, json};
use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

const SOURCE_OCCURRENCE_SIDECAR_PRODUCT: &str = "omena-lsp-server.source-occurrence-index-sidecar";
const SOURCE_OCCURRENCE_SIDECAR_DIR: &str = "source-occurrence-index-v0";

#[derive(Debug, Clone)]
pub(crate) struct LspSourceSelectorOccurrenceSidecarLoadV0 {
    pub(crate) definitions: Vec<OmenaQueryStyleSelectorDefinitionV0>,
    pub(crate) index: Arc<OmenaQuerySourceSelectorOccurrenceIndexV0>,
}

pub(crate) fn load_source_selector_occurrence_sidecar(
    state: &LspShellState,
    workspace_folder_uri: Option<&str>,
    document_keys: &[LspSourceSelectorOccurrenceDocumentKey],
) -> Option<LspSourceSelectorOccurrenceSidecarLoadV0> {
    let key = source_occurrence_sidecar_key(workspace_folder_uri, document_keys)?;
    let path = source_occurrence_sidecar_path(state, workspace_folder_uri, key.as_str())?;
    let bytes = fs::read(path).ok()?;
    let shard: Value = serde_json::from_slice(bytes.as_slice()).ok()?;
    if shard.pointer("/schemaVersion").and_then(Value::as_str) != Some("0")
        || shard.pointer("/product").and_then(Value::as_str)
            != Some(SOURCE_OCCURRENCE_SIDECAR_PRODUCT)
        || shard.pointer("/key").and_then(Value::as_str) != Some(key.as_str())
        || shard.pointer("/workspaceFolderUri").and_then(Value::as_str) != workspace_folder_uri
    {
        return None;
    }
    let expected_document_keys = serde_json::to_value(document_keys).ok()?;
    if shard.pointer("/documentKeys") != Some(&expected_document_keys) {
        return None;
    }
    let payload = shard.pointer("/payload")?;
    let payload_digest = source_occurrence_sidecar_digest(payload)?;
    if shard.pointer("/payloadDigest").and_then(Value::as_str) != Some(payload_digest.as_str()) {
        return None;
    }
    source_occurrence_sidecar_load_from_payload(payload)
}

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

fn source_occurrence_sidecar_load_from_payload(
    payload: &Value,
) -> Option<LspSourceSelectorOccurrenceSidecarLoadV0> {
    let definitions = payload
        .pointer("/definitions")?
        .as_array()?
        .iter()
        .map(source_occurrence_definition_from_value)
        .collect::<Option<Vec<_>>>()?;
    let index = source_occurrence_index_from_value(payload.pointer("/index")?)?;
    Some(LspSourceSelectorOccurrenceSidecarLoadV0 {
        definitions,
        index: Arc::new(index),
    })
}

fn source_occurrence_definition_from_value(
    value: &Value,
) -> Option<OmenaQueryStyleSelectorDefinitionV0> {
    Some(OmenaQueryStyleSelectorDefinitionV0 {
        uri: value.get("uri")?.as_str()?.to_string(),
        name: value.get("name")?.as_str()?.to_string(),
        range: source_occurrence_range_from_value(value.get("range")?)?,
    })
}

fn source_occurrence_index_from_value(
    value: &Value,
) -> Option<OmenaQuerySourceSelectorOccurrenceIndexV0> {
    if value.pointer("/schemaVersion").and_then(Value::as_str) != Some("0")
        || value.pointer("/product").and_then(Value::as_str)
            != Some("omena-query.source-selector-occurrence-index")
    {
        return None;
    }
    let occurrences = value
        .get("occurrences")?
        .as_array()?
        .iter()
        .map(source_occurrence_from_value)
        .collect::<Option<Vec<_>>>()?;
    let occurrence_count = value.get("occurrenceCount")?.as_u64()? as usize;
    if occurrence_count != occurrences.len() {
        return None;
    }
    let moniker_count = occurrences
        .iter()
        .map(|occurrence| occurrence.moniker.as_str())
        .collect::<BTreeSet<_>>()
        .len();
    if value.get("monikerCount")?.as_u64()? as usize != moniker_count {
        return None;
    }
    Some(OmenaQuerySourceSelectorOccurrenceIndexV0 {
        schema_version: "0",
        product: "omena-query.source-selector-occurrence-index",
        moniker_count,
        occurrence_count,
        occurrences,
        ready_surfaces: vec![
            "sourceSelectorOccurrenceIndex",
            "workspaceWideSelectorReferences",
            "workspaceWideSelectorRename",
        ],
    })
}

fn source_occurrence_from_value(value: &Value) -> Option<OmenaQuerySourceSelectorOccurrenceV0> {
    Some(OmenaQuerySourceSelectorOccurrenceV0 {
        moniker: value.get("moniker")?.as_str()?.to_string(),
        uri: value.get("uri")?.as_str()?.to_string(),
        selector_name: value.get("selectorName")?.as_str()?.to_string(),
        range: source_occurrence_range_from_value(value.get("range")?)?,
        kind: source_occurrence_kind_from_value(value.get("kind")?.as_str()?)?,
        role: source_occurrence_role_from_value(value.get("role")?.as_str()?)?,
        source: source_occurrence_source_from_value(value.get("source")?.as_str()?)?,
        target_style_uri: value
            .get("targetStyleUri")
            .and_then(Value::as_str)
            .map(str::to_string),
        rename_target: value.get("renameTarget")?.as_bool()?,
    })
}

fn source_occurrence_range_from_value(value: &Value) -> Option<ParserRangeV0> {
    serde_json::from_value(value.clone()).ok()
}

fn source_occurrence_kind_from_value(value: &str) -> Option<&'static str> {
    match value {
        "sourceSelectorReference" => Some("sourceSelectorReference"),
        "sourceSelectorPrefixReference" => Some("sourceSelectorPrefixReference"),
        _ => None,
    }
}

fn source_occurrence_role_from_value(value: &str) -> Option<&'static str> {
    match value {
        "reference" => Some("reference"),
        _ => None,
    }
}

fn source_occurrence_source_from_value(value: &str) -> Option<&'static str> {
    match value {
        "omenaQuerySourceSyntaxIndex" => Some("omenaQuerySourceSyntaxIndex"),
        _ => None,
    }
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
