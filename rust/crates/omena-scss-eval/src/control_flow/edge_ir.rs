use std::collections::BTreeMap;

use omena_abstract_value::ControlFlowEdgeGraphV0;
use omena_parser::{StyleDialect, lex};

use super::{
    blocks::{control_flow_block_from_token, scss_else_if_header_condition},
    dialect_label,
    model::{
        OmenaScssEvalControlFlowBlockIdV0, OmenaScssEvalControlFlowBlockV0,
        OmenaScssEvalControlFlowEdgeV0, OmenaScssEvalControlFlowGraphBlockV0,
        OmenaScssEvalControlFlowGraphV0,
    },
};

/// Build the transient per-region SCSS control-flow edge IR.
///
/// The graph records explicit outcome edges for value-flow pruning. It does not
/// rewrite source bytes by itself and does not merge cross-file control flow.
pub fn build_scss_control_flow_graph(
    source: &str,
    dialect: StyleDialect,
) -> Option<OmenaScssEvalControlFlowGraphV0> {
    if !matches!(
        dialect,
        StyleDialect::Css | StyleDialect::Scss | StyleDialect::Sass
    ) {
        return None;
    }
    let lexed = lex(source, dialect);
    let tokens = lexed.tokens();
    let blocks = tokens
        .iter()
        .enumerate()
        .filter_map(|(index, token)| {
            control_flow_block_from_token(source, tokens, index, token, dialect)
        })
        .collect::<Vec<_>>();
    if dialect == StyleDialect::Css && blocks.is_empty() {
        return None;
    }
    let graph_blocks = blocks
        .iter()
        .enumerate()
        .map(|(index, block)| OmenaScssEvalControlFlowGraphBlockV0 {
            id: OmenaScssEvalControlFlowBlockIdV0(index as u32),
            node_key: block.node_key.clone(),
            block: block.clone(),
        })
        .collect::<Vec<_>>();
    let edges = blocks
        .iter()
        .enumerate()
        .flat_map(|(index, _block)| {
            block_control_flow_edges(
                OmenaScssEvalControlFlowBlockIdV0(index as u32),
                index,
                &blocks,
            )
        })
        .collect::<Vec<_>>();
    let edge_count = edges
        .iter()
        .filter(|edge| edge.target_block_id.is_some())
        .count();

    Some(OmenaScssEvalControlFlowGraphV0 {
        schema_version: "0",
        product: "omena-scss-eval.control-flow-edge-ir",
        mode: "oracleOnly",
        dialect: dialect_label(dialect),
        block_id_type: "u32",
        node_key_type: "StableNodeKeyV0",
        flat_css_cfg_built: true,
        merged_cross_file_graph: false,
        block_count: graph_blocks.len(),
        edge_count,
        outcome_count: edges.len(),
        blocks: graph_blocks,
        edges,
    })
}

fn block_control_flow_edges(
    source_block_id: OmenaScssEvalControlFlowBlockIdV0,
    index: usize,
    blocks: &[OmenaScssEvalControlFlowBlockV0],
) -> Vec<OmenaScssEvalControlFlowEdgeV0> {
    let Some(block) = blocks.get(index) else {
        return Vec::new();
    };
    let next_block = blocks.get(index + 1);
    let next_block_id = next_block.map(|_| OmenaScssEvalControlFlowBlockIdV0((index + 1) as u32));
    match block.kind {
        "branchIf" => branch_edges(
            source_block_id,
            next_block_id,
            branch_target_kind(block, next_block),
        ),
        "branchElse" if scss_else_if_header_condition(block.header_text.as_str()).is_some() => {
            branch_edges(
                source_block_id,
                next_block_id,
                branch_target_kind(block, next_block),
            )
        }
        "branchElse" => vec![control_flow_edge(
            source_block_id,
            if next_block_is_inside_block(block, next_block) {
                "then"
            } else {
                "fallthrough"
            },
            next_block_id,
        )],
        "loop" => vec![
            control_flow_edge(source_block_id, "body", Some(source_block_id)),
            control_flow_edge(source_block_id, "fallthrough", next_block_id),
        ],
        _ => next_block_id
            .map(|target| {
                vec![control_flow_edge(
                    source_block_id,
                    "fallthrough",
                    Some(target),
                )]
            })
            .unwrap_or_default(),
    }
}

fn branch_edges(
    source_block_id: OmenaScssEvalControlFlowBlockIdV0,
    next_block_id: Option<OmenaScssEvalControlFlowBlockIdV0>,
    target_kind: &'static str,
) -> Vec<OmenaScssEvalControlFlowEdgeV0> {
    ["then", "else", "fallthrough"]
        .into_iter()
        .map(|outcome| {
            control_flow_edge(
                source_block_id,
                outcome,
                (outcome == target_kind).then_some(next_block_id).flatten(),
            )
        })
        .collect()
}

fn branch_target_kind(
    block: &OmenaScssEvalControlFlowBlockV0,
    next_block: Option<&OmenaScssEvalControlFlowBlockV0>,
) -> &'static str {
    if next_block_is_inside_block(block, next_block) {
        "then"
    } else if next_block.is_some_and(|next| next.at_rule_name.eq_ignore_ascii_case("@else")) {
        "else"
    } else {
        "fallthrough"
    }
}

fn next_block_is_inside_block(
    block: &OmenaScssEvalControlFlowBlockV0,
    next_block: Option<&OmenaScssEvalControlFlowBlockV0>,
) -> bool {
    next_block.is_some_and(|next| {
        block.source_span_start < next.source_span_start
            && next.source_span_start < block.source_span_end
    })
}

fn control_flow_edge(
    source_block_id: OmenaScssEvalControlFlowBlockIdV0,
    outcome: &'static str,
    target_block_id: Option<OmenaScssEvalControlFlowBlockIdV0>,
) -> OmenaScssEvalControlFlowEdgeV0 {
    OmenaScssEvalControlFlowEdgeV0 {
        source_block_id,
        outcome,
        target_block_id,
    }
}

impl ControlFlowEdgeGraphV0 for OmenaScssEvalControlFlowGraphV0 {
    type BlockId = OmenaScssEvalControlFlowBlockIdV0;

    fn entry_block_id(&self) -> Option<Self::BlockId> {
        self.blocks.first().map(|block| block.id)
    }

    fn successor_block_ids_by_source(&self) -> Vec<(Self::BlockId, Vec<Self::BlockId>)> {
        let mut successors_by_id = self
            .blocks
            .iter()
            .map(|block| (block.id, Vec::new()))
            .collect::<BTreeMap<_, _>>();
        for edge in &self.edges {
            let successors = successors_by_id.entry(edge.source_block_id).or_default();
            if let Some(target_block_id) = edge.target_block_id {
                successors.push(target_block_id);
            }
        }
        successors_by_id.into_iter().collect()
    }
}
