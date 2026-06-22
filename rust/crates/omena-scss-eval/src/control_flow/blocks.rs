use omena_parser::{LexedToken, StyleDialect};
use omena_syntax::SyntaxKind;
use omena_transform_cst::StableNodeKeyV0;

use super::model::OmenaScssEvalControlFlowBlockV0;
use super::tokens::{matching_block_end_token_index, next_block_start_token_index};

pub(super) fn control_flow_block_from_token(
    source: &str,
    tokens: &[LexedToken],
    token_index: usize,
    token: &LexedToken,
    dialect: StyleDialect,
) -> Option<OmenaScssEvalControlFlowBlockV0> {
    if token.kind != SyntaxKind::AtKeyword {
        return None;
    }
    let node_kind = scss_control_node_kind_from_name(token.text.as_str(), dialect)?;
    let kind = scss_control_block_kind(node_kind)?;
    let has_back_edge = scss_control_block_has_back_edge(node_kind);
    let source_span_start = token.range.start().into();
    let source_span_end =
        control_flow_body_span_end(tokens, token_index).unwrap_or_else(|| token.range.end().into());
    let header_text = control_flow_header_text(source, tokens, token_index);
    let successor_count = scss_control_block_successor_count(node_kind, header_text.as_str());
    Some(OmenaScssEvalControlFlowBlockV0 {
        node_key: scss_eval_stable_node_key(
            "scss-control",
            kind,
            source_span_start,
            source_span_end,
        ),
        kind,
        at_rule_name: token.text.to_string(),
        header_text,
        source_span_start,
        source_span_end,
        successor_count,
        has_back_edge,
    })
}

fn control_flow_body_span_end(tokens: &[LexedToken], token_index: usize) -> Option<usize> {
    let block_start_index = next_block_start_token_index(tokens, token_index + 1)?;
    let block_end_index = matching_block_end_token_index(tokens, block_start_index)?;
    tokens
        .get(block_end_index)
        .map(|token| token.range.end().into())
}

pub(super) fn scss_eval_stable_node_key(
    prefix: &str,
    kind: &str,
    source_span_start: usize,
    source_span_end: usize,
) -> StableNodeKeyV0 {
    StableNodeKeyV0(format!(
        "{prefix}:{kind}@{source_span_start}..{source_span_end}"
    ))
}

pub(super) fn control_flow_header_text(
    source: &str,
    tokens: &[LexedToken],
    token_index: usize,
) -> String {
    let Some(token) = tokens.get(token_index) else {
        return String::new();
    };
    let header_start = token.range.end().into();
    let header_end = tokens
        .iter()
        .skip(token_index + 1)
        .find(|candidate| {
            matches!(
                candidate.kind,
                SyntaxKind::LeftBrace
                    | SyntaxKind::Semicolon
                    | SyntaxKind::SassIndent
                    | SyntaxKind::SassOptionalSemicolon
            )
        })
        .map(|candidate| candidate.range.start().into())
        .unwrap_or(header_start);
    source
        .get(header_start..header_end)
        .unwrap_or("")
        .trim()
        .to_string()
}

fn scss_control_node_kind_from_name(name: &str, dialect: StyleDialect) -> Option<SyntaxKind> {
    match (dialect, name.to_ascii_lowercase().as_str()) {
        (StyleDialect::Css, "@when") => Some(SyntaxKind::WhenRule),
        (StyleDialect::Css, "@else") => Some(SyntaxKind::ElseRule),
        (StyleDialect::Scss | StyleDialect::Sass, "@if") => Some(SyntaxKind::ScssControlIf),
        (StyleDialect::Scss | StyleDialect::Sass, "@else") => Some(SyntaxKind::ScssControlElse),
        (StyleDialect::Scss | StyleDialect::Sass, "@for") => Some(SyntaxKind::ScssControlFor),
        (StyleDialect::Scss | StyleDialect::Sass, "@each") => Some(SyntaxKind::ScssControlEach),
        (StyleDialect::Scss | StyleDialect::Sass, "@while") => Some(SyntaxKind::ScssControlWhile),
        _ => None,
    }
}

fn scss_control_block_kind(kind: SyntaxKind) -> Option<&'static str> {
    match kind {
        SyntaxKind::ScssControlIf | SyntaxKind::WhenRule => Some("branchIf"),
        SyntaxKind::ScssControlElse | SyntaxKind::ElseRule => Some("branchElse"),
        SyntaxKind::ScssControlFor | SyntaxKind::ScssControlEach | SyntaxKind::ScssControlWhile => {
            Some("loop")
        }
        _ => None,
    }
}

const fn scss_control_block_has_back_edge(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::ScssControlFor | SyntaxKind::ScssControlEach | SyntaxKind::ScssControlWhile
    )
}

fn scss_control_block_successor_count(kind: SyntaxKind, header: &str) -> usize {
    match kind {
        SyntaxKind::ScssControlIf | SyntaxKind::WhenRule => 2,
        SyntaxKind::ScssControlElse if scss_else_if_header_condition(header).is_some() => 2,
        SyntaxKind::ScssControlElse | SyntaxKind::ElseRule => 1,
        SyntaxKind::ScssControlFor | SyntaxKind::ScssControlEach | SyntaxKind::ScssControlWhile => {
            2
        }
        _ => 0,
    }
}

pub(super) fn scss_else_if_header_condition(header: &str) -> Option<&str> {
    let trimmed = header.trim();
    let prefix = trimmed.get(..2)?;
    let rest = trimmed.get(2..)?;
    if !prefix.eq_ignore_ascii_case("if") || !rest.chars().next().is_some_and(char::is_whitespace) {
        return None;
    }
    Some(rest.trim()).filter(|condition| !condition.is_empty())
}
