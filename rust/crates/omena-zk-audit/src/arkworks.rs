use ark_bls12_381::{Bls12_381, Fr};
use ark_ff::{Field, One};
use ark_groth16::{Groth16, prepare_verifying_key};
use ark_relations::gr1cs::{
    ConstraintSynthesizer, ConstraintSystem, ConstraintSystemRef, SynthesisError, lc,
};
use ark_std::rand::{SeedableRng, rngs::StdRng};
use omena_cascade::BoxLonghandInputV0;
use omena_smt::{CanonicalSmtInputV0, StubSmtBackendV0, smt_prove_box_shorthand_combination_v0};
use omena_zk_circuit::{CascadeCircuitSpecV0, cascade_circuit_spec_from_canonical_terms_v0};
use serde::Serialize;
use std::marker::PhantomData;

use crate::{
    SetupKindV0, ZK_AUDIT_FEATURE_GATE_V0, ZK_AUDIT_LAYER_MARKER_V0, ZK_AUDIT_SCHEMA_VERSION_V0,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArkworksGroth16RoundTripV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub setup_kind: SetupKindV0,
    pub backend: &'static str,
    pub obligation_id: String,
    pub circuit: CascadeCircuitSpecV0,
    pub requirement_count: usize,
    pub proof_generated: bool,
    pub proof_verified: bool,
}

pub fn prove_and_verify_cascade_smt_payload_with_arkworks_v0(
    payload: &CanonicalSmtInputV0,
) -> Result<ArkworksGroth16RoundTripV0, String> {
    let requirement_values = canonical_requirement_values_v0(payload);
    if requirement_values.is_empty() {
        return Err("canonical SMT payload has no R1CS-backed requirements".to_string());
    }

    let circuit = cascade_circuit_spec_from_canonical_terms_v0(
        format!("cascade-smt-{}", payload.obligation_id),
        &payload.canonical_terms,
    );

    // The Groth16 prover asserts `cs.is_satisfied()` internally, so an
    // unsatisfiable cascade obligation (a `require:...=false` term) would panic
    // inside arkworks. Pre-flight the witness assignment so an unsatisfiable
    // obligation is reported as a rejected (unverified) proof instead.
    let obligation_satisfiable = cascade_obligation_satisfiable_v0(requirement_values.clone())
        .map_err(|error| format!("arkworks witness satisfiability check failed: {error:?}"))?;
    if !obligation_satisfiable {
        return Ok(ArkworksGroth16RoundTripV0 {
            schema_version: ZK_AUDIT_SCHEMA_VERSION_V0,
            product: "omena-zk-audit.arkworks-groth16-roundtrip",
            layer_marker: ZK_AUDIT_LAYER_MARKER_V0,
            feature_gate: ZK_AUDIT_FEATURE_GATE_V0,
            setup_kind: SetupKindV0::ArkworksGroth16,
            backend: "arkworks-groth16",
            obligation_id: payload.obligation_id.clone(),
            circuit,
            requirement_count: requirement_values.len(),
            proof_generated: false,
            proof_verified: false,
        });
    }

    let mut rng = StdRng::seed_from_u64(0x0c53_0008);
    let setup_circuit = RequirementSatisfactionCircuit::<Fr>::setup_shape(requirement_values.len());
    let proving_key =
        Groth16::<Bls12_381>::generate_random_parameters_with_reduction(setup_circuit, &mut rng)
            .map_err(|error| format!("arkworks setup failed: {error:?}"))?;
    let prepared_verifying_key = prepare_verifying_key::<Bls12_381>(&proving_key.vk);
    let proof = Groth16::<Bls12_381>::create_random_proof_with_reduction(
        RequirementSatisfactionCircuit::<Fr>::with_values(requirement_values.clone()),
        &proving_key,
        &mut rng,
    )
    .map_err(|error| format!("arkworks proof generation failed: {error:?}"))?;
    let proof_verified =
        Groth16::<Bls12_381>::verify_proof(&prepared_verifying_key, &proof, &[Fr::one()])
            .map_err(|error| format!("arkworks proof verification failed: {error:?}"))?;

    Ok(ArkworksGroth16RoundTripV0 {
        schema_version: ZK_AUDIT_SCHEMA_VERSION_V0,
        product: "omena-zk-audit.arkworks-groth16-roundtrip",
        layer_marker: ZK_AUDIT_LAYER_MARKER_V0,
        feature_gate: ZK_AUDIT_FEATURE_GATE_V0,
        setup_kind: SetupKindV0::ArkworksGroth16,
        backend: "arkworks-groth16",
        obligation_id: payload.obligation_id.clone(),
        circuit,
        requirement_count: requirement_values.len(),
        proof_generated: true,
        proof_verified,
    })
}

/// Drive a real box-shorthand cascade obligation end to end into the arkworks
/// Groth16 round-trip.
///
/// The `require:...=true/false` terms are computed by the L1 cascade algorithm
/// `prove_box_shorthand_combination` (via omena-smt's obligation builder), so a
/// canonical longhand quartet yields a satisfiable obligation that produces a
/// verified proof, while a reordered/important/non-canonical quartet yields an
/// unsatisfiable obligation whose proof is rejected. The discriminating field is
/// never fed as a literal.
pub fn prove_and_verify_box_shorthand_cascade_with_arkworks_v0(
    shorthand_property: &str,
    longhands: &[BoxLonghandInputV0],
) -> Result<ArkworksGroth16RoundTripV0, String> {
    let backend = StubSmtBackendV0::default();
    let proof = smt_prove_box_shorthand_combination_v0(shorthand_property, longhands, &backend);
    prove_and_verify_cascade_smt_payload_with_arkworks_v0(&proof.canonical_input)
}

/// CLI-facing helper: build the canonical `margin` longhand quartet (or a
/// non-canonical, reordered variant) and run the arkworks Groth16 round-trip.
///
/// `reorder = false` produces the canonical quartet, so the cascade algorithm
/// derives a satisfiable obligation and the proof verifies. `reorder = true`
/// swaps two longhands, so the cascade algorithm derives
/// `require:canonical-longhand-quartet=false`, the obligation is unsatisfiable,
/// and no verified proof is produced. The CLI passes only the boolean; the
/// discriminating term is computed by the cascade algorithm, never hardcoded.
pub fn prove_and_verify_canonical_margin_cascade_with_arkworks_v0(
    reorder: bool,
) -> Result<ArkworksGroth16RoundTripV0, String> {
    let order: [&str; 4] = if reorder {
        ["margin-top", "margin-bottom", "margin-right", "margin-left"]
    } else {
        ["margin-top", "margin-right", "margin-bottom", "margin-left"]
    };
    let longhands: Vec<BoxLonghandInputV0> = order
        .iter()
        .enumerate()
        .map(|(index, property)| BoxLonghandInputV0 {
            property: (*property).to_string(),
            value: "1px".to_string(),
            important: false,
            source_order: (index as u32) + 1,
        })
        .collect();
    prove_and_verify_box_shorthand_cascade_with_arkworks_v0("margin", &longhands)
}

fn cascade_obligation_satisfiable_v0(
    requirement_values: Vec<bool>,
) -> Result<bool, SynthesisError> {
    let cs = ConstraintSystem::<Fr>::new_ref();
    RequirementSatisfactionCircuit::<Fr>::with_values(requirement_values)
        .generate_constraints(cs.clone())?;
    cs.finalize();
    cs.is_satisfied()
}

#[derive(Clone)]
struct RequirementSatisfactionCircuit<F: Field> {
    requirement_values: Vec<Option<bool>>,
    _field: PhantomData<F>,
}

impl<F: Field> RequirementSatisfactionCircuit<F> {
    fn setup_shape(requirement_count: usize) -> Self {
        Self {
            requirement_values: vec![None; requirement_count],
            _field: PhantomData,
        }
    }

    fn with_values(requirement_values: Vec<bool>) -> Self {
        Self {
            requirement_values: requirement_values.into_iter().map(Some).collect(),
            _field: PhantomData,
        }
    }
}

impl<F: Field> ConstraintSynthesizer<F> for RequirementSatisfactionCircuit<F> {
    fn generate_constraints(self, cs: ConstraintSystemRef<F>) -> Result<(), SynthesisError> {
        let public_one = cs.new_input_variable(|| Ok(F::one()))?;
        for requirement_value in self.requirement_values {
            let witness_value = match requirement_value {
                Some(true) | None => F::one(),
                Some(false) => F::zero(),
            };
            let witness = cs.new_witness_variable(|| Ok(witness_value))?;
            cs.enforce_r1cs_constraint(
                || lc!() + witness,
                || lc!() + public_one,
                || lc!() + public_one,
            )?;
        }
        Ok(())
    }
}

fn canonical_requirement_values_v0(payload: &CanonicalSmtInputV0) -> Vec<bool> {
    payload
        .canonical_terms
        .iter()
        .filter_map(|term| {
            let (_name, value) = term.strip_prefix("require:")?.rsplit_once('=')?;
            match value {
                "true" => Some(true),
                "false" => Some(false),
                _ => None,
            }
        })
        .collect()
}
