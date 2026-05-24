//! Pure circuit-side contracts for zero-knowledge cascade audit.

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn circuit_contract_is_pure_and_schema_zero() {
        let spec = cascade_circuit_spec_v0("cascade", ArithmetizationKindV0::Plonkish);
        assert_eq!(spec.schema_version, "0");
        assert!(spec.salsa_dependency_free);
    }
}
