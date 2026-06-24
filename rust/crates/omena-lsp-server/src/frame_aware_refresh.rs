use std::collections::{BTreeMap, BTreeSet};

use omena_cascade::{
    DiagnosticFrameFootprintV0, RecheckSelectionV0, compute_edit_footprint, select_recheck_set,
};
use omena_incremental::{
    IncrementalGraphInputV0, IncrementalNodeInputV0, IncrementalRevisionV0,
    OmenaIncrementalDatabaseV0,
};
use omena_parser::{ParseReuseCache, StyleDialect, facts_from_cst, parse_with_reuse_cache};
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrameAwareStyleModuleInputV0 {
    pub module_id: String,
    pub source: String,
    pub dialect: StyleDialect,
}

#[derive(Default)]
pub struct FrameAwareRefreshRuntimeV0 {
    revision: u64,
    incremental_database: OmenaIncrementalDatabaseV0,
    parse_caches_by_module_id: BTreeMap<String, ParseReuseCache>,
}

impl FrameAwareRefreshRuntimeV0 {
    pub fn refresh_diagnostics_with_style_modules_policy(
        &mut self,
        frames: &[DiagnosticFrameFootprintV0],
        modules: &[FrameAwareStyleModuleInputV0],
        selective_refresh_enabled: bool,
    ) -> FrameAwareRefreshReportV0 {
        let graph = self.frame_refresh_graph_input(modules);
        let update = self
            .incremental_database
            .plan_and_upsert_graph_input(&graph);
        let dirty_module_ids = update
            .incremental_plan
            .nodes
            .iter()
            .filter(|node| node.dirty)
            .map(|node| node.id.clone())
            .collect::<Vec<_>>();
        let edited_module_count = dirty_module_ids.len();
        let selection = if selective_refresh_enabled {
            select_recheck_set_for_module_ids(frames, dirty_module_ids)
        } else {
            select_recheck_set_for_module_ids(frames, frames_to_module_ids(frames))
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

    fn frame_refresh_graph_input(
        &mut self,
        modules: &[FrameAwareStyleModuleInputV0],
    ) -> IncrementalGraphInputV0 {
        self.revision = self.revision.saturating_add(1);
        IncrementalGraphInputV0 {
            revision: IncrementalRevisionV0 {
                value: self.revision,
            },
            nodes: modules
                .iter()
                .map(|module| self.frame_refresh_node_input(module))
                .collect(),
        }
    }

    fn frame_refresh_node_input(
        &mut self,
        module: &FrameAwareStyleModuleInputV0,
    ) -> IncrementalNodeInputV0 {
        let cache = self
            .parse_caches_by_module_id
            .entry(module.module_id.clone())
            .or_default();
        let parsed = parse_with_reuse_cache(module.source.as_str(), module.dialect, cache);
        let facts = facts_from_cst(module.source.as_str(), &parsed);
        let dependency_ids = frame_refresh_dependency_ids(&facts);
        let digest = frame_refresh_digest(module, &parsed, &dependency_ids);

        IncrementalNodeInputV0 {
            id: module.module_id.clone(),
            digest,
            dependency_ids,
        }
    }
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
        select_recheck_set_for_module_ids(frames, edited_module_ids)
    } else {
        select_recheck_set_for_module_ids(frames, frames_to_module_ids(frames))
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

fn select_recheck_set_for_module_ids(
    frames: &[DiagnosticFrameFootprintV0],
    module_ids: Vec<String>,
) -> RecheckSelectionV0 {
    let footprint = compute_edit_footprint(module_ids);
    select_recheck_set(frames, &footprint)
}

fn frame_refresh_dependency_ids(facts: &omena_parser::ParsedStyleFacts) -> Vec<String> {
    let mut dependency_ids = BTreeSet::new();
    for edge in &facts.sass_module_edges {
        dependency_ids.insert(edge.source.clone());
    }
    for edge in &facts.css_module_value_import_edges {
        dependency_ids.insert(edge.import_source.clone());
    }
    for edge in &facts.css_module_composes_edges {
        if let Some(import_source) = &edge.import_source {
            dependency_ids.insert(import_source.clone());
        }
    }
    for edge in &facts.icss_import_edges {
        dependency_ids.insert(edge.import_source.clone());
    }
    dependency_ids.into_iter().collect()
}

fn frame_refresh_digest(
    module: &FrameAwareStyleModuleInputV0,
    parsed: &omena_parser::ParseResult,
    dependency_ids: &[String],
) -> String {
    frame_refresh_stable_hash_hex(
        format!(
            "source={};tokens={};errors={};deps={}",
            module.source,
            parsed.token_count(),
            parsed.errors().len(),
            dependency_ids.join(",")
        )
        .as_bytes(),
    )
}

fn frame_refresh_stable_hash_hex(bytes: &[u8]) -> String {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("{hash:016x}")
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

    #[test]
    fn runtime_refresh_uses_parser_facts_and_salsa_dirty_dependencies() {
        let mut runtime = FrameAwareRefreshRuntimeV0::default();
        let frames = vec![
            derive_frame_for_diagnostic(
                "missing-static-class",
                "a-diagnostic",
                vec!["a".to_string()],
            ),
            derive_frame_for_diagnostic(
                "missing-static-class",
                "b-diagnostic",
                vec!["b".to_string()],
            ),
        ];
        let initial = vec![
            FrameAwareStyleModuleInputV0 {
                module_id: "a".to_string(),
                source: ".a { color: red; }".to_string(),
                dialect: StyleDialect::Scss,
            },
            FrameAwareStyleModuleInputV0 {
                module_id: "b".to_string(),
                source: "@use \"a\"; .b { color: blue; }".to_string(),
                dialect: StyleDialect::Scss,
            },
        ];
        let first = runtime.refresh_diagnostics_with_style_modules_policy(&frames, &initial, true);
        assert_eq!(
            first.selected_diagnostic_instance_ids,
            vec!["a-diagnostic", "b-diagnostic"]
        );

        let changed = vec![
            FrameAwareStyleModuleInputV0 {
                module_id: "a".to_string(),
                source: ".a { color: green; }".to_string(),
                dialect: StyleDialect::Scss,
            },
            FrameAwareStyleModuleInputV0 {
                module_id: "b".to_string(),
                source: "@use \"a\"; .b { color: blue; }".to_string(),
                dialect: StyleDialect::Scss,
            },
        ];
        let second = runtime.refresh_diagnostics_with_style_modules_policy(&frames, &changed, true);

        assert_eq!(second.edited_module_count, 2);
        assert_eq!(
            second.selected_diagnostic_instance_ids,
            vec!["a-diagnostic", "b-diagnostic"]
        );
    }
}
