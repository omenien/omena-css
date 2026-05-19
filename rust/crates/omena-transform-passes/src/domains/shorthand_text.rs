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

pub(crate) fn text_emphasis_shorthand_replacement_for_declarations(
    tokens: &[LexedToken],
    declarations: &[SimpleDeclarationSlice],
) -> Option<(usize, usize, String)> {
    let [style, color] = declarations else {
        return None;
    };
    if style.property != "text-emphasis-style"
        || color.property != "text-emphasis-color"
        || style.important != color.important
        || !declaration_ranges_are_adjacent(tokens, declarations)
    {
        return None;
    }
    let style_value = text_emphasis_style_without_important(&style.value, style.important)?;
    let color_value = single_component_value_without_important(&color.value, color.important)?;
    let shorthand_value = compressed_text_emphasis_components(&style_value, &color_value)?;
    let important = if style.important { "!important" } else { "" };

    Some((
        style.start,
        color.end,
        format!("text-emphasis: {shorthand_value}{important};"),
    ))
}

pub(crate) fn collect_text_emphasis_replacements(
    tokens: &[LexedToken],
    declarations: &[SimpleDeclarationSlice],
) -> Vec<(usize, usize, String)> {
    declarations
        .windows(2)
        .filter_map(|pair| text_emphasis_shorthand_replacement_for_declarations(tokens, pair))
        .collect()
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

pub(crate) fn compress_text_emphasis_position_value(
    value: &str,
    important: bool,
) -> Option<String> {
    let mut components = split_top_level_whitespace_value_components(value)?;
    if important
        && components.last().is_some_and(|component| {
            component.eq_ignore_ascii_case("!important")
                || component.eq_ignore_ascii_case("important")
        })
    {
        components.pop();
    }

    let replacement = match components.as_slice() {
        [first, second] => {
            let first = first.to_ascii_lowercase();
            let second = second.to_ascii_lowercase();
            if is_text_emphasis_over_under(&first) && is_text_emphasis_side(&second) {
                compressed_text_emphasis_position(&first, &second)?
            } else if is_text_emphasis_side(&first) && is_text_emphasis_over_under(&second) {
                compressed_text_emphasis_position(&second, &first)?
            } else {
                return None;
            }
        }
        _ => return None,
    };

    (replacement != normalize_ascii_whitespace(value)).then_some(replacement)
}

fn text_emphasis_style_without_important(value: &str, important: bool) -> Option<String> {
    let mut components = split_top_level_whitespace_value_components(value)?;
    if important
        && components.last().is_some_and(|component| {
            component.eq_ignore_ascii_case("!important")
                || component.eq_ignore_ascii_case("important")
        })
    {
        components.pop();
    }
    if components.is_empty() || components.len() > 2 {
        return None;
    }
    Some(components.join(" "))
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

fn compressed_text_emphasis_position(vertical: &str, side: &str) -> Option<String> {
    if side == "right" {
        Some(vertical.to_string())
    } else if side == "left" {
        Some(format!("{vertical} left"))
    } else {
        None
    }
}

fn is_text_emphasis_over_under(value: &str) -> bool {
    matches!(value, "over" | "under")
}

fn is_text_emphasis_side(value: &str) -> bool {
    matches!(value, "left" | "right")
}

fn compressed_text_emphasis_components(style: &str, color: &str) -> Option<String> {
    let style = compressed_text_emphasis_style(style)?;
    let color = color.to_ascii_lowercase();
    if !is_text_decoration_color_component(&color) {
        return None;
    }
    if color == "currentcolor" {
        Some(style)
    } else {
        Some(format!("{style} {color}"))
    }
}

fn compressed_text_emphasis_style(value: &str) -> Option<String> {
    let components = split_top_level_whitespace_value_components(value)?;
    let components = components
        .into_iter()
        .map(|component| component.to_ascii_lowercase())
        .collect::<Vec<_>>();
    match components
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>()
        .as_slice()
    {
        ["none"] => Some("none".to_string()),
        [mark] if is_text_emphasis_mark(mark) => Some(mark.to_string()),
        ["filled", mark] if is_text_emphasis_mark(mark) => Some(mark.to_string()),
        ["open", mark] if is_text_emphasis_mark(mark) => Some(format!("open {mark}")),
        _ => None,
    }
}

fn is_text_emphasis_mark(value: &str) -> bool {
    matches!(
        value,
        "dot" | "circle" | "double-circle" | "triangle" | "sesame"
    )
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
