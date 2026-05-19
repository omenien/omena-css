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
    for quartet in declarations.windows(4) {
        if let Some(replacement) = logical_four_side_replacement_for_declarations(tokens, quartet) {
            ranges.push(replacement);
        }
    }
    for pair in declarations.windows(2) {
        if let Some(replacement) = logical_axis_replacement_for_declarations(tokens, pair)
            && !replacement_range_overlaps_existing(&ranges, replacement.0, replacement.1)
        {
            ranges.push(replacement);
        }
    }
    ranges
}

fn logical_four_side_replacement_for_declarations(
    tokens: &[LexedToken],
    declarations: &[SimpleDeclarationSlice],
) -> Option<(usize, usize, String)> {
    let [first, _, _, fourth] = declarations else {
        return None;
    };
    if declarations
        .iter()
        .any(|declaration| declaration.important != first.important)
        || !declaration_ranges_are_adjacent(tokens, declarations)
    {
        return None;
    }

    let family = logical_four_side_family(declarations)?;
    let important = first.important;
    let block_start = logical_family_value(declarations, family.block_start, important)?;
    let block_end = logical_family_value(declarations, family.block_end, important)?;
    let inline_start = logical_family_value(declarations, family.inline_start, important)?;
    let inline_end = logical_family_value(declarations, family.inline_end, important)?;
    let block_value = compressed_two_axis_shorthand_value(&block_start, &block_end);
    let inline_value = compressed_two_axis_shorthand_value(&inline_start, &inline_end);
    let important_suffix = if important { "!important" } else { "" };

    if block_start == block_end
        && block_start == inline_start
        && block_start == inline_end
        && let Some(physical_shorthand) = family.all_equal_physical_shorthand
    {
        return Some((
            first.start,
            fourth.end,
            format!("{physical_shorthand}: {block_start}{important_suffix};"),
        ));
    }

    Some((
        first.start,
        fourth.end,
        format!(
            "{}: {}{}; {}: {}{};",
            family.block_shorthand,
            block_value,
            important_suffix,
            family.inline_shorthand,
            inline_value,
            important_suffix
        ),
    ))
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

#[derive(Clone, Copy)]
struct LogicalFourSideFamily {
    block_shorthand: &'static str,
    inline_shorthand: &'static str,
    block_start: &'static str,
    block_end: &'static str,
    inline_start: &'static str,
    inline_end: &'static str,
    all_equal_physical_shorthand: Option<&'static str>,
}

const LOGICAL_FOUR_SIDE_FAMILIES: &[LogicalFourSideFamily] = &[
    LogicalFourSideFamily {
        block_shorthand: "margin-block",
        inline_shorthand: "margin-inline",
        block_start: "margin-block-start",
        block_end: "margin-block-end",
        inline_start: "margin-inline-start",
        inline_end: "margin-inline-end",
        all_equal_physical_shorthand: None,
    },
    LogicalFourSideFamily {
        block_shorthand: "padding-block",
        inline_shorthand: "padding-inline",
        block_start: "padding-block-start",
        block_end: "padding-block-end",
        inline_start: "padding-inline-start",
        inline_end: "padding-inline-end",
        all_equal_physical_shorthand: None,
    },
    LogicalFourSideFamily {
        block_shorthand: "inset-block",
        inline_shorthand: "inset-inline",
        block_start: "inset-block-start",
        block_end: "inset-block-end",
        inline_start: "inset-inline-start",
        inline_end: "inset-inline-end",
        all_equal_physical_shorthand: None,
    },
    LogicalFourSideFamily {
        block_shorthand: "scroll-margin-block",
        inline_shorthand: "scroll-margin-inline",
        block_start: "scroll-margin-block-start",
        block_end: "scroll-margin-block-end",
        inline_start: "scroll-margin-inline-start",
        inline_end: "scroll-margin-inline-end",
        all_equal_physical_shorthand: None,
    },
    LogicalFourSideFamily {
        block_shorthand: "scroll-padding-block",
        inline_shorthand: "scroll-padding-inline",
        block_start: "scroll-padding-block-start",
        block_end: "scroll-padding-block-end",
        inline_start: "scroll-padding-inline-start",
        inline_end: "scroll-padding-inline-end",
        all_equal_physical_shorthand: None,
    },
    LogicalFourSideFamily {
        block_shorthand: "border-block-color",
        inline_shorthand: "border-inline-color",
        block_start: "border-block-start-color",
        block_end: "border-block-end-color",
        inline_start: "border-inline-start-color",
        inline_end: "border-inline-end-color",
        all_equal_physical_shorthand: Some("border-color"),
    },
    LogicalFourSideFamily {
        block_shorthand: "border-block-style",
        inline_shorthand: "border-inline-style",
        block_start: "border-block-start-style",
        block_end: "border-block-end-style",
        inline_start: "border-inline-start-style",
        inline_end: "border-inline-end-style",
        all_equal_physical_shorthand: Some("border-style"),
    },
    LogicalFourSideFamily {
        block_shorthand: "border-block-width",
        inline_shorthand: "border-inline-width",
        block_start: "border-block-start-width",
        block_end: "border-block-end-width",
        inline_start: "border-inline-start-width",
        inline_end: "border-inline-end-width",
        all_equal_physical_shorthand: Some("border-width"),
    },
];

fn logical_four_side_family(
    declarations: &[SimpleDeclarationSlice],
) -> Option<LogicalFourSideFamily> {
    LOGICAL_FOUR_SIDE_FAMILIES.iter().copied().find(|family| {
        [
            family.block_start,
            family.block_end,
            family.inline_start,
            family.inline_end,
        ]
        .iter()
        .all(|property| {
            declarations
                .iter()
                .filter(|declaration| declaration.property == *property)
                .count()
                == 1
        })
    })
}

fn logical_family_value(
    declarations: &[SimpleDeclarationSlice],
    property: &str,
    important: bool,
) -> Option<String> {
    let declaration = declarations
        .iter()
        .find(|declaration| declaration.property == property)?;
    single_component_value_without_important(&declaration.value, important)
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
        ("scroll-margin-block-start", "scroll-margin-block-end") => Some((
            "scroll-margin-block",
            first.value.as_str(),
            second.value.as_str(),
        )),
        ("scroll-margin-block-end", "scroll-margin-block-start") => Some((
            "scroll-margin-block",
            second.value.as_str(),
            first.value.as_str(),
        )),
        ("scroll-margin-inline-start", "scroll-margin-inline-end") => Some((
            "scroll-margin-inline",
            first.value.as_str(),
            second.value.as_str(),
        )),
        ("scroll-margin-inline-end", "scroll-margin-inline-start") => Some((
            "scroll-margin-inline",
            second.value.as_str(),
            first.value.as_str(),
        )),
        ("scroll-padding-block-start", "scroll-padding-block-end") => Some((
            "scroll-padding-block",
            first.value.as_str(),
            second.value.as_str(),
        )),
        ("scroll-padding-block-end", "scroll-padding-block-start") => Some((
            "scroll-padding-block",
            second.value.as_str(),
            first.value.as_str(),
        )),
        ("scroll-padding-inline-start", "scroll-padding-inline-end") => Some((
            "scroll-padding-inline",
            first.value.as_str(),
            second.value.as_str(),
        )),
        ("scroll-padding-inline-end", "scroll-padding-inline-start") => Some((
            "scroll-padding-inline",
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

fn replacement_range_overlaps_existing(
    ranges: &[(usize, usize, String)],
    start: usize,
    end: usize,
) -> bool {
    ranges
        .iter()
        .any(|(existing_start, existing_end, _)| start < *existing_end && *existing_start < end)
}
