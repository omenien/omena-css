//! Differential corpus harness for the omena-css parser stack.
//!
//! This crate is the Rust workspace home for parser differential checks that
//! were previously represented only by runner scripts. It treats
//! `engine-style-parser` as a legacy oracle and `omena-parser` as the product
//! parser surface.

use std::collections::BTreeSet;

use engine_style_parser::{parse_style_module, summarize_css_modules_intermediate};
use omena_incremental::{
    IncrementalGraphInputV0, IncrementalNodeInputV0, IncrementalRevisionV0,
    plan_incremental_computation, snapshot_from_graph_input,
};
use omena_parser::{StyleDialect, summarize_omena_parser_style_facts};
use omena_query::{
    OmenaQueryExternalModuleModeV0, OmenaQueryExternalSifInputV0,
    OmenaQueryStyleDiagnosticsForFileV0, OmenaQueryStyleSourceInputV0,
    summarize_omena_query_style_diagnostics_for_file,
    summarize_omena_query_style_diagnostics_for_workspace_file_with_external_mode_and_sifs,
    summarize_omena_query_style_hover_candidates,
};
use omena_sif::{
    OmenaSifExportsV1, OmenaSifGeneratorV1, OmenaSifSourceSyntaxV1, OmenaSifSourceV1, OmenaSifV1,
    OmenaSifVariableExportV1,
};
use omena_testkit::{
    OmenaFixtureDiagnosticV0, OmenaFixtureExpectationOutcomeV0, evaluate_omena_fixture_v0_with,
};
pub use omena_testkit::{
    OmenaFixtureExpectationV0, OmenaFixtureFileV0, OmenaFixtureV0, OmenaTestkitFixtureSeedV0,
    parse_omena_fixture_v0, summarize_omena_testkit_fixture_seed_corpus,
};
use serde::{Deserialize, Serialize};

mod cache_equivalence;
pub use cache_equivalence::{
    OmenaDiffCacheEquivalenceFileReportV0, OmenaDiffCacheEquivalenceReportV0,
    OmenaDiffSalsaMemoEquivalencePhaseV0, OmenaDiffSalsaMemoEquivalenceReportV0,
    evaluate_workspace_diagnostics_from_scratch_v0,
    evaluate_workspace_diagnostics_from_scratch_with_inputs_v0,
    omena_diff_cache_equivalence_default_corpus_v0, summarize_workspace_diagnostics_equivalence_v0,
    summarize_workspace_diagnostics_parallel_salsa_views_equivalence_v0,
    summarize_workspace_diagnostics_salsa_memo_equivalence_v0,
    summarize_workspace_diagnostics_warm_pass_equivalence_v0,
};

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

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IncrementalIdentityReuseEquivalenceReportV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub source_pair: &'static str,
    pub unchanged_syntax_id_stable: bool,
    pub changed_syntax_id_differs: bool,
    pub incremental_matches_from_scratch_delta: bool,
    pub fields: Vec<DiffFieldReport>,
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
    /// WPT value-differential fixtures routed through the value hand-models.
    pub wpt_value_differential_fixture_count: usize,
    /// Fixtures whose value folds to its WPT expected value.
    pub wpt_value_differential_match_count: usize,
    /// Fixtures the hand-models do not fold (declared triage, never a pass).
    pub wpt_value_differential_triage_count: usize,
    /// Soundiness metamorphic relation count.
    pub soundiness_metamorphic_relation_count: usize,
    /// Whether every soundiness metamorphic relation currently holds.
    pub all_soundiness_metamorphic_relations_hold: bool,
    /// Diagnostic metamorphic relation count.
    pub diagnostic_metamorphic_relation_count: usize,
    /// Whether every diagnostic metamorphic relation currently holds.
    pub all_diagnostic_metamorphic_relations_hold: bool,
    /// Cache-equivalence oracle corpus size (RFC 0009 §0).
    pub cache_equivalence_file_count: usize,
    /// Whether the cached-vs-from-scratch equivalence gate holds.
    pub all_cache_equivalence_files_identical: bool,
    /// Salsa-memo lifecycle comparisons (RFC 0009 Pillar B merge gate).
    pub salsa_memo_equivalence_comparison_count: usize,
    /// Whether the salsa-memoized evaluator matched from-scratch in every phase.
    pub all_salsa_memo_equivalence_phases_identical: bool,
    /// Parallel fixed-revision view comparisons (RFC 0009 Pillar F merge gate).
    pub parallel_salsa_equivalence_comparison_count: usize,
    /// Whether every parallel-view comparison matched from-scratch in every phase.
    pub all_parallel_salsa_equivalence_phases_identical: bool,
    /// WPT-style seed metadata report.
    pub wpt_seed_metadata_report: WptSeedCorpusMetadataReportV0,
    /// WPT value-differential report (specified-value hand-model agreement).
    pub wpt_value_differential_report: WptValueDifferentialReportV0,
    /// Soundiness metamorphic relation report.
    pub soundiness_metamorphic_report: SoundinessMetamorphicReportV0,
    /// Internal omena-vs-omena diagnostic metamorphic relation report.
    pub diagnostic_metamorphic_report: DiagnosticMetamorphicReportV0,
    /// Cached-vs-from-scratch diagnostic equivalence report (RFC 0009 §0).
    pub cache_equivalence_report: OmenaDiffCacheEquivalenceReportV0,
    /// Salsa-memo lifecycle equivalence report (RFC 0009 Pillar B).
    pub salsa_memo_equivalence_report: OmenaDiffSalsaMemoEquivalenceReportV0,
    /// Parallel fixed-revision view equivalence report (RFC 0009 Pillar F).
    pub parallel_salsa_equivalence_report: OmenaDiffSalsaMemoEquivalenceReportV0,
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
    /// Raw `omena-fixture-v0` text.
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
    /// Whether the fixture parses with `omena-fixture-v0`.
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
    /// Fixture count in Stage 2 blocking chunks.
    pub blocking_fixture_count: usize,
    /// Fixture count in Stage 1 advisory chunks.
    pub advisory_fixture_count: usize,
    /// Fixture coverage by pinned sparse WPT path.
    pub sparse_path_fixture_counts: Vec<WptSeedSparsePathFixtureCountV0>,
    /// Whether each pinned sparse path is represented by at least one fixture.
    pub all_sparse_paths_have_fixtures: bool,
    /// Whether generated manifest sparse-path counts match checked fixtures.
    pub manifest_sparse_path_fixture_counts_valid: bool,
    /// Known-failure entry count.
    pub known_failure_count: usize,
    /// Known-failure entries whose fixture or subtest no longer exists.
    pub stale_known_failure_count: usize,
    /// Whether the current policy is already blocking Stage 2.
    pub stage2_blocking: bool,
    /// Minimum fixture count required before Stage 2 can become blocking.
    pub required_min_fixture_count_for_stage2: usize,
    /// Required consecutive green advisory runs before Stage 2 promotion.
    pub required_consecutive_green_runs: usize,
    /// Current consecutive green advisory run count for this pinned corpus.
    pub consecutive_green_runs: usize,
    /// Reviewed green-run evidence entry count.
    pub green_run_evidence_count: usize,
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

/// Fixture coverage count for one pinned sparse WPT path.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WptSeedSparsePathFixtureCountV0 {
    /// Sparse WPT path from the generated corpus manifest.
    pub sparse_path: String,
    /// Fixture count whose WPT path is below this sparse path.
    pub fixture_count: usize,
}

/// Soundiness metamorphic report for external-boundary diagnostics.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SoundinessMetamorphicReportV0 {
    /// Schema version.
    pub schema_version: &'static str,
    /// Product surface name.
    pub product: &'static str,
    /// Relation count.
    pub relation_count: usize,
    /// Whether every relation currently holds.
    pub all_relations_hold: bool,
    /// Relation reports.
    pub relations: Vec<SoundinessMetamorphicRelationReportV0>,
    /// Named gates closed by this report.
    pub closed_gates: Vec<&'static str>,
}

/// One soundiness metamorphic relation result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SoundinessMetamorphicRelationReportV0 {
    /// Stable relation label.
    pub relation: &'static str,
    /// Diagnostic codes before applying the relation transform.
    pub before_diagnostic_codes: Vec<String>,
    /// Diagnostic codes after applying the relation transform.
    pub after_diagnostic_codes: Vec<String>,
    /// Diagnostic count before applying the relation transform.
    pub before_diagnostic_count: usize,
    /// Diagnostic count after applying the relation transform.
    pub after_diagnostic_count: usize,
    /// Whether this relation currently holds.
    pub holds: bool,
    /// Product surfaces exercised by the relation.
    pub evidence_surfaces: Vec<&'static str>,
}

/// Internal omena-vs-omena metamorphic report for diagnostic code-set stability.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiagnosticMetamorphicReportV0 {
    /// Schema version.
    pub schema_version: &'static str,
    /// Product surface name.
    pub product: &'static str,
    /// Relation count.
    pub relation_count: usize,
    /// Whether every relation currently holds.
    pub all_relations_hold: bool,
    /// Relation reports.
    pub relations: Vec<DiagnosticMetamorphicRelationReportV0>,
    /// Named gates closed by this report.
    pub closed_gates: Vec<&'static str>,
}

/// One diagnostic metamorphic relation result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiagnosticMetamorphicRelationReportV0 {
    /// Stable relation label.
    pub relation: &'static str,
    /// Diagnostic code set before applying the relation transform.
    pub before_diagnostic_codes: Vec<String>,
    /// Diagnostic code set after applying the relation transform.
    pub after_diagnostic_codes: Vec<String>,
    /// Whether this relation preserves the diagnostic code set.
    pub holds: bool,
    /// Product surfaces exercised by the relation.
    pub evidence_surfaces: Vec<&'static str>,
}

const WPT_SEED_MANIFEST_SOURCE: &str = include_str!("../wpt-corpus/manifest.json");
const WPT_SEED_CHUNK_SOURCES: &[&str] = &[
    include_str!("../wpt-corpus/css-values.json"),
    include_str!("../wpt-corpus/css-values-advisory.json"),
];
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

pub fn summarize_incremental_identity_reuse_equivalence_v0()
-> IncrementalIdentityReuseEquivalenceReportV0 {
    let previous_source = ".alpha { color: red; } .beta { color: blue; }";
    let next_source = ".alpha { color: green; } .beta { color: blue; }";
    let previous_alpha_id = parser_rule_syntax_node_id(previous_source, ".alpha");
    let previous_beta_id = parser_rule_syntax_node_id(previous_source, ".beta");
    let next_alpha_id = parser_rule_syntax_node_id(next_source, ".alpha");
    let next_beta_id = parser_rule_syntax_node_id(next_source, ".beta");
    let previous = IncrementalGraphInputV0 {
        revision: IncrementalRevisionV0 { value: 1 },
        nodes: vec![
            IncrementalNodeInputV0 {
                id: previous_alpha_id.clone(),
                digest: "alpha:red".to_string(),
                dependency_ids: Vec::new(),
            },
            IncrementalNodeInputV0 {
                id: previous_beta_id.clone(),
                digest: "beta:blue".to_string(),
                dependency_ids: Vec::new(),
            },
        ],
    };
    let next = IncrementalGraphInputV0 {
        revision: IncrementalRevisionV0 { value: 2 },
        nodes: vec![
            IncrementalNodeInputV0 {
                id: next_alpha_id.clone(),
                digest: "alpha:green".to_string(),
                dependency_ids: Vec::new(),
            },
            IncrementalNodeInputV0 {
                id: next_beta_id.clone(),
                digest: "beta:blue".to_string(),
                dependency_ids: Vec::new(),
            },
        ],
    };
    let previous_snapshot = snapshot_from_graph_input(&previous);
    let identity_keyed_reuse = plan_incremental_computation(&next, Some(&previous_snapshot));
    let full_rebuild_snapshot = snapshot_from_graph_input(&next);
    let fields = vec![
        field_report(
            "nodeIdentityDigest",
            full_rebuild_snapshot
                .nodes
                .iter()
                .map(|node| format!("{}|{}", node.id, node.digest)),
            identity_keyed_reuse
                .nodes
                .iter()
                .map(|node| format!("{}|{}", node.id, node.digest)),
        ),
        field_report(
            "dependencyEdges",
            full_rebuild_snapshot.nodes.iter().flat_map(|node| {
                node.dependency_ids
                    .iter()
                    .map(|dependency_id| format!("{}->{dependency_id}", node.id))
            }),
            identity_keyed_reuse.nodes.iter().flat_map(|node| {
                node.dependency_ids
                    .iter()
                    .map(|dependency_id| format!("{}->{dependency_id}", node.id))
            }),
        ),
        field_report(
            "dirtyIds",
            identity_keyed_reuse
                .shadow_delta_oracle
                .from_scratch_dirty_ids
                .clone(),
            identity_keyed_reuse
                .shadow_delta_oracle
                .incremental_dirty_ids
                .clone(),
        ),
        field_report(
            "reusableCleanIds",
            vec![next_beta_id.clone()],
            identity_keyed_reuse
                .nodes
                .iter()
                .filter(|node| !node.dirty)
                .map(|node| node.id.clone()),
        ),
    ];
    let all_fields_match = fields.iter().all(|field| field.matches);

    IncrementalIdentityReuseEquivalenceReportV0 {
        schema_version: "0",
        product: "omena-diff-test.incremental-identity-reuse-equivalence",
        source_pair: "css-two-rule-alpha-edit-beta-unchanged",
        unchanged_syntax_id_stable: previous_beta_id == next_beta_id,
        changed_syntax_id_differs: previous_alpha_id != next_alpha_id,
        incremental_matches_from_scratch_delta: identity_keyed_reuse
            .shadow_delta_oracle
            .incremental_matches_from_scratch_delta,
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
    let wpt_value_differential_report = summarize_wpt_value_differential();
    let soundiness_metamorphic_report = summarize_soundiness_metamorphic_relations();
    let diagnostic_metamorphic_report = summarize_diagnostic_metamorphic_relations();
    let (cache_equivalence_corpus, cache_equivalence_resolution_inputs) =
        omena_diff_cache_equivalence_default_corpus_v0();
    let cache_equivalence_report = summarize_workspace_diagnostics_warm_pass_equivalence_v0(
        cache_equivalence_corpus.as_slice(),
        &cache_equivalence_resolution_inputs,
    );
    let salsa_memo_equivalence_report = summarize_workspace_diagnostics_salsa_memo_equivalence_v0(
        cache_equivalence_corpus.as_slice(),
        &cache_equivalence_resolution_inputs,
    );
    let parallel_salsa_equivalence_report =
        summarize_workspace_diagnostics_parallel_salsa_views_equivalence_v0(
            cache_equivalence_corpus.as_slice(),
            &cache_equivalence_resolution_inputs,
        );

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
        wpt_value_differential_fixture_count: wpt_value_differential_report.fixture_count,
        wpt_value_differential_match_count: wpt_value_differential_report.value_match_count,
        wpt_value_differential_triage_count: wpt_value_differential_report.triage_fixture_ids.len(),
        soundiness_metamorphic_relation_count: soundiness_metamorphic_report.relation_count,
        all_soundiness_metamorphic_relations_hold: soundiness_metamorphic_report.all_relations_hold,
        diagnostic_metamorphic_relation_count: diagnostic_metamorphic_report.relation_count,
        all_diagnostic_metamorphic_relations_hold: diagnostic_metamorphic_report.all_relations_hold,
        cache_equivalence_file_count: cache_equivalence_report.file_count,
        all_cache_equivalence_files_identical: cache_equivalence_report.all_files_identical,
        salsa_memo_equivalence_comparison_count: salsa_memo_equivalence_report.comparison_count,
        all_salsa_memo_equivalence_phases_identical: salsa_memo_equivalence_report
            .all_phases_identical,
        parallel_salsa_equivalence_comparison_count: parallel_salsa_equivalence_report
            .comparison_count,
        all_parallel_salsa_equivalence_phases_identical: parallel_salsa_equivalence_report
            .all_phases_identical,
        closed_gates: vec![
            "parserVsLegacyOracle",
            "legacyParserQuarantinedAsOracle",
            "h1DifferentialHarnessOwnedByRustCrate",
            "m3FixtureSeedsConsumeOmenaTestkitParser",
            "wptSeedCorpusMetadataPolicy",
            "soundinessMetamorphicRelations",
            "diagnosticMetamorphicRelations",
            "cachedVsFromScratchEquivalenceOracle",
            "salsaMemoizedVsFromScratchEquivalence",
            "externalSifsSalsaMemoizedVsFromScratchEquivalence",
            "parallelSalsaViewsVsFromScratchEquivalence",
            "wptValueDifferentialHandModelAgreement",
        ],
        reports,
        m3_fixture_seed_report,
        wpt_seed_metadata_report,
        wpt_value_differential_report,
        soundiness_metamorphic_report,
        diagnostic_metamorphic_report,
        cache_equivalence_report,
        salsa_memo_equivalence_report,
        parallel_salsa_equivalence_report,
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
        fixture_grammar: "omena-fixture-v0",
        fixture_count: reports.len(),
        lane_count,
        all_seeds_parse,
        reports,
    }
}

/// Summarize soundiness metamorphic relations over the real workspace diagnostics path.
pub fn summarize_soundiness_metamorphic_relations() -> SoundinessMetamorphicReportV0 {
    let relations = vec![
        soundiness_resolved_sif_relation(),
        soundiness_partial_to_complete_relation(),
        soundiness_blob_soundness_relation(),
    ];
    let all_relations_hold = relations.iter().all(|relation| relation.holds);

    SoundinessMetamorphicReportV0 {
        schema_version: "0",
        product: "omena-diff-test.soundiness-metamorphic-relations",
        relation_count: relations.len(),
        all_relations_hold,
        relations,
        closed_gates: vec![
            "soundinessMetamorphicRelationHarness",
            "soundinessResolvedSifRelation",
            "soundinessMonotonicityRelation",
            "soundinessBlobSoundnessRelation",
        ],
    }
}

/// Summarize internal diagnostic metamorphic relations over the product diagnostics path.
pub fn summarize_diagnostic_metamorphic_relations() -> DiagnosticMetamorphicReportV0 {
    let relations = vec![
        diagnostic_relation_report(
            "mr-selector-list-distribution",
            "/tmp/SelectorList.module.css",
            ":root { --known: red; }\n.a, .b { color: var(--missing); animation: fade 1s; }",
            ":root { --known: red; }\n.a { color: var(--missing); animation: fade 1s; }\n.b { color: var(--missing); animation: fade 1s; }",
        ),
        diagnostic_relation_report(
            "mr-declaration-permutation",
            "/tmp/DeclarationPermutation.module.css",
            ":root { --known: red; }\n.button { color: var(--missing); animation: fade 1s; }",
            ":root { --known: red; }\n.button { animation: fade 1s; color: var(--missing); }",
        ),
        diagnostic_relation_report(
            "mr-selector-rename-invariance",
            "/tmp/Rename.module.css",
            ":root { --known: red; }\n.button { color: var(--missing); animation: fade 1s; }",
            ":root { --known: red; }\n.card { color: var(--missing); animation: fade 1s; }",
        ),
        diagnostic_relation_report(
            "mr-whitespace-comment-invariance",
            "/tmp/Whitespace.module.css",
            ":root { --known: red; }\n.button { color: var(--missing); animation: fade 1s; }",
            "/* comment */\n:root { --known: red; }\n.button {\n  color: var(--missing);\n  animation: fade 1s;\n}",
        ),
        diagnostic_relation_report(
            "mr-nesting-depth-equivalence",
            "/tmp/Nesting.module.scss",
            ":root { --known: red; }\n.button { &__icon { color: var(--missing); animation: fade 1s; } }",
            ":root { --known: red; }\n.button__icon { color: var(--missing); animation: fade 1s; }",
        ),
        diagnostic_relation_report(
            "mr-media-query-equivalence",
            "/tmp/Media.module.css",
            ":root { --known: red; }\n@media screen { .button { color: var(--missing); animation: fade 1s; } }",
            ":root { --known: red; }\n.button { color: var(--missing); animation: fade 1s; }\n@media screen {}",
        ),
        diagnostic_relation_report(
            "mr-canonicalizer-idempotence",
            "/tmp/Canonical.module.css",
            ":root { --known: red; }\n.button { color: var(--missing); animation: fade 1s; }",
            ":root{--known:red}.button{color:var(--missing);animation:fade 1s}",
        ),
    ];
    let all_relations_hold = relations.iter().all(|relation| relation.holds);

    DiagnosticMetamorphicReportV0 {
        schema_version: "0",
        product: "omena-diff-test.diagnostic-metamorphic-relations",
        relation_count: relations.len(),
        all_relations_hold,
        relations,
        closed_gates: vec![
            "diagnosticMetamorphicRelationHarness",
            "diagnosticMetamorphicSelectorListDistribution",
            "diagnosticMetamorphicDeclarationPermutation",
            "diagnosticMetamorphicSelectorRename",
            "diagnosticMetamorphicWhitespaceComment",
            "diagnosticMetamorphicNestingDepth",
            "diagnosticMetamorphicMediaQuery",
            "diagnosticMetamorphicCanonicalizerIdempotence",
        ],
    }
}

fn diagnostic_relation_report(
    relation: &'static str,
    style_uri: &str,
    before_source: &str,
    after_source: &str,
) -> DiagnosticMetamorphicRelationReportV0 {
    let before_diagnostic_codes = diagnostic_code_set_for_source(style_uri, before_source);
    let after_diagnostic_codes = diagnostic_code_set_for_source(style_uri, after_source);
    let holds = before_diagnostic_codes == after_diagnostic_codes;
    DiagnosticMetamorphicRelationReportV0 {
        relation,
        before_diagnostic_codes,
        after_diagnostic_codes,
        holds,
        evidence_surfaces: vec![
            "omena-query.style-hover-candidates",
            "omena-query.style-diagnostics",
            "omena-diff-test.diagnostic-metamorphic-relations",
        ],
    }
}

fn diagnostic_code_set_for_source(style_uri: &str, source: &str) -> Vec<String> {
    let Some(candidates) = summarize_omena_query_style_hover_candidates(style_uri, source) else {
        return vec!["summaryUnavailable".to_string()];
    };
    summarize_omena_query_style_diagnostics_for_file(
        style_uri,
        source,
        candidates.candidates.as_slice(),
    )
    .diagnostics
    .iter()
    .map(|diagnostic| diagnostic.code.to_string())
    .collect::<BTreeSet<_>>()
    .into_iter()
    .collect()
}

fn soundiness_resolved_sif_relation() -> SoundinessMetamorphicRelationReportV0 {
    let source = r#"@use "https://cdn.example/tokens.scss" as remote;
.button { color: remote.$brand; }"#;
    let before = soundiness_workspace_diagnostics(source, &[]);
    let after_sif = soundiness_sif_input("https://cdn.example/tokens.scss", &["$brand"]);
    let after = after_sif
        .as_ref()
        .ok()
        .and_then(|sif| soundiness_workspace_diagnostics(source, std::slice::from_ref(sif)));
    soundiness_relation_report(
        "mr-soundiness-resolved-sif",
        before,
        after,
        |before, after| {
            diagnostic_codes_contain(before, "missingExternalSif")
                && !diagnostic_codes_contain(after, "missingExternalSif")
                && !diagnostic_codes_contain(after, "missingSassSymbol")
                && after.diagnostic_count <= before.diagnostic_count
        },
    )
}

fn soundiness_partial_to_complete_relation() -> SoundinessMetamorphicRelationReportV0 {
    let source = r#"@use "https://cdn.example/tokens.scss" as remote;
.button { color: remote.$brand; border-color: remote.$accent; }"#;
    let partial_sif = soundiness_sif_input("https://cdn.example/tokens.scss", &["$brand"]);
    let complete_sif =
        soundiness_sif_input("https://cdn.example/tokens.scss", &["$brand", "$accent"]);
    let before = partial_sif
        .as_ref()
        .ok()
        .and_then(|sif| soundiness_workspace_diagnostics(source, std::slice::from_ref(sif)));
    let after = complete_sif
        .as_ref()
        .ok()
        .and_then(|sif| soundiness_workspace_diagnostics(source, std::slice::from_ref(sif)));
    soundiness_relation_report(
        "mr-monotonicity-partial-to-complete-sif",
        before,
        after,
        |before, after| {
            diagnostic_codes_contain(before, "partialExternalSif")
                && !diagnostic_codes_contain(after, "partialExternalSif")
                && !diagnostic_codes_contain(after, "missingSassSymbol")
                && after.diagnostic_count < before.diagnostic_count
        },
    )
}

fn soundiness_blob_soundness_relation() -> SoundinessMetamorphicRelationReportV0 {
    let before_source = r#"@use "https://cdn.example/tokens.scss" as remote;
.button { color: remote.$brand; }"#;
    let after_source = r#"// @omena-strict: closed
@use "https://cdn.example/tokens.scss" as remote;
.button { color: remote.$brand; }"#;
    let before = soundiness_workspace_diagnostics(before_source, &[]);
    let after = soundiness_workspace_diagnostics(after_source, &[]);
    soundiness_relation_report(
        "mr-blob-soundness-closed-world-exposes-top-any",
        before,
        after,
        |before, after| {
            !diagnostic_codes_contain(before, "missingSassSymbol")
                && diagnostic_codes_contain(before, "missingExternalSif")
                && diagnostic_codes_contain(after, "missingSassSymbol")
                && diagnostic_codes_contain(after, "missingExternalSif")
        },
    )
}

fn soundiness_workspace_diagnostics(
    source: &str,
    external_sifs: &[OmenaQueryExternalSifInputV0],
) -> Option<OmenaQueryStyleDiagnosticsForFileV0> {
    let style_sources = vec![OmenaQueryStyleSourceInputV0 {
        style_path: "/tmp/Soundiness.module.scss".to_string(),
        style_source: source.to_string(),
    }];
    summarize_omena_query_style_diagnostics_for_workspace_file_with_external_mode_and_sifs(
        "/tmp/Soundiness.module.scss",
        style_sources.as_slice(),
        &[],
        &[],
        None,
        OmenaQueryExternalModuleModeV0::Sif,
        external_sifs,
    )
}

fn soundiness_sif_input(
    canonical_url: &str,
    variable_names: &[&str],
) -> Result<OmenaQueryExternalSifInputV0, serde_json::Error> {
    let exports = OmenaSifExportsV1 {
        variables: variable_names
            .iter()
            .map(|name| OmenaSifVariableExportV1 {
                name: (*name).to_string(),
                defaulted: true,
                value_repr: Some("red".to_string()),
            })
            .collect(),
        mixins: Vec::new(),
        functions: Vec::new(),
        placeholders: Vec::new(),
        forwards: Vec::new(),
    };
    let source_bytes = variable_names
        .iter()
        .map(|name| format!("{name}: red !default;"))
        .collect::<Vec<_>>()
        .join("\n");
    let sif = OmenaSifV1::from_static_exports(
        canonical_url,
        OmenaSifGeneratorV1 {
            name: "omena-diff-test-sifgen".to_string(),
            version: "0.0.0".to_string(),
            toolchain_id: "omena-diff-test-sifgen@0.0.0".to_string(),
        },
        OmenaSifSourceV1 {
            syntax: OmenaSifSourceSyntaxV1::Scss,
        },
        exports,
        Vec::new(),
        source_bytes.as_bytes(),
    )?;
    Ok(OmenaQueryExternalSifInputV0 {
        canonical_url: canonical_url.to_string(),
        sif,
    })
}

fn soundiness_relation_report(
    relation: &'static str,
    before: Option<OmenaQueryStyleDiagnosticsForFileV0>,
    after: Option<OmenaQueryStyleDiagnosticsForFileV0>,
    predicate: impl FnOnce(
        &OmenaQueryStyleDiagnosticsForFileV0,
        &OmenaQueryStyleDiagnosticsForFileV0,
    ) -> bool,
) -> SoundinessMetamorphicRelationReportV0 {
    let holds = before
        .as_ref()
        .zip(after.as_ref())
        .is_some_and(|(before, after)| predicate(before, after));
    SoundinessMetamorphicRelationReportV0 {
        relation,
        before_diagnostic_codes: before
            .as_ref()
            .map(diagnostic_codes)
            .unwrap_or_else(|| vec!["summaryUnavailable".to_string()]),
        after_diagnostic_codes: after
            .as_ref()
            .map(diagnostic_codes)
            .unwrap_or_else(|| vec!["summaryUnavailable".to_string()]),
        before_diagnostic_count: before
            .as_ref()
            .map(|summary| summary.diagnostic_count)
            .unwrap_or(0),
        after_diagnostic_count: after
            .as_ref()
            .map(|summary| summary.diagnostic_count)
            .unwrap_or(0),
        holds,
        evidence_surfaces: vec![
            "omena-query.workspace-style-diagnostics",
            "omena-query.external-sif-boundary",
            "omena-diff-test.boundary",
        ],
    }
}

fn diagnostic_codes(summary: &OmenaQueryStyleDiagnosticsForFileV0) -> Vec<String> {
    summary
        .diagnostics
        .iter()
        .map(|diagnostic| diagnostic.code.to_string())
        .collect()
}

fn diagnostic_codes_contain(summary: &OmenaQueryStyleDiagnosticsForFileV0, code: &str) -> bool {
    summary
        .diagnostics
        .iter()
        .any(|diagnostic| diagnostic.code == code)
}

/// Evaluate a parsed `omena-fixture-v0` against the diagnostics the *real* engine
/// produces for the fixture's style files.
///
/// This is the missing engine-backed caller of
/// [`omena_testkit::evaluate_omena_fixture_v0_with`] (#37): the testkit defines the
/// evaluator but stays free of an `omena-query` dependency to preserve the
/// workspace DAG, so it can only check fixtures against diagnostics an
/// engine-aware consumer feeds in. `omena-diff-test` already sits above the
/// engine, so it supplies the real chain here:
/// [`summarize_omena_query_style_hover_candidates`] then
/// [`summarize_omena_query_style_diagnostics_for_file`] (the exact path the LSP
/// and CLI surfaces use), projecting each produced
/// `OmenaQueryStyleDiagnosticV0.code` into the testkit's
/// [`OmenaFixtureDiagnosticV0`].
///
/// The closure is invoked once per fixture file. Non-style files (e.g. JSON
/// `engine-input.json` seeds) where the summarizer returns `None` contribute no
/// diagnostics, so the caller is safe across mixed-file fixtures. Boundary-state
/// and cascade families are out of scope for this diagnostics-only caller and
/// are passed `&[]`; a fixture that only asserts diagnostic-family expectations
/// therefore evaluates entirely against real engine output.
pub fn evaluate_omena_fixture_against_real_diagnostics_v0(
    fixture: &OmenaFixtureV0,
) -> Vec<OmenaFixtureExpectationOutcomeV0> {
    evaluate_omena_fixture_v0_with(fixture, &[], &[], |file: &OmenaFixtureFileV0| {
        let Some(candidates) =
            summarize_omena_query_style_hover_candidates(&file.path, &file.source)
        else {
            // Non-style file (e.g. JSON engine input): no style diagnostics.
            return Vec::new();
        };
        let summary = summarize_omena_query_style_diagnostics_for_file(
            &file.path,
            &file.source,
            candidates.candidates.as_slice(),
        );
        summary
            .diagnostics
            .iter()
            .map(|diagnostic| OmenaFixtureDiagnosticV0::new(diagnostic.code))
            .collect()
    })
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
/// WPT value-differential report: the WPT specified-value pairs
/// (`wptValue` → `wptExpectedValue`) routed through the `omena-value-lattice`
/// hand-models. STRING-domain agreement only — never a typed-eval claim, and a
/// fixture the hand-models cannot fold is a declared triage record, never a pass.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WptValueDifferentialReportV0 {
    /// Schema version.
    pub schema_version: &'static str,
    /// Product surface name.
    pub product: &'static str,
    /// Stage2-blocking fixtures compared.
    pub fixture_count: usize,
    /// Fixtures whose value folds to its WPT expected value.
    pub value_match_count: usize,
    /// Fixture ids the hand-models do not fold (declared, never an implicit pass).
    pub triage_fixture_ids: Vec<String>,
    /// Whether every non-agreeing fixture is on the declared triage allowlist
    /// (an undeclared non-fold — a regression — makes this false).
    pub all_foldable_matches_hold: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WptValueDifferentialChunkV0 {
    fixtures: Vec<WptValueDifferentialFixtureV0>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WptValueDifferentialFixtureV0 {
    id: String,
    wpt_value: String,
    wpt_expected_value: String,
}

// Fixtures whose WPT computed/used value the string-domain hand-models do not
// fold (e.g. `opacity: 50%` resolving to `0.5` needs property-aware computed-value
// semantics the value lattice does not model). Declared so coverage stays honest;
// each is reported, never silently treated as a pass.
const WPT_VALUE_DIFFERENTIAL_TRIAGE: &[&str] = &["css-opacity-percent-half"];

fn flatten_calc_keywords(value: &str) -> String {
    // `calc()` is pure grouping; replace each `calc(` (case-insensitive) with a
    // bare paren so the numeric reducer folds arbitrarily nested calc expressions.
    // NOTE: this is a lexical substring replace, not a tokenizer — it assumes a
    // numeric-value context, so a `calc(` appearing inside a string or url() literal
    // would be rewritten. Sound for the present corpus (such values are raw-equal
    // and short-circuit before this runs); revisit if the corpus grows to carry
    // calc-bearing string/url values.
    let mut result = value.to_string();
    while let Some(pos) = result.to_ascii_lowercase().find("calc(") {
        result.replace_range(pos..pos + "calc(".len(), "(");
    }
    result
}

fn reduce_static_math(value: &str) -> Option<String> {
    let flattened = flatten_calc_keywords(value.trim());
    let lower = flattened.to_ascii_lowercase();
    if lower.starts_with("clamp(") {
        omena_value_lattice::parse_reducible_clamp_value(&flattened)
    } else if lower.starts_with("min(") {
        omena_value_lattice::parse_reducible_min_value(&flattened)
    } else if lower.starts_with("max(") {
        omena_value_lattice::parse_reducible_max_value(&flattened)
    } else {
        omena_value_lattice::reduce_static_numeric_expression(&flattened)
    }
}

/// Whether `value` folds to `expected` under the string-domain hand-models. Sound
/// by construction: only raw equality, canonical color/number equality, or a
/// numeric reduction that agrees counts as a match — disagreement is never a pass.
fn wpt_values_agree(value: &str, expected: &str) -> bool {
    if value.trim() == expected.trim() {
        return true;
    }
    if omena_value_lattice::css_values_canonically_equal(value, expected) {
        return true;
    }
    // NOTE: compare the reduced forms by EXACT text only. css_values_canonically_equal
    // collapses every absolute-zero unit to "0" (`0px` == `0deg`), so using it here
    // would unsoundly match a cross-unit zero (e.g. `calc(0px)` vs `0deg`). The
    // reducers already emit a canonical shortest form, so exact equality of the two
    // reductions is the sound test.
    match (reduce_static_math(value), reduce_static_math(expected)) {
        (Some(folded_value), Some(folded_expected)) => {
            folded_value.trim() == folded_expected.trim()
        }
        _ => false,
    }
}

fn wpt_value_differential_fixtures() -> Vec<WptValueDifferentialFixtureV0> {
    // The first chunk source is the stage2-blocking seed corpus (css-values.json);
    // the advisory chunk is intentionally out of the blocking value gate.
    WPT_SEED_CHUNK_SOURCES
        .first()
        .and_then(|source| serde_json::from_str::<WptValueDifferentialChunkV0>(source).ok())
        .map(|chunk| chunk.fixtures)
        .unwrap_or_default()
}

// DEFERRED (recorded so the next session does not assume these landed):
//  - Net-new computed-value WPT fixtures (selections.json regeneration) are NOT
//    added — the existing pairs are already specified-value level, and authoring
//    new ones would require sourcing real WPT data (fabricating violates the
//    no-guess invariant). The helper allowlist is widened to admit them when sourced.
//  - The comparator is property-AGNOSTIC (raw / canonical / numeric folds) rather
//    than property-dispatched; the two cascade hand-models are not used because
//    run_wpt_cascade_seed_corpus() / compute_cascade_computed_value take/return
//    structs, not a value -> Option<String> fold (the goal doc's "all four return
//    Option<String>" is false for those two).
//  - The value gate lands as the boundary-bin exit condition, running in PARALLEL
//    with the structural wpt-seed-policy.toml green-run gate rather than superseding it.
/// Route the stage2-blocking WPT value pairs through the hand-models.
pub fn summarize_wpt_value_differential() -> WptValueDifferentialReportV0 {
    let fixtures = wpt_value_differential_fixtures();
    let mut value_match_count = 0usize;
    let mut triage_fixture_ids = Vec::new();
    for fixture in &fixtures {
        if wpt_values_agree(
            fixture.wpt_value.as_str(),
            fixture.wpt_expected_value.as_str(),
        ) {
            value_match_count += 1;
        } else {
            triage_fixture_ids.push(fixture.id.clone());
        }
    }
    // A non-empty corpus is required: an empty fixture set (e.g. a corpus that
    // failed to deserialize) would vacuously satisfy `.all(..)` and green the value
    // gate with zero comparisons.
    let all_foldable_matches_hold = !fixtures.is_empty()
        && triage_fixture_ids
            .iter()
            .all(|id| WPT_VALUE_DIFFERENTIAL_TRIAGE.contains(&id.as_str()));
    WptValueDifferentialReportV0 {
        schema_version: "0",
        product: "omena-diff-test.wpt-value-differential",
        fixture_count: fixtures.len(),
        value_match_count,
        triage_fixture_ids,
        all_foldable_matches_hold,
    }
}

pub fn summarize_wpt_seed_corpus_metadata() -> WptSeedCorpusMetadataReportV0 {
    let manifest = serde_json::from_str::<serde_json::Value>(WPT_SEED_MANIFEST_SOURCE).ok();
    let chunks = WPT_SEED_CHUNK_SOURCES
        .iter()
        .filter_map(|source| serde_json::from_str::<serde_json::Value>(source).ok())
        .collect::<Vec<_>>();
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
    let fixture_count = wpt_seed_chunk_fixture_count(chunks.as_slice(), None);
    let blocking_fixture_count =
        wpt_seed_manifest_chunk_fixture_count(manifest.as_ref(), "stage2-blocking");
    let advisory_fixture_count =
        wpt_seed_manifest_chunk_fixture_count(manifest.as_ref(), "stage1-advisory");
    let sparse_path_fixture_counts =
        wpt_seed_sparse_path_fixture_counts(manifest.as_ref(), chunks.as_slice());
    let all_sparse_paths_have_fixtures = !sparse_path_fixture_counts.is_empty()
        && sparse_path_fixture_counts
            .iter()
            .all(|count| count.fixture_count > 0);
    let manifest_sparse_path_fixture_counts_valid =
        wpt_seed_manifest_sparse_path_fixture_counts_valid(
            manifest.as_ref(),
            sparse_path_fixture_counts.as_slice(),
        );
    let known_failure_subtests = wpt_seed_policy_known_failure_subtests();
    let known_failure_count = known_failure_subtests.len();
    let stale_known_failure_count =
        wpt_seed_stale_known_failure_count(chunks.as_slice(), known_failure_subtests.as_slice());
    let green_run_evidence_count = WPT_SEED_KNOWN_FAILURE_POLICY_SOURCE
        .lines()
        .filter(|line| line.trim() == "[[green_run]]")
        .count();
    let stage2_blocking = wpt_seed_policy_bool_value("stage2_blocking").unwrap_or(true);
    let required_min_fixture_count_for_stage2 =
        wpt_seed_policy_usize_value("required_min_fixture_count_for_stage2").unwrap_or(0);
    let required_consecutive_green_runs =
        wpt_seed_policy_usize_value("required_consecutive_green_runs").unwrap_or(0);
    let consecutive_green_runs = wpt_seed_policy_usize_value("consecutive_green_runs").unwrap_or(0);
    let known_failure_review_interval_days =
        wpt_seed_policy_usize_value("review_interval_days").unwrap_or(0);
    let all_metadata_valid = wpt_seed_manifest_metadata_valid(WptSeedManifestValidationInput {
        manifest: manifest.as_ref(),
        chunks: chunks.as_slice(),
        fixture_count,
        known_failure_count,
        stale_known_failure_count,
        green_run_evidence_count,
        stage2_blocking,
        all_sparse_paths_have_fixtures,
        manifest_sparse_path_fixture_counts_valid,
    });
    let stage2_promotion_blockers =
        wpt_seed_stage2_promotion_blockers(WptSeedStage2PromotionInput {
            all_metadata_valid,
            stage: stage.as_str(),
            stage2_blocking,
            fixture_count: blocking_fixture_count,
            known_failure_count,
            stale_known_failure_count,
            required_min_fixture_count_for_stage2,
            required_consecutive_green_runs,
            consecutive_green_runs,
        });
    let stage2_candidate_ready = stage2_promotion_blockers.is_empty();

    WptSeedCorpusMetadataReportV0 {
        schema_version: "0",
        product: "omena-diff-test.wpt-seed-corpus-metadata",
        stage,
        source_pin,
        chunk_count,
        fixture_count,
        blocking_fixture_count,
        advisory_fixture_count,
        sparse_path_fixture_counts,
        all_sparse_paths_have_fixtures,
        manifest_sparse_path_fixture_counts_valid,
        known_failure_count,
        stale_known_failure_count,
        stage2_blocking,
        required_min_fixture_count_for_stage2,
        required_consecutive_green_runs,
        consecutive_green_runs,
        green_run_evidence_count,
        known_failure_review_interval_days,
        stage2_candidate_ready,
        stage2_promotion_blockers,
        all_metadata_valid,
        closed_gates: vec![
            "wptSeedSourcePin",
            "wptSeedChunkSchema",
            "wptSeedKnownFailurePolicy",
            "wptSeedStaleKnownFailurePruning",
            "wptSeedSparsePathCoverage",
            "wptSeedGeneratedSparsePathCounts",
            "wptSeedStageOneAdvisoryLane",
            "wptSeedStageMatchesBlockingPolicy",
            "wptSeedStageTwoPromotionPolicy",
        ],
    }
}

struct WptSeedManifestValidationInput<'a> {
    manifest: Option<&'a serde_json::Value>,
    chunks: &'a [serde_json::Value],
    fixture_count: usize,
    known_failure_count: usize,
    stale_known_failure_count: usize,
    green_run_evidence_count: usize,
    stage2_blocking: bool,
    all_sparse_paths_have_fixtures: bool,
    manifest_sparse_path_fixture_counts_valid: bool,
}

fn wpt_seed_manifest_metadata_valid(input: WptSeedManifestValidationInput<'_>) -> bool {
    let WptSeedManifestValidationInput {
        manifest,
        chunks,
        fixture_count,
        known_failure_count,
        stale_known_failure_count,
        green_run_evidence_count,
        stage2_blocking,
        all_sparse_paths_have_fixtures,
        manifest_sparse_path_fixture_counts_valid,
    } = input;
    let Some(manifest) = manifest else {
        return false;
    };
    if chunks.is_empty() {
        return false;
    }
    let manifest_source_pin = manifest
        .pointer("/source/pin")
        .and_then(serde_json::Value::as_str);
    let manifest_chunks = manifest
        .get("chunks")
        .and_then(serde_json::Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default();
    let manifest_fixture_count = manifest_chunks
        .iter()
        .filter_map(|chunk| {
            chunk
                .get("fixtureCount")
                .and_then(serde_json::Value::as_u64)
        })
        .map(|value| value as usize)
        .sum::<usize>();
    let chunk_metadata_valid = manifest_chunks.len() == chunks.len()
        && manifest_chunks
            .iter()
            .zip(chunks)
            .all(|(manifest_chunk, chunk)| {
                let manifest_count = manifest_chunk
                    .get("fixtureCount")
                    .and_then(serde_json::Value::as_u64)
                    .map(|value| value as usize);
                let chunk_source_pin = chunk.get("sourcePin").and_then(serde_json::Value::as_str);
                let chunk_count = chunk
                    .get("fixtures")
                    .and_then(serde_json::Value::as_array)
                    .map(Vec::len);
                chunk
                    .get("schemaVersion")
                    .and_then(serde_json::Value::as_str)
                    == Some("0")
                    && chunk.get("product").and_then(serde_json::Value::as_str)
                        == Some("omena-diff-test.wpt-seed-corpus.chunk")
                    && manifest_count == chunk_count
                    && manifest_source_pin == chunk_source_pin
            });
    let has_advisory_chunk = manifest_chunks.iter().any(|manifest_chunk| {
        manifest_chunk
            .get("stage")
            .and_then(serde_json::Value::as_str)
            == Some("stage1-advisory")
    });
    let manifest_policy_stage2_blocking = manifest
        .pointer("/knownFailurePolicy/stage2Blocking")
        .and_then(serde_json::Value::as_bool);
    manifest
        .get("schemaVersion")
        .and_then(serde_json::Value::as_str)
        == Some("0")
        && manifest.get("product").and_then(serde_json::Value::as_str)
            == Some("omena-diff-test.wpt-seed-corpus.manifest")
        && manifest.get("stage").and_then(serde_json::Value::as_str)
            == Some(wpt_seed_expected_manifest_stage(stage2_blocking))
        && manifest_source_pin.is_some_and(wpt_source_pin_is_full_sha)
        && manifest_fixture_count == fixture_count
        && chunk_metadata_valid
        && has_advisory_chunk
        && wpt_seed_policy_string_value("schema_version") == Some("0")
        && wpt_seed_policy_string_value("stage")
            == Some(wpt_seed_expected_policy_stage(stage2_blocking))
        && manifest_policy_stage2_blocking == Some(stage2_blocking)
        && wpt_seed_policy_string_value("source_pin") == manifest_source_pin
        && wpt_seed_policy_usize_value("review_interval_days").is_some_and(|days| days > 0)
        && wpt_seed_policy_usize_value("required_min_fixture_count_for_stage2")
            .is_some_and(|count| count > 0)
        && wpt_seed_policy_usize_value("required_consecutive_green_runs")
            .is_some_and(|runs| runs > 0)
        && wpt_seed_policy_usize_value("consecutive_green_runs") == Some(green_run_evidence_count)
        && all_sparse_paths_have_fixtures
        && manifest_sparse_path_fixture_counts_valid
        && stale_known_failure_count == 0
        && known_failure_count == 0
}

struct WptSeedStage2PromotionInput<'a> {
    all_metadata_valid: bool,
    stage: &'a str,
    stage2_blocking: bool,
    fixture_count: usize,
    known_failure_count: usize,
    stale_known_failure_count: usize,
    required_min_fixture_count_for_stage2: usize,
    required_consecutive_green_runs: usize,
    consecutive_green_runs: usize,
}

fn wpt_seed_stage2_promotion_blockers(input: WptSeedStage2PromotionInput<'_>) -> Vec<&'static str> {
    let WptSeedStage2PromotionInput {
        all_metadata_valid,
        stage,
        stage2_blocking,
        fixture_count,
        known_failure_count,
        stale_known_failure_count,
        required_min_fixture_count_for_stage2,
        required_consecutive_green_runs,
        consecutive_green_runs,
    } = input;
    let mut blockers = Vec::new();
    if !all_metadata_valid {
        blockers.push("metadataInvalid");
    }
    if stage != wpt_seed_expected_manifest_stage(stage2_blocking) {
        blockers.push("stageMismatch");
    }
    if known_failure_count > 0 {
        blockers.push("knownFailuresPresent");
    }
    if stale_known_failure_count > 0 {
        blockers.push("staleKnownFailuresPresent");
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

fn wpt_seed_expected_manifest_stage(stage2_blocking: bool) -> &'static str {
    if stage2_blocking {
        "stage2-blocking"
    } else {
        "stage1-advisory"
    }
}

fn wpt_seed_expected_policy_stage(stage2_blocking: bool) -> &'static str {
    if stage2_blocking {
        "blocking"
    } else {
        "advisory"
    }
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
        if line.starts_with("[[") {
            break;
        }
        if line.is_empty() {
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

fn wpt_seed_policy_known_failure_subtests() -> Vec<(&'static str, &'static str)> {
    let mut subtests = Vec::new();
    let mut fixture: Option<&'static str> = None;
    let mut name: Option<&'static str> = None;
    let mut in_subtest = false;

    for raw_line in WPT_SEED_KNOWN_FAILURE_POLICY_SOURCE.lines() {
        let line = raw_line.split('#').next().unwrap_or("").trim();
        if line == "[[subtest]]" {
            if let (Some(fixture), Some(name)) = (fixture.take(), name.take()) {
                subtests.push((fixture, name));
            }
            in_subtest = true;
            continue;
        }
        if line.starts_with("[[") {
            if let (Some(fixture), Some(name)) = (fixture.take(), name.take()) {
                subtests.push((fixture, name));
            }
            in_subtest = false;
            continue;
        }
        if !in_subtest || line.is_empty() {
            continue;
        }
        let Some((candidate_key, value)) = line.split_once('=') else {
            continue;
        };
        match candidate_key.trim() {
            "fixture" => fixture = wpt_seed_policy_string_literal(value.trim()),
            "name" => name = wpt_seed_policy_string_literal(value.trim()),
            _ => {}
        }
    }
    if let (Some(fixture), Some(name)) = (fixture, name) {
        subtests.push((fixture, name));
    }

    subtests
}

fn wpt_seed_policy_string_literal(value: &'static str) -> Option<&'static str> {
    value.strip_prefix('"')?.strip_suffix('"')
}

fn wpt_seed_chunk_fixture_count(chunks: &[serde_json::Value], stage: Option<&str>) -> usize {
    chunks
        .iter()
        .filter(|chunk| {
            stage.is_none_or(|expected_stage| {
                chunk.get("stage").and_then(serde_json::Value::as_str) == Some(expected_stage)
            })
        })
        .filter_map(|chunk| chunk.get("fixtures").and_then(serde_json::Value::as_array))
        .map(Vec::len)
        .sum()
}

fn wpt_seed_manifest_chunk_fixture_count(
    manifest: Option<&serde_json::Value>,
    stage: &str,
) -> usize {
    manifest
        .and_then(|value| value.get("chunks"))
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .filter(|chunk| chunk.get("stage").and_then(serde_json::Value::as_str) == Some(stage))
        .filter_map(|chunk| {
            chunk
                .get("fixtureCount")
                .and_then(serde_json::Value::as_u64)
        })
        .map(|value| value as usize)
        .sum()
}

fn wpt_seed_sparse_path_fixture_counts(
    manifest: Option<&serde_json::Value>,
    chunks: &[serde_json::Value],
) -> Vec<WptSeedSparsePathFixtureCountV0> {
    let sparse_paths = manifest
        .and_then(|value| value.pointer("/source/sparsePaths"))
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(serde_json::Value::as_str)
        .collect::<Vec<_>>();

    sparse_paths
        .into_iter()
        .map(|sparse_path| {
            let fixture_count = chunks
                .iter()
                .filter_map(|chunk| chunk.get("fixtures").and_then(serde_json::Value::as_array))
                .flatten()
                .filter(|fixture| {
                    fixture
                        .get("wptPath")
                        .and_then(serde_json::Value::as_str)
                        .is_some_and(|wpt_path| wpt_path.starts_with(&format!("{sparse_path}/")))
                })
                .count();
            WptSeedSparsePathFixtureCountV0 {
                sparse_path: sparse_path.to_string(),
                fixture_count,
            }
        })
        .collect()
}

fn wpt_seed_manifest_sparse_path_fixture_counts_valid(
    manifest: Option<&serde_json::Value>,
    sparse_path_fixture_counts: &[WptSeedSparsePathFixtureCountV0],
) -> bool {
    let Some(manifest_counts) = manifest
        .and_then(|value| value.get("sparsePathFixtureCounts"))
        .and_then(serde_json::Value::as_array)
    else {
        return false;
    };

    manifest_counts.len() == sparse_path_fixture_counts.len()
        && manifest_counts.iter().zip(sparse_path_fixture_counts).all(
            |(manifest_count, expected_count)| {
                manifest_count
                    .get("sparsePath")
                    .and_then(serde_json::Value::as_str)
                    == Some(expected_count.sparse_path.as_str())
                    && manifest_count
                        .get("fixtureCount")
                        .and_then(serde_json::Value::as_u64)
                        == Some(expected_count.fixture_count as u64)
            },
        )
}

fn wpt_seed_stale_known_failure_count(
    chunks: &[serde_json::Value],
    known_failure_subtests: &[(&str, &str)],
) -> usize {
    let mut fixture_keys = BTreeSet::new();
    let mut subtest_keys = BTreeSet::new();
    for fixtures in chunks
        .iter()
        .filter_map(|value| value.get("fixtures").and_then(serde_json::Value::as_array))
    {
        for fixture in fixtures {
            let Some(id) = fixture.get("id").and_then(serde_json::Value::as_str) else {
                continue;
            };
            let Some(subtest) = fixture.get("subtest").and_then(serde_json::Value::as_str) else {
                continue;
            };
            fixture_keys.insert(id.to_string());
            subtest_keys.insert(format!("{id}\n{subtest}"));
        }
    }

    known_failure_subtests
        .iter()
        .filter(|(fixture, name)| {
            !fixture_keys.contains(*fixture)
                || !subtest_keys.contains(format!("{fixture}\n{name}").as_str())
        })
        .count()
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

fn parser_rule_syntax_node_id(source: &str, needle: &str) -> String {
    let parsed = omena_parser::parse(source, StyleDialect::Css);
    let syntax = parsed.syntax();
    syntax
        .descendants()
        .find(|node| {
            node.try_resolved()
                .map(|resolved| {
                    let text = resolved.text().to_string();
                    text.starts_with(needle) && text.contains('{')
                })
                .unwrap_or(false)
        })
        .map(|node| omena_parser::syntax_node_id(node).as_str().to_string())
        .unwrap_or_else(|| format!("missing-syntax-node:{needle}"))
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
    fn wpt_value_differential_routes_pairs_through_hand_models() {
        let report = summarize_wpt_value_differential();
        assert_eq!(report.fixture_count, 25);
        assert_eq!(report.value_match_count, 24);
        assert_eq!(
            report.triage_fixture_ids,
            vec!["css-opacity-percent-half".to_string()]
        );
        assert!(report.all_foldable_matches_hold);
        // The declared triage allowlist is tight: every declared id actually fails
        // to fold (no stale declaration that would mask a real regression).
        for id in WPT_VALUE_DIFFERENTIAL_TRIAGE {
            assert!(
                report
                    .triage_fixture_ids
                    .iter()
                    .any(|reported| reported.as_str() == *id)
            );
        }
        // Boundary integration (scalar fields + closed gate) is exercised in
        // `parser_legacy_seed_fixtures_match`, which already builds the full
        // (expensive) boundary summary.
    }

    #[test]
    fn wpt_values_agree_is_sound_string_domain_fold() {
        // Canonical color agreement (hex vs rgb, percent alpha vs decimal).
        assert!(wpt_values_agree("#FEDCBA", "rgb(254, 220, 186)"));
        assert!(wpt_values_agree("rgba(2, 3, 4, 50%)", "rgba(2, 3, 4, 0.5)"));
        // Nested calc folds to the same value as the wrapped expected.
        assert!(wpt_values_agree("calc(20px + calc(80px))", "calc(100px)"));
        assert!(wpt_values_agree("min(50%, 0%)", "calc(0%)"));
        // Raw passthrough.
        assert!(wpt_values_agree("currentcolor", "currentcolor"));
        // Soundness: genuine disagreement is never a pass.
        assert!(!wpt_values_agree("10px", "20px"));
        assert!(!wpt_values_agree("calc(10px + 10px)", "calc(100px)"));
        assert!(!wpt_values_agree("50%", "0.5"));
        // A cross-unit zero must NOT pass through the numeric branch (the reducers
        // emit `0px` vs `0deg`, never collapsed to a common `0`).
        assert!(!wpt_values_agree("calc(0px)", "0deg"));
    }

    /// A style source the real engine deterministically diagnoses: `--missing`
    /// is referenced but never declared, yielding exactly one
    /// `missingCustomProperty` diagnostic (verified in
    /// `omena-query/src/tests/style_diagnostics.rs`).
    const REAL_DIAGNOSTIC_FIXTURE: &str = r#"//- src/Component.module.scss dialect:scss
:root { --brand: red; }
.alert { color: var(--missing); }
--- expect: diagnostic
code: missingCustomProperty
--- expect: count missingCustomProperty:1
--- expect: no-diagnostic missingKeyframes
"#;

    #[test]
    fn evaluates_omena_fixture_against_real_engine_diagnostics() -> Result<(), String> {
        let fixture = parse_omena_fixture_v0(REAL_DIAGNOSTIC_FIXTURE)?;
        let outcomes = evaluate_omena_fixture_against_real_diagnostics_v0(&fixture);

        // The three diagnostic-family expectations evaluate against the REAL
        // engine output flowing through `evaluate_omena_fixture_v0_with`.
        assert_eq!(outcomes.len(), 3);
        assert!(
            outcomes.iter().all(|outcome| outcome.evaluated),
            "diagnostic-family expectations must be evaluated, not deferred: {outcomes:?}"
        );
        assert!(
            outcomes.iter().all(|outcome| outcome.satisfied),
            "real engine output must satisfy a correct fixture: {outcomes:?}"
        );
        Ok(())
    }

    #[test]
    fn wrong_expectation_fails_against_real_engine_diagnostics() -> Result<(), String> {
        // Deliberately-wrong expectation: assert the engine emits NO
        // `missingCustomProperty` for a source where it provably does. If the
        // evaluation were stubbed, this would spuriously pass; because it runs
        // the real engine, the assertion must fail.
        let fixture = parse_omena_fixture_v0(
            r#"//- src/Component.module.scss dialect:scss
:root { --brand: red; }
.alert { color: var(--missing); }
--- expect: no-diagnostic missingCustomProperty
"#,
        )?;
        let outcomes = evaluate_omena_fixture_against_real_diagnostics_v0(&fixture);

        assert_eq!(outcomes.len(), 1);
        assert!(
            outcomes[0].evaluated,
            "no-diagnostic expectation must be evaluated: {:?}",
            outcomes[0]
        );
        assert!(
            !outcomes[0].satisfied,
            "engine emits missingCustomProperty, so `no-diagnostic missingCustomProperty` must fail: {:?}",
            outcomes[0]
        );
        Ok(())
    }

    #[test]
    fn clean_source_produces_no_diagnostics_from_real_engine() -> Result<(), String> {
        // A fully-resolved source emits no diagnostics; a `no-diagnostic`
        // expectation against it passes through the real engine path.
        let fixture = parse_omena_fixture_v0(
            r#"//- src/Component.module.scss dialect:scss
:root { --brand: red; }
.alert { color: var(--brand); }
--- expect: no-diagnostic missingCustomProperty
--- expect: count missingCustomProperty:0
"#,
        )?;
        let outcomes = evaluate_omena_fixture_against_real_diagnostics_v0(&fixture);

        assert!(
            outcomes
                .iter()
                .all(|outcome| outcome.evaluated && outcome.satisfied),
            "clean source must satisfy no-diagnostic expectations: {outcomes:?}"
        );
        Ok(())
    }

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
        assert!(summary.wpt_seed_fixture_count >= 25);
        assert!(summary.wpt_value_differential_fixture_count >= 25);
        assert_eq!(
            summary.wpt_value_differential_fixture_count,
            summary.wpt_value_differential_match_count
                + summary.wpt_value_differential_triage_count
        );
        assert!(
            summary
                .wpt_value_differential_report
                .all_foldable_matches_hold
        );
        assert!(
            summary
                .closed_gates
                .contains(&"wptValueDifferentialHandModelAgreement")
        );
        assert_eq!(summary.soundiness_metamorphic_relation_count, 3);
        assert!(summary.all_soundiness_metamorphic_relations_hold);
        assert_eq!(summary.diagnostic_metamorphic_relation_count, 7);
        assert!(summary.all_diagnostic_metamorphic_relations_hold);
        assert_eq!(
            summary.wpt_seed_metadata_report.stale_known_failure_count,
            0
        );
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
        assert!(
            summary
                .closed_gates
                .contains(&"wptValueDifferentialHandModelAgreement")
        );
        assert!(
            summary
                .closed_gates
                .contains(&"soundinessMetamorphicRelations")
        );
        assert!(
            summary
                .closed_gates
                .contains(&"diagnosticMetamorphicRelations")
        );
        assert!(
            summary
                .closed_gates
                .contains(&"parallelSalsaViewsVsFromScratchEquivalence")
        );
        assert!(
            summary
                .closed_gates
                .contains(&"externalSifsSalsaMemoizedVsFromScratchEquivalence")
        );
        assert!(summary.all_parallel_salsa_equivalence_phases_identical);
        assert_eq!(
            summary.parallel_salsa_equivalence_comparison_count,
            summary.cache_equivalence_file_count * 4,
        );
        assert!(
            summary
                .wpt_seed_metadata_report
                .closed_gates
                .contains(&"wptSeedStaleKnownFailurePruning")
        );
    }

    #[test]
    fn soundiness_metamorphic_relations_hold_on_real_diagnostics_path() {
        let report = summarize_soundiness_metamorphic_relations();

        assert_eq!(
            report.product,
            "omena-diff-test.soundiness-metamorphic-relations"
        );
        assert_eq!(report.relation_count, 3);
        assert!(report.all_relations_hold);
        assert!(
            report
                .closed_gates
                .contains(&"soundinessMetamorphicRelationHarness")
        );
        assert!(
            report
                .closed_gates
                .contains(&"soundinessResolvedSifRelation")
        );
        assert!(
            report
                .closed_gates
                .contains(&"soundinessMonotonicityRelation")
        );
        assert!(
            report
                .closed_gates
                .contains(&"soundinessBlobSoundnessRelation")
        );
        assert!(report.relations.iter().all(|relation| relation.holds));
        assert!(report.relations.iter().any(|relation| {
            relation.relation == "mr-soundiness-resolved-sif"
                && relation
                    .before_diagnostic_codes
                    .contains(&"missingExternalSif".to_string())
                && !relation
                    .after_diagnostic_codes
                    .contains(&"missingExternalSif".to_string())
        }));
        assert!(report.relations.iter().any(|relation| {
            relation.relation == "mr-monotonicity-partial-to-complete-sif"
                && relation
                    .before_diagnostic_codes
                    .contains(&"partialExternalSif".to_string())
                && !relation
                    .after_diagnostic_codes
                    .contains(&"partialExternalSif".to_string())
        }));
        assert!(report.relations.iter().any(|relation| {
            relation.relation == "mr-blob-soundness-closed-world-exposes-top-any"
                && !relation
                    .before_diagnostic_codes
                    .contains(&"missingSassSymbol".to_string())
                && relation
                    .after_diagnostic_codes
                    .contains(&"missingSassSymbol".to_string())
        }));
    }

    #[test]
    fn diagnostic_metamorphic_relations_hold_on_real_diagnostics_path() {
        let report = summarize_diagnostic_metamorphic_relations();

        assert_eq!(
            report.product,
            "omena-diff-test.diagnostic-metamorphic-relations"
        );
        assert_eq!(report.relation_count, 7);
        assert!(report.all_relations_hold);
        assert!(
            report
                .closed_gates
                .contains(&"diagnosticMetamorphicRelationHarness")
        );
        assert!(report.relations.iter().all(|relation| relation.holds));
        for relation in [
            "mr-selector-list-distribution",
            "mr-declaration-permutation",
            "mr-selector-rename-invariance",
            "mr-whitespace-comment-invariance",
            "mr-nesting-depth-equivalence",
            "mr-media-query-equivalence",
            "mr-canonicalizer-idempotence",
        ] {
            assert!(
                report
                    .relations
                    .iter()
                    .any(|candidate| candidate.relation == relation),
                "missing diagnostic metamorphic relation: {relation}"
            );
        }
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
        assert_eq!(report.fixture_grammar, "omena-fixture-v0");
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
    fn parses_reusable_omena_fixture_v0_sections() -> Result<(), String> {
        let seed = M3_THEORETICAL_MOAT_FIXTURE_SEEDS
            .iter()
            .find(|seed| seed.label == "cascade-transform-proof-obligations")
            .ok_or_else(|| "cascade fixture seed should stay registered".to_string())?;
        let fixture = parse_omena_fixture_v0(seed.raw)?;

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
        assert_eq!(report.stage, "stage2-blocking");
        assert!(wpt_source_pin_is_full_sha(report.source_pin.as_str()));
        assert_eq!(report.chunk_count, 2);
        assert!(report.fixture_count > report.blocking_fixture_count);
        assert!(report.blocking_fixture_count >= 25);
        assert!(report.advisory_fixture_count > 0);
        assert!(report.all_sparse_paths_have_fixtures);
        assert!(
            report
                .sparse_path_fixture_counts
                .iter()
                .any(|count| count.sparse_path == "css/css-values" && count.fixture_count > 0)
        );
        assert!(
            report
                .sparse_path_fixture_counts
                .iter()
                .any(|count| count.sparse_path == "css/css-color" && count.fixture_count > 0)
        );
        assert_eq!(report.known_failure_count, 0);
        assert!(report.manifest_sparse_path_fixture_counts_valid);
        assert!(report.stage2_blocking);
        assert_eq!(report.required_min_fixture_count_for_stage2, 25);
        assert_eq!(report.required_consecutive_green_runs, 5);
        assert_eq!(report.consecutive_green_runs, 5);
        assert_eq!(report.green_run_evidence_count, 5);
        assert_eq!(report.known_failure_review_interval_days, 30);
        assert!(report.stage2_candidate_ready);
        assert!(
            !report
                .stage2_promotion_blockers
                .contains(&"seedCorpusBelowStageTwoMinimum")
        );
        assert!(
            !report
                .stage2_promotion_blockers
                .contains(&"insufficientConsecutiveGreenRuns")
        );
        assert!(report.all_metadata_valid);
        assert!(report.closed_gates.contains(&"wptSeedSourcePin"));
        assert!(report.closed_gates.contains(&"wptSeedKnownFailurePolicy"));
        assert!(report.closed_gates.contains(&"wptSeedSparsePathCoverage"));
        assert!(
            report
                .closed_gates
                .contains(&"wptSeedGeneratedSparsePathCounts")
        );
        assert!(report.closed_gates.contains(&"wptSeedStageOneAdvisoryLane"));
        assert!(
            report
                .closed_gates
                .contains(&"wptSeedStageTwoPromotionPolicy")
        );
    }

    #[test]
    fn wpt_seed_stale_known_failure_count_detects_orphans() {
        let chunk = serde_json::json!({
            "fixtures": [
                {
                    "id": "css/css-values/fixture-a.html",
                    "subtest": "supported subtest"
                },
                {
                    "id": "css/css-values/fixture-b.html",
                    "subtest": "still present"
                }
            ]
        });
        let known_failure_subtests = [
            ("css/css-values/fixture-a.html", "supported subtest"),
            ("css/css-values/fixture-b.html", "stale subtest"),
            ("css/css-values/removed-fixture.html", "removed fixture"),
        ];

        assert_eq!(
            wpt_seed_stale_known_failure_count(&[chunk], &known_failure_subtests),
            2
        );
    }

    #[test]
    fn reports_incremental_identity_reuse_equivalence_with_field_reports() {
        let report = summarize_incremental_identity_reuse_equivalence_v0();

        assert_eq!(
            report.product,
            "omena-diff-test.incremental-identity-reuse-equivalence"
        );
        assert!(report.unchanged_syntax_id_stable);
        assert!(report.changed_syntax_id_differs);
        assert!(report.incremental_matches_from_scratch_delta);
        assert!(report.all_fields_match);
        assert_eq!(
            report
                .fields
                .iter()
                .map(|field| field.field)
                .collect::<Vec<_>>(),
            vec![
                "nodeIdentityDigest",
                "dependencyEdges",
                "dirtyIds",
                "reusableCleanIds"
            ]
        );
        assert!(report.fields.iter().all(|field| field.matches));
    }
}
