use omena_parser::LexedToken;

use crate::{
    domains::number::numeric_prefix_end,
    helpers::{
        ascii::normalize_ascii_whitespace,
        declarations::{SimpleDeclarationSlice, declaration_ranges_are_adjacent},
        values::split_top_level_whitespace_value_components,
    },
};

pub(crate) fn text_decoration_shorthand_replacement_for_declarations(
    tokens: &[LexedToken],
    declarations: &[SimpleDeclarationSlice],
) -> Option<(usize, usize, String)> {
    let [line, style, color, thickness] = declarations else {
        return None;
    };
    if line.property != "text-decoration-line"
        || style.property != "text-decoration-style"
        || color.property != "text-decoration-color"
        || thickness.property != "text-decoration-thickness"
        || declarations
            .iter()
            .any(|declaration| declaration.important != line.important)
        || !declaration_ranges_are_adjacent(tokens, declarations)
    {
        return None;
    }
    let line_value = single_component_value_without_important(&line.value, line.important)?;
    let style_value = single_component_value_without_important(&style.value, style.important)?;
    let color_value = single_component_value_without_important(&color.value, color.important)?;
    let thickness_value =
        single_component_value_without_important(&thickness.value, thickness.important)?;
    let shorthand_value = compressed_text_decoration_components(
        &line_value,
        &style_value,
        &color_value,
        &thickness_value,
    )?;
    let important = if line.important { "!important" } else { "" };
    Some((
        line.start,
        thickness.end,
        format!("text-decoration: {shorthand_value}{important};"),
    ))
}

pub(crate) fn compress_text_decoration_value(value: &str, important: bool) -> Option<String> {
    let mut components = split_top_level_whitespace_value_components(value)?;
    if important
        && components.last().is_some_and(|component| {
            component.eq_ignore_ascii_case("!important")
                || component.eq_ignore_ascii_case("important")
        })
    {
        components.pop();
    }

    let mut line = None;
    let mut style = None;
    let mut color = None;
    let mut thickness = None;
    for component in &components {
        let normalized = component.to_ascii_lowercase();
        if is_text_decoration_line_component(&normalized) && line.is_none() {
            line = Some(normalized);
        } else if is_text_decoration_style_component(&normalized) && style.is_none() {
            style = Some(normalized);
        } else if is_text_decoration_color_component(component) && color.is_none() {
            color = Some(normalized);
        } else if is_text_decoration_thickness_component(&normalized) && thickness.is_none() {
            thickness = Some(normalized);
        } else {
            return None;
        }
    }

    let replacement = compressed_text_decoration_components(
        line.as_deref()?,
        style.as_deref().unwrap_or("solid"),
        color.as_deref().unwrap_or("currentcolor"),
        thickness.as_deref().unwrap_or("auto"),
    )?;
    (replacement != normalize_ascii_whitespace(value)).then_some(replacement)
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

fn compressed_text_decoration_components(
    line: &str,
    style: &str,
    color: &str,
    thickness: &str,
) -> Option<String> {
    let line = line.to_ascii_lowercase();
    let style = style.to_ascii_lowercase();
    let color = color.to_ascii_lowercase();
    let thickness = thickness.to_ascii_lowercase();

    if !is_text_decoration_line_component(&line)
        || !is_text_decoration_style_component(&style)
        || !is_text_decoration_thickness_component(&thickness)
    {
        return None;
    }

    let mut components = vec![line];
    if thickness != "auto" {
        components.push(thickness);
    }
    if style != "solid" {
        components.push(style);
    }
    if color != "currentcolor" {
        components.push(color);
    }
    Some(components.join(" "))
}

fn is_text_decoration_line_component(value: &str) -> bool {
    matches!(value, "none" | "underline" | "overline" | "line-through")
}

fn is_text_decoration_style_component(value: &str) -> bool {
    matches!(value, "solid" | "double" | "dotted" | "dashed" | "wavy")
}

fn is_text_decoration_thickness_component(value: &str) -> bool {
    value == "auto"
        || value == "from-font"
        || numeric_prefix_end(value).is_some_and(|end| {
            value
                .get(end..)
                .is_some_and(is_text_decoration_thickness_unit)
        })
}

fn is_text_decoration_thickness_unit(unit: &str) -> bool {
    matches!(
        unit.to_ascii_lowercase().as_str(),
        "px" | "em" | "rem" | "ch" | "ex" | "lh" | "rlh" | "vw" | "vh" | "vmin" | "vmax" | "%"
    )
}

fn is_text_decoration_color_component(value: &str) -> bool {
    let normalized = value.to_ascii_lowercase();
    if matches!(
        normalized.as_str(),
        "inherit" | "initial" | "revert" | "revert-layer" | "unset"
    ) {
        return false;
    }
    normalized == "currentcolor"
        || normalized.starts_with('#')
        || normalized.starts_with("rgb(")
        || normalized.starts_with("rgba(")
        || normalized.starts_with("hsl(")
        || normalized.starts_with("hsla(")
        || normalized.chars().all(|character| {
            character.is_ascii_alphabetic() || character == '-' || character.is_ascii_digit()
        })
}
