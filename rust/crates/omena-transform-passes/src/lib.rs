//! Transform pass registry and DAG planner for the post-v5 omena-css track.
//!
//! This crate consumes `omena-transform-cst` contracts. It does not duplicate
//! transform metadata; its job is to register safe mutations, cascade-proven
//! combinations, conservative lowerings, and emission boundaries as a
//! DAG-respecting execution plan for downstream transform crates.

pub use omena_cascade::CustomPropertyLeastFixedPointSummaryV0;
use omena_cascade::{
    BoxLonghandInputV0, CascadeValue, prove_box_shorthand_combination,
    summarize_custom_property_least_fixed_point,
};
use omena_parser::{StyleDialect, lex};
use omena_syntax::SyntaxKind;

mod domains;
mod helpers;
mod model;
mod runtime;

pub use domains::css_modules_values::resolve_static_css_modules_local_value_resolutions_from_source;
use domains::{
    cascade_flatten::{flatten_css_layers_with_lexer, flatten_css_scopes_with_lexer},
    color::{
        compress_hex_color_token_text, is_static_color_reference_property,
        parse_basic_named_static_color_with_alpha, parse_static_hsl_function_color_with_alpha,
        parse_static_hwb_function_color_with_alpha, parse_static_rgb_function_color_with_alpha,
        parse_static_srgb_color_with_alpha, shortest_static_srgb_color_with_alpha_text,
    },
    color_lowering::{
        lower_css_color_function_with_lexer, lower_css_color_mix_with_lexer,
        lower_css_light_dark_with_lexer, lower_css_oklab_oklch_with_lexer,
    },
    css_modules_classes::{
        rewrite_css_module_class_names_with_lexer, strip_resolved_css_module_composes_with_lexer,
        tree_shake_css_class_rules_with_lexer,
    },
    css_modules_values::{
        resolve_static_css_modules_values_with_lexer, tree_shake_css_modules_values_with_lexer,
    },
    custom_property::{
        collect_static_root_custom_property_env, parse_static_custom_property_env_value,
        substitute_static_css_custom_properties_with_lexer,
        tree_shake_css_custom_properties_with_lexer,
    },
    design_token::route_design_token_values_with_lexer,
    import_inline::inline_css_imports_with_lexer,
    keyframes::{is_keyframes_at_keyword, tree_shake_css_keyframes_with_lexer},
    logical::lower_css_logical_to_physical_with_lexer,
    nesting::unwrap_css_nesting_with_lexer,
    number::{
        compress_number_prefix, format_css_number, numeric_prefix_end, parse_reducible_calc_value,
        parse_reducible_clamp_value, parse_reducible_max_value, parse_reducible_min_value,
    },
    reachability::class_name_is_reachable,
    rule_merge::merge_adjacent_same_conditional_at_rule_blocks_with_lexer,
    selector::{compress_css_is_where_selectors_with_lexer, dedupe_selector_arguments},
    shorthand::{
        compress_background_repeat_value, compress_border_radius_value,
        compress_box_shorthand_value, compress_box_shorthand_values, compress_list_style_value,
        compressed_list_style_components, is_box_shorthand_property, is_overflow_axis_keyword,
        is_single_axis_border_radius_value,
    },
    static_eval::{
        StaticMediaEvaluationOptions, evaluate_static_media_rules_with_lexer,
        evaluate_static_supports_rules_with_lexer,
    },
    text::{
        normalize_css_font_declarations_with_lexer, normalize_css_string_quotes_with_lexer,
        strip_css_url_quotes_with_lexer,
    },
    vendor_prefix::add_css_vendor_prefixes_with_lexer,
};
use helpers::declarations::{
    SimpleDeclarationSlice, collect_simple_declarations_in_block, declaration_ranges_are_adjacent,
    format_replacement_declaration_like_source,
};
use helpers::identifiers::{is_css_ident_continue, is_css_ident_start};
use helpers::rules::{
    SimpleRuleSlice, collect_declaration_ordinary_rule_slices,
    collect_top_level_ordinary_rule_slices, first_non_trivia_token_start, is_ordinary_rule_prelude,
    rule_gap_is_whitespace_only, set_prelude_start,
};
use helpers::source_rewrite::{remove_source_ranges, replace_source_ranges, rewrite_lexer_tokens};
use helpers::tokens::{
    is_comment_token, matching_right_brace_index, next_non_comment_token_kind,
    previous_non_comment_token_kind, token_end,
};
use helpers::values::{
    matching_function_call_end, split_top_level_whitespace_value_components,
    substitute_static_css_function_references_in_value,
    substitute_static_css_function_references_in_value_until_stable,
};
use model::TransformSemanticRemovalCandidate;
pub use model::*;
pub use runtime::executor::{
    execute_transform_passes_on_source, execute_transform_passes_on_source_with_dialect,
    execute_transform_passes_on_source_with_dialect_and_context,
};
pub use runtime::fuzz::{run_transform_cascade_safe_fuzz_case, run_transform_fuzz_seed_corpus};
pub use runtime::incremental::{
    execute_transform_passes_incremental_with_database, transform_pass_incremental_graph_input,
};
pub use runtime::planner::{
    implemented_mutation_pass_ids, plan_transform_passes, summarize_omena_transform_passes_boundary,
};

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
    let (source, font_declaration_mutations) =
        normalize_css_font_declarations_with_lexer(source, dialect);
    let (source, token_mutations) = normalize_css_string_quotes_with_lexer(&source, dialect);
    (source, font_declaration_mutations + token_mutations)
}

fn compress_css_is_where_selectors(source: &str, dialect: StyleDialect) -> (String, usize) {
    compress_css_is_where_selectors_with_lexer(source, dialect)
}

fn remove_empty_css_rules(source: &str, dialect: StyleDialect) -> (String, usize) {
    remove_empty_css_rules_with_lexer(source, dialect)
}

fn combine_css_shorthands(source: &str, dialect: StyleDialect) -> (String, usize) {
    combine_css_shorthands_with_lexer(source, dialect)
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

fn resolve_static_css_modules_values(
    source: &str,
    dialect: StyleDialect,
    resolutions: &[TransformCssModuleValueResolutionV0],
) -> (String, usize) {
    resolve_static_css_modules_values_with_lexer(source, dialect, resolutions)
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

fn reachable_class_names_with_composes_exports(
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
    expanded
}

fn tree_shake_css_keyframes_with_removals(
    source: &str,
    dialect: StyleDialect,
    reachable_keyframe_names: &[String],
    reachable_class_names: &[String],
) -> (String, Vec<TransformSemanticRemovalCandidate>) {
    tree_shake_css_keyframes_with_lexer(
        source,
        dialect,
        reachable_keyframe_names,
        reachable_class_names,
    )
}

fn tree_shake_css_modules_values_with_removals(
    source: &str,
    dialect: StyleDialect,
    reachable_value_names: &[String],
    reachable_class_names: &[String],
) -> (String, Vec<TransformSemanticRemovalCandidate>) {
    tree_shake_css_modules_values_with_lexer(
        source,
        dialect,
        reachable_value_names,
        reachable_class_names,
    )
}

fn tree_shake_css_custom_properties_with_removals(
    source: &str,
    dialect: StyleDialect,
    reachable_custom_property_names: &[String],
    reachable_keyframe_names: &[String],
    reachable_class_names: &[String],
) -> (String, Vec<TransformSemanticRemovalCandidate>) {
    tree_shake_css_custom_properties_with_lexer(
        source,
        dialect,
        reachable_custom_property_names,
        reachable_keyframe_names,
        reachable_class_names,
    )
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

pub fn parse_static_css_cascade_value(value: &str) -> Option<CascadeValue> {
    match value.trim() {
        "initial" => Some(CascadeValue::Initial),
        "inherit" => Some(CascadeValue::Inherit),
        "unset" => Some(CascadeValue::Unset),
        value => parse_static_custom_property_env_value(value),
    }
}

fn reduce_css_calc(source: &str, dialect: StyleDialect) -> (String, usize) {
    reduce_css_calc_with_lexer(source, dialect)
}

fn normalize_css_whitespace(source: &str, dialect: StyleDialect) -> (String, usize) {
    normalize_css_whitespace_with_lexer(source, dialect)
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
                let Some(replacement_value) =
                    substitute_static_css_function_references_in_value_until_stable(
                        &declaration.value,
                        &[
                            ("calc", parse_reducible_calc_value),
                            ("min", parse_reducible_min_value),
                            ("max", parse_reducible_max_value),
                            ("clamp", parse_reducible_clamp_value),
                        ],
                    )
                else {
                    continue;
                };
                replacements.push((
                    declaration.start,
                    declaration.end,
                    format_replacement_declaration_like_source(
                        source,
                        &declaration,
                        &replacement_value,
                    ),
                ));
            }
            index += 1;
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
    let (output, ordinary_mutation_count) =
        merge_adjacent_same_selector_ordinary_css_rules_with_lexer(source, dialect);
    let (output, at_rule_mutation_count) =
        merge_adjacent_same_conditional_at_rule_blocks_with_lexer(&output, dialect);
    (output, ordinary_mutation_count + at_rule_mutation_count)
}

fn merge_adjacent_same_selector_ordinary_css_rules_with_lexer(
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
                format!(
                    "{} {{ {} }}",
                    current.selector,
                    join_rule_blocks_for_merge(&blocks)
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

fn join_rule_blocks_for_merge(blocks: &[String]) -> String {
    blocks
        .iter()
        .filter_map(|block| {
            let trimmed = block.trim();
            if trimmed.is_empty() {
                None
            } else if trimmed.ends_with(';') {
                Some(trimmed.to_string())
            } else {
                Some(format!("{trimmed};"))
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
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

fn combine_css_shorthands_with_lexer(source: &str, dialect: StyleDialect) -> (String, usize) {
    let lexed = lex(source, dialect);
    let tokens = lexed.tokens();
    let mut ranges = collect_shorthand_replacement_ranges(source, tokens);
    if ranges.is_empty() {
        return (source.to_string(), 0);
    }
    ranges.sort_by_key(|(start, _, _)| *start);

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

fn collect_shorthand_replacement_ranges(
    source: &str,
    tokens: &[omena_parser::LexedToken],
) -> Vec<(usize, usize, String)> {
    let mut ranges = Vec::new();
    let mut index = 0;
    while index < tokens.len() {
        if tokens[index].kind == SyntaxKind::LeftBrace
            && let Some(close_index) = matching_right_brace_index(tokens, index)
        {
            ranges.extend(collect_shorthand_replacements_in_block(
                source,
                tokens,
                index,
                close_index,
            ));
            index += 1;
            continue;
        }
        index += 1;
    }
    ranges
}

fn collect_shorthand_replacements_in_block(
    source: &str,
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
                .or_else(|| {
                    border_radius_shorthand_replacement_for_declarations(
                        tokens,
                        &declarations[index..index + 4],
                    )
                })
                .or_else(|| {
                    inset_shorthand_replacement_for_declarations(
                        tokens,
                        &declarations[index..index + 4],
                    )
                })
        {
            ranges.push((start, end, replacement));
            index += 4;
        } else {
            index += 1;
        }
    }
    let mut index = 0;
    while index + 2 < declarations.len() {
        if let Some((start, end, replacement)) = list_style_shorthand_replacement_for_declarations(
            tokens,
            &declarations[index..index + 3],
        ) {
            ranges.push((start, end, replacement));
            index += 3;
        } else {
            index += 1;
        }
    }
    for declaration in &declarations {
        if let Some((start, end, replacement)) =
            shorthand_value_replacement_for_declaration(source, declaration)
        {
            ranges.push((start, end, replacement));
        }
    }
    ranges.extend(collect_overflow_axis_replacements(tokens, &declarations));
    ranges
}

fn box_shorthand_replacement_for_declarations(
    tokens: &[omena_parser::LexedToken],
    declarations: &[SimpleDeclarationSlice],
) -> Option<(usize, usize, String)> {
    let shorthand_property = match declarations.first()?.property.as_str() {
        "margin-top" => "margin",
        "padding-top" => "padding",
        "border-top-color" => "border-color",
        "border-top-style" => "border-style",
        "border-top-width" => "border-width",
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

fn shorthand_value_replacement_for_declaration(
    source: &str,
    declaration: &SimpleDeclarationSlice,
) -> Option<(usize, usize, String)> {
    if declaration.important {
        return None;
    }
    let replacement_value = if is_box_shorthand_property(&declaration.property) {
        compress_box_shorthand_value(&declaration.value)
    } else if declaration.property == "background-repeat" {
        compress_background_repeat_value(&declaration.value)
    } else if declaration.property == "border-radius" {
        compress_border_radius_value(&declaration.value)
    } else if declaration.property == "inset" {
        compress_box_shorthand_value(&declaration.value)
    } else if declaration.property == "list-style" {
        compress_list_style_value(&declaration.value)
    } else {
        None
    }?;
    let replacement =
        format_replacement_declaration_like_source(source, declaration, &replacement_value);
    Some((declaration.start, declaration.end, replacement))
}

fn border_radius_shorthand_replacement_for_declarations(
    tokens: &[omena_parser::LexedToken],
    declarations: &[SimpleDeclarationSlice],
) -> Option<(usize, usize, String)> {
    let [top_left, top_right, bottom_right, bottom_left] = declarations else {
        return None;
    };
    if top_left.property != "border-top-left-radius"
        || top_right.property != "border-top-right-radius"
        || bottom_right.property != "border-bottom-right-radius"
        || bottom_left.property != "border-bottom-left-radius"
        || declarations.iter().any(|declaration| declaration.important)
        || !declaration_ranges_are_adjacent(tokens, declarations)
        || declarations
            .iter()
            .any(|declaration| !is_single_axis_border_radius_value(&declaration.value))
    {
        return None;
    }
    let values = declarations
        .iter()
        .map(|declaration| declaration.value.as_str())
        .collect::<Vec<_>>();
    let shorthand_value = compress_box_shorthand_values(&values)?;
    Some((
        top_left.start,
        bottom_left.end,
        format!("border-radius: {shorthand_value};"),
    ))
}

fn inset_shorthand_replacement_for_declarations(
    tokens: &[omena_parser::LexedToken],
    declarations: &[SimpleDeclarationSlice],
) -> Option<(usize, usize, String)> {
    let [top, right, bottom, left] = declarations else {
        return None;
    };
    if top.property != "top"
        || right.property != "right"
        || bottom.property != "bottom"
        || left.property != "left"
        || declarations.iter().any(|declaration| declaration.important)
        || !declaration_ranges_are_adjacent(tokens, declarations)
    {
        return None;
    }
    let values = declarations
        .iter()
        .map(|declaration| declaration.value.as_str())
        .collect::<Vec<_>>();
    let shorthand_value = compress_box_shorthand_values(&values)?;
    Some((top.start, left.end, format!("inset: {shorthand_value};")))
}

fn list_style_shorthand_replacement_for_declarations(
    tokens: &[omena_parser::LexedToken],
    declarations: &[SimpleDeclarationSlice],
) -> Option<(usize, usize, String)> {
    let [style_type, position, image] = declarations else {
        return None;
    };
    if style_type.property != "list-style-type"
        || position.property != "list-style-position"
        || image.property != "list-style-image"
        || declarations.iter().any(|declaration| declaration.important)
        || !declaration_ranges_are_adjacent(tokens, declarations)
    {
        return None;
    }
    let shorthand_value =
        compressed_list_style_components(&style_type.value, &position.value, &image.value)?;
    Some((
        style_type.start,
        image.end,
        format!("list-style: {shorthand_value};"),
    ))
}

fn collect_overflow_axis_replacements(
    tokens: &[omena_parser::LexedToken],
    declarations: &[SimpleDeclarationSlice],
) -> Vec<(usize, usize, String)> {
    let mut ranges = Vec::new();
    for pair in declarations.windows(2) {
        let [x, y] = pair else {
            continue;
        };
        if x.property != "overflow-x"
            || y.property != "overflow-y"
            || x.important
            || y.important
            || x.value != y.value
            || !is_overflow_axis_keyword(&x.value)
            || !declaration_ranges_are_adjacent(tokens, pair)
        {
            continue;
        }
        ranges.push((x.start, y.end, format!("overflow: {};", x.value)));
    }
    ranges
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
    let mut keyframes_contexts = vec![false];
    let mut index = 0;

    while index < tokens.len() {
        match tokens[index].kind {
            SyntaxKind::LeftBrace => {
                let prelude_start = prelude_starts.get(depth).copied().unwrap_or(0);
                let inside_keyframes = keyframes_contexts.get(depth).copied().unwrap_or(false);
                if let Some(close_index) = matching_right_brace_index(tokens, index)
                    && is_empty_rule_block(tokens, index + 1, close_index)
                    && ((!inside_keyframes
                        && is_ordinary_rule_prelude(tokens, prelude_start, index))
                        || is_empty_group_rule_prelude(tokens, prelude_start, index))
                    && let Some(start) = first_non_trivia_token_start(tokens, prelude_start, index)
                {
                    let end = token_end(&tokens[close_index]);
                    ranges.push((start, end));
                    index = close_index + 1;
                    set_prelude_start(&mut prelude_starts, depth, index);
                    continue;
                }
                let child_inside_keyframes = inside_keyframes
                    || is_keyframes_group_rule_prelude(tokens, prelude_start, index);
                depth += 1;
                set_prelude_start(&mut prelude_starts, depth, index + 1);
                set_bool_context(&mut keyframes_contexts, depth, child_inside_keyframes);
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

fn set_bool_context(contexts: &mut Vec<bool>, depth: usize, value: bool) {
    if contexts.len() <= depth {
        contexts.resize(depth + 1, false);
    }
    contexts[depth] = value;
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

fn is_keyframes_group_rule_prelude(
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
    first.kind == SyntaxKind::AtKeyword && is_keyframes_at_keyword(&first.text)
}

fn is_empty_removable_group_at_keyword(text: &str) -> bool {
    matches!(
        text.to_ascii_lowercase().as_str(),
        "@container" | "@layer" | "@media" | "@scope" | "@supports"
    )
}

fn compress_css_colors_with_lexer(source: &str, dialect: StyleDialect) -> (String, usize) {
    let (source, hex_mutation_count) = compress_css_hex_color_tokens_with_lexer(source, dialect);
    let (source, function_mutation_count) =
        compress_static_color_function_declaration_values_with_lexer(&source, dialect);
    let (source, duplicate_mutation_count) =
        remove_adjacent_duplicate_static_color_declarations_with_lexer(&source, dialect);

    (
        source,
        hex_mutation_count + function_mutation_count + duplicate_mutation_count,
    )
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
                let Some(replacement_value) = compress_static_color_references_in_declaration_value(
                    &declaration.property,
                    &declaration.value,
                ) else {
                    continue;
                };
                replacements.push((
                    declaration.start,
                    declaration.end,
                    format_replacement_declaration_like_source(
                        source,
                        &declaration,
                        &replacement_value,
                    ),
                ));
            }
            index += 1;
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

fn remove_adjacent_duplicate_static_color_declarations_with_lexer(
    source: &str,
    dialect: StyleDialect,
) -> (String, usize) {
    let lexed = lex(source, dialect);
    let tokens = lexed.tokens();
    let mut ranges = Vec::new();
    let mut index = 0;

    while index < tokens.len() {
        if tokens[index].kind == SyntaxKind::LeftBrace
            && let Some(close_index) = matching_right_brace_index(tokens, index)
        {
            let declarations = collect_simple_declarations_in_block(tokens, index, close_index);
            for pair in declarations.windows(2) {
                let [left, right] = pair else {
                    continue;
                };
                if !declaration_ranges_are_adjacent(tokens, pair)
                    || left.important
                    || right.important
                    || left.property != right.property
                    || left.value != right.value
                    || !is_static_color_reference_property(&left.property)
                {
                    continue;
                }
                ranges.push((right.start, right.end));
            }
            index = close_index + 1;
            continue;
        }
        index += 1;
    }

    let (output, removed_count) = remove_source_ranges(source, &ranges);
    (output, removed_count)
}

fn compress_static_color_value(value: &str) -> Option<String> {
    let trimmed = value.trim();
    let color = parse_static_srgb_color_with_alpha(value)
        .or_else(|| parse_static_rgb_function_color_with_alpha(value))
        .or_else(|| parse_static_hsl_function_color_with_alpha(value))
        .or_else(|| parse_static_hwb_function_color_with_alpha(value))?;
    let replacement = shortest_static_srgb_color_with_alpha_text(color);
    (replacement.len() < trimmed.len()
        || (replacement.len() == trimmed.len() && replacement != trimmed))
        .then_some(replacement)
}

fn compress_static_color_references_in_value(value: &str) -> Option<String> {
    substitute_static_css_function_references_in_value(
        value,
        &[
            ("rgb", compress_static_color_value),
            ("rgba", compress_static_color_value),
            ("hsl", compress_static_color_value),
            ("hsla", compress_static_color_value),
            ("hwb", compress_static_color_value),
        ],
    )
    .or_else(|| compress_static_color_value(value))
}

fn compress_static_color_references_in_declaration_value(
    property: &str,
    value: &str,
) -> Option<String> {
    if !is_static_color_reference_property(property) {
        return None;
    }

    let mut current = value.to_string();
    let mut changed = false;

    if let Some(replacement) = compress_static_color_references_in_value(&current) {
        current = replacement;
        changed = true;
    }
    if let Some(replacement) = compress_static_named_srgb_color_references_in_value(&current) {
        current = replacement;
        changed = true;
    }

    changed.then_some(current)
}

fn compress_static_named_srgb_color_references_in_value(value: &str) -> Option<String> {
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
                if let Some(escaped) = value[index..].chars().next() {
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
                index += ch.len_utf8();
            }
            _ if is_css_ident_start(ch) => {
                let start = index;
                index += ch.len_utf8();
                while let Some(next_ch) = value[index..].chars().next() {
                    if !is_css_ident_continue(next_ch) {
                        break;
                    }
                    index += next_ch.len_utf8();
                }
                let ident = &value[start..index];
                if ident.eq_ignore_ascii_case("url")
                    && value[index..].trim_start().starts_with('(')
                    && let Some(open_offset) = value[index..].find('(')
                    && let Some(close_index) =
                        matching_function_call_end(value, index + open_offset)
                {
                    index = close_index + ')'.len_utf8();
                    continue;
                }
                let Some(color) = parse_basic_named_static_color_with_alpha(ident) else {
                    continue;
                };
                let replacement = shortest_static_srgb_color_with_alpha_text(color);
                if replacement == ident {
                    continue;
                }
                output.push_str(&value[cursor..start]);
                output.push_str(&replacement);
                cursor = index;
                changed = true;
            }
            _ => index += ch.len_utf8(),
        }
    }

    if !changed {
        return None;
    }
    output.push_str(&value[cursor..]);
    Some(output)
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

        let replacement = match token.kind {
            SyntaxKind::Dimension => active_property
                .as_deref()
                .and_then(|property| normalize_dimension_unit_token(&token.text, property)),
            SyntaxKind::Percentage => active_property
                .as_deref()
                .and_then(|property| normalize_percentage_unit_token(&token.text, property)),
            _ => None,
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

    let (output, declaration_value_mutation_count) =
        normalize_static_unit_declaration_values_with_lexer(&output, dialect);
    (output, mutation_count + declaration_value_mutation_count)
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
            | "line-height"
    )
}

fn normalize_dimension_unit_token(text: &str, property: &str) -> Option<String> {
    if property.starts_with("--") {
        return None;
    }

    let split = numeric_prefix_end(text)?;
    let (number, unit) = text.split_at(split);
    if let Some(replacement) = normalize_css_time_unit_token(number, unit) {
        return (replacement != text).then_some(replacement);
    }
    if is_zero_length_unit_property(property)
        && is_zero_number_prefix(number)
        && is_css_length_unit(unit)
    {
        return Some("0".to_string());
    }

    normalize_known_css_unit_case(number, unit)
}

fn normalize_percentage_unit_token(text: &str, property: &str) -> Option<String> {
    if property.starts_with("--") {
        return None;
    }

    let number = text.strip_suffix('%')?;
    if !is_zero_number_prefix(number) {
        return None;
    }
    if is_zero_percentage_unit_property(property) || property == "opacity" {
        Some("0".to_string())
    } else {
        None
    }
}

fn is_zero_percentage_unit_property(property: &str) -> bool {
    matches!(
        property,
        "background-position"
            | "mask-position"
            | "-webkit-mask-position"
            | "perspective-origin"
            | "transform-origin"
    )
}

fn normalize_static_unit_declaration_values_with_lexer(
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
            for declaration in collect_simple_declarations_in_block(tokens, index, close_index) {
                let Some(replacement_value) = normalize_static_unit_declaration_value(
                    &declaration.property,
                    &declaration.value,
                ) else {
                    continue;
                };
                replacements.push((
                    declaration.start,
                    declaration.end,
                    format_replacement_declaration_like_source(
                        source,
                        &declaration,
                        &replacement_value,
                    ),
                ));
            }
            index += 1;
            continue;
        }
        index += 1;
    }

    replace_source_ranges(source, &replacements)
}

fn normalize_static_unit_declaration_value(property: &str, value: &str) -> Option<String> {
    match property {
        "background-size" | "mask-size" | "-webkit-mask-size" => {
            normalize_repeated_pair_value(value, "auto")
        }
        _ => None,
    }
}

fn normalize_repeated_pair_value(value: &str, repeated: &str) -> Option<String> {
    let components = split_top_level_whitespace_value_components(value)?;
    match components.as_slice() {
        [first, second]
            if first.eq_ignore_ascii_case(repeated) && second.eq_ignore_ascii_case(repeated) =>
        {
            Some(repeated.to_string())
        }
        _ => None,
    }
}

fn normalize_css_time_unit_token(number: &str, unit: &str) -> Option<String> {
    let normalized_unit = unit.to_ascii_lowercase();
    if !matches!(normalized_unit.as_str(), "ms" | "s") {
        return None;
    }

    let value = number.parse::<f64>().ok()?;
    if !value.is_finite() {
        return None;
    }
    if value == 0.0 {
        return Some("0s".to_string());
    }

    let seconds = if normalized_unit == "ms" {
        value / 1000.0
    } else {
        value
    };
    let seconds_text = format!("{}s", format_css_time_number(seconds));
    let milliseconds_text = format!("{}ms", format_css_time_number(seconds * 1000.0));

    if seconds_text.len() < milliseconds_text.len() {
        Some(seconds_text)
    } else {
        Some(milliseconds_text)
    }
}

fn format_css_time_number(value: f64) -> String {
    compress_number_prefix(&format_css_number(value))
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

fn compress_numeric_token_text(text: &str) -> Option<String> {
    let split = numeric_prefix_end(text)?;
    let (number, suffix) = text.split_at(split);
    let compressed = compress_number_prefix(number);
    let rewritten = format!("{compressed}{suffix}");
    (rewritten != text).then_some(rewritten)
}

fn normalize_css_whitespace_with_lexer(source: &str, dialect: StyleDialect) -> (String, usize) {
    let lexed = lex(source, dialect);
    let tokens = lexed.tokens();
    let mut output = String::with_capacity(source.len());
    let mut mutation_count = 0;

    for (index, token) in tokens.iter().enumerate() {
        if token.kind == SyntaxKind::Semicolon
            && matches!(
                next_non_comment_token_kind(tokens, index),
                Some(SyntaxKind::RightBrace)
            )
        {
            mutation_count += 1;
            continue;
        }

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

fn can_remove_whitespace_after(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::LeftBrace
            | SyntaxKind::RightBrace
            | SyntaxKind::LeftParen
            | SyntaxKind::LeftBracket
            | SyntaxKind::Comma
            | SyntaxKind::Colon
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
            | SyntaxKind::Colon
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

#[cfg(test)]
mod tests;
