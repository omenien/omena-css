use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum RegisteredPropertySyntaxV0 {
    Universal,
    Supported {
        components: Vec<RegisteredPropertySyntaxComponentV0>,
    },
    Unsupported {
        source: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum RegisteredPropertySyntaxComponentV0 {
    Length,
    Percentage,
    LengthPercentage,
    Number,
    Integer,
    Color,
    Angle,
    Time,
    Ident(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum DeclaredValueKindV0 {
    Length,
    Percentage,
    Number,
    Integer,
    Color,
    Angle,
    Time,
    Ident(String),
    CssWide,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum RegisteredSyntaxMatchV0 {
    Accepts,
    Rejects,
    Unknown,
}

/// Parse the subset of CSS registered-property syntax descriptors that the
/// product checker can classify lexically. Unsupported descriptor forms remain
/// non-rejecting so the checker only warns on definite type mismatches.
pub fn parse_registered_property_syntax_v0(source: &str) -> RegisteredPropertySyntaxV0 {
    let unquoted = strip_matching_quotes(source.trim());
    let syntax = unquoted.trim();
    if syntax == "*" {
        return RegisteredPropertySyntaxV0::Universal;
    }
    if syntax.is_empty() {
        return RegisteredPropertySyntaxV0::Unsupported {
            source: source.to_string(),
        };
    }

    let mut components = Vec::new();
    for raw_component in syntax.split('|') {
        let component = raw_component.trim();
        if component.is_empty() || component_has_unsupported_multiplier(component) {
            return RegisteredPropertySyntaxV0::Unsupported {
                source: source.to_string(),
            };
        }
        let Some(parsed) = parse_registered_property_syntax_component_v0(component) else {
            return RegisteredPropertySyntaxV0::Unsupported {
                source: source.to_string(),
            };
        };
        components.push(parsed);
    }

    RegisteredPropertySyntaxV0::Supported { components }
}

pub fn registered_property_syntax_requires_initial_value_v0(source: &str) -> bool {
    !matches!(
        parse_registered_property_syntax_v0(source),
        RegisteredPropertySyntaxV0::Universal
    )
}

pub fn classify_registered_property_declared_value_v0(value: &str) -> DeclaredValueKindV0 {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return DeclaredValueKindV0::Unknown;
    }
    let lower = trimmed.to_ascii_lowercase();
    let compact = lower
        .chars()
        .filter(|character| !character.is_ascii_whitespace())
        .collect::<String>();
    if compact.contains("var(") || compact.contains("env(") || compact.contains("attr(") {
        return DeclaredValueKindV0::Unknown;
    }
    if is_css_wide_keyword(lower.as_str()) {
        return DeclaredValueKindV0::CssWide;
    }
    if is_hex_color(trimmed) || is_named_color(lower.as_str()) {
        return DeclaredValueKindV0::Color;
    }
    if let Some(open_index) = trimmed.find('(') {
        let head = trimmed[..open_index].trim().to_ascii_lowercase();
        if trimmed.ends_with(')') && is_color_function_head(head.as_str()) {
            return DeclaredValueKindV0::Color;
        }
        return DeclaredValueKindV0::Unknown;
    }
    if parse_number_with_unit(lower.as_str(), &["%"]).is_some() {
        return DeclaredValueKindV0::Percentage;
    }
    if parse_number_with_unit(lower.as_str(), TIME_UNITS).is_some() {
        return DeclaredValueKindV0::Time;
    }
    if parse_number_with_unit(lower.as_str(), ANGLE_UNITS).is_some() {
        return DeclaredValueKindV0::Angle;
    }
    if parse_number_with_unit(lower.as_str(), LENGTH_UNITS).is_some() {
        return DeclaredValueKindV0::Length;
    }
    if lower.parse::<i64>().is_ok() {
        return DeclaredValueKindV0::Integer;
    }
    if lower.parse::<f64>().is_ok() {
        return DeclaredValueKindV0::Number;
    }
    if is_literal_css_ident(lower.as_str()) {
        return DeclaredValueKindV0::Ident(lower);
    }
    DeclaredValueKindV0::Unknown
}

pub fn registered_syntax_match(syntax: &str, value: &str) -> RegisteredSyntaxMatchV0 {
    let syntax = parse_registered_property_syntax_v0(syntax);
    match syntax {
        RegisteredPropertySyntaxV0::Universal => RegisteredSyntaxMatchV0::Accepts,
        RegisteredPropertySyntaxV0::Unsupported { .. } => RegisteredSyntaxMatchV0::Unknown,
        RegisteredPropertySyntaxV0::Supported { components } => {
            let value_kind = classify_registered_property_declared_value_v0(value);
            match value_kind {
                DeclaredValueKindV0::CssWide => RegisteredSyntaxMatchV0::Accepts,
                DeclaredValueKindV0::Unknown => RegisteredSyntaxMatchV0::Unknown,
                _ if components
                    .iter()
                    .any(|component| component_accepts_value_kind(component, &value_kind)) =>
                {
                    RegisteredSyntaxMatchV0::Accepts
                }
                _ => RegisteredSyntaxMatchV0::Rejects,
            }
        }
    }
}

fn parse_registered_property_syntax_component_v0(
    component: &str,
) -> Option<RegisteredPropertySyntaxComponentV0> {
    match component {
        "<length>" => Some(RegisteredPropertySyntaxComponentV0::Length),
        "<percentage>" => Some(RegisteredPropertySyntaxComponentV0::Percentage),
        "<length-percentage>" => Some(RegisteredPropertySyntaxComponentV0::LengthPercentage),
        "<number>" => Some(RegisteredPropertySyntaxComponentV0::Number),
        "<integer>" => Some(RegisteredPropertySyntaxComponentV0::Integer),
        "<color>" => Some(RegisteredPropertySyntaxComponentV0::Color),
        "<angle>" => Some(RegisteredPropertySyntaxComponentV0::Angle),
        "<time>" => Some(RegisteredPropertySyntaxComponentV0::Time),
        _ if is_literal_css_ident(component) => Some(RegisteredPropertySyntaxComponentV0::Ident(
            component.to_ascii_lowercase(),
        )),
        _ => None,
    }
}

fn component_accepts_value_kind(
    component: &RegisteredPropertySyntaxComponentV0,
    value_kind: &DeclaredValueKindV0,
) -> bool {
    match (component, value_kind) {
        (RegisteredPropertySyntaxComponentV0::Length, DeclaredValueKindV0::Length) => true,
        (RegisteredPropertySyntaxComponentV0::Percentage, DeclaredValueKindV0::Percentage) => true,
        (
            RegisteredPropertySyntaxComponentV0::LengthPercentage,
            DeclaredValueKindV0::Length | DeclaredValueKindV0::Percentage,
        ) => true,
        (
            RegisteredPropertySyntaxComponentV0::Number,
            DeclaredValueKindV0::Number | DeclaredValueKindV0::Integer,
        ) => true,
        (RegisteredPropertySyntaxComponentV0::Integer, DeclaredValueKindV0::Integer) => true,
        (RegisteredPropertySyntaxComponentV0::Color, DeclaredValueKindV0::Color) => true,
        (RegisteredPropertySyntaxComponentV0::Angle, DeclaredValueKindV0::Angle) => true,
        (RegisteredPropertySyntaxComponentV0::Time, DeclaredValueKindV0::Time) => true,
        (
            RegisteredPropertySyntaxComponentV0::Ident(expected),
            DeclaredValueKindV0::Ident(actual),
        ) => expected == actual,
        _ => false,
    }
}

fn strip_matching_quotes(source: &str) -> &str {
    if source.len() >= 2 {
        let bytes = source.as_bytes();
        let first = bytes[0];
        let last = bytes[source.len() - 1];
        if (first == b'\'' && last == b'\'') || (first == b'"' && last == b'"') {
            return &source[1..source.len() - 1];
        }
    }
    source
}

fn component_has_unsupported_multiplier(component: &str) -> bool {
    component
        .chars()
        .any(|character| matches!(character, '+' | '#' | '?' | '*'))
}

fn is_css_wide_keyword(value: &str) -> bool {
    matches!(
        value,
        "initial" | "inherit" | "unset" | "revert" | "revert-layer"
    )
}

fn is_hex_color(value: &str) -> bool {
    let Some(hex) = value.strip_prefix('#') else {
        return false;
    };
    matches!(hex.len(), 3 | 4 | 6 | 8) && hex.chars().all(|character| character.is_ascii_hexdigit())
}

fn is_named_color(value: &str) -> bool {
    matches!(
        value,
        "aliceblue"
            | "black"
            | "blue"
            | "currentcolor"
            | "cyan"
            | "gray"
            | "green"
            | "grey"
            | "magenta"
            | "orange"
            | "purple"
            | "red"
            | "transparent"
            | "white"
            | "yellow"
    )
}

fn is_color_function_head(head: &str) -> bool {
    matches!(
        head,
        "rgb" | "rgba" | "hsl" | "hsla" | "hwb" | "lab" | "lch" | "oklab" | "oklch" | "color"
    )
}

fn parse_number_with_unit<'value, 'unit>(
    value: &'value str,
    units: &'unit [&'unit str],
) -> Option<(&'value str, &'unit str)> {
    for unit in units {
        if let Some(number) = value.strip_suffix(unit)
            && !number.is_empty()
            && number.parse::<f64>().is_ok()
        {
            return Some((number, unit));
        }
    }
    None
}

fn is_literal_css_ident(value: &str) -> bool {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    (first.is_ascii_alphabetic() || first == '_' || first == '-')
        && chars
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '_' | '-'))
}

const LENGTH_UNITS: &[&str] = &[
    "cap", "ch", "cm", "dvb", "dvh", "dvi", "dvw", "em", "ex", "ic", "in", "lh", "lvb", "lvh",
    "lvi", "lvw", "mm", "pc", "pt", "px", "q", "rem", "rlh", "svb", "svh", "svi", "svw", "vb",
    "vh", "vi", "vmax", "vmin", "vw",
];
const ANGLE_UNITS: &[&str] = &["deg", "grad", "rad", "turn"];
const TIME_UNITS: &[&str] = &["ms", "s"];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registered_property_syntax_accepts_supported_descriptor_matches() {
        assert_eq!(
            registered_syntax_match("'<length>'", "16px"),
            RegisteredSyntaxMatchV0::Accepts
        );
        assert_eq!(
            registered_syntax_match("'<length> | <percentage>'", "50%"),
            RegisteredSyntaxMatchV0::Accepts
        );
        assert_eq!(
            registered_syntax_match("'compact'", "compact"),
            RegisteredSyntaxMatchV0::Accepts
        );
    }

    #[test]
    fn registered_property_syntax_rejects_definite_mismatches() {
        assert_eq!(
            registered_syntax_match("'<length>'", "red"),
            RegisteredSyntaxMatchV0::Rejects
        );
        assert_eq!(
            registered_syntax_match("'<color>'", "8px"),
            RegisteredSyntaxMatchV0::Rejects
        );
    }

    #[test]
    fn registered_property_syntax_keeps_ambiguous_values_silent() {
        for (syntax, value) in [
            ("'*'", "red"),
            ("'<length>'", "var(--x)"),
            ("'<length>'", "calc(100% - 8px)"),
            ("'<length>+'", "8px"),
            ("'<foo>'", "8px"),
        ] {
            assert_ne!(
                registered_syntax_match(syntax, value),
                RegisteredSyntaxMatchV0::Rejects
            );
        }
        assert_eq!(
            registered_syntax_match("'<color>'", "inherit"),
            RegisteredSyntaxMatchV0::Accepts
        );
    }
}
