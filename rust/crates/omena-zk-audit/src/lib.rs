//! Zero-knowledge cascade audit protocol contracts.
//!
//! Heavy proving systems remain opt-in features. The default build records the
//! protocol and CI matrix without linking arkworks, halo2, winterfell, or
//! binius.

use omena_smt::CanonicalSmtInputV0;
use omena_zk_circuit::{ArithmetizationKindV0, CascadeCircuitSpecV0};
use serde::Serialize;

pub const ZK_AUDIT_SCHEMA_VERSION_V0: &str = "0";
pub const ZK_AUDIT_LAYER_MARKER_V0: &str = "cryptographic-implementation";
pub const ZK_AUDIT_FEATURE_GATE_V0: &str = "zk-audit";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum SetupKindV0 {
    Halo2Ipa,
    PlonkUniversal,
    StarkFri,
    Binius,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum SemiringKindV0 {
    Lin01,
    NaturalCount,
    Tropical,
    Viterbi,
    SecurityLabel,
    MultivariatePolynomial,
    ProbabilityLog,
    Custom {
        id: String,
        k_carrier_digest: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CascadeZKAuditV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub audit_id: String,
    pub setup_kind: SetupKindV0,
    pub circuit: CascadeCircuitSpecV0,
    pub proof_payload: Option<CanonicalSmtInputV0>,
    pub per_pr_delta_fold: bool,
    pub recursion_overhead: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ZKPolynomialProvenanceV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub semiring_kind: SemiringKindV0,
    pub polynomial_commitment: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ZKAuditCiMatrixV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub cells: Vec<&'static str>,
    pub heavy_dependencies_default_off: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ZKFoldChainStepV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub step_index: usize,
    pub accumulator_digest: String,
    pub recursion_overhead: &'static str,
}

pub fn cascade_zk_audit_v0(audit_id: impl Into<String>) -> CascadeZKAuditV0 {
    CascadeZKAuditV0 {
        schema_version: ZK_AUDIT_SCHEMA_VERSION_V0,
        product: "omena-zk-audit.cascade-audit",
        layer_marker: ZK_AUDIT_LAYER_MARKER_V0,
        feature_gate: ZK_AUDIT_FEATURE_GATE_V0,
        audit_id: audit_id.into(),
        setup_kind: SetupKindV0::Halo2Ipa,
        circuit: CascadeCircuitSpecV0 {
            schema_version: "0",
            product: "omena-zk-circuit.cascade-circuit-spec",
            layer_marker: "cryptographic-implementation",
            feature_gate: "zk-circuit",
            circuit_id: "cascade".to_string(),
            arithmetization: ArithmetizationKindV0::Plonkish,
            constraint_count: 0,
            salsa_dependency_free: true,
        },
        proof_payload: None,
        per_pr_delta_fold: true,
        recursion_overhead: "O(1)",
    }
}

pub fn cascade_zk_audit_with_smt_payload_v0(
    audit_id: impl Into<String>,
    proof_payload: CanonicalSmtInputV0,
) -> CascadeZKAuditV0 {
    CascadeZKAuditV0 {
        proof_payload: Some(proof_payload),
        ..cascade_zk_audit_v0(audit_id)
    }
}

pub fn zk_audit_ci_matrix_v0() -> ZKAuditCiMatrixV0 {
    ZKAuditCiMatrixV0 {
        schema_version: ZK_AUDIT_SCHEMA_VERSION_V0,
        product: "omena-zk-audit.ci-matrix",
        layer_marker: ZK_AUDIT_LAYER_MARKER_V0,
        feature_gate: ZK_AUDIT_FEATURE_GATE_V0,
        cells: vec!["default", "zk-audit", "zk-audit-stark", "zk-audit-binius"],
        heavy_dependencies_default_off: true,
    }
}

pub fn zk_audit_fold_chain_v0(step_count: usize) -> Vec<ZKFoldChainStepV0> {
    (0..step_count)
        .map(|step_index| ZKFoldChainStepV0 {
            schema_version: ZK_AUDIT_SCHEMA_VERSION_V0,
            product: "omena-zk-audit.fold-chain-step",
            layer_marker: ZK_AUDIT_LAYER_MARKER_V0,
            feature_gate: ZK_AUDIT_FEATURE_GATE_V0,
            step_index,
            accumulator_digest: format!("salsa-ivc-fold-v0-{step_index:04}"),
            recursion_overhead: "O(1)",
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn audit_default_keeps_heavy_dependencies_off() {
        let audit = cascade_zk_audit_v0("audit");
        let matrix = zk_audit_ci_matrix_v0();
        assert_eq!(audit.schema_version, "0");
        assert_eq!(audit.setup_kind, SetupKindV0::Halo2Ipa);
        assert!(matrix.heavy_dependencies_default_off);
        assert_eq!(matrix.cells.len(), 4);
    }

    #[test]
    fn audit_accepts_canonical_smt_payload_without_enabling_heavy_deps() {
        let payload = omena_smt::canonical_smt_input_v0(
            "obligation",
            "prove_box_shorthand_combination",
            vec!["margin".to_string()],
        );
        let audit = cascade_zk_audit_with_smt_payload_v0("audit", payload);
        assert!(audit.proof_payload.is_some());
        assert_eq!(audit.setup_kind, SetupKindV0::Halo2Ipa);
    }

    #[test]
    fn salsa_ivc_fold_chain_keeps_constant_recursion_overhead_for_ten_steps() {
        let chain = zk_audit_fold_chain_v0(10);
        assert_eq!(chain.len(), 10);
        assert!(chain.iter().all(|step| step.schema_version == "0"));
        assert!(chain.iter().all(|step| step.recursion_overhead == "O(1)"));
    }
}
