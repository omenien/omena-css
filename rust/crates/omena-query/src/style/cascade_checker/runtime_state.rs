use std::collections::BTreeSet;

use omena_cascade::{
    CascadeDeclaration, CascadeKey, CascadeOriginV0, CascadeOutcome, CascadeValue,
    FirstWitnessManagerConfigV0, FirstWitnessManagerV0, GuardedCascadeCandidateV0,
    GuardedCascadeConditionAtomV0, GuardedCascadeFragmentV0, GuardedCascadeSpecificityExactnessV0,
    GuardedCascadeWinnerAuthorityV0, LayerOrdinal, OpenWorldTieEvidence, SelectorMatchVerdict,
    Specificity, SpecificityExactnessV0, StaticSupportsAssumptionV0, StaticSupportsEvalVerdictV0,
    at_rule_nesting_dfs_paths_v0, at_rule_nesting_order_for_fragment_v0,
    build_guarded_cascade_winner_v0, cascade_level_for_origin, cascade_property,
    compute_guarded_cascade_robustness_radius_v0, evaluate_static_supports_condition,
    guarded_cascade_perturbation_cost_model_v0, guarded_cascade_winner_authority_v0,
    guarded_cascade_winner_is_total_v0, normalized_layer_rank, parse_simple_selector_signature,
    selector_co_match_verdict,
};
use omena_query_checker_orchestrator::{
    OmenaCheckerCascadeDeclarationInputV0, OmenaCheckerCascadeEvaluationV0,
};
use omena_query_core::{
    AbstractClassValueV0, AbstractPropertyValueCandidateV0,
    narrow_abstract_property_value_for_cascade_branch, prefix_suffix_class_value,
};
use omena_syntax::{css_keyword, ident::class_selector_names};

#[cfg(test)]
use crate::types::runtime_state_result_certainty_labels;
use crate::{
    OMENA_QUERY_FRAGILE_GUARDED_WINNER_THRESHOLD_V0,
    style::substrate::summarize_omena_query_fragile_guarded_winner_v0,
    types::runtime_state_unknown_activation_declaration_id,
};

use super::super::{
    OmenaQueryCascadeLayerTopologyIncompleteV0, OmenaQueryInlineStyleRuntimeOverrideV0,
    OmenaQueryRuntimeStateDriverSummaryV0, OmenaQueryRuntimeStateScenarioEvidenceV0,
    OmenaQueryRuntimeStateScenarioV0, OmenaQueryRuntimeStateStaticBoundaryV0,
    OmenaQueryStaticConditionPruningEvidenceV0,
};

const RUNTIME_STATE_STATIC_BOUNDARY_KIND: &str = "staticValueAssumingNoRuntimeOverride";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ScenarioActivation {
    Active,
    Inactive,
    Unknown,
}

pub(super) fn summarize_query_runtime_state_for_evaluation(
    evaluation: &OmenaCheckerCascadeEvaluationV0,
    declarations: &[OmenaCheckerCascadeDeclarationInputV0],
    topology_incomplete_unresolved_count: Option<usize>,
) -> Option<OmenaQueryRuntimeStateScenarioEvidenceV0> {
    let anchor_id = evaluation.declaration_ids.first()?;
    let anchor = declarations
        .iter()
        .find(|declaration| declaration.declaration_id == *anchor_id)?;
    let selector_class_names = query_selector_class_names(anchor.selector.as_str());
    let candidate_declarations = declarations
        .iter()
        .filter(|declaration| declaration.property == anchor.property)
        .filter(|declaration| {
            query_runtime_selector_matches_anchor_classes(
                anchor.selector.as_str(),
                declaration.selector.as_str(),
            )
        })
        .collect::<Vec<_>>();
    if candidate_declarations.is_empty() {
        return None;
    }

    let pseudo_states = query_runtime_candidate_pseudo_states(candidate_declarations.as_slice());
    let (condition_contexts, static_condition_pruning) = query_runtime_candidate_condition_contexts(
        candidate_declarations.as_slice(),
        anchor.condition_context.as_slice(),
    );
    let mut scenarios = Vec::new();

    for condition_context in &condition_contexts {
        scenarios.push(query_runtime_state_scenario(
            anchor.property.as_str(),
            None,
            condition_context.as_slice(),
            candidate_declarations.as_slice(),
        ));
        for pseudo_state in &pseudo_states {
            scenarios.push(query_runtime_state_scenario(
                anchor.property.as_str(),
                Some(pseudo_state.as_str()),
                condition_context.as_slice(),
                candidate_declarations.as_slice(),
            ));
        }
    }

    let pseudo_scenario_count = scenarios
        .iter()
        .filter(|scenario| scenario.pseudo_state.is_some())
        .count();
    let media_scenario_count = condition_contexts
        .iter()
        .filter(|context| !context.is_empty())
        .count();
    let static_boundary = OmenaQueryRuntimeStateStaticBoundaryV0 {
        boundary_kind: RUNTIME_STATE_STATIC_BOUNDARY_KIND,
        static_value_assuming_no_runtime_override: true,
        tracks_dom_mutation: false,
        tracks_class_list_mutation: false,
    };
    let (confidence_tier, confidence_tier_within_modeled_environment) =
        query_runtime_state_confidence_tier(
            scenarios.as_slice(),
            &[],
            static_boundary.boundary_kind,
        );
    let guarded_winner_analysis = query_runtime_guarded_winner_analysis(
        anchor.selector.as_str(),
        anchor.property.as_str(),
        candidate_declarations.as_slice(),
        scenarios.as_slice(),
    );
    let (guarded_winner_authority, fragile_guarded_winner_diagnostics) = guarded_winner_analysis
        .map_or((None, Vec::new()), |(authority, diagnostics)| {
            (Some(authority), diagnostics)
        });
    let mut driver_summaries = vec![
        OmenaQueryRuntimeStateDriverSummaryV0 {
            driver: "pseudoStateScenarioSweep",
            status: if pseudo_scenario_count == 0 {
                "noRuntimePseudoStates"
            } else {
                "fixtureWitnessed"
            },
            scenario_count: pseudo_scenario_count,
            provenance: omena_query_evidence_graph_provenance![
                "omena-cascade.selector-signature",
                "omena-query.runtime-state-driver",
            ],
        },
        OmenaQueryRuntimeStateDriverSummaryV0 {
            driver: "inlineStyleCascadeJoin",
            status: "awaitingSourceFacts",
            scenario_count: 0,
            provenance: omena_query_evidence_graph_provenance![
                "omena-bridge.source-syntax-index",
                "omena-query.runtime-state-driver",
            ],
        },
        OmenaQueryRuntimeStateDriverSummaryV0 {
            driver: "mediaEnvironmentScenarioSweep",
            status: if media_scenario_count == 0 {
                "noConditionalEnvironment"
            } else {
                "fixtureWitnessed"
            },
            scenario_count: media_scenario_count,
            provenance: omena_query_evidence_graph_provenance![
                "omena-query.cascade-condition-context",
                "omena-query.runtime-state-driver",
            ],
        },
        OmenaQueryRuntimeStateDriverSummaryV0 {
            driver: "staticRuntimeOverrideBoundary",
            status: "documentedAnalyticalBoundary",
            scenario_count: scenarios.len(),
            provenance: omena_query_evidence_graph_provenance![
                "omena-query.static-runtime-boundary",
                "omena-query.runtime-state-driver",
            ],
        },
    ];
    if let Some(unresolved_count) = topology_incomplete_unresolved_count {
        driver_summaries.push(OmenaQueryRuntimeStateDriverSummaryV0 {
            driver: "cascadeLayerTopologyCompleteness",
            status: "incomplete",
            scenario_count: unresolved_count,
            provenance: omena_query_evidence_graph_provenance![
                "omena-semantic.layer-order",
                "omena-query.runtime-state-driver",
            ],
        });
    }
    Some(OmenaQueryRuntimeStateScenarioEvidenceV0 {
        schema_version: "0",
        product: "omena-query.runtime-state-scenario-evidence",
        selector: anchor.selector.as_str().to_string(),
        selector_class_names,
        property_name: anchor.property.clone(),
        scenario_join_kind: "fixtureWitnessedScenarioJoin",
        confidence_tier,
        confidence_tier_within_modeled_environment,
        static_boundary,
        driver_summaries,
        scenarios,
        static_condition_pruning,
        inline_style_overrides: Vec::new(),
        cascade_layer_topology_incomplete: topology_incomplete_unresolved_count.map(
            |unresolved_count| OmenaQueryCascadeLayerTopologyIncompleteV0 { unresolved_count },
        ),
        guarded_winner_authority,
        fragile_guarded_winner_diagnostics,
    })
}

pub(crate) fn query_runtime_state_confidence_tier(
    scenarios: &[OmenaQueryRuntimeStateScenarioV0],
    inline_style_overrides: &[OmenaQueryInlineStyleRuntimeOverrideV0],
    static_boundary_kind: &'static str,
) -> (&'static str, &'static str) {
    assert_eq!(
        static_boundary_kind, RUNTIME_STATE_STATIC_BOUNDARY_KIND,
        "runtime-state confidence requires the modeled static boundary"
    );

    let (tier, tier_within_modeled_environment) = if !inline_style_overrides.is_empty()
        || scenarios.iter().any(|scenario| {
            scenario.pseudo_state.is_some()
                || !scenario.condition_context.is_empty()
                || scenario.scenario_kind == "inlineStyleOverride"
        }) {
        (
            "conditionalDefinite",
            "conditionalDefiniteWithinModeledEnvironment",
        )
    } else {
        ("staticDefinite", "staticDefiniteWithinModeledEnvironment")
    };

    (tier, tier_within_modeled_environment)
}

#[cfg(test)]
fn query_runtime_guarded_winner_authority(
    anchor_selector: &str,
    property_name: &str,
    declarations: &[&OmenaCheckerCascadeDeclarationInputV0],
) -> Option<GuardedCascadeWinnerAuthorityV0> {
    let fragment =
        query_runtime_guarded_winner_fragment(anchor_selector, property_name, declarations)?;
    query_runtime_guarded_winner_authority_for_fragment(&fragment)
}

fn query_runtime_guarded_winner_analysis(
    anchor_selector: &str,
    property_name: &str,
    declarations: &[&OmenaCheckerCascadeDeclarationInputV0],
    scenarios: &[OmenaQueryRuntimeStateScenarioV0],
) -> Option<(
    GuardedCascadeWinnerAuthorityV0,
    Vec<super::super::OmenaQueryFragileGuardedWinnerDiagnosticV0>,
)> {
    let fragment =
        query_runtime_guarded_winner_fragment(anchor_selector, property_name, declarations)?;
    let authority = query_runtime_guarded_winner_authority_for_fragment(&fragment)?;
    let order = at_rule_nesting_order_for_fragment_v0(&fragment).ok()?;
    let cost_model = guarded_cascade_perturbation_cost_model_v0();
    let mut observed_assignments = BTreeSet::new();
    let diagnostics = scenarios
        .iter()
        .filter_map(|scenario| {
            let assignment = order
                .atoms()
                .iter()
                .map(|atom| scenario.condition_context.contains(atom))
                .collect::<Vec<_>>();
            if !observed_assignments.insert(assignment.clone()) {
                return None;
            }
            let robustness = compute_guarded_cascade_robustness_radius_v0(
                &fragment,
                assignment.as_slice(),
                &cost_model,
            )
            .ok()?;
            let declaration_id = declarations
                .iter()
                .map(|declaration| declaration.declaration_id.as_str())
                .collect::<BTreeSet<_>>()
                .into_iter()
                .nth(usize::try_from(robustness.baseline_winner_declaration_id).ok()?)?;
            summarize_omena_query_fragile_guarded_winner_v0(
                &robustness,
                declaration_id,
                OMENA_QUERY_FRAGILE_GUARDED_WINNER_THRESHOLD_V0,
            )
        })
        .collect();
    Some((authority, diagnostics))
}

fn query_runtime_guarded_winner_fragment(
    anchor_selector: &str,
    property_name: &str,
    declarations: &[&OmenaCheckerCascadeDeclarationInputV0],
) -> Option<GuardedCascadeFragmentV0<CascadeKey>> {
    let declaration_ids = declarations
        .iter()
        .map(|declaration| declaration.declaration_id.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .enumerate()
        .map(|(index, declaration_id)| {
            u32::try_from(index)
                .ok()
                .map(|index| (declaration_id, index))
        })
        .collect::<Option<std::collections::BTreeMap<_, _>>>()?;
    let contexts = declarations
        .iter()
        .map(|declaration| declaration.condition_context.clone())
        .collect::<Vec<_>>();
    let paths = at_rule_nesting_dfs_paths_v0(contexts.as_slice()).ok()?;
    let mut alphabet = BTreeSet::new();
    let candidates = declarations
        .iter()
        .zip(paths)
        .map(|(declaration, paths)| {
            let signature = parse_simple_selector_signature(declaration.selector.as_str())?;
            if signature.specificity_exactness != SpecificityExactnessV0::Exact
                || !signature.required_pseudo_states.is_empty()
            {
                return None;
            }
            let ranked = query_runtime_cascade_declaration_from_input(declaration);
            let conditions = query_runtime_guarded_conditions(
                declaration.condition_context.as_slice(),
                paths.as_slice(),
                &mut alphabet,
            )?;
            Some(GuardedCascadeCandidateV0::new(
                *declaration_ids.get(declaration.declaration_id.as_str())?,
                anchor_selector,
                property_name,
                ranked.key,
                GuardedCascadeSpecificityExactnessV0::Exact,
                0,
                conditions,
            ))
        })
        .collect::<Option<Vec<_>>>()?;
    GuardedCascadeFragmentV0::admit(alphabet, candidates).ok()
}

fn query_runtime_guarded_winner_authority_for_fragment(
    fragment: &GuardedCascadeFragmentV0<CascadeKey>,
) -> Option<GuardedCascadeWinnerAuthorityV0> {
    let order = at_rule_nesting_order_for_fragment_v0(fragment).ok()?;
    let mut manager = FirstWitnessManagerV0::new(order, FirstWitnessManagerConfigV0::default());
    let root = build_guarded_cascade_winner_v0(&mut manager, fragment).ok()?;
    let total = guarded_cascade_winner_is_total_v0(&manager, root).ok()?;
    Some(guarded_cascade_winner_authority_v0(
        fragment.predicate(),
        root,
        total,
    ))
}

fn query_runtime_guarded_conditions(
    context: &[String],
    paths: &[Vec<u32>],
    alphabet: &mut BTreeSet<String>,
) -> Option<Vec<GuardedCascadeConditionAtomV0>> {
    (context.len() == paths.len()).then_some(())?;
    context
        .iter()
        .zip(paths)
        .map(|(component, path)| {
            let component = component.trim();
            alphabet.insert(component.to_string());
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

pub(super) fn query_runtime_selector_matches_anchor_classes(
    anchor_selector: &str,
    candidate_selector: &str,
) -> bool {
    selector_co_match_verdict(anchor_selector, candidate_selector) != SelectorMatchVerdict::No
}

fn query_runtime_candidate_pseudo_states(
    declarations: &[&OmenaCheckerCascadeDeclarationInputV0],
) -> Vec<String> {
    let mut pseudo_states = BTreeSet::new();
    for declaration in declarations {
        let Some(signature) = parse_simple_selector_signature(declaration.selector.as_str()) else {
            // The selector remains represented as an Unknown activation in the
            // default scenario; do not fabricate a pseudo-state name from text.
            continue;
        };
        pseudo_states.extend(
            signature
                .required_pseudo_states
                .into_iter()
                .filter(|pseudo_state| {
                    query_runtime_pseudo_state_is_dynamic(pseudo_state.as_str())
                }),
        );
    }
    pseudo_states.into_iter().collect()
}

fn query_runtime_pseudo_state_is_dynamic(pseudo_state: &str) -> bool {
    matches!(
        pseudo_state,
        "active"
            | "checked"
            | "disabled"
            | "enabled"
            | "focus"
            | "focus-visible"
            | "focus-within"
            | "hover"
            | "target"
            | "visited"
    )
}

fn query_runtime_candidate_condition_contexts(
    declarations: &[&OmenaCheckerCascadeDeclarationInputV0],
    anchor_condition_context: &[String],
) -> (
    Vec<Vec<String>>,
    Vec<OmenaQueryStaticConditionPruningEvidenceV0>,
) {
    let mut contexts = declarations
        .iter()
        .map(|declaration| declaration.condition_context.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    if contexts.is_empty() {
        contexts.push(Vec::new());
    }
    let mut pruning = Vec::new();
    contexts.retain(|context| {
        let Some(evidence) = query_condition_context_static_supports_pruning_evidence(
            context.as_slice(),
            Some(anchor_condition_context),
        ) else {
            return true;
        };
        let keep = !evidence.pruned;
        pruning.push(evidence);
        keep
    });
    if contexts.is_empty() {
        contexts.push(anchor_condition_context.to_vec());
    }
    (contexts, pruning)
}

pub(crate) fn query_condition_context_static_supports_pruning_evidence(
    condition_context: &[String],
    anchor_condition_context: Option<&[String]>,
) -> Option<OmenaQueryStaticConditionPruningEvidenceV0> {
    let verdict = query_condition_context_static_supports_verdict(condition_context)?;
    if verdict != StaticSupportsEvalVerdictV0::AlwaysFalse {
        return None;
    }
    let anchor_context = anchor_condition_context.is_some_and(|anchor| anchor == condition_context);
    Some(OmenaQueryStaticConditionPruningEvidenceV0 {
        schema_version: "0",
        product: "omena-query.static-condition-pruning-evidence",
        condition_context: condition_context.to_vec(),
        assumption: "modernBrowser",
        verdict: query_static_supports_verdict_label(verdict),
        pruned: !anchor_context,
        anchor_context,
    })
}

fn query_condition_context_static_supports_verdict(
    condition_context: &[String],
) -> Option<StaticSupportsEvalVerdictV0> {
    let mut saw_supports = false;
    let mut saw_unknown = false;
    for entry in condition_context {
        let Some(condition) = query_supports_condition_from_context_entry(entry.as_str()) else {
            continue;
        };
        saw_supports = true;
        let witness = evaluate_static_supports_condition(
            condition,
            StaticSupportsAssumptionV0::ModernBrowser,
        );
        match witness.verdict {
            StaticSupportsEvalVerdictV0::AlwaysFalse => {
                return Some(StaticSupportsEvalVerdictV0::AlwaysFalse);
            }
            StaticSupportsEvalVerdictV0::Unknown => {
                saw_unknown = true;
            }
            StaticSupportsEvalVerdictV0::AlwaysTrue => {}
        }
    }
    if !saw_supports {
        None
    } else if saw_unknown {
        Some(StaticSupportsEvalVerdictV0::Unknown)
    } else {
        Some(StaticSupportsEvalVerdictV0::AlwaysTrue)
    }
}

fn query_supports_condition_from_context_entry(entry: &str) -> Option<&str> {
    let trimmed = entry.trim_start();
    let prefix = "@supports";
    if trimmed.len() < prefix.len() {
        return None;
    }
    let (candidate_prefix, rest) = trimmed.split_at(prefix.len());
    if !candidate_prefix.eq_ignore_ascii_case(prefix) {
        return None;
    }
    if rest
        .chars()
        .next()
        .is_some_and(|ch| !ch.is_whitespace() && ch != '(')
    {
        return None;
    }
    let condition = rest.trim_start();
    if condition.is_empty() {
        None
    } else {
        Some(condition)
    }
}

fn query_static_supports_verdict_label(verdict: StaticSupportsEvalVerdictV0) -> &'static str {
    match verdict {
        StaticSupportsEvalVerdictV0::AlwaysTrue => "AlwaysTrue",
        StaticSupportsEvalVerdictV0::AlwaysFalse => "AlwaysFalse",
        StaticSupportsEvalVerdictV0::Unknown => "Unknown",
    }
}

fn query_runtime_state_scenario(
    property_name: &str,
    pseudo_state: Option<&str>,
    condition_context: &[String],
    declarations: &[&OmenaCheckerCascadeDeclarationInputV0],
) -> OmenaQueryRuntimeStateScenarioV0 {
    query_runtime_state_scenario_with_optional_inline_override(
        property_name,
        pseudo_state,
        condition_context,
        declarations,
        None,
    )
}

pub(in crate::style) fn query_runtime_state_scenario_with_inline_override(
    selector: &str,
    property_name: &str,
    declarations: &[OmenaCheckerCascadeDeclarationInputV0],
    override_fact: &OmenaQueryInlineStyleRuntimeOverrideV0,
) -> OmenaQueryRuntimeStateScenarioV0 {
    let candidate_declarations = declarations
        .iter()
        .filter(|declaration| declaration.property == property_name)
        .filter(|declaration| {
            query_runtime_selector_matches_anchor_classes(selector, declaration.selector.as_str())
        })
        .collect::<Vec<_>>();
    query_runtime_state_scenario_with_optional_inline_override(
        property_name,
        None,
        &[],
        candidate_declarations.as_slice(),
        Some(override_fact),
    )
}

fn query_runtime_state_scenario_with_optional_inline_override(
    property_name: &str,
    pseudo_state: Option<&str>,
    condition_context: &[String],
    declarations: &[&OmenaCheckerCascadeDeclarationInputV0],
    inline_override: Option<&OmenaQueryInlineStyleRuntimeOverrideV0>,
) -> OmenaQueryRuntimeStateScenarioV0 {
    query_runtime_state_scenario_evaluation(
        property_name,
        pseudo_state,
        condition_context,
        declarations,
        inline_override,
    )
    .0
}

fn query_runtime_state_scenario_evaluation(
    property_name: &str,
    pseudo_state: Option<&str>,
    condition_context: &[String],
    declarations: &[&OmenaCheckerCascadeDeclarationInputV0],
    inline_override: Option<&OmenaQueryInlineStyleRuntimeOverrideV0>,
) -> (OmenaQueryRuntimeStateScenarioV0, CascadeOutcome) {
    let scenario_declarations = declarations
        .iter()
        .copied()
        .filter(|declaration| declaration.condition_context == condition_context)
        .map(|declaration| {
            let activation =
                query_runtime_selector_active_for_pseudo_state(declaration, pseudo_state);
            (declaration, activation)
        })
        .collect::<Vec<_>>();
    let has_unknown_activation =
        scenario_declarations
            .iter()
            .any(|(_, activation)| match activation {
                ScenarioActivation::Active | ScenarioActivation::Inactive => false,
                ScenarioActivation::Unknown => true,
            });
    let active_declarations = scenario_declarations
        .iter()
        .filter_map(|(declaration, activation)| match activation {
            ScenarioActivation::Active | ScenarioActivation::Unknown => Some(*declaration),
            ScenarioActivation::Inactive => None,
        })
        .collect::<Vec<_>>();
    let mut property_candidates = active_declarations
        .iter()
        .map(|declaration| AbstractPropertyValueCandidateV0 {
            property_name: declaration.property.clone(),
            value: declaration.value.clone(),
            pseudo_state: query_runtime_declaration_primary_pseudo_state(declaration),
            condition_context: declaration.condition_context.clone(),
            layer_name: declaration.layer_name.clone(),
            layer_order: declaration.layer_order,
            source_order: Some(declaration.source_order),
            important: declaration.important,
            same_selector_ordering: false,
        })
        .collect::<Vec<_>>();
    let mut ranked_declarations = active_declarations
        .iter()
        .map(|declaration| query_runtime_cascade_declaration_from_input(declaration))
        .collect::<Vec<_>>();
    let inline_declaration_id = inline_override
        .filter(|override_fact| !override_fact.important_suffix_present())
        .map(query_runtime_inline_style_declaration_id);
    if let Some(override_fact) =
        inline_override.filter(|override_fact| !override_fact.important_suffix_present())
    {
        let value = override_fact
            .value
            .clone()
            .unwrap_or_else(|| "<dynamic>".to_string());
        property_candidates.push(AbstractPropertyValueCandidateV0 {
            property_name: property_name.to_string(),
            value,
            pseudo_state: None,
            condition_context: Vec::new(),
            layer_name: None,
            layer_order: None,
            source_order: Some(u32::MAX),
            important: override_fact.important_suffix_present(),
            same_selector_ordering: false,
        });
        ranked_declarations.push(query_runtime_inline_style_cascade_declaration(
            property_name,
            override_fact,
        ));
    }
    let property_value_narrowing = narrow_abstract_property_value_for_cascade_branch(
        property_name,
        pseudo_state,
        condition_context,
        None,
        None,
        false,
        property_candidates.as_slice(),
    );
    let outcome = if ranked_declarations.is_empty() {
        CascadeOutcome::Top
    } else {
        cascade_property(ranked_declarations, property_name)
    };
    let (winner_declaration_id, winner_value) = if has_unknown_activation {
        (None, None)
    } else {
        match &outcome {
            CascadeOutcome::Definite { winner, .. } => {
                let value = match &winner.value {
                    CascadeValue::Literal(value) => Some(value.clone()),
                    _ => None,
                };
                (Some(winner.id.clone()), value)
            }
            _ => (None, None),
        }
    };

    let mut declaration_ids = scenario_declarations
        .iter()
        .filter_map(|(declaration, activation)| match activation {
            ScenarioActivation::Active => Some(declaration.declaration_id.clone()),
            ScenarioActivation::Inactive => None,
            ScenarioActivation::Unknown => Some(runtime_state_unknown_activation_declaration_id(
                declaration.declaration_id.as_str(),
            )),
        })
        .collect::<Vec<_>>();
    declaration_ids.extend(inline_declaration_id);

    (
        OmenaQueryRuntimeStateScenarioV0 {
            scenario_kind: if inline_override.is_some() {
                "inlineStyleOverride"
            } else if condition_context.is_empty() {
                "pseudoState"
            } else {
                "mediaEnvironment"
            },
            pseudo_state: pseudo_state.map(str::to_string),
            condition_context: condition_context.to_vec(),
            declaration_ids,
            winner_declaration_id,
            winner_value,
            property_value_narrowing,
        },
        outcome,
    )
}

fn query_runtime_inline_style_declaration_id(
    override_fact: &OmenaQueryInlineStyleRuntimeOverrideV0,
) -> String {
    format!(
        "inline-style:{}:{}:{}",
        override_fact.source_path,
        override_fact.range.start.line,
        override_fact.range.start.character
    )
}

fn query_runtime_inline_style_cascade_declaration(
    property_name: &str,
    override_fact: &OmenaQueryInlineStyleRuntimeOverrideV0,
) -> CascadeDeclaration {
    let value = override_fact
        .value
        .clone()
        .unwrap_or_else(|| "<dynamic>".to_string());
    CascadeDeclaration {
        id: query_runtime_inline_style_declaration_id(override_fact),
        property: property_name.to_string(),
        value: CascadeValue::Literal(value),
        key: CascadeKey::new(
            cascade_level_for_origin(CascadeOriginV0::Inline, false),
            normalized_layer_rank(false, None),
            0,
            Specificity::ZERO,
            u32::MAX,
        ),
        open_world_tie_evidence: OpenWorldTieEvidence::NONE,
        specificity_exactness: SpecificityExactnessV0::Exact,
    }
}

fn query_runtime_selector_active_for_pseudo_state(
    declaration: &OmenaCheckerCascadeDeclarationInputV0,
    pseudo_state: Option<&str>,
) -> ScenarioActivation {
    let Some(signature) = parse_simple_selector_signature(declaration.selector.as_str()) else {
        return ScenarioActivation::Unknown;
    };
    let required = signature
        .required_pseudo_states
        .into_iter()
        .filter(|state| query_runtime_pseudo_state_is_dynamic(state.as_str()))
        .collect::<BTreeSet<_>>();
    let active = match pseudo_state {
        Some(pseudo_state) => required.is_empty() || required.contains(pseudo_state),
        None => required.is_empty(),
    };
    match active {
        true => ScenarioActivation::Active,
        false => ScenarioActivation::Inactive,
    }
}

fn query_runtime_declaration_primary_pseudo_state(
    declaration: &OmenaCheckerCascadeDeclarationInputV0,
) -> Option<String> {
    parse_simple_selector_signature(declaration.selector.as_str())?
        .required_pseudo_states
        .into_iter()
        .find(|state| query_runtime_pseudo_state_is_dynamic(state.as_str()))
}

pub(in crate::style) fn query_runtime_cascade_declaration_from_input(
    input: &OmenaCheckerCascadeDeclarationInputV0,
) -> CascadeDeclaration {
    let level = cascade_level_for_origin(input.origin, input.important);
    let layer_rank = normalized_layer_rank(
        input.important,
        input.layer_order.and_then(LayerOrdinal::new),
    );
    let (specificity, specificity_exactness) =
        parse_simple_selector_signature(input.selector.as_str()).map_or(
            (Specificity::ZERO, SpecificityExactnessV0::Inexact),
            |signature| (signature.specificity, signature.specificity_exactness),
        );
    let value = input.value.trim().to_string();

    CascadeDeclaration {
        id: input.declaration_id.clone(),
        property: input.property.clone(),
        value: CascadeValue::Literal(value),
        key: CascadeKey::new(level, layer_rank, 0, specificity, input.source_order),
        open_world_tie_evidence: OpenWorldTieEvidence::NONE,
        specificity_exactness,
    }
}

pub(super) fn query_element_class_signature_constraints(
    selector_class_names: &[String],
) -> Vec<AbstractClassValueV0> {
    if selector_class_names.is_empty() {
        return Vec::new();
    }

    let first = selector_class_names.first().cloned().unwrap_or_default();
    let last = selector_class_names.last().cloned().unwrap_or_default();
    let signature_min_length = selector_class_names
        .iter()
        .map(String::len)
        .sum::<usize>()
        .saturating_add(selector_class_names.len().saturating_sub(1));

    vec![prefix_suffix_class_value(
        first,
        last,
        Some(signature_min_length),
        None,
    )]
}

pub(super) fn query_selector_class_names(selector: &str) -> Vec<String> {
    let mut names = BTreeSet::new();
    for entry in class_selector_names(selector) {
        names.insert(entry.name.into_raw());
    }
    names.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use omena_cascade::{CascadeLevel, CascadeOriginV0};
    use omena_query_checker_orchestrator::CanonicalSelector;

    fn declaration(
        id: &str,
        origin: CascadeOriginV0,
        important: bool,
    ) -> OmenaCheckerCascadeDeclarationInputV0 {
        OmenaCheckerCascadeDeclarationInputV0 {
            declaration_id: id.to_string(),
            selector: CanonicalSelector::from_canonical(".target"),
            property: "color".to_string(),
            value: id.to_string(),
            source_order: 0,
            condition_context: Vec::new(),
            layer_name: None,
            layer_order: None,
            origin,
            important,
            var_references: Vec::new(),
        }
    }

    fn selector_declaration(
        id: &str,
        selector: &str,
        value: &str,
        source_order: u32,
    ) -> OmenaCheckerCascadeDeclarationInputV0 {
        OmenaCheckerCascadeDeclarationInputV0 {
            declaration_id: id.to_string(),
            selector: CanonicalSelector::from_canonical(selector),
            property: "color".to_string(),
            value: value.to_string(),
            source_order,
            condition_context: Vec::new(),
            layer_name: None,
            layer_order: None,
            origin: CascadeOriginV0::Author,
            important: false,
            var_references: Vec::new(),
        }
    }

    #[test]
    fn extracts_selector_classes_without_string_or_depth_phantoms() {
        // Each assertion can be falsified directly by the selector supplied
        // here; selectors with all of these shapes are accepted from product
        // inputs before this extractor runs.
        assert!(query_selector_class_names(r#"[data-x="a.b"]"#).is_empty());
        assert_eq!(query_selector_class_names(r".a\.b"), [r"a\.b"]);
        assert_eq!(query_selector_class_names(r".\31 23"), [r"\31 23"]);
        assert_eq!(query_selector_class_names(".카드"), ["카드"]);
        assert_eq!(query_selector_class_names(".café"), ["café"]);
        assert_eq!(
            query_selector_class_names(".card .title"),
            ["card", "title"]
        );
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
        let mut alphabet = BTreeSet::new();
        let conditions = query_runtime_guarded_conditions(&context, &paths[0], &mut alphabet)
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
        assert_eq!(alphabet, context.into_iter().collect());
        Ok(())
    }

    #[test]
    fn drives_the_origin_ladder_from_checker_inputs() {
        let declarations = [
            declaration("ua-normal", CascadeOriginV0::UserAgent, false),
            declaration("user-normal", CascadeOriginV0::User, false),
            declaration("author-normal", CascadeOriginV0::Author, false),
            declaration("inline-normal", CascadeOriginV0::Inline, false),
            declaration("author-important", CascadeOriginV0::Author, true),
            declaration("user-important", CascadeOriginV0::User, true),
            declaration("ua-important", CascadeOriginV0::UserAgent, true),
        ];
        let levels = declarations
            .iter()
            .map(query_runtime_cascade_declaration_from_input)
            .map(|declaration| declaration.key.level)
            .collect::<Vec<_>>();
        assert_eq!(
            levels,
            vec![
                CascadeLevel::UserAgentNormal,
                CascadeLevel::UserNormal,
                CascadeLevel::AuthorNormal,
                CascadeLevel::InlineNormal,
                CascadeLevel::AuthorImportant,
                CascadeLevel::UserImportant,
                CascadeLevel::UserAgentImportant,
            ]
        );

        let references = declarations.iter().collect::<Vec<_>>();
        let scenario = query_runtime_state_scenario("color", None, &[], &references);
        assert_eq!(
            scenario.winner_declaration_id.as_deref(),
            Some("ua-important")
        );

        let normal_references = declarations[..3].iter().collect::<Vec<_>>();
        let normal_scenario = query_runtime_state_scenario("color", None, &[], &normal_references);
        assert_eq!(
            normal_scenario.winner_declaration_id.as_deref(),
            Some("author-normal")
        );
    }

    #[test]
    fn complex_functional_specificity_selects_the_browser_winner() {
        let declarations = [
            selector_declaration("complex", ":is(#root .item)", "red", 0),
            selector_declaration("simple", ".item", "blue", 1),
        ];
        let references = declarations.iter().collect::<Vec<_>>();
        let scenario = query_runtime_state_scenario("color", None, &[], &references);

        assert_eq!(scenario.winner_declaration_id.as_deref(), Some("complex"));
        assert_eq!(scenario.winner_value.as_deref(), Some("red"));
    }

    #[test]
    fn unsupported_selector_specificity_does_not_claim_a_winner() {
        let declarations = [selector_declaration(
            "unsupported",
            ":unknown(.item)",
            "red",
            0,
        )];
        let references = declarations.iter().collect::<Vec<_>>();
        let scenario = query_runtime_state_scenario("color", None, &[], &references);

        assert_eq!(scenario.winner_declaration_id, None);
        assert_eq!(scenario.winner_value, None);
    }

    #[test]
    fn unsupported_selector_declaration_preserves_inexact_specificity() {
        let input = selector_declaration("unsupported", ":unknown(.item)", "red", 0);
        let declaration = query_runtime_cascade_declaration_from_input(&input);

        assert_eq!(
            declaration.specificity_exactness,
            SpecificityExactnessV0::Inexact
        );
    }

    #[test]
    fn inline_style_remains_in_the_ranked_candidate_set_when_author_important_wins() {
        let author_important = declaration("author-important", CascadeOriginV0::Author, true);
        let inline_override = OmenaQueryInlineStyleRuntimeOverrideV0 {
            source_path: "file:///workspace/src/App.tsx".to_string(),
            range: Default::default(),
            property_name: "color".to_string(),
            value: Some("blue".to_string()),
            cascade_tier: "authorInlineStyle",
            important: false,
            static_value: true,
        };
        let (scenario, outcome) = query_runtime_state_scenario_evaluation(
            "color",
            None,
            &[],
            &[&author_important],
            Some(&inline_override),
        );

        assert_eq!(
            scenario.winner_declaration_id.as_deref(),
            Some("author-important")
        );
        assert!(
            matches!(outcome, CascadeOutcome::Definite { .. }),
            "author-important and inline style must produce a definite ranking"
        );
        let CascadeOutcome::Definite {
            winner,
            also_considered,
            ..
        } = outcome
        else {
            return;
        };
        assert_eq!(winner.id, "author-important");
        assert!(
            also_considered
                .iter()
                .any(|declaration| declaration.id.starts_with("inline-style:"))
        );
    }

    #[test]
    fn jsx_inline_important_suffix_remains_observational() {
        let author_normal = declaration("author-normal", CascadeOriginV0::Author, false);
        let inline_override = OmenaQueryInlineStyleRuntimeOverrideV0 {
            source_path: "file:///workspace/src/App.tsx".to_string(),
            range: Default::default(),
            property_name: "color".to_string(),
            value: Some("blue !important".to_string()),
            cascade_tier: "authorInlineStyleImportantSuffix",
            important: true,
            static_value: true,
        };
        let (_, outcome) = query_runtime_state_scenario_evaluation(
            "color",
            None,
            &[],
            &[&author_normal],
            Some(&inline_override),
        );

        assert!(matches!(outcome, CascadeOutcome::Definite { .. }));
        let CascadeOutcome::Definite { winner, .. } = outcome else {
            return;
        };
        assert_eq!(winner.id, "author-normal");
        assert_eq!(winner.key.level, CascadeLevel::AuthorNormal);
        assert!(inline_override.important_suffix_present());
    }

    #[test]
    fn confidence_tiers_are_derived_within_the_declared_static_boundary() {
        let (static_tier, static_tier_within_modeled_environment) =
            query_runtime_state_confidence_tier(&[], &[], RUNTIME_STATE_STATIC_BOUNDARY_KIND);
        assert_eq!(static_tier, "staticDefinite");
        assert_eq!(
            static_tier_within_modeled_environment,
            "staticDefiniteWithinModeledEnvironment"
        );

        let inline_style_overrides = [OmenaQueryInlineStyleRuntimeOverrideV0 {
            source_path: "file:///workspace/src/App.tsx".to_string(),
            range: Default::default(),
            property_name: "color".to_string(),
            value: Some("red".to_string()),
            cascade_tier: "authorInlineStyle",
            important: false,
            static_value: true,
        }];
        let (conditional_tier, conditional_tier_within_modeled_environment) =
            query_runtime_state_confidence_tier(
                &[],
                inline_style_overrides.as_slice(),
                RUNTIME_STATE_STATIC_BOUNDARY_KIND,
            );
        assert_eq!(conditional_tier, "conditionalDefinite");
        assert_eq!(
            conditional_tier_within_modeled_environment,
            "conditionalDefiniteWithinModeledEnvironment"
        );

        let definite_declarations = [selector_declaration("decl-0", ".target", "red", 0)];
        let definite_references = definite_declarations.iter().collect::<Vec<_>>();
        let definite_scenario =
            query_runtime_state_scenario("color", None, &[], &definite_references);
        let unknown_scenario = OmenaQueryRuntimeStateScenarioV0 {
            declaration_ids: vec![runtime_state_unknown_activation_declaration_id("decl-1")],
            winner_declaration_id: None,
            winner_value: None,
            ..definite_scenario.clone()
        };
        assert_eq!(
            unknown_scenario.unknown_activation_declaration_ids(),
            vec!["decl-1"]
        );
        let indeterminate_scenario = OmenaQueryRuntimeStateScenarioV0 {
            winner_declaration_id: None,
            winner_value: None,
            ..definite_scenario.clone()
        };
        let certainty_tiers = [
            runtime_state_result_certainty_labels(
                std::slice::from_ref(&definite_scenario),
                static_tier,
                false,
                None,
            ),
            runtime_state_result_certainty_labels(
                std::slice::from_ref(&indeterminate_scenario),
                static_tier,
                false,
                None,
            ),
            runtime_state_result_certainty_labels(
                std::slice::from_ref(&unknown_scenario),
                static_tier,
                false,
                None,
            ),
            runtime_state_result_certainty_labels(
                std::slice::from_ref(&definite_scenario),
                conditional_tier,
                false,
                None,
            ),
            runtime_state_result_certainty_labels(
                std::slice::from_ref(&indeterminate_scenario),
                conditional_tier,
                false,
                None,
            ),
            runtime_state_result_certainty_labels(
                std::slice::from_ref(&unknown_scenario),
                conditional_tier,
                false,
                None,
            ),
        ];
        assert_eq!(
            certainty_tiers,
            [
                ("staticDefinite", "staticDefiniteWithinModeledEnvironment"),
                (
                    "staticIndeterminate",
                    "staticIndeterminateWithinModeledEnvironment",
                ),
                ("staticUnknown", "staticUnknownWithinModeledEnvironment"),
                (
                    "conditionalDefinite",
                    "conditionalDefiniteWithinModeledEnvironment",
                ),
                (
                    "conditionalIndeterminate",
                    "conditionalIndeterminateWithinModeledEnvironment",
                ),
                (
                    "conditionalUnknown",
                    "conditionalUnknownWithinModeledEnvironment",
                ),
            ]
        );

        for qualified_tier in [
            static_tier_within_modeled_environment,
            conditional_tier_within_modeled_environment,
            certainty_tiers[0].1,
            certainty_tiers[1].1,
            certainty_tiers[2].1,
            certainty_tiers[3].1,
            certainty_tiers[4].1,
            certainty_tiers[5].1,
        ] {
            assert!(qualified_tier.ends_with("WithinModeledEnvironment"));
            assert!(
                !["proven", "verified", "certified", "complete"]
                    .iter()
                    .any(|claim| qualified_tier.to_ascii_lowercase().contains(claim))
            );
        }
    }

    #[test]
    fn guarded_winner_authority_upgrades_result_certainty_without_rekeying_confidence()
    -> Result<(), &'static str> {
        let base = selector_declaration("base", ".target", "black", 0);
        let mut guarded = selector_declaration("guarded", ".target", "red", 1);
        guarded.condition_context = vec!["@media (min-width: 40rem)".to_string()];
        let declarations = [&base, &guarded];
        let authority =
            query_runtime_guarded_winner_authority(".target", "color", declarations.as_slice())
                .ok_or("declared fragment authority missing")?;
        assert!(authority.winner_defined_for_all_assignments);

        let scenario = OmenaQueryRuntimeStateScenarioV0 {
            scenario_kind: "mediaEnvironment",
            pseudo_state: None,
            condition_context: guarded.condition_context.clone(),
            declaration_ids: vec![guarded.declaration_id.clone()],
            winner_declaration_id: None,
            winner_value: None,
            property_value_narrowing: query_runtime_state_scenario(
                "color",
                None,
                guarded.condition_context.as_slice(),
                &[&guarded],
            )
            .property_value_narrowing,
        };
        let without_authority = runtime_state_result_certainty_labels(
            std::slice::from_ref(&scenario),
            "conditionalDefinite",
            false,
            None,
        );
        let with_authority = runtime_state_result_certainty_labels(
            std::slice::from_ref(&scenario),
            "conditionalDefinite",
            false,
            Some(&authority),
        );
        assert_eq!(
            without_authority,
            (
                "conditionalIndeterminate",
                "conditionalIndeterminateWithinModeledEnvironment",
            )
        );
        assert_eq!(
            with_authority,
            (
                "conditionalDefinite",
                "conditionalDefiniteWithinModeledEnvironment",
            )
        );
        let (_, fragile_guarded_winner_diagnostics) = query_runtime_guarded_winner_analysis(
            ".target",
            "color",
            declarations.as_slice(),
            std::slice::from_ref(&scenario),
        )
        .ok_or("declared fragment robustness missing")?;
        assert_eq!(fragile_guarded_winner_diagnostics.len(), 1);
        assert_eq!(fragile_guarded_winner_diagnostics[0].robustness_radius, 1);
        let evidence = OmenaQueryRuntimeStateScenarioEvidenceV0 {
            schema_version: "0",
            product: "omena-query.runtime-state-scenario-evidence",
            selector: ".target".to_string(),
            selector_class_names: vec!["target".to_string()],
            property_name: "color".to_string(),
            scenario_join_kind: "fixtureWitnessedScenarioJoin",
            confidence_tier: "conditionalDefinite",
            confidence_tier_within_modeled_environment: "conditionalDefiniteWithinModeledEnvironment",
            static_boundary: OmenaQueryRuntimeStateStaticBoundaryV0 {
                boundary_kind: RUNTIME_STATE_STATIC_BOUNDARY_KIND,
                static_value_assuming_no_runtime_override: true,
                tracks_dom_mutation: false,
                tracks_class_list_mutation: false,
            },
            driver_summaries: Vec::new(),
            scenarios: vec![scenario],
            static_condition_pruning: Vec::new(),
            inline_style_overrides: Vec::new(),
            cascade_layer_topology_incomplete: None,
            guarded_winner_authority: Some(authority),
            fragile_guarded_winner_diagnostics,
        };
        let serialized = serde_json::to_value(&evidence)
            .map_err(|_| "guarded authority serialization failed")?;
        assert_eq!(serialized["confidenceTier"], "conditionalDefinite");
        assert_eq!(serialized["resultCertainty"], "conditionalDefinite");
        assert_eq!(
            serialized["guardedWinnerAuthority"]["rule"]["kind"],
            "canonicalMtbddInsideFragment"
        );
        assert_eq!(
            serialized["fragileGuardedWinnerDiagnostics"][0]["robustnessRadius"],
            1
        );
        assert_eq!(
            serialized["fragileGuardedWinnerDiagnostics"][0]["baselineWinnerDeclarationId"],
            "guarded"
        );
        assert_eq!(
            serialized["fragileGuardedWinnerDiagnostics"][0]["calibrationStage"],
            "schemaOnlyUncalibrated"
        );
        assert_eq!(
            serialized["fragileGuardedWinnerDiagnostics"][0]["publicSafetyClaimReady"],
            false
        );
        Ok(())
    }

    #[test]
    #[should_panic(expected = "runtime-state confidence requires the modeled static boundary")]
    fn confidence_tiers_reject_an_unrelated_boundary() {
        let _ = query_runtime_state_confidence_tier(&[], &[], "tracksDomMutation");
    }
}
