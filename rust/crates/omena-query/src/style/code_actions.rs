use super::*;

pub fn summarize_omena_query_style_extract_code_actions(
    style_uri: &str,
    source: &str,
    range: ParserRangeV0,
) -> OmenaQueryCodeActionPlanV0 {
    let mut actions = Vec::new();

    if let Some(value) = selected_extractable_css_value(source, range) {
        let property_name = next_custom_property_name(source, custom_property_stem(value));
        actions.push(OmenaQueryCodeActionV0 {
            title: format!("Extract CSS custom property '{property_name}'"),
            kind: "refactor.extract",
            edits: vec![
                OmenaQueryWorkspaceTextEditV0 {
                    uri: style_uri.to_string(),
                    range: start_of_source_range(),
                    new_text: format!(":root {{\n  {property_name}: {value};\n}}\n\n"),
                },
                OmenaQueryWorkspaceTextEditV0 {
                    uri: style_uri.to_string(),
                    range,
                    new_text: format!("var({property_name})"),
                },
            ],
            source: "omenaQueryStyleExtractCodeActions",
        });

        let value_name = next_value_name(source, value_name_stem(value));
        actions.push(OmenaQueryCodeActionV0 {
            title: format!("Extract @value '{value_name}'"),
            kind: "refactor.extract",
            edits: vec![
                OmenaQueryWorkspaceTextEditV0 {
                    uri: style_uri.to_string(),
                    range: start_of_source_range(),
                    new_text: format!("@value {value_name}: {value};\n\n"),
                },
                OmenaQueryWorkspaceTextEditV0 {
                    uri: style_uri.to_string(),
                    range,
                    new_text: value_name,
                },
            ],
            source: "omenaQueryStyleExtractCodeActions",
        });
    }

    OmenaQueryCodeActionPlanV0 {
        schema_version: "0",
        product: "omena-query.code-actions",
        file_uri: style_uri.to_string(),
        file_kind: "style",
        action_count: actions.len(),
        actions,
        ready_surfaces: vec!["styleExtractRefactorActions", "productFacingCodeActions"],
    }
}

fn selected_extractable_css_value(source: &str, range: ParserRangeV0) -> Option<&str> {
    let start = byte_offset_for_parser_position(source, range.start)?;
    let end = byte_offset_for_parser_position(source, range.end)?;
    if end <= start {
        return None;
    }
    let value = source.get(start..end)?.trim();
    is_extractable_css_value(value).then_some(value)
}

fn start_of_source_range() -> ParserRangeV0 {
    ParserRangeV0 {
        start: ParserPositionV0 {
            line: 0,
            character: 0,
        },
        end: ParserPositionV0 {
            line: 0,
            character: 0,
        },
    }
}

fn is_extractable_css_value(value: &str) -> bool {
    if value.is_empty() || value.contains('\n') || value.starts_with("var(") {
        return false;
    }
    is_hex_color(value)
        || numeric_value_unit_kind(value).is_some()
        || is_color_function(value)
        || is_ascii_css_keyword(value)
}

fn is_hex_color(value: &str) -> bool {
    value.strip_prefix('#').is_some_and(|hex| {
        (3..=8).contains(&hex.len()) && hex.chars().all(|ch| ch.is_ascii_hexdigit())
    })
}

fn numeric_value_unit_kind(value: &str) -> Option<&'static str> {
    let mut chars = value.char_indices().peekable();
    if matches!(chars.peek(), Some((_, '-'))) {
        chars.next();
    }

    let mut digit_count = 0usize;
    let mut last_number_end = 0usize;
    while let Some((index, ch)) = chars.peek().copied() {
        if !ch.is_ascii_digit() {
            break;
        }
        digit_count += 1;
        last_number_end = index + ch.len_utf8();
        chars.next();
    }
    if digit_count == 0 {
        return None;
    }

    if matches!(chars.peek(), Some((_, '.'))) {
        chars.next();
        let mut fractional_digit_count = 0usize;
        while let Some((index, ch)) = chars.peek().copied() {
            if !ch.is_ascii_digit() {
                break;
            }
            fractional_digit_count += 1;
            last_number_end = index + ch.len_utf8();
            chars.next();
        }
        if fractional_digit_count == 0 {
            return None;
        }
    }

    let unit = value.get(last_number_end..)?;
    match unit {
        "s" | "ms" => Some("duration"),
        "deg" => Some("angle"),
        "" | "px" | "rem" | "em" | "%" | "vh" | "vw" | "vmin" | "vmax" | "ch" | "ex" => {
            Some("size")
        }
        _ => None,
    }
}

fn is_color_function(value: &str) -> bool {
    ["rgb(", "rgba(", "hsl(", "hsla("]
        .iter()
        .any(|prefix| value.starts_with(prefix))
        && value.ends_with(')')
}

fn is_ascii_css_keyword(value: &str) -> bool {
    let mut chars = value.chars();
    chars.next().is_some_and(|ch| ch.is_ascii_alphabetic())
        && chars.all(|ch| ch.is_ascii_alphabetic() || ch == '-')
}

fn custom_property_stem(value: &str) -> &'static str {
    if is_hex_color(value) || is_color_function(value) {
        return "extracted-color";
    }
    match numeric_value_unit_kind(value) {
        Some("duration") => "extracted-duration",
        Some("angle") => "extracted-angle",
        Some("size") => "extracted-size",
        _ => "extracted-token",
    }
}

fn next_custom_property_name(source: &str, stem: &str) -> String {
    let mut candidate = format!("--{stem}");
    let mut suffix = 2usize;
    while source.contains(candidate.as_str()) {
        candidate = format!("--{stem}-{suffix}");
        suffix += 1;
    }
    candidate
}

fn value_name_stem(value: &str) -> &'static str {
    if is_hex_color(value) || is_color_function(value) {
        return "extractedColor";
    }
    match numeric_value_unit_kind(value) {
        Some("duration") => "extractedDuration",
        Some("angle") => "extractedAngle",
        Some("size") => "extractedSize",
        _ => "extractedToken",
    }
}

fn next_value_name(source: &str, stem: &str) -> String {
    let mut candidate = stem.to_string();
    let mut suffix = 2usize;
    while contains_identifier(source, candidate.as_str()) {
        candidate = format!("{stem}{suffix}");
        suffix += 1;
    }
    candidate
}

fn contains_identifier(source: &str, candidate: &str) -> bool {
    source.match_indices(candidate).any(|(start, _)| {
        let end = start + candidate.len();
        let before_is_identifier = source
            .get(..start)
            .and_then(|prefix| prefix.chars().next_back())
            .is_some_and(is_identifier_char);
        let after_is_identifier = source
            .get(end..)
            .and_then(|suffix| suffix.chars().next())
            .is_some_and(is_identifier_char);
        !before_is_identifier && !after_is_identifier
    })
}

fn is_identifier_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '_'
}
