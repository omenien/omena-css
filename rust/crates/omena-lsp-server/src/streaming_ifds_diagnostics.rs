use omena_query::{
    OmenaQueryCrossFileSummaryV0, OmenaQueryStyleDiagnosticV0, ParserRangeV0,
    summarize_omena_query_unified_cross_file_hypergraph,
};
use omena_streaming_ifds::{
    StreamingIFDSCrossFileReachabilityReportV0, StreamingIfdsReachabilityCondensationV0,
    streaming_ifds_reachability_condensation_v0,
    summarize_streaming_ifds_cross_file_reachability_v0,
    summarize_streaming_ifds_cross_file_reachability_with_condensation_v0,
};

/// Target-INDEPENDENT reachability state shared across a wave (rfcs#111, the
/// first C1 slice): the unified hypergraph derivation and its SCC
/// condensation, built once per committed graph instead of once per target.
#[derive(Debug)]
pub(crate) struct SharedStreamingReachabilityV0 {
    condensation: StreamingIfdsReachabilityCondensationV0,
}

pub(crate) fn shared_streaming_reachability_for_lsp(
    committed_cross_file_summary: &OmenaQueryCrossFileSummaryV0,
) -> SharedStreamingReachabilityV0 {
    let hypergraph =
        summarize_omena_query_unified_cross_file_hypergraph(committed_cross_file_summary);
    SharedStreamingReachabilityV0 {
        condensation: streaming_ifds_reachability_condensation_v0(hypergraph.hyperedges.as_slice()),
    }
}

pub(crate) fn summarize_cross_file_streaming_reachability_diagnostics_for_lsp(
    target_style_path: &str,
    committed_cross_file_summary: &OmenaQueryCrossFileSummaryV0,
) -> Vec<OmenaQueryStyleDiagnosticV0> {
    let hypergraph =
        summarize_omena_query_unified_cross_file_hypergraph(committed_cross_file_summary);
    let report = summarize_streaming_ifds_cross_file_reachability_v0(
        target_style_path,
        hypergraph.hyperedges.as_slice(),
    );
    reachability_report_diagnostics(report)
}

/// The shared-condensation arm — byte-identical to the per-call arm above
/// (gated by the parity test in omena-streaming-ifds).
pub(crate) fn summarize_cross_file_streaming_reachability_diagnostics_for_lsp_shared(
    target_style_path: &str,
    shared: &SharedStreamingReachabilityV0,
) -> Vec<OmenaQueryStyleDiagnosticV0> {
    let report = summarize_streaming_ifds_cross_file_reachability_with_condensation_v0(
        target_style_path,
        &shared.condensation,
    );
    reachability_report_diagnostics(report)
}

fn reachability_report_diagnostics(
    report: StreamingIFDSCrossFileReachabilityReportV0,
) -> Vec<OmenaQueryStyleDiagnosticV0> {
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
