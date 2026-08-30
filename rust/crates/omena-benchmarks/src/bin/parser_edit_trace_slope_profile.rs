use omena_parser::{
    ParseReuseCache, StyleDialect, collect_emission_selector_facts_from_cst,
    collect_style_fact_collection, facts_from_cst, parse_with_reuse_cache,
    with_omena_parser_parse_instrumentation,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    hint::black_box,
    path::{Path, PathBuf},
    time::Instant,
};

const SAMPLE_BATCH_COUNT: usize = 7;
const EDIT_LOCAL_THRESHOLD_NS: u128 = 3_200_000;
const DISPOSITION_POLICY: &str =
    "absolute-per-band-p90-budgets-and-two-run-two-independent-source-reentry-v1";
const PARK_SCOPE: &str =
    "hand-authored, non-minified, non-dist stylesheets below 100KB from independent sources";

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CorpusManifest {
    schema_version: String,
    product: String,
    corpora: Vec<CorpusEntry>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CorpusEntry {
    id: String,
    band: String,
    source_project: String,
    source_kind: String,
    dialect: String,
    expected_bytes: usize,
    sha256: String,
    local_path: Option<String>,
    remote_file_name: Option<String>,
    source_url: String,
    version: String,
    license: String,
    park_eligible: bool,
    park_scope_reason: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ParserEditBaseline {
    schema_version: String,
    product: String,
    generated_at_utc: String,
    omena_git_sha: String,
    measurement: String,
    allowed_regression_ratio: f64,
    reentry_regression_ratio: f64,
    required_independent_source_count: usize,
    pins: Vec<ParserEditBaselinePin>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ParserEditBaselinePin {
    corpus_id: String,
    band: String,
    source_project: String,
    source_kind: String,
    source_url: String,
    source_version: String,
    source_sha256: String,
    source_bytes: usize,
    park_eligible: bool,
    measured_minimum_path_p90_nanoseconds: u128,
    allowed_minimum_path_p90_nanoseconds: u128,
}

struct LoadedCorpus {
    entry: CorpusEntry,
    dialect: StyleDialect,
    source: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct Profile {
    schema_version: &'static str,
    product: &'static str,
    measurement_pin: String,
    release_build: bool,
    sample_batch_count: usize,
    corpus_manifest: String,
    corpus_manifest_schema_version: String,
    corpus_manifest_product: String,
    edit_shapes: Vec<&'static str>,
    measurement_paths: Vec<&'static str>,
    edit_local_threshold_nanoseconds: u128,
    disposition_policy: &'static str,
    park_scope: &'static str,
    yardstick_validation: &'static str,
    original_research_trigger: OriginalResearchTrigger,
    baseline_measurement_pin: String,
    band_budget_comparisons: Vec<BandBudgetComparison>,
    disposition_inputs: DispositionInputs,
    disposition: &'static str,
    summaries: Vec<BandSummary>,
    samples: Vec<Sample>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct OriginalResearchTrigger {
    source_scope: &'static str,
    real_world_p90_nanoseconds: u128,
    threshold_nanoseconds: u128,
    threshold_consumption_milli: u128,
    fires: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct DispositionInputs {
    comparison_run_count: usize,
    per_band_budget_count: usize,
    within_budget_count: usize,
    over_budget_count: usize,
    reentry_regression_ratio_milli: u128,
    qualifying_source_count: usize,
    qualifying_independent_source_count: usize,
    required_independent_source_count: usize,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct BandBudgetComparison {
    corpus_id: String,
    band: String,
    source_project: String,
    source_kind: String,
    source_url: String,
    source_version: String,
    source_sha256: String,
    source_bytes: usize,
    park_eligible: bool,
    pinned_minimum_path_p90_nanoseconds: u128,
    allowed_minimum_path_p90_nanoseconds: u128,
    current_minimum_path_p90_nanoseconds: u128,
    within_budget: bool,
    reentry_threshold_nanoseconds: u128,
    reentry_candidate: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct BandSummary {
    corpus_id: String,
    band: String,
    source_project: String,
    source_kind: String,
    source_url: String,
    source_version: String,
    source_sha256: String,
    source_bytes: usize,
    park_eligible: bool,
    park_scope_reason: String,
    minimum_path: &'static str,
    minimum_path_p90_nanoseconds: u128,
    minimum_path_nanoseconds_per_byte: f64,
    normalized_slope_ratio: f64,
    path_p90_nanoseconds: BTreeMap<&'static str, u128>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct Sample {
    corpus_id: String,
    band: String,
    source_project: String,
    source_kind: String,
    source_url: String,
    source_version: String,
    source_license: String,
    source_sha256: String,
    dialect: &'static str,
    source_bytes: usize,
    edited_bytes: usize,
    changed_bytes: usize,
    park_eligible: bool,
    edit_shape: &'static str,
    measurement_path: &'static str,
    iterations_per_batch: usize,
    nanoseconds_p50: u128,
    nanoseconds_p90: u128,
    parse_invocations_per_iteration: f64,
    parse_tokens_per_iteration: f64,
}

#[derive(Clone, Copy)]
enum MeasurementPath {
    FullCollection,
    SameTextReuse,
    EditedTextReuse,
}

impl MeasurementPath {
    const ALL: [Self; 3] = [
        Self::FullCollection,
        Self::SameTextReuse,
        Self::EditedTextReuse,
    ];

    const fn label(self) -> &'static str {
        match self {
            Self::FullCollection => "full-parse-and-collection",
            Self::SameTextReuse => "same-text-reuse-cache-and-cst-facts",
            Self::EditedTextReuse => "edited-text-reuse-cache-and-cst-facts",
        }
    }
}

#[derive(Clone, Copy)]
enum EditShape {
    Suffix,
    MidFileInsert,
    Delete,
    TransientUnbalancedBrace,
    Prefix,
}

impl EditShape {
    const ALL: [Self; 5] = [
        Self::Suffix,
        Self::MidFileInsert,
        Self::Delete,
        Self::TransientUnbalancedBrace,
        Self::Prefix,
    ];

    const fn label(self) -> &'static str {
        match self {
            Self::Suffix => "suffix",
            Self::MidFileInsert => "mid-file-insert",
            Self::Delete => "delete",
            Self::TransientUnbalancedBrace => "transiently-unbalanced-left-brace",
            Self::Prefix => "prefix",
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manifest_path = PathBuf::from(
        std::env::var("OMENA_PARSER_EDIT_CORPUS_MANIFEST").unwrap_or_else(|_| {
            "rust/crates/omena-benchmarks/fixtures/parser-edit-corpus-v0.json".to_string()
        }),
    );
    let manifest: CorpusManifest =
        serde_json::from_str(fs::read_to_string(&manifest_path)?.as_str())?;
    if manifest.schema_version != "0" || manifest.product != "omena-benchmarks.parser-edit-corpus" {
        return Err("unsupported parser-edit corpus manifest".into());
    }
    let baseline_path = PathBuf::from(std::env::var("OMENA_PARSER_EDIT_BASELINE").unwrap_or_else(
        |_| "rust/crates/omena-benchmarks/baselines/parser-edit-slope-baseline-v0.json".to_string(),
    ));
    let baseline: ParserEditBaseline =
        serde_json::from_str(fs::read_to_string(&baseline_path)?.as_str())?;
    validate_baseline_header(&baseline)?;
    let remote_root = std::env::var("OMENA_PARSER_EDIT_CORPUS_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("benchmark-artifacts/parser-edit-corpus"));
    let corpora = manifest
        .corpora
        .into_iter()
        .map(|entry| load_corpus(entry, &remote_root))
        .collect::<Result<Vec<_>, _>>()?;
    let mut samples = Vec::with_capacity(corpora.len() * EditShape::ALL.len() * 3);

    for corpus in &corpora {
        for shape in EditShape::ALL {
            let edited = apply_edit(corpus.source.as_str(), shape);
            let changed_bytes = corpus.source.len().abs_diff(edited.len());
            let iterations_per_batch = iterations_for_size(edited.len());
            for path in MeasurementPath::ALL {
                let measurement = measure_path(
                    path,
                    corpus.source.as_str(),
                    edited.as_str(),
                    corpus.dialect,
                    iterations_per_batch,
                );
                samples.push(Sample {
                    corpus_id: corpus.entry.id.clone(),
                    band: corpus.entry.band.clone(),
                    source_project: corpus.entry.source_project.clone(),
                    source_kind: corpus.entry.source_kind.clone(),
                    source_url: corpus.entry.source_url.clone(),
                    source_version: corpus.entry.version.clone(),
                    source_license: corpus.entry.license.clone(),
                    source_sha256: corpus.entry.sha256.clone(),
                    dialect: dialect_label(corpus.dialect),
                    source_bytes: corpus.source.len(),
                    edited_bytes: edited.len(),
                    changed_bytes,
                    park_eligible: corpus.entry.park_eligible,
                    edit_shape: shape.label(),
                    measurement_path: path.label(),
                    iterations_per_batch,
                    nanoseconds_p50: measurement.p50,
                    nanoseconds_p90: measurement.p90,
                    parse_invocations_per_iteration: measurement.parse_invocations_per_iteration,
                    parse_tokens_per_iteration: measurement.parse_tokens_per_iteration,
                });
            }
        }
    }

    let mut summaries = corpora
        .iter()
        .map(|corpus| summarize_corpus(corpus, &samples))
        .collect::<Result<Vec<_>, Box<dyn std::error::Error>>>()?;
    let reference_ns_per_byte = summaries
        .first()
        .ok_or("parser-edit corpus cannot be empty")?
        .minimum_path_nanoseconds_per_byte;
    for summary in &mut summaries {
        summary.normalized_slope_ratio =
            summary.minimum_path_nanoseconds_per_byte / reference_ns_per_byte;
    }

    let band_budget_comparisons = compare_band_budgets(&summaries, &baseline)?;
    let qualifying_sources = reentry_source_projects(&band_budget_comparisons);
    let disposition = parser_edit_disposition(
        qualifying_sources.len(),
        baseline.required_independent_source_count,
    );
    let original_research_p90 = 4_355_000;
    let profile = Profile {
        schema_version: "0",
        product: "omena-benchmarks.parser-edit-trace-slope-profile",
        measurement_pin: std::env::var("OMENA_MEASUREMENT_PIN")
            .unwrap_or_else(|_| "unrecorded".to_string()),
        release_build: !cfg!(debug_assertions),
        sample_batch_count: SAMPLE_BATCH_COUNT,
        corpus_manifest: manifest_path.display().to_string(),
        corpus_manifest_schema_version: "0".to_string(),
        corpus_manifest_product: "omena-benchmarks.parser-edit-corpus".to_string(),
        edit_shapes: EditShape::ALL.iter().map(|shape| shape.label()).collect(),
        measurement_paths: MeasurementPath::ALL
            .iter()
            .map(|path| path.label())
            .collect(),
        edit_local_threshold_nanoseconds: EDIT_LOCAL_THRESHOLD_NS,
        disposition_policy: DISPOSITION_POLICY,
        park_scope: PARK_SCOPE,
        yardstick_validation: "unvalidated-wall-clock-yardstick-with-load-bearing-absolute-per-band-p90-pins",
        original_research_trigger: OriginalResearchTrigger {
            source_scope: "measured real-world third-party distribution corpus",
            real_world_p90_nanoseconds: original_research_p90,
            threshold_nanoseconds: EDIT_LOCAL_THRESHOLD_NS,
            threshold_consumption_milli: original_research_p90 * 1_000 / EDIT_LOCAL_THRESHOLD_NS,
            fires: original_research_p90 >= EDIT_LOCAL_THRESHOLD_NS,
        },
        baseline_measurement_pin: baseline.omena_git_sha.clone(),
        disposition_inputs: DispositionInputs {
            comparison_run_count: usize::from(!baseline.generated_at_utc.is_empty()) + 1,
            per_band_budget_count: band_budget_comparisons.len(),
            within_budget_count: band_budget_comparisons
                .iter()
                .filter(|comparison| comparison.within_budget)
                .count(),
            over_budget_count: band_budget_comparisons
                .iter()
                .filter(|comparison| !comparison.within_budget)
                .count(),
            reentry_regression_ratio_milli: (baseline.reentry_regression_ratio * 1_000.0).round()
                as u128,
            qualifying_source_count: band_budget_comparisons
                .iter()
                .filter(|comparison| comparison.reentry_candidate)
                .count(),
            qualifying_independent_source_count: qualifying_sources.len(),
            required_independent_source_count: baseline.required_independent_source_count,
        },
        band_budget_comparisons,
        disposition,
        summaries,
        samples,
    };
    println!("{}", serde_json::to_string_pretty(&profile)?);
    Ok(())
}

fn validate_baseline_header(
    baseline: &ParserEditBaseline,
) -> Result<(), Box<dyn std::error::Error>> {
    if baseline.schema_version != "0"
        || baseline.product != "omena-benchmarks.parser-edit-slope-baseline"
        || baseline.measurement != "absolute-minimum-path-p90-nanoseconds"
    {
        return Err("unsupported parser-edit baseline".into());
    }
    if baseline.generated_at_utc.is_empty()
        || baseline.omena_git_sha.is_empty()
        || !(0.0..1.0).contains(&baseline.allowed_regression_ratio)
        || baseline.reentry_regression_ratio < 1.0
        || baseline.required_independent_source_count < 2
    {
        return Err("invalid parser-edit baseline policy".into());
    }
    Ok(())
}

fn compare_band_budgets(
    summaries: &[BandSummary],
    baseline: &ParserEditBaseline,
) -> Result<Vec<BandBudgetComparison>, Box<dyn std::error::Error>> {
    if summaries.len() != baseline.pins.len() {
        return Err("parser-edit baseline band count drifted".into());
    }
    summaries
        .iter()
        .zip(&baseline.pins)
        .map(|(summary, pin)| {
            if pin.corpus_id != summary.corpus_id
                || pin.band != summary.band
                || pin.source_project != summary.source_project
                || pin.source_kind != summary.source_kind
                || pin.source_url != summary.source_url
                || pin.source_version != summary.source_version
                || pin.source_sha256 != summary.source_sha256
                || pin.source_bytes != summary.source_bytes
                || pin.park_eligible != summary.park_eligible
            {
                return Err(
                    format!("{} parser-edit baseline identity drifted", summary.band).into(),
                );
            }
            let expected_budget = ((pin.measured_minimum_path_p90_nanoseconds as f64)
                * (1.0 + baseline.allowed_regression_ratio))
                .ceil() as u128;
            if pin.allowed_minimum_path_p90_nanoseconds != expected_budget {
                return Err(format!("{} parser-edit absolute budget drifted", summary.band).into());
            }
            let reentry_threshold = ((pin.measured_minimum_path_p90_nanoseconds as f64)
                * baseline.reentry_regression_ratio)
                .ceil() as u128;
            Ok(BandBudgetComparison {
                corpus_id: summary.corpus_id.clone(),
                band: summary.band.clone(),
                source_project: summary.source_project.clone(),
                source_kind: summary.source_kind.clone(),
                source_url: summary.source_url.clone(),
                source_version: summary.source_version.clone(),
                source_sha256: summary.source_sha256.clone(),
                source_bytes: summary.source_bytes,
                park_eligible: summary.park_eligible,
                pinned_minimum_path_p90_nanoseconds: pin.measured_minimum_path_p90_nanoseconds,
                allowed_minimum_path_p90_nanoseconds: pin.allowed_minimum_path_p90_nanoseconds,
                current_minimum_path_p90_nanoseconds: summary.minimum_path_p90_nanoseconds,
                within_budget: summary.minimum_path_p90_nanoseconds
                    <= pin.allowed_minimum_path_p90_nanoseconds,
                reentry_threshold_nanoseconds: reentry_threshold,
                reentry_candidate: summary.park_eligible
                    && summary.minimum_path_p90_nanoseconds >= reentry_threshold,
            })
        })
        .collect()
}

fn reentry_source_projects(comparisons: &[BandBudgetComparison]) -> BTreeSet<&str> {
    comparisons
        .iter()
        .filter(|comparison| comparison.reentry_candidate)
        .map(|comparison| comparison.source_project.as_str())
        .collect()
}

const fn parser_edit_disposition(
    qualifying_independent_source_count: usize,
    required_independent_source_count: usize,
) -> &'static str {
    if qualifying_independent_source_count >= required_independent_source_count {
        "draft-edit-local-parser-design"
    } else {
        "park-edit-local-parser"
    }
}

fn load_corpus(
    entry: CorpusEntry,
    remote_root: &Path,
) -> Result<LoadedCorpus, Box<dyn std::error::Error>> {
    let path = if let Some(local_path) = &entry.local_path {
        PathBuf::from(local_path)
    } else {
        remote_root.join(
            entry
                .remote_file_name
                .as_deref()
                .ok_or("remote corpus entry requires remoteFileName")?,
        )
    };
    let source = fs::read_to_string(&path)?;
    if source.len() != entry.expected_bytes {
        return Err(format!(
            "parser-edit corpus {} has {} bytes; expected {}",
            entry.id,
            source.len(),
            entry.expected_bytes
        )
        .into());
    }
    let dialect = match entry.dialect.as_str() {
        "css" => StyleDialect::Css,
        "scss" => StyleDialect::Scss,
        "sass" => StyleDialect::Sass,
        "less" => StyleDialect::Less,
        other => return Err(format!("unsupported corpus dialect {other}").into()),
    };
    Ok(LoadedCorpus {
        entry,
        dialect,
        source,
    })
}

struct PathMeasurement {
    p50: u128,
    p90: u128,
    parse_invocations_per_iteration: f64,
    parse_tokens_per_iteration: f64,
}

fn measure_path(
    path: MeasurementPath,
    original: &str,
    edited: &str,
    dialect: StyleDialect,
    iterations_per_batch: usize,
) -> PathMeasurement {
    let mut cache = ParseReuseCache::default();
    match path {
        MeasurementPath::FullCollection => {
            for _ in 0..2 {
                black_box(collect_style_fact_collection(black_box(edited), dialect));
            }
        }
        MeasurementPath::SameTextReuse => {
            for _ in 0..2 {
                consume_cached(edited, dialect, &mut cache);
            }
        }
        MeasurementPath::EditedTextReuse => {
            for _ in 0..2 {
                consume_cached(original, dialect, &mut cache);
                consume_cached(edited, dialect, &mut cache);
            }
            consume_cached(original, dialect, &mut cache);
        }
    }

    let mut timings = Vec::with_capacity(SAMPLE_BATCH_COUNT);
    let mut parse_invocations = 0_u64;
    let mut parse_tokens = 0_u64;
    for _ in 0..SAMPLE_BATCH_COUNT {
        let mut elapsed = 0_u128;
        for _ in 0..iterations_per_batch {
            let (_, snapshot) = with_omena_parser_parse_instrumentation(|| {
                let started = Instant::now();
                match path {
                    MeasurementPath::FullCollection => {
                        black_box(collect_style_fact_collection(black_box(edited), dialect));
                    }
                    MeasurementPath::SameTextReuse | MeasurementPath::EditedTextReuse => {
                        consume_cached(edited, dialect, &mut cache);
                    }
                }
                elapsed += started.elapsed().as_nanos();
            });
            parse_invocations += snapshot.parse_invocation_count;
            parse_tokens += snapshot.parse_token_count;
            if matches!(path, MeasurementPath::EditedTextReuse) {
                consume_cached(original, dialect, &mut cache);
            }
        }
        timings.push(elapsed / iterations_per_batch as u128);
    }
    timings.sort_unstable();
    let measured_iteration_count = (SAMPLE_BATCH_COUNT * iterations_per_batch) as f64;
    PathMeasurement {
        p50: timings[timings.len() / 2],
        p90: timings[percentile_index(timings.len(), 90)],
        parse_invocations_per_iteration: parse_invocations as f64 / measured_iteration_count,
        parse_tokens_per_iteration: parse_tokens as f64 / measured_iteration_count,
    }
}

fn consume_cached(text: &str, dialect: StyleDialect, cache: &mut ParseReuseCache) {
    let parsed = parse_with_reuse_cache(black_box(text), dialect, cache);
    let facts = facts_from_cst(text, &parsed);
    let emission = collect_emission_selector_facts_from_cst(text, &parsed);
    black_box((facts, emission, parsed.errors().len()));
}

fn summarize_corpus(
    corpus: &LoadedCorpus,
    samples: &[Sample],
) -> Result<BandSummary, Box<dyn std::error::Error>> {
    let mut path_p90_nanoseconds = BTreeMap::new();
    for path in MeasurementPath::ALL {
        let p90 = samples
            .iter()
            .filter(|sample| {
                sample.corpus_id == corpus.entry.id && sample.measurement_path == path.label()
            })
            .map(|sample| sample.nanoseconds_p90)
            .max()
            .ok_or("every corpus/path pair must have edit-shape samples")?;
        path_p90_nanoseconds.insert(path.label(), p90);
    }
    let (minimum_path, minimum_path_p90_nanoseconds) = path_p90_nanoseconds
        .iter()
        .min_by_key(|(_, value)| **value)
        .map(|(path, value)| (*path, *value))
        .ok_or("every corpus must have path measurements")?;
    Ok(BandSummary {
        corpus_id: corpus.entry.id.clone(),
        band: corpus.entry.band.clone(),
        source_project: corpus.entry.source_project.clone(),
        source_kind: corpus.entry.source_kind.clone(),
        source_url: corpus.entry.source_url.clone(),
        source_version: corpus.entry.version.clone(),
        source_sha256: corpus.entry.sha256.clone(),
        source_bytes: corpus.source.len(),
        park_eligible: corpus.entry.park_eligible,
        park_scope_reason: corpus.entry.park_scope_reason.clone(),
        minimum_path,
        minimum_path_p90_nanoseconds,
        minimum_path_nanoseconds_per_byte: minimum_path_p90_nanoseconds as f64
            / corpus.source.len() as f64,
        normalized_slope_ratio: 0.0,
        path_p90_nanoseconds,
    })
}

fn apply_edit(source: &str, shape: EditShape) -> String {
    let midpoint = nearest_char_boundary(source, source.len() / 2);
    match shape {
        EditShape::Suffix => format!("{source}\n"),
        EditShape::MidFileInsert => {
            let mut edited = source.to_string();
            edited.insert_str(midpoint, "/*omena*/");
            edited
        }
        EditShape::Delete => {
            let mut end = nearest_char_boundary(source, (midpoint + 8).min(source.len()));
            if end == midpoint {
                end = source.len();
            }
            let mut edited = source.to_string();
            edited.replace_range(midpoint..end, "");
            edited
        }
        EditShape::TransientUnbalancedBrace => {
            let mut edited = source.to_string();
            edited.insert(midpoint, '{');
            edited
        }
        EditShape::Prefix => format!("/*omena*/{source}"),
    }
}

fn nearest_char_boundary(source: &str, mut offset: usize) -> usize {
    while offset > 0 && !source.is_char_boundary(offset) {
        offset -= 1;
    }
    offset
}

fn percentile_index(sample_count: usize, percentile: usize) -> usize {
    sample_count
        .saturating_mul(percentile)
        .div_ceil(100)
        .saturating_sub(1)
        .min(sample_count.saturating_sub(1))
}

const fn iterations_for_size(byte_length: usize) -> usize {
    if byte_length <= 2 * 1024 {
        400
    } else if byte_length <= 16 * 1024 {
        50
    } else if byte_length <= 64 * 1024 {
        15
    } else if byte_length <= 256 * 1024 {
        4
    } else {
        1
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

#[cfg(test)]
mod tests {
    use super::parser_edit_disposition;

    #[test]
    fn disposition_requires_two_independent_reentry_sources() {
        assert_eq!(
            parser_edit_disposition(2, 2),
            "draft-edit-local-parser-design"
        );
        assert_eq!(parser_edit_disposition(1, 2), "park-edit-local-parser");
        assert_eq!(parser_edit_disposition(0, 2), "park-edit-local-parser");
    }
}
