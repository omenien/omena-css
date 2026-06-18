use omena_value_lattice::{
    css_values_canonically_equal,
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
    parse_oklab_oklch_value, parse_static_srgb_color_with_alpha,
    parse_whole_function_value_arguments,
    substitute_static_css_function_references_in_value_until_stable,
};

pub(crate) fn reduce_static_scss_value(value: String) -> String {
    let trimmed = value.trim();
    let value = substitute_static_css_function_references_in_value_until_stable(
        trimmed,
        &[
            ("if", parse_static_scss_if_value),
            ("type-of", parse_static_scss_type_of_value),
            ("meta.type-of", parse_static_scss_meta_type_of_value),
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
            ("list.separator", parse_static_scss_list_separator_value),
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
            ("percentage", parse_static_scss_percentage_value),
            ("unit", parse_static_scss_unit_value),
            ("math.unit", parse_static_scss_math_unit_value),
            ("unitless", parse_static_scss_unitless_value),
            ("math.is-unitless", parse_static_scss_math_is_unitless_value),
            ("comparable", parse_static_scss_comparable_value),
            ("math.compatible", parse_static_scss_math_compatible_value),
        ],
    )
    .unwrap_or_else(|| trimmed.to_string());
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
    let arguments = parse_whole_function_value_arguments(value, "list.separator")?;
    let [list] = arguments.as_slice() else {
        return None;
    };
    Some(format!("\"{}\"", static_scss_list_separator(list)?))
}

fn parse_static_scss_list_is_bracketed_value(value: &str) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, "list.is-bracketed")?;
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

fn parse_static_scss_math_ceil_value(value: &str) -> Option<String> {
    parse_static_scss_numeric_alias_value(value, "math.ceil", "ceil", parse_reducible_ceil_value)
}

fn parse_static_scss_math_floor_value(value: &str) -> Option<String> {
    parse_static_scss_numeric_alias_value(value, "math.floor", "floor", parse_reducible_floor_value)
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

fn parse_static_scss_numeric_alias_value(
    value: &str,
    alias_name: &str,
    kernel_name: &str,
    parse_kernel_value: fn(&str) -> Option<String>,
) -> Option<String> {
    let inner = omena_value_lattice::parse_whole_function_value_inner(value, alias_name)?;
    parse_kernel_value(format!("{kernel_name}({inner})").as_str())
}

fn parse_static_scss_percentage_value(value: &str) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, "percentage")?;
    let [number] = arguments.as_slice() else {
        return None;
    };
    let number = parse_numeric_value_with_unit(number.trim())?;
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
    let number = parse_numeric_value_with_unit(number.trim())?;
    Some(format!("\"{}\"", number.unit))
}

fn parse_static_scss_unitless_value_with_name(value: &str, function_name: &str) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, function_name)?;
    let [number] = arguments.as_slice() else {
        return None;
    };
    let number = parse_numeric_value_with_unit(number.trim())?;
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
    let left = parse_numeric_value_with_unit(left.trim())?;
    let right = parse_numeric_value_with_unit(right.trim())?;
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
        _ => (
            split_static_scss_top_level_whitespace(source)?,
            StaticScssListSeparator::Space,
        ),
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
    }
}

fn static_scss_join_list_items(list: &StaticScssListValue) -> String {
    match list.separator {
        StaticScssListSeparator::Space => list.items.join(" "),
        StaticScssListSeparator::Comma => list.items.join(", "),
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
    let reduced = reduce_static_numeric_value(value.trim().to_string());
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
