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
    CanonicalSelector, CategoricalCascadeEvidenceV0, OmenaCheckerCascadeDeclarationInputV0,
    OmenaCheckerCascadeEvaluationV0, OmenaCheckerCascadeInputV0,
    OmenaCheckerCategoricalEvaluationV0, OmenaCheckerCategoricalInputV0,
    OmenaCheckerCategoricalPrimitiveRolePairInputV0, OmenaCheckerCategoricalRoleMappingInputV0,
    OmenaCheckerCustomPropertyInputV0, OmenaCheckerCustomPropertyRegistrationInputV0,
    OmenaCheckerMTierEvaluationV0, OmenaCheckerReplicaEnsembleEvaluationV0,
    OmenaCheckerReplicaEnsembleInputV0, OmenaCheckerReplicaEnsembleReportInputV0,
    OmenaCheckerRgFlowCouplingInputV0, OmenaCheckerRgFlowCouplingSpaceInputV0,
    OmenaCheckerRgFlowEvaluationV0, OmenaCheckerRgFlowInputV0, OmenaCheckerRuleCodeV0,
    OmenaCheckerSmtEvaluationV0, OmenaCheckerSmtInputV0,
    OmenaCheckerSmtLayerInversionDeclarationInputV0, OmenaCheckerSmtLayerInversionInputV0,
    OmenaCheckerSmtLayerInversionObligationInputV0, OmenaCheckerSmtObligationInputV0,
    RG_FLOW_DEFAULT_PRODUCT_DECISION_MECHANISM_V0, RG_FLOW_MECHANISM_SCOPE_V0,
    RG_FLOW_PRODUCT_SURFACE_V0, checker_cascade_primitive_role_catalog_v0,
    checker_categorical_cascade_evidence_for_exercised_primitives_v0,
    checker_categorical_cascade_evidence_v0,
};
use omena_checker::{
    OmenaCheckerDynamicClassDomainInputV0, active_omena_checker_smt_backend_kind_name_v0,
    active_omena_checker_smt_product_scope_v0, active_omena_checker_smt_solver_backed_v0,
    evaluate_omena_checker_cascade_rules, evaluate_omena_checker_categorical_rules,
    evaluate_omena_checker_m_tier_rules, evaluate_omena_checker_replica_ensemble_rules,
    evaluate_omena_checker_rg_flow_rules, evaluate_omena_checker_smt_layer_inversion_rules,
    evaluate_omena_checker_smt_rules, list_omena_checker_m_tier_rule_code_names,
    list_omena_checker_rule_code_names,
};
pub use omena_product_hints::{
    CATEGORICAL_FEATURE_GATE_V0, CATEGORICAL_LAYER_MARKER_V0, CATEGORICAL_SCHEMA_VERSION_V0,
    DesignSystemEdgeKindCountV0, DesignSystemInvariantSummaryV0, DesignSystemModelV0,
    DesignSystemProjectSummaryInputV0,
};
pub use omena_product_hints::{
    ModuleGraphEdgeV0, ModuleGraphV0, OutcomeMode,
    REPLICA_ENSEMBLE_DEFAULT_PRODUCT_DECISION_MECHANISM_V0, REPLICA_ENSEMBLE_FEATURE_GATE_V0,
    REPLICA_ENSEMBLE_LAYER_MARKER_V0, REPLICA_ENSEMBLE_MECHANISM_SCOPE_V0,
    REPLICA_ENSEMBLE_PRODUCT_SURFACE_V0, REPLICA_ENSEMBLE_SCHEMA_VERSION_V0, ReplicaSiteOutcomeV0,
    ReplicaSnapshotV0, ReportOptionsV0, ReportRecommendation,
    build_cross_file_inconsistency_report, site,
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
        "registeredPropertyTypeMismatch" => {
            Some(OmenaCheckerRuleCodeV0::RegisteredPropertyTypeMismatch.as_str())
        }
        "invalidPropertyValue" => Some(OmenaCheckerRuleCodeV0::InvalidPropertyValue.as_str()),
        "unspecifiedCascadeTie" => Some(OmenaCheckerRuleCodeV0::UnspecifiedCascadeTie.as_str()),
        "designerIntentInconsistency" => {
            Some(OmenaCheckerRuleCodeV0::DesignerIntentInconsistency.as_str())
        }
        "cascadeSmtViolation" => Some(OmenaCheckerRuleCodeV0::CascadeSMTViolation.as_str()),
        "rgFlowRelevantOperator" => Some(OmenaCheckerRuleCodeV0::RgFlowRelevantOperator.as_str()),
        "categoricalCascadeEvidenceInconsistency" => {
            Some(OmenaCheckerRuleCodeV0::CategoricalCascadeEvidenceInconsistency.as_str())
        }
        "replicaEnsembleInconsistency" => {
            Some(OmenaCheckerRuleCodeV0::ReplicaEnsembleInconsistency.as_str())
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
        OmenaCheckerRuleCodeV0::RegisteredPropertyTypeMismatch.as_str(),
        OmenaCheckerRuleCodeV0::InvalidPropertyValue.as_str(),
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
    pub mechanism_scope: &'static str,
    pub product_surface: &'static str,
    pub default_product_decision_mechanism: bool,
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
        mechanism_scope: RG_FLOW_MECHANISM_SCOPE_V0,
        product_surface: RG_FLOW_PRODUCT_SURFACE_V0,
        default_product_decision_mechanism: RG_FLOW_DEFAULT_PRODUCT_DECISION_MECHANISM_V0,
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
            "rgFlowOptInDeepAnalysisHintScope",
            "registeredRuleDiagnosticGate",
            "queryDiagnosticHandoff",
        ],
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryCheckerCategoricalGateV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub orchestrator_kind: &'static str,
    pub enabled_rule_names: Vec<&'static str>,
    pub emitted_rule_names: Vec<&'static str>,
    pub registered_rule_count: usize,
    pub unregistered_rule_count: usize,
    pub evaluation_count: usize,
    pub enforcement_passed: bool,
    pub evaluations: Vec<OmenaCheckerCategoricalEvaluationV0>,
    pub ready_surfaces: Vec<&'static str>,
}

/// Invoke the real categorical cascade primitive-to-role functor evaluator and
/// gate its emitted rule codes against the registered checker rule registry.
///
/// The role mappings are produced by the caller from the cascade primitives a
/// real parsed stylesheet exercises; this gate runs the genuine
/// `apply_cascade_role_mapping_functor_v0` verdict inside
/// `evaluate_omena_checker_categorical_rules` and only surfaces
/// `categorical-cascade-evidence-inconsistency` diagnostics when the functor
/// rejects the mapping (identity or composition is not witnessable). A mapping
/// that exercises enough distinct primitives to witness composition is accepted
/// by the functor and nothing is surfaced.
pub fn run_omena_query_checker_categorical_gate_v0(
    input: OmenaCheckerCategoricalInputV0,
) -> OmenaQueryCheckerCategoricalGateV0 {
    let registered_rules = list_omena_checker_rule_code_names()
        .into_iter()
        .collect::<BTreeSet<_>>();
    let enabled_rule_names =
        vec![OmenaCheckerRuleCodeV0::CategoricalCascadeEvidenceInconsistency.as_str()];
    let evaluations = evaluate_omena_checker_categorical_rules(input);
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

    OmenaQueryCheckerCategoricalGateV0 {
        schema_version: "0",
        product: "omena-query-checker-orchestrator.categorical-gate",
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
            "categoricalCascadeRoleFunctor",
            "registeredRuleDiagnosticGate",
            "queryDiagnosticHandoff",
        ],
    }
}

/// Build the cross-project design-system model through the query checker
/// boundary so `omena-query` does not depend directly on theory crates.
pub fn build_omena_query_checker_design_system_model_from_project_summary_v0(
    theory_id: impl Into<String>,
    input: DesignSystemProjectSummaryInputV0,
) -> DesignSystemModelV0 {
    omena_product_hints::design_system_model_from_project_summary_v0(theory_id, input)
}

/// Compare design-system models through the query checker boundary. This keeps
/// the product path explicit: `omena-query` -> orchestrator -> categorical.
pub fn compare_omena_query_checker_design_system_models_for_invariant_v0(
    invariant_id: impl Into<String>,
    models: &[DesignSystemModelV0],
) -> DesignSystemInvariantSummaryV0 {
    omena_product_hints::compare_design_system_models_for_invariant_v0(invariant_id, models)
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryCheckerReplicaEnsembleGateV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub orchestrator_kind: &'static str,
    pub enabled_rule_names: Vec<&'static str>,
    pub emitted_rule_names: Vec<&'static str>,
    pub registered_rule_count: usize,
    pub unregistered_rule_count: usize,
    pub evaluation_count: usize,
    pub enforcement_passed: bool,
    pub mechanism_scope: &'static str,
    pub product_surface: &'static str,
    pub default_product_decision_mechanism: bool,
    pub evaluations: Vec<OmenaCheckerReplicaEnsembleEvaluationV0>,
    pub ready_surfaces: Vec<&'static str>,
}

/// Invoke the real replica-ensemble cross-file inconsistency evaluator and gate
/// its emitted rule codes against the registered checker rule registry.
///
/// The reports are produced by the caller from a real cross-file replica overlap
/// computation: each report carries the `recommendation`, `meanQ`, `varianceQ`,
/// and disagreement-pair count that `omena-ensemble`'s
/// `build_cross_file_inconsistency_report` derived from per-file cascade winners.
/// This gate runs the genuine `evaluate_omena_checker_replica_ensemble_rules`
/// mechanism, which only emits `replica-ensemble-inconsistency` when a report's
/// overlap statistics signal a disagreement (recommendation other than
/// `noActionNeeded`, a non-empty disagreement-pair set, or `meanQ < 1.0`). A
/// consistent ensemble (every shared cascade outcome agrees, `meanQ == 1.0`, no
/// disagreement pairs, `noActionNeeded`) is filtered out and nothing is surfaced,
/// so the diagnostic depends on the overlap statistics over the real winners, not
/// on a literal.
pub fn run_omena_query_checker_replica_ensemble_gate_v0(
    input: OmenaCheckerReplicaEnsembleInputV0,
) -> OmenaQueryCheckerReplicaEnsembleGateV0 {
    let registered_rules = list_omena_checker_rule_code_names()
        .into_iter()
        .collect::<BTreeSet<_>>();
    let enabled_rule_names = vec![OmenaCheckerRuleCodeV0::ReplicaEnsembleInconsistency.as_str()];
    let evaluations = evaluate_omena_checker_replica_ensemble_rules(input);
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

    OmenaQueryCheckerReplicaEnsembleGateV0 {
        schema_version: "0",
        product: "omena-query-checker-orchestrator.replica-ensemble-gate",
        orchestrator_kind: "registered-rule-diagnostic-gate",
        enabled_rule_names,
        emitted_rule_names,
        registered_rule_count: registered_rules.len(),
        unregistered_rule_count,
        evaluation_count: evaluations.len(),
        enforcement_passed: unregistered_rule_count == 0,
        mechanism_scope: REPLICA_ENSEMBLE_MECHANISM_SCOPE_V0,
        product_surface: REPLICA_ENSEMBLE_PRODUCT_SURFACE_V0,
        default_product_decision_mechanism: REPLICA_ENSEMBLE_DEFAULT_PRODUCT_DECISION_MECHANISM_V0,
        evaluations,
        ready_surfaces: vec![
            "checkerRuleRegistry",
            "crossFileReplicaOverlap",
            "crossFileReplicaEnsembleHintScope",
            "registeredRuleDiagnosticGate",
            "queryDiagnosticHandoff",
        ],
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryCheckerSmtGateV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub orchestrator_kind: &'static str,
    pub backend_kind_name: &'static str,
    pub solver_backed: bool,
    pub product_scope: &'static str,
    pub enabled_rule_names: Vec<&'static str>,
    pub emitted_rule_names: Vec<&'static str>,
    pub registered_rule_count: usize,
    pub unregistered_rule_count: usize,
    pub evaluation_count: usize,
    pub enforcement_passed: bool,
    pub evaluations: Vec<OmenaCheckerSmtEvaluationV0>,
    pub ready_surfaces: Vec<&'static str>,
}

/// Invoke the real SMT cascade proof-obligation evaluator and gate its emitted
/// rule codes against the registered checker rule registry.
///
/// The obligations are produced by the caller from a real parsed cascade signal
/// (e.g. the canonical box-shorthand combination obligation derived from a
/// stylesheet's longhand declarations): each obligation carries the
/// `require:name=bool` literals that encode the cascade-safety preconditions for
/// that flatten/combination rewrite. This gate runs the genuine
/// `evaluate_omena_checker_smt_rules` mechanism, which discharges each obligation
/// through the active SMT backend. The default product build stays solver-free
/// and uses the product-owned propositional backend; builds that opt into the
/// `smt-z3` feature route the same product gate through the z3 backend. The gate only surfaces
/// `cascade.smt-violation`
/// diagnostics when the backend's verdict on the conjunction is `Unsat` — i.e. a
/// required precondition is violated. An obligation whose preconditions all hold
/// is satisfiable and nothing is surfaced; the diagnostic therefore depends on
/// the solver verdict over the parsed facts, not on a literal.
pub fn run_omena_query_checker_smt_gate_v0(
    input: OmenaCheckerSmtInputV0,
) -> OmenaQueryCheckerSmtGateV0 {
    let registered_rules = list_omena_checker_rule_code_names()
        .into_iter()
        .collect::<BTreeSet<_>>();
    let enabled_rule_names = vec![OmenaCheckerRuleCodeV0::CascadeSMTViolation.as_str()];
    let evaluations = evaluate_omena_checker_smt_rules(input);
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

    OmenaQueryCheckerSmtGateV0 {
        schema_version: "0",
        product: "omena-query-checker-orchestrator.smt-gate",
        orchestrator_kind: "registered-rule-diagnostic-gate",
        backend_kind_name: active_omena_checker_smt_backend_kind_name_v0(),
        solver_backed: active_omena_checker_smt_solver_backed_v0(),
        product_scope: active_omena_checker_smt_product_scope_v0(),
        enabled_rule_names,
        emitted_rule_names,
        registered_rule_count: registered_rules.len(),
        unregistered_rule_count,
        evaluation_count: evaluations.len(),
        enforcement_passed: unregistered_rule_count == 0,
        evaluations,
        ready_surfaces: vec![
            "checkerRuleRegistry",
            "activeSmtBackendScope",
            "smtCascadeProofObligation",
            "registeredRuleDiagnosticGate",
            "queryDiagnosticHandoff",
        ],
    }
}

/// Opt-in z3 product lane for the non-propositional SMT cascade-ordering check.
///
/// The default product build remains solver-free and returns no layer-inversion
/// evaluations: the propositional stub cannot decide QF_LIA ordering searches
/// without over-warning. Builds that opt into `smt-z3` route this same gate
/// through the z3 backend, so `cascade.smt-violation` is emitted only when z3
/// finds an actual `@layer` flattening inversion.
pub fn run_omena_query_checker_smt_layer_inversion_gate_v0(
    input: OmenaCheckerSmtLayerInversionInputV0,
) -> OmenaQueryCheckerSmtGateV0 {
    let registered_rules = list_omena_checker_rule_code_names()
        .into_iter()
        .collect::<BTreeSet<_>>();
    let enabled_rule_names = vec![OmenaCheckerRuleCodeV0::CascadeSMTViolation.as_str()];
    let evaluations = evaluate_omena_checker_smt_layer_inversion_rules(input);
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

    OmenaQueryCheckerSmtGateV0 {
        schema_version: "0",
        product: "omena-query-checker-orchestrator.smt-layer-inversion-gate",
        orchestrator_kind: "registered-rule-diagnostic-gate",
        backend_kind_name: active_omena_checker_smt_backend_kind_name_v0(),
        solver_backed: active_omena_checker_smt_solver_backed_v0(),
        product_scope: active_omena_checker_smt_product_scope_v0(),
        enabled_rule_names,
        emitted_rule_names,
        registered_rule_count: registered_rules.len(),
        unregistered_rule_count,
        evaluation_count: evaluations.len(),
        enforcement_passed: unregistered_rule_count == 0,
        evaluations,
        ready_surfaces: vec![
            "checkerRuleRegistry",
            "activeSmtBackendScope",
            "smtZ3LayerInversionProductLane",
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
                cascade_declaration(CascadeDeclarationFixture {
                    declaration_id: "registered-gap",
                    selector: ".card",
                    property: "--gap-size",
                    value: "red",
                    source_order: 6,
                    layer_name: None,
                    layer_order: None,
                    important: false,
                    var_references: &[],
                }),
                cascade_declaration(CascadeDeclarationFixture {
                    declaration_id: "invalid-box-sizing",
                    selector: ".card",
                    property: "box-sizing",
                    value: "inline-box",
                    source_order: 7,
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
            custom_property_registrations: vec![OmenaCheckerCustomPropertyRegistrationInputV0 {
                name: "--gap-size".to_string(),
                syntax: Some("'<length>'".to_string()),
                inherits: Some("false".to_string()),
                initial_value: Some("8px".to_string()),
            }],
        });

        assert!(gate.enforcement_passed);
        assert_eq!(gate.unregistered_rule_count, 0);
        assert!(gate.emitted_rule_names.contains(&"unreachable-declaration"));
        assert!(gate.emitted_rule_names.contains(&"dead-cascade-layer"));
        assert!(gate.emitted_rule_names.contains(&"iacvt-prone"));
        assert!(gate.emitted_rule_names.contains(&"circular-var"));
        assert!(
            gate.emitted_rule_names
                .contains(&"registered-property-type-mismatch")
        );
        assert!(gate.emitted_rule_names.contains(&"invalid-property-value"));
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
            custom_property_registrations: Vec::new(),
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
            "registeredPropertyTypeMismatch",
            "unspecifiedCascadeTie",
            "designerIntentInconsistency",
            "cascadeSmtViolation",
            "rgFlowRelevantOperator",
            "categoricalCascadeEvidenceInconsistency",
            "replicaEnsembleInconsistency",
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
        assert_eq!(gate.mechanism_scope, RG_FLOW_MECHANISM_SCOPE_V0);
        assert_eq!(gate.product_surface, RG_FLOW_PRODUCT_SURFACE_V0);
        assert!(!gate.default_product_decision_mechanism);
        assert!(
            gate.ready_surfaces
                .contains(&"rgFlowOptInDeepAnalysisHintScope")
        );
        assert!(
            gate.emitted_rule_names
                .contains(&"rg-flow-relevant-operator")
        );
        assert!(gate.evaluations[0].spectral_radius > 1.0);
        assert_eq!(
            gate.evaluations[0].mechanism_scope,
            RG_FLOW_MECHANISM_SCOPE_V0
        );
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
    fn categorical_gate_emits_inconsistency_for_non_functorial_role_mapping() {
        // Only two distinct cascade primitives are exercised, so the functor can
        // build a single non-identity morphism and cannot witness composition.
        // The real apply_cascade_role_mapping_functor_v0 verdict rejects the
        // mapping and the gate surfaces the categorical inconsistency.
        let gate = run_omena_query_checker_categorical_gate_v0(OmenaCheckerCategoricalInputV0 {
            mappings: vec![OmenaCheckerCategoricalRoleMappingInputV0 {
                mapping_id: "stylesheet://narrow-cascade-evidence".to_string(),
                primitive_role_pairs: vec![
                    OmenaCheckerCategoricalPrimitiveRolePairInputV0 {
                        primitive_name: "cascade_property".to_string(),
                        categorical_role: "cosheaf colimit witness".to_string(),
                    },
                    OmenaCheckerCategoricalPrimitiveRolePairInputV0 {
                        primitive_name: "prove_layer_flatten_candidate".to_string(),
                        categorical_role: "Beck-Chevalley origin inversion witness".to_string(),
                    },
                ],
            }],
        });

        assert!(gate.enforcement_passed);
        assert_eq!(gate.unregistered_rule_count, 0);
        assert_eq!(gate.evaluation_count, 1);
        assert!(
            gate.emitted_rule_names
                .contains(&"categorical-cascade-evidence-inconsistency")
        );
        assert!(!gate.evaluations[0].functor_accepted);
        assert!(!gate.evaluations[0].composition_preserved);
        assert_eq!(
            gate.evaluations[0].mechanism_products,
            vec!["omena-categorical.cascade-primitive-role-functor"]
        );
    }

    #[test]
    fn categorical_gate_records_clear_suppression_for_composable_role_mapping() {
        // Three distinct cascade primitives give two composable non-identity
        // morphisms, so the functor witnesses composition, accepts the mapping,
        // and the gate surfaces nothing.
        let gate = run_omena_query_checker_categorical_gate_v0(OmenaCheckerCategoricalInputV0 {
            mappings: vec![OmenaCheckerCategoricalRoleMappingInputV0 {
                mapping_id: "stylesheet://broad-cascade-evidence".to_string(),
                primitive_role_pairs: vec![
                    OmenaCheckerCategoricalPrimitiveRolePairInputV0 {
                        primitive_name: "cascade_property".to_string(),
                        categorical_role: "cosheaf colimit witness".to_string(),
                    },
                    OmenaCheckerCategoricalPrimitiveRolePairInputV0 {
                        primitive_name: "prove_layer_flatten_candidate".to_string(),
                        categorical_role: "Beck-Chevalley origin inversion witness".to_string(),
                    },
                    OmenaCheckerCategoricalPrimitiveRolePairInputV0 {
                        primitive_name: "evaluate_static_supports_condition".to_string(),
                        categorical_role: "site-axis decidability witness".to_string(),
                    },
                ],
            }],
        });

        assert!(gate.enforcement_passed);
        assert_eq!(gate.evaluation_count, 0);
        assert!(gate.emitted_rule_names.is_empty());
    }

    #[test]
    fn replica_ensemble_gate_advertises_hint_scope_not_product_decision() {
        let gate =
            run_omena_query_checker_replica_ensemble_gate_v0(OmenaCheckerReplicaEnsembleInputV0 {
                reports: vec![OmenaCheckerReplicaEnsembleReportInputV0 {
                    workspace_root: "/workspace".to_string(),
                    recommendation: "investigateRsbBroken".to_string(),
                    mean_q: 0.5,
                    variance_q: 0.25,
                    top_disagreement_pair_count: 1,
                    mechanism_scope: REPLICA_ENSEMBLE_MECHANISM_SCOPE_V0.to_string(),
                    product_surface: REPLICA_ENSEMBLE_PRODUCT_SURFACE_V0.to_string(),
                    default_product_decision_mechanism:
                        REPLICA_ENSEMBLE_DEFAULT_PRODUCT_DECISION_MECHANISM_V0,
                }],
            });

        assert!(gate.enforcement_passed);
        assert_eq!(gate.evaluation_count, 1);
        assert_eq!(gate.mechanism_scope, REPLICA_ENSEMBLE_MECHANISM_SCOPE_V0);
        assert_eq!(gate.product_surface, REPLICA_ENSEMBLE_PRODUCT_SURFACE_V0);
        assert!(!gate.default_product_decision_mechanism);
        assert!(
            gate.ready_surfaces
                .contains(&"crossFileReplicaEnsembleHintScope")
        );
        assert_eq!(
            gate.evaluations[0].mechanism_scope,
            REPLICA_ENSEMBLE_MECHANISM_SCOPE_V0
        );
        assert_eq!(
            gate.evaluations[0].product_surface,
            REPLICA_ENSEMBLE_PRODUCT_SURFACE_V0
        );
        assert!(!gate.evaluations[0].default_product_decision_mechanism);
    }

    #[test]
    fn smt_gate_emits_violation_for_unsatisfiable_cascade_obligation() {
        // A box-shorthand combination obligation whose `no-important-longhand`
        // precondition is violated: the conjunction is `Unsat`, so the real
        // The active backend verdict drives the gate to surface
        // `cascade.smt-violation`. Default builds use the product-owned
        // propositional backend; opt-in `smt-z3` builds route this same product
        // gate through z3.
        let gate = run_omena_query_checker_smt_gate_v0(OmenaCheckerSmtInputV0 {
            obligations: vec![OmenaCheckerSmtObligationInputV0 {
                obligation_id: "stylesheet://.box::box-shorthand-combination".to_string(),
                l1_primitive: "boxShorthandCombination".to_string(),
                canonical_terms: vec![
                    "require:supported-shorthand-property=true".to_string(),
                    "require:canonical-longhand-quartet=true".to_string(),
                    "require:no-important-longhand=false".to_string(),
                    "require:no-empty-longhand-value=true".to_string(),
                    "require:adjacent-source-order=true".to_string(),
                ],
            }],
        });

        assert!(gate.enforcement_passed);
        assert_eq!(
            gate.backend_kind_name,
            if cfg!(feature = "smt-z3") {
                "z3"
            } else {
                "stub"
            }
        );
        assert_eq!(gate.solver_backed, cfg!(feature = "smt-z3"));
        assert_eq!(
            gate.product_scope,
            if cfg!(feature = "smt-z3") {
                "explicitOptInZ3SolverBackedProductGate"
            } else {
                "defaultSolverFreeStubProductGate"
            }
        );
        assert!(gate.ready_surfaces.contains(&"activeSmtBackendScope"));
        assert_eq!(gate.unregistered_rule_count, 0);
        assert_eq!(gate.evaluation_count, 1);
        assert!(gate.emitted_rule_names.contains(&"cascade.smt-violation"));
        assert_eq!(
            gate.evaluations[0].backend_kind_name,
            if cfg!(feature = "smt-z3") {
                "z3"
            } else {
                "stub"
            }
        );
        assert_eq!(gate.evaluations[0].sat_result_name, "unsat");
        assert!(
            gate.evaluations[0]
                .mechanism_products
                .contains(&"omena-smt.backend-check")
        );
        assert!(
            gate.evaluations[0]
                .mechanism_products
                .contains(&if cfg!(feature = "smt-z3") {
                    "omena-smt.backend.z3"
                } else {
                    "omena-smt.backend.stub"
                })
        );
    }

    #[test]
    fn smt_gate_records_clear_suppression_for_satisfiable_cascade_obligation() {
        // The same obligation shape with every precondition satisfied: the
        // conjunction is `Sat`, so the gate surfaces nothing.
        let gate = run_omena_query_checker_smt_gate_v0(OmenaCheckerSmtInputV0 {
            obligations: vec![OmenaCheckerSmtObligationInputV0 {
                obligation_id: "stylesheet://.box::box-shorthand-combination".to_string(),
                l1_primitive: "boxShorthandCombination".to_string(),
                canonical_terms: vec![
                    "require:supported-shorthand-property=true".to_string(),
                    "require:canonical-longhand-quartet=true".to_string(),
                    "require:no-important-longhand=true".to_string(),
                    "require:no-empty-longhand-value=true".to_string(),
                    "require:adjacent-source-order=true".to_string(),
                ],
            }],
        });

        assert!(gate.enforcement_passed);
        assert_eq!(
            gate.backend_kind_name,
            if cfg!(feature = "smt-z3") {
                "z3"
            } else {
                "stub"
            }
        );
        assert_eq!(gate.solver_backed, cfg!(feature = "smt-z3"));
        assert_eq!(
            gate.product_scope,
            if cfg!(feature = "smt-z3") {
                "explicitOptInZ3SolverBackedProductGate"
            } else {
                "defaultSolverFreeStubProductGate"
            }
        );
        assert!(gate.ready_surfaces.contains(&"activeSmtBackendScope"));
        assert_eq!(gate.evaluation_count, 0);
        assert!(gate.emitted_rule_names.is_empty());
    }

    #[test]
    fn smt_layer_inversion_gate_is_explicit_z3_opt_in_product_scope() {
        let gate = run_omena_query_checker_smt_layer_inversion_gate_v0(
            OmenaCheckerSmtLayerInversionInputV0 {
                obligations: vec![OmenaCheckerSmtLayerInversionObligationInputV0 {
                    obligation_id: "stylesheet://.box::color-layer-flatten-inversion".to_string(),
                    declarations: vec![
                        OmenaCheckerSmtLayerInversionDeclarationInputV0 {
                            declaration_id: "utilities-color".to_string(),
                            layer_rank: 1,
                            source_order: 0,
                        },
                        OmenaCheckerSmtLayerInversionDeclarationInputV0 {
                            declaration_id: "base-color".to_string(),
                            layer_rank: 0,
                            source_order: 1,
                        },
                    ],
                }],
            },
        );

        assert!(gate.enforcement_passed);
        assert_eq!(
            gate.product_scope,
            if cfg!(feature = "smt-z3") {
                "explicitOptInZ3SolverBackedProductGate"
            } else {
                "defaultSolverFreeStubProductGate"
            }
        );
        assert_eq!(gate.solver_backed, cfg!(feature = "smt-z3"));
        assert!(
            gate.ready_surfaces
                .contains(&"smtZ3LayerInversionProductLane")
        );
        if cfg!(feature = "smt-z3") {
            assert_eq!(gate.evaluation_count, 1);
            assert!(gate.emitted_rule_names.contains(&"cascade.smt-violation"));
            assert!(
                gate.evaluations[0]
                    .mechanism_products
                    .contains(&"omena-smt.layer-flatten-inversion")
            );
        } else {
            assert_eq!(gate.evaluation_count, 0);
            assert!(gate.emitted_rule_names.is_empty());
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
            selector: CanonicalSelector::from_canonical(fixture.selector),
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
