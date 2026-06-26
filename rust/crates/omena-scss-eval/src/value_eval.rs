use omena_value_lattice::{
    SrgbColor, StaticSrgbColorWithAlpha,
    number::{
        format_css_number, parse_reducible_abs_value, parse_reducible_ceil_value,
        parse_reducible_clamp_value, parse_reducible_exp_value, parse_reducible_floor_value,
        parse_reducible_hypot_value, parse_reducible_log_value, parse_reducible_max_value,
        parse_reducible_min_value, parse_reducible_mod_value, parse_reducible_pow_value,
        parse_reducible_rem_value, parse_reducible_round_to_integer_value,
        parse_reducible_sign_value, parse_reducible_sqrt_value, reduce_static_numeric_expression,
    },
    parse_color_function_value, parse_color_mix_value, parse_numeric_value_with_unit,
    parse_oklab_oklch_value, parse_static_hsl_function_color_with_alpha,
    parse_static_hwb_function_color_with_alpha, parse_static_rgb_function_color_with_alpha,
    parse_static_srgb_color_with_alpha, parse_whole_function_value_arguments,
    parse_whole_function_value_inner, shortest_static_srgb_color_with_alpha_text,
    split_top_level_value_arguments_owned, split_top_level_whitespace_value_components_owned,
    substitute_static_css_function_references_in_value_until_stable,
};

mod collection;
mod map;
mod numeric;
mod string;
mod truthiness;
#[cfg(feature = "scanner-oracle")]
mod truthiness_scanner;
mod unit;

use collection::{
    parse_static_scss_map_entries, split_static_scss_top_level,
    split_static_scss_top_level_whitespace, static_scss_collection_member_is_static,
    static_scss_collection_values_equal, static_scss_comparable_collection_value,
    static_scss_list_separator, static_scss_named_argument, static_scss_named_argument_value,
    static_scss_quoted_value_end, static_scss_top_level_separator_index,
    strip_static_scss_outer_container, strip_static_scss_quotes,
};
use map::{
    parse_static_scss_map_deep_merge_namespaced_value,
    parse_static_scss_map_deep_remove_namespaced_value, parse_static_scss_map_get_namespaced_value,
    parse_static_scss_map_get_value, parse_static_scss_map_has_key_namespaced_value,
    parse_static_scss_map_has_key_value, parse_static_scss_map_keys_namespaced_value,
    parse_static_scss_map_keys_value, parse_static_scss_map_merge_namespaced_value,
    parse_static_scss_map_merge_value, parse_static_scss_map_remove_namespaced_value,
    parse_static_scss_map_remove_value, parse_static_scss_map_set_value,
    parse_static_scss_map_values_namespaced_value, parse_static_scss_map_values_value,
};
pub(crate) use numeric::{reduce_static_less_numeric_value, reduce_static_numeric_value};
use string::{
    parse_static_scss_quote_value, parse_static_scss_str_index_value,
    parse_static_scss_str_insert_value, parse_static_scss_str_length_value,
    parse_static_scss_str_slice_value, parse_static_scss_string_argument,
    parse_static_scss_string_index_value, parse_static_scss_string_insert_value,
    parse_static_scss_string_length_value, parse_static_scss_string_quote_value,
    parse_static_scss_string_slice_value, parse_static_scss_string_to_lower_case_value,
    parse_static_scss_string_to_upper_case_value, parse_static_scss_string_unquote_value,
    parse_static_scss_to_lower_case_value, parse_static_scss_to_upper_case_value,
    parse_static_scss_unquote_value, static_scss_quote_string, static_scss_quoted_string_text,
};
pub(crate) use truthiness::static_scss_literal_truthiness;
#[cfg(feature = "scanner-oracle")]
pub(crate) use truthiness::static_scss_literal_truthiness_scanner_oracle;
#[cfg(feature = "scanner-oracle")]
pub use truthiness::{
    OmenaScssEvalTruthinessCstEquivalenceFixtureReportV0,
    OmenaScssEvalTruthinessCstEquivalenceReportV0, summarize_scss_eval_truthiness_cst_equivalence,
};
use unit::{
    parse_static_scss_comparable_value, parse_static_scss_math_compatible_value,
    parse_static_scss_math_is_unitless_value, parse_static_scss_math_percentage_value,
    parse_static_scss_math_unit_value, parse_static_scss_percentage_value,
    parse_static_scss_unit_value, parse_static_scss_unitless_value,
};

pub(crate) fn reduce_static_scss_value(value: String) -> String {
    let trimmed = value.trim();
    if let Some(value) = parse_static_scss_math_constant_value(trimmed) {
        return value;
    }
    let value = substitute_static_css_function_references_in_value_until_stable(
        trimmed,
        &[
            ("if", parse_static_scss_if_value),
            ("type-of", parse_static_scss_type_of_value),
            ("meta.type-of", parse_static_scss_meta_type_of_value),
            ("inspect", parse_static_scss_inspect_value),
            ("meta.inspect", parse_static_scss_meta_inspect_value),
            ("meta.calc-args", parse_static_scss_meta_calc_args_value),
            ("meta.calc-name", parse_static_scss_meta_calc_name_value),
            ("feature-exists", parse_static_scss_feature_exists_value),
            (
                "meta.feature-exists",
                parse_static_scss_meta_feature_exists_value,
            ),
            ("nth", parse_static_scss_nth_value),
            ("list.nth", parse_static_scss_list_nth_value),
            ("length", parse_static_scss_length_value),
            ("list.length", parse_static_scss_list_length_value),
            ("index", parse_static_scss_index_value),
            ("list.index", parse_static_scss_list_index_value),
            (
                "list-separator",
                parse_static_scss_list_separator_legacy_value,
            ),
            ("list.separator", parse_static_scss_list_separator_value),
            (
                "is-bracketed",
                parse_static_scss_list_is_bracketed_legacy_value,
            ),
            (
                "list.is-bracketed",
                parse_static_scss_list_is_bracketed_value,
            ),
            ("append", parse_static_scss_append_value),
            ("list.append", parse_static_scss_list_append_value),
            ("join", parse_static_scss_join_value),
            ("list.join", parse_static_scss_list_join_value),
            ("set-nth", parse_static_scss_set_nth_value),
            ("list.set-nth", parse_static_scss_list_set_nth_value),
            ("zip", parse_static_scss_zip_value),
            ("list.zip", parse_static_scss_list_zip_value),
            ("list.slash", parse_static_scss_list_slash_value),
            ("quote", parse_static_scss_quote_value),
            ("string.quote", parse_static_scss_string_quote_value),
            ("unquote", parse_static_scss_unquote_value),
            ("string.unquote", parse_static_scss_string_unquote_value),
            ("str-length", parse_static_scss_str_length_value),
            ("string.length", parse_static_scss_string_length_value),
            ("str-index", parse_static_scss_str_index_value),
            ("string.index", parse_static_scss_string_index_value),
            ("str-insert", parse_static_scss_str_insert_value),
            ("string.insert", parse_static_scss_string_insert_value),
            ("str-slice", parse_static_scss_str_slice_value),
            ("string.slice", parse_static_scss_string_slice_value),
            ("to-upper-case", parse_static_scss_to_upper_case_value),
            (
                "string.to-upper-case",
                parse_static_scss_string_to_upper_case_value,
            ),
            ("to-lower-case", parse_static_scss_to_lower_case_value),
            (
                "string.to-lower-case",
                parse_static_scss_string_to_lower_case_value,
            ),
            ("map-get", parse_static_scss_map_get_value),
            ("map.get", parse_static_scss_map_get_namespaced_value),
            ("map-has-key", parse_static_scss_map_has_key_value),
            (
                "map.has-key",
                parse_static_scss_map_has_key_namespaced_value,
            ),
            ("map-keys", parse_static_scss_map_keys_value),
            ("map.keys", parse_static_scss_map_keys_namespaced_value),
            ("map-values", parse_static_scss_map_values_value),
            ("map.values", parse_static_scss_map_values_namespaced_value),
            ("map-merge", parse_static_scss_map_merge_value),
            ("map.merge", parse_static_scss_map_merge_namespaced_value),
            (
                "map.deep-merge",
                parse_static_scss_map_deep_merge_namespaced_value,
            ),
            ("map-remove", parse_static_scss_map_remove_value),
            ("map.remove", parse_static_scss_map_remove_namespaced_value),
            (
                "map.deep-remove",
                parse_static_scss_map_deep_remove_namespaced_value,
            ),
            ("map.set", parse_static_scss_map_set_value),
            ("ceil", parse_static_scss_ceil_value),
            ("floor", parse_static_scss_floor_value),
            ("round", parse_static_scss_round_value),
            ("math.div", parse_static_scss_math_div_value),
            ("math.min", parse_static_scss_math_min_value),
            ("math.max", parse_static_scss_math_max_value),
            ("math.abs", parse_static_scss_math_abs_value),
            ("math.sign", parse_static_scss_math_sign_value),
            ("math.ceil", parse_static_scss_math_ceil_value),
            ("math.floor", parse_static_scss_math_floor_value),
            ("math.round", parse_static_scss_math_round_value),
            ("math.clamp", parse_static_scss_math_clamp_value),
            ("math.mod", parse_static_scss_math_mod_value),
            ("math.rem", parse_static_scss_math_rem_value),
            ("math.hypot", parse_static_scss_math_hypot_value),
            ("math.sqrt", parse_static_scss_math_sqrt_value),
            ("math.pow", parse_static_scss_math_pow_value),
            ("math.exp", parse_static_scss_math_exp_value),
            ("math.log", parse_static_scss_math_log_value),
            ("math.sin", parse_static_scss_math_sin_value),
            ("math.cos", parse_static_scss_math_cos_value),
            ("math.tan", parse_static_scss_math_tan_value),
            ("math.asin", parse_static_scss_math_asin_value),
            ("math.acos", parse_static_scss_math_acos_value),
            ("math.atan", parse_static_scss_math_atan_value),
            ("math.atan2", parse_static_scss_math_atan2_value),
            ("mix", parse_static_scss_global_mix_value),
            ("color.mix", parse_static_scss_color_mix_value),
            ("color.channel", parse_static_scss_color_channel_value),
            ("transparentize", parse_static_scss_transparentize_value),
            ("fade-out", parse_static_scss_fade_out_value),
            ("opacify", parse_static_scss_opacify_value),
            ("fade-in", parse_static_scss_fade_in_value),
            ("color.adjust", parse_static_scss_color_adjust_alias_value),
            ("adjust-color", parse_static_scss_adjust_color_value),
            ("color.change", parse_static_scss_color_change_alias_value),
            ("change-color", parse_static_scss_change_color_value),
            ("color.scale", parse_static_scss_color_scale_alias_value),
            ("scale-color", parse_static_scss_scale_color_value),
            ("adjust-hue", parse_static_scss_adjust_hue_value),
            ("complement", parse_static_scss_complement_value),
            ("color.complement", parse_static_scss_color_complement_value),
            ("lighten", parse_static_scss_lighten_value),
            ("darken", parse_static_scss_darken_value),
            ("saturate", parse_static_scss_saturate_value),
            ("desaturate", parse_static_scss_desaturate_value),
            ("grayscale", parse_static_scss_grayscale_value),
            ("color.grayscale", parse_static_scss_color_grayscale_value),
            ("invert", parse_static_scss_invert_value),
            ("color.invert", parse_static_scss_color_invert_value),
            ("red", parse_static_scss_global_red_value),
            ("green", parse_static_scss_global_green_value),
            ("blue", parse_static_scss_global_blue_value),
            ("alpha", parse_static_scss_global_alpha_value),
            ("opacity", parse_static_scss_global_opacity_value),
            ("color.red", parse_static_scss_color_red_value),
            ("color.green", parse_static_scss_color_green_value),
            ("color.blue", parse_static_scss_color_blue_value),
            ("color.alpha", parse_static_scss_color_alpha_value),
            ("color.opacity", parse_static_scss_color_opacity_alias_value),
            ("hue", parse_static_scss_global_hue_value),
            ("color.hue", parse_static_scss_color_hue_value),
            ("saturation", parse_static_scss_global_saturation_value),
            ("color.saturation", parse_static_scss_color_saturation_value),
            ("lightness", parse_static_scss_global_lightness_value),
            ("color.lightness", parse_static_scss_color_lightness_value),
            ("ie-hex-str", parse_static_scss_ie_hex_str_value),
            ("percentage", parse_static_scss_percentage_value),
            ("math.percentage", parse_static_scss_math_percentage_value),
            ("unit", parse_static_scss_unit_value),
            ("math.unit", parse_static_scss_math_unit_value),
            ("unitless", parse_static_scss_unitless_value),
            ("math.is-unitless", parse_static_scss_math_is_unitless_value),
            ("comparable", parse_static_scss_comparable_value),
            ("math.compatible", parse_static_scss_math_compatible_value),
        ],
    )
    .unwrap_or_else(|| trimmed.to_string());
    let value = parse_static_scss_sass_color_constructor_value(value.as_str()).unwrap_or(value);
    let value = parse_static_scss_sass_color_opacity_value(value.as_str()).unwrap_or(value);
    let value = parse_static_scss_sass_color_transform_value(value.as_str()).unwrap_or(value);
    let value = parse_static_scss_sass_color_channel_value(value.as_str()).unwrap_or(value);
    reduce_static_numeric_value(value)
}

pub(crate) fn static_scss_bang_usage_is_comparison_only(value: &str) -> bool {
    let mut index = 0usize;
    while let Some(relative_index) = value[index..].find('!') {
        let bang_index = index + relative_index;
        if !value
            .get(bang_index + '!'.len_utf8()..)
            .is_some_and(|suffix| suffix.starts_with('='))
        {
            return false;
        }
        index = bang_index + '!'.len_utf8();
    }
    true
}

fn parse_static_scss_if_value(value: &str) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, "if")?;
    let [condition, truthy, falsey] = arguments.as_slice() else {
        return None;
    };
    let truthiness = static_scss_literal_truthiness(condition.trim())?;
    Some(if truthiness {
        truthy.trim().to_string()
    } else {
        falsey.trim().to_string()
    })
}

fn parse_static_scss_type_of_value(value: &str) -> Option<String> {
    parse_static_scss_type_of_value_with_name(value, "type-of")
}

fn parse_static_scss_meta_type_of_value(value: &str) -> Option<String> {
    parse_static_scss_type_of_value_with_name(value, "meta.type-of")
}

fn parse_static_scss_type_of_value_with_name(value: &str, function_name: &str) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, function_name)?;
    let [value] = arguments.as_slice() else {
        return None;
    };
    Some(static_scss_value_type(value)?.to_string())
}

fn parse_static_scss_inspect_value(value: &str) -> Option<String> {
    parse_static_scss_inspect_value_with_name(value, "inspect")
}

fn parse_static_scss_meta_inspect_value(value: &str) -> Option<String> {
    parse_static_scss_inspect_value_with_name(value, "meta.inspect")
}

fn parse_static_scss_inspect_value_with_name(value: &str, function_name: &str) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, function_name)?;
    let [value] = arguments.as_slice() else {
        return None;
    };
    let reduced = reduce_static_scss_value(value.to_string());
    match static_scss_value_type(reduced.as_str())? {
        "map" => None,
        _ => Some(reduced.trim().to_string()),
    }
}

fn parse_static_scss_meta_calc_name_value(value: &str) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, "meta.calc-name")?;
    let [calculation] = arguments.as_slice() else {
        return None;
    };
    static_scss_quote_string(static_scss_calculation_name(calculation)?)
}

fn parse_static_scss_meta_calc_args_value(value: &str) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, "meta.calc-args")?;
    let [calculation] = arguments.as_slice() else {
        return None;
    };
    static_scss_calculation_args(calculation)
}

fn parse_static_scss_feature_exists_value(value: &str) -> Option<String> {
    parse_static_scss_feature_exists_value_with_name(value, "feature-exists")
}

fn parse_static_scss_meta_feature_exists_value(value: &str) -> Option<String> {
    parse_static_scss_feature_exists_value_with_name(value, "meta.feature-exists")
}

fn parse_static_scss_feature_exists_value_with_name(
    value: &str,
    function_name: &str,
) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, function_name)?;
    let [feature] = arguments.as_slice() else {
        return None;
    };
    let feature = parse_static_scss_string_argument(feature)?;
    Some(static_scss_feature_exists(feature.text.as_str()).to_string())
}

fn parse_static_scss_nth_value(value: &str) -> Option<String> {
    parse_static_scss_nth_value_with_name(value, "nth")
}

fn parse_static_scss_list_nth_value(value: &str) -> Option<String> {
    parse_static_scss_nth_value_with_name(value, "list.nth")
}

fn parse_static_scss_length_value(value: &str) -> Option<String> {
    parse_static_scss_length_value_with_name(value, "length")
}

fn parse_static_scss_list_length_value(value: &str) -> Option<String> {
    parse_static_scss_length_value_with_name(value, "list.length")
}

fn parse_static_scss_index_value(value: &str) -> Option<String> {
    parse_static_scss_index_value_with_name(value, "index")
}

fn parse_static_scss_list_index_value(value: &str) -> Option<String> {
    parse_static_scss_index_value_with_name(value, "list.index")
}

fn parse_static_scss_list_separator_value(value: &str) -> Option<String> {
    parse_static_scss_list_separator_value_with_name(value, "list.separator")
}

fn parse_static_scss_list_separator_legacy_value(value: &str) -> Option<String> {
    parse_static_scss_list_separator_value_with_name(value, "list-separator")
}

fn parse_static_scss_list_separator_value_with_name(
    value: &str,
    function_name: &str,
) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, function_name)?;
    let [list] = arguments.as_slice() else {
        return None;
    };
    Some(format!("\"{}\"", static_scss_list_separator(list)?))
}

fn parse_static_scss_list_is_bracketed_value(value: &str) -> Option<String> {
    parse_static_scss_list_is_bracketed_value_with_name(value, "list.is-bracketed")
}

fn parse_static_scss_list_is_bracketed_legacy_value(value: &str) -> Option<String> {
    parse_static_scss_list_is_bracketed_value_with_name(value, "is-bracketed")
}

fn parse_static_scss_list_is_bracketed_value_with_name(
    value: &str,
    function_name: &str,
) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, function_name)?;
    let [list] = arguments.as_slice() else {
        return None;
    };
    let list = list.trim();
    let bracketed = list.starts_with('[') && strip_static_scss_outer_container(list).is_some();
    Some(bracketed.to_string())
}

fn parse_static_scss_append_value(value: &str) -> Option<String> {
    parse_static_scss_append_value_with_name(value, "append")
}

fn parse_static_scss_list_append_value(value: &str) -> Option<String> {
    parse_static_scss_append_value_with_name(value, "list.append")
}

fn parse_static_scss_join_value(value: &str) -> Option<String> {
    parse_static_scss_join_value_with_name(value, "join")
}

fn parse_static_scss_list_join_value(value: &str) -> Option<String> {
    parse_static_scss_join_value_with_name(value, "list.join")
}

fn parse_static_scss_set_nth_value(value: &str) -> Option<String> {
    parse_static_scss_set_nth_value_with_name(value, "set-nth")
}

fn parse_static_scss_list_set_nth_value(value: &str) -> Option<String> {
    parse_static_scss_set_nth_value_with_name(value, "list.set-nth")
}

fn parse_static_scss_zip_value(value: &str) -> Option<String> {
    parse_static_scss_zip_value_with_name(value, "zip")
}

fn parse_static_scss_list_zip_value(value: &str) -> Option<String> {
    parse_static_scss_zip_value_with_name(value, "list.zip")
}

fn parse_static_scss_list_slash_value(value: &str) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, "list.slash")?;
    if arguments.is_empty() {
        return None;
    }
    let items = arguments
        .iter()
        .map(|argument| static_scss_list_append_item_text(argument))
        .collect::<Option<Vec<_>>>()?;
    static_scss_render_list_value(&StaticScssListValue {
        items,
        separator: StaticScssListSeparator::Slash,
        bracketed: false,
    })
}

fn parse_static_scss_append_value_with_name(value: &str, function_name: &str) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, function_name)?;
    let [list, value, options @ ..] = arguments.as_slice() else {
        return None;
    };
    if options.len() > 1 {
        return None;
    }
    let mut list = parse_static_scss_list_value(list)?;
    let separator = match options {
        [] => list.separator,
        [option] => {
            let option = static_scss_named_argument_value(option, "separator")?.unwrap_or(option);
            parse_static_scss_list_separator_option(option, list.separator)?
        }
        _ => return None,
    };
    list.separator = separator;
    list.items.push(static_scss_list_append_item_text(value)?);
    static_scss_render_list_value(&list)
}

fn parse_static_scss_join_value_with_name(value: &str, function_name: &str) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, function_name)?;
    let [left, right, options @ ..] = arguments.as_slice() else {
        return None;
    };
    if options.len() > 2 {
        return None;
    }
    let left = parse_static_scss_list_value(left)?;
    let right = parse_static_scss_list_value(right)?;
    let mut separator = if left.items.len() > 1 {
        left.separator
    } else if right.items.len() > 1 {
        right.separator
    } else {
        StaticScssListSeparator::Space
    };
    let mut bracketed = left.bracketed;
    for (index, option) in options.iter().enumerate() {
        match static_scss_named_argument(option)? {
            Some(("separator", value)) => {
                separator = parse_static_scss_list_separator_option(value, separator)?;
            }
            Some(("bracketed", value)) => {
                bracketed = parse_static_scss_list_bracketed_option(value, left.bracketed)?;
            }
            Some(_) => return None,
            None if index == 0 => {
                separator = parse_static_scss_list_separator_option(option, separator)?;
            }
            None if index == 1 => {
                bracketed = parse_static_scss_list_bracketed_option(option, left.bracketed)?;
            }
            None => return None,
        }
    }
    let items = left
        .items
        .into_iter()
        .chain(right.items)
        .collect::<Vec<_>>();
    static_scss_render_list_value(&StaticScssListValue {
        items,
        separator,
        bracketed,
    })
}

fn parse_static_scss_set_nth_value_with_name(value: &str, function_name: &str) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, function_name)?;
    let [list, index, replacement] = arguments.as_slice() else {
        return None;
    };
    let mut list = parse_static_scss_list_value(list)?;
    let index = parse_static_scss_list_index(index)?;
    let resolved_index = static_scss_resolved_list_index(index, list.items.len())?;
    list.items[resolved_index] = static_scss_list_append_item_text(replacement)?;
    static_scss_render_list_value(&list)
}

fn parse_static_scss_zip_value_with_name(value: &str, function_name: &str) -> Option<String> {
    let lists = parse_whole_function_value_arguments(value, function_name)?;
    if lists.is_empty() {
        return None;
    }
    let lists = lists
        .iter()
        .map(|list| parse_static_scss_list_value(list))
        .collect::<Option<Vec<_>>>()?;
    let len = lists.iter().map(|list| list.items.len()).min()?;
    let items = (0..len)
        .map(|index| {
            lists
                .iter()
                .map(|list| list.items[index].clone())
                .collect::<Vec<_>>()
                .join(" ")
        })
        .collect::<Vec<_>>();
    static_scss_render_list_value(&StaticScssListValue {
        items,
        separator: StaticScssListSeparator::Comma,
        bracketed: false,
    })
}

fn parse_static_scss_nth_value_with_name(value: &str, function_name: &str) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, function_name)?;
    let [list, index] = arguments.as_slice() else {
        return None;
    };
    let items = parse_static_scss_list_items(list)?;
    let index = parse_static_scss_list_index(index)?;
    let resolved_index = static_scss_resolved_list_index(index, items.len())?;
    items.get(resolved_index).cloned()
}

fn parse_static_scss_length_value_with_name(value: &str, function_name: &str) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, function_name)?;
    let [list] = arguments.as_slice() else {
        return None;
    };
    if let Some(entries) = parse_static_scss_map_entries(list) {
        return Some(entries.len().to_string());
    }
    Some(parse_static_scss_list_items(list)?.len().to_string())
}

fn parse_static_scss_index_value_with_name(value: &str, function_name: &str) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, function_name)?;
    let [list, needle] = arguments.as_slice() else {
        return None;
    };
    let needle = static_scss_comparable_collection_value(needle)?;
    parse_static_scss_list_items(list)?
        .into_iter()
        .position(|item| {
            static_scss_comparable_collection_value(item.as_str()).is_some_and(|item| {
                static_scss_collection_values_equal(item.as_str(), needle.as_str())
            })
        })
        .map(|index| (index + 1).to_string())
}

fn parse_static_scss_math_div_value(value: &str) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, "math.div")?;
    let [left, right] = arguments.as_slice() else {
        return None;
    };
    reduce_static_numeric_expression(format!("{} / {}", left.trim(), right.trim()).as_str())
}

fn parse_static_scss_math_min_value(value: &str) -> Option<String> {
    parse_static_scss_numeric_alias_value(value, "math.min", "min", parse_reducible_min_value)
}

fn parse_static_scss_math_max_value(value: &str) -> Option<String> {
    parse_static_scss_numeric_alias_value(value, "math.max", "max", parse_reducible_max_value)
}

fn parse_static_scss_math_abs_value(value: &str) -> Option<String> {
    parse_static_scss_numeric_alias_value(value, "math.abs", "abs", parse_reducible_abs_value)
}

fn parse_static_scss_math_sign_value(value: &str) -> Option<String> {
    parse_static_scss_numeric_alias_value(value, "math.sign", "sign", parse_reducible_sign_value)
}

fn parse_static_scss_ceil_value(value: &str) -> Option<String> {
    parse_static_scss_numeric_alias_value(value, "ceil", "ceil", parse_reducible_ceil_value)
}

fn parse_static_scss_math_ceil_value(value: &str) -> Option<String> {
    parse_static_scss_numeric_alias_value(value, "math.ceil", "ceil", parse_reducible_ceil_value)
}

fn parse_static_scss_floor_value(value: &str) -> Option<String> {
    parse_static_scss_numeric_alias_value(value, "floor", "floor", parse_reducible_floor_value)
}

fn parse_static_scss_math_floor_value(value: &str) -> Option<String> {
    parse_static_scss_numeric_alias_value(value, "math.floor", "floor", parse_reducible_floor_value)
}

fn parse_static_scss_round_value(value: &str) -> Option<String> {
    parse_static_scss_numeric_alias_value(
        value,
        "round",
        "round",
        parse_reducible_round_to_integer_value,
    )
}

fn parse_static_scss_math_round_value(value: &str) -> Option<String> {
    parse_static_scss_numeric_alias_value(
        value,
        "math.round",
        "round",
        parse_reducible_round_to_integer_value,
    )
}

fn parse_static_scss_math_clamp_value(value: &str) -> Option<String> {
    parse_static_scss_numeric_alias_value(value, "math.clamp", "clamp", parse_reducible_clamp_value)
}

fn parse_static_scss_math_mod_value(value: &str) -> Option<String> {
    parse_static_scss_numeric_alias_value(value, "math.mod", "mod", parse_reducible_mod_value)
}

fn parse_static_scss_math_rem_value(value: &str) -> Option<String> {
    parse_static_scss_numeric_alias_value(value, "math.rem", "rem", parse_reducible_rem_value)
}

fn parse_static_scss_math_hypot_value(value: &str) -> Option<String> {
    parse_static_scss_numeric_alias_value(value, "math.hypot", "hypot", parse_reducible_hypot_value)
}

fn parse_static_scss_math_sqrt_value(value: &str) -> Option<String> {
    parse_static_scss_numeric_alias_value(value, "math.sqrt", "sqrt", parse_reducible_sqrt_value)
}

fn parse_static_scss_math_pow_value(value: &str) -> Option<String> {
    parse_static_scss_numeric_alias_value(value, "math.pow", "pow", parse_reducible_pow_value)
}

fn parse_static_scss_math_exp_value(value: &str) -> Option<String> {
    parse_static_scss_numeric_alias_value(value, "math.exp", "exp", parse_reducible_exp_value)
}

fn parse_static_scss_math_log_value(value: &str) -> Option<String> {
    parse_static_scss_numeric_alias_value(value, "math.log", "log", parse_reducible_log_value)
}

fn parse_static_scss_math_constant_value(value: &str) -> Option<String> {
    match value.trim() {
        "math.$pi" => Some("3.1415926536".to_string()),
        "math.$e" => Some("2.7182818285".to_string()),
        "math.$epsilon" | "math.$min-number" => Some("0".to_string()),
        "math.$max-safe-integer" => Some("9007199254740991".to_string()),
        "math.$min-safe-integer" => Some("-9007199254740991".to_string()),
        _ => None,
    }
}

fn parse_static_scss_math_sin_value(value: &str) -> Option<String> {
    parse_static_scss_math_trig_value(value, "math.sin", f64::sin)
}

fn parse_static_scss_math_cos_value(value: &str) -> Option<String> {
    parse_static_scss_math_trig_value(value, "math.cos", f64::cos)
}

fn parse_static_scss_math_tan_value(value: &str) -> Option<String> {
    parse_static_scss_math_trig_value(value, "math.tan", f64::tan)
}

fn parse_static_scss_math_asin_value(value: &str) -> Option<String> {
    parse_static_scss_math_inverse_trig_value(value, "math.asin", f64::asin, true)
}

fn parse_static_scss_math_acos_value(value: &str) -> Option<String> {
    parse_static_scss_math_inverse_trig_value(value, "math.acos", f64::acos, true)
}

fn parse_static_scss_math_atan_value(value: &str) -> Option<String> {
    parse_static_scss_math_inverse_trig_value(value, "math.atan", f64::atan, false)
}

fn parse_static_scss_math_atan2_value(value: &str) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, "math.atan2")?;
    let [y, x] = arguments.as_slice() else {
        return None;
    };
    let y = reduce_static_scss_value(y.trim().to_string());
    let x = reduce_static_scss_value(x.trim().to_string());
    let y = parse_numeric_value_with_unit(y.as_str())?;
    let x = parse_numeric_value_with_unit(x.as_str())?;
    if !y.unit.eq_ignore_ascii_case(x.unit) {
        return None;
    }
    let degrees = y.value.atan2(x.value).to_degrees();
    Some(format!(
        "{}deg",
        format_static_scss_math_trig_number(degrees)?
    ))
}

fn parse_static_scss_math_trig_value(
    value: &str,
    function_name: &str,
    evaluate: fn(f64) -> f64,
) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, function_name)?;
    let [angle] = arguments.as_slice() else {
        return None;
    };
    let radians = parse_static_scss_angle_radians(angle.trim())?;
    format_static_scss_math_trig_number(evaluate(radians))
}

fn parse_static_scss_math_inverse_trig_value(
    value: &str,
    function_name: &str,
    evaluate: fn(f64) -> f64,
    requires_unit_interval: bool,
) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, function_name)?;
    let [number] = arguments.as_slice() else {
        return None;
    };
    let number = reduce_static_scss_value(number.trim().to_string());
    let number = parse_numeric_value_with_unit(number.as_str())?;
    if !number.unit.is_empty() {
        return None;
    }
    if requires_unit_interval && !(-1.0..=1.0).contains(&number.value) {
        return None;
    }
    Some(format!(
        "{}deg",
        format_static_scss_math_trig_number(evaluate(number.value).to_degrees())?
    ))
}

fn parse_static_scss_angle_radians(value: &str) -> Option<f64> {
    let value = reduce_static_scss_value(value.to_string());
    let angle = parse_numeric_value_with_unit(value.as_str())?;
    let radians = match angle.unit.to_ascii_lowercase().as_str() {
        "" | "rad" => angle.value,
        "deg" => angle.value.to_radians(),
        "grad" => angle.value * std::f64::consts::PI / 200.0,
        "turn" => angle.value * std::f64::consts::TAU,
        _ => return None,
    };
    radians.is_finite().then_some(radians)
}

fn format_static_scss_math_trig_number(value: f64) -> Option<String> {
    if !value.is_finite() {
        return None;
    }
    let value = if value.abs() < 1e-10 { 0.0 } else { value };
    Some(format_css_number(value))
}

fn parse_static_scss_numeric_alias_value(
    value: &str,
    alias_name: &str,
    kernel_name: &str,
    parse_kernel_value: fn(&str) -> Option<String>,
) -> Option<String> {
    let inner = omena_value_lattice::parse_whole_function_value_inner(value, alias_name)?;
    parse_kernel_value(format!("{kernel_name}({inner})").as_str())
}

fn parse_static_scss_rgb_color_constructor_value(value: &str) -> Option<String> {
    parse_static_scss_sass_color_constructor_value_with_name(value, "rgb")
}

fn parse_static_scss_rgba_color_constructor_value(value: &str) -> Option<String> {
    parse_static_scss_sass_color_constructor_value_with_name(value, "rgba")
}

fn parse_static_scss_hsl_color_constructor_value(value: &str) -> Option<String> {
    parse_static_scss_sass_hsl_color_constructor_value_with_name(value, "hsl")
}

fn parse_static_scss_hsla_color_constructor_value(value: &str) -> Option<String> {
    parse_static_scss_sass_hsl_color_constructor_value_with_name(value, "hsla")
}

fn parse_static_scss_sass_color_constructor_value(value: &str) -> Option<String> {
    parse_static_scss_rgb_color_constructor_value(value)
        .or_else(|| parse_static_scss_rgba_color_constructor_value(value))
        .or_else(|| parse_static_scss_hsl_color_constructor_value(value))
        .or_else(|| parse_static_scss_hsla_color_constructor_value(value))
}

fn parse_static_scss_sass_color_opacity_value(value: &str) -> Option<String> {
    parse_static_scss_color_alpha_delta_value(value, "transparentize", -1.0)
        .or_else(|| parse_static_scss_color_alpha_delta_value(value, "fade-out", -1.0))
        .or_else(|| parse_static_scss_color_alpha_delta_value(value, "opacify", 1.0))
        .or_else(|| parse_static_scss_color_alpha_delta_value(value, "fade-in", 1.0))
}

fn parse_static_scss_transparentize_value(value: &str) -> Option<String> {
    parse_static_scss_color_alpha_delta_value(value, "transparentize", -1.0)
}

fn parse_static_scss_fade_out_value(value: &str) -> Option<String> {
    parse_static_scss_color_alpha_delta_value(value, "fade-out", -1.0)
}

fn parse_static_scss_opacify_value(value: &str) -> Option<String> {
    parse_static_scss_color_alpha_delta_value(value, "opacify", 1.0)
}

fn parse_static_scss_fade_in_value(value: &str) -> Option<String> {
    parse_static_scss_color_alpha_delta_value(value, "fade-in", 1.0)
}

fn parse_static_scss_sass_color_transform_value(value: &str) -> Option<String> {
    parse_static_scss_color_adjust_value(value, "color.adjust")
        .or_else(|| parse_static_scss_color_adjust_value(value, "adjust-color"))
        .or_else(|| parse_static_scss_color_change_value(value, "color.change"))
        .or_else(|| parse_static_scss_color_change_value(value, "change-color"))
        .or_else(|| parse_static_scss_color_scale_value(value, "color.scale"))
        .or_else(|| parse_static_scss_color_scale_value(value, "scale-color"))
}

fn parse_static_scss_color_adjust_alias_value(value: &str) -> Option<String> {
    parse_static_scss_color_adjust_value(value, "color.adjust")
}

fn parse_static_scss_adjust_color_value(value: &str) -> Option<String> {
    parse_static_scss_color_adjust_value(value, "adjust-color")
}

fn parse_static_scss_color_change_alias_value(value: &str) -> Option<String> {
    parse_static_scss_color_change_value(value, "color.change")
}

fn parse_static_scss_change_color_value(value: &str) -> Option<String> {
    parse_static_scss_color_change_value(value, "change-color")
}

fn parse_static_scss_color_scale_alias_value(value: &str) -> Option<String> {
    parse_static_scss_color_scale_value(value, "color.scale")
}

fn parse_static_scss_scale_color_value(value: &str) -> Option<String> {
    parse_static_scss_color_scale_value(value, "scale-color")
}

fn parse_static_scss_sass_color_channel_value(value: &str) -> Option<String> {
    parse_static_scss_legacy_color_channel_value(value, "opacity", StaticScssColorChannel::Alpha)
        .or_else(|| {
            parse_static_scss_legacy_color_channel_value(
                value,
                "color.opacity",
                StaticScssColorChannel::Alpha,
            )
        })
        .or_else(|| {
            parse_static_scss_legacy_color_channel_value(value, "hue", StaticScssColorChannel::Hue)
        })
        .or_else(|| {
            parse_static_scss_legacy_color_channel_value(
                value,
                "color.hue",
                StaticScssColorChannel::Hue,
            )
        })
        .or_else(|| {
            parse_static_scss_legacy_color_channel_value(
                value,
                "saturation",
                StaticScssColorChannel::Saturation,
            )
        })
        .or_else(|| {
            parse_static_scss_legacy_color_channel_value(
                value,
                "color.saturation",
                StaticScssColorChannel::Saturation,
            )
        })
        .or_else(|| {
            parse_static_scss_legacy_color_channel_value(
                value,
                "lightness",
                StaticScssColorChannel::Lightness,
            )
        })
        .or_else(|| {
            parse_static_scss_legacy_color_channel_value(
                value,
                "color.lightness",
                StaticScssColorChannel::Lightness,
            )
        })
}

fn parse_static_scss_global_opacity_value(value: &str) -> Option<String> {
    parse_static_scss_legacy_color_channel_value(value, "opacity", StaticScssColorChannel::Alpha)
}

fn parse_static_scss_color_opacity_alias_value(value: &str) -> Option<String> {
    parse_static_scss_legacy_color_channel_value(
        value,
        "color.opacity",
        StaticScssColorChannel::Alpha,
    )
}

fn parse_static_scss_global_hue_value(value: &str) -> Option<String> {
    parse_static_scss_legacy_color_channel_value(value, "hue", StaticScssColorChannel::Hue)
}

fn parse_static_scss_color_hue_value(value: &str) -> Option<String> {
    parse_static_scss_legacy_color_channel_value(value, "color.hue", StaticScssColorChannel::Hue)
}

fn parse_static_scss_global_saturation_value(value: &str) -> Option<String> {
    parse_static_scss_legacy_color_channel_value(
        value,
        "saturation",
        StaticScssColorChannel::Saturation,
    )
}

fn parse_static_scss_color_saturation_value(value: &str) -> Option<String> {
    parse_static_scss_legacy_color_channel_value(
        value,
        "color.saturation",
        StaticScssColorChannel::Saturation,
    )
}

fn parse_static_scss_global_lightness_value(value: &str) -> Option<String> {
    parse_static_scss_legacy_color_channel_value(
        value,
        "lightness",
        StaticScssColorChannel::Lightness,
    )
}

fn parse_static_scss_color_lightness_value(value: &str) -> Option<String> {
    parse_static_scss_legacy_color_channel_value(
        value,
        "color.lightness",
        StaticScssColorChannel::Lightness,
    )
}

fn parse_static_scss_sass_color_constructor_value_with_name(
    value: &str,
    function_name: &str,
) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, function_name)?;
    let [color, alpha] = arguments.as_slice() else {
        return None;
    };
    let color = parse_static_scss_srgb_color_argument(
        reduce_static_scss_value(color.to_string()).as_str(),
    )?;
    let alpha = parse_static_scss_alpha_channel(alpha)?;
    Some(render_static_scss_sass_color_constructor(color, alpha))
}

fn parse_static_scss_sass_hsl_color_constructor_value_with_name(
    value: &str,
    function_name: &str,
) -> Option<String> {
    let inner = parse_whole_function_value_inner(value, function_name)?;
    let color =
        parse_static_hsl_function_color_with_alpha(format!("{function_name}({inner})").as_str())?;
    Some(render_static_scss_sass_color_constructor(
        static_srgb_with_alpha_to_scss_value(color),
        color.alpha.unwrap_or(1.0),
    ))
}

fn parse_static_scss_color_alpha_delta_value(
    value: &str,
    function_name: &str,
    direction: f64,
) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, function_name)?;
    let arguments = parse_static_scss_color_alpha_amount_arguments(arguments.as_slice())?;
    let color = parse_static_scss_srgb_color_argument(
        reduce_static_scss_value(arguments.color.to_string()).as_str(),
    )?;
    let amount = parse_static_scss_unitless_alpha_amount(arguments.amount)?;
    let alpha = static_scss_clamp_alpha(color.alpha + direction * amount);
    Some(render_static_scss_sass_color_constructor(color, alpha))
}

fn parse_static_scss_color_adjust_value(value: &str, function_name: &str) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, function_name)?;
    let arguments = parse_static_scss_color_named_channel_arguments(arguments.as_slice())?;
    if !arguments.has_any_adjustment() {
        return None;
    }
    let color = parse_static_scss_srgb_color_argument(
        reduce_static_scss_value(arguments.color.to_string()).as_str(),
    )?;
    let mut hsl = StaticScssHslColorValue::from_srgb(color);
    if let Some(hue) = arguments.hue {
        hsl.hue = (hsl.hue + parse_static_scss_hue_degrees(hue)?).rem_euclid(360.0);
    }
    if let Some(saturation) = arguments.saturation {
        hsl.saturation =
            (hsl.saturation + parse_static_scss_percent_adjustment(saturation)?).clamp(0.0, 1.0);
    }
    if let Some(lightness) = arguments.lightness {
        hsl.lightness =
            (hsl.lightness + parse_static_scss_percent_adjustment(lightness)?).clamp(0.0, 1.0);
    }
    if let Some(alpha) = arguments.alpha {
        hsl.alpha = static_scss_clamp_alpha(hsl.alpha + parse_static_scss_alpha_adjustment(alpha)?);
    }
    Some(render_static_scss_sass_color_constructor(
        hsl.to_srgb(),
        hsl.alpha,
    ))
}

fn parse_static_scss_adjust_hue_value(value: &str) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, "adjust-hue")?;
    let arguments = parse_static_scss_color_hue_amount_arguments(arguments.as_slice())?;
    let color = parse_static_scss_srgb_color_argument(
        reduce_static_scss_value(arguments.color.to_string()).as_str(),
    )?;
    let mut hsl = StaticScssHslColorValue::from_srgb(color);
    hsl.hue = (hsl.hue + parse_static_scss_hue_degrees(arguments.amount)?).rem_euclid(360.0);
    Some(render_static_scss_sass_color_constructor(
        hsl.to_srgb(),
        hsl.alpha,
    ))
}

fn parse_static_scss_complement_value(value: &str) -> Option<String> {
    parse_static_scss_complement_value_with_name(value, "complement")
}

fn parse_static_scss_color_complement_value(value: &str) -> Option<String> {
    parse_static_scss_complement_value_with_name(value, "color.complement")
}

fn parse_static_scss_complement_value_with_name(
    value: &str,
    function_name: &str,
) -> Option<String> {
    let color = parse_static_scss_single_color_function_argument(value, function_name)?;
    let mut hsl = StaticScssHslColorValue::from_srgb(color);
    hsl.hue = (hsl.hue + 180.0).rem_euclid(360.0);
    Some(render_static_scss_sass_color_constructor(
        hsl.to_srgb(),
        hsl.alpha,
    ))
}

fn parse_static_scss_lighten_value(value: &str) -> Option<String> {
    parse_static_scss_lightness_delta_value(value, "lighten", 1.0)
}

fn parse_static_scss_darken_value(value: &str) -> Option<String> {
    parse_static_scss_lightness_delta_value(value, "darken", -1.0)
}

fn parse_static_scss_lightness_delta_value(
    value: &str,
    function_name: &str,
    direction: f64,
) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, function_name)?;
    let arguments = parse_static_scss_color_alpha_amount_arguments(arguments.as_slice())?;
    let color = parse_static_scss_srgb_color_argument(
        reduce_static_scss_value(arguments.color.to_string()).as_str(),
    )?;
    let amount = parse_static_scss_percent_adjustment(arguments.amount)?;
    if !(0.0..=1.0).contains(&amount) {
        return None;
    }
    let mut hsl = StaticScssHslColorValue::from_srgb(color);
    hsl.lightness = (hsl.lightness + direction * amount).clamp(0.0, 1.0);
    Some(render_static_scss_sass_color_constructor(
        hsl.to_srgb(),
        hsl.alpha,
    ))
}

fn parse_static_scss_saturate_value(value: &str) -> Option<String> {
    parse_static_scss_saturation_delta_value(value, "saturate", 1.0)
}

fn parse_static_scss_desaturate_value(value: &str) -> Option<String> {
    parse_static_scss_saturation_delta_value(value, "desaturate", -1.0)
}

fn parse_static_scss_saturation_delta_value(
    value: &str,
    function_name: &str,
    direction: f64,
) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, function_name)?;
    let arguments = parse_static_scss_color_alpha_amount_arguments(arguments.as_slice())?;
    let color = parse_static_scss_srgb_color_argument(
        reduce_static_scss_value(arguments.color.to_string()).as_str(),
    )?;
    let amount = parse_static_scss_percent_adjustment(arguments.amount)?;
    if !(0.0..=1.0).contains(&amount) {
        return None;
    }
    let mut hsl = StaticScssHslColorValue::from_srgb(color);
    hsl.saturation = (hsl.saturation + direction * amount).clamp(0.0, 1.0);
    Some(render_static_scss_sass_color_constructor(
        hsl.to_srgb(),
        hsl.alpha,
    ))
}

fn parse_static_scss_grayscale_value(value: &str) -> Option<String> {
    parse_static_scss_grayscale_value_with_name(value, "grayscale")
}

fn parse_static_scss_color_grayscale_value(value: &str) -> Option<String> {
    parse_static_scss_grayscale_value_with_name(value, "color.grayscale")
}

fn parse_static_scss_grayscale_value_with_name(value: &str, function_name: &str) -> Option<String> {
    let color = parse_static_scss_single_color_function_argument(value, function_name)?;
    let mut hsl = StaticScssHslColorValue::from_srgb(color);
    hsl.saturation = 0.0;
    Some(render_static_scss_sass_color_constructor(
        hsl.to_srgb(),
        hsl.alpha,
    ))
}

fn parse_static_scss_invert_value(value: &str) -> Option<String> {
    parse_static_scss_invert_value_with_name(value, "invert")
}

fn parse_static_scss_color_invert_value(value: &str) -> Option<String> {
    parse_static_scss_invert_value_with_name(value, "color.invert")
}

fn parse_static_scss_invert_value_with_name(value: &str, function_name: &str) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, function_name)?;
    let arguments = parse_static_scss_color_invert_arguments(arguments.as_slice())?;
    let color = parse_static_scss_srgb_color_argument(
        reduce_static_scss_value(arguments.color.to_string()).as_str(),
    )?;
    let weight = match arguments.weight {
        Some(weight) => parse_static_scss_invert_weight(weight)?,
        None => 1.0,
    };
    Some(render_static_scss_sass_color_constructor(
        StaticScssSrgbColorValue {
            red: color
                .red
                .mul_add(1.0 - weight, (255.0 - color.red) * weight),
            green: color
                .green
                .mul_add(1.0 - weight, (255.0 - color.green) * weight),
            blue: color
                .blue
                .mul_add(1.0 - weight, (255.0 - color.blue) * weight),
            alpha: color.alpha,
        },
        color.alpha,
    ))
}

fn parse_static_scss_color_change_value(value: &str, function_name: &str) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, function_name)?;
    let arguments = parse_static_scss_color_named_channel_arguments(arguments.as_slice())?;
    if !arguments.has_any_adjustment() {
        return None;
    }
    let color = parse_static_scss_srgb_color_argument(
        reduce_static_scss_value(arguments.color.to_string()).as_str(),
    )?;
    let mut hsl = StaticScssHslColorValue::from_srgb(color);
    if let Some(hue) = arguments.hue {
        hsl.hue = parse_static_scss_hue_degrees(hue)?.rem_euclid(360.0);
    }
    if let Some(saturation) = arguments.saturation {
        hsl.saturation = parse_static_scss_percent_adjustment(saturation)?.clamp(0.0, 1.0);
    }
    if let Some(lightness) = arguments.lightness {
        hsl.lightness = parse_static_scss_percent_adjustment(lightness)?.clamp(0.0, 1.0);
    }
    if let Some(alpha) = arguments.alpha {
        hsl.alpha = parse_static_scss_alpha_channel(alpha)?;
    }
    Some(render_static_scss_sass_color_constructor(
        hsl.to_srgb(),
        hsl.alpha,
    ))
}

fn parse_static_scss_color_scale_value(value: &str, function_name: &str) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, function_name)?;
    let arguments = parse_static_scss_color_named_channel_arguments(arguments.as_slice())?;
    if arguments.hue.is_some() || !arguments.has_any_adjustment() {
        return None;
    }
    let color = parse_static_scss_srgb_color_argument(
        reduce_static_scss_value(arguments.color.to_string()).as_str(),
    )?;
    let mut hsl = StaticScssHslColorValue::from_srgb(color);
    if let Some(saturation) = arguments.saturation {
        hsl.saturation = static_scss_scale_unit_interval(
            hsl.saturation,
            parse_static_scss_percent_adjustment(saturation)?,
        );
    }
    if let Some(lightness) = arguments.lightness {
        hsl.lightness = static_scss_scale_unit_interval(
            hsl.lightness,
            parse_static_scss_percent_adjustment(lightness)?,
        );
    }
    if let Some(alpha) = arguments.alpha {
        hsl.alpha = static_scss_scale_unit_interval(
            hsl.alpha,
            parse_static_scss_alpha_scale_percent(alpha)?,
        );
    }
    Some(render_static_scss_sass_color_constructor(
        hsl.to_srgb(),
        hsl.alpha,
    ))
}

struct StaticScssColorAlphaAmountArguments<'a> {
    color: &'a str,
    amount: &'a str,
}

fn parse_static_scss_color_alpha_amount_arguments(
    arguments: &[String],
) -> Option<StaticScssColorAlphaAmountArguments<'_>> {
    let mut positional = Vec::<&str>::new();
    let mut color = None;
    let mut amount = None;

    for argument in arguments {
        match static_scss_named_argument(argument)? {
            Some(("color", value)) => color = Some(value),
            Some(("amount", value)) => amount = Some(value),
            Some(_) => return None,
            None => positional.push(argument.as_str()),
        }
    }

    if color.is_none()
        && let Some(value) = positional.first()
    {
        color = Some(*value);
    }
    if amount.is_none()
        && let Some(value) = positional.get(1)
    {
        amount = Some(*value);
    }
    (positional.len() <= 2).then_some(StaticScssColorAlphaAmountArguments {
        color: color?,
        amount: amount?,
    })
}

struct StaticScssColorHueAmountArguments<'a> {
    color: &'a str,
    amount: &'a str,
}

fn parse_static_scss_color_hue_amount_arguments(
    arguments: &[String],
) -> Option<StaticScssColorHueAmountArguments<'_>> {
    let mut positional = Vec::<&str>::new();
    let mut color = None;
    let mut amount = None;

    for argument in arguments {
        match static_scss_named_argument(argument)? {
            Some(("color", value)) => color = Some(value),
            Some(("degrees", value)) => amount = Some(value),
            Some(_) => return None,
            None => positional.push(argument.as_str()),
        }
    }

    if color.is_none()
        && let Some(value) = positional.first()
    {
        color = Some(*value);
    }
    if amount.is_none()
        && let Some(value) = positional.get(1)
    {
        amount = Some(*value);
    }
    (positional.len() <= 2).then_some(StaticScssColorHueAmountArguments {
        color: color?,
        amount: amount?,
    })
}

struct StaticScssColorNamedChannelArguments<'a> {
    color: &'a str,
    hue: Option<&'a str>,
    saturation: Option<&'a str>,
    lightness: Option<&'a str>,
    alpha: Option<&'a str>,
}

impl StaticScssColorNamedChannelArguments<'_> {
    fn has_any_adjustment(&self) -> bool {
        self.hue.is_some()
            || self.saturation.is_some()
            || self.lightness.is_some()
            || self.alpha.is_some()
    }
}

fn parse_static_scss_color_named_channel_arguments(
    arguments: &[String],
) -> Option<StaticScssColorNamedChannelArguments<'_>> {
    let mut positional = Vec::<&str>::new();
    let mut color = None;
    let mut hue = None;
    let mut saturation = None;
    let mut lightness = None;
    let mut alpha = None;

    for argument in arguments {
        match static_scss_named_argument(argument)? {
            Some(("color", value)) => color = Some(value),
            Some(("hue", value)) => hue = Some(value),
            Some(("saturation", value)) => saturation = Some(value),
            Some(("lightness", value)) => lightness = Some(value),
            Some(("alpha", value)) => alpha = Some(value),
            Some(_) => return None,
            None => positional.push(argument.as_str()),
        }
    }

    if color.is_none()
        && let Some(value) = positional.first()
    {
        color = Some(*value);
    }
    (positional.len() <= 1).then_some(StaticScssColorNamedChannelArguments {
        color: color?,
        hue,
        saturation,
        lightness,
        alpha,
    })
}

struct StaticScssColorInvertArguments<'a> {
    color: &'a str,
    weight: Option<&'a str>,
}

fn parse_static_scss_color_invert_arguments(
    arguments: &[String],
) -> Option<StaticScssColorInvertArguments<'_>> {
    let mut positional = Vec::<&str>::new();
    let mut color = None;
    let mut weight = None;

    for argument in arguments {
        match static_scss_named_argument(argument)? {
            Some(("color", value)) => color = Some(value),
            Some(("weight", value)) => weight = Some(value),
            Some(_) => return None,
            None => positional.push(argument.as_str()),
        }
    }

    if color.is_none()
        && let Some(value) = positional.first()
    {
        color = Some(*value);
    }
    if weight.is_none()
        && let Some(value) = positional.get(1)
    {
        weight = Some(*value);
    }
    (positional.len() <= 2).then_some(StaticScssColorInvertArguments {
        color: color?,
        weight,
    })
}

fn parse_static_scss_single_color_function_argument(
    value: &str,
    function_name: &str,
) -> Option<StaticScssSrgbColorValue> {
    let arguments = parse_whole_function_value_arguments(value, function_name)?;
    let mut positional = Vec::<&str>::new();
    let mut color = None;
    for argument in &arguments {
        match static_scss_named_argument(argument.as_str())? {
            Some(("color", value)) => color = Some(value),
            Some(_) => return None,
            None => positional.push(argument.as_str()),
        }
    }
    if color.is_none()
        && let Some(value) = positional.first()
    {
        color = Some(*value);
    }
    let color = color?;
    (positional.len() <= 1).then(|| {
        parse_static_scss_srgb_color_argument(reduce_static_scss_value(color.to_string()).as_str())
    })?
}

fn parse_static_scss_alpha_adjustment(value: &str) -> Option<f64> {
    let value = parse_static_scss_plain_f64(value.trim())?;
    value
        .is_finite()
        .then_some(value)
        .filter(|value| (-1.0..=1.0).contains(value))
}

fn parse_static_scss_unitless_alpha_amount(value: &str) -> Option<f64> {
    let value = parse_static_scss_plain_f64(value.trim())?;
    value
        .is_finite()
        .then_some(value)
        .filter(|value| (0.0..=1.0).contains(value))
}

fn parse_static_scss_alpha_scale_percent(value: &str) -> Option<f64> {
    let percent = value.trim().strip_suffix('%')?;
    let value = parse_static_scss_plain_f64(percent)? / 100.0;
    value
        .is_finite()
        .then_some(value)
        .filter(|value| (-1.0..=1.0).contains(value))
}

fn parse_static_scss_hue_degrees(value: &str) -> Option<f64> {
    let value = value.trim();
    let value = value.strip_suffix("deg").unwrap_or(value);
    let value = parse_static_scss_plain_f64(value.trim())?;
    value.is_finite().then_some(value)
}

fn parse_static_scss_percent_adjustment(value: &str) -> Option<f64> {
    let value = value.trim();
    let value = value.strip_suffix('%').unwrap_or(value);
    let value = parse_static_scss_plain_f64(value.trim())? / 100.0;
    value
        .is_finite()
        .then_some(value)
        .filter(|value| (-1.0..=1.0).contains(value))
}

fn parse_static_scss_invert_weight(value: &str) -> Option<f64> {
    parse_static_scss_percent_adjustment(value).filter(|value| (0.0..=1.0).contains(value))
}

fn static_scss_clamp_alpha(value: f64) -> f64 {
    value.clamp(0.0, 1.0)
}

fn static_scss_scale_unit_interval(value: f64, scale: f64) -> f64 {
    if scale >= 0.0 {
        value + (1.0 - value) * scale
    } else {
        value + value * scale
    }
    .clamp(0.0, 1.0)
}

fn render_static_scss_sass_color_constructor(
    color: StaticScssSrgbColorValue,
    alpha: f64,
) -> String {
    if (alpha - 1.0).abs() <= f64::EPSILON {
        if let Some(color) = static_scss_integral_srgb_color(color) {
            return shortest_static_srgb_color_with_alpha_text(StaticSrgbColorWithAlpha {
                color,
                alpha: None,
            });
        }
        return format!(
            "rgb({}, {}, {})",
            format_css_number(color.red),
            format_css_number(color.green),
            format_css_number(color.blue)
        );
    }
    format!(
        "rgba({}, {}, {}, {})",
        format_css_number(color.red),
        format_css_number(color.green),
        format_css_number(color.blue),
        format_css_number(alpha)
    )
}

fn static_scss_integral_srgb_color(color: StaticScssSrgbColorValue) -> Option<SrgbColor> {
    Some(SrgbColor {
        red: static_scss_u8_color_channel(color.red)?,
        green: static_scss_u8_color_channel(color.green)?,
        blue: static_scss_u8_color_channel(color.blue)?,
    })
}

fn static_scss_u8_color_channel(value: f64) -> Option<u8> {
    if !value.is_finite() || (value.round() - value).abs() > 1e-9 {
        return None;
    }
    let value = value.round();
    (0.0..=255.0).contains(&value).then_some(value as u8)
}

fn parse_static_scss_ie_hex_str_value(value: &str) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, "ie-hex-str")?;
    let [color] = arguments.as_slice() else {
        return None;
    };
    let color = parse_static_scss_srgb_color_argument(
        reduce_static_scss_value(color.to_string()).as_str(),
    )?;
    let red = static_scss_u8_color_channel(color.red)?;
    let green = static_scss_u8_color_channel(color.green)?;
    let blue = static_scss_u8_color_channel(color.blue)?;
    let alpha = static_scss_ie_hex_alpha_channel(color.alpha)?;
    Some(format!("#{alpha:02X}{red:02X}{green:02X}{blue:02X}"))
}

fn static_scss_ie_hex_alpha_channel(alpha: f64) -> Option<u8> {
    if !alpha.is_finite() {
        return None;
    }
    let channel = (static_scss_clamp_alpha(alpha) * 255.0).round();
    (0.0..=255.0).contains(&channel).then_some(channel as u8)
}

fn parse_static_scss_color_mix_value(value: &str) -> Option<String> {
    parse_static_scss_color_mix_value_with_name(value, "color.mix")
}

fn parse_static_scss_global_mix_value(value: &str) -> Option<String> {
    parse_static_scss_color_mix_value_with_name(value, "mix")
}

fn parse_static_scss_color_mix_value_with_name(value: &str, function_name: &str) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, function_name)?;
    let arguments = parse_static_scss_color_mix_arguments(arguments.as_slice())?;
    let first = parse_static_scss_srgb_color_argument(
        reduce_static_scss_value(arguments.first_color.to_string()).as_str(),
    )?;
    let second = parse_static_scss_srgb_color_argument(
        reduce_static_scss_value(arguments.second_color.to_string()).as_str(),
    )?;
    let weight = match arguments.weight {
        Some(weight) => parse_static_scss_color_mix_weight(weight)?,
        None => 50.0,
    };
    let mix = static_scss_color_mix_weights(weight, first.alpha, second.alpha)?;
    let alpha = first
        .alpha
        .mul_add(mix.alpha_first, second.alpha * mix.alpha_second);
    Some(render_static_scss_sass_color_constructor(
        StaticScssSrgbColorValue {
            red: first.red.mul_add(mix.first, second.red * mix.second),
            green: first.green.mul_add(mix.first, second.green * mix.second),
            blue: first.blue.mul_add(mix.first, second.blue * mix.second),
            alpha,
        },
        alpha,
    ))
}

struct StaticScssColorMixWeights {
    first: f64,
    second: f64,
    alpha_first: f64,
    alpha_second: f64,
}

fn static_scss_color_mix_weights(
    weight_percent: f64,
    first_alpha: f64,
    second_alpha: f64,
) -> Option<StaticScssColorMixWeights> {
    let alpha_first = weight_percent / 100.0;
    let alpha_second = 1.0 - alpha_first;
    let scaled_weight = alpha_first * 2.0 - 1.0;
    let alpha_delta = first_alpha - second_alpha;
    let channel_weight = if (scaled_weight * alpha_delta + 1.0).abs() <= f64::EPSILON {
        scaled_weight
    } else {
        (scaled_weight + alpha_delta) / (1.0 + scaled_weight * alpha_delta)
    };
    let first = (channel_weight + 1.0) / 2.0;
    let second = 1.0 - first;
    [first, second, alpha_first, alpha_second]
        .into_iter()
        .all(f64::is_finite)
        .then_some(StaticScssColorMixWeights {
            first,
            second,
            alpha_first,
            alpha_second,
        })
}

struct StaticScssColorMixArguments<'a> {
    first_color: &'a str,
    second_color: &'a str,
    weight: Option<&'a str>,
}

fn parse_static_scss_color_mix_arguments(
    arguments: &[String],
) -> Option<StaticScssColorMixArguments<'_>> {
    let mut positional = Vec::<&str>::new();
    let mut first_color = None;
    let mut second_color = None;
    let mut weight = None;

    for argument in arguments {
        match static_scss_named_argument(argument)? {
            Some(("color1", value)) => first_color = Some(value),
            Some(("color2", value)) => second_color = Some(value),
            Some(("weight", value)) => weight = Some(value),
            Some(_) => return None,
            None => positional.push(argument.as_str()),
        }
    }

    if first_color.is_none()
        && let Some(value) = positional.first()
    {
        first_color = Some(*value);
    }
    if second_color.is_none()
        && let Some(value) = positional.get(1)
    {
        second_color = Some(*value);
    }
    if weight.is_none()
        && let Some(value) = positional.get(2)
    {
        weight = Some(*value);
    }
    (positional.len() <= 3).then_some(StaticScssColorMixArguments {
        first_color: first_color?,
        second_color: second_color?,
        weight,
    })
}

fn parse_static_scss_color_mix_weight(value: &str) -> Option<f64> {
    let percent = value.trim().strip_suffix('%')?;
    let weight = percent.trim().parse::<f64>().ok()?;
    weight
        .is_finite()
        .then_some(weight)
        .filter(|weight| (0.0..=100.0).contains(weight))
}

fn parse_static_scss_color_channel_value(value: &str) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, "color.channel")?;
    let arguments = parse_static_scss_color_channel_arguments(arguments.as_slice())?;
    let space = match arguments.space {
        Some(space) => parse_static_scss_color_channel_space(space)?,
        None => StaticScssColorSpace::Rgb,
    };
    let color_text = reduce_static_scss_value(arguments.color.to_string());
    let color = parse_static_scss_srgb_color_argument(color_text.as_str())?;
    let channel = parse_static_scss_color_channel_name(arguments.channel, space)?;
    Some(render_static_scss_color_channel(color, channel))
}

fn parse_static_scss_color_red_value(value: &str) -> Option<String> {
    parse_static_scss_legacy_color_channel_value(value, "color.red", StaticScssColorChannel::Red)
}

fn parse_static_scss_global_red_value(value: &str) -> Option<String> {
    parse_static_scss_legacy_color_channel_value(value, "red", StaticScssColorChannel::Red)
}

fn parse_static_scss_color_green_value(value: &str) -> Option<String> {
    parse_static_scss_legacy_color_channel_value(
        value,
        "color.green",
        StaticScssColorChannel::Green,
    )
}

fn parse_static_scss_global_green_value(value: &str) -> Option<String> {
    parse_static_scss_legacy_color_channel_value(value, "green", StaticScssColorChannel::Green)
}

fn parse_static_scss_color_blue_value(value: &str) -> Option<String> {
    parse_static_scss_legacy_color_channel_value(value, "color.blue", StaticScssColorChannel::Blue)
}

fn parse_static_scss_global_blue_value(value: &str) -> Option<String> {
    parse_static_scss_legacy_color_channel_value(value, "blue", StaticScssColorChannel::Blue)
}

fn parse_static_scss_color_alpha_value(value: &str) -> Option<String> {
    parse_static_scss_legacy_color_channel_value(
        value,
        "color.alpha",
        StaticScssColorChannel::Alpha,
    )
}

fn parse_static_scss_global_alpha_value(value: &str) -> Option<String> {
    parse_static_scss_legacy_color_channel_value(value, "alpha", StaticScssColorChannel::Alpha)
}

fn parse_static_scss_legacy_color_channel_value(
    value: &str,
    function_name: &str,
    channel: StaticScssColorChannel,
) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, function_name)?;
    let [color] = arguments.as_slice() else {
        return None;
    };
    Some(render_static_scss_color_channel(
        parse_static_scss_srgb_color_argument(
            reduce_static_scss_value(color.to_string()).as_str(),
        )?,
        channel,
    ))
}

struct StaticScssColorChannelArguments<'a> {
    color: &'a str,
    channel: &'a str,
    space: Option<&'a str>,
}

fn parse_static_scss_color_channel_arguments(
    arguments: &[String],
) -> Option<StaticScssColorChannelArguments<'_>> {
    let mut positional = Vec::<&str>::new();
    let mut color = None;
    let mut channel = None;
    let mut space = None;

    for argument in arguments {
        match static_scss_named_argument(argument)? {
            Some(("color", value)) => color = Some(value),
            Some(("channel", value)) => channel = Some(value),
            Some(("space", value)) => space = Some(value),
            Some(_) => return None,
            None => positional.push(argument.as_str()),
        }
    }

    if color.is_none()
        && let Some(value) = positional.first()
    {
        color = Some(*value);
    }
    if channel.is_none()
        && let Some(value) = positional.get(1)
    {
        channel = Some(*value);
    }
    if space.is_none()
        && let Some(value) = positional.get(2)
    {
        space = Some(*value);
    }
    (positional.len() <= 3).then_some(StaticScssColorChannelArguments {
        color: color?,
        channel: channel?,
        space,
    })
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct StaticScssSrgbColorValue {
    red: f64,
    green: f64,
    blue: f64,
    alpha: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct StaticScssHslColorValue {
    hue: f64,
    saturation: f64,
    lightness: f64,
    alpha: f64,
}

impl StaticScssHslColorValue {
    fn from_srgb(color: StaticScssSrgbColorValue) -> Self {
        Self {
            hue: static_scss_srgb_hue(color),
            saturation: static_scss_srgb_saturation(color),
            lightness: static_scss_srgb_lightness(color),
            alpha: color.alpha,
        }
    }

    fn to_srgb(self) -> StaticScssSrgbColorValue {
        let hue = (self.hue / 60.0).rem_euclid(6.0);
        let saturation = self.saturation.clamp(0.0, 1.0);
        let lightness = self.lightness.clamp(0.0, 1.0);
        let chroma = (1.0 - (2.0 * lightness - 1.0).abs()) * saturation;
        let secondary = chroma * (1.0 - (hue.rem_euclid(2.0) - 1.0).abs());
        let (red1, green1, blue1) = if hue < 1.0 {
            (chroma, secondary, 0.0)
        } else if hue < 2.0 {
            (secondary, chroma, 0.0)
        } else if hue < 3.0 {
            (0.0, chroma, secondary)
        } else if hue < 4.0 {
            (0.0, secondary, chroma)
        } else if hue < 5.0 {
            (secondary, 0.0, chroma)
        } else {
            (chroma, 0.0, secondary)
        };
        let match_value = lightness - chroma / 2.0;
        StaticScssSrgbColorValue {
            red: (red1 + match_value) * 255.0,
            green: (green1 + match_value) * 255.0,
            blue: (blue1 + match_value) * 255.0,
            alpha: self.alpha,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StaticScssColorChannel {
    Red,
    Green,
    Blue,
    Alpha,
    Hue,
    Saturation,
    Lightness,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StaticScssColorSpace {
    Rgb,
    Hsl,
}

fn parse_static_scss_color_channel_space(value: &str) -> Option<StaticScssColorSpace> {
    match value.trim().to_ascii_lowercase().as_str() {
        "rgb" => Some(StaticScssColorSpace::Rgb),
        "hsl" => Some(StaticScssColorSpace::Hsl),
        _ => None,
    }
}

fn parse_static_scss_color_channel_name(
    value: &str,
    space: StaticScssColorSpace,
) -> Option<StaticScssColorChannel> {
    let channel = parse_static_scss_string_argument(value)?;
    if !channel.quoted {
        return None;
    }
    match (space, channel.text.to_ascii_lowercase().as_str()) {
        (StaticScssColorSpace::Rgb, "red") => Some(StaticScssColorChannel::Red),
        (StaticScssColorSpace::Rgb, "green") => Some(StaticScssColorChannel::Green),
        (StaticScssColorSpace::Rgb, "blue") => Some(StaticScssColorChannel::Blue),
        (StaticScssColorSpace::Rgb, "alpha") => Some(StaticScssColorChannel::Alpha),
        (StaticScssColorSpace::Hsl, "hue") => Some(StaticScssColorChannel::Hue),
        (StaticScssColorSpace::Hsl, "saturation") => Some(StaticScssColorChannel::Saturation),
        (StaticScssColorSpace::Hsl, "lightness") => Some(StaticScssColorChannel::Lightness),
        _ => None,
    }
}

fn render_static_scss_color_channel(
    color: StaticScssSrgbColorValue,
    channel: StaticScssColorChannel,
) -> String {
    match channel {
        StaticScssColorChannel::Red => format_css_number(color.red),
        StaticScssColorChannel::Green => format_css_number(color.green),
        StaticScssColorChannel::Blue => format_css_number(color.blue),
        StaticScssColorChannel::Alpha => format_css_number(color.alpha),
        StaticScssColorChannel::Hue => {
            format!("{}deg", format_css_number(static_scss_srgb_hue(color)))
        }
        StaticScssColorChannel::Saturation => {
            format!(
                "{}%",
                format_css_number(static_scss_srgb_saturation(color) * 100.0)
            )
        }
        StaticScssColorChannel::Lightness => {
            format!(
                "{}%",
                format_css_number(static_scss_srgb_lightness(color) * 100.0)
            )
        }
    }
}

fn static_scss_srgb_hue(color: StaticScssSrgbColorValue) -> f64 {
    let (red, green, blue) = static_scss_normalized_rgb(color);
    let max = red.max(green).max(blue);
    let min = red.min(green).min(blue);
    let delta = max - min;
    if delta.abs() <= f64::EPSILON {
        return 0.0;
    }
    let hue_sector = if (max - red).abs() <= f64::EPSILON {
        ((green - blue) / delta).rem_euclid(6.0)
    } else if (max - green).abs() <= f64::EPSILON {
        (blue - red) / delta + 2.0
    } else {
        (red - green) / delta + 4.0
    };
    (hue_sector * 60.0).rem_euclid(360.0)
}

fn static_scss_srgb_saturation(color: StaticScssSrgbColorValue) -> f64 {
    let (red, green, blue) = static_scss_normalized_rgb(color);
    let max = red.max(green).max(blue);
    let min = red.min(green).min(blue);
    let delta = max - min;
    if delta.abs() <= f64::EPSILON {
        return 0.0;
    }
    let lightness = (max + min) / 2.0;
    delta / (1.0 - (2.0 * lightness - 1.0).abs())
}

fn static_scss_srgb_lightness(color: StaticScssSrgbColorValue) -> f64 {
    let (red, green, blue) = static_scss_normalized_rgb(color);
    (red.max(green).max(blue) + red.min(green).min(blue)) / 2.0
}

fn static_scss_normalized_rgb(color: StaticScssSrgbColorValue) -> (f64, f64, f64) {
    (color.red / 255.0, color.green / 255.0, color.blue / 255.0)
}

fn parse_static_scss_srgb_color_argument(value: &str) -> Option<StaticScssSrgbColorValue> {
    let value = value.trim();
    parse_static_scss_rgb_function_color(value)
        .or_else(|| parse_static_scss_basic_srgb_color(value))
        .or_else(|| {
            parse_color_function_value(value)
                .and_then(|value| parse_static_scss_rgb_function_color(value.as_str()))
        })
        .or_else(|| {
            parse_color_mix_value(value)
                .and_then(|value| parse_static_scss_rgb_function_color(value.as_str()))
        })
        .or_else(|| {
            parse_oklab_oklch_value(value)
                .and_then(|value| parse_static_scss_rgb_function_color(value.as_str()))
        })
}

fn parse_static_scss_basic_srgb_color(value: &str) -> Option<StaticScssSrgbColorValue> {
    parse_static_srgb_color_with_alpha(value)
        .or_else(|| parse_static_hsl_function_color_with_alpha(value))
        .or_else(|| parse_static_hwb_function_color_with_alpha(value))
        .map(static_srgb_with_alpha_to_scss_value)
}

fn static_srgb_with_alpha_to_scss_value(
    color: StaticSrgbColorWithAlpha,
) -> StaticScssSrgbColorValue {
    StaticScssSrgbColorValue {
        red: f64::from(color.color.red),
        green: f64::from(color.color.green),
        blue: f64::from(color.color.blue),
        alpha: color.alpha.unwrap_or(1.0),
    }
}

fn parse_static_scss_rgb_function_color(value: &str) -> Option<StaticScssSrgbColorValue> {
    let inner = parse_whole_function_value_inner(value, "rgb")
        .or_else(|| parse_whole_function_value_inner(value, "rgba"))?;
    let (channels, alpha) = split_static_scss_rgb_channels_with_optional_alpha(inner)?;
    let [red, green, blue] = channels.as_slice() else {
        return None;
    };
    Some(StaticScssSrgbColorValue {
        red: parse_static_scss_rgb_channel(red)?,
        green: parse_static_scss_rgb_channel(green)?,
        blue: parse_static_scss_rgb_channel(blue)?,
        alpha: match alpha {
            Some(value) => parse_static_scss_alpha_channel(value.as_str())?,
            None => 1.0,
        },
    })
}

fn split_static_scss_rgb_channels_with_optional_alpha(
    inner: &str,
) -> Option<(Vec<String>, Option<String>)> {
    if inner.contains(',') {
        let arguments = split_top_level_value_arguments_owned(inner)?;
        return match arguments.as_slice() {
            [red, green, blue] => Some((vec![red.clone(), green.clone(), blue.clone()], None)),
            [red, green, blue, alpha] => Some((
                vec![red.clone(), green.clone(), blue.clone()],
                Some(alpha.clone()),
            )),
            _ => None,
        };
    }

    let arguments = split_top_level_whitespace_value_components_owned(inner)?;
    match arguments.as_slice() {
        [red, green, blue] => Some((vec![red.clone(), green.clone(), blue.clone()], None)),
        [red, green, blue, slash, alpha] if slash == "/" => Some((
            vec![red.clone(), green.clone(), blue.clone()],
            Some(alpha.clone()),
        )),
        _ => None,
    }
}

fn parse_static_scss_rgb_channel(value: &str) -> Option<f64> {
    let value = if let Some(percent) = value.trim().strip_suffix('%') {
        parse_static_scss_plain_f64(percent)? * 255.0 / 100.0
    } else {
        parse_static_scss_plain_f64(value.trim())?
    };
    value
        .is_finite()
        .then_some(value)
        .filter(|value| (0.0..=255.0).contains(value))
}

fn parse_static_scss_alpha_channel(value: &str) -> Option<f64> {
    let value = if let Some(percent) = value.trim().strip_suffix('%') {
        parse_static_scss_plain_f64(percent)? / 100.0
    } else {
        parse_static_scss_plain_f64(value.trim())?
    };
    value
        .is_finite()
        .then_some(value)
        .filter(|value| (0.0..=1.0).contains(value))
}

fn parse_static_scss_plain_f64(value: &str) -> Option<f64> {
    if value.contains('%') {
        return None;
    }
    value.parse::<f64>().ok().filter(|value| value.is_finite())
}

fn parse_static_scss_list_index(value: &str) -> Option<isize> {
    let reduced = reduce_static_numeric_value(value.trim().to_string());
    let index = reduced.trim().parse::<isize>().ok()?;
    (index != 0).then_some(index)
}

fn static_scss_value_type(value: &str) -> Option<&'static str> {
    let value = value.trim();
    if value.is_empty() || value.contains('$') {
        return None;
    }
    let reduced = reduce_static_scss_value(value.to_string());
    let value = reduced.trim();
    let normalized = value.to_ascii_lowercase();
    match normalized.as_str() {
        "true" | "false" => return Some("bool"),
        "null" => return Some("null"),
        _ => {}
    }
    if static_scss_quoted_string_text(value).is_some() {
        return Some("string");
    }
    if static_scss_non_empty_map_value_is_static(value) {
        return Some("map");
    }
    if static_scss_value_is_list(value) {
        return Some("list");
    }
    if static_scss_value_is_color(value) {
        return Some("color");
    }
    if parse_numeric_value_with_unit(value).is_some() {
        return Some("number");
    }
    if static_scss_value_is_calculation(value) {
        return Some("calculation");
    }
    static_scss_collection_member_is_static(value).then_some("string")
}

fn static_scss_value_is_color(value: &str) -> bool {
    parse_static_srgb_color_with_alpha(value).is_some()
        || parse_static_rgb_function_color_with_alpha(value).is_some()
        || parse_static_hsl_function_color_with_alpha(value).is_some()
        || parse_static_hwb_function_color_with_alpha(value).is_some()
        || parse_color_function_value(value).is_some()
        || parse_color_mix_value(value).is_some()
        || parse_oklab_oklch_value(value).is_some()
}

fn static_scss_value_is_calculation(value: &str) -> bool {
    static_scss_calculation_name(value).is_some()
}

fn static_scss_calculation_name(value: &str) -> Option<&'static str> {
    ["calc", "clamp", "min", "max"]
        .into_iter()
        .find(|name| omena_value_lattice::parse_whole_function_value_inner(value, name).is_some())
}

fn static_scss_calculation_args(value: &str) -> Option<String> {
    let calculation_name = static_scss_calculation_name(value)?;
    let inner = omena_value_lattice::parse_whole_function_value_inner(value, calculation_name)?;
    let items = match calculation_name {
        "calc" => return None,
        "clamp" | "min" | "max" => split_static_scss_top_level(inner, ',')?,
        _ => return None,
    };
    if items.len() < 2
        || !items
            .iter()
            .all(|item| static_scss_calculation_arg_is_static(item))
    {
        return None;
    }
    static_scss_render_list_value(&StaticScssListValue {
        items,
        separator: StaticScssListSeparator::Comma,
        bracketed: false,
    })
}

fn static_scss_calculation_arg_is_static(value: &str) -> bool {
    let value = value.trim();
    !value.is_empty()
        && !value.contains('$')
        && static_scss_top_level_separator_index(value, ':').is_some_and(|index| index.is_none())
}

fn static_scss_feature_exists(feature: &str) -> bool {
    matches!(
        feature,
        "global-variable-shadowing"
            | "extend-selector-pseudoclass"
            | "units-level3"
            | "at-error"
            | "custom-property"
    )
}

fn static_scss_non_empty_map_value_is_static(value: &str) -> bool {
    parse_static_scss_map_entries(value).is_some_and(|entries| !entries.is_empty())
}

fn static_scss_value_is_list(value: &str) -> bool {
    let value = value.trim();
    if value == "()" {
        return true;
    }
    if value.starts_with('[') && strip_static_scss_outer_container(value).is_some() {
        return true;
    }
    let source = strip_static_scss_outer_container(value).unwrap_or(value);
    split_static_scss_top_level(source, ',').is_some_and(|items| items.len() > 1)
        || split_static_scss_top_level(source, '/').is_some_and(|items| items.len() > 1)
        || split_static_scss_top_level_whitespace(source).is_some_and(|items| items.len() > 1)
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct StaticScssListValue {
    items: Vec<String>,
    separator: StaticScssListSeparator,
    bracketed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StaticScssListSeparator {
    Space,
    Comma,
    Slash,
}

fn parse_static_scss_list_items(value: &str) -> Option<Vec<String>> {
    Some(parse_static_scss_list_value(value)?.items)
}

fn parse_static_scss_list_value(value: &str) -> Option<StaticScssListValue> {
    let value = value.trim();
    let bracketed = value.starts_with('[') && strip_static_scss_outer_container(value).is_some();
    let source = strip_static_scss_outer_container(value).unwrap_or(value);
    if source.is_empty() {
        return Some(StaticScssListValue {
            items: Vec::new(),
            separator: StaticScssListSeparator::Space,
            bracketed,
        });
    }
    let (items, separator) = match split_static_scss_top_level(source, ',') {
        Some(items) if items.len() > 1 => (items, StaticScssListSeparator::Comma),
        _ => match split_static_scss_top_level(source, '/') {
            Some(items) if items.len() > 1 => (items, StaticScssListSeparator::Slash),
            _ => (
                split_static_scss_top_level_whitespace(source)?,
                StaticScssListSeparator::Space,
            ),
        },
    };
    if items.is_empty()
        || items
            .iter()
            .any(|item| !static_scss_collection_member_is_static(item))
    {
        return None;
    }
    Some(StaticScssListValue {
        items,
        separator,
        bracketed,
    })
}

fn static_scss_resolved_list_index(index: isize, len: usize) -> Option<usize> {
    if index > 0 {
        Some(index.checked_sub(1)? as usize).filter(|index| *index < len)
    } else {
        len.checked_sub(index.unsigned_abs())
    }
}

fn static_scss_list_append_item_text(value: &str) -> Option<String> {
    let value = value.trim();
    if !static_scss_collection_member_is_static(value) {
        return None;
    }
    if strip_static_scss_outer_container(value).is_some() {
        return Some(value.to_string());
    }
    let list = parse_static_scss_list_value(value)?;
    if list.items.len() > 1 {
        Some(format!("({value})"))
    } else {
        Some(value.to_string())
    }
}

fn static_scss_render_list_value(list: &StaticScssListValue) -> Option<String> {
    if list.bracketed {
        return Some(format!("[{}]", static_scss_join_list_items(list)));
    }
    if list.items.is_empty() {
        return Some("()".to_string());
    }
    match list.separator {
        StaticScssListSeparator::Space => Some(static_scss_join_list_items(list)),
        StaticScssListSeparator::Comma => Some(format!("({})", static_scss_join_list_items(list))),
        StaticScssListSeparator::Slash => Some(static_scss_join_list_items(list)),
    }
}

fn static_scss_join_list_items(list: &StaticScssListValue) -> String {
    match list.separator {
        StaticScssListSeparator::Space => list.items.join(" "),
        StaticScssListSeparator::Comma => list.items.join(", "),
        StaticScssListSeparator::Slash => list.items.join(" / "),
    }
}

fn parse_static_scss_list_separator_option(
    value: &str,
    auto_separator: StaticScssListSeparator,
) -> Option<StaticScssListSeparator> {
    match strip_static_scss_quotes(value.trim())
        .unwrap_or_else(|| value.trim())
        .to_ascii_lowercase()
        .as_str()
    {
        "auto" => Some(auto_separator),
        "space" => Some(StaticScssListSeparator::Space),
        "comma" => Some(StaticScssListSeparator::Comma),
        "slash" => Some(StaticScssListSeparator::Slash),
        _ => None,
    }
}

fn parse_static_scss_list_bracketed_option(value: &str, auto_bracketed: bool) -> Option<bool> {
    let value = strip_static_scss_quotes(value.trim()).unwrap_or_else(|| value.trim());
    if value.eq_ignore_ascii_case("auto") {
        return Some(auto_bracketed);
    }
    static_scss_literal_truthiness(value)
}
