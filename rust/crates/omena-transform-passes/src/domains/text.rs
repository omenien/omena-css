use omena_parser::{StyleDialect, lex};
use omena_syntax::SyntaxKind;

use crate::helpers::{
    declarations::{
        SimpleDeclarationSlice, collect_simple_declarations_in_block,
        format_replacement_declaration_like_source,
    },
    source_rewrite::{remove_source_ranges, replace_source_ranges, rewrite_lexer_tokens},
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

    let (output, replacement_count) = replace_source_ranges(source, &replacements);
    let (output, removal_count) = remove_overridden_static_font_longhands(&output, dialect);
    (output, replacement_count + removal_count)
}

fn normalize_static_font_declaration_value(property: &str, value: &str) -> Option<String> {
    match property {
        "cursor" => normalize_single_known_keyword_case(value, CURSOR_KEYWORDS),
        "display" => normalize_static_display_value(value),
        "font-family" => normalize_static_font_family_value(value),
        "font-weight" => normalize_static_font_weight_value(value),
        "font-stretch" => normalize_static_font_stretch_value(value),
        "position" => normalize_single_known_keyword_case(value, POSITION_KEYWORDS),
        "text-align" => normalize_single_known_keyword_case(value, TEXT_ALIGN_KEYWORDS),
        "user-select" => normalize_single_known_keyword_case(value, USER_SELECT_KEYWORDS),
        "visibility" => normalize_single_known_keyword_case(value, VISIBILITY_KEYWORDS),
        _ => None,
    }
}

const CURSOR_KEYWORDS: &[&str] = &[
    "alias",
    "all-scroll",
    "auto",
    "cell",
    "col-resize",
    "context-menu",
    "copy",
    "crosshair",
    "default",
    "e-resize",
    "ew-resize",
    "grab",
    "grabbing",
    "help",
    "move",
    "n-resize",
    "ne-resize",
    "nesw-resize",
    "no-drop",
    "none",
    "not-allowed",
    "ns-resize",
    "nw-resize",
    "nwse-resize",
    "pointer",
    "progress",
    "row-resize",
    "s-resize",
    "se-resize",
    "sw-resize",
    "text",
    "vertical-text",
    "w-resize",
    "wait",
    "zoom-in",
    "zoom-out",
];
const POSITION_KEYWORDS: &[&str] = &["absolute", "fixed", "relative", "static", "sticky"];
const TEXT_ALIGN_KEYWORDS: &[&str] = &[
    "center",
    "end",
    "justify",
    "left",
    "match-parent",
    "right",
    "start",
];
const USER_SELECT_KEYWORDS: &[&str] = &["all", "auto", "contain", "none", "text"];
const VISIBILITY_KEYWORDS: &[&str] = &["collapse", "hidden", "visible"];

fn normalize_single_known_keyword_case(value: &str, keywords: &[&str]) -> Option<String> {
    let components = split_top_level_whitespace_value_components(value)?;
    let [component] = components.as_slice() else {
        return None;
    };
    let lowered = component.to_ascii_lowercase();
    keywords
        .contains(&lowered.as_str())
        .then_some(lowered)
        .filter(|replacement| replacement != component)
}

fn remove_overridden_static_font_longhands(source: &str, dialect: StyleDialect) -> (String, usize) {
    let lexed = lex(source, dialect);
    let tokens = lexed.tokens();
    let mut ranges = Vec::new();
    let mut index = 0;

    while index < tokens.len() {
        if tokens[index].kind == SyntaxKind::LeftBrace
            && let Some(close_index) = matching_right_brace_index(tokens, index)
        {
            let declarations = collect_simple_declarations_in_block(tokens, index, close_index);
            for (declaration_index, declaration) in declarations.iter().enumerate() {
                if !is_static_font_override_candidate(declaration) {
                    continue;
                }
                let later_declaration = declarations[declaration_index + 1..]
                    .iter()
                    .find(|candidate| candidate.property == declaration.property);
                if later_declaration.is_some_and(|candidate| {
                    is_static_font_override_candidate(candidate)
                        && !declaration.important
                        && !candidate.important
                }) {
                    ranges.push((declaration.start, declaration.end));
                }
            }
            index = close_index + 1;
            continue;
        }
        index += 1;
    }

    remove_source_ranges(source, &ranges)
}

fn is_static_font_override_candidate(declaration: &SimpleDeclarationSlice) -> bool {
    match declaration.property.as_str() {
        "font-weight" => is_static_font_weight_value(&declaration.value),
        "font-stretch" => is_static_font_stretch_value(&declaration.value),
        _ => false,
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
        [single] => normalize_single_keyword_display_value(single)?,
        [outer, inner] => normalize_two_keyword_display_value(outer, inner)?,
        [first, second, third] => normalize_three_keyword_display_value(first, second, third)?,
        _ => return None,
    };
    (replacement != components.join(" ")).then_some(replacement)
}

fn normalize_single_keyword_display_value(value: &str) -> Option<String> {
    match value {
        "block"
        | "inline"
        | "run-in"
        | "flow-root"
        | "none"
        | "contents"
        | "flex"
        | "grid"
        | "ruby"
        | "list-item"
        | "table"
        | "inline-table"
        | "table-row-group"
        | "table-header-group"
        | "table-footer-group"
        | "table-row"
        | "table-cell"
        | "table-column-group"
        | "table-column"
        | "table-caption"
        | "ruby-base"
        | "ruby-text"
        | "ruby-base-container"
        | "ruby-text-container" => Some(value.to_string()),
        _ => None,
    }
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
        ("inline", "ruby") => Some("ruby".to_string()),
        ("block", "ruby") => Some("block ruby".to_string()),
        ("block", "list-item")
        | ("flow", "list-item")
        | ("list-item", "block")
        | ("list-item", "flow") => Some("list-item".to_string()),
        ("inline", "list-item") | ("list-item", "inline") => Some("inline list-item".to_string()),
        ("flow-root", "list-item") | ("list-item", "flow-root") => {
            Some("flow-root list-item".to_string())
        }
        _ => None,
    }
}

fn normalize_three_keyword_display_value(first: &str, second: &str, third: &str) -> Option<String> {
    match (first, second, third) {
        ("list-item", "block", "flow")
        | ("block", "list-item", "flow")
        | ("block", "flow", "list-item")
        | ("flow", "block", "list-item")
        | ("flow", "list-item", "block")
        | ("list-item", "flow", "block") => Some("list-item".to_string()),
        ("list-item", "inline", "flow")
        | ("inline", "list-item", "flow")
        | ("inline", "flow", "list-item")
        | ("flow", "inline", "list-item")
        | ("flow", "list-item", "inline")
        | ("list-item", "flow", "inline") => Some("inline list-item".to_string()),
        ("list-item", "block", "flow-root")
        | ("block", "list-item", "flow-root")
        | ("block", "flow-root", "list-item")
        | ("flow-root", "block", "list-item")
        | ("flow-root", "list-item", "block")
        | ("list-item", "flow-root", "block") => Some("flow-root list-item".to_string()),
        ("list-item", "inline", "flow-root")
        | ("inline", "list-item", "flow-root")
        | ("inline", "flow-root", "list-item")
        | ("flow-root", "inline", "list-item")
        | ("flow-root", "list-item", "inline")
        | ("list-item", "flow-root", "inline") => Some("inline flow-root list-item".to_string()),
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

fn is_static_font_weight_value(value: &str) -> bool {
    match value.trim().to_ascii_lowercase().as_str() {
        "normal" | "bold" => true,
        numeric => numeric
            .parse::<f64>()
            .is_ok_and(|value| value.is_finite() && (1.0..=1000.0).contains(&value)),
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

fn is_static_font_stretch_value(value: &str) -> bool {
    if normalize_static_font_stretch_value(value).is_some() {
        return true;
    }
    value
        .trim()
        .strip_suffix('%')
        .and_then(|number| number.parse::<f64>().ok())
        .is_some_and(|value| value.is_finite() && value >= 0.0)
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
