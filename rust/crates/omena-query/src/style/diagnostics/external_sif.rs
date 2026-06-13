use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use omena_parser::{ParsedSassModuleEdgeFact, ParsedSassModuleEdgeFactKind};
use omena_resolver::{
    canonicalize_omena_resolver_style_identity_path,
    collect_omena_resolver_style_module_source_candidates_with_load_path_roots,
};

use super::diagnostic_suppressions::OmenaStrictnessLevelV0;
use super::parser_facade::collect_omena_query_omena_parser_style_facts_raw;
use super::sass_symbols::{
    apply_sass_forward_prefix, fold_sass_symbol_name, sass_forward_filter_name_matches_symbol,
};
use super::substrate::OmenaQueryWorkspaceDiagnosticsSubstrateV0;
use super::*;

#[derive(Clone, Copy)]
pub(in crate::style) struct OmenaQueryExternalSifResolutionContext<'a> {
    pub(in crate::style) package_manifests: &'a [OmenaQueryStylePackageManifestV0],
    pub(in crate::style) bundler_path_mappings: &'a [OmenaResolverBundlerPathAliasMappingV0],
    pub(in crate::style) tsconfig_path_mappings: &'a [OmenaResolverTsconfigPathMappingV0],
    pub(in crate::style) external_sifs: &'a [OmenaQueryExternalSifInputV0],
}

#[allow(clippy::too_many_arguments)]
pub(super) fn collect_omena_query_external_top_any_sass_symbol_ranges(
    target_style_path: &str,
    style_sources: &[OmenaQueryStyleSourceInputV0],
    package_manifests: &[OmenaQueryStylePackageManifestV0],
    bundler_path_mappings: &[OmenaResolverBundlerPathAliasMappingV0],
    tsconfig_path_mappings: &[OmenaResolverTsconfigPathMappingV0],
    external_sifs: &[OmenaQueryExternalSifInputV0],
    substrate: &OmenaQueryWorkspaceDiagnosticsSubstrateV0,
) -> BTreeSet<ParserRangeV0> {
    let Some(target) = style_sources
        .iter()
        .find(|source| source.style_path == target_style_path)
    else {
        return BTreeSet::new();
    };
    // Substrate RES-D slot: SIF-promoted resolution over the same
    // (package_manifests, bundler, tsconfig, external_sifs) arguments.
    let resolution = &substrate.sass_resolution_with_external_sifs;
    let external_sif_context = OmenaQueryExternalSifResolutionContext {
        package_manifests,
        bundler_path_mappings,
        tsconfig_path_mappings,
        external_sifs,
    };
    let external_sources = resolution
        .edges
        .iter()
        .filter(|edge| edge.from_style_path == target_style_path)
        .filter(|edge| edge.status == "external")
        .map(|edge| edge.source.as_str())
        .collect::<BTreeSet<_>>();
    if external_sources.is_empty() {
        return BTreeSet::new();
    }

    let facts = collect_omena_query_omena_parser_style_facts_raw(
        target.style_source.as_str(),
        omena_parser_dialect_for_style_path(target_style_path),
    );
    // The protocol lattice is the single source of truth: a namespace is TopAny iff its
    // external edge classifies to a `top == TopAny` state (Missing/Partial/Stale). A
    // Resolved (TopOpaque) edge, backed by a complete SIF, stays subject to ordinary
    // missing-symbol checking.
    let top_any_namespaces = facts
        .sass_module_edges
        .iter()
        .filter(|edge| external_sources.contains(edge.source.as_str()))
        .filter(|edge| {
            let sif = resolution
                .edges
                .iter()
                .find(|resolution_edge| {
                    resolution_edge.from_style_path == target_style_path
                        && resolution_edge.source == edge.source
                        && resolution_edge.status == "external"
                })
                .and_then(|resolution_edge| {
                    find_omena_query_external_sif_for_edge(resolution_edge, external_sif_context)
                });
            classify_external_boundary_state(edge, sif, &facts, external_sifs).top
                == OmenaResolverBoundaryTopV0::TopAny
        })
        .filter_map(|edge| match edge.kind {
            ParsedSassModuleEdgeFactKind::Use
                if edge.namespace_kind == Some("default")
                    || edge.namespace_kind == Some("alias") =>
            {
                edge.namespace.clone().map(Some)
            }
            ParsedSassModuleEdgeFactKind::Use if edge.namespace_kind == Some("wildcard") => {
                Some(None)
            }
            ParsedSassModuleEdgeFactKind::Import => Some(None),
            _ => None,
        })
        .collect::<BTreeSet<_>>();
    if top_any_namespaces.is_empty() {
        return BTreeSet::new();
    }

    facts
        .sass_symbols
        .into_iter()
        .filter(|symbol| omena_query_sass_symbol_fact_kind_is_reference(symbol.kind))
        .filter(|symbol| top_any_namespaces.contains(&symbol.namespace))
        .map(|symbol| {
            let start: u32 = symbol.range.start().into();
            let end: u32 = symbol.range.end().into();
            parser_range_for_byte_span(
                target.style_source.as_str(),
                ParserByteSpanV0 {
                    start: start as usize,
                    end: end as usize,
                },
            )
        })
        .collect()
}

/// Classify a single external (`status == "external"`) Sass module edge onto the
/// resolver's five-state boundary lattice.
///
/// Four states are derivable from the local SIF set: missing, stale, partial, and
/// resolved. The unresolved state is classified by the caller because unresolved
/// edges have no SIF lattice to inspect.
fn classify_external_boundary_state(
    edge: &ParsedSassModuleEdgeFact,
    sif: Option<&OmenaQueryExternalSifInputV0>,
    target_facts: &omena_parser::ParsedStyleFacts,
    external_sifs: &[OmenaQueryExternalSifInputV0],
) -> OmenaResolverBoundaryStateV0 {
    let Some(sif) = sif else {
        return OmenaResolverBoundaryStateV0::missing(
            None,
            "SIF mode requires a local SIF artifact for this external Sass module",
        );
    };

    let canonical_url = OmenaResolverCanonicalUrlV0 {
        url: edge.source.clone(),
    };

    if let Some(dependency) = sif.sif.dependencies.iter().find(|dependency| {
        find_omena_query_external_sif(dependency.canonical_url.as_str(), external_sifs)
            .map(|dependency_sif| {
                dependency_sif.sif.fingerprints.interface_hash != dependency.interface_hash
            })
            .unwrap_or(false)
    }) {
        return OmenaResolverBoundaryStateV0::stale(
            canonical_url,
            format!(
                "external SIF dependency '{}' interface hash drifted from the lockfile-recorded hash",
                dependency.canonical_url
            ),
        );
    }

    let exported = collect_sif_exported_sass_symbol_keys(&sif.sif, external_sifs);
    let mut referenced = 0usize;
    let mut covered = 0usize;
    for symbol in &target_facts.sass_symbols {
        if !omena_query_sass_symbol_fact_kind_is_reference(symbol.kind) {
            continue;
        }
        if !sass_symbol_reference_belongs_to_edge(edge, symbol.namespace.as_deref()) {
            continue;
        }
        referenced += 1;
        if exported.contains(&(symbol.symbol_kind, fold_sass_symbol_name(&symbol.name))) {
            covered += 1;
        }
    }

    if referenced > 0 && covered < referenced {
        return OmenaResolverBoundaryStateV0::partial(format!(
            "external SIF for '{}' exports only {}/{} referenced symbol(s)",
            edge.source, covered, referenced
        ));
    }

    OmenaResolverBoundaryStateV0::resolved(canonical_url)
}

fn sass_symbol_reference_belongs_to_edge(
    edge: &ParsedSassModuleEdgeFact,
    reference_namespace: Option<&str>,
) -> bool {
    match edge.kind {
        ParsedSassModuleEdgeFactKind::Use
            if edge.namespace_kind == Some("default") || edge.namespace_kind == Some("alias") =>
        {
            edge.namespace.as_deref() == reference_namespace
        }
        ParsedSassModuleEdgeFactKind::Use if edge.namespace_kind == Some("wildcard") => {
            reference_namespace.is_none()
        }
        ParsedSassModuleEdgeFactKind::Import | ParsedSassModuleEdgeFactKind::Forward => {
            reference_namespace.is_none()
        }
        _ => false,
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn summarize_omena_query_external_sif_boundary_diagnostics(
    target_style_path: &str,
    style_sources: &[OmenaQueryStyleSourceInputV0],
    package_manifests: &[OmenaQueryStylePackageManifestV0],
    bundler_path_mappings: &[OmenaResolverBundlerPathAliasMappingV0],
    tsconfig_path_mappings: &[OmenaResolverTsconfigPathMappingV0],
    external_sifs: &[OmenaQueryExternalSifInputV0],
    strictness: OmenaStrictnessLevelV0,
    substrate: &OmenaQueryWorkspaceDiagnosticsSubstrateV0,
) -> Vec<OmenaQueryStyleDiagnosticV0> {
    let Some(target) = style_sources
        .iter()
        .find(|source| source.style_path == target_style_path)
    else {
        return Vec::new();
    };
    // Substrate RES-D slot: SIF-promoted resolution over the same
    // (package_manifests, bundler, tsconfig, external_sifs) arguments.
    let resolution = &substrate.sass_resolution_with_external_sifs;
    let external_sif_context = OmenaQueryExternalSifResolutionContext {
        package_manifests,
        bundler_path_mappings,
        tsconfig_path_mappings,
        external_sifs,
    };
    let external_sources = resolution
        .edges
        .iter()
        .filter(|edge| edge.from_style_path == target_style_path)
        .filter(|edge| edge.status == "external")
        .map(|edge| edge.source.as_str())
        .collect::<BTreeSet<_>>();
    let unresolved_sources = resolution
        .edges
        .iter()
        .filter(|edge| edge.from_style_path == target_style_path)
        .filter(|edge| edge.status == "unresolved")
        .filter(|edge| !sass_module_source_is_workspace_local(edge.source.as_str()))
        .map(|edge| edge.source.as_str())
        .collect::<BTreeSet<_>>();
    if external_sources.is_empty() && unresolved_sources.is_empty() {
        return Vec::new();
    }

    let facts = collect_omena_query_omena_parser_style_facts_raw(
        target.style_source.as_str(),
        omena_parser_dialect_for_style_path(target_style_path),
    );
    let mut emitted = BTreeSet::new();
    let mut diagnostics = Vec::new();
    for edge in &facts.sass_module_edges {
        let is_external = external_sources.contains(edge.source.as_str());
        let is_unresolved = unresolved_sources.contains(edge.source.as_str());
        if !is_external && !is_unresolved {
            continue;
        }
        if !emitted.insert((edge.kind, edge.source.clone())) {
            continue;
        }
        let state = if is_unresolved {
            omena_resolver_boundary_state_for_unresolved_reference_v0(edge.source.as_str())
        } else {
            let sif = resolution
                .edges
                .iter()
                .find(|resolution_edge| {
                    resolution_edge.from_style_path == target_style_path
                        && resolution_edge.source == edge.source
                        && resolution_edge.status == "external"
                })
                .and_then(|resolution_edge| {
                    find_omena_query_external_sif_for_edge(resolution_edge, external_sif_context)
                });
            classify_external_boundary_state(edge, sif, &facts, external_sifs)
        };
        let (code, default_severity) = match state.state {
            OmenaResolverBoundaryStateKindV0::Resolved => continue,
            OmenaResolverBoundaryStateKindV0::Stale => ("staleExternalSif", "warning"),
            OmenaResolverBoundaryStateKindV0::Partial => ("partialExternalSif", "information"),
            OmenaResolverBoundaryStateKindV0::Missing => ("missingExternalSif", "warning"),
            OmenaResolverBoundaryStateKindV0::Unresolved => {
                ("unresolvedExternalReference", "warning")
            }
        };
        let severity = strictness.boundary_severity(default_severity);
        let start: u32 = edge.range.start().into();
        let end: u32 = edge.range.end().into();
        diagnostics.push(OmenaQueryStyleDiagnosticV0 {
            code,
            severity,
            provenance: vec![
                "omena-resolver.boundary-state",
                "omena-query.external-sif-boundary-diagnostics",
            ],
            range: parser_range_for_byte_span(
                target.style_source.as_str(),
                ParserByteSpanV0 {
                    start: start as usize,
                    end: end as usize,
                },
            ),
            message: format!(
                "External Sass module '{}' is {} ({}); {}",
                edge.source,
                state.state_name,
                state.top_name,
                external_boundary_remediation_hint(state.state)
            ),
            tags: Vec::new(),
            create_custom_property: None,
            cascade_narrowing: None,
            cascade_confidence: None,
            polynomial_provenance: None,
            cross_file_scc: None,
        });
    }
    diagnostics
}

fn external_boundary_remediation_hint(state: OmenaResolverBoundaryStateKindV0) -> &'static str {
    match state {
        OmenaResolverBoundaryStateKindV0::Missing => {
            "generate or provide a SIF artifact, or use --external ignored."
        }
        OmenaResolverBoundaryStateKindV0::Stale => {
            "regenerate the SIF/lockfile so its dependency interface hashes match."
        }
        OmenaResolverBoundaryStateKindV0::Partial => {
            "some referenced symbols are absent from the SIF interface; regenerate the SIF or fix the reference."
        }
        OmenaResolverBoundaryStateKindV0::Unresolved => {
            "the resolver cannot canonicalize this reference; fix the specifier or add it to the workspace."
        }
        OmenaResolverBoundaryStateKindV0::Resolved => "",
    }
}

pub(super) fn summarize_sass_module_cross_file_resolution_with_external_sifs(
    style_fact_entries: &[OmenaQueryStyleFactEntry],
    package_manifests: &[OmenaQueryStylePackageManifestV0],
    bundler_path_mappings: &[OmenaResolverBundlerPathAliasMappingV0],
    tsconfig_path_mappings: &[OmenaResolverTsconfigPathMappingV0],
    external_sifs: &[OmenaQueryExternalSifInputV0],
) -> OmenaQuerySassModuleCrossFileResolutionV0 {
    let mut resolution = summarize_sass_module_cross_file_resolution(
        style_fact_entries,
        package_manifests,
        bundler_path_mappings,
        tsconfig_path_mappings,
    );
    promote_sif_backed_external_edges(
        &mut resolution,
        OmenaQueryExternalSifResolutionContext {
            package_manifests,
            bundler_path_mappings,
            tsconfig_path_mappings,
            external_sifs,
        },
    );
    resolution
}

pub(in crate::style) fn promote_sif_backed_external_edges(
    resolution: &mut OmenaQuerySassModuleCrossFileResolutionV0,
    external_sif_context: OmenaQueryExternalSifResolutionContext<'_>,
) {
    let external_sifs = external_sif_context.external_sifs;
    if external_sifs.is_empty() {
        return;
    }
    for edge in &mut resolution.edges {
        if edge.status == "unresolved"
            && !sass_module_source_is_workspace_local(edge.source.as_str())
            && find_omena_query_external_sif_for_edge(edge, external_sif_context).is_some()
        {
            edge.status = "external";
            edge.resolution_kind = "externalSifCanonicalUrl";
        }
    }
    resolution.resolved_module_edge_count = resolution
        .edges
        .iter()
        .filter(|edge| edge.status == "resolved")
        .count();
    resolution.external_module_edge_count = resolution
        .edges
        .iter()
        .filter(|edge| edge.status == "external")
        .count();
    resolution.unresolved_module_edge_count = resolution.module_edge_count.saturating_sub(
        resolution.resolved_module_edge_count + resolution.external_module_edge_count,
    );
}

pub(super) fn find_omena_query_external_sif_for_edge<'a>(
    edge: &OmenaQuerySassModuleEdgeResolutionV0,
    external_sif_context: OmenaQueryExternalSifResolutionContext<'a>,
) -> Option<&'a OmenaQueryExternalSifInputV0> {
    let external_sifs = external_sif_context.external_sifs;
    if let Some(sif) = find_omena_query_external_sif(edge.source.as_str(), external_sifs) {
        return Some(sif);
    }

    let resolver_package_manifests = external_sif_context
        .package_manifests
        .iter()
        .map(|manifest| OmenaResolverStylePackageManifestV0 {
            package_json_path: manifest.package_json_path.clone(),
            package_json_source: manifest.package_json_source.clone(),
        })
        .collect::<Vec<_>>();
    collect_omena_resolver_style_module_source_candidates_with_load_path_roots(
        edge.from_style_path.as_str(),
        edge.source.as_str(),
        resolver_package_manifests.as_slice(),
        external_sif_context.bundler_path_mappings,
        external_sif_context.tsconfig_path_mappings,
        &[],
    )
    .into_iter()
    .find_map(|candidate| find_omena_query_external_sif(candidate.as_str(), external_sifs))
}

pub(super) fn collect_sif_exported_sass_symbol_keys(
    sif: &omena_sif::OmenaSifV1,
    external_sifs: &[OmenaQueryExternalSifInputV0],
) -> BTreeSet<(&'static str, String)> {
    let mut visiting = BTreeSet::new();
    collect_sif_exported_sass_symbol_keys_inner(sif, external_sifs, &mut visiting)
}

fn collect_sif_exported_sass_symbol_keys_inner(
    sif: &omena_sif::OmenaSifV1,
    external_sifs: &[OmenaQueryExternalSifInputV0],
    visiting: &mut BTreeSet<String>,
) -> BTreeSet<(&'static str, String)> {
    if !visiting.insert(sif.canonical_url.clone()) {
        return BTreeSet::new();
    }
    let mut exported = BTreeSet::new();
    exported.extend(sif.exports.variables.iter().map(|variable| {
        (
            "variable",
            variable.name.trim_start_matches('$').to_string(),
        )
    }));
    exported.extend(
        sif.exports
            .mixins
            .iter()
            .map(|mixin| ("mixin", mixin.name.clone())),
    );
    exported.extend(
        sif.exports
            .functions
            .iter()
            .map(|function| ("function", function.name.clone())),
    );
    exported.extend(sif.exports.placeholders.iter().map(|placeholder| {
        (
            "placeholder",
            placeholder.name.trim_start_matches('%').to_string(),
        )
    }));
    for forward in &sif.exports.forwards {
        let Some(forwarded_sif) =
            find_omena_query_external_sif_for_forward(sif, forward, external_sifs)
        else {
            continue;
        };
        let forwarded_exports = collect_sif_exported_sass_symbol_keys_inner(
            &forwarded_sif.sif,
            external_sifs,
            visiting,
        );
        for (symbol_kind, name) in forwarded_exports {
            if !sif_forward_visibility_allows(forward, symbol_kind, name.as_str()) {
                continue;
            }
            exported.insert((
                symbol_kind,
                apply_sass_forward_prefix(forward.prefix.as_deref(), name.as_str()),
            ));
        }
    }
    visiting.remove(sif.canonical_url.as_str());
    exported
}

fn find_omena_query_external_sif_for_forward<'a>(
    sif: &omena_sif::OmenaSifV1,
    forward: &omena_sif::OmenaSifForwardExportV1,
    external_sifs: &'a [OmenaQueryExternalSifInputV0],
) -> Option<&'a OmenaQueryExternalSifInputV0> {
    let candidates = collect_sif_forward_canonical_url_candidates(
        sif.canonical_url.as_str(),
        forward.canonical_url.as_str(),
    );
    candidates
        .iter()
        .find_map(|candidate| find_omena_query_external_sif(candidate.as_str(), external_sifs))
}

fn collect_sif_forward_canonical_url_candidates(
    base_canonical_url: &str,
    source: &str,
) -> Vec<String> {
    let mut candidates = BTreeSet::new();
    candidates.insert(source.to_string());
    if let Some(base_file_path) = base_canonical_url.strip_prefix("file://") {
        push_file_uri_forward_candidates(&mut candidates, base_file_path, source);
    }
    candidates.into_iter().collect()
}

fn push_file_uri_forward_candidates(
    candidates: &mut BTreeSet<String>,
    base_file_path: &str,
    source: &str,
) {
    if source.starts_with("sass:")
        || source.starts_with("http://")
        || source.starts_with("https://")
        || source.starts_with("file://")
        || source.starts_with("pkg:")
    {
        return;
    }
    let base_path = Path::new(base_file_path);
    let joined = if source.starts_with('/') {
        PathBuf::from(source)
    } else {
        base_path
            .parent()
            .unwrap_or_else(|| Path::new(""))
            .join(source)
    };
    push_file_uri_style_path_candidates(candidates, joined.as_path());
}

fn push_file_uri_style_path_candidates(candidates: &mut BTreeSet<String>, path: &Path) {
    let has_extension = path
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some();
    if has_extension {
        push_file_uri_candidate(candidates, path);
        return;
    }
    for extension in ["scss", "sass", "css"] {
        let with_extension = path.with_extension(extension);
        push_file_uri_candidate(candidates, with_extension.as_path());
        if let Some(file_name) = path.file_name().and_then(|file_name| file_name.to_str()) {
            let partial = path
                .with_file_name(format!("_{file_name}"))
                .with_extension(extension);
            push_file_uri_candidate(candidates, partial.as_path());
        }
    }
}

fn push_file_uri_candidate(candidates: &mut BTreeSet<String>, path: &Path) {
    candidates.insert(format!("file://{}", normalize_sif_file_uri_path(path)));
}

fn normalize_sif_file_uri_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn find_omena_query_external_sif<'a>(
    canonical_url: &str,
    external_sifs: &'a [OmenaQueryExternalSifInputV0],
) -> Option<&'a OmenaQueryExternalSifInputV0> {
    external_sifs.iter().find(|input| {
        omena_query_external_sif_canonical_urls_match(input.canonical_url.as_str(), canonical_url)
            || omena_query_external_sif_canonical_urls_match(
                input.sif.canonical_url.as_str(),
                canonical_url,
            )
    })
}

#[cfg(test)]
pub(super) fn omena_query_external_sif_canonical_urls_match(left: &str, right: &str) -> bool {
    omena_query_external_sif_canonical_urls_match_inner(left, right)
}

#[cfg(not(test))]
fn omena_query_external_sif_canonical_urls_match(left: &str, right: &str) -> bool {
    omena_query_external_sif_canonical_urls_match_inner(left, right)
}

fn omena_query_external_sif_canonical_urls_match_inner(left: &str, right: &str) -> bool {
    if left == right {
        return true;
    }
    let Some(left_path) = omena_query_external_sif_canonical_url_path(left) else {
        return false;
    };
    let Some(right_path) = omena_query_external_sif_canonical_url_path(right) else {
        return false;
    };
    canonicalize_omena_resolver_style_identity_path(left_path.as_str())
        == canonicalize_omena_resolver_style_identity_path(right_path.as_str())
}

fn omena_query_external_sif_canonical_url_path(canonical_url: &str) -> Option<String> {
    if let Some(path) = canonical_url.strip_prefix("file://") {
        return Some(path.to_string());
    }
    Path::new(canonical_url)
        .is_absolute()
        .then(|| canonical_url.to_string())
}

fn sif_forward_visibility_allows(
    forward: &omena_sif::OmenaSifForwardExportV1,
    symbol_kind: &'static str,
    name: &str,
) -> bool {
    let matches_filter = |filter_name: &String| {
        sass_forward_filter_name_matches_symbol(
            filter_name,
            forward.prefix.as_deref(),
            symbol_kind,
            name,
        )
    };
    if !forward.show.is_empty() {
        return forward.show.iter().any(matches_filter);
    }
    !forward.hide.iter().any(matches_filter)
}
