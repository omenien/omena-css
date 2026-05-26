//! Pure circuit-side contracts for zero-knowledge cascade audit.
//!
//! claim_level: constraint-generation substrate, not a standalone proving
//! backend.

use serde::Serialize;

pub const ZK_CIRCUIT_SCHEMA_VERSION_V0: &str = "0";
pub const ZK_CIRCUIT_LAYER_MARKER_V0: &str = "cryptographic-implementation";
pub const ZK_CIRCUIT_FEATURE_GATE_V0: &str = "zk-circuit";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CascadeCircuitSpecV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub circuit_id: String,
    pub arithmetization: ArithmetizationKindV0,
    pub constraint_count: usize,
    pub salsa_dependency_free: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ArithmetizationKindV0 {
    R1cs,
    Plonkish,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct R1CSConstraintV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub constraint_id: String,
    pub left_wire: String,
    pub right_wire: String,
    pub output_wire: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlonkishGateV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub gate_id: String,
    pub selector_polynomial: String,
}

pub fn cascade_circuit_spec_v0(
    circuit_id: impl Into<String>,
    arithmetization: ArithmetizationKindV0,
) -> CascadeCircuitSpecV0 {
    CascadeCircuitSpecV0 {
        schema_version: ZK_CIRCUIT_SCHEMA_VERSION_V0,
        product: "omena-zk-circuit.cascade-circuit-spec",
        layer_marker: ZK_CIRCUIT_LAYER_MARKER_V0,
        feature_gate: ZK_CIRCUIT_FEATURE_GATE_V0,
        circuit_id: circuit_id.into(),
        arithmetization,
        constraint_count: 0,
        salsa_dependency_free: true,
    }
}

pub fn cascade_circuit_spec_from_canonical_terms_v0(
    circuit_id: impl Into<String>,
    canonical_terms: &[String],
) -> CascadeCircuitSpecV0 {
    let constraints = cascade_r1cs_constraints_from_canonical_terms_v0(canonical_terms);
    CascadeCircuitSpecV0 {
        constraint_count: constraints.len(),
        ..cascade_circuit_spec_v0(circuit_id, ArithmetizationKindV0::R1cs)
    }
}

pub fn cascade_r1cs_constraints_from_canonical_terms_v0(
    canonical_terms: &[String],
) -> Vec<R1CSConstraintV0> {
    canonical_terms
        .iter()
        .filter_map(|term| {
            let (name, _value) = term.strip_prefix("require:")?.rsplit_once('=')?;
            Some(R1CSConstraintV0 {
                schema_version: ZK_CIRCUIT_SCHEMA_VERSION_V0,
                product: "omena-zk-circuit.r1cs-constraint",
                layer_marker: ZK_CIRCUIT_LAYER_MARKER_V0,
                feature_gate: ZK_CIRCUIT_FEATURE_GATE_V0,
                constraint_id: format!("requirement-{name}"),
                left_wire: format!("witness.{name}"),
                right_wire: "public.one".to_string(),
                output_wire: "public.one".to_string(),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn circuit_contract_is_pure_and_schema_zero() {
        let spec = cascade_circuit_spec_v0("cascade", ArithmetizationKindV0::Plonkish);
        assert_eq!(spec.schema_version, "0");
        assert!(spec.salsa_dependency_free);
    }

    #[test]
    fn circuit_spec_counts_r1cs_constraints_from_canonical_terms() {
        let terms = vec![
            "require:supported-shorthand-property=true".to_string(),
            "require:canonical-longhand-quartet=true".to_string(),
            "unknown:supports-condition".to_string(),
        ];
        let constraints = cascade_r1cs_constraints_from_canonical_terms_v0(&terms);
        let spec = cascade_circuit_spec_from_canonical_terms_v0("cascade-smt", &terms);

        assert_eq!(constraints.len(), 2);
        assert_eq!(spec.arithmetization, ArithmetizationKindV0::R1cs);
        assert_eq!(spec.constraint_count, constraints.len());
        assert_eq!(
            constraints[0].constraint_id,
            "requirement-supported-shorthand-property"
        );
    }
}
