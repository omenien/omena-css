//! RFC 0009 Pillar C (rfcs#66) stage 1: a persistent, content-addressed
//! workspace style-diagnostics shard store.
//!
//! This is deliberately NOT a serde shim over the salsa database. Each shard
//! is a standalone JSON file under
//! `<workspaceFolder>/.cache/omena/diagnostics-cache-v0/` named by the blake3
//! content hash of the FULL diagnostics input surface; a shard is served only
//! on an exact key match, which makes a hit byte-identical to a recompute by
//! construction. Everything here is fail-soft: read errors, unparsable or
//! oversized shards, and write failures degrade to cache misses without ever
//! surfacing into the LSP loop. The store is local-workspace-disk only — the
//! trust boundary's `neverFetch` network invariant is untouched.

use crate::LspShellState;
use crate::protocol::file_uri_to_path;
use omena_query::{
    OmenaQueryExternalSifInputV0, OmenaQuerySourceDocumentInputV0,
    OmenaQueryStylePackageManifestV0, OmenaQueryStyleResolutionInputsV0,
    OmenaQueryStyleSourceInputV0,
};
use omena_sif::{compute_omena_sif_leaf_hash_v1, write_omena_canonical_json_bytes_v1};
use serde::Serialize;
use serde_json::{Value, json};
use std::cell::RefCell;
use std::fs;
use std::path::{Path, PathBuf};

pub(crate) const DISK_DIAGNOSTICS_CACHE_SCHEMA_VERSION_V0: &str = "1";
/// Serving more than this many diagnostics from one shard is not a plausible
/// healthy state; such shards are deleted-as-miss.
const DISK_DIAGNOSTICS_CACHE_MAX_DIAGNOSTICS: usize = 4096;
const DISK_DIAGNOSTICS_CACHE_SHARD_PRODUCT_V0: &str =
    "omena-lsp-server.disk-diagnostics-cache-shard";
pub(crate) const DISK_DIAGNOSTICS_CACHE_ENV_KILL_SWITCH: &str = "OMENA_LSP_DISK_CACHE";
/// Lives under `.cache/`, which the workspace style indexer skip-list already
/// excludes; shard filenames are hex digests so they can never collide with
/// the thin-client watcher globs (package.json, tsconfig*.json, *.module.*).
const DISK_DIAGNOSTICS_CACHE_RELATIVE_DIR: &str = ".cache/omena/diagnostics-cache-v0";
/// After this many write failures (read-only fs, sandbox, permissions) the
/// session stops attempting writes entirely instead of retrying hot.
const DISK_DIAGNOSTICS_CACHE_MAX_WRITE_FAILURES: usize = 3;
/// Distinguishes the two diagnostics arms so their shards never cross-serve.
/// Byte-identity between the arms is oracle-gated in omena-diff-test, but the
/// cache format stays versioned per arm regardless.
const DISK_DIAGNOSTICS_CACHE_ARM_V0: &str = if cfg!(feature = "salsa-style-diagnostics") {
    "salsaStyleDiagnostics"
} else {
    "straightLine"
};

#[derive(Debug, Clone, Copy)]
pub(crate) struct DiskDiagnosticsCacheLimitsV0 {
    /// Shards larger than this are never written and deleted-as-miss on read.
    pub(crate) max_shard_bytes: u64,
    /// Per-workspace shard count cap; oldest-mtime shards are evicted first.
    pub(crate) max_shards: usize,
    /// Per-workspace total byte cap across all shards.
    pub(crate) max_total_bytes: u64,
}

impl DiskDiagnosticsCacheLimitsV0 {
    pub(crate) const fn with_defaults() -> Self {
        Self {
            max_shard_bytes: 4 * 1024 * 1024,
            max_shards: 256,
            max_total_bytes: 64 * 1024 * 1024,
        }
    }
}

/// Session-scoped write breaker. Content-addressed shard files are race-benign
/// across concurrent servers (atomic temp-file + rename writes), so the only
/// mutable session state is the fail-soft write counter.
#[derive(Debug, Default)]
pub(crate) struct DiskDiagnosticsCacheSessionV0 {
    write_failure_count: usize,
}

impl DiskDiagnosticsCacheSessionV0 {
    pub(crate) fn writes_disabled(&self) -> bool {
        self.write_failure_count >= DISK_DIAGNOSTICS_CACHE_MAX_WRITE_FAILURES
    }

    #[cfg(test)]
    pub(crate) fn write_failure_count(&self) -> usize {
        self.write_failure_count
    }

    fn record_write_failure(&mut self) {
        self.write_failure_count = self.write_failure_count.saturating_add(1);
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiskDiagnosticsCacheBoundaryV0 {
    pub product: &'static str,
    pub owner: &'static str,
    pub cache_model: &'static str,
    pub storage_location: &'static str,
    pub reuse_policy: Vec<&'static str>,
    pub write_policy: Vec<&'static str>,
    pub kill_switches: Vec<&'static str>,
}

pub fn disk_diagnostics_cache_contract() -> DiskDiagnosticsCacheBoundaryV0 {
    DiskDiagnosticsCacheBoundaryV0 {
        product: "omena-lsp-server.disk-diagnostics-cache",
        owner: "omena-lsp-server/diskDiagnosticsCache",
        cache_model: "contentAddressedExactMatchShardStore",
        storage_location: "<workspaceFolder>/.cache/omena/diagnostics-cache-v0",
        reuse_policy: vec![
            "contentAddressedExactKeyMatchOnly",
            "keyChainsFullDiagnosticsInputSurface",
            "keyChainsBinaryIdentityFingerprint",
            "schemaValidateShardsBeforeServe",
            "outputDigestVerifiedBeforeServe",
            "diagnosticShapeAndCountValidatedBeforeServe",
            "oversizedOrUnparsableShardsAreDeletedMisses",
            "neverTrustShardContentBeyondExactKeyServe",
        ],
        write_policy: vec![
            "writeBehindAfterComputeOnly",
            "atomicTempFileRenameWrites",
            "failSoftDisableWritesAfterRepeatedIoFailures",
            "boundedShardCountAndTotalBytesWithOldestMtimeEviction",
            "localWorkspaceDiskOnlyNeverNetwork",
        ],
        kill_switches: vec![
            "envOmenaLspDiskCacheOff",
            "noWorkspaceFolderFilesystemPathDisables",
        ],
    }
}

/// The full input surface of `resolve_style_diagnostics_for_uri`, hashed into
/// the composite key. Canonical-JSON serialization (sorted object keys) fixes
/// the byte order; the COMPONENT SET is the correctness contract — every input
/// that can change the final diagnostics JSON must appear here, plus the
/// crate/schema/arm versions so stale-format shards can never load.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct DiskDiagnosticsCacheKeyInputV0<'a> {
    cache_schema_version: &'a str,
    crate_version: &'a str,
    /// The analysis MECHANISM lives in dependency crates that share the
    /// workspace version, so the crate version alone cannot distinguish two
    /// dev builds with edited diagnostics behavior. The running binary's
    /// identity (length + mtime) does: any rebuild or reinstall invalidates
    /// every shard, at the cost of one cold start per new binary.
    binary_fingerprint: &'a str,
    diagnostics_arm: &'a str,
    target_style_path: &'a str,
    style_sources: &'a [OmenaQueryStyleSourceInputV0],
    source_documents: &'a [OmenaQuerySourceDocumentInputV0],
    package_manifests: &'a [OmenaQueryStylePackageManifestV0],
    external_sifs: &'a [OmenaQueryExternalSifInputV0],
    resolution_inputs: &'a OmenaQueryStyleResolutionInputsV0,
    severity: u8,
    deep_analysis: bool,
}

#[derive(Debug)]
pub(crate) struct DiskDiagnosticsCacheKeyComponentsV0<'a> {
    pub(crate) target_style_path: &'a str,
    pub(crate) style_sources: &'a [OmenaQueryStyleSourceInputV0],
    pub(crate) source_documents: &'a [OmenaQuerySourceDocumentInputV0],
    pub(crate) package_manifests: &'a [OmenaQueryStylePackageManifestV0],
    pub(crate) external_sifs: &'a [OmenaQueryExternalSifInputV0],
    pub(crate) resolution_inputs: &'a OmenaQueryStyleResolutionInputsV0,
    pub(crate) severity: u8,
    pub(crate) deep_analysis: bool,
}

pub(crate) fn disk_diagnostics_cache_key_v0(
    components: &DiskDiagnosticsCacheKeyComponentsV0<'_>,
) -> Option<String> {
    let input = DiskDiagnosticsCacheKeyInputV0 {
        cache_schema_version: DISK_DIAGNOSTICS_CACHE_SCHEMA_VERSION_V0,
        crate_version: env!("CARGO_PKG_VERSION"),
        binary_fingerprint: disk_diagnostics_cache_binary_fingerprint(),
        diagnostics_arm: DISK_DIAGNOSTICS_CACHE_ARM_V0,
        target_style_path: components.target_style_path,
        style_sources: components.style_sources,
        source_documents: components.source_documents,
        package_manifests: components.package_manifests,
        external_sifs: components.external_sifs,
        resolution_inputs: components.resolution_inputs,
        severity: components.severity,
        deep_analysis: components.deep_analysis,
    };
    let canonical_bytes = write_omena_canonical_json_bytes_v1(&input).ok()?;
    Some(
        compute_omena_sif_leaf_hash_v1(canonical_bytes.as_slice())
            .as_str()
            .to_string(),
    )
}

/// One resolved cache placement: directory + composite key + target. Built
/// before the workspace diagnostics compute; `load` is the exact-match read
/// path and `store_write_behind` persists the computed result.
#[derive(Debug, Clone)]
pub(crate) struct DiskDiagnosticsCacheSlotV0 {
    dir: PathBuf,
    key: String,
    target_style_path: String,
}

impl DiskDiagnosticsCacheSlotV0 {
    pub(crate) fn load(&self) -> Option<Value> {
        load_disk_diagnostics_shard_with_limits(
            self.dir.as_path(),
            self.key.as_str(),
            self.target_style_path.as_str(),
            &DiskDiagnosticsCacheLimitsV0::with_defaults(),
        )
    }

    pub(crate) fn store_write_behind(&self, state: &LspShellState, diagnostics: &Value) {
        store_disk_diagnostics_shard_with_limits(
            &state.disk_diagnostics_cache_session,
            self.dir.as_path(),
            self.key.as_str(),
            self.target_style_path.as_str(),
            diagnostics,
            &DiskDiagnosticsCacheLimitsV0::with_defaults(),
        );
    }
}

pub(crate) fn disk_diagnostics_cache_slot_for_resolve(
    state: &LspShellState,
    workspace_folder_uri: Option<&str>,
    target_style_path: &str,
    style_sources: &[OmenaQueryStyleSourceInputV0],
    source_documents: &[OmenaQuerySourceDocumentInputV0],
    external_sifs: &[OmenaQueryExternalSifInputV0],
    resolution_inputs: &OmenaQueryStyleResolutionInputsV0,
) -> Option<DiskDiagnosticsCacheSlotV0> {
    let dir = disk_diagnostics_cache_dir(state, workspace_folder_uri)?;
    let key = disk_diagnostics_cache_key_v0(&DiskDiagnosticsCacheKeyComponentsV0 {
        target_style_path,
        style_sources,
        source_documents,
        package_manifests: state.resolution.package_manifests.as_slice(),
        external_sifs,
        resolution_inputs,
        severity: state.diagnostics.severity,
        deep_analysis: state.diagnostics.deep_analysis,
    })?;
    Some(DiskDiagnosticsCacheSlotV0 {
        dir,
        key,
        target_style_path: target_style_path.to_string(),
    })
}

fn disk_diagnostics_cache_dir(
    state: &LspShellState,
    workspace_folder_uri: Option<&str>,
) -> Option<PathBuf> {
    disk_diagnostics_cache_dir_with_kill_switch(
        state,
        workspace_folder_uri,
        disk_diagnostics_cache_kill_switch_engaged(),
    )
}

/// Cache placement: the document's owning workspace folder when its filesystem
/// path resolves, else the first registered folder with a filesystem path,
/// else disabled (orphan documents in pathless workspaces have no cache home).
pub(crate) fn disk_diagnostics_cache_dir_with_kill_switch(
    state: &LspShellState,
    workspace_folder_uri: Option<&str>,
    kill_switch_engaged: bool,
) -> Option<PathBuf> {
    if kill_switch_engaged {
        return None;
    }
    let root = disk_diagnostics_cache_workspace_root(state, workspace_folder_uri)?;
    Some(root.join(DISK_DIAGNOSTICS_CACHE_RELATIVE_DIR))
}

fn disk_diagnostics_cache_workspace_root(
    state: &LspShellState,
    workspace_folder_uri: Option<&str>,
) -> Option<PathBuf> {
    if let Some(uri) = workspace_folder_uri
        && state.workspace_runtime_registry.get(uri).is_some()
        && let Some(path) = file_uri_to_path(uri)
    {
        return Some(path);
    }
    state
        .workspace_runtime_registry
        .folders()
        .find_map(|folder| file_uri_to_path(folder.uri.as_str()))
}

fn disk_diagnostics_cache_kill_switch_engaged() -> bool {
    std::env::var(DISK_DIAGNOSTICS_CACHE_ENV_KILL_SWITCH)
        .is_ok_and(|value| is_disk_diagnostics_cache_kill_switch_value(value.as_str()))
}

pub(crate) fn is_disk_diagnostics_cache_kill_switch_value(value: &str) -> bool {
    value.eq_ignore_ascii_case("off") || value == "0" || value.eq_ignore_ascii_case("false")
}

/// Identity of the running binary, folded into the cache key so a rebuilt
/// server (changed analysis behavior, same workspace version) never serves a
/// previous build's shards. Falls back to a constant when the executable
/// cannot be inspected — version + schema still guard released builds.
fn disk_diagnostics_cache_binary_fingerprint() -> &'static str {
    static FINGERPRINT: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    FINGERPRINT.get_or_init(|| {
        std::env::current_exe()
            .and_then(fs::metadata)
            .ok()
            .and_then(|metadata| {
                let modified = metadata
                    .modified()
                    .ok()?
                    .duration_since(std::time::UNIX_EPOCH)
                    .ok()?;
                Some(format!(
                    "len{}-mtime{}.{:09}",
                    metadata.len(),
                    modified.as_secs(),
                    modified.subsec_nanos(),
                ))
            })
            .unwrap_or_else(|| "unknownBinaryIdentity".to_string())
    })
}

/// The shard filename is the hex component of a `blake3:<hex>` key. The hex
/// is validated here so containment is a property of the helper, not of every
/// caller: any non-hex component (which could otherwise smuggle separators or
/// `..`) disables the path entirely.
fn disk_diagnostics_shard_file_path(dir: &Path, key: &str) -> Option<PathBuf> {
    let hex = key.split(':').next_back().unwrap_or(key);
    if hex.is_empty()
        || hex.len() > 64
        || !hex
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return None;
    }
    Some(dir.join(format!("{hex}.json")))
}

pub(crate) fn load_disk_diagnostics_shard_with_limits(
    dir: &Path,
    key: &str,
    target_style_path: &str,
    limits: &DiskDiagnosticsCacheLimitsV0,
) -> Option<Value> {
    let shard_path = disk_diagnostics_shard_file_path(dir, key)?;
    let metadata = fs::metadata(shard_path.as_path()).ok()?;
    if !metadata.is_file() {
        return None;
    }
    if metadata.len() > limits.max_shard_bytes {
        let _ = fs::remove_file(shard_path.as_path());
        return None;
    }
    let bytes = fs::read(shard_path.as_path()).ok()?;
    let Ok(mut shard) = serde_json::from_slice::<Value>(bytes.as_slice()) else {
        let _ = fs::remove_file(shard_path.as_path());
        return None;
    };
    if !disk_diagnostics_shard_matches(&shard, key, target_style_path) {
        let _ = fs::remove_file(shard_path.as_path());
        return None;
    }
    shard.get_mut("diagnosticsJson").map(Value::take)
}

/// The composite key proves the INPUTS match; the stored `outputDigest`
/// (blake3 over the canonical-JSON diagnostics bytes) binds the OUTPUT to
/// what the writer computed for those inputs, so bit-rot, truncation that
/// still parses, or a payload swapped under a valid key all miss instead of
/// being served. The digest carries no secret — it is an integrity check
/// against corruption and buggy writes, not an authentication mechanism;
/// an actor who can write into `.cache/` already controls the workspace.
fn disk_diagnostics_shard_matches(shard: &Value, key: &str, target_style_path: &str) -> bool {
    shard.get("schemaVersion").and_then(Value::as_str)
        == Some(DISK_DIAGNOSTICS_CACHE_SCHEMA_VERSION_V0)
        && shard.get("product").and_then(Value::as_str)
            == Some(DISK_DIAGNOSTICS_CACHE_SHARD_PRODUCT_V0)
        && shard.get("key").and_then(Value::as_str) == Some(key)
        && shard.get("targetStylePath").and_then(Value::as_str) == Some(target_style_path)
        && shard
            .get("diagnosticsJson")
            .is_some_and(disk_diagnostics_payload_is_well_formed)
        && shard.get("outputDigest").and_then(Value::as_str)
            == shard
                .get("diagnosticsJson")
                .and_then(disk_diagnostics_output_digest)
                .as_deref()
}

/// Every served element must look like an LSP diagnostic (object with a
/// `range` object and a `message` string) and the count must be plausible —
/// the writer only ever produces such arrays, so anything else is corruption.
fn disk_diagnostics_payload_is_well_formed(diagnostics: &Value) -> bool {
    let Some(elements) = diagnostics.as_array() else {
        return false;
    };
    elements.len() <= DISK_DIAGNOSTICS_CACHE_MAX_DIAGNOSTICS
        && elements.iter().all(|element| {
            element.get("range").is_some_and(Value::is_object)
                && element
                    .get("message")
                    .is_some_and(|message| message.is_string())
        })
}

fn disk_diagnostics_output_digest(diagnostics: &Value) -> Option<String> {
    let canonical_bytes = write_omena_canonical_json_bytes_v1(diagnostics).ok()?;
    Some(
        compute_omena_sif_leaf_hash_v1(canonical_bytes.as_slice())
            .as_str()
            .to_string(),
    )
}

pub(crate) fn store_disk_diagnostics_shard_with_limits(
    session: &RefCell<DiskDiagnosticsCacheSessionV0>,
    dir: &Path,
    key: &str,
    target_style_path: &str,
    diagnostics: &Value,
    limits: &DiskDiagnosticsCacheLimitsV0,
) {
    if session.borrow().writes_disabled() || !disk_diagnostics_payload_is_well_formed(diagnostics) {
        return;
    }
    let Some(output_digest) = disk_diagnostics_output_digest(diagnostics) else {
        return;
    };
    let shard = json!({
        "schemaVersion": DISK_DIAGNOSTICS_CACHE_SCHEMA_VERSION_V0,
        "product": DISK_DIAGNOSTICS_CACHE_SHARD_PRODUCT_V0,
        "key": key,
        "targetStylePath": target_style_path,
        "outputDigest": output_digest,
        "diagnosticsJson": diagnostics,
    });
    let Ok(bytes) = serde_json::to_vec(&shard) else {
        return;
    };
    if bytes.len() as u64 > limits.max_shard_bytes {
        return;
    }
    if write_disk_diagnostics_shard_atomically(dir, key, bytes.as_slice()).is_err() {
        session.borrow_mut().record_write_failure();
        return;
    }
    enforce_disk_diagnostics_cache_caps(dir, limits);
}

fn write_disk_diagnostics_shard_atomically(
    dir: &Path,
    key: &str,
    bytes: &[u8],
) -> std::io::Result<()> {
    fs::create_dir_all(dir)?;
    let final_path = disk_diagnostics_shard_file_path(dir, key).ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::InvalidInput, "non-hex shard key")
    })?;
    // Same-directory rename keeps the swap atomic on POSIX; the pid suffix
    // keeps concurrent servers (multi-editor, multi-window) from clobbering
    // each other's in-flight temp files.
    let temp_path = final_path.with_extension(format!("tmp-{}", std::process::id()));
    fs::write(temp_path.as_path(), bytes)?;
    let renamed = fs::rename(temp_path.as_path(), final_path.as_path());
    if renamed.is_err() {
        let _ = fs::remove_file(temp_path.as_path());
        // Windows refuses rename-over-existing. A destination that appeared
        // in the meantime means a concurrent server already wrote this
        // content-addressed shard — that is success, not failure.
        if final_path.is_file() {
            return Ok(());
        }
    }
    renamed
}

/// Content-addressed keys never overwrite, so growth is bounded here: evict
/// oldest-mtime shards (write-order; reads do not refresh mtime in stage 1)
/// until both the shard-count and total-byte caps hold. At least the
/// globally-newest shard is retained — under concurrent writers a peer's
/// newer shard may outrank the one just written here, which is a benign
/// miss. Best-effort like every other write.
fn enforce_disk_diagnostics_cache_caps(dir: &Path, limits: &DiskDiagnosticsCacheLimitsV0) {
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
    let mut total_bytes = shards.iter().map(|shard| shard.1).sum::<u64>();
    let mut shard_count = shards.len();
    for (_, shard_bytes, shard_path) in shards {
        if shard_count <= 1
            || (shard_count <= limits.max_shards && total_bytes <= limits.max_total_bytes)
        {
            break;
        }
        if fs::remove_file(shard_path.as_path()).is_ok() {
            shard_count -= 1;
            total_bytes = total_bytes.saturating_sub(shard_bytes);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    fn temp_cache_dir(suffix: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "omena-lsp-disk-cache-{suffix}-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(dir.as_path());
        dir
    }

    struct KeyFixture {
        target_style_path: String,
        style_sources: Vec<OmenaQueryStyleSourceInputV0>,
        source_documents: Vec<OmenaQuerySourceDocumentInputV0>,
        package_manifests: Vec<OmenaQueryStylePackageManifestV0>,
        external_sifs: Vec<OmenaQueryExternalSifInputV0>,
        resolution_inputs: OmenaQueryStyleResolutionInputsV0,
        severity: u8,
        deep_analysis: bool,
    }

    impl KeyFixture {
        fn base() -> Self {
            Self {
                target_style_path: "file:///repo/src/App.module.scss".to_string(),
                style_sources: vec![OmenaQueryStyleSourceInputV0 {
                    style_path: "file:///repo/src/App.module.scss".to_string(),
                    style_source: ".btn { color: red; }".to_string(),
                }],
                source_documents: vec![OmenaQuerySourceDocumentInputV0 {
                    source_path: "file:///repo/src/App.tsx".to_string(),
                    source_source: "import styles from './App.module.scss';".to_string(),
                    source_syntax_index: None,
                    has_unresolved_style_import: false,
                }],
                package_manifests: Vec::new(),
                external_sifs: Vec::new(),
                resolution_inputs: OmenaQueryStyleResolutionInputsV0::default(),
                severity: 2,
                deep_analysis: false,
            }
        }

        fn key(&self) -> Option<String> {
            disk_diagnostics_cache_key_v0(&DiskDiagnosticsCacheKeyComponentsV0 {
                target_style_path: self.target_style_path.as_str(),
                style_sources: self.style_sources.as_slice(),
                source_documents: self.source_documents.as_slice(),
                package_manifests: self.package_manifests.as_slice(),
                external_sifs: self.external_sifs.as_slice(),
                resolution_inputs: &self.resolution_inputs,
                severity: self.severity,
                deep_analysis: self.deep_analysis,
            })
        }
    }

    fn fixture_external_sif() -> Option<OmenaQueryExternalSifInputV0> {
        let sif = omena_sif::OmenaSifV1::from_static_exports(
            "https://cdn.example/tokens.scss",
            omena_sif::OmenaSifGeneratorV1 {
                name: "fixture-sifgen".to_string(),
                version: "0.1.0".to_string(),
                toolchain_id: "fixture-sifgen@0.1.0".to_string(),
            },
            omena_sif::OmenaSifSourceV1 {
                syntax: omena_sif::OmenaSifSourceSyntaxV1::Scss,
            },
            omena_sif::OmenaSifExportsV1 {
                variables: vec![omena_sif::OmenaSifVariableExportV1 {
                    name: "$brand".to_string(),
                    defaulted: true,
                    value_repr: Some("red".to_string()),
                }],
                mixins: Vec::new(),
                functions: Vec::new(),
                placeholders: Vec::new(),
                forwards: Vec::new(),
            },
            Vec::new(),
            b"$brand: red !default;",
        )
        .ok()?;
        Some(OmenaQueryExternalSifInputV0 {
            canonical_url: "https://cdn.example/tokens.scss".to_string(),
            sif,
        })
    }

    #[test]
    fn key_is_stable_for_identical_inputs_and_blake3_prefixed() -> Result<(), &'static str> {
        let key = KeyFixture::base().key().ok_or("base key")?;
        let key_again = KeyFixture::base().key().ok_or("base key again")?;
        assert_eq!(key, key_again);
        assert!(key.starts_with("blake3:"));
        Ok(())
    }

    #[test]
    fn key_changes_when_any_input_component_changes() -> Result<(), &'static str> {
        let mut keys = BTreeSet::new();
        keys.insert(KeyFixture::base().key().ok_or("base")?);

        let mut corpus_text = KeyFixture::base();
        corpus_text.style_sources[0].style_source = ".btn { color: blue; }".to_string();
        keys.insert(corpus_text.key().ok_or("corpus text")?);

        let mut corpus_path = KeyFixture::base();
        corpus_path.style_sources[0].style_path = "file:///repo/src/Other.module.scss".to_string();
        keys.insert(corpus_path.key().ok_or("corpus path")?);

        let mut extra_style = KeyFixture::base();
        extra_style
            .style_sources
            .push(OmenaQueryStyleSourceInputV0 {
                style_path: "file:///repo/src/theme.scss".to_string(),
                style_source: "$brand: red;".to_string(),
            });
        keys.insert(extra_style.key().ok_or("extra style")?);

        let mut source_document = KeyFixture::base();
        source_document.source_documents[0].source_source =
            "import other from './App.module.scss';".to_string();
        keys.insert(source_document.key().ok_or("source document")?);

        let mut manifest = KeyFixture::base();
        manifest
            .package_manifests
            .push(OmenaQueryStylePackageManifestV0 {
                package_json_path: "file:///repo/package.json".to_string(),
                package_json_source: "{\"name\":\"repo\"}".to_string(),
            });
        keys.insert(manifest.key().ok_or("manifest")?);

        let mut resolution = KeyFixture::base();
        resolution
            .resolution_inputs
            .package_manifests
            .push(OmenaQueryStylePackageManifestV0 {
                package_json_path: "file:///repo/packages/ui/package.json".to_string(),
                package_json_source: "{\"name\":\"ui\"}".to_string(),
            });
        keys.insert(resolution.key().ok_or("resolution inputs")?);

        let mut external_sif = KeyFixture::base();
        external_sif
            .external_sifs
            .push(fixture_external_sif().ok_or("external sif fixture")?);
        keys.insert(external_sif.key().ok_or("external sif")?);

        let mut target = KeyFixture::base();
        target.target_style_path = "file:///repo/src/Other.module.scss".to_string();
        keys.insert(target.key().ok_or("target")?);

        let mut severity = KeyFixture::base();
        severity.severity = 1;
        keys.insert(severity.key().ok_or("severity")?);

        let mut deep_analysis = KeyFixture::base();
        deep_analysis.deep_analysis = true;
        keys.insert(deep_analysis.key().ok_or("deep analysis")?);

        assert_eq!(keys.len(), 11, "every varied component must change the key");
        Ok(())
    }

    #[test]
    fn shard_roundtrips_through_store_and_load() -> Result<(), &'static str> {
        let dir = temp_cache_dir("roundtrip");
        let fixture = KeyFixture::base();
        let key = fixture.key().ok_or("key")?;
        let diagnostics = json!([
            {"code": "missingCustomProperty", "message": "unknown --brand",
             "range": {"start": {"line": 0, "character": 0}, "end": {"line": 0, "character": 1}}},
        ]);
        let session = RefCell::new(DiskDiagnosticsCacheSessionV0::default());
        let limits = DiskDiagnosticsCacheLimitsV0::with_defaults();
        store_disk_diagnostics_shard_with_limits(
            &session,
            dir.as_path(),
            key.as_str(),
            fixture.target_style_path.as_str(),
            &diagnostics,
            &limits,
        );

        assert!(
            disk_diagnostics_shard_file_path(dir.as_path(), key.as_str())
                .ok_or("shard path")?
                .is_file()
        );
        let loaded = load_disk_diagnostics_shard_with_limits(
            dir.as_path(),
            key.as_str(),
            fixture.target_style_path.as_str(),
            &limits,
        );
        assert_eq!(loaded, Some(diagnostics));
        assert_eq!(session.borrow().write_failure_count(), 0);
        Ok(())
    }

    #[test]
    fn load_misses_on_key_or_target_mismatch_without_serving_foreign_content()
    -> Result<(), &'static str> {
        let dir = temp_cache_dir("mismatch");
        let fixture = KeyFixture::base();
        let key = fixture.key().ok_or("key")?;
        let session = RefCell::new(DiskDiagnosticsCacheSessionV0::default());
        let limits = DiskDiagnosticsCacheLimitsV0::with_defaults();
        store_disk_diagnostics_shard_with_limits(
            &session,
            dir.as_path(),
            key.as_str(),
            fixture.target_style_path.as_str(),
            &json!([]),
            &limits,
        );

        let other_key = "blake3:0000000000000000000000000000000000000000000000000000000000000000";
        assert_eq!(
            load_disk_diagnostics_shard_with_limits(
                dir.as_path(),
                other_key,
                fixture.target_style_path.as_str(),
                &limits,
            ),
            None,
        );
        // Same shard file, mismatched target: deleted-as-miss.
        assert_eq!(
            load_disk_diagnostics_shard_with_limits(
                dir.as_path(),
                key.as_str(),
                "file:///repo/src/Other.module.scss",
                &limits,
            ),
            None,
        );
        assert!(
            !disk_diagnostics_shard_file_path(dir.as_path(), key.as_str())
                .ok_or("shard path")?
                .exists()
        );
        Ok(())
    }

    #[test]
    fn garbage_and_stale_schema_shards_are_deleted_misses() -> Result<(), &'static str> {
        let dir = temp_cache_dir("garbage");
        let key = KeyFixture::base().key().ok_or("key")?;
        let target = "file:///repo/src/App.module.scss";
        let limits = DiskDiagnosticsCacheLimitsV0::with_defaults();
        let shard_path =
            disk_diagnostics_shard_file_path(dir.as_path(), key.as_str()).ok_or("shard path")?;
        fs::create_dir_all(dir.as_path()).map_err(|_| "create cache dir")?;

        fs::write(shard_path.as_path(), b"{ truncated garbage").map_err(|_| "write garbage")?;
        assert_eq!(
            load_disk_diagnostics_shard_with_limits(dir.as_path(), key.as_str(), target, &limits),
            None,
        );
        assert!(!shard_path.exists(), "garbage shard must be deleted");

        let stale = json!({
            "schemaVersion": "999",
            "product": DISK_DIAGNOSTICS_CACHE_SHARD_PRODUCT_V0,
            "key": key,
            "targetStylePath": target,
            "diagnosticsJson": [],
        });
        fs::write(
            shard_path.as_path(),
            serde_json::to_vec(&stale).map_err(|_| "serialize stale")?,
        )
        .map_err(|_| "write stale")?;
        assert_eq!(
            load_disk_diagnostics_shard_with_limits(dir.as_path(), key.as_str(), target, &limits),
            None,
        );
        assert!(!shard_path.exists(), "stale-schema shard must be deleted");

        let non_array = json!({
            "schemaVersion": DISK_DIAGNOSTICS_CACHE_SCHEMA_VERSION_V0,
            "product": DISK_DIAGNOSTICS_CACHE_SHARD_PRODUCT_V0,
            "key": key,
            "targetStylePath": target,
            "diagnosticsJson": {"not": "an array"},
        });
        fs::write(
            shard_path.as_path(),
            serde_json::to_vec(&non_array).map_err(|_| "serialize non-array")?,
        )
        .map_err(|_| "write non-array")?;
        assert_eq!(
            load_disk_diagnostics_shard_with_limits(dir.as_path(), key.as_str(), target, &limits),
            None,
        );
        assert!(!shard_path.exists(), "non-array shard must be deleted");
        Ok(())
    }

    #[test]
    fn oversized_shards_are_neither_written_nor_served() -> Result<(), &'static str> {
        let dir = temp_cache_dir("oversize");
        let key = KeyFixture::base().key().ok_or("key")?;
        let target = "file:///repo/src/App.module.scss";
        let limits = DiskDiagnosticsCacheLimitsV0 {
            max_shard_bytes: 128,
            max_shards: 256,
            max_total_bytes: 64 * 1024 * 1024,
        };
        let session = RefCell::new(DiskDiagnosticsCacheSessionV0::default());
        let oversized = json!([
            {"message": "x".repeat(512), "range": {"start": {}, "end": {}}},
        ]);
        store_disk_diagnostics_shard_with_limits(
            &session,
            dir.as_path(),
            key.as_str(),
            target,
            &oversized,
            &limits,
        );
        let shard_path =
            disk_diagnostics_shard_file_path(dir.as_path(), key.as_str()).ok_or("shard path")?;
        assert!(
            !shard_path.exists(),
            "oversized payload must not be written"
        );
        assert_eq!(session.borrow().write_failure_count(), 0);

        fs::create_dir_all(dir.as_path()).map_err(|_| "create cache dir")?;
        fs::write(shard_path.as_path(), vec![b'x'; 512]).map_err(|_| "write oversized")?;
        assert_eq!(
            load_disk_diagnostics_shard_with_limits(dir.as_path(), key.as_str(), target, &limits),
            None,
        );
        assert!(!shard_path.exists(), "oversized shard must be deleted");
        Ok(())
    }

    #[test]
    fn write_failures_disable_writes_for_the_session_after_three() -> Result<(), &'static str> {
        let base = temp_cache_dir("write-failure");
        fs::create_dir_all(base.as_path()).map_err(|_| "create base")?;
        let blocker = base.join("blocker");
        fs::write(blocker.as_path(), b"not a directory").map_err(|_| "write blocker")?;
        let undeliverable_dir = blocker.join("nested");
        let key = KeyFixture::base().key().ok_or("key")?;
        let target = "file:///repo/src/App.module.scss";
        let limits = DiskDiagnosticsCacheLimitsV0::with_defaults();
        let session = RefCell::new(DiskDiagnosticsCacheSessionV0::default());

        for _ in 0..5 {
            store_disk_diagnostics_shard_with_limits(
                &session,
                undeliverable_dir.as_path(),
                key.as_str(),
                target,
                &json!([]),
                &limits,
            );
        }
        assert_eq!(
            session.borrow().write_failure_count(),
            3,
            "breaker must stop retrying after three failures",
        );
        assert!(session.borrow().writes_disabled());

        let healthy_dir = base.join("healthy");
        store_disk_diagnostics_shard_with_limits(
            &session,
            healthy_dir.as_path(),
            key.as_str(),
            target,
            &json!([]),
            &limits,
        );
        assert!(
            !healthy_dir.exists(),
            "writes stay disabled for the session once the breaker trips",
        );
        Ok(())
    }

    #[test]
    fn caps_evict_oldest_shards_first_and_keep_the_newest() -> Result<(), &'static str> {
        let dir = temp_cache_dir("caps");
        let target = "file:///repo/src/App.module.scss";
        let limits = DiskDiagnosticsCacheLimitsV0 {
            max_shard_bytes: 4 * 1024 * 1024,
            max_shards: 2,
            max_total_bytes: 64 * 1024 * 1024,
        };
        let session = RefCell::new(DiskDiagnosticsCacheSessionV0::default());
        let keys = ["blake3:aaaa", "blake3:bbbb", "blake3:cccc"];
        for key in keys {
            store_disk_diagnostics_shard_with_limits(
                &session,
                dir.as_path(),
                key,
                target,
                &json!([]),
                &limits,
            );
            std::thread::sleep(std::time::Duration::from_millis(10));
        }

        assert!(
            !disk_diagnostics_shard_file_path(dir.as_path(), keys[0])
                .ok_or("evicted path")?
                .exists()
        );
        assert!(
            disk_diagnostics_shard_file_path(dir.as_path(), keys[2])
                .ok_or("newest path")?
                .is_file()
        );
        let remaining = fs::read_dir(dir.as_path())
            .map_err(|_| "read cache dir")?
            .flatten()
            .count();
        assert_eq!(remaining, 2);
        Ok(())
    }

    #[test]
    fn total_byte_cap_bounds_the_store() -> Result<(), &'static str> {
        let dir = temp_cache_dir("byte-cap");
        let target = "file:///repo/src/App.module.scss";
        let limits = DiskDiagnosticsCacheLimitsV0 {
            max_shard_bytes: 10_000,
            max_shards: 100,
            max_total_bytes: 300,
        };
        let session = RefCell::new(DiskDiagnosticsCacheSessionV0::default());
        let diagnostics = json!([
            {"message": "y".repeat(120), "range": {"start": {}, "end": {}}},
        ]);
        store_disk_diagnostics_shard_with_limits(
            &session,
            dir.as_path(),
            "blake3:dddd",
            target,
            &diagnostics,
            &limits,
        );
        std::thread::sleep(std::time::Duration::from_millis(10));
        store_disk_diagnostics_shard_with_limits(
            &session,
            dir.as_path(),
            "blake3:eeee",
            target,
            &diagnostics,
            &limits,
        );

        assert!(
            !disk_diagnostics_shard_file_path(dir.as_path(), "blake3:dddd")
                .ok_or("evicted path")?
                .exists()
        );
        assert!(
            disk_diagnostics_shard_file_path(dir.as_path(), "blake3:eeee")
                .ok_or("kept path")?
                .is_file()
        );
        Ok(())
    }

    #[test]
    fn non_hex_key_components_disable_the_shard_path_entirely() {
        let dir = Path::new("/var/lib/omena-cache");
        assert_eq!(
            disk_diagnostics_shard_file_path(dir, "../../../etc/evil"),
            None,
            "a colon-less non-hex key must never derive a path",
        );
        assert_eq!(
            disk_diagnostics_shard_file_path(dir, "blake3:../escape"),
            None,
        );
        assert_eq!(disk_diagnostics_shard_file_path(dir, "blake3:"), None);
        assert_eq!(
            disk_diagnostics_shard_file_path(dir, "blake3:ABCDEF"),
            None,
            "uppercase is not produced by the digest formatter",
        );
        assert_eq!(
            disk_diagnostics_shard_file_path(dir, "blake3:abc123"),
            Some(dir.join("abc123.json")),
        );
    }

    #[test]
    fn tampered_payload_with_stale_output_digest_is_a_deleted_miss() -> Result<(), &'static str> {
        let dir = temp_cache_dir("digest");
        let fixture = KeyFixture::base();
        let key = fixture.key().ok_or("key")?;
        let limits = DiskDiagnosticsCacheLimitsV0::with_defaults();
        let session = RefCell::new(DiskDiagnosticsCacheSessionV0::default());
        let genuine = json!([
            {"message": "real", "range": {"start": {}, "end": {}}},
        ]);
        store_disk_diagnostics_shard_with_limits(
            &session,
            dir.as_path(),
            key.as_str(),
            fixture.target_style_path.as_str(),
            &genuine,
            &limits,
        );
        let shard_path =
            disk_diagnostics_shard_file_path(dir.as_path(), key.as_str()).ok_or("path")?;
        let mut shard: Value = serde_json::from_slice(
            fs::read(shard_path.as_path())
                .map_err(|_| "read shard")?
                .as_slice(),
        )
        .map_err(|_| "parse shard")?;
        shard["diagnosticsJson"] = json!([
            {"message": "forged", "range": {"start": {}, "end": {}}},
        ]);
        fs::write(
            shard_path.as_path(),
            serde_json::to_vec(&shard).map_err(|_| "serialize")?,
        )
        .map_err(|_| "write tampered")?;

        assert_eq!(
            load_disk_diagnostics_shard_with_limits(
                dir.as_path(),
                key.as_str(),
                fixture.target_style_path.as_str(),
                &limits,
            ),
            None,
            "payload not matching the output digest must miss",
        );
        assert!(!shard_path.exists(), "digest-mismatched shard is deleted");
        Ok(())
    }

    #[test]
    fn malformed_diagnostic_elements_are_rejected_even_with_a_valid_digest()
    -> Result<(), &'static str> {
        let dir = temp_cache_dir("shape");
        let fixture = KeyFixture::base();
        let key = fixture.key().ok_or("key")?;
        let limits = DiskDiagnosticsCacheLimitsV0::with_defaults();
        let malformed = json!(["not-an-object", 42]);
        let digest = disk_diagnostics_output_digest(&malformed).ok_or("digest")?;
        let shard = json!({
            "schemaVersion": DISK_DIAGNOSTICS_CACHE_SCHEMA_VERSION_V0,
            "product": DISK_DIAGNOSTICS_CACHE_SHARD_PRODUCT_V0,
            "key": key,
            "targetStylePath": fixture.target_style_path,
            "outputDigest": digest,
            "diagnosticsJson": malformed,
        });
        fs::create_dir_all(dir.as_path()).map_err(|_| "create dir")?;
        let shard_path =
            disk_diagnostics_shard_file_path(dir.as_path(), key.as_str()).ok_or("path")?;
        fs::write(
            shard_path.as_path(),
            serde_json::to_vec(&shard).map_err(|_| "serialize")?,
        )
        .map_err(|_| "write")?;

        assert_eq!(
            load_disk_diagnostics_shard_with_limits(
                dir.as_path(),
                key.as_str(),
                fixture.target_style_path.as_str(),
                &limits,
            ),
            None,
            "elements without range+message must miss even when digested",
        );
        assert!(!shard_path.exists());
        Ok(())
    }

    #[test]
    fn kill_switch_values_disable_reads_and_writes() {
        assert!(is_disk_diagnostics_cache_kill_switch_value("off"));
        assert!(is_disk_diagnostics_cache_kill_switch_value("OFF"));
        assert!(is_disk_diagnostics_cache_kill_switch_value("0"));
        assert!(is_disk_diagnostics_cache_kill_switch_value("false"));
        assert!(!is_disk_diagnostics_cache_kill_switch_value(""));
        assert!(!is_disk_diagnostics_cache_kill_switch_value("on"));
        assert!(!is_disk_diagnostics_cache_kill_switch_value("1"));

        let mut state = LspShellState::default();
        state
            .workspace_runtime_registry
            .insert("file:///omena-disk-cache-kill-switch-root", "root");
        assert_eq!(
            disk_diagnostics_cache_dir_with_kill_switch(
                &state,
                Some("file:///omena-disk-cache-kill-switch-root"),
                true,
            ),
            None,
        );
        assert_eq!(
            disk_diagnostics_cache_dir_with_kill_switch(
                &state,
                Some("file:///omena-disk-cache-kill-switch-root"),
                false,
            ),
            Some(PathBuf::from(
                "/omena-disk-cache-kill-switch-root/.cache/omena/diagnostics-cache-v0",
            )),
        );
    }

    #[test]
    fn cache_dir_uses_owning_folder_then_first_folder_then_disables() {
        let state = LspShellState::default();
        assert_eq!(
            disk_diagnostics_cache_dir_with_kill_switch(&state, None, false),
            None,
            "no workspace folder filesystem path disables the cache",
        );

        let mut state = LspShellState::default();
        state
            .workspace_runtime_registry
            .insert("file:///omena-disk-cache-first-root", "first");
        state
            .workspace_runtime_registry
            .insert("file:///omena-disk-cache-owner-root", "owner");
        assert_eq!(
            disk_diagnostics_cache_dir_with_kill_switch(
                &state,
                Some("file:///omena-disk-cache-owner-root"),
                false,
            ),
            Some(PathBuf::from(
                "/omena-disk-cache-owner-root/.cache/omena/diagnostics-cache-v0",
            )),
        );
        // Orphan documents fall back to the first registered folder.
        assert_eq!(
            disk_diagnostics_cache_dir_with_kill_switch(&state, None, false),
            Some(PathBuf::from(
                "/omena-disk-cache-first-root/.cache/omena/diagnostics-cache-v0",
            )),
        );
    }
}
