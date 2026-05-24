use omena_cascade::DiagnosticFrameFootprintV0;
use omena_incremental::select_frame_aware_recheck_set;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FrameAwareRefreshReportV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub selective_refresh_enabled: bool,
    pub edited_module_count: usize,
    pub selected_diagnostic_instance_ids: Vec<String>,
    pub skipped_diagnostic_instance_ids: Vec<String>,
    pub layer_marker: &'static str,
}

pub fn refresh_diagnostics_with_frame(
    frames: &[DiagnosticFrameFootprintV0],
    edited_module_ids: Vec<String>,
) -> FrameAwareRefreshReportV0 {
    let selective_refresh_enabled =
        std::env::var_os("OMENA_LSP_DISABLE_FRAME_AWARE_REFRESH").is_none();
    let edited_module_count = edited_module_ids.len();
    let selection = select_frame_aware_recheck_set(frames, edited_module_ids);

    FrameAwareRefreshReportV0 {
        schema_version: "0",
        product: "omena-lsp-server.frame-aware-refresh",
        selective_refresh_enabled,
        edited_module_count,
        selected_diagnostic_instance_ids: selection.selected_diagnostic_instance_ids,
        skipped_diagnostic_instance_ids: selection.skipped_diagnostic_instance_ids,
        layer_marker: "frame-rule",
    }
}

#[cfg(test)]
mod tests {
    use omena_cascade::derive_frame_for_diagnostic;

    use super::*;

    #[test]
    fn refresh_report_selects_only_intersecting_frames() {
        let frame = derive_frame_for_diagnostic(
            "missing-static-class",
            "d1",
            vec!["file:///workspace/a.module.css".to_string()],
        );
        let report = refresh_diagnostics_with_frame(
            &[frame],
            vec!["file:///workspace/a.module.css".to_string()],
        );

        assert_eq!(report.schema_version, "0");
        assert_eq!(report.layer_marker, "frame-rule");
        assert_eq!(report.selected_diagnostic_instance_ids, vec!["d1"]);
    }
}
