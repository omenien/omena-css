//! Theory-observation harness contracts for semantic milestones.
//!
//! The observation surface records whether selector identity, source evidence,
//! semantic graph readiness, and coupling boundaries are present before a gate
//! declares a semantic milestone ready.

use std::collections::BTreeMap;

use serde::Serialize;

use crate::StyleSemanticGraphSummaryV0;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TheoryObservationHarnessSummaryV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub graph_product: &'static str,
    pub selector_identity: SelectorIdentityObservationV0,
    pub source_evidence: SourceEvidenceObservationV0,
    pub downstream_readiness: SemanticGraphDownstreamReadinessV0,
    pub coupling_boundary: SemanticCouplingBoundaryObservationV0,
    pub blocking_gaps: Vec<&'static str>,
    pub next_priorities: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TheoryObservationContractV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub observation_product: &'static str,
    pub ready: bool,
    pub publish_ready: bool,
    pub selector_identity_status: &'static str,
    pub source_evidence_status: &'static str,
    pub downstream_readiness_status: &'static str,
    pub generic_observation_count: usize,
    pub cme_coupled_observation_count: usize,
    pub blocking_gaps: Vec<&'static str>,
    pub publish_blocking_gaps: Vec<&'static str>,
    pub observation_gaps: Vec<&'static str>,
    pub next_priorities: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SelectorIdentityObservationV0 {
    pub status: &'static str,
    pub observed_selector_count: usize,
    pub rename_safe_selector_count: usize,
    pub rewrite_blocked_selector_count: usize,
    pub precise_rename_span_ready: bool,
    pub rename_safe: bool,
    pub blockers: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceEvidenceObservationV0 {
    pub status: &'static str,
    pub reference_site_count: usize,
    pub editable_direct_site_count: usize,
    pub expression_count: usize,
    pub explainable_certainty_reason_count: usize,
    pub missing_certainty_reason_count: usize,
    pub certainty_reason_counts: BTreeMap<String, usize>,
    pub cme_coupled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SemanticGraphDownstreamReadinessV0 {
    pub status: &'static str,
    pub semantic_graph_ready: bool,
    pub downstream_check_ready: bool,
    pub precise_rename_ready: bool,
    pub formatter_ready: bool,
    pub recovery_diagnostics_observed: bool,
    pub blocking_gap_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SemanticCouplingBoundaryObservationV0 {
    pub status: &'static str,
    pub generic_observation_count: usize,
    pub cme_coupled_observation_count: usize,
    pub generic_surfaces: Vec<&'static str>,
    pub cme_coupled_surfaces: Vec<&'static str>,
    pub split_recommendation: &'static str,
}

pub trait TheoryObservationHarnessInput {
    fn summarize_theory_observation_harness(&self) -> TheoryObservationHarnessSummaryV0;

    fn summarize_theory_observation_contract(&self) -> TheoryObservationContractV0 {
        summarize_theory_observation_contract(self)
    }
}

impl TheoryObservationHarnessInput for StyleSemanticGraphSummaryV0 {
    fn summarize_theory_observation_harness(&self) -> TheoryObservationHarnessSummaryV0 {
        summarize_style_semantic_graph_observation(self)
    }
}

pub fn summarize_theory_observation_harness<T>(input: &T) -> TheoryObservationHarnessSummaryV0
where
    T: TheoryObservationHarnessInput + ?Sized,
{
    input.summarize_theory_observation_harness()
}

pub fn summarize_theory_observation_contract<T>(input: &T) -> TheoryObservationContractV0
where
    T: TheoryObservationHarnessInput + ?Sized,
{
    summarize_theory_observation_contract_from_summary(
        &input.summarize_theory_observation_harness(),
    )
}

fn summarize_style_semantic_graph_observation(
    graph: &StyleSemanticGraphSummaryV0,
) -> TheoryObservationHarnessSummaryV0 {
    let selector_identity = observe_selector_identity(graph);
    let source_evidence = observe_source_evidence(graph);
    let downstream_readiness = observe_downstream_readiness(graph, &selector_identity);
    let coupling_boundary = observe_coupling_boundary(graph);
    let mut blocking_gaps = Vec::new();

    if selector_identity.status != "ready" {
        blocking_gaps.push("selectorRewriteSafety");
    }
    if source_evidence.status != "ready" {
        blocking_gaps.push("sourceEvidence");
    }
    if downstream_readiness.status != "ready" {
        blocking_gaps.push("downstreamReadiness");
    }

    let next_priorities = if blocking_gaps.is_empty() {
        vec!["externalCorpus", "traitDogfooding"]
    } else {
        blocking_gaps.clone()
    };

    TheoryObservationHarnessSummaryV0 {
        schema_version: "0",
        product: "omena-semantic.theory-observation-harness",
        graph_product: graph.product,
        selector_identity,
        source_evidence,
        downstream_readiness,
        coupling_boundary,
        blocking_gaps,
        next_priorities,
    }
}

fn summarize_theory_observation_contract_from_summary(
    observation: &TheoryObservationHarnessSummaryV0,
) -> TheoryObservationContractV0 {
    let publish_blocking_gaps = publish_blocking_gaps_for_observation(observation);
    let observation_gaps = observation
        .blocking_gaps
        .iter()
        .copied()
        .filter(|gap| !publish_blocking_gaps.contains(gap))
        .collect::<Vec<_>>();

    TheoryObservationContractV0 {
        schema_version: "0",
        product: "omena-semantic.theory-observation-contract",
        observation_product: observation.product,
        ready: observation.blocking_gaps.is_empty(),
        publish_ready: publish_blocking_gaps.is_empty(),
        selector_identity_status: observation.selector_identity.status,
        source_evidence_status: observation.source_evidence.status,
        downstream_readiness_status: observation.downstream_readiness.status,
        generic_observation_count: observation.coupling_boundary.generic_observation_count,
        cme_coupled_observation_count: observation.coupling_boundary.cme_coupled_observation_count,
        blocking_gaps: observation.blocking_gaps.clone(),
        publish_blocking_gaps,
        observation_gaps,
        next_priorities: observation.next_priorities.clone(),
    }
}

fn publish_blocking_gaps_for_observation(
    observation: &TheoryObservationHarnessSummaryV0,
) -> Vec<&'static str> {
    let mut gaps = Vec::new();

    if observation.selector_identity.status != "ready" {
        gaps.push("selectorRewriteSafety");
    }
    if observation.coupling_boundary.generic_observation_count == 0 {
        gaps.push("genericObservationBoundary");
    }

    gaps
}

fn observe_selector_identity(graph: &StyleSemanticGraphSummaryV0) -> SelectorIdentityObservationV0 {
    let observed_selector_count = graph.selector_identity_engine.canonical_id_count;
    let rewrite_blocked_selector_count = graph
        .selector_identity_engine
        .rewrite_safety
        .blocked_canonical_ids
        .len();
    let rename_safe_selector_count = graph
        .selector_identity_engine
        .rewrite_safety
        .safe_canonical_ids
        .len();
    let precise_rename_span_ready = graph
        .lossless_cst_contract
        .consumer_readiness
        .precise_rename_base_ready;
    let rename_safe = observed_selector_count > 0
        && rewrite_blocked_selector_count == 0
        && precise_rename_span_ready;

    SelectorIdentityObservationV0 {
        status: if observed_selector_count == 0 {
            "gap"
        } else if rename_safe {
            "ready"
        } else {
            "partial"
        },
        observed_selector_count,
        rename_safe_selector_count,
        rewrite_blocked_selector_count,
        precise_rename_span_ready,
        rename_safe,
        blockers: graph
            .selector_identity_engine
            .rewrite_safety
            .blockers
            .clone(),
    }
}

fn observe_source_evidence(graph: &StyleSemanticGraphSummaryV0) -> SourceEvidenceObservationV0 {
    let evidence = &graph.source_input_evidence;
    let expression_count = evidence.certainty_reason.expression_count;
    let missing_certainty_reason_count = evidence.certainty_reason.missing_reason_count;
    let explainable_certainty_reason_count =
        expression_count.saturating_sub(missing_certainty_reason_count);
    let source_observed =
        evidence.reference_site_identity.reference_site_count > 0 || expression_count > 0;
    let source_ready = evidence.reference_site_identity.status == "ready"
        && evidence.certainty_reason.status == "ready"
        && evidence.binding_origin.status == "ready"
        && evidence.style_module_edge.status == "ready"
        && evidence.value_domain_explanation.status == "ready";

    SourceEvidenceObservationV0 {
        status: if source_ready {
            "ready"
        } else if source_observed {
            "partial"
        } else {
            "gap"
        },
        reference_site_count: evidence.reference_site_identity.reference_site_count,
        editable_direct_site_count: evidence.reference_site_identity.editable_direct_site_count,
        expression_count,
        explainable_certainty_reason_count,
        missing_certainty_reason_count,
        certainty_reason_counts: evidence.certainty_reason.reason_counts.clone(),
        cme_coupled: true,
    }
}

fn observe_downstream_readiness(
    graph: &StyleSemanticGraphSummaryV0,
    selector_identity: &SelectorIdentityObservationV0,
) -> SemanticGraphDownstreamReadinessV0 {
    let semantic_graph_ready = graph.product == "omena-semantic.style-semantic-graph"
        && graph.promotion_evidence.blocking_gaps.is_empty()
        && graph.selector_identity_engine.canonical_id_count > 0
        && graph
            .lossless_cst_contract
            .span_invariants
            .byte_span_contract_ready;
    let downstream_check_ready = semantic_graph_ready && selector_identity.status != "gap";
    let precise_rename_ready = downstream_check_ready && selector_identity.rename_safe;
    let formatter_ready = graph
        .lossless_cst_contract
        .consumer_readiness
        .formatter_base_ready;

    SemanticGraphDownstreamReadinessV0 {
        status: if downstream_check_ready && precise_rename_ready {
            "ready"
        } else if semantic_graph_ready {
            "partial"
        } else {
            "gap"
        },
        semantic_graph_ready,
        downstream_check_ready,
        precise_rename_ready,
        formatter_ready,
        recovery_diagnostics_observed: graph
            .lossless_cst_contract
            .consumer_readiness
            .recovery_diagnostics_observed,
        blocking_gap_count: graph.promotion_evidence.blocking_gaps.len(),
    }
}

fn observe_coupling_boundary(
    graph: &StyleSemanticGraphSummaryV0,
) -> SemanticCouplingBoundaryObservationV0 {
    let generic_surfaces = vec![
        "parserSemanticFacts",
        "designTokenSemantics",
        "selectorIdentity",
        "losslessCstContract",
    ];
    let cme_coupled_surfaces = vec!["sourceInputEvidence", "promotionEvidenceWithSourceInput"];
    let cme_coupled_observation_count = if graph.source_input_evidence.input_version.is_empty() {
        0
    } else {
        cme_coupled_surfaces.len()
    };
    let split_recommendation = if cme_coupled_observation_count == 0 {
        "keep-integrated-source-gap"
    } else {
        "keep-integrated-observe-boundary"
    };

    SemanticCouplingBoundaryObservationV0 {
        status: "ready",
        generic_observation_count: generic_surfaces.len(),
        cme_coupled_observation_count,
        generic_surfaces,
        cme_coupled_surfaces,
        split_recommendation,
    }
}
