use std::collections::BTreeMap;

use omena_parser::{LexedToken, StyleDialect, lex};
use omena_syntax::SyntaxKind;
use omena_transform_cst::StableNodeKeyV0;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaScssEvalControlFlowIrSummaryV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub mode: &'static str,
    pub dialect: &'static str,
    pub node_key_type: &'static str,
    pub flat_css_cfg_built: bool,
    pub merged_cross_file_graph: bool,
    pub block_count: usize,
    pub branch_block_count: usize,
    pub loop_block_count: usize,
    pub back_edge_count: usize,
    pub edge_count: usize,
    pub blocks: Vec<OmenaScssEvalControlFlowBlockV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaScssEvalControlFlowBlockV0 {
    pub node_key: StableNodeKeyV0,
    pub kind: &'static str,
    pub at_rule_name: String,
    pub source_span_start: usize,
    pub source_span_end: usize,
    pub successor_count: usize,
    pub has_back_edge: bool,
}

pub fn summarize_scss_control_flow_ir(
    source: &str,
    dialect: StyleDialect,
) -> Option<OmenaScssEvalControlFlowIrSummaryV0> {
    if !matches!(dialect, StyleDialect::Scss | StyleDialect::Sass) {
        return None;
    }
    let lexed = lex(source, dialect);
    let mut ordinals = BTreeMap::<&'static str, usize>::new();
    let blocks = lexed
        .tokens()
        .iter()
        .filter_map(|token| control_flow_block_from_token(token, &mut ordinals))
        .collect::<Vec<_>>();
    let branch_block_count = blocks
        .iter()
        .filter(|block| block.kind.starts_with("branch"))
        .count();
    let loop_block_count = blocks.iter().filter(|block| block.kind == "loop").count();
    let back_edge_count = blocks.iter().filter(|block| block.has_back_edge).count();
    let edge_count = blocks.iter().map(|block| block.successor_count).sum();
    Some(OmenaScssEvalControlFlowIrSummaryV0 {
        schema_version: "0",
        product: "omena-scss-eval.control-flow-ir",
        mode: "oracleOnly",
        dialect: dialect_label(dialect),
        node_key_type: "StableNodeKeyV0",
        flat_css_cfg_built: false,
        merged_cross_file_graph: false,
        block_count: blocks.len(),
        branch_block_count,
        loop_block_count,
        back_edge_count,
        edge_count,
        blocks,
    })
}

fn control_flow_block_from_token(
    token: &LexedToken,
    ordinals: &mut BTreeMap<&'static str, usize>,
) -> Option<OmenaScssEvalControlFlowBlockV0> {
    if token.kind != SyntaxKind::AtKeyword {
        return None;
    }
    let node_kind = scss_control_node_kind_from_name(token.text.as_str())?;
    let kind = scss_control_block_kind(node_kind)?;
    let ordinal = ordinals
        .entry(kind)
        .and_modify(|value| *value += 1)
        .or_insert(0);
    let has_back_edge = scss_control_block_has_back_edge(node_kind);
    Some(OmenaScssEvalControlFlowBlockV0 {
        node_key: StableNodeKeyV0(format!("scss-control:{kind}#{}", *ordinal)),
        kind,
        at_rule_name: token.text.to_string(),
        source_span_start: token.range.start().into(),
        source_span_end: token.range.end().into(),
        successor_count: scss_control_block_successor_count(node_kind),
        has_back_edge,
    })
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

const fn scss_control_block_successor_count(kind: SyntaxKind) -> usize {
    match kind {
        SyntaxKind::ScssControlIf => 2,
        SyntaxKind::ScssControlElse => 1,
        SyntaxKind::ScssControlFor | SyntaxKind::ScssControlEach | SyntaxKind::ScssControlWhile => {
            2
        }
        _ => 0,
    }
}

const fn dialect_label(dialect: StyleDialect) -> &'static str {
    match dialect {
        StyleDialect::Css => "css",
        StyleDialect::Scss => "scss",
        StyleDialect::Sass => "sass",
        StyleDialect::Less => "less",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scss_control_flow_ir_summarizes_branch_and_loop_blocks() {
        let source = "@if $enabled { .on { color: green; } } @else { .off { color: red; } } @for $i from 1 through 3 { .n { order: $i; } } @each $k, $v in $map { .e { color: $v; } } @while $enabled { .w { color: red; } }";
        let report = summarize_scss_control_flow_ir(source, StyleDialect::Scss);
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.mode, "oracleOnly");
        assert!(!report.flat_css_cfg_built);
        assert!(!report.merged_cross_file_graph);
        assert_eq!(report.node_key_type, "StableNodeKeyV0");
        assert_eq!(report.block_count, 5);
        assert_eq!(report.branch_block_count, 2);
        assert_eq!(report.loop_block_count, 3);
        assert_eq!(report.back_edge_count, 3);
        assert!(
            report
                .blocks
                .iter()
                .any(|block| block.node_key.as_str() == "scss-control:branchIf#0")
        );
    }

    #[test]
    fn control_flow_ir_does_not_build_flat_css_cfg() {
        assert!(
            summarize_scss_control_flow_ir(".button { color: red; }", StyleDialect::Css).is_none()
        );
    }
}
