use omena_parser::LexedToken;

use crate::helpers::{
    ascii::normalize_ascii_whitespace,
    declarations::{SimpleDeclarationSlice, declaration_ranges_are_adjacent},
    values::split_top_level_whitespace_value_components,
};

pub(crate) fn border_side_shorthand_replacement_for_declarations(
    tokens: &[LexedToken],
    declarations: &[SimpleDeclarationSlice],
) -> Option<(usize, usize, String)> {
    let [top, right, bottom, left] = declarations else {
        return None;
    };
    if top.property != "border-top"
        || right.property != "border-right"
        || bottom.property != "border-bottom"
        || left.property != "border-left"
        || !declaration_ranges_are_adjacent(tokens, declarations)
    {
        return None;
    }

    let important = top.important;
    let values = declarations
        .iter()
        .map(|declaration| {
            if declaration.important != important {
                return None;
            }
            normalized_declaration_value_without_important(declaration)
        })
        .collect::<Option<Vec<_>>>()?;
    let [top_value, right_value, bottom_value, left_value] = values.as_slice() else {
        return None;
    };
    if top_value != right_value || top_value != bottom_value || top_value != left_value {
        return None;
    }

    let important = if important { "!important" } else { "" };
    Some((
        top.start,
        left.end,
        format!("border: {top_value}{important};"),
    ))
}

pub(crate) fn line_shorthand_replacement_for_declarations(
    tokens: &[LexedToken],
    declarations: &[SimpleDeclarationSlice],
) -> Option<(usize, usize, String)> {
    let [first, _, last] = declarations else {
        return None;
    };
    if !declaration_ranges_are_adjacent(tokens, declarations) {
        return None;
    }

    let important = first.important;
    let mut shorthand = None;
    let mut width_value = None;
    let mut style_value = None;
    let mut color_value = None;

    for declaration in declarations {
        if declaration.important != important {
            return None;
        }
        let (declaration_shorthand, component) =
            line_shorthand_component_for_property(&declaration.property)?;
        if shorthand.is_some_and(|current_shorthand| current_shorthand != declaration_shorthand) {
            return None;
        }
        shorthand = Some(declaration_shorthand);

        let value = single_component_value_without_important(&declaration.value, important)?;
        let slot = match component {
            LineShorthandComponent::Width => &mut width_value,
            LineShorthandComponent::Style => &mut style_value,
            LineShorthandComponent::Color => &mut color_value,
        };
        if slot.replace(value).is_some() {
            return None;
        }
    }

    let shorthand = shorthand?;
    let width_value = width_value?;
    let style_value = style_value?;
    let color_value = color_value?;
    let shorthand_value =
        compressed_line_shorthand_value(&width_value, &style_value, &color_value)?;
    let important = if important { "!important" } else { "" };

    Some((
        first.start,
        last.end,
        format!("{shorthand}: {shorthand_value}{important};"),
    ))
}

enum LineShorthandComponent {
    Width,
    Style,
    Color,
}

fn line_shorthand_component_for_property(
    property: &str,
) -> Option<(&'static str, LineShorthandComponent)> {
    match property {
        "border-width" => Some(("border", LineShorthandComponent::Width)),
        "border-style" => Some(("border", LineShorthandComponent::Style)),
        "border-color" => Some(("border", LineShorthandComponent::Color)),
        "border-top-width" => Some(("border-top", LineShorthandComponent::Width)),
        "border-top-style" => Some(("border-top", LineShorthandComponent::Style)),
        "border-top-color" => Some(("border-top", LineShorthandComponent::Color)),
        "border-right-width" => Some(("border-right", LineShorthandComponent::Width)),
        "border-right-style" => Some(("border-right", LineShorthandComponent::Style)),
        "border-right-color" => Some(("border-right", LineShorthandComponent::Color)),
        "border-bottom-width" => Some(("border-bottom", LineShorthandComponent::Width)),
        "border-bottom-style" => Some(("border-bottom", LineShorthandComponent::Style)),
        "border-bottom-color" => Some(("border-bottom", LineShorthandComponent::Color)),
        "border-left-width" => Some(("border-left", LineShorthandComponent::Width)),
        "border-left-style" => Some(("border-left", LineShorthandComponent::Style)),
        "border-left-color" => Some(("border-left", LineShorthandComponent::Color)),
        "border-block-width" => Some(("border-block", LineShorthandComponent::Width)),
        "border-block-style" => Some(("border-block", LineShorthandComponent::Style)),
        "border-block-color" => Some(("border-block", LineShorthandComponent::Color)),
        "border-inline-width" => Some(("border-inline", LineShorthandComponent::Width)),
        "border-inline-style" => Some(("border-inline", LineShorthandComponent::Style)),
        "border-inline-color" => Some(("border-inline", LineShorthandComponent::Color)),
        "outline-width" => Some(("outline", LineShorthandComponent::Width)),
        "outline-style" => Some(("outline", LineShorthandComponent::Style)),
        "outline-color" => Some(("outline", LineShorthandComponent::Color)),
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

fn normalized_declaration_value_without_important(
    declaration: &SimpleDeclarationSlice,
) -> Option<String> {
    let mut components = split_top_level_whitespace_value_components(&declaration.value)?;
    if declaration.important
        && components.last().is_some_and(|component| {
            component.eq_ignore_ascii_case("!important")
                || component.eq_ignore_ascii_case("important")
        })
    {
        components.pop();
    }
    if components.is_empty() {
        return None;
    }
    Some(normalize_ascii_whitespace(&components.join(" ")))
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
