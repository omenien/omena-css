#[cfg(test)]
use std::collections::BTreeSet;

use super::diagnostic_suppressions::apply_omena_query_style_diagnostic_suppressions;
use super::diagnostic_suppressions::parse_omena_query_style_strictness_level;
use super::diagnostic_suppressions::report_omena_query_style_diagnostic_suppressions;
use super::*;

mod cascade_runtime;
mod cross_file_scc;
mod css_modules;
mod external_sif;
mod render;
mod replica_ensemble;
mod sass;
mod sass_builtins;
mod sass_resolution;
mod sass_symbols;
mod shared;
mod single_file;
mod source_usage;
mod substrate;
mod types;

use cascade_runtime::{
    attach_omena_query_module_graph_property_value_narrowing_for_workspace,
    attach_omena_query_runtime_state_inline_overrides_for_workspace,
};
use cross_file_scc::summarize_omena_query_unified_cross_file_scc_diagnostics_for_workspace;
use css_modules::summarize_omena_query_css_modules_resolution_style_diagnostics_from_entries;
pub use css_modules::{
    summarize_omena_query_css_modules_local_composes_style_diagnostics,
    summarize_omena_query_css_modules_resolution_style_diagnostics,
};
#[cfg(test)]
use external_sif::omena_query_external_sif_canonical_urls_match;
pub(super) use external_sif::{
    OmenaQueryExternalSifResolutionContext, promote_sif_backed_external_edges,
};
use external_sif::{
    collect_omena_query_external_top_any_sass_symbol_ranges,
    summarize_omena_query_external_sif_boundary_diagnostics,
    target_has_auto_external_boundary_edges,
};
use replica_ensemble::summarize_omena_query_replica_ensemble_inconsistency_diagnostics_for_workspace;
#[cfg(test)]
use sass::sass_import_is_plain_css;
pub(super) use sass::sass_module_source_is_workspace_local;
pub use sass::{
    summarize_omena_query_missing_extend_target_diagnostics,
    summarize_omena_query_missing_sass_symbol_diagnostics,
    summarize_omena_query_missing_sass_symbol_diagnostics_for_workspace,
    summarize_omena_query_sass_import_deprecation_hints,
    summarize_omena_query_target_unresolved_sass_import_diagnostics_for_workspace_paths,
};
use sass::{
    summarize_omena_query_missing_extend_target_diagnostics_for_workspace,
    summarize_omena_query_missing_sass_symbol_diagnostics_for_workspace_with_sifs_from_substrate,
    summarize_omena_query_sass_use_cycle_diagnostics_for_workspace,
    summarize_omena_query_unresolved_sass_import_diagnostics_for_workspace,
};
#[cfg(test)]
use sass_builtins::{sass_builtin_module_function_names, sass_builtin_module_mixin_names};
pub use sass_resolution::summarize_omena_query_sass_module_resolution_identity_diagnostics_for_workspace;
use sass_resolution::summarize_omena_query_sass_module_resolution_identity_diagnostics_for_workspace_from_resolution;
pub(crate) use sass_symbols::SassSymbolKey;
#[cfg(test)]
pub(crate) use sass_symbols::collect_omena_query_visible_sass_symbol_keys_for_workspace_file;
pub(super) use sass_symbols::collect_visible_sass_symbol_keys;
pub use single_file::{
    summarize_omena_query_cascade_aware_style_diagnostics,
    summarize_omena_query_cascade_aware_style_diagnostics_with_deep_analysis,
    summarize_omena_query_missing_custom_property_diagnostics,
    summarize_omena_query_missing_keyframes_diagnostics,
};
use source_usage::summarize_omena_query_unused_selector_style_diagnostics_with_path_mappings_from_entries;
pub use source_usage::{
    summarize_omena_query_unused_selector_style_diagnostics,
    summarize_omena_query_unused_selector_style_diagnostics_with_path_mappings,
};
use substrate::collect_omena_query_workspace_diagnostics_substrate;
pub(super) use substrate::collect_sass_module_graph_reachable_style_paths;
pub use types::OmenaQueryExternalModuleModeV0;

pub fn summarize_omena_query_style_diagnostics_for_file(
    style_uri: &str,
    source: &str,
    candidates: &[OmenaQueryStyleHoverCandidateV0],
) -> OmenaQueryStyleDiagnosticsForFileV0 {
    summarize_omena_query_style_diagnostics_for_file_with_deep_analysis(
        style_uri, source, candidates, false,
    )
}

/// File-level diagnostics summary with an explicit opt-in deep-analysis switch.
/// `deep_analysis == false` (the default LSP/CLI surface) keeps only the product
/// cascade diagnostics; `deep_analysis == true` surfaces the rg-flow / categorical
/// theory hints, deduplicated against `circularVar`.
pub fn summarize_omena_query_style_diagnostics_for_file_with_deep_analysis(
    style_uri: &str,
    source: &str,
    candidates: &[OmenaQueryStyleHoverCandidateV0],
    deep_analysis: bool,
) -> OmenaQueryStyleDiagnosticsForFileV0 {
    let mut diagnostics =
        summarize_omena_query_missing_custom_property_diagnostics(style_uri, source, candidates);
    diagnostics.extend(
        summarize_omena_query_cascade_aware_style_diagnostics_with_deep_analysis(
            style_uri,
            source,
            candidates,
            deep_analysis,
        ),
    );
    diagnostics.extend(summarize_omena_query_missing_keyframes_diagnostics(
        style_uri, source,
    ));
    diagnostics.extend(summarize_omena_query_sass_import_deprecation_hints(
        style_uri, source,
    ));
    diagnostics.extend(summarize_omena_query_missing_sass_symbol_diagnostics(
        style_uri, source,
    ));
    diagnostics.extend(summarize_omena_query_missing_extend_target_diagnostics(
        style_uri, source,
    ));
    apply_omena_query_checker_product_gate_to_style_diagnostics(&mut diagnostics);
    let mut summary = OmenaQueryStyleDiagnosticsForFileV0 {
        schema_version: "0",
        product: "omena-query.diagnostics-for-file",
        file_uri: style_uri.to_string(),
        file_kind: "style",
        diagnostic_count: diagnostics.len(),
        diagnostics,
        ready_surfaces: vec![
            "missingCustomPropertyDiagnostics",
            "cascadeAwareDiagnostics",
            "missingKeyframesDiagnostics",
            "sassImportDeprecationHints",
            "missingSassSymbolDiagnostics",
            "missingExtendTargetDiagnostics",
            "checkerProductDiagnosticGate",
            "runtimeStateScenarioEvidence",
        ],
        suppression_summary: None,
    };
    apply_omena_query_style_diagnostic_suppressions(source, &mut summary);
    summary
}

/// RFC-0007-F (#46): single-file `style-diagnostics` (no `--source`) used to skip composes-target
/// validation entirely, so a bare invocation and one with any unrelated `--source` produced
/// different diagnostics for the same file. This variant augments the single-file summary with the
/// composes outcomes that are fully resolvable without cross-file context — only `composes: x`
/// (Local edges) against the file's own selectors. Global edges produce nothing and External edges
/// (`composes: x from './other'`) are deliberately left to the `--source`-backed workspace path, so
/// no false `missingComposedSelector`/`missingComposedModule` is invented for an unseen sibling.
pub fn summarize_omena_query_style_diagnostics_for_file_with_local_composes(
    style_uri: &str,
    source: &str,
    candidates: &[OmenaQueryStyleHoverCandidateV0],
) -> OmenaQueryStyleDiagnosticsForFileV0 {
    summarize_omena_query_style_diagnostics_for_file_with_local_composes_and_deep_analysis(
        style_uri, source, candidates, false,
    )
}

/// Single-file (local composes) diagnostics summary with an explicit opt-in
/// deep-analysis switch. `deep_analysis == false` (the default surface) keeps only
/// the product cascade diagnostics; `deep_analysis == true` surfaces the rg-flow /
/// categorical theory hints, deduplicated against `circularVar`.
pub fn summarize_omena_query_style_diagnostics_for_file_with_local_composes_and_deep_analysis(
    style_uri: &str,
    source: &str,
    candidates: &[OmenaQueryStyleHoverCandidateV0],
    deep_analysis: bool,
) -> OmenaQueryStyleDiagnosticsForFileV0 {
    let mut summary = summarize_omena_query_style_diagnostics_for_file_with_deep_analysis(
        style_uri,
        source,
        candidates,
        deep_analysis,
    );
    let mut local_composes =
        summarize_omena_query_css_modules_local_composes_style_diagnostics(style_uri, source);
    apply_omena_query_checker_product_gate_to_style_diagnostics(&mut local_composes);
    if !local_composes.is_empty() {
        summary.diagnostics.extend(local_composes);
        push_omena_query_ready_surface(
            &mut summary.ready_surfaces,
            "cssModulesComposesResolutionDiagnostics",
        );
        // Re-run suppressions so the appended composes diagnostics honour the same inline directives.
        apply_omena_query_style_diagnostic_suppressions(source, &mut summary);
        summary.diagnostic_count = summary.diagnostics.len();
    }
    summary
}

pub fn summarize_omena_query_style_diagnostics_for_workspace_file(
    target_style_path: &str,
    style_sources: &[OmenaQueryStyleSourceInputV0],
    source_documents: &[OmenaQuerySourceDocumentInputV0],
    package_manifests: &[OmenaQueryStylePackageManifestV0],
    classname_transform: Option<&str>,
) -> Option<OmenaQueryStyleDiagnosticsForFileV0> {
    summarize_omena_query_style_diagnostics_for_workspace_file_with_external_mode(
        target_style_path,
        style_sources,
        source_documents,
        package_manifests,
        classname_transform,
        OmenaQueryExternalModuleModeV0::Ignored,
    )
}

pub fn summarize_omena_query_style_diagnostics_for_workspace_file_with_external_mode(
    target_style_path: &str,
    style_sources: &[OmenaQueryStyleSourceInputV0],
    source_documents: &[OmenaQuerySourceDocumentInputV0],
    package_manifests: &[OmenaQueryStylePackageManifestV0],
    classname_transform: Option<&str>,
    external_mode: OmenaQueryExternalModuleModeV0,
) -> Option<OmenaQueryStyleDiagnosticsForFileV0> {
    summarize_omena_query_style_diagnostics_for_workspace_file_with_external_mode_and_sifs(
        target_style_path,
        style_sources,
        source_documents,
        package_manifests,
        classname_transform,
        external_mode,
        &[],
    )
}

pub fn summarize_omena_query_style_diagnostics_for_workspace_file_with_external_mode_and_sifs(
    target_style_path: &str,
    style_sources: &[OmenaQueryStyleSourceInputV0],
    source_documents: &[OmenaQuerySourceDocumentInputV0],
    package_manifests: &[OmenaQueryStylePackageManifestV0],
    classname_transform: Option<&str>,
    external_mode: OmenaQueryExternalModuleModeV0,
    external_sifs: &[OmenaQueryExternalSifInputV0],
) -> Option<OmenaQueryStyleDiagnosticsForFileV0> {
    summarize_omena_query_style_diagnostics_for_workspace_file_with_external_mode_and_sifs_and_suppression_mode(
        target_style_path,
        style_sources,
        source_documents,
        package_manifests,
        classname_transform,
        external_mode,
        external_sifs,
        OmenaQueryDiagnosticSuppressionModeV0::Apply,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn summarize_omena_query_style_diagnostics_for_workspace_file_with_external_mode_and_sifs_and_suppression_mode(
    target_style_path: &str,
    style_sources: &[OmenaQueryStyleSourceInputV0],
    source_documents: &[OmenaQuerySourceDocumentInputV0],
    package_manifests: &[OmenaQueryStylePackageManifestV0],
    classname_transform: Option<&str>,
    external_mode: OmenaQueryExternalModuleModeV0,
    external_sifs: &[OmenaQueryExternalSifInputV0],
    suppression_mode: OmenaQueryDiagnosticSuppressionModeV0,
) -> Option<OmenaQueryStyleDiagnosticsForFileV0> {
    summarize_omena_query_style_diagnostics_for_workspace_file_with_external_mode_and_sifs_and_resolution_inputs_and_suppression_mode(
        target_style_path,
        style_sources,
        source_documents,
        package_manifests,
        classname_transform,
        external_mode,
        external_sifs,
        &OmenaQueryStyleResolutionInputsV0 {
            package_manifests: package_manifests.to_vec(),
            ..Default::default()
        },
        suppression_mode,
    )
}

/// Workspace-file style diagnostics variant that additionally carries the workspace's
/// tsconfig/bundler path mappings. RFC-0007-J (#50): the unused-selector usage collector resolves
/// source-document style imports through these mappings so an alias import (`@/styles/a.module.scss`)
/// is attributed to its real module — matching the reference/goto path — instead of leaving every
/// selector dimmed `unusedSelector`. Path mappings only affect alias resolution; with empty mappings
/// the behaviour is byte-for-byte the no-mappings entry above.
#[allow(clippy::too_many_arguments)]
pub fn summarize_omena_query_style_diagnostics_for_workspace_file_with_external_mode_and_sifs_and_resolution_inputs(
    target_style_path: &str,
    style_sources: &[OmenaQueryStyleSourceInputV0],
    source_documents: &[OmenaQuerySourceDocumentInputV0],
    package_manifests: &[OmenaQueryStylePackageManifestV0],
    classname_transform: Option<&str>,
    external_mode: OmenaQueryExternalModuleModeV0,
    external_sifs: &[OmenaQueryExternalSifInputV0],
    resolution_inputs: &OmenaQueryStyleResolutionInputsV0,
) -> Option<OmenaQueryStyleDiagnosticsForFileV0> {
    summarize_omena_query_style_diagnostics_for_workspace_file_with_external_mode_and_sifs_and_resolution_inputs_and_suppression_mode(
        target_style_path,
        style_sources,
        source_documents,
        package_manifests,
        classname_transform,
        external_mode,
        external_sifs,
        resolution_inputs,
        OmenaQueryDiagnosticSuppressionModeV0::Apply,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn summarize_omena_query_style_diagnostics_for_workspace_file_with_external_mode_and_sifs_and_resolution_inputs_and_suppression_mode(
    target_style_path: &str,
    style_sources: &[OmenaQueryStyleSourceInputV0],
    source_documents: &[OmenaQuerySourceDocumentInputV0],
    package_manifests: &[OmenaQueryStylePackageManifestV0],
    classname_transform: Option<&str>,
    external_mode: OmenaQueryExternalModuleModeV0,
    external_sifs: &[OmenaQueryExternalSifInputV0],
    resolution_inputs: &OmenaQueryStyleResolutionInputsV0,
    suppression_mode: OmenaQueryDiagnosticSuppressionModeV0,
) -> Option<OmenaQueryStyleDiagnosticsForFileV0> {
    let target = style_sources
        .iter()
        .find(|source| source.style_path == target_style_path)?;
    let candidates =
        summarize_omena_query_style_hover_candidates(target_style_path, &target.style_source)?;
    let mut summary = summarize_omena_query_style_diagnostics_for_file(
        target_style_path,
        &target.style_source,
        candidates.candidates.as_slice(),
    );
    summary
        .diagnostics
        .retain(|diagnostic| diagnostic.code != "missingSassSymbol");
    // RFC-0007-E1 (#45): the file-local `missingExtendTarget` rule cannot see a placeholder/class
    // declared in another in-graph file reachable via `@use`/`@forward`/`@import`, so it would
    // false-positive on a cross-file `@extend`. In workspace mode we drop the file-local result and
    // re-emit only those whose target is also absent from EVERY other in-graph style source — a
    // conservative cross-file-aware pass that keeps a genuinely-missing target flagged while never
    // inventing a false positive for a target defined in an imported partial. (Single-file surface
    // keeps the file-local rule unchanged.)
    summary
        .diagnostics
        .retain(|diagnostic| diagnostic.code != "missingExtendTarget");
    // RFC 0009 Pillar B stage-2 (#65): one shared substrate per monolith call. Every
    // workspace sub-pass below receives the precomputed (entries, resolution-variant)
    // slot it would otherwise have rebuilt from the same corpus itself.
    let substrate = collect_omena_query_workspace_diagnostics_substrate(
        style_sources,
        package_manifests,
        external_sifs,
        resolution_inputs.bundler_path_mappings.as_slice(),
        resolution_inputs.tsconfig_path_mappings.as_slice(),
    );
    summary.diagnostics.extend(
        summarize_omena_query_missing_extend_target_diagnostics_for_workspace(
            target_style_path,
            style_sources,
            &substrate,
        ),
    );
    summary.diagnostics.extend(
        summarize_omena_query_missing_sass_symbol_diagnostics_for_workspace_with_sifs_from_substrate(
            target_style_path,
            style_sources,
            package_manifests,
            external_sifs,
            resolution_inputs.bundler_path_mappings.as_slice(),
            resolution_inputs.tsconfig_path_mappings.as_slice(),
            &substrate.style_fact_entries,
            &substrate.sass_resolution_with_external_sifs,
        ),
    );
    summary.diagnostics.extend(
        summarize_omena_query_css_modules_resolution_style_diagnostics_from_entries(
            target_style_path,
            &target.style_source,
            &substrate.style_fact_entries,
            package_manifests,
            resolution_inputs,
        ),
    );
    summary.diagnostics.extend(
        summarize_omena_query_sass_use_cycle_diagnostics_for_workspace(
            target_style_path,
            style_sources,
            &substrate,
        ),
    );
    summary.diagnostics.extend(
        summarize_omena_query_unified_cross_file_scc_diagnostics_for_workspace(
            target_style_path,
            &target.style_source,
            style_sources,
            source_documents,
            package_manifests,
            &substrate,
        ),
    );
    summary.diagnostics.extend(
        summarize_omena_query_unresolved_sass_import_diagnostics_for_workspace(
            target_style_path,
            style_sources,
            &substrate,
        ),
    );
    summary.diagnostics.extend(
        summarize_omena_query_sass_module_resolution_identity_diagnostics_for_workspace_from_resolution(
            target_style_path,
            style_sources,
            &substrate.sass_resolution,
        ),
    );
    summary.diagnostics.extend(
        summarize_omena_query_unused_selector_style_diagnostics_with_path_mappings_from_entries(
            target_style_path,
            &target.style_source,
            &substrate.style_fact_entries,
            source_documents,
            package_manifests,
            classname_transform,
            resolution_inputs.bundler_path_mappings.as_slice(),
            resolution_inputs.tsconfig_path_mappings.as_slice(),
            resolution_inputs.disk_style_path_identities.as_slice(),
        ),
    );
    summary.diagnostics.extend(
        summarize_omena_query_replica_ensemble_inconsistency_diagnostics_for_workspace(
            target_style_path,
            style_sources,
            &substrate,
        ),
    );
    attach_omena_query_module_graph_property_value_narrowing_for_workspace(
        target_style_path,
        &mut summary,
        style_sources,
        &substrate,
    );
    summary.diagnostic_count = summary.diagnostics.len();
    push_omena_query_ready_surface(
        &mut summary.ready_surfaces,
        "cssModulesComposesResolutionDiagnostics",
    );
    push_omena_query_ready_surface(
        &mut summary.ready_surfaces,
        "cssModulesValueResolutionDiagnostics",
    );
    push_omena_query_ready_surface(&mut summary.ready_surfaces, "unusedSelectorDiagnostics");
    push_omena_query_ready_surface(&mut summary.ready_surfaces, "sassUseCycleDiagnostics");
    push_omena_query_ready_surface(&mut summary.ready_surfaces, "crossFileSccDiagnostics");
    push_omena_query_ready_surface(
        &mut summary.ready_surfaces,
        "unresolvedSassImportDiagnostics",
    );
    push_omena_query_ready_surface(
        &mut summary.ready_surfaces,
        "sassModuleResolutionIdentityDiagnostics",
    );
    push_omena_query_ready_surface(
        &mut summary.ready_surfaces,
        "missingExtendTargetDiagnostics",
    );
    push_omena_query_ready_surface(
        &mut summary.ready_surfaces,
        "graphAwareSassSymbolDiagnostics",
    );
    push_omena_query_ready_surface(
        &mut summary.ready_surfaces,
        "crossFileReplicaEnsembleDiagnostics",
    );
    push_omena_query_ready_surface(
        &mut summary.ready_surfaces,
        "crossFileReplicaEnsembleHintScope",
    );
    push_omena_query_ready_surface(
        &mut summary.ready_surfaces,
        "moduleGraphPropertyValueNarrowing",
    );
    let external_sif_context = OmenaQueryExternalSifResolutionContext {
        package_manifests,
        bundler_path_mappings: resolution_inputs.bundler_path_mappings.as_slice(),
        tsconfig_path_mappings: resolution_inputs.tsconfig_path_mappings.as_slice(),
        external_sifs,
    };
    let external_boundary_enabled = match external_mode {
        OmenaQueryExternalModuleModeV0::Ignored => false,
        OmenaQueryExternalModuleModeV0::Sif => true,
        OmenaQueryExternalModuleModeV0::Auto => target_has_auto_external_boundary_edges(
            target_style_path,
            external_sif_context,
            &substrate,
        ),
    };
    if external_boundary_enabled {
        // RFC 0004 #28 / #35: the file-scoped `@omena-strict: <level>` sigil dials the
        // external-boundary lattice behaviour. Absent/malformed sigil => `Standard`, which
        // keeps every branch below a no-op (byte-for-byte identical to the un-sigiled flow).
        let strictness = parse_omena_query_style_strictness_level(&target.style_source);
        let top_any_external_symbol_ranges =
            collect_omena_query_external_top_any_sass_symbol_ranges(
                target_style_path,
                style_sources,
                package_manifests,
                resolution_inputs.bundler_path_mappings.as_slice(),
                resolution_inputs.tsconfig_path_mappings.as_slice(),
                external_sifs,
                &substrate,
            );
        if strictness.suppresses_top_any_external_symbols() {
            summary.diagnostics.retain(|diagnostic| {
                diagnostic.code != "missingSassSymbol"
                    || !top_any_external_symbol_ranges.contains(&diagnostic.range)
            });
        } else {
            // `Closed` (#35): `TopOpaque` everywhere — genuinely-unknown external symbols are no
            // longer suppressed and are escalated to `error` rather than left as warnings.
            for diagnostic in summary.diagnostics.iter_mut() {
                if diagnostic.code == "missingSassSymbol"
                    && top_any_external_symbol_ranges.contains(&diagnostic.range)
                {
                    diagnostic.severity = "error";
                }
            }
        }
        if strictness.emits_external_boundary_diagnostics() {
            summary
                .diagnostics
                .extend(summarize_omena_query_external_sif_boundary_diagnostics(
                    target_style_path,
                    style_sources,
                    package_manifests,
                    resolution_inputs.bundler_path_mappings.as_slice(),
                    resolution_inputs.tsconfig_path_mappings.as_slice(),
                    external_sifs,
                    strictness,
                    &substrate,
                ));
        }
        push_omena_query_ready_surface(
            &mut summary.ready_surfaces,
            "externalSifBoundaryDiagnostics",
        );
        push_omena_query_ready_surface(&mut summary.ready_surfaces, "strictnessSigilGating");
    }
    attach_omena_query_runtime_state_inline_overrides_for_workspace(
        target_style_path,
        &mut summary,
        style_sources,
        source_documents,
        resolution_inputs,
    );
    push_omena_query_ready_surface(
        &mut summary.ready_surfaces,
        "runtimeStateInlineStyleOverrides",
    );
    apply_omena_query_checker_product_gate_to_style_diagnostics(&mut summary.diagnostics);
    push_omena_query_ready_surface(&mut summary.ready_surfaces, "checkerProductDiagnosticGate");
    match suppression_mode {
        OmenaQueryDiagnosticSuppressionModeV0::Apply => {
            apply_omena_query_style_diagnostic_suppressions(&target.style_source, &mut summary);
        }
        OmenaQueryDiagnosticSuppressionModeV0::ReportOnly => {
            report_omena_query_style_diagnostic_suppressions(&target.style_source, &mut summary);
        }
    }
    summary.diagnostic_count = summary.diagnostics.len();
    Some(summary)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn external_sif_canonical_url_match_accepts_raw_path_and_file_uri() {
        assert!(omena_query_external_sif_canonical_urls_match(
            "/tmp/tokens.scss",
            "file:///tmp/tokens.scss",
        ));
    }

    #[test]
    fn sass_import_is_plain_css_classifies_css_forms() {
        // CSS-form imports Sass explicitly KEEPS (must NOT be flagged).
        assert!(sass_import_is_plain_css("url(theme.css)"));
        assert!(sass_import_is_plain_css("url(theme.scss)")); // unquoted url is always CSS
        assert!(sass_import_is_plain_css("vendor.css"));
        assert!(sass_import_is_plain_css("VENDOR.CSS")); // case-insensitive
        assert!(sass_import_is_plain_css("//cdn.example/x.css"));
        assert!(sass_import_is_plain_css("https://x.com/y.css"));
        assert!(sass_import_is_plain_css("http://x.com/y")); // protocol URL, no .css
    }

    #[test]
    fn sass_import_is_plain_css_keeps_partials_flaggable() {
        // Over-correction guard: genuine Sass-form partials must STILL be classified
        // as Sass imports (i.e. NOT plain CSS), so the deprecation hint still fires.
        assert!(!sass_import_is_plain_css("partial"));
        assert!(!sass_import_is_plain_css("./legacy"));
        assert!(!sass_import_is_plain_css("foundation/buttons"));
        assert!(!sass_import_is_plain_css("legacy")); // bare partial name
    }

    #[test]
    fn media_qualified_import_is_not_deprecated() {
        // RFC-0007-D1 (#44): `@import "foo" screen` and `@import "foo" (min-width: ...)`
        // are kept as plain CSS by Sass; the media qualifier must suppress the hint.
        let screen = summarize_omena_query_sass_import_deprecation_hints(
            "theme.scss",
            "@import \"foo\" screen;",
        );
        assert!(
            screen.is_empty(),
            "media-qualified (Ident) @import must NOT warn deprecatedSassImport, got {screen:?}"
        );
        let feature = summarize_omena_query_sass_import_deprecation_hints(
            "theme.scss",
            "@import \"foo\" (min-width: 100px);",
        );
        assert!(
            feature.is_empty(),
            "media-feature-qualified @import must NOT warn deprecatedSassImport, got {feature:?}"
        );
    }

    #[test]
    fn bare_partial_import_still_deprecated() {
        // Over-correction guard: a genuine Sass-form `@import 'partial'` (no media, no
        // url, no `.css`) MUST still warn deprecatedSassImport.
        let diagnostics =
            summarize_omena_query_sass_import_deprecation_hints("theme.scss", "@import 'partial';");
        assert_eq!(
            diagnostics.len(),
            1,
            "bare Sass partial @import must still warn, got {diagnostics:?}"
        );
        assert_eq!(diagnostics[0].code, "deprecatedSassImport");
    }

    #[test]
    fn media_qualified_comma_peer_classifies_per_target() {
        // `@import "a", "b" screen`: only `"b"` is media-qualified; `"a"` stays a Sass
        // partial and must still warn. Per-target classification, not per-statement.
        let diagnostics = summarize_omena_query_sass_import_deprecation_hints(
            "theme.scss",
            "@import \"a\", \"b\" screen;",
        );
        assert_eq!(
            diagnostics.len(),
            1,
            "exactly the bare partial peer must warn, got {diagnostics:?}"
        );
    }

    /// Durable CI drift check: pin the full `sass:meta` module surface against a
    /// known-good Sass 1.77 member set. If a future edit adds or removes a member
    /// (or the upstream module surface changes and we update one site but not the
    /// pinned set), this fails loudly instead of silently rotting. (#44 D2)
    ///
    /// Functions and mixins are tracked separately because Sass distinguishes them
    /// (`meta.get-mixin` is a function returning a mixin reference; `meta.apply` is a
    /// mixin invoked via `@include`).
    #[test]
    fn sass_meta_allowlist_matches_pinned_1_77_surface() {
        let pinned_functions: BTreeSet<&str> = [
            "accepts-content",
            "calc-args",
            "calc-name",
            "call",
            "content-exists",
            "feature-exists",
            "function-exists",
            "get-function",
            "get-mixin",
            "global-variable-exists",
            "inspect",
            "keywords",
            "mixin-exists",
            "module-functions",
            "module-mixins",
            "module-variables",
            "type-of",
            "variable-exists",
        ]
        .into_iter()
        .collect();
        let pinned_mixins: BTreeSet<&str> = ["apply", "load-css"].into_iter().collect();

        let actual_functions: BTreeSet<&str> = sass_builtin_module_function_names("meta")
            .iter()
            .copied()
            .collect();
        let actual_mixins: BTreeSet<&str> = sass_builtin_module_mixin_names("meta")
            .iter()
            .copied()
            .collect();

        assert_eq!(
            actual_functions, pinned_functions,
            "sass:meta function allowlist drifted from pinned Sass 1.77 surface; \
             update both the allowlist and this pinned set together"
        );
        assert_eq!(
            actual_mixins, pinned_mixins,
            "sass:meta mixin allowlist drifted from pinned Sass 1.77 surface; \
             update both the allowlist and this pinned set together"
        );
    }

    // RFC-0007-F (#46): single-file local-composes validation.

    #[test]
    fn local_composes_flags_real_same_file_typo() {
        // True positive: `composes: missing` references a class that does not exist in this
        // file. With no cross-file context, this is fully resolvable and MUST be flagged.
        let source = ".base { color: red; }\n.button { composes: missing; }\n";
        let diagnostics = summarize_omena_query_css_modules_local_composes_style_diagnostics(
            "/tmp/foo.module.scss",
            source,
        );
        assert_eq!(diagnostics.len(), 1, "expected one missingComposedSelector");
        assert_eq!(diagnostics[0].code, "missingComposedSelector");
        assert!(
            diagnostics[0].message.contains("not found in this file"),
            "message should reference same-file resolution, got: {}",
            diagnostics[0].message
        );
    }

    #[test]
    fn local_composes_keeps_resolvable_target_silent() {
        // A local composes target that DOES exist in the file must NOT be flagged.
        let source = ".base { color: red; }\n.button { composes: base; }\n";
        let diagnostics = summarize_omena_query_css_modules_local_composes_style_diagnostics(
            "/tmp/foo.module.scss",
            source,
        );
        assert!(
            diagnostics.is_empty(),
            "resolvable local composes target should not be flagged, got: {diagnostics:?}"
        );
    }

    #[test]
    fn local_composes_does_not_flag_external_target_without_source() {
        // Over-correction guard: `composes: x from './other'` is an External edge that needs the
        // sibling module's facts. In single-file mode we have no access to `./other`, so we must
        // NOT invent a missingComposedSelector/missingComposedModule for it.
        let source = ".button { composes: shared from './other.module.scss'; color: blue; }\n";
        let diagnostics = summarize_omena_query_css_modules_local_composes_style_diagnostics(
            "/tmp/foo.module.scss",
            source,
        );
        assert!(
            diagnostics.is_empty(),
            "external composes target must not be flagged without cross-file source, got: {diagnostics:?}"
        );
    }

    #[test]
    fn local_composes_does_not_flag_global_target() {
        // `composes: x from global` references no concrete selector and must produce nothing.
        let source = ".button { composes: someGlobal from global; color: blue; }\n";
        let diagnostics = summarize_omena_query_css_modules_local_composes_style_diagnostics(
            "/tmp/foo.module.scss",
            source,
        );
        assert!(
            diagnostics.is_empty(),
            "global composes target must not be flagged, got: {diagnostics:?}"
        );
    }

    #[test]
    fn single_file_summary_includes_local_composes_typo() -> Result<(), String> {
        // The CLI bare path (`style-diagnostics foo` with no --source) routes through this
        // wrapper. A real same-file composes typo must surface even without --source, closing
        // the invocation-mode inconsistency the issue describes.
        let style_uri = "/tmp/foo.module.scss";
        let source = ".base { color: red; }\n.button { composes: missing; }\n";
        let candidates = summarize_omena_query_style_hover_candidates(style_uri, source)
            .ok_or("hover candidates")?;
        let summary = summarize_omena_query_style_diagnostics_for_file_with_local_composes(
            style_uri,
            source,
            candidates.candidates.as_slice(),
        );
        assert!(
            summary
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "missingComposedSelector"),
            "bare single-file summary should surface the local composes typo, got: {:?}",
            summary.diagnostics
        );
        assert_eq!(summary.diagnostic_count, summary.diagnostics.len());
        Ok(())
    }

    #[test]
    fn single_file_summary_does_not_flag_external_composes() -> Result<(), String> {
        // Over-correction guard at the wrapper level: a clean file whose only composes target is
        // cross-file must stay free of composes diagnostics in single-file mode.
        let style_uri = "/tmp/foo.module.scss";
        let source = ".button { composes: shared from './other.module.scss'; color: blue; }\n";
        let candidates = summarize_omena_query_style_hover_candidates(style_uri, source)
            .ok_or("hover candidates")?;
        let summary = summarize_omena_query_style_diagnostics_for_file_with_local_composes(
            style_uri,
            source,
            candidates.candidates.as_slice(),
        );
        assert!(
            !summary.diagnostics.iter().any(|diagnostic| {
                diagnostic.code == "missingComposedSelector"
                    || diagnostic.code == "missingComposedModule"
            }),
            "external composes target must not be flagged in single-file mode, got: {:?}",
            summary.diagnostics
        );
        Ok(())
    }
}
