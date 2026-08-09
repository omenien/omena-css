use crate::LspShellState;
use crate::boundary::{
    CacheStorageRungV0, CacheWriteSurfaceKindV0, declared_cache_write_surfaces_for_rungs,
};
use crate::protocol::file_uri_to_path;
use serde::Serialize;
use std::{fs, path::PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
struct ResolvedOwnedCacheTargetV0 {
    root_kind: CacheWriteSurfaceKindV0,
    resolved_rung: CacheStorageRungV0,
    root: PathBuf,
    cache_directories: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LspCacheClearReportV0 {
    schema_version: &'static str,
    product: &'static str,
    target_count: usize,
    removed_path_count: usize,
    targets: Vec<LspCacheClearTargetReportV0>,
    failures: Vec<LspCacheClearFailureV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
struct LspCacheClearTargetReportV0 {
    root_kind: CacheWriteSurfaceKindV0,
    resolved_rung: CacheStorageRungV0,
    root: String,
    removed_paths: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
struct LspCacheClearFailureV0 {
    path: String,
    reason: String,
}

pub(crate) fn clear_owned_cache_paths(state: &LspShellState) -> LspCacheClearReportV0 {
    let targets = resolved_owned_cache_targets(state);
    let mut target_reports = Vec::with_capacity(targets.len());
    let mut failures = Vec::new();
    let mut removed_path_count = 0usize;

    for target in targets.iter() {
        let mut removed_paths = Vec::new();
        for cache_directory in target.cache_directories.iter() {
            let path = target.root.join(cache_directory);
            let metadata = match fs::symlink_metadata(path.as_path()) {
                Ok(metadata) => metadata,
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
                Err(error) => {
                    failures.push(LspCacheClearFailureV0 {
                        path: path.to_string_lossy().into_owned(),
                        reason: error.to_string(),
                    });
                    continue;
                }
            };
            let result = if metadata.file_type().is_dir() {
                fs::remove_dir_all(path.as_path())
            } else {
                fs::remove_file(path.as_path())
            };
            match result {
                Ok(()) => {
                    removed_path_count = removed_path_count.saturating_add(1);
                    removed_paths.push(path.to_string_lossy().into_owned());
                }
                Err(error) => failures.push(LspCacheClearFailureV0 {
                    path: path.to_string_lossy().into_owned(),
                    reason: error.to_string(),
                }),
            }
        }
        target_reports.push(LspCacheClearTargetReportV0 {
            root_kind: target.root_kind,
            resolved_rung: target.resolved_rung,
            root: target.root.to_string_lossy().into_owned(),
            removed_paths,
        });
    }

    LspCacheClearReportV0 {
        schema_version: "0",
        product: "omena-lsp-server.cache-clear-report",
        target_count: targets.len(),
        removed_path_count,
        targets: target_reports,
        failures,
    }
}

fn resolved_owned_cache_targets(state: &LspShellState) -> Vec<ResolvedOwnedCacheTargetV0> {
    let mut targets = Vec::new();
    for folder in state.workspace_runtime_registry.folder_snapshots() {
        let Some(workspace_root) = file_uri_to_path(folder.uri.as_str()) else {
            continue;
        };
        let lsp_roots = crate::cache_root::process_cache_roots(
            &state.resolution.cache_storage,
            folder.uri.as_str(),
            workspace_root.as_path(),
        );
        let bridge_roots = crate::cache_root::process_bridge_cache_roots(
            &state.resolution.cache_storage,
            folder.uri.as_str(),
            workspace_root.as_path(),
        );
        let lsp_rung = CacheStorageRungV0::from(lsp_roots.source);
        let bridge_rung = CacheStorageRungV0::from(bridge_roots.source);
        // Structural entailment: the boundary declaration is the sole owner
        // of clearable directory names. Re-enter this code whenever a cache
        // writer changes that declaration; the foreign-file test below keeps
        // root-wide deletion outside the entailed surface.
        let declared = declared_cache_write_surfaces_for_rungs(lsp_rung, bridge_rung);
        for surface in declared {
            let root = match surface.root_kind {
                CacheWriteSurfaceKindV0::LspWorkspaceCache => lsp_roots.workspace.clone(),
                CacheWriteSurfaceKindV0::BridgeExternalSifCache => bridge_roots.workspace.clone(),
            };
            let Some(root) = root else {
                continue;
            };
            if targets.iter().any(|target: &ResolvedOwnedCacheTargetV0| {
                target.root_kind == surface.root_kind && target.root == root
            }) {
                continue;
            }
            targets.push(ResolvedOwnedCacheTargetV0 {
                root_kind: surface.root_kind,
                resolved_rung: surface.resolved_rung,
                root,
                cache_directories: surface.cache_directories,
            });
        }
    }
    for document in state
        .documents
        .values()
        .map(AsRef::as_ref)
        .filter(|document| {
            document.workspace_folder_uri.is_none()
                && crate::protocol::is_style_document_uri(document.uri.as_str())
        })
    {
        let Some(document_path) = file_uri_to_path(document.uri.as_str()) else {
            continue;
        };
        let Some(document_root) = document_path.parent() else {
            continue;
        };
        let workspace_identity = document_root.to_string_lossy();
        let bridge_roots = crate::cache_root::process_bridge_cache_roots(
            &state.resolution.cache_storage,
            workspace_identity.as_ref(),
            document_root,
        );
        let bridge_rung = CacheStorageRungV0::from(bridge_roots.source);
        let Some(root) = bridge_roots.workspace else {
            continue;
        };
        let Some(surface) =
            declared_cache_write_surfaces_for_rungs(CacheStorageRungV0::Disabled, bridge_rung)
                .into_iter()
                .find(|surface| {
                    surface.root_kind == CacheWriteSurfaceKindV0::BridgeExternalSifCache
                })
        else {
            continue;
        };
        if targets.iter().any(|target| {
            target.root_kind == CacheWriteSurfaceKindV0::BridgeExternalSifCache
                && target.root == root
        }) {
            continue;
        }
        targets.push(ResolvedOwnedCacheTargetV0 {
            root_kind: surface.root_kind,
            resolved_rung: surface.resolved_rung,
            root,
            cache_directories: surface.cache_directories,
        });
    }
    targets.sort_by(|left, right| {
        format!("{:?}", left.root_kind)
            .cmp(&format!("{:?}", right.root_kind))
            .then_with(|| left.root.cmp(&right.root))
    });
    targets
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cache_root::{CacheLocationV0, LspCacheStorageConfigV0};
    use serde_json::json;

    #[test]
    fn clear_cache_targets_equal_the_declared_set_and_preserve_foreign_files()
    -> Result<(), Box<dyn std::error::Error>> {
        let temp_root =
            std::env::temp_dir().join(format!("omena-clear-owned-caches-{}", std::process::id()));
        let workspace_root = temp_root.join("workspace");
        let editor_root = temp_root.join("editor");
        fs::create_dir_all(workspace_root.as_path())?;
        let workspace_uri = crate::protocol::path_to_file_uri(workspace_root.as_path());
        let mut state = LspShellState::default();
        state
            .workspace_runtime_registry
            .insert(workspace_uri.clone(), "workspace");
        state.resolution.cache_storage = LspCacheStorageConfigV0 {
            initialization_global_storage: Some(editor_root.join("global")),
            initialization_workspace_storage: Some(editor_root.join("workspace")),
            location: CacheLocationV0::Editor,
            ..LspCacheStorageConfigV0::default()
        };

        let targets = resolved_owned_cache_targets(&state);
        let expected = declared_cache_write_surfaces_for_rungs(
            CacheStorageRungV0::InitializationOptions,
            CacheStorageRungV0::InitializationOptions,
        );
        assert_eq!(targets.len(), expected.len());
        for expected_surface in expected.iter() {
            let target = targets
                .iter()
                .find(|target| target.root_kind == expected_surface.root_kind)
                .ok_or_else(|| std::io::Error::other("declared cache target missing"))?;
            assert_eq!(target.resolved_rung, expected_surface.resolved_rung);
            assert_eq!(target.cache_directories, expected_surface.cache_directories);
        }

        for target in targets.iter() {
            fs::create_dir_all(target.root.as_path())?;
            fs::write(target.root.join("foreign-owner.txt"), b"keep")?;
            for cache_directory in target.cache_directories.iter() {
                let owned = target.root.join(cache_directory);
                fs::create_dir_all(owned.as_path())?;
                fs::write(owned.join("shard.json"), b"{}")?;
            }
        }

        let response = crate::handle_lsp_message(
            &mut state,
            json!({
                "jsonrpc": "2.0",
                "id": 17,
                "method": crate::CLEAR_CACHES_REQUEST,
                "params": {},
            }),
        )
        .ok_or_else(|| std::io::Error::other("clear-caches response missing"))?;
        let report = response
            .get("result")
            .ok_or_else(|| std::io::Error::other("clear-caches result missing"))?;
        assert_eq!(
            report.get("targetCount").and_then(|value| value.as_u64()),
            Some(2)
        );
        assert_eq!(
            report
                .get("removedPathCount")
                .and_then(|value| value.as_u64()),
            Some(5)
        );
        assert_eq!(
            report
                .get("failures")
                .and_then(|value| value.as_array())
                .map(Vec::len),
            Some(0)
        );
        for target in targets.iter() {
            assert!(target.root.join("foreign-owner.txt").is_file());
            for cache_directory in target.cache_directories.iter() {
                assert!(!target.root.join(cache_directory).exists());
            }
        }
        eprintln!(
            "cacheClear targets=2 removedPaths=5 failures=0 foreignFilesSurvive=2 declaredSetEqual=true"
        );

        let _ = fs::remove_dir_all(temp_root);
        Ok(())
    }

    #[test]
    fn clear_cache_report_has_no_targets_without_a_workspace() {
        let report = clear_owned_cache_paths(&LspShellState::default());
        assert_eq!(report.target_count, 0);
        assert_eq!(report.removed_path_count, 0);
        assert!(report.targets.is_empty());
        assert!(report.failures.is_empty());
    }

    #[test]
    fn clear_cache_targets_include_folderless_style_document_partition()
    -> Result<(), Box<dyn std::error::Error>> {
        let temp_root = std::env::temp_dir().join(format!(
            "omena-clear-folderless-cache-{}",
            std::process::id()
        ));
        let editor_global = temp_root.join("editor-global");
        let document_path = temp_root.join("loose").join("tokens.scss");
        fs::create_dir_all(
            document_path
                .parent()
                .ok_or_else(|| std::io::Error::other("document parent"))?,
        )?;
        fs::write(document_path.as_path(), "$brand: #0af;\n")?;
        let document_uri = crate::protocol::path_to_file_uri(document_path.as_path());
        let mut state = LspShellState::default();
        crate::handle_lsp_message(
            &mut state,
            json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "initialize",
                "params": {
                    "initializationOptions": {
                        "storage": {
                            "globalStoragePath": editor_global,
                            "location": "editor"
                        }
                    }
                }
            }),
        );
        crate::handle_lsp_message(
            &mut state,
            json!({
                "jsonrpc": "2.0",
                "method": "textDocument/didOpen",
                "params": {
                    "textDocument": {
                        "uri": document_uri,
                        "languageId": "scss",
                        "version": 1,
                        "text": "$brand: #0af;\n"
                    }
                }
            }),
        );

        let targets = resolved_owned_cache_targets(&state);
        assert_eq!(targets.len(), 1);
        let target = targets
            .first()
            .ok_or_else(|| std::io::Error::other("folderless bridge target"))?;
        assert_eq!(
            target.root_kind,
            CacheWriteSurfaceKindV0::BridgeExternalSifCache
        );
        assert_eq!(
            target.resolved_rung,
            CacheStorageRungV0::InitializationOptions
        );
        assert!(
            target
                .root
                .starts_with(editor_global.join("omena/workspaces"))
        );
        fs::create_dir_all(target.root.join("external-sif-v0"))?;
        fs::write(target.root.join("foreign-owner.txt"), b"keep")?;
        let report = clear_owned_cache_paths(&state);
        assert_eq!(report.target_count, 1);
        assert_eq!(report.removed_path_count, 1);
        assert!(target.root.join("foreign-owner.txt").is_file());
        eprintln!(
            "folderlessCacheClear targets=1 removedPaths=1 foreignFileSurvives=true rung=initializationOptions"
        );

        let _ = fs::remove_dir_all(temp_root);
        Ok(())
    }
}
