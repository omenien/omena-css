//! Transform pass execution over source text and explicit workspace context.
//!
//! The executor is the mutation boundary for ordered transform plans. It applies
//! registered pass kinds, records provenance outcomes, and preserves semantic
//! removal evidence for downstream query and consumer surfaces.

use omena_parser::StyleDialect;
use omena_transform_cst::TransformPassKind;

use super::{
    cascade_proof::{
        collect_cascade_proof_obligations_for_pass_input, summarize_cascade_proof_obligations,
    },
    outcome::{mutation_outcome, no_change_outcome, planned_only_outcome},
    planner::{plan_transform_passes, transform_pass_kind_from_id},
    provenance::{derive_transform_mutation_spans, provenance_derivation_forest_from_outcomes},
};
use crate::model::{
    TransformExecutionContextV0, TransformExecutionSummaryV0, TransformPassRuntimeStatus,
};
use crate::registry::{
    add_css_vendor_prefixes, combine_css_shorthands, compress_css_colors,
    compress_css_is_where_selectors, compress_css_numbers,
    css_module_composes_resolutions_for_source, dedupe_exact_css_rules,
    evaluate_dead_media_branch_rules, evaluate_static_media_rules, evaluate_static_supports_rules,
    flatten_css_layers, flatten_css_scopes, inline_css_imports, lower_css_color_function,
    lower_css_color_mix, lower_css_light_dark, lower_css_logical_to_physical,
    lower_css_oklab_oklch, merge_adjacent_same_block_css_selectors,
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

#[cfg(feature = "lawvere-trace")]
pub fn execute_transform_passes_on_source_with_lawvere_trace(
    source: &str,
    requested: &[TransformPassKind],
) -> (
    TransformExecutionSummaryV0,
    omena_lawvere::LawvereModelTraceV0,
) {
    execute_transform_passes_on_source_with_lawvere_trace_and_dialect(
        source,
        StyleDialect::Css,
        requested,
    )
}

#[cfg(feature = "lawvere-trace")]
pub fn execute_transform_passes_on_source_with_lawvere_trace_and_dialect(
    source: &str,
    dialect: StyleDialect,
    requested: &[TransformPassKind],
) -> (
    TransformExecutionSummaryV0,
    omena_lawvere::LawvereModelTraceV0,
) {
    let summary = execute_transform_passes_on_source_with_dialect(source, dialect, requested);
    let trace = omena_lawvere::trace_lawvere_model_v0(requested, summary.ordered_pass_ids.clone());
    (summary, trace)
}

#[cfg(feature = "lawvere-trace")]
pub fn evaluate_lawvere_reorderability_with_differential_corpus(
    left: TransformPassKind,
    right: TransformPassKind,
    fixtures: &[&str],
) -> (
    omena_lawvere::ReorderabilityCertificateV0,
    omena_lawvere::LawvereDifferentialCommutativityWitnessV0,
) {
    let cases = fixtures
        .iter()
        .enumerate()
        .map(|(index, source)| {
            let left_first = execute_transform_passes_on_source(source, &[left]);
            let left_then_right =
                execute_transform_passes_on_source(&left_first.output_css, &[right]);
            let right_first = execute_transform_passes_on_source(source, &[right]);
            let right_then_left =
                execute_transform_passes_on_source(&right_first.output_css, &[left]);
            let left_then_right_mutation_count =
                left_first.mutation_count + left_then_right.mutation_count;
            let right_then_left_mutation_count =
                right_first.mutation_count + right_then_left.mutation_count;

            omena_lawvere::LawvereDifferentialCommutativityCaseV0 {
                label: format!("fixture-{index}"),
                input_css: (*source).to_string(),
                left_then_right_css: left_then_right.output_css.clone(),
                right_then_left_css: right_then_left.output_css.clone(),
                left_then_right_mutation_count,
                right_then_left_mutation_count,
                equal_output: left_then_right.output_css == right_then_left.output_css,
            }
        })
        .collect::<Vec<_>>();
    let witness = omena_lawvere::lawvere_differential_commutativity_witness_v0(left, right, cases);
    let certificate =
        omena_lawvere::reorderability_certificate_from_differential_v0(left, right, &witness);
    (certificate, witness)
}

pub fn execute_transform_passes_on_source_with_dialect_and_context(
    source: &str,
    dialect: StyleDialect,
    requested: &[TransformPassKind],
    context: &TransformExecutionContextV0,
) -> TransformExecutionSummaryV0 {
    super::lex_cache::with_transform_lex_cache(|| {
        execute_transform_passes_on_source_with_active_lex_cache(
            source, dialect, requested, context,
        )
    })
}

fn execute_transform_passes_on_source_with_active_lex_cache(
    source: &str,
    dialect: StyleDialect,
    requested: &[TransformPassKind],
    context: &TransformExecutionContextV0,
) -> TransformExecutionSummaryV0 {
    let pass_plan = plan_transform_passes(requested);
    let requested_pass_ids = requested.iter().map(|pass| pass.id()).collect::<Vec<_>>();
    let ordered_pass_ids = pass_plan.ordered_pass_ids.clone();
    let reachable_class_names = reachable_class_names_with_composes_exports(
        source,
        dialect,
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
    let mut cascade_proof_obligations = Vec::new();

    macro_rules! apply_mutation_pass {
        ($pass_id:expr, $input_byte_len:expr, $run:expr, $detail:literal) => {{
            let (next_css, mutation_count) = $run;
            let outcome = mutation_outcome(
                $pass_id,
                $input_byte_len,
                next_css.len(),
                mutation_count,
                $detail,
            );
            (Some(next_css), outcome)
        }};
    }

    macro_rules! planned_only_pass {
        ($pass_id:expr, $input_byte_len:expr, $detail:literal) => {
            (
                None,
                planned_only_outcome($pass_id, $input_byte_len, $input_byte_len, $detail),
            )
        };
    }

    macro_rules! no_change_pass {
        ($pass_id:expr, $input_byte_len:expr, $detail:literal) => {
            (
                None,
                no_change_outcome($pass_id, $input_byte_len, $input_byte_len, $detail),
            )
        };
    }

    for (pass_index, pass_id) in ordered_pass_ids.iter().enumerate() {
        let has_remaining_lex_consumers = ordered_pass_ids
            .iter()
            .skip(pass_index + 1)
            .filter_map(|pass_id| transform_pass_kind_from_id(pass_id))
            .any(transform_pass_may_consume_lex_cache);
        let pass = transform_pass_kind_from_id(pass_id);
        let pass_input_css = output_css;
        let input_byte_len = pass_input_css.len();
        cascade_proof_obligations.extend(collect_cascade_proof_obligations_for_pass_input(
            pass_id,
            pass,
            &pass_input_css,
            dialect,
            context,
        ));
        let (next_output_css, outcome) = match pass {
            Some(TransformPassKind::WhitespaceStrip) => {
                apply_mutation_pass!(
                    pass_id,
                    input_byte_len,
                    normalize_css_whitespace(&pass_input_css, dialect),
                    "normalized lexer trivia where adjacent token boundaries remain unambiguous"
                )
            }
            Some(TransformPassKind::CommentStrip) => {
                apply_mutation_pass!(
                    pass_id,
                    input_byte_len,
                    strip_css_comments(&pass_input_css, dialect),
                    "removed CSS block comments outside string literals"
                )
            }
            Some(TransformPassKind::NumberCompression) => {
                apply_mutation_pass!(
                    pass_id,
                    input_byte_len,
                    compress_css_numbers(&pass_input_css, dialect),
                    "compressed lexer numeric tokens without touching identifiers or strings"
                )
            }
            Some(TransformPassKind::UnitNormalization) => {
                apply_mutation_pass!(
                    pass_id,
                    input_byte_len,
                    normalize_css_units(&pass_input_css, dialect),
                    "normalized zero length units and known CSS unit casing inside declaration contexts"
                )
            }
            Some(TransformPassKind::ColorCompression) => {
                apply_mutation_pass!(
                    pass_id,
                    input_byte_len,
                    compress_css_colors(&pass_input_css, dialect),
                    "compressed static declaration color values and hex color tokens"
                )
            }
            Some(TransformPassKind::UrlQuoteStrip) => {
                apply_mutation_pass!(
                    pass_id,
                    input_byte_len,
                    strip_css_url_quotes(&pass_input_css, dialect),
                    "stripped quotes from safe url() string arguments"
                )
            }
            Some(TransformPassKind::StringQuoteNormalize) => {
                apply_mutation_pass!(
                    pass_id,
                    input_byte_len,
                    normalize_css_string_quotes(&pass_input_css, dialect),
                    "normalized safe CSS string tokens, declaration-scoped font family strings, and static font keyword aliases"
                )
            }
            Some(TransformPassKind::SelectorIsWhereCompression) => {
                apply_mutation_pass!(
                    pass_id,
                    input_byte_len,
                    compress_css_is_where_selectors(&pass_input_css, dialect),
                    "compressed :is/:where selector functions and keyframe selector aliases only when matching semantics are preserved"
                )
            }
            Some(TransformPassKind::ShorthandCombining) => {
                apply_mutation_pass!(
                    pass_id,
                    input_byte_len,
                    combine_css_shorthands(&pass_input_css, dialect),
                    "combined safe shorthand declarations and adjacent longhands only with cascade-preserving proofs"
                )
            }
            Some(TransformPassKind::RuleDeduplication) => {
                apply_mutation_pass!(
                    pass_id,
                    input_byte_len,
                    dedupe_exact_css_rules(&pass_input_css, dialect),
                    "removed cascade-safe duplicate ordinary rules while preserving the final occurrence"
                )
            }
            Some(TransformPassKind::RuleMerging) => {
                apply_mutation_pass!(
                    pass_id,
                    input_byte_len,
                    merge_adjacent_same_selector_css_rules(&pass_input_css, dialect),
                    "merged adjacent same-selector ordinary rule runs without reordering declarations"
                )
            }
            Some(TransformPassKind::SelectorMerging) => {
                apply_mutation_pass!(
                    pass_id,
                    input_byte_len,
                    merge_adjacent_same_block_css_selectors(&pass_input_css, dialect),
                    "merged adjacent ordinary rule runs with identical declaration blocks"
                )
            }
            Some(TransformPassKind::VendorPrefixing) => {
                apply_mutation_pass!(
                    pass_id,
                    input_byte_len,
                    add_css_vendor_prefixes(&pass_input_css, dialect),
                    "inserted conservative vendor-prefixed declaration synonyms when absent"
                )
            }
            Some(TransformPassKind::LightDarkLowering) => {
                apply_mutation_pass!(
                    pass_id,
                    input_byte_len,
                    lower_css_light_dark(&pass_input_css, dialect),
                    "lowered light-dark() color references into dark media branches"
                )
            }
            Some(TransformPassKind::ColorMixLowering) => {
                apply_mutation_pass!(
                    pass_id,
                    input_byte_len,
                    lower_css_color_mix(&pass_input_css, dialect),
                    "lowered static srgb color-mix() references with static color operands"
                )
            }
            Some(TransformPassKind::OklchOklabLowering) => {
                apply_mutation_pass!(
                    pass_id,
                    input_byte_len,
                    lower_css_oklab_oklch(&pass_input_css, dialect),
                    "lowered in-gamut oklab()/oklch() color references to srgb"
                )
            }
            Some(TransformPassKind::ColorFunctionLowering) => {
                apply_mutation_pass!(
                    pass_id,
                    input_byte_len,
                    lower_css_color_function(&pass_input_css, dialect),
                    "lowered static color(...) references with static channels"
                )
            }
            Some(TransformPassKind::LogicalToPhysical) => {
                apply_mutation_pass!(
                    pass_id,
                    input_byte_len,
                    lower_css_logical_to_physical(&pass_input_css, dialect),
                    "lowered logical properties only under static horizontal writing direction"
                )
            }
            Some(TransformPassKind::NestingUnwrap) => {
                apply_mutation_pass!(
                    pass_id,
                    input_byte_len,
                    unwrap_css_nesting(&pass_input_css, dialect),
                    "unwrapped nested ordinary rules and conditional group rules"
                )
            }
            Some(TransformPassKind::ScopeFlatten) => {
                apply_mutation_pass!(
                    pass_id,
                    input_byte_len,
                    flatten_css_scopes(&pass_input_css, dialect),
                    "flattened only @scope candidates accepted by the cascade scope-flatten proof"
                )
            }
            Some(TransformPassKind::LayerFlatten) if context.closed_style_world => {
                apply_mutation_pass!(
                    pass_id,
                    input_byte_len,
                    flatten_css_layers(&pass_input_css, dialect, true),
                    "flattened only @layer candidates accepted by the closed-bundle cascade proof"
                )
            }
            Some(TransformPassKind::LayerFlatten) => {
                planned_only_pass!(
                    pass_id,
                    input_byte_len,
                    "requires an explicit closed-style-world bundle witness before mutation"
                )
            }
            Some(TransformPassKind::SupportsStaticEval) => {
                apply_mutation_pass!(
                    pass_id,
                    input_byte_len,
                    evaluate_static_supports_rules(&pass_input_css, dialect),
                    "evaluated simple @supports branches with cascade supports-static witness"
                )
            }
            Some(TransformPassKind::MediaStaticEval) => {
                apply_mutation_pass!(
                    pass_id,
                    input_byte_len,
                    evaluate_static_media_rules(&pass_input_css, dialect),
                    "evaluated literal @media all/not all branches and normalized simple min/max media ranges"
                )
            }
            Some(TransformPassKind::DeadMediaBranchRemoval) => {
                apply_mutation_pass!(
                    pass_id,
                    input_byte_len,
                    evaluate_dead_media_branch_rules(&pass_input_css, dialect, context),
                    "removed dead @media branches through the static cascade witness evaluator"
                )
            }
            Some(TransformPassKind::DeadSupportsBranchRemoval) => {
                apply_mutation_pass!(
                    pass_id,
                    input_byte_len,
                    evaluate_static_supports_rules(&pass_input_css, dialect),
                    "removed dead @supports branches through the static cascade witness evaluator"
                )
            }
            Some(TransformPassKind::ScssModuleEvaluate)
                if matches!(dialect, StyleDialect::Scss | StyleDialect::Sass) =>
            {
                if let Some(evaluation) = context.scss_module_evaluation.as_ref() {
                    let mutation_count = usize::from(pass_input_css != evaluation.evaluated_css);
                    let next_css = evaluation.evaluated_css.clone();
                    let outcome = mutation_outcome(
                        pass_id,
                        input_byte_len,
                        next_css.len(),
                        mutation_count,
                        "applied explicit SCSS module evaluation output from the evaluator boundary",
                    );
                    css_module_evaluation = Some(evaluation.clone());
                    (Some(next_css), outcome)
                } else {
                    planned_only_pass!(
                        pass_id,
                        input_byte_len,
                        "requires explicit SCSS evaluator output before mutation"
                    )
                }
            }
            Some(TransformPassKind::ScssModuleEvaluate) => {
                planned_only_pass!(
                    pass_id,
                    input_byte_len,
                    "requires explicit SCSS evaluator output before mutation"
                )
            }
            Some(TransformPassKind::LessModuleEvaluate) if dialect == StyleDialect::Less => {
                if let Some(evaluation) = context.less_module_evaluation.as_ref() {
                    let mutation_count = usize::from(pass_input_css != evaluation.evaluated_css);
                    let next_css = evaluation.evaluated_css.clone();
                    let outcome = mutation_outcome(
                        pass_id,
                        input_byte_len,
                        next_css.len(),
                        mutation_count,
                        "applied explicit Less module evaluation output from the evaluator boundary",
                    );
                    css_module_evaluation = Some(evaluation.clone());
                    (Some(next_css), outcome)
                } else {
                    planned_only_pass!(
                        pass_id,
                        input_byte_len,
                        "requires explicit Less evaluator output before mutation"
                    )
                }
            }
            Some(TransformPassKind::LessModuleEvaluate) => {
                planned_only_pass!(
                    pass_id,
                    input_byte_len,
                    "requires explicit Less evaluator output before mutation"
                )
            }
            Some(TransformPassKind::ImportInline)
                if dialect == StyleDialect::Less || !context.import_inlines.is_empty() =>
            {
                let (next_css, mutation_count) =
                    inline_css_imports(&pass_input_css, dialect, &context.import_inlines);
                let outcome = mutation_outcome(
                    pass_id,
                    input_byte_len,
                    next_css.len(),
                    mutation_count,
                    "replaced resolved @import directives and optional Less imports",
                );
                css_import_inlines = context.import_inlines.clone();
                (Some(next_css), outcome)
            }
            Some(TransformPassKind::ImportInline) => {
                planned_only_pass!(
                    pass_id,
                    input_byte_len,
                    "requires explicit resolved import replacements before mutation"
                )
            }
            Some(TransformPassKind::ResolveCssModulesComposes) => {
                let resolutions = css_module_composes_resolutions_for_source(
                    &pass_input_css,
                    dialect,
                    &context.css_module_composes_resolutions,
                );
                if resolutions.is_empty() {
                    planned_only_pass!(
                        pass_id,
                        input_byte_len,
                        "requires CSS Modules composes declarations or an explicit export set before mutation"
                    )
                } else {
                    let (next_css, mutation_count) =
                        resolve_css_module_composes(&pass_input_css, dialect, &resolutions);
                    let outcome = mutation_outcome(
                        pass_id,
                        input_byte_len,
                        next_css.len(),
                        mutation_count,
                        "removed resolved CSS Modules composes declarations using an explicit export set",
                    );
                    css_module_composes_exports = resolutions;
                    (Some(next_css), outcome)
                }
            }
            Some(TransformPassKind::DesignTokenRouting)
                if !context.design_token_routes.is_empty() =>
            {
                let (next_css, mutation_count) = route_design_token_values(
                    &pass_input_css,
                    dialect,
                    &context.design_token_routes,
                );
                let outcome = mutation_outcome(
                    pass_id,
                    input_byte_len,
                    next_css.len(),
                    mutation_count,
                    "routed whole-value design-token references through explicit bridge token routes",
                );
                design_token_routes = context.design_token_routes.clone();
                (Some(next_css), outcome)
            }
            Some(TransformPassKind::DesignTokenRouting) => {
                planned_only_pass!(
                    pass_id,
                    input_byte_len,
                    "requires explicit bridge design-token routes before mutation"
                )
            }
            Some(TransformPassKind::HashCssModuleClassNames)
                if !context.class_name_rewrites.is_empty() =>
            {
                let (next_css, mutation_count) = rewrite_css_module_class_names(
                    &pass_input_css,
                    dialect,
                    &context.class_name_rewrites,
                );
                let outcome = mutation_outcome(
                    pass_id,
                    input_byte_len,
                    next_css.len(),
                    mutation_count,
                    "rewrote CSS Modules class selectors through an explicit selector identity map",
                );
                (Some(next_css), outcome)
            }
            Some(TransformPassKind::HashCssModuleClassNames) => {
                planned_only_pass!(
                    pass_id,
                    input_byte_len,
                    "requires an explicit selector identity map before mutation"
                )
            }
            Some(TransformPassKind::TreeShakeClass) if context.closed_style_world => {
                let (next_css, removals) = tree_shake_css_class_rules_with_removals(
                    &pass_input_css,
                    dialect,
                    &reachable_class_names,
                );
                let mutation_count = removals.len();
                let outcome = mutation_outcome(
                    pass_id,
                    input_byte_len,
                    next_css.len(),
                    mutation_count,
                    "removed unreachable class-owned selector rules under an explicit closed-style-world reachability context",
                );
                semantic_removals.extend(
                    removals
                        .into_iter()
                        .map(|removal| removal.into_public(pass_id)),
                );
                (Some(next_css), outcome)
            }
            Some(TransformPassKind::TreeShakeClass) => {
                planned_only_pass!(
                    pass_id,
                    input_byte_len,
                    "requires an explicit closed-style-world reachability context before mutation"
                )
            }
            Some(TransformPassKind::TreeShakeKeyframes) if context.closed_style_world => {
                let (next_css, removals) = tree_shake_css_keyframes_with_removals(
                    &pass_input_css,
                    dialect,
                    &context.reachable_keyframe_names,
                    &reachable_class_names,
                );
                let mutation_count = removals.len();
                let outcome = mutation_outcome(
                    pass_id,
                    input_byte_len,
                    next_css.len(),
                    mutation_count,
                    "removed unreferenced @keyframes under an explicit closed-style-world reachability context",
                );
                semantic_removals.extend(
                    removals
                        .into_iter()
                        .map(|removal| removal.into_public(pass_id)),
                );
                (Some(next_css), outcome)
            }
            Some(TransformPassKind::TreeShakeKeyframes) => {
                planned_only_pass!(
                    pass_id,
                    input_byte_len,
                    "requires an explicit closed-style-world reachability context before mutation"
                )
            }
            Some(TransformPassKind::TreeShakeValue) if context.closed_style_world => {
                let (next_css, removals) = tree_shake_css_modules_values_with_removals(
                    &pass_input_css,
                    dialect,
                    &context.reachable_value_names,
                    &context.reachable_keyframe_names,
                    &reachable_class_names,
                );
                let mutation_count = removals.len();
                let outcome = mutation_outcome(
                    pass_id,
                    input_byte_len,
                    next_css.len(),
                    mutation_count,
                    "removed unreachable local CSS Modules @value declarations under an explicit closed-style-world reachability context",
                );
                semantic_removals.extend(
                    removals
                        .into_iter()
                        .map(|removal| removal.into_public(pass_id)),
                );
                (Some(next_css), outcome)
            }
            Some(TransformPassKind::TreeShakeValue) => {
                planned_only_pass!(
                    pass_id,
                    input_byte_len,
                    "requires an explicit closed-style-world reachability context before mutation"
                )
            }
            Some(TransformPassKind::TreeShakeCustomProperty) if context.closed_style_world => {
                let (next_css, removals) = tree_shake_css_custom_properties_with_removals(
                    &pass_input_css,
                    dialect,
                    &context.reachable_custom_property_names,
                    &context.reachable_keyframe_names,
                    &reachable_class_names,
                );
                let mutation_count = removals.len();
                let outcome = mutation_outcome(
                    pass_id,
                    input_byte_len,
                    next_css.len(),
                    mutation_count,
                    "removed unreachable custom-property declarations under an explicit closed-style-world reachability context",
                );
                semantic_removals.extend(
                    removals
                        .into_iter()
                        .map(|removal| removal.into_public(pass_id)),
                );
                (Some(next_css), outcome)
            }
            Some(TransformPassKind::TreeShakeCustomProperty) => {
                planned_only_pass!(
                    pass_id,
                    input_byte_len,
                    "requires an explicit closed-style-world reachability context before mutation"
                )
            }
            Some(TransformPassKind::ValueResolution) => {
                apply_mutation_pass!(
                    pass_id,
                    input_byte_len,
                    resolve_static_css_modules_values(
                        &pass_input_css,
                        dialect,
                        &context.css_module_value_resolutions,
                    ),
                    "resolved whole-value references from unique local literal CSS Modules @value declarations"
                )
            }
            Some(TransformPassKind::StaticVarSubstitution) => {
                apply_mutation_pass!(
                    pass_id,
                    input_byte_len,
                    substitute_static_css_custom_properties(&pass_input_css, dialect),
                    "resolved whole-value var() references from unique static :root custom properties"
                )
            }
            Some(TransformPassKind::CalcReduction) => {
                apply_mutation_pass!(
                    pass_id,
                    input_byte_len,
                    reduce_css_calc(&pass_input_css, dialect),
                    "reduced whole-value CSS math functions with static same-unit arithmetic and identity operations"
                )
            }
            Some(TransformPassKind::EmptyRuleRemoval) => {
                apply_mutation_pass!(
                    pass_id,
                    input_byte_len,
                    remove_empty_css_rules(&pass_input_css, dialect),
                    "removed ordinary empty rules with no comments or at-rule semantics"
                )
            }
            Some(TransformPassKind::PrintCss) => {
                no_change_pass!(pass_id, input_byte_len, "observed final emission boundary")
            }
            None => {
                planned_only_pass!(pass_id, input_byte_len, "unknown pass id in execution plan")
            }
        };
        match next_output_css {
            Some(next_css) => {
                let mutation_spans = derive_transform_mutation_spans(&pass_input_css, &next_css);
                if has_remaining_lex_consumers {
                    super::lex_cache::update_cached_lex_from_splice(
                        &pass_input_css,
                        &next_css,
                        dialect,
                        mutation_spans.as_slice(),
                    );
                }
                outcome_mutation_spans.push(mutation_spans);
                output_css = next_css;
            }
            None => {
                outcome_mutation_spans.push(derive_transform_mutation_spans(
                    &pass_input_css,
                    &pass_input_css,
                ));
                output_css = pass_input_css;
            }
        }
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
    let cascade_proof_obligations = summarize_cascade_proof_obligations(cascade_proof_obligations);
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
        cascade_proof_obligations,
        provenance_derivation_forest,
        outcomes,
        pass_plan,
    }
}

fn transform_pass_may_consume_lex_cache(pass: TransformPassKind) -> bool {
    !matches!(pass, TransformPassKind::PrintCss)
}
