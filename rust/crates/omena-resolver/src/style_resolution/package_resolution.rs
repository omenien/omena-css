use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::types::OmenaResolverStylePackageManifestV0;

use super::{is_external_style_module_source, normalize_style_path, push_unique_pathbuf};

pub(super) enum PackageStyleCandidateResolution {
    Candidates(Vec<PathBuf>),
    Blocked,
}

enum PackageJsonEntryResolution {
    Resolved(String),
    Blocked,
}

impl PackageJsonEntryResolution {
    fn map_resolved(self, map: impl FnOnce(String) -> String) -> Self {
        match self {
            Self::Resolved(entry) => Self::Resolved(map(entry)),
            Self::Blocked => Self::Blocked,
        }
    }
}

pub(super) fn package_style_module_base_candidates(
    from_style_path: &str,
    source: &str,
) -> Vec<PathBuf> {
    let Some(package_source) = parse_package_style_source(source) else {
        return Vec::new();
    };
    let Some(from_dir) = Path::new(from_style_path).parent() else {
        return Vec::new();
    };
    let mut candidates = Vec::new();
    let mut current_dir = Some(from_dir);
    while let Some(dir) = current_dir {
        let package_root = dir.join("node_modules").join(package_source.package_name);
        let package_entry = match package_source.subpath {
            Some(subpath) => package_root.join(subpath),
            None => package_root.clone(),
        };
        push_unique_pathbuf(&mut candidates, package_entry.clone());
        if let Some(subpath) = package_source.subpath {
            push_unique_pathbuf(&mut candidates, package_root.join("src").join(subpath));
        } else {
            push_unique_pathbuf(&mut candidates, package_root.join("index"));
            push_unique_pathbuf(&mut candidates, package_root.join("src").join("index"));
        }
        current_dir = dir.parent();
    }
    candidates
}

pub(super) fn package_manifest_style_module_base_candidates(
    from_style_path: &str,
    source: &str,
    package_manifests: &[OmenaResolverStylePackageManifestV0],
) -> PackageStyleCandidateResolution {
    let Some(package_source) = parse_package_style_source(source) else {
        return PackageStyleCandidateResolution::Candidates(Vec::new());
    };
    let Some(from_dir) = Path::new(from_style_path).parent() else {
        return PackageStyleCandidateResolution::Candidates(Vec::new());
    };
    let manifest_by_package_dir = package_manifests
        .iter()
        .map(|manifest| {
            (
                package_dir_from_package_json_path(&manifest.package_json_path),
                manifest.package_json_source.as_str(),
            )
        })
        .collect::<BTreeMap<_, _>>();

    let mut candidates = Vec::new();
    let mut current_dir = Some(from_dir);
    while let Some(dir) = current_dir {
        let package_root = dir.join("node_modules").join(package_source.package_name);
        let package_root_key = normalize_style_path(package_root.clone());
        if let Some(package_json_source) = manifest_by_package_dir.get(&package_root_key)
            && let Some(entry) = read_package_manifest_style_entry(
                package_json_source,
                package_source.subpath,
                package_source.uses_sass_node_package_importer,
            )
        {
            match entry {
                PackageJsonEntryResolution::Resolved(entry) => {
                    push_unique_pathbuf(&mut candidates, package_root.join(entry));
                }
                PackageJsonEntryResolution::Blocked => {
                    return PackageStyleCandidateResolution::Blocked;
                }
            }
        }
        current_dir = dir.parent();
    }
    PackageStyleCandidateResolution::Candidates(candidates)
}

pub(super) fn package_import_style_module_base_candidates(
    from_style_path: &str,
    source: &str,
    package_manifests: &[OmenaResolverStylePackageManifestV0],
) -> PackageStyleCandidateResolution {
    let Some(from_dir) = Path::new(from_style_path).parent() else {
        return PackageStyleCandidateResolution::Candidates(Vec::new());
    };
    let manifest_by_package_dir = package_manifests
        .iter()
        .map(|manifest| {
            (
                package_dir_from_package_json_path(&manifest.package_json_path),
                manifest.package_json_source.as_str(),
            )
        })
        .collect::<BTreeMap<_, _>>();

    let mut candidates = Vec::new();
    let mut current_dir = Some(from_dir);
    while let Some(dir) = current_dir {
        let package_dir_key = normalize_style_path(dir.to_path_buf());
        if let Some(package_json_source) = manifest_by_package_dir.get(&package_dir_key)
            && let Some(entry) = read_package_import_entry(package_json_source, source)
        {
            match entry {
                PackageJsonEntryResolution::Resolved(entry) => {
                    if push_package_import_entry_candidates(
                        &mut candidates,
                        dir,
                        from_style_path,
                        &entry,
                        package_manifests,
                    ) {
                        return PackageStyleCandidateResolution::Blocked;
                    }
                }
                PackageJsonEntryResolution::Blocked => {
                    return PackageStyleCandidateResolution::Blocked;
                }
            }
            break;
        }
        current_dir = dir.parent();
    }
    PackageStyleCandidateResolution::Candidates(candidates)
}

fn push_package_import_entry_candidates(
    candidates: &mut Vec<PathBuf>,
    package_dir: &Path,
    from_style_path: &str,
    entry: &str,
    package_manifests: &[OmenaResolverStylePackageManifestV0],
) -> bool {
    if entry.starts_with("./") {
        push_unique_pathbuf(
            candidates,
            package_dir.join(normalize_package_json_entry(entry)),
        );
        return false;
    }
    if entry.starts_with('#') || is_external_style_module_source(entry) {
        return false;
    }
    match package_manifest_style_module_base_candidates(from_style_path, entry, package_manifests) {
        PackageStyleCandidateResolution::Candidates(base_paths) => {
            for package_manifest_base_path in base_paths {
                push_unique_pathbuf(candidates, package_manifest_base_path);
            }
        }
        PackageStyleCandidateResolution::Blocked => {
            return true;
        }
    }
    for package_base_path in package_style_module_base_candidates(from_style_path, entry) {
        push_unique_pathbuf(candidates, package_base_path);
    }
    false
}

fn package_dir_from_package_json_path(package_json_path: &str) -> String {
    Path::new(package_json_path)
        .parent()
        .map(|path| normalize_style_path(path.to_path_buf()))
        .unwrap_or_default()
}

fn read_package_manifest_style_entry(
    package_json_source: &str,
    subpath: Option<&str>,
    uses_sass_node_package_importer: bool,
) -> Option<PackageJsonEntryResolution> {
    let package_json = serde_json::from_str::<serde_json::Value>(package_json_source).ok()?;
    let package_object = package_json.as_object()?;
    if uses_sass_node_package_importer {
        return read_sass_node_package_importer_style_entry(package_object, subpath)
            .map(|entry| entry.map_resolved(|entry| normalize_package_json_entry(&entry)));
    }
    if let Some(exports_value) = package_object.get("exports") {
        let entry = if let Some(subpath) = subpath {
            read_package_export_subpath_entry(Some(exports_value), subpath)
        } else {
            read_package_export_entry(Some(exports_value))
        };
        return Some(entry.unwrap_or(PackageJsonEntryResolution::Blocked));
    }
    if subpath.is_some() {
        return None;
    }
    let entry = read_package_json_string_field(package_object, "sass")
        .or_else(|| read_package_json_string_field(package_object, "scss"))
        .or_else(|| read_package_json_string_field(package_object, "style"))?;
    Some(PackageJsonEntryResolution::Resolved(
        normalize_package_json_entry(&entry),
    ))
}

fn read_sass_node_package_importer_style_entry(
    package_object: &serde_json::Map<String, serde_json::Value>,
    subpath: Option<&str>,
) -> Option<PackageJsonEntryResolution> {
    if let Some(exports_value) = package_object.get("exports") {
        let export_entry = if let Some(subpath) = subpath {
            read_sass_node_package_export_subpath_entry(Some(exports_value), subpath)
        } else {
            read_sass_node_package_export_entry(Some(exports_value))
        };
        return Some(export_entry.unwrap_or(PackageJsonEntryResolution::Blocked));
    }
    let entry = {
        if subpath.is_some() {
            return None;
        }
        read_package_json_string_field(package_object, "sass")
            .or_else(|| read_package_json_string_field(package_object, "style"))
    }?;
    Some(PackageJsonEntryResolution::Resolved(entry))
}

fn read_package_import_entry(
    package_json_source: &str,
    specifier: &str,
) -> Option<PackageJsonEntryResolution> {
    let package_json = serde_json::from_str::<serde_json::Value>(package_json_source).ok()?;
    let package_object = package_json.as_object()?;
    let imports_object = package_object.get("imports")?.as_object()?;
    if let Some(entry) = read_package_export_entry(imports_object.get(specifier)) {
        return Some(entry);
    }
    for (key, import_value) in sorted_package_pattern_entries(imports_object) {
        let Some(pattern_match) = match_package_import_pattern(key, specifier) else {
            continue;
        };
        let Some(entry) = read_package_export_entry(Some(import_value)) else {
            continue;
        };
        return Some(
            entry.map_resolved(|entry| substitute_package_export_pattern(&entry, &pattern_match)),
        );
    }
    None
}

fn sorted_package_pattern_entries(
    object: &serde_json::Map<String, serde_json::Value>,
) -> Vec<(&str, &serde_json::Value)> {
    let mut entries = object
        .iter()
        .enumerate()
        .filter_map(|(index, (key, value))| {
            package_pattern_priority(key).map(|priority| (index, priority, key.as_str(), value))
        })
        .collect::<Vec<_>>();
    entries.sort_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(&right.0)));
    entries
        .into_iter()
        .map(|(_, _, key, value)| (key, value))
        .collect()
}

fn package_pattern_priority(pattern_key: &str) -> Option<PackagePatternPriority> {
    let star_index = pattern_key.find('*')?;
    if pattern_key[star_index + 1..].contains('*') {
        return None;
    }
    Some(PackagePatternPriority {
        base_len: star_index,
        key_len: pattern_key.len(),
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct PackagePatternPriority {
    base_len: usize,
    key_len: usize,
}

fn match_package_import_pattern(pattern_key: &str, specifier: &str) -> Option<String> {
    let (prefix, suffix) = pattern_key.split_once('*')?;
    if suffix.contains('*') || !specifier.starts_with(prefix) || !specifier.ends_with(suffix) {
        return None;
    }
    Some(specifier[prefix.len()..specifier.len() - suffix.len()].to_string())
}

fn read_package_export_subpath_entry(
    exports_value: Option<&serde_json::Value>,
    subpath: &str,
) -> Option<PackageJsonEntryResolution> {
    let exports_object = exports_value?.as_object()?;
    for key in package_export_subpath_keys(subpath) {
        if let Some(entry) = read_package_export_entry(exports_object.get(&key)) {
            return Some(entry);
        }
    }
    for (key, export_value) in sorted_package_pattern_entries(exports_object) {
        let Some(pattern_match) = match_package_export_subpath_pattern(key, subpath) else {
            continue;
        };
        let Some(entry) = read_package_export_entry(Some(export_value)) else {
            continue;
        };
        return Some(
            entry.map_resolved(|entry| substitute_package_export_pattern(&entry, &pattern_match)),
        );
    }
    None
}

fn read_sass_node_package_export_subpath_entry(
    exports_value: Option<&serde_json::Value>,
    subpath: &str,
) -> Option<PackageJsonEntryResolution> {
    let exports_object = exports_value?.as_object()?;
    for key in package_export_subpath_keys(subpath) {
        if let Some(entry) = read_sass_node_package_export_entry(exports_object.get(&key)) {
            return Some(entry);
        }
    }
    for (key, export_value) in sorted_package_pattern_entries(exports_object) {
        let Some(pattern_match) = match_package_export_subpath_pattern(key, subpath) else {
            continue;
        };
        let Some(entry) = read_sass_node_package_export_entry(Some(export_value)) else {
            continue;
        };
        return Some(
            entry.map_resolved(|entry| substitute_package_export_pattern(&entry, &pattern_match)),
        );
    }
    None
}

fn package_export_subpath_keys(subpath: &str) -> Vec<String> {
    let normalized = subpath
        .trim_start_matches("./")
        .trim_start_matches('/')
        .to_string();
    vec![
        format!("./{normalized}"),
        format!("./{normalized}.scss"),
        format!("./{normalized}.sass"),
        format!("./{normalized}.css"),
    ]
}

fn match_package_export_subpath_pattern(pattern_key: &str, subpath: &str) -> Option<String> {
    let normalized_pattern = pattern_key.trim_start_matches("./").trim_start_matches('/');
    let (prefix, suffix) = normalized_pattern.split_once('*')?;
    if suffix.contains('*') {
        return None;
    }

    for candidate_key in package_export_subpath_keys(subpath) {
        let normalized_candidate = candidate_key
            .trim_start_matches("./")
            .trim_start_matches('/')
            .to_string();
        if !normalized_candidate.starts_with(prefix) || !normalized_candidate.ends_with(suffix) {
            continue;
        }
        return Some(
            normalized_candidate[prefix.len()..normalized_candidate.len() - suffix.len()]
                .to_string(),
        );
    }
    None
}

fn substitute_package_export_pattern(entry: &str, pattern_match: &str) -> String {
    if entry.contains('*') {
        entry.replace('*', pattern_match)
    } else {
        entry.to_string()
    }
}

fn read_package_export_entry(
    exports_value: Option<&serde_json::Value>,
) -> Option<PackageJsonEntryResolution> {
    read_package_export_entry_with_policy(exports_value, true)
}

fn read_sass_node_package_export_entry(
    exports_value: Option<&serde_json::Value>,
) -> Option<PackageJsonEntryResolution> {
    let exports_value = exports_value?;
    if exports_value.is_null() {
        return Some(PackageJsonEntryResolution::Blocked);
    }
    if let Some(entry) = exports_value.as_str() {
        if is_package_style_export_entry(entry) {
            return Some(PackageJsonEntryResolution::Resolved(entry.to_string()));
        }
        return None;
    }
    if let Some(entries) = exports_value.as_array() {
        for entry_value in entries {
            if let Some(entry) = read_sass_node_package_export_entry(Some(entry_value)) {
                return Some(entry);
            }
        }
        return None;
    }
    let exports_object = exports_value.as_object()?;
    if let Some(root_entry) = read_sass_node_package_export_entry(exports_object.get(".")) {
        return Some(root_entry);
    }
    for condition in ["sass", "style", "default"] {
        let entry = exports_object
            .get(condition)
            .and_then(|export_value| read_sass_node_package_export_entry(Some(export_value)));
        if let Some(entry) = entry {
            return Some(entry);
        }
    }
    None
}

fn read_package_export_entry_with_policy(
    exports_value: Option<&serde_json::Value>,
    require_style_entry: bool,
) -> Option<PackageJsonEntryResolution> {
    let exports_value = exports_value?;
    if exports_value.is_null() {
        return Some(PackageJsonEntryResolution::Blocked);
    }
    if let Some(entry) = exports_value.as_str() {
        if !require_style_entry || is_package_style_export_entry(entry) {
            return Some(PackageJsonEntryResolution::Resolved(entry.to_string()));
        }
        return None;
    }
    if let Some(entries) = exports_value.as_array() {
        for entry_value in entries {
            if let Some(entry) =
                read_package_export_entry_with_policy(Some(entry_value), require_style_entry)
            {
                return Some(entry);
            }
        }
        return None;
    }
    let exports_object = exports_value.as_object()?;
    if let Some(root_entry) =
        read_package_export_entry_with_policy(exports_object.get("."), require_style_entry)
    {
        return Some(root_entry);
    }
    for (key, export_value) in exports_object {
        let entry = if is_package_style_export_condition(key) {
            read_package_export_entry_with_policy(Some(export_value), false)
        } else if key == "default" {
            read_package_export_entry_with_policy(Some(export_value), true)
        } else {
            None
        };
        if let Some(entry) = entry {
            return Some(entry);
        }
    }
    None
}

fn is_package_style_export_condition(key: &str) -> bool {
    matches!(key, "sass" | "scss" | "style")
}

fn is_package_style_export_entry(entry: &str) -> bool {
    let normalized = normalize_package_json_entry(entry);
    let extension = Path::new(&normalized)
        .extension()
        .and_then(|extension| extension.to_str());
    matches!(extension, None | Some("css" | "scss" | "sass" | "less"))
}

fn read_package_json_string_field(
    package_object: &serde_json::Map<String, serde_json::Value>,
    key: &str,
) -> Option<String> {
    package_object
        .get(key)
        .and_then(|value| value.as_str())
        .map(ToString::to_string)
}

fn normalize_package_json_entry(entry: &str) -> String {
    entry
        .trim_start_matches("./")
        .trim_start_matches('/')
        .to_string()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct PackageStyleSource<'a> {
    pub(super) package_name: &'a str,
    pub(super) subpath: Option<&'a str>,
    pub(super) uses_sass_node_package_importer: bool,
}

pub(super) fn parse_package_style_source(source: &str) -> Option<PackageStyleSource<'_>> {
    let uses_sass_node_package_importer = source.starts_with("pkg:");
    let package_source = source.strip_prefix("pkg:").unwrap_or(source);
    if package_source.starts_with('.')
        || package_source.starts_with('/')
        || is_external_style_module_source(package_source)
    {
        return None;
    }

    if package_source.starts_with('@') {
        let mut segments = package_source.splitn(3, '/');
        let scope = segments.next()?;
        let package = segments.next()?;
        if scope.len() <= 1 || package.is_empty() {
            return None;
        }
        let package_name_end = scope.len() + 1 + package.len();
        let package_name = &package_source[..package_name_end];
        let subpath = segments.next().filter(|subpath| !subpath.is_empty());
        return Some(PackageStyleSource {
            package_name,
            subpath,
            uses_sass_node_package_importer,
        });
    }

    let mut segments = package_source.splitn(2, '/');
    let package_name = segments.next()?;
    if package_name.is_empty() {
        return None;
    }
    let subpath = segments.next().filter(|subpath| !subpath.is_empty());
    Some(PackageStyleSource {
        package_name,
        subpath,
        uses_sass_node_package_importer,
    })
}

pub(super) fn is_package_import_style_source(source: &str) -> bool {
    source.starts_with('#')
}
