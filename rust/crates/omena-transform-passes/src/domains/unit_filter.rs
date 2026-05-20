use crate::{
    domains::{
        number::numeric_prefix_end,
        unit_properties::{is_css_angle_unit, is_css_length_unit},
    },
    helpers::{
        ascii::normalize_ascii_whitespace,
        values::{
            compact_adjacent_css_function_separators, split_top_level_whitespace_value_components,
            substitute_static_css_function_references_in_value,
        },
    },
};

pub(crate) fn normalize_static_filter_functions(value: &str) -> Option<String> {
    let normalized = substitute_static_css_function_references_in_value(
        value,
        &[
            ("opacity", normalize_opacity_filter_function),
            ("brightness", normalize_brightness_filter_function),
            ("grayscale", normalize_grayscale_filter_function),
            ("sepia", normalize_sepia_filter_function),
            ("invert", normalize_invert_filter_function),
            ("contrast", normalize_contrast_filter_function),
            ("saturate", normalize_saturate_filter_function),
            ("blur", normalize_blur_filter_function),
            ("hue-rotate", normalize_hue_rotate_filter_function),
            ("drop-shadow", normalize_drop_shadow_filter_function),
        ],
    )
    .unwrap_or_else(|| value.to_string());
    let compacted =
        compact_adjacent_css_function_separators(&normalize_ascii_whitespace(&normalized));
    let compacted = if compacted.is_empty() {
        "none".to_string()
    } else {
        compacted
    };
    (compacted != value).then_some(compacted)
}

fn normalize_opacity_filter_function(value: &str) -> Option<String> {
    normalize_default_one_filter_function(value, "opacity")
}

fn normalize_brightness_filter_function(value: &str) -> Option<String> {
    normalize_default_one_filter_function(value, "brightness")
}

fn normalize_grayscale_filter_function(value: &str) -> Option<String> {
    normalize_identity_zero_filter_function(value)
}

fn normalize_sepia_filter_function(value: &str) -> Option<String> {
    normalize_identity_zero_filter_function(value)
}

fn normalize_invert_filter_function(value: &str) -> Option<String> {
    normalize_identity_zero_filter_function(value)
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

fn normalize_identity_zero_filter_function(value: &str) -> Option<String> {
    let inner = whole_function_inner(value)?.trim();
    static_filter_number_is_zero(inner).then_some(String::new())
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

fn normalize_drop_shadow_filter_function(value: &str) -> Option<String> {
    let inner = whole_function_inner(value)?;
    let components = split_top_level_whitespace_value_components(inner)?;
    let mut lengths = Vec::new();
    let mut other_components = Vec::new();

    for component in components {
        if let Some(length) = normalize_drop_shadow_length_component(&component) {
            lengths.push(length);
        } else {
            other_components.push(component.trim().to_string());
        }
    }

    if !(2..=3).contains(&lengths.len()) {
        return None;
    }

    let mut replacement_components = vec![lengths[0].clone(), lengths[1].clone()];
    if lengths.len() == 3 && lengths[2] != "0" {
        replacement_components.push(lengths[2].clone());
    }
    replacement_components.extend(other_components);

    let replacement = format!("drop-shadow({})", replacement_components.join(" "));
    (replacement != value).then_some(replacement)
}

fn normalize_drop_shadow_length_component(component: &str) -> Option<String> {
    let component = component.trim();
    let split = numeric_prefix_end(component)?;
    let (number, unit) = component.split_at(split);
    let parsed = number.parse::<f64>().ok()?;
    if !parsed.is_finite() || (!unit.is_empty() && !is_css_length_unit(unit)) {
        return None;
    }
    if unit.is_empty() && parsed != 0.0 {
        return None;
    }

    if parsed == 0.0 {
        Some("0".to_string())
    } else {
        Some(component.to_string())
    }
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

fn static_filter_number_is_zero(value: &str) -> bool {
    if let Some(number) = value.strip_suffix('%') {
        return number
            .parse::<f64>()
            .is_ok_and(|parsed| parsed.is_finite() && parsed == 0.0);
    }

    value
        .parse::<f64>()
        .is_ok_and(|parsed| parsed.is_finite() && parsed == 0.0)
}

fn is_zero_filter_numeric_unit_argument(value: &str, is_unit: fn(&str) -> bool) -> Option<()> {
    let split = numeric_prefix_end(value)?;
    let (number, unit) = value.split_at(split);
    (number.parse::<f64>().is_ok_and(|parsed| parsed == 0.0) && is_unit(unit)).then_some(())
}
