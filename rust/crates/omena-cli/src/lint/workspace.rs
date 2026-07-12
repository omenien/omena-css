use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::paths::path_string;

pub(crate) struct WorkspaceFiles {
    pub(crate) style_paths: Vec<PathBuf>,
    pub(crate) source_paths: Vec<PathBuf>,
    pub(crate) package_manifest_paths: Vec<PathBuf>,
}

pub(crate) fn discover_workspace_files(root: &Path) -> Result<WorkspaceFiles, String> {
    let mut files = WorkspaceFiles {
        style_paths: Vec::new(),
        source_paths: Vec::new(),
        package_manifest_paths: Vec::new(),
    };
    if root.is_file() {
        classify_file(root.to_path_buf(), &mut files);
    } else {
        visit_directory(root, &mut files)?;
    }
    files.style_paths.sort();
    files.source_paths.sort();
    files.package_manifest_paths.sort();
    Ok(files)
}

fn visit_directory(directory: &Path, files: &mut WorkspaceFiles) -> Result<(), String> {
    let entries = fs::read_dir(directory)
        .map_err(|error| format!("failed to read {}: {error}", path_string(directory)))?;
    for entry in entries {
        let entry = entry.map_err(|error| {
            format!(
                "failed to read an entry under {}: {error}",
                path_string(directory)
            )
        })?;
        let path = entry.path();
        let file_type = entry.file_type().map_err(|error| {
            format!("failed to inspect {}: {error}", path_string(path.as_path()))
        })?;
        if file_type.is_symlink() {
            continue;
        }
        if file_type.is_dir() {
            if !ignored_directory(entry.file_name().to_string_lossy().as_ref()) {
                visit_directory(path.as_path(), files)?;
            }
        } else if file_type.is_file() {
            classify_file(path, files);
        }
    }
    Ok(())
}

fn ignored_directory(name: &str) -> bool {
    matches!(
        name,
        ".git"
            | ".hg"
            | ".svn"
            | "node_modules"
            | "target"
            | "dist"
            | "build"
            | "coverage"
            | ".next"
            | ".turbo"
    )
}

fn classify_file(path: PathBuf, files: &mut WorkspaceFiles) {
    let Some(file_name) = path.file_name().and_then(|value| value.to_str()) else {
        return;
    };
    if file_name == "package.json" {
        files.package_manifest_paths.push(path);
        return;
    }
    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default();
    if matches!(extension, "css" | "scss" | "sass" | "less") {
        files.style_paths.push(path);
    } else if matches!(
        extension,
        "ts" | "tsx"
            | "mts"
            | "cts"
            | "js"
            | "jsx"
            | "mjs"
            | "cjs"
            | "vue"
            | "svelte"
            | "astro"
            | "html"
    ) {
        files.source_paths.push(path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT_FIXTURE_ID: AtomicU64 = AtomicU64::new(0);

    #[test]
    fn discovery_is_sorted_and_skips_generated_directories() -> Result<(), String> {
        let root = fixture_root();
        fs::create_dir_all(root.join("src")).map_err(|error| error.to_string())?;
        fs::create_dir_all(root.join("node_modules/pkg")).map_err(|error| error.to_string())?;
        fs::write(root.join("src/a.module.scss"), ".a {}\n").map_err(|error| error.to_string())?;
        fs::write(root.join("src/a.tsx"), "export {};\n").map_err(|error| error.to_string())?;
        fs::write(root.join("package.json"), "{}\n").map_err(|error| error.to_string())?;
        fs::write(root.join("node_modules/pkg/ignored.css"), ".ignored {}\n")
            .map_err(|error| error.to_string())?;

        let files = discover_workspace_files(root.as_path())?;
        assert_eq!(files.style_paths.len(), 1);
        assert_eq!(files.source_paths.len(), 1);
        assert_eq!(files.package_manifest_paths.len(), 1);
        fs::remove_dir_all(root).map_err(|error| error.to_string())?;
        Ok(())
    }

    fn fixture_root() -> PathBuf {
        let id = NEXT_FIXTURE_ID.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!("omena-lint-workspace-{}-{id}", std::process::id()))
    }
}
