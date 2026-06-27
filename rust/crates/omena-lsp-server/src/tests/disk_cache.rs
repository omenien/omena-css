use super::*;
use omena_query::{
    OmenaQuerySourceDocumentInputV0, OmenaQueryStylePackageManifestV0, OmenaQueryStyleSourceInputV0,
};
use std::path::PathBuf;

const DISK_CACHE_STYLE_TEXT: &str =
    ":root { --brand: red; }\n.btn { width: var(--missing); color: red; color: blue; }";

fn disk_cache_workspace_root(suffix: &str) -> PathBuf {
    let workspace_root = std::env::temp_dir().join(format!(
        "omena-lsp-server-disk-cache-{suffix}-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&workspace_root);
    workspace_root
}

fn write_disk_cache_style_fixture(workspace_root: &Path, text: &str) -> (String, String) {
    let src_dir = workspace_root.join("src");
    let style_path = src_dir.join("App.module.scss");
    let create_dir_result = std::fs::create_dir_all(&src_dir);
    assert!(
        create_dir_result.is_ok(),
        "create disk-cache fixture directory: {:?}",
        create_dir_result.err(),
    );
    let write_result = std::fs::write(&style_path, text);
    assert!(
        write_result.is_ok(),
        "write disk-cache style fixture: {:?}",
        write_result.err(),
    );
    (
        format!("file://{}", workspace_root.display()),
        format!("file://{}", style_path.display()),
    )
}

fn run_disk_cache_session(
    workspace_uri: &str,
    style_uri: &str,
    style_text: &str,
) -> Vec<ScheduledLspOutput> {
    let mut state = LspShellState::default();
    let initialize_response = handle_lsp_message(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "workspaceFolders": [
                    {
                        "uri": workspace_uri,
                        "name": "disk-cache",
                    },
                ],
            },
        }),
    );
    assert!(initialize_response.is_some());
    handle_lsp_message_scheduled_outputs(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "method": "initialized",
            "params": {},
        }),
    );
    handle_lsp_message_scheduled_outputs(
        &mut state,
        json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didOpen",
            "params": {
                "textDocument": {
                    "uri": style_uri,
                    "languageId": "scss",
                    "version": 1,
                    "text": style_text,
                },
            },
        }),
    )
}

fn disk_cache_dir(workspace_root: &Path) -> PathBuf {
    workspace_root.join(".cache/omena/diagnostics-cache-v0")
}

fn shard_files(cache_dir: &Path) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(cache_dir) else {
        return Vec::new();
    };
    let mut files = entries
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|extension| extension.to_str()) == Some("json"))
        .collect::<Vec<_>>();
    files.sort();
    files
}

fn serialized_outputs(outputs: &[ScheduledLspOutput]) -> String {
    serde_json::to_string(
        &outputs
            .iter()
            .map(|output| {
                json!({
                    "delayMillis": output.delay_millis,
                    "coalesceKey": output.coalesce_key,
                    "value": output.value,
                })
            })
            .collect::<Vec<_>>(),
    )
    .unwrap_or_default()
}

fn outputs_contain_diagnostic_code(outputs: &[ScheduledLspOutput], code: &str) -> bool {
    outputs.iter().any(|output| {
        output
            .value
            .pointer("/params/diagnostics")
            .and_then(Value::as_array)
            .is_some_and(|diagnostics| {
                diagnostics
                    .iter()
                    .any(|diagnostic| diagnostic.pointer("/code") == Some(&json!(code)))
            })
    })
}

#[test]
fn disk_cache_key_includes_style_resolution_disk_identity_snapshot() {
    let style_sources = vec![OmenaQueryStyleSourceInputV0 {
        style_path: "file:///workspace/src/App.module.scss".to_string(),
        style_source: "@use \"@styles/tokens\";".to_string(),
    }];
    let source_documents = Vec::<OmenaQuerySourceDocumentInputV0>::new();
    let external_sifs = Vec::<OmenaQueryExternalSifInputV0>::new();
    let package_manifests = Vec::<OmenaQueryStylePackageManifestV0>::new();
    let base_inputs = omena_query::OmenaQueryStyleResolutionInputsV0::default();
    let disk_backed_inputs = omena_query::OmenaQueryStyleResolutionInputsV0 {
        disk_style_path_identities: vec![
            omena_query::OmenaQueryStyleModuleDiskCandidateIdentityV0 {
                style_path: "/workspace/src/tokens.scss".to_string(),
                metadata_identity: "file|len12|mtime1".to_string(),
            },
        ],
        ..Default::default()
    };

    let base_key = crate::disk_cache::disk_diagnostics_cache_key_v0(
        &crate::disk_cache::DiskDiagnosticsCacheKeyComponentsV0 {
            target_style_path: "file:///workspace/src/App.module.scss",
            style_sources: style_sources.as_slice(),
            source_documents: source_documents.as_slice(),
            package_manifests: package_manifests.as_slice(),
            external_sifs: external_sifs.as_slice(),
            resolution_inputs: &base_inputs,
            severity: 2,
            deep_analysis: false,
        },
    );
    let disk_backed_key = crate::disk_cache::disk_diagnostics_cache_key_v0(
        &crate::disk_cache::DiskDiagnosticsCacheKeyComponentsV0 {
            target_style_path: "file:///workspace/src/App.module.scss",
            style_sources: style_sources.as_slice(),
            source_documents: source_documents.as_slice(),
            package_manifests: package_manifests.as_slice(),
            external_sifs: external_sifs.as_slice(),
            resolution_inputs: &disk_backed_inputs,
            severity: 2,
            deep_analysis: false,
        },
    );

    assert_ne!(base_key, disk_backed_key);
}

#[test]
fn first_resolve_writes_shard_and_fresh_state_replays_byte_identical_diagnostics() {
    let workspace_root = disk_cache_workspace_root("replay");
    let (workspace_uri, style_uri) =
        write_disk_cache_style_fixture(&workspace_root, DISK_CACHE_STYLE_TEXT);

    let first_outputs = run_disk_cache_session(
        workspace_uri.as_str(),
        style_uri.as_str(),
        DISK_CACHE_STYLE_TEXT,
    );
    assert!(
        outputs_contain_diagnostic_code(&first_outputs, "missingCustomProperty"),
        "first session must compute real diagnostics",
    );
    let cache_dir = disk_cache_dir(&workspace_root);
    assert_eq!(
        shard_files(&cache_dir).len(),
        1,
        "first resolve must write exactly one shard under {}",
        cache_dir.display(),
    );

    let second_outputs = run_disk_cache_session(
        workspace_uri.as_str(),
        style_uri.as_str(),
        DISK_CACHE_STYLE_TEXT,
    );
    assert_eq!(
        serialized_outputs(&first_outputs),
        serialized_outputs(&second_outputs),
        "a fresh state with identical inputs must publish byte-equal payloads",
    );
    assert_eq!(
        shard_files(&cache_dir).len(),
        1,
        "an exact-key hit must not write additional shards",
    );
}

#[test]
fn exact_key_hit_serves_diagnostics_from_the_shard_on_disk() -> Result<(), String> {
    let workspace_root = disk_cache_workspace_root("sentinel");
    let (workspace_uri, style_uri) =
        write_disk_cache_style_fixture(&workspace_root, DISK_CACHE_STYLE_TEXT);

    run_disk_cache_session(
        workspace_uri.as_str(),
        style_uri.as_str(),
        DISK_CACHE_STYLE_TEXT,
    );
    let cache_dir = disk_cache_dir(&workspace_root);
    let shards = shard_files(&cache_dir);
    let shard_path = shards.first().ok_or("first session must write a shard")?;

    // Replace the shard payload while keeping schema/key/target intact: the
    // follow-up session publishing the sentinel proves the diagnostics were
    // served from the disk shard rather than recomputed.
    let shard_source =
        std::fs::read_to_string(shard_path).map_err(|error| format!("read shard: {error}"))?;
    let mut shard: Value =
        serde_json::from_str(shard_source.as_str()).map_err(|error| format!("parse: {error}"))?;
    shard["diagnosticsJson"] = json!([
        {
            "range": {
                "start": {"line": 0, "character": 0},
                "end": {"line": 0, "character": 1},
            },
            "severity": 1,
            "code": "diskCacheSentinel",
            "source": "omena-css",
            "message": "served from the tampered shard",
            "data": {},
        },
    ]);
    // The output digest binds the payload to the shard, so the sentinel swap
    // must re-digest its payload (computed independently of the production
    // code path) — and a swap WITHOUT a matching digest must be rejected.
    let sentinel_digest = omena_sif::compute_omena_sif_leaf_hash_v1(
        omena_sif::write_omena_canonical_json_bytes_v1(&shard["diagnosticsJson"])
            .map_err(|error| format!("canonicalize sentinel: {error}"))?
            .as_slice(),
    )
    .as_str()
    .to_string();
    let stale_digest_shard =
        serde_json::to_vec(&shard).map_err(|error| format!("serialize stale: {error}"))?;
    std::fs::write(shard_path, stale_digest_shard)
        .map_err(|error| format!("write stale: {error}"))?;
    let stale_outputs = run_disk_cache_session(
        workspace_uri.as_str(),
        style_uri.as_str(),
        DISK_CACHE_STYLE_TEXT,
    );
    assert!(
        !outputs_contain_diagnostic_code(&stale_outputs, "diskCacheSentinel"),
        "a payload swap without a matching output digest must be rejected",
    );

    shard["outputDigest"] = json!(sentinel_digest);
    let tampered =
        serde_json::to_vec(&shard).map_err(|error| format!("serialize tampered: {error}"))?;
    std::fs::write(shard_path, tampered).map_err(|error| format!("write tampered: {error}"))?;

    let outputs = run_disk_cache_session(
        workspace_uri.as_str(),
        style_uri.as_str(),
        DISK_CACHE_STYLE_TEXT,
    );
    assert!(
        outputs_contain_diagnostic_code(&outputs, "diskCacheSentinel"),
        "an exact key match with a bound digest must serve the shard content from disk",
    );
    assert!(
        !outputs_contain_diagnostic_code(&outputs, "missingCustomProperty"),
        "a shard hit must not recompute diagnostics",
    );
    Ok(())
}

#[test]
fn edited_document_text_misses_the_shard_and_recomputes() -> Result<(), String> {
    let workspace_root = disk_cache_workspace_root("miss");
    let (workspace_uri, style_uri) =
        write_disk_cache_style_fixture(&workspace_root, DISK_CACHE_STYLE_TEXT);

    run_disk_cache_session(
        workspace_uri.as_str(),
        style_uri.as_str(),
        DISK_CACHE_STYLE_TEXT,
    );
    let cache_dir = disk_cache_dir(&workspace_root);
    let shards = shard_files(&cache_dir);
    let shard_path = shards.first().ok_or("first session must write a shard")?;
    let shard_source =
        std::fs::read_to_string(shard_path).map_err(|error| format!("read shard: {error}"))?;
    let mut shard: Value =
        serde_json::from_str(shard_source.as_str()).map_err(|error| format!("parse: {error}"))?;
    shard["diagnosticsJson"] = json!([{"code": "diskCacheSentinel"}]);
    let tampered =
        serde_json::to_vec(&shard).map_err(|error| format!("serialize tampered: {error}"))?;
    std::fs::write(shard_path, tampered).map_err(|error| format!("write tampered: {error}"))?;

    // Different buffer text => different composite key => the tampered shard
    // must be ignored and the diagnostics recomputed (and written as a NEW
    // shard alongside the old one).
    let edited_text = ":root { --brand: red; }\n.btn { width: var(--missing); }";
    let outputs = run_disk_cache_session(workspace_uri.as_str(), style_uri.as_str(), edited_text);
    assert!(
        !outputs_contain_diagnostic_code(&outputs, "diskCacheSentinel"),
        "an edited document must not serve the stale shard",
    );
    assert!(
        outputs_contain_diagnostic_code(&outputs, "missingCustomProperty"),
        "an edited document must recompute real diagnostics",
    );
    assert_eq!(
        shard_files(&cache_dir).len(),
        2,
        "the recompute must write-behind a second shard",
    );
    Ok(())
}
