//! Checker diagnostic orchestration boundary for the `omena-query` facade.
//!
//! `omena-query` owns the consumer-facing diagnostic API. This crate owns the
//! narrower checker handoff: it invokes `omena-checker`, verifies that emitted
//! rule codes are registered, and returns a gate summary alongside evaluations.

use std::collections::BTreeSet;

use serde::Serialize;

pub use omena_abstract_value::AbstractClassValueV0;
use omena_abstract_value::{
    ClassValueFlowGraphV0, ClassValueFlowNodeV0, ClassValueFlowTransferV0,
    KLimitedCallSiteFlowInputV0, abstract_class_value_kind, analyze_k_limited_call_site_flows,
    external_string_type_facts_from_abstract_class_value,
};
pub use omena_checker::{
    CategoricalCascadeEvidenceV0, OmenaCheckerCascadeDeclarationInputV0,
    OmenaCheckerCascadeEvaluationV0, OmenaCheckerCascadeInputV0, OmenaCheckerCustomPropertyInputV0,
    OmenaCheckerMTierEvaluationV0, OmenaCheckerRgFlowCouplingInputV0,
    OmenaCheckerRgFlowCouplingSpaceInputV0, OmenaCheckerRgFlowEvaluationV0,
    OmenaCheckerRgFlowInputV0, OmenaCheckerRuleCodeV0, checker_categorical_cascade_evidence_v0,
};
use omena_checker::{
    OmenaCheckerDynamicClassDomainInputV0, evaluate_omena_checker_cascade_rules,
    evaluate_omena_checker_m_tier_rules, evaluate_omena_checker_rg_flow_rules,
    list_omena_checker_m_tier_rule_code_names, list_omena_checker_rule_code_names,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryCheckerCascadeGateV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub orchestrator_kind: &'static str,
    pub enabled_rule_names: Vec<&'static str>,
    pub emitted_rule_names: Vec<&'static str>,
    pub suppressed_rule_names: Vec<&'static str>,
    pub registered_rule_count: usize,
    pub unregistered_rule_count: usize,
    pub evaluation_count: usize,
    pub enforcement_passed: bool,
    pub evaluations: Vec<OmenaCheckerCascadeEvaluationV0>,
    pub ready_surfaces: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryCheckerProductDiagnosticGateV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub product_diagnostic_code: String,
    pub checker_rule_code_name: Option<&'static str>,
    pub checker_rule_registered: bool,
    pub checker_owned: bool,
    pub enforcement_passed: bool,
    pub provenance: Vec<&'static str>,
}

pub fn gate_omena_query_checker_product_diagnostic_code_v0(
    product_diagnostic_code: &str,
) -> OmenaQueryCheckerProductDiagnosticGateV0 {
    let checker_rule_code_name =
        query_product_diagnostic_checker_rule_code_name_v0(product_diagnostic_code);
    let checker_rule_registered = checker_rule_code_name
        .is_some_and(|rule_code| list_omena_checker_rule_code_names().contains(&rule_code));
    let checker_owned = checker_rule_code_name.is_some();
    let provenance = if checker_rule_registered {
        vec![
            "omena-query-checker-orchestrator.product-diagnostic-gate",
            "omena-checker.rule-registry",
        ]
    } else {
        Vec::new()
    };

    OmenaQueryCheckerProductDiagnosticGateV0 {
        schema_version: "0",
        product: "omena-query-checker-orchestrator.product-diagnostic-gate",
        product_diagnostic_code: product_diagnostic_code.to_string(),
        checker_rule_code_name,
        checker_rule_registered,
        checker_owned,
        enforcement_passed: !checker_owned || checker_rule_registered,
        provenance,
    }
}

pub fn query_product_diagnostic_checker_rule_code_name_v0(
    product_diagnostic_code: &str,
) -> Option<&'static str> {
    match product_diagnostic_code {
        "missingModule" => Some(OmenaCheckerRuleCodeV0::MissingModule.as_str()),
        "missingSelector" | "missingStaticClass" => {
            Some(OmenaCheckerRuleCodeV0::MissingStaticClass.as_str())
        }
        "missingTemplatePrefix" => Some(OmenaCheckerRuleCodeV0::MissingTemplatePrefix.as_str()),
        "missingResolvedClassValues" => {
            Some(OmenaCheckerRuleCodeV0::MissingResolvedClassValues.as_str())
        }
        "missingResolvedClassDomain" => {
            Some(OmenaCheckerRuleCodeV0::MissingResolvedClassDomain.as_str())
        }
        "missingComposedModule" => Some(OmenaCheckerRuleCodeV0::MissingComposedModule.as_str()),
        "missingComposedSelector" => Some(OmenaCheckerRuleCodeV0::MissingComposedSelector.as_str()),
        "missingValueModule" => Some(OmenaCheckerRuleCodeV0::MissingValueModule.as_str()),
        "missingImportedValue" => Some(OmenaCheckerRuleCodeV0::MissingImportedValue.as_str()),
        "missingKeyframes" => Some(OmenaCheckerRuleCodeV0::MissingKeyframes.as_str()),
        "missingCustomProperty" => Some(OmenaCheckerRuleCodeV0::MissingCustomProperty.as_str()),
        "missingSassSymbol" => Some(OmenaCheckerRuleCodeV0::MissingSassSymbol.as_str()),
        "unusedSelector" => Some(OmenaCheckerRuleCodeV0::UnusedSelector.as_str()),
        "unreachableDeclaration" => Some(OmenaCheckerRuleCodeV0::UnreachableDeclaration.as_str()),
        "deadCascadeLayer" => Some(OmenaCheckerRuleCodeV0::DeadCascadeLayer.as_str()),
        "iacvtProne" | "guaranteedInvalidCustomProperty" => {
            Some(OmenaCheckerRuleCodeV0::IacvtProne.as_str())
        }
        "circularVar" => Some(OmenaCheckerRuleCodeV0::CircularVar.as_str()),
        "unspecifiedCascadeTie" => Some(OmenaCheckerRuleCodeV0::UnspecifiedCascadeTie.as_str()),
        "designerIntentInconsistency" => {
            Some(OmenaCheckerRuleCodeV0::DesignerIntentInconsistency.as_str())
        }
        _ => None,
    }
}

pub fn run_omena_query_checker_cascade_gate_v0(
    input: OmenaCheckerCascadeInputV0,
) -> OmenaQueryCheckerCascadeGateV0 {
    let registered_rules = list_omena_checker_rule_code_names()
        .into_iter()
        .collect::<BTreeSet<_>>();
    let enabled_rule_names = cascade_gate_enabled_rule_names_v0();
    let evaluations = evaluate_omena_checker_cascade_rules(input);
    let emitted_rule_names = evaluations
        .iter()
        .map(|evaluation| evaluation.rule_code_name)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let emitted_rule_set = emitted_rule_names.iter().copied().collect::<BTreeSet<_>>();
    let unregistered_rule_count = emitted_rule_names
        .iter()
        .filter(|rule| !registered_rules.contains(**rule))
        .count();
    let suppressed_rule_names = enabled_rule_names
        .iter()
        .copied()
        .filter(|rule| !emitted_rule_set.contains(rule))
        .collect::<Vec<_>>();

    OmenaQueryCheckerCascadeGateV0 {
        schema_version: "0",
        product: "omena-query-checker-orchestrator.cascade-gate",
        orchestrator_kind: "registered-rule-diagnostic-gate",
        enabled_rule_names,
        emitted_rule_names,
        suppressed_rule_names,
        registered_rule_count: registered_rules.len(),
        unregistered_rule_count,
        evaluation_count: evaluations.len(),
        enforcement_passed: unregistered_rule_count == 0,
        evaluations,
        ready_surfaces: vec![
            "checkerRuleRegistry",
            "checkerCascadeEvaluation",
            "registeredRuleDiagnosticGate",
            "queryDiagnosticHandoff",
        ],
    }
}

fn cascade_gate_enabled_rule_names_v0() -> Vec<&'static str> {
    vec![
        OmenaCheckerRuleCodeV0::UnreachableDeclaration.as_str(),
        OmenaCheckerRuleCodeV0::DeadCascadeLayer.as_str(),
        OmenaCheckerRuleCodeV0::IacvtProne.as_str(),
        OmenaCheckerRuleCodeV0::CircularVar.as_str(),
        OmenaCheckerRuleCodeV0::UnspecifiedCascadeTie.as_str(),
        OmenaCheckerRuleCodeV0::DesignerIntentInconsistency.as_str(),
    ]
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryCheckerRgFlowGateV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub orchestrator_kind: &'static str,
    pub enabled_rule_names: Vec<&'static str>,
    pub emitted_rule_names: Vec<&'static str>,
    pub registered_rule_count: usize,
    pub unregistered_rule_count: usize,
    pub evaluation_count: usize,
    pub enforcement_passed: bool,
    pub evaluations: Vec<OmenaCheckerRgFlowEvaluationV0>,
    pub ready_surfaces: Vec<&'static str>,
}

/// Invoke the real RG-flow coupling-Jacobian-spectrum evaluator and gate its
/// emitted rule codes against the registered checker rule registry.
///
/// The coupling flows are produced by the caller from real parsed stylesheet
/// structure; this gate runs the genuine `estimate_coupling_jacobian_spectrum_v0`
/// mechanism inside `evaluate_omena_checker_rg_flow_rules` and only surfaces
/// `rg-flow-relevant-operator` diagnostics when the spectral radius exceeds one.
pub fn run_omena_query_checker_rg_flow_gate_v0(
    input: OmenaCheckerRgFlowInputV0,
) -> OmenaQueryCheckerRgFlowGateV0 {
    let registered_rules = list_omena_checker_rule_code_names()
        .into_iter()
        .collect::<BTreeSet<_>>();
    let enabled_rule_names = vec![OmenaCheckerRuleCodeV0::RgFlowRelevantOperator.as_str()];
    let evaluations = evaluate_omena_checker_rg_flow_rules(input);
    let emitted_rule_names = evaluations
        .iter()
        .map(|evaluation| evaluation.rule_code_name)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let unregistered_rule_count = emitted_rule_names
        .iter()
        .filter(|rule| !registered_rules.contains(**rule))
        .count();

    OmenaQueryCheckerRgFlowGateV0 {
        schema_version: "0",
        product: "omena-query-checker-orchestrator.rg-flow-gate",
        orchestrator_kind: "registered-rule-diagnostic-gate",
        enabled_rule_names,
        emitted_rule_names,
        registered_rule_count: registered_rules.len(),
        unregistered_rule_count,
        evaluation_count: evaluations.len(),
        enforcement_passed: unregistered_rule_count == 0,
        evaluations,
        ready_surfaces: vec![
            "checkerRuleRegistry",
            "rgFlowCouplingSpectrum",
            "registeredRuleDiagnosticGate",
            "queryDiagnosticHandoff",
        ],
    }
}

/// A dynamic-className call-site context handed to the k-limited flow M-tier
/// gate: the callee under analysis, the call-string that reaches it, and the
/// abstract class value observed at that call site's exit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OmenaQueryCheckerKLimitedFlowContextV0 {
    pub callee_key: String,
    pub call_site_stack: Vec<String>,
    pub exit_value: AbstractClassValueV0,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryCheckerKLimitedFlowMTierContextV0 {
    pub callee_key: String,
    pub call_site_stack: Vec<String>,
    pub context_key: String,
    pub exit_value_kind: &'static str,
    pub exit_value: AbstractClassValueV0,
    pub evaluation_count: usize,
    pub rule_code_names: Vec<&'static str>,
    pub evaluations: Vec<OmenaCheckerMTierEvaluationV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryCheckerKLimitedFlowMTierGateV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub orchestrator_kind: &'static str,
    pub flow_product: &'static str,
    pub context_sensitivity: String,
    pub max_context_depth: usize,
    pub enabled_rule_names: Vec<&'static str>,
    pub emitted_rule_names: Vec<&'static str>,
    pub registered_rule_count: usize,
    pub unregistered_rule_count: usize,
    pub context_count: usize,
    pub evaluation_count: usize,
    pub enforcement_passed: bool,
    pub contexts: Vec<OmenaQueryCheckerKLimitedFlowMTierContextV0>,
    pub ready_surfaces: Vec<&'static str>,
}

/// Run the real k-limited (k-CFA) call-string flow analysis on the supplied
/// dynamic-className call-site contexts and feed each context's joined exit
/// value into the checker M-tier rules, gating the emitted rule codes against
/// the registered checker rule registry.
///
/// This is the product-path handoff that lets the `omena-query` consumer surface
/// raise context-sensitive `no-unknown-dynamic-class` / `no-imprecise-value` /
/// `no-impossible-selector` diagnostics. The `max_context_depth` is the genuine
/// call-string bound `k`: at a low `k`, call sites that share a callee collapse
/// into one context and their exit values are joined (so a finite-set value can
/// include selectors outside the target universe and trip `no-impossible-selector`);
/// at a higher `k`, the call sites separate into distinct contexts so a precise
/// per-context value can be proven clean. The diagnostics therefore change with
/// `k`, driven by `analyze_k_limited_call_site_flows`, not by metadata.
pub fn run_omena_query_checker_k_limited_flow_m_tier_gate_v0(
    contexts: &[OmenaQueryCheckerKLimitedFlowContextV0],
    selector_universe: &[String],
    max_context_depth: usize,
) -> OmenaQueryCheckerKLimitedFlowMTierGateV0 {
    let registered_rules = list_omena_checker_rule_code_names()
        .into_iter()
        .collect::<BTreeSet<_>>();
    let enabled_rule_names = list_omena_checker_m_tier_rule_code_names();

    let flow_inputs = contexts
        .iter()
        .map(|context| KLimitedCallSiteFlowInputV0 {
            callee_key: context.callee_key.clone(),
            call_site_stack: context.call_site_stack.clone(),
            graph: ClassValueFlowGraphV0 {
                context_key: None,
                nodes: vec![ClassValueFlowNodeV0 {
                    id: "exit".to_string(),
                    predecessors: Vec::new(),
                    transfer: ClassValueFlowTransferV0::AssignFacts(
                        external_string_type_facts_from_abstract_class_value(&context.exit_value),
                    ),
                }],
            },
            exit_node_id: "exit".to_string(),
        })
        .collect::<Vec<_>>();
    let flow = analyze_k_limited_call_site_flows(&flow_inputs, max_context_depth);

    let gate_contexts = flow
        .entries
        .into_iter()
        .map(|entry| {
            let evaluations =
                evaluate_omena_checker_m_tier_rules(OmenaCheckerDynamicClassDomainInputV0 {
                    abstract_value: entry.exit_value.clone(),
                    selector_universe: selector_universe.to_vec(),
                });
            let rule_code_names = evaluations
                .iter()
                .map(|evaluation| evaluation.rule_code_name)
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect::<Vec<_>>();

            OmenaQueryCheckerKLimitedFlowMTierContextV0 {
                callee_key: entry.callee_key,
                call_site_stack: entry.call_site_stack,
                context_key: entry.context_key,
                exit_value_kind: abstract_class_value_kind(&entry.exit_value),
                exit_value: entry.exit_value,
                evaluation_count: evaluations.len(),
                rule_code_names,
                evaluations,
            }
        })
        .collect::<Vec<_>>();

    let emitted_rule_names = gate_contexts
        .iter()
        .flat_map(|context| context.rule_code_names.iter().copied())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let unregistered_rule_count = emitted_rule_names
        .iter()
        .filter(|rule| !registered_rules.contains(**rule))
        .count();
    let evaluation_count = gate_contexts
        .iter()
        .map(|context| context.evaluation_count)
        .sum();

    OmenaQueryCheckerKLimitedFlowMTierGateV0 {
        schema_version: "0",
        product: "omena-query-checker-orchestrator.k-limited-flow-m-tier-gate",
        orchestrator_kind: "registered-rule-diagnostic-gate",
        flow_product: flow.product,
        context_sensitivity: flow.context_sensitivity,
        max_context_depth: flow.max_context_depth,
        enabled_rule_names,
        emitted_rule_names,
        registered_rule_count: registered_rules.len(),
        unregistered_rule_count,
        context_count: gate_contexts.len(),
        evaluation_count,
        enforcement_passed: unregistered_rule_count == 0,
        contexts: gate_contexts,
        ready_surfaces: vec![
            "checkerRuleRegistry",
            "kLimitedCallSiteFlow",
            "checkerMTierEvaluation",
            "registeredRuleDiagnosticGate",
            "queryDiagnosticHandoff",
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cascade_gate_filters_emitted_rules_through_registered_checker_codes() {
        let gate = run_omena_query_checker_cascade_gate_v0(OmenaCheckerCascadeInputV0 {
            declarations: vec![
                cascade_declaration(CascadeDeclarationFixture {
                    declaration_id: "base-color",
                    selector: ".btn",
                    property: "color",
                    value: "red",
                    source_order: 1,
                    layer_name: Some("base"),
                    layer_order: Some(0),
                    important: false,
                    var_references: &[],
                }),
                cascade_declaration(CascadeDeclarationFixture {
                    declaration_id: "override-color",
                    selector: ".btn",
                    property: "color",
                    value: "blue",
                    source_order: 2,
                    layer_name: Some("overrides"),
                    layer_order: Some(1),
                    important: false,
                    var_references: &[],
                }),
                cascade_declaration(CascadeDeclarationFixture {
                    declaration_id: "gap-use",
                    selector: ".card",
                    property: "margin",
                    value: "var(--gap)",
                    source_order: 3,
                    layer_name: Some("components"),
                    layer_order: Some(1),
                    important: false,
                    var_references: &["--gap"],
                }),
                cascade_declaration(CascadeDeclarationFixture {
                    declaration_id: "tie-a",
                    selector: ".button--primary",
                    property: "color",
                    value: "red",
                    source_order: 4,
                    layer_name: None,
                    layer_order: None,
                    important: false,
                    var_references: &[],
                }),
                cascade_declaration(CascadeDeclarationFixture {
                    declaration_id: "tie-b",
                    selector: ".button--primary",
                    property: "color",
                    value: "green",
                    source_order: 5,
                    layer_name: None,
                    layer_order: None,
                    important: false,
                    var_references: &[],
                }),
            ],
            custom_properties: vec![
                OmenaCheckerCustomPropertyInputV0 {
                    name: "--gap".to_string(),
                    dependencies: Vec::new(),
                    guaranteed_invalid: true,
                },
                OmenaCheckerCustomPropertyInputV0 {
                    name: "--a".to_string(),
                    dependencies: vec!["--b".to_string()],
                    guaranteed_invalid: false,
                },
                OmenaCheckerCustomPropertyInputV0 {
                    name: "--b".to_string(),
                    dependencies: vec!["--a".to_string()],
                    guaranteed_invalid: false,
                },
            ],
        });

        assert!(gate.enforcement_passed);
        assert_eq!(gate.unregistered_rule_count, 0);
        assert!(gate.emitted_rule_names.contains(&"unreachable-declaration"));
        assert!(gate.emitted_rule_names.contains(&"dead-cascade-layer"));
        assert!(gate.emitted_rule_names.contains(&"iacvt-prone"));
        assert!(gate.emitted_rule_names.contains(&"circular-var"));
        assert!(gate.emitted_rule_names.contains(&"unspecified-cascade-tie"));
        assert!(
            gate.emitted_rule_names
                .contains(&"designer-intent-inconsistency")
        );
        assert!(gate.suppressed_rule_names.is_empty());
        assert!(gate.evaluation_count >= gate.emitted_rule_names.len());
    }

    #[test]
    fn cascade_gate_records_clear_suppression_for_clean_fixture() {
        let gate = run_omena_query_checker_cascade_gate_v0(OmenaCheckerCascadeInputV0 {
            declarations: vec![cascade_declaration(CascadeDeclarationFixture {
                declaration_id: "only-color",
                selector: ".btn",
                property: "color",
                value: "red",
                source_order: 1,
                layer_name: None,
                layer_order: None,
                important: false,
                var_references: &[],
            })],
            custom_properties: Vec::new(),
        });

        assert!(gate.enforcement_passed);
        assert_eq!(gate.evaluation_count, 0);
        assert_eq!(gate.suppressed_rule_names, gate.enabled_rule_names);
    }

    #[test]
    fn product_diagnostic_gate_maps_query_diagnostics_to_registered_checker_rules() {
        let product_codes = [
            "missingModule",
            "missingSelector",
            "missingStaticClass",
            "missingTemplatePrefix",
            "missingResolvedClassValues",
            "missingResolvedClassDomain",
            "missingComposedModule",
            "missingComposedSelector",
            "missingValueModule",
            "missingImportedValue",
            "missingKeyframes",
            "missingCustomProperty",
            "missingSassSymbol",
            "unusedSelector",
            "unreachableDeclaration",
            "deadCascadeLayer",
            "iacvtProne",
            "guaranteedInvalidCustomProperty",
            "circularVar",
            "unspecifiedCascadeTie",
            "designerIntentInconsistency",
        ];

        for product_code in product_codes {
            let gate = gate_omena_query_checker_product_diagnostic_code_v0(product_code);
            assert!(
                gate.enforcement_passed,
                "{product_code} must map to a registered checker rule"
            );
            assert!(gate.checker_owned);
            assert!(gate.checker_rule_registered);
            assert_eq!(
                gate.provenance,
                vec![
                    "omena-query-checker-orchestrator.product-diagnostic-gate",
                    "omena-checker.rule-registry",
                ]
            );
        }
    }

    #[test]
    fn rg_flow_gate_emits_relevant_operator_for_divergent_coupling_flow() {
        let gate = run_omena_query_checker_rg_flow_gate_v0(OmenaCheckerRgFlowInputV0 {
            flows: vec![OmenaCheckerRgFlowCouplingInputV0 {
                workspace_path: "workspace://divergent-token-graph".to_string(),
                before: rg_flow_coupling(2, 2, 0, 0),
                after: rg_flow_coupling(2, 2, 2, 0),
            }],
        });

        assert!(gate.enforcement_passed);
        assert_eq!(gate.unregistered_rule_count, 0);
        assert_eq!(gate.evaluation_count, 1);
        assert!(
            gate.emitted_rule_names
                .contains(&"rg-flow-relevant-operator")
        );
        assert!(gate.evaluations[0].spectral_radius > 1.0);
        assert_eq!(
            gate.evaluations[0].mechanism_products,
            vec!["omena-rg-flow.coupling-jacobian-spectrum"]
        );
    }

    #[test]
    fn rg_flow_gate_records_clear_suppression_for_settled_coupling_flow() {
        let gate = run_omena_query_checker_rg_flow_gate_v0(OmenaCheckerRgFlowInputV0 {
            flows: vec![OmenaCheckerRgFlowCouplingInputV0 {
                workspace_path: "workspace://settled-token-graph".to_string(),
                before: rg_flow_coupling(2, 2, 0, 0),
                after: rg_flow_coupling(2, 2, 0, 0),
            }],
        });

        assert!(gate.enforcement_passed);
        assert_eq!(gate.evaluation_count, 0);
        assert!(gate.emitted_rule_names.is_empty());
    }

    #[test]
    fn product_diagnostic_gate_leaves_non_checker_advisories_unowned() {
        let gate = gate_omena_query_checker_product_diagnostic_code_v0("deprecatedSassImport");

        assert!(gate.enforcement_passed);
        assert!(!gate.checker_owned);
        assert_eq!(gate.checker_rule_code_name, None);
        assert!(gate.provenance.is_empty());
    }

    #[test]
    fn k_limited_flow_m_tier_gate_changes_diagnostics_with_context_depth() {
        let contexts = vec![
            OmenaQueryCheckerKLimitedFlowContextV0 {
                callee_key: "classForVariant".to_string(),
                call_site_stack: vec![
                    "RouteA.tsx:render".to_string(),
                    "PrimaryButton.tsx:className".to_string(),
                ],
                exit_value: AbstractClassValueV0::Exact {
                    value: "btn-primary".to_string(),
                },
            },
            OmenaQueryCheckerKLimitedFlowContextV0 {
                callee_key: "classForVariant".to_string(),
                call_site_stack: vec![
                    "RouteB.tsx:render".to_string(),
                    "SecondaryButton.tsx:className".to_string(),
                ],
                exit_value: AbstractClassValueV0::Exact {
                    value: "btn-secondary".to_string(),
                },
            },
        ];
        let selector_universe = vec!["btn-primary".to_string()];

        let zero_cfa =
            run_omena_query_checker_k_limited_flow_m_tier_gate_v0(&contexts, &selector_universe, 0);
        let two_cfa =
            run_omena_query_checker_k_limited_flow_m_tier_gate_v0(&contexts, &selector_universe, 2);

        assert!(zero_cfa.enforcement_passed);
        assert!(two_cfa.enforcement_passed);
        assert_eq!(zero_cfa.context_sensitivity, "0-cfa");
        assert_eq!(two_cfa.context_sensitivity, "2-cfa");

        // 0-CFA collapses both call sites into the shared <root> context. The
        // joined exit value is the finite set {btn-primary, btn-secondary}, so
        // every root-context entry sees btn-secondary outside the btn-primary
        // universe and reports an impossible selector.
        let zero_root = context_ending_with(&zero_cfa, "<root>");
        assert_eq!(
            zero_root.len(),
            2,
            "0-cfa must keep both call sites under the shared root context key"
        );
        assert!(
            zero_root
                .iter()
                .all(|context| context.rule_code_names.contains(&"no-impossible-selector")),
            "0-CFA root join must surface a missing secondary selector for every entry"
        );

        // 2-CFA separates the two call sites; the primary call site narrows to a
        // clean exact btn-primary value with no diagnostics, while the secondary
        // keeps its impossible-selector finding.
        let two_primary =
            context_ending_with(&two_cfa, "RouteA.tsx:render > PrimaryButton.tsx:className");
        let two_secondary = context_ending_with(
            &two_cfa,
            "RouteB.tsx:render > SecondaryButton.tsx:className",
        );
        assert_eq!(
            two_primary.len(),
            1,
            "2-cfa must keep a distinct primary call-site context"
        );
        assert_eq!(
            two_secondary.len(),
            1,
            "2-cfa must keep a distinct secondary call-site context"
        );
        assert_eq!(
            two_primary[0].evaluation_count, 0,
            "2-CFA must narrow the primary call-site to a clean exact selector"
        );
        assert!(
            two_secondary[0]
                .rule_code_names
                .contains(&"no-impossible-selector"),
            "2-CFA must keep the secondary call-site diagnostic"
        );

        // The load-bearing claim: increasing context depth changes the diagnostic
        // output, not just metadata.
        assert!(
            zero_cfa.evaluation_count > two_cfa.evaluation_count,
            "increasing k must change the M-tier evaluation output"
        );
    }

    fn context_ending_with<'a>(
        gate: &'a OmenaQueryCheckerKLimitedFlowMTierGateV0,
        suffix: &str,
    ) -> Vec<&'a OmenaQueryCheckerKLimitedFlowMTierContextV0> {
        gate.contexts
            .iter()
            .filter(|context| context.context_key.ends_with(suffix))
            .collect()
    }

    struct CascadeDeclarationFixture<'a> {
        declaration_id: &'a str,
        selector: &'a str,
        property: &'a str,
        value: &'a str,
        source_order: u32,
        layer_name: Option<&'a str>,
        layer_order: Option<i32>,
        important: bool,
        var_references: &'a [&'a str],
    }

    fn rg_flow_coupling(
        k_env: usize,
        k_decl: usize,
        k_cycle: usize,
        k_dirty: usize,
    ) -> OmenaCheckerRgFlowCouplingSpaceInputV0 {
        OmenaCheckerRgFlowCouplingSpaceInputV0 {
            k_env,
            k_decl,
            k_cycle,
            k_dirty,
        }
    }

    fn cascade_declaration(
        fixture: CascadeDeclarationFixture<'_>,
    ) -> OmenaCheckerCascadeDeclarationInputV0 {
        OmenaCheckerCascadeDeclarationInputV0 {
            declaration_id: fixture.declaration_id.to_string(),
            selector: fixture.selector.to_string(),
            property: fixture.property.to_string(),
            value: fixture.value.to_string(),
            source_order: fixture.source_order,
            condition_context: Vec::new(),
            layer_name: fixture.layer_name.map(str::to_string),
            layer_order: fixture.layer_order,
            important: fixture.important,
            var_references: fixture
                .var_references
                .iter()
                .map(|name| (*name).to_string())
                .collect(),
        }
    }
}
