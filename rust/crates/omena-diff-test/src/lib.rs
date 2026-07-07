//! Differential corpus harness for the omena-css parser stack.
//!
//! This crate is the Rust workspace home for parser differential checks that
//! were previously represented only by runner scripts. It treats
//! `engine-style-parser` as a legacy oracle and `omena-parser` as the product
//! parser surface.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use engine_input_producers::{
    EngineInputV2, StringTypeFactsV2, TypeFactControlFlowBlockV2, TypeFactControlFlowGraphV2,
    TypeFactEntryV2, summarize_expression_domain_control_flow_analysis_input,
    summarize_expression_domain_flow_analysis_input,
};
use engine_style_parser::{parse_style_module, summarize_css_modules_intermediate};
use omena_abstract_value::{
    AbstractClassValueV0, AbstractCssValueV0, abstract_class_value_kind,
    abstract_css_values_canonically_equal, enumerate_finite_class_values,
    join_abstract_class_values,
};
use omena_benchmarks::{bundler_productization_corpus, style_corpus};
use omena_cascade::{SelectorMatchVerdict, selector_context_witness};
use omena_cross_file_summary::{
    CROSS_FILE_SUMMARY_NODE_ROLE_LABELS_V0, CROSS_FILE_SUMMARY_RAW_EDGE_KIND_LABELS_V0,
    UNIFIED_HYPERGRAPH_EDGE_KIND_VARIANTS_V0, summarize_cross_file_graph_delta_v0,
    summarize_cross_file_summary_view_v0,
};
use omena_incremental::{
    IncrementalGraphInputV0, IncrementalNodeInputV0, IncrementalRevisionV0,
    OmenaIncrementalDatabaseV0, OmenaWorkspaceSnapshotIdV0, snapshot_from_graph_input,
};
use omena_parser::{
    ParsedStyleFacts, StyleDialect, facts_from_cst, parse, summarize_omena_parser_style_facts,
};
use omena_query::{
    OmenaQueryExternalModuleModeV0, OmenaQueryExternalSifInputV0, OmenaQuerySourceDocumentInputV0,
    OmenaQueryStyleDiagnosticsForFileV0, OmenaQueryStyleMemoHostV0, OmenaQueryStyleSourceInputV0,
    summarize_omena_query_style_diagnostics_for_file,
    summarize_omena_query_style_diagnostics_for_workspace_file_with_external_mode_and_sifs,
    summarize_omena_query_style_hover_candidates,
    summarize_omena_query_workspace_cross_file_summary,
};
use omena_scss_eval::{
    OmenaScssEvalTruthinessCstEquivalenceReportV0, summarize_scss_eval_truthiness_cst_equivalence,
    summarize_static_stylesheet_value_resolution,
};
use omena_semantic::summarize_omena_parser_style_semantic_boundary_from_source;
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
use omena_transform_cst::summarize_transform_ir_identity_round_trip;
use serde::{Deserialize, Serialize};

mod cache_equivalence;
mod deletion_stale_reuse;
mod external_corpus_envelope_idl_generated;
mod fold_reachability_soundness;
pub mod hrx;
mod oss_corpus_farm;
mod reachability_equivalence;
mod scss_eval_equivalence;
mod source_precision;
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
pub use deletion_stale_reuse::{
    OmenaDiffDeletionStaleReuseCorpusReportV0, OmenaDiffDeletionStaleReuseFixtureReportV0,
    summarize_deletion_stale_reuse_corpus_v0,
};
pub use fold_reachability_soundness::{
    OmenaDiffNativeCssFoldPruneArtifactRecordV0, OmenaDiffNativeCssFoldPruneBranchSiteReportV0,
    OmenaDiffNativeCssFoldPruneFixtureReportV0, OmenaDiffNativeCssFoldPruneReportV0,
    summarize_native_css_fold_prune_branch_agreement_v0,
};
pub use oss_corpus_farm::{
    OmenaDiffOssCorpusFarmManifestReportV0, summarize_oss_corpus_farm_manifest_v0,
};
pub use reachability_equivalence::{
    OmenaDiffClosureHashBitsetParityFileReportV0, OmenaDiffReachabilityEquivalenceFileReportV0,
    OmenaDiffReachabilityEquivalenceReportV0, OmenaDiffSelectorEqualityRelationReportV0,
    summarize_reachability_second_oracle_equivalence_v0,
};
pub use scss_eval_equivalence::{
    OmenaDiffScssEvalPublicSummaryEquivalenceReportV0,
    summarize_scss_eval_public_summary_equivalence_v0,
};
pub use source_precision::{
    OmenaSourcePrecisionBaselineV0, OmenaSourcePrecisionReferenceReportV0,
    summarize_omena_source_precision_baseline,
};

const PARSER_CST_FACT_AUTHORITY_SNAPSHOT_SOURCE: &str =
    include_str!("../regressions/parser-cst-fact-authority.json");

#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct TemplatePlaceholderDefaultNoneFixtureV0 {
    id: &'static str,
    dialect: StyleDialect,
    source: &'static str,
    expected_token_hash: u64,
    expected_syntax_hash: u64,
}

#[cfg(test)]
const TEMPLATE_PLACEHOLDER_DEFAULT_NONE_FIXTURES: &[TemplatePlaceholderDefaultNoneFixtureV0] = &[
    TemplatePlaceholderDefaultNoneFixtureV0 {
        id: "css-template-bytes",
        dialect: StyleDialect::Css,
        source: ".button { color: ${color}; content: \"#{literal}\"; }",
        expected_token_hash: 15062872718744947890,
        expected_syntax_hash: 13984136162951412351,
    },
    TemplatePlaceholderDefaultNoneFixtureV0 {
        id: "scss-template-and-native-interpolation",
        dialect: StyleDialect::Scss,
        source: ".button-#{$variant} { color: ${color}; content: \"@{literal}\"; }",
        expected_token_hash: 918384541960856734,
        expected_syntax_hash: 15272715944274077939,
    },
    TemplatePlaceholderDefaultNoneFixtureV0 {
        id: "sass-template-and-native-interpolation",
        dialect: StyleDialect::Sass,
        source: ".button-#{$variant}\n  color: ${color}\n  content: \"@{literal}\"\n",
        expected_token_hash: 753703821381278381,
        expected_syntax_hash: 16457391882163996945,
    },
    TemplatePlaceholderDefaultNoneFixtureV0 {
        id: "less-template-and-native-interpolation",
        dialect: StyleDialect::Less,
        source: ".button-@{variant} { color: ${color}; content: \"#{literal}\"; }",
        expected_token_hash: 7742129699976696706,
        expected_syntax_hash: 14499180279058795893,
    },
];

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

fn style_dialect_label(dialect: StyleDialect) -> &'static str {
    match dialect {
        StyleDialect::Css => "css",
        StyleDialect::Scss => "scss",
        StyleDialect::Sass => "sass",
        StyleDialect::Less => "less",
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
pub struct TransformIrIdentityRoundTripFixtureReportV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub label: &'static str,
    pub dialect: &'static str,
    pub node_count: usize,
    pub fields: Vec<DiffFieldReport>,
    pub all_fields_match: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformIrIdentityRoundTripEquivalenceReportV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub fixture_count: usize,
    pub reports: Vec<TransformIrIdentityRoundTripFixtureReportV0>,
    pub all_fields_match: bool,
    pub closed_gates: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ParserCstFactAuthorityCategoryReportV0 {
    pub fixture: &'static str,
    pub category: &'static str,
    pub legacy_values: Vec<String>,
    pub cst_values: Vec<String>,
    pub legacy_spans: Vec<String>,
    pub cst_spans: Vec<String>,
    pub values_match: bool,
    pub spans_match: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ParserCstFactAuthorityMetamorphicReportV0 {
    pub relation: &'static str,
    pub fixture: &'static str,
    pub before_values: Vec<String>,
    pub after_values: Vec<String>,
    pub holds: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ParserCstFactAuthorityReportV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub fixture_count: usize,
    pub category_count: usize,
    pub comparisons: Vec<ParserCstFactAuthorityCategoryReportV0>,
    pub all_value_sets_match: bool,
    pub all_span_sets_match: bool,
    pub metamorphic_relation_count: usize,
    pub metamorphic_relations: Vec<ParserCstFactAuthorityMetamorphicReportV0>,
    pub all_metamorphic_relations_hold: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ParserCstFactAuthoritySnapshotV0 {
    fixture_count: usize,
    category_count: usize,
    comparisons: Vec<ParserCstFactAuthoritySnapshotCategoryV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ParserCstFactAuthoritySnapshotCategoryV0 {
    fixture: String,
    category: String,
    legacy_values: Vec<String>,
    legacy_spans: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ParserCstContextRawScanFixture {
    label: &'static str,
    file_path: &'static str,
    source: &'static str,
    expected_statement_layers: &'static [&'static str],
    expected_block_layers: &'static [&'static str],
    expected_layer_selector_memberships: &'static [&'static str],
    rejected_names: &'static [&'static str],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SelectorContextSoundnessFixtureV0 {
    fixture_id: &'static str,
    declaration_selector: &'static str,
    reference_selector: &'static str,
    expected_verdict: SelectorMatchVerdict,
    baseline_positive: bool,
    removed_spurious_positive: bool,
    unmodeled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ParserCstContextRawScanFixtureReportV0 {
    pub label: &'static str,
    pub statement_layers: Vec<String>,
    pub expected_statement_layers: Vec<&'static str>,
    pub block_layers: Vec<String>,
    pub expected_block_layers: Vec<&'static str>,
    pub layer_selector_memberships: Vec<String>,
    pub expected_layer_selector_memberships: Vec<&'static str>,
    pub rejected_names: Vec<&'static str>,
    pub rejected_names_absent: bool,
    pub matches: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ParserCstContextRawScanDivergenceReportV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub fixture_count: usize,
    pub reports: Vec<ParserCstContextRawScanFixtureReportV0>,
    pub all_fixtures_match: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SelectorContextSoundnessFixtureReportV0 {
    pub fixture_id: &'static str,
    pub declaration_selector: &'static str,
    pub reference_selector: &'static str,
    pub expected_verdict: SelectorMatchVerdict,
    pub actual_verdict: SelectorMatchVerdict,
    pub expected_match: bool,
    pub actual_match: bool,
    pub baseline_positive: bool,
    pub removed_spurious_positive: bool,
    pub unmodeled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SelectorContextSoundnessReportV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub fixture_count: usize,
    pub reports: Vec<SelectorContextSoundnessFixtureReportV0>,
    pub all_expected_verdicts_match: bool,
    pub all_unmodeled_fixtures_stay_maybe: bool,
    pub baseline_positive_count: usize,
    pub removed_spurious_positive_count: usize,
    pub actual_positive_count: usize,
    pub positive_preservation_matches: bool,
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceCfgRefinementFixtureReportV0 {
    pub fixture_id: &'static str,
    pub baseline_graph_id: String,
    pub source_graph_ids: Vec<String>,
    pub baseline_value_kind: &'static str,
    pub source_value_kinds: Vec<&'static str>,
    pub all_source_values_le_baseline: bool,
    pub strict_refinement_count: usize,
    pub all_source_values_covered_by_baseline: bool,
    pub branch_block_observed: bool,
    pub concat_transfer_observed: bool,
    pub file_merge_absent_from_source_cfg: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceCfgRefinementOracleReportV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub fixture_count: usize,
    pub strict_refinement_witness_count: usize,
    pub all_source_values_le_baseline: bool,
    pub all_source_values_covered_by_baseline: bool,
    pub all_shape_witnesses_present: bool,
    pub reports: Vec<SourceCfgRefinementFixtureReportV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TypedGraphSummaryPlaneFoundationReportV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub status: &'static str,
    pub fixture_count: usize,
    pub unified_edge_kind_variant_count: usize,
    pub raw_edge_kind_catalog_count: usize,
    pub node_role_catalog_count: usize,
    pub summary_edge_count: usize,
    pub graph_delta_added_edge_count: usize,
    pub graph_delta_removed_edge_count: usize,
    pub all_summary_views_ready: bool,
    pub all_summary_view_json_counts_match: bool,
    pub all_raw_edge_kinds_in_catalog: bool,
    pub all_node_roles_in_catalog: bool,
    pub known_edit_graph_delta_ready: bool,
    pub workspace_snapshot_id_contract_ready: bool,
    pub workspace_snapshot_id_contract_status: &'static str,
    pub workspace_snapshot_id_type_census_count: usize,
    pub workspace_snapshot_id_surface_without_id_count: usize,
    pub workspace_snapshot_id_rekey_equivalence_ready: bool,
    pub workspace_snapshot_id_query_surface_ready: bool,
    pub workspace_snapshot_id_lsp_report_surface_ready: bool,
    pub workspace_graph_contract_texts_ready: bool,
    pub workspace_summary_plane_and_snapshot_id_green: bool,
    pub all_foundation_checks_hold: bool,
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
    /// Imported sass-spec fixture count.
    pub sass_spec_imported_fixture_count: usize,
    /// HRX archives currently scanned from the imported sass-spec source root.
    pub sass_spec_import_source_archive_count: usize,
    /// Imported sass-spec chunk count.
    pub sass_spec_import_chunk_count: usize,
    /// Per-push sass-spec smoke fixture count.
    pub sass_spec_per_push_smoke_fixture_count: usize,
    /// Minimum fixture count required by the seed smoke floor.
    pub sass_spec_per_push_smoke_floor_fixture_count: usize,
    /// Whether imported sass-spec manifest, chunk, and source-scan counts agree.
    pub all_sass_spec_import_scale_counts_match: bool,
    /// Whether the per-push smoke subset keeps the seed floor.
    pub all_sass_spec_smoke_floor_holds: bool,
    /// Imported sass-spec fixtures expected to match statically.
    pub sass_spec_static_must_match_count: usize,
    /// Imported sass-spec fixtures expected to stay in the sound-bail bucket.
    pub sass_spec_expected_sound_bail_count: usize,
    /// Imported sass-spec fixtures routed to parser-recovery checks.
    pub sass_spec_parser_recovery_count: usize,
    /// Imported sass-spec fixtures excluded by an explicit policy.
    pub sass_spec_out_of_scope_count: usize,
    /// Whether every imported sass-spec fixture carries exactly one expectation bucket.
    pub all_sass_spec_imported_fixtures_have_one_expectation_kind: bool,
    /// Whether every imported sass-spec bucket matches the current criteria.
    pub all_sass_spec_imported_expectation_kinds_match_criteria: bool,
    /// Imported sass-spec sound-bail fixture count.
    pub sass_spec_sound_bail_case_count: usize,
    /// Imported sass-spec sound-bail cases checked against dart-sass values.
    pub sass_spec_sound_bail_checked_case_count: usize,
    /// Imported sass-spec sound-bail cases with a non-top abstract value.
    pub sass_spec_sound_bail_non_top_case_count: usize,
    /// Imported sass-spec sound-bail cases exercising raw-value tightness.
    pub sass_spec_sound_bail_raw_tightness_case_count: usize,
    /// Whether every sound-bail concrete value is included by its abstract value.
    pub all_sass_spec_sound_bail_membership_checks_hold: bool,
    /// Imported sass-spec static equality fixture count.
    pub sass_spec_static_match_case_count: usize,
    /// Imported sass-spec static equality cases checked against dart-sass values.
    pub sass_spec_static_match_checked_case_count: usize,
    /// Imported sass-spec static equality declaration values matched by omena.
    pub sass_spec_static_match_declaration_value_match_count: usize,
    /// Whether every static equality dart-sass value is matched by omena resolution.
    pub all_sass_spec_static_match_checks_hold: bool,
    /// Whether imported sass-spec bucket totals match the committed ledger.
    pub all_sass_spec_expectation_bucket_totals_match_ledger: bool,
    /// Whether imported sass-spec fixture assignments match the committed ledger.
    pub all_sass_spec_expectation_fixture_assignments_match_ledger: bool,
    /// Current semantic bail-site count from the SCSS evaluator sources.
    pub sass_spec_bail_site_ledger_site_count: usize,
    /// Raw bail-pattern hit count including non-semantic matches.
    pub sass_spec_bail_site_raw_pattern_hit_count: usize,
    /// Non-semantic bail-pattern hit count excluded from the ledger.
    pub sass_spec_bail_site_non_semantic_pattern_hit_count: usize,
    /// Bail-site ledger entries that link at least one imported fixture.
    pub sass_spec_bail_site_linked_site_count: usize,
    /// Bail-site ledger entries that carry a named coverage gap.
    pub sass_spec_bail_site_named_gap_count: usize,
    /// Imported fixtures linked from the bail-site ledger.
    pub sass_spec_bail_site_linked_case_count: usize,
    /// Whether the committed bail-site ledger matches the current source census.
    pub all_sass_spec_bail_site_ledger_counts_match: bool,
    /// Whether every bail-site ledger entry links a case or names a gap.
    pub all_sass_spec_bail_site_ledger_entries_linked_or_named_gap: bool,
    /// Whether every linked case uses the same reason class as its bail site.
    pub all_sass_spec_bail_site_ledger_link_reason_classes_match: bool,
    /// Whether linked cases are imported expected sound-bail fixtures.
    pub all_sass_spec_bail_site_ledger_linked_cases_are_imported: bool,
    /// Soundiness metamorphic relation count.
    pub soundiness_metamorphic_relation_count: usize,
    /// Whether every soundiness metamorphic relation currently holds.
    pub all_soundiness_metamorphic_relations_hold: bool,
    /// Diagnostic metamorphic relation count.
    pub diagnostic_metamorphic_relation_count: usize,
    /// Whether every diagnostic metamorphic relation currently holds.
    pub all_diagnostic_metamorphic_relations_hold: bool,
    /// Parser fact authority fixtures compared against the CST-derived path.
    pub parser_cst_fact_authority_fixture_count: usize,
    /// Parser fact authority category comparisons.
    pub parser_cst_fact_authority_comparison_count: usize,
    /// Whether parser fact values match the internal authority oracle.
    pub all_parser_cst_fact_authority_values_match: bool,
    /// Whether parser fact spans match the internal authority oracle.
    pub all_parser_cst_fact_authority_spans_match: bool,
    /// Parser fact authority metamorphic relation count.
    pub parser_cst_fact_authority_metamorphic_relation_count: usize,
    /// Whether parser fact authority metamorphic relations hold.
    pub all_parser_cst_fact_authority_metamorphic_relations_hold: bool,
    /// Context-index fixtures that keep comment/string/interpolation text out of facts.
    pub parser_cst_context_raw_scan_fixture_count: usize,
    /// Whether context-index fixtures match their intended CST-derived output.
    pub all_parser_cst_context_raw_scan_fixtures_match: bool,
    /// Selector-context soundness fixtures for cascade-aware variable lookup.
    pub selector_context_soundness_fixture_count: usize,
    /// Whether selector-context verdicts match the soundness corpus.
    pub all_selector_context_soundness_fixtures_match: bool,
    /// Source-CFG-vs-file-merge refinement fixture count.
    pub source_cfg_refinement_fixture_count: usize,
    /// Whether every source-CFG fixture is equal-or-more precise than baseline.
    pub all_source_cfg_values_le_file_merge_baseline: bool,
    /// Strict source-CFG refinement witnesses.
    pub source_cfg_strict_refinement_witness_count: usize,
    /// Whether source-CFG values remain covered by the baseline value set.
    pub all_source_cfg_values_covered_by_file_merge_baseline: bool,
    /// Whether source-CFG shape witnesses include branch and concat transfers.
    pub all_source_cfg_shape_witnesses_present: bool,
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
    /// Reachability fixtures compared against an independent Datalog witness.
    pub reachability_equivalence_fixture_count: usize,
    /// Whether the independent reachability witness matched the public batch oracle.
    pub all_reachability_second_oracle_sets_equal: bool,
    /// Whether the streaming reachability oracle matched the public batch oracle.
    pub all_reachability_streaming_matches_batch: bool,
    /// Whether the dense bitset reachability arm matched the public batch oracle.
    pub all_reachability_bitset_parity_equal: bool,
    /// Whether closed-world closure hashes match between BTreeSet and bitset reachability arms.
    pub all_reachability_closure_hash_bitset_parity_equal: bool,
    /// Whether warm product reachability stayed parity-gated against batch on the same fixtures.
    pub all_reachability_product_parity_with_batch: bool,
    /// Whether incremental, batch, and independent Datalog fact-key sets match.
    pub all_reachability_fact_keys_three_way_equal: bool,
    /// Whether demand, projected batch, incremental, and independent Datalog fact-key gates hold.
    pub all_reachability_fact_keys_four_way_equal: bool,
    /// Whether the selector side relation agreed with its seeded fixtures.
    pub all_reachability_selector_relations_equal: bool,
    /// SCSS evaluator truthiness fixtures compared through scanner and CST paths.
    pub scss_eval_truthiness_cst_equivalence_fixture_count: usize,
    /// Whether scanner and CST truthiness agree for every migration fixture.
    pub all_scss_eval_truthiness_cst_equivalence_fixtures_match: bool,
    /// Public SCSS evaluator summary snapshot comparisons.
    pub scss_eval_public_summary_comparison_count: usize,
    /// Whether every public SCSS evaluator summary matches its pinned JSON hash.
    pub all_scss_eval_public_summaries_match: bool,
    /// Transform IR identity round-trip fixture count.
    pub transform_ir_identity_round_trip_fixture_count: usize,
    /// Whether the transform IR lowering/printer keeps original bytes and origins.
    pub all_transform_ir_identity_round_trip_fields_match: bool,
    /// OSS corpus farm manifest entry count.
    pub oss_corpus_farm_entry_count: usize,
    /// Distinct upstream repositories used by the OSS corpus farm.
    pub oss_corpus_farm_repository_count: usize,
    /// Distinct dialects covered by the OSS corpus farm.
    pub oss_corpus_farm_dialect_count: usize,
    /// Whether every OSS corpus farm manifest check currently holds.
    pub all_oss_corpus_farm_manifest_checks_hold: bool,
    /// Deletion stale-reuse corpus fixture count.
    pub deletion_stale_reuse_fixture_count: usize,
    /// Deletion stale-reuse fixtures that characterize stale incremental output facts.
    pub deletion_stale_reuse_divergence_fixture_count: usize,
    /// In-cycle deletion fixtures whose warm reachability set changes.
    pub deletion_stale_reuse_cycle_deletion_fixture_count: usize,
    /// Deletion fixtures whose demand facts match batch facts on the query projection.
    pub deletion_stale_reuse_demand_projected_equal_fixture_count: usize,
    /// Whether every deletion fixture keeps demand facts equal to projected batch facts.
    pub deletion_stale_reuse_all_demand_projected_equal: bool,
    /// Whether the deletion stale-reuse corpus exposes both required consumer shapes.
    pub deletion_stale_reuse_ready_for_relocation_consumer: bool,
    /// Whether typed summary vocabulary/view/delta checks hold on the summary corpus.
    pub all_typed_graph_summary_plane_foundation_checks_hold: bool,
    /// Whether the summary-plane and snapshot-id contract checks hold together.
    pub workspace_summary_plane_and_snapshot_id_green: bool,
    /// WPT-style seed metadata report.
    pub wpt_seed_metadata_report: WptSeedCorpusMetadataReportV0,
    /// WPT value-differential report (specified-value hand-model agreement).
    pub wpt_value_differential_report: WptValueDifferentialReportV0,
    /// Imported sass-spec expectation bucket report.
    pub sass_spec_expectation_bucket_report: SassSpecExpectationBucketReportV0,
    /// Imported sass-spec sound-bail membership report.
    pub sass_spec_sound_bail_membership_report: SassSpecSoundBailMembershipReportV0,
    /// Imported sass-spec static equality report.
    pub sass_spec_static_match_report: SassSpecStaticMustMatchReportV0,
    /// Imported sass-spec expectation bucket ledger report.
    pub sass_spec_expectation_bucket_ledger_report: SassSpecExpectationBucketLedgerReportV0,
    /// Imported sass-spec bail-site ledger report.
    pub sass_spec_bail_site_ledger_report: SassSpecBailSiteLedgerReportV0,
    /// Imported sass-spec scale and smoke-floor report.
    pub sass_spec_import_scale_report: SassSpecImportScaleReportV0,
    /// Soundiness metamorphic relation report.
    pub soundiness_metamorphic_report: SoundinessMetamorphicReportV0,
    /// Internal omena-vs-omena diagnostic metamorphic relation report.
    pub diagnostic_metamorphic_report: DiagnosticMetamorphicReportV0,
    /// Internal parser fact authority report.
    pub parser_cst_fact_authority_report: ParserCstFactAuthorityReportV0,
    /// Transform IR identity round-trip report.
    pub transform_ir_identity_round_trip_report: TransformIrIdentityRoundTripEquivalenceReportV0,
    /// CST-derived context-index raw-text divergence report.
    pub parser_cst_context_raw_scan_report: ParserCstContextRawScanDivergenceReportV0,
    /// Selector-context soundness corpus report.
    pub selector_context_soundness_report: SelectorContextSoundnessReportV0,
    /// Source-CFG refinement oracle report.
    pub source_cfg_refinement_report: SourceCfgRefinementOracleReportV0,
    /// Cached-vs-from-scratch diagnostic equivalence report (RFC 0009 §0).
    pub cache_equivalence_report: OmenaDiffCacheEquivalenceReportV0,
    /// Salsa-memo lifecycle equivalence report (RFC 0009 Pillar B).
    pub salsa_memo_equivalence_report: OmenaDiffSalsaMemoEquivalenceReportV0,
    /// Parallel fixed-revision view equivalence report (RFC 0009 Pillar F).
    pub parallel_salsa_equivalence_report: OmenaDiffSalsaMemoEquivalenceReportV0,
    /// Reachability second-oracle equivalence report.
    pub reachability_equivalence_report: OmenaDiffReachabilityEquivalenceReportV0,
    /// SCSS evaluator scanner-vs-CST truthiness migration report.
    pub scss_eval_truthiness_cst_equivalence_report: OmenaScssEvalTruthinessCstEquivalenceReportV0,
    /// SCSS evaluator public summary preservation report.
    pub scss_eval_public_summary_equivalence_report:
        OmenaDiffScssEvalPublicSummaryEquivalenceReportV0,
    /// OSS corpus farm manifest report.
    pub oss_corpus_farm_manifest_report: OmenaDiffOssCorpusFarmManifestReportV0,
    /// Deletion stale-reuse corpus report.
    pub deletion_stale_reuse_corpus_report: OmenaDiffDeletionStaleReuseCorpusReportV0,
    /// Typed graph summary plane foundation report.
    pub typed_graph_summary_plane_foundation_report: TypedGraphSummaryPlaneFoundationReportV0,
    /// Named evidence gates closed by this crate.
    pub closed_gates: Vec<&'static str>,
    /// Field-level reports for every seed fixture.
    pub reports: Vec<ParserDifferentialReport>,
    /// M3 fixture seed corpus report.
    pub m3_fixture_seed_report: M3FixtureSeedCorpusReportV0,
}

/// Imported sass-spec expectation bucket counts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SassSpecExpectationBucketReportV0 {
    /// Schema version.
    pub schema_version: &'static str,
    /// Product surface name.
    pub product: &'static str,
    /// Imported fixture count.
    pub fixture_count: usize,
    /// Static equality bucket count.
    pub static_must_match_count: usize,
    /// Abstract-value membership bucket count.
    pub expected_sound_bail_count: usize,
    /// Parser-recovery bucket count.
    pub parser_recovery_count: usize,
    /// Explicit exclusion bucket count.
    pub out_of_scope_count: usize,
    /// Fixtures missing an expectation bucket.
    pub missing_expectation_kind_fixture_ids: Vec<String>,
    /// Fixtures whose assigned bucket differs from the current criteria.
    pub criteria_mismatch_fixture_ids: Vec<String>,
    /// Whether the bucket union equals the imported fixture count.
    pub all_fixtures_have_one_expectation_kind: bool,
    /// Whether every assigned bucket matches the current classification criteria.
    pub all_expectation_kinds_match_criteria: bool,
}

/// Imported sass-spec scale and smoke-floor summary.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SassSpecImportScaleReportV0 {
    /// Schema version.
    pub schema_version: &'static str,
    /// Product surface name.
    pub product: &'static str,
    /// Pinned sass-spec source identifier.
    pub source_pin: String,
    /// Checked-in HRX archive root.
    pub source_archive_root: String,
    /// HRX archives found by scanning the source archive root.
    pub source_archive_count: usize,
    /// Whether the source archive root was readable.
    pub source_archive_scan_succeeded: bool,
    /// HRX archives found in the pinned upstream sparse checkout.
    pub upstream_archive_count: usize,
    /// Whether the pinned upstream scale artifact matched the imported manifest.
    pub upstream_scale_artifact_matches_manifest: bool,
    /// Imported sass-spec fixture count.
    pub imported_fixture_count: usize,
    /// Imported sass-spec chunk count.
    pub imported_chunk_count: usize,
    /// Seed sass-spec fixture count used as the smoke floor.
    pub seed_fixture_count: usize,
    /// Per-push smoke fixture count.
    pub per_push_smoke_fixture_count: usize,
    /// Minimum fixture count required by the seed smoke floor.
    pub per_push_smoke_floor_fixture_count: usize,
    /// Static equality bucket count.
    pub static_must_match_count: usize,
    /// Abstract-value membership bucket count.
    pub expected_sound_bail_count: usize,
    /// Parser-recovery bucket count.
    pub parser_recovery_count: usize,
    /// Explicit exclusion bucket count.
    pub out_of_scope_count: usize,
    /// Whether chunk fixture counts match the manifest.
    pub all_imported_counts_match_manifest: bool,
    /// Whether chunk hashes match the manifest.
    pub all_chunk_hashes_match_manifest: bool,
    /// Whether sparse-path fixture counts match the manifest.
    pub all_sparse_path_counts_match_manifest: bool,
    /// Whether scanned HRX archive paths stay under the declared sparse paths.
    pub all_source_archives_under_sparse_paths: bool,
    /// Whether the scanned HRX archive count matches imported fixtures.
    pub all_source_archive_count_matches_imported_fixtures: bool,
    /// Whether the upstream sparse checkout is larger than the smoke corpus.
    pub all_upstream_archive_count_exceeds_imported_fixtures: bool,
    /// Whether the upstream sparse checkout is larger than the checked-in source sample.
    pub all_upstream_archive_count_exceeds_source_archives: bool,
    /// Whether the smoke fixture count keeps the seed floor.
    pub all_smoke_floor_holds: bool,
    /// Whether the complete import-scale gate holds.
    pub all_import_scale_checks_hold: bool,
    /// Per-chunk import count and hash records.
    pub chunks: Vec<SassSpecImportScaleChunkReportV0>,
}

/// One imported sass-spec chunk count and hash record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SassSpecImportScaleChunkReportV0 {
    /// Chunk id.
    pub chunk_id: String,
    /// Chunk path relative to the imported manifest.
    pub path: String,
    /// Fixture count recorded in the manifest.
    pub manifest_fixture_count: usize,
    /// Fixture count observed by parsing the chunk.
    pub actual_fixture_count: usize,
    /// Chunk hash recorded in the manifest.
    pub manifest_sha256: String,
    /// Chunk hash observed from the committed chunk source.
    pub actual_sha256: String,
    /// Whether the fixture count matches.
    pub fixture_count_matches: bool,
    /// Whether the chunk hash matches.
    pub sha256_matches: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ImportedSassSpecChunkV0 {
    schema_version: String,
    product: String,
    chunk_id: String,
    source_pin: String,
    fixtures: Vec<ImportedSassSpecFixtureV0>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SassSpecUpstreamScaleArtifactV0 {
    schema_version: String,
    product: String,
    source: SassSpecUpstreamScaleSourceV0,
    archive_extension: String,
    archive_count: usize,
    sparse_path_archive_counts: Vec<SassSpecUpstreamSparsePathArchiveCountV0>,
    imported_source_archive_count: usize,
    imported_source_archive_byte_match_count: usize,
    all_imported_source_archives_match_upstream: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SassSpecUpstreamScaleSourceV0 {
    repository: String,
    pin: String,
    sparse_paths: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SassSpecUpstreamSparsePathArchiveCountV0 {
    sparse_path: String,
    archive_count: usize,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ImportedSassSpecFixtureV0 {
    id: String,
    upstream_path: String,
    dialect: String,
    source: String,
    expectation_kind:
        Option<external_corpus_envelope_idl_generated::ExternalCorpusExpectationKindV1Json>,
    expected_css: Option<String>,
    expected_error: Option<String>,
    expected_warning: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FixtureListChunkV0 {
    fixtures: Vec<serde_json::Value>,
}

/// Imported sass-spec sound-bail membership summary.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SassSpecSoundBailMembershipReportV0 {
    /// Schema version.
    pub schema_version: &'static str,
    /// Product surface name.
    pub product: &'static str,
    /// Expected sound-bail fixture count.
    pub case_count: usize,
    /// Cases with a matching dart-sass oracle record and abstract value.
    pub checked_case_count: usize,
    /// Cases whose abstract value is not top.
    pub non_top_case_count: usize,
    /// Cases whose abstract value is raw and tight against the concrete value.
    pub raw_tightness_case_count: usize,
    /// Whether concrete values are included by the omena abstract values.
    pub all_concrete_values_in_abstract_values: bool,
    /// Whether weakening exact values preserves membership.
    pub weakening_preserves_membership: bool,
    /// Whether exact-value membership rejects a different concrete value.
    pub exact_tightness_holds: bool,
    /// Whether the complete membership gate holds.
    pub all_membership_checks_hold: bool,
    /// Per-case membership records.
    pub records: Vec<SassSpecSoundBailMembershipCaseReportV0>,
}

/// One imported sass-spec sound-bail membership record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SassSpecSoundBailMembershipCaseReportV0 {
    /// Fixture id.
    pub fixture_id: String,
    /// CSS declaration property from the dart-sass oracle.
    pub property: String,
    /// CSS declaration value from the dart-sass oracle.
    pub concrete_value: String,
    /// Omena abstract value kind.
    pub abstract_value_kind: String,
    /// Omena static value reason.
    pub reason: String,
    /// Whether the concrete value is included by the omena abstract value.
    pub concrete_in_abstract_value: bool,
    /// Whether exact-to-finite-to-top weakening preserves membership.
    pub weakening_preserves_membership: bool,
    /// Whether exact membership is tight for a distinct value.
    pub exact_tightness_holds: bool,
    /// Whether this case exercises a raw-value membership check.
    pub raw_tightness_case: bool,
}

/// Imported sass-spec static equality summary.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SassSpecStaticMustMatchReportV0 {
    /// Schema version.
    pub schema_version: &'static str,
    /// Product surface name.
    pub product: &'static str,
    /// Static equality fixture count.
    pub case_count: usize,
    /// Static equality fixtures checked against dart-sass declaration values.
    pub checked_case_count: usize,
    /// Dart-sass declaration value count.
    pub declaration_value_count: usize,
    /// Dart-sass declaration values matched by omena static resolution.
    pub matched_declaration_value_count: usize,
    /// Whether every static fixture has matching omena static values.
    pub all_static_values_match_oracle: bool,
    /// Whether the complete static equality gate holds.
    pub all_static_match_checks_hold: bool,
    /// Per-declaration static equality records.
    pub records: Vec<SassSpecStaticMustMatchCaseReportV0>,
}

/// One imported sass-spec static equality declaration record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SassSpecStaticMustMatchCaseReportV0 {
    /// Fixture id.
    pub fixture_id: String,
    /// CSS declaration property from the dart-sass oracle.
    pub property: String,
    /// CSS declaration value from the dart-sass oracle.
    pub concrete_value: String,
    /// Whether an omena resolved value matched the dart-sass concrete value.
    pub matched_by_omena_resolution: bool,
    /// Omena static values available for this fixture.
    pub omena_rendered_values: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SassSpecOracleCaptureV0 {
    records: Vec<SassSpecOracleRecordV0>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SassSpecOracleRecordV0 {
    fixture_id: String,
    compiled: bool,
    declaration_value_pairs: Option<Vec<SassSpecDeclarationValuePairV0>>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SassSpecDeclarationValuePairV0 {
    property: String,
    value: String,
}

/// Imported sass-spec expectation bucket ledger comparison.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SassSpecExpectationBucketLedgerReportV0 {
    /// Schema version.
    pub schema_version: &'static str,
    /// Product surface name.
    pub product: &'static str,
    /// Current imported fixture count.
    pub fixture_count: usize,
    /// Current static equality bucket count.
    pub current_static_must_match_count: usize,
    /// Current sound-bail bucket count.
    pub current_expected_sound_bail_count: usize,
    /// Current parser-recovery bucket count.
    pub current_parser_recovery_count: usize,
    /// Current explicit exclusion bucket count.
    pub current_out_of_scope_count: usize,
    /// Ledger static equality bucket count.
    pub ledger_static_must_match_count: usize,
    /// Ledger sound-bail bucket count.
    pub ledger_expected_sound_bail_count: usize,
    /// Ledger parser-recovery bucket count.
    pub ledger_parser_recovery_count: usize,
    /// Ledger explicit exclusion bucket count.
    pub ledger_out_of_scope_count: usize,
    /// Whether current bucket totals match the committed ledger.
    pub all_bucket_totals_match_ledger: bool,
    /// Whether current fixture assignments match the committed ledger.
    pub all_fixture_assignments_match_ledger: bool,
    /// Whether reclassification entries carry review metadata.
    pub all_reclassification_entries_have_rationale: bool,
    /// Whether the committed ledger metadata matches this corpus.
    pub ledger_metadata_valid: bool,
    /// Current fixtures missing from the ledger.
    pub missing_ledger_fixture_ids: Vec<String>,
    /// Ledger fixtures absent from the current chunk.
    pub stale_ledger_fixture_ids: Vec<String>,
    /// Fixtures whose current expectation differs from the ledger.
    pub assignment_mismatch_fixture_ids: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct SassSpecExpectationBucketLedgerTomlV0 {
    schema_version: String,
    corpus_chunk: String,
    static_must_match_count: usize,
    expected_sound_bail_count: usize,
    parser_recovery_count: usize,
    out_of_scope_count: usize,
    fixture: Vec<SassSpecExpectationBucketLedgerFixtureTomlV0>,
    #[serde(default)]
    reclassification: Vec<SassSpecExpectationBucketReclassificationTomlV0>,
}

#[derive(Debug, Deserialize)]
struct SassSpecExpectationBucketLedgerFixtureTomlV0 {
    id: String,
    expectation_kind: external_corpus_envelope_idl_generated::ExternalCorpusExpectationKindV1Json,
}

#[derive(Debug, Deserialize)]
struct SassSpecExpectationBucketReclassificationTomlV0 {
    fixture_id: String,
    from: external_corpus_envelope_idl_generated::ExternalCorpusExpectationKindV1Json,
    to: external_corpus_envelope_idl_generated::ExternalCorpusExpectationKindV1Json,
    reason: String,
    since: String,
    review_after: String,
}

/// Imported sass-spec bail-site ledger comparison.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SassSpecBailSiteLedgerReportV0 {
    /// Schema version.
    pub schema_version: &'static str,
    /// Product surface name.
    pub product: &'static str,
    /// Current semantic bail-site count from source.
    pub semantic_site_count: usize,
    /// Ledger semantic bail-site count.
    pub ledger_semantic_site_count: usize,
    /// Raw bail-pattern hit count, including excluded matches.
    pub raw_pattern_hit_count: usize,
    /// Raw bail-pattern hits excluded from the semantic ledger.
    pub non_semantic_pattern_hit_count: usize,
    /// Sites with at least one linked imported fixture.
    pub linked_site_count: usize,
    /// Sites with a named coverage gap.
    pub named_gap_site_count: usize,
    /// Distinct imported fixtures linked by the ledger.
    pub linked_case_count: usize,
    /// Whether the ledger metadata matches the current source census.
    pub ledger_metadata_valid: bool,
    /// Whether source census and ledger site keys match.
    pub all_semantic_sites_match_ledger: bool,
    /// Whether every ledger entry links a case or names a gap.
    pub all_sites_linked_or_named_gap: bool,
    /// Whether every linked case reason matches its site reason.
    pub all_linked_cases_match_reason_class: bool,
    /// Whether every linked case is an imported sound-bail fixture.
    pub all_linked_cases_are_imported_sound_bail_cases: bool,
    /// Whether the complete bail-site ledger gate holds.
    pub all_bail_site_ledger_checks_hold: bool,
    /// Current source sites missing from the ledger.
    pub missing_ledger_site_keys: Vec<String>,
    /// Ledger sites absent from the current source census.
    pub stale_ledger_site_keys: Vec<String>,
    /// Ledger sites with neither a link nor a named gap.
    pub gapless_site_keys: Vec<String>,
    /// Linked fixtures that are missing from imported sound-bail cases.
    pub unknown_link_fixture_ids: Vec<String>,
    /// Links whose case reason differs from the site reason.
    pub reason_mismatch_link_keys: Vec<String>,
    /// Per-site ledger rows joined with current source state.
    pub records: Vec<SassSpecBailSiteLedgerRecordReportV0>,
}

/// One imported sass-spec bail-site ledger record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SassSpecBailSiteLedgerRecordReportV0 {
    /// Source file path.
    pub file: String,
    /// Ordinal within this file and reason class.
    pub ordinal: usize,
    /// Static stylesheet reason class.
    pub reason: String,
    /// Current line number if the site is still present.
    pub current_line: Option<usize>,
    /// Human line hint recorded in the ledger.
    pub ledger_line_hint: usize,
    /// Whether current source still contains this site.
    pub present_in_current_sources: bool,
    /// Imported fixtures linked to this site.
    pub linked_fixture_ids: Vec<String>,
    /// Named coverage gap, when no imported fixture is linked.
    pub gap: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct SassSpecBailSiteCensusRecordV0 {
    file: String,
    ordinal: usize,
    reason: String,
    line: usize,
}

#[derive(Debug, Deserialize)]
struct SassSpecBailSiteLedgerTomlV0 {
    schema_version: String,
    source_root: String,
    semantic_site_count: usize,
    raw_pattern_hit_count: usize,
    non_semantic_pattern_hit_count: usize,
    site: Vec<SassSpecBailSiteLedgerSiteTomlV0>,
}

#[derive(Debug, Deserialize)]
struct SassSpecBailSiteLedgerSiteTomlV0 {
    file: String,
    ordinal: usize,
    reason: String,
    line_hint: usize,
    #[serde(default)]
    linked_fixture_ids: Vec<String>,
    gap: Option<String>,
}

struct SassSpecBailSiteSourceFileV0 {
    file: &'static str,
    source: &'static str,
    include_in_semantic_census: bool,
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
const SASS_SPEC_SEED_MANIFEST_SOURCE: &str = include_str!("../sass-spec-corpus/manifest.json");
const SASS_SPEC_SEED_CHUNK_SOURCES: &[&str] =
    &[include_str!("../sass-spec-corpus/language-core.json")];
#[cfg(test)]
const SASS_SPEC_SEED_KNOWN_FAILURE_POLICY_SOURCE: &str =
    include_str!("../known-failures/sass-spec-seed-policy.toml");
const SASS_SPEC_IMPORTED_MANIFEST_SOURCE: &str =
    include_str!("../sass-spec-corpus/imported-smoke-manifest.json");
const SASS_SPEC_IMPORTED_CHUNK_SOURCE: &str =
    include_str!("../sass-spec-corpus/imported-smoke.json");
const SASS_SPEC_IMPORTED_ORACLE_CAPTURE_SOURCE: &str =
    include_str!("../sass-spec-corpus/imported-smoke-oracle.json");
const SASS_SPEC_UPSTREAM_SCALE_SOURCE: &str =
    include_str!("../sass-spec-corpus/upstream-scale.json");
const SASS_SPEC_EXPECTATION_BUCKET_LEDGER_SOURCE: &str =
    include_str!("../sass-spec-corpus/expectation-bucket-ledger.toml");
const SASS_SPEC_BAIL_SITE_LEDGER_SOURCE: &str =
    include_str!("../sass-spec-corpus/bail-site-ledger.toml");
const SASS_SPEC_BAIL_SITE_SOURCE_FILES: &[SassSpecBailSiteSourceFileV0] = &[
    SassSpecBailSiteSourceFileV0 {
        file: "rust/crates/omena-scss-eval/src/static_stylesheet/scss_loop_returns.rs",
        source: include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../omena-scss-eval/src/static_stylesheet/scss_loop_returns.rs"
        )),
        include_in_semantic_census: true,
    },
    SassSpecBailSiteSourceFileV0 {
        file: "rust/crates/omena-scss-eval/src/static_stylesheet/scss_variables.rs",
        source: include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../omena-scss-eval/src/static_stylesheet/scss_variables.rs"
        )),
        include_in_semantic_census: true,
    },
    SassSpecBailSiteSourceFileV0 {
        file: "rust/crates/omena-scss-eval/src/static_stylesheet/less_variables.rs",
        source: include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../omena-scss-eval/src/static_stylesheet/less_variables.rs"
        )),
        include_in_semantic_census: true,
    },
    SassSpecBailSiteSourceFileV0 {
        file: "rust/crates/omena-scss-eval/src/static_stylesheet/value_resolution_model.rs",
        source: include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../omena-scss-eval/src/static_stylesheet/value_resolution_model.rs"
        )),
        include_in_semantic_census: true,
    },
    SassSpecBailSiteSourceFileV0 {
        file: "rust/crates/omena-scss-eval/src/static_stylesheet.rs",
        source: include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../omena-scss-eval/src/static_stylesheet.rs"
        )),
        include_in_semantic_census: true,
    },
    SassSpecBailSiteSourceFileV0 {
        file: "rust/crates/omena-scss-eval/src/native_css.rs",
        source: include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../omena-scss-eval/src/native_css.rs"
        )),
        include_in_semantic_census: false,
    },
];
#[cfg(test)]
const LESS_SEED_MANIFEST_SOURCE: &str = include_str!("../less-corpus/manifest.json");
#[cfg(test)]
const LESS_SEED_CHUNK_SOURCES: &[&str] = &[include_str!("../less-corpus/language-core.json")];
#[cfg(test)]
const LESS_SEED_KNOWN_FAILURE_POLICY_SOURCE: &str =
    include_str!("../known-failures/less-seed-policy.toml");
#[cfg(test)]
const SASS_DIFFERENTIAL_MANIFEST_SOURCE: &str = include_str!("../sass-differential/manifest.json");
#[cfg(test)]
const STATIC_STYLESHEET_EXTERNAL_DIFFERENTIAL_MANIFEST_SOURCE: &str =
    include_str!("../static-stylesheet-external-differential/manifest.json");

#[cfg(test)]
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DialectSeedChunkV0 {
    fixtures: Vec<DialectSeedFixtureV0>,
}

#[cfg(test)]
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DialectSeedFixtureV0 {
    id: String,
    source: String,
    subtest: String,
    #[serde(default)]
    dialect: String,
    #[serde(default)]
    expected_bogus_kinds: Vec<String>,
    #[serde(default)]
    expected_error_codes: Vec<String>,
}

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

const PARSER_FACT_AUTHORITY_CATEGORY_COUNT: usize = 16;

const PARSER_FACT_AUTHORITY_FIXTURES: &[ParserDifferentialFixture] = &[
    ParserDifferentialFixture {
        label: "css-modules-values-and-icss",
        file_path: "/values.module.css",
        source: r#"
@value brand: red;
@value remote as local from "./tokens.module.css";
:import("./theme.module.css") { themeCard: card; }
:export { exported: brand local; }
.card { composes: local from "./mixins.module.css"; color: brand; }
"#,
        dialect: DiffDialect::Css,
    },
    ParserDifferentialFixture {
        label: "scss-modules-symbols-and-extends",
        file_path: "/symbols.module.scss",
        source: r#"
@use "./tokens" as tokens;
@forward "./theme" show tone;
@import "./legacy" screen;
@mixin raised($depth) { box-shadow: 0 0 $depth black; }
@function double($value) { @return $value * 2; }
%surface { color: tokens.$brand; }
@keyframes spin { from { opacity: 0; } to { opacity: 1; } }
.card { @include raised($gap); @extend %surface !optional; animation: spin 1s; }
"#,
        dialect: DiffDialect::Scss,
    },
    ParserDifferentialFixture {
        label: "less-variables-and-nested-selectors",
        file_path: "/nested.module.less",
        source: r#"
@color: red;
.card { --tone: @color; &__icon { color: var(--tone); } }
"#,
        dialect: DiffDialect::Less,
    },
    ParserDifferentialFixture {
        label: "scss-interpolation-token-contexts",
        file_path: "/interpolation.module.scss",
        source: r#"
$tone: brand;
@mixin paint($value) { color: $value; }
@keyframes spin { to { opacity: 1; } }
.button-#{$variant} {
  @include paint($tone);
  animation: #{$tone}-spin 1s;
}
.button {
  animation: #{$tone} spin 1s;
}
"#,
        dialect: DiffDialect::Scss,
    },
];

const PARSER_CST_CONTEXT_RAW_SCAN_FIXTURES: &[ParserCstContextRawScanFixture] = &[
    ParserCstContextRawScanFixture {
        label: "comment-embedded-context-tokens",
        file_path: "/comment.module.scss",
        source: r#"
/* @layer fakeComment; @layer fakeCommentBlock { .fakeComment { color: red; } } */
@layer reset;
@layer components {
  .card { color: red; }
}
"#,
        expected_statement_layers: &["reset"],
        expected_block_layers: &["components"],
        expected_layer_selector_memberships: &["card"],
        rejected_names: &["fakeComment", "fakeCommentBlock"],
    },
    ParserCstContextRawScanFixture {
        label: "string-embedded-context-tokens",
        file_path: "/string.module.scss",
        source: r#"
.noise::before {
  content: "@layer fakeString; @layer fakeStringBlock { .fakeString {";
}
@layer reset;
@layer components {
  .card { content: "{"; color: red; }
}
"#,
        expected_statement_layers: &["reset"],
        expected_block_layers: &["components"],
        expected_layer_selector_memberships: &["card"],
        rejected_names: &["fakeString", "fakeStringBlock"],
    },
    ParserCstContextRawScanFixture {
        label: "interpolation-embedded-context-tokens",
        file_path: "/interpolation.module.scss",
        source: r#"
.noise-#{"@layer fakeInterpolation; @layer fakeInterpolationBlock { .fakeInterpolation {"} {
  color: red;
}
@layer reset;
@layer components {
  .card { color: red; }
}
"#,
        expected_statement_layers: &["reset"],
        expected_block_layers: &["components"],
        expected_layer_selector_memberships: &["card"],
        rejected_names: &["fakeInterpolation", "fakeInterpolationBlock"],
    },
];

const SELECTOR_CONTEXT_SOUNDNESS_FIXTURES: &[SelectorContextSoundnessFixtureV0] = &[
    SelectorContextSoundnessFixtureV0 {
        fixture_id: "prefix-reject-dot-foo",
        declaration_selector: ".foo",
        reference_selector: ".foobar",
        expected_verdict: SelectorMatchVerdict::No,
        baseline_positive: true,
        removed_spurious_positive: true,
        unmodeled: false,
    },
    SelectorContextSoundnessFixtureV0 {
        fixture_id: "prefix-reject-bem-btn",
        declaration_selector: ".btn",
        reference_selector: ".btn-primary",
        expected_verdict: SelectorMatchVerdict::No,
        baseline_positive: true,
        removed_spurious_positive: true,
        unmodeled: false,
    },
    SelectorContextSoundnessFixtureV0 {
        fixture_id: "descendant-preserve-theme",
        declaration_selector: ".theme",
        reference_selector: ".theme .button",
        expected_verdict: SelectorMatchVerdict::Yes,
        baseline_positive: true,
        removed_spurious_positive: false,
        unmodeled: false,
    },
    SelectorContextSoundnessFixtureV0 {
        fixture_id: "child-combinator-preserve",
        declaration_selector: ".a",
        reference_selector: ".a > .b",
        expected_verdict: SelectorMatchVerdict::Yes,
        baseline_positive: true,
        removed_spurious_positive: false,
        unmodeled: false,
    },
    SelectorContextSoundnessFixtureV0 {
        fixture_id: "sibling-combinator-preserve",
        declaration_selector: ".a",
        reference_selector: ".a ~ .b",
        expected_verdict: SelectorMatchVerdict::Yes,
        baseline_positive: true,
        removed_spurious_positive: false,
        unmodeled: false,
    },
    SelectorContextSoundnessFixtureV0 {
        fixture_id: "adjacent-combinator-preserve",
        declaration_selector: ".a",
        reference_selector: ".a + .b",
        expected_verdict: SelectorMatchVerdict::Yes,
        baseline_positive: true,
        removed_spurious_positive: false,
        unmodeled: false,
    },
    SelectorContextSoundnessFixtureV0 {
        fixture_id: "unmodeled-declaration-stays-maybe",
        declaration_selector: ".card:unknown(.x)",
        reference_selector: ".button",
        expected_verdict: SelectorMatchVerdict::Maybe,
        baseline_positive: false,
        removed_spurious_positive: false,
        unmodeled: true,
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

pub fn summarize_parser_cst_fact_authority_equivalence_v0() -> ParserCstFactAuthorityReportV0 {
    let fixtures = parser_fact_authority_fixtures();
    let snapshot = parser_cst_fact_authority_snapshot();
    let comparisons = fixtures
        .iter()
        .copied()
        .flat_map(|fixture| parser_cst_fact_authority_reports_for_fixture(fixture, &snapshot))
        .collect::<Vec<_>>();
    let metamorphic_relations = parser_cst_fact_authority_metamorphic_reports(&fixtures);
    let all_value_sets_match = comparisons.iter().all(|report| report.values_match);
    let all_span_sets_match = comparisons.iter().all(|report| report.spans_match);
    let all_metamorphic_relations_hold = metamorphic_relations.iter().all(|report| report.holds);

    ParserCstFactAuthorityReportV0 {
        schema_version: "0",
        product: "omena-diff-test.parser-cst-fact-authority-equivalence",
        fixture_count: fixtures.len(),
        category_count: PARSER_FACT_AUTHORITY_CATEGORY_COUNT,
        comparisons,
        all_value_sets_match,
        all_span_sets_match,
        metamorphic_relation_count: metamorphic_relations.len(),
        metamorphic_relations,
        all_metamorphic_relations_hold,
    }
}

pub fn summarize_transform_ir_identity_round_trip_equivalence_v0()
-> TransformIrIdentityRoundTripEquivalenceReportV0 {
    let reports = transform_ir_identity_round_trip_reports();
    let all_fields_match = reports.iter().all(|report| report.all_fields_match);

    TransformIrIdentityRoundTripEquivalenceReportV0 {
        schema_version: "0",
        product: "omena-diff-test.transform-ir-identity-round-trip-equivalence",
        fixture_count: reports.len(),
        reports,
        all_fields_match,
        closed_gates: vec!["transformIrIdentityRoundTrip"],
    }
}

fn transform_ir_identity_round_trip_reports() -> Vec<TransformIrIdentityRoundTripFixtureReportV0> {
    let mut reports = parser_fact_authority_fixtures()
        .iter()
        .copied()
        .map(transform_ir_identity_round_trip_report_for_parser_fixture)
        .collect::<Vec<_>>();
    reports.extend(style_corpus().into_iter().map(|sample| {
        transform_ir_identity_round_trip_report_for_source(
            sample.name,
            sample.dialect,
            sample.source.as_str(),
        )
    }));
    reports.extend(bundler_productization_corpus().into_iter().map(|sample| {
        transform_ir_identity_round_trip_report_for_source(
            sample.name,
            sample.dialect,
            sample.source.as_str(),
        )
    }));
    reports
}

fn parser_fact_authority_fixtures() -> Vec<ParserDifferentialFixture> {
    PARSER_LEGACY_SEED_FIXTURES
        .iter()
        .chain(PARSER_FACT_AUTHORITY_FIXTURES)
        .copied()
        .collect()
}

fn transform_ir_identity_round_trip_report_for_parser_fixture(
    fixture: ParserDifferentialFixture,
) -> TransformIrIdentityRoundTripFixtureReportV0 {
    transform_ir_identity_round_trip_report_for_source(
        fixture.label,
        fixture.dialect.as_omena_dialect(),
        fixture.source,
    )
}

fn transform_ir_identity_round_trip_report_for_source(
    label: &'static str,
    dialect: StyleDialect,
    source: &str,
) -> TransformIrIdentityRoundTripFixtureReportV0 {
    let summary = summarize_transform_ir_identity_round_trip(source, dialect, label);
    let (node_count, fields) = match summary {
        Ok(summary) => (
            summary.node_count,
            vec![
                field_report("sourceBytes", [source.to_string()], [summary.printed_css]),
                field_report(
                    "allNodesOriginal",
                    ["true".to_string()],
                    [summary.all_nodes_original.to_string()],
                ),
                field_report(
                    "synthesizedNodeCount",
                    ["0".to_string()],
                    [summary.synthesized_node_count.to_string()],
                ),
                field_report(
                    "byteIdentical",
                    ["true".to_string()],
                    [summary.byte_identical.to_string()],
                ),
            ],
        ),
        Err(error) => (
            0,
            vec![field_report(
                "printError",
                ["none".to_string()],
                [format!("{error:?}")],
            )],
        ),
    };
    let all_fields_match = fields.iter().all(|field| field.matches);

    TransformIrIdentityRoundTripFixtureReportV0 {
        schema_version: "0",
        product: "omena-diff-test.transform-ir-identity-round-trip-fixture",
        label,
        dialect: style_dialect_label(dialect),
        node_count,
        fields,
        all_fields_match,
    }
}

fn parser_cst_fact_authority_reports_for_fixture(
    fixture: ParserDifferentialFixture,
    snapshot: &ParserCstFactAuthoritySnapshotV0,
) -> Vec<ParserCstFactAuthorityCategoryReportV0> {
    let dialect = fixture.dialect.as_omena_dialect();
    let parsed = parse(fixture.source, dialect);
    let cst = facts_from_cst(fixture.source, &parsed);
    style_fact_category_reports(fixture.label, snapshot, &cst)
}

fn parser_cst_fact_authority_snapshot() -> ParserCstFactAuthoritySnapshotV0 {
    match serde_json::from_str(PARSER_CST_FACT_AUTHORITY_SNAPSHOT_SOURCE) {
        Ok(snapshot) => snapshot,
        Err(error) => ParserCstFactAuthoritySnapshotV0 {
            fixture_count: 0,
            category_count: 0,
            comparisons: vec![ParserCstFactAuthoritySnapshotCategoryV0 {
                fixture: "invalid-snapshot".to_string(),
                category: "parse-error".to_string(),
                legacy_values: vec![error.to_string()],
                legacy_spans: Vec::new(),
            }],
        },
    }
}

fn parser_cst_fact_authority_metamorphic_reports(
    fixtures: &[ParserDifferentialFixture],
) -> Vec<ParserCstFactAuthorityMetamorphicReportV0> {
    fixtures
        .iter()
        .flat_map(|fixture| {
            let comment_source = format!("/* inserted parser comment */\n{}", fixture.source);
            let whitespace_source = format!("\n{}\n", fixture.source);
            [
                parser_cst_fact_authority_metamorphic_report(
                    "comment-insertion",
                    *fixture,
                    comment_source.as_str(),
                ),
                parser_cst_fact_authority_metamorphic_report(
                    "whitespace-insertion",
                    *fixture,
                    whitespace_source.as_str(),
                ),
            ]
        })
        .collect()
}

fn parser_cst_fact_authority_metamorphic_report(
    relation: &'static str,
    fixture: ParserDifferentialFixture,
    after_source: &str,
) -> ParserCstFactAuthorityMetamorphicReportV0 {
    let dialect = fixture.dialect.as_omena_dialect();
    let before_values = parser_cst_fact_value_signature(fixture.source, dialect);
    let after_values = parser_cst_fact_value_signature(after_source, dialect);
    let holds = before_values == after_values;
    ParserCstFactAuthorityMetamorphicReportV0 {
        relation,
        fixture: fixture.label,
        before_values,
        after_values,
        holds,
    }
}

fn parser_cst_fact_value_signature(source: &str, dialect: StyleDialect) -> Vec<String> {
    let parsed = parse(source, dialect);
    let facts = facts_from_cst(source, &parsed);
    sorted_unique(style_fact_category_value_sets(&facts).into_iter().flat_map(
        |(category, values)| {
            values
                .into_iter()
                .map(move |value| format!("{category}:{value}"))
        },
    ))
}

fn style_fact_category_reports(
    fixture: &'static str,
    snapshot: &ParserCstFactAuthoritySnapshotV0,
    cst: &ParsedStyleFacts,
) -> Vec<ParserCstFactAuthorityCategoryReportV0> {
    let cst_values = style_fact_category_value_sets(cst);
    let cst_spans = style_fact_category_span_sets(cst);
    assert_eq!(cst_values.len(), PARSER_FACT_AUTHORITY_CATEGORY_COUNT);
    assert_eq!(cst_spans.len(), PARSER_FACT_AUTHORITY_CATEGORY_COUNT);

    cst_values
        .into_iter()
        .zip(cst_spans)
        .map(|((category, cst_values), (cst_span_category, cst_spans))| {
            assert_eq!(category, cst_span_category);
            let (legacy_values, legacy_spans) =
                match parser_cst_fact_authority_snapshot_category(snapshot, fixture, category) {
                    Some(expected) => (
                        expected.legacy_values.clone(),
                        expected.legacy_spans.clone(),
                    ),
                    None => (
                        vec![format!("missing snapshot row: {fixture}/{category}")],
                        Vec::new(),
                    ),
                };
            let values_match = legacy_values == cst_values;
            let spans_match = legacy_spans == cst_spans;
            ParserCstFactAuthorityCategoryReportV0 {
                fixture,
                category,
                legacy_values,
                cst_values,
                legacy_spans,
                cst_spans,
                values_match,
                spans_match,
            }
        })
        .collect()
}

fn parser_cst_fact_authority_snapshot_category<'snapshot>(
    snapshot: &'snapshot ParserCstFactAuthoritySnapshotV0,
    fixture: &str,
    category: &str,
) -> Option<&'snapshot ParserCstFactAuthoritySnapshotCategoryV0> {
    snapshot
        .comparisons
        .iter()
        .find(|entry| entry.fixture == fixture && entry.category == category)
}

pub fn summarize_parser_cst_context_raw_scan_divergence_v0()
-> ParserCstContextRawScanDivergenceReportV0 {
    let reports = PARSER_CST_CONTEXT_RAW_SCAN_FIXTURES
        .iter()
        .copied()
        .map(parser_cst_context_raw_scan_fixture_report)
        .collect::<Vec<_>>();
    let all_fixtures_match = reports.iter().all(|report| report.matches);
    ParserCstContextRawScanDivergenceReportV0 {
        schema_version: "0",
        product: "omena-diff-test.parser-cst-context-raw-scan-divergence",
        fixture_count: reports.len(),
        reports,
        all_fixtures_match,
    }
}

pub fn summarize_selector_context_soundness_v0() -> SelectorContextSoundnessReportV0 {
    let reports = SELECTOR_CONTEXT_SOUNDNESS_FIXTURES
        .iter()
        .copied()
        .map(selector_context_soundness_fixture_report)
        .collect::<Vec<_>>();
    let all_expected_verdicts_match = reports
        .iter()
        .all(|report| report.expected_verdict == report.actual_verdict);
    let all_unmodeled_fixtures_stay_maybe = reports
        .iter()
        .filter(|report| report.unmodeled)
        .all(|report| report.actual_verdict == SelectorMatchVerdict::Maybe);
    let baseline_positive_ids = reports
        .iter()
        .filter(|report| report.baseline_positive)
        .map(|report| report.fixture_id)
        .collect::<BTreeSet<_>>();
    let removed_spurious_positive_ids = reports
        .iter()
        .filter(|report| report.removed_spurious_positive)
        .map(|report| report.fixture_id)
        .collect::<BTreeSet<_>>();
    let expected_positive_ids = baseline_positive_ids
        .difference(&removed_spurious_positive_ids)
        .copied()
        .collect::<BTreeSet<_>>();
    let actual_positive_ids = reports
        .iter()
        .filter(|report| report.actual_verdict == SelectorMatchVerdict::Yes)
        .map(|report| report.fixture_id)
        .collect::<BTreeSet<_>>();
    let positive_preservation_matches = expected_positive_ids == actual_positive_ids;

    SelectorContextSoundnessReportV0 {
        schema_version: "0",
        product: "omena-diff-test.selector-context-soundness",
        fixture_count: reports.len(),
        reports,
        all_expected_verdicts_match,
        all_unmodeled_fixtures_stay_maybe,
        baseline_positive_count: baseline_positive_ids.len(),
        removed_spurious_positive_count: removed_spurious_positive_ids.len(),
        actual_positive_count: actual_positive_ids.len(),
        positive_preservation_matches,
    }
}

fn selector_context_soundness_fixture_report(
    fixture: SelectorContextSoundnessFixtureV0,
) -> SelectorContextSoundnessFixtureReportV0 {
    let witness = selector_context_witness(
        &[fixture.declaration_selector.to_string()],
        &[fixture.reference_selector.to_string()],
    );
    let expected_match = fixture.expected_verdict != SelectorMatchVerdict::No;
    let actual_match = witness.verdict != SelectorMatchVerdict::No;

    SelectorContextSoundnessFixtureReportV0 {
        fixture_id: fixture.fixture_id,
        declaration_selector: fixture.declaration_selector,
        reference_selector: fixture.reference_selector,
        expected_verdict: fixture.expected_verdict,
        actual_verdict: witness.verdict,
        expected_match,
        actual_match,
        baseline_positive: fixture.baseline_positive,
        removed_spurious_positive: fixture.removed_spurious_positive,
        unmodeled: fixture.unmodeled,
    }
}

fn parser_cst_context_raw_scan_fixture_report(
    fixture: ParserCstContextRawScanFixture,
) -> ParserCstContextRawScanFixtureReportV0 {
    let summary = summarize_omena_parser_style_semantic_boundary_from_source(
        fixture.file_path,
        fixture.source,
    );
    let context_index = summary.semantic_facts.context_index;
    let statement_layers = context_index
        .layer_index
        .statement_layers
        .iter()
        .map(|layer| layer.name.clone())
        .collect::<Vec<_>>();
    let block_layers = context_index
        .layer_index
        .block_layers
        .iter()
        .filter_map(|block| block.name.clone())
        .collect::<Vec<_>>();
    let layer_selector_memberships = sorted_unique(
        context_index
            .layer_index
            .selector_memberships
            .iter()
            .map(|membership| membership.selector_name.clone()),
    );
    let observed_names = statement_layers
        .iter()
        .chain(block_layers.iter())
        .chain(layer_selector_memberships.iter())
        .collect::<Vec<_>>();
    let rejected_names_absent = fixture.rejected_names.iter().all(|rejected| {
        observed_names
            .iter()
            .all(|observed| observed.as_str() != *rejected)
    });
    let expected_statement_layers = fixture.expected_statement_layers.to_vec();
    let expected_block_layers = fixture.expected_block_layers.to_vec();
    let expected_layer_selector_memberships = fixture.expected_layer_selector_memberships.to_vec();
    let matches = statement_layers == strings_from_static(&expected_statement_layers)
        && block_layers == strings_from_static(&expected_block_layers)
        && layer_selector_memberships == strings_from_static(&expected_layer_selector_memberships)
        && rejected_names_absent;

    ParserCstContextRawScanFixtureReportV0 {
        label: fixture.label,
        statement_layers,
        expected_statement_layers,
        block_layers,
        expected_block_layers,
        layer_selector_memberships,
        expected_layer_selector_memberships,
        rejected_names: fixture.rejected_names.to_vec(),
        rejected_names_absent,
        matches,
    }
}

fn strings_from_static(values: &[&'static str]) -> Vec<String> {
    values.iter().map(|value| (*value).to_string()).collect()
}

fn style_fact_category_value_sets(facts: &ParsedStyleFacts) -> Vec<(&'static str, Vec<String>)> {
    vec![
        (
            "selectors",
            sorted_unique(
                facts
                    .selectors
                    .iter()
                    .map(|fact| format!("{:?}:{}", fact.kind, fact.name)),
            ),
        ),
        (
            "variables",
            sorted_unique(facts.variables.iter().map(|fact| {
                format!(
                    "{:?}:{}:fallback={}",
                    fact.kind, fact.name, fact.has_fallback
                )
            })),
        ),
        (
            "sass_symbols",
            sorted_unique(facts.sass_symbols.iter().map(|fact| {
                format!(
                    "{:?}:{}:{}:{}:{:?}",
                    fact.kind, fact.symbol_kind, fact.name, fact.role, fact.namespace
                )
            })),
        ),
        (
            "sass_includes",
            sorted_unique(
                facts
                    .sass_includes
                    .iter()
                    .map(|fact| format!("{}:{:?}:{}", fact.name, fact.namespace, fact.params)),
            ),
        ),
        (
            "sass_module_edges",
            sorted_unique(facts.sass_module_edges.iter().map(|fact| {
                format!(
                    "{:?}:{}:{:?}:{:?}:{:?}:{:?}:{:?}:media={}",
                    fact.kind,
                    fact.source,
                    fact.namespace_kind,
                    fact.namespace,
                    fact.forward_prefix,
                    fact.visibility_filter_kind,
                    fact.visibility_filter_names,
                    fact.media_qualified
                )
            })),
        ),
        (
            "extend_targets",
            sorted_unique(
                facts.extend_targets.iter().map(|fact| {
                    format!("{:?}:{}:optional={}", fact.kind, fact.name, fact.optional)
                }),
            ),
        ),
        (
            "animations",
            sorted_unique(
                facts
                    .animations
                    .iter()
                    .map(|fact| format!("{:?}:{}", fact.kind, fact.name)),
            ),
        ),
        (
            "css_module_values",
            sorted_unique(
                facts
                    .css_module_values
                    .iter()
                    .map(|fact| format!("{:?}:{}", fact.kind, fact.name)),
            ),
        ),
        (
            "css_module_value_import_edges",
            sorted_unique(facts.css_module_value_import_edges.iter().map(|fact| {
                format!(
                    "{}:{}:{}",
                    fact.remote_name, fact.local_name, fact.import_source
                )
            })),
        ),
        (
            "css_module_value_definition_edges",
            sorted_unique(
                facts
                    .css_module_value_definition_edges
                    .iter()
                    .map(|fact| format!("{}:{:?}", fact.definition_name, fact.reference_names)),
            ),
        ),
        (
            "css_module_composes",
            sorted_unique(
                facts
                    .css_module_composes
                    .iter()
                    .map(|fact| format!("{:?}:{}", fact.kind, fact.name)),
            ),
        ),
        (
            "css_module_composes_edges",
            sorted_unique(facts.css_module_composes_edges.iter().map(|fact| {
                format!(
                    "{:?}:{:?}:{:?}:{:?}",
                    fact.kind, fact.owner_selector_names, fact.target_names, fact.import_source
                )
            })),
        ),
        (
            "icss",
            sorted_unique(
                facts
                    .icss
                    .iter()
                    .map(|fact| format!("{:?}:{}", fact.kind, fact.name)),
            ),
        ),
        (
            "icss_import_edges",
            sorted_unique(facts.icss_import_edges.iter().map(|fact| {
                format!(
                    "{}:{}:{}",
                    fact.local_name, fact.remote_name, fact.import_source
                )
            })),
        ),
        (
            "icss_export_edges",
            sorted_unique(
                facts
                    .icss_export_edges
                    .iter()
                    .map(|fact| format!("{}:{:?}", fact.export_name, fact.reference_names)),
            ),
        ),
        (
            "at_rules",
            sorted_unique(
                facts
                    .at_rules
                    .iter()
                    .map(|fact| format!("{}:{:?}", fact.name, fact.node_kind)),
            ),
        ),
    ]
}

fn style_fact_category_span_sets(facts: &ParsedStyleFacts) -> Vec<(&'static str, Vec<String>)> {
    vec![
        (
            "selectors",
            sorted_unique(
                facts
                    .selectors
                    .iter()
                    .map(|fact| format!("{:?}:{}", fact.kind, span_record(&fact.range))),
            ),
        ),
        (
            "variables",
            sorted_unique(
                facts
                    .variables
                    .iter()
                    .map(|fact| format!("{:?}:{}", fact.kind, span_record(&fact.range))),
            ),
        ),
        (
            "sass_symbols",
            sorted_unique(
                facts
                    .sass_symbols
                    .iter()
                    .map(|fact| format!("{:?}:{}", fact.kind, span_record(&fact.range))),
            ),
        ),
        (
            "sass_includes",
            sorted_unique(
                facts
                    .sass_includes
                    .iter()
                    .map(|fact| span_record(&fact.range)),
            ),
        ),
        (
            "sass_module_edges",
            sorted_unique(
                facts
                    .sass_module_edges
                    .iter()
                    .map(|fact| format!("{:?}:{}", fact.kind, span_record(&fact.range))),
            ),
        ),
        (
            "extend_targets",
            sorted_unique(
                facts
                    .extend_targets
                    .iter()
                    .map(|fact| format!("{:?}:{}", fact.kind, span_record(&fact.range))),
            ),
        ),
        (
            "animations",
            sorted_unique(
                facts
                    .animations
                    .iter()
                    .map(|fact| format!("{:?}:{}", fact.kind, span_record(&fact.range))),
            ),
        ),
        (
            "css_module_values",
            sorted_unique(
                facts
                    .css_module_values
                    .iter()
                    .map(|fact| format!("{:?}:{}", fact.kind, span_record(&fact.range))),
            ),
        ),
        (
            "css_module_value_import_edges",
            sorted_unique(facts.css_module_value_import_edges.iter().map(|fact| {
                format!(
                    "local={}:remote={}:statement={}",
                    span_record(&fact.local_range),
                    span_record(&fact.remote_range),
                    span_record(&fact.range)
                )
            })),
        ),
        (
            "css_module_value_definition_edges",
            sorted_unique(
                facts
                    .css_module_value_definition_edges
                    .iter()
                    .map(|fact| span_record(&fact.range)),
            ),
        ),
        (
            "css_module_composes",
            sorted_unique(
                facts
                    .css_module_composes
                    .iter()
                    .map(|fact| format!("{:?}:{}", fact.kind, span_record(&fact.range))),
            ),
        ),
        (
            "css_module_composes_edges",
            sorted_unique(
                facts
                    .css_module_composes_edges
                    .iter()
                    .map(|fact| format!("{:?}:{}", fact.kind, span_record(&fact.range))),
            ),
        ),
        (
            "icss",
            sorted_unique(
                facts
                    .icss
                    .iter()
                    .map(|fact| format!("{:?}:{}", fact.kind, span_record(&fact.range))),
            ),
        ),
        (
            "icss_import_edges",
            sorted_unique(
                facts
                    .icss_import_edges
                    .iter()
                    .map(|fact| span_record(&fact.range)),
            ),
        ),
        (
            "icss_export_edges",
            sorted_unique(
                facts
                    .icss_export_edges
                    .iter()
                    .map(|fact| span_record(&fact.range)),
            ),
        ),
        (
            "at_rules",
            sorted_unique(
                facts
                    .at_rules
                    .iter()
                    .map(|fact| format!("{}:{}", fact.name, span_record(&fact.range))),
            ),
        ),
    ]
}

fn span_record(range: &impl std::fmt::Debug) -> String {
    format!("{range:?}")
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
    let mut incremental_database = OmenaIncrementalDatabaseV0::default();
    incremental_database.restore_snapshot(&previous_snapshot);
    let identity_keyed_reuse = incremental_database
        .plan_and_upsert_graph_input(&next)
        .incremental_plan;
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

pub fn summarize_expression_domain_source_cfg_refinement_oracle_v0()
-> SourceCfgRefinementOracleReportV0 {
    let reports = vec![source_cfg_refinement_fixture_report(
        "branchy-two-expression-file",
    )];
    let strict_refinement_witness_count = reports
        .iter()
        .map(|report| report.strict_refinement_count)
        .sum();
    let all_source_values_le_baseline = reports
        .iter()
        .all(|report| report.all_source_values_le_baseline);
    let all_source_values_covered_by_baseline = reports
        .iter()
        .all(|report| report.all_source_values_covered_by_baseline);
    let all_shape_witnesses_present = reports.iter().all(|report| {
        report.branch_block_observed
            && report.concat_transfer_observed
            && report.file_merge_absent_from_source_cfg
    });

    SourceCfgRefinementOracleReportV0 {
        schema_version: "0",
        product: "omena-diff-test.expression-domain-source-cfg-refinement-oracle",
        fixture_count: reports.len(),
        strict_refinement_witness_count,
        all_source_values_le_baseline,
        all_source_values_covered_by_baseline,
        all_shape_witnesses_present,
        reports,
    }
}

fn source_cfg_refinement_fixture_report(
    fixture_id: &'static str,
) -> SourceCfgRefinementFixtureReportV0 {
    let baseline_summary =
        summarize_expression_domain_flow_analysis_input(&source_cfg_refinement_input(false));
    let source_summary =
        summarize_expression_domain_control_flow_analysis_input(&source_cfg_refinement_input(true));
    let baseline_value = flow_baseline_value(&baseline_summary);
    let baseline_value_set = baseline_value
        .as_ref()
        .and_then(enumerate_finite_class_values)
        .map(|values| values.into_iter().collect::<BTreeSet<_>>());
    let source_exits = source_summary
        .analyses
        .iter()
        .filter_map(|entry| control_flow_exit_value(&entry.analysis, "exit"))
        .collect::<Vec<_>>();
    let all_source_values_le_baseline = !source_exits.is_empty()
        && baseline_value.is_some()
        && source_exits.iter().all(|exit| {
            baseline_value
                .as_ref()
                .is_some_and(|baseline| derived_le_class_value(exit.value, baseline))
        });
    let strict_refinement_count = source_exits
        .iter()
        .filter(|exit| {
            baseline_value.as_ref().is_some_and(|baseline| {
                derived_le_class_value(exit.value, baseline) && exit.value != baseline
            })
        })
        .count();
    let all_source_values_covered_by_baseline = !source_exits.is_empty()
        && baseline_value_set.as_ref().is_some_and(|baseline_values| {
            source_exits.iter().all(|exit| {
                enumerate_finite_class_values(exit.value).is_some_and(|source_values| {
                    !source_values.is_empty()
                        && source_values
                            .iter()
                            .all(|value| baseline_values.contains(value))
                })
            })
        });
    let branch_block_observed = source_summary
        .analyses
        .iter()
        .any(|entry| !entry.analysis.branch_block_ids.is_empty());
    let concat_transfer_observed = source_summary.analyses.iter().any(|entry| {
        entry
            .analysis
            .flow_analysis
            .nodes
            .iter()
            .any(|node| node.transfer_kind == "concatFacts")
    });
    let file_merge_absent_from_source_cfg = source_summary.analyses.iter().all(|entry| {
        entry.analysis.blocks.iter().all(|block| {
            block.block_id != "file-merge"
                && block.node_ids.iter().all(|node_id| node_id != "file-merge")
        })
    });

    SourceCfgRefinementFixtureReportV0 {
        fixture_id,
        baseline_graph_id: if baseline_summary.analyses.is_empty() {
            String::new()
        } else {
            format!("{fixture_id}:legacy-file-merge-baseline")
        },
        source_graph_ids: source_summary
            .analyses
            .iter()
            .map(|entry| entry.graph_id.clone())
            .collect(),
        baseline_value_kind: baseline_value
            .as_ref()
            .map_or("missing", abstract_class_value_kind),
        source_value_kinds: source_exits.iter().map(|exit| exit.kind).collect(),
        all_source_values_le_baseline,
        strict_refinement_count,
        all_source_values_covered_by_baseline,
        branch_block_observed,
        concat_transfer_observed,
        file_merge_absent_from_source_cfg,
    }
}

fn derived_le_class_value(left: &AbstractClassValueV0, right: &AbstractClassValueV0) -> bool {
    join_abstract_class_values(left, right) == *right
}

fn flow_baseline_value(
    summary: &engine_input_producers::ExpressionDomainFlowAnalysisV0,
) -> Option<AbstractClassValueV0> {
    summary
        .analyses
        .iter()
        .flat_map(|entry| entry.analysis.nodes.iter())
        .map(|node| node.value.clone())
        .reduce(|left, right| join_abstract_class_values(&left, &right))
}

#[derive(Clone, Copy)]
struct ControlFlowExitValue<'a> {
    kind: &'static str,
    value: &'a AbstractClassValueV0,
}

fn control_flow_exit_value<'a>(
    analysis: &'a omena_abstract_value::ClassValueControlFlowAnalysisV0,
    preferred_block_id: &str,
) -> Option<ControlFlowExitValue<'a>> {
    analysis
        .blocks
        .iter()
        .find(|block| block.block_id == preferred_block_id)
        .or_else(|| {
            analysis
                .blocks
                .iter()
                .find(|block| block.successor_block_ids.is_empty())
        })
        .map(|block| ControlFlowExitValue {
            kind: block.exit_value_kind,
            value: &block.exit_value,
        })
}

fn source_cfg_refinement_input(include_source_cfg: bool) -> EngineInputV2 {
    EngineInputV2 {
        version: "2".to_string(),
        sources: Vec::new(),
        styles: Vec::new(),
        type_facts: vec![
            exact_type_fact_with_optional_cfg(
                "expr-primary",
                "btn-primary",
                include_source_cfg.then(branchy_type_fact_control_flow_graph),
            ),
            exact_type_fact_with_optional_cfg(
                "expr-secondary",
                "btn-secondary",
                include_source_cfg.then(branchy_type_fact_control_flow_graph),
            ),
        ],
    }
}

fn exact_type_fact_with_optional_cfg(
    expression_id: &str,
    value: &str,
    control_flow_graph: Option<TypeFactControlFlowGraphV2>,
) -> TypeFactEntryV2 {
    TypeFactEntryV2 {
        file_path: "/tmp/App.tsx".to_string(),
        expression_id: expression_id.to_string(),
        facts: StringTypeFactsV2 {
            kind: "exact".to_string(),
            constraint_kind: None,
            values: Some(vec![value.to_string()]),
            prefix: None,
            suffix: None,
            min_len: None,
            max_len: None,
            char_must: None,
            char_may: None,
            may_include_other_chars: None,
            provenance: None,
        },
        control_flow_graph,
    }
}

fn branchy_type_fact_control_flow_graph() -> TypeFactControlFlowGraphV2 {
    TypeFactControlFlowGraphV2 {
        entry_block_id: "entry".to_string(),
        blocks: vec![
            type_fact_control_flow_block("entry", "entry", "entry", &["branch:0"]),
            type_fact_control_flow_block("branch:0", "branch", "branch", &["then:0", "else:0"]),
            type_fact_control_flow_block("then:0", "assignment", "concatFacts", &["join:0"]),
            type_fact_control_flow_block("else:0", "assignment", "assignFacts", &["join:0"]),
            type_fact_control_flow_block("join:0", "join", "join", &["exit"]),
            type_fact_control_flow_block("exit", "exit", "exit", &[]),
        ],
    }
}

fn type_fact_control_flow_block(
    id: &str,
    kind: &str,
    transfer_kind: &str,
    successor_block_ids: &[&str],
) -> TypeFactControlFlowBlockV2 {
    TypeFactControlFlowBlockV2 {
        id: id.to_string(),
        kind: kind.to_string(),
        transfer_kind: transfer_kind.to_string(),
        successor_block_ids: successor_block_ids
            .iter()
            .map(|id| (*id).to_string())
            .collect(),
        symbol_ordinal: None,
        variable_name: None,
        expression_kind: None,
        facts: None,
    }
}

fn workspace_snapshot_id_contract_status_v0() -> (bool, bool, bool, usize, usize) {
    let revision = IncrementalRevisionV0 { value: 11 };
    let same_revision = IncrementalRevisionV0 { value: 11 };
    let next_revision = IncrementalRevisionV0 { value: 12 };
    let id = OmenaWorkspaceSnapshotIdV0::from_revision(revision);
    let rekey_equivalence_ready = id == OmenaWorkspaceSnapshotIdV0::from_revision(same_revision)
        && id != OmenaWorkspaceSnapshotIdV0::from_revision(next_revision)
        && id.revision() == revision;

    let query_surface_ready = workspace_snapshot_id_query_surface_ready_v0();
    let (lsp_report_surface_ready, type_census_count, surface_without_id_count) =
        workspace_snapshot_id_type_census_v0();

    (
        rekey_equivalence_ready,
        query_surface_ready,
        lsp_report_surface_ready,
        type_census_count,
        surface_without_id_count,
    )
}

fn workspace_snapshot_id_query_surface_ready_v0() -> bool {
    let resolution_inputs = omena_query::OmenaQueryStyleResolutionInputsV0::default();
    let corpus = vec![
        OmenaQueryStyleSourceInputV0 {
            style_path: "/tmp/base.module.scss".to_string(),
            style_source: ".base { color: red; }".to_string(),
        },
        OmenaQueryStyleSourceInputV0 {
            style_path: "/tmp/App.module.scss".to_string(),
            style_source: ".app { composes: base from \"./base.module.scss\"; }".to_string(),
        },
    ];
    let mut host = OmenaQueryStyleMemoHostV0::new();
    let Some(initial) = host.workspace_style_diagnostics_with_selector(
        "/tmp/App.module.scss",
        corpus.as_slice(),
        &[],
        &[],
        &[],
        &resolution_inputs,
    ) else {
        return false;
    };
    let Some(unchanged) = host.workspace_style_diagnostics_with_selector(
        "/tmp/App.module.scss",
        corpus.as_slice(),
        &[],
        &[],
        &[],
        &resolution_inputs,
    ) else {
        return false;
    };
    let mut edited_corpus = corpus.clone();
    edited_corpus[1]
        .style_source
        .push_str("\n.app__icon { color: blue; }\n");
    let Some(edited) = host.workspace_style_diagnostics_with_selector(
        "/tmp/App.module.scss",
        edited_corpus.as_slice(),
        &[],
        &[],
        &[],
        &resolution_inputs,
    ) else {
        return false;
    };

    initial.snapshot_id() == initial.selector.snapshot_id()
        && initial.snapshot_id()
            == OmenaWorkspaceSnapshotIdV0::from_revision(IncrementalRevisionV0 { value: 1 })
        && unchanged.snapshot_id() == initial.snapshot_id()
        && edited.snapshot_id()
            == OmenaWorkspaceSnapshotIdV0::from_revision(IncrementalRevisionV0 { value: 2 })
}

fn workspace_snapshot_id_type_census_v0() -> (bool, usize, usize) {
    let salsa_memo_source = include_str!("../../omena-query/src/style/salsa_memo.rs");
    let lsp_output_source = include_str!("../../omena-lsp-server/src/lsp_output.rs");
    let lsp_style_diagnostics_source =
        include_str!("../../omena-lsp-server/src/style_diagnostics.rs");
    let lsp_style_diagnostics_snapshot_source =
        include_str!("../../omena-lsp-server/src/style_diagnostics_snapshot.rs");
    let lsp_deferred_source = include_str!("../../omena-lsp-server/src/deferred_notification.rs");
    let lsp_parallel_source = include_str!("../../omena-lsp-server/src/parallel_style_wave.rs");

    let surface_checks = [
        salsa_memo_source.contains("pub struct OmenaQueryStyleDiagnosticsWithSelectorV0"),
        salsa_memo_source.contains("pub fn snapshot_id(&self) -> OmenaWorkspaceSnapshotIdV0"),
        lsp_output_source
            .contains("pub snapshot_id: Option<omena_query::OmenaWorkspaceSnapshotIdV0>"),
        lsp_output_source
            .contains("pub workspace_snapshot_id: Option<omena_query::OmenaWorkspaceSnapshotIdV0>"),
        lsp_style_diagnostics_source.contains("inputs.snapshot_id")
            && lsp_style_diagnostics_snapshot_source.contains("\"snapshotId\"")
            && lsp_style_diagnostics_snapshot_source
                .contains("OmenaWorkspaceSnapshotIdV0::from_revision")
            && !lsp_style_diagnostics_snapshot_source.contains("compute_omena_sif_leaf_hash_v1"),
        lsp_deferred_source.contains("dispatch.workspace_snapshot_id.or(snapshot_id)"),
        lsp_parallel_source.contains("OmenaWorkspaceSnapshotIdV0::from_revision")
            && !lsp_parallel_source.contains("workspace_snapshot_id_for_style_diagnostics_surface"),
    ];
    let type_census_count = [
        salsa_memo_source,
        lsp_output_source,
        lsp_style_diagnostics_source,
        lsp_style_diagnostics_snapshot_source,
        lsp_deferred_source,
        lsp_parallel_source,
    ]
    .iter()
    .map(|source| source.matches("OmenaWorkspaceSnapshotIdV0").count())
    .sum();
    let surface_without_id_count = surface_checks
        .iter()
        .filter(|surface_has_id| !**surface_has_id)
        .count();
    (
        surface_checks.iter().all(|surface_has_id| *surface_has_id),
        type_census_count,
        surface_without_id_count,
    )
}

fn workspace_graph_contract_texts_ready_v0() -> bool {
    let text = include_str!("../../../omena-workspace-graph-contracts.md");
    [
        "Workspace Snapshot Id Contract",
        "Typed Graph Summary Plane Contract",
        "Residual Ledger",
        "OmenaWorkspaceSnapshotIdV0",
        "OmenaQueryCrossFileSummaryV0",
    ]
    .iter()
    .all(|required| text.contains(required))
}

fn summarize_typed_graph_summary_plane_foundation_v0() -> TypedGraphSummaryPlaneFoundationReportV0 {
    let source_documents = vec![OmenaQuerySourceDocumentInputV0 {
        source_path: "/tmp/Button.tsx".to_string(),
        source_source: r#"import bind from "classnames/bind";
import styles from "./Button.module.scss";
const cx = bind.bind(styles);
export function Button({ variant }) {
  return <div className={cx(`btn-${variant}`)} />;
}"#
        .to_string(),
        source_syntax_index: None,
        has_unresolved_style_import: false,
    }];
    let before = summarize_omena_query_workspace_cross_file_summary(
        &[OmenaQueryStyleSourceInputV0 {
            style_path: "/tmp/Button.module.scss".to_string(),
            style_source: ".btn-primary { color: red; }".to_string(),
        }],
        source_documents.as_slice(),
        &[],
    );
    let after = summarize_omena_query_workspace_cross_file_summary(
        &[
            OmenaQueryStyleSourceInputV0 {
                style_path: "/tmp/base.module.scss".to_string(),
                style_source: ".base { color: red; }".to_string(),
            },
            OmenaQueryStyleSourceInputV0 {
                style_path: "/tmp/Button.module.scss".to_string(),
                style_source: ".btn-primary { composes: base from \"./base.module.scss\"; }"
                    .to_string(),
            },
        ],
        source_documents.as_slice(),
        &[],
    );
    let before_view = summarize_cross_file_summary_view_v0(&before);
    let after_view = summarize_cross_file_summary_view_v0(&after);
    let delta = summarize_cross_file_graph_delta_v0(&before, &after);
    let before_json_counts_match = serde_json::to_string(&before_view.recomputed_edge_kind_counts)
        .ok()
        == serde_json::to_string(&before.edge_kind_counts).ok();
    let after_json_counts_match = serde_json::to_string(&after_view.recomputed_edge_kind_counts)
        .ok()
        == serde_json::to_string(&after.edge_kind_counts).ok();
    let known_edit_graph_delta_ready = delta.all_delta_edges_typed
        && delta.removed_edges.is_empty()
        && delta
            .added_edges
            .iter()
            .any(|edge| edge.raw_edge_kind == "cssModulesComposesImport")
        && delta
            .added_edges
            .iter()
            .any(|edge| edge.raw_edge_kind == "cssModulesComposesClosure");
    let all_summary_views_ready = before_view.summary_view_ready && after_view.summary_view_ready;
    let all_summary_view_json_counts_match = before_json_counts_match && after_json_counts_match;
    let all_raw_edge_kinds_in_catalog =
        before_view.all_raw_edge_kinds_in_catalog && after_view.all_raw_edge_kinds_in_catalog;
    let all_node_roles_in_catalog =
        before_view.all_node_roles_in_catalog && after_view.all_node_roles_in_catalog;
    let (
        workspace_snapshot_id_rekey_equivalence_ready,
        workspace_snapshot_id_query_surface_ready,
        workspace_snapshot_id_lsp_report_surface_ready,
        workspace_snapshot_id_type_census_count,
        workspace_snapshot_id_surface_without_id_count,
    ) = workspace_snapshot_id_contract_status_v0();
    let workspace_snapshot_id_contract_ready = workspace_snapshot_id_rekey_equivalence_ready
        && workspace_snapshot_id_query_surface_ready
        && workspace_snapshot_id_lsp_report_surface_ready
        && workspace_snapshot_id_surface_without_id_count == 0;
    let workspace_graph_contract_texts_ready = workspace_graph_contract_texts_ready_v0();
    let all_foundation_checks_hold = UNIFIED_HYPERGRAPH_EDGE_KIND_VARIANTS_V0.len() == 11
        && CROSS_FILE_SUMMARY_RAW_EDGE_KIND_LABELS_V0.len() >= 22
        && CROSS_FILE_SUMMARY_NODE_ROLE_LABELS_V0 == ["source", "style"]
        && all_summary_views_ready
        && all_summary_view_json_counts_match
        && all_raw_edge_kinds_in_catalog
        && all_node_roles_in_catalog
        && known_edit_graph_delta_ready
        && workspace_snapshot_id_contract_ready
        && workspace_graph_contract_texts_ready;
    let workspace_summary_plane_and_snapshot_id_green =
        all_foundation_checks_hold && workspace_snapshot_id_contract_ready;

    TypedGraphSummaryPlaneFoundationReportV0 {
        schema_version: "0",
        product: "omena-diff-test.typed-graph-summary-plane-foundation",
        status: if all_foundation_checks_hold {
            "ready"
        } else {
            "needsInput"
        },
        fixture_count: 2,
        unified_edge_kind_variant_count: UNIFIED_HYPERGRAPH_EDGE_KIND_VARIANTS_V0.len(),
        raw_edge_kind_catalog_count: CROSS_FILE_SUMMARY_RAW_EDGE_KIND_LABELS_V0.len(),
        node_role_catalog_count: CROSS_FILE_SUMMARY_NODE_ROLE_LABELS_V0.len(),
        summary_edge_count: after.summary_edge_count,
        graph_delta_added_edge_count: delta.added_edges.len(),
        graph_delta_removed_edge_count: delta.removed_edges.len(),
        all_summary_views_ready,
        all_summary_view_json_counts_match,
        all_raw_edge_kinds_in_catalog,
        all_node_roles_in_catalog,
        known_edit_graph_delta_ready,
        workspace_snapshot_id_contract_ready,
        workspace_snapshot_id_contract_status: if workspace_snapshot_id_contract_ready {
            "ready"
        } else {
            "needsInput"
        },
        workspace_snapshot_id_type_census_count,
        workspace_snapshot_id_surface_without_id_count,
        workspace_snapshot_id_rekey_equivalence_ready,
        workspace_snapshot_id_query_surface_ready,
        workspace_snapshot_id_lsp_report_surface_ready,
        workspace_graph_contract_texts_ready,
        workspace_summary_plane_and_snapshot_id_green,
        all_foundation_checks_hold,
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
    let parser_cst_fact_authority_report = summarize_parser_cst_fact_authority_equivalence_v0();
    let transform_ir_identity_round_trip_report =
        summarize_transform_ir_identity_round_trip_equivalence_v0();
    let sass_spec_import_scale_report = summarize_sass_spec_import_scale();
    let sass_spec_expectation_bucket_report = summarize_sass_spec_expectation_buckets();
    let sass_spec_sound_bail_membership_report = summarize_sass_spec_sound_bail_membership();
    let sass_spec_static_match_report = summarize_sass_spec_static_must_match();
    let sass_spec_expectation_bucket_ledger_report =
        summarize_sass_spec_expectation_bucket_ledger();
    let sass_spec_bail_site_ledger_report = summarize_sass_spec_bail_site_ledger();
    let parser_cst_context_raw_scan_report = summarize_parser_cst_context_raw_scan_divergence_v0();
    let selector_context_soundness_report = summarize_selector_context_soundness_v0();
    let source_cfg_refinement_report =
        summarize_expression_domain_source_cfg_refinement_oracle_v0();
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
    let reachability_equivalence_report = summarize_reachability_second_oracle_equivalence_v0();
    let scss_eval_truthiness_cst_equivalence_report =
        summarize_scss_eval_truthiness_cst_equivalence();
    let scss_eval_public_summary_equivalence_report =
        summarize_scss_eval_public_summary_equivalence_v0();
    let oss_corpus_farm_manifest_report = summarize_oss_corpus_farm_manifest_v0();
    let all_oss_corpus_farm_manifest_checks_hold = oss_corpus_farm_manifest_report
        .all_entries_follow_generated_envelope_shape
        && oss_corpus_farm_manifest_report.all_entries_stage1_advisory
        && oss_corpus_farm_manifest_report.all_entries_out_of_scope
        && oss_corpus_farm_manifest_report.all_entries_have_permissive_spdx
        && oss_corpus_farm_manifest_report.all_entry_pins_are_sha_locked
        && oss_corpus_farm_manifest_report.all_recorded_shas_match_source_pins
        && oss_corpus_farm_manifest_report.all_sparse_paths_are_bounded
        && oss_corpus_farm_manifest_report.all_chunk_hashes_match;
    let deletion_stale_reuse_corpus_report = summarize_deletion_stale_reuse_corpus_v0();
    let typed_graph_summary_plane_foundation_report =
        summarize_typed_graph_summary_plane_foundation_v0();

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
        sass_spec_imported_fixture_count: sass_spec_expectation_bucket_report.fixture_count,
        sass_spec_import_source_archive_count: sass_spec_import_scale_report.source_archive_count,
        sass_spec_import_chunk_count: sass_spec_import_scale_report.imported_chunk_count,
        sass_spec_per_push_smoke_fixture_count: sass_spec_import_scale_report
            .per_push_smoke_fixture_count,
        sass_spec_per_push_smoke_floor_fixture_count: sass_spec_import_scale_report
            .per_push_smoke_floor_fixture_count,
        all_sass_spec_import_scale_counts_match: sass_spec_import_scale_report
            .all_import_scale_checks_hold,
        all_sass_spec_smoke_floor_holds: sass_spec_import_scale_report.all_smoke_floor_holds,
        sass_spec_static_must_match_count: sass_spec_expectation_bucket_report
            .static_must_match_count,
        sass_spec_expected_sound_bail_count: sass_spec_expectation_bucket_report
            .expected_sound_bail_count,
        sass_spec_parser_recovery_count: sass_spec_expectation_bucket_report.parser_recovery_count,
        sass_spec_out_of_scope_count: sass_spec_expectation_bucket_report.out_of_scope_count,
        all_sass_spec_imported_fixtures_have_one_expectation_kind:
            sass_spec_expectation_bucket_report.all_fixtures_have_one_expectation_kind,
        all_sass_spec_imported_expectation_kinds_match_criteria:
            sass_spec_expectation_bucket_report.all_expectation_kinds_match_criteria,
        sass_spec_sound_bail_case_count: sass_spec_sound_bail_membership_report.case_count,
        sass_spec_sound_bail_checked_case_count: sass_spec_sound_bail_membership_report
            .checked_case_count,
        sass_spec_sound_bail_non_top_case_count: sass_spec_sound_bail_membership_report
            .non_top_case_count,
        sass_spec_sound_bail_raw_tightness_case_count: sass_spec_sound_bail_membership_report
            .raw_tightness_case_count,
        all_sass_spec_sound_bail_membership_checks_hold: sass_spec_sound_bail_membership_report
            .all_membership_checks_hold,
        sass_spec_static_match_case_count: sass_spec_static_match_report.case_count,
        sass_spec_static_match_checked_case_count: sass_spec_static_match_report.checked_case_count,
        sass_spec_static_match_declaration_value_match_count: sass_spec_static_match_report
            .matched_declaration_value_count,
        all_sass_spec_static_match_checks_hold: sass_spec_static_match_report
            .all_static_match_checks_hold,
        all_sass_spec_expectation_bucket_totals_match_ledger:
            sass_spec_expectation_bucket_ledger_report.all_bucket_totals_match_ledger,
        all_sass_spec_expectation_fixture_assignments_match_ledger:
            sass_spec_expectation_bucket_ledger_report.all_fixture_assignments_match_ledger,
        sass_spec_bail_site_ledger_site_count: sass_spec_bail_site_ledger_report
            .semantic_site_count,
        sass_spec_bail_site_raw_pattern_hit_count: sass_spec_bail_site_ledger_report
            .raw_pattern_hit_count,
        sass_spec_bail_site_non_semantic_pattern_hit_count: sass_spec_bail_site_ledger_report
            .non_semantic_pattern_hit_count,
        sass_spec_bail_site_linked_site_count: sass_spec_bail_site_ledger_report.linked_site_count,
        sass_spec_bail_site_named_gap_count: sass_spec_bail_site_ledger_report.named_gap_site_count,
        sass_spec_bail_site_linked_case_count: sass_spec_bail_site_ledger_report.linked_case_count,
        all_sass_spec_bail_site_ledger_counts_match: sass_spec_bail_site_ledger_report
            .all_semantic_sites_match_ledger,
        all_sass_spec_bail_site_ledger_entries_linked_or_named_gap:
            sass_spec_bail_site_ledger_report.all_sites_linked_or_named_gap,
        all_sass_spec_bail_site_ledger_link_reason_classes_match: sass_spec_bail_site_ledger_report
            .all_linked_cases_match_reason_class,
        all_sass_spec_bail_site_ledger_linked_cases_are_imported: sass_spec_bail_site_ledger_report
            .all_linked_cases_are_imported_sound_bail_cases,
        soundiness_metamorphic_relation_count: soundiness_metamorphic_report.relation_count,
        all_soundiness_metamorphic_relations_hold: soundiness_metamorphic_report.all_relations_hold,
        diagnostic_metamorphic_relation_count: diagnostic_metamorphic_report.relation_count,
        all_diagnostic_metamorphic_relations_hold: diagnostic_metamorphic_report.all_relations_hold,
        parser_cst_fact_authority_fixture_count: parser_cst_fact_authority_report.fixture_count,
        parser_cst_fact_authority_comparison_count: parser_cst_fact_authority_report
            .comparisons
            .len(),
        all_parser_cst_fact_authority_values_match: parser_cst_fact_authority_report
            .all_value_sets_match,
        all_parser_cst_fact_authority_spans_match: parser_cst_fact_authority_report
            .all_span_sets_match,
        parser_cst_fact_authority_metamorphic_relation_count: parser_cst_fact_authority_report
            .metamorphic_relation_count,
        all_parser_cst_fact_authority_metamorphic_relations_hold: parser_cst_fact_authority_report
            .all_metamorphic_relations_hold,
        parser_cst_context_raw_scan_fixture_count: parser_cst_context_raw_scan_report.fixture_count,
        all_parser_cst_context_raw_scan_fixtures_match: parser_cst_context_raw_scan_report
            .all_fixtures_match,
        selector_context_soundness_fixture_count: selector_context_soundness_report.fixture_count,
        all_selector_context_soundness_fixtures_match: selector_context_soundness_report
            .all_expected_verdicts_match
            && selector_context_soundness_report.all_unmodeled_fixtures_stay_maybe
            && selector_context_soundness_report.positive_preservation_matches,
        source_cfg_refinement_fixture_count: source_cfg_refinement_report.fixture_count,
        all_source_cfg_values_le_file_merge_baseline: source_cfg_refinement_report
            .all_source_values_le_baseline,
        source_cfg_strict_refinement_witness_count: source_cfg_refinement_report
            .strict_refinement_witness_count,
        all_source_cfg_values_covered_by_file_merge_baseline: source_cfg_refinement_report
            .all_source_values_covered_by_baseline,
        all_source_cfg_shape_witnesses_present: source_cfg_refinement_report
            .all_shape_witnesses_present,
        cache_equivalence_file_count: cache_equivalence_report.file_count,
        all_cache_equivalence_files_identical: cache_equivalence_report.all_files_identical,
        salsa_memo_equivalence_comparison_count: salsa_memo_equivalence_report.comparison_count,
        all_salsa_memo_equivalence_phases_identical: salsa_memo_equivalence_report
            .all_phases_identical,
        parallel_salsa_equivalence_comparison_count: parallel_salsa_equivalence_report
            .comparison_count,
        all_parallel_salsa_equivalence_phases_identical: parallel_salsa_equivalence_report
            .all_phases_identical,
        reachability_equivalence_fixture_count: reachability_equivalence_report.fixture_count,
        all_reachability_second_oracle_sets_equal: reachability_equivalence_report.all_sets_equal,
        all_reachability_streaming_matches_batch: reachability_equivalence_report
            .streaming_matches_batch,
        all_reachability_bitset_parity_equal: reachability_equivalence_report
            .all_reachability_bitset_parity_equal,
        all_reachability_closure_hash_bitset_parity_equal: reachability_equivalence_report
            .all_closure_hash_bitset_parity_equal,
        all_reachability_product_parity_with_batch: reachability_equivalence_report
            .product_reachability_parity_with_batch,
        all_reachability_fact_keys_three_way_equal: reachability_equivalence_report
            .all_fact_keys_three_way_equal,
        all_reachability_fact_keys_four_way_equal: reachability_equivalence_report
            .all_fact_keys_four_way_equal,
        all_reachability_selector_relations_equal: reachability_equivalence_report
            .selector_relations_equal,
        scss_eval_truthiness_cst_equivalence_fixture_count:
            scss_eval_truthiness_cst_equivalence_report.fixture_count,
        all_scss_eval_truthiness_cst_equivalence_fixtures_match:
            scss_eval_truthiness_cst_equivalence_report.all_fixtures_match,
        scss_eval_public_summary_comparison_count: scss_eval_public_summary_equivalence_report
            .comparison_count,
        all_scss_eval_public_summaries_match: scss_eval_public_summary_equivalence_report
            .all_summaries_match,
        transform_ir_identity_round_trip_fixture_count: transform_ir_identity_round_trip_report
            .fixture_count,
        all_transform_ir_identity_round_trip_fields_match: transform_ir_identity_round_trip_report
            .all_fields_match,
        oss_corpus_farm_entry_count: oss_corpus_farm_manifest_report.entry_count,
        oss_corpus_farm_repository_count: oss_corpus_farm_manifest_report.repository_count,
        oss_corpus_farm_dialect_count: oss_corpus_farm_manifest_report.dialect_count,
        all_oss_corpus_farm_manifest_checks_hold,
        deletion_stale_reuse_fixture_count: deletion_stale_reuse_corpus_report.fixture_count,
        deletion_stale_reuse_divergence_fixture_count: deletion_stale_reuse_corpus_report
            .deletion_divergence_fixture_count,
        deletion_stale_reuse_cycle_deletion_fixture_count: deletion_stale_reuse_corpus_report
            .reachability_changing_cycle_deletion_fixture_count,
        deletion_stale_reuse_demand_projected_equal_fixture_count:
            deletion_stale_reuse_corpus_report.demand_projected_equal_fixture_count,
        deletion_stale_reuse_all_demand_projected_equal: deletion_stale_reuse_corpus_report
            .all_deletion_fixtures_match_projected_batch,
        deletion_stale_reuse_ready_for_relocation_consumer: deletion_stale_reuse_corpus_report
            .ready_for_relocation_consumer,
        all_typed_graph_summary_plane_foundation_checks_hold:
            typed_graph_summary_plane_foundation_report.all_foundation_checks_hold,
        workspace_summary_plane_and_snapshot_id_green: typed_graph_summary_plane_foundation_report
            .workspace_summary_plane_and_snapshot_id_green,
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
            "reachabilitySecondOracleEquivalence",
            "reachabilityBitsetParity",
            "reachabilityClosureHashBitsetParity",
            "reachabilityFactKeyThreeWayEquivalence",
            "reachabilityFactKeyFourWayEquivalence",
            "wptValueDifferentialHandModelAgreement",
            "sassSpecExpectationBucketClassification",
            "sassSpecSoundBailMembership",
            "sassSpecStaticMatchValueAgreement",
            "sassSpecExpectationBucketLedger",
            "sassSpecBailSiteLedger",
            "sassSpecImportScaleCounts",
            "parserCstFactAuthorityEquivalence",
            "parserCstContextRawScanDivergence",
            "selectorContextSoundness",
            "typedGraphSummaryPlaneFoundation",
            "workspaceSnapshotIdContract",
            "workspaceGraphSummaryPlaneContract",
            "expressionDomainSourceCfgRefinementOracle",
            "scssEvalTruthinessCstEquivalence",
            "scssEvalPublicSummaryPreservation",
            "transformIrIdentityRoundTrip",
            "ossCorpusFarmManifest",
            "deletionStaleReuseCorpus",
            "deletionDemandProjectedBatchEquivalence",
        ],
        reports,
        m3_fixture_seed_report,
        wpt_seed_metadata_report,
        wpt_value_differential_report,
        sass_spec_expectation_bucket_report,
        sass_spec_sound_bail_membership_report,
        sass_spec_static_match_report,
        sass_spec_expectation_bucket_ledger_report,
        sass_spec_bail_site_ledger_report,
        sass_spec_import_scale_report,
        soundiness_metamorphic_report,
        diagnostic_metamorphic_report,
        parser_cst_fact_authority_report,
        transform_ir_identity_round_trip_report,
        parser_cst_context_raw_scan_report,
        selector_context_soundness_report,
        source_cfg_refinement_report,
        cache_equivalence_report,
        salsa_memo_equivalence_report,
        parallel_salsa_equivalence_report,
        reachability_equivalence_report,
        scss_eval_truthiness_cst_equivalence_report,
        scss_eval_public_summary_equivalence_report,
        oss_corpus_farm_manifest_report,
        deletion_stale_reuse_corpus_report,
        typed_graph_summary_plane_foundation_report,
    }
}

/// Summarize imported sass-spec expectation buckets.
pub fn summarize_sass_spec_import_scale() -> SassSpecImportScaleReportV0 {
    use external_corpus_envelope_idl_generated::ExternalCorpusEnvelopeV1Json;

    let seed_manifest = match serde_json::from_str::<ExternalCorpusEnvelopeV1Json>(
        SASS_SPEC_SEED_MANIFEST_SOURCE,
    ) {
        Ok(manifest) => manifest,
        Err(_) => return empty_sass_spec_import_scale_report(),
    };
    let imported_manifest = match serde_json::from_str::<ExternalCorpusEnvelopeV1Json>(
        SASS_SPEC_IMPORTED_MANIFEST_SOURCE,
    ) {
        Ok(manifest) => manifest,
        Err(_) => return empty_sass_spec_import_scale_report(),
    };
    let imported_chunk =
        match serde_json::from_str::<ImportedSassSpecChunkV0>(SASS_SPEC_IMPORTED_CHUNK_SOURCE) {
            Ok(chunk) => chunk,
            Err(_) => return empty_sass_spec_import_scale_report(),
        };
    let upstream_scale_artifact = match serde_json::from_str::<SassSpecUpstreamScaleArtifactV0>(
        SASS_SPEC_UPSTREAM_SCALE_SOURCE,
    ) {
        Ok(artifact) => artifact,
        Err(_) => return empty_sass_spec_import_scale_report(),
    };
    let expectation_report = summarize_sass_spec_expectation_buckets();
    let seed_fixture_count = seed_manifest
        .chunks
        .iter()
        .map(|chunk| usize::try_from(chunk.fixture_count).unwrap_or(0))
        .sum::<usize>();
    let seed_chunk_fixture_count = SASS_SPEC_SEED_CHUNK_SOURCES
        .iter()
        .filter_map(|source| serde_json::from_str::<FixtureListChunkV0>(source).ok())
        .map(|chunk| chunk.fixtures.len())
        .sum::<usize>();
    let source_archive_root = imported_manifest.generation.selection_path.clone();
    let scan = sass_spec_scan_source_archives(
        &source_archive_root,
        imported_manifest.source.sparse_paths.as_slice(),
    );
    let chunks = imported_manifest
        .chunks
        .iter()
        .map(|chunk| {
            let actual_source = sass_spec_imported_chunk_source_for_path(&chunk.path).unwrap_or("");
            let actual_chunk = serde_json::from_str::<ImportedSassSpecChunkV0>(actual_source).ok();
            let actual_fixture_count = actual_chunk
                .as_ref()
                .map(|chunk| chunk.fixtures.len())
                .unwrap_or(0);
            let actual_sha256 = diff_test_sha256_hex(actual_source.as_bytes());
            let manifest_fixture_count = usize::try_from(chunk.fixture_count).unwrap_or(0);
            SassSpecImportScaleChunkReportV0 {
                chunk_id: chunk.chunk_id.clone(),
                path: chunk.path.clone(),
                manifest_fixture_count,
                actual_fixture_count,
                manifest_sha256: chunk.sha256.clone(),
                actual_sha256: actual_sha256.clone(),
                fixture_count_matches: manifest_fixture_count == actual_fixture_count,
                sha256_matches: chunk.sha256 == actual_sha256,
            }
        })
        .collect::<Vec<_>>();
    let imported_fixture_count = imported_chunk.fixtures.len();
    let imported_manifest_fixture_count = imported_manifest
        .chunks
        .iter()
        .map(|chunk| usize::try_from(chunk.fixture_count).unwrap_or(0))
        .sum::<usize>();
    let sparse_manifest_count = imported_manifest
        .sparse_path_fixture_counts
        .iter()
        .map(|count| usize::try_from(count.fixture_count).unwrap_or(0))
        .sum::<usize>();
    let chunk_sparse_manifest_count = imported_manifest
        .chunks
        .iter()
        .flat_map(|chunk| chunk.sparse_path_fixture_counts.iter())
        .map(|count| usize::try_from(count.fixture_count).unwrap_or(0))
        .sum::<usize>();
    let all_imported_counts_match_manifest = imported_fixture_count
        == imported_manifest_fixture_count
        && chunks.iter().all(|chunk| chunk.fixture_count_matches);
    let all_chunk_hashes_match_manifest = chunks.iter().all(|chunk| chunk.sha256_matches);
    let all_sparse_path_counts_match_manifest = sparse_manifest_count == imported_fixture_count
        && chunk_sparse_manifest_count == imported_fixture_count;
    let all_source_archive_count_matches_imported_fixtures =
        scan.archive_count == imported_fixture_count;
    let upstream_archive_count = upstream_scale_artifact.archive_count;
    let upstream_sparse_path_archive_count = upstream_scale_artifact
        .sparse_path_archive_counts
        .iter()
        .map(|entry| entry.archive_count)
        .sum::<usize>();
    let upstream_scale_artifact_matches_manifest = upstream_scale_artifact.schema_version == "0"
        && upstream_scale_artifact.product == "omena-diff-test.sass-spec-upstream-scale"
        && upstream_scale_artifact.archive_extension == ".hrx"
        && upstream_scale_artifact.source.repository == imported_manifest.source.repository
        && upstream_scale_artifact.source.pin == imported_manifest.source.pin
        && upstream_scale_artifact.source.sparse_paths == imported_manifest.source.sparse_paths
        && upstream_archive_count == upstream_sparse_path_archive_count
        && upstream_scale_artifact
            .sparse_path_archive_counts
            .iter()
            .map(|entry| entry.sparse_path.clone())
            .collect::<Vec<_>>()
            == imported_manifest.source.sparse_paths
        && upstream_scale_artifact.imported_source_archive_count == scan.archive_count
        && upstream_scale_artifact.imported_source_archive_byte_match_count == scan.archive_count
        && upstream_scale_artifact.all_imported_source_archives_match_upstream;
    let all_upstream_archive_count_exceeds_imported_fixtures =
        upstream_archive_count > imported_fixture_count;
    let all_upstream_archive_count_exceeds_source_archives =
        upstream_archive_count > scan.archive_count;
    let per_push_smoke_fixture_count = seed_fixture_count + imported_fixture_count;
    let per_push_smoke_floor_fixture_count = seed_fixture_count;
    let all_smoke_floor_holds = seed_fixture_count > 0
        && seed_chunk_fixture_count == seed_fixture_count
        && per_push_smoke_fixture_count >= per_push_smoke_floor_fixture_count
        && imported_fixture_count > 0;
    let all_import_scale_checks_hold = imported_manifest.product.contains("sass-spec")
        && imported_chunk.schema_version == "0"
        && imported_chunk.product == "omena-diff-test.sass-spec-imported-corpus.chunk"
        && imported_chunk.chunk_id == "sass-spec-import-smoke"
        && imported_chunk.source_pin == imported_manifest.source.pin
        && imported_manifest.source.pin == seed_manifest.source.pin
        && !chunks.is_empty()
        && scan.succeeded
        && all_imported_counts_match_manifest
        && all_chunk_hashes_match_manifest
        && all_sparse_path_counts_match_manifest
        && scan.all_under_sparse_paths
        && all_source_archive_count_matches_imported_fixtures
        && upstream_scale_artifact_matches_manifest
        && all_upstream_archive_count_exceeds_imported_fixtures
        && all_upstream_archive_count_exceeds_source_archives
        && all_smoke_floor_holds;

    SassSpecImportScaleReportV0 {
        schema_version: "0",
        product: "omena-diff-test.sass-spec-import-scale",
        source_pin: imported_manifest.source.pin,
        source_archive_root,
        source_archive_count: scan.archive_count,
        source_archive_scan_succeeded: scan.succeeded,
        upstream_archive_count,
        upstream_scale_artifact_matches_manifest,
        imported_fixture_count,
        imported_chunk_count: chunks.len(),
        seed_fixture_count,
        per_push_smoke_fixture_count,
        per_push_smoke_floor_fixture_count,
        static_must_match_count: expectation_report.static_must_match_count,
        expected_sound_bail_count: expectation_report.expected_sound_bail_count,
        parser_recovery_count: expectation_report.parser_recovery_count,
        out_of_scope_count: expectation_report.out_of_scope_count,
        all_imported_counts_match_manifest,
        all_chunk_hashes_match_manifest,
        all_sparse_path_counts_match_manifest,
        all_source_archives_under_sparse_paths: scan.all_under_sparse_paths,
        all_source_archive_count_matches_imported_fixtures,
        all_upstream_archive_count_exceeds_imported_fixtures,
        all_upstream_archive_count_exceeds_source_archives,
        all_smoke_floor_holds,
        all_import_scale_checks_hold,
        chunks,
    }
}

fn empty_sass_spec_import_scale_report() -> SassSpecImportScaleReportV0 {
    SassSpecImportScaleReportV0 {
        schema_version: "0",
        product: "omena-diff-test.sass-spec-import-scale",
        source_pin: String::new(),
        source_archive_root: String::new(),
        source_archive_count: 0,
        source_archive_scan_succeeded: false,
        upstream_archive_count: 0,
        upstream_scale_artifact_matches_manifest: false,
        imported_fixture_count: 0,
        imported_chunk_count: 0,
        seed_fixture_count: 0,
        per_push_smoke_fixture_count: 0,
        per_push_smoke_floor_fixture_count: 0,
        static_must_match_count: 0,
        expected_sound_bail_count: 0,
        parser_recovery_count: 0,
        out_of_scope_count: 0,
        all_imported_counts_match_manifest: false,
        all_chunk_hashes_match_manifest: false,
        all_sparse_path_counts_match_manifest: false,
        all_source_archives_under_sparse_paths: false,
        all_source_archive_count_matches_imported_fixtures: false,
        all_upstream_archive_count_exceeds_imported_fixtures: false,
        all_upstream_archive_count_exceeds_source_archives: false,
        all_smoke_floor_holds: false,
        all_import_scale_checks_hold: false,
        chunks: Vec::new(),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SassSpecSourceArchiveScanV0 {
    archive_count: usize,
    succeeded: bool,
    all_under_sparse_paths: bool,
}

fn sass_spec_scan_source_archives(
    source_archive_root: &str,
    sparse_paths: &[String],
) -> SassSpecSourceArchiveScanV0 {
    let Some(repo_root) = sass_spec_repo_root() else {
        return SassSpecSourceArchiveScanV0 {
            archive_count: 0,
            succeeded: false,
            all_under_sparse_paths: false,
        };
    };
    let root = repo_root.join(source_archive_root);
    let mut archive_paths = Vec::new();
    if !sass_spec_collect_hrx_archives(&root, &root, &mut archive_paths) {
        return SassSpecSourceArchiveScanV0 {
            archive_count: 0,
            succeeded: false,
            all_under_sparse_paths: false,
        };
    }
    archive_paths.sort();
    let all_under_sparse_paths = !archive_paths.is_empty()
        && archive_paths.iter().all(|archive_path| {
            sparse_paths.iter().any(|sparse_path| {
                archive_path == sparse_path || archive_path.starts_with(&format!("{sparse_path}/"))
            })
        });
    SassSpecSourceArchiveScanV0 {
        archive_count: archive_paths.len(),
        succeeded: true,
        all_under_sparse_paths,
    }
}

fn sass_spec_repo_root() -> Option<PathBuf> {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(3)
        .map(Path::to_path_buf)
}

fn sass_spec_collect_hrx_archives(root: &Path, dir: &Path, output: &mut Vec<String>) -> bool {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return false;
    };
    for entry in entries {
        let Ok(entry) = entry else {
            return false;
        };
        let path = entry.path();
        let Ok(file_type) = entry.file_type() else {
            return false;
        };
        if file_type.is_dir() {
            if !sass_spec_collect_hrx_archives(root, &path, output) {
                return false;
            }
        } else if path.extension().and_then(|extension| extension.to_str()) == Some("hrx") {
            let Ok(relative) = path.strip_prefix(root) else {
                return false;
            };
            output.push(relative.to_string_lossy().replace('\\', "/"));
        }
    }
    true
}

fn sass_spec_imported_chunk_source_for_path(path: &str) -> Option<&'static str> {
    match path {
        "imported-smoke.json" => Some(SASS_SPEC_IMPORTED_CHUNK_SOURCE),
        _ => None,
    }
}

fn diff_test_sha256_hex(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};

    const HEX: &[u8; 16] = b"0123456789abcdef";
    let digest = Sha256::digest(bytes);
    let mut output = String::with_capacity(digest.len() * 2);
    for byte in digest {
        output.push(char::from(HEX[usize::from(byte >> 4)]));
        output.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    output
}

pub fn summarize_sass_spec_expectation_buckets() -> SassSpecExpectationBucketReportV0 {
    use external_corpus_envelope_idl_generated::ExternalCorpusExpectationKindV1Json;

    let chunk =
        match serde_json::from_str::<ImportedSassSpecChunkV0>(SASS_SPEC_IMPORTED_CHUNK_SOURCE) {
            Ok(chunk) => chunk,
            Err(error) => {
                return SassSpecExpectationBucketReportV0 {
                    schema_version: "0",
                    product: "omena-diff-test.sass-spec-expectation-buckets",
                    fixture_count: 0,
                    static_must_match_count: 0,
                    expected_sound_bail_count: 0,
                    parser_recovery_count: 0,
                    out_of_scope_count: 0,
                    missing_expectation_kind_fixture_ids: vec![format!("parse-error:{error}")],
                    criteria_mismatch_fixture_ids: Vec::new(),
                    all_fixtures_have_one_expectation_kind: false,
                    all_expectation_kinds_match_criteria: false,
                };
            }
        };
    let mut static_must_match_count = 0;
    let mut expected_sound_bail_count = 0;
    let mut parser_recovery_count = 0;
    let mut out_of_scope_count = 0;
    let mut missing_expectation_kind_fixture_ids = Vec::new();
    let mut criteria_mismatch_fixture_ids = Vec::new();

    for fixture in &chunk.fixtures {
        let expected_kind = infer_sass_spec_expectation_kind(fixture);
        match fixture.expectation_kind.as_ref() {
            Some(kind @ ExternalCorpusExpectationKindV1Json::StaticMustMatch) => {
                static_must_match_count += 1;
                if kind != &expected_kind {
                    criteria_mismatch_fixture_ids.push(fixture.id.clone());
                }
            }
            Some(kind @ ExternalCorpusExpectationKindV1Json::ExpectedSoundBail) => {
                expected_sound_bail_count += 1;
                if kind != &expected_kind {
                    criteria_mismatch_fixture_ids.push(fixture.id.clone());
                }
            }
            Some(kind @ ExternalCorpusExpectationKindV1Json::ParserRecovery) => {
                parser_recovery_count += 1;
                if kind != &expected_kind {
                    criteria_mismatch_fixture_ids.push(fixture.id.clone());
                }
            }
            Some(kind @ ExternalCorpusExpectationKindV1Json::OutOfScope) => {
                out_of_scope_count += 1;
                if kind != &expected_kind {
                    criteria_mismatch_fixture_ids.push(fixture.id.clone());
                }
            }
            None => missing_expectation_kind_fixture_ids.push(fixture.id.clone()),
        }
    }

    let bucket_union_count = static_must_match_count
        + expected_sound_bail_count
        + parser_recovery_count
        + out_of_scope_count;

    SassSpecExpectationBucketReportV0 {
        schema_version: "0",
        product: "omena-diff-test.sass-spec-expectation-buckets",
        fixture_count: chunk.fixtures.len(),
        static_must_match_count,
        expected_sound_bail_count,
        parser_recovery_count,
        out_of_scope_count,
        missing_expectation_kind_fixture_ids,
        criteria_mismatch_fixture_ids: criteria_mismatch_fixture_ids.clone(),
        all_fixtures_have_one_expectation_kind: bucket_union_count == chunk.fixtures.len(),
        all_expectation_kinds_match_criteria: criteria_mismatch_fixture_ids.is_empty(),
    }
}

fn infer_sass_spec_expectation_kind(
    fixture: &ImportedSassSpecFixtureV0,
) -> external_corpus_envelope_idl_generated::ExternalCorpusExpectationKindV1Json {
    use external_corpus_envelope_idl_generated::ExternalCorpusExpectationKindV1Json;

    if fixture.upstream_path.contains("/non_conformant/")
        || fixture.upstream_path.contains("libsass-todo-")
        || sass_spec_fixture_requires_external_suite_layout(&fixture.source)
    {
        return ExternalCorpusExpectationKindV1Json::OutOfScope;
    }
    if fixture.expected_error.is_some() || fixture.expected_warning.is_some() {
        return ExternalCorpusExpectationKindV1Json::ParserRecovery;
    }

    let Some(dialect) = sass_spec_fixture_dialect(fixture.dialect.as_str()) else {
        return ExternalCorpusExpectationKindV1Json::ExpectedSoundBail;
    };
    let Some(resolution) = summarize_static_stylesheet_value_resolution(&fixture.source, dialect)
    else {
        return ExternalCorpusExpectationKindV1Json::ExpectedSoundBail;
    };

    if resolution.fuel_exhausted_count > 0 || resolution.unsupported_dynamic_count > 0 {
        return ExternalCorpusExpectationKindV1Json::ExpectedSoundBail;
    }
    if fixture.expected_css.is_some()
        && resolution.raw_count == 0
        && resolution.top_count == 0
        && resolution.cycle_count == 0
        && resolution.unresolved_reference_count == 0
    {
        return ExternalCorpusExpectationKindV1Json::StaticMustMatch;
    }
    ExternalCorpusExpectationKindV1Json::ExpectedSoundBail
}

fn sass_spec_fixture_requires_external_suite_layout(source: &str) -> bool {
    source.lines().any(|line| {
        let trimmed = line.trim_start();
        (trimmed.starts_with("@use") || trimmed.starts_with("@import"))
            && (trimmed.contains("\"../") || trimmed.contains("'../"))
    })
}

fn sass_spec_fixture_dialect(dialect: &str) -> Option<StyleDialect> {
    match dialect {
        "scss" => Some(StyleDialect::Scss),
        "sass" => Some(StyleDialect::Sass),
        _ => None,
    }
}

/// Summarize imported sass-spec sound-bail membership checks.
pub fn summarize_sass_spec_sound_bail_membership() -> SassSpecSoundBailMembershipReportV0 {
    use external_corpus_envelope_idl_generated::ExternalCorpusExpectationKindV1Json;

    let chunk =
        match serde_json::from_str::<ImportedSassSpecChunkV0>(SASS_SPEC_IMPORTED_CHUNK_SOURCE) {
            Ok(chunk) => chunk,
            Err(_) => return empty_sass_spec_sound_bail_membership_report(),
        };
    let capture = match serde_json::from_str::<SassSpecOracleCaptureV0>(
        SASS_SPEC_IMPORTED_ORACLE_CAPTURE_SOURCE,
    ) {
        Ok(capture) => capture,
        Err(_) => return empty_sass_spec_sound_bail_membership_report(),
    };
    let oracle_by_fixture_id: BTreeMap<&str, &SassSpecOracleRecordV0> = capture
        .records
        .iter()
        .map(|record| (record.fixture_id.as_str(), record))
        .collect();
    let sound_bail_fixtures: Vec<_> = chunk
        .fixtures
        .iter()
        .filter(|fixture| {
            fixture.expectation_kind.as_ref()
                == Some(&ExternalCorpusExpectationKindV1Json::ExpectedSoundBail)
        })
        .collect();
    let mut checked_fixture_ids = BTreeSet::new();
    let mut records = Vec::new();

    for fixture in &sound_bail_fixtures {
        let Some(record) = oracle_by_fixture_id.get(fixture.id.as_str()) else {
            continue;
        };
        if !record.compiled {
            continue;
        }
        let Some(dialect) = sass_spec_fixture_dialect(fixture.dialect.as_str()) else {
            continue;
        };
        let Some(resolution) =
            summarize_static_stylesheet_value_resolution(&fixture.source, dialect)
        else {
            continue;
        };
        let bail_values: Vec<_> = resolution
            .values
            .iter()
            .filter(|value| value.reason == "unsupportedDynamic" || value.reason == "fuelExhausted")
            .collect();
        let Some(value_pairs) = record.declaration_value_pairs.as_ref() else {
            continue;
        };

        for pair in value_pairs {
            let Some(value) = bail_values
                .iter()
                .copied()
                .find(|value| {
                    abstract_css_value_contains_concrete(&value.abstract_value, &pair.value)
                })
                .or(match bail_values.as_slice() {
                    [value, ..] => Some(*value),
                    [] => None,
                })
            else {
                continue;
            };
            checked_fixture_ids.insert(fixture.id.clone());
            let concrete_in_abstract_value =
                abstract_css_value_contains_concrete(&value.abstract_value, &pair.value);
            let weakening_preserves_membership =
                sass_spec_weakening_preserves_membership(pair.value.as_str());
            let exact_tightness_holds = sass_spec_exact_tightness_holds(pair.value.as_str());
            let raw_tightness_case = matches!(value.abstract_value, AbstractCssValueV0::Raw { .. })
                && concrete_in_abstract_value
                && exact_tightness_holds;
            records.push(SassSpecSoundBailMembershipCaseReportV0 {
                fixture_id: fixture.id.clone(),
                property: pair.property.clone(),
                concrete_value: pair.value.clone(),
                abstract_value_kind: value.abstract_value_kind.to_string(),
                reason: value.reason.to_string(),
                concrete_in_abstract_value,
                weakening_preserves_membership,
                exact_tightness_holds,
                raw_tightness_case,
            });
        }
    }

    let non_top_case_count = records
        .iter()
        .filter(|record| record.abstract_value_kind != "top")
        .count();
    let raw_tightness_case_count = records
        .iter()
        .filter(|record| record.raw_tightness_case)
        .count();
    let all_concrete_values_in_abstract_values = !records.is_empty()
        && records
            .iter()
            .all(|record| record.concrete_in_abstract_value);
    let weakening_preserves_membership = !records.is_empty()
        && records
            .iter()
            .all(|record| record.weakening_preserves_membership);
    let exact_tightness_holds =
        !records.is_empty() && records.iter().all(|record| record.exact_tightness_holds);
    let case_count = sound_bail_fixtures.len();
    let checked_case_count = checked_fixture_ids.len();
    let all_membership_checks_hold = case_count > 0
        && checked_case_count == case_count
        && non_top_case_count > 0
        && raw_tightness_case_count > 0
        && all_concrete_values_in_abstract_values
        && weakening_preserves_membership
        && exact_tightness_holds;

    SassSpecSoundBailMembershipReportV0 {
        schema_version: "0",
        product: "omena-diff-test.sass-spec-sound-bail-membership",
        case_count,
        checked_case_count,
        non_top_case_count,
        raw_tightness_case_count,
        all_concrete_values_in_abstract_values,
        weakening_preserves_membership,
        exact_tightness_holds,
        all_membership_checks_hold,
        records,
    }
}

/// Summarize imported sass-spec static equality checks.
pub fn summarize_sass_spec_static_must_match() -> SassSpecStaticMustMatchReportV0 {
    use external_corpus_envelope_idl_generated::ExternalCorpusExpectationKindV1Json;

    let chunk =
        match serde_json::from_str::<ImportedSassSpecChunkV0>(SASS_SPEC_IMPORTED_CHUNK_SOURCE) {
            Ok(chunk) => chunk,
            Err(_) => return empty_sass_spec_static_must_match_report(),
        };
    let capture = match serde_json::from_str::<SassSpecOracleCaptureV0>(
        SASS_SPEC_IMPORTED_ORACLE_CAPTURE_SOURCE,
    ) {
        Ok(capture) => capture,
        Err(_) => return empty_sass_spec_static_must_match_report(),
    };
    let oracle_by_fixture_id: BTreeMap<&str, &SassSpecOracleRecordV0> = capture
        .records
        .iter()
        .map(|record| (record.fixture_id.as_str(), record))
        .collect();
    let static_fixtures: Vec<_> = chunk
        .fixtures
        .iter()
        .filter(|fixture| {
            fixture.expectation_kind.as_ref()
                == Some(&ExternalCorpusExpectationKindV1Json::StaticMustMatch)
        })
        .collect();
    let mut checked_fixture_ids = BTreeSet::new();
    let mut records = Vec::new();

    for fixture in &static_fixtures {
        let Some(record) = oracle_by_fixture_id.get(fixture.id.as_str()) else {
            continue;
        };
        if !record.compiled {
            continue;
        }
        let Some(value_pairs) = record.declaration_value_pairs.as_ref() else {
            continue;
        };
        let Some(dialect) = sass_spec_fixture_dialect(fixture.dialect.as_str()) else {
            continue;
        };
        let Some(resolution) =
            summarize_static_stylesheet_value_resolution(&fixture.source, dialect)
        else {
            continue;
        };
        let omena_rendered_values: Vec<_> = resolution
            .values
            .iter()
            .filter_map(|value| value.rendered_value.as_deref())
            .map(normalize_sass_spec_css_value)
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();
        checked_fixture_ids.insert(fixture.id.clone());

        for pair in value_pairs {
            let concrete_value = normalize_sass_spec_css_value(pair.value.as_str());
            records.push(SassSpecStaticMustMatchCaseReportV0 {
                fixture_id: fixture.id.clone(),
                property: pair.property.clone(),
                matched_by_omena_resolution: omena_rendered_values
                    .iter()
                    .any(|value| sass_spec_css_values_match(value, &concrete_value)),
                concrete_value,
                omena_rendered_values: omena_rendered_values.clone(),
            });
        }
    }

    let case_count = static_fixtures.len();
    let checked_case_count = checked_fixture_ids.len();
    let declaration_value_count = records.len();
    let matched_declaration_value_count = records
        .iter()
        .filter(|record| record.matched_by_omena_resolution)
        .count();
    let all_static_values_match_oracle = !records.is_empty()
        && records
            .iter()
            .all(|record| record.matched_by_omena_resolution);
    let all_static_match_checks_hold = case_count > 0
        && checked_case_count == case_count
        && declaration_value_count > 0
        && matched_declaration_value_count == declaration_value_count
        && all_static_values_match_oracle;

    SassSpecStaticMustMatchReportV0 {
        schema_version: "0",
        product: "omena-diff-test.sass-spec-static-match",
        case_count,
        checked_case_count,
        declaration_value_count,
        matched_declaration_value_count,
        all_static_values_match_oracle,
        all_static_match_checks_hold,
        records,
    }
}

fn empty_sass_spec_static_must_match_report() -> SassSpecStaticMustMatchReportV0 {
    SassSpecStaticMustMatchReportV0 {
        schema_version: "0",
        product: "omena-diff-test.sass-spec-static-match",
        case_count: 0,
        checked_case_count: 0,
        declaration_value_count: 0,
        matched_declaration_value_count: 0,
        all_static_values_match_oracle: false,
        all_static_match_checks_hold: false,
        records: Vec::new(),
    }
}

fn normalize_sass_spec_css_value(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn sass_spec_css_values_match(left: &str, right: &str) -> bool {
    let normalized_left = normalize_sass_spec_css_value(left);
    let normalized_right = normalize_sass_spec_css_value(right);
    normalized_left == normalized_right
        || sass_spec_unquoted_string_value(normalized_left.as_str())
            .zip(sass_spec_unquoted_string_value(normalized_right.as_str()))
            .is_some_and(|(left, right)| left == right)
        || abstract_css_values_canonically_equal(&normalized_left, &normalized_right)
}

fn sass_spec_unquoted_string_value(value: &str) -> Option<&str> {
    let bytes = value.as_bytes();
    if bytes.len() < 2 {
        return None;
    }
    let quote = bytes[0];
    if (quote == b'\'' || quote == b'"') && bytes.last().copied() == Some(quote) {
        return value.get(1..value.len() - 1);
    }
    None
}

fn empty_sass_spec_sound_bail_membership_report() -> SassSpecSoundBailMembershipReportV0 {
    SassSpecSoundBailMembershipReportV0 {
        schema_version: "0",
        product: "omena-diff-test.sass-spec-sound-bail-membership",
        case_count: 0,
        checked_case_count: 0,
        non_top_case_count: 0,
        raw_tightness_case_count: 0,
        all_concrete_values_in_abstract_values: false,
        weakening_preserves_membership: false,
        exact_tightness_holds: false,
        all_membership_checks_hold: false,
        records: Vec::new(),
    }
}

fn abstract_css_value_contains_concrete(value: &AbstractCssValueV0, concrete: &str) -> bool {
    match value {
        AbstractCssValueV0::Bottom => false,
        AbstractCssValueV0::Exact { value, .. } => sass_spec_css_values_match(value, concrete),
        AbstractCssValueV0::Raw { value } => {
            sass_spec_css_values_match(value, concrete)
                || sass_spec_raw_value_is_preserved_in_concrete(value, concrete)
        }
        AbstractCssValueV0::FiniteSet { values, .. } => values
            .iter()
            .any(|value| sass_spec_css_values_match(value, concrete)),
        AbstractCssValueV0::Top => true,
    }
}

fn sass_spec_raw_value_is_preserved_in_concrete(raw_value: &str, concrete: &str) -> bool {
    let raw_value = normalize_sass_spec_css_value(raw_value);
    let concrete = normalize_sass_spec_css_value(concrete);
    !raw_value.is_empty() && concrete.contains(raw_value.as_str())
}

fn sass_spec_weakening_preserves_membership(concrete: &str) -> bool {
    let exact = AbstractCssValueV0::Exact {
        value: concrete.to_string(),
        typed: None,
    };
    let finite_set = AbstractCssValueV0::FiniteSet {
        values: vec![concrete.to_string(), format!("{concrete}__alternate")],
        typed: None,
    };
    abstract_css_value_contains_concrete(&exact, concrete)
        && abstract_css_value_contains_concrete(&finite_set, concrete)
        && abstract_css_value_contains_concrete(&AbstractCssValueV0::Top, concrete)
}

fn sass_spec_exact_tightness_holds(concrete: &str) -> bool {
    let exact = AbstractCssValueV0::Exact {
        value: concrete.to_string(),
        typed: None,
    };
    abstract_css_value_contains_concrete(&exact, concrete)
        && !abstract_css_value_contains_concrete(&exact, &format!("{concrete}__different"))
}

/// Summarize imported sass-spec expectation bucket ledger consistency.
pub fn summarize_sass_spec_expectation_bucket_ledger() -> SassSpecExpectationBucketLedgerReportV0 {
    use external_corpus_envelope_idl_generated::ExternalCorpusExpectationKindV1Json;

    let current_report = summarize_sass_spec_expectation_buckets();
    let chunk =
        match serde_json::from_str::<ImportedSassSpecChunkV0>(SASS_SPEC_IMPORTED_CHUNK_SOURCE) {
            Ok(chunk) => chunk,
            Err(_) => return empty_sass_spec_expectation_bucket_ledger_report(current_report),
        };
    let ledger = match toml::from_str::<SassSpecExpectationBucketLedgerTomlV0>(
        SASS_SPEC_EXPECTATION_BUCKET_LEDGER_SOURCE,
    ) {
        Ok(ledger) => ledger,
        Err(_) => return empty_sass_spec_expectation_bucket_ledger_report(current_report),
    };
    let current_by_fixture_id: BTreeMap<String, ExternalCorpusExpectationKindV1Json> = chunk
        .fixtures
        .iter()
        .filter_map(|fixture| {
            fixture
                .expectation_kind
                .clone()
                .map(|kind| (fixture.id.clone(), kind))
        })
        .collect();
    let ledger_by_fixture_id: BTreeMap<String, ExternalCorpusExpectationKindV1Json> = ledger
        .fixture
        .iter()
        .map(|fixture| (fixture.id.clone(), fixture.expectation_kind.clone()))
        .collect();

    let missing_ledger_fixture_ids = current_by_fixture_id
        .keys()
        .filter(|id| !ledger_by_fixture_id.contains_key(*id))
        .cloned()
        .collect::<Vec<_>>();
    let stale_ledger_fixture_ids = ledger_by_fixture_id
        .keys()
        .filter(|id| !current_by_fixture_id.contains_key(*id))
        .cloned()
        .collect::<Vec<_>>();
    let assignment_mismatch_fixture_ids = current_by_fixture_id
        .iter()
        .filter_map(|(id, current_kind)| {
            let ledger_kind = ledger_by_fixture_id.get(id)?;
            (ledger_kind != current_kind).then(|| id.clone())
        })
        .collect::<Vec<_>>();
    let all_bucket_totals_match_ledger = current_report.static_must_match_count
        == ledger.static_must_match_count
        && current_report.expected_sound_bail_count == ledger.expected_sound_bail_count
        && current_report.parser_recovery_count == ledger.parser_recovery_count
        && current_report.out_of_scope_count == ledger.out_of_scope_count;
    let all_fixture_assignments_match_ledger = missing_ledger_fixture_ids.is_empty()
        && stale_ledger_fixture_ids.is_empty()
        && assignment_mismatch_fixture_ids.is_empty();
    let all_reclassification_entries_have_rationale = ledger.reclassification.iter().all(|entry| {
        !entry.fixture_id.is_empty()
            && entry.from != entry.to
            && !entry.reason.is_empty()
            && !entry.since.is_empty()
            && !entry.review_after.is_empty()
    });
    let ledger_metadata_valid = ledger.schema_version == "0"
        && ledger.corpus_chunk
            == "rust/crates/omena-diff-test/sass-spec-corpus/imported-smoke.json";

    SassSpecExpectationBucketLedgerReportV0 {
        schema_version: "0",
        product: "omena-diff-test.sass-spec-expectation-bucket-ledger",
        fixture_count: current_report.fixture_count,
        current_static_must_match_count: current_report.static_must_match_count,
        current_expected_sound_bail_count: current_report.expected_sound_bail_count,
        current_parser_recovery_count: current_report.parser_recovery_count,
        current_out_of_scope_count: current_report.out_of_scope_count,
        ledger_static_must_match_count: ledger.static_must_match_count,
        ledger_expected_sound_bail_count: ledger.expected_sound_bail_count,
        ledger_parser_recovery_count: ledger.parser_recovery_count,
        ledger_out_of_scope_count: ledger.out_of_scope_count,
        all_bucket_totals_match_ledger,
        all_fixture_assignments_match_ledger,
        all_reclassification_entries_have_rationale,
        ledger_metadata_valid,
        missing_ledger_fixture_ids,
        stale_ledger_fixture_ids,
        assignment_mismatch_fixture_ids,
    }
}

fn empty_sass_spec_expectation_bucket_ledger_report(
    current_report: SassSpecExpectationBucketReportV0,
) -> SassSpecExpectationBucketLedgerReportV0 {
    SassSpecExpectationBucketLedgerReportV0 {
        schema_version: "0",
        product: "omena-diff-test.sass-spec-expectation-bucket-ledger",
        fixture_count: current_report.fixture_count,
        current_static_must_match_count: current_report.static_must_match_count,
        current_expected_sound_bail_count: current_report.expected_sound_bail_count,
        current_parser_recovery_count: current_report.parser_recovery_count,
        current_out_of_scope_count: current_report.out_of_scope_count,
        ledger_static_must_match_count: 0,
        ledger_expected_sound_bail_count: 0,
        ledger_parser_recovery_count: 0,
        ledger_out_of_scope_count: 0,
        all_bucket_totals_match_ledger: false,
        all_fixture_assignments_match_ledger: false,
        all_reclassification_entries_have_rationale: false,
        ledger_metadata_valid: false,
        missing_ledger_fixture_ids: Vec::new(),
        stale_ledger_fixture_ids: Vec::new(),
        assignment_mismatch_fixture_ids: Vec::new(),
    }
}

/// Summarize imported sass-spec bail-site ledger consistency.
pub fn summarize_sass_spec_bail_site_ledger() -> SassSpecBailSiteLedgerReportV0 {
    use external_corpus_envelope_idl_generated::ExternalCorpusExpectationKindV1Json;

    let current_sites = sass_spec_current_bail_site_census();
    let raw_pattern_hit_count = sass_spec_raw_bail_pattern_hit_count();
    let non_semantic_pattern_hit_count = raw_pattern_hit_count.saturating_sub(current_sites.len());
    let empty_report = || {
        empty_sass_spec_bail_site_ledger_report(
            current_sites.len(),
            raw_pattern_hit_count,
            non_semantic_pattern_hit_count,
        )
    };
    let ledger =
        match toml::from_str::<SassSpecBailSiteLedgerTomlV0>(SASS_SPEC_BAIL_SITE_LEDGER_SOURCE) {
            Ok(ledger) => ledger,
            Err(_) => return empty_report(),
        };
    let chunk =
        match serde_json::from_str::<ImportedSassSpecChunkV0>(SASS_SPEC_IMPORTED_CHUNK_SOURCE) {
            Ok(chunk) => chunk,
            Err(_) => return empty_report(),
        };
    let sound_bail_reason_by_fixture_id: BTreeMap<String, String> =
        summarize_sass_spec_sound_bail_membership()
            .records
            .into_iter()
            .map(|record| (record.fixture_id, record.reason))
            .collect();
    let imported_sound_bail_fixture_ids: BTreeSet<String> = chunk
        .fixtures
        .into_iter()
        .filter(|fixture| {
            fixture.expectation_kind.as_ref()
                == Some(&ExternalCorpusExpectationKindV1Json::ExpectedSoundBail)
        })
        .map(|fixture| fixture.id)
        .collect();
    let current_by_key: BTreeMap<String, SassSpecBailSiteCensusRecordV0> = current_sites
        .into_iter()
        .map(|site| {
            (
                sass_spec_bail_site_key(&site.file, site.ordinal, &site.reason),
                site,
            )
        })
        .collect();
    let ledger_by_key: BTreeMap<String, &SassSpecBailSiteLedgerSiteTomlV0> = ledger
        .site
        .iter()
        .map(|site| {
            (
                sass_spec_bail_site_key(&site.file, site.ordinal, &site.reason),
                site,
            )
        })
        .collect();

    let missing_ledger_site_keys = current_by_key
        .keys()
        .filter(|key| !ledger_by_key.contains_key(*key))
        .cloned()
        .collect::<Vec<_>>();
    let stale_ledger_site_keys = ledger_by_key
        .keys()
        .filter(|key| !current_by_key.contains_key(*key))
        .cloned()
        .collect::<Vec<_>>();
    let gapless_site_keys = ledger
        .site
        .iter()
        .filter(|site| site.linked_fixture_ids.is_empty() && !sass_spec_has_named_gap(site))
        .map(|site| sass_spec_bail_site_key(&site.file, site.ordinal, &site.reason))
        .collect::<Vec<_>>();
    let mut linked_fixture_ids = BTreeSet::new();
    let mut unknown_link_fixture_ids = BTreeSet::new();
    let mut reason_mismatch_link_keys = Vec::new();

    for site in &ledger.site {
        let site_key = sass_spec_bail_site_key(&site.file, site.ordinal, &site.reason);
        for fixture_id in &site.linked_fixture_ids {
            linked_fixture_ids.insert(fixture_id.clone());
            if !imported_sound_bail_fixture_ids.contains(fixture_id) {
                unknown_link_fixture_ids.insert(fixture_id.clone());
                continue;
            }
            match sound_bail_reason_by_fixture_id.get(fixture_id) {
                Some(reason) if reason == &site.reason => {}
                Some(_) | None => {
                    reason_mismatch_link_keys.push(format!("{site_key}->{fixture_id}"));
                }
            }
        }
    }

    let linked_site_count = ledger
        .site
        .iter()
        .filter(|site| !site.linked_fixture_ids.is_empty())
        .count();
    let named_gap_site_count = ledger
        .site
        .iter()
        .filter(|site| sass_spec_has_named_gap(site))
        .count();
    let ledger_metadata_valid = ledger.schema_version == "0"
        && ledger.source_root == "rust/crates/omena-scss-eval/src"
        && ledger.semantic_site_count == current_by_key.len()
        && ledger.raw_pattern_hit_count == raw_pattern_hit_count
        && ledger.non_semantic_pattern_hit_count == non_semantic_pattern_hit_count;
    let all_semantic_sites_match_ledger = ledger_metadata_valid
        && missing_ledger_site_keys.is_empty()
        && stale_ledger_site_keys.is_empty()
        && ledger.site.len() == current_by_key.len();
    let all_sites_linked_or_named_gap = gapless_site_keys.is_empty();
    let unknown_link_fixture_ids = unknown_link_fixture_ids.into_iter().collect::<Vec<_>>();
    let all_linked_cases_are_imported_sound_bail_cases = unknown_link_fixture_ids.is_empty();
    let all_linked_cases_match_reason_class = reason_mismatch_link_keys.is_empty();
    let all_bail_site_ledger_checks_hold = all_semantic_sites_match_ledger
        && all_sites_linked_or_named_gap
        && all_linked_cases_are_imported_sound_bail_cases
        && all_linked_cases_match_reason_class
        && linked_site_count > 0
        && !linked_fixture_ids.is_empty();
    let records = ledger
        .site
        .iter()
        .map(|site| {
            let key = sass_spec_bail_site_key(&site.file, site.ordinal, &site.reason);
            let current = current_by_key.get(&key);
            SassSpecBailSiteLedgerRecordReportV0 {
                file: site.file.clone(),
                ordinal: site.ordinal,
                reason: site.reason.clone(),
                current_line: current.map(|site| site.line),
                ledger_line_hint: site.line_hint,
                present_in_current_sources: current.is_some(),
                linked_fixture_ids: site.linked_fixture_ids.clone(),
                gap: site.gap.clone(),
            }
        })
        .collect();

    SassSpecBailSiteLedgerReportV0 {
        schema_version: "0",
        product: "omena-diff-test.sass-spec-bail-site-ledger",
        semantic_site_count: current_by_key.len(),
        ledger_semantic_site_count: ledger.semantic_site_count,
        raw_pattern_hit_count,
        non_semantic_pattern_hit_count,
        linked_site_count,
        named_gap_site_count,
        linked_case_count: linked_fixture_ids.len(),
        ledger_metadata_valid,
        all_semantic_sites_match_ledger,
        all_sites_linked_or_named_gap,
        all_linked_cases_match_reason_class,
        all_linked_cases_are_imported_sound_bail_cases,
        all_bail_site_ledger_checks_hold,
        missing_ledger_site_keys,
        stale_ledger_site_keys,
        gapless_site_keys,
        unknown_link_fixture_ids,
        reason_mismatch_link_keys,
        records,
    }
}

fn empty_sass_spec_bail_site_ledger_report(
    semantic_site_count: usize,
    raw_pattern_hit_count: usize,
    non_semantic_pattern_hit_count: usize,
) -> SassSpecBailSiteLedgerReportV0 {
    SassSpecBailSiteLedgerReportV0 {
        schema_version: "0",
        product: "omena-diff-test.sass-spec-bail-site-ledger",
        semantic_site_count,
        ledger_semantic_site_count: 0,
        raw_pattern_hit_count,
        non_semantic_pattern_hit_count,
        linked_site_count: 0,
        named_gap_site_count: 0,
        linked_case_count: 0,
        ledger_metadata_valid: false,
        all_semantic_sites_match_ledger: false,
        all_sites_linked_or_named_gap: false,
        all_linked_cases_match_reason_class: false,
        all_linked_cases_are_imported_sound_bail_cases: false,
        all_bail_site_ledger_checks_hold: false,
        missing_ledger_site_keys: Vec::new(),
        stale_ledger_site_keys: Vec::new(),
        gapless_site_keys: Vec::new(),
        unknown_link_fixture_ids: Vec::new(),
        reason_mismatch_link_keys: Vec::new(),
        records: Vec::new(),
    }
}

fn sass_spec_current_bail_site_census() -> Vec<SassSpecBailSiteCensusRecordV0> {
    let mut records = Vec::new();
    for source_file in SASS_SPEC_BAIL_SITE_SOURCE_FILES
        .iter()
        .filter(|source_file| source_file.include_in_semantic_census)
    {
        let mut ordinal_by_reason = BTreeMap::<String, usize>::new();
        for (line_index, line) in source_file.source.lines().enumerate() {
            let Some(reason) = sass_spec_semantic_bail_reason_from_line(line) else {
                continue;
            };
            let ordinal = ordinal_by_reason.entry(reason.clone()).or_default();
            *ordinal += 1;
            records.push(SassSpecBailSiteCensusRecordV0 {
                file: source_file.file.to_string(),
                ordinal: *ordinal,
                reason,
                line: line_index + 1,
            });
        }
    }
    records
}

fn sass_spec_semantic_bail_reason_from_line(line: &str) -> Option<String> {
    if line.contains("StaticStylesheetResolutionReason::UnsupportedDynamic")
        && !line.contains("Self::UnsupportedDynamic")
    {
        Some("unsupportedDynamic".to_string())
    } else if line.contains("StaticStylesheetResolutionReason::FuelExhausted") {
        Some("fuelExhausted".to_string())
    } else {
        None
    }
}

fn sass_spec_raw_bail_pattern_hit_count() -> usize {
    SASS_SPEC_BAIL_SITE_SOURCE_FILES
        .iter()
        .map(|source_file| {
            source_file
                .source
                .lines()
                .filter(|line| {
                    line.contains("UnsupportedDynamic")
                        || line.contains("FuelExhausted")
                        || line.contains("::Unsupported")
                })
                .count()
        })
        .sum()
}

fn sass_spec_bail_site_key(file: &str, ordinal: usize, reason: &str) -> String {
    format!("{file}#{ordinal}:{reason}")
}

fn sass_spec_has_named_gap(site: &SassSpecBailSiteLedgerSiteTomlV0) -> bool {
    site.gap.as_ref().is_some_and(|gap| !gap.trim().is_empty())
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

#[cfg(test)]
fn template_placeholder_default_none_snapshot(
    fixture: TemplatePlaceholderDefaultNoneFixtureV0,
) -> (u64, u64) {
    let lexed = omena_parser::lex(fixture.source, fixture.dialect);
    let parsed = parse(fixture.source, fixture.dialect);
    let mut token_snapshot = String::new();
    for token in lexed.tokens() {
        token_snapshot.push_str(
            format!(
                "{}:{}:{}:{}\n",
                token.kind.as_u32(),
                u32::from(token.range.start()),
                u32::from(token.range.end()),
                token.text
            )
            .as_str(),
        );
    }
    token_snapshot.push_str(format!("errors={:?}\n", lexed.errors()).as_str());
    let syntax = parsed.syntax();
    let mut syntax_snapshot = format!(
        "dialect={:?}\nerrors={:?}\n",
        parsed.dialect(),
        parsed.errors()
    );
    let root_range = syntax.text_range();
    syntax_snapshot.push_str(
        format!(
            "{}:{}:{}\n",
            syntax.kind().as_u32(),
            u32::from(root_range.start()),
            u32::from(root_range.end()),
        )
        .as_str(),
    );
    for node in syntax.descendants() {
        let range = node.text_range();
        syntax_snapshot.push_str(
            format!(
                "{}:{}:{}\n",
                node.kind().as_u32(),
                u32::from(range.start()),
                u32::from(range.end()),
            )
            .as_str(),
        );
    }

    (
        stable_fnv1a64(token_snapshot.as_bytes()),
        stable_fnv1a64(syntax_snapshot.as_bytes()),
    )
}

#[cfg(test)]
fn stable_fnv1a64(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

#[cfg(test)]
fn conformance_seed_fixtures(
    chunk_sources: &[&str],
    default_dialect: &str,
) -> Result<Vec<DialectSeedFixtureV0>, serde_json::Error> {
    let mut fixtures = Vec::new();
    for source in chunk_sources {
        let chunk = serde_json::from_str::<DialectSeedChunkV0>(source)?;
        fixtures.extend(chunk.fixtures.into_iter().map(|mut fixture| {
            if fixture.dialect.is_empty() {
                fixture.dialect = default_dialect.to_string();
            }
            fixture.expected_bogus_kinds = sorted_owned(fixture.expected_bogus_kinds);
            fixture.expected_error_codes = sorted_owned(fixture.expected_error_codes);
            fixture
        }));
    }
    Ok(fixtures)
}

#[cfg(test)]
fn dialect_from_seed_fixture(value: &str) -> Option<StyleDialect> {
    match value {
        "css" => Some(StyleDialect::Css),
        "scss" => Some(StyleDialect::Scss),
        "sass" => Some(StyleDialect::Sass),
        "less" => Some(StyleDialect::Less),
        _ => None,
    }
}

#[cfg(test)]
fn actual_bogus_kinds(source: &str, dialect: StyleDialect) -> Vec<String> {
    let parsed = parse(source, dialect);
    sorted_owned(
        parsed
            .syntax()
            .descendants()
            .filter(|node| node.kind().is_bogus())
            .map(|node| format!("{:?}", node.kind()))
            .collect(),
    )
}

#[cfg(test)]
fn actual_error_codes(source: &str, dialect: StyleDialect) -> Vec<String> {
    let parsed = parse(source, dialect);
    sorted_owned(
        parsed
            .errors()
            .iter()
            .map(|error| format!("{:?}", error.code))
            .collect(),
    )
}

#[cfg(test)]
fn seed_policy_known_failure_subtests(source: &str) -> BTreeSet<String> {
    let mut subtests = BTreeSet::new();
    let mut fixture: Option<String> = None;
    let mut name: Option<String> = None;
    let mut in_subtest = false;

    for raw_line in source.lines() {
        let line = raw_line.split('#').next().unwrap_or("").trim();
        if line == "[[subtest]]" {
            if let (Some(fixture), Some(name)) = (fixture.take(), name.take()) {
                subtests.insert(format!("{fixture}\n{name}"));
            }
            in_subtest = true;
            continue;
        }
        if line.starts_with("[[") {
            if let (Some(fixture), Some(name)) = (fixture.take(), name.take()) {
                subtests.insert(format!("{fixture}\n{name}"));
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
            "fixture" => fixture = toml_string_literal(value.trim()).map(str::to_string),
            "name" => name = toml_string_literal(value.trim()).map(str::to_string),
            _ => {}
        }
    }
    if let (Some(fixture), Some(name)) = (fixture, name) {
        subtests.insert(format!("{fixture}\n{name}"));
    }

    subtests
}

#[cfg(test)]
fn toml_string_literal(value: &str) -> Option<&str> {
    value.strip_prefix('"')?.strip_suffix('"')
}

#[cfg(test)]
fn sorted_owned(values: Vec<String>) -> Vec<String> {
    values
        .into_iter()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
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

    const PIPELINE_MODULE_STRUCTURAL_INTERPASS_SOURCE: &str = r#"
@import "./tokens.css";
.card {
  composes: base utility;
  --local-tone: var(--pkg-brand);
  color: var(--pkg-brand);
  animation: spin 1s;
  &__icon { color: var(--pkg-brand); }
}
.base { color: red; }
.utility { display: block; }
@keyframes spin { from { opacity: 0; } to { opacity: 1; } }
"#;

    const PIPELINE_RULE_STRUCTURAL_INTERPASS_SOURCE: &str = r#"
@layer components {
  @scope (:root) {
    .card { color: red; &__icon { color: blue; } }
    .dup { color: red; }
    .dup { color: red; }
    .empty {}
    @supports (display: grid) { .grid { display: grid; } }
    @media screen { .media { color: green; } }
  }
}
"#;

    fn expected_structural_transform_pass_ids() -> Vec<&'static str> {
        let mut pass_ids = omena_transform_cst::default_transform_pass_descriptors()
            .into_iter()
            .filter(|descriptor| {
                descriptor.pass_class == omena_transform_cst::TransformPassClassV0::Structural
            })
            .map(|descriptor| descriptor.id)
            .collect::<Vec<_>>();
        pass_ids.sort_unstable();
        pass_ids
    }

    fn structural_transform_ir_shadow_corpus_fixtures<'source>(
        samples: &'source [omena_benchmarks::StyleSample],
    ) -> Vec<omena_transform_passes::TransformStructuralIrShadowFixtureInputV0<'source>> {
        samples
            .iter()
            .flat_map(|sample| {
                [
                    omena_transform_passes::TransformStructuralIrShadowFixtureInputV0 {
                        fixture: sample.name,
                        pass: omena_transform_cst::TransformPassKind::NestingUnwrap,
                        dialect: sample.dialect,
                        source: sample.source.as_str(),
                        closed_bundle: false,
                    },
                    omena_transform_passes::TransformStructuralIrShadowFixtureInputV0 {
                        fixture: sample.name,
                        pass: omena_transform_cst::TransformPassKind::ScopeFlatten,
                        dialect: sample.dialect,
                        source: sample.source.as_str(),
                        closed_bundle: false,
                    },
                    omena_transform_passes::TransformStructuralIrShadowFixtureInputV0 {
                        fixture: sample.name,
                        pass: omena_transform_cst::TransformPassKind::LayerFlatten,
                        dialect: sample.dialect,
                        source: sample.source.as_str(),
                        closed_bundle: true,
                    },
                    omena_transform_passes::TransformStructuralIrShadowFixtureInputV0 {
                        fixture: sample.name,
                        pass: omena_transform_cst::TransformPassKind::RuleDeduplication,
                        dialect: sample.dialect,
                        source: sample.source.as_str(),
                        closed_bundle: false,
                    },
                    omena_transform_passes::TransformStructuralIrShadowFixtureInputV0 {
                        fixture: sample.name,
                        pass: omena_transform_cst::TransformPassKind::RuleMerging,
                        dialect: sample.dialect,
                        source: sample.source.as_str(),
                        closed_bundle: false,
                    },
                    omena_transform_passes::TransformStructuralIrShadowFixtureInputV0 {
                        fixture: sample.name,
                        pass: omena_transform_cst::TransformPassKind::SelectorMerging,
                        dialect: sample.dialect,
                        source: sample.source.as_str(),
                        closed_bundle: false,
                    },
                    omena_transform_passes::TransformStructuralIrShadowFixtureInputV0 {
                        fixture: sample.name,
                        pass: omena_transform_cst::TransformPassKind::EmptyRuleRemoval,
                        dialect: sample.dialect,
                        source: sample.source.as_str(),
                        closed_bundle: false,
                    },
                    omena_transform_passes::TransformStructuralIrShadowFixtureInputV0 {
                        fixture: sample.name,
                        pass: omena_transform_cst::TransformPassKind::SupportsStaticEval,
                        dialect: sample.dialect,
                        source: sample.source.as_str(),
                        closed_bundle: false,
                    },
                    omena_transform_passes::TransformStructuralIrShadowFixtureInputV0 {
                        fixture: sample.name,
                        pass: omena_transform_cst::TransformPassKind::MediaStaticEval,
                        dialect: sample.dialect,
                        source: sample.source.as_str(),
                        closed_bundle: false,
                    },
                    omena_transform_passes::TransformStructuralIrShadowFixtureInputV0 {
                        fixture: sample.name,
                        pass: omena_transform_cst::TransformPassKind::ContainerStaticEval,
                        dialect: sample.dialect,
                        source: sample.source.as_str(),
                        closed_bundle: false,
                    },
                    omena_transform_passes::TransformStructuralIrShadowFixtureInputV0 {
                        fixture: sample.name,
                        pass: omena_transform_cst::TransformPassKind::NativeCssStaticEval,
                        dialect: sample.dialect,
                        source: sample.source.as_str(),
                        closed_bundle: false,
                    },
                    omena_transform_passes::TransformStructuralIrShadowFixtureInputV0 {
                        fixture: sample.name,
                        pass: omena_transform_cst::TransformPassKind::DeadMediaBranchRemoval,
                        dialect: sample.dialect,
                        source: sample.source.as_str(),
                        closed_bundle: false,
                    },
                    omena_transform_passes::TransformStructuralIrShadowFixtureInputV0 {
                        fixture: sample.name,
                        pass: omena_transform_cst::TransformPassKind::DeadSupportsBranchRemoval,
                        dialect: sample.dialect,
                        source: sample.source.as_str(),
                        closed_bundle: false,
                    },
                    omena_transform_passes::TransformStructuralIrShadowFixtureInputV0 {
                        fixture: sample.name,
                        pass: omena_transform_cst::TransformPassKind::TreeShakeClass,
                        dialect: sample.dialect,
                        source: sample.source.as_str(),
                        closed_bundle: false,
                    },
                    omena_transform_passes::TransformStructuralIrShadowFixtureInputV0 {
                        fixture: sample.name,
                        pass: omena_transform_cst::TransformPassKind::TreeShakeKeyframes,
                        dialect: sample.dialect,
                        source: sample.source.as_str(),
                        closed_bundle: false,
                    },
                    omena_transform_passes::TransformStructuralIrShadowFixtureInputV0 {
                        fixture: sample.name,
                        pass: omena_transform_cst::TransformPassKind::TreeShakeValue,
                        dialect: sample.dialect,
                        source: sample.source.as_str(),
                        closed_bundle: false,
                    },
                    omena_transform_passes::TransformStructuralIrShadowFixtureInputV0 {
                        fixture: sample.name,
                        pass: omena_transform_cst::TransformPassKind::TreeShakeCustomProperty,
                        dialect: sample.dialect,
                        source: sample.source.as_str(),
                        closed_bundle: false,
                    },
                    omena_transform_passes::TransformStructuralIrShadowFixtureInputV0 {
                        fixture: sample.name,
                        pass: omena_transform_cst::TransformPassKind::ImportInline,
                        dialect: sample.dialect,
                        source: sample.source.as_str(),
                        closed_bundle: false,
                    },
                    omena_transform_passes::TransformStructuralIrShadowFixtureInputV0 {
                        fixture: sample.name,
                        pass: omena_transform_cst::TransformPassKind::ResolveCssModulesComposes,
                        dialect: sample.dialect,
                        source: sample.source.as_str(),
                        closed_bundle: false,
                    },
                    omena_transform_passes::TransformStructuralIrShadowFixtureInputV0 {
                        fixture: sample.name,
                        pass: omena_transform_cst::TransformPassKind::HashCssModuleClassNames,
                        dialect: sample.dialect,
                        source: sample.source.as_str(),
                        closed_bundle: false,
                    },
                    omena_transform_passes::TransformStructuralIrShadowFixtureInputV0 {
                        fixture: sample.name,
                        pass: omena_transform_cst::TransformPassKind::DesignTokenRouting,
                        dialect: sample.dialect,
                        source: sample.source.as_str(),
                        closed_bundle: false,
                    },
                ]
            })
            .collect()
    }

    fn structural_transform_ir_pipeline_shadow_corpus_fixtures<'source>(
        samples: &'source [omena_benchmarks::StyleSample],
    ) -> Vec<omena_transform_passes::TransformStructuralIrPipelineShadowFixtureInputV0<'source>>
    {
        let mut fixtures = samples
            .iter()
            .map(|sample| {
                omena_transform_passes::TransformStructuralIrPipelineShadowFixtureInputV0 {
                    fixture: sample.name,
                    dialect: sample.dialect,
                    source: sample.source.as_str(),
                    closed_bundle: true,
                }
            })
            .collect::<Vec<_>>();
        fixtures.extend([
            omena_transform_passes::TransformStructuralIrPipelineShadowFixtureInputV0 {
                fixture: "pipeline-module-structural-interpass",
                dialect: omena_parser::StyleDialect::Css,
                source: PIPELINE_MODULE_STRUCTURAL_INTERPASS_SOURCE,
                closed_bundle: true,
            },
            omena_transform_passes::TransformStructuralIrPipelineShadowFixtureInputV0 {
                fixture: "pipeline-rule-structural-interpass",
                dialect: omena_parser::StyleDialect::Css,
                source: PIPELINE_RULE_STRUCTURAL_INTERPASS_SOURCE,
                closed_bundle: true,
            },
        ]);
        fixtures
    }

    #[test]
    fn template_placeholder_default_none_identity_matches_committed_snapshots() {
        let snapshots = TEMPLATE_PLACEHOLDER_DEFAULT_NONE_FIXTURES
            .iter()
            .map(|fixture| {
                (
                    *fixture,
                    template_placeholder_default_none_snapshot(*fixture),
                )
            })
            .collect::<Vec<_>>();
        for (fixture, (token_hash, syntax_hash)) in &snapshots {
            eprintln!("{} token={token_hash} syntax={syntax_hash}", fixture.id);
        }
        for (fixture, (token_hash, syntax_hash)) in snapshots {
            assert_eq!(
                token_hash, fixture.expected_token_hash,
                "token snapshot drift for {}",
                fixture.id
            );
            assert_eq!(
                syntax_hash, fixture.expected_syntax_hash,
                "syntax snapshot drift for {}",
                fixture.id
            );
        }
    }

    #[test]
    fn seed_corpora_parse_to_complete_trees() -> Result<(), Box<dyn std::error::Error>> {
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(SASS_SPEC_SEED_MANIFEST_SOURCE)
                .ok()
                .and_then(|manifest| manifest.get("schemaVersion").cloned()),
            Some(serde_json::Value::String("0".to_string()))
        );
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(LESS_SEED_MANIFEST_SOURCE)
                .ok()
                .and_then(|manifest| manifest.get("schemaVersion").cloned()),
            Some(serde_json::Value::String("0".to_string()))
        );
        let mut fixtures = conformance_seed_fixtures(WPT_SEED_CHUNK_SOURCES, "css")?;
        fixtures.extend(conformance_seed_fixtures(
            SASS_SPEC_SEED_CHUNK_SOURCES,
            "scss",
        )?);
        fixtures.extend(conformance_seed_fixtures(LESS_SEED_CHUNK_SOURCES, "less")?);

        assert!(
            !fixtures.is_empty(),
            "seed conformance corpus must not be empty"
        );
        for fixture in fixtures {
            let dialect = dialect_from_seed_fixture(fixture.dialect.as_str());
            assert!(
                dialect.is_some(),
                "{} declares an unsupported dialect: {}",
                fixture.id,
                fixture.dialect
            );
            let Some(dialect) = dialect else {
                continue;
            };
            let parsed = parse(fixture.source.as_str(), dialect);
            let covered_len = u32::from(parsed.syntax().text_range().len()) as usize;
            assert_eq!(
                covered_len,
                fixture.source.len(),
                "{} must parse to a byte-complete CST",
                fixture.id
            );
        }
        Ok(())
    }

    #[test]
    fn external_corpus_contract_covers_committed_manifests()
    -> Result<(), Box<dyn std::error::Error>> {
        use crate::external_corpus_envelope_idl_generated::{
            ExternalCorpusDifferentialManifestV1Json, ExternalCorpusEnvelopeV1Json,
            ExternalCorpusSeedPolicyV1Toml, ExternalCorpusStageV1Json,
        };

        for (label, source) in [
            ("wpt", WPT_SEED_MANIFEST_SOURCE),
            ("sass-spec", SASS_SPEC_SEED_MANIFEST_SOURCE),
            ("sass-spec-imported", SASS_SPEC_IMPORTED_MANIFEST_SOURCE),
            ("less", LESS_SEED_MANIFEST_SOURCE),
        ] {
            let envelope = serde_json::from_str::<ExternalCorpusEnvelopeV1Json>(source)?;
            assert_eq!(envelope.schema_version, "0", "{label} schema version");
            assert!(envelope.product.contains("manifest"), "{label} product");
            assert!(
                envelope.source.pin.contains('@'),
                "{label} source pin must include repo@sha"
            );
            assert!(
                !envelope.source.sparse_paths.is_empty(),
                "{label} sparse paths must not be empty"
            );
            assert!(
                !envelope.chunks.is_empty(),
                "{label} chunks must not be empty"
            );
            assert!(
                envelope
                    .chunks
                    .iter()
                    .all(|chunk| !chunk.sha256.is_empty() && chunk.fixture_count >= 0),
                "{label} chunks must carry hash and fixture count"
            );
            assert_eq!(
                envelope.known_failure_policy.stage2_blocking,
                matches!(envelope.stage, ExternalCorpusStageV1Json::Stage2Blocking),
                "{label} stage and policy blocking flag must agree"
            );
        }

        let imported = serde_json::from_str::<ExternalCorpusEnvelopeV1Json>(
            SASS_SPEC_IMPORTED_MANIFEST_SOURCE,
        )?;
        assert_eq!(
            imported.chunks[0].sha256,
            sha256_hex(SASS_SPEC_IMPORTED_CHUNK_SOURCE.as_bytes()),
            "sass-spec imported chunk sha256 drift"
        );

        for (label, source) in [
            ("sass-differential", SASS_DIFFERENTIAL_MANIFEST_SOURCE),
            (
                "static-stylesheet-external-differential",
                STATIC_STYLESHEET_EXTERNAL_DIFFERENTIAL_MANIFEST_SOURCE,
            ),
        ] {
            let manifest =
                serde_json::from_str::<ExternalCorpusDifferentialManifestV1Json>(source)?;
            assert_eq!(manifest.schema_version, "0", "{label} schema version");
            assert!(!manifest.mode.is_empty(), "{label} mode");
            assert!(!manifest.fixtures.is_empty(), "{label} fixtures");
        }

        for (label, source) in [
            ("wpt", WPT_SEED_KNOWN_FAILURE_POLICY_SOURCE),
            ("sass-spec", SASS_SPEC_SEED_KNOWN_FAILURE_POLICY_SOURCE),
            ("less", LESS_SEED_KNOWN_FAILURE_POLICY_SOURCE),
        ] {
            let policy = toml::from_str::<ExternalCorpusSeedPolicyV1Toml>(source)?;
            assert_eq!(policy.schema_version, "0", "{label} policy schema");
            assert!(
                policy.corpus_manifest.ends_with("manifest.json"),
                "{label} policy manifest path"
            );
            assert!(
                policy.source_pin.contains('@'),
                "{label} policy source pin must include repo@sha"
            );
            assert!(
                policy.review_interval_days > 0,
                "{label} policy review interval"
            );
        }

        Ok(())
    }

    #[test]
    fn sass_spec_imported_expectation_buckets_are_total() {
        let report = summarize_sass_spec_expectation_buckets();
        assert!(report.fixture_count >= 1, "{report:#?}");
        assert_eq!(
            report.fixture_count,
            report.static_must_match_count
                + report.expected_sound_bail_count
                + report.parser_recovery_count
                + report.out_of_scope_count,
            "{report:#?}"
        );
        assert!(report.static_must_match_count >= 1, "{report:#?}");
        assert!(report.expected_sound_bail_count >= 1, "{report:#?}");
        assert!(report.parser_recovery_count >= 1, "{report:#?}");
        assert!(report.out_of_scope_count >= 1, "{report:#?}");
        assert!(report.all_fixtures_have_one_expectation_kind, "{report:#?}");
        assert!(report.all_expectation_kinds_match_criteria, "{report:#?}");
        assert!(
            report.missing_expectation_kind_fixture_ids.is_empty(),
            "{report:#?}"
        );
        assert!(
            report.criteria_mismatch_fixture_ids.is_empty(),
            "{report:#?}"
        );
    }

    #[test]
    fn sass_spec_sound_bail_membership_is_non_vacuous() {
        let report = summarize_sass_spec_sound_bail_membership();
        assert!(report.case_count >= 1, "{report:#?}");
        assert_eq!(report.checked_case_count, report.case_count, "{report:#?}");
        assert!(report.non_top_case_count >= 1, "{report:#?}");
        assert!(report.raw_tightness_case_count >= 1, "{report:#?}");
        assert!(report.all_concrete_values_in_abstract_values, "{report:#?}");
        assert!(report.weakening_preserves_membership, "{report:#?}");
        assert!(report.exact_tightness_holds, "{report:#?}");
        assert!(report.all_membership_checks_hold, "{report:#?}");
    }

    #[test]
    fn sass_spec_static_must_match_values_agree_with_oracle() {
        let report = summarize_sass_spec_static_must_match();
        assert!(report.case_count >= 1, "{report:#?}");
        assert_eq!(report.checked_case_count, report.case_count, "{report:#?}");
        assert!(report.declaration_value_count >= 1, "{report:#?}");
        assert_eq!(
            report.matched_declaration_value_count, report.declaration_value_count,
            "{report:#?}"
        );
        assert!(report.all_static_values_match_oracle, "{report:#?}");
        assert!(report.all_static_match_checks_hold, "{report:#?}");
        assert!(
            report
                .records
                .iter()
                .all(|record| !record.omena_rendered_values.is_empty()),
            "{report:#?}"
        );
    }

    #[test]
    fn sass_spec_expectation_bucket_ledger_matches_imported_chunk() {
        let report = summarize_sass_spec_expectation_bucket_ledger();
        assert!(report.fixture_count >= 4, "{report:#?}");
        assert!(report.ledger_metadata_valid, "{report:#?}");
        assert!(report.all_bucket_totals_match_ledger, "{report:#?}");
        assert!(report.all_fixture_assignments_match_ledger, "{report:#?}");
        assert!(
            report.all_reclassification_entries_have_rationale,
            "{report:#?}"
        );
        assert!(report.missing_ledger_fixture_ids.is_empty(), "{report:#?}");
        assert!(report.stale_ledger_fixture_ids.is_empty(), "{report:#?}");
        assert!(
            report.assignment_mismatch_fixture_ids.is_empty(),
            "{report:#?}"
        );
    }

    #[test]
    fn sass_spec_bail_site_ledger_matches_current_sources() {
        let report = summarize_sass_spec_bail_site_ledger();
        assert_eq!(report.semantic_site_count, 33, "{report:#?}");
        assert_eq!(report.ledger_semantic_site_count, 33, "{report:#?}");
        assert_eq!(report.raw_pattern_hit_count, 38, "{report:#?}");
        assert_eq!(report.non_semantic_pattern_hit_count, 5, "{report:#?}");
        assert!(report.ledger_metadata_valid, "{report:#?}");
        assert!(report.all_semantic_sites_match_ledger, "{report:#?}");
        assert!(report.all_sites_linked_or_named_gap, "{report:#?}");
        assert!(report.all_linked_cases_match_reason_class, "{report:#?}");
        assert!(
            report.all_linked_cases_are_imported_sound_bail_cases,
            "{report:#?}"
        );
        assert!(report.all_bail_site_ledger_checks_hold, "{report:#?}");
        assert!(report.linked_site_count >= 1, "{report:#?}");
        assert!(report.linked_case_count >= 1, "{report:#?}");
        assert_eq!(
            report.semantic_site_count,
            report.linked_site_count + report.named_gap_site_count,
            "{report:#?}"
        );
        assert!(report.missing_ledger_site_keys.is_empty(), "{report:#?}");
        assert!(report.stale_ledger_site_keys.is_empty(), "{report:#?}");
        assert!(report.gapless_site_keys.is_empty(), "{report:#?}");
        assert!(report.unknown_link_fixture_ids.is_empty(), "{report:#?}");
        assert!(report.reason_mismatch_link_keys.is_empty(), "{report:#?}");
    }

    #[test]
    fn sass_spec_import_scale_counts_are_scan_derived() {
        let report = summarize_sass_spec_import_scale();
        assert_eq!(report.product, "omena-diff-test.sass-spec-import-scale");
        assert!(report.source_archive_scan_succeeded, "{report:#?}");
        assert!(report.source_archive_count >= 1, "{report:#?}");
        assert_eq!(
            report.source_archive_count, report.imported_fixture_count,
            "{report:#?}"
        );
        assert!(
            report.upstream_archive_count > report.imported_fixture_count,
            "{report:#?}"
        );
        assert!(
            report.upstream_archive_count > report.source_archive_count,
            "{report:#?}"
        );
        assert!(
            report.upstream_scale_artifact_matches_manifest,
            "{report:#?}"
        );
        assert_eq!(report.imported_chunk_count, 1, "{report:#?}");
        assert!(report.seed_fixture_count >= 4, "{report:#?}");
        assert_eq!(
            report.per_push_smoke_floor_fixture_count, report.seed_fixture_count,
            "{report:#?}"
        );
        assert!(
            report.per_push_smoke_fixture_count >= report.per_push_smoke_floor_fixture_count,
            "{report:#?}"
        );
        assert_eq!(
            report.imported_fixture_count,
            report.static_must_match_count
                + report.expected_sound_bail_count
                + report.parser_recovery_count
                + report.out_of_scope_count,
            "{report:#?}"
        );
        assert!(report.all_imported_counts_match_manifest, "{report:#?}");
        assert!(report.all_chunk_hashes_match_manifest, "{report:#?}");
        assert!(report.all_sparse_path_counts_match_manifest, "{report:#?}");
        assert!(report.all_source_archives_under_sparse_paths, "{report:#?}");
        assert!(
            report.all_source_archive_count_matches_imported_fixtures,
            "{report:#?}"
        );
        assert!(
            report.all_upstream_archive_count_exceeds_imported_fixtures,
            "{report:#?}"
        );
        assert!(
            report.all_upstream_archive_count_exceeds_source_archives,
            "{report:#?}"
        );
        assert!(report.all_smoke_floor_holds, "{report:#?}");
        assert!(report.all_import_scale_checks_hold, "{report:#?}");
    }

    fn sha256_hex(bytes: &[u8]) -> String {
        use sha2::{Digest, Sha256};

        const HEX: &[u8; 16] = b"0123456789abcdef";
        let digest = Sha256::digest(bytes);
        let mut output = String::with_capacity(digest.len() * 2);
        for byte in digest {
            output.push(char::from(HEX[usize::from(byte >> 4)]));
            output.push(char::from(HEX[usize::from(byte & 0x0f)]));
        }
        output
    }

    #[test]
    fn sass_less_seed_recorded_bogus_sets_match_policy() -> Result<(), Box<dyn std::error::Error>> {
        for (corpus_label, chunks, policy_source, default_dialect) in [
            (
                "sass-spec",
                SASS_SPEC_SEED_CHUNK_SOURCES,
                SASS_SPEC_SEED_KNOWN_FAILURE_POLICY_SOURCE,
                "scss",
            ),
            (
                "less",
                LESS_SEED_CHUNK_SOURCES,
                LESS_SEED_KNOWN_FAILURE_POLICY_SOURCE,
                "less",
            ),
        ] {
            let fixtures = conformance_seed_fixtures(chunks, default_dialect)?;
            let mut actual_recorded = BTreeSet::new();
            for fixture in fixtures {
                let dialect = dialect_from_seed_fixture(fixture.dialect.as_str());
                assert!(
                    dialect.is_some(),
                    "{corpus_label}/{} declares an unsupported dialect: {}",
                    fixture.id,
                    fixture.dialect
                );
                let Some(dialect) = dialect else {
                    continue;
                };
                let bogus_kinds = actual_bogus_kinds(fixture.source.as_str(), dialect);
                let error_codes = actual_error_codes(fixture.source.as_str(), dialect);
                assert_eq!(
                    bogus_kinds, fixture.expected_bogus_kinds,
                    "{corpus_label}/{} bogus-kind set drift",
                    fixture.id
                );
                assert_eq!(
                    error_codes, fixture.expected_error_codes,
                    "{corpus_label}/{} error-code set drift",
                    fixture.id
                );
                if !bogus_kinds.is_empty() || !error_codes.is_empty() {
                    actual_recorded.insert(format!("{}\n{}", fixture.id, fixture.subtest));
                }
            }
            assert_eq!(
                actual_recorded,
                seed_policy_known_failure_subtests(policy_source),
                "{corpus_label} known-failure register drift"
            );
        }
        Ok(())
    }

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
    fn selector_context_soundness_corpus_preserves_positive_witnesses() {
        let report = summarize_selector_context_soundness_v0();

        assert_eq!(report.product, "omena-diff-test.selector-context-soundness");
        assert_eq!(
            report.fixture_count,
            SELECTOR_CONTEXT_SOUNDNESS_FIXTURES.len()
        );
        assert!(
            report.all_expected_verdicts_match,
            "selector-context verdicts drifted: {report:#?}"
        );
        assert!(
            report.all_unmodeled_fixtures_stay_maybe,
            "unmodeled selector fixtures must stay conservative: {report:#?}"
        );
        assert!(
            report.positive_preservation_matches,
            "known positive selector relations changed unexpectedly: {report:#?}"
        );
    }

    #[test]
    fn expression_domain_source_cfg_refinement_oracle_is_non_vacuous() {
        let report = summarize_expression_domain_source_cfg_refinement_oracle_v0();

        assert_eq!(
            report.product,
            "omena-diff-test.expression-domain-source-cfg-refinement-oracle"
        );
        assert_eq!(report.fixture_count, 1);
        assert!(report.all_source_values_le_baseline, "{report:#?}");
        assert!(
            report.strict_refinement_witness_count >= 1,
            "source CFG oracle must include a strict non-vacuous witness: {report:#?}"
        );
        assert!(
            report.all_source_values_covered_by_baseline,
            "source CFG values must stay inside the file-merge baseline coverage: {report:#?}"
        );
        assert!(
            report.all_shape_witnesses_present,
            "source CFG oracle must observe branch/concat shape and no file-merge source block: {report:#?}"
        );
        assert!(report.reports.iter().all(|fixture| {
            fixture.all_source_values_le_baseline
                && fixture.strict_refinement_count > 0
                && fixture.all_source_values_covered_by_baseline
                && fixture.branch_block_observed
                && fixture.concat_transfer_observed
                && fixture.file_merge_absent_from_source_cfg
        }));
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
        assert!(summary.sass_spec_imported_fixture_count >= 1);
        assert_eq!(
            summary.sass_spec_import_source_archive_count,
            summary.sass_spec_imported_fixture_count
        );
        assert!(summary.sass_spec_import_chunk_count >= 1);
        assert!(
            summary.sass_spec_per_push_smoke_fixture_count
                >= summary.sass_spec_per_push_smoke_floor_fixture_count
        );
        assert!(summary.all_sass_spec_import_scale_counts_match);
        assert!(summary.all_sass_spec_smoke_floor_holds);
        assert_eq!(
            summary.sass_spec_imported_fixture_count,
            summary.sass_spec_static_must_match_count
                + summary.sass_spec_expected_sound_bail_count
                + summary.sass_spec_parser_recovery_count
                + summary.sass_spec_out_of_scope_count
        );
        assert!(summary.all_sass_spec_imported_fixtures_have_one_expectation_kind);
        assert!(summary.all_sass_spec_imported_expectation_kinds_match_criteria);
        assert!(summary.sass_spec_sound_bail_case_count >= 1);
        assert_eq!(
            summary.sass_spec_sound_bail_checked_case_count,
            summary.sass_spec_sound_bail_case_count
        );
        assert!(summary.sass_spec_sound_bail_non_top_case_count >= 1);
        assert!(summary.sass_spec_sound_bail_raw_tightness_case_count >= 1);
        assert!(summary.all_sass_spec_sound_bail_membership_checks_hold);
        assert!(summary.sass_spec_static_match_case_count >= 1);
        assert_eq!(
            summary.sass_spec_static_match_checked_case_count,
            summary.sass_spec_static_match_case_count
        );
        assert!(summary.sass_spec_static_match_declaration_value_match_count >= 1);
        assert!(summary.all_sass_spec_static_match_checks_hold);
        assert!(summary.all_sass_spec_expectation_bucket_totals_match_ledger);
        assert!(summary.all_sass_spec_expectation_fixture_assignments_match_ledger);
        assert_eq!(summary.sass_spec_bail_site_ledger_site_count, 33);
        assert_eq!(summary.sass_spec_bail_site_raw_pattern_hit_count, 38);
        assert_eq!(
            summary.sass_spec_bail_site_non_semantic_pattern_hit_count,
            5
        );
        assert!(summary.sass_spec_bail_site_linked_site_count >= 1);
        assert!(summary.sass_spec_bail_site_named_gap_count >= 1);
        assert!(summary.sass_spec_bail_site_linked_case_count >= 1);
        assert!(summary.all_sass_spec_bail_site_ledger_counts_match);
        assert!(summary.all_sass_spec_bail_site_ledger_entries_linked_or_named_gap);
        assert!(summary.all_sass_spec_bail_site_ledger_link_reason_classes_match);
        assert!(summary.all_sass_spec_bail_site_ledger_linked_cases_are_imported);
        assert!(
            summary
                .wpt_value_differential_report
                .all_foldable_matches_hold
        );
        assert!(summary.scss_eval_truthiness_cst_equivalence_fixture_count >= 12);
        assert!(summary.all_scss_eval_truthiness_cst_equivalence_fixtures_match);
        assert_eq!(summary.scss_eval_public_summary_comparison_count, 20);
        assert!(
            summary.all_scss_eval_public_summaries_match,
            "{:#?}",
            summary.scss_eval_public_summary_equivalence_report
        );
        assert_eq!(
            summary.transform_ir_identity_round_trip_fixture_count,
            PARSER_LEGACY_SEED_FIXTURES.len()
                + PARSER_FACT_AUTHORITY_FIXTURES.len()
                + style_corpus().len()
                + bundler_productization_corpus().len()
        );
        assert!(
            summary.all_transform_ir_identity_round_trip_fields_match,
            "{:#?}",
            summary.transform_ir_identity_round_trip_report
        );
        assert!(summary.oss_corpus_farm_entry_count >= 3);
        assert!(summary.oss_corpus_farm_repository_count >= 1);
        assert!(summary.oss_corpus_farm_dialect_count >= 3);
        assert!(summary.all_oss_corpus_farm_manifest_checks_hold);
        assert!(
            summary
                .oss_corpus_farm_manifest_report
                .all_entries_follow_generated_envelope_shape
        );
        assert!(
            summary
                .oss_corpus_farm_manifest_report
                .dialects
                .contains(&"css")
        );
        assert!(
            summary
                .oss_corpus_farm_manifest_report
                .dialects
                .contains(&"scss")
        );
        assert!(
            summary
                .oss_corpus_farm_manifest_report
                .dialects
                .contains(&"less")
        );
        assert!(summary.deletion_stale_reuse_fixture_count >= 2);
        assert!(summary.deletion_stale_reuse_divergence_fixture_count >= 1);
        assert!(summary.deletion_stale_reuse_cycle_deletion_fixture_count >= 1);
        assert_eq!(
            summary.deletion_stale_reuse_demand_projected_equal_fixture_count,
            summary.deletion_stale_reuse_fixture_count
        );
        assert!(summary.deletion_stale_reuse_all_demand_projected_equal);
        assert!(summary.deletion_stale_reuse_ready_for_relocation_consumer);
        assert_eq!(
            summary
                .deletion_stale_reuse_corpus_report
                .readiness_artifact,
            "DELETION-STALE-REUSE-CORPUS-READY"
        );
        assert!(
            summary
                .closed_gates
                .contains(&"deletionDemandProjectedBatchEquivalence")
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
            summary.parser_cst_fact_authority_fixture_count,
            PARSER_LEGACY_SEED_FIXTURES.len() + PARSER_FACT_AUTHORITY_FIXTURES.len()
        );
        assert_eq!(
            summary.parser_cst_fact_authority_comparison_count,
            summary.parser_cst_fact_authority_fixture_count * 16
        );
        assert!(summary.all_parser_cst_fact_authority_values_match);
        assert!(summary.all_parser_cst_fact_authority_spans_match);
        assert_eq!(
            summary.parser_cst_fact_authority_metamorphic_relation_count,
            summary.parser_cst_fact_authority_fixture_count * 2
        );
        assert!(summary.all_parser_cst_fact_authority_metamorphic_relations_hold);
        assert_eq!(
            summary.parser_cst_context_raw_scan_fixture_count,
            PARSER_CST_CONTEXT_RAW_SCAN_FIXTURES.len()
        );
        assert!(summary.all_parser_cst_context_raw_scan_fixtures_match);
        assert_eq!(
            summary.selector_context_soundness_fixture_count,
            SELECTOR_CONTEXT_SOUNDNESS_FIXTURES.len()
        );
        assert!(summary.all_selector_context_soundness_fixtures_match);
        assert_eq!(summary.source_cfg_refinement_fixture_count, 1);
        assert!(summary.all_source_cfg_values_le_file_merge_baseline);
        assert!(summary.source_cfg_strict_refinement_witness_count >= 1);
        assert!(summary.all_source_cfg_values_covered_by_file_merge_baseline);
        assert!(summary.all_source_cfg_shape_witnesses_present);
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
                .contains(&"sassSpecExpectationBucketClassification")
        );
        assert!(
            summary
                .closed_gates
                .contains(&"sassSpecSoundBailMembership")
        );
        assert!(
            summary
                .closed_gates
                .contains(&"sassSpecExpectationBucketLedger")
        );
        assert!(summary.closed_gates.contains(&"sassSpecBailSiteLedger"));
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
                .contains(&"parserCstFactAuthorityEquivalence")
        );
        assert!(
            summary
                .closed_gates
                .contains(&"parserCstContextRawScanDivergence")
        );
        assert!(summary.closed_gates.contains(&"selectorContextSoundness"));
        assert!(
            summary
                .closed_gates
                .contains(&"expressionDomainSourceCfgRefinementOracle")
        );
        assert!(
            summary
                .closed_gates
                .contains(&"scssEvalTruthinessCstEquivalence")
        );
        assert!(
            summary
                .closed_gates
                .contains(&"scssEvalPublicSummaryPreservation")
        );
        assert!(
            summary
                .closed_gates
                .contains(&"transformIrIdentityRoundTrip")
        );
        assert!(summary.closed_gates.contains(&"ossCorpusFarmManifest"));
        assert!(summary.closed_gates.contains(&"deletionStaleReuseCorpus"));
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
        assert!(summary.reachability_equivalence_fixture_count >= 4);
        assert!(summary.all_reachability_second_oracle_sets_equal);
        assert!(summary.all_reachability_streaming_matches_batch);
        assert!(summary.all_reachability_bitset_parity_equal);
        assert!(summary.all_reachability_closure_hash_bitset_parity_equal);
        assert!(summary.all_reachability_product_parity_with_batch);
        assert!(summary.all_reachability_fact_keys_three_way_equal);
        assert!(summary.all_reachability_fact_keys_four_way_equal);
        assert!(summary.all_reachability_selector_relations_equal);
        assert!(
            summary
                .reachability_equivalence_report
                .files
                .iter()
                .any(|file| file.fixture_id == "multi-hop-composes-sass-chain")
        );
        assert!(
            summary
                .closed_gates
                .contains(&"reachabilitySecondOracleEquivalence")
        );
        assert!(summary.closed_gates.contains(&"reachabilityBitsetParity"));
        assert!(
            summary
                .closed_gates
                .contains(&"reachabilityClosureHashBitsetParity")
        );
        assert!(
            summary
                .closed_gates
                .contains(&"reachabilityFactKeyThreeWayEquivalence")
        );
        assert!(
            summary
                .closed_gates
                .contains(&"reachabilityFactKeyFourWayEquivalence")
        );
        assert!(
            summary
                .wpt_seed_metadata_report
                .closed_gates
                .contains(&"wptSeedStaleKnownFailurePruning")
        );
    }

    #[test]
    fn parser_cst_fact_authority_matches_legacy_collectors() {
        let report = summarize_parser_cst_fact_authority_equivalence_v0();

        assert_eq!(
            report.product,
            "omena-diff-test.parser-cst-fact-authority-equivalence"
        );
        assert_eq!(report.category_count, 16);
        assert_eq!(
            report.fixture_count,
            PARSER_LEGACY_SEED_FIXTURES.len() + PARSER_FACT_AUTHORITY_FIXTURES.len()
        );
        assert_eq!(
            report.comparisons.len(),
            report.fixture_count * report.category_count
        );
        assert!(report.all_value_sets_match, "{report:#?}");
        assert!(report.all_span_sets_match, "{report:#?}");
        assert_eq!(report.metamorphic_relation_count, report.fixture_count * 2);
        assert!(report.all_metamorphic_relations_hold, "{report:#?}");

        for category in [
            "selectors",
            "variables",
            "sass_symbols",
            "sass_includes",
            "sass_module_edges",
            "extend_targets",
            "animations",
            "css_module_values",
            "css_module_value_import_edges",
            "css_module_value_definition_edges",
            "css_module_composes",
            "css_module_composes_edges",
            "icss",
            "icss_import_edges",
            "icss_export_edges",
            "at_rules",
        ] {
            assert!(
                report
                    .comparisons
                    .iter()
                    .any(|comparison| comparison.category == category),
                "missing parser fact category: {category}"
            );
        }
    }

    #[test]
    fn parser_cst_context_raw_scan_divergence_fixtures_match_intended_output() {
        let report = summarize_parser_cst_context_raw_scan_divergence_v0();

        assert_eq!(
            report.product,
            "omena-diff-test.parser-cst-context-raw-scan-divergence"
        );
        assert_eq!(
            report.fixture_count,
            PARSER_CST_CONTEXT_RAW_SCAN_FIXTURES.len()
        );
        assert_eq!(report.fixture_count, 3);
        assert!(report.all_fixtures_match, "{report:#?}");

        for fixture_report in &report.reports {
            assert!(fixture_report.rejected_names_absent, "{fixture_report:#?}");
            assert_eq!(
                fixture_report.statement_layers,
                strings_from_static(&fixture_report.expected_statement_layers)
            );
            assert_eq!(
                fixture_report.block_layers,
                strings_from_static(&fixture_report.expected_block_layers)
            );
            assert_eq!(
                fixture_report.layer_selector_memberships,
                strings_from_static(&fixture_report.expected_layer_selector_memberships)
            );
        }
    }

    #[test]
    fn semantic_context_index_has_no_raw_scan_helpers() {
        let semantic_source = include_str!("../../omena-semantic/src/lib.rs");
        for forbidden in [
            ".find(\"@layer\")",
            "selector_class_names",
            "block_header_and_start_before_open_brace",
        ] {
            assert!(
                !semantic_source.contains(forbidden),
                "semantic context index must stay CST-derived: {forbidden}"
            );
        }
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
    fn transform_ir_identity_round_trip_equivalence_keeps_original_bytes_and_origins() {
        let report = summarize_transform_ir_identity_round_trip_equivalence_v0();

        assert_eq!(
            report.product,
            "omena-diff-test.transform-ir-identity-round-trip-equivalence"
        );
        assert_eq!(
            report.fixture_count,
            PARSER_LEGACY_SEED_FIXTURES.len()
                + PARSER_FACT_AUTHORITY_FIXTURES.len()
                + style_corpus().len()
                + bundler_productization_corpus().len()
        );
        assert!(report.all_fields_match, "{report:#?}");
        assert!(
            report
                .closed_gates
                .contains(&"transformIrIdentityRoundTrip")
        );
        assert!(report.reports.iter().all(|fixture| {
            fixture.node_count > 0
                && fixture.all_fields_match
                && fixture
                    .fields
                    .iter()
                    .any(|field| field.field == "allNodesOriginal" && field.matches)
        }));
    }

    #[test]
    fn structural_transform_ir_shadow_equivalence_covers_structural_ir_paths() {
        let report = omena_transform_passes::summarize_structural_ir_shadow_equivalence_v0();

        assert_eq!(
            report.product,
            "omena-transform-passes.structural-ir-shadow-equivalence"
        );
        let expected_pass_ids = expected_structural_transform_pass_ids();

        assert_eq!(report.compared_pass_ids, expected_pass_ids);
        assert_eq!(
            report
                .reports
                .iter()
                .map(|fixture| fixture.pass_id)
                .collect::<BTreeSet<_>>(),
            report
                .compared_pass_ids
                .iter()
                .copied()
                .collect::<BTreeSet<_>>()
        );
        assert_eq!(report.fixture_count, 28);
        assert!(report.all_fields_match, "{report:#?}");
        assert!(report.reports.iter().all(|fixture| {
            fixture.fields.iter().any(|field| {
                field.field == "canonicalCssBytes"
                    && field.matches
                    && !field.string_path_values.is_empty()
            }) && fixture
                .fields
                .iter()
                .any(|field| field.field == "mutationSpanRanges" && field.matches)
                && fixture.ir_path_transaction_commit_count.is_some()
        }));
    }

    #[test]
    fn structural_transform_ir_shadow_equivalence_covers_style_corpora() {
        let samples = style_corpus()
            .into_iter()
            .chain(bundler_productization_corpus())
            .collect::<Vec<_>>();
        let fixtures = structural_transform_ir_shadow_corpus_fixtures(samples.as_slice());
        let report =
            omena_transform_passes::summarize_structural_ir_shadow_equivalence_for_fixtures_v0(
                fixtures.as_slice(),
            );

        assert_eq!(
            report.product,
            "omena-transform-passes.structural-ir-shadow-equivalence"
        );
        let expected_pass_ids = expected_structural_transform_pass_ids();

        assert_eq!(report.compared_pass_ids, expected_pass_ids);
        assert_eq!(
            report
                .reports
                .iter()
                .map(|fixture| fixture.pass_id)
                .collect::<BTreeSet<_>>(),
            report
                .compared_pass_ids
                .iter()
                .copied()
                .collect::<BTreeSet<_>>()
        );
        assert_eq!(
            report.fixture_count,
            samples.len() * report.compared_pass_ids.len()
        );
        assert!(report.all_fields_match, "{report:#?}");
        assert!(report.reports.iter().all(|fixture| {
            fixture
                .fields
                .iter()
                .all(|field| field.matches && field.field != "unknown")
        }));
    }

    #[test]
    fn structural_transform_ir_pipeline_shadow_equivalence_covers_style_corpora()
    -> Result<(), String> {
        let samples = style_corpus()
            .into_iter()
            .chain(bundler_productization_corpus())
            .collect::<Vec<_>>();
        let fixtures = structural_transform_ir_pipeline_shadow_corpus_fixtures(samples.as_slice());
        let report =
            omena_transform_passes::summarize_structural_ir_pipeline_shadow_equivalence_for_fixtures_v0(
                fixtures.as_slice(),
            );

        assert_eq!(
            report.product,
            "omena-transform-passes.structural-ir-pipeline-shadow-equivalence"
        );
        assert_eq!(
            report.compared_pass_ids,
            expected_structural_transform_pass_ids()
        );
        assert_eq!(report.fixture_count, samples.len() + 2);
        assert!(report.all_fields_match, "{report:#?}");
        assert!(report.reports.iter().all(|fixture| {
            fixture.pass_id == "structural-pipeline"
                && fixture
                    .fields
                    .iter()
                    .all(|field| field.matches && field.field != "unknown")
                && fixture.ir_path_transaction_commit_count.is_some()
        }));
        let module_interpass = report
            .reports
            .iter()
            .find(|fixture| fixture.fixture == "pipeline-module-structural-interpass")
            .ok_or_else(|| "module inter-pass pipeline fixture should be covered".to_string())?;
        assert!(
            module_interpass
                .string_path_mutation_count
                .is_some_and(|count| count >= 4),
            "{module_interpass:#?}"
        );
        assert!(module_interpass.fields.iter().any(|field| {
            field.field == "cssImportInlines"
                && field.matches
                && !field.string_path_values.is_empty()
        }));
        assert!(module_interpass.fields.iter().any(|field| {
            field.field == "cssModuleComposesExports"
                && field.matches
                && !field.string_path_values.is_empty()
        }));
        assert!(module_interpass.fields.iter().any(|field| {
            field.field == "designTokenRoutes"
                && field.matches
                && !field.string_path_values.is_empty()
        }));
        let rule_interpass = report
            .reports
            .iter()
            .find(|fixture| fixture.fixture == "pipeline-rule-structural-interpass")
            .ok_or_else(|| "rule inter-pass pipeline fixture should be covered".to_string())?;
        assert!(
            rule_interpass
                .string_path_mutation_count
                .is_some_and(|count| count >= 3),
            "{rule_interpass:#?}"
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
