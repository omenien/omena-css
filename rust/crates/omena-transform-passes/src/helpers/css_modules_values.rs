use omena_parser::LexedToken;
use omena_syntax::SyntaxKind;

use crate::helpers::tokens::{is_comment_token, skip_whitespace_tokens, token_end, token_start};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct StaticCssModulesValueDefinition {
    pub(crate) name: String,
    pub(crate) value: String,
    pub(crate) start: usize,
    pub(crate) end: usize,
}

pub(crate) fn collect_static_local_css_modules_value_definitions(
    tokens: &[LexedToken],
) -> Vec<StaticCssModulesValueDefinition> {
    let mut definitions = Vec::new();
    let mut depth = 0usize;
    let mut index = 0;

    while index < tokens.len() {
        match tokens[index].kind {
            SyntaxKind::AtKeyword
                if depth == 0 && tokens[index].text.eq_ignore_ascii_case("@value") =>
            {
                let Some((definition, next_index)) =
                    parse_static_local_css_modules_value_definition(tokens, index)
                else {
                    index += 1;
                    continue;
                };
                definitions.push(definition);
                index = next_index;
                continue;
            }
            SyntaxKind::LeftBrace => depth += 1,
            SyntaxKind::RightBrace => depth = depth.saturating_sub(1),
            _ => {}
        }
        index += 1;
    }

    definitions
}

fn parse_static_local_css_modules_value_definition(
    tokens: &[LexedToken],
    at_value_index: usize,
) -> Option<(StaticCssModulesValueDefinition, usize)> {
    let mut index = skip_whitespace_tokens(tokens, at_value_index + 1, tokens.len());
    let name_token = tokens.get(index)?;
    if name_token.kind != SyntaxKind::Ident {
        return None;
    }
    let name = name_token.text.clone();

    index = skip_whitespace_tokens(tokens, index + 1, tokens.len());
    if tokens.get(index)?.kind != SyntaxKind::Colon {
        return None;
    }

    let value_start = index + 1;
    index += 1;
    while index < tokens.len() {
        match tokens[index].kind {
            SyntaxKind::Semicolon => {
                let value_tokens = tokens[value_start..index].iter().collect::<Vec<_>>();
                if value_tokens.is_empty()
                    || value_tokens.iter().any(|token| {
                        is_comment_token(token.kind) || token.kind == SyntaxKind::AtKeyword
                    })
                {
                    return None;
                }
                let value = value_tokens
                    .iter()
                    .map(|token| token.text.as_str())
                    .collect::<String>()
                    .trim()
                    .to_string();
                return Some((
                    StaticCssModulesValueDefinition {
                        name,
                        value,
                        start: token_start(&tokens[at_value_index]),
                        end: token_end(&tokens[index]),
                    },
                    index + 1,
                ));
            }
            SyntaxKind::LeftBrace | SyntaxKind::RightBrace => return None,
            _ => index += 1,
        }
    }

    None
}
