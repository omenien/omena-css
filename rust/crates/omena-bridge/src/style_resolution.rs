use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    ffi::OsString,
    fs,
    path::{Component, Path, PathBuf},
    sync::{Mutex, OnceLock},
};

use crate::bundler_config_alias::load_omena_bridge_workspace_bundler_path_alias_mappings;
use omena_resolver::{
    OmenaResolverBundlerPathAliasMappingV0, OmenaResolverStyleModuleConfirmationOptionsV0,
    OmenaResolverStyleModuleDiskCandidateIdentityV0, OmenaResolverStylePackageManifestV0,
    OmenaResolverTsconfigPathMappingV0,
    collect_omena_resolver_style_module_source_candidates_with_path_mappings,
    confirm_omena_resolver_style_module_candidate_with_options,
    is_omena_resolver_indexable_style_module_path,
    normalize_omena_resolver_style_module_source_for_routing,
};
use omena_sif::{
    OmenaSifSourceSyntaxV1, OmenaSifStaticGeneratorInputV1, OmenaSifV1,
    compute_omena_sif_leaf_hash_v1, generate_static_omena_sif_v1, read_omena_sif_json_v1,
    write_omena_canonical_json_bytes_v1, write_omena_sif_json_v1,
};
use serde::Serialize;
use serde_json::{Value, json};

const WORKSPACE_PACKAGE_MANIFEST_SCAN_LIMIT: usize = 1024;
const EXTERNAL_SIF_CACHE_SCHEMA_VERSION: &str = "0";
const EXTERNAL_SIF_CACHE_PRODUCT: &str = "omena-bridge.external-sif-cache-shard";
const EXTERNAL_SIF_CACHE_RELATIVE_DIR: &str = ".cache/omena/external-sif-v0";
const EXTERNAL_SIF_CACHE_ENV_KILL_SWITCH: &str = "OMENA_BRIDGE_EXTERNAL_SIF_CACHE";
const EXTERNAL_SIF_CACHE_MAX_MEMORY_ENTRIES: usize = 256;
const EXTERNAL_SIF_CACHE_MAX_SHARDS: usize = 2048;
const EXTERNAL_SIF_CACHE_MAX_TOTAL_BYTES: u64 = 256 * 1024 * 1024;
const EXTERNAL_SIF_CACHE_MAX_SHARD_BYTES: u64 = 8 * 1024 * 1024;
const WORKSPACE_STYLE_PATH_IDENTITY_SCAN_LIMIT: usize = 4096;
const WORKSPACE_STYLE_PATH_IDENTITY_MAX_DEPTH: usize = 8;

static EXTERNAL_SIF_MEMORY_CACHE: OnceLock<Mutex<BTreeMap<String, OmenaSifV1>>> = OnceLock::new();

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaBridgeStyleResolutionSummaryV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub owner_crate: &'static str,
    pub resolver_name: &'static str,
    pub supported_specifier_kinds: Vec<&'static str>,
    pub candidate_extensions: Vec<&'static str>,
    pub request_path_policy: Vec<&'static str>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaBridgeStyleResolutionInputsV0 {
    pub package_manifests: Vec<OmenaResolverStylePackageManifestV0>,
    pub tsconfig_path_mappings: Vec<OmenaResolverTsconfigPathMappingV0>,
    pub bundler_path_mappings: Vec<OmenaResolverBundlerPathAliasMappingV0>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub disk_style_path_identities: Vec<OmenaResolverStyleModuleDiskCandidateIdentityV0>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaBridgeExternalSifCacheContextV0 {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub freshness_fingerprint: Option<String>,
}

pub fn summarize_omena_bridge_style_resolution_boundary() -> OmenaBridgeStyleResolutionSummaryV0 {
    OmenaBridgeStyleResolutionSummaryV0 {
        schema_version: "0",
        product: "omena-bridge.style-resolution",
        owner_crate: "omena-bridge",
        resolver_name: "style-import-specifier-resolver",
        supported_specifier_kinds: vec![
            "relative",
            "tsconfigPaths",
            "jsconfigPaths",
            "bundlerAliases",
            "npmPackages",
            "packageImports",
        ],
        candidate_extensions: vec!["scss", "sass", "css", "less"],
        request_path_policy: vec![
            "resolverConsumesSourceUriWorkspaceUriAndRawSpecifier",
            "relativeSpecifierExpandsStyleModuleCandidates",
            "pathAliasResolutionUsesNearestWorkspaceTsconfigOrJsconfig",
            "pathAliasResolutionFollowsRelativeTsconfigExtends",
            "bundlerAliasResolutionUsesLiteralViteWebpackConfig",
            "packageSpecifierResolutionUsesOmenaResolver",
            "fileUriOutputIsPercentEncoded",
            "lspServerOwnsOnlyDocumentRoutingAndUriRangeMapping",
        ],
    }
}

pub fn resolve_omena_bridge_style_uri_for_specifier(
    source_uri: &str,
    workspace_folder_uri: Option<&str>,
    specifier: &str,
) -> Option<String> {
    resolve_omena_bridge_style_uri_for_specifier_with_package_manifests(
        source_uri,
        workspace_folder_uri,
        specifier,
        &[],
    )
}

pub fn resolve_omena_bridge_style_uri_for_specifier_with_package_manifests(
    source_uri: &str,
    workspace_folder_uri: Option<&str>,
    specifier: &str,
    configured_package_manifests: &[OmenaResolverStylePackageManifestV0],
) -> Option<String> {
    let source_path = normalize_path(file_uri_to_path(source_uri)?);
    let workspace_path = workspace_folder_uri
        .and_then(file_uri_to_path)
        .map(normalize_path);
    let package_manifests = merged_package_manifests_for_request(
        source_path.parent(),
        workspace_path.as_deref(),
        specifier,
        configured_package_manifests,
    );
    let inputs = OmenaBridgeStyleResolutionInputsV0 {
        package_manifests,
        tsconfig_path_mappings: tsconfig_path_mappings_for_workspace(workspace_path.as_deref())
            .unwrap_or_default(),
        bundler_path_mappings: load_omena_bridge_workspace_bundler_path_alias_mappings(
            workspace_path.as_deref(),
        ),
        disk_style_path_identities: workspace_path
            .as_deref()
            .map(workspace_style_path_identities)
            .unwrap_or_default(),
    };
    resolve_omena_bridge_style_uri_for_specifier_with_resolution_inputs(
        source_uri,
        workspace_folder_uri,
        specifier,
        &inputs,
    )
}

pub fn resolve_omena_bridge_style_uri_for_specifier_with_resolution_inputs(
    source_uri: &str,
    _workspace_folder_uri: Option<&str>,
    specifier: &str,
    resolution_inputs: &OmenaBridgeStyleResolutionInputsV0,
) -> Option<String> {
    let source_path = normalize_path(file_uri_to_path(source_uri)?);
    let source_path_text = source_path.to_string_lossy().to_string();
    let routing_specifier = normalize_omena_resolver_style_module_source_for_routing(specifier);
    let requires_existing_candidate = (package_name_from_specifier(routing_specifier).is_some()
        || is_package_import_specifier(routing_specifier))
        && !resolution_inputs
            .tsconfig_path_mappings
            .iter()
            .any(|mapping| {
                tsconfig_path_pattern_matches(mapping.pattern.as_str(), routing_specifier)
            })
        && !resolution_inputs
            .bundler_path_mappings
            .iter()
            .any(|mapping| {
                bundler_path_alias_pattern_matches(mapping.pattern.as_str(), routing_specifier)
            });
    let candidates = collect_omena_resolver_style_module_source_candidates_with_path_mappings(
        source_path_text.as_str(),
        specifier,
        resolution_inputs.package_manifests.as_slice(),
        resolution_inputs.bundler_path_mappings.as_slice(),
        resolution_inputs.tsconfig_path_mappings.as_slice(),
    );

    style_uri_for_resolver_candidates(
        candidates.as_slice(),
        resolution_inputs.disk_style_path_identities.as_slice(),
        requires_existing_candidate,
    )
}

/// Bridges the resolver→generator hop in-process: takes a resolved external
/// style module entry (the `file://` URI returned by
/// `resolve_omena_bridge_style_uri_for_specifier*`, or a plain filesystem
/// path) and produces an [`OmenaSifV1`] by reading the entry's source and
/// running the static SIF generator.
///
/// The returned SIF's `canonical_url` matches the resolved entry's `file://`
/// URI so the query layer can pair it against import targets. The CLI converts
/// each result into an `OmenaQueryExternalSifInputV0` without a JSON round-trip.
///
/// Errors gracefully (never panics) when the path is unresolvable, missing, or
/// unreadable.
pub fn generate_omena_bridge_sif_for_resolved_style_path(
    resolved_path: &str,
) -> Result<OmenaSifV1, String> {
    generate_omena_bridge_sif_for_resolved_style_path_with_cache_context(
        resolved_path,
        &OmenaBridgeExternalSifCacheContextV0::default(),
    )
}

pub fn generate_omena_bridge_sif_for_resolved_style_path_with_cache_context(
    resolved_path: &str,
    cache_context: &OmenaBridgeExternalSifCacheContextV0,
) -> Result<OmenaSifV1, String> {
    let raw_path = raw_resolved_style_entry_path(resolved_path)
        .ok_or_else(|| format!("unresolvable style module entry path: {resolved_path}"))?;
    let path = normalize_path(raw_path.clone());
    let canonical_url = path_to_file_uri(path.as_path());
    let source_bytes = fs::read(path.as_path()).map_err(|error| {
        format!(
            "failed to read resolved style module {}: {error}",
            path.to_string_lossy()
        )
    })?;
    let source_hash = compute_omena_sif_leaf_hash_v1(source_bytes.as_slice())
        .as_str()
        .to_string();
    let resolved_base_dir = path
        .parent()
        .map(|base_dir| base_dir.to_string_lossy().to_string())
        .unwrap_or_default();
    let cache_key = external_sif_cache_key(
        source_hash.as_str(),
        resolved_base_dir.as_str(),
        canonical_url.as_str(),
        cache_context.freshness_fingerprint.as_deref(),
    );
    if !external_sif_cache_kill_switch_engaged() {
        if let Some(sif) = load_external_sif_from_memory_cache(cache_key.as_str()) {
            return Ok(sif);
        }
        if let Some(cache_dir) = external_sif_cache_dir_for_path(raw_path.as_path())
            && let Some(sif) = load_external_sif_cache_shard(
                cache_dir.as_path(),
                cache_key.as_str(),
                canonical_url.as_str(),
                source_hash.as_str(),
                resolved_base_dir.as_str(),
            )
        {
            store_external_sif_in_memory_cache(cache_key.clone(), sif.clone());
            return Ok(sif);
        }
    }
    let source = String::from_utf8(source_bytes).map_err(|error| {
        format!(
            "failed to decode resolved style module {} as utf-8: {error}",
            path.to_string_lossy()
        )
    })?;
    let syntax = infer_omena_bridge_sif_source_syntax(path.as_path());
    let sif = generate_static_omena_sif_v1(OmenaSifStaticGeneratorInputV1 {
        canonical_url: canonical_url.as_str(),
        source: source.as_str(),
        syntax,
    })
    .map_err(|error| format!("failed to generate SIF for {canonical_url}: {error}"))?;
    if !external_sif_cache_kill_switch_engaged() {
        store_external_sif_in_memory_cache(cache_key.clone(), sif.clone());
        if let Some(cache_dir) = external_sif_cache_dir_for_path(raw_path.as_path()) {
            store_external_sif_cache_shard(
                cache_dir.as_path(),
                cache_key.as_str(),
                canonical_url.as_str(),
                source_hash.as_str(),
                resolved_base_dir.as_str(),
                &sif,
            );
        }
    }
    Ok(sif)
}

fn raw_resolved_style_entry_path(resolved_path: &str) -> Option<PathBuf> {
    let path = if resolved_path.starts_with("file://") {
        file_uri_to_path(resolved_path)?
    } else if resolved_path.is_empty() {
        return None;
    } else {
        PathBuf::from(resolved_path)
    };
    Some(normalize_path_lexical(path))
}

fn external_sif_cache_key(
    source_hash: &str,
    resolved_base_dir: &str,
    canonical_url: &str,
    freshness_fingerprint: Option<&str>,
) -> String {
    let input = json!({
        "schemaVersion": EXTERNAL_SIF_CACHE_SCHEMA_VERSION,
        "product": "omena-bridge.external-sif-cache-key",
        "crateVersion": env!("CARGO_PKG_VERSION"),
        "sourceHash": source_hash,
        "resolvedBaseDir": resolved_base_dir,
        "canonicalUrl": canonical_url,
        "freshnessFingerprint": freshness_fingerprint,
    });
    write_omena_canonical_json_bytes_v1(&input)
        .map(|bytes| {
            compute_omena_sif_leaf_hash_v1(bytes.as_slice())
                .as_str()
                .to_string()
        })
        .unwrap_or_else(|_| {
            compute_omena_sif_leaf_hash_v1(
                format!(
                    "{source_hash}\0{resolved_base_dir}\0{canonical_url}\0{}",
                    freshness_fingerprint.unwrap_or("")
                )
                .as_bytes(),
            )
            .as_str()
            .to_string()
        })
}

fn load_external_sif_from_memory_cache(key: &str) -> Option<OmenaSifV1> {
    EXTERNAL_SIF_MEMORY_CACHE
        .get_or_init(|| Mutex::new(BTreeMap::new()))
        .lock()
        .ok()?
        .get(key)
        .cloned()
}

fn store_external_sif_in_memory_cache(key: String, sif: OmenaSifV1) {
    let Ok(mut cache) = EXTERNAL_SIF_MEMORY_CACHE
        .get_or_init(|| Mutex::new(BTreeMap::new()))
        .lock()
    else {
        return;
    };
    cache.insert(key, sif);
    while cache.len() > EXTERNAL_SIF_CACHE_MAX_MEMORY_ENTRIES {
        let Some(first_key) = cache.keys().next().cloned() else {
            break;
        };
        cache.remove(first_key.as_str());
    }
}

fn external_sif_cache_dir_for_path(path: &Path) -> Option<PathBuf> {
    external_sif_cache_workspace_root(path).map(|root| root.join(EXTERNAL_SIF_CACHE_RELATIVE_DIR))
}

fn external_sif_cache_workspace_root(path: &Path) -> Option<PathBuf> {
    let mut current = path.parent();
    while let Some(dir) = current {
        if dir.file_name().and_then(|value| value.to_str()) == Some("node_modules") {
            return dir.parent().map(Path::to_path_buf);
        }
        current = dir.parent();
    }

    let mut current = path.parent();
    while let Some(dir) = current {
        if dir.join("package.json").is_file() {
            return Some(dir.to_path_buf());
        }
        current = dir.parent();
    }

    path.parent().map(Path::to_path_buf)
}

fn external_sif_cache_shard_file_path(dir: &Path, key: &str) -> Option<PathBuf> {
    let hex = key.strip_prefix("blake3:")?;
    if hex.is_empty() || !hex.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return None;
    }
    Some(dir.join(format!("{hex}.json")))
}

fn load_external_sif_cache_shard(
    dir: &Path,
    key: &str,
    canonical_url: &str,
    source_hash: &str,
    resolved_base_dir: &str,
) -> Option<OmenaSifV1> {
    let shard_path = external_sif_cache_shard_file_path(dir, key)?;
    let metadata = fs::metadata(shard_path.as_path()).ok()?;
    if !metadata.is_file() || metadata.len() > EXTERNAL_SIF_CACHE_MAX_SHARD_BYTES {
        let _ = fs::remove_file(shard_path.as_path());
        return None;
    }
    let bytes = fs::read(shard_path.as_path()).ok()?;
    let shard = serde_json::from_slice::<Value>(bytes.as_slice()).ok()?;
    if !external_sif_cache_shard_matches(&shard, key, canonical_url, source_hash, resolved_base_dir)
    {
        let _ = fs::remove_file(shard_path.as_path());
        return None;
    }
    let sif_json = shard.get("sifJson")?.as_str()?;
    read_omena_sif_json_v1(sif_json).ok()
}

fn external_sif_cache_shard_matches(
    shard: &Value,
    key: &str,
    canonical_url: &str,
    source_hash: &str,
    resolved_base_dir: &str,
) -> bool {
    shard.get("schemaVersion").and_then(Value::as_str) == Some(EXTERNAL_SIF_CACHE_SCHEMA_VERSION)
        && shard.get("product").and_then(Value::as_str) == Some(EXTERNAL_SIF_CACHE_PRODUCT)
        && shard.get("key").and_then(Value::as_str) == Some(key)
        && shard.get("canonicalUrl").and_then(Value::as_str) == Some(canonical_url)
        && shard.get("sourceHash").and_then(Value::as_str) == Some(source_hash)
        && shard.get("resolvedBaseDir").and_then(Value::as_str) == Some(resolved_base_dir)
        && shard.get("payloadDigest").and_then(Value::as_str)
            == shard
                .get("sifJson")
                .and_then(Value::as_str)
                .map(|sif_json| compute_omena_sif_leaf_hash_v1(sif_json.as_bytes()))
                .as_ref()
                .map(|digest| digest.as_str())
}

fn store_external_sif_cache_shard(
    dir: &Path,
    key: &str,
    canonical_url: &str,
    source_hash: &str,
    resolved_base_dir: &str,
    sif: &OmenaSifV1,
) {
    let Ok(sif_json) = write_omena_sif_json_v1(sif) else {
        return;
    };
    let payload_digest = compute_omena_sif_leaf_hash_v1(sif_json.as_bytes())
        .as_str()
        .to_string();
    let shard = json!({
        "schemaVersion": EXTERNAL_SIF_CACHE_SCHEMA_VERSION,
        "product": EXTERNAL_SIF_CACHE_PRODUCT,
        "key": key,
        "canonicalUrl": canonical_url,
        "sourceHash": source_hash,
        "resolvedBaseDir": resolved_base_dir,
        "payloadDigest": payload_digest,
        "sifJson": sif_json,
    });
    let Ok(bytes) = write_omena_canonical_json_bytes_v1(&shard) else {
        return;
    };
    if bytes.len() as u64 > EXTERNAL_SIF_CACHE_MAX_SHARD_BYTES {
        return;
    }
    if write_external_sif_cache_shard_atomically(dir, key, bytes.as_slice()).is_ok() {
        enforce_external_sif_cache_caps(dir);
    }
}

fn write_external_sif_cache_shard_atomically(
    dir: &Path,
    key: &str,
    bytes: &[u8],
) -> std::io::Result<()> {
    fs::create_dir_all(dir)?;
    let final_path = external_sif_cache_shard_file_path(dir, key).ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "invalid external SIF cache key",
        )
    })?;
    let temporary_path = final_path.with_extension(format!("tmp-{}", std::process::id()));
    fs::write(temporary_path.as_path(), bytes)?;
    let renamed = fs::rename(temporary_path.as_path(), final_path.as_path());
    if renamed.is_err() {
        let _ = fs::remove_file(temporary_path.as_path());
        if final_path.is_file() {
            return Ok(());
        }
    }
    renamed
}

fn enforce_external_sif_cache_caps(dir: &Path) {
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
    let mut total_bytes = shards.iter().map(|(_, bytes, _)| *bytes).sum::<u64>();
    let mut shard_count = shards.len();
    for (_, bytes, path) in shards {
        if shard_count <= 1
            || (shard_count <= EXTERNAL_SIF_CACHE_MAX_SHARDS
                && total_bytes <= EXTERNAL_SIF_CACHE_MAX_TOTAL_BYTES)
        {
            break;
        }
        if fs::remove_file(path.as_path()).is_ok() {
            shard_count -= 1;
            total_bytes = total_bytes.saturating_sub(bytes);
        }
    }
}

fn external_sif_cache_kill_switch_engaged() -> bool {
    std::env::var(EXTERNAL_SIF_CACHE_ENV_KILL_SWITCH)
        .is_ok_and(|value| value.eq_ignore_ascii_case("off") || value == "0" || value == "false")
}

#[cfg(test)]
fn clear_external_sif_memory_cache_for_test() {
    if let Some(cache) = EXTERNAL_SIF_MEMORY_CACHE.get()
        && let Ok(mut cache) = cache.lock()
    {
        cache.clear();
    }
}

fn infer_omena_bridge_sif_source_syntax(path: &Path) -> OmenaSifSourceSyntaxV1 {
    match path
        .extension()
        .and_then(|extension| extension.to_str())
        .map(str::to_ascii_lowercase)
        .as_deref()
    {
        Some("css") => OmenaSifSourceSyntaxV1::Css,
        Some("sass") => OmenaSifSourceSyntaxV1::Sass,
        _ => OmenaSifSourceSyntaxV1::Scss,
    }
}

pub fn load_omena_bridge_workspace_style_resolution_inputs(
    workspace_folder_uri: Option<&str>,
    configured_package_manifests: &[OmenaResolverStylePackageManifestV0],
) -> OmenaBridgeStyleResolutionInputsV0 {
    let workspace_path = workspace_folder_uri
        .and_then(file_uri_to_path)
        .map(normalize_path);
    load_omena_bridge_workspace_style_resolution_inputs_from_path(
        workspace_path.as_deref(),
        configured_package_manifests,
    )
}

fn load_omena_bridge_workspace_style_resolution_inputs_from_path(
    workspace_path: Option<&Path>,
    configured_package_manifests: &[OmenaResolverStylePackageManifestV0],
) -> OmenaBridgeStyleResolutionInputsV0 {
    OmenaBridgeStyleResolutionInputsV0 {
        package_manifests: merge_package_manifest_lists(
            configured_package_manifests,
            workspace_package_manifests(workspace_path).as_slice(),
        ),
        tsconfig_path_mappings: tsconfig_path_mappings_for_workspace(workspace_path)
            .unwrap_or_default(),
        bundler_path_mappings: load_omena_bridge_workspace_bundler_path_alias_mappings(
            workspace_path,
        ),
        disk_style_path_identities: workspace_path
            .map(workspace_style_path_identities)
            .unwrap_or_default(),
    }
}

fn workspace_style_path_identities(
    workspace_path: &Path,
) -> Vec<OmenaResolverStyleModuleDiskCandidateIdentityV0> {
    let mut identities = Vec::new();
    let mut queue = VecDeque::from([workspace_path.to_path_buf()]);
    while let Some(dir) = queue.pop_front() {
        if identities.len() >= WORKSPACE_STYLE_PATH_IDENTITY_SCAN_LIMIT {
            break;
        }
        let Ok(entries) = fs::read_dir(dir.as_path()) else {
            continue;
        };
        for entry in entries.flatten() {
            if identities.len() >= WORKSPACE_STYLE_PATH_IDENTITY_SCAN_LIMIT {
                break;
            }
            let path = entry.path();
            let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
                continue;
            };
            if path.is_dir() {
                let relative_depth = path
                    .strip_prefix(workspace_path)
                    .ok()
                    .map(|relative| relative.components().count())
                    .unwrap_or(usize::MAX);
                if relative_depth > WORKSPACE_STYLE_PATH_IDENTITY_MAX_DEPTH {
                    continue;
                }
                if should_skip_style_identity_scan_dir(file_name) {
                    continue;
                }
                queue.push_back(path);
                continue;
            }
            if !is_indexable_style_path(path.as_path()) {
                continue;
            }
            let Some(metadata_identity) = file_metadata_identity(path.as_path()) else {
                continue;
            };
            identities.push(OmenaResolverStyleModuleDiskCandidateIdentityV0 {
                style_path: normalize_path(path).to_string_lossy().to_string(),
                metadata_identity,
            });
        }
    }
    identities.sort_by(|left, right| left.style_path.cmp(&right.style_path));
    identities.dedup_by(|left, right| left.style_path == right.style_path);
    identities
}

fn should_skip_style_identity_scan_dir(name: &str) -> bool {
    matches!(
        name,
        ".git" | ".next" | ".nuxt" | ".svelte-kit" | "coverage" | "target"
    )
}

fn file_metadata_identity(path: &Path) -> Option<String> {
    let metadata = fs::symlink_metadata(path).ok()?;
    let modified = metadata
        .modified()
        .ok()
        .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|duration| format!("{}.{:09}", duration.as_secs(), duration.subsec_nanos()))
        .unwrap_or_else(|| "unknownMtime".to_string());
    let file_type = if metadata.file_type().is_symlink() {
        "symlink"
    } else if metadata.is_file() {
        "file"
    } else {
        "other"
    };
    Some(format!("{file_type}|len{}|mtime{modified}", metadata.len()))
}

fn merge_package_manifest_lists(
    primary: &[OmenaResolverStylePackageManifestV0],
    secondary: &[OmenaResolverStylePackageManifestV0],
) -> Vec<OmenaResolverStylePackageManifestV0> {
    let mut manifests = primary.to_vec();
    let mut seen = manifests
        .iter()
        .map(|manifest| manifest.package_json_path.clone())
        .collect::<BTreeSet<_>>();
    for manifest in secondary {
        if seen.insert(manifest.package_json_path.clone()) {
            manifests.push(manifest.clone());
        }
    }
    manifests
}

fn merged_package_manifests_for_specifier(
    source_dir: Option<&Path>,
    specifier: &str,
    configured_package_manifests: &[OmenaResolverStylePackageManifestV0],
) -> Vec<OmenaResolverStylePackageManifestV0> {
    merge_package_manifest_lists(
        configured_package_manifests,
        package_manifests_for_specifier(source_dir, specifier)
            .unwrap_or_default()
            .as_slice(),
    )
}

fn merged_package_manifests_for_request(
    source_dir: Option<&Path>,
    workspace_path: Option<&Path>,
    specifier: &str,
    configured_package_manifests: &[OmenaResolverStylePackageManifestV0],
) -> Vec<OmenaResolverStylePackageManifestV0> {
    let source_manifests =
        merged_package_manifests_for_specifier(source_dir, specifier, configured_package_manifests);
    merge_package_manifest_lists(
        source_manifests.as_slice(),
        workspace_package_manifests(workspace_path).as_slice(),
    )
}

fn tsconfig_path_mappings_for_workspace(
    workspace_path: Option<&Path>,
) -> Option<Vec<OmenaResolverTsconfigPathMappingV0>> {
    let workspace_path = workspace_path?;
    let mut mappings = Vec::new();
    for config_path in [
        workspace_path.join("tsconfig.json"),
        workspace_path.join("jsconfig.json"),
    ] {
        mappings.extend(tsconfig_path_mappings_for_config(config_path.as_path()));
    }
    Some(mappings)
}

fn tsconfig_path_mappings_for_config(
    config_path: &Path,
) -> Vec<OmenaResolverTsconfigPathMappingV0> {
    tsconfig_path_mappings_for_config_with_seen(config_path, &mut BTreeSet::new())
}

fn tsconfig_path_mappings_for_config_with_seen(
    config_path: &Path,
    seen: &mut BTreeSet<PathBuf>,
) -> Vec<OmenaResolverTsconfigPathMappingV0> {
    let normalized_config_path = normalize_path(config_path.to_path_buf());
    if !seen.insert(normalized_config_path.clone()) {
        return Vec::new();
    }
    let Some(config_text) = fs::read_to_string(config_path).ok() else {
        return Vec::new();
    };
    let Some(config) = serde_json::from_str::<Value>(config_text.as_str()).ok() else {
        return Vec::new();
    };
    let own_mappings = tsconfig_path_mappings_from_value(config_path, &config).unwrap_or_default();
    if !own_mappings.is_empty() {
        return own_mappings;
    }
    resolve_tsconfig_extends_path(config_path, &config)
        .map(|extends_path| {
            tsconfig_path_mappings_for_config_with_seen(extends_path.as_path(), seen)
        })
        .unwrap_or_default()
}

fn tsconfig_path_mappings_from_value(
    config_path: &Path,
    config: &Value,
) -> Option<Vec<OmenaResolverTsconfigPathMappingV0>> {
    let compiler_options = config.get("compilerOptions")?;
    let paths = compiler_options.get("paths")?.as_object()?;
    let config_dir = config_path.parent()?;
    let base_url = compiler_options
        .get("baseUrl")
        .and_then(Value::as_str)
        .unwrap_or(".");
    let base_path = normalize_path(config_dir.join(base_url));
    let mut mappings = Vec::new();
    for (pattern, targets) in paths {
        let Some(targets) = targets.as_array() else {
            continue;
        };
        let target_patterns = targets
            .iter()
            .filter_map(Value::as_str)
            .map(ToString::to_string)
            .collect::<Vec<_>>();
        if target_patterns.is_empty() {
            continue;
        }
        mappings.push(OmenaResolverTsconfigPathMappingV0 {
            base_path: base_path.to_string_lossy().to_string(),
            pattern: pattern.to_string(),
            target_patterns,
        });
    }
    Some(mappings)
}

fn resolve_tsconfig_extends_path(config_path: &Path, config: &Value) -> Option<PathBuf> {
    let extends = config.get("extends")?.as_str()?;
    if !extends.starts_with('.') {
        return None;
    }
    let config_dir = config_path.parent()?;
    let raw_path = config_dir.join(extends);
    tsconfig_extends_candidates(raw_path)
        .into_iter()
        .find(|candidate| candidate.exists())
}

fn tsconfig_extends_candidates(path: PathBuf) -> Vec<PathBuf> {
    if path.extension().is_some() {
        return vec![path];
    }
    vec![path.with_extension("json"), path.join("tsconfig.json")]
}

fn package_manifests_for_specifier(
    source_dir: Option<&Path>,
    specifier: &str,
) -> Option<Vec<OmenaResolverStylePackageManifestV0>> {
    if is_package_import_specifier(specifier) {
        return Some(package_scope_manifests_for_source_dir(source_dir));
    }
    let package_name = package_name_from_specifier(specifier)?;
    let mut manifests = Vec::new();
    let mut seen = BTreeSet::new();
    let mut current_dir = source_dir;
    while let Some(dir) = current_dir {
        let package_json_path = dir
            .join("node_modules")
            .join(package_name)
            .join("package.json");
        if seen.insert(package_json_path.clone())
            && let Ok(package_json_source) = fs::read_to_string(package_json_path.as_path())
        {
            manifests.push(OmenaResolverStylePackageManifestV0 {
                package_json_path: normalize_path(package_json_path)
                    .to_string_lossy()
                    .to_string(),
                package_json_source,
            });
        }
        current_dir = dir.parent();
    }
    Some(manifests)
}

fn package_scope_manifests_for_source_dir(
    source_dir: Option<&Path>,
) -> Vec<OmenaResolverStylePackageManifestV0> {
    let mut manifests = Vec::new();
    let mut current_dir = source_dir;
    while let Some(dir) = current_dir {
        push_workspace_package_manifest(dir.join("package.json"), &mut manifests);
        current_dir = dir.parent();
    }
    manifests
}

fn workspace_package_manifests(
    workspace_path: Option<&Path>,
) -> Vec<OmenaResolverStylePackageManifestV0> {
    let Some(workspace_path) = workspace_path else {
        return Vec::new();
    };
    let mut manifests = Vec::new();
    push_workspace_package_manifest(workspace_path.join("package.json"), &mut manifests);

    let node_modules = workspace_path.join("node_modules");
    let Ok(entries) = fs::read_dir(node_modules.as_path()) else {
        return manifests;
    };
    for entry in entries.flatten() {
        if manifests.len() >= WORKSPACE_PACKAGE_MANIFEST_SCAN_LIMIT {
            break;
        }
        let path = entry.path();
        let Some(file_name) = path.file_name().and_then(|value| value.to_str()) else {
            continue;
        };
        if file_name.starts_with('@') {
            push_scoped_workspace_package_manifests(path.as_path(), &mut manifests);
        } else {
            push_workspace_package_manifest(path.join("package.json"), &mut manifests);
        }
    }
    manifests.sort_by(|left, right| left.package_json_path.cmp(&right.package_json_path));
    manifests.dedup_by(|left, right| left.package_json_path == right.package_json_path);
    manifests
}

fn push_scoped_workspace_package_manifests(
    scope_path: &Path,
    manifests: &mut Vec<OmenaResolverStylePackageManifestV0>,
) {
    let Ok(entries) = fs::read_dir(scope_path) else {
        return;
    };
    for entry in entries.flatten() {
        if manifests.len() >= WORKSPACE_PACKAGE_MANIFEST_SCAN_LIMIT {
            return;
        }
        push_workspace_package_manifest(entry.path().join("package.json"), manifests);
    }
}

fn push_workspace_package_manifest(
    package_json_path: PathBuf,
    manifests: &mut Vec<OmenaResolverStylePackageManifestV0>,
) {
    if manifests.len() >= WORKSPACE_PACKAGE_MANIFEST_SCAN_LIMIT {
        return;
    }
    let normalized_package_json_path = normalize_path(package_json_path);
    let package_json_path_text = normalized_package_json_path.to_string_lossy().to_string();
    if manifests
        .iter()
        .any(|manifest| manifest.package_json_path == package_json_path_text)
    {
        return;
    }
    let Ok(package_json_source) = fs::read_to_string(normalized_package_json_path.as_path()) else {
        return;
    };
    manifests.push(OmenaResolverStylePackageManifestV0 {
        package_json_path: package_json_path_text,
        package_json_source,
    });
}

fn package_name_from_specifier(specifier: &str) -> Option<&str> {
    let specifier = specifier.strip_prefix("pkg:").unwrap_or(specifier);
    if specifier.starts_with('.')
        || specifier.starts_with('/')
        || is_package_import_specifier(specifier)
        || is_external_style_specifier(specifier)
    {
        return None;
    }
    if specifier.starts_with('@') {
        let mut segments = specifier.splitn(3, '/');
        let scope = segments.next()?;
        let package = segments.next()?;
        if scope.len() <= 1 || package.is_empty() {
            return None;
        }
        return specifier.get(..scope.len() + 1 + package.len());
    }
    specifier.split('/').next().filter(|name| !name.is_empty())
}

fn is_package_import_specifier(specifier: &str) -> bool {
    specifier
        .strip_prefix("pkg:")
        .unwrap_or(specifier)
        .starts_with('#')
}

fn tsconfig_path_pattern_matches(pattern: &str, specifier: &str) -> bool {
    if let Some((prefix, suffix)) = pattern.split_once('*') {
        return !suffix.contains('*')
            && specifier.starts_with(prefix)
            && specifier.ends_with(suffix)
            && specifier.len() >= prefix.len() + suffix.len();
    }
    pattern == specifier
}

fn bundler_path_alias_pattern_matches(pattern: &str, specifier: &str) -> bool {
    if pattern.is_empty() {
        return false;
    }
    if let Some(exact_pattern) = pattern.strip_suffix('$') {
        return specifier == exact_pattern;
    }
    if pattern == specifier {
        return true;
    }
    let prefix = if pattern.ends_with('/') {
        pattern.to_string()
    } else {
        format!("{pattern}/")
    };
    specifier.starts_with(prefix.as_str())
}

fn is_external_style_specifier(specifier: &str) -> bool {
    specifier.starts_with("sass:")
        || specifier.starts_with("http://")
        || specifier.starts_with("https://")
}

fn style_uri_for_resolver_candidates(
    candidates: &[String],
    disk_style_path_identities: &[OmenaResolverStyleModuleDiskCandidateIdentityV0],
    requires_existing_candidate: bool,
) -> Option<String> {
    let empty_available = BTreeSet::new();
    let confirmation = confirm_omena_resolver_style_module_candidate_with_options(
        candidates,
        &empty_available,
        disk_style_path_identities,
        OmenaResolverStyleModuleConfirmationOptionsV0 {
            allow_disk_confirmation: true,
            allow_live_disk_confirmation: true,
            allow_unconfirmed_indexable_candidate: !requires_existing_candidate,
            ..OmenaResolverStyleModuleConfirmationOptionsV0::default()
        },
    );
    confirmation
        .resolved_style_path
        .map(PathBuf::from)
        .map(|path| path_to_file_uri(normalize_path(path).as_path()))
}

fn is_indexable_style_path(path: &Path) -> bool {
    is_omena_resolver_indexable_style_module_path(path.to_string_lossy().as_ref())
}

fn file_uri_to_path(uri: &str) -> Option<PathBuf> {
    let raw_path = uri.strip_prefix("file://")?;
    Some(PathBuf::from(percent_decode_uri_path(raw_path)?))
}

fn percent_decode_uri_path(raw_path: &str) -> Option<String> {
    let bytes = raw_path.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0usize;
    while index < bytes.len() {
        if bytes[index] == b'%' {
            let high = bytes.get(index + 1).and_then(|byte| hex_value(*byte))?;
            let low = bytes.get(index + 2).and_then(|byte| hex_value(*byte))?;
            decoded.push((high << 4) | low);
            index += 3;
        } else {
            decoded.push(bytes[index]);
            index += 1;
        }
    }
    String::from_utf8(decoded).ok()
}

fn hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

fn path_to_file_uri(path: &Path) -> String {
    let path = normalize_path(path.to_path_buf());
    format!(
        "file://{}",
        percent_encode_uri_path(path.to_string_lossy().as_ref())
    )
}

fn percent_encode_uri_path(path: &str) -> String {
    let mut encoded = String::with_capacity(path.len());
    for byte in path.as_bytes() {
        match *byte {
            b'A'..=b'Z'
            | b'a'..=b'z'
            | b'0'..=b'9'
            | b'-'
            | b'.'
            | b'_'
            | b'~'
            | b'/'
            | b'@'
            | b':'
            | b'!'
            | b'$'
            | b'&'
            | b'\''
            | b'*'
            | b'+'
            | b','
            | b';'
            | b'=' => encoded.push(*byte as char),
            _ => encoded.push_str(format!("%{byte:02X}").as_str()),
        }
    }
    encoded
}

fn normalize_path(path: PathBuf) -> PathBuf {
    if let Some(canonical) = canonicalize_existing_path_or_parent(path.as_path()) {
        return normalize_path_lexical(canonical);
    }
    normalize_path_lexical(path)
}

fn canonicalize_existing_path_or_parent(path: &Path) -> Option<PathBuf> {
    if let Ok(canonical) = fs::canonicalize(path) {
        return Some(canonical);
    }

    let mut current = path.to_path_buf();
    let mut suffix = Vec::<OsString>::new();
    while let Some(parent) = current.parent() {
        if let Some(file_name) = current.file_name() {
            suffix.push(file_name.to_os_string());
        }
        if let Ok(mut canonical_parent) = fs::canonicalize(parent) {
            for segment in suffix.iter().rev() {
                canonical_parent.push(segment);
            }
            return Some(canonical_parent);
        }
        current = parent.to_path_buf();
    }
    None
}

fn normalize_path_lexical(path: PathBuf) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            Component::Normal(_) | Component::RootDir | Component::Prefix(_) => {
                normalized.push(component.as_os_str());
            }
        }
    }
    normalized
}

#[cfg(test)]
mod tests {
    use std::{fs, time::SystemTime};

    use super::*;

    #[test]
    fn resolves_relative_style_candidates() -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_dir("omena_bridge_style_relative")?;
        let source = root.join("src/App.tsx");
        let style = root.join("src/Button.module.scss");
        fs::create_dir_all(
            source
                .parent()
                .ok_or_else(|| std::io::Error::other("parent"))?,
        )?;
        fs::write(&source, "")?;
        fs::write(&style, ".root {}")?;

        let uri = resolve_omena_bridge_style_uri_for_specifier(
            path_to_file_uri(source.as_path()).as_str(),
            Some(path_to_file_uri(root.as_path()).as_str()),
            "./Button.module.scss",
        );

        assert_eq!(
            uri.as_deref(),
            Some(path_to_file_uri(style.as_path()).as_str())
        );
        let _ = fs::remove_dir_all(root);
        Ok(())
    }

    #[test]
    fn generates_sif_for_resolved_relative_style_module() -> Result<(), Box<dyn std::error::Error>>
    {
        let root = temp_dir("omena_bridge_sif_resolved")?;
        let source = root.join("src/App.tsx");
        let style = root.join("src/theme.scss");
        fs::create_dir_all(
            style
                .parent()
                .ok_or_else(|| std::io::Error::other("parent"))?,
        )?;
        fs::write(&source, "")?;
        fs::write(&style, "$brand: #0af;\n@mixin focus-ring {}\n")?;

        let resolved = resolve_omena_bridge_style_uri_for_specifier(
            path_to_file_uri(source.as_path()).as_str(),
            Some(path_to_file_uri(root.as_path()).as_str()),
            "./theme.scss",
        )
        .ok_or_else(|| std::io::Error::other("resolution failed"))?;

        let sif = generate_omena_bridge_sif_for_resolved_style_path(resolved.as_str())?;

        assert_eq!(sif.canonical_url, resolved);
        assert_eq!(sif.source.syntax, OmenaSifSourceSyntaxV1::Scss);
        assert!(
            sif.exports
                .variables
                .iter()
                .any(|variable| variable.name == "$brand"),
            "expected $brand variable export, got {:?}",
            sif.exports.variables
        );
        assert!(
            sif.exports
                .mixins
                .iter()
                .any(|mixin| mixin.name == "focus-ring"),
            "expected focus-ring mixin export, got {:?}",
            sif.exports.mixins
        );
        // The produced SIF must round-trip through the exact JSON contract the
        // CLI's `read_external_sifs` consumes, proving it is a valid artifact.
        let json = omena_sif::write_omena_sif_json_v1(&sif)?;
        let parsed = omena_sif::read_omena_sif_json_v1(json.as_str())?;
        assert_eq!(parsed, sif);
        let _ = fs::remove_dir_all(root);
        Ok(())
    }

    #[test]
    fn generates_sif_from_plain_resolved_path() -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_dir("omena_bridge_sif_plain")?;
        let style = root.join("tokens.sass");
        fs::write(&style, "$gap: 8px\n")?;

        let sif =
            generate_omena_bridge_sif_for_resolved_style_path(style.to_string_lossy().as_ref())?;

        assert_eq!(sif.source.syntax, OmenaSifSourceSyntaxV1::Sass);
        let _ = fs::remove_dir_all(root);
        Ok(())
    }

    #[test]
    fn external_sif_cache_key_is_base_dir_sensitive_and_serves_fresh_sif()
    -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_dir("omena_bridge_external_sif_cache")?;
        let first_dir = root.join("node_modules/design-a");
        let second_dir = root.join("node_modules/design-b");
        fs::create_dir_all(first_dir.as_path())?;
        fs::create_dir_all(second_dir.as_path())?;
        fs::write(root.join("package.json"), r#"{"name":"workspace"}"#)?;
        let first_style = first_dir.join("tokens.scss");
        let second_style = second_dir.join("tokens.scss");
        let source = "$brand: #0af;\n";
        fs::write(first_style.as_path(), source)?;
        fs::write(second_style.as_path(), source)?;

        let first_path = normalize_path(first_style.clone());
        let second_path = normalize_path(second_style.clone());
        let source_hash = compute_omena_sif_leaf_hash_v1(source.as_bytes())
            .as_str()
            .to_string();
        let first_base_dir = first_path
            .parent()
            .ok_or_else(|| std::io::Error::other("first parent"))?
            .to_string_lossy()
            .to_string();
        let second_base_dir = second_path
            .parent()
            .ok_or_else(|| std::io::Error::other("second parent"))?
            .to_string_lossy()
            .to_string();
        let first_key = external_sif_cache_key(
            source_hash.as_str(),
            first_base_dir.as_str(),
            path_to_file_uri(first_path.as_path()).as_str(),
            None,
        );
        let second_key = external_sif_cache_key(
            source_hash.as_str(),
            second_base_dir.as_str(),
            path_to_file_uri(second_path.as_path()).as_str(),
            None,
        );
        assert_ne!(
            first_key, second_key,
            "same bytes under different resolved bases must not share an external SIF cache key"
        );
        let old_fingerprint_key = external_sif_cache_key(
            source_hash.as_str(),
            first_base_dir.as_str(),
            path_to_file_uri(first_path.as_path()).as_str(),
            Some("lockfile:old"),
        );
        let new_fingerprint_key = external_sif_cache_key(
            source_hash.as_str(),
            first_base_dir.as_str(),
            path_to_file_uri(first_path.as_path()).as_str(),
            Some("lockfile:new"),
        );
        assert_ne!(
            old_fingerprint_key, new_fingerprint_key,
            "lockfile or package-manager freshness changes must invalidate external SIF cache keys"
        );

        let first_uri = path_to_file_uri(first_style.as_path());
        let fresh = generate_omena_bridge_sif_for_resolved_style_path(first_uri.as_str())?;
        clear_external_sif_memory_cache_for_test();
        let cached = generate_omena_bridge_sif_for_resolved_style_path(first_uri.as_str())?;
        assert_eq!(cached, fresh);
        let cache_dir = external_sif_cache_dir_for_path(first_style.as_path())
            .ok_or_else(|| std::io::Error::other("cache dir"))?;
        assert!(
            cache_dir.read_dir()?.flatten().any(|entry| entry
                .path()
                .extension()
                .and_then(|ext| ext.to_str())
                == Some("json")),
            "expected a disk external SIF cache shard in {}",
            cache_dir.display()
        );
        let _ = fs::remove_dir_all(root);
        Ok(())
    }

    #[test]
    fn errors_gracefully_for_missing_resolved_style_module() {
        let missing = std::env::temp_dir().join("omena_bridge_sif_missing/does-not-exist.scss");
        let result =
            generate_omena_bridge_sif_for_resolved_style_path(missing.to_string_lossy().as_ref());
        assert!(result.is_err(), "expected error for missing entry");
    }

    #[test]
    fn errors_gracefully_for_empty_resolved_path() {
        let result = generate_omena_bridge_sif_for_resolved_style_path("");
        assert!(result.is_err(), "expected error for empty path");
    }

    #[test]
    fn resolves_tsconfig_path_alias_style_candidates() -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_dir("omena_bridge_style_alias")?;
        let source = root.join("src/App.tsx");
        let style = root.join("src/styles/Button.module.scss");
        fs::create_dir_all(
            style
                .parent()
                .ok_or_else(|| std::io::Error::other("parent"))?,
        )?;
        fs::write(&source, "")?;
        fs::write(&style, ".root {}")?;
        fs::write(
            root.join("tsconfig.json"),
            r#"{"compilerOptions":{"baseUrl":".","paths":{"@styles/*":["src/styles/*"]}}}"#,
        )?;

        let uri = resolve_omena_bridge_style_uri_for_specifier(
            path_to_file_uri(source.as_path()).as_str(),
            Some(path_to_file_uri(root.as_path()).as_str()),
            "@styles/Button.module.scss",
        );

        assert_eq!(
            uri.as_deref(),
            Some(path_to_file_uri(style.as_path()).as_str())
        );
        let _ = fs::remove_dir_all(root);
        Ok(())
    }

    #[test]
    fn resolves_tsconfig_extends_path_alias_style_candidates()
    -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_dir("omena_bridge_style_alias_extends")?;
        let source = root.join("src/App.tsx");
        let style = root.join("src/shared/Button.module.scss");
        let config_dir = root.join("config");
        fs::create_dir_all(
            style
                .parent()
                .ok_or_else(|| std::io::Error::other("parent"))?,
        )?;
        fs::create_dir_all(config_dir.as_path())?;
        fs::write(&source, "")?;
        fs::write(&style, ".root {}")?;
        fs::write(
            config_dir.join("base.json"),
            r#"{"compilerOptions":{"baseUrl":"..","paths":{"$shared/*":["src/shared/*"]}}}"#,
        )?;
        fs::write(root.join("tsconfig.json"), r#"{"extends":"./config/base"}"#)?;

        let uri = resolve_omena_bridge_style_uri_for_specifier(
            path_to_file_uri(source.as_path()).as_str(),
            Some(path_to_file_uri(root.as_path()).as_str()),
            "$shared/Button.module.scss",
        );

        assert_eq!(
            uri.as_deref(),
            Some(path_to_file_uri(style.as_path()).as_str())
        );
        let _ = fs::remove_dir_all(root);
        Ok(())
    }

    #[test]
    fn tsconfig_extends_child_paths_override_parent_paths() -> Result<(), Box<dyn std::error::Error>>
    {
        let root = temp_dir("omena_bridge_style_alias_extends_override")?;
        let source = root.join("src/App.tsx");
        let parent_style = root.join("src/parent/Button.module.scss");
        let child_style = root.join("src/child/Button.module.scss");
        fs::create_dir_all(
            parent_style
                .parent()
                .ok_or_else(|| std::io::Error::other("parent"))?,
        )?;
        fs::create_dir_all(
            child_style
                .parent()
                .ok_or_else(|| std::io::Error::other("child"))?,
        )?;
        fs::write(&source, "")?;
        fs::write(&parent_style, ".root { color: red; }")?;
        fs::write(&child_style, ".root { color: green; }")?;
        fs::write(
            root.join("base.json"),
            r#"{"compilerOptions":{"baseUrl":".","paths":{"$shared/*":["src/parent/*"]}}}"#,
        )?;
        fs::write(
            root.join("tsconfig.json"),
            r#"{"extends":"./base.json","compilerOptions":{"baseUrl":".","paths":{"$shared/*":["src/child/*"]}}}"#,
        )?;

        let uri = resolve_omena_bridge_style_uri_for_specifier(
            path_to_file_uri(source.as_path()).as_str(),
            Some(path_to_file_uri(root.as_path()).as_str()),
            "$shared/Button.module.scss",
        );

        assert_eq!(
            uri.as_deref(),
            Some(path_to_file_uri(child_style.as_path()).as_str())
        );
        let _ = fs::remove_dir_all(root);
        Ok(())
    }

    #[test]
    fn resolves_vite_bundler_alias_style_candidates() -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_dir("omena_bridge_style_bundler_alias")?;
        let source = root.join("src/App.tsx");
        let style = root.join("src/styles/Button.module.scss");
        fs::create_dir_all(
            style
                .parent()
                .ok_or_else(|| std::io::Error::other("parent"))?,
        )?;
        fs::write(&source, "")?;
        fs::write(&style, ".root {}")?;
        fs::write(
            root.join("vite.config.ts"),
            r#"export default { resolve: { alias: { "@styles": "./src/styles" } } };"#,
        )?;

        let uri = resolve_omena_bridge_style_uri_for_specifier(
            path_to_file_uri(source.as_path()).as_str(),
            Some(path_to_file_uri(root.as_path()).as_str()),
            "@styles/Button.module.scss",
        );

        assert_eq!(
            uri.as_deref(),
            Some(path_to_file_uri(style.as_path()).as_str())
        );
        let _ = fs::remove_dir_all(root);
        Ok(())
    }

    #[test]
    fn resolves_webpack_exact_bundler_alias_style_candidates()
    -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_dir("omena_bridge_style_bundler_exact_alias")?;
        let source = root.join("src/App.tsx");
        let style = root.join("src/styles/index.module.scss");
        fs::create_dir_all(
            style
                .parent()
                .ok_or_else(|| std::io::Error::other("parent"))?,
        )?;
        fs::write(&source, "")?;
        fs::write(&style, ".root {}")?;
        fs::write(
            root.join("webpack.config.js"),
            r#"module.exports = { resolve: { alias: [{ find: "@theme$", replacement: "./src/styles/index.module.scss" }] } };"#,
        )?;

        let exact_uri = resolve_omena_bridge_style_uri_for_specifier(
            path_to_file_uri(source.as_path()).as_str(),
            Some(path_to_file_uri(root.as_path()).as_str()),
            "@theme",
        );
        let prefix_uri = resolve_omena_bridge_style_uri_for_specifier(
            path_to_file_uri(source.as_path()).as_str(),
            Some(path_to_file_uri(root.as_path()).as_str()),
            "@theme/Button.module.scss",
        );

        assert_eq!(
            exact_uri.as_deref(),
            Some(path_to_file_uri(style.as_path()).as_str())
        );
        assert!(prefix_uri.is_none());
        let _ = fs::remove_dir_all(root);
        Ok(())
    }

    #[test]
    fn resolves_sass_style_candidates_without_legacy_language_filter()
    -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_dir("omena_bridge_style_sass")?;
        let source = root.join("src/App.tsx");
        let style = root.join("src/Button.module.sass");
        fs::create_dir_all(
            source
                .parent()
                .ok_or_else(|| std::io::Error::other("parent"))?,
        )?;
        fs::write(&source, "")?;
        fs::write(&style, ".root\n  color: red\n")?;

        let uri = resolve_omena_bridge_style_uri_for_specifier(
            path_to_file_uri(source.as_path()).as_str(),
            Some(path_to_file_uri(root.as_path()).as_str()),
            "./Button.module.sass",
        );

        assert_eq!(
            uri.as_deref(),
            Some(path_to_file_uri(style.as_path()).as_str())
        );
        let _ = fs::remove_dir_all(root);
        Ok(())
    }

    #[test]
    fn resolves_package_style_candidates_through_omena_resolver()
    -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_dir("omena_bridge_style_package")?;
        let source = root.join("src/App.module.scss");
        let package_root = root.join("node_modules/@design/tokens");
        let style = package_root.join("src/index.scss");
        fs::create_dir_all(
            style
                .parent()
                .ok_or_else(|| std::io::Error::other("parent"))?,
        )?;
        fs::create_dir_all(
            source
                .parent()
                .ok_or_else(|| std::io::Error::other("source parent"))?,
        )?;
        fs::write(&source, "@use \"@design/tokens\";")?;
        fs::write(
            package_root.join("package.json"),
            r#"{"sass":"src/index.scss"}"#,
        )?;
        fs::write(&style, "$gap: 1rem;")?;

        let uri = resolve_omena_bridge_style_uri_for_specifier(
            path_to_file_uri(source.as_path()).as_str(),
            Some(path_to_file_uri(root.as_path()).as_str()),
            "@design/tokens",
        );

        assert_eq!(
            uri.as_deref(),
            Some(path_to_file_uri(style.as_path()).as_str())
        );
        let _ = fs::remove_dir_all(root);
        Ok(())
    }

    #[test]
    fn resolves_sass_pkg_style_candidates_through_manifest_discovery()
    -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_dir("omena_bridge_style_pkg_manifest")?;
        let source = root.join("src/App.module.scss");
        let package_root = root.join("node_modules/@design/tokens");
        let style = package_root.join("dist/theme.scss");
        fs::create_dir_all(
            style
                .parent()
                .ok_or_else(|| std::io::Error::other("style parent"))?,
        )?;
        fs::create_dir_all(
            source
                .parent()
                .ok_or_else(|| std::io::Error::other("source parent"))?,
        )?;
        fs::write(&source, "@use \"pkg:@design/tokens/theme\";")?;
        fs::write(
            package_root.join("package.json"),
            r#"{"exports":{"./theme":{"sass":"./dist/theme.scss"}}}"#,
        )?;
        fs::write(&style, "$gap: 1rem;")?;

        let uri = resolve_omena_bridge_style_uri_for_specifier(
            path_to_file_uri(source.as_path()).as_str(),
            Some(path_to_file_uri(root.as_path()).as_str()),
            "pkg:@design/tokens/theme",
        );

        assert_eq!(
            uri.as_deref(),
            Some(path_to_file_uri(style.as_path()).as_str())
        );
        let _ = fs::remove_dir_all(root);
        Ok(())
    }

    #[test]
    fn resolves_package_import_style_candidates_through_workspace_manifests()
    -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_dir("omena_bridge_style_package_import_manifest")?;
        let source = root.join("src/App.module.scss");
        let package_root = root.join("node_modules/@design/tokens");
        let style = package_root.join("dist/theme.scss");
        fs::create_dir_all(
            style
                .parent()
                .ok_or_else(|| std::io::Error::other("style parent"))?,
        )?;
        fs::create_dir_all(
            source
                .parent()
                .ok_or_else(|| std::io::Error::other("source parent"))?,
        )?;
        fs::write(&source, "@use \"#theme\" as tokens;")?;
        fs::write(
            root.join("package.json"),
            r##"{"imports":{"#theme":"@design/tokens/theme"}}"##,
        )?;
        fs::write(
            package_root.join("package.json"),
            r#"{"exports":{"./theme":{"sass":"./dist/theme.scss"}}}"#,
        )?;
        fs::write(&style, "$gap: 1rem;")?;

        let uri = resolve_omena_bridge_style_uri_for_specifier(
            path_to_file_uri(source.as_path()).as_str(),
            Some(path_to_file_uri(root.as_path()).as_str()),
            "#theme",
        );

        assert_eq!(
            uri.as_deref(),
            Some(path_to_file_uri(style.as_path()).as_str())
        );
        let _ = fs::remove_dir_all(root);
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn resolves_symlinked_package_style_candidates_to_canonical_uri()
    -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_dir("omena_bridge_style_symlinked_package")?;
        let source = root.join("src/App.module.scss");
        let real_package = root.join(".pnpm/@design+tokens@1.0.0/node_modules/@design/tokens");
        let linked_scope = root.join("node_modules/@design");
        let linked_package = linked_scope.join("tokens");
        let style = real_package.join("src/index.scss");
        fs::create_dir_all(
            style
                .parent()
                .ok_or_else(|| std::io::Error::other("style parent"))?,
        )?;
        fs::create_dir_all(
            source
                .parent()
                .ok_or_else(|| std::io::Error::other("source parent"))?,
        )?;
        fs::create_dir_all(linked_scope.as_path())?;
        fs::write(&source, "@use \"@design/tokens\";")?;
        fs::write(
            real_package.join("package.json"),
            r#"{"sass":"src/index.scss"}"#,
        )?;
        fs::write(&style, "$gap: 1rem;")?;
        std::os::unix::fs::symlink(real_package.as_path(), linked_package.as_path())?;

        let uri = resolve_omena_bridge_style_uri_for_specifier(
            path_to_file_uri(source.as_path()).as_str(),
            Some(path_to_file_uri(root.as_path()).as_str()),
            "@design/tokens",
        );
        let expected_uri = path_to_file_uri(fs::canonicalize(style)?.as_path());

        assert_eq!(uri.as_deref(), Some(expected_uri.as_str()));
        let _ = fs::remove_dir_all(root);
        Ok(())
    }

    #[test]
    fn does_not_fabricate_missing_package_style_candidates()
    -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_dir("omena_bridge_style_missing_package")?;
        let source = root.join("src/App.tsx");
        fs::create_dir_all(
            source
                .parent()
                .ok_or_else(|| std::io::Error::other("parent"))?,
        )?;
        fs::write(&source, "")?;

        let uri = resolve_omena_bridge_style_uri_for_specifier(
            path_to_file_uri(source.as_path()).as_str(),
            Some(path_to_file_uri(root.as_path()).as_str()),
            "@design/tokens",
        );

        assert!(uri.is_none(), "{uri:?}");
        let _ = fs::remove_dir_all(root);
        Ok(())
    }

    #[test]
    fn emits_percent_encoded_file_uris_for_route_group_paths()
    -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_dir("omena_bridge_style_route_group")?;
        let source = root.join("app/(marketing)/page.tsx");
        let style = root.join("app/(marketing)/Card.module.scss");
        fs::create_dir_all(
            source
                .parent()
                .ok_or_else(|| std::io::Error::other("parent"))?,
        )?;
        fs::write(&source, "")?;
        fs::write(&style, ".card {}")?;

        let uri = resolve_omena_bridge_style_uri_for_specifier(
            path_to_file_uri(source.as_path()).as_str(),
            Some(path_to_file_uri(root.as_path()).as_str()),
            "./Card.module.scss",
        )
        .ok_or_else(|| std::io::Error::other("route group style should resolve"))?;

        assert!(uri.contains("%28marketing%29"), "{uri}");
        assert_eq!(uri, path_to_file_uri(style.as_path()));
        let _ = fs::remove_dir_all(root);
        Ok(())
    }

    #[test]
    fn declares_bridge_owned_style_resolution_boundary() {
        let summary = summarize_omena_bridge_style_resolution_boundary();

        assert_eq!(summary.product, "omena-bridge.style-resolution");
        assert_eq!(summary.owner_crate, "omena-bridge");
        assert!(summary.supported_specifier_kinds.contains(&"tsconfigPaths"));
        assert!(
            summary
                .supported_specifier_kinds
                .contains(&"bundlerAliases")
        );
        assert!(summary.supported_specifier_kinds.contains(&"npmPackages"));
        assert!(
            summary
                .request_path_policy
                .contains(&"pathAliasResolutionFollowsRelativeTsconfigExtends")
        );
        assert!(
            summary
                .request_path_policy
                .contains(&"bundlerAliasResolutionUsesLiteralViteWebpackConfig")
        );
        assert!(
            summary
                .request_path_policy
                .contains(&"lspServerOwnsOnlyDocumentRoutingAndUriRangeMapping")
        );
    }

    #[test]
    fn resolves_nested_next_config_alias_style_candidates() -> Result<(), Box<dyn std::error::Error>>
    {
        let root = temp_dir("omena_bridge_style_next_nested")?;
        let app_dir = root.join("apps/web");
        let source = app_dir.join("src/App.tsx");
        let style = app_dir.join("src/styles/Button.module.scss");
        fs::create_dir_all(
            style
                .parent()
                .ok_or_else(|| std::io::Error::other("style parent"))?,
        )?;
        fs::write(&source, "")?;
        fs::write(&style, ".root {}")?;
        fs::write(
            app_dir.join("next.config.mjs"),
            r#"export default { resolve: { alias: { "@styles": "./src/styles" } } };"#,
        )?;

        let uri = resolve_omena_bridge_style_uri_for_specifier(
            path_to_file_uri(source.as_path()).as_str(),
            Some(path_to_file_uri(root.as_path()).as_str()),
            "@styles/Button.module.scss",
        );

        assert_eq!(
            uri.as_deref(),
            Some(path_to_file_uri(style.as_path()).as_str())
        );
        let _ = fs::remove_dir_all(root);
        Ok(())
    }

    #[test]
    fn resolves_tilde_package_style_candidates() -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_dir("omena_bridge_style_tilde_package")?;
        let source = root.join("src/App.module.scss");
        let package_root = root.join("node_modules/@scope/theme");
        let style = package_root.join("index.scss");
        fs::create_dir_all(
            source
                .parent()
                .ok_or_else(|| std::io::Error::other("source parent"))?,
        )?;
        fs::create_dir_all(package_root.as_path())?;
        fs::write(&source, "@use \"~@scope/theme\";")?;
        fs::write(
            package_root.join("package.json"),
            r#"{"sass":"./index.scss"}"#,
        )?;
        fs::write(&style, "$brand: red;")?;

        let uri = resolve_omena_bridge_style_uri_for_specifier(
            path_to_file_uri(source.as_path()).as_str(),
            Some(path_to_file_uri(root.as_path()).as_str()),
            "~@scope/theme",
        );

        assert_eq!(
            uri.as_deref(),
            Some(path_to_file_uri(style.as_path()).as_str())
        );
        let _ = fs::remove_dir_all(root);
        Ok(())
    }

    fn temp_dir(prefix: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let suffix = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)?
            .as_nanos();
        let path = std::env::temp_dir().join(format!("{prefix}_{suffix}"));
        fs::create_dir_all(path.as_path())?;
        Ok(path)
    }
}
