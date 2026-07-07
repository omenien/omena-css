//! RFC 0009 Pillar C (rfcs#66) stage 2: a persistent, verifying-trace
//! workspace style-diagnostics shard store (the Tidepool — what a tide
//! leaves behind for the next cold open).
//!
//! Stage 1 keyed each shard by a hash of the FULL diagnostics input surface,
//! which made a single changed file invalidate every target's shard and paid
//! an O(corpus) serialize+hash PER TARGET. Stage 2 flips the trace direction
//! (the verifying-trace rebuilder of Mokhov, Mitchell & Peyton Jones, "Build
//! Systems à la Carte", ICFP 2018): each shard lives at a STABLE address
//! derived from the target path alone and records the read-set the compute
//! actually declared — per-dependency `(path, contentHash)` entries plus an
//! environment fingerprint over everything path-membership- or settings-
//! shaped. A lookup re-hashes only the recorded dependencies against the
//! current surface; a hit therefore survives edits OUTSIDE the target's
//! dependency cone, and the depfile argument (same recorded inputs => same
//! reads => same output, with the membership fingerprint pinning the
//! resolver's candidate space) keeps a verified hit byte-identical to a
//! recompute. Read-set completeness is oracle-gated, not assumed.
//! Everything here is fail-soft: read errors, unparsable or oversized
//! shards, and write failures degrade to cache misses without ever
//! surfacing into the LSP loop. The store is local-workspace-disk only —
//! the trust boundary's `neverFetch` network invariant is untouched.

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

pub(crate) const DISK_DIAGNOSTICS_CACHE_SCHEMA_VERSION_V1: &str = "2";
/// Serving more than this many diagnostics from one shard is not a plausible
/// healthy state; such shards are deleted-as-miss.
const DISK_DIAGNOSTICS_CACHE_MAX_DIAGNOSTICS: usize = 4096;
/// A recorded read-set beyond this is not a plausible dependency cone for
/// one target; such shards are treated as misses instead of hashing forever.
const DISK_DIAGNOSTICS_CACHE_MAX_DEPS: usize = 16_384;
const DISK_DIAGNOSTICS_CACHE_SHARD_PRODUCT_V0: &str =
    "omena-lsp-server.disk-diagnostics-cache-shard";
pub(crate) const DISK_DIAGNOSTICS_CACHE_ENV_KILL_SWITCH: &str = "OMENA_LSP_DISK_CACHE";
/// Serve-side shadow oracle: when set truthy, verified hits do NOT short-
/// circuit the wave — the target is recomputed anyway and byte-compared
/// against the shard, so read-set completeness is checked empirically.
pub(crate) const DISK_DIAGNOSTICS_CACHE_ORACLE_ENV: &str = "OMENA_LSP_DISK_CACHE_ORACLE";
/// Lives under `.cache/`, which the workspace style indexer skip-list already
/// excludes; shard filenames are hex digests so they can never collide with
/// the thin-client watcher globs (package.json, tsconfig*.json, *.module.*).
const DISK_DIAGNOSTICS_CACHE_RELATIVE_DIR_V1: &str = ".cache/omena/diagnostics-cache-v1";
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
        cache_model: "verifyingTraceStableAddressShardStore",
        storage_location: "<workspaceFolder>/.cache/omena/diagnostics-cache-v1",
        reuse_policy: vec![
            "stableAddressPerTargetOneShardEach",
            "recordedReadSetVerifiedPerDependencyContentHash",
            "environmentFingerprintPinsMembershipAndSettings",
            "manifestMustIncludeTargetItself",
            "binaryIdentityFingerprintVerifiedBeforeServe",
            "schemaValidateShardsBeforeServe",
            "outputDigestVerifiedBeforeServe",
            "diagnosticShapeAndCountValidatedBeforeServe",
            "oversizedOrUnparsableShardsAreDeletedMisses",
            "verificationMissesLeaveShardForInPlaceOverwrite",
            "readSetCompletenessOracleGatedNotAssumed",
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

/// The environment half of the verifying trace: every input that is NOT a
/// per-file content read — settings, resolver configuration, external
/// resolution facts, and the PATH MEMBERSHIP of both corpora. Membership is
/// load-bearing for soundness: creating or deleting a file can change what a
/// previously-failing specifier resolves to without touching any recorded
/// dependency's content, so the candidate space itself is fingerprinted.
/// Canonical-JSON serialization (sorted object keys) fixes the byte order.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct DiskDiagnosticsCacheEnvironmentInputV1<'a> {
    cache_schema_version: &'a str,
    crate_version: &'a str,
    diagnostics_arm: &'a str,
    style_paths: Vec<&'a str>,
    source_paths: Vec<&'a str>,
    package_manifests: &'a [OmenaQueryStylePackageManifestV0],
    external_sifs: &'a [OmenaQueryExternalSifInputV0],
    resolution_inputs: &'a OmenaQueryStyleResolutionInputsV0,
    severity: u8,
    deep_analysis: bool,
}

#[derive(Debug)]
pub(crate) struct DiskDiagnosticsCacheEnvironmentComponentsV1<'a> {
    pub(crate) style_sources: &'a [OmenaQueryStyleSourceInputV0],
    pub(crate) source_documents: &'a [OmenaQuerySourceDocumentInputV0],
    pub(crate) package_manifests: &'a [OmenaQueryStylePackageManifestV0],
    pub(crate) external_sifs: &'a [OmenaQueryExternalSifInputV0],
    pub(crate) resolution_inputs: &'a OmenaQueryStyleResolutionInputsV0,
    pub(crate) severity: u8,
    pub(crate) deep_analysis: bool,
}

/// One dependency row of the recorded read-set.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DiskDiagnosticsCacheDepV0 {
    pub(crate) path: String,
    pub(crate) content_hash: String,
}

/// Per-wave verification plan, built ONCE per wave (or per serial resolve):
/// the environment fingerprint plus a content-hash map over every style and
/// source input. Stage 1 re-serialized the full surface per target; this is
/// the O(corpus)-once replacement that both `load` verification and `store`
/// manifest construction read from.
#[derive(Debug, Clone)]
pub(crate) struct DiskDiagnosticsCacheWavePlanV0 {
    environment_fingerprint: String,
    content_hash_by_path: std::sync::Arc<std::collections::BTreeMap<String, String>>,
}

pub(crate) fn disk_diagnostics_cache_wave_plan_v1(
    components: &DiskDiagnosticsCacheEnvironmentComponentsV1<'_>,
) -> Option<DiskDiagnosticsCacheWavePlanV0> {
    let mut style_paths = components
        .style_sources
        .iter()
        .map(|source| source.style_path.as_str())
        .collect::<Vec<_>>();
    style_paths.sort_unstable();
    let mut source_paths = components
        .source_documents
        .iter()
        .map(|document| document.source_path.as_str())
        .collect::<Vec<_>>();
    source_paths.sort_unstable();
    let mut content_hash_by_path = std::collections::BTreeMap::new();
    for source in components.style_sources {
        content_hash_by_path.insert(
            source.style_path.clone(),
            disk_diagnostics_content_hash(source.style_source.as_bytes()),
        );
    }
    for document in components.source_documents {
        content_hash_by_path.insert(
            document.source_path.clone(),
            disk_diagnostics_content_hash(document.source_source.as_bytes()),
        );
    }
    // Two classes of resolution facts are DERIVED from corpus members whose
    // content hashes are already manifest rows — a strictly stronger guard —
    // so they must not re-enter the environment fingerprint, where a single
    // member's save would invalidate the whole store:
    //  - disk candidate identities (length+mtime snapshots of member files);
    //  - bridge SIFs GENERATED from member files (chain-import targets):
    //    their bytes embed the member's content, and every compute that
    //    reads such a SIF reaches the member through an import edge, so the
    //    member sits in that compute's recorded read-set.
    // Only facts about the world OUTSIDE the corpora stay environmental.
    let resolution_inputs = OmenaQueryStyleResolutionInputsV0 {
        disk_style_path_identities: components
            .resolution_inputs
            .disk_style_path_identities
            .iter()
            .filter(|identity| {
                !disk_diagnostics_corpus_contains_path(
                    &content_hash_by_path,
                    identity.style_path.as_str(),
                )
            })
            .cloned()
            .collect(),
        ..components.resolution_inputs.clone()
    };
    let environmental_external_sifs = components
        .external_sifs
        .iter()
        .filter(|input| {
            !disk_diagnostics_corpus_contains_path(
                &content_hash_by_path,
                input.sif.canonical_url.as_str(),
            ) && !disk_diagnostics_corpus_contains_path(
                &content_hash_by_path,
                input.canonical_url.as_str(),
            )
        })
        .cloned()
        .collect::<Vec<_>>();
    let input = DiskDiagnosticsCacheEnvironmentInputV1 {
        cache_schema_version: DISK_DIAGNOSTICS_CACHE_SCHEMA_VERSION_V1,
        crate_version: env!("CARGO_PKG_VERSION"),
        diagnostics_arm: DISK_DIAGNOSTICS_CACHE_ARM_V0,
        style_paths,
        source_paths,
        package_manifests: components.package_manifests,
        external_sifs: environmental_external_sifs.as_slice(),
        resolution_inputs: &resolution_inputs,
        severity: components.severity,
        deep_analysis: components.deep_analysis,
    };
    let canonical_bytes = write_omena_canonical_json_bytes_v1(&input).ok()?;
    let environment_fingerprint = compute_omena_sif_leaf_hash_v1(canonical_bytes.as_slice())
        .as_str()
        .to_string();
    Some(DiskDiagnosticsCacheWavePlanV0 {
        environment_fingerprint,
        content_hash_by_path: std::sync::Arc::new(content_hash_by_path),
    })
}

/// Corpus membership across the two path vocabularies in play: corpus maps
/// are keyed by `file://` URIs while resolver disk identities carry plain
/// filesystem paths.
fn disk_diagnostics_corpus_contains_path(
    content_hash_by_path: &std::collections::BTreeMap<String, String>,
    path: &str,
) -> bool {
    if content_hash_by_path.contains_key(path) {
        return true;
    }
    if let Some(stripped) = path.strip_prefix("file://") {
        return content_hash_by_path.contains_key(stripped);
    }
    content_hash_by_path.contains_key(format!("file://{path}").as_str())
}

fn disk_diagnostics_content_hash(bytes: &[u8]) -> String {
    compute_omena_sif_leaf_hash_v1(bytes).as_str().to_string()
}

/// The stable shard address: target identity only — never content — so a
/// target overwrites its own shard in place across edits and rebuilds (one
/// file per target; the stage-1 accumulation of dead content-keyed shards
/// cannot recur).
fn disk_diagnostics_stable_shard_address_v1(target_style_path: &str) -> Option<String> {
    let input = json!({
        "cacheSchemaVersion": DISK_DIAGNOSTICS_CACHE_SCHEMA_VERSION_V1,
        "diagnosticsArm": DISK_DIAGNOSTICS_CACHE_ARM_V0,
        "targetStylePath": target_style_path,
    });
    let canonical_bytes = write_omena_canonical_json_bytes_v1(&input).ok()?;
    Some(
        compute_omena_sif_leaf_hash_v1(canonical_bytes.as_slice())
            .as_str()
            .to_string(),
    )
}

impl DiskDiagnosticsCacheWavePlanV0 {
    #[cfg(test)]
    pub(crate) fn environment_fingerprint_for_test(&self) -> &str {
        self.environment_fingerprint.as_str()
    }
}

/// Serial-arm slot construction: the per-resolve analog of the wave plan —
/// ONE environment fingerprint + content-hash pass covering this resolve.
pub(crate) fn disk_diagnostics_cache_slot_for_serial_resolve(
    state: &LspShellState,
    workspace_folder_uri: Option<&str>,
    target_style_path: &str,
    components: &DiskDiagnosticsCacheEnvironmentComponentsV1<'_>,
) -> Option<DiskDiagnosticsCacheSlotV0> {
    let plan = disk_diagnostics_cache_wave_plan_v1(components)?;
    disk_diagnostics_cache_slot_for_resolve(state, workspace_folder_uri, target_style_path, &plan)
}

/// Serial-arm write-behind: declare the read-set over the committed
/// summary's edges, then store. Without a summary (the straight-line arm)
/// there is no sound manifest, so nothing is stored — fail-soft, not
/// fail-broad. Io errors are swallowed and a session breaker stops
/// retrying hot.
pub(crate) fn store_disk_diagnostics_shard_for_serial_resolve(
    state: &LspShellState,
    slot: Option<DiskDiagnosticsCacheSlotV0>,
    committed_cross_file_summary: Option<&omena_query::OmenaQueryCrossFileSummaryV0>,
    target_style_path: &str,
    diagnostics: &Value,
) {
    let (Some(mut slot), Some(summary)) = (slot, committed_cross_file_summary) else {
        return;
    };
    let read_set_index =
        omena_query::reverse_dependency_index_from_edges_v0(summary.edges.as_slice());
    slot.set_read_set_paths(omena_query::diagnostics_read_set_for_target_v0(
        &read_set_index,
        target_style_path,
    ));
    slot.store_write_behind(state, diagnostics);
}

/// One resolved cache placement: directory + stable address + the wave plan
/// it verifies against. Built before the workspace diagnostics compute;
/// `load` is the manifest-verified read path. The read-set is attached
/// AFTER compute via [`Self::set_read_set_paths`] — a slot without one
/// never stores (a shard without a manifest could only be membership-
/// guarded, which is not sound to serve).
#[derive(Debug, Clone)]
pub(crate) struct DiskDiagnosticsCacheSlotV0 {
    dir: PathBuf,
    address: String,
    target_style_path: String,
    environment_fingerprint: String,
    content_hash_by_path: std::sync::Arc<std::collections::BTreeMap<String, String>>,
    read_set_paths: Option<Vec<String>>,
}

impl DiskDiagnosticsCacheSlotV0 {
    pub(crate) fn load(&self) -> Option<Value> {
        load_disk_diagnostics_shard_with_limits(
            self,
            &DiskDiagnosticsCacheLimitsV0::with_defaults(),
        )
    }

    /// Declare the read-set discovered by the compute this slot caches. Paths
    /// outside the plan's corpora (external facts) are dropped — they are
    /// covered by the environment fingerprint, not per-file hashes.
    pub(crate) fn set_read_set_paths(&mut self, paths: impl IntoIterator<Item = String>) {
        let paths = paths
            .into_iter()
            .filter(|path| self.content_hash_by_path.contains_key(path.as_str()))
            .collect::<Vec<_>>();
        self.read_set_paths = Some(paths);
    }

    pub(crate) fn store_write_behind(&self, state: &LspShellState, diagnostics: &Value) {
        store_disk_diagnostics_shard_with_limits(
            &state.disk_diagnostics_cache_session,
            self,
            diagnostics,
            &DiskDiagnosticsCacheLimitsV0::with_defaults(),
        );
    }
}

/// Self-ignore markers for the workspace-relative cache root (`.cache/omena`):
/// a `.gitignore` containing `*` (the `.next/.gitignore` pattern — git and
/// git-driven tooling skip the tree without touching any user config) and a
/// standard `CACHEDIR.TAG` (archivers/backup tools skip tagged directories).
/// Written once, next to whichever cache subsystem touches the root first.
pub(crate) fn ensure_omena_cache_root_markers(cache_subdir: &Path) {
    let Some(omena_root) = cache_subdir.parent() else {
        return;
    };
    let gitignore = omena_root.join(".gitignore");
    if !gitignore.exists() {
        let _ = fs::write(
            gitignore,
            "# machine-generated omena cache - safe to delete\n*\n",
        );
    }
    let cachedir_tag = omena_root.join("CACHEDIR.TAG");
    if !cachedir_tag.exists() {
        let _ = fs::write(
            cachedir_tag,
            "Signature: 8a477f597d28d172789f06886806bc55\n# This directory is an omena cache; contents are regenerable.\n",
        );
    }
}

pub(crate) fn disk_diagnostics_cache_slot_for_resolve(
    state: &LspShellState,
    workspace_folder_uri: Option<&str>,
    target_style_path: &str,
    plan: &DiskDiagnosticsCacheWavePlanV0,
) -> Option<DiskDiagnosticsCacheSlotV0> {
    let dir = disk_diagnostics_cache_dir(state, workspace_folder_uri)?;
    let address = disk_diagnostics_stable_shard_address_v1(target_style_path)?;
    Some(DiskDiagnosticsCacheSlotV0 {
        dir,
        address,
        target_style_path: target_style_path.to_string(),
        environment_fingerprint: plan.environment_fingerprint.clone(),
        content_hash_by_path: std::sync::Arc::clone(&plan.content_hash_by_path),
        read_set_paths: None,
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
    Some(root.join(DISK_DIAGNOSTICS_CACHE_RELATIVE_DIR_V1))
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

pub(crate) fn disk_diagnostics_cache_oracle_engaged() -> bool {
    std::env::var(DISK_DIAGNOSTICS_CACHE_ORACLE_ENV)
        .is_ok_and(|value| value == "1" || value.eq_ignore_ascii_case("on"))
}

static DISK_DIAGNOSTICS_CACHE_ORACLE_HITS: std::sync::atomic::AtomicU64 =
    std::sync::atomic::AtomicU64::new(0);
static DISK_DIAGNOSTICS_CACHE_ORACLE_MISMATCHES: std::sync::atomic::AtomicU64 =
    std::sync::atomic::AtomicU64::new(0);

/// Shadow-oracle telemetry: a verified hit that is NOT byte-identical to the
/// recompute means the recorded read-set under-approximates some diagnostics
/// arm's true read window — a completeness bug to widen, never to ship past.
pub(crate) fn record_disk_diagnostics_cache_oracle_outcome(target_uri: &str, matched: bool) {
    if matched {
        DISK_DIAGNOSTICS_CACHE_ORACLE_HITS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        return;
    }
    DISK_DIAGNOSTICS_CACHE_ORACLE_MISMATCHES.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    crate::loop_trace!("disk-cache-oracle MISMATCH target={target_uri}");
}

#[cfg(test)]
pub(crate) fn disk_diagnostics_cache_oracle_counts() -> (u64, u64) {
    (
        DISK_DIAGNOSTICS_CACHE_ORACLE_HITS.load(std::sync::atomic::Ordering::Relaxed),
        DISK_DIAGNOSTICS_CACHE_ORACLE_MISMATCHES.load(std::sync::atomic::Ordering::Relaxed),
    )
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
    slot: &DiskDiagnosticsCacheSlotV0,
    limits: &DiskDiagnosticsCacheLimitsV0,
) -> Option<Value> {
    let shard_path = disk_diagnostics_shard_file_path(slot.dir.as_path(), slot.address.as_str())?;
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
    if !disk_diagnostics_shard_identity_matches(&shard, slot) {
        // A foreign format, arm, build, or target at this address is dead
        // weight; a stable address means the overwrite would happen anyway.
        let _ = fs::remove_file(shard_path.as_path());
        return None;
    }
    if !disk_diagnostics_shard_trace_verifies(&shard, slot) {
        // A clean verification miss (content or environment moved on): the
        // shard is NOT deleted — the recompute overwrites it in place.
        return None;
    }
    shard.get_mut("diagnosticsJson").map(Value::take)
}

/// Structural identity: is this shard the CURRENT format, build, and target
/// for its address? Failing this is corruption or staleness worth deleting.
fn disk_diagnostics_shard_identity_matches(
    shard: &Value,
    slot: &DiskDiagnosticsCacheSlotV0,
) -> bool {
    shard.get("schemaVersion").and_then(Value::as_str)
        == Some(DISK_DIAGNOSTICS_CACHE_SCHEMA_VERSION_V1)
        && shard.get("product").and_then(Value::as_str)
            == Some(DISK_DIAGNOSTICS_CACHE_SHARD_PRODUCT_V0)
        && shard.get("diagnosticsArm").and_then(Value::as_str)
            == Some(DISK_DIAGNOSTICS_CACHE_ARM_V0)
        && shard.get("binaryFingerprint").and_then(Value::as_str)
            == Some(disk_diagnostics_cache_binary_fingerprint())
        && shard.get("targetStylePath").and_then(Value::as_str)
            == Some(slot.target_style_path.as_str())
        && shard
            .get("diagnosticsJson")
            .is_some_and(disk_diagnostics_payload_is_well_formed)
        && shard.get("outputDigest").and_then(Value::as_str)
            == shard
                .get("diagnosticsJson")
                .and_then(disk_diagnostics_output_digest)
                .as_deref()
}

/// The verifying trace itself: the recorded environment fingerprint must
/// equal the current one, and every recorded dependency's content hash must
/// equal the current surface's. The recorded manifest must include the
/// target — a manifest that never read its own target is not a plausible
/// trace. The stored `outputDigest` (checked in identity above) binds the
/// OUTPUT to what the writer computed, so bit-rot or truncation that still
/// parses misses instead of being served; it is an integrity check, not an
/// authentication mechanism — an actor who can write into `.cache/` already
/// controls the workspace.
fn disk_diagnostics_shard_trace_verifies(shard: &Value, slot: &DiskDiagnosticsCacheSlotV0) -> bool {
    if shard.get("environmentFingerprint").and_then(Value::as_str)
        != Some(slot.environment_fingerprint.as_str())
    {
        return false;
    }
    let Some(deps) = shard.get("depsManifest").and_then(Value::as_array) else {
        return false;
    };
    if deps.is_empty() || deps.len() > DISK_DIAGNOSTICS_CACHE_MAX_DEPS {
        return false;
    }
    let mut saw_target = false;
    for dep in deps {
        let (Some(path), Some(content_hash)) = (
            dep.get("path").and_then(Value::as_str),
            dep.get("contentHash").and_then(Value::as_str),
        ) else {
            return false;
        };
        if slot.content_hash_by_path.get(path).map(String::as_str) != Some(content_hash) {
            return false;
        }
        saw_target |= path == slot.target_style_path.as_str();
    }
    saw_target
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
    slot: &DiskDiagnosticsCacheSlotV0,
    diagnostics: &Value,
    limits: &DiskDiagnosticsCacheLimitsV0,
) {
    if session.borrow().writes_disabled() || !disk_diagnostics_payload_is_well_formed(diagnostics) {
        return;
    }
    // No declared read-set, or one that never read its own target: nothing
    // sound to record, so nothing is stored (fail-soft, not fail-broad).
    let Some(read_set_paths) = slot.read_set_paths.as_ref() else {
        return;
    };
    if read_set_paths.len() > DISK_DIAGNOSTICS_CACHE_MAX_DEPS
        || !read_set_paths
            .iter()
            .any(|path| path == slot.target_style_path.as_str())
    {
        return;
    }
    let deps_manifest = read_set_paths
        .iter()
        .filter_map(|path| {
            slot.content_hash_by_path
                .get(path.as_str())
                .map(|content_hash| DiskDiagnosticsCacheDepV0 {
                    path: path.clone(),
                    content_hash: content_hash.clone(),
                })
        })
        .collect::<Vec<_>>();
    let Some(output_digest) = disk_diagnostics_output_digest(diagnostics) else {
        return;
    };
    let shard = json!({
        "schemaVersion": DISK_DIAGNOSTICS_CACHE_SCHEMA_VERSION_V1,
        "product": DISK_DIAGNOSTICS_CACHE_SHARD_PRODUCT_V0,
        "diagnosticsArm": DISK_DIAGNOSTICS_CACHE_ARM_V0,
        "binaryFingerprint": disk_diagnostics_cache_binary_fingerprint(),
        "targetStylePath": slot.target_style_path.as_str(),
        "environmentFingerprint": slot.environment_fingerprint.as_str(),
        "depsManifest": deps_manifest,
        "outputDigest": output_digest,
        "diagnosticsJson": diagnostics,
    });
    let Ok(bytes) = serde_json::to_vec(&shard) else {
        return;
    };
    if bytes.len() as u64 > limits.max_shard_bytes {
        return;
    }
    remove_legacy_disk_diagnostics_cache_dir_once(slot.dir.as_path());
    match write_disk_diagnostics_shard_atomically(
        slot.dir.as_path(),
        slot.address.as_str(),
        bytes.as_slice(),
    ) {
        Err(_) => {
            session.borrow_mut().record_write_failure();
        }
        // Stable addresses overwrite in place; only a NEW shard file can
        // grow the store, so only creations pay the read_dir+stat sweep —
        // a cold wave stats the directory O(new targets) times, not
        // O(stores x shard cap).
        Ok(true) => enforce_disk_diagnostics_cache_caps(slot.dir.as_path(), limits),
        Ok(false) => {}
    }
}

/// The content-keyed stage-1 stores this crate has replaced with stable-
/// address stores. Their shards were dead weight on any input change and
/// the directories accumulated without bound; every directory here is
/// regenerable by contract.
const LEGACY_CACHE_DIR_NAMES: &[&str] = &[
    "diagnostics-cache-v0",
    "source-occurrence-index-v0",
    "source-document-index-v0",
    "source-type-fact-cache-v0",
    "style-symbol-occurrence-index-v0",
    "workspace-occurrence-shards-v0",
];

/// Best-effort, once per process: drop the abandoned content-keyed stores
/// next to this one.
fn remove_legacy_disk_diagnostics_cache_dir_once(current_dir: &Path) {
    static LEGACY_SWEEP: std::sync::Once = std::sync::Once::new();
    LEGACY_SWEEP.call_once(|| remove_legacy_cache_dirs(current_dir));
}

fn remove_legacy_cache_dirs(current_dir: &Path) {
    let Some(omena_root) = current_dir.parent() else {
        return;
    };
    for legacy_name in LEGACY_CACHE_DIR_NAMES {
        let legacy_dir = omena_root.join(legacy_name);
        if legacy_dir != current_dir && legacy_dir.is_dir() {
            let _ = fs::remove_dir_all(legacy_dir);
        }
    }
}

/// Stable shard address for ANY cache subsystem: IDENTITY parts only —
/// never content — hashed into the shard filename, so one logical entry
/// overwrites itself in place across content changes while the content key
/// stays INSIDE the shard as a load-verified field. The Tidepool rule,
/// shared by every sidecar (the content-keyed alternative grew without
/// bound: one new file per corpus state, no eviction).
pub(crate) fn stable_cache_shard_address(product: &str, identity_parts: &[&str]) -> Option<String> {
    let input = json!({
        "schemaVersion": "address-v1",
        "product": product,
        "identityParts": identity_parts,
    });
    let canonical_bytes = write_omena_canonical_json_bytes_v1(&input).ok()?;
    Some(
        compute_omena_sif_leaf_hash_v1(canonical_bytes.as_slice())
            .as_str()
            .to_string(),
    )
}

/// Returns whether the write CREATED a new shard file (as opposed to
/// overwriting one in place) — with stable addresses, only creations can
/// grow the store, so only creations pay the eviction sweep.
fn write_disk_diagnostics_shard_atomically(
    dir: &Path,
    key: &str,
    bytes: &[u8],
) -> std::io::Result<bool> {
    fs::create_dir_all(dir)?;
    ensure_omena_cache_root_markers(dir);
    let final_path = disk_diagnostics_shard_file_path(dir, key).ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::InvalidInput, "non-hex shard key")
    })?;
    let existed = final_path.is_file();
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
        // shard — that is success, not failure.
        if final_path.is_file() {
            return Ok(!existed);
        }
    }
    renamed.map(|_| !existed)
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

    #[test]
    fn cache_writes_stamp_self_ignore_markers_at_omena_root() -> Result<(), &'static str> {
        let base = temp_cache_dir("markers");
        let dir = base.join(DISK_DIAGNOSTICS_CACHE_RELATIVE_DIR_V1);
        fs::create_dir_all(dir.as_path()).map_err(|_| "create cache dir")?;
        ensure_omena_cache_root_markers(dir.as_path());
        let omena_root = dir.parent().ok_or("omena root")?;
        let gitignore =
            fs::read_to_string(omena_root.join(".gitignore")).map_err(|_| "read gitignore")?;
        assert!(
            gitignore.contains("*"),
            "gitignore must ignore the whole cache tree"
        );
        let tag =
            fs::read_to_string(omena_root.join("CACHEDIR.TAG")).map_err(|_| "read cachedir tag")?;
        assert!(tag.starts_with("Signature: 8a477f597d28d172789f06886806bc55"));
        // idempotent: second call must not error or duplicate
        ensure_omena_cache_root_markers(dir.as_path());
        let _ = fs::remove_dir_all(base.as_path());
        Ok(())
    }

    fn temp_cache_dir(suffix: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "omena-lsp-disk-cache-{suffix}-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(dir.as_path());
        dir
    }

    const FIXTURE_TARGET: &str = "file:///repo/src/App.module.scss";
    const FIXTURE_DEP: &str = "file:///repo/src/tokens.module.scss";
    const FIXTURE_UNRELATED: &str = "file:///repo/src/Other.module.scss";
    const FIXTURE_SOURCE: &str = "file:///repo/src/App.tsx";

    /// The verifying-trace fixture: a three-file style corpus where the
    /// target's recorded read-set covers the target, ONE dependency, and
    /// ONE source document — and deliberately NOT the unrelated file.
    struct TraceFixture {
        target_style_path: String,
        style_sources: Vec<OmenaQueryStyleSourceInputV0>,
        source_documents: Vec<OmenaQuerySourceDocumentInputV0>,
        package_manifests: Vec<OmenaQueryStylePackageManifestV0>,
        external_sifs: Vec<OmenaQueryExternalSifInputV0>,
        resolution_inputs: OmenaQueryStyleResolutionInputsV0,
        severity: u8,
        deep_analysis: bool,
        read_set: Vec<String>,
    }

    impl TraceFixture {
        fn base() -> Self {
            Self {
                target_style_path: FIXTURE_TARGET.to_string(),
                style_sources: vec![
                    OmenaQueryStyleSourceInputV0 {
                        style_path: FIXTURE_TARGET.to_string(),
                        style_source: ".btn { color: red; }".to_string(),
                    },
                    OmenaQueryStyleSourceInputV0 {
                        style_path: FIXTURE_DEP.to_string(),
                        style_source: "$brand: red;".to_string(),
                    },
                    OmenaQueryStyleSourceInputV0 {
                        style_path: FIXTURE_UNRELATED.to_string(),
                        style_source: ".other { color: green; }".to_string(),
                    },
                ],
                source_documents: vec![OmenaQuerySourceDocumentInputV0 {
                    source_path: FIXTURE_SOURCE.to_string(),
                    source_source: "import styles from './App.module.scss';".to_string(),
                    source_syntax_index: None,
                    has_unresolved_style_import: false,
                }],
                package_manifests: Vec::new(),
                external_sifs: Vec::new(),
                resolution_inputs: OmenaQueryStyleResolutionInputsV0::default(),
                severity: 2,
                deep_analysis: false,
                read_set: vec![
                    FIXTURE_TARGET.to_string(),
                    FIXTURE_DEP.to_string(),
                    FIXTURE_SOURCE.to_string(),
                ],
            }
        }

        fn plan(&self) -> Option<DiskDiagnosticsCacheWavePlanV0> {
            disk_diagnostics_cache_wave_plan_v1(&DiskDiagnosticsCacheEnvironmentComponentsV1 {
                style_sources: self.style_sources.as_slice(),
                source_documents: self.source_documents.as_slice(),
                package_manifests: self.package_manifests.as_slice(),
                external_sifs: self.external_sifs.as_slice(),
                resolution_inputs: &self.resolution_inputs,
                severity: self.severity,
                deep_analysis: self.deep_analysis,
            })
        }

        fn slot(&self, dir: &Path) -> Option<DiskDiagnosticsCacheSlotV0> {
            let plan = self.plan()?;
            let mut slot = DiskDiagnosticsCacheSlotV0 {
                dir: dir.to_path_buf(),
                address: disk_diagnostics_stable_shard_address_v1(self.target_style_path.as_str())?,
                target_style_path: self.target_style_path.clone(),
                environment_fingerprint: plan.environment_fingerprint.clone(),
                content_hash_by_path: std::sync::Arc::clone(&plan.content_hash_by_path),
                read_set_paths: None,
            };
            slot.set_read_set_paths(self.read_set.iter().cloned());
            Some(slot)
        }
    }

    fn store_fixture(
        fixture: &TraceFixture,
        dir: &Path,
        diagnostics: &Value,
    ) -> Result<DiskDiagnosticsCacheSlotV0, &'static str> {
        let slot = fixture.slot(dir).ok_or("slot")?;
        let session = RefCell::new(DiskDiagnosticsCacheSessionV0::default());
        store_disk_diagnostics_shard_with_limits(
            &session,
            &slot,
            diagnostics,
            &DiskDiagnosticsCacheLimitsV0::with_defaults(),
        );
        Ok(slot)
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
    fn stable_address_is_target_identity_only() -> Result<(), &'static str> {
        let dir = temp_cache_dir("address");
        let base = TraceFixture::base()
            .slot(dir.as_path())
            .ok_or("base slot")?;
        let mut edited = TraceFixture::base();
        edited.style_sources[0].style_source = ".btn { color: blue; }".to_string();
        let edited = edited.slot(dir.as_path()).ok_or("edited slot")?;
        assert_eq!(
            base.address, edited.address,
            "content edits must not move the shard address"
        );
        let mut other_target = TraceFixture::base();
        other_target.target_style_path = FIXTURE_UNRELATED.to_string();
        other_target.read_set = vec![FIXTURE_UNRELATED.to_string()];
        let other_target = other_target.slot(dir.as_path()).ok_or("other slot")?;
        assert_ne!(base.address, other_target.address);
        assert!(base.address.starts_with("blake3:"));
        Ok(())
    }

    #[test]
    fn environment_fingerprint_pins_membership_and_settings_not_content() -> Result<(), &'static str>
    {
        let base_fingerprint = TraceFixture::base()
            .plan()
            .ok_or("base plan")?
            .environment_fingerprint;

        // Content-only edits leave the environment fingerprint UNTOUCHED —
        // that is the whole point of the verifying trace.
        let mut content = TraceFixture::base();
        content.style_sources[2].style_source = ".other { color: hotpink; }".to_string();
        assert_eq!(
            content
                .plan()
                .ok_or("content plan")?
                .environment_fingerprint,
            base_fingerprint,
        );

        let mut fingerprints = BTreeSet::from([base_fingerprint]);
        let mut membership = TraceFixture::base();
        membership.style_sources.push(OmenaQueryStyleSourceInputV0 {
            style_path: "file:///repo/src/new.module.scss".to_string(),
            style_source: String::new(),
        });
        fingerprints.insert(
            membership
                .plan()
                .ok_or("membership plan")?
                .environment_fingerprint,
        );

        let mut source_membership = TraceFixture::base();
        source_membership
            .source_documents
            .push(OmenaQuerySourceDocumentInputV0 {
                source_path: "file:///repo/src/New.tsx".to_string(),
                source_source: String::new(),
                source_syntax_index: None,
                has_unresolved_style_import: false,
            });
        fingerprints.insert(
            source_membership
                .plan()
                .ok_or("source membership plan")?
                .environment_fingerprint,
        );

        let mut severity = TraceFixture::base();
        severity.severity = 1;
        fingerprints.insert(
            severity
                .plan()
                .ok_or("severity plan")?
                .environment_fingerprint,
        );

        let mut deep_analysis = TraceFixture::base();
        deep_analysis.deep_analysis = true;
        fingerprints.insert(
            deep_analysis
                .plan()
                .ok_or("deep analysis plan")?
                .environment_fingerprint,
        );

        let mut manifest = TraceFixture::base();
        manifest
            .package_manifests
            .push(OmenaQueryStylePackageManifestV0 {
                package_json_path: "file:///repo/package.json".to_string(),
                package_json_source: "{\"name\":\"repo\"}".to_string(),
            });
        fingerprints.insert(
            manifest
                .plan()
                .ok_or("manifest plan")?
                .environment_fingerprint,
        );

        let mut resolution = TraceFixture::base();
        resolution
            .resolution_inputs
            .package_manifests
            .push(OmenaQueryStylePackageManifestV0 {
                package_json_path: "file:///repo/packages/ui/package.json".to_string(),
                package_json_source: "{\"name\":\"ui\"}".to_string(),
            });
        fingerprints.insert(
            resolution
                .plan()
                .ok_or("resolution plan")?
                .environment_fingerprint,
        );

        let mut external_sif = TraceFixture::base();
        external_sif
            .external_sifs
            .push(fixture_external_sif().ok_or("external sif fixture")?);
        fingerprints.insert(
            external_sif
                .plan()
                .ok_or("external sif plan")?
                .environment_fingerprint,
        );

        assert_eq!(
            fingerprints.len(),
            8,
            "every membership / settings variation must move the fingerprint"
        );
        Ok(())
    }

    #[test]
    fn corpus_backed_bridge_sifs_do_not_move_the_environment_fingerprint()
    -> Result<(), &'static str> {
        let base_fingerprint = TraceFixture::base()
            .plan()
            .ok_or("base plan")?
            .environment_fingerprint;

        // A bridge SIF generated FROM a corpus member (chain-import target):
        // its bytes embed the member's content, which is already a manifest
        // row, so it must stay OUT of the environment fingerprint.
        let member_sif = |content: &[u8]| -> Option<OmenaQueryExternalSifInputV0> {
            let sif = omena_sif::OmenaSifV1::from_static_exports(
                FIXTURE_DEP,
                omena_sif::OmenaSifGeneratorV1 {
                    name: "bridge".to_string(),
                    version: "0.1.0".to_string(),
                    toolchain_id: "bridge@0.1.0".to_string(),
                },
                omena_sif::OmenaSifSourceV1 {
                    syntax: omena_sif::OmenaSifSourceSyntaxV1::Scss,
                },
                omena_sif::OmenaSifExportsV1 {
                    variables: Vec::new(),
                    mixins: Vec::new(),
                    functions: Vec::new(),
                    placeholders: Vec::new(),
                    forwards: Vec::new(),
                },
                Vec::new(),
                content,
            )
            .ok()?;
            Some(OmenaQueryExternalSifInputV0 {
                canonical_url: FIXTURE_DEP.to_string(),
                sif,
            })
        };
        let mut corpus_backed = TraceFixture::base();
        corpus_backed
            .external_sifs
            .push(member_sif(b"$brand: red;").ok_or("member sif")?);
        assert_eq!(
            corpus_backed
                .plan()
                .ok_or("corpus-backed plan")?
                .environment_fingerprint,
            base_fingerprint,
            "a member-derived SIF must not enter the environment fingerprint"
        );
        let mut corpus_backed_edited = TraceFixture::base();
        corpus_backed_edited
            .external_sifs
            .push(member_sif(b"$brand: blue;").ok_or("edited member sif")?);
        assert_eq!(
            corpus_backed_edited
                .plan()
                .ok_or("edited corpus-backed plan")?
                .environment_fingerprint,
            base_fingerprint,
        );

        // A TRUE external SIF stays environmental.
        let mut external = TraceFixture::base();
        external
            .external_sifs
            .push(fixture_external_sif().ok_or("external sif fixture")?);
        assert_ne!(
            external
                .plan()
                .ok_or("external plan")?
                .environment_fingerprint,
            base_fingerprint,
        );
        Ok(())
    }

    #[test]
    fn corpus_member_disk_identities_do_not_move_the_environment_fingerprint()
    -> Result<(), &'static str> {
        let base_fingerprint = TraceFixture::base()
            .plan()
            .ok_or("base plan")?
            .environment_fingerprint;

        // A corpus member's disk identity (length+mtime) is filtered out of
        // the fingerprint: its content hash already guards it, so a plain
        // SAVE (mtime move) must not invalidate the whole store.
        let identity_for =
            |path: &str, mtime: &str| omena_query::OmenaQueryStyleModuleDiskCandidateIdentityV0 {
                style_path: path.to_string(),
                metadata_identity: format!("file|len20|{mtime}"),
            };
        let mut member = TraceFixture::base();
        member.resolution_inputs.disk_style_path_identities =
            vec![identity_for("/repo/src/tokens.module.scss", "mtime1")];
        assert_eq!(
            member.plan().ok_or("member plan")?.environment_fingerprint,
            base_fingerprint,
        );
        let mut member_saved = TraceFixture::base();
        member_saved.resolution_inputs.disk_style_path_identities =
            vec![identity_for("/repo/src/tokens.module.scss", "mtime2")];
        assert_eq!(
            member_saved
                .plan()
                .ok_or("member saved plan")?
                .environment_fingerprint,
            base_fingerprint,
        );

        // A disk-only candidate OUTSIDE the corpora stays environmental:
        // both its presence and its identity move the fingerprint.
        let mut disk_only = TraceFixture::base();
        disk_only.resolution_inputs.disk_style_path_identities =
            vec![identity_for("/repo/src/disk-only.scss", "mtime1")];
        let disk_only_fingerprint = disk_only
            .plan()
            .ok_or("disk only plan")?
            .environment_fingerprint;
        assert_ne!(disk_only_fingerprint, base_fingerprint);
        let mut disk_only_saved = TraceFixture::base();
        disk_only_saved.resolution_inputs.disk_style_path_identities =
            vec![identity_for("/repo/src/disk-only.scss", "mtime2")];
        assert_ne!(
            disk_only_saved
                .plan()
                .ok_or("disk only saved plan")?
                .environment_fingerprint,
            disk_only_fingerprint,
        );
        Ok(())
    }

    #[test]
    fn roundtrip_hit_survives_edits_outside_the_read_set() -> Result<(), &'static str> {
        let dir = temp_cache_dir("outside-edit");
        let diagnostics = json!([
            {"code": "missingCustomProperty", "message": "unknown --brand",
             "range": {"start": {"line": 0, "character": 0}, "end": {"line": 0, "character": 1}}},
        ]);
        store_fixture(&TraceFixture::base(), dir.as_path(), &diagnostics)?;

        let same = TraceFixture::base()
            .slot(dir.as_path())
            .ok_or("same slot")?;
        assert_eq!(
            same.load(),
            Some(diagnostics.clone()),
            "identical corpus hits"
        );

        // THE stage-2 property: an edit to a file OUTSIDE the recorded
        // read-set (same membership) still hits.
        let mut outside = TraceFixture::base();
        outside.style_sources[2].style_source = ".other { color: rebeccapurple; }".to_string();
        let outside = outside.slot(dir.as_path()).ok_or("outside slot")?;
        assert_eq!(
            outside.load(),
            Some(diagnostics),
            "an unrelated file's content edit must not invalidate the shard"
        );
        Ok(())
    }

    #[test]
    fn edits_inside_the_read_set_miss_without_deleting_the_shard() -> Result<(), &'static str> {
        let dir = temp_cache_dir("inside-edit");
        let slot = store_fixture(&TraceFixture::base(), dir.as_path(), &json!([]))?;
        let shard_path = disk_diagnostics_shard_file_path(dir.as_path(), slot.address.as_str())
            .ok_or("shard path")?;
        assert!(shard_path.is_file());

        let mut dep_edit = TraceFixture::base();
        dep_edit.style_sources[1].style_source = "$brand: blue;".to_string();
        let dep_edit = dep_edit.slot(dir.as_path()).ok_or("dep slot")?;
        assert_eq!(dep_edit.load(), None, "recorded dependency edits must miss");
        assert!(
            shard_path.is_file(),
            "a clean verification miss leaves the shard for in-place overwrite"
        );

        let mut target_edit = TraceFixture::base();
        target_edit.style_sources[0].style_source = ".btn { color: blue; }".to_string();
        let target_edit = target_edit.slot(dir.as_path()).ok_or("target slot")?;
        assert_eq!(target_edit.load(), None, "target edits must miss");

        let mut source_edit = TraceFixture::base();
        source_edit.source_documents[0].source_source =
            "import other from './App.module.scss';".to_string();
        let source_edit = source_edit.slot(dir.as_path()).ok_or("source slot")?;
        assert_eq!(source_edit.load(), None, "recorded source edits must miss");
        Ok(())
    }

    #[test]
    fn membership_and_settings_changes_miss() -> Result<(), &'static str> {
        let dir = temp_cache_dir("membership");
        store_fixture(&TraceFixture::base(), dir.as_path(), &json!([]))?;

        let mut added = TraceFixture::base();
        added.style_sources.push(OmenaQueryStyleSourceInputV0 {
            style_path: "file:///repo/src/appeared.module.scss".to_string(),
            style_source: ".appeared {}".to_string(),
        });
        let added = added.slot(dir.as_path()).ok_or("added slot")?;
        assert_eq!(
            added.load(),
            None,
            "a new corpus member can change failed-specifier resolution"
        );

        let mut removed = TraceFixture::base();
        removed.style_sources.remove(2);
        let removed = removed.slot(dir.as_path()).ok_or("removed slot")?;
        assert_eq!(removed.load(), None, "membership shrink must miss");

        let mut severity = TraceFixture::base();
        severity.severity = 1;
        let severity = severity.slot(dir.as_path()).ok_or("severity slot")?;
        assert_eq!(severity.load(), None, "settings changes must miss");
        Ok(())
    }

    #[test]
    fn store_requires_a_read_set_that_includes_the_target() -> Result<(), &'static str> {
        let dir = temp_cache_dir("no-read-set");
        let session = RefCell::new(DiskDiagnosticsCacheSessionV0::default());
        let limits = DiskDiagnosticsCacheLimitsV0::with_defaults();

        let fixture = TraceFixture::base();
        let plan = fixture.plan().ok_or("plan")?;
        let undeclared = DiskDiagnosticsCacheSlotV0 {
            dir: dir.clone(),
            address: disk_diagnostics_stable_shard_address_v1(FIXTURE_TARGET).ok_or("address")?,
            target_style_path: FIXTURE_TARGET.to_string(),
            environment_fingerprint: plan.environment_fingerprint.clone(),
            content_hash_by_path: std::sync::Arc::clone(&plan.content_hash_by_path),
            read_set_paths: None,
        };
        store_disk_diagnostics_shard_with_limits(&session, &undeclared, &json!([]), &limits);
        assert!(
            !dir.exists(),
            "a slot without a declared read-set must not store"
        );

        let mut target_missing = TraceFixture::base();
        target_missing.read_set = vec![FIXTURE_DEP.to_string()];
        let target_missing = target_missing.slot(dir.as_path()).ok_or("slot")?;
        store_disk_diagnostics_shard_with_limits(&session, &target_missing, &json!([]), &limits);
        assert!(
            !dir.exists(),
            "a read-set that never read its own target is not a plausible trace"
        );
        Ok(())
    }

    #[test]
    fn read_set_paths_outside_the_corpora_are_dropped_from_the_manifest() -> Result<(), &'static str>
    {
        let dir = temp_cache_dir("foreign-paths");
        let mut fixture = TraceFixture::base();
        fixture
            .read_set
            .push("https://cdn.example/tokens.scss".to_string());
        let slot = store_fixture(&fixture, dir.as_path(), &json!([]))?;
        let shard_path = disk_diagnostics_shard_file_path(dir.as_path(), slot.address.as_str())
            .ok_or("shard path")?;
        let shard: Value = serde_json::from_slice(
            fs::read(shard_path.as_path())
                .map_err(|_| "read shard")?
                .as_slice(),
        )
        .map_err(|_| "parse shard")?;
        let deps = shard
            .get("depsManifest")
            .and_then(Value::as_array)
            .ok_or("deps manifest")?;
        assert_eq!(
            deps.len(),
            3,
            "external facts are environment-fingerprinted, not per-dep entries"
        );
        Ok(())
    }

    #[test]
    fn garbage_and_stale_schema_shards_are_deleted_misses() -> Result<(), &'static str> {
        let dir = temp_cache_dir("garbage");
        let slot = TraceFixture::base().slot(dir.as_path()).ok_or("slot")?;
        let limits = DiskDiagnosticsCacheLimitsV0::with_defaults();
        let shard_path = disk_diagnostics_shard_file_path(dir.as_path(), slot.address.as_str())
            .ok_or("shard path")?;
        fs::create_dir_all(dir.as_path()).map_err(|_| "create cache dir")?;

        fs::write(shard_path.as_path(), b"{ truncated garbage").map_err(|_| "write garbage")?;
        assert_eq!(
            load_disk_diagnostics_shard_with_limits(&slot, &limits),
            None
        );
        assert!(!shard_path.exists(), "garbage shard must be deleted");

        let stale = json!({
            "schemaVersion": "999",
            "product": DISK_DIAGNOSTICS_CACHE_SHARD_PRODUCT_V0,
            "targetStylePath": FIXTURE_TARGET,
            "diagnosticsJson": [],
        });
        fs::write(
            shard_path.as_path(),
            serde_json::to_vec(&stale).map_err(|_| "serialize stale")?,
        )
        .map_err(|_| "write stale")?;
        assert_eq!(
            load_disk_diagnostics_shard_with_limits(&slot, &limits),
            None
        );
        assert!(!shard_path.exists(), "stale-schema shard must be deleted");
        Ok(())
    }

    #[test]
    fn oversized_shards_are_neither_written_nor_served() -> Result<(), &'static str> {
        let dir = temp_cache_dir("oversize");
        let fixture = TraceFixture::base();
        let slot = fixture.slot(dir.as_path()).ok_or("slot")?;
        let limits = DiskDiagnosticsCacheLimitsV0 {
            max_shard_bytes: 128,
            max_shards: 256,
            max_total_bytes: 64 * 1024 * 1024,
        };
        let session = RefCell::new(DiskDiagnosticsCacheSessionV0::default());
        let oversized = json!([
            {"message": "x".repeat(512), "range": {"start": {}, "end": {}}},
        ]);
        store_disk_diagnostics_shard_with_limits(&session, &slot, &oversized, &limits);
        let shard_path = disk_diagnostics_shard_file_path(dir.as_path(), slot.address.as_str())
            .ok_or("shard path")?;
        assert!(
            !shard_path.exists(),
            "oversized payload must not be written"
        );
        assert_eq!(session.borrow().write_failure_count(), 0);

        fs::create_dir_all(dir.as_path()).map_err(|_| "create cache dir")?;
        fs::write(shard_path.as_path(), vec![b'x'; 512]).map_err(|_| "write oversized")?;
        assert_eq!(
            load_disk_diagnostics_shard_with_limits(&slot, &limits),
            None
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
        let fixture = TraceFixture::base();
        let undeliverable_slot = fixture.slot(undeliverable_dir.as_path()).ok_or("slot")?;
        let limits = DiskDiagnosticsCacheLimitsV0::with_defaults();
        let session = RefCell::new(DiskDiagnosticsCacheSessionV0::default());

        for _ in 0..5 {
            store_disk_diagnostics_shard_with_limits(
                &session,
                &undeliverable_slot,
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
        let healthy_slot = fixture.slot(healthy_dir.as_path()).ok_or("healthy slot")?;
        store_disk_diagnostics_shard_with_limits(&session, &healthy_slot, &json!([]), &limits);
        assert!(
            !healthy_dir.exists(),
            "writes stay disabled for the session once the breaker trips",
        );
        Ok(())
    }

    #[test]
    fn caps_evict_oldest_shards_first_and_keep_the_newest() -> Result<(), &'static str> {
        let dir = temp_cache_dir("caps");
        let limits = DiskDiagnosticsCacheLimitsV0 {
            max_shard_bytes: 4 * 1024 * 1024,
            max_shards: 2,
            max_total_bytes: 64 * 1024 * 1024,
        };
        let session = RefCell::new(DiskDiagnosticsCacheSessionV0::default());
        let targets = [FIXTURE_TARGET, FIXTURE_DEP, FIXTURE_UNRELATED];
        let mut addresses = Vec::new();
        for target in targets {
            let mut fixture = TraceFixture::base();
            fixture.target_style_path = target.to_string();
            fixture.read_set = vec![target.to_string()];
            let slot = fixture.slot(dir.as_path()).ok_or("slot")?;
            addresses.push(slot.address.clone());
            store_disk_diagnostics_shard_with_limits(&session, &slot, &json!([]), &limits);
            std::thread::sleep(std::time::Duration::from_millis(10));
        }

        assert!(
            !disk_diagnostics_shard_file_path(dir.as_path(), addresses[0].as_str())
                .ok_or("evicted path")?
                .exists()
        );
        assert!(
            disk_diagnostics_shard_file_path(dir.as_path(), addresses[2].as_str())
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
    fn stores_overwrite_in_place_instead_of_accumulating() -> Result<(), &'static str> {
        let dir = temp_cache_dir("overwrite");
        store_fixture(&TraceFixture::base(), dir.as_path(), &json!([]))?;
        let mut edited = TraceFixture::base();
        edited.style_sources[0].style_source = ".btn { color: blue; }".to_string();
        let slot = store_fixture(&edited, dir.as_path(), &json!([]))?;
        let shard_files = fs::read_dir(dir.as_path())
            .map_err(|_| "read cache dir")?
            .flatten()
            .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "json"))
            .count();
        assert_eq!(
            shard_files, 1,
            "one target must own exactly one shard file across content edits"
        );
        assert!(slot.load().is_some(), "the overwrite serves the new trace");
        Ok(())
    }

    #[test]
    fn legacy_stage_one_store_is_swept() -> Result<(), &'static str> {
        let base = temp_cache_dir("legacy-sweep");
        let omena_root = base.join(".cache/omena");
        let legacy_dir = omena_root.join("diagnostics-cache-v0");
        let current_dir = omena_root.join("diagnostics-cache-v1");
        fs::create_dir_all(legacy_dir.as_path()).map_err(|_| "create legacy")?;
        fs::create_dir_all(current_dir.as_path()).map_err(|_| "create current")?;
        fs::write(legacy_dir.join("dead.json"), b"{}").map_err(|_| "write dead shard")?;
        remove_legacy_cache_dirs(current_dir.as_path());
        assert!(!legacy_dir.exists(), "the stage-1 store must be removed");
        assert!(current_dir.exists());
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
        let genuine = json!([
            {"message": "real", "range": {"start": {}, "end": {}}},
        ]);
        let slot = store_fixture(&TraceFixture::base(), dir.as_path(), &genuine)?;
        let shard_path =
            disk_diagnostics_shard_file_path(dir.as_path(), slot.address.as_str()).ok_or("path")?;
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
            slot.load(),
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
        let slot = store_fixture(&TraceFixture::base(), dir.as_path(), &json!([]))?;
        let shard_path =
            disk_diagnostics_shard_file_path(dir.as_path(), slot.address.as_str()).ok_or("path")?;
        let mut shard: Value = serde_json::from_slice(
            fs::read(shard_path.as_path())
                .map_err(|_| "read shard")?
                .as_slice(),
        )
        .map_err(|_| "parse shard")?;
        let malformed = json!(["not-an-object", 42]);
        shard["outputDigest"] =
            Value::String(disk_diagnostics_output_digest(&malformed).ok_or("digest")?);
        shard["diagnosticsJson"] = malformed;
        fs::write(
            shard_path.as_path(),
            serde_json::to_vec(&shard).map_err(|_| "serialize")?,
        )
        .map_err(|_| "write")?;

        assert_eq!(
            slot.load(),
            None,
            "elements without range+message must miss even when digested",
        );
        assert!(!shard_path.exists());
        Ok(())
    }

    #[test]
    fn oracle_counters_track_hits_and_mismatches() {
        let (hits_before, mismatches_before) = disk_diagnostics_cache_oracle_counts();
        record_disk_diagnostics_cache_oracle_outcome("file:///repo/src/App.module.scss", true);
        record_disk_diagnostics_cache_oracle_outcome("file:///repo/src/App.module.scss", false);
        let (hits_after, mismatches_after) = disk_diagnostics_cache_oracle_counts();
        assert_eq!(hits_after - hits_before, 1);
        assert_eq!(mismatches_after - mismatches_before, 1);
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
                "/omena-disk-cache-kill-switch-root/.cache/omena/diagnostics-cache-v1",
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
                "/omena-disk-cache-owner-root/.cache/omena/diagnostics-cache-v1",
            )),
        );
        // Orphan documents fall back to the first registered folder.
        assert_eq!(
            disk_diagnostics_cache_dir_with_kill_switch(&state, None, false),
            Some(PathBuf::from(
                "/omena-disk-cache-first-root/.cache/omena/diagnostics-cache-v1",
            )),
        );
    }
}
