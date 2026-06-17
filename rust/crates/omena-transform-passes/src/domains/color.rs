use omena_parser::StyleDialect;
use omena_syntax::SyntaxKind;

use crate::runtime::lex_cache::lex_cached as lex;

use crate::{
    domains::number::{compress_number_prefix, format_css_number},
    helpers::{
        ascii::normalize_ascii_whitespace,
        declarations::{
            collect_simple_declarations_in_block, declaration_ranges_are_adjacent,
            format_replacement_declaration_like_source,
        },
        identifiers::{is_css_ident_continue, is_css_ident_start},
        source_rewrite::remove_source_ranges,
        tokens::{
            is_comment_token, is_declaration_boundary_end, is_declaration_boundary_start,
            matching_right_brace_index,
        },
        values::{
            parse_whole_function_value_arguments, parse_whole_function_value_inner,
            split_top_level_value_arguments, split_top_level_whitespace_value_components,
            substitute_static_css_function_references_in_value,
        },
    },
};

pub(crate) fn parse_color_mix_value(value: &str) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, "color-mix")?;
    let [space, first, second] = arguments.as_slice() else {
        return None;
    };
    let interpolation_space = parse_static_color_mix_space(space)?;

    let first_stop = parse_static_color_mix_stop(first)?;
    let second_stop = parse_static_color_mix_stop(second)?;
    let color_mix = color_mix_weights(first_stop.percentage, second_stop.percentage)?;
    let mixed = mix_srgb_colors(
        interpolation_space,
        first_stop.color,
        second_stop.color,
        color_mix.first_weight,
        color_mix.second_weight,
        color_mix.alpha_multiplier,
    );
    Some(mixed.color.to_css_rgb_with_alpha(mixed.alpha))
}

pub(crate) fn is_static_color_reference_property(property: &str) -> bool {
    matches!(
        property,
        "accent-color"
            | "background"
            | "background-color"
            | "background-image"
            | "border"
            | "border-block"
            | "border-block-color"
            | "border-block-end"
            | "border-block-end-color"
            | "border-block-start"
            | "border-block-start-color"
            | "border-bottom"
            | "border-bottom-color"
            | "border-color"
            | "border-image"
            | "border-image-source"
            | "border-inline"
            | "border-inline-color"
            | "border-inline-end"
            | "border-inline-end-color"
            | "border-inline-start"
            | "border-inline-start-color"
            | "border-left"
            | "border-left-color"
            | "border-right"
            | "border-right-color"
            | "border-top"
            | "border-top-color"
            | "box-shadow"
            | "caret-color"
            | "color"
            | "column-rule"
            | "column-rule-color"
            | "fill"
            | "filter"
            | "flood-color"
            | "lighting-color"
            | "list-style-image"
            | "mask"
            | "mask-image"
            | "outline"
            | "outline-color"
            | "scrollbar-color"
            | "stop-color"
            | "stroke"
            | "text-decoration-color"
            | "text-emphasis-color"
            | "text-shadow"
            | "-webkit-mask"
            | "-webkit-mask-image"
    )
}

fn is_static_color_compression_property(property: &str) -> bool {
    is_static_color_reference_property(property)
}

fn preserves_currentcolor_keyword_case(property: &str) -> bool {
    matches!(property, "column-rule" | "column-rule-color")
}

pub(crate) fn compress_css_colors_with_lexer(
    source: &str,
    dialect: StyleDialect,
) -> (String, usize) {
    let (source, hex_mutation_count) = compress_css_hex_color_tokens_with_lexer(source, dialect);
    let (source, function_mutation_count) =
        compress_static_color_function_declaration_values_with_lexer(&source, dialect);
    let (source, duplicate_mutation_count) =
        remove_adjacent_duplicate_static_color_declarations_with_lexer(&source, dialect);

    (
        source,
        hex_mutation_count + function_mutation_count + duplicate_mutation_count,
    )
}

fn compress_css_hex_color_tokens_with_lexer(
    source: &str,
    dialect: StyleDialect,
) -> (String, usize) {
    let lexed = lex(source, dialect);
    let tokens = lexed.tokens();
    let mut output = String::with_capacity(source.len());
    let mut mutation_count = 0;
    let mut property_candidate = false;
    let mut inside_declaration_value = false;

    for token in tokens {
        if is_declaration_boundary_start(token.kind) {
            property_candidate = true;
            inside_declaration_value = false;
        } else if is_declaration_boundary_end(token.kind) {
            property_candidate = token.kind == SyntaxKind::Semicolon;
            inside_declaration_value = false;
        } else if token.kind == SyntaxKind::Colon && property_candidate {
            property_candidate = false;
            inside_declaration_value = true;
        } else if property_candidate
            && !is_comment_token(token.kind)
            && token.kind != SyntaxKind::Whitespace
            && !matches!(
                token.kind,
                SyntaxKind::Ident | SyntaxKind::CustomPropertyName
            )
        {
            property_candidate = false;
        }

        let replacement = if token.kind == SyntaxKind::Hash && inside_declaration_value {
            compress_hex_color_token_text(&token.text)
        } else {
            None
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

    (output, mutation_count)
}

fn compress_static_color_function_declaration_values_with_lexer(
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
            let declarations = collect_simple_declarations_in_block(tokens, index, close_index);
            for declaration in declarations {
                if declaration.property.starts_with("--") || declaration.important {
                    continue;
                }
                let Some(replacement_value) = compress_static_color_references_in_declaration_value(
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

    if replacements.is_empty() {
        return (source.to_string(), 0);
    }

    let mut output = String::with_capacity(source.len());
    let mut cursor = 0;
    for (start, end, replacement) in &replacements {
        if *start > cursor {
            output.push_str(&source[cursor..*start]);
        }
        output.push_str(replacement);
        cursor = *end;
    }
    if cursor < source.len() {
        output.push_str(&source[cursor..]);
    }

    (output, replacements.len())
}

fn remove_adjacent_duplicate_static_color_declarations_with_lexer(
    source: &str,
    dialect: StyleDialect,
) -> (String, usize) {
    let lexed = lex(source, dialect);
    let tokens = lexed.tokens();
    let mut ranges = Vec::new();
    let mut index = 0;

    while index < tokens.len() {
        if tokens[index].kind == SyntaxKind::LeftBrace
            && let Some(close_index) = matching_right_brace_index(tokens, index)
        {
            let declarations = collect_simple_declarations_in_block(tokens, index, close_index);
            for pair in declarations.windows(2) {
                let [left, right] = pair else {
                    continue;
                };
                if !declaration_ranges_are_adjacent(tokens, pair)
                    || left.important
                    || right.important
                    || left.property != right.property
                    || left.value != right.value
                    || !is_static_color_compression_property(&left.property)
                {
                    continue;
                }
                ranges.push((right.start, right.end));
            }
            index = close_index + 1;
            continue;
        }
        index += 1;
    }

    let (output, removed_count) = remove_source_ranges(source, &ranges);
    (output, removed_count)
}

fn compress_static_color_value(value: &str) -> Option<String> {
    let trimmed = value.trim();
    let color = parse_static_srgb_color_with_alpha(value)
        .or_else(|| parse_static_rgb_function_color_with_alpha(value))
        .or_else(|| parse_static_hsl_function_color_with_alpha(value))
        .or_else(|| parse_static_hwb_function_color_with_alpha(value))?;
    let replacement = shortest_static_srgb_color_with_alpha_text(color);
    (replacement.len() < trimmed.len()
        || (replacement.len() == trimmed.len() && replacement != trimmed))
        .then_some(replacement)
}

fn compress_static_color_references_in_value(value: &str) -> Option<String> {
    substitute_static_css_function_references_in_value(
        value,
        &[
            ("rgb", compress_static_color_value),
            ("rgba", compress_static_color_value),
            ("hsl", compress_static_color_value),
            ("hsla", compress_static_color_value),
            ("hwb", compress_static_color_value),
        ],
    )
    .or_else(|| compress_static_color_value(value))
}

fn compress_static_color_references_in_declaration_value(
    property: &str,
    value: &str,
) -> Option<String> {
    if !is_static_color_compression_property(property) {
        return None;
    }

    let mut current = value.to_string();
    let mut changed = false;

    if let Some(replacement) = compress_static_color_references_in_value(&current) {
        current = replacement;
        changed = true;
    }
    if let Some(replacement) = compress_static_named_srgb_color_references_in_value(
        &current,
        preserves_currentcolor_keyword_case(property),
    ) {
        current = replacement;
        changed = true;
    }
    if let Some(replacement) = compress_static_linear_gradient_references_in_value(&current) {
        current = replacement;
        changed = true;
    }

    changed.then_some(current)
}

fn compress_static_linear_gradient_references_in_value(value: &str) -> Option<String> {
    substitute_static_css_function_references_in_value(
        value,
        &[
            (
                "linear-gradient",
                compress_static_default_linear_gradient_direction,
            ),
            (
                "repeating-linear-gradient",
                compress_static_default_repeating_linear_gradient_direction,
            ),
            (
                "radial-gradient",
                compress_static_default_radial_gradient_shape,
            ),
            (
                "repeating-radial-gradient",
                compress_static_default_repeating_radial_gradient_shape,
            ),
            (
                "conic-gradient",
                compress_static_default_conic_gradient_angle,
            ),
            (
                "repeating-conic-gradient",
                compress_static_default_repeating_conic_gradient_angle,
            ),
        ],
    )
}

fn compress_static_default_linear_gradient_direction(value: &str) -> Option<String> {
    compress_static_default_gradient_direction(value, "linear-gradient")
}

fn compress_static_default_repeating_linear_gradient_direction(value: &str) -> Option<String> {
    compress_static_default_gradient_direction(value, "repeating-linear-gradient")
}

fn compress_static_default_gradient_direction(value: &str, function_name: &str) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, function_name)?;
    let [direction, stops @ ..] = arguments.as_slice() else {
        return None;
    };
    if let Some(shorter_direction) = shorter_static_linear_gradient_direction(direction) {
        let replacement = format!("{function_name}({shorter_direction},{})", stops.join(","));
        return (replacement.len() < value.len()).then_some(replacement);
    }
    if stops.len() < 2 || !is_default_linear_gradient_direction(direction) {
        if !is_reverse_default_linear_gradient_direction(direction) {
            return None;
        }
        let replacement = format!(
            "{function_name}({})",
            reverse_static_linear_gradient_stops_preserving_positions(
                stops,
                function_name != "repeating-linear-gradient",
            )?
            .join(",")
        );
        return (replacement.len() < value.len()).then_some(replacement);
    }

    let replacement = format!("{function_name}({})", stops.join(","));
    (replacement.len() < value.len()).then_some(replacement)
}

fn shorter_static_linear_gradient_direction(value: &str) -> Option<&'static str> {
    match normalize_ascii_whitespace(value)
        .to_ascii_lowercase()
        .as_str()
    {
        "to right" => Some("90deg"),
        "to left" => Some("270deg"),
        _ => None,
    }
}

fn is_default_linear_gradient_direction(value: &str) -> bool {
    matches!(
        normalize_ascii_whitespace(value)
            .to_ascii_lowercase()
            .as_str(),
        "to bottom" | "180deg" | ".5turn" | "0.5turn" | "200grad"
    )
}

fn is_reverse_default_linear_gradient_direction(value: &str) -> bool {
    matches!(
        normalize_ascii_whitespace(value)
            .to_ascii_lowercase()
            .as_str(),
        "to top" | "0deg" | "-0deg" | "0turn" | "0"
    )
}

fn reverse_static_linear_gradient_stops_preserving_positions(
    stops: &[String],
    allow_position_suffixes: bool,
) -> Option<Vec<String>> {
    let parsed_stops = stops
        .iter()
        .map(|stop| parse_static_linear_gradient_color_stop(stop))
        .collect::<Option<Vec<_>>>()?;
    if !allow_position_suffixes && parsed_stops.iter().any(|stop| !stop.suffix.is_empty()) {
        return None;
    }
    let reversed_colors = parsed_stops
        .iter()
        .rev()
        .map(|stop| stop.color.as_str())
        .collect::<Vec<_>>();
    Some(
        parsed_stops
            .iter()
            .zip(reversed_colors)
            .map(|(stop, color)| format!("{color}{}", stop.suffix))
            .collect(),
    )
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct StaticLinearGradientColorStop {
    color: String,
    suffix: String,
}

fn parse_static_linear_gradient_color_stop(stop: &str) -> Option<StaticLinearGradientColorStop> {
    let trimmed = stop.trim();
    let components = split_top_level_whitespace_value_components(trimmed)?;
    let color = components.first()?.clone();
    parse_static_srgb_color(&color)?;
    let suffix = trimmed
        .get(color.len()..)
        .map(str::to_string)
        .unwrap_or_default();
    Some(StaticLinearGradientColorStop { color, suffix })
}

fn compress_static_default_radial_gradient_shape(value: &str) -> Option<String> {
    compress_static_default_radial_gradient_descriptor(value, "radial-gradient")
}

fn compress_static_default_repeating_radial_gradient_shape(value: &str) -> Option<String> {
    compress_static_default_radial_gradient_descriptor(value, "repeating-radial-gradient")
}

fn compress_static_default_radial_gradient_descriptor(
    value: &str,
    function_name: &str,
) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, function_name)?;
    let [descriptor, stops @ ..] = arguments.as_slice() else {
        return None;
    };
    if stops.len() < 2 {
        return None;
    }
    let normalized = normalize_ascii_whitespace(descriptor).to_ascii_lowercase();
    let replacement = match normalized.as_str() {
        "circle at center" | "circle farthest-corner at center" => {
            format!("{function_name}(circle,{})", stops.join(","))
        }
        "at center" | "ellipse at center" | "farthest-corner at center" => {
            format!("{function_name}({})", stops.join(","))
        }
        _ => return None,
    };
    (replacement.len() < value.len()).then_some(replacement)
}

fn compress_static_default_conic_gradient_angle(value: &str) -> Option<String> {
    compress_static_default_conic_gradient_descriptor(value, "conic-gradient")
}

fn compress_static_default_repeating_conic_gradient_angle(value: &str) -> Option<String> {
    compress_static_default_conic_gradient_descriptor(value, "repeating-conic-gradient")
}

fn compress_static_default_conic_gradient_descriptor(
    value: &str,
    function_name: &str,
) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, function_name)?;
    let [descriptor, stops @ ..] = arguments.as_slice() else {
        return None;
    };
    if stops.len() < 2 || !is_default_conic_gradient_angle(descriptor) {
        return None;
    }

    let replacement = format!("{function_name}({})", stops.join(","));
    (replacement.len() < value.len()).then_some(replacement)
}

fn is_default_conic_gradient_angle(value: &str) -> bool {
    matches!(
        normalize_ascii_whitespace(value)
            .to_ascii_lowercase()
            .as_str(),
        "from 0deg" | "from -0deg" | "from 0turn" | "from 0"
    )
}

fn compress_static_named_srgb_color_references_in_value(
    value: &str,
    preserve_currentcolor_keyword_case: bool,
) -> Option<String> {
    let mut output = String::with_capacity(value.len());
    let mut cursor = 0usize;
    let mut index = 0usize;
    let mut quote: Option<char> = None;
    let mut changed = false;

    while index < value.len() {
        let ch = value[index..].chars().next()?;
        if let Some(quote_ch) = quote {
            index += ch.len_utf8();
            if ch == '\\' {
                if let Some(escaped) = value[index..].chars().next() {
                    index += escaped.len_utf8();
                }
            } else if ch == quote_ch {
                quote = None;
            }
            continue;
        }

        match ch {
            '"' | '\'' => {
                quote = Some(ch);
                index += ch.len_utf8();
            }
            _ if is_css_ident_start(ch) => {
                let start = index;
                index += ch.len_utf8();
                while let Some(next_ch) = value[index..].chars().next() {
                    if !is_css_ident_continue(next_ch) {
                        break;
                    }
                    index += next_ch.len_utf8();
                }
                let ident = &value[start..index];
                if ident.eq_ignore_ascii_case("url")
                    && value[index..].trim_start().starts_with('(')
                    && let Some(open_offset) = value[index..].find('(')
                    && let Some(close_index) = crate::helpers::values::matching_function_call_end(
                        value,
                        index + open_offset,
                    )
                {
                    index = close_index + ')'.len_utf8();
                    continue;
                }
                if ident.eq_ignore_ascii_case("currentcolor") {
                    if preserve_currentcolor_keyword_case {
                        continue;
                    }
                    let replacement = "currentColor";
                    if replacement == ident {
                        continue;
                    }
                    output.push_str(&value[cursor..start]);
                    output.push_str(replacement);
                    cursor = index;
                    changed = true;
                    continue;
                }
                let Some(color) = parse_basic_named_static_color_with_alpha(ident) else {
                    continue;
                };
                let replacement = shortest_static_srgb_color_with_alpha_text(color);
                if replacement == ident {
                    continue;
                }
                output.push_str(&value[cursor..start]);
                output.push_str(&replacement);
                cursor = index;
                changed = true;
            }
            _ => index += ch.len_utf8(),
        }
    }

    if !changed {
        return None;
    }
    output.push_str(&value[cursor..]);
    Some(output)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StaticColorMixSpace {
    Srgb,
    SrgbLinear,
}

fn parse_static_color_mix_space(space: &str) -> Option<StaticColorMixSpace> {
    match normalize_ascii_whitespace(space).as_str() {
        "in srgb" => Some(StaticColorMixSpace::Srgb),
        "in srgb-linear" => Some(StaticColorMixSpace::SrgbLinear),
        _ => None,
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct StaticColorMixStop {
    color: StaticSrgbColorWithAlpha,
    percentage: Option<f64>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct StaticColorMixWeights {
    first_weight: f64,
    second_weight: f64,
    alpha_multiplier: Option<f64>,
}

fn parse_static_color_mix_stop(input: &str) -> Option<StaticColorMixStop> {
    let (color_text, percentage) = split_static_color_mix_stop(input)?;
    Some(StaticColorMixStop {
        color: parse_static_color_mix_operand(&color_text)?,
        percentage,
    })
}

fn split_static_color_mix_stop(input: &str) -> Option<(String, Option<f64>)> {
    let input = input.trim();
    if input.is_empty() {
        return None;
    }

    let mut depth = 0usize;
    let mut bracket_depth = 0usize;
    let mut quote: Option<char> = None;
    let mut escaped = false;
    let mut in_top_level_whitespace = false;
    let mut top_level_whitespace_runs = Vec::new();

    for (index, ch) in input.char_indices() {
        if let Some(active_quote) = quote {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == active_quote {
                quote = None;
            }
            continue;
        }

        match ch {
            '"' | '\'' => {
                quote = Some(ch);
                in_top_level_whitespace = false;
            }
            '(' => {
                depth += 1;
                in_top_level_whitespace = false;
            }
            ')' => {
                depth = depth.checked_sub(1)?;
                in_top_level_whitespace = false;
            }
            '[' => {
                bracket_depth += 1;
                in_top_level_whitespace = false;
            }
            ']' => {
                bracket_depth = bracket_depth.checked_sub(1)?;
                in_top_level_whitespace = false;
            }
            ch if ch.is_ascii_whitespace() && depth == 0 && bracket_depth == 0 => {
                let whitespace_end = index + ch.len_utf8();
                if !in_top_level_whitespace {
                    top_level_whitespace_runs.push((index, whitespace_end));
                } else if let Some((_, end)) = top_level_whitespace_runs.last_mut() {
                    *end = whitespace_end;
                }
                in_top_level_whitespace = true;
            }
            _ => in_top_level_whitespace = false,
        }
    }

    if quote.is_some() || depth != 0 || bracket_depth != 0 {
        return None;
    }

    if let Some((separator_start, separator_end)) = top_level_whitespace_runs.first() {
        let percentage = input[..*separator_start].trim();
        let color = input[*separator_end..].trim();
        if !color.is_empty()
            && let Some(percentage) = parse_bounded_percentage(percentage)
        {
            return Some((color.to_string(), Some(percentage)));
        }
    }

    if let Some((separator_start, separator_end)) = top_level_whitespace_runs.last() {
        let color = input[..*separator_start].trim();
        let percentage = input[*separator_end..].trim();
        if !color.is_empty()
            && let Some(percentage) = parse_bounded_percentage(percentage)
        {
            return Some((color.to_string(), Some(percentage)));
        }
    }

    Some((input.to_string(), None))
}

fn parse_static_color_mix_operand(text: &str) -> Option<StaticSrgbColorWithAlpha> {
    parse_static_srgb_color_with_alpha(text)
        .or_else(|| parse_static_rgb_function_color_with_alpha(text))
        .or_else(|| parse_static_hsl_function_color_with_alpha(text))
        .or_else(|| parse_static_hwb_function_color_with_alpha(text))
}

fn color_mix_weights(first: Option<f64>, second: Option<f64>) -> Option<StaticColorMixWeights> {
    match (first, second) {
        (None, None) => Some(StaticColorMixWeights {
            first_weight: 0.5,
            second_weight: 0.5,
            alpha_multiplier: None,
        }),
        (Some(first), None) => Some(StaticColorMixWeights {
            first_weight: first,
            second_weight: 1.0 - first,
            alpha_multiplier: None,
        }),
        (None, Some(second)) => Some(StaticColorMixWeights {
            first_weight: 1.0 - second,
            second_weight: second,
            alpha_multiplier: None,
        }),
        (Some(first), Some(second)) => {
            let sum = first + second;
            if sum <= 0.0 {
                return None;
            }
            Some(StaticColorMixWeights {
                first_weight: first / sum,
                second_weight: second / sum,
                alpha_multiplier: (sum < 1.0).then_some(sum),
            })
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SrgbColor {
    red: u8,
    green: u8,
    blue: u8,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct StaticSrgbColorWithAlpha {
    color: SrgbColor,
    alpha: Option<f64>,
}

impl SrgbColor {
    fn to_css_rgb(self) -> String {
        format!("rgb({} {} {})", self.red, self.green, self.blue)
    }

    fn to_css_rgb_with_alpha(self, alpha: Option<f64>) -> String {
        match alpha {
            Some(alpha) => format!(
                "rgb({} {} {} / {})",
                self.red,
                self.green,
                self.blue,
                format_css_alpha(alpha)
            ),
            None => self.to_css_rgb(),
        }
    }
}

fn mix_srgb_colors(
    interpolation_space: StaticColorMixSpace,
    first: StaticSrgbColorWithAlpha,
    second: StaticSrgbColorWithAlpha,
    first_weight: f64,
    second_weight: f64,
    alpha_multiplier: Option<f64>,
) -> StaticSrgbColorWithAlpha {
    let first_alpha = first.alpha.unwrap_or(1.0);
    let second_alpha = second.alpha.unwrap_or(1.0);
    let interpolated_alpha = first_alpha * first_weight + second_alpha * second_weight;
    if interpolated_alpha <= f64::EPSILON {
        return StaticSrgbColorWithAlpha {
            color: SrgbColor {
                red: 0,
                green: 0,
                blue: 0,
            },
            alpha: Some(0.0),
        };
    }

    let final_alpha = (interpolated_alpha * alpha_multiplier.unwrap_or(1.0)).clamp(0.0, 1.0);
    let channel_mix = StaticColorMixChannelContext {
        interpolation_space,
        first_alpha,
        second_alpha,
        first_weight,
        second_weight,
        interpolated_alpha,
    };
    StaticSrgbColorWithAlpha {
        color: SrgbColor {
            red: mix_premultiplied_srgb_channel(first.color.red, second.color.red, channel_mix),
            green: mix_premultiplied_srgb_channel(
                first.color.green,
                second.color.green,
                channel_mix,
            ),
            blue: mix_premultiplied_srgb_channel(first.color.blue, second.color.blue, channel_mix),
        },
        alpha: non_opaque_alpha(final_alpha),
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct StaticColorMixChannelContext {
    interpolation_space: StaticColorMixSpace,
    first_alpha: f64,
    second_alpha: f64,
    first_weight: f64,
    second_weight: f64,
    interpolated_alpha: f64,
}

fn mix_premultiplied_srgb_channel(
    first: u8,
    second: u8,
    context: StaticColorMixChannelContext,
) -> u8 {
    let value = (f64::from(first) * context.first_alpha * context.first_weight
        + f64::from(second) * context.second_alpha * context.second_weight)
        / context.interpolated_alpha;
    match context.interpolation_space {
        StaticColorMixSpace::Srgb => value.round().clamp(0.0, 255.0) as u8,
        StaticColorMixSpace::SrgbLinear => {
            let first_linear = decode_srgb_channel(f64::from(first) / 255.0);
            let second_linear = decode_srgb_channel(f64::from(second) / 255.0);
            let value = (first_linear * context.first_alpha * context.first_weight
                + second_linear * context.second_alpha * context.second_weight)
                / context.interpolated_alpha;
            encode_srgb_channel(value)
        }
    }
}

fn non_opaque_alpha(alpha: f64) -> Option<f64> {
    ((alpha - 1.0).abs() > f64::EPSILON).then_some(alpha)
}

pub(crate) fn parse_static_srgb_color(text: &str) -> Option<SrgbColor> {
    parse_static_hex_color(text).or_else(|| parse_basic_named_srgb_color(text))
}

pub(crate) fn parse_static_srgb_color_with_alpha(text: &str) -> Option<StaticSrgbColorWithAlpha> {
    parse_static_hex_color_with_alpha(text)
        .or_else(|| parse_basic_named_static_color_with_alpha(text))
}

fn parse_static_hex_color(text: &str) -> Option<SrgbColor> {
    let hex = text.strip_prefix('#')?;
    match hex.len() {
        3 => {
            let mut chars = hex.chars();
            Some(SrgbColor {
                red: parse_repeated_hex_digit(chars.next()?)?,
                green: parse_repeated_hex_digit(chars.next()?)?,
                blue: parse_repeated_hex_digit(chars.next()?)?,
            })
        }
        6 => Some(SrgbColor {
            red: u8::from_str_radix(hex.get(0..2)?, 16).ok()?,
            green: u8::from_str_radix(hex.get(2..4)?, 16).ok()?,
            blue: u8::from_str_radix(hex.get(4..6)?, 16).ok()?,
        }),
        _ => None,
    }
}

fn parse_static_hex_color_with_alpha(text: &str) -> Option<StaticSrgbColorWithAlpha> {
    let hex = text.strip_prefix('#')?;
    match hex.len() {
        3 | 6 => Some(StaticSrgbColorWithAlpha {
            color: parse_static_hex_color(text)?,
            alpha: None,
        }),
        4 => {
            let mut chars = hex.chars();
            Some(StaticSrgbColorWithAlpha {
                color: SrgbColor {
                    red: parse_repeated_hex_digit(chars.next()?)?,
                    green: parse_repeated_hex_digit(chars.next()?)?,
                    blue: parse_repeated_hex_digit(chars.next()?)?,
                },
                alpha: non_opaque_alpha(
                    f64::from(parse_repeated_hex_digit(chars.next()?)?) / 255.0,
                ),
            })
        }
        8 => {
            let color = SrgbColor {
                red: u8::from_str_radix(hex.get(0..2)?, 16).ok()?,
                green: u8::from_str_radix(hex.get(2..4)?, 16).ok()?,
                blue: u8::from_str_radix(hex.get(4..6)?, 16).ok()?,
            };
            let alpha = u8::from_str_radix(hex.get(6..8)?, 16).ok()?;
            Some(StaticSrgbColorWithAlpha {
                color,
                alpha: non_opaque_alpha(f64::from(alpha) / 255.0),
            })
        }
        _ => None,
    }
}

fn parse_repeated_hex_digit(ch: char) -> Option<u8> {
    let digit = ch.to_digit(16)? as u8;
    Some(digit * 17)
}

fn parse_basic_named_srgb_color(text: &str) -> Option<SrgbColor> {
    let color = omena_value_lattice::parse_basic_named_srgb_color(text)?;
    Some(SrgbColor {
        red: color.red,
        green: color.green,
        blue: color.blue,
    })
}

pub(crate) fn parse_basic_named_static_color_with_alpha(
    text: &str,
) -> Option<StaticSrgbColorWithAlpha> {
    if text.eq_ignore_ascii_case("transparent") {
        return Some(StaticSrgbColorWithAlpha {
            color: SrgbColor {
                red: 0,
                green: 0,
                blue: 0,
            },
            alpha: Some(0.0),
        });
    }

    Some(StaticSrgbColorWithAlpha {
        color: parse_basic_named_srgb_color(text)?,
        alpha: None,
    })
}

pub(crate) fn parse_oklab_oklch_value(value: &str) -> Option<String> {
    parse_oklab_value(value)
        .or_else(|| parse_oklch_value(value))
        .map(|(color, alpha)| color.to_css_rgb_with_alpha(alpha))
}

pub(crate) fn parse_color_function_value(value: &str) -> Option<String> {
    let inner = parse_whole_function_value_inner(value, "color")?;
    if inner.contains(',') {
        return None;
    }
    let parts = inner.split_whitespace().collect::<Vec<_>>();
    let (space, red, green, blue, alpha) = match parts.as_slice() {
        [space, red, green, blue] => (*space, *red, *green, *blue, None),
        [space, red, green, blue, "/", alpha] => {
            let alpha = non_opaque_alpha_value(alpha)?;
            (*space, *red, *green, *blue, alpha)
        }
        _ => return None,
    };
    let color = if space.eq_ignore_ascii_case("srgb") {
        SrgbColor {
            red: parse_srgb_component(red)?,
            green: parse_srgb_component(green)?,
            blue: parse_srgb_component(blue)?,
        }
    } else if space.eq_ignore_ascii_case("srgb-linear") {
        SrgbColor {
            red: encode_srgb_channel(parse_unit_interval_component(red)?),
            green: encode_srgb_channel(parse_unit_interval_component(green)?),
            blue: encode_srgb_channel(parse_unit_interval_component(blue)?),
        }
    } else if space.eq_ignore_ascii_case("display-p3") {
        display_p3_to_srgb(
            parse_unit_interval_component(red)?,
            parse_unit_interval_component(green)?,
            parse_unit_interval_component(blue)?,
        )?
    } else {
        return None;
    };
    Some(color.to_css_rgb_with_alpha(alpha))
}

fn parse_alpha_value(text: &str) -> Option<f64> {
    parse_unit_interval_component(text)
}

fn non_opaque_alpha_value(text: &str) -> Option<Option<f64>> {
    let alpha = parse_alpha_value(text)?;
    Some(((alpha - 1.0).abs() > f64::EPSILON).then_some(alpha))
}

fn format_css_alpha(value: f64) -> String {
    compress_number_prefix(&format_css_number(value))
}

fn parse_oklab_value(value: &str) -> Option<(SrgbColor, Option<f64>)> {
    let inner = parse_whole_function_value_inner(value, "oklab")?;
    let (parts, alpha) = split_ascii_space_separated_color_args_with_optional_alpha(inner)?;
    let [lightness, a_axis, b_axis] = parts.as_slice() else {
        return None;
    };
    let lightness = parse_ok_lightness(lightness)?;
    let a_axis = parse_plain_f64(a_axis)?;
    let b_axis = parse_plain_f64(b_axis)?;
    Some((oklab_to_srgb(lightness, a_axis, b_axis)?, alpha))
}

fn parse_oklch_value(value: &str) -> Option<(SrgbColor, Option<f64>)> {
    let inner = parse_whole_function_value_inner(value, "oklch")?;
    let (parts, alpha) = split_ascii_space_separated_color_args_with_optional_alpha(inner)?;
    let [lightness, chroma, hue] = parts.as_slice() else {
        return None;
    };
    let lightness = parse_ok_lightness(lightness)?;
    let chroma = parse_plain_f64(chroma)?;
    let hue = parse_hue_degrees(hue)?.to_radians();
    Some((
        oklab_to_srgb(lightness, chroma * hue.cos(), chroma * hue.sin())?,
        alpha,
    ))
}

fn split_ascii_space_separated_color_args_with_optional_alpha(
    inner: &str,
) -> Option<(Vec<&str>, Option<f64>)> {
    if inner.contains(',') {
        return None;
    }
    let parts = inner.split_whitespace().collect::<Vec<_>>();
    match parts.as_slice() {
        [first, second, third] => Some((vec![*first, *second, *third], None)),
        [first, second, third, "/", alpha] => Some((
            vec![*first, *second, *third],
            non_opaque_alpha_value(alpha)?,
        )),
        _ => None,
    }
}

fn parse_ok_lightness(text: &str) -> Option<f64> {
    let value = if let Some(percent) = text.strip_suffix('%') {
        parse_plain_f64(percent)? / 100.0
    } else {
        parse_plain_f64(text)?
    };
    value
        .is_finite()
        .then_some(value)
        .filter(|value| *value >= 0.0 && *value <= 1.0)
}

fn parse_hue_degrees(text: &str) -> Option<f64> {
    let lower = text.to_ascii_lowercase();
    let value = if lower.ends_with("deg") {
        parse_plain_f64(text.get(..text.len() - 3)?)?
    } else if lower.ends_with("turn") {
        parse_plain_f64(text.get(..text.len() - 4)?)? * 360.0
    } else if lower.ends_with("grad") {
        parse_plain_f64(text.get(..text.len() - 4)?)? * 0.9
    } else if lower.ends_with("rad") {
        parse_plain_f64(text.get(..text.len() - 3)?)?.to_degrees()
    } else {
        parse_plain_f64(text)?
    };
    value.is_finite().then_some(value)
}

fn parse_plain_f64(text: &str) -> Option<f64> {
    if text.contains('%') {
        return None;
    }
    text.parse::<f64>().ok().filter(|value| value.is_finite())
}

fn parse_srgb_component(text: &str) -> Option<u8> {
    Some((parse_unit_interval_component(text)? * 255.0).round() as u8)
}

fn parse_unit_interval_component(text: &str) -> Option<f64> {
    let value = if let Some(percent) = text.strip_suffix('%') {
        parse_plain_f64(percent)? / 100.0
    } else {
        parse_plain_f64(text)?
    };
    if !(0.0..=1.0).contains(&value) {
        return None;
    }
    Some(value)
}

fn display_p3_to_srgb(red: f64, green: f64, blue: f64) -> Option<SrgbColor> {
    let red_linear = decode_srgb_channel(red);
    let green_linear = decode_srgb_channel(green);
    let blue_linear = decode_srgb_channel(blue);

    let x = 0.486_570_948_648_216_2 * red_linear
        + 0.265_667_693_169_093_1 * green_linear
        + 0.198_217_285_234_362_5 * blue_linear;
    let y = 0.228_974_564_069_748_8 * red_linear
        + 0.691_738_521_836_506_4 * green_linear
        + 0.079_286_914_093_745 * blue_linear;
    let z = 0.045_113_381_858_902_6 * green_linear + 1.043_944_368_900_976 * blue_linear;

    let red_linear_srgb =
        3.240_969_941_904_522_6 * x - 1.537_383_177_570_094 * y - 0.498_610_760_293_003_4 * z;
    let green_linear_srgb =
        -0.969_243_636_280_879_6 * x + 1.875_967_501_507_720_2 * y + 0.041_555_057_407_175_59 * z;
    let blue_linear_srgb =
        0.055_630_079_696_993_66 * x - 0.203_976_958_888_976_52 * y + 1.056_971_514_242_878_6 * z;

    if !is_in_gamut_linear_srgb(red_linear_srgb)
        || !is_in_gamut_linear_srgb(green_linear_srgb)
        || !is_in_gamut_linear_srgb(blue_linear_srgb)
    {
        return None;
    }

    Some(SrgbColor {
        red: encode_srgb_channel(red_linear_srgb),
        green: encode_srgb_channel(green_linear_srgb),
        blue: encode_srgb_channel(blue_linear_srgb),
    })
}

fn decode_srgb_channel(value: f64) -> f64 {
    if value <= 0.040_45 {
        value / 12.92
    } else {
        ((value + 0.055) / 1.055).powf(2.4)
    }
}

fn oklab_to_srgb(lightness: f64, a_axis: f64, b_axis: f64) -> Option<SrgbColor> {
    let l_prime = lightness + 0.396_337_777_4 * a_axis + 0.215_803_757_3 * b_axis;
    let m_prime = lightness - 0.105_561_345_8 * a_axis - 0.063_854_172_8 * b_axis;
    let s_prime = lightness - 0.089_484_177_5 * a_axis - 1.291_485_548_0 * b_axis;

    let l = l_prime.powi(3);
    let m = m_prime.powi(3);
    let s = s_prime.powi(3);

    let red_linear = 4.076_741_662_1 * l - 3.307_711_591_3 * m + 0.230_969_929_2 * s;
    let green_linear = -1.268_438_004_6 * l + 2.609_757_401_1 * m - 0.341_319_396_5 * s;
    let blue_linear = -0.004_196_086_3 * l - 0.703_418_614_7 * m + 1.707_614_701_0 * s;

    if !is_in_gamut_linear_srgb(red_linear)
        || !is_in_gamut_linear_srgb(green_linear)
        || !is_in_gamut_linear_srgb(blue_linear)
    {
        return None;
    }

    Some(SrgbColor {
        red: encode_srgb_channel(red_linear),
        green: encode_srgb_channel(green_linear),
        blue: encode_srgb_channel(blue_linear),
    })
}

fn is_in_gamut_linear_srgb(value: f64) -> bool {
    (-0.000_001..=1.000_001).contains(&value)
}

fn encode_srgb_channel(value: f64) -> u8 {
    let clamped = value.clamp(0.0, 1.0);
    let encoded = if clamped <= 0.003_130_8 {
        12.92 * clamped
    } else {
        1.055 * clamped.powf(1.0 / 2.4) - 0.055
    };
    (encoded * 255.0).round().clamp(0.0, 255.0) as u8
}

pub(crate) fn parse_static_rgb_function_color_with_alpha(
    value: &str,
) -> Option<StaticSrgbColorWithAlpha> {
    let inner = parse_whole_function_value_inner(value, "rgb")
        .or_else(|| parse_whole_function_value_inner(value, "rgba"))?;
    let (parts, alpha) = split_static_color_channels_with_optional_alpha(inner)?;
    let [red, green, blue] = parts.as_slice() else {
        return None;
    };

    Some(StaticSrgbColorWithAlpha {
        color: SrgbColor {
            red: parse_rgb_component_byte(red)?,
            green: parse_rgb_component_byte(green)?,
            blue: parse_rgb_component_byte(blue)?,
        },
        alpha,
    })
}

pub(crate) fn parse_static_hsl_function_color_with_alpha(
    value: &str,
) -> Option<StaticSrgbColorWithAlpha> {
    let inner = parse_whole_function_value_inner(value, "hsl")
        .or_else(|| parse_whole_function_value_inner(value, "hsla"))?;
    let (parts, alpha) = split_static_color_channels_with_optional_alpha(inner)?;
    let [hue, saturation, lightness] = parts.as_slice() else {
        return None;
    };

    Some(StaticSrgbColorWithAlpha {
        color: hsl_to_srgb(
            parse_hue_degrees(hue)?,
            parse_bounded_percentage(saturation)?,
            parse_bounded_percentage(lightness)?,
        )?,
        alpha,
    })
}

pub(crate) fn parse_static_hwb_function_color_with_alpha(
    value: &str,
) -> Option<StaticSrgbColorWithAlpha> {
    let inner = parse_whole_function_value_inner(value, "hwb")?;
    let (parts, alpha) = split_static_color_channels_with_optional_alpha(inner)?;
    let [hue, whiteness, blackness] = parts.as_slice() else {
        return None;
    };

    Some(StaticSrgbColorWithAlpha {
        color: hwb_to_srgb(
            parse_hue_degrees(hue)?,
            parse_bounded_percentage(whiteness)?,
            parse_bounded_percentage(blackness)?,
        )?,
        alpha,
    })
}

fn split_static_color_channels_with_optional_alpha(
    inner: &str,
) -> Option<(Vec<String>, Option<f64>)> {
    if inner.contains(',') {
        if inner.contains('/') {
            return None;
        }
        let arguments = split_top_level_value_arguments(inner)?;
        return match arguments.as_slice() {
            [first, second, third] => {
                Some((vec![first.clone(), second.clone(), third.clone()], None))
            }
            [first, second, third, alpha] => Some((
                vec![first.clone(), second.clone(), third.clone()],
                non_opaque_alpha_value(alpha)?,
            )),
            _ => None,
        };
    }

    let parts = inner.split_whitespace().collect::<Vec<_>>();
    match parts.as_slice() {
        [first, second, third] => Some((
            vec![
                (*first).to_string(),
                (*second).to_string(),
                (*third).to_string(),
            ],
            None,
        )),
        [first, second, third, "/", alpha] => Some((
            vec![
                (*first).to_string(),
                (*second).to_string(),
                (*third).to_string(),
            ],
            non_opaque_alpha_value(alpha)?,
        )),
        _ => None,
    }
}

fn parse_bounded_percentage(text: &str) -> Option<f64> {
    let value = parse_plain_f64(text.trim().strip_suffix('%')?)?;
    if !(0.0..=100.0).contains(&value) {
        return None;
    }
    Some(value / 100.0)
}

fn hwb_to_srgb(hue_degrees: f64, whiteness: f64, blackness: f64) -> Option<SrgbColor> {
    if !hue_degrees.is_finite() || !whiteness.is_finite() || !blackness.is_finite() {
        return None;
    }

    if whiteness + blackness >= 1.0 {
        let gray = whiteness / (whiteness + blackness);
        return Some(SrgbColor {
            red: encode_css_rgb_component(gray),
            green: encode_css_rgb_component(gray),
            blue: encode_css_rgb_component(gray),
        });
    }

    let pure = hsl_to_srgb(hue_degrees, 1.0, 0.5)?;
    let scale = 1.0 - whiteness - blackness;
    Some(SrgbColor {
        red: mix_hwb_channel(pure.red, scale, whiteness),
        green: mix_hwb_channel(pure.green, scale, whiteness),
        blue: mix_hwb_channel(pure.blue, scale, whiteness),
    })
}

fn mix_hwb_channel(channel: u8, scale: f64, whiteness: f64) -> u8 {
    ((f64::from(channel) / 255.0) * scale + whiteness)
        .mul_add(255.0, 0.0)
        .round()
        .clamp(0.0, 255.0) as u8
}

fn hsl_to_srgb(hue_degrees: f64, saturation: f64, lightness: f64) -> Option<SrgbColor> {
    if !hue_degrees.is_finite() || !saturation.is_finite() || !lightness.is_finite() {
        return None;
    }

    let hue = hue_degrees.rem_euclid(360.0);
    let chroma = (1.0 - (2.0 * lightness - 1.0).abs()) * saturation;
    let hue_sector = hue / 60.0;
    let x = chroma * (1.0 - (hue_sector.rem_euclid(2.0) - 1.0).abs());
    let (red1, green1, blue1) = match hue_sector.floor() as u8 {
        0 => (chroma, x, 0.0),
        1 => (x, chroma, 0.0),
        2 => (0.0, chroma, x),
        3 => (0.0, x, chroma),
        4 => (x, 0.0, chroma),
        _ => (chroma, 0.0, x),
    };
    let offset = lightness - chroma / 2.0;

    Some(SrgbColor {
        red: encode_css_rgb_component(red1 + offset),
        green: encode_css_rgb_component(green1 + offset),
        blue: encode_css_rgb_component(blue1 + offset),
    })
}

fn encode_css_rgb_component(value: f64) -> u8 {
    (value * 255.0).round().clamp(0.0, 255.0) as u8
}

fn parse_rgb_component_byte(text: &str) -> Option<u8> {
    if let Some(percent) = text.trim().strip_suffix('%') {
        let value = parse_plain_f64(percent)?;
        if !(0.0..=100.0).contains(&value) {
            return None;
        }
        return Some(((value / 100.0) * 255.0).round().clamp(0.0, 255.0) as u8);
    }

    let value = parse_plain_f64(text.trim())?;
    if !(0.0..=255.0).contains(&value) {
        return None;
    }
    Some(value.round().clamp(0.0, 255.0) as u8)
}

fn shortest_static_srgb_color_text(color: SrgbColor) -> String {
    let hex = compressed_hex_color_for_srgb(color);
    match shortest_named_srgb_color(color) {
        Some(name) if name.len() < hex.len() => name.to_string(),
        _ => hex,
    }
}

pub(crate) fn shortest_static_srgb_color_with_alpha_text(
    color: StaticSrgbColorWithAlpha,
) -> String {
    match color.alpha {
        Some(alpha) => compressed_hex_color_for_srgb_with_alpha(color.color, alpha),
        None => shortest_static_srgb_color_text(color.color),
    }
}

fn shortest_named_srgb_color(color: SrgbColor) -> Option<&'static str> {
    omena_value_lattice::shortest_named_srgb_color(omena_value_lattice::SrgbColor {
        red: color.red,
        green: color.green,
        blue: color.blue,
    })
}

fn compressed_hex_color_for_srgb(color: SrgbColor) -> String {
    let hex = format!("{:02x}{:02x}{:02x}", color.red, color.green, color.blue);
    let compressed = if can_shorten_hex_pairs(&hex) {
        shorten_hex_pairs(&hex)
    } else {
        hex
    };
    format!("#{compressed}")
}

pub(crate) fn can_shorten_hex_pairs(hex: &str) -> bool {
    hex.as_bytes()
        .chunks_exact(2)
        .all(|pair| pair[0] == pair[1])
}

pub(crate) fn shorten_hex_pairs(hex: &str) -> String {
    hex.as_bytes()
        .chunks_exact(2)
        .map(|pair| pair[0] as char)
        .collect()
}

pub(crate) fn compress_hex_color_token_text(text: &str) -> Option<String> {
    let hex = text.strip_prefix('#')?;
    if !matches!(hex.len(), 3 | 4 | 6 | 8) || !hex.chars().all(|ch| ch.is_ascii_hexdigit()) {
        return None;
    }

    let lower = hex.to_ascii_lowercase();
    let compressed = match lower.len() {
        4 if lower.ends_with('f') => lower[..3].to_string(),
        6 if can_shorten_hex_pairs(&lower) => shorten_hex_pairs(&lower),
        8 if lower.ends_with("ff") && can_shorten_hex_pairs(&lower[..6]) => {
            shorten_hex_pairs(&lower[..6])
        }
        8 if lower.ends_with("ff") => lower[..6].to_string(),
        8 if can_shorten_hex_pairs(&lower) => shorten_hex_pairs(&lower),
        _ => lower.clone(),
    };
    let rewritten = if matches!(lower.len(), 3 | 6) {
        parse_static_hex_color(&format!("#{lower}"))
            .map(shortest_static_srgb_color_text)
            .unwrap_or_else(|| format!("#{compressed}"))
    } else {
        format!("#{compressed}")
    };
    (rewritten != text).then_some(rewritten)
}

fn compressed_hex_color_for_srgb_with_alpha(color: SrgbColor, alpha: f64) -> String {
    let alpha = encode_css_rgb_component(alpha);
    let hex = format!(
        "{:02x}{:02x}{:02x}{:02x}",
        color.red, color.green, color.blue, alpha
    );
    let compressed = if can_shorten_hex_pairs(&hex) {
        shorten_hex_pairs(&hex)
    } else {
        hex
    };
    format!("#{compressed}")
}
