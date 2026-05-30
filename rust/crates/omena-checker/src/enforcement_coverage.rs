use std::collections::BTreeSet;

use serde::Serialize;

use super::{OmenaCheckerRuleCodeV0, list_omena_checker_rule_codes};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaCheckerRuleEnforcementEvidenceV0 {
    pub rule_code: OmenaCheckerRuleCodeV0,
    pub rule_code_name: &'static str,
    pub evidence_kind: &'static str,
    pub product_path: &'static str,
    pub emit_fixture: &'static str,
    pub clear_fixture: &'static str,
    pub mechanism_products: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaCheckerRuleEnforcementCoverageV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub registered_rule_count: usize,
    pub mapped_rule_count: usize,
    pub direct_evaluator_rule_count: usize,
    pub mechanism_evaluator_rule_count: usize,
    pub product_diagnostic_gate_rule_count: usize,
    pub missing_rule_names: Vec<&'static str>,
    pub extra_rule_names: Vec<&'static str>,
    pub coverage_passed: bool,
    pub evidence: Vec<OmenaCheckerRuleEnforcementEvidenceV0>,
}

pub fn summarize_omena_checker_rule_enforcement_coverage_v0()
-> OmenaCheckerRuleEnforcementCoverageV0 {
    let registered_rule_names = list_omena_checker_rule_codes()
        .into_iter()
        .map(OmenaCheckerRuleCodeV0::as_str)
        .collect::<BTreeSet<_>>();
    let evidence = list_omena_checker_rule_enforcement_evidence_v0();
    let mapped_rule_names = evidence
        .iter()
        .map(|entry| entry.rule_code_name)
        .collect::<BTreeSet<_>>();
    let missing_rule_names = registered_rule_names
        .difference(&mapped_rule_names)
        .copied()
        .collect::<Vec<_>>();
    let extra_rule_names = mapped_rule_names
        .difference(&registered_rule_names)
        .copied()
        .collect::<Vec<_>>();

    OmenaCheckerRuleEnforcementCoverageV0 {
        schema_version: "0",
        product: "omena-checker.rule-enforcement-coverage",
        registered_rule_count: registered_rule_names.len(),
        mapped_rule_count: mapped_rule_names.len(),
        direct_evaluator_rule_count: count_evidence_kind(&evidence, "directEvaluator"),
        mechanism_evaluator_rule_count: count_evidence_kind(&evidence, "mechanismEvaluator"),
        product_diagnostic_gate_rule_count: count_evidence_kind(&evidence, "productDiagnosticGate"),
        missing_rule_names,
        extra_rule_names,
        coverage_passed: registered_rule_names == mapped_rule_names,
        evidence,
    }
}

pub fn list_omena_checker_rule_enforcement_evidence_v0()
-> Vec<OmenaCheckerRuleEnforcementEvidenceV0> {
    use OmenaCheckerRuleCodeV0::{
        CascadeDeepConflict, CascadeSMTViolation, CascadeUnreachableRule,
        CategoricalCascadeEvidenceInconsistency, CircularVar, DeadCascadeLayer,
        DesignSystemMdlBudget, DesignerIntentInconsistency, IacvtProne, MissingComposedModule,
        MissingComposedSelector, MissingCustomProperty, MissingImportedValue, MissingKeyframes,
        MissingModule, MissingResolvedClassDomain, MissingResolvedClassValues, MissingSassSymbol,
        MissingStaticClass, MissingTemplatePrefix, MissingValueModule, NoImpossibleSelector,
        NoImpreciseValue, NoUnknownDynamicClass, ReplicaEnsembleInconsistency,
        RgFlowRelevantOperator, StreamingIfdsPrecisionParity, UnreachableDeclaration,
        UnspecifiedCascadeTie, UnusedSelector,
    };

    vec![
        direct(
            NoUnknownDynamicClass,
            "engine-shadow-runner.omena-checker-m-tier-evaluations",
            "evaluates_m_tier_unknown_and_impossible_dynamic_classes",
            "evaluates_finite_dynamic_class_domains",
        ),
        direct(
            NoImpreciseValue,
            "engine-shadow-runner.omena-checker-m-tier-evaluations",
            "evaluates_m_tier_imprecise_domains_without_unknown_values",
            "evaluates_finite_dynamic_class_domains",
        ),
        direct(
            NoImpossibleSelector,
            "engine-shadow-runner.omena-checker-m-tier-evaluations",
            "evaluates_m_tier_unknown_and_impossible_dynamic_classes",
            "evaluates_finite_dynamic_class_domains",
        ),
        product_gate(MissingModule),
        product_gate(MissingStaticClass),
        product_gate(MissingTemplatePrefix),
        product_gate(MissingResolvedClassValues),
        product_gate(MissingResolvedClassDomain),
        product_gate(UnusedSelector),
        product_gate(MissingComposedModule),
        product_gate(MissingComposedSelector),
        product_gate(MissingValueModule),
        product_gate(MissingImportedValue),
        product_gate(MissingKeyframes),
        product_gate(MissingCustomProperty),
        product_gate(MissingSassSymbol),
        direct(
            UnreachableDeclaration,
            "omena-query-checker-orchestrator.cascade-gate",
            "cascade_gate_filters_emitted_rules_through_registered_checker_codes",
            "cascade_gate_records_clear_suppression_for_clean_fixture",
        ),
        direct(
            DeadCascadeLayer,
            "omena-query-checker-orchestrator.cascade-gate",
            "cascade_gate_filters_emitted_rules_through_registered_checker_codes",
            "cascade_gate_records_clear_suppression_for_clean_fixture",
        ),
        direct(
            IacvtProne,
            "omena-query-checker-orchestrator.cascade-gate",
            "cascade_gate_filters_emitted_rules_through_registered_checker_codes",
            "cascade_gate_records_clear_suppression_for_clean_fixture",
        ),
        direct(
            CircularVar,
            "omena-query-checker-orchestrator.cascade-gate",
            "cascade_gate_filters_emitted_rules_through_registered_checker_codes",
            "cascade_gate_records_clear_suppression_for_clean_fixture",
        ),
        direct(
            UnspecifiedCascadeTie,
            "omena-query-checker-orchestrator.cascade-gate",
            "cascade_gate_filters_emitted_rules_through_registered_checker_codes",
            "cascade_gate_records_clear_suppression_for_clean_fixture",
        ),
        mechanism(
            DesignerIntentInconsistency,
            "omena-query-checker-orchestrator.cascade-gate",
            "cascade_gate_filters_emitted_rules_through_registered_checker_codes",
            "cascade_gate_records_clear_suppression_for_clean_fixture",
            &["omena-variational.designer-intent-posterior"],
        ),
        mechanism(
            CascadeSMTViolation,
            "engine-shadow-runner.omena-checker-smt-evaluations",
            "evaluates_smt_rule_family_from_canonical_obligations",
            "evaluates_smt_rule_family_from_canonical_obligations",
            &["omena-smt.backend-check"],
        ),
        mechanism(
            DesignSystemMdlBudget,
            "engine-shadow-runner.omena-checker-mdl-evaluations",
            "evaluates_mdl_budget_rule_family_from_query_mdl_summaries",
            "evaluates_mdl_budget_rule_family_from_query_mdl_summaries",
            &["omena-query.design-system-minimum-description"],
        ),
        mechanism(
            StreamingIfdsPrecisionParity,
            "engine-shadow-runner.omena-checker-streaming-ifds-evaluations",
            "evaluates_streaming_ifds_precision_parity_rule_family",
            "evaluates_streaming_ifds_precision_parity_rule_family",
            &["omena-streaming-ifds.analysis-report"],
        ),
        mechanism(
            RgFlowRelevantOperator,
            "omena-query-checker-orchestrator.rg-flow-gate",
            "rg_flow_gate_emits_relevant_operator_for_divergent_coupling_flow",
            "rg_flow_gate_records_clear_suppression_for_settled_coupling_flow",
            &["omena-rg-flow.coupling-jacobian-spectrum"],
        ),
        mechanism(
            ReplicaEnsembleInconsistency,
            "engine-shadow-runner.omena-checker-replica-ensemble-evaluations",
            "evaluates_replica_ensemble_inconsistency_rule_family",
            "evaluates_replica_ensemble_inconsistency_rule_family",
            &["omena-ensemble.cross-file-inconsistency-report"],
        ),
        mechanism(
            CascadeDeepConflict,
            "engine-shadow-runner.omena-checker-grn-evaluations",
            "evaluates_grn_rule_family_from_cascade_projection",
            "evaluates_grn_rule_family_from_cascade_projection",
            &["omena-cascade.grn-outcome-projection"],
        ),
        mechanism(
            CascadeUnreachableRule,
            "engine-shadow-runner.omena-checker-grn-evaluations",
            "evaluates_grn_rule_family_from_cascade_projection",
            "evaluates_grn_rule_family_from_cascade_projection",
            &["omena-cascade.grn-outcome-projection"],
        ),
        mechanism(
            CategoricalCascadeEvidenceInconsistency,
            "omena-query-checker-orchestrator.categorical-gate",
            "categorical_gate_emits_inconsistency_for_non_functorial_role_mapping",
            "categorical_gate_records_clear_suppression_for_composable_role_mapping",
            &["omena-categorical.cascade-primitive-role-functor"],
        ),
    ]
}

fn product_gate(rule_code: OmenaCheckerRuleCodeV0) -> OmenaCheckerRuleEnforcementEvidenceV0 {
    evidence(
        rule_code,
        "productDiagnosticGate",
        "omena-query-checker-orchestrator.product-diagnostic-gate",
        "product_diagnostic_gate_maps_query_diagnostics_to_registered_checker_rules",
        "product_diagnostic_gate_leaves_non_checker_advisories_unowned",
        &[],
    )
}

fn direct(
    rule_code: OmenaCheckerRuleCodeV0,
    product_path: &'static str,
    emit_fixture: &'static str,
    clear_fixture: &'static str,
) -> OmenaCheckerRuleEnforcementEvidenceV0 {
    evidence(
        rule_code,
        "directEvaluator",
        product_path,
        emit_fixture,
        clear_fixture,
        &[],
    )
}

fn mechanism(
    rule_code: OmenaCheckerRuleCodeV0,
    product_path: &'static str,
    emit_fixture: &'static str,
    clear_fixture: &'static str,
    mechanism_products: &[&'static str],
) -> OmenaCheckerRuleEnforcementEvidenceV0 {
    evidence(
        rule_code,
        "mechanismEvaluator",
        product_path,
        emit_fixture,
        clear_fixture,
        mechanism_products,
    )
}

fn evidence(
    rule_code: OmenaCheckerRuleCodeV0,
    evidence_kind: &'static str,
    product_path: &'static str,
    emit_fixture: &'static str,
    clear_fixture: &'static str,
    mechanism_products: &[&'static str],
) -> OmenaCheckerRuleEnforcementEvidenceV0 {
    OmenaCheckerRuleEnforcementEvidenceV0 {
        rule_code,
        rule_code_name: rule_code.as_str(),
        evidence_kind,
        product_path,
        emit_fixture,
        clear_fixture,
        mechanism_products: mechanism_products.to_vec(),
    }
}

fn count_evidence_kind(
    evidence: &[OmenaCheckerRuleEnforcementEvidenceV0],
    evidence_kind: &str,
) -> usize {
    evidence
        .iter()
        .filter(|entry| entry.evidence_kind == evidence_kind)
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enforcement_coverage_maps_every_registered_rule_once() {
        let summary = summarize_omena_checker_rule_enforcement_coverage_v0();

        assert!(summary.coverage_passed);
        assert_eq!(summary.registered_rule_count, 30);
        assert_eq!(summary.mapped_rule_count, summary.registered_rule_count);
        assert!(summary.missing_rule_names.is_empty());
        assert!(summary.extra_rule_names.is_empty());
        assert_eq!(
            summary
                .evidence
                .iter()
                .map(|entry| entry.rule_code_name)
                .collect::<BTreeSet<_>>()
                .len(),
            summary.evidence.len()
        );
    }

    #[test]
    fn enforcement_coverage_keeps_mechanism_rules_product_path_backed() {
        let summary = summarize_omena_checker_rule_enforcement_coverage_v0();
        let mechanism_evidence = summary
            .evidence
            .iter()
            .filter(|entry| entry.evidence_kind == "mechanismEvaluator")
            .collect::<Vec<_>>();

        assert_eq!(mechanism_evidence.len(), 9);
        assert!(mechanism_evidence.iter().all(|entry| {
            !entry.product_path.is_empty()
                && !entry.emit_fixture.is_empty()
                && !entry.clear_fixture.is_empty()
                && !entry.mechanism_products.is_empty()
        }));
    }
}
