use crate::{commands::ProvenanceCommand, io::read_source, output::print_json, paths::path_string};
use omena_sif::{read_omena_lock_json_v1, summarize_omena_sif_provenance_advisory_v1};
use std::path::PathBuf;

pub(crate) fn provenance_command(command: ProvenanceCommand) -> Result<(), String> {
    match command {
        ProvenanceCommand::Status { lockfile, json } => provenance_status(lockfile, json),
    }
}

fn provenance_status(lockfile: PathBuf, json: bool) -> Result<(), String> {
    let lockfile_source = read_source(&lockfile)?;
    let lock = read_omena_lock_json_v1(&lockfile_source)
        .map_err(|error| format!("failed to parse {}: {error}", path_string(&lockfile)))?;
    let report = summarize_omena_sif_provenance_advisory_v1(&lock);

    if json {
        print_json(&report)?;
    } else {
        match report.enforcement {
            "lockVerifyTier2Tier3WhenRequested" => {
                println!(
                    "SIF provenance enforcement is available through `omena lock verify --tier t2|t3`."
                );
            }
            "invalidRecordedAttestationEvidence" => {
                println!(
                    "SIF provenance has invalid recorded attestation evidence; run `omena lock verify --tier t2|t3`."
                );
            }
            _ => {
                println!(
                    "SIF provenance references are advisory until verified attestation evidence is recorded."
                );
            }
        }
        println!(
            "network access: {}; entries: {}",
            report.network_access,
            report.entries.len()
        );
        for entry in &report.entries {
            println!(
                "{} {} attestations={} recordedVerifications={} verifiedAttestations={} invalidAttestations={}: {}",
                entry.trust_tier.as_str(),
                entry.canonical_url,
                entry.attestation_reference_count,
                entry.recorded_attestation_verification_count,
                entry.attestation_verification_count,
                entry.invalid_attestation_verification_count,
                entry.advisory_message
            );
            for policy in &entry.attestation_verification_policies {
                println!(
                    "  verified policy {} kind={} issuer={} identity={}",
                    policy.verified_trust_tier.as_str(),
                    policy.kind,
                    policy
                        .certificate_issuer
                        .as_deref()
                        .unwrap_or("unspecified"),
                    policy
                        .certificate_identity
                        .as_deref()
                        .unwrap_or("unspecified")
                );
            }
            for issue in &entry.invalid_attestation_verification_issues {
                println!(
                    "  invalid attestation {} kind={} verifier={}: {}",
                    issue.verified_trust_tier.as_str(),
                    issue.kind,
                    issue.verifier,
                    issue.reason
                );
            }
        }
    }

    Ok(())
}
