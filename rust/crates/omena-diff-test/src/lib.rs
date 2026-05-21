//! Differential corpus harness for the omena-css parser stack.
//!
//! This crate is the Rust workspace home for parser differential checks that
//! were previously represented only by runner scripts. It treats
//! `engine-style-parser` as a legacy oracle and `omena-parser` as the product
//! parser surface.

use std::collections::BTreeSet;

use engine_style_parser::{parse_style_module, summarize_css_modules_intermediate};
use omena_parser::{StyleDialect, summarize_omena_parser_style_facts};
pub use omena_testkit::{
    CmeFixtureExpectationV0, CmeFixtureFileV0, CmeFixtureV0, OmenaTestkitFixtureSeedV0,
    parse_cme_fixture_v0, summarize_omena_testkit_fixture_seed_corpus,
};
use serde::Serialize;

/// Style dialects that can be compared against the legacy parser oracle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum DiffDialect {
    /// Plain CSS or CSS Modules.
    Css,
    /// SCSS or SCSS Modules.
    Scss,
    /// Less or Less Modules.
    Less,
}

impl DiffDialect {
    fn as_label(self) -> &'static str {
        match self {
            Self::Css => "css",
            Self::Scss => "scss",
            Self::Less => "less",
        }
    }

    fn as_omena_dialect(self) -> StyleDialect {
        match self {
            Self::Css => StyleDialect::Css,
            Self::Scss => StyleDialect::Scss,
            Self::Less => StyleDialect::Less,
        }
    }
}

/// A parser differential fixture.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParserDifferentialFixture {
    /// Stable fixture label printed by boundary reports.
    pub label: &'static str,
    /// Module path used by the legacy parser to infer dialect.
    pub file_path: &'static str,
    /// Fixture source text.
    pub source: &'static str,
    /// Fixture dialect.
    pub dialect: DiffDialect,
}

/// One named field comparison in a differential report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiffFieldReport {
    /// Field name being compared.
    pub field: &'static str,
    /// Sorted expected values from the legacy oracle.
    pub legacy_values: Vec<String>,
    /// Sorted actual values from the omena parser surface.
    pub omena_values: Vec<String>,
    /// Whether both sides match exactly after normalization.
    pub matches: bool,
}

/// Differential result for one parser fixture.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ParserDifferentialReport {
    /// Schema version for this boundary report.
    pub schema_version: &'static str,
    /// Product surface name.
    pub product: &'static str,
    /// Fixture label.
    pub label: &'static str,
    /// Fixture dialect.
    pub dialect: &'static str,
    /// Field-level comparisons.
    pub fields: Vec<DiffFieldReport>,
    /// Whether every field matched.
    pub all_fields_match: bool,
}

/// Boundary summary for the omena-css differential harness.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaDiffTestBoundarySummary {
    /// Schema version for the boundary summary.
    pub schema_version: &'static str,
    /// Product surface name.
    pub product: &'static str,
    /// Owning omena-css layer.
    pub owner: &'static str,
    /// Compared fixture count.
    pub parser_legacy_fixture_count: usize,
    /// Whether all parser-vs-legacy fixtures matched.
    pub all_parser_legacy_fixtures_match: bool,
    /// M3 reusable fixture seeds intended for future omena-testkit promotion.
    pub m3_fixture_seed_count: usize,
    /// Whether every M3 fixture seed parses with the shared fixture grammar.
    pub all_m3_fixture_seeds_parse: bool,
    /// WPT-style seed fixture count.
    pub wpt_seed_fixture_count: usize,
    /// Whether WPT seed corpus metadata and known-failure policy are valid.
    pub all_wpt_seed_metadata_valid: bool,
    /// WPT-style seed metadata report.
    pub wpt_seed_metadata_report: WptSeedCorpusMetadataReportV0,
    /// Named evidence gates closed by this crate.
    pub closed_gates: Vec<&'static str>,
    /// Field-level reports for every seed fixture.
    pub reports: Vec<ParserDifferentialReport>,
    /// M3 fixture seed corpus report.
    pub m3_fixture_seed_report: M3FixtureSeedCorpusReportV0,
}

/// M3 fixture seed lane.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum M3FixtureLaneV0 {
    /// Sass module graph and Sass-language false-positive behavior.
    SassLanguage,
    /// Cascade proof obligations attached to transform safety.
    CascadeProof,
    /// Abstract-value provenance explanations.
    Provenance,
    /// k-CFA and reduced-product abstract-value behavior.
    AbstractValue,
}

/// One reusable M3 fixture seed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct M3FixtureSeedV0 {
    /// Stable fixture label.
    pub label: &'static str,
    /// Fixture lane.
    pub lane: M3FixtureLaneV0,
    /// Raw `cme-fixture-v0` text.
    pub raw: &'static str,
    /// Product surfaces expected to consume this fixture.
    pub expected_products: &'static [&'static str],
    /// Promotion target for M4.
    pub promotion_target: &'static str,
}

/// Parsed M3 fixture seed evidence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct M3FixtureSeedReportV0 {
    /// Stable fixture label.
    pub label: &'static str,
    /// Fixture lane.
    pub lane: M3FixtureLaneV0,
    /// Whether the fixture parses with `cme-fixture-v0`.
    pub parses: bool,
    /// Parse error when present.
    pub parse_error: Option<String>,
    /// Parsed file count.
    pub file_count: usize,
    /// Parsed expectation count.
    pub expectation_count: usize,
    /// Expected product surfaces.
    pub expected_products: Vec<&'static str>,
    /// Promotion target for M4.
    pub promotion_target: &'static str,
}

/// M3 fixture seed corpus summary.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct M3FixtureSeedCorpusReportV0 {
    /// Schema version.
    pub schema_version: &'static str,
    /// Product surface name.
    pub product: &'static str,
    /// Fixture grammar.
    pub fixture_grammar: &'static str,
    /// Fixture count.
    pub fixture_count: usize,
    /// Covered lane count.
    pub lane_count: usize,
    /// Whether every seed parses with the shared fixture grammar.
    pub all_seeds_parse: bool,
    /// Seed reports.
    pub reports: Vec<M3FixtureSeedReportV0>,
}

/// WPT-style seed corpus metadata summary.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WptSeedCorpusMetadataReportV0 {
    /// Schema version.
    pub schema_version: &'static str,
    /// Product surface name.
    pub product: &'static str,
    /// WPT conformance stage.
    pub stage: String,
    /// Upstream source pin.
    pub source_pin: String,
    /// Chunk count.
    pub chunk_count: usize,
    /// Fixture count across chunks.
    pub fixture_count: usize,
    /// Known-failure entry count.
    pub known_failure_count: usize,
    /// Whether the current policy is already blocking Stage 2.
    pub stage2_blocking: bool,
    /// Minimum fixture count required before Stage 2 can become blocking.
    pub required_min_fixture_count_for_stage2: usize,
    /// Required consecutive green advisory runs before Stage 2 promotion.
    pub required_consecutive_green_runs: usize,
    /// Current consecutive green advisory run count for this pinned corpus.
    pub consecutive_green_runs: usize,
    /// Maximum review interval for known-failure entries.
    pub known_failure_review_interval_days: usize,
    /// Whether Stage 2 promotion prerequisites are currently satisfied.
    pub stage2_candidate_ready: bool,
    /// Current blockers that prevent Stage 2 promotion.
    pub stage2_promotion_blockers: Vec<&'static str>,
    /// Whether the seed metadata is internally consistent.
    pub all_metadata_valid: bool,
    /// Named gates closed by this report.
    pub closed_gates: Vec<&'static str>,
}

const WPT_SEED_MANIFEST_SOURCE: &str = include_str!("../wpt-corpus/manifest.json");
const WPT_SEED_VALUES_CHUNK_SOURCE: &str = include_str!("../wpt-corpus/css-values.json");
const WPT_SEED_KNOWN_FAILURE_POLICY_SOURCE: &str =
    include_str!("../known-failures/wpt-seed-policy.toml");

/// Seed corpus that exercises the legacy-compatible parser differential path.
pub const PARSER_LEGACY_SEED_FIXTURES: &[ParserDifferentialFixture] = &[
    ParserDifferentialFixture {
        label: "css-custom-properties",
        file_path: "/fixture.module.css",
        source: ":root { --brand: red; }\n.card { color: var(--brand); }",
        dialect: DiffDialect::Css,
    },
    ParserDifferentialFixture {
        label: "css-selector-list-custom-properties",
        file_path: "/selector-list.module.css",
        source: ".card, .tile { --tone: red; color: var(--tone); }\n.card__icon { color: blue; }",
        dialect: DiffDialect::Css,
    },
    ParserDifferentialFixture {
        label: "scss-nested-bem-and-sass-vars",
        file_path: "/fixture.module.scss",
        source: "@use \"./tokens\";\n@forward \"./theme\";\n$gap: 1rem;\n.card { &__icon { color: $gap; } }",
        dialect: DiffDialect::Scss,
    },
    ParserDifferentialFixture {
        label: "scss-use-forward-import-and-mixin",
        file_path: "/module-edges.module.scss",
        source: "@use \"./tokens\" as tokens;\n@forward \"./theme\" show tone;\n@import \"./legacy\";\n$gap: 1rem;\n@mixin raised($depth) { box-shadow: 0 0 $depth black; }\n.card { @include raised($gap); }",
        dialect: DiffDialect::Scss,
    },
    ParserDifferentialFixture {
        label: "less-variable-and-selector",
        file_path: "/fixture.module.less",
        source: "@color: red;\n.card { color: @color; }",
        dialect: DiffDialect::Less,
    },
    ParserDifferentialFixture {
        label: "less-nested-selector-and-custom-property",
        file_path: "/nested.module.less",
        source: "@color: red;\n.card { --tone: @color; &__icon { color: var(--tone); } }",
        dialect: DiffDialect::Less,
    },
];

/// M3 reusable fixture seeds for future `omena-testkit` promotion.
pub const M3_THEORETICAL_MOAT_FIXTURE_SEEDS: &[M3FixtureSeedV0] = &[
    M3FixtureSeedV0 {
        label: "sass-builtins-forward-import-configured-use",
        lane: M3FixtureLaneV0::SassLanguage,
        raw: r#"--- file: src/_tokens.scss
@use "sass:color";
$brand: color.scale(red, $lightness: 10%) !default;
--- file: src/_theme.scss
@forward "./tokens" as theme-* show $brand;
--- file: src/Button.module.scss
@use "./theme" as theme with ($theme-brand: blue);
@import "./legacy";
.button { color: theme.$theme-brand; }
--- expect: product
omena-query.style-diagnostics
--- expect: assertion
valid sass:color built-ins, @forward prefixing, @import hints, and configured @use identity do not become missingSassSymbol false positives
"#,
        expected_products: &[
            "omena-query.style-diagnostics",
            "omena-parser.sass-symbol-facts",
        ],
        promotion_target: "omena-testkit/sass-language",
    },
    M3FixtureSeedV0 {
        label: "cascade-transform-proof-obligations",
        lane: M3FixtureLaneV0::CascadeProof,
        raw: r#"--- file: src/proof.css
.a { margin-top: 1px; margin-right: 2px; margin-bottom: 1px; margin-left: 2px; }
@scope (:root) { .card { color: red; } }
@supports (display: grid) { .grid { display: grid; } }
--- expect: product
omena-transform-passes.cascade-proof-obligations
--- expect: assertion
shorthand, scope, and supports transforms expose accepted proof obligations through omena-query transform execution
"#,
        expected_products: &[
            "omena-transform-passes.cascade-proof-obligations",
            "omena-query.transform-execute",
        ],
        promotion_target: "omena-testkit/cascade-proof",
    },
    M3FixtureSeedV0 {
        label: "abstract-value-provenance-explanation",
        lane: M3FixtureLaneV0::Provenance,
        raw: r#"--- file: input/engine-input.json
{"version":"2","typeFacts":[{"expressionId":"expr-1","filePath":"src/App.tsx","facts":{"kind":"constrained","constraint":{"kind":"prefixSuffix","prefix":"button--","suffix":"active"}}}]}
--- expect: product
engine-input-producers.expression-domain-provenance-explanations
--- expect: assertion
derivation and provenance-tree payloads round-trip through omena-query and engine-shadow-runner
"#,
        expected_products: &[
            "engine-input-producers.expression-domain-provenance-explanations",
            "omena-abstract-value.provenance-tree",
        ],
        promotion_target: "omena-testkit/provenance",
    },
    M3FixtureSeedV0 {
        label: "zero-cfa-reduced-product-iteration",
        lane: M3FixtureLaneV0::AbstractValue,
        raw: r#"--- file: input/engine-input.json
{"version":"2","typeFacts":[{"expressionId":"call-a","filePath":"src/App.tsx","facts":{"kind":"literalUnion","values":["button","button--active"]}},{"expressionId":"call-b","filePath":"src/App.tsx","facts":{"kind":"literalUnion","values":["card","card--active"]}}]}
--- expect: product
engine-input-producers.expression-domain-reduced-product-iteration
--- expect: assertion
k=0 joins call-site exits while reduced product Pr x Su x CI converges with monotone iteration evidence
"#,
        expected_products: &[
            "engine-input-producers.expression-domain-call-site-flow-analysis",
            "engine-input-producers.expression-domain-reduced-product-iteration",
        ],
        promotion_target: "omena-testkit/abstract-value",
    },
];

/// Compare one fixture against the legacy parser oracle.
pub fn compare_omena_parser_with_legacy(
    fixture: ParserDifferentialFixture,
) -> ParserDifferentialReport {
    let legacy_sheet = parse_style_module(fixture.file_path, fixture.source);
    let legacy_summary = legacy_sheet
        .as_ref()
        .map(summarize_css_modules_intermediate);
    let omena_summary =
        summarize_omena_parser_style_facts(fixture.source, fixture.dialect.as_omena_dialect());

    let empty = Vec::new();
    let legacy_selectors = legacy_summary
        .as_ref()
        .map(|summary| &summary.selectors.names)
        .unwrap_or(&empty);
    let legacy_custom_properties = legacy_summary
        .as_ref()
        .map(|summary| {
            sorted_unique(
                summary
                    .custom_properties
                    .decl_names
                    .iter()
                    .chain(summary.custom_properties.ref_names.iter())
                    .cloned(),
            )
        })
        .unwrap_or_default();
    let legacy_sass_variables = legacy_summary
        .as_ref()
        .map(|summary| {
            sorted_unique(
                summary
                    .sass
                    .variable_decl_names
                    .iter()
                    .chain(summary.sass.variable_ref_names.iter())
                    .cloned(),
            )
        })
        .unwrap_or_default();
    let legacy_sass_module_edges = legacy_summary
        .as_ref()
        .map(|summary| {
            sorted_unique(
                summary
                    .sass
                    .module_use_sources
                    .iter()
                    .map(|_| "@use".to_string())
                    .chain(
                        summary
                            .sass
                            .module_forward_sources
                            .iter()
                            .map(|_| "@forward".to_string()),
                    ),
            )
        })
        .unwrap_or_default();

    let mut fields = vec![
        field_report(
            "classSelectorNames",
            legacy_selectors.iter().cloned(),
            omena_summary.class_selector_names,
        ),
        field_report(
            "customPropertyNames",
            legacy_custom_properties,
            omena_summary.custom_property_names,
        ),
    ];

    if fixture.dialect == DiffDialect::Scss {
        fields.push(field_report(
            "sassVariableNames",
            legacy_sass_variables,
            omena_summary
                .variable_names
                .into_iter()
                .map(|name| normalize_sass_variable_name(name.as_str())),
        ));
        fields.push(field_report(
            "sassModuleEdgeKinds",
            legacy_sass_module_edges,
            omena_summary
                .at_rule_names
                .into_iter()
                .filter(|name| name == "@use" || name == "@forward"),
        ));
    }

    let all_fields_match = fields.iter().all(|field| field.matches);
    ParserDifferentialReport {
        schema_version: "0",
        product: "omena-diff-test.parser-legacy-differential",
        label: fixture.label,
        dialect: fixture.dialect.as_label(),
        fields,
        all_fields_match,
    }
}

/// Summarize the differential harness boundary for parser cutover readiness gates.
pub fn summarize_omena_diff_test_boundary() -> OmenaDiffTestBoundarySummary {
    let reports: Vec<_> = PARSER_LEGACY_SEED_FIXTURES
        .iter()
        .copied()
        .map(compare_omena_parser_with_legacy)
        .collect();
    let all_parser_legacy_fixtures_match = reports.iter().all(|report| report.all_fields_match);
    let m3_fixture_seed_report = summarize_m3_fixture_seed_corpus();
    let wpt_seed_metadata_report = summarize_wpt_seed_corpus_metadata();

    OmenaDiffTestBoundarySummary {
        schema_version: "0",
        product: "omena-diff-test.boundary",
        owner: "omena-css/differential-corpus",
        parser_legacy_fixture_count: reports.len(),
        all_parser_legacy_fixtures_match,
        m3_fixture_seed_count: m3_fixture_seed_report.fixture_count,
        all_m3_fixture_seeds_parse: m3_fixture_seed_report.all_seeds_parse,
        wpt_seed_fixture_count: wpt_seed_metadata_report.fixture_count,
        all_wpt_seed_metadata_valid: wpt_seed_metadata_report.all_metadata_valid,
        closed_gates: vec![
            "parserVsLegacyOracle",
            "legacyParserQuarantinedAsOracle",
            "h1DifferentialHarnessOwnedByRustCrate",
            "m3FixtureSeedsConsumeOmenaTestkitParser",
            "wptSeedCorpusMetadataPolicy",
        ],
        reports,
        m3_fixture_seed_report,
        wpt_seed_metadata_report,
    }
}

/// Summarize the M3 reusable fixture seed corpus.
pub fn summarize_m3_fixture_seed_corpus() -> M3FixtureSeedCorpusReportV0 {
    let testkit_seeds = M3_THEORETICAL_MOAT_FIXTURE_SEEDS
        .iter()
        .copied()
        .map(testkit_seed_from_m3_seed)
        .collect::<Vec<_>>();
    let testkit_report = summarize_omena_testkit_fixture_seed_corpus(testkit_seeds.as_slice());
    let reports = M3_THEORETICAL_MOAT_FIXTURE_SEEDS
        .iter()
        .copied()
        .zip(testkit_report.reports)
        .map(|(seed, report)| M3FixtureSeedReportV0 {
            label: seed.label,
            lane: seed.lane,
            parses: report.parses,
            parse_error: report.parse_error,
            file_count: report.file_count,
            expectation_count: report.expectation_count,
            expected_products: report.expected_products,
            promotion_target: report.promotion_target,
        })
        .collect::<Vec<_>>();
    let lane_count = reports
        .iter()
        .map(|report| report.lane)
        .collect::<BTreeSet<_>>()
        .len();
    let all_seeds_parse = reports.iter().all(|report| report.parses);

    M3FixtureSeedCorpusReportV0 {
        schema_version: "0",
        product: "omena-diff-test.m3-fixture-seed-corpus",
        fixture_grammar: "cme-fixture-v0",
        fixture_count: reports.len(),
        lane_count,
        all_seeds_parse,
        reports,
    }
}

fn testkit_seed_from_m3_seed(seed: M3FixtureSeedV0) -> OmenaTestkitFixtureSeedV0 {
    OmenaTestkitFixtureSeedV0 {
        label: seed.label,
        lane: seed.lane.as_label(),
        raw: seed.raw,
        expected_products: seed.expected_products,
        promotion_target: seed.promotion_target,
    }
}

/// Summarize the WPT-style seed corpus metadata.
pub fn summarize_wpt_seed_corpus_metadata() -> WptSeedCorpusMetadataReportV0 {
    let manifest = serde_json::from_str::<serde_json::Value>(WPT_SEED_MANIFEST_SOURCE).ok();
    let chunk = serde_json::from_str::<serde_json::Value>(WPT_SEED_VALUES_CHUNK_SOURCE).ok();
    let stage = manifest
        .as_ref()
        .and_then(|value| value.get("stage"))
        .and_then(serde_json::Value::as_str)
        .unwrap_or("invalid")
        .to_string();
    let source_pin = manifest
        .as_ref()
        .and_then(|value| value.pointer("/source/pin"))
        .and_then(serde_json::Value::as_str)
        .unwrap_or("invalid")
        .to_string();
    let chunk_count = manifest
        .as_ref()
        .and_then(|value| value.get("chunks"))
        .and_then(serde_json::Value::as_array)
        .map_or(0, Vec::len);
    let fixture_count = chunk
        .as_ref()
        .and_then(|value| value.get("fixtures"))
        .and_then(serde_json::Value::as_array)
        .map_or(0, Vec::len);
    let known_failure_count = WPT_SEED_KNOWN_FAILURE_POLICY_SOURCE
        .lines()
        .filter(|line| line.trim() == "[[subtest]]")
        .count();
    let stage2_blocking = wpt_seed_policy_bool_value("stage2_blocking").unwrap_or(true);
    let required_min_fixture_count_for_stage2 =
        wpt_seed_policy_usize_value("required_min_fixture_count_for_stage2").unwrap_or(0);
    let required_consecutive_green_runs =
        wpt_seed_policy_usize_value("required_consecutive_green_runs").unwrap_or(0);
    let consecutive_green_runs = wpt_seed_policy_usize_value("consecutive_green_runs").unwrap_or(0);
    let known_failure_review_interval_days =
        wpt_seed_policy_usize_value("review_interval_days").unwrap_or(0);
    let all_metadata_valid = wpt_seed_manifest_metadata_valid(
        manifest.as_ref(),
        chunk.as_ref(),
        fixture_count,
        known_failure_count,
    );
    let stage2_promotion_blockers = wpt_seed_stage2_promotion_blockers(
        all_metadata_valid,
        stage.as_str(),
        fixture_count,
        known_failure_count,
        required_min_fixture_count_for_stage2,
        required_consecutive_green_runs,
        consecutive_green_runs,
    );
    let stage2_candidate_ready = stage2_promotion_blockers.is_empty();

    WptSeedCorpusMetadataReportV0 {
        schema_version: "0",
        product: "omena-diff-test.wpt-seed-corpus-metadata",
        stage,
        source_pin,
        chunk_count,
        fixture_count,
        known_failure_count,
        stage2_blocking,
        required_min_fixture_count_for_stage2,
        required_consecutive_green_runs,
        consecutive_green_runs,
        known_failure_review_interval_days,
        stage2_candidate_ready,
        stage2_promotion_blockers,
        all_metadata_valid,
        closed_gates: vec![
            "wptSeedSourcePin",
            "wptSeedChunkSchema",
            "wptSeedKnownFailurePolicy",
            "wptSeedStageOneAdvisory",
            "wptSeedStageTwoPromotionPolicy",
        ],
    }
}

fn wpt_seed_manifest_metadata_valid(
    manifest: Option<&serde_json::Value>,
    chunk: Option<&serde_json::Value>,
    fixture_count: usize,
    known_failure_count: usize,
) -> bool {
    let Some(manifest) = manifest else {
        return false;
    };
    let Some(chunk) = chunk else {
        return false;
    };
    let manifest_source_pin = manifest
        .pointer("/source/pin")
        .and_then(serde_json::Value::as_str);
    let chunk_source_pin = chunk.get("sourcePin").and_then(serde_json::Value::as_str);
    let manifest_fixture_count = manifest
        .pointer("/chunks/0/fixtureCount")
        .and_then(serde_json::Value::as_u64)
        .map(|value| value as usize);
    manifest
        .get("schemaVersion")
        .and_then(serde_json::Value::as_str)
        == Some("0")
        && manifest.get("product").and_then(serde_json::Value::as_str)
            == Some("omena-diff-test.wpt-seed-corpus.manifest")
        && manifest.get("stage").and_then(serde_json::Value::as_str) == Some("stage1-advisory")
        && manifest_source_pin.is_some_and(wpt_source_pin_is_full_sha)
        && manifest_source_pin == chunk_source_pin
        && manifest_fixture_count == Some(fixture_count)
        && chunk
            .get("schemaVersion")
            .and_then(serde_json::Value::as_str)
            == Some("0")
        && chunk.get("product").and_then(serde_json::Value::as_str)
            == Some("omena-diff-test.wpt-seed-corpus.chunk")
        && wpt_seed_policy_string_value("schema_version") == Some("0")
        && wpt_seed_policy_string_value("stage") == Some("advisory")
        && wpt_seed_policy_bool_value("stage2_blocking") == Some(false)
        && wpt_seed_policy_string_value("source_pin") == manifest_source_pin
        && wpt_seed_policy_usize_value("review_interval_days").is_some_and(|days| days > 0)
        && wpt_seed_policy_usize_value("required_min_fixture_count_for_stage2")
            .is_some_and(|count| count > 0)
        && wpt_seed_policy_usize_value("required_consecutive_green_runs")
            .is_some_and(|runs| runs > 0)
        && wpt_seed_policy_usize_value("consecutive_green_runs").is_some()
        && known_failure_count == 0
}

fn wpt_seed_stage2_promotion_blockers(
    all_metadata_valid: bool,
    stage: &str,
    fixture_count: usize,
    known_failure_count: usize,
    required_min_fixture_count_for_stage2: usize,
    required_consecutive_green_runs: usize,
    consecutive_green_runs: usize,
) -> Vec<&'static str> {
    let mut blockers = Vec::new();
    if !all_metadata_valid {
        blockers.push("metadataInvalid");
    }
    if stage != "stage1-advisory" {
        blockers.push("stageNotAdvisory");
    }
    if known_failure_count > 0 {
        blockers.push("knownFailuresPresent");
    }
    if required_min_fixture_count_for_stage2 == 0 {
        blockers.push("stageTwoFixtureThresholdMissing");
    } else if fixture_count < required_min_fixture_count_for_stage2 {
        blockers.push("seedCorpusBelowStageTwoMinimum");
    }
    if required_consecutive_green_runs == 0 {
        blockers.push("stageTwoGreenRunThresholdMissing");
    } else if consecutive_green_runs < required_consecutive_green_runs {
        blockers.push("insufficientConsecutiveGreenRuns");
    }
    blockers
}

fn wpt_seed_policy_string_value(key: &str) -> Option<&'static str> {
    let value = wpt_seed_policy_raw_value(key)?;
    value.strip_prefix('"')?.strip_suffix('"')
}

fn wpt_seed_policy_bool_value(key: &str) -> Option<bool> {
    match wpt_seed_policy_raw_value(key)? {
        "true" => Some(true),
        "false" => Some(false),
        _ => None,
    }
}

fn wpt_seed_policy_usize_value(key: &str) -> Option<usize> {
    wpt_seed_policy_raw_value(key)?.parse::<usize>().ok()
}

fn wpt_seed_policy_raw_value(key: &str) -> Option<&'static str> {
    for raw_line in WPT_SEED_KNOWN_FAILURE_POLICY_SOURCE.lines() {
        let line = raw_line.split('#').next().unwrap_or("").trim();
        if line.is_empty() || line.starts_with("[[") {
            continue;
        }
        let Some((candidate_key, value)) = line.split_once('=') else {
            continue;
        };
        if candidate_key.trim() == key {
            return Some(value.trim());
        }
    }
    None
}

fn wpt_source_pin_is_full_sha(pin: &str) -> bool {
    let Some(sha) = pin.strip_prefix("web-platform-tests/wpt@") else {
        return false;
    };
    sha.len() == 40 && sha.chars().all(|char| char.is_ascii_hexdigit())
}

impl M3FixtureLaneV0 {
    fn as_label(self) -> &'static str {
        match self {
            Self::SassLanguage => "sass-language",
            Self::CascadeProof => "cascade-proof",
            Self::Provenance => "provenance",
            Self::AbstractValue => "abstract-value",
        }
    }
}

fn field_report(
    field: &'static str,
    legacy_values: impl IntoIterator<Item = String>,
    omena_values: impl IntoIterator<Item = String>,
) -> DiffFieldReport {
    let legacy_values = sorted_unique(legacy_values);
    let omena_values = sorted_unique(omena_values);
    let matches = legacy_values == omena_values;
    DiffFieldReport {
        field,
        legacy_values,
        omena_values,
        matches,
    }
}

fn sorted_unique(values: impl IntoIterator<Item = String>) -> Vec<String> {
    values
        .into_iter()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn normalize_sass_variable_name(name: &str) -> String {
    name.trim_start_matches(['$', '@']).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parser_legacy_seed_fixtures_match() {
        let summary = summarize_omena_diff_test_boundary();
        assert_eq!(
            summary.parser_legacy_fixture_count,
            PARSER_LEGACY_SEED_FIXTURES.len()
        );
        assert!(summary.all_parser_legacy_fixtures_match);
        assert_eq!(
            summary.m3_fixture_seed_count,
            M3_THEORETICAL_MOAT_FIXTURE_SEEDS.len()
        );
        assert!(summary.all_m3_fixture_seeds_parse);
        assert!(summary.all_wpt_seed_metadata_valid);
        assert!(summary.wpt_seed_fixture_count >= 10);
        assert!(
            summary
                .closed_gates
                .contains(&"h1DifferentialHarnessOwnedByRustCrate")
        );
        assert!(
            summary
                .closed_gates
                .contains(&"m3FixtureSeedsConsumeOmenaTestkitParser")
        );
        assert!(
            summary
                .closed_gates
                .contains(&"wptSeedCorpusMetadataPolicy")
        );
    }

    #[test]
    fn reports_field_level_evidence_for_scss_fixture() -> Result<(), String> {
        let fixture = PARSER_LEGACY_SEED_FIXTURES
            .iter()
            .copied()
            .find(|fixture| fixture.label == "scss-nested-bem-and-sass-vars")
            .ok_or_else(|| "SCSS differential fixture should stay registered".to_string())?;
        let report = compare_omena_parser_with_legacy(fixture);
        assert!(report.all_fields_match);
        assert_eq!(
            report
                .fields
                .iter()
                .map(|field| field.field)
                .collect::<Vec<_>>(),
            vec![
                "classSelectorNames",
                "customPropertyNames",
                "sassVariableNames",
                "sassModuleEdgeKinds"
            ]
        );
        Ok(())
    }

    #[test]
    fn m3_fixture_seed_corpus_covers_all_theoretical_moat_lanes() {
        let report = summarize_m3_fixture_seed_corpus();

        assert_eq!(report.product, "omena-diff-test.m3-fixture-seed-corpus");
        assert_eq!(report.fixture_grammar, "cme-fixture-v0");
        assert_eq!(report.fixture_count, 4);
        assert_eq!(report.lane_count, 4);
        assert!(report.all_seeds_parse);
        assert!(report.reports.iter().all(|report| {
            report.file_count > 0
                && report.expectation_count >= 2
                && report.promotion_target.starts_with("omena-testkit/")
        }));
        assert!(report.reports.iter().any(|report| {
            report.lane == M3FixtureLaneV0::CascadeProof
                && report
                    .expected_products
                    .contains(&"omena-transform-passes.cascade-proof-obligations")
        }));
        assert!(report.reports.iter().any(|report| {
            report.lane == M3FixtureLaneV0::Provenance
                && report
                    .expected_products
                    .contains(&"omena-abstract-value.provenance-tree")
        }));
    }

    #[test]
    fn parses_reusable_cme_fixture_v0_sections() -> Result<(), String> {
        let seed = M3_THEORETICAL_MOAT_FIXTURE_SEEDS
            .iter()
            .find(|seed| seed.label == "cascade-transform-proof-obligations")
            .ok_or_else(|| "cascade fixture seed should stay registered".to_string())?;
        let fixture = parse_cme_fixture_v0(seed.raw)?;

        assert_eq!(fixture.schema_version, "0");
        assert_eq!(fixture.files.len(), 1);
        assert_eq!(fixture.files[0].path, "src/proof.css");
        assert!(fixture.files[0].source.contains("@scope (:root)"));
        assert_eq!(fixture.expectations.len(), 2);
        assert_eq!(fixture.expectations[0].key, "product");
        assert_eq!(
            fixture.expectations[0].value,
            "omena-transform-passes.cascade-proof-obligations"
        );

        Ok(())
    }

    #[test]
    fn wpt_seed_corpus_metadata_has_source_pin_and_policy() {
        let report = summarize_wpt_seed_corpus_metadata();

        assert_eq!(report.product, "omena-diff-test.wpt-seed-corpus-metadata");
        assert_eq!(report.stage, "stage1-advisory");
        assert!(wpt_source_pin_is_full_sha(report.source_pin.as_str()));
        assert_eq!(report.chunk_count, 1);
        assert!(report.fixture_count >= 10);
        assert_eq!(report.known_failure_count, 0);
        assert!(!report.stage2_blocking);
        assert_eq!(report.required_min_fixture_count_for_stage2, 25);
        assert_eq!(report.required_consecutive_green_runs, 5);
        assert_eq!(report.consecutive_green_runs, 0);
        assert_eq!(report.known_failure_review_interval_days, 30);
        assert!(!report.stage2_candidate_ready);
        assert!(
            report
                .stage2_promotion_blockers
                .contains(&"seedCorpusBelowStageTwoMinimum")
        );
        assert!(
            report
                .stage2_promotion_blockers
                .contains(&"insufficientConsecutiveGreenRuns")
        );
        assert!(report.all_metadata_valid);
        assert!(report.closed_gates.contains(&"wptSeedSourcePin"));
        assert!(report.closed_gates.contains(&"wptSeedKnownFailurePolicy"));
        assert!(
            report
                .closed_gates
                .contains(&"wptSeedStageTwoPromotionPolicy")
        );
    }
}
