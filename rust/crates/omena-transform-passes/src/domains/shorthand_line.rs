use omena_parser::LexedToken;

use crate::helpers::{
    declarations::{SimpleDeclarationSlice, declaration_ranges_are_adjacent},
    values::split_top_level_whitespace_value_components,
};

pub(crate) fn line_shorthand_replacement_for_declarations(
    tokens: &[LexedToken],
    declarations: &[SimpleDeclarationSlice],
) -> Option<(usize, usize, String)> {
    let [width, style, color] = declarations else {
        return None;
    };
    let shorthand = line_shorthand_property_for_width_longhand(&width.property)?;
    if style.property != format!("{shorthand}-style")
        || color.property != format!("{shorthand}-color")
        || width.important != style.important
        || width.important != color.important
        || !declaration_ranges_are_adjacent(tokens, declarations)
    {
        return None;
    }

    let width_value = single_component_value_without_important(&width.value, width.important)?;
    let style_value = single_component_value_without_important(&style.value, style.important)?;
    let color_value = single_component_value_without_important(&color.value, color.important)?;
    let shorthand_value =
        compressed_line_shorthand_value(&width_value, &style_value, &color_value)?;
    let important = if width.important { "!important" } else { "" };

    Some((
        width.start,
        color.end,
        format!("{shorthand}: {shorthand_value}{important};"),
    ))
}

fn line_shorthand_property_for_width_longhand(property: &str) -> Option<&'static str> {
    match property {
        "border-width" => Some("border"),
        "border-top-width" => Some("border-top"),
        "border-right-width" => Some("border-right"),
        "border-bottom-width" => Some("border-bottom"),
        "border-left-width" => Some("border-left"),
        "border-block-width" => Some("border-block"),
        "border-inline-width" => Some("border-inline"),
        "outline-width" => Some("outline"),
        _ => None,
    }
}

fn single_component_value_without_important(value: &str, important: bool) -> Option<String> {
    let mut components = split_top_level_whitespace_value_components(value)?;
    if important
        && components.last().is_some_and(|component| {
            component.eq_ignore_ascii_case("!important")
                || component.eq_ignore_ascii_case("important")
        })
    {
        components.pop();
    }
    let [component] = components.as_slice() else {
        return None;
    };
    Some(component.clone())
}

fn compressed_line_shorthand_value(width: &str, style: &str, color: &str) -> Option<String> {
    let width = width.to_ascii_lowercase();
    let style = style.to_ascii_lowercase();
    let color = color.to_ascii_lowercase();
    if !is_border_line_style(&style) {
        return None;
    }

    let mut components = Vec::new();
    if width != "medium" {
        components.push(width);
    }
    if style != "none" {
        components.push(style);
    }
    if color != "currentcolor" {
        components.push(color);
    }
    if components.is_empty() {
        components.push("none".to_string());
    }
    Some(components.join(" "))
}

fn is_border_line_style(value: &str) -> bool {
    matches!(
        value,
        "none"
            | "hidden"
            | "dotted"
            | "dashed"
            | "solid"
            | "double"
            | "groove"
            | "ridge"
            | "inset"
            | "outset"
    )
}
