mod loader;
mod report;
mod resolution;
mod schema;

pub(crate) use loader::{find_omena_build_config_for_path, find_omena_config_for_path};
use resolution::resolve_config_document;
use schema::OmenaBuildConfig;

use omena_query::OmenaQueryTargetTransformOptionsV0;
use std::path::{Path, PathBuf};

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
