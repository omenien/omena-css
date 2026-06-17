use std::collections::BTreeMap;

use omena_abstract_value::{
    AbstractCssValueV0, BoundedJoinFixpointNodeV0, abstract_css_value_from_text,
    analyze_bounded_join_fixpoint, join_abstract_css_values,
};
use omena_parser::{LexedToken, StyleDialect, lex};
use omena_syntax::SyntaxKind;
use omena_transform_cst::StableNodeKeyV0;
use serde::Serialize;

use crate::abstract_css_value_kind;

const SCSS_CONTROL_FLOW_FIXPOINT_ITERATION_LIMIT: usize = 32;

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
    pub header_text: String,
    pub source_span_start: usize,
    pub source_span_end: usize,
    pub successor_count: usize,
    pub has_back_edge: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaScssEvalControlFlowValueAnalysisV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub mode: &'static str,
    pub dialect: &'static str,
    pub value_type: &'static str,
    pub max_iterations: usize,
    pub converged: bool,
    pub iteration_count: usize,
    pub block_count: usize,
    pub back_edge_count: usize,
    pub loop_carried_binding_count: usize,
    pub widened_to_top_count: usize,
    pub flat_css_cfg_built: bool,
    pub merged_cross_file_graph: bool,
    pub blocks: Vec<OmenaScssEvalControlFlowValueBlockV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaScssEvalControlFlowValueBlockV0 {
    pub node_key: StableNodeKeyV0,
    pub kind: &'static str,
    pub transfer_kind: &'static str,
    pub predecessor_node_keys: Vec<StableNodeKeyV0>,
    pub loop_carried_bindings: Vec<String>,
    pub loop_carried_binding_values: Vec<OmenaScssEvalControlFlowBindingValueV0>,
    pub input_value: AbstractCssValueV0,
    pub input_value_kind: &'static str,
    pub output_value: AbstractCssValueV0,
    pub output_value_kind: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaScssEvalControlFlowBindingValueV0 {
    pub name: String,
    pub value: AbstractCssValueV0,
    pub value_kind: &'static str,
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
    let tokens = lexed.tokens();
    let blocks = tokens
        .iter()
        .enumerate()
        .filter_map(|(index, token)| {
            control_flow_block_from_token(source, tokens, index, token, &mut ordinals)
        })
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

pub fn analyze_scss_control_flow_values(
    source: &str,
    dialect: StyleDialect,
) -> Option<OmenaScssEvalControlFlowValueAnalysisV0> {
    if !matches!(dialect, StyleDialect::Scss | StyleDialect::Sass) {
        return None;
    }
    let summary = summarize_scss_control_flow_ir(source, dialect)?;
    let lexical_bindings = collect_lexical_scss_bindings(source, dialect);
    let nodes = summary
        .blocks
        .iter()
        .enumerate()
        .map(|(index, block)| {
            let predecessor_indices = control_flow_predecessor_indices(index, block);
            ScssControlFlowAnalysisNode {
                block: block.clone(),
                predecessor_indices,
                transfer: control_flow_transfer_for_block(block, &lexical_bindings),
            }
        })
        .collect::<Vec<_>>();
    let fixpoint = run_scss_control_flow_fixpoint(&nodes);
    let back_edge_count = nodes.iter().filter(|node| node.block.has_back_edge).count();
    let loop_carried_binding_count = nodes
        .iter()
        .map(|node| node.transfer.loop_carried_bindings().len())
        .sum();
    let blocks = nodes
        .iter()
        .zip(fixpoint.input_values.iter())
        .zip(fixpoint.output_values.iter())
        .map(
            |((node, input_value), output_value)| OmenaScssEvalControlFlowValueBlockV0 {
                node_key: node.block.node_key.clone(),
                kind: node.block.kind,
                transfer_kind: node.transfer.kind_label(),
                predecessor_node_keys: node
                    .predecessor_indices
                    .iter()
                    .filter_map(|index| nodes.get(*index).map(|node| node.block.node_key.clone()))
                    .collect(),
                loop_carried_bindings: node.transfer.loop_carried_bindings(),
                loop_carried_binding_values: node.transfer.loop_carried_binding_values(),
                input_value_kind: abstract_css_value_kind(input_value),
                input_value: input_value.clone(),
                output_value_kind: abstract_css_value_kind(output_value),
                output_value: output_value.clone(),
            },
        )
        .collect::<Vec<_>>();
    Some(OmenaScssEvalControlFlowValueAnalysisV0 {
        schema_version: "0",
        product: "omena-scss-eval.control-flow-value-analysis",
        mode: "oracleOnly",
        dialect: dialect_label(dialect),
        value_type: "AbstractCssValueV0",
        max_iterations: SCSS_CONTROL_FLOW_FIXPOINT_ITERATION_LIMIT,
        converged: fixpoint.converged,
        iteration_count: fixpoint.iteration_count,
        block_count: nodes.len(),
        back_edge_count,
        loop_carried_binding_count,
        widened_to_top_count: fixpoint.widened_to_top_count,
        flat_css_cfg_built: false,
        merged_cross_file_graph: false,
        blocks,
    })
}

fn control_flow_block_from_token(
    source: &str,
    tokens: &[LexedToken],
    token_index: usize,
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
        header_text: control_flow_header_text(source, tokens, token_index),
        source_span_start: token.range.start().into(),
        source_span_end: token.range.end().into(),
        successor_count: scss_control_block_successor_count(node_kind),
        has_back_edge,
    })
}

fn control_flow_header_text(source: &str, tokens: &[LexedToken], token_index: usize) -> String {
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

#[derive(Debug, Clone, PartialEq, Eq)]
struct ScssControlFlowAnalysisNode {
    block: OmenaScssEvalControlFlowBlockV0,
    predecessor_indices: Vec<usize>,
    transfer: ScssControlFlowTransfer,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ScssControlFlowTransfer {
    BranchCondition {
        value: AbstractCssValueV0,
    },
    LoopCarried {
        bindings: Vec<ScssControlFlowBindingValue>,
        value: AbstractCssValueV0,
    },
    PassThrough,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ScssControlFlowBindingValue {
    name: String,
    value: AbstractCssValueV0,
}

impl ScssControlFlowTransfer {
    const fn kind_label(&self) -> &'static str {
        match self {
            Self::BranchCondition { .. } => "branchCondition",
            Self::LoopCarried { .. } => "loopCarriedBindings",
            Self::PassThrough => "passThrough",
        }
    }

    fn loop_carried_bindings(&self) -> Vec<String> {
        match self {
            Self::LoopCarried { bindings, .. } => bindings
                .iter()
                .map(|binding| binding.name.clone())
                .collect(),
            Self::BranchCondition { .. } | Self::PassThrough => Vec::new(),
        }
    }

    fn loop_carried_binding_values(&self) -> Vec<OmenaScssEvalControlFlowBindingValueV0> {
        match self {
            Self::LoopCarried { bindings, .. } => bindings
                .iter()
                .map(|binding| OmenaScssEvalControlFlowBindingValueV0 {
                    name: binding.name.clone(),
                    value_kind: abstract_css_value_kind(&binding.value),
                    value: binding.value.clone(),
                })
                .collect(),
            Self::BranchCondition { .. } | Self::PassThrough => Vec::new(),
        }
    }

    fn apply(&self, input_value: &AbstractCssValueV0) -> AbstractCssValueV0 {
        match self {
            Self::BranchCondition { value } | Self::LoopCarried { value, .. } => {
                join_abstract_css_values(input_value, value)
            }
            Self::PassThrough => input_value.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ScssControlFlowFixpointResult {
    converged: bool,
    iteration_count: usize,
    widened_to_top_count: usize,
    input_values: Vec<AbstractCssValueV0>,
    output_values: Vec<AbstractCssValueV0>,
}

fn run_scss_control_flow_fixpoint(
    nodes: &[ScssControlFlowAnalysisNode],
) -> ScssControlFlowFixpointResult {
    let flow_nodes = nodes
        .iter()
        .map(|node| BoundedJoinFixpointNodeV0 {
            id: node.block.node_key.0.clone(),
            predecessor_ids: node
                .predecessor_indices
                .iter()
                .filter_map(|index| nodes.get(*index).map(|node| node.block.node_key.0.clone()))
                .collect(),
            transfer: node.transfer.clone(),
        })
        .collect::<Vec<_>>();
    let fixpoint = analyze_bounded_join_fixpoint(
        &flow_nodes,
        SCSS_CONTROL_FLOW_FIXPOINT_ITERATION_LIMIT,
        AbstractCssValueV0::Bottom,
        AbstractCssValueV0::Top,
        join_abstract_css_values,
        |input_value, transfer| transfer.apply(input_value),
    );
    let input_values = fixpoint
        .nodes
        .iter()
        .map(|node| node.input_value.clone())
        .collect::<Vec<_>>();
    let mut output_values = fixpoint
        .nodes
        .iter()
        .map(|node| node.output_value.clone())
        .collect::<Vec<_>>();
    let widened_to_top_count = if fixpoint.converged {
        0
    } else {
        output_values
            .iter_mut()
            .filter(|value| !matches!(value, AbstractCssValueV0::Top))
            .map(|value| {
                *value = AbstractCssValueV0::Top;
            })
            .count()
    };

    ScssControlFlowFixpointResult {
        converged: fixpoint.converged,
        iteration_count: fixpoint.iteration_count,
        widened_to_top_count,
        input_values,
        output_values,
    }
}

fn control_flow_predecessor_indices(
    index: usize,
    block: &OmenaScssEvalControlFlowBlockV0,
) -> Vec<usize> {
    let mut predecessors = Vec::new();
    if index > 0 {
        predecessors.push(index - 1);
    }
    if block.has_back_edge {
        predecessors.push(index);
    }
    predecessors
}

fn control_flow_transfer_for_block(
    block: &OmenaScssEvalControlFlowBlockV0,
    lexical_bindings: &BTreeMap<String, AbstractCssValueV0>,
) -> ScssControlFlowTransfer {
    match block.at_rule_name.to_ascii_lowercase().as_str() {
        "@if" | "@while" => ScssControlFlowTransfer::BranchCondition {
            value: scss_header_value(block.header_text.as_str(), lexical_bindings),
        },
        "@for" | "@each" => {
            let bindings =
                loop_carried_binding_values(block.header_text.as_str(), lexical_bindings);
            ScssControlFlowTransfer::LoopCarried {
                bindings,
                value: loop_carried_value(block.header_text.as_str(), lexical_bindings),
            }
        }
        "@else" => ScssControlFlowTransfer::PassThrough,
        _ => ScssControlFlowTransfer::PassThrough,
    }
}

fn collect_lexical_scss_bindings(
    source: &str,
    dialect: StyleDialect,
) -> BTreeMap<String, AbstractCssValueV0> {
    let lexed = lex(source, dialect);
    let tokens = lexed.tokens();
    let mut bindings = BTreeMap::new();
    for (index, token) in tokens.iter().enumerate() {
        if token.kind != SyntaxKind::ScssVariable {
            continue;
        }
        let Some(colon_index) = next_non_trivia_token_index(tokens, index + 1) else {
            continue;
        };
        if tokens[colon_index].kind != SyntaxKind::Colon {
            continue;
        }
        let value_start = tokens[colon_index].range.end().into();
        let Some(value_end_index) = declaration_end_token_index(tokens, colon_index + 1) else {
            continue;
        };
        let value_end = tokens[value_end_index].range.start().into();
        if let Some(value) = source.get(value_start..value_end).map(str::trim)
            && !value.is_empty()
        {
            bindings.insert(token.text.to_string(), abstract_css_value_from_text(value));
        }
    }
    bindings
}

fn scss_header_value(
    header: &str,
    lexical_bindings: &BTreeMap<String, AbstractCssValueV0>,
) -> AbstractCssValueV0 {
    let variables = variable_names_in_text(header);
    if variables.is_empty() {
        return abstract_css_value_from_text(header);
    }
    variables
        .iter()
        .map(|name| {
            lexical_bindings
                .get(name)
                .cloned()
                .unwrap_or(AbstractCssValueV0::Top)
        })
        .fold(AbstractCssValueV0::Bottom, |acc, value| {
            join_abstract_css_values(&acc, &value)
        })
}

fn loop_carried_value(
    header: &str,
    lexical_bindings: &BTreeMap<String, AbstractCssValueV0>,
) -> AbstractCssValueV0 {
    parse_static_for_loop_range(header)
        .or_else(|| parse_static_each_loop_source_value(header, lexical_bindings))
        .unwrap_or_else(|| scss_header_value(header, lexical_bindings))
}

fn loop_carried_binding_values(
    header: &str,
    lexical_bindings: &BTreeMap<String, AbstractCssValueV0>,
) -> Vec<ScssControlFlowBindingValue> {
    let value = loop_carried_value(header, lexical_bindings);
    loop_carried_bindings(header)
        .into_iter()
        .map(|name| ScssControlFlowBindingValue {
            name,
            value: value.clone(),
        })
        .collect()
}

fn parse_static_for_loop_range(header: &str) -> Option<AbstractCssValueV0> {
    let parts = header.split_whitespace().collect::<Vec<_>>();
    let from_index = parts
        .iter()
        .position(|part| part.eq_ignore_ascii_case("from"))?;
    let to_index = parts
        .iter()
        .position(|part| part.eq_ignore_ascii_case("to") || part.eq_ignore_ascii_case("through"))?;
    let start = parts.get(from_index + 1)?.parse::<i32>().ok()?;
    let end = parts.get(to_index + 1)?.parse::<i32>().ok()?;
    if start > end || end.saturating_sub(start) > 64 {
        return Some(AbstractCssValueV0::Top);
    }
    Some(
        (start..=end).fold(AbstractCssValueV0::Bottom, |acc, value| {
            let value = abstract_css_value_from_text(value.to_string().as_str());
            join_abstract_css_values(&acc, &value)
        }),
    )
}

fn parse_static_each_loop_source_value(
    header: &str,
    lexical_bindings: &BTreeMap<String, AbstractCssValueV0>,
) -> Option<AbstractCssValueV0> {
    let (_, source) = split_header_at_keyword(header, "in")?;
    let source = source.trim();
    if source.is_empty() {
        return None;
    }
    let source_variables = variable_names_in_text(source);
    if !source_variables.is_empty() {
        return Some(
            source_variables
                .iter()
                .map(|name| {
                    lexical_bindings
                        .get(name)
                        .cloned()
                        .unwrap_or(AbstractCssValueV0::Top)
                })
                .fold(AbstractCssValueV0::Bottom, |acc, value| {
                    join_abstract_css_values(&acc, &value)
                }),
        );
    }
    let values = source
        .split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>();
    if values.len() <= 1 || values.len() > 64 {
        return None;
    }
    Some(
        values
            .into_iter()
            .fold(AbstractCssValueV0::Bottom, |acc, value| {
                let value = abstract_css_value_from_text(value);
                join_abstract_css_values(&acc, &value)
            }),
    )
}

fn loop_carried_bindings(header: &str) -> Vec<String> {
    let separator = if header
        .split_whitespace()
        .any(|part| part.eq_ignore_ascii_case("from"))
    {
        "from"
    } else {
        "in"
    };
    let before_separator = split_header_at_keyword(header, separator)
        .map(|(left, _)| left)
        .unwrap_or(header);
    variable_names_in_text(before_separator)
}

fn split_header_at_keyword<'a>(header: &'a str, keyword: &str) -> Option<(&'a str, &'a str)> {
    let lower_header = header.to_ascii_lowercase();
    let lower_keyword = keyword.to_ascii_lowercase();
    let mut search_start = 0usize;
    while search_start < lower_header.len() {
        let relative_index = lower_header
            .get(search_start..)?
            .find(lower_keyword.as_str())?;
        let index = search_start + relative_index;
        let right_start = index + keyword.len();
        if header_keyword_has_boundaries(header, index, right_start) {
            let left = header.get(..index)?;
            let right = header.get(right_start..)?;
            return Some((left, right));
        }
        search_start = right_start;
    }
    None
}

fn header_keyword_has_boundaries(header: &str, start: usize, end: usize) -> bool {
    let before_ok = header.get(..start).is_none_or(|text| {
        text.chars()
            .next_back()
            .is_none_or(|ch| ch.is_ascii_whitespace())
    });
    let after_ok = header.get(end..).is_none_or(|text| {
        text.chars()
            .next()
            .is_none_or(|ch| ch.is_ascii_whitespace())
    });
    before_ok && after_ok
}

fn variable_names_in_text(text: &str) -> Vec<String> {
    let mut names = Vec::new();
    let mut index = 0usize;
    while index < text.len() {
        let Some(ch) = text[index..].chars().next() else {
            break;
        };
        if ch != '$' {
            index += ch.len_utf8();
            continue;
        }
        let name_start = index + ch.len_utf8();
        let name_end = variable_name_end(text, name_start);
        if name_end > name_start {
            names.push(text[index..name_end].to_string());
        }
        index = name_end.max(index + ch.len_utf8());
    }
    names.sort();
    names.dedup();
    names
}

fn variable_name_end(text: &str, mut index: usize) -> usize {
    while index < text.len() {
        let Some(ch) = text[index..].chars().next() else {
            break;
        };
        if !(ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-')) {
            break;
        }
        index += ch.len_utf8();
    }
    index
}

fn next_non_trivia_token_index(tokens: &[LexedToken], mut index: usize) -> Option<usize> {
    while tokens
        .get(index)
        .is_some_and(|token| is_trivia_token(token.kind))
    {
        index += 1;
    }
    (index < tokens.len()).then_some(index)
}

fn declaration_end_token_index(tokens: &[LexedToken], mut index: usize) -> Option<usize> {
    let mut paren_depth = 0usize;
    while index < tokens.len() {
        match tokens[index].kind {
            SyntaxKind::LeftParen => paren_depth += 1,
            SyntaxKind::RightParen => paren_depth = paren_depth.checked_sub(1)?,
            SyntaxKind::Semicolon | SyntaxKind::SassOptionalSemicolon if paren_depth == 0 => {
                return Some(index);
            }
            _ => {}
        }
        index += 1;
    }
    None
}

const fn is_trivia_token(kind: SyntaxKind) -> bool {
    matches!(
        kind,
        SyntaxKind::Whitespace
            | SyntaxKind::LineComment
            | SyntaxKind::BlockComment
            | SyntaxKind::ScssSilentComment
            | SyntaxKind::SassIndentedNewline
    )
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

    #[test]
    fn control_flow_value_analysis_uses_single_abstract_css_value_domain() {
        let source = "$enabled: 1; @if $enabled { .on { color: green; } } @for $i from 1 through 3 { .n { order: $i; } }";
        let report = analyze_scss_control_flow_values(source, StyleDialect::Scss);
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.mode, "oracleOnly");
        assert_eq!(report.value_type, "AbstractCssValueV0");
        assert!(!report.flat_css_cfg_built);
        assert!(!report.merged_cross_file_graph);
        assert!(report.converged);
        assert_eq!(report.block_count, 2);
        assert_eq!(report.back_edge_count, 1);
        assert_eq!(report.loop_carried_binding_count, 1);
        assert_eq!(report.widened_to_top_count, 0);
        assert_eq!(report.blocks[0].transfer_kind, "branchCondition");
        assert_eq!(report.blocks[1].transfer_kind, "loopCarriedBindings");
        assert_eq!(report.blocks[1].loop_carried_bindings, vec!["$i"]);
        assert_eq!(report.blocks[1].loop_carried_binding_values.len(), 1);
        assert_eq!(report.blocks[1].loop_carried_binding_values[0].name, "$i");
        assert_eq!(
            report.blocks[1].loop_carried_binding_values[0].value_kind,
            "finiteSet"
        );
        assert_eq!(report.blocks[1].output_value_kind, "finiteSet");
    }

    #[test]
    fn control_flow_value_analysis_keeps_dynamic_each_loop_top() {
        let source = "@each $key, $value in $tokens { .item { color: $value; } }";
        let report = analyze_scss_control_flow_values(source, StyleDialect::Scss);
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.block_count, 1);
        assert_eq!(report.back_edge_count, 1);
        assert_eq!(
            report.blocks[0].loop_carried_bindings,
            vec!["$key", "$value"]
        );
        assert!(
            report.blocks[0]
                .loop_carried_binding_values
                .iter()
                .all(|binding| binding.value_kind == "top")
        );
        assert_eq!(report.blocks[0].output_value_kind, "top");
    }

    #[test]
    fn control_flow_value_analysis_tracks_static_each_binding_values() {
        let source = "@each $tone in red, blue { .item { color: $tone; } }";
        let report = analyze_scss_control_flow_values(source, StyleDialect::Scss);
        assert!(report.is_some());
        let Some(report) = report else {
            return;
        };

        assert_eq!(report.block_count, 1);
        assert_eq!(report.blocks[0].loop_carried_bindings, vec!["$tone"]);
        assert_eq!(report.blocks[0].loop_carried_binding_values.len(), 1);
        assert_eq!(
            report.blocks[0].loop_carried_binding_values[0].value_kind,
            "finiteSet"
        );
        assert_eq!(report.blocks[0].output_value_kind, "finiteSet");
    }

    #[test]
    fn control_flow_value_analysis_does_not_build_flat_css_cfg() {
        assert!(
            analyze_scss_control_flow_values(".button { color: red; }", StyleDialect::Css)
                .is_none()
        );
    }
}
