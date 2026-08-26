use omena_parser::LexedToken;
use omena_syntax::{
    SyntaxKind,
    ident::{
        AuthoredPropertyTextV0, CanonicalPropertyKeyV0, PropertyNameKindV0, PropertyNameV0,
        render_authored,
    },
};

use super::tokens::{
    is_comment_token, matching_right_brace_index, skip_whitespace_tokens, token_end, token_start,
    tokens_between_byte_range,
};

#[derive(Debug, Clone)]
pub(crate) struct SimpleDeclarationSlice {
    /// Authored property spelling retained for source-preserving emission.
    pub(crate) property: AuthoredPropertyTextV0,
    pub(crate) property_key: CanonicalPropertyKeyV0,
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
        SyntaxKind::Ident => {
            PropertyNameV0::new(property_token.text.clone(), PropertyNameKindV0::Standard)
        }
        SyntaxKind::CustomPropertyName => {
            PropertyNameV0::new(property_token.text.clone(), PropertyNameKindV0::Custom)
        }
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
    property: PropertyNameV0,
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
            property: property.authored_text(),
            property_key: property.canonical_key(),
            value,
            important,
            start: token_start(property_token),
            end,
            source_order,
        },
        next_index,
    ))
}

pub(crate) fn format_replacement_declaration_like_source(
    source: &str,
    declaration: &SimpleDeclarationSlice,
    replacement_value: &str,
) -> String {
    let original = source
        .get(declaration.start..declaration.end)
        .unwrap_or_default();
    let after_colon = original
        .split_once(':')
        .map(|(_, after_colon)| after_colon)
        .unwrap_or_default();
    let separator = if after_colon.chars().next().is_some_and(char::is_whitespace) {
        ": "
    } else {
        ":"
    };
    let terminator = if original.trim_end().ends_with(';') {
        ";"
    } else {
        ""
    };
    let mut replacement = String::new();
    let _ = render_authored(&declaration.property, &mut replacement);
    replacement.push_str(separator);
    replacement.push_str(replacement_value);
    replacement.push_str(terminator);
    replacement
}

pub(crate) fn declaration_ranges_are_adjacent(
    tokens: &[LexedToken],
    declarations: &[SimpleDeclarationSlice],
) -> bool {
    declarations.windows(2).all(|pair| {
        tokens_between_byte_range(tokens, pair[0].end, pair[1].start)
            .iter()
            .all(|token| token.kind == SyntaxKind::Whitespace)
    })
}
