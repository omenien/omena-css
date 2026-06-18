//! Region-local CSS value lens and canonical value equality substrate.
//!
//! This crate intentionally accepts declaration value slices plus a base byte
//! offset. It does not accept rules, declarations, or stylesheets, so whole-doc
//! property analysis stays unrepresentable at this layer.

use omena_parser::StyleDialect;

pub mod color;
mod color_names;
mod functions;
pub mod number;

pub use color::{
    StaticSrgbColorWithAlpha, can_shorten_hex_pairs, compress_hex_color_token_text,
    parse_basic_named_static_color_with_alpha, parse_color_function_value, parse_color_mix_value,
    parse_oklab_oklch_value, parse_static_hsl_function_color_with_alpha,
    parse_static_hwb_function_color_with_alpha, parse_static_rgb_function_color_with_alpha,
    parse_static_srgb_color, parse_static_srgb_color_with_alpha, shorten_hex_pairs,
    shortest_static_srgb_color_with_alpha_text,
};
pub use functions::{
    StaticCssFunctionParser, StaticCssFunctionSpec, matching_function_call_end,
    parse_whole_function_value_arguments, parse_whole_function_value_inner,
    split_top_level_value_arguments as split_top_level_value_arguments_owned,
    split_top_level_whitespace_value_components as split_top_level_whitespace_value_components_owned,
    substitute_static_css_function_references_in_value,
    substitute_static_css_function_references_in_value_until_stable,
};
pub use number::{
    compress_number_prefix, compress_numeric_token_text, format_css_number, numeric_prefix_end,
    parse_numeric_value_with_unit, parse_reducible_abs_value, parse_reducible_calc_value,
    parse_reducible_ceil_value, parse_reducible_clamp_value, parse_reducible_exp_value,
    parse_reducible_floor_value, parse_reducible_hypot_value, parse_reducible_log_value,
    parse_reducible_max_value, parse_reducible_min_value, parse_reducible_mod_value,
    parse_reducible_pow_value, parse_reducible_rem_value, parse_reducible_round_to_integer_value,
    parse_reducible_round_value, parse_reducible_sign_value, parse_reducible_sqrt_value,
    reduce_static_numeric_expression,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ValueByteSpanV0 {
    pub start: usize,
    pub end: usize,
}

impl ValueByteSpanV0 {
    pub const fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ValueSegmentV0<'a> {
    pub text: &'a str,
    pub span: ValueByteSpanV0,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NumericValueV0<'a> {
    pub value: f64,
    pub unit: &'a str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SrgbColor {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

pub fn parse_basic_named_srgb_color(text: &str) -> Option<SrgbColor> {
    color_names::parse_basic_named_srgb_color(text)
}

pub fn shortest_named_srgb_color(color: SrgbColor) -> Option<&'static str> {
    color_names::shortest_named_srgb_color(color)
}

#[derive(Debug, Clone, PartialEq)]
pub enum ValueNodeV0<'a> {
    Raw {
        span: ValueByteSpanV0,
        text: &'a str,
    },
    Number {
        span: ValueByteSpanV0,
        value: NumericValueV0<'a>,
    },
    Color {
        span: ValueByteSpanV0,
        text: &'a str,
    },
    Keyword {
        span: ValueByteSpanV0,
        text: &'a str,
    },
    List {
        span: ValueByteSpanV0,
        items: Vec<usize>,
    },
    Function {
        span: ValueByteSpanV0,
        name: &'a str,
        arguments: Vec<usize>,
    },
    SassMap {
        span: ValueByteSpanV0,
        text: &'a str,
    },
    SassList {
        span: ValueByteSpanV0,
        items: Vec<usize>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct DeclarationValueLensV0<'a> {
    source: &'a str,
    base_offset: usize,
    root_node: usize,
    nodes: Vec<ValueNodeV0<'a>>,
}

impl<'a> DeclarationValueLensV0<'a> {
    pub fn source(&self) -> &'a str {
        self.source
    }

    pub const fn base_offset(&self) -> usize {
        self.base_offset
    }

    pub fn root(&self) -> &ValueNodeV0<'a> {
        &self.nodes[self.root_node]
    }

    pub fn nodes(&self) -> &[ValueNodeV0<'a>] {
        self.nodes.as_slice()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct CanonicalCssValueV0 {
    pub serialized: String,
}

pub fn declaration_value_lens(value: &str, base_offset: usize) -> DeclarationValueLensV0<'_> {
    let span = ValueByteSpanV0::new(base_offset, base_offset + value.len());
    let mut nodes = Vec::new();
    let root_node = push_value_node(value, span, &mut nodes);
    DeclarationValueLensV0 {
        source: value,
        base_offset,
        root_node,
        nodes,
    }
}

pub fn css_values_canonically_equal(left: &str, right: &str) -> bool {
    if left.trim() == right.trim() {
        return true;
    }
    let Some(left) = canonicalize_css_value(left) else {
        return false;
    };
    let Some(right) = canonicalize_css_value(right) else {
        return false;
    };
    left == right
}

pub fn css_number_is_zero(number: &str) -> bool {
    number::css_number_is_zero(number)
}

pub fn canonicalize_css_value(value: &str) -> Option<CanonicalCssValueV0> {
    let value = value.trim();
    if value.is_empty()
        || value.contains("var(")
        || value.contains("VAR(")
        || value.contains("env(")
        || value.contains("ENV(")
    {
        return None;
    }
    if let Some(serialized) = canonicalize_static_color_value(value) {
        return Some(CanonicalCssValueV0 { serialized });
    }
    if value.contains('(') || value.contains(')') {
        return None;
    }

    let numeric = parse_numeric_value(value)?;
    canonicalize_numeric_value(numeric)
}

fn canonicalize_static_color_value(value: &str) -> Option<String> {
    parse_static_srgb_color_with_alpha(value)
        .or_else(|| parse_static_rgb_function_color_with_alpha(value))
        .or_else(|| parse_static_hsl_function_color_with_alpha(value))
        .or_else(|| parse_static_hwb_function_color_with_alpha(value))
        .map(shortest_static_srgb_color_with_alpha_text)
        .or_else(|| {
            parse_color_function_value(value)
                .or_else(|| parse_color_mix_value(value))
                .or_else(|| parse_oklab_oklch_value(value))
                .and_then(|value| canonicalize_static_color_value(value.as_str()))
        })
}

pub fn split_top_level_value_arguments<'a>(
    inner: &'a str,
    base_offset: usize,
) -> Option<Vec<ValueSegmentV0<'a>>> {
    split_top_level_segments(inner, base_offset, SegmentDelimiterV0::Comma)
}

pub fn split_top_level_whitespace_value_components<'a>(
    value: &'a str,
    base_offset: usize,
) -> Option<Vec<ValueSegmentV0<'a>>> {
    split_top_level_segments(value, base_offset, SegmentDelimiterV0::Whitespace)
}

fn push_value_node<'a>(
    value: &'a str,
    span: ValueByteSpanV0,
    nodes: &mut Vec<ValueNodeV0<'a>>,
) -> usize {
    let node = if let Some(number) = parse_numeric_value(value) {
        ValueNodeV0::Number {
            span,
            value: number,
        }
    } else if looks_like_unknown_function(value) {
        ValueNodeV0::Raw { span, text: value }
    } else if looks_like_sass_collection(value) {
        ValueNodeV0::SassList {
            span,
            items: Vec::new(),
        }
    } else if value.starts_with('#') {
        ValueNodeV0::Color { span, text: value }
    } else if is_css_keyword_like(value) {
        ValueNodeV0::Keyword { span, text: value }
    } else {
        ValueNodeV0::Raw { span, text: value }
    };
    nodes.push(node);
    nodes.len() - 1
}

fn canonicalize_numeric_value(value: NumericValueV0<'_>) -> Option<CanonicalCssValueV0> {
    let unit = value.unit.trim();
    let normalized_unit = unit.to_ascii_lowercase();
    if !value.value.is_finite() {
        return None;
    }
    if value.value == 0.0 {
        if unit.is_empty() || is_absolute_zero_collapsible_unit(&normalized_unit) {
            return Some(CanonicalCssValueV0 {
                serialized: "0".to_string(),
            });
        }
        if normalized_unit == "%" {
            return Some(CanonicalCssValueV0 {
                serialized: "0%".to_string(),
            });
        }
        return None;
    }
    if unit.is_empty() || is_absolute_zero_collapsible_unit(&normalized_unit) {
        return Some(CanonicalCssValueV0 {
            serialized: format!("{}{}", format_css_number(value.value), normalized_unit),
        });
    }
    None
}

fn parse_numeric_value(text: &str) -> Option<NumericValueV0<'_>> {
    number::parse_numeric_value_with_unit(text)
}

fn is_absolute_zero_collapsible_unit(unit: &str) -> bool {
    matches!(
        unit,
        "px" | "cm" | "mm" | "q" | "in" | "pc" | "pt" | "deg" | "grad" | "rad" | "turn"
    )
}

fn looks_like_unknown_function(value: &str) -> bool {
    value.contains('(') || value.contains(')')
}

fn looks_like_sass_collection(value: &str) -> bool {
    value.contains('{') || value.contains('}')
}

fn is_css_keyword_like(value: &str) -> bool {
    value
        .chars()
        .all(|ch| ch == '-' || ch == '_' || ch.is_ascii_alphabetic())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SegmentDelimiterV0 {
    Comma,
    Whitespace,
}

fn split_top_level_segments<'a>(
    value: &'a str,
    base_offset: usize,
    delimiter: SegmentDelimiterV0,
) -> Option<Vec<ValueSegmentV0<'a>>> {
    let mut segments = Vec::new();
    let mut start = 0usize;
    let mut index = 0usize;
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;
    let mut brace_depth = 0usize;
    let mut quote: Option<char> = None;
    let mut escaped = false;

    while index < value.len() {
        let ch = value[index..].chars().next()?;
        if let Some(active_quote) = quote {
            index += ch.len_utf8();
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
                index += ch.len_utf8();
            }
            '(' => {
                paren_depth += 1;
                index += ch.len_utf8();
            }
            ')' => {
                paren_depth = paren_depth.checked_sub(1)?;
                index += ch.len_utf8();
            }
            '[' => {
                bracket_depth += 1;
                index += ch.len_utf8();
            }
            ']' => {
                bracket_depth = bracket_depth.checked_sub(1)?;
                index += ch.len_utf8();
            }
            '{' => {
                brace_depth += 1;
                index += ch.len_utf8();
            }
            '}' => {
                brace_depth = brace_depth.checked_sub(1)?;
                index += ch.len_utf8();
            }
            ',' if delimiter == SegmentDelimiterV0::Comma
                && paren_depth == 0
                && bracket_depth == 0
                && brace_depth == 0 =>
            {
                if !push_trimmed_segment(value, base_offset, start, index, &mut segments)? {
                    return None;
                }
                index += ch.len_utf8();
                start = index;
            }
            _ if delimiter == SegmentDelimiterV0::Whitespace
                && ch.is_whitespace()
                && paren_depth == 0
                && bracket_depth == 0
                && brace_depth == 0 =>
            {
                push_trimmed_segment(value, base_offset, start, index, &mut segments)?;
                index += ch.len_utf8();
                start = index;
            }
            _ => {
                index += ch.len_utf8();
            }
        }
    }

    if quote.is_some() || paren_depth != 0 || bracket_depth != 0 || brace_depth != 0 {
        return None;
    }
    let pushed = push_trimmed_segment(value, base_offset, start, value.len(), &mut segments)?;
    if delimiter == SegmentDelimiterV0::Comma && !pushed {
        return None;
    }
    (!segments.is_empty()).then_some(segments)
}

fn push_trimmed_segment<'a>(
    source: &'a str,
    base_offset: usize,
    start: usize,
    end: usize,
    segments: &mut Vec<ValueSegmentV0<'a>>,
) -> Option<bool> {
    if start > end || end > source.len() {
        return None;
    }
    let (start, end) = trim_byte_span(source, start, end);
    if start >= end {
        return Some(false);
    }
    segments.push(ValueSegmentV0 {
        text: &source[start..end],
        span: ValueByteSpanV0::new(base_offset + start, base_offset + end),
    });
    Some(true)
}

fn trim_byte_span(source: &str, mut start: usize, mut end: usize) -> (usize, usize) {
    while start < end {
        let Some(ch) = source[start..end].chars().next() else {
            break;
        };
        if !ch.is_whitespace() {
            break;
        }
        start += ch.len_utf8();
    }
    while start < end {
        let Some(ch) = source[start..end].chars().next_back() else {
            break;
        };
        if !ch.is_whitespace() {
            break;
        }
        end -= ch.len_utf8();
    }
    (start, end)
}

pub fn dialect_is_supported_for_raw_value_lens(dialect: StyleDialect) -> bool {
    matches!(
        dialect,
        StyleDialect::Css | StyleDialect::Scss | StyleDialect::Sass | StyleDialect::Less
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn value_lens_is_region_local_and_keeps_base_offsets() {
        let lens = declaration_value_lens("  0px  ", 20);
        assert_eq!(lens.source(), "  0px  ");
        assert_eq!(lens.base_offset(), 20);
        assert_eq!(lens.nodes().len(), 1);
        assert!(matches!(lens.root(), ValueNodeV0::Number { .. }));

        let segments = split_top_level_whitespace_value_components("  0px  var(--x)  ", 10);
        assert!(segments.is_some());
        let segments = segments.unwrap_or_default();
        assert_eq!(segments[0].text, "0px");
        assert_eq!(segments[0].span, ValueByteSpanV0::new(12, 15));
        assert_eq!(segments[1].text, "var(--x)");
        assert_eq!(segments[1].span, ValueByteSpanV0::new(17, 25));
    }

    #[test]
    fn canonical_equality_collapses_absolute_zero_but_not_percent_or_relative_units() {
        assert!(css_values_canonically_equal("0px", "0"));
        assert!(css_values_canonically_equal("0deg", "0"));
        assert!(css_values_canonically_equal("+0.000PX", "-0"));
        assert!(!css_values_canonically_equal("0%", "0"));
        assert!(!css_values_canonically_equal("0em", "0"));
        assert!(!css_values_canonically_equal("0cqw", "0"));
    }

    #[test]
    fn canonical_equality_uses_raw_identity_without_typed_transfer_for_unknowns() {
        assert!(css_values_canonically_equal("var(--x)", "var(--x)"));
        assert!(!css_values_canonically_equal("var(--x)", "0"));
        assert!(!css_values_canonically_equal("calc(0px)", "0"));
        assert_eq!(canonicalize_css_value("env(safe-area-inset-top)"), None);
    }

    #[test]
    fn canonical_equality_tracks_static_color_values() {
        assert_eq!(
            canonicalize_css_value("#ff0000").map(|value| value.serialized),
            Some("red".to_string())
        );
        assert!(css_values_canonically_equal("#f00", "rgb(255 0 0)"));
        assert_eq!(
            canonicalize_css_value("color-mix(in srgb, red 50%, blue 50%)")
                .map(|value| value.serialized),
            Some("purple".to_string())
        );
        assert_eq!(
            canonicalize_css_value("color(srgb 1 0 0 / .5)").map(|value| value.serialized),
            Some("#ff000080".to_string())
        );
    }

    #[test]
    fn css_number_zero_predicate_accepts_css_numeric_spelling_only() {
        assert!(css_number_is_zero("0"));
        assert!(css_number_is_zero("-.0"));
        assert!(css_number_is_zero("+0e10"));
        assert!(!css_number_is_zero("-"));
        assert!(!css_number_is_zero("."));
        assert!(!css_number_is_zero("0px"));
        assert!(!css_number_is_zero("1"));
    }

    #[test]
    fn value_splitter_respects_depth_quotes_brackets_and_braces() {
        let args = split_top_level_value_arguments("rgb(0, 0, 0), \"a,b\", [x,y], {a:b}", 3);
        assert!(args.is_some());
        let args = args.unwrap_or_default();
        assert_eq!(
            args.iter().map(|segment| segment.text).collect::<Vec<_>>(),
            vec!["rgb(0, 0, 0)", "\"a,b\"", "[x,y]", "{a:b}"]
        );
        assert_eq!(args[0].span, ValueByteSpanV0::new(3, 15));
        assert_eq!(args[3].span, ValueByteSpanV0::new(31, 36));

        let parts = split_top_level_whitespace_value_components(
            "1px minmax(0, 1fr) \"a b\" [line name]",
            0,
        );
        assert!(parts.is_some());
        let parts = parts.unwrap_or_default();
        assert_eq!(
            parts.iter().map(|segment| segment.text).collect::<Vec<_>>(),
            vec!["1px", "minmax(0, 1fr)", "\"a b\"", "[line name]"]
        );
    }

    #[test]
    fn numeric_kernel_reduces_static_css_expressions() {
        assert_eq!(
            reduce_static_numeric_expression("1px + 2px").as_deref(),
            Some("3px")
        );
        assert_eq!(
            parse_reducible_calc_value("calc((2px + 4px) / 2)").as_deref(),
            Some("3px")
        );
        assert_eq!(
            parse_reducible_sign_value("sign(-2px)").as_deref(),
            Some("-1")
        );
        assert_eq!(
            parse_reducible_ceil_value("ceil(1.2px)").as_deref(),
            Some("2px")
        );
        assert_eq!(
            parse_reducible_floor_value("floor(1.8px)").as_deref(),
            Some("1px")
        );
        assert_eq!(
            parse_reducible_round_to_integer_value("round(1.5px)").as_deref(),
            Some("2px")
        );
        assert_eq!(
            parse_reducible_clamp_value("clamp(1px, 3px, 2px)").as_deref(),
            Some("2px")
        );
        assert_eq!(
            compress_numeric_token_text("+001.5000px").as_deref(),
            Some("1.5px")
        );
    }

    #[test]
    fn static_function_substitution_respects_function_name_boundaries() {
        fn parse_static_max(value: &str) -> Option<String> {
            parse_reducible_max_value(value)
        }

        assert_eq!(
            substitute_static_css_function_references_in_value(
                "max(1px, 2px)",
                &[("max", parse_static_max)]
            )
            .as_deref(),
            Some("2px")
        );
        assert_eq!(
            substitute_static_css_function_references_in_value(
                "calc(max(1px, 2px) + 1px)",
                &[("max", parse_static_max)]
            )
            .as_deref(),
            Some("calc(2px + 1px)")
        );
        assert_eq!(
            substitute_static_css_function_references_in_value(
                "math.max(1px, 2px)",
                &[("max", parse_static_max)]
            ),
            None
        );
        assert_eq!(
            substitute_static_css_function_references_in_value(
                "mymax(1px, 2px)",
                &[("max", parse_static_max)]
            ),
            None
        );
    }

    #[test]
    fn static_function_substitution_reduces_nested_calls_before_outer_calls() {
        fn parse_static_outer(value: &str) -> Option<String> {
            let inner = parse_whole_function_value_inner(value, "outer")?.trim();
            Some(if inner == "ok" { "good" } else { "bad" }.to_string())
        }

        fn parse_static_inner(value: &str) -> Option<String> {
            (value.trim() == "inner()").then(|| "ok".to_string())
        }

        assert_eq!(
            substitute_static_css_function_references_in_value_until_stable(
                "outer(inner())",
                &[("outer", parse_static_outer), ("inner", parse_static_inner),],
            )
            .as_deref(),
            Some("good")
        );
    }

    #[test]
    fn color_kernel_parses_static_color_values() {
        assert!(parse_static_srgb_color("rebeccapurple").is_some());
        assert_eq!(
            compress_hex_color_token_text("#ff0000").as_deref(),
            Some("red")
        );
        assert_eq!(
            parse_color_function_value("color(srgb 1 0 0 / .5)").as_deref(),
            Some("rgb(255 0 0 / .5)")
        );
        assert_eq!(
            parse_color_mix_value("color-mix(in srgb, red 50%, blue 50%)").as_deref(),
            Some("rgb(128 0 128)")
        );
        assert_eq!(
            parse_oklab_oklch_value("oklab(1 0 0)").as_deref(),
            Some("rgb(255 255 255)")
        );
    }
}
