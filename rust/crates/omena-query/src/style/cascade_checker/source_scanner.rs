use std::collections::BTreeSet;

use omena_query_core::split_top_level_value_arguments as split_lattice_top_level_value_arguments;
use omena_query_transform_runner::expand_css_nested_selector;
use omena_syntax::css_keyword;

/// Splits a selector-list prelude on top-level commas, ignoring commas nested
/// inside `()` (e.g. `:is(.a, .b)`), `[]`, or string literals (RFC-0007 B2).
/// Returns one entry per member; a prelude with no top-level comma returns a
/// single-element vector containing the whole (trimmed) prelude.
pub(super) fn split_query_selector_list(prelude: &str) -> Vec<String> {
    let mut members = Vec::new();
    let mut segment_start = 0usize;
    let mut index = 0usize;
    let mut quote: Option<char> = None;
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;

    while index < prelude.len() {
        let Some(ch) = prelude[index..].chars().next() else {
            break;
        };
        if let Some(quote_ch) = quote {
            index += ch.len_utf8();
            if ch == '\\' {
                if let Some(escaped) = prelude[index..].chars().next() {
                    index += escaped.len_utf8();
                }
            } else if ch == quote_ch {
                quote = None;
            }
            continue;
        }
        match ch {
            '"' | '\'' => quote = Some(ch),
            '(' => paren_depth += 1,
            ')' => paren_depth = paren_depth.saturating_sub(1),
            '[' => bracket_depth += 1,
            ']' => bracket_depth = bracket_depth.saturating_sub(1),
            ',' if paren_depth == 0 && bracket_depth == 0 => {
                let member = prelude[segment_start..index].trim();
                if !member.is_empty() {
                    members.push(member.to_string());
                }
                segment_start = index + ch.len_utf8();
            }
            _ => {}
        }
        index += ch.len_utf8();
    }

    let tail = prelude[segment_start..].trim();
    if !tail.is_empty() {
        members.push(tail.to_string());
    }
    if members.is_empty() {
        members.push(prelude.trim().to_string());
    }
    members
}

pub(super) fn canonical_query_checker_selector(
    parent_selector: Option<&str>,
    selector: &str,
) -> String {
    let selector = selector.trim();
    match parent_selector {
        Some(parent_selector) => expand_css_nested_selector(parent_selector, selector)
            .unwrap_or_else(|| fallback_expand_query_nested_selector(parent_selector, selector)),
        None => selector.to_string(),
    }
}

fn fallback_expand_query_nested_selector(parent_selector: &str, selector: &str) -> String {
    if selector.contains('&') {
        selector.replace('&', parent_selector)
    } else {
        format!("{parent_selector} {selector}")
    }
}

pub(super) fn query_value_has_important_suffix(value: &str) -> bool {
    value
        .trim_end()
        .to_ascii_lowercase()
        .ends_with("!important")
}

pub(super) fn trimmed_query_span(source: &str, start: usize, end: usize) -> Option<(usize, usize)> {
    let mut trimmed_start = start;
    let mut trimmed_end = end;
    while trimmed_start < trimmed_end
        && source[trimmed_start..]
            .chars()
            .next()
            .is_some_and(char::is_whitespace)
    {
        trimmed_start += source[trimmed_start..].chars().next()?.len_utf8();
    }
    while trimmed_end > trimmed_start
        && source[..trimmed_end]
            .chars()
            .next_back()
            .is_some_and(char::is_whitespace)
    {
        trimmed_end -= source[..trimmed_end].chars().next_back()?.len_utf8();
    }
    (trimmed_start < trimmed_end).then_some((trimmed_start, trimmed_end))
}

/// Removes CSS/Sass comments from a single declaration statement, quote-aware.
///
/// `/* ... */` block comments are elided entirely; `//` line comments are
/// truncated to the end of their line (Sass semantics). Comment delimiters
/// inside string literals are preserved. The result is used for the
/// property/value split so a comment positioned before a property name no
/// longer poisons it (RFC-0007 B1).
pub(super) fn strip_query_statement_comments(statement: &str) -> String {
    let mut out = String::with_capacity(statement.len());
    let mut index = 0usize;
    let mut quote: Option<char> = None;
    let mut paren_depth = 0usize;
    while index < statement.len() {
        let Some(ch) = statement[index..].chars().next() else {
            break;
        };
        if let Some(quote_ch) = quote {
            out.push(ch);
            index += ch.len_utf8();
            if ch == '\\' {
                if let Some(escaped) = statement[index..].chars().next() {
                    out.push(escaped);
                    index += escaped.len_utf8();
                }
            } else if ch == quote_ch {
                quote = None;
            }
            continue;
        }
        if statement[index..].starts_with("/*") {
            match statement[index + 2..].find("*/") {
                Some(close_offset) => {
                    // Replace the comment with a single space so adjacent tokens
                    // (e.g. `color/* x */:`) do not get glued together.
                    out.push(' ');
                    index += close_offset + 4;
                }
                // Unterminated block comment: drop the remainder.
                None => break,
            }
            continue;
        }
        // A `//` outside parentheses is a Sass line comment; inside parentheses
        // (e.g. `url(http://example.com)`) it is part of a value and must be
        // preserved, otherwise the value is corrupted into an unbalanced token.
        if paren_depth == 0 && statement[index..].starts_with("//") {
            // Sass line comment: skip to the next newline (or end of statement).
            match statement[index..].find('\n') {
                Some(newline_offset) => {
                    out.push('\n');
                    index += newline_offset + 1;
                }
                None => break,
            }
            continue;
        }
        match ch {
            '"' | '\'' => quote = Some(ch),
            '(' => paren_depth += 1,
            ')' => paren_depth = paren_depth.saturating_sub(1),
            _ => {}
        }
        out.push(ch);
        index += ch.len_utf8();
    }
    out
}

pub(super) fn find_query_top_level_colon(statement: &str) -> Option<usize> {
    let mut index = 0usize;
    let mut quote: Option<char> = None;
    let mut paren_depth = 0usize;

    while index < statement.len() {
        let ch = statement[index..].chars().next()?;
        if let Some(quote_ch) = quote {
            index += ch.len_utf8();
            if ch == '\\' {
                if let Some(escaped) = statement[index..].chars().next() {
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
            '(' => {
                paren_depth += 1;
                index += ch.len_utf8();
            }
            ')' => {
                paren_depth = paren_depth.saturating_sub(1);
                index += ch.len_utf8();
            }
            ':' if paren_depth == 0 => return Some(index),
            _ => index += ch.len_utf8(),
        }
    }
    None
}

pub(super) fn query_prelude_start(source: &str, search_start: usize, open_index: usize) -> usize {
    source[search_start..open_index]
        .rfind(['{', '}', ';'])
        .map(|offset| search_start + offset + 1)
        .unwrap_or(search_start)
}

pub(super) fn query_layer_name_from_prelude(prelude: &str) -> Option<String> {
    let rest = css_keyword(prelude.trim_start())
        .strip_prefix("@layer")?
        .trim();
    let name = rest
        .split(|ch: char| ch.is_ascii_whitespace() || matches!(ch, ',' | '{' | ';'))
        .next()
        .unwrap_or_default()
        .trim_matches(['"', '\'']);
    if name.is_empty() {
        Some("(anonymous-layer)".to_string())
    } else {
        Some(name.to_string())
    }
}

pub(super) fn normalize_query_condition_prelude(prelude: &str) -> String {
    prelude.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// RFC-0007-E4 (#45): recognize the `@at-root <selector>` form and return the trailing selector
/// list. Returns `None` for the bare block form (`@at-root { … }`, no selector) — which already
/// works via the generic selector recursion — and for the `@at-root (with: …) <selector>` /
/// `@at-root (without: …) <selector>` query forms, whose leading `(...)` clause we do not yet
/// model; those keep falling through to the generic at-rule handling rather than risk mis-rooting.
/// Any other at-rule returns `None`.
pub(super) fn query_at_root_selector_from_prelude(prelude: &str) -> Option<String> {
    let rest = css_keyword(prelude.trim_start()).strip_prefix("@at-root")?;
    // Require a boundary after the keyword so `@at-rootish` never matches.
    if let Some(next) = rest.chars().next()
        && !next.is_ascii_whitespace()
    {
        return None;
    }
    let selector = rest.trim();
    // Bare block form (no selector) or the `(with:/without:)` query form: defer to generic handling.
    if selector.is_empty() || selector.starts_with('(') {
        return None;
    }
    Some(selector.to_string())
}

pub(super) fn collect_query_var_references_in_value(value: &str) -> Vec<String> {
    let mut refs = BTreeSet::new();
    let mut index = 0usize;
    let mut quote: Option<char> = None;
    while index < value.len() {
        let Some(ch) = value[index..].chars().next() else {
            break;
        };
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
            _ if query_function_name_starts_at(value, index, "var") => {
                let open_index = index + "var".len();
                let Some(close_index) = matching_query_paren_end(value, open_index, value.len())
                else {
                    index += ch.len_utf8();
                    continue;
                };
                collect_query_var_references_from_arguments(
                    &value[open_index + 1..close_index],
                    &mut refs,
                );
                index = close_index + 1;
            }
            _ => {
                index += ch.len_utf8();
            }
        }
    }
    refs.into_iter().collect()
}

fn collect_query_var_references_from_arguments(arguments: &str, refs: &mut BTreeSet<String>) {
    let parts = split_query_top_level_arguments(arguments);
    let Some(first_argument) = parts.first().map(|part| part.trim()) else {
        return;
    };
    if first_argument.starts_with("--") {
        refs.insert(first_argument.to_string());
    }
    for fallback in parts.iter().skip(1) {
        for reference in collect_query_var_references_in_value(fallback) {
            refs.insert(reference);
        }
    }
}

fn split_query_top_level_arguments(arguments: &str) -> Vec<&str> {
    split_lattice_top_level_value_arguments(arguments, 0)
        .map(|segments| segments.into_iter().map(|segment| segment.text).collect())
        .unwrap_or_else(|| vec![arguments])
}

fn query_function_name_starts_at(value: &str, index: usize, function_name: &str) -> bool {
    value
        .get(index..index + function_name.len())
        .is_some_and(|name| name.eq_ignore_ascii_case(function_name))
        && value[index + function_name.len()..].starts_with('(')
}

pub(super) fn find_query_top_level_byte(
    source: &str,
    start: usize,
    end: usize,
    needle: u8,
) -> Option<usize> {
    let mut index = start;
    let mut quote: Option<char> = None;
    let mut paren_depth = 0usize;
    while index < end {
        let ch = source[index..].chars().next()?;
        if let Some(quote_ch) = quote {
            index += ch.len_utf8();
            if ch == '\\' {
                if let Some(escaped) = source[index..].chars().next() {
                    index += escaped.len_utf8();
                }
            } else if ch == quote_ch {
                quote = None;
            }
            continue;
        }
        if source[index..].starts_with("/*")
            && let Some(close_offset) = source[index + 2..end].find("*/")
        {
            index += close_offset + 4;
            continue;
        }
        // Sass `//` line comments outside parentheses are not declaration
        // boundaries, so a `;` (or `{`) buried in one must not be treated as a
        // statement delimiter (RFC-0007 B1). Inside parens (`url(http://…)`) the
        // `//` is part of a value, so it is left intact.
        if paren_depth == 0 && source[index..end].starts_with("//") {
            match source[index..end].find('\n') {
                Some(newline_offset) => {
                    index += newline_offset + 1;
                    continue;
                }
                None => return None,
            }
        }
        // Match the requested delimiter exactly as before (paren-unaware) so the
        // existing statement-boundary behavior is unchanged; `paren_depth` is
        // tracked only to gate the `//` line-comment skip above.
        if ch.len_utf8() == 1 && source.as_bytes()[index] == needle {
            return Some(index);
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
                paren_depth = paren_depth.saturating_sub(1);
                index += ch.len_utf8();
            }
            _ => index += ch.len_utf8(),
        }
    }
    None
}

pub(super) fn matching_query_block_end(
    source: &str,
    open_index: usize,
    end: usize,
) -> Option<usize> {
    matching_query_delimiter_end(source, open_index, end, b'{', b'}')
}

fn matching_query_paren_end(source: &str, open_index: usize, end: usize) -> Option<usize> {
    matching_query_delimiter_end(source, open_index, end, b'(', b')')
}

fn matching_query_delimiter_end(
    source: &str,
    open_index: usize,
    end: usize,
    open: u8,
    close: u8,
) -> Option<usize> {
    if source.as_bytes().get(open_index).copied()? != open {
        return None;
    }
    let mut index = open_index + 1;
    let mut depth = 1usize;
    let mut quote: Option<char> = None;
    // Only gate `//` line-comment skipping for brace matching, where a `}` in a
    // comment would otherwise close the block early. A `//` inside a value's
    // parentheses (`url(http://…)`) is part of the value, so it must be left
    // intact — track an inner paren depth to distinguish the two.
    let track_line_comments = open == b'{';
    let mut inner_paren_depth = 0usize;

    while index < end {
        let ch = source[index..].chars().next()?;
        if let Some(quote_ch) = quote {
            index += ch.len_utf8();
            if ch == '\\' {
                if let Some(escaped) = source[index..].chars().next() {
                    index += escaped.len_utf8();
                }
            } else if ch == quote_ch {
                quote = None;
            }
            continue;
        }
        if source[index..].starts_with("/*")
            && let Some(close_offset) = source[index + 2..end].find("*/")
        {
            index += close_offset + 4;
            continue;
        }
        // Sass `//` line comment: skip to the next newline so a `}` (RFC-0007 B1)
        // buried in a comment does not close the block prematurely. Restricted to
        // brace matching, and only outside value parentheses.
        if track_line_comments && inner_paren_depth == 0 && source[index..end].starts_with("//") {
            match source[index..end].find('\n') {
                Some(newline_offset) => {
                    index += newline_offset + 1;
                    continue;
                }
                None => return None,
            }
        }
        if track_line_comments {
            match ch {
                '(' => inner_paren_depth += 1,
                ')' => inner_paren_depth = inner_paren_depth.saturating_sub(1),
                _ => {}
            }
        }
        match ch {
            '"' | '\'' => {
                quote = Some(ch);
                index += ch.len_utf8();
            }
            _ if ch.len_utf8() == 1 && source.as_bytes()[index] == open => {
                depth += 1;
                index += 1;
            }
            _ if ch.len_utf8() == 1 && source.as_bytes()[index] == close => {
                depth -= 1;
                if depth == 0 {
                    return Some(index);
                }
                index += 1;
            }
            _ => index += ch.len_utf8(),
        }
    }
    None
}
