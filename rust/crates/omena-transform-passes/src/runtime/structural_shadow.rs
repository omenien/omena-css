use std::{
    collections::{BTreeSet, hash_map::DefaultHasher},
    hash::{Hash, Hasher},
};

use omena_cascade::StaticSupportsAssumptionV0;
use omena_cst_typed::{ParsedTypedCst, TypedCstNode};
use omena_incremental::IncrementalRevisionV0;
use omena_parser::{
    ClosedWorldBundleV0, ClosedWorldLinkedModuleV0, ConfigurationHashV0, ModuleIdV0,
    ModuleInstanceKeyV0, StyleDialect, parse, summarize_omena_parser_parity_lite,
    summarize_omena_parser_style_facts,
};
use omena_syntax::SyntaxKind;
use omena_transform_cst::{
    IrNodeIdV0, IrNodeKindV0, TransformIrV0, TransformPassClassV0, TransformPassKind,
    default_transform_pass_descriptors, lower_transform_ir_from_source, print_transform_ir_css,
};

use super::planner::{plan_transform_passes, transform_pass_kind_from_id};
use super::provenance::derive_transform_mutation_spans;
use crate::{
    TransformProvenanceMutationSpanV0, TransformSemanticRemovalCandidate,
    TransformStructuralIrShadowEquivalenceReportV0, TransformStructuralIrShadowFieldReportV0,
    TransformStructuralIrShadowFixtureReportV0,
    domains::{
        cascade_flatten::{
            collect_layer_flatten_proof_candidates_with_lexer,
            collect_scope_flatten_proof_candidates_with_lexer, flatten_css_layers_with_lexer,
            flatten_css_scopes_with_lexer,
        },
        css_modules_classes::{
            local_css_module_composes_resolutions_with_lexer,
            rewrite_css_module_class_names_with_lexer,
            strip_resolved_css_module_composes_with_lexer, tree_shake_css_class_rules_with_lexer,
        },
        css_modules_values::tree_shake_css_modules_values_with_lexer,
        custom_property::tree_shake_css_custom_properties_with_lexer,
        design_token::route_design_token_values_with_lexer,
        import_inline::inline_css_imports_with_lexer,
        keyframes::tree_shake_css_keyframes_with_lexer,
        nesting::unwrap_css_nesting_with_lexer,
        rule_cleanup::{dedupe_exact_css_rules_with_lexer, remove_empty_css_rules_with_lexer},
        rule_merge::{
            merge_adjacent_same_block_css_selectors_with_lexer,
            merge_adjacent_same_selector_css_rules_with_lexer,
        },
        static_eval::{
            StaticMediaEvaluationOptions, evaluate_static_container_rules_with_lexer,
            evaluate_static_media_rules_with_lexer, evaluate_static_supports_rules_with_lexer,
        },
    },
    helpers::ir_transaction::{
        reset_structural_ir_transaction_telemetry, structural_ir_transaction_telemetry_snapshot,
    },
    model::{
        TransformClassNameRewriteV0, TransformCssModuleComposesResolutionV0,
        TransformDesignTokenRouteV0, TransformExecutionContextV0, TransformImportInlineV0,
        TransformSemanticRemovalV0, TransformStructuralIrTransactionTelemetryV0,
    },
    registry::{evaluate_native_css_static_values_with_plan, unwrap_css_nesting_in_ir},
    runtime::executor::{
        execute_transform_passes_on_source_with_dialect_and_context,
        execute_transform_passes_on_source_with_dialect_context_and_closed_world_bundle,
    },
};

const COMPARED_FIELDS: [&str; 12] = [
    "canonicalCssBytes",
    "selectorSet",
    "declarationSet",
    "cascadeOutcome",
    "mutationSpanRanges",
    "mutationCount",
    "semanticRemovals",
    "cssImportInlines",
    "cssModuleComposesExports",
    "cssModuleEvaluation",
    "designTokenRoutes",
    "irTransactionCommitCount",
];

#[derive(Debug, Clone, Copy)]
pub struct TransformStructuralIrShadowFixtureInputV0<'source> {
    pub fixture: &'source str,
    pub pass: TransformPassKind,
    pub dialect: StyleDialect,
    pub source: &'source str,
    pub closed_bundle: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct TransformStructuralIrPipelineShadowFixtureInputV0<'source> {
    pub fixture: &'source str,
    pub dialect: StyleDialect,
    pub source: &'source str,
    pub closed_bundle: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct StructuralShadowPathSnapshotV0 {
    output_css: String,
    mutation_count: usize,
    selector_values: Vec<String>,
    declaration_values: Vec<String>,
    cascade_values: Vec<String>,
    mutation_span_values: Vec<String>,
    semantic_removal_values: Vec<String>,
    css_import_inline_values: Vec<String>,
    css_module_composes_values: Vec<String>,
    css_module_evaluation_values: Vec<String>,
    design_token_route_values: Vec<String>,
    ir_transaction_telemetry: TransformStructuralIrTransactionTelemetryV0,
    typed_payload_telemetry: StructuralShadowTypedPayloadTelemetryV0,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct StructuralShadowTypedPayloadTelemetryV0 {
    projections_consumed: usize,
    memo_hits: usize,
}

#[derive(Debug, Clone)]
struct TypedPayloadProjectionV0 {
    node_id: IrNodeIdV0,
    revision: IncrementalRevisionV0,
    content_signature: u64,
    typed_node_count: usize,
    style_rule_count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct TypedPayloadProjectionKeyV0 {
    node_id: IrNodeIdV0,
    revision: IncrementalRevisionV0,
    content_signature: u64,
}

#[derive(Default)]
struct TypedPayloadProjectionMemoV0 {
    entries: Vec<(TypedPayloadProjectionKeyV0, TypedPayloadProjectionV0)>,
    telemetry: StructuralShadowTypedPayloadTelemetryV0,
}

struct StructuralShadowReachabilityV0 {
    class_names: Vec<String>,
    keyframe_names: Vec<String>,
    value_names: Vec<String>,
    custom_property_names: Vec<String>,
}

#[derive(Debug, Clone, Default)]
struct StructuralShadowModuleContextV0 {
    import_inlines: Vec<TransformImportInlineV0>,
    class_name_rewrites: Vec<TransformClassNameRewriteV0>,
    css_module_composes_resolutions: Vec<TransformCssModuleComposesResolutionV0>,
    design_token_routes: Vec<TransformDesignTokenRouteV0>,
}

#[derive(Debug, Clone, Default)]
struct StructuralShadowModuleEgressValuesV0 {
    css_import_inline_values: Vec<String>,
    css_module_composes_values: Vec<String>,
    css_module_evaluation_values: Vec<String>,
    design_token_route_values: Vec<String>,
}

pub fn summarize_structural_ir_shadow_equivalence_v0()
-> TransformStructuralIrShadowEquivalenceReportV0 {
    let fixtures = structural_shadow_fixtures();
    summarize_structural_ir_shadow_equivalence_for_fixtures_v0(fixtures.as_slice())
}

pub fn summarize_structural_ir_shadow_equivalence_for_fixtures_v0(
    fixtures: &[TransformStructuralIrShadowFixtureInputV0<'_>],
) -> TransformStructuralIrShadowEquivalenceReportV0 {
    let reports = fixtures
        .iter()
        .copied()
        .map(structural_shadow_report_for_fixture)
        .collect::<Vec<_>>();
    let all_fields_match = reports.iter().all(|report| report.all_fields_match);
    let all_typed_path_fields_match = reports
        .iter()
        .all(|report| report.all_typed_path_fields_match);
    let typed_payload_projections_consumed = reports
        .iter()
        .map(|report| report.typed_payload_projections_consumed)
        .sum::<usize>();
    let typed_payload_memo_hits = reports
        .iter()
        .map(|report| report.typed_payload_memo_hits)
        .sum::<usize>();

    TransformStructuralIrShadowEquivalenceReportV0 {
        schema_version: "0",
        product: "omena-transform-passes.structural-ir-shadow-equivalence",
        fixture_count: reports.len(),
        compared_pass_ids: compared_pass_ids(),
        compared_fields: COMPARED_FIELDS.to_vec(),
        reports,
        all_fields_match,
        all_typed_path_fields_match,
        typed_payload_projections_consumed,
        typed_payload_memo_hits,
    }
}

pub fn summarize_structural_ir_pipeline_shadow_equivalence_for_fixtures_v0(
    fixtures: &[TransformStructuralIrPipelineShadowFixtureInputV0<'_>],
) -> TransformStructuralIrShadowEquivalenceReportV0 {
    let reports = fixtures
        .iter()
        .copied()
        .map(structural_pipeline_shadow_report_for_fixture)
        .collect::<Vec<_>>();
    let all_fields_match = reports.iter().all(|report| report.all_fields_match);
    let all_typed_path_fields_match = reports
        .iter()
        .all(|report| report.all_typed_path_fields_match);
    let typed_payload_projections_consumed = reports
        .iter()
        .map(|report| report.typed_payload_projections_consumed)
        .sum::<usize>();
    let typed_payload_memo_hits = reports
        .iter()
        .map(|report| report.typed_payload_memo_hits)
        .sum::<usize>();

    TransformStructuralIrShadowEquivalenceReportV0 {
        schema_version: "0",
        product: "omena-transform-passes.structural-ir-pipeline-shadow-equivalence",
        fixture_count: reports.len(),
        compared_pass_ids: compared_pass_ids(),
        compared_fields: COMPARED_FIELDS.to_vec(),
        reports,
        all_fields_match,
        all_typed_path_fields_match,
        typed_payload_projections_consumed,
        typed_payload_memo_hits,
    }
}

fn structural_shadow_report_for_fixture(
    fixture: TransformStructuralIrShadowFixtureInputV0<'_>,
) -> TransformStructuralIrShadowFixtureReportV0 {
    let string_snapshot = string_path_snapshot(fixture);
    let ir_snapshot = ir_path_snapshot(fixture);
    let typed_snapshot = typed_path_snapshot(fixture);
    let expected_commit_flag = expected_ir_transaction_commit_flag(string_snapshot.mutation_count);
    let (
        ir_path_mutation_count,
        typed_path_mutation_count,
        ir_path_transaction_commit_count,
        typed_payload_projections_consumed,
        typed_payload_memo_hits,
        fields,
    ) = match (ir_snapshot, typed_snapshot) {
        (Ok(ir_snapshot), Ok(typed_snapshot)) => {
            let telemetry = ir_snapshot.ir_transaction_telemetry;
            let typed_payload_telemetry = typed_snapshot.typed_payload_telemetry;
            let actual_commit_flag = if telemetry.transaction_commit_count > 0 {
                "1".to_string()
            } else {
                "0".to_string()
            };
            (
                Some(ir_snapshot.mutation_count),
                Some(typed_snapshot.mutation_count),
                Some(telemetry.transaction_commit_count),
                typed_payload_telemetry.projections_consumed,
                typed_payload_telemetry.memo_hits,
                vec![
                    shadow_field_report_with_typed(
                        "canonicalCssBytes",
                        [string_snapshot.output_css.clone()],
                        [ir_snapshot.output_css],
                        [typed_snapshot.output_css],
                    ),
                    shadow_field_report_with_typed(
                        "selectorSet",
                        string_snapshot.selector_values,
                        ir_snapshot.selector_values,
                        typed_snapshot.selector_values,
                    ),
                    shadow_field_report_with_typed(
                        "declarationSet",
                        string_snapshot.declaration_values,
                        ir_snapshot.declaration_values,
                        typed_snapshot.declaration_values,
                    ),
                    shadow_field_report_with_typed(
                        "cascadeOutcome",
                        string_snapshot.cascade_values,
                        ir_snapshot.cascade_values,
                        typed_snapshot.cascade_values,
                    ),
                    shadow_field_report_with_typed(
                        "mutationSpanRanges",
                        string_snapshot.mutation_span_values,
                        ir_snapshot.mutation_span_values,
                        typed_snapshot.mutation_span_values,
                    ),
                    shadow_field_report_with_typed(
                        "mutationCount",
                        [string_snapshot.mutation_count.to_string()],
                        [ir_snapshot.mutation_count.to_string()],
                        [typed_snapshot.mutation_count.to_string()],
                    ),
                    shadow_field_report_with_typed(
                        "semanticRemovals",
                        string_snapshot.semantic_removal_values,
                        ir_snapshot.semantic_removal_values,
                        typed_snapshot.semantic_removal_values,
                    ),
                    shadow_field_report_with_typed(
                        "cssImportInlines",
                        string_snapshot.css_import_inline_values,
                        ir_snapshot.css_import_inline_values,
                        typed_snapshot.css_import_inline_values,
                    ),
                    shadow_field_report_with_typed(
                        "cssModuleComposesExports",
                        string_snapshot.css_module_composes_values,
                        ir_snapshot.css_module_composes_values,
                        typed_snapshot.css_module_composes_values,
                    ),
                    shadow_field_report_with_typed(
                        "cssModuleEvaluation",
                        string_snapshot.css_module_evaluation_values,
                        ir_snapshot.css_module_evaluation_values,
                        typed_snapshot.css_module_evaluation_values,
                    ),
                    shadow_field_report_with_typed(
                        "designTokenRoutes",
                        string_snapshot.design_token_route_values,
                        ir_snapshot.design_token_route_values,
                        typed_snapshot.design_token_route_values,
                    ),
                    shadow_field_report_with_typed(
                        "irTransactionCommitCount",
                        [expected_commit_flag.clone()],
                        [actual_commit_flag],
                        [expected_ir_transaction_commit_flag(
                            typed_snapshot.mutation_count,
                        )],
                    ),
                ],
            )
        }
        (Err(ir_error), typed_result) => {
            let ir_error = format!("irPathError:{ir_error}");
            let typed_error = typed_result
                .err()
                .map(|error| format!("typedPathError:{error}"))
                .unwrap_or_else(|| "typedPathUnavailableAfterIrError".to_string());
            (
                None,
                None,
                None,
                0,
                0,
                vec![
                    shadow_field_report_with_typed(
                        "canonicalCssBytes",
                        [string_snapshot.output_css],
                        [ir_error.clone()],
                        [typed_error.clone()],
                    ),
                    shadow_field_report_with_typed(
                        "selectorSet",
                        string_snapshot.selector_values,
                        [ir_error.clone()],
                        [typed_error.clone()],
                    ),
                    shadow_field_report_with_typed(
                        "declarationSet",
                        string_snapshot.declaration_values,
                        [ir_error.clone()],
                        [typed_error.clone()],
                    ),
                    shadow_field_report_with_typed(
                        "cascadeOutcome",
                        string_snapshot.cascade_values,
                        [ir_error.clone()],
                        [typed_error.clone()],
                    ),
                    shadow_field_report_with_typed(
                        "mutationSpanRanges",
                        string_snapshot.mutation_span_values,
                        [ir_error.clone()],
                        [typed_error.clone()],
                    ),
                    shadow_field_report_with_typed(
                        "mutationCount",
                        [string_snapshot.mutation_count.to_string()],
                        [ir_error.clone()],
                        [typed_error.clone()],
                    ),
                    shadow_field_report_with_typed(
                        "semanticRemovals",
                        string_snapshot.semantic_removal_values,
                        [ir_error.clone()],
                        [typed_error.clone()],
                    ),
                    shadow_field_report_with_typed(
                        "cssImportInlines",
                        string_snapshot.css_import_inline_values,
                        [ir_error.clone()],
                        [typed_error.clone()],
                    ),
                    shadow_field_report_with_typed(
                        "cssModuleComposesExports",
                        string_snapshot.css_module_composes_values,
                        [ir_error.clone()],
                        [typed_error.clone()],
                    ),
                    shadow_field_report_with_typed(
                        "cssModuleEvaluation",
                        string_snapshot.css_module_evaluation_values,
                        [ir_error.clone()],
                        [typed_error.clone()],
                    ),
                    shadow_field_report_with_typed(
                        "designTokenRoutes",
                        string_snapshot.design_token_route_values,
                        [ir_error.clone()],
                        [typed_error.clone()],
                    ),
                    shadow_field_report_with_typed(
                        "irTransactionCommitCount",
                        [expected_commit_flag],
                        [ir_error.clone()],
                        [typed_error.clone()],
                    ),
                ],
            )
        }
        (Ok(ir_snapshot), Err(typed_error)) => {
            let telemetry = ir_snapshot.ir_transaction_telemetry;
            let actual_commit_flag = if telemetry.transaction_commit_count > 0 {
                "1".to_string()
            } else {
                "0".to_string()
            };
            let typed_error = format!("typedPathError:{typed_error}");
            (
                Some(ir_snapshot.mutation_count),
                None,
                Some(telemetry.transaction_commit_count),
                0,
                0,
                vec![
                    shadow_field_report_with_typed(
                        "canonicalCssBytes",
                        [string_snapshot.output_css],
                        [ir_snapshot.output_css],
                        [typed_error.clone()],
                    ),
                    shadow_field_report_with_typed(
                        "selectorSet",
                        string_snapshot.selector_values,
                        ir_snapshot.selector_values,
                        [typed_error.clone()],
                    ),
                    shadow_field_report_with_typed(
                        "declarationSet",
                        string_snapshot.declaration_values,
                        ir_snapshot.declaration_values,
                        [typed_error.clone()],
                    ),
                    shadow_field_report_with_typed(
                        "cascadeOutcome",
                        string_snapshot.cascade_values,
                        ir_snapshot.cascade_values,
                        [typed_error.clone()],
                    ),
                    shadow_field_report_with_typed(
                        "mutationSpanRanges",
                        string_snapshot.mutation_span_values,
                        ir_snapshot.mutation_span_values,
                        [typed_error.clone()],
                    ),
                    shadow_field_report_with_typed(
                        "mutationCount",
                        [string_snapshot.mutation_count.to_string()],
                        [ir_snapshot.mutation_count.to_string()],
                        [typed_error.clone()],
                    ),
                    shadow_field_report_with_typed(
                        "semanticRemovals",
                        string_snapshot.semantic_removal_values,
                        ir_snapshot.semantic_removal_values,
                        [typed_error.clone()],
                    ),
                    shadow_field_report_with_typed(
                        "cssImportInlines",
                        string_snapshot.css_import_inline_values,
                        ir_snapshot.css_import_inline_values,
                        [typed_error.clone()],
                    ),
                    shadow_field_report_with_typed(
                        "cssModuleComposesExports",
                        string_snapshot.css_module_composes_values,
                        ir_snapshot.css_module_composes_values,
                        [typed_error.clone()],
                    ),
                    shadow_field_report_with_typed(
                        "cssModuleEvaluation",
                        string_snapshot.css_module_evaluation_values,
                        ir_snapshot.css_module_evaluation_values,
                        [typed_error.clone()],
                    ),
                    shadow_field_report_with_typed(
                        "designTokenRoutes",
                        string_snapshot.design_token_route_values,
                        ir_snapshot.design_token_route_values,
                        [typed_error.clone()],
                    ),
                    shadow_field_report_with_typed(
                        "irTransactionCommitCount",
                        [expected_commit_flag],
                        [actual_commit_flag],
                        [typed_error],
                    ),
                ],
            )
        }
    };
    let all_fields_match = fields.iter().all(|field| field.matches);
    let all_typed_path_fields_match = all_fields_match;

    TransformStructuralIrShadowFixtureReportV0 {
        schema_version: "0",
        product: "omena-transform-passes.structural-ir-shadow-fixture",
        fixture: fixture.fixture.to_string(),
        pass_id: fixture.pass.id(),
        dialect: dialect_label(fixture.dialect),
        string_path_mutation_count: Some(string_snapshot.mutation_count),
        ir_path_mutation_count,
        typed_path_mutation_count,
        ir_path_transaction_commit_count,
        typed_payload_projections_consumed,
        typed_payload_memo_hits,
        fields,
        all_fields_match,
        all_typed_path_fields_match,
    }
}

fn structural_pipeline_shadow_report_for_fixture(
    fixture: TransformStructuralIrPipelineShadowFixtureInputV0<'_>,
) -> TransformStructuralIrShadowFixtureReportV0 {
    let string_snapshot = string_pipeline_snapshot(fixture);
    let expected_commit_flag = expected_ir_transaction_commit_flag(string_snapshot.mutation_count);
    let (ir_path_mutation_count, ir_path_transaction_commit_count, fields) =
        match ir_pipeline_snapshot(fixture) {
            Ok(ir_snapshot) => {
                let actual_commit_flag = if ir_snapshot
                    .ir_transaction_telemetry
                    .transaction_commit_count
                    > 0
                {
                    "1".to_string()
                } else {
                    "0".to_string()
                };
                (
                    Some(ir_snapshot.mutation_count),
                    Some(
                        ir_snapshot
                            .ir_transaction_telemetry
                            .transaction_commit_count,
                    ),
                    vec![
                        shadow_field_report(
                            "canonicalCssBytes",
                            [string_snapshot.output_css.clone()],
                            [ir_snapshot.output_css],
                        ),
                        shadow_field_report(
                            "selectorSet",
                            string_snapshot.selector_values,
                            ir_snapshot.selector_values,
                        ),
                        shadow_field_report(
                            "declarationSet",
                            string_snapshot.declaration_values,
                            ir_snapshot.declaration_values,
                        ),
                        shadow_field_report(
                            "cascadeOutcome",
                            string_snapshot.cascade_values,
                            ir_snapshot.cascade_values,
                        ),
                        shadow_field_report(
                            "mutationSpanRanges",
                            string_snapshot.mutation_span_values,
                            ir_snapshot.mutation_span_values,
                        ),
                        shadow_field_report(
                            "mutationCount",
                            [string_snapshot.mutation_count.to_string()],
                            [ir_snapshot.mutation_count.to_string()],
                        ),
                        shadow_field_report(
                            "semanticRemovals",
                            string_snapshot.semantic_removal_values,
                            ir_snapshot.semantic_removal_values,
                        ),
                        shadow_field_report(
                            "cssImportInlines",
                            string_snapshot.css_import_inline_values,
                            ir_snapshot.css_import_inline_values,
                        ),
                        shadow_field_report(
                            "cssModuleComposesExports",
                            string_snapshot.css_module_composes_values,
                            ir_snapshot.css_module_composes_values,
                        ),
                        shadow_field_report(
                            "cssModuleEvaluation",
                            string_snapshot.css_module_evaluation_values,
                            ir_snapshot.css_module_evaluation_values,
                        ),
                        shadow_field_report(
                            "designTokenRoutes",
                            string_snapshot.design_token_route_values,
                            ir_snapshot.design_token_route_values,
                        ),
                        shadow_field_report(
                            "irTransactionCommitCount",
                            [expected_commit_flag.clone()],
                            [actual_commit_flag],
                        ),
                    ],
                )
            }
            Err(error) => {
                let error = format!("irPipelinePathError:{error}");
                (
                    None,
                    None,
                    vec![
                        shadow_field_report(
                            "canonicalCssBytes",
                            [string_snapshot.output_css.clone()],
                            [error.clone()],
                        ),
                        shadow_field_report(
                            "selectorSet",
                            string_snapshot.selector_values,
                            [error.clone()],
                        ),
                        shadow_field_report(
                            "declarationSet",
                            string_snapshot.declaration_values,
                            [error.clone()],
                        ),
                        shadow_field_report(
                            "cascadeOutcome",
                            string_snapshot.cascade_values,
                            [error.clone()],
                        ),
                        shadow_field_report(
                            "mutationSpanRanges",
                            string_snapshot.mutation_span_values,
                            [error.clone()],
                        ),
                        shadow_field_report(
                            "mutationCount",
                            [string_snapshot.mutation_count.to_string()],
                            [error.clone()],
                        ),
                        shadow_field_report(
                            "semanticRemovals",
                            string_snapshot.semantic_removal_values,
                            [error.clone()],
                        ),
                        shadow_field_report(
                            "cssImportInlines",
                            string_snapshot.css_import_inline_values,
                            [error.clone()],
                        ),
                        shadow_field_report(
                            "cssModuleComposesExports",
                            string_snapshot.css_module_composes_values,
                            [error.clone()],
                        ),
                        shadow_field_report(
                            "cssModuleEvaluation",
                            string_snapshot.css_module_evaluation_values,
                            [error.clone()],
                        ),
                        shadow_field_report(
                            "designTokenRoutes",
                            string_snapshot.design_token_route_values,
                            [error.clone()],
                        ),
                        shadow_field_report(
                            "irTransactionCommitCount",
                            [expected_commit_flag],
                            [error],
                        ),
                    ],
                )
            }
        };
    let all_fields_match = fields.iter().all(|field| field.matches);
    let all_typed_path_fields_match = all_fields_match;

    TransformStructuralIrShadowFixtureReportV0 {
        schema_version: "0",
        product: "omena-transform-passes.structural-ir-shadow-fixture",
        fixture: fixture.fixture.to_string(),
        pass_id: "structural-pipeline",
        dialect: dialect_label(fixture.dialect),
        string_path_mutation_count: Some(string_snapshot.mutation_count),
        ir_path_mutation_count,
        typed_path_mutation_count: ir_path_mutation_count,
        ir_path_transaction_commit_count,
        typed_payload_projections_consumed: 0,
        typed_payload_memo_hits: 0,
        fields,
        all_fields_match,
        all_typed_path_fields_match,
    }
}

fn expected_ir_transaction_commit_flag(mutation_count: usize) -> String {
    if mutation_count > 0 {
        "1".to_string()
    } else {
        "0".to_string()
    }
}

fn string_path_snapshot(
    fixture: TransformStructuralIrShadowFixtureInputV0<'_>,
) -> StructuralShadowPathSnapshotV0 {
    let reachability = reachability_for_fixture(fixture);
    let module_context = module_context_for_fixture(fixture);
    let (output_css, mutation_count, semantic_removal_values) = match fixture.pass {
        TransformPassKind::NestingUnwrap => {
            let (output_css, mutation_count) =
                unwrap_css_nesting_with_lexer(fixture.source, fixture.dialect);
            (output_css, mutation_count, Vec::new())
        }
        TransformPassKind::ScopeFlatten => {
            let (output_css, mutation_count) =
                flatten_css_scopes_with_lexer(fixture.source, fixture.dialect);
            (output_css, mutation_count, Vec::new())
        }
        TransformPassKind::LayerFlatten => {
            let (output_css, mutation_count) = flatten_css_layers_with_lexer(
                fixture.source,
                fixture.dialect,
                fixture.closed_bundle,
            );
            (output_css, mutation_count, Vec::new())
        }
        TransformPassKind::RuleDeduplication => {
            let (output_css, mutation_count) =
                dedupe_exact_css_rules_with_lexer(fixture.source, fixture.dialect);
            (output_css, mutation_count, Vec::new())
        }
        TransformPassKind::RuleMerging => {
            let (output_css, mutation_count) =
                merge_adjacent_same_selector_css_rules_with_lexer(fixture.source, fixture.dialect);
            (output_css, mutation_count, Vec::new())
        }
        TransformPassKind::SelectorMerging => {
            let (output_css, mutation_count) =
                merge_adjacent_same_block_css_selectors_with_lexer(fixture.source, fixture.dialect);
            (output_css, mutation_count, Vec::new())
        }
        TransformPassKind::EmptyRuleRemoval => {
            let (output_css, mutation_count) =
                remove_empty_css_rules_with_lexer(fixture.source, fixture.dialect);
            (output_css, mutation_count, Vec::new())
        }
        TransformPassKind::SupportsStaticEval | TransformPassKind::DeadSupportsBranchRemoval => {
            let (output_css, mutation_count) = evaluate_static_supports_rules_with_lexer(
                fixture.source,
                fixture.dialect,
                StaticSupportsAssumptionV0::ModernBrowser,
            );
            (output_css, mutation_count, Vec::new())
        }
        TransformPassKind::MediaStaticEval | TransformPassKind::DeadMediaBranchRemoval => {
            let (output_css, mutation_count) = evaluate_static_media_rules_with_lexer(
                fixture.source,
                fixture.dialect,
                StaticMediaEvaluationOptions::default(),
            );
            (output_css, mutation_count, Vec::new())
        }
        TransformPassKind::ContainerStaticEval => {
            let (output_css, mutation_count) =
                evaluate_static_container_rules_with_lexer(fixture.source, fixture.dialect);
            (output_css, mutation_count, Vec::new())
        }
        TransformPassKind::NativeCssStaticEval => {
            let (output_css, mutation_count) =
                evaluate_native_css_static_values_with_plan(fixture.source, fixture.dialect);
            (output_css, mutation_count, Vec::new())
        }
        TransformPassKind::TreeShakeClass => {
            let (output_css, removals) = tree_shake_css_class_rules_with_lexer(
                fixture.source,
                fixture.dialect,
                reachability.class_names.as_slice(),
            );
            let mutation_count = removals.len();
            (
                output_css,
                mutation_count,
                semantic_removal_values(removals),
            )
        }
        TransformPassKind::TreeShakeKeyframes => {
            let (output_css, removals) = tree_shake_css_keyframes_with_lexer(
                fixture.source,
                fixture.dialect,
                reachability.keyframe_names.as_slice(),
                reachability.class_names.as_slice(),
            );
            let mutation_count = removals.len();
            (
                output_css,
                mutation_count,
                semantic_removal_values(removals),
            )
        }
        TransformPassKind::TreeShakeValue => {
            let (output_css, removals) = tree_shake_css_modules_values_with_lexer(
                fixture.source,
                fixture.dialect,
                reachability.value_names.as_slice(),
                reachability.keyframe_names.as_slice(),
                reachability.class_names.as_slice(),
            );
            let mutation_count = removals.len();
            (
                output_css,
                mutation_count,
                semantic_removal_values(removals),
            )
        }
        TransformPassKind::TreeShakeCustomProperty => {
            let (output_css, removals) = tree_shake_css_custom_properties_with_lexer(
                fixture.source,
                fixture.dialect,
                reachability.custom_property_names.as_slice(),
                reachability.keyframe_names.as_slice(),
                reachability.class_names.as_slice(),
            );
            let mutation_count = removals.len();
            (
                output_css,
                mutation_count,
                semantic_removal_values(removals),
            )
        }
        TransformPassKind::ImportInline => {
            let (output_css, mutation_count) = inline_css_imports_with_lexer(
                fixture.source,
                fixture.dialect,
                module_context.import_inlines.as_slice(),
            );
            (output_css, mutation_count, Vec::new())
        }
        TransformPassKind::ResolveCssModulesComposes => {
            let resolutions = css_module_composes_resolutions_for_fixture(fixture, &module_context);
            let (output_css, mutation_count) = strip_resolved_css_module_composes_with_lexer(
                fixture.source,
                fixture.dialect,
                resolutions.as_slice(),
            );
            (output_css, mutation_count, Vec::new())
        }
        TransformPassKind::HashCssModuleClassNames => {
            let (output_css, mutation_count) = rewrite_css_module_class_names_with_lexer(
                fixture.source,
                fixture.dialect,
                module_context.class_name_rewrites.as_slice(),
            );
            (output_css, mutation_count, Vec::new())
        }
        TransformPassKind::DesignTokenRouting => {
            let (output_css, mutation_count) = route_design_token_values_with_lexer(
                fixture.source,
                fixture.dialect,
                module_context.design_token_routes.as_slice(),
            );
            (output_css, mutation_count, Vec::new())
        }
        _ => (fixture.source.to_string(), 0, Vec::new()),
    };
    path_snapshot_from_output(
        fixture,
        output_css,
        mutation_count,
        semantic_removal_values,
        module_egress_values_for_fixture(fixture, &module_context),
        TransformStructuralIrTransactionTelemetryV0::default(),
    )
}

fn string_pipeline_snapshot(
    fixture: TransformStructuralIrPipelineShadowFixtureInputV0<'_>,
) -> StructuralShadowPathSnapshotV0 {
    let mut current_source = fixture.source.to_string();
    let mut mutation_count = 0;
    let mut semantic_removal_values = Vec::new();
    let mut css_import_inline_values = Vec::new();
    let mut css_module_composes_values = Vec::new();
    let mut css_module_evaluation_values = Vec::new();
    let mut design_token_route_values = Vec::new();

    for pass in structural_pipeline_passes() {
        let pass_fixture = TransformStructuralIrShadowFixtureInputV0 {
            fixture: fixture.fixture,
            pass,
            dialect: fixture.dialect,
            source: current_source.as_str(),
            closed_bundle: fixture.closed_bundle,
        };
        let snapshot = string_path_snapshot(pass_fixture);
        mutation_count += snapshot.mutation_count;
        semantic_removal_values.extend(snapshot.semantic_removal_values);
        css_import_inline_values.extend(snapshot.css_import_inline_values);
        css_module_composes_values.extend(snapshot.css_module_composes_values);
        css_module_evaluation_values.extend(snapshot.css_module_evaluation_values);
        design_token_route_values.extend(snapshot.design_token_route_values);
        current_source = snapshot.output_css;
    }

    path_snapshot_from_output(
        TransformStructuralIrShadowFixtureInputV0 {
            fixture: fixture.fixture,
            pass: TransformPassKind::NestingUnwrap,
            dialect: fixture.dialect,
            source: fixture.source,
            closed_bundle: fixture.closed_bundle,
        },
        current_source,
        mutation_count,
        semantic_removal_values,
        StructuralShadowModuleEgressValuesV0 {
            css_import_inline_values,
            css_module_composes_values,
            css_module_evaluation_values,
            design_token_route_values,
        },
        TransformStructuralIrTransactionTelemetryV0::default(),
    )
}

fn ir_path_snapshot(
    fixture: TransformStructuralIrShadowFixtureInputV0<'_>,
) -> Result<StructuralShadowPathSnapshotV0, String> {
    let reachability = reachability_for_fixture(fixture);
    let module_context = module_context_for_fixture(fixture);
    let context = execution_context_for_fixture(&reachability, &module_context);
    let passes = [fixture.pass];
    let summary = if fixture_requires_closed_world_bundle(fixture) {
        let bundle = closed_world_bundle_for_shadow_fixture(fixture.fixture, &reachability)?;
        execute_transform_passes_on_source_with_dialect_context_and_closed_world_bundle(
            fixture.source,
            fixture.dialect,
            &passes,
            &context,
            &bundle,
        )
    } else {
        execute_transform_passes_on_source_with_dialect_and_context(
            fixture.source,
            fixture.dialect,
            &passes,
            &context,
        )
    };

    Ok(path_snapshot_from_output(
        fixture,
        summary.output_css,
        summary.mutation_count,
        public_semantic_removal_values(summary.semantic_removals),
        StructuralShadowModuleEgressValuesV0 {
            css_import_inline_values: json_values(summary.css_import_inlines.as_slice()),
            css_module_composes_values: json_values(summary.css_module_composes_exports.as_slice()),
            css_module_evaluation_values: summary
                .css_module_evaluation
                .as_ref()
                .map(|evaluation| serde_json::to_string(evaluation).unwrap_or_default())
                .into_iter()
                .collect(),
            design_token_route_values: json_values(summary.design_token_routes.as_slice()),
        },
        summary.structural_ir_transaction_telemetry,
    ))
}

fn typed_path_snapshot(
    fixture: TransformStructuralIrShadowFixtureInputV0<'_>,
) -> Result<StructuralShadowPathSnapshotV0, String> {
    if fixture.pass != TransformPassKind::NestingUnwrap {
        return ir_path_snapshot(fixture);
    }

    let mut ir = lower_transform_ir_from_source(
        fixture.source,
        fixture.dialect,
        "omena-transform-passes.typed-payload-shadow",
    );
    let mut memo = TypedPayloadProjectionMemoV0::default();
    let revision = IncrementalRevisionV0 { value: 1 };
    let mut typed_payload_ready = false;
    for node_id in top_level_style_rule_node_ids(&ir) {
        let Some(node_source) = node_source_for_typed_payload(&ir, node_id) else {
            continue;
        };
        if let Some(projection) =
            memo.project_style_rule_payload(node_id, node_source, fixture.dialect, revision)
        {
            typed_payload_ready |= projection_supports_nesting_unwrap(&projection, node_id);
            let _ =
                memo.project_style_rule_payload(node_id, node_source, fixture.dialect, revision);
        }
    }

    reset_structural_ir_transaction_telemetry();
    let mutation_count = if typed_payload_ready {
        unwrap_css_nesting_in_ir(&mut ir, fixture.dialect)
            .map_err(|error| format!("typed payload structural rewrite failed: {error:?}"))?
    } else {
        0
    };
    let telemetry = structural_ir_transaction_telemetry_snapshot();
    let output_css = print_transform_ir_css(&ir)
        .map_err(|error| format!("typed payload structural print failed: {error:?}"))?;
    let mut snapshot = path_snapshot_from_output(
        fixture,
        output_css,
        mutation_count,
        Vec::new(),
        StructuralShadowModuleEgressValuesV0::default(),
        telemetry,
    );
    snapshot.typed_payload_telemetry = memo.telemetry;
    Ok(snapshot)
}

impl TypedPayloadProjectionMemoV0 {
    fn project_style_rule_payload(
        &mut self,
        node_id: IrNodeIdV0,
        source: &str,
        dialect: StyleDialect,
        revision: IncrementalRevisionV0,
    ) -> Option<TypedPayloadProjectionV0> {
        let key = TypedPayloadProjectionKeyV0 {
            node_id,
            revision,
            content_signature: typed_payload_content_signature(source),
        };
        self.telemetry.projections_consumed += 1;
        if let Some((_, projection)) = self.entries.iter().find(|(candidate, _)| *candidate == key)
        {
            self.telemetry.memo_hits += 1;
            return Some(projection.clone());
        }

        let parsed = parse(source, dialect);
        let cst = ParsedTypedCst::from_parse_result(&parsed);
        let stylesheet = cst.stylesheet()?;
        let mut stack = stylesheet.children();
        let mut typed_node_count = 1usize;
        let mut style_rule_count = 0usize;
        while let Some(node) = stack.pop() {
            typed_node_count += 1;
            if matches!(node.kind(), SyntaxKind::Rule | SyntaxKind::QualifiedRule) {
                style_rule_count += 1;
            }
            stack.extend(node.children());
        }

        let projection = TypedPayloadProjectionV0 {
            node_id,
            revision,
            content_signature: key.content_signature,
            typed_node_count,
            style_rule_count,
        };
        self.entries.push((key, projection.clone()));
        Some(projection)
    }
}

fn projection_supports_nesting_unwrap(
    projection: &TypedPayloadProjectionV0,
    node_id: IrNodeIdV0,
) -> bool {
    projection.node_id == node_id
        && projection.revision.value > 0
        && projection.content_signature != 0
        && projection.typed_node_count > 0
        && projection.style_rule_count > 0
}

fn top_level_style_rule_node_ids(ir: &TransformIrV0) -> Vec<IrNodeIdV0> {
    ir.nodes
        .iter()
        .filter(|node| {
            !node.deleted
                && node.kind == IrNodeKindV0::StyleRule
                && node
                    .parent
                    .and_then(|parent_id| ir.nodes.get(parent_id.index()))
                    .is_none_or(|parent| parent.deleted || parent.kind != IrNodeKindV0::StyleRule)
        })
        .map(|node| node.node_id)
        .collect()
}

fn node_source_for_typed_payload(ir: &TransformIrV0, node_id: IrNodeIdV0) -> Option<&str> {
    let node = ir.nodes.get(node_id.index())?;
    ir.source_text()
        .get(node.source_span_start..node.source_span_end)
}

fn typed_payload_content_signature(source: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    source.hash(&mut hasher);
    hasher.finish()
}

fn ir_pipeline_snapshot(
    fixture: TransformStructuralIrPipelineShadowFixtureInputV0<'_>,
) -> Result<StructuralShadowPathSnapshotV0, String> {
    let reachability = reachability_for_pipeline_fixture(fixture);
    let module_context = module_context_for_pipeline_fixture(fixture);
    let context = TransformExecutionContextV0 {
        reachable_class_names: reachability.class_names,
        reachable_keyframe_names: reachability.keyframe_names,
        reachable_value_names: reachability.value_names,
        reachable_custom_property_names: reachability.custom_property_names,
        import_inlines: module_context.import_inlines,
        class_name_rewrites: module_context.class_name_rewrites,
        css_module_composes_resolutions: module_context.css_module_composes_resolutions,
        design_token_routes: module_context.design_token_routes,
        ..TransformExecutionContextV0::default()
    };
    let passes = structural_pipeline_passes();
    let bundle = closed_world_bundle_for_shadow_fixture(
        fixture.fixture,
        &reachability_for_pipeline_fixture(fixture),
    )?;
    let summary = execute_transform_passes_on_source_with_dialect_context_and_closed_world_bundle(
        fixture.source,
        fixture.dialect,
        passes.as_slice(),
        &context,
        &bundle,
    );

    Ok(path_snapshot_from_output(
        TransformStructuralIrShadowFixtureInputV0 {
            fixture: fixture.fixture,
            pass: TransformPassKind::NestingUnwrap,
            dialect: fixture.dialect,
            source: fixture.source,
            closed_bundle: fixture.closed_bundle,
        },
        summary.output_css,
        summary.mutation_count,
        public_semantic_removal_values(summary.semantic_removals),
        StructuralShadowModuleEgressValuesV0 {
            css_import_inline_values: json_values(summary.css_import_inlines.as_slice()),
            css_module_composes_values: json_values(summary.css_module_composes_exports.as_slice()),
            css_module_evaluation_values: summary
                .css_module_evaluation
                .as_ref()
                .map(|evaluation| serde_json::to_string(evaluation).unwrap_or_default())
                .into_iter()
                .collect(),
            design_token_route_values: json_values(summary.design_token_routes.as_slice()),
        },
        summary.structural_ir_transaction_telemetry,
    ))
}

fn execution_context_for_fixture(
    reachability: &StructuralShadowReachabilityV0,
    module_context: &StructuralShadowModuleContextV0,
) -> TransformExecutionContextV0 {
    TransformExecutionContextV0 {
        reachable_class_names: reachability.class_names.clone(),
        reachable_keyframe_names: reachability.keyframe_names.clone(),
        reachable_value_names: reachability.value_names.clone(),
        reachable_custom_property_names: reachability.custom_property_names.clone(),
        import_inlines: module_context.import_inlines.clone(),
        class_name_rewrites: module_context.class_name_rewrites.clone(),
        css_module_composes_resolutions: module_context.css_module_composes_resolutions.clone(),
        design_token_routes: module_context.design_token_routes.clone(),
        ..TransformExecutionContextV0::default()
    }
}

fn fixture_requires_closed_world_bundle(
    fixture: TransformStructuralIrShadowFixtureInputV0<'_>,
) -> bool {
    fixture.closed_bundle
        || matches!(
            fixture.pass,
            TransformPassKind::TreeShakeClass
                | TransformPassKind::TreeShakeKeyframes
                | TransformPassKind::TreeShakeValue
                | TransformPassKind::TreeShakeCustomProperty
        )
}

fn closed_world_bundle_for_shadow_fixture(
    fixture_name: &str,
    reachability: &StructuralShadowReachabilityV0,
) -> Result<ClosedWorldBundleV0, String> {
    let instance = ModuleInstanceKeyV0::new(
        ModuleIdV0::new(format!("omena-transform-passes.shadow.{fixture_name}")),
        ConfigurationHashV0::none(),
    );
    let mut module = ClosedWorldLinkedModuleV0::new(instance.clone());
    for name in &reachability.class_names {
        module = module.with_class_name(name.clone());
    }
    for name in &reachability.keyframe_names {
        module = module.with_keyframe_name(name.clone());
    }
    for name in &reachability.value_names {
        module = module.with_value_name(name.clone());
    }
    for name in &reachability.custom_property_names {
        module = module.with_custom_property_name(name.clone());
    }

    ClosedWorldBundleV0::try_from_linked_modules(vec![instance], vec![module])
        .map_err(|err| format!("closed-world bundle construction failed: {err:?}"))
}

fn structural_pipeline_passes() -> Vec<TransformPassKind> {
    let structural_passes = default_transform_pass_descriptors()
        .into_iter()
        .filter(|descriptor| descriptor.pass_class == TransformPassClassV0::Structural)
        .map(|descriptor| descriptor.kind)
        .collect::<Vec<_>>();
    plan_transform_passes(structural_passes.as_slice())
        .ordered_pass_ids
        .iter()
        .filter_map(|pass_id| transform_pass_kind_from_id(pass_id))
        .collect()
}

fn path_snapshot_from_output(
    fixture: TransformStructuralIrShadowFixtureInputV0<'_>,
    output_css: String,
    mutation_count: usize,
    semantic_removal_values: Vec<String>,
    module_egress_values: StructuralShadowModuleEgressValuesV0,
    ir_transaction_telemetry: TransformStructuralIrTransactionTelemetryV0,
) -> StructuralShadowPathSnapshotV0 {
    let cascade_values = cascade_values_for_source(fixture, output_css.as_str());
    StructuralShadowPathSnapshotV0 {
        selector_values: selector_values_for_source(&output_css, fixture.dialect),
        declaration_values: declaration_values_for_source(&output_css, fixture.dialect),
        cascade_values,
        mutation_span_values: mutation_span_values(derive_transform_mutation_spans(
            fixture.source,
            output_css.as_str(),
        )),
        output_css,
        mutation_count,
        semantic_removal_values,
        css_import_inline_values: module_egress_values.css_import_inline_values,
        css_module_composes_values: module_egress_values.css_module_composes_values,
        css_module_evaluation_values: module_egress_values.css_module_evaluation_values,
        design_token_route_values: module_egress_values.design_token_route_values,
        ir_transaction_telemetry,
        typed_payload_telemetry: StructuralShadowTypedPayloadTelemetryV0::default(),
    }
}

fn structural_shadow_fixtures() -> Vec<TransformStructuralIrShadowFixtureInputV0<'static>> {
    vec![
        TransformStructuralIrShadowFixtureInputV0 {
            fixture: "nesting-descendant-and-pseudo",
            pass: TransformPassKind::NestingUnwrap,
            dialect: StyleDialect::Css,
            source: ".card { color: red; & .title { color: blue; } &:hover { color: green; } }",
            closed_bundle: false,
        },
        TransformStructuralIrShadowFixtureInputV0 {
            fixture: "nesting-conditional-group",
            pass: TransformPassKind::NestingUnwrap,
            dialect: StyleDialect::Css,
            source: "@media (min-width: 40rem) { .card { color: red; & .title { color: blue; } } }",
            closed_bundle: false,
        },
        TransformStructuralIrShadowFixtureInputV0 {
            fixture: "scope-root-flatten",
            pass: TransformPassKind::ScopeFlatten,
            dialect: StyleDialect::Css,
            source: "@scope (:root) { .card { color: red; } }",
            closed_bundle: false,
        },
        TransformStructuralIrShadowFixtureInputV0 {
            fixture: "scope-limit-blocked",
            pass: TransformPassKind::ScopeFlatten,
            dialect: StyleDialect::Css,
            source: "@scope (.theme) to (.stop) { .card { color: red; } }",
            closed_bundle: false,
        },
        TransformStructuralIrShadowFixtureInputV0 {
            fixture: "layer-closed-bundle-flatten",
            pass: TransformPassKind::LayerFlatten,
            dialect: StyleDialect::Css,
            source: "@layer theme { .card { color: red; } }",
            closed_bundle: true,
        },
        TransformStructuralIrShadowFixtureInputV0 {
            fixture: "layer-open-bundle-blocked",
            pass: TransformPassKind::LayerFlatten,
            dialect: StyleDialect::Css,
            source: "@layer theme { .card { color: red; } }",
            closed_bundle: false,
        },
        TransformStructuralIrShadowFixtureInputV0 {
            fixture: "rule-dedup-overridden-declarations",
            pass: TransformPassKind::RuleDeduplication,
            dialect: StyleDialect::Css,
            source: ".a { color: red; color: blue; --tone: red; --tone: blue; color: green !important; color: black !important; } :export { token: red; token: blue; }",
            closed_bundle: false,
        },
        TransformStructuralIrShadowFixtureInputV0 {
            fixture: "rule-dedup-duplicate-rules",
            pass: TransformPassKind::RuleDeduplication,
            dialect: StyleDialect::Css,
            source: ".a { color: red; } .b { color: red; } .a { color: blue; } .a { color: red; }",
            closed_bundle: false,
        },
        TransformStructuralIrShadowFixtureInputV0 {
            fixture: "rule-merge-adjacent-ordinary",
            pass: TransformPassKind::RuleMerging,
            dialect: StyleDialect::Css,
            source: ".a { color: red; } .a { background: blue; } .a { outline: 0; } .b { color: red; }",
            closed_bundle: false,
        },
        TransformStructuralIrShadowFixtureInputV0 {
            fixture: "rule-merge-adjacent-conditional-wrappers",
            pass: TransformPassKind::RuleMerging,
            dialect: StyleDialect::Css,
            source: "@media (prefers-color-scheme: dark) { .card { color: white; } } @media (prefers-color-scheme: dark) { .card .title { color: #ddd; } } @supports (display: grid) { .grid { display: grid; } }",
            closed_bundle: false,
        },
        TransformStructuralIrShadowFixtureInputV0 {
            fixture: "selector-merge-adjacent-same-block",
            pass: TransformPassKind::SelectorMerging,
            dialect: StyleDialect::Css,
            source: ".a { color: red; } .b { color: red; } .c { color: red; } .d { color: blue; }",
            closed_bundle: false,
        },
        TransformStructuralIrShadowFixtureInputV0 {
            fixture: "selector-merge-nested-same-block",
            pass: TransformPassKind::SelectorMerging,
            dialect: StyleDialect::Css,
            source: "@media (min-width: 1px) { .m { color: black; } .n { color: black; } }",
            closed_bundle: false,
        },
        TransformStructuralIrShadowFixtureInputV0 {
            fixture: "empty-rule-ordinary-and-group",
            pass: TransformPassKind::EmptyRuleRemoval,
            dialect: StyleDialect::Css,
            source: ".a {} @media (min-width: 1px) { .b {} } @keyframes spin { from {} to { opacity: 1; } }",
            closed_bundle: false,
        },
        TransformStructuralIrShadowFixtureInputV0 {
            fixture: "empty-rule-preserves-comment-block",
            pass: TransformPassKind::EmptyRuleRemoval,
            dialect: StyleDialect::Css,
            source: ".a { /* keep */ } .b { color: red; }",
            closed_bundle: false,
        },
        TransformStructuralIrShadowFixtureInputV0 {
            fixture: "supports-static-true-unwrap",
            pass: TransformPassKind::SupportsStaticEval,
            dialect: StyleDialect::Css,
            source: "@supports (display: grid) { .a { display: grid; } }",
            closed_bundle: false,
        },
        TransformStructuralIrShadowFixtureInputV0 {
            fixture: "supports-static-false-remove",
            pass: TransformPassKind::DeadSupportsBranchRemoval,
            dialect: StyleDialect::Css,
            source: "@supports not (display: grid) { .a { display: grid; } } .b { color: red; }",
            closed_bundle: false,
        },
        TransformStructuralIrShadowFixtureInputV0 {
            fixture: "media-static-true-unwrap",
            pass: TransformPassKind::MediaStaticEval,
            dialect: StyleDialect::Css,
            source: "@media all { .a { color: red; } } @media (min-width: 40PX) { .b { color: blue; } }",
            closed_bundle: false,
        },
        TransformStructuralIrShadowFixtureInputV0 {
            fixture: "media-static-false-remove",
            pass: TransformPassKind::DeadMediaBranchRemoval,
            dialect: StyleDialect::Css,
            source: "@media not all { .a { color: red; } } .b { color: blue; }",
            closed_bundle: false,
        },
        TransformStructuralIrShadowFixtureInputV0 {
            fixture: "container-static-false-remove",
            pass: TransformPassKind::ContainerStaticEval,
            dialect: StyleDialect::Css,
            source: "@container (max-width: -1px) { .a { color: red; } } .b { color: blue; }",
            closed_bundle: false,
        },
        TransformStructuralIrShadowFixtureInputV0 {
            fixture: "native-css-static-when-fold",
            pass: TransformPassKind::NativeCssStaticEval,
            dialect: StyleDialect::Css,
            source: "@when supports(display: grid) { .grid { display: grid; } } @else { .fallback { display: block; } }",
            closed_bundle: false,
        },
        TransformStructuralIrShadowFixtureInputV0 {
            fixture: "tree-shake-class-reachable-owner",
            pass: TransformPassKind::TreeShakeClass,
            dialect: StyleDialect::Css,
            source: ".used { color: green; } .unused, .also-unused { color: red; } :global(.external) { color: black; }",
            closed_bundle: false,
        },
        TransformStructuralIrShadowFixtureInputV0 {
            fixture: "tree-shake-keyframes-referenced-animation",
            pass: TransformPassKind::TreeShakeKeyframes,
            dialect: StyleDialect::Css,
            source: "@keyframes spin { to { opacity: 1; } } @keyframes fade { to { opacity: 0; } } .used { animation: spin 1s; }",
            closed_bundle: false,
        },
        TransformStructuralIrShadowFixtureInputV0 {
            fixture: "tree-shake-css-modules-values",
            pass: TransformPassKind::TreeShakeValue,
            dialect: StyleDialect::Css,
            source: "@value keep: 1px; @value dead: 2px; @value imported, unused from \"./tokens.css\"; :export { keepExport: keep; deadExport: dead; }",
            closed_bundle: false,
        },
        TransformStructuralIrShadowFixtureInputV0 {
            fixture: "tree-shake-custom-properties",
            pass: TransformPassKind::TreeShakeCustomProperty,
            dialect: StyleDialect::Css,
            source: "@property --dead-reg { syntax: \"<color>\"; inherits: false; initial-value: red; } .used { color: var(--keep); --keep: green; --dead: red; } :export { keepExport: var(--keep); deadExport: var(--dead); }",
            closed_bundle: false,
        },
        TransformStructuralIrShadowFixtureInputV0 {
            fixture: "module-import-inline",
            pass: TransformPassKind::ImportInline,
            dialect: StyleDialect::Css,
            source: "@import \"./tokens.css\"; .used { color: var(--brand); }",
            closed_bundle: false,
        },
        TransformStructuralIrShadowFixtureInputV0 {
            fixture: "module-composes-resolution",
            pass: TransformPassKind::ResolveCssModulesComposes,
            dialect: StyleDialect::Css,
            source: ".button { composes: base utility; color: red; } .base { color: blue; } .utility { color: green; }",
            closed_bundle: false,
        },
        TransformStructuralIrShadowFixtureInputV0 {
            fixture: "module-class-hashing",
            pass: TransformPassKind::HashCssModuleClassNames,
            dialect: StyleDialect::Css,
            source: ".button { composes: base utility global(reset); color: red; } :local { .button { color: blue; } } @supports selector(.button) { .button { color: green; } }",
            closed_bundle: false,
        },
        TransformStructuralIrShadowFixtureInputV0 {
            fixture: "module-design-token-routing",
            pass: TransformPassKind::DesignTokenRouting,
            dialect: StyleDialect::Css,
            source: "@media (min-width: var(--pkg-breakpoint)) { .button { color: var(--pkg-brand); } }",
            closed_bundle: false,
        },
    ]
}

fn compared_pass_ids() -> Vec<&'static str> {
    let mut pass_ids = default_transform_pass_descriptors()
        .into_iter()
        .filter(|descriptor| descriptor.pass_class == TransformPassClassV0::Structural)
        .map(|descriptor| descriptor.id)
        .collect::<Vec<_>>();
    pass_ids.sort_unstable();
    pass_ids
}

fn reachability_for_fixture(
    fixture: TransformStructuralIrShadowFixtureInputV0<'_>,
) -> StructuralShadowReachabilityV0 {
    match fixture.fixture {
        "tree-shake-class-reachable-owner" => StructuralShadowReachabilityV0 {
            class_names: string_vec(["used"]),
            keyframe_names: Vec::new(),
            value_names: Vec::new(),
            custom_property_names: Vec::new(),
        },
        "tree-shake-keyframes-referenced-animation" => StructuralShadowReachabilityV0 {
            class_names: string_vec(["used"]),
            keyframe_names: Vec::new(),
            value_names: Vec::new(),
            custom_property_names: Vec::new(),
        },
        "tree-shake-css-modules-values" => StructuralShadowReachabilityV0 {
            class_names: Vec::new(),
            keyframe_names: Vec::new(),
            value_names: string_vec(["keepExport"]),
            custom_property_names: Vec::new(),
        },
        "tree-shake-custom-properties" => StructuralShadowReachabilityV0 {
            class_names: string_vec(["used"]),
            keyframe_names: Vec::new(),
            value_names: Vec::new(),
            custom_property_names: string_vec(["keepExport"]),
        },
        "pipeline-module-structural-interpass" => StructuralShadowReachabilityV0 {
            class_names: string_vec(["card", "card__icon", "base", "utility"]),
            keyframe_names: string_vec(["spin"]),
            value_names: Vec::new(),
            custom_property_names: string_vec(["pkg-brand", "local-tone"]),
        },
        "pipeline-rule-structural-interpass" => StructuralShadowReachabilityV0 {
            class_names: string_vec(["card", "card__icon", "dup", "grid", "media"]),
            keyframe_names: Vec::new(),
            value_names: Vec::new(),
            custom_property_names: Vec::new(),
        },
        _ => StructuralShadowReachabilityV0 {
            class_names: Vec::new(),
            keyframe_names: Vec::new(),
            value_names: Vec::new(),
            custom_property_names: Vec::new(),
        },
    }
}

fn reachability_for_pipeline_fixture(
    fixture: TransformStructuralIrPipelineShadowFixtureInputV0<'_>,
) -> StructuralShadowReachabilityV0 {
    reachability_for_fixture(TransformStructuralIrShadowFixtureInputV0 {
        fixture: fixture.fixture,
        pass: TransformPassKind::TreeShakeClass,
        dialect: fixture.dialect,
        source: fixture.source,
        closed_bundle: fixture.closed_bundle,
    })
}

fn module_context_for_pipeline_fixture(
    fixture: TransformStructuralIrPipelineShadowFixtureInputV0<'_>,
) -> StructuralShadowModuleContextV0 {
    module_context_for_fixture(TransformStructuralIrShadowFixtureInputV0 {
        fixture: fixture.fixture,
        pass: TransformPassKind::ImportInline,
        dialect: fixture.dialect,
        source: fixture.source,
        closed_bundle: fixture.closed_bundle,
    })
}

fn module_context_for_fixture(
    fixture: TransformStructuralIrShadowFixtureInputV0<'_>,
) -> StructuralShadowModuleContextV0 {
    match fixture.fixture {
        "module-import-inline" => StructuralShadowModuleContextV0 {
            import_inlines: vec![TransformImportInlineV0 {
                import_source: "./tokens.css".to_string(),
                replacement_css: ":root { --brand: red; }".to_string(),
            }],
            ..StructuralShadowModuleContextV0::default()
        },
        "module-class-hashing" => StructuralShadowModuleContextV0 {
            class_name_rewrites: vec![
                TransformClassNameRewriteV0 {
                    original_name: "button".to_string(),
                    rewritten_name: "_button_hash".to_string(),
                },
                TransformClassNameRewriteV0 {
                    original_name: "base".to_string(),
                    rewritten_name: "_base_hash".to_string(),
                },
                TransformClassNameRewriteV0 {
                    original_name: "utility".to_string(),
                    rewritten_name: "_utility_hash".to_string(),
                },
            ],
            ..StructuralShadowModuleContextV0::default()
        },
        "module-design-token-routing" => StructuralShadowModuleContextV0 {
            design_token_routes: vec![
                TransformDesignTokenRouteV0 {
                    token_name: "--pkg-breakpoint".to_string(),
                    routed_value: "40rem".to_string(),
                },
                TransformDesignTokenRouteV0 {
                    token_name: "--pkg-brand".to_string(),
                    routed_value: "#123456".to_string(),
                },
            ],
            ..StructuralShadowModuleContextV0::default()
        },
        "pipeline-module-structural-interpass" => StructuralShadowModuleContextV0 {
            import_inlines: vec![TransformImportInlineV0 {
                import_source: "./tokens.css".to_string(),
                replacement_css: ":root { --pkg-brand: #123456; }".to_string(),
            }],
            class_name_rewrites: vec![
                TransformClassNameRewriteV0 {
                    original_name: "card".to_string(),
                    rewritten_name: "_card_hash".to_string(),
                },
                TransformClassNameRewriteV0 {
                    original_name: "card__icon".to_string(),
                    rewritten_name: "_card__icon_hash".to_string(),
                },
                TransformClassNameRewriteV0 {
                    original_name: "base".to_string(),
                    rewritten_name: "_base_hash".to_string(),
                },
                TransformClassNameRewriteV0 {
                    original_name: "utility".to_string(),
                    rewritten_name: "_utility_hash".to_string(),
                },
            ],
            css_module_composes_resolutions: vec![TransformCssModuleComposesResolutionV0 {
                local_class_name: "card".to_string(),
                exported_class_names: vec!["base".to_string(), "utility".to_string()],
            }],
            design_token_routes: vec![TransformDesignTokenRouteV0 {
                token_name: "--pkg-brand".to_string(),
                routed_value: "#123456".to_string(),
            }],
        },
        _ => StructuralShadowModuleContextV0::default(),
    }
}

fn css_module_composes_resolutions_for_fixture(
    fixture: TransformStructuralIrShadowFixtureInputV0<'_>,
    module_context: &StructuralShadowModuleContextV0,
) -> Vec<TransformCssModuleComposesResolutionV0> {
    let mut merged =
        local_css_module_composes_resolutions_with_lexer(fixture.source, fixture.dialect);
    for resolution in &module_context.css_module_composes_resolutions {
        let Some(existing) = merged
            .iter_mut()
            .find(|existing| existing.local_class_name == resolution.local_class_name)
        else {
            merged.push(resolution.clone());
            continue;
        };
        for exported_class_name in &resolution.exported_class_names {
            if !existing
                .exported_class_names
                .iter()
                .any(|existing| existing == exported_class_name)
            {
                existing
                    .exported_class_names
                    .push(exported_class_name.clone());
            }
        }
    }
    merged.sort_by(|left, right| left.local_class_name.cmp(&right.local_class_name));
    merged
}

fn module_egress_values_for_fixture(
    fixture: TransformStructuralIrShadowFixtureInputV0<'_>,
    module_context: &StructuralShadowModuleContextV0,
) -> StructuralShadowModuleEgressValuesV0 {
    match fixture.pass {
        TransformPassKind::ImportInline => StructuralShadowModuleEgressValuesV0 {
            css_import_inline_values: json_values(module_context.import_inlines.as_slice()),
            ..StructuralShadowModuleEgressValuesV0::default()
        },
        TransformPassKind::ResolveCssModulesComposes => StructuralShadowModuleEgressValuesV0 {
            css_module_composes_values: json_values(
                css_module_composes_resolutions_for_fixture(fixture, module_context).as_slice(),
            ),
            ..StructuralShadowModuleEgressValuesV0::default()
        },
        TransformPassKind::DesignTokenRouting => StructuralShadowModuleEgressValuesV0 {
            design_token_route_values: json_values(module_context.design_token_routes.as_slice()),
            ..StructuralShadowModuleEgressValuesV0::default()
        },
        _ => StructuralShadowModuleEgressValuesV0::default(),
    }
}

fn json_values<T: serde::Serialize>(values: &[T]) -> Vec<String> {
    values
        .iter()
        .map(|value| serde_json::to_string(value).unwrap_or_default())
        .collect()
}

fn selector_values_for_source(source: &str, dialect: StyleDialect) -> Vec<String> {
    let summary = summarize_omena_parser_style_facts(source, dialect);
    sorted_unique(
        summary
            .class_selector_names
            .into_iter()
            .map(|name| format!("class:{name}"))
            .chain(
                summary
                    .id_selector_names
                    .into_iter()
                    .map(|name| format!("id:{name}")),
            )
            .chain(
                summary
                    .placeholder_selector_names
                    .into_iter()
                    .map(|name| format!("placeholder:{name}")),
            )
            .collect::<Vec<_>>(),
    )
}

fn declaration_values_for_source(source: &str, dialect: StyleDialect) -> Vec<String> {
    let summary = summarize_omena_parser_parity_lite(source, dialect);
    sorted_unique(vec![
        format!("ruleCount:{}", summary.rule_count),
        format!("declarationCount:{}", summary.declaration_count),
        format!(
            "declarationKindCounts:{}",
            serde_json::to_string(&summary.declaration_kind_counts).unwrap_or_default()
        ),
        format!(
            "atRuleKindCounts:{}",
            serde_json::to_string(&summary.at_rule_kind_counts).unwrap_or_default()
        ),
    ])
}

fn cascade_values_for_source(
    fixture: TransformStructuralIrShadowFixtureInputV0<'_>,
    source: &str,
) -> Vec<String> {
    match fixture.pass {
        TransformPassKind::ScopeFlatten => sorted_unique(
            collect_scope_flatten_proof_candidates_with_lexer(source, fixture.dialect)
                .into_iter()
                .map(|candidate| {
                    format!(
                        "scope:{}..{}:accepted={}:blocked={:?}:root={}:witness={}",
                        candidate.source_span_start,
                        candidate.source_span_end,
                        candidate.proof.accepted,
                        candidate.proof.blocked_reason,
                        candidate.proof.root_selector,
                        candidate.proof.cascade_safe_witness
                    )
                })
                .collect::<Vec<_>>(),
        ),
        TransformPassKind::LayerFlatten => sorted_unique(
            collect_layer_flatten_proof_candidates_with_lexer(
                source,
                fixture.dialect,
                fixture.closed_bundle,
            )
            .into_iter()
            .map(|candidate| {
                format!(
                    "layer:{}..{}:accepted={}:blocked={:?}:name={:?}:witness={}",
                    candidate.source_span_start,
                    candidate.source_span_end,
                    candidate.proof.accepted,
                    candidate.proof.blocked_reason,
                    candidate.proof.layer_name,
                    candidate.proof.cascade_safe_witness
                )
            })
            .collect::<Vec<_>>(),
        ),
        _ => Vec::new(),
    }
}

fn mutation_span_values(spans: Vec<TransformProvenanceMutationSpanV0>) -> Vec<String> {
    spans
        .into_iter()
        .map(|span| {
            format!(
                "{}..{}=>{}..{}",
                span.source_span_start,
                span.source_span_end,
                span.generated_span_start,
                span.generated_span_end
            )
        })
        .collect()
}

fn semantic_removal_values(removals: Vec<TransformSemanticRemovalCandidate>) -> Vec<String> {
    removals
        .into_iter()
        .map(|removal| {
            format!(
                "{}:{}:{}..{}:{}",
                removal.symbol_kind,
                removal.name,
                removal.source_span_start,
                removal.source_span_end,
                removal.reason
            )
        })
        .collect()
}

fn public_semantic_removal_values(removals: Vec<TransformSemanticRemovalV0>) -> Vec<String> {
    removals
        .into_iter()
        .map(|removal| {
            format!(
                "{}:{}:{}..{}:{}",
                removal.symbol_kind,
                removal.name,
                removal.source_span_start,
                removal.source_span_end,
                removal.reason
            )
        })
        .collect()
}

fn string_vec<const N: usize>(values: [&str; N]) -> Vec<String> {
    values.into_iter().map(str::to_string).collect()
}

fn shadow_field_report(
    field: &'static str,
    string_path_values: impl IntoIterator<Item = String>,
    ir_path_values: impl IntoIterator<Item = String>,
) -> TransformStructuralIrShadowFieldReportV0 {
    let ir_path_values = sorted_unique(ir_path_values);
    shadow_field_report_with_typed(
        field,
        string_path_values,
        ir_path_values.clone(),
        ir_path_values,
    )
}

fn shadow_field_report_with_typed(
    field: &'static str,
    string_path_values: impl IntoIterator<Item = String>,
    ir_path_values: impl IntoIterator<Item = String>,
    typed_path_values: impl IntoIterator<Item = String>,
) -> TransformStructuralIrShadowFieldReportV0 {
    let string_path_values = sorted_unique(string_path_values);
    let ir_path_values = sorted_unique(ir_path_values);
    let typed_path_values = sorted_unique(typed_path_values);
    let matches = string_path_values == ir_path_values && string_path_values == typed_path_values;
    TransformStructuralIrShadowFieldReportV0 {
        field,
        string_path_values,
        ir_path_values,
        typed_path_values,
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

fn dialect_label(dialect: StyleDialect) -> &'static str {
    match dialect {
        StyleDialect::Css => "css",
        StyleDialect::Scss => "scss",
        StyleDialect::Sass => "sass",
        StyleDialect::Less => "less",
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use serde::Deserialize;

    use super::*;

    const STRING_AUTHORITY_STRUCTURAL_GOLDEN: &str =
        include_str!("../../data/string-authority-structural-golden-v0.json");

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct StringAuthorityStructuralGoldenEntryV0 {
        fixture: String,
        pass_id: String,
        dialect: String,
        output_css: String,
    }

    #[test]
    fn structural_ir_output_matches_string_authority_golden() -> Result<(), String> {
        let entries = serde_json::from_str::<Vec<StringAuthorityStructuralGoldenEntryV0>>(
            STRING_AUTHORITY_STRUCTURAL_GOLDEN,
        )
        .map_err(|err| format!("String authority structural golden should parse: {err:?}"))?;
        let mut entries_by_key = BTreeMap::new();
        for entry in entries {
            let key = (entry.fixture.clone(), entry.pass_id.clone());
            if entries_by_key.insert(key.clone(), entry).is_some() {
                return Err(format!(
                    "String authority structural golden contains duplicate fixture/pass key {key:?}"
                ));
            }
        }

        let fixtures = structural_shadow_fixtures();
        assert_eq!(
            entries_by_key.len(),
            fixtures.len(),
            "String authority structural golden must cover every structural shadow fixture"
        );

        for fixture in fixtures {
            let key = (fixture.fixture.to_string(), fixture.pass.id().to_string());
            let Some(golden) = entries_by_key.remove(&key) else {
                return Err(format!(
                    "String authority structural golden is missing fixture/pass key {key:?}"
                ));
            };
            assert_eq!(golden.dialect, dialect_label(fixture.dialect));
            let snapshot = ir_path_snapshot(fixture)?;
            assert_eq!(
                snapshot.output_css, golden.output_css,
                "IR output drifted from String authority golden for {} / {}",
                fixture.fixture, golden.pass_id
            );
        }

        assert!(
            entries_by_key.is_empty(),
            "String authority structural golden contains stale keys: {:?}",
            entries_by_key.keys().collect::<Vec<_>>()
        );
        Ok(())
    }

    #[test]
    fn typed_payload_projection_memo_distinguishes_same_node_after_source_change() {
        let mut memo = TypedPayloadProjectionMemoV0::default();
        let node_id = IrNodeIdV0(7);
        let revision = IncrementalRevisionV0 { value: 1 };

        let first = memo.project_style_rule_payload(
            node_id,
            ".card { &__icon { color: red; } }",
            StyleDialect::Scss,
            revision,
        );
        let second = memo.project_style_rule_payload(
            node_id,
            ".card { &__icon { color: red; } }",
            StyleDialect::Scss,
            revision,
        );
        let changed = memo.project_style_rule_payload(
            node_id,
            ".card { &__icon { color: blue; } }",
            StyleDialect::Scss,
            revision,
        );

        assert!(first.is_some());
        assert!(second.is_some());
        assert!(changed.is_some());
        assert_eq!(memo.entries.len(), 2);
        assert_eq!(memo.telemetry.projections_consumed, 3);
        assert_eq!(memo.telemetry.memo_hits, 1);
    }
}
