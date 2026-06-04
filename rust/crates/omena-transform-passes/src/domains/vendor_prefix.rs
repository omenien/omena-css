use omena_parser::{LexedToken, StyleDialect, lex};
use omena_syntax::SyntaxKind;

use crate::helpers::{
    blocks::at_rule_block_start,
    declarations::collect_simple_declarations_in_block,
    tokens::{matching_right_brace_index, skip_whitespace_tokens, token_end, token_start},
};

pub(crate) fn add_css_vendor_prefixes_with_lexer(
    source: &str,
    dialect: StyleDialect,
) -> (String, usize) {
    let lexed = lex(source, dialect);
    let tokens = lexed.tokens();
    let mut insertions = collect_vendor_prefix_insertions(source, tokens);
    if insertions.is_empty() {
        return (source.to_string(), 0);
    }
    insertions.sort_by_key(|(position, _)| *position);

    let mut output = String::with_capacity(source.len());
    let mut cursor = 0;
    for (position, insertion) in &insertions {
        if *position > cursor {
            output.push_str(&source[cursor..*position]);
        }
        output.push_str(insertion);
        cursor = *position;
    }
    if cursor < source.len() {
        output.push_str(&source[cursor..]);
    }

    (output, insertions.len())
}

fn collect_vendor_prefix_insertions(source: &str, tokens: &[LexedToken]) -> Vec<(usize, String)> {
    let mut insertions = Vec::new();
    insertions.extend(collect_keyframes_vendor_prefix_insertions(source, tokens));
    let mut index = 0;

    while index < tokens.len() {
        if tokens[index].kind == SyntaxKind::LeftBrace
            && let Some(close_index) = matching_right_brace_index(tokens, index)
        {
            let declarations = collect_simple_declarations_in_block(tokens, index, close_index);
            for declaration in &declarations {
                for prefixed_property in prefixed_properties_for(&declaration.property)
                    .iter()
                    .copied()
                {
                    if declarations
                        .iter()
                        .any(|candidate| candidate.property == prefixed_property)
                    {
                        continue;
                    }
                    insertions.push((
                        declaration.start,
                        format!("{prefixed_property}: {}; ", declaration.value),
                    ));
                }
                for prefixed_value in prefixed_values_for(&declaration.property, &declaration.value)
                {
                    if declarations.iter().any(|candidate| {
                        candidate.property == declaration.property
                            && candidate.value.eq_ignore_ascii_case(prefixed_value)
                    }) {
                        continue;
                    }
                    insertions.push((
                        declaration.start,
                        format!("{}: {prefixed_value}; ", declaration.property),
                    ));
                }
            }
            index = close_index + 1;
            continue;
        }
        index += 1;
    }

    insertions
}

fn collect_keyframes_vendor_prefix_insertions(
    source: &str,
    tokens: &[LexedToken],
) -> Vec<(usize, String)> {
    let prefixed_names = collect_keyframes_names(tokens, "@-webkit-keyframes");
    let mut insertions = Vec::new();
    let mut index = 0;

    while index < tokens.len() {
        if tokens[index].kind == SyntaxKind::AtKeyword
            && tokens[index].text.eq_ignore_ascii_case("@keyframes")
            && let Some(name) = keyframes_name_after(tokens, index)
            && !prefixed_names
                .iter()
                .any(|prefixed_name| prefixed_name == &name.to_ascii_lowercase())
            && let Some(block_start) = at_rule_block_start(tokens, index + 1)
            && let Some(block_end) = matching_right_brace_index(tokens, block_start)
        {
            let start = token_start(&tokens[index]);
            let end = token_end(&tokens[block_end]);
            let original = &source[start..end];
            let prefixed = original.replacen(&tokens[index].text, "@-webkit-keyframes", 1);
            insertions.push((start, format!("{prefixed} ")));
            index = block_end + 1;
            continue;
        }
        index += 1;
    }

    insertions
}

fn collect_keyframes_names(tokens: &[LexedToken], at_keyword: &str) -> Vec<String> {
    let mut names = Vec::new();
    let mut index = 0;
    while index < tokens.len() {
        if tokens[index].kind == SyntaxKind::AtKeyword
            && tokens[index].text.eq_ignore_ascii_case(at_keyword)
            && let Some(name) = keyframes_name_after(tokens, index)
        {
            names.push(name.to_ascii_lowercase());
        }
        index += 1;
    }
    names
}

fn keyframes_name_after(tokens: &[LexedToken], at_keyword_index: usize) -> Option<&str> {
    let name_index = skip_whitespace_tokens(tokens, at_keyword_index + 1, tokens.len());
    let name_token = tokens.get(name_index)?;
    matches!(name_token.kind, SyntaxKind::Ident | SyntaxKind::String)
        .then_some(name_token.text.as_str())
}

fn prefixed_properties_for(property: &str) -> &'static [&'static str] {
    match property {
        "appearance" => &["-webkit-appearance", "-moz-appearance"],
        "backdrop-filter" => &["-webkit-backdrop-filter"],
        "backface-visibility" => &["-webkit-backface-visibility"],
        "border-image" => &["-webkit-border-image"],
        "box-decoration-break" => &["-webkit-box-decoration-break"],
        "clip-path" => &["-webkit-clip-path"],
        "column-count" => &["-webkit-column-count", "-moz-column-count"],
        "column-fill" => &["-moz-column-fill"],
        "column-gap" => &["-webkit-column-gap", "-moz-column-gap"],
        "column-rule" => &["-webkit-column-rule", "-moz-column-rule"],
        "column-rule-color" => &["-webkit-column-rule-color", "-moz-column-rule-color"],
        "column-rule-style" => &["-webkit-column-rule-style", "-moz-column-rule-style"],
        "column-rule-width" => &["-webkit-column-rule-width", "-moz-column-rule-width"],
        "column-span" => &["-webkit-column-span"],
        "column-width" => &["-webkit-column-width", "-moz-column-width"],
        "columns" => &["-webkit-columns", "-moz-columns"],
        "filter" => &["-webkit-filter"],
        "hyphens" => &["-webkit-hyphens", "-ms-hyphens"],
        "mask-clip" => &["-webkit-mask-clip"],
        "mask-composite" => &["-webkit-mask-composite"],
        "mask-image" => &["-webkit-mask-image"],
        "mask-mode" => &["-webkit-mask-mode"],
        "mask-origin" => &["-webkit-mask-origin"],
        "mask-position" => &["-webkit-mask-position"],
        "mask-repeat" => &["-webkit-mask-repeat"],
        "mask-size" => &["-webkit-mask-size"],
        "perspective" => &["-webkit-perspective"],
        "perspective-origin" => &["-webkit-perspective-origin"],
        "print-color-adjust" => &["-webkit-print-color-adjust"],
        "tab-size" => &["-moz-tab-size"],
        "text-size-adjust" => &["-webkit-text-size-adjust"],
        "touch-action" => &["-ms-touch-action"],
        "transform" => &["-webkit-transform", "-ms-transform"],
        "transform-origin" => &["-webkit-transform-origin", "-ms-transform-origin"],
        "transform-style" => &["-webkit-transform-style"],
        "user-select" => &["-webkit-user-select", "-moz-user-select", "-ms-user-select"],
        _ => &[],
    }
}

fn prefixed_values_for(property: &str, value: &str) -> Vec<&'static str> {
    match (property, value.trim().to_ascii_lowercase().as_str()) {
        ("display", "flex") => vec!["-webkit-box", "-ms-flexbox"],
        ("display", "grid") => vec!["-ms-grid"],
        ("display", "inline-flex") => vec!["-webkit-inline-box", "-ms-inline-flexbox"],
        ("display", "inline-grid") => vec!["-ms-inline-grid"],
        ("position", "sticky") => vec!["-webkit-sticky"],
        _ => Vec::new(),
    }
}
