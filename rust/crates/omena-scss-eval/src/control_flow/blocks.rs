use omena_parser::LexedToken;
use omena_syntax::SyntaxKind;
use omena_transform_cst::StableNodeKeyV0;

use super::model::OmenaScssEvalControlFlowBlockV0;
use super::tokens::matching_right_brace_token_index;

pub(super) fn control_flow_block_from_token(
    source: &str,
    tokens: &[LexedToken],
    token_index: usize,
    token: &LexedToken,
) -> Option<OmenaScssEvalControlFlowBlockV0> {
    if token.kind != SyntaxKind::AtKeyword {
        return None;
    }
    let node_kind = scss_control_node_kind_from_name(token.text.as_str())?;
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
    let mut left_brace_index = None;
    for (index, token) in tokens.iter().enumerate().skip(token_index + 1) {
        match token.kind {
            SyntaxKind::LeftBrace => {
                left_brace_index = Some(index);
                break;
            }
            SyntaxKind::Semicolon | SyntaxKind::SassOptionalSemicolon => return None,
            _ => {}
        }
    }
    let left_brace_index = left_brace_index?;
    let right_brace_index = matching_right_brace_token_index(tokens, left_brace_index)?;
    tokens
        .get(right_brace_index)
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

fn scss_control_node_kind_from_name(name: &str) -> Option<SyntaxKind> {
    match name.to_ascii_lowercase().as_str() {
        "@if" => Some(SyntaxKind::ScssControlIf),
        "@else" => Some(SyntaxKind::ScssControlElse),
        "@for" => Some(SyntaxKind::ScssControlFor),
        "@each" => Some(SyntaxKind::ScssControlEach),
        "@while" => Some(SyntaxKind::ScssControlWhile),
        _ => None,
    }
}

fn scss_control_block_kind(kind: SyntaxKind) -> Option<&'static str> {
    match kind {
        SyntaxKind::ScssControlIf => Some("branchIf"),
        SyntaxKind::ScssControlElse => Some("branchElse"),
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
        SyntaxKind::ScssControlIf => 2,
        SyntaxKind::ScssControlElse if scss_else_if_header_condition(header).is_some() => 2,
        SyntaxKind::ScssControlElse => 1,
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
