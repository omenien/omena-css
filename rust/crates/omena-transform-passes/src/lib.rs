//! Transform pass registry and DAG planner for the post-v5 omena-css track.
//!
//! This crate consumes `omena-transform-cst` contracts. It does not duplicate
//! transform metadata; its job is to register safe mutations, cascade-proven
//! combinations, conservative lowerings, and emission boundaries as a
//! DAG-respecting execution plan for downstream transform crates.

pub use omena_cascade::CustomPropertyLeastFixedPointSummaryV0;
use omena_cascade::{
    BoxLonghandInputV0, CascadeValue, LayerFlattenInputV0, ScopeFlattenInputV0,
    prove_box_shorthand_combination, prove_layer_flatten_candidate, prove_scope_flatten_candidate,
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
    color::{
        compress_hex_color_token_text, parse_basic_named_static_color_with_alpha,
        parse_color_function_value, parse_color_mix_value, parse_oklab_oklch_value,
        parse_static_hsl_function_color_with_alpha, parse_static_hwb_function_color_with_alpha,
        parse_static_rgb_function_color_with_alpha, parse_static_srgb_color_with_alpha,
        shortest_static_srgb_color_with_alpha_text,
    },
    css_module_global::{
        CssModuleScopeBlockKind, collect_css_module_scope_blocks, css_module_scope_kind_for_range,
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
    number::{
        compress_number_prefix, format_css_number, numeric_prefix_end, parse_reducible_calc_value,
        parse_reducible_clamp_value, parse_reducible_max_value, parse_reducible_min_value,
    },
    reachability::{
        class_name_is_reachable, normalize_reachable_class_name,
        selector_list_class_tree_shake_plan,
    },
    rule_merge::merge_adjacent_same_conditional_at_rule_blocks_with_lexer,
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
};
use helpers::ascii::{ascii_css_identifier_end, starts_with_ascii_case_insensitive};
use helpers::blocks::{at_rule_block_indexes, at_rule_block_start, rule_block_token_indexes};
use helpers::declarations::{
    SimpleDeclarationSlice, collect_simple_declarations_in_block, declaration_ranges_are_adjacent,
    format_replacement_declaration_like_source,
};
use helpers::identifiers::{
    css_identifier_text_is_plain, is_css_ident_continue, is_css_ident_start,
};
use helpers::rules::{
    SimpleRuleSlice, collect_declaration_ordinary_rule_slices,
    collect_ordinary_rule_selector_slices, collect_top_level_ordinary_rule_slices,
    first_non_trivia_token_start, is_ordinary_rule_prelude, is_ordinary_top_level_rule_prelude,
    rule_gap_is_whitespace_only, set_prelude_start,
};
use helpers::selectors::{
    global_pseudo_function_end, local_pseudo_function_end, simple_class_selector_names,
    split_css_selector_list,
};
use helpers::source_rewrite::{remove_source_ranges, replace_source_ranges, rewrite_lexer_tokens};
use helpers::tokens::{
    is_comment_token, matching_right_brace_index, matching_right_paren_index,
    next_non_comment_token_kind, previous_non_comment_token_kind, skip_whitespace_tokens,
    token_end, token_start,
};
use helpers::values::{
    matching_function_call_end, matching_function_end, parse_whole_function_value_arguments,
    split_top_level_value_arguments, split_top_level_whitespace_value_components,
    static_css_string_value,
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
            if !is_static_color_reference_property(&declaration.property) {
                continue;
            }
            let Some((light_value, dark_value)) =
                substitute_light_dark_references_in_value(&declaration.value)
            else {
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
                if !is_static_color_reference_property(&declaration.property) {
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
                if !is_static_color_reference_property(&declaration.property) {
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
                if !is_static_color_reference_property(&declaration.property) {
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
                    format_replacement_declaration_like_source(
                        source,
                        &declaration,
                        &replacement_value,
                    ),
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

fn tree_shake_css_class_rules_with_lexer(
    source: &str,
    dialect: StyleDialect,
    reachable_class_names: &[String],
) -> (String, Vec<TransformSemanticRemovalCandidate>) {
    let lexed = lex(source, dialect);
    let tokens = lexed.tokens();
    let rules = collect_declaration_ordinary_rule_slices(source, tokens);
    let scope_blocks = collect_css_module_scope_blocks(source, tokens);
    let mut removals = Vec::new();
    let mut replacements = Vec::new();

    for rule in &rules {
        if css_module_scope_kind_for_range(rule.start, rule.end, &scope_blocks)
            == Some(CssModuleScopeBlockKind::Global)
        {
            continue;
        }
        let Some(plan) = selector_list_class_tree_shake_plan(&rule.selector, reachable_class_names)
        else {
            continue;
        };
        removals.push(TransformSemanticRemovalCandidate {
            symbol_kind: "class",
            name: plan.unreachable_owner_class_names.join(","),
            source_span_start: rule.start,
            source_span_end: rule.end,
            reason: "selector owner classes were absent from the closed-style-world reachable class set",
        });
        if let Some(reachable_selector) = plan.reachable_selector {
            replacements.push((
                rule.start,
                rule.block_start,
                format!("{reachable_selector} "),
            ));
        } else {
            replacements.push((rule.start, rule.end, String::new()));
        }
    }

    let (output, _) = replace_source_ranges(source, &replacements);
    (output, removals)
}

fn strip_resolved_css_module_composes_with_lexer(
    source: &str,
    dialect: StyleDialect,
    resolutions: &[TransformCssModuleComposesResolutionV0],
) -> (String, usize) {
    let lexed = lex(source, dialect);
    let tokens = lexed.tokens();
    let rules = collect_declaration_ordinary_rule_slices(source, tokens);
    let scope_blocks = collect_css_module_scope_blocks(source, tokens);
    let mut ranges = Vec::new();

    for rule in &rules {
        if css_module_scope_kind_for_range(rule.start, rule.end, &scope_blocks)
            == Some(CssModuleScopeBlockKind::Global)
        {
            continue;
        }
        let Some(class_names) = simple_class_selector_names(&rule.selector) else {
            continue;
        };
        if !class_names
            .iter()
            .all(|class_name| css_module_composes_resolution_exists(class_name, resolutions))
        {
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

fn rewrite_css_module_class_names_with_lexer(
    source: &str,
    dialect: StyleDialect,
    rewrites: &[TransformClassNameRewriteV0],
) -> (String, usize) {
    let lexed = lex(source, dialect);
    let tokens = lexed.tokens();
    let rules = collect_declaration_ordinary_rule_slices(source, tokens);
    let scope_blocks = collect_css_module_scope_blocks(source, tokens);
    let mut replacements = Vec::new();

    for block in &scope_blocks {
        replacements.push((block.start, block.body_start, String::new()));
        replacements.push((block.body_end, block.end, String::new()));
    }

    for rule in &rules {
        if css_module_scope_kind_for_range(rule.start, rule.end, &scope_blocks)
            == Some(CssModuleScopeBlockKind::Global)
        {
            continue;
        }
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
            if css_module_scope_kind_for_range(
                token_start(&tokens[index]),
                token_end(&tokens[close_index]),
                &scope_blocks,
            ) == Some(CssModuleScopeBlockKind::Global)
            {
                index = close_index + 1;
                continue;
            }
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

        if bracket_depth == 0
            && let Some(global_end) = global_pseudo_function_end(selector, index)
        {
            let inner_start = index + ":global(".len();
            let inner_end = global_end.saturating_sub(1);
            output.push_str(&selector[inner_start..inner_end]);
            index = global_end;
            changed = true;
            continue;
        }
        if bracket_depth == 0
            && let Some(local_end) = local_pseudo_function_end(selector, index)
        {
            let inner_start = index + ":local(".len();
            let inner_end = local_end.saturating_sub(1);
            let inner = &selector[inner_start..inner_end];
            if let Some(rewritten_inner) = rewrite_class_selectors_in_selector(inner, rewrites) {
                output.push_str(&rewritten_inner);
            } else {
                output.push_str(inner);
            }
            index = local_end;
            changed = true;
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
        if let Some(global_name) = parse_global_composes_part(part) {
            changed = true;
            parts.push(global_name.to_string());
            continue;
        }
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

fn parse_global_composes_part(part: &str) -> Option<&str> {
    const GLOBAL_PREFIX: &str = "global(";
    if !starts_with_ascii_case_insensitive(part, GLOBAL_PREFIX) {
        return None;
    }
    let end = matching_function_end(part, GLOBAL_PREFIX.len() - 1)?;
    if end != part.len() {
        return None;
    }
    let inner = part[GLOBAL_PREFIX.len()..end.saturating_sub(1)].trim();
    let class_name = normalize_reachable_class_name(inner)?;
    css_identifier_text_is_plain(class_name).then_some(class_name)
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

fn substitute_light_dark_references_in_value(value: &str) -> Option<(String, String)> {
    let mut light_output = String::with_capacity(value.len());
    let mut dark_output = String::with_capacity(value.len());
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
                .get(.."light-dark(".len())
                .is_some_and(|text| text.eq_ignore_ascii_case("light-dark(")) =>
            {
                let left_paren_index = index + "light-dark".len();
                let Some(close_index) = matching_function_call_end(value, left_paren_index) else {
                    index += ch.len_utf8();
                    continue;
                };
                let function_value = &value[index..close_index + ')'.len_utf8()];
                let Some((light_value, dark_value)) = parse_light_dark_value(function_value) else {
                    index += ch.len_utf8();
                    continue;
                };
                light_output.push_str(&value[cursor..index]);
                dark_output.push_str(&value[cursor..index]);
                light_output.push_str(&light_value);
                dark_output.push_str(&dark_value);
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
    light_output.push_str(&value[cursor..]);
    dark_output.push_str(&value[cursor..]);
    Some((light_output, dark_output))
}

type StaticCssFunctionParser = fn(&str) -> Option<String>;
type StaticCssFunctionSpec<'a> = (&'a str, StaticCssFunctionParser);

fn substitute_static_css_function_references_in_value(
    value: &str,
    functions: &[StaticCssFunctionSpec<'_>],
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

fn substitute_static_css_function_references_in_value_until_stable(
    value: &str,
    functions: &[StaticCssFunctionSpec<'_>],
) -> Option<String> {
    let mut current = value.to_string();
    let mut changed = false;

    for _ in 0..8 {
        let Some(next) = substitute_static_css_function_references_in_value(&current, functions)
        else {
            break;
        };
        if next == current {
            break;
        }
        current = next;
        changed = true;
    }

    changed.then_some(current)
}

fn static_css_function_at<'a>(
    value: &str,
    index: usize,
    functions: &'a [StaticCssFunctionSpec<'a>],
) -> Option<StaticCssFunctionSpec<'a>> {
    functions.iter().find_map(|(function_name, parser)| {
        let name = value.get(index..index + function_name.len())?;
        let open_paren = value[index + function_name.len()..].chars().next()?;
        (name.eq_ignore_ascii_case(function_name) && open_paren == '(')
            .then_some((*function_name, *parser))
    })
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
        "inset-block-start" => Some("top"),
        "inset-block-end" => Some("bottom"),
        "inset-inline-start" => Some(inline_start_property(direction, "left", "right")),
        "inset-inline-end" => Some(inline_end_property(direction, "left", "right")),
        "margin-block-start" => Some("margin-top"),
        "margin-block-end" => Some("margin-bottom"),
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
        "padding-block-start" => Some("padding-top"),
        "padding-block-end" => Some("padding-bottom"),
        "border-block-start-color" => Some("border-top-color"),
        "border-block-end-color" => Some("border-bottom-color"),
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
        "border-block-start-style" => Some("border-top-style"),
        "border-block-end-style" => Some("border-bottom-style"),
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
        "border-block-start-width" => Some("border-top-width"),
        "border-block-end-width" => Some("border-bottom-width"),
        "border-block-start" => Some("border-top"),
        "border-block-end" => Some("border-bottom"),
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
        "border-start-start-radius" => Some(inline_start_property(
            direction,
            "border-top-left-radius",
            "border-top-right-radius",
        )),
        "border-start-end-radius" => Some(inline_end_property(
            direction,
            "border-top-left-radius",
            "border-top-right-radius",
        )),
        "border-end-start-radius" => Some(inline_start_property(
            direction,
            "border-bottom-left-radius",
            "border-bottom-right-radius",
        )),
        "border-end-end-radius" => Some(inline_end_property(
            direction,
            "border-bottom-left-radius",
            "border-bottom-right-radius",
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
        "inset-block" => Some(("top", "bottom")),
        "inset-inline" => Some(inline_start_end_properties(direction, "left", "right")),
        "margin-block" => Some(("margin-top", "margin-bottom")),
        "margin-inline" => Some(inline_start_end_properties(
            direction,
            "margin-left",
            "margin-right",
        )),
        "padding-block" => Some(("padding-top", "padding-bottom")),
        "padding-inline" => Some(inline_start_end_properties(
            direction,
            "padding-left",
            "padding-right",
        )),
        "scroll-margin-block" => Some(("scroll-margin-top", "scroll-margin-bottom")),
        "scroll-margin-inline" => Some(inline_start_end_properties(
            direction,
            "scroll-margin-left",
            "scroll-margin-right",
        )),
        "scroll-padding-block" => Some(("scroll-padding-top", "scroll-padding-bottom")),
        "scroll-padding-inline" => Some(inline_start_end_properties(
            direction,
            "scroll-padding-left",
            "scroll-padding-right",
        )),
        "border-block-color" => Some(("border-top-color", "border-bottom-color")),
        "border-inline-color" => Some(inline_start_end_properties(
            direction,
            "border-left-color",
            "border-right-color",
        )),
        "border-block-style" => Some(("border-top-style", "border-bottom-style")),
        "border-inline-style" => Some(inline_start_end_properties(
            direction,
            "border-left-style",
            "border-right-style",
        )),
        "border-block-width" => Some(("border-top-width", "border-bottom-width")),
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
        "border-block" => Some(("border-top", "border-bottom")),
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

fn compress_css_is_where_selectors_with_lexer(
    source: &str,
    dialect: StyleDialect,
) -> (String, usize) {
    let (source, function_mutation_count) =
        compress_css_is_where_functions_with_lexer(source, dialect);
    let (source, list_expansion_mutation_count) =
        expand_specificity_safe_is_selector_lists_with_lexer(&source, dialect);
    let (source, selector_list_mutation_count) =
        dedupe_ordinary_selector_lists_with_lexer(&source, dialect);
    let (source, keyframe_selector_mutation_count) =
        normalize_keyframe_selector_aliases_with_lexer(&source, dialect);

    (
        source,
        function_mutation_count
            + list_expansion_mutation_count
            + selector_list_mutation_count
            + keyframe_selector_mutation_count,
    )
}

fn normalize_keyframe_selector_aliases_with_lexer(
    source: &str,
    dialect: StyleDialect,
) -> (String, usize) {
    let lexed = lex(source, dialect);
    let tokens = lexed.tokens();
    let mut replacements = Vec::new();
    let mut index = 0;

    while index < tokens.len() {
        if tokens[index].kind == SyntaxKind::AtKeyword
            && is_keyframes_at_keyword(&tokens[index].text)
            && let Some((block_start_index, block_end_index)) = at_rule_block_indexes(tokens, index)
        {
            collect_keyframe_selector_alias_replacements(
                source,
                tokens,
                block_start_index,
                block_end_index,
                &mut replacements,
            );
            index = block_end_index + 1;
            continue;
        }
        index += 1;
    }

    if replacements.is_empty() {
        return (source.to_string(), 0);
    }

    replacements.sort_by_key(|(start, _, _)| *start);
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

fn collect_keyframe_selector_alias_replacements(
    source: &str,
    tokens: &[omena_parser::LexedToken],
    keyframes_block_start_index: usize,
    keyframes_block_end_index: usize,
    replacements: &mut Vec<(usize, usize, String)>,
) {
    let mut frame_prelude_start_index = keyframes_block_start_index + 1;
    let mut index = keyframes_block_start_index + 1;

    while index < keyframes_block_end_index {
        match tokens[index].kind {
            SyntaxKind::LeftBrace => {
                let Some(frame_prelude_start) =
                    first_non_trivia_token_start(tokens, frame_prelude_start_index, index)
                else {
                    index += 1;
                    continue;
                };
                let frame_prelude_end = token_start(&tokens[index]);
                let frame_prelude = source[frame_prelude_start..frame_prelude_end].trim();
                if let Some(normalized_frame_prelude) =
                    normalize_keyframe_selector_alias_list(frame_prelude)
                    && normalized_frame_prelude != frame_prelude
                {
                    replacements.push((
                        frame_prelude_start,
                        frame_prelude_end,
                        normalized_frame_prelude,
                    ));
                }

                let Some(close_index) = matching_right_brace_index(tokens, index) else {
                    return;
                };
                index = close_index + 1;
                frame_prelude_start_index = index;
                continue;
            }
            SyntaxKind::Semicolon => {
                frame_prelude_start_index = index + 1;
            }
            _ => {}
        }
        index += 1;
    }
}

fn normalize_keyframe_selector_alias_list(selector_list: &str) -> Option<String> {
    let selectors = split_top_level_value_arguments(selector_list)?;
    let mut changed = false;
    let normalized = selectors
        .into_iter()
        .map(
            |selector| match normalize_keyframe_selector_alias(&selector) {
                Some(normalized_selector) => {
                    changed = true;
                    normalized_selector.to_string()
                }
                None => selector,
            },
        )
        .collect::<Vec<_>>();

    changed.then(|| normalized.join(","))
}

fn normalize_keyframe_selector_alias(selector: &str) -> Option<&'static str> {
    match selector.trim().to_ascii_lowercase().as_str() {
        "from" => Some("0%"),
        "100%" | "to" => Some("to"),
        _ => None,
    }
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

fn expand_specificity_safe_is_selector_lists_with_lexer(
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
        let mut expanded_selectors = Vec::new();
        let mut changed = false;
        for selector in selectors {
            if let Some(expanded) = expand_specificity_safe_is_selector(&selector) {
                expanded_selectors.extend(expanded);
                changed = true;
            } else {
                expanded_selectors.push(selector);
            }
        }
        if !changed {
            continue;
        }
        let deduped = dedupe_selector_arguments(&expanded_selectors);
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

fn expand_specificity_safe_is_selector(selector: &str) -> Option<Vec<String>> {
    let start = selector.find(":is(")?;
    if selector[start + ":is(".len()..].contains(":is(") {
        return None;
    }
    let left_paren_index = start + ":is".len();
    let close_index = matching_function_call_end(selector, left_paren_index)?;
    if selector[close_index + ')'.len_utf8()..].contains(":is(") {
        return None;
    }

    let inner = selector[left_paren_index + '('.len_utf8()..close_index].trim();
    let arguments = split_css_selector_list(inner)?;
    if arguments.len() < 2
        || !arguments
            .iter()
            .all(|argument| is_simple_class_selector(argument))
    {
        return None;
    }

    let prefix = &selector[..start];
    let suffix = &selector[close_index + ')'.len_utf8()..];
    Some(
        arguments
            .into_iter()
            .map(|argument| format!("{prefix}{argument}{suffix}"))
            .collect(),
    )
}

fn is_simple_class_selector(selector: &str) -> bool {
    let Some(class_name) = selector.trim().strip_prefix('.') else {
        return false;
    };
    !class_name.is_empty()
        && class_name
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_'))
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

fn normalize_css_font_declarations_with_lexer(
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
                let Some(replacement_value) = normalize_static_font_declaration_value(
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

fn normalize_static_font_declaration_value(property: &str, value: &str) -> Option<String> {
    match property {
        "font-family" => normalize_static_font_family_value(value),
        "font-weight" => normalize_static_font_weight_value(value),
        "font-stretch" => normalize_static_font_stretch_value(value),
        _ => None,
    }
}

fn normalize_static_font_family_value(value: &str) -> Option<String> {
    let families = split_top_level_value_arguments(value)?;
    let mut normalized = Vec::with_capacity(families.len());
    let mut changed = false;

    for family in families {
        let Some(quoted_family) = static_css_string_value(&family) else {
            normalized.push(family);
            continue;
        };
        let Some(unquoted_family) = unquote_static_font_family_name(&quoted_family) else {
            normalized.push(family);
            continue;
        };
        changed = true;
        normalized.push(unquoted_family);
    }

    changed.then(|| normalized.join(","))
}

fn normalize_static_font_weight_value(value: &str) -> Option<String> {
    match value.trim().to_ascii_lowercase().as_str() {
        "normal" => Some("400".to_string()),
        "bold" => Some("700".to_string()),
        _ => None,
    }
}

fn normalize_static_font_stretch_value(value: &str) -> Option<String> {
    let normalized = match value.trim().to_ascii_lowercase().as_str() {
        "ultra-condensed" => "50%",
        "extra-condensed" => "62.5%",
        "condensed" => "75%",
        "semi-condensed" => "87.5%",
        "normal" => "100%",
        "semi-expanded" => "112.5%",
        "expanded" => "125%",
        "extra-expanded" => "150%",
        "ultra-expanded" => "200%",
        _ => return None,
    };
    Some(normalized.to_string())
}

fn unquote_static_font_family_name(value: &str) -> Option<String> {
    let parts = value.split_ascii_whitespace().collect::<Vec<_>>();
    if parts.is_empty() {
        return None;
    }
    if parts
        .iter()
        .any(|part| !is_safe_unquoted_font_family_identifier(part))
    {
        return None;
    }
    Some(parts.join(" "))
}

fn is_safe_unquoted_font_family_identifier(value: &str) -> bool {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if value.starts_with("--") && value.len() > 2 {
        return chars.all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_'))
            && !is_reserved_unquoted_font_family_identifier(value);
    }
    if first == '-' {
        let Some(second) = chars.next() else {
            return false;
        };
        if !(second.is_ascii_alphabetic() || second == '_') {
            return false;
        }
        return chars.all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_'))
            && !is_reserved_unquoted_font_family_identifier(value);
    }
    if !(first.is_ascii_alphabetic() || first == '_') {
        return false;
    }
    if !chars.all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_')) {
        return false;
    }
    !is_reserved_unquoted_font_family_identifier(value)
}

fn is_reserved_unquoted_font_family_identifier(value: &str) -> bool {
    matches!(
        value.to_ascii_lowercase().as_str(),
        "serif"
            | "sans-serif"
            | "monospace"
            | "cursive"
            | "fantasy"
            | "system-ui"
            | "ui-serif"
            | "ui-sans-serif"
            | "ui-monospace"
            | "ui-rounded"
            | "math"
            | "emoji"
            | "fangsong"
            | "inherit"
            | "initial"
            | "unset"
            | "revert"
            | "revert-layer"
    )
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

fn is_static_color_reference_property(property: &str) -> bool {
    matches!(
        property,
        "accent-color"
            | "background"
            | "background-color"
            | "border"
            | "border-block"
            | "border-block-color"
            | "border-block-end"
            | "border-block-end-color"
            | "border-block-start"
            | "border-block-start-color"
            | "border-bottom"
            | "border-bottom-color"
            | "border-color"
            | "border-inline"
            | "border-inline-color"
            | "border-inline-end"
            | "border-inline-end-color"
            | "border-inline-start"
            | "border-inline-start-color"
            | "border-left"
            | "border-left-color"
            | "border-right"
            | "border-right-color"
            | "border-top"
            | "border-top-color"
            | "box-shadow"
            | "caret-color"
            | "color"
            | "column-rule"
            | "column-rule-color"
            | "fill"
            | "filter"
            | "flood-color"
            | "lighting-color"
            | "outline"
            | "outline-color"
            | "scrollbar-color"
            | "stop-color"
            | "stroke"
            | "text-decoration-color"
            | "text-emphasis-color"
            | "text-shadow"
    )
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
