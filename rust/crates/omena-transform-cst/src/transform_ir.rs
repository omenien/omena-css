use omena_parser::{StyleDialect, TypedCstNode, parse_only};
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(transparent)]
pub struct IrNodeIdV0(pub usize);

impl IrNodeIdV0 {
    pub const fn index(self) -> usize {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum IrNodeKindV0 {
    StyleRule,
    AtRule,
    Declaration,
    Selector,
    Value,
}

impl IrNodeKindV0 {
    pub const fn as_label(self) -> &'static str {
        match self {
            Self::StyleRule => "style-rule",
            Self::AtRule => "at-rule",
            Self::Declaration => "declaration",
            Self::Selector => "selector",
            Self::Value => "value",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum NodeTextOriginV0 {
    Original {
        source_id: String,
        source_span_start: usize,
        source_span_end: usize,
    },
    Synthesized {
        pass_id: String,
        parent_node_ids: Vec<IrNodeIdV0>,
    },
}

impl NodeTextOriginV0 {
    pub const fn is_original(&self) -> bool {
        matches!(self, Self::Original { .. })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IrNodeV0 {
    pub node_id: IrNodeIdV0,
    pub kind: IrNodeKindV0,
    pub parent: Option<IrNodeIdV0>,
    pub children: Vec<IrNodeIdV0>,
    pub source_span_start: usize,
    pub source_span_end: usize,
    pub origin_index: usize,
    pub global_order: usize,
    pub dirty: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub canonical_text: Option<String>,
}

impl IrNodeV0 {
    pub fn source_span_len(&self) -> usize {
        self.source_span_end.saturating_sub(self.source_span_start)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformIrKindIndexV0 {
    pub kind: IrNodeKindV0,
    pub node_ids: Vec<IrNodeIdV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformIrParentIndexV0 {
    pub parent: Option<IrNodeIdV0>,
    pub node_ids: Vec<IrNodeIdV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformIrIndexesV0 {
    pub by_kind: Vec<TransformIrKindIndexV0>,
    pub by_parent: Vec<TransformIrParentIndexV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformIrV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub source_id: String,
    pub dialect: &'static str,
    pub source_byte_len: usize,
    pub parser_error_count: usize,
    pub root_nodes: Vec<IrNodeIdV0>,
    pub nodes: Vec<IrNodeV0>,
    pub origins: Vec<NodeTextOriginV0>,
    pub indexes: TransformIrIndexesV0,
    pub original_node_count: usize,
    pub synthesized_node_count: usize,
    source_text: String,
}

impl TransformIrV0 {
    pub fn all_nodes_original(&self) -> bool {
        self.nodes.iter().all(|node| {
            !node.dirty
                && self
                    .origins
                    .get(node.origin_index)
                    .is_some_and(NodeTextOriginV0::is_original)
        })
    }

    pub fn source_text(&self) -> &str {
        self.source_text.as_str()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum TransformIrPrintErrorV0 {
    MissingNodeOrigin {
        node_index: usize,
    },
    InvalidOriginalSpan {
        node_index: usize,
        source_span_start: usize,
        source_span_end: usize,
        source_byte_len: usize,
    },
    MissingSynthesizedText {
        node_index: usize,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformIrIdentityRoundTripV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub source_id: String,
    pub dialect: &'static str,
    pub source_byte_len: usize,
    pub printed_byte_len: usize,
    pub node_count: usize,
    pub original_node_count: usize,
    pub synthesized_node_count: usize,
    pub parser_error_count: usize,
    pub all_nodes_original: bool,
    pub byte_identical: bool,
    pub printed_css: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct CandidateNodeV0 {
    kind: IrNodeKindV0,
    source_span_start: usize,
    source_span_end: usize,
}

pub fn lower_transform_ir_from_source(
    source: &str,
    dialect: StyleDialect,
    source_id: impl Into<String>,
) -> TransformIrV0 {
    let source_id = source_id.into();
    let parse = parse_only(source, dialect);
    let cst = parse.cst();
    let mut candidates = Vec::new();

    candidates.extend(
        cst.rules()
            .into_iter()
            .map(|node| candidate_from_typed_node(IrNodeKindV0::StyleRule, node)),
    );
    candidates.extend(
        cst.at_rules()
            .into_iter()
            .map(|node| candidate_from_typed_node(IrNodeKindV0::AtRule, node)),
    );
    candidates.extend(
        cst.declarations()
            .into_iter()
            .map(|node| candidate_from_typed_node(IrNodeKindV0::Declaration, node)),
    );
    candidates.extend(
        cst.selectors()
            .into_iter()
            .map(|node| candidate_from_typed_node(IrNodeKindV0::Selector, node)),
    );
    candidates.extend(
        cst.values()
            .into_iter()
            .map(|node| candidate_from_typed_node(IrNodeKindV0::Value, node)),
    );

    candidates.sort_by_key(|candidate| {
        (
            candidate.source_span_start,
            std::cmp::Reverse(candidate.source_span_end),
            kind_order(candidate.kind),
        )
    });
    candidates.dedup();

    let mut nodes = candidates
        .iter()
        .enumerate()
        .map(|(index, candidate)| IrNodeV0 {
            node_id: IrNodeIdV0(index),
            kind: candidate.kind,
            parent: None,
            children: Vec::new(),
            source_span_start: candidate.source_span_start,
            source_span_end: candidate.source_span_end,
            origin_index: index,
            global_order: index,
            dirty: false,
            canonical_text: None,
        })
        .collect::<Vec<_>>();

    let origins = candidates
        .iter()
        .map(|candidate| NodeTextOriginV0::Original {
            source_id: source_id.clone(),
            source_span_start: candidate.source_span_start,
            source_span_end: candidate.source_span_end,
        })
        .collect::<Vec<_>>();

    assign_parent_links(&mut nodes);
    let root_nodes = nodes
        .iter()
        .filter(|node| node.parent.is_none())
        .map(|node| node.node_id)
        .collect::<Vec<_>>();
    let indexes = build_indexes(&nodes);
    let original_node_count = origins.iter().filter(|origin| origin.is_original()).count();

    TransformIrV0 {
        schema_version: "0",
        product: "omena-transform-cst.transform-ir",
        source_id,
        dialect: dialect_label(dialect),
        source_byte_len: source.len(),
        parser_error_count: parse.errors().len(),
        root_nodes,
        nodes,
        origins,
        indexes,
        original_node_count,
        synthesized_node_count: 0,
        source_text: source.to_string(),
    }
}

pub fn print_transform_ir_css(ir: &TransformIrV0) -> Result<String, TransformIrPrintErrorV0> {
    validate_node_origins(ir)?;
    if ir.all_nodes_original() {
        return Ok(ir.source_text.clone());
    }

    let mut output = String::new();
    for node_id in &ir.root_nodes {
        let node = &ir.nodes[node_id.index()];
        match &ir.origins[node.origin_index] {
            NodeTextOriginV0::Original {
                source_span_start,
                source_span_end,
                ..
            } => output.push_str(source_slice(
                ir,
                node.node_id.index(),
                *source_span_start,
                *source_span_end,
            )?),
            NodeTextOriginV0::Synthesized { .. } => {
                let Some(canonical_text) = &node.canonical_text else {
                    return Err(TransformIrPrintErrorV0::MissingSynthesizedText {
                        node_index: node.node_id.index(),
                    });
                };
                output.push_str(canonical_text);
            }
        }
    }
    Ok(output)
}

pub fn summarize_transform_ir_identity_round_trip(
    source: &str,
    dialect: StyleDialect,
    source_id: impl Into<String>,
) -> Result<TransformIrIdentityRoundTripV0, TransformIrPrintErrorV0> {
    let source_id = source_id.into();
    let ir = lower_transform_ir_from_source(source, dialect, source_id.clone());
    let printed_css = print_transform_ir_css(&ir)?;
    Ok(TransformIrIdentityRoundTripV0 {
        schema_version: "0",
        product: "omena-transform-cst.transform-ir-identity-round-trip",
        source_id,
        dialect: dialect_label(dialect),
        source_byte_len: source.len(),
        printed_byte_len: printed_css.len(),
        node_count: ir.nodes.len(),
        original_node_count: ir.original_node_count,
        synthesized_node_count: ir.synthesized_node_count,
        parser_error_count: ir.parser_error_count,
        all_nodes_original: ir.all_nodes_original(),
        byte_identical: printed_css == source,
        printed_css,
    })
}

fn candidate_from_typed_node<T: TypedCstNode>(kind: IrNodeKindV0, node: T) -> CandidateNodeV0 {
    let range = node.text_range();
    CandidateNodeV0 {
        kind,
        source_span_start: range.start().into(),
        source_span_end: range.end().into(),
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

fn assign_parent_links(nodes: &mut [IrNodeV0]) {
    for index in 0..nodes.len() {
        let parent = nearest_parent_index(index, nodes);
        nodes[index].parent = parent.map(IrNodeIdV0);
    }
    for index in 0..nodes.len() {
        if let Some(parent) = nodes[index].parent {
            nodes[parent.index()].children.push(IrNodeIdV0(index));
        }
    }
}

fn nearest_parent_index(index: usize, nodes: &[IrNodeV0]) -> Option<usize> {
    let node = &nodes[index];
    nodes
        .iter()
        .enumerate()
        .filter(|(candidate_index, candidate)| {
            *candidate_index != index
                && candidate.source_span_start <= node.source_span_start
                && candidate.source_span_end >= node.source_span_end
                && candidate.source_span_len() > node.source_span_len()
        })
        .min_by_key(|(_, candidate)| candidate.source_span_len())
        .map(|(candidate_index, _)| candidate_index)
}

fn build_indexes(nodes: &[IrNodeV0]) -> TransformIrIndexesV0 {
    let mut by_kind = Vec::new();
    for kind in [
        IrNodeKindV0::StyleRule,
        IrNodeKindV0::AtRule,
        IrNodeKindV0::Declaration,
        IrNodeKindV0::Selector,
        IrNodeKindV0::Value,
    ] {
        by_kind.push(TransformIrKindIndexV0 {
            kind,
            node_ids: nodes
                .iter()
                .filter(|node| node.kind == kind)
                .map(|node| node.node_id)
                .collect(),
        });
    }

    let mut parents = nodes.iter().map(|node| node.parent).collect::<Vec<_>>();
    parents.sort();
    parents.dedup();
    let by_parent = parents
        .into_iter()
        .map(|parent| TransformIrParentIndexV0 {
            parent,
            node_ids: nodes
                .iter()
                .filter(|node| node.parent == parent)
                .map(|node| node.node_id)
                .collect(),
        })
        .collect();

    TransformIrIndexesV0 { by_kind, by_parent }
}

fn validate_node_origins(ir: &TransformIrV0) -> Result<(), TransformIrPrintErrorV0> {
    for node in &ir.nodes {
        let Some(origin) = ir.origins.get(node.origin_index) else {
            return Err(TransformIrPrintErrorV0::MissingNodeOrigin {
                node_index: node.node_id.index(),
            });
        };
        if let NodeTextOriginV0::Original {
            source_span_start,
            source_span_end,
            ..
        } = origin
        {
            source_slice(
                ir,
                node.node_id.index(),
                *source_span_start,
                *source_span_end,
            )?;
        }
    }
    Ok(())
}

fn source_slice(
    ir: &TransformIrV0,
    node_index: usize,
    source_span_start: usize,
    source_span_end: usize,
) -> Result<&str, TransformIrPrintErrorV0> {
    if source_span_start > source_span_end
        || source_span_end > ir.source_text.len()
        || !ir.source_text.is_char_boundary(source_span_start)
        || !ir.source_text.is_char_boundary(source_span_end)
    {
        return Err(TransformIrPrintErrorV0::InvalidOriginalSpan {
            node_index,
            source_span_start,
            source_span_end,
            source_byte_len: ir.source_text.len(),
        });
    }
    Ok(&ir.source_text[source_span_start..source_span_end])
}

const fn kind_order(kind: IrNodeKindV0) -> u8 {
    match kind {
        IrNodeKindV0::StyleRule => 0,
        IrNodeKindV0::AtRule => 1,
        IrNodeKindV0::Selector => 2,
        IrNodeKindV0::Declaration => 3,
        IrNodeKindV0::Value => 4,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        IrNodeKindV0, NodeTextOriginV0, TransformIrPrintErrorV0, lower_transform_ir_from_source,
        print_transform_ir_css, summarize_transform_ir_identity_round_trip,
    };
    use omena_parser::StyleDialect;

    #[test]
    fn transform_ir_identity_round_trip_keeps_original_origins() -> Result<(), String> {
        let source = r#".card {
  color: red;
}
@media (min-width: 40rem) {
  .card { color: blue; }
}
"#;
        let summary =
            summarize_transform_ir_identity_round_trip(source, StyleDialect::Css, "fixture:card")
                .map_err(|err| format!("round trip should print: {err:?}"))?;

        assert_eq!(
            summary.product,
            "omena-transform-cst.transform-ir-identity-round-trip"
        );
        assert!(summary.byte_identical);
        assert!(summary.all_nodes_original);
        assert_eq!(summary.synthesized_node_count, 0);
        assert_eq!(summary.printed_css, source);
        assert!(summary.node_count >= 5);
        Ok(())
    }

    #[test]
    fn transform_ir_indexes_structural_node_kinds() {
        let ir = lower_transform_ir_from_source(
            ".card { color: red; }\n@supports (display: grid) { .grid { display: grid; } }",
            StyleDialect::Css,
            "fixture:index",
        );

        assert_eq!(ir.product, "omena-transform-cst.transform-ir");
        assert!(ir.all_nodes_original());
        assert_eq!(ir.original_node_count, ir.nodes.len());
        assert!(ir.root_nodes.iter().all(|node_id| {
            ir.nodes[node_id.index()].kind == IrNodeKindV0::StyleRule
                || ir.nodes[node_id.index()].kind == IrNodeKindV0::AtRule
        }));
        assert!(ir.indexes.by_kind.iter().any(|index| {
            index.kind == IrNodeKindV0::Declaration && !index.node_ids.is_empty()
        }));
        assert!(ir.origins.iter().all(NodeTextOriginV0::is_original));
    }

    #[test]
    fn transform_ir_printer_rejects_invalid_original_span() -> Result<(), String> {
        let mut ir =
            lower_transform_ir_from_source(".card { color: red; }", StyleDialect::Css, "bad-span");
        let first_node = ir
            .nodes
            .first()
            .ok_or_else(|| "fixture should produce an IR node".to_string())?;
        let origin_index = first_node.origin_index;
        ir.origins[origin_index] = NodeTextOriginV0::Original {
            source_id: "bad-span".to_string(),
            source_span_start: 0,
            source_span_end: usize::MAX,
        };

        let err = match print_transform_ir_css(&ir) {
            Ok(_) => return Err("invalid original span must fail printing".to_string()),
            Err(err) => err,
        };

        assert_eq!(
            err,
            TransformIrPrintErrorV0::InvalidOriginalSpan {
                node_index: first_node.node_id.index(),
                source_span_start: 0,
                source_span_end: usize::MAX,
                source_byte_len: 21,
            }
        );
        Ok(())
    }
}
