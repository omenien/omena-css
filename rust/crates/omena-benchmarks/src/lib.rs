pub const Z5_PERFORMANCE_BASELINE: &str = "z5-performance-baseline";

mod corpus;

use omena_parser::StyleDialect;
use serde::Serialize;

pub use corpus::{
    StyleCorpusSampleSnapshotV0, StyleCorpusSnapshotV0, StyleSample, style_corpus,
    summarize_style_corpus_snapshot,
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
        command: "pnpm cme-check run rust/z5-criterion-surface-snapshot",
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
        measure_legacy_parser_product_sample, measure_omena_parser_product_sample,
        parser_product_benchmark_boundaries, style_corpus, summarize_criterion_surface_snapshot,
        summarize_parser_product_benchmark_readiness, summarize_style_corpus_snapshot,
        validate_legacy_style_sample, validate_omena_style_sample,
        validate_parser_product_benchmark_boundary_symmetry,
    };

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
        assert_eq!(snapshot.corpus_sample_count, 7);
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
        assert_eq!(snapshot.benchmark_function_count, 38);
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
            Some(&serde_json::json!(38))
        );
        assert_eq!(
            serialized
                .pointer("/timingPolicy")
                .and_then(|value| value.as_str()),
            Some("no-local-timing-claim-without-full-criterion-run")
        );
        Ok(())
    }
}
