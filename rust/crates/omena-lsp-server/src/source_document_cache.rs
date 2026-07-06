use crate::protocol::file_uri_to_path;
use omena_query::{
    OmenaQuerySourceClassValueUniverseAxisV0, OmenaQuerySourceClassValueUniverseEntryV0,
    OmenaQuerySourceDomainClassReferenceFactV0, OmenaQuerySourceImportedStyleBindingV0,
    OmenaQuerySourceInlineStyleDeclarationFactV0, OmenaQuerySourceSelectorReferenceFactV0,
    OmenaQuerySourceSelectorReferenceMatchKindV0, OmenaQuerySourceStylePropertyAccessFactV0,
    OmenaQuerySourceSyntaxIndexV0, OmenaQuerySourceTypeFactProviderUnavailableFactV0,
    OmenaQuerySourceTypeFactTargetV0, OmenaQueryStyleResolutionInputsV0, ParserByteSpanV0,
};
use omena_sif::{compute_omena_sif_leaf_hash_v1, write_omena_canonical_json_bytes_v1};
use serde::Serialize;
use serde_json::{Value, json};
use std::{fs, path::PathBuf};

const SOURCE_DOCUMENT_INDEX_SCHEMA_VERSION: &str = "0";
const SOURCE_DOCUMENT_INDEX_SIDECAR_PRODUCT: &str =
    "omena-lsp-server.source-document-index-sidecar";
const SOURCE_DOCUMENT_INDEX_KEY_PRODUCT: &str = "omena-lsp-server.source-document-index-key";
const SOURCE_DOCUMENT_INDEX_DIR: &str = "source-document-index-v0";

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SourceDocumentIndexKeyInputV0<'a> {
    schema_version: &'a str,
    crate_version: &'a str,
    product: &'a str,
    document_uri: &'a str,
    workspace_folder_uri: Option<&'a str>,
    language_id: &'a str,
    text_hash: &'a str,
    resolution_inputs: &'a OmenaQueryStyleResolutionInputsV0,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct LspSourceDocumentIndexSidecarLoadV0 {
    pub(crate) source_syntax_index: OmenaQuerySourceSyntaxIndexV0,
    pub(crate) has_unresolved_style_import: bool,
}

pub(crate) fn load_source_document_index_sidecar(
    workspace_folder_uri: Option<&str>,
    document_uri: &str,
    language_id: &str,
    text_hash: &str,
    resolution_inputs: &OmenaQueryStyleResolutionInputsV0,
) -> Option<LspSourceDocumentIndexSidecarLoadV0> {
    let key = source_document_index_key(
        workspace_folder_uri,
        document_uri,
        language_id,
        text_hash,
        resolution_inputs,
    )?;
    let path = source_document_index_sidecar_path(workspace_folder_uri, key.as_str())?;
    let bytes = fs::read(path).ok()?;
    let shard: Value = serde_json::from_slice(bytes.as_slice()).ok()?;
    if shard.pointer("/schemaVersion").and_then(Value::as_str)
        != Some(SOURCE_DOCUMENT_INDEX_SCHEMA_VERSION)
        || shard.pointer("/product").and_then(Value::as_str)
            != Some(SOURCE_DOCUMENT_INDEX_SIDECAR_PRODUCT)
        || shard.pointer("/key").and_then(Value::as_str) != Some(key.as_str())
        || shard.pointer("/documentUri").and_then(Value::as_str) != Some(document_uri)
        || shard.pointer("/workspaceFolderUri").and_then(Value::as_str) != workspace_folder_uri
        || shard.pointer("/languageId").and_then(Value::as_str) != Some(language_id)
        || shard.pointer("/textHash").and_then(Value::as_str) != Some(text_hash)
    {
        return None;
    }
    let payload = shard.pointer("/payload")?;
    let payload_digest = source_document_index_payload_digest(payload)?;
    if shard.pointer("/payloadDigest").and_then(Value::as_str) != Some(payload_digest.as_str()) {
        return None;
    }
    Some(LspSourceDocumentIndexSidecarLoadV0 {
        source_syntax_index: source_syntax_index_from_value(
            payload.pointer("/sourceSyntaxIndex")?,
        )?,
        has_unresolved_style_import: payload
            .pointer("/hasUnresolvedStyleImport")
            .and_then(Value::as_bool)?,
    })
}

pub(crate) fn store_source_document_index_sidecar(
    workspace_folder_uri: Option<&str>,
    document_uri: &str,
    language_id: &str,
    text_hash: &str,
    resolution_inputs: &OmenaQueryStyleResolutionInputsV0,
    source_syntax_index: &OmenaQuerySourceSyntaxIndexV0,
    has_unresolved_style_import: bool,
) {
    let Some(key) = source_document_index_key(
        workspace_folder_uri,
        document_uri,
        language_id,
        text_hash,
        resolution_inputs,
    ) else {
        return;
    };
    let Some(path) = source_document_index_sidecar_path(workspace_folder_uri, key.as_str()) else {
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
        "sourceSyntaxIndex": source_syntax_index,
        "hasUnresolvedStyleImport": has_unresolved_style_import,
    });
    let Some(payload_digest) = source_document_index_payload_digest(&payload) else {
        return;
    };
    let shard = json!({
        "schemaVersion": SOURCE_DOCUMENT_INDEX_SCHEMA_VERSION,
        "product": SOURCE_DOCUMENT_INDEX_SIDECAR_PRODUCT,
        "key": key,
        "documentUri": document_uri,
        "workspaceFolderUri": workspace_folder_uri,
        "languageId": language_id,
        "textHash": text_hash,
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

pub(crate) fn source_document_text_hash(text: &str) -> String {
    compute_omena_sif_leaf_hash_v1(text.as_bytes())
        .as_str()
        .to_string()
}

#[cfg(test)]
pub(crate) fn source_document_index_sidecar_file_path_for_test(
    workspace_folder_uri: Option<&str>,
    document_uri: &str,
    language_id: &str,
    text_hash: &str,
    resolution_inputs: &OmenaQueryStyleResolutionInputsV0,
) -> Option<PathBuf> {
    let key = source_document_index_key(
        workspace_folder_uri,
        document_uri,
        language_id,
        text_hash,
        resolution_inputs,
    )?;
    source_document_index_sidecar_path(workspace_folder_uri, key.as_str())
}

fn source_document_index_key(
    workspace_folder_uri: Option<&str>,
    document_uri: &str,
    language_id: &str,
    text_hash: &str,
    resolution_inputs: &OmenaQueryStyleResolutionInputsV0,
) -> Option<String> {
    let input = SourceDocumentIndexKeyInputV0 {
        schema_version: SOURCE_DOCUMENT_INDEX_SCHEMA_VERSION,
        crate_version: env!("CARGO_PKG_VERSION"),
        product: SOURCE_DOCUMENT_INDEX_KEY_PRODUCT,
        document_uri,
        workspace_folder_uri,
        language_id,
        text_hash,
        resolution_inputs,
    };
    let bytes = write_omena_canonical_json_bytes_v1(&input).ok()?;
    Some(
        compute_omena_sif_leaf_hash_v1(bytes.as_slice())
            .as_str()
            .to_string(),
    )
}

fn source_document_index_sidecar_path(
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
            .join(SOURCE_DOCUMENT_INDEX_DIR)
            .join(format!("{hex}.json")),
    )
}

fn source_document_index_payload_digest(value: &Value) -> Option<String> {
    let bytes = write_omena_canonical_json_bytes_v1(value).ok()?;
    Some(
        compute_omena_sif_leaf_hash_v1(bytes.as_slice())
            .as_str()
            .to_string(),
    )
}

fn source_syntax_index_from_value(value: &Value) -> Option<OmenaQuerySourceSyntaxIndexV0> {
    if value.pointer("/schemaVersion").and_then(Value::as_str) != Some("0")
        || value.pointer("/product").and_then(Value::as_str)
            != Some("omena-bridge.source-syntax-index")
    {
        return None;
    }
    Some(OmenaQuerySourceSyntaxIndexV0 {
        schema_version: "0",
        product: "omena-bridge.source-syntax-index",
        imported_style_bindings: imported_style_bindings_from_value(
            value.get("importedStyleBindings")?,
        )?,
        class_string_literals: byte_spans_from_value(value.get("classStringLiterals")?)?,
        style_property_accesses: style_property_accesses_from_value(
            value.get("stylePropertyAccesses")?,
        )?,
        inline_style_declarations: inline_style_declarations_from_value(
            value.get("inlineStyleDeclarations")?,
        )?,
        selector_references: selector_references_from_value(value.get("selectorReferences")?)?,
        type_fact_targets: type_fact_targets_from_value(value.get("typeFactTargets")?)?,
        type_fact_provider_unavailable: match value.get("typeFactProviderUnavailable") {
            Some(facts) => type_fact_provider_unavailable_from_value(facts)?,
            None => Vec::new(),
        },
        class_value_universes: class_value_universes_from_value(value.get("classValueUniverses")?)?,
        domain_class_references: domain_class_references_from_value(
            value.get("domainClassReferences")?,
        )?,
    })
}

fn imported_style_bindings_from_value(
    value: &Value,
) -> Option<Vec<OmenaQuerySourceImportedStyleBindingV0>> {
    value
        .as_array()?
        .iter()
        .map(|binding| {
            Some(OmenaQuerySourceImportedStyleBindingV0 {
                binding: binding.get("binding")?.as_str()?.to_string(),
                style_uri: binding.get("styleUri")?.as_str()?.to_string(),
            })
        })
        .collect()
}

fn byte_spans_from_value(value: &Value) -> Option<Vec<ParserByteSpanV0>> {
    value.as_array()?.iter().map(byte_span_from_value).collect()
}

fn style_property_accesses_from_value(
    value: &Value,
) -> Option<Vec<OmenaQuerySourceStylePropertyAccessFactV0>> {
    value
        .as_array()?
        .iter()
        .map(|access| {
            Some(OmenaQuerySourceStylePropertyAccessFactV0 {
                byte_span: byte_span_from_value(access.get("byteSpan")?)?,
                target_style_uri: access
                    .get("targetStyleUri")
                    .and_then(Value::as_str)
                    .map(str::to_string),
            })
        })
        .collect()
}

fn inline_style_declarations_from_value(
    value: &Value,
) -> Option<Vec<OmenaQuerySourceInlineStyleDeclarationFactV0>> {
    value
        .as_array()?
        .iter()
        .map(|declaration| {
            Some(OmenaQuerySourceInlineStyleDeclarationFactV0 {
                byte_span: byte_span_from_value(declaration.get("byteSpan")?)?,
                value_byte_span: optional_byte_span_from_value(declaration.get("valueByteSpan"))?,
                property_name: declaration.get("propertyName")?.as_str()?.to_string(),
                value: declaration
                    .get("value")
                    .and_then(Value::as_str)
                    .map(str::to_string),
                target_style_uri: declaration
                    .get("targetStyleUri")
                    .and_then(Value::as_str)
                    .map(str::to_string),
                cascade_tier: cascade_tier_from_value(declaration.get("cascadeTier")?)?,
                static_value: declaration.get("staticValue")?.as_bool()?,
            })
        })
        .collect()
}

fn selector_references_from_value(
    value: &Value,
) -> Option<Vec<OmenaQuerySourceSelectorReferenceFactV0>> {
    value
        .as_array()?
        .iter()
        .map(|reference| {
            Some(OmenaQuerySourceSelectorReferenceFactV0 {
                byte_span: byte_span_from_value(reference.get("byteSpan")?)?,
                selector_name: reference
                    .get("selectorName")
                    .and_then(Value::as_str)
                    .map(str::to_string),
                match_kind: selector_match_kind_from_value(reference.get("matchKind")?)?,
                target_style_uri: reference
                    .get("targetStyleUri")
                    .and_then(Value::as_str)
                    .map(str::to_string),
            })
        })
        .collect()
}

fn type_fact_targets_from_value(value: &Value) -> Option<Vec<OmenaQuerySourceTypeFactTargetV0>> {
    value
        .as_array()?
        .iter()
        .map(|target| {
            Some(OmenaQuerySourceTypeFactTargetV0 {
                byte_span: byte_span_from_value(target.get("byteSpan")?)?,
                expression_id: target.get("expressionId")?.as_str()?.to_string(),
                target_style_uri: target
                    .get("targetStyleUri")
                    .and_then(Value::as_str)
                    .map(str::to_string),
                prefix: target.get("prefix")?.as_str()?.to_string(),
                suffix: target.get("suffix")?.as_str()?.to_string(),
            })
        })
        .collect()
}

fn type_fact_provider_unavailable_from_value(
    value: &Value,
) -> Option<Vec<OmenaQuerySourceTypeFactProviderUnavailableFactV0>> {
    value
        .as_array()?
        .iter()
        .map(|fact| {
            Some(OmenaQuerySourceTypeFactProviderUnavailableFactV0 {
                byte_span: byte_span_from_value(fact.get("byteSpan")?)?,
                expression_id: fact.get("expressionId")?.as_str()?.to_string(),
                target_style_uri: fact
                    .get("targetStyleUri")
                    .and_then(Value::as_str)
                    .map(str::to_string),
                provider_id: provider_id_from_value(fact.get("providerId")?)?,
                reason: type_fact_provider_unavailable_reason_from_value(fact.get("reason")?)?,
            })
        })
        .collect()
}

fn provider_id_from_value(value: &Value) -> Option<&'static str> {
    match value.as_str()? {
        "tsgo" => Some("tsgo"),
        _ => None,
    }
}

fn type_fact_provider_unavailable_reason_from_value(value: &Value) -> Option<&'static str> {
    match value.as_str()? {
        "projectMiss" => Some("projectMiss"),
        "noTransport" => Some("noTransport"),
        "processUnavailable" => Some("processUnavailable"),
        "requestFailed" => Some("requestFailed"),
        "missingResult" => Some("missingResult"),
        "unresolvable" => Some("unresolvable"),
        _ => None,
    }
}

fn class_value_universes_from_value(
    value: &Value,
) -> Option<Vec<OmenaQuerySourceClassValueUniverseEntryV0>> {
    value
        .as_array()?
        .iter()
        .map(|universe| {
            Some(OmenaQuerySourceClassValueUniverseEntryV0 {
                plugin_id: recipe_plugin_id_from_value(universe.get("pluginId")?)?,
                domain: recipe_domain_from_value(universe.get("domain")?)?,
                owner_name: universe.get("ownerName")?.as_str()?.to_string(),
                class_names: strings_from_value(universe.get("classNames")?)?,
                axes: class_value_universe_axes_from_value(universe.get("axes")?)?,
                byte_span: byte_span_from_value(universe.get("byteSpan")?)?,
            })
        })
        .collect()
}

fn class_value_universe_axes_from_value(
    value: &Value,
) -> Option<Vec<OmenaQuerySourceClassValueUniverseAxisV0>> {
    value
        .as_array()?
        .iter()
        .map(|axis| {
            Some(OmenaQuerySourceClassValueUniverseAxisV0 {
                axis_name: axis.get("axisName")?.as_str()?.to_string(),
                values: strings_from_value(axis.get("values")?)?,
            })
        })
        .collect()
}

fn domain_class_references_from_value(
    value: &Value,
) -> Option<Vec<OmenaQuerySourceDomainClassReferenceFactV0>> {
    value
        .as_array()?
        .iter()
        .map(|reference| {
            Some(OmenaQuerySourceDomainClassReferenceFactV0 {
                byte_span: byte_span_from_value(reference.get("byteSpan")?)?,
                plugin_id: recipe_plugin_id_from_value(reference.get("pluginId")?)?,
                domain: recipe_domain_from_value(reference.get("domain")?)?,
                owner_name: reference.get("ownerName")?.as_str()?.to_string(),
                axis_name: reference.get("axisName")?.as_str()?.to_string(),
                option_name: reference
                    .get("optionName")
                    .and_then(Value::as_str)
                    .map(str::to_string),
                prefix: reference
                    .get("prefix")
                    .and_then(Value::as_str)
                    .map(str::to_string),
            })
        })
        .collect()
}

fn byte_span_from_value(value: &Value) -> Option<ParserByteSpanV0> {
    Some(ParserByteSpanV0 {
        start: value.get("start")?.as_u64()? as usize,
        end: value.get("end")?.as_u64()? as usize,
    })
}

fn optional_byte_span_from_value(value: Option<&Value>) -> Option<Option<ParserByteSpanV0>> {
    match value {
        Some(Value::Null) | None => Some(None),
        Some(value) => byte_span_from_value(value).map(Some),
    }
}

fn selector_match_kind_from_value(
    value: &Value,
) -> Option<OmenaQuerySourceSelectorReferenceMatchKindV0> {
    match value.as_str()? {
        "exact" | "Exact" => Some(OmenaQuerySourceSelectorReferenceMatchKindV0::Exact),
        "prefix" | "Prefix" => Some(OmenaQuerySourceSelectorReferenceMatchKindV0::Prefix),
        _ => None,
    }
}

fn strings_from_value(value: &Value) -> Option<Vec<String>> {
    value
        .as_array()?
        .iter()
        .map(|item| item.as_str().map(str::to_string))
        .collect()
}

fn cascade_tier_from_value(value: &Value) -> Option<&'static str> {
    match value.as_str()? {
        "authorInlineStyle" => Some("authorInlineStyle"),
        _ => None,
    }
}

fn recipe_plugin_id_from_value(value: &Value) -> Option<&'static str> {
    match value.as_str()? {
        "cva-recipe-domain" => Some("cva-recipe-domain"),
        "vanilla-extract-recipe-domain" => Some("vanilla-extract-recipe-domain"),
        _ => None,
    }
}

fn recipe_domain_from_value(value: &Value) -> Option<&'static str> {
    match value.as_str()? {
        "cva-recipe" => Some("cva-recipe"),
        "vanilla-extract-recipe" => Some("vanilla-extract-recipe"),
        _ => None,
    }
}
