use omena_parser::StyleDialect;
use omena_transform_cst::TransformPassKind;

use super::{
    planner::{plan_transform_passes, transform_pass_kind_from_id},
    provenance::{derive_transform_mutation_spans, provenance_derivation_forest_from_outcomes},
};
use crate::{
    TransformExecutionContextV0, TransformExecutionSummaryV0, TransformPassExecutionOutcomeV0,
    TransformPassRuntimeStatus, add_css_vendor_prefixes, combine_css_shorthands,
    compress_css_colors, compress_css_is_where_selectors, compress_css_numbers,
    dedupe_exact_css_rules, evaluate_dead_media_branch_rules, evaluate_static_media_rules,
    evaluate_static_supports_rules, flatten_css_layers, flatten_css_scopes, inline_css_imports,
    lower_css_color_function, lower_css_color_mix, lower_css_light_dark,
    lower_css_logical_to_physical, lower_css_oklab_oklch, merge_adjacent_same_block_css_selectors,
    merge_adjacent_same_selector_css_rules, normalize_css_string_quotes, normalize_css_units,
    normalize_css_whitespace, reachable_class_names_with_composes_exports, reduce_css_calc,
    remove_empty_css_rules, resolve_css_module_composes, resolve_static_css_modules_values,
    rewrite_css_module_class_names, route_design_token_values, strip_css_comments,
    strip_css_url_quotes, substitute_static_css_custom_properties,
    tree_shake_css_class_rules_with_removals, tree_shake_css_custom_properties_with_removals,
    tree_shake_css_keyframes_with_removals, tree_shake_css_modules_values_with_removals,
    unwrap_css_nesting,
};

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
    let reachable_class_names = reachable_class_names_with_composes_exports(
        &context.reachable_class_names,
        &context.css_module_composes_resolutions,
    );
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
                    detail: "normalized safe CSS string tokens, declaration-scoped font family strings, and static font keyword aliases",
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
                    detail: "compressed :is/:where selector functions and keyframe selector aliases only when matching semantics are preserved",
                }
            }
            Some(TransformPassKind::ShorthandCombining) => {
                let (next_css, mutation_count) = combine_css_shorthands(&output_css, dialect);
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
                    detail: "combined safe shorthand declarations and adjacent longhands only with cascade-preserving proofs",
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
                    detail: "lowered light-dark() color references into dark media branches",
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
                    detail: "evaluated literal @media all/not all branches and normalized simple min/max media ranges",
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
                    &reachable_class_names,
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
                    &reachable_class_names,
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
                    &reachable_class_names,
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
                    &context.reachable_keyframe_names,
                    &reachable_class_names,
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
                let (next_css, mutation_count) = resolve_static_css_modules_values(
                    &output_css,
                    dialect,
                    &context.css_module_value_resolutions,
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
                    detail: "reduced whole-value CSS math functions with static same-unit arithmetic and identity operations",
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
