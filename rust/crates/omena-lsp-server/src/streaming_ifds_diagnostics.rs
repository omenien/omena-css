use omena_query::{
    OmenaQueryCrossFileSummaryV0, OmenaQuerySourceDocumentInputV0, OmenaQueryStyleDiagnosticV0,
    OmenaQueryStylePackageManifestV0, OmenaQueryStyleSourceInputV0, ParserRangeV0,
    summarize_omena_query_unified_cross_file_hypergraph,
    summarize_omena_query_workspace_cross_file_summary,
};
use omena_streaming_ifds::summarize_streaming_ifds_cross_file_reachability_v0;

pub(crate) fn summarize_cross_file_streaming_reachability_diagnostics_for_lsp(
    target_style_path: &str,
    style_sources: &[OmenaQueryStyleSourceInputV0],
    source_documents: &[OmenaQuerySourceDocumentInputV0],
    package_manifests: &[OmenaQueryStylePackageManifestV0],
    committed_cross_file_summary: Option<&OmenaQueryCrossFileSummaryV0>,
) -> Vec<OmenaQueryStyleDiagnosticV0> {
    let hypergraph = if let Some(summary) = committed_cross_file_summary {
        summarize_omena_query_unified_cross_file_hypergraph(summary)
    } else {
        summarize_omena_query_unified_cross_file_hypergraph(
            &summarize_omena_query_workspace_cross_file_summary(
                style_sources,
                source_documents,
                package_manifests,
            ),
        )
    };
    let report = summarize_streaming_ifds_cross_file_reachability_v0(
        target_style_path,
        hypergraph.hyperedges.as_slice(),
    );
    if report.reachable_foreign_paths.is_empty() {
        return Vec::new();
    }

    vec![OmenaQueryStyleDiagnosticV0 {
        code: "crossFileStreamingReachability",
        severity: "hint",
        provenance: vec![
            "omena-lsp-server.style-diagnostics",
            "omena-streaming-ifds.cross-file-reachability-report",
            "omena-streaming-ifds.analysis-report",
            "omena-query.unified-cross-file-hypergraph",
            "omena-query.cross-file-summary",
        ],
        range: ParserRangeV0::default(),
        message: format!(
            "cross-file dataflow reaches {} module(s) via resolved edges; paths are omitted from diagnostics",
            report.reachable_foreign_path_count
        ),
        tags: Vec::new(),
        create_custom_property: None,
        cascade_narrowing: None,
        cascade_confidence: None,
        polynomial_provenance: None,
        cross_file_scc: None,
    }]
}
