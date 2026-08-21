use omena_benchmarks::{bundler_productization_corpus, style_corpus};
use omena_parser::{StyleDialect, collect_style_fact_collection};
use serde::Serialize;
use std::{hint::black_box, time::Instant};

const SAMPLE_BATCH_COUNT: usize = 7;
const EDIT_LOCAL_THRESHOLD_NS: u128 = 3_200_000;
const EDIT_BYTE_COUNTS: [usize; 3] = [1, 8, 64];

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ParserEditTraceSlopeProfileV0 {
    schema_version: &'static str,
    product: &'static str,
    measurement_pin: String,
    release_build: bool,
    sample_batch_count: usize,
    corpus_file_count: usize,
    session_style_document_count: usize,
    session_source_document_count: usize,
    workspace_stress_file_count: usize,
    p90_file_name: String,
    p90_file_bytes: usize,
    p90_full_parse_and_collection_nanoseconds: u128,
    workspace_stress_p90_file_name: String,
    workspace_stress_p90_file_bytes: usize,
    workspace_stress_p90_full_parse_and_collection_nanoseconds: u128,
    edit_local_threshold_nanoseconds: u128,
    threshold_consumption_milli: u128,
    changed_byte_slope_nanoseconds_per_byte: f64,
    source_size_slope_nanoseconds_per_byte: f64,
    disposition: &'static str,
    samples: Vec<ParserEditTraceSampleV0>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ParserEditTraceSampleV0 {
    name: String,
    corpus_kind: &'static str,
    dialect: &'static str,
    source_bytes: usize,
    changed_bytes: usize,
    iterations_per_batch: usize,
    full_parse_and_collection_nanoseconds_p50: u128,
    full_parse_and_collection_nanoseconds_p90: u128,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut session_corpus = session_style_inputs();
    session_corpus.sort_by_key(|sample| sample.source.len());
    let p90_index = percentile_index(session_corpus.len(), 90);
    let p90_file_name = session_corpus[p90_index].name.clone();
    let p90_file_bytes = session_corpus[p90_index].source.len();
    let session_style_document_count = session_corpus.len();
    let session_source_document_count = 10;
    let mut workspace_corpus = style_corpus()
        .into_iter()
        .chain(bundler_productization_corpus())
        .map(|sample| TraceInputV0 {
            name: sample.name.to_string(),
            corpus_kind: "workspace-stress",
            dialect: sample.dialect,
            source: sample.source,
        })
        .collect::<Vec<_>>();
    workspace_corpus.sort_by_key(|sample| sample.source.len());
    let workspace_stress_p90_index = percentile_index(workspace_corpus.len(), 90);
    let workspace_stress_p90_file_name = workspace_corpus[workspace_stress_p90_index].name.clone();
    let workspace_stress_p90_file_bytes = workspace_corpus[workspace_stress_p90_index].source.len();
    let workspace_stress_file_count = workspace_corpus.len();
    let mut corpus = session_corpus;
    corpus.append(&mut workspace_corpus);
    let mut samples = Vec::new();

    for sample in &corpus {
        for changed_bytes in EDIT_BYTE_COUNTS {
            let edited = apply_suffix_edit(sample.source.as_str(), changed_bytes);
            let iterations_per_batch = iterations_for_size(edited.len());
            for _ in 0..2 {
                black_box(collect_style_fact_collection(
                    black_box(edited.as_str()),
                    sample.dialect,
                ));
            }
            let mut timings = Vec::with_capacity(SAMPLE_BATCH_COUNT);
            for _ in 0..SAMPLE_BATCH_COUNT {
                let started = Instant::now();
                for _ in 0..iterations_per_batch {
                    black_box(collect_style_fact_collection(
                        black_box(edited.as_str()),
                        sample.dialect,
                    ));
                }
                timings.push(started.elapsed().as_nanos() / iterations_per_batch as u128);
            }
            timings.sort_unstable();
            samples.push(ParserEditTraceSampleV0 {
                name: sample.name.clone(),
                corpus_kind: sample.corpus_kind,
                dialect: dialect_label(sample.dialect),
                source_bytes: sample.source.len(),
                changed_bytes,
                iterations_per_batch,
                full_parse_and_collection_nanoseconds_p50: timings[timings.len() / 2],
                full_parse_and_collection_nanoseconds_p90: timings
                    [percentile_index(timings.len(), 90)],
            });
        }
    }

    let p90_file_samples = samples
        .iter()
        .filter(|sample| sample.name == p90_file_name)
        .collect::<Vec<_>>();
    let p90_full_parse_and_collection_nanoseconds = p90_file_samples
        .iter()
        .map(|sample| sample.full_parse_and_collection_nanoseconds_p90)
        .max()
        .ok_or("P90 corpus file must have edit samples")?;
    let workspace_stress_p90_full_parse_and_collection_nanoseconds = samples
        .iter()
        .filter(|sample| sample.name == workspace_stress_p90_file_name)
        .map(|sample| sample.full_parse_and_collection_nanoseconds_p90)
        .max()
        .ok_or("workspace stress P90 file must have edit samples")?;
    let changed_byte_slope_nanoseconds_per_byte = linear_slope(
        p90_file_samples
            .iter()
            .map(|sample| {
                (
                    sample.changed_bytes as f64,
                    sample.full_parse_and_collection_nanoseconds_p50 as f64,
                )
            })
            .collect::<Vec<_>>()
            .as_slice(),
    );
    let source_size_slope_nanoseconds_per_byte = linear_slope(
        samples
            .iter()
            .filter(|sample| sample.changed_bytes == 1)
            .map(|sample| {
                (
                    sample.source_bytes as f64,
                    sample.full_parse_and_collection_nanoseconds_p50 as f64,
                )
            })
            .collect::<Vec<_>>()
            .as_slice(),
    );
    let disposition = if p90_full_parse_and_collection_nanoseconds >= EDIT_LOCAL_THRESHOLD_NS {
        "draft-edit-local-parser-design"
    } else {
        "park-edit-local-parser"
    };
    let profile = ParserEditTraceSlopeProfileV0 {
        schema_version: "0",
        product: "omena-benchmarks.parser-edit-trace-slope-profile",
        measurement_pin: std::env::var("OMENA_MEASUREMENT_PIN")
            .unwrap_or_else(|_| "unrecorded".to_string()),
        release_build: !cfg!(debug_assertions),
        sample_batch_count: SAMPLE_BATCH_COUNT,
        corpus_file_count: corpus.len(),
        session_style_document_count,
        session_source_document_count,
        workspace_stress_file_count,
        p90_file_name,
        p90_file_bytes,
        p90_full_parse_and_collection_nanoseconds,
        workspace_stress_p90_file_name,
        workspace_stress_p90_file_bytes,
        workspace_stress_p90_full_parse_and_collection_nanoseconds,
        edit_local_threshold_nanoseconds: EDIT_LOCAL_THRESHOLD_NS,
        threshold_consumption_milli: p90_full_parse_and_collection_nanoseconds
            .saturating_mul(1_000)
            / EDIT_LOCAL_THRESHOLD_NS,
        changed_byte_slope_nanoseconds_per_byte,
        source_size_slope_nanoseconds_per_byte,
        disposition,
        samples,
    };
    println!("{}", serde_json::to_string_pretty(&profile)?);
    Ok(())
}

struct TraceInputV0 {
    name: String,
    corpus_kind: &'static str,
    dialect: StyleDialect,
    source: String,
}

fn session_style_inputs() -> Vec<TraceInputV0> {
    (0..140)
        .map(|ordinal| {
            let source = if ordinal == 0 {
                "$tone: rgb(1, 3, 7);\n.item000 { color: $tone; }\n".to_string()
            } else {
                format!(
                    "@use \"./Style000.module.scss\" as base;\n.item{ordinal:03} {{ color: base.$tone; }}\n"
                )
            };
            TraceInputV0 {
                name: format!("session-style-{ordinal:03}"),
                corpus_kind: "session-fixture",
                dialect: StyleDialect::Scss,
                source,
            }
        })
        .collect()
}

fn apply_suffix_edit(source: &str, changed_bytes: usize) -> String {
    let suffix = if changed_bytes == 1 {
        "\n".to_string()
    } else {
        format!("/*{}*/", "x".repeat(changed_bytes.saturating_sub(4)))
    };
    debug_assert_eq!(suffix.len(), changed_bytes);
    let mut edited = String::with_capacity(source.len() + suffix.len());
    edited.push_str(source);
    edited.push_str(suffix.as_str());
    edited
}

fn percentile_index(sample_count: usize, percentile: usize) -> usize {
    sample_count
        .saturating_mul(percentile)
        .div_ceil(100)
        .saturating_sub(1)
        .min(sample_count.saturating_sub(1))
}

fn linear_slope(points: &[(f64, f64)]) -> f64 {
    let count = points.len() as f64;
    let mean_x = points.iter().map(|(x, _)| *x).sum::<f64>() / count;
    let mean_y = points.iter().map(|(_, y)| *y).sum::<f64>() / count;
    let numerator = points
        .iter()
        .map(|(x, y)| (*x - mean_x) * (*y - mean_y))
        .sum::<f64>();
    let denominator = points
        .iter()
        .map(|(x, _)| (*x - mean_x).powi(2))
        .sum::<f64>();
    numerator / denominator.max(f64::EPSILON)
}

const fn iterations_for_size(byte_length: usize) -> usize {
    if byte_length <= 20 * 1024 {
        15
    } else if byte_length <= 80 * 1024 {
        5
    } else {
        2
    }
}

const fn dialect_label(dialect: StyleDialect) -> &'static str {
    match dialect {
        StyleDialect::Css => "css",
        StyleDialect::Scss => "scss",
        StyleDialect::Sass => "sass",
        StyleDialect::Less => "less",
    }
}
