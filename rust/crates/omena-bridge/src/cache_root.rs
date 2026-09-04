use omena_sif::compute_omena_stable_cache_shard_address_v1;
use serde::Serialize;
use std::{
    fs,
    path::{Path, PathBuf},
};

pub(crate) const OMENA_CACHE_DIR_ENV: &str = "OMENA_CACHE_DIR";
pub(crate) const OMENA_GLOBAL_CACHE_DIR_ENV: &str = "OMENA_GLOBAL_CACHE_DIR";
pub(crate) const OMENA_CACHE_GITIGNORE_BYTES: &[u8] =
    b"# machine-generated omena cache - safe to delete\n*\n";
pub(crate) const OMENA_CACHEDIR_TAG_BYTES: &[u8] = b"Signature: 8a477f597d28d172789f06886806bc55\n# This directory is an omena cache; contents are regenerable.\n";
const OMENA_CACHE_ATTRIBUTION_FILE: &str = ".omena-cache-owner.json";

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
        return OmenaCacheRootsV0 {
            global,
            workspace: scoped_workspace_root(
                workspace_base.map(owned_cache_root),
                inputs.workspace_identity,
            ),
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

pub(crate) fn process_external_sif_cache_root(path: &Path) -> Option<OmenaCacheRootsV0> {
    let workspace_root = external_sif_workspace_root(path)?;
    let workspace_identity = workspace_root.to_string_lossy();
    let environment_global_cache_dir = std::env::var_os(OMENA_GLOBAL_CACHE_DIR_ENV)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from);
    let environment_workspace_cache_dir = std::env::var_os(OMENA_CACHE_DIR_ENV)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from);
    let platform_cache_home = platform_cache_home();
    Some(resolve_omena_cache_roots(CacheRootResolverInputsV0 {
        environment_global_cache_dir: environment_global_cache_dir.as_deref(),
        environment_workspace_cache_dir: environment_workspace_cache_dir.as_deref(),
        platform_cache_home: platform_cache_home.as_deref(),
        workspace_root: Some(workspace_root.as_path()),
        workspace_identity: Some(workspace_identity.as_ref()),
        workspace_opt_in: false,
        ..CacheRootResolverInputsV0::default()
    }))
}

fn platform_cache_home() -> Option<PathBuf> {
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

pub(crate) fn external_sif_workspace_root(path: &Path) -> Option<PathBuf> {
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

#[expect(
    clippy::disallowed_methods,
    reason = "cache-root owner: retain standard marker publication"
)]
pub(crate) fn ensure_omena_cache_root_markers(cache_subdir: &Path) {
    let Some(omena_root) = cache_subdir.parent() else {
        return;
    };
    let gitignore = omena_root.join(".gitignore");
    if !gitignore.exists() {
        let _ = fs::write(gitignore, OMENA_CACHE_GITIGNORE_BYTES);
    }
    let cachedir_tag = omena_root.join("CACHEDIR.TAG");
    if !cachedir_tag.exists() {
        let _ = fs::write(cachedir_tag, OMENA_CACHEDIR_TAG_BYTES);
    }
}

#[expect(
    clippy::disallowed_methods,
    reason = "cache-root owner: retain attribution publication"
)]
pub(crate) fn ensure_omena_cache_root_attribution(cache_subdir: &Path, workspace_identity: &str) {
    let Some(cache_root) = cache_subdir.parent() else {
        return;
    };
    let attribution = serde_json::json!({
        "schemaVersion": "0",
        "product": "omena.cache-root-attribution",
        "workspaceIdentity": workspace_identity,
    });
    let Ok(bytes) = serde_json::to_vec(&attribution) else {
        return;
    };
    let _ = fs::write(cache_root.join(OMENA_CACHE_ATTRIBUTION_FILE), bytes);
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
    let address = compute_omena_stable_cache_shard_address_v1(
        "omena-bridge.workspace-cache-root",
        &[workspace_identity],
    )
    .ok()?;
    let hex = address.as_str().strip_prefix("blake3:")?;
    Some(base.join("workspaces").join(hex))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn external_sif_cache_ladder_preserves_workspace_default_and_rung_precedence() {
        let workspace = Path::new("/workspace");
        let identity = "/workspace";
        let workspace_roots = resolve_omena_cache_roots(CacheRootResolverInputsV0 {
            workspace_root: Some(workspace),
            workspace_identity: Some(identity),
            workspace_opt_in: true,
            ..CacheRootResolverInputsV0::default()
        });
        assert_eq!(workspace_roots.source, CacheRootSourceV0::Workspace);
        assert_eq!(
            workspace_roots.workspace,
            Some(workspace.join(".cache").join("omena"))
        );

        let rows = [
            (
                CacheRootResolverInputsV0 {
                    initialization_global_storage: Some(Path::new("/editor/global")),
                    initialization_workspace_storage: Some(Path::new("/editor/workspace")),
                    environment_workspace_cache_dir: Some(Path::new("/forced")),
                    platform_cache_home: Some(Path::new("/platform")),
                    workspace_root: Some(workspace),
                    workspace_identity: Some(identity),
                    workspace_opt_in: true,
                    ..CacheRootResolverInputsV0::default()
                },
                CacheRootSourceV0::InitializationOptions,
                "/editor/workspace/omena/workspaces",
            ),
            (
                CacheRootResolverInputsV0 {
                    environment_workspace_cache_dir: Some(Path::new("/forced")),
                    platform_cache_home: Some(Path::new("/platform")),
                    workspace_root: Some(workspace),
                    workspace_identity: Some(identity),
                    workspace_opt_in: true,
                    ..CacheRootResolverInputsV0::default()
                },
                CacheRootSourceV0::Environment,
                "/forced/omena/workspaces",
            ),
            (
                CacheRootResolverInputsV0 {
                    platform_cache_home: Some(Path::new("/platform")),
                    workspace_root: Some(workspace),
                    workspace_identity: Some(identity),
                    workspace_opt_in: true,
                    ..CacheRootResolverInputsV0::default()
                },
                CacheRootSourceV0::Platform,
                "/platform/omena/workspaces",
            ),
        ];
        let expected_address = compute_omena_stable_cache_shard_address_v1(
            "omena-bridge.workspace-cache-root",
            &[identity],
        )
        .ok()
        .and_then(|address| address.as_str().strip_prefix("blake3:").map(str::to_string));
        assert!(expected_address.is_some());
        for (inputs, expected_source, expected_prefix) in rows {
            let roots = resolve_omena_cache_roots(inputs);
            assert_eq!(roots.source, expected_source);
            assert_eq!(
                roots.workspace,
                expected_address
                    .as_deref()
                    .map(|address| Path::new(expected_prefix).join(address)),
                "source={expected_source:?}: workspace partitions must use the shared stable-address helper"
            );
        }

        let disabled = resolve_omena_cache_roots(CacheRootResolverInputsV0 {
            workspace_identity: Some(identity),
            ..CacheRootResolverInputsV0::default()
        });
        assert_eq!(disabled.source, CacheRootSourceV0::Disabled);
        assert!(disabled.workspace.is_none());
    }

    #[test]
    fn marker_bytes_equal_the_lsp_writer_contract() {
        let lsp_disk_cache_source = include_str!("../../omena-lsp-server/src/disk_cache.rs");
        let lsp_gitignore = rust_byte_const(lsp_disk_cache_source, "OMENA_CACHE_GITIGNORE_BYTES");
        let lsp_cachedir_tag = rust_byte_const(lsp_disk_cache_source, "OMENA_CACHEDIR_TAG_BYTES");
        assert_eq!(
            lsp_gitignore.as_deref(),
            Some(OMENA_CACHE_GITIGNORE_BYTES),
            "the independently authored .gitignore literals must be byte-equal"
        );
        assert_eq!(
            lsp_cachedir_tag.as_deref(),
            Some(OMENA_CACHEDIR_TAG_BYTES),
            "the independently authored CACHEDIR.TAG literals must be byte-equal"
        );
    }

    fn rust_byte_const(source: &str, name: &str) -> Option<Vec<u8>> {
        let declaration = format!("const {name}: &[u8]");
        let declaration_start = source.find(declaration.as_str())?;
        let literal_tail = source.get(declaration_start..)?;
        let literal_start = literal_tail.find("b\"")? + 2;
        let mut bytes = Vec::new();
        let mut escaped = false;
        for byte in literal_tail.get(literal_start..)?.bytes() {
            if escaped {
                bytes.push(match byte {
                    b'n' => b'\n',
                    b'r' => b'\r',
                    b't' => b'\t',
                    b'\\' => b'\\',
                    b'"' => b'"',
                    _ => return None,
                });
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == b'"' {
                return Some(bytes);
            } else {
                bytes.push(byte);
            }
        }
        None
    }

    #[test]
    fn external_sif_process_root_is_global_and_workspace_partitioned() {
        let workspace = std::env::temp_dir().join(format!(
            "omena-bridge-global-cache-root-{}",
            std::process::id()
        ));
        let style = workspace.join("tokens.scss");
        let roots = process_external_sif_cache_root(style.as_path()).unwrap_or(OmenaCacheRootsV0 {
            global: None,
            workspace: None,
            source: CacheRootSourceV0::Disabled,
        });
        assert!(
            matches!(
                roots.source,
                CacheRootSourceV0::Environment | CacheRootSourceV0::Platform
            ),
            "the direct bridge path must use an environment or platform global root: {roots:?}"
        );
        assert!(
            roots.workspace.is_some_and(|root| {
                root.components()
                    .any(|component| component.as_os_str() == "workspaces")
                    && !root.starts_with(workspace.join(".cache"))
            }),
            "the global physical store must retain a workspace partition"
        );
    }
}
