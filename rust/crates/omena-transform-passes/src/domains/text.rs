use omena_parser::{StyleDialect, lex};
use omena_syntax::SyntaxKind;

use crate::helpers::{
    declarations::{
        collect_simple_declarations_in_block, format_replacement_declaration_like_source,
    },
    source_rewrite::{replace_source_ranges, rewrite_lexer_tokens},
    tokens::matching_right_brace_index,
    values::{
        split_top_level_value_arguments, split_top_level_whitespace_value_components,
        static_css_string_value,
    },
};

pub(crate) fn normalize_css_string_quotes_with_lexer(
    source: &str,
    dialect: StyleDialect,
) -> (String, usize) {
    rewrite_lexer_tokens(source, dialect, |kind, text| {
        if kind == SyntaxKind::String {
            return normalize_css_string_token_quotes(text);
        }
        None
    })
}

pub(crate) fn normalize_css_font_declarations_with_lexer(
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
                let Some(replacement_value) = normalize_static_font_declaration_value(
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

    replace_source_ranges(source, &replacements)
}

fn normalize_static_font_declaration_value(property: &str, value: &str) -> Option<String> {
    match property {
        "display" => normalize_static_display_value(value),
        "font-family" => normalize_static_font_family_value(value),
        "font-weight" => normalize_static_font_weight_value(value),
        "font-stretch" => normalize_static_font_stretch_value(value),
        _ => None,
    }
}

fn normalize_static_display_value(value: &str) -> Option<String> {
    let components = split_top_level_whitespace_value_components(value)?;
    let lowered_components = components
        .iter()
        .map(|component| component.to_ascii_lowercase())
        .collect::<Vec<_>>();
    let replacement = match lowered_components
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>()
        .as_slice()
    {
        [outer, inner] => normalize_two_keyword_display_value(outer, inner)?,
        ["list-item", "block", "flow"] => "list-item".to_string(),
        _ => return None,
    };
    (replacement != components.join(" ")).then_some(replacement)
}

fn normalize_two_keyword_display_value(outer: &str, inner: &str) -> Option<String> {
    match (outer, inner) {
        ("block", "flow") => Some("block".to_string()),
        ("inline", "flow") => Some("inline".to_string()),
        ("block", "flow-root") => Some("flow-root".to_string()),
        ("inline", "flow-root") => Some("inline-block".to_string()),
        ("block", "flex") => Some("flex".to_string()),
        ("inline", "flex") => Some("inline-flex".to_string()),
        ("block", "grid") => Some("grid".to_string()),
        ("inline", "grid") => Some("inline-grid".to_string()),
        _ => None,
    }
}

fn normalize_static_font_family_value(value: &str) -> Option<String> {
    let families = split_top_level_value_arguments(value)?;
    let mut normalized = Vec::with_capacity(families.len());
    let mut changed = false;

    for family in families {
        let Some(quoted_family) = static_css_string_value(&family) else {
            normalized.push(family);
            continue;
        };
        let Some(unquoted_family) = unquote_static_font_family_name(&quoted_family) else {
            normalized.push(family);
            continue;
        };
        changed = true;
        normalized.push(unquoted_family);
    }

    changed.then(|| normalized.join(","))
}

fn normalize_static_font_weight_value(value: &str) -> Option<String> {
    match value.trim().to_ascii_lowercase().as_str() {
        "normal" => Some("400".to_string()),
        "bold" => Some("700".to_string()),
        _ => None,
    }
}

fn normalize_static_font_stretch_value(value: &str) -> Option<String> {
    let normalized = match value.trim().to_ascii_lowercase().as_str() {
        "ultra-condensed" => "50%",
        "extra-condensed" => "62.5%",
        "condensed" => "75%",
        "semi-condensed" => "87.5%",
        "normal" => "100%",
        "semi-expanded" => "112.5%",
        "expanded" => "125%",
        "extra-expanded" => "150%",
        "ultra-expanded" => "200%",
        _ => return None,
    };
    Some(normalized.to_string())
}

fn unquote_static_font_family_name(value: &str) -> Option<String> {
    let parts = value.split_ascii_whitespace().collect::<Vec<_>>();
    if parts.is_empty() {
        return None;
    }
    if parts
        .iter()
        .any(|part| !is_safe_unquoted_font_family_identifier(part))
    {
        return None;
    }
    Some(parts.join(" "))
}

fn is_safe_unquoted_font_family_identifier(value: &str) -> bool {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if value.starts_with("--") && value.len() > 2 {
        return chars.all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_'))
            && !is_reserved_unquoted_font_family_identifier(value);
    }
    if first == '-' {
        let Some(second) = chars.next() else {
            return false;
        };
        if !(second.is_ascii_alphabetic() || second == '_') {
            return false;
        }
        return chars.all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_'))
            && !is_reserved_unquoted_font_family_identifier(value);
    }
    if !(first.is_ascii_alphabetic() || first == '_') {
        return false;
    }
    if !chars.all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_')) {
        return false;
    }
    !is_reserved_unquoted_font_family_identifier(value)
}

fn is_reserved_unquoted_font_family_identifier(value: &str) -> bool {
    matches!(
        value.to_ascii_lowercase().as_str(),
        "serif"
            | "sans-serif"
            | "monospace"
            | "cursive"
            | "fantasy"
            | "system-ui"
            | "ui-serif"
            | "ui-sans-serif"
            | "ui-monospace"
            | "ui-rounded"
            | "math"
            | "emoji"
            | "fangsong"
            | "inherit"
            | "initial"
            | "unset"
            | "revert"
            | "revert-layer"
    )
}

fn normalize_css_string_token_quotes(text: &str) -> Option<String> {
    if !text.starts_with('\'') || !text.ends_with('\'') || text.len() < 2 {
        return None;
    }
    let inner = &text[1..text.len() - 1];
    if inner
        .chars()
        .any(|ch| matches!(ch, '"' | '\\' | '\n' | '\r'))
    {
        return None;
    }

    Some(format!("\"{inner}\""))
}

pub(crate) fn strip_css_url_quotes_with_lexer(
    source: &str,
    dialect: StyleDialect,
) -> (String, usize) {
    let lexed = lex(source, dialect);
    let tokens = lexed.tokens();
    let mut output = String::with_capacity(source.len());
    let mut index = 0;
    let mut mutation_count = 0;

    while index < tokens.len() {
        if let Some((replacement, consumed)) = rewrite_safe_quoted_url(tokens, index) {
            output.push_str(&replacement);
            mutation_count += 1;
            index += consumed;
            continue;
        }

        output.push_str(&tokens[index].text);
        index += 1;
    }

    (output, mutation_count)
}

fn rewrite_safe_quoted_url(
    tokens: &[omena_parser::LexedToken],
    index: usize,
) -> Option<(String, usize)> {
    let ident = tokens.get(index)?;
    let left_paren = tokens.get(index + 1)?;
    let string = tokens.get(index + 2)?;
    let right_paren = tokens.get(index + 3)?;

    if ident.kind != SyntaxKind::Ident
        || !ident.text.eq_ignore_ascii_case("url")
        || left_paren.kind != SyntaxKind::LeftParen
        || string.kind != SyntaxKind::String
        || right_paren.kind != SyntaxKind::RightParen
    {
        return None;
    }

    let inner = unquote_safe_url_string(&string.text)?;
    Some((format!("{}({inner})", ident.text), 4))
}

fn unquote_safe_url_string(text: &str) -> Option<&str> {
    let quote = text.as_bytes().first().copied()?;
    if quote != b'\'' && quote != b'"' {
        return None;
    }
    if text.as_bytes().last().copied() != Some(quote) || text.len() < 2 {
        return None;
    }

    let inner = &text[1..text.len() - 1];
    if inner
        .chars()
        .any(|ch| ch.is_whitespace() || matches!(ch, '"' | '\'' | '(' | ')' | '\\'))
    {
        return None;
    }

    Some(inner)
}
