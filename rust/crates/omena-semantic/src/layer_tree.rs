use std::collections::{BTreeMap, BTreeSet};

use omena_parser::ParsedCst;
use omena_syntax::{SyntaxKind, SyntaxNode, css_keyword};

use crate::{ParserByteSpanV0, StyleLayerBlockBindingV0, StyleLayerOrderNodeV0};

pub(crate) struct LayerOrderFactsV0 {
    pub(crate) order_nodes: Vec<StyleLayerOrderNodeV0>,
    pub(crate) block_bindings: Vec<StyleLayerBlockBindingV0>,
    pub(crate) unresolved_topology_count: usize,
    pub(crate) topology_complete: bool,
}

#[derive(Clone)]
struct LayerBlockDraftV0 {
    context_id: String,
    node: SyntaxNode,
    local_path: Option<String>,
    canonical_name: Option<String>,
}

#[derive(Clone)]
struct LayerNodeDraftV0 {
    canonical_name: String,
    local_name: String,
    parent_name: Option<String>,
    first_source_order: usize,
    implicit_prefix: bool,
}

pub(crate) fn summarize_layer_order_from_cst(_source: &str, cst: &ParsedCst) -> LayerOrderFactsV0 {
    let mut all_context_order = 0usize;
    let mut blocks = Vec::<LayerBlockDraftV0>::new();
    for node in cst.root().descendants().filter(|node| {
        matches!(
            node.kind(),
            SyntaxKind::LayerRule | SyntaxKind::ContainerRule | SyntaxKind::ScopeRule
        ) && node_has_block(node)
    }) {
        if node.kind() == SyntaxKind::LayerRule {
            let names = layer_names(node);
            blocks.push(LayerBlockDraftV0 {
                context_id: format!("layer:{all_context_order}"),
                node: node.clone(),
                local_path: (names.len() == 1).then(|| names[0].clone()),
                canonical_name: None,
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
        let parent_name = parent.and_then(|parent| blocks[parent].canonical_name.as_deref());
        let Some(local_path) = blocks[index].local_path.as_deref() else {
            unresolved_topology_count = unresolved_topology_count.saturating_add(1);
            continue;
        };
        if parent.is_some() && parent_name.is_none() {
            unresolved_topology_count = unresolved_topology_count.saturating_add(1);
            continue;
        }
        blocks[index].canonical_name = canonical_layer_path(parent_name, local_path);
        if blocks[index].canonical_name.is_none() {
            unresolved_topology_count = unresolved_topology_count.saturating_add(1);
        }
    }

    let mut events = cst
        .root()
        .descendants()
        .filter(|node| node.kind() == SyntaxKind::LayerRule)
        .collect::<Vec<_>>();
    events.sort_by_key(|node| u32::from(node.text_range().start()) as usize);

    let mut nodes = BTreeMap::<String, LayerNodeDraftV0>::new();
    let mut source_order = 0usize;
    for event in events {
        let parent = nearest_enclosing_block_for_node(event, blocks.as_slice());
        if parent.is_some_and(|block| block.canonical_name.is_none()) {
            unresolved_topology_count = unresolved_topology_count.saturating_add(1);
            continue;
        }
        let parent_name = parent.and_then(|block| block.canonical_name.clone());
        let names = layer_names(event);
        if names.is_empty() || (node_has_block(event) && names.len() != 1) {
            continue;
        }
        for name in names {
            let Some(canonical_name) = canonical_layer_path(parent_name.as_deref(), name.as_str())
            else {
                unresolved_topology_count = unresolved_topology_count.saturating_add(1);
                continue;
            };
            register_layer_path(&mut nodes, canonical_name.as_str(), source_order);
            source_order = source_order.saturating_add(1);
        }
    }

    let ranks = cascade_ranks(nodes.values());
    let mut order_nodes = nodes
        .into_values()
        .map(|node| StyleLayerOrderNodeV0 {
            cascade_rank: ranks
                .get(node.canonical_name.as_str())
                .copied()
                .unwrap_or(0),
            nesting_depth: node.canonical_name.split('.').count().saturating_sub(1),
            canonical_name: node.canonical_name,
            local_name: node.local_name,
            parent_name: node.parent_name,
            first_source_order: node.first_source_order,
            implicit_prefix: node.implicit_prefix,
        })
        .collect::<Vec<_>>();
    order_nodes.sort_by_key(|node| node.cascade_rank);

    let mut block_bindings = blocks
        .iter()
        .filter_map(|block| {
            let canonical_name = block.canonical_name.as_ref()?;
            let range = block.node.text_range();
            Some(StyleLayerBlockBindingV0 {
                context_id: block.context_id.clone(),
                canonical_name: canonical_name.clone(),
                cascade_rank: ranks.get(canonical_name.as_str()).copied().unwrap_or(0),
                nesting_depth: canonical_name.split('.').count().saturating_sub(1),
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
        unresolved_topology_count,
    }
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

fn canonical_layer_path(parent: Option<&str>, local_path: &str) -> Option<String> {
    let local_path = local_path.trim();
    if !plain_layer_path(local_path) {
        return None;
    }
    Some(match parent {
        Some(parent) => format!("{parent}.{local_path}"),
        None => local_path.to_string(),
    })
}

fn plain_layer_path(path: &str) -> bool {
    path.split('.').all(|segment| {
        !segment.is_empty()
            && segment
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
    })
}

fn register_layer_path(
    nodes: &mut BTreeMap<String, LayerNodeDraftV0>,
    canonical_name: &str,
    source_order: usize,
) {
    let segments = canonical_name.split('.').collect::<Vec<_>>();
    for length in 1..=segments.len() {
        let name = segments[..length].join(".");
        let parent_name = (length > 1).then(|| segments[..length - 1].join("."));
        let implicit_prefix = length != segments.len();
        nodes.entry(name.clone()).or_insert(LayerNodeDraftV0 {
            canonical_name: name,
            local_name: segments[length - 1].to_string(),
            parent_name,
            first_source_order: source_order,
            implicit_prefix,
        });
    }
}

fn cascade_ranks<'a>(nodes: impl Iterator<Item = &'a LayerNodeDraftV0>) -> BTreeMap<String, usize> {
    let nodes = nodes
        .map(|node| (node.canonical_name.clone(), node.clone()))
        .collect::<BTreeMap<_, _>>();
    let mut children = BTreeMap::<Option<String>, Vec<String>>::new();
    for node in nodes.values() {
        children
            .entry(node.parent_name.clone())
            .or_default()
            .push(node.canonical_name.clone());
    }
    for names in children.values_mut() {
        names.sort_by_key(|name| {
            nodes
                .get(name)
                .map(|node| (node.first_source_order, node.canonical_name.clone()))
                .unwrap_or((usize::MAX, name.clone()))
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
    parent: Option<&str>,
    children: &BTreeMap<Option<String>, Vec<String>>,
    ordered: &mut Vec<String>,
    visited: &mut BTreeSet<String>,
) {
    let key = parent.map(ToString::to_string);
    let Some(names) = children.get(&key) else {
        return;
    };
    for name in names {
        if !visited.insert(name.clone()) {
            continue;
        }
        append_postorder(Some(name.as_str()), children, ordered, visited);
        ordered.push(name.clone());
    }
}

fn layer_names(node: &SyntaxNode) -> Vec<String> {
    let text = syntax_node_text(node);
    let Some(rest) = css_keyword(text.trim_start()).strip_prefix("@layer") else {
        return Vec::new();
    };
    rest.split(['{', ';', '\n'])
        .next()
        .unwrap_or_default()
        .split(',')
        .filter_map(|name| {
            let name = name.trim();
            (!name.is_empty()).then(|| name.to_string())
        })
        .collect()
}

fn node_has_block(node: &SyntaxNode) -> bool {
    node.descendants_with_tokens()
        .filter_map(|element| element.into_token())
        .any(|token| matches!(token.kind(), SyntaxKind::LeftBrace | SyntaxKind::SassIndent))
}

fn syntax_node_text(node: &SyntaxNode) -> String {
    node.try_resolved()
        .map(|resolved| resolved.text().to_string())
        .unwrap_or_default()
}
