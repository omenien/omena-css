use omena_parser::LexedToken;

use crate::helpers::{
    declarations::{SimpleDeclarationSlice, declaration_ranges_are_adjacent},
    values::split_top_level_whitespace_value_components,
};

pub(crate) fn collect_logical_axis_replacements(
    tokens: &[LexedToken],
    declarations: &[SimpleDeclarationSlice],
) -> Vec<(usize, usize, String)> {
    let mut ranges = Vec::new();
    for pair in declarations.windows(2) {
        if let Some(replacement) = logical_axis_replacement_for_declarations(tokens, pair) {
            ranges.push(replacement);
        }
    }
    ranges
}

fn logical_axis_replacement_for_declarations(
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

    let (shorthand, start_value, end_value) = logical_axis_shorthand_components(first, second)?;
    let start_value = single_component_value_without_important(start_value, first.important)?;
    let end_value = single_component_value_without_important(end_value, second.important)?;
    let shorthand_value = compressed_two_axis_shorthand_value(&start_value, &end_value);
    let important = if first.important { "!important" } else { "" };

    Some((
        first.start,
        second.end,
        format!("{shorthand}: {shorthand_value}{important};"),
    ))
}

fn logical_axis_shorthand_components<'a>(
    first: &'a SimpleDeclarationSlice,
    second: &'a SimpleDeclarationSlice,
) -> Option<(&'static str, &'a str, &'a str)> {
    match (first.property.as_str(), second.property.as_str()) {
        ("margin-block-start", "margin-block-end") => {
            Some(("margin-block", first.value.as_str(), second.value.as_str()))
        }
        ("margin-block-end", "margin-block-start") => {
            Some(("margin-block", second.value.as_str(), first.value.as_str()))
        }
        ("margin-inline-start", "margin-inline-end") => {
            Some(("margin-inline", first.value.as_str(), second.value.as_str()))
        }
        ("margin-inline-end", "margin-inline-start") => {
            Some(("margin-inline", second.value.as_str(), first.value.as_str()))
        }
        ("padding-block-start", "padding-block-end") => {
            Some(("padding-block", first.value.as_str(), second.value.as_str()))
        }
        ("padding-block-end", "padding-block-start") => {
            Some(("padding-block", second.value.as_str(), first.value.as_str()))
        }
        ("padding-inline-start", "padding-inline-end") => Some((
            "padding-inline",
            first.value.as_str(),
            second.value.as_str(),
        )),
        ("padding-inline-end", "padding-inline-start") => Some((
            "padding-inline",
            second.value.as_str(),
            first.value.as_str(),
        )),
        ("inset-block-start", "inset-block-end") => {
            Some(("inset-block", first.value.as_str(), second.value.as_str()))
        }
        ("inset-block-end", "inset-block-start") => {
            Some(("inset-block", second.value.as_str(), first.value.as_str()))
        }
        ("inset-inline-start", "inset-inline-end") => {
            Some(("inset-inline", first.value.as_str(), second.value.as_str()))
        }
        ("inset-inline-end", "inset-inline-start") => {
            Some(("inset-inline", second.value.as_str(), first.value.as_str()))
        }
        ("border-block-start-color", "border-block-end-color") => Some((
            "border-block-color",
            first.value.as_str(),
            second.value.as_str(),
        )),
        ("border-block-end-color", "border-block-start-color") => Some((
            "border-block-color",
            second.value.as_str(),
            first.value.as_str(),
        )),
        ("border-inline-start-color", "border-inline-end-color") => Some((
            "border-inline-color",
            first.value.as_str(),
            second.value.as_str(),
        )),
        ("border-inline-end-color", "border-inline-start-color") => Some((
            "border-inline-color",
            second.value.as_str(),
            first.value.as_str(),
        )),
        ("border-block-start-style", "border-block-end-style") => Some((
            "border-block-style",
            first.value.as_str(),
            second.value.as_str(),
        )),
        ("border-block-end-style", "border-block-start-style") => Some((
            "border-block-style",
            second.value.as_str(),
            first.value.as_str(),
        )),
        ("border-inline-start-style", "border-inline-end-style") => Some((
            "border-inline-style",
            first.value.as_str(),
            second.value.as_str(),
        )),
        ("border-inline-end-style", "border-inline-start-style") => Some((
            "border-inline-style",
            second.value.as_str(),
            first.value.as_str(),
        )),
        ("border-block-start-width", "border-block-end-width") => Some((
            "border-block-width",
            first.value.as_str(),
            second.value.as_str(),
        )),
        ("border-block-end-width", "border-block-start-width") => Some((
            "border-block-width",
            second.value.as_str(),
            first.value.as_str(),
        )),
        ("border-inline-start-width", "border-inline-end-width") => Some((
            "border-inline-width",
            first.value.as_str(),
            second.value.as_str(),
        )),
        ("border-inline-end-width", "border-inline-start-width") => Some((
            "border-inline-width",
            second.value.as_str(),
            first.value.as_str(),
        )),
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

fn compressed_two_axis_shorthand_value(first: &str, second: &str) -> String {
    if first == second {
        first.to_string()
    } else {
        format!("{first} {second}")
    }
}
