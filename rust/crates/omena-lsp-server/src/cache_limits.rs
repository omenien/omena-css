use std::{fs, path::Path};

const CACHE_ATTRIBUTION_FILE: &str = ".omena-cache-owner.json";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PersistentCacheLimitsV0 {
    pub(crate) max_shards: usize,
    pub(crate) max_total_bytes: u64,
    pub(crate) max_shard_bytes: u64,
}

pub(crate) const DEFAULT_PERSISTENT_CACHE_LIMITS: PersistentCacheLimitsV0 =
    PersistentCacheLimitsV0 {
        max_shards: 4096,
        max_total_bytes: 256 * 1024 * 1024,
        max_shard_bytes: 8 * 1024 * 1024,
    };

#[expect(
    clippy::disallowed_methods,
    reason = "persistent-cache owner: remove oversized shards at read boundary"
)]
pub(crate) fn read_cache_shard_with_limits(
    path: &Path,
    limits: &PersistentCacheLimitsV0,
) -> Option<Vec<u8>> {
    let metadata = fs::metadata(path).ok()?;
    if !metadata.is_file() || metadata.len() > limits.max_shard_bytes {
        let _ = fs::remove_file(path);
        return None;
    }
    fs::read(path).ok()
}

#[expect(
    clippy::disallowed_methods,
    reason = "persistent-cache owner: retain atomic bounded shard publication"
)]
pub(crate) fn write_cache_shard_atomically_with_limits(
    path: &Path,
    bytes: &[u8],
    limits: &PersistentCacheLimitsV0,
) -> bool {
    if bytes.len() as u64 > limits.max_shard_bytes {
        return false;
    }
    let Some(dir) = path.parent() else {
        return false;
    };
    if fs::create_dir_all(dir).is_err() {
        return false;
    }
    let temporary_path = path.with_extension(format!("tmp-{}", std::process::id()));
    if fs::write(temporary_path.as_path(), bytes).is_err() {
        return false;
    }
    let renamed = fs::rename(temporary_path.as_path(), path).is_ok();
    if !renamed {
        let _ = fs::remove_file(temporary_path);
    }
    if renamed {
        enforce_cache_limits(dir, limits);
    }
    renamed
}

#[expect(
    clippy::disallowed_methods,
    reason = "persistent-cache owner: retain bounded shard eviction"
)]
fn enforce_cache_limits(dir: &Path, limits: &PersistentCacheLimitsV0) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    let mut shards = entries
        .flatten()
        .filter_map(|entry| {
            let path = entry.path();
            if path.extension().and_then(|extension| extension.to_str()) != Some("json") {
                return None;
            }
            let metadata = entry.metadata().ok()?;
            if !metadata.is_file() {
                return None;
            }
            let modified = metadata.modified().ok()?;
            Some((modified, metadata.len(), path))
        })
        .collect::<Vec<_>>();
    shards.sort_by(|left, right| left.0.cmp(&right.0).then_with(|| left.2.cmp(&right.2)));
    let mut shard_count = shards.len();
    let mut total_bytes = shards.iter().map(|(_, bytes, _)| *bytes).sum::<u64>();
    for (_, bytes, path) in shards {
        if shard_count <= limits.max_shards && total_bytes <= limits.max_total_bytes {
            break;
        }
        if fs::remove_file(path).is_ok() {
            shard_count = shard_count.saturating_sub(1);
            total_bytes = total_bytes.saturating_sub(bytes);
        }
    }
}

#[expect(
    clippy::disallowed_methods,
    reason = "persistent-cache owner: retain attribution publication"
)]
pub(crate) fn ensure_cache_root_attribution(cache_subdir: &Path, workspace_identity: &str) {
    let Some(cache_root) = cache_subdir.parent() else {
        return;
    };
    let attribution_path = cache_root.join(CACHE_ATTRIBUTION_FILE);
    let value = serde_json::json!({
        "schemaVersion": "0",
        "product": "omena.cache-root-attribution",
        "workspaceIdentity": workspace_identity,
    });
    let Ok(bytes) = serde_json::to_vec(&value) else {
        return;
    };
    let _ = fs::write(attribution_path, bytes);
}

#[cfg(test)]
pub(crate) fn assert_real_cache_store_enforces_reachable_limits(
    cache_name: &str,
    write: impl Fn(&Path, &[u8], &PersistentCacheLimitsV0) -> bool,
    read: impl Fn(&Path, &PersistentCacheLimitsV0) -> Option<Vec<u8>>,
) {
    use std::time::{SystemTime, UNIX_EPOCH};

    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let editor_storage_root = std::env::temp_dir().join(format!(
        "omena-cache-limits-{}-{}-{nonce}",
        cache_name.replace('/', "-"),
        std::process::id()
    ));
    let cache_storage = crate::cache_root::LspCacheStorageConfigV0 {
        initialization_global_storage: Some(editor_storage_root.join("global")),
        initialization_workspace_storage: Some(editor_storage_root.join("workspace")),
        log_path: Some(editor_storage_root.join("logs")),
        command_cache_dir: None,
        location: crate::cache_root::CacheLocationV0::Editor,
    };
    let resolved_root = crate::cache_root::resolved_workspace_cache_dir(
        &cache_storage,
        "file:///workspace",
        Path::new("/workspace"),
        cache_name,
    );
    assert!(resolved_root.is_some(), "editor cache root must resolve");
    let Some(root) = resolved_root else {
        return;
    };
    assert!(
        root.starts_with(editor_storage_root.join("workspace")),
        "{cache_name}: cap exercise must run under the resolved editor root: {root:?}"
    );

    let count_dir = root.join("count");
    let count_limits = PersistentCacheLimitsV0 {
        max_shards: 2,
        max_total_bytes: 1024,
        max_shard_bytes: 128,
    };
    let count_victim = count_dir.join("00-victim.json");
    let count_survivor = count_dir.join("10-survivor.json");
    let count_newest = count_dir.join("20-newest.json");
    assert!(write(&count_victim, &[b'v'; 8], &count_limits));
    assert!(write(&count_survivor, &[b's'; 8], &count_limits));
    assert!(write(&count_newest, &[b'n'; 8], &count_limits));
    assert!(
        !count_victim.exists(),
        "{cache_name}: count victim must be evicted"
    );
    assert!(count_survivor.exists(), "{cache_name}: survivor missing");
    assert!(count_newest.exists(), "{cache_name}: newest missing");

    let bytes_dir = root.join("bytes");
    let byte_limits = PersistentCacheLimitsV0 {
        max_shards: 10,
        max_total_bytes: 16,
        max_shard_bytes: 128,
    };
    let byte_victim = bytes_dir.join("00-victim.json");
    let byte_survivor = bytes_dir.join("10-survivor.json");
    let byte_newest = bytes_dir.join("20-newest.json");
    assert!(write(&byte_victim, &[b'v'; 8], &byte_limits));
    assert!(write(&byte_survivor, &[b's'; 8], &byte_limits));
    assert!(write(&byte_newest, &[b'n'; 8], &byte_limits));
    assert!(
        !byte_victim.exists(),
        "{cache_name}: byte victim must be evicted"
    );
    assert!(
        byte_survivor.exists(),
        "{cache_name}: byte survivor missing"
    );
    assert!(byte_newest.exists(), "{cache_name}: byte newest missing");

    let oversize_limits = PersistentCacheLimitsV0 {
        max_shards: 10,
        max_total_bytes: 1024,
        max_shard_bytes: 48,
    };
    let oversize_dir = root.join("oversize");
    let oversize = oversize_dir.join("oversize.json");
    assert!(
        !write(&oversize, &[b'x'; 49], &oversize_limits),
        "{cache_name}: oversize write must be refused"
    );
    assert!(
        !oversize.exists(),
        "{cache_name}: oversize shard was written"
    );
    assert!(fs::create_dir_all(oversize_dir).is_ok(), "oversize dir");
    assert!(
        fs::write(oversize.as_path(), [b'x'; 49]).is_ok(),
        "oversize fixture"
    );
    assert!(
        read(oversize.as_path(), &oversize_limits).is_none(),
        "{cache_name}: oversize shard must not be served"
    );
    assert!(
        !oversize.exists(),
        "{cache_name}: refused oversize shard must be removed"
    );

    eprintln!(
        "cacheLimit cache={cache_name} root={} countVictimExists={} countSurvivors=2 byteVictimExists={} byteSurvivors=2 oversizeExists={}",
        root.display(),
        count_victim.exists(),
        byte_victim.exists(),
        oversize.exists(),
    );

    let _ = fs::remove_dir_all(editor_storage_root);
}

#[cfg(test)]
pub(crate) fn assert_production_store_enforces_default_count_and_total(
    cache_name: &str,
    cache_dir: &Path,
    new_shard_path: &Path,
    mut store: impl FnMut(),
) {
    assert_eq!(new_shard_path.parent(), Some(cache_dir));
    let _ = fs::remove_dir_all(cache_dir);
    assert!(fs::create_dir_all(cache_dir).is_ok());
    for ordinal in 0..DEFAULT_PERSISTENT_CACHE_LIMITS.max_shards {
        assert!(
            fs::write(cache_dir.join(format!("fixture-{ordinal:04}.json")), b"{}").is_ok(),
            "{cache_name}: default count fixture {ordinal}"
        );
    }
    store();
    assert!(
        new_shard_path.is_file(),
        "{cache_name}: production store must write under the default count cap"
    );
    let count_after_store = fs::read_dir(cache_dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter(|entry| entry.path().extension().and_then(|value| value.to_str()) == Some("json"))
        .count();
    assert_eq!(
        count_after_store, DEFAULT_PERSISTENT_CACHE_LIMITS.max_shards,
        "{cache_name}: production store must enforce the default shard-count cap"
    );

    let _ = fs::remove_dir_all(cache_dir);
    assert!(fs::create_dir_all(cache_dir).is_ok());
    let total_victim = cache_dir.join("00-total-victim.json");
    let total_fixture = fs::File::create(total_victim.as_path());
    assert!(total_fixture.is_ok(), "{cache_name}: total-byte fixture");
    if let Ok(total_fixture) = total_fixture {
        assert!(
            total_fixture
                .set_len(DEFAULT_PERSISTENT_CACHE_LIMITS.max_total_bytes)
                .is_ok(),
            "{cache_name}: sparse total-byte fixture"
        );
    }
    store();
    assert!(
        new_shard_path.is_file(),
        "{cache_name}: production store must retain the newest shard under total-byte pressure"
    );
    let total_bytes_after_store = fs::read_dir(cache_dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|entry| entry.metadata().ok().map(|metadata| metadata.len()))
        .sum::<u64>();
    assert!(
        total_bytes_after_store <= DEFAULT_PERSISTENT_CACHE_LIMITS.max_total_bytes,
        "{cache_name}: production store must enforce the default total-byte cap"
    );
    assert!(
        !total_victim.exists(),
        "{cache_name}: the named sparse total-byte victim must be evicted"
    );
    eprintln!(
        "defaultStoreCaps cache={cache_name} countAfterStore={count_after_store} maxShards={} totalBytesAfterStore={total_bytes_after_store} maxTotalBytes={} totalVictimExists={}",
        DEFAULT_PERSISTENT_CACHE_LIMITS.max_shards,
        DEFAULT_PERSISTENT_CACHE_LIMITS.max_total_bytes,
        total_victim.exists(),
    );
    let _ = fs::remove_dir_all(cache_dir);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cache_root_attribution_stamp_names_the_workspace_owner()
    -> Result<(), Box<dyn std::error::Error>> {
        let root =
            std::env::temp_dir().join(format!("omena-cache-attribution-{}", std::process::id()));
        let cache_dir = root.join("source-document-index-v1");
        assert!(fs::create_dir_all(cache_dir.as_path()).is_ok());
        ensure_cache_root_attribution(cache_dir.as_path(), "file:///workspace");
        let bytes = fs::read(root.join(CACHE_ATTRIBUTION_FILE))?;
        let value = serde_json::from_slice::<serde_json::Value>(bytes.as_slice())?;
        assert_eq!(
            value
                .pointer("/workspaceIdentity")
                .and_then(serde_json::Value::as_str),
            Some("file:///workspace")
        );
        let _ = fs::remove_dir_all(root);
        Ok(())
    }

    #[test]
    fn synthetic_document_corpus_justifies_persistent_cache_limits()
    -> Result<(), Box<dyn std::error::Error>> {
        const DOCUMENT_COUNT: usize = 8;
        let products = [
            "source-document-index",
            "workspace-occurrence-shard",
            "source-type-fact-sidecar",
        ];
        for product in products {
            let mut sizes = Vec::new();
            for document_ordinal in 1..=DOCUMENT_COUNT {
                let repeated_entries = (0..document_ordinal)
                    .map(|entry_ordinal| {
                        serde_json::json!({
                            "documentUri": format!("file:///workspace/src/{document_ordinal}.tsx"),
                            "entryOrdinal": entry_ordinal,
                            "selector": format!("component-{document_ordinal}-{entry_ordinal}"),
                            "range": {"start": entry_ordinal * 8, "end": entry_ordinal * 8 + 7},
                        })
                    })
                    .collect::<Vec<_>>();
                let shard = serde_json::json!({
                    "schemaVersion": "1",
                    "product": product,
                    "workspaceFolderUri": "file:///workspace",
                    "documentOrdinal": document_ordinal,
                    "payload": {"entries": repeated_entries},
                });
                let bytes = serde_json::to_vec(&shard)?;
                sizes.push(bytes.len() as u64);
            }
            let min = sizes.iter().copied().min().unwrap_or(0);
            let max = sizes.iter().copied().max().unwrap_or(0);
            let total = sizes.iter().sum::<u64>();
            println!(
                "syntheticCorpus product={product} documents={DOCUMENT_COUNT} minBytes={min} maxBytes={max} totalBytes={total}"
            );
            assert!(max <= DEFAULT_PERSISTENT_CACHE_LIMITS.max_shard_bytes);
            assert!(
                max <= DEFAULT_PERSISTENT_CACHE_LIMITS.max_total_bytes
                    / DEFAULT_PERSISTENT_CACHE_LIMITS.max_shards as u64,
                "{product}: the synthetic maximum must fit the per-entry average budget at the count cap"
            );
        }
        Ok(())
    }
}
