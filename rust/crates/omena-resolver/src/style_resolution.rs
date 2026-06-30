use super::*;
use std::cell::RefCell;
use std::fs;
#[cfg(test)]
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::{AtomicU64, Ordering};

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

static STYLE_IDENTITY_CACHE_VERSION: AtomicU64 = AtomicU64::new(0);
#[cfg(test)]
static STYLE_IDENTITY_INDEX_BUILD_COUNT: AtomicUsize = AtomicUsize::new(0);
#[cfg(test)]
static STYLE_IDENTITY_INDEX_BUILD_WORK_COUNT: AtomicUsize = AtomicUsize::new(0);

thread_local! {
    static STYLE_IDENTITY_CANONICALIZE_CACHE: RefCell<StyleIdentityCanonicalizeCache> = const {
        RefCell::new(StyleIdentityCanonicalizeCache {
            version: 0,
            paths: BTreeMap::new(),
        })
    };
    static STYLE_IDENTITY_READ_LINK_CACHE: RefCell<StyleIdentityReadLinkCache> = const {
        RefCell::new(StyleIdentityReadLinkCache {
            version: 0,
            links: BTreeMap::new(),
        })
    };

    #[cfg(test)]
    static STYLE_IDENTITY_CANONICALIZE_SYSCALL_COUNT: std::cell::Cell<usize> = const {
        std::cell::Cell::new(0)
    };
    #[cfg(test)]
    static STYLE_IDENTITY_READ_LINK_SYSCALL_COUNT: std::cell::Cell<usize> = const {
        std::cell::Cell::new(0)
    };
}

struct StyleIdentityCanonicalizeCache {
    version: u64,
    paths: BTreeMap<PathBuf, Option<String>>,
}

struct StyleIdentityReadLinkCache {
    version: u64,
    links: BTreeMap<PathBuf, Option<PathBuf>>,
}

pub fn invalidate_omena_resolver_style_identity_cache() {
    let next_version = STYLE_IDENTITY_CACHE_VERSION
        .fetch_add(1, Ordering::AcqRel)
        .saturating_add(1);
    STYLE_IDENTITY_CANONICALIZE_CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        cache.version = next_version;
        cache.paths.clear();
    });
    STYLE_IDENTITY_READ_LINK_CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        cache.version = next_version;
        cache.links.clear();
    });
}

#[cfg(test)]
pub(crate) fn reset_omena_resolver_style_identity_cache_for_test() {
    invalidate_omena_resolver_style_identity_cache();
    reset_omena_resolver_style_identity_syscall_counts_for_test();
}

#[cfg(test)]
pub(crate) fn reset_omena_resolver_style_identity_syscall_counts_for_test() {
    STYLE_IDENTITY_CANONICALIZE_SYSCALL_COUNT.with(|count| count.set(0));
    STYLE_IDENTITY_READ_LINK_SYSCALL_COUNT.with(|count| count.set(0));
    STYLE_IDENTITY_INDEX_BUILD_COUNT.store(0, Ordering::Release);
    STYLE_IDENTITY_INDEX_BUILD_WORK_COUNT.store(0, Ordering::Release);
}

#[cfg(test)]
pub(crate) fn omena_resolver_style_identity_canonicalize_syscall_count_for_test() -> usize {
    STYLE_IDENTITY_CANONICALIZE_SYSCALL_COUNT.with(std::cell::Cell::get)
}

#[cfg(test)]
pub(crate) fn omena_resolver_style_identity_read_link_syscall_count_for_test() -> usize {
    STYLE_IDENTITY_READ_LINK_SYSCALL_COUNT.with(std::cell::Cell::get)
}

#[cfg(test)]
pub(crate) fn omena_resolver_style_identity_index_build_count_for_test() -> usize {
    STYLE_IDENTITY_INDEX_BUILD_COUNT.load(Ordering::Acquire)
}

#[cfg(test)]
pub(crate) fn omena_resolver_style_identity_index_build_work_count_for_test() -> usize {
    STYLE_IDENTITY_INDEX_BUILD_WORK_COUNT.load(Ordering::Acquire)
}

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
    summarize_omena_resolver_style_module_resolution_with_load_path_roots(
        from_style_path,
        source,
        available_style_paths,
        package_manifests,
        bundler_path_mappings,
        tsconfig_path_mappings,
        &[],
    )
}

/// Resolve a style-module specifier, additionally trying each `load_path_roots` entry as a
/// load-path root for **path-shaped, non-`./`-relative, non-package** specifiers (the dart-sass
/// `--load-path` behavior). File-relative, alias, and bare-package routing are unchanged: the
/// load-path candidates are appended last and only accepted when they exist in
/// `available_style_paths`, so a genuinely missing module still flags and a real external package
/// import still routes through the package resolver. (RFC-0007-I, #49)
pub fn summarize_omena_resolver_style_module_resolution_with_load_path_roots(
    from_style_path: &str,
    source: &str,
    available_style_paths: &BTreeSet<&str>,
    package_manifests: &[OmenaResolverStylePackageManifestV0],
    bundler_path_mappings: &[OmenaResolverBundlerPathAliasMappingV0],
    tsconfig_path_mappings: &[OmenaResolverTsconfigPathMappingV0],
    load_path_roots: &[&str],
) -> OmenaResolverStyleModuleResolutionV0 {
    summarize_omena_resolver_style_module_resolution_with_confirmation_inputs(
        from_style_path,
        source,
        available_style_paths,
        &[],
        package_manifests,
        bundler_path_mappings,
        tsconfig_path_mappings,
        load_path_roots,
        OmenaResolverStyleModuleConfirmationOptionsV0::default(),
    )
}

#[allow(clippy::too_many_arguments)]
pub fn summarize_omena_resolver_style_module_resolution_with_confirmation_inputs(
    from_style_path: &str,
    source: &str,
    available_style_paths: &BTreeSet<&str>,
    disk_style_path_identities: &[OmenaResolverStyleModuleDiskCandidateIdentityV0],
    package_manifests: &[OmenaResolverStylePackageManifestV0],
    bundler_path_mappings: &[OmenaResolverBundlerPathAliasMappingV0],
    tsconfig_path_mappings: &[OmenaResolverTsconfigPathMappingV0],
    load_path_roots: &[&str],
    confirmation_options: OmenaResolverStyleModuleConfirmationOptionsV0<'_>,
) -> OmenaResolverStyleModuleResolutionV0 {
    let routing_source = normalize_omena_resolver_style_module_source_for_routing(source);
    let candidates = collect_omena_resolver_style_module_source_candidates_with_load_path_roots(
        from_style_path,
        source,
        package_manifests,
        bundler_path_mappings,
        tsconfig_path_mappings,
        load_path_roots,
    );
    let confirmation = confirm_omena_resolver_style_module_candidate_with_options(
        &candidates,
        available_style_paths,
        disk_style_path_identities,
        confirmation_options,
    );
    let resolved_style_path = confirmation.resolved_style_path;
    let resolution_kind = if let Some(resolved_style_path) = resolved_style_path.as_deref() {
        if source != routing_source {
            "tildeStyleModule"
        } else if source_matches_bundler_path_mapping(routing_source, bundler_path_mappings) {
            "bundlerPathStyleModule"
        } else if source_matches_tsconfig_path_mapping(routing_source, tsconfig_path_mappings) {
            "tsconfigPathStyleModule"
        } else if is_package_import_style_source(routing_source) {
            "packageImportStyleModule"
        } else if resolved_via_load_path_root_candidate(
            from_style_path,
            routing_source,
            resolved_style_path,
            load_path_roots,
        ) {
            // A load-path-rooted resolution can `parse_package_style_source` (a path-shaped
            // specifier like `src/scss/design-system.scss` reads as package `src`), so this
            // probe must precede the bare-package classification to avoid mislabeling. (#49)
            "loadPathStyleModule"
        } else if parse_package_style_source(routing_source).is_some() {
            "packageStyleModule"
        } else {
            "relativeStyleModule"
        }
    } else if is_external_style_module_source(routing_source) {
        "externalIgnored"
    } else {
        "unresolved"
    };

    OmenaResolverStyleModuleResolutionV0 {
        schema_version: "0",
        product: "omena-resolver.style-module-resolution",
        from_style_path: from_style_path.to_string(),
        source: source.to_string(),
        symlink_chain: summarize_omena_resolver_symlink_chain_for_style_resolution(
            candidates.as_slice(),
            resolved_style_path.as_deref(),
        ),
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

pub fn summarize_omena_resolver_style_resolution_policy_v0() -> OmenaResolverStyleResolutionPolicyV0
{
    OmenaResolverStyleResolutionPolicyV0 {
        schema_version: "0",
        product: "omena-resolver.style-resolution-policy",
        candidate_strategy: "orderedFirstExistingCandidate",
        network_access: "neverFetch",
        steps: vec![
            OmenaResolverStyleResolutionPolicyStepV0 {
                order: 0,
                key: "externalUrlBoundary",
                applies_to: "http/https/protocol-relative references",
                precedence: "blocked before local candidate generation",
                candidate_semantics: "network references are external boundaries and are never fetched",
            },
            OmenaResolverStyleResolutionPolicyStepV0 {
                order: 10,
                key: "bundlerPathMapping",
                applies_to: "configured bundler resolve.alias mappings",
                precedence: "before tsconfig paths and package resolution",
                candidate_semantics: "webpack-compatible first matching alias, with `$` exact aliases",
            },
            OmenaResolverStyleResolutionPolicyStepV0 {
                order: 20,
                key: "tsconfigPathMapping",
                applies_to: "configured tsconfig paths mappings",
                precedence: "after bundler aliases, before package resolution",
                candidate_semantics: "TypeScript-compatible exact and wildcard path candidates from configured base paths",
            },
            OmenaResolverStyleResolutionPolicyStepV0 {
                order: 30,
                key: "sassPkgImporter",
                applies_to: "explicit pkg: Sass package importer specifiers",
                precedence: "exclusive package-import route",
                candidate_semantics: "package exports/style/sass candidates are generated and no local fallback is attempted",
            },
            OmenaResolverStyleResolutionPolicyStepV0 {
                order: 40,
                key: "fileRelativeOrAbsolute",
                applies_to: "relative, parent-relative, and absolute style specifiers",
                precedence: "before package-manifest and node package fallback",
                candidate_semantics: "extension and Sass partial candidates are generated relative to the importer",
            },
            OmenaResolverStyleResolutionPolicyStepV0 {
                order: 50,
                key: "packageManifestSubpath",
                applies_to: "path-shaped package subpaths with available package manifests",
                precedence: "after file-relative candidates, before node package fallback",
                candidate_semantics: "package.json exports/imports/style/sass candidates are appended when not blocked",
            },
            OmenaResolverStyleResolutionPolicyStepV0 {
                order: 60,
                key: "nodePackageFallback",
                applies_to: "bare package-style specifiers",
                precedence: "after package-manifest subpath candidates",
                candidate_semantics: "node_modules-style package candidates are appended as filesystem candidates",
            },
            OmenaResolverStyleResolutionPolicyStepV0 {
                order: 70,
                key: "sassLoadPathRoot",
                applies_to: "path-shaped non-relative specifiers with explicit style extensions",
                precedence: "last local fallback",
                candidate_semantics: "dart-sass load-path candidates are appended only when relative/package routes did not already win",
            },
        ],
        ready_surfaces: vec![
            "resolutionPolicyReport",
            "bundlerAliasBeforeTsconfig",
            "webpackFirstAliasMatch",
            "tsconfigPathMapping",
            "sassPkgImporterBoundary",
            "sassLoadPathFallback",
            "networkFetchForbidden",
        ],
    }
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
    collect_omena_resolver_style_module_source_candidates_with_load_path_roots(
        from_style_path,
        source,
        package_manifests,
        bundler_path_mappings,
        tsconfig_path_mappings,
        &[],
    )
}

pub fn collect_omena_resolver_style_module_source_candidates_with_load_path_roots(
    from_style_path: &str,
    source: &str,
    package_manifests: &[OmenaResolverStylePackageManifestV0],
    bundler_path_mappings: &[OmenaResolverBundlerPathAliasMappingV0],
    tsconfig_path_mappings: &[OmenaResolverTsconfigPathMappingV0],
    load_path_roots: &[&str],
) -> Vec<String> {
    let source = normalize_omena_resolver_style_module_source_for_routing(source);
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
            // Append load-path candidates even when package-manifest resolution is blocked: a
            // path-shaped specifier whose leading segment is misread as a package (`src/...`)
            // is exactly the load-path case, and the file-relative candidate above is the only
            // other route. Load-path candidates are appended last so file-relative wins. (#49)
            push_load_path_rooted_candidates(&mut candidates, source, load_path_roots);
            return candidates;
        }
    }
    for package_base_path in package_style_module_base_candidates(from_style_path, source) {
        push_style_module_path_candidates(&mut candidates, package_base_path, true);
    }
    push_load_path_rooted_candidates(&mut candidates, source, load_path_roots);

    candidates
}

pub fn normalize_omena_resolver_style_module_source_for_routing(source: &str) -> &str {
    source
        .strip_prefix("~/")
        .or_else(|| {
            source
                .strip_prefix('~')
                .filter(|stripped| !stripped.is_empty())
        })
        .unwrap_or(source)
}

/// True iff a non-`./`-relative, non-package-import specifier is path-shaped, i.e. eligible for
/// load-path rooting (dart-sass `--load-path`). Bare-package specifiers (`pkg`, `@scope/pkg`)
/// and explicit relative/absolute specifiers are excluded so the package and file-relative
/// routes are never shadowed. A specifier qualifies when it carries a recognized style
/// extension (`.scss`/`.sass`/`.css`/`.less`) — that is the dart-sass load-path form that the
/// design-system corpus uses (`'src/scss/design-system.scss'`). (#49)
fn is_load_path_shaped_style_specifier(source: &str) -> bool {
    if source.starts_with("./")
        || source.starts_with("../")
        || source.starts_with('/')
        || source.starts_with('#')
        || source.starts_with('@')
        || source.starts_with("pkg:")
        || is_external_style_module_source(source)
    {
        return false;
    }
    // Require a directory segment so a single bare filename never reroutes through load paths.
    if !source.contains('/') {
        return false;
    }
    let extension = Path::new(source)
        .extension()
        .and_then(|extension| extension.to_str())
        .map(str::to_ascii_lowercase);
    matches!(extension.as_deref(), Some("scss" | "sass" | "css" | "less"))
}

fn push_load_path_rooted_candidates(
    candidates: &mut Vec<String>,
    source: &str,
    load_path_roots: &[&str],
) {
    if load_path_roots.is_empty() || !is_load_path_shaped_style_specifier(source) {
        return;
    }
    for root in load_path_roots {
        let base_path = Path::new(root).join(source);
        // The specifier already carries an explicit style extension, so emit exactly that path
        // (plus its `_partial` sibling) without re-appending extension variants.
        push_style_module_path_candidates(candidates, base_path, false);
    }
}

/// Determine whether `resolved_style_path` was reached through a load-path root rather than the
/// file-relative or bare-package routes, used only to classify `resolution_kind`. (#49)
fn resolved_via_load_path_root_candidate(
    from_style_path: &str,
    source: &str,
    resolved_style_path: &str,
    load_path_roots: &[&str],
) -> bool {
    if load_path_roots.is_empty() || !is_load_path_shaped_style_specifier(source) {
        return false;
    }
    // If the file-relative candidate already produced this path, it is not a load-path resolution.
    let relative_base = Path::new(from_style_path)
        .parent()
        .map(|parent| parent.join(source))
        .unwrap_or_else(|| PathBuf::from(source));
    let mut relative_candidates = Vec::new();
    push_style_module_path_candidates(&mut relative_candidates, relative_base, false);
    if relative_candidates
        .iter()
        .any(|candidate| style_paths_share_identity(candidate, resolved_style_path))
    {
        return false;
    }
    let mut load_path_candidates = Vec::new();
    push_load_path_rooted_candidates(&mut load_path_candidates, source, load_path_roots);
    load_path_candidates
        .iter()
        .any(|candidate| style_paths_share_identity(candidate, resolved_style_path))
}

fn style_paths_share_identity(left: &str, right: &str) -> bool {
    left == right
        || canonicalize_omena_resolver_style_identity_path(left)
            == canonicalize_omena_resolver_style_identity_path(right)
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

/// True iff a specifier is a fully-qualified external module URL — one the resolver must NOT
/// attempt to canonicalize against the in-graph `available_style_paths` and must route through the
/// external-SIF branch (`externalIgnored` -> `status == "external"`) instead. (#33/#34)
///
/// The `sass:` builtin and `http(s)://` remote schemes were always external. `file://` joins them
/// because it is the canonical-URL form that a bridge-generated SIF carries
/// (`generate_omena_bridge_sif_for_resolved_style_path` returns `canonical_url = file://<abs>`):
/// an `@use "file:///…"` edge is a fully-resolved on-disk URL, so it is matched 1:1 against an
/// in-scope SIF's `canonical_url` rather than re-joined as a workspace-relative candidate. A
/// `file://` edge with NO SIF in scope stays in the external lane and surfaces as
/// `missingExternalSif` (the #34 boundary state) — it is never silently demoted to `unresolved`.
/// `file://` is never an in-graph `@use` source (style files are referenced relatively), so no
/// in-graph resolution is shadowed by this classification.
fn is_external_style_module_source(source: &str) -> bool {
    source.starts_with("sass:")
        || source.starts_with("http://")
        || source.starts_with("https://")
        || source.starts_with("file://")
}

pub fn canonicalize_omena_resolver_style_identity_path(path: &str) -> String {
    let path = PathBuf::from(style_identity_path_input(path));
    canonicalize_omena_resolver_style_identity_existing_path(path.as_path())
        .unwrap_or_else(|| normalize_style_path(path))
}

fn style_identity_path_input(path: &str) -> &str {
    if let Some(path) = path.strip_prefix("file://") {
        return path;
    }
    path.strip_prefix("file:")
        .filter(|path| path.starts_with('/'))
        .unwrap_or(path)
}

pub(super) fn canonicalize_omena_resolver_style_identity_existing_path(
    path: &Path,
) -> Option<String> {
    if let Some(cached) = style_identity_canonicalize_cache_get(path) {
        return cached;
    }

    let canonical = fs_canonicalize_omena_resolver_style_identity_path(path)
        .ok()
        .map(normalize_style_path);
    style_identity_canonicalize_cache_insert(path.to_path_buf(), canonical.clone());
    if let Some(canonical_path) = canonical.as_ref() {
        style_identity_canonicalize_cache_insert(
            PathBuf::from(canonical_path),
            Some(canonical_path.clone()),
        );
    }
    canonical
}

fn style_identity_canonicalize_cache_get(path: &Path) -> Option<Option<String>> {
    STYLE_IDENTITY_CANONICALIZE_CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        sync_style_identity_canonicalize_cache_version(&mut cache);
        cache.paths.get(path).cloned()
    })
}

fn style_identity_canonicalize_cache_insert(path: PathBuf, canonical: Option<String>) {
    STYLE_IDENTITY_CANONICALIZE_CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        sync_style_identity_canonicalize_cache_version(&mut cache);
        cache.paths.insert(path, canonical);
    });
}

fn sync_style_identity_canonicalize_cache_version(cache: &mut StyleIdentityCanonicalizeCache) {
    let current = STYLE_IDENTITY_CACHE_VERSION.load(Ordering::Acquire);
    if cache.version != current {
        cache.version = current;
        cache.paths.clear();
    }
}

fn fs_canonicalize_omena_resolver_style_identity_path(path: &Path) -> std::io::Result<PathBuf> {
    #[cfg(test)]
    STYLE_IDENTITY_CANONICALIZE_SYSCALL_COUNT.with(|count| {
        count.set(count.get().saturating_add(1));
    });
    fs::canonicalize(path)
}

fn read_link_omena_resolver_style_identity_path(path: &Path) -> Option<PathBuf> {
    if let Some(cached) = style_identity_read_link_cache_get(path) {
        return cached;
    }

    let target = fs_read_link_omena_resolver_style_identity_path(path).ok();
    style_identity_read_link_cache_insert(path.to_path_buf(), target.clone());
    target
}

fn style_identity_read_link_cache_get(path: &Path) -> Option<Option<PathBuf>> {
    STYLE_IDENTITY_READ_LINK_CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        sync_style_identity_read_link_cache_version(&mut cache);
        cache.links.get(path).cloned()
    })
}

fn style_identity_read_link_cache_insert(path: PathBuf, target: Option<PathBuf>) {
    STYLE_IDENTITY_READ_LINK_CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        sync_style_identity_read_link_cache_version(&mut cache);
        cache.links.insert(path, target);
    });
}

fn sync_style_identity_read_link_cache_version(cache: &mut StyleIdentityReadLinkCache) {
    let current = STYLE_IDENTITY_CACHE_VERSION.load(Ordering::Acquire);
    if cache.version != current {
        cache.version = current;
        cache.links.clear();
    }
}

fn fs_read_link_omena_resolver_style_identity_path(path: &Path) -> std::io::Result<PathBuf> {
    #[cfg(test)]
    STYLE_IDENTITY_READ_LINK_SYSCALL_COUNT.with(|count| {
        count.set(count.get().saturating_add(1));
    });
    fs::read_link(path)
}

pub fn inspect_omena_resolver_symlink_chain_v0(
    path: &str,
) -> OmenaResolverSymlinkChainInspectionV0 {
    let requested_path = normalize_style_path(PathBuf::from(path));
    let mut current = PathBuf::new();
    let mut inspected_component_count = 0usize;
    let mut links = Vec::new();

    for component in Path::new(&requested_path).components() {
        match component {
            Component::CurDir => continue,
            Component::ParentDir
            | Component::Normal(_)
            | Component::RootDir
            | Component::Prefix(_) => current.push(component.as_os_str()),
        }
        if current.as_os_str().is_empty() {
            continue;
        }
        inspected_component_count += 1;
        let Some(target) = read_link_omena_resolver_style_identity_path(current.as_path()) else {
            continue;
        };
        let target_was_absolute = target.is_absolute();
        let target_path = if target_was_absolute {
            target
        } else {
            current
                .parent()
                .map(|parent| parent.join(target.as_path()))
                .unwrap_or_else(|| target.clone())
        };
        links.push(OmenaResolverSymlinkChainLinkV0 {
            link_path: normalize_style_path(current.clone()),
            target_path: normalize_style_path(target_path),
            target_was_absolute,
        });
    }

    OmenaResolverSymlinkChainInspectionV0 {
        schema_version: "0",
        product: "omena-resolver.symlink-chain-inspection",
        requested_path,
        inspected_component_count,
        link_count: links.len(),
        links,
    }
}

fn summarize_omena_resolver_symlink_chain_for_style_resolution(
    candidates: &[String],
    resolved_style_path: Option<&str>,
) -> OmenaResolverSymlinkChainInspectionV0 {
    if let Some(resolved_style_path) = resolved_style_path {
        for candidate in candidates {
            if !style_paths_share_identity(candidate, resolved_style_path) {
                continue;
            }
            let inspection = inspect_omena_resolver_symlink_chain_v0(candidate);
            if inspection.link_count > 0 {
                return inspection;
            }
        }
        let inspection = inspect_omena_resolver_symlink_chain_v0(resolved_style_path);
        if inspection.link_count > 0 {
            return inspection;
        }
    }
    for candidate in candidates {
        let inspection = inspect_omena_resolver_symlink_chain_v0(candidate);
        if inspection.link_count > 0 {
            return inspection;
        }
    }
    if let Some(resolved_style_path) = resolved_style_path {
        return inspect_omena_resolver_symlink_chain_v0(resolved_style_path);
    }
    inspect_omena_resolver_symlink_chain_v0(candidates.first().map(String::as_str).unwrap_or(""))
}

pub fn resolve_omena_resolver_style_module_candidate_from_available_paths(
    candidates: &[String],
    available_style_paths: &BTreeSet<&str>,
) -> Option<String> {
    confirm_omena_resolver_style_module_candidate_with_options(
        candidates,
        available_style_paths,
        &[],
        OmenaResolverStyleModuleConfirmationOptionsV0::default(),
    )
    .resolved_style_path
}

pub fn build_omena_resolver_style_module_confirmation_identity_index(
    available_style_paths: &BTreeSet<&str>,
    disk_style_path_identities: &[OmenaResolverStyleModuleDiskCandidateIdentityV0],
) -> OmenaResolverStyleModuleConfirmationIdentityIndexV0 {
    #[cfg(test)]
    {
        STYLE_IDENTITY_INDEX_BUILD_COUNT.fetch_add(1, Ordering::AcqRel);
        STYLE_IDENTITY_INDEX_BUILD_WORK_COUNT.fetch_add(
            available_style_paths
                .len()
                .saturating_add(disk_style_path_identities.len()),
            Ordering::AcqRel,
        );
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
    let disk_by_identity = disk_style_path_identities
        .iter()
        .map(|identity| {
            (
                canonicalize_omena_resolver_style_identity_path(&identity.style_path),
                identity.style_path.clone(),
            )
        })
        .collect::<BTreeMap<_, _>>();
    OmenaResolverStyleModuleConfirmationIdentityIndexV0 {
        available_by_identity,
        disk_by_identity,
    }
}

pub fn confirm_omena_resolver_style_module_candidate_with_options(
    candidates: &[String],
    available_style_paths: &BTreeSet<&str>,
    disk_style_path_identities: &[OmenaResolverStyleModuleDiskCandidateIdentityV0],
    options: OmenaResolverStyleModuleConfirmationOptionsV0<'_>,
) -> OmenaResolverStyleModuleCandidateConfirmationV0 {
    for candidate in candidates {
        if available_style_paths.contains(candidate.as_str()) {
            return OmenaResolverStyleModuleCandidateConfirmationV0 {
                resolved_style_path: Some(candidate.clone()),
                confirmation_kind: "inGraphExact",
                disk_candidate_count: disk_style_path_identities.len(),
                candidate_count: candidates.len(),
            };
        }
    }

    let owned_identity_index = options.identity_index.is_none().then(|| {
        build_omena_resolver_style_module_confirmation_identity_index(
            available_style_paths,
            disk_style_path_identities,
        )
    });
    let identity_index = match options.identity_index.or(owned_identity_index.as_ref()) {
        Some(identity_index) => identity_index,
        None => {
            return OmenaResolverStyleModuleCandidateConfirmationV0 {
                resolved_style_path: None,
                confirmation_kind: "unresolved",
                disk_candidate_count: disk_style_path_identities.len(),
                candidate_count: candidates.len(),
            };
        }
    };

    for candidate in candidates {
        if let Some(path) = identity_index
            .available_by_identity
            .get(canonicalize_omena_resolver_style_identity_path(candidate).as_str())
            .cloned()
        {
            return OmenaResolverStyleModuleCandidateConfirmationV0 {
                resolved_style_path: Some(path),
                confirmation_kind: "inGraphIdentity",
                disk_candidate_count: disk_style_path_identities.len(),
                candidate_count: candidates.len(),
            };
        }
    }

    if options.allow_disk_confirmation && !disk_style_path_identities.is_empty() {
        let candidate_limit = options.max_disk_candidate_count.max(1);
        for candidate in candidates.iter().take(candidate_limit) {
            if !is_omena_resolver_indexable_style_module_path(candidate) {
                continue;
            }
            if let Some(path) = identity_index
                .disk_by_identity
                .get(canonicalize_omena_resolver_style_identity_path(candidate).as_str())
                .cloned()
            {
                return OmenaResolverStyleModuleCandidateConfirmationV0 {
                    resolved_style_path: Some(path),
                    confirmation_kind: "diskIdentity",
                    disk_candidate_count: disk_style_path_identities.len(),
                    candidate_count: candidates.len(),
                };
            }
        }
    }

    if options.allow_live_disk_confirmation {
        let candidate_limit = options.max_disk_candidate_count.max(1);
        for candidate in candidates.iter().take(candidate_limit) {
            if !is_omena_resolver_indexable_style_module_path(candidate) {
                continue;
            }
            if Path::new(candidate).exists() {
                return OmenaResolverStyleModuleCandidateConfirmationV0 {
                    resolved_style_path: Some(normalize_style_path(PathBuf::from(candidate))),
                    confirmation_kind: "liveDisk",
                    disk_candidate_count: disk_style_path_identities.len(),
                    candidate_count: candidates.len(),
                };
            }
        }
    }

    if options.allow_unconfirmed_indexable_candidate
        && let Some(candidate) = candidates
            .iter()
            .find(|candidate| is_omena_resolver_indexable_style_module_path(candidate))
    {
        return OmenaResolverStyleModuleCandidateConfirmationV0 {
            resolved_style_path: Some(candidate.clone()),
            confirmation_kind: "unconfirmedIndexableCandidate",
            disk_candidate_count: disk_style_path_identities.len(),
            candidate_count: candidates.len(),
        };
    }

    OmenaResolverStyleModuleCandidateConfirmationV0 {
        resolved_style_path: None,
        confirmation_kind: "unresolved",
        disk_candidate_count: disk_style_path_identities.len(),
        candidate_count: candidates.len(),
    }
}

pub fn is_omena_resolver_indexable_style_module_path(path: &str) -> bool {
    path.ends_with(".module.css")
        || path.ends_with(".css")
        || path.ends_with(".module.scss")
        || path.ends_with(".scss")
        || path.ends_with(".module.sass")
        || path.ends_with(".sass")
        || path.ends_with(".module.less")
        || path.ends_with(".less")
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
