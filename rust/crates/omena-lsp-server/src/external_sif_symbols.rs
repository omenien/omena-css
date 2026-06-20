use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
};

use crate::normalize_path;

pub(super) fn external_sif_forward_canonical_url_candidates(
    base_canonical_url: &str,
    source: &str,
) -> Vec<String> {
    let mut candidates = BTreeSet::from([source.to_string()]);
    if !source.starts_with("sass:")
        && !source.starts_with("http://")
        && !source.starts_with("https://")
        && !source.starts_with("file://")
        && !source.starts_with("pkg:")
        && let Some(base_file_path) = base_canonical_url.strip_prefix("file://")
    {
        let base_path = Path::new(base_file_path);
        let joined = if source.starts_with('/') {
            PathBuf::from(source)
        } else {
            base_path
                .parent()
                .unwrap_or_else(|| Path::new(""))
                .join(source)
        };
        push_external_sif_file_uri_candidates(&mut candidates, joined.as_path());
    }
    candidates.into_iter().collect()
}

pub(super) fn external_sif_forward_visibility_allows(
    forward: &omena_sif::OmenaSifForwardExportV1,
    family: &'static str,
    name: &str,
) -> bool {
    let matches_filter = |filter_name: &String| {
        let exposed_name = apply_sass_forward_prefix(forward.prefix.as_deref(), name);
        filter_name == &exposed_name
            || filter_name.trim_start_matches('$') == exposed_name
            || (family != "variable" && filter_name.trim_start_matches('@') == exposed_name)
    };
    if !forward.show.is_empty() {
        return forward.show.iter().any(matches_filter);
    }
    !forward.hide.iter().any(matches_filter)
}

pub(super) fn external_sif_canonical_urls_match(left: &str, right: &str) -> bool {
    if left == right {
        return true;
    }
    let Some(left_path) = external_sif_canonical_url_path(left) else {
        return false;
    };
    let Some(right_path) = external_sif_canonical_url_path(right) else {
        return false;
    };
    normalize_path(Path::new(left_path.as_str()).to_path_buf())
        == normalize_path(Path::new(right_path.as_str()).to_path_buf())
}

pub(super) fn sass_symbol_names_match(left: &str, right: &str) -> bool {
    fold_sass_symbol_name(left.trim_start_matches('$')) == fold_sass_symbol_name(right)
}

fn push_external_sif_file_uri_candidates(candidates: &mut BTreeSet<String>, path: &Path) {
    if path
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some()
    {
        candidates.insert(format!(
            "file://{}",
            normalize_path(path.to_path_buf()).to_string_lossy()
        ));
        return;
    }
    for extension in ["scss", "sass", "css"] {
        let with_extension = path.with_extension(extension);
        candidates.insert(format!(
            "file://{}",
            normalize_path(with_extension).to_string_lossy()
        ));
        if let Some(file_name) = path.file_name().and_then(|file_name| file_name.to_str()) {
            let partial = path
                .with_file_name(format!("_{file_name}"))
                .with_extension(extension);
            candidates.insert(format!(
                "file://{}",
                normalize_path(partial).to_string_lossy()
            ));
        }
    }
}

fn apply_sass_forward_prefix(prefix: Option<&str>, name: &str) -> String {
    match prefix {
        Some(prefix) if prefix.contains('*') => prefix.replace('*', name),
        Some(prefix) => format!("{prefix}{name}"),
        None => name.to_string(),
    }
}

fn external_sif_canonical_url_path(canonical_url: &str) -> Option<String> {
    if let Some(path) = canonical_url.strip_prefix("file://") {
        return Some(path.to_string());
    }
    Path::new(canonical_url)
        .is_absolute()
        .then(|| canonical_url.to_string())
}

fn fold_sass_symbol_name(name: &str) -> String {
    name.replace('_', "-")
}
