use ark_bls12_381::{Bls12_381, Fr};
use ark_ff::{Field, One};
use ark_groth16::{Groth16, prepare_verifying_key};
use ark_relations::gr1cs::{
    ConstraintSynthesizer, ConstraintSystemRef, LinearCombination, SynthesisError, Variable,
};
use ark_std::rand::{SeedableRng, rngs::StdRng};
use omena_cascade::BoxLonghandInputV0;
use omena_cascade_proof::{
    CanonicalSmtInputV0, StubSmtBackendV0, smt_prove_box_shorthand_combination_v0,
};
use omena_zk_circuit::{
    CascadeCircuitSpecV0, R1CSConstraintV0, R1CSLinearCombinationV0, R1CSWitnessAssignmentV0,
    cascade_circuit_spec_from_canonical_terms_v0, cascade_r1cs_constraints_from_canonical_terms_v0,
    cascade_r1cs_witness_from_canonical_terms_v0, check_r1cs_witness_satisfaction_v0,
    r1cs_wire_ids_v0,
};
use serde::Serialize;
use std::collections::BTreeMap;
use std::marker::PhantomData;

use crate::{
    SetupKindV0, ZK_AUDIT_DEFAULT_PROOF_BACKEND_ENABLED_V0, ZK_AUDIT_FEATURE_GATE_V0,
    ZK_AUDIT_LAYER_MARKER_V0, ZK_AUDIT_MECHANISM_SCOPE_V0, ZK_AUDIT_SCHEMA_VERSION_V0,
    active_zk_audit_proof_backend_scope_v0,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArkworksGroth16RoundTripV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub mechanism_scope: &'static str,
    pub default_proof_backend_enabled: bool,
    pub active_proof_backend_scope: &'static str,
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
    let constraints = cascade_r1cs_constraints_from_canonical_terms_v0(&payload.canonical_terms);
    let witness = cascade_r1cs_witness_from_canonical_terms_v0(&payload.canonical_terms);
    if constraints.is_empty() {
        return Err("canonical SMT payload has no R1CS-backed requirements".to_string());
    }

    let circuit = cascade_circuit_spec_from_canonical_terms_v0(
        format!("cascade-smt-{}", payload.obligation_id),
        &payload.canonical_terms,
    );

    // The pure circuit crate owns satisfiability now. Arkworks only adapts the
    // same R1CS constraints/witness into a Groth16 backend, so rejected
    // obligations fail through circuit semantics before proof generation.
    let satisfaction = check_r1cs_witness_satisfaction_v0(&constraints, &witness);
    if !satisfaction.satisfied {
        return Ok(ArkworksGroth16RoundTripV0 {
            schema_version: ZK_AUDIT_SCHEMA_VERSION_V0,
            product: "omena-zk-audit.arkworks-groth16-roundtrip",
            layer_marker: ZK_AUDIT_LAYER_MARKER_V0,
            feature_gate: ZK_AUDIT_FEATURE_GATE_V0,
            mechanism_scope: ZK_AUDIT_MECHANISM_SCOPE_V0,
            default_proof_backend_enabled: ZK_AUDIT_DEFAULT_PROOF_BACKEND_ENABLED_V0,
            active_proof_backend_scope: active_zk_audit_proof_backend_scope_v0(),
            setup_kind: SetupKindV0::ArkworksGroth16,
            backend: "arkworks-groth16",
            obligation_id: payload.obligation_id.clone(),
            circuit,
            requirement_count: constraints.len(),
            proof_generated: false,
            proof_verified: satisfaction.satisfied,
        });
    }

    let mut rng = StdRng::seed_from_u64(0x0c53_0008);
    let setup_circuit = R1CSConstraintSystemCircuit::<Fr>::setup_shape(constraints.clone());
    let proving_key =
        Groth16::<Bls12_381>::generate_random_parameters_with_reduction(setup_circuit, &mut rng)
            .map_err(|error| format!("arkworks setup failed: {error:?}"))?;
    let prepared_verifying_key = prepare_verifying_key::<Bls12_381>(&proving_key.vk);
    let proof = Groth16::<Bls12_381>::create_random_proof_with_reduction(
        R1CSConstraintSystemCircuit::<Fr>::with_witness(constraints.clone(), witness),
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
        mechanism_scope: ZK_AUDIT_MECHANISM_SCOPE_V0,
        default_proof_backend_enabled: ZK_AUDIT_DEFAULT_PROOF_BACKEND_ENABLED_V0,
        active_proof_backend_scope: active_zk_audit_proof_backend_scope_v0(),
        setup_kind: SetupKindV0::ArkworksGroth16,
        backend: "arkworks-groth16",
        obligation_id: payload.obligation_id.clone(),
        circuit,
        requirement_count: constraints.len(),
        proof_generated: true,
        proof_verified,
    })
}

/// Drive a real box-shorthand cascade obligation end to end into the arkworks
/// Groth16 round-trip.
///
/// The `require:...=true/false` terms are computed by the L1 cascade algorithm
/// `prove_box_shorthand_combination` (via the product cascade proof builder), so a
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

#[derive(Clone)]
struct R1CSConstraintSystemCircuit<F: Field> {
    constraints: Vec<R1CSConstraintV0>,
    witness: Option<R1CSWitnessAssignmentV0>,
    _field: PhantomData<F>,
}

impl<F: Field> R1CSConstraintSystemCircuit<F> {
    fn setup_shape(constraints: Vec<R1CSConstraintV0>) -> Self {
        Self {
            constraints,
            witness: None,
            _field: PhantomData,
        }
    }

    fn with_witness(constraints: Vec<R1CSConstraintV0>, witness: R1CSWitnessAssignmentV0) -> Self {
        Self {
            constraints,
            witness: Some(witness),
            _field: PhantomData,
        }
    }
}

impl<F: Field> ConstraintSynthesizer<F> for R1CSConstraintSystemCircuit<F> {
    fn generate_constraints(self, cs: ConstraintSystemRef<F>) -> Result<(), SynthesisError> {
        let mut variables = BTreeMap::new();
        variables.insert(
            "public.one".to_string(),
            cs.new_input_variable(|| Ok(F::one()))?,
        );
        let witness_values = self
            .witness
            .as_ref()
            .map(witness_value_map_v0)
            .unwrap_or_default();
        for wire_id in r1cs_wire_ids_v0(&self.constraints) {
            if wire_id == "public.one" {
                continue;
            }
            let witness_value = self
                .witness
                .as_ref()
                .map(|_| {
                    witness_values
                        .get(&wire_id)
                        .copied()
                        .ok_or(SynthesisError::AssignmentMissing)
                })
                .unwrap_or(Ok(1))?;
            let field_value = field_from_i64_v0::<F>(witness_value);
            let variable = cs.new_witness_variable(|| Ok(field_value))?;
            variables.insert(wire_id, variable);
        }

        for constraint in self.constraints {
            cs.enforce_r1cs_constraint(
                || ark_lc_from_r1cs_v0(&constraint.left, &variables),
                || ark_lc_from_r1cs_v0(&constraint.right, &variables),
                || ark_lc_from_r1cs_v0(&constraint.output, &variables),
            )?;
        }
        Ok(())
    }
}

fn witness_value_map_v0(witness: &R1CSWitnessAssignmentV0) -> BTreeMap<String, i64> {
    witness
        .values
        .iter()
        .map(|value| (value.wire_id.clone(), value.value))
        .collect()
}

fn ark_lc_from_r1cs_v0<F: Field>(
    combination: &R1CSLinearCombinationV0,
    variables: &BTreeMap<String, Variable>,
) -> LinearCombination<F> {
    let mut linear_combination = LinearCombination::<F>::new();
    if combination.constant != 0 {
        linear_combination += (field_from_i64_v0(combination.constant), Variable::one());
    }
    for term in &combination.terms {
        if let Some(variable) = variables.get(&term.wire_id) {
            linear_combination += (field_from_i64_v0(term.coefficient), *variable);
        }
    }
    linear_combination
}

fn field_from_i64_v0<F: Field>(value: i64) -> F {
    let mut field_value = F::zero();
    for _ in 0..value.unsigned_abs() {
        field_value += F::one();
    }
    if value < 0 { -field_value } else { field_value }
}
