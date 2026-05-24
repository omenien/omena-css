use omena_cascade::{
    BoxLonghandInputV0, LayerFlattenInputV0, ScopeFlattenInputV0, StaticSupportsAssumptionV0,
    StaticSupportsEvalVerdictV0, evaluate_static_supports_condition,
    prove_box_shorthand_combination, prove_layer_flatten_candidate, prove_scope_flatten_candidate,
};

use crate::{CascadeSMTProofV0, SmtBackendV0, proof::cascade_smt_proof_v0};

pub fn smt_prove_box_shorthand_combination_v0<B: SmtBackendV0>(
    shorthand_property: &str,
    longhands: &[BoxLonghandInputV0],
    backend: &B,
) -> CascadeSMTProofV0 {
    let proof = prove_box_shorthand_combination(shorthand_property, longhands);
    cascade_smt_proof_v0(
        "box-shorthand-combination",
        backend,
        "prove_box_shorthand_combination",
        Some(proof.accepted),
    )
}

pub fn smt_prove_scope_flatten_candidate_v0<B: SmtBackendV0>(
    input: ScopeFlattenInputV0,
    backend: &B,
) -> CascadeSMTProofV0 {
    let proof = prove_scope_flatten_candidate(input);
    cascade_smt_proof_v0(
        "scope-flatten-candidate",
        backend,
        "prove_scope_flatten_candidate",
        Some(proof.accepted),
    )
}

pub fn smt_prove_layer_flatten_candidate_v0<B: SmtBackendV0>(
    input: LayerFlattenInputV0,
    backend: &B,
) -> CascadeSMTProofV0 {
    let proof = prove_layer_flatten_candidate(input);
    cascade_smt_proof_v0(
        "layer-flatten-candidate",
        backend,
        "prove_layer_flatten_candidate",
        Some(proof.accepted),
    )
}

pub fn smt_evaluate_static_supports_condition_v0<B: SmtBackendV0>(
    condition: &str,
    assumption: StaticSupportsAssumptionV0,
    backend: &B,
) -> CascadeSMTProofV0 {
    let witness = evaluate_static_supports_condition(condition, assumption);
    let l1_accepted = match witness.verdict {
        StaticSupportsEvalVerdictV0::AlwaysTrue => Some(true),
        StaticSupportsEvalVerdictV0::AlwaysFalse => Some(false),
        StaticSupportsEvalVerdictV0::Unknown => None,
    };
    cascade_smt_proof_v0(
        "static-supports-condition",
        backend,
        "evaluate_static_supports_condition",
        l1_accepted,
    )
}
