//! Transform pass registry and DAG planner for the post-v5 omena-css track.
//!
//! This crate consumes `omena-transform-cst` contracts. It does not duplicate
//! transform metadata; its job is to register safe mutations, cascade-proven
//! combinations, conservative lowerings, and emission boundaries as a
//! DAG-respecting execution plan for downstream transform crates.

use std::collections::{BTreeMap, VecDeque};

pub use omena_cascade::CustomPropertyLeastFixedPointSummaryV0;
use omena_cascade::{
    BoxLonghandInputV0, CascadeValue, CustomPropertyEnv, LayerFlattenInputV0, ScopeFlattenInputV0,
    StaticSupportsAssumptionV0, StaticSupportsEvalVerdictV0, evaluate_static_supports_condition,
    prove_box_shorthand_combination, prove_layer_flatten_candidate, prove_scope_flatten_candidate,
    resolve_custom_property_env_least_fixed_point, substitute_custom_properties,
    summarize_custom_property_least_fixed_point,
};
use omena_incremental::{
    IncrementalComputationPlanV0, IncrementalGraphInputV0, IncrementalNodeInputV0,
    IncrementalRevisionV0, IncrementalSnapshotV0, OmenaIncrementalDatabaseV0,
};
use omena_parser::{StyleDialect, lex};
use omena_syntax::SyntaxKind;
use omena_transform_cst::{
    TRANSFORM_PASS_CATALOG_LEN, TransformDagEdgeV0, TransformLayer, TransformPassContractV0,
    TransformPassKind, all_transform_pass_kinds, default_transform_dag_edges,
    default_transform_pass_contracts,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum TransformPassExecutionStatus {
    RegistryAndPlannerReady,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformPassRegistryEntryV0 {
    pub contract: TransformPassContractV0,
    pub module_family: &'static str,
    pub query_family: &'static str,
    pub execution_status: TransformPassExecutionStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformPassesBoundarySummaryV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub registry_entries: Vec<TransformPassRegistryEntryV0>,
    pub dag_edges: Vec<TransformDagEdgeV0>,
    pub pass_count: usize,
    pub full_catalog_registered: bool,
    pub semantic_aware_pass_count: usize,
    pub cascade_aware_pass_count: usize,
    pub planner_enforces_dag_edges: bool,
    pub execution_runtime_ready: bool,
    pub incremental_execution_runtime_ready: bool,
    pub implemented_mutation_pass_ids: Vec<&'static str>,
    pub next_surfaces: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformPassPlanV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub requested_pass_ids: Vec<&'static str>,
    pub ordered_pass_ids: Vec<&'static str>,
    pub satisfied_dag_edge_count: usize,
    pub violated_dag_edge_count: usize,
    pub all_requested_registered: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum TransformPassRuntimeStatus {
    Applied,
    NoChange,
    PlannedOnly,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformPassExecutionOutcomeV0 {
    pub pass_id: &'static str,
    pub status: TransformPassRuntimeStatus,
    pub input_byte_len: usize,
    pub output_byte_len: usize,
    pub mutation_count: usize,
    pub provenance_preserved: bool,
    pub detail: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformProvenanceDerivationForestV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub root_count: usize,
    pub node_count: usize,
    pub nodes: Vec<TransformProvenanceDerivationNodeV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformProvenanceDerivationNodeV0 {
    pub node_index: usize,
    pub parent_index: Option<usize>,
    pub pass_id: &'static str,
    pub status: TransformPassRuntimeStatus,
    pub input_byte_len: usize,
    pub output_byte_len: usize,
    pub source_span_start: usize,
    pub source_span_end: usize,
    pub generated_span_start: usize,
    pub generated_span_end: usize,
    pub mutation_spans: Vec<TransformProvenanceMutationSpanV0>,
    pub mutation_count: usize,
    pub provenance_preserved: bool,
    pub detail: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformProvenanceMutationSpanV0 {
    pub source_span_start: usize,
    pub source_span_end: usize,
    pub generated_span_start: usize,
    pub generated_span_end: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformExecutionSummaryV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub input_byte_len: usize,
    pub output_byte_len: usize,
    pub requested_pass_ids: Vec<&'static str>,
    pub ordered_pass_ids: Vec<&'static str>,
    pub executed_pass_ids: Vec<&'static str>,
    pub planned_only_pass_ids: Vec<&'static str>,
    pub mutation_count: usize,
    pub provenance_preserved: bool,
    pub output_css: String,
    pub css_module_evaluation: Option<TransformModuleEvaluationV0>,
    pub css_import_inlines: Vec<TransformImportInlineV0>,
    pub css_module_composes_exports: Vec<TransformCssModuleComposesResolutionV0>,
    pub design_token_routes: Vec<TransformDesignTokenRouteV0>,
    pub semantic_removals: Vec<TransformSemanticRemovalV0>,
    pub provenance_derivation_forest: TransformProvenanceDerivationForestV0,
    pub outcomes: Vec<TransformPassExecutionOutcomeV0>,
    pub pass_plan: TransformPassPlanV0,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformSemanticRemovalV0 {
    pub pass_id: &'static str,
    pub symbol_kind: &'static str,
    pub name: String,
    pub source_span_start: usize,
    pub source_span_end: usize,
    pub reason: &'static str,
    pub certainty: &'static str,
    pub derivation_steps: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TransformSemanticRemovalCandidate {
    symbol_kind: &'static str,
    name: String,
    source_span_start: usize,
    source_span_end: usize,
    reason: &'static str,
}

impl TransformSemanticRemovalCandidate {
    fn into_public(self, pass_id: &'static str) -> TransformSemanticRemovalV0 {
        TransformSemanticRemovalV0 {
            pass_id,
            symbol_kind: self.symbol_kind,
            name: self.name,
            source_span_start: self.source_span_start,
            source_span_end: self.source_span_end,
            reason: self.reason,
            certainty: "high",
            derivation_steps: vec![
                "closedStyleWorld",
                "reachableRootSetComputed",
                "symbolNotMarkedReachable",
                "sourceRangeRemoved",
            ],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformIncrementalExecutionSummaryV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub incremental_engine: &'static str,
    pub query_model: &'static str,
    pub reuse_policy: &'static str,
    pub reused_previous_execution: bool,
    pub incremental_plan: IncrementalComputationPlanV0,
    pub next_snapshot: IncrementalSnapshotV0,
    pub execution: TransformExecutionSummaryV0,
    pub ready_surfaces: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformCascadeSafetyFuzzCaseV0 {
    pub seed: u64,
    pub pass_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformCascadeSafetyFuzzResultV0 {
    pub seed: u64,
    pub pass_count: usize,
    pub requested_pass_ids: Vec<&'static str>,
    pub executed_pass_ids: Vec<&'static str>,
    pub output_byte_len: usize,
    pub output_token_count: usize,
    pub output_error_count: usize,
    pub provenance_node_count: usize,
    pub passed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformFuzzSeedReportV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub case_count: usize,
    pub passed_count: usize,
    pub failed_count: usize,
    pub results: Vec<TransformCascadeSafetyFuzzResultV0>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(default, rename_all = "camelCase")]
pub struct TransformExecutionContextV0 {
    pub closed_style_world: bool,
    pub drop_dark_mode_media_queries: bool,
    pub reachable_class_names: Vec<String>,
    pub reachable_keyframe_names: Vec<String>,
    pub reachable_value_names: Vec<String>,
    pub reachable_custom_property_names: Vec<String>,
    pub scss_module_evaluation: Option<TransformModuleEvaluationV0>,
    pub less_module_evaluation: Option<TransformModuleEvaluationV0>,
    pub import_inlines: Vec<TransformImportInlineV0>,
    pub class_name_rewrites: Vec<TransformClassNameRewriteV0>,
    pub css_module_composes_resolutions: Vec<TransformCssModuleComposesResolutionV0>,
    pub design_token_routes: Vec<TransformDesignTokenRouteV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformModuleEvaluationV0 {
    pub evaluator: String,
    pub evaluated_css: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformImportInlineV0 {
    pub import_source: String,
    pub replacement_css: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformClassNameRewriteV0 {
    pub original_name: String,
    pub rewritten_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformCssModuleComposesResolutionV0 {
    pub local_class_name: String,
    pub exported_class_names: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformDesignTokenRouteV0 {
    pub token_name: String,
    pub routed_value: String,
}

pub fn summarize_omena_transform_passes_boundary() -> TransformPassesBoundarySummaryV0 {
    let registry_entries = default_transform_pass_contracts()
        .into_iter()
        .map(registry_entry_for_contract)
        .collect::<Vec<_>>();
    let pass_count = registry_entries.len();
    let semantic_aware_pass_count = registry_entries
        .iter()
        .filter(|entry| entry.contract.layer == TransformLayer::SemanticAware)
        .count();
    let cascade_aware_pass_count = registry_entries
        .iter()
        .filter(|entry| entry.contract.reads_cascade_model)
        .count();

    TransformPassesBoundarySummaryV0 {
        schema_version: "0",
        product: "omena-transform-passes.boundary",
        registry_entries,
        dag_edges: default_transform_dag_edges(),
        pass_count,
        full_catalog_registered: pass_count == TRANSFORM_PASS_CATALOG_LEN,
        semantic_aware_pass_count,
        cascade_aware_pass_count,
        planner_enforces_dag_edges: true,
        execution_runtime_ready: true,
        incremental_execution_runtime_ready: true,
        implemented_mutation_pass_ids: implemented_mutation_pass_ids(),
        next_surfaces: Vec::new(),
    }
}

pub fn plan_transform_passes(requested: &[TransformPassKind]) -> TransformPassPlanV0 {
    let requested_pass_ids = requested.iter().map(|pass| pass.id()).collect::<Vec<_>>();
    let ordered_passes = order_passes_by_dag(requested);
    let ordered_pass_ids = ordered_passes
        .iter()
        .map(|pass| pass.id())
        .collect::<Vec<_>>();
    let dag_edges = default_transform_dag_edges();
    let satisfied_dag_edge_count = dag_edges
        .iter()
        .filter(|edge| {
            edge_applies(edge, &ordered_pass_ids) && edge_is_satisfied(edge, &ordered_pass_ids)
        })
        .count();
    let violated_dag_edge_count = dag_edges
        .iter()
        .filter(|edge| {
            edge_applies(edge, &ordered_pass_ids) && !edge_is_satisfied(edge, &ordered_pass_ids)
        })
        .count();

    TransformPassPlanV0 {
        schema_version: "0",
        product: "omena-transform-passes.plan",
        requested_pass_ids,
        ordered_pass_ids,
        satisfied_dag_edge_count,
        violated_dag_edge_count,
        all_requested_registered: requested.iter().all(pass_is_registered),
    }
}

pub fn execute_transform_passes_on_source(
    source: &str,
    requested: &[TransformPassKind],
) -> TransformExecutionSummaryV0 {
    execute_transform_passes_on_source_with_dialect(source, StyleDialect::Css, requested)
}

pub fn execute_transform_passes_on_source_with_dialect(
    source: &str,
    dialect: StyleDialect,
    requested: &[TransformPassKind],
) -> TransformExecutionSummaryV0 {
    let context = TransformExecutionContextV0::default();
    execute_transform_passes_on_source_with_dialect_and_context(
        source, dialect, requested, &context,
    )
}

pub fn execute_transform_passes_on_source_with_dialect_and_context(
    source: &str,
    dialect: StyleDialect,
    requested: &[TransformPassKind],
    context: &TransformExecutionContextV0,
) -> TransformExecutionSummaryV0 {
    let pass_plan = plan_transform_passes(requested);
    let requested_pass_ids = requested.iter().map(|pass| pass.id()).collect::<Vec<_>>();
    let ordered_pass_ids = pass_plan.ordered_pass_ids.clone();
    let mut output_css = source.to_string();
    let mut outcomes = Vec::new();
    let mut css_module_evaluation = None;
    let mut css_import_inlines = Vec::new();
    let mut css_module_composes_exports = Vec::new();
    let mut design_token_routes = Vec::new();
    let mut semantic_removals = Vec::new();
    let mut outcome_mutation_spans = Vec::new();

    for pass_id in &ordered_pass_ids {
        let pass = transform_pass_kind_from_id(pass_id);
        let pass_input_css = output_css.clone();
        let input_byte_len = output_css.len();
        let outcome = match pass {
            Some(TransformPassKind::WhitespaceStrip) => {
                let (next_css, mutation_count) = normalize_css_whitespace(&output_css, dialect);
                let status = if mutation_count == 0 {
                    TransformPassRuntimeStatus::NoChange
                } else {
                    TransformPassRuntimeStatus::Applied
                };
                output_css = next_css;
                TransformPassExecutionOutcomeV0 {
                    pass_id,
                    status,
                    input_byte_len,
                    output_byte_len: output_css.len(),
                    mutation_count,
                    provenance_preserved: true,
                    detail: "normalized lexer trivia where adjacent token boundaries remain unambiguous",
                }
            }
            Some(TransformPassKind::CommentStrip) => {
                let (next_css, mutation_count) = strip_css_comments(&output_css, dialect);
                let status = if mutation_count == 0 {
                    TransformPassRuntimeStatus::NoChange
                } else {
                    TransformPassRuntimeStatus::Applied
                };
                output_css = next_css;
                TransformPassExecutionOutcomeV0 {
                    pass_id,
                    status,
                    input_byte_len,
                    output_byte_len: output_css.len(),
                    mutation_count,
                    provenance_preserved: true,
                    detail: "removed CSS block comments outside string literals",
                }
            }
            Some(TransformPassKind::NumberCompression) => {
                let (next_css, mutation_count) = compress_css_numbers(&output_css, dialect);
                let status = if mutation_count == 0 {
                    TransformPassRuntimeStatus::NoChange
                } else {
                    TransformPassRuntimeStatus::Applied
                };
                output_css = next_css;
                TransformPassExecutionOutcomeV0 {
                    pass_id,
                    status,
                    input_byte_len,
                    output_byte_len: output_css.len(),
                    mutation_count,
                    provenance_preserved: true,
                    detail: "compressed lexer numeric tokens without touching identifiers or strings",
                }
            }
            Some(TransformPassKind::UnitNormalization) => {
                let (next_css, mutation_count) = normalize_css_units(&output_css, dialect);
                let status = if mutation_count == 0 {
                    TransformPassRuntimeStatus::NoChange
                } else {
                    TransformPassRuntimeStatus::Applied
                };
                output_css = next_css;
                TransformPassExecutionOutcomeV0 {
                    pass_id,
                    status,
                    input_byte_len,
                    output_byte_len: output_css.len(),
                    mutation_count,
                    provenance_preserved: true,
                    detail: "normalized zero length units and known CSS unit casing inside declaration contexts",
                }
            }
            Some(TransformPassKind::ColorCompression) => {
                let (next_css, mutation_count) = compress_css_colors(&output_css, dialect);
                let status = if mutation_count == 0 {
                    TransformPassRuntimeStatus::NoChange
                } else {
                    TransformPassRuntimeStatus::Applied
                };
                output_css = next_css;
                TransformPassExecutionOutcomeV0 {
                    pass_id,
                    status,
                    input_byte_len,
                    output_byte_len: output_css.len(),
                    mutation_count,
                    provenance_preserved: true,
                    detail: "compressed static declaration color values and hex color tokens",
                }
            }
            Some(TransformPassKind::UrlQuoteStrip) => {
                let (next_css, mutation_count) = strip_css_url_quotes(&output_css, dialect);
                let status = if mutation_count == 0 {
                    TransformPassRuntimeStatus::NoChange
                } else {
                    TransformPassRuntimeStatus::Applied
                };
                output_css = next_css;
                TransformPassExecutionOutcomeV0 {
                    pass_id,
                    status,
                    input_byte_len,
                    output_byte_len: output_css.len(),
                    mutation_count,
                    provenance_preserved: true,
                    detail: "stripped quotes from safe url() string arguments",
                }
            }
            Some(TransformPassKind::StringQuoteNormalize) => {
                let (next_css, mutation_count) = normalize_css_string_quotes(&output_css, dialect);
                let status = if mutation_count == 0 {
                    TransformPassRuntimeStatus::NoChange
                } else {
                    TransformPassRuntimeStatus::Applied
                };
                output_css = next_css;
                TransformPassExecutionOutcomeV0 {
                    pass_id,
                    status,
                    input_byte_len,
                    output_byte_len: output_css.len(),
                    mutation_count,
                    provenance_preserved: true,
                    detail: "normalized safe single-quoted CSS string tokens",
                }
            }
            Some(TransformPassKind::SelectorIsWhereCompression) => {
                let (next_css, mutation_count) =
                    compress_css_is_where_selectors(&output_css, dialect);
                let status = if mutation_count == 0 {
                    TransformPassRuntimeStatus::NoChange
                } else {
                    TransformPassRuntimeStatus::Applied
                };
                output_css = next_css;
                TransformPassExecutionOutcomeV0 {
                    pass_id,
                    status,
                    input_byte_len,
                    output_byte_len: output_css.len(),
                    mutation_count,
                    provenance_preserved: true,
                    detail: "compressed :is/:where selector functions only when specificity and matching semantics are preserved",
                }
            }
            Some(TransformPassKind::ShorthandCombining) => {
                let (next_css, mutation_count) = combine_css_box_shorthands(&output_css, dialect);
                let status = if mutation_count == 0 {
                    TransformPassRuntimeStatus::NoChange
                } else {
                    TransformPassRuntimeStatus::Applied
                };
                output_css = next_css;
                TransformPassExecutionOutcomeV0 {
                    pass_id,
                    status,
                    input_byte_len,
                    output_byte_len: output_css.len(),
                    mutation_count,
                    provenance_preserved: true,
                    detail: "combined adjacent margin/padding longhands only with cascade shorthand proof",
                }
            }
            Some(TransformPassKind::RuleDeduplication) => {
                let (next_css, mutation_count) = dedupe_exact_css_rules(&output_css, dialect);
                let status = if mutation_count == 0 {
                    TransformPassRuntimeStatus::NoChange
                } else {
                    TransformPassRuntimeStatus::Applied
                };
                output_css = next_css;
                TransformPassExecutionOutcomeV0 {
                    pass_id,
                    status,
                    input_byte_len,
                    output_byte_len: output_css.len(),
                    mutation_count,
                    provenance_preserved: true,
                    detail: "removed cascade-safe duplicate ordinary rules while preserving the final occurrence",
                }
            }
            Some(TransformPassKind::RuleMerging) => {
                let (next_css, mutation_count) =
                    merge_adjacent_same_selector_css_rules(&output_css, dialect);
                let status = if mutation_count == 0 {
                    TransformPassRuntimeStatus::NoChange
                } else {
                    TransformPassRuntimeStatus::Applied
                };
                output_css = next_css;
                TransformPassExecutionOutcomeV0 {
                    pass_id,
                    status,
                    input_byte_len,
                    output_byte_len: output_css.len(),
                    mutation_count,
                    provenance_preserved: true,
                    detail: "merged adjacent same-selector ordinary rule runs without reordering declarations",
                }
            }
            Some(TransformPassKind::SelectorMerging) => {
                let (next_css, mutation_count) =
                    merge_adjacent_same_block_css_selectors(&output_css, dialect);
                let status = if mutation_count == 0 {
                    TransformPassRuntimeStatus::NoChange
                } else {
                    TransformPassRuntimeStatus::Applied
                };
                output_css = next_css;
                TransformPassExecutionOutcomeV0 {
                    pass_id,
                    status,
                    input_byte_len,
                    output_byte_len: output_css.len(),
                    mutation_count,
                    provenance_preserved: true,
                    detail: "merged adjacent ordinary rule runs with identical declaration blocks",
                }
            }
            Some(TransformPassKind::VendorPrefixing) => {
                let (next_css, mutation_count) = add_css_vendor_prefixes(&output_css, dialect);
                let status = if mutation_count == 0 {
                    TransformPassRuntimeStatus::NoChange
                } else {
                    TransformPassRuntimeStatus::Applied
                };
                output_css = next_css;
                TransformPassExecutionOutcomeV0 {
                    pass_id,
                    status,
                    input_byte_len,
                    output_byte_len: output_css.len(),
                    mutation_count,
                    provenance_preserved: true,
                    detail: "inserted conservative vendor-prefixed declaration synonyms when absent",
                }
            }
            Some(TransformPassKind::LightDarkLowering) => {
                let (next_css, mutation_count) = lower_css_light_dark(&output_css, dialect);
                let status = if mutation_count == 0 {
                    TransformPassRuntimeStatus::NoChange
                } else {
                    TransformPassRuntimeStatus::Applied
                };
                output_css = next_css;
                TransformPassExecutionOutcomeV0 {
                    pass_id,
                    status,
                    input_byte_len,
                    output_byte_len: output_css.len(),
                    mutation_count,
                    provenance_preserved: true,
                    detail: "lowered whole-value light-dark() color declarations into dark media branches",
                }
            }
            Some(TransformPassKind::ColorMixLowering) => {
                let (next_css, mutation_count) = lower_css_color_mix(&output_css, dialect);
                let status = if mutation_count == 0 {
                    TransformPassRuntimeStatus::NoChange
                } else {
                    TransformPassRuntimeStatus::Applied
                };
                output_css = next_css;
                TransformPassExecutionOutcomeV0 {
                    pass_id,
                    status,
                    input_byte_len,
                    output_byte_len: output_css.len(),
                    mutation_count,
                    provenance_preserved: true,
                    detail: "lowered static srgb color-mix() references with static color operands",
                }
            }
            Some(TransformPassKind::OklchOklabLowering) => {
                let (next_css, mutation_count) = lower_css_oklab_oklch(&output_css, dialect);
                let status = if mutation_count == 0 {
                    TransformPassRuntimeStatus::NoChange
                } else {
                    TransformPassRuntimeStatus::Applied
                };
                output_css = next_css;
                TransformPassExecutionOutcomeV0 {
                    pass_id,
                    status,
                    input_byte_len,
                    output_byte_len: output_css.len(),
                    mutation_count,
                    provenance_preserved: true,
                    detail: "lowered in-gamut oklab()/oklch() color references to srgb",
                }
            }
            Some(TransformPassKind::ColorFunctionLowering) => {
                let (next_css, mutation_count) = lower_css_color_function(&output_css, dialect);
                let status = if mutation_count == 0 {
                    TransformPassRuntimeStatus::NoChange
                } else {
                    TransformPassRuntimeStatus::Applied
                };
                output_css = next_css;
                TransformPassExecutionOutcomeV0 {
                    pass_id,
                    status,
                    input_byte_len,
                    output_byte_len: output_css.len(),
                    mutation_count,
                    provenance_preserved: true,
                    detail: "lowered static color(...) references with static channels",
                }
            }
            Some(TransformPassKind::LogicalToPhysical) => {
                let (next_css, mutation_count) =
                    lower_css_logical_to_physical(&output_css, dialect);
                let status = if mutation_count == 0 {
                    TransformPassRuntimeStatus::NoChange
                } else {
                    TransformPassRuntimeStatus::Applied
                };
                output_css = next_css;
                TransformPassExecutionOutcomeV0 {
                    pass_id,
                    status,
                    input_byte_len,
                    output_byte_len: output_css.len(),
                    mutation_count,
                    provenance_preserved: true,
                    detail: "lowered logical properties only under static horizontal writing direction",
                }
            }
            Some(TransformPassKind::NestingUnwrap) => {
                let (next_css, mutation_count) = unwrap_css_nesting(&output_css, dialect);
                let status = if mutation_count == 0 {
                    TransformPassRuntimeStatus::NoChange
                } else {
                    TransformPassRuntimeStatus::Applied
                };
                output_css = next_css;
                TransformPassExecutionOutcomeV0 {
                    pass_id,
                    status,
                    input_byte_len,
                    output_byte_len: output_css.len(),
                    mutation_count,
                    provenance_preserved: true,
                    detail: "unwrapped nested ordinary rules and conditional group rules",
                }
            }
            Some(TransformPassKind::ScopeFlatten) => {
                let (next_css, mutation_count) = flatten_css_scopes(&output_css, dialect);
                let status = if mutation_count == 0 {
                    TransformPassRuntimeStatus::NoChange
                } else {
                    TransformPassRuntimeStatus::Applied
                };
                output_css = next_css;
                TransformPassExecutionOutcomeV0 {
                    pass_id,
                    status,
                    input_byte_len,
                    output_byte_len: output_css.len(),
                    mutation_count,
                    provenance_preserved: true,
                    detail: "flattened only @scope candidates accepted by the cascade scope-flatten proof",
                }
            }
            Some(TransformPassKind::LayerFlatten) if context.closed_style_world => {
                let (next_css, mutation_count) = flatten_css_layers(&output_css, dialect, true);
                let status = if mutation_count == 0 {
                    TransformPassRuntimeStatus::NoChange
                } else {
                    TransformPassRuntimeStatus::Applied
                };
                output_css = next_css;
                TransformPassExecutionOutcomeV0 {
                    pass_id,
                    status,
                    input_byte_len,
                    output_byte_len: output_css.len(),
                    mutation_count,
                    provenance_preserved: true,
                    detail: "flattened only @layer candidates accepted by the closed-bundle cascade proof",
                }
            }
            Some(TransformPassKind::LayerFlatten) => TransformPassExecutionOutcomeV0 {
                pass_id,
                status: TransformPassRuntimeStatus::PlannedOnly,
                input_byte_len,
                output_byte_len: output_css.len(),
                mutation_count: 0,
                provenance_preserved: true,
                detail: "requires an explicit closed-style-world bundle witness before mutation",
            },
            Some(TransformPassKind::SupportsStaticEval) => {
                let (next_css, mutation_count) =
                    evaluate_static_supports_rules(&output_css, dialect);
                let status = if mutation_count == 0 {
                    TransformPassRuntimeStatus::NoChange
                } else {
                    TransformPassRuntimeStatus::Applied
                };
                output_css = next_css;
                TransformPassExecutionOutcomeV0 {
                    pass_id,
                    status,
                    input_byte_len,
                    output_byte_len: output_css.len(),
                    mutation_count,
                    provenance_preserved: true,
                    detail: "evaluated simple @supports branches with cascade supports-static witness",
                }
            }
            Some(TransformPassKind::MediaStaticEval) => {
                let (next_css, mutation_count) = evaluate_static_media_rules(&output_css, dialect);
                let status = if mutation_count == 0 {
                    TransformPassRuntimeStatus::NoChange
                } else {
                    TransformPassRuntimeStatus::Applied
                };
                output_css = next_css;
                TransformPassExecutionOutcomeV0 {
                    pass_id,
                    status,
                    input_byte_len,
                    output_byte_len: output_css.len(),
                    mutation_count,
                    provenance_preserved: true,
                    detail: "evaluated literal @media all/not all branches",
                }
            }
            Some(TransformPassKind::DeadMediaBranchRemoval) => {
                let (next_css, mutation_count) =
                    evaluate_dead_media_branch_rules(&output_css, dialect, context);
                let status = if mutation_count == 0 {
                    TransformPassRuntimeStatus::NoChange
                } else {
                    TransformPassRuntimeStatus::Applied
                };
                output_css = next_css;
                TransformPassExecutionOutcomeV0 {
                    pass_id,
                    status,
                    input_byte_len,
                    output_byte_len: output_css.len(),
                    mutation_count,
                    provenance_preserved: true,
                    detail: "removed dead @media branches through the static cascade witness evaluator",
                }
            }
            Some(TransformPassKind::DeadSupportsBranchRemoval) => {
                let (next_css, mutation_count) =
                    evaluate_static_supports_rules(&output_css, dialect);
                let status = if mutation_count == 0 {
                    TransformPassRuntimeStatus::NoChange
                } else {
                    TransformPassRuntimeStatus::Applied
                };
                output_css = next_css;
                TransformPassExecutionOutcomeV0 {
                    pass_id,
                    status,
                    input_byte_len,
                    output_byte_len: output_css.len(),
                    mutation_count,
                    provenance_preserved: true,
                    detail: "removed dead @supports branches through the static cascade witness evaluator",
                }
            }
            Some(TransformPassKind::ScssModuleEvaluate)
                if matches!(dialect, StyleDialect::Scss | StyleDialect::Sass) =>
            {
                if let Some(evaluation) = context.scss_module_evaluation.as_ref() {
                    let mutation_count = usize::from(output_css != evaluation.evaluated_css);
                    let status = if mutation_count == 0 {
                        TransformPassRuntimeStatus::NoChange
                    } else {
                        TransformPassRuntimeStatus::Applied
                    };
                    output_css = evaluation.evaluated_css.clone();
                    css_module_evaluation = Some(evaluation.clone());
                    TransformPassExecutionOutcomeV0 {
                        pass_id,
                        status,
                        input_byte_len,
                        output_byte_len: output_css.len(),
                        mutation_count,
                        provenance_preserved: true,
                        detail: "applied explicit SCSS module evaluation output from the evaluator boundary",
                    }
                } else {
                    TransformPassExecutionOutcomeV0 {
                        pass_id,
                        status: TransformPassRuntimeStatus::PlannedOnly,
                        input_byte_len,
                        output_byte_len: output_css.len(),
                        mutation_count: 0,
                        provenance_preserved: true,
                        detail: "requires explicit SCSS evaluator output before mutation",
                    }
                }
            }
            Some(TransformPassKind::ScssModuleEvaluate) => TransformPassExecutionOutcomeV0 {
                pass_id,
                status: TransformPassRuntimeStatus::PlannedOnly,
                input_byte_len,
                output_byte_len: output_css.len(),
                mutation_count: 0,
                provenance_preserved: true,
                detail: "requires explicit SCSS evaluator output before mutation",
            },
            Some(TransformPassKind::LessModuleEvaluate) if dialect == StyleDialect::Less => {
                if let Some(evaluation) = context.less_module_evaluation.as_ref() {
                    let mutation_count = usize::from(output_css != evaluation.evaluated_css);
                    let status = if mutation_count == 0 {
                        TransformPassRuntimeStatus::NoChange
                    } else {
                        TransformPassRuntimeStatus::Applied
                    };
                    output_css = evaluation.evaluated_css.clone();
                    css_module_evaluation = Some(evaluation.clone());
                    TransformPassExecutionOutcomeV0 {
                        pass_id,
                        status,
                        input_byte_len,
                        output_byte_len: output_css.len(),
                        mutation_count,
                        provenance_preserved: true,
                        detail: "applied explicit Less module evaluation output from the evaluator boundary",
                    }
                } else {
                    TransformPassExecutionOutcomeV0 {
                        pass_id,
                        status: TransformPassRuntimeStatus::PlannedOnly,
                        input_byte_len,
                        output_byte_len: output_css.len(),
                        mutation_count: 0,
                        provenance_preserved: true,
                        detail: "requires explicit Less evaluator output before mutation",
                    }
                }
            }
            Some(TransformPassKind::LessModuleEvaluate) => TransformPassExecutionOutcomeV0 {
                pass_id,
                status: TransformPassRuntimeStatus::PlannedOnly,
                input_byte_len,
                output_byte_len: output_css.len(),
                mutation_count: 0,
                provenance_preserved: true,
                detail: "requires explicit Less evaluator output before mutation",
            },
            Some(TransformPassKind::ImportInline) if !context.import_inlines.is_empty() => {
                let (next_css, mutation_count) =
                    inline_css_imports(&output_css, dialect, &context.import_inlines);
                let status = if mutation_count == 0 {
                    TransformPassRuntimeStatus::NoChange
                } else {
                    TransformPassRuntimeStatus::Applied
                };
                output_css = next_css;
                css_import_inlines = context.import_inlines.clone();
                TransformPassExecutionOutcomeV0 {
                    pass_id,
                    status,
                    input_byte_len,
                    output_byte_len: output_css.len(),
                    mutation_count,
                    provenance_preserved: true,
                    detail: "replaced resolved @import directives using explicit inline CSS replacements",
                }
            }
            Some(TransformPassKind::ImportInline) => TransformPassExecutionOutcomeV0 {
                pass_id,
                status: TransformPassRuntimeStatus::PlannedOnly,
                input_byte_len,
                output_byte_len: output_css.len(),
                mutation_count: 0,
                provenance_preserved: true,
                detail: "requires explicit resolved import replacements before mutation",
            },
            Some(TransformPassKind::ResolveCssModulesComposes)
                if !context.css_module_composes_resolutions.is_empty() =>
            {
                let (next_css, mutation_count) = resolve_css_module_composes(
                    &output_css,
                    dialect,
                    &context.css_module_composes_resolutions,
                );
                let status = if mutation_count == 0 {
                    TransformPassRuntimeStatus::NoChange
                } else {
                    TransformPassRuntimeStatus::Applied
                };
                output_css = next_css;
                css_module_composes_exports = context.css_module_composes_resolutions.clone();
                TransformPassExecutionOutcomeV0 {
                    pass_id,
                    status,
                    input_byte_len,
                    output_byte_len: output_css.len(),
                    mutation_count,
                    provenance_preserved: true,
                    detail: "removed resolved CSS Modules composes declarations using an explicit export set",
                }
            }
            Some(TransformPassKind::ResolveCssModulesComposes) => TransformPassExecutionOutcomeV0 {
                pass_id,
                status: TransformPassRuntimeStatus::PlannedOnly,
                input_byte_len,
                output_byte_len: output_css.len(),
                mutation_count: 0,
                provenance_preserved: true,
                detail: "requires an explicit CSS Modules composes export set before mutation",
            },
            Some(TransformPassKind::DesignTokenRouting)
                if !context.design_token_routes.is_empty() =>
            {
                let (next_css, mutation_count) =
                    route_design_token_values(&output_css, dialect, &context.design_token_routes);
                let status = if mutation_count == 0 {
                    TransformPassRuntimeStatus::NoChange
                } else {
                    TransformPassRuntimeStatus::Applied
                };
                output_css = next_css;
                design_token_routes = context.design_token_routes.clone();
                TransformPassExecutionOutcomeV0 {
                    pass_id,
                    status,
                    input_byte_len,
                    output_byte_len: output_css.len(),
                    mutation_count,
                    provenance_preserved: true,
                    detail: "routed whole-value design-token references through explicit bridge token routes",
                }
            }
            Some(TransformPassKind::DesignTokenRouting) => TransformPassExecutionOutcomeV0 {
                pass_id,
                status: TransformPassRuntimeStatus::PlannedOnly,
                input_byte_len,
                output_byte_len: output_css.len(),
                mutation_count: 0,
                provenance_preserved: true,
                detail: "requires explicit bridge design-token routes before mutation",
            },
            Some(TransformPassKind::HashCssModuleClassNames)
                if !context.class_name_rewrites.is_empty() =>
            {
                let (next_css, mutation_count) = rewrite_css_module_class_names(
                    &output_css,
                    dialect,
                    &context.class_name_rewrites,
                );
                let status = if mutation_count == 0 {
                    TransformPassRuntimeStatus::NoChange
                } else {
                    TransformPassRuntimeStatus::Applied
                };
                output_css = next_css;
                TransformPassExecutionOutcomeV0 {
                    pass_id,
                    status,
                    input_byte_len,
                    output_byte_len: output_css.len(),
                    mutation_count,
                    provenance_preserved: true,
                    detail: "rewrote CSS Modules class selectors through an explicit selector identity map",
                }
            }
            Some(TransformPassKind::HashCssModuleClassNames) => TransformPassExecutionOutcomeV0 {
                pass_id,
                status: TransformPassRuntimeStatus::PlannedOnly,
                input_byte_len,
                output_byte_len: output_css.len(),
                mutation_count: 0,
                provenance_preserved: true,
                detail: "requires an explicit selector identity map before mutation",
            },
            Some(TransformPassKind::TreeShakeClass) if context.closed_style_world => {
                let (next_css, removals) = tree_shake_css_class_rules_with_removals(
                    &output_css,
                    dialect,
                    &context.reachable_class_names,
                );
                let mutation_count = removals.len();
                let status = if mutation_count == 0 {
                    TransformPassRuntimeStatus::NoChange
                } else {
                    TransformPassRuntimeStatus::Applied
                };
                output_css = next_css;
                semantic_removals.extend(
                    removals
                        .into_iter()
                        .map(|removal| removal.into_public(pass_id)),
                );
                TransformPassExecutionOutcomeV0 {
                    pass_id,
                    status,
                    input_byte_len,
                    output_byte_len: output_css.len(),
                    mutation_count,
                    provenance_preserved: true,
                    detail: "removed unreachable class-owned selector rules under an explicit closed-style-world reachability context",
                }
            }
            Some(TransformPassKind::TreeShakeClass) => TransformPassExecutionOutcomeV0 {
                pass_id,
                status: TransformPassRuntimeStatus::PlannedOnly,
                input_byte_len,
                output_byte_len: output_css.len(),
                mutation_count: 0,
                provenance_preserved: true,
                detail: "requires an explicit closed-style-world reachability context before mutation",
            },
            Some(TransformPassKind::TreeShakeKeyframes) if context.closed_style_world => {
                let (next_css, removals) = tree_shake_css_keyframes_with_removals(
                    &output_css,
                    dialect,
                    &context.reachable_keyframe_names,
                );
                let mutation_count = removals.len();
                let status = if mutation_count == 0 {
                    TransformPassRuntimeStatus::NoChange
                } else {
                    TransformPassRuntimeStatus::Applied
                };
                output_css = next_css;
                semantic_removals.extend(
                    removals
                        .into_iter()
                        .map(|removal| removal.into_public(pass_id)),
                );
                TransformPassExecutionOutcomeV0 {
                    pass_id,
                    status,
                    input_byte_len,
                    output_byte_len: output_css.len(),
                    mutation_count,
                    provenance_preserved: true,
                    detail: "removed unreferenced @keyframes under an explicit closed-style-world reachability context",
                }
            }
            Some(TransformPassKind::TreeShakeKeyframes) => TransformPassExecutionOutcomeV0 {
                pass_id,
                status: TransformPassRuntimeStatus::PlannedOnly,
                input_byte_len,
                output_byte_len: output_css.len(),
                mutation_count: 0,
                provenance_preserved: true,
                detail: "requires an explicit closed-style-world reachability context before mutation",
            },
            Some(TransformPassKind::TreeShakeValue) if context.closed_style_world => {
                let (next_css, removals) = tree_shake_css_modules_values_with_removals(
                    &output_css,
                    dialect,
                    &context.reachable_value_names,
                );
                let mutation_count = removals.len();
                let status = if mutation_count == 0 {
                    TransformPassRuntimeStatus::NoChange
                } else {
                    TransformPassRuntimeStatus::Applied
                };
                output_css = next_css;
                semantic_removals.extend(
                    removals
                        .into_iter()
                        .map(|removal| removal.into_public(pass_id)),
                );
                TransformPassExecutionOutcomeV0 {
                    pass_id,
                    status,
                    input_byte_len,
                    output_byte_len: output_css.len(),
                    mutation_count,
                    provenance_preserved: true,
                    detail: "removed unreachable local CSS Modules @value declarations under an explicit closed-style-world reachability context",
                }
            }
            Some(TransformPassKind::TreeShakeValue) => TransformPassExecutionOutcomeV0 {
                pass_id,
                status: TransformPassRuntimeStatus::PlannedOnly,
                input_byte_len,
                output_byte_len: output_css.len(),
                mutation_count: 0,
                provenance_preserved: true,
                detail: "requires an explicit closed-style-world reachability context before mutation",
            },
            Some(TransformPassKind::TreeShakeCustomProperty) if context.closed_style_world => {
                let (next_css, removals) = tree_shake_css_custom_properties_with_removals(
                    &output_css,
                    dialect,
                    &context.reachable_custom_property_names,
                );
                let mutation_count = removals.len();
                let status = if mutation_count == 0 {
                    TransformPassRuntimeStatus::NoChange
                } else {
                    TransformPassRuntimeStatus::Applied
                };
                output_css = next_css;
                semantic_removals.extend(
                    removals
                        .into_iter()
                        .map(|removal| removal.into_public(pass_id)),
                );
                TransformPassExecutionOutcomeV0 {
                    pass_id,
                    status,
                    input_byte_len,
                    output_byte_len: output_css.len(),
                    mutation_count,
                    provenance_preserved: true,
                    detail: "removed unreachable custom-property declarations under an explicit closed-style-world reachability context",
                }
            }
            Some(TransformPassKind::TreeShakeCustomProperty) => TransformPassExecutionOutcomeV0 {
                pass_id,
                status: TransformPassRuntimeStatus::PlannedOnly,
                input_byte_len,
                output_byte_len: output_css.len(),
                mutation_count: 0,
                provenance_preserved: true,
                detail: "requires an explicit closed-style-world reachability context before mutation",
            },
            Some(TransformPassKind::ValueResolution) => {
                let (next_css, mutation_count) =
                    resolve_static_css_modules_values(&output_css, dialect);
                let status = if mutation_count == 0 {
                    TransformPassRuntimeStatus::NoChange
                } else {
                    TransformPassRuntimeStatus::Applied
                };
                output_css = next_css;
                TransformPassExecutionOutcomeV0 {
                    pass_id,
                    status,
                    input_byte_len,
                    output_byte_len: output_css.len(),
                    mutation_count,
                    provenance_preserved: true,
                    detail: "resolved whole-value references from unique local literal CSS Modules @value declarations",
                }
            }
            Some(TransformPassKind::StaticVarSubstitution) => {
                let (next_css, mutation_count) =
                    substitute_static_css_custom_properties(&output_css, dialect);
                let status = if mutation_count == 0 {
                    TransformPassRuntimeStatus::NoChange
                } else {
                    TransformPassRuntimeStatus::Applied
                };
                output_css = next_css;
                TransformPassExecutionOutcomeV0 {
                    pass_id,
                    status,
                    input_byte_len,
                    output_byte_len: output_css.len(),
                    mutation_count,
                    provenance_preserved: true,
                    detail: "resolved whole-value var() references from unique static :root custom properties",
                }
            }
            Some(TransformPassKind::CalcReduction) => {
                let (next_css, mutation_count) = reduce_css_calc(&output_css, dialect);
                let status = if mutation_count == 0 {
                    TransformPassRuntimeStatus::NoChange
                } else {
                    TransformPassRuntimeStatus::Applied
                };
                output_css = next_css;
                TransformPassExecutionOutcomeV0 {
                    pass_id,
                    status,
                    input_byte_len,
                    output_byte_len: output_css.len(),
                    mutation_count,
                    provenance_preserved: true,
                    detail: "reduced whole-value calc() expressions with simple same-unit addition/subtraction",
                }
            }
            Some(TransformPassKind::EmptyRuleRemoval) => {
                let (next_css, mutation_count) = remove_empty_css_rules(&output_css, dialect);
                let status = if mutation_count == 0 {
                    TransformPassRuntimeStatus::NoChange
                } else {
                    TransformPassRuntimeStatus::Applied
                };
                output_css = next_css;
                TransformPassExecutionOutcomeV0 {
                    pass_id,
                    status,
                    input_byte_len,
                    output_byte_len: output_css.len(),
                    mutation_count,
                    provenance_preserved: true,
                    detail: "removed ordinary empty rules with no comments or at-rule semantics",
                }
            }
            Some(TransformPassKind::PrintCss) => TransformPassExecutionOutcomeV0 {
                pass_id,
                status: TransformPassRuntimeStatus::NoChange,
                input_byte_len,
                output_byte_len: output_css.len(),
                mutation_count: 0,
                provenance_preserved: true,
                detail: "observed final emission boundary",
            },
            None => TransformPassExecutionOutcomeV0 {
                pass_id,
                status: TransformPassRuntimeStatus::PlannedOnly,
                input_byte_len,
                output_byte_len: output_css.len(),
                mutation_count: 0,
                provenance_preserved: true,
                detail: "unknown pass id in execution plan",
            },
        };
        outcome_mutation_spans.push(derive_transform_mutation_spans(
            &pass_input_css,
            &output_css,
        ));
        outcomes.push(outcome);
    }

    let executed_pass_ids = outcomes
        .iter()
        .filter(|outcome| outcome.status != TransformPassRuntimeStatus::PlannedOnly)
        .map(|outcome| outcome.pass_id)
        .collect::<Vec<_>>();
    let planned_only_pass_ids = outcomes
        .iter()
        .filter(|outcome| outcome.status == TransformPassRuntimeStatus::PlannedOnly)
        .map(|outcome| outcome.pass_id)
        .collect::<Vec<_>>();
    let mutation_count = outcomes
        .iter()
        .map(|outcome| outcome.mutation_count)
        .sum::<usize>();
    let provenance_preserved = outcomes.iter().all(|outcome| outcome.provenance_preserved);
    let provenance_derivation_forest =
        provenance_derivation_forest_from_outcomes(&outcomes, &outcome_mutation_spans);
    let output_byte_len = output_css.len();

    TransformExecutionSummaryV0 {
        schema_version: "0",
        product: "omena-transform-passes.execution",
        input_byte_len: source.len(),
        output_byte_len,
        requested_pass_ids,
        ordered_pass_ids,
        executed_pass_ids,
        planned_only_pass_ids,
        mutation_count,
        provenance_preserved,
        output_css,
        css_module_evaluation,
        css_import_inlines,
        css_module_composes_exports,
        design_token_routes,
        semantic_removals,
        provenance_derivation_forest,
        outcomes,
        pass_plan,
    }
}

pub fn execute_transform_passes_incremental_with_database(
    source: &str,
    dialect: StyleDialect,
    requested: &[TransformPassKind],
    context: &TransformExecutionContextV0,
    incremental_database: &mut OmenaIncrementalDatabaseV0,
    previous_execution: Option<&TransformExecutionSummaryV0>,
    revision: IncrementalRevisionV0,
) -> TransformIncrementalExecutionSummaryV0 {
    let incremental_input =
        transform_pass_incremental_graph_input(source, dialect, requested, context, revision);
    let update = incremental_database.plan_and_upsert_graph_input(&incremental_input);
    let reused_previous_execution =
        update.incremental_plan.dirty_node_count == 0 && previous_execution.is_some();
    let execution = match (reused_previous_execution, previous_execution) {
        (true, Some(previous_execution)) => previous_execution.clone(),
        _ => execute_transform_passes_on_source_with_dialect_and_context(
            source, dialect, requested, context,
        ),
    };

    TransformIncrementalExecutionSummaryV0 {
        schema_version: "0",
        product: "omena-transform-passes.incremental-execution",
        incremental_engine: "omena-incremental",
        query_model: "persistentSalsaDatabase+transformPassDependencyGraph",
        reuse_policy: "reuse previous transform execution when the omena-incremental plan is clean",
        reused_previous_execution,
        incremental_plan: update.incremental_plan,
        next_snapshot: update.next_snapshot,
        execution,
        ready_surfaces: vec![
            "transformSalsaQueries",
            "transformPassIncrementalGraph",
            "cleanTransformExecutionReuse",
        ],
    }
}

pub fn run_transform_cascade_safe_fuzz_case(
    case: TransformCascadeSafetyFuzzCaseV0,
) -> TransformCascadeSafetyFuzzResultV0 {
    let pass_count = case.pass_count.clamp(1, TRANSFORM_PASS_CATALOG_LEN);
    let source = generated_transform_fuzz_source(case.seed);
    let requested = generated_transform_fuzz_passes(case.seed, pass_count);
    let context = generated_transform_fuzz_context(case.seed);
    let execution = execute_transform_passes_on_source_with_dialect_and_context(
        &source,
        StyleDialect::Css,
        &requested,
        &context,
    );
    let lexed_output = lex(&execution.output_css, StyleDialect::Css);
    let requested_pass_ids = requested.iter().map(|pass| pass.id()).collect::<Vec<_>>();
    let output_byte_len = execution.output_css.len();
    let output_token_count = lexed_output.tokens().len();
    let output_error_count = lexed_output.errors().len();
    let provenance_node_count = execution.provenance_derivation_forest.node_count;
    let passed = execution.pass_plan.violated_dag_edge_count == 0
        && output_error_count == 0
        && output_byte_len <= source.len().saturating_mul(4).saturating_add(256)
        && provenance_node_count == execution.outcomes.len()
        && execution.provenance_preserved
            == execution
                .outcomes
                .iter()
                .all(|outcome| outcome.provenance_preserved);

    TransformCascadeSafetyFuzzResultV0 {
        seed: case.seed,
        pass_count,
        requested_pass_ids,
        executed_pass_ids: execution.executed_pass_ids,
        output_byte_len,
        output_token_count,
        output_error_count,
        provenance_node_count,
        passed,
    }
}

pub fn run_transform_fuzz_seed_corpus() -> TransformFuzzSeedReportV0 {
    let seeds = [1, 2, 3, 5, 8, 13, 21, 34, 55, 89, 144, 233];
    let results = seeds
        .into_iter()
        .enumerate()
        .map(|(index, seed)| {
            run_transform_cascade_safe_fuzz_case(TransformCascadeSafetyFuzzCaseV0 {
                seed,
                pass_count: index + 1,
            })
        })
        .collect::<Vec<_>>();
    let passed_count = results.iter().filter(|result| result.passed).count();
    let case_count = results.len();

    TransformFuzzSeedReportV0 {
        schema_version: "0",
        product: "omena-transform-passes.fuzz-seed-corpus",
        case_count,
        passed_count,
        failed_count: case_count - passed_count,
        results,
    }
}

pub fn transform_pass_incremental_graph_input(
    source: &str,
    dialect: StyleDialect,
    requested: &[TransformPassKind],
    context: &TransformExecutionContextV0,
    revision: IncrementalRevisionV0,
) -> IncrementalGraphInputV0 {
    let pass_plan = plan_transform_passes(requested);
    let dialect_label = transform_style_dialect_label(dialect);
    let context_digest = transform_execution_context_digest(context);
    let ordered_pass_ids = pass_plan.ordered_pass_ids.join("|");
    let mut nodes = vec![
        IncrementalNodeInputV0 {
            id: "transform:source".to_string(),
            digest: stable_transform_digest(&["source", dialect_label, source]),
            dependency_ids: Vec::new(),
        },
        IncrementalNodeInputV0 {
            id: "transform:context".to_string(),
            digest: stable_transform_digest(&["context", context_digest.as_str()]),
            dependency_ids: Vec::new(),
        },
        IncrementalNodeInputV0 {
            id: "transform:plan".to_string(),
            digest: stable_transform_digest(&["plan", ordered_pass_ids.as_str()]),
            dependency_ids: Vec::new(),
        },
    ];

    let mut previous_pass_node_id = None;
    for pass_id in pass_plan.ordered_pass_ids {
        let node_id = format!("transform:pass:{pass_id}");
        let mut dependency_ids = vec![
            "transform:source".to_string(),
            "transform:context".to_string(),
            "transform:plan".to_string(),
        ];
        if let Some(previous_pass_node_id) = previous_pass_node_id {
            dependency_ids.push(previous_pass_node_id);
        }

        nodes.push(IncrementalNodeInputV0 {
            id: node_id.clone(),
            digest: stable_transform_digest(&["pass", pass_id]),
            dependency_ids,
        });
        previous_pass_node_id = Some(node_id);
    }

    let mut execution_dependency_ids = vec![
        "transform:source".to_string(),
        "transform:context".to_string(),
        "transform:plan".to_string(),
    ];
    if let Some(previous_pass_node_id) = previous_pass_node_id {
        execution_dependency_ids.push(previous_pass_node_id);
    }
    nodes.push(IncrementalNodeInputV0 {
        id: "transform:execution".to_string(),
        digest: stable_transform_digest(&["execution", ordered_pass_ids.as_str()]),
        dependency_ids: execution_dependency_ids,
    });

    IncrementalGraphInputV0 { revision, nodes }
}

fn transform_style_dialect_label(dialect: StyleDialect) -> &'static str {
    match dialect {
        StyleDialect::Css => "css",
        StyleDialect::Scss => "scss",
        StyleDialect::Sass => "sass",
        StyleDialect::Less => "less",
    }
}

fn transform_execution_context_digest(context: &TransformExecutionContextV0) -> String {
    let serialized = match serde_json::to_string(context) {
        Ok(serialized) => serialized,
        Err(error) => format!("serialization-error:{error}"),
    };
    stable_transform_digest(&["transform-context", serialized.as_str()])
}

fn stable_transform_digest(parts: &[&str]) -> String {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for part in parts {
        for byte in part.as_bytes() {
            hash ^= u64::from(*byte);
            hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
        }
        hash ^= 0xff;
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("fnv1a64:{hash:016x}")
}

fn generated_transform_fuzz_source(seed: u64) -> String {
    let mut state = seed ^ 0xe703_7ed1_a0b4_28db;
    let spacing = if fuzz_transform_next(&mut state).is_multiple_of(2) {
        "  "
    } else {
        "\n  "
    };
    let color = ["red", "blue", "oklch(60% 0.2 20)", "#ff0000"]
        [(fuzz_transform_next(&mut state) % 4) as usize];
    let margin = fuzz_transform_next(&mut state) % 24;
    format!(
        "/* fuzz:{seed} */\n.button-{seed} {{{spacing}--brand: {color};{spacing}color: var(--brand, red);{spacing}margin: {margin}.0px;{spacing}&__icon {{ color: var(--brand); }}\n}}\n@media (min-width: 40rem) {{ .button-{seed} {{ margin: {margin}px; }} }}\n"
    )
}

fn generated_transform_fuzz_passes(seed: u64, pass_count: usize) -> Vec<TransformPassKind> {
    let all_passes = all_transform_pass_kinds();
    let mut state = seed;
    let mut passes = Vec::new();
    for _ in 0..pass_count {
        let index = (fuzz_transform_next(&mut state) % all_passes.len() as u64) as usize;
        let pass = all_passes[index];
        if !passes.contains(&pass) {
            passes.push(pass);
        }
    }
    if passes.is_empty() {
        passes.push(TransformPassKind::PrintCss);
    }
    passes
}

fn generated_transform_fuzz_context(seed: u64) -> TransformExecutionContextV0 {
    let class_name = format!("button-{seed}");
    TransformExecutionContextV0 {
        closed_style_world: seed.is_multiple_of(2),
        reachable_class_names: vec![class_name.clone(), format!("{class_name}__icon")],
        reachable_keyframe_names: vec![format!("fade-{seed}")],
        reachable_value_names: vec![format!("spacing-{seed}")],
        reachable_custom_property_names: vec!["--brand".to_string()],
        class_name_rewrites: vec![TransformClassNameRewriteV0 {
            original_name: class_name,
            rewritten_name: format!("button-{seed}_hash"),
        }],
        design_token_routes: vec![TransformDesignTokenRouteV0 {
            token_name: "--brand".to_string(),
            routed_value: "var(--brand)".to_string(),
        }],
        ..TransformExecutionContextV0::default()
    }
}

fn fuzz_transform_next(state: &mut u64) -> u64 {
    *state = state
        .wrapping_mul(6_364_136_223_846_793_005)
        .wrapping_add(1_442_695_040_888_963_407);
    *state
}

fn provenance_derivation_forest_from_outcomes(
    outcomes: &[TransformPassExecutionOutcomeV0],
    outcome_mutation_spans: &[Vec<TransformProvenanceMutationSpanV0>],
) -> TransformProvenanceDerivationForestV0 {
    let nodes = outcomes
        .iter()
        .enumerate()
        .map(|(index, outcome)| {
            let mutation_spans = outcome_mutation_spans
                .get(index)
                .cloned()
                .unwrap_or_default();
            let (source_span_start, source_span_end, generated_span_start, generated_span_end) =
                provenance_node_span_envelope(
                    outcome.input_byte_len,
                    outcome.output_byte_len,
                    mutation_spans.as_slice(),
                );

            TransformProvenanceDerivationNodeV0 {
                node_index: index,
                parent_index: index.checked_sub(1),
                pass_id: outcome.pass_id,
                status: outcome.status,
                input_byte_len: outcome.input_byte_len,
                output_byte_len: outcome.output_byte_len,
                source_span_start,
                source_span_end,
                generated_span_start,
                generated_span_end,
                mutation_spans,
                mutation_count: outcome.mutation_count,
                provenance_preserved: outcome.provenance_preserved,
                detail: outcome.detail,
            }
        })
        .collect::<Vec<_>>();

    TransformProvenanceDerivationForestV0 {
        schema_version: "0",
        product: "omena-transform-passes.provenance-derivation-forest",
        root_count: usize::from(!nodes.is_empty()),
        node_count: nodes.len(),
        nodes,
    }
}

fn provenance_node_span_envelope(
    input_byte_len: usize,
    output_byte_len: usize,
    mutation_spans: &[TransformProvenanceMutationSpanV0],
) -> (usize, usize, usize, usize) {
    if mutation_spans.is_empty() {
        return (0, input_byte_len, 0, output_byte_len);
    }

    let source_span_start = mutation_spans
        .iter()
        .map(|span| span.source_span_start)
        .min()
        .unwrap_or(0);
    let source_span_end = mutation_spans
        .iter()
        .map(|span| span.source_span_end)
        .max()
        .unwrap_or(input_byte_len);
    let generated_span_start = mutation_spans
        .iter()
        .map(|span| span.generated_span_start)
        .min()
        .unwrap_or(0);
    let generated_span_end = mutation_spans
        .iter()
        .map(|span| span.generated_span_end)
        .max()
        .unwrap_or(output_byte_len);

    (
        source_span_start,
        source_span_end,
        generated_span_start,
        generated_span_end,
    )
}

fn derive_transform_mutation_spans(
    input: &str,
    output: &str,
) -> Vec<TransformProvenanceMutationSpanV0> {
    if input == output {
        return Vec::new();
    }

    let input_line_spans = line_spans(input);
    let output_line_spans = line_spans(output);
    if input_line_spans.len() == output_line_spans.len() {
        let spans = input_line_spans
            .iter()
            .zip(output_line_spans.iter())
            .filter_map(
                |(&(source_start, source_end), &(generated_start, generated_end))| {
                    derive_changed_slice_mutation_span(
                        input,
                        output,
                        source_start,
                        source_end,
                        generated_start,
                        generated_end,
                    )
                },
            )
            .collect::<Vec<_>>();
        if !spans.is_empty() {
            return spans;
        }
    }

    let prefix = common_prefix_byte_len(input.as_bytes(), output.as_bytes());
    let suffix = common_suffix_byte_len(input.as_bytes(), output.as_bytes(), prefix);
    vec![TransformProvenanceMutationSpanV0 {
        source_span_start: prefix,
        source_span_end: input.len().saturating_sub(suffix),
        generated_span_start: prefix,
        generated_span_end: output.len().saturating_sub(suffix),
    }]
}

fn derive_changed_slice_mutation_span(
    input: &str,
    output: &str,
    source_start: usize,
    source_end: usize,
    generated_start: usize,
    generated_end: usize,
) -> Option<TransformProvenanceMutationSpanV0> {
    let source_slice = &input[source_start..source_end];
    let generated_slice = &output[generated_start..generated_end];
    if source_slice == generated_slice {
        return None;
    }

    let prefix = common_prefix_byte_len(source_slice.as_bytes(), generated_slice.as_bytes());
    let suffix =
        common_suffix_byte_len(source_slice.as_bytes(), generated_slice.as_bytes(), prefix);

    Some(TransformProvenanceMutationSpanV0 {
        source_span_start: source_start + prefix,
        source_span_end: source_end.saturating_sub(suffix),
        generated_span_start: generated_start + prefix,
        generated_span_end: generated_end.saturating_sub(suffix),
    })
}

fn line_spans(source: &str) -> Vec<(usize, usize)> {
    if source.is_empty() {
        return vec![(0, 0)];
    }

    let mut spans = Vec::new();
    let mut start = 0usize;
    for (index, byte) in source.bytes().enumerate() {
        if byte == b'\n' {
            let end = index + 1;
            spans.push((start, end));
            start = end;
        }
    }

    if start < source.len() {
        spans.push((start, source.len()));
    }

    spans
}

fn common_prefix_byte_len(left: &[u8], right: &[u8]) -> usize {
    left.iter()
        .zip(right.iter())
        .take_while(|(left, right)| left == right)
        .count()
}

fn common_suffix_byte_len(left: &[u8], right: &[u8], prefix_len: usize) -> usize {
    let mut suffix_len = 0usize;
    while left.len() > prefix_len + suffix_len
        && right.len() > prefix_len + suffix_len
        && left[left.len() - suffix_len - 1] == right[right.len() - suffix_len - 1]
    {
        suffix_len += 1;
    }
    suffix_len
}

pub fn implemented_mutation_pass_ids() -> Vec<&'static str> {
    vec![
        TransformPassKind::WhitespaceStrip.id(),
        TransformPassKind::CommentStrip.id(),
        TransformPassKind::NumberCompression.id(),
        TransformPassKind::UnitNormalization.id(),
        TransformPassKind::ColorCompression.id(),
        TransformPassKind::UrlQuoteStrip.id(),
        TransformPassKind::StringQuoteNormalize.id(),
        TransformPassKind::SelectorIsWhereCompression.id(),
        TransformPassKind::ShorthandCombining.id(),
        TransformPassKind::RuleDeduplication.id(),
        TransformPassKind::RuleMerging.id(),
        TransformPassKind::SelectorMerging.id(),
        TransformPassKind::EmptyRuleRemoval.id(),
        TransformPassKind::VendorPrefixing.id(),
        TransformPassKind::LightDarkLowering.id(),
        TransformPassKind::ColorMixLowering.id(),
        TransformPassKind::OklchOklabLowering.id(),
        TransformPassKind::ColorFunctionLowering.id(),
        TransformPassKind::LogicalToPhysical.id(),
        TransformPassKind::NestingUnwrap.id(),
        TransformPassKind::ScopeFlatten.id(),
        TransformPassKind::LayerFlatten.id(),
        TransformPassKind::SupportsStaticEval.id(),
        TransformPassKind::MediaStaticEval.id(),
        TransformPassKind::DeadMediaBranchRemoval.id(),
        TransformPassKind::DeadSupportsBranchRemoval.id(),
        TransformPassKind::ImportInline.id(),
        TransformPassKind::ScssModuleEvaluate.id(),
        TransformPassKind::LessModuleEvaluate.id(),
        TransformPassKind::ValueResolution.id(),
        TransformPassKind::StaticVarSubstitution.id(),
        TransformPassKind::ResolveCssModulesComposes.id(),
        TransformPassKind::HashCssModuleClassNames.id(),
        TransformPassKind::TreeShakeClass.id(),
        TransformPassKind::TreeShakeKeyframes.id(),
        TransformPassKind::TreeShakeValue.id(),
        TransformPassKind::TreeShakeCustomProperty.id(),
        TransformPassKind::DesignTokenRouting.id(),
        TransformPassKind::CalcReduction.id(),
        TransformPassKind::PrintCss.id(),
    ]
}

fn registry_entry_for_contract(contract: TransformPassContractV0) -> TransformPassRegistryEntryV0 {
    TransformPassRegistryEntryV0 {
        module_family: module_family_for_pass(contract.kind),
        query_family: query_family_for_pass(contract.kind),
        execution_status: TransformPassExecutionStatus::RegistryAndPlannerReady,
        contract,
    }
}

fn module_family_for_pass(kind: TransformPassKind) -> &'static str {
    match kind.ordinal() {
        1..=7 => "commodity-token",
        8 | 25 => "egg-backed",
        9..=13 => "cascade-proven-structural",
        14..=24 => "target-lowering",
        26..=28 => "module-bundle",
        29..=32 => "css-modules-resolution",
        33..=39 => "semantic-reachability",
        40 => "emission",
        _ => "unknown",
    }
}

fn query_family_for_pass(kind: TransformPassKind) -> &'static str {
    match kind.layer() {
        TransformLayer::SemanticAware => "semantic-aware-transform-query",
        TransformLayer::Commodity => "commodity-transform-query",
        TransformLayer::Emission => "emission-transform-query",
        TransformLayer::SemanticReadOnly => "semantic-read-only-query",
    }
}

fn order_passes_by_dag(requested: &[TransformPassKind]) -> Vec<TransformPassKind> {
    let mut remaining = dedupe_requested_passes(requested);
    remaining.sort_by_key(|kind| (execution_rank(*kind), kind.ordinal()));

    let mut ordered = Vec::with_capacity(remaining.len());
    while !remaining.is_empty() {
        let next_index = remaining
            .iter()
            .position(|candidate| !has_incoming_edge_from_remaining(*candidate, &remaining))
            .unwrap_or_default();
        ordered.push(remaining.remove(next_index));
    }

    ordered
}

fn dedupe_requested_passes(requested: &[TransformPassKind]) -> Vec<TransformPassKind> {
    let mut unique = Vec::new();
    for pass in requested {
        if !unique.contains(pass) {
            unique.push(*pass);
        }
    }
    unique
}

fn has_incoming_edge_from_remaining(
    candidate: TransformPassKind,
    remaining: &[TransformPassKind],
) -> bool {
    default_transform_dag_edges().iter().any(|edge| {
        edge.to == candidate.id()
            && remaining
                .iter()
                .any(|other| other.id() == edge.from && *other != candidate)
    })
}

fn edge_applies(edge: &TransformDagEdgeV0, ordered_pass_ids: &[&'static str]) -> bool {
    ordered_pass_ids.contains(&edge.from) && ordered_pass_ids.contains(&edge.to)
}

fn edge_is_satisfied(edge: &TransformDagEdgeV0, ordered_pass_ids: &[&'static str]) -> bool {
    let from = position_of_pass_id(edge.from, ordered_pass_ids);
    let to = position_of_pass_id(edge.to, ordered_pass_ids);
    match (from, to) {
        (Some(from), Some(to)) => from < to,
        _ => false,
    }
}

fn position_of_pass_id(pass_id: &'static str, ordered_pass_ids: &[&'static str]) -> Option<usize> {
    ordered_pass_ids
        .iter()
        .position(|ordered_pass_id| *ordered_pass_id == pass_id)
}

fn pass_is_registered(pass: &TransformPassKind) -> bool {
    default_transform_pass_contracts()
        .iter()
        .any(|contract| contract.kind == *pass)
}

fn transform_pass_kind_from_id(pass_id: &str) -> Option<TransformPassKind> {
    all_transform_pass_kinds()
        .into_iter()
        .find(|kind| kind.id() == pass_id)
}

fn execution_rank(kind: TransformPassKind) -> u8 {
    match kind.ordinal() {
        26..=28 => 10,
        29..=39 => 20,
        14..=24 => 30,
        8..=13 | 25 => 40,
        1..=7 => 50,
        40 => 60,
        _ => 70,
    }
}

fn strip_css_comments(source: &str, dialect: StyleDialect) -> (String, usize) {
    strip_css_comments_with_lexer(source, dialect)
}

fn compress_css_numbers(source: &str, dialect: StyleDialect) -> (String, usize) {
    compress_css_numbers_with_lexer(source, dialect)
}

fn compress_css_colors(source: &str, dialect: StyleDialect) -> (String, usize) {
    compress_css_colors_with_lexer(source, dialect)
}

fn normalize_css_units(source: &str, dialect: StyleDialect) -> (String, usize) {
    normalize_css_units_with_lexer(source, dialect)
}

fn strip_css_url_quotes(source: &str, dialect: StyleDialect) -> (String, usize) {
    strip_css_url_quotes_with_lexer(source, dialect)
}

fn normalize_css_string_quotes(source: &str, dialect: StyleDialect) -> (String, usize) {
    normalize_css_string_quotes_with_lexer(source, dialect)
}

fn compress_css_is_where_selectors(source: &str, dialect: StyleDialect) -> (String, usize) {
    compress_css_is_where_selectors_with_lexer(source, dialect)
}

fn remove_empty_css_rules(source: &str, dialect: StyleDialect) -> (String, usize) {
    remove_empty_css_rules_with_lexer(source, dialect)
}

fn combine_css_box_shorthands(source: &str, dialect: StyleDialect) -> (String, usize) {
    combine_css_box_shorthands_with_lexer(source, dialect)
}

fn dedupe_exact_css_rules(source: &str, dialect: StyleDialect) -> (String, usize) {
    dedupe_exact_css_rules_with_lexer(source, dialect)
}

fn merge_adjacent_same_selector_css_rules(source: &str, dialect: StyleDialect) -> (String, usize) {
    merge_adjacent_same_selector_css_rules_with_lexer(source, dialect)
}

fn merge_adjacent_same_block_css_selectors(source: &str, dialect: StyleDialect) -> (String, usize) {
    merge_adjacent_same_block_css_selectors_with_lexer(source, dialect)
}

fn add_css_vendor_prefixes(source: &str, dialect: StyleDialect) -> (String, usize) {
    add_css_vendor_prefixes_with_lexer(source, dialect)
}

fn lower_css_light_dark(source: &str, dialect: StyleDialect) -> (String, usize) {
    lower_css_light_dark_with_lexer(source, dialect)
}

fn lower_css_color_mix(source: &str, dialect: StyleDialect) -> (String, usize) {
    lower_css_color_mix_with_lexer(source, dialect)
}

fn lower_css_oklab_oklch(source: &str, dialect: StyleDialect) -> (String, usize) {
    lower_css_oklab_oklch_with_lexer(source, dialect)
}

fn lower_css_color_function(source: &str, dialect: StyleDialect) -> (String, usize) {
    lower_css_color_function_with_lexer(source, dialect)
}

fn lower_css_logical_to_physical(source: &str, dialect: StyleDialect) -> (String, usize) {
    lower_css_logical_to_physical_with_lexer(source, dialect)
}

fn unwrap_css_nesting(source: &str, dialect: StyleDialect) -> (String, usize) {
    unwrap_css_nesting_with_lexer(source, dialect)
}

fn flatten_css_scopes(source: &str, dialect: StyleDialect) -> (String, usize) {
    flatten_css_scopes_with_lexer(source, dialect)
}

fn flatten_css_layers(source: &str, dialect: StyleDialect, closed_bundle: bool) -> (String, usize) {
    flatten_css_layers_with_lexer(source, dialect, closed_bundle)
}

fn evaluate_static_supports_rules(source: &str, dialect: StyleDialect) -> (String, usize) {
    evaluate_static_supports_rules_with_lexer(source, dialect)
}

fn evaluate_static_media_rules(source: &str, dialect: StyleDialect) -> (String, usize) {
    evaluate_static_media_rules_with_lexer(source, dialect, StaticMediaEvaluationOptions::default())
}

fn evaluate_dead_media_branch_rules(
    source: &str,
    dialect: StyleDialect,
    context: &TransformExecutionContextV0,
) -> (String, usize) {
    evaluate_static_media_rules_with_lexer(
        source,
        dialect,
        StaticMediaEvaluationOptions {
            drop_dark_mode_media_queries: context.drop_dark_mode_media_queries,
        },
    )
}

fn inline_css_imports(
    source: &str,
    dialect: StyleDialect,
    inlines: &[TransformImportInlineV0],
) -> (String, usize) {
    inline_css_imports_with_lexer(source, dialect, inlines)
}

fn resolve_static_css_modules_values(source: &str, dialect: StyleDialect) -> (String, usize) {
    resolve_static_css_modules_values_with_lexer(source, dialect)
}

fn resolve_css_module_composes(
    source: &str,
    dialect: StyleDialect,
    resolutions: &[TransformCssModuleComposesResolutionV0],
) -> (String, usize) {
    strip_resolved_css_module_composes_with_lexer(source, dialect, resolutions)
}

fn route_design_token_values(
    source: &str,
    dialect: StyleDialect,
    routes: &[TransformDesignTokenRouteV0],
) -> (String, usize) {
    route_design_token_values_with_lexer(source, dialect, routes)
}

fn tree_shake_css_class_rules_with_removals(
    source: &str,
    dialect: StyleDialect,
    reachable_class_names: &[String],
) -> (String, Vec<TransformSemanticRemovalCandidate>) {
    tree_shake_css_class_rules_with_lexer(source, dialect, reachable_class_names)
}

fn tree_shake_css_keyframes_with_removals(
    source: &str,
    dialect: StyleDialect,
    reachable_keyframe_names: &[String],
) -> (String, Vec<TransformSemanticRemovalCandidate>) {
    tree_shake_css_keyframes_with_lexer(source, dialect, reachable_keyframe_names)
}

fn tree_shake_css_modules_values_with_removals(
    source: &str,
    dialect: StyleDialect,
    reachable_value_names: &[String],
) -> (String, Vec<TransformSemanticRemovalCandidate>) {
    tree_shake_css_modules_values_with_lexer(source, dialect, reachable_value_names)
}

fn tree_shake_css_custom_properties_with_removals(
    source: &str,
    dialect: StyleDialect,
    reachable_custom_property_names: &[String],
) -> (String, Vec<TransformSemanticRemovalCandidate>) {
    tree_shake_css_custom_properties_with_lexer(source, dialect, reachable_custom_property_names)
}

fn rewrite_css_module_class_names(
    source: &str,
    dialect: StyleDialect,
    rewrites: &[TransformClassNameRewriteV0],
) -> (String, usize) {
    rewrite_css_module_class_names_with_lexer(source, dialect, rewrites)
}

fn substitute_static_css_custom_properties(source: &str, dialect: StyleDialect) -> (String, usize) {
    substitute_static_css_custom_properties_with_lexer(source, dialect)
}

pub fn summarize_static_css_custom_property_fixed_point_from_source(
    source: &str,
    dialect: StyleDialect,
) -> CustomPropertyLeastFixedPointSummaryV0 {
    let lexed = lex(source, dialect);
    let tokens = lexed.tokens();
    let env_rules = collect_top_level_ordinary_rule_slices(source, tokens);
    let env = collect_static_root_custom_property_env(tokens, &env_rules);
    summarize_custom_property_least_fixed_point(&env)
}

fn reduce_css_calc(source: &str, dialect: StyleDialect) -> (String, usize) {
    reduce_css_calc_with_lexer(source, dialect)
}

fn normalize_css_whitespace(source: &str, dialect: StyleDialect) -> (String, usize) {
    normalize_css_whitespace_with_lexer(source, dialect)
}

fn lower_css_light_dark_with_lexer(source: &str, dialect: StyleDialect) -> (String, usize) {
    let lexed = lex(source, dialect);
    let tokens = lexed.tokens();
    let rules = collect_declaration_ordinary_rule_slices(source, tokens);
    let mut replacements = Vec::new();
    let mut insertions = Vec::new();

    for rule in &rules {
        let Some((block_start_index, block_end_index)) =
            rule_block_token_indexes(tokens, rule.block_start, rule.block_end)
        else {
            continue;
        };
        let declarations =
            collect_simple_declarations_in_block(tokens, block_start_index, block_end_index);
        for declaration in declarations {
            if !is_light_dark_lowerable_property(&declaration.property) {
                continue;
            }
            let Some((light_value, dark_value)) = parse_light_dark_value(&declaration.value) else {
                continue;
            };
            replacements.push((
                declaration.start,
                declaration.end,
                format!("{}: {light_value};", declaration.property),
            ));
            insertions.push((
                rule.end,
                format!(
                    " @media (prefers-color-scheme: dark) {{ {} {{ {}: {dark_value}; }} }}",
                    rule.selector, declaration.property
                ),
            ));
        }
    }

    if replacements.is_empty() && insertions.is_empty() {
        return (source.to_string(), 0);
    }

    let mut output = String::with_capacity(source.len());
    let mut cursor = 0;
    let mut insertion_index = 0;
    for (start, end, replacement) in &replacements {
        while insertion_index < insertions.len() && insertions[insertion_index].0 <= *start {
            let (position, insertion) = &insertions[insertion_index];
            if *position > cursor {
                output.push_str(&source[cursor..*position]);
                cursor = *position;
            }
            output.push_str(insertion);
            insertion_index += 1;
        }
        if *start > cursor {
            output.push_str(&source[cursor..*start]);
        }
        output.push_str(replacement);
        cursor = *end;
    }
    while insertion_index < insertions.len() {
        let (position, insertion) = &insertions[insertion_index];
        if *position > cursor {
            output.push_str(&source[cursor..*position]);
            cursor = *position;
        }
        output.push_str(insertion);
        insertion_index += 1;
    }
    if cursor < source.len() {
        output.push_str(&source[cursor..]);
    }

    (output, replacements.len())
}

fn lower_css_color_mix_with_lexer(source: &str, dialect: StyleDialect) -> (String, usize) {
    let lexed = lex(source, dialect);
    let tokens = lexed.tokens();
    let mut replacements = Vec::new();
    let mut index = 0;

    while index < tokens.len() {
        if tokens[index].kind == SyntaxKind::LeftBrace
            && let Some(close_index) = matching_right_brace_index(tokens, index)
        {
            let declarations = collect_simple_declarations_in_block(tokens, index, close_index);
            for declaration in declarations {
                if !is_light_dark_lowerable_property(&declaration.property) {
                    continue;
                }
                let Some(replacement_value) = substitute_static_css_function_references_in_value(
                    &declaration.value,
                    &[("color-mix", parse_color_mix_value)],
                ) else {
                    continue;
                };
                replacements.push((
                    declaration.start,
                    declaration.end,
                    format!("{}: {replacement_value};", declaration.property),
                ));
            }
            index = close_index + 1;
            continue;
        }
        index += 1;
    }

    if replacements.is_empty() {
        return (source.to_string(), 0);
    }

    let mut output = String::with_capacity(source.len());
    let mut cursor = 0;
    for (start, end, replacement) in &replacements {
        if *start > cursor {
            output.push_str(&source[cursor..*start]);
        }
        output.push_str(replacement);
        cursor = *end;
    }
    if cursor < source.len() {
        output.push_str(&source[cursor..]);
    }

    (output, replacements.len())
}

fn lower_css_oklab_oklch_with_lexer(source: &str, dialect: StyleDialect) -> (String, usize) {
    let lexed = lex(source, dialect);
    let tokens = lexed.tokens();
    let mut replacements = Vec::new();
    let mut index = 0;

    while index < tokens.len() {
        if tokens[index].kind == SyntaxKind::LeftBrace
            && let Some(close_index) = matching_right_brace_index(tokens, index)
        {
            let declarations = collect_simple_declarations_in_block(tokens, index, close_index);
            for declaration in declarations {
                if !is_light_dark_lowerable_property(&declaration.property) {
                    continue;
                }
                let Some(replacement_value) = substitute_static_css_function_references_in_value(
                    &declaration.value,
                    &[
                        ("oklab", parse_oklab_oklch_value),
                        ("oklch", parse_oklab_oklch_value),
                    ],
                ) else {
                    continue;
                };
                replacements.push((
                    declaration.start,
                    declaration.end,
                    format!("{}: {replacement_value};", declaration.property),
                ));
            }
            index = close_index + 1;
            continue;
        }
        index += 1;
    }

    if replacements.is_empty() {
        return (source.to_string(), 0);
    }

    let mut output = String::with_capacity(source.len());
    let mut cursor = 0;
    for (start, end, replacement) in &replacements {
        if *start > cursor {
            output.push_str(&source[cursor..*start]);
        }
        output.push_str(replacement);
        cursor = *end;
    }
    if cursor < source.len() {
        output.push_str(&source[cursor..]);
    }

    (output, replacements.len())
}

fn lower_css_color_function_with_lexer(source: &str, dialect: StyleDialect) -> (String, usize) {
    let lexed = lex(source, dialect);
    let tokens = lexed.tokens();
    let mut replacements = Vec::new();
    let mut index = 0;

    while index < tokens.len() {
        if tokens[index].kind == SyntaxKind::LeftBrace
            && let Some(close_index) = matching_right_brace_index(tokens, index)
        {
            let declarations = collect_simple_declarations_in_block(tokens, index, close_index);
            for declaration in declarations {
                if !is_light_dark_lowerable_property(&declaration.property) {
                    continue;
                }
                let Some(replacement_value) = substitute_static_css_function_references_in_value(
                    &declaration.value,
                    &[("color", parse_color_function_value)],
                ) else {
                    continue;
                };
                replacements.push((
                    declaration.start,
                    declaration.end,
                    format!("{}: {replacement_value};", declaration.property),
                ));
            }
            index = close_index + 1;
            continue;
        }
        index += 1;
    }

    if replacements.is_empty() {
        return (source.to_string(), 0);
    }

    let mut output = String::with_capacity(source.len());
    let mut cursor = 0;
    for (start, end, replacement) in &replacements {
        if *start > cursor {
            output.push_str(&source[cursor..*start]);
        }
        output.push_str(replacement);
        cursor = *end;
    }
    if cursor < source.len() {
        output.push_str(&source[cursor..]);
    }

    (output, replacements.len())
}

fn lower_css_logical_to_physical_with_lexer(
    source: &str,
    dialect: StyleDialect,
) -> (String, usize) {
    let lexed = lex(source, dialect);
    let tokens = lexed.tokens();
    let mut replacements = Vec::new();
    let mut index = 0;

    while index < tokens.len() {
        if tokens[index].kind == SyntaxKind::LeftBrace
            && let Some(close_index) = matching_right_brace_index(tokens, index)
        {
            let declarations = collect_simple_declarations_in_block(tokens, index, close_index);
            let Some(direction) = static_horizontal_direction_for_declarations(&declarations)
            else {
                index = close_index + 1;
                continue;
            };
            for declaration in declarations {
                let Some(physical_declaration) = physical_declaration_for_logical_declaration(
                    &declaration.property,
                    &declaration.value,
                    direction,
                ) else {
                    continue;
                };
                replacements.push((declaration.start, declaration.end, physical_declaration));
            }
            index = close_index + 1;
            continue;
        }
        index += 1;
    }

    if replacements.is_empty() {
        return (source.to_string(), 0);
    }

    let mut output = String::with_capacity(source.len());
    let mut cursor = 0;
    for (start, end, replacement) in &replacements {
        if *start > cursor {
            output.push_str(&source[cursor..*start]);
        }
        output.push_str(replacement);
        cursor = *end;
    }
    if cursor < source.len() {
        output.push_str(&source[cursor..]);
    }

    (output, replacements.len())
}

fn unwrap_css_nesting_with_lexer(source: &str, dialect: StyleDialect) -> (String, usize) {
    let lexed = lex(source, dialect);
    let tokens = lexed.tokens();
    let mut replacements = Vec::new();
    let mut depth = 0usize;
    let mut top_level_prelude_start = 0usize;
    let mut index = 0;

    while index < tokens.len() {
        match tokens[index].kind {
            SyntaxKind::LeftBrace => {
                if depth == 0
                    && let Some(close_index) = matching_right_brace_index(tokens, index)
                    && is_ordinary_top_level_rule_prelude(tokens, top_level_prelude_start, index)
                    && let Some(start) =
                        first_non_trivia_token_start(tokens, top_level_prelude_start, index)
                    && let Some(replacement) =
                        unwrap_simple_nested_rule(source, tokens, start, index, close_index)
                {
                    replacements.push((start, token_end(&tokens[close_index]), replacement));
                    index = close_index + 1;
                    top_level_prelude_start = index;
                    continue;
                }
                depth += 1;
            }
            SyntaxKind::RightBrace => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    top_level_prelude_start = index + 1;
                }
            }
            SyntaxKind::Semicolon if depth == 0 => {
                top_level_prelude_start = index + 1;
            }
            _ => {}
        }
        index += 1;
    }

    if replacements.is_empty() {
        return (source.to_string(), 0);
    }

    let mut output = String::with_capacity(source.len());
    let mut cursor = 0;
    for (start, end, replacement) in &replacements {
        if *start > cursor {
            output.push_str(&source[cursor..*start]);
        }
        output.push_str(replacement);
        cursor = *end;
    }
    if cursor < source.len() {
        output.push_str(&source[cursor..]);
    }

    (output, replacements.len())
}

fn unwrap_simple_nested_rule(
    source: &str,
    tokens: &[omena_parser::LexedToken],
    rule_start: usize,
    block_start_index: usize,
    block_end_index: usize,
) -> Option<String> {
    if tokens[block_start_index + 1..block_end_index]
        .iter()
        .any(|token| is_comment_token(token.kind))
    {
        return None;
    }

    let parent_selector = source[rule_start..token_start(&tokens[block_start_index])]
        .trim()
        .to_string();
    if parent_selector.is_empty() || split_css_selector_list(&parent_selector).is_none() {
        return None;
    }

    let rule_texts = unwrap_nested_rule_body(
        source,
        tokens,
        &parent_selector,
        block_start_index,
        block_end_index,
        true,
    )?;
    Some(rule_texts.join(" "))
}

fn unwrap_nested_rule_body(
    source: &str,
    tokens: &[omena_parser::LexedToken],
    parent_selector: &str,
    block_start_index: usize,
    block_end_index: usize,
    require_nested_rule: bool,
) -> Option<Vec<String>> {
    let declarations =
        collect_simple_declarations_in_block(tokens, block_start_index, block_end_index);
    let nested_rules =
        collect_direct_nested_rule_slices(source, tokens, block_start_index, block_end_index)?;
    if require_nested_rule && nested_rules.is_empty() {
        return None;
    }

    let mut rule_texts = Vec::new();
    if !declarations.is_empty() {
        let declarations_text = declarations
            .iter()
            .map(|declaration| format!("{}: {};", declaration.property, declaration.value))
            .collect::<Vec<_>>()
            .join(" ");
        rule_texts.push(format!("{parent_selector} {{ {declarations_text} }}"));
    }

    for nested_rule in nested_rules {
        match nested_rule.kind {
            NestedRuleKind::Style => {
                let selector = expand_nested_selector(parent_selector, &nested_rule.selector)?;
                let nested_rule_texts = unwrap_nested_rule_body(
                    source,
                    tokens,
                    &selector,
                    nested_rule.block_start_index,
                    nested_rule.block_end_index,
                    false,
                )?;
                rule_texts.extend(nested_rule_texts);
            }
            NestedRuleKind::ConditionalGroup => {
                let nested_rule_texts = unwrap_nested_rule_body(
                    source,
                    tokens,
                    parent_selector,
                    nested_rule.block_start_index,
                    nested_rule.block_end_index,
                    false,
                )?;
                rule_texts.push(format!(
                    "{} {{ {} }}",
                    nested_rule.selector,
                    nested_rule_texts.join(" ")
                ));
            }
        }
    }

    if rule_texts.is_empty() {
        None
    } else {
        Some(rule_texts)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum NestedRuleKind {
    Style,
    ConditionalGroup,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct NestedRuleSlice {
    selector: String,
    block_start_index: usize,
    block_end_index: usize,
    kind: NestedRuleKind,
}

fn collect_direct_nested_rule_slices(
    source: &str,
    tokens: &[omena_parser::LexedToken],
    block_start_index: usize,
    block_end_index: usize,
) -> Option<Vec<NestedRuleSlice>> {
    let mut nested_rules = Vec::new();
    let mut segment_start_index = block_start_index + 1;
    let mut index = block_start_index + 1;

    while index < block_end_index {
        if tokens[index].kind == SyntaxKind::LeftBrace {
            let nested_close_index = matching_right_brace_index(tokens, index)?;
            if nested_close_index > block_end_index {
                return None;
            }
            let selector_start = first_non_trivia_token_start(tokens, segment_start_index, index)?;
            let selector = source[selector_start..token_start(&tokens[index])]
                .trim()
                .to_string();
            if selector.is_empty() {
                return None;
            }
            let kind = if selector.starts_with('@') {
                if !is_supported_nested_conditional_group_rule(&selector) {
                    return None;
                }
                NestedRuleKind::ConditionalGroup
            } else {
                split_css_selector_list(&selector)?;
                NestedRuleKind::Style
            };
            if source[token_end(&tokens[index])..token_start(&tokens[nested_close_index])]
                .trim()
                .is_empty()
            {
                return None;
            }
            nested_rules.push(NestedRuleSlice {
                selector,
                block_start_index: index,
                block_end_index: nested_close_index,
                kind,
            });
            index = nested_close_index + 1;
            segment_start_index = index;
            continue;
        }
        if tokens[index].kind == SyntaxKind::Semicolon {
            segment_start_index = index + 1;
        }
        index += 1;
    }

    Some(nested_rules)
}

fn is_supported_nested_conditional_group_rule(selector: &str) -> bool {
    let selector = selector.trim_start().to_ascii_lowercase();
    ["@media", "@supports", "@container", "@layer"]
        .iter()
        .any(|prefix| selector.starts_with(prefix))
}

fn expand_nested_selector(parent_selector: &str, nested_selector: &str) -> Option<String> {
    let parent_selectors = split_css_selector_list(parent_selector)?;
    let nested_selectors = split_css_selector_list(nested_selector)?;
    let mut expanded_selectors = Vec::new();

    for parent in &parent_selectors {
        for nested in &nested_selectors {
            if nested.contains('&') {
                expanded_selectors.push(nested.replace('&', parent));
            } else {
                expanded_selectors.push(format!("{parent} {nested}"));
            }
        }
    }

    if expanded_selectors.is_empty() {
        None
    } else {
        Some(expanded_selectors.join(", "))
    }
}

fn split_css_selector_list(selector: &str) -> Option<Vec<String>> {
    let mut selectors = Vec::new();
    let mut segment_start = 0usize;
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;
    let mut quote = None::<char>;
    let mut escaped = false;

    for (index, character) in selector.char_indices() {
        if let Some(active_quote) = quote {
            if escaped {
                escaped = false;
                continue;
            }
            if character == '\\' {
                escaped = true;
                continue;
            }
            if character == active_quote {
                quote = None;
            }
            continue;
        }

        match character {
            '\'' | '"' => quote = Some(character),
            '(' => paren_depth += 1,
            ')' => paren_depth = paren_depth.checked_sub(1)?,
            '[' => bracket_depth += 1,
            ']' => bracket_depth = bracket_depth.checked_sub(1)?,
            ',' if paren_depth == 0 && bracket_depth == 0 => {
                let selector = selector[segment_start..index].trim();
                if selector.is_empty() {
                    return None;
                }
                selectors.push(selector.to_string());
                segment_start = index + character.len_utf8();
            }
            _ => {}
        }
    }

    if quote.is_some() || paren_depth != 0 || bracket_depth != 0 {
        return None;
    }

    let selector = selector[segment_start..].trim();
    if selector.is_empty() {
        return None;
    }
    selectors.push(selector.to_string());
    Some(selectors)
}

fn flatten_css_scopes_with_lexer(source: &str, dialect: StyleDialect) -> (String, usize) {
    let lexed = lex(source, dialect);
    let tokens = lexed.tokens();
    let top_level_scope_count = count_top_level_at_rules(tokens, "@scope");
    let competing_unscoped_rule_count =
        collect_top_level_ordinary_rule_slices(source, tokens).len();
    let mut replacements = Vec::new();
    let mut depth = 0usize;
    let mut index = 0;

    while index < tokens.len() {
        match tokens[index].kind {
            SyntaxKind::AtKeyword
                if depth == 0 && tokens[index].text.eq_ignore_ascii_case("@scope") =>
            {
                let Some((block_start_index, block_end_index)) =
                    at_rule_block_indexes(tokens, index)
                else {
                    index += 1;
                    continue;
                };
                let prelude = source
                    [token_end(&tokens[index])..token_start(&tokens[block_start_index])]
                    .trim();
                let Some((root_selector, limit_selector)) = parse_scope_flatten_prelude(prelude)
                else {
                    index = block_end_index + 1;
                    continue;
                };
                let scoped_rule_count = count_direct_ordinary_rules_in_block(
                    tokens,
                    block_start_index,
                    block_end_index,
                );
                let proof = prove_scope_flatten_candidate(ScopeFlattenInputV0 {
                    root_selector,
                    limit_selector,
                    scoped_rule_count,
                    peer_scope_count: top_level_scope_count.saturating_sub(1),
                    competing_unscoped_rule_count,
                    inside_layer: false,
                });
                if proof.accepted {
                    let replacement = source[token_end(&tokens[block_start_index])
                        ..token_start(&tokens[block_end_index])]
                        .trim()
                        .to_string();
                    replacements.push((
                        token_start(&tokens[index]),
                        token_end(&tokens[block_end_index]),
                        replacement,
                    ));
                }
                index = block_end_index + 1;
                continue;
            }
            SyntaxKind::LeftBrace => depth += 1,
            SyntaxKind::RightBrace => depth = depth.saturating_sub(1),
            _ => {}
        }
        index += 1;
    }

    replace_source_ranges(source, &replacements)
}

fn flatten_css_layers_with_lexer(
    source: &str,
    dialect: StyleDialect,
    closed_bundle: bool,
) -> (String, usize) {
    let lexed = lex(source, dialect);
    let tokens = lexed.tokens();
    let top_level_layer_count = count_top_level_at_rules(tokens, "@layer");
    let unlayered_rule_count = collect_top_level_ordinary_rule_slices(source, tokens).len();
    let mut replacements = Vec::new();
    let mut depth = 0usize;
    let mut index = 0;

    while index < tokens.len() {
        match tokens[index].kind {
            SyntaxKind::AtKeyword
                if depth == 0 && tokens[index].text.eq_ignore_ascii_case("@layer") =>
            {
                let Some((block_start_index, block_end_index)) =
                    at_rule_block_indexes(tokens, index)
                else {
                    index += 1;
                    continue;
                };
                let prelude = source
                    [token_end(&tokens[index])..token_start(&tokens[block_start_index])]
                    .trim();
                let layer_name = parse_single_layer_name(prelude);
                let important_declaration_count = tokens[block_start_index + 1..block_end_index]
                    .iter()
                    .filter(|token| token.kind == SyntaxKind::Important)
                    .count();
                let proof = prove_layer_flatten_candidate(LayerFlattenInputV0 {
                    layer_name,
                    layer_rule_count: count_direct_ordinary_rules_in_block(
                        tokens,
                        block_start_index,
                        block_end_index,
                    ),
                    peer_layer_count: top_level_layer_count.saturating_sub(1),
                    unlayered_rule_count,
                    important_declaration_count,
                    closed_bundle,
                });
                if proof.accepted {
                    let replacement = source[token_end(&tokens[block_start_index])
                        ..token_start(&tokens[block_end_index])]
                        .trim()
                        .to_string();
                    replacements.push((
                        token_start(&tokens[index]),
                        token_end(&tokens[block_end_index]),
                        replacement,
                    ));
                }
                index = block_end_index + 1;
                continue;
            }
            SyntaxKind::LeftBrace => depth += 1,
            SyntaxKind::RightBrace => depth = depth.saturating_sub(1),
            _ => {}
        }
        index += 1;
    }

    replace_source_ranges(source, &replacements)
}

fn count_top_level_at_rules(tokens: &[omena_parser::LexedToken], at_rule: &str) -> usize {
    let mut count = 0;
    let mut depth = 0usize;
    for token in tokens {
        match token.kind {
            SyntaxKind::AtKeyword if depth == 0 && token.text.eq_ignore_ascii_case(at_rule) => {
                count += 1;
            }
            SyntaxKind::LeftBrace => depth += 1,
            SyntaxKind::RightBrace => depth = depth.saturating_sub(1),
            _ => {}
        }
    }
    count
}

fn count_direct_ordinary_rules_in_block(
    tokens: &[omena_parser::LexedToken],
    block_start_index: usize,
    block_end_index: usize,
) -> usize {
    let mut count = 0;
    let mut depth = 0usize;
    let mut index = block_start_index + 1;
    while index < block_end_index {
        match tokens[index].kind {
            SyntaxKind::LeftBrace => {
                if depth == 0
                    && is_ordinary_top_level_rule_prelude(tokens, block_start_index + 1, index)
                {
                    count += 1;
                }
                depth += 1;
            }
            SyntaxKind::RightBrace => depth = depth.saturating_sub(1),
            _ => {}
        }
        index += 1;
    }
    count
}

fn parse_scope_flatten_prelude(prelude: &str) -> Option<(String, Option<String>)> {
    let prelude = prelude.trim();
    let (root, limit) = match prelude.split_once(" to ") {
        Some((root, limit)) => (root, Some(limit)),
        None => (prelude, None),
    };
    let root = strip_wrapping_parentheses(root.trim())?.trim().to_string();
    let limit = match limit {
        Some(limit) => Some(strip_wrapping_parentheses(limit.trim())?.trim().to_string()),
        None => None,
    };
    Some((root, limit))
}

fn strip_wrapping_parentheses(text: &str) -> Option<&str> {
    let text = text.trim();
    text.strip_prefix('(')
        .and_then(|value| value.strip_suffix(')'))
        .or(Some(text))
}

fn parse_single_layer_name(prelude: &str) -> Option<String> {
    let prelude = prelude.trim();
    if prelude.is_empty() || prelude.contains(',') || !css_identifier_text_is_plain(prelude) {
        return None;
    }
    Some(prelude.to_string())
}

fn evaluate_static_supports_rules_with_lexer(
    source: &str,
    dialect: StyleDialect,
) -> (String, usize) {
    let mut output = source.to_string();
    let mut mutation_count = 0;

    loop {
        let (next_output, next_mutation_count) =
            evaluate_static_supports_rules_once_with_lexer(&output, dialect);
        if next_mutation_count == 0 {
            return (output, mutation_count);
        }
        output = next_output;
        mutation_count += next_mutation_count;
    }
}

fn evaluate_static_supports_rules_once_with_lexer(
    source: &str,
    dialect: StyleDialect,
) -> (String, usize) {
    let lexed = lex(source, dialect);
    let tokens = lexed.tokens();
    let mut replacements = Vec::new();
    let mut index = 0;

    while index < tokens.len() {
        match tokens[index].kind {
            SyntaxKind::AtKeyword if tokens[index].text.eq_ignore_ascii_case("@supports") => {
                let Some((block_start_index, block_end_index)) =
                    at_rule_block_indexes(tokens, index)
                else {
                    index += 1;
                    continue;
                };
                let condition = source
                    [token_end(&tokens[index])..token_start(&tokens[block_start_index])]
                    .trim();
                let witness = evaluate_static_supports_condition(
                    condition,
                    StaticSupportsAssumptionV0::ModernBrowser,
                );
                let replacement = match witness.verdict {
                    StaticSupportsEvalVerdictV0::AlwaysTrue => {
                        source[token_end(&tokens[block_start_index])
                            ..token_start(&tokens[block_end_index])]
                            .trim()
                            .to_string()
                    }
                    StaticSupportsEvalVerdictV0::AlwaysFalse => String::new(),
                    StaticSupportsEvalVerdictV0::Unknown => {
                        index += 1;
                        continue;
                    }
                };
                replacements.push((
                    token_start(&tokens[index]),
                    token_end(&tokens[block_end_index]),
                    replacement,
                ));
                index = block_end_index + 1;
                continue;
            }
            _ => {}
        }
        index += 1;
    }

    if replacements.is_empty() {
        return (source.to_string(), 0);
    }

    let mut output = String::with_capacity(source.len());
    let mut cursor = 0;
    for (start, end, replacement) in &replacements {
        if *start > cursor {
            output.push_str(&source[cursor..*start]);
        }
        output.push_str(replacement);
        cursor = *end;
    }
    if cursor < source.len() {
        output.push_str(&source[cursor..]);
    }

    (output, replacements.len())
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct StaticMediaEvaluationOptions {
    drop_dark_mode_media_queries: bool,
}

fn evaluate_static_media_rules_with_lexer(
    source: &str,
    dialect: StyleDialect,
    options: StaticMediaEvaluationOptions,
) -> (String, usize) {
    let mut output = source.to_string();
    let mut mutation_count = 0;

    loop {
        let (next_output, next_mutation_count) =
            evaluate_static_media_rules_once_with_lexer(&output, dialect, options);
        if next_mutation_count == 0 {
            return (output, mutation_count);
        }
        output = next_output;
        mutation_count += next_mutation_count;
    }
}

fn evaluate_static_media_rules_once_with_lexer(
    source: &str,
    dialect: StyleDialect,
    options: StaticMediaEvaluationOptions,
) -> (String, usize) {
    let lexed = lex(source, dialect);
    let tokens = lexed.tokens();
    let mut replacements = Vec::new();
    let mut index = 0;

    while index < tokens.len() {
        match tokens[index].kind {
            SyntaxKind::AtKeyword if tokens[index].text.eq_ignore_ascii_case("@media") => {
                let Some((block_start_index, block_end_index)) =
                    at_rule_block_indexes(tokens, index)
                else {
                    index += 1;
                    continue;
                };
                let condition = normalize_ascii_whitespace(
                    source[token_end(&tokens[index])..token_start(&tokens[block_start_index])]
                        .trim(),
                )
                .to_ascii_lowercase();
                let replacement = match evaluate_static_media_condition(&condition, options) {
                    StaticMediaEvalVerdict::AlwaysTrue => {
                        source[token_end(&tokens[block_start_index])
                            ..token_start(&tokens[block_end_index])]
                            .trim()
                            .to_string()
                    }
                    StaticMediaEvalVerdict::AlwaysFalse => String::new(),
                    StaticMediaEvalVerdict::Unknown => {
                        index += 1;
                        continue;
                    }
                };
                replacements.push((
                    token_start(&tokens[index]),
                    token_end(&tokens[block_end_index]),
                    replacement,
                ));
                index = block_end_index + 1;
                continue;
            }
            _ => {}
        }
        index += 1;
    }

    if replacements.is_empty() {
        return (source.to_string(), 0);
    }

    let mut output = String::with_capacity(source.len());
    let mut cursor = 0;
    for (start, end, replacement) in &replacements {
        if *start > cursor {
            output.push_str(&source[cursor..*start]);
        }
        output.push_str(replacement);
        cursor = *end;
    }
    if cursor < source.len() {
        output.push_str(&source[cursor..]);
    }

    (output, replacements.len())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StaticMediaEvalVerdict {
    AlwaysTrue,
    AlwaysFalse,
    Unknown,
}

fn evaluate_static_media_condition(
    condition: &str,
    options: StaticMediaEvaluationOptions,
) -> StaticMediaEvalVerdict {
    match condition {
        "all" => StaticMediaEvalVerdict::AlwaysTrue,
        "not all" => StaticMediaEvalVerdict::AlwaysFalse,
        "(max-width: 0px)" | "screen and (max-width: 0px)" | "all and (max-width: 0px)" => {
            StaticMediaEvalVerdict::AlwaysFalse
        }
        "(prefers-color-scheme: dark)"
        | "screen and (prefers-color-scheme: dark)"
        | "all and (prefers-color-scheme: dark)"
            if options.drop_dark_mode_media_queries =>
        {
            StaticMediaEvalVerdict::AlwaysFalse
        }
        _ => StaticMediaEvalVerdict::Unknown,
    }
}

fn at_rule_block_indexes(
    tokens: &[omena_parser::LexedToken],
    at_keyword_index: usize,
) -> Option<(usize, usize)> {
    let mut index = at_keyword_index + 1;
    while index < tokens.len() {
        match tokens[index].kind {
            SyntaxKind::Semicolon => return None,
            SyntaxKind::LeftBrace => {
                return matching_right_brace_index(tokens, index).map(|end| (index, end));
            }
            _ => index += 1,
        }
    }
    None
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct StaticCssModulesValueDefinition {
    name: String,
    value: String,
    start: usize,
    end: usize,
}

fn resolve_static_css_modules_values_with_lexer(
    source: &str,
    dialect: StyleDialect,
) -> (String, usize) {
    let lexed = lex(source, dialect);
    let tokens = lexed.tokens();
    let definitions = collect_static_local_css_modules_value_definitions(tokens);
    let unique_definitions_by_name = definitions
        .iter()
        .filter(|definition| {
            definitions
                .iter()
                .filter(|candidate| candidate.name == definition.name)
                .count()
                == 1
        })
        .map(|definition| (definition.name.clone(), definition))
        .collect::<BTreeMap<_, _>>();
    let resolved_definitions = unique_definitions_by_name
        .keys()
        .filter_map(|name| {
            let definition = unique_definitions_by_name.get(name)?;
            let resolved_value = resolve_static_css_modules_value_definition(
                name,
                &unique_definitions_by_name,
                &mut Vec::new(),
            )?;
            Some((*definition, resolved_value))
        })
        .collect::<Vec<_>>();
    if resolved_definitions.is_empty() {
        return (source.to_string(), 0);
    }

    let mut replacements = resolved_definitions
        .iter()
        .map(|(definition, _)| (definition.start, definition.end, String::new()))
        .collect::<Vec<_>>();
    let mut index = 0;
    while index < tokens.len() {
        if tokens[index].kind == SyntaxKind::LeftBrace
            && let Some(close_index) = matching_right_brace_index(tokens, index)
        {
            for declaration in collect_simple_declarations_in_block(tokens, index, close_index) {
                let Some((_, resolved_value)) = resolved_definitions
                    .iter()
                    .find(|(definition, _)| declaration.value == definition.name)
                else {
                    continue;
                };
                replacements.push((
                    declaration.start,
                    declaration.end,
                    format!("{}: {resolved_value};", declaration.property),
                ));
            }
            index = close_index + 1;
            continue;
        }
        index += 1;
    }

    replacements.sort_by_key(|(start, _, _)| *start);
    let mut output = String::with_capacity(source.len());
    let mut cursor = 0;
    let mut mutation_count = 0;
    for (start, end, replacement) in &replacements {
        if *start < cursor {
            continue;
        }
        if *start > cursor {
            output.push_str(&source[cursor..*start]);
        }
        output.push_str(replacement);
        cursor = *end;
        mutation_count += 1;
    }
    if cursor < source.len() {
        output.push_str(&source[cursor..]);
    }

    (output, mutation_count)
}

fn resolve_static_css_modules_value_definition(
    name: &str,
    definitions_by_name: &BTreeMap<String, &StaticCssModulesValueDefinition>,
    visiting: &mut Vec<String>,
) -> Option<String> {
    if visiting.iter().any(|candidate| candidate == name) {
        return None;
    }
    let definition = definitions_by_name.get(name)?;
    if is_static_css_modules_value_literal(&definition.value) {
        return Some(definition.value.clone());
    }
    let alias = definition.value.trim();
    if !css_identifier_text_is_plain(alias) || !definitions_by_name.contains_key(alias) {
        return None;
    }
    visiting.push(name.to_string());
    let resolved =
        resolve_static_css_modules_value_definition(alias, definitions_by_name, visiting);
    visiting.pop();
    resolved
}

fn collect_static_local_css_modules_value_definitions(
    tokens: &[omena_parser::LexedToken],
) -> Vec<StaticCssModulesValueDefinition> {
    let mut definitions = Vec::new();
    let mut depth = 0usize;
    let mut index = 0;

    while index < tokens.len() {
        match tokens[index].kind {
            SyntaxKind::AtKeyword
                if depth == 0 && tokens[index].text.eq_ignore_ascii_case("@value") =>
            {
                let Some((definition, next_index)) =
                    parse_static_local_css_modules_value_definition(tokens, index)
                else {
                    index += 1;
                    continue;
                };
                definitions.push(definition);
                index = next_index;
                continue;
            }
            SyntaxKind::LeftBrace => depth += 1,
            SyntaxKind::RightBrace => depth = depth.saturating_sub(1),
            _ => {}
        }
        index += 1;
    }

    definitions
}

fn parse_static_local_css_modules_value_definition(
    tokens: &[omena_parser::LexedToken],
    at_value_index: usize,
) -> Option<(StaticCssModulesValueDefinition, usize)> {
    let mut index = skip_whitespace_tokens(tokens, at_value_index + 1, tokens.len());
    let name_token = tokens.get(index)?;
    if name_token.kind != SyntaxKind::Ident {
        return None;
    }
    let name = name_token.text.clone();

    index = skip_whitespace_tokens(tokens, index + 1, tokens.len());
    if tokens.get(index)?.kind != SyntaxKind::Colon {
        return None;
    }

    let value_start = index + 1;
    index += 1;
    while index < tokens.len() {
        match tokens[index].kind {
            SyntaxKind::Semicolon => {
                let value_tokens = tokens[value_start..index].iter().collect::<Vec<_>>();
                if value_tokens.is_empty()
                    || value_tokens.iter().any(|token| {
                        is_comment_token(token.kind) || token.kind == SyntaxKind::AtKeyword
                    })
                {
                    return None;
                }
                let value = value_tokens
                    .iter()
                    .map(|token| token.text.as_str())
                    .collect::<String>()
                    .trim()
                    .to_string();
                return Some((
                    StaticCssModulesValueDefinition {
                        name,
                        value,
                        start: token_start(&tokens[at_value_index]),
                        end: token_end(&tokens[index]),
                    },
                    index + 1,
                ));
            }
            SyntaxKind::LeftBrace | SyntaxKind::RightBrace => return None,
            _ => index += 1,
        }
    }

    None
}

fn is_static_css_modules_value_literal(value: &str) -> bool {
    parse_static_srgb_color(value).is_some()
        || parse_numeric_value_with_unit(value)
            .map(|numeric| {
                numeric.unit.is_empty() || css_modules_value_unit_is_static(numeric.unit)
            })
            .unwrap_or(false)
}

fn css_modules_value_unit_is_static(unit: &str) -> bool {
    matches!(
        unit.to_ascii_lowercase().as_str(),
        "%" | "ch"
            | "cm"
            | "deg"
            | "dppx"
            | "em"
            | "fr"
            | "in"
            | "ms"
            | "pc"
            | "pt"
            | "px"
            | "rem"
            | "s"
            | "turn"
            | "vh"
            | "vmax"
            | "vmin"
            | "vw"
    )
}

fn tree_shake_css_class_rules_with_lexer(
    source: &str,
    dialect: StyleDialect,
    reachable_class_names: &[String],
) -> (String, Vec<TransformSemanticRemovalCandidate>) {
    let lexed = lex(source, dialect);
    let tokens = lexed.tokens();
    let rules = collect_declaration_ordinary_rule_slices(source, tokens);
    let removals = rules
        .iter()
        .filter_map(|rule| {
            selector_list_unreachable_owner_class_names(&rule.selector, reachable_class_names).map(
                |owner_class_names| TransformSemanticRemovalCandidate {
                    symbol_kind: "class",
                    name: owner_class_names.join(","),
                    source_span_start: rule.start,
                    source_span_end: rule.end,
                    reason: "selector owner classes were absent from the closed-style-world reachable class set",
                },
            )
        })
        .collect::<Vec<_>>();
    let ranges = removals
        .iter()
        .map(|removal| (removal.source_span_start, removal.source_span_end))
        .collect::<Vec<_>>();

    let (output, _) = remove_source_ranges(source, &ranges);
    (output, removals)
}

fn selector_list_unreachable_owner_class_names(
    selector: &str,
    reachable_class_names: &[String],
) -> Option<Vec<String>> {
    let branches = split_top_level_value_arguments(selector)?;
    if branches.is_empty() {
        return None;
    }
    let mut owner_class_names = Vec::new();
    for branch in branches {
        let class_name = selector_branch_owner_class_name(&branch)?;
        if class_name_is_reachable(&class_name, reachable_class_names) {
            return None;
        }
        push_unique_string(&mut owner_class_names, class_name);
    }
    Some(owner_class_names)
}

fn selector_branch_owner_class_name(selector: &str) -> Option<String> {
    let selector = selector.trim();
    if selector.is_empty() || find_ascii_case_insensitive(selector, ":global").is_some() {
        return None;
    }

    let mut index = 0usize;
    let mut quote: Option<char> = None;
    let mut bracket_depth = 0usize;
    let mut paren_depth = 0usize;

    while index < selector.len() {
        let ch = selector[index..].chars().next()?;

        if let Some(quote_ch) = quote {
            index += ch.len_utf8();
            if ch == '\\' {
                let escaped = selector[index..].chars().next()?;
                index += escaped.len_utf8();
            } else if ch == quote_ch {
                quote = None;
            }
            continue;
        }

        match ch {
            '"' | '\'' => {
                quote = Some(ch);
                index += ch.len_utf8();
            }
            '[' => {
                bracket_depth += 1;
                index += ch.len_utf8();
            }
            ']' => {
                bracket_depth = bracket_depth.saturating_sub(1);
                index += ch.len_utf8();
            }
            '(' => {
                paren_depth += 1;
                index += ch.len_utf8();
            }
            ')' => {
                paren_depth = paren_depth.saturating_sub(1);
                index += ch.len_utf8();
            }
            '\\' => return None,
            '.' if bracket_depth == 0 && paren_depth == 0 => {
                let name_start = index + ch.len_utf8();
                let name_end = ascii_css_identifier_end(selector, name_start);
                if name_end == name_start {
                    return None;
                }
                return Some(selector[name_start..name_end].to_string());
            }
            _ => {
                index += ch.len_utf8();
            }
        }
    }

    None
}

fn simple_class_selector_name(selector: &str) -> Option<String> {
    let name = selector.trim().strip_prefix('.')?;
    if name.is_empty() || !css_identifier_text_is_plain(name) {
        return None;
    }
    Some(name.to_string())
}

fn class_name_is_reachable(class_name: &str, reachable_class_names: &[String]) -> bool {
    reachable_class_names
        .iter()
        .filter_map(|name| normalize_reachable_class_name(name))
        .any(|name| name == class_name)
}

fn normalize_reachable_class_name(name: &str) -> Option<&str> {
    let name = name.trim();
    let name = name.strip_prefix('.').unwrap_or(name);
    if name.is_empty() {
        return None;
    }
    Some(name)
}

fn inline_css_imports_with_lexer(
    source: &str,
    dialect: StyleDialect,
    inlines: &[TransformImportInlineV0],
) -> (String, usize) {
    let lexed = lex(source, dialect);
    let tokens = lexed.tokens();
    let mut replacements = Vec::new();
    let mut depth = 0usize;
    let mut index = 0;

    while index < tokens.len() {
        match tokens[index].kind {
            SyntaxKind::LeftBrace => depth += 1,
            SyntaxKind::RightBrace => depth = depth.saturating_sub(1),
            SyntaxKind::AtKeyword
                if depth == 0 && tokens[index].text.eq_ignore_ascii_case("@import") =>
            {
                let Some(end_index) = find_import_rule_semicolon(tokens, index) else {
                    index += 1;
                    continue;
                };
                let start = token_start(&tokens[index]);
                let end = token_end(&tokens[end_index]);
                let rule_text = &source[start..end];
                let Some(import_rule) = parse_css_import_rule(rule_text) else {
                    index = end_index + 1;
                    continue;
                };
                if let Some(replacement_css) =
                    inline_replacement_for_import_source(&import_rule.source, inlines)
                {
                    replacements.push((
                        start,
                        end,
                        wrap_import_replacement(&import_rule, replacement_css),
                    ));
                }
                index = end_index + 1;
                continue;
            }
            _ => {}
        }
        index += 1;
    }

    replace_source_ranges(source, &replacements)
}

fn find_import_rule_semicolon(
    tokens: &[omena_parser::LexedToken],
    at_import_index: usize,
) -> Option<usize> {
    let mut index = at_import_index + 1;
    while index < tokens.len() {
        match tokens[index].kind {
            SyntaxKind::Semicolon => return Some(index),
            SyntaxKind::LeftBrace | SyntaxKind::RightBrace => return None,
            _ => index += 1,
        }
    }
    None
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CssImportRule {
    source: String,
    layer_name: Option<String>,
    supports_condition: Option<String>,
    media_query: Option<String>,
}

fn parse_css_import_rule(rule_text: &str) -> Option<CssImportRule> {
    let rest = strip_ascii_prefix_ignore_case(rule_text.trim(), "@import")?;
    let rest = rest.trim().trim_end_matches(';').trim();
    if rest.is_empty() {
        return None;
    }
    let (source, rest) = parse_css_import_source_prefix(rest)?;
    let mut rest = rest.trim();
    let mut layer_name = None;
    let mut supports_condition = None;

    loop {
        if let Some((layer, next_rest)) = parse_layer_import_option(rest) {
            layer_name = Some(layer);
            rest = next_rest.trim();
            continue;
        }
        if let Some((supports, next_rest)) = parse_function_prefix(rest, "supports") {
            supports_condition = Some(format!("({})", supports.trim()));
            rest = next_rest.trim();
            continue;
        }
        break;
    }

    Some(CssImportRule {
        source,
        layer_name,
        supports_condition,
        media_query: (!rest.is_empty()).then(|| rest.to_string()),
    })
}

fn parse_css_import_source_prefix(text: &str) -> Option<(String, &str)> {
    parse_quoted_css_string_prefix(text).or_else(|| parse_url_import_source_prefix(text))
}

fn parse_quoted_css_string_prefix(text: &str) -> Option<(String, &str)> {
    let mut chars = text.char_indices();
    let (_, quote) = chars.next()?;
    if !matches!(quote, '"' | '\'') {
        return None;
    }
    let mut escaped = false;
    let mut output = String::new();
    for (index, ch) in chars {
        if escaped {
            output.push(ch);
            escaped = false;
            continue;
        }
        if ch == '\\' {
            escaped = true;
            continue;
        }
        if ch == quote {
            let end = index + ch.len_utf8();
            return Some((output, &text[end..]));
        }
        output.push(ch);
    }
    None
}

fn parse_url_import_source_prefix(text: &str) -> Option<(String, &str)> {
    let rest = strip_ascii_prefix_ignore_case(text, "url(")?;
    let close_index = matching_function_close_index(rest)?;
    let inner = rest[..close_index].trim();
    if let Some((source, trailing)) = parse_quoted_css_string_prefix(inner)
        && trailing.trim().is_empty()
    {
        return Some((source, &rest[close_index + 1..]));
    }
    if inner.is_empty()
        || inner
            .chars()
            .any(|ch| ch.is_ascii_whitespace() || matches!(ch, '"' | '\'' | '(' | ')'))
    {
        return None;
    }
    Some((inner.to_string(), &rest[close_index + 1..]))
}

fn parse_layer_import_option(text: &str) -> Option<(String, &str)> {
    if let Some((layer, rest)) = parse_function_prefix(text, "layer") {
        return Some((layer.trim().to_string(), rest));
    }
    let rest = strip_ascii_prefix_ignore_case(text, "layer")?;
    if !rest.is_empty() && !rest.starts_with(char::is_whitespace) {
        return None;
    }
    Some((String::new(), rest))
}

fn parse_function_prefix<'a>(text: &'a str, name: &str) -> Option<(String, &'a str)> {
    let rest = strip_ascii_prefix_ignore_case(text.trim_start(), name)?;
    let rest = rest.strip_prefix('(')?;
    let close_index = matching_function_close_index(rest)?;
    Some((rest[..close_index].to_string(), &rest[close_index + 1..]))
}

fn matching_function_close_index(text: &str) -> Option<usize> {
    let mut depth = 1usize;
    let mut quote = None::<char>;
    let mut escaped = false;

    for (index, ch) in text.char_indices() {
        if let Some(active_quote) = quote {
            if escaped {
                escaped = false;
                continue;
            }
            if ch == '\\' {
                escaped = true;
                continue;
            }
            if ch == active_quote {
                quote = None;
            }
            continue;
        }

        match ch {
            '"' | '\'' => quote = Some(ch),
            '(' => depth += 1,
            ')' => {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    return Some(index);
                }
            }
            _ => {}
        }
    }
    None
}

fn wrap_import_replacement(import_rule: &CssImportRule, replacement_css: &str) -> String {
    let mut output = replacement_css.to_string();
    if let Some(layer_name) = &import_rule.layer_name {
        output = if layer_name.is_empty() {
            format!("@layer {{ {output} }}")
        } else {
            format!("@layer {layer_name} {{ {output} }}")
        };
    }
    if let Some(supports_condition) = &import_rule.supports_condition {
        output = format!("@supports {supports_condition} {{ {output} }}");
    }
    if let Some(media_query) = &import_rule.media_query {
        output = format!("@media {media_query} {{ {output} }}");
    }
    output
}

fn strip_ascii_prefix_ignore_case<'a>(text: &'a str, prefix: &str) -> Option<&'a str> {
    text.get(..prefix.len())?
        .eq_ignore_ascii_case(prefix)
        .then(|| &text[prefix.len()..])
}

fn inline_replacement_for_import_source<'a>(
    import_source: &str,
    inlines: &'a [TransformImportInlineV0],
) -> Option<&'a str> {
    inlines
        .iter()
        .find(|inline| inline.import_source == import_source)
        .map(|inline| inline.replacement_css.as_str())
}

fn strip_resolved_css_module_composes_with_lexer(
    source: &str,
    dialect: StyleDialect,
    resolutions: &[TransformCssModuleComposesResolutionV0],
) -> (String, usize) {
    let lexed = lex(source, dialect);
    let tokens = lexed.tokens();
    let rules = collect_top_level_ordinary_rule_slices(source, tokens);
    let mut ranges = Vec::new();

    for rule in &rules {
        let Some(class_name) = single_simple_class_selector_name(&rule.selector) else {
            continue;
        };
        if !css_module_composes_resolution_exists(&class_name, resolutions) {
            continue;
        }
        let Some(block_start_index) = tokens.iter().position(|token| {
            token.kind == SyntaxKind::LeftBrace && token_start(token) == rule.block_start
        }) else {
            continue;
        };
        let Some(block_end_index) = matching_right_brace_index(tokens, block_start_index) else {
            continue;
        };
        for declaration in
            collect_simple_declarations_in_block(tokens, block_start_index, block_end_index)
        {
            if declaration.property == "composes" {
                ranges.push((declaration.start, declaration.end));
            }
        }
    }

    remove_source_ranges(source, &ranges)
}

fn single_simple_class_selector_name(selector: &str) -> Option<String> {
    let branches = split_top_level_value_arguments(selector)?;
    if branches.len() != 1 {
        return None;
    }
    simple_class_selector_name(&branches[0])
}

fn css_module_composes_resolution_exists(
    class_name: &str,
    resolutions: &[TransformCssModuleComposesResolutionV0],
) -> bool {
    resolutions.iter().any(|resolution| {
        !resolution.exported_class_names.is_empty()
            && normalize_reachable_class_name(&resolution.local_class_name)
                .is_some_and(|resolved_name| resolved_name == class_name)
            && resolution
                .exported_class_names
                .iter()
                .all(|name| normalize_reachable_class_name(name).is_some())
    })
}

fn route_design_token_values_with_lexer(
    source: &str,
    dialect: StyleDialect,
    routes: &[TransformDesignTokenRouteV0],
) -> (String, usize) {
    let lexed = lex(source, dialect);
    let tokens = lexed.tokens();
    let rules = collect_declaration_ordinary_rule_slices(source, tokens);
    let mut replacements = Vec::new();

    for rule in &rules {
        let Some((block_start_index, block_end_index)) =
            rule_block_token_indexes(tokens, rule.block_start, rule.block_end)
        else {
            continue;
        };
        for declaration in
            collect_simple_declarations_in_block(tokens, block_start_index, block_end_index)
        {
            if declaration.property.starts_with("--") || declaration.important {
                continue;
            }
            let Some(routed_value) =
                route_design_token_references_in_value(&declaration.value, routes)
            else {
                continue;
            };
            replacements.push((
                declaration.start,
                declaration.end,
                format!("{}: {routed_value};", declaration.property),
            ));
        }
    }

    replace_source_ranges(source, &replacements)
}

fn route_design_token_references_in_value(
    value: &str,
    routes: &[TransformDesignTokenRouteV0],
) -> Option<String> {
    let mut output = String::with_capacity(value.len());
    let mut cursor = 0usize;
    let mut index = 0usize;
    let mut quote: Option<char> = None;
    let mut changed = false;

    while index < value.len() {
        let ch = value[index..].chars().next()?;

        if let Some(quote_ch) = quote {
            index += ch.len_utf8();
            if ch == '\\' {
                let escaped = value[index..].chars().next()?;
                index += escaped.len_utf8();
            } else if ch == quote_ch {
                quote = None;
            }
            continue;
        }

        match ch {
            '"' | '\'' => {
                quote = Some(ch);
                index += ch.len_utf8();
            }
            _ if value[index..]
                .get(.."var(".len())
                .is_some_and(|text| text.eq_ignore_ascii_case("var(")) =>
            {
                let left_paren_index = index + "var".len();
                let close_index = matching_function_call_end(value, left_paren_index)?;
                let arguments =
                    split_top_level_value_arguments(&value[left_paren_index + 1..close_index])?;
                if let Some(routed_value) =
                    routed_design_token_value_for_var_arguments(&arguments, routes)
                {
                    output.push_str(&value[cursor..index]);
                    output.push_str(&routed_value);
                    index = close_index + ')'.len_utf8();
                    cursor = index;
                    changed = true;
                } else {
                    index += ch.len_utf8();
                }
            }
            _ => {
                index += ch.len_utf8();
            }
        }
    }

    if !changed {
        return None;
    }
    output.push_str(&value[cursor..]);
    Some(output)
}

fn routed_design_token_value_for_var_arguments(
    arguments: &[String],
    routes: &[TransformDesignTokenRouteV0],
) -> Option<String> {
    let ([token_name] | [token_name, _]) = arguments else {
        return None;
    };
    let token_name = normalize_design_token_name(token_name)?;
    let routed_value = design_token_routed_value(token_name, routes)?;
    if let [_, fallback] = arguments
        && let Some(routed_token_name) = parse_single_custom_property_var_reference(routed_value)
    {
        return Some(format!("var({routed_token_name}, {fallback})"));
    }
    Some(routed_value.to_string())
}

fn parse_single_custom_property_var_reference(value: &str) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, "var")?;
    let [name] = arguments.as_slice() else {
        return None;
    };
    Some(normalize_design_token_name(name)?.to_string())
}

fn matching_function_call_end(value: &str, left_paren_index: usize) -> Option<usize> {
    if value[left_paren_index..].chars().next()? != '(' {
        return None;
    }

    let mut depth = 0usize;
    let mut index = left_paren_index;
    let mut quote: Option<char> = None;

    while index < value.len() {
        let ch = value[index..].chars().next()?;

        if let Some(quote_ch) = quote {
            index += ch.len_utf8();
            if ch == '\\' {
                let escaped = value[index..].chars().next()?;
                index += escaped.len_utf8();
            } else if ch == quote_ch {
                quote = None;
            }
            continue;
        }

        match ch {
            '"' | '\'' => {
                quote = Some(ch);
                index += ch.len_utf8();
            }
            '(' => {
                depth += 1;
                index += ch.len_utf8();
            }
            ')' => {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    return Some(index);
                }
                index += ch.len_utf8();
            }
            _ => {
                index += ch.len_utf8();
            }
        }
    }

    None
}

fn design_token_routed_value<'a>(
    token_name: &str,
    routes: &'a [TransformDesignTokenRouteV0],
) -> Option<&'a str> {
    routes.iter().find_map(|route| {
        let route_name = normalize_design_token_name(&route.token_name)?;
        let routed_value = route.routed_value.trim();
        if routed_value.is_empty() || routed_value.chars().any(|ch| matches!(ch, ';' | '{' | '}')) {
            return None;
        }
        (route_name == token_name).then_some(routed_value)
    })
}

fn normalize_design_token_name(name: &str) -> Option<&str> {
    let name = name.trim();
    if name.starts_with("--") && name.len() > 2 {
        return Some(name);
    }
    None
}

fn rewrite_css_module_class_names_with_lexer(
    source: &str,
    dialect: StyleDialect,
    rewrites: &[TransformClassNameRewriteV0],
) -> (String, usize) {
    let lexed = lex(source, dialect);
    let tokens = lexed.tokens();
    let rules = collect_top_level_ordinary_rule_slices(source, tokens);
    let mut replacements = Vec::new();

    for rule in &rules {
        let Some(rewritten_selector) =
            rewrite_class_selectors_in_selector(&rule.selector, rewrites)
        else {
            continue;
        };
        replacements.push((rule.start, rule.block_start, rewritten_selector));
    }

    let mut index = 0;
    while index < tokens.len() {
        if tokens[index].kind == SyntaxKind::LeftBrace
            && let Some(close_index) = matching_right_brace_index(tokens, index)
        {
            for declaration in collect_simple_declarations_in_block(tokens, index, close_index) {
                if declaration.property != "composes" {
                    continue;
                }
                let Some(rewritten_value) =
                    rewrite_local_composes_value(&declaration.value, rewrites)
                else {
                    continue;
                };
                replacements.push((
                    declaration.start,
                    declaration.end,
                    format!("composes: {rewritten_value};"),
                ));
            }
            index = close_index + 1;
            continue;
        }
        index += 1;
    }

    replace_source_ranges(source, &replacements)
}

fn rewrite_class_selectors_in_selector(
    selector: &str,
    rewrites: &[TransformClassNameRewriteV0],
) -> Option<String> {
    let mut output = String::with_capacity(selector.len());
    let mut index = 0usize;
    let mut changed = false;
    let mut quote: Option<char> = None;
    let mut bracket_depth = 0usize;

    while index < selector.len() {
        let ch = selector[index..].chars().next()?;

        if let Some(quote_ch) = quote {
            output.push(ch);
            index += ch.len_utf8();
            if ch == '\\' {
                if let Some(escaped) = selector[index..].chars().next() {
                    output.push(escaped);
                    index += escaped.len_utf8();
                }
            } else if ch == quote_ch {
                quote = None;
            }
            continue;
        }

        match ch {
            '"' | '\'' => {
                quote = Some(ch);
                output.push(ch);
                index += ch.len_utf8();
            }
            '[' => {
                bracket_depth += 1;
                output.push(ch);
                index += ch.len_utf8();
            }
            ']' => {
                bracket_depth = bracket_depth.saturating_sub(1);
                output.push(ch);
                index += ch.len_utf8();
            }
            '.' if bracket_depth == 0 => {
                let name_start = index + ch.len_utf8();
                let name_end = ascii_css_identifier_end(selector, name_start);
                if name_end == name_start {
                    output.push(ch);
                    index += ch.len_utf8();
                    continue;
                }
                let class_name = &selector[name_start..name_end];
                if let Some(rewritten_name) = rewritten_class_name_for(class_name, rewrites) {
                    output.push('.');
                    output.push_str(rewritten_name);
                    index = name_end;
                    changed = true;
                } else {
                    output.push_str(&selector[index..name_end]);
                    index = name_end;
                }
            }
            _ => {
                output.push(ch);
                index += ch.len_utf8();
            }
        }
    }

    changed.then_some(output)
}

fn ascii_css_identifier_end(text: &str, start: usize) -> usize {
    let bytes = text.as_bytes();
    let mut end = start;
    while end < bytes.len() && css_identifier_byte_is_plain(bytes[end]) {
        end += 1;
    }
    end
}

fn css_identifier_byte_is_plain(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-')
}

fn rewrite_local_composes_value(
    value: &str,
    rewrites: &[TransformClassNameRewriteV0],
) -> Option<String> {
    if value
        .split_whitespace()
        .any(|part| matches!(part, "from" | "global"))
        || value.contains(',')
    {
        return None;
    }
    let mut changed = false;
    let mut parts = Vec::new();
    for part in value.split_whitespace() {
        if !css_identifier_text_is_plain(part) {
            return None;
        }
        if let Some(rewritten_name) = rewritten_class_name_for(part, rewrites) {
            changed = true;
            parts.push(rewritten_name.to_string());
        } else {
            parts.push(part.to_string());
        }
    }
    changed.then(|| parts.join(" "))
}

fn rewritten_class_name_for<'a>(
    class_name: &str,
    rewrites: &'a [TransformClassNameRewriteV0],
) -> Option<&'a str> {
    rewrites.iter().find_map(|rewrite| {
        let original_name = normalize_reachable_class_name(&rewrite.original_name)?;
        let rewritten_name = normalize_reachable_class_name(&rewrite.rewritten_name)?;
        (original_name == class_name).then_some(rewritten_name)
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct KeyframesRuleSlice {
    name: String,
    start: usize,
    end: usize,
}

fn tree_shake_css_keyframes_with_lexer(
    source: &str,
    dialect: StyleDialect,
    reachable_keyframe_names: &[String],
) -> (String, Vec<TransformSemanticRemovalCandidate>) {
    let lexed = lex(source, dialect);
    let tokens = lexed.tokens();
    let keyframes = collect_top_level_keyframes_rules(tokens);
    if keyframes.is_empty() {
        return (source.to_string(), Vec::new());
    }

    let Some(mut referenced_names) = collect_referenced_keyframe_names(tokens) else {
        return (source.to_string(), Vec::new());
    };
    for name in reachable_keyframe_names {
        push_unique_string(&mut referenced_names, name.clone());
    }

    let removals = keyframes
        .iter()
        .filter(|keyframe| !referenced_names.iter().any(|name| name == &keyframe.name))
        .map(|keyframe| TransformSemanticRemovalCandidate {
            symbol_kind: "keyframes",
            name: keyframe.name.clone(),
            source_span_start: keyframe.start,
            source_span_end: keyframe.end,
            reason: "keyframes name was absent from animation references and the closed-style-world reachable keyframe set",
        })
        .collect::<Vec<_>>();
    let ranges = removals
        .iter()
        .map(|removal| (removal.source_span_start, removal.source_span_end))
        .collect::<Vec<_>>();
    let (output, _) = remove_source_ranges(source, &ranges);
    (output, removals)
}

fn collect_top_level_keyframes_rules(
    tokens: &[omena_parser::LexedToken],
) -> Vec<KeyframesRuleSlice> {
    let mut rules = Vec::new();
    let mut depth = 0usize;
    let mut index = 0;

    while index < tokens.len() {
        match tokens[index].kind {
            SyntaxKind::AtKeyword if depth == 0 && is_keyframes_at_keyword(&tokens[index].text) => {
                if let Some((rule, next_index)) = parse_top_level_keyframes_rule(tokens, index) {
                    rules.push(rule);
                    index = next_index;
                    continue;
                }
            }
            SyntaxKind::LeftBrace => depth += 1,
            SyntaxKind::RightBrace => depth = depth.saturating_sub(1),
            _ => {}
        }
        index += 1;
    }

    rules
}

fn is_keyframes_at_keyword(text: &str) -> bool {
    matches!(
        text.to_ascii_lowercase().as_str(),
        "@keyframes" | "@-webkit-keyframes"
    )
}

fn parse_top_level_keyframes_rule(
    tokens: &[omena_parser::LexedToken],
    at_keyframes_index: usize,
) -> Option<(KeyframesRuleSlice, usize)> {
    let name_index = skip_whitespace_tokens(tokens, at_keyframes_index + 1, tokens.len());
    let name_token = tokens.get(name_index)?;
    let name = static_keyframe_name_from_rule_name_token(name_token)?;
    let mut index = name_index + 1;
    while index < tokens.len() {
        match tokens[index].kind {
            SyntaxKind::Semicolon => return None,
            SyntaxKind::LeftBrace => {
                let close_index = matching_right_brace_index(tokens, index)?;
                return Some((
                    KeyframesRuleSlice {
                        name,
                        start: token_start(&tokens[at_keyframes_index]),
                        end: token_end(&tokens[close_index]),
                    },
                    close_index + 1,
                ));
            }
            _ => index += 1,
        }
    }

    None
}

fn static_keyframe_name_from_rule_name_token(token: &omena_parser::LexedToken) -> Option<String> {
    match token.kind {
        SyntaxKind::Ident => Some(token.text.clone()),
        SyntaxKind::String => static_css_string_value(&token.text),
        _ => None,
    }
}

fn collect_referenced_keyframe_names(tokens: &[omena_parser::LexedToken]) -> Option<Vec<String>> {
    let mut names = Vec::new();
    for (index, token) in tokens.iter().enumerate() {
        if token.kind != SyntaxKind::LeftBrace {
            continue;
        }
        let Some(close_index) = matching_right_brace_index(tokens, index) else {
            continue;
        };
        for declaration in collect_simple_declarations_in_block(tokens, index, close_index) {
            match declaration.property.as_str() {
                "animation-name" => {
                    if declaration.value.contains("var(") {
                        return None;
                    }
                    for name in split_top_level_value_arguments(&declaration.value)? {
                        if let Some(candidate) = static_animation_name_candidate(&name)
                            && (candidate.quoted || !candidate.name.eq_ignore_ascii_case("none"))
                        {
                            push_unique_string(&mut names, candidate.name);
                        }
                    }
                }
                "animation" => {
                    if declaration.value.contains("var(") {
                        return None;
                    }
                    for name in extract_animation_shorthand_name_candidates(&declaration.value)? {
                        push_unique_string(&mut names, name);
                    }
                }
                _ => {}
            }
        }
    }

    Some(names)
}

fn extract_animation_shorthand_name_candidates(value: &str) -> Option<Vec<String>> {
    let mut candidates = Vec::new();
    for branch in split_top_level_value_arguments(value)? {
        for part in branch.split_whitespace() {
            let candidate = part.trim();
            if let Some(candidate) = static_animation_name_candidate(candidate)
                && (candidate.quoted || !is_known_animation_shorthand_keyword(&candidate.name))
            {
                push_unique_string(&mut candidates, candidate.name);
            }
        }
    }
    Some(candidates)
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct StaticAnimationNameCandidate {
    name: String,
    quoted: bool,
}

fn static_animation_name_candidate(value: &str) -> Option<StaticAnimationNameCandidate> {
    let value = value.trim();
    if value.is_empty() {
        return None;
    }
    if let Some(name) = static_css_string_value(value) {
        return Some(StaticAnimationNameCandidate { name, quoted: true });
    }
    if value.contains(['(', ')', '"', '\'', '/', '\\'])
        || parse_numeric_value_with_unit(value).is_some()
    {
        return None;
    }
    Some(StaticAnimationNameCandidate {
        name: value.to_string(),
        quoted: false,
    })
}

fn static_css_string_value(value: &str) -> Option<String> {
    let value = value.trim();
    if value.len() < 2 {
        return None;
    }
    let quote = value.as_bytes()[0];
    if !matches!(quote, b'"' | b'\'') || value.as_bytes().last().copied() != Some(quote) {
        return None;
    }
    let inner = &value[1..value.len() - 1];
    if inner.is_empty() || inner.contains(['\\', '\n', '\r', '\x0c']) {
        return None;
    }
    Some(inner.to_string())
}

fn is_known_animation_shorthand_keyword(value: &str) -> bool {
    matches!(
        value.to_ascii_lowercase().as_str(),
        "alternate"
            | "alternate-reverse"
            | "backwards"
            | "both"
            | "ease"
            | "ease-in"
            | "ease-in-out"
            | "ease-out"
            | "forwards"
            | "infinite"
            | "linear"
            | "none"
            | "normal"
            | "paused"
            | "reverse"
            | "running"
            | "step-end"
            | "step-start"
    )
}

fn push_unique_string(values: &mut Vec<String>, value: String) {
    if !values.iter().any(|existing| existing == &value) {
        values.push(value);
    }
}

fn tree_shake_css_modules_values_with_lexer(
    source: &str,
    dialect: StyleDialect,
    reachable_value_names: &[String],
) -> (String, Vec<TransformSemanticRemovalCandidate>) {
    let lexed = lex(source, dialect);
    let tokens = lexed.tokens();
    let definitions = collect_static_local_css_modules_value_definitions(tokens);
    if definitions.is_empty() {
        return (source.to_string(), Vec::new());
    }

    let referenced_names = collect_reachable_css_modules_value_names(
        tokens,
        dialect,
        &definitions,
        reachable_value_names,
    );

    let removals = definitions
        .iter()
        .filter(|definition| {
            can_tree_shake_local_css_modules_value_definition(definition, dialect, &definitions)
                && !referenced_names.iter().any(|name| name == &definition.name)
        })
        .map(|definition| TransformSemanticRemovalCandidate {
            symbol_kind: "cssModuleValue",
            name: definition.name.clone(),
            source_span_start: definition.start,
            source_span_end: definition.end,
            reason: "CSS Modules value definition was absent from transitive value references and the closed-style-world reachable value set",
        })
        .collect::<Vec<_>>();

    let ranges = removals
        .iter()
        .map(|removal| (removal.source_span_start, removal.source_span_end))
        .collect::<Vec<_>>();
    let (output, _) = remove_source_ranges(source, &ranges);
    (output, removals)
}

fn collect_reachable_css_modules_value_names(
    tokens: &[omena_parser::LexedToken],
    dialect: StyleDialect,
    definitions: &[StaticCssModulesValueDefinition],
    external_roots: &[String],
) -> Vec<String> {
    let mut root_names = external_roots.to_vec();
    let mut dependencies_by_name = BTreeMap::<String, Vec<String>>::new();
    let definition_names = definitions
        .iter()
        .map(|definition| definition.name.clone())
        .collect::<Vec<_>>();

    for definition in definitions {
        for reference_name in collect_css_modules_value_references_in_value(
            &definition.value,
            dialect,
            &definition_names,
        ) {
            if reference_name == definition.name {
                continue;
            }
            let dependencies = dependencies_by_name
                .entry(definition.name.clone())
                .or_default();
            push_unique_string(dependencies, reference_name);
        }
    }

    let mut index = 0;
    while index < tokens.len() {
        if tokens[index].kind == SyntaxKind::LeftBrace
            && let Some(close_index) = matching_right_brace_index(tokens, index)
        {
            for declaration in collect_simple_declarations_in_block(tokens, index, close_index) {
                for reference_name in collect_css_modules_value_references_in_value(
                    &declaration.value,
                    dialect,
                    &definition_names,
                ) {
                    push_unique_string(&mut root_names, reference_name);
                }
            }
            index = close_index + 1;
            continue;
        }
        index += 1;
    }
    collect_css_modules_value_references_in_at_rule_preludes(
        tokens,
        &definition_names,
        &mut root_names,
    );

    close_css_modules_value_dependency_graph(root_names, &dependencies_by_name)
}

fn collect_css_modules_value_references_in_at_rule_preludes(
    tokens: &[omena_parser::LexedToken],
    definition_names: &[String],
    root_names: &mut Vec<String>,
) {
    let mut index = 0;
    while index < tokens.len() {
        if tokens[index].kind != SyntaxKind::AtKeyword
            || !at_rule_prelude_can_reference_css_modules_values(&tokens[index].text)
        {
            index += 1;
            continue;
        }

        let mut prelude_index = index + 1;
        while prelude_index < tokens.len() {
            match tokens[prelude_index].kind {
                SyntaxKind::Ident
                    if definition_names
                        .iter()
                        .any(|name| name == &tokens[prelude_index].text) =>
                {
                    push_unique_string(root_names, tokens[prelude_index].text.clone());
                }
                SyntaxKind::LeftBrace | SyntaxKind::Semicolon | SyntaxKind::RightBrace => break,
                _ => {}
            }
            prelude_index += 1;
        }
        index = prelude_index.saturating_add(1);
    }
}

fn at_rule_prelude_can_reference_css_modules_values(text: &str) -> bool {
    matches!(
        text.to_ascii_lowercase().as_str(),
        "@media" | "@supports" | "@container" | "@custom-media" | "@scope"
    )
}

fn close_css_modules_value_dependency_graph(
    roots: Vec<String>,
    dependencies_by_name: &BTreeMap<String, Vec<String>>,
) -> Vec<String> {
    let mut reachable = Vec::new();
    let mut queue = roots.into_iter().collect::<VecDeque<_>>();

    while let Some(name) = queue.pop_front() {
        if reachable.iter().any(|existing| existing == &name) {
            continue;
        }
        reachable.push(name.clone());
        if let Some(dependencies) = dependencies_by_name.get(&name) {
            for dependency in dependencies {
                queue.push_back(dependency.clone());
            }
        }
    }

    reachable.sort();
    reachable
}

fn can_tree_shake_local_css_modules_value_definition(
    definition: &StaticCssModulesValueDefinition,
    dialect: StyleDialect,
    definitions: &[StaticCssModulesValueDefinition],
) -> bool {
    let definition_names = definitions
        .iter()
        .map(|candidate| candidate.name.clone())
        .collect::<Vec<_>>();
    definitions
        .iter()
        .filter(|candidate| candidate.name == definition.name)
        .count()
        == 1
        && (is_static_css_modules_value_literal(&definition.value)
            || !collect_css_modules_value_references_in_value(
                &definition.value,
                dialect,
                &definition_names,
            )
            .is_empty())
}

fn collect_css_modules_value_references_in_value(
    value: &str,
    dialect: StyleDialect,
    definition_names: &[String],
) -> Vec<String> {
    let lexed = lex(value, dialect);
    let mut references = Vec::new();
    for token in lexed.tokens() {
        if token.kind == SyntaxKind::Ident
            && definition_names.iter().any(|name| name == &token.text)
        {
            push_unique_string(&mut references, token.text.clone());
        }
    }
    references
}

fn tree_shake_css_custom_properties_with_lexer(
    source: &str,
    dialect: StyleDialect,
    reachable_custom_property_names: &[String],
) -> (String, Vec<TransformSemanticRemovalCandidate>) {
    let lexed = lex(source, dialect);
    let tokens = lexed.tokens();
    let Some(referenced_names) =
        collect_reachable_custom_property_names(tokens, reachable_custom_property_names)
    else {
        return (source.to_string(), Vec::new());
    };

    let mut removals = Vec::new();
    let mut index = 0;
    while index < tokens.len() {
        if tokens[index].kind == SyntaxKind::LeftBrace
            && let Some(close_index) = matching_right_brace_index(tokens, index)
        {
            for declaration in collect_simple_declarations_in_block(tokens, index, close_index) {
                if declaration.property.starts_with("--")
                    && !referenced_names
                        .iter()
                        .any(|name| name == &declaration.property)
                {
                    removals.push(TransformSemanticRemovalCandidate {
                        symbol_kind: "customProperty",
                        name: declaration.property,
                        source_span_start: declaration.start,
                        source_span_end: declaration.end,
                        reason: "custom property declaration was absent from transitive var() references and the closed-style-world reachable custom-property set",
                    });
                }
            }
            index = close_index + 1;
            continue;
        }
        index += 1;
    }

    let ranges = removals
        .iter()
        .map(|removal| (removal.source_span_start, removal.source_span_end))
        .collect::<Vec<_>>();
    let (output, _) = remove_source_ranges(source, &ranges);
    (output, removals)
}

fn collect_reachable_custom_property_names(
    tokens: &[omena_parser::LexedToken],
    external_roots: &[String],
) -> Option<Vec<String>> {
    let mut root_names = Vec::new();
    let mut dependencies_by_name = BTreeMap::<String, Vec<String>>::new();

    for name in external_roots {
        if let Some(name) = normalize_custom_property_name(name) {
            push_unique_string(&mut root_names, name.to_string());
        }
    }

    for (index, token) in tokens.iter().enumerate() {
        if token.kind != SyntaxKind::LeftBrace {
            continue;
        }
        let Some(close_index) = matching_right_brace_index(tokens, index) else {
            continue;
        };
        for declaration in collect_simple_declarations_in_block(tokens, index, close_index) {
            let referenced_names = collect_custom_property_references_in_value(&declaration.value)?;
            if declaration.property.starts_with("--") {
                let dependencies = dependencies_by_name
                    .entry(declaration.property)
                    .or_default();
                for name in referenced_names {
                    push_unique_string(dependencies, name);
                }
            } else {
                for name in referenced_names {
                    push_unique_string(&mut root_names, name);
                }
            }
        }
    }

    Some(close_custom_property_dependency_graph(
        root_names,
        &dependencies_by_name,
    ))
}

fn close_custom_property_dependency_graph(
    roots: Vec<String>,
    dependencies_by_name: &BTreeMap<String, Vec<String>>,
) -> Vec<String> {
    let mut reachable = Vec::new();
    let mut queue = roots.into_iter().collect::<VecDeque<_>>();

    while let Some(name) = queue.pop_front() {
        if reachable.iter().any(|existing| existing == &name) {
            continue;
        }
        reachable.push(name.clone());
        if let Some(dependencies) = dependencies_by_name.get(&name) {
            for dependency in dependencies {
                queue.push_back(dependency.clone());
            }
        }
    }

    reachable.sort();
    reachable
}

fn collect_custom_property_references_in_value(value: &str) -> Option<Vec<String>> {
    let mut names = Vec::new();
    let mut search_start = 0;
    while let Some(relative_index) = find_ascii_case_insensitive(&value[search_start..], "var(") {
        let mut index = search_start + relative_index + "var(".len();
        while matches!(
            value.as_bytes().get(index),
            Some(b' ' | b'\n' | b'\r' | b'\t')
        ) {
            index += 1;
        }
        let name_start = index;
        if !value[name_start..].starts_with("--") {
            return None;
        }
        index += 2;
        while let Some(ch) = value[index..].chars().next() {
            if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_') {
                index += ch.len_utf8();
            } else {
                break;
            }
        }
        if index == name_start + 2 || !value[index..].contains(')') {
            return None;
        }
        push_unique_string(&mut names, value[name_start..index].to_string());
        search_start = index;
    }
    Some(names)
}

fn find_ascii_case_insensitive(haystack: &str, needle: &str) -> Option<usize> {
    let needle = needle.as_bytes();
    if needle.is_empty() || haystack.len() < needle.len() {
        return None;
    }
    haystack
        .as_bytes()
        .windows(needle.len())
        .position(|window| window.eq_ignore_ascii_case(needle))
}

fn normalize_custom_property_name(name: &str) -> Option<&str> {
    let name = name.trim();
    if name.starts_with("--") && name.len() > 2 {
        return Some(name);
    }
    None
}

fn css_identifier_text_is_plain(text: &str) -> bool {
    text.chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_'))
}

fn remove_source_ranges(source: &str, ranges: &[(usize, usize)]) -> (String, usize) {
    if ranges.is_empty() {
        return (source.to_string(), 0);
    }

    let mut ranges = ranges.to_vec();
    ranges.sort_by_key(|(start, _)| *start);
    let mut output = String::with_capacity(source.len());
    let mut cursor = 0;
    let mut removed_count = 0;
    for (start, end) in &ranges {
        if *start < cursor {
            continue;
        }
        if *start > cursor {
            output.push_str(&source[cursor..*start]);
        }
        cursor = *end;
        removed_count += 1;
    }
    if cursor < source.len() {
        output.push_str(&source[cursor..]);
    }

    (output, removed_count)
}

fn replace_source_ranges(source: &str, replacements: &[(usize, usize, String)]) -> (String, usize) {
    if replacements.is_empty() {
        return (source.to_string(), 0);
    }

    let mut replacements = replacements.to_vec();
    replacements.sort_by_key(|(start, _, _)| *start);
    let mut output = String::with_capacity(source.len());
    let mut cursor = 0;
    let mut replacement_count = 0;
    for (start, end, replacement) in &replacements {
        if *start < cursor {
            continue;
        }
        if *start > cursor {
            output.push_str(&source[cursor..*start]);
        }
        output.push_str(replacement);
        cursor = *end;
        replacement_count += 1;
    }
    if cursor < source.len() {
        output.push_str(&source[cursor..]);
    }

    (output, replacement_count)
}

fn substitute_static_css_custom_properties_with_lexer(
    source: &str,
    dialect: StyleDialect,
) -> (String, usize) {
    let lexed = lex(source, dialect);
    let tokens = lexed.tokens();
    let env_rules = collect_top_level_ordinary_rule_slices(source, tokens);
    let rules = collect_declaration_ordinary_rule_slices(source, tokens);
    let env = resolve_custom_property_env_least_fixed_point(
        &collect_static_root_custom_property_env(tokens, &env_rules),
    );
    if env.is_empty() {
        return (source.to_string(), 0);
    }

    let mut replacements = Vec::new();
    for rule in &rules {
        let Some((block_start_index, block_end_index)) =
            rule_block_token_indexes(tokens, rule.block_start, rule.block_end)
        else {
            continue;
        };
        for declaration in
            collect_simple_declarations_in_block(tokens, block_start_index, block_end_index)
        {
            if declaration.property.starts_with("--") {
                continue;
            }
            let Some(resolved_value) =
                substitute_static_custom_property_references_in_value(&declaration.value, &env)
            else {
                continue;
            };
            replacements.push((
                declaration.start,
                declaration.end,
                format!("{}: {resolved_value};", declaration.property),
            ));
        }
    }

    if replacements.is_empty() {
        return (source.to_string(), 0);
    }

    let mut output = String::with_capacity(source.len());
    let mut cursor = 0;
    for (start, end, replacement) in &replacements {
        if *start > cursor {
            output.push_str(&source[cursor..*start]);
        }
        output.push_str(replacement);
        cursor = *end;
    }
    if cursor < source.len() {
        output.push_str(&source[cursor..]);
    }

    (output, replacements.len())
}

fn substitute_static_custom_property_references_in_value(
    value: &str,
    env: &CustomPropertyEnv,
) -> Option<String> {
    let mut output = String::with_capacity(value.len());
    let mut cursor = 0usize;
    let mut index = 0usize;
    let mut quote: Option<char> = None;
    let mut changed = false;

    while index < value.len() {
        let ch = value[index..].chars().next()?;

        if let Some(quote_ch) = quote {
            index += ch.len_utf8();
            if ch == '\\' {
                let escaped = value[index..].chars().next()?;
                index += escaped.len_utf8();
            } else if ch == quote_ch {
                quote = None;
            }
            continue;
        }

        match ch {
            '"' | '\'' => {
                quote = Some(ch);
                index += ch.len_utf8();
            }
            _ if value[index..]
                .get(.."var(".len())
                .is_some_and(|text| text.eq_ignore_ascii_case("var(")) =>
            {
                let left_paren_index = index + "var".len();
                let Some(close_index) = matching_function_call_end(value, left_paren_index) else {
                    return None;
                };
                let Some(arguments) =
                    split_top_level_value_arguments(&value[left_paren_index + 1..close_index])
                else {
                    index += ch.len_utf8();
                    continue;
                };
                let Some(var_value) = parse_static_var_arguments(&arguments) else {
                    index += ch.len_utf8();
                    continue;
                };
                let CascadeValue::Literal(resolved_value) =
                    substitute_custom_properties(&var_value, env)
                else {
                    index += ch.len_utf8();
                    continue;
                };
                output.push_str(&value[cursor..index]);
                output.push_str(&resolved_value);
                index = close_index + ')'.len_utf8();
                cursor = index;
                changed = true;
            }
            _ => {
                index += ch.len_utf8();
            }
        }
    }

    if !changed {
        return None;
    }
    output.push_str(&value[cursor..]);
    Some(output)
}

fn collect_static_root_custom_property_env(
    tokens: &[omena_parser::LexedToken],
    rules: &[SimpleRuleSlice],
) -> CustomPropertyEnv {
    let mut env = CustomPropertyEnv::new();
    let mut blocked_names = Vec::new();

    for rule in rules {
        if rule.selector != ":root" {
            continue;
        }
        let Some((block_start_index, block_end_index)) =
            rule_block_token_indexes(tokens, rule.block_start, rule.block_end)
        else {
            continue;
        };
        for declaration in
            collect_simple_declarations_in_block(tokens, block_start_index, block_end_index)
        {
            if !declaration.property.starts_with("--") || declaration.important {
                continue;
            }
            if blocked_names.contains(&declaration.property) {
                continue;
            }
            if env.contains_key(&declaration.property) {
                env.remove(&declaration.property);
                blocked_names.push(declaration.property);
                continue;
            }
            let Some(value) = parse_static_custom_property_env_value(&declaration.value) else {
                continue;
            };
            env.insert(declaration.property, value);
        }
    }

    env
}

fn parse_static_custom_property_env_value(value: &str) -> Option<CascadeValue> {
    parse_static_var_value(value)
        .or_else(|| (!value.contains("var(")).then(|| CascadeValue::Literal(value.to_string())))
}

fn parse_static_var_value(value: &str) -> Option<CascadeValue> {
    let arguments = parse_whole_function_value_arguments(value, "var")?;
    parse_static_var_arguments(&arguments)
}

fn parse_static_var_arguments(arguments: &[String]) -> Option<CascadeValue> {
    match arguments {
        [name] if name.starts_with("--") => Some(CascadeValue::Var {
            name: name.clone(),
            fallback: None,
        }),
        [name, fallback] if name.starts_with("--") => {
            let fallback = parse_static_custom_property_env_value(fallback)?;
            Some(CascadeValue::Var {
                name: name.clone(),
                fallback: Some(Box::new(fallback)),
            })
        }
        _ => None,
    }
}

fn reduce_css_calc_with_lexer(source: &str, dialect: StyleDialect) -> (String, usize) {
    let lexed = lex(source, dialect);
    let tokens = lexed.tokens();
    let mut replacements = Vec::new();
    let mut index = 0;

    while index < tokens.len() {
        if tokens[index].kind == SyntaxKind::LeftBrace
            && let Some(close_index) = matching_right_brace_index(tokens, index)
        {
            let declarations = collect_simple_declarations_in_block(tokens, index, close_index);
            for declaration in declarations {
                let Some(replacement_value) = parse_reducible_calc_value(&declaration.value) else {
                    continue;
                };
                replacements.push((
                    declaration.start,
                    declaration.end,
                    format!("{}: {replacement_value};", declaration.property),
                ));
            }
            index = close_index + 1;
            continue;
        }
        index += 1;
    }

    if replacements.is_empty() {
        return (source.to_string(), 0);
    }

    let mut output = String::with_capacity(source.len());
    let mut cursor = 0;
    for (start, end, replacement) in &replacements {
        if *start > cursor {
            output.push_str(&source[cursor..*start]);
        }
        output.push_str(replacement);
        cursor = *end;
    }
    if cursor < source.len() {
        output.push_str(&source[cursor..]);
    }

    (output, replacements.len())
}

fn parse_reducible_calc_value(value: &str) -> Option<String> {
    let inner = parse_whole_function_value_inner(value, "calc")?;

    for (operator_index, operator) in top_level_calc_additive_operators(inner) {
        let Some(left) = parse_numeric_value_with_unit(inner[..operator_index].trim()) else {
            continue;
        };
        let Some(right) =
            parse_numeric_value_with_unit(inner[operator_index + operator.len_utf8()..].trim())
        else {
            continue;
        };
        if left.unit != right.unit {
            continue;
        }
        let value = match operator {
            '+' => left.value + right.value,
            '-' => left.value - right.value,
            _ => return None,
        };
        return Some(format!("{}{}", format_css_number(value), left.unit));
    }

    None
}

fn top_level_calc_additive_operators(inner: &str) -> Vec<(usize, char)> {
    let mut operators = Vec::new();
    let mut depth = 0usize;
    let mut bracket_depth = 0usize;
    let mut quote: Option<char> = None;
    let mut escaped = false;

    for (index, ch) in inner.char_indices() {
        if let Some(active_quote) = quote {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == active_quote {
                quote = None;
            }
            continue;
        }

        match ch {
            '"' | '\'' => quote = Some(ch),
            '(' => depth += 1,
            ')' => depth = depth.saturating_sub(1),
            '[' => bracket_depth += 1,
            ']' => bracket_depth = bracket_depth.saturating_sub(1),
            '+' | '-' if depth == 0 && bracket_depth == 0 => {
                let left = inner[..index].trim_end();
                let right = inner[index + ch.len_utf8()..].trim_start();
                if left.is_empty() || right.is_empty() || left.ends_with(['e', 'E']) {
                    continue;
                }
                operators.push((index, ch));
            }
            _ => {}
        }
    }

    operators
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct NumericValueWithUnit<'a> {
    value: f64,
    unit: &'a str,
}

fn parse_numeric_value_with_unit(text: &str) -> Option<NumericValueWithUnit<'_>> {
    let split = numeric_prefix_end(text)?;
    let (number, unit) = text.split_at(split);
    let value = number.parse::<f64>().ok()?;
    value
        .is_finite()
        .then_some(NumericValueWithUnit { value, unit })
}

fn format_css_number(value: f64) -> String {
    if value.fract() == 0.0 {
        return format!("{value:.0}");
    }
    let formatted = format!("{value:.6}");
    formatted
        .trim_end_matches('0')
        .trim_end_matches('.')
        .to_string()
}

fn rule_block_token_indexes(
    tokens: &[omena_parser::LexedToken],
    block_start: usize,
    block_end: usize,
) -> Option<(usize, usize)> {
    let start_index = tokens
        .iter()
        .position(|token| token_start(token) == block_start)?;
    let end_index = tokens
        .iter()
        .position(|token| token_start(token) == block_end)?;
    Some((start_index, end_index))
}

fn is_light_dark_lowerable_property(property: &str) -> bool {
    matches!(
        property,
        "background"
            | "background-color"
            | "border-color"
            | "caret-color"
            | "color"
            | "fill"
            | "outline-color"
            | "stroke"
            | "text-decoration-color"
    )
}

fn parse_light_dark_value(value: &str) -> Option<(String, String)> {
    let arguments = parse_whole_function_value_arguments(value, "light-dark")?;
    let [light, dark] = arguments.as_slice() else {
        return None;
    };
    if light.is_empty() || dark.is_empty() {
        return None;
    }
    Some((light.clone(), dark.clone()))
}

fn substitute_static_css_function_references_in_value(
    value: &str,
    functions: &[(&str, fn(&str) -> Option<String>)],
) -> Option<String> {
    let mut output = String::with_capacity(value.len());
    let mut cursor = 0usize;
    let mut index = 0usize;
    let mut quote: Option<char> = None;
    let mut changed = false;

    while index < value.len() {
        let ch = value[index..].chars().next()?;

        if let Some(quote_ch) = quote {
            index += ch.len_utf8();
            if ch == '\\' {
                let escaped = value[index..].chars().next()?;
                index += escaped.len_utf8();
            } else if ch == quote_ch {
                quote = None;
            }
            continue;
        }

        match ch {
            '"' | '\'' => {
                quote = Some(ch);
                index += ch.len_utf8();
            }
            _ => {
                let Some((function_name, parse_function_value)) =
                    static_css_function_at(value, index, functions)
                else {
                    index += ch.len_utf8();
                    continue;
                };
                let left_paren_index = index + function_name.len();
                let Some(close_index) = matching_function_call_end(value, left_paren_index) else {
                    index += ch.len_utf8();
                    continue;
                };
                let function_value = &value[index..close_index + ')'.len_utf8()];
                let Some(replacement_value) = parse_function_value(function_value) else {
                    index += ch.len_utf8();
                    continue;
                };
                output.push_str(&value[cursor..index]);
                output.push_str(&replacement_value);
                index = close_index + ')'.len_utf8();
                cursor = index;
                changed = true;
            }
        }
    }

    if !changed {
        return None;
    }
    output.push_str(&value[cursor..]);
    Some(output)
}

fn static_css_function_at<'a>(
    value: &str,
    index: usize,
    functions: &'a [(&str, fn(&str) -> Option<String>)],
) -> Option<(&'a str, fn(&str) -> Option<String>)> {
    functions.iter().find_map(|(function_name, parser)| {
        let name = value.get(index..index + function_name.len())?;
        let open_paren = value[index + function_name.len()..].chars().next()?;
        (name.eq_ignore_ascii_case(function_name) && open_paren == '(')
            .then_some((*function_name, *parser))
    })
}

fn parse_color_mix_value(value: &str) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, "color-mix")?;
    let [space, first, second] = arguments.as_slice() else {
        return None;
    };
    if normalize_ascii_whitespace(space) != "in srgb" {
        return None;
    }

    let first_stop = parse_static_color_mix_stop(first)?;
    let second_stop = parse_static_color_mix_stop(second)?;
    let (first_weight, second_weight) =
        color_mix_weights(first_stop.percentage, second_stop.percentage)?;
    let mixed = mix_srgb_colors(
        first_stop.color,
        second_stop.color,
        first_weight,
        second_weight,
    );
    Some(mixed.to_css_rgb())
}

fn parse_whole_function_value_arguments(value: &str, function_name: &str) -> Option<Vec<String>> {
    split_top_level_value_arguments(parse_whole_function_value_inner(value, function_name)?)
}

fn parse_whole_function_value_inner<'a>(value: &'a str, function_name: &str) -> Option<&'a str> {
    let value = value.trim();
    let name = value.get(..function_name.len())?;
    if !name.eq_ignore_ascii_case(function_name) {
        return None;
    }
    value
        .get(function_name.len()..)?
        .strip_prefix('(')?
        .strip_suffix(')')
}

fn split_top_level_value_arguments(inner: &str) -> Option<Vec<String>> {
    let mut arguments = Vec::new();
    let mut current = String::new();
    let mut depth = 0usize;
    let mut bracket_depth = 0usize;
    let mut quote: Option<char> = None;
    let mut escaped = false;

    for ch in inner.chars() {
        if let Some(active_quote) = quote {
            current.push(ch);
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == active_quote {
                quote = None;
            }
            continue;
        }

        match ch {
            '"' | '\'' => {
                quote = Some(ch);
                current.push(ch);
            }
            '(' => {
                depth += 1;
                current.push(ch);
            }
            ')' => {
                depth = depth.checked_sub(1)?;
                current.push(ch);
            }
            '[' => {
                bracket_depth += 1;
                current.push(ch);
            }
            ']' => {
                bracket_depth = bracket_depth.checked_sub(1)?;
                current.push(ch);
            }
            ',' if depth == 0 && bracket_depth == 0 => {
                let argument = current.trim().to_string();
                if argument.is_empty() {
                    return None;
                }
                arguments.push(argument);
                current.clear();
            }
            _ => current.push(ch),
        }
    }

    if quote.is_some() || depth != 0 || bracket_depth != 0 {
        return None;
    }

    let argument = current.trim().to_string();
    if argument.is_empty() {
        return None;
    }
    arguments.push(argument);
    Some(arguments)
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct StaticColorMixStop {
    color: SrgbColor,
    percentage: Option<f64>,
}

fn parse_static_color_mix_stop(input: &str) -> Option<StaticColorMixStop> {
    let (color_text, percentage) = split_static_color_mix_stop(input)?;
    Some(StaticColorMixStop {
        color: parse_static_color_mix_operand(&color_text)?,
        percentage,
    })
}

fn split_static_color_mix_stop(input: &str) -> Option<(String, Option<f64>)> {
    let input = input.trim();
    if input.is_empty() {
        return None;
    }

    let mut depth = 0usize;
    let mut bracket_depth = 0usize;
    let mut quote: Option<char> = None;
    let mut escaped = false;
    let mut in_top_level_whitespace = false;
    let mut last_top_level_whitespace_start = None;

    for (index, ch) in input.char_indices() {
        if let Some(active_quote) = quote {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == active_quote {
                quote = None;
            }
            continue;
        }

        match ch {
            '"' | '\'' => {
                quote = Some(ch);
                in_top_level_whitespace = false;
            }
            '(' => {
                depth += 1;
                in_top_level_whitespace = false;
            }
            ')' => {
                depth = depth.checked_sub(1)?;
                in_top_level_whitespace = false;
            }
            '[' => {
                bracket_depth += 1;
                in_top_level_whitespace = false;
            }
            ']' => {
                bracket_depth = bracket_depth.checked_sub(1)?;
                in_top_level_whitespace = false;
            }
            ch if ch.is_ascii_whitespace() && depth == 0 && bracket_depth == 0 => {
                if !in_top_level_whitespace {
                    last_top_level_whitespace_start = Some(index);
                }
                in_top_level_whitespace = true;
            }
            _ => in_top_level_whitespace = false,
        }
    }

    if quote.is_some() || depth != 0 || bracket_depth != 0 {
        return None;
    }

    if let Some(separator_start) = last_top_level_whitespace_start {
        let color = input[..separator_start].trim();
        let percentage = input[separator_start..].trim();
        if !color.is_empty()
            && let Some(percentage) = parse_bounded_percentage(percentage)
        {
            return Some((color.to_string(), Some(percentage)));
        }
    }

    Some((input.to_string(), None))
}

fn parse_static_color_mix_operand(text: &str) -> Option<SrgbColor> {
    parse_static_srgb_color(text)
        .or_else(|| parse_static_rgb_function_color(text))
        .or_else(|| parse_static_hsl_function_color(text))
        .or_else(|| parse_static_hwb_function_color(text))
}

fn color_mix_weights(first: Option<f64>, second: Option<f64>) -> Option<(f64, f64)> {
    match (first, second) {
        (None, None) => Some((0.5, 0.5)),
        (Some(first), None) => Some((first, 1.0 - first)),
        (None, Some(second)) => Some((1.0 - second, second)),
        (Some(first), Some(second)) if (first + second - 1.0).abs() <= 0.000_001 => {
            Some((first, second))
        }
        _ => None,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SrgbColor {
    red: u8,
    green: u8,
    blue: u8,
}

impl SrgbColor {
    fn to_css_rgb(self) -> String {
        format!("rgb({} {} {})", self.red, self.green, self.blue)
    }
}

fn mix_srgb_colors(
    first: SrgbColor,
    second: SrgbColor,
    first_weight: f64,
    second_weight: f64,
) -> SrgbColor {
    SrgbColor {
        red: mix_srgb_channel(first.red, second.red, first_weight, second_weight),
        green: mix_srgb_channel(first.green, second.green, first_weight, second_weight),
        blue: mix_srgb_channel(first.blue, second.blue, first_weight, second_weight),
    }
}

fn mix_srgb_channel(first: u8, second: u8, first_weight: f64, second_weight: f64) -> u8 {
    let value = f64::from(first) * first_weight + f64::from(second) * second_weight;
    value.round().clamp(0.0, 255.0) as u8
}

fn parse_static_srgb_color(text: &str) -> Option<SrgbColor> {
    parse_static_hex_color(text).or_else(|| parse_basic_named_srgb_color(text))
}

fn parse_static_hex_color(text: &str) -> Option<SrgbColor> {
    let hex = text.strip_prefix('#')?;
    match hex.len() {
        3 => {
            let mut chars = hex.chars();
            Some(SrgbColor {
                red: parse_repeated_hex_digit(chars.next()?)?,
                green: parse_repeated_hex_digit(chars.next()?)?,
                blue: parse_repeated_hex_digit(chars.next()?)?,
            })
        }
        6 => Some(SrgbColor {
            red: u8::from_str_radix(hex.get(0..2)?, 16).ok()?,
            green: u8::from_str_radix(hex.get(2..4)?, 16).ok()?,
            blue: u8::from_str_radix(hex.get(4..6)?, 16).ok()?,
        }),
        _ => None,
    }
}

fn parse_repeated_hex_digit(ch: char) -> Option<u8> {
    let digit = ch.to_digit(16)? as u8;
    Some(digit * 17)
}

fn parse_basic_named_srgb_color(text: &str) -> Option<SrgbColor> {
    match text.to_ascii_lowercase().as_str() {
        "aqua" | "cyan" => Some(SrgbColor {
            red: 0,
            green: 255,
            blue: 255,
        }),
        "black" => Some(SrgbColor {
            red: 0,
            green: 0,
            blue: 0,
        }),
        "blue" => Some(SrgbColor {
            red: 0,
            green: 0,
            blue: 255,
        }),
        "fuchsia" | "magenta" => Some(SrgbColor {
            red: 255,
            green: 0,
            blue: 255,
        }),
        "gray" | "grey" => Some(SrgbColor {
            red: 128,
            green: 128,
            blue: 128,
        }),
        "green" => Some(SrgbColor {
            red: 0,
            green: 128,
            blue: 0,
        }),
        "lime" => Some(SrgbColor {
            red: 0,
            green: 255,
            blue: 0,
        }),
        "maroon" => Some(SrgbColor {
            red: 128,
            green: 0,
            blue: 0,
        }),
        "navy" => Some(SrgbColor {
            red: 0,
            green: 0,
            blue: 128,
        }),
        "olive" => Some(SrgbColor {
            red: 128,
            green: 128,
            blue: 0,
        }),
        "orange" => Some(SrgbColor {
            red: 255,
            green: 165,
            blue: 0,
        }),
        "purple" => Some(SrgbColor {
            red: 128,
            green: 0,
            blue: 128,
        }),
        "red" => Some(SrgbColor {
            red: 255,
            green: 0,
            blue: 0,
        }),
        "silver" => Some(SrgbColor {
            red: 192,
            green: 192,
            blue: 192,
        }),
        "teal" => Some(SrgbColor {
            red: 0,
            green: 128,
            blue: 128,
        }),
        "white" => Some(SrgbColor {
            red: 255,
            green: 255,
            blue: 255,
        }),
        "yellow" => Some(SrgbColor {
            red: 255,
            green: 255,
            blue: 0,
        }),
        _ => None,
    }
}

fn normalize_ascii_whitespace(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn parse_oklab_oklch_value(value: &str) -> Option<String> {
    parse_oklab_value(value)
        .or_else(|| parse_oklch_value(value))
        .map(SrgbColor::to_css_rgb)
}

fn parse_color_function_value(value: &str) -> Option<String> {
    let inner = parse_whole_function_value_inner(value, "color")?;
    if inner.contains(',') {
        return None;
    }
    let parts = inner.split_whitespace().collect::<Vec<_>>();
    let (space, red, green, blue) = match parts.as_slice() {
        [space, red, green, blue] => (*space, *red, *green, *blue),
        [space, red, green, blue, "/", alpha] if parse_opaque_alpha(alpha)? => {
            (*space, *red, *green, *blue)
        }
        _ => return None,
    };
    let color = if space.eq_ignore_ascii_case("srgb") {
        SrgbColor {
            red: parse_srgb_component(red)?,
            green: parse_srgb_component(green)?,
            blue: parse_srgb_component(blue)?,
        }
    } else if space.eq_ignore_ascii_case("display-p3") {
        display_p3_to_srgb(
            parse_unit_interval_component(red)?,
            parse_unit_interval_component(green)?,
            parse_unit_interval_component(blue)?,
        )?
    } else {
        return None;
    };
    Some(color.to_css_rgb())
}

fn parse_opaque_alpha(text: &str) -> Option<bool> {
    let value = if let Some(percent) = text.strip_suffix('%') {
        parse_plain_f64(percent)? / 100.0
    } else {
        parse_plain_f64(text)?
    };
    Some((value - 1.0).abs() <= f64::EPSILON)
}

fn parse_oklab_value(value: &str) -> Option<SrgbColor> {
    let inner = parse_whole_function_value_inner(value, "oklab")?;
    let parts = split_ascii_space_separated_color_args(inner)?;
    let [lightness, a_axis, b_axis] = parts.as_slice() else {
        return None;
    };
    let lightness = parse_ok_lightness(lightness)?;
    let a_axis = parse_plain_f64(a_axis)?;
    let b_axis = parse_plain_f64(b_axis)?;
    oklab_to_srgb(lightness, a_axis, b_axis)
}

fn parse_oklch_value(value: &str) -> Option<SrgbColor> {
    let inner = parse_whole_function_value_inner(value, "oklch")?;
    let parts = split_ascii_space_separated_color_args(inner)?;
    let [lightness, chroma, hue] = parts.as_slice() else {
        return None;
    };
    let lightness = parse_ok_lightness(lightness)?;
    let chroma = parse_plain_f64(chroma)?;
    let hue = parse_hue_degrees(hue)?.to_radians();
    oklab_to_srgb(lightness, chroma * hue.cos(), chroma * hue.sin())
}

fn split_ascii_space_separated_color_args(inner: &str) -> Option<Vec<&str>> {
    if inner.contains('/') || inner.contains(',') {
        return None;
    }
    let parts = inner.split_whitespace().collect::<Vec<_>>();
    (!parts.is_empty()).then_some(parts)
}

fn parse_ok_lightness(text: &str) -> Option<f64> {
    let value = if let Some(percent) = text.strip_suffix('%') {
        parse_plain_f64(percent)? / 100.0
    } else {
        parse_plain_f64(text)?
    };
    value
        .is_finite()
        .then_some(value)
        .filter(|value| *value >= 0.0 && *value <= 1.0)
}

fn parse_hue_degrees(text: &str) -> Option<f64> {
    let lower = text.to_ascii_lowercase();
    let value = if lower.ends_with("deg") {
        parse_plain_f64(text.get(..text.len() - 3)?)?
    } else if lower.ends_with("turn") {
        parse_plain_f64(text.get(..text.len() - 4)?)? * 360.0
    } else if lower.ends_with("grad") {
        parse_plain_f64(text.get(..text.len() - 4)?)? * 0.9
    } else if lower.ends_with("rad") {
        parse_plain_f64(text.get(..text.len() - 3)?)?.to_degrees()
    } else {
        parse_plain_f64(text)?
    };
    value.is_finite().then_some(value)
}

fn parse_plain_f64(text: &str) -> Option<f64> {
    if text.contains('%') {
        return None;
    }
    text.parse::<f64>().ok().filter(|value| value.is_finite())
}

fn parse_srgb_component(text: &str) -> Option<u8> {
    Some((parse_unit_interval_component(text)? * 255.0).round() as u8)
}

fn parse_unit_interval_component(text: &str) -> Option<f64> {
    let value = if let Some(percent) = text.strip_suffix('%') {
        parse_plain_f64(percent)? / 100.0
    } else {
        parse_plain_f64(text)?
    };
    if !(0.0..=1.0).contains(&value) {
        return None;
    }
    Some(value)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InlineDirection {
    Ltr,
    Rtl,
}

fn static_horizontal_direction_for_declarations(
    declarations: &[SimpleDeclarationSlice],
) -> Option<InlineDirection> {
    let writing_mode = declarations
        .iter()
        .rev()
        .find(|declaration| declaration.property == "writing-mode")
        .map(|declaration| declaration.value.as_str());
    if !matches!(writing_mode, None | Some("horizontal-tb")) {
        return None;
    }

    declarations
        .iter()
        .rev()
        .find(|declaration| declaration.property == "direction")
        .and_then(|declaration| match declaration.value.as_str() {
            "ltr" => Some(InlineDirection::Ltr),
            "rtl" => Some(InlineDirection::Rtl),
            _ => None,
        })
}

fn physical_property_for_logical_property(
    property: &str,
    direction: InlineDirection,
) -> Option<&'static str> {
    match property {
        "block-size" => Some("height"),
        "inline-size" => Some("width"),
        "max-block-size" => Some("max-height"),
        "max-inline-size" => Some("max-width"),
        "min-block-size" => Some("min-height"),
        "min-inline-size" => Some("min-width"),
        "inset-inline-start" => Some(inline_start_property(direction, "left", "right")),
        "inset-inline-end" => Some(inline_end_property(direction, "left", "right")),
        "margin-inline-start" => Some(inline_start_property(
            direction,
            "margin-left",
            "margin-right",
        )),
        "margin-inline-end" => Some(inline_end_property(
            direction,
            "margin-left",
            "margin-right",
        )),
        "padding-inline-start" => Some(inline_start_property(
            direction,
            "padding-left",
            "padding-right",
        )),
        "padding-inline-end" => Some(inline_end_property(
            direction,
            "padding-left",
            "padding-right",
        )),
        "border-inline-start-color" => Some(inline_start_property(
            direction,
            "border-left-color",
            "border-right-color",
        )),
        "border-inline-end-color" => Some(inline_end_property(
            direction,
            "border-left-color",
            "border-right-color",
        )),
        "border-inline-start-style" => Some(inline_start_property(
            direction,
            "border-left-style",
            "border-right-style",
        )),
        "border-inline-end-style" => Some(inline_end_property(
            direction,
            "border-left-style",
            "border-right-style",
        )),
        "border-inline-start-width" => Some(inline_start_property(
            direction,
            "border-left-width",
            "border-right-width",
        )),
        "border-inline-end-width" => Some(inline_end_property(
            direction,
            "border-left-width",
            "border-right-width",
        )),
        "border-inline-start" => Some(inline_start_property(
            direction,
            "border-left",
            "border-right",
        )),
        "border-inline-end" => Some(inline_end_property(
            direction,
            "border-left",
            "border-right",
        )),
        _ => None,
    }
}

fn physical_declaration_for_logical_declaration(
    property: &str,
    value: &str,
    direction: InlineDirection,
) -> Option<String> {
    if let Some(physical_property) = physical_property_for_logical_property(property, direction) {
        return Some(format!("{physical_property}: {value};"));
    }

    if let Some((start_property, end_property)) =
        physical_pair_properties_for_logical_pair(property, direction)
    {
        let (start_value, end_value) = logical_pair_values(value)?;
        return Some(format!(
            "{start_property}: {start_value}; {end_property}: {end_value};"
        ));
    }

    if let Some((start_property, end_property)) =
        physical_pair_properties_for_logical_mirror(property, direction)
    {
        return Some(format!(
            "{start_property}: {value}; {end_property}: {value};"
        ));
    }

    None
}

fn physical_pair_properties_for_logical_pair(
    property: &str,
    direction: InlineDirection,
) -> Option<(&'static str, &'static str)> {
    match property {
        "inset-inline" => Some(inline_start_end_properties(direction, "left", "right")),
        "margin-inline" => Some(inline_start_end_properties(
            direction,
            "margin-left",
            "margin-right",
        )),
        "padding-inline" => Some(inline_start_end_properties(
            direction,
            "padding-left",
            "padding-right",
        )),
        "scroll-margin-inline" => Some(inline_start_end_properties(
            direction,
            "scroll-margin-left",
            "scroll-margin-right",
        )),
        "scroll-padding-inline" => Some(inline_start_end_properties(
            direction,
            "scroll-padding-left",
            "scroll-padding-right",
        )),
        "border-inline-color" => Some(inline_start_end_properties(
            direction,
            "border-left-color",
            "border-right-color",
        )),
        "border-inline-style" => Some(inline_start_end_properties(
            direction,
            "border-left-style",
            "border-right-style",
        )),
        "border-inline-width" => Some(inline_start_end_properties(
            direction,
            "border-left-width",
            "border-right-width",
        )),
        _ => None,
    }
}

fn physical_pair_properties_for_logical_mirror(
    property: &str,
    direction: InlineDirection,
) -> Option<(&'static str, &'static str)> {
    match property {
        "border-inline" => Some(inline_start_end_properties(
            direction,
            "border-left",
            "border-right",
        )),
        _ => None,
    }
}

fn logical_pair_values(value: &str) -> Option<(String, String)> {
    let components = split_top_level_whitespace_value_components(value)?;
    match components.as_slice() {
        [both] => Some((both.clone(), both.clone())),
        [start, end] => Some((start.clone(), end.clone())),
        _ => None,
    }
}

fn split_top_level_whitespace_value_components(value: &str) -> Option<Vec<String>> {
    let mut components = Vec::new();
    let mut current = String::new();
    let mut depth = 0usize;
    let mut bracket_depth = 0usize;
    let mut quote: Option<char> = None;
    let mut escaped = false;

    for ch in value.chars() {
        if let Some(active_quote) = quote {
            current.push(ch);
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == active_quote {
                quote = None;
            }
            continue;
        }

        match ch {
            '"' | '\'' => {
                quote = Some(ch);
                current.push(ch);
            }
            '(' => {
                depth += 1;
                current.push(ch);
            }
            ')' => {
                depth = depth.checked_sub(1)?;
                current.push(ch);
            }
            '[' => {
                bracket_depth += 1;
                current.push(ch);
            }
            ']' => {
                bracket_depth = bracket_depth.checked_sub(1)?;
                current.push(ch);
            }
            ch if ch.is_ascii_whitespace() && depth == 0 && bracket_depth == 0 => {
                if !current.trim().is_empty() {
                    components.push(current.trim().to_string());
                    current.clear();
                }
            }
            _ => current.push(ch),
        }
    }

    if quote.is_some() || depth != 0 || bracket_depth != 0 {
        return None;
    }
    if !current.trim().is_empty() {
        components.push(current.trim().to_string());
    }
    (!components.is_empty()).then_some(components)
}

fn inline_start_end_properties(
    direction: InlineDirection,
    ltr_start_property: &'static str,
    ltr_end_property: &'static str,
) -> (&'static str, &'static str) {
    match direction {
        InlineDirection::Ltr => (ltr_start_property, ltr_end_property),
        InlineDirection::Rtl => (ltr_end_property, ltr_start_property),
    }
}

fn inline_start_property(
    direction: InlineDirection,
    ltr_property: &'static str,
    rtl_property: &'static str,
) -> &'static str {
    match direction {
        InlineDirection::Ltr => ltr_property,
        InlineDirection::Rtl => rtl_property,
    }
}

fn inline_end_property(
    direction: InlineDirection,
    ltr_property: &'static str,
    rtl_property: &'static str,
) -> &'static str {
    match direction {
        InlineDirection::Ltr => rtl_property,
        InlineDirection::Rtl => ltr_property,
    }
}

fn display_p3_to_srgb(red: f64, green: f64, blue: f64) -> Option<SrgbColor> {
    let red_linear = decode_srgb_channel(red);
    let green_linear = decode_srgb_channel(green);
    let blue_linear = decode_srgb_channel(blue);

    let x = 0.486_570_948_648_216_2 * red_linear
        + 0.265_667_693_169_093_1 * green_linear
        + 0.198_217_285_234_362_5 * blue_linear;
    let y = 0.228_974_564_069_748_8 * red_linear
        + 0.691_738_521_836_506_4 * green_linear
        + 0.079_286_914_093_745 * blue_linear;
    let z = 0.045_113_381_858_902_6 * green_linear + 1.043_944_368_900_976 * blue_linear;

    let red_linear_srgb =
        3.240_969_941_904_522_6 * x - 1.537_383_177_570_094 * y - 0.498_610_760_293_003_4 * z;
    let green_linear_srgb =
        -0.969_243_636_280_879_6 * x + 1.875_967_501_507_720_2 * y + 0.041_555_057_407_175_59 * z;
    let blue_linear_srgb =
        0.055_630_079_696_993_66 * x - 0.203_976_958_888_976_52 * y + 1.056_971_514_242_878_6 * z;

    if !is_in_gamut_linear_srgb(red_linear_srgb)
        || !is_in_gamut_linear_srgb(green_linear_srgb)
        || !is_in_gamut_linear_srgb(blue_linear_srgb)
    {
        return None;
    }

    Some(SrgbColor {
        red: encode_srgb_channel(red_linear_srgb),
        green: encode_srgb_channel(green_linear_srgb),
        blue: encode_srgb_channel(blue_linear_srgb),
    })
}

fn decode_srgb_channel(value: f64) -> f64 {
    if value <= 0.040_45 {
        value / 12.92
    } else {
        ((value + 0.055) / 1.055).powf(2.4)
    }
}

fn oklab_to_srgb(lightness: f64, a_axis: f64, b_axis: f64) -> Option<SrgbColor> {
    let l_prime = lightness + 0.396_337_777_4 * a_axis + 0.215_803_757_3 * b_axis;
    let m_prime = lightness - 0.105_561_345_8 * a_axis - 0.063_854_172_8 * b_axis;
    let s_prime = lightness - 0.089_484_177_5 * a_axis - 1.291_485_548_0 * b_axis;

    let l = l_prime.powi(3);
    let m = m_prime.powi(3);
    let s = s_prime.powi(3);

    let red_linear = 4.076_741_662_1 * l - 3.307_711_591_3 * m + 0.230_969_929_2 * s;
    let green_linear = -1.268_438_004_6 * l + 2.609_757_401_1 * m - 0.341_319_396_5 * s;
    let blue_linear = -0.004_196_086_3 * l - 0.703_418_614_7 * m + 1.707_614_701_0 * s;

    if !is_in_gamut_linear_srgb(red_linear)
        || !is_in_gamut_linear_srgb(green_linear)
        || !is_in_gamut_linear_srgb(blue_linear)
    {
        return None;
    }

    Some(SrgbColor {
        red: encode_srgb_channel(red_linear),
        green: encode_srgb_channel(green_linear),
        blue: encode_srgb_channel(blue_linear),
    })
}

fn is_in_gamut_linear_srgb(value: f64) -> bool {
    (-0.000_001..=1.000_001).contains(&value)
}

fn encode_srgb_channel(value: f64) -> u8 {
    let clamped = value.clamp(0.0, 1.0);
    let encoded = if clamped <= 0.003_130_8 {
        12.92 * clamped
    } else {
        1.055 * clamped.powf(1.0 / 2.4) - 0.055
    };
    (encoded * 255.0).round().clamp(0.0, 255.0) as u8
}

fn add_css_vendor_prefixes_with_lexer(source: &str, dialect: StyleDialect) -> (String, usize) {
    let lexed = lex(source, dialect);
    let tokens = lexed.tokens();
    let mut insertions = collect_vendor_prefix_insertions(source, tokens);
    if insertions.is_empty() {
        return (source.to_string(), 0);
    }
    insertions.sort_by_key(|(position, _)| *position);

    let mut output = String::with_capacity(source.len());
    let mut cursor = 0;
    for (position, insertion) in &insertions {
        if *position > cursor {
            output.push_str(&source[cursor..*position]);
        }
        output.push_str(insertion);
        cursor = *position;
    }
    if cursor < source.len() {
        output.push_str(&source[cursor..]);
    }

    (output, insertions.len())
}

fn collect_vendor_prefix_insertions(
    source: &str,
    tokens: &[omena_parser::LexedToken],
) -> Vec<(usize, String)> {
    let mut insertions = Vec::new();
    insertions.extend(collect_keyframes_vendor_prefix_insertions(source, tokens));
    let mut index = 0;

    while index < tokens.len() {
        if tokens[index].kind == SyntaxKind::LeftBrace
            && let Some(close_index) = matching_right_brace_index(tokens, index)
        {
            let declarations = collect_simple_declarations_in_block(tokens, index, close_index);
            for declaration in &declarations {
                for prefixed_property in prefixed_properties_for(&declaration.property)
                    .iter()
                    .copied()
                {
                    if declarations
                        .iter()
                        .any(|candidate| candidate.property == prefixed_property)
                    {
                        continue;
                    }
                    insertions.push((
                        declaration.start,
                        format!("{prefixed_property}: {}; ", declaration.value),
                    ));
                }
                for prefixed_value in prefixed_values_for(&declaration.property, &declaration.value)
                {
                    if declarations.iter().any(|candidate| {
                        candidate.property == declaration.property
                            && candidate.value.eq_ignore_ascii_case(prefixed_value)
                    }) {
                        continue;
                    }
                    insertions.push((
                        declaration.start,
                        format!("{}: {prefixed_value}; ", declaration.property),
                    ));
                }
            }
            index = close_index + 1;
            continue;
        }
        index += 1;
    }

    insertions
}

fn collect_keyframes_vendor_prefix_insertions(
    source: &str,
    tokens: &[omena_parser::LexedToken],
) -> Vec<(usize, String)> {
    let prefixed_names = collect_keyframes_names(tokens, "@-webkit-keyframes");
    let mut insertions = Vec::new();
    let mut index = 0;

    while index < tokens.len() {
        if tokens[index].kind == SyntaxKind::AtKeyword
            && tokens[index].text.eq_ignore_ascii_case("@keyframes")
            && let Some(name) = keyframes_name_after(tokens, index)
            && !prefixed_names
                .iter()
                .any(|prefixed_name| prefixed_name == &name.to_ascii_lowercase())
            && let Some(block_start) = at_rule_block_start(tokens, index + 1)
            && let Some(block_end) = matching_right_brace_index(tokens, block_start)
        {
            let start = token_start(&tokens[index]);
            let end = token_end(&tokens[block_end]);
            let original = &source[start..end];
            let prefixed = original.replacen(&tokens[index].text, "@-webkit-keyframes", 1);
            insertions.push((start, format!("{prefixed} ")));
            index = block_end + 1;
            continue;
        }
        index += 1;
    }

    insertions
}

fn collect_keyframes_names(tokens: &[omena_parser::LexedToken], at_keyword: &str) -> Vec<String> {
    let mut names = Vec::new();
    let mut index = 0;
    while index < tokens.len() {
        if tokens[index].kind == SyntaxKind::AtKeyword
            && tokens[index].text.eq_ignore_ascii_case(at_keyword)
            && let Some(name) = keyframes_name_after(tokens, index)
        {
            names.push(name.to_ascii_lowercase());
        }
        index += 1;
    }
    names
}

fn keyframes_name_after(
    tokens: &[omena_parser::LexedToken],
    at_keyword_index: usize,
) -> Option<&str> {
    let name_index = skip_whitespace_tokens(tokens, at_keyword_index + 1, tokens.len());
    let name_token = tokens.get(name_index)?;
    matches!(name_token.kind, SyntaxKind::Ident | SyntaxKind::String)
        .then_some(name_token.text.as_str())
}

fn at_rule_block_start(tokens: &[omena_parser::LexedToken], start_index: usize) -> Option<usize> {
    let mut index = start_index;
    while index < tokens.len() {
        match tokens[index].kind {
            SyntaxKind::LeftBrace => return Some(index),
            SyntaxKind::RightBrace | SyntaxKind::Semicolon => return None,
            _ => index += 1,
        }
    }
    None
}

fn prefixed_properties_for(property: &str) -> &'static [&'static str] {
    match property {
        "appearance" => &["-webkit-appearance", "-moz-appearance"],
        "backdrop-filter" => &["-webkit-backdrop-filter"],
        "hyphens" => &["-webkit-hyphens", "-ms-hyphens"],
        "mask-clip" => &["-webkit-mask-clip"],
        "mask-composite" => &["-webkit-mask-composite"],
        "mask-image" => &["-webkit-mask-image"],
        "mask-mode" => &["-webkit-mask-mode"],
        "mask-origin" => &["-webkit-mask-origin"],
        "mask-position" => &["-webkit-mask-position"],
        "mask-repeat" => &["-webkit-mask-repeat"],
        "mask-size" => &["-webkit-mask-size"],
        "print-color-adjust" => &["-webkit-print-color-adjust"],
        "text-size-adjust" => &["-webkit-text-size-adjust"],
        "user-select" => &["-webkit-user-select", "-moz-user-select", "-ms-user-select"],
        _ => &[],
    }
}

fn prefixed_values_for(property: &str, value: &str) -> Vec<&'static str> {
    match (property, value.trim().to_ascii_lowercase().as_str()) {
        ("display", "flex") => vec!["-webkit-box", "-ms-flexbox"],
        ("display", "inline-flex") => vec!["-webkit-inline-box", "-ms-inline-flexbox"],
        ("position", "sticky") => vec!["-webkit-sticky"],
        _ => Vec::new(),
    }
}

fn merge_adjacent_same_block_css_selectors_with_lexer(
    source: &str,
    dialect: StyleDialect,
) -> (String, usize) {
    let lexed = lex(source, dialect);
    let tokens = lexed.tokens();
    let rules = collect_declaration_ordinary_rule_slices(source, tokens);
    let mut replacements = Vec::new();
    let mut index = 0;

    while index < rules.len() {
        let current = &rules[index];
        let mut selectors = vec![current.selector.clone()];
        let mut run_end = index + 1;

        while run_end < rules.len() {
            let previous = &rules[run_end - 1];
            let next = &rules[run_end];
            if current.block != next.block
                || !rule_gap_is_whitespace_only(tokens, previous.end, next.start)
            {
                break;
            }
            selectors.push(next.selector.clone());
            run_end += 1;
        }

        let deduped_selectors = dedupe_selector_arguments(&selectors);
        if deduped_selectors.len() > 1 {
            let last = &rules[run_end - 1];
            replacements.push((
                current.start,
                last.end,
                format!(
                    "{}, {} {{ {} }}",
                    deduped_selectors[0],
                    deduped_selectors[1..].join(", "),
                    current.block
                ),
            ));
        } else {
            index += 1;
            continue;
        }

        index = run_end;
    }

    if replacements.is_empty() {
        return (source.to_string(), 0);
    }

    let mut output = String::with_capacity(source.len());
    let mut cursor = 0;
    for (start, end, replacement) in &replacements {
        if *start > cursor {
            output.push_str(&source[cursor..*start]);
        }
        output.push_str(replacement);
        cursor = *end;
    }
    if cursor < source.len() {
        output.push_str(&source[cursor..]);
    }

    (output, replacements.len())
}

fn merge_adjacent_same_selector_css_rules_with_lexer(
    source: &str,
    dialect: StyleDialect,
) -> (String, usize) {
    let lexed = lex(source, dialect);
    let tokens = lexed.tokens();
    let rules = collect_declaration_ordinary_rule_slices(source, tokens);
    let mut replacements = Vec::new();
    let mut index = 0;

    while index < rules.len() {
        let current = &rules[index];
        let mut blocks = vec![current.block.clone()];
        let mut run_end = index + 1;

        while run_end < rules.len() {
            let previous = &rules[run_end - 1];
            let next = &rules[run_end];
            if current.selector != next.selector
                || !rule_gap_is_whitespace_only(tokens, previous.end, next.start)
            {
                break;
            }
            blocks.push(next.block.clone());
            run_end += 1;
        }

        if blocks.len() > 1 && blocks.iter().any(|block| block != &blocks[0]) {
            let last = &rules[run_end - 1];
            replacements.push((
                current.start,
                last.end,
                format!("{} {{ {} }}", current.selector, blocks.join(" ")),
            ));
        } else {
            index += 1;
            continue;
        }

        index = run_end;
    }

    if replacements.is_empty() {
        return (source.to_string(), 0);
    }

    let mut output = String::with_capacity(source.len());
    let mut cursor = 0;
    for (start, end, replacement) in &replacements {
        if *start > cursor {
            output.push_str(&source[cursor..*start]);
        }
        output.push_str(replacement);
        cursor = *end;
    }
    if cursor < source.len() {
        output.push_str(&source[cursor..]);
    }

    (output, replacements.len())
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SimpleRuleSlice {
    selector: String,
    block: String,
    start: usize,
    end: usize,
    block_start: usize,
    block_end: usize,
    context_start: usize,
    context_end: usize,
}

fn dedupe_exact_css_rules_with_lexer(source: &str, dialect: StyleDialect) -> (String, usize) {
    let lexed = lex(source, dialect);
    let tokens = lexed.tokens();
    let rules = collect_declaration_ordinary_rule_slices(source, tokens);
    let ranges = collect_duplicate_ordinary_rule_ranges(&rules);

    if ranges.is_empty() {
        return (source.to_string(), 0);
    }

    remove_source_ranges(source, &ranges)
}

fn collect_duplicate_ordinary_rule_ranges(rules: &[SimpleRuleSlice]) -> Vec<(usize, usize)> {
    let mut ranges = Vec::new();

    for (index, rule) in rules.iter().enumerate() {
        let has_later_duplicate = rules[index + 1..].iter().any(|candidate| {
            rule.selector == candidate.selector
                && rule.block == candidate.block
                && rule.context_start == candidate.context_start
                && rule.context_end == candidate.context_end
        });
        if has_later_duplicate {
            ranges.push((rule.start, rule.end));
        }
    }

    ranges
}

fn collect_top_level_ordinary_rule_slices(
    source: &str,
    tokens: &[omena_parser::LexedToken],
) -> Vec<SimpleRuleSlice> {
    let mut rules = Vec::new();
    let mut depth = 0usize;
    let mut top_level_prelude_start = 0usize;
    let mut index = 0;

    while index < tokens.len() {
        match tokens[index].kind {
            SyntaxKind::LeftBrace => {
                if depth == 0
                    && let Some(close_index) = matching_right_brace_index(tokens, index)
                    && is_ordinary_top_level_rule_prelude(tokens, top_level_prelude_start, index)
                    && !tokens[index + 1..close_index].iter().any(|token| {
                        matches!(token.kind, SyntaxKind::LeftBrace | SyntaxKind::RightBrace)
                            || is_comment_token(token.kind)
                    })
                    && let Some(start) =
                        first_non_trivia_token_start(tokens, top_level_prelude_start, index)
                {
                    let selector = source[start..token_start(&tokens[index])]
                        .trim()
                        .to_string();
                    let block = source
                        [token_end(&tokens[index])..token_start(&tokens[close_index])]
                        .trim()
                        .to_string();
                    if !selector.is_empty() && !block.is_empty() {
                        rules.push(SimpleRuleSlice {
                            selector,
                            block,
                            start,
                            end: token_end(&tokens[close_index]),
                            block_start: token_start(&tokens[index]),
                            block_end: token_start(&tokens[close_index]),
                            context_start: 0,
                            context_end: source.len(),
                        });
                    }
                    index = close_index + 1;
                    top_level_prelude_start = index;
                    continue;
                }
                depth += 1;
            }
            SyntaxKind::RightBrace => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    top_level_prelude_start = index + 1;
                }
            }
            SyntaxKind::Semicolon if depth == 0 => {
                top_level_prelude_start = index + 1;
            }
            _ => {}
        }
        index += 1;
    }

    rules
}

fn collect_declaration_ordinary_rule_slices(
    source: &str,
    tokens: &[omena_parser::LexedToken],
) -> Vec<SimpleRuleSlice> {
    let mut rules = Vec::new();
    let mut depth = 0usize;
    let mut prelude_starts = vec![0usize];
    let mut rule_contexts = vec![(0usize, source.len())];
    let mut index = 0;

    while index < tokens.len() {
        match tokens[index].kind {
            SyntaxKind::LeftBrace => {
                let prelude_start = prelude_starts.get(depth).copied().unwrap_or(0);
                let parent_context = rule_contexts
                    .get(depth)
                    .copied()
                    .unwrap_or((0, source.len()));
                if let Some(close_index) = matching_right_brace_index(tokens, index)
                    && is_ordinary_rule_prelude(tokens, prelude_start, index)
                    && !tokens[index + 1..close_index].iter().any(|token| {
                        matches!(token.kind, SyntaxKind::LeftBrace | SyntaxKind::RightBrace)
                            || is_comment_token(token.kind)
                    })
                    && let Some(start) = first_non_trivia_token_start(tokens, prelude_start, index)
                {
                    let selector = source[start..token_start(&tokens[index])]
                        .trim()
                        .to_string();
                    let block = source
                        [token_end(&tokens[index])..token_start(&tokens[close_index])]
                        .trim()
                        .to_string();
                    if !selector.is_empty() && !block.is_empty() {
                        rules.push(SimpleRuleSlice {
                            selector,
                            block,
                            start,
                            end: token_end(&tokens[close_index]),
                            block_start: token_start(&tokens[index]),
                            block_end: token_start(&tokens[close_index]),
                            context_start: parent_context.0,
                            context_end: parent_context.1,
                        });
                    }
                }
                let child_context = matching_right_brace_index(tokens, index)
                    .map(|close_index| {
                        (token_start(&tokens[index]), token_end(&tokens[close_index]))
                    })
                    .unwrap_or((token_start(&tokens[index]), token_end(&tokens[index])));
                depth += 1;
                set_prelude_start(&mut prelude_starts, depth, index + 1);
                set_rule_context(&mut rule_contexts, depth, child_context);
            }
            SyntaxKind::RightBrace => {
                depth = depth.saturating_sub(1);
                set_prelude_start(&mut prelude_starts, depth, index + 1);
            }
            SyntaxKind::Semicolon => {
                set_prelude_start(&mut prelude_starts, depth, index + 1);
            }
            _ => {}
        }
        index += 1;
    }

    rules
}

fn rule_gap_is_whitespace_only(
    tokens: &[omena_parser::LexedToken],
    start: usize,
    end: usize,
) -> bool {
    tokens_between_byte_range(tokens, start, end)
        .iter()
        .all(|token| token.kind == SyntaxKind::Whitespace)
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SimpleDeclarationSlice {
    property: String,
    value: String,
    important: bool,
    start: usize,
    end: usize,
    source_order: u32,
}

fn combine_css_box_shorthands_with_lexer(source: &str, dialect: StyleDialect) -> (String, usize) {
    let lexed = lex(source, dialect);
    let tokens = lexed.tokens();
    let ranges = collect_box_shorthand_replacement_ranges(tokens);
    if ranges.is_empty() {
        return (source.to_string(), 0);
    }

    let mut output = String::with_capacity(source.len());
    let mut cursor = 0;
    for (start, end, replacement) in &ranges {
        if *start > cursor {
            output.push_str(&source[cursor..*start]);
        }
        output.push_str(replacement);
        cursor = *end;
    }
    if cursor < source.len() {
        output.push_str(&source[cursor..]);
    }

    (output, ranges.len())
}

fn collect_box_shorthand_replacement_ranges(
    tokens: &[omena_parser::LexedToken],
) -> Vec<(usize, usize, String)> {
    let mut ranges = Vec::new();
    let mut index = 0;
    while index < tokens.len() {
        if tokens[index].kind == SyntaxKind::LeftBrace
            && let Some(close_index) = matching_right_brace_index(tokens, index)
        {
            ranges.extend(collect_box_shorthand_replacements_in_block(
                tokens,
                index,
                close_index,
            ));
            index = close_index + 1;
            continue;
        }
        index += 1;
    }
    ranges
}

fn collect_box_shorthand_replacements_in_block(
    tokens: &[omena_parser::LexedToken],
    block_start: usize,
    block_end: usize,
) -> Vec<(usize, usize, String)> {
    let declarations = collect_simple_declarations_in_block(tokens, block_start, block_end);
    let mut ranges = Vec::new();
    let mut index = 0;
    while index + 3 < declarations.len() {
        if let Some((start, end, replacement)) =
            box_shorthand_replacement_for_declarations(tokens, &declarations[index..index + 4])
        {
            ranges.push((start, end, replacement));
            index += 4;
        } else {
            index += 1;
        }
    }
    ranges
}

fn collect_simple_declarations_in_block(
    tokens: &[omena_parser::LexedToken],
    block_start: usize,
    block_end: usize,
) -> Vec<SimpleDeclarationSlice> {
    let mut declarations = Vec::new();
    let mut index = block_start + 1;
    let mut source_order = 0u32;

    while index < block_end {
        index = skip_whitespace_tokens(tokens, index, block_end);
        if index >= block_end {
            break;
        }

        if tokens[index].kind == SyntaxKind::LeftBrace
            && let Some(close_index) = matching_right_brace_index(tokens, index)
        {
            index = close_index + 1;
            continue;
        }

        if let Some((declaration, next_index)) =
            parse_simple_declaration_slice(tokens, index, block_end, source_order)
        {
            declarations.push(declaration);
            source_order += 1;
            index = next_index;
        } else {
            index += 1;
        }
    }

    declarations
}

fn parse_simple_declaration_slice(
    tokens: &[omena_parser::LexedToken],
    start_index: usize,
    block_end: usize,
    source_order: u32,
) -> Option<(SimpleDeclarationSlice, usize)> {
    let property_token = tokens.get(start_index)?;
    let property = match property_token.kind {
        SyntaxKind::Ident => property_token.text.to_ascii_lowercase(),
        SyntaxKind::CustomPropertyName => property_token.text.clone(),
        _ => return None,
    };

    let colon_index = skip_whitespace_tokens(tokens, start_index + 1, block_end);
    if tokens.get(colon_index)?.kind != SyntaxKind::Colon {
        return None;
    }

    let mut value_tokens: Vec<&omena_parser::LexedToken> = Vec::new();
    let mut index = colon_index + 1;
    while index < block_end {
        match tokens[index].kind {
            SyntaxKind::Semicolon => {
                if value_tokens
                    .iter()
                    .any(|token| is_comment_token(token.kind))
                {
                    return None;
                }
                let value = value_tokens
                    .iter()
                    .map(|token| token.text.as_str())
                    .collect::<String>()
                    .trim()
                    .to_string();
                if value.is_empty() {
                    return None;
                }
                let important = value_tokens
                    .iter()
                    .any(|token| token.kind == SyntaxKind::Important);
                return Some((
                    SimpleDeclarationSlice {
                        property,
                        value,
                        important,
                        start: token_start(property_token),
                        end: token_end(&tokens[index]),
                        source_order,
                    },
                    index + 1,
                ));
            }
            SyntaxKind::LeftBrace | SyntaxKind::RightBrace => return None,
            _ => value_tokens.push(&tokens[index]),
        }
        index += 1;
    }

    None
}

fn box_shorthand_replacement_for_declarations(
    tokens: &[omena_parser::LexedToken],
    declarations: &[SimpleDeclarationSlice],
) -> Option<(usize, usize, String)> {
    let shorthand_property = match declarations.first()?.property.as_str() {
        "margin-top" => "margin",
        "padding-top" => "padding",
        _ => return None,
    };
    if !declaration_ranges_are_adjacent(tokens, declarations) {
        return None;
    }

    let proof_inputs = declarations
        .iter()
        .map(|declaration| BoxLonghandInputV0 {
            property: declaration.property.clone(),
            value: declaration.value.clone(),
            important: declaration.important,
            source_order: declaration.source_order,
        })
        .collect::<Vec<_>>();
    let proof = prove_box_shorthand_combination(shorthand_property, &proof_inputs);
    if !proof.accepted {
        return None;
    }

    let values = declarations
        .iter()
        .map(|declaration| declaration.value.as_str())
        .collect::<Vec<_>>();
    let shorthand_value = compress_box_shorthand_values(&values)?;
    let replacement = format!("{shorthand_property}: {shorthand_value};");
    Some((
        declarations.first()?.start,
        declarations.last()?.end,
        replacement,
    ))
}

fn declaration_ranges_are_adjacent(
    tokens: &[omena_parser::LexedToken],
    declarations: &[SimpleDeclarationSlice],
) -> bool {
    declarations.windows(2).all(|pair| {
        tokens_between_byte_range(tokens, pair[0].end, pair[1].start)
            .iter()
            .all(|token| token.kind == SyntaxKind::Whitespace)
    })
}

fn tokens_between_byte_range(
    tokens: &[omena_parser::LexedToken],
    start: usize,
    end: usize,
) -> Vec<&omena_parser::LexedToken> {
    tokens
        .iter()
        .filter(|token| token_start(token) >= start && token_end(token) <= end)
        .collect()
}

fn compress_box_shorthand_values(values: &[&str]) -> Option<String> {
    let [top, right, bottom, left] = values else {
        return None;
    };

    let parts = if top == right && top == bottom && top == left {
        vec![*top]
    } else if top == bottom && right == left {
        vec![*top, *right]
    } else if right == left {
        vec![*top, *right, *bottom]
    } else {
        vec![*top, *right, *bottom, *left]
    };
    Some(parts.join(" "))
}

fn skip_whitespace_tokens(
    tokens: &[omena_parser::LexedToken],
    mut index: usize,
    end_exclusive: usize,
) -> usize {
    while index < end_exclusive && tokens[index].kind == SyntaxKind::Whitespace {
        index += 1;
    }
    index
}

fn remove_empty_css_rules_with_lexer(source: &str, dialect: StyleDialect) -> (String, usize) {
    let mut output = source.to_string();
    let mut mutation_count = 0;

    loop {
        let lexed = lex(&output, dialect);
        let tokens = lexed.tokens();
        let ranges = collect_empty_rule_ranges(tokens);
        let (next_output, removed_count) = remove_source_ranges(&output, &ranges);
        if removed_count == 0 {
            return (output, mutation_count);
        }
        output = next_output;
        mutation_count += removed_count;
    }
}

fn collect_empty_rule_ranges(tokens: &[omena_parser::LexedToken]) -> Vec<(usize, usize)> {
    let mut ranges = Vec::new();
    let mut depth = 0usize;
    let mut prelude_starts = vec![0usize];
    let mut index = 0;

    while index < tokens.len() {
        match tokens[index].kind {
            SyntaxKind::LeftBrace => {
                let prelude_start = prelude_starts.get(depth).copied().unwrap_or(0);
                if let Some(close_index) = matching_right_brace_index(tokens, index)
                    && is_empty_rule_block(tokens, index + 1, close_index)
                    && (is_ordinary_rule_prelude(tokens, prelude_start, index)
                        || is_empty_group_rule_prelude(tokens, prelude_start, index))
                    && let Some(start) = first_non_trivia_token_start(tokens, prelude_start, index)
                {
                    let end = token_end(&tokens[close_index]);
                    ranges.push((start, end));
                    index = close_index + 1;
                    set_prelude_start(&mut prelude_starts, depth, index);
                    continue;
                }
                depth += 1;
                set_prelude_start(&mut prelude_starts, depth, index + 1);
            }
            SyntaxKind::RightBrace => {
                depth = depth.saturating_sub(1);
                set_prelude_start(&mut prelude_starts, depth, index + 1);
            }
            SyntaxKind::Semicolon => {
                set_prelude_start(&mut prelude_starts, depth, index + 1);
            }
            _ => {}
        }
        index += 1;
    }

    ranges
}

fn set_prelude_start(prelude_starts: &mut Vec<usize>, depth: usize, start: usize) {
    if prelude_starts.len() <= depth {
        prelude_starts.resize(depth + 1, start);
    }
    prelude_starts[depth] = start;
}

fn set_rule_context(
    rule_contexts: &mut Vec<(usize, usize)>,
    depth: usize,
    context: (usize, usize),
) {
    if rule_contexts.len() <= depth {
        rule_contexts.resize(depth + 1, context);
    }
    rule_contexts[depth] = context;
}

fn matching_right_brace_index(
    tokens: &[omena_parser::LexedToken],
    left_brace_index: usize,
) -> Option<usize> {
    let mut depth = 0usize;
    for (index, token) in tokens.iter().enumerate().skip(left_brace_index) {
        match token.kind {
            SyntaxKind::LeftBrace => depth += 1,
            SyntaxKind::RightBrace => {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    return Some(index);
                }
            }
            _ => {}
        }
    }
    None
}

fn is_empty_rule_block(
    tokens: &[omena_parser::LexedToken],
    start: usize,
    end_exclusive: usize,
) -> bool {
    tokens[start..end_exclusive].iter().all(|token| {
        matches!(
            token.kind,
            SyntaxKind::Whitespace | SyntaxKind::SassIndentedNewline
        )
    })
}

fn is_ordinary_rule_prelude(
    tokens: &[omena_parser::LexedToken],
    start: usize,
    end_exclusive: usize,
) -> bool {
    let prelude = &tokens[start..end_exclusive];
    prelude
        .iter()
        .any(|token| !is_comment_token(token.kind) && token.kind != SyntaxKind::Whitespace)
        && prelude
            .iter()
            .all(|token| token.kind != SyntaxKind::AtKeyword && !is_comment_token(token.kind))
}

fn is_empty_group_rule_prelude(
    tokens: &[omena_parser::LexedToken],
    start: usize,
    end_exclusive: usize,
) -> bool {
    let prelude = &tokens[start..end_exclusive];
    let mut significant_tokens = prelude
        .iter()
        .filter(|token| !is_comment_token(token.kind) && token.kind != SyntaxKind::Whitespace);
    let Some(first) = significant_tokens.next() else {
        return false;
    };
    first.kind == SyntaxKind::AtKeyword && is_empty_removable_group_at_keyword(&first.text)
}

fn is_empty_removable_group_at_keyword(text: &str) -> bool {
    matches!(
        text.to_ascii_lowercase().as_str(),
        "@container" | "@layer" | "@media" | "@scope" | "@supports"
    )
}

fn is_ordinary_top_level_rule_prelude(
    tokens: &[omena_parser::LexedToken],
    start: usize,
    end_exclusive: usize,
) -> bool {
    is_ordinary_rule_prelude(tokens, start, end_exclusive)
}

fn first_non_trivia_token_start(
    tokens: &[omena_parser::LexedToken],
    start: usize,
    end_exclusive: usize,
) -> Option<usize> {
    tokens[start..end_exclusive]
        .iter()
        .find(|token| !is_comment_token(token.kind) && token.kind != SyntaxKind::Whitespace)
        .map(token_start)
}

fn token_start(token: &omena_parser::LexedToken) -> usize {
    u32::from(token.range.start()) as usize
}

fn token_end(token: &omena_parser::LexedToken) -> usize {
    u32::from(token.range.end()) as usize
}

fn compress_css_is_where_selectors_with_lexer(
    source: &str,
    dialect: StyleDialect,
) -> (String, usize) {
    let (source, function_mutation_count) =
        compress_css_is_where_functions_with_lexer(source, dialect);
    let (source, selector_list_mutation_count) =
        dedupe_ordinary_selector_lists_with_lexer(&source, dialect);

    (
        source,
        function_mutation_count + selector_list_mutation_count,
    )
}

fn compress_css_is_where_functions_with_lexer(
    source: &str,
    dialect: StyleDialect,
) -> (String, usize) {
    let lexed = lex(source, dialect);
    let tokens = lexed.tokens();
    let mut output = String::with_capacity(source.len());
    let mut mutation_count = 0;
    let mut index = 0;

    while index < tokens.len() {
        if let Some((replacement, consumed)) = rewrite_is_where_selector_function(tokens, index) {
            output.push_str(&replacement);
            mutation_count += 1;
            index += consumed;
            continue;
        }

        output.push_str(&tokens[index].text);
        index += 1;
    }

    (output, mutation_count)
}

fn dedupe_ordinary_selector_lists_with_lexer(
    source: &str,
    dialect: StyleDialect,
) -> (String, usize) {
    let lexed = lex(source, dialect);
    let tokens = lexed.tokens();
    let rules = collect_ordinary_rule_selector_slices(source, tokens);
    let mut replacements = Vec::new();

    for rule in rules {
        let Some(selectors) = split_css_selector_list(&rule.selector) else {
            continue;
        };
        let deduped = dedupe_selector_arguments(&selectors);
        if deduped.len() != selectors.len() {
            let separator = if source[rule.start..rule.block_start]
                .chars()
                .last()
                .is_some_and(char::is_whitespace)
            {
                " "
            } else {
                ""
            };
            replacements.push((
                rule.start,
                rule.block_start,
                format!("{}{separator}", deduped.join(", ")),
            ));
        }
    }

    if replacements.is_empty() {
        return (source.to_string(), 0);
    }

    let mut output = String::with_capacity(source.len());
    let mut cursor = 0;
    for (start, end, replacement) in &replacements {
        if *start > cursor {
            output.push_str(&source[cursor..*start]);
        }
        output.push_str(replacement);
        cursor = *end;
    }
    if cursor < source.len() {
        output.push_str(&source[cursor..]);
    }

    (output, replacements.len())
}

fn collect_ordinary_rule_selector_slices(
    source: &str,
    tokens: &[omena_parser::LexedToken],
) -> Vec<SimpleRuleSlice> {
    let mut rules = Vec::new();
    let mut depth = 0usize;
    let mut prelude_starts = vec![0usize];
    let mut index = 0;

    while index < tokens.len() {
        match tokens[index].kind {
            SyntaxKind::LeftBrace => {
                let prelude_start = prelude_starts.get(depth).copied().unwrap_or(0);
                if let Some(close_index) = matching_right_brace_index(tokens, index)
                    && is_ordinary_rule_prelude(tokens, prelude_start, index)
                    && let Some(start) = first_non_trivia_token_start(tokens, prelude_start, index)
                {
                    let selector = source[start..token_start(&tokens[index])]
                        .trim()
                        .to_string();
                    if !selector.is_empty() {
                        rules.push(SimpleRuleSlice {
                            selector,
                            block: source
                                [token_end(&tokens[index])..token_start(&tokens[close_index])]
                                .trim()
                                .to_string(),
                            start,
                            end: token_end(&tokens[close_index]),
                            block_start: token_start(&tokens[index]),
                            block_end: token_start(&tokens[close_index]),
                            context_start: 0,
                            context_end: source.len(),
                        });
                    }
                }
                depth += 1;
                set_prelude_start(&mut prelude_starts, depth, index + 1);
            }
            SyntaxKind::RightBrace => {
                depth = depth.saturating_sub(1);
                set_prelude_start(&mut prelude_starts, depth, index + 1);
            }
            SyntaxKind::Semicolon => {
                set_prelude_start(&mut prelude_starts, depth, index + 1);
            }
            _ => {}
        }
        index += 1;
    }

    rules
}

fn rewrite_is_where_selector_function(
    tokens: &[omena_parser::LexedToken],
    index: usize,
) -> Option<(String, usize)> {
    let colon = tokens.get(index)?;
    let ident = tokens.get(index + 1)?;
    let left_paren = tokens.get(index + 2)?;
    if colon.kind != SyntaxKind::Colon
        || ident.kind != SyntaxKind::Ident
        || left_paren.kind != SyntaxKind::LeftParen
    {
        return None;
    }

    let pseudo_name = ident.text.to_ascii_lowercase();
    if pseudo_name != "is" && pseudo_name != "where" {
        return None;
    }

    let close_index = matching_right_paren_index(tokens, index + 2)?;
    let inner_tokens = &tokens[index + 3..close_index];
    let mut arguments = split_top_level_selector_arguments(inner_tokens)?;
    if arguments.is_empty() {
        return None;
    }

    if pseudo_name == "is" {
        arguments = flatten_nested_is_selector_arguments(&arguments)?;
    } else {
        arguments = flatten_nested_where_selector_arguments(&arguments)?;
    }

    let deduped = dedupe_selector_arguments(&arguments);
    let replacement = if pseudo_name == "is" {
        if deduped.len() == 1 {
            deduped[0].clone()
        } else if deduped.len() != arguments.len() {
            format!(":is({})", deduped.join(","))
        } else {
            return None;
        }
    } else if deduped.len() != arguments.len() {
        format!(":where({})", deduped.join(","))
    } else {
        return None;
    };

    let original = tokens[index..=close_index]
        .iter()
        .map(|token| token.text.as_str())
        .collect::<String>();
    (replacement != original).then_some((replacement, close_index - index + 1))
}

fn flatten_nested_is_selector_arguments(arguments: &[String]) -> Option<Vec<String>> {
    let mut flattened = Vec::new();
    for argument in arguments {
        if let Some(inner_arguments) = parse_exact_selector_function_argument(argument, "is")? {
            flattened.extend(inner_arguments);
        } else {
            flattened.push(argument.clone());
        }
    }
    Some(flattened)
}

fn flatten_nested_where_selector_arguments(arguments: &[String]) -> Option<Vec<String>> {
    let mut flattened = Vec::new();
    for argument in arguments {
        if let Some(inner_arguments) = parse_exact_selector_function_argument(argument, "where")? {
            flattened.extend(inner_arguments);
        } else {
            flattened.push(argument.clone());
        }
    }
    Some(flattened)
}

fn parse_exact_selector_function_argument(
    argument: &str,
    function_name: &str,
) -> Option<Option<Vec<String>>> {
    let trimmed = argument.trim();
    let lexed = lex(trimmed, StyleDialect::Css);
    let tokens = lexed.tokens();
    if tokens.len() < 4 {
        return Some(None);
    }

    let colon = tokens.first()?;
    let ident = tokens.get(1)?;
    let left_paren = tokens.get(2)?;
    if colon.kind != SyntaxKind::Colon
        || ident.kind != SyntaxKind::Ident
        || !ident.text.eq_ignore_ascii_case(function_name)
        || left_paren.kind != SyntaxKind::LeftParen
    {
        return Some(None);
    }

    let close_index = matching_right_paren_index(tokens, 2)?;
    if close_index != tokens.len() - 1 {
        return Some(None);
    }

    split_top_level_selector_arguments(&tokens[3..close_index]).map(Some)
}

fn matching_right_paren_index(
    tokens: &[omena_parser::LexedToken],
    left_paren_index: usize,
) -> Option<usize> {
    let mut depth = 0usize;
    for (index, token) in tokens.iter().enumerate().skip(left_paren_index) {
        match token.kind {
            SyntaxKind::LeftParen => depth += 1,
            SyntaxKind::RightParen => {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    return Some(index);
                }
            }
            _ => {}
        }
    }
    None
}

fn split_top_level_selector_arguments(tokens: &[omena_parser::LexedToken]) -> Option<Vec<String>> {
    let mut arguments = Vec::new();
    let mut current = String::new();
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;

    for token in tokens {
        match token.kind {
            SyntaxKind::LeftParen => {
                paren_depth += 1;
                current.push_str(&token.text);
            }
            SyntaxKind::RightParen => {
                paren_depth = paren_depth.checked_sub(1)?;
                current.push_str(&token.text);
            }
            SyntaxKind::LeftBracket => {
                bracket_depth += 1;
                current.push_str(&token.text);
            }
            SyntaxKind::RightBracket => {
                bracket_depth = bracket_depth.checked_sub(1)?;
                current.push_str(&token.text);
            }
            SyntaxKind::Comma if paren_depth == 0 && bracket_depth == 0 => {
                let argument = current.trim().to_string();
                if argument.is_empty() {
                    return None;
                }
                arguments.push(argument);
                current.clear();
            }
            _ => current.push_str(&token.text),
        }
    }

    let argument = current.trim().to_string();
    if argument.is_empty() {
        return None;
    }
    arguments.push(argument);
    Some(arguments)
}

fn dedupe_selector_arguments(arguments: &[String]) -> Vec<String> {
    let mut deduped = Vec::new();
    for argument in arguments {
        if !deduped.contains(argument) {
            deduped.push(argument.clone());
        }
    }
    deduped
}

fn normalize_css_string_quotes_with_lexer(source: &str, dialect: StyleDialect) -> (String, usize) {
    rewrite_lexer_tokens(source, dialect, |kind, text| {
        if kind == SyntaxKind::String {
            return normalize_css_string_token_quotes(text);
        }
        None
    })
}

fn normalize_css_string_token_quotes(text: &str) -> Option<String> {
    if !text.starts_with('\'') || !text.ends_with('\'') || text.len() < 2 {
        return None;
    }
    let inner = &text[1..text.len() - 1];
    if inner
        .chars()
        .any(|ch| matches!(ch, '"' | '\\' | '\n' | '\r'))
    {
        return None;
    }

    Some(format!("\"{inner}\""))
}

fn strip_css_url_quotes_with_lexer(source: &str, dialect: StyleDialect) -> (String, usize) {
    let lexed = lex(source, dialect);
    let tokens = lexed.tokens();
    let mut output = String::with_capacity(source.len());
    let mut index = 0;
    let mut mutation_count = 0;

    while index < tokens.len() {
        if let Some((replacement, consumed)) = rewrite_safe_quoted_url(tokens, index) {
            output.push_str(&replacement);
            mutation_count += 1;
            index += consumed;
            continue;
        }

        output.push_str(&tokens[index].text);
        index += 1;
    }

    (output, mutation_count)
}

fn rewrite_safe_quoted_url(
    tokens: &[omena_parser::LexedToken],
    index: usize,
) -> Option<(String, usize)> {
    let ident = tokens.get(index)?;
    let left_paren = tokens.get(index + 1)?;
    let string = tokens.get(index + 2)?;
    let right_paren = tokens.get(index + 3)?;

    if ident.kind != SyntaxKind::Ident
        || !ident.text.eq_ignore_ascii_case("url")
        || left_paren.kind != SyntaxKind::LeftParen
        || string.kind != SyntaxKind::String
        || right_paren.kind != SyntaxKind::RightParen
    {
        return None;
    }

    let inner = unquote_safe_url_string(&string.text)?;
    Some((format!("{}({inner})", ident.text), 4))
}

fn unquote_safe_url_string(text: &str) -> Option<&str> {
    let quote = text.as_bytes().first().copied()?;
    if quote != b'\'' && quote != b'"' {
        return None;
    }
    if text.as_bytes().last().copied() != Some(quote) || text.len() < 2 {
        return None;
    }

    let inner = &text[1..text.len() - 1];
    if inner
        .chars()
        .any(|ch| ch.is_whitespace() || matches!(ch, '"' | '\'' | '(' | ')' | '\\'))
    {
        return None;
    }

    Some(inner)
}

fn compress_css_colors_with_lexer(source: &str, dialect: StyleDialect) -> (String, usize) {
    let (source, hex_mutation_count) = compress_css_hex_color_tokens_with_lexer(source, dialect);
    let (source, function_mutation_count) =
        compress_static_color_function_declaration_values_with_lexer(&source, dialect);

    (source, hex_mutation_count + function_mutation_count)
}

fn compress_css_hex_color_tokens_with_lexer(
    source: &str,
    dialect: StyleDialect,
) -> (String, usize) {
    let lexed = lex(source, dialect);
    let tokens = lexed.tokens();
    let mut output = String::with_capacity(source.len());
    let mut mutation_count = 0;
    let mut property_candidate = false;
    let mut inside_declaration_value = false;

    for token in tokens {
        if is_declaration_boundary_start(token.kind) {
            property_candidate = true;
            inside_declaration_value = false;
        } else if is_declaration_boundary_end(token.kind) {
            property_candidate = token.kind == SyntaxKind::Semicolon;
            inside_declaration_value = false;
        } else if token.kind == SyntaxKind::Colon && property_candidate {
            property_candidate = false;
            inside_declaration_value = true;
        } else if property_candidate
            && !is_comment_token(token.kind)
            && token.kind != SyntaxKind::Whitespace
            && !matches!(
                token.kind,
                SyntaxKind::Ident | SyntaxKind::CustomPropertyName
            )
        {
            property_candidate = false;
        }

        let replacement = if token.kind == SyntaxKind::Hash && inside_declaration_value {
            compress_hex_color_token_text(&token.text)
        } else {
            None
        };

        if let Some(replacement) = replacement {
            if replacement != token.text {
                mutation_count += 1;
            }
            output.push_str(&replacement);
        } else {
            output.push_str(&token.text);
        }
    }

    (output, mutation_count)
}

fn compress_static_color_function_declaration_values_with_lexer(
    source: &str,
    dialect: StyleDialect,
) -> (String, usize) {
    let lexed = lex(source, dialect);
    let tokens = lexed.tokens();
    let mut replacements = Vec::new();
    let mut index = 0;

    while index < tokens.len() {
        if tokens[index].kind == SyntaxKind::LeftBrace
            && let Some(close_index) = matching_right_brace_index(tokens, index)
        {
            let declarations = collect_simple_declarations_in_block(tokens, index, close_index);
            for declaration in declarations {
                if declaration.property.starts_with("--") || declaration.important {
                    continue;
                }
                let Some(replacement_value) = compress_static_color_value(&declaration.value)
                else {
                    continue;
                };
                replacements.push((
                    declaration.start,
                    declaration.end,
                    format!("{}: {replacement_value};", declaration.property),
                ));
            }
            index = close_index + 1;
            continue;
        }
        index += 1;
    }

    if replacements.is_empty() {
        return (source.to_string(), 0);
    }

    let mut output = String::with_capacity(source.len());
    let mut cursor = 0;
    for (start, end, replacement) in &replacements {
        if *start > cursor {
            output.push_str(&source[cursor..*start]);
        }
        output.push_str(replacement);
        cursor = *end;
    }
    if cursor < source.len() {
        output.push_str(&source[cursor..]);
    }

    (output, replacements.len())
}

fn compress_static_color_value(value: &str) -> Option<String> {
    let color = parse_static_srgb_color(value)
        .or_else(|| parse_static_rgb_function_color(value))
        .or_else(|| parse_static_hsl_function_color(value))
        .or_else(|| parse_static_hwb_function_color(value))?;
    let replacement = shortest_static_srgb_color_text(color);
    (replacement.len() < value.trim().len()).then_some(replacement)
}

fn parse_static_rgb_function_color(value: &str) -> Option<SrgbColor> {
    let inner = parse_whole_function_value_inner(value, "rgb")?;
    let parts = split_static_color_channels_with_optional_opaque_alpha(inner)?;
    let [red, green, blue] = parts.as_slice() else {
        return None;
    };

    Some(SrgbColor {
        red: parse_rgb_component_byte(red)?,
        green: parse_rgb_component_byte(green)?,
        blue: parse_rgb_component_byte(blue)?,
    })
}

fn parse_static_hsl_function_color(value: &str) -> Option<SrgbColor> {
    let inner = parse_whole_function_value_inner(value, "hsl")?;
    let parts = split_static_color_channels_with_optional_opaque_alpha(inner)?;
    let [hue, saturation, lightness] = parts.as_slice() else {
        return None;
    };

    hsl_to_srgb(
        parse_hue_degrees(hue)?,
        parse_bounded_percentage(saturation)?,
        parse_bounded_percentage(lightness)?,
    )
}

fn parse_static_hwb_function_color(value: &str) -> Option<SrgbColor> {
    let inner = parse_whole_function_value_inner(value, "hwb")?;
    let parts = split_static_color_channels_with_optional_opaque_alpha(inner)?;
    let [hue, whiteness, blackness] = parts.as_slice() else {
        return None;
    };

    hwb_to_srgb(
        parse_hue_degrees(hue)?,
        parse_bounded_percentage(whiteness)?,
        parse_bounded_percentage(blackness)?,
    )
}

fn split_static_color_channels_with_optional_opaque_alpha(inner: &str) -> Option<Vec<String>> {
    if inner.contains(',') {
        if inner.contains('/') {
            return None;
        }
        return split_top_level_value_arguments(inner);
    }

    let parts = inner.split_whitespace().collect::<Vec<_>>();
    match parts.as_slice() {
        [first, second, third] => Some(vec![
            (*first).to_string(),
            (*second).to_string(),
            (*third).to_string(),
        ]),
        [first, second, third, "/", alpha] if parse_opaque_alpha(alpha)? => Some(vec![
            (*first).to_string(),
            (*second).to_string(),
            (*third).to_string(),
        ]),
        _ => None,
    }
}

fn parse_bounded_percentage(text: &str) -> Option<f64> {
    let value = parse_plain_f64(text.trim().strip_suffix('%')?)?;
    if !(0.0..=100.0).contains(&value) {
        return None;
    }
    Some(value / 100.0)
}

fn hwb_to_srgb(hue_degrees: f64, whiteness: f64, blackness: f64) -> Option<SrgbColor> {
    if !hue_degrees.is_finite() || !whiteness.is_finite() || !blackness.is_finite() {
        return None;
    }

    if whiteness + blackness >= 1.0 {
        let gray = whiteness / (whiteness + blackness);
        return Some(SrgbColor {
            red: encode_css_rgb_component(gray),
            green: encode_css_rgb_component(gray),
            blue: encode_css_rgb_component(gray),
        });
    }

    let pure = hsl_to_srgb(hue_degrees, 1.0, 0.5)?;
    let scale = 1.0 - whiteness - blackness;
    Some(SrgbColor {
        red: mix_hwb_channel(pure.red, scale, whiteness),
        green: mix_hwb_channel(pure.green, scale, whiteness),
        blue: mix_hwb_channel(pure.blue, scale, whiteness),
    })
}

fn mix_hwb_channel(channel: u8, scale: f64, whiteness: f64) -> u8 {
    ((f64::from(channel) / 255.0) * scale + whiteness)
        .mul_add(255.0, 0.0)
        .round()
        .clamp(0.0, 255.0) as u8
}

fn hsl_to_srgb(hue_degrees: f64, saturation: f64, lightness: f64) -> Option<SrgbColor> {
    if !hue_degrees.is_finite() || !saturation.is_finite() || !lightness.is_finite() {
        return None;
    }

    let hue = hue_degrees.rem_euclid(360.0);
    let chroma = (1.0 - (2.0 * lightness - 1.0).abs()) * saturation;
    let hue_sector = hue / 60.0;
    let x = chroma * (1.0 - (hue_sector.rem_euclid(2.0) - 1.0).abs());
    let (red1, green1, blue1) = match hue_sector.floor() as u8 {
        0 => (chroma, x, 0.0),
        1 => (x, chroma, 0.0),
        2 => (0.0, chroma, x),
        3 => (0.0, x, chroma),
        4 => (x, 0.0, chroma),
        _ => (chroma, 0.0, x),
    };
    let offset = lightness - chroma / 2.0;

    Some(SrgbColor {
        red: encode_css_rgb_component(red1 + offset),
        green: encode_css_rgb_component(green1 + offset),
        blue: encode_css_rgb_component(blue1 + offset),
    })
}

fn encode_css_rgb_component(value: f64) -> u8 {
    (value * 255.0).round().clamp(0.0, 255.0) as u8
}

fn parse_rgb_component_byte(text: &str) -> Option<u8> {
    if let Some(percent) = text.trim().strip_suffix('%') {
        let value = parse_plain_f64(percent)?;
        if !(0.0..=100.0).contains(&value) {
            return None;
        }
        return Some(((value / 100.0) * 255.0).round().clamp(0.0, 255.0) as u8);
    }

    let value = parse_plain_f64(text.trim())?;
    if !(0.0..=255.0).contains(&value) {
        return None;
    }
    Some(value.round().clamp(0.0, 255.0) as u8)
}

fn shortest_static_srgb_color_text(color: SrgbColor) -> String {
    let hex = compressed_hex_color_for_srgb(color);
    match shortest_named_srgb_color(color) {
        Some(name) if name.len() < hex.len() => name.to_string(),
        _ => hex,
    }
}

fn shortest_named_srgb_color(color: SrgbColor) -> Option<&'static str> {
    match (color.red, color.green, color.blue) {
        (0, 0, 128) => Some("navy"),
        (0, 128, 128) => Some("teal"),
        (0, 128, 0) => Some("green"),
        (128, 0, 0) => Some("maroon"),
        (128, 0, 128) => Some("purple"),
        (128, 128, 0) => Some("olive"),
        (128, 128, 128) => Some("gray"),
        (192, 192, 192) => Some("silver"),
        (255, 0, 0) => Some("red"),
        (255, 165, 0) => Some("orange"),
        _ => None,
    }
}

fn compressed_hex_color_for_srgb(color: SrgbColor) -> String {
    let hex = format!("{:02x}{:02x}{:02x}", color.red, color.green, color.blue);
    let compressed = if can_shorten_hex_pairs(&hex) {
        shorten_hex_pairs(&hex)
    } else {
        hex
    };
    format!("#{compressed}")
}

fn normalize_css_units_with_lexer(source: &str, dialect: StyleDialect) -> (String, usize) {
    let lexed = lex(source, dialect);
    let mut output = String::with_capacity(source.len());
    let mut mutation_count = 0;
    let mut property_candidate: Option<String> = None;
    let mut active_property: Option<String> = None;
    let mut awaiting_property = false;

    for token in lexed.tokens() {
        if is_declaration_boundary_start(token.kind) {
            awaiting_property = true;
            property_candidate = None;
            active_property = None;
        } else if is_declaration_boundary_end(token.kind) {
            awaiting_property = token.kind == SyntaxKind::Semicolon;
            property_candidate = None;
            active_property = None;
        } else if token.kind == SyntaxKind::Colon && awaiting_property {
            active_property = property_candidate.clone();
            awaiting_property = false;
        } else if awaiting_property
            && !is_comment_token(token.kind)
            && token.kind != SyntaxKind::Whitespace
        {
            if matches!(
                token.kind,
                SyntaxKind::Ident | SyntaxKind::CustomPropertyName
            ) {
                property_candidate = Some(token.text.to_ascii_lowercase());
            } else {
                awaiting_property = false;
                property_candidate = None;
            }
        }

        let replacement = if token.kind == SyntaxKind::Dimension {
            active_property
                .as_deref()
                .and_then(|property| normalize_dimension_unit_token(&token.text, property))
        } else {
            None
        };

        if let Some(replacement) = replacement {
            if replacement != token.text {
                mutation_count += 1;
            }
            output.push_str(&replacement);
        } else {
            output.push_str(&token.text);
        }
    }

    (output, mutation_count)
}

fn is_declaration_boundary_start(kind: SyntaxKind) -> bool {
    matches!(kind, SyntaxKind::LeftBrace | SyntaxKind::Semicolon)
}

fn is_declaration_boundary_end(kind: SyntaxKind) -> bool {
    matches!(kind, SyntaxKind::RightBrace | SyntaxKind::Semicolon)
}

fn is_zero_length_unit_property(property: &str) -> bool {
    matches!(
        property,
        "border-block-end-width"
            | "border-block-start-width"
            | "border-block-width"
            | "border-bottom-left-radius"
            | "border-bottom-right-radius"
            | "border-bottom-width"
            | "border-end-end-radius"
            | "border-end-start-radius"
            | "border-inline-end-width"
            | "border-inline-start-width"
            | "border-inline-width"
            | "border-left-width"
            | "border-radius"
            | "border-right-width"
            | "border-start-end-radius"
            | "border-start-start-radius"
            | "border-top-left-radius"
            | "border-top-right-radius"
            | "border-top-width"
            | "border-width"
            | "margin"
            | "margin-block"
            | "margin-block-end"
            | "margin-block-start"
            | "margin-bottom"
            | "margin-inline"
            | "margin-inline-end"
            | "margin-inline-start"
            | "margin-left"
            | "margin-right"
            | "margin-top"
            | "padding"
            | "padding-block"
            | "padding-block-end"
            | "padding-block-start"
            | "padding-bottom"
            | "padding-inline"
            | "padding-inline-end"
            | "padding-inline-start"
            | "padding-left"
            | "padding-right"
            | "padding-top"
            | "inset"
            | "inset-block"
            | "inset-block-end"
            | "inset-block-start"
            | "inset-inline"
            | "inset-inline-end"
            | "inset-inline-start"
            | "top"
            | "right"
            | "bottom"
            | "left"
            | "width"
            | "min-width"
            | "max-width"
            | "height"
            | "min-height"
            | "max-height"
            | "block-size"
            | "min-block-size"
            | "max-block-size"
            | "inline-size"
            | "min-inline-size"
            | "max-inline-size"
            | "outline-width"
            | "scroll-margin"
            | "scroll-margin-block"
            | "scroll-margin-block-end"
            | "scroll-margin-block-start"
            | "scroll-margin-bottom"
            | "scroll-margin-inline"
            | "scroll-margin-inline-end"
            | "scroll-margin-inline-start"
            | "scroll-margin-left"
            | "scroll-margin-right"
            | "scroll-margin-top"
            | "scroll-padding"
            | "scroll-padding-block"
            | "scroll-padding-block-end"
            | "scroll-padding-block-start"
            | "scroll-padding-bottom"
            | "scroll-padding-inline"
            | "scroll-padding-inline-end"
            | "scroll-padding-inline-start"
            | "scroll-padding-left"
            | "scroll-padding-right"
            | "scroll-padding-top"
            | "gap"
            | "row-gap"
            | "column-gap"
    )
}

fn normalize_dimension_unit_token(text: &str, property: &str) -> Option<String> {
    if property.starts_with("--") {
        return None;
    }

    let split = numeric_prefix_end(text)?;
    let (number, unit) = text.split_at(split);
    if is_zero_length_unit_property(property)
        && is_zero_number_prefix(number)
        && is_css_length_unit(unit)
    {
        return Some("0".to_string());
    }

    normalize_known_css_unit_case(number, unit)
}

fn is_zero_number_prefix(number: &str) -> bool {
    number.parse::<f64>().is_ok_and(|value| value == 0.0)
}

fn is_css_length_unit(unit: &str) -> bool {
    matches!(
        unit.to_ascii_lowercase().as_str(),
        "cap"
            | "ch"
            | "cm"
            | "em"
            | "ex"
            | "ic"
            | "in"
            | "lh"
            | "mm"
            | "pc"
            | "pt"
            | "px"
            | "q"
            | "rem"
            | "rlh"
            | "vb"
            | "vh"
            | "vi"
            | "vmax"
            | "vmin"
            | "vw"
    )
}

fn normalize_known_css_unit_case(number: &str, unit: &str) -> Option<String> {
    let normalized_unit = unit.to_ascii_lowercase();
    if normalized_unit == unit || !is_known_css_unit(&normalized_unit) {
        return None;
    }

    Some(format!("{number}{normalized_unit}"))
}

fn is_known_css_unit(unit: &str) -> bool {
    is_css_length_unit(unit)
        || matches!(
            unit,
            "deg"
                | "grad"
                | "rad"
                | "turn"
                | "ms"
                | "s"
                | "hz"
                | "khz"
                | "dpi"
                | "dpcm"
                | "dppx"
                | "x"
                | "fr"
        )
}

fn compress_hex_color_token_text(text: &str) -> Option<String> {
    let hex = text.strip_prefix('#')?;
    if !matches!(hex.len(), 3 | 4 | 6 | 8) || !hex.chars().all(|ch| ch.is_ascii_hexdigit()) {
        return None;
    }

    let lower = hex.to_ascii_lowercase();
    let compressed = match lower.len() {
        6 if can_shorten_hex_pairs(&lower) => shorten_hex_pairs(&lower),
        8 if can_shorten_hex_pairs(&lower) => shorten_hex_pairs(&lower),
        _ => lower,
    };
    let rewritten = format!("#{compressed}");
    (rewritten != text).then_some(rewritten)
}

fn can_shorten_hex_pairs(hex: &str) -> bool {
    hex.as_bytes()
        .chunks_exact(2)
        .all(|pair| pair[0] == pair[1])
}

fn shorten_hex_pairs(hex: &str) -> String {
    hex.as_bytes()
        .chunks_exact(2)
        .map(|pair| pair[0] as char)
        .collect()
}

fn compress_css_numbers_with_lexer(source: &str, dialect: StyleDialect) -> (String, usize) {
    rewrite_lexer_tokens(source, dialect, |kind, text| {
        if matches!(
            kind,
            SyntaxKind::Number | SyntaxKind::Percentage | SyntaxKind::Dimension
        ) {
            return compress_numeric_token_text(text);
        }
        None
    })
}

fn rewrite_lexer_tokens(
    source: &str,
    dialect: StyleDialect,
    mut rewrite: impl FnMut(SyntaxKind, &str) -> Option<String>,
) -> (String, usize) {
    let lexed = lex(source, dialect);
    let mut output = String::with_capacity(source.len());
    let mut cursor = 0;
    let mut mutation_count = 0;

    for token in lexed.tokens() {
        let start = u32::from(token.range.start()) as usize;
        let end = u32::from(token.range.end()) as usize;
        if start > cursor {
            output.push_str(&source[cursor..start]);
        }
        if let Some(replacement) = rewrite(token.kind, &token.text) {
            if replacement != token.text {
                mutation_count += 1;
            }
            output.push_str(&replacement);
        } else {
            output.push_str(&source[start..end]);
        }
        cursor = end;
    }

    if cursor < source.len() {
        output.push_str(&source[cursor..]);
    }

    (output, mutation_count)
}

fn compress_numeric_token_text(text: &str) -> Option<String> {
    let split = numeric_prefix_end(text)?;
    let (number, suffix) = text.split_at(split);
    let compressed = compress_number_prefix(number);
    let rewritten = format!("{compressed}{suffix}");
    (rewritten != text).then_some(rewritten)
}

fn numeric_prefix_end(text: &str) -> Option<usize> {
    let bytes = text.as_bytes();
    let mut index = 0;

    if matches!(bytes.get(index), Some(b'+') | Some(b'-')) {
        index += 1;
    }

    let integer_start = index;
    while matches!(bytes.get(index), Some(b'0'..=b'9')) {
        index += 1;
    }
    let saw_integer_digit = index > integer_start;

    if bytes.get(index) == Some(&b'.') {
        index += 1;
        let fraction_start = index;
        while matches!(bytes.get(index), Some(b'0'..=b'9')) {
            index += 1;
        }
        if !saw_integer_digit && index == fraction_start {
            return None;
        }
    } else if !saw_integer_digit {
        return None;
    }

    if matches!(bytes.get(index), Some(b'e') | Some(b'E')) {
        let exponent_marker = index;
        let mut exponent_index = index + 1;
        if matches!(bytes.get(exponent_index), Some(b'+') | Some(b'-')) {
            exponent_index += 1;
        }
        let exponent_digit_start = exponent_index;
        while matches!(bytes.get(exponent_index), Some(b'0'..=b'9')) {
            exponent_index += 1;
        }
        if exponent_index > exponent_digit_start {
            index = exponent_index;
        } else {
            index = exponent_marker;
        }
    }

    Some(index)
}

fn compress_number_prefix(number: &str) -> String {
    let (sign, unsigned) = match number.as_bytes().first() {
        Some(b'+') | Some(b'-') => (&number[..1], &number[1..]),
        _ => ("", number),
    };
    let (mantissa, exponent) = split_number_exponent(unsigned);
    let compressed_mantissa = compress_decimal_mantissa(mantissa);
    let mut compressed = format!("{sign}{compressed_mantissa}");

    if let Some(exponent) = exponent {
        let normalized_exponent = normalize_exponent_suffix(exponent);
        if normalized_exponent != "0" && !is_zero_number_prefix(&compressed) {
            compressed.push('e');
            compressed.push_str(&normalized_exponent);
        }
    }

    compressed
}

fn split_number_exponent(number: &str) -> (&str, Option<&str>) {
    if let Some(index) = number.find(['e', 'E']) {
        (&number[..index], Some(&number[index + 1..]))
    } else {
        (number, None)
    }
}

fn compress_decimal_mantissa(mantissa: &str) -> String {
    let Some((before_dot, after_dot)) = mantissa.split_once('.') else {
        return mantissa.to_string();
    };

    let trimmed_fraction = after_dot.trim_end_matches('0');
    let mut compressed_unsigned = if trimmed_fraction.is_empty() {
        before_dot.to_string()
    } else {
        format!("{before_dot}.{trimmed_fraction}")
    };

    if let Some(rest) = compressed_unsigned.strip_prefix("0.") {
        compressed_unsigned = format!(".{rest}");
    }

    if compressed_unsigned.is_empty() {
        compressed_unsigned.push('0');
    }

    compressed_unsigned
}

fn normalize_exponent_suffix(exponent: &str) -> String {
    let (sign, digits) = match exponent.as_bytes().first() {
        Some(b'+') => ("", &exponent[1..]),
        Some(b'-') => ("-", &exponent[1..]),
        _ => ("", exponent),
    };
    let digits = digits.trim_start_matches('0');
    let digits = if digits.is_empty() { "0" } else { digits };
    if digits == "0" {
        digits.to_string()
    } else {
        format!("{sign}{digits}")
    }
}

fn normalize_css_whitespace_with_lexer(source: &str, dialect: StyleDialect) -> (String, usize) {
    let lexed = lex(source, dialect);
    let tokens = lexed.tokens();
    let mut output = String::with_capacity(source.len());
    let mut mutation_count = 0;

    for (index, token) in tokens.iter().enumerate() {
        if token.kind != SyntaxKind::Whitespace && token.kind != SyntaxKind::SassIndentedNewline {
            output.push_str(&token.text);
            continue;
        }

        let replacement = whitespace_replacement_for_tokens(
            previous_non_comment_token_kind(tokens, index),
            next_non_comment_token_kind(tokens, index),
        );
        if replacement != token.text {
            mutation_count += 1;
        }
        output.push_str(replacement);
    }

    (output, mutation_count)
}

fn whitespace_replacement_for_tokens(
    previous: Option<SyntaxKind>,
    next: Option<SyntaxKind>,
) -> &'static str {
    match (previous, next) {
        (None, _) | (_, None) => "",
        (Some(previous), Some(next))
            if can_remove_whitespace_after(previous) || can_remove_whitespace_before(next) =>
        {
            ""
        }
        _ => " ",
    }
}

fn previous_non_comment_token_kind(
    tokens: &[omena_parser::LexedToken],
    index: usize,
) -> Option<SyntaxKind> {
    tokens[..index]
        .iter()
        .rev()
        .find(|token| !is_comment_token(token.kind) && token.kind != SyntaxKind::Whitespace)
        .map(|token| token.kind)
}

fn next_non_comment_token_kind(
    tokens: &[omena_parser::LexedToken],
    index: usize,
) -> Option<SyntaxKind> {
    tokens
        .get(index + 1..)
        .unwrap_or_default()
        .iter()
        .find(|token| !is_comment_token(token.kind) && token.kind != SyntaxKind::Whitespace)
        .map(|token| token.kind)
}

fn can_remove_whitespace_after(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::LeftBrace
            | SyntaxKind::RightBrace
            | SyntaxKind::LeftParen
            | SyntaxKind::LeftBracket
            | SyntaxKind::Comma
            | SyntaxKind::Semicolon
    )
}

fn can_remove_whitespace_before(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::LeftBrace
            | SyntaxKind::RightBrace
            | SyntaxKind::RightParen
            | SyntaxKind::RightBracket
            | SyntaxKind::Comma
            | SyntaxKind::Semicolon
    )
}

fn strip_css_comments_with_lexer(source: &str, dialect: StyleDialect) -> (String, usize) {
    let lexed = lex(source, dialect);
    let mut output = String::with_capacity(source.len());
    let mut cursor = 0;
    let mut removed_comment_count = 0;

    for token in lexed.tokens() {
        let start = u32::from(token.range.start()) as usize;
        let end = u32::from(token.range.end()) as usize;
        if start > cursor {
            output.push_str(&source[cursor..start]);
        }
        if is_comment_token(token.kind) {
            removed_comment_count += 1;
        } else {
            output.push_str(&source[start..end]);
        }
        cursor = end;
    }

    if cursor < source.len() {
        output.push_str(&source[cursor..]);
    }

    (output, removed_comment_count)
}

fn is_comment_token(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::LineComment | SyntaxKind::BlockComment | SyntaxKind::ScssSilentComment
    )
}

#[cfg(test)]
mod tests {
    use super::{
        TransformClassNameRewriteV0, TransformCssModuleComposesResolutionV0,
        TransformDesignTokenRouteV0, TransformExecutionContextV0, TransformImportInlineV0,
        TransformModuleEvaluationV0, TransformPassRuntimeStatus,
        execute_transform_passes_incremental_with_database, execute_transform_passes_on_source,
        execute_transform_passes_on_source_with_dialect,
        execute_transform_passes_on_source_with_dialect_and_context, plan_transform_passes,
        run_transform_fuzz_seed_corpus, summarize_omena_transform_passes_boundary,
        transform_pass_incremental_graph_input,
    };
    use omena_incremental::{IncrementalRevisionV0, OmenaIncrementalDatabaseV0};
    use omena_parser::StyleDialect;
    use omena_transform_cst::{TRANSFORM_PASS_CATALOG_LEN, TransformPassKind};

    #[test]
    fn registry_covers_full_transform_catalog() {
        let boundary = summarize_omena_transform_passes_boundary();

        assert_eq!(boundary.schema_version, "0");
        assert_eq!(boundary.product, "omena-transform-passes.boundary");
        assert_eq!(boundary.pass_count, TRANSFORM_PASS_CATALOG_LEN);
        assert!(boundary.full_catalog_registered);
        assert_eq!(boundary.semantic_aware_pass_count, 14);
        assert!(boundary.cascade_aware_pass_count >= 9);
        assert!(boundary.planner_enforces_dag_edges);
        assert!(boundary.execution_runtime_ready);
        assert!(boundary.incremental_execution_runtime_ready);
        assert_eq!(
            boundary.implemented_mutation_pass_ids,
            vec![
                "whitespace-strip",
                "comment-strip",
                "number-compression",
                "unit-normalization",
                "color-compression",
                "url-quote-strip",
                "string-quote-normalize",
                "selector-is-where-compression",
                "shorthand-combining",
                "rule-deduplication",
                "rule-merging",
                "selector-merging",
                "empty-rule-removal",
                "vendor-prefixing",
                "light-dark-lowering",
                "color-mix-lowering",
                "oklch-oklab-lowering",
                "color-function-lowering",
                "logical-to-physical",
                "nesting-unwrap",
                "scope-flatten",
                "layer-flatten",
                "supports-static-eval",
                "media-static-eval",
                "dead-media-branch-removal",
                "dead-supports-branch-removal",
                "import-inline",
                "scss-module-evaluate",
                "less-module-evaluate",
                "value-resolution",
                "custom-property-static-resolve",
                "composes-resolution",
                "css-modules-class-hashing",
                "tree-shake-class",
                "tree-shake-keyframes",
                "tree-shake-value",
                "tree-shake-custom-property",
                "design-token-routing",
                "calc-reduction",
                "print-css"
            ]
        );
        assert!(boundary.registry_entries.iter().any(|entry| {
            entry.contract.kind == TransformPassKind::TreeShakeClass
                && entry.module_family == "semantic-reachability"
        }));
        assert!(
            !boundary
                .next_surfaces
                .contains(&"transformContextProducers")
        );
        assert!(
            !boundary
                .next_surfaces
                .contains(&"provenanceSourceSpanMapping")
        );
        assert!(!boundary.next_surfaces.contains(&"transformSalsaQueries"));
        assert!(!boundary.next_surfaces.contains(&"sourceMapSpanPrecision"));
    }

    #[test]
    fn incremental_transform_graph_tracks_source_context_plan_and_pass_dependencies() {
        let context = TransformExecutionContextV0 {
            reachable_class_names: vec!["used".to_string()],
            ..TransformExecutionContextV0::default()
        };
        let graph = transform_pass_incremental_graph_input(
            ".used { color: red; }",
            StyleDialect::Css,
            &[
                TransformPassKind::TreeShakeClass,
                TransformPassKind::PrintCss,
            ],
            &context,
            IncrementalRevisionV0 { value: 1 },
        );

        assert_eq!(graph.revision.value, 1);
        assert!(graph.nodes.iter().any(|node| node.id == "transform:source"));
        assert!(
            graph
                .nodes
                .iter()
                .any(|node| node.id == "transform:context")
        );
        assert!(graph.nodes.iter().any(|node| node.id == "transform:plan"));
        let execution_node = graph
            .nodes
            .iter()
            .find(|node| node.id == "transform:execution");
        assert!(execution_node.is_some());
        if let Some(execution_node) = execution_node {
            assert!(
                execution_node
                    .dependency_ids
                    .contains(&"transform:pass:print-css".to_string())
            );
        }
    }

    #[test]
    fn incremental_transform_execution_reuses_clean_salsa_database_plan() {
        let mut incremental_database = OmenaIncrementalDatabaseV0::default();
        let context = TransformExecutionContextV0::default();
        let requested = [TransformPassKind::CommentStrip, TransformPassKind::PrintCss];
        let first = execute_transform_passes_incremental_with_database(
            ".button { /* keep no comment */ color: red; }",
            StyleDialect::Css,
            &requested,
            &context,
            &mut incremental_database,
            None,
            IncrementalRevisionV0 { value: 1 },
        );

        assert_eq!(
            first.product,
            "omena-transform-passes.incremental-execution"
        );
        assert_eq!(first.incremental_engine, "omena-incremental");
        assert!(!first.reused_previous_execution);
        assert!(first.incremental_plan.dirty_node_count > 0);
        assert!(first.ready_surfaces.contains(&"transformSalsaQueries"));

        let reused = execute_transform_passes_incremental_with_database(
            ".button { /* keep no comment */ color: red; }",
            StyleDialect::Css,
            &requested,
            &context,
            &mut incremental_database,
            Some(&first.execution),
            IncrementalRevisionV0 { value: 2 },
        );

        assert!(reused.reused_previous_execution);
        assert_eq!(reused.incremental_plan.dirty_node_count, 0);
        assert_eq!(reused.execution.output_css, first.execution.output_css);

        let changed = execute_transform_passes_incremental_with_database(
            ".button { /* changed */ color: blue; }",
            StyleDialect::Css,
            &requested,
            &context,
            &mut incremental_database,
            Some(&reused.execution),
            IncrementalRevisionV0 { value: 3 },
        );

        assert!(!changed.reused_previous_execution);
        assert!(changed.incremental_plan.changed_input_count >= 1);
        assert!(changed.execution.output_css.contains("blue"));
    }

    #[test]
    fn fuzz_seed_corpus_preserves_transform_cascade_safe_invariants() {
        let report = run_transform_fuzz_seed_corpus();

        assert_eq!(report.product, "omena-transform-passes.fuzz-seed-corpus");
        assert_eq!(report.failed_count, 0);
        assert_eq!(report.passed_count, report.case_count);
        assert!(
            report
                .results
                .iter()
                .all(|result| result.output_error_count == 0)
        );
        assert!(
            report
                .results
                .iter()
                .any(|result| !result.executed_pass_ids.is_empty())
        );
    }

    #[test]
    fn planner_respects_var_before_calc_before_print_edges() {
        let plan = plan_transform_passes(&[
            TransformPassKind::PrintCss,
            TransformPassKind::CalcReduction,
            TransformPassKind::StaticVarSubstitution,
        ]);

        assert_eq!(plan.violated_dag_edge_count, 0);
        assert!(plan.all_requested_registered);
        assert_eq!(
            plan.ordered_pass_ids,
            vec![
                "custom-property-static-resolve",
                "calc-reduction",
                "print-css"
            ]
        );
    }

    #[test]
    fn planner_respects_composes_before_hash_before_selector_merge_edges() {
        let plan = plan_transform_passes(&[
            TransformPassKind::SelectorMerging,
            TransformPassKind::HashCssModuleClassNames,
            TransformPassKind::ResolveCssModulesComposes,
        ]);

        assert_eq!(plan.violated_dag_edge_count, 0);
        assert_eq!(
            plan.ordered_pass_ids,
            vec![
                "composes-resolution",
                "css-modules-class-hashing",
                "selector-merging"
            ]
        );
    }

    #[test]
    fn planner_respects_nesting_before_hash_edges() {
        let plan = plan_transform_passes(&[
            TransformPassKind::HashCssModuleClassNames,
            TransformPassKind::NestingUnwrap,
            TransformPassKind::PrintCss,
        ]);

        assert_eq!(plan.violated_dag_edge_count, 0);
        assert_eq!(
            plan.ordered_pass_ids,
            vec!["nesting-unwrap", "css-modules-class-hashing", "print-css"]
        );
    }

    #[test]
    fn execution_runtime_inlines_imports_from_explicit_replacements() {
        let source = r#"@import "./tokens.css"; @import url(./theme.css); @import "./conditional.css" layer(theme) supports(display: grid) screen and (min-width: 40rem); .button { color: var(--brand); }"#;
        let context = TransformExecutionContextV0 {
            import_inlines: vec![
                TransformImportInlineV0 {
                    import_source: "./tokens.css".to_string(),
                    replacement_css: r#":root { --brand: red; }"#.to_string(),
                },
                TransformImportInlineV0 {
                    import_source: "./theme.css".to_string(),
                    replacement_css: r#"@media screen { .theme { color: blue; } }"#.to_string(),
                },
                TransformImportInlineV0 {
                    import_source: "./conditional.css".to_string(),
                    replacement_css: r#".conditional { color: green; }"#.to_string(),
                },
            ],
            ..TransformExecutionContextV0::default()
        };
        let execution = execute_transform_passes_on_source_with_dialect_and_context(
            source,
            StyleDialect::Css,
            &[TransformPassKind::ImportInline, TransformPassKind::PrintCss],
            &context,
        );

        assert_eq!(execution.mutation_count, 3);
        assert_eq!(
            execution.output_css,
            r#":root { --brand: red; } @media screen { .theme { color: blue; } } @media screen and (min-width: 40rem) { @supports (display: grid) { @layer theme { .conditional { color: green; } } } } .button { color: var(--brand); }"#
        );
        assert_eq!(
            execution.css_import_inlines,
            vec![
                TransformImportInlineV0 {
                    import_source: "./tokens.css".to_string(),
                    replacement_css: r#":root { --brand: red; }"#.to_string(),
                },
                TransformImportInlineV0 {
                    import_source: "./theme.css".to_string(),
                    replacement_css: r#"@media screen { .theme { color: blue; } }"#.to_string(),
                },
                TransformImportInlineV0 {
                    import_source: "./conditional.css".to_string(),
                    replacement_css: r#".conditional { color: green; }"#.to_string(),
                },
            ]
        );
        assert_eq!(
            execution.executed_pass_ids,
            vec!["import-inline", "print-css"]
        );
    }

    #[test]
    fn execution_runtime_applies_explicit_scss_module_evaluation() {
        let source = r#"$brand: red; .button { color: $brand; }"#;
        let context = TransformExecutionContextV0 {
            scss_module_evaluation: Some(TransformModuleEvaluationV0 {
                evaluator: "dart-sass-compatible".to_string(),
                evaluated_css: ".button { color: red; }".to_string(),
            }),
            ..TransformExecutionContextV0::default()
        };
        let execution = execute_transform_passes_on_source_with_dialect_and_context(
            source,
            StyleDialect::Scss,
            &[
                TransformPassKind::ScssModuleEvaluate,
                TransformPassKind::PrintCss,
            ],
            &context,
        );

        assert_eq!(execution.mutation_count, 1);
        assert_eq!(execution.output_css, ".button { color: red; }");
        assert_eq!(
            execution.css_module_evaluation,
            Some(TransformModuleEvaluationV0 {
                evaluator: "dart-sass-compatible".to_string(),
                evaluated_css: ".button { color: red; }".to_string(),
            })
        );
        assert_eq!(
            execution.executed_pass_ids,
            vec!["scss-module-evaluate", "print-css"]
        );
    }

    #[test]
    fn execution_runtime_applies_explicit_less_module_evaluation() {
        let source = r#"@brand: red; .button { color: @brand; }"#;
        let context = TransformExecutionContextV0 {
            less_module_evaluation: Some(TransformModuleEvaluationV0 {
                evaluator: "less-js-compatible".to_string(),
                evaluated_css: ".button { color: red; }".to_string(),
            }),
            ..TransformExecutionContextV0::default()
        };
        let execution = execute_transform_passes_on_source_with_dialect_and_context(
            source,
            StyleDialect::Less,
            &[
                TransformPassKind::LessModuleEvaluate,
                TransformPassKind::PrintCss,
            ],
            &context,
        );

        assert_eq!(execution.mutation_count, 1);
        assert_eq!(execution.output_css, ".button { color: red; }");
        assert_eq!(
            execution.css_module_evaluation,
            Some(TransformModuleEvaluationV0 {
                evaluator: "less-js-compatible".to_string(),
                evaluated_css: ".button { color: red; }".to_string(),
            })
        );
        assert_eq!(
            execution.executed_pass_ids,
            vec!["less-module-evaluate", "print-css"]
        );
    }

    #[test]
    fn execution_runtime_resolves_css_module_composes_with_export_set() {
        let source = r#".button { composes: base from "./base.module.css"; color: red; } .button:hover { color: blue; } .card, .panel { composes: shared; color: green; }"#;
        let context = TransformExecutionContextV0 {
            css_module_composes_resolutions: vec![TransformCssModuleComposesResolutionV0 {
                local_class_name: "button".to_string(),
                exported_class_names: vec!["button".to_string(), "base".to_string()],
            }],
            ..TransformExecutionContextV0::default()
        };
        let execution = execute_transform_passes_on_source_with_dialect_and_context(
            source,
            StyleDialect::Css,
            &[
                TransformPassKind::ResolveCssModulesComposes,
                TransformPassKind::PrintCss,
            ],
            &context,
        );

        assert_eq!(execution.mutation_count, 1);
        assert_eq!(
            execution.output_css,
            r#".button {  color: red; } .button:hover { color: blue; } .card, .panel { composes: shared; color: green; }"#
        );
        assert_eq!(
            execution.css_module_composes_exports,
            vec![TransformCssModuleComposesResolutionV0 {
                local_class_name: "button".to_string(),
                exported_class_names: vec!["button".to_string(), "base".to_string()],
            }]
        );
        assert_eq!(
            execution.executed_pass_ids,
            vec!["composes-resolution", "print-css"]
        );
    }

    #[test]
    fn execution_runtime_routes_design_tokens_from_bridge_context() {
        let source = r#".button { color: var(--pkg-brand); background: var(--pkg-brand, blue); border: 1px solid var(--pkg-border); box-shadow: 0 0 1px var(--unsafe); --local: var(--pkg-brand); } @media screen { .button { outline-color: var(--pkg-brand); } }"#;
        let context = TransformExecutionContextV0 {
            design_token_routes: vec![
                TransformDesignTokenRouteV0 {
                    token_name: "--pkg-brand".to_string(),
                    routed_value: "var(--theme-brand)".to_string(),
                },
                TransformDesignTokenRouteV0 {
                    token_name: "--pkg-border".to_string(),
                    routed_value: "#123456".to_string(),
                },
                TransformDesignTokenRouteV0 {
                    token_name: "--unsafe".to_string(),
                    routed_value: "red; color: blue".to_string(),
                },
            ],
            ..TransformExecutionContextV0::default()
        };
        let execution = execute_transform_passes_on_source_with_dialect_and_context(
            source,
            StyleDialect::Css,
            &[
                TransformPassKind::DesignTokenRouting,
                TransformPassKind::PrintCss,
            ],
            &context,
        );

        assert_eq!(execution.mutation_count, 4);
        assert_eq!(
            execution.output_css,
            r#".button { color: var(--theme-brand); background: var(--theme-brand, blue); border: 1px solid #123456; box-shadow: 0 0 1px var(--unsafe); --local: var(--pkg-brand); } @media screen { .button { outline-color: var(--theme-brand); } }"#
        );
        assert_eq!(execution.design_token_routes, context.design_token_routes);
        assert_eq!(
            execution.executed_pass_ids,
            vec!["design-token-routing", "print-css"]
        );
    }

    #[test]
    fn execution_runtime_applies_comment_strip_without_touching_strings() {
        let source = r#".a { color: red; /* remove */ content: "/* keep */"; }"#;
        let execution = execute_transform_passes_on_source(
            source,
            &[
                TransformPassKind::CommentStrip,
                TransformPassKind::HashCssModuleClassNames,
                TransformPassKind::PrintCss,
            ],
        );

        assert_eq!(execution.product, "omena-transform-passes.execution");
        assert_eq!(execution.mutation_count, 1);
        assert_eq!(
            execution.output_css,
            r#".a { color: red;  content: "/* keep */"; }"#
        );
        assert_eq!(
            execution.executed_pass_ids,
            vec!["comment-strip", "print-css"]
        );
        assert_eq!(
            execution.planned_only_pass_ids,
            vec!["css-modules-class-hashing"]
        );
        assert!(execution.provenance_preserved);
        assert_eq!(execution.pass_plan.violated_dag_edge_count, 0);
        assert!(execution.outcomes.iter().any(|outcome| {
            outcome.pass_id == "comment-strip"
                && outcome.status == TransformPassRuntimeStatus::Applied
                && outcome.mutation_count == 1
        }));
        assert!(execution.outcomes.iter().any(|outcome| {
            outcome.pass_id == "css-modules-class-hashing"
                && outcome.status == TransformPassRuntimeStatus::PlannedOnly
        }));
        assert_eq!(
            execution.provenance_derivation_forest.product,
            "omena-transform-passes.provenance-derivation-forest"
        );
        assert_eq!(execution.provenance_derivation_forest.root_count, 1);
        assert_eq!(
            execution.provenance_derivation_forest.node_count,
            execution.outcomes.len()
        );
        let comment_node = execution
            .provenance_derivation_forest
            .nodes
            .iter()
            .find(|node| node.pass_id == "comment-strip");
        assert!(
            comment_node.is_some(),
            "comment strip provenance node should exist"
        );
        let Some(comment_node) = comment_node else {
            return;
        };
        assert_eq!(comment_node.status, TransformPassRuntimeStatus::Applied);
        assert_eq!(comment_node.mutation_count, 1);
        assert_eq!(comment_node.mutation_spans.len(), 1);
        assert_eq!(comment_node.source_span_start, 17);
        assert!(comment_node.source_span_end < comment_node.input_byte_len);
        assert_eq!(comment_node.generated_span_start, 17);
        assert_eq!(comment_node.generated_span_end, 17);
        assert_eq!(
            execution.provenance_derivation_forest.nodes[0].parent_index,
            None
        );
        for (index, node) in execution
            .provenance_derivation_forest
            .nodes
            .iter()
            .enumerate()
            .skip(1)
        {
            assert_eq!(node.parent_index, Some(index - 1));
        }
    }

    #[test]
    fn execution_runtime_rewrites_css_module_class_names_with_identity_map() {
        let source = r#".button { composes: base utility; color: red; } .base, .utility { color: blue; } .button:hover { color: green; }"#;
        let context = TransformExecutionContextV0 {
            class_name_rewrites: vec![
                TransformClassNameRewriteV0 {
                    original_name: "button".to_string(),
                    rewritten_name: "_button_abc123".to_string(),
                },
                TransformClassNameRewriteV0 {
                    original_name: "base".to_string(),
                    rewritten_name: "_base_def456".to_string(),
                },
                TransformClassNameRewriteV0 {
                    original_name: "utility".to_string(),
                    rewritten_name: "_utility_ghi789".to_string(),
                },
            ],
            ..TransformExecutionContextV0::default()
        };
        let execution = execute_transform_passes_on_source_with_dialect_and_context(
            source,
            StyleDialect::Css,
            &[
                TransformPassKind::HashCssModuleClassNames,
                TransformPassKind::PrintCss,
            ],
            &context,
        );

        assert_eq!(execution.mutation_count, 4);
        assert_eq!(
            execution.output_css,
            r#"._button_abc123{ composes: _base_def456 _utility_ghi789; color: red; } ._base_def456, ._utility_ghi789{ color: blue; } ._button_abc123:hover{ color: green; }"#
        );
        assert_eq!(
            execution.executed_pass_ids,
            vec!["css-modules-class-hashing", "print-css"]
        );
    }

    #[test]
    fn execution_runtime_hashes_nested_css_module_selectors_after_unwrap() {
        let source =
            r#".item { color: red; &--primary { color: blue; } & .body { color: green; } }"#;
        let context = TransformExecutionContextV0 {
            class_name_rewrites: vec![
                TransformClassNameRewriteV0 {
                    original_name: "item".to_string(),
                    rewritten_name: "_item_0".to_string(),
                },
                TransformClassNameRewriteV0 {
                    original_name: "item--primary".to_string(),
                    rewritten_name: "_item--primary_1".to_string(),
                },
                TransformClassNameRewriteV0 {
                    original_name: "body".to_string(),
                    rewritten_name: "_body_2".to_string(),
                },
            ],
            ..TransformExecutionContextV0::default()
        };
        let execution = execute_transform_passes_on_source_with_dialect_and_context(
            source,
            StyleDialect::Scss,
            &[
                TransformPassKind::HashCssModuleClassNames,
                TransformPassKind::NestingUnwrap,
                TransformPassKind::PrintCss,
            ],
            &context,
        );

        assert_eq!(
            execution.ordered_pass_ids,
            vec!["nesting-unwrap", "css-modules-class-hashing", "print-css"]
        );
        assert!(execution.output_css.contains("._item_0{ color: red; }"));
        assert!(
            execution
                .output_css
                .contains("._item--primary_1{ color: blue; }")
        );
        assert!(
            execution
                .output_css
                .contains("._item_0 ._body_2{ color: green; }")
        );
        assert_eq!(
            execution.executed_pass_ids,
            vec!["nesting-unwrap", "css-modules-class-hashing", "print-css"]
        );
    }

    #[test]
    fn execution_runtime_applies_conservative_whitespace_normalization() {
        let source = r#".a , .b { color : red ; content: "x y"; }"#;
        let execution = execute_transform_passes_on_source(
            source,
            &[
                TransformPassKind::WhitespaceStrip,
                TransformPassKind::CommentStrip,
                TransformPassKind::PrintCss,
            ],
        );

        assert_eq!(execution.mutation_count, 7);
        assert_eq!(
            execution.output_css,
            r#".a,.b{color : red;content: "x y";}"#
        );
        assert_eq!(
            execution.executed_pass_ids,
            vec!["whitespace-strip", "comment-strip", "print-css"]
        );
    }

    #[test]
    fn execution_runtime_compresses_numeric_tokens_only() {
        let source = r#".a { width: 0.50rem; opacity: 1.0; margin: -0.25px 10.00%; scale: 1.0E+03; flex-grow: 1e+00; translate: 0e+3px; content: "0.50 1.0E+03"; }"#;
        let execution = execute_transform_passes_on_source(
            source,
            &[
                TransformPassKind::NumberCompression,
                TransformPassKind::PrintCss,
            ],
        );

        assert_eq!(execution.mutation_count, 7);
        assert_eq!(
            execution.output_css,
            r#".a { width: .5rem; opacity: 1; margin: -.25px 10%; scale: 1e3; flex-grow: 1; translate: 0px; content: "0.50 1.0E+03"; }"#
        );
    }

    #[test]
    fn execution_runtime_normalizes_zero_length_units_with_property_context() {
        let source = r#".a { margin: 0px 0.0rem -0em; border-top-width: 0PX; border-radius: -0em; scroll-margin-inline: 0rem; outline-width: 0pt; rotate: 1TURN; animation-delay: 200MS; grid-template-columns: 1FR 2fr; --x: 0PX; width: 10PX; }"#;
        let execution = execute_transform_passes_on_source(
            source,
            &[
                TransformPassKind::UnitNormalization,
                TransformPassKind::PrintCss,
            ],
        );

        assert_eq!(execution.mutation_count, 11);
        assert_eq!(
            execution.output_css,
            r#".a { margin: 0 0 0; border-top-width: 0; border-radius: 0; scroll-margin-inline: 0; outline-width: 0; rotate: 1turn; animation-delay: 200ms; grid-template-columns: 1fr 2fr; --x: 0PX; width: 10px; }"#
        );
        assert_eq!(
            execution.executed_pass_ids,
            vec!["unit-normalization", "print-css"]
        );
    }

    #[test]
    fn execution_runtime_compresses_static_declaration_colors_only() {
        let source = r#".a { color: #FFFFFF; box-shadow: 0 0 #AABBCC; background-color: rgb(255 0 0); border-color: rgb(0, 128, 0); outline-color: rgb(50% 50% 50%); text-emphasis-color: rgb(128 0 128); text-decoration-color: hsl(240 100% 50%); caret-color: hsl(0, 0%, 0%); fill: hwb(0 0% 0%); stroke: hwb(120 0% 50%); column-rule-color: hwb(0 100% 0%); flood-color: white; lighting-color: black; stop-color: blue; scrollbar-color: hsl(.5TURN 100% 50%); border-block-color: hwb(200GRAD 0% 0%); border-left-color: rgb(255 0 0 / 100%); border-right-color: hsl(120 100% 25% / 1); border-top-color: hwb(240 0% 0% / 100%); border-bottom-color: rgb(255 0 0 / .5); accent-color: hsl(0 0% 0% / 50%); --brand: rgb(255 0 0); } #FFFFFF { color: red; }"#;
        let execution = execute_transform_passes_on_source(
            source,
            &[
                TransformPassKind::ColorCompression,
                TransformPassKind::PrintCss,
            ],
        );

        assert_eq!(execution.mutation_count, 18);
        assert_eq!(
            execution.output_css,
            r#".a { color: #fff; box-shadow: 0 0 #abc; background-color: red; border-color: green; outline-color: gray; text-emphasis-color: purple; text-decoration-color: #00f; caret-color: #000; fill: red; stroke: green; column-rule-color: #fff; flood-color: #fff; lighting-color: #000; stop-color: blue; scrollbar-color: #0ff; border-block-color: #0ff; border-left-color: red; border-right-color: green; border-top-color: #00f; border-bottom-color: rgb(255 0 0 / .5); accent-color: hsl(0 0% 0% / 50%); --brand: rgb(255 0 0); } #FFFFFF { color: red; }"#
        );
    }

    #[test]
    fn execution_runtime_strips_safe_url_quotes_only() {
        let source = r#".a { background: url("img/icon.svg"); mask: url("has space.svg"); content: "url(\"keep\")"; }"#;
        let execution = execute_transform_passes_on_source(
            source,
            &[
                TransformPassKind::UrlQuoteStrip,
                TransformPassKind::PrintCss,
            ],
        );

        assert_eq!(execution.mutation_count, 1);
        assert_eq!(
            execution.output_css,
            r#".a { background: url(img/icon.svg); mask: url("has space.svg"); content: "url(\"keep\")"; }"#
        );
    }

    #[test]
    fn execution_runtime_normalizes_safe_single_quoted_strings_only() {
        let source =
            r#".a { font-family: 'Demo'; content: 'has "quote"'; background: url('asset.svg'); }"#;
        let execution = execute_transform_passes_on_source(
            source,
            &[
                TransformPassKind::StringQuoteNormalize,
                TransformPassKind::PrintCss,
            ],
        );

        assert_eq!(execution.mutation_count, 2);
        assert_eq!(
            execution.output_css,
            r#".a { font-family: "Demo"; content: 'has "quote"'; background: url("asset.svg"); }"#
        );
    }

    #[test]
    fn execution_runtime_compresses_specificity_safe_is_where_selectors() {
        let source = r#".a:is(.ready) { color: red; } .b:where(.x, .x) { color: blue; } .c:where(.y) { color: green; } .d:is(:is(.u, .v), .u) { color: orange; } .e, .e, .f { color: purple; } .w:where(:where(.one, .two), .one) { color: teal; } @media (min-width: 1px) { .m, .m, .n { color: black; } } @supports (display: grid) { .s, .s { display: grid; } }"#;
        let execution = execute_transform_passes_on_source(
            source,
            &[
                TransformPassKind::SelectorIsWhereCompression,
                TransformPassKind::PrintCss,
            ],
        );

        assert_eq!(execution.mutation_count, 7);
        assert_eq!(
            execution.output_css,
            r#".a.ready { color: red; } .b:where(.x) { color: blue; } .c:where(.y) { color: green; } .d:is(.u,.v) { color: orange; } .e, .f { color: purple; } .w:where(.one,.two) { color: teal; } @media (min-width: 1px) { .m, .n { color: black; } } @supports (display: grid) { .s { display: grid; } }"#
        );
        assert_eq!(
            execution.executed_pass_ids,
            vec!["selector-is-where-compression", "print-css"]
        );
    }

    #[test]
    fn execution_runtime_removes_only_plain_empty_rules() {
        let source = r#".empty { } @media (min-width: 1px) { .nested { } } .outer { .inner { } } .with-comment { /* keep */ } .filled { color: red; }"#;
        let execution = execute_transform_passes_on_source(
            source,
            &[
                TransformPassKind::EmptyRuleRemoval,
                TransformPassKind::PrintCss,
            ],
        );

        assert_eq!(execution.mutation_count, 5);
        assert_eq!(
            execution.output_css,
            r#"   .with-comment { /* keep */ } .filled { color: red; }"#
        );
        assert_eq!(
            execution.executed_pass_ids,
            vec!["empty-rule-removal", "print-css"]
        );
    }

    #[test]
    fn execution_runtime_combines_adjacent_box_longhands_with_cascade_proof() {
        let source = r#".a { margin-top: 1px; margin-right: 2px; margin-bottom: 1px; margin-left: 2px; padding-top: 1px; color: red; padding-right: 2px; padding-bottom: 3px; padding-left: 4px; }"#;
        let execution = execute_transform_passes_on_source(
            source,
            &[
                TransformPassKind::ShorthandCombining,
                TransformPassKind::PrintCss,
            ],
        );

        assert_eq!(execution.mutation_count, 1);
        assert_eq!(
            execution.output_css,
            r#".a { margin: 1px 2px; padding-top: 1px; color: red; padding-right: 2px; padding-bottom: 3px; padding-left: 4px; }"#
        );
        assert_eq!(
            execution.executed_pass_ids,
            vec!["shorthand-combining", "print-css"]
        );
    }

    #[test]
    fn execution_runtime_removes_cascade_safe_duplicate_rules() {
        let source = r#".a { color: red; } .b { color: red; } .a { color: blue; } .a { color: red; } @media (min-width: 1px) { .m { color: red; } .x { color: blue; } .m { color: red; } } @media (max-width: 1px) { .m { color: red; } }"#;
        let execution = execute_transform_passes_on_source(
            source,
            &[
                TransformPassKind::RuleDeduplication,
                TransformPassKind::PrintCss,
            ],
        );

        assert_eq!(execution.mutation_count, 2);
        assert_eq!(
            execution.output_css,
            r#" .b { color: red; } .a { color: blue; } .a { color: red; } @media (min-width: 1px) {  .x { color: blue; } .m { color: red; } } @media (max-width: 1px) { .m { color: red; } }"#
        );
        assert_eq!(
            execution.executed_pass_ids,
            vec!["rule-deduplication", "print-css"]
        );
    }

    #[test]
    fn execution_runtime_merges_adjacent_same_selector_rules_only() {
        let source = r#".a { color: red; } .a { background: blue; } .a { outline: 0; } .b { color: red; } .a { border: 0; } @media (min-width: 1px) { .m { color: red; } .m { background: blue; } }"#;
        let execution = execute_transform_passes_on_source(
            source,
            &[TransformPassKind::RuleMerging, TransformPassKind::PrintCss],
        );

        assert_eq!(execution.mutation_count, 2);
        assert_eq!(
            execution.output_css,
            r#".a { color: red; background: blue; outline: 0; } .b { color: red; } .a { border: 0; } @media (min-width: 1px) { .m { color: red; background: blue; } }"#
        );
        assert_eq!(
            execution.executed_pass_ids,
            vec!["rule-merging", "print-css"]
        );
    }

    #[test]
    fn execution_runtime_merges_adjacent_same_block_selectors_only() {
        let source = r#".a { color: red; } .b { color: red; } .c { color: red; } .d { color: blue; } .e { color: red; } @media (min-width: 1px) { .m { color: black; } .n { color: black; } }"#;
        let execution = execute_transform_passes_on_source(
            source,
            &[
                TransformPassKind::SelectorMerging,
                TransformPassKind::PrintCss,
            ],
        );

        assert_eq!(execution.mutation_count, 2);
        assert_eq!(
            execution.output_css,
            r#".a, .b, .c { color: red; } .d { color: blue; } .e { color: red; } @media (min-width: 1px) { .m, .n { color: black; } }"#
        );
        assert_eq!(
            execution.executed_pass_ids,
            vec!["selector-merging", "print-css"]
        );
    }

    #[test]
    fn execution_runtime_adds_conservative_vendor_prefixes_when_absent() {
        let source = r#".a { user-select: none; -webkit-appearance: none; appearance: none; backdrop-filter: blur(2px); } .flex { display: flex; position: sticky; } .inline { display: -webkit-inline-box; display: inline-flex; } .extra { text-size-adjust: 100%; mask-image: linear-gradient(red, blue); hyphens: auto; } .print { print-color-adjust: exact; -webkit-mask-size: cover; mask-size: cover; } @keyframes fade { from { opacity: 0; } to { opacity: 1; } } @-webkit-keyframes spin { from { opacity: 0; } } @keyframes spin { from { opacity: 0; } }"#;
        let execution = execute_transform_passes_on_source(
            source,
            &[
                TransformPassKind::VendorPrefixing,
                TransformPassKind::PrintCss,
            ],
        );

        assert_eq!(execution.mutation_count, 15);
        assert_eq!(
            execution.output_css,
            r#".a { -webkit-user-select: none; -moz-user-select: none; -ms-user-select: none; user-select: none; -webkit-appearance: none; -moz-appearance: none; appearance: none; -webkit-backdrop-filter: blur(2px); backdrop-filter: blur(2px); } .flex { display: -webkit-box; display: -ms-flexbox; display: flex; position: -webkit-sticky; position: sticky; } .inline { display: -webkit-inline-box; display: -ms-inline-flexbox; display: inline-flex; } .extra { -webkit-text-size-adjust: 100%; text-size-adjust: 100%; -webkit-mask-image: linear-gradient(red, blue); mask-image: linear-gradient(red, blue); -webkit-hyphens: auto; -ms-hyphens: auto; hyphens: auto; } .print { -webkit-print-color-adjust: exact; print-color-adjust: exact; -webkit-mask-size: cover; mask-size: cover; } @-webkit-keyframes fade { from { opacity: 0; } to { opacity: 1; } } @keyframes fade { from { opacity: 0; } to { opacity: 1; } } @-webkit-keyframes spin { from { opacity: 0; } } @keyframes spin { from { opacity: 0; } }"#
        );
        assert_eq!(
            execution.executed_pass_ids,
            vec!["vendor-prefixing", "print-css"]
        );
    }

    #[test]
    fn execution_runtime_lowers_whole_value_light_dark_declarations() {
        let source = r#".card { color: light-dark(#000, #fff); background: var(--keep); }"#;
        let execution = execute_transform_passes_on_source(
            source,
            &[
                TransformPassKind::LightDarkLowering,
                TransformPassKind::PrintCss,
            ],
        );

        assert_eq!(execution.mutation_count, 1);
        assert_eq!(
            execution.output_css,
            r#".card { color: #000; background: var(--keep); } @media (prefers-color-scheme: dark) { .card { color: #fff; } }"#
        );
        assert_eq!(
            execution.executed_pass_ids,
            vec!["light-dark-lowering", "print-css"]
        );
    }

    #[test]
    fn execution_runtime_lowers_static_srgb_color_mix_declarations() {
        let source = r#".card { color: color-mix(in srgb, red 50%, blue 50%); background-color: color-mix(in srgb, #000, #fff 25%); outline-color: color-mix(in srgb, rgb(255 0 0) 25%, hsl(240 100% 50%) 75%); text-decoration-color: color-mix(in srgb, hwb(120 0% 50%) 40%, white 60%); caret-color: color-mix(in srgb, black 12.5%, white 87.5%); background: linear-gradient(color-mix(in srgb, red 25%, blue 75%), white); border-color: color-mix(in oklab, red, blue); }"#;
        let execution = execute_transform_passes_on_source(
            source,
            &[
                TransformPassKind::ColorMixLowering,
                TransformPassKind::PrintCss,
            ],
        );

        assert_eq!(execution.mutation_count, 6);
        assert_eq!(
            execution.output_css,
            r#".card { color: rgb(128 0 128); background-color: rgb(64 64 64); outline-color: rgb(64 0 191); text-decoration-color: rgb(153 204 153); caret-color: rgb(223 223 223); background: linear-gradient(rgb(64 0 191), white); border-color: color-mix(in oklab, red, blue); }"#
        );
        assert_eq!(
            execution.executed_pass_ids,
            vec!["color-mix-lowering", "print-css"]
        );
    }

    #[test]
    fn execution_runtime_lowers_in_gamut_oklab_oklch_declarations() {
        let source = r#".card { color: oklab(1 0 0); background-color: oklch(0% 0 0deg); outline-color: oklch(0% 0 0.5TURN); background: linear-gradient(oklch(0% 0 0deg), white); border-color: oklch(70% 0.4 40deg); }"#;
        let execution = execute_transform_passes_on_source(
            source,
            &[
                TransformPassKind::OklchOklabLowering,
                TransformPassKind::PrintCss,
            ],
        );

        assert_eq!(execution.mutation_count, 4);
        assert_eq!(
            execution.output_css,
            r#".card { color: rgb(255 255 255); background-color: rgb(0 0 0); outline-color: rgb(0 0 0); background: linear-gradient(rgb(0 0 0), white); border-color: oklch(70% 0.4 40deg); }"#
        );
        assert_eq!(
            execution.executed_pass_ids,
            vec!["oklch-oklab-lowering", "print-css"]
        );
    }

    #[test]
    fn execution_runtime_lowers_static_srgb_color_function_declarations() {
        let source = r#".card { color: color(srgb 1 0 0); background-color: color(srgb 50% 25% 0% / 100%); outline-color: color(srgb 0 0 1 / 1); fill: color(display-p3 0.5 0.5 0.5 / 100%); background: linear-gradient(color(srgb 1 0 0), white); accent-color: color(srgb 1 0 0 / .5); border-color: color(display-p3 1 0 0); }"#;
        let execution = execute_transform_passes_on_source(
            source,
            &[
                TransformPassKind::ColorFunctionLowering,
                TransformPassKind::PrintCss,
            ],
        );

        assert_eq!(execution.mutation_count, 5);
        assert_eq!(
            execution.output_css,
            r#".card { color: rgb(255 0 0); background-color: rgb(128 64 0); outline-color: rgb(0 0 255); fill: rgb(128 128 128); background: linear-gradient(rgb(255 0 0), white); accent-color: color(srgb 1 0 0 / .5); border-color: color(display-p3 1 0 0); }"#
        );
        assert_eq!(
            execution.executed_pass_ids,
            vec!["color-function-lowering", "print-css"]
        );
    }

    #[test]
    fn execution_runtime_lowers_logical_properties_only_with_static_direction() {
        let source = r#".ltr { direction: ltr; margin-inline-start: 1px; padding-inline-end: 2px; inline-size: 10rem; margin-inline: 1px 2px; padding-inline: calc(1rem + 1px) 3px; border-inline-color: red blue; } .unknown { margin-inline-start: 1px; } .rtl { direction: rtl; writing-mode: horizontal-tb; inset-inline-start: 3px; border-inline-end-color: red; inset-inline: 4px 5px; border-inline: 1px solid red; border-inline-start: 2px dashed blue; }"#;
        let execution = execute_transform_passes_on_source(
            source,
            &[
                TransformPassKind::LogicalToPhysical,
                TransformPassKind::PrintCss,
            ],
        );

        assert_eq!(execution.mutation_count, 11);
        assert_eq!(
            execution.output_css,
            r#".ltr { direction: ltr; margin-left: 1px; padding-right: 2px; width: 10rem; margin-left: 1px; margin-right: 2px; padding-left: calc(1rem + 1px); padding-right: 3px; border-left-color: red; border-right-color: blue; } .unknown { margin-inline-start: 1px; } .rtl { direction: rtl; writing-mode: horizontal-tb; right: 3px; border-left-color: red; right: 4px; left: 5px; border-right: 1px solid red; border-left: 1px solid red; border-right: 2px dashed blue; }"#
        );
        assert_eq!(
            execution.executed_pass_ids,
            vec!["logical-to-physical", "print-css"]
        );
    }

    #[test]
    fn execution_runtime_unwraps_simple_single_depth_nesting() {
        let source = r#".card { color: red; & .title { color: blue; } &:hover { color: green; } } .comma, .skip { & .x { color: red; } }"#;
        let execution = execute_transform_passes_on_source(
            source,
            &[
                TransformPassKind::NestingUnwrap,
                TransformPassKind::PrintCss,
            ],
        );

        assert_eq!(execution.mutation_count, 2);
        assert_eq!(
            execution.output_css,
            r#".card { color: red; } .card .title { color: blue; } .card:hover { color: green; } .comma .x, .skip .x { color: red; }"#
        );
        assert_eq!(
            execution.executed_pass_ids,
            vec!["nesting-unwrap", "print-css"]
        );
    }

    #[test]
    fn execution_runtime_unwraps_selector_list_nesting_without_splitting_function_commas() {
        let source = r#".card:is(.active, .selected), .panel { &:hover, &--open { color: red; } }"#;
        let execution = execute_transform_passes_on_source(
            source,
            &[
                TransformPassKind::NestingUnwrap,
                TransformPassKind::PrintCss,
            ],
        );

        assert_eq!(execution.mutation_count, 1);
        assert_eq!(
            execution.output_css,
            r#".card:is(.active, .selected):hover, .card:is(.active, .selected)--open, .panel:hover, .panel--open { color: red; }"#
        );
    }

    #[test]
    fn execution_runtime_unwraps_nested_rule_descendants() {
        let source = r#".card { color: red; & .title { font-weight: bold; &:hover { color: blue; } .icon, &__icon { color: green; } } }"#;
        let execution = execute_transform_passes_on_source(
            source,
            &[
                TransformPassKind::NestingUnwrap,
                TransformPassKind::PrintCss,
            ],
        );

        assert_eq!(execution.mutation_count, 1);
        assert_eq!(
            execution.output_css,
            r#".card { color: red; } .card .title { font-weight: bold; } .card .title:hover { color: blue; } .card .title .icon, .card .title__icon { color: green; }"#
        );
    }

    #[test]
    fn execution_runtime_bubbles_nested_conditional_group_rules() {
        let source = r#".card { color: red; @media (min-width: 40rem) { color: blue; &:hover { color: green; } } @supports (display: grid) { & .title { display: grid; } } }"#;
        let execution = execute_transform_passes_on_source(
            source,
            &[
                TransformPassKind::NestingUnwrap,
                TransformPassKind::PrintCss,
            ],
        );

        assert_eq!(execution.mutation_count, 1);
        assert_eq!(
            execution.output_css,
            r#".card { color: red; } @media (min-width: 40rem) { .card { color: blue; } .card:hover { color: green; } } @supports (display: grid) { .card .title { display: grid; } }"#
        );
    }

    #[test]
    fn execution_runtime_flattens_only_root_scope_proof_candidates() {
        let source = r#"@scope (:root) { .card { color: red; } } @scope (.theme) { .title { color: blue; } }"#;
        let execution = execute_transform_passes_on_source(
            source,
            &[TransformPassKind::ScopeFlatten, TransformPassKind::PrintCss],
        );

        assert_eq!(execution.mutation_count, 0);
        assert_eq!(execution.output_css, source);

        let accepted = execute_transform_passes_on_source(
            r#"@scope (:root) { .card { color: red; } }"#,
            &[TransformPassKind::ScopeFlatten, TransformPassKind::PrintCss],
        );
        assert_eq!(accepted.mutation_count, 1);
        assert_eq!(accepted.output_css, r#".card { color: red; }"#);
        assert_eq!(
            accepted.executed_pass_ids,
            vec!["scope-flatten", "print-css"]
        );
    }

    #[test]
    fn execution_runtime_flattens_layers_only_with_closed_bundle_context() {
        let source = r#"@layer theme { .card { color: red; } }"#;
        let planned = execute_transform_passes_on_source(
            source,
            &[TransformPassKind::LayerFlatten, TransformPassKind::PrintCss],
        );
        assert_eq!(planned.output_css, source);
        assert_eq!(planned.planned_only_pass_ids, vec!["layer-flatten"]);

        let context = TransformExecutionContextV0 {
            closed_style_world: true,
            ..TransformExecutionContextV0::default()
        };
        let execution = execute_transform_passes_on_source_with_dialect_and_context(
            source,
            StyleDialect::Css,
            &[TransformPassKind::LayerFlatten, TransformPassKind::PrintCss],
            &context,
        );
        assert_eq!(execution.mutation_count, 1);
        assert_eq!(execution.output_css, r#".card { color: red; }"#);
        assert_eq!(
            execution.executed_pass_ids,
            vec!["layer-flatten", "print-css"]
        );
    }

    #[test]
    fn execution_runtime_evaluates_literal_media_branches() {
        let source = r#"@media all { .a { color: red; } } @media not all { .b { color: blue; } } @media (max-width: 0px) { .zero { color: red; } } @media screen { .c { color: green; } } @supports (display: grid) { @media all { @media all { .d { color: black; } } } }"#;
        let execution = execute_transform_passes_on_source(
            source,
            &[
                TransformPassKind::MediaStaticEval,
                TransformPassKind::PrintCss,
            ],
        );

        assert_eq!(execution.mutation_count, 5);
        assert_eq!(
            execution.output_css,
            r#".a { color: red; }   @media screen { .c { color: green; } } @supports (display: grid) { .d { color: black; } }"#
        );
        assert_eq!(
            execution.executed_pass_ids,
            vec!["media-static-eval", "print-css"]
        );
    }

    #[test]
    fn execution_runtime_evaluates_simple_supports_branches_with_cascade_witness() {
        let source = r#"@supports (display: grid) { .a { display: grid; } } @supports not (display: grid) { .b { display: block; } } @supports (display: grid) and (color: red) { .c { color: red; } } @media all { @supports (display: grid) { @supports (display: grid) { .d { display: grid; } } } }"#;
        let execution = execute_transform_passes_on_source(
            source,
            &[
                TransformPassKind::SupportsStaticEval,
                TransformPassKind::PrintCss,
            ],
        );

        assert_eq!(execution.mutation_count, 5);
        assert_eq!(
            execution.output_css,
            r#".a { display: grid; }  .c { color: red; } @media all { .d { display: grid; } }"#
        );
        assert_eq!(
            execution.executed_pass_ids,
            vec!["supports-static-eval", "print-css"]
        );
    }

    #[test]
    fn execution_runtime_reduces_simple_same_unit_calc_values() {
        let source = r#".card { width: calc(1px + 2px); height: calc(10rem - 2rem); margin: calc(1px + 2rem); color: calc(1 + 2); gap: calc(.5rem+.25rem); inset: calc(1px - -2px); }"#;
        let execution = execute_transform_passes_on_source(
            source,
            &[
                TransformPassKind::CalcReduction,
                TransformPassKind::PrintCss,
            ],
        );

        assert_eq!(execution.mutation_count, 5);
        assert_eq!(
            execution.output_css,
            r#".card { width: 3px; height: 8rem; margin: calc(1px + 2rem); color: 3; gap: 0.75rem; inset: 3px; }"#
        );
        assert_eq!(
            execution.executed_pass_ids,
            vec!["calc-reduction", "print-css"]
        );
    }

    #[test]
    fn execution_runtime_resolves_unique_static_root_custom_properties() {
        let source = r#":root { --brand: red; --gap: 2rem; --alias: var(--brand); --dynamic: var(--alias); --fallback: var(--missing, blue); --dup: red; --dup: blue; --cycle-a: var(--cycle-b); --cycle-b: var(--cycle-a); } .card { color: var(--brand); margin: var(--gap); border-color: var(--missing, blue); background: var(--dup); outline-color: var(--dynamic); text-decoration-color: var(--fallback); caret-color: var(--cycle-a, green); box-shadow: 0 0 1px var(--gap); filter: drop-shadow(var(--missing, blue) 0 0); } @media screen { .card { color: var(--dynamic); } }"#;
        let execution = execute_transform_passes_on_source(
            source,
            &[
                TransformPassKind::StaticVarSubstitution,
                TransformPassKind::PrintCss,
            ],
        );

        assert_eq!(execution.mutation_count, 9);
        assert_eq!(
            execution.output_css,
            r#":root { --brand: red; --gap: 2rem; --alias: var(--brand); --dynamic: var(--alias); --fallback: var(--missing, blue); --dup: red; --dup: blue; --cycle-a: var(--cycle-b); --cycle-b: var(--cycle-a); } .card { color: red; margin: 2rem; border-color: blue; background: var(--dup); outline-color: red; text-decoration-color: blue; caret-color: green; box-shadow: 0 0 1px 2rem; filter: drop-shadow(blue 0 0); } @media screen { .card { color: red; } }"#
        );
        assert_eq!(
            execution.executed_pass_ids,
            vec!["custom-property-static-resolve", "print-css"]
        );
    }

    #[test]
    fn execution_runtime_resolves_static_local_css_modules_values() {
        let source = r#"@value primary: #fff; @value spacing: 8px; @value alias: primary; @value modulePath: "./tokens.module.css"; @value dup: red; @value dup: blue; .btn { color: primary; margin: spacing; background: alias; border-color: dup; }"#;
        let execution = execute_transform_passes_on_source(
            source,
            &[
                TransformPassKind::ValueResolution,
                TransformPassKind::PrintCss,
            ],
        );

        assert_eq!(execution.mutation_count, 6);
        assert_eq!(
            execution.output_css,
            r#"   @value modulePath: "./tokens.module.css"; @value dup: red; @value dup: blue; .btn { color: #fff; margin: 8px; background: #fff; border-color: dup; }"#
        );
        assert_eq!(
            execution.executed_pass_ids,
            vec!["value-resolution", "print-css"]
        );
    }

    #[test]
    fn execution_runtime_removes_dead_branches_through_semantic_pass_surfaces() {
        let source = r#"@media not all { .dead { color: red; } } @supports (display: grid) { .grid { display: grid; } } @supports (display: -ms-grid) { .ms { display: -ms-grid; } } @supports (display: grid) and (color: red) { .conjunction { color: red; } }"#;
        let execution = execute_transform_passes_on_source(
            source,
            &[
                TransformPassKind::DeadMediaBranchRemoval,
                TransformPassKind::DeadSupportsBranchRemoval,
                TransformPassKind::PrintCss,
            ],
        );

        assert_eq!(execution.mutation_count, 4);
        assert_eq!(
            execution.output_css,
            r#" .grid { display: grid; }  .conjunction { color: red; }"#
        );
        assert_eq!(
            execution.executed_pass_ids,
            vec![
                "dead-media-branch-removal",
                "dead-supports-branch-removal",
                "print-css"
            ]
        );
    }

    #[test]
    fn execution_runtime_removes_dark_media_branches_with_workspace_context() {
        let source = r#"@media (prefers-color-scheme: dark) { .dark { color: white; } } @media (prefers-color-scheme: light) { .light { color: black; } } @media screen and (prefers-color-scheme: dark) { .screen-dark { color: white; } }"#;
        let context = TransformExecutionContextV0 {
            drop_dark_mode_media_queries: true,
            ..TransformExecutionContextV0::default()
        };
        let execution = execute_transform_passes_on_source_with_dialect_and_context(
            source,
            StyleDialect::Css,
            &[
                TransformPassKind::DeadMediaBranchRemoval,
                TransformPassKind::PrintCss,
            ],
            &context,
        );

        assert_eq!(execution.mutation_count, 2);
        assert_eq!(
            execution.output_css,
            r#" @media (prefers-color-scheme: light) { .light { color: black; } } "#
        );
        assert!(!execution.output_css.contains("prefers-color-scheme: dark"));
    }

    #[test]
    fn execution_runtime_keeps_keyframe_tree_shaking_planned_without_closed_world_context() {
        let source = r#"@keyframes unused { to { opacity: 1; } } .btn { color: red; }"#;
        let execution = execute_transform_passes_on_source(
            source,
            &[
                TransformPassKind::TreeShakeKeyframes,
                TransformPassKind::PrintCss,
            ],
        );

        assert_eq!(execution.output_css, source);
        assert_eq!(execution.mutation_count, 0);
        assert_eq!(execution.executed_pass_ids, vec!["print-css"]);
        assert_eq!(
            execution.planned_only_pass_ids,
            vec!["tree-shake-keyframes"]
        );
    }

    #[test]
    fn execution_runtime_tree_shakes_keyframes_with_closed_world_context() {
        let source = r#"@-webkit-keyframes fade { to { opacity: 1; } } @keyframes fade { to { opacity: 1; } } @-webkit-keyframes spin { to { transform: rotate(1turn); } } @keyframes spin { to { transform: rotate(1turn); } } @-webkit-keyframes dead { to { opacity: 0; } } @keyframes dead { to { opacity: 0; } } .btn { animation: 1s ease fade; }"#;
        let context = TransformExecutionContextV0 {
            closed_style_world: true,
            reachable_keyframe_names: vec!["spin".to_string()],
            ..TransformExecutionContextV0::default()
        };
        let execution = execute_transform_passes_on_source_with_dialect_and_context(
            source,
            StyleDialect::Css,
            &[
                TransformPassKind::TreeShakeKeyframes,
                TransformPassKind::PrintCss,
            ],
            &context,
        );

        assert_eq!(execution.mutation_count, 2);
        assert_eq!(
            execution.output_css,
            r#"@-webkit-keyframes fade { to { opacity: 1; } } @keyframes fade { to { opacity: 1; } } @-webkit-keyframes spin { to { transform: rotate(1turn); } } @keyframes spin { to { transform: rotate(1turn); } }   .btn { animation: 1s ease fade; }"#
        );
        assert_eq!(
            execution.executed_pass_ids,
            vec!["tree-shake-keyframes", "print-css"]
        );
        assert_eq!(execution.semantic_removals.len(), 2);
        assert!(
            execution
                .semantic_removals
                .iter()
                .all(|removal| removal.symbol_kind == "keyframes")
        );
        assert!(
            execution
                .semantic_removals
                .iter()
                .any(|removal| removal.name == "dead" && removal.pass_id == "tree-shake-keyframes")
        );
    }

    #[test]
    fn execution_runtime_tree_shakes_quoted_keyframes_with_closed_world_context() {
        let source = r#"@keyframes "slide" { to { opacity: 1; } } @keyframes "ghost" { to { opacity: 0; } } .btn { animation-name: "slide"; } .alt { animation: "slide" 1s ease; }"#;
        let context = TransformExecutionContextV0 {
            closed_style_world: true,
            ..TransformExecutionContextV0::default()
        };
        let execution = execute_transform_passes_on_source_with_dialect_and_context(
            source,
            StyleDialect::Css,
            &[
                TransformPassKind::TreeShakeKeyframes,
                TransformPassKind::PrintCss,
            ],
            &context,
        );

        assert_eq!(execution.mutation_count, 1);
        assert_eq!(
            execution.output_css,
            r#"@keyframes "slide" { to { opacity: 1; } }  .btn { animation-name: "slide"; } .alt { animation: "slide" 1s ease; }"#
        );
        assert_eq!(
            execution.executed_pass_ids,
            vec!["tree-shake-keyframes", "print-css"]
        );
    }

    #[test]
    fn execution_runtime_tree_shakes_class_owned_rules_with_closed_world_context() {
        let source = r#".used { color: red; } .dead { color: blue; } .dead:hover { color: green; } button.other-dead { color: black; } .also-dead, .other-dead { color: black; } .used .child { color: purple; } :global(.external) { color: gray; } @media (min-width: 1px) { .media-dead { color: orange; } .used { color: brown; } }"#;
        let context = TransformExecutionContextV0 {
            closed_style_world: true,
            reachable_class_names: vec!["used".to_string()],
            ..TransformExecutionContextV0::default()
        };
        let execution = execute_transform_passes_on_source_with_dialect_and_context(
            source,
            StyleDialect::Css,
            &[
                TransformPassKind::TreeShakeClass,
                TransformPassKind::PrintCss,
            ],
            &context,
        );

        assert_eq!(execution.mutation_count, 5);
        assert!(execution.output_css.contains(".used { color: red; }"));
        assert!(
            execution
                .output_css
                .contains(".used .child { color: purple; }")
        );
        assert!(
            execution
                .output_css
                .contains("@media (min-width: 1px) {  .used { color: brown; } }")
        );
        assert!(
            execution
                .output_css
                .contains(":global(.external) { color: gray; }")
        );
        assert!(!execution.output_css.contains(".dead {"));
        assert!(!execution.output_css.contains(".dead:hover"));
        assert!(!execution.output_css.contains("button.other-dead"));
        assert!(!execution.output_css.contains(".also-dead"));
        assert!(!execution.output_css.contains(".other-dead"));
        assert!(!execution.output_css.contains(".media-dead"));
        assert_eq!(
            execution.executed_pass_ids,
            vec!["tree-shake-class", "print-css"]
        );
        assert_eq!(execution.semantic_removals.len(), 5);
        assert!(execution.semantic_removals.iter().any(|removal| {
            removal.symbol_kind == "class"
                && removal.name == "also-dead,other-dead"
                && removal.pass_id == "tree-shake-class"
                && removal
                    .derivation_steps
                    .contains(&"symbolNotMarkedReachable")
        }));
    }

    #[test]
    fn execution_runtime_tree_shakes_local_values_with_closed_world_context() {
        let source = r#"@value used: red; @value dead: blue; @value alias: used; @value shadow: 0 0 4px used; @value bp: 40rem; @value deadAlias: dead; @value deadShadow: 0 0 4px dead; @value deadBp: 50rem; .btn { color: used; background: alias; box-shadow: shadow; } @media (min-width: bp) { .btn { color: red; } }"#;
        let context = TransformExecutionContextV0 {
            closed_style_world: true,
            ..TransformExecutionContextV0::default()
        };
        let execution = execute_transform_passes_on_source_with_dialect_and_context(
            source,
            StyleDialect::Css,
            &[
                TransformPassKind::TreeShakeValue,
                TransformPassKind::PrintCss,
            ],
            &context,
        );

        assert_eq!(execution.mutation_count, 4);
        assert!(execution.output_css.contains("@value used: red;"));
        assert!(execution.output_css.contains("@value alias: used;"));
        assert!(
            execution
                .output_css
                .contains("@value shadow: 0 0 4px used;")
        );
        assert!(execution.output_css.contains("@value bp: 40rem;"));
        assert!(execution.output_css.contains("box-shadow: shadow;"));
        assert!(execution.output_css.contains("@media (min-width: bp)"));
        assert!(!execution.output_css.contains("@value dead:"));
        assert!(!execution.output_css.contains("@value deadAlias:"));
        assert!(!execution.output_css.contains("@value deadShadow:"));
        assert!(!execution.output_css.contains("@value deadBp:"));
        assert_eq!(
            execution.executed_pass_ids,
            vec!["tree-shake-value", "print-css"]
        );
        assert_eq!(
            execution
                .semantic_removals
                .iter()
                .map(|removal| removal.name.as_str())
                .collect::<Vec<_>>(),
            vec!["dead", "deadAlias", "deadShadow", "deadBp"]
        );
    }

    #[test]
    fn execution_runtime_tree_shakes_custom_properties_with_closed_world_context() {
        let source = r#":root { --used: VAR(--alias); --alias: red; --dead: VAR(--dead-dep); --dead-dep: blue; color: VAR(--used); } .btn { color: var(--external); }"#;
        let context = TransformExecutionContextV0 {
            closed_style_world: true,
            reachable_custom_property_names: vec!["--external".to_string()],
            ..TransformExecutionContextV0::default()
        };
        let execution = execute_transform_passes_on_source_with_dialect_and_context(
            source,
            StyleDialect::Css,
            &[
                TransformPassKind::TreeShakeCustomProperty,
                TransformPassKind::PrintCss,
            ],
            &context,
        );

        assert_eq!(execution.mutation_count, 2);
        assert!(execution.output_css.contains("--used: VAR(--alias);"));
        assert!(execution.output_css.contains("--alias: red;"));
        assert!(execution.output_css.contains("color: VAR(--used);"));
        assert!(execution.output_css.contains("color: var(--external);"));
        assert!(!execution.output_css.contains("--dead:"));
        assert!(!execution.output_css.contains("--dead-dep:"));
        assert_eq!(
            execution.executed_pass_ids,
            vec!["tree-shake-custom-property", "print-css"]
        );
        assert_eq!(
            execution
                .semantic_removals
                .iter()
                .map(|removal| (removal.symbol_kind, removal.name.as_str()))
                .collect::<Vec<_>>(),
            vec![
                ("customProperty", "--dead"),
                ("customProperty", "--dead-dep")
            ]
        );
    }

    #[test]
    fn execution_runtime_uses_dialect_lexer_for_scss_silent_comments() {
        let source = ".a { // remove\n  color: red;\n  content: \"// keep\";\n}";
        let execution = execute_transform_passes_on_source_with_dialect(
            source,
            StyleDialect::Scss,
            &[TransformPassKind::CommentStrip],
        );

        assert_eq!(execution.mutation_count, 1);
        assert_eq!(
            execution.output_css,
            ".a { \n  color: red;\n  content: \"// keep\";\n}"
        );
    }
}
