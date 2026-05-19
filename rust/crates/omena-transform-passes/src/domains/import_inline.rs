use std::collections::BTreeSet;

use omena_parser::{StyleDialect, lex};
use omena_syntax::SyntaxKind;

use crate::{
    TransformImportInlineV0,
    helpers::{
        ascii::strip_ascii_prefix_ignore_case,
        source_rewrite::replace_source_ranges,
        tokens::{token_end, token_start},
    },
};

pub(crate) fn inline_css_imports_with_lexer(
    source: &str,
    dialect: StyleDialect,
    inlines: &[TransformImportInlineV0],
) -> (String, usize) {
    let lexed = lex(source, dialect);
    let tokens = lexed.tokens();
    let mut replacements = Vec::new();
    let mut emitted_less_import_sources = BTreeSet::<String>::new();
    let mut depth = 0usize;
    let mut index = 0;

    while index < tokens.len() {
        match tokens[index].kind {
            SyntaxKind::LeftBrace => depth += 1,
            SyntaxKind::RightBrace => depth = depth.saturating_sub(1),
            SyntaxKind::AtKeyword
                if depth == 0 && tokens[index].text.eq_ignore_ascii_case("@import") =>
            {
                let Some(end_index) = find_import_rule_semicolon(tokens, index) else {
                    index += 1;
                    continue;
                };
                let start = token_start(&tokens[index]);
                let end = token_end(&tokens[end_index]);
                let rule_text = &source[start..end];
                let Some(import_rule) = parse_css_import_rule(rule_text) else {
                    index = end_index + 1;
                    continue;
                };
                if let Some(replacement_css) =
                    inline_replacement_for_import_source(&import_rule.source, inlines)
                {
                    let mut replacement_css = replacement_css.to_string();
                    if dialect == StyleDialect::Less && import_rule.reference_only {
                        replacement_css =
                            filter_less_reference_import_replacement(&replacement_css);
                    }
                    if dialect == StyleDialect::Less
                        && !import_rule.allow_duplicate
                        && !emitted_less_import_sources.insert(import_rule.source.clone())
                    {
                        replacement_css.clear();
                    }
                    replacements.push((
                        start,
                        end,
                        wrap_import_replacement(&import_rule, &replacement_css),
                    ));
                } else if dialect == StyleDialect::Less && import_rule.optional {
                    replacements.push((start, end, String::new()));
                }
                index = end_index + 1;
                continue;
            }
            _ => {}
        }
        index += 1;
    }

    replace_source_ranges(source, &replacements)
}

fn find_import_rule_semicolon(
    tokens: &[omena_parser::LexedToken],
    at_import_index: usize,
) -> Option<usize> {
    let mut index = at_import_index + 1;
    while index < tokens.len() {
        match tokens[index].kind {
            SyntaxKind::Semicolon => return Some(index),
            SyntaxKind::LeftBrace | SyntaxKind::RightBrace => return None,
            _ => index += 1,
        }
    }
    None
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CssImportRule {
    source: String,
    layer_name: Option<String>,
    supports_condition: Option<String>,
    media_query: Option<String>,
    allow_duplicate: bool,
    reference_only: bool,
    optional: bool,
}

fn parse_css_import_rule(rule_text: &str) -> Option<CssImportRule> {
    let rest = strip_ascii_prefix_ignore_case(rule_text.trim(), "@import")?;
    let rest = rest.trim().trim_end_matches(';').trim();
    if rest.is_empty() {
        return None;
    }
    let (rest, allow_duplicate, reference_only, optional) = strip_leading_less_import_options(rest);
    let (source, rest) = parse_css_import_source_prefix(rest)?;
    let mut rest = rest.trim();
    let mut layer_name = None;
    let mut supports_condition = None;

    loop {
        if let Some((layer, next_rest)) = parse_layer_import_option(rest) {
            layer_name = Some(layer);
            rest = next_rest.trim();
            continue;
        }
        if let Some((supports, next_rest)) = parse_function_prefix(rest, "supports") {
            supports_condition = Some(format!("({})", supports.trim()));
            rest = next_rest.trim();
            continue;
        }
        break;
    }

    Some(CssImportRule {
        source,
        layer_name,
        supports_condition,
        media_query: (!rest.is_empty()).then(|| rest.to_string()),
        allow_duplicate,
        reference_only,
        optional,
    })
}

fn parse_css_import_source_prefix(text: &str) -> Option<(String, &str)> {
    parse_quoted_css_string_prefix(text).or_else(|| parse_url_import_source_prefix(text))
}

fn strip_leading_less_import_options(mut text: &str) -> (&str, bool, bool, bool) {
    let mut allow_duplicate = false;
    let mut reference_only = false;
    let mut optional = false;
    loop {
        let rest = text.trim_start();
        let Some(after_left_paren) = rest.strip_prefix('(') else {
            return (rest, allow_duplicate, reference_only, optional);
        };
        let Some(close_index) = matching_function_close_index(after_left_paren) else {
            return (rest, allow_duplicate, reference_only, optional);
        };
        let option = after_left_paren[..close_index].trim();
        if option.is_empty() || !less_import_option_list_is_safe(option) {
            return (rest, allow_duplicate, reference_only, optional);
        }
        allow_duplicate |= less_import_option_list_allows_duplicate(option);
        reference_only |= less_import_option_list_is_reference_only(option);
        optional |= less_import_option_list_is_optional(option);
        text = &after_left_paren[close_index + 1..];
    }
}

fn less_import_option_list_is_safe(option: &str) -> bool {
    option.chars().all(|ch| {
        ch.is_ascii_alphanumeric() || ch.is_ascii_whitespace() || matches!(ch, '-' | '_' | ',')
    })
}

fn less_import_option_list_allows_duplicate(option: &str) -> bool {
    option
        .split(',')
        .map(str::trim)
        .any(|entry| entry.eq_ignore_ascii_case("multiple"))
}

fn less_import_option_list_is_reference_only(option: &str) -> bool {
    option
        .split(',')
        .map(str::trim)
        .any(|entry| entry.eq_ignore_ascii_case("reference"))
}

fn less_import_option_list_is_optional(option: &str) -> bool {
    option
        .split(',')
        .map(str::trim)
        .any(|entry| entry.eq_ignore_ascii_case("optional"))
}

fn filter_less_reference_import_replacement(source: &str) -> String {
    let mut output = String::new();
    let mut statement_start = 0usize;
    let mut depth = 0usize;
    let mut quote = None::<char>;
    let mut escaped = false;

    for (index, ch) in source.char_indices() {
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
            '"' | '\'' => quote = Some(ch),
            '{' | '(' | '[' => depth += 1,
            '}' | ')' | ']' => depth = depth.saturating_sub(1),
            ';' if depth == 0 => {
                let statement = source[statement_start..=index].trim();
                if less_reference_import_statement_is_static_binding(statement) {
                    if !output.is_empty() {
                        output.push(' ');
                    }
                    output.push_str(statement);
                }
                statement_start = index + ch.len_utf8();
            }
            _ => {}
        }
    }

    output
}

fn less_reference_import_statement_is_static_binding(statement: &str) -> bool {
    let statement = statement.trim();
    if statement.is_empty() || statement.contains('{') || statement.contains('}') {
        return false;
    }
    let Some(first_char) = statement.chars().next() else {
        return false;
    };
    matches!(first_char, '@' | '$') && top_level_colon_index(statement).is_some()
}

fn top_level_colon_index(content: &str) -> Option<usize> {
    let mut delimiter_stack = Vec::<char>::new();
    let mut quote = None::<char>;
    let mut escaped = false;

    for (index, ch) in content.char_indices() {
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
            '"' | '\'' => quote = Some(ch),
            '(' | '[' => delimiter_stack.push(ch),
            ')' if delimiter_stack.last() == Some(&'(') => {
                delimiter_stack.pop();
            }
            ']' if delimiter_stack.last() == Some(&'[') => {
                delimiter_stack.pop();
            }
            ':' if delimiter_stack.is_empty() => return Some(index),
            _ => {}
        }
    }
    None
}

fn parse_quoted_css_string_prefix(text: &str) -> Option<(String, &str)> {
    let mut chars = text.char_indices();
    let (_, quote) = chars.next()?;
    if !matches!(quote, '"' | '\'') {
        return None;
    }
    let mut escaped = false;
    let mut output = String::new();
    for (index, ch) in chars {
        if escaped {
            output.push(ch);
            escaped = false;
            continue;
        }
        if ch == '\\' {
            escaped = true;
            continue;
        }
        if ch == quote {
            let end = index + ch.len_utf8();
            return Some((output, &text[end..]));
        }
        output.push(ch);
    }
    None
}

fn parse_url_import_source_prefix(text: &str) -> Option<(String, &str)> {
    let rest = strip_ascii_prefix_ignore_case(text, "url(")?;
    let close_index = matching_function_close_index(rest)?;
    let inner = rest[..close_index].trim();
    if let Some((source, trailing)) = parse_quoted_css_string_prefix(inner)
        && trailing.trim().is_empty()
    {
        return Some((source, &rest[close_index + 1..]));
    }
    if inner.is_empty()
        || inner
            .chars()
            .any(|ch| ch.is_ascii_whitespace() || matches!(ch, '"' | '\'' | '(' | ')'))
    {
        return None;
    }
    Some((inner.to_string(), &rest[close_index + 1..]))
}

fn parse_layer_import_option(text: &str) -> Option<(String, &str)> {
    if let Some((layer, rest)) = parse_function_prefix(text, "layer") {
        return Some((layer.trim().to_string(), rest));
    }
    let rest = strip_ascii_prefix_ignore_case(text, "layer")?;
    if !rest.is_empty() && !rest.starts_with(char::is_whitespace) {
        return None;
    }
    Some((String::new(), rest))
}

fn parse_function_prefix<'a>(text: &'a str, name: &str) -> Option<(String, &'a str)> {
    let rest = strip_ascii_prefix_ignore_case(text.trim_start(), name)?;
    let rest = rest.strip_prefix('(')?;
    let close_index = matching_function_close_index(rest)?;
    Some((rest[..close_index].to_string(), &rest[close_index + 1..]))
}

fn matching_function_close_index(text: &str) -> Option<usize> {
    let mut depth = 1usize;
    let mut quote = None::<char>;
    let mut escaped = false;

    for (index, ch) in text.char_indices() {
        if let Some(active_quote) = quote {
            if escaped {
                escaped = false;
                continue;
            }
            if ch == '\\' {
                escaped = true;
                continue;
            }
            if ch == active_quote {
                quote = None;
            }
            continue;
        }

        match ch {
            '"' | '\'' => quote = Some(ch),
            '(' => depth += 1,
            ')' => {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    return Some(index);
                }
            }
            _ => {}
        }
    }
    None
}

fn wrap_import_replacement(import_rule: &CssImportRule, replacement_css: &str) -> String {
    let mut output = replacement_css.to_string();
    if let Some(layer_name) = &import_rule.layer_name {
        output = if layer_name.is_empty() {
            format!("@layer {{ {output} }}")
        } else {
            format!("@layer {layer_name} {{ {output} }}")
        };
    }
    if let Some(supports_condition) = &import_rule.supports_condition {
        output = format!("@supports {supports_condition} {{ {output} }}");
    }
    if let Some(media_query) = &import_rule.media_query {
        output = format!("@media {media_query} {{ {output} }}");
    }
    output
}

fn inline_replacement_for_import_source<'a>(
    import_source: &str,
    inlines: &'a [TransformImportInlineV0],
) -> Option<&'a str> {
    inlines
        .iter()
        .find(|inline| inline.import_source == import_source)
        .map(|inline| inline.replacement_css.as_str())
}
