use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    sync::{Arc, Mutex, OnceLock},
};

use super::{
    report::OmenaConfigReport,
    resolve_config_document,
    schema::{OmenaBuildConfig, OmenaConfig},
};

const CONFIG_CANDIDATES: &[&str] = &["omena.toml", "omena.config.toml", "omena.config.json"];
const PROJECT_ROOT_MARKERS: &[&str] = &[".git", "pnpm-workspace.yaml"];

#[derive(Debug, Clone)]
pub(crate) struct LoadedOmenaConfig {
    pub(crate) directory: PathBuf,
    pub(crate) config: Arc<OmenaConfig>,
    pub(crate) reports: Arc<[OmenaConfigReport]>,
    pub(crate) config_content_digest: Arc<str>,
}

#[derive(Debug, Clone)]
pub(crate) struct LoadedOmenaBuildConfig {
    pub(crate) directory: PathBuf,
    pub(crate) build: OmenaBuildConfig,
    pub(crate) reports: Arc<[OmenaConfigReport]>,
    pub(crate) config_content_digest: Arc<str>,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct ConfigCacheKey {
    config_path: PathBuf,
    target_path: PathBuf,
}

#[derive(Default)]
struct OmenaConfigLoader {
    cache: HashMap<ConfigCacheKey, Arc<LoadedOmenaConfig>>,
    parse_count: usize,
}

struct DiscoveredConfig {
    selected: PathBuf,
    shadowed: Vec<PathBuf>,
}

static CONFIG_LOADER: OnceLock<Mutex<OmenaConfigLoader>> = OnceLock::new();

pub(crate) fn find_omena_config_for_path(
    target_path: &Path,
) -> Result<Option<Arc<LoadedOmenaConfig>>, String> {
    let mut loader = CONFIG_LOADER
        .get_or_init(|| Mutex::new(OmenaConfigLoader::default()))
        .lock()
        .map_err(|_| "Omena config loader lock was poisoned".to_string())?;
    loader.find_for_path(target_path)
}

pub(crate) fn find_omena_build_config_for_path(
    style_path: &Path,
) -> Result<Option<LoadedOmenaBuildConfig>, String> {
    Ok(
        find_omena_config_for_path(style_path)?.map(|loaded| LoadedOmenaBuildConfig {
            directory: loaded.directory.clone(),
            build: loaded.config.build.clone(),
            reports: Arc::clone(&loaded.reports),
            config_content_digest: Arc::clone(&loaded.config_content_digest),
        }),
    )
}

impl OmenaConfigLoader {
    fn find_for_path(
        &mut self,
        target_path: &Path,
    ) -> Result<Option<Arc<LoadedOmenaConfig>>, String> {
        let absolute_target = absolute_path(target_path)?;
        let Some(discovered) = discover_config(&absolute_target)? else {
            return Ok(None);
        };
        let selected = fs::canonicalize(&discovered.selected).map_err(|error| {
            format!(
                "failed to resolve Omena config {}: {error}",
                discovered.selected.display()
            )
        })?;
        let key = ConfigCacheKey {
            config_path: selected.clone(),
            target_path: absolute_target.clone(),
        };
        if let Some(loaded) = self.cache.get(&key) {
            return Ok(Some(Arc::clone(loaded)));
        }

        let resolved = resolve_config_document(&selected, &absolute_target)?;
        self.parse_count += 1;
        let mut reports = resolved.reports;
        reports.extend(discovered.shadowed.into_iter().map(|path| {
            OmenaConfigReport::shadowed(path.display().to_string(), selected.display().to_string())
        }));
        reports.sort_by(|left, right| {
            (left.kind.as_str(), left.path.as_str())
                .cmp(&(right.kind.as_str(), right.path.as_str()))
        });
        let loaded = Arc::new(LoadedOmenaConfig {
            directory: selected
                .parent()
                .unwrap_or_else(|| Path::new("."))
                .to_path_buf(),
            config: Arc::new(resolved.config),
            reports: reports.into(),
            config_content_digest: resolved.config_content_digest.into(),
        });
        self.cache.insert(key, Arc::clone(&loaded));
        Ok(Some(loaded))
    }
}

fn discover_config(target_path: &Path) -> Result<Option<DiscoveredConfig>, String> {
    let mut current = if target_path.is_dir() {
        target_path.to_path_buf()
    } else {
        target_path
            .parent()
            .ok_or_else(|| format!("target path {} has no parent", target_path.display()))?
            .to_path_buf()
    };

    loop {
        let candidates = CONFIG_CANDIDATES
            .iter()
            .map(|file_name| current.join(file_name))
            .filter(|path| path.is_file())
            .collect::<Vec<_>>();
        if let Some(selected) = candidates.first() {
            return Ok(Some(DiscoveredConfig {
                selected: selected.clone(),
                shadowed: candidates.into_iter().skip(1).collect(),
            }));
        }
        let at_project_root = PROJECT_ROOT_MARKERS
            .iter()
            .any(|marker| current.join(marker).exists());
        if at_project_root || !current.pop() {
            return Ok(None);
        }
    }
}

fn absolute_path(path: &Path) -> Result<PathBuf, String> {
    if path.is_absolute() {
        Ok(path.to_path_buf())
    } else {
        std::env::current_dir()
            .map(|directory| directory.join(path))
            .map_err(|error| format!("failed to read current directory: {error}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn canonical_config_wins_and_shadowed_legacy_files_are_reported() -> Result<(), String> {
        let root = temp_dir("candidate-priority");
        fs::create_dir_all(&root).map_err(|error| error.to_string())?;
        let target = root.join("a.css");
        fs::write(&target, ".a {}\n").map_err(|error| error.to_string())?;
        fs::write(root.join("omena.toml"), "[build]\nminify = true\n")
            .map_err(|error| error.to_string())?;
        fs::write(root.join("omena.config.toml"), "[build]\nminify = false\n")
            .map_err(|error| error.to_string())?;

        let mut loader = OmenaConfigLoader::default();
        let loaded = loader
            .find_for_path(&target)?
            .ok_or_else(|| "expected config".to_string())?;
        assert_eq!(loaded.config.build.minify, Some(true));
        assert!(loaded.reports.iter().any(|report| {
            report.kind.as_str() == "shadowedConfig" && report.path.ends_with("omena.config.toml")
        }));
        cleanup(&root);
        Ok(())
    }

    #[test]
    fn resolved_path_is_parsed_once_and_shared_by_identity() -> Result<(), String> {
        let root = temp_dir("memoized");
        fs::create_dir_all(&root).map_err(|error| error.to_string())?;
        let target = root.join("a.css");
        fs::write(&target, ".a {}\n").map_err(|error| error.to_string())?;
        fs::write(
            root.join("omena.toml"),
            "[lint]\nprofile = \"recommended\"\n",
        )
        .map_err(|error| error.to_string())?;

        let mut loader = OmenaConfigLoader::default();
        let first = loader
            .find_for_path(&target)?
            .ok_or_else(|| "expected first config".to_string())?;
        let second = loader
            .find_for_path(&target)?
            .ok_or_else(|| "expected second config".to_string())?;
        assert_eq!(loader.parse_count, 1);
        assert!(Arc::ptr_eq(&first, &second));
        assert!(Arc::ptr_eq(&first.config, &second.config));
        assert_eq!(first.config_content_digest, second.config_content_digest);
        cleanup(&root);
        Ok(())
    }

    #[test]
    fn legacy_flat_json_and_build_table_keep_the_build_contract() -> Result<(), String> {
        let root = temp_dir("legacy-build");
        fs::create_dir_all(&root).map_err(|error| error.to_string())?;
        let target = root.join("a.css");
        fs::write(&target, ".a {}\n").map_err(|error| error.to_string())?;
        fs::write(
            root.join("omena.config.json"),
            r#"{"minify":true,"source_map":true,"output":"dist.css"}"#,
        )
        .map_err(|error| error.to_string())?;

        let mut loader = OmenaConfigLoader::default();
        let loaded = loader
            .find_for_path(&target)?
            .ok_or_else(|| "expected legacy config".to_string())?;
        assert_eq!(
            serde_json::to_value(&loaded.config.build).map_err(|error| error.to_string())?,
            serde_json::json!({
                "passes": null,
                "minify": true,
                "postcssCompat": null,
                "targetQuery": null,
                "closedStyleWorld": null,
                "treeShake": null,
                "bundle": null,
                "sourceMap": true,
                "output": "dist.css",
                "sources": null,
                "packageManifests": null,
                "bundleEntries": null,
                "splitOutDir": null,
                "contextJson": null,
                "engineInputJson": null,
                "inputSourceMaps": null,
                "allowLogicalToPhysical": null,
                "allowScopeFlatten": null,
                "allowLayerFlatten": null,
                "enableSupportsStaticEval": null,
                "enableMediaStaticEval": null,
                "enableContainerStaticEval": null,
                "dropDarkModeMediaQueries": null
            })
        );
        cleanup(&root);
        Ok(())
    }

    #[test]
    fn project_root_marker_prevents_parent_config_leakage() -> Result<(), String> {
        let root = temp_dir("project-root");
        let workspace = root.join("workspace");
        let source_dir = workspace.join("packages/app/src");
        fs::create_dir_all(&source_dir).map_err(|error| error.to_string())?;
        let target = source_dir.join("a.css");
        fs::write(&target, ".a {}\n").map_err(|error| error.to_string())?;
        fs::write(root.join("omena.toml"), "[build]\nminify = true\n")
            .map_err(|error| error.to_string())?;
        fs::write(
            workspace.join("pnpm-workspace.yaml"),
            "packages:\n  - packages/*\n",
        )
        .map_err(|error| error.to_string())?;

        let mut loader = OmenaConfigLoader::default();
        assert!(loader.find_for_path(&target)?.is_none());

        fs::write(workspace.join("omena.toml"), "[build]\nminify = false\n")
            .map_err(|error| error.to_string())?;
        let loaded = loader
            .find_for_path(&target)?
            .ok_or_else(|| "expected workspace config".to_string())?;
        assert_eq!(loaded.config.build.minify, Some(false));
        cleanup(&root);
        Ok(())
    }

    fn temp_dir(label: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |duration| duration.as_nanos());
        std::env::temp_dir().join(format!("omena-config-loader-{label}-{nonce}"))
    }

    fn cleanup(path: &Path) {
        let _ = fs::remove_dir_all(path);
    }
}
