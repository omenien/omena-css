use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    ffi::OsString,
    fs,
    path::{Component, Path, PathBuf},
    sync::{Mutex, OnceLock},
};

use crate::bundler_config_alias::load_omena_bridge_workspace_bundler_path_alias_mappings;
use crate::external_sif_signature::verify_omena_external_sif_keyless_bundle;
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
    OMENA_SIF_PUBLISHED_ATTESTATION_SUBJECT_PRODUCT_V1,
    OMENA_SIF_PUBLISHED_ATTESTATION_SUBJECT_SCHEMA_VERSION_V1,
    OMENA_SIF_SHARD_TRUST_ENVELOPE_PRODUCT_V1, OMENA_SIF_SHARD_TRUST_ENVELOPE_SCHEMA_VERSION_V1,
    OMENA_SIF_SHARD_VERDICT_DIR_V1, OmenaLifExportsV1, OmenaSifPublishedAttestationSubjectV1,
    OmenaSifShardLockBindingV1, OmenaSifShardRecordedVerdictV1, OmenaSifShardTrustEnvelopeV1,
    OmenaSifSourceSyntaxV1, OmenaSifStaticGeneratorInputV1, OmenaSifTrustTierV1, OmenaSifV1,
    compute_omena_sif_artifact_hash_v1, compute_omena_sif_leaf_hash_v1,
    compute_omena_sif_shard_recorded_verdict_address_v1, generate_static_omena_lif_exports_v1,
    generate_static_omena_sif_v1, read_omena_sif_json_v1,
    read_omena_sif_shard_recorded_verdict_json_v1, read_omena_sif_shard_trust_envelope_json_v1,
    validate_omena_sif_published_attestation_subject_v1, write_omena_canonical_json_bytes_v1,
    write_omena_sif_json_v1, write_omena_sif_published_attestation_subject_json_v1,
};
use serde::Serialize;
use serde_json::{Value, json};

const WORKSPACE_PACKAGE_MANIFEST_SCAN_LIMIT: usize = 1024;
const EXTERNAL_SIF_CACHE_SCHEMA_VERSION: &str = "1";
const EXTERNAL_SIF_CACHE_LEGACY_SCHEMA_VERSION: &str = "0";
const EXTERNAL_SIF_CACHE_KEY_SCHEMA_VERSION: &str = "0";
const EXTERNAL_SIF_CACHE_PRODUCT: &str = "omena-bridge.external-sif-cache-shard";
const EXTERNAL_SIF_CACHE_DIR: &str = "external-sif-v0";
const EXTERNAL_SIF_CACHE_ENV_KILL_SWITCH: &str = "OMENA_BRIDGE_EXTERNAL_SIF_CACHE";
const EXTERNAL_SIF_CACHE_MAX_MEMORY_ENTRIES: usize = 256;
const EXTERNAL_SIF_CACHE_MAX_SHARDS: usize = 2048;
const EXTERNAL_SIF_CACHE_MAX_TOTAL_BYTES: u64 = 256 * 1024 * 1024;
const EXTERNAL_SIF_CACHE_MAX_SHARD_BYTES: u64 = 8 * 1024 * 1024;
const EXTERNAL_SIF_RECORDED_BUNDLE_DIR_V1: &str = "bundles-v1";
const EXTERNAL_SIF_RECORDED_BUNDLE_SUFFIX_V1: &str = ".sigstore.json";
const EXTERNAL_SIF_RECORDED_BUNDLE_MAX_BYTES: u64 = 4 * 1024 * 1024;
const EXTERNAL_SIF_RECORDED_VERDICT_SCAN_LIMIT: usize = 4096;
const WORKSPACE_STYLE_PATH_IDENTITY_SCAN_LIMIT: usize = 4096;
const WORKSPACE_STYLE_PATH_IDENTITY_MAX_DEPTH: usize = 8;

static EXTERNAL_SIF_MEMORY_CACHE: OnceLock<
    Mutex<BTreeMap<String, OmenaBridgeExternalSifWithTrustV1>>,
> = OnceLock::new();

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OmenaBridgeExternalSifStorageV0 {
    workspace_cache_root: PathBuf,
    workspace_identity: Option<String>,
    recorded_verdict_dir: Option<PathBuf>,
}

impl OmenaBridgeExternalSifStorageV0 {
    pub fn from_workspace_cache_root(workspace_cache_root: PathBuf) -> Self {
        Self {
            recorded_verdict_dir: Some(workspace_cache_root.join(OMENA_SIF_SHARD_VERDICT_DIR_V1)),
            workspace_cache_root,
            workspace_identity: None,
        }
    }

    pub fn from_workspace_cache_root_and_identity(
        workspace_cache_root: PathBuf,
        workspace_identity: impl Into<String>,
    ) -> Self {
        Self {
            recorded_verdict_dir: Some(workspace_cache_root.join(OMENA_SIF_SHARD_VERDICT_DIR_V1)),
            workspace_cache_root,
            workspace_identity: Some(workspace_identity.into()),
        }
    }

    pub fn workspace_cache_root(&self) -> &Path {
        self.workspace_cache_root.as_path()
    }

    pub fn with_recorded_verdict_dir(mut self, recorded_verdict_dir: PathBuf) -> Self {
        self.recorded_verdict_dir = Some(recorded_verdict_dir);
        self
    }

    pub fn recorded_verdict_dir(&self) -> Option<&Path> {
        self.recorded_verdict_dir.as_deref()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum OmenaBridgeExternalSifTrustSourceV1 {
    RecordedVerdict,
    UnsignedLegacy,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaBridgeExternalSifWithTrustV1 {
    pub sif: OmenaSifV1,
    pub trust_envelope: OmenaSifShardTrustEnvelopeV1,
    pub trust_source: OmenaBridgeExternalSifTrustSourceV1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OmenaBridgeExternalSifShardRefusalV1 {
    ShardIdentityMismatch,
    CanonicalUrlMismatch,
    LocalRegenerationMismatch,
    PayloadDigestMismatch,
    MalformedSif,
    MissingTrustEnvelope,
    InvalidTrustEnvelope,
    LockBindingMismatch,
    MissingRecordedVerdict,
    RecordedVerdictMismatch,
    TierAboveRecordedVerdict,
    RecordedVerdictDowngrade,
    RecordedVerdictSignatureVerificationFailed,
}

impl OmenaBridgeExternalSifShardRefusalV1 {
    pub fn code(self) -> &'static str {
        match self {
            Self::ShardIdentityMismatch => "shardIdentityMismatch",
            Self::CanonicalUrlMismatch => "canonicalUrlMismatch",
            Self::LocalRegenerationMismatch => "localRegenerationMismatch",
            Self::PayloadDigestMismatch => "payloadDigestMismatch",
            Self::MalformedSif => "malformedSif",
            Self::MissingTrustEnvelope => "missingTrustEnvelope",
            Self::InvalidTrustEnvelope => "invalidTrustEnvelope",
            Self::LockBindingMismatch => "lockBindingMismatch",
            Self::MissingRecordedVerdict => "missingRecordedVerdict",
            Self::RecordedVerdictMismatch => "recordedVerdictMismatch",
            Self::TierAboveRecordedVerdict => "tierAboveRecordedVerdict",
            Self::RecordedVerdictDowngrade => "recordedVerdictDowngrade",
            Self::RecordedVerdictSignatureVerificationFailed => {
                "recordedVerdictSignatureVerificationFailed"
            }
        }
    }
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

pub fn generate_omena_bridge_lif_exports_for_resolved_style_path(
    resolved_path: &str,
) -> Result<OmenaLifExportsV1, String> {
    let raw_path = raw_resolved_style_entry_path(resolved_path)
        .ok_or_else(|| format!("unresolvable style module entry path: {resolved_path}"))?;
    let path = normalize_path(raw_path);
    let source = fs::read_to_string(path.as_path()).map_err(|error| {
        format!(
            "failed to read resolved style module {}: {error}",
            path.to_string_lossy()
        )
    })?;
    let syntax = infer_omena_bridge_sif_source_syntax(path.as_path());
    Ok(generate_static_omena_lif_exports_v1(
        OmenaSifStaticGeneratorInputV1 {
            canonical_url: path_to_file_uri(path.as_path()).as_str(),
            source: source.as_str(),
            syntax,
        },
    ))
}

pub fn generate_omena_bridge_sif_for_resolved_style_path_with_cache_context(
    resolved_path: &str,
    cache_context: &OmenaBridgeExternalSifCacheContextV0,
) -> Result<OmenaSifV1, String> {
    generate_omena_bridge_sif_for_resolved_style_path_with_cache_context_and_storage(
        resolved_path,
        cache_context,
        None,
    )
}

pub fn generate_omena_bridge_sif_for_resolved_style_path_with_cache_context_and_storage(
    resolved_path: &str,
    cache_context: &OmenaBridgeExternalSifCacheContextV0,
    cache_storage: Option<&OmenaBridgeExternalSifStorageV0>,
) -> Result<OmenaSifV1, String> {
    generate_omena_bridge_sif_for_resolved_style_path_with_cache_context_storage_and_trust(
        resolved_path,
        cache_context,
        cache_storage,
    )
    .map(|result| result.sif)
}

pub fn generate_omena_bridge_sif_for_resolved_style_path_with_cache_context_storage_and_trust(
    resolved_path: &str,
    cache_context: &OmenaBridgeExternalSifCacheContextV0,
    cache_storage: Option<&OmenaBridgeExternalSifStorageV0>,
) -> Result<OmenaBridgeExternalSifWithTrustV1, String> {
    generate_omena_bridge_sif_for_resolved_style_path_with_canonical_url_impl(
        resolved_path,
        None,
        cache_context,
        cache_storage,
    )
}

pub fn generate_omena_bridge_sif_for_resolved_style_path_with_canonical_url_cache_context_storage_and_trust(
    resolved_path: &str,
    canonical_url: &str,
    cache_context: &OmenaBridgeExternalSifCacheContextV0,
    cache_storage: Option<&OmenaBridgeExternalSifStorageV0>,
) -> Result<OmenaBridgeExternalSifWithTrustV1, String> {
    if canonical_url.trim().is_empty() {
        return Err("external SIF canonical URL must not be empty".to_string());
    }
    generate_omena_bridge_sif_for_resolved_style_path_with_canonical_url_impl(
        resolved_path,
        Some(canonical_url),
        cache_context,
        cache_storage,
    )
}

fn generate_omena_bridge_sif_for_resolved_style_path_with_canonical_url_impl(
    resolved_path: &str,
    canonical_url_override: Option<&str>,
    cache_context: &OmenaBridgeExternalSifCacheContextV0,
    cache_storage: Option<&OmenaBridgeExternalSifStorageV0>,
) -> Result<OmenaBridgeExternalSifWithTrustV1, String> {
    let raw_path = raw_resolved_style_entry_path(resolved_path)
        .ok_or_else(|| format!("unresolvable style module entry path: {resolved_path}"))?;
    let path = normalize_path(raw_path.clone());
    let canonical_url = canonical_url_override
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| path_to_file_uri(path.as_path()));
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
    let cache_enabled = !external_sif_cache_kill_switch_engaged();
    let cache_dir = cache_enabled
        .then(|| external_sif_cache_dir_for_path(raw_path.as_path(), cache_storage))
        .flatten();
    let cache_workspace_identity = cache_storage
        .and_then(|storage| storage.workspace_identity.clone())
        .or_else(|| {
            crate::cache_root::external_sif_workspace_root(raw_path.as_path())
                .map(|root| root.to_string_lossy().into_owned())
        });
    let recorded_verdict_dir =
        external_sif_recorded_verdict_dir_for_path(raw_path.as_path(), cache_storage);
    let memory_cache_key = external_sif_memory_cache_key(cache_key.as_str(), cache_dir.as_deref());
    let source = String::from_utf8(source_bytes).map_err(|error| {
        format!(
            "failed to decode resolved style module {} as utf-8: {error}",
            path.to_string_lossy()
        )
    })?;
    let syntax = infer_omena_bridge_sif_source_syntax(path.as_path());
    let locally_regenerated_sif = generate_static_omena_sif_v1(OmenaSifStaticGeneratorInputV1 {
        canonical_url: canonical_url.as_str(),
        source: source.as_str(),
        syntax,
    })
    .map_err(|error| format!("failed to generate SIF for {canonical_url}: {error}"))?;
    if cache_enabled {
        if let Some(result) = load_external_sif_from_memory_cache(memory_cache_key.as_str()) {
            if result.sif == locally_regenerated_sif {
                let result = external_sif_result_with_recorded_verdict(
                    result.sif,
                    recorded_verdict_dir.as_deref(),
                )?;
                store_external_sif_in_memory_cache(memory_cache_key.clone(), result.clone());
                return Ok(result);
            }
            remove_external_sif_from_memory_cache(memory_cache_key.as_str());
        }
        if let Some(cache_dir) = cache_dir.as_deref()
            && let Some(result) = load_external_sif_cache_shard(
                cache_dir,
                cache_key.as_str(),
                canonical_url.as_str(),
                source_hash.as_str(),
                resolved_base_dir.as_str(),
                recorded_verdict_dir.as_deref(),
                &locally_regenerated_sif,
            )
        {
            store_external_sif_in_memory_cache(memory_cache_key.clone(), result.clone());
            return Ok(result);
        }
    }
    let result = external_sif_result_with_recorded_verdict(
        locally_regenerated_sif,
        recorded_verdict_dir.as_deref(),
    )?;
    if cache_enabled {
        store_external_sif_in_memory_cache(memory_cache_key, result.clone());
        if let Some(cache_dir) = cache_dir.as_deref() {
            store_external_sif_cache_shard(
                cache_dir,
                cache_key.as_str(),
                canonical_url.as_str(),
                source_hash.as_str(),
                resolved_base_dir.as_str(),
                &result,
                cache_workspace_identity.as_deref(),
            );
        }
    }
    Ok(result)
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
    external_sif_cache_key_with_crate_version(
        source_hash,
        resolved_base_dir,
        canonical_url,
        freshness_fingerprint,
        env!("CARGO_PKG_VERSION"),
    )
}

fn external_sif_cache_key_with_crate_version(
    source_hash: &str,
    resolved_base_dir: &str,
    canonical_url: &str,
    freshness_fingerprint: Option<&str>,
    crate_version: &str,
) -> String {
    let input = json!({
        "schemaVersion": EXTERNAL_SIF_CACHE_KEY_SCHEMA_VERSION,
        "product": "omena-bridge.external-sif-cache-key",
        "crateVersion": crate_version,
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
                    "{crate_version}\0{source_hash}\0{resolved_base_dir}\0{canonical_url}\0{}",
                    freshness_fingerprint.unwrap_or("")
                )
                .as_bytes(),
            )
            .as_str()
            .to_string()
        })
}

fn load_external_sif_from_memory_cache(key: &str) -> Option<OmenaBridgeExternalSifWithTrustV1> {
    EXTERNAL_SIF_MEMORY_CACHE
        .get_or_init(|| Mutex::new(BTreeMap::new()))
        .lock()
        .ok()?
        .get(key)
        .cloned()
}

fn store_external_sif_in_memory_cache(key: String, result: OmenaBridgeExternalSifWithTrustV1) {
    let Ok(mut cache) = EXTERNAL_SIF_MEMORY_CACHE
        .get_or_init(|| Mutex::new(BTreeMap::new()))
        .lock()
    else {
        return;
    };
    cache.insert(key, result);
    while cache.len() > EXTERNAL_SIF_CACHE_MAX_MEMORY_ENTRIES {
        let Some(first_key) = cache.keys().next().cloned() else {
            break;
        };
        cache.remove(first_key.as_str());
    }
}

fn remove_external_sif_from_memory_cache(key: &str) {
    let Ok(mut cache) = EXTERNAL_SIF_MEMORY_CACHE
        .get_or_init(|| Mutex::new(BTreeMap::new()))
        .lock()
    else {
        return;
    };
    cache.remove(key);
}

fn external_sif_memory_cache_key(key: &str, cache_dir: Option<&Path>) -> String {
    cache_dir
        .map(|cache_dir| format!("{}\0{key}", cache_dir.to_string_lossy()))
        .unwrap_or_else(|| key.to_string())
}

fn external_sif_cache_dir_for_path(
    path: &Path,
    cache_storage: Option<&OmenaBridgeExternalSifStorageV0>,
) -> Option<PathBuf> {
    if let Some(cache_storage) = cache_storage {
        return Some(
            cache_storage
                .workspace_cache_root()
                .join(EXTERNAL_SIF_CACHE_DIR),
        );
    }
    crate::cache_root::process_external_sif_cache_root(path)?
        .workspace
        .map(|root| root.join(EXTERNAL_SIF_CACHE_DIR))
}

fn external_sif_recorded_verdict_dir_for_path(
    path: &Path,
    cache_storage: Option<&OmenaBridgeExternalSifStorageV0>,
) -> Option<PathBuf> {
    if let Some(dir) = cache_storage.and_then(|storage| storage.recorded_verdict_dir()) {
        return Some(dir.to_path_buf());
    }
    crate::cache_root::external_sif_workspace_root(path).map(|root| {
        root.join(".cache")
            .join("omena")
            .join(OMENA_SIF_SHARD_VERDICT_DIR_V1)
    })
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
    recorded_verdict_dir: Option<&Path>,
    locally_regenerated_sif: &OmenaSifV1,
) -> Option<OmenaBridgeExternalSifWithTrustV1> {
    let shard_path = external_sif_cache_shard_file_path(dir, key)?;
    let metadata = fs::metadata(shard_path.as_path()).ok()?;
    if !metadata.is_file() || metadata.len() > EXTERNAL_SIF_CACHE_MAX_SHARD_BYTES {
        let _ = fs::remove_file(shard_path.as_path());
        return None;
    }
    let bytes = fs::read(shard_path.as_path()).ok()?;
    let shard = serde_json::from_slice::<Value>(bytes.as_slice()).ok()?;
    match validate_external_sif_cache_shard(
        &shard,
        key,
        canonical_url,
        source_hash,
        resolved_base_dir,
        recorded_verdict_dir,
        locally_regenerated_sif,
    ) {
        Ok(result) => Some(result),
        Err(_) => {
            let _ = fs::remove_file(shard_path.as_path());
            None
        }
    }
}

fn validate_external_sif_cache_shard(
    shard: &Value,
    key: &str,
    canonical_url: &str,
    source_hash: &str,
    resolved_base_dir: &str,
    recorded_verdict_dir: Option<&Path>,
    locally_regenerated_sif: &OmenaSifV1,
) -> Result<OmenaBridgeExternalSifWithTrustV1, OmenaBridgeExternalSifShardRefusalV1> {
    let schema_version = shard.get("schemaVersion").and_then(Value::as_str);
    if !matches!(
        schema_version,
        Some(EXTERNAL_SIF_CACHE_SCHEMA_VERSION | EXTERNAL_SIF_CACHE_LEGACY_SCHEMA_VERSION)
    ) || shard.get("product").and_then(Value::as_str) != Some(EXTERNAL_SIF_CACHE_PRODUCT)
        || shard.get("key").and_then(Value::as_str) != Some(key)
        || shard.get("canonicalUrl").and_then(Value::as_str) != Some(canonical_url)
        || shard.get("sourceHash").and_then(Value::as_str) != Some(source_hash)
        || shard.get("resolvedBaseDir").and_then(Value::as_str) != Some(resolved_base_dir)
    {
        return Err(OmenaBridgeExternalSifShardRefusalV1::ShardIdentityMismatch);
    }
    let sif_json = shard
        .get("sifJson")
        .and_then(Value::as_str)
        .ok_or(OmenaBridgeExternalSifShardRefusalV1::MalformedSif)?;
    let payload_digest = compute_omena_sif_leaf_hash_v1(sif_json.as_bytes());
    if shard.get("payloadDigest").and_then(Value::as_str) != Some(payload_digest.as_str()) {
        return Err(OmenaBridgeExternalSifShardRefusalV1::PayloadDigestMismatch);
    }
    let sif = read_omena_sif_json_v1(sif_json)
        .map_err(|_| OmenaBridgeExternalSifShardRefusalV1::MalformedSif)?;
    if sif.canonical_url != canonical_url {
        return Err(OmenaBridgeExternalSifShardRefusalV1::CanonicalUrlMismatch);
    }
    if sif != *locally_regenerated_sif {
        return Err(OmenaBridgeExternalSifShardRefusalV1::LocalRegenerationMismatch);
    }
    let sif_hash = compute_omena_sif_artifact_hash_v1(&sif)
        .map_err(|_| OmenaBridgeExternalSifShardRefusalV1::MalformedSif)?;
    if schema_version == Some(EXTERNAL_SIF_CACHE_LEGACY_SCHEMA_VERSION) {
        if has_recorded_shard_verdict_for_canonical_url(recorded_verdict_dir, canonical_url) {
            return Err(OmenaBridgeExternalSifShardRefusalV1::RecordedVerdictDowngrade);
        }
        return Ok(unsigned_external_sif_result(sif, payload_digest, sif_hash));
    }
    let envelope_value = shard
        .get("trustEnvelope")
        .ok_or(OmenaBridgeExternalSifShardRefusalV1::MissingTrustEnvelope)?;
    let envelope_source = serde_json::to_string(envelope_value)
        .map_err(|_| OmenaBridgeExternalSifShardRefusalV1::InvalidTrustEnvelope)?;
    let envelope = read_omena_sif_shard_trust_envelope_json_v1(envelope_source.as_str())
        .map_err(|_| OmenaBridgeExternalSifShardRefusalV1::InvalidTrustEnvelope)?;
    if envelope.payload_digest != payload_digest {
        return Err(OmenaBridgeExternalSifShardRefusalV1::PayloadDigestMismatch);
    }
    if envelope.lock_binding.canonical_url != canonical_url
        || envelope.lock_binding.sif_hash != sif_hash
    {
        return Err(OmenaBridgeExternalSifShardRefusalV1::LockBindingMismatch);
    }
    if envelope.trust_tier < OmenaSifTrustTierV1::T2 {
        if has_recorded_shard_verdict_for_canonical_url(recorded_verdict_dir, canonical_url) {
            return Err(OmenaBridgeExternalSifShardRefusalV1::RecordedVerdictDowngrade);
        }
        return Ok(OmenaBridgeExternalSifWithTrustV1 {
            sif,
            trust_envelope: envelope,
            trust_source: OmenaBridgeExternalSifTrustSourceV1::UnsignedLegacy,
        });
    }
    let verdict = load_recorded_shard_verdict(recorded_verdict_dir, canonical_url, &sif_hash)
        .ok_or(OmenaBridgeExternalSifShardRefusalV1::MissingRecordedVerdict)?;
    if envelope.trust_tier > verdict.trust_tier {
        return Err(OmenaBridgeExternalSifShardRefusalV1::TierAboveRecordedVerdict);
    }
    if envelope.lock_binding.canonical_url != verdict.canonical_url
        || envelope.lock_binding.sif_hash != verdict.sif_hash
        || envelope.signature.as_ref() != Some(&verdict.signature)
    {
        return Err(OmenaBridgeExternalSifShardRefusalV1::RecordedVerdictMismatch);
    }
    verify_recorded_shard_verdict(recorded_verdict_dir, &verdict)?;
    Ok(OmenaBridgeExternalSifWithTrustV1 {
        sif,
        trust_envelope: envelope,
        trust_source: OmenaBridgeExternalSifTrustSourceV1::RecordedVerdict,
    })
}

fn store_external_sif_cache_shard(
    dir: &Path,
    key: &str,
    canonical_url: &str,
    source_hash: &str,
    resolved_base_dir: &str,
    result: &OmenaBridgeExternalSifWithTrustV1,
    workspace_identity: Option<&str>,
) {
    let Ok(sif_json) = write_omena_sif_json_v1(&result.sif) else {
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
        "trustEnvelope": result.trust_envelope,
        "sifJson": sif_json,
    });
    let Ok(bytes) = write_omena_canonical_json_bytes_v1(&shard) else {
        return;
    };
    if bytes.len() as u64 > EXTERNAL_SIF_CACHE_MAX_SHARD_BYTES {
        return;
    }
    if write_external_sif_cache_shard_atomically(dir, key, bytes.as_slice()).is_ok() {
        if let Some(workspace_identity) = workspace_identity {
            crate::cache_root::ensure_omena_cache_root_attribution(dir, workspace_identity);
        }
        enforce_external_sif_cache_caps(dir);
    }
}

fn external_sif_result_with_recorded_verdict(
    sif: OmenaSifV1,
    recorded_verdict_dir: Option<&Path>,
) -> Result<OmenaBridgeExternalSifWithTrustV1, String> {
    let sif_json = write_omena_sif_json_v1(&sif)
        .map_err(|error| format!("failed to serialize generated SIF trust payload: {error}"))?;
    let payload_digest = compute_omena_sif_leaf_hash_v1(sif_json.as_bytes());
    let sif_hash = compute_omena_sif_artifact_hash_v1(&sif)
        .map_err(|error| format!("failed to hash generated SIF trust payload: {error}"))?;
    let verdict =
        load_recorded_shard_verdict(recorded_verdict_dir, sif.canonical_url.as_str(), &sif_hash);
    let verified_verdict = verdict
        .filter(|verdict| verify_recorded_shard_verdict(recorded_verdict_dir, verdict).is_ok());
    let (trust_tier, signature, trust_source) = match verified_verdict {
        Some(verdict) => (
            verdict.trust_tier,
            Some(verdict.signature),
            OmenaBridgeExternalSifTrustSourceV1::RecordedVerdict,
        ),
        None => (
            OmenaSifTrustTierV1::T1,
            None,
            OmenaBridgeExternalSifTrustSourceV1::UnsignedLegacy,
        ),
    };
    let trust_envelope = OmenaSifShardTrustEnvelopeV1 {
        schema_version: OMENA_SIF_SHARD_TRUST_ENVELOPE_SCHEMA_VERSION_V1.to_string(),
        product: OMENA_SIF_SHARD_TRUST_ENVELOPE_PRODUCT_V1.to_string(),
        trust_tier,
        payload_digest,
        signature,
        lock_binding: OmenaSifShardLockBindingV1 {
            canonical_url: sif.canonical_url.clone(),
            sif_hash,
        },
    };
    omena_sif::validate_omena_sif_shard_trust_envelope_v1(&trust_envelope)
        .map_err(|error| format!("generated SIF trust envelope is invalid: {error}"))?;
    Ok(OmenaBridgeExternalSifWithTrustV1 {
        sif,
        trust_envelope,
        trust_source,
    })
}

fn unsigned_external_sif_result(
    sif: OmenaSifV1,
    payload_digest: omena_sif::OmenaSifDigestV1,
    sif_hash: omena_sif::OmenaSifDigestV1,
) -> OmenaBridgeExternalSifWithTrustV1 {
    OmenaBridgeExternalSifWithTrustV1 {
        trust_envelope: OmenaSifShardTrustEnvelopeV1 {
            schema_version: OMENA_SIF_SHARD_TRUST_ENVELOPE_SCHEMA_VERSION_V1.to_string(),
            product: OMENA_SIF_SHARD_TRUST_ENVELOPE_PRODUCT_V1.to_string(),
            trust_tier: OmenaSifTrustTierV1::T1,
            payload_digest,
            signature: None,
            lock_binding: OmenaSifShardLockBindingV1 {
                canonical_url: sif.canonical_url.clone(),
                sif_hash,
            },
        },
        sif,
        trust_source: OmenaBridgeExternalSifTrustSourceV1::UnsignedLegacy,
    }
}

fn load_recorded_shard_verdict(
    recorded_verdict_dir: Option<&Path>,
    canonical_url: &str,
    sif_hash: &omena_sif::OmenaSifDigestV1,
) -> Option<OmenaSifShardRecordedVerdictV1> {
    let verdict_dir = recorded_verdict_dir?;
    let address =
        compute_omena_sif_shard_recorded_verdict_address_v1(canonical_url, sif_hash).ok()?;
    let hex = address.as_str().strip_prefix("blake3:")?;
    let verdict_path = verdict_dir.join(format!("{hex}.json"));
    let metadata = fs::metadata(verdict_path.as_path()).ok()?;
    if !metadata.is_file() || metadata.len() > 1024 * 1024 {
        return None;
    }
    let source = fs::read_to_string(verdict_path.as_path()).ok()?;
    let verdict = read_omena_sif_shard_recorded_verdict_json_v1(source.as_str()).ok()?;
    (verdict.canonical_url == canonical_url && verdict.sif_hash == *sif_hash).then_some(verdict)
}

fn has_recorded_shard_verdict_for_canonical_url(
    recorded_verdict_dir: Option<&Path>,
    canonical_url: &str,
) -> bool {
    let Some(verdict_dir) = recorded_verdict_dir else {
        return false;
    };
    let entries = match fs::read_dir(verdict_dir) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return false,
        Err(_) => return true,
    };
    let mut verdict_file_count = 0usize;
    for entry in entries {
        let Ok(entry) = entry else {
            return true;
        };
        let path = entry.path();
        if path.extension().and_then(|extension| extension.to_str()) != Some("json") {
            continue;
        }
        verdict_file_count += 1;
        if verdict_file_count > EXTERNAL_SIF_RECORDED_VERDICT_SCAN_LIMIT {
            return true;
        }
        let Ok(metadata) = entry.metadata() else {
            return true;
        };
        if !metadata.is_file() || metadata.len() > 1024 * 1024 {
            return true;
        }
        let Ok(source) = fs::read_to_string(path) else {
            return true;
        };
        let Ok(verdict) = read_omena_sif_shard_recorded_verdict_json_v1(source.as_str()) else {
            return true;
        };
        if verdict.canonical_url == canonical_url {
            return true;
        }
    }
    false
}

fn verify_recorded_shard_verdict(
    recorded_verdict_dir: Option<&Path>,
    verdict: &OmenaSifShardRecordedVerdictV1,
) -> Result<(), OmenaBridgeExternalSifShardRefusalV1> {
    let verdict_dir = recorded_verdict_dir
        .ok_or(OmenaBridgeExternalSifShardRefusalV1::RecordedVerdictSignatureVerificationFailed)?;
    let reference = verdict.signature.reference.as_str();
    let relative_bundle_name = reference
        .strip_prefix(&format!("{EXTERNAL_SIF_RECORDED_BUNDLE_DIR_V1}/"))
        .and_then(|name| name.strip_suffix(EXTERNAL_SIF_RECORDED_BUNDLE_SUFFIX_V1))
        .filter(|digest| {
            digest.len() == 64
                && digest
                    .bytes()
                    .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
        })
        .ok_or(OmenaBridgeExternalSifShardRefusalV1::RecordedVerdictSignatureVerificationFailed)?;
    let bundle_path = verdict_dir
        .join(EXTERNAL_SIF_RECORDED_BUNDLE_DIR_V1)
        .join(format!(
            "{relative_bundle_name}{EXTERNAL_SIF_RECORDED_BUNDLE_SUFFIX_V1}"
        ));
    let metadata = fs::metadata(bundle_path.as_path()).map_err(|_| {
        OmenaBridgeExternalSifShardRefusalV1::RecordedVerdictSignatureVerificationFailed
    })?;
    if !metadata.is_file() || metadata.len() > EXTERNAL_SIF_RECORDED_BUNDLE_MAX_BYTES {
        return Err(
            OmenaBridgeExternalSifShardRefusalV1::RecordedVerdictSignatureVerificationFailed,
        );
    }
    let bundle_bytes = fs::read(bundle_path.as_path()).map_err(|_| {
        OmenaBridgeExternalSifShardRefusalV1::RecordedVerdictSignatureVerificationFailed
    })?;
    let attested_subject = OmenaSifPublishedAttestationSubjectV1 {
        schema_version: OMENA_SIF_PUBLISHED_ATTESTATION_SUBJECT_SCHEMA_VERSION_V1.to_string(),
        product: OMENA_SIF_PUBLISHED_ATTESTATION_SUBJECT_PRODUCT_V1.to_string(),
        canonical_url: verdict.canonical_url.clone(),
        trust_tier: verdict.trust_tier,
        sif_hash: verdict.sif_hash.clone(),
    };
    validate_omena_sif_published_attestation_subject_v1(&attested_subject).map_err(|_| {
        OmenaBridgeExternalSifShardRefusalV1::RecordedVerdictSignatureVerificationFailed
    })?;
    let attested_subject_json =
        write_omena_sif_published_attestation_subject_json_v1(&attested_subject).map_err(|_| {
            OmenaBridgeExternalSifShardRefusalV1::RecordedVerdictSignatureVerificationFailed
        })?;
    verify_omena_external_sif_keyless_bundle(
        attested_subject_json.as_bytes(),
        bundle_bytes.as_slice(),
        relative_bundle_name,
    )
    .map_err(|_| OmenaBridgeExternalSifShardRefusalV1::RecordedVerdictSignatureVerificationFailed)
}

fn write_external_sif_cache_shard_atomically(
    dir: &Path,
    key: &str,
    bytes: &[u8],
) -> std::io::Result<()> {
    fs::create_dir_all(dir)?;
    crate::cache_root::ensure_omena_cache_root_markers(dir);
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
fn clear_external_sif_memory_cache_for_storage_for_test(storage: &OmenaBridgeExternalSifStorageV0) {
    let namespace = format!(
        "{}\0",
        storage
            .workspace_cache_root()
            .join(EXTERNAL_SIF_CACHE_DIR)
            .to_string_lossy()
    );
    if let Some(cache) = EXTERNAL_SIF_MEMORY_CACHE.get()
        && let Ok(mut cache) = cache.lock()
    {
        cache.retain(|key, _| !key.starts_with(namespace.as_str()));
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
        Some("less") => OmenaSifSourceSyntaxV1::Less,
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

        let sif = generate_fixture_sif_for_resolved_style_path(style.as_path(), resolved.as_str())?;

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
    fn generates_lif_exports_for_resolved_less_specifier() -> Result<(), Box<dyn std::error::Error>>
    {
        let root = temp_dir("omena_bridge_lif_resolved_less")?;
        let source = root.join("src/App.tsx");
        let style = root.join("src/tokens.less");
        fs::create_dir_all(
            style
                .parent()
                .ok_or_else(|| std::io::Error::other("parent"))?,
        )?;
        fs::write(&source, "")?;
        fs::write(
            &style,
            "@brand: #fff;\n@tokens: { primary: @brand; @gap: 2px; };\n.button(@gap: 1rem) when (@gap > 0) { color: @brand; }\n",
        )?;

        let resolved = resolve_omena_bridge_style_uri_for_specifier(
            path_to_file_uri(source.as_path()).as_str(),
            Some(path_to_file_uri(root.as_path()).as_str()),
            "./tokens.less",
        )
        .ok_or_else(|| std::io::Error::other("Less resolution failed"))?;

        let exports = generate_omena_bridge_lif_exports_for_resolved_style_path(resolved.as_str())?;

        assert_eq!(exports.less_variables[0].name, "@brand");
        assert_eq!(
            exports.less_variables[0].value_repr.as_deref(),
            Some("#fff")
        );
        assert_eq!(exports.less_mixins[0].name, ".button");
        assert!(exports.less_mixins[0].guarded);
        assert_eq!(exports.less_detached_rulesets[0].name, "@tokens");
        assert_eq!(
            exports.less_detached_rulesets[0].member_names,
            vec!["@gap", "primary"]
        );
        let _ = fs::remove_dir_all(root);
        Ok(())
    }

    #[test]
    fn generates_sif_from_plain_resolved_path() -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_dir("omena_bridge_sif_plain")?;
        let style = root.join("tokens.sass");
        fs::write(&style, "$gap: 8px\n")?;

        let sif = generate_fixture_sif_for_resolved_style_path(
            style.as_path(),
            style.to_string_lossy().as_ref(),
        )?;

        assert_eq!(sif.source.syntax, OmenaSifSourceSyntaxV1::Sass);
        let _ = fs::remove_dir_all(root);
        Ok(())
    }

    #[test]
    fn generates_sif_with_less_source_syntax() -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_dir("omena_bridge_sif_less")?;
        let style = root.join("tokens.less");
        fs::write(&style, "@gap: 8px;\n.button { margin: @gap; }\n")?;

        let sif = generate_fixture_sif_for_resolved_style_path(
            style.as_path(),
            style.to_string_lossy().as_ref(),
        )?;

        assert_eq!(sif.source.syntax, OmenaSifSourceSyntaxV1::Less);
        let json = omena_sif::write_omena_sif_json_v1(&sif)?;
        assert!(json.contains(r#""syntax":"less""#));
        let _ = fs::remove_dir_all(root);
        Ok(())
    }

    #[test]
    fn generates_lif_exports_for_resolved_less_path() -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_dir("omena_bridge_lif_less")?;
        let style = root.join("tokens.less");
        fs::write(
            &style,
            "@brand: red;\n@tokens: { primary: @brand; };\n.button(@gap: 1rem) { color: @brand; }\n",
        )?;

        let exports = generate_omena_bridge_lif_exports_for_resolved_style_path(
            style.to_string_lossy().as_ref(),
        )?;

        assert_eq!(exports.less_variables[0].name, "@brand");
        assert_eq!(exports.less_mixins[0].name, ".button");
        assert_eq!(exports.less_detached_rulesets[0].name, "@tokens");
        assert_eq!(
            exports.less_detached_rulesets[0].member_names,
            vec!["primary"]
        );
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
        let old_crate_version_key = external_sif_cache_key_with_crate_version(
            source_hash.as_str(),
            first_base_dir.as_str(),
            path_to_file_uri(first_path.as_path()).as_str(),
            None,
            "0.2.0",
        );
        let current_crate_version_key = external_sif_cache_key_with_crate_version(
            source_hash.as_str(),
            first_base_dir.as_str(),
            path_to_file_uri(first_path.as_path()).as_str(),
            None,
            env!("CARGO_PKG_VERSION"),
        );
        assert_ne!(
            old_crate_version_key, current_crate_version_key,
            "a shard address from another crate version must never be served"
        );

        let first_uri = path_to_file_uri(first_style.as_path());
        let cache_storage = fixture_cache_storage(first_style.as_path());
        let fresh =
            generate_omena_bridge_sif_for_resolved_style_path_with_cache_context_and_storage(
                first_uri.as_str(),
                &OmenaBridgeExternalSifCacheContextV0::default(),
                Some(&cache_storage),
            )?;
        clear_external_sif_memory_cache_for_storage_for_test(&cache_storage);
        let cached =
            generate_omena_bridge_sif_for_resolved_style_path_with_cache_context_and_storage(
                first_uri.as_str(),
                &OmenaBridgeExternalSifCacheContextV0::default(),
                Some(&cache_storage),
            )?;
        assert_eq!(cached, fresh);
        let cache_dir =
            external_sif_cache_dir_for_path(first_style.as_path(), Some(&cache_storage))
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
    fn global_external_sif_storage_never_cross_serves_workspace_partitions()
    -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_dir("omena_bridge_workspace_partitioned_global_sif")?;
        let workspace_a = root.join("workspace-a");
        let workspace_b = root.join("workspace-b");
        fs::create_dir_all(workspace_a.as_path())?;
        fs::create_dir_all(workspace_b.as_path())?;
        fs::write(
            workspace_a.join("package.json"),
            r#"{"name":"workspace-a"}"#,
        )?;
        fs::write(
            workspace_b.join("package.json"),
            r#"{"name":"workspace-b"}"#,
        )?;
        let shared_package = root.join("node_modules").join("design-system");
        fs::create_dir_all(shared_package.as_path())?;
        let style = shared_package.join("tokens.scss");
        fs::write(style.as_path(), "$brand: #0af;\n")?;

        let platform_cache_home = root.join("global");
        let workspace_identity_a = workspace_a.to_string_lossy().into_owned();
        let workspace_identity_b = workspace_b.to_string_lossy().into_owned();
        let derived_workspace_root = |workspace_root: &Path, workspace_identity: &str| {
            crate::cache_root::resolve_omena_cache_roots(
                crate::cache_root::CacheRootResolverInputsV0 {
                    platform_cache_home: Some(platform_cache_home.as_path()),
                    workspace_root: Some(workspace_root),
                    workspace_identity: Some(workspace_identity),
                    ..crate::cache_root::CacheRootResolverInputsV0::default()
                },
            )
            .workspace
        };
        let workspace_cache_root_a =
            derived_workspace_root(workspace_a.as_path(), workspace_identity_a.as_str())
                .ok_or_else(|| std::io::Error::other("workspace A cache root"))?;
        let workspace_cache_root_b =
            derived_workspace_root(workspace_b.as_path(), workspace_identity_b.as_str())
                .ok_or_else(|| std::io::Error::other("workspace B cache root"))?;
        assert_ne!(workspace_cache_root_a, workspace_cache_root_b);
        let storage_a = OmenaBridgeExternalSifStorageV0::from_workspace_cache_root_and_identity(
            workspace_cache_root_a,
            workspace_identity_a,
        );
        let storage_b = OmenaBridgeExternalSifStorageV0::from_workspace_cache_root_and_identity(
            workspace_cache_root_b,
            workspace_identity_b,
        );
        let cache_context = OmenaBridgeExternalSifCacheContextV0::default();
        clear_external_sif_memory_cache_for_storage_for_test(&storage_a);
        clear_external_sif_memory_cache_for_storage_for_test(&storage_b);
        let sif_a =
            generate_omena_bridge_sif_for_resolved_style_path_with_cache_context_and_storage(
                style.to_string_lossy().as_ref(),
                &cache_context,
                Some(&storage_a),
            )?;
        let sif_b =
            generate_omena_bridge_sif_for_resolved_style_path_with_cache_context_and_storage(
                style.to_string_lossy().as_ref(),
                &cache_context,
                Some(&storage_b),
            )?;
        assert_eq!(sif_a, sif_b);

        let cache_dir_a = storage_a
            .workspace_cache_root()
            .join(EXTERNAL_SIF_CACHE_DIR);
        let cache_dir_b = storage_b
            .workspace_cache_root()
            .join(EXTERNAL_SIF_CACHE_DIR);
        let shard_files = |dir: &Path| -> Vec<PathBuf> {
            let Ok(entries) = fs::read_dir(dir) else {
                return Vec::new();
            };
            let mut files = entries
                .flatten()
                .map(|entry| entry.path())
                .filter(|path| {
                    path.extension()
                        .is_some_and(|extension| extension == "json")
                })
                .collect::<Vec<_>>();
            files.sort();
            files
        };
        let shard_a = shard_files(cache_dir_a.as_path());
        let shard_b = shard_files(cache_dir_b.as_path());
        assert_eq!(shard_a.len(), 1);
        assert_eq!(shard_b.len(), 1);
        assert_ne!(shard_a, shard_b);
        let shard_a_bytes = fs::read(shard_a[0].as_path())?;
        let mut poisoned_shard = serde_json::from_slice::<Value>(shard_a_bytes.as_slice())?;
        let mut poisoned_sif = sif_a.clone();
        poisoned_sif.exports.variables.clear();
        let poisoned_sif_json = write_omena_sif_json_v1(&poisoned_sif)?;
        poisoned_shard["sifJson"] = Value::String(poisoned_sif_json.clone());
        poisoned_shard["payloadDigest"] = Value::String(
            compute_omena_sif_leaf_hash_v1(poisoned_sif_json.as_bytes())
                .as_str()
                .to_string(),
        );
        fs::write(
            shard_a[0].as_path(),
            write_omena_canonical_json_bytes_v1(&poisoned_shard)?,
        )?;
        clear_external_sif_memory_cache_for_storage_for_test(&storage_a);
        let poisoned_a =
            generate_omena_bridge_sif_for_resolved_style_path_with_cache_context_and_storage(
                style.to_string_lossy().as_ref(),
                &cache_context,
                Some(&storage_a),
            )?;
        assert_eq!(poisoned_a, sif_a);
        assert_ne!(poisoned_a, poisoned_sif);
        let isolated_b =
            generate_omena_bridge_sif_for_resolved_style_path_with_cache_context_and_storage(
                style.to_string_lossy().as_ref(),
                &cache_context,
                Some(&storage_b),
            )?;
        assert_eq!(isolated_b, sif_b);
        assert_eq!(isolated_b, poisoned_a);
        assert_ne!(isolated_b, poisoned_sif);
        assert_eq!(shard_files(cache_dir_a.as_path()).len(), 1);
        assert_eq!(shard_files(cache_dir_b.as_path()).len(), 1);
        eprintln!(
            "externalSifStorage globalBase={} workspaceA={} workspaceB={} shardsA=1 shardsB=1 crossWorkspaceServe=false",
            platform_cache_home
                .join("omena")
                .join("workspaces")
                .display(),
            cache_dir_a.display(),
            cache_dir_b.display(),
        );

        let _ = fs::remove_dir_all(root);
        Ok(())
    }

    #[test]
    fn cache_payload_without_recorded_verdict_is_replaced_by_local_regeneration()
    -> Result<(), Box<dyn std::error::Error>> {
        let fixture = recorded_verdict_attack_fixture("cache-payload-without-verdict")?;
        write_poisoned_low_tier_fixture_shard(&fixture)?;
        clear_external_sif_memory_cache_for_storage_for_test(&fixture.storage);

        let loaded =
            generate_omena_bridge_sif_for_resolved_style_path_with_cache_context_storage_and_trust(
                fixture.resolved.as_str(),
                &OmenaBridgeExternalSifCacheContextV0::default(),
                Some(&fixture.storage),
            )?;

        assert_eq!(loaded.sif, fixture.original_sif);
        assert_ne!(loaded.sif, fixture.poisoned_sif);
        fs::remove_dir_all(fixture.root)?;
        Ok(())
    }

    #[test]
    fn deleting_recorded_verdict_does_not_enable_cached_payload_substitution()
    -> Result<(), Box<dyn std::error::Error>> {
        let fixture = recorded_verdict_attack_fixture("deleted-verdict-cache-payload")?;
        write_fixture_recorded_shard_verdict(
            &fixture.storage,
            &fixture.original_sif,
            OmenaSifTrustTierV1::T2,
        )?;
        fs::remove_dir_all(fixture.verdict_dir.as_path())?;
        write_poisoned_low_tier_fixture_shard(&fixture)?;
        clear_external_sif_memory_cache_for_storage_for_test(&fixture.storage);

        let loaded =
            generate_omena_bridge_sif_for_resolved_style_path_with_cache_context_storage_and_trust(
                fixture.resolved.as_str(),
                &OmenaBridgeExternalSifCacheContextV0::default(),
                Some(&fixture.storage),
            )?;

        assert_eq!(loaded.sif, fixture.original_sif);
        assert_ne!(loaded.sif, fixture.poisoned_sif);
        fs::remove_dir_all(fixture.root)?;
        Ok(())
    }

    #[test]
    fn parsed_sif_canonical_url_must_match_requested_resource()
    -> Result<(), Box<dyn std::error::Error>> {
        let fixture = recorded_verdict_attack_fixture("canonical-url-confused-deputy")?;
        let mut substituted_sif = fixture.poisoned_sif.clone();
        substituted_sif.canonical_url = "pkg:untrusted/substituted.scss".to_string();
        let substituted_sif_json = write_omena_sif_json_v1(&substituted_sif)?;
        let substituted_payload_digest =
            compute_omena_sif_leaf_hash_v1(substituted_sif_json.as_bytes());
        let substituted_sif_hash = compute_omena_sif_artifact_hash_v1(&substituted_sif)?;
        let mut shard = fixture.original_shard.clone();
        shard["sifJson"] = Value::String(substituted_sif_json);
        shard["payloadDigest"] = Value::String(substituted_payload_digest.as_str().to_string());
        shard["trustEnvelope"] = serde_json::to_value(OmenaSifShardTrustEnvelopeV1 {
            schema_version: OMENA_SIF_SHARD_TRUST_ENVELOPE_SCHEMA_VERSION_V1.to_string(),
            product: OMENA_SIF_SHARD_TRUST_ENVELOPE_PRODUCT_V1.to_string(),
            trust_tier: OmenaSifTrustTierV1::T1,
            payload_digest: substituted_payload_digest,
            signature: None,
            lock_binding: OmenaSifShardLockBindingV1 {
                canonical_url: fixture.resolved.clone(),
                sif_hash: substituted_sif_hash,
            },
        })?;

        let loaded = validate_external_sif_cache_shard(
            &shard,
            fixture.key.as_str(),
            fixture.resolved.as_str(),
            fixture.source_hash.as_str(),
            fixture.resolved_base_dir.as_str(),
            None,
            &fixture.original_sif,
        );
        assert!(
            loaded.is_err(),
            "a SIF for another canonical URL was accepted"
        );
        fs::remove_dir_all(fixture.root)?;
        Ok(())
    }

    #[test]
    fn unverified_recorded_verdict_never_elevates_memory_or_disk_shards()
    -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_dir("omena_bridge_recorded_shard_verdict")?;
        fs::write(root.join("package.json"), r#"{"name":"workspace"}"#)?;
        let style = root.join("tokens.scss");
        fs::write(style.as_path(), "$brand: #0af;\n")?;
        let resolved = path_to_file_uri(style.as_path());
        let storage = fixture_cache_storage(style.as_path());
        let cache_context = OmenaBridgeExternalSifCacheContextV0::default();

        clear_external_sif_memory_cache_for_storage_for_test(&storage);
        let unsigned =
            generate_omena_bridge_sif_for_resolved_style_path_with_cache_context_and_storage(
                resolved.as_str(),
                &cache_context,
                Some(&storage),
            )?;
        let cache_dir = external_sif_cache_dir_for_path(style.as_path(), Some(&storage))
            .ok_or_else(|| std::io::Error::other("cache dir"))?;
        write_fixture_recorded_shard_verdict(&storage, &unsigned, OmenaSifTrustTierV1::T2)?;
        let verdict_dir = storage
            .recorded_verdict_dir()
            .ok_or_else(|| std::io::Error::other("verdict dir"))?;

        let memory_hit =
            generate_omena_bridge_sif_for_resolved_style_path_with_cache_context_storage_and_trust(
                resolved.as_str(),
                &cache_context,
                Some(&storage),
            )?;
        assert_eq!(
            memory_hit.trust_envelope.trust_tier,
            OmenaSifTrustTierV1::T1
        );
        assert_eq!(
            memory_hit.trust_source,
            OmenaBridgeExternalSifTrustSourceV1::UnsignedLegacy
        );
        clear_external_sif_memory_cache_for_storage_for_test(&storage);
        let disk_hit =
            generate_omena_bridge_sif_for_resolved_style_path_with_cache_context_storage_and_trust(
                resolved.as_str(),
                &cache_context,
                Some(&storage),
            )?;
        assert_eq!(disk_hit.sif, unsigned);
        assert_eq!(disk_hit.trust_envelope.trust_tier, OmenaSifTrustTierV1::T1);
        assert_eq!(
            disk_hit.trust_source,
            OmenaBridgeExternalSifTrustSourceV1::UnsignedLegacy
        );

        let shard_path = only_fixture_cache_shard_path(cache_dir.as_path())?;
        let mut elevated_shard = serde_json::from_slice::<Value>(&fs::read(shard_path)?)?;
        let key = elevated_shard
            .get("key")
            .and_then(Value::as_str)
            .ok_or_else(|| std::io::Error::other("shard key"))?
            .to_string();
        let source_hash = elevated_shard
            .get("sourceHash")
            .and_then(Value::as_str)
            .ok_or_else(|| std::io::Error::other("shard source hash"))?
            .to_string();
        let resolved_base_dir = elevated_shard
            .get("resolvedBaseDir")
            .and_then(Value::as_str)
            .ok_or_else(|| std::io::Error::other("shard resolved base"))?
            .to_string();
        let sif_hash = compute_omena_sif_artifact_hash_v1(&unsigned)?;
        elevated_shard["trustEnvelope"]["trustTier"] = Value::String("t2".to_string());
        elevated_shard["trustEnvelope"]["signature"] =
            serde_json::to_value(omena_sif::OmenaSifShardSignatureV1 {
                algorithm_version: omena_sif::OMENA_SIF_SHARD_SIGNATURE_ALGORITHM_VERSION_V1
                    .to_string(),
                reference: "fixture:keyless-attestation".to_string(),
                signed_payload_digest: sif_hash,
            })?;
        assert_eq!(
            validate_external_sif_cache_shard(
                &elevated_shard,
                key.as_str(),
                resolved.as_str(),
                source_hash.as_str(),
                resolved_base_dir.as_str(),
                Some(verdict_dir),
                &unsigned,
            ),
            Err(OmenaBridgeExternalSifShardRefusalV1::RecordedVerdictSignatureVerificationFailed)
        );

        let mut over_tier = elevated_shard;
        over_tier["trustEnvelope"]["trustTier"] = Value::String("t3".to_string());
        assert_eq!(
            validate_external_sif_cache_shard(
                &over_tier,
                key.as_str(),
                resolved.as_str(),
                source_hash.as_str(),
                resolved_base_dir.as_str(),
                Some(verdict_dir),
                &unsigned,
            ),
            Err(OmenaBridgeExternalSifShardRefusalV1::TierAboveRecordedVerdict)
        );

        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn forged_recorded_verdict_cannot_mint_elevated_trust_for_poisoned_payload()
    -> Result<(), Box<dyn std::error::Error>> {
        let fixture = recorded_verdict_attack_fixture("forged-sidecar")?;
        write_fixture_recorded_shard_verdict(
            &fixture.storage,
            &fixture.poisoned_sif,
            OmenaSifTrustTierV1::T3,
        )?;
        let mut forged = fixture.original_shard.clone();
        forged["sifJson"] = Value::String(fixture.poisoned_sif_json.clone());
        forged["payloadDigest"] =
            Value::String(fixture.poisoned_payload_digest.as_str().to_string());
        forged["trustEnvelope"] = serde_json::to_value(OmenaSifShardTrustEnvelopeV1 {
            schema_version: OMENA_SIF_SHARD_TRUST_ENVELOPE_SCHEMA_VERSION_V1.to_string(),
            product: OMENA_SIF_SHARD_TRUST_ENVELOPE_PRODUCT_V1.to_string(),
            trust_tier: OmenaSifTrustTierV1::T3,
            payload_digest: fixture.poisoned_payload_digest.clone(),
            signature: Some(omena_sif::OmenaSifShardSignatureV1 {
                algorithm_version: omena_sif::OMENA_SIF_SHARD_SIGNATURE_ALGORITHM_VERSION_V1
                    .to_string(),
                reference: "fixture:keyless-attestation".to_string(),
                signed_payload_digest: fixture.poisoned_sif_hash.clone(),
            }),
            lock_binding: OmenaSifShardLockBindingV1 {
                canonical_url: fixture.resolved.clone(),
                sif_hash: fixture.poisoned_sif_hash.clone(),
            },
        })?;

        let attack = validate_external_sif_cache_shard(
            &forged,
            fixture.key.as_str(),
            fixture.resolved.as_str(),
            fixture.source_hash.as_str(),
            fixture.resolved_base_dir.as_str(),
            Some(fixture.verdict_dir.as_path()),
            &fixture.poisoned_sif,
        );
        assert_eq!(
            attack,
            Err(OmenaBridgeExternalSifShardRefusalV1::RecordedVerdictSignatureVerificationFailed),
            "forged local sidecar did not reach the typed signature refusal"
        );
        fs::remove_dir_all(fixture.root)?;
        Ok(())
    }

    #[test]
    fn recorded_verdict_blocks_legacy_schema_downgrade_for_canonical_url()
    -> Result<(), Box<dyn std::error::Error>> {
        let fixture = recorded_verdict_attack_fixture("schema-downgrade")?;
        write_fixture_recorded_shard_verdict(
            &fixture.storage,
            &fixture.original_sif,
            OmenaSifTrustTierV1::T2,
        )?;
        let mut downgraded = fixture.original_shard.clone();
        downgraded["schemaVersion"] =
            Value::String(EXTERNAL_SIF_CACHE_LEGACY_SCHEMA_VERSION.to_string());
        downgraded["sifJson"] = Value::String(fixture.poisoned_sif_json.clone());
        downgraded["payloadDigest"] =
            Value::String(fixture.poisoned_payload_digest.as_str().to_string());
        downgraded
            .as_object_mut()
            .ok_or_else(|| std::io::Error::other("fixture shard object"))?
            .remove("trustEnvelope");

        let attack = validate_external_sif_cache_shard(
            &downgraded,
            fixture.key.as_str(),
            fixture.resolved.as_str(),
            fixture.source_hash.as_str(),
            fixture.resolved_base_dir.as_str(),
            Some(fixture.verdict_dir.as_path()),
            &fixture.poisoned_sif,
        );
        assert_eq!(
            attack,
            Err(OmenaBridgeExternalSifShardRefusalV1::RecordedVerdictDowngrade),
            "schema downgrade did not reach the typed downgrade refusal"
        );
        fs::remove_dir_all(fixture.root)?;
        Ok(())
    }

    #[test]
    fn recorded_verdict_blocks_tier_downgrade_for_canonical_url()
    -> Result<(), Box<dyn std::error::Error>> {
        let fixture = recorded_verdict_attack_fixture("tier-downgrade")?;
        write_fixture_recorded_shard_verdict(
            &fixture.storage,
            &fixture.original_sif,
            OmenaSifTrustTierV1::T2,
        )?;
        let mut downgraded = fixture.original_shard.clone();
        downgraded["sifJson"] = Value::String(fixture.poisoned_sif_json.clone());
        downgraded["payloadDigest"] =
            Value::String(fixture.poisoned_payload_digest.as_str().to_string());
        downgraded["trustEnvelope"] = serde_json::to_value(OmenaSifShardTrustEnvelopeV1 {
            schema_version: OMENA_SIF_SHARD_TRUST_ENVELOPE_SCHEMA_VERSION_V1.to_string(),
            product: OMENA_SIF_SHARD_TRUST_ENVELOPE_PRODUCT_V1.to_string(),
            trust_tier: OmenaSifTrustTierV1::T1,
            payload_digest: fixture.poisoned_payload_digest.clone(),
            signature: None,
            lock_binding: OmenaSifShardLockBindingV1 {
                canonical_url: fixture.resolved.clone(),
                sif_hash: fixture.poisoned_sif_hash.clone(),
            },
        })?;

        let attack = validate_external_sif_cache_shard(
            &downgraded,
            fixture.key.as_str(),
            fixture.resolved.as_str(),
            fixture.source_hash.as_str(),
            fixture.resolved_base_dir.as_str(),
            Some(fixture.verdict_dir.as_path()),
            &fixture.poisoned_sif,
        );
        assert_eq!(
            attack,
            Err(OmenaBridgeExternalSifShardRefusalV1::RecordedVerdictDowngrade),
            "tier downgrade did not reach the typed downgrade refusal"
        );
        fs::remove_dir_all(fixture.root)?;
        Ok(())
    }

    #[test]
    fn omena_published_bundle_is_the_only_fixture_path_to_elevated_advisory_tier()
    -> Result<(), Box<dyn std::error::Error>> {
        const BUNDLE_SHA256: &str =
            "0c99e37ac1b1d3cbfd677416a74218c9a1ca8e28c3aac95c7614549f3b3b0ce1";
        let root = temp_dir("verified-keyless-shard")?;
        let storage = OmenaBridgeExternalSifStorageV0::from_workspace_cache_root(root.clone());
        let verdict_dir = storage
            .recorded_verdict_dir()
            .ok_or_else(|| std::io::Error::other("verdict dir"))?;
        let bundle_dir = verdict_dir.join(EXTERNAL_SIF_RECORDED_BUNDLE_DIR_V1);
        fs::create_dir_all(bundle_dir.as_path())?;
        let bundle_reference = format!(
            "{EXTERNAL_SIF_RECORDED_BUNDLE_DIR_V1}/{BUNDLE_SHA256}{EXTERNAL_SIF_RECORDED_BUNDLE_SUFFIX_V1}"
        );
        let bundle_path = verdict_dir.join(bundle_reference.as_str());
        let bundle_bytes =
            include_bytes!("../tests/fixtures/published-sif-attestation.sigstore.json");
        fs::write(bundle_path.as_path(), bundle_bytes)?;
        let sif_source =
            include_str!("../tests/fixtures/published-sif-attestation.sif.json").trim_end();
        let sif = read_omena_sif_json_v1(sif_source)?;
        write_fixture_recorded_shard_verdict_with_reference(
            &storage,
            &sif,
            OmenaSifTrustTierV1::T3,
            bundle_reference.as_str(),
        )?;

        let elevated = external_sif_result_with_recorded_verdict(sif.clone(), Some(verdict_dir))?;
        assert_eq!(elevated.trust_envelope.trust_tier, OmenaSifTrustTierV1::T3);
        assert_eq!(
            elevated.trust_source,
            OmenaBridgeExternalSifTrustSourceV1::RecordedVerdict
        );
        let payload_digest = compute_omena_sif_leaf_hash_v1(sif_source.as_bytes());
        let shard = json!({
            "schemaVersion": EXTERNAL_SIF_CACHE_SCHEMA_VERSION,
            "product": EXTERNAL_SIF_CACHE_PRODUCT,
            "key": "fixture-key",
            "canonicalUrl": sif.canonical_url,
            "sourceHash": "fixture-source-hash",
            "resolvedBaseDir": "fixture-base",
            "payloadDigest": payload_digest,
            "trustEnvelope": elevated.trust_envelope,
            "sifJson": sif_source,
        });
        let validated = validate_external_sif_cache_shard(
            &shard,
            "fixture-key",
            sif.canonical_url.as_str(),
            "fixture-source-hash",
            "fixture-base",
            Some(verdict_dir),
            &sif,
        )
        .map_err(|refusal| {
            std::io::Error::other(format!("verified fixture shard was refused: {refusal:?}"))
        })?;
        assert_eq!(validated.trust_envelope.trust_tier, OmenaSifTrustTierV1::T3);

        fs::write(bundle_path.as_path(), b"{}")?;
        assert_eq!(
            validate_external_sif_cache_shard(
                &shard,
                "fixture-key",
                sif.canonical_url.as_str(),
                "fixture-source-hash",
                "fixture-base",
                Some(verdict_dir),
                &sif,
            ),
            Err(OmenaBridgeExternalSifShardRefusalV1::RecordedVerdictSignatureVerificationFailed)
        );
        let downgraded = external_sif_result_with_recorded_verdict(sif, Some(verdict_dir))?;
        assert_eq!(
            downgraded.trust_envelope.trust_tier,
            OmenaSifTrustTierV1::T1
        );
        assert_eq!(
            downgraded.trust_source,
            OmenaBridgeExternalSifTrustSourceV1::UnsignedLegacy
        );

        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn published_bundle_cannot_elevate_a_different_sif_artifact()
    -> Result<(), Box<dyn std::error::Error>> {
        const BUNDLE_SHA256: &str =
            "0c99e37ac1b1d3cbfd677416a74218c9a1ca8e28c3aac95c7614549f3b3b0ce1";
        let root = temp_dir("published-bundle-artifact-substitution")?;
        let storage = OmenaBridgeExternalSifStorageV0::from_workspace_cache_root(root.clone());
        let verdict_dir = storage
            .recorded_verdict_dir()
            .ok_or_else(|| std::io::Error::other("verdict dir"))?;
        let bundle_dir = verdict_dir.join(EXTERNAL_SIF_RECORDED_BUNDLE_DIR_V1);
        fs::create_dir_all(bundle_dir.as_path())?;
        let bundle_reference = format!(
            "{EXTERNAL_SIF_RECORDED_BUNDLE_DIR_V1}/{BUNDLE_SHA256}{EXTERNAL_SIF_RECORDED_BUNDLE_SUFFIX_V1}"
        );
        fs::write(
            verdict_dir.join(bundle_reference.as_str()),
            include_bytes!("../tests/fixtures/published-sif-attestation.sigstore.json"),
        )?;
        let mut poisoned_sif = read_omena_sif_json_v1(
            include_str!("../tests/fixtures/published-sif-attestation.sif.json").trim_end(),
        )?;
        poisoned_sif.generator.name = "untrusted-generator".to_string();
        write_fixture_recorded_shard_verdict_with_reference(
            &storage,
            &poisoned_sif,
            OmenaSifTrustTierV1::T3,
            bundle_reference.as_str(),
        )?;
        let poisoned_hash = compute_omena_sif_artifact_hash_v1(&poisoned_sif)?;
        let verdict = load_recorded_shard_verdict(
            Some(verdict_dir),
            poisoned_sif.canonical_url.as_str(),
            &poisoned_hash,
        )
        .ok_or_else(|| std::io::Error::other("poisoned verdict"))?;

        assert_eq!(
            verify_recorded_shard_verdict(Some(verdict_dir), &verdict),
            Err(OmenaBridgeExternalSifShardRefusalV1::RecordedVerdictSignatureVerificationFailed)
        );
        let result = external_sif_result_with_recorded_verdict(poisoned_sif, Some(verdict_dir))?;
        assert_eq!(result.trust_envelope.trust_tier, OmenaSifTrustTierV1::T1);
        assert_eq!(
            result.trust_source,
            OmenaBridgeExternalSifTrustSourceV1::UnsignedLegacy
        );

        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn same_version_legacy_shard_without_trust_fields_remains_workspace_local()
    -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_dir("omena_bridge_legacy_shard")?;
        fs::write(root.join("package.json"), r#"{"name":"workspace"}"#)?;
        let style = root.join("tokens.scss");
        fs::write(style.as_path(), "$brand: #0af;\n")?;
        let resolved = path_to_file_uri(style.as_path());
        let storage = fixture_cache_storage(style.as_path());
        let cache_context = OmenaBridgeExternalSifCacheContextV0::default();
        clear_external_sif_memory_cache_for_storage_for_test(&storage);
        let fresh =
            generate_omena_bridge_sif_for_resolved_style_path_with_cache_context_storage_and_trust(
                resolved.as_str(),
                &cache_context,
                Some(&storage),
            )?;
        let cache_dir = external_sif_cache_dir_for_path(style.as_path(), Some(&storage))
            .ok_or_else(|| std::io::Error::other("cache dir"))?;
        let shard_path = only_fixture_cache_shard_path(cache_dir.as_path())?;
        let mut legacy = serde_json::from_slice::<Value>(&fs::read(shard_path.as_path())?)?;
        legacy["schemaVersion"] =
            Value::String(EXTERNAL_SIF_CACHE_LEGACY_SCHEMA_VERSION.to_string());
        legacy
            .as_object_mut()
            .ok_or_else(|| std::io::Error::other("legacy shard object"))?
            .remove("trustEnvelope");
        fs::write(
            shard_path.as_path(),
            write_omena_canonical_json_bytes_v1(&legacy)?,
        )?;
        clear_external_sif_memory_cache_for_storage_for_test(&storage);
        let loaded =
            generate_omena_bridge_sif_for_resolved_style_path_with_cache_context_storage_and_trust(
                resolved.as_str(),
                &cache_context,
                Some(&storage),
            )?;
        assert_eq!(loaded.sif, fresh.sif);
        assert_eq!(loaded.trust_envelope.trust_tier, OmenaSifTrustTierV1::T1);
        assert_eq!(
            loaded.trust_source,
            OmenaBridgeExternalSifTrustSourceV1::UnsignedLegacy
        );
        let _ = fs::remove_dir_all(root);
        Ok(())
    }

    #[test]
    fn external_sif_disk_cache_root_carries_self_ignore_markers()
    -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_dir("omena_bridge_external_sif_cache_markers")?;
        fs::write(root.join("package.json"), r#"{"name":"workspace"}"#)?;
        let style = root.join("tokens.scss");
        fs::write(style.as_path(), "$brand: #0af;\n")?;

        let cache_storage = fixture_cache_storage(style.as_path());
        generate_omena_bridge_sif_for_resolved_style_path_with_cache_context_and_storage(
            style.to_string_lossy().as_ref(),
            &OmenaBridgeExternalSifCacheContextV0::default(),
            Some(&cache_storage),
        )?;

        let cache_dir = external_sif_cache_dir_for_path(style.as_path(), Some(&cache_storage))
            .ok_or_else(|| std::io::Error::other("cache dir"))?;
        let cache_root = cache_dir
            .parent()
            .ok_or_else(|| std::io::Error::other("cache root"))?;
        assert_eq!(
            fs::read(cache_root.join(".gitignore"))?,
            b"# machine-generated omena cache - safe to delete\n*\n",
            "external SIF cache root {} must self-ignore generated files",
            cache_root.display()
        );
        assert_eq!(
            fs::read(cache_root.join("CACHEDIR.TAG"))?,
            b"Signature: 8a477f597d28d172789f06886806bc55\n# This directory is an omena cache; contents are regenerable.\n",
            "external SIF cache root {} must carry the standard cache tag",
            cache_root.display()
        );
        let attribution = serde_json::from_slice::<Value>(&fs::read(
            cache_root.join(".omena-cache-owner.json"),
        )?)?;
        assert_eq!(
            attribution.get("product").and_then(Value::as_str),
            Some("omena.cache-root-attribution")
        );
        assert!(
            attribution
                .get("workspaceIdentity")
                .and_then(Value::as_str)
                .is_some_and(|identity| !identity.is_empty())
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

    fn fixture_cache_storage(path: &Path) -> OmenaBridgeExternalSifStorageV0 {
        OmenaBridgeExternalSifStorageV0::from_workspace_cache_root(
            path.parent()
                .unwrap_or_else(|| Path::new("."))
                .join(".cache")
                .join("omena"),
        )
    }

    fn only_fixture_cache_shard_path(
        cache_dir: &Path,
    ) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let mut shards = fs::read_dir(cache_dir)?
            .flatten()
            .map(|entry| entry.path())
            .filter(|path| {
                path.extension()
                    .is_some_and(|extension| extension == "json")
            })
            .collect::<Vec<_>>();
        shards.sort();
        if shards.len() != 1 {
            return Err(std::io::Error::other(format!(
                "expected one fixture cache shard, found {}",
                shards.len()
            ))
            .into());
        }
        Ok(shards.remove(0))
    }

    fn write_fixture_recorded_shard_verdict(
        storage: &OmenaBridgeExternalSifStorageV0,
        sif: &OmenaSifV1,
        trust_tier: OmenaSifTrustTierV1,
    ) -> Result<(), Box<dyn std::error::Error>> {
        write_fixture_recorded_shard_verdict_with_reference(
            storage,
            sif,
            trust_tier,
            "fixture:keyless-attestation",
        )
    }

    fn write_fixture_recorded_shard_verdict_with_reference(
        storage: &OmenaBridgeExternalSifStorageV0,
        sif: &OmenaSifV1,
        trust_tier: OmenaSifTrustTierV1,
        signature_reference: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let sif_hash = compute_omena_sif_artifact_hash_v1(sif)?;
        let verdict = OmenaSifShardRecordedVerdictV1 {
            schema_version: omena_sif::OMENA_SIF_SHARD_RECORDED_VERDICT_SCHEMA_VERSION_V1
                .to_string(),
            product: omena_sif::OMENA_SIF_SHARD_RECORDED_VERDICT_PRODUCT_V1.to_string(),
            verification_owner: omena_sif::OMENA_SIF_SHARD_VERIFICATION_OWNER_V1.to_string(),
            canonical_url: sif.canonical_url.clone(),
            sif_hash: sif_hash.clone(),
            trust_tier,
            signature: omena_sif::OmenaSifShardSignatureV1 {
                algorithm_version: omena_sif::OMENA_SIF_SHARD_SIGNATURE_ALGORITHM_VERSION_V1
                    .to_string(),
                reference: signature_reference.to_string(),
                signed_payload_digest: sif_hash.clone(),
            },
        };
        omena_sif::validate_omena_sif_shard_recorded_verdict_v1(&verdict)?;
        let verdict_dir = storage
            .recorded_verdict_dir()
            .ok_or_else(|| std::io::Error::other("verdict dir"))?;
        fs::create_dir_all(verdict_dir)?;
        let address = compute_omena_sif_shard_recorded_verdict_address_v1(
            sif.canonical_url.as_str(),
            &sif_hash,
        )?;
        let hex = address
            .as_str()
            .strip_prefix("blake3:")
            .ok_or_else(|| std::io::Error::other("verdict address"))?;
        let source = omena_sif::write_omena_sif_shard_recorded_verdict_json_v1(&verdict)?;
        fs::write(verdict_dir.join(format!("{hex}.json")), source)?;
        Ok(())
    }

    struct RecordedVerdictAttackFixture {
        root: PathBuf,
        resolved: String,
        storage: OmenaBridgeExternalSifStorageV0,
        verdict_dir: PathBuf,
        key: String,
        source_hash: String,
        resolved_base_dir: String,
        original_shard: Value,
        original_sif: OmenaSifV1,
        poisoned_sif: OmenaSifV1,
        poisoned_sif_json: String,
        poisoned_payload_digest: omena_sif::OmenaSifDigestV1,
        poisoned_sif_hash: omena_sif::OmenaSifDigestV1,
    }

    fn recorded_verdict_attack_fixture(
        label: &str,
    ) -> Result<RecordedVerdictAttackFixture, Box<dyn std::error::Error>> {
        let root = temp_dir(label)?;
        fs::write(root.join("package.json"), r#"{"name":"workspace"}"#)?;
        let style = root.join("tokens.scss");
        fs::write(style.as_path(), "$brand: #0af;\n")?;
        let resolved = path_to_file_uri(style.as_path());
        let storage = fixture_cache_storage(style.as_path());
        clear_external_sif_memory_cache_for_storage_for_test(&storage);
        let original_sif =
            generate_omena_bridge_sif_for_resolved_style_path_with_cache_context_and_storage(
                resolved.as_str(),
                &OmenaBridgeExternalSifCacheContextV0::default(),
                Some(&storage),
            )?;
        let cache_dir = external_sif_cache_dir_for_path(style.as_path(), Some(&storage))
            .ok_or_else(|| std::io::Error::other("cache dir"))?;
        let original_shard = serde_json::from_slice::<Value>(&fs::read(
            only_fixture_cache_shard_path(cache_dir.as_path())?,
        )?)?;
        let key = original_shard
            .get("key")
            .and_then(Value::as_str)
            .ok_or_else(|| std::io::Error::other("shard key"))?
            .to_string();
        let source_hash = original_shard
            .get("sourceHash")
            .and_then(Value::as_str)
            .ok_or_else(|| std::io::Error::other("source hash"))?
            .to_string();
        let resolved_base_dir = original_shard
            .get("resolvedBaseDir")
            .and_then(Value::as_str)
            .ok_or_else(|| std::io::Error::other("resolved base"))?
            .to_string();
        let mut poisoned_sif = original_sif.clone();
        poisoned_sif.exports.variables.clear();
        let poisoned_sif_json = write_omena_sif_json_v1(&poisoned_sif)?;
        let poisoned_payload_digest = compute_omena_sif_leaf_hash_v1(poisoned_sif_json.as_bytes());
        let poisoned_sif_hash = compute_omena_sif_artifact_hash_v1(&poisoned_sif)?;
        let verdict_dir = storage
            .recorded_verdict_dir()
            .ok_or_else(|| std::io::Error::other("verdict dir"))?
            .to_path_buf();
        Ok(RecordedVerdictAttackFixture {
            root,
            resolved,
            storage,
            verdict_dir,
            key,
            source_hash,
            resolved_base_dir,
            original_shard,
            original_sif,
            poisoned_sif,
            poisoned_sif_json,
            poisoned_payload_digest,
            poisoned_sif_hash,
        })
    }

    fn write_poisoned_low_tier_fixture_shard(
        fixture: &RecordedVerdictAttackFixture,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut poisoned = fixture.original_shard.clone();
        poisoned["sifJson"] = Value::String(fixture.poisoned_sif_json.clone());
        poisoned["payloadDigest"] =
            Value::String(fixture.poisoned_payload_digest.as_str().to_string());
        poisoned["trustEnvelope"] = serde_json::to_value(OmenaSifShardTrustEnvelopeV1 {
            schema_version: OMENA_SIF_SHARD_TRUST_ENVELOPE_SCHEMA_VERSION_V1.to_string(),
            product: OMENA_SIF_SHARD_TRUST_ENVELOPE_PRODUCT_V1.to_string(),
            trust_tier: OmenaSifTrustTierV1::T1,
            payload_digest: fixture.poisoned_payload_digest.clone(),
            signature: None,
            lock_binding: OmenaSifShardLockBindingV1 {
                canonical_url: fixture.resolved.clone(),
                sif_hash: fixture.poisoned_sif_hash.clone(),
            },
        })?;
        let cache_dir = fixture
            .storage
            .workspace_cache_root()
            .join(EXTERNAL_SIF_CACHE_DIR);
        let shard_path = only_fixture_cache_shard_path(cache_dir.as_path())?;
        fs::write(shard_path, write_omena_canonical_json_bytes_v1(&poisoned)?)?;
        Ok(())
    }

    fn generate_fixture_sif_for_resolved_style_path(
        path: &Path,
        resolved_path: &str,
    ) -> Result<OmenaSifV1, String> {
        let cache_storage = fixture_cache_storage(path);
        generate_omena_bridge_sif_for_resolved_style_path_with_cache_context_and_storage(
            resolved_path,
            &OmenaBridgeExternalSifCacheContextV0::default(),
            Some(&cache_storage),
        )
    }
}
