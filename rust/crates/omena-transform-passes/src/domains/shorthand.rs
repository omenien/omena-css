use crate::helpers::{
    ascii::normalize_ascii_whitespace, values::split_top_level_whitespace_value_components,
};

pub(crate) fn is_box_shorthand_property(property: &str) -> bool {
    matches!(
        property,
        "margin" | "padding" | "border-color" | "border-style" | "border-width"
    )
}

pub(crate) fn compress_background_repeat_value(value: &str) -> Option<String> {
    let components = split_top_level_whitespace_value_components(value)?;
    let [x, y] = components.as_slice() else {
        return None;
    };
    if x != y || !is_background_repeat_axis_keyword(x) {
        return None;
    }
    Some(x.clone())
}

fn is_background_repeat_axis_keyword(value: &str) -> bool {
    matches!(value, "repeat" | "no-repeat" | "space" | "round")
}

pub(crate) fn compress_border_radius_value(value: &str) -> Option<String> {
    let components = split_top_level_whitespace_value_components(value)?;
    if !(1..=4).contains(&components.len())
        || components
            .iter()
            .any(|component| !is_single_axis_border_radius_value(component))
    {
        return None;
    }
    let values = match components.as_slice() {
        [value] => [
            value.as_str(),
            value.as_str(),
            value.as_str(),
            value.as_str(),
        ],
        [top_left_bottom_right, top_right_bottom_left] => [
            top_left_bottom_right.as_str(),
            top_right_bottom_left.as_str(),
            top_left_bottom_right.as_str(),
            top_right_bottom_left.as_str(),
        ],
        [top_left, top_right_bottom_left, bottom_right] => [
            top_left.as_str(),
            top_right_bottom_left.as_str(),
            bottom_right.as_str(),
            top_right_bottom_left.as_str(),
        ],
        [top_left, top_right, bottom_right, bottom_left] => [
            top_left.as_str(),
            top_right.as_str(),
            bottom_right.as_str(),
            bottom_left.as_str(),
        ],
        _ => return None,
    };
    let compressed = compress_box_shorthand_values(&values)?;
    (compressed != normalize_ascii_whitespace(value)).then_some(compressed)
}

pub(crate) fn is_single_axis_border_radius_value(value: &str) -> bool {
    split_top_level_whitespace_value_components(value)
        .is_some_and(|components| components.len() == 1 && components[0] != "/")
}

pub(crate) fn compress_list_style_value(value: &str) -> Option<String> {
    let components = split_top_level_whitespace_value_components(value)?;
    let mut style_type = "disc".to_string();
    let mut position = "outside".to_string();
    let mut image = "none".to_string();

    for component in &components {
        if is_list_style_position(component) {
            position = component.clone();
        } else if is_list_style_image(component) {
            image = component.clone();
        } else if is_list_style_type(component) {
            style_type = component.clone();
        } else {
            return None;
        }
    }

    let compressed = compressed_list_style_components(&style_type, &position, &image)?;
    (compressed != normalize_ascii_whitespace(value)).then_some(compressed)
}

pub(crate) fn compressed_list_style_components(
    style_type: &str,
    position: &str,
    image: &str,
) -> Option<String> {
    if !is_list_style_type(style_type)
        || !is_list_style_position(position)
        || !is_list_style_image(image)
    {
        return None;
    }
    if style_type == "none" && image == "none" {
        return Some(if position == "outside" {
            "none".to_string()
        } else {
            format!("{position} none")
        });
    }

    let mut components = Vec::new();
    if position != "outside" || (style_type == "disc" && image == "none") {
        components.push(position.to_string());
    }
    if style_type != "disc" && !(style_type == "none" && image == "none") {
        components.push(style_type.to_string());
    }
    if image != "none" {
        components.push(image.to_string());
    }
    if components.is_empty() {
        components.push("outside".to_string());
    }
    Some(components.join(" "))
}

fn is_list_style_position(value: &str) -> bool {
    matches!(value, "inside" | "outside")
}

fn is_list_style_type(value: &str) -> bool {
    matches!(
        value,
        "disc"
            | "circle"
            | "square"
            | "decimal"
            | "decimal-leading-zero"
            | "lower-roman"
            | "upper-roman"
            | "lower-alpha"
            | "upper-alpha"
            | "none"
    )
}

fn is_list_style_image(value: &str) -> bool {
    value == "none"
        || value
            .get(.."url(".len())
            .is_some_and(|prefix| prefix.eq_ignore_ascii_case("url("))
}

pub(crate) fn compress_box_shorthand_value(value: &str) -> Option<String> {
    let components = split_top_level_whitespace_value_components(value)?;
    let [top, right, bottom, left] = match components.as_slice() {
        [value] => [value, value, value, value],
        [block, inline] => [block, inline, block, inline],
        [top, inline, bottom] => [top, inline, bottom, inline],
        [top, right, bottom, left] => [top, right, bottom, left],
        _ => return None,
    };
    let values = [top.as_str(), right.as_str(), bottom.as_str(), left.as_str()];
    let compressed = compress_box_shorthand_values(&values)?;
    (compressed != normalize_ascii_whitespace(value)).then_some(compressed)
}

pub(crate) fn compress_box_shorthand_values(values: &[&str]) -> Option<String> {
    let [top, right, bottom, left] = values else {
        return None;
    };

    let parts = if top == right && top == bottom && top == left {
        vec![*top]
    } else if top == bottom && right == left {
        vec![*top, *right]
    } else if right == left {
        vec![*top, *right, *bottom]
    } else {
        vec![*top, *right, *bottom, *left]
    };
    Some(parts.join(" "))
}

pub(crate) fn is_overflow_axis_keyword(value: &str) -> bool {
    matches!(value, "visible" | "hidden" | "clip" | "scroll" | "auto")
}
