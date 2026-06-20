use super::super::parser_facade::lex_omena_query_omena_parser_style_source;
use super::super::{transform_token_end, transform_token_start};
use super::scss_variable_overrides;
use crate::OmenaParserStyleDialect;
use omena_syntax::SyntaxKind;
use std::collections::BTreeMap;

pub(super) fn static_scss_identifier_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-')
}

pub(super) fn static_scss_use_rule_semicolon(
    tokens: &[omena_parser::LexedToken],
    at_use_index: usize,
) -> Option<usize> {
    let mut index = at_use_index + 1;
    while index < tokens.len() {
        match tokens[index].kind {
            SyntaxKind::Semicolon => return Some(index),
            SyntaxKind::LeftBrace | SyntaxKind::RightBrace => return None,
            _ => index += 1,
        }
    }
    None
}

pub(super) fn static_scss_module_rule_source_name(
    tokens: &[omena_parser::LexedToken],
    start_index: usize,
    end_index: usize,
) -> Option<String> {
    tokens[start_index..end_index]
        .iter()
        .find(|token| matches!(token.kind, SyntaxKind::String | SyntaxKind::Url))
        .map(|token| token.text.trim_matches('"').trim_matches('\'').to_string())
}

pub(in crate::style::transform) fn derive_static_scss_module_rule_variable_overrides_at_ordinal(
    style_source: &str,
    at_keyword: &str,
    rule_ordinal: usize,
) -> BTreeMap<String, String> {
    static_scss_module_rule_source_at_ordinal(style_source, at_keyword, rule_ordinal)
        .map(parse_static_scss_use_variable_overrides_from_rule)
        .unwrap_or_default()
}

pub(super) fn static_scss_module_rule_source_at_ordinal<'a>(
    style_source: &'a str,
    at_keyword: &str,
    rule_ordinal: usize,
) -> Option<&'a str> {
    let lexed =
        lex_omena_query_omena_parser_style_source(style_source, OmenaParserStyleDialect::Scss);
    let tokens = lexed.tokens();
    let mut depth = 0usize;
    let mut index = 0usize;
    let mut current_rule_ordinal = 0usize;

    while index < tokens.len() {
        match tokens[index].kind {
            SyntaxKind::LeftBrace => depth += 1,
            SyntaxKind::RightBrace => depth = depth.saturating_sub(1),
            SyntaxKind::AtKeyword
                if depth == 0 && tokens[index].text.eq_ignore_ascii_case(at_keyword) =>
            {
                let Some(end_index) = static_scss_use_rule_semicolon(tokens, index) else {
                    index += 1;
                    continue;
                };
                if static_scss_module_rule_source_name(tokens, index + 1, end_index).is_some() {
                    if current_rule_ordinal == rule_ordinal {
                        let start = transform_token_start(&tokens[index]);
                        let end = transform_token_end(&tokens[end_index]);
                        return style_source.get(start..end);
                    }
                    current_rule_ordinal += 1;
                }
                index = end_index + 1;
                continue;
            }
            _ => {}
        }
        index += 1;
    }

    None
}

fn parse_static_scss_use_variable_overrides_from_rule(
    rule_source: &str,
) -> BTreeMap<String, String> {
    let lexed =
        lex_omena_query_omena_parser_style_source(rule_source, OmenaParserStyleDialect::Scss);
    let tokens = lexed.tokens();
    let Some(with_index) = tokens
        .iter()
        .position(|token| token.text.eq_ignore_ascii_case("with"))
    else {
        return BTreeMap::new();
    };
    let Some(left_paren_index) = tokens[with_index + 1..]
        .iter()
        .position(|token| token.kind == SyntaxKind::LeftParen)
        .map(|offset| with_index + 1 + offset)
    else {
        return BTreeMap::new();
    };
    let Some(right_paren_index) =
        scss_variable_overrides::static_scss_matching_right_paren(tokens, left_paren_index)
    else {
        return BTreeMap::new();
    };
    let start = transform_token_end(&tokens[left_paren_index]);
    let end = transform_token_start(&tokens[right_paren_index]);
    rule_source
        .get(start..end)
        .map(scss_variable_overrides::parse_static_scss_use_variable_override_list)
        .unwrap_or_default()
}
