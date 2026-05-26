use ark_bls12_381::{Bls12_381, Fr};
use ark_ff::{Field, One};
use ark_groth16::{Groth16, prepare_verifying_key};
use ark_relations::gr1cs::{ConstraintSynthesizer, ConstraintSystemRef, SynthesisError, lc};
use ark_std::rand::{SeedableRng, rngs::StdRng};
use omena_smt::CanonicalSmtInputV0;
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
