use std::collections::BTreeSet;

use omena_cascade::{
    LayerFlattenProofV0, ScopeFlattenProofV0, ShorthandCombinationProofV0,
    StaticSupportsEvalVerdictV0, StaticSupportsEvalWitnessV0,
};
use omena_parser::StyleDialect;
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

fn shorthand_obligation(
    pass_id: &'static str,
    source_span_start: usize,
    source_span_end: usize,
    proof: ShorthandCombinationProofV0,
) -> TransformCascadeProofObligationV0 {
    let accepted = proof.accepted;
    let blocked_reason = proof.blocked_reason.map(str::to_string);
    let provenance_preserved = proof.provenance_preserved;
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
        proof,
    )
}

fn scope_obligation(
    pass_id: &'static str,
    source_span_start: usize,
    source_span_end: usize,
    proof: ScopeFlattenProofV0,
) -> TransformCascadeProofObligationV0 {
    let accepted = proof.accepted;
    let blocked_reason = proof.blocked_reason.map(str::to_string);
    let provenance_preserved = proof.provenance_preserved;
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
        proof,
    )
}

fn layer_obligation(
    pass_id: &'static str,
    source_span_start: usize,
    source_span_end: usize,
    proof: LayerFlattenProofV0,
) -> TransformCascadeProofObligationV0 {
    let accepted = proof.accepted;
    let blocked_reason = proof.blocked_reason.map(str::to_string);
    let provenance_preserved = proof.provenance_preserved;
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
        proof,
    )
}

fn supports_obligation(
    pass_id: &'static str,
    source_span_start: usize,
    source_span_end: usize,
    witness: StaticSupportsEvalWitnessV0,
) -> TransformCascadeProofObligationV0 {
    let accepted = witness.verdict != StaticSupportsEvalVerdictV0::Unknown;
    let blocked_reason = (!accepted).then(|| witness.reason.to_string());
    let provenance_preserved = witness.provenance_preserved;
    let cascade_safe_witness = witness.reason.to_string();

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
        proof_payload: serde_json::to_value(proof).unwrap_or(Value::Null),
    }
}
