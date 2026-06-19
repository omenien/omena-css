use omena_value_lattice::{
    NumericValueV0, parse_numeric_value_with_unit, parse_whole_function_value_arguments,
    split_top_level_whitespace_value_components_owned,
};

use super::less_strings::static_less_quoted_string_contents;
use super::{
    format_static_less_channel_number, format_static_less_math_number, format_static_less_number,
};

pub(super) fn parse_static_less_unit_value(value: &str) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, "unit")?;
    match arguments.as_slice() {
        [number] => {
            let parsed = parse_numeric_value_with_unit(number.trim())?;
            Some(format_static_less_number(parsed.value))
        }
        [number, unit] => {
            let parsed = parse_numeric_value_with_unit(number.trim())?;
            let unit = parse_static_less_unit_argument(unit.trim())?;
            Some(format!(
                "{}{}",
                format_static_less_number(parsed.value),
                unit
            ))
        }
        _ => None,
    }
}

pub(super) fn parse_static_less_get_unit_value(value: &str) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, "get-unit")?;
    let [number] = arguments.as_slice() else {
        return None;
    };
    let parsed = parse_numeric_value_with_unit(number.trim())?;
    Some(parsed.unit.to_string())
}

pub(super) fn parse_static_less_convert_value(value: &str) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, "convert")?;
    let [number, target_unit] = arguments.as_slice() else {
        return None;
    };
    let parsed = parse_numeric_value_with_unit(number.trim())?;
    let target_unit = parse_static_less_convert_unit_argument(target_unit.trim())?;
    let original = || {
        format!(
            "{}{}",
            format_static_less_channel_number(parsed.value),
            parsed.unit
        )
    };
    let Some(source_unit) = static_less_convertible_unit(parsed.unit) else {
        return Some(original());
    };
    let Some(target_unit) = static_less_convertible_unit(target_unit.as_str()) else {
        return Some(original());
    };
    if source_unit.family != target_unit.family {
        return Some(original());
    }
    let converted = parsed.value * source_unit.base_factor / target_unit.base_factor;
    Some(format!(
        "{}{}",
        format_static_less_channel_number(converted),
        target_unit.unit
    ))
}

pub(super) fn parse_static_less_percentage_value(value: &str) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, "percentage")?;
    let [number] = arguments.as_slice() else {
        return None;
    };
    let parsed = parse_numeric_value_with_unit(number.trim())?;
    Some(format!(
        "{}%",
        format_static_less_number(parsed.value * 100.0)
    ))
}

pub(super) fn parse_static_less_round_value(value: &str) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, "round")?;
    let (number, decimal_places) = match arguments.as_slice() {
        [number] => (number.as_str(), 0usize),
        [number, decimal_places] => {
            let decimal_places = parse_static_less_unitless_integer(decimal_places.trim())?;
            (number.as_str(), decimal_places)
        }
        _ => return None,
    };
    let parsed = parse_numeric_value_with_unit(number.trim())?;
    let factor = 10_f64.powi(i32::try_from(decimal_places).ok()?);
    let rounded = (parsed.value * factor).round() / factor;
    Some(format!(
        "{}{}",
        format_static_less_number(rounded),
        parsed.unit
    ))
}

pub(super) fn parse_static_less_pi_value(value: &str) -> Option<String> {
    value
        .trim()
        .eq_ignore_ascii_case("pi()")
        .then(|| format_static_less_math_number(std::f64::consts::PI))
        .flatten()
}

pub(super) fn parse_static_less_sin_value(value: &str) -> Option<String> {
    parse_static_less_trig_value(value, "sin", f64::sin)
}

pub(super) fn parse_static_less_cos_value(value: &str) -> Option<String> {
    parse_static_less_trig_value(value, "cos", f64::cos)
}

pub(super) fn parse_static_less_tan_value(value: &str) -> Option<String> {
    parse_static_less_trig_value(value, "tan", f64::tan)
}

pub(super) fn parse_static_less_asin_value(value: &str) -> Option<String> {
    parse_static_less_inverse_trig_value(value, "asin", f64::asin, true)
}

pub(super) fn parse_static_less_acos_value(value: &str) -> Option<String> {
    parse_static_less_inverse_trig_value(value, "acos", f64::acos, true)
}

pub(super) fn parse_static_less_atan_value(value: &str) -> Option<String> {
    parse_static_less_inverse_trig_value(value, "atan", f64::atan, false)
}

fn parse_static_less_trig_value(
    value: &str,
    function_name: &str,
    evaluate: fn(f64) -> f64,
) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, function_name)?;
    let [angle] = arguments.as_slice() else {
        return None;
    };
    format_static_less_math_number(evaluate(parse_static_less_angle_radians(angle.trim())?))
}

fn parse_static_less_inverse_trig_value(
    value: &str,
    function_name: &str,
    evaluate: fn(f64) -> f64,
    requires_unit_interval: bool,
) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, function_name)?;
    let [number] = arguments.as_slice() else {
        return None;
    };
    let parsed = parse_numeric_value_with_unit(number.trim())?;
    if !parsed.unit.is_empty() {
        return None;
    }
    if requires_unit_interval && !(-1.0..=1.0).contains(&parsed.value) {
        return None;
    }
    let radians = evaluate(parsed.value);
    if !radians.is_finite() {
        return None;
    }
    Some(format!("{}rad", format_static_less_math_number(radians)?))
}

fn parse_static_less_angle_radians(value: &str) -> Option<f64> {
    let parsed = parse_numeric_value_with_unit(value)?;
    if !parsed.value.is_finite() {
        return None;
    }
    match parsed.unit {
        "" | "rad" => Some(parsed.value),
        "deg" => Some(parsed.value.to_radians()),
        "grad" => Some(parsed.value * std::f64::consts::PI / 200.0),
        "turn" => Some(parsed.value * std::f64::consts::TAU),
        _ => None,
    }
}

pub(super) fn parse_static_less_length_value(value: &str) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, "length")?;
    let items = static_less_list_items_from_arguments(arguments.as_slice())?;
    Some(items.len().to_string())
}

pub(super) fn parse_static_less_extract_value(value: &str) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, "extract")?;
    if arguments.len() < 2 {
        return None;
    }
    let (index, list_arguments) = arguments.split_last()?;
    let index = parse_static_less_unitless_integer(index.trim())?;
    let items = static_less_list_items_from_arguments(list_arguments)?;
    items.get(index.checked_sub(1)?).cloned()
}

pub(super) fn parse_static_less_range_value(value: &str) -> Option<String> {
    const MAX_STATIC_LESS_RANGE_ITEMS: usize = 1024;

    let arguments = parse_whole_function_value_arguments(value, "range")?;
    let (start, end, step) = match arguments.as_slice() {
        [end] => {
            let end = parse_numeric_value_with_unit(end.trim())?;
            let start = StaticLessRangeEndpoint {
                value: 1.0,
                unit: end.unit,
            };
            let step = StaticLessRangeEndpoint {
                value: 1.0,
                unit: "",
            };
            (start, static_less_range_endpoint_from_numeric(end)?, step)
        }
        [start, end] => {
            let start = static_less_range_endpoint(start.trim())?;
            let end = static_less_range_endpoint(end.trim())?;
            let step = StaticLessRangeEndpoint {
                value: 1.0,
                unit: "",
            };
            (start, end, step)
        }
        [start, end, step] => (
            static_less_range_endpoint(start.trim())?,
            static_less_range_endpoint(end.trim())?,
            static_less_range_endpoint(step.trim())?,
        ),
        _ => return None,
    };

    if step.value <= 0.0 {
        return None;
    }
    if start.value > end.value {
        return Some(String::new());
    }

    let mut items = Vec::new();
    let mut current = start.value;
    while current <= end.value + f64::EPSILON {
        if items.len() >= MAX_STATIC_LESS_RANGE_ITEMS {
            return None;
        }
        items.push(format!(
            "{}{}",
            format_static_less_number(current),
            end.unit
        ));
        current += step.value;
    }
    Some(items.join(" "))
}

#[derive(Debug, Clone, Copy)]
struct StaticLessRangeEndpoint<'a> {
    value: f64,
    unit: &'a str,
}

fn static_less_range_endpoint(value: &str) -> Option<StaticLessRangeEndpoint<'_>> {
    static_less_range_endpoint_from_numeric(parse_numeric_value_with_unit(value)?)
}

fn static_less_range_endpoint_from_numeric(
    parsed: NumericValueV0<'_>,
) -> Option<StaticLessRangeEndpoint<'_>> {
    parsed.value.is_finite().then_some(StaticLessRangeEndpoint {
        value: parsed.value,
        unit: parsed.unit,
    })
}

fn static_less_list_items_from_arguments(arguments: &[String]) -> Option<Vec<String>> {
    if arguments.len() == 1 {
        return split_top_level_whitespace_value_components_owned(arguments[0].as_str())
            .filter(|items| !items.is_empty());
    }
    Some(
        arguments
            .iter()
            .map(|argument| argument.trim().to_string())
            .filter(|argument| !argument.is_empty())
            .collect::<Vec<_>>(),
    )
    .filter(|items| !items.is_empty())
}

fn parse_static_less_unitless_integer(value: &str) -> Option<usize> {
    let parsed = parse_numeric_value_with_unit(value)?;
    if !parsed.unit.is_empty() || !parsed.value.is_finite() || parsed.value.fract() != 0.0 {
        return None;
    }
    usize::try_from(parsed.value as i64).ok()
}

fn parse_static_less_unit_argument(unit: &str) -> Option<&str> {
    if unit == "%" {
        return Some(unit);
    }
    if unit.is_empty()
        || !unit
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_')
    {
        return None;
    }
    Some(unit)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StaticLessConvertibleUnitFamily {
    Length,
    Time,
    Angle,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct StaticLessConvertibleUnit {
    family: StaticLessConvertibleUnitFamily,
    unit: &'static str,
    base_factor: f64,
}

fn static_less_convertible_unit(unit: &str) -> Option<StaticLessConvertibleUnit> {
    match unit {
        "px" => Some(static_less_convertible_length_unit("px", 1.0)),
        "in" => Some(static_less_convertible_length_unit("in", 96.0)),
        "cm" => Some(static_less_convertible_length_unit("cm", 96.0 / 2.54)),
        "mm" => Some(static_less_convertible_length_unit("mm", 96.0 / 25.4)),
        "pt" => Some(static_less_convertible_length_unit("pt", 96.0 / 72.0)),
        "pc" => Some(static_less_convertible_length_unit("pc", 16.0)),
        "s" => Some(StaticLessConvertibleUnit {
            family: StaticLessConvertibleUnitFamily::Time,
            unit: "s",
            base_factor: 1.0,
        }),
        "ms" => Some(StaticLessConvertibleUnit {
            family: StaticLessConvertibleUnitFamily::Time,
            unit: "ms",
            base_factor: 0.001,
        }),
        "deg" => Some(static_less_convertible_angle_unit("deg", 1.0)),
        "rad" => Some(static_less_convertible_angle_unit(
            "rad",
            180.0 / std::f64::consts::PI,
        )),
        "grad" => Some(static_less_convertible_angle_unit("grad", 0.9)),
        "turn" => Some(static_less_convertible_angle_unit("turn", 360.0)),
        _ => None,
    }
}

fn static_less_convertible_length_unit(
    unit: &'static str,
    base_factor: f64,
) -> StaticLessConvertibleUnit {
    StaticLessConvertibleUnit {
        family: StaticLessConvertibleUnitFamily::Length,
        unit,
        base_factor,
    }
}

fn static_less_convertible_angle_unit(
    unit: &'static str,
    base_factor: f64,
) -> StaticLessConvertibleUnit {
    StaticLessConvertibleUnit {
        family: StaticLessConvertibleUnitFamily::Angle,
        unit,
        base_factor,
    }
}

fn parse_static_less_convert_unit_argument(unit: &str) -> Option<String> {
    static_less_quoted_string_contents(unit).or_else(|| {
        parse_static_less_unit_argument(unit)
            .map(str::to_string)
            .filter(|unit| unit != "%")
    })
}
