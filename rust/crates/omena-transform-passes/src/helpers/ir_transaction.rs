use std::cell::RefCell;

use omena_parser::StyleDialect;
use omena_transform_cst::{
    IrEditRegionV0, IrNodeIdV0, IrNodeKindV0, IrNodeV0, IrTransactionErrorV0, IrTransactionV0,
    StableTransformIrNodeKindV0, StableTransformIrV0, TransformIrPrintErrorV0, TransformIrV0,
    build_stable_transform_ir_from_source, lower_transform_ir_from_source,
    materialize_transform_ir_printed_source, print_transform_ir_css,
};

use crate::TransformStructuralIrTransactionTelemetryV0;
use crate::helpers::source_rewrite::replace_source_ranges;

thread_local! {
    static STRUCTURAL_IR_TRANSACTION_TELEMETRY:
        RefCell<TransformStructuralIrTransactionTelemetryV0> = const {
            RefCell::new(TransformStructuralIrTransactionTelemetryV0 {
                transaction_commit_count: 0,
                source_range_rewrite_fallback_count: 0,
                print_relower_fallback_count: 0,
                tree_shake_class_source_fact_fallback_count: 0,
            })
        };
}

pub(crate) fn reset_structural_ir_transaction_telemetry() {
    STRUCTURAL_IR_TRANSACTION_TELEMETRY.with(|telemetry| {
        *telemetry.borrow_mut() = TransformStructuralIrTransactionTelemetryV0::default();
    });
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

fn record_source_range_rewrite_fallback() {
    STRUCTURAL_IR_TRANSACTION_TELEMETRY.with(|telemetry| {
        let mut telemetry = telemetry.borrow_mut();
        telemetry.source_range_rewrite_fallback_count = telemetry
            .source_range_rewrite_fallback_count
            .saturating_add(1);
    });
}

fn record_print_relower_fallback() {
    STRUCTURAL_IR_TRANSACTION_TELEMETRY.with(|telemetry| {
        let mut telemetry = telemetry.borrow_mut();
        telemetry.print_relower_fallback_count =
            telemetry.print_relower_fallback_count.saturating_add(1);
    });
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TransformIrReplacementKindV0 {
    StyleRule,
    AtRule,
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
            Self::Declaration => Some(IrNodeKindV0::Declaration),
            Self::CustomPropertyDeclaration
            | Self::CustomPropertyReference
            | Self::CssModuleValueDefinition
            | Self::CssModuleValueImportSource
            | Self::CssModuleComposesTarget
            | Self::IcssExportName => None,
        }
    }

    const fn stable_ir_kind(self) -> Option<StableTransformIrNodeKindV0> {
        match self {
            Self::CustomPropertyDeclaration => {
                Some(StableTransformIrNodeKindV0::CustomPropertyDeclaration)
            }
            Self::CustomPropertyReference => {
                Some(StableTransformIrNodeKindV0::CustomPropertyReference)
            }
            Self::CssModuleValueDefinition => {
                Some(StableTransformIrNodeKindV0::CssModuleValueDefinition)
            }
            Self::CssModuleValueImportSource => {
                Some(StableTransformIrNodeKindV0::CssModuleValueImportSource)
            }
            Self::CssModuleComposesTarget => {
                Some(StableTransformIrNodeKindV0::CssModuleComposesTarget)
            }
            Self::IcssExportName => Some(StableTransformIrNodeKindV0::IcssExportName),
            Self::StyleRule | Self::AtRule | Self::Declaration => None,
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

pub(crate) fn apply_ir_source_replacements(
    source: &str,
    dialect: StyleDialect,
    source_id: &str,
    pass_id: &str,
    replacements: &[TransformIrSourceReplacementV0],
) -> Result<(String, usize), TransformIrSourceReplacementErrorV0> {
    let mut ir = lower_transform_ir_from_source(source, dialect, source_id);
    apply_ir_source_replacements_to_ir(&mut ir, dialect, pass_id, replacements)
}

pub(crate) fn apply_ir_source_replacements_to_ir(
    ir: &mut TransformIrV0,
    dialect: StyleDialect,
    pass_id: &str,
    replacements: &[TransformIrSourceReplacementV0],
) -> Result<(String, usize), TransformIrSourceReplacementErrorV0> {
    let source = ir.source_text().to_string();
    if replacements.is_empty() {
        return Ok((source, 0));
    }

    let replacements = non_overlapping_replacements(replacements);
    let stable_ir = replacements
        .iter()
        .any(|replacement| replacement.kind.stable_ir_kind().is_some())
        .then(|| {
            build_stable_transform_ir_from_source(source.as_str(), dialect, ir.source_id.as_str())
        });
    let stable_fact_transaction_candidate =
        stable_ir.is_some() && stable_fact_replacements_can_transact(replacements.as_slice());
    if stable_ir.is_some() && !stable_fact_transaction_candidate {
        let source_id = ir.source_id.clone();
        validate_source_range_replacements(
            source.as_str(),
            dialect,
            source_id.as_str(),
            &replacements,
        )?;
        return apply_source_range_replacements_to_ir(ir, dialect, &replacements);
    }
    let replacement_targets = match replacements
        .iter()
        .map(|replacement| {
            find_replacement_targets(source.as_str(), ir, stable_ir.as_ref(), replacement)
        })
        .collect::<Result<Vec<_>, _>>()
    {
        Ok(targets) => targets,
        Err(error) => {
            if stable_fact_transaction_candidate {
                let source_id = ir.source_id.clone();
                validate_source_range_replacements(
                    source.as_str(),
                    dialect,
                    source_id.as_str(),
                    &replacements,
                )?;
                return apply_source_range_replacements_to_ir(ir, dialect, &replacements);
            }
            return Err(error);
        }
    }
    .into_iter()
    .flatten()
    .collect::<Vec<_>>();
    let replacement_targets =
        coalesce_repeated_replacement_targets(source.as_str(), replacement_targets.as_slice());
    let edit_region = edit_region_for_replacement_targets(source.len(), &replacement_targets);
    let transaction_result = {
        let mut transaction = IrTransactionV0::new(ir, pass_id, edit_region);
        let mut mutation_error = None;

        for target in replacement_targets {
            let result = match target.action {
                TransformIrReplacementTargetActionV0::DeleteNode => {
                    transaction.delete_node(target.node_id)
                }
                TransformIrReplacementTargetActionV0::ReplaceNode => {
                    transaction.replace_node(target.node_id, target.canonical_text)
                }
                TransformIrReplacementTargetActionV0::ReplaceNodeCoveringSpan {
                    source_span_start,
                    source_span_end,
                } => transaction.replace_node_covering_span(
                    target.node_id,
                    target.canonical_text,
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
    if let Err(error) = transaction_result {
        if stable_fact_transaction_candidate {
            let source_id = ir.source_id.clone();
            validate_source_range_replacements(
                source.as_str(),
                dialect,
                source_id.as_str(),
                &replacements,
            )?;
            return apply_source_range_replacements_to_ir(ir, dialect, &replacements);
        }
        return Err(error);
    }
    record_ir_transaction_commit();
    let printed_css = match materialize_transform_ir_printed_source(ir) {
        Ok(printed_css) => printed_css,
        Err(_) => match print_transform_ir_css(ir) {
            Ok(printed_css) => {
                record_print_relower_fallback();
                let source_id = ir.source_id.clone();
                *ir = lower_transform_ir_from_source(printed_css.as_str(), dialect, source_id);
                printed_css
            }
            Err(error) => {
                return Err(TransformIrSourceReplacementErrorV0::Print(error));
            }
        },
    };

    Ok((printed_css, replacements.len()))
}

fn apply_source_range_replacements_to_ir(
    ir: &mut TransformIrV0,
    dialect: StyleDialect,
    replacements: &[TransformIrSourceReplacementV0],
) -> Result<(String, usize), TransformIrSourceReplacementErrorV0> {
    record_source_range_rewrite_fallback();
    let source = ir.source_text().to_string();
    let source_id = ir.source_id.clone();
    let ranges = replacements
        .iter()
        .map(|replacement| {
            (
                replacement.source_span_start,
                replacement.source_span_end,
                replacement.replacement.clone(),
            )
        })
        .collect::<Vec<_>>();
    let (printed_css, mutation_count) = replace_source_ranges(source.as_str(), &ranges);
    *ir = lower_transform_ir_from_source(printed_css.as_str(), dialect, source_id);
    Ok((printed_css, mutation_count))
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

fn stable_fact_replacements_can_transact(replacements: &[TransformIrSourceReplacementV0]) -> bool {
    replacements.iter().all(|replacement| {
        replacement.kind.stable_ir_kind().is_some() || replacement.kind.ir_kind().is_some()
    })
}

fn find_replacement_targets(
    source: &str,
    ir: &TransformIrV0,
    stable_ir: Option<&StableTransformIrV0>,
    replacement: &TransformIrSourceReplacementV0,
) -> Result<Vec<TransformIrReplacementTargetV0>, TransformIrSourceReplacementErrorV0> {
    let Some(kind) = replacement.kind.ir_kind() else {
        return find_stable_fact_replacement_targets(source, ir, stable_ir, replacement);
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

fn find_stable_fact_replacement_targets(
    source: &str,
    ir: &TransformIrV0,
    stable_ir: Option<&StableTransformIrV0>,
    replacement: &TransformIrSourceReplacementV0,
) -> Result<Vec<TransformIrReplacementTargetV0>, TransformIrSourceReplacementErrorV0> {
    let Some(stable_kind) = replacement.kind.stable_ir_kind() else {
        return Err(TransformIrSourceReplacementErrorV0::MissingNode {
            source_span_start: replacement.source_span_start,
            source_span_end: replacement.source_span_end,
            kind: replacement.kind,
            candidate_spans: Vec::new(),
        });
    };
    let Some(stable_ir) = stable_ir else {
        return Err(TransformIrSourceReplacementErrorV0::MissingNode {
            source_span_start: replacement.source_span_start,
            source_span_end: replacement.source_span_end,
            kind: replacement.kind,
            candidate_spans: Vec::new(),
        });
    };
    if !source_span_contains_stable_fact(stable_ir, replacement, stable_kind) {
        return Err(TransformIrSourceReplacementErrorV0::MissingNode {
            source_span_start: replacement.source_span_start,
            source_span_end: replacement.source_span_end,
            kind: replacement.kind,
            candidate_spans: stable_ir
                .nodes
                .iter()
                .filter(|node| node.kind == stable_kind)
                .map(|node| (node.source_span_start, node.source_span_end))
                .collect(),
        });
    }

    let Some(node) = ir
        .nodes
        .iter()
        .filter(|node| {
            !node.deleted
                && node.source_span_start <= replacement.source_span_start
                && replacement.source_span_end <= node.source_span_end
        })
        .min_by_key(|node| {
            (
                node.source_span_len(),
                stable_fact_owner_kind_rank(node.kind),
                node.global_order,
            )
        })
    else {
        return Err(TransformIrSourceReplacementErrorV0::MissingNode {
            source_span_start: replacement.source_span_start,
            source_span_end: replacement.source_span_end,
            kind: replacement.kind,
            candidate_spans: stable_ir
                .nodes
                .iter()
                .filter(|node| node.kind == stable_kind)
                .map(|node| (node.source_span_start, node.source_span_end))
                .collect(),
        });
    };
    let canonical_text = canonical_text_for_node_span(source, replacement, node)?;
    let action = if canonical_text.is_empty() {
        TransformIrReplacementTargetActionV0::DeleteNode
    } else {
        TransformIrReplacementTargetActionV0::ReplaceNode
    };
    Ok(vec![TransformIrReplacementTargetV0 {
        node_id: node.node_id,
        source_span_start: node.source_span_start,
        source_span_end: node.source_span_end,
        replacement_source_span_start: replacement.source_span_start,
        replacement_source_span_end: replacement.source_span_end,
        replacement_text: replacement.replacement.clone(),
        canonical_text,
        action,
    }])
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

const fn stable_fact_owner_kind_rank(kind: IrNodeKindV0) -> u8 {
    match kind {
        IrNodeKindV0::Value => 0,
        IrNodeKindV0::Declaration => 1,
        IrNodeKindV0::AtRule => 2,
        IrNodeKindV0::StyleRule => 3,
        IrNodeKindV0::Selector => 4,
    }
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

fn source_span_contains_stable_fact(
    stable_ir: &omena_transform_cst::StableTransformIrV0,
    replacement: &TransformIrSourceReplacementV0,
    stable_kind: StableTransformIrNodeKindV0,
) -> bool {
    stable_ir.nodes.iter().any(|node| {
        node.kind == stable_kind
            && replacement.source_span_start <= node.source_span_start
            && node.source_span_end <= replacement.source_span_end
    })
}

fn validate_source_range_replacements(
    source: &str,
    dialect: StyleDialect,
    source_id: &str,
    replacements: &[TransformIrSourceReplacementV0],
) -> Result<(), TransformIrSourceReplacementErrorV0> {
    let ir = lower_transform_ir_from_source(source, dialect, source_id);
    let stable_ir = build_stable_transform_ir_from_source(source, dialect, source_id);
    for replacement in replacements {
        if let Some(stable_kind) = replacement.kind.stable_ir_kind() {
            if source_span_contains_stable_fact(&stable_ir, replacement, stable_kind) {
                continue;
            }
            return Err(TransformIrSourceReplacementErrorV0::MissingNode {
                source_span_start: replacement.source_span_start,
                source_span_end: replacement.source_span_end,
                kind: replacement.kind,
                candidate_spans: stable_ir
                    .nodes
                    .iter()
                    .filter(|node| node.kind == stable_kind)
                    .map(|node| (node.source_span_start, node.source_span_end))
                    .collect(),
            });
        }
        find_replacement_targets(source, &ir, None, replacement)?;
    }
    Ok(())
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
    use omena_transform_cst::{
        IrEditRegionV0, IrTransactionV0, lower_transform_ir_from_source, print_transform_ir_css,
    };

    #[test]
    fn stable_fact_replacement_targets_containing_ir_node() -> Result<(), String> {
        let source = ".card { color: var(--brand); }";
        let ir = lower_transform_ir_from_source(source, StyleDialect::Css, "stable-fact-target");
        let stable_ir =
            build_stable_transform_ir_from_source(source, StyleDialect::Css, "stable-fact-target");
        let start = source
            .find("var(--brand)")
            .ok_or_else(|| "fixture should contain a var reference".to_string())?;
        let end = start + "var(--brand)".len();
        let replacement = TransformIrSourceReplacementV0 {
            source_span_start: start,
            source_span_end: end,
            replacement: "var(--accent)".to_string(),
            kind: TransformIrReplacementKindV0::CustomPropertyReference,
        };

        let targets = find_replacement_targets(source, &ir, Some(&stable_ir), &replacement)
            .map_err(|err| format!("stable fact should map to an IR owner: {err:?}"))?;

        assert_eq!(targets.len(), 1);
        let target = &targets[0];
        let node = &ir.nodes[target.node_id.index()];
        assert!(node.source_span_start <= start);
        assert!(end <= node.source_span_end);
        assert!(matches!(
            target.action,
            TransformIrReplacementTargetActionV0::ReplaceNode
        ));
        assert!(target.canonical_text.contains("var(--accent)"));
        Ok(())
    }

    #[test]
    fn stable_fact_replacement_materializes_ir_for_follow_up_transaction() -> Result<(), String> {
        let mut ir = lower_transform_ir_from_source(
            ".card { color: var(--brand); }",
            StyleDialect::Css,
            "stable-fact-materialized",
        );
        let start = ir
            .source_text()
            .find("var(--brand)")
            .ok_or_else(|| "fixture should contain a var reference".to_string())?;
        let end = start + "var(--brand)".len();
        let replacement = TransformIrSourceReplacementV0 {
            source_span_start: start,
            source_span_end: end,
            replacement: "var(--accent)".to_string(),
            kind: TransformIrReplacementKindV0::CustomPropertyReference,
        };

        let (output, mutation_count) = apply_ir_source_replacements_to_ir(
            &mut ir,
            StyleDialect::Css,
            "design-token-routing",
            &[replacement],
        )
        .map_err(|err| format!("stable fact transaction should apply: {err:?}"))?;

        assert_eq!(output, ".card { color: var(--accent); }");
        assert_eq!(mutation_count, 1);
        assert_eq!(ir.source_text(), output);
        assert!(ir.all_nodes_original());

        let value_id = ir
            .nodes
            .iter()
            .find(|node| {
                node.kind == IrNodeKindV0::Value
                    && ir.source_text()[node.source_span_start..node.source_span_end]
                        .contains("var(--accent)")
            })
            .map(|node| node.node_id)
            .ok_or_else(|| "materialized IR should expose the routed value span".to_string())?;
        let mut transaction =
            IrTransactionV0::new(&mut ir, "follow-up-value-rewrite", IrEditRegionV0::full(32));
        transaction
            .rewrite_value(value_id, " blue")
            .map_err(|err| {
                format!("follow-up value rewrite should target rebased span: {err:?}")
            })?;
        transaction
            .commit()
            .map_err(|err| format!("follow-up transaction should commit: {err:?}"))?;
        assert_eq!(
            print_transform_ir_css(&ir)
                .map_err(|err| format!("follow-up transaction should print: {err:?}"))?,
            ".card { color: blue; }"
        );
        Ok(())
    }

    #[test]
    fn mixed_style_rule_and_stable_fact_replacements_coalesce_on_same_node() -> Result<(), String> {
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

        let (output, mutation_count) = apply_ir_source_replacements_to_ir(
            &mut ir,
            StyleDialect::Css,
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
                    kind: TransformIrReplacementKindV0::CssModuleComposesTarget,
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

        let (output, mutation_count) = apply_ir_source_replacements_to_ir(
            &mut ir,
            StyleDialect::Css,
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

        let targets = find_replacement_targets(source, &ir, None, &replacement)
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
        let targets = find_replacement_targets(source, &ir, None, &replacement)
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
