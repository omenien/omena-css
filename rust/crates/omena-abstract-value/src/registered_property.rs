use omena_spec_audit::spec_vocabulary;
use omena_value_lattice::is_container_query_length_unit;
use serde::{Deserialize, Serialize};

use crate::{
    CssValueValidationClassV0, validate_registered_property_value_v0,
    validate_standard_property_value_v0,
};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum RegisteredPropertySyntaxV0 {
    Universal,
    Supported {
        alternatives: Vec<RegisteredPropertySyntaxAlternativeV0>,
    },
    Unsupported {
        source: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum RegisteredPropertySyntaxAlternativeV0 {
    Sequence {
        components: Vec<RegisteredPropertySyntaxComponentV0>,
    },
    Unsupported {
        source: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RegisteredPropertySyntaxComponentV0 {
    pub base: RegisteredPropertySyntaxBaseV0,
    pub multiplier: RegisteredPropertySyntaxMultiplierV0,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum RegisteredPropertySyntaxBaseV0 {
    Length,
    Percentage,
    LengthPercentage,
    Number,
    Integer,
    Color,
    Image,
    Url,
    Angle,
    Time,
    Resolution,
    TransformFunction,
    TransformList,
    CustomIdent,
    QuotedString,
    Ident(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum RegisteredPropertySyntaxMultiplierV0 {
    One,
    Plus,
    Hash,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum DeclaredNumericTypeV0 {
    Length,
    Percentage,
    Angle,
    Time,
    Resolution,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum DeclaredValueKindV0 {
    Dimension(DeclaredNumericTypeV0),
    Number,
    Integer,
    HexColor,
    ColorFunction,
    ColorKeyword(String),
    Url,
    ImageFunction,
    TransformFunction,
    QuotedString,
    BareIdent(String),
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

/// Parses the CSS Properties & Values Level 1 descriptor subset retained by
/// syntax-surface compatibility reports. Value validation uses the complete
/// registry matcher instead of this reduced descriptor projection.
pub fn parse_registered_property_syntax_v0(source: &str) -> RegisteredPropertySyntaxV0 {
    let unquoted = strip_matching_quotes(source.trim());
    let syntax = unquoted.trim();
    if syntax == "*" {
        return RegisteredPropertySyntaxV0::Universal;
    }
    if syntax.is_empty() || syntax.contains('*') {
        return RegisteredPropertySyntaxV0::Unsupported {
            source: source.to_string(),
        };
    }

    let mut alternatives = Vec::new();
    for raw_alternative in syntax.split('|') {
        let alternative = raw_alternative.trim();
        if alternative.is_empty() {
            return RegisteredPropertySyntaxV0::Unsupported {
                source: source.to_string(),
            };
        }
        alternatives.push(parse_registered_property_syntax_alternative_v0(alternative));
    }

    RegisteredPropertySyntaxV0::Supported { alternatives }
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
    if is_quoted_string(trimmed) {
        return DeclaredValueKindV0::QuotedString;
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
    if is_hex_color(trimmed) {
        return DeclaredValueKindV0::HexColor;
    }
    if is_named_color(lower.as_str()) {
        return DeclaredValueKindV0::ColorKeyword(trimmed.to_string());
    }
    if let Some(open_index) = trimmed.find('(') {
        let head = trimmed[..open_index].trim().to_ascii_lowercase();
        if is_single_function_call(trimmed, open_index) {
            if is_color_function_head(head.as_str()) {
                return DeclaredValueKindV0::ColorFunction;
            }
            if head == "url" {
                return DeclaredValueKindV0::Url;
            }
            if is_image_function_head(head.as_str()) {
                return DeclaredValueKindV0::ImageFunction;
            }
            if is_transform_function_head(head.as_str()) {
                return DeclaredValueKindV0::TransformFunction;
            }
        }
        return DeclaredValueKindV0::Unknown;
    }
    if parse_number_with_unit(lower.as_str(), &["%"]).is_some() {
        return DeclaredValueKindV0::Dimension(DeclaredNumericTypeV0::Percentage);
    }
    if parse_number_with_unit(lower.as_str(), TIME_UNITS).is_some() {
        return DeclaredValueKindV0::Dimension(DeclaredNumericTypeV0::Time);
    }
    if parse_number_with_unit(lower.as_str(), ANGLE_UNITS).is_some() {
        return DeclaredValueKindV0::Dimension(DeclaredNumericTypeV0::Angle);
    }
    if parse_number_with_unit(lower.as_str(), RESOLUTION_UNITS).is_some() {
        return DeclaredValueKindV0::Dimension(DeclaredNumericTypeV0::Resolution);
    }
    if parse_number_with_unit(lower.as_str(), LENGTH_UNITS).is_some()
        || is_container_query_length_dimension(lower.as_str())
    {
        return DeclaredValueKindV0::Dimension(DeclaredNumericTypeV0::Length);
    }
    if is_css_integer_token(lower.as_str()) {
        return DeclaredValueKindV0::Integer;
    }
    if is_css_number_token(lower.as_str()) {
        return DeclaredValueKindV0::Number;
    }
    if is_literal_css_ident(trimmed) {
        return DeclaredValueKindV0::BareIdent(trimmed.to_string());
    }
    DeclaredValueKindV0::Unknown
}

pub fn registered_syntax_match(syntax: &str, value: &str) -> RegisteredSyntaxMatchV0 {
    match validate_registered_property_value_v0(syntax, value).class {
        CssValueValidationClassV0::Valid => RegisteredSyntaxMatchV0::Accepts,
        CssValueValidationClassV0::Invalid => RegisteredSyntaxMatchV0::Rejects,
        CssValueValidationClassV0::NotValidatable => RegisteredSyntaxMatchV0::Unknown,
    }
}

/// Compatibility adapter over the complete pinned property-grammar matcher.
/// Consumers that need the reason or full verdict use
/// [`validate_standard_property_value_v0`] directly.
pub fn standard_property_syntax_match(property: &str, value: &str) -> RegisteredSyntaxMatchV0 {
    match validate_standard_property_value_v0(property, value).class {
        CssValueValidationClassV0::Valid => RegisteredSyntaxMatchV0::Accepts,
        CssValueValidationClassV0::Invalid => RegisteredSyntaxMatchV0::Rejects,
        CssValueValidationClassV0::NotValidatable => RegisteredSyntaxMatchV0::Unknown,
    }
}

fn parse_registered_property_syntax_alternative_v0(
    alternative: &str,
) -> RegisteredPropertySyntaxAlternativeV0 {
    if alternative.contains('?') {
        return RegisteredPropertySyntaxAlternativeV0::Unsupported {
            source: alternative.to_string(),
        };
    }

    let mut components = Vec::new();
    for raw_component in alternative.split_whitespace() {
        let Some(component) = parse_registered_property_syntax_component_v0(raw_component) else {
            return RegisteredPropertySyntaxAlternativeV0::Unsupported {
                source: alternative.to_string(),
            };
        };
        components.push(component);
    }

    if components.is_empty() {
        return RegisteredPropertySyntaxAlternativeV0::Unsupported {
            source: alternative.to_string(),
        };
    }
    RegisteredPropertySyntaxAlternativeV0::Sequence { components }
}

fn parse_registered_property_syntax_component_v0(
    component: &str,
) -> Option<RegisteredPropertySyntaxComponentV0> {
    let (base_source, multiplier) = if let Some(base) = component.strip_suffix('+') {
        (base, RegisteredPropertySyntaxMultiplierV0::Plus)
    } else if let Some(base) = component.strip_suffix('#') {
        (base, RegisteredPropertySyntaxMultiplierV0::Hash)
    } else {
        (component, RegisteredPropertySyntaxMultiplierV0::One)
    };
    if base_source.is_empty()
        || base_source
            .chars()
            .any(|character| matches!(character, '+' | '#' | '?' | '*'))
    {
        return None;
    }

    let base = parse_registered_property_syntax_base_v0(base_source)?;
    if base == RegisteredPropertySyntaxBaseV0::TransformList
        && multiplier != RegisteredPropertySyntaxMultiplierV0::One
    {
        return None;
    }
    Some(RegisteredPropertySyntaxComponentV0 { base, multiplier })
}

fn parse_registered_property_syntax_base_v0(
    component: &str,
) -> Option<RegisteredPropertySyntaxBaseV0> {
    match component {
        "<length>" => Some(RegisteredPropertySyntaxBaseV0::Length),
        "<percentage>" => Some(RegisteredPropertySyntaxBaseV0::Percentage),
        "<length-percentage>" => Some(RegisteredPropertySyntaxBaseV0::LengthPercentage),
        "<number>" => Some(RegisteredPropertySyntaxBaseV0::Number),
        "<integer>" => Some(RegisteredPropertySyntaxBaseV0::Integer),
        "<color>" => Some(RegisteredPropertySyntaxBaseV0::Color),
        "<image>" => Some(RegisteredPropertySyntaxBaseV0::Image),
        "<url>" => Some(RegisteredPropertySyntaxBaseV0::Url),
        "<angle>" => Some(RegisteredPropertySyntaxBaseV0::Angle),
        "<time>" => Some(RegisteredPropertySyntaxBaseV0::Time),
        "<resolution>" => Some(RegisteredPropertySyntaxBaseV0::Resolution),
        "<transform-function>" => Some(RegisteredPropertySyntaxBaseV0::TransformFunction),
        "<transform-list>" => Some(RegisteredPropertySyntaxBaseV0::TransformList),
        "<custom-ident>" => Some(RegisteredPropertySyntaxBaseV0::CustomIdent),
        "<string>" => Some(RegisteredPropertySyntaxBaseV0::QuotedString),
        _ if is_literal_css_ident(component) => {
            Some(RegisteredPropertySyntaxBaseV0::Ident(component.to_string()))
        }
        _ => None,
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

fn is_quoted_string(value: &str) -> bool {
    value.len() >= 2
        && ((value.starts_with('"') && value.ends_with('"'))
            || (value.starts_with('\'') && value.ends_with('\'')))
}

fn is_css_wide_keyword(value: &str) -> bool {
    // Spec-driven: the `all` property's grammar is exactly the CSS-wide keyword set
    // (it resets every property), so its closed alternation is the authoritative
    // source — including newer members like `revert-rule` the inline floor omits. The
    // floor keeps recognition from regressing if the feed snapshot fails to parse.
    if spec_vocabulary()
        .property_keywords("all")
        .is_some_and(|keywords| {
            keywords
                .iter()
                .any(|keyword| keyword.eq_ignore_ascii_case(value))
        })
    {
        return true;
    }
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
    // Spec-driven: the vendored webref `<named-color>` closed alternation (the full
    // set). The inline list is a recognition floor so a feed snapshot that fails to
    // parse never regresses below the historically recognized set.
    if matches!(
        spec_vocabulary().type_accepts("named-color", value),
        Some(true)
    ) {
        return true;
    }
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

fn is_single_function_call(value: &str, open_index: usize) -> bool {
    let mut depth = 0usize;
    for (index, character) in value
        .char_indices()
        .skip_while(|(index, _)| *index < open_index)
    {
        match character {
            '(' => depth += 1,
            ')' => {
                if depth == 0 {
                    return false;
                }
                depth -= 1;
                if depth == 0 && index != value.len() - 1 {
                    return false;
                }
            }
            _ => {}
        }
    }
    depth == 0 && value.ends_with(')')
}

fn is_color_function_head(head: &str) -> bool {
    matches!(
        head,
        "rgb" | "rgba" | "hsl" | "hsla" | "hwb" | "lab" | "lch" | "oklab" | "oklch" | "color"
    )
}

fn is_image_function_head(head: &str) -> bool {
    matches!(
        head,
        "image"
            | "image-set"
            | "cross-fade"
            | "element"
            | "linear-gradient"
            | "radial-gradient"
            | "conic-gradient"
            | "repeating-linear-gradient"
            | "repeating-radial-gradient"
            | "repeating-conic-gradient"
    )
}

fn is_transform_function_head(head: &str) -> bool {
    matches!(
        head,
        "matrix"
            | "matrix3d"
            | "perspective"
            | "rotate"
            | "rotate3d"
            | "rotatex"
            | "rotatey"
            | "rotatez"
            | "scale"
            | "scale3d"
            | "scalex"
            | "scaley"
            | "scalez"
            | "skew"
            | "skewx"
            | "skewy"
            | "translate"
            | "translate3d"
            | "translatex"
            | "translatey"
            | "translatez"
    )
}

/// Whether `value` is a number suffixed by a container-query length unit
/// (`cqw`/`cqh`/`cqi`/`cqb`/`cqmin`/`cqmax`). These are `<length>`s that `LENGTH_UNITS`
/// omits; recognizing them here is the first production consumer of the
/// `omena-value-lattice` recognizer, so a registered `<length>` property accepts e.g.
/// `50cqw` instead of silently leaving it undetermined.
fn is_container_query_length_dimension(value: &str) -> bool {
    let unit_len = value
        .chars()
        .rev()
        .take_while(|character| character.is_ascii_alphabetic())
        .count();
    if unit_len == 0 || unit_len == value.len() {
        return false;
    }
    let (number, unit) = value.split_at(value.len() - unit_len);
    is_css_number_token(number) && is_container_query_length_unit(unit)
}

fn parse_number_with_unit<'value, 'unit>(
    value: &'value str,
    units: &'unit [&'unit str],
) -> Option<(&'value str, &'unit str)> {
    for unit in units {
        if let Some(number) = value.strip_suffix(unit)
            && !number.is_empty()
            && is_css_number_token(number)
        {
            return Some((number, unit));
        }
    }
    None
}

fn is_css_integer_token(value: &str) -> bool {
    let unsigned = strip_number_sign(value);
    !unsigned.is_empty() && unsigned.chars().all(|character| character.is_ascii_digit())
}

fn is_css_number_token(value: &str) -> bool {
    let unsigned = strip_number_sign(value);
    if unsigned.is_empty() {
        return false;
    }
    let (mantissa, exponent) = split_number_exponent(unsigned);
    if let Some(exponent) = exponent {
        let exponent = strip_number_sign(exponent);
        if exponent.is_empty() || !exponent.chars().all(|character| character.is_ascii_digit()) {
            return false;
        }
    }
    is_css_number_mantissa(mantissa)
}

fn strip_number_sign(value: &str) -> &str {
    value
        .strip_prefix('+')
        .or_else(|| value.strip_prefix('-'))
        .unwrap_or(value)
}

fn split_number_exponent(value: &str) -> (&str, Option<&str>) {
    for (index, character) in value.char_indices() {
        if matches!(character, 'e' | 'E') {
            return (
                &value[..index],
                Some(&value[index + character.len_utf8()..]),
            );
        }
    }
    (value, None)
}

fn is_css_number_mantissa(value: &str) -> bool {
    let Some(dot_index) = value.find('.') else {
        return !value.is_empty() && value.chars().all(|character| character.is_ascii_digit());
    };
    if value[dot_index + 1..].contains('.') {
        return false;
    }
    let before = &value[..dot_index];
    let after = &value[dot_index + 1..];
    !after.is_empty()
        && before.chars().all(|character| character.is_ascii_digit())
        && after.chars().all(|character| character.is_ascii_digit())
        && (!before.is_empty() || !after.is_empty())
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
const RESOLUTION_UNITS: &[&str] = &["dpcm", "dppx", "dpi", "x"];

#[cfg(test)]
mod tests {
    use super::*;

    const PVA_L1_COMPONENTS: &[&str] = &[
        "<length>",
        "<number>",
        "<percentage>",
        "<length-percentage>",
        "<string>",
        "<color>",
        "<image>",
        "<url>",
        "<integer>",
        "<angle>",
        "<time>",
        "<resolution>",
        "<transform-function>",
        "<custom-ident>",
        "<transform-list>",
    ];

    const NON_FAST_PATH_NAMED_COLORS: &[&str] = &[
        "antiquewhite",
        "aqua",
        "aquamarine",
        "azure",
        "beige",
        "bisque",
        "blanchedalmond",
        "blueviolet",
        "brown",
        "burlywood",
        "cadetblue",
        "chartreuse",
        "chocolate",
        "coral",
        "cornflowerblue",
        "cornsilk",
        "crimson",
        "darkblue",
        "darkcyan",
        "darkgoldenrod",
        "darkgray",
        "darkgreen",
        "darkgrey",
        "darkkhaki",
        "darkmagenta",
        "darkolivegreen",
        "darkorange",
        "darkorchid",
        "darkred",
        "darksalmon",
        "darkseagreen",
        "darkslateblue",
        "darkslategray",
        "darkslategrey",
        "darkturquoise",
        "darkviolet",
        "deeppink",
        "deepskyblue",
        "dimgray",
        "dimgrey",
        "dodgerblue",
        "firebrick",
        "floralwhite",
        "forestgreen",
        "fuchsia",
        "gainsboro",
        "ghostwhite",
        "gold",
        "goldenrod",
        "greenyellow",
        "honeydew",
        "hotpink",
        "indianred",
        "indigo",
        "ivory",
        "khaki",
        "lavender",
        "lavenderblush",
        "lawngreen",
        "lemonchiffon",
        "lightblue",
        "lightcoral",
        "lightcyan",
        "lightgoldenrodyellow",
        "lightgray",
        "lightgreen",
        "lightgrey",
        "lightpink",
        "lightsalmon",
        "lightseagreen",
        "lightskyblue",
        "lightslategray",
        "lightslategrey",
        "lightsteelblue",
        "lightyellow",
        "lime",
        "limegreen",
        "linen",
        "maroon",
        "mediumaquamarine",
        "mediumblue",
        "mediumorchid",
        "mediumpurple",
        "mediumseagreen",
        "mediumslateblue",
        "mediumspringgreen",
        "mediumturquoise",
        "mediumvioletred",
        "midnightblue",
        "mintcream",
        "mistyrose",
        "moccasin",
        "navajowhite",
        "navy",
        "oldlace",
        "olive",
        "olivedrab",
        "orangered",
        "orchid",
        "palegoldenrod",
        "palegreen",
        "paleturquoise",
        "palevioletred",
        "papayawhip",
        "peachpuff",
        "peru",
        "pink",
        "plum",
        "powderblue",
        "rebeccapurple",
        "rosybrown",
        "royalblue",
        "saddlebrown",
        "salmon",
        "sandybrown",
        "seagreen",
        "seashell",
        "sienna",
        "silver",
        "skyblue",
        "slateblue",
        "slategray",
        "slategrey",
        "snow",
        "springgreen",
        "steelblue",
        "tan",
        "teal",
        "thistle",
        "turquoise",
        "violet",
        "wheat",
        "whitesmoke",
        "yellowgreen",
    ];

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
            ("'<length>'", "env(safe-area-inset-top)"),
            ("'<length>'", "attr(data-gap)"),
            ("'<length>'", "calc(100% - 8px)"),
            ("'<foo>'", "8px"),
            ("'* | <length>'", "8px"),
        ] {
            assert_ne!(
                registered_syntax_match(syntax, value),
                RegisteredSyntaxMatchV0::Rejects,
                "{syntax} should stay silent for {value}"
            );
        }
        assert_eq!(
            registered_syntax_match("'<color>'", "inherit"),
            RegisteredSyntaxMatchV0::Accepts
        );
    }

    #[test]
    fn registered_property_syntax_corpus_matches_expected_tri_state() {
        for (syntax, value, expected) in [
            ("'<color>'", "customvalue", RegisteredSyntaxMatchV0::Rejects),
            ("'<color>'", "red", RegisteredSyntaxMatchV0::Accepts),
            ("'<color>'", "8px", RegisteredSyntaxMatchV0::Rejects),
            ("'<length>'", "red", RegisteredSyntaxMatchV0::Rejects),
            (
                "'<length>'",
                "customvalue",
                RegisteredSyntaxMatchV0::Rejects,
            ),
            (
                "'<custom-ident>'",
                "customvalue",
                RegisteredSyntaxMatchV0::Accepts,
            ),
            (
                "'small | medium'",
                "large",
                RegisteredSyntaxMatchV0::Rejects,
            ),
            (
                "'small | medium | large'",
                "medium",
                RegisteredSyntaxMatchV0::Accepts,
            ),
            (
                "'small | <custom-ident>'",
                "customvalue",
                RegisteredSyntaxMatchV0::Accepts,
            ),
            ("'<number>'", "infinity", RegisteredSyntaxMatchV0::Rejects),
            ("'<number>'", "1e3", RegisteredSyntaxMatchV0::Accepts),
            ("'<integer>'", "1e3", RegisteredSyntaxMatchV0::Rejects),
            ("'<string>'", "\"hello\"", RegisteredSyntaxMatchV0::Accepts),
            ("'<string>'", "hello", RegisteredSyntaxMatchV0::Rejects),
            ("'<url>'", "url(a.png)", RegisteredSyntaxMatchV0::Accepts),
            (
                "'<image>'",
                "linear-gradient(red, blue)",
                RegisteredSyntaxMatchV0::Accepts,
            ),
            ("'<resolution>'", "2dppx", RegisteredSyntaxMatchV0::Accepts),
            (
                "'<transform-function>'",
                "rotate(45deg)",
                RegisteredSyntaxMatchV0::Accepts,
            ),
            (
                "'<transform-list>'",
                "rotate(45deg)",
                RegisteredSyntaxMatchV0::Accepts,
            ),
            ("'<length>+'", "8px", RegisteredSyntaxMatchV0::Accepts),
            ("'<color>#'", "8px", RegisteredSyntaxMatchV0::Rejects),
            (
                "'<length># | red'",
                "30deg",
                RegisteredSyntaxMatchV0::Rejects,
            ),
            (
                "'<length># | red'",
                "customvalue",
                RegisteredSyntaxMatchV0::Rejects,
            ),
        ] {
            assert_eq!(
                registered_syntax_match(syntax, value),
                expected,
                "{syntax} vs {value}"
            );
        }
    }

    #[test]
    fn registered_property_syntax_covers_all_pva_l1_component_names() {
        for component in PVA_L1_COMPONENTS {
            assert!(
                matches!(
                    parse_registered_property_syntax_v0(component),
                    RegisteredPropertySyntaxV0::Supported { .. }
                ),
                "{component} should parse"
            );
        }
    }

    #[test]
    fn bare_idents_are_rejected_by_incompatible_typed_components() {
        for component in PVA_L1_COMPONENTS {
            let expected = if *component == "<custom-ident>" {
                RegisteredSyntaxMatchV0::Accepts
            } else {
                RegisteredSyntaxMatchV0::Rejects
            };
            assert_eq!(registered_syntax_match(component, "customvalue"), expected);
        }
    }

    #[test]
    fn spec_driven_named_color_widening_narrows_against_noncolor_bases() {
        // V2: the feed-driven `is_named_color` recognizes the full `<named-color>`
        // set, so `rebeccapurple` (absent from the historical fast-path stub) is now
        // a ColorKeyword, not an under-determined bare ident. This is a CORRECT
        // narrowing — it positively Accepts against `<color>` AND definitely Rejects
        // against a non-color base (`rebeccapurple` is genuinely not a length). It is
        // NOT a pure false-negative shrink: it introduces a new, correct Rejects.
        assert_eq!(
            classify_registered_property_declared_value_v0("rebeccapurple"),
            DeclaredValueKindV0::ColorKeyword("rebeccapurple".to_string())
        );
        assert_eq!(
            registered_syntax_match("'<color>'", "rebeccapurple"),
            RegisteredSyntaxMatchV0::Accepts
        );
        assert_eq!(
            registered_syntax_match("'<length>'", "rebeccapurple"),
            RegisteredSyntaxMatchV0::Rejects
        );
        // `currentcolor` is a `<color>` keyword, not a `<named-color>`, so the feed
        // alone would miss it; the inline floor keeps it recognized.
        assert_eq!(
            classify_registered_property_declared_value_v0("currentcolor"),
            DeclaredValueKindV0::ColorKeyword("currentcolor".to_string())
        );
    }

    #[test]
    fn container_query_length_units_classify_as_length() {
        // Container-query length units are `<length>`s that LENGTH_UNITS omits.
        // Recognizing them (the first production consumer of the value-lattice
        // recognizer) fixes a false-negative: a registered `<length>` property now
        // accepts `50cqw` instead of silently leaving it undetermined.
        for value in ["50cqw", "10cqh", "1cqi", "100cqmin", "2cqmax"] {
            assert_eq!(
                classify_registered_property_declared_value_v0(value),
                DeclaredValueKindV0::Dimension(DeclaredNumericTypeV0::Length),
                "{value} must classify as a length dimension"
            );
            assert_eq!(
                registered_syntax_match("'<length>'", value),
                RegisteredSyntaxMatchV0::Accepts,
                "{value} must be accepted by a <length> registration"
            );
        }
        // A bare unit token without a number is not a dimension.
        assert_ne!(
            classify_registered_property_declared_value_v0("cqw"),
            DeclaredValueKindV0::Dimension(DeclaredNumericTypeV0::Length)
        );
    }

    #[test]
    fn standard_property_syntax_match_flags_only_definite_keyword_violations() {
        // `box-sizing: content-box | border-box` is a closed alternation in the feed.
        assert_eq!(
            standard_property_syntax_match("box-sizing", "border-box"),
            RegisteredSyntaxMatchV0::Accepts
        );
        assert_eq!(
            standard_property_syntax_match("box-sizing", "inline-box"),
            RegisteredSyntaxMatchV0::Rejects
        );
        // CSS-wide keywords are valid for every property.
        assert_eq!(
            standard_property_syntax_match("box-sizing", "inherit"),
            RegisteredSyntaxMatchV0::Accepts
        );
        // Case-insensitive membership.
        assert_eq!(
            standard_property_syntax_match("box-sizing", "Border-Box"),
            RegisteredSyntaxMatchV0::Accepts
        );
        // Deferred values stay silent because their computed substitution is not
        // statically validatable.
        for value in ["var(--x)", "calc(1px)"] {
            assert_eq!(
                standard_property_syntax_match("box-sizing", value),
                RegisteredSyntaxMatchV0::Unknown,
                "box-sizing: {value} must stay silent"
            );
        }
        for value in ["10px", "content-box border-box"] {
            assert_eq!(
                standard_property_syntax_match("box-sizing", value),
                RegisteredSyntaxMatchV0::Rejects,
                "box-sizing: {value} must be rejected by the complete grammar"
            );
        }
        assert_eq!(
            standard_property_syntax_match("color", "tomato"),
            RegisteredSyntaxMatchV0::Accepts
        );
        assert_eq!(
            standard_property_syntax_match("not-a-real-property", "anything"),
            RegisteredSyntaxMatchV0::Unknown
        );
    }

    #[test]
    fn standard_property_syntax_match_never_rejects_a_spec_listed_value() {
        // No false positives: every keyword the feed lists for a closed-alternation
        // property must Accept against it, as must every CSS-wide keyword.
        let mut checked = 0usize;
        for property in ["box-sizing", "float", "clear", "visibility", "caption-side"] {
            let Some(keywords) = spec_vocabulary().property_keywords(property) else {
                continue;
            };
            for keyword in keywords {
                assert_eq!(
                    standard_property_syntax_match(property, keyword),
                    RegisteredSyntaxMatchV0::Accepts,
                    "{property}: spec-listed {keyword} must accept"
                );
                checked += 1;
            }
            for global in ["inherit", "initial", "unset", "revert", "revert-layer"] {
                assert_eq!(
                    standard_property_syntax_match(property, global),
                    RegisteredSyntaxMatchV0::Accepts
                );
            }
        }
        assert!(
            checked > 0,
            "expected closed-alternation properties in the feed"
        );
    }

    #[test]
    fn standard_property_syntax_match_keeps_vendor_prefixed_values_silent() {
        // A leading `-` marks a legal browser extension the standard closed grammar
        // never lists; it must stay silent, not be flagged as a value violation. The
        // guard is exercised directly on box-sizing (a confirmed closed alternation):
        // without the prefix the same shape would Reject, with it stays Unknown.
        assert_eq!(
            standard_property_syntax_match("box-sizing", "-webkit-border-box"),
            RegisteredSyntaxMatchV0::Unknown
        );
        for value in [
            "-webkit-optimize-contrast",
            "-moz-crisp-edges",
            "-moz-none",
            "-webkit-auto",
        ] {
            assert_ne!(
                standard_property_syntax_match("image-rendering", value),
                RegisteredSyntaxMatchV0::Rejects,
                "vendor-prefixed {value} must not be flagged"
            );
        }
        // A non-prefixed unlisted keyword is still flagged (the typo-catching value).
        assert_eq!(
            standard_property_syntax_match("box-sizing", "inline-box"),
            RegisteredSyntaxMatchV0::Rejects
        );
    }

    #[test]
    fn css_wide_keywords_are_spec_driven_from_the_all_property() {
        // The `all` property's grammar is the authoritative CSS-wide keyword set,
        // including `revert-rule`, which the historical inline floor omits.
        assert_eq!(
            classify_registered_property_declared_value_v0("revert-rule"),
            DeclaredValueKindV0::CssWide
        );
        // A CSS-wide keyword is valid for every property, so the value diagnostic
        // stays silent even on a closed-alternation property (no false positive).
        assert_eq!(
            standard_property_syntax_match("box-sizing", "revert-rule"),
            RegisteredSyntaxMatchV0::Accepts
        );
        // The inline floor still covers the classic keywords if the feed were absent.
        for keyword in ["initial", "inherit", "unset", "revert", "revert-layer"] {
            assert_eq!(
                classify_registered_property_declared_value_v0(keyword),
                DeclaredValueKindV0::CssWide
            );
        }
    }

    #[test]
    fn registry_named_colors_are_accepted_by_color_syntax() {
        for color in NON_FAST_PATH_NAMED_COLORS {
            assert_eq!(
                registered_syntax_match("'<color>'", color),
                RegisteredSyntaxMatchV0::Accepts,
                "{color} must be accepted through the registry grammar"
            );
        }
    }

    #[test]
    fn registered_property_syntax_has_component_accept_silent_reject_matrix() {
        for (syntax, accepts, silent, rejects) in [
            (
                "'<length>'",
                &["16px"][..],
                &["var(--x)", "calc(100% - 8px)"][..],
                &["customvalue", "red", "50%"][..],
            ),
            (
                "'<percentage>'",
                &["50%"][..],
                &["var(--x)"][..],
                &["customvalue", "16px", "red"][..],
            ),
            (
                "'<length-percentage>'",
                &["16px", "50%"][..],
                &["var(--x)"][..],
                &["customvalue", "red", "30deg"][..],
            ),
            (
                "'<number>'",
                &["3", "1.5", "1e3"][..],
                &[][..],
                &["inf", "customvalue", "16px", "red"][..],
            ),
            (
                "'<integer>'",
                &["3", "+5"][..],
                &[][..],
                &["inf", "customvalue", "1.5", "16px"][..],
            ),
            (
                "'<color>'",
                &["#fff", "rgb(1 2 3)", "oklch(60% 0.1 120)", "red"][..],
                &["var(--x)"][..],
                &["customvalue", "16px", "30deg"][..],
            ),
            (
                "'<angle>'",
                &["30deg", "1turn"][..],
                &["var(--x)"][..],
                &["customvalue", "16px", "red"][..],
            ),
            (
                "'<time>'",
                &["200ms", "1s"][..],
                &["var(--x)"][..],
                &["customvalue", "16px", "red"][..],
            ),
            (
                "'<resolution>'",
                &["2dppx", "96dpi"][..],
                &["var(--x)"][..],
                &["customvalue", "16px", "red"][..],
            ),
            (
                "'<image>'",
                &["url(a.png)", "linear-gradient(red, blue)"][..],
                &["var(--x)"][..],
                &["customvalue", "16px"][..],
            ),
            (
                "'<url>'",
                &["url(a.png)"][..],
                &["var(--x)"][..],
                &["customvalue", "16px", "red"][..],
            ),
            (
                "'<transform-function>'",
                &["rotate(45deg)", "translateX(1px)"][..],
                &["var(--x)"][..],
                &["customvalue", "16px", "red"][..],
            ),
            (
                "'<transform-list>'",
                &["rotate(45deg)"][..],
                &["var(--x)"][..],
                &["customvalue", "16px", "red"][..],
            ),
            (
                "'<custom-ident>'",
                &["customvalue", "red"][..],
                &["var(--x)"][..],
                &["0x10", "16px", "50%"][..],
            ),
            (
                "'<string>'",
                &["\"hello\"", "'hello'"][..],
                &["var(--x)"][..],
                &["customvalue", "16px", "red"][..],
            ),
        ] {
            assert_matrix(syntax, accepts, RegisteredSyntaxMatchV0::Accepts);
            assert_matrix(syntax, silent, RegisteredSyntaxMatchV0::Unknown);
            assert_matrix(syntax, rejects, RegisteredSyntaxMatchV0::Rejects);
        }
    }

    #[test]
    fn multiplied_single_token_disjointness_matches_base_component() {
        for (syntax, value) in [
            ("'<length>+'", "red"),
            ("'<length>#'", "red"),
            ("'<color>+'", "8px"),
            ("'<color>#'", "8px"),
            ("'<resolution>+'", "1s"),
            ("'<transform-function>#'", "16px"),
        ] {
            assert_eq!(
                registered_syntax_match(syntax, value),
                RegisteredSyntaxMatchV0::Rejects,
                "{syntax} should preserve single-token disjointness for {value}"
            );
        }
    }

    #[test]
    fn literal_keywords_are_case_insensitive_and_closed_lists_can_reject() {
        assert_eq!(
            registered_syntax_match("'FOO'", "FOO"),
            RegisteredSyntaxMatchV0::Accepts
        );
        assert_eq!(
            registered_syntax_match("'FOO'", "foo"),
            RegisteredSyntaxMatchV0::Accepts
        );
        assert_eq!(
            registered_syntax_match("'FOO | <custom-ident>'", "foo"),
            RegisteredSyntaxMatchV0::Accepts
        );
        assert_eq!(
            registered_syntax_match("'<color>'", "RED"),
            RegisteredSyntaxMatchV0::Accepts
        );
        assert_eq!(
            registered_syntax_match("'<length>'", "1PX"),
            RegisteredSyntaxMatchV0::Accepts
        );
    }

    #[test]
    fn css_number_tokenization_rejects_rust_only_numbers() {
        for value in ["inf", "infinity", "nan", "NaN", "1.", "0x10"] {
            assert_ne!(
                classify_registered_property_declared_value_v0(value),
                DeclaredValueKindV0::Number,
                "{value} must not classify as a CSS number"
            );
            assert_ne!(
                registered_syntax_match("'<number>'", value),
                RegisteredSyntaxMatchV0::Accepts,
                "{value} must not be accepted as a CSS number"
            );
        }

        assert_eq!(
            classify_registered_property_declared_value_v0("+5"),
            DeclaredValueKindV0::Integer
        );
        assert_eq!(
            classify_registered_property_declared_value_v0(".5"),
            DeclaredValueKindV0::Number
        );
        assert_eq!(
            classify_registered_property_declared_value_v0("5e3"),
            DeclaredValueKindV0::Number
        );
        assert_eq!(
            registered_syntax_match("'<integer>'", "5.0"),
            RegisteredSyntaxMatchV0::Rejects
        );
    }

    #[test]
    fn positive_leaf_cross_product_preserves_definite_rejects() {
        for (syntax, value) in [
            ("'<color>'", "8px"),
            ("'<length>'", "red"),
            ("'<string>'", "8px"),
            ("'<url>'", "red"),
            ("'<image>'", "16px"),
            ("'<resolution>'", "1s"),
            ("'<transform-function>'", "16px"),
            ("'<integer>'", "1.5"),
        ] {
            assert_eq!(
                registered_syntax_match(syntax, value),
                RegisteredSyntaxMatchV0::Rejects,
                "{syntax} should reject definite {value}"
            );
        }
    }

    #[test]
    fn registered_property_syntax_no_panic_fuzz_scaffold() {
        let alphabet = [
            "",
            "a",
            "A",
            "-",
            "_",
            "0",
            "9",
            ".",
            "+",
            "-",
            "e",
            "E",
            "(",
            ")",
            "<",
            ">",
            "'",
            "\"",
            "|",
            "#",
            "%",
            " var(--x)",
            "calc(1px + 1%)",
        ];
        for left in alphabet {
            for right in alphabet {
                let value = format!("{left}{right}");
                let syntax = format!("'{left}{right}'");
                let _ = classify_registered_property_declared_value_v0(value.as_str());
                let _ = registered_syntax_match(syntax.as_str(), value.as_str());
                let _ = registered_syntax_match(
                    "'<length> | <color># | <custom-ident>'",
                    value.as_str(),
                );
            }
        }
    }

    fn assert_matrix(syntax: &str, values: &[&str], expected: RegisteredSyntaxMatchV0) {
        for value in values {
            assert_eq!(
                registered_syntax_match(syntax, value),
                expected,
                "{syntax} vs {value}"
            );
        }
    }
}
