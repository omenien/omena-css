use super::external_sif::{
    OmenaQueryExternalSifResolutionContext, promote_sif_backed_external_edges,
};
use super::shared::*;

/// RFC 0009 Pillar B stage-2 (#65): the per-call workspace diagnostics substrate.
///
/// Each workspace sub-pass used to rebuild the style fact entries (one full parse per
/// in-graph style source) and its own Sass cross-file resolution from the same corpus on
/// every monolith call. The substrate hoists those rebuilds to ONE collection per call.
///
/// The resolution variants are intentionally NOT collapsed into one slot: the sub-passes
/// call `summarize_sass_module_cross_file_resolution` with *different* argument shapes
/// (empty manifests, empty path mappings, SIF promotion), and substituting one variant
/// for another would change reachability / unresolved-edge sets — package manifests
/// enable bare-package and pkg-export edges, path mappings enable alias edges. Every
/// distinct `(entries, resolution-arguments)` value the monolith's sub-passes actually
/// compute is carried as its own slot, byte-identical to what the sub-pass would have
/// computed itself. The slots are built from the monolith's separate `package_manifests`
/// parameter (NOT `resolution_inputs.package_manifests`, which no sub-pass reads).
#[derive(Clone, PartialEq, Eq)]
pub(in crate::style) struct OmenaQueryWorkspaceDiagnosticsSubstrateV0 {
    /// ENTRIES: `collect_omena_query_style_fact_entries` over ALL
    /// `(style_path, style_source)` pairs in input order. Order-sensitive: the
    /// resolution derives per-kind `rule_ordinal`s from iteration order, so the corpus
    /// is never sorted/deduped/keyed here.
    pub(in crate::style) style_fact_entries: Vec<OmenaQueryStyleFactEntry>,
    /// RES-A: plain resolution with `(package_manifests, bundler, tsconfig)`.
    /// Consumers: sass-use-cycle, resolution-identity, replica-ensemble, module-graph
    /// property-value narrowing.
    pub(in crate::style) sass_resolution: OmenaQuerySassModuleCrossFileResolutionV0,
    /// RES-B: plain resolution with EMPTY manifests + `(bundler, tsconfig)`. Sole
    /// consumer: missing-extend-target. Equals RES-A only when `package_manifests` is
    /// empty, so it stays a separate slot.
    pub(in crate::style) sass_resolution_without_manifests:
        OmenaQuerySassModuleCrossFileResolutionV0,
    /// RES-C: plain resolution with `(package_manifests)` + EMPTY path mappings.
    /// Consumers: unresolved-sass-import (always) and the unified-SCC pass's Sass leg
    /// (`hypergraph-ifds` builds only). Equals RES-A only when both mapping vecs are
    /// empty, so it stays a separate slot.
    pub(in crate::style) sass_resolution_without_path_mappings:
        OmenaQuerySassModuleCrossFileResolutionV0,
    /// RES-D: SIF-promoted resolution (== RES-A + `promote_sif_backed_external_edges`;
    /// byte-identical to RES-A when `external_sifs` is empty since the promotion
    /// early-returns). Consumers: missing-sass-symbol (unconditional, even in `Ignored`
    /// mode), external top-any ranges + SIF boundary (Sif mode).
    pub(in crate::style) sass_resolution_with_external_sifs:
        OmenaQuerySassModuleCrossFileResolutionV0,
    /// RES-E (`hypergraph-ifds` only): css-modules resolution with
    /// `(package_manifests)`, consumed by the unified-SCC pass via the workspace
    /// cross-file summary. Default builds run the empty SCC stub and never compute it.
    #[cfg(feature = "hypergraph-ifds")]
    pub(in crate::style) css_modules_resolution: OmenaQueryCssModulesCrossFileResolutionV0,
}

pub(in crate::style) fn collect_omena_query_workspace_diagnostics_substrate(
    style_sources: &[OmenaQueryStyleSourceInputV0],
    package_manifests: &[OmenaQueryStylePackageManifestV0],
    external_sifs: &[OmenaQueryExternalSifInputV0],
    bundler_path_mappings: &[OmenaResolverBundlerPathAliasMappingV0],
    tsconfig_path_mappings: &[OmenaResolverTsconfigPathMappingV0],
) -> OmenaQueryWorkspaceDiagnosticsSubstrateV0 {
    let style_source_refs = style_sources
        .iter()
        .map(|source| (source.style_path.as_str(), source.style_source.as_str()))
        .collect::<Vec<_>>();
    let style_fact_entries = collect_omena_query_style_fact_entries(style_source_refs.as_slice());
    collect_omena_query_workspace_diagnostics_substrate_from_entries(
        style_fact_entries,
        package_manifests,
        external_sifs,
        bundler_path_mappings,
        tsconfig_path_mappings,
    )
}

pub(in crate::style) fn collect_omena_query_workspace_diagnostics_substrate_from_entries(
    style_fact_entries: Vec<OmenaQueryStyleFactEntry>,
    package_manifests: &[OmenaQueryStylePackageManifestV0],
    external_sifs: &[OmenaQueryExternalSifInputV0],
    bundler_path_mappings: &[OmenaResolverBundlerPathAliasMappingV0],
    tsconfig_path_mappings: &[OmenaResolverTsconfigPathMappingV0],
) -> OmenaQueryWorkspaceDiagnosticsSubstrateV0 {
    let sass_resolution = summarize_sass_module_cross_file_resolution(
        &style_fact_entries,
        package_manifests,
        bundler_path_mappings,
        tsconfig_path_mappings,
    );
    // RES-D is what `summarize_sass_module_cross_file_resolution_with_external_sifs`
    // returns for the same arguments — and that function is exactly "plain resolution
    // (RES-A's arguments) + in-place `promote_sif_backed_external_edges`" — so it is
    // derived from a clone of RES-A instead of re-running the plain resolution again.
    let mut sass_resolution_with_external_sifs = sass_resolution.clone();
    promote_sif_backed_external_edges(
        &mut sass_resolution_with_external_sifs,
        OmenaQueryExternalSifResolutionContext {
            package_manifests,
            bundler_path_mappings,
            tsconfig_path_mappings,
            external_sifs,
        },
    );
    let sass_resolution_without_manifests = summarize_sass_module_cross_file_resolution(
        &style_fact_entries,
        &[],
        bundler_path_mappings,
        tsconfig_path_mappings,
    );
    let sass_resolution_without_path_mappings = summarize_sass_module_cross_file_resolution(
        &style_fact_entries,
        package_manifests,
        &[],
        &[],
    );
    #[cfg(feature = "hypergraph-ifds")]
    let css_modules_resolution =
        summarize_css_modules_cross_file_resolution(&style_fact_entries, package_manifests);
    OmenaQueryWorkspaceDiagnosticsSubstrateV0 {
        style_fact_entries,
        sass_resolution,
        sass_resolution_without_manifests,
        sass_resolution_without_path_mappings,
        sass_resolution_with_external_sifs,
        #[cfg(feature = "hypergraph-ifds")]
        css_modules_resolution,
    }
}

pub(in crate::style) fn collect_sass_module_graph_reachable_style_paths<'a>(
    target_style_path: &'a str,
    resolution: &'a OmenaQuerySassModuleCrossFileResolutionV0,
) -> BTreeSet<&'a str> {
    let mut reachable = BTreeSet::new();
    let mut stack = vec![target_style_path];
    while let Some(current) = stack.pop() {
        if !reachable.insert(current) {
            continue;
        }
        for edge in resolution
            .edges
            .iter()
            .filter(|edge| edge.from_style_path == current && edge.status == "resolved")
        {
            if let Some(next) = edge.resolved_style_path.as_deref() {
                stack.push(next);
            }
        }
    }
    reachable
}
