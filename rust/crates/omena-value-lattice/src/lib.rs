//! Region-local CSS value lens and canonical value equality substrate.
//!
//! This crate intentionally accepts declaration value slices plus a base byte
//! offset. It does not accept rules, declarations, or stylesheets, so whole-doc
//! property analysis stays unrepresentable at this layer.

use omena_parser::StyleDialect;

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
    parse_numeric_value(number).is_some_and(|value| value.unit.is_empty() && value.value == 0.0)
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
    if value.contains('(') || value.contains(')') {
        return None;
    }

    let numeric = parse_numeric_value(value)?;
    canonicalize_numeric_value(numeric)
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
    let text = text.trim();
    let split = numeric_prefix_end(text)?;
    if split == 0 {
        return None;
    }
    let number = &text[..split];
    let unit = &text[split..];
    if unit
        .chars()
        .any(|ch| !(ch == '%' || ch.is_ascii_alphabetic()))
    {
        return None;
    }
    let value = number.parse::<f64>().ok()?;
    value.is_finite().then_some(NumericValueV0 { value, unit })
}

fn numeric_prefix_end(text: &str) -> Option<usize> {
    let bytes = text.as_bytes();
    let mut index = 0;

    if matches!(bytes.get(index), Some(b'+') | Some(b'-')) {
        index += 1;
    }

    let integer_start = index;
    while matches!(bytes.get(index), Some(b'0'..=b'9')) {
        index += 1;
    }
    let saw_integer_digit = index > integer_start;

    if bytes.get(index) == Some(&b'.') {
        index += 1;
        let fraction_start = index;
        while matches!(bytes.get(index), Some(b'0'..=b'9')) {
            index += 1;
        }
        if !saw_integer_digit && index == fraction_start {
            return None;
        }
    } else if !saw_integer_digit {
        return None;
    }

    if matches!(bytes.get(index), Some(b'e') | Some(b'E')) {
        let exponent_marker = index;
        let mut exponent_index = index + 1;
        if matches!(bytes.get(exponent_index), Some(b'+') | Some(b'-')) {
            exponent_index += 1;
        }
        let exponent_digit_start = exponent_index;
        while matches!(bytes.get(exponent_index), Some(b'0'..=b'9')) {
            exponent_index += 1;
        }
        if exponent_index > exponent_digit_start {
            index = exponent_index;
        } else {
            index = exponent_marker;
        }
    }

    Some(index)
}

fn format_css_number(value: f64) -> String {
    if value.fract() == 0.0 {
        return format!("{value:.0}");
    }
    let formatted = format!("{value:.6}");
    formatted
        .trim_end_matches('0')
        .trim_end_matches('.')
        .to_string()
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
}
