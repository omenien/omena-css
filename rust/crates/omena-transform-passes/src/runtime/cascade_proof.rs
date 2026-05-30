use std::collections::BTreeSet;

use omena_cascade::{
    BoxLonghandInputV0, LayerFlattenInputV0, LayerFlattenProofV0, ScopeFlattenInputV0,
    ScopeFlattenProofV0, ShorthandCombinationProofV0, StaticSupportsEvalVerdictV0,
    StaticSupportsEvalWitnessV0,
};
use omena_parser::StyleDialect;
use omena_smt::{
    CanonicalSmtInputV0, SmtBackendSatResultV0, SmtBackendV0, StubSmtBackendV0,
    canonical_smt_input_v0, smt_evaluate_static_supports_condition_v0,
    smt_prove_box_shorthand_combination_v0, smt_prove_layer_flatten_candidate_v0,
    smt_prove_scope_flatten_candidate_v0,
};
use omena_transform_cst::TransformPassKind;
use serde::Serialize;
use serde_json::{Value, json};

use crate::{
    domains::{
        cascade_flatten::{
            collect_layer_flatten_proof_candidates_with_lexer,
            collect_scope_flatten_proof_candidates_with_lexer,
        },
        shorthand::collect_box_shorthand_proof_candidates_with_lexer,
        static_eval::collect_static_supports_proof_candidates_with_lexer,
    },
    model::{
        TransformCascadeProofObligationReportV0, TransformCascadeProofObligationV0,
        TransformExecutionContextV0,
    },
};

pub(crate) fn collect_cascade_proof_obligations_for_pass_input(
    pass_id: &'static str,
    pass: Option<TransformPassKind>,
    source: &str,
    dialect: StyleDialect,
    context: &TransformExecutionContextV0,
) -> Vec<TransformCascadeProofObligationV0> {
    match pass {
        Some(TransformPassKind::ShorthandCombining) => {
            collect_box_shorthand_proof_candidates_with_lexer(source, dialect)
                .into_iter()
                .map(|candidate| {
                    shorthand_obligation(
                        pass_id,
                        candidate.source_span_start,
                        candidate.source_span_end,
                        candidate.shorthand_property,
                        &candidate.longhands,
                        candidate.proof,
                    )
                })
                .collect()
        }
        Some(TransformPassKind::ScopeFlatten) => {
            collect_scope_flatten_proof_candidates_with_lexer(source, dialect)
                .into_iter()
                .map(|candidate| {
                    scope_obligation(
                        pass_id,
                        candidate.source_span_start,
                        candidate.source_span_end,
                        candidate.input,
                        candidate.proof,
                    )
                })
                .collect()
        }
        Some(TransformPassKind::LayerFlatten) if context.closed_style_world => {
            collect_layer_flatten_proof_candidates_with_lexer(source, dialect, true)
                .into_iter()
                .map(|candidate| {
                    layer_obligation(
                        pass_id,
                        candidate.source_span_start,
                        candidate.source_span_end,
                        candidate.input,
                        candidate.proof,
                    )
                })
                .collect()
        }
        Some(TransformPassKind::LayerFlatten) => {
            vec![TransformCascadeProofObligationV0 {
                pass_id,
                proof_product: "omena-cascade.layer-flatten-proof",
                accepted: false,
                blocked_reason: Some(
                    "requires an explicit closed-style-world bundle witness before mutation"
                        .to_string(),
                ),
                provenance_preserved: false,
                cascade_safe_witness: "layer rank cannot be erased without a closed bundle witness"
                    .to_string(),
                source_span_start: None,
                source_span_end: None,
                checked_obligations: vec!["closedBundleWitness"],
                canonical_smt_input: Some(canonical_smt_input_v0(
                    "layer-flatten-candidate",
                    "prove_layer_flatten_candidate",
                    vec![
                        "require:closed-bundle=false".to_string(),
                        "require:no-peer-layer=false".to_string(),
                        "require:no-unlayered-rule=false".to_string(),
                    ],
                )),
                proof_payload: json!({
                    "product": "omena-cascade.layer-flatten-proof",
                    "accepted": false,
                    "blockedReason": "requires an explicit closed-style-world bundle witness before mutation"
                }),
            }]
        }
        Some(
            TransformPassKind::SupportsStaticEval | TransformPassKind::DeadSupportsBranchRemoval,
        ) => collect_static_supports_proof_candidates_with_lexer(source, dialect)
            .into_iter()
            .map(|candidate| {
                supports_obligation(
                    pass_id,
                    candidate.source_span_start,
                    candidate.source_span_end,
                    candidate.witness,
                )
            })
            .collect(),
        _ => Vec::new(),
    }
}

pub(crate) fn summarize_cascade_proof_obligations(
    obligations: Vec<TransformCascadeProofObligationV0>,
) -> TransformCascadeProofObligationReportV0 {
    let accepted_count = obligations
        .iter()
        .filter(|obligation| obligation.accepted)
        .count();
    let checked_pass_ids = obligations
        .iter()
        .map(|obligation| obligation.pass_id)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let obligation_count = obligations.len();

    TransformCascadeProofObligationReportV0 {
        schema_version: "0",
        product: "omena-transform-passes.cascade-proof-obligations",
        obligation_count,
        accepted_count,
        blocked_count: obligation_count.saturating_sub(accepted_count),
        checked_pass_ids,
        obligations,
    }
}

/// Discharge the SMT obligation for a cascade-proof candidate and report whether
/// the solver accepted it.
///
/// The transform records the obligation's `accepted` outcome from the SMT
/// solver's `sat_result` (`Sat` => accepted, `Unsat`/`Unknown` => rejected),
/// not from the L1 proof flag. The canonical input the solver consumed is
/// returned so it can be carried on the obligation for audit.
fn discharge_smt_obligation(
    canonical_input: CanonicalSmtInputV0,
    backend: &StubSmtBackendV0,
) -> (bool, CanonicalSmtInputV0) {
    let sat_result = backend
        .check_canonical_input_v0(&canonical_input)
        .sat_result;
    let accepted = matches!(sat_result, SmtBackendSatResultV0::Sat);
    (accepted, canonical_input)
}

fn shorthand_obligation(
    pass_id: &'static str,
    source_span_start: usize,
    source_span_end: usize,
    shorthand_property: &str,
    longhands: &[BoxLonghandInputV0],
    proof: ShorthandCombinationProofV0,
) -> TransformCascadeProofObligationV0 {
    let smt_proof = smt_prove_box_shorthand_combination_v0(
        shorthand_property,
        longhands,
        &StubSmtBackendV0::default(),
    );
    let (accepted, canonical_smt_input) =
        discharge_smt_obligation(smt_proof.canonical_input, &StubSmtBackendV0::default());
    let provenance_preserved = accepted && proof.provenance_preserved;
    let blocked_reason = smt_blocked_reason(accepted, proof.blocked_reason.map(str::to_string));
    let cascade_safe_witness = proof.cascade_safe_witness.clone();

    proof_obligation(
        pass_id,
        "omena-cascade.shorthand-combination-proof",
        accepted,
        blocked_reason,
        provenance_preserved,
        cascade_safe_witness,
        Some(source_span_start),
        Some(source_span_end),
        vec![
            "canonicalLonghandSet",
            "adjacentSourceOrder",
            "nonImportantDeclarations",
            "provenancePreservation",
        ],
        Some(canonical_smt_input),
        proof,
    )
}

fn scope_obligation(
    pass_id: &'static str,
    source_span_start: usize,
    source_span_end: usize,
    input: ScopeFlattenInputV0,
    proof: ScopeFlattenProofV0,
) -> TransformCascadeProofObligationV0 {
    let smt_proof = smt_prove_scope_flatten_candidate_v0(input, &StubSmtBackendV0::default());
    let (accepted, canonical_smt_input) =
        discharge_smt_obligation(smt_proof.canonical_input, &StubSmtBackendV0::default());
    let provenance_preserved = accepted && proof.provenance_preserved;
    let blocked_reason = smt_blocked_reason(accepted, proof.blocked_reason.map(str::to_string));
    let cascade_safe_witness = proof.cascade_safe_witness.clone();

    proof_obligation(
        pass_id,
        "omena-cascade.scope-flatten-proof",
        accepted,
        blocked_reason,
        provenance_preserved,
        cascade_safe_witness,
        Some(source_span_start),
        Some(source_span_end),
        vec![
            "rootScopeOnly",
            "noLimitSelector",
            "noPeerScopes",
            "noUnscopedCompetition",
            "noLayerComposition",
        ],
        Some(canonical_smt_input),
        proof,
    )
}

fn layer_obligation(
    pass_id: &'static str,
    source_span_start: usize,
    source_span_end: usize,
    input: LayerFlattenInputV0,
    proof: LayerFlattenProofV0,
) -> TransformCascadeProofObligationV0 {
    let smt_proof = smt_prove_layer_flatten_candidate_v0(input, &StubSmtBackendV0::default());
    let (accepted, canonical_smt_input) =
        discharge_smt_obligation(smt_proof.canonical_input, &StubSmtBackendV0::default());
    let provenance_preserved = accepted && proof.provenance_preserved;
    let blocked_reason = smt_blocked_reason(accepted, proof.blocked_reason.map(str::to_string));
    let cascade_safe_witness = proof.cascade_safe_witness.clone();

    proof_obligation(
        pass_id,
        "omena-cascade.layer-flatten-proof",
        accepted,
        blocked_reason,
        provenance_preserved,
        cascade_safe_witness,
        Some(source_span_start),
        Some(source_span_end),
        vec![
            "closedBundleWitness",
            "singleLayerContext",
            "noUnlayeredCompetition",
            "noImportantLayerInversion",
        ],
        Some(canonical_smt_input),
        proof,
    )
}

fn supports_obligation(
    pass_id: &'static str,
    source_span_start: usize,
    source_span_end: usize,
    witness: StaticSupportsEvalWitnessV0,
) -> TransformCascadeProofObligationV0 {
    // The supports transform's `accepted` means "the condition is statically
    // *decided*" (true or false), not "satisfiable", so it is not driven by the
    // Sat/Unsat solver result the way the shorthand/scope/layer obligations are.
    // The canonical input is still produced by the real SMT proof so it is
    // carried for audit, but a known-false condition (`Unsat`) is still a
    // decided, accepted dead-branch removal.
    let smt_proof = smt_evaluate_static_supports_condition_v0(
        witness.condition.as_str(),
        witness.assumption,
        &StubSmtBackendV0::default(),
    );
    let accepted = witness.verdict != StaticSupportsEvalVerdictV0::Unknown;
    let blocked_reason = (!accepted).then(|| witness.reason.to_string());
    let provenance_preserved = witness.provenance_preserved;
    let cascade_safe_witness = witness.reason.to_string();
    let canonical_smt_input = smt_proof.canonical_input;

    proof_obligation(
        pass_id,
        "omena-cascade.supports-static-eval",
        accepted,
        blocked_reason,
        provenance_preserved,
        cascade_safe_witness,
        Some(source_span_start),
        Some(source_span_end),
        vec![
            "staticSupportsCondition",
            "modernBrowserAssumption",
            "knownFeatureQueryShape",
        ],
        Some(canonical_smt_input),
        witness,
    )
}

#[allow(clippy::too_many_arguments)]
fn proof_obligation<T: Serialize>(
    pass_id: &'static str,
    proof_product: &'static str,
    accepted: bool,
    blocked_reason: Option<String>,
    provenance_preserved: bool,
    cascade_safe_witness: String,
    source_span_start: Option<usize>,
    source_span_end: Option<usize>,
    checked_obligations: Vec<&'static str>,
    canonical_smt_input: Option<CanonicalSmtInputV0>,
    proof: T,
) -> TransformCascadeProofObligationV0 {
    TransformCascadeProofObligationV0 {
        pass_id,
        proof_product,
        accepted,
        blocked_reason,
        provenance_preserved,
        cascade_safe_witness,
        source_span_start,
        source_span_end,
        checked_obligations,
        canonical_smt_input,
        proof_payload: serde_json::to_value(proof).unwrap_or(Value::Null),
    }
}

/// Pick the blocked reason recorded on an obligation once the SMT solver has
/// produced the `accepted` verdict.
///
/// When the solver rejects the obligation (`accepted == false`) and the L1
/// proof had not already recorded a reason, the rejection is attributed to the
/// solver so the diagnostic never claims acceptance the solver did not grant.
fn smt_blocked_reason(accepted: bool, l1_reason: Option<String>) -> Option<String> {
    if accepted {
        None
    } else {
        Some(l1_reason.unwrap_or_else(|| {
            "smt solver rejected the cascade-safety obligation (unsat)".to_string()
        }))
    }
}
