use std::path::{Path, PathBuf};

pub(crate) fn cli_path_to_file_uri(path: &Path) -> String {
    format!("file://{}", path.to_string_lossy())
}

pub(crate) fn cli_file_uri_to_path(uri: &str) -> Option<PathBuf> {
    uri.strip_prefix("file://").map(PathBuf::from)
}

pub(crate) fn path_string(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

pub(crate) fn style_resolution_workspace_uri_for_path(path: &Path) -> Option<String> {
    path.parent()
        .and_then(discover_style_resolution_workspace_root)
        .map(|workspace_root| format!("file://{}", workspace_root.to_string_lossy()))
}

pub(crate) fn discover_style_resolution_workspace_root(path: &Path) -> Option<&Path> {
    path.ancestors().find(|candidate| {
        [
            "tsconfig.json",
            "tsconfig.base.json",
            "jsconfig.json",
            "package.json",
            "vite.config.ts",
            "vite.config.js",
            "webpack.config.js",
        ]
        .iter()
        .any(|marker| candidate.join(marker).is_file())
    })
}
