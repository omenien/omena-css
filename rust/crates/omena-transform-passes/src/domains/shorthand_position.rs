use omena_parser::LexedToken;

use crate::helpers::{
    declarations::{SimpleDeclarationSlice, declaration_ranges_are_adjacent},
    values::split_top_level_whitespace_value_components,
};

pub(crate) fn collect_background_position_axis_replacements(
    tokens: &[LexedToken],
    declarations: &[SimpleDeclarationSlice],
) -> Vec<(usize, usize, String)> {
    let mut ranges = Vec::new();
    for pair in declarations.windows(2) {
        if let Some(replacement) =
            background_position_axis_replacement_for_declarations(tokens, pair)
        {
            ranges.push(replacement);
        }
    }
    ranges
}

pub(crate) fn background_position_axis_replacement_for_declarations(
    tokens: &[LexedToken],
    declarations: &[SimpleDeclarationSlice],
) -> Option<(usize, usize, String)> {
    let [first, second] = declarations else {
        return None;
    };
    if first.important != second.important || !declaration_ranges_are_adjacent(tokens, declarations)
    {
        return None;
    }

    let (x_value, y_value) = match (first.property.as_str(), second.property.as_str()) {
        ("background-position-x", "background-position-y") => {
            (first.value.as_str(), second.value.as_str())
        }
        ("background-position-y", "background-position-x") => {
            (second.value.as_str(), first.value.as_str())
        }
        _ => return None,
    };
    let x_component =
        background_position_axis_component(x_value, first.important, PositionAxis::Horizontal)?;
    let y_component =
        background_position_axis_component(y_value, second.important, PositionAxis::Vertical)?;
    let shorthand_value = compressed_background_position_axis_value(&x_component, &y_component);
    let important = if first.important { "!important" } else { "" };

    Some((
        first.start,
        second.end,
        format!("background-position: {shorthand_value}{important};"),
    ))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PositionAxis {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct BackgroundPositionAxisComponent {
    value: String,
    keyword: Option<&'static str>,
}

fn background_position_axis_component(
    value: &str,
    important: bool,
    axis: PositionAxis,
) -> Option<BackgroundPositionAxisComponent> {
    let component = single_component_value_without_important(value, important)?;
    if component.contains(',') || is_css_wide_keyword(&component) {
        return None;
    }
    let lower = component.to_ascii_lowercase();
    match (axis, lower.as_str()) {
        (PositionAxis::Horizontal, "left") => Some(background_position_keyword("0", "left")),
        (PositionAxis::Horizontal, "center") => Some(background_position_keyword("50%", "center")),
        (PositionAxis::Horizontal, "right") => Some(background_position_keyword("100%", "right")),
        (PositionAxis::Horizontal, "top" | "bottom") => None,
        (PositionAxis::Vertical, "top") => Some(background_position_keyword("0", "top")),
        (PositionAxis::Vertical, "center") => Some(background_position_keyword("50%", "center")),
        (PositionAxis::Vertical, "bottom") => Some(background_position_keyword("100%", "bottom")),
        (PositionAxis::Vertical, "left" | "right") => None,
        _ => Some(BackgroundPositionAxisComponent {
            value: component,
            keyword: None,
        }),
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

fn background_position_keyword(
    value: &'static str,
    keyword: &'static str,
) -> BackgroundPositionAxisComponent {
    BackgroundPositionAxisComponent {
        value: value.to_string(),
        keyword: Some(keyword),
    }
}

fn compressed_background_position_axis_value(
    x: &BackgroundPositionAxisComponent,
    y: &BackgroundPositionAxisComponent,
) -> String {
    if x.value == "50%" && y.value == "50%" {
        return "50%".to_string();
    }
    if y.value == "50%" {
        return x.value.clone();
    }
    if x.keyword == Some("center") {
        if y.keyword == Some("top") {
            return "top".to_string();
        }
        if y.keyword == Some("bottom") {
            return "bottom".to_string();
        }
    }
    format!("{} {}", x.value, y.value)
}

fn is_css_wide_keyword(value: &str) -> bool {
    matches!(
        value.to_ascii_lowercase().as_str(),
        "inherit" | "initial" | "revert" | "revert-layer" | "unset"
    )
}
