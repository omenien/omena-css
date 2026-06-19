use omena_value_lattice::{
    SrgbColor, StaticSrgbColorWithAlpha, css_values_canonically_equal,
    number::{
        format_css_number, parse_reducible_abs_value, parse_reducible_calc_value,
        parse_reducible_ceil_value, parse_reducible_clamp_value, parse_reducible_exp_value,
        parse_reducible_floor_value, parse_reducible_hypot_value, parse_reducible_log_value,
        parse_reducible_max_value, parse_reducible_min_value, parse_reducible_mod_value,
        parse_reducible_pow_value, parse_reducible_rem_value,
        parse_reducible_round_to_integer_value, parse_reducible_round_value,
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

pub(crate) fn reduce_static_numeric_value(value: String) -> String {
    let trimmed = value.trim();
    if let Some(reduced) = substitute_static_css_function_references_in_value_until_stable(
        trimmed,
        &[
            ("calc", parse_reducible_calc_value),
            ("min", parse_reducible_min_value),
            ("max", parse_reducible_max_value),
            ("clamp", parse_reducible_clamp_value),
            ("abs", parse_reducible_abs_value),
            ("sign", parse_reducible_sign_value),
            ("round", parse_reducible_round_value),
            ("mod", parse_reducible_mod_value),
            ("rem", parse_reducible_rem_value),
            ("hypot", parse_reducible_hypot_value),
            ("sqrt", parse_reducible_sqrt_value),
            ("pow", parse_reducible_pow_value),
            ("exp", parse_reducible_exp_value),
            ("log", parse_reducible_log_value),
        ],
    ) {
        return reduced;
    }
    if let Some(reduced) = reduce_static_numeric_expression(trimmed) {
        return reduced;
    }
    let Some(inner) = trimmed
        .strip_prefix('(')
        .and_then(|without_left| without_left.strip_suffix(')'))
    else {
        return value;
    };
    reduce_static_numeric_expression(inner.trim()).unwrap_or(value)
}

pub(crate) fn reduce_static_less_numeric_value(value: String) -> String {
    let trimmed = value.trim();
    // Less intentionally exposes a narrower math function surface than CSS/Sass.
    // Preserve unsupported CSS math calls byte-for-byte instead of over-reducing.
    if let Some(reduced) = substitute_static_css_function_references_in_value_until_stable(
        trimmed,
        &[
            ("min", parse_reducible_min_value),
            ("max", parse_reducible_max_value),
            ("abs", parse_reducible_abs_value),
            ("mod", parse_reducible_mod_value),
            ("sqrt", parse_reducible_sqrt_value),
            ("pow", parse_reducible_pow_value),
        ],
    ) {
        return reduced;
    }
    if let Some(reduced) = reduce_static_numeric_expression(trimmed) {
        return reduced;
    }
    let Some(inner) = trimmed
        .strip_prefix('(')
        .and_then(|without_left| without_left.strip_suffix(')'))
    else {
        return value;
    };
    reduce_static_numeric_expression(inner.trim()).unwrap_or(value)
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

fn parse_static_scss_quote_value(value: &str) -> Option<String> {
    parse_static_scss_quote_value_with_name(value, "quote")
}

fn parse_static_scss_string_quote_value(value: &str) -> Option<String> {
    parse_static_scss_quote_value_with_name(value, "string.quote")
}

fn parse_static_scss_unquote_value(value: &str) -> Option<String> {
    parse_static_scss_unquote_value_with_name(value, "unquote")
}

fn parse_static_scss_string_unquote_value(value: &str) -> Option<String> {
    parse_static_scss_unquote_value_with_name(value, "string.unquote")
}

fn parse_static_scss_str_length_value(value: &str) -> Option<String> {
    parse_static_scss_string_length_value_with_name(value, "str-length")
}

fn parse_static_scss_string_length_value(value: &str) -> Option<String> {
    parse_static_scss_string_length_value_with_name(value, "string.length")
}

fn parse_static_scss_str_index_value(value: &str) -> Option<String> {
    parse_static_scss_string_index_value_with_name(value, "str-index")
}

fn parse_static_scss_string_index_value(value: &str) -> Option<String> {
    parse_static_scss_string_index_value_with_name(value, "string.index")
}

fn parse_static_scss_str_insert_value(value: &str) -> Option<String> {
    parse_static_scss_string_insert_value_with_name(value, "str-insert")
}

fn parse_static_scss_string_insert_value(value: &str) -> Option<String> {
    parse_static_scss_string_insert_value_with_name(value, "string.insert")
}

fn parse_static_scss_str_slice_value(value: &str) -> Option<String> {
    parse_static_scss_string_slice_value_with_name(value, "str-slice")
}

fn parse_static_scss_string_slice_value(value: &str) -> Option<String> {
    parse_static_scss_string_slice_value_with_name(value, "string.slice")
}

fn parse_static_scss_to_upper_case_value(value: &str) -> Option<String> {
    parse_static_scss_string_case_value_with_name(
        value,
        "to-upper-case",
        StaticScssStringCase::Upper,
    )
}

fn parse_static_scss_string_to_upper_case_value(value: &str) -> Option<String> {
    parse_static_scss_string_case_value_with_name(
        value,
        "string.to-upper-case",
        StaticScssStringCase::Upper,
    )
}

fn parse_static_scss_to_lower_case_value(value: &str) -> Option<String> {
    parse_static_scss_string_case_value_with_name(
        value,
        "to-lower-case",
        StaticScssStringCase::Lower,
    )
}

fn parse_static_scss_string_to_lower_case_value(value: &str) -> Option<String> {
    parse_static_scss_string_case_value_with_name(
        value,
        "string.to-lower-case",
        StaticScssStringCase::Lower,
    )
}

fn parse_static_scss_quote_value_with_name(value: &str, function_name: &str) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, function_name)?;
    let [string] = arguments.as_slice() else {
        return None;
    };
    static_scss_quote_string(parse_static_scss_string_argument(string)?.text.as_str())
}

fn parse_static_scss_unquote_value_with_name(value: &str, function_name: &str) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, function_name)?;
    let [string] = arguments.as_slice() else {
        return None;
    };
    Some(parse_static_scss_string_argument(string)?.text)
}

fn parse_static_scss_string_length_value_with_name(
    value: &str,
    function_name: &str,
) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, function_name)?;
    let [string] = arguments.as_slice() else {
        return None;
    };
    Some(
        parse_static_scss_string_argument(string)?
            .text
            .chars()
            .count()
            .to_string(),
    )
}

fn parse_static_scss_string_index_value_with_name(
    value: &str,
    function_name: &str,
) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, function_name)?;
    let [string, substring] = arguments.as_slice() else {
        return None;
    };
    let string = parse_static_scss_string_argument(string)?;
    let substring = parse_static_scss_string_argument(substring)?;
    if substring.text.is_empty() {
        return None;
    }
    Some(
        string
            .text
            .find(substring.text.as_str())
            .and_then(|byte_index| {
                string
                    .text
                    .get(..byte_index)
                    .map(|prefix| prefix.chars().count() + 1)
            })
            .map(|index| index.to_string())
            .unwrap_or_else(|| "null".to_string()),
    )
}

fn parse_static_scss_string_insert_value_with_name(
    value: &str,
    function_name: &str,
) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, function_name)?;
    let [string, insert, index] = arguments.as_slice() else {
        return None;
    };
    let string = parse_static_scss_string_argument(string)?;
    let insert = parse_static_scss_string_argument(insert)?;
    let index = parse_static_scss_list_index(index)?;
    let chars = string.text.chars().collect::<Vec<_>>();
    let offset = static_scss_string_insert_offset(index, chars.len())?;
    let output = chars
        .iter()
        .take(offset)
        .copied()
        .chain(insert.text.chars())
        .chain(chars.iter().skip(offset).copied())
        .collect::<String>();
    static_scss_render_string_value(output.as_str(), string.quoted)
}

fn parse_static_scss_string_slice_value_with_name(
    value: &str,
    function_name: &str,
) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, function_name)?;
    let [string, start, end @ ..] = arguments.as_slice() else {
        return None;
    };
    if end.len() > 1 {
        return None;
    }
    let string = parse_static_scss_string_argument(string)?;
    let start = parse_static_scss_list_index(start)?;
    let end = match end {
        [] => -1,
        [end] => parse_static_scss_list_index(end)?,
        _ => return None,
    };
    let chars = string.text.chars().collect::<Vec<_>>();
    let start_offset = static_scss_string_slice_start_offset(start, chars.len())?;
    let end_offset = static_scss_string_slice_end_offset(end, chars.len())?;
    let output = if start_offset >= end_offset {
        String::new()
    } else {
        chars[start_offset..end_offset].iter().collect::<String>()
    };
    static_scss_render_string_value(output.as_str(), string.quoted)
}

fn parse_static_scss_string_case_value_with_name(
    value: &str,
    function_name: &str,
    case: StaticScssStringCase,
) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, function_name)?;
    let [string] = arguments.as_slice() else {
        return None;
    };
    let string = parse_static_scss_string_argument(string)?;
    let output = match case {
        StaticScssStringCase::Upper => string.text.to_ascii_uppercase(),
        StaticScssStringCase::Lower => string.text.to_ascii_lowercase(),
    };
    static_scss_render_string_value(output.as_str(), string.quoted)
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct StaticScssStringValue {
    text: String,
    quoted: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StaticScssStringCase {
    Upper,
    Lower,
}

fn parse_static_scss_string_argument(value: &str) -> Option<StaticScssStringValue> {
    let value = value.trim();
    if value.contains('$') {
        return None;
    }
    if let Some(text) = static_scss_quoted_string_text(value) {
        return Some(StaticScssStringValue {
            text: text.to_string(),
            quoted: true,
        });
    }
    if value.is_empty()
        || value.contains('(')
        || value.contains(')')
        || value.contains('[')
        || value.contains(']')
        || static_scss_top_level_separator_index(value, ',')?.is_some()
    {
        return None;
    }
    Some(StaticScssStringValue {
        text: value.to_string(),
        quoted: false,
    })
}

fn static_scss_quoted_string_text(value: &str) -> Option<&str> {
    let quote = value.chars().next()?;
    if !matches!(quote, '"' | '\'') || static_scss_quoted_value_end(value, 0, quote)? != value.len()
    {
        return None;
    }
    strip_static_scss_quotes(value)
}

fn static_scss_render_string_value(value: &str, quoted: bool) -> Option<String> {
    if quoted {
        static_scss_quote_string(value)
    } else {
        Some(value.to_string())
    }
}

fn static_scss_quote_string(value: &str) -> Option<String> {
    let mut output = String::with_capacity(value.len() + 2);
    output.push('"');
    for ch in value.chars() {
        match ch {
            '"' | '\\' => {
                output.push('\\');
                output.push(ch);
            }
            _ if ch.is_control() => return None,
            _ => output.push(ch),
        }
    }
    output.push('"');
    Some(output)
}

fn static_scss_string_insert_offset(index: isize, len: usize) -> Option<usize> {
    let len = isize::try_from(len).ok()?;
    Some(if index > 0 {
        (index - 1).clamp(0, len) as usize
    } else {
        (len + index + 1).clamp(0, len) as usize
    })
}

fn static_scss_string_slice_start_offset(index: isize, len: usize) -> Option<usize> {
    let len = isize::try_from(len).ok()?;
    Some(if index > 0 {
        (index - 1).clamp(0, len) as usize
    } else {
        (len + index).clamp(0, len) as usize
    })
}

fn static_scss_string_slice_end_offset(index: isize, len: usize) -> Option<usize> {
    let len = isize::try_from(len).ok()?;
    Some(if index > 0 {
        index.clamp(0, len) as usize
    } else {
        (len + index + 1).clamp(0, len) as usize
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

fn parse_static_scss_map_get_value(value: &str) -> Option<String> {
    parse_static_scss_map_get_value_with_name(value, "map-get")
}

fn parse_static_scss_map_get_namespaced_value(value: &str) -> Option<String> {
    parse_static_scss_map_get_value_with_name(value, "map.get")
}

fn parse_static_scss_map_has_key_value(value: &str) -> Option<String> {
    parse_static_scss_map_has_key_value_with_name(value, "map-has-key")
}

fn parse_static_scss_map_has_key_namespaced_value(value: &str) -> Option<String> {
    parse_static_scss_map_has_key_value_with_name(value, "map.has-key")
}

fn parse_static_scss_map_keys_value(value: &str) -> Option<String> {
    parse_static_scss_map_keys_value_with_name(value, "map-keys")
}

fn parse_static_scss_map_keys_namespaced_value(value: &str) -> Option<String> {
    parse_static_scss_map_keys_value_with_name(value, "map.keys")
}

fn parse_static_scss_map_values_value(value: &str) -> Option<String> {
    parse_static_scss_map_values_value_with_name(value, "map-values")
}

fn parse_static_scss_map_values_namespaced_value(value: &str) -> Option<String> {
    parse_static_scss_map_values_value_with_name(value, "map.values")
}

fn parse_static_scss_map_merge_value(value: &str) -> Option<String> {
    parse_static_scss_map_merge_value_with_name(value, "map-merge")
}

fn parse_static_scss_map_merge_namespaced_value(value: &str) -> Option<String> {
    parse_static_scss_map_merge_value_with_name(value, "map.merge")
}

fn parse_static_scss_map_deep_merge_namespaced_value(value: &str) -> Option<String> {
    parse_static_scss_map_deep_merge_value_with_name(value, "map.deep-merge")
}

fn parse_static_scss_map_remove_value(value: &str) -> Option<String> {
    parse_static_scss_map_remove_value_with_name(value, "map-remove")
}

fn parse_static_scss_map_remove_namespaced_value(value: &str) -> Option<String> {
    parse_static_scss_map_remove_value_with_name(value, "map.remove")
}

fn parse_static_scss_map_deep_remove_namespaced_value(value: &str) -> Option<String> {
    parse_static_scss_map_deep_remove_value_with_name(value, "map.deep-remove")
}

fn parse_static_scss_map_set_value(value: &str) -> Option<String> {
    parse_static_scss_map_set_value_with_name(value, "map.set")
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

fn parse_static_scss_percentage_value(value: &str) -> Option<String> {
    parse_static_scss_percentage_value_with_name(value, "percentage")
}

fn parse_static_scss_math_percentage_value(value: &str) -> Option<String> {
    parse_static_scss_percentage_value_with_name(value, "math.percentage")
}

fn parse_static_scss_percentage_value_with_name(
    value: &str,
    function_name: &str,
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
    Some(format!("{}%", format_css_number(number.value * 100.0)))
}

fn parse_static_scss_unit_value(value: &str) -> Option<String> {
    parse_static_scss_unit_value_with_name(value, "unit")
}

fn parse_static_scss_math_unit_value(value: &str) -> Option<String> {
    parse_static_scss_unit_value_with_name(value, "math.unit")
}

fn parse_static_scss_unitless_value(value: &str) -> Option<String> {
    parse_static_scss_unitless_value_with_name(value, "unitless")
}

fn parse_static_scss_math_is_unitless_value(value: &str) -> Option<String> {
    parse_static_scss_unitless_value_with_name(value, "math.is-unitless")
}

fn parse_static_scss_comparable_value(value: &str) -> Option<String> {
    parse_static_scss_compatible_value_with_name(value, "comparable")
}

fn parse_static_scss_math_compatible_value(value: &str) -> Option<String> {
    parse_static_scss_compatible_value_with_name(value, "math.compatible")
}

fn parse_static_scss_unit_value_with_name(value: &str, function_name: &str) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, function_name)?;
    let [number] = arguments.as_slice() else {
        return None;
    };
    let number = reduce_static_scss_value(number.trim().to_string());
    let number = parse_numeric_value_with_unit(number.as_str())?;
    Some(format!("\"{}\"", number.unit))
}

fn parse_static_scss_unitless_value_with_name(value: &str, function_name: &str) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, function_name)?;
    let [number] = arguments.as_slice() else {
        return None;
    };
    let number = reduce_static_scss_value(number.trim().to_string());
    let number = parse_numeric_value_with_unit(number.as_str())?;
    Some(number.unit.is_empty().to_string())
}

fn parse_static_scss_compatible_value_with_name(
    value: &str,
    function_name: &str,
) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, function_name)?;
    let [left, right] = arguments.as_slice() else {
        return None;
    };
    let left = reduce_static_scss_value(left.trim().to_string());
    let right = reduce_static_scss_value(right.trim().to_string());
    let left = parse_numeric_value_with_unit(left.as_str())?;
    let right = parse_numeric_value_with_unit(right.as_str())?;
    if left.unit.eq_ignore_ascii_case(right.unit) {
        return Some("true".to_string());
    }
    if left.unit.is_empty() != right.unit.is_empty() {
        return Some("false".to_string());
    }
    None
}

fn parse_static_scss_map_get_value_with_name(value: &str, function_name: &str) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, function_name)?;
    let [map, keys @ ..] = arguments.as_slice() else {
        return None;
    };
    if keys.is_empty() {
        return None;
    }
    let mut current_map = map.trim().to_string();
    for (index, key) in keys.iter().enumerate() {
        let key = canonical_static_scss_map_key(key)?;
        let value = static_scss_map_entry_value(current_map.as_str(), key.as_str())?;
        if index + 1 == keys.len() {
            return Some(value);
        }
        current_map = value;
    }
    None
}

fn parse_static_scss_map_has_key_value_with_name(
    value: &str,
    function_name: &str,
) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, function_name)?;
    let [map, keys @ ..] = arguments.as_slice() else {
        return None;
    };
    if keys.is_empty() {
        return None;
    }
    let mut current_map = map.trim().to_string();
    for (index, key) in keys.iter().enumerate() {
        let key = canonical_static_scss_map_key(key)?;
        if index + 1 == keys.len() {
            return Some(
                static_scss_map_contains_key(current_map.as_str(), key.as_str()).to_string(),
            );
        }
        let Some(value) = static_scss_map_entry_value(current_map.as_str(), key.as_str()) else {
            return Some("false".to_string());
        };
        current_map = value;
    }
    None
}

fn parse_static_scss_map_keys_value_with_name(value: &str, function_name: &str) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, function_name)?;
    let [map] = arguments.as_slice() else {
        return None;
    };
    let keys = parse_static_scss_map_entries(map)?
        .into_iter()
        .map(|(key, _)| key)
        .collect::<Vec<_>>();
    static_scss_render_comma_list(keys)
}

fn parse_static_scss_map_values_value_with_name(
    value: &str,
    function_name: &str,
) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, function_name)?;
    let [map] = arguments.as_slice() else {
        return None;
    };
    let values = parse_static_scss_map_entries(map)?
        .into_iter()
        .map(|(_, value)| value)
        .collect::<Vec<_>>();
    static_scss_render_comma_list(values)
}

fn parse_static_scss_map_merge_value_with_name(value: &str, function_name: &str) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, function_name)?;
    let [left_map, merge_args @ ..] = arguments.as_slice() else {
        return None;
    };
    let (right_map, merge_path) = merge_args.split_last()?;
    let left_entries = parse_static_scss_map_entries(left_map)?;
    let right_entries = parse_static_scss_map_entries(right_map)?;
    let merged = if merge_path.is_empty() {
        static_scss_merge_map_entries(left_entries, right_entries)?
    } else {
        static_scss_update_nested_map_entries(left_entries, merge_path, |target_entries| {
            static_scss_merge_map_entries(target_entries, right_entries)
        })?
    };
    static_scss_render_map_entries(merged)
}

fn static_scss_merge_map_entries(
    mut left_entries: Vec<(String, String)>,
    right_entries: Vec<(String, String)>,
) -> Option<Vec<(String, String)>> {
    for (right_key, right_value) in right_entries {
        let right_canonical_key = canonical_static_scss_map_key(right_key.as_str())?;
        if let Some(index) =
            static_scss_map_entry_index(left_entries.as_slice(), right_canonical_key.as_str())?
        {
            left_entries[index].1 = right_value;
        } else {
            left_entries.push((right_key, right_value));
        }
    }
    Some(left_entries)
}

fn parse_static_scss_map_deep_merge_value_with_name(
    value: &str,
    function_name: &str,
) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, function_name)?;
    let [left_map, right_map] = arguments.as_slice() else {
        return None;
    };
    let merged = static_scss_deep_merge_map_entries(
        parse_static_scss_map_entries(left_map)?,
        parse_static_scss_map_entries(right_map)?,
    )?;
    static_scss_render_map_entries(merged)
}

fn static_scss_deep_merge_map_entries(
    mut left_entries: Vec<(String, String)>,
    right_entries: Vec<(String, String)>,
) -> Option<Vec<(String, String)>> {
    for (right_key, right_value) in right_entries {
        let right_canonical_key = canonical_static_scss_map_key(right_key.as_str())?;
        let merged_value = if let Some(index) =
            static_scss_map_entry_index(left_entries.as_slice(), right_canonical_key.as_str())?
        {
            match (
                parse_static_scss_map_entries(left_entries[index].1.as_str()),
                parse_static_scss_map_entries(right_value.as_str()),
            ) {
                (Some(left_child), Some(right_child)) => static_scss_render_map_entries(
                    static_scss_deep_merge_map_entries(left_child, right_child)?,
                )?,
                _ => right_value,
            }
        } else {
            right_value
        };
        if let Some(index) =
            static_scss_map_entry_index(left_entries.as_slice(), right_canonical_key.as_str())?
        {
            left_entries[index].1 = merged_value;
        } else {
            left_entries.push((right_key, merged_value));
        }
    }
    Some(left_entries)
}

fn parse_static_scss_map_remove_value_with_name(
    value: &str,
    function_name: &str,
) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, function_name)?;
    let [map, keys @ ..] = arguments.as_slice() else {
        return None;
    };
    if keys.is_empty() {
        return None;
    }
    let entries = static_scss_remove_map_entries(parse_static_scss_map_entries(map)?, keys)?;
    static_scss_render_map_entries(entries)
}

fn parse_static_scss_map_deep_remove_value_with_name(
    value: &str,
    function_name: &str,
) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, function_name)?;
    let [map, path @ ..] = arguments.as_slice() else {
        return None;
    };
    if path.is_empty() {
        return None;
    }
    let entries = static_scss_deep_remove_map_entries(parse_static_scss_map_entries(map)?, path)?;
    static_scss_render_map_entries(entries)
}

fn parse_static_scss_map_set_value_with_name(value: &str, function_name: &str) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, function_name)?;
    let [map, set_args @ ..] = arguments.as_slice() else {
        return None;
    };
    let [path_and_key @ .., value] = set_args else {
        return None;
    };
    let (key, set_path) = path_and_key.split_last()?;
    let mut entries = parse_static_scss_map_entries(map)?;
    if set_path.is_empty() {
        entries = static_scss_set_map_entry(entries, key, value)?;
    } else {
        entries = static_scss_update_nested_map_entries(entries, set_path, |target_entries| {
            static_scss_set_map_entry(target_entries, key, value)
        })?;
    }
    static_scss_render_map_entries(entries)
}

fn static_scss_set_map_entry(
    mut entries: Vec<(String, String)>,
    key: &str,
    value: &str,
) -> Option<Vec<(String, String)>> {
    let set_key = canonical_static_scss_map_key(key)?;
    if !static_scss_collection_member_is_static(value) {
        return None;
    }
    if let Some(index) = static_scss_map_entry_index(entries.as_slice(), set_key.as_str())? {
        entries[index].1 = value.trim().to_string();
    } else {
        entries.push((key.trim().to_string(), value.trim().to_string()));
    }
    Some(entries)
}

fn static_scss_remove_map_entries(
    entries: Vec<(String, String)>,
    keys: &[String],
) -> Option<Vec<(String, String)>> {
    let remove_keys = keys
        .iter()
        .map(|key| canonical_static_scss_map_key(key))
        .collect::<Option<Vec<_>>>()?;
    let mut retained_entries = Vec::new();
    for (key, value) in entries {
        let candidate_key = canonical_static_scss_map_key(key.as_str())?;
        if !remove_keys.contains(&candidate_key) {
            retained_entries.push((key, value));
        }
    }
    Some(retained_entries)
}

fn static_scss_deep_remove_map_entries(
    mut entries: Vec<(String, String)>,
    path: &[String],
) -> Option<Vec<(String, String)>> {
    let Some((key, remaining_path)) = path.split_first() else {
        return Some(entries);
    };
    let canonical_key = canonical_static_scss_map_key(key)?;
    if remaining_path.is_empty() {
        if let Some(index) =
            static_scss_map_entry_index(entries.as_slice(), canonical_key.as_str())?
        {
            entries.remove(index);
        }
        return Some(entries);
    }
    let Some(index) = static_scss_map_entry_index(entries.as_slice(), canonical_key.as_str())?
    else {
        return Some(entries);
    };
    let Some(child_entries) =
        static_scss_existing_nested_map_child_entries(entries[index].1.as_str())?
    else {
        return Some(entries);
    };
    let updated_child_entries = static_scss_deep_remove_map_entries(child_entries, remaining_path)?;
    entries[index].1 = static_scss_render_map_entries(updated_child_entries)?;
    Some(entries)
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

fn static_scss_named_argument(value: &str) -> Option<Option<(&str, &str)>> {
    let Some(index) = static_scss_top_level_separator_index(value, ':')? else {
        return Some(None);
    };
    let name = value.get(..index)?.trim().strip_prefix('$')?;
    let argument_value = value.get(index + ':'.len_utf8()..)?.trim();
    if name.is_empty() || argument_value.is_empty() {
        return None;
    }
    Some(Some((name, argument_value)))
}

fn static_scss_named_argument_value<'a>(value: &'a str, name: &str) -> Option<Option<&'a str>> {
    match static_scss_named_argument(value)? {
        Some((argument_name, argument_value)) if argument_name == name => {
            Some(Some(argument_value))
        }
        Some(_) => None,
        None => Some(None),
    }
}

fn static_scss_render_comma_list(items: Vec<String>) -> Option<String> {
    Some(if items.is_empty() {
        "()".to_string()
    } else {
        format!("({})", items.join(", "))
    })
}

fn static_scss_render_map_entries(entries: Vec<(String, String)>) -> Option<String> {
    Some(if entries.is_empty() {
        "()".to_string()
    } else {
        let entries = entries
            .into_iter()
            .map(|(key, value)| format!("{key}: {value}"))
            .collect::<Vec<_>>();
        format!("({})", entries.join(", "))
    })
}

fn static_scss_list_separator(value: &str) -> Option<&'static str> {
    let source = strip_static_scss_outer_container(value.trim()).unwrap_or_else(|| value.trim());
    if source.is_empty() {
        return None;
    }
    if split_static_scss_top_level(source, ',').is_some_and(|items| items.len() > 1) {
        return Some("comma");
    }
    if split_static_scss_top_level(source, '/').is_some_and(|items| items.len() > 1) {
        return Some("slash");
    }
    if split_static_scss_top_level_whitespace(source).is_some_and(|items| items.len() > 1) {
        return Some("space");
    }
    static_scss_collection_member_is_static(source).then_some("space")
}

fn parse_static_scss_map_entries(value: &str) -> Option<Vec<(String, String)>> {
    let source = strip_static_scss_outer_container(value.trim())?;
    if source.is_empty() {
        return Some(Vec::new());
    }
    let entries = split_static_scss_top_level(source, ',')?;
    let mut pairs = Vec::with_capacity(entries.len());
    for entry in entries {
        let colon_index = static_scss_top_level_separator_index(entry.as_str(), ':')??;
        let key = entry.get(..colon_index)?.trim();
        let value = entry.get(colon_index + ':'.len_utf8()..)?.trim();
        if key.is_empty()
            || value.is_empty()
            || key.contains('$')
            || !static_scss_collection_member_is_static(value)
        {
            return None;
        }
        pairs.push((key.to_string(), value.to_string()));
    }
    Some(pairs)
}

fn static_scss_update_nested_map_entries<F>(
    mut entries: Vec<(String, String)>,
    path: &[String],
    update: F,
) -> Option<Vec<(String, String)>>
where
    F: FnOnce(Vec<(String, String)>) -> Option<Vec<(String, String)>>,
{
    let Some((key, remaining_path)) = path.split_first() else {
        return update(entries);
    };
    let canonical_key = canonical_static_scss_map_key(key)?;
    let existing_index = static_scss_map_entry_index(entries.as_slice(), canonical_key.as_str())?;
    let child_entries = match existing_index {
        Some(index) => static_scss_nested_map_child_entries(entries[index].1.as_str())?,
        None => Vec::new(),
    };
    let updated_child_entries =
        static_scss_update_nested_map_entries(child_entries, remaining_path, update)?;
    let updated_child_value = static_scss_render_map_entries(updated_child_entries)?;
    if let Some(index) = existing_index {
        entries[index].1 = updated_child_value;
    } else {
        entries.push((key.trim().to_string(), updated_child_value));
    }
    Some(entries)
}

fn static_scss_nested_map_child_entries(value: &str) -> Option<Vec<(String, String)>> {
    if let Some(entries) = parse_static_scss_map_entries(value) {
        return Some(entries);
    }
    static_scss_collection_member_is_static(value).then(Vec::new)
}

fn static_scss_existing_nested_map_child_entries(
    value: &str,
) -> Option<Option<Vec<(String, String)>>> {
    if let Some(entries) = parse_static_scss_map_entries(value) {
        return Some(Some(entries));
    }
    static_scss_collection_member_is_static(value).then_some(None)
}

fn static_scss_map_entry_index(
    entries: &[(String, String)],
    canonical_key: &str,
) -> Option<Option<usize>> {
    for (index, (key, _)) in entries.iter().enumerate() {
        if canonical_static_scss_map_key(key.as_str())? == canonical_key {
            return Some(Some(index));
        }
    }
    Some(None)
}

fn static_scss_map_entry_value(map: &str, key: &str) -> Option<String> {
    parse_static_scss_map_entries(map)?
        .into_iter()
        .find_map(|(candidate_key, candidate_value)| {
            canonical_static_scss_map_key(candidate_key.as_str())
                .is_some_and(|candidate| candidate == key)
                .then_some(candidate_value)
        })
}

fn static_scss_map_contains_key(map: &str, key: &str) -> bool {
    parse_static_scss_map_entries(map).is_some_and(|entries| {
        entries.into_iter().any(|(candidate_key, _)| {
            canonical_static_scss_map_key(candidate_key.as_str())
                .is_some_and(|candidate| candidate == key)
        })
    })
}

fn canonical_static_scss_map_key(value: &str) -> Option<String> {
    let value = value.trim();
    if value.is_empty()
        || value.contains('$')
        || static_scss_top_level_separator_index(value, ':')?.is_some()
    {
        return None;
    }
    Some(strip_static_scss_quotes(value).unwrap_or(value).to_string())
}

fn static_scss_comparable_collection_value(value: &str) -> Option<String> {
    let value = value.trim();
    if !static_scss_collection_member_is_static(value) {
        return None;
    }
    Some(strip_static_scss_quotes(value).unwrap_or(value).to_string())
}

fn static_scss_collection_values_equal(left: &str, right: &str) -> bool {
    left == right || css_values_canonically_equal(left, right)
}

fn static_scss_collection_member_is_static(value: &str) -> bool {
    !value.trim().is_empty()
        && !value.contains('$')
        && static_scss_top_level_separator_index(value, ':').is_some_and(|index| index.is_none())
}

fn strip_static_scss_quotes(value: &str) -> Option<&str> {
    let quote = value.chars().next()?;
    if !matches!(quote, '"' | '\'') || !value.ends_with(quote) || value.len() < 2 {
        return None;
    }
    value.get(quote.len_utf8()..value.len().saturating_sub(quote.len_utf8()))
}

fn strip_static_scss_outer_container(value: &str) -> Option<&str> {
    let trimmed = value.trim();
    if trimmed.len() < 2 {
        return None;
    }
    let (open, close) = match trimmed.chars().next()? {
        '(' => ('(', ')'),
        '[' => ('[', ']'),
        _ => return None,
    };
    let end = static_scss_balanced_value_end(trimmed, 0, open, close)?;
    if end != trimmed.len() {
        return None;
    }
    trimmed
        .get(open.len_utf8()..trimmed.len().saturating_sub(close.len_utf8()))
        .map(str::trim)
}

fn split_static_scss_top_level(source: &str, separator: char) -> Option<Vec<String>> {
    let mut values = Vec::new();
    let mut cursor = 0usize;
    let mut index = 0usize;
    while index < source.len() {
        let ch = source[index..].chars().next()?;
        if ch == separator {
            let value = source.get(cursor..index)?.trim();
            if value.is_empty() {
                return None;
            }
            values.push(value.to_string());
            cursor = index + ch.len_utf8();
        }
        index = static_scss_next_value_index(source, index)?;
    }
    let value = source.get(cursor..)?.trim();
    if value.is_empty() {
        return None;
    }
    values.push(value.to_string());
    Some(values)
}

fn split_static_scss_top_level_whitespace(source: &str) -> Option<Vec<String>> {
    let mut values = Vec::new();
    let mut cursor = 0usize;
    let mut index = 0usize;
    while index < source.len() {
        let ch = source[index..].chars().next()?;
        if ch.is_ascii_whitespace() {
            let value = source.get(cursor..index)?.trim();
            if !value.is_empty() {
                values.push(value.to_string());
            }
            index += ch.len_utf8();
            while index < source.len() {
                let Some(next_ch) = source[index..].chars().next() else {
                    break;
                };
                if !next_ch.is_ascii_whitespace() {
                    break;
                }
                index += next_ch.len_utf8();
            }
            cursor = index;
            continue;
        }
        index = static_scss_next_value_index(source, index)?;
    }
    let value = source.get(cursor..)?.trim();
    if !value.is_empty() {
        values.push(value.to_string());
    }
    Some(values)
}

fn static_scss_top_level_separator_index(source: &str, separator: char) -> Option<Option<usize>> {
    let mut index = 0usize;
    while index < source.len() {
        let ch = source[index..].chars().next()?;
        if ch == separator {
            return Some(Some(index));
        }
        index = static_scss_next_value_index(source, index)?;
    }
    Some(None)
}

fn static_scss_next_value_index(source: &str, index: usize) -> Option<usize> {
    let ch = source[index..].chars().next()?;
    match ch {
        '"' | '\'' => static_scss_quoted_value_end(source, index, ch),
        '(' => static_scss_balanced_value_end(source, index, '(', ')'),
        '[' => static_scss_balanced_value_end(source, index, '[', ']'),
        ')' | ']' => None,
        _ => Some(index + ch.len_utf8()),
    }
}

fn static_scss_quoted_value_end(source: &str, start: usize, quote: char) -> Option<usize> {
    let mut index = start + quote.len_utf8();
    while index < source.len() {
        let ch = source[index..].chars().next()?;
        index += ch.len_utf8();
        if ch == '\\' {
            if let Some(escaped) = source[index..].chars().next() {
                index += escaped.len_utf8();
            }
        } else if ch == quote {
            return Some(index);
        }
    }
    None
}

fn static_scss_balanced_value_end(
    source: &str,
    start: usize,
    open: char,
    close: char,
) -> Option<usize> {
    let mut depth = 0usize;
    let mut index = start;
    while index < source.len() {
        let ch = source[index..].chars().next()?;
        match ch {
            '"' | '\'' => index = static_scss_quoted_value_end(source, index, ch)?,
            _ if ch == open => {
                depth += 1;
                index += ch.len_utf8();
                continue;
            }
            _ if ch == close => {
                depth = depth.checked_sub(1)?;
                index += ch.len_utf8();
                if depth == 0 {
                    return Some(index);
                }
                continue;
            }
            _ => index += ch.len_utf8(),
        }
    }
    None
}

pub(crate) fn static_scss_literal_truthiness(value: &str) -> Option<bool> {
    let trimmed = value.trim();
    let normalized = trimmed.to_ascii_lowercase();
    if let Some(inner) = strip_static_scss_outer_parens(trimmed) {
        return static_scss_literal_truthiness(inner);
    }
    match split_static_scss_boolean_operands(trimmed, "or") {
        Ok(Some(operands)) => return static_scss_or_truthiness(operands),
        Ok(None) => {}
        Err(()) => return None,
    }
    match split_static_scss_boolean_operands(trimmed, "and") {
        Ok(Some(operands)) => return static_scss_and_truthiness(operands),
        Ok(None) => {}
        Err(()) => return None,
    }
    if trimmed
        .get(..3)
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case("not"))
        && let Some(operand) = trimmed.get(3..)
        && operand.chars().next().is_some_and(char::is_whitespace)
    {
        return static_scss_literal_truthiness(operand.trim()).map(|truthy| !truthy);
    }
    match static_scss_comparison_truthiness(trimmed) {
        Ok(Some(truthy)) => return Some(truthy),
        Ok(None) => {}
        Err(()) => return None,
    }
    match normalized.as_str() {
        "false" | "null" => Some(false),
        "" => None,
        _ if normalized.starts_with('$') || normalized.contains('(') => None,
        _ => Some(true),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StaticScssComparisonOperator {
    Equal,
    NotEqual,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
}

fn static_scss_comparison_truthiness(value: &str) -> Result<Option<bool>, ()> {
    let Some((left, operator, right)) = split_static_scss_comparison(value)? else {
        return Ok(None);
    };
    let left = static_scss_comparable_operand(left).ok_or(())?;
    let right = static_scss_comparable_operand(right).ok_or(())?;
    let equal = left == right || css_values_canonically_equal(left.as_str(), right.as_str());
    Ok(Some(match operator {
        StaticScssComparisonOperator::Equal => equal,
        StaticScssComparisonOperator::NotEqual => !equal,
        StaticScssComparisonOperator::LessThan
        | StaticScssComparisonOperator::LessThanOrEqual
        | StaticScssComparisonOperator::GreaterThan
        | StaticScssComparisonOperator::GreaterThanOrEqual => {
            static_scss_numeric_ordering_truthiness(left.as_str(), operator, right.as_str())
                .ok_or(())?
        }
    }))
}

fn static_scss_numeric_ordering_truthiness(
    left: &str,
    operator: StaticScssComparisonOperator,
    right: &str,
) -> Option<bool> {
    let left_value = parse_numeric_value_with_unit(left)?;
    let right_value = parse_numeric_value_with_unit(right)?;
    if !left_value.unit.eq_ignore_ascii_case(right_value.unit)
        && !static_scss_zero_values_share_unitless_canonical_form(left, right)
    {
        return None;
    }
    Some(match operator {
        StaticScssComparisonOperator::LessThan => left_value.value < right_value.value,
        StaticScssComparisonOperator::LessThanOrEqual => left_value.value <= right_value.value,
        StaticScssComparisonOperator::GreaterThan => left_value.value > right_value.value,
        StaticScssComparisonOperator::GreaterThanOrEqual => left_value.value >= right_value.value,
        StaticScssComparisonOperator::Equal | StaticScssComparisonOperator::NotEqual => {
            return None;
        }
    })
}

fn static_scss_zero_values_share_unitless_canonical_form(left: &str, right: &str) -> bool {
    let Some(left_value) = parse_numeric_value_with_unit(left) else {
        return false;
    };
    let Some(right_value) = parse_numeric_value_with_unit(right) else {
        return false;
    };
    if left_value.value != 0.0 || right_value.value != 0.0 {
        return false;
    }
    if !left_value.unit.is_empty() && !right_value.unit.is_empty() {
        return false;
    }
    css_values_canonically_equal(left, right)
}

fn static_scss_comparable_operand(value: &str) -> Option<String> {
    let reduced = reduce_static_scss_value(value.trim().to_string());
    let normalized = reduced.to_ascii_lowercase();
    if reduced.is_empty()
        || reduced.contains('$')
        || normalized.contains("var(")
        || normalized.contains("env(")
        || normalized.contains('(')
        || normalized.contains(')')
    {
        return None;
    }
    Some(reduced)
}

fn split_static_scss_comparison(
    value: &str,
) -> Result<Option<(&str, StaticScssComparisonOperator, &str)>, ()> {
    let mut comparison = None;
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;
    let mut quote: Option<char> = None;
    let mut index = 0usize;

    while index < value.len() {
        let ch = value[index..].chars().next().ok_or(())?;
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
        if matches!(ch, '"' | '\'') {
            quote = Some(ch);
            index += ch.len_utf8();
            continue;
        }
        match ch {
            '(' => paren_depth += 1,
            ')' => paren_depth = paren_depth.checked_sub(1).ok_or(())?,
            '[' => bracket_depth += 1,
            ']' => bracket_depth = bracket_depth.checked_sub(1).ok_or(())?,
            '=' | '!' | '<' | '>' if paren_depth == 0 && bracket_depth == 0 => {
                let (operator, width) = static_scss_comparison_operator_at(value, index)?;
                let left = value.get(..index).ok_or(())?.trim();
                let right = value.get(index + width..).ok_or(())?.trim();
                if left.is_empty() || right.is_empty() || comparison.is_some() {
                    return Err(());
                }
                comparison = Some((left, operator, right));
                index += width;
                continue;
            }
            _ => {}
        }
        index += ch.len_utf8();
    }
    if quote.is_some() || paren_depth != 0 || bracket_depth != 0 {
        return Err(());
    }
    Ok(comparison)
}

fn static_scss_comparison_operator_at(
    value: &str,
    index: usize,
) -> Result<(StaticScssComparisonOperator, usize), ()> {
    let suffix = value.get(index..).ok_or(())?;
    if suffix.starts_with("==") {
        return Ok((StaticScssComparisonOperator::Equal, 2));
    }
    if suffix.starts_with("!=") {
        return Ok((StaticScssComparisonOperator::NotEqual, 2));
    }
    if suffix.starts_with("<=") {
        return Ok((StaticScssComparisonOperator::LessThanOrEqual, 2));
    }
    if suffix.starts_with(">=") {
        return Ok((StaticScssComparisonOperator::GreaterThanOrEqual, 2));
    }
    if suffix.starts_with('<') {
        return Ok((StaticScssComparisonOperator::LessThan, 1));
    }
    if suffix.starts_with('>') {
        return Ok((StaticScssComparisonOperator::GreaterThan, 1));
    }
    Err(())
}

fn strip_static_scss_outer_parens(value: &str) -> Option<&str> {
    let inner_start = value.strip_prefix('(')?;
    value.strip_suffix(')')?;
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;
    let mut quote: Option<char> = None;
    let mut index = 0usize;
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
        if matches!(ch, '"' | '\'') {
            quote = Some(ch);
            index += ch.len_utf8();
            continue;
        }
        match ch {
            '(' => paren_depth += 1,
            ')' => {
                paren_depth = paren_depth.checked_sub(1)?;
                if paren_depth == 0 && index + ch.len_utf8() != value.len() {
                    return None;
                }
            }
            '[' => bracket_depth += 1,
            ']' => bracket_depth = bracket_depth.checked_sub(1)?,
            _ => {}
        }
        index += ch.len_utf8();
    }
    (quote.is_none() && paren_depth == 0 && bracket_depth == 0)
        .then(|| inner_start.strip_suffix(')').unwrap_or(inner_start).trim())
}

fn static_scss_or_truthiness(operands: Vec<&str>) -> Option<bool> {
    let mut saw_unknown = false;
    for operand in operands {
        match static_scss_literal_truthiness(operand) {
            Some(true) => return Some(true),
            Some(false) => {}
            None => saw_unknown = true,
        }
    }
    (!saw_unknown).then_some(false)
}

fn static_scss_and_truthiness(operands: Vec<&str>) -> Option<bool> {
    let mut saw_unknown = false;
    for operand in operands {
        match static_scss_literal_truthiness(operand) {
            Some(true) => {}
            Some(false) => return Some(false),
            None => saw_unknown = true,
        }
    }
    (!saw_unknown).then_some(true)
}

fn split_static_scss_boolean_operands<'a>(
    value: &'a str,
    keyword: &str,
) -> Result<Option<Vec<&'a str>>, ()> {
    let mut operands = Vec::new();
    let mut cursor = 0usize;
    let mut index = 0usize;
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;
    let mut quote: Option<char> = None;

    while index < value.len() {
        let ch = value[index..].chars().next().ok_or(())?;
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
        if matches!(ch, '"' | '\'') {
            quote = Some(ch);
            index += ch.len_utf8();
            continue;
        }
        match ch {
            '(' => paren_depth += 1,
            ')' => paren_depth = paren_depth.checked_sub(1).ok_or(())?,
            '[' => bracket_depth += 1,
            ']' => bracket_depth = bracket_depth.checked_sub(1).ok_or(())?,
            _ => {}
        }
        if paren_depth == 0
            && bracket_depth == 0
            && static_scss_boolean_keyword_at(value, index, keyword)
        {
            let operand = value.get(cursor..index).ok_or(())?.trim();
            if operand.is_empty() {
                return Err(());
            }
            operands.push(operand);
            index += keyword.len();
            cursor = index;
            continue;
        }
        index += ch.len_utf8();
    }

    if quote.is_some() || paren_depth != 0 || bracket_depth != 0 {
        return Err(());
    }
    if operands.is_empty() {
        return Ok(None);
    }
    let operand = value.get(cursor..).ok_or(())?.trim();
    if operand.is_empty() {
        return Err(());
    }
    operands.push(operand);
    Ok(Some(operands))
}

fn static_scss_boolean_keyword_at(value: &str, index: usize, keyword: &str) -> bool {
    if !value
        .get(index..)
        .is_some_and(|suffix| suffix.starts_with(keyword))
    {
        return false;
    }
    let before_ok = value
        .get(..index)
        .and_then(|prefix| prefix.chars().next_back())
        .is_some_and(char::is_whitespace);
    let after_index = index + keyword.len();
    let after_ok = value
        .get(after_index..)
        .and_then(|suffix| suffix.chars().next())
        .is_some_and(char::is_whitespace);
    before_ok && after_ok
}
