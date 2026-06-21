use std::collections::{BTreeMap, BTreeSet};

use omena_abstract_value::{
    AbstractCssValueV0, ControlFlowEdgeGraphV0, MAX_FLOW_ANALYSIS_ITERATIONS,
    reachable_control_flow_block_ids,
};
use omena_parser::StyleDialect;

use super::{
    analyze_scss_control_flow_values_with_initial_bindings, build_scss_control_flow_graph,
    dialect_label,
    model::{
        OmenaScssEvalControlFlowBlockIdV0, OmenaScssEvalControlFlowEdgeV0,
        OmenaScssEvalControlFlowPruneReachabilityV0,
    },
};

/// Summarize the value-driven prune/reachability loop for a style region.
///
/// Each iteration folds known control-flow truthiness, prunes only proved-dead
/// outcome edges, and recomputes reachability. Unknown values keep their edges
/// so the product path can preserve source bytes when the lattice cannot decide.
pub fn summarize_scss_control_flow_prune_reachability(
    source: &str,
    dialect: StyleDialect,
) -> Option<OmenaScssEvalControlFlowPruneReachabilityV0> {
    summarize_scss_control_flow_prune_reachability_with_initial_bindings(
        source,
        dialect,
        &BTreeMap::new(),
    )
}

pub(crate) fn summarize_scss_control_flow_prune_reachability_with_initial_bindings(
    source: &str,
    dialect: StyleDialect,
    initial_bindings: &BTreeMap<String, AbstractCssValueV0>,
) -> Option<OmenaScssEvalControlFlowPruneReachabilityV0> {
    let graph = build_scss_control_flow_graph(source, dialect)?;
    let block_ids = graph
        .blocks
        .iter()
        .map(|block| block.id)
        .collect::<BTreeSet<_>>();
    let original_reachable_block_ids = reachable_control_flow_block_ids(&graph);
    let original_edge_count = graph
        .edges
        .iter()
        .filter(|edge| edge.target_block_id.is_some())
        .count();

    if graph.blocks.is_empty() {
        return Some(OmenaScssEvalControlFlowPruneReachabilityV0 {
            schema_version: "0",
            product: "omena-scss-eval.control-flow-prune-reachability",
            mode: "oracleOnlyPrunedReachability",
            dialect: dialect_label(dialect),
            block_id_type: "u32",
            node_key_type: "StableNodeKeyV0",
            max_iterations: MAX_FLOW_ANALYSIS_ITERATIONS,
            iteration_count: 0,
            converged: true,
            flat_css_cfg_built: true,
            merged_cross_file_graph: false,
            block_count: 0,
            original_edge_count,
            pruned_edge_count: 0,
            reachable_block_count: 0,
            unreachable_block_count: 0,
            have_terminals_changed: false,
            reachable_block_ids: Vec::new(),
            unreachable_block_ids: Vec::new(),
        });
    }

    let truthiness_by_block_id =
        control_flow_truthiness_by_block_id(source, dialect, initial_bindings)?;
    let mut reachable_block_ids = original_reachable_block_ids.clone();
    let mut pruned_edge_count = 0usize;
    let mut iteration_count = 0usize;
    let mut converged = false;

    for iteration in 1..=MAX_FLOW_ANALYSIS_ITERATIONS {
        iteration_count = iteration;
        let pruned_graph = PrunedScssControlFlowGraph::new(
            &graph.edges,
            &truthiness_by_block_id,
            &reachable_block_ids,
        );
        pruned_edge_count = original_edge_count.saturating_sub(pruned_graph.edge_count());
        let next_reachable_block_ids = reachable_control_flow_block_ids(&pruned_graph);
        if next_reachable_block_ids == reachable_block_ids {
            converged = true;
            break;
        }
        reachable_block_ids = next_reachable_block_ids;
    }

    let unreachable_block_ids = block_ids
        .difference(&reachable_block_ids)
        .copied()
        .collect::<Vec<_>>();
    let reachable_block_ids = reachable_block_ids.into_iter().collect::<Vec<_>>();
    let have_terminals_changed = pruned_edge_count > 0
        || reachable_block_ids != sorted_block_ids(&original_reachable_block_ids);

    Some(OmenaScssEvalControlFlowPruneReachabilityV0 {
        schema_version: "0",
        product: "omena-scss-eval.control-flow-prune-reachability",
        mode: "oracleOnlyPrunedReachability",
        dialect: dialect_label(dialect),
        block_id_type: "u32",
        node_key_type: "StableNodeKeyV0",
        max_iterations: MAX_FLOW_ANALYSIS_ITERATIONS,
        iteration_count,
        converged,
        flat_css_cfg_built: true,
        merged_cross_file_graph: false,
        block_count: graph.blocks.len(),
        original_edge_count,
        pruned_edge_count,
        reachable_block_count: reachable_block_ids.len(),
        unreachable_block_count: unreachable_block_ids.len(),
        have_terminals_changed,
        reachable_block_ids,
        unreachable_block_ids,
    })
}

fn control_flow_truthiness_by_block_id(
    source: &str,
    dialect: StyleDialect,
    initial_bindings: &BTreeMap<String, AbstractCssValueV0>,
) -> Option<BTreeMap<OmenaScssEvalControlFlowBlockIdV0, bool>> {
    let graph = build_scss_control_flow_graph(source, dialect)?;
    let analysis =
        analyze_scss_control_flow_values_with_initial_bindings(source, dialect, initial_bindings)?;
    let block_id_by_node_key = graph
        .blocks
        .iter()
        .map(|block| (block.node_key.as_str().to_string(), block.id))
        .collect::<BTreeMap<_, _>>();

    Some(
        analysis
            .blocks
            .iter()
            .filter_map(|block| {
                let truthy = match block.transfer_truthiness {
                    Some("truthy") => true,
                    Some("falsey") => false,
                    _ => return None,
                };
                block_id_by_node_key
                    .get(block.node_key.as_str())
                    .copied()
                    .map(|block_id| (block_id, truthy))
            })
            .collect(),
    )
}

fn sorted_block_ids(
    block_ids: &BTreeSet<OmenaScssEvalControlFlowBlockIdV0>,
) -> Vec<OmenaScssEvalControlFlowBlockIdV0> {
    block_ids.iter().copied().collect()
}

struct PrunedScssControlFlowGraph {
    successor_ids_by_block_id:
        BTreeMap<OmenaScssEvalControlFlowBlockIdV0, Vec<OmenaScssEvalControlFlowBlockIdV0>>,
}

impl PrunedScssControlFlowGraph {
    fn new(
        edges: &[OmenaScssEvalControlFlowEdgeV0],
        truthiness_by_block_id: &BTreeMap<OmenaScssEvalControlFlowBlockIdV0, bool>,
        reachable_block_ids: &BTreeSet<OmenaScssEvalControlFlowBlockIdV0>,
    ) -> Self {
        let mut successor_ids_by_block_id = BTreeMap::new();
        for edge in edges {
            let Some(target_block_id) = edge.target_block_id else {
                continue;
            };
            successor_ids_by_block_id
                .entry(edge.source_block_id)
                .or_insert_with(Vec::new);
            if !reachable_block_ids.contains(&edge.source_block_id)
                || !control_flow_edge_survives_prune(edge, truthiness_by_block_id)
            {
                continue;
            }
            successor_ids_by_block_id
                .entry(edge.source_block_id)
                .or_insert_with(Vec::new)
                .push(target_block_id);
        }
        Self {
            successor_ids_by_block_id,
        }
    }

    fn edge_count(&self) -> usize {
        self.successor_ids_by_block_id.values().map(Vec::len).sum()
    }
}

impl ControlFlowEdgeGraphV0 for PrunedScssControlFlowGraph {
    type BlockId = OmenaScssEvalControlFlowBlockIdV0;

    fn entry_block_id(&self) -> Option<Self::BlockId> {
        Some(OmenaScssEvalControlFlowBlockIdV0(0))
    }

    fn successor_block_ids_by_source(&self) -> Vec<(Self::BlockId, Vec<Self::BlockId>)> {
        self.successor_ids_by_block_id
            .iter()
            .map(|(block_id, successor_ids)| (*block_id, successor_ids.clone()))
            .collect()
    }
}

fn control_flow_edge_survives_prune(
    edge: &OmenaScssEvalControlFlowEdgeV0,
    truthiness_by_block_id: &BTreeMap<OmenaScssEvalControlFlowBlockIdV0, bool>,
) -> bool {
    let Some(truthy) = truthiness_by_block_id.get(&edge.source_block_id).copied() else {
        return true;
    };
    match edge.outcome {
        "then" | "body" => truthy,
        "else" | "fallthrough" => !truthy,
        _ => true,
    }
}
