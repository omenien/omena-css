use omena_parser::{StyleDialect, lex};
use omena_syntax::SyntaxKind;

use crate::{
    domains::number::{compress_number_prefix, format_css_number, numeric_prefix_end},
    helpers::{
        declarations::{
            collect_simple_declarations_in_block, format_replacement_declaration_like_source,
        },
        source_rewrite::replace_source_ranges,
        tokens::{
            is_comment_token, is_declaration_boundary_end, is_declaration_boundary_start,
            matching_right_brace_index,
        },
        values::split_top_level_value_arguments,
        values::split_top_level_whitespace_value_components,
        values::substitute_static_css_function_references_in_value,
    },
};

pub(crate) fn normalize_css_units_with_lexer(
    source: &str,
    dialect: StyleDialect,
) -> (String, usize) {
    let lexed = lex(source, dialect);
    let mut output = String::with_capacity(source.len());
    let mut mutation_count = 0;
    let mut property_candidate: Option<String> = None;
    let mut active_property: Option<String> = None;
    let mut awaiting_property = false;

    for token in lexed.tokens() {
        if is_declaration_boundary_start(token.kind) {
            awaiting_property = true;
            property_candidate = None;
            active_property = None;
        } else if is_declaration_boundary_end(token.kind) {
            awaiting_property = token.kind == SyntaxKind::Semicolon;
            property_candidate = None;
            active_property = None;
        } else if token.kind == SyntaxKind::Colon && awaiting_property {
            active_property = property_candidate.clone();
            awaiting_property = false;
        } else if awaiting_property
            && !is_comment_token(token.kind)
            && token.kind != SyntaxKind::Whitespace
        {
            if matches!(
                token.kind,
                SyntaxKind::Ident | SyntaxKind::CustomPropertyName
            ) {
                property_candidate = Some(token.text.to_ascii_lowercase());
            } else {
                awaiting_property = false;
                property_candidate = None;
            }
        }

        let replacement = match token.kind {
            SyntaxKind::Dimension => active_property
                .as_deref()
                .and_then(|property| normalize_dimension_unit_token(&token.text, property)),
            SyntaxKind::Percentage => active_property
                .as_deref()
                .and_then(|property| normalize_percentage_unit_token(&token.text, property)),
            _ => None,
        };

        if let Some(replacement) = replacement {
            if replacement != token.text {
                mutation_count += 1;
            }
            output.push_str(&replacement);
        } else {
            output.push_str(&token.text);
        }
    }

    let (output, declaration_value_mutation_count) =
        normalize_static_unit_declaration_values_with_lexer(&output, dialect);
    (output, mutation_count + declaration_value_mutation_count)
}

fn is_zero_length_unit_property(property: &str) -> bool {
    matches!(
        property,
        "border-block-end-width"
            | "border-block-start-width"
            | "border-block-width"
            | "border-bottom-left-radius"
            | "border-bottom-right-radius"
            | "border-bottom-width"
            | "border-end-end-radius"
            | "border-end-start-radius"
            | "border-inline-end-width"
            | "border-inline-start-width"
            | "border-inline-width"
            | "border-left-width"
            | "border-radius"
            | "border-right-width"
            | "border-start-end-radius"
            | "border-start-start-radius"
            | "border-top-left-radius"
            | "border-top-right-radius"
            | "border-top-width"
            | "border-width"
            | "margin"
            | "margin-block"
            | "margin-block-end"
            | "margin-block-start"
            | "margin-bottom"
            | "margin-inline"
            | "margin-inline-end"
            | "margin-inline-start"
            | "margin-left"
            | "margin-right"
            | "margin-top"
            | "padding"
            | "padding-block"
            | "padding-block-end"
            | "padding-block-start"
            | "padding-bottom"
            | "padding-inline"
            | "padding-inline-end"
            | "padding-inline-start"
            | "padding-left"
            | "padding-right"
            | "padding-top"
            | "inset"
            | "inset-block"
            | "inset-block-end"
            | "inset-block-start"
            | "inset-inline"
            | "inset-inline-end"
            | "inset-inline-start"
            | "top"
            | "right"
            | "bottom"
            | "left"
            | "width"
            | "min-width"
            | "max-width"
            | "height"
            | "min-height"
            | "max-height"
            | "box-shadow"
            | "block-size"
            | "min-block-size"
            | "max-block-size"
            | "inline-size"
            | "min-inline-size"
            | "max-inline-size"
            | "outline-width"
            | "scroll-margin"
            | "scroll-margin-block"
            | "scroll-margin-block-end"
            | "scroll-margin-block-start"
            | "scroll-margin-bottom"
            | "scroll-margin-inline"
            | "scroll-margin-inline-end"
            | "scroll-margin-inline-start"
            | "scroll-margin-left"
            | "scroll-margin-right"
            | "scroll-margin-top"
            | "scroll-padding"
            | "scroll-padding-block"
            | "scroll-padding-block-end"
            | "scroll-padding-block-start"
            | "scroll-padding-bottom"
            | "scroll-padding-inline"
            | "scroll-padding-inline-end"
            | "scroll-padding-inline-start"
            | "scroll-padding-left"
            | "scroll-padding-right"
            | "scroll-padding-top"
            | "text-shadow"
            | "gap"
            | "row-gap"
            | "column-gap"
            | "line-height"
    )
}

fn normalize_dimension_unit_token(text: &str, property: &str) -> Option<String> {
    if property.starts_with("--") {
        return None;
    }

    let split = numeric_prefix_end(text)?;
    let (number, unit) = text.split_at(split);
    if let Some(replacement) = normalize_css_time_unit_token(number, unit) {
        return (replacement != text).then_some(replacement);
    }
    if is_zero_length_unit_property(property)
        && is_zero_number_prefix(number)
        && is_css_length_unit(unit)
    {
        return Some("0".to_string());
    }

    normalize_known_css_unit_case(number, unit)
}

fn normalize_percentage_unit_token(text: &str, property: &str) -> Option<String> {
    if property.starts_with("--") {
        return None;
    }

    let number = text.strip_suffix('%')?;
    if property == "opacity" {
        return normalize_opacity_percentage_token(text, number);
    }
    if !is_zero_number_prefix(number) {
        return None;
    }
    if is_zero_percentage_unit_property(property) {
        Some("0".to_string())
    } else {
        None
    }
}

fn normalize_opacity_percentage_token(text: &str, number: &str) -> Option<String> {
    let value = number.parse::<f64>().ok()?;
    if !value.is_finite() || !(0.0..=100.0).contains(&value) {
        return None;
    }

    let replacement = compress_number_prefix(&format_css_number(value / 100.0));
    (replacement.len() < text.len()).then_some(replacement)
}

fn is_zero_percentage_unit_property(property: &str) -> bool {
    matches!(
        property,
        "background-position"
            | "mask-position"
            | "-webkit-mask-position"
            | "perspective-origin"
            | "transform-origin"
    )
}

fn normalize_static_unit_declaration_values_with_lexer(
    source: &str,
    dialect: StyleDialect,
) -> (String, usize) {
    let lexed = lex(source, dialect);
    let tokens = lexed.tokens();
    let mut replacements = Vec::new();
    let mut index = 0;

    while index < tokens.len() {
        if tokens[index].kind == SyntaxKind::LeftBrace
            && let Some(close_index) = matching_right_brace_index(tokens, index)
        {
            for declaration in collect_simple_declarations_in_block(tokens, index, close_index) {
                let Some(replacement_value) = normalize_static_unit_declaration_value(
                    &declaration.property,
                    &declaration.value,
                ) else {
                    continue;
                };
                replacements.push((
                    declaration.start,
                    declaration.end,
                    format_replacement_declaration_like_source(
                        source,
                        &declaration,
                        &replacement_value,
                    ),
                ));
            }
            index += 1;
            continue;
        }
        index += 1;
    }

    replace_source_ranges(source, &replacements)
}

fn normalize_static_unit_declaration_value(property: &str, value: &str) -> Option<String> {
    match property {
        "aspect-ratio" => normalize_aspect_ratio_value(value),
        "background-position"
        | "mask-position"
        | "-webkit-mask-position"
        | "perspective-origin"
        | "transform-origin" => normalize_static_position_keyword_value(value),
        "background-size" | "mask-size" | "-webkit-mask-size" => {
            normalize_repeated_pair_value(value, "auto")
        }
        "box-shadow" => normalize_shadow_value(value, true),
        "text-shadow" => normalize_shadow_value(value, false),
        "transform" => normalize_static_transform_functions(value),
        _ => None,
    }
}

fn normalize_static_position_keyword_value(value: &str) -> Option<String> {
    let components = split_top_level_whitespace_value_components(value)?;
    let replacement = match components.as_slice() {
        [component] => normalize_single_position_keyword(component)?,
        [first, second] => normalize_position_keyword_pair(first, second)?,
        _ => return None,
    };
    (replacement.len() < normalize_ascii_position_value(value).len()).then_some(replacement)
}

fn normalize_single_position_keyword(component: &str) -> Option<String> {
    match component.to_ascii_lowercase().as_str() {
        "left" => Some("0".to_string()),
        "right" => Some("100%".to_string()),
        "center" => Some("50%".to_string()),
        "top" | "bottom" => Some(component.to_ascii_lowercase()),
        _ => None,
    }
}

fn normalize_position_keyword_pair(first: &str, second: &str) -> Option<String> {
    let first = position_keyword_axis(first)?;
    let second = position_keyword_axis(second)?;
    let (horizontal, vertical) = match (first, second) {
        (PositionKeywordAxis::Center, PositionKeywordAxis::Center) => ("50%", "50%"),
        (PositionKeywordAxis::Horizontal(horizontal), PositionKeywordAxis::Center)
        | (PositionKeywordAxis::Center, PositionKeywordAxis::Horizontal(horizontal)) => {
            (horizontal, "50%")
        }
        (PositionKeywordAxis::Vertical(vertical), PositionKeywordAxis::Center)
        | (PositionKeywordAxis::Center, PositionKeywordAxis::Vertical(vertical)) => {
            ("50%", vertical)
        }
        (PositionKeywordAxis::Horizontal(horizontal), PositionKeywordAxis::Vertical(vertical))
        | (PositionKeywordAxis::Vertical(vertical), PositionKeywordAxis::Horizontal(horizontal)) => {
            (horizontal, vertical)
        }
        _ => return None,
    };
    match (horizontal, vertical) {
        ("50%", "50%") => Some("50%".to_string()),
        ("50%", "0") => Some("top".to_string()),
        ("50%", "100%") => Some("bottom".to_string()),
        (_, "50%") => Some(horizontal.to_string()),
        _ => Some(format!("{horizontal} {vertical}")),
    }
}

enum PositionKeywordAxis {
    Horizontal(&'static str),
    Vertical(&'static str),
    Center,
}

fn position_keyword_axis(component: &str) -> Option<PositionKeywordAxis> {
    match component.to_ascii_lowercase().as_str() {
        "left" => Some(PositionKeywordAxis::Horizontal("0")),
        "right" => Some(PositionKeywordAxis::Horizontal("100%")),
        "top" => Some(PositionKeywordAxis::Vertical("0")),
        "bottom" => Some(PositionKeywordAxis::Vertical("100%")),
        "center" => Some(PositionKeywordAxis::Center),
        _ => None,
    }
}

fn normalize_ascii_position_value(value: &str) -> String {
    split_top_level_whitespace_value_components(value)
        .map(|components| components.join(" "))
        .unwrap_or_else(|| value.to_string())
}

fn normalize_static_transform_functions(value: &str) -> Option<String> {
    let normalized = substitute_static_css_function_references_in_value(
        value,
        &[
            ("rotate", normalize_zero_angle_transform_function),
            ("rotateX", normalize_zero_angle_transform_function),
            ("rotateY", normalize_zero_angle_transform_function),
            ("rotateZ", normalize_zero_angle_transform_function),
            ("scale", normalize_repeated_scale_transform_function),
            ("translate", normalize_unary_zero_length_transform_function),
        ],
    )?;
    Some(compact_transform_function_separators(&normalized))
}

fn normalize_zero_angle_transform_function(value: &str) -> Option<String> {
    normalize_unary_zero_transform_function(value, is_css_angle_unit)
}

fn normalize_unary_zero_length_transform_function(value: &str) -> Option<String> {
    normalize_unary_zero_transform_function(value, is_css_length_unit)
}

fn normalize_repeated_scale_transform_function(value: &str) -> Option<String> {
    let open_index = value.find('(')?;
    let arguments =
        split_top_level_value_arguments(value.get(open_index + 1..value.len().checked_sub(1)?)?)?;
    let [first, second] = arguments.as_slice() else {
        return None;
    };
    let first = normalize_transform_number(first)?;
    let second = normalize_transform_number(second)?;
    if first != second {
        return None;
    }

    let replacement = format!("scale({first})");
    (replacement.len() < value.len()).then_some(replacement)
}

fn normalize_transform_number(value: &str) -> Option<String> {
    let value = value.trim();
    let split = numeric_prefix_end(value)?;
    if split != value.len() {
        return None;
    }
    let parsed = value.parse::<f64>().ok()?;
    parsed
        .is_finite()
        .then(|| compress_number_prefix(&format_css_number(parsed)))
}

fn normalize_unary_zero_transform_function(
    value: &str,
    is_unit: fn(&str) -> bool,
) -> Option<String> {
    let open_index = value.find('(')?;
    let function_name = value.get(..open_index)?;
    let inner = value
        .get(open_index + 1..value.len().checked_sub(1)?)?
        .trim();
    if inner.contains(',') {
        return None;
    }
    let split = numeric_prefix_end(inner)?;
    if split == 0 || split == inner.len() {
        return None;
    }
    let (number, unit) = inner.split_at(split);
    if !is_zero_number_prefix(number) || !is_unit(unit) {
        return None;
    }

    Some(format!("{function_name}(0)"))
}

fn is_css_angle_unit(unit: &str) -> bool {
    matches!(
        unit.to_ascii_lowercase().as_str(),
        "deg" | "grad" | "rad" | "turn"
    )
}

fn compact_transform_function_separators(value: &str) -> String {
    let mut output = String::with_capacity(value.len());
    let mut index = 0usize;
    let mut depth = 0usize;

    while index < value.len() {
        let Some(ch) = value[index..].chars().next() else {
            break;
        };
        if ch.is_ascii_whitespace()
            && depth == 0
            && output.ends_with(')')
            && next_transform_component_starts(value, index)
        {
            while index < value.len() {
                let Some(whitespace) = value[index..].chars().next() else {
                    break;
                };
                if !whitespace.is_ascii_whitespace() {
                    break;
                }
                index += whitespace.len_utf8();
            }
            continue;
        }

        match ch {
            '(' => depth += 1,
            ')' => depth = depth.saturating_sub(1),
            _ => {}
        }
        output.push(ch);
        index += ch.len_utf8();
    }

    output
}

fn next_transform_component_starts(value: &str, index: usize) -> bool {
    let mut cursor = index;
    while cursor < value.len() {
        let Some(ch) = value[cursor..].chars().next() else {
            return false;
        };
        if !ch.is_ascii_whitespace() {
            break;
        }
        cursor += ch.len_utf8();
    }
    let name_start = cursor;
    while cursor < value.len() {
        let Some(ch) = value[cursor..].chars().next() else {
            return false;
        };
        if !(ch.is_ascii_alphabetic() || ch == '-') {
            break;
        }
        cursor += ch.len_utf8();
    }
    cursor > name_start && value[cursor..].starts_with('(')
}

fn normalize_aspect_ratio_value(value: &str) -> Option<String> {
    let (left, right) = value.split_once('/')?;
    if right.contains('/') {
        return None;
    }

    let left_components = split_top_level_whitespace_value_components(left.trim())?;
    let (prefix, numerator) = match left_components.as_slice() {
        [numerator] => ("", numerator.as_str()),
        [auto, numerator] if auto.eq_ignore_ascii_case("auto") => ("auto ", numerator.as_str()),
        _ => return None,
    };
    let right_components = split_top_level_whitespace_value_components(right.trim())?;
    let [denominator] = right_components.as_slice() else {
        return None;
    };

    let numerator = normalize_ratio_number(numerator)?;
    let denominator = normalize_ratio_number(denominator)?;
    let replacement = format!("{prefix}{numerator}/{denominator}");
    (replacement.len() < value.len()).then_some(replacement)
}

fn normalize_ratio_number(text: &str) -> Option<String> {
    let split = numeric_prefix_end(text)?;
    if split != text.len() {
        return None;
    }
    let value = text.parse::<f64>().ok()?;
    if !value.is_finite() || value <= 0.0 {
        return None;
    }

    Some(compress_number_prefix(&format_css_number(value)))
}

fn normalize_repeated_pair_value(value: &str, repeated: &str) -> Option<String> {
    let components = split_top_level_whitespace_value_components(value)?;
    match components.as_slice() {
        [first, second]
            if first.eq_ignore_ascii_case(repeated) && second.eq_ignore_ascii_case(repeated) =>
        {
            Some(repeated.to_string())
        }
        _ => None,
    }
}

fn normalize_shadow_value(value: &str, allow_inset: bool) -> Option<String> {
    let shadows = split_top_level_value_arguments(value)?;
    let mut changed = false;
    let mut normalized_shadows = Vec::with_capacity(shadows.len());

    for shadow in shadows {
        let normalized =
            normalize_shadow_component(&shadow, allow_inset).unwrap_or_else(|| shadow.clone());
        changed |= normalized != shadow;
        normalized_shadows.push(normalized);
    }

    let replacement = normalized_shadows.join(",");
    (changed && replacement.len() < value.len()).then_some(replacement)
}

fn normalize_shadow_component(component: &str, allow_inset: bool) -> Option<String> {
    let mut components = split_top_level_whitespace_value_components(component)?;
    let length_start = if allow_inset
        && components
            .first()
            .is_some_and(|component| component.eq_ignore_ascii_case("inset"))
    {
        1
    } else {
        0
    };
    if components.len() <= length_start + 2 {
        return None;
    }

    let blur_index = length_start + 2;
    if !components
        .get(blur_index)
        .is_some_and(|component| is_zero_shadow_length_component(component))
    {
        return None;
    }

    if allow_inset {
        let spread_index = blur_index + 1;
        if components
            .get(spread_index)
            .is_some_and(|component| is_shadow_length_component(component))
        {
            if !components
                .get(spread_index)
                .is_some_and(|component| is_zero_shadow_length_component(component))
            {
                return None;
            }
            components.remove(spread_index);
        }
    }

    components.remove(blur_index);
    Some(components.join(" "))
}

fn is_shadow_length_component(component: &str) -> bool {
    if component == "0" {
        return true;
    }

    let split = numeric_prefix_end(component).unwrap_or(0);
    if split == 0 || split == component.len() {
        return false;
    }
    let (_, unit) = component.split_at(split);
    is_css_length_unit(unit)
}

fn is_zero_shadow_length_component(component: &str) -> bool {
    if component == "0" {
        return true;
    }

    if !is_shadow_length_component(component) {
        return false;
    }
    let split = numeric_prefix_end(component).unwrap_or(0);
    let (number, unit) = component.split_at(split);
    is_zero_number_prefix(number) && is_css_length_unit(unit)
}

fn normalize_css_time_unit_token(number: &str, unit: &str) -> Option<String> {
    let normalized_unit = unit.to_ascii_lowercase();
    if !matches!(normalized_unit.as_str(), "ms" | "s") {
        return None;
    }

    let value = number.parse::<f64>().ok()?;
    if !value.is_finite() {
        return None;
    }
    if value == 0.0 {
        return Some("0s".to_string());
    }

    let seconds = if normalized_unit == "ms" {
        value / 1000.0
    } else {
        value
    };
    let seconds_text = format!("{}s", format_css_time_number(seconds));
    let milliseconds_text = format!("{}ms", format_css_time_number(seconds * 1000.0));

    if seconds_text.len() < milliseconds_text.len() {
        Some(seconds_text)
    } else {
        Some(milliseconds_text)
    }
}

fn format_css_time_number(value: f64) -> String {
    compress_number_prefix(&format_css_number(value))
}

fn is_zero_number_prefix(number: &str) -> bool {
    number.parse::<f64>().is_ok_and(|value| value == 0.0)
}

fn is_css_length_unit(unit: &str) -> bool {
    matches!(
        unit.to_ascii_lowercase().as_str(),
        "cap"
            | "ch"
            | "cm"
            | "em"
            | "ex"
            | "ic"
            | "in"
            | "lh"
            | "mm"
            | "pc"
            | "pt"
            | "px"
            | "q"
            | "rem"
            | "rlh"
            | "vb"
            | "vh"
            | "vi"
            | "vmax"
            | "vmin"
            | "vw"
    )
}

fn normalize_known_css_unit_case(number: &str, unit: &str) -> Option<String> {
    let normalized_unit = unit.to_ascii_lowercase();
    if normalized_unit == unit || !is_known_css_unit(&normalized_unit) {
        return None;
    }

    Some(format!("{number}{normalized_unit}"))
}

fn is_known_css_unit(unit: &str) -> bool {
    is_css_length_unit(unit)
        || matches!(
            unit,
            "deg"
                | "grad"
                | "rad"
                | "turn"
                | "ms"
                | "s"
                | "hz"
                | "khz"
                | "dpi"
                | "dpcm"
                | "dppx"
                | "x"
                | "fr"
        )
}
