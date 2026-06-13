use std::collections::{BTreeMap, BTreeSet};

use omena_parser::{
    ParsedAnimationFactKind, ParsedCssModuleComposesEdgeKind, ParsedVariableFactKind,
};
use omena_query_checker_orchestrator::{
    ModuleGraphEdgeV0, ModuleGraphV0, OutcomeMode,
    REPLICA_ENSEMBLE_DEFAULT_PRODUCT_DECISION_MECHANISM_V0, REPLICA_ENSEMBLE_FEATURE_GATE_V0,
    REPLICA_ENSEMBLE_LAYER_MARKER_V0, REPLICA_ENSEMBLE_MECHANISM_SCOPE_V0,
    REPLICA_ENSEMBLE_PRODUCT_SURFACE_V0, REPLICA_ENSEMBLE_SCHEMA_VERSION_V0, ReplicaSnapshotV0,
    ReportOptionsV0, ReportRecommendation, build_cross_file_inconsistency_report,
};
use omena_query_checker_orchestrator::{
    OmenaCheckerReplicaEnsembleInputV0, OmenaCheckerReplicaEnsembleReportInputV0,
    run_omena_query_checker_replica_ensemble_gate_v0,
};

use super::cascade_checker::collect_query_replica_ensemble_site_outcomes;
use super::cascade_checker::query_runtime_state_confidence_tier;
use super::cascade_checker::summarize_query_cascade_checker_diagnostics_with_deep_analysis;
use super::diagnostic_suppressions::apply_omena_query_style_diagnostic_suppressions;
use super::diagnostic_suppressions::parse_omena_query_style_strictness_level;
use super::diagnostic_suppressions::report_omena_query_style_diagnostic_suppressions;
use super::parser_facade::collect_omena_query_omena_parser_style_facts_raw;
use super::*;

mod cascade_runtime;
mod css_modules;
mod external_sif;
mod render;
mod sass;
mod sass_builtins;
mod sass_symbols;
mod single_file;
mod source_usage;
mod substrate;
mod types;

use cascade_runtime::{
    attach_omena_query_module_graph_property_value_narrowing_for_workspace,
    attach_omena_query_runtime_state_inline_overrides_for_workspace,
};
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
};
use render::whole_file_omena_query_style_range;
#[cfg(test)]
use sass::sass_import_is_plain_css;
pub(super) use sass::{
    collect_sass_module_graph_reachable_style_paths, sass_module_source_is_workspace_local,
};
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
use substrate::{
    OmenaQueryWorkspaceDiagnosticsSubstrateV0, collect_omena_query_workspace_diagnostics_substrate,
};
use types::LSP_DIAGNOSTIC_TAG_UNNECESSARY;
pub use types::OmenaQueryExternalModuleModeV0;

/// Surface the real cross-file replica-ensemble inconsistency diagnostic in the
/// workspace style path (#33 / L0 / L2).
///
/// When the target file's resolved `@use`/`@forward`/`@import` graph closure spans
/// two or more in-graph CSS modules, each module is treated as one *replica* of the
/// shared design surface and its REAL per-`(selector, property)` cascade winners are
/// extracted via `collect_query_replica_ensemble_site_outcomes` (genuine
/// `cascade_property` ranking over the parsed declarations — no fabricated
/// snapshots). `omena-ensemble`'s `build_cross_file_inconsistency_report` then
/// computes the replica overlap-Q distribution over the modules' winners and the
/// SBM detectability over the resolved module graph; the report's overlap statistics
/// (recommendation, `meanQ`, genuine disagreement-pair count) drive the registered
/// `replicaEnsembleInconsistency` checker rule through
/// `run_omena_query_checker_replica_ensemble_gate_v0`.
///
/// The diagnostic depends entirely on the overlap statistics over the real winners:
/// a workspace whose modules agree on every shared `(selector, property)` outcome
/// has `meanQ == 1.0`, zero disagreement pairs, and a `noActionNeeded`
/// recommendation, so the checker rule filters the report out and nothing is
/// surfaced; a workspace where two modules resolve a shared site to different
/// winning values drops `meanQ` below one and surfaces the diagnostic. The report is
/// whole-graph, so the single emitted diagnostic is anchored on the target file's
/// whole-file span.
fn summarize_omena_query_replica_ensemble_inconsistency_diagnostics_for_workspace(
    target_style_path: &str,
    style_sources: &[OmenaQueryStyleSourceInputV0],
    substrate: &OmenaQueryWorkspaceDiagnosticsSubstrateV0,
) -> Vec<OmenaQueryStyleDiagnosticV0> {
    let Some(target) = style_sources
        .iter()
        .find(|source| source.style_path == target_style_path)
    else {
        return Vec::new();
    };

    // Substrate RES-A slot: plain resolution with (package_manifests, bundler, tsconfig).
    let resolution = &substrate.sass_resolution;
    let reachable_paths =
        collect_sass_module_graph_reachable_style_paths(target_style_path, resolution);

    // A single-module closure is not an ensemble: there is no second replica to
    // overlap against, so there is no cross-file inconsistency to surface.
    if reachable_paths.len() < 2 {
        return Vec::new();
    }

    // Build one replica per in-graph module from its REAL cascade winners. A module
    // that declares no comparable definite cascade site contributes an empty replica
    // (it cannot agree or disagree with anything), so drop it from the ensemble.
    let replicas = style_sources
        .iter()
        .filter(|source| reachable_paths.contains(source.style_path.as_str()))
        .filter_map(|source| {
            let sites = collect_query_replica_ensemble_site_outcomes(source.style_source.as_str());
            if sites.is_empty() {
                return None;
            }
            Some(ReplicaSnapshotV0 {
                schema_version: REPLICA_ENSEMBLE_SCHEMA_VERSION_V0,
                product: "omena-ensemble.replica-snapshot",
                layer_marker: REPLICA_ENSEMBLE_LAYER_MARKER_V0,
                feature_gate: REPLICA_ENSEMBLE_FEATURE_GATE_V0,
                path: source.style_path.clone(),
                sites,
            })
        })
        .collect::<Vec<_>>();

    // Fewer than two non-empty replicas => no shared cascade surface to compare.
    if replicas.len() < 2 {
        return Vec::new();
    }

    let module_graph = replica_ensemble_module_graph_from_resolution(
        target_style_path,
        resolution,
        &reachable_paths,
        &replicas,
    );
    let report = build_cross_file_inconsistency_report(
        target_style_path,
        replicas.clone(),
        &module_graph,
        OutcomeMode::DefiniteOnly,
        ReportOptionsV0::default(),
        None,
    );
    debug_assert_eq!(report.mechanism_scope, REPLICA_ENSEMBLE_MECHANISM_SCOPE_V0);
    debug_assert_eq!(report.product_surface, REPLICA_ENSEMBLE_PRODUCT_SURFACE_V0);
    debug_assert_eq!(
        report.default_product_decision_mechanism,
        REPLICA_ENSEMBLE_DEFAULT_PRODUCT_DECISION_MECHANISM_V0
    );

    // Genuine disagreement count: only replica pairs whose computed overlap-Q is
    // strictly below 1.0 actually disagree on a shared cascade outcome. (The report's
    // `top_disagreement_pairs` keeps the lowest-Q pairs even when every pair fully
    // agrees, so counting that list directly would fire on a consistent ensemble.)
    let genuine_disagreement_pair_count = report
        .top_disagreement_pairs
        .iter()
        .filter(|pair| pair.shared_site_count > 0 && pair.overlap_q < 1.0)
        .count();
    let recommendation = replica_ensemble_recommendation_name(report.recommendation);

    let gate =
        run_omena_query_checker_replica_ensemble_gate_v0(OmenaCheckerReplicaEnsembleInputV0 {
            reports: vec![OmenaCheckerReplicaEnsembleReportInputV0 {
                workspace_root: target_style_path.to_string(),
                recommendation: recommendation.to_string(),
                mean_q: report.distribution.mean_q,
                variance_q: report.distribution.variance_q,
                top_disagreement_pair_count: genuine_disagreement_pair_count,
                mechanism_scope: report.mechanism_scope.to_string(),
                product_surface: report.product_surface.to_string(),
                default_product_decision_mechanism: report.default_product_decision_mechanism,
            }],
        });
    if !gate.enforcement_passed {
        return Vec::new();
    }

    let whole_file_range = parser_range_for_byte_span(
        target.style_source.as_str(),
        ParserByteSpanV0 {
            start: 0,
            end: target.style_source.len(),
        },
    );

    gate.evaluations
        .into_iter()
        .map(|evaluation| {
            let mut provenance = vec![
                "omena-query-checker-orchestrator.replica-ensemble-gate",
                "omena-checker.replica-ensemble-rules",
                "omena-ensemble.cross-file-inconsistency-report",
                "omena-query.cross-file-replica-ensemble",
            ];
            provenance.extend(evaluation.mechanism_products.iter().copied());
            OmenaQueryStyleDiagnosticV0 {
                code: "replicaEnsembleInconsistency",
                severity: "hint",
                provenance,
                range: whole_file_range,
                message: evaluation.message,
                tags: Vec::new(),
                create_custom_property: None,
                cascade_narrowing: None,
                cascade_confidence: None,
                polynomial_provenance: None,
                cross_file_scc: None,
            }
        })
        .collect()
}

/// Build the replica-ensemble module graph from the target's resolved import graph:
/// nodes are the in-graph modules that contributed a non-empty replica, and edges
/// are the resolved `@use`/`@forward`/`@import` edges between those modules. This is
/// the real dependency structure the SBM detectability reasons over — not a
/// synthesized clique.
fn replica_ensemble_module_graph_from_resolution(
    workspace_root: &str,
    resolution: &OmenaQuerySassModuleCrossFileResolutionV0,
    reachable_paths: &BTreeSet<&str>,
    replicas: &[ReplicaSnapshotV0],
) -> ModuleGraphV0 {
    let nodes = replicas
        .iter()
        .map(|replica| replica.path.clone())
        .collect::<Vec<_>>();
    let node_set = nodes.iter().map(String::as_str).collect::<BTreeSet<_>>();

    let edges = resolution
        .edges
        .iter()
        .filter(|edge| edge.status == "resolved")
        .filter(|edge| reachable_paths.contains(edge.from_style_path.as_str()))
        .filter_map(|edge| {
            let to = edge.resolved_style_path.as_deref()?;
            if node_set.contains(edge.from_style_path.as_str()) && node_set.contains(to) {
                Some(ModuleGraphEdgeV0 {
                    schema_version: REPLICA_ENSEMBLE_SCHEMA_VERSION_V0,
                    product: "omena-ensemble.module-graph-edge",
                    layer_marker: REPLICA_ENSEMBLE_LAYER_MARKER_V0,
                    feature_gate: REPLICA_ENSEMBLE_FEATURE_GATE_V0,
                    from_module: edge.from_style_path.clone(),
                    to_module: to.to_string(),
                    edge_kind: "resolvedModuleEdge",
                })
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    ModuleGraphV0 {
        schema_version: REPLICA_ENSEMBLE_SCHEMA_VERSION_V0,
        product: "omena-ensemble.module-graph",
        layer_marker: REPLICA_ENSEMBLE_LAYER_MARKER_V0,
        feature_gate: REPLICA_ENSEMBLE_FEATURE_GATE_V0,
        workspace_root: workspace_root.to_string(),
        nodes,
        edges,
    }
}

fn replica_ensemble_recommendation_name(recommendation: ReportRecommendation) -> &'static str {
    match recommendation {
        ReportRecommendation::NoActionNeeded => "noActionNeeded",
        ReportRecommendation::InvestigateRsbBroken => "investigateRsbBroken",
        ReportRecommendation::UndetectablePhase => "undetectablePhase",
    }
}

#[cfg(feature = "hypergraph-ifds")]
fn summarize_omena_query_unified_cross_file_scc_diagnostics_for_workspace(
    target_style_path: &str,
    target_style_source: &str,
    style_sources: &[OmenaQueryStyleSourceInputV0],
    source_documents: &[OmenaQuerySourceDocumentInputV0],
    package_manifests: &[OmenaQueryStylePackageManifestV0],
    substrate: &OmenaQueryWorkspaceDiagnosticsSubstrateV0,
) -> Vec<OmenaQueryStyleDiagnosticV0> {
    // Substrate slots ENTRIES + RES-E (css-modules resolution) + RES-C (plain Sass
    // resolution with EMPTY path mappings) — exactly what the workspace cross-file
    // summary collected for itself. The summary's source-selector leg still re-parses
    // `source_documents` internally (not covered by the substrate).
    let summary = super::cross_file_summary::summarize_omena_query_workspace_cross_file_summary_with_substrate(
        style_sources,
        source_documents,
        package_manifests,
        &substrate.style_fact_entries,
        &substrate.css_modules_resolution,
        &substrate.sass_resolution_without_path_mappings,
    );
    let hypergraph = summarize_omena_query_unified_cross_file_hypergraph(&summary);
    let report = summarize_omena_query_unified_cross_file_scc_report(&hypergraph);
    if report.sccs.is_empty() {
        return Vec::new();
    }

    let whole_file_range = parser_range_for_byte_span(
        target_style_source,
        ParserByteSpanV0 {
            start: 0,
            end: target_style_source.len(),
        },
    );
    let mut emitted = BTreeSet::new();
    report
        .sccs
        .into_iter()
        .filter(|scc| scc.cross_file)
        .filter(|scc| scc.style_paths.iter().any(|path| path == target_style_path))
        .filter(|scc| {
            scc.edge_kinds
                .iter()
                .any(|edge_kind| edge_kind.starts_with("composes"))
        })
        .filter_map(|scc| {
            if !emitted.insert(scc.scc_id.clone()) {
                return None;
            }
            let style_path_count = scc.style_paths.len();
            let edge_kinds = scc.edge_kinds.join(", ");
            let cycle_paths = scc.style_paths.join(" -> ");
            Some(OmenaQueryStyleDiagnosticV0 {
                code: "crossFileStyleCycle",
                severity: "warning",
                provenance: vec![
                    "omena-query.unified-cross-file-scc-report",
                    "omena-query.unified-cross-file-hypergraph",
                    "omena-query.cross-file-summary",
                ],
                range: whole_file_range,
                message: format!(
                    "Cross-file style dependency cycle across {style_path_count} files via {edge_kinds}: {cycle_paths}"
                ),
                tags: Vec::new(),
                create_custom_property: None,
                cascade_narrowing: None,
                cascade_confidence: None,
                polynomial_provenance: None,
                cross_file_scc: Some(scc),
            })
        })
        .collect()
}

#[cfg(not(feature = "hypergraph-ifds"))]
fn summarize_omena_query_unified_cross_file_scc_diagnostics_for_workspace(
    _target_style_path: &str,
    _target_style_source: &str,
    _style_sources: &[OmenaQueryStyleSourceInputV0],
    _source_documents: &[OmenaQuerySourceDocumentInputV0],
    _package_manifests: &[OmenaQueryStylePackageManifestV0],
    _substrate: &OmenaQueryWorkspaceDiagnosticsSubstrateV0,
) -> Vec<OmenaQueryStyleDiagnosticV0> {
    Vec::new()
}

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
    if external_mode == OmenaQueryExternalModuleModeV0::Sif {
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
        package_manifests,
        resolution_inputs.bundler_path_mappings.as_slice(),
        resolution_inputs.tsconfig_path_mappings.as_slice(),
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

pub fn summarize_omena_query_sass_module_resolution_identity_diagnostics_for_workspace(
    target_style_path: &str,
    workspace_sources: &[OmenaQueryStyleSourceInputV0],
    package_manifests: &[OmenaQueryStylePackageManifestV0],
    resolution_inputs: &OmenaQueryStyleResolutionInputsV0,
) -> Vec<OmenaQueryStyleDiagnosticV0> {
    if !workspace_sources
        .iter()
        .any(|source| source.style_path == target_style_path)
    {
        return Vec::new();
    }
    let resolution = summarize_omena_query_sass_module_cross_file_resolution_for_workspace(
        workspace_sources,
        package_manifests,
        resolution_inputs.bundler_path_mappings.as_slice(),
        resolution_inputs.tsconfig_path_mappings.as_slice(),
    );
    summarize_omena_query_sass_module_resolution_identity_diagnostics_for_workspace_from_resolution(
        target_style_path,
        workspace_sources,
        &resolution,
    )
}

/// Substrate-threaded core of the resolution-identity pass (RFC 0009 Pillar B stage-2,
/// #65). `resolution` is the substrate's RES-A slot — plain resolution with
/// (package_manifests, bundler, tsconfig), identical to what the pub wrapper above
/// computes via `summarize_omena_query_sass_module_cross_file_resolution_for_workspace`.
fn summarize_omena_query_sass_module_resolution_identity_diagnostics_for_workspace_from_resolution(
    target_style_path: &str,
    workspace_sources: &[OmenaQueryStyleSourceInputV0],
    resolution: &OmenaQuerySassModuleCrossFileResolutionV0,
) -> Vec<OmenaQueryStyleDiagnosticV0> {
    let Some(target) = workspace_sources
        .iter()
        .find(|source| source.style_path == target_style_path)
    else {
        return Vec::new();
    };
    let range = whole_file_omena_query_style_range(target.style_source.as_str());
    let mut emitted = BTreeSet::new();
    let mut diagnostics = Vec::new();

    for edge in resolution
        .edges
        .iter()
        .filter(|edge| edge.from_style_path == target_style_path)
    {
        let visible_symlink_links = edge
            .symlink_chain_links
            .iter()
            .filter(|link| !is_platform_alias_omena_query_symlink_link(link))
            .collect::<Vec<_>>();
        if !visible_symlink_links.is_empty()
            && emitted.insert((
                "sassModuleSymlinkResolution",
                edge.source.clone(),
                edge.resolved_style_path.clone(),
            ))
        {
            let target_path = edge
                .resolved_style_path
                .as_deref()
                .unwrap_or(edge.source.as_str());
            let link_summary = visible_symlink_links
                .first()
                .map(|link| format!("; first link {} -> {}", link.link_path, link.target_path))
                .unwrap_or_default();
            diagnostics.push(OmenaQueryStyleDiagnosticV0 {
                code: "sassModuleSymlinkResolution",
                severity: "hint",
                provenance: vec![
                    "omena-query.sass-module-cross-file-resolution",
                    "omena-resolver.symlink-chain-metadata",
                    "omena-query.style-diagnostics",
                ],
                range,
                message: format!(
                    "Sass module '{}' resolves to '{}' through {} symlink link(s){}.",
                    edge.source,
                    target_path,
                    visible_symlink_links.len(),
                    link_summary
                ),
                tags: Vec::new(),
                create_custom_property: None,
                cascade_narrowing: None,
                cascade_confidence: None,
                polynomial_provenance: None,
                cross_file_scc: None,
            });
        }

        if edge.configuration_variable_count > 0
            && let Some(identity_key) = edge.module_instance_identity_key.as_ref()
            && emitted.insert((
                "sassModuleInstanceIdentity",
                edge.source.clone(),
                Some(identity_key.clone()),
            ))
        {
            diagnostics.push(OmenaQueryStyleDiagnosticV0 {
                code: "sassModuleInstanceIdentity",
                severity: "hint",
                provenance: vec![
                    "omena-query.sass-module-cross-file-resolution",
                    "omena-query.module-instance-identity",
                    "omena-query.style-diagnostics",
                ],
                range,
                message: format!(
                    "Sass module '{}' uses {} configured variable(s); module instance identity is {}.",
                    edge.source, edge.configuration_variable_count, identity_key
                ),
                tags: Vec::new(),
                create_custom_property: None,
                cascade_narrowing: None,
                cascade_confidence: None,
                polynomial_provenance: None,
                cross_file_scc: None,
            });
        }

        if !edge.invalid_configuration_variable_names.is_empty()
            && emitted.insert((
                "sassModuleInvalidConfiguration",
                edge.source.clone(),
                edge.resolved_style_path.clone(),
            ))
        {
            let target_path = edge
                .resolved_style_path
                .as_deref()
                .unwrap_or(edge.source.as_str());
            diagnostics.push(OmenaQueryStyleDiagnosticV0 {
                code: "sassModuleInvalidConfiguration",
                severity: "error",
                provenance: vec![
                    "omena-query.sass-module-cross-file-resolution",
                    "omena-query.module-instance-identity",
                    "omena-query.style-diagnostics",
                ],
                range,
                message: format!(
                    "Sass module '{}' configures {} on '{}', but Sass @use/@forward with(...) can configure only public !default variables.",
                    edge.source,
                    format_omena_query_sass_configuration_variable_names(
                        edge.invalid_configuration_variable_names.as_slice()
                    ),
                    target_path
                ),
                tags: Vec::new(),
                create_custom_property: None,
                cascade_narrowing: None,
                cascade_confidence: None,
                polynomial_provenance: None,
                cross_file_scc: None,
            });
        }
    }

    for edge in resolution
        .graph_closure_edges
        .iter()
        .filter(|edge| edge.from_style_path == target_style_path)
        .filter(|edge| edge.configuration_variable_count > 0)
    {
        let Some(identity_key) = edge.module_instance_identity_key.as_ref() else {
            continue;
        };
        if !emitted.insert((
            "sassModuleInstanceIdentity",
            edge.target_style_path.clone(),
            Some(identity_key.clone()),
        )) {
            continue;
        }
        diagnostics.push(OmenaQueryStyleDiagnosticV0 {
            code: "sassModuleInstanceIdentity",
            severity: "hint",
            provenance: vec![
                "omena-query.sass-module-cross-file-resolution",
                "omena-query.module-instance-identity",
                "omena-query.style-diagnostics",
            ],
            range,
            message: format!(
                "Sass module graph reaches configured module instance '{}' in {} hop(s); module instance identity is {}.",
                edge.target_style_path, edge.depth, identity_key
            ),
            tags: Vec::new(),
            create_custom_property: None,
            cascade_narrowing: None,
            cascade_confidence: None,
            polynomial_provenance: None,
            cross_file_scc: None,
        });
    }
    for edge in resolution
        .graph_closure_edges
        .iter()
        .filter(|edge| edge.from_style_path == target_style_path)
        .filter(|edge| !edge.invalid_configuration_variable_names.is_empty())
    {
        if !emitted.insert((
            "sassModuleInvalidConfiguration",
            edge.target_style_path.clone(),
            Some(edge.configuration_signature.clone()),
        )) {
            continue;
        }
        diagnostics.push(OmenaQueryStyleDiagnosticV0 {
            code: "sassModuleInvalidConfiguration",
            severity: "error",
            provenance: vec![
                "omena-query.sass-module-cross-file-resolution",
                "omena-query.module-instance-identity",
                "omena-query.style-diagnostics",
            ],
            range,
            message: format!(
                "Sass module graph reaches invalid configuration for '{}': {} are not public !default variables.",
                edge.target_style_path,
                format_omena_query_sass_configuration_variable_names(
                    edge.invalid_configuration_variable_names.as_slice()
                )
            ),
            tags: Vec::new(),
            create_custom_property: None,
            cascade_narrowing: None,
            cascade_confidence: None,
            polynomial_provenance: None,
            cross_file_scc: None,
        });
    }
    diagnostics.extend(
        summarize_omena_query_sass_module_configuration_conflict_diagnostics(
            target_style_path,
            workspace_sources,
            resolution,
            range,
        ),
    );

    diagnostics
}

fn summarize_omena_query_sass_module_configuration_conflict_diagnostics(
    target_style_path: &str,
    workspace_sources: &[OmenaQueryStyleSourceInputV0],
    resolution: &OmenaQuerySassModuleCrossFileResolutionV0,
    range: ParserRangeV0,
) -> Vec<OmenaQueryStyleDiagnosticV0> {
    let mut signatures_by_target = BTreeMap::<String, BTreeSet<String>>::new();
    for edge in resolution
        .graph_closure_edges
        .iter()
        .filter(|edge| edge.from_style_path == target_style_path)
        .filter(|edge| edge.configuration_variable_count > 0)
    {
        signatures_by_target
            .entry(edge.target_style_path.clone())
            .or_default()
            .insert(edge.configuration_signature.clone());
    }
    for (target, signatures) in collect_omena_query_sass_module_load_order_configuration_conflicts(
        target_style_path,
        workspace_sources,
        resolution,
    ) {
        signatures_by_target
            .entry(target)
            .or_default()
            .extend(signatures);
    }

    signatures_by_target
        .into_iter()
        .filter(|(_, signatures)| signatures.len() > 1)
        .map(|(target, signatures)| OmenaQueryStyleDiagnosticV0 {
            code: "sassModuleConfigurationConflict",
            severity: "error",
            provenance: vec![
                "omena-query.sass-module-cross-file-resolution",
                "omena-query.module-instance-identity",
                "omena-query.style-diagnostics",
            ],
            range,
            message: format!(
                "Sass module '{target}' is reached with {} different configurations ({}); Sass modules can be configured only once per compilation.",
                signatures.len(),
                signatures.into_iter().collect::<Vec<_>>().join(", ")
            ),
            tags: Vec::new(),
            create_custom_property: None,
            cascade_narrowing: None,
            cascade_confidence: None,
            polynomial_provenance: None,
            cross_file_scc: None,
        })
        .collect()
}

fn collect_omena_query_sass_module_load_order_configuration_conflicts(
    target_style_path: &str,
    workspace_sources: &[OmenaQueryStyleSourceInputV0],
    resolution: &OmenaQuerySassModuleCrossFileResolutionV0,
) -> BTreeMap<String, BTreeSet<String>> {
    let source_by_path = workspace_sources
        .iter()
        .map(|source| (source.style_path.as_str(), source.style_source.as_str()))
        .collect::<BTreeMap<_, _>>();
    let mut edges_by_from = BTreeMap::<&str, Vec<&OmenaQuerySassModuleEdgeResolutionV0>>::new();
    for edge in resolution
        .edges
        .iter()
        .filter(|edge| edge.status == "resolved" && edge.resolved_style_path.is_some())
    {
        edges_by_from
            .entry(edge.from_style_path.as_str())
            .or_default()
            .push(edge);
    }
    for (style_path, edges) in &mut edges_by_from {
        let style_source = source_by_path.get(style_path).copied().unwrap_or_default();
        edges.sort_by_key(|edge| {
            (
                omena_query_sass_module_edge_source_offset(
                    style_source,
                    edge.edge_kind,
                    edge.source.as_str(),
                ),
                edge.edge_kind,
                edge.rule_ordinal,
                edge.source.clone(),
            )
        });
    }

    let mut loaded_signatures_by_target = BTreeMap::new();
    let mut active_stack = BTreeSet::new();
    let mut conflicts_by_target = BTreeMap::new();
    collect_omena_query_sass_module_load_order_configuration_conflicts_for_style(
        target_style_path,
        &edges_by_from,
        &mut loaded_signatures_by_target,
        &mut active_stack,
        &mut conflicts_by_target,
    );
    conflicts_by_target
}

fn collect_omena_query_sass_module_load_order_configuration_conflicts_for_style(
    style_path: &str,
    edges_by_from: &BTreeMap<&str, Vec<&OmenaQuerySassModuleEdgeResolutionV0>>,
    loaded_signatures_by_target: &mut BTreeMap<String, String>,
    active_stack: &mut BTreeSet<String>,
    conflicts_by_target: &mut BTreeMap<String, BTreeSet<String>>,
) {
    if !active_stack.insert(style_path.to_string()) {
        return;
    }
    if let Some(edges) = edges_by_from.get(style_path) {
        for edge in edges {
            let Some(target_style_path) = edge.resolved_style_path.as_ref() else {
                continue;
            };
            let requested_signature = edge.configuration_signature.clone();
            let should_visit_target =
                match loaded_signatures_by_target.get(target_style_path.as_str()) {
                    Some(existing_signature)
                        if is_unconfigured_omena_query_sass_module_signature(
                            requested_signature.as_str(),
                        ) || existing_signature == &requested_signature =>
                    {
                        false
                    }
                    Some(existing_signature) => {
                        let signatures = conflicts_by_target
                            .entry(target_style_path.clone())
                            .or_default();
                        signatures.insert(existing_signature.clone());
                        signatures.insert(requested_signature);
                        false
                    }
                    None => {
                        loaded_signatures_by_target
                            .insert(target_style_path.clone(), requested_signature);
                        true
                    }
                };
            if should_visit_target {
                collect_omena_query_sass_module_load_order_configuration_conflicts_for_style(
                    target_style_path.as_str(),
                    edges_by_from,
                    loaded_signatures_by_target,
                    active_stack,
                    conflicts_by_target,
                );
            }
        }
    }
    active_stack.remove(style_path);
}

fn is_unconfigured_omena_query_sass_module_signature(signature: &str) -> bool {
    signature == "with:none"
}

fn format_omena_query_sass_configuration_variable_names(names: &[String]) -> String {
    names
        .iter()
        .map(|name| format!("${name}"))
        .collect::<Vec<_>>()
        .join(", ")
}

fn omena_query_sass_module_edge_source_offset(
    style_source: &str,
    edge_kind: &str,
    source: &str,
) -> usize {
    let keyword = match edge_kind {
        "sassUse" => "@use",
        "sassForward" => "@forward",
        _ => return usize::MAX,
    };
    let mut search_start = 0usize;
    while let Some(relative_keyword_start) = style_source[search_start..].find(keyword) {
        let keyword_start = search_start + relative_keyword_start;
        let after_keyword = &style_source[keyword_start + keyword.len()..];
        let Some(relative_source_start) = after_keyword.find(source) else {
            search_start = keyword_start + keyword.len();
            continue;
        };
        let between_keyword_and_source = &after_keyword[..relative_source_start];
        if !between_keyword_and_source.contains(';') && !between_keyword_and_source.contains('{') {
            return keyword_start;
        }
        search_start = keyword_start + keyword.len();
    }
    usize::MAX
}

fn is_platform_alias_omena_query_symlink_link(link: &OmenaQuerySymlinkChainLinkV0) -> bool {
    matches!(
        (link.link_path.as_str(), link.target_path.as_str()),
        ("/var", "/private/var") | ("/tmp", "/private/tmp") | ("/etc", "/private/etc")
    )
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
