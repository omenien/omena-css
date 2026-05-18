use omena_cascade::{BoxLonghandInputV0, prove_box_shorthand_combination};
use omena_parser::{LexedToken, StyleDialect, lex};
use omena_syntax::SyntaxKind;

use crate::{
    domains::{
        number::{compress_number_prefix, format_css_number, numeric_prefix_end},
        shorthand_line::line_shorthand_replacement_for_declarations,
        shorthand_list::{
            compress_list_style_value, list_style_shorthand_replacement_for_declarations,
        },
        shorthand_logical::collect_logical_axis_replacements,
        shorthand_motion::{compress_animation_value, compress_transition_value},
        shorthand_text::{
            compress_text_decoration_value, text_decoration_shorthand_replacement_for_declarations,
        },
    },
    helpers::{
        ascii::normalize_ascii_whitespace,
        declarations::{
            SimpleDeclarationSlice, collect_simple_declarations_in_block,
            declaration_ranges_are_adjacent, format_replacement_declaration_like_source,
        },
        tokens::matching_right_brace_index,
        values::split_top_level_whitespace_value_components,
    },
};

pub(crate) fn combine_css_shorthands_with_lexer(
    source: &str,
    dialect: StyleDialect,
) -> (String, usize) {
    let lexed = lex(source, dialect);
    let tokens = lexed.tokens();
    let mut ranges = collect_shorthand_replacement_ranges(source, tokens);
    if ranges.is_empty() {
        return (source.to_string(), 0);
    }
    ranges.sort_by_key(|(start, _, _)| *start);

    let mut output = String::with_capacity(source.len());
    let mut cursor = 0;
    for (start, end, replacement) in &ranges {
        if *start > cursor {
            output.push_str(&source[cursor..*start]);
        }
        output.push_str(replacement);
        cursor = *end;
    }
    if cursor < source.len() {
        output.push_str(&source[cursor..]);
    }

    (output, ranges.len())
}

fn collect_shorthand_replacement_ranges(
    source: &str,
    tokens: &[LexedToken],
) -> Vec<(usize, usize, String)> {
    let mut ranges = Vec::new();
    let mut index = 0;
    while index < tokens.len() {
        if tokens[index].kind == SyntaxKind::LeftBrace
            && let Some(close_index) = matching_right_brace_index(tokens, index)
        {
            ranges.extend(collect_shorthand_replacements_in_block(
                source,
                tokens,
                index,
                close_index,
            ));
            index += 1;
            continue;
        }
        index += 1;
    }
    ranges
}

fn collect_shorthand_replacements_in_block(
    source: &str,
    tokens: &[LexedToken],
    block_start: usize,
    block_end: usize,
) -> Vec<(usize, usize, String)> {
    let declarations = collect_simple_declarations_in_block(tokens, block_start, block_end);
    let mut ranges = Vec::new();
    let mut index = 0;
    while index + 3 < declarations.len() {
        if let Some((start, end, replacement)) =
            box_shorthand_replacement_for_declarations(tokens, &declarations[index..index + 4])
                .or_else(|| {
                    border_radius_shorthand_replacement_for_declarations(
                        tokens,
                        &declarations[index..index + 4],
                    )
                })
                .or_else(|| {
                    inset_shorthand_replacement_for_declarations(
                        tokens,
                        &declarations[index..index + 4],
                    )
                })
                .or_else(|| {
                    text_decoration_shorthand_replacement_for_declarations(
                        tokens,
                        &declarations[index..index + 4],
                    )
                })
        {
            ranges.push((start, end, replacement));
            index += 4;
        } else {
            index += 1;
        }
    }
    let mut index = 0;
    while index + 2 < declarations.len() {
        if let Some((start, end, replacement)) = list_style_shorthand_replacement_for_declarations(
            tokens,
            &declarations[index..index + 3],
        )
        .or_else(|| {
            line_shorthand_replacement_for_declarations(tokens, &declarations[index..index + 3])
        }) {
            ranges.push((start, end, replacement));
            index += 3;
        } else {
            index += 1;
        }
    }
    for declaration in &declarations {
        if let Some((start, end, replacement)) =
            shorthand_value_replacement_for_declaration(source, declaration)
        {
            ranges.push((start, end, replacement));
        }
    }
    ranges.extend(collect_overflow_axis_replacements(tokens, &declarations));
    ranges.extend(collect_place_axis_replacements(tokens, &declarations));
    ranges.extend(collect_gap_axis_replacements(tokens, &declarations));
    ranges.extend(collect_flex_flow_replacements(tokens, &declarations));
    ranges.extend(collect_logical_axis_replacements(tokens, &declarations));
    ranges
}

fn box_shorthand_replacement_for_declarations(
    tokens: &[LexedToken],
    declarations: &[SimpleDeclarationSlice],
) -> Option<(usize, usize, String)> {
    let shorthand_property = match declarations.first()?.property.as_str() {
        "margin-top" => "margin",
        "padding-top" => "padding",
        "border-top-color" => "border-color",
        "border-top-style" => "border-style",
        "border-top-width" => "border-width",
        "scroll-margin-top" => "scroll-margin",
        "scroll-padding-top" => "scroll-padding",
        _ => return None,
    };
    if !declaration_ranges_are_adjacent(tokens, declarations) {
        return None;
    }

    let proof_inputs = declarations
        .iter()
        .map(|declaration| BoxLonghandInputV0 {
            property: declaration.property.clone(),
            value: declaration.value.clone(),
            important: declaration.important,
            source_order: declaration.source_order,
        })
        .collect::<Vec<_>>();
    let proof = prove_box_shorthand_combination(shorthand_property, &proof_inputs);
    if !proof.accepted {
        return None;
    }

    let values = declarations
        .iter()
        .map(|declaration| declaration.value.as_str())
        .collect::<Vec<_>>();
    let shorthand_value = compress_box_shorthand_values(&values)?;
    let replacement = format!("{shorthand_property}: {shorthand_value};");
    Some((
        declarations.first()?.start,
        declarations.last()?.end,
        replacement,
    ))
}

fn shorthand_value_replacement_for_declaration(
    source: &str,
    declaration: &SimpleDeclarationSlice,
) -> Option<(usize, usize, String)> {
    if declaration.important {
        return None;
    }
    let replacement_value = if is_box_shorthand_property(&declaration.property) {
        compress_box_shorthand_value(&declaration.value)
    } else if is_border_none_shorthand_property(&declaration.property) {
        compress_border_none_shorthand_value(&declaration.value)
    } else if is_repeat_shorthand_property(&declaration.property) {
        compress_background_repeat_value(&declaration.value)
    } else if is_repeated_two_axis_shorthand_property(&declaration.property) {
        compress_repeated_two_axis_value(&declaration.value)
    } else if declaration.property == "border-radius" {
        compress_border_radius_value(&declaration.value)
    } else if declaration.property == "flex" {
        compress_flex_value(&declaration.value)
    } else if declaration.property == "flex-flow" {
        compress_flex_flow_value(&declaration.value)
    } else if is_place_axis_shorthand_property(&declaration.property) {
        compress_place_axis_shorthand_value(&declaration.property, &declaration.value)
    } else if declaration.property == "gap" {
        compress_gap_value(&declaration.value, declaration.important)
    } else if declaration.property == "inset" {
        compress_box_shorthand_value(&declaration.value)
    } else if declaration.property == "list-style" {
        compress_list_style_value(&declaration.value)
    } else if declaration.property == "transition" {
        compress_transition_value(&declaration.value)
    } else if declaration.property == "animation" {
        compress_animation_value(&declaration.value)
    } else if declaration.property == "text-decoration" {
        compress_text_decoration_value(&declaration.value, declaration.important)
    } else {
        None
    }?;
    let replacement =
        format_replacement_declaration_like_source(source, declaration, &replacement_value);
    Some((declaration.start, declaration.end, replacement))
}

fn border_radius_shorthand_replacement_for_declarations(
    tokens: &[LexedToken],
    declarations: &[SimpleDeclarationSlice],
) -> Option<(usize, usize, String)> {
    let [top_left, top_right, bottom_right, bottom_left] = declarations else {
        return None;
    };
    if top_left.property != "border-top-left-radius"
        || top_right.property != "border-top-right-radius"
        || bottom_right.property != "border-bottom-right-radius"
        || bottom_left.property != "border-bottom-left-radius"
        || declarations.iter().any(|declaration| declaration.important)
        || !declaration_ranges_are_adjacent(tokens, declarations)
        || declarations
            .iter()
            .any(|declaration| !is_single_axis_border_radius_value(&declaration.value))
    {
        return None;
    }
    let values = declarations
        .iter()
        .map(|declaration| declaration.value.as_str())
        .collect::<Vec<_>>();
    let shorthand_value = compress_box_shorthand_values(&values)?;
    Some((
        top_left.start,
        bottom_left.end,
        format!("border-radius: {shorthand_value};"),
    ))
}

fn inset_shorthand_replacement_for_declarations(
    tokens: &[LexedToken],
    declarations: &[SimpleDeclarationSlice],
) -> Option<(usize, usize, String)> {
    let [top, right, bottom, left] = declarations else {
        return None;
    };
    if top.property != "top"
        || right.property != "right"
        || bottom.property != "bottom"
        || left.property != "left"
        || declarations.iter().any(|declaration| declaration.important)
        || !declaration_ranges_are_adjacent(tokens, declarations)
    {
        return None;
    }
    let values = declarations
        .iter()
        .map(|declaration| declaration.value.as_str())
        .collect::<Vec<_>>();
    let shorthand_value = compress_box_shorthand_values(&values)?;
    Some((top.start, left.end, format!("inset: {shorthand_value};")))
}

fn collect_overflow_axis_replacements(
    tokens: &[LexedToken],
    declarations: &[SimpleDeclarationSlice],
) -> Vec<(usize, usize, String)> {
    let mut ranges = Vec::new();
    for pair in declarations.windows(2) {
        let [x, y] = pair else {
            continue;
        };
        if x.property != "overflow-x"
            || y.property != "overflow-y"
            || x.important
            || y.important
            || x.value != y.value
            || !is_overflow_axis_keyword(&x.value)
            || !declaration_ranges_are_adjacent(tokens, pair)
        {
            continue;
        }
        ranges.push((x.start, y.end, format!("overflow: {};", x.value)));
    }
    ranges
}

fn collect_place_axis_replacements(
    tokens: &[LexedToken],
    declarations: &[SimpleDeclarationSlice],
) -> Vec<(usize, usize, String)> {
    let mut ranges = Vec::new();
    for pair in declarations.windows(2) {
        if let Some(replacement) = place_axis_replacement_for_declarations(tokens, pair) {
            ranges.push(replacement);
        }
    }
    ranges
}

fn place_axis_replacement_for_declarations(
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

    let (shorthand, align_value, justify_value) = place_axis_shorthand_components(first, second)?;
    let align_value = normalize_single_component_place_value(align_value, first.important)?;
    let justify_value = normalize_single_component_place_value(justify_value, second.important)?;
    let shorthand_value = compressed_place_axis_value(shorthand, &align_value, &justify_value);
    let important = if first.important { "!important" } else { "" };

    Some((
        first.start,
        second.end,
        format!("{shorthand}: {shorthand_value}{important};"),
    ))
}

fn place_axis_shorthand_components<'a>(
    first: &'a SimpleDeclarationSlice,
    second: &'a SimpleDeclarationSlice,
) -> Option<(&'static str, &'a str, &'a str)> {
    match (first.property.as_str(), second.property.as_str()) {
        ("align-items", "justify-items") => {
            Some(("place-items", first.value.as_str(), second.value.as_str()))
        }
        ("justify-items", "align-items") => {
            Some(("place-items", second.value.as_str(), first.value.as_str()))
        }
        ("align-content", "justify-content") => {
            Some(("place-content", first.value.as_str(), second.value.as_str()))
        }
        ("justify-content", "align-content") => {
            Some(("place-content", second.value.as_str(), first.value.as_str()))
        }
        ("align-self", "justify-self") => {
            Some(("place-self", first.value.as_str(), second.value.as_str()))
        }
        ("justify-self", "align-self") => {
            Some(("place-self", second.value.as_str(), first.value.as_str()))
        }
        _ => None,
    }
}

fn collect_gap_axis_replacements(
    tokens: &[LexedToken],
    declarations: &[SimpleDeclarationSlice],
) -> Vec<(usize, usize, String)> {
    let mut ranges = Vec::new();
    for pair in declarations.windows(2) {
        if let Some(replacement) = gap_axis_replacement_for_declarations(tokens, pair) {
            ranges.push(replacement);
        }
    }
    ranges
}

fn gap_axis_replacement_for_declarations(
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

    let (row_gap, column_gap) = match (first.property.as_str(), second.property.as_str()) {
        ("row-gap", "column-gap") => (first.value.as_str(), second.value.as_str()),
        ("column-gap", "row-gap") => (second.value.as_str(), first.value.as_str()),
        _ => return None,
    };
    let row_gap = single_component_value_without_important(row_gap, first.important)?;
    let column_gap = single_component_value_without_important(column_gap, second.important)?;
    let shorthand_value = compressed_two_axis_shorthand_value(&row_gap, &column_gap);
    let important = if first.important { "!important" } else { "" };

    Some((
        first.start,
        second.end,
        format!("gap: {shorthand_value}{important};"),
    ))
}

fn collect_flex_flow_replacements(
    tokens: &[LexedToken],
    declarations: &[SimpleDeclarationSlice],
) -> Vec<(usize, usize, String)> {
    let mut ranges = Vec::new();
    for pair in declarations.windows(2) {
        if let Some(replacement) = flex_flow_replacement_for_declarations(tokens, pair) {
            ranges.push(replacement);
        }
    }
    ranges
}

fn flex_flow_replacement_for_declarations(
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

    let (direction, wrap) = match (first.property.as_str(), second.property.as_str()) {
        ("flex-direction", "flex-wrap") => (first.value.as_str(), second.value.as_str()),
        ("flex-wrap", "flex-direction") => (second.value.as_str(), first.value.as_str()),
        _ => return None,
    };
    let direction = single_component_value_without_important(direction, first.important)?;
    let wrap = single_component_value_without_important(wrap, second.important)?;
    let shorthand_value = compressed_flex_flow_components(&direction, &wrap)?;
    let important = if first.important { "!important" } else { "" };

    Some((
        first.start,
        second.end,
        format!("flex-flow: {shorthand_value}{important};"),
    ))
}

fn normalize_single_component_place_value(value: &str, important: bool) -> Option<String> {
    let component = single_component_value_without_important(value, important)?;
    Some(component.to_ascii_lowercase())
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

fn compressed_place_axis_value(shorthand: &str, align_value: &str, justify_value: &str) -> String {
    if align_value == justify_value
        && !(matches!(shorthand, "place-items" | "place-self") && align_value == "stretch")
    {
        align_value.to_string()
    } else {
        format!("{align_value} {justify_value}")
    }
}

fn is_place_axis_shorthand_property(property: &str) -> bool {
    matches!(property, "place-content" | "place-items" | "place-self")
}

fn compress_place_axis_shorthand_value(property: &str, value: &str) -> Option<String> {
    let components = split_top_level_whitespace_value_components(value)?;
    let [align_value, justify_value] = components.as_slice() else {
        return None;
    };
    let align_value = align_value.to_ascii_lowercase();
    let justify_value = justify_value.to_ascii_lowercase();
    if !is_place_single_component_keyword(&align_value)
        || !is_place_single_component_keyword(&justify_value)
    {
        return None;
    }
    let replacement = compressed_place_axis_value(property, &align_value, &justify_value);
    (replacement != normalize_ascii_whitespace(value)).then_some(replacement)
}

fn is_place_single_component_keyword(value: &str) -> bool {
    matches!(
        value,
        "auto"
            | "normal"
            | "stretch"
            | "center"
            | "start"
            | "end"
            | "flex-start"
            | "flex-end"
            | "self-start"
            | "self-end"
            | "left"
            | "right"
            | "baseline"
            | "space-between"
            | "space-around"
            | "space-evenly"
    )
}

fn compress_gap_value(value: &str, important: bool) -> Option<String> {
    let mut components = split_top_level_whitespace_value_components(value)?;
    if important
        && components.last().is_some_and(|component| {
            component.eq_ignore_ascii_case("!important")
                || component.eq_ignore_ascii_case("important")
        })
    {
        components.pop();
    }
    let [row_gap, column_gap] = components.as_slice() else {
        return None;
    };
    if row_gap != column_gap {
        return None;
    }
    let replacement = row_gap.clone();
    (replacement != normalize_ascii_whitespace(value)).then_some(replacement)
}

fn compressed_two_axis_shorthand_value(first: &str, second: &str) -> String {
    if first == second {
        first.to_string()
    } else {
        format!("{first} {second}")
    }
}

pub(crate) fn is_box_shorthand_property(property: &str) -> bool {
    matches!(
        property,
        "margin"
            | "padding"
            | "border-color"
            | "border-style"
            | "border-width"
            | "scroll-margin"
            | "scroll-padding"
    )
}

fn is_border_none_shorthand_property(property: &str) -> bool {
    matches!(
        property,
        "border" | "border-top" | "border-right" | "border-bottom" | "border-left" | "outline"
    )
}

fn compress_border_none_shorthand_value(value: &str) -> Option<String> {
    let components = split_top_level_whitespace_value_components(value)?;
    if components.len() != 3 {
        return None;
    }

    let mut style = None;
    let mut saw_default_width = false;
    let mut saw_default_color = false;
    for component in components {
        let normalized = component.to_ascii_lowercase();
        match normalized.as_str() {
            "none" if style.is_none() => style = Some(normalized),
            "medium" if !saw_default_width => saw_default_width = true,
            "currentcolor" if !saw_default_color => saw_default_color = true,
            _ => return None,
        }
    }

    (saw_default_width && saw_default_color).then_some(style?)
}

pub(crate) fn compress_background_repeat_value(value: &str) -> Option<String> {
    let components = split_top_level_whitespace_value_components(value)?;
    let [x, y] = components.as_slice() else {
        return None;
    };
    let x = x.to_ascii_lowercase();
    let y = y.to_ascii_lowercase();
    if x != y || !is_background_repeat_axis_keyword(&x) {
        return None;
    }
    Some(x)
}

fn is_repeat_shorthand_property(property: &str) -> bool {
    matches!(
        property,
        "background-repeat" | "mask-repeat" | "-webkit-mask-repeat"
    )
}

fn is_background_repeat_axis_keyword(value: &str) -> bool {
    matches!(value, "repeat" | "no-repeat" | "space" | "round")
}

fn is_repeated_two_axis_shorthand_property(property: &str) -> bool {
    matches!(
        property,
        "border-block-color"
            | "border-block-style"
            | "border-block-width"
            | "border-inline-color"
            | "border-inline-style"
            | "border-inline-width"
            | "border-spacing"
            | "inset-block"
            | "inset-inline"
            | "margin-block"
            | "margin-inline"
            | "padding-block"
            | "padding-inline"
            | "scroll-margin-block"
            | "scroll-margin-inline"
            | "scroll-padding-block"
            | "scroll-padding-inline"
    )
}

fn compress_repeated_two_axis_value(value: &str) -> Option<String> {
    let components = split_top_level_whitespace_value_components(value)?;
    let [first, second] = components.as_slice() else {
        return None;
    };
    (first == second).then(|| first.clone())
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

pub(crate) fn compress_flex_value(value: &str) -> Option<String> {
    let components = split_top_level_whitespace_value_components(value)?;
    let [grow, shrink, basis] = components.as_slice() else {
        return None;
    };
    let grow = normalize_flex_number(grow)?;
    let shrink = normalize_flex_number(shrink)?;
    let basis_lower = basis.to_ascii_lowercase();

    let compressed = if basis_lower == "auto" && grow == "0" && shrink == "0" {
        "none".to_string()
    } else if basis_lower == "auto" && shrink == "1" {
        if grow == "1" {
            "auto".to_string()
        } else {
            format!("{grow} auto")
        }
    } else if is_zero_flex_basis(&basis_lower) {
        if shrink == "1" {
            grow
        } else {
            format!("{grow} {shrink}")
        }
    } else {
        return None;
    };

    (compressed.len() < normalize_ascii_whitespace(value).len()).then_some(compressed)
}

fn compress_flex_flow_value(value: &str) -> Option<String> {
    let components = split_top_level_whitespace_value_components(value)?;
    let [first, second] = components.as_slice() else {
        return None;
    };
    let shorthand_value = compressed_flex_flow_unordered_components(first, second)?;
    (shorthand_value.len() < normalize_ascii_whitespace(value).len()).then_some(shorthand_value)
}

fn compressed_flex_flow_unordered_components(first: &str, second: &str) -> Option<String> {
    let first = first.to_ascii_lowercase();
    let second = second.to_ascii_lowercase();
    if is_flex_direction_keyword(&first) && is_flex_wrap_keyword(&second) {
        return compressed_flex_flow_components(&first, &second);
    }
    if is_flex_wrap_keyword(&first) && is_flex_direction_keyword(&second) {
        return compressed_flex_flow_components(&second, &first);
    }
    None
}

fn compressed_flex_flow_components(direction: &str, wrap: &str) -> Option<String> {
    let direction = direction.to_ascii_lowercase();
    let wrap = wrap.to_ascii_lowercase();
    if !is_flex_direction_keyword(&direction) || !is_flex_wrap_keyword(&wrap) {
        return None;
    }
    if direction == "row" && wrap == "nowrap" {
        Some("row".to_string())
    } else if direction == "row" {
        Some(wrap)
    } else if wrap == "nowrap" {
        Some(direction)
    } else {
        Some(format!("{direction} {wrap}"))
    }
}

fn is_flex_direction_keyword(value: &str) -> bool {
    matches!(value, "row" | "row-reverse" | "column" | "column-reverse")
}

fn is_flex_wrap_keyword(value: &str) -> bool {
    matches!(value, "nowrap" | "wrap" | "wrap-reverse")
}

fn normalize_flex_number(value: &str) -> Option<String> {
    let split = numeric_prefix_end(value)?;
    if split != value.len() {
        return None;
    }
    let parsed = value.parse::<f64>().ok()?;
    if !parsed.is_finite() || parsed < 0.0 {
        return None;
    }

    Some(compress_number_prefix(&format_css_number(parsed)))
}

fn is_zero_flex_basis(value: &str) -> bool {
    if value == "0" {
        return true;
    }
    let Some(number) = value.strip_suffix('%') else {
        return false;
    };
    number.parse::<f64>().is_ok_and(|parsed| parsed == 0.0)
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
