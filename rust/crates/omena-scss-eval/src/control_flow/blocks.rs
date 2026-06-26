use cstree::syntax::SyntaxNode;
use omena_parser::{LexedToken, StyleDialect};
use omena_syntax::SyntaxKind;
use omena_transform_cst::StableNodeKeyV0;

use super::model::OmenaScssEvalControlFlowBlockV0;
use super::scanner_tokens::{
    matching_block_end_token_index, matching_right_paren_token_index, next_block_start_token_index,
    next_non_trivia_token_index,
};

pub(super) fn control_flow_block_from_token_scanner_oracle(
    source: &str,
    tokens: &[LexedToken],
    token_index: usize,
    token: &LexedToken,
    dialect: StyleDialect,
) -> Option<OmenaScssEvalControlFlowBlockV0> {
    if let Some(block) =
        control_flow_at_rule_block_from_token(source, tokens, token_index, token, dialect)
    {
        return Some(block);
    }
    native_css_if_function_block_from_token(source, tokens, token_index, token, dialect)
}

pub(super) fn control_flow_blocks_from_cst(
    source: &str,
    root: &SyntaxNode<SyntaxKind>,
    dialect: StyleDialect,
) -> Vec<OmenaScssEvalControlFlowBlockV0> {
    root.descendants()
        .filter_map(|node| {
            control_flow_block_from_cst_node(source, node, dialect)
                .or_else(|| native_css_if_function_block_from_cst_node(source, node, dialect))
        })
        .collect()
}

fn control_flow_block_from_cst_node(
    source: &str,
    node: &SyntaxNode<SyntaxKind>,
    dialect: StyleDialect,
) -> Option<OmenaScssEvalControlFlowBlockV0> {
    if !cst_control_flow_kind_matches_dialect(node.kind(), dialect) {
        return None;
    }
    let kind = scss_control_block_kind(node.kind())?;
    let has_back_edge = scss_control_block_has_back_edge(node.kind());
    let (at_rule_name, source_span_start, header_start) = cst_at_keyword_name_and_span(node)?;
    let source_span_end = u32::from(node.text_range().end()) as usize;
    let header_text = cst_control_flow_header_text(source, node, header_start);
    let successor_count = scss_control_block_successor_count(node.kind(), header_text.as_str());
    Some(OmenaScssEvalControlFlowBlockV0 {
        node_key: scss_eval_stable_node_key(
            "scss-control",
            kind,
            source_span_start,
            source_span_end,
        ),
        kind,
        at_rule_name,
        header_text,
        source_span_start,
        source_span_end,
        successor_count,
        has_back_edge,
    })
}

fn cst_control_flow_kind_matches_dialect(kind: SyntaxKind, dialect: StyleDialect) -> bool {
    matches!(
        (dialect, kind),
        (
            StyleDialect::Css,
            SyntaxKind::WhenRule | SyntaxKind::ElseRule
        ) | (
            StyleDialect::Scss | StyleDialect::Sass,
            SyntaxKind::ScssControlIf
                | SyntaxKind::ScssControlElse
                | SyntaxKind::ScssControlFor
                | SyntaxKind::ScssControlEach
                | SyntaxKind::ScssControlWhile
        )
    )
}

fn cst_at_keyword_name_and_span(node: &SyntaxNode<SyntaxKind>) -> Option<(String, usize, usize)> {
    for token in node
        .descendants_with_tokens()
        .filter_map(|element| element.into_token())
    {
        if token.kind() != SyntaxKind::AtKeyword {
            continue;
        }
        let name = syntax_token_text(token)?;
        let source_span_start = u32::from(token.text_range().start()) as usize;
        let header_start = u32::from(token.text_range().end()) as usize;
        return Some((name, source_span_start, header_start));
    }
    None
}

fn cst_control_flow_header_text(
    source: &str,
    node: &SyntaxNode<SyntaxKind>,
    header_start: usize,
) -> String {
    let header_end = node
        .descendants_with_tokens()
        .filter_map(|element| element.into_token())
        .find(|token| {
            matches!(
                token.kind(),
                SyntaxKind::LeftBrace
                    | SyntaxKind::Semicolon
                    | SyntaxKind::SassIndent
                    | SyntaxKind::SassOptionalSemicolon
            )
        })
        .map(|token| u32::from(token.text_range().start()) as usize)
        .unwrap_or_else(|| u32::from(node.text_range().end()) as usize);
    source
        .get(header_start..header_end)
        .unwrap_or("")
        .trim()
        .to_string()
}

fn syntax_token_text(token: &cstree::syntax::SyntaxToken<SyntaxKind>) -> Option<String> {
    if let Some(resolver) = token.resolver() {
        Some(token.resolve_text(&**resolver).to_string())
    } else {
        token.static_text().map(str::to_string)
    }
}

fn control_flow_at_rule_block_from_token(
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

fn native_css_if_function_block_from_token(
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

fn native_css_if_function_block_from_cst_node(
    source: &str,
    node: &SyntaxNode<SyntaxKind>,
    dialect: StyleDialect,
) -> Option<OmenaScssEvalControlFlowBlockV0> {
    if dialect != StyleDialect::Css
        || !matches!(
            node.kind(),
            SyntaxKind::FunctionCall | SyntaxKind::IfFunction
        )
    {
        return None;
    }
    let tokens = node
        .descendants_with_tokens()
        .filter_map(|element| element.into_token())
        .filter(|token| !token.kind().is_trivia())
        .collect::<Vec<_>>();
    let [function_name, left_paren, ..] = tokens.as_slice() else {
        return None;
    };
    let source_span_start = u32::from(function_name.text_range().start()) as usize;
    let function_name_end = u32::from(function_name.text_range().end()) as usize;
    let function_name_text = source.get(source_span_start..function_name_end)?;
    if function_name.kind() != SyntaxKind::Ident
        || !function_name_text.eq_ignore_ascii_case("if")
        || left_paren.kind() != SyntaxKind::LeftParen
    {
        return None;
    }
    let right_paren = tokens
        .iter()
        .rev()
        .find(|token| token.kind() == SyntaxKind::RightParen)?;
    let source_span_end = u32::from(right_paren.text_range().end()) as usize;
    let header_start = u32::from(left_paren.text_range().end()) as usize;
    let header_end = u32::from(right_paren.text_range().start()) as usize;
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
