use omena_value_lattice::{
    format_css_number, parse_reducible_ceil_value, parse_reducible_floor_value,
    substitute_static_css_function_references_in_value_until_stable,
};

use crate::value_eval::reduce_static_less_numeric_value;

use super::{
    less_colors::{
        parse_static_less_alpha_value, parse_static_less_argb_value,
        parse_static_less_average_value, parse_static_less_blue_value,
        parse_static_less_color_value, parse_static_less_contrast_value,
        parse_static_less_darken_value, parse_static_less_desaturate_value,
        parse_static_less_difference_value, parse_static_less_exclusion_value,
        parse_static_less_fade_value, parse_static_less_fadein_value,
        parse_static_less_fadeout_value, parse_static_less_green_value,
        parse_static_less_greyscale_value, parse_static_less_hardlight_value,
        parse_static_less_hsv_value, parse_static_less_hsva_value, parse_static_less_hsvhue_value,
        parse_static_less_hsvsaturation_value, parse_static_less_hsvvalue_value,
        parse_static_less_hue_value, parse_static_less_lighten_value,
        parse_static_less_lightness_value, parse_static_less_luma_value,
        parse_static_less_luminance_value, parse_static_less_mix_value,
        parse_static_less_multiply_value, parse_static_less_negation_value,
        parse_static_less_overlay_value, parse_static_less_red_value,
        parse_static_less_rgb_color_value, parse_static_less_saturate_value,
        parse_static_less_saturation_value, parse_static_less_screen_value,
        parse_static_less_shade_value, parse_static_less_softlight_value,
        parse_static_less_spin_value, parse_static_less_tint_value,
    },
    less_numbers::{
        parse_static_less_acos_value, parse_static_less_asin_value, parse_static_less_atan_value,
        parse_static_less_convert_value, parse_static_less_cos_value,
        parse_static_less_extract_value, parse_static_less_get_unit_value,
        parse_static_less_length_value, parse_static_less_percentage_value,
        parse_static_less_pi_value, parse_static_less_range_value, parse_static_less_round_value,
        parse_static_less_sin_value, parse_static_less_tan_value, parse_static_less_unit_value,
    },
    less_predicates::{
        parse_static_less_boolean_value, parse_static_less_if_value,
        parse_static_less_iscolor_value, parse_static_less_isdefined_value,
        parse_static_less_isem_value, parse_static_less_iskeyword_value,
        parse_static_less_isnumber_value, parse_static_less_ispercentage_value,
        parse_static_less_ispixel_value, parse_static_less_isruleset_value,
        parse_static_less_isstring_value, parse_static_less_isunit_value,
        parse_static_less_isurl_value,
    },
    less_strings::{
        parse_static_less_escape_value, parse_static_less_format_value,
        parse_static_less_replace_value, parse_static_less_url_escape_value,
        reduce_static_less_escaped_string_value,
    },
    model::StaticLessResolvedValue,
};

pub(super) fn reduce_static_less_value(value: String) -> String {
    reduce_static_less_value_with_escape_flag(value).text
}

pub(super) fn reduce_static_less_value_with_escape_flag(value: String) -> StaticLessResolvedValue {
    if let Some(escaped) = parse_static_less_escape_value(value.as_str())
        .or_else(|| reduce_static_less_escaped_string_value(value.as_str()))
    {
        return StaticLessResolvedValue {
            text: escaped,
            escaped: true,
        };
    }
    if let Some(encoded) = parse_static_less_url_escape_value(value.as_str()) {
        return StaticLessResolvedValue {
            text: encoded,
            escaped: false,
        };
    }
    let value = reduce_static_less_numeric_value(value);
    let text = substitute_static_css_function_references_in_value_until_stable(
        value.as_str(),
        &[
            ("unit", parse_static_less_unit_value),
            ("get-unit", parse_static_less_get_unit_value),
            ("convert", parse_static_less_convert_value),
            ("if", parse_static_less_if_value),
            ("boolean", parse_static_less_boolean_value),
            ("percentage", parse_static_less_percentage_value),
            ("red", parse_static_less_red_value),
            ("green", parse_static_less_green_value),
            ("blue", parse_static_less_blue_value),
            ("alpha", parse_static_less_alpha_value),
            ("hue", parse_static_less_hue_value),
            ("saturation", parse_static_less_saturation_value),
            ("lightness", parse_static_less_lightness_value),
            ("hsv", parse_static_less_hsv_value),
            ("hsva", parse_static_less_hsva_value),
            ("hsvhue", parse_static_less_hsvhue_value),
            ("hsvsaturation", parse_static_less_hsvsaturation_value),
            ("hsvvalue", parse_static_less_hsvvalue_value),
            ("luma", parse_static_less_luma_value),
            ("luminance", parse_static_less_luminance_value),
            ("contrast", parse_static_less_contrast_value),
            ("color", parse_static_less_color_value),
            ("argb", parse_static_less_argb_value),
            ("fade", parse_static_less_fade_value),
            ("fadein", parse_static_less_fadein_value),
            ("fadeout", parse_static_less_fadeout_value),
            ("mix", parse_static_less_mix_value),
            ("tint", parse_static_less_tint_value),
            ("shade", parse_static_less_shade_value),
            ("multiply", parse_static_less_multiply_value),
            ("screen", parse_static_less_screen_value),
            ("overlay", parse_static_less_overlay_value),
            ("softlight", parse_static_less_softlight_value),
            ("hardlight", parse_static_less_hardlight_value),
            ("difference", parse_static_less_difference_value),
            ("exclusion", parse_static_less_exclusion_value),
            ("average", parse_static_less_average_value),
            ("negation", parse_static_less_negation_value),
            ("lighten", parse_static_less_lighten_value),
            ("darken", parse_static_less_darken_value),
            ("saturate", parse_static_less_saturate_value),
            ("desaturate", parse_static_less_desaturate_value),
            ("spin", parse_static_less_spin_value),
            ("greyscale", parse_static_less_greyscale_value),
            ("ceil", parse_reducible_ceil_value),
            ("floor", parse_reducible_floor_value),
            ("round", parse_static_less_round_value),
            ("pi", parse_static_less_pi_value),
            ("sin", parse_static_less_sin_value),
            ("cos", parse_static_less_cos_value),
            ("tan", parse_static_less_tan_value),
            ("asin", parse_static_less_asin_value),
            ("acos", parse_static_less_acos_value),
            ("atan", parse_static_less_atan_value),
            ("isnumber", parse_static_less_isnumber_value),
            ("iscolor", parse_static_less_iscolor_value),
            ("isstring", parse_static_less_isstring_value),
            ("iskeyword", parse_static_less_iskeyword_value),
            ("isurl", parse_static_less_isurl_value),
            ("isdefined", parse_static_less_isdefined_value),
            ("isruleset", parse_static_less_isruleset_value),
            ("ispixel", parse_static_less_ispixel_value),
            ("ispercentage", parse_static_less_ispercentage_value),
            ("isem", parse_static_less_isem_value),
            ("isunit", parse_static_less_isunit_value),
            ("length", parse_static_less_length_value),
            ("extract", parse_static_less_extract_value),
            ("range", parse_static_less_range_value),
            ("replace", parse_static_less_replace_value),
            ("%", parse_static_less_format_value),
        ],
    )
    .unwrap_or(value);
    let text = parse_static_less_rgb_color_value(text.as_str()).unwrap_or(text);
    StaticLessResolvedValue {
        text,
        escaped: false,
    }
}

pub(super) fn format_static_less_math_number(value: f64) -> Option<String> {
    value
        .is_finite()
        .then(|| format_static_less_channel_number(if value.abs() < 1e-10 { 0.0 } else { value }))
}

pub(super) fn format_static_less_number(value: f64) -> String {
    let formatted = format_css_number(value);
    if let Some(suffix) = formatted.strip_prefix('.') {
        return format!("0.{suffix}");
    }
    if let Some(suffix) = formatted.strip_prefix("-.") {
        return format!("-0.{suffix}");
    }
    formatted
}

pub(super) fn format_static_less_channel_number(value: f64) -> String {
    let formatted = if value.fract() == 0.0 {
        format!("{value:.0}")
    } else {
        format!("{value:.8}")
            .trim_end_matches('0')
            .trim_end_matches('.')
            .to_string()
    };
    if let Some(suffix) = formatted.strip_prefix('.') {
        return format!("0.{suffix}");
    }
    if let Some(suffix) = formatted.strip_prefix("-.") {
        return format!("-0.{suffix}");
    }
    formatted
}
