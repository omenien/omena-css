use crate::{
    domains::{
        number::numeric_prefix_end,
        unit_properties::{is_css_angle_unit, is_css_length_unit},
    },
    helpers::values::{
        compact_adjacent_css_function_separators,
        substitute_static_css_function_references_in_value,
    },
};

pub(crate) fn normalize_static_filter_functions(value: &str) -> Option<String> {
    let normalized = substitute_static_css_function_references_in_value(
        value,
        &[
            ("opacity", normalize_opacity_filter_function),
            ("brightness", normalize_brightness_filter_function),
            ("contrast", normalize_contrast_filter_function),
            ("saturate", normalize_saturate_filter_function),
            ("blur", normalize_blur_filter_function),
            ("hue-rotate", normalize_hue_rotate_filter_function),
        ],
    )
    .unwrap_or_else(|| value.to_string());
    let compacted = compact_adjacent_css_function_separators(&normalized);
    (compacted != value).then_some(compacted)
}

fn normalize_opacity_filter_function(value: &str) -> Option<String> {
    normalize_default_one_filter_function(value, "opacity")
}

fn normalize_brightness_filter_function(value: &str) -> Option<String> {
    normalize_default_one_filter_function(value, "brightness")
}

fn normalize_contrast_filter_function(value: &str) -> Option<String> {
    normalize_default_one_filter_function(value, "contrast")
}

fn normalize_saturate_filter_function(value: &str) -> Option<String> {
    normalize_default_one_filter_function(value, "saturate")
}

fn normalize_default_one_filter_function(value: &str, function_name: &str) -> Option<String> {
    let inner = whole_function_inner(value)?.trim();
    if static_filter_number_is_default_one(inner) {
        return Some(format!("{function_name}()"));
    }
    None
}

fn normalize_blur_filter_function(value: &str) -> Option<String> {
    let inner = whole_function_inner(value)?.trim();
    is_zero_filter_numeric_unit_argument(inner, |unit| {
        unit.is_empty() || is_css_length_unit(unit)
    })?;
    Some("blur()".to_string())
}

fn normalize_hue_rotate_filter_function(value: &str) -> Option<String> {
    let inner = whole_function_inner(value)?.trim();
    is_zero_filter_numeric_unit_argument(inner, is_css_angle_unit)?;
    Some("hue-rotate()".to_string())
}

fn whole_function_inner(value: &str) -> Option<&str> {
    let open_index = value.find('(')?;
    value.get(open_index + 1..value.len().checked_sub(1)?)
}

fn static_filter_number_is_default_one(value: &str) -> bool {
    if let Some(number) = value.strip_suffix('%') {
        return number
            .parse::<f64>()
            .is_ok_and(|parsed| parsed.is_finite() && parsed == 100.0);
    }

    value
        .parse::<f64>()
        .is_ok_and(|parsed| parsed.is_finite() && parsed == 1.0)
}

fn is_zero_filter_numeric_unit_argument(value: &str, is_unit: fn(&str) -> bool) -> Option<()> {
    let split = numeric_prefix_end(value)?;
    let (number, unit) = value.split_at(split);
    (number.parse::<f64>().is_ok_and(|parsed| parsed == 0.0) && is_unit(unit)).then_some(())
}
