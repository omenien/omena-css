use omena_cascade::DiagnosticFrameFootprintV0;
use omena_incremental::select_frame_aware_recheck_set;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FrameAwareRefreshReportV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub feature_gate: &'static str,
    pub selective_refresh_enabled: bool,
    pub edited_module_count: usize,
    pub selected_diagnostic_instance_ids: Vec<String>,
    pub skipped_diagnostic_instance_ids: Vec<String>,
    pub layer_marker: &'static str,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FrameAwareRefreshComparisonV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub fixed_workspace_fixture: &'static str,
    pub diagnostic_frame_count: usize,
    pub edited_module_count: usize,
    pub unconditional_selected_count: usize,
    pub selective_selected_count: usize,
    pub skipped_diagnostic_count: usize,
    pub selected_work_reduction_count: usize,
    pub selected_work_reduction_ratio: f64,
    pub measured_latency_proxy: &'static str,
    pub selective_refresh_enabled_by_default: bool,
    pub feature_gate: &'static str,
    pub disable_gate: &'static str,
    pub layer_marker: &'static str,
}

pub fn refresh_diagnostics_with_frame(
    frames: &[DiagnosticFrameFootprintV0],
    edited_module_ids: Vec<String>,
) -> FrameAwareRefreshReportV0 {
    let selective_refresh_enabled = frame_aware_refresh_enabled_from_env();
    refresh_diagnostics_with_frame_policy(frames, edited_module_ids, selective_refresh_enabled)
}

pub fn refresh_diagnostics_with_frame_policy(
    frames: &[DiagnosticFrameFootprintV0],
    edited_module_ids: Vec<String>,
    selective_refresh_enabled: bool,
) -> FrameAwareRefreshReportV0 {
    let edited_module_count = edited_module_ids.len();
    let selection = if selective_refresh_enabled {
        select_frame_aware_recheck_set(frames, edited_module_ids)
    } else {
        select_frame_aware_recheck_set(frames, frames_to_module_ids(frames))
    };

    FrameAwareRefreshReportV0 {
        schema_version: "0",
        product: "omena-lsp-server.frame-aware-refresh",
        feature_gate: "OMENA_LSP_ENABLE_FRAME_AWARE_REFRESH",
        selective_refresh_enabled,
        edited_module_count,
        selected_diagnostic_instance_ids: selection.selected_diagnostic_instance_ids,
        skipped_diagnostic_instance_ids: selection.skipped_diagnostic_instance_ids,
        layer_marker: "frame-rule",
    }
}

pub fn compare_frame_refresh_against_unconditional_baseline(
    frames: &[DiagnosticFrameFootprintV0],
    edited_module_ids: Vec<String>,
) -> FrameAwareRefreshComparisonV0 {
    let unconditional =
        refresh_diagnostics_with_frame_policy(frames, edited_module_ids.clone(), false);
    let selective = refresh_diagnostics_with_frame_policy(frames, edited_module_ids, true);
    let unconditional_selected_count = unconditional.selected_diagnostic_instance_ids.len();
    let selective_selected_count = selective.selected_diagnostic_instance_ids.len();
    let selected_work_reduction_count =
        unconditional_selected_count.saturating_sub(selective_selected_count);
    let selected_work_reduction_ratio = if unconditional_selected_count == 0 {
        0.0
    } else {
        selected_work_reduction_count as f64 / unconditional_selected_count as f64
    };

    FrameAwareRefreshComparisonV0 {
        schema_version: "0",
        product: "omena-lsp-server.frame-aware-refresh-comparison",
        fixed_workspace_fixture: "m4-alpha-fixed-workspace-frame-refresh",
        diagnostic_frame_count: frames.len(),
        edited_module_count: unconditional.edited_module_count,
        unconditional_selected_count,
        selective_selected_count,
        skipped_diagnostic_count: selective.skipped_diagnostic_instance_ids.len(),
        selected_work_reduction_count,
        selected_work_reduction_ratio,
        measured_latency_proxy: "selected-diagnostic-work-count",
        selective_refresh_enabled_by_default: frame_aware_refresh_enabled(false, false),
        feature_gate: "OMENA_LSP_ENABLE_FRAME_AWARE_REFRESH",
        disable_gate: "OMENA_LSP_DISABLE_FRAME_AWARE_REFRESH",
        layer_marker: "frame-rule",
    }
}

fn frames_to_module_ids(frames: &[DiagnosticFrameFootprintV0]) -> Vec<String> {
    frames
        .iter()
        .flat_map(|frame| frame.evidence_module_ids.iter().cloned())
        .collect()
}

fn frame_aware_refresh_enabled_from_env() -> bool {
    frame_aware_refresh_enabled(
        std::env::var_os("OMENA_LSP_ENABLE_FRAME_AWARE_REFRESH").is_some(),
        std::env::var_os("OMENA_LSP_DISABLE_FRAME_AWARE_REFRESH").is_some(),
    )
}

fn frame_aware_refresh_enabled(enable_requested: bool, disable_requested: bool) -> bool {
    enable_requested && !disable_requested
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
        assert_eq!(report.feature_gate, "OMENA_LSP_ENABLE_FRAME_AWARE_REFRESH");
        assert_eq!(report.selected_diagnostic_instance_ids, vec!["d1"]);
    }

    #[test]
    fn refresh_report_preserves_full_refresh_when_disabled() {
        let selected = derive_frame_for_diagnostic(
            "missing-static-class",
            "selected",
            vec!["file:///workspace/a.module.css".to_string()],
        );
        let otherwise_skipped = derive_frame_for_diagnostic(
            "missing-static-class",
            "otherwise-skipped",
            vec!["file:///workspace/b.module.css".to_string()],
        );
        let report = refresh_diagnostics_with_frame_policy(
            &[selected, otherwise_skipped],
            vec!["file:///workspace/a.module.css".to_string()],
            false,
        );

        assert!(!report.selective_refresh_enabled);
        assert_eq!(
            report.selected_diagnostic_instance_ids,
            vec!["selected", "otherwise-skipped"]
        );
        assert!(report.skipped_diagnostic_instance_ids.is_empty());
    }

    #[test]
    fn refresh_policy_is_positive_opt_in() {
        assert!(!frame_aware_refresh_enabled(false, false));
        assert!(frame_aware_refresh_enabled(true, false));
        assert!(!frame_aware_refresh_enabled(true, true));
        assert!(!frame_aware_refresh_enabled(false, true));
    }

    #[test]
    fn fixed_workspace_comparison_reports_work_reduction_when_enabled() {
        let frames = (0..100)
            .map(|index| {
                derive_frame_for_diagnostic(
                    "missing-static-class",
                    format!("d{index}"),
                    vec![format!("file:///workspace/module-{index}.module.css")],
                )
            })
            .collect::<Vec<_>>();
        let report = compare_frame_refresh_against_unconditional_baseline(
            &frames,
            vec!["file:///workspace/module-7.module.css".to_string()],
        );

        assert_eq!(report.schema_version, "0");
        assert_eq!(report.layer_marker, "frame-rule");
        assert_eq!(report.feature_gate, "OMENA_LSP_ENABLE_FRAME_AWARE_REFRESH");
        assert!(!report.selective_refresh_enabled_by_default);
        assert_eq!(report.diagnostic_frame_count, 100);
        assert_eq!(report.unconditional_selected_count, 100);
        assert_eq!(report.selective_selected_count, 1);
        assert_eq!(report.skipped_diagnostic_count, 99);
        assert!(report.selected_work_reduction_ratio >= 0.99);
    }
}
