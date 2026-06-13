use std::collections::BTreeSet;

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

use super::super::cascade_checker::collect_query_replica_ensemble_site_outcomes;
use super::sass::collect_sass_module_graph_reachable_style_paths;
use super::substrate::OmenaQueryWorkspaceDiagnosticsSubstrateV0;
use super::*;

pub(super) fn summarize_omena_query_replica_ensemble_inconsistency_diagnostics_for_workspace(
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

    let resolution = &substrate.sass_resolution;
    let reachable_paths =
        collect_sass_module_graph_reachable_style_paths(target_style_path, resolution);

    if reachable_paths.len() < 2 {
        return Vec::new();
    }

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
