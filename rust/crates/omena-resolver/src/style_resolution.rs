use super::*;
use std::fs;

mod package_resolution;
mod path_mappings;

use package_resolution::{
    PackageStyleCandidateResolution, is_package_import_style_source,
    package_import_style_module_base_candidates, package_manifest_style_module_base_candidates,
    package_style_module_base_candidates, parse_package_style_source,
};
use path_mappings::{
    bundler_style_module_base_candidates, source_matches_bundler_path_mapping,
    source_matches_tsconfig_path_mapping, tsconfig_style_module_base_candidates,
};

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

pub fn resolve_omena_resolver_style_module_source_with_path_mappings(
    from_style_path: &str,
    source: &str,
    available_style_paths: &BTreeSet<&str>,
    package_manifests: &[OmenaResolverStylePackageManifestV0],
    bundler_path_mappings: &[OmenaResolverBundlerPathAliasMappingV0],
    tsconfig_path_mappings: &[OmenaResolverTsconfigPathMappingV0],
) -> Option<String> {
    summarize_omena_resolver_style_module_resolution_with_path_mappings(
        from_style_path,
        source,
        available_style_paths,
        package_manifests,
        bundler_path_mappings,
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
    summarize_omena_resolver_style_module_resolution_with_path_mappings(
        from_style_path,
        source,
        available_style_paths,
        package_manifests,
        &[],
        tsconfig_path_mappings,
    )
}

pub fn summarize_omena_resolver_style_module_resolution_with_path_mappings(
    from_style_path: &str,
    source: &str,
    available_style_paths: &BTreeSet<&str>,
    package_manifests: &[OmenaResolverStylePackageManifestV0],
    bundler_path_mappings: &[OmenaResolverBundlerPathAliasMappingV0],
    tsconfig_path_mappings: &[OmenaResolverTsconfigPathMappingV0],
) -> OmenaResolverStyleModuleResolutionV0 {
    let candidates = collect_omena_resolver_style_module_source_candidates_with_path_mappings(
        from_style_path,
        source,
        package_manifests,
        bundler_path_mappings,
        tsconfig_path_mappings,
    );
    let resolved_style_path =
        resolve_style_module_candidate_from_available_paths(&candidates, available_style_paths);
    let resolution_kind = if resolved_style_path.is_some() {
        if source_matches_bundler_path_mapping(source, bundler_path_mappings) {
            "bundlerPathStyleModule"
        } else if source_matches_tsconfig_path_mapping(source, tsconfig_path_mappings) {
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
    collect_omena_resolver_style_module_source_candidates_with_path_mappings(
        from_style_path,
        source,
        package_manifests,
        &[],
        tsconfig_path_mappings,
    )
}

pub fn collect_omena_resolver_style_module_source_candidates_with_path_mappings(
    from_style_path: &str,
    source: &str,
    package_manifests: &[OmenaResolverStylePackageManifestV0],
    bundler_path_mappings: &[OmenaResolverBundlerPathAliasMappingV0],
    tsconfig_path_mappings: &[OmenaResolverTsconfigPathMappingV0],
) -> Vec<String> {
    if is_external_style_module_source(source) {
        return Vec::new();
    }

    let mut candidates = Vec::new();
    for base_path in bundler_style_module_base_candidates(source, bundler_path_mappings) {
        push_style_module_path_candidates(&mut candidates, base_path, true);
    }
    for base_path in tsconfig_style_module_base_candidates(source, tsconfig_path_mappings) {
        push_style_module_path_candidates(&mut candidates, base_path, true);
    }

    if is_package_import_style_source(source) {
        match package_import_style_module_base_candidates(
            from_style_path,
            source,
            package_manifests,
        ) {
            PackageStyleCandidateResolution::Candidates(base_paths) => {
                for base_path in base_paths {
                    push_style_module_path_candidates(&mut candidates, base_path, true);
                }
            }
            PackageStyleCandidateResolution::Blocked => {}
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
    match package_manifest_style_module_base_candidates(from_style_path, source, package_manifests)
    {
        PackageStyleCandidateResolution::Candidates(base_paths) => {
            for package_manifest_base_path in base_paths {
                push_style_module_path_candidates(
                    &mut candidates,
                    package_manifest_base_path,
                    true,
                );
            }
        }
        PackageStyleCandidateResolution::Blocked => {
            return candidates;
        }
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
    summarize_omena_resolver_specifier_resolution_runtime_with_path_mappings(
        from_style_path,
        sources,
        available_style_paths,
        package_manifests,
        &[],
        tsconfig_path_mappings,
    )
}

pub fn summarize_omena_resolver_specifier_resolution_runtime_with_path_mappings(
    from_style_path: &str,
    sources: &[String],
    available_style_paths: &BTreeSet<&str>,
    package_manifests: &[OmenaResolverStylePackageManifestV0],
    bundler_path_mappings: &[OmenaResolverBundlerPathAliasMappingV0],
    tsconfig_path_mappings: &[OmenaResolverTsconfigPathMappingV0],
) -> OmenaResolverSpecifierResolutionRuntimeV0 {
    let mut entries = sources
        .iter()
        .map(|source| {
            let resolution = summarize_omena_resolver_style_module_resolution_with_path_mappings(
                from_style_path,
                source,
                available_style_paths,
                package_manifests,
                bundler_path_mappings,
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
            "bundlerPathAliasMapping",
            "tsconfigPathMapping",
            "packageManifestResolution",
            "externalSpecifierFiltering",
        ],
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

pub(crate) fn normalize_style_path(path: PathBuf) -> String {
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
