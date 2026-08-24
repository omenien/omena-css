//! CST-backed cascade-layer path producer.

use omena_syntax::{LayerPathV0, SyntaxKind, SyntaxNode};

pub fn layer_paths_from_cst(source: &str, layer_rule: &SyntaxNode) -> Vec<LayerPathV0> {
    if layer_rule.kind() != SyntaxKind::LayerRule {
        return Vec::new();
    }

    let mut paths = Vec::new();
    let mut authored_segments = Vec::<String>::new();
    let mut saw_layer_keyword = false;
    let mut path_valid = true;
    for token in layer_rule
        .descendants_with_tokens()
        .filter_map(|element| element.into_token())
    {
        let kind = token.kind();
        if matches!(
            kind,
            SyntaxKind::LeftBrace | SyntaxKind::SassIndent | SyntaxKind::Semicolon
        ) {
            break;
        }
        if !saw_layer_keyword {
            if kind == SyntaxKind::AtKeyword {
                saw_layer_keyword = true;
            }
            continue;
        }
        match kind {
            SyntaxKind::Ident | SyntaxKind::CustomPropertyName => {
                if let Some(text) = source_text_for_token(source, &token) {
                    authored_segments.push(text.to_string());
                } else {
                    path_valid = false;
                }
            }
            SyntaxKind::Dot
            | SyntaxKind::Whitespace
            | SyntaxKind::LineComment
            | SyntaxKind::BlockComment => {}
            SyntaxKind::Comma => {
                if path_valid
                    && let Some(path) = LayerPathV0::from_authored_segments(
                        authored_segments.iter().map(String::as_str),
                    )
                {
                    paths.push(path);
                }
                authored_segments.clear();
                path_valid = true;
            }
            _ => path_valid = false,
        }
    }
    if path_valid
        && let Some(path) =
            LayerPathV0::from_authored_segments(authored_segments.iter().map(String::as_str))
    {
        paths.push(path);
    }
    paths
}

fn source_text_for_token<'a>(
    source: &'a str,
    token: &omena_syntax::SyntaxToken,
) -> Option<&'a str> {
    let range = token.text_range();
    source.get(u32::from(range.start()) as usize..u32::from(range.end()) as usize)
}
