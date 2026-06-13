#[cfg(feature = "hypergraph-ifds")]
use std::collections::BTreeSet;

use super::shared::*;
use super::substrate::OmenaQueryWorkspaceDiagnosticsSubstrateV0;

#[cfg(feature = "hypergraph-ifds")]
pub(super) fn summarize_omena_query_unified_cross_file_scc_diagnostics_for_workspace(
    target_style_path: &str,
    target_style_source: &str,
    style_sources: &[OmenaQueryStyleSourceInputV0],
    source_documents: &[OmenaQuerySourceDocumentInputV0],
    package_manifests: &[OmenaQueryStylePackageManifestV0],
    substrate: &OmenaQueryWorkspaceDiagnosticsSubstrateV0,
) -> Vec<OmenaQueryStyleDiagnosticV0> {
    let summary = super::super::cross_file_summary::summarize_omena_query_workspace_cross_file_summary_with_substrate(
        style_sources,
        source_documents,
        package_manifests,
        &substrate.style_fact_entries,
        &substrate.css_modules_resolution,
        &substrate.sass_resolution_without_path_mappings,
    );
    let hypergraph = super::super::summarize_omena_query_unified_cross_file_hypergraph(&summary);
    let report = super::super::summarize_omena_query_unified_cross_file_scc_report(&hypergraph);
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
pub(super) fn summarize_omena_query_unified_cross_file_scc_diagnostics_for_workspace(
    _target_style_path: &str,
    _target_style_source: &str,
    _style_sources: &[OmenaQueryStyleSourceInputV0],
    _source_documents: &[OmenaQuerySourceDocumentInputV0],
    _package_manifests: &[OmenaQueryStylePackageManifestV0],
    _substrate: &OmenaQueryWorkspaceDiagnosticsSubstrateV0,
) -> Vec<OmenaQueryStyleDiagnosticV0> {
    Vec::new()
}
