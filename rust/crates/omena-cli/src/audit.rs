use crate::{
    commands::{AuditCommand, ZkAuditCommand},
    output::print_json,
};
use omena_zk_audit::{
    ArkworksGroth16RoundTripV0, CascadeZKAuditV0, ZK_AUDIT_DEFAULT_PROOF_BACKEND_ENABLED_V0,
    ZK_AUDIT_MECHANISM_SCOPE_V0, ZKAuditCiMatrixV0, active_zk_audit_proof_backend_scope_v0,
    cascade_zk_audit_v0, prove_and_verify_canonical_margin_cascade_with_arkworks_v0,
    zk_audit_ci_matrix_v0,
};
use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ZkAuditCliResultV0 {
    schema_version: &'static str,
    product: &'static str,
    layer_marker: &'static str,
    feature_gate: &'static str,
    mechanism_scope: &'static str,
    default_proof_backend_enabled: bool,
    active_proof_backend_scope: &'static str,
    command: &'static str,
    audit: Option<CascadeZKAuditV0>,
    ci_matrix: Option<ZKAuditCiMatrixV0>,
    #[serde(skip_serializing_if = "Option::is_none")]
    groth16_roundtrip: Option<ArkworksGroth16RoundTripV0>,
    verified: bool,
}

pub(crate) fn audit_command(command: AuditCommand) -> Result<(), String> {
    match command {
        AuditCommand::Zk { command } => zk_audit_command(command),
    }
}

fn zk_audit_command(command: ZkAuditCommand) -> Result<(), String> {
    match command {
        ZkAuditCommand::Prove {
            audit_id,
            reorder,
            json,
        } => {
            let roundtrip = prove_and_verify_canonical_margin_cascade_with_arkworks_v0(reorder)?;
            let verified = roundtrip.proof_generated && roundtrip.proof_verified;
            let result = zk_audit_cli_result_v0(
                "omena-cli.audit.zk.prove",
                "prove",
                Some(cascade_zk_audit_v0(audit_id)),
                None,
                Some(roundtrip),
                verified,
            );
            print_zk_audit_result(&result, json)
        }
        ZkAuditCommand::Verify {
            audit_id,
            reorder,
            json,
        } => {
            let roundtrip = prove_and_verify_canonical_margin_cascade_with_arkworks_v0(reorder)?;
            let verified = roundtrip.proof_generated && roundtrip.proof_verified;
            let result = zk_audit_cli_result_v0(
                "omena-cli.audit.zk.verify",
                "verify",
                Some(cascade_zk_audit_v0(audit_id)),
                None,
                Some(roundtrip),
                verified,
            );
            print_zk_audit_result(&result, json)
        }
        ZkAuditCommand::SetupStatus { json } => {
            let result = zk_audit_cli_result_v0(
                "omena-cli.audit.zk.setup-status",
                "setup-status",
                None,
                Some(zk_audit_ci_matrix_v0()),
                None,
                false,
            );
            print_zk_audit_result(&result, json)
        }
    }
}

fn zk_audit_cli_result_v0(
    product: &'static str,
    command: &'static str,
    audit: Option<CascadeZKAuditV0>,
    ci_matrix: Option<ZKAuditCiMatrixV0>,
    groth16_roundtrip: Option<ArkworksGroth16RoundTripV0>,
    verified: bool,
) -> ZkAuditCliResultV0 {
    ZkAuditCliResultV0 {
        schema_version: "0",
        product,
        layer_marker: "cryptographic-implementation",
        feature_gate: "zk-audit",
        mechanism_scope: ZK_AUDIT_MECHANISM_SCOPE_V0,
        default_proof_backend_enabled: ZK_AUDIT_DEFAULT_PROOF_BACKEND_ENABLED_V0,
        active_proof_backend_scope: active_zk_audit_proof_backend_scope_v0(),
        command,
        audit,
        ci_matrix,
        groth16_roundtrip,
        verified,
    }
}

fn print_zk_audit_result(result: &ZkAuditCliResultV0, json: bool) -> Result<(), String> {
    if json {
        print_json(result)?;
        return Ok(());
    }

    println!("product: {}", result.product);
    println!("command: {}", result.command);
    println!("feature gate: {}", result.feature_gate);
    println!("mechanism scope: {}", result.mechanism_scope);
    println!(
        "default proof backend enabled: {}",
        result.default_proof_backend_enabled
    );
    println!(
        "active proof backend scope: {}",
        result.active_proof_backend_scope
    );
    println!("verified: {}", result.verified);
    if let Some(audit) = &result.audit {
        println!("audit: {}", audit.audit_id);
        println!("setup: {:?}", audit.setup_kind);
        println!("recursion overhead: {}", audit.recursion_overhead);
    }
    if let Some(roundtrip) = &result.groth16_roundtrip {
        println!("backend: {}", roundtrip.backend);
        println!("obligation: {}", roundtrip.obligation_id);
        println!("constraint count: {}", roundtrip.circuit.constraint_count);
        println!("requirement count: {}", roundtrip.requirement_count);
        println!("proof generated: {}", roundtrip.proof_generated);
        println!("proof verified: {}", roundtrip.proof_verified);
    }
    if let Some(ci_matrix) = &result.ci_matrix {
        println!("ci cells: {}", ci_matrix.cells.join(","));
        println!(
            "heavy dependencies default off: {}",
            ci_matrix.heavy_dependencies_default_off
        );
    }
    Ok(())
}
