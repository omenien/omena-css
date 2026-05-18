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
        values::split_top_level_whitespace_value_components,
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
        "background-size" | "mask-size" | "-webkit-mask-size" => {
            normalize_repeated_pair_value(value, "auto")
        }
        _ => None,
    }
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
