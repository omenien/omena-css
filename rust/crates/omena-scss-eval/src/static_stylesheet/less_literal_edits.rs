use std::collections::BTreeMap;

use omena_parser::LexedToken;
use omena_syntax::SyntaxKind;

use super::{
    StaticStylesheetEvaluationEdit, StaticStylesheetVariableDeclaration,
    less_values::reduce_static_less_value, static_stylesheet_literal_value_is_safe,
    static_stylesheet_position_is_inside_ranges, static_stylesheet_token_end,
    static_stylesheet_token_is_trivia, static_stylesheet_token_start,
};

pub(super) fn collect_static_less_literal_value_edits(
    style_source: &str,
    tokens: &[LexedToken],
    declarations: &BTreeMap<(usize, String), StaticStylesheetVariableDeclaration>,
    mixin_declaration_ranges: &[(usize, usize)],
) -> Option<Vec<StaticStylesheetEvaluationEdit>> {
    let declaration_removal_ranges = declarations
        .values()
        .flat_map(|declaration| declaration.removal_spans.iter().copied())
        .collect::<Vec<_>>();
    let mut edits = Vec::new();
    for (index, token) in tokens.iter().enumerate() {
        if token.kind != SyntaxKind::LessEscapedString {
            continue;
        }
        let start = static_stylesheet_token_start(token);
        if static_stylesheet_position_is_inside_ranges(start, &declaration_removal_ranges)
            || static_stylesheet_position_is_inside_ranges(start, mixin_declaration_ranges)
            || !static_less_escaped_string_token_is_declaration_value(tokens, index)
        {
            continue;
        }
        let end = static_stylesheet_token_end(token);
        let value = style_source.get(start..end)?;
        if !static_stylesheet_literal_value_is_safe(value) {
            continue;
        }
        let replacement = reduce_static_less_value(value.to_string());
        if replacement != value {
            edits.push(StaticStylesheetEvaluationEdit {
                start,
                end,
                replacement,
            });
        }
    }
    Some(edits)
}

fn static_less_escaped_string_token_is_declaration_value(
    tokens: &[LexedToken],
    token_index: usize,
) -> bool {
    let mut index = token_index;
    while index > 0 {
        index -= 1;
        let kind = tokens[index].kind;
        if static_stylesheet_token_is_trivia(kind) {
            continue;
        }
        match kind {
            SyntaxKind::Colon => return true,
            SyntaxKind::LeftBrace | SyntaxKind::RightBrace | SyntaxKind::Semicolon => return false,
            _ => {}
        }
    }
    false
}
