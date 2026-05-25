use crate::{ModalCheckWitnessSourceV0, ModalCheckWitnessV0, StaticSupportsEvalVerdictV0};

pub fn summarize_modal_check_witness_v0(
    witnesses: Vec<ModalCheckWitnessSourceV0>,
) -> ModalCheckWitnessV0 {
    let accepted_count = witnesses
        .iter()
        .filter(|witness| modal_witness_accepted(witness))
        .count();
    let source_products = witnesses
        .iter()
        .map(modal_witness_product)
        .collect::<Vec<_>>();
    let all_provenance_preserved = witnesses.iter().all(modal_witness_provenance_preserved);
    let obligation_count = witnesses.len();

    ModalCheckWitnessV0 {
        schema_version: "0",
        product: "omena-cascade.modal-check-witness",
        modal_family: "cascadeProofObligationStrictSuperset",
        substrate: "omena-cascade.proof-witnesses",
        obligation_count,
        accepted_count,
        blocked_count: obligation_count - accepted_count,
        all_provenance_preserved,
        source_products,
        witnesses,
    }
}

fn modal_witness_accepted(witness: &ModalCheckWitnessSourceV0) -> bool {
    match witness {
        ModalCheckWitnessSourceV0::ShorthandCombination(proof) => proof.accepted,
        ModalCheckWitnessSourceV0::StaticSupportsEval(witness) => {
            witness.verdict != StaticSupportsEvalVerdictV0::Unknown
        }
        ModalCheckWitnessSourceV0::ScopeFlatten(proof) => proof.accepted,
        ModalCheckWitnessSourceV0::LayerFlatten(proof) => proof.accepted,
    }
}

fn modal_witness_provenance_preserved(witness: &ModalCheckWitnessSourceV0) -> bool {
    match witness {
        ModalCheckWitnessSourceV0::ShorthandCombination(proof) => proof.provenance_preserved,
        ModalCheckWitnessSourceV0::StaticSupportsEval(witness) => witness.provenance_preserved,
        ModalCheckWitnessSourceV0::ScopeFlatten(proof) => proof.provenance_preserved,
        ModalCheckWitnessSourceV0::LayerFlatten(proof) => proof.provenance_preserved,
    }
}

fn modal_witness_product(witness: &ModalCheckWitnessSourceV0) -> &'static str {
    match witness {
        ModalCheckWitnessSourceV0::ShorthandCombination(proof) => proof.product,
        ModalCheckWitnessSourceV0::StaticSupportsEval(witness) => witness.product,
        ModalCheckWitnessSourceV0::ScopeFlatten(proof) => proof.product,
        ModalCheckWitnessSourceV0::LayerFlatten(proof) => proof.product,
    }
}
