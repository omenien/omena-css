use std::collections::BTreeSet;

use omena_cascade::{
    CascadeDeclaration, CascadeKey, CascadeOutcome, CascadeValue, LayerRank, ModuleRank,
    SelectorMatchVerdict, Specificity, SpecificityExactnessV0, StaticSupportsAssumptionV0,
    StaticSupportsEvalVerdictV0, cascade_level_for_origin, cascade_property,
    evaluate_static_supports_condition, parse_simple_selector_signature, selector_co_match_verdict,
};
use omena_query_checker_orchestrator::{
    OmenaCheckerCascadeDeclarationInputV0, OmenaCheckerCascadeEvaluationV0,
};
use omena_query_core::{
    AbstractClassValueV0, AbstractPropertyValueCandidateV0,
    narrow_abstract_property_value_for_cascade_branch, prefix_suffix_class_value,
};

#[cfg(test)]
use crate::types::runtime_state_result_certainty_labels;
use crate::types::runtime_state_unknown_activation_declaration_id;

use super::super::{
    OmenaQueryInlineStyleRuntimeOverrideV0, OmenaQueryRuntimeStateDriverSummaryV0,
    OmenaQueryRuntimeStateScenarioEvidenceV0, OmenaQueryRuntimeStateScenarioV0,
    OmenaQueryRuntimeStateStaticBoundaryV0, OmenaQueryStaticConditionPruningEvidenceV0,
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
        driver_summaries: vec![
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
                driver: "inlineStyleHighestSpecificityTier",
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
        ],
        scenarios,
        static_condition_pruning,
        inline_style_overrides: Vec::new(),
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
    let property_candidates = active_declarations
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
    let property_value_narrowing = narrow_abstract_property_value_for_cascade_branch(
        property_name,
        pseudo_state,
        condition_context,
        None,
        None,
        false,
        property_candidates.as_slice(),
    );
    let outcome = if active_declarations.is_empty() {
        CascadeOutcome::Top
    } else {
        cascade_property(
            active_declarations
                .iter()
                .map(|declaration| query_runtime_cascade_declaration_from_input(declaration))
                .collect::<Vec<_>>(),
            property_name,
        )
    };
    let (winner_declaration_id, winner_value) = if has_unknown_activation {
        (None, None)
    } else {
        match outcome {
            CascadeOutcome::Definite { winner, .. } => {
                let value = match winner.value {
                    CascadeValue::Literal(value) => Some(value),
                    _ => None,
                };
                (Some(winner.id), value)
            }
            _ => (None, None),
        }
    };

    OmenaQueryRuntimeStateScenarioV0 {
        scenario_kind: if condition_context.is_empty() {
            "pseudoState"
        } else {
            "mediaEnvironment"
        },
        pseudo_state: pseudo_state.map(str::to_string),
        condition_context: condition_context.to_vec(),
        declaration_ids: scenario_declarations
            .iter()
            .filter_map(|(declaration, activation)| match activation {
                ScenarioActivation::Active => Some(declaration.declaration_id.clone()),
                ScenarioActivation::Inactive => None,
                ScenarioActivation::Unknown => {
                    Some(runtime_state_unknown_activation_declaration_id(
                        declaration.declaration_id.as_str(),
                    ))
                }
            })
            .collect(),
        winner_declaration_id,
        winner_value,
        property_value_narrowing,
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
    let layer_rank = LayerRank(input.layer_order.unwrap_or(0));
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
        key: CascadeKey::new(
            level,
            layer_rank,
            0,
            specificity,
            ModuleRank::ZERO,
            input.source_order,
        ),
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
    let bytes = selector.as_bytes();
    let mut index = 0usize;
    let mut names = BTreeSet::new();
    while index < bytes.len() {
        if bytes[index] != b'.' {
            index += 1;
            continue;
        }
        let start = index + 1;
        let mut end = start;
        while end < bytes.len() && query_selector_class_name_byte(bytes[end]) {
            end += 1;
        }
        if end > start {
            names.insert(selector[start..end].to_string());
            index = end;
        } else {
            index += 1;
        }
    }
    names.into_iter().collect()
}

fn query_selector_class_name_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'-'
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
            ),
            runtime_state_result_certainty_labels(
                std::slice::from_ref(&indeterminate_scenario),
                static_tier,
            ),
            runtime_state_result_certainty_labels(
                std::slice::from_ref(&unknown_scenario),
                static_tier,
            ),
            runtime_state_result_certainty_labels(
                std::slice::from_ref(&definite_scenario),
                conditional_tier,
            ),
            runtime_state_result_certainty_labels(
                std::slice::from_ref(&indeterminate_scenario),
                conditional_tier,
            ),
            runtime_state_result_certainty_labels(
                std::slice::from_ref(&unknown_scenario),
                conditional_tier,
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
    #[should_panic(expected = "runtime-state confidence requires the modeled static boundary")]
    fn confidence_tiers_reject_an_unrelated_boundary() {
        let _ = query_runtime_state_confidence_tier(&[], &[], "tracksDomMutation");
    }
}
