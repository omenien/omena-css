use omena_parser::{LexedToken, StyleDialect};
use omena_syntax::SyntaxKind;

use super::{
    blocks::{
        scss_control_block_has_back_edge, scss_control_block_kind,
        scss_control_block_successor_count, scss_eval_stable_node_key,
    },
    model::OmenaScssEvalControlFlowBlockV0,
    scanner_tokens::{
        matching_block_end_token_index, matching_right_paren_token_index,
        next_block_start_token_index, next_non_trivia_token_index,
    },
};

pub(super) fn control_flow_block_from_token_scanner_oracle(
    source: &str,
    tokens: &[LexedToken],
    token_index: usize,
    token: &LexedToken,
    dialect: StyleDialect,
) -> Option<OmenaScssEvalControlFlowBlockV0> {
    if let Some(block) = control_flow_at_rule_block_from_token_scanner_oracle(
        source,
        tokens,
        token_index,
        token,
        dialect,
    ) {
        return Some(block);
    }
    native_css_if_function_block_from_token_scanner_oracle(
        source,
        tokens,
        token_index,
        token,
        dialect,
    )
}

fn control_flow_at_rule_block_from_token_scanner_oracle(
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
    let source_span_end = control_flow_body_span_end_scanner_oracle(tokens, token_index)
        .unwrap_or_else(|| token.range.end().into());
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

fn native_css_if_function_block_from_token_scanner_oracle(
    source: &str,
    tokens: &[LexedToken],
    token_index: usize,
    token: &LexedToken,
    dialect: StyleDialect,
) -> Option<OmenaScssEvalControlFlowBlockV0> {
    if dialect != StyleDialect::Css
        || token.kind != SyntaxKind::Ident
        || !token.text.eq_ignore_ascii_case("if")
    {
        return None;
    }
    let left_paren_index = next_non_trivia_token_index(tokens, token_index + 1)?;
    if tokens.get(left_paren_index)?.kind != SyntaxKind::LeftParen {
        return None;
    }
    let right_paren_index = matching_right_paren_token_index(tokens, left_paren_index)?;
    let source_span_start = token.range.start().into();
    let source_span_end = tokens.get(right_paren_index)?.range.end().into();
    let header_start: usize = tokens.get(left_paren_index)?.range.end().into();
    let header_end: usize = tokens.get(right_paren_index)?.range.start().into();
    let header_text = source
        .get(header_start..header_end)
        .unwrap_or("")
        .trim()
        .to_string();

    Some(OmenaScssEvalControlFlowBlockV0 {
        node_key: scss_eval_stable_node_key(
            "css-value-control",
            "branchIf",
            source_span_start,
            source_span_end,
        ),
        kind: "branchIf",
        at_rule_name: "if()".to_string(),
        header_text,
        source_span_start,
        source_span_end,
        successor_count: 2,
        has_back_edge: false,
    })
}

fn control_flow_body_span_end_scanner_oracle(
    tokens: &[LexedToken],
    token_index: usize,
) -> Option<usize> {
    let block_start_index = next_block_start_token_index(tokens, token_index + 1)?;
    let block_end_index = matching_block_end_token_index(tokens, block_start_index)?;
    tokens
        .get(block_end_index)
        .map(|token| token.range.end().into())
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
