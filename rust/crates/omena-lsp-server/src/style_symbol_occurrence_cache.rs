use crate::LspShellState;
use crate::protocol::file_uri_to_path;
use crate::state::{LspSourceSelectorOccurrenceDocumentKey, LspStyleSymbolOccurrenceV0};
use omena_query::ParserRangeV0;
use omena_sif::compute_omena_sif_leaf_hash_v1;
use serde_json::{Value, json};
use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

const STYLE_SYMBOL_OCCURRENCE_SIDECAR_PRODUCT: &str =
    "omena-lsp-server.style-symbol-occurrence-index-sidecar";
const STYLE_SYMBOL_OCCURRENCE_SIDECAR_DIR: &str = "style-symbol-occurrence-index-v0";

#[derive(Debug, Clone)]
pub(crate) struct LspStyleSymbolOccurrenceSidecarLoadV0 {
    pub(crate) occurrences: Arc<Vec<LspStyleSymbolOccurrenceV0>>,
}

pub(crate) fn load_style_symbol_occurrence_sidecar(
    state: &LspShellState,
    workspace_folder_uri: Option<&str>,
    document_keys: &[LspSourceSelectorOccurrenceDocumentKey],
) -> Option<LspStyleSymbolOccurrenceSidecarLoadV0> {
    let key = style_symbol_occurrence_sidecar_key(workspace_folder_uri, document_keys)?;
    let path = style_symbol_occurrence_sidecar_path(state, workspace_folder_uri, key.as_str())?;
    let bytes = fs::read(path).ok()?;
    let shard: Value = serde_json::from_slice(bytes.as_slice()).ok()?;
    if shard.pointer("/schemaVersion").and_then(Value::as_str) != Some("0")
        || shard.pointer("/product").and_then(Value::as_str)
            != Some(STYLE_SYMBOL_OCCURRENCE_SIDECAR_PRODUCT)
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
    let payload_digest = style_symbol_occurrence_sidecar_digest(payload)?;
    if shard.pointer("/payloadDigest").and_then(Value::as_str) != Some(payload_digest.as_str()) {
        return None;
    }
    style_symbol_occurrence_sidecar_load_from_payload(payload)
}

pub(crate) fn store_style_symbol_occurrence_sidecar(
    state: &LspShellState,
    workspace_folder_uri: Option<&str>,
    document_keys: &[LspSourceSelectorOccurrenceDocumentKey],
    occurrences: &[LspStyleSymbolOccurrenceV0],
) {
    let Some(key) = style_symbol_occurrence_sidecar_key(workspace_folder_uri, document_keys) else {
        return;
    };
    let Some(path) =
        style_symbol_occurrence_sidecar_path(state, workspace_folder_uri, key.as_str())
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

fn style_symbol_occurrence_sidecar_load_from_payload(
    payload: &Value,
) -> Option<LspStyleSymbolOccurrenceSidecarLoadV0> {
    let occurrences = payload
        .get("occurrences")?
        .as_array()?
        .iter()
        .map(style_symbol_occurrence_from_value)
        .collect::<Option<Vec<_>>>()?;
    let occurrence_count = payload.get("occurrenceCount")?.as_u64()? as usize;
    if occurrence_count != occurrences.len() {
        return None;
    }
    let moniker_count = occurrences
        .iter()
        .map(|occurrence| occurrence.moniker.as_str())
        .collect::<BTreeSet<_>>()
        .len();
    if payload.get("monikerCount")?.as_u64()? as usize != moniker_count {
        return None;
    }
    Some(LspStyleSymbolOccurrenceSidecarLoadV0 {
        occurrences: Arc::new(occurrences),
    })
}

fn style_symbol_occurrence_from_value(value: &Value) -> Option<LspStyleSymbolOccurrenceV0> {
    Some(LspStyleSymbolOccurrenceV0 {
        moniker: value.get("moniker")?.as_str()?.to_string(),
        uri: value.get("uri")?.as_str()?.to_string(),
        kind: style_symbol_occurrence_kind_from_value(value.get("kind")?.as_str()?)?,
        family: style_symbol_occurrence_family_from_value(value.get("family")?.as_str()?)?,
        name: value.get("name")?.as_str()?.to_string(),
        range: style_symbol_occurrence_range_from_value(value.get("range")?)?,
        role: style_symbol_occurrence_role_from_value(value.get("role")?.as_str()?)?,
        namespace: value
            .get("namespace")
            .and_then(Value::as_str)
            .map(str::to_string),
    })
}

fn style_symbol_occurrence_range_from_value(value: &Value) -> Option<ParserRangeV0> {
    serde_json::from_value(value.clone()).ok()
}

fn style_symbol_occurrence_kind_from_value(value: &str) -> Option<&'static str> {
    match value {
        "customPropertyDeclaration" => Some("customPropertyDeclaration"),
        "customPropertyReference" => Some("customPropertyReference"),
        "sassVariableDeclaration" => Some("sassVariableDeclaration"),
        "sassVariableReference" => Some("sassVariableReference"),
        "sassMixinDeclaration" => Some("sassMixinDeclaration"),
        "sassMixinInclude" => Some("sassMixinInclude"),
        "sassFunctionDeclaration" => Some("sassFunctionDeclaration"),
        "sassFunctionCall" => Some("sassFunctionCall"),
        _ => None,
    }
}

fn style_symbol_occurrence_family_from_value(value: &str) -> Option<&'static str> {
    match value {
        "customProperty" => Some("customProperty"),
        "variable" => Some("variable"),
        "mixin" => Some("mixin"),
        "function" => Some("function"),
        "symbol" => Some("symbol"),
        _ => None,
    }
}

fn style_symbol_occurrence_role_from_value(value: &str) -> Option<&'static str> {
    match value {
        "definition" => Some("definition"),
        "reference" => Some("reference"),
        _ => None,
    }
}

#[cfg(test)]
pub(crate) fn style_symbol_occurrence_sidecar_file_path_for_test(
    state: &LspShellState,
    workspace_folder_uri: Option<&str>,
    document_keys: &[LspSourceSelectorOccurrenceDocumentKey],
) -> Option<PathBuf> {
    let key = style_symbol_occurrence_sidecar_key(workspace_folder_uri, document_keys)?;
    style_symbol_occurrence_sidecar_path(state, workspace_folder_uri, key.as_str())
}
