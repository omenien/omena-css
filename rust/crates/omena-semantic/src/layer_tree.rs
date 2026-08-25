use std::collections::{BTreeMap, BTreeSet};

use omena_cascade::LayerOrdinal;
use omena_parser::{ParsedCst, layer_paths_from_cst};
use omena_syntax::{LayerPathV0, SyntaxKind, SyntaxNode};

use crate::{ParserByteSpanV0, StyleLayerBlockBindingV0, StyleLayerIndexV0, StyleLayerOrderNodeV0};

/// Result of resolving a source span against the canonical layer topology.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayerBindingResolutionV0 {
    /// `Some` identifies a layered declaration; `None` is unlayered.
    Resolved(Option<LayerOrdinal>),
    /// The span belongs to an invalid layer rule that the parser discarded.
    DiscardedInvalidRule,
    /// Existing bindings cannot prove a complete layer order.
    TopologyIncomplete { unresolved_count: usize },
}

/// Resolves the innermost layer block containing a source span.
pub fn layer_ordinal_for_byte_span(
    layer_index: &StyleLayerIndexV0,
    span_start: usize,
    span_end: usize,
) -> LayerBindingResolutionV0 {
    if layer_index
        .discarded_block_spans
        .iter()
        .any(|discarded| discarded.start <= span_start && span_end <= discarded.end)
    {
        return LayerBindingResolutionV0::DiscardedInvalidRule;
    }
    if !layer_index.topology_complete {
        return LayerBindingResolutionV0::TopologyIncomplete {
            unresolved_count: layer_index.unresolved_topology_count,
        };
    }

    let ordinal = layer_index
        .block_bindings
        .iter()
        .filter(|binding| {
            binding.byte_span.start <= span_start && span_end <= binding.byte_span.end
        })
        .max_by_key(|binding| binding.nesting_depth)
        .map(|binding| i32::try_from(binding.cascade_rank).unwrap_or(i32::MAX - 1))
        .and_then(LayerOrdinal::new);
    LayerBindingResolutionV0::Resolved(ordinal)
}

pub(crate) struct LayerOrderFactsV0 {
    pub(crate) order_nodes: Vec<StyleLayerOrderNodeV0>,
    pub(crate) block_bindings: Vec<StyleLayerBlockBindingV0>,
    pub(crate) discarded_block_spans: Vec<ParserByteSpanV0>,
    pub(crate) unresolved_topology_count: usize,
    pub(crate) topology_complete: bool,
}

pub(crate) fn invalid_layer_block_spans(cst: &ParsedCst) -> Vec<ParserByteSpanV0> {
    cst.root()
        .descendants()
        .filter(|node| {
            node.kind() == SyntaxKind::LayerRule
                && node_has_block(node)
                && layer_rule_has_invalid_prelude(node)
        })
        .map(|node| {
            let range = node.text_range();
            ParserByteSpanV0 {
                start: u32::from(range.start()) as usize,
                end: u32::from(range.end()) as usize,
            }
        })
        .collect()
}

pub(crate) fn byte_span_is_within_invalid_layer_block(
    byte_span: ParserByteSpanV0,
    invalid_layer_blocks: &[ParserByteSpanV0],
) -> bool {
    invalid_layer_blocks
        .iter()
        .any(|invalid| invalid.start <= byte_span.start && byte_span.end <= invalid.end)
}

fn node_is_within_invalid_layer_block(
    node: &SyntaxNode,
    invalid_layer_blocks: &[ParserByteSpanV0],
) -> bool {
    let range = node.text_range();
    byte_span_is_within_invalid_layer_block(
        ParserByteSpanV0 {
            start: u32::from(range.start()) as usize,
            end: u32::from(range.end()) as usize,
        },
        invalid_layer_blocks,
    )
}

#[derive(Clone)]
struct LayerBlockDraftV0 {
    context_id: String,
    node: SyntaxNode,
    local_path: Option<LayerPathV0>,
    canonical_path: Option<LayerPathV0>,
}

#[derive(Clone)]
struct LayerNodeDraftV0 {
    canonical_path: LayerPathV0,
    parent_path: Option<LayerPathV0>,
    first_source_order: usize,
    implicit_prefix: bool,
}

pub(crate) fn summarize_layer_order_from_cst(source: &str, cst: &ParsedCst) -> LayerOrderFactsV0 {
    let invalid_layer_blocks = invalid_layer_block_spans(cst);
    let mut all_context_order = 0usize;
    let mut blocks = Vec::<LayerBlockDraftV0>::new();
    for node in cst.root().descendants().filter(|node| {
        matches!(
            node.kind(),
            SyntaxKind::LayerRule | SyntaxKind::ContainerRule | SyntaxKind::ScopeRule
        ) && node_has_block(node)
            && !node_is_within_invalid_layer_block(node, &invalid_layer_blocks)
    }) {
        if node.kind() == SyntaxKind::LayerRule {
            let paths = layer_paths_from_cst(source, node);
            blocks.push(LayerBlockDraftV0 {
                context_id: format!("layer:{all_context_order}"),
                node: node.clone(),
                local_path: (paths.len() == 1).then(|| paths[0].clone()),
                canonical_path: None,
            });
        }
        all_context_order = all_context_order.saturating_add(1);
    }

    blocks.sort_by_key(|block| {
        let range = block.node.text_range();
        (
            u32::from(range.start()) as usize,
            usize::MAX.saturating_sub(u32::from(range.end()) as usize),
        )
    });

    let mut unresolved_topology_count = 0usize;
    for index in 0..blocks.len() {
        let parent = nearest_enclosing_block(index, blocks.as_slice());
        let parent_path = parent.and_then(|parent| blocks[parent].canonical_path.as_ref());
        let Some(local_path) = blocks[index].local_path.as_ref() else {
            unresolved_topology_count = unresolved_topology_count.saturating_add(1);
            continue;
        };
        if parent.is_some() && parent_path.is_none() {
            unresolved_topology_count = unresolved_topology_count.saturating_add(1);
            continue;
        }
        blocks[index].canonical_path = Some(canonical_layer_path(parent_path, local_path));
    }

    let mut events = cst
        .root()
        .descendants()
        .filter(|node| {
            node.kind() == SyntaxKind::LayerRule
                && !node_is_within_invalid_layer_block(node, &invalid_layer_blocks)
        })
        .collect::<Vec<_>>();
    events.sort_by_key(|node| u32::from(node.text_range().start()) as usize);

    let mut nodes = BTreeMap::<LayerPathV0, LayerNodeDraftV0>::new();
    let mut source_order = 0usize;
    for event in events {
        if layer_rule_has_invalid_prelude(event) {
            continue;
        }
        let parent = nearest_enclosing_block_for_node(event, blocks.as_slice());
        if parent.is_some_and(|block| block.canonical_path.is_none()) {
            unresolved_topology_count = unresolved_topology_count.saturating_add(1);
            continue;
        }
        let parent_path = parent.and_then(|block| block.canonical_path.as_ref());
        let paths = layer_paths_from_cst(source, event);
        let has_block = node_has_block(event);
        if paths.is_empty() {
            if !has_block {
                unresolved_topology_count = unresolved_topology_count.saturating_add(1);
            }
            continue;
        }
        if has_block && paths.len() != 1 {
            continue;
        }
        for path in paths {
            let canonical_path = canonical_layer_path(parent_path, &path);
            register_layer_path(&mut nodes, &canonical_path, source_order);
            source_order = source_order.saturating_add(1);
        }
    }

    let ranks = cascade_ranks(nodes.values());
    let mut order_nodes = nodes
        .into_values()
        .map(|node| {
            let canonical_name = node.canonical_path.canonical_name();
            StyleLayerOrderNodeV0 {
                cascade_rank: ranks.get(&node.canonical_path).copied().unwrap_or(0),
                nesting_depth: node.canonical_path.nesting_depth(),
                canonical_name,
                local_name: node.canonical_path.local_name().to_string(),
                parent_name: node.parent_path.as_ref().map(LayerPathV0::canonical_name),
                first_source_order: node.first_source_order,
                implicit_prefix: node.implicit_prefix,
            }
        })
        .collect::<Vec<_>>();
    order_nodes.sort_by_key(|node| node.cascade_rank);

    let mut block_bindings = blocks
        .iter()
        .filter_map(|block| {
            let canonical_path = block.canonical_path.as_ref()?;
            let canonical_name = canonical_path.canonical_name();
            let range = block.node.text_range();
            Some(StyleLayerBlockBindingV0 {
                context_id: block.context_id.clone(),
                canonical_name: canonical_name.clone(),
                cascade_rank: ranks.get(canonical_path).copied().unwrap_or(0),
                nesting_depth: canonical_path.nesting_depth(),
                byte_span: ParserByteSpanV0 {
                    start: u32::from(range.start()) as usize,
                    end: u32::from(range.end()) as usize,
                },
            })
        })
        .collect::<Vec<_>>();
    block_bindings.sort_by_key(|binding| (binding.byte_span.start, binding.byte_span.end));

    LayerOrderFactsV0 {
        topology_complete: unresolved_topology_count == 0,
        order_nodes,
        block_bindings,
        discarded_block_spans: invalid_layer_blocks,
        unresolved_topology_count,
    }
}

fn layer_rule_has_invalid_prelude(layer_rule: &SyntaxNode) -> bool {
    layer_rule
        .children()
        .any(|node| node.kind() == SyntaxKind::BogusLayerName)
}

fn nearest_enclosing_block(index: usize, blocks: &[LayerBlockDraftV0]) -> Option<usize> {
    let range = blocks[index].node.text_range();
    blocks
        .iter()
        .enumerate()
        .filter(|(candidate_index, candidate)| {
            *candidate_index != index
                && candidate.node.text_range().start() < range.start()
                && range.end() < candidate.node.text_range().end()
        })
        .min_by_key(|(_, candidate)| {
            u32::from(candidate.node.text_range().end())
                .saturating_sub(u32::from(candidate.node.text_range().start()))
        })
        .map(|(candidate_index, _)| candidate_index)
}

fn nearest_enclosing_block_for_node<'a>(
    node: &SyntaxNode,
    blocks: &'a [LayerBlockDraftV0],
) -> Option<&'a LayerBlockDraftV0> {
    let range = node.text_range();
    blocks
        .iter()
        .filter(|block| {
            block.node.text_range() != range
                && block.node.text_range().start() < range.start()
                && range.end() < block.node.text_range().end()
        })
        .min_by_key(|block| {
            u32::from(block.node.text_range().end())
                .saturating_sub(u32::from(block.node.text_range().start()))
        })
}

fn canonical_layer_path(parent: Option<&LayerPathV0>, local_path: &LayerPathV0) -> LayerPathV0 {
    parent.map_or_else(|| local_path.clone(), |parent| parent.joined(local_path))
}

fn register_layer_path(
    nodes: &mut BTreeMap<LayerPathV0, LayerNodeDraftV0>,
    canonical_path: &LayerPathV0,
    source_order: usize,
) {
    for length in 1..=canonical_path.segments().len() {
        let Some(path) = canonical_path.prefix(length) else {
            continue;
        };
        let parent_path = path.parent();
        let implicit_prefix = length != canonical_path.segments().len();
        nodes.entry(path.clone()).or_insert(LayerNodeDraftV0 {
            canonical_path: path,
            parent_path,
            first_source_order: source_order,
            implicit_prefix,
        });
    }
}

fn cascade_ranks<'a>(
    nodes: impl Iterator<Item = &'a LayerNodeDraftV0>,
) -> BTreeMap<LayerPathV0, usize> {
    let nodes = nodes
        .map(|node| (node.canonical_path.clone(), node.clone()))
        .collect::<BTreeMap<_, _>>();
    let mut children = BTreeMap::<Option<LayerPathV0>, Vec<LayerPathV0>>::new();
    for node in nodes.values() {
        children
            .entry(node.parent_path.clone())
            .or_default()
            .push(node.canonical_path.clone());
    }
    for paths in children.values_mut() {
        paths.sort_by_key(|path| {
            nodes
                .get(path)
                .map(|node| (node.first_source_order, node.canonical_path.clone()))
                .unwrap_or((usize::MAX, path.clone()))
        });
    }

    let mut ordered = Vec::new();
    append_postorder(None, &children, &mut ordered, &mut BTreeSet::new());
    ordered
        .into_iter()
        .enumerate()
        .map(|(rank, name)| (name, rank))
        .collect()
}

fn append_postorder(
    parent: Option<&LayerPathV0>,
    children: &BTreeMap<Option<LayerPathV0>, Vec<LayerPathV0>>,
    ordered: &mut Vec<LayerPathV0>,
    visited: &mut BTreeSet<LayerPathV0>,
) {
    let key = parent.cloned();
    let Some(names) = children.get(&key) else {
        return;
    };
    for name in names {
        if !visited.insert(name.clone()) {
            continue;
        }
        append_postorder(Some(name), children, ordered, visited);
        ordered.push(name.clone());
    }
}

fn node_has_block(node: &SyntaxNode) -> bool {
    node.descendants_with_tokens()
        .filter_map(|element| element.into_token())
        .any(|token| matches!(token.kind(), SyntaxKind::LeftBrace | SyntaxKind::SassIndent))
}
