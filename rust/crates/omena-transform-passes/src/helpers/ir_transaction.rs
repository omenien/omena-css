use omena_parser::StyleDialect;
use omena_transform_cst::{
    IrEditRegionV0, IrNodeIdV0, IrNodeKindV0, IrNodeV0, IrTransactionErrorV0, IrTransactionV0,
    StableTransformIrNodeKindV0, TransformIrPrintErrorV0, TransformIrV0,
    build_stable_transform_ir_from_source, lower_transform_ir_from_source,
    materialize_transform_ir_printed_source, print_transform_ir_css,
};

use crate::helpers::source_rewrite::replace_source_ranges;

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

struct TransformIrReplacementTargetV0 {
    node_id: IrNodeIdV0,
    source_span_start: usize,
    source_span_end: usize,
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
    if replacements
        .iter()
        .any(|replacement| replacement.kind.stable_ir_kind().is_some())
    {
        let source_id = ir.source_id.clone();
        validate_ir_source_replacements(
            source.as_str(),
            dialect,
            source_id.as_str(),
            &replacements,
        )?;
        return apply_source_range_replacements_to_ir(ir, dialect, &replacements);
    }
    if pass_id == "css-modules-class-hashing" && dialect == StyleDialect::Less {
        return apply_source_range_replacements_to_ir(ir, dialect, &replacements);
    }

    let replacement_targets = match replacements
        .iter()
        .map(|replacement| find_replacement_targets(source.as_str(), ir, replacement))
        .collect::<Result<Vec<_>, _>>()
    {
        Ok(targets) => targets,
        Err(error) => {
            return tree_shake_class_source_fact_fallback(
                ir,
                dialect,
                pass_id,
                &replacements,
                error,
            );
        }
    }
    .into_iter()
    .flatten()
    .collect::<Vec<_>>();
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
        return tree_shake_class_source_fact_fallback(ir, dialect, pass_id, &replacements, error);
    }
    let printed_css = match materialize_transform_ir_printed_source(ir) {
        Ok(printed_css) => printed_css,
        Err(_) => match print_transform_ir_css(ir) {
            Ok(printed_css) => {
                let source_id = ir.source_id.clone();
                *ir = lower_transform_ir_from_source(printed_css.as_str(), dialect, source_id);
                printed_css
            }
            Err(error) => {
                return tree_shake_class_source_fact_fallback(
                    ir,
                    dialect,
                    pass_id,
                    &replacements,
                    TransformIrSourceReplacementErrorV0::Print(error),
                );
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
                canonical_text: String::new(),
                action: TransformIrReplacementTargetActionV0::DeleteNode,
            }),
    );
    Ok(targets)
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

fn validate_ir_source_replacements(
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
        find_replacement_targets(source, &ir, replacement)?;
    }
    Ok(())
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

fn tree_shake_class_source_fact_fallback(
    ir: &mut TransformIrV0,
    dialect: StyleDialect,
    pass_id: &str,
    replacements: &[TransformIrSourceReplacementV0],
    error: TransformIrSourceReplacementErrorV0,
) -> Result<(String, usize), TransformIrSourceReplacementErrorV0> {
    let source = ir.source_text().to_string();
    if pass_id != "tree-shake-class"
        || !replacements
            .iter()
            .all(|replacement| replacement.kind == TransformIrReplacementKindV0::StyleRule)
    {
        return Err(error);
    }
    let source_id = ir.source_id.clone();
    let stable_ir =
        build_stable_transform_ir_from_source(source.as_str(), dialect, source_id.as_str());
    if !replacements.iter().all(|replacement| {
        source_span_contains_stable_fact(
            &stable_ir,
            replacement,
            StableTransformIrNodeKindV0::ClassSelector,
        )
    }) {
        return Err(error);
    }
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
