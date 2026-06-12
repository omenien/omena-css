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
    TransformExecutionSummaryV0, TransformPassPlanV0,
    execute_transform_passes_on_source_with_dialect, plan_transform_passes,
};
use serde::{Deserialize, Serialize};

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
pub struct TransformSourceMapPointV0 {
    pub byte_offset: usize,
    pub line: usize,
    pub utf8_column: usize,
    pub utf16_column: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformSourceMapSegmentV0 {
    pub source_path: String,
    pub original_start: usize,
    pub original_end: usize,
    pub generated_start: usize,
    pub generated_end: usize,
    pub original_start_point: TransformSourceMapPointV0,
    pub original_end_point: TransformSourceMapPointV0,
    pub generated_start_point: TransformSourceMapPointV0,
    pub generated_end_point: TransformSourceMapPointV0,
    pub pass_id: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformSourceMapV3V0 {
    pub version: u8,
    pub file: String,
    pub sources: Vec<String>,
    #[serde(rename = "sourcesContent")]
    pub sources_content: Vec<String>,
    pub names: Vec<String>,
    pub mappings: String,
    #[serde(rename = "x_omenaSchemaVersion")]
    pub x_omena_schema_version: &'static str,
    #[serde(rename = "x_omenaProduct")]
    pub x_omena_product: &'static str,
    #[serde(rename = "x_omenaSegmentCount")]
    pub x_omena_segment_count: usize,
    #[serde(rename = "x_omenaPassIds")]
    pub x_omena_pass_ids: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedTransformSourceMapV3V0 {
    pub version: u8,
    pub file: Option<String>,
    pub sources: Vec<String>,
    pub sources_content: Vec<Option<String>>,
    pub names: Vec<String>,
    pub mappings: String,
    pub decoded_segments: Vec<DecodedSourceMapSegmentV0>,
    pub generated_line_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodedSourceMapSegmentV0 {
    pub generated_line: usize,
    pub generated_utf16_column: usize,
    pub source_index: usize,
    pub original_line: usize,
    pub original_utf16_column: usize,
    pub name_index: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransformSourceMapCompositionV0 {
    pub source_map: TransformSourceMapV3V0,
    pub upstream_map_applied: bool,
    pub fallback_reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransformSourceMapV3ParseErrorV0 {
    Json(String),
    UnsupportedVersion(u8),
    InvalidMapping(String),
    SourceIndexOutOfRange { index: usize, source_count: usize },
    NameIndexOutOfRange { index: usize, name_count: usize },
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawSourceMapV3V0 {
    version: u8,
    #[serde(default)]
    file: Option<String>,
    #[serde(default)]
    sources: Vec<String>,
    #[serde(rename = "sourcesContent", default)]
    sources_content: Vec<Option<String>>,
    #[serde(default)]
    names: Vec<String>,
    mappings: String,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_map_v3: Option<TransformSourceMapV3V0>,
    pub cst_artifact: TransformCstArtifactV0,
    pub pass_plan: TransformPassPlanV0,
    pub provenance_preserved: bool,
}

pub fn summarize_omena_transform_print_boundary() -> TransformPrintBoundarySummaryV0 {
    TransformPrintBoundarySummaryV0 {
        schema_version: "0",
        product: "omena-transform-print.boundary",
        emission_pass_id: TransformPassKind::PrintCss.id(),
        supported_modes: vec![TransformPrintMode::Identity, TransformPrintMode::Minified],
        source_map_contract: "stable-IR provenance-anchor emission segments with byte offsets, UTF-8/UTF-16 line-column points, lexical identity fallback, minified deletion projection, mutation-span segments, and Source Map V3 mappings serialization",
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
    let rendered = render_css_for_print_mode(source, dialect, options.mode);
    let css = rendered.css;
    let source_map_segments = if options.include_source_map {
        compose_identity_source_map_segments(
            &source_path,
            source,
            &css,
            &cst_artifact,
            TransformPassKind::PrintCss.id(),
            rendered.generated_offset_lookup.as_deref(),
        )
    } else {
        Vec::new()
    };
    let source_map_v3 = options.include_source_map.then(|| {
        serialize_transform_source_map_v3(
            &source_path,
            &css,
            &source_path,
            Some(source),
            source_map_segments.as_slice(),
        )
    });

    TransformPrintArtifactV0 {
        schema_version: "0",
        product: "omena-transform-print.artifact",
        source_path,
        css,
        source_map_segments,
        source_map_v3,
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

pub fn print_transform_execution_artifact_with_source(
    source_path: impl Into<String>,
    original_source: &str,
    semantic_signature: impl Into<String>,
    upstream_passes: &[TransformPassKind],
    options: TransformPrintOptionsV0,
    execution: &TransformExecutionSummaryV0,
) -> TransformPrintArtifactV0 {
    print_transform_execution_artifact_with_dialect_and_source(
        source_path,
        original_source,
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
        artifact.source_map_segments = compose_source_map_segments_from_execution(
            &source_path,
            &execution.output_css,
            &artifact.css,
            execution,
            generated_offset_lookup_for_print_mode(&execution.output_css, &artifact.css),
        );
        artifact.source_map_v3 = Some(serialize_transform_source_map_v3(
            &source_path,
            &artifact.css,
            &source_path,
            Some(&execution.output_css),
            artifact.source_map_segments.as_slice(),
        ));
    }

    artifact.provenance_preserved = artifact.provenance_preserved && execution.provenance_preserved;
    artifact
}

pub fn print_transform_execution_artifact_with_dialect_and_source(
    source_path: impl Into<String>,
    original_source: &str,
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
        artifact.source_map_segments = compose_source_map_segments_from_execution(
            &source_path,
            original_source,
            &artifact.css,
            execution,
            generated_offset_lookup_for_print_mode(&execution.output_css, &artifact.css),
        );
        artifact.source_map_v3 = Some(serialize_transform_source_map_v3(
            &source_path,
            &artifact.css,
            &source_path,
            Some(original_source),
            artifact.source_map_segments.as_slice(),
        ));
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
    original_source: &str,
    generated_source: &str,
    execution: &TransformExecutionSummaryV0,
    generated_offset_lookup: Option<Vec<usize>>,
) -> Vec<TransformSourceMapSegmentV0> {
    let generated_offset_lookup = generated_offset_lookup.as_deref();
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
                    let generated_start =
                        generated_offset_lookup.map_or(generated_start, |lookup| {
                            project_generated_offset(
                                generated_start,
                                generated_source.len(),
                                Some(lookup),
                            )
                        });
                    let generated_end = generated_offset_lookup.map_or(generated_end, |lookup| {
                        project_generated_offset(
                            generated_end,
                            generated_source.len(),
                            Some(lookup),
                        )
                    });
                    source_map_segment(
                        source_path,
                        SourceMapSources {
                            original: original_source,
                            generated: generated_source,
                        },
                        SourceMapSpanOffsets {
                            original_start,
                            original_end,
                            generated_start,
                            generated_end,
                        },
                        node.pass_id,
                    )
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
    generated_offset_lookup: Option<&[usize]>,
) -> Vec<TransformSourceMapSegmentV0> {
    let anchor_segments = cst_artifact
        .stable_ir
        .provenance_anchors
        .iter()
        .filter(|anchor| anchor.source_span_start <= anchor.source_span_end)
        .map(|anchor| {
            source_map_segment(
                source_path,
                SourceMapSources {
                    original: source,
                    generated,
                },
                SourceMapSpanOffsets {
                    original_start: anchor.source_span_start,
                    original_end: anchor.source_span_end,
                    generated_start: project_generated_offset(
                        anchor.source_span_start,
                        generated.len(),
                        generated_offset_lookup,
                    ),
                    generated_end: project_generated_offset(
                        anchor.source_span_end,
                        generated.len(),
                        generated_offset_lookup,
                    ),
                },
                pass_id,
            )
        })
        .collect::<Vec<_>>();
    if !anchor_segments.is_empty() {
        return anchor_segments;
    }

    if source.is_empty() {
        return vec![source_map_segment(
            source_path,
            SourceMapSources {
                original: source,
                generated,
            },
            SourceMapSpanOffsets {
                original_start: 0,
                original_end: 0,
                generated_start: project_generated_offset(
                    0,
                    generated.len(),
                    generated_offset_lookup,
                ),
                generated_end: project_generated_offset(
                    0,
                    generated.len(),
                    generated_offset_lookup,
                ),
            },
            pass_id,
        )];
    }

    let mut segments = Vec::new();
    let mut segment_start = None;
    for (index, byte) in source.bytes().enumerate() {
        if byte.is_ascii_whitespace() {
            if let Some(start) = segment_start.take() {
                segments.push(source_map_segment(
                    source_path,
                    SourceMapSources {
                        original: source,
                        generated,
                    },
                    SourceMapSpanOffsets {
                        original_start: start,
                        original_end: index,
                        generated_start: project_generated_offset(
                            start,
                            generated.len(),
                            generated_offset_lookup,
                        ),
                        generated_end: project_generated_offset(
                            index,
                            generated.len(),
                            generated_offset_lookup,
                        ),
                    },
                    pass_id,
                ));
            }
        } else if segment_start.is_none() {
            segment_start = Some(index);
        }
    }

    if let Some(start) = segment_start {
        segments.push(source_map_segment(
            source_path,
            SourceMapSources {
                original: source,
                generated,
            },
            SourceMapSpanOffsets {
                original_start: start,
                original_end: source.len(),
                generated_start: project_generated_offset(
                    start,
                    generated.len(),
                    generated_offset_lookup,
                ),
                generated_end: project_generated_offset(
                    source.len(),
                    generated.len(),
                    generated_offset_lookup,
                ),
            },
            pass_id,
        ));
    }

    segments
}

pub fn serialize_transform_source_map_v3(
    file: impl Into<String>,
    generated_css: &str,
    source_path: impl Into<String>,
    source_content: Option<&str>,
    segments: &[TransformSourceMapSegmentV0],
) -> TransformSourceMapV3V0 {
    let source_path = source_path.into();
    let source_contents = source_content
        .map(|source_content| vec![(source_path.as_str(), source_content)])
        .unwrap_or_default();
    serialize_transform_source_map_v3_with_source_contents(
        file,
        generated_css,
        source_path.as_str(),
        source_contents.as_slice(),
        segments,
    )
}

pub fn serialize_transform_source_map_v3_with_source_contents(
    file: impl Into<String>,
    generated_css: &str,
    source_path: impl Into<String>,
    source_contents: &[(&str, &str)],
    segments: &[TransformSourceMapSegmentV0],
) -> TransformSourceMapV3V0 {
    let source_path = source_path.into();
    let sources = collect_source_map_sources(&source_path, segments);
    let mappings = encode_source_map_v3_mappings(generated_css, &sources, segments);
    let pass_ids = collect_source_map_pass_ids(segments);
    let sources_content = sources
        .iter()
        .map(|source| source_content_for_source_path(source, source_contents))
        .collect::<Vec<_>>();

    TransformSourceMapV3V0 {
        version: 3,
        file: file.into(),
        sources,
        sources_content,
        names: Vec::new(),
        mappings,
        x_omena_schema_version: "0",
        x_omena_product: "omena-transform-print.source-map-v3",
        x_omena_segment_count: segments.len(),
        x_omena_pass_ids: pass_ids,
    }
}

pub fn transform_source_map_point(source: &str, byte_offset: usize) -> TransformSourceMapPointV0 {
    source_map_point(source, byte_offset)
}

pub fn parse_transform_source_map_v3_json(
    source_map_json: &str,
) -> Result<ParsedTransformSourceMapV3V0, TransformSourceMapV3ParseErrorV0> {
    let raw = serde_json::from_str::<RawSourceMapV3V0>(source_map_json)
        .map_err(|error| TransformSourceMapV3ParseErrorV0::Json(error.to_string()))?;
    if raw.version != 3 {
        return Err(TransformSourceMapV3ParseErrorV0::UnsupportedVersion(
            raw.version,
        ));
    }
    let generated_line_count = raw.mappings.split(';').count().max(1);
    let decoded_segments =
        decode_source_map_v3_mappings(&raw.mappings, raw.sources.len(), raw.names.len())?;

    Ok(ParsedTransformSourceMapV3V0 {
        version: raw.version,
        file: raw.file,
        sources: raw.sources,
        sources_content: raw.sources_content,
        names: raw.names,
        mappings: raw.mappings,
        decoded_segments,
        generated_line_count,
    })
}

fn parse_transform_source_map_v3(
    source_map: &TransformSourceMapV3V0,
) -> Result<ParsedTransformSourceMapV3V0, TransformSourceMapV3ParseErrorV0> {
    let generated_line_count = source_map.mappings.split(';').count().max(1);
    let decoded_segments = decode_source_map_v3_mappings(
        &source_map.mappings,
        source_map.sources.len(),
        source_map.names.len(),
    )?;

    Ok(ParsedTransformSourceMapV3V0 {
        version: source_map.version,
        file: Some(source_map.file.clone()),
        sources: source_map.sources.clone(),
        sources_content: source_map
            .sources_content
            .iter()
            .map(|source_content| Some(source_content.clone()))
            .collect(),
        names: source_map.names.clone(),
        mappings: source_map.mappings.clone(),
        decoded_segments,
        generated_line_count,
    })
}

pub fn compose_transform_source_map_v3_with_upstream_map(
    downstream_map: &TransformSourceMapV3V0,
    upstream_source_path: &str,
    upstream_map_json: &str,
) -> TransformSourceMapCompositionV0 {
    let downstream_parsed = match parse_transform_source_map_v3(downstream_map) {
        Ok(parsed) => parsed,
        Err(error) => {
            return TransformSourceMapCompositionV0 {
                source_map: downstream_map.clone(),
                upstream_map_applied: false,
                fallback_reason: Some(format!("downstream source map parse failed: {error:?}")),
            };
        }
    };
    let upstream_parsed = match parse_transform_source_map_v3_json(upstream_map_json) {
        Ok(parsed) => parsed,
        Err(error) => {
            return TransformSourceMapCompositionV0 {
                source_map: downstream_map.clone(),
                upstream_map_applied: false,
                fallback_reason: Some(format!("upstream source map parse failed: {error:?}")),
            };
        }
    };
    let Some(upstream_source_index) = downstream_map
        .sources
        .iter()
        .position(|source| source == upstream_source_path)
    else {
        return TransformSourceMapCompositionV0 {
            source_map: downstream_map.clone(),
            upstream_map_applied: false,
            fallback_reason: Some(format!(
                "downstream source map does not include upstream source {upstream_source_path}"
            )),
        };
    };

    let mut composed_segments = Vec::<ComposedSourceMapSegmentV0>::new();
    let mut upstream_map_applied = false;
    for segment in downstream_parsed.decoded_segments {
        if segment.source_index == upstream_source_index
            && let Some(upstream_segment) = find_decoded_source_map_segment_for_generated_position(
                upstream_parsed.decoded_segments.as_slice(),
                segment.original_line,
                segment.original_utf16_column,
            )
        {
            let source_path = upstream_parsed
                .sources
                .get(upstream_segment.source_index)
                .cloned()
                .unwrap_or_else(|| upstream_source_path.to_string());
            let source_content = source_content_at_index(
                upstream_parsed.sources_content.as_slice(),
                upstream_segment.source_index,
            );
            composed_segments.push(ComposedSourceMapSegmentV0 {
                generated_line: segment.generated_line,
                generated_utf16_column: segment.generated_utf16_column,
                source_path,
                source_content,
                original_line: upstream_segment.original_line,
                original_utf16_column: upstream_segment.original_utf16_column,
                name_index: None,
            });
            upstream_map_applied = true;
            continue;
        }

        let source_path = downstream_map
            .sources
            .get(segment.source_index)
            .cloned()
            .unwrap_or_else(|| upstream_source_path.to_string());
        let source_content = source_content_at_index_as_string(
            downstream_map.sources_content.as_slice(),
            segment.source_index,
        );
        composed_segments.push(ComposedSourceMapSegmentV0 {
            generated_line: segment.generated_line,
            generated_utf16_column: segment.generated_utf16_column,
            source_path,
            source_content,
            original_line: segment.original_line,
            original_utf16_column: segment.original_utf16_column,
            name_index: segment.name_index,
        });
    }

    if !upstream_map_applied {
        return TransformSourceMapCompositionV0 {
            source_map: downstream_map.clone(),
            upstream_map_applied: false,
            fallback_reason: Some(format!(
                "upstream source map had no segment for {upstream_source_path}"
            )),
        };
    }

    let (sources, sources_content) = collect_composed_source_map_sources(&composed_segments);
    let mappings = encode_composed_source_map_v3_mappings(
        downstream_parsed.generated_line_count,
        sources.as_slice(),
        composed_segments.as_slice(),
    );
    TransformSourceMapCompositionV0 {
        source_map: TransformSourceMapV3V0 {
            version: downstream_map.version,
            file: downstream_map.file.clone(),
            sources,
            sources_content,
            names: downstream_map.names.clone(),
            mappings,
            x_omena_schema_version: downstream_map.x_omena_schema_version,
            x_omena_product: downstream_map.x_omena_product,
            x_omena_segment_count: composed_segments.len(),
            x_omena_pass_ids: downstream_map.x_omena_pass_ids.clone(),
        },
        upstream_map_applied: true,
        fallback_reason: None,
    }
}

fn source_content_for_source_path(source_path: &str, source_contents: &[(&str, &str)]) -> String {
    source_contents
        .iter()
        .find_map(|(candidate_path, source_content)| {
            (*candidate_path == source_path).then(|| (*source_content).to_string())
        })
        .unwrap_or_default()
}

fn collect_source_map_sources(
    fallback_source_path: &str,
    segments: &[TransformSourceMapSegmentV0],
) -> Vec<String> {
    let mut sources = segments
        .iter()
        .map(|segment| segment.source_path.clone())
        .collect::<Vec<_>>();
    if sources.is_empty() {
        sources.push(fallback_source_path.to_string());
    }
    sources.sort();
    sources.dedup();
    sources
}

fn collect_source_map_pass_ids(segments: &[TransformSourceMapSegmentV0]) -> Vec<&'static str> {
    let mut pass_ids = segments
        .iter()
        .map(|segment| segment.pass_id)
        .collect::<Vec<_>>();
    pass_ids.sort();
    pass_ids.dedup();
    pass_ids
}

fn encode_source_map_v3_mappings(
    generated_css: &str,
    sources: &[String],
    segments: &[TransformSourceMapSegmentV0],
) -> String {
    let mut sortable_segments = segments
        .iter()
        .filter(|segment| {
            segment.generated_start_point.byte_offset <= segment.generated_end_point.byte_offset
        })
        .collect::<Vec<_>>();
    sortable_segments.sort_by(|left, right| {
        (
            left.generated_start_point.line,
            left.generated_start_point.utf16_column,
            left.source_path.as_str(),
            left.original_start_point.line,
            left.original_start_point.utf16_column,
        )
            .cmp(&(
                right.generated_start_point.line,
                right.generated_start_point.utf16_column,
                right.source_path.as_str(),
                right.original_start_point.line,
                right.original_start_point.utf16_column,
            ))
    });

    let max_generated_line = generated_css
        .chars()
        .filter(|character| *character == '\n')
        .count()
        .max(
            sortable_segments
                .iter()
                .map(|segment| segment.generated_start_point.line)
                .max()
                .unwrap_or(0),
        );
    let mut lines = vec![Vec::<String>::new(); max_generated_line + 1];
    let mut previous_source_index = 0isize;
    let mut previous_original_line = 0isize;
    let mut previous_original_column = 0isize;

    let mut index = 0;
    while index < sortable_segments.len() {
        let line_index = sortable_segments[index].generated_start_point.line;
        let mut line_end = index + 1;
        while line_end < sortable_segments.len()
            && sortable_segments[line_end].generated_start_point.line == line_index
        {
            line_end += 1;
        }

        let mut previous_generated_column = 0isize;
        let mut seen = Vec::<(usize, usize, usize, usize)>::new();
        for segment in &sortable_segments[index..line_end] {
            let source_index = sources
                .iter()
                .position(|source| source == &segment.source_path)
                .unwrap_or(0);
            let generated_column = segment.generated_start_point.utf16_column;
            let original_line = segment.original_start_point.line;
            let original_column = segment.original_start_point.utf16_column;
            let key = (
                generated_column,
                source_index,
                original_line,
                original_column,
            );
            if seen.contains(&key) {
                continue;
            }
            seen.push(key);

            let encoded_segment = [
                generated_column as isize - previous_generated_column,
                source_index as isize - previous_source_index,
                original_line as isize - previous_original_line,
                original_column as isize - previous_original_column,
            ]
            .into_iter()
            .map(encode_vlq_value)
            .collect::<String>();
            lines[line_index].push(encoded_segment);

            previous_generated_column = generated_column as isize;
            previous_source_index = source_index as isize;
            previous_original_line = original_line as isize;
            previous_original_column = original_column as isize;
        }

        index = line_end;
    }

    lines
        .into_iter()
        .map(|line_segments| line_segments.join(","))
        .collect::<Vec<_>>()
        .join(";")
}

fn encode_vlq_value(value: isize) -> String {
    let mut value = if value < 0 {
        ((-value as usize) << 1) + 1
    } else {
        (value as usize) << 1
    };
    let mut output = String::new();
    loop {
        let mut digit = value & 0b11111;
        value >>= 5;
        if value > 0 {
            digit |= 0b100000;
        }
        output.push(BASE64_VLQ_DIGITS[digit] as char);
        if value == 0 {
            break;
        }
    }
    output
}

pub fn decode_vlq_values(
    encoded_segment: &str,
) -> Result<Vec<isize>, TransformSourceMapV3ParseErrorV0> {
    let mut values = Vec::new();
    let mut value = 0usize;
    let mut shift = 0usize;
    let mut has_digit = false;

    for character in encoded_segment.bytes() {
        let digit = decode_base64_vlq_digit(character).ok_or_else(|| {
            TransformSourceMapV3ParseErrorV0::InvalidMapping(format!(
                "invalid base64 VLQ digit '{}'",
                character as char
            ))
        })?;
        has_digit = true;
        value += (digit & 0b11111) << shift;
        if (digit & 0b100000) == 0 {
            values.push(decode_signed_vlq_value(value));
            value = 0;
            shift = 0;
            has_digit = false;
        } else {
            shift += 5;
            if shift > 60 {
                return Err(TransformSourceMapV3ParseErrorV0::InvalidMapping(
                    "base64 VLQ value exceeds supported width".to_string(),
                ));
            }
        }
    }

    if has_digit {
        return Err(TransformSourceMapV3ParseErrorV0::InvalidMapping(
            "unterminated base64 VLQ value".to_string(),
        ));
    }

    Ok(values)
}

fn decode_source_map_v3_mappings(
    mappings: &str,
    source_count: usize,
    name_count: usize,
) -> Result<Vec<DecodedSourceMapSegmentV0>, TransformSourceMapV3ParseErrorV0> {
    let mut decoded_segments = Vec::new();
    let mut previous_source_index = 0isize;
    let mut previous_original_line = 0isize;
    let mut previous_original_column = 0isize;
    let mut previous_name_index = 0isize;

    for (generated_line, line) in mappings.split(';').enumerate() {
        let mut previous_generated_column = 0isize;
        for encoded_segment in line.split(',').filter(|segment| !segment.is_empty()) {
            let values = decode_vlq_values(encoded_segment)?;
            if values.len() == 1 {
                previous_generated_column += values[0];
                continue;
            }
            if values.len() != 4 && values.len() != 5 {
                return Err(TransformSourceMapV3ParseErrorV0::InvalidMapping(
                    "source map segments must have 1, 4, or 5 fields".to_string(),
                ));
            }

            previous_generated_column += values[0];
            previous_source_index += values[1];
            previous_original_line += values[2];
            previous_original_column += values[3];
            if previous_generated_column < 0
                || previous_source_index < 0
                || previous_original_line < 0
                || previous_original_column < 0
            {
                return Err(TransformSourceMapV3ParseErrorV0::InvalidMapping(
                    "source map segment produced a negative position".to_string(),
                ));
            }
            let source_index = previous_source_index as usize;
            if source_index >= source_count {
                return Err(TransformSourceMapV3ParseErrorV0::SourceIndexOutOfRange {
                    index: source_index,
                    source_count,
                });
            }
            let name_index = if values.len() == 5 {
                previous_name_index += values[4];
                if previous_name_index < 0 {
                    return Err(TransformSourceMapV3ParseErrorV0::InvalidMapping(
                        "source map segment produced a negative name index".to_string(),
                    ));
                }
                let name_index = previous_name_index as usize;
                if name_index >= name_count {
                    return Err(TransformSourceMapV3ParseErrorV0::NameIndexOutOfRange {
                        index: name_index,
                        name_count,
                    });
                }
                Some(name_index)
            } else {
                None
            };

            decoded_segments.push(DecodedSourceMapSegmentV0 {
                generated_line,
                generated_utf16_column: previous_generated_column as usize,
                source_index,
                original_line: previous_original_line as usize,
                original_utf16_column: previous_original_column as usize,
                name_index,
            });
        }
    }

    Ok(decoded_segments)
}

fn decode_base64_vlq_digit(character: u8) -> Option<usize> {
    BASE64_VLQ_DIGITS
        .iter()
        .position(|candidate| *candidate == character)
}

fn decode_signed_vlq_value(value: usize) -> isize {
    let shifted = (value >> 1) as isize;
    if (value & 1) == 1 { -shifted } else { shifted }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ComposedSourceMapSegmentV0 {
    generated_line: usize,
    generated_utf16_column: usize,
    source_path: String,
    source_content: String,
    original_line: usize,
    original_utf16_column: usize,
    name_index: Option<usize>,
}

fn find_decoded_source_map_segment_for_generated_position(
    segments: &[DecodedSourceMapSegmentV0],
    generated_line: usize,
    generated_utf16_column: usize,
) -> Option<&DecodedSourceMapSegmentV0> {
    segments
        .iter()
        .filter(|segment| {
            segment.generated_line == generated_line
                && segment.generated_utf16_column <= generated_utf16_column
        })
        .max_by_key(|segment| segment.generated_utf16_column)
}

fn source_content_at_index(source_contents: &[Option<String>], source_index: usize) -> String {
    source_contents
        .get(source_index)
        .and_then(|source_content| source_content.clone())
        .unwrap_or_default()
}

fn source_content_at_index_as_string(source_contents: &[String], source_index: usize) -> String {
    source_contents
        .get(source_index)
        .cloned()
        .unwrap_or_default()
}

fn collect_composed_source_map_sources(
    segments: &[ComposedSourceMapSegmentV0],
) -> (Vec<String>, Vec<String>) {
    let mut sources = Vec::<String>::new();
    let mut sources_content = Vec::<String>::new();
    for segment in segments {
        if sources.contains(&segment.source_path) {
            continue;
        }
        sources.push(segment.source_path.clone());
        sources_content.push(segment.source_content.clone());
    }
    if sources.is_empty() {
        sources.push(String::new());
        sources_content.push(String::new());
    }
    (sources, sources_content)
}

fn encode_composed_source_map_v3_mappings(
    generated_line_count: usize,
    sources: &[String],
    segments: &[ComposedSourceMapSegmentV0],
) -> String {
    let mut sortable_segments = segments.iter().collect::<Vec<_>>();
    sortable_segments.sort_by(|left, right| {
        (
            left.generated_line,
            left.generated_utf16_column,
            left.source_path.as_str(),
            left.original_line,
            left.original_utf16_column,
        )
            .cmp(&(
                right.generated_line,
                right.generated_utf16_column,
                right.source_path.as_str(),
                right.original_line,
                right.original_utf16_column,
            ))
    });
    let max_generated_line = generated_line_count.saturating_sub(1).max(
        sortable_segments
            .iter()
            .map(|segment| segment.generated_line)
            .max()
            .unwrap_or(0),
    );
    let mut lines = vec![Vec::<String>::new(); max_generated_line + 1];
    let mut previous_source_index = 0isize;
    let mut previous_original_line = 0isize;
    let mut previous_original_column = 0isize;
    let mut previous_name_index = 0isize;

    let mut index = 0;
    while index < sortable_segments.len() {
        let line_index = sortable_segments[index].generated_line;
        let mut line_end = index + 1;
        while line_end < sortable_segments.len()
            && sortable_segments[line_end].generated_line == line_index
        {
            line_end += 1;
        }

        let mut previous_generated_column = 0isize;
        for segment in &sortable_segments[index..line_end] {
            let source_index = sources
                .iter()
                .position(|source| source == &segment.source_path)
                .unwrap_or(0);
            let mut values = vec![
                segment.generated_utf16_column as isize - previous_generated_column,
                source_index as isize - previous_source_index,
                segment.original_line as isize - previous_original_line,
                segment.original_utf16_column as isize - previous_original_column,
            ];
            if let Some(name_index) = segment.name_index {
                values.push(name_index as isize - previous_name_index);
                previous_name_index = name_index as isize;
            }
            let encoded_segment = values.into_iter().map(encode_vlq_value).collect::<String>();
            lines[line_index].push(encoded_segment);

            previous_generated_column = segment.generated_utf16_column as isize;
            previous_source_index = source_index as isize;
            previous_original_line = segment.original_line as isize;
            previous_original_column = segment.original_utf16_column as isize;
        }

        index = line_end;
    }

    lines
        .into_iter()
        .map(|line_segments| line_segments.join(","))
        .collect::<Vec<_>>()
        .join(";")
}

const BASE64_VLQ_DIGITS: &[u8; 64] =
    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

#[derive(Debug, Clone, Copy)]
struct SourceMapSources<'a> {
    original: &'a str,
    generated: &'a str,
}

#[derive(Debug, Clone, Copy)]
struct SourceMapSpanOffsets {
    original_start: usize,
    original_end: usize,
    generated_start: usize,
    generated_end: usize,
}

fn source_map_segment(
    source_path: &str,
    sources: SourceMapSources<'_>,
    offsets: SourceMapSpanOffsets,
    pass_id: &'static str,
) -> TransformSourceMapSegmentV0 {
    TransformSourceMapSegmentV0 {
        source_path: source_path.to_string(),
        original_start: offsets.original_start,
        original_end: offsets.original_end,
        generated_start: offsets.generated_start,
        generated_end: offsets.generated_end,
        original_start_point: source_map_point(sources.original, offsets.original_start),
        original_end_point: source_map_point(sources.original, offsets.original_end),
        generated_start_point: source_map_point(sources.generated, offsets.generated_start),
        generated_end_point: source_map_point(sources.generated, offsets.generated_end),
        pass_id,
    }
}

fn source_map_point(source: &str, byte_offset: usize) -> TransformSourceMapPointV0 {
    let byte_offset = byte_offset.min(source.len());
    let mut line = 0;
    let mut line_start_byte = 0;
    let mut utf16_column = 0;

    for (index, character) in source.char_indices() {
        if index >= byte_offset {
            break;
        }
        if character == '\n' {
            line += 1;
            line_start_byte = index + character.len_utf8();
            utf16_column = 0;
        } else {
            utf16_column += character.len_utf16();
        }
    }

    TransformSourceMapPointV0 {
        byte_offset,
        line,
        utf8_column: byte_offset.saturating_sub(line_start_byte),
        utf16_column,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RenderedPrintCss {
    css: String,
    generated_offset_lookup: Option<Vec<usize>>,
}

fn render_css_for_print_mode(
    source: &str,
    dialect: StyleDialect,
    mode: TransformPrintMode,
) -> RenderedPrintCss {
    match mode {
        TransformPrintMode::Minified => {
            let execution = execute_transform_passes_on_source_with_dialect(
                source,
                dialect,
                &[
                    TransformPassKind::CommentStrip,
                    TransformPassKind::WhitespaceStrip,
                    TransformPassKind::PrintCss,
                ],
            );
            let generated_offset_lookup =
                generated_offset_lookup_for_deleted_subsequence(source, &execution.output_css);
            RenderedPrintCss {
                css: execution.output_css,
                generated_offset_lookup: Some(generated_offset_lookup),
            }
        }
        TransformPrintMode::Identity | TransformPrintMode::Pretty => RenderedPrintCss {
            css: source.to_string(),
            generated_offset_lookup: None,
        },
    }
}

fn generated_offset_lookup_for_print_mode(source: &str, generated: &str) -> Option<Vec<usize>> {
    (source != generated)
        .then(|| generated_offset_lookup_for_deleted_subsequence(source, generated))
}

fn generated_offset_lookup_for_deleted_subsequence(source: &str, generated: &str) -> Vec<usize> {
    let source_bytes = source.as_bytes();
    let generated_bytes = generated.as_bytes();
    let mut lookup = vec![0; source_bytes.len() + 1];
    let mut generated_index = 0usize;

    for index in 0..source_bytes.len() {
        lookup[index] = generated_index;
        if generated_index < generated_bytes.len()
            && source_bytes[index] == generated_bytes[generated_index]
        {
            generated_index += 1;
        }
    }
    lookup[source_bytes.len()] = generated_bytes.len();
    lookup
}

fn project_generated_offset(
    source_offset: usize,
    generated_len: usize,
    generated_offset_lookup: Option<&[usize]>,
) -> usize {
    generated_offset_lookup
        .and_then(|lookup| lookup.get(source_offset).copied())
        .unwrap_or(source_offset)
        .min(generated_len)
}

#[cfg(test)]
mod tests {
    use super::{
        TransformPrintMode, TransformPrintOptionsV0, TransformSourceMapV3V0,
        compose_transform_source_map_v3_with_upstream_map, decode_vlq_values,
        default_print_options, encode_vlq_value, parse_transform_source_map_v3_json,
        print_transform_cst_source, print_transform_execution_artifact,
        print_transform_execution_artifact_with_source, serialize_transform_source_map_v3,
        source_map_point, summarize_omena_transform_print_boundary,
    };
    use omena_transform_cst::TransformPassKind;
    use omena_transform_passes::execute_transform_passes_on_source;

    #[test]
    fn exposes_print_boundary() {
        let boundary = summarize_omena_transform_print_boundary();

        assert_eq!(boundary.product, "omena-transform-print.boundary");
        assert_eq!(boundary.emission_pass_id, "print-css");
        assert_eq!(
            boundary.supported_modes,
            vec![TransformPrintMode::Identity, TransformPrintMode::Minified]
        );
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
        assert!(artifact.source_map_v3.as_ref().is_some_and(|source_map| {
            source_map.version == 3
                && source_map.file == "Button.module.css"
                && source_map.sources.len() == 1
                && source_map
                    .sources
                    .first()
                    .is_some_and(|source_path| source_path == "Button.module.css")
                && source_map.sources_content.len() == 1
                && source_map
                    .sources_content
                    .first()
                    .is_some_and(|source_content| source_content == source)
                && !source_map.mappings.is_empty()
                && source_map.x_omena_segment_count == artifact.source_map_segments.len()
        }));
    }

    #[test]
    fn prints_minified_css_with_projected_source_map_offsets() {
        let source = "/* remove */ .button { color: red; margin: 0px; }";
        let artifact = print_transform_cst_source(
            "Button.module.css",
            source,
            "semantic:button",
            &[TransformPassKind::PrintCss],
            TransformPrintOptionsV0 {
                mode: TransformPrintMode::Minified,
                include_source_map: true,
            },
        );

        assert_eq!(artifact.css, ".button{color:red;margin:0px}");
        assert!(artifact.provenance_preserved);
        assert!(!artifact.source_map_segments.is_empty());
        assert!(
            artifact
                .source_map_segments
                .iter()
                .all(|segment| segment.generated_end <= artifact.css.len())
        );
        assert!(
            artifact
                .source_map_segments
                .iter()
                .any(|segment| segment.generated_start < segment.original_start)
        );
        assert!(
            artifact
                .source_map_v3
                .as_ref()
                .is_some_and(|source_map| !source_map.mappings.is_empty())
        );
    }

    #[test]
    fn serializes_source_map_v3_mappings_from_existing_segments() {
        let source = ".a { color: red; }\n.b { color: blue; }";
        let artifact = print_transform_cst_source(
            "Mapped.module.css",
            source,
            "semantic:mapped",
            &[TransformPassKind::PrintCss],
            default_print_options(),
        );
        let source_map = serialize_transform_source_map_v3(
            "Mapped.module.css",
            &artifact.css,
            "Mapped.module.css",
            Some(source),
            artifact.source_map_segments.as_slice(),
        );

        assert_eq!(source_map.version, 3);
        assert_eq!(source_map.names, Vec::<String>::new());
        assert_eq!(source_map.sources, vec!["Mapped.module.css"]);
        assert_eq!(source_map.sources_content, vec![source]);
        assert_eq!(
            source_map.x_omena_pass_ids,
            vec![TransformPassKind::PrintCss.id()]
        );
        assert_eq!(
            source_map.x_omena_segment_count,
            artifact.source_map_segments.len()
        );
        assert!(
            source_map.mappings.contains(';'),
            "multi-line generated CSS should produce semicolon-separated mapping lines"
        );
    }

    #[test]
    fn vlq_encoder_matches_source_map_signed_base64_values() {
        assert_eq!(encode_vlq_value(0), "A");
        assert_eq!(encode_vlq_value(1), "C");
        assert_eq!(encode_vlq_value(-1), "D");
        assert_eq!(encode_vlq_value(16), "gB");
    }

    #[test]
    fn vlq_decoder_matches_source_map_signed_base64_values() {
        assert_eq!(decode_vlq_values("A"), Ok(vec![0]));
        assert_eq!(decode_vlq_values("C"), Ok(vec![1]));
        assert_eq!(decode_vlq_values("D"), Ok(vec![-1]));
        assert_eq!(decode_vlq_values("gB"), Ok(vec![16]));
        assert!(decode_vlq_values("!").is_err());
    }

    #[test]
    fn parses_source_map_v3_fixture_mappings() {
        let upstream_map = include_str!("../fixtures/source-map/button.css.map");
        let parsed = parse_transform_source_map_v3_json(upstream_map);

        assert!(
            parsed.is_ok(),
            "fixture source map should parse: {parsed:?}"
        );
        let Ok(parsed) = parsed else {
            return;
        };
        assert_eq!(parsed.version, 3);
        assert_eq!(parsed.sources, vec!["button.scss"]);
        let remapped_color = super::find_decoded_source_map_segment_for_generated_position(
            parsed.decoded_segments.as_slice(),
            1,
            2,
        );
        assert!(
            remapped_color.is_some(),
            "fixture should map compiled CSS color declaration"
        );
        if let Some(remapped_color) = remapped_color {
            assert_eq!(remapped_color.source_index, 0);
            assert_eq!(remapped_color.original_line, 3);
            assert_eq!(remapped_color.original_utf16_column, 9);
        }
    }

    #[test]
    fn composes_upstream_sass_source_map_with_omena_origin_chain() {
        let compiled_css = include_str!("../fixtures/source-map/button.css");
        let original_scss = include_str!("../fixtures/source-map/button.scss");
        let upstream_map = include_str!("../fixtures/source-map/button.css.map");
        let downstream_map = TransformSourceMapV3V0 {
            version: 3,
            file: "bundle.css".to_string(),
            sources: vec!["button.css".to_string()],
            sources_content: vec![compiled_css.to_string()],
            names: Vec::new(),
            mappings: "AAAA;EACE;AACF".to_string(),
            x_omena_schema_version: "0",
            x_omena_product: "omena-transform-print.source-map-v3",
            x_omena_segment_count: 3,
            x_omena_pass_ids: vec![TransformPassKind::PrintCss.id()],
        };

        let composition = compose_transform_source_map_v3_with_upstream_map(
            &downstream_map,
            "button.css",
            upstream_map,
        );

        assert!(
            composition.upstream_map_applied,
            "upstream map should compose: {:?}",
            composition.fallback_reason
        );
        assert_eq!(composition.fallback_reason, None);
        assert_eq!(composition.source_map.sources, vec!["button.scss"]);
        assert_eq!(composition.source_map.sources_content, vec![original_scss]);
        let composed_json = serde_json::to_string(&composition.source_map);
        assert!(
            composed_json.is_ok(),
            "composed source map should serialize: {composed_json:?}"
        );
        let Ok(composed_json) = composed_json else {
            return;
        };
        let parsed_composed = parse_transform_source_map_v3_json(&composed_json);
        assert!(
            parsed_composed.is_ok(),
            "composed source map should parse: {parsed_composed:?}"
        );
        let Ok(parsed_composed) = parsed_composed else {
            return;
        };
        let composed_color = super::find_decoded_source_map_segment_for_generated_position(
            parsed_composed.decoded_segments.as_slice(),
            1,
            2,
        );
        assert!(
            composed_color.is_some(),
            "composed map should retain the bundle output color position"
        );
        if let Some(composed_color) = composed_color {
            assert_eq!(
                parsed_composed.sources.get(composed_color.source_index),
                Some(&"button.scss".to_string())
            );
            assert_eq!(composed_color.original_line, 3);
            assert_eq!(composed_color.original_utf16_column, 9);
        }
    }

    #[test]
    fn upstream_source_map_composition_falls_back_on_malformed_maps() {
        let compiled_css = include_str!("../fixtures/source-map/button.css");
        let malformed_map = include_str!("../fixtures/source-map/malformed-vlq.css.map");
        let downstream_map = TransformSourceMapV3V0 {
            version: 3,
            file: "bundle.css".to_string(),
            sources: vec!["button.css".to_string()],
            sources_content: vec![compiled_css.to_string()],
            names: Vec::new(),
            mappings: "AAAA;EACE;AACF".to_string(),
            x_omena_schema_version: "0",
            x_omena_product: "omena-transform-print.source-map-v3",
            x_omena_segment_count: 3,
            x_omena_pass_ids: vec![TransformPassKind::PrintCss.id()],
        };

        let composition = compose_transform_source_map_v3_with_upstream_map(
            &downstream_map,
            "button.css",
            malformed_map,
        );

        assert!(!composition.upstream_map_applied);
        assert!(
            composition
                .fallback_reason
                .as_deref()
                .is_some_and(|reason| { reason.contains("upstream source map parse failed") })
        );
        assert_eq!(composition.source_map, downstream_map);
    }

    #[test]
    fn source_map_points_include_column_precision_for_unicode_and_newlines() {
        let source = ".초기 { color: red; }\n.button { color: blue; }";
        let artifact = print_transform_cst_source(
            "Button.module.css",
            source,
            "semantic:unicode:button",
            &[TransformPassKind::PrintCss],
            default_print_options(),
        );
        let unicode_start = source.find("초기").unwrap_or(source.len());
        assert!(unicode_start < source.len());
        let unicode_end = unicode_start + "초기".len();
        let unicode_start_point = source_map_point(source, unicode_start);
        let unicode_end_point = source_map_point(source, unicode_end);
        let button_segment = artifact
            .source_map_segments
            .iter()
            .find(|segment| &source[segment.original_start..segment.original_end] == "button");
        assert!(button_segment.is_some());

        assert_eq!(unicode_start_point.line, 0);
        assert_eq!(unicode_start_point.utf8_column, 1);
        assert_eq!(unicode_start_point.utf16_column, 1);
        assert_eq!(unicode_end_point.utf8_column, 7);
        assert_eq!(unicode_end_point.utf16_column, 3);
        if let Some(button_segment) = button_segment {
            assert_eq!(button_segment.original_start_point.line, 1);
            assert_eq!(button_segment.original_start_point.utf8_column, 1);
            assert_eq!(button_segment.original_start_point.utf16_column, 1);
            assert_eq!(
                button_segment.generated_start_point,
                button_segment.original_start_point
            );
        }
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
    fn minified_execution_artifact_projects_upstream_segments_to_final_css() {
        let source = ".button { color: red; }\n.card { color: blue; }";
        let execution = execute_transform_passes_on_source(source, &[TransformPassKind::PrintCss]);
        let artifact = print_transform_execution_artifact_with_source(
            "Button.module.css",
            source,
            "semantic:button-card",
            &[TransformPassKind::PrintCss],
            TransformPrintOptionsV0 {
                mode: TransformPrintMode::Minified,
                include_source_map: true,
            },
            &execution,
        );

        assert_eq!(artifact.css, ".button{color:red}.card{color:blue}");
        assert!(artifact.provenance_preserved);
        assert_eq!(artifact.source_map_segments[0].generated_start, 0);
        assert_eq!(
            artifact
                .source_map_segments
                .last()
                .map(|segment| segment.generated_end),
            Some(artifact.css.len())
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

        let artifact = print_transform_execution_artifact_with_source(
            "Multi.module.css",
            source,
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
        assert_eq!(comment_segments[1].original_start_point.line, 1);
    }
}
