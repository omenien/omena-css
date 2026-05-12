//! Transform pass registry and DAG planner for the post-v5 omena-css track.
//!
//! This crate consumes `omena-transform-cst` contracts. It does not duplicate
//! the pass catalog; its job is to register every P01-P40 pass and produce a
//! DAG-respecting execution plan for downstream transform crates.

use omena_cascade::{
    BoxLonghandInputV0, CascadeValue, CustomPropertyEnv, StaticSupportsAssumptionV0,
    StaticSupportsEvalVerdictV0, evaluate_static_supports_condition,
    prove_box_shorthand_combination, substitute_custom_properties,
};
use omena_parser::{StyleDialect, lex};
use omena_syntax::SyntaxKind;
use omena_transform_cst::{
    TRANSFORM_PASS_CATALOG_LEN, TransformDagEdgeV0, TransformLayer, TransformPassContractV0,
    TransformPassKind, all_transform_pass_kinds, default_transform_dag_edges,
    default_transform_pass_contracts,
};
use serde::Serialize;

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
    pub outcomes: Vec<TransformPassExecutionOutcomeV0>,
    pub pass_plan: TransformPassPlanV0,
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
        implemented_mutation_pass_ids: implemented_mutation_pass_ids(),
        next_surfaces: vec![
            "remainingPassMutationEngines",
            "transformSalsaQueries",
            "omena-transform-bundle",
            "omena-transform-print",
        ],
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
    let pass_plan = plan_transform_passes(requested);
    let requested_pass_ids = requested.iter().map(|pass| pass.id()).collect::<Vec<_>>();
    let ordered_pass_ids = pass_plan.ordered_pass_ids.clone();
    let mut output_css = source.to_string();
    let mut outcomes = Vec::new();

    for pass_id in &ordered_pass_ids {
        let pass = transform_pass_kind_from_id(pass_id);
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
                    detail: "normalized zero length units only inside property contexts that accept unitless zero",
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
                    detail: "compressed declaration-leading hex color tokens",
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
                let (next_css, mutation_count) =
                    dedupe_adjacent_exact_css_rules(&output_css, dialect);
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
                    detail: "removed adjacent exact duplicate ordinary rules only",
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
                    detail: "merged adjacent same-selector ordinary rules without reordering declarations",
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
                    detail: "merged adjacent ordinary rules with identical declaration blocks",
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
                    detail: "lowered whole-value srgb color-mix() declarations with static color operands",
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
                    detail: "lowered in-gamut whole-value oklab()/oklch() color declarations to srgb",
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
                    detail: "lowered whole-value color(srgb ...) declarations with static channels",
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
                    detail: "unwrapped simple single-depth nested ordinary rules",
                }
            }
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
                    detail: "resolved whole-value var() references from unique literal :root custom properties",
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
                    detail: "removed top-level ordinary empty rules with no comments or at-rule semantics",
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
            Some(_) | None => TransformPassExecutionOutcomeV0 {
                pass_id,
                status: TransformPassRuntimeStatus::PlannedOnly,
                input_byte_len,
                output_byte_len: output_css.len(),
                mutation_count: 0,
                provenance_preserved: true,
                detail: "registered in DAG planner; mutation engine not implemented yet",
            },
        };
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
        outcomes,
        pass_plan,
    }
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
        TransformPassKind::SupportsStaticEval.id(),
        TransformPassKind::MediaStaticEval.id(),
        TransformPassKind::StaticVarSubstitution.id(),
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
        let next_index = match remaining
            .iter()
            .position(|candidate| !has_incoming_edge_from_remaining(*candidate, &remaining))
        {
            Some(index) => index,
            None => 0,
        };
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

fn dedupe_adjacent_exact_css_rules(source: &str, dialect: StyleDialect) -> (String, usize) {
    dedupe_adjacent_exact_css_rules_with_lexer(source, dialect)
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

fn evaluate_static_supports_rules(source: &str, dialect: StyleDialect) -> (String, usize) {
    evaluate_static_supports_rules_with_lexer(source, dialect)
}

fn evaluate_static_media_rules(source: &str, dialect: StyleDialect) -> (String, usize) {
    evaluate_static_media_rules_with_lexer(source, dialect)
}

fn substitute_static_css_custom_properties(source: &str, dialect: StyleDialect) -> (String, usize) {
    substitute_static_css_custom_properties_with_lexer(source, dialect)
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
    let rules = collect_top_level_ordinary_rule_slices(source, tokens);
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
                let Some(replacement_value) = parse_color_mix_value(&declaration.value) else {
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
                let Some(replacement_value) = parse_oklab_oklch_value(&declaration.value) else {
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
                let Some(replacement_value) = parse_color_function_value(&declaration.value) else {
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
                let Some(physical_property) =
                    physical_property_for_logical_property(&declaration.property, direction)
                else {
                    continue;
                };
                replacements.push((
                    declaration.start,
                    declaration.end,
                    format!("{physical_property}: {};", declaration.value),
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
    if parent_selector.is_empty() || parent_selector.contains(',') {
        return None;
    }

    let declarations =
        collect_simple_declarations_in_block(tokens, block_start_index, block_end_index);
    let nested_rules =
        collect_direct_nested_rule_slices(source, tokens, block_start_index, block_end_index)?;
    if nested_rules.is_empty() {
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
        let selector = expand_nested_selector(&parent_selector, &nested_rule.selector)?;
        rule_texts.push(format!("{} {{ {} }}", selector, nested_rule.block));
    }

    Some(rule_texts.join(" "))
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct NestedRuleSlice {
    selector: String,
    block: String,
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
            if tokens[index + 1..nested_close_index].iter().any(|token| {
                matches!(token.kind, SyntaxKind::LeftBrace | SyntaxKind::RightBrace)
                    || is_comment_token(token.kind)
            }) {
                return None;
            }
            let selector_start = first_non_trivia_token_start(tokens, segment_start_index, index)?;
            let selector = source[selector_start..token_start(&tokens[index])]
                .trim()
                .to_string();
            if selector.is_empty() || selector.starts_with('@') || selector.contains(',') {
                return None;
            }
            let block = source[token_end(&tokens[index])..token_start(&tokens[nested_close_index])]
                .trim()
                .to_string();
            if block.is_empty() {
                return None;
            }
            nested_rules.push(NestedRuleSlice { selector, block });
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

fn expand_nested_selector(parent_selector: &str, nested_selector: &str) -> Option<String> {
    if nested_selector.contains(',') {
        return None;
    }
    if nested_selector.contains('&') {
        Some(nested_selector.replace('&', parent_selector))
    } else {
        Some(format!("{parent_selector} {nested_selector}"))
    }
}

fn evaluate_static_supports_rules_with_lexer(
    source: &str,
    dialect: StyleDialect,
) -> (String, usize) {
    let lexed = lex(source, dialect);
    let tokens = lexed.tokens();
    let mut replacements = Vec::new();
    let mut depth = 0usize;
    let mut index = 0;

    while index < tokens.len() {
        match tokens[index].kind {
            SyntaxKind::AtKeyword
                if depth == 0 && tokens[index].text.eq_ignore_ascii_case("@supports") =>
            {
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
            SyntaxKind::LeftBrace => depth += 1,
            SyntaxKind::RightBrace => depth = depth.saturating_sub(1),
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

fn evaluate_static_media_rules_with_lexer(source: &str, dialect: StyleDialect) -> (String, usize) {
    let lexed = lex(source, dialect);
    let tokens = lexed.tokens();
    let mut replacements = Vec::new();
    let mut depth = 0usize;
    let mut index = 0;

    while index < tokens.len() {
        match tokens[index].kind {
            SyntaxKind::AtKeyword
                if depth == 0 && tokens[index].text.eq_ignore_ascii_case("@media") =>
            {
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
                let replacement = match condition.as_str() {
                    "all" => source[token_end(&tokens[block_start_index])
                        ..token_start(&tokens[block_end_index])]
                        .trim()
                        .to_string(),
                    "not all" => String::new(),
                    _ => {
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
            SyntaxKind::LeftBrace => depth += 1,
            SyntaxKind::RightBrace => depth = depth.saturating_sub(1),
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

fn substitute_static_css_custom_properties_with_lexer(
    source: &str,
    dialect: StyleDialect,
) -> (String, usize) {
    let lexed = lex(source, dialect);
    let tokens = lexed.tokens();
    let rules = collect_top_level_ordinary_rule_slices(source, tokens);
    let env = collect_static_root_custom_property_env(tokens, &rules);
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
            let Some(var_value) = parse_static_var_value(&declaration.value) else {
                continue;
            };
            let resolved = substitute_custom_properties(&var_value, &env);
            let CascadeValue::Literal(resolved_value) = resolved else {
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
            if declaration.value.contains("var(") {
                continue;
            }
            env.insert(
                declaration.property,
                CascadeValue::Literal(declaration.value),
            );
        }
    }

    env
}

fn parse_static_var_value(value: &str) -> Option<CascadeValue> {
    let arguments = parse_whole_function_value_arguments(value, "var")?;
    match arguments.as_slice() {
        [name] if name.starts_with("--") => Some(CascadeValue::Var {
            name: name.clone(),
            fallback: None,
        }),
        [name, fallback] if name.starts_with("--") && !fallback.contains("var(") => {
            Some(CascadeValue::Var {
                name: name.clone(),
                fallback: Some(Box::new(CascadeValue::Literal(fallback.clone()))),
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
    let parts = inner.split_whitespace().collect::<Vec<_>>();
    let [left, operator, right] = parts.as_slice() else {
        return None;
    };
    if !matches!(*operator, "+" | "-") {
        return None;
    }

    let left = parse_numeric_value_with_unit(left)?;
    let right = parse_numeric_value_with_unit(right)?;
    if left.unit != right.unit {
        return None;
    }
    let value = match *operator {
        "+" => left.value + right.value,
        "-" => left.value - right.value,
        _ => return None,
    };
    Some(format!("{}{}", format_css_number(value), left.unit))
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct StaticColorMixStop {
    color: SrgbColor,
    percentage: Option<u8>,
}

fn parse_static_color_mix_stop(input: &str) -> Option<StaticColorMixStop> {
    let parts = input.split_whitespace().collect::<Vec<_>>();
    let (color_text, percentage) = match parts.as_slice() {
        [color] => (*color, None),
        [color, percentage] => (*color, Some(parse_percentage_integer(percentage)?)),
        _ => return None,
    };

    Some(StaticColorMixStop {
        color: parse_static_srgb_color(color_text)?,
        percentage,
    })
}

fn parse_percentage_integer(text: &str) -> Option<u8> {
    let number = text.strip_suffix('%')?;
    let value = number.parse::<u8>().ok()?;
    (value <= 100).then_some(value)
}

fn color_mix_weights(first: Option<u8>, second: Option<u8>) -> Option<(f64, f64)> {
    match (first, second) {
        (None, None) => Some((0.5, 0.5)),
        (Some(first), None) => Some((f64::from(first) / 100.0, f64::from(100 - first) / 100.0)),
        (None, Some(second)) => Some((f64::from(100 - second) / 100.0, f64::from(second) / 100.0)),
        (Some(first), Some(second)) if first + second == 100 => {
            Some((f64::from(first) / 100.0, f64::from(second) / 100.0))
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
        "green" => Some(SrgbColor {
            red: 0,
            green: 128,
            blue: 0,
        }),
        "red" => Some(SrgbColor {
            red: 255,
            green: 0,
            blue: 0,
        }),
        "white" => Some(SrgbColor {
            red: 255,
            green: 255,
            blue: 255,
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
    let parts = split_ascii_space_separated_color_args(inner)?;
    let [space, red, green, blue] = parts.as_slice() else {
        return None;
    };
    if !space.eq_ignore_ascii_case("srgb") {
        return None;
    }
    Some(
        SrgbColor {
            red: parse_srgb_component(red)?,
            green: parse_srgb_component(green)?,
            blue: parse_srgb_component(blue)?,
        }
        .to_css_rgb(),
    )
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
    let value = text
        .strip_suffix("deg")
        .map_or_else(|| parse_plain_f64(text), parse_plain_f64)?;
    value.is_finite().then_some(value)
}

fn parse_plain_f64(text: &str) -> Option<f64> {
    if text.contains('%') {
        return None;
    }
    text.parse::<f64>().ok().filter(|value| value.is_finite())
}

fn parse_srgb_component(text: &str) -> Option<u8> {
    let value = if let Some(percent) = text.strip_suffix('%') {
        parse_plain_f64(percent)? / 100.0
    } else {
        parse_plain_f64(text)?
    };
    if !(0.0..=1.0).contains(&value) {
        return None;
    }
    Some((value * 255.0).round().clamp(0.0, 255.0) as u8)
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
        _ => None,
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
    let insertions = collect_vendor_prefix_insertions(tokens);
    if insertions.is_empty() {
        return (source.to_string(), 0);
    }

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

fn collect_vendor_prefix_insertions(tokens: &[omena_parser::LexedToken]) -> Vec<(usize, String)> {
    let mut insertions = Vec::new();
    let mut index = 0;

    while index < tokens.len() {
        if tokens[index].kind == SyntaxKind::LeftBrace
            && let Some(close_index) = matching_right_brace_index(tokens, index)
        {
            let declarations = collect_simple_declarations_in_block(tokens, index, close_index);
            for declaration in &declarations {
                if let Some(prefixed_property) = prefixed_property_for(&declaration.property)
                    && !declarations
                        .iter()
                        .any(|candidate| candidate.property == prefixed_property)
                {
                    insertions.push((
                        declaration.start,
                        format!("{prefixed_property}: {}; ", declaration.value),
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

fn prefixed_property_for(property: &str) -> Option<&'static str> {
    match property {
        "appearance" => Some("-webkit-appearance"),
        "backdrop-filter" => Some("-webkit-backdrop-filter"),
        "user-select" => Some("-webkit-user-select"),
        _ => None,
    }
}

fn merge_adjacent_same_block_css_selectors_with_lexer(
    source: &str,
    dialect: StyleDialect,
) -> (String, usize) {
    let lexed = lex(source, dialect);
    let tokens = lexed.tokens();
    let rules = collect_top_level_ordinary_rule_slices(source, tokens);
    let mut replacements = Vec::new();
    let mut index = 0;

    while index + 1 < rules.len() {
        let current = &rules[index];
        let next = &rules[index + 1];
        if current.selector != next.selector
            && current.block == next.block
            && rule_gap_is_whitespace_only(tokens, current.end, next.start)
        {
            replacements.push((
                current.start,
                next.end,
                format!(
                    "{}, {} {{ {} }}",
                    current.selector, next.selector, current.block
                ),
            ));
            index += 2;
        } else {
            index += 1;
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

fn merge_adjacent_same_selector_css_rules_with_lexer(
    source: &str,
    dialect: StyleDialect,
) -> (String, usize) {
    let lexed = lex(source, dialect);
    let tokens = lexed.tokens();
    let rules = collect_top_level_ordinary_rule_slices(source, tokens);
    let mut replacements = Vec::new();
    let mut index = 0;

    while index + 1 < rules.len() {
        let current = &rules[index];
        let next = &rules[index + 1];
        if current.selector == next.selector
            && current.block != next.block
            && rule_gap_is_whitespace_only(tokens, current.end, next.start)
        {
            replacements.push((
                current.start,
                next.end,
                format!(
                    "{} {{ {} {} }}",
                    current.selector, current.block, next.block
                ),
            ));
            index += 2;
        } else {
            index += 1;
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

#[derive(Debug, Clone, PartialEq, Eq)]
struct SimpleRuleSlice {
    selector: String,
    block: String,
    start: usize,
    end: usize,
    block_start: usize,
    block_end: usize,
}

fn dedupe_adjacent_exact_css_rules_with_lexer(
    source: &str,
    dialect: StyleDialect,
) -> (String, usize) {
    let lexed = lex(source, dialect);
    let tokens = lexed.tokens();
    let rules = collect_top_level_ordinary_rule_slices(source, tokens);
    let mut ranges = Vec::new();

    for pair in rules.windows(2) {
        let [previous, current] = pair else {
            continue;
        };
        if previous.selector == current.selector
            && previous.block == current.block
            && rule_gap_is_whitespace_only(tokens, previous.end, current.start)
        {
            ranges.push((current.start, current.end));
        }
    }

    if ranges.is_empty() {
        return (source.to_string(), 0);
    }

    let mut output = String::with_capacity(source.len());
    let mut cursor = 0;
    for (start, end) in &ranges {
        if *start > cursor {
            output.push_str(&source[cursor..*start]);
        }
        cursor = *end;
    }
    if cursor < source.len() {
        output.push_str(&source[cursor..]);
    }

    (output, ranges.len())
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
        if tokens[index].kind == SyntaxKind::LeftBrace {
            if let Some(close_index) = matching_right_brace_index(tokens, index) {
                ranges.extend(collect_box_shorthand_replacements_in_block(
                    tokens,
                    index,
                    close_index,
                ));
                index = close_index + 1;
                continue;
            }
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

        if tokens[index].kind == SyntaxKind::LeftBrace {
            if let Some(close_index) = matching_right_brace_index(tokens, index) {
                index = close_index + 1;
                continue;
            }
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
    let lexed = lex(source, dialect);
    let tokens = lexed.tokens();
    let ranges = collect_top_level_empty_rule_ranges(tokens);
    if ranges.is_empty() {
        return (source.to_string(), 0);
    }

    let mut output = String::with_capacity(source.len());
    let mut cursor = 0;
    for (start, end) in &ranges {
        if *start > cursor {
            output.push_str(&source[cursor..*start]);
        }
        cursor = *end;
    }
    if cursor < source.len() {
        output.push_str(&source[cursor..]);
    }

    (output, ranges.len())
}

fn collect_top_level_empty_rule_ranges(tokens: &[omena_parser::LexedToken]) -> Vec<(usize, usize)> {
    let mut ranges = Vec::new();
    let mut depth = 0usize;
    let mut top_level_prelude_start = 0usize;
    let mut index = 0;

    while index < tokens.len() {
        match tokens[index].kind {
            SyntaxKind::LeftBrace => {
                if depth == 0 {
                    if let Some(close_index) = matching_right_brace_index(tokens, index)
                        && is_empty_rule_block(tokens, index + 1, close_index)
                        && is_ordinary_top_level_rule_prelude(
                            tokens,
                            top_level_prelude_start,
                            index,
                        )
                        && let Some(start) =
                            first_non_trivia_token_start(tokens, top_level_prelude_start, index)
                    {
                        let end = token_end(&tokens[close_index]);
                        ranges.push((start, end));
                        index = close_index + 1;
                        top_level_prelude_start = index;
                        continue;
                    }
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

    ranges
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

fn is_ordinary_top_level_rule_prelude(
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
    let arguments = split_top_level_selector_arguments(inner_tokens)?;
    if arguments.is_empty() {
        return None;
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
    let lexed = lex(source, dialect);
    let tokens = lexed.tokens();
    let mut output = String::with_capacity(source.len());
    let mut mutation_count = 0;

    for (index, token) in tokens.iter().enumerate() {
        let replacement = if token.kind == SyntaxKind::Hash
            && previous_non_comment_token_kind(tokens, index) == Some(SyntaxKind::Colon)
        {
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

        let replacement = if token.kind == SyntaxKind::Dimension
            && active_property
                .as_deref()
                .is_some_and(is_zero_length_unit_property)
        {
            normalize_zero_length_dimension_token(&token.text)
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
        "margin"
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
            | "gap"
            | "row-gap"
            | "column-gap"
    )
}

fn normalize_zero_length_dimension_token(text: &str) -> Option<String> {
    let split = numeric_prefix_end(text)?;
    let (number, unit) = text.split_at(split);
    if !is_zero_number_prefix(number) || !is_css_length_unit(unit) {
        return None;
    }

    Some("0".to_string())
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
    let Some((before_dot, after_dot)) = unsigned.split_once('.') else {
        return number.to_string();
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

    format!("{sign}{compressed_unsigned}")
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
        TransformPassRuntimeStatus, execute_transform_passes_on_source,
        execute_transform_passes_on_source_with_dialect, plan_transform_passes,
        summarize_omena_transform_passes_boundary,
    };
    use omena_parser::StyleDialect;
    use omena_transform_cst::{TRANSFORM_PASS_CATALOG_LEN, TransformPassKind};

    #[test]
    fn registry_covers_full_p01_to_p40_catalog() {
        let boundary = summarize_omena_transform_passes_boundary();

        assert_eq!(boundary.schema_version, "0");
        assert_eq!(boundary.product, "omena-transform-passes.boundary");
        assert_eq!(boundary.pass_count, TRANSFORM_PASS_CATALOG_LEN);
        assert!(boundary.full_catalog_registered);
        assert_eq!(boundary.semantic_aware_pass_count, 14);
        assert!(boundary.cascade_aware_pass_count >= 9);
        assert!(boundary.planner_enforces_dag_edges);
        assert!(boundary.execution_runtime_ready);
        assert_eq!(
            boundary.implemented_mutation_pass_ids,
            vec![
                "p01-whitespace-strip",
                "p02-comment-strip",
                "p03-number-compression",
                "p04-unit-normalization",
                "p05-color-compression",
                "p06-url-quote-strip",
                "p07-string-quote-normalize",
                "p08-selector-is-where-compression",
                "p09-shorthand-combining",
                "p10-rule-deduplication",
                "p11-rule-merging",
                "p12-selector-merging",
                "p13-empty-rule-removal",
                "p14-vendor-prefixing",
                "p15-light-dark-lowering",
                "p16-color-mix-lowering",
                "p17-oklch-oklab-lowering",
                "p18-color-function-lowering",
                "p19-logical-to-physical",
                "p20-nesting-unwrap",
                "p23-supports-static-eval",
                "p24-media-static-eval",
                "p32-custom-property-static-resolve",
                "p25-calc-reduction",
                "p40-print-css"
            ]
        );
        assert!(boundary.registry_entries.iter().any(|entry| {
            entry.contract.kind == TransformPassKind::TreeShakeClass
                && entry.module_family == "semantic-reachability"
        }));
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
                "p32-custom-property-static-resolve",
                "p25-calc-reduction",
                "p40-print-css"
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
                "p30-composes-resolution",
                "p29-css-modules-class-hashing",
                "p12-selector-merging"
            ]
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
            vec!["p02-comment-strip", "p40-print-css"]
        );
        assert_eq!(
            execution.planned_only_pass_ids,
            vec!["p29-css-modules-class-hashing"]
        );
        assert!(execution.provenance_preserved);
        assert_eq!(execution.pass_plan.violated_dag_edge_count, 0);
        assert!(execution.outcomes.iter().any(|outcome| {
            outcome.pass_id == "p02-comment-strip"
                && outcome.status == TransformPassRuntimeStatus::Applied
                && outcome.mutation_count == 1
        }));
        assert!(execution.outcomes.iter().any(|outcome| {
            outcome.pass_id == "p29-css-modules-class-hashing"
                && outcome.status == TransformPassRuntimeStatus::PlannedOnly
        }));
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
            vec!["p01-whitespace-strip", "p02-comment-strip", "p40-print-css"]
        );
    }

    #[test]
    fn execution_runtime_compresses_numeric_tokens_only() {
        let source =
            r#".a { width: 0.50rem; opacity: 1.0; margin: -0.25px 10.00%; content: "0.50"; }"#;
        let execution = execute_transform_passes_on_source(
            source,
            &[
                TransformPassKind::NumberCompression,
                TransformPassKind::PrintCss,
            ],
        );

        assert_eq!(execution.mutation_count, 4);
        assert_eq!(
            execution.output_css,
            r#".a { width: .5rem; opacity: 1; margin: -.25px 10%; content: "0.50"; }"#
        );
    }

    #[test]
    fn execution_runtime_normalizes_zero_length_units_with_property_context() {
        let source = r#".a { margin: 0px 0.0rem -0em; rotate: 0deg; animation-delay: 0s; --x: 0px; width: 10px; }"#;
        let execution = execute_transform_passes_on_source(
            source,
            &[
                TransformPassKind::UnitNormalization,
                TransformPassKind::PrintCss,
            ],
        );

        assert_eq!(execution.mutation_count, 3);
        assert_eq!(
            execution.output_css,
            r#".a { margin: 0 0 0; rotate: 0deg; animation-delay: 0s; --x: 0px; width: 10px; }"#
        );
        assert_eq!(
            execution.executed_pass_ids,
            vec!["p04-unit-normalization", "p40-print-css"]
        );
    }

    #[test]
    fn execution_runtime_compresses_declaration_leading_hex_colors_only() {
        let source = r#".a { color: #FFFFFF; box-shadow: 0 0 #AABBCC; } #FFFFFF { color: red; }"#;
        let execution = execute_transform_passes_on_source(
            source,
            &[
                TransformPassKind::ColorCompression,
                TransformPassKind::PrintCss,
            ],
        );

        assert_eq!(execution.mutation_count, 1);
        assert_eq!(
            execution.output_css,
            r#".a { color: #fff; box-shadow: 0 0 #AABBCC; } #FFFFFF { color: red; }"#
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
        let source = r#".a:is(.ready) { color: red; } .b:where(.x, .x) { color: blue; } .c:where(.y) { color: green; }"#;
        let execution = execute_transform_passes_on_source(
            source,
            &[
                TransformPassKind::SelectorIsWhereCompression,
                TransformPassKind::PrintCss,
            ],
        );

        assert_eq!(execution.mutation_count, 2);
        assert_eq!(
            execution.output_css,
            r#".a.ready { color: red; } .b:where(.x) { color: blue; } .c:where(.y) { color: green; }"#
        );
        assert_eq!(
            execution.executed_pass_ids,
            vec!["p08-selector-is-where-compression", "p40-print-css"]
        );
    }

    #[test]
    fn execution_runtime_removes_only_plain_top_level_empty_rules() {
        let source = r#".empty { } @media (min-width: 1px) { } .with-comment { /* keep */ } .filled { color: red; }"#;
        let execution = execute_transform_passes_on_source(
            source,
            &[
                TransformPassKind::EmptyRuleRemoval,
                TransformPassKind::PrintCss,
            ],
        );

        assert_eq!(execution.mutation_count, 1);
        assert_eq!(
            execution.output_css,
            r#" @media (min-width: 1px) { } .with-comment { /* keep */ } .filled { color: red; }"#
        );
        assert_eq!(
            execution.executed_pass_ids,
            vec!["p13-empty-rule-removal", "p40-print-css"]
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
            vec!["p09-shorthand-combining", "p40-print-css"]
        );
    }

    #[test]
    fn execution_runtime_removes_adjacent_exact_duplicate_rules_only() {
        let source =
            r#".a { color: red; } .a { color: red; } .b { color: red; } .a { color: red; }"#;
        let execution = execute_transform_passes_on_source(
            source,
            &[
                TransformPassKind::RuleDeduplication,
                TransformPassKind::PrintCss,
            ],
        );

        assert_eq!(execution.mutation_count, 1);
        assert_eq!(
            execution.output_css,
            r#".a { color: red; }  .b { color: red; } .a { color: red; }"#
        );
        assert_eq!(
            execution.executed_pass_ids,
            vec!["p10-rule-deduplication", "p40-print-css"]
        );
    }

    #[test]
    fn execution_runtime_merges_adjacent_same_selector_rules_only() {
        let source =
            r#".a { color: red; } .a { background: blue; } .b { color: red; } .a { border: 0; }"#;
        let execution = execute_transform_passes_on_source(
            source,
            &[TransformPassKind::RuleMerging, TransformPassKind::PrintCss],
        );

        assert_eq!(execution.mutation_count, 1);
        assert_eq!(
            execution.output_css,
            r#".a { color: red; background: blue; } .b { color: red; } .a { border: 0; }"#
        );
        assert_eq!(
            execution.executed_pass_ids,
            vec!["p11-rule-merging", "p40-print-css"]
        );
    }

    #[test]
    fn execution_runtime_merges_adjacent_same_block_selectors_only() {
        let source =
            r#".a { color: red; } .b { color: red; } .c { color: blue; } .d { color: red; }"#;
        let execution = execute_transform_passes_on_source(
            source,
            &[
                TransformPassKind::SelectorMerging,
                TransformPassKind::PrintCss,
            ],
        );

        assert_eq!(execution.mutation_count, 1);
        assert_eq!(
            execution.output_css,
            r#".a, .b { color: red; } .c { color: blue; } .d { color: red; }"#
        );
        assert_eq!(
            execution.executed_pass_ids,
            vec!["p12-selector-merging", "p40-print-css"]
        );
    }

    #[test]
    fn execution_runtime_adds_conservative_vendor_prefixes_when_absent() {
        let source = r#".a { user-select: none; -webkit-appearance: none; appearance: none; backdrop-filter: blur(2px); }"#;
        let execution = execute_transform_passes_on_source(
            source,
            &[
                TransformPassKind::VendorPrefixing,
                TransformPassKind::PrintCss,
            ],
        );

        assert_eq!(execution.mutation_count, 2);
        assert_eq!(
            execution.output_css,
            r#".a { -webkit-user-select: none; user-select: none; -webkit-appearance: none; appearance: none; -webkit-backdrop-filter: blur(2px); backdrop-filter: blur(2px); }"#
        );
        assert_eq!(
            execution.executed_pass_ids,
            vec!["p14-vendor-prefixing", "p40-print-css"]
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
            vec!["p15-light-dark-lowering", "p40-print-css"]
        );
    }

    #[test]
    fn execution_runtime_lowers_static_srgb_color_mix_declarations() {
        let source = r#".card { color: color-mix(in srgb, red 50%, blue 50%); background-color: color-mix(in srgb, #000, #fff 25%); border-color: color-mix(in oklab, red, blue); }"#;
        let execution = execute_transform_passes_on_source(
            source,
            &[
                TransformPassKind::ColorMixLowering,
                TransformPassKind::PrintCss,
            ],
        );

        assert_eq!(execution.mutation_count, 2);
        assert_eq!(
            execution.output_css,
            r#".card { color: rgb(128 0 128); background-color: rgb(64 64 64); border-color: color-mix(in oklab, red, blue); }"#
        );
        assert_eq!(
            execution.executed_pass_ids,
            vec!["p16-color-mix-lowering", "p40-print-css"]
        );
    }

    #[test]
    fn execution_runtime_lowers_in_gamut_oklab_oklch_declarations() {
        let source = r#".card { color: oklab(1 0 0); background-color: oklch(0% 0 0deg); border-color: oklch(70% 0.4 40deg); }"#;
        let execution = execute_transform_passes_on_source(
            source,
            &[
                TransformPassKind::OklchOklabLowering,
                TransformPassKind::PrintCss,
            ],
        );

        assert_eq!(execution.mutation_count, 2);
        assert_eq!(
            execution.output_css,
            r#".card { color: rgb(255 255 255); background-color: rgb(0 0 0); border-color: oklch(70% 0.4 40deg); }"#
        );
        assert_eq!(
            execution.executed_pass_ids,
            vec!["p17-oklch-oklab-lowering", "p40-print-css"]
        );
    }

    #[test]
    fn execution_runtime_lowers_static_srgb_color_function_declarations() {
        let source = r#".card { color: color(srgb 1 0 0); background-color: color(srgb 50% 25% 0%); border-color: color(display-p3 1 0 0); }"#;
        let execution = execute_transform_passes_on_source(
            source,
            &[
                TransformPassKind::ColorFunctionLowering,
                TransformPassKind::PrintCss,
            ],
        );

        assert_eq!(execution.mutation_count, 2);
        assert_eq!(
            execution.output_css,
            r#".card { color: rgb(255 0 0); background-color: rgb(128 64 0); border-color: color(display-p3 1 0 0); }"#
        );
        assert_eq!(
            execution.executed_pass_ids,
            vec!["p18-color-function-lowering", "p40-print-css"]
        );
    }

    #[test]
    fn execution_runtime_lowers_logical_properties_only_with_static_direction() {
        let source = r#".ltr { direction: ltr; margin-inline-start: 1px; padding-inline-end: 2px; inline-size: 10rem; } .unknown { margin-inline-start: 1px; } .rtl { direction: rtl; writing-mode: horizontal-tb; inset-inline-start: 3px; border-inline-end-color: red; }"#;
        let execution = execute_transform_passes_on_source(
            source,
            &[
                TransformPassKind::LogicalToPhysical,
                TransformPassKind::PrintCss,
            ],
        );

        assert_eq!(execution.mutation_count, 5);
        assert_eq!(
            execution.output_css,
            r#".ltr { direction: ltr; margin-left: 1px; padding-right: 2px; width: 10rem; } .unknown { margin-inline-start: 1px; } .rtl { direction: rtl; writing-mode: horizontal-tb; right: 3px; border-left-color: red; }"#
        );
        assert_eq!(
            execution.executed_pass_ids,
            vec!["p19-logical-to-physical", "p40-print-css"]
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

        assert_eq!(execution.mutation_count, 1);
        assert_eq!(
            execution.output_css,
            r#".card { color: red; } .card .title { color: blue; } .card:hover { color: green; } .comma, .skip { & .x { color: red; } }"#
        );
        assert_eq!(
            execution.executed_pass_ids,
            vec!["p20-nesting-unwrap", "p40-print-css"]
        );
    }

    #[test]
    fn execution_runtime_evaluates_literal_media_branches() {
        let source = r#"@media all { .a { color: red; } } @media not all { .b { color: blue; } } @media screen { .c { color: green; } }"#;
        let execution = execute_transform_passes_on_source(
            source,
            &[
                TransformPassKind::MediaStaticEval,
                TransformPassKind::PrintCss,
            ],
        );

        assert_eq!(execution.mutation_count, 2);
        assert_eq!(
            execution.output_css,
            r#".a { color: red; }  @media screen { .c { color: green; } }"#
        );
        assert_eq!(
            execution.executed_pass_ids,
            vec!["p24-media-static-eval", "p40-print-css"]
        );
    }

    #[test]
    fn execution_runtime_evaluates_simple_supports_branches_with_cascade_witness() {
        let source = r#"@supports (display: grid) { .a { display: grid; } } @supports not (display: grid) { .b { display: block; } } @supports (display: grid) and (color: red) { .c { color: red; } }"#;
        let execution = execute_transform_passes_on_source(
            source,
            &[
                TransformPassKind::SupportsStaticEval,
                TransformPassKind::PrintCss,
            ],
        );

        assert_eq!(execution.mutation_count, 2);
        assert_eq!(
            execution.output_css,
            r#".a { display: grid; }  @supports (display: grid) and (color: red) { .c { color: red; } }"#
        );
        assert_eq!(
            execution.executed_pass_ids,
            vec!["p23-supports-static-eval", "p40-print-css"]
        );
    }

    #[test]
    fn execution_runtime_reduces_simple_same_unit_calc_values() {
        let source = r#".card { width: calc(1px + 2px); height: calc(10rem - 2rem); margin: calc(1px + 2rem); color: calc(1 + 2); }"#;
        let execution = execute_transform_passes_on_source(
            source,
            &[
                TransformPassKind::CalcReduction,
                TransformPassKind::PrintCss,
            ],
        );

        assert_eq!(execution.mutation_count, 3);
        assert_eq!(
            execution.output_css,
            r#".card { width: 3px; height: 8rem; margin: calc(1px + 2rem); color: 3; }"#
        );
        assert_eq!(
            execution.executed_pass_ids,
            vec!["p25-calc-reduction", "p40-print-css"]
        );
    }

    #[test]
    fn execution_runtime_resolves_unique_literal_root_custom_properties() {
        let source = r#":root { --brand: red; --gap: 2rem; --dup: red; --dup: blue; --dynamic: var(--brand); } .card { color: var(--brand); margin: var(--gap); border-color: var(--missing, blue); background: var(--dup); outline-color: var(--dynamic); }"#;
        let execution = execute_transform_passes_on_source(
            source,
            &[
                TransformPassKind::StaticVarSubstitution,
                TransformPassKind::PrintCss,
            ],
        );

        assert_eq!(execution.mutation_count, 3);
        assert_eq!(
            execution.output_css,
            r#":root { --brand: red; --gap: 2rem; --dup: red; --dup: blue; --dynamic: var(--brand); } .card { color: red; margin: 2rem; border-color: blue; background: var(--dup); outline-color: var(--dynamic); }"#
        );
        assert_eq!(
            execution.executed_pass_ids,
            vec!["p32-custom-property-static-resolve", "p40-print-css"]
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
