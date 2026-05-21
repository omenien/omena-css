use super::*;
use std::fs;

pub fn resolve_omena_resolver_style_module_source(
    from_style_path: &str,
    source: &str,
    available_style_paths: &BTreeSet<&str>,
    package_manifests: &[OmenaResolverStylePackageManifestV0],
) -> Option<String> {
    summarize_omena_resolver_style_module_resolution(
        from_style_path,
        source,
        available_style_paths,
        package_manifests,
    )
    .resolved_style_path
}

pub fn resolve_omena_resolver_style_module_source_with_tsconfig_paths(
    from_style_path: &str,
    source: &str,
    available_style_paths: &BTreeSet<&str>,
    package_manifests: &[OmenaResolverStylePackageManifestV0],
    tsconfig_path_mappings: &[OmenaResolverTsconfigPathMappingV0],
) -> Option<String> {
    summarize_omena_resolver_style_module_resolution_with_tsconfig_paths(
        from_style_path,
        source,
        available_style_paths,
        package_manifests,
        tsconfig_path_mappings,
    )
    .resolved_style_path
}

pub fn summarize_omena_resolver_style_module_resolution(
    from_style_path: &str,
    source: &str,
    available_style_paths: &BTreeSet<&str>,
    package_manifests: &[OmenaResolverStylePackageManifestV0],
) -> OmenaResolverStyleModuleResolutionV0 {
    summarize_omena_resolver_style_module_resolution_with_tsconfig_paths(
        from_style_path,
        source,
        available_style_paths,
        package_manifests,
        &[],
    )
}

pub fn summarize_omena_resolver_style_module_resolution_with_tsconfig_paths(
    from_style_path: &str,
    source: &str,
    available_style_paths: &BTreeSet<&str>,
    package_manifests: &[OmenaResolverStylePackageManifestV0],
    tsconfig_path_mappings: &[OmenaResolverTsconfigPathMappingV0],
) -> OmenaResolverStyleModuleResolutionV0 {
    let candidates = collect_omena_resolver_style_module_source_candidates_with_tsconfig_paths(
        from_style_path,
        source,
        package_manifests,
        tsconfig_path_mappings,
    );
    let resolved_style_path =
        resolve_style_module_candidate_from_available_paths(&candidates, available_style_paths);
    let resolution_kind = if resolved_style_path.is_some() {
        if source_matches_tsconfig_path_mapping(source, tsconfig_path_mappings) {
            "tsconfigPathStyleModule"
        } else if is_package_import_style_source(source) {
            "packageImportStyleModule"
        } else if parse_package_style_source(source).is_some() {
            "packageStyleModule"
        } else {
            "relativeStyleModule"
        }
    } else if is_external_style_module_source(source) {
        "externalIgnored"
    } else {
        "unresolved"
    };

    OmenaResolverStyleModuleResolutionV0 {
        schema_version: "0",
        product: "omena-resolver.style-module-resolution",
        from_style_path: from_style_path.to_string(),
        source: source.to_string(),
        resolved_style_path,
        candidate_count: candidates.len(),
        candidates,
        resolution_kind,
    }
}

pub fn collect_omena_resolver_style_module_source_candidates(
    from_style_path: &str,
    source: &str,
    package_manifests: &[OmenaResolverStylePackageManifestV0],
) -> Vec<String> {
    collect_omena_resolver_style_module_source_candidates_with_tsconfig_paths(
        from_style_path,
        source,
        package_manifests,
        &[],
    )
}

pub fn collect_omena_resolver_style_module_source_candidates_with_tsconfig_paths(
    from_style_path: &str,
    source: &str,
    package_manifests: &[OmenaResolverStylePackageManifestV0],
    tsconfig_path_mappings: &[OmenaResolverTsconfigPathMappingV0],
) -> Vec<String> {
    if is_external_style_module_source(source) {
        return Vec::new();
    }

    let mut candidates = Vec::new();
    for base_path in tsconfig_style_module_base_candidates(source, tsconfig_path_mappings) {
        push_style_module_path_candidates(&mut candidates, base_path, true);
    }

    if is_package_import_style_source(source) {
        for base_path in
            package_import_style_module_base_candidates(from_style_path, source, package_manifests)
        {
            push_style_module_path_candidates(&mut candidates, base_path, true);
        }
        return candidates;
    }

    let source_path = Path::new(source);
    let base_path = if source_path.is_absolute() {
        PathBuf::from(source)
    } else {
        Path::new(from_style_path)
            .parent()
            .map(|parent| parent.join(source))
            .unwrap_or_else(|| PathBuf::from(source))
    };
    push_style_module_path_candidates(
        &mut candidates,
        base_path,
        source_path.extension().is_none(),
    );
    for package_manifest_base_path in
        package_manifest_style_module_base_candidates(from_style_path, source, package_manifests)
    {
        push_style_module_path_candidates(&mut candidates, package_manifest_base_path, true);
    }
    for package_base_path in package_style_module_base_candidates(from_style_path, source) {
        push_style_module_path_candidates(&mut candidates, package_base_path, true);
    }

    candidates
}

pub fn summarize_omena_resolver_specifier_resolution_runtime(
    from_style_path: &str,
    sources: &[String],
    available_style_paths: &BTreeSet<&str>,
    package_manifests: &[OmenaResolverStylePackageManifestV0],
    tsconfig_path_mappings: &[OmenaResolverTsconfigPathMappingV0],
) -> OmenaResolverSpecifierResolutionRuntimeV0 {
    let mut entries = sources
        .iter()
        .map(|source| {
            let resolution = summarize_omena_resolver_style_module_resolution_with_tsconfig_paths(
                from_style_path,
                source,
                available_style_paths,
                package_manifests,
                tsconfig_path_mappings,
            );
            let status = if resolution.resolution_kind == "externalIgnored" {
                "external"
            } else if resolution.resolved_style_path.is_some() {
                "resolved"
            } else {
                "unresolved"
            };
            OmenaResolverSpecifierResolutionRuntimeEntryV0 {
                source: source.clone(),
                resolved_style_path: resolution.resolved_style_path,
                candidate_count: resolution.candidate_count,
                resolution_kind: resolution.resolution_kind,
                status,
            }
        })
        .collect::<Vec<_>>();
    entries.sort_by_key(|entry| (entry.status, entry.source.clone()));

    let resolved_specifier_count = entries
        .iter()
        .filter(|entry| entry.status == "resolved")
        .count();
    let external_specifier_count = entries
        .iter()
        .filter(|entry| entry.status == "external")
        .count();
    let unresolved_specifier_count = entries
        .len()
        .saturating_sub(resolved_specifier_count + external_specifier_count);

    OmenaResolverSpecifierResolutionRuntimeV0 {
        schema_version: "0",
        product: "omena-resolver.specifier-resolution-runtime",
        from_style_path: from_style_path.to_string(),
        specifier_count: entries.len(),
        resolved_specifier_count,
        external_specifier_count,
        unresolved_specifier_count,
        entries,
        ready_surfaces: vec![
            "specifierResolutionRuntime",
            "batchStyleModuleResolution",
            "tsconfigPathMapping",
            "packageManifestResolution",
            "externalSpecifierFiltering",
        ],
    }
}

fn tsconfig_style_module_base_candidates(
    source: &str,
    tsconfig_path_mappings: &[OmenaResolverTsconfigPathMappingV0],
) -> Vec<PathBuf> {
    let mut candidates = Vec::new();
    for (mapping, pattern_match) in
        best_tsconfig_path_mapping_matches(source, tsconfig_path_mappings)
    {
        for target_pattern in &mapping.target_patterns {
            let substituted_target =
                substitute_tsconfig_path_pattern(target_pattern, pattern_match);
            push_unique_pathbuf(
                &mut candidates,
                Path::new(&mapping.base_path).join(substituted_target),
            );
        }
    }
    candidates
}

fn best_tsconfig_path_mapping_matches<'mapping, 'source>(
    source: &'source str,
    tsconfig_path_mappings: &'mapping [OmenaResolverTsconfigPathMappingV0],
) -> Vec<(&'mapping OmenaResolverTsconfigPathMappingV0, &'source str)> {
    let mut matches = tsconfig_path_mappings
        .iter()
        .enumerate()
        .filter_map(|(index, mapping)| {
            let priority = tsconfig_path_mapping_priority(&mapping.pattern, source)?;
            let pattern_match = match_tsconfig_path_pattern(&mapping.pattern, source)?;
            Some((index, priority, mapping, pattern_match))
        })
        .collect::<Vec<_>>();
    // TypeScript resolves path mappings through the best pattern, not the
    // first matching entry, so a less-specific alias must not shadow it.
    matches.sort_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(&right.0)));

    let Some(best_priority) = matches.first().map(|(_, priority, _, _)| *priority) else {
        return Vec::new();
    };
    matches
        .into_iter()
        .take_while(|(_, priority, _, _)| *priority == best_priority)
        .map(|(_, _, mapping, pattern_match)| (mapping, pattern_match))
        .collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct TsconfigPathMappingPriority {
    exact_rank: u8,
    prefix_len: usize,
    suffix_len: usize,
    pattern_len: usize,
}

fn tsconfig_path_mapping_priority(
    pattern: &str,
    source: &str,
) -> Option<TsconfigPathMappingPriority> {
    if let Some((prefix, suffix)) = pattern.split_once('*') {
        if suffix.contains('*') || !source.starts_with(prefix) || !source.ends_with(suffix) {
            return None;
        }
        return Some(TsconfigPathMappingPriority {
            exact_rank: 0,
            prefix_len: prefix.len(),
            suffix_len: suffix.len(),
            pattern_len: pattern.len(),
        });
    }
    (pattern == source).then_some(TsconfigPathMappingPriority {
        exact_rank: 1,
        prefix_len: pattern.len(),
        suffix_len: 0,
        pattern_len: pattern.len(),
    })
}

fn source_matches_tsconfig_path_mapping(
    source: &str,
    tsconfig_path_mappings: &[OmenaResolverTsconfigPathMappingV0],
) -> bool {
    tsconfig_path_mappings
        .iter()
        .any(|mapping| match_tsconfig_path_pattern(&mapping.pattern, source).is_some())
}

fn match_tsconfig_path_pattern<'a>(pattern: &str, source: &'a str) -> Option<&'a str> {
    if let Some((prefix, suffix)) = pattern.split_once('*') {
        if suffix.contains('*') || !source.starts_with(prefix) || !source.ends_with(suffix) {
            return None;
        }
        return Some(&source[prefix.len()..source.len() - suffix.len()]);
    }
    (pattern == source).then_some("")
}

fn substitute_tsconfig_path_pattern(target_pattern: &str, pattern_match: &str) -> String {
    if target_pattern.contains('*') {
        target_pattern.replace('*', pattern_match)
    } else {
        target_pattern.to_string()
    }
}

fn is_external_style_module_source(source: &str) -> bool {
    source.starts_with("sass:") || source.starts_with("http://") || source.starts_with("https://")
}

pub fn canonicalize_omena_resolver_style_identity_path(path: &str) -> String {
    fs::canonicalize(path)
        .map(normalize_style_path)
        .unwrap_or_else(|_| normalize_style_path(PathBuf::from(path)))
}

fn resolve_style_module_candidate_from_available_paths(
    candidates: &[String],
    available_style_paths: &BTreeSet<&str>,
) -> Option<String> {
    for candidate in candidates {
        if available_style_paths.contains(candidate.as_str()) {
            return Some(candidate.clone());
        }
    }

    let available_by_identity = available_style_paths
        .iter()
        .map(|path| {
            (
                canonicalize_omena_resolver_style_identity_path(path),
                (*path).to_string(),
            )
        })
        .collect::<BTreeMap<_, _>>();

    candidates.iter().find_map(|candidate| {
        available_by_identity
            .get(canonicalize_omena_resolver_style_identity_path(candidate).as_str())
            .cloned()
    })
}

fn push_style_module_path_candidates(
    candidates: &mut Vec<String>,
    base_path: PathBuf,
    include_extension_variants: bool,
) {
    push_style_path_candidate(candidates, base_path.clone());
    push_partial_style_path_candidate(candidates, &base_path);

    if !include_extension_variants {
        return;
    }

    for extension in [
        ".module.scss",
        ".module.sass",
        ".module.css",
        ".module.less",
        ".scss",
        ".sass",
        ".css",
        ".less",
    ] {
        let candidate = PathBuf::from(format!("{}{}", base_path.display(), extension));
        push_style_path_candidate(candidates, candidate.clone());
        push_partial_style_path_candidate(candidates, &candidate);
    }
}

fn package_style_module_base_candidates(from_style_path: &str, source: &str) -> Vec<PathBuf> {
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

fn package_manifest_style_module_base_candidates(
    from_style_path: &str,
    source: &str,
    package_manifests: &[OmenaResolverStylePackageManifestV0],
) -> Vec<PathBuf> {
    let Some(package_source) = parse_package_style_source(source) else {
        return Vec::new();
    };
    let Some(from_dir) = Path::new(from_style_path).parent() else {
        return Vec::new();
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
            && let Some(entry) =
                read_package_manifest_style_entry(package_json_source, package_source.subpath)
        {
            push_unique_pathbuf(&mut candidates, package_root.join(entry));
        }
        current_dir = dir.parent();
    }
    candidates
}

fn package_import_style_module_base_candidates(
    from_style_path: &str,
    source: &str,
    package_manifests: &[OmenaResolverStylePackageManifestV0],
) -> Vec<PathBuf> {
    let Some(from_dir) = Path::new(from_style_path).parent() else {
        return Vec::new();
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
            push_package_import_entry_candidates(
                &mut candidates,
                dir,
                from_style_path,
                &entry,
                package_manifests,
            );
            break;
        }
        current_dir = dir.parent();
    }
    candidates
}

fn push_package_import_entry_candidates(
    candidates: &mut Vec<PathBuf>,
    package_dir: &Path,
    from_style_path: &str,
    entry: &str,
    package_manifests: &[OmenaResolverStylePackageManifestV0],
) {
    if entry.starts_with("./") {
        push_unique_pathbuf(
            candidates,
            package_dir.join(normalize_package_json_entry(entry)),
        );
        return;
    }
    if entry.starts_with('#') || is_external_style_module_source(entry) {
        return;
    }
    for package_manifest_base_path in
        package_manifest_style_module_base_candidates(from_style_path, entry, package_manifests)
    {
        push_unique_pathbuf(candidates, package_manifest_base_path);
    }
    for package_base_path in package_style_module_base_candidates(from_style_path, entry) {
        push_unique_pathbuf(candidates, package_base_path);
    }
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
) -> Option<PathBuf> {
    let package_json = serde_json::from_str::<serde_json::Value>(package_json_source).ok()?;
    let package_object = package_json.as_object()?;
    let entry = if let Some(subpath) = subpath {
        read_package_export_subpath_entry(package_object.get("exports"), subpath)
    } else {
        read_package_export_entry(package_object.get("exports"))
            .or_else(|| read_package_json_string_field(package_object, "sass"))
            .or_else(|| read_package_json_string_field(package_object, "scss"))
            .or_else(|| read_package_json_string_field(package_object, "style"))
    }?;
    Some(PathBuf::from(normalize_package_json_entry(&entry)))
}

fn read_package_import_entry(package_json_source: &str, specifier: &str) -> Option<String> {
    let package_json = serde_json::from_str::<serde_json::Value>(package_json_source).ok()?;
    let package_object = package_json.as_object()?;
    let imports_object = package_object.get("imports")?.as_object()?;
    if let Some(entry) = read_package_export_entry(imports_object.get(specifier)) {
        return Some(entry);
    }
    for (key, import_value) in imports_object {
        let Some(pattern_match) = match_package_import_pattern(key, specifier) else {
            continue;
        };
        let Some(entry) = read_package_export_entry(Some(import_value)) else {
            continue;
        };
        return Some(substitute_package_export_pattern(&entry, &pattern_match));
    }
    None
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
) -> Option<String> {
    let exports_object = exports_value?.as_object()?;
    for key in package_export_subpath_keys(subpath) {
        if let Some(entry) = read_package_export_entry(exports_object.get(&key)) {
            return Some(entry);
        }
    }
    for (key, export_value) in exports_object {
        let Some(pattern_match) = match_package_export_subpath_pattern(key, subpath) else {
            continue;
        };
        let Some(entry) = read_package_export_entry(Some(export_value)) else {
            continue;
        };
        return Some(substitute_package_export_pattern(&entry, &pattern_match));
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

fn read_package_export_entry(exports_value: Option<&serde_json::Value>) -> Option<String> {
    let exports_value = exports_value?;
    if let Some(entry) = exports_value.as_str() {
        return Some(entry.to_string());
    }
    if let Some(entries) = exports_value.as_array() {
        for entry_value in entries {
            if let Some(entry) = read_package_export_entry(Some(entry_value)) {
                return Some(entry);
            }
        }
        return None;
    }
    let exports_object = exports_value.as_object()?;
    if let Some(root_entry) = read_package_export_entry(exports_object.get(".")) {
        return Some(root_entry);
    }
    for (key, export_value) in exports_object {
        if !is_package_style_export_condition(key) {
            continue;
        }
        if let Some(entry) = read_package_export_entry(Some(export_value)) {
            return Some(entry);
        }
    }
    None
}

fn is_package_style_export_condition(key: &str) -> bool {
    matches!(
        key,
        "sass" | "scss" | "style" | "default" | "import" | "require"
    )
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
struct PackageStyleSource<'a> {
    package_name: &'a str,
    subpath: Option<&'a str>,
}

fn parse_package_style_source(source: &str) -> Option<PackageStyleSource<'_>> {
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
    })
}

fn is_package_import_style_source(source: &str) -> bool {
    source.starts_with('#')
}

fn push_unique_pathbuf(candidates: &mut Vec<PathBuf>, value: PathBuf) {
    if !candidates.contains(&value) {
        candidates.push(value);
    }
}

fn push_partial_style_path_candidate(candidates: &mut Vec<String>, path: &Path) {
    let Some(file_name) = path.file_name().and_then(|file_name| file_name.to_str()) else {
        return;
    };
    if file_name.starts_with('_') {
        return;
    }
    let mut partial_path = path.to_path_buf();
    partial_path.set_file_name(format!("_{file_name}"));
    push_style_path_candidate(candidates, partial_path);
}

fn push_style_path_candidate(candidates: &mut Vec<String>, path: PathBuf) {
    let candidate = normalize_style_path(path);
    if !candidates.contains(&candidate) {
        candidates.push(candidate);
    }
}

fn normalize_style_path(path: PathBuf) -> String {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            Component::Normal(part) => normalized.push(part),
            Component::RootDir | Component::Prefix(_) => normalized.push(component.as_os_str()),
        }
    }
    percent_decode_style_path(&normalized.to_string_lossy().replace('\\', "/"))
}

fn percent_decode_style_path(path: &str) -> String {
    let bytes = path.as_bytes();
    let mut output = Vec::with_capacity(bytes.len());
    let mut index = 0;
    let mut changed = false;

    while index < bytes.len() {
        if bytes[index] == b'%'
            && index + 2 < bytes.len()
            && let (Some(high), Some(low)) = (
                decode_hex_byte(bytes[index + 1]),
                decode_hex_byte(bytes[index + 2]),
            )
        {
            output.push((high << 4) | low);
            index += 3;
            changed = true;
            continue;
        }
        output.push(bytes[index]);
        index += 1;
    }

    if !changed {
        return path.to_string();
    }
    String::from_utf8(output).unwrap_or_else(|_| path.to_string())
}

fn decode_hex_byte(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}
