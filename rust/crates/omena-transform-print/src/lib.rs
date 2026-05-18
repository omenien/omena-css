//! CSS emission and source-map boundary for Omena CSS transforms.
//!
//! The initial printer is deliberately identity-preserving. It establishes the
//! output and source-map contract before later pretty/minified emitters start
//! changing bytes.

use omena_transform_cst::{
    StyleDialect, TransformCstArtifactV0, TransformPassKind,
    build_transform_cst_artifact_with_dialect,
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
        source_map_contract: "stable-IR provenance-anchor identity segments with lexical identity fallback plus mutation-span segments",
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
    print_transform_cst_source_with_dialect(
        source_path,
        source,
        StyleDialect::Css,
        semantic_signature,
        upstream_passes,
        options,
    )
}

pub fn print_transform_cst_source_with_dialect(
    source_path: impl Into<String>,
    source: &str,
    dialect: StyleDialect,
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
    let cst_artifact =
        build_transform_cst_artifact_with_dialect(source, dialect, semantic_signature, &passes);
    let css = render_identity_preserving_css(source, options.mode);
    let source_map_segments = if options.include_source_map {
        compose_identity_source_map_segments(
            &source_path,
            source,
            &css,
            &cst_artifact,
            TransformPassKind::PrintCss.id(),
        )
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
    print_transform_execution_artifact_with_dialect(
        source_path,
        StyleDialect::Css,
        semantic_signature,
        upstream_passes,
        options,
        execution,
    )
}

pub fn print_transform_execution_artifact_with_dialect(
    source_path: impl Into<String>,
    dialect: StyleDialect,
    semantic_signature: impl Into<String>,
    upstream_passes: &[TransformPassKind],
    options: TransformPrintOptionsV0,
    execution: &TransformExecutionSummaryV0,
) -> TransformPrintArtifactV0 {
    let source_path = source_path.into();
    let mut artifact = print_transform_cst_source_with_dialect(
        source_path.clone(),
        &execution.output_css,
        dialect,
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
        .flat_map(|node| {
            let spans = if node.mutation_spans.is_empty() {
                vec![(
                    node.source_span_start,
                    node.source_span_end,
                    node.generated_span_start,
                    node.generated_span_end,
                )]
            } else {
                node.mutation_spans
                    .iter()
                    .map(|span| {
                        (
                            span.source_span_start,
                            span.source_span_end,
                            span.generated_span_start,
                            span.generated_span_end,
                        )
                    })
                    .collect::<Vec<_>>()
            };

            spans.into_iter().map(
                |(original_start, original_end, generated_start, generated_end)| {
                    TransformSourceMapSegmentV0 {
                        source_path: source_path.to_string(),
                        original_start,
                        original_end,
                        generated_start,
                        generated_end,
                        pass_id: node.pass_id,
                    }
                },
            )
        })
        .collect()
}

fn compose_identity_source_map_segments(
    source_path: &str,
    source: &str,
    generated: &str,
    cst_artifact: &TransformCstArtifactV0,
    pass_id: &'static str,
) -> Vec<TransformSourceMapSegmentV0> {
    let anchor_segments = cst_artifact
        .stable_ir
        .provenance_anchors
        .iter()
        .filter(|anchor| anchor.source_span_start <= anchor.source_span_end)
        .map(|anchor| TransformSourceMapSegmentV0 {
            source_path: source_path.to_string(),
            original_start: anchor.source_span_start,
            original_end: anchor.source_span_end,
            generated_start: anchor.source_span_start.min(generated.len()),
            generated_end: anchor.source_span_end.min(generated.len()),
            pass_id,
        })
        .collect::<Vec<_>>();
    if !anchor_segments.is_empty() {
        return anchor_segments;
    }

    if source.is_empty() {
        return vec![TransformSourceMapSegmentV0 {
            source_path: source_path.to_string(),
            original_start: 0,
            original_end: 0,
            generated_start: 0,
            generated_end: 0,
            pass_id,
        }];
    }

    let mut segments = Vec::new();
    let mut segment_start = None;
    for (index, byte) in source.bytes().enumerate() {
        if byte.is_ascii_whitespace() {
            if let Some(start) = segment_start.take() {
                segments.push(TransformSourceMapSegmentV0 {
                    source_path: source_path.to_string(),
                    original_start: start,
                    original_end: index,
                    generated_start: start.min(generated.len()),
                    generated_end: index.min(generated.len()),
                    pass_id,
                });
            }
        } else if segment_start.is_none() {
            segment_start = Some(index);
        }
    }

    if let Some(start) = segment_start {
        segments.push(TransformSourceMapSegmentV0 {
            source_path: source_path.to_string(),
            original_start: start,
            original_end: source.len(),
            generated_start: start.min(generated.len()),
            generated_end: generated.len(),
            pass_id,
        });
    }

    segments
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
    fn exposes_print_boundary() {
        let boundary = summarize_omena_transform_print_boundary();

        assert_eq!(boundary.product, "omena-transform-print.boundary");
        assert_eq!(boundary.emission_pass_id, "print-css");
        assert_eq!(boundary.supported_modes.len(), 3);
    }

    #[test]
    fn prints_identity_css_with_stable_ir_source_map_segments() {
        let source = ".button { color: var(--brand); background: red; }";
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
        assert!(artifact.source_map_segments.len() > 1);
        if artifact
            .cst_artifact
            .stable_ir
            .provenance_anchors
            .is_empty()
        {
            assert!(artifact.source_map_segments.len() > 1);
        } else {
            assert_eq!(
                artifact.source_map_segments.len(),
                artifact.cst_artifact.stable_ir.provenance_anchors.len()
            );
        }
        assert!(
            artifact
                .source_map_segments
                .iter()
                .any(|segment| &source[segment.original_start..segment.original_end] == "button")
        );
        let expected_last_original_end = artifact
            .cst_artifact
            .stable_ir
            .provenance_anchors
            .last()
            .map(|anchor| anchor.source_span_end)
            .unwrap_or(source.len());
        assert_eq!(
            artifact
                .source_map_segments
                .last()
                .map(|segment| segment.original_end),
            Some(expected_last_original_end)
        );
        assert_eq!(
            artifact.pass_plan.ordered_pass_ids,
            vec!["calc-reduction", "print-css"]
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
        assert_eq!(
            artifact.source_map_segments[0].original_start,
            execution.provenance_derivation_forest.nodes[0].source_span_start
        );
        assert_eq!(
            artifact.source_map_segments[0].original_end,
            execution.provenance_derivation_forest.nodes[0].source_span_end
        );
        assert_eq!(
            artifact.source_map_segments[0].generated_start,
            execution.provenance_derivation_forest.nodes[0].generated_span_start
        );
        assert_eq!(
            artifact.source_map_segments[0].generated_end,
            execution.provenance_derivation_forest.nodes[0].generated_span_end
        );
        assert!(
            artifact
                .source_map_segments
                .iter()
                .any(|segment| segment.pass_id == "comment-strip")
        );
        assert_eq!(
            artifact
                .source_map_segments
                .last()
                .map(|segment| segment.generated_end),
            Some(execution.output_byte_len)
        );
    }

    #[test]
    fn emits_one_source_map_segment_per_mutation_span() {
        let source = ".a { /* one */ color: red; }\n.b { /* two */ color: blue; }";
        let execution = execute_transform_passes_on_source(
            source,
            &[TransformPassKind::CommentStrip, TransformPassKind::PrintCss],
        );
        let comment_node = execution
            .provenance_derivation_forest
            .nodes
            .iter()
            .find(|node| node.pass_id == "comment-strip");
        assert!(comment_node.is_some());
        if let Some(comment_node) = comment_node {
            assert_eq!(comment_node.mutation_spans.len(), 2);
        }

        let artifact = print_transform_execution_artifact(
            "Multi.module.css",
            "semantic:multi",
            &[TransformPassKind::CommentStrip, TransformPassKind::PrintCss],
            default_print_options(),
            &execution,
        );
        let comment_segments = artifact
            .source_map_segments
            .iter()
            .filter(|segment| segment.pass_id == "comment-strip")
            .collect::<Vec<_>>();

        assert_eq!(comment_segments.len(), 2);
        assert!(comment_segments[0].original_start < comment_segments[1].original_start);
    }
}
