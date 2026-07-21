//! Observational cascade-winner comparisons for admitted transform mutations.

use std::collections::{BTreeMap, BTreeSet};

use omena_cascade::{
    CascadeDeclaration, CascadeLevel, CascadeValue, CascadeWinnerAxisV0, ElementSignature,
    LayerRank, ModuleRank, SelectorMatchVerdict, SpecificityExactnessV0, cascade_driven_levels_v0,
    cascade_driven_winner_axes_v0, cascade_level_for_origin, cascade_property,
    parse_simple_selector_signature, selector_match_witness,
};
use omena_parser::{StyleDialect, css_keyword};
use omena_semantic::summarize_style_layer_order_from_source;
use omena_transform_cst::{
    ObservationKindV0, PassAssumptionKindV0, PassObservationSurfaceV0, TransformIrV0,
    TransformPassKind, pass_observation_contract,
};

use super::semantic_preservation::{
    SemanticCascadeCandidateV0, SemanticObservationScopeV0, semantic_cascade_candidates,
};
use crate::model::{
    TransformCascadeEnvironmentV0, TransformProvenanceMutationSpanV0,
    TransformSemanticGuaranteeTierV0, TransformWinnerEqualityAbsenceReasonV0,
    TransformWinnerEqualityAbsenceV0, TransformWinnerEqualityAffectedPairV0,
    TransformWinnerEqualityAxisV0, TransformWinnerEqualityObligationV0,
    TransformWinnerEqualityObservationV0, TransformWinnerEqualityWitnessV0,
};

#[derive(Debug)]
pub(crate) struct TransformWinnerEqualityEvaluationV0 {
    pub(crate) obligations: Vec<TransformWinnerEqualityObligationV0>,
    pub(crate) unresolved_reasons: Vec<TransformWinnerEqualityAbsenceV0>,
    pub(crate) tier: TransformSemanticGuaranteeTierV0,
}

#[derive(Clone, Copy)]
pub(crate) struct TransformWinnerEqualityContextV0<'a> {
    pub(crate) input_scope: SemanticObservationScopeV0<'a>,
    pub(crate) output_scope: SemanticObservationScopeV0<'a>,
    pub(crate) cascade_environment: Option<&'a TransformCascadeEnvironmentV0>,
}

pub(crate) fn evaluate_transform_winner_equality(
    pass: TransformPassKind,
    input_ir: &TransformIrV0,
    output_ir: &TransformIrV0,
    mutation_spans: &[TransformProvenanceMutationSpanV0],
    dialect: StyleDialect,
    context: TransformWinnerEqualityContextV0<'_>,
) -> TransformWinnerEqualityEvaluationV0 {
    let input_candidates = semantic_cascade_candidates(input_ir, context.input_scope);
    let output_candidates = semantic_cascade_candidates(output_ir, context.output_scope);
    let mut pairs = BTreeMap::new();
    let mut inexact_pair_ids = BTreeSet::new();
    let mut pair_derivation_failed = false;

    for candidate in input_candidates
        .iter()
        .filter(|candidate| overlaps_input_mutation(candidate, mutation_spans))
        .chain(
            output_candidates
                .iter()
                .filter(|candidate| overlaps_output_mutation(candidate, mutation_spans)),
        )
    {
        let Some(signature) = parse_simple_selector_signature(candidate.selector.as_str()) else {
            pair_derivation_failed = true;
            continue;
        };
        let element_signature = ElementSignature {
            tag: signature.required_tag,
            id: signature.required_id,
            classes: signature.required_classes,
            attributes: signature.required_attributes,
            pseudo_states: signature.required_pseudo_states,
            classes_are_exact: true,
            attributes_are_exact: true,
            pseudo_states_are_exact: true,
            tag_is_exact: true,
            id_is_exact: true,
        };
        let pair = TransformWinnerEqualityAffectedPairV0 {
            element_signature,
            property: candidate.property.clone(),
        };
        let pair_id = pair_identity(&pair);
        if signature.specificity_exactness == SpecificityExactnessV0::Inexact {
            inexact_pair_ids.insert(pair_id.clone());
        }
        pairs.entry(pair_id).or_insert(pair);
    }

    if pairs.is_empty() {
        let unresolved_reasons = vec![TransformWinnerEqualityAbsenceV0 {
            axis: TransformWinnerEqualityAxisV0::Specificity,
            reason: TransformWinnerEqualityAbsenceReasonV0::AffectedPairUnavailable,
        }];
        return TransformWinnerEqualityEvaluationV0 {
            obligations: Vec::new(),
            unresolved_reasons: unresolved_reasons.clone(),
            tier: TransformSemanticGuaranteeTierV0::Absent {
                reasons: unresolved_reasons,
            },
        };
    }

    let input_layers = summarize_style_layer_order_from_source(input_ir.source_text(), dialect);
    let output_layers = summarize_style_layer_order_from_source(output_ir.source_text(), dialect);
    let axes = driven_transform_axes();
    let driven_levels = cascade_driven_levels_v0()
        .into_iter()
        .collect::<BTreeSet<_>>();
    let mut obligations = Vec::new();

    for (pair_id, pair) in pairs {
        let mut reasons = Vec::new();
        if context.cascade_environment.is_none() {
            reasons.push(TransformWinnerEqualityAbsenceV0 {
                axis: TransformWinnerEqualityAxisV0::CascadeLevel,
                reason: TransformWinnerEqualityAbsenceReasonV0::DriverUnavailable { level: None },
            });
        }
        if pair_derivation_failed {
            reasons.push(TransformWinnerEqualityAbsenceV0 {
                axis: TransformWinnerEqualityAxisV0::Specificity,
                reason: TransformWinnerEqualityAbsenceReasonV0::AffectedPairUnavailable,
            });
        }
        if inexact_pair_ids.contains(pair_id.as_str()) {
            reasons.push(TransformWinnerEqualityAbsenceV0 {
                axis: TransformWinnerEqualityAxisV0::Specificity,
                reason: TransformWinnerEqualityAbsenceReasonV0::SpecificityInexact,
            });
        }
        let input = winner_for_pair(
            &pair,
            input_candidates.as_slice(),
            &input_layers,
            &driven_levels,
            &mut reasons,
            context.cascade_environment,
        );
        let output = winner_for_pair(
            &pair,
            output_candidates.as_slice(),
            &output_layers,
            &driven_levels,
            &mut reasons,
            context.cascade_environment,
        );
        deduplicate_absences(&mut reasons);

        let observation = if !reasons.is_empty() {
            TransformWinnerEqualityObservationV0::Absent { reasons }
        } else if let (Some(input), Some(output)) = (input, output) {
            if winner_witnesses_are_observationally_equal(&input, &output) {
                TransformWinnerEqualityObservationV0::ObservedEqual {
                    axes: axes.clone(),
                    input,
                    output,
                }
            } else {
                TransformWinnerEqualityObservationV0::ObservedDifferent {
                    axes: axes.clone(),
                    input,
                    output,
                }
            }
        } else {
            TransformWinnerEqualityObservationV0::Absent {
                reasons: vec![TransformWinnerEqualityAbsenceV0 {
                    axis: TransformWinnerEqualityAxisV0::CascadeLevel,
                    reason: TransformWinnerEqualityAbsenceReasonV0::WinnerNotDefinite,
                }],
            }
        };

        obligations.push(TransformWinnerEqualityObligationV0 {
            pass_id: pass.id(),
            affected_pair: pair,
            observation,
        });
    }

    let unresolved_reasons = obligations
        .iter()
        .flat_map(|obligation| match &obligation.observation {
            TransformWinnerEqualityObservationV0::Absent { reasons } => reasons.clone(),
            TransformWinnerEqualityObservationV0::ObservedEqual { .. }
            | TransformWinnerEqualityObservationV0::ObservedDifferent { .. } => Vec::new(),
        })
        .collect();
    let tier = tier_from_obligations(obligations.as_slice(), axes);
    TransformWinnerEqualityEvaluationV0 {
        obligations,
        unresolved_reasons,
        tier,
    }
}

fn winner_witnesses_are_observationally_equal(
    input: &TransformWinnerEqualityWitnessV0,
    output: &TransformWinnerEqualityWitnessV0,
) -> bool {
    // Absolute source ordinals may be renumbered when unrelated declarations
    // disappear; source-order effects remain observable through winner identity.
    input.winner.id == output.winner.id
        && input.winner.property == output.winner.property
        && input.winner.value == output.winner.value
        && input.proof.level == output.proof.level
        && input.proof.layer_rank == output.proof.layer_rank
        && input.proof.scope_proximity == output.proof.scope_proximity
        && input.proof.specificity == output.proof.specificity
        && input.proof.module_rank == output.proof.module_rank
}

pub fn compare_transform_winner_equality_for_conformance_v0(
    input: &str,
    output: &str,
    dialect: StyleDialect,
    pass: TransformPassKind,
) -> Vec<TransformWinnerEqualityObligationV0> {
    let input_ir = omena_transform_cst::lower_transform_ir_from_source(
        input,
        dialect,
        "omena-transform-passes.winner-equality.input",
    );
    let output_ir = omena_transform_cst::lower_transform_ir_from_source(
        output,
        dialect,
        "omena-transform-passes.winner-equality.output",
    );
    evaluate_transform_winner_equality(
        pass,
        &input_ir,
        &output_ir,
        &[TransformProvenanceMutationSpanV0 {
            source_span_start: 0,
            source_span_end: input.len(),
            generated_span_start: 0,
            generated_span_end: output.len(),
            node_key: None,
        }],
        dialect,
        TransformWinnerEqualityContextV0 {
            input_scope: SemanticObservationScopeV0::default(),
            output_scope: SemanticObservationScopeV0::default(),
            cascade_environment: Some(&TransformCascadeEnvironmentV0::default()),
        },
    )
    .obligations
}

fn winner_for_pair(
    pair: &TransformWinnerEqualityAffectedPairV0,
    candidates: &[SemanticCascadeCandidateV0],
    layer_index: &omena_semantic::StyleLayerIndexV0,
    driven_levels: &BTreeSet<CascadeLevel>,
    reasons: &mut Vec<TransformWinnerEqualityAbsenceV0>,
    cascade_environment: Option<&TransformCascadeEnvironmentV0>,
) -> Option<TransformWinnerEqualityWitnessV0> {
    let mut declarations = Vec::new();
    let mut matched_ordinal = 0usize;
    let stylesheet_source_order_base = cascade_environment
        .map(|environment| environment.stylesheet_source_order_base)
        .unwrap_or_default();
    for candidate in candidates
        .iter()
        .filter(|candidate| css_keyword(candidate.property.as_str()).equals(pair.property.as_str()))
    {
        let witness = selector_match_witness(candidate.selector.as_str(), &pair.element_signature);
        match witness.verdict {
            SelectorMatchVerdict::No => continue,
            SelectorMatchVerdict::Maybe => {
                reasons.push(TransformWinnerEqualityAbsenceV0 {
                    axis: TransformWinnerEqualityAxisV0::Specificity,
                    reason: TransformWinnerEqualityAbsenceReasonV0::WinnerNotDefinite,
                });
                continue;
            }
            SelectorMatchVerdict::Yes => {}
        }
        if conditional_context_is_open(candidate.context_key.as_str()) {
            reasons.push(TransformWinnerEqualityAbsenceV0 {
                axis: TransformWinnerEqualityAxisV0::CascadeLevel,
                reason: TransformWinnerEqualityAbsenceReasonV0::SpecificityInexact,
            });
        }
        if css_keyword(candidate.context_key.as_str()).contains("@scope") {
            reasons.push(TransformWinnerEqualityAbsenceV0 {
                axis: TransformWinnerEqualityAxisV0::ScopeProximity,
                reason: TransformWinnerEqualityAbsenceReasonV0::DriverUnavailable { level: None },
            });
        }
        let level =
            cascade_level_for_origin(omena_cascade::CascadeOriginV0::Author, candidate.important);
        if !driven_levels.contains(&level) {
            reasons.push(TransformWinnerEqualityAbsenceV0 {
                axis: TransformWinnerEqualityAxisV0::CascadeLevel,
                reason: TransformWinnerEqualityAbsenceReasonV0::DriverUnavailable {
                    level: Some(level),
                },
            });
        }
        let layer_rank = layer_rank_for_candidate(candidate, layer_index, reasons);
        let Some(signature) = parse_simple_selector_signature(candidate.selector.as_str()) else {
            reasons.push(TransformWinnerEqualityAbsenceV0 {
                axis: TransformWinnerEqualityAxisV0::Specificity,
                reason: TransformWinnerEqualityAbsenceReasonV0::WinnerNotDefinite,
            });
            continue;
        };
        if signature.specificity_exactness == SpecificityExactnessV0::Inexact {
            reasons.push(TransformWinnerEqualityAbsenceV0 {
                axis: TransformWinnerEqualityAxisV0::Specificity,
                reason: TransformWinnerEqualityAbsenceReasonV0::WinnerNotDefinite,
            });
        }
        let specificity = signature.specificity;
        let source_order = stylesheet_source_order_base
            .saturating_add(u32::try_from(matched_ordinal).unwrap_or(u32::MAX));
        matched_ordinal = matched_ordinal.saturating_add(1);
        declarations.push(CascadeDeclaration {
            id: format!(
                "{}|{}|{}|{}",
                candidate.selector, candidate.property, candidate.value, candidate.important
            ),
            property: candidate.property.clone(),
            value: CascadeValue::Literal(candidate.value.clone()),
            key: omena_cascade::CascadeKey::new(
                level,
                layer_rank,
                0,
                specificity,
                ModuleRank::ZERO,
                source_order,
            ),
            specificity_exactness: signature.specificity_exactness,
        });
    }

    if let Some(environment) = cascade_environment {
        for declaration in environment.declarations.iter().filter(|declaration| {
            css_keyword(declaration.property.as_str()).equals(pair.property.as_str())
        }) {
            let witness =
                selector_match_witness(declaration.selector.as_str(), &pair.element_signature);
            match witness.verdict {
                SelectorMatchVerdict::No => continue,
                SelectorMatchVerdict::Maybe => {
                    reasons.push(TransformWinnerEqualityAbsenceV0 {
                        axis: TransformWinnerEqualityAxisV0::Specificity,
                        reason: TransformWinnerEqualityAbsenceReasonV0::WinnerNotDefinite,
                    });
                    continue;
                }
                SelectorMatchVerdict::Yes => {}
            }
            let level = cascade_level_for_origin(declaration.origin, declaration.important);
            if !driven_levels.contains(&level) {
                reasons.push(TransformWinnerEqualityAbsenceV0 {
                    axis: TransformWinnerEqualityAxisV0::CascadeLevel,
                    reason: TransformWinnerEqualityAbsenceReasonV0::DriverUnavailable {
                        level: Some(level),
                    },
                });
            }
            let Some(signature) = parse_simple_selector_signature(declaration.selector.as_str())
            else {
                reasons.push(TransformWinnerEqualityAbsenceV0 {
                    axis: TransformWinnerEqualityAxisV0::Specificity,
                    reason: TransformWinnerEqualityAbsenceReasonV0::SpecificityInexact,
                });
                continue;
            };
            if signature.specificity_exactness == SpecificityExactnessV0::Inexact {
                reasons.push(TransformWinnerEqualityAbsenceV0 {
                    axis: TransformWinnerEqualityAxisV0::Specificity,
                    reason: TransformWinnerEqualityAbsenceReasonV0::WinnerNotDefinite,
                });
            }
            let raw_layer_rank = declaration.layer_rank;
            let layer_rank = match (declaration.important, raw_layer_rank) {
                (false, Some(rank)) => LayerRank(rank),
                (false, None) => LayerRank(i32::MAX),
                (true, Some(rank)) => LayerRank(rank.saturating_neg()),
                (true, None) => LayerRank(i32::MIN),
            };
            declarations.push(CascadeDeclaration {
                id: declaration.declaration_id.clone(),
                property: declaration.property.clone(),
                value: CascadeValue::Literal(declaration.value.clone()),
                key: omena_cascade::CascadeKey::new(
                    level,
                    layer_rank,
                    declaration.scope_proximity.unwrap_or(0),
                    signature.specificity,
                    ModuleRank::ZERO,
                    declaration.source_order,
                ),
                specificity_exactness: signature.specificity_exactness,
            });
        }
    }

    TransformWinnerEqualityWitnessV0::from_cascade_outcome(&cascade_property(
        declarations,
        pair.property.as_str(),
    ))
}

fn layer_rank_for_candidate(
    candidate: &SemanticCascadeCandidateV0,
    layer_index: &omena_semantic::StyleLayerIndexV0,
    reasons: &mut Vec<TransformWinnerEqualityAbsenceV0>,
) -> LayerRank {
    if !layer_index.topology_complete && !layer_index.block_bindings.is_empty() {
        reasons.push(TransformWinnerEqualityAbsenceV0 {
            axis: TransformWinnerEqualityAxisV0::LayerRank,
            reason: TransformWinnerEqualityAbsenceReasonV0::DriverUnavailable { level: None },
        });
    }
    let rank = layer_index
        .block_bindings
        .iter()
        .filter(|binding| {
            binding.byte_span.start <= candidate.source_span_start
                && candidate.source_span_end <= binding.byte_span.end
        })
        .max_by_key(|binding| binding.nesting_depth)
        .map(|binding| i32::try_from(binding.cascade_rank).unwrap_or(i32::MAX - 1));
    match (candidate.important, rank) {
        (false, Some(rank)) => LayerRank(rank),
        (false, None) => LayerRank(i32::MAX),
        (true, Some(rank)) => LayerRank(rank.saturating_neg()),
        (true, None) => LayerRank(i32::MIN),
    }
}

pub(crate) fn driven_transform_axes() -> Vec<TransformWinnerEqualityAxisV0> {
    cascade_driven_winner_axes_v0()
        .into_iter()
        .map(|axis| match axis {
            CascadeWinnerAxisV0::CascadeLevel => TransformWinnerEqualityAxisV0::CascadeLevel,
            CascadeWinnerAxisV0::LayerRank => TransformWinnerEqualityAxisV0::LayerRank,
            CascadeWinnerAxisV0::ScopeProximity => TransformWinnerEqualityAxisV0::ScopeProximity,
            CascadeWinnerAxisV0::Specificity => TransformWinnerEqualityAxisV0::Specificity,
            CascadeWinnerAxisV0::SourceOrder => TransformWinnerEqualityAxisV0::SourceOrder,
        })
        .collect()
}

pub(crate) fn strict_required_winner_axes(
    pass: TransformPassKind,
) -> Vec<TransformWinnerEqualityAxisV0> {
    let PassObservationSurfaceV0::Declared(contract) = pass_observation_contract(pass) else {
        return Vec::new();
    };
    let mut axes = BTreeSet::new();
    let declared = contract
        .observes
        .iter()
        .chain(contract.preserves.iter())
        .copied()
        .collect::<BTreeSet<_>>();
    if declared.contains(&ObservationKindV0::CascadeWinner) {
        axes.insert(TransformWinnerEqualityAxisV0::CascadeLevel);
    }
    if declared.contains(&ObservationKindV0::LayerRank) {
        axes.insert(TransformWinnerEqualityAxisV0::LayerRank);
    }
    if declared.contains(&ObservationKindV0::Specificity) {
        axes.insert(TransformWinnerEqualityAxisV0::Specificity);
    }
    if declared.contains(&ObservationKindV0::DeclarationOrder) {
        axes.insert(TransformWinnerEqualityAxisV0::SourceOrder);
    }
    if contract
        .requires
        .contains(&PassAssumptionKindV0::ScopedMatching)
    {
        axes.insert(TransformWinnerEqualityAxisV0::ScopeProximity);
    }
    axes.into_iter().collect()
}

fn tier_from_obligations(
    obligations: &[TransformWinnerEqualityObligationV0],
    axes: Vec<TransformWinnerEqualityAxisV0>,
) -> TransformSemanticGuaranteeTierV0 {
    let mut reasons = obligations
        .iter()
        .flat_map(|obligation| match &obligation.observation {
            TransformWinnerEqualityObservationV0::Absent { reasons } => reasons.clone(),
            TransformWinnerEqualityObservationV0::ObservedDifferent { input, output, .. } => {
                winner_difference_absences(input, output, axes.as_slice())
            }
            TransformWinnerEqualityObservationV0::ObservedEqual { .. } => Vec::new(),
        })
        .collect::<Vec<_>>();
    deduplicate_absences(&mut reasons);
    if reasons.is_empty()
        && obligations.iter().all(|obligation| {
            matches!(
                obligation.observation,
                TransformWinnerEqualityObservationV0::ObservedEqual { .. }
            )
        })
    {
        TransformSemanticGuaranteeTierV0::WinnerEqualityObserved { axes }
    } else {
        TransformSemanticGuaranteeTierV0::Absent { reasons }
    }
}

fn winner_difference_absences(
    input: &TransformWinnerEqualityWitnessV0,
    output: &TransformWinnerEqualityWitnessV0,
    driven_axes: &[TransformWinnerEqualityAxisV0],
) -> Vec<TransformWinnerEqualityAbsenceV0> {
    if input.winner.id != output.winner.id
        || input.winner.property != output.winner.property
        || input.winner.value != output.winner.value
    {
        return driven_axes
            .iter()
            .copied()
            .map(|axis| TransformWinnerEqualityAbsenceV0 {
                axis,
                reason: TransformWinnerEqualityAbsenceReasonV0::WinnerChanged,
            })
            .collect();
    }

    let mut axes = Vec::new();
    if input.proof.level != output.proof.level {
        axes.push(TransformWinnerEqualityAxisV0::CascadeLevel);
    }
    if input.proof.layer_rank != output.proof.layer_rank {
        axes.push(TransformWinnerEqualityAxisV0::LayerRank);
    }
    if input.proof.scope_proximity != output.proof.scope_proximity {
        axes.push(TransformWinnerEqualityAxisV0::ScopeProximity);
    }
    if input.proof.specificity != output.proof.specificity {
        axes.push(TransformWinnerEqualityAxisV0::Specificity);
    }
    if input.proof.source_order != output.proof.source_order {
        axes.push(TransformWinnerEqualityAxisV0::SourceOrder);
    }
    if axes.is_empty() {
        // A proof-shape difference outside the modeled fields cannot be
        // attributed more narrowly than the covered axis set.
        axes.extend_from_slice(driven_axes);
    }
    axes.into_iter()
        .map(|axis| TransformWinnerEqualityAbsenceV0 {
            axis,
            reason: TransformWinnerEqualityAbsenceReasonV0::WinnerChanged,
        })
        .collect()
}

fn pair_identity(pair: &TransformWinnerEqualityAffectedPairV0) -> String {
    format!("{:?}|{}", pair.element_signature, pair.property)
}

fn overlaps_input_mutation(
    candidate: &SemanticCascadeCandidateV0,
    spans: &[TransformProvenanceMutationSpanV0],
) -> bool {
    spans.iter().any(|span| {
        ranges_overlap(
            candidate.source_span_start,
            candidate.source_span_end,
            span.source_span_start,
            span.source_span_end,
        )
    })
}

fn overlaps_output_mutation(
    candidate: &SemanticCascadeCandidateV0,
    spans: &[TransformProvenanceMutationSpanV0],
) -> bool {
    spans.iter().any(|span| {
        ranges_overlap(
            candidate.source_span_start,
            candidate.source_span_end,
            span.generated_span_start,
            span.generated_span_end,
        )
    })
}

fn ranges_overlap(
    left_start: usize,
    left_end: usize,
    right_start: usize,
    right_end: usize,
) -> bool {
    left_start < right_end && right_start < left_end
}

fn conditional_context_is_open(context: &str) -> bool {
    css_keyword(context).contains("@media")
        || css_keyword(context).contains("@supports")
        || css_keyword(context).contains("@container")
}

fn deduplicate_absences(reasons: &mut Vec<TransformWinnerEqualityAbsenceV0>) {
    reasons.sort_by_key(|reason| format!("{reason:?}"));
    reasons.dedup();
}

#[cfg(test)]
mod tests {
    use crate::model::TransformCascadeEnvironmentDeclarationV0;
    use omena_transform_cst::lower_transform_ir_from_source;

    use super::*;

    fn mutation_span(source: &str, output: &str) -> TransformProvenanceMutationSpanV0 {
        TransformProvenanceMutationSpanV0 {
            source_span_start: 0,
            source_span_end: source.len(),
            generated_span_start: 0,
            generated_span_end: output.len(),
            node_key: None,
        }
    }

    #[test]
    fn winner_comparison_detects_a_layer_order_flip() {
        let input = "@layer low, high; @layer low { .a { color: red; } } @layer high { .a { color: blue; } }";
        let output = "@layer high, low; @layer low { .a { color: red; } } @layer high { .a { color: blue; } }";
        let input_ir = lower_transform_ir_from_source(input, StyleDialect::Css, "winner-input");
        let output_ir = lower_transform_ir_from_source(output, StyleDialect::Css, "winner-output");
        let result = evaluate_transform_winner_equality(
            TransformPassKind::LayerFlatten,
            &input_ir,
            &output_ir,
            &[mutation_span(input, output)],
            StyleDialect::Css,
            TransformWinnerEqualityContextV0 {
                input_scope: SemanticObservationScopeV0::default(),
                output_scope: SemanticObservationScopeV0::default(),
                cascade_environment: Some(&TransformCascadeEnvironmentV0::default()),
            },
        );

        assert!(result.obligations.iter().any(|obligation| matches!(
            obligation.observation,
            TransformWinnerEqualityObservationV0::ObservedDifferent { .. }
        )));
        assert!(matches!(
            result.tier,
            TransformSemanticGuaranteeTierV0::Absent { reasons }
                if reasons.iter().any(|reason| {
                    reason.axis == TransformWinnerEqualityAxisV0::LayerRank
                        && reason.reason
                            == TransformWinnerEqualityAbsenceReasonV0::WinnerChanged
                })
        ));
    }

    #[test]
    fn unresolved_scope_proximity_is_typed_instead_of_claimed() {
        let source = "@scope (.root) { .a { color: red; } }";
        let input_ir = lower_transform_ir_from_source(source, StyleDialect::Css, "scope-input");
        let output_ir = lower_transform_ir_from_source(source, StyleDialect::Css, "scope-output");
        let result = evaluate_transform_winner_equality(
            TransformPassKind::ScopeFlatten,
            &input_ir,
            &output_ir,
            &[mutation_span(source, source)],
            StyleDialect::Css,
            TransformWinnerEqualityContextV0 {
                input_scope: SemanticObservationScopeV0::default(),
                output_scope: SemanticObservationScopeV0::default(),
                cascade_environment: Some(&TransformCascadeEnvironmentV0::default()),
            },
        );

        assert!(result.obligations.iter().any(|obligation| matches!(
            &obligation.observation,
            TransformWinnerEqualityObservationV0::Absent { reasons }
                if reasons.iter().any(|reason| reason.axis == TransformWinnerEqualityAxisV0::ScopeProximity)
        )));
    }

    #[test]
    fn missing_cascade_environment_is_reported_as_typed_absence() {
        let source = ".a { color: red; } .a { color: blue; }";
        let input_ir = lower_transform_ir_from_source(source, StyleDialect::Css, "origin-input");
        let output_ir = lower_transform_ir_from_source(source, StyleDialect::Css, "origin-output");
        let result = evaluate_transform_winner_equality(
            TransformPassKind::RuleMerging,
            &input_ir,
            &output_ir,
            &[mutation_span(source, source)],
            StyleDialect::Css,
            TransformWinnerEqualityContextV0 {
                input_scope: SemanticObservationScopeV0::default(),
                output_scope: SemanticObservationScopeV0::default(),
                cascade_environment: None,
            },
        );

        assert!(result.obligations.iter().all(|obligation| matches!(
            &obligation.observation,
            TransformWinnerEqualityObservationV0::Absent { reasons }
                if reasons.iter().any(|reason| {
                    reason.axis == TransformWinnerEqualityAxisV0::CascadeLevel
                        && reason.reason
                            == TransformWinnerEqualityAbsenceReasonV0::DriverUnavailable {
                                level: None,
                            }
                })
        )));
    }

    #[test]
    fn inexact_specificity_is_reported_as_typed_absence() {
        let source = ":is(:unknown(.a), .b) { color: red; }";
        let input_ir = lower_transform_ir_from_source(source, StyleDialect::Css, "inexact-input");
        let output_ir = lower_transform_ir_from_source(source, StyleDialect::Css, "inexact-output");
        let result = evaluate_transform_winner_equality(
            TransformPassKind::RuleMerging,
            &input_ir,
            &output_ir,
            &[mutation_span(source, source)],
            StyleDialect::Css,
            TransformWinnerEqualityContextV0 {
                input_scope: SemanticObservationScopeV0::default(),
                output_scope: SemanticObservationScopeV0::default(),
                cascade_environment: Some(&TransformCascadeEnvironmentV0::default()),
            },
        );
        assert!(result.unresolved_reasons.iter().any(|absence| {
            absence.axis == TransformWinnerEqualityAxisV0::Specificity
                && absence.reason == TransformWinnerEqualityAbsenceReasonV0::SpecificityInexact
        }));
        assert!(matches!(
            result.tier,
            TransformSemanticGuaranteeTierV0::Absent { .. }
        ));
    }

    #[test]
    fn exact_specificity_does_not_emit_an_inexactness_absence() {
        let source = ".a { color: red; }";
        let input_ir = lower_transform_ir_from_source(source, StyleDialect::Css, "exact-input");
        let output_ir = lower_transform_ir_from_source(source, StyleDialect::Css, "exact-output");
        let result = evaluate_transform_winner_equality(
            TransformPassKind::RuleMerging,
            &input_ir,
            &output_ir,
            &[mutation_span(source, source)],
            StyleDialect::Css,
            TransformWinnerEqualityContextV0 {
                input_scope: SemanticObservationScopeV0::default(),
                output_scope: SemanticObservationScopeV0::default(),
                cascade_environment: Some(&TransformCascadeEnvironmentV0::default()),
            },
        );

        assert!(!result.unresolved_reasons.iter().any(|absence| {
            absence.reason == TransformWinnerEqualityAbsenceReasonV0::SpecificityInexact
        }));
    }

    #[test]
    fn unsupported_selector_keeps_the_distinct_pair_absence() {
        let source = ":unknown(.a) { color: red; }";
        let input_ir =
            lower_transform_ir_from_source(source, StyleDialect::Css, "unsupported-input");
        let output_ir =
            lower_transform_ir_from_source(source, StyleDialect::Css, "unsupported-output");
        let result = evaluate_transform_winner_equality(
            TransformPassKind::RuleMerging,
            &input_ir,
            &output_ir,
            &[mutation_span(source, source)],
            StyleDialect::Css,
            TransformWinnerEqualityContextV0 {
                input_scope: SemanticObservationScopeV0::default(),
                output_scope: SemanticObservationScopeV0::default(),
                cascade_environment: Some(&TransformCascadeEnvironmentV0::default()),
            },
        );

        assert!(result.unresolved_reasons.iter().any(|absence| {
            absence.reason == TransformWinnerEqualityAbsenceReasonV0::AffectedPairUnavailable
        }));
        assert!(!result.unresolved_reasons.iter().any(|absence| {
            absence.reason == TransformWinnerEqualityAbsenceReasonV0::SpecificityInexact
        }));
    }

    #[test]
    fn source_order_flip_changes_the_observed_winner() {
        let input = ".a { color: red; } .a { color: blue; }";
        let output = ".a { color: blue; } .a { color: red; }";
        let obligations = compare_transform_winner_equality_for_conformance_v0(
            input,
            output,
            StyleDialect::Css,
            TransformPassKind::RuleMerging,
        );

        assert!(obligations.iter().any(|obligation| matches!(
            obligation.observation,
            TransformWinnerEqualityObservationV0::ObservedDifferent { .. }
        )));
    }

    #[test]
    fn complete_environment_participates_in_origin_winner_selection() {
        let input = ".a { color: red !important; }";
        let output = ".a { color: blue !important; }";
        let input_ir = lower_transform_ir_from_source(input, StyleDialect::Css, "origin-input");
        let output_ir = lower_transform_ir_from_source(output, StyleDialect::Css, "origin-output");
        let environment = TransformCascadeEnvironmentV0 {
            stylesheet_source_order_base: 0,
            declarations: vec![TransformCascadeEnvironmentDeclarationV0 {
                declaration_id: "user-important".to_string(),
                selector: ".a".to_string(),
                property: "color".to_string(),
                value: "green".to_string(),
                origin: omena_cascade::CascadeOriginV0::User,
                important: true,
                layer_rank: None,
                scope_proximity: None,
                source_order: 0,
            }],
        };
        let result = evaluate_transform_winner_equality(
            TransformPassKind::RuleMerging,
            &input_ir,
            &output_ir,
            &[mutation_span(input, output)],
            StyleDialect::Css,
            TransformWinnerEqualityContextV0 {
                input_scope: SemanticObservationScopeV0::default(),
                output_scope: SemanticObservationScopeV0::default(),
                cascade_environment: Some(&environment),
            },
        );

        assert!(result.obligations.iter().any(|obligation| matches!(
            &obligation.observation,
            TransformWinnerEqualityObservationV0::ObservedEqual { input, output, .. }
                if input.winner.id == "user-important" && output.winner.id == "user-important"
        )));
    }

    #[test]
    fn complete_environment_uses_a_shared_source_order_coordinate() {
        let input = ".a { color: red; }";
        let output = ".a { color: blue; }";
        let input_ir = lower_transform_ir_from_source(input, StyleDialect::Css, "order-input");
        let output_ir = lower_transform_ir_from_source(output, StyleDialect::Css, "order-output");
        let environment = TransformCascadeEnvironmentV0 {
            stylesheet_source_order_base: 10,
            declarations: vec![TransformCascadeEnvironmentDeclarationV0 {
                declaration_id: "later-author-rule".to_string(),
                selector: ".a".to_string(),
                property: "color".to_string(),
                value: "green".to_string(),
                origin: omena_cascade::CascadeOriginV0::Author,
                important: false,
                layer_rank: None,
                scope_proximity: None,
                source_order: 20,
            }],
        };
        let result = evaluate_transform_winner_equality(
            TransformPassKind::RuleMerging,
            &input_ir,
            &output_ir,
            &[mutation_span(input, output)],
            StyleDialect::Css,
            TransformWinnerEqualityContextV0 {
                input_scope: SemanticObservationScopeV0::default(),
                output_scope: SemanticObservationScopeV0::default(),
                cascade_environment: Some(&environment),
            },
        );

        assert!(result.obligations.iter().any(|obligation| matches!(
            &obligation.observation,
            TransformWinnerEqualityObservationV0::ObservedEqual { input, output, .. }
                if input.winner.id == "later-author-rule"
                    && output.winner.id == "later-author-rule"
        )));
    }
}
