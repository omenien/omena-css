use crate::helpers::values::{
    parse_whole_function_value_arguments, parse_whole_function_value_inner,
};

pub(crate) fn parse_reducible_calc_value(value: &str) -> Option<String> {
    let inner = parse_whole_function_value_inner(value, "calc")?;
    let reduced = parse_reducible_numeric_expression(inner)?;
    Some(format_numeric_value_with_unit(reduced))
}

pub(crate) fn parse_reducible_abs_value(value: &str) -> Option<String> {
    let inner = parse_whole_function_value_inner(value, "abs")?;
    let parsed = parse_reducible_numeric_expression(inner)?;
    Some(format_numeric_value_with_unit(NumericValueWithUnit {
        value: parsed.value.abs(),
        unit: parsed.unit,
    }))
}

pub(crate) fn parse_reducible_sign_value(value: &str) -> Option<String> {
    let inner = parse_whole_function_value_inner(value, "sign")?;
    let parsed = parse_reducible_numeric_expression(inner)?;
    let value = if parsed.value > 0.0 {
        1.0
    } else if parsed.value < 0.0 {
        -1.0
    } else {
        0.0
    };
    Some(format_css_number(value))
}

pub(crate) fn parse_reducible_min_value(value: &str) -> Option<String> {
    parse_reducible_extreme_value(value, "min", f64::min)
}

pub(crate) fn parse_reducible_max_value(value: &str) -> Option<String> {
    parse_reducible_extreme_value(value, "max", f64::max)
}

pub(crate) fn parse_reducible_clamp_value(value: &str) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, "clamp")?;
    let [minimum, preferred, maximum] = arguments.as_slice() else {
        return None;
    };
    let minimum = parse_numeric_value_with_unit(minimum.trim())?;
    let preferred = parse_numeric_value_with_unit(preferred.trim())?;
    let maximum = parse_numeric_value_with_unit(maximum.trim())?;
    if preferred.unit != minimum.unit || maximum.unit != minimum.unit {
        return None;
    }
    let selected = preferred.value.min(maximum.value).max(minimum.value);
    Some(format!("{}{}", format_css_number(selected), minimum.unit))
}

fn parse_reducible_extreme_value(
    value: &str,
    function_name: &str,
    reduce: fn(f64, f64) -> f64,
) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, function_name)?;
    let first = arguments.first()?;
    let first = parse_numeric_value_with_unit(first.trim())?;
    let mut selected = first.value;
    let unit = first.unit;

    for argument in arguments.iter().skip(1) {
        let candidate = parse_numeric_value_with_unit(argument.trim())?;
        if candidate.unit != unit {
            return None;
        }
        selected = reduce(selected, candidate.value);
    }

    Some(format!("{}{}", format_css_number(selected), unit))
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct NumericValueWithUnit<'a> {
    pub(crate) value: f64,
    pub(crate) unit: &'a str,
}

pub(crate) fn parse_numeric_value_with_unit(text: &str) -> Option<NumericValueWithUnit<'_>> {
    let text = text.trim();
    let mut parser = NumericExpressionParser::new(text);
    let parsed = parser.parse_number()?;
    parser.skip_whitespace();
    (parser.is_eof()).then_some(parsed)
}

fn parse_reducible_numeric_expression(inner: &str) -> Option<NumericValueWithUnit<'_>> {
    let mut parser = NumericExpressionParser::new(inner);
    let parsed = parser.parse_expression()?;
    parser.skip_whitespace();
    parser.is_eof().then_some(parsed)
}

struct NumericExpressionParser<'a> {
    text: &'a str,
    index: usize,
}

impl<'a> NumericExpressionParser<'a> {
    fn new(text: &'a str) -> Self {
        Self { text, index: 0 }
    }

    fn parse_expression(&mut self) -> Option<NumericValueWithUnit<'a>> {
        let mut left = self.parse_term()?;
        loop {
            self.skip_whitespace();
            let Some(operator) = self.peek_char().filter(|ch| matches!(ch, '+' | '-')) else {
                break;
            };
            self.index += operator.len_utf8();
            let right = self.parse_term()?;
            left = combine_numeric_additive(left, right, operator)?;
        }
        Some(left)
    }

    fn parse_term(&mut self) -> Option<NumericValueWithUnit<'a>> {
        let mut left = self.parse_factor()?;
        loop {
            self.skip_whitespace();
            let Some(operator) = self.peek_char().filter(|ch| matches!(ch, '*' | '/')) else {
                break;
            };
            self.index += operator.len_utf8();
            let right = self.parse_factor()?;
            left = combine_numeric_multiplicative(left, right, operator)?;
        }
        Some(left)
    }

    fn parse_factor(&mut self) -> Option<NumericValueWithUnit<'a>> {
        self.skip_whitespace();
        if self.consume_char('(') {
            let parsed = self.parse_expression()?;
            self.skip_whitespace();
            self.consume_char(')').then_some(parsed)
        } else {
            self.parse_number()
        }
    }

    fn parse_number(&mut self) -> Option<NumericValueWithUnit<'a>> {
        self.skip_whitespace();
        let start = self.index;
        let split = numeric_prefix_end(&self.text[start..])?;
        let number_end = start + split;
        let unit_start = number_end;
        self.index = number_end;
        if self.peek_char() == Some('%') {
            self.index += '%'.len_utf8();
        } else {
            while self.peek_char().is_some_and(is_css_numeric_unit_continue) {
                let ch = self.peek_char()?;
                self.index += ch.len_utf8();
            }
        }
        let number = &self.text[start..number_end];
        let unit = &self.text[unit_start..self.index];
        let value = number.parse::<f64>().ok()?;
        value
            .is_finite()
            .then_some(NumericValueWithUnit { value, unit })
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.peek_char() {
            if !ch.is_whitespace() {
                break;
            }
            self.index += ch.len_utf8();
        }
    }

    fn consume_char(&mut self, expected: char) -> bool {
        if self.peek_char() == Some(expected) {
            self.index += expected.len_utf8();
            true
        } else {
            false
        }
    }

    fn peek_char(&self) -> Option<char> {
        self.text[self.index..].chars().next()
    }

    fn is_eof(&self) -> bool {
        self.index == self.text.len()
    }
}

fn combine_numeric_additive<'a>(
    left: NumericValueWithUnit<'a>,
    right: NumericValueWithUnit<'a>,
    operator: char,
) -> Option<NumericValueWithUnit<'a>> {
    if left.unit != right.unit {
        return None;
    }
    let value = if operator == '+' {
        left.value + right.value
    } else {
        left.value - right.value
    };
    Some(NumericValueWithUnit {
        value,
        unit: left.unit,
    })
}

fn combine_numeric_multiplicative<'a>(
    left: NumericValueWithUnit<'a>,
    right: NumericValueWithUnit<'a>,
    operator: char,
) -> Option<NumericValueWithUnit<'a>> {
    match operator {
        '*' if left.unit.is_empty() && right.unit.is_empty() => Some(NumericValueWithUnit {
            value: left.value * right.value,
            unit: "",
        }),
        '*' if left.unit.is_empty() => Some(NumericValueWithUnit {
            value: left.value * right.value,
            unit: right.unit,
        }),
        '*' if right.unit.is_empty() => Some(NumericValueWithUnit {
            value: left.value * right.value,
            unit: left.unit,
        }),
        '/' if right.unit.is_empty() && right.value != 0.0 => Some(NumericValueWithUnit {
            value: left.value / right.value,
            unit: left.unit,
        }),
        _ => None,
    }
}

fn format_numeric_value_with_unit(value: NumericValueWithUnit<'_>) -> String {
    format!("{}{}", format_css_number(value.value), value.unit)
}

fn is_css_numeric_unit_continue(ch: char) -> bool {
    ch.is_ascii_alphabetic()
}

pub(crate) fn format_css_number(value: f64) -> String {
    if value.fract() == 0.0 {
        return format!("{value:.0}");
    }
    let formatted = format!("{value:.6}");
    formatted
        .trim_end_matches('0')
        .trim_end_matches('.')
        .to_string()
}

pub(crate) fn numeric_prefix_end(text: &str) -> Option<usize> {
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

pub(crate) fn compress_number_prefix(number: &str) -> String {
    let (sign, unsigned) = match number.as_bytes().first() {
        Some(b'+') | Some(b'-') => (&number[..1], &number[1..]),
        _ => ("", number),
    };
    let sign = if sign == "+" || is_zero_number_prefix(unsigned) {
        ""
    } else {
        sign
    };
    let (mantissa, exponent) = split_number_exponent(unsigned);
    let compressed_mantissa = compress_decimal_mantissa(mantissa);
    let mut compressed = format!("{sign}{compressed_mantissa}");

    if let Some(exponent) = exponent {
        let normalized_exponent = normalize_exponent_suffix(exponent);
        if normalized_exponent != "0" && !is_zero_number_prefix(&compressed) {
            compressed.push('e');
            compressed.push_str(&normalized_exponent);
        }
    }

    compressed
}

fn split_number_exponent(number: &str) -> (&str, Option<&str>) {
    if let Some(index) = number.find(['e', 'E']) {
        (&number[..index], Some(&number[index + 1..]))
    } else {
        (number, None)
    }
}

fn compress_decimal_mantissa(mantissa: &str) -> String {
    let Some((before_dot, after_dot)) = mantissa.split_once('.') else {
        return compress_integer_digits(mantissa);
    };

    let trimmed_fraction = after_dot.trim_end_matches('0');
    let compressed_integer = compress_integer_digits(before_dot);
    let mut compressed_unsigned = if trimmed_fraction.is_empty() {
        compressed_integer
    } else {
        format!("{compressed_integer}.{trimmed_fraction}")
    };

    if let Some(rest) = compressed_unsigned.strip_prefix("0.") {
        compressed_unsigned = format!(".{rest}");
    }

    if compressed_unsigned.is_empty() {
        compressed_unsigned.push('0');
    }

    compressed_unsigned
}

fn compress_integer_digits(digits: &str) -> String {
    let trimmed = digits.trim_start_matches('0');
    if trimmed.is_empty() {
        "0".to_string()
    } else {
        trimmed.to_string()
    }
}

fn normalize_exponent_suffix(exponent: &str) -> String {
    let (sign, digits) = match exponent.as_bytes().first() {
        Some(b'+') => ("", &exponent[1..]),
        Some(b'-') => ("-", &exponent[1..]),
        _ => ("", exponent),
    };
    let digits = digits.trim_start_matches('0');
    let digits = if digits.is_empty() { "0" } else { digits };
    if digits == "0" {
        digits.to_string()
    } else {
        format!("{sign}{digits}")
    }
}

fn is_zero_number_prefix(number: &str) -> bool {
    number.chars().all(|ch| matches!(ch, '0' | '.' | '+' | '-'))
}
