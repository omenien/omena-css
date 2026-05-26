//! Checker diagnostic orchestration boundary for the `omena-query` facade.
//!
//! `omena-query` owns the consumer-facing diagnostic API. This crate owns the
//! narrower checker handoff: it invokes `omena-checker`, verifies that emitted
//! rule codes are registered, and returns a gate summary alongside evaluations.

use std::collections::BTreeSet;

use serde::Serialize;

pub use omena_checker::{
    CategoricalCascadeEvidenceV0, OmenaCheckerCascadeDeclarationInputV0,
    OmenaCheckerCascadeEvaluationV0, OmenaCheckerCascadeInputV0, OmenaCheckerCustomPropertyInputV0,
    OmenaCheckerRuleCodeV0, checker_categorical_cascade_evidence_v0,
};
use omena_checker::{evaluate_omena_checker_cascade_rules, list_omena_checker_rule_code_names};

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
    fn product_diagnostic_gate_leaves_non_checker_advisories_unowned() {
        let gate = gate_omena_query_checker_product_diagnostic_code_v0("deprecatedSassImport");

        assert!(gate.enforcement_passed);
        assert!(!gate.checker_owned);
        assert_eq!(gate.checker_rule_code_name, None);
        assert!(gate.provenance.is_empty());
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
