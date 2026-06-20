use omena_value_lattice::{
    number::format_css_number, parse_numeric_value_with_unit, parse_whole_function_value_arguments,
};

use super::reduce_static_scss_value;

pub(super) fn parse_static_scss_percentage_value(value: &str) -> Option<String> {
    parse_static_scss_percentage_value_with_name(value, "percentage")
}

pub(super) fn parse_static_scss_math_percentage_value(value: &str) -> Option<String> {
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

pub(super) fn parse_static_scss_unit_value(value: &str) -> Option<String> {
    parse_static_scss_unit_value_with_name(value, "unit")
}

pub(super) fn parse_static_scss_math_unit_value(value: &str) -> Option<String> {
    parse_static_scss_unit_value_with_name(value, "math.unit")
}

pub(super) fn parse_static_scss_unitless_value(value: &str) -> Option<String> {
    parse_static_scss_unitless_value_with_name(value, "unitless")
}

pub(super) fn parse_static_scss_math_is_unitless_value(value: &str) -> Option<String> {
    parse_static_scss_unitless_value_with_name(value, "math.is-unitless")
}

pub(super) fn parse_static_scss_comparable_value(value: &str) -> Option<String> {
    parse_static_scss_compatible_value_with_name(value, "comparable")
}

pub(super) fn parse_static_scss_math_compatible_value(value: &str) -> Option<String> {
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
