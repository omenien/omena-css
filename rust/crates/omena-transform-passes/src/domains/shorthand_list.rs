use omena_parser::LexedToken;

use crate::helpers::{
    ascii::normalize_ascii_whitespace,
    declarations::{SimpleDeclarationSlice, declaration_ranges_are_adjacent},
    values::split_top_level_whitespace_value_components,
};

pub(crate) fn list_style_shorthand_replacement_for_declarations(
    tokens: &[LexedToken],
    declarations: &[SimpleDeclarationSlice],
) -> Option<(usize, usize, String)> {
    let [style_type, position, image] = declarations else {
        return None;
    };
    if style_type.property != "list-style-type"
        || position.property != "list-style-position"
        || image.property != "list-style-image"
        || declarations.iter().any(|declaration| declaration.important)
        || !declaration_ranges_are_adjacent(tokens, declarations)
    {
        return None;
    }
    let shorthand_value =
        compressed_list_style_components(&style_type.value, &position.value, &image.value)?;
    Some((
        style_type.start,
        image.end,
        format!("list-style: {shorthand_value};"),
    ))
}

pub(crate) fn compress_list_style_value(value: &str) -> Option<String> {
    let components = split_top_level_whitespace_value_components(value)?;
    let mut style_type = None;
    let mut position = "outside".to_string();
    let mut image = None;

    for component in &components {
        if is_list_style_position(component) {
            position = component.clone();
        } else if component == "none" {
            if style_type.is_none() {
                style_type = Some(component.clone());
            } else if image.is_none() {
                image = Some(component.clone());
            } else {
                return None;
            }
        } else if is_list_style_image(component) && image.is_none() {
            image = Some(component.clone());
        } else if is_list_style_type(component) && style_type.is_none() {
            style_type = Some(component.clone());
        } else {
            return None;
        }
    }

    let style_type = style_type.unwrap_or_else(|| "disc".to_string());
    let image = image.unwrap_or_else(|| "none".to_string());
    let compressed = compressed_list_style_components(&style_type, &position, &image)?;
    (compressed != normalize_ascii_whitespace(value)).then_some(compressed)
}

fn compressed_list_style_components(
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
    if style_type == "none" {
        return Some(if position == "outside" {
            format!("{image} none")
        } else {
            format!("{position} {image} none")
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
