use std::collections::BTreeSet;

use omena_parser::{
    StyleDialect, summarize_omena_parser_parity_lite, summarize_omena_parser_style_facts,
};
use omena_transform_cst::TransformPassKind;

use super::provenance::derive_transform_mutation_spans;
use crate::{
    TransformProvenanceMutationSpanV0, TransformSemanticRemovalCandidate,
    TransformStructuralIrShadowEquivalenceReportV0, TransformStructuralIrShadowFieldReportV0,
    TransformStructuralIrShadowFixtureReportV0,
    domains::{
        cascade_flatten::{
            collect_layer_flatten_proof_candidates_with_lexer,
            collect_scope_flatten_proof_candidates_with_lexer,
            flatten_css_layers_with_ir_transaction, flatten_css_layers_with_lexer,
            flatten_css_scopes_with_ir_transaction, flatten_css_scopes_with_lexer,
        },
        css_modules_classes::{
            local_css_module_composes_resolutions_with_lexer,
            rewrite_css_module_class_names_with_ir_transaction,
            rewrite_css_module_class_names_with_lexer,
            strip_resolved_css_module_composes_with_ir_transaction,
            strip_resolved_css_module_composes_with_lexer,
            tree_shake_css_class_rules_with_ir_transaction, tree_shake_css_class_rules_with_lexer,
        },
        css_modules_values::{
            tree_shake_css_modules_values_with_ir_transaction,
            tree_shake_css_modules_values_with_lexer,
        },
        custom_property::{
            tree_shake_css_custom_properties_with_ir_transaction,
            tree_shake_css_custom_properties_with_lexer,
        },
        design_token::{
            route_design_token_values_with_ir_transaction, route_design_token_values_with_lexer,
        },
        import_inline::{inline_css_imports_with_ir_transaction, inline_css_imports_with_lexer},
        keyframes::{
            tree_shake_css_keyframes_with_ir_transaction, tree_shake_css_keyframes_with_lexer,
        },
        nesting::{unwrap_css_nesting_with_ir_transaction, unwrap_css_nesting_with_lexer},
        rule_cleanup::{
            dedupe_exact_css_rules_with_ir_transaction, dedupe_exact_css_rules_with_lexer,
            remove_empty_css_rules_with_ir_transaction, remove_empty_css_rules_with_lexer,
        },
        rule_merge::{
            merge_adjacent_same_block_css_selectors_with_ir_transaction,
            merge_adjacent_same_block_css_selectors_with_lexer,
            merge_adjacent_same_selector_css_rules_with_ir_transaction,
            merge_adjacent_same_selector_css_rules_with_lexer,
        },
        static_eval::{
            StaticMediaEvaluationOptions, evaluate_static_container_rules_with_ir_transaction,
            evaluate_static_container_rules_with_lexer,
            evaluate_static_media_rules_with_ir_transaction,
            evaluate_static_media_rules_with_lexer,
            evaluate_static_supports_rules_with_ir_transaction,
            evaluate_static_supports_rules_with_lexer,
        },
    },
    model::{
        TransformClassNameRewriteV0, TransformCssModuleComposesResolutionV0,
        TransformDesignTokenRouteV0, TransformImportInlineV0,
    },
};

const COMPARED_FIELDS: [&str; 11] = [
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
];

#[derive(Debug, Clone, Copy)]
pub struct TransformStructuralIrShadowFixtureInputV0<'source> {
    pub fixture: &'source str,
    pub pass: TransformPassKind,
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

    TransformStructuralIrShadowEquivalenceReportV0 {
        schema_version: "0",
        product: "omena-transform-passes.structural-ir-shadow-equivalence",
        fixture_count: reports.len(),
        compared_pass_ids: compared_pass_ids(),
        compared_fields: COMPARED_FIELDS.to_vec(),
        reports,
        all_fields_match,
    }
}

fn structural_shadow_report_for_fixture(
    fixture: TransformStructuralIrShadowFixtureInputV0<'_>,
) -> TransformStructuralIrShadowFixtureReportV0 {
    let string_snapshot = string_path_snapshot(fixture);
    let ir_snapshot = ir_path_snapshot(fixture);
    let (ir_path_mutation_count, fields) = match ir_snapshot {
        Ok(ir_snapshot) => (
            Some(ir_snapshot.mutation_count),
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
            ],
        ),
        Err(error) => {
            let error = format!("irPathError:{error}");
            (
                None,
                vec![
                    shadow_field_report(
                        "canonicalCssBytes",
                        [string_snapshot.output_css],
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
                ],
            )
        }
    };
    let all_fields_match = fields.iter().all(|field| field.matches);

    TransformStructuralIrShadowFixtureReportV0 {
        schema_version: "0",
        product: "omena-transform-passes.structural-ir-shadow-fixture",
        fixture: fixture.fixture.to_string(),
        pass_id: fixture.pass.id(),
        dialect: dialect_label(fixture.dialect),
        string_path_mutation_count: Some(string_snapshot.mutation_count),
        ir_path_mutation_count,
        fields,
        all_fields_match,
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
            let (output_css, mutation_count) =
                evaluate_static_supports_rules_with_lexer(fixture.source, fixture.dialect);
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
    )
}

fn ir_path_snapshot(
    fixture: TransformStructuralIrShadowFixtureInputV0<'_>,
) -> Result<StructuralShadowPathSnapshotV0, String> {
    let reachability = reachability_for_fixture(fixture);
    let module_context = module_context_for_fixture(fixture);
    let (output_css, mutation_count, semantic_removal_values) = match fixture.pass {
        TransformPassKind::NestingUnwrap => {
            let (output_css, mutation_count) =
                unwrap_css_nesting_with_ir_transaction(fixture.source, fixture.dialect)
                    .map_err(|error| format!("{error:?}"))?;
            (output_css, mutation_count, Vec::new())
        }
        TransformPassKind::ScopeFlatten => {
            let (output_css, mutation_count) =
                flatten_css_scopes_with_ir_transaction(fixture.source, fixture.dialect)
                    .map_err(|error| format!("{error:?}"))?;
            (output_css, mutation_count, Vec::new())
        }
        TransformPassKind::LayerFlatten => {
            let (output_css, mutation_count) = flatten_css_layers_with_ir_transaction(
                fixture.source,
                fixture.dialect,
                fixture.closed_bundle,
            )
            .map_err(|error| format!("{error:?}"))?;
            (output_css, mutation_count, Vec::new())
        }
        TransformPassKind::RuleDeduplication => {
            let (output_css, mutation_count) =
                dedupe_exact_css_rules_with_ir_transaction(fixture.source, fixture.dialect)
                    .map_err(|error| format!("{error:?}"))?;
            (output_css, mutation_count, Vec::new())
        }
        TransformPassKind::RuleMerging => {
            let (output_css, mutation_count) =
                merge_adjacent_same_selector_css_rules_with_ir_transaction(
                    fixture.source,
                    fixture.dialect,
                )
                .map_err(|error| format!("{error:?}"))?;
            (output_css, mutation_count, Vec::new())
        }
        TransformPassKind::SelectorMerging => {
            let (output_css, mutation_count) =
                merge_adjacent_same_block_css_selectors_with_ir_transaction(
                    fixture.source,
                    fixture.dialect,
                )
                .map_err(|error| format!("{error:?}"))?;
            (output_css, mutation_count, Vec::new())
        }
        TransformPassKind::EmptyRuleRemoval => {
            let (output_css, mutation_count) =
                remove_empty_css_rules_with_ir_transaction(fixture.source, fixture.dialect)
                    .map_err(|error| format!("{error:?}"))?;
            (output_css, mutation_count, Vec::new())
        }
        TransformPassKind::SupportsStaticEval | TransformPassKind::DeadSupportsBranchRemoval => {
            let (output_css, mutation_count) =
                evaluate_static_supports_rules_with_ir_transaction(fixture.source, fixture.dialect)
                    .map_err(|error| format!("{error:?}"))?;
            (output_css, mutation_count, Vec::new())
        }
        TransformPassKind::MediaStaticEval | TransformPassKind::DeadMediaBranchRemoval => {
            let (output_css, mutation_count) = evaluate_static_media_rules_with_ir_transaction(
                fixture.source,
                fixture.dialect,
                StaticMediaEvaluationOptions::default(),
            )
            .map_err(|error| format!("{error:?}"))?;
            (output_css, mutation_count, Vec::new())
        }
        TransformPassKind::ContainerStaticEval => {
            let (output_css, mutation_count) = evaluate_static_container_rules_with_ir_transaction(
                fixture.source,
                fixture.dialect,
            )
            .map_err(|error| format!("{error:?}"))?;
            (output_css, mutation_count, Vec::new())
        }
        TransformPassKind::TreeShakeClass => {
            let (output_css, removals) = tree_shake_css_class_rules_with_ir_transaction(
                fixture.source,
                fixture.dialect,
                reachability.class_names.as_slice(),
            )
            .map_err(|error| format!("{error:?}"))?;
            let mutation_count = removals.len();
            (
                output_css,
                mutation_count,
                semantic_removal_values(removals),
            )
        }
        TransformPassKind::TreeShakeKeyframes => {
            let (output_css, removals) = tree_shake_css_keyframes_with_ir_transaction(
                fixture.source,
                fixture.dialect,
                reachability.keyframe_names.as_slice(),
                reachability.class_names.as_slice(),
            )
            .map_err(|error| format!("{error:?}"))?;
            let mutation_count = removals.len();
            (
                output_css,
                mutation_count,
                semantic_removal_values(removals),
            )
        }
        TransformPassKind::TreeShakeValue => {
            let (output_css, removals) = tree_shake_css_modules_values_with_ir_transaction(
                fixture.source,
                fixture.dialect,
                reachability.value_names.as_slice(),
                reachability.keyframe_names.as_slice(),
                reachability.class_names.as_slice(),
            )
            .map_err(|error| format!("{error:?}"))?;
            let mutation_count = removals.len();
            (
                output_css,
                mutation_count,
                semantic_removal_values(removals),
            )
        }
        TransformPassKind::TreeShakeCustomProperty => {
            let (output_css, removals) = tree_shake_css_custom_properties_with_ir_transaction(
                fixture.source,
                fixture.dialect,
                reachability.custom_property_names.as_slice(),
                reachability.keyframe_names.as_slice(),
                reachability.class_names.as_slice(),
            )
            .map_err(|error| format!("{error:?}"))?;
            let mutation_count = removals.len();
            (
                output_css,
                mutation_count,
                semantic_removal_values(removals),
            )
        }
        TransformPassKind::ImportInline => {
            let (output_css, mutation_count) = inline_css_imports_with_ir_transaction(
                fixture.source,
                fixture.dialect,
                module_context.import_inlines.as_slice(),
            )
            .map_err(|error| format!("{error:?}"))?;
            (output_css, mutation_count, Vec::new())
        }
        TransformPassKind::ResolveCssModulesComposes => {
            let resolutions = css_module_composes_resolutions_for_fixture(fixture, &module_context);
            let (output_css, mutation_count) =
                strip_resolved_css_module_composes_with_ir_transaction(
                    fixture.source,
                    fixture.dialect,
                    resolutions.as_slice(),
                )
                .map_err(|error| format!("{error:?}"))?;
            (output_css, mutation_count, Vec::new())
        }
        TransformPassKind::HashCssModuleClassNames => {
            let (output_css, mutation_count) = rewrite_css_module_class_names_with_ir_transaction(
                fixture.source,
                fixture.dialect,
                module_context.class_name_rewrites.as_slice(),
            )
            .map_err(|error| format!("{error:?}"))?;
            (output_css, mutation_count, Vec::new())
        }
        TransformPassKind::DesignTokenRouting => {
            let (output_css, mutation_count) = route_design_token_values_with_ir_transaction(
                fixture.source,
                fixture.dialect,
                module_context.design_token_routes.as_slice(),
            )
            .map_err(|error| format!("{error:?}"))?;
            (output_css, mutation_count, Vec::new())
        }
        _ => (fixture.source.to_string(), 0, Vec::new()),
    };
    Ok(path_snapshot_from_output(
        fixture,
        output_css,
        mutation_count,
        semantic_removal_values,
        module_egress_values_for_fixture(fixture, &module_context),
    ))
}

fn path_snapshot_from_output(
    fixture: TransformStructuralIrShadowFixtureInputV0<'_>,
    output_css: String,
    mutation_count: usize,
    semantic_removal_values: Vec<String>,
    module_egress_values: StructuralShadowModuleEgressValuesV0,
) -> StructuralShadowPathSnapshotV0 {
    StructuralShadowPathSnapshotV0 {
        selector_values: selector_values_for_source(&output_css, fixture.dialect),
        declaration_values: declaration_values_for_source(&output_css, fixture.dialect),
        cascade_values: cascade_values_for_fixture(fixture),
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
    vec![
        "container-static-eval",
        "dead-media-branch-removal",
        "dead-supports-branch-removal",
        "composes-resolution",
        "css-modules-class-hashing",
        "design-token-routing",
        "empty-rule-removal",
        "import-inline",
        "layer-flatten",
        "media-static-eval",
        "nesting-unwrap",
        "rule-deduplication",
        "rule-merging",
        "scope-flatten",
        "selector-merging",
        "supports-static-eval",
        "tree-shake-class",
        "tree-shake-custom-property",
        "tree-shake-keyframes",
        "tree-shake-value",
    ]
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
        _ => StructuralShadowReachabilityV0 {
            class_names: Vec::new(),
            keyframe_names: Vec::new(),
            value_names: Vec::new(),
            custom_property_names: Vec::new(),
        },
    }
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

fn cascade_values_for_fixture(
    fixture: TransformStructuralIrShadowFixtureInputV0<'_>,
) -> Vec<String> {
    match fixture.pass {
        TransformPassKind::ScopeFlatten => sorted_unique(
            collect_scope_flatten_proof_candidates_with_lexer(fixture.source, fixture.dialect)
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
                fixture.source,
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

fn string_vec<const N: usize>(values: [&str; N]) -> Vec<String> {
    values.into_iter().map(str::to_string).collect()
}

fn shadow_field_report(
    field: &'static str,
    string_path_values: impl IntoIterator<Item = String>,
    ir_path_values: impl IntoIterator<Item = String>,
) -> TransformStructuralIrShadowFieldReportV0 {
    let string_path_values = sorted_unique(string_path_values);
    let ir_path_values = sorted_unique(ir_path_values);
    let matches = string_path_values == ir_path_values;
    TransformStructuralIrShadowFieldReportV0 {
        field,
        string_path_values,
        ir_path_values,
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
