//! Zero-knowledge cascade audit protocol contracts.
//!
//! Heavy proving systems remain opt-in features. The default build records the
//! protocol and CI matrix without linking arkworks, halo2, winterfell, or
//! binius.
//!
//! claim_level: opt-in arkworks proof round-trip, while the default build stays
//! protocol metadata only.

use omena_smt::CanonicalSmtInputV0;
use omena_zk_circuit::{
    ArithmetizationKindV0, CascadeCircuitSpecV0, cascade_circuit_spec_from_canonical_terms_v0,
};
use serde::Serialize;

#[cfg(feature = "zk-audit")]
pub mod arkworks;

#[cfg(feature = "zk-audit")]
pub use arkworks::{
    ArkworksGroth16RoundTripV0, prove_and_verify_box_shorthand_cascade_with_arkworks_v0,
    prove_and_verify_canonical_margin_cascade_with_arkworks_v0,
    prove_and_verify_cascade_smt_payload_with_arkworks_v0,
};

pub const ZK_AUDIT_SCHEMA_VERSION_V0: &str = "0";
pub const ZK_AUDIT_LAYER_MARKER_V0: &str = "cryptographic-implementation";
pub const ZK_AUDIT_FEATURE_GATE_V0: &str = "zk-audit";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum SetupKindV0 {
    ArkworksGroth16,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ZKBackendLinkStatusV0 {
    ProtocolOnly,
    OptInFeatureDeclared,
    RealBackendLinked,
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
pub struct ZKBackendLinkPolicyV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub layer_marker: &'static str,
    pub feature_gate: &'static str,
    pub setup_kind: SetupKindV0,
    pub cargo_feature: &'static str,
    pub status: ZKBackendLinkStatusV0,
    pub default_enabled: bool,
    pub proof_generation_available: bool,
    pub verification_available: bool,
    pub external_dependency_family: Option<&'static str>,
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
    let circuit = cascade_circuit_spec_from_canonical_terms_v0(
        format!("cascade-smt-{}", proof_payload.obligation_id),
        &proof_payload.canonical_terms,
    );
    CascadeZKAuditV0 {
        circuit,
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

pub fn zk_backend_link_policy_v0() -> Vec<ZKBackendLinkPolicyV0> {
    vec![
        zk_backend_link_policy_cell_v0(
            SetupKindV0::ArkworksGroth16,
            "zk-audit",
            "arkworks-groth16",
            ZKBackendLinkStatusV0::RealBackendLinked,
        ),
        zk_backend_link_policy_cell_v0(
            SetupKindV0::PlonkUniversal,
            "zk-audit",
            "plonk",
            ZKBackendLinkStatusV0::ProtocolOnly,
        ),
        zk_backend_link_policy_cell_v0(
            SetupKindV0::StarkFri,
            "zk-audit-stark",
            "winterfell",
            ZKBackendLinkStatusV0::OptInFeatureDeclared,
        ),
        zk_backend_link_policy_cell_v0(
            SetupKindV0::Binius,
            "zk-audit-binius",
            "binius",
            ZKBackendLinkStatusV0::OptInFeatureDeclared,
        ),
    ]
}

fn zk_backend_link_policy_cell_v0(
    setup_kind: SetupKindV0,
    cargo_feature: &'static str,
    external_dependency_family: &'static str,
    status: ZKBackendLinkStatusV0,
) -> ZKBackendLinkPolicyV0 {
    let real_backend_linked = status == ZKBackendLinkStatusV0::RealBackendLinked;
    ZKBackendLinkPolicyV0 {
        schema_version: ZK_AUDIT_SCHEMA_VERSION_V0,
        product: "omena-zk-audit.backend-link-policy",
        layer_marker: ZK_AUDIT_LAYER_MARKER_V0,
        feature_gate: ZK_AUDIT_FEATURE_GATE_V0,
        setup_kind,
        cargo_feature,
        status,
        default_enabled: false,
        proof_generation_available: real_backend_linked,
        verification_available: real_backend_linked,
        external_dependency_family: Some(external_dependency_family),
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
            vec![
                "require:supported-shorthand-property=true".to_string(),
                "require:canonical-longhand-quartet=true".to_string(),
            ],
        );
        let audit = cascade_zk_audit_with_smt_payload_v0("audit", payload);
        assert!(audit.proof_payload.is_some());
        assert_eq!(audit.setup_kind, SetupKindV0::Halo2Ipa);
        assert_eq!(audit.circuit.arithmetization, ArithmetizationKindV0::R1cs);
        assert_eq!(audit.circuit.constraint_count, 2);
    }

    #[test]
    fn salsa_ivc_fold_chain_keeps_constant_recursion_overhead_for_ten_steps() {
        let chain = zk_audit_fold_chain_v0(10);
        assert_eq!(chain.len(), 10);
        assert!(chain.iter().all(|step| step.schema_version == "0"));
        assert!(chain.iter().all(|step| step.recursion_overhead == "O(1)"));
    }

    #[test]
    fn zk_backend_link_policy_keeps_real_backends_feature_gated() {
        let policy = zk_backend_link_policy_v0();

        assert_eq!(policy.len(), 4);
        assert!(policy.iter().all(|cell| !cell.default_enabled));
        assert!(policy.iter().all(|cell| cell.schema_version == "0"));
        assert!(
            policy
                .iter()
                .any(|cell| cell.status == ZKBackendLinkStatusV0::RealBackendLinked)
        );
        assert!(
            policy
                .iter()
                .any(|cell| cell.proof_generation_available && cell.verification_available)
        );
        assert!(
            policy
                .iter()
                .any(|cell| cell.setup_kind == SetupKindV0::ArkworksGroth16
                    && cell.cargo_feature == "zk-audit"
                    && cell.external_dependency_family == Some("arkworks-groth16"))
        );
    }

    #[cfg(feature = "zk-audit")]
    #[test]
    fn arkworks_groth16_roundtrip_generates_and_verifies_proof() {
        let payload = omena_smt::canonical_smt_input_v0(
            "box-shorthand-combination",
            "prove_box_shorthand_combination",
            vec![
                "require:supported-shorthand-property=true".to_string(),
                "require:canonical-longhand-quartet=true".to_string(),
                "require:no-important-longhand=true".to_string(),
            ],
        );
        let roundtrip = prove_and_verify_cascade_smt_payload_with_arkworks_v0(&payload);
        assert!(roundtrip.is_ok(), "{roundtrip:?}");
        if let Ok(roundtrip) = roundtrip {
            assert_eq!(roundtrip.setup_kind, SetupKindV0::ArkworksGroth16);
            assert_eq!(roundtrip.requirement_count, 3);
            assert!(roundtrip.proof_generated);
            assert!(roundtrip.proof_verified);
            assert_eq!(roundtrip.circuit.constraint_count, 3);
        }
    }

    #[cfg(feature = "zk-audit")]
    #[test]
    fn arkworks_rejects_unsatisfied_circuit_witness_without_proof_generation() {
        let payload = omena_smt::canonical_smt_input_v0(
            "box-shorthand-combination",
            "prove_box_shorthand_combination",
            vec![
                "require:supported-shorthand-property=true".to_string(),
                "require:canonical-longhand-quartet=false".to_string(),
                "require:no-important-longhand=true".to_string(),
            ],
        );
        let roundtrip = prove_and_verify_cascade_smt_payload_with_arkworks_v0(&payload);
        assert!(roundtrip.is_ok(), "{roundtrip:?}");
        if let Ok(roundtrip) = roundtrip {
            assert_eq!(roundtrip.circuit.constraint_count, 3);
            assert_eq!(roundtrip.requirement_count, 3);
            assert!(!roundtrip.proof_generated);
            assert!(!roundtrip.proof_verified);
        }
    }
}
