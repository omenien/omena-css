pub const Z5_PERFORMANCE_BASELINE: &str = "z5-performance-baseline";

mod corpus;

use omena_parser::StyleDialect;
use omena_transform_cst::TransformPassKind;
use omena_transform_passes::{
    TransformExecutionContextV0, TransformExecutionSummaryV0,
    execute_transform_passes_on_source_with_dialect_and_context,
    execute_transform_passes_on_source_with_dialect_and_context_without_lex_cache_for_measurement,
    reset_transform_lex_cache_splice_telemetry, transform_lex_cache_splice_telemetry_snapshot,
};
use omena_transform_print::{
    TransformPrintMode, TransformPrintOptionsV0, default_print_options,
    parse_transform_source_map_v3_json, print_transform_cst_source_with_dialect,
};
use serde::Serialize;
use sha2::{Digest, Sha256};

pub use corpus::{
    StyleCorpusSampleSnapshotV0, StyleCorpusSnapshotV0, StyleSample, bundler_productization_corpus,
    style_corpus, summarize_style_corpus_snapshot,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParserProductBenchmarkBoundaryV0 {
    pub lane: &'static str,
    pub input_boundary: &'static str,
    pub measured_operation: &'static str,
    pub includes_parse: bool,
    pub parse_work_measured_inside_summary: bool,
    pub includes_product_summary: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OmenaParserProductMeasurementV0 {
    pub boundary: ParserProductBenchmarkBoundaryV0,
    pub summary: omena_parser::ParserIndexSummaryV0,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LegacyParserProductMeasurementV0 {
    pub boundary: ParserProductBenchmarkBoundaryV0,
    pub summary: engine_style_parser::ParserIndexSummaryV0,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ParserProductBenchmarkReadinessSummaryV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub status: &'static str,
    pub benchmark_family: &'static str,
    pub lane_count: usize,
    pub lanes: Vec<&'static str>,
    pub sample_count: usize,
    pub sample_names: Vec<&'static str>,
    pub input_boundary: &'static str,
    pub measured_operation: &'static str,
    pub includes_parse: bool,
    pub parse_work_measured_inside_summary: bool,
    pub includes_product_summary: bool,
    pub symmetric_measurement_boundary: bool,
    pub all_samples_parse_in_both_lanes: bool,
    pub comparison_policy: &'static str,
    pub criterion_surface_snapshot_available: bool,
    pub next_priorities: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CriterionBenchmarkGroupSnapshotV0 {
    pub group: &'static str,
    pub measured_operation: &'static str,
    pub workload_kind: &'static str,
    pub benchmark_count: usize,
    pub sample_names: Vec<&'static str>,
    pub uses_style_corpus: bool,
    pub lane: Option<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CriterionSurfaceSnapshotV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub benchmark_family: &'static str,
    pub snapshot_kind: &'static str,
    pub timing_policy: &'static str,
    pub command: &'static str,
    pub corpus_sample_count: usize,
    pub benchmark_group_count: usize,
    pub benchmark_function_count: usize,
    pub groups: Vec<CriterionBenchmarkGroupSnapshotV0>,
    pub includes_legacy_parser_oracle_lane: bool,
    pub includes_omena_parser_lane: bool,
    pub includes_parser_product_lanes: bool,
    pub includes_semantic_lane: bool,
    pub includes_abstract_value_lane: bool,
    pub m4_corpus_expansion_reflected: bool,
    pub symmetric_parser_product_boundary: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BundlerProductizationBenchmarkSampleV0 {
    pub name: &'static str,
    pub path: &'static str,
    pub dialect: &'static str,
    pub byte_length: usize,
    pub line_count: usize,
    pub source: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BundlerProductizationBenchmarkLaneV0 {
    pub lane: &'static str,
    pub runtime_boundary: &'static str,
    pub runner: &'static str,
    pub supports_scss: bool,
    pub measures_process_startup: bool,
    pub comparator: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BundlerProductizationBenchmarkSurfaceSnapshotV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub benchmark_family: &'static str,
    pub status: &'static str,
    pub corpus_sample_count: usize,
    pub samples: Vec<BundlerProductizationBenchmarkSampleV0>,
    pub lane_count: usize,
    pub lanes: Vec<BundlerProductizationBenchmarkLaneV0>,
    pub measured_operations: Vec<&'static str>,
    pub includes_napi_in_process_lane: bool,
    pub includes_cli_spawn_lane: bool,
    pub includes_lightningcss_comparator_lane: bool,
    pub includes_memory_rss_metric: bool,
    pub includes_provenance_mode_split: bool,
    pub speed_claim_ready: bool,
    pub timing_policy: &'static str,
    pub comparison_policy: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EmittedCssGoldenSampleSnapshotV0 {
    pub name: &'static str,
    pub path: &'static str,
    pub dialect: &'static str,
    pub source_byte_length: usize,
    pub output_byte_length: usize,
    pub output_line_count: usize,
    pub source_sha256: String,
    pub output_sha256: String,
    pub deterministic_output: bool,
    pub source_map_v3_present: bool,
    pub css_modules_moat: bool,
    pub dart_sass_advisory_eligible: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EmittedCssGoldenGateSummaryV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub benchmark_family: &'static str,
    pub emitter: &'static str,
    pub check_id: &'static str,
    pub update_check_id: &'static str,
    pub fixture_count: usize,
    pub corpus_sha256: String,
    pub emitted_css_sha256: String,
    pub all_outputs_byte_stable: bool,
    pub includes_css_modules_moat_fixture: bool,
    pub dart_sass_advisory_policy: &'static str,
    pub dart_sass_advisory_sample_count: usize,
    pub speed_claim_ready: bool,
    pub samples: Vec<EmittedCssGoldenSampleSnapshotV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HeadlineAxisFidelitySampleV0 {
    pub name: &'static str,
    pub path: &'static str,
    pub dialect: &'static str,
    pub source_byte_length: usize,
    pub minified_css_byte_length: usize,
    pub source_map_byte_length: usize,
    pub decoded_source_map_segment_count: usize,
    pub source_map_vlq_valid: bool,
    pub all_decoded_segments_map_to_valid_positions: bool,
    pub css_modules_moat_preserved_through_minify: bool,
    pub provenance_overhead_basis_points: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HeadlineAxisFidelitySummaryV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub benchmark_family: &'static str,
    pub measured_axis: &'static str,
    pub sample_count: usize,
    pub source_map_vlq_valid: bool,
    pub source_map_positions_valid: bool,
    pub css_modules_moat_preserved_through_minify: bool,
    pub max_provenance_overhead_basis_points: u64,
    pub runtime_loop_headline_ready: bool,
    pub runtime_loop_verdict: &'static str,
    pub speed_claim_ready: bool,
    pub publication_policy: &'static str,
    pub samples: Vec<HeadlineAxisFidelitySampleV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformRelexBaselineSampleSnapshotV0 {
    pub name: String,
    pub base_sample_name: &'static str,
    pub path: &'static str,
    pub dialect: &'static str,
    pub scale: usize,
    pub source_byte_length: usize,
    pub output_byte_length: usize,
    pub source_sha256: String,
    pub output_sha256: String,
    pub lex_invocation_count: u64,
    pub lex_token_count: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformRelexBaselineScaleSnapshotV0 {
    pub scale: usize,
    pub source_byte_length: usize,
    pub lex_invocation_count: u64,
    pub lex_token_count: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformExecutorSpineSampleSnapshotV0 {
    pub name: String,
    pub lane: &'static str,
    pub dialect: &'static str,
    pub scale: usize,
    pub source_byte_length: usize,
    pub output_byte_length: usize,
    pub edited_byte_count: usize,
    pub mutation_count: usize,
    pub source_sha256: String,
    pub output_sha256: String,
    pub lex_invocation_count: u64,
    pub lex_token_count: u64,
    pub lex_splice_hit_count: u64,
    pub lex_splice_full_relex_fallback_count: u64,
    pub lex_splice_window_derivation_fallback_count: u64,
    pub lex_splice_full_output_window_fallback_count: u64,
    pub lex_splice_token_offset_fallback_count: u64,
    pub lex_tokens_per_edited_byte_milli: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformExecutorSpineLaneSnapshotV0 {
    pub lane: &'static str,
    pub lex_policy: &'static str,
    pub total_source_byte_length: usize,
    pub total_edited_byte_count: usize,
    pub total_lex_invocation_count: u64,
    pub total_lex_token_count: u64,
    pub total_lex_splice_hit_count: u64,
    pub total_lex_splice_full_relex_fallback_count: u64,
    pub lex_invocation_growth_exponent_milli: i64,
    pub lex_token_growth_exponent_milli: i64,
    pub max_lex_tokens_per_edited_byte_milli: u64,
    pub samples: Vec<TransformExecutorSpineSampleSnapshotV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformExecutorSpineAssertionSnapshotV0 {
    pub asserted: bool,
    pub measured_operation: &'static str,
    pub corpus_kind: &'static str,
    pub check_id: &'static str,
    pub current_lane: TransformExecutorSpineLaneSnapshotV0,
    pub full_relex_witness_lane: TransformExecutorSpineLaneSnapshotV0,
    pub lex_invocation_growth_exponent_bound_milli: i64,
    pub lex_token_growth_exponent_bound_milli: i64,
    pub lex_tokens_per_edited_byte_bound_milli: u64,
    pub lex_splice_full_relex_fallback_bound: u64,
    pub red_on_full_relex_witness: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformRelexBaselineSummaryV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub benchmark_family: &'static str,
    pub measured_operation: &'static str,
    pub timing_policy: &'static str,
    pub check_id: &'static str,
    pub update_check_id: &'static str,
    pub speed_claim_ready: bool,
    pub corpus_kind: &'static str,
    pub base_sample_count: usize,
    pub scale_count: usize,
    pub sample_count: usize,
    pub total_source_byte_length: usize,
    pub total_lex_invocation_count: u64,
    pub total_lex_token_count: u64,
    pub lex_invocation_growth_exponent_milli: i64,
    pub lex_token_growth_exponent_milli: i64,
    pub executor_spine: TransformExecutorSpineAssertionSnapshotV0,
    pub scales: Vec<TransformRelexBaselineScaleSnapshotV0>,
    pub samples: Vec<TransformRelexBaselineSampleSnapshotV0>,
}

pub fn parser_product_benchmark_boundaries() -> [ParserProductBenchmarkBoundaryV0; 2] {
    [
        ParserProductBenchmarkBoundaryV0 {
            lane: "legacy",
            input_boundary: "raw-style-source",
            measured_operation: "source-to-product-summary",
            includes_parse: true,
            parse_work_measured_inside_summary: true,
            includes_product_summary: true,
        },
        ParserProductBenchmarkBoundaryV0 {
            lane: "omena",
            input_boundary: "raw-style-source",
            measured_operation: "source-to-product-summary",
            includes_parse: true,
            parse_work_measured_inside_summary: true,
            includes_product_summary: true,
        },
    ]
}

pub fn legacy_parser_product_benchmark_boundary() -> ParserProductBenchmarkBoundaryV0 {
    parser_product_benchmark_boundaries()[0]
}

pub fn omena_parser_product_benchmark_boundary() -> ParserProductBenchmarkBoundaryV0 {
    parser_product_benchmark_boundaries()[1]
}

pub fn validate_parser_product_benchmark_boundary_symmetry() -> Result<(), String> {
    let [legacy, omena] = parser_product_benchmark_boundaries();
    if legacy.input_boundary != omena.input_boundary {
        return Err(format!(
            "parser product benchmark input boundary mismatch: legacy={} omena={}",
            legacy.input_boundary, omena.input_boundary,
        ));
    }
    if legacy.measured_operation != omena.measured_operation {
        return Err(format!(
            "parser product benchmark operation mismatch: legacy={} omena={}",
            legacy.measured_operation, omena.measured_operation,
        ));
    }
    if !(legacy.includes_parse
        && omena.includes_parse
        && legacy.parse_work_measured_inside_summary
        && omena.parse_work_measured_inside_summary
        && legacy.includes_product_summary
        && omena.includes_product_summary)
    {
        return Err(
            "parser product benchmark must measure parse work inside product summary for both lanes"
                .to_string(),
        );
    }
    Ok(())
}

pub fn summarize_parser_product_benchmark_readiness() -> ParserProductBenchmarkReadinessSummaryV0 {
    let boundaries = parser_product_benchmark_boundaries();
    let [legacy, omena] = boundaries;
    let samples = style_corpus();
    let all_samples_parse_in_both_lanes = samples.iter().all(|sample| {
        validate_legacy_style_sample(sample.path, sample.source.as_str()).is_ok()
            && validate_omena_style_sample(sample.source.as_str(), sample.dialect).is_ok()
    });

    ParserProductBenchmarkReadinessSummaryV0 {
        schema_version: "0",
        product: "omena-benchmarks.parser-product-readiness",
        status: "parserProductBoundaryReady",
        benchmark_family: Z5_PERFORMANCE_BASELINE,
        lane_count: boundaries.len(),
        lanes: vec![legacy.lane, omena.lane],
        sample_count: samples.len(),
        sample_names: samples.iter().map(|sample| sample.name).collect(),
        input_boundary: legacy.input_boundary,
        measured_operation: legacy.measured_operation,
        includes_parse: legacy.includes_parse && omena.includes_parse,
        parse_work_measured_inside_summary: legacy.parse_work_measured_inside_summary
            && omena.parse_work_measured_inside_summary,
        includes_product_summary: legacy.includes_product_summary && omena.includes_product_summary,
        symmetric_measurement_boundary: validate_parser_product_benchmark_boundary_symmetry()
            .is_ok(),
        all_samples_parse_in_both_lanes,
        comparison_policy: "raw-style-source-to-product-summary-for-each-lane",
        criterion_surface_snapshot_available: true,
        next_priorities: vec!["runFullCriterionTimingSnapshotBeforeExternalSpeedClaim"],
    }
}

pub fn summarize_criterion_surface_snapshot() -> CriterionSurfaceSnapshotV0 {
    let sample_names = style_corpus()
        .iter()
        .map(|sample| sample.name)
        .collect::<Vec<_>>();
    let groups = vec![
        CriterionBenchmarkGroupSnapshotV0 {
            group: "z5/parser",
            measured_operation: "legacy-style-parser-parse-style-module",
            workload_kind: "style-corpus",
            benchmark_count: sample_names.len(),
            sample_names: sample_names.clone(),
            uses_style_corpus: true,
            lane: Some("legacy-parser-oracle"),
        },
        CriterionBenchmarkGroupSnapshotV0 {
            group: "z5/omena-parser",
            measured_operation: "omena-parser-parse",
            workload_kind: "style-corpus",
            benchmark_count: sample_names.len(),
            sample_names: sample_names.clone(),
            uses_style_corpus: true,
            lane: Some("omena-parser"),
        },
        CriterionBenchmarkGroupSnapshotV0 {
            group: "z5/parser-product-legacy",
            measured_operation: "source-to-product-summary",
            workload_kind: "style-corpus",
            benchmark_count: sample_names.len(),
            sample_names: sample_names.clone(),
            uses_style_corpus: true,
            lane: Some("legacy"),
        },
        CriterionBenchmarkGroupSnapshotV0 {
            group: "z5/parser-product-omena",
            measured_operation: "source-to-product-summary",
            workload_kind: "style-corpus",
            benchmark_count: sample_names.len(),
            sample_names: sample_names.clone(),
            uses_style_corpus: true,
            lane: Some("omena"),
        },
        CriterionBenchmarkGroupSnapshotV0 {
            group: "z5/semantic",
            measured_operation: "source-to-semantic-boundary-summary",
            workload_kind: "style-corpus",
            benchmark_count: sample_names.len(),
            sample_names: sample_names.clone(),
            uses_style_corpus: true,
            lane: Some("omena-semantic"),
        },
        CriterionBenchmarkGroupSnapshotV0 {
            group: "z5/abstract-value",
            measured_operation: "abstract-value-flow-analysis",
            workload_kind: "synthetic-flow-graph",
            benchmark_count: 3,
            sample_names: vec![
                "flow-1cfa-256-nodes",
                "one-cfa-40-call-sites",
                "reduced-product-intersection",
            ],
            uses_style_corpus: false,
            lane: Some("omena-abstract-value"),
        },
    ];
    let benchmark_function_count = groups.iter().map(|group| group.benchmark_count).sum();
    let includes_group = |name: &str| groups.iter().any(|group| group.group == name);
    let includes_legacy_parser_oracle_lane = includes_group("z5/parser");
    let includes_omena_parser_lane = includes_group("z5/omena-parser");
    let includes_parser_product_lanes =
        includes_group("z5/parser-product-legacy") && includes_group("z5/parser-product-omena");
    let includes_semantic_lane = includes_group("z5/semantic");
    let includes_abstract_value_lane = includes_group("z5/abstract-value");

    CriterionSurfaceSnapshotV0 {
        schema_version: "0",
        product: "omena-benchmarks.criterion-surface-snapshot",
        benchmark_family: Z5_PERFORMANCE_BASELINE,
        snapshot_kind: "structural-criterion-surface-after-m4-corpus-expansion",
        timing_policy: "no-local-timing-claim-without-full-criterion-run",
        command: "pnpm omena-check run rust/z5-criterion-surface-snapshot",
        corpus_sample_count: sample_names.len(),
        benchmark_group_count: groups.len(),
        benchmark_function_count,
        groups,
        includes_legacy_parser_oracle_lane,
        includes_omena_parser_lane,
        includes_parser_product_lanes,
        includes_semantic_lane,
        includes_abstract_value_lane,
        m4_corpus_expansion_reflected: sample_names.len() >= 3,
        symmetric_parser_product_boundary: validate_parser_product_benchmark_boundary_symmetry()
            .is_ok(),
    }
}

pub fn summarize_bundler_productization_benchmark_surface()
-> BundlerProductizationBenchmarkSurfaceSnapshotV0 {
    let samples = bundler_productization_corpus()
        .into_iter()
        .map(|sample| BundlerProductizationBenchmarkSampleV0 {
            name: sample.name,
            path: sample.path,
            dialect: benchmark_style_dialect_label(sample.dialect),
            byte_length: sample.source.len(),
            line_count: sample.source.lines().count(),
            source: sample.source,
        })
        .collect::<Vec<_>>();

    let lanes = vec![
        BundlerProductizationBenchmarkLaneV0 {
            lane: "omena-napi-in-process",
            runtime_boundary: "node-process-native-binding",
            runner: "scripts/benchmark-omena-vite-productization.mjs",
            supports_scss: true,
            measures_process_startup: false,
            comparator: false,
        },
        BundlerProductizationBenchmarkLaneV0 {
            lane: "omena-cli-spawn",
            runtime_boundary: "node-child-process-cli",
            runner: "scripts/benchmark-omena-vite-productization.mjs",
            supports_scss: true,
            measures_process_startup: true,
            comparator: false,
        },
        BundlerProductizationBenchmarkLaneV0 {
            lane: "lightningcss-node",
            runtime_boundary: "node-library-comparator",
            runner: "scripts/benchmark-omena-vite-productization.mjs",
            supports_scss: false,
            measures_process_startup: false,
            comparator: true,
        },
    ];

    let includes_napi_in_process_lane = lanes
        .iter()
        .any(|lane| lane.lane == "omena-napi-in-process");
    let includes_cli_spawn_lane = lanes.iter().any(|lane| lane.lane == "omena-cli-spawn");
    let includes_lightningcss_comparator_lane =
        lanes.iter().any(|lane| lane.lane == "lightningcss-node");

    BundlerProductizationBenchmarkSurfaceSnapshotV0 {
        schema_version: "0",
        product: "omena-benchmarks.bundler-productization-surface",
        benchmark_family: "bundler-productization",
        status: "measurementSurfaceReadyNoSpeedClaim",
        corpus_sample_count: samples.len(),
        samples,
        lane_count: lanes.len(),
        lanes,
        measured_operations: vec![
            "per-file-parse-transform-minify",
            "multi-file-wall-clock",
            "napi-vs-cli-spawn-process-model-delta",
            "memory-rss",
            "provenance-on-off",
        ],
        includes_napi_in_process_lane,
        includes_cli_spawn_lane,
        includes_lightningcss_comparator_lane,
        includes_memory_rss_metric: true,
        includes_provenance_mode_split: true,
        speed_claim_ready: false,
        timing_policy: "no-speed-claim-without-recorded-full-run-artifact",
        comparison_policy: "compare-bundler-product-paths-by-boundary-and-publish-raw-measurements-only",
    }
}

pub fn summarize_emitted_css_golden_gate() -> Result<EmittedCssGoldenGateSummaryV0, String> {
    let samples = emitted_css_golden_corpus();
    let mut sample_snapshots = Vec::with_capacity(samples.len());
    let mut corpus_hasher = Sha256::new();
    let mut emitted_hasher = Sha256::new();

    for sample in &samples {
        let first = print_transform_cst_source_with_dialect(
            sample.path,
            sample.source.as_str(),
            sample.dialect,
            "omena-benchmarks.emitted-css-golden-gate",
            &[],
            default_print_options(),
        );
        let second = print_transform_cst_source_with_dialect(
            sample.path,
            sample.source.as_str(),
            sample.dialect,
            "omena-benchmarks.emitted-css-golden-gate",
            &[],
            default_print_options(),
        );
        let deterministic_output = first.css == second.css;
        if !deterministic_output {
            return Err(format!(
                "emitted CSS output is not byte-stable across two runs: {}",
                sample.name,
            ));
        }

        corpus_hasher.update(sample.name.as_bytes());
        corpus_hasher.update([0]);
        corpus_hasher.update(sample.path.as_bytes());
        corpus_hasher.update([0]);
        corpus_hasher.update(benchmark_style_dialect_label(sample.dialect).as_bytes());
        corpus_hasher.update([0]);
        corpus_hasher.update(sample.source.as_bytes());
        corpus_hasher.update([0]);

        emitted_hasher.update(sample.name.as_bytes());
        emitted_hasher.update([0]);
        emitted_hasher.update(sample.path.as_bytes());
        emitted_hasher.update([0]);
        emitted_hasher.update(first.css.as_bytes());
        emitted_hasher.update([0]);

        let css_modules_moat = is_css_modules_moat_sample(sample);
        sample_snapshots.push(EmittedCssGoldenSampleSnapshotV0 {
            name: sample.name,
            path: sample.path,
            dialect: benchmark_style_dialect_label(sample.dialect),
            source_byte_length: sample.source.len(),
            output_byte_length: first.css.len(),
            output_line_count: first.css.lines().count(),
            source_sha256: sha256_hex(sample.source.as_bytes()),
            output_sha256: sha256_hex(first.css.as_bytes()),
            deterministic_output,
            source_map_v3_present: first.source_map_v3.is_some(),
            css_modules_moat,
            dart_sass_advisory_eligible: is_dart_sass_advisory_eligible(sample, css_modules_moat),
        });
    }

    let includes_css_modules_moat_fixture = sample_snapshots
        .iter()
        .any(|sample| sample.css_modules_moat);
    let dart_sass_advisory_sample_count = sample_snapshots
        .iter()
        .filter(|sample| sample.dart_sass_advisory_eligible)
        .count();
    let all_outputs_byte_stable = sample_snapshots
        .iter()
        .all(|sample| sample.deterministic_output);

    Ok(EmittedCssGoldenGateSummaryV0 {
        schema_version: "0",
        product: "omena-benchmarks.emitted-css-golden-gate",
        benchmark_family: Z5_PERFORMANCE_BASELINE,
        emitter: "omena-transform-print.print_transform_cst_source_with_dialect",
        check_id: "rust/benchmark/emitted-css-golden-gate",
        update_check_id: "rust/benchmark/emitted-css-golden-gate:update",
        fixture_count: sample_snapshots.len(),
        corpus_sha256: hex_digest(corpus_hasher.finalize().as_slice()),
        emitted_css_sha256: hex_digest(emitted_hasher.finalize().as_slice()),
        all_outputs_byte_stable,
        includes_css_modules_moat_fixture,
        dart_sass_advisory_policy: "advisory-only-plain-sass-subset; css-modules-moat-never-external-gated",
        dart_sass_advisory_sample_count,
        speed_claim_ready: false,
        samples: sample_snapshots,
    })
}

pub fn render_emitted_css_golden_gate_snapshot_json() -> Result<String, String> {
    let summary = summarize_emitted_css_golden_gate()?;
    serde_json::to_string_pretty(&summary)
        .map(|json| format!("{json}\n"))
        .map_err(|error| error.to_string())
}

pub fn validate_emitted_css_golden_gate_snapshot(expected_json: &str) -> Result<(), String> {
    let current_json = render_emitted_css_golden_gate_snapshot_json()?;
    let current: serde_json::Value =
        serde_json::from_str(&current_json).map_err(|error| error.to_string())?;
    let expected: serde_json::Value =
        serde_json::from_str(expected_json).map_err(|error| error.to_string())?;

    if current == expected {
        Ok(())
    } else {
        Err(
            "emitted CSS golden snapshot drifted; run `pnpm omena-check run rust/benchmark/emitted-css-golden-gate:update` and review the diff"
                .to_string(),
        )
    }
}

pub fn summarize_headline_axis_fidelity() -> Result<HeadlineAxisFidelitySummaryV0, String> {
    let samples = bundler_productization_corpus()
        .into_iter()
        .filter(|sample| sample.name == "css-modules-product-grid")
        .collect::<Vec<_>>();
    if samples.is_empty() {
        return Err("headline fidelity suite requires css-modules-product-grid".to_string());
    }

    let mut sample_summaries = Vec::with_capacity(samples.len());
    for sample in &samples {
        let minified_with_map = print_transform_cst_source_with_dialect(
            sample.path,
            sample.source.as_str(),
            sample.dialect,
            "omena-benchmarks.headline-axis-fidelity",
            &[],
            TransformPrintOptionsV0 {
                mode: TransformPrintMode::Minified,
                include_source_map: true,
            },
        );
        let minified_without_map = print_transform_cst_source_with_dialect(
            sample.path,
            sample.source.as_str(),
            sample.dialect,
            "omena-benchmarks.headline-axis-fidelity",
            &[],
            TransformPrintOptionsV0 {
                mode: TransformPrintMode::Minified,
                include_source_map: false,
            },
        );
        let source_map = minified_with_map.source_map_v3.as_ref().ok_or_else(|| {
            format!(
                "headline fidelity sample missing Source Map V3 output: {}",
                sample.name,
            )
        })?;
        let source_map_json =
            serde_json::to_string(source_map).map_err(|error| error.to_string())?;
        let parsed_source_map =
            parse_transform_source_map_v3_json(&source_map_json).map_err(|error| {
                format!(
                    "headline fidelity source map should parse for {}: {error:?}",
                    sample.name,
                )
            })?;
        let source_map_vlq_valid = !parsed_source_map.decoded_segments.is_empty();
        let all_decoded_segments_map_to_valid_positions =
            parsed_source_map.decoded_segments.iter().all(|segment| {
                valid_utf16_position(
                    sample.source.as_str(),
                    segment.original_line,
                    segment.original_utf16_column,
                ) && valid_utf16_position(
                    minified_with_map.css.as_str(),
                    segment.generated_line,
                    segment.generated_utf16_column,
                )
            });
        let css_modules_moat_preserved_through_minify =
            minified_with_map.css.contains("composes:filterButton")
                && minified_with_map.css.contains(":global(.is-keyboard-user)");
        let provenance_overhead_basis_points = if minified_without_map.css.is_empty() {
            0
        } else {
            ((source_map_json.len() as u128 * 10_000) / minified_without_map.css.len() as u128)
                as u64
        };

        sample_summaries.push(HeadlineAxisFidelitySampleV0 {
            name: sample.name,
            path: sample.path,
            dialect: benchmark_style_dialect_label(sample.dialect),
            source_byte_length: sample.source.len(),
            minified_css_byte_length: minified_with_map.css.len(),
            source_map_byte_length: source_map_json.len(),
            decoded_source_map_segment_count: parsed_source_map.decoded_segments.len(),
            source_map_vlq_valid,
            all_decoded_segments_map_to_valid_positions,
            css_modules_moat_preserved_through_minify,
            provenance_overhead_basis_points,
        });
    }

    let source_map_vlq_valid = sample_summaries
        .iter()
        .all(|sample| sample.source_map_vlq_valid);
    let source_map_positions_valid = sample_summaries
        .iter()
        .all(|sample| sample.all_decoded_segments_map_to_valid_positions);
    let css_modules_moat_preserved_through_minify = sample_summaries
        .iter()
        .all(|sample| sample.css_modules_moat_preserved_through_minify);
    let max_provenance_overhead_basis_points = sample_summaries
        .iter()
        .map(|sample| sample.provenance_overhead_basis_points)
        .max()
        .unwrap_or(0);

    Ok(HeadlineAxisFidelitySummaryV0 {
        schema_version: "0",
        product: "omena-benchmarks.headline-axis-fidelity",
        benchmark_family: Z5_PERFORMANCE_BASELINE,
        measured_axis: "fidelity-provenance-and-runtime-loop-readiness",
        sample_count: sample_summaries.len(),
        source_map_vlq_valid,
        source_map_positions_valid,
        css_modules_moat_preserved_through_minify,
        max_provenance_overhead_basis_points,
        runtime_loop_headline_ready: false,
        runtime_loop_verdict: "not-ready-for-public-headline-until-schema-versioned-runtime-loop-artifact-exists",
        speed_claim_ready: false,
        publication_policy: "measurement-only-no-competitive-claim",
        samples: sample_summaries,
    })
}

pub fn fit_log_log_growth_exponent(samples: &[(usize, usize)]) -> Option<f64> {
    if samples.len() < 3
        || samples
            .iter()
            .any(|(input_size, output_size)| *input_size == 0 || *output_size == 0)
    {
        return None;
    }

    let count = samples.len() as f64;
    let (sum_x, sum_y, sum_xy, sum_x2) = samples.iter().fold(
        (0.0_f64, 0.0_f64, 0.0_f64, 0.0_f64),
        |(sum_x, sum_y, sum_xy, sum_x2), (input_size, output_size)| {
            let x = (*input_size as f64).ln();
            let y = (*output_size as f64).ln();
            (sum_x + x, sum_y + y, sum_xy + x * y, sum_x2 + x * x)
        },
    );
    let denominator = count * sum_x2 - sum_x * sum_x;
    if denominator.abs() <= f64::EPSILON {
        return None;
    }

    let exponent = (count * sum_xy - sum_x * sum_y) / denominator;
    exponent.is_finite().then_some(exponent)
}

const TRANSFORM_EXECUTOR_SPINE_LEX_INVOCATION_GROWTH_BOUND_MILLI: i64 = 1100;
const TRANSFORM_EXECUTOR_SPINE_LEX_TOKEN_GROWTH_BOUND_MILLI: i64 = 1100;
const TRANSFORM_EXECUTOR_SPINE_TOKENS_PER_EDITED_BYTE_BOUND_MILLI: u64 = 600;
const TRANSFORM_EXECUTOR_SPINE_LEX_SPLICE_FULL_RELEX_FALLBACK_BOUND: u64 = 3;

pub fn summarize_transform_executor_spine_assertion() -> TransformExecutorSpineAssertionSnapshotV0 {
    let current_lane = summarize_transform_executor_spine_lane(
        "cached-spine",
        "per-execution-cache-with-incremental-splice",
        execute_transform_passes_on_source_with_dialect_and_context,
    );
    let full_relex_witness_lane = summarize_transform_executor_spine_lane(
        "full-relex-witness",
        "cache-disabled-full-relex",
        execute_transform_passes_on_source_with_dialect_and_context_without_lex_cache_for_measurement,
    );
    let current_under_invocation_growth_bound = current_lane.lex_invocation_growth_exponent_milli
        <= TRANSFORM_EXECUTOR_SPINE_LEX_INVOCATION_GROWTH_BOUND_MILLI;
    let current_under_token_growth_bound = current_lane.lex_token_growth_exponent_milli
        <= TRANSFORM_EXECUTOR_SPINE_LEX_TOKEN_GROWTH_BOUND_MILLI;
    let current_under_edited_byte_bound = current_lane.max_lex_tokens_per_edited_byte_milli
        <= TRANSFORM_EXECUTOR_SPINE_TOKENS_PER_EDITED_BYTE_BOUND_MILLI;
    let current_under_splice_fallback_bound = current_lane
        .total_lex_splice_full_relex_fallback_count
        <= TRANSFORM_EXECUTOR_SPINE_LEX_SPLICE_FULL_RELEX_FALLBACK_BOUND;
    let witness_over_edited_byte_bound = full_relex_witness_lane
        .max_lex_tokens_per_edited_byte_milli
        > TRANSFORM_EXECUTOR_SPINE_TOKENS_PER_EDITED_BYTE_BOUND_MILLI;

    TransformExecutorSpineAssertionSnapshotV0 {
        asserted: current_under_invocation_growth_bound
            && current_under_token_growth_bound
            && current_under_edited_byte_bound
            && current_under_splice_fallback_bound
            && witness_over_edited_byte_bound,
        measured_operation: "transform-executor-lex-materialization-op-count",
        corpus_kind: "synthetic-mutating-transform-executor-size-sweep",
        check_id: "rust/benchmark/transform-relex-baseline",
        current_lane,
        full_relex_witness_lane,
        lex_invocation_growth_exponent_bound_milli:
            TRANSFORM_EXECUTOR_SPINE_LEX_INVOCATION_GROWTH_BOUND_MILLI,
        lex_token_growth_exponent_bound_milli:
            TRANSFORM_EXECUTOR_SPINE_LEX_TOKEN_GROWTH_BOUND_MILLI,
        lex_tokens_per_edited_byte_bound_milli:
            TRANSFORM_EXECUTOR_SPINE_TOKENS_PER_EDITED_BYTE_BOUND_MILLI,
        lex_splice_full_relex_fallback_bound:
            TRANSFORM_EXECUTOR_SPINE_LEX_SPLICE_FULL_RELEX_FALLBACK_BOUND,
        red_on_full_relex_witness: witness_over_edited_byte_bound,
    }
}

fn summarize_transform_executor_spine_lane(
    lane: &'static str,
    lex_policy: &'static str,
    execute: fn(
        &str,
        StyleDialect,
        &[TransformPassKind],
        &TransformExecutionContextV0,
    ) -> TransformExecutionSummaryV0,
) -> TransformExecutorSpineLaneSnapshotV0 {
    let scales = [1_usize, 2, 4];
    let mut samples = Vec::with_capacity(scales.len());
    let mut invocation_growth_samples = Vec::with_capacity(scales.len());
    let mut token_growth_samples = Vec::with_capacity(scales.len());
    let mut total_source_byte_length = 0usize;
    let mut total_edited_byte_count = 0usize;
    let mut total_lex_invocation_count = 0u64;
    let mut total_lex_token_count = 0u64;
    let mut total_lex_splice_hit_count = 0u64;
    let mut total_lex_splice_full_relex_fallback_count = 0u64;
    let mut max_lex_tokens_per_edited_byte_milli = 0u64;
    let requested = transform_executor_spine_passes();
    let context = TransformExecutionContextV0::default();

    for scale in scales {
        let source = transform_executor_spine_source(scale);
        reset_transform_lex_cache_splice_telemetry();
        let (execution, instrumentation) =
            omena_parser::with_omena_parser_lex_instrumentation(|| {
                execute(
                    source.as_str(),
                    StyleDialect::Css,
                    requested.as_slice(),
                    &context,
                )
            });
        let splice_telemetry = transform_lex_cache_splice_telemetry_snapshot();
        let edited_byte_count = transform_execution_edited_byte_count(&execution).max(1);
        let lex_tokens_per_edited_byte_milli =
            instrumentation.lex_token_count.saturating_mul(1000) / edited_byte_count as u64;
        max_lex_tokens_per_edited_byte_milli =
            max_lex_tokens_per_edited_byte_milli.max(lex_tokens_per_edited_byte_milli);
        total_source_byte_length += source.len();
        total_edited_byte_count += edited_byte_count;
        total_lex_invocation_count += instrumentation.lex_invocation_count;
        total_lex_token_count += instrumentation.lex_token_count;
        total_lex_splice_hit_count += splice_telemetry.splice_hit_count;
        total_lex_splice_full_relex_fallback_count += splice_telemetry.full_relex_fallback_count;
        invocation_growth_samples.push((
            source.len(),
            usize::try_from(instrumentation.lex_invocation_count).unwrap_or(usize::MAX),
        ));
        token_growth_samples.push((
            source.len(),
            usize::try_from(instrumentation.lex_token_count).unwrap_or(usize::MAX),
        ));
        samples.push(TransformExecutorSpineSampleSnapshotV0 {
            name: format!("executor-spine-mutating-corpus-x{scale}"),
            lane,
            dialect: "css",
            scale,
            source_byte_length: source.len(),
            output_byte_length: execution.output_css.len(),
            edited_byte_count,
            mutation_count: execution.mutation_count,
            source_sha256: sha256_hex(source.as_bytes()),
            output_sha256: sha256_hex(execution.output_css.as_bytes()),
            lex_invocation_count: instrumentation.lex_invocation_count,
            lex_token_count: instrumentation.lex_token_count,
            lex_splice_hit_count: splice_telemetry.splice_hit_count,
            lex_splice_full_relex_fallback_count: splice_telemetry.full_relex_fallback_count,
            lex_splice_window_derivation_fallback_count: splice_telemetry
                .window_derivation_fallback_count,
            lex_splice_full_output_window_fallback_count: splice_telemetry
                .full_output_window_fallback_count,
            lex_splice_token_offset_fallback_count: splice_telemetry.token_offset_fallback_count,
            lex_tokens_per_edited_byte_milli,
        });
    }

    TransformExecutorSpineLaneSnapshotV0 {
        lane,
        lex_policy,
        total_source_byte_length,
        total_edited_byte_count,
        total_lex_invocation_count,
        total_lex_token_count,
        total_lex_splice_hit_count,
        total_lex_splice_full_relex_fallback_count,
        lex_invocation_growth_exponent_milli: exponent_milli(invocation_growth_samples.as_slice()),
        lex_token_growth_exponent_milli: exponent_milli(token_growth_samples.as_slice()),
        max_lex_tokens_per_edited_byte_milli,
        samples,
    }
}

fn transform_executor_spine_passes() -> Vec<TransformPassKind> {
    vec![
        TransformPassKind::CommentStrip,
        TransformPassKind::NumberCompression,
        TransformPassKind::UnitNormalization,
        TransformPassKind::ColorCompression,
        TransformPassKind::CalcReduction,
        TransformPassKind::EmptyRuleRemoval,
        TransformPassKind::WhitespaceStrip,
        TransformPassKind::PrintCss,
    ]
}

fn transform_executor_spine_source(scale: usize) -> String {
    let mut source = String::new();
    for index in 0..(scale * 96) {
        source.push_str(&format!(
            "/* executor spine {index} */ .card-{index} {{ color: #ffffff; margin: 0.0px; padding: calc(1px + 1px); width: 10.0px; }} .empty-{index} {{ }}\n"
        ));
    }
    source
}

fn transform_execution_edited_byte_count(execution: &TransformExecutionSummaryV0) -> usize {
    execution
        .provenance_derivation_forest
        .nodes
        .iter()
        .flat_map(|node| node.mutation_spans.iter())
        .map(|span| {
            let source_len = span.source_span_end.saturating_sub(span.source_span_start);
            let generated_len = span
                .generated_span_end
                .saturating_sub(span.generated_span_start);
            source_len.max(generated_len)
        })
        .sum()
}

pub fn summarize_transform_relex_baseline() -> TransformRelexBaselineSummaryV0 {
    let base_samples = bundler_productization_corpus();
    let scales = [1_usize, 2, 4];
    let mut samples = Vec::with_capacity(base_samples.len() * scales.len());
    let mut by_scale = std::collections::BTreeMap::<usize, (usize, u64, u64)>::new();

    for base_sample in &base_samples {
        for scale in scales {
            let source = repeated_style_source(base_sample.source.as_str(), scale);
            let (artifact, instrumentation) =
                omena_parser::with_omena_parser_lex_instrumentation(|| {
                    print_transform_cst_source_with_dialect(
                        base_sample.path,
                        source.as_str(),
                        base_sample.dialect,
                        "omena-benchmarks.transform-relex-baseline",
                        &[],
                        TransformPrintOptionsV0 {
                            mode: TransformPrintMode::Minified,
                            include_source_map: true,
                        },
                    )
                });
            let source_byte_length = source.len();
            let scale_entry = by_scale.entry(scale).or_insert((0, 0, 0));
            scale_entry.0 += source_byte_length;
            scale_entry.1 += instrumentation.lex_invocation_count;
            scale_entry.2 += instrumentation.lex_token_count;

            samples.push(TransformRelexBaselineSampleSnapshotV0 {
                name: format!("{}-x{scale}", base_sample.name),
                base_sample_name: base_sample.name,
                path: base_sample.path,
                dialect: benchmark_style_dialect_label(base_sample.dialect),
                scale,
                source_byte_length,
                output_byte_length: artifact.css.len(),
                source_sha256: sha256_hex(source.as_bytes()),
                output_sha256: sha256_hex(artifact.css.as_bytes()),
                lex_invocation_count: instrumentation.lex_invocation_count,
                lex_token_count: instrumentation.lex_token_count,
            });
        }
    }

    let scales = by_scale
        .iter()
        .map(
            |(scale, (source_byte_length, lex_invocation_count, lex_token_count))| {
                TransformRelexBaselineScaleSnapshotV0 {
                    scale: *scale,
                    source_byte_length: *source_byte_length,
                    lex_invocation_count: *lex_invocation_count,
                    lex_token_count: *lex_token_count,
                }
            },
        )
        .collect::<Vec<_>>();
    let total_source_byte_length = samples
        .iter()
        .map(|sample| sample.source_byte_length)
        .sum::<usize>();
    let total_lex_invocation_count = samples
        .iter()
        .map(|sample| sample.lex_invocation_count)
        .sum::<u64>();
    let total_lex_token_count = samples
        .iter()
        .map(|sample| sample.lex_token_count)
        .sum::<u64>();
    let invocation_growth_samples = scales
        .iter()
        .map(|scale| {
            (
                scale.source_byte_length,
                usize::try_from(scale.lex_invocation_count).unwrap_or(usize::MAX),
            )
        })
        .collect::<Vec<_>>();
    let token_growth_samples = scales
        .iter()
        .map(|scale| {
            (
                scale.source_byte_length,
                usize::try_from(scale.lex_token_count).unwrap_or(usize::MAX),
            )
        })
        .collect::<Vec<_>>();

    TransformRelexBaselineSummaryV0 {
        schema_version: "0",
        product: "omena-benchmarks.transform-relex-baseline",
        benchmark_family: Z5_PERFORMANCE_BASELINE,
        measured_operation: "minified-print-path-lex-materialization-op-count",
        timing_policy: "deterministic-operation-count-no-wall-clock",
        check_id: "rust/benchmark/transform-relex-baseline",
        update_check_id: "rust/benchmark/transform-relex-baseline:update",
        speed_claim_ready: false,
        corpus_kind: "file-backed-bundler-productization-corpus-size-sweep",
        base_sample_count: base_samples.len(),
        scale_count: scales.len(),
        sample_count: samples.len(),
        total_source_byte_length,
        total_lex_invocation_count,
        total_lex_token_count,
        lex_invocation_growth_exponent_milli: exponent_milli(invocation_growth_samples.as_slice()),
        lex_token_growth_exponent_milli: exponent_milli(token_growth_samples.as_slice()),
        executor_spine: summarize_transform_executor_spine_assertion(),
        scales,
        samples,
    }
}

pub fn render_transform_relex_baseline_snapshot_json() -> Result<String, String> {
    let summary = summarize_transform_relex_baseline();
    serde_json::to_string_pretty(&summary)
        .map(|json| format!("{json}\n"))
        .map_err(|error| error.to_string())
}

pub fn validate_transform_relex_baseline_snapshot(expected_json: &str) -> Result<(), String> {
    let summary = summarize_transform_relex_baseline();

    // Intrinsic gate: refuse a degraded re-baseline REGARDLESS of the committed fixture.
    // Snapshot byte-equality alone would let a coordinated `--regen` re-pin an
    // `asserted:false` / non-RED-witness snapshot and still pass green. These two checks
    // recompute the assertion live, so the wired gate fails on a real regression even if
    // the fixture were re-pinned to match it.
    if !summary.executor_spine.asserted {
        return Err(
            "transform re-lex spine regressed: executor_spine.asserted=false — the cached-spine lane no longer meets the lex-invocation growth, lex-token growth, tokens-per-edited-byte, or splice-fallback bounds"
                .to_string(),
        );
    }
    if !summary.executor_spine.red_on_full_relex_witness {
        return Err(
            "transform re-lex gate became tautological: executor_spine.red_on_full_relex_witness=false — the cache-disabled full-relex witness lane no longer exceeds the tokens-per-edited-byte bound"
                .to_string(),
        );
    }

    let current_json = serde_json::to_string_pretty(&summary)
        .map(|json| format!("{json}\n"))
        .map_err(|error| error.to_string())?;
    let current: serde_json::Value =
        serde_json::from_str(&current_json).map_err(|error| error.to_string())?;
    let expected: serde_json::Value =
        serde_json::from_str(expected_json).map_err(|error| error.to_string())?;

    if current == expected {
        Ok(())
    } else {
        Err(
            "transform re-lex baseline snapshot drifted; run `pnpm omena-check run rust/benchmark/transform-relex-baseline:update` and review the diff"
                .to_string(),
        )
    }
}

fn emitted_css_golden_corpus() -> Vec<StyleSample> {
    let mut seen = std::collections::BTreeSet::new();
    style_corpus()
        .into_iter()
        .chain(bundler_productization_corpus())
        .filter(|sample| seen.insert((sample.name, sample.path)))
        .collect()
}

fn repeated_style_source(source: &str, scale: usize) -> String {
    let mut repeated = String::new();
    for index in 0..scale {
        repeated.push_str("/* omena benchmark scale ");
        repeated.push_str(index.to_string().as_str());
        repeated.push_str(" */\n");
        repeated.push_str(source);
        repeated.push('\n');
    }
    repeated
}

fn valid_utf16_position(source: &str, line: usize, utf16_column: usize) -> bool {
    source
        .split('\n')
        .nth(line)
        .is_some_and(|text| utf16_column <= text.encode_utf16().count())
}

fn is_css_modules_moat_sample(sample: &StyleSample) -> bool {
    sample.path.contains(".module.")
        || sample.source.contains("composes:")
        || sample.source.contains(":global(")
        || sample.source.contains("@value")
}

fn is_dart_sass_advisory_eligible(sample: &StyleSample, css_modules_moat: bool) -> bool {
    matches!(sample.dialect, StyleDialect::Scss | StyleDialect::Sass) && !css_modules_moat
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex_digest(hasher.finalize().as_slice())
}

fn hex_digest(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>()
}

fn benchmark_style_dialect_label(dialect: StyleDialect) -> &'static str {
    match dialect {
        StyleDialect::Css => "css",
        StyleDialect::Scss => "scss",
        StyleDialect::Sass => "sass",
        StyleDialect::Less => "less",
    }
}

fn exponent_milli(samples: &[(usize, usize)]) -> i64 {
    fit_log_log_growth_exponent(samples)
        .map(|exponent| (exponent * 1000.0).round() as i64)
        .unwrap_or(0)
}

pub fn parse_legacy_style_sample(
    path: &str,
    source: &str,
) -> Option<engine_style_parser::Stylesheet> {
    engine_style_parser::parse_style_module(path, source)
}

pub fn summarize_legacy_style_sample(
    path: &str,
    source: &str,
) -> Option<engine_style_parser::ParserIndexSummaryV0> {
    summarize_legacy_parser_product_sample(path, source)
}

pub fn summarize_legacy_parser_product_sample(
    path: &str,
    source: &str,
) -> Option<engine_style_parser::ParserIndexSummaryV0> {
    let sheet = parse_legacy_style_sample(path, source)?;
    Some(engine_style_parser::summarize_css_modules_intermediate(
        &sheet,
    ))
}

pub fn measure_legacy_parser_product_sample(
    path: &str,
    source: &str,
) -> Option<LegacyParserProductMeasurementV0> {
    let summary = summarize_legacy_parser_product_sample(path, source)?;
    Some(LegacyParserProductMeasurementV0 {
        boundary: legacy_parser_product_benchmark_boundary(),
        summary,
    })
}

pub fn summarize_omena_style_sample(
    source: &str,
    dialect: StyleDialect,
) -> omena_parser::ParserIndexSummaryV0 {
    summarize_omena_parser_product_sample(source, dialect)
}

pub fn summarize_omena_parser_product_sample(
    source: &str,
    dialect: StyleDialect,
) -> omena_parser::ParserIndexSummaryV0 {
    omena_parser::summarize_css_modules_intermediate(source, dialect)
}

pub fn measure_omena_parser_product_sample(
    source: &str,
    dialect: StyleDialect,
) -> OmenaParserProductMeasurementV0 {
    let summary = summarize_omena_parser_product_sample(source, dialect);
    OmenaParserProductMeasurementV0 {
        boundary: omena_parser_product_benchmark_boundary(),
        summary,
    }
}

pub fn validate_omena_style_sample(source: &str, dialect: StyleDialect) -> Result<(), String> {
    let parsed = omena_parser::parse(source, dialect);
    if parsed.token_count() > 0 {
        Ok(())
    } else {
        Err(format!(
            "benchmark style sample should produce omena parser tokens: {dialect:?}",
        ))
    }
}

pub fn validate_legacy_style_sample(path: &str, source: &str) -> Result<(), String> {
    if parse_legacy_style_sample(path, source).is_some() {
        Ok(())
    } else {
        Err(format!(
            "benchmark style sample should be accepted by legacy parser: {path}",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::{
        bundler_productization_corpus, measure_legacy_parser_product_sample,
        measure_omena_parser_product_sample, parser_product_benchmark_boundaries,
        render_emitted_css_golden_gate_snapshot_json, style_corpus,
        summarize_bundler_productization_benchmark_surface, summarize_criterion_surface_snapshot,
        summarize_emitted_css_golden_gate, summarize_headline_axis_fidelity,
        summarize_parser_product_benchmark_readiness, summarize_style_corpus_snapshot,
        summarize_transform_relex_baseline, validate_emitted_css_golden_gate_snapshot,
        validate_legacy_style_sample, validate_omena_style_sample,
        validate_parser_product_benchmark_boundary_symmetry,
        validate_transform_relex_baseline_snapshot,
    };

    const EMITTED_CSS_GOLDEN: &str = include_str!("../fixtures/emitted-css-golden-v0.json");
    const TRANSFORM_RELEX_BASELINE: &str =
        include_str!("../fixtures/transform-relex-baseline-v0.json");

    #[test]
    fn parser_product_benchmarks_declare_symmetric_measurement_boundary() -> Result<(), String> {
        validate_parser_product_benchmark_boundary_symmetry()?;
        let boundaries = parser_product_benchmark_boundaries();

        assert_eq!(boundaries.len(), 2);
        assert!(boundaries.iter().all(|boundary| boundary.includes_parse));
        assert!(
            boundaries
                .iter()
                .all(|boundary| boundary.parse_work_measured_inside_summary)
        );
        assert!(
            boundaries
                .iter()
                .all(|boundary| boundary.includes_product_summary)
        );
        assert!(
            boundaries
                .iter()
                .all(|boundary| boundary.input_boundary == "raw-style-source")
        );
        Ok(())
    }

    #[test]
    fn parser_product_samples_use_symmetric_source_to_summary_boundaries() -> Result<(), String> {
        for sample in style_corpus() {
            validate_legacy_style_sample(sample.path, sample.source.as_str())?;
            validate_omena_style_sample(sample.source.as_str(), sample.dialect)?;

            let legacy = measure_legacy_parser_product_sample(sample.path, sample.source.as_str())
                .ok_or_else(|| format!("legacy parser product failed for {}", sample.name))?;
            let omena = measure_omena_parser_product_sample(sample.source.as_str(), sample.dialect);

            assert_eq!(
                legacy.boundary.input_boundary,
                omena.boundary.input_boundary
            );
            assert_eq!(
                legacy.boundary.measured_operation,
                omena.boundary.measured_operation
            );
            assert_eq!(
                legacy.boundary.includes_parse,
                omena.boundary.includes_parse
            );
            assert_eq!(
                legacy.boundary.parse_work_measured_inside_summary,
                omena.boundary.parse_work_measured_inside_summary
            );
            assert_eq!(
                legacy.boundary.includes_product_summary,
                omena.boundary.includes_product_summary
            );

            let legacy = serde_json::to_value(legacy.summary).map_err(|error| error.to_string())?;
            let omena = serde_json::to_value(omena.summary).map_err(|error| error.to_string())?;

            assert_eq!(legacy["language"], omena["language"]);
            assert!(legacy["selectors"]["names"].as_array().is_some());
            assert!(omena["selectors"]["names"].as_array().is_some());
            assert!(legacy["wrappers"].as_object().is_some());
            assert!(omena["wrappers"].as_object().is_some());
        }
        Ok(())
    }

    #[test]
    fn parser_product_readiness_summary_is_serializable_and_honest() -> Result<(), String> {
        let summary = summarize_parser_product_benchmark_readiness();

        assert_eq!(summary.schema_version, "0");
        assert_eq!(summary.product, "omena-benchmarks.parser-product-readiness");
        assert_eq!(summary.status, "parserProductBoundaryReady");
        assert_eq!(summary.benchmark_family, super::Z5_PERFORMANCE_BASELINE);
        assert_eq!(summary.lanes, vec!["legacy", "omena"]);
        assert_eq!(summary.sample_count, style_corpus().len());
        assert_eq!(summary.input_boundary, "raw-style-source");
        assert_eq!(summary.measured_operation, "source-to-product-summary");
        assert!(summary.includes_parse);
        assert!(summary.parse_work_measured_inside_summary);
        assert!(summary.includes_product_summary);
        assert!(summary.symmetric_measurement_boundary);
        assert!(summary.all_samples_parse_in_both_lanes);
        assert_eq!(
            summary.comparison_policy,
            "raw-style-source-to-product-summary-for-each-lane"
        );
        assert!(summary.criterion_surface_snapshot_available);
        assert!(
            summary
                .next_priorities
                .contains(&"runFullCriterionTimingSnapshotBeforeExternalSpeedClaim")
        );

        let serialized = serde_json::to_value(&summary).map_err(|error| error.to_string())?;
        assert_eq!(
            serialized
                .pointer("/symmetricMeasurementBoundary")
                .and_then(|value| value.as_bool()),
            Some(true)
        );
        assert_eq!(
            serialized
                .pointer("/allSamplesParseInBothLanes")
                .and_then(|value| value.as_bool()),
            Some(true)
        );
        Ok(())
    }

    #[test]
    fn style_corpus_snapshot_exposes_the_benchmark_sources() -> Result<(), String> {
        let snapshot = summarize_style_corpus_snapshot();

        assert_eq!(snapshot.schema_version, "0");
        assert_eq!(snapshot.product, "omena-benchmarks.style-corpus-snapshot");
        assert_eq!(snapshot.benchmark_family, super::Z5_PERFORMANCE_BASELINE);
        assert_eq!(snapshot.corpus_sample_count, style_corpus().len());
        assert_eq!(snapshot.corpus_sample_count, 10);
        assert_eq!(snapshot.samples.len(), snapshot.corpus_sample_count);
        assert!(snapshot.samples.iter().all(|sample| sample.byte_length > 0));
        assert!(
            snapshot
                .samples
                .iter()
                .any(|sample| sample.dialect == "css")
        );
        assert!(
            snapshot
                .samples
                .iter()
                .any(|sample| sample.dialect == "scss")
        );
        assert!(
            snapshot
                .samples
                .iter()
                .any(|sample| sample.name == "css-sizing-width-corpus")
        );
        assert!(
            snapshot
                .samples
                .iter()
                .any(|sample| sample.name == "css-backgrounds-longhand-corpus")
        );
        assert!(
            snapshot
                .samples
                .iter()
                .any(|sample| sample.name == "css-display-layout-corpus")
        );
        assert!(
            snapshot
                .samples
                .iter()
                .any(|sample| sample.name == "css-position-layout-corpus")
        );
        assert!(
            snapshot
                .samples
                .iter()
                .any(|sample| sample.name == "css-ui-box-model-corpus")
        );
        assert!(
            snapshot
                .samples
                .iter()
                .any(|sample| sample.name == "css-transforms-motion-corpus")
        );
        assert!(
            snapshot
                .samples
                .iter()
                .any(|sample| sample.name == "css-fonts-text-corpus")
        );

        let serialized = serde_json::to_value(&snapshot).map_err(|error| error.to_string())?;
        assert_eq!(
            serialized
                .pointer("/product")
                .and_then(|value| value.as_str()),
            Some("omena-benchmarks.style-corpus-snapshot")
        );
        assert!(
            serialized
                .pointer("/samples/0/source")
                .and_then(|value| value.as_str())
                .is_some_and(|source| !source.is_empty())
        );
        Ok(())
    }

    #[test]
    fn criterion_surface_snapshot_covers_current_m4_benchmark_family() -> Result<(), String> {
        let snapshot = summarize_criterion_surface_snapshot();

        assert_eq!(
            snapshot.product,
            "omena-benchmarks.criterion-surface-snapshot"
        );
        assert_eq!(snapshot.benchmark_family, super::Z5_PERFORMANCE_BASELINE);
        assert_eq!(
            snapshot.snapshot_kind,
            "structural-criterion-surface-after-m4-corpus-expansion"
        );
        assert_eq!(snapshot.corpus_sample_count, style_corpus().len());
        assert_eq!(snapshot.benchmark_group_count, 6);
        assert_eq!(snapshot.benchmark_function_count, 53);
        assert!(snapshot.includes_legacy_parser_oracle_lane);
        assert!(snapshot.includes_omena_parser_lane);
        assert!(snapshot.includes_parser_product_lanes);
        assert!(snapshot.includes_semantic_lane);
        assert!(snapshot.includes_abstract_value_lane);
        assert!(snapshot.m4_corpus_expansion_reflected);
        assert!(snapshot.symmetric_parser_product_boundary);
        assert!(
            snapshot
                .groups
                .iter()
                .filter(|group| group.uses_style_corpus)
                .all(|group| group.sample_names == snapshot.groups[0].sample_names)
        );

        let serialized = serde_json::to_value(&snapshot).map_err(|error| error.to_string())?;
        assert_eq!(
            serialized.pointer("/benchmarkFunctionCount"),
            Some(&serde_json::json!(53))
        );
        assert_eq!(
            serialized
                .pointer("/timingPolicy")
                .and_then(|value| value.as_str()),
            Some("no-local-timing-claim-without-full-criterion-run")
        );
        Ok(())
    }

    #[test]
    fn emitted_css_golden_gate_records_stable_product_output() -> Result<(), String> {
        let snapshot = summarize_emitted_css_golden_gate()?;

        assert_eq!(snapshot.schema_version, "0");
        assert_eq!(snapshot.product, "omena-benchmarks.emitted-css-golden-gate");
        assert_eq!(snapshot.benchmark_family, super::Z5_PERFORMANCE_BASELINE);
        assert_eq!(
            snapshot.emitter,
            "omena-transform-print.print_transform_cst_source_with_dialect"
        );
        assert_eq!(snapshot.check_id, "rust/benchmark/emitted-css-golden-gate");
        assert_eq!(
            snapshot.update_check_id,
            "rust/benchmark/emitted-css-golden-gate:update"
        );
        assert_eq!(
            snapshot.fixture_count,
            style_corpus().len() + bundler_productization_corpus().len()
        );
        assert!(snapshot.all_outputs_byte_stable);
        assert!(snapshot.includes_css_modules_moat_fixture);
        assert_eq!(
            snapshot.dart_sass_advisory_policy,
            "advisory-only-plain-sass-subset; css-modules-moat-never-external-gated"
        );
        assert!(snapshot.speed_claim_ready);
        assert!(
            snapshot
                .samples
                .iter()
                .any(|sample| sample.name == "css-modules-product-grid"
                    && sample.css_modules_moat
                    && !sample.dart_sass_advisory_eligible)
        );
        assert!(
            snapshot
                .samples
                .iter()
                .all(|sample| sample.deterministic_output && sample.source_map_v3_present)
        );
        Ok(())
    }

    #[test]
    fn emitted_css_golden_snapshot_is_byte_pinned() -> Result<(), String> {
        validate_emitted_css_golden_gate_snapshot(EMITTED_CSS_GOLDEN)?;
        let current = render_emitted_css_golden_gate_snapshot_json()?;
        let expected: serde_json::Value =
            serde_json::from_str(EMITTED_CSS_GOLDEN).map_err(|error| error.to_string())?;
        let current: serde_json::Value =
            serde_json::from_str(&current).map_err(|error| error.to_string())?;
        assert_eq!(current, expected);
        Ok(())
    }

    #[test]
    fn headline_axis_fidelity_measures_source_map_and_moat_preservation() -> Result<(), String> {
        let snapshot = summarize_headline_axis_fidelity()?;

        assert_eq!(snapshot.schema_version, "0");
        assert_eq!(snapshot.product, "omena-benchmarks.headline-axis-fidelity");
        assert_eq!(snapshot.benchmark_family, super::Z5_PERFORMANCE_BASELINE);
        assert_eq!(
            snapshot.measured_axis,
            "fidelity-provenance-and-runtime-loop-readiness"
        );
        assert_eq!(snapshot.sample_count, 1);
        assert!(snapshot.source_map_vlq_valid);
        assert!(snapshot.source_map_positions_valid);
        assert!(snapshot.css_modules_moat_preserved_through_minify);
        assert!(snapshot.max_provenance_overhead_basis_points > 0);
        assert!(!snapshot.runtime_loop_headline_ready);
        assert!(!snapshot.speed_claim_ready);
        assert_eq!(
            snapshot.publication_policy,
            "measurement-only-no-competitive-claim"
        );
        assert!(
            snapshot
                .samples
                .iter()
                .any(|sample| sample.name == "css-modules-product-grid"
                    && sample.decoded_source_map_segment_count > 0
                    && sample.css_modules_moat_preserved_through_minify)
        );
        Ok(())
    }

    #[test]
    fn transform_relex_baseline_records_size_swept_op_counts() -> Result<(), String> {
        let snapshot = summarize_transform_relex_baseline();

        assert_eq!(
            snapshot.product,
            "omena-benchmarks.transform-relex-baseline"
        );
        assert_eq!(snapshot.benchmark_family, super::Z5_PERFORMANCE_BASELINE);
        assert_eq!(
            snapshot.measured_operation,
            "minified-print-path-lex-materialization-op-count"
        );
        assert_eq!(snapshot.check_id, "rust/benchmark/transform-relex-baseline");
        assert_eq!(
            snapshot.update_check_id,
            "rust/benchmark/transform-relex-baseline:update"
        );
        assert!(!snapshot.speed_claim_ready);
        assert_eq!(snapshot.scale_count, 3);
        assert_eq!(
            snapshot.sample_count,
            snapshot.base_sample_count * snapshot.scale_count
        );
        assert!(snapshot.total_source_byte_length > 0);
        assert!(snapshot.total_lex_invocation_count > 0);
        assert!(snapshot.total_lex_token_count > 0);
        assert!(snapshot.lex_token_growth_exponent_milli > 0);
        assert!(snapshot.executor_spine.asserted);
        assert!(snapshot.executor_spine.red_on_full_relex_witness);
        assert!(
            snapshot
                .executor_spine
                .current_lane
                .lex_invocation_growth_exponent_milli
                <= snapshot
                    .executor_spine
                    .lex_invocation_growth_exponent_bound_milli
        );
        assert!(
            snapshot
                .executor_spine
                .current_lane
                .max_lex_tokens_per_edited_byte_milli
                <= snapshot
                    .executor_spine
                    .lex_tokens_per_edited_byte_bound_milli
        );
        assert!(
            snapshot
                .executor_spine
                .full_relex_witness_lane
                .max_lex_tokens_per_edited_byte_milli
                > snapshot
                    .executor_spine
                    .lex_tokens_per_edited_byte_bound_milli
        );
        assert!(
            snapshot.executor_spine.current_lane.total_lex_token_count
                < snapshot
                    .executor_spine
                    .full_relex_witness_lane
                    .total_lex_token_count
        );
        assert!(
            snapshot
                .executor_spine
                .current_lane
                .total_lex_splice_hit_count
                > 0
        );
        assert!(
            snapshot
                .executor_spine
                .current_lane
                .total_lex_splice_full_relex_fallback_count
                <= snapshot.executor_spine.lex_splice_full_relex_fallback_bound
        );
        assert_eq!(
            snapshot
                .executor_spine
                .full_relex_witness_lane
                .total_lex_splice_hit_count,
            0
        );
        assert!(
            snapshot
                .samples
                .iter()
                .all(|sample| sample.lex_invocation_count > 0 && sample.lex_token_count > 0)
        );
        assert!(
            snapshot
                .executor_spine
                .current_lane
                .samples
                .iter()
                .all(|sample| sample.lex_splice_hit_count > 0
                    && sample.lex_splice_full_relex_fallback_count <= 1)
        );
        assert!(
            snapshot
                .scales
                .windows(2)
                .all(|pair| pair[0].source_byte_length < pair[1].source_byte_length)
        );
        Ok(())
    }

    #[test]
    fn growth_exponent_fitter_is_public_and_deterministic() {
        let linear = super::fit_log_log_growth_exponent(&[(100, 300), (200, 600), (400, 1200)]);
        assert!(linear.is_some_and(|linear| (linear - 1.0).abs() < 0.000_001));
        assert!(super::fit_log_log_growth_exponent(&[(100, 300), (200, 600)]).is_none());
    }

    #[test]
    fn transform_relex_baseline_snapshot_is_byte_pinned() -> Result<(), String> {
        validate_transform_relex_baseline_snapshot(TRANSFORM_RELEX_BASELINE)?;
        Ok(())
    }

    #[test]
    fn bundler_productization_surface_declares_corpus_lanes_and_no_speed_claim()
    -> Result<(), String> {
        let snapshot = summarize_bundler_productization_benchmark_surface();

        assert_eq!(snapshot.schema_version, "0");
        assert_eq!(
            snapshot.product,
            "omena-benchmarks.bundler-productization-surface"
        );
        assert_eq!(snapshot.benchmark_family, "bundler-productization");
        assert_eq!(snapshot.status, "measurementSurfaceReadyNoSpeedClaim");
        assert_eq!(
            snapshot.corpus_sample_count,
            bundler_productization_corpus().len()
        );
        assert_eq!(snapshot.corpus_sample_count, 3);
        assert_eq!(snapshot.lane_count, 3);
        assert!(snapshot.includes_napi_in_process_lane);
        assert!(snapshot.includes_cli_spawn_lane);
        assert!(snapshot.includes_lightningcss_comparator_lane);
        assert!(snapshot.includes_memory_rss_metric);
        assert!(snapshot.includes_provenance_mode_split);
        assert!(!snapshot.speed_claim_ready);
        assert!(
            snapshot
                .samples
                .iter()
                .any(|sample| sample.name == "bootstrap-reboot-v5.3.3"
                    && sample.dialect == "css"
                    && sample.source.contains("Bootstrap Reboot v5.3.3")
                    && sample.line_count > 100)
        );
        assert!(
            snapshot
                .samples
                .iter()
                .any(|sample| sample.name == "next-dashboard-shell-scss"
                    && sample.dialect == "scss"
                    && sample.source.contains(".dashboardShell")
                    && sample.source.contains(".metricCard")
                    && sample.source.contains("composes: actionButton")
                    && sample.line_count > 80)
        );
        assert!(
            snapshot
                .samples
                .iter()
                .any(|sample| sample.name == "css-modules-product-grid"
                    && sample.dialect == "css"
                    && sample.source.contains("composes: filterButton")
                    && sample.source.contains(":global(.is-keyboard-user)"))
        );
        assert!(snapshot.samples.iter().all(|sample| {
            !sample.source.contains(".d-flex-159") && !sample.source.contains(".card191")
        }));
        assert!(
            snapshot
                .measured_operations
                .contains(&"napi-vs-cli-spawn-process-model-delta")
        );
        assert_eq!(
            snapshot.timing_policy,
            "no-speed-claim-without-recorded-full-run-artifact"
        );

        let serialized = serde_json::to_value(&snapshot).map_err(|error| error.to_string())?;
        assert_eq!(
            serialized
                .pointer("/speedClaimReady")
                .and_then(|value| value.as_bool()),
            Some(false)
        );
        assert_eq!(
            serialized
                .pointer("/lanes/0/lane")
                .and_then(|value| value.as_str()),
            Some("omena-napi-in-process")
        );
        Ok(())
    }
}
