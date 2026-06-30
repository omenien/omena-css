use std::cell::RefCell;

use omena_transform_cst::{
    IrEditRegionV0, IrNodeIdV0, IrNodeKindV0, IrNodeV0, IrTransactionErrorV0, IrTransactionV0,
    TransformIrPrintErrorV0, TransformIrV0, materialize_transform_ir_printed_source,
};

use crate::helpers::source_rewrite::replace_source_ranges;
use crate::{TransformProvenanceMutationSpanV0, TransformStructuralIrTransactionTelemetryV0};

thread_local! {
    static STRUCTURAL_IR_TRANSACTION_TELEMETRY:
        RefCell<TransformStructuralIrTransactionTelemetryV0> = const {
            RefCell::new(TransformStructuralIrTransactionTelemetryV0 {
                transaction_commit_count: 0,
            })
        };
    static STRUCTURAL_IR_TRANSACTION_MUTATION_SPAN_BATCHES:
        RefCell<Vec<Vec<TransformProvenanceMutationSpanV0>>> = const {
            RefCell::new(Vec::new())
        };
}

pub(crate) fn reset_structural_ir_transaction_telemetry() {
    STRUCTURAL_IR_TRANSACTION_TELEMETRY.with(|telemetry| {
        *telemetry.borrow_mut() = TransformStructuralIrTransactionTelemetryV0::default();
    });
    reset_structural_ir_transaction_mutation_span_batches();
}

pub(crate) fn structural_ir_transaction_telemetry_snapshot()
-> TransformStructuralIrTransactionTelemetryV0 {
    STRUCTURAL_IR_TRANSACTION_TELEMETRY.with(|telemetry| *telemetry.borrow())
}

fn record_ir_transaction_commit() {
    STRUCTURAL_IR_TRANSACTION_TELEMETRY.with(|telemetry| {
        let mut telemetry = telemetry.borrow_mut();
        telemetry.transaction_commit_count = telemetry.transaction_commit_count.saturating_add(1);
    });
}

pub(crate) fn reset_structural_ir_transaction_mutation_span_batches() {
    STRUCTURAL_IR_TRANSACTION_MUTATION_SPAN_BATCHES.with(|batches| {
        batches.borrow_mut().clear();
    });
}

pub(crate) fn take_structural_ir_transaction_mutation_span_batches()
-> Vec<Vec<TransformProvenanceMutationSpanV0>> {
    STRUCTURAL_IR_TRANSACTION_MUTATION_SPAN_BATCHES
        .with(|batches| std::mem::take(&mut *batches.borrow_mut()))
}

fn record_ir_transaction_commit_with_spans(
    input_byte_len: usize,
    output_byte_len: usize,
    source_spans: &[(usize, usize)],
) {
    record_ir_transaction_commit();
    let mutation_spans =
        transaction_mutation_span_envelope(input_byte_len, output_byte_len, source_spans);
    STRUCTURAL_IR_TRANSACTION_MUTATION_SPAN_BATCHES.with(|batches| {
        batches.borrow_mut().push(mutation_spans);
    });
}

fn transaction_mutation_span_envelope(
    input_byte_len: usize,
    output_byte_len: usize,
    source_spans: &[(usize, usize)],
) -> Vec<TransformProvenanceMutationSpanV0> {
    if source_spans.is_empty()
        || input_byte_len == output_byte_len && source_spans.iter().all(|(start, end)| start == end)
    {
        return Vec::new();
    }
    let source_span_start = source_spans
        .iter()
        .map(|(start, _)| *start)
        .min()
        .unwrap_or(0);
    let source_span_end = source_spans
        .iter()
        .map(|(_, end)| *end)
        .max()
        .unwrap_or(source_span_start);
    let generated_span_end = if output_byte_len >= input_byte_len {
        source_span_end.saturating_add(output_byte_len - input_byte_len)
    } else {
        source_span_end.saturating_sub(input_byte_len - output_byte_len)
    };

    vec![TransformProvenanceMutationSpanV0 {
        source_span_start,
        source_span_end,
        generated_span_start: source_span_start,
        generated_span_end,
        node_key: None,
    }]
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TransformIrReplacementKindV0 {
    StyleRule,
    AtRule,
    Selector,
    Declaration,
    CustomPropertyDeclaration,
    CustomPropertyReference,
    CssModuleValueDefinition,
    CssModuleValueImportSource,
    CssModuleComposesTarget,
    IcssExportName,
}

impl TransformIrReplacementKindV0 {
    const fn ir_kind(self) -> Option<IrNodeKindV0> {
        match self {
            Self::StyleRule => Some(IrNodeKindV0::StyleRule),
            Self::AtRule => Some(IrNodeKindV0::AtRule),
            Self::Selector => Some(IrNodeKindV0::Selector),
            Self::Declaration => Some(IrNodeKindV0::Declaration),
            Self::CustomPropertyDeclaration
            | Self::CustomPropertyReference
            | Self::CssModuleValueDefinition
            | Self::CssModuleValueImportSource
            | Self::CssModuleComposesTarget
            | Self::IcssExportName => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TransformIrSourceReplacementV0 {
    pub(crate) source_span_start: usize,
    pub(crate) source_span_end: usize,
    pub(crate) replacement: String,
    pub(crate) kind: TransformIrReplacementKindV0,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum TransformIrSourceReplacementErrorV0 {
    MissingNode {
        source_span_start: usize,
        source_span_end: usize,
        kind: TransformIrReplacementKindV0,
        candidate_spans: Vec<(usize, usize)>,
    },
    IncompatibleNodeSpan {
        source_span_start: usize,
        source_span_end: usize,
        node_span_start: usize,
        node_span_end: usize,
    },
    Transaction(IrTransactionErrorV0),
    Print(TransformIrPrintErrorV0),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TransformIrReplacementTargetActionV0 {
    ReplaceNode,
    ReplaceNodeCoveringSpan {
        source_span_start: usize,
        source_span_end: usize,
    },
    DeleteNode,
}

#[derive(Debug, Clone)]
struct TransformIrReplacementTargetV0 {
    node_id: IrNodeIdV0,
    source_span_start: usize,
    source_span_end: usize,
    replacement_source_span_start: usize,
    replacement_source_span_end: usize,
    replacement_text: String,
    canonical_text: String,
    action: TransformIrReplacementTargetActionV0,
}

pub(crate) fn delete_ir_nodes_in_ir(
    ir: &mut TransformIrV0,
    pass_id: &str,
    node_ids: &[IrNodeIdV0],
) -> Result<(String, usize), TransformIrSourceReplacementErrorV0> {
    let mut node_ids = node_ids.to_vec();
    node_ids.sort_unstable();
    node_ids.dedup();

    if node_ids.is_empty() {
        return Ok((ir.source_text().to_string(), 0));
    }

    let input_byte_len = ir.source_byte_len;
    let source_spans = node_ids
        .iter()
        .filter_map(|node_id| {
            ir.nodes
                .get(node_id.index())
                .map(|node| (node.source_span_start, node.source_span_end))
        })
        .collect::<Vec<_>>();
    let edit_region = edit_region_for_node_ids(ir, node_ids.as_slice())?;
    let transaction_result = {
        let mut transaction = IrTransactionV0::new(ir, pass_id, edit_region);
        let mut mutation_error = None;

        for node_id in &node_ids {
            if let Err(error) = transaction.delete_node(*node_id) {
                mutation_error = Some(TransformIrSourceReplacementErrorV0::Transaction(error));
                break;
            }
        }

        if let Some(error) = mutation_error {
            Err(error)
        } else {
            transaction
                .commit()
                .map_err(TransformIrSourceReplacementErrorV0::Transaction)
        }
    };
    transaction_result?;
    let printed_css = materialize_transform_ir_printed_source(ir)
        .map_err(TransformIrSourceReplacementErrorV0::Print)?;
    record_ir_transaction_commit_with_spans(
        input_byte_len,
        printed_css.len(),
        source_spans.as_slice(),
    );

    Ok((printed_css, node_ids.len()))
}

pub(crate) fn replace_ir_nodes_in_ir(
    ir: &mut TransformIrV0,
    pass_id: &str,
    replacements: &[TransformIrSourceReplacementV0],
) -> Result<(String, usize), TransformIrSourceReplacementErrorV0> {
    if replacements.is_empty() {
        return Ok((ir.source_text().to_string(), 0));
    }

    let input_byte_len = ir.source_byte_len;
    let replacements = non_overlapping_replacements(replacements);
    let targets = replacements
        .iter()
        .map(|replacement| exact_replacement_node_target(ir, replacement))
        .collect::<Result<Vec<_>, _>>()?;
    let node_ids = targets
        .iter()
        .map(|target| target.node_id)
        .collect::<Vec<_>>();
    let edit_region = edit_region_for_node_ids(ir, node_ids.as_slice())?;
    let transaction_result = {
        let mut transaction = IrTransactionV0::new(ir, pass_id, edit_region);
        let mut mutation_error = None;

        for target in &targets {
            if let Err(error) =
                transaction.replace_node(target.node_id, target.canonical_text.clone())
            {
                mutation_error = Some(TransformIrSourceReplacementErrorV0::Transaction(error));
                break;
            }
        }

        if let Some(error) = mutation_error {
            Err(error)
        } else {
            transaction
                .commit()
                .map_err(TransformIrSourceReplacementErrorV0::Transaction)
        }
    };
    transaction_result?;
    let printed_css = materialize_transform_ir_printed_source(ir)
        .map_err(TransformIrSourceReplacementErrorV0::Print)?;
    let source_spans = targets
        .iter()
        .map(|target| {
            (
                target.replacement_source_span_start,
                target.replacement_source_span_end,
            )
        })
        .collect::<Vec<_>>();
    record_ir_transaction_commit_with_spans(
        input_byte_len,
        printed_css.len(),
        source_spans.as_slice(),
    );

    Ok((printed_css, targets.len()))
}

pub(crate) fn replace_ir_node_spans_in_ir(
    ir: &mut TransformIrV0,
    pass_id: &str,
    replacements: &[TransformIrSourceReplacementV0],
) -> Result<(String, usize), TransformIrSourceReplacementErrorV0> {
    let source = ir.source_text();
    if replacements.is_empty() {
        return Ok((source.to_string(), 0));
    }

    let replacements = non_overlapping_replacements(replacements);
    let targets = replacements
        .iter()
        .map(|replacement| {
            if replacement.kind.ir_kind().is_none() {
                return Err(TransformIrSourceReplacementErrorV0::MissingNode {
                    source_span_start: replacement.source_span_start,
                    source_span_end: replacement.source_span_end,
                    kind: replacement.kind,
                    candidate_spans: Vec::new(),
                });
            }
            find_replacement_targets(source, ir, replacement)
        })
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();
    let targets = coalesce_repeated_replacement_targets(source, targets.as_slice());
    commit_ir_replacement_targets(ir, pass_id, targets.as_slice(), replacements.len())
}

pub(crate) fn replace_ir_node_with_inserted_nodes_in_ir(
    ir: &mut TransformIrV0,
    pass_id: &str,
    anchor_id: IrNodeIdV0,
    kind: IrNodeKindV0,
    canonical_texts: &[String],
) -> Result<(String, usize), TransformIrSourceReplacementErrorV0> {
    if canonical_texts.is_empty() {
        return Ok((ir.source_text().to_string(), 0));
    }

    let input_byte_len = ir.source_byte_len;
    let source_spans = ir
        .nodes
        .get(anchor_id.index())
        .map(|node| vec![(node.source_span_start, node.source_span_end)])
        .unwrap_or_default();
    let edit_region = edit_region_for_node_ids(ir, &[anchor_id])?;
    let transaction_result = {
        let mut transaction = IrTransactionV0::new(ir, pass_id, edit_region);
        let mut mutation_error = None;

        for canonical_text in canonical_texts {
            if let Err(error) = transaction.insert_before(anchor_id, kind, canonical_text.clone()) {
                mutation_error = Some(TransformIrSourceReplacementErrorV0::Transaction(error));
                break;
            }
        }
        if mutation_error.is_none()
            && let Err(error) = transaction.delete_node(anchor_id)
        {
            mutation_error = Some(TransformIrSourceReplacementErrorV0::Transaction(error));
        }

        if let Some(error) = mutation_error {
            Err(error)
        } else {
            transaction
                .commit()
                .map_err(TransformIrSourceReplacementErrorV0::Transaction)
        }
    };
    transaction_result?;
    let printed_css = materialize_transform_ir_printed_source(ir)
        .map_err(TransformIrSourceReplacementErrorV0::Print)?;
    record_ir_transaction_commit_with_spans(
        input_byte_len,
        printed_css.len(),
        source_spans.as_slice(),
    );

    Ok((printed_css, 1))
}

fn commit_ir_replacement_targets(
    ir: &mut TransformIrV0,
    pass_id: &str,
    targets: &[TransformIrReplacementTargetV0],
    mutation_count: usize,
) -> Result<(String, usize), TransformIrSourceReplacementErrorV0> {
    if targets.is_empty() {
        return Ok((ir.source_text().to_string(), 0));
    }

    let input_byte_len = ir.source_byte_len;
    let source_spans = targets
        .iter()
        .map(|target| {
            (
                target.replacement_source_span_start,
                target.replacement_source_span_end,
            )
        })
        .collect::<Vec<_>>();
    let edit_region = edit_region_for_replacement_targets(ir.source_byte_len, targets);
    let transaction_result = {
        let mut transaction = IrTransactionV0::new(ir, pass_id, edit_region);
        let mut mutation_error = None;

        for target in targets {
            let result = match target.action {
                TransformIrReplacementTargetActionV0::DeleteNode => {
                    transaction.delete_node(target.node_id)
                }
                TransformIrReplacementTargetActionV0::ReplaceNode => {
                    transaction.replace_node(target.node_id, target.canonical_text.clone())
                }
                TransformIrReplacementTargetActionV0::ReplaceNodeCoveringSpan {
                    source_span_start,
                    source_span_end,
                } => transaction.replace_node_covering_span(
                    target.node_id,
                    target.canonical_text.clone(),
                    source_span_start,
                    source_span_end,
                ),
            };
            if let Err(error) = result {
                mutation_error = Some(TransformIrSourceReplacementErrorV0::Transaction(error));
                break;
            }
        }

        if let Some(error) = mutation_error {
            Err(error)
        } else {
            transaction
                .commit()
                .map_err(TransformIrSourceReplacementErrorV0::Transaction)
        }
    };
    transaction_result?;
    let printed_css = materialize_transform_ir_printed_source(ir)
        .map_err(TransformIrSourceReplacementErrorV0::Print)?;
    record_ir_transaction_commit_with_spans(
        input_byte_len,
        printed_css.len(),
        source_spans.as_slice(),
    );

    Ok((printed_css, mutation_count))
}

fn exact_replacement_node_target(
    ir: &TransformIrV0,
    replacement: &TransformIrSourceReplacementV0,
) -> Result<TransformIrReplacementTargetV0, TransformIrSourceReplacementErrorV0> {
    let Some(kind) = replacement.kind.ir_kind() else {
        return Err(TransformIrSourceReplacementErrorV0::MissingNode {
            source_span_start: replacement.source_span_start,
            source_span_end: replacement.source_span_end,
            kind: replacement.kind,
            candidate_spans: Vec::new(),
        });
    };
    ir.nodes
        .iter()
        .find(|node| {
            !node.deleted
                && node.kind == kind
                && node.source_span_start == replacement.source_span_start
                && node.source_span_end == replacement.source_span_end
        })
        .map(|node| TransformIrReplacementTargetV0 {
            node_id: node.node_id,
            source_span_start: node.source_span_start,
            source_span_end: node.source_span_end,
            replacement_source_span_start: replacement.source_span_start,
            replacement_source_span_end: replacement.source_span_end,
            replacement_text: replacement.replacement.clone(),
            canonical_text: replacement.replacement.clone(),
            action: TransformIrReplacementTargetActionV0::ReplaceNode,
        })
        .ok_or_else(|| TransformIrSourceReplacementErrorV0::MissingNode {
            source_span_start: replacement.source_span_start,
            source_span_end: replacement.source_span_end,
            kind: replacement.kind,
            candidate_spans: replacement_node_candidate_spans(ir, replacement.kind),
        })
}

fn edit_region_for_node_ids(
    ir: &TransformIrV0,
    node_ids: &[IrNodeIdV0],
) -> Result<IrEditRegionV0, TransformIrSourceReplacementErrorV0> {
    let mut start = ir.source_byte_len;
    let mut end = 0usize;

    for node_id in node_ids {
        let Some(node) = ir.nodes.get(node_id.index()) else {
            return Err(TransformIrSourceReplacementErrorV0::Transaction(
                IrTransactionErrorV0::UnknownNode {
                    node_index: node_id.index(),
                },
            ));
        };
        start = start.min(node.source_span_start);
        end = end.max(node.source_span_end);
    }

    Ok(IrEditRegionV0 {
        source_span_start: start,
        source_span_end: end,
    })
}

fn non_overlapping_replacements(
    replacements: &[TransformIrSourceReplacementV0],
) -> Vec<TransformIrSourceReplacementV0> {
    let mut replacements = replacements.to_vec();
    replacements.sort_by_key(|replacement| replacement.source_span_start);
    let mut retained = Vec::new();
    let mut cursor = 0;
    for replacement in replacements {
        if replacement.source_span_start >= cursor {
            cursor = replacement.source_span_end;
            retained.push(replacement);
        }
    }
    retained
}

fn find_replacement_targets(
    source: &str,
    ir: &TransformIrV0,
    replacement: &TransformIrSourceReplacementV0,
) -> Result<Vec<TransformIrReplacementTargetV0>, TransformIrSourceReplacementErrorV0> {
    let Some(kind) = replacement.kind.ir_kind() else {
        return Err(TransformIrSourceReplacementErrorV0::MissingNode {
            source_span_start: replacement.source_span_start,
            source_span_end: replacement.source_span_end,
            kind: replacement.kind,
            candidate_spans: Vec::new(),
        });
    };
    let single_node = ir
        .nodes
        .iter()
        .filter(|node| {
            !node.deleted
                && node.kind == kind
                && node.source_span_start == replacement.source_span_start
                && node.source_span_end == replacement.source_span_end
        })
        .min_by_key(|node| node.source_span_len())
        .or_else(|| {
            ir.nodes
                .iter()
                .filter(|node| {
                    !node.deleted
                        && node.kind == kind
                        && node.source_span_start <= replacement.source_span_start
                        && node.source_span_end >= replacement.source_span_end
                })
                .min_by_key(|node| node.source_span_len())
        });
    if let Some(node) = single_node {
        let canonical_text = canonical_text_for_node_span(source, replacement, node)?;
        let action = if canonical_text.is_empty() {
            TransformIrReplacementTargetActionV0::DeleteNode
        } else {
            TransformIrReplacementTargetActionV0::ReplaceNode
        };
        return Ok(vec![TransformIrReplacementTargetV0 {
            node_id: node.node_id,
            source_span_start: node.source_span_start,
            source_span_end: node.source_span_end,
            replacement_source_span_start: replacement.source_span_start,
            replacement_source_span_end: replacement.source_span_end,
            replacement_text: replacement.replacement.clone(),
            canonical_text,
            action,
        }]);
    }

    let covered_nodes = replacement_covered_nodes(ir, replacement);
    let Some(first_node) = covered_nodes.first() else {
        return Err(TransformIrSourceReplacementErrorV0::MissingNode {
            source_span_start: replacement.source_span_start,
            source_span_end: replacement.source_span_end,
            kind: replacement.kind,
            candidate_spans: replacement_node_candidate_spans(ir, replacement.kind),
        });
    };
    if covered_nodes
        .iter()
        .any(|node| node.parent != first_node.parent)
    {
        return Err(TransformIrSourceReplacementErrorV0::IncompatibleNodeSpan {
            source_span_start: replacement.source_span_start,
            source_span_end: replacement.source_span_end,
            node_span_start: first_node.source_span_start,
            node_span_end: first_node.source_span_end,
        });
    }
    if covered_nodes.len() == 1 {
        let canonical_text = canonical_text_for_node_span(source, replacement, first_node)?;
        let action = if canonical_text.is_empty() {
            TransformIrReplacementTargetActionV0::DeleteNode
        } else {
            TransformIrReplacementTargetActionV0::ReplaceNode
        };
        return Ok(vec![TransformIrReplacementTargetV0 {
            node_id: first_node.node_id,
            source_span_start: first_node.source_span_start,
            source_span_end: first_node.source_span_end,
            replacement_source_span_start: replacement.source_span_start,
            replacement_source_span_end: replacement.source_span_end,
            replacement_text: replacement.replacement.clone(),
            canonical_text,
            action,
        }]);
    }

    let mut targets = Vec::new();
    if replacement.replacement.is_empty() {
        targets.push(TransformIrReplacementTargetV0 {
            node_id: first_node.node_id,
            source_span_start: replacement.source_span_start,
            source_span_end: replacement.source_span_end,
            replacement_source_span_start: replacement.source_span_start,
            replacement_source_span_end: replacement.source_span_end,
            replacement_text: replacement.replacement.clone(),
            canonical_text: String::new(),
            action: TransformIrReplacementTargetActionV0::ReplaceNodeCoveringSpan {
                source_span_start: replacement.source_span_start,
                source_span_end: replacement.source_span_end,
            },
        });
    } else {
        targets.push(TransformIrReplacementTargetV0 {
            node_id: first_node.node_id,
            source_span_start: replacement.source_span_start,
            source_span_end: replacement.source_span_end,
            replacement_source_span_start: replacement.source_span_start,
            replacement_source_span_end: replacement.source_span_end,
            replacement_text: replacement.replacement.clone(),
            canonical_text: replacement.replacement.clone(),
            action: TransformIrReplacementTargetActionV0::ReplaceNodeCoveringSpan {
                source_span_start: replacement.source_span_start,
                source_span_end: replacement.source_span_end,
            },
        });
    }
    targets.extend(
        covered_nodes
            .iter()
            .skip(1)
            .map(|node| TransformIrReplacementTargetV0 {
                node_id: node.node_id,
                source_span_start: node.source_span_start,
                source_span_end: node.source_span_end,
                replacement_source_span_start: node.source_span_start,
                replacement_source_span_end: node.source_span_end,
                replacement_text: String::new(),
                canonical_text: String::new(),
                action: TransformIrReplacementTargetActionV0::DeleteNode,
            }),
    );
    Ok(targets)
}

fn coalesce_repeated_replacement_targets(
    source: &str,
    targets: &[TransformIrReplacementTargetV0],
) -> Vec<TransformIrReplacementTargetV0> {
    let root_indexes = targets
        .iter()
        .enumerate()
        .map(|(index, target)| coalescing_root_index(targets, index, target))
        .collect::<Vec<_>>();
    let mut retained = Vec::new();
    let mut consumed = vec![false; targets.len()];

    for index in 0..targets.len() {
        if consumed[index] {
            continue;
        }
        let root_index = root_indexes[index];
        if root_index != index {
            continue;
        }
        let target = &targets[root_index];
        let coalesced_indexes = targets
            .iter()
            .enumerate()
            .filter(|(candidate_index, _)| {
                !consumed[*candidate_index] && root_indexes[*candidate_index] == root_index
            })
            .map(|(candidate_index, _)| candidate_index)
            .collect::<Vec<_>>();

        if coalesced_indexes.len() <= 1
            || !coalesced_indexes.iter().all(|candidate_index| {
                matches!(
                    targets[*candidate_index].action,
                    TransformIrReplacementTargetActionV0::ReplaceNode
                )
            })
        {
            retained.push(target.clone());
            consumed[index] = true;
            continue;
        }

        let node_start = target.source_span_start;
        let node_end = target.source_span_end;
        let Some(node_source) = source.get(node_start..node_end) else {
            retained.push(target.clone());
            consumed[index] = true;
            continue;
        };
        if !coalesced_indexes.iter().all(|candidate_index| {
            let candidate = &targets[*candidate_index];
            node_start <= candidate.replacement_source_span_start
                && candidate.replacement_source_span_start <= candidate.replacement_source_span_end
                && candidate.replacement_source_span_end <= node_end
        }) {
            retained.push(target.clone());
            consumed[index] = true;
            continue;
        }
        let ranges = coalesced_indexes
            .iter()
            .map(|candidate_index| {
                let candidate = &targets[*candidate_index];
                (
                    candidate.replacement_source_span_start - node_start,
                    candidate.replacement_source_span_end - node_start,
                    candidate.replacement_text.clone(),
                )
            })
            .collect::<Vec<_>>();
        let (canonical_text, _) = replace_source_ranges(node_source, &ranges);
        retained.push(TransformIrReplacementTargetV0 {
            node_id: target.node_id,
            source_span_start: node_start,
            source_span_end: node_end,
            replacement_source_span_start: node_start,
            replacement_source_span_end: node_end,
            replacement_text: canonical_text.clone(),
            canonical_text,
            action: TransformIrReplacementTargetActionV0::ReplaceNode,
        });
        for candidate_index in coalesced_indexes {
            consumed[candidate_index] = true;
        }
    }

    retained
}

fn coalescing_root_index(
    targets: &[TransformIrReplacementTargetV0],
    index: usize,
    target: &TransformIrReplacementTargetV0,
) -> usize {
    targets
        .iter()
        .enumerate()
        .filter(|(_, candidate)| {
            matches!(
                candidate.action,
                TransformIrReplacementTargetActionV0::ReplaceNode
            ) && candidate.source_span_start <= target.replacement_source_span_start
                && target.replacement_source_span_end <= candidate.source_span_end
        })
        .max_by_key(|(candidate_index, candidate)| {
            (
                candidate
                    .source_span_end
                    .saturating_sub(candidate.source_span_start),
                std::cmp::Reverse(*candidate_index),
            )
        })
        .map_or(index, |(candidate_index, _)| candidate_index)
}

fn canonical_text_for_node_span(
    source: &str,
    replacement: &TransformIrSourceReplacementV0,
    node: &omena_transform_cst::IrNodeV0,
) -> Result<String, TransformIrSourceReplacementErrorV0> {
    if replacement.replacement.is_empty()
        && replacement.source_span_start <= node.source_span_start
        && node.source_span_end <= replacement.source_span_end
    {
        return Ok(String::new());
    }

    if node.source_span_start <= replacement.source_span_start
        && replacement.source_span_end <= node.source_span_end
    {
        let replacement_prefix = &source[node.source_span_start..replacement.source_span_start];
        let replacement_suffix = &source[replacement.source_span_end..node.source_span_end];
        return Ok(format!(
            "{replacement_prefix}{}{replacement_suffix}",
            replacement.replacement
        ));
    }

    if node.source_span_start < replacement.source_span_start
        || node.source_span_end > replacement.source_span_end
    {
        return Err(TransformIrSourceReplacementErrorV0::IncompatibleNodeSpan {
            source_span_start: replacement.source_span_start,
            source_span_end: replacement.source_span_end,
            node_span_start: node.source_span_start,
            node_span_end: node.source_span_end,
        });
    }
    let replacement_prefix = &source[replacement.source_span_start..node.source_span_start];
    let replacement_suffix = &source[node.source_span_end..replacement.source_span_end];
    let Some(canonical_text) = replacement.replacement.strip_prefix(replacement_prefix) else {
        return Err(TransformIrSourceReplacementErrorV0::IncompatibleNodeSpan {
            source_span_start: replacement.source_span_start,
            source_span_end: replacement.source_span_end,
            node_span_start: node.source_span_start,
            node_span_end: node.source_span_end,
        });
    };
    let Some(canonical_text) = canonical_text.strip_suffix(replacement_suffix) else {
        return Err(TransformIrSourceReplacementErrorV0::IncompatibleNodeSpan {
            source_span_start: replacement.source_span_start,
            source_span_end: replacement.source_span_end,
            node_span_start: node.source_span_start,
            node_span_end: node.source_span_end,
        });
    };
    Ok(canonical_text.to_string())
}

fn replacement_node_candidate_spans(
    ir: &TransformIrV0,
    kind: TransformIrReplacementKindV0,
) -> Vec<(usize, usize)> {
    let Some(kind) = kind.ir_kind() else {
        return Vec::new();
    };
    ir.nodes
        .iter()
        .filter(|node| !node.deleted && node.kind == kind)
        .map(|node| (node.source_span_start, node.source_span_end))
        .collect()
}

fn replacement_covered_nodes<'ir>(
    ir: &'ir TransformIrV0,
    replacement: &TransformIrSourceReplacementV0,
) -> Vec<&'ir IrNodeV0> {
    let Some(kind) = replacement.kind.ir_kind() else {
        return Vec::new();
    };
    let mut nodes = ir
        .nodes
        .iter()
        .filter(|node| {
            !node.deleted
                && node.kind == kind
                && replacement.source_span_start <= node.source_span_start
                && node.source_span_end <= replacement.source_span_end
        })
        .collect::<Vec<_>>();
    nodes.sort_by_key(|node| {
        (
            node.source_span_start,
            node.source_span_end,
            node.global_order,
        )
    });
    nodes
        .into_iter()
        .filter(|node| {
            !ir.nodes.iter().any(|candidate| {
                !candidate.deleted
                    && candidate.kind == kind
                    && candidate.node_id != node.node_id
                    && replacement.source_span_start <= candidate.source_span_start
                    && candidate.source_span_end <= replacement.source_span_end
                    && candidate.source_span_start <= node.source_span_start
                    && node.source_span_end <= candidate.source_span_end
                    && candidate.source_span_len() > node.source_span_len()
            })
        })
        .collect()
}

fn edit_region_for_replacement_targets(
    source_byte_len: usize,
    replacement_targets: &[TransformIrReplacementTargetV0],
) -> IrEditRegionV0 {
    let Some(first) = replacement_targets.first() else {
        return IrEditRegionV0::full(source_byte_len);
    };
    let source_span_start = first.source_span_start;
    let source_span_end = replacement_targets
        .iter()
        .map(|target| target.source_span_end)
        .max()
        .unwrap_or(source_byte_len);
    IrEditRegionV0 {
        source_span_start,
        source_span_end,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use omena_parser::StyleDialect;
    use omena_transform_cst::{
        IrTransactionV0, lower_transform_ir_from_source, print_transform_ir_css,
    };

    #[test]
    fn mixed_style_rule_and_declaration_replacements_coalesce_on_same_node() -> Result<(), String> {
        let source = ".button { composes: base utility global(reset); color: red; }";
        let mut ir =
            lower_transform_ir_from_source(source, StyleDialect::Css, "class-hash-composes");
        let selector_end = source
            .find('{')
            .ok_or_else(|| "fixture should contain a rule block".to_string())?;
        let composes_start = source
            .find("composes")
            .ok_or_else(|| "fixture should contain composes".to_string())?;
        let composes_end = composes_start
            + source[composes_start..]
                .find(';')
                .ok_or_else(|| "fixture should terminate composes".to_string())?
            + 1;

        let (output, mutation_count) = replace_ir_node_spans_in_ir(
            &mut ir,
            "css-modules-class-hashing",
            &[
                TransformIrSourceReplacementV0 {
                    source_span_start: 0,
                    source_span_end: selector_end,
                    replacement: "._button_x".to_string(),
                    kind: TransformIrReplacementKindV0::StyleRule,
                },
                TransformIrSourceReplacementV0 {
                    source_span_start: composes_start,
                    source_span_end: composes_end,
                    replacement: "composes: _base_x _utility_x global(reset);".to_string(),
                    kind: TransformIrReplacementKindV0::Declaration,
                },
            ],
        )
        .map_err(|err| format!("mixed replacements should transact: {err:?}"))?;

        assert_eq!(mutation_count, 2);
        assert_eq!(
            output,
            "._button_x{ composes: _base_x _utility_x global(reset); color: red; }"
        );
        assert_eq!(ir.source_text(), output);
        Ok(())
    }

    #[test]
    fn ir_transaction_records_single_batch_mutation_span_envelope() -> Result<(), String> {
        reset_structural_ir_transaction_mutation_span_batches();
        let source = ".button { color: red; }";
        let mut ir = lower_transform_ir_from_source(source, StyleDialect::Css, "span-envelope");
        let selector_end = source
            .find('{')
            .ok_or_else(|| "fixture should contain a rule block".to_string())?;

        let (output, mutation_count) = replace_ir_node_spans_in_ir(
            &mut ir,
            "css-modules-class-hashing",
            &[TransformIrSourceReplacementV0 {
                source_span_start: 0,
                source_span_end: selector_end,
                replacement: "._button_x".to_string(),
                kind: TransformIrReplacementKindV0::StyleRule,
            }],
        )
        .map_err(|err| format!("selector replacement should transact: {err:?}"))?;
        let batches = take_structural_ir_transaction_mutation_span_batches();

        assert_eq!(mutation_count, 1);
        assert_eq!(output, "._button_x{ color: red; }");
        assert_eq!(batches.len(), 1);
        assert_eq!(batches[0].len(), 1);
        assert_eq!(batches[0][0].source_span_start, 0);
        assert_eq!(batches[0][0].source_span_end, selector_end);
        assert_eq!(batches[0][0].generated_span_start, 0);
        assert_eq!(
            batches[0][0].generated_span_end,
            selector_end + output.len() - source.len()
        );
        Ok(())
    }

    #[test]
    fn nested_style_rule_replacements_coalesce_on_mutated_ancestor() -> Result<(), String> {
        let source = ":local { .button { color: maroon; } }";
        let mut ir = lower_transform_ir_from_source(source, StyleDialect::Css, "local-wrapper");
        let inner_selector_start = source
            .find(".button")
            .ok_or_else(|| "fixture should contain an inner rule".to_string())?;
        let inner_selector_end = inner_selector_start + ".button ".len();
        let wrapper_suffix_start = source
            .rfind('}')
            .ok_or_else(|| "fixture should close the local wrapper".to_string())?;

        let (output, mutation_count) = replace_ir_node_spans_in_ir(
            &mut ir,
            "css-modules-class-hashing",
            &[
                TransformIrSourceReplacementV0 {
                    source_span_start: 0,
                    source_span_end: inner_selector_start,
                    replacement: String::new(),
                    kind: TransformIrReplacementKindV0::StyleRule,
                },
                TransformIrSourceReplacementV0 {
                    source_span_start: inner_selector_start,
                    source_span_end: inner_selector_end,
                    replacement: "._button_x".to_string(),
                    kind: TransformIrReplacementKindV0::StyleRule,
                },
                TransformIrSourceReplacementV0 {
                    source_span_start: wrapper_suffix_start,
                    source_span_end: source.len(),
                    replacement: String::new(),
                    kind: TransformIrReplacementKindV0::StyleRule,
                },
            ],
        )
        .map_err(|err| format!("nested replacements should transact: {err:?}"))?;

        assert_eq!(mutation_count, 3);
        assert_eq!(output, "._button_x{ color: maroon; } ");
        assert_eq!(ir.source_text(), output);
        Ok(())
    }

    #[test]
    fn style_rule_replacement_targets_less_rule_after_mixin_declaration() -> Result<(), String> {
        let source = ".space() when (isnumber($margin)) { padding: $margin; } .button { .space(); margin: 2px; }";
        let ir = lower_transform_ir_from_source(source, StyleDialect::Less, "less-mixin-rule");
        let start = source
            .find(".button")
            .ok_or_else(|| "fixture should contain ordinary Less rule".to_string())?;
        let end = start + ".button ".len();
        let replacement = TransformIrSourceReplacementV0 {
            source_span_start: start,
            source_span_end: end,
            replacement: "._button_x".to_string(),
            kind: TransformIrReplacementKindV0::StyleRule,
        };

        let targets = find_replacement_targets(source, &ir, &replacement)
            .map_err(|err| format!("Less rule selector should map to an IR target: {err:?}"))?;

        assert_eq!(targets.len(), 1);
        let target = &targets[0];
        let node = &ir.nodes[target.node_id.index()];
        assert_eq!(node.kind, IrNodeKindV0::StyleRule);
        assert!(source[node.source_span_start..node.source_span_end].starts_with(".button"));
        assert!(target.canonical_text.starts_with("._button_x"));
        Ok(())
    }

    #[test]
    fn style_rule_replacement_commits_less_rule_after_mixin_declaration() -> Result<(), String> {
        let source = ".space() when (isnumber($margin)) { padding: $margin; } .button { .space(); margin: 2px; }";
        let mut ir = lower_transform_ir_from_source(source, StyleDialect::Less, "less-mixin-rule");
        let start = source
            .find(".button")
            .ok_or_else(|| "fixture should contain ordinary Less rule".to_string())?;
        let end = start + ".button ".len();
        let replacement = TransformIrSourceReplacementV0 {
            source_span_start: start,
            source_span_end: end,
            replacement: "._button_x".to_string(),
            kind: TransformIrReplacementKindV0::StyleRule,
        };
        let targets = find_replacement_targets(source, &ir, &replacement)
            .map_err(|err| format!("Less rule selector should map to an IR target: {err:?}"))?;
        let region = edit_region_for_replacement_targets(source.len(), targets.as_slice());
        let mut transaction = IrTransactionV0::new(&mut ir, "css-modules-class-hashing", region);
        for target in targets {
            transaction
                .replace_node(target.node_id, target.canonical_text)
                .map_err(|err| format!("Less rule selector replacement should apply: {err:?}"))?;
        }

        transaction
            .commit()
            .map_err(|err| format!("Less rule selector replacement should commit: {err:?}"))?;
        assert!(
            print_transform_ir_css(&ir)
                .map_err(|err| format!("Less rule selector replacement should print: {err:?}"))?
                .contains("._button_x")
        );
        Ok(())
    }
}
