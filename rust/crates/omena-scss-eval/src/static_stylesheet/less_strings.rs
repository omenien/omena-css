use omena_value_lattice::{parse_whole_function_value_arguments, parse_whole_function_value_inner};

use super::model::StaticLessResolvedValue;

pub(super) fn parse_static_less_replace_value(value: &str) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, "replace")?;
    let (input, pattern, replacement, flags) = match arguments.as_slice() {
        [input, pattern, replacement] => {
            (input.as_str(), pattern.as_str(), replacement.as_str(), None)
        }
        [input, pattern, replacement, flags] => (
            input.as_str(),
            pattern.as_str(),
            replacement.as_str(),
            Some(flags.as_str()),
        ),
        _ => return None,
    };
    let input = static_less_string_argument(input.trim())?;
    let pattern = static_less_string_argument(pattern.trim())?.text;
    let replacement = static_less_string_argument(replacement.trim())?.text;
    if !static_less_replace_pattern_is_literal(pattern.as_str())
        || replacement.contains('$')
        || replacement
            .chars()
            .any(|ch| matches!(ch, '\n' | '\r' | '\u{000c}'))
    {
        return None;
    }
    let flags = flags
        .map(|flags| static_less_replace_flags(flags.trim()))
        .unwrap_or(Some(StaticLessReplaceFlags {
            global: false,
            case_insensitive: false,
        }))?;
    if flags.case_insensitive
        && (!input.text.is_ascii() || !pattern.is_ascii() || !replacement.is_ascii())
    {
        return None;
    }
    if pattern.is_empty() && flags.global {
        return None;
    }

    let output = static_less_replace_literal(
        input.text.as_str(),
        pattern.as_str(),
        replacement.as_str(),
        flags,
    )?;
    input.render(output.as_str())
}

pub(super) fn parse_static_less_format_value(value: &str) -> Option<String> {
    let arguments = parse_whole_function_value_arguments(value, "%")?;
    let [format, replacements @ ..] = arguments.as_slice() else {
        return None;
    };
    let format = static_less_string_argument(format.trim())?;
    let mut replacement_index = 0usize;
    let mut output = String::new();
    let mut chars = format.text.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch != '%' {
            output.push(ch);
            continue;
        }

        let Some(specifier) = chars.next() else {
            output.push('%');
            break;
        };
        if specifier == '%' {
            output.push('%');
            continue;
        }
        if !matches!(specifier, 's' | 'S' | 'd' | 'D' | 'a' | 'A') {
            return None;
        }

        let Some(replacement) = replacements.get(replacement_index) else {
            output.push('%');
            output.push(specifier);
            continue;
        };
        replacement_index += 1;
        let replacement = static_less_format_argument_text(replacement.trim())?;
        if specifier.is_ascii_uppercase() {
            output.push_str(percent_encode_static_less_escape_value(replacement.as_str()).as_str());
        } else {
            output.push_str(replacement.as_str());
        }
    }

    format.render(output.as_str())
}

fn static_less_format_argument_text(value: &str) -> Option<String> {
    if let Some(argument) = static_less_string_argument(value) {
        return Some(argument.text);
    }
    (!value.is_empty()
        && !value
            .chars()
            .any(|ch| matches!(ch, '\n' | '\r' | '\u{000c}')))
    .then(|| value.to_string())
}

#[derive(Debug, Clone)]
struct StaticLessStringArgument {
    text: String,
    quote: Option<char>,
    escaped: bool,
}

impl StaticLessStringArgument {
    fn render(&self, output: &str) -> Option<String> {
        if self.escaped || self.quote.is_none() {
            return static_less_unquoted_string_argument_is_safe(output)
                .then(|| output.to_string());
        }
        let quote = self.quote?;
        if output
            .chars()
            .any(|ch| ch == quote || matches!(ch, '\\' | '\n' | '\r' | '\u{000c}'))
        {
            return None;
        }
        Some(format!("{quote}{output}{quote}"))
    }
}

#[derive(Debug, Clone, Copy)]
struct StaticLessReplaceFlags {
    global: bool,
    case_insensitive: bool,
}

fn static_less_replace_flags(value: &str) -> Option<StaticLessReplaceFlags> {
    let text = static_less_string_argument(value)?.text;
    let mut flags = StaticLessReplaceFlags {
        global: false,
        case_insensitive: false,
    };
    for ch in text.chars() {
        match ch {
            'g' if !flags.global => flags.global = true,
            'i' if !flags.case_insensitive => flags.case_insensitive = true,
            _ => return None,
        }
    }
    Some(flags)
}

fn static_less_replace_literal(
    input: &str,
    pattern: &str,
    replacement: &str,
    flags: StaticLessReplaceFlags,
) -> Option<String> {
    if pattern.is_empty() {
        return Some(format!("{replacement}{input}"));
    }
    if !flags.case_insensitive {
        return if flags.global {
            Some(input.replace(pattern, replacement))
        } else {
            Some(input.replacen(pattern, replacement, 1))
        };
    }

    let mut output = String::new();
    let mut cursor = 0usize;
    let mut replaced = false;
    while cursor <= input.len() {
        let Some(relative) = static_less_ascii_case_insensitive_find(&input[cursor..], pattern)
        else {
            break;
        };
        let start = cursor + relative;
        let end = start + pattern.len();
        output.push_str(&input[cursor..start]);
        output.push_str(replacement);
        cursor = end;
        replaced = true;
        if !flags.global {
            break;
        }
    }
    if !replaced {
        return Some(input.to_string());
    }
    output.push_str(&input[cursor..]);
    Some(output)
}

fn static_less_ascii_case_insensitive_find(input: &str, pattern: &str) -> Option<usize> {
    input
        .as_bytes()
        .windows(pattern.len())
        .position(|window| window.eq_ignore_ascii_case(pattern.as_bytes()))
}

fn static_less_replace_pattern_is_literal(pattern: &str) -> bool {
    pattern.chars().all(|ch| {
        !matches!(
            ch,
            '\\' | '^' | '$' | '.' | '|' | '?' | '*' | '+' | '(' | ')' | '[' | ']' | '{' | '}'
        ) && !matches!(ch, '\n' | '\r' | '\u{000c}')
    })
}

fn static_less_string_argument(value: &str) -> Option<StaticLessStringArgument> {
    if let Some(rest) = value.trim().strip_prefix('~') {
        let (quote, text) = static_less_quoted_string(rest)?;
        return Some(StaticLessStringArgument {
            text,
            quote: Some(quote),
            escaped: true,
        });
    }
    if let Some((quote, text)) = static_less_quoted_string(value) {
        return Some(StaticLessStringArgument {
            text,
            quote: Some(quote),
            escaped: false,
        });
    }
    static_less_unquoted_string_argument_is_safe(value).then(|| StaticLessStringArgument {
        text: value.to_string(),
        quote: None,
        escaped: false,
    })
}

fn static_less_unquoted_string_argument_is_safe(value: &str) -> bool {
    !value.is_empty()
        && value
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-'))
}

pub(super) fn parse_static_less_escape_value(value: &str) -> Option<String> {
    let argument = parse_whole_function_value_inner(value, "e")?.trim();
    static_less_quoted_string_contents(argument).or_else(|| {
        (!argument.is_empty()
            && argument
                .chars()
                .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-')))
        .then(|| argument.to_string())
    })
}

pub(super) fn parse_static_less_url_escape_value(value: &str) -> Option<String> {
    let argument = parse_whole_function_value_inner(value, "escape")?.trim();
    let text = static_less_quoted_string_contents(argument).unwrap_or_else(|| argument.to_string());
    Some(percent_encode_static_less_escape_value(text.as_str()))
}

fn percent_encode_static_less_escape_value(value: &str) -> String {
    const HEX: &[u8; 16] = b"0123456789ABCDEF";
    let mut output = String::new();
    for byte in value.bytes() {
        if static_less_escape_byte_is_safe(byte) {
            output.push(char::from(byte));
        } else {
            output.push('%');
            output.push(char::from(HEX[usize::from(byte >> 4)]));
            output.push(char::from(HEX[usize::from(byte & 0x0f)]));
        }
    }
    output
}

fn static_less_escape_byte_is_safe(byte: u8) -> bool {
    matches!(
        byte,
        b'A'..=b'Z'
            | b'a'..=b'z'
            | b'0'..=b'9'
            | b'-'
            | b'_'
            | b'.'
            | b'!'
            | b'~'
            | b'*'
            | b'\''
            | b'/'
            | b'?'
            | b'&'
            | b'@'
            | b'+'
            | b','
            | b'$'
    )
}

pub(super) fn reduce_static_less_escaped_string_value(value: &str) -> Option<String> {
    let trimmed = value.trim();
    let rest = trimmed.strip_prefix('~')?;
    static_less_quoted_string_contents(rest)
}

pub(super) fn preserve_static_less_dynamic_escaped_string_value(
    value: &str,
) -> Option<StaticLessResolvedValue> {
    let trimmed = value.trim();
    let rest = trimmed.strip_prefix('~')?;
    let contents = static_less_quoted_string_contents(rest)?;
    contents.contains("@{").then(|| StaticLessResolvedValue {
        text: trimmed.to_string(),
        escaped: true,
    })
}

pub(super) fn static_less_quoted_string_contents(value: &str) -> Option<String> {
    static_less_quoted_string(value).map(|(_, text)| text)
}

fn static_less_quoted_string(value: &str) -> Option<(char, String)> {
    let rest = value.trim();
    let quote = rest.chars().next()?;
    if !matches!(quote, '"' | '\'') {
        return None;
    }

    let mut output = String::new();
    let mut index = quote.len_utf8();
    while index < rest.len() {
        let ch = rest[index..].chars().next()?;
        if matches!(ch, '\n' | '\r' | '\u{000c}') {
            return None;
        }
        if ch == quote {
            return (index + ch.len_utf8() == rest.len()).then_some((quote, output));
        }
        if ch == '\\' {
            index += ch.len_utf8();
            let escaped = rest[index..].chars().next()?;
            if matches!(escaped, '\n' | '\r' | '\u{000c}') {
                return None;
            }
            output.push(escaped);
            index += escaped.len_utf8();
            continue;
        }
        output.push(ch);
        index += ch.len_utf8();
    }
    None
}
