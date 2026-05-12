//! P40 emission and source-map boundary for Omena CSS transforms.
//!
//! The initial printer is deliberately identity-preserving. It establishes the
//! output and source-map contract before later pretty/minified emitters start
//! changing bytes.

use omena_transform_cst::{
    TransformCstArtifactV0, TransformPassKind, build_transform_cst_artifact,
};
use omena_transform_passes::{
    TransformExecutionSummaryV0, TransformPassPlanV0, plan_transform_passes,
};
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum TransformPrintMode {
    Identity,
    Pretty,
    Minified,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformPrintOptionsV0 {
    pub mode: TransformPrintMode,
    pub include_source_map: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformSourceMapSegmentV0 {
    pub source_path: String,
    pub original_start: usize,
    pub original_end: usize,
    pub generated_start: usize,
    pub generated_end: usize,
    pub pass_id: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformPrintBoundarySummaryV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub emission_pass_id: &'static str,
    pub supported_modes: Vec<TransformPrintMode>,
    pub source_map_contract: &'static str,
    pub planner_surface: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformPrintArtifactV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub source_path: String,
    pub css: String,
    pub source_map_segments: Vec<TransformSourceMapSegmentV0>,
    pub cst_artifact: TransformCstArtifactV0,
    pub pass_plan: TransformPassPlanV0,
    pub provenance_preserved: bool,
}

pub fn summarize_omena_transform_print_boundary() -> TransformPrintBoundarySummaryV0 {
    TransformPrintBoundarySummaryV0 {
        schema_version: "0",
        product: "omena-transform-print.boundary",
        emission_pass_id: TransformPassKind::PrintCss.id(),
        supported_modes: vec![
            TransformPrintMode::Identity,
            TransformPrintMode::Pretty,
            TransformPrintMode::Minified,
        ],
        source_map_contract: "one segment per emitted source range until mutation printers split spans",
        planner_surface: "omena-transform-passes.plan",
    }
}

pub fn print_transform_cst_source(
    source_path: impl Into<String>,
    source: &str,
    semantic_signature: impl Into<String>,
    upstream_passes: &[TransformPassKind],
    options: TransformPrintOptionsV0,
) -> TransformPrintArtifactV0 {
    let source_path = source_path.into();
    let mut passes = upstream_passes.to_vec();
    if !passes.contains(&TransformPassKind::PrintCss) {
        passes.push(TransformPassKind::PrintCss);
    }
    let pass_plan = plan_transform_passes(&passes);
    let cst_artifact = build_transform_cst_artifact(source, semantic_signature, &passes);
    let css = render_identity_preserving_css(source, options.mode);
    let source_map_segments = if options.include_source_map {
        vec![TransformSourceMapSegmentV0 {
            source_path: source_path.clone(),
            original_start: 0,
            original_end: source.len(),
            generated_start: 0,
            generated_end: css.len(),
            pass_id: TransformPassKind::PrintCss.id(),
        }]
    } else {
        Vec::new()
    };

    TransformPrintArtifactV0 {
        schema_version: "0",
        product: "omena-transform-print.artifact",
        source_path,
        css,
        source_map_segments,
        cst_artifact,
        pass_plan,
        provenance_preserved: true,
    }
}

pub fn print_transform_execution_artifact(
    source_path: impl Into<String>,
    semantic_signature: impl Into<String>,
    upstream_passes: &[TransformPassKind],
    options: TransformPrintOptionsV0,
    execution: &TransformExecutionSummaryV0,
) -> TransformPrintArtifactV0 {
    let source_path = source_path.into();
    let mut artifact = print_transform_cst_source(
        source_path.clone(),
        &execution.output_css,
        semantic_signature,
        upstream_passes,
        options,
    );

    if options.include_source_map {
        artifact.source_map_segments =
            compose_source_map_segments_from_execution(&source_path, execution);
    }

    artifact.provenance_preserved = artifact.provenance_preserved && execution.provenance_preserved;
    artifact
}

pub const fn default_print_options() -> TransformPrintOptionsV0 {
    TransformPrintOptionsV0 {
        mode: TransformPrintMode::Identity,
        include_source_map: true,
    }
}

fn compose_source_map_segments_from_execution(
    source_path: &str,
    execution: &TransformExecutionSummaryV0,
) -> Vec<TransformSourceMapSegmentV0> {
    execution
        .provenance_derivation_forest
        .nodes
        .iter()
        .map(|node| TransformSourceMapSegmentV0 {
            source_path: source_path.to_string(),
            original_start: 0,
            original_end: node.input_byte_len,
            generated_start: 0,
            generated_end: node.output_byte_len,
            pass_id: node.pass_id,
        })
        .collect()
}

fn render_identity_preserving_css(source: &str, _mode: TransformPrintMode) -> String {
    source.to_string()
}

#[cfg(test)]
mod tests {
    use super::{
        default_print_options, print_transform_cst_source, print_transform_execution_artifact,
        summarize_omena_transform_print_boundary,
    };
    use omena_transform_cst::TransformPassKind;
    use omena_transform_passes::execute_transform_passes_on_source;

    #[test]
    fn exposes_p40_print_boundary() {
        let boundary = summarize_omena_transform_print_boundary();

        assert_eq!(boundary.product, "omena-transform-print.boundary");
        assert_eq!(boundary.emission_pass_id, "p40-print-css");
        assert_eq!(boundary.supported_modes.len(), 3);
    }

    #[test]
    fn prints_identity_css_with_full_source_map_segment() {
        let source = ".button { color: var(--brand); }";
        let artifact = print_transform_cst_source(
            "Button.module.css",
            source,
            "semantic:button:brand",
            &[TransformPassKind::CalcReduction],
            default_print_options(),
        );

        assert_eq!(artifact.product, "omena-transform-print.artifact");
        assert_eq!(artifact.css, source);
        assert!(artifact.provenance_preserved);
        assert_eq!(artifact.source_map_segments.len(), 1);
        assert_eq!(artifact.source_map_segments[0].original_end, source.len());
        assert_eq!(
            artifact.pass_plan.ordered_pass_ids,
            vec!["p25-calc-reduction", "p40-print-css"]
        );
    }

    #[test]
    fn composes_source_map_segments_from_execution_provenance() {
        let source = ".button { color: red; /* remove */ }";
        let execution = execute_transform_passes_on_source(
            source,
            &[
                TransformPassKind::CommentStrip,
                TransformPassKind::WhitespaceStrip,
                TransformPassKind::PrintCss,
            ],
        );
        let artifact = print_transform_execution_artifact(
            "Button.module.css",
            "semantic:button",
            &[
                TransformPassKind::CommentStrip,
                TransformPassKind::WhitespaceStrip,
                TransformPassKind::PrintCss,
            ],
            default_print_options(),
            &execution,
        );

        assert_eq!(artifact.css, execution.output_css);
        assert!(artifact.provenance_preserved);
        assert_eq!(
            artifact.source_map_segments.len(),
            execution.provenance_derivation_forest.node_count
        );
        assert_eq!(
            artifact.source_map_segments[0].source_path,
            "Button.module.css"
        );
        assert!(
            artifact
                .source_map_segments
                .iter()
                .any(|segment| segment.pass_id == "p02-comment-strip")
        );
        assert_eq!(
            artifact
                .source_map_segments
                .last()
                .map(|segment| segment.generated_end),
            Some(execution.output_byte_len)
        );
    }
}
