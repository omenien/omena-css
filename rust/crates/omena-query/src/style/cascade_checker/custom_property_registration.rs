use std::collections::BTreeMap;

use omena_parser::{LexedToken, lex};
use omena_query_checker_orchestrator::OmenaCheckerCustomPropertyRegistrationInputV0;
use omena_syntax::SyntaxKind;

use super::omena_parser_dialect_for_style_path;

pub(in crate::style) fn collect_query_checker_custom_property_registrations(
    style_uri: &str,
    source: &str,
) -> Vec<OmenaCheckerCustomPropertyRegistrationInputV0> {
    let dialect = omena_parser_dialect_for_style_path(style_uri);
    let lexed = lex(source, dialect);
    let tokens = lexed.tokens();
    let mut registrations = Vec::new();
    let mut index = 0usize;
    while index < tokens.len() {
        if tokens[index].kind == SyntaxKind::AtKeyword
            && tokens[index].text.eq_ignore_ascii_case("@property")
            && let Some((registration, next_index)) =
                parse_query_checker_custom_property_registration(source, tokens, index)
        {
            registrations.push(registration);
            index = next_index;
            continue;
        }
        index += 1;
    }
    registrations
}

fn parse_query_checker_custom_property_registration(
    source: &str,
    tokens: &[LexedToken],
    at_property_index: usize,
) -> Option<(OmenaCheckerCustomPropertyRegistrationInputV0, usize)> {
    let name_index = skip_query_trivia(tokens, at_property_index + 1, tokens.len());
    let name = normalize_query_checker_custom_property_name(tokens.get(name_index)?.text.as_str())?;
    let block_start_index = find_query_registration_block_start(tokens, name_index + 1)?;
    let block_end_index = matching_query_registration_block_end(tokens, block_start_index)?;
    let declarations = collect_query_checker_registration_declarations(
        source,
        tokens,
        block_start_index,
        block_end_index,
    );

    Some((
        OmenaCheckerCustomPropertyRegistrationInputV0 {
            name,
            syntax: declarations.get("syntax").cloned(),
            inherits: declarations.get("inherits").cloned(),
            initial_value: declarations.get("initial-value").cloned(),
        },
        block_end_index + 1,
    ))
}

fn collect_query_checker_registration_declarations(
    source: &str,
    tokens: &[LexedToken],
    block_start_index: usize,
    block_end_index: usize,
) -> BTreeMap<String, String> {
    let mut declarations = BTreeMap::new();
    let mut index = block_start_index + 1;
    while index < block_end_index {
        index = skip_query_trivia(tokens, index, block_end_index);
        while index < block_end_index && query_registration_statement_ends(tokens[index].kind) {
            index = skip_query_trivia(tokens, index + 1, block_end_index);
        }
        if index >= block_end_index {
            break;
        }
        let property_index = index;
        let Some(colon_index) =
            find_query_registration_colon(tokens, property_index, block_end_index)
        else {
            break;
        };
        let property = source
            [token_start(tokens, property_index)..token_start(tokens, colon_index)]
            .trim()
            .to_ascii_lowercase();
        let value_start_index = skip_query_trivia(tokens, colon_index + 1, block_end_index);
        let value_end_index =
            find_query_registration_statement_end(tokens, value_start_index, block_end_index);
        if value_start_index < value_end_index {
            let raw_value = source
                [token_start(tokens, value_start_index)..token_end(tokens, value_end_index - 1)]
                .trim();
            let (value, important) = strip_query_registration_important(raw_value);
            if !important && !property.is_empty() {
                declarations.insert(property, value.to_string());
            }
        }
        index = value_end_index.saturating_add(1);
    }
    declarations
}

fn skip_query_trivia(tokens: &[LexedToken], mut index: usize, end: usize) -> usize {
    while index < end && tokens[index].kind.is_trivia() {
        index += 1;
    }
    index
}

fn normalize_query_checker_custom_property_name(text: &str) -> Option<String> {
    let name = text.trim();
    (name.starts_with("--") && name.len() > 2).then(|| name.to_string())
}

fn find_query_registration_block_start(tokens: &[LexedToken], index: usize) -> Option<usize> {
    tokens
        .iter()
        .enumerate()
        .skip(index)
        .find_map(|(candidate_index, token)| {
            (token.kind == SyntaxKind::LeftBrace).then_some(candidate_index)
        })
}

fn matching_query_registration_block_end(
    tokens: &[LexedToken],
    block_start_index: usize,
) -> Option<usize> {
    let mut depth = 0usize;
    for (index, token) in tokens.iter().enumerate().skip(block_start_index) {
        match token.kind {
            SyntaxKind::LeftBrace => depth += 1,
            SyntaxKind::RightBrace => {
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

fn find_query_registration_colon(tokens: &[LexedToken], start: usize, end: usize) -> Option<usize> {
    for (index, token) in tokens.iter().enumerate().take(end).skip(start) {
        match token.kind {
            SyntaxKind::Colon => return Some(index),
            SyntaxKind::RightBrace => return None,
            kind if query_registration_statement_ends(kind) => return None,
            _ => {}
        }
    }
    None
}

fn find_query_registration_statement_end(tokens: &[LexedToken], start: usize, end: usize) -> usize {
    tokens
        .iter()
        .enumerate()
        .take(end)
        .skip(start)
        .find_map(|(index, token)| {
            (query_registration_statement_ends(token.kind) || token.kind == SyntaxKind::RightBrace)
                .then_some(index)
        })
        .unwrap_or(end)
}

fn query_registration_statement_ends(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::Semicolon | SyntaxKind::SassOptionalSemicolon
    )
}

fn strip_query_registration_important(value: &str) -> (&str, bool) {
    let compact = value
        .chars()
        .filter(|character| !character.is_ascii_whitespace())
        .collect::<String>()
        .to_ascii_lowercase();
    if !compact.ends_with("!important") {
        return (value, false);
    }
    let Some(bang_index) = value.rfind('!') else {
        return (value, false);
    };
    (value[..bang_index].trim_end(), true)
}

fn token_start(tokens: &[LexedToken], index: usize) -> usize {
    u32::from(tokens[index].range.start()) as usize
}

fn token_end(tokens: &[LexedToken], index: usize) -> usize {
    u32::from(tokens[index].range.end()) as usize
}
