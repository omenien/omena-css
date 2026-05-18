use omena_cascade::{BoxLonghandInputV0, prove_box_shorthand_combination};
use omena_parser::{LexedToken, StyleDialect, lex};
use omena_syntax::SyntaxKind;

use crate::helpers::{
    ascii::normalize_ascii_whitespace,
    declarations::{
        SimpleDeclarationSlice, collect_simple_declarations_in_block,
        declaration_ranges_are_adjacent, format_replacement_declaration_like_source,
    },
    tokens::matching_right_brace_index,
    values::split_top_level_whitespace_value_components,
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
        ) {
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
    } else if declaration.property == "background-repeat" {
        compress_background_repeat_value(&declaration.value)
    } else if declaration.property == "border-radius" {
        compress_border_radius_value(&declaration.value)
    } else if declaration.property == "inset" {
        compress_box_shorthand_value(&declaration.value)
    } else if declaration.property == "list-style" {
        compress_list_style_value(&declaration.value)
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

fn list_style_shorthand_replacement_for_declarations(
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
