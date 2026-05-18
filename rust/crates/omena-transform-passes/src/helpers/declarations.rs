use omena_parser::LexedToken;
use omena_syntax::SyntaxKind;

use super::tokens::{
    is_comment_token, matching_right_brace_index, skip_whitespace_tokens, token_end, token_start,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SimpleDeclarationSlice {
    pub(crate) property: String,
    pub(crate) value: String,
    pub(crate) important: bool,
    pub(crate) start: usize,
    pub(crate) end: usize,
    pub(crate) source_order: u32,
}

pub(crate) fn collect_simple_declarations_in_block(
    tokens: &[LexedToken],
    block_start: usize,
    block_end: usize,
) -> Vec<SimpleDeclarationSlice> {
    let mut declarations = Vec::new();
    let mut index = block_start + 1;
    let mut source_order = 0u32;

    while index < block_end {
        index = skip_whitespace_tokens(tokens, index, block_end);
        if index >= block_end {
            break;
        }

        if tokens[index].kind == SyntaxKind::LeftBrace
            && let Some(close_index) = matching_right_brace_index(tokens, index)
        {
            index = close_index + 1;
            continue;
        }

        if let Some((declaration, next_index)) =
            parse_simple_declaration_slice(tokens, index, block_end, source_order)
        {
            declarations.push(declaration);
            source_order += 1;
            index = next_index;
        } else {
            index += 1;
        }
    }

    declarations
}

fn parse_simple_declaration_slice(
    tokens: &[LexedToken],
    start_index: usize,
    block_end: usize,
    source_order: u32,
) -> Option<(SimpleDeclarationSlice, usize)> {
    let property_token = tokens.get(start_index)?;
    let property = match property_token.kind {
        SyntaxKind::Ident => property_token.text.to_ascii_lowercase(),
        SyntaxKind::CustomPropertyName => property_token.text.clone(),
        _ => return None,
    };

    let colon_index = skip_whitespace_tokens(tokens, start_index + 1, block_end);
    if tokens.get(colon_index)?.kind != SyntaxKind::Colon {
        return None;
    }

    let mut value_tokens: Vec<&LexedToken> = Vec::new();
    let mut index = colon_index + 1;
    while index < block_end {
        match tokens[index].kind {
            SyntaxKind::Semicolon => {
                return build_simple_declaration_slice(
                    property,
                    property_token,
                    &value_tokens,
                    token_end(&tokens[index]),
                    source_order,
                    index + 1,
                );
            }
            SyntaxKind::LeftBrace | SyntaxKind::RightBrace => return None,
            _ => value_tokens.push(&tokens[index]),
        }
        index += 1;
    }

    let last_value_token = value_tokens.last()?;
    build_simple_declaration_slice(
        property,
        property_token,
        &value_tokens,
        token_end(last_value_token),
        source_order,
        index,
    )
}

fn build_simple_declaration_slice(
    property: String,
    property_token: &LexedToken,
    value_tokens: &[&LexedToken],
    end: usize,
    source_order: u32,
    next_index: usize,
) -> Option<(SimpleDeclarationSlice, usize)> {
    if value_tokens
        .iter()
        .any(|token| is_comment_token(token.kind))
    {
        return None;
    }
    let value = value_tokens
        .iter()
        .map(|token| token.text.as_str())
        .collect::<String>()
        .trim()
        .to_string();
    if value.is_empty() {
        return None;
    }
    let important = value_tokens
        .iter()
        .any(|token| token.kind == SyntaxKind::Important);
    Some((
        SimpleDeclarationSlice {
            property,
            value,
            important,
            start: token_start(property_token),
            end,
            source_order,
        },
        next_index,
    ))
}
