use omena_cascade::{
    CascadeValue, CustomPropertyLeastFixedPointSummaryV0,
    summarize_custom_property_least_fixed_point,
};
use omena_parser::StyleDialect;
use omena_scss_eval::summarize_native_css_static_edit_plan;
use omena_transform_cst::TransformIrV0;

use crate::domains::{
    calc::reduce_css_calc_with_lexer,
    cascade_flatten::{
        flatten_css_layers_with_ir_transaction_on_ir, flatten_css_scopes_with_ir_transaction_on_ir,
    },
    color::compress_css_colors_with_lexer,
    color_lowering::{
        lower_css_color_function_with_lexer, lower_css_color_mix_with_lexer,
        lower_css_light_dark_with_lexer, lower_css_oklab_oklch_with_lexer,
        lower_relative_color_with_lexer,
    },
    css_modules_classes::{
        local_css_module_composes_resolutions_with_lexer,
        reachable_class_names_with_local_composes,
        rewrite_css_module_class_names_with_ir_transaction,
        strip_resolved_css_module_composes_with_ir_transaction,
        tree_shake_css_class_rules_with_ir_transaction_on_ir,
    },
    css_modules_values::{
        resolve_static_css_modules_values_with_lexer,
        tree_shake_css_modules_values_with_ir_transaction_on_ir,
    },
    custom_property::{
        collect_static_root_custom_property_env, parse_static_custom_property_env_value,
        substitute_static_css_custom_properties_with_lexer,
        tree_shake_css_custom_properties_with_ir_transaction_on_ir,
    },
    design_token::route_design_token_values_with_ir_transaction,
    import_inline::{
        inline_css_imports_for_static_module_evaluation_with_lexer,
        inline_css_imports_with_ir_transaction, inline_css_imports_with_lexer,
        restore_less_inline_literal_placeholders as restore_less_inline_literal_placeholders_with_lexer,
    },
    keyframes::tree_shake_css_keyframes_with_ir_transaction_on_ir,
    logical::lower_css_logical_to_physical_with_lexer,
    nesting::unwrap_css_nesting_with_ir_transaction_on_ir,
    number::compress_css_numbers_with_lexer,
    reachability::class_name_is_reachable,
    rule_cleanup::{
        dedupe_exact_css_rules_with_ir_transaction_on_ir,
        remove_empty_css_rules_with_ir_transaction_on_ir,
    },
    rule_merge::{
        merge_adjacent_same_block_css_selectors_with_ir_transaction_on_ir,
        merge_adjacent_same_selector_css_rules_with_ir_transaction_on_ir,
    },
    selector::compress_css_is_where_selectors_with_lexer,
    shorthand::combine_css_shorthands_with_lexer,
    static_eval::{
        StaticMediaEvaluationOptions, evaluate_static_container_rules_with_ir_transaction_on_ir,
        evaluate_static_media_rules_with_ir_transaction_on_ir,
        evaluate_static_supports_rules_with_ir_transaction_on_ir,
    },
    text::{
        normalize_css_font_declarations_with_lexer, normalize_css_string_quotes_with_lexer,
        strip_css_url_quotes_with_lexer,
    },
    trivia::{normalize_css_whitespace_with_lexer, strip_css_comments_with_lexer},
    unit::normalize_css_units_with_lexer,
    vendor_prefix::{
        add_css_vendor_prefixes_with_lexer_and_policy, remove_stale_css_vendor_prefixes_with_lexer,
    },
};
use crate::helpers::ir_transaction::{
    TransformIrReplacementKindV0, TransformIrSourceReplacementErrorV0,
    TransformIrSourceReplacementV0, apply_ir_source_replacements_to_ir,
};
use crate::helpers::rules::collect_top_level_ordinary_rule_slices;
use crate::model::{
    TransformClassNameRewriteV0, TransformCssModuleComposesResolutionV0,
    TransformCssModuleValueResolutionV0, TransformDesignTokenRouteV0, TransformImportInlineV0,
    TransformLessInlineLiteralPlaceholderV0, TransformSemanticRemovalCandidate,
    TransformVendorPrefixPolicyV0,
};
use crate::runtime::lex_cache::lex_cached as lex;

pub(crate) fn strip_css_comments(source: &str, dialect: StyleDialect) -> (String, usize) {
    strip_css_comments_with_lexer(source, dialect)
}

pub(crate) fn compress_css_numbers(source: &str, dialect: StyleDialect) -> (String, usize) {
    compress_css_numbers_with_lexer(source, dialect)
}

pub(crate) fn compress_css_colors(source: &str, dialect: StyleDialect) -> (String, usize) {
    compress_css_colors_with_lexer(source, dialect)
}

pub(crate) fn normalize_css_units(source: &str, dialect: StyleDialect) -> (String, usize) {
    normalize_css_units_with_lexer(source, dialect)
}

pub(crate) fn strip_css_url_quotes(source: &str, dialect: StyleDialect) -> (String, usize) {
    strip_css_url_quotes_with_lexer(source, dialect)
}

pub(crate) fn normalize_css_string_quotes(source: &str, dialect: StyleDialect) -> (String, usize) {
    let (source, font_declaration_mutations) =
        normalize_css_font_declarations_with_lexer(source, dialect);
    let (source, token_mutations) = normalize_css_string_quotes_with_lexer(&source, dialect);
    (source, font_declaration_mutations + token_mutations)
}

pub(crate) fn compress_css_is_where_selectors(
    source: &str,
    dialect: StyleDialect,
) -> (String, usize) {
    compress_css_is_where_selectors_with_lexer(source, dialect)
}

pub(crate) fn remove_empty_css_rules_in_ir(
    ir: &mut TransformIrV0,
    dialect: StyleDialect,
) -> Result<(String, usize), TransformIrSourceReplacementErrorV0> {
    remove_empty_css_rules_with_ir_transaction_on_ir(ir, dialect)
}

pub(crate) fn combine_css_shorthands(source: &str, dialect: StyleDialect) -> (String, usize) {
    combine_css_shorthands_with_lexer(source, dialect)
}

pub(crate) fn dedupe_exact_css_rules_in_ir(
    ir: &mut TransformIrV0,
    dialect: StyleDialect,
) -> Result<(String, usize), TransformIrSourceReplacementErrorV0> {
    dedupe_exact_css_rules_with_ir_transaction_on_ir(ir, dialect)
}

pub(crate) fn merge_adjacent_same_selector_css_rules_in_ir(
    ir: &mut TransformIrV0,
    dialect: StyleDialect,
) -> Result<(String, usize), TransformIrSourceReplacementErrorV0> {
    merge_adjacent_same_selector_css_rules_with_ir_transaction_on_ir(ir, dialect)
}

pub(crate) fn merge_adjacent_same_block_css_selectors_in_ir(
    ir: &mut TransformIrV0,
    dialect: StyleDialect,
) -> Result<(String, usize), TransformIrSourceReplacementErrorV0> {
    merge_adjacent_same_block_css_selectors_with_ir_transaction_on_ir(ir, dialect)
}

pub(crate) fn add_css_vendor_prefixes(
    source: &str,
    dialect: StyleDialect,
    policy: TransformVendorPrefixPolicyV0,
) -> (String, usize) {
    add_css_vendor_prefixes_with_lexer_and_policy(source, dialect, policy)
}

pub(crate) fn remove_stale_css_vendor_prefixes(
    source: &str,
    dialect: StyleDialect,
) -> (String, usize) {
    remove_stale_css_vendor_prefixes_with_lexer(source, dialect)
}

pub(crate) fn lower_css_light_dark(source: &str, dialect: StyleDialect) -> (String, usize) {
    lower_css_light_dark_with_lexer(source, dialect)
}

pub(crate) fn lower_css_color_mix(source: &str, dialect: StyleDialect) -> (String, usize) {
    lower_css_color_mix_with_lexer(source, dialect)
}

pub(crate) fn lower_css_oklab_oklch(source: &str, dialect: StyleDialect) -> (String, usize) {
    lower_css_oklab_oklch_with_lexer(source, dialect)
}

pub(crate) fn lower_css_color_function(source: &str, dialect: StyleDialect) -> (String, usize) {
    lower_css_color_function_with_lexer(source, dialect)
}

pub(crate) fn lower_relative_color(source: &str, dialect: StyleDialect) -> (String, usize) {
    lower_relative_color_with_lexer(source, dialect)
}

pub(crate) fn lower_css_logical_to_physical(
    source: &str,
    dialect: StyleDialect,
) -> (String, usize) {
    lower_css_logical_to_physical_with_lexer(source, dialect)
}

pub(crate) fn unwrap_css_nesting_in_ir(
    ir: &mut TransformIrV0,
    dialect: StyleDialect,
) -> Result<(String, usize), TransformIrSourceReplacementErrorV0> {
    unwrap_css_nesting_with_ir_transaction_on_ir(ir, dialect)
}

pub(crate) fn flatten_css_scopes_in_ir(
    ir: &mut TransformIrV0,
    dialect: StyleDialect,
) -> Result<(String, usize), TransformIrSourceReplacementErrorV0> {
    flatten_css_scopes_with_ir_transaction_on_ir(ir, dialect)
}

pub(crate) fn flatten_css_layers_in_ir(
    ir: &mut TransformIrV0,
    dialect: StyleDialect,
    closed_bundle: bool,
) -> Result<(String, usize), TransformIrSourceReplacementErrorV0> {
    flatten_css_layers_with_ir_transaction_on_ir(ir, dialect, closed_bundle)
}

pub(crate) fn evaluate_static_supports_rules_in_ir(
    ir: &mut TransformIrV0,
    dialect: StyleDialect,
) -> Result<(String, usize), TransformIrSourceReplacementErrorV0> {
    evaluate_static_supports_rules_with_ir_transaction_on_ir(ir, dialect)
}

pub(crate) fn evaluate_static_media_rules_in_ir(
    ir: &mut TransformIrV0,
    dialect: StyleDialect,
) -> Result<(String, usize), TransformIrSourceReplacementErrorV0> {
    evaluate_static_media_rules_with_ir_transaction_on_ir(
        ir,
        dialect,
        StaticMediaEvaluationOptions::default(),
    )
}

pub(crate) fn evaluate_static_container_rules_in_ir(
    ir: &mut TransformIrV0,
    dialect: StyleDialect,
) -> Result<(String, usize), TransformIrSourceReplacementErrorV0> {
    evaluate_static_container_rules_with_ir_transaction_on_ir(ir, dialect)
}

pub(crate) fn evaluate_native_css_static_values_with_plan(
    source: &str,
    dialect: StyleDialect,
) -> (String, usize) {
    if dialect != StyleDialect::Css {
        return (source.to_string(), 0);
    }
    let Some(plan) = summarize_native_css_static_edit_plan(source, dialect) else {
        return (source.to_string(), 0);
    };
    let mutation_count = if plan.output_changed {
        plan.edit_count
    } else {
        0
    };
    (plan.edited_css, mutation_count)
}

pub(crate) fn evaluate_native_css_static_values(
    source: &str,
    dialect: StyleDialect,
) -> Result<(String, usize), TransformIrSourceReplacementErrorV0> {
    let mut ir = omena_transform_cst::lower_transform_ir_from_source(
        source,
        dialect,
        "omena-transform-passes.native-css-static-eval",
    );
    evaluate_native_css_static_values_in_ir(&mut ir, dialect)
}

pub(crate) fn evaluate_native_css_static_values_in_ir(
    ir: &mut TransformIrV0,
    dialect: StyleDialect,
) -> Result<(String, usize), TransformIrSourceReplacementErrorV0> {
    let source = ir.source_text().to_string();
    if dialect != StyleDialect::Css {
        return Ok((source, 0));
    }
    let Some(plan) = summarize_native_css_static_edit_plan(source.as_str(), dialect) else {
        return Ok((source, 0));
    };
    let replacements = plan
        .edits
        .into_iter()
        .map(|edit| TransformIrSourceReplacementV0 {
            source_span_start: edit.start,
            source_span_end: edit.end,
            replacement: edit.replacement,
            kind: match edit.edit_kind {
                "whenRuleBranchFold" => TransformIrReplacementKindV0::AtRule,
                _ => TransformIrReplacementKindV0::Declaration,
            },
        })
        .collect::<Vec<_>>();
    apply_ir_source_replacements_to_ir(
        ir,
        dialect,
        "native-css-static-eval",
        replacements.as_slice(),
    )
}

pub(crate) fn evaluate_dead_media_branch_rules_in_ir(
    ir: &mut TransformIrV0,
    dialect: StyleDialect,
    drop_dark_mode_media_queries: bool,
) -> Result<(String, usize), TransformIrSourceReplacementErrorV0> {
    evaluate_static_media_rules_with_ir_transaction_on_ir(
        ir,
        dialect,
        StaticMediaEvaluationOptions {
            drop_dark_mode_media_queries,
        },
    )
}

/// Applies resolved CSS `@import` replacements for the import-inline pass.
pub fn inline_css_imports(
    source: &str,
    dialect: StyleDialect,
    inlines: &[TransformImportInlineV0],
) -> (String, usize) {
    inline_css_imports_with_lexer(source, dialect, inlines)
}

pub(crate) fn inline_css_imports_with_ir_result(
    source: &str,
    dialect: StyleDialect,
    inlines: &[TransformImportInlineV0],
) -> Result<(String, usize), TransformIrSourceReplacementErrorV0> {
    inline_css_imports_with_ir_transaction(source, dialect, inlines)
}

/// Applies import inlining before static Sass/Less module evaluation.
pub fn inline_css_imports_for_static_module_evaluation(
    source: &str,
    dialect: StyleDialect,
    inlines: &[TransformImportInlineV0],
) -> (String, usize, Vec<TransformLessInlineLiteralPlaceholderV0>) {
    inline_css_imports_for_static_module_evaluation_with_lexer(source, dialect, inlines)
}

/// Restores Less `(inline)` literal import placeholders after static evaluation.
pub fn restore_less_inline_literal_placeholders(
    source: &str,
    placeholders: &[TransformLessInlineLiteralPlaceholderV0],
) -> String {
    restore_less_inline_literal_placeholders_with_lexer(source, placeholders)
}

pub(crate) fn resolve_static_css_modules_values(
    source: &str,
    dialect: StyleDialect,
    resolutions: &[TransformCssModuleValueResolutionV0],
) -> (String, usize) {
    resolve_static_css_modules_values_with_lexer(source, dialect, resolutions)
}

pub(crate) fn resolve_css_module_composes(
    source: &str,
    dialect: StyleDialect,
    resolutions: &[TransformCssModuleComposesResolutionV0],
) -> Result<(String, usize), TransformIrSourceReplacementErrorV0> {
    strip_resolved_css_module_composes_with_ir_transaction(source, dialect, resolutions)
}

pub(crate) fn css_module_composes_resolutions_for_source(
    source: &str,
    dialect: StyleDialect,
    resolutions: &[TransformCssModuleComposesResolutionV0],
) -> Vec<TransformCssModuleComposesResolutionV0> {
    let mut merged = local_css_module_composes_resolutions_with_lexer(source, dialect);
    for resolution in resolutions {
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

pub(crate) fn route_design_token_values(
    source: &str,
    dialect: StyleDialect,
    routes: &[TransformDesignTokenRouteV0],
) -> Result<(String, usize), TransformIrSourceReplacementErrorV0> {
    route_design_token_values_with_ir_transaction(source, dialect, routes)
}

pub(crate) fn tree_shake_css_class_rules_in_ir(
    ir: &mut TransformIrV0,
    dialect: StyleDialect,
    reachable_class_names: &[String],
) -> Result<(String, Vec<TransformSemanticRemovalCandidate>), TransformIrSourceReplacementErrorV0> {
    tree_shake_css_class_rules_with_ir_transaction_on_ir(ir, dialect, reachable_class_names)
}

pub(crate) fn reachable_class_names_with_composes_exports(
    source: &str,
    dialect: StyleDialect,
    reachable_class_names: &[String],
    resolutions: &[TransformCssModuleComposesResolutionV0],
) -> Vec<String> {
    let mut expanded = reachable_class_names.to_vec();
    let mut changed = true;

    while changed {
        changed = false;
        for resolution in resolutions {
            if !class_name_is_reachable(&resolution.local_class_name, &expanded) {
                continue;
            }
            for exported_class_name in &resolution.exported_class_names {
                if !class_name_is_reachable(exported_class_name, &expanded) {
                    expanded.push(exported_class_name.clone());
                    changed = true;
                }
            }
        }
    }

    expanded.sort();
    expanded.dedup();
    reachable_class_names_with_local_composes(source, dialect, &expanded)
}

pub(crate) fn tree_shake_css_keyframes_in_ir(
    ir: &mut TransformIrV0,
    dialect: StyleDialect,
    reachable_keyframe_names: &[String],
    reachable_class_names: &[String],
) -> Result<(String, Vec<TransformSemanticRemovalCandidate>), TransformIrSourceReplacementErrorV0> {
    tree_shake_css_keyframes_with_ir_transaction_on_ir(
        ir,
        dialect,
        reachable_keyframe_names,
        reachable_class_names,
    )
}

pub(crate) fn tree_shake_css_modules_values_in_ir(
    ir: &mut TransformIrV0,
    dialect: StyleDialect,
    reachable_value_names: &[String],
    reachable_keyframe_names: &[String],
    reachable_class_names: &[String],
) -> Result<(String, Vec<TransformSemanticRemovalCandidate>), TransformIrSourceReplacementErrorV0> {
    tree_shake_css_modules_values_with_ir_transaction_on_ir(
        ir,
        dialect,
        reachable_value_names,
        reachable_keyframe_names,
        reachable_class_names,
    )
}

pub(crate) fn tree_shake_css_custom_properties_in_ir(
    ir: &mut TransformIrV0,
    dialect: StyleDialect,
    reachable_custom_property_names: &[String],
    reachable_keyframe_names: &[String],
    reachable_class_names: &[String],
) -> Result<(String, Vec<TransformSemanticRemovalCandidate>), TransformIrSourceReplacementErrorV0> {
    tree_shake_css_custom_properties_with_ir_transaction_on_ir(
        ir,
        dialect,
        reachable_custom_property_names,
        reachable_keyframe_names,
        reachable_class_names,
    )
}

pub(crate) fn rewrite_css_module_class_names(
    source: &str,
    dialect: StyleDialect,
    rewrites: &[TransformClassNameRewriteV0],
) -> Result<(String, usize), TransformIrSourceReplacementErrorV0> {
    rewrite_css_module_class_names_with_ir_transaction(source, dialect, rewrites)
}

pub(crate) fn substitute_static_css_custom_properties(
    source: &str,
    dialect: StyleDialect,
) -> (String, usize) {
    substitute_static_css_custom_properties_with_lexer(source, dialect)
}

/// Summarizes the static custom-property least fixed point for a style source.
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

/// Parses a static CSS value into the cascade value model used by query consumers.
pub fn parse_static_css_cascade_value(value: &str) -> Option<CascadeValue> {
    match value.trim() {
        "initial" => Some(CascadeValue::Initial),
        "inherit" => Some(CascadeValue::Inherit),
        "unset" => Some(CascadeValue::Unset),
        value => parse_static_custom_property_env_value(value),
    }
}

pub(crate) fn reduce_css_calc(source: &str, dialect: StyleDialect) -> (String, usize) {
    reduce_css_calc_with_lexer(source, dialect)
}

pub(crate) fn normalize_css_whitespace(source: &str, dialect: StyleDialect) -> (String, usize) {
    normalize_css_whitespace_with_lexer(source, dialect)
}
