use omena_value_lattice::css_number_is_zero;

use crate::{
    domains::{
        number::{compress_number_prefix, format_css_number, numeric_prefix_end},
        unit_properties::{is_css_angle_unit, is_css_length_unit},
    },
    helpers::values::{
        compact_adjacent_css_function_separators, split_top_level_value_arguments,
        split_top_level_whitespace_value_components,
        substitute_static_css_function_references_in_value,
    },
};

pub(crate) fn normalize_static_transform_functions(value: &str) -> Option<String> {
    let normalized = substitute_static_css_function_references_in_value(
        value,
        &[
            ("rotate", normalize_zero_angle_transform_function),
            ("rotateX", normalize_zero_angle_transform_function),
            ("rotateY", normalize_zero_angle_transform_function),
            ("rotateZ", normalize_zero_angle_transform_function),
            ("rotate3d", normalize_rotate3d_transform_function),
            ("scale", normalize_scale_transform_function),
            ("scale3d", normalize_scale3d_transform_function),
            ("skew", normalize_skew_transform_function),
            ("skewX", normalize_skew_x_zero_transform_function),
            ("skewY", normalize_zero_angle_transform_function),
            ("translate", normalize_translate_transform_function),
            ("translate3d", normalize_translate3d_transform_function),
            ("translateX", normalize_translate_x_zero_transform_function),
            (
                "translateY",
                normalize_unary_zero_length_percentage_transform_function,
            ),
            ("translateZ", normalize_unary_zero_length_transform_function),
            (
                "perspective",
                normalize_unary_zero_length_transform_function,
            ),
        ],
    )?;
    Some(compact_adjacent_css_function_separators(&normalized))
}

pub(crate) fn normalize_individual_translate_value(value: &str) -> Option<String> {
    let components = split_top_level_whitespace_value_components(value)?;
    let replacement = match components.as_slice() {
        [x] => normalize_translate_component(x)?,
        [x, y] => {
            let x = normalize_translate_component(x)?;
            let y = normalize_translate_component(y)?;
            match (
                is_zero_translate_component(&x),
                is_zero_translate_component(&y),
            ) {
                (true, true) => "0".to_string(),
                (_, true) => x,
                _ => format!("{x} {y}"),
            }
        }
        [x, y, z] => {
            let x = normalize_translate_component(x)?;
            let y = normalize_translate_component(y)?;
            let z = normalize_translate_component(z)?;
            match (
                is_zero_translate_component(&x),
                is_zero_translate_component(&y),
                is_zero_translate_component(&z),
            ) {
                (true, true, true) => "0".to_string(),
                (_, true, true) => x,
                (_, _, true) => format!("{x} {y}"),
                _ => format!("{x} {y} {z}"),
            }
        }
        _ => return None,
    };

    (replacement.len() < value.len()).then_some(replacement)
}

pub(crate) fn normalize_individual_rotate_value(value: &str) -> Option<String> {
    let components = split_top_level_whitespace_value_components(value)?;
    let replacement = match components.as_slice() {
        [angle] => normalize_individual_rotate_angle(angle)?,
        [axis, angle] => {
            let axis = normalize_individual_rotate_axis(axis)?;
            let angle = normalize_individual_rotate_angle(angle)?;
            if axis == "z" {
                angle
            } else {
                format!("{axis} {angle}")
            }
        }
        [x, y, z, angle] => {
            let x = normalize_transform_number(x)?;
            let y = normalize_transform_number(y)?;
            let z = normalize_transform_number(z)?;
            let angle = normalize_individual_rotate_angle(angle)?;
            match (x.as_str(), y.as_str(), z.as_str()) {
                ("1", "0", "0") => format!("x {angle}"),
                ("0", "1", "0") => format!("y {angle}"),
                ("0", "0", "1") => angle,
                _ => return None,
            }
        }
        _ => return None,
    };

    let trimmed = value.trim();
    (replacement != trimmed && replacement.len() <= trimmed.len()).then_some(replacement)
}

fn normalize_individual_rotate_axis(axis: &str) -> Option<&'static str> {
    match axis.to_ascii_lowercase().as_str() {
        "x" => Some("x"),
        "y" => Some("y"),
        "z" => Some("z"),
        _ => None,
    }
}

fn normalize_individual_rotate_angle(angle: &str) -> Option<String> {
    let angle = angle.trim();
    let split = numeric_prefix_end(angle)?;
    let (number, unit) = angle.split_at(split);
    if !is_css_angle_unit(unit) {
        return None;
    }
    let unit = unit.to_ascii_lowercase();

    let parsed = number.parse::<f64>().ok()?;
    if !parsed.is_finite() {
        return None;
    }
    if parsed == 0.0 && matches!(unit.as_str(), "deg" | "rad") {
        return Some("0deg".to_string());
    }

    Some(format!(
        "{}{}",
        compress_number_prefix(&format_css_number(parsed)),
        unit
    ))
}

fn normalize_translate_component(component: &str) -> Option<String> {
    let component = component.trim();
    if is_zero_transform_numeric_unit_argument(component, is_css_transform_length_percentage_unit)
        .is_some()
    {
        return Some("0".to_string());
    }

    Some(component.to_string())
}

fn is_zero_translate_component(component: &str) -> bool {
    component == "0"
}

pub(crate) fn normalize_individual_scale_value(value: &str) -> Option<String> {
    let components = split_top_level_whitespace_value_components(value)?;
    let replacement = match components.as_slice() {
        [x] => normalize_individual_scale_component(x)?,
        [x, y] => {
            let x = normalize_individual_scale_component(x)?;
            let y = normalize_individual_scale_component(y)?;
            if x == y { x } else { format!("{x} {y}") }
        }
        [x, y, z] => {
            let x = normalize_individual_scale_component(x)?;
            let y = normalize_individual_scale_component(y)?;
            let z = normalize_individual_scale_component(z)?;
            if z == "1" {
                if x == y { x } else { format!("{x} {y}") }
            } else {
                format!("{x} {y} {z}")
            }
        }
        _ => return None,
    };

    (replacement.len() < value.len()).then_some(replacement)
}

fn normalize_individual_scale_component(component: &str) -> Option<String> {
    let component = component.trim();
    if let Some(number) = component.strip_suffix('%') {
        let value = number.parse::<f64>().ok()?;
        if !value.is_finite() {
            return None;
        }
        return Some(compress_number_prefix(&format_css_number(value / 100.0)));
    }

    normalize_transform_number(component)
}

fn normalize_zero_angle_transform_function(value: &str) -> Option<String> {
    normalize_unary_zero_transform_function(value, is_css_angle_unit)
}

fn normalize_unary_zero_length_transform_function(value: &str) -> Option<String> {
    normalize_unary_zero_transform_function(value, is_css_length_unit)
}

fn normalize_unary_zero_length_percentage_transform_function(value: &str) -> Option<String> {
    normalize_unary_zero_transform_function(value, is_css_transform_length_percentage_unit)
}

fn normalize_translate_transform_function(value: &str) -> Option<String> {
    let arguments = transform_function_arguments(value)?;
    let replacement = match arguments.as_slice() {
        [x] if is_zero_transform_length_percentage_argument(x) => "translate(0)",
        [x, y]
            if is_zero_transform_length_percentage_argument(x)
                && is_zero_transform_length_percentage_argument(y) =>
        {
            "translate(0)"
        }
        [x, y] if is_zero_transform_length_percentage_argument(y) => {
            return Some(format!("translate({x})"));
        }
        _ => return None,
    }
    .to_string();

    (replacement != value).then_some(replacement)
}

fn normalize_translate_x_zero_transform_function(value: &str) -> Option<String> {
    zero_unary_transform_function_name(value, is_css_transform_length_percentage_unit)?;
    let replacement = "translate(0)".to_string();
    (replacement != value).then_some(replacement)
}

fn normalize_translate3d_transform_function(value: &str) -> Option<String> {
    let arguments = transform_function_arguments(value)?;
    let [x, y, z] = arguments.as_slice() else {
        return None;
    };
    let x_zero = is_zero_transform_length_percentage_argument(x);
    let y_zero = is_zero_transform_length_percentage_argument(y);
    let z_zero = is_zero_transform_numeric_unit_argument(z.trim(), is_css_length_unit).is_some();
    let normalized_x = if x_zero { "0" } else { x.as_str() };
    let normalized_y = if y_zero { "0" } else { y.as_str() };
    let normalized_z = if z_zero { "0" } else { z.as_str() };

    let replacement = match (x_zero, y_zero, z_zero) {
        (true, true, true) => "translate(0,0)".to_string(),
        (false, true, true) => format!("translate({normalized_x})"),
        (true, false, true) => format!("translateY({normalized_y})"),
        (false, false, true) => format!("translate({normalized_x},{normalized_y})"),
        (true, true, false) => format!("translateZ({normalized_z})"),
        _ => format!("translate3d({normalized_x},{normalized_y},{normalized_z})"),
    };

    (replacement != value && replacement.len() <= value.len()).then_some(replacement)
}

fn normalize_skew_transform_function(value: &str) -> Option<String> {
    let arguments = transform_function_arguments(value)?;
    let [x, y] = arguments.as_slice() else {
        return None;
    };
    if !is_zero_transform_angle_argument(y) {
        return None;
    }

    let replacement = format!("skew({x})");
    (replacement.len() < value.len()).then_some(replacement)
}

fn normalize_skew_x_zero_transform_function(value: &str) -> Option<String> {
    zero_unary_transform_function_name(value, is_css_angle_unit)?;
    let replacement = "skew(0)".to_string();
    (replacement != value).then_some(replacement)
}

fn normalize_scale_transform_function(value: &str) -> Option<String> {
    let arguments = transform_function_arguments(value)?;
    let [first, second] = arguments.as_slice() else {
        return None;
    };
    let first = normalize_transform_number(first)?;
    let second = normalize_transform_number(second)?;
    let replacement = if first == second {
        format!("scale({first})")
    } else if first == "1" {
        format!("scaleY({second})")
    } else if second == "1" {
        format!("scaleX({first})")
    } else {
        return None;
    };
    (replacement.len() < value.len()).then_some(replacement)
}

fn normalize_scale3d_transform_function(value: &str) -> Option<String> {
    let arguments = transform_function_arguments(value)?;
    let [x, y, z] = arguments.as_slice() else {
        return None;
    };
    let x = normalize_transform_number(x)?;
    let y = normalize_transform_number(y)?;
    let z = normalize_transform_number(z)?;

    let replacement = if z == "1" {
        if x == y {
            format!("scale({x})")
        } else {
            format!("scale({x},{y})")
        }
    } else if x == "1" && y == "1" {
        format!("scaleZ({z})")
    } else {
        return None;
    };

    (replacement.len() < value.len()).then_some(replacement)
}

fn normalize_rotate3d_transform_function(value: &str) -> Option<String> {
    let arguments = transform_function_arguments(value)?;
    let [x, y, z, angle] = arguments.as_slice() else {
        return None;
    };
    let x = normalize_transform_number(x)?;
    let y = normalize_transform_number(y)?;
    let z = normalize_transform_number(z)?;
    let angle = normalize_zero_transform_angle_argument_text(angle);

    let replacement = match (x.as_str(), y.as_str(), z.as_str()) {
        ("1", "0", "0") => format!("rotateX({angle})"),
        ("0", "1", "0") => format!("rotateY({angle})"),
        ("0", "0", "1") => format!("rotate({angle})"),
        _ => return None,
    };

    (replacement.len() < value.len()).then_some(replacement)
}

fn normalize_zero_transform_angle_argument_text(value: &str) -> String {
    if is_zero_transform_angle_argument(value) {
        "0".to_string()
    } else {
        value.trim().to_string()
    }
}

fn normalize_transform_number(value: &str) -> Option<String> {
    let value = value.trim();
    let split = numeric_prefix_end(value)?;
    if split != value.len() {
        return None;
    }
    let parsed = value.parse::<f64>().ok()?;
    parsed
        .is_finite()
        .then(|| compress_number_prefix(&format_css_number(parsed)))
}

fn normalize_unary_zero_transform_function(
    value: &str,
    is_unit: fn(&str) -> bool,
) -> Option<String> {
    let function_name = zero_unary_transform_function_name(value, is_unit)?;
    let replacement = format!("{function_name}(0)");
    (replacement != value).then_some(replacement)
}

fn zero_unary_transform_function_name(value: &str, is_unit: fn(&str) -> bool) -> Option<&str> {
    let open_index = value.find('(')?;
    let function_name = value.get(..open_index)?;
    let inner = value
        .get(open_index + 1..value.len().checked_sub(1)?)?
        .trim();
    if inner.contains(',') {
        return None;
    }
    is_zero_transform_numeric_unit_argument(inner, is_unit)?;

    Some(function_name)
}

fn is_css_transform_length_percentage_unit(unit: &str) -> bool {
    unit == "%" || is_css_length_unit(unit)
}

fn transform_function_arguments(value: &str) -> Option<Vec<String>> {
    let open_index = value.find('(')?;
    split_top_level_value_arguments(value.get(open_index + 1..value.len().checked_sub(1)?)?)
}

fn is_zero_transform_length_percentage_argument(value: &str) -> bool {
    is_zero_transform_numeric_unit_argument(value.trim(), is_css_transform_length_percentage_unit)
        .is_some()
}

fn is_zero_transform_angle_argument(value: &str) -> bool {
    is_zero_transform_numeric_unit_argument(value.trim(), is_css_angle_unit).is_some()
}

fn is_zero_transform_numeric_unit_argument(value: &str, is_unit: fn(&str) -> bool) -> Option<()> {
    let split = numeric_prefix_end(value)?;
    let (number, unit) = value.split_at(split);
    (is_zero_number_prefix(number) && (unit.is_empty() || is_unit(unit))).then_some(())
}

fn is_zero_number_prefix(number: &str) -> bool {
    css_number_is_zero(number)
}
