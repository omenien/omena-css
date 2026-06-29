use omena_parser::{StyleDialect, TypedCstNode, parse_only};
use serde::Serialize;
use std::collections::BTreeSet;

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
    pub deleted: bool,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformIrParseErrorSpanV0 {
    pub source_span_start: usize,
    pub source_span_end: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IrEditRegionV0 {
    pub source_span_start: usize,
    pub source_span_end: usize,
}

impl IrEditRegionV0 {
    pub const fn full(source_byte_len: usize) -> Self {
        Self {
            source_span_start: 0,
            source_span_end: source_byte_len,
        }
    }

    pub const fn contains_span(self, source_span_start: usize, source_span_end: usize) -> bool {
        self.source_span_start <= source_span_start && source_span_end <= self.source_span_end
    }
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
    pub parse_error_spans: Vec<TransformIrParseErrorSpanV0>,
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
                && !node.deleted
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
    CannotMaterializeParseErrorSpans {
        parser_error_count: usize,
    },
    MissingRenderedSpan {
        node_index: usize,
    },
    UnprojectableDirtyChild {
        node_index: usize,
        child_index: usize,
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum IrTransactionValidationErrorV0 {
    DanglingNode {
        node_index: usize,
        dangling_node_index: usize,
    },
    ParentChildLinkMismatch {
        node_index: usize,
        parent_index: usize,
    },
    DeclarationWithoutRuleOwner {
        node_index: usize,
    },
    DuplicateGlobalOrder {
        global_order: usize,
    },
    MissingProvenance {
        node_index: usize,
        origin_index: usize,
    },
    EditOutsideDeclaredRegion {
        node_index: usize,
        region: IrEditRegionV0,
    },
    EditInsideParseErrorRegion {
        node_index: usize,
        parse_error_span: TransformIrParseErrorSpanV0,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum IrTransactionErrorV0 {
    UnknownNode {
        node_index: usize,
    },
    InvalidSourceSpan {
        node_index: usize,
        source_span_start: usize,
        source_span_end: usize,
    },
    NodeKindMismatch {
        node_index: usize,
        expected: IrNodeKindV0,
        actual: IrNodeKindV0,
    },
    Validation(IrTransactionValidationErrorV0),
}

pub struct IrTransactionV0<'ir> {
    ir: &'ir mut TransformIrV0,
    working: TransformIrV0,
    pass_id: String,
    declared_region: IrEditRegionV0,
    changed_node_ids: Vec<IrNodeIdV0>,
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
            deleted: false,
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
        parse_error_spans: parse
            .errors()
            .iter()
            .map(|error| TransformIrParseErrorSpanV0 {
                source_span_start: error.range.start().into(),
                source_span_end: error.range.end().into(),
            })
            .collect(),
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
    let mut cursor = 0;
    for node_id in sorted_root_nodes(ir) {
        let node = &ir.nodes[node_id.index()];
        if node.source_span_start > cursor {
            output.push_str(source_slice(
                ir,
                node.node_id.index(),
                cursor,
                node.source_span_start,
            )?);
        }
        output.push_str(render_node_css(ir, node.node_id)?.as_str());
        cursor = cursor.max(node.source_span_end);
    }
    if cursor < ir.source_text.len() {
        output.push_str(source_slice(ir, 0, cursor, ir.source_text.len())?);
    }
    Ok(output)
}

pub fn materialize_transform_ir_printed_source(
    ir: &mut TransformIrV0,
) -> Result<String, TransformIrPrintErrorV0> {
    if ir.parser_error_count > 0 || !ir.parse_error_spans.is_empty() {
        return Err(TransformIrPrintErrorV0::CannotMaterializeParseErrorSpans {
            parser_error_count: ir.parser_error_count.max(ir.parse_error_spans.len()),
        });
    }

    let rendered = render_transform_ir_css_with_node_spans(ir)?;
    let printed_css = rendered.css;
    let source_id = ir.source_id.clone();
    let materialized_spans = ir
        .nodes
        .iter()
        .map(|node| {
            rendered
                .node_spans
                .get(node.node_id.index())
                .and_then(|span| *span)
                .ok_or(TransformIrPrintErrorV0::MissingRenderedSpan {
                    node_index: node.node_id.index(),
                })
        })
        .collect::<Result<Vec<_>, _>>()?;
    let mut origins = Vec::with_capacity(ir.nodes.len());

    for node in &mut ir.nodes {
        let (source_span_start, source_span_end) = materialized_spans[node.node_id.index()];
        let origin_index = origins.len();
        origins.push(NodeTextOriginV0::Original {
            source_id: source_id.clone(),
            source_span_start,
            source_span_end,
        });
        node.source_span_start = source_span_start;
        node.source_span_end = source_span_end;
        node.origin_index = origin_index;
        node.dirty = false;
        node.canonical_text = None;
    }

    ir.source_text = printed_css.clone();
    ir.source_byte_len = ir.source_text.len();
    ir.origins = origins;
    ir.parser_error_count = 0;
    ir.parse_error_spans.clear();
    refresh_transform_ir_metadata(ir);
    Ok(printed_css)
}

struct RenderedTransformIrCssV0 {
    css: String,
    node_spans: Vec<Option<(usize, usize)>>,
}

fn render_transform_ir_css_with_node_spans(
    ir: &TransformIrV0,
) -> Result<RenderedTransformIrCssV0, TransformIrPrintErrorV0> {
    validate_node_origins(ir)?;
    let mut css = String::new();
    let mut node_spans = vec![None; ir.nodes.len()];
    let mut cursor = 0;

    for node_id in sorted_root_nodes(ir) {
        let node = &ir.nodes[node_id.index()];
        if node.source_span_start > cursor {
            css.push_str(source_slice(
                ir,
                node.node_id.index(),
                cursor,
                node.source_span_start,
            )?);
        }
        render_node_css_with_spans(ir, node.node_id, &mut css, node_spans.as_mut_slice())?;
        cursor = cursor.max(node.source_span_end);
    }
    if cursor < ir.source_text.len() {
        css.push_str(source_slice(ir, 0, cursor, ir.source_text.len())?);
    }

    Ok(RenderedTransformIrCssV0 { css, node_spans })
}

impl<'ir> IrTransactionV0<'ir> {
    pub fn new(
        ir: &'ir mut TransformIrV0,
        pass_id: impl Into<String>,
        declared_region: IrEditRegionV0,
    ) -> Self {
        Self {
            working: ir.clone(),
            ir,
            pass_id: pass_id.into(),
            declared_region,
            changed_node_ids: Vec::new(),
        }
    }

    pub fn replace_node(
        &mut self,
        node_id: IrNodeIdV0,
        canonical_text: impl Into<String>,
    ) -> Result<(), IrTransactionErrorV0> {
        self.mark_node_synthesized(node_id, canonical_text.into(), false)
    }

    pub fn replace_node_covering_span(
        &mut self,
        node_id: IrNodeIdV0,
        canonical_text: impl Into<String>,
        source_span_start: usize,
        source_span_end: usize,
    ) -> Result<(), IrTransactionErrorV0> {
        self.mark_node_covering_span(
            node_id,
            canonical_text.into(),
            false,
            source_span_start,
            source_span_end,
        )
    }

    pub fn delete_node(&mut self, node_id: IrNodeIdV0) -> Result<(), IrTransactionErrorV0> {
        self.mark_node_synthesized(node_id, String::new(), true)
    }

    pub fn insert_before(
        &mut self,
        anchor_id: IrNodeIdV0,
        kind: IrNodeKindV0,
        canonical_text: impl Into<String>,
    ) -> Result<IrNodeIdV0, IrTransactionErrorV0> {
        let Some(anchor) = self.working.nodes.get(anchor_id.index()).cloned() else {
            return Err(IrTransactionErrorV0::UnknownNode {
                node_index: anchor_id.index(),
            });
        };
        let anchor_order = anchor.global_order;
        for node in &mut self.working.nodes {
            if node.global_order >= anchor_order {
                node.global_order += 1;
            }
        }
        let node_id = IrNodeIdV0(self.working.nodes.len());
        let origin_index = self.push_synthesized_origin([anchor_id]);
        let node = IrNodeV0 {
            node_id,
            kind,
            parent: anchor.parent,
            children: Vec::new(),
            source_span_start: anchor.source_span_start,
            source_span_end: anchor.source_span_start,
            origin_index,
            global_order: anchor_order,
            dirty: true,
            deleted: false,
            canonical_text: Some(canonical_text.into()),
        };
        self.working.nodes.push(node);
        self.insert_node_in_parent(anchor_id, node_id);
        self.changed_node_ids.push(node_id);
        refresh_transform_ir_metadata(&mut self.working);
        Ok(node_id)
    }

    pub fn rewrite_value(
        &mut self,
        node_id: IrNodeIdV0,
        canonical_text: impl Into<String>,
    ) -> Result<(), IrTransactionErrorV0> {
        let Some(node) = self.working.nodes.get(node_id.index()) else {
            return Err(IrTransactionErrorV0::UnknownNode {
                node_index: node_id.index(),
            });
        };
        if node.kind != IrNodeKindV0::Value {
            return Err(IrTransactionErrorV0::NodeKindMismatch {
                node_index: node_id.index(),
                expected: IrNodeKindV0::Value,
                actual: node.kind,
            });
        }
        self.mark_node_synthesized(node_id, canonical_text.into(), false)
    }

    pub fn commit(mut self) -> Result<(), IrTransactionErrorV0> {
        refresh_transform_ir_metadata(&mut self.working);
        validate_transaction_commit(&self.working, &self.changed_node_ids, self.declared_region)
            .map_err(IrTransactionErrorV0::Validation)?;
        *self.ir = self.working;
        Ok(())
    }

    fn mark_node_synthesized(
        &mut self,
        node_id: IrNodeIdV0,
        canonical_text: String,
        deleted: bool,
    ) -> Result<(), IrTransactionErrorV0> {
        if self.working.nodes.get(node_id.index()).is_none() {
            return Err(IrTransactionErrorV0::UnknownNode {
                node_index: node_id.index(),
            });
        }
        let origin_index = self.push_synthesized_origin([node_id]);
        let node = &mut self.working.nodes[node_id.index()];
        node.origin_index = origin_index;
        node.dirty = true;
        node.deleted = deleted;
        node.canonical_text = Some(canonical_text);
        self.changed_node_ids.push(node_id);
        refresh_transform_ir_metadata(&mut self.working);
        Ok(())
    }

    fn mark_node_covering_span(
        &mut self,
        node_id: IrNodeIdV0,
        canonical_text: String,
        deleted: bool,
        source_span_start: usize,
        source_span_end: usize,
    ) -> Result<(), IrTransactionErrorV0> {
        let Some(node) = self.working.nodes.get(node_id.index()) else {
            return Err(IrTransactionErrorV0::UnknownNode {
                node_index: node_id.index(),
            });
        };
        if source_span_start > node.source_span_start
            || source_span_end < node.source_span_end
            || source_span_start > source_span_end
            || source_span_end > self.working.source_text.len()
            || !self.working.source_text.is_char_boundary(source_span_start)
            || !self.working.source_text.is_char_boundary(source_span_end)
        {
            return Err(IrTransactionErrorV0::InvalidSourceSpan {
                node_index: node_id.index(),
                source_span_start,
                source_span_end,
            });
        }
        self.mark_node_synthesized(node_id, canonical_text, deleted)?;
        let node = &mut self.working.nodes[node_id.index()];
        node.source_span_start = source_span_start;
        node.source_span_end = source_span_end;
        refresh_transform_ir_metadata(&mut self.working);
        Ok(())
    }

    fn push_synthesized_origin(
        &mut self,
        parent_node_ids: impl IntoIterator<Item = IrNodeIdV0>,
    ) -> usize {
        let origin_index = self.working.origins.len();
        self.working.origins.push(NodeTextOriginV0::Synthesized {
            pass_id: self.pass_id.clone(),
            parent_node_ids: parent_node_ids.into_iter().collect(),
        });
        origin_index
    }

    fn insert_node_in_parent(&mut self, anchor_id: IrNodeIdV0, node_id: IrNodeIdV0) {
        let parent = self.working.nodes[node_id.index()].parent;
        match parent {
            Some(parent_id) => insert_before_in_list(
                &mut self.working.nodes[parent_id.index()].children,
                anchor_id,
                node_id,
            ),
            None => insert_before_in_list(&mut self.working.root_nodes, anchor_id, node_id),
        }
    }
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

fn refresh_transform_ir_metadata(ir: &mut TransformIrV0) {
    ir.indexes = build_indexes(&ir.nodes);
    ir.original_node_count = ir
        .nodes
        .iter()
        .filter(|node| {
            !node.deleted
                && ir
                    .origins
                    .get(node.origin_index)
                    .is_some_and(NodeTextOriginV0::is_original)
        })
        .count();
    ir.synthesized_node_count = ir
        .nodes
        .iter()
        .filter(|node| {
            !node.deleted
                && ir
                    .origins
                    .get(node.origin_index)
                    .is_some_and(|origin| !origin.is_original())
        })
        .count();
}

fn validate_transaction_commit(
    ir: &TransformIrV0,
    changed_node_ids: &[IrNodeIdV0],
    declared_region: IrEditRegionV0,
) -> Result<(), IrTransactionValidationErrorV0> {
    validate_no_dangling_nodes(ir)?;
    validate_parent_child_links(ir)?;
    validate_declaration_ownership(ir)?;
    validate_global_order_slots(ir)?;
    validate_provenance(ir)?;
    validate_changed_nodes_inside_region(ir, changed_node_ids, declared_region)?;
    validate_changed_nodes_outside_parse_errors(ir, changed_node_ids)?;
    Ok(())
}

fn validate_no_dangling_nodes(ir: &TransformIrV0) -> Result<(), IrTransactionValidationErrorV0> {
    for node in &ir.nodes {
        if let Some(parent) = node.parent
            && parent.index() >= ir.nodes.len()
        {
            return Err(IrTransactionValidationErrorV0::DanglingNode {
                node_index: node.node_id.index(),
                dangling_node_index: parent.index(),
            });
        }
        for child in &node.children {
            if child.index() >= ir.nodes.len() {
                return Err(IrTransactionValidationErrorV0::DanglingNode {
                    node_index: node.node_id.index(),
                    dangling_node_index: child.index(),
                });
            }
        }
    }
    Ok(())
}

fn validate_parent_child_links(ir: &TransformIrV0) -> Result<(), IrTransactionValidationErrorV0> {
    for node in &ir.nodes {
        if let Some(parent) = node.parent {
            let parent_node = &ir.nodes[parent.index()];
            if parent == node.node_id || !parent_node.children.contains(&node.node_id) {
                return Err(IrTransactionValidationErrorV0::ParentChildLinkMismatch {
                    node_index: node.node_id.index(),
                    parent_index: parent.index(),
                });
            }
        }
        for child in &node.children {
            if ir.nodes[child.index()].parent != Some(node.node_id) {
                return Err(IrTransactionValidationErrorV0::ParentChildLinkMismatch {
                    node_index: child.index(),
                    parent_index: node.node_id.index(),
                });
            }
        }
    }
    Ok(())
}

fn validate_declaration_ownership(
    ir: &TransformIrV0,
) -> Result<(), IrTransactionValidationErrorV0> {
    for node in &ir.nodes {
        if node.deleted || node.kind != IrNodeKindV0::Declaration {
            continue;
        }
        if !has_rule_owner(ir, node)
            && !has_icss_root_declaration_owner(ir, node)
            && !has_less_mixin_declaration_owner(ir, node)
        {
            return Err(
                IrTransactionValidationErrorV0::DeclarationWithoutRuleOwner {
                    node_index: node.node_id.index(),
                },
            );
        }
    }
    Ok(())
}

fn validate_global_order_slots(ir: &TransformIrV0) -> Result<(), IrTransactionValidationErrorV0> {
    let mut seen = BTreeSet::new();
    for node in ir.nodes.iter().filter(|node| !node.deleted) {
        if !seen.insert(node.global_order) {
            return Err(IrTransactionValidationErrorV0::DuplicateGlobalOrder {
                global_order: node.global_order,
            });
        }
    }
    Ok(())
}

fn validate_provenance(ir: &TransformIrV0) -> Result<(), IrTransactionValidationErrorV0> {
    for node in ir.nodes.iter().filter(|node| !node.deleted) {
        let Some(origin) = ir.origins.get(node.origin_index) else {
            return Err(IrTransactionValidationErrorV0::MissingProvenance {
                node_index: node.node_id.index(),
                origin_index: node.origin_index,
            });
        };
        match origin {
            NodeTextOriginV0::Original {
                source_span_start,
                source_span_end,
                ..
            } => {
                if source_slice(
                    ir,
                    node.node_id.index(),
                    *source_span_start,
                    *source_span_end,
                )
                .is_err()
                {
                    return Err(IrTransactionValidationErrorV0::MissingProvenance {
                        node_index: node.node_id.index(),
                        origin_index: node.origin_index,
                    });
                }
            }
            NodeTextOriginV0::Synthesized { .. } => {
                if node.canonical_text.is_none() {
                    return Err(IrTransactionValidationErrorV0::MissingProvenance {
                        node_index: node.node_id.index(),
                        origin_index: node.origin_index,
                    });
                }
            }
        }
    }
    Ok(())
}

fn validate_changed_nodes_inside_region(
    ir: &TransformIrV0,
    changed_node_ids: &[IrNodeIdV0],
    declared_region: IrEditRegionV0,
) -> Result<(), IrTransactionValidationErrorV0> {
    for node_id in changed_node_ids {
        let node = &ir.nodes[node_id.index()];
        if !declared_region.contains_span(node.source_span_start, node.source_span_end) {
            return Err(IrTransactionValidationErrorV0::EditOutsideDeclaredRegion {
                node_index: node.node_id.index(),
                region: declared_region,
            });
        }
    }
    Ok(())
}

fn validate_changed_nodes_outside_parse_errors(
    ir: &TransformIrV0,
    changed_node_ids: &[IrNodeIdV0],
) -> Result<(), IrTransactionValidationErrorV0> {
    for node_id in changed_node_ids {
        let node = &ir.nodes[node_id.index()];
        if let Some(parse_error_span) = ir.parse_error_spans.iter().copied().find(|span| {
            spans_overlap(
                node.source_span_start,
                node.source_span_end,
                span.source_span_start,
                span.source_span_end,
            ) && !changed_node_preserves_parse_error_source(ir, node, *span)
        }) {
            return Err(IrTransactionValidationErrorV0::EditInsideParseErrorRegion {
                node_index: node.node_id.index(),
                parse_error_span,
            });
        }
    }
    Ok(())
}

fn changed_node_preserves_parse_error_source(
    ir: &TransformIrV0,
    node: &IrNodeV0,
    parse_error_span: TransformIrParseErrorSpanV0,
) -> bool {
    if node.deleted
        || parse_error_span.source_span_start < node.source_span_start
        || parse_error_span.source_span_end > node.source_span_end
    {
        return false;
    }

    let Some(canonical_text) = node.canonical_text.as_deref() else {
        return false;
    };
    let Some((context_start, context_end)) = parse_error_context_span(ir, node, parse_error_span)
    else {
        return false;
    };
    let Ok(parse_error_source) = source_slice(ir, node.node_id.index(), context_start, context_end)
    else {
        return false;
    };

    !parse_error_source.is_empty() && canonical_text.contains(parse_error_source)
}

fn parse_error_context_span(
    ir: &TransformIrV0,
    node: &IrNodeV0,
    parse_error_span: TransformIrParseErrorSpanV0,
) -> Option<(usize, usize)> {
    if parse_error_span.source_span_start > parse_error_span.source_span_end
        || parse_error_span.source_span_end > ir.source_text.len()
    {
        return None;
    }
    let bytes = ir.source_text.as_bytes();
    let mut start = parse_error_span.source_span_start;
    let mut end = parse_error_span.source_span_end;

    while start > node.source_span_start && is_parse_error_context_byte(bytes[start - 1]) {
        start -= 1;
    }
    while end < node.source_span_end && is_parse_error_context_byte(bytes[end]) {
        end += 1;
    }

    (start < end).then_some((start, end))
}

const fn is_parse_error_context_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric()
        || matches!(
            byte,
            b'-' | b'_' | b'.' | b'$' | b'#' | b'%' | b'@' | b'/' | b'\\'
        )
}

fn has_rule_owner(ir: &TransformIrV0, node: &IrNodeV0) -> bool {
    let mut parent = node.parent;
    while let Some(parent_id) = parent {
        let parent_node = &ir.nodes[parent_id.index()];
        if matches!(
            parent_node.kind,
            IrNodeKindV0::StyleRule | IrNodeKindV0::AtRule | IrNodeKindV0::Selector
        ) {
            return true;
        }
        parent = parent_node.parent;
    }
    false
}

fn has_icss_root_declaration_owner(ir: &TransformIrV0, node: &IrNodeV0) -> bool {
    if node.parent.is_some() {
        return false;
    }
    let Some(open_brace) = containing_block_open_brace(ir.source_text.as_str(), node) else {
        return false;
    };
    let Some(close_brace_offset) = ir.source_text[node.source_span_end..].find('}') else {
        return false;
    };
    let close_brace = node.source_span_end + close_brace_offset;
    if open_brace >= close_brace {
        return false;
    }
    let prelude_start = ir.source_text[..open_brace]
        .rfind(['}', ';'])
        .map_or(0, |index| index + 1);
    let prelude = ir.source_text[prelude_start..open_brace].trim();
    prelude == ":export" || prelude.starts_with(":import(")
}

fn has_less_mixin_declaration_owner(ir: &TransformIrV0, node: &IrNodeV0) -> bool {
    if ir.dialect != "less" || node.parent.is_some() {
        return false;
    }
    let Some(open_brace) = containing_block_open_brace(ir.source_text.as_str(), node) else {
        return false;
    };
    let Some(close_brace_offset) = ir.source_text[node.source_span_end..].find('}') else {
        return false;
    };
    let close_brace = node.source_span_end + close_brace_offset;
    if open_brace >= close_brace {
        return false;
    }
    let prelude_start = ir.source_text[..open_brace]
        .rfind(['}', ';'])
        .map_or(0, |index| index + 1);
    let prelude = ir.source_text[prelude_start..open_brace].trim();
    less_prelude_is_callable_mixin(prelude)
}

fn containing_block_open_brace(source: &str, node: &IrNodeV0) -> Option<usize> {
    let mut stack = Vec::new();
    for (index, byte) in source[..node.source_span_start].bytes().enumerate() {
        match byte {
            b'{' => stack.push(index),
            b'}' => {
                stack.pop();
            }
            _ => {}
        }
    }
    stack.pop()
}

fn less_prelude_is_callable_mixin(prelude: &str) -> bool {
    let bytes = prelude.as_bytes();
    let mut index = 0;
    while index < bytes.len() && bytes[index].is_ascii_whitespace() {
        index += 1;
    }
    if index >= bytes.len() || !matches!(bytes[index], b'.' | b'#') {
        return false;
    }
    index += 1;
    if index >= bytes.len() {
        return false;
    }
    while index < bytes.len() {
        match bytes[index] {
            byte if byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-') => {
                index += 1;
            }
            b'\\' => {
                index = index.saturating_add(2);
            }
            _ => break,
        }
    }
    while index < bytes.len() && bytes[index].is_ascii_whitespace() {
        index += 1;
    }
    bytes.get(index) == Some(&b'(')
}

fn sorted_root_nodes(ir: &TransformIrV0) -> Vec<IrNodeIdV0> {
    let mut root_nodes = ir.root_nodes.clone();
    root_nodes.sort_by_key(|node_id| {
        let node = &ir.nodes[node_id.index()];
        (node.source_span_start, node.global_order)
    });
    root_nodes
}

fn render_node_css(
    ir: &TransformIrV0,
    node_id: IrNodeIdV0,
) -> Result<String, TransformIrPrintErrorV0> {
    let node = &ir.nodes[node_id.index()];
    if node.deleted {
        return Ok(String::new());
    }
    if node.dirty {
        let Some(canonical_text) = &node.canonical_text else {
            return Err(TransformIrPrintErrorV0::MissingSynthesizedText {
                node_index: node.node_id.index(),
            });
        };
        return render_dirty_node_with_children(ir, node, canonical_text);
    }

    render_original_node_with_children(ir, node)
}

fn render_node_css_with_spans(
    ir: &TransformIrV0,
    node_id: IrNodeIdV0,
    output: &mut String,
    node_spans: &mut [Option<(usize, usize)>],
) -> Result<(), TransformIrPrintErrorV0> {
    let node = &ir.nodes[node_id.index()];
    let rendered_start = output.len();
    if node.deleted {
        node_spans[node_id.index()] = Some((rendered_start, rendered_start));
        return Ok(());
    }

    if node.dirty {
        let Some(canonical_text) = &node.canonical_text else {
            return Err(TransformIrPrintErrorV0::MissingSynthesizedText {
                node_index: node.node_id.index(),
            });
        };
        render_dirty_node_with_children_and_spans(ir, node, canonical_text, output, node_spans)?;
    } else {
        render_original_node_with_children_and_spans(ir, node, output, node_spans)?;
    }

    node_spans[node_id.index()] = Some((rendered_start, output.len()));
    Ok(())
}

struct DirtyNodeTextProjectionV0 {
    original_replacement_start: usize,
    original_replacement_end: usize,
    rendered_replacement_end: usize,
}

fn render_dirty_node_with_children(
    ir: &TransformIrV0,
    node: &IrNodeV0,
    canonical_text: &str,
) -> Result<String, TransformIrPrintErrorV0> {
    let projection = dirty_node_text_projection(ir, node, canonical_text)?;
    let mut output = String::new();
    let mut cursor = 0;
    let mut child_was_composed = false;

    for child_id in sorted_child_nodes(ir, node) {
        if !node_subtree_has_mutation(ir, child_id) {
            continue;
        }
        let child = &ir.nodes[child_id.index()];
        let Some(child_start) = child
            .source_span_start
            .checked_sub(node.source_span_start)
            .and_then(|offset| project_dirty_node_original_offset(&projection, offset))
        else {
            return Ok(canonical_text.to_string());
        };
        let Some(child_end) = child
            .source_span_end
            .checked_sub(node.source_span_start)
            .and_then(|offset| project_dirty_node_original_offset(&projection, offset))
        else {
            return Ok(canonical_text.to_string());
        };
        if child_start < cursor
            || child_end < child_start
            || child_end > canonical_text.len()
            || !canonical_text.is_char_boundary(child_start)
            || !canonical_text.is_char_boundary(child_end)
        {
            return Ok(canonical_text.to_string());
        }
        output.push_str(&canonical_text[cursor..child_start]);
        output.push_str(render_node_css(ir, child_id)?.as_str());
        cursor = child_end;
        child_was_composed = true;
    }

    if !child_was_composed {
        return Ok(canonical_text.to_string());
    }
    output.push_str(&canonical_text[cursor..]);
    Ok(output)
}

fn render_dirty_node_with_children_and_spans(
    ir: &TransformIrV0,
    node: &IrNodeV0,
    canonical_text: &str,
    output: &mut String,
    node_spans: &mut [Option<(usize, usize)>],
) -> Result<(), TransformIrPrintErrorV0> {
    let projection = dirty_node_text_projection(ir, node, canonical_text)?;
    let rendered_start = output.len();
    let mut cursor = 0;
    let mut child_was_composed = false;

    for child_id in sorted_child_nodes(ir, node) {
        let child = &ir.nodes[child_id.index()];
        let Some(child_start) = child
            .source_span_start
            .checked_sub(node.source_span_start)
            .and_then(|offset| project_dirty_node_original_offset(&projection, offset))
        else {
            return Err(TransformIrPrintErrorV0::UnprojectableDirtyChild {
                node_index: node.node_id.index(),
                child_index: child.node_id.index(),
            });
        };
        let Some(child_end) = child
            .source_span_end
            .checked_sub(node.source_span_start)
            .and_then(|offset| project_dirty_node_original_offset(&projection, offset))
        else {
            return Err(TransformIrPrintErrorV0::UnprojectableDirtyChild {
                node_index: node.node_id.index(),
                child_index: child.node_id.index(),
            });
        };
        if child_start < cursor
            || child_end < child_start
            || child_end > canonical_text.len()
            || !canonical_text.is_char_boundary(child_start)
            || !canonical_text.is_char_boundary(child_end)
        {
            return Err(TransformIrPrintErrorV0::UnprojectableDirtyChild {
                node_index: node.node_id.index(),
                child_index: child.node_id.index(),
            });
        }
        output.push_str(&canonical_text[cursor..child_start]);
        if node_subtree_has_mutation(ir, child_id) {
            render_node_css_with_spans(ir, child_id, output, node_spans)?;
        } else {
            output.push_str(&canonical_text[child_start..child_end]);
            assign_projected_original_subtree_spans(
                ir,
                child_id,
                rendered_start,
                &projection,
                node.source_span_start,
                canonical_text,
                node_spans,
            )?;
        }
        cursor = child_end;
        child_was_composed = true;
    }

    if !child_was_composed {
        output.push_str(canonical_text);
        return Ok(());
    }
    output.push_str(&canonical_text[cursor..]);
    Ok(())
}

fn dirty_node_text_projection(
    ir: &TransformIrV0,
    node: &IrNodeV0,
    canonical_text: &str,
) -> Result<DirtyNodeTextProjectionV0, TransformIrPrintErrorV0> {
    let original_text = source_slice(
        ir,
        node.node_id.index(),
        node.source_span_start,
        node.source_span_end,
    )?;
    let common_prefix_len = common_prefix_byte_len(original_text, canonical_text);
    let common_suffix_len =
        common_suffix_byte_len_after_prefix(original_text, canonical_text, common_prefix_len);

    Ok(DirtyNodeTextProjectionV0 {
        original_replacement_start: common_prefix_len,
        original_replacement_end: original_text.len().saturating_sub(common_suffix_len),
        rendered_replacement_end: canonical_text.len().saturating_sub(common_suffix_len),
    })
}

fn project_dirty_node_original_offset(
    projection: &DirtyNodeTextProjectionV0,
    original_offset: usize,
) -> Option<usize> {
    if original_offset <= projection.original_replacement_start {
        return Some(original_offset);
    }
    if original_offset >= projection.original_replacement_end {
        let delta = projection.rendered_replacement_end as isize
            - projection.original_replacement_end as isize;
        return apply_offset_delta(original_offset, delta);
    }
    None
}

fn common_prefix_byte_len(left: &str, right: &str) -> usize {
    let mut byte_len = 0;
    for (left_char, right_char) in left.chars().zip(right.chars()) {
        if left_char != right_char {
            break;
        }
        byte_len += left_char.len_utf8();
    }
    byte_len
}

fn common_suffix_byte_len_after_prefix(left: &str, right: &str, prefix_len: usize) -> usize {
    let mut byte_len = 0;
    for (left_char, right_char) in left[prefix_len..]
        .chars()
        .rev()
        .zip(right[prefix_len..].chars().rev())
    {
        if left_char != right_char {
            break;
        }
        byte_len += left_char.len_utf8();
    }
    byte_len
}

fn apply_offset_delta(offset: usize, delta: isize) -> Option<usize> {
    if delta >= 0 {
        offset.checked_add(delta as usize)
    } else {
        offset.checked_sub((-delta) as usize)
    }
}

fn node_subtree_has_mutation(ir: &TransformIrV0, node_id: IrNodeIdV0) -> bool {
    let node = &ir.nodes[node_id.index()];
    node.deleted
        || node.dirty
        || node
            .children
            .iter()
            .any(|child_id| node_subtree_has_mutation(ir, *child_id))
}

fn render_original_node_with_children(
    ir: &TransformIrV0,
    node: &IrNodeV0,
) -> Result<String, TransformIrPrintErrorV0> {
    let mut output = String::new();
    let mut cursor = node.source_span_start;
    for child_id in sorted_child_nodes(ir, node) {
        let child = &ir.nodes[child_id.index()];
        if child.source_span_start < node.source_span_start
            || child.source_span_end > node.source_span_end
            || child.source_span_start < cursor
        {
            continue;
        }
        output.push_str(source_slice(
            ir,
            node.node_id.index(),
            cursor,
            child.source_span_start,
        )?);
        output.push_str(render_node_css(ir, child_id)?.as_str());
        cursor = child.source_span_end;
    }
    output.push_str(source_slice(
        ir,
        node.node_id.index(),
        cursor,
        node.source_span_end,
    )?);
    Ok(output)
}

fn render_original_node_with_children_and_spans(
    ir: &TransformIrV0,
    node: &IrNodeV0,
    output: &mut String,
    node_spans: &mut [Option<(usize, usize)>],
) -> Result<(), TransformIrPrintErrorV0> {
    let mut cursor = node.source_span_start;
    for child_id in sorted_child_nodes(ir, node) {
        let child = &ir.nodes[child_id.index()];
        if child.source_span_start < node.source_span_start
            || child.source_span_end > node.source_span_end
            || child.source_span_start < cursor
        {
            continue;
        }
        output.push_str(source_slice(
            ir,
            node.node_id.index(),
            cursor,
            child.source_span_start,
        )?);
        render_node_css_with_spans(ir, child_id, output, node_spans)?;
        cursor = child.source_span_end;
    }
    output.push_str(source_slice(
        ir,
        node.node_id.index(),
        cursor,
        node.source_span_end,
    )?);
    Ok(())
}

fn assign_projected_original_subtree_spans(
    ir: &TransformIrV0,
    node_id: IrNodeIdV0,
    rendered_parent_start: usize,
    projection: &DirtyNodeTextProjectionV0,
    original_parent_start: usize,
    canonical_text: &str,
    node_spans: &mut [Option<(usize, usize)>],
) -> Result<(), TransformIrPrintErrorV0> {
    let node = &ir.nodes[node_id.index()];
    let Some(rendered_start) = node
        .source_span_start
        .checked_sub(original_parent_start)
        .and_then(|offset| project_dirty_node_original_offset(projection, offset))
        .and_then(|offset| rendered_parent_start.checked_add(offset))
    else {
        return Err(TransformIrPrintErrorV0::UnprojectableDirtyChild {
            node_index: node_id.index(),
            child_index: node_id.index(),
        });
    };
    let Some(rendered_end) = node
        .source_span_end
        .checked_sub(original_parent_start)
        .and_then(|offset| project_dirty_node_original_offset(projection, offset))
        .and_then(|offset| rendered_parent_start.checked_add(offset))
    else {
        return Err(TransformIrPrintErrorV0::UnprojectableDirtyChild {
            node_index: node_id.index(),
            child_index: node_id.index(),
        });
    };
    if rendered_end < rendered_start
        || rendered_end.saturating_sub(rendered_parent_start) > canonical_text.len()
    {
        return Err(TransformIrPrintErrorV0::UnprojectableDirtyChild {
            node_index: node_id.index(),
            child_index: node_id.index(),
        });
    }
    node_spans[node_id.index()] = Some((rendered_start, rendered_end));

    for child_id in sorted_child_nodes(ir, node) {
        assign_projected_original_subtree_spans(
            ir,
            child_id,
            rendered_parent_start,
            projection,
            original_parent_start,
            canonical_text,
            node_spans,
        )?;
    }
    Ok(())
}

fn sorted_child_nodes(ir: &TransformIrV0, node: &IrNodeV0) -> Vec<IrNodeIdV0> {
    let mut children = node.children.clone();
    children.sort_by_key(|child_id| {
        let child = &ir.nodes[child_id.index()];
        (child.source_span_start, child.global_order)
    });
    children
}

fn insert_before_in_list(list: &mut Vec<IrNodeIdV0>, anchor_id: IrNodeIdV0, node_id: IrNodeIdV0) {
    let insert_index = list
        .iter()
        .position(|candidate| *candidate == anchor_id)
        .unwrap_or(list.len());
    list.insert(insert_index, node_id);
}

const fn spans_overlap(
    left_start: usize,
    left_end: usize,
    right_start: usize,
    right_end: usize,
) -> bool {
    left_start < right_end && right_start < left_end
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
        IrEditRegionV0, IrNodeIdV0, IrNodeKindV0, IrTransactionErrorV0, IrTransactionV0,
        IrTransactionValidationErrorV0, NodeTextOriginV0, TransformIrParseErrorSpanV0,
        TransformIrPrintErrorV0, lower_transform_ir_from_source,
        materialize_transform_ir_printed_source, print_transform_ir_css,
        summarize_transform_ir_identity_round_trip, validate_transaction_commit,
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
    fn ir_transaction_commits_value_rewrite_through_printer() -> Result<(), String> {
        let mut ir =
            lower_transform_ir_from_source(".card { color: red; }", StyleDialect::Css, "rewrite");
        let value_id = first_node_id(&ir, IrNodeKindV0::Value)?;
        let region = IrEditRegionV0::full(ir.source_byte_len);
        let mut transaction = IrTransactionV0::new(&mut ir, "rewrite-value", region);
        transaction
            .rewrite_value(value_id, " blue")
            .map_err(|err| format!("rewrite value should be accepted: {err:?}"))?;
        transaction
            .commit()
            .map_err(|err| format!("transaction should commit: {err:?}"))?;

        assert!(!ir.all_nodes_original());
        assert_eq!(
            print_transform_ir_css(&ir)
                .map_err(|err| format!("mutated IR should print: {err:?}"))?,
            ".card { color: blue; }"
        );
        Ok(())
    }

    #[test]
    fn materialized_transaction_rebases_source_spans_for_next_transaction() -> Result<(), String> {
        let mut ir =
            lower_transform_ir_from_source(".card { color: red; }", StyleDialect::Css, "material");
        let value_id = first_node_id(&ir, IrNodeKindV0::Value)?;
        let region = IrEditRegionV0::full(ir.source_byte_len);
        let mut transaction = IrTransactionV0::new(&mut ir, "rewrite-value", region);
        transaction
            .rewrite_value(value_id, " blue")
            .map_err(|err| format!("first rewrite should be accepted: {err:?}"))?;
        transaction
            .commit()
            .map_err(|err| format!("first transaction should commit: {err:?}"))?;

        let printed = materialize_transform_ir_printed_source(&mut ir)
            .map_err(|err| format!("materialization should succeed: {err:?}"))?;
        assert_eq!(printed, ".card { color: blue; }");
        assert_eq!(ir.source_text(), ".card { color: blue; }");
        assert!(ir.all_nodes_original());
        assert_eq!(ir.synthesized_node_count, 0);
        let value = &ir.nodes[value_id.index()];
        assert_eq!(
            &ir.source_text()[value.source_span_start..value.source_span_end],
            " blue"
        );

        let mut transaction =
            IrTransactionV0::new(&mut ir, "rewrite-value-again", IrEditRegionV0::full(22));
        transaction
            .rewrite_value(value_id, " green")
            .map_err(|err| format!("second rewrite should use materialized spans: {err:?}"))?;
        transaction
            .commit()
            .map_err(|err| format!("second transaction should commit: {err:?}"))?;

        assert_eq!(
            print_transform_ir_css(&ir)
                .map_err(|err| format!("second mutation should print: {err:?}"))?,
            ".card { color: green; }"
        );
        Ok(())
    }

    #[test]
    fn ir_transaction_exposes_replace_delete_and_insert_mutators() -> Result<(), String> {
        let mut ir = lower_transform_ir_from_source(
            ".card { color: red; }\n.tile { color: blue; }",
            StyleDialect::Css,
            "mutators",
        );
        let selector_id = first_node_id(&ir, IrNodeKindV0::Selector)?;
        let rule_id = first_node_id(&ir, IrNodeKindV0::StyleRule)?;
        let region = IrEditRegionV0::full(ir.source_byte_len);
        let mut transaction = IrTransactionV0::new(&mut ir, "mutator-smoke", region);
        transaction
            .replace_node(selector_id, ".panel")
            .map_err(|err| format!("replace node should be accepted: {err:?}"))?;
        transaction
            .insert_before(
                rule_id,
                IrNodeKindV0::StyleRule,
                ".inserted { color: green; }\n",
            )
            .map_err(|err| format!("insert before should be accepted: {err:?}"))?;
        transaction
            .delete_node(rule_id)
            .map_err(|err| format!("delete node should be accepted: {err:?}"))?;
        transaction
            .commit()
            .map_err(|err| format!("transaction should commit: {err:?}"))?;

        assert_eq!(ir.synthesized_node_count, 2);
        assert!(ir.nodes.iter().any(|node| node.deleted));
        Ok(())
    }

    #[test]
    fn ir_transaction_replaces_node_across_consumed_sibling_span() -> Result<(), String> {
        let mut ir = lower_transform_ir_from_source(
            ".a { color: red; } .a { background: blue; }",
            StyleDialect::Css,
            "covering-span",
        );
        let rule_ids = ir
            .nodes
            .iter()
            .filter(|node| node.kind == IrNodeKindV0::StyleRule)
            .map(|node| node.node_id)
            .collect::<Vec<_>>();
        let first_rule = *rule_ids
            .first()
            .ok_or_else(|| "fixture should produce the first rule".to_string())?;
        let second_rule = *rule_ids
            .get(1)
            .ok_or_else(|| "fixture should produce the second rule".to_string())?;
        let span_start = ir.nodes[first_rule.index()].source_span_start;
        let span_end = ir.nodes[second_rule.index()].source_span_end;
        let region = IrEditRegionV0 {
            source_span_start: span_start,
            source_span_end: span_end,
        };
        let mut transaction = IrTransactionV0::new(&mut ir, "rule-merge", region);
        transaction
            .replace_node_covering_span(
                first_rule,
                ".a { color: red; background: blue; }",
                span_start,
                span_end,
            )
            .map_err(|err| format!("covering replacement should be accepted: {err:?}"))?;
        transaction
            .delete_node(second_rule)
            .map_err(|err| format!("covered sibling should be deletable: {err:?}"))?;
        transaction
            .commit()
            .map_err(|err| format!("transaction should commit: {err:?}"))?;

        assert_eq!(
            print_transform_ir_css(&ir)
                .map_err(|err| format!("mutated IR should print: {err:?}"))?,
            ".a { color: red; background: blue; }"
        );
        Ok(())
    }

    #[test]
    fn ir_transaction_prints_dirty_child_inside_dirty_parent_when_spans_project()
    -> Result<(), String> {
        let source = "@scope (.card) { .title { color: red; } }";
        let mut ir = lower_transform_ir_from_source(source, StyleDialect::Css, "nested-dirty");
        let at_rule = first_node_id(&ir, IrNodeKindV0::AtRule)?;
        let nested_rule = ir
            .nodes
            .iter()
            .find(|node| node.kind == IrNodeKindV0::StyleRule && node.parent == Some(at_rule))
            .map(|node| node.node_id)
            .ok_or_else(|| "fixture should expose a nested style rule".to_string())?;
        let region = IrEditRegionV0::full(ir.source_byte_len);
        let mut transaction = IrTransactionV0::new(&mut ir, "nested-dirty", region);
        transaction
            .replace_node(at_rule, "@scope (._card_x) { .title { color: red; } }")
            .map_err(|err| format!("at-rule rewrite should be accepted: {err:?}"))?;
        transaction
            .replace_node(nested_rule, "._title_z{ color: red; }")
            .map_err(|err| format!("nested rule rewrite should be accepted: {err:?}"))?;
        transaction
            .commit()
            .map_err(|err| format!("transaction should commit: {err:?}"))?;

        assert_eq!(
            print_transform_ir_css(&ir)
                .map_err(|err| format!("mutated IR should print: {err:?}"))?,
            "@scope (._card_x) { ._title_z{ color: red; } }"
        );
        Ok(())
    }

    #[test]
    fn materialized_nested_dirty_transaction_rebases_projected_child_span() -> Result<(), String> {
        let source = "@scope (.card) { .title { color: red; } }";
        let mut ir =
            lower_transform_ir_from_source(source, StyleDialect::Css, "nested-materialized");
        let at_rule = first_node_id(&ir, IrNodeKindV0::AtRule)?;
        let nested_rule = ir
            .nodes
            .iter()
            .find(|node| node.kind == IrNodeKindV0::StyleRule && node.parent == Some(at_rule))
            .map(|node| node.node_id)
            .ok_or_else(|| "fixture should expose a nested style rule".to_string())?;
        let mut transaction = IrTransactionV0::new(
            &mut ir,
            "nested-materialized",
            IrEditRegionV0::full(source.len()),
        );
        transaction
            .replace_node(at_rule, "@scope (._card_x) { .title { color: red; } }")
            .map_err(|err| format!("at-rule rewrite should be accepted: {err:?}"))?;
        transaction
            .replace_node(nested_rule, "._title_z{ color: red; }")
            .map_err(|err| format!("nested rule rewrite should be accepted: {err:?}"))?;
        transaction
            .commit()
            .map_err(|err| format!("transaction should commit: {err:?}"))?;

        let printed = materialize_transform_ir_printed_source(&mut ir)
            .map_err(|err| format!("nested materialization should succeed: {err:?}"))?;
        assert_eq!(printed, "@scope (._card_x) { ._title_z{ color: red; } }");
        assert!(ir.all_nodes_original());
        let nested = &ir.nodes[nested_rule.index()];
        assert_eq!(
            &ir.source_text()[nested.source_span_start..nested.source_span_end],
            "._title_z{ color: red; }"
        );
        Ok(())
    }

    #[test]
    fn lower_transform_ir_preserves_less_rule_after_mixin_declaration() -> Result<(), String> {
        let source = ".space() when (isnumber($margin)) { padding: $margin; } .button { .space(); margin: 2px; }";
        let ir = lower_transform_ir_from_source(source, StyleDialect::Less, "less-mixin-rule");
        let button_rule = ir.nodes.iter().find(|node| {
            node.kind == IrNodeKindV0::StyleRule
                && source[node.source_span_start..node.source_span_end].starts_with(".button")
        });

        assert!(
            button_rule.is_some(),
            "ordinary Less rule after mixin declaration should lower as a style-rule node"
        );
        Ok(())
    }

    #[test]
    fn ir_transaction_rejects_dangling_nodes() -> Result<(), String> {
        let mut ir =
            lower_transform_ir_from_source(".card { color: red; }", StyleDialect::Css, "dangling");
        ir.nodes[0].children.push(IrNodeIdV0(usize::MAX));

        let err = validate_transaction_commit(&ir, &[], IrEditRegionV0::full(ir.source_byte_len))
            .err()
            .ok_or_else(|| "dangling child must fail validation".to_string())?;

        assert_eq!(
            err,
            IrTransactionValidationErrorV0::DanglingNode {
                node_index: 0,
                dangling_node_index: usize::MAX,
            }
        );
        Ok(())
    }

    #[test]
    fn ir_transaction_rejects_parent_child_mismatch() -> Result<(), String> {
        let mut ir =
            lower_transform_ir_from_source(".card { color: red; }", StyleDialect::Css, "links");
        let child = first_node_id(&ir, IrNodeKindV0::Declaration)?;
        let parent = ir.nodes[child.index()]
            .parent
            .ok_or_else(|| "declaration should have a parent".to_string())?;
        ir.nodes[parent.index()]
            .children
            .retain(|candidate| *candidate != child);

        let err = validate_transaction_commit(&ir, &[], IrEditRegionV0::full(ir.source_byte_len))
            .err()
            .ok_or_else(|| "parent/child mismatch must fail validation".to_string())?;

        assert_eq!(
            err,
            IrTransactionValidationErrorV0::ParentChildLinkMismatch {
                node_index: child.index(),
                parent_index: parent.index(),
            }
        );
        Ok(())
    }

    #[test]
    fn ir_transaction_rejects_declaration_without_rule_owner() -> Result<(), String> {
        let mut ir =
            lower_transform_ir_from_source(".card { color: red; }", StyleDialect::Css, "owner");
        let declaration = first_node_id(&ir, IrNodeKindV0::Declaration)?;
        if let Some(parent) = ir.nodes[declaration.index()].parent {
            ir.nodes[parent.index()]
                .children
                .retain(|candidate| *candidate != declaration);
        }
        ir.nodes[declaration.index()].parent = None;

        let err = validate_transaction_commit(&ir, &[], IrEditRegionV0::full(ir.source_byte_len))
            .err()
            .ok_or_else(|| "orphan declaration must fail validation".to_string())?;

        assert_eq!(
            err,
            IrTransactionValidationErrorV0::DeclarationWithoutRuleOwner {
                node_index: declaration.index(),
            }
        );
        Ok(())
    }

    #[test]
    fn ir_transaction_accepts_less_mixin_declaration_owned_root_declarations() -> Result<(), String>
    {
        let source = ".space() when (isnumber($margin)) { padding: $margin; }";
        let ir = lower_transform_ir_from_source(source, StyleDialect::Less, "less-mixin-owner");

        validate_transaction_commit(&ir, &[], IrEditRegionV0::full(ir.source_byte_len))
            .map_err(|err| format!("Less mixin declaration contents should be owned: {err:?}"))?;
        Ok(())
    }

    #[test]
    fn ir_transaction_rejects_duplicate_global_order() -> Result<(), String> {
        let mut ir =
            lower_transform_ir_from_source(".card { color: red; }", StyleDialect::Css, "order");
        let duplicate_order = ir.nodes[0].global_order;
        ir.nodes[1].global_order = duplicate_order;

        let err = validate_transaction_commit(&ir, &[], IrEditRegionV0::full(ir.source_byte_len))
            .err()
            .ok_or_else(|| "duplicate global order must fail validation".to_string())?;

        assert_eq!(
            err,
            IrTransactionValidationErrorV0::DuplicateGlobalOrder {
                global_order: duplicate_order,
            }
        );
        Ok(())
    }

    #[test]
    fn ir_transaction_rejects_missing_provenance() -> Result<(), String> {
        let mut ir = lower_transform_ir_from_source(
            ".card { color: red; }",
            StyleDialect::Css,
            "provenance",
        );
        ir.nodes[0].origin_index = usize::MAX;

        let err = validate_transaction_commit(&ir, &[], IrEditRegionV0::full(ir.source_byte_len))
            .err()
            .ok_or_else(|| "missing provenance must fail validation".to_string())?;

        assert_eq!(
            err,
            IrTransactionValidationErrorV0::MissingProvenance {
                node_index: 0,
                origin_index: usize::MAX,
            }
        );
        Ok(())
    }

    #[test]
    fn ir_transaction_rejects_edits_outside_declared_region() -> Result<(), String> {
        let ir =
            lower_transform_ir_from_source(".card { color: red; }", StyleDialect::Css, "region");
        let rule = first_node_id(&ir, IrNodeKindV0::StyleRule)?;
        let region = IrEditRegionV0 {
            source_span_start: ir.source_byte_len,
            source_span_end: ir.source_byte_len,
        };

        let err = validate_transaction_commit(&ir, &[rule], region)
            .err()
            .ok_or_else(|| "outside-region edit must fail validation".to_string())?;

        assert_eq!(
            err,
            IrTransactionValidationErrorV0::EditOutsideDeclaredRegion {
                node_index: rule.index(),
                region,
            }
        );
        Ok(())
    }

    #[test]
    fn ir_transaction_rejects_edits_inside_parse_error_region() -> Result<(), String> {
        let mut ir = lower_transform_ir_from_source(
            ".card { color: red; }",
            StyleDialect::Css,
            "parse-error",
        );
        let rule = first_node_id(&ir, IrNodeKindV0::StyleRule)?;
        let parse_error_span = TransformIrParseErrorSpanV0 {
            source_span_start: ir.nodes[rule.index()].source_span_start,
            source_span_end: ir.nodes[rule.index()].source_span_end,
        };
        ir.parse_error_spans.push(parse_error_span);

        let err =
            validate_transaction_commit(&ir, &[rule], IrEditRegionV0::full(ir.source_byte_len))
                .err()
                .ok_or_else(|| "parse-error edit must fail validation".to_string())?;

        assert_eq!(
            err,
            IrTransactionValidationErrorV0::EditInsideParseErrorRegion {
                node_index: rule.index(),
                parse_error_span,
            }
        );
        Ok(())
    }

    #[test]
    fn ir_transaction_allows_parent_rewrite_when_parse_error_source_is_preserved()
    -> Result<(), String> {
        let source = ".card { color: tokens.$accent; }";
        let mut ir =
            lower_transform_ir_from_source(source, StyleDialect::Scss, "preserved-parse-error");
        if ir.parser_error_count == 0 {
            return Err("fixture must expose a SCSS parse-error token".to_string());
        }
        let rule = first_node_id(&ir, IrNodeKindV0::StyleRule)?;
        let region = IrEditRegionV0::full(ir.source_byte_len);
        let canonical_text = ".card { color: tokens.$accent; background: red; }";

        let mut transaction = IrTransactionV0::new(&mut ir, "preserve-parse-error", region);
        transaction
            .replace_node(rule, canonical_text)
            .map_err(|error| format!("{error:?}"))?;
        transaction.commit().map_err(|error| format!("{error:?}"))?;

        assert_eq!(
            print_transform_ir_css(&ir).map_err(|error| format!("{error:?}"))?,
            canonical_text
        );
        Ok(())
    }

    #[test]
    fn ir_transaction_rejects_parent_rewrite_when_parse_error_source_is_removed()
    -> Result<(), String> {
        let source = ".card { color: tokens.$accent; }";
        let mut ir =
            lower_transform_ir_from_source(source, StyleDialect::Scss, "removed-parse-error");
        let parse_error_span = ir
            .parse_error_spans
            .first()
            .copied()
            .ok_or_else(|| "fixture must expose a SCSS parse-error token".to_string())?;
        let rule = first_node_id(&ir, IrNodeKindV0::StyleRule)?;
        let region = IrEditRegionV0::full(ir.source_byte_len);
        let mut transaction = IrTransactionV0::new(&mut ir, "remove-parse-error", region);
        transaction
            .replace_node(rule, ".card { color: blue; }")
            .map_err(|error| format!("{error:?}"))?;

        let err = transaction
            .commit()
            .err()
            .ok_or_else(|| "parse-error removal must fail validation".to_string())?;

        assert_eq!(
            err,
            IrTransactionErrorV0::Validation(
                IrTransactionValidationErrorV0::EditInsideParseErrorRegion {
                    node_index: rule.index(),
                    parse_error_span,
                }
            )
        );
        Ok(())
    }

    #[test]
    fn ir_transaction_rejects_non_value_rewrite_value() -> Result<(), String> {
        let mut ir =
            lower_transform_ir_from_source(".card { color: red; }", StyleDialect::Css, "kind");
        let rule = first_node_id(&ir, IrNodeKindV0::StyleRule)?;
        let region = IrEditRegionV0::full(ir.source_byte_len);
        let mut transaction = IrTransactionV0::new(&mut ir, "rewrite-value", region);
        let err = transaction
            .rewrite_value(rule, "blue")
            .err()
            .ok_or_else(|| "non-value rewrite must fail".to_string())?;

        assert_eq!(
            err,
            IrTransactionErrorV0::NodeKindMismatch {
                node_index: rule.index(),
                expected: IrNodeKindV0::Value,
                actual: IrNodeKindV0::StyleRule,
            }
        );
        Ok(())
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

    fn first_node_id(ir: &super::TransformIrV0, kind: IrNodeKindV0) -> Result<IrNodeIdV0, String> {
        ir.nodes
            .iter()
            .find(|node| node.kind == kind)
            .map(|node| node.node_id)
            .ok_or_else(|| format!("missing node kind {kind:?}"))
    }
}
