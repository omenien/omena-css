use crate::cache_limits::{
    DEFAULT_PERSISTENT_CACHE_LIMITS, PersistentCacheLimitsV0, ensure_cache_root_attribution,
    read_cache_shard_with_limits, write_cache_shard_atomically_with_limits,
};
use crate::cache_root::LspCacheStorageConfigV0;
use crate::protocol::file_uri_to_path;
use omena_query::{OmenaQueryStyleResolutionInputsV0, OmenaWorkspaceOccurrenceV0};
use omena_sif::{compute_omena_sif_leaf_hash_v1, write_omena_canonical_json_bytes_v1};
use serde::Serialize;
use serde_json::{Value, json};
#[cfg(test)]
use std::{collections::BTreeMap, fs};
use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

const WORKSPACE_OCCURRENCE_SHARD_SCHEMA_VERSION: &str = "2";
const WORKSPACE_OCCURRENCE_SHARD_PRODUCT: &str = "omena-lsp-server.workspace-occurrence-shard";
const WORKSPACE_OCCURRENCE_SHARD_KEY_PRODUCT: &str =
    "omena-lsp-server.workspace-occurrence-shard-key";
const WORKSPACE_OCCURRENCE_SHARD_DIR: &str = "workspace-occurrence-shards-v2";
const WORKSPACE_OCCURRENCE_SHARD_LIMITS: PersistentCacheLimitsV0 = DEFAULT_PERSISTENT_CACHE_LIMITS;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct WorkspaceOccurrenceShardKeyInputV0<'a> {
    schema_version: &'a str,
    crate_version: &'a str,
    product: &'a str,
    document_workspace_folder_uri: Option<&'a str>,
    workspace_folder_uri: Option<&'a str>,
    document_uri: &'a str,
    language_id: &'a str,
    text_hash: &'a str,
    dependency_digest: Option<&'a str>,
    resolution_inputs: &'a OmenaQueryStyleResolutionInputsV0,
}

#[derive(Debug, Clone)]
pub(crate) struct LspWorkspaceOccurrenceShardLoadV0 {
    pub(crate) key: String,
    pub(crate) occurrences: Vec<OmenaWorkspaceOccurrenceV0>,
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn load_workspace_occurrence_shard(
    cache_storage: &LspCacheStorageConfigV0,
    document_workspace_folder_uri: Option<&str>,
    workspace_folder_uri: Option<&str>,
    document_uri: &str,
    language_id: &str,
    text_hash: &str,
    dependency_digest: Option<&str>,
    resolution_inputs: &OmenaQueryStyleResolutionInputsV0,
) -> Option<LspWorkspaceOccurrenceShardLoadV0> {
    #[cfg(test)]
    record_workspace_occurrence_shard_read(document_uri);
    let shard_workspace_folder_uri =
        workspace_occurrence_shard_key_workspace_folder_uri(workspace_folder_uri);
    let key = workspace_occurrence_shard_key(
        document_workspace_folder_uri,
        workspace_folder_uri,
        document_uri,
        language_id,
        text_hash,
        dependency_digest,
        resolution_inputs,
    )?;
    let path = workspace_occurrence_shard_path(
        cache_storage,
        document_workspace_folder_uri,
        document_uri,
        language_id,
    )?;
    let bytes =
        read_workspace_occurrence_shard(path.as_path(), &WORKSPACE_OCCURRENCE_SHARD_LIMITS)?;
    let shard: Value = serde_json::from_slice(bytes.as_slice()).ok()?;
    if shard.pointer("/schemaVersion").and_then(Value::as_str)
        != Some(WORKSPACE_OCCURRENCE_SHARD_SCHEMA_VERSION)
        || shard.pointer("/product").and_then(Value::as_str)
            != Some(WORKSPACE_OCCURRENCE_SHARD_PRODUCT)
        || shard.pointer("/key").and_then(Value::as_str) != Some(key.as_str())
        || shard.pointer("/documentUri").and_then(Value::as_str) != Some(document_uri)
        || shard
            .pointer("/documentWorkspaceFolderUri")
            .and_then(Value::as_str)
            != document_workspace_folder_uri
        || shard.pointer("/workspaceFolderUri").and_then(Value::as_str)
            != shard_workspace_folder_uri
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
    Some(LspWorkspaceOccurrenceShardLoadV0 { key, occurrences })
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn store_workspace_occurrence_shard(
    cache_storage: &LspCacheStorageConfigV0,
    document_workspace_folder_uri: Option<&str>,
    workspace_folder_uri: Option<&str>,
    document_uri: &str,
    language_id: &str,
    text_hash: &str,
    dependency_digest: Option<&str>,
    resolution_inputs: &OmenaQueryStyleResolutionInputsV0,
    occurrences: &[OmenaWorkspaceOccurrenceV0],
) {
    let shard_workspace_folder_uri =
        workspace_occurrence_shard_key_workspace_folder_uri(workspace_folder_uri);
    let Some(key) = workspace_occurrence_shard_key(
        document_workspace_folder_uri,
        workspace_folder_uri,
        document_uri,
        language_id,
        text_hash,
        dependency_digest,
        resolution_inputs,
    ) else {
        return;
    };
    let Some(path) = workspace_occurrence_shard_path(
        cache_storage,
        document_workspace_folder_uri,
        document_uri,
        language_id,
    ) else {
        return;
    };
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
        "documentWorkspaceFolderUri": document_workspace_folder_uri,
        "workspaceFolderUri": shard_workspace_folder_uri,
        "languageId": language_id,
        "textHash": text_hash,
        "containsText": false,
        "payloadDigest": payload_digest,
        "payload": payload,
    });
    let Ok(bytes) = serde_json::to_vec(&shard) else {
        return;
    };
    let Some(dir) = path.parent() else {
        return;
    };
    if std::fs::create_dir_all(dir).is_err() {
        return;
    }
    crate::disk_cache::ensure_omena_cache_root_markers(dir);
    ensure_cache_root_attribution(dir, document_workspace_folder_uri.unwrap_or(document_uri));
    let _ = write_workspace_occurrence_shard(
        path.as_path(),
        bytes.as_slice(),
        &WORKSPACE_OCCURRENCE_SHARD_LIMITS,
    );
}

fn read_workspace_occurrence_shard(
    path: &Path,
    limits: &PersistentCacheLimitsV0,
) -> Option<Vec<u8>> {
    read_cache_shard_with_limits(path, limits)
}

fn write_workspace_occurrence_shard(
    path: &Path,
    bytes: &[u8],
    limits: &PersistentCacheLimitsV0,
) -> bool {
    write_cache_shard_atomically_with_limits(path, bytes, limits)
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
    document_workspace_folder_uri: Option<&str>,
    workspace_folder_uri: Option<&str>,
    document_uri: &str,
    language_id: &str,
    text_hash: &str,
    dependency_digest: Option<&str>,
    resolution_inputs: &OmenaQueryStyleResolutionInputsV0,
) -> Option<String> {
    let dependency_digest = workspace_occurrence_shard_key_dependency_digest(dependency_digest);
    let workspace_folder_uri =
        workspace_occurrence_shard_key_workspace_folder_uri(workspace_folder_uri);
    let input = WorkspaceOccurrenceShardKeyInputV0 {
        schema_version: WORKSPACE_OCCURRENCE_SHARD_SCHEMA_VERSION,
        crate_version: env!("CARGO_PKG_VERSION"),
        product: WORKSPACE_OCCURRENCE_SHARD_KEY_PRODUCT,
        document_workspace_folder_uri,
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

#[cfg(test)]
pub(crate) fn workspace_occurrence_shard_key_for_test(
    document_workspace_folder_uri: Option<&str>,
    workspace_folder_uri: Option<&str>,
    document_uri: &str,
    language_id: &str,
    text_hash: &str,
    dependency_digest: Option<&str>,
    resolution_inputs: &OmenaQueryStyleResolutionInputsV0,
) -> Option<String> {
    workspace_occurrence_shard_key(
        document_workspace_folder_uri,
        workspace_folder_uri,
        document_uri,
        language_id,
        text_hash,
        dependency_digest,
        resolution_inputs,
    )
}

fn workspace_occurrence_shard_key_workspace_folder_uri(
    workspace_folder_uri: Option<&str>,
) -> Option<&str> {
    #[cfg(test)]
    if WORKSPACE_OCCURRENCE_KEY_DROP_WORKSPACE_FOLDER_URI.with(std::cell::Cell::get) {
        return None;
    }
    workspace_folder_uri
}

fn workspace_occurrence_shard_key_dependency_digest(
    dependency_digest: Option<&str>,
) -> Option<&str> {
    #[cfg(test)]
    if WORKSPACE_OCCURRENCE_KEY_DROP_DEPENDENCY.with(std::cell::Cell::get) {
        return None;
    }
    dependency_digest
}

pub(crate) fn workspace_occurrence_shard_should_shadow(key: &str) -> bool {
    #[cfg(test)]
    if WORKSPACE_OCCURRENCE_SHADOW_NONE_FOR_TEST.with(std::cell::Cell::get) {
        return false;
    }
    #[cfg(test)]
    if WORKSPACE_OCCURRENCE_SHADOW_ALL_FOR_TEST.with(std::cell::Cell::get) {
        return true;
    }
    workspace_occurrence_shadow_sample_nibble(key).is_some_and(|nibble| nibble == 0)
}

pub(crate) fn workspace_occurrence_shadow_sample_nibble(key: &str) -> Option<u32> {
    key.as_bytes()
        .last()
        .and_then(|byte| (*byte as char).to_digit(16))
}

pub(crate) fn workspace_occurrence_shadow_asserts_on_mismatch() -> bool {
    #[cfg(test)]
    {
        !WORKSPACE_OCCURRENCE_SHADOW_RECOVERY_FOR_TEST.with(std::cell::Cell::get)
    }
    #[cfg(not(test))]
    {
        false
    }
}

pub(crate) fn record_workspace_occurrence_shadow_mismatch(
    mismatch_count: &AtomicU64,
    document_uri: &str,
) {
    mismatch_count.fetch_add(1, Ordering::Relaxed);
    crate::loop_trace!("workspace-occurrence-shadow MISMATCH target={document_uri}");
}

#[cfg(test)]
thread_local! {
    static WORKSPACE_OCCURRENCE_KEY_DROP_DEPENDENCY: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    static WORKSPACE_OCCURRENCE_KEY_DROP_WORKSPACE_FOLDER_URI: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    static WORKSPACE_OCCURRENCE_SHARD_READS: std::cell::RefCell<BTreeMap<String, u64>> = const { std::cell::RefCell::new(BTreeMap::new()) };
    static WORKSPACE_OCCURRENCE_SHADOW_RECOVERY_FOR_TEST: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    static WORKSPACE_OCCURRENCE_SHADOW_ALL_FOR_TEST: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    static WORKSPACE_OCCURRENCE_SHADOW_NONE_FOR_TEST: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

#[cfg(test)]
fn record_workspace_occurrence_shard_read(document_uri: &str) {
    WORKSPACE_OCCURRENCE_SHARD_READS.with(|reads| {
        *reads
            .borrow_mut()
            .entry(document_uri.to_string())
            .or_default() += 1;
    });
}

#[cfg(test)]
pub(crate) fn reset_workspace_occurrence_shard_read_counts_for_test() {
    WORKSPACE_OCCURRENCE_SHARD_READS.with(|reads| reads.borrow_mut().clear());
}

#[cfg(test)]
pub(crate) fn workspace_occurrence_shard_read_count_for_test(document_uri: &str) -> u64 {
    WORKSPACE_OCCURRENCE_SHARD_READS.with(|reads| {
        reads
            .borrow()
            .get(document_uri)
            .copied()
            .unwrap_or_default()
    })
}

#[cfg(test)]
pub(crate) fn with_workspace_occurrence_shadow_recovery_for_test<R>(body: impl FnOnce() -> R) -> R {
    WORKSPACE_OCCURRENCE_SHADOW_RECOVERY_FOR_TEST.with(|recovery| {
        let previous = recovery.replace(true);
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(body));
        recovery.set(previous);
        match result {
            Ok(result) => result,
            Err(payload) => std::panic::resume_unwind(payload),
        }
    })
}

#[cfg(test)]
pub(crate) fn with_workspace_occurrence_shadow_all_for_test<R>(body: impl FnOnce() -> R) -> R {
    WORKSPACE_OCCURRENCE_SHADOW_ALL_FOR_TEST.with(|shadow_all| {
        let previous = shadow_all.replace(true);
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(body));
        shadow_all.set(previous);
        match result {
            Ok(result) => result,
            Err(payload) => std::panic::resume_unwind(payload),
        }
    })
}

#[cfg(test)]
pub(crate) fn with_workspace_occurrence_shadow_none_for_test<R>(body: impl FnOnce() -> R) -> R {
    WORKSPACE_OCCURRENCE_SHADOW_NONE_FOR_TEST.with(|shadow_none| {
        let previous = shadow_none.replace(true);
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(body));
        shadow_none.set(previous);
        match result {
            Ok(result) => result,
            Err(payload) => std::panic::resume_unwind(payload),
        }
    })
}

#[cfg(test)]
pub(crate) fn with_workspace_occurrence_key_dependency_drop_for_test<R>(
    body: impl FnOnce() -> R,
) -> R {
    WORKSPACE_OCCURRENCE_KEY_DROP_DEPENDENCY.with(|drop_dependency| {
        let previous = drop_dependency.replace(true);
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(body));
        drop_dependency.set(previous);
        match result {
            Ok(result) => result,
            Err(payload) => std::panic::resume_unwind(payload),
        }
    })
}

#[cfg(test)]
pub(crate) fn with_workspace_occurrence_key_workspace_folder_uri_drop_for_test<R>(
    body: impl FnOnce() -> R,
) -> R {
    WORKSPACE_OCCURRENCE_KEY_DROP_WORKSPACE_FOLDER_URI.with(|drop_workspace_folder_uri| {
        let previous = drop_workspace_folder_uri.replace(true);
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(body));
        drop_workspace_folder_uri.set(previous);
        match result {
            Ok(result) => result,
            Err(payload) => std::panic::resume_unwind(payload),
        }
    })
}

pub(crate) fn evict_workspace_occurrence_shard(
    cache_storage: &LspCacheStorageConfigV0,
    document_workspace_folder_uri: Option<&str>,
    document_uri: &str,
    language_id: &str,
) {
    let Some(path) = workspace_occurrence_shard_path(
        cache_storage,
        document_workspace_folder_uri,
        document_uri,
        language_id,
    ) else {
        return;
    };
    let _ = std::fs::remove_file(path);
}

fn workspace_occurrence_shard_path(
    cache_storage: &LspCacheStorageConfigV0,
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
    crate::cache_root::resolved_workspace_cache_dir(
        cache_storage,
        workspace_folder_uri,
        root.as_path(),
        WORKSPACE_OCCURRENCE_SHARD_DIR,
    )
    .map(|dir| dir.join(format!("{hex}.json")))
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
    fn workspace_occurrence_shadow_sampler_pins_one_of_sixteen_nibbles() {
        let sampled = "0123456789abcdef"
            .chars()
            .filter(|nibble| {
                workspace_occurrence_shard_should_shadow(format!("blake3:key{nibble}").as_str())
            })
            .collect::<String>();
        assert_eq!(
            sampled, "0",
            "the production sampler must stay exactly 1/16"
        );
        assert_eq!(workspace_occurrence_shadow_sample_nibble("not-a-key"), None);
        assert!(with_workspace_occurrence_shadow_none_for_test(|| {
            !workspace_occurrence_shard_should_shadow("blake3:key0")
        }));
    }

    #[test]
    fn workspace_occurrence_key_binds_the_extractor_workspace_scope() {
        let resolution_inputs = OmenaQueryStyleResolutionInputsV0::default();
        let scoped = workspace_occurrence_shard_key(
            Some("file:///workspace"),
            Some("file:///workspace"),
            "file:///workspace/src/App.module.scss",
            "scss",
            "blake3:text",
            Some("blake3:read-set"),
            &resolution_inputs,
        );
        let all_workspaces = workspace_occurrence_shard_key(
            Some("file:///workspace"),
            None,
            "file:///workspace/src/App.module.scss",
            "scss",
            "blake3:text",
            Some("blake3:read-set"),
            &resolution_inputs,
        );
        assert!(scoped.is_some());
        assert!(all_workspaces.is_some());
        assert_ne!(scoped, all_workspaces);
        let dropped_scoped =
            with_workspace_occurrence_key_workspace_folder_uri_drop_for_test(|| {
                workspace_occurrence_shard_key(
                    Some("file:///workspace"),
                    Some("file:///workspace"),
                    "file:///workspace/src/App.module.scss",
                    "scss",
                    "blake3:text",
                    Some("blake3:read-set"),
                    &resolution_inputs,
                )
            });
        let dropped_all = with_workspace_occurrence_key_workspace_folder_uri_drop_for_test(|| {
            workspace_occurrence_shard_key(
                Some("file:///workspace"),
                None,
                "file:///workspace/src/App.module.scss",
                "scss",
                "blake3:text",
                Some("blake3:read-set"),
                &resolution_inputs,
            )
        });
        assert_eq!(
            dropped_scoped, dropped_all,
            "the seeded drop must collapse exactly the workspace-scope key distinction",
        );
    }

    #[test]
    fn workspace_occurrence_store_enforces_reachable_count_byte_and_shard_limits() {
        let root = unique_temp_root("omena_workspace_occurrence_store_limits")
            .unwrap_or_else(|_| std::env::temp_dir().join("omena-workspace-occurrence-limits"));
        let workspace_uri = path_to_file_uri(root.as_path());
        let editor_workspace_storage = root.join("editor-storage").join("workspace");
        let cache_storage = LspCacheStorageConfigV0 {
            initialization_global_storage: Some(root.join("editor-storage").join("global")),
            initialization_workspace_storage: Some(editor_workspace_storage.clone()),
            location: crate::cache_root::CacheLocationV0::Editor,
            ..LspCacheStorageConfigV0::default()
        };
        let resolution_inputs = OmenaQueryStyleResolutionInputsV0::default();
        let default_document_uri = path_to_file_uri(root.join("src/default-limit.tsx").as_path());
        let default_path = workspace_occurrence_shard_path(
            &cache_storage,
            Some(workspace_uri.as_str()),
            default_document_uri.as_str(),
            "typescriptreact",
        );
        assert!(default_path.is_some(), "workspace-occurrence default path");
        let Some(default_path) = default_path else {
            return;
        };
        assert!(
            default_path.starts_with(editor_workspace_storage.as_path()),
            "workspace-occurrence production cap exercise must use the resolved editor root"
        );
        if let Some(parent) = default_path.parent() {
            assert!(fs::create_dir_all(parent).is_ok());
        }
        let oversized_text_hash = "x".repeat(
            usize::try_from(WORKSPACE_OCCURRENCE_SHARD_LIMITS.max_shard_bytes)
                .unwrap_or(8 * 1024 * 1024)
                + 1,
        );
        store_workspace_occurrence_shard(
            &cache_storage,
            Some(workspace_uri.as_str()),
            Some(workspace_uri.as_str()),
            default_document_uri.as_str(),
            "typescriptreact",
            oversized_text_hash.as_str(),
            None,
            &resolution_inputs,
            &[],
        );
        assert!(
            !default_path.exists(),
            "workspace-occurrence default max-shard constant must be reachable through the real store"
        );
        let cache_dir = default_path.parent();
        assert!(
            cache_dir.is_some(),
            "workspace-occurrence default cache dir"
        );
        if let Some(cache_dir) = cache_dir {
            crate::cache_limits::assert_production_store_enforces_default_count_and_total(
                "workspace-occurrence-shard",
                cache_dir,
                default_path.as_path(),
                || {
                    store_workspace_occurrence_shard(
                        &cache_storage,
                        Some(workspace_uri.as_str()),
                        Some(workspace_uri.as_str()),
                        default_document_uri.as_str(),
                        "typescriptreact",
                        "blake3:default-cap-fixture",
                        None,
                        &resolution_inputs,
                        &[],
                    );
                },
            );
        }

        crate::cache_limits::assert_real_cache_store_enforces_reachable_limits(
            "workspace-occurrence-shard",
            write_workspace_occurrence_shard,
            read_workspace_occurrence_shard,
        );
        eprintln!(
            "storeEntryCaps cache=workspace-occurrence-shard resolvedEditorRoot=true defaultCount=true defaultTotalBytes=true defaultMaxShardRefused=true lowLevelCount=true lowLevelTotalBytes=true lowLevelShardBytes=true"
        );
        let _ = fs::remove_dir_all(root);
    }

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
        let cache_storage = LspCacheStorageConfigV0::default();

        store_workspace_occurrence_shard(
            &cache_storage,
            Some(workspace_uri.as_str()),
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
            &cache_storage,
            Some(workspace_uri.as_str()),
            document_uri.as_str(),
            "typescriptreact",
        )
        .ok_or("missing workspace occurrence shard path")?;
        let first_bytes = fs::read(shard_path.as_path())?;
        let first_json: Value = serde_json::from_slice(first_bytes.as_slice())?;
        let cache_root = shard_path
            .parent()
            .and_then(Path::parent)
            .ok_or("missing workspace occurrence cache root")?;
        assert!(
            cache_root.join(".gitignore").is_file()
                && cache_root.join("CACHEDIR.TAG").is_file()
                && cache_root.join(".omena-cache-owner.json").is_file(),
            "cache ownership and self-ignore markers must precede persistent shard writes"
        );

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
            &cache_storage,
            Some(workspace_uri.as_str()),
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
            &cache_storage,
            Some(workspace_uri.as_str()),
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

    #[test]
    fn workspace_occurrence_v1_shard_cannot_collide_with_v2_namespace() -> Result<(), Box<dyn Error>>
    {
        let root = unique_temp_root("omena_workspace_occurrence_v1_noncollision")?;
        let workspace_uri = path_to_file_uri(root.as_path());
        let document_uri = path_to_file_uri(root.join("src/App.module.scss").as_path());
        let cache_storage = LspCacheStorageConfigV0::default();
        let resolution_inputs = OmenaQueryStyleResolutionInputsV0::default();
        let v2_path = workspace_occurrence_shard_path(
            &cache_storage,
            Some(workspace_uri.as_str()),
            document_uri.as_str(),
            "scss",
        )
        .ok_or("missing v2 workspace occurrence path")?;
        let file_name = v2_path
            .file_name()
            .ok_or("missing workspace occurrence shard name")?;
        let v1_path = v2_path
            .parent()
            .and_then(Path::parent)
            .ok_or("missing workspace occurrence cache root")?
            .join("workspace-occurrence-shards-v1")
            .join(file_name);
        fs::create_dir_all(v1_path.parent().ok_or("missing v1 parent")?)?;
        fs::write(v1_path.as_path(), br#"{"schemaVersion":"0"}"#)?;

        assert!(
            load_workspace_occurrence_shard(
                &cache_storage,
                Some(workspace_uri.as_str()),
                Some(workspace_uri.as_str()),
                document_uri.as_str(),
                "scss",
                "blake3:text",
                Some("blake3:read-set"),
                &resolution_inputs,
            )
            .is_none(),
            "a planted v1 shard must not be visible through the v2 loader"
        );
        assert!(v1_path.is_file());
        assert!(!v2_path.exists());

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
