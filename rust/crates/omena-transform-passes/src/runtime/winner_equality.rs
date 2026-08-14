//! Observational cascade-winner comparisons for admitted transform mutations.

use std::collections::{BTreeMap, BTreeSet};

use omena_cascade::{
    CascadeDeclaration, CascadeLevel, CascadeValue, CascadeWinnerAxisV0, ElementSignature,
    FirstWitnessManagerConfigV0, FirstWitnessManagerV0, GuardedCascadeCandidateV0,
    GuardedCascadeConditionAtomV0, GuardedCascadeFragmentV0, GuardedCascadeSpecificityExactnessV0,
    GuardedCascadeWinnerAuthorityErrorV0, GuardedCascadeWinnerAuthorityV0,
    GuardedCascadeWinnerFunctionEqualityDecisionV0, GuardedCascadeWinnerFunctionEqualityRefusalV0,
    GuardedCascadeWinnerPlaneAnswerV0, LayerOrdinal, LayerRank, OpenWorldTieEvidence,
    SelectorMatchVerdict, SpecificityExactnessV0, at_rule_nesting_dfs_paths_v0,
    at_rule_nesting_order_for_fragment_v0, build_guarded_cascade_winner_v0,
    cascade_driven_levels_v0, cascade_driven_winner_axes_v0, cascade_level_for_origin,
    cascade_property, compare_guarded_cascade_winner_functions_v0,
    evaluate_guarded_cascade_winner_v0, guarded_cascade_winner_is_total_v0, normalized_layer_rank,
    parse_simple_selector_signature, reconcile_guarded_cascade_winner_planes_v0,
    selector_match_witness,
};
use omena_parser::{StyleDialect, css_keyword};
use omena_semantic::{
    LayerBindingResolutionV0, layer_ordinal_for_byte_span, summarize_style_layer_order_from_source,
};
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

#[derive(Debug)]
struct WinnerForPairV0 {
    witness: Option<TransformWinnerEqualityWitnessV0>,
    conditional_context_open: bool,
}

#[derive(Debug, Clone)]
struct GuardedCandidateSeedV0 {
    stable_identity: String,
    scenario_witness_id: String,
    element_signature: String,
    property: String,
    cascade_key: omena_cascade::CascadeKey,
    conditions: Vec<GuardedCascadeConditionAtomV0>,
}

#[derive(Debug)]
struct GuardedWinnerFunctionEvaluationV0 {
    decision: GuardedCascadeWinnerFunctionEqualityDecisionV0,
    input_canonical_answer: GuardedCascadeWinnerPlaneAnswerV0,
    output_canonical_answer: GuardedCascadeWinnerPlaneAnswerV0,
    input_scenario_ids: BTreeMap<(String, omena_cascade::CascadeKey), u32>,
    output_scenario_ids: BTreeMap<(String, omena_cascade::CascadeKey), u32>,
}

#[derive(Debug)]
struct GuardedEqualPlanesV0 {
    authority: GuardedCascadeWinnerAuthorityV0,
    input_canonical_answer: GuardedCascadeWinnerPlaneAnswerV0,
    output_canonical_answer: GuardedCascadeWinnerPlaneAnswerV0,
    input_scenario_ids: BTreeMap<(String, omena_cascade::CascadeKey), u32>,
    output_scenario_ids: BTreeMap<(String, omena_cascade::CascadeKey), u32>,
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
        let guarded_evaluation =
            if input.conditional_context_open || output.conditional_context_open {
                guarded_winner_function_equality_for_pair(
                    &pair,
                    input_candidates.as_slice(),
                    output_candidates.as_slice(),
                    &input_layers,
                    &output_layers,
                    context.cascade_environment,
                )
            } else {
                None
            };
        let guarded_equal = match guarded_evaluation {
            Some(GuardedWinnerFunctionEvaluationV0 {
                decision: GuardedCascadeWinnerFunctionEqualityDecisionV0::Equal { authority },
                input_canonical_answer,
                output_canonical_answer,
                input_scenario_ids,
                output_scenario_ids,
            }) => Some(GuardedEqualPlanesV0 {
                authority,
                input_canonical_answer,
                output_canonical_answer,
                input_scenario_ids,
                output_scenario_ids,
            }),
            Some(GuardedWinnerFunctionEvaluationV0 {
                decision:
                    GuardedCascadeWinnerFunctionEqualityDecisionV0::Refused {
                        refusal:
                            GuardedCascadeWinnerFunctionEqualityRefusalV0::CanonicalRootsDiffer {
                                input_root,
                                output_root,
                            },
                        ..
                    },
                ..
            }) => {
                reasons.push(TransformWinnerEqualityAbsenceV0 {
                    axis: TransformWinnerEqualityAxisV0::CascadeLevel,
                    reason: TransformWinnerEqualityAbsenceReasonV0::GuardedWinnerFunctionsDiffer {
                        input_root,
                        output_root,
                    },
                });
                None
            }
            Some(_) | None if input.conditional_context_open || output.conditional_context_open => {
                reasons.push(TransformWinnerEqualityAbsenceV0 {
                    axis: TransformWinnerEqualityAxisV0::CascadeLevel,
                    reason: TransformWinnerEqualityAbsenceReasonV0::WinnerNotDefinite,
                });
                None
            }
            Some(_) | None => None,
        };
        deduplicate_absences(&mut reasons);

        let observation = if !reasons.is_empty() {
            TransformWinnerEqualityObservationV0::Absent { reasons }
        } else if let (Some(input), Some(output)) = (input.witness, output.witness) {
            if let Some(guarded) = guarded_equal {
                guarded_winner_observation_from_reconciled_planes(
                    axes.clone(),
                    input,
                    output,
                    guarded,
                )
            } else if winner_witnesses_are_observationally_equal(&input, &output) {
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
            | TransformWinnerEqualityObservationV0::ObservedGuardedEqual { .. }
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

fn guarded_winner_observation_from_reconciled_planes(
    axes: Vec<TransformWinnerEqualityAxisV0>,
    input: TransformWinnerEqualityWitnessV0,
    output: TransformWinnerEqualityWitnessV0,
    guarded: GuardedEqualPlanesV0,
) -> TransformWinnerEqualityObservationV0 {
    let Some(input_scenario_answer) = scenario_plane_answer(&input, &guarded.input_scenario_ids)
    else {
        return guarded_winner_mapping_absence();
    };
    let Some(output_scenario_answer) = scenario_plane_answer(&output, &guarded.output_scenario_ids)
    else {
        return guarded_winner_mapping_absence();
    };
    let mut reasons = Vec::new();
    if let Err(error) = reconcile_guarded_cascade_winner_planes_v0(
        &guarded.authority,
        guarded.input_canonical_answer,
        input_scenario_answer,
    ) {
        reasons.push(guarded_plane_disagreement_absence("input", error));
    }
    if let Err(error) = reconcile_guarded_cascade_winner_planes_v0(
        &guarded.authority,
        guarded.output_canonical_answer,
        output_scenario_answer,
    ) {
        reasons.push(guarded_plane_disagreement_absence("output", error));
    }
    if !reasons.is_empty() {
        return TransformWinnerEqualityObservationV0::Absent { reasons };
    }
    if winner_witnesses_are_observationally_equal(&input, &output) {
        TransformWinnerEqualityObservationV0::ObservedGuardedEqual {
            axes,
            input,
            output,
            authority: guarded.authority,
        }
    } else {
        TransformWinnerEqualityObservationV0::ObservedDifferent {
            axes,
            input,
            output,
        }
    }
}

fn scenario_plane_answer(
    witness: &TransformWinnerEqualityWitnessV0,
    declaration_ids: &BTreeMap<(String, omena_cascade::CascadeKey), u32>,
) -> Option<GuardedCascadeWinnerPlaneAnswerV0> {
    declaration_ids
        .get(&(witness.winner.id.clone(), witness.winner.key))
        .copied()
        .map(|declaration_id| GuardedCascadeWinnerPlaneAnswerV0::Declaration { declaration_id })
}

fn guarded_winner_mapping_absence() -> TransformWinnerEqualityObservationV0 {
    TransformWinnerEqualityObservationV0::Absent {
        reasons: vec![TransformWinnerEqualityAbsenceV0 {
            axis: TransformWinnerEqualityAxisV0::CascadeLevel,
            reason: TransformWinnerEqualityAbsenceReasonV0::WinnerNotDefinite,
        }],
    }
}

fn guarded_plane_disagreement_absence(
    side: &'static str,
    error: GuardedCascadeWinnerAuthorityErrorV0,
) -> TransformWinnerEqualityAbsenceV0 {
    let reason = match error {
        GuardedCascadeWinnerAuthorityErrorV0::InFragmentPlaneDisagreement {
            canonical_mtbdd,
            scenario_sweep,
        } => TransformWinnerEqualityAbsenceReasonV0::GuardedWinnerPlaneDisagreement {
            side,
            canonical_mtbdd,
            scenario_sweep,
        },
        _ => TransformWinnerEqualityAbsenceReasonV0::WinnerNotDefinite,
    };
    TransformWinnerEqualityAbsenceV0 {
        axis: TransformWinnerEqualityAxisV0::CascadeLevel,
        reason,
    }
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
) -> WinnerForPairV0 {
    let mut declarations = Vec::new();
    let mut matched_ordinal = 0usize;
    let mut conditional_context_open = false;
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
                if witness.specificity_exactness == SpecificityExactnessV0::Inexact {
                    reasons.push(TransformWinnerEqualityAbsenceV0 {
                        axis: TransformWinnerEqualityAxisV0::Specificity,
                        reason: TransformWinnerEqualityAbsenceReasonV0::SpecificityInexact,
                    });
                }
                reasons.push(TransformWinnerEqualityAbsenceV0 {
                    axis: TransformWinnerEqualityAxisV0::Specificity,
                    reason: TransformWinnerEqualityAbsenceReasonV0::WinnerNotDefinite,
                });
                continue;
            }
            SelectorMatchVerdict::Yes => {}
        }
        if conditional_context_is_open(candidate.context_key.as_str()) {
            conditional_context_open = true;
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
                reason: TransformWinnerEqualityAbsenceReasonV0::SpecificityInexact,
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
            key: omena_cascade::CascadeKey::new(level, layer_rank, 0, specificity, source_order),
            open_world_tie_evidence: OpenWorldTieEvidence::NONE,
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
                    if witness.specificity_exactness == SpecificityExactnessV0::Inexact {
                        reasons.push(TransformWinnerEqualityAbsenceV0 {
                            axis: TransformWinnerEqualityAxisV0::Specificity,
                            reason: TransformWinnerEqualityAbsenceReasonV0::SpecificityInexact,
                        });
                    }
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
                    reason: TransformWinnerEqualityAbsenceReasonV0::WinnerNotDefinite,
                });
                continue;
            };
            if signature.specificity_exactness == SpecificityExactnessV0::Inexact {
                reasons.push(TransformWinnerEqualityAbsenceV0 {
                    axis: TransformWinnerEqualityAxisV0::Specificity,
                    reason: TransformWinnerEqualityAbsenceReasonV0::SpecificityInexact,
                });
            }
            let layer_ordinal = match declaration.layer_rank {
                Some(rank) => {
                    let Some(ordinal) = LayerOrdinal::new(rank) else {
                        reasons.push(TransformWinnerEqualityAbsenceV0 {
                            axis: TransformWinnerEqualityAxisV0::LayerRank,
                            reason: TransformWinnerEqualityAbsenceReasonV0::DriverUnavailable {
                                level: None,
                            },
                        });
                        continue;
                    };
                    Some(ordinal)
                }
                None => None,
            };
            let layer_rank = normalized_layer_rank(declaration.important, layer_ordinal);
            declarations.push(CascadeDeclaration {
                id: declaration.declaration_id.clone(),
                property: declaration.property.clone(),
                value: CascadeValue::Literal(declaration.value.clone()),
                key: omena_cascade::CascadeKey::new(
                    level,
                    layer_rank,
                    declaration.scope_proximity.unwrap_or(0),
                    signature.specificity,
                    declaration.source_order,
                ),
                open_world_tie_evidence: OpenWorldTieEvidence::NONE,
                specificity_exactness: signature.specificity_exactness,
            });
        }
    }

    WinnerForPairV0 {
        witness: TransformWinnerEqualityWitnessV0::from_cascade_outcome(&cascade_property(
            declarations,
            pair.property.as_str(),
        )),
        conditional_context_open,
    }
}

fn guarded_winner_function_equality_for_pair(
    pair: &TransformWinnerEqualityAffectedPairV0,
    input_candidates: &[SemanticCascadeCandidateV0],
    output_candidates: &[SemanticCascadeCandidateV0],
    input_layers: &omena_semantic::StyleLayerIndexV0,
    output_layers: &omena_semantic::StyleLayerIndexV0,
    cascade_environment: Option<&TransformCascadeEnvironmentV0>,
) -> Option<GuardedWinnerFunctionEvaluationV0> {
    let element_signature = format!("{:?}", pair.element_signature);
    let stylesheet_source_order_base = cascade_environment
        .map(|environment| environment.stylesheet_source_order_base)
        .unwrap_or_default();
    let mut input = guarded_candidate_seeds_for_pair(
        pair,
        input_candidates,
        input_layers,
        element_signature.as_str(),
        stylesheet_source_order_base,
    )?;
    let mut output = guarded_candidate_seeds_for_pair(
        pair,
        output_candidates,
        output_layers,
        element_signature.as_str(),
        stylesheet_source_order_base,
    )?;
    if let Some(environment) = cascade_environment {
        let environment_seeds = guarded_environment_candidate_seeds_for_pair(
            pair,
            environment,
            element_signature.as_str(),
        )?;
        input.extend(environment_seeds.iter().cloned());
        output.extend(environment_seeds);
    }
    let input_alphabet = guarded_condition_alphabet(input.as_slice());
    let output_alphabet = guarded_condition_alphabet(output.as_slice());
    if input_alphabet != output_alphabet {
        return None;
    }
    let identities = input
        .iter()
        .chain(output.iter())
        .map(|seed| seed.stable_identity.clone())
        .collect::<BTreeSet<_>>();
    let declaration_ids = identities
        .into_iter()
        .enumerate()
        .map(|(index, identity)| u32::try_from(index).ok().map(|index| (identity, index)))
        .collect::<Option<BTreeMap<_, _>>>()?;
    let input_scenario_ids = guarded_scenario_declaration_ids(&input, &declaration_ids)?;
    let output_scenario_ids = guarded_scenario_declaration_ids(&output, &declaration_ids)?;
    let input_fragment =
        guarded_fragment_from_seeds(input_alphabet.iter().cloned(), input, &declaration_ids)?;
    let output_fragment =
        guarded_fragment_from_seeds(output_alphabet.iter().cloned(), output, &declaration_ids)?;
    let order = at_rule_nesting_order_for_fragment_v0(&input_fragment).ok()?;
    let mut manager = FirstWitnessManagerV0::new(order, FirstWitnessManagerConfigV0::default());
    manager
        .register_declaration_terminals(declaration_ids.values().copied())
        .ok()?;
    let input_root = build_guarded_cascade_winner_v0(&mut manager, &input_fragment).ok()?;
    let output_root = build_guarded_cascade_winner_v0(&mut manager, &output_fragment).ok()?;
    let all_conditions_active = vec![true; input_alphabet.len()];
    let input_canonical_answer = guarded_plane_answer(
        evaluate_guarded_cascade_winner_v0(&manager, input_root, &all_conditions_active).ok()?,
    );
    let output_canonical_answer = guarded_plane_answer(
        evaluate_guarded_cascade_winner_v0(&manager, output_root, &all_conditions_active).ok()?,
    );
    #[cfg(test)]
    let output_canonical_answer =
        if std::env::var_os("OMENA_G122_INJECT_GUARDED_PLANE_DISAGREEMENT").is_some() {
            GuardedCascadeWinnerPlaneAnswerV0::NoWinner
        } else {
            output_canonical_answer
        };
    let winner_defined_for_all_assignments =
        guarded_cascade_winner_is_total_v0(&manager, input_root).ok()?
            && guarded_cascade_winner_is_total_v0(&manager, output_root).ok()?;
    #[cfg(test)]
    let decision = if std::env::var_os("OMENA_G122_INJECT_ACCEPT_DIFFERENT_WINNER_ROOTS").is_some()
    {
        compare_guarded_cascade_winner_functions_v0(
            input_fragment.predicate(),
            input_root,
            input_root,
            guarded_cascade_winner_is_total_v0(&manager, input_root).ok()?,
        )
    } else {
        compare_guarded_cascade_winner_functions_v0(
            input_fragment.predicate(),
            input_root,
            output_root,
            winner_defined_for_all_assignments,
        )
    };
    #[cfg(not(test))]
    let decision = compare_guarded_cascade_winner_functions_v0(
        input_fragment.predicate(),
        input_root,
        output_root,
        winner_defined_for_all_assignments,
    );
    Some(GuardedWinnerFunctionEvaluationV0 {
        decision,
        input_canonical_answer,
        output_canonical_answer,
        input_scenario_ids,
        output_scenario_ids,
    })
}

fn guarded_scenario_declaration_ids(
    seeds: &[GuardedCandidateSeedV0],
    declaration_ids: &BTreeMap<String, u32>,
) -> Option<BTreeMap<(String, omena_cascade::CascadeKey), u32>> {
    seeds
        .iter()
        .map(|seed| {
            declaration_ids
                .get(seed.stable_identity.as_str())
                .copied()
                .map(|declaration_id| {
                    (
                        (seed.scenario_witness_id.clone(), seed.cascade_key),
                        declaration_id,
                    )
                })
        })
        .collect()
}

fn guarded_plane_answer(winner: Option<u32>) -> GuardedCascadeWinnerPlaneAnswerV0 {
    winner.map_or(
        GuardedCascadeWinnerPlaneAnswerV0::NoWinner,
        |declaration_id| GuardedCascadeWinnerPlaneAnswerV0::Declaration { declaration_id },
    )
}

fn guarded_candidate_seeds_for_pair(
    pair: &TransformWinnerEqualityAffectedPairV0,
    candidates: &[SemanticCascadeCandidateV0],
    layer_index: &omena_semantic::StyleLayerIndexV0,
    element_signature: &str,
    stylesheet_source_order_base: u32,
) -> Option<Vec<GuardedCandidateSeedV0>> {
    let mut prepared = Vec::new();
    let mut matched_ordinal = 0usize;
    for candidate in candidates
        .iter()
        .filter(|candidate| css_keyword(candidate.property.as_str()).equals(pair.property.as_str()))
    {
        let witness = selector_match_witness(candidate.selector.as_str(), &pair.element_signature);
        match witness.verdict {
            SelectorMatchVerdict::No => continue,
            SelectorMatchVerdict::Maybe => return None,
            SelectorMatchVerdict::Yes => {}
        }
        let signature = parse_simple_selector_signature(candidate.selector.as_str())?;
        if signature.specificity_exactness != SpecificityExactnessV0::Exact {
            return None;
        }
        let mut layer_reasons = Vec::new();
        let layer_rank = layer_rank_for_candidate(candidate, layer_index, &mut layer_reasons);
        if !layer_reasons.is_empty() {
            return None;
        }
        let source_order =
            stylesheet_source_order_base.checked_add(u32::try_from(matched_ordinal).ok()?)?;
        matched_ordinal += 1;
        let context = if candidate.context_key.trim().is_empty() {
            Vec::new()
        } else {
            candidate
                .context_key
                .split('|')
                .map(|component| component.trim().to_string())
                .collect()
        };
        prepared.push((
            candidate,
            signature.specificity,
            layer_rank,
            source_order,
            context,
        ));
    }
    if prepared.is_empty() {
        return None;
    }
    let contexts = prepared
        .iter()
        .map(|(_, _, _, _, context)| context.clone())
        .collect::<Vec<_>>();
    let paths = at_rule_nesting_dfs_paths_v0(contexts.as_slice()).ok()?;
    let mut seeds = Vec::new();
    let mut occurrence_by_identity = BTreeMap::<String, usize>::new();
    for ((candidate, specificity, layer_rank, source_order, context), paths) in
        prepared.into_iter().zip(paths)
    {
        let scenario_witness_id = format!(
            "{}|{}|{}|{}",
            candidate.selector, candidate.property, candidate.value, candidate.important
        );
        let base_identity = format!("source|{scenario_witness_id}");
        let occurrence = occurrence_by_identity
            .entry(base_identity.clone())
            .or_default();
        let stable_identity = format!("{base_identity}|{occurrence}");
        *occurrence += 1;
        seeds.push(GuardedCandidateSeedV0 {
            stable_identity,
            scenario_witness_id,
            element_signature: element_signature.to_string(),
            property: candidate.property.clone(),
            cascade_key: omena_cascade::CascadeKey::new(
                cascade_level_for_origin(
                    omena_cascade::CascadeOriginV0::Author,
                    candidate.important,
                ),
                layer_rank,
                0,
                specificity,
                source_order,
            ),
            conditions: guarded_conditions_from_context(context.as_slice(), paths.as_slice())?,
        });
    }
    Some(seeds)
}

fn guarded_environment_candidate_seeds_for_pair(
    pair: &TransformWinnerEqualityAffectedPairV0,
    environment: &TransformCascadeEnvironmentV0,
    element_signature: &str,
) -> Option<Vec<GuardedCandidateSeedV0>> {
    let mut seeds = Vec::new();
    for declaration in environment.declarations.iter().filter(|declaration| {
        css_keyword(declaration.property.as_str()).equals(pair.property.as_str())
    }) {
        let witness =
            selector_match_witness(declaration.selector.as_str(), &pair.element_signature);
        match witness.verdict {
            SelectorMatchVerdict::No => continue,
            SelectorMatchVerdict::Maybe => return None,
            SelectorMatchVerdict::Yes => {}
        }
        let signature = parse_simple_selector_signature(declaration.selector.as_str())?;
        if signature.specificity_exactness != SpecificityExactnessV0::Exact
            || declaration.scope_proximity.unwrap_or(0) != 0
        {
            return None;
        }
        let layer_ordinal = match declaration.layer_rank {
            Some(rank) => Some(LayerOrdinal::new(rank)?),
            None => None,
        };
        seeds.push(GuardedCandidateSeedV0 {
            stable_identity: format!("environment|{}", declaration.declaration_id),
            scenario_witness_id: declaration.declaration_id.clone(),
            element_signature: element_signature.to_string(),
            property: declaration.property.clone(),
            cascade_key: omena_cascade::CascadeKey::new(
                cascade_level_for_origin(declaration.origin, declaration.important),
                normalized_layer_rank(declaration.important, layer_ordinal),
                0,
                signature.specificity,
                declaration.source_order,
            ),
            conditions: Vec::new(),
        });
    }
    Some(seeds)
}

fn guarded_conditions_from_context(
    context: &[String],
    paths: &[Vec<u32>],
) -> Option<Vec<GuardedCascadeConditionAtomV0>> {
    (context.len() == paths.len()).then_some(())?;
    context
        .iter()
        .zip(paths)
        .map(|(component, path)| {
            let component = component.trim();
            let numeric = component
                .chars()
                .any(|character| character.is_ascii_digit());
            if css_keyword(component).strip_prefix("@media").is_some() {
                Some(GuardedCascadeConditionAtomV0::media(
                    component,
                    path.iter().copied(),
                    numeric,
                ))
            } else if css_keyword(component).strip_prefix("@supports").is_some() {
                Some(GuardedCascadeConditionAtomV0::supports(
                    component,
                    path.iter().copied(),
                    numeric,
                ))
            } else if css_keyword(component).strip_prefix("@container").is_some() {
                Some(GuardedCascadeConditionAtomV0::container(
                    component,
                    path.iter().copied(),
                ))
            } else {
                Some(GuardedCascadeConditionAtomV0::structural_pseudo(component))
            }
        })
        .collect()
}

fn guarded_condition_alphabet(seeds: &[GuardedCandidateSeedV0]) -> BTreeSet<String> {
    seeds
        .iter()
        .flat_map(|seed| seed.conditions.iter())
        .map(|condition| condition.atom().to_string())
        .collect()
}

fn guarded_fragment_from_seeds(
    alphabet: impl IntoIterator<Item = String>,
    seeds: Vec<GuardedCandidateSeedV0>,
    declaration_ids: &BTreeMap<String, u32>,
) -> Option<GuardedCascadeFragmentV0<omena_cascade::CascadeKey>> {
    let candidates = seeds
        .into_iter()
        .map(|seed| {
            Some(GuardedCascadeCandidateV0::new(
                *declaration_ids.get(seed.stable_identity.as_str())?,
                seed.element_signature,
                seed.property,
                seed.cascade_key,
                GuardedCascadeSpecificityExactnessV0::Exact,
                0,
                seed.conditions,
            ))
        })
        .collect::<Option<Vec<_>>>()?;
    GuardedCascadeFragmentV0::admit(alphabet, candidates).ok()
}

fn layer_rank_for_candidate(
    candidate: &SemanticCascadeCandidateV0,
    layer_index: &omena_semantic::StyleLayerIndexV0,
    reasons: &mut Vec<TransformWinnerEqualityAbsenceV0>,
) -> LayerRank {
    let ordinal = match layer_ordinal_for_byte_span(
        layer_index,
        candidate.source_span_start,
        candidate.source_span_end,
    ) {
        LayerBindingResolutionV0::Resolved(ordinal) => ordinal,
        LayerBindingResolutionV0::TopologyIncomplete { .. } => {
            reasons.push(TransformWinnerEqualityAbsenceV0 {
                axis: TransformWinnerEqualityAxisV0::LayerRank,
                reason: TransformWinnerEqualityAbsenceReasonV0::DriverUnavailable { level: None },
            });
            None
        }
    };
    normalized_layer_rank(candidate.important, ordinal)
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
            TransformWinnerEqualityObservationV0::ObservedEqual { .. }
            | TransformWinnerEqualityObservationV0::ObservedGuardedEqual { .. } => Vec::new(),
        })
        .collect::<Vec<_>>();
    deduplicate_absences(&mut reasons);
    if reasons.is_empty()
        && obligations.iter().all(|obligation| {
            matches!(
                obligation.observation,
                TransformWinnerEqualityObservationV0::ObservedEqual { .. }
                    | TransformWinnerEqualityObservationV0::ObservedGuardedEqual { .. }
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
    fn guarded_conditions_classify_mixed_case_at_rules_through_keyword_authority()
    -> Result<(), &'static str> {
        let context = [
            "@MeDiA (min-width: 1px)".to_string(),
            "@SuPpOrTs (display: grid)".to_string(),
            "@CoNtAiNeR card (min-width: 1px)".to_string(),
        ];
        let paths = at_rule_nesting_dfs_paths_v0(&[context.to_vec()])
            .map_err(|_| "mixed-case guarded paths")?;
        let conditions = guarded_conditions_from_context(&context, &paths[0])
            .ok_or("mixed-case guarded conditions")?;

        assert_eq!(
            conditions
                .iter()
                .map(|condition| condition.kind())
                .collect::<Vec<_>>(),
            [
                omena_cascade::GuardedCascadeConditionKindV0::Media,
                omena_cascade::GuardedCascadeConditionKindV0::Supports,
                omena_cascade::GuardedCascadeConditionKindV0::Container,
            ]
        );
        Ok(())
    }

    #[test]
    fn equal_guarded_winner_roots_replace_the_conditional_absence_with_authority()
    -> Result<(), &'static str> {
        let input = "@media (min-width: 1px) { .a { color: red; } } @media (min-width: 1px) { .b { color: blue; } }";
        let output = "@media (min-width: 1px) { .a { color: red; } .b { color: blue; } }";
        assert_ne!(
            input, output,
            "the GREEN arm must exercise a real rule merge"
        );
        let input_ir =
            lower_transform_ir_from_source(input, StyleDialect::Css, "conditional-input");
        let output_ir =
            lower_transform_ir_from_source(output, StyleDialect::Css, "conditional-output");
        let result = evaluate_transform_winner_equality(
            TransformPassKind::RuleMerging,
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
        let color_observation = result
            .obligations
            .iter()
            .find(|obligation| obligation.affected_pair.property == "color")
            .map(|obligation| &obligation.observation);
        eprintln!("GUARDED_RECONCILED_OBSERVATION={color_observation:?}");

        assert!(result.unresolved_reasons.is_empty());
        let (obligation, authority) = result
            .obligations
            .iter()
            .find_map(|obligation| match &obligation.observation {
                TransformWinnerEqualityObservationV0::ObservedGuardedEqual {
                    authority, ..
                } => Some((obligation, authority)),
                _ => None,
            })
            .ok_or("guarded winner authority carrier missing")?;
        assert!(authority.root.node_id() > 0);
        let fragment = match &authority.rule {
            omena_cascade::GuardedCascadeWinnerAuthorityRuleV0::CanonicalMtbddInsideFragment {
                fragment,
                ..
            } => fragment,
            _ => return Err("inside-fragment MTBDD authority expected"),
        };
        assert_eq!(
            fragment.element_signature,
            format!("{:?}", obligation.affected_pair.element_signature)
        );
        assert_eq!(fragment.property, "color");
        assert_eq!(
            fragment.condition_alphabet,
            vec!["@media (min-width: 1px)".to_string()]
        );
        assert!(!result.unresolved_reasons.iter().any(|absence| {
            absence.reason == TransformWinnerEqualityAbsenceReasonV0::SpecificityInexact
        }));
        Ok(())
    }

    #[test]
    fn correlated_numeric_root_inequality_is_a_typed_refusal() -> Result<(), &'static str> {
        let input = ".a { color: black; } @media (min-width: 768px) { .a { color: red; } } @media (min-width: 1200px) { .a { color: blue; } }";
        let output = ".a { color: black; } @media (min-width: 768px) { .a { color: red; } @media (min-width: 1200px) { .a { color: blue; } } }";
        let input_ir = lower_transform_ir_from_source(input, StyleDialect::Css, "numeric-input");
        let output_ir = lower_transform_ir_from_source(output, StyleDialect::Css, "numeric-output");
        let result = evaluate_transform_winner_equality(
            TransformPassKind::RuleMerging,
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

        let refusal = result.obligations.iter().find_map(|obligation| {
            let TransformWinnerEqualityObservationV0::Absent { reasons } = &obligation.observation
            else {
                return None;
            };
            reasons.iter().find_map(|absence| match absence.reason {
                TransformWinnerEqualityAbsenceReasonV0::GuardedWinnerFunctionsDiffer {
                    input_root,
                    output_root,
                } => Some((input_root, output_root)),
                _ => None,
            })
        });
        let (input_root, output_root) =
            refusal.ok_or("typed guarded-root inequality refusal missing")?;
        assert_ne!(input_root, output_root);
        assert!(result.obligations.iter().all(|obligation| !matches!(
            obligation.observation,
            TransformWinnerEqualityObservationV0::ObservedGuardedEqual { .. }
        )));
        Ok(())
    }

    #[test]
    fn guarded_fixture_corpus_reduces_conditional_absences_to_outside_fragment_only() {
        let fixtures = [
            "@media (min-width: 1px) { .a { color: red; } }",
            "@supports (display: grid) { .a { color: red; } }",
            "@container card (min-width: 1px) { .a { color: red; } }",
        ];
        let exit_absence_count = fixtures
            .iter()
            .filter(|source| {
                compare_transform_winner_equality_for_conformance_v0(
                    source,
                    source,
                    StyleDialect::Css,
                    TransformPassKind::RuleMerging,
                )
                .iter()
                .any(|obligation| {
                    matches!(
                        &obligation.observation,
                        TransformWinnerEqualityObservationV0::Absent { reasons }
                            if reasons.iter().any(|absence| {
                                absence.reason
                                    == TransformWinnerEqualityAbsenceReasonV0::WinnerNotDefinite
                            })
                    )
                })
            })
            .count();

        // The pre-S5 implementation emitted this absence for all three rows.
        // Media and supports are admitted now; container remains outside v0.
        assert_eq!(exit_absence_count, 1);
    }

    #[test]
    fn inexact_environment_selector_reports_specificity_cause() {
        let source_selector = ":is(.a)";
        let environment_selector = ":is(:unknown(.a), .a)";
        let source = ":is(.a) { color: red; }";
        assert_eq!(
            parse_simple_selector_signature(source_selector)
                .map(|signature| signature.specificity_exactness),
            Some(SpecificityExactnessV0::Exact)
        );
        assert_eq!(
            parse_simple_selector_signature(environment_selector)
                .map(|signature| signature.specificity_exactness),
            Some(SpecificityExactnessV0::Inexact)
        );
        let input_ir =
            lower_transform_ir_from_source(source, StyleDialect::Css, "environment-input");
        let output_ir =
            lower_transform_ir_from_source(source, StyleDialect::Css, "environment-output");
        let environment = TransformCascadeEnvironmentV0 {
            stylesheet_source_order_base: 0,
            declarations: vec![TransformCascadeEnvironmentDeclarationV0 {
                declaration_id: "environment-inexact".to_string(),
                selector: environment_selector.to_string(),
                property: "color".to_string(),
                value: "blue".to_string(),
                origin: omena_cascade::CascadeOriginV0::Author,
                important: false,
                layer_rank: None,
                scope_proximity: None,
                source_order: 1,
            }],
        };
        let result = evaluate_transform_winner_equality(
            TransformPassKind::RuleMerging,
            &input_ir,
            &output_ir,
            &[mutation_span(source, source)],
            StyleDialect::Css,
            TransformWinnerEqualityContextV0 {
                input_scope: SemanticObservationScopeV0::default(),
                output_scope: SemanticObservationScopeV0::default(),
                cascade_environment: Some(&environment),
            },
        );

        assert!(result.unresolved_reasons.iter().any(|absence| {
            absence.axis == TransformWinnerEqualityAxisV0::Specificity
                && absence.reason == TransformWinnerEqualityAbsenceReasonV0::SpecificityInexact
        }));
    }

    #[test]
    fn inexact_candidate_reports_specificity_cause_inside_winner_selection() {
        let selector = ":is(:unknown(.a), .b)";
        assert_eq!(
            parse_simple_selector_signature(selector)
                .map(|signature| signature.specificity_exactness),
            Some(SpecificityExactnessV0::Inexact)
        );
        let pair = TransformWinnerEqualityAffectedPairV0 {
            element_signature: ElementSignature {
                tag: None,
                id: None,
                classes: BTreeSet::new(),
                attributes: BTreeSet::new(),
                pseudo_states: BTreeSet::from(["is".to_string()]),
                classes_are_exact: true,
                attributes_are_exact: true,
                pseudo_states_are_exact: true,
                tag_is_exact: true,
                id_is_exact: true,
            },
            property: "color".to_string(),
        };
        let candidates = vec![SemanticCascadeCandidateV0 {
            selector: selector.to_string(),
            property: "color".to_string(),
            value: "red".to_string(),
            important: false,
            source_span_start: 0,
            source_span_end: selector.len(),
            context_key: String::new(),
        }];
        let layer_index = summarize_style_layer_order_from_source("", StyleDialect::Css);
        let driven_levels = cascade_driven_levels_v0()
            .into_iter()
            .collect::<BTreeSet<_>>();
        let environment = TransformCascadeEnvironmentV0::default();
        let mut reasons = Vec::new();

        let winner = winner_for_pair(
            &pair,
            candidates.as_slice(),
            &layer_index,
            &driven_levels,
            &mut reasons,
            Some(&environment),
        );

        assert!(winner.witness.is_none());
        assert!(reasons.iter().any(|absence| {
            absence.axis == TransformWinnerEqualityAxisV0::Specificity
                && absence.reason == TransformWinnerEqualityAbsenceReasonV0::SpecificityInexact
        }));
    }

    #[test]
    fn unparsed_environment_selector_keeps_nonspecific_winner_absence() {
        let source = ".a { color: red; }";
        let input_ir = lower_transform_ir_from_source(source, StyleDialect::Css, "unparsed-input");
        let output_ir =
            lower_transform_ir_from_source(source, StyleDialect::Css, "unparsed-output");
        let environment = TransformCascadeEnvironmentV0 {
            stylesheet_source_order_base: 0,
            declarations: vec![TransformCascadeEnvironmentDeclarationV0 {
                declaration_id: "environment-unparsed".to_string(),
                selector: ".a, :unknown(".to_string(),
                property: "color".to_string(),
                value: "blue".to_string(),
                origin: omena_cascade::CascadeOriginV0::Author,
                important: false,
                layer_rank: None,
                scope_proximity: None,
                source_order: 1,
            }],
        };
        let result = evaluate_transform_winner_equality(
            TransformPassKind::RuleMerging,
            &input_ir,
            &output_ir,
            &[mutation_span(source, source)],
            StyleDialect::Css,
            TransformWinnerEqualityContextV0 {
                input_scope: SemanticObservationScopeV0::default(),
                output_scope: SemanticObservationScopeV0::default(),
                cascade_environment: Some(&environment),
            },
        );

        assert!(result.unresolved_reasons.iter().any(|absence| {
            absence.axis == TransformWinnerEqualityAxisV0::Specificity
                && absence.reason == TransformWinnerEqualityAbsenceReasonV0::WinnerNotDefinite
        }));
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
    fn functional_pseudo_reordering_reports_an_inexact_winner() {
        let input = ".foo { color: red; } :is(.foo) { color: blue; }";
        let output = ":is(.foo) { color: blue; } .foo { color: red; }";
        let obligations = compare_transform_winner_equality_for_conformance_v0(
            input,
            output,
            StyleDialect::Css,
            TransformPassKind::SelectorIsWhereCompression,
        );

        assert!(obligations.iter().any(|obligation| matches!(
            &obligation.observation,
            TransformWinnerEqualityObservationV0::Absent { reasons }
                if reasons.iter().any(|absence| {
                    absence.reason
                        == TransformWinnerEqualityAbsenceReasonV0::WinnerNotDefinite
                })
        )));
    }

    #[test]
    fn lossy_selector_pairs_do_not_mask_base_element_changes() {
        let cases = [
            (
                "[type=\"text\"] { color: red; } [type=\"number\"] { color: blue; }",
                "[type=\"text\"] { color: green; } [type=\"number\"] { color: blue; }",
            ),
            (
                ".button { color: red; } .button::before { color: blue; }",
                ".button { color: green; } .button::before { color: blue; }",
            ),
        ];

        for (input, output) in cases {
            let obligations = compare_transform_winner_equality_for_conformance_v0(
                input,
                output,
                StyleDialect::Css,
                TransformPassKind::ColorCompression,
            );
            assert!(obligations.iter().any(|obligation| matches!(
                &obligation.observation,
                TransformWinnerEqualityObservationV0::Absent { reasons }
                    if reasons.iter().any(|absence| {
                        absence.reason
                            == TransformWinnerEqualityAbsenceReasonV0::WinnerNotDefinite
                    })
            )));
        }
    }

    #[test]
    fn definite_non_matches_remain_silent_in_winner_comparison() {
        let input = ".foo { color: red; } .bar { color: blue; }";
        let output = ".foo { color: red; } .bar { color: green; }";
        let obligations = compare_transform_winner_equality_for_conformance_v0(
            input,
            output,
            StyleDialect::Css,
            TransformPassKind::ColorCompression,
        );

        assert!(obligations.iter().all(|obligation| {
            !matches!(
                &obligation.observation,
                TransformWinnerEqualityObservationV0::Absent { reasons }
                    if reasons.iter().any(|absence| {
                        absence.reason
                            == TransformWinnerEqualityAbsenceReasonV0::WinnerNotDefinite
                    })
            )
        }));
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

    #[test]
    fn guarded_rule_merging_respects_the_stylesheet_source_order_base() {
        let input = "@media (min-width: 1px) { .a { color: red; } }";
        let output = "@media (min-width: 1px) { .a { color: blue; } }";
        let input_ir =
            lower_transform_ir_from_source(input, StyleDialect::Css, "guarded-base-input");
        let output_ir =
            lower_transform_ir_from_source(output, StyleDialect::Css, "guarded-base-output");
        let environment = TransformCascadeEnvironmentV0 {
            stylesheet_source_order_base: 10,
            declarations: vec![TransformCascadeEnvironmentDeclarationV0 {
                declaration_id: "earlier-environment-rule".to_string(),
                selector: ".a".to_string(),
                property: "color".to_string(),
                value: "green".to_string(),
                origin: omena_cascade::CascadeOriginV0::Author,
                important: false,
                layer_rank: None,
                scope_proximity: None,
                source_order: 5,
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
        let color_observation = result
            .obligations
            .iter()
            .find(|obligation| obligation.affected_pair.property == "color")
            .map(|obligation| &obligation.observation);
        eprintln!("GUARDED_BASE_OBSERVATION={color_observation:?}");

        assert!(result.obligations.iter().any(|obligation| matches!(
            &obligation.observation,
            TransformWinnerEqualityObservationV0::Absent { reasons }
                if reasons.iter().any(|absence| matches!(
                    absence.reason,
                    TransformWinnerEqualityAbsenceReasonV0::GuardedWinnerFunctionsDiffer { .. }
                ))
        )));
        assert!(result.obligations.iter().all(|obligation| !matches!(
            obligation.observation,
            TransformWinnerEqualityObservationV0::ObservedGuardedEqual { .. }
        )));

        let unguarded = compare_transform_winner_equality_for_conformance_v0(
            ".a { color: red; }",
            ".a { color: blue; }",
            StyleDialect::Css,
            TransformPassKind::RuleMerging,
        );
        assert!(unguarded.iter().any(|obligation| matches!(
            obligation.observation,
            TransformWinnerEqualityObservationV0::ObservedDifferent { .. }
        )));
    }
}
