use crate::disk_cache::stable_cache_shard_address;
use serde::Serialize;
use serde_json::Value;
use std::path::{Path, PathBuf};

pub(crate) const OMENA_CACHE_DIR_ENV: &str = "OMENA_CACHE_DIR";
pub(crate) const OMENA_GLOBAL_CACHE_DIR_ENV: &str = "OMENA_GLOBAL_CACHE_DIR";
const WORKSPACE_CACHE_ADDRESS_PRODUCT: &str = "omena-lsp-server.workspace-cache-root";

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) enum CacheLocationV0 {
    Editor,
    #[default]
    Workspace,
    Global,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct LspCacheStorageConfigV0 {
    pub(crate) initialization_global_storage: Option<PathBuf>,
    pub(crate) initialization_workspace_storage: Option<PathBuf>,
    pub(crate) log_path: Option<PathBuf>,
    pub(crate) command_cache_dir: Option<PathBuf>,
    pub(crate) location: CacheLocationV0,
}

impl LspCacheStorageConfigV0 {
    pub(crate) fn standalone(command_cache_dir: Option<PathBuf>) -> Self {
        Self {
            command_cache_dir,
            location: CacheLocationV0::Editor,
            ..Self::default()
        }
    }

    pub(crate) fn apply_initialization_options(&mut self, params: Option<&Value>) {
        let Some(storage) = params
            .and_then(|value| value.pointer("/initializationOptions/storage"))
            .and_then(Value::as_object)
        else {
            return;
        };
        self.initialization_global_storage = storage
            .get("globalStoragePath")
            .and_then(Value::as_str)
            .filter(|value| !value.is_empty())
            .map(PathBuf::from);
        self.initialization_workspace_storage = storage
            .get("workspaceStoragePath")
            .and_then(Value::as_str)
            .filter(|value| !value.is_empty())
            .map(PathBuf::from);
        self.log_path = storage
            .get("logPath")
            .and_then(Value::as_str)
            .filter(|value| !value.is_empty())
            .map(PathBuf::from);
        self.location = storage
            .get("location")
            .and_then(Value::as_str)
            .and_then(CacheLocationV0::from_wire)
            .unwrap_or(CacheLocationV0::Editor);
    }

    pub(crate) fn apply_location_setting(&mut self, value: Option<&Value>) -> bool {
        let Some(location) = value
            .and_then(Value::as_str)
            .and_then(CacheLocationV0::from_wire)
        else {
            return false;
        };
        let changed = self.location != location;
        self.location = location;
        changed
    }
}

impl CacheLocationV0 {
    fn from_wire(value: &str) -> Option<Self> {
        match value {
            "editor" => Some(Self::Editor),
            "workspace" => Some(Self::Workspace),
            "global" => Some(Self::Global),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) enum CacheRootSourceV0 {
    InitializationOptions,
    Environment,
    Platform,
    Workspace,
    Disabled,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct OmenaCacheRootsV0 {
    pub(crate) global: Option<PathBuf>,
    pub(crate) workspace: Option<PathBuf>,
    pub(crate) source: CacheRootSourceV0,
}

#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct CacheRootResolverInputsV0<'a> {
    pub(crate) initialization_global_storage: Option<&'a Path>,
    pub(crate) initialization_workspace_storage: Option<&'a Path>,
    pub(crate) environment_global_cache_dir: Option<&'a Path>,
    pub(crate) environment_workspace_cache_dir: Option<&'a Path>,
    pub(crate) platform_cache_home: Option<&'a Path>,
    pub(crate) workspace_root: Option<&'a Path>,
    pub(crate) workspace_identity: Option<&'a str>,
    pub(crate) workspace_opt_in: bool,
}

pub(crate) fn resolve_omena_cache_roots(
    inputs: CacheRootResolverInputsV0<'_>,
) -> OmenaCacheRootsV0 {
    if inputs.initialization_global_storage.is_some()
        || inputs.initialization_workspace_storage.is_some()
    {
        let global = inputs.initialization_global_storage.map(owned_cache_root);
        let workspace_base = inputs
            .initialization_workspace_storage
            .or(inputs.initialization_global_storage);
        let workspace = scoped_workspace_root(
            workspace_base.map(owned_cache_root),
            inputs.workspace_identity,
        );
        return OmenaCacheRootsV0 {
            global,
            workspace,
            source: CacheRootSourceV0::InitializationOptions,
        };
    }

    if inputs.environment_global_cache_dir.is_some()
        || inputs.environment_workspace_cache_dir.is_some()
    {
        let global_base = inputs
            .environment_global_cache_dir
            .or(inputs.environment_workspace_cache_dir);
        let workspace_base = inputs
            .environment_workspace_cache_dir
            .or(inputs.environment_global_cache_dir);
        return OmenaCacheRootsV0 {
            global: global_base.map(owned_cache_root),
            workspace: scoped_workspace_root(
                workspace_base.map(owned_cache_root),
                inputs.workspace_identity,
            ),
            source: CacheRootSourceV0::Environment,
        };
    }

    if let Some(platform_cache_home) = inputs.platform_cache_home {
        let global = owned_cache_root(platform_cache_home);
        return OmenaCacheRootsV0 {
            global: Some(global.clone()),
            workspace: scoped_workspace_root(Some(global), inputs.workspace_identity),
            source: CacheRootSourceV0::Platform,
        };
    }

    if inputs.workspace_opt_in
        && let Some(workspace_root) = inputs.workspace_root
    {
        return OmenaCacheRootsV0 {
            global: None,
            workspace: Some(workspace_root.join(".cache").join("omena")),
            source: CacheRootSourceV0::Workspace,
        };
    }

    OmenaCacheRootsV0 {
        global: None,
        workspace: None,
        source: CacheRootSourceV0::Disabled,
    }
}

pub(crate) fn resolved_workspace_cache_dir(
    config: &LspCacheStorageConfigV0,
    workspace_identity: &str,
    workspace_root: &Path,
    cache_dir_name: &str,
) -> Option<PathBuf> {
    process_cache_roots(config, workspace_identity, workspace_root)
        .workspace
        .map(|root| root.join(cache_dir_name))
}

pub(crate) fn process_cache_roots(
    config: &LspCacheStorageConfigV0,
    workspace_identity: &str,
    workspace_root: &Path,
) -> OmenaCacheRootsV0 {
    if config.location == CacheLocationV0::Workspace {
        return resolve_omena_cache_roots(CacheRootResolverInputsV0 {
            workspace_root: Some(workspace_root),
            workspace_identity: Some(workspace_identity),
            workspace_opt_in: true,
            ..CacheRootResolverInputsV0::default()
        });
    }

    let process_global_cache_dir = std::env::var_os(OMENA_GLOBAL_CACHE_DIR_ENV)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from);
    let process_workspace_cache_dir = std::env::var_os(OMENA_CACHE_DIR_ENV)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from);
    let environment_global_cache_dir = config
        .command_cache_dir
        .as_ref()
        .or(process_global_cache_dir.as_ref())
        .or(process_workspace_cache_dir.as_ref());
    let environment_workspace_cache_dir = config
        .command_cache_dir
        .as_ref()
        .or(process_workspace_cache_dir.as_ref())
        .or(process_global_cache_dir.as_ref());
    let (initialization_global_storage, initialization_workspace_storage) = match config.location {
        CacheLocationV0::Editor => (
            config.initialization_global_storage.as_deref(),
            config.initialization_workspace_storage.as_deref(),
        ),
        CacheLocationV0::Global => (
            config.initialization_global_storage.as_deref(),
            config.initialization_global_storage.as_deref(),
        ),
        CacheLocationV0::Workspace => unreachable!("workspace returned above"),
    };
    let platform_cache_home = platform_cache_home();
    resolve_omena_cache_roots(CacheRootResolverInputsV0 {
        initialization_global_storage,
        initialization_workspace_storage,
        environment_global_cache_dir: environment_global_cache_dir.map(PathBuf::as_path),
        environment_workspace_cache_dir: environment_workspace_cache_dir.map(PathBuf::as_path),
        platform_cache_home: platform_cache_home.as_deref(),
        workspace_root: Some(workspace_root),
        workspace_identity: Some(workspace_identity),
        workspace_opt_in: false,
    })
}

pub(crate) fn platform_cache_home() -> Option<PathBuf> {
    if let Some(path) = std::env::var_os("XDG_CACHE_HOME")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
    {
        return Some(path);
    }
    #[cfg(target_os = "windows")]
    {
        return std::env::var_os("LOCALAPPDATA")
            .filter(|value| !value.is_empty())
            .map(PathBuf::from);
    }
    #[cfg(target_os = "macos")]
    {
        return std::env::var_os("HOME")
            .filter(|value| !value.is_empty())
            .map(PathBuf::from)
            .map(|home| home.join("Library").join("Caches"));
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        return std::env::var_os("HOME")
            .filter(|value| !value.is_empty())
            .map(PathBuf::from)
            .map(|home| home.join(".cache"));
    }
    #[allow(unreachable_code)]
    None
}

fn owned_cache_root(base: &Path) -> PathBuf {
    base.join("omena")
}

fn scoped_workspace_root(
    base: Option<PathBuf>,
    workspace_identity: Option<&str>,
) -> Option<PathBuf> {
    let base = base?;
    let workspace_identity = workspace_identity?;
    let address =
        stable_cache_shard_address(WORKSPACE_CACHE_ADDRESS_PRODUCT, &[workspace_identity])?;
    let hex = address.strip_prefix("blake3:")?;
    Some(base.join("workspaces").join(hex))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cache_root_rung_precedence_is_typed_and_exhaustive() {
        let initialization_global = Path::new("/editor/global");
        let initialization_workspace = Path::new("/editor/workspace");
        let environment_global = Path::new("/env/global");
        let environment_workspace = Path::new("/env/workspace");
        let platform = Path::new("/platform/cache");
        let workspace = Path::new("/workspace");
        let identity = "file:///workspace";
        let rows = [
            (
                CacheRootResolverInputsV0 {
                    initialization_global_storage: Some(initialization_global),
                    initialization_workspace_storage: Some(initialization_workspace),
                    environment_global_cache_dir: Some(environment_global),
                    environment_workspace_cache_dir: Some(environment_workspace),
                    platform_cache_home: Some(platform),
                    workspace_root: Some(workspace),
                    workspace_identity: Some(identity),
                    workspace_opt_in: true,
                },
                CacheRootSourceV0::InitializationOptions,
                "/editor/workspace/omena/workspaces/",
            ),
            (
                CacheRootResolverInputsV0 {
                    environment_global_cache_dir: Some(environment_global),
                    environment_workspace_cache_dir: Some(environment_workspace),
                    platform_cache_home: Some(platform),
                    workspace_root: Some(workspace),
                    workspace_identity: Some(identity),
                    workspace_opt_in: true,
                    ..CacheRootResolverInputsV0::default()
                },
                CacheRootSourceV0::Environment,
                "/env/workspace/omena/workspaces/",
            ),
            (
                CacheRootResolverInputsV0 {
                    platform_cache_home: Some(platform),
                    workspace_root: Some(workspace),
                    workspace_identity: Some(identity),
                    workspace_opt_in: true,
                    ..CacheRootResolverInputsV0::default()
                },
                CacheRootSourceV0::Platform,
                "/platform/cache/omena/workspaces/",
            ),
            (
                CacheRootResolverInputsV0 {
                    workspace_root: Some(workspace),
                    workspace_identity: Some(identity),
                    workspace_opt_in: true,
                    ..CacheRootResolverInputsV0::default()
                },
                CacheRootSourceV0::Workspace,
                "/workspace/.cache/omena",
            ),
            (
                CacheRootResolverInputsV0 {
                    workspace_identity: Some(identity),
                    ..CacheRootResolverInputsV0::default()
                },
                CacheRootSourceV0::Disabled,
                "",
            ),
        ];

        for (inputs, expected_source, expected_prefix) in rows {
            let roots = resolve_omena_cache_roots(inputs);
            assert_eq!(roots.source, expected_source);
            if expected_source == CacheRootSourceV0::Disabled {
                assert!(roots.workspace.is_none());
            } else {
                assert!(
                    roots
                        .workspace
                        .as_ref()
                        .is_some_and(|root| root.to_string_lossy().starts_with(expected_prefix)),
                    "source={expected_source:?} roots={roots:?}"
                );
            }
        }
    }

    #[test]
    fn workspace_default_preserves_all_lsp_cache_paths_byte_for_byte() {
        let workspace = Path::new("/workspace");
        let roots = resolve_omena_cache_roots(CacheRootResolverInputsV0 {
            workspace_root: Some(workspace),
            workspace_identity: Some("file:///workspace"),
            workspace_opt_in: true,
            ..CacheRootResolverInputsV0::default()
        });
        let root = workspace.join(".cache").join("omena");
        assert_eq!(roots.workspace.as_deref(), Some(root.as_path()));
        for cache_dir_name in [
            "diagnostics-cache-v1",
            "source-document-index-v1",
            "workspace-occurrence-shards-v1",
            "source-type-fact-cache-v1",
        ] {
            assert_eq!(
                root.join(cache_dir_name),
                workspace.join(".cache").join("omena").join(cache_dir_name)
            );
        }
    }

    #[test]
    fn environment_rung_moves_every_lsp_cache_under_the_forced_root() {
        let roots = resolve_omena_cache_roots(CacheRootResolverInputsV0 {
            environment_workspace_cache_dir: Some(Path::new("/forced")),
            workspace_identity: Some("file:///workspace"),
            workspace_opt_in: true,
            ..CacheRootResolverInputsV0::default()
        });
        for cache_dir_name in [
            "diagnostics-cache-v1",
            "source-document-index-v1",
            "workspace-occurrence-shards-v1",
            "source-type-fact-cache-v1",
        ] {
            assert!(roots.workspace.as_ref().is_some_and(|root| {
                root.join(cache_dir_name)
                    .starts_with("/forced/omena/workspaces")
            }));
        }
    }

    #[test]
    fn editor_location_partitions_multi_root_storage_by_workspace_identity() {
        let config = LspCacheStorageConfigV0 {
            initialization_global_storage: Some(PathBuf::from("/editor/global")),
            initialization_workspace_storage: Some(PathBuf::from("/editor/workspace")),
            log_path: Some(PathBuf::from("/editor/logs")),
            command_cache_dir: None,
            location: CacheLocationV0::Editor,
        };
        let first = process_cache_roots(&config, "file:///workspace-a", Path::new("/workspace-a"));
        let second = process_cache_roots(&config, "file:///workspace-b", Path::new("/workspace-b"));
        assert_eq!(first.source, CacheRootSourceV0::InitializationOptions);
        assert_eq!(second.source, CacheRootSourceV0::InitializationOptions);
        assert_ne!(first.workspace, second.workspace);
        for root in [first.workspace, second.workspace] {
            assert!(
                root.is_some_and(|root| root.starts_with("/editor/workspace/omena/workspaces"))
            );
        }

        let first_address = stable_cache_shard_address(
            "omena-lsp-server.workspace-occurrence-shard",
            &["file:///workspace-a", "src/App.tsx"],
        );
        let second_address = stable_cache_shard_address(
            "omena-lsp-server.workspace-occurrence-shard",
            &["file:///workspace-b", "src/App.tsx"],
        );
        assert_ne!(
            first_address, second_address,
            "one editor storageUri is intentionally shared, so the workspace identity fold must separate shard addresses"
        );
    }

    #[test]
    fn workspace_location_restores_the_legacy_workspace_root() {
        let config = LspCacheStorageConfigV0 {
            initialization_global_storage: Some(PathBuf::from("/editor/global")),
            initialization_workspace_storage: Some(PathBuf::from("/editor/workspace")),
            location: CacheLocationV0::Workspace,
            ..LspCacheStorageConfigV0::default()
        };
        let roots = process_cache_roots(&config, "file:///workspace", Path::new("/workspace"));
        assert_eq!(roots.source, CacheRootSourceV0::Workspace);
        assert_eq!(
            roots.workspace,
            Some(PathBuf::from("/workspace/.cache/omena"))
        );
    }

    #[test]
    fn editor_storage_without_a_folder_falls_to_global_storage() {
        let config = LspCacheStorageConfigV0 {
            initialization_global_storage: Some(PathBuf::from("/editor/global")),
            initialization_workspace_storage: None,
            log_path: Some(PathBuf::from("/editor/logs")),
            command_cache_dir: None,
            location: CacheLocationV0::Editor,
        };
        let roots = process_cache_roots(&config, "untitled-window", Path::new("/no-workspace"));
        assert_eq!(roots.source, CacheRootSourceV0::InitializationOptions);
        assert!(
            roots
                .workspace
                .is_some_and(|root| { root.starts_with("/editor/global/omena/workspaces") }),
            "an undefined VS Code storageUri must fall to globalStorageUri without failing"
        );
    }

    #[test]
    fn global_location_uses_global_editor_storage_for_workspace_shards() {
        let config = LspCacheStorageConfigV0 {
            initialization_global_storage: Some(PathBuf::from("/editor/global")),
            initialization_workspace_storage: Some(PathBuf::from("/editor/workspace")),
            location: CacheLocationV0::Global,
            ..LspCacheStorageConfigV0::default()
        };
        let roots = process_cache_roots(&config, "file:///workspace", Path::new("/workspace"));
        assert_eq!(roots.source, CacheRootSourceV0::InitializationOptions);
        assert!(
            roots
                .workspace
                .is_some_and(|root| { root.starts_with("/editor/global/omena/workspaces") }),
            "global mode must not reuse the window-scoped workspaceStorageUri"
        );
    }
}
