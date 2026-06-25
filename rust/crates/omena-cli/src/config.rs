use omena_query::OmenaQueryTargetTransformOptionsV0;
use serde::Deserialize;
use std::{
    fs,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone)]
pub(crate) struct LoadedOmenaBuildConfig {
    pub(crate) directory: PathBuf,
    pub(crate) build: OmenaBuildConfig,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub(crate) struct OmenaBuildConfig {
    #[serde(alias = "pass")]
    pub(crate) passes: Option<Vec<String>>,
    pub(crate) minify: Option<bool>,
    #[serde(alias = "target_query", alias = "target-query")]
    pub(crate) target_query: Option<String>,
    #[serde(alias = "closed_style_world", alias = "closed-style-world")]
    pub(crate) closed_style_world: Option<bool>,
    #[serde(alias = "tree_shake", alias = "tree-shake")]
    pub(crate) tree_shake: Option<bool>,
    pub(crate) bundle: Option<bool>,
    #[serde(alias = "source_map", alias = "source-map")]
    pub(crate) source_map: Option<bool>,
    pub(crate) output: Option<PathBuf>,
    #[serde(alias = "source", alias = "source_paths", alias = "source-paths")]
    pub(crate) sources: Option<Vec<PathBuf>>,
    #[serde(
        alias = "package_manifest",
        alias = "package_manifests",
        alias = "package-manifest",
        alias = "package-manifests"
    )]
    pub(crate) package_manifests: Option<Vec<PathBuf>>,
    #[serde(
        alias = "bundle_entry",
        alias = "bundle_entries",
        alias = "bundle-entry",
        alias = "bundle-entries"
    )]
    pub(crate) bundle_entries: Option<Vec<PathBuf>>,
    #[serde(alias = "split_out_dir", alias = "split-out-dir")]
    pub(crate) split_out_dir: Option<PathBuf>,
    #[serde(alias = "context_json", alias = "context-json")]
    pub(crate) context_json: Option<PathBuf>,
    #[serde(alias = "engine_input_json", alias = "engine-input-json")]
    pub(crate) engine_input_json: Option<PathBuf>,
    #[serde(alias = "input_source_map", alias = "input-source-map")]
    pub(crate) input_source_maps: Option<Vec<String>>,
    #[serde(
        alias = "allow_logical_to_physical",
        alias = "allow-logical-to-physical"
    )]
    pub(crate) allow_logical_to_physical: Option<bool>,
    #[serde(alias = "allow_scope_flatten", alias = "allow-scope-flatten")]
    pub(crate) allow_scope_flatten: Option<bool>,
    #[serde(alias = "allow_layer_flatten", alias = "allow-layer-flatten")]
    pub(crate) allow_layer_flatten: Option<bool>,
    #[serde(
        alias = "enable_supports_static_eval",
        alias = "enable-supports-static-eval"
    )]
    pub(crate) enable_supports_static_eval: Option<bool>,
    #[serde(alias = "enable_media_static_eval", alias = "enable-media-static-eval")]
    pub(crate) enable_media_static_eval: Option<bool>,
    #[serde(
        alias = "enable_container_static_eval",
        alias = "enable-container-static-eval"
    )]
    pub(crate) enable_container_static_eval: Option<bool>,
    #[serde(
        alias = "drop_dark_mode_media_queries",
        alias = "drop-dark-mode-media-queries"
    )]
    pub(crate) drop_dark_mode_media_queries: Option<bool>,
}

pub(crate) fn find_omena_build_config_for_path(
    style_path: &Path,
) -> Result<Option<LoadedOmenaBuildConfig>, String> {
    let absolute_style_path = if style_path.is_absolute() {
        style_path.to_path_buf()
    } else {
        std::env::current_dir()
            .map_err(|error| format!("failed to read current directory: {error}"))?
            .join(style_path)
    };
    let mut current = absolute_style_path
        .parent()
        .ok_or_else(|| format!("style path {} has no parent", style_path.display()))?
        .to_path_buf();

    loop {
        for file_name in ["omena.config.toml", "omena.config.json"] {
            let candidate = current.join(file_name);
            if candidate.exists() {
                let build = read_omena_build_config(&candidate)?;
                return Ok(Some(LoadedOmenaBuildConfig {
                    directory: current,
                    build,
                }));
            }
        }
        if !current.pop() {
            return Ok(None);
        }
    }
}

pub(crate) fn apply_configured_target_options(
    target_options: &mut OmenaQueryTargetTransformOptionsV0,
    config: &OmenaBuildConfig,
) {
    if !target_options.allow_logical_to_physical {
        target_options.allow_logical_to_physical =
            config.allow_logical_to_physical.unwrap_or(false);
    }
    if !target_options.allow_scope_flatten {
        target_options.allow_scope_flatten = config.allow_scope_flatten.unwrap_or(false);
    }
    if !target_options.allow_layer_flatten {
        target_options.allow_layer_flatten = config.allow_layer_flatten.unwrap_or(false);
    }
    if !target_options.enable_supports_static_eval {
        target_options.enable_supports_static_eval =
            config.enable_supports_static_eval.unwrap_or(false);
    }
    if !target_options.enable_media_static_eval {
        target_options.enable_media_static_eval = config.enable_media_static_eval.unwrap_or(false);
    }
    if !target_options.enable_container_static_eval {
        target_options.enable_container_static_eval =
            config.enable_container_static_eval.unwrap_or(false);
    }
    if !target_options.drop_dark_mode_media_queries {
        target_options.drop_dark_mode_media_queries =
            config.drop_dark_mode_media_queries.unwrap_or(false);
    }
}

pub(crate) fn resolve_config_path(config_dir: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        config_dir.join(path)
    }
}

pub(crate) fn resolve_config_paths(config_dir: &Path, paths: &[PathBuf]) -> Vec<PathBuf> {
    paths
        .iter()
        .map(|path| resolve_config_path(config_dir, path))
        .collect()
}

fn read_omena_build_config(config_path: &Path) -> Result<OmenaBuildConfig, String> {
    let source = fs::read_to_string(config_path).map_err(|error| {
        format!(
            "failed to read Omena config {}: {error}",
            config_path.display()
        )
    })?;
    match config_path
        .extension()
        .and_then(|extension| extension.to_str())
    {
        Some("toml") => read_toml_build_config(config_path, &source),
        Some("json") => read_json_build_config(config_path, &source),
        _ => Err(format!(
            "unsupported Omena config extension for {}",
            config_path.display()
        )),
    }
}

fn read_toml_build_config(config_path: &Path, source: &str) -> Result<OmenaBuildConfig, String> {
    let value = toml::from_str::<toml::Value>(source).map_err(|error| {
        format!(
            "failed to parse Omena TOML config {}: {error}",
            config_path.display()
        )
    })?;
    let build_value = value.get("build").cloned().unwrap_or(value);
    build_value.try_into().map_err(|error| {
        format!(
            "failed to decode Omena build config {}: {error}",
            config_path.display()
        )
    })
}

fn read_json_build_config(config_path: &Path, source: &str) -> Result<OmenaBuildConfig, String> {
    let value = serde_json::from_str::<serde_json::Value>(source).map_err(|error| {
        format!(
            "failed to parse Omena JSON config {}: {error}",
            config_path.display()
        )
    })?;
    let build_value = value.get("build").cloned().unwrap_or(value);
    serde_json::from_value(build_value).map_err(|error| {
        format!(
            "failed to decode Omena build config {}: {error}",
            config_path.display()
        )
    })
}
