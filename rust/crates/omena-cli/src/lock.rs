use crate::{
    commands::LockCommand,
    io::read_source,
    output::{CliOutputMetadataV0, print_json},
    paths::path_string,
};
use omena_query::summarize_omena_query_sass_module_sources;
use omena_sif::{
    OMENA_SIF_ATTESTATION_VERIFICATION_REPORT_PRODUCT_V1,
    OMENA_SIF_ATTESTATION_VERIFICATION_REPORT_SCHEMA_VERSION_V1, OmenaLockV1,
    OmenaLockVerificationIssueV1, OmenaSifAttestationStatementV1,
    OmenaSifAttestationSubjectDigestV1, OmenaSifAttestationVerificationReportV1,
    OmenaSifSigstoreVerificationPolicyV1,
    apply_omena_sif_attestation_verification_report_to_lock_entry_v1,
    apply_omena_sif_npm_provenance_references_to_lock_entry_v1, build_omena_lock_sif_entry_v1,
    collect_omena_sif_npm_provenance_attestation_references_v1, compute_omena_sif_artifact_hash_v1,
    read_omena_lock_json_v1, read_omena_sif_attestation_verification_report_json_v1,
    read_omena_sif_json_v1, verify_omena_lock_frozen_v1, write_omena_lock_json_v1,
};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::{Path, PathBuf},
};

pub(crate) fn lock_command(
    status_lockfile: PathBuf,
    status_json: bool,
    command: Option<LockCommand>,
) -> Result<(), String> {
    match command {
        None => lock_status(status_lockfile, status_json),
        Some(LockCommand::Update {
            package,
            lockfile,
            sif_paths,
            json,
        }) => lock_update(lockfile, package, sif_paths, json),
        Some(LockCommand::Add {
            package,
            lockfile,
            sif_paths,
            json,
        }) => lock_add(lockfile, package, sif_paths, json),
        Some(LockCommand::FetchProvenance {
            package,
            lockfile,
            npm_metadata,
            json,
        }) => lock_fetch_provenance(lockfile, package, npm_metadata, json),
        Some(LockCommand::RecordVerification {
            package,
            lockfile,
            verification,
            artifact,
            json,
        }) => lock_record_verification(lockfile, package, verification, artifact, json),
        Some(LockCommand::VerifyAttestation {
            package,
            lockfile,
            artifact,
            bundle,
            reference,
            kind,
            verified_tier,
            identity,
            issuer,
            statement_type,
            statement_predicate_type,
            statement_source_repository,
            statement_source_ref,
            statement_source_commit,
            statement_builder_id,
            statement_build_type,
            statement_subject_names,
            statement_subject_digests,
            json,
        }) => lock_verify_attestation(LockVerifyAttestationInput {
            lockfile,
            package,
            artifact,
            bundle,
            reference,
            kind,
            verified_tier,
            identity,
            issuer,
            statement_policy: AttestationStatementPolicy {
                statement_type,
                predicate_type: statement_predicate_type,
                source_repository: statement_source_repository,
                source_ref: statement_source_ref,
                source_commit: statement_source_commit,
                builder_id: statement_builder_id,
                build_type: statement_build_type,
                subject_names: statement_subject_names,
                subject_digests: statement_subject_digests,
            },
            json,
        }),
        Some(LockCommand::Verify {
            lockfile,
            source_paths,
            frozen,
            tier,
            json,
        }) => lock_verify(lockfile, source_paths, frozen, tier, json),
    }
}

fn lock_update(
    lockfile: PathBuf,
    package: Option<String>,
    sif_paths: Vec<PathBuf>,
    json: bool,
) -> Result<(), String> {
    if sif_paths.is_empty() {
        return Err("omena lock update requires at least one --sif <path>".to_string());
    }

    let mut entries = build_lock_entries_from_sif_paths(&lockfile, &sif_paths)?;
    if let Some(package) = package.as_deref() {
        entries = filter_lock_entries_for_package(entries, package)?;
        let existing = read_lockfile_or_empty(&lockfile)?;
        let mut merged = existing
            .entries
            .into_iter()
            .filter(|entry| !lock_entry_matches_package_selector(entry, package))
            .collect::<Vec<_>>();
        merged.extend(entries);
        entries = merged;
    }

    let lock = OmenaLockV1::new(entries);
    let lock_json = write_omena_lock_json_v1(&lock)
        .map_err(|error| format!("failed to serialize {}: {error}", path_string(&lockfile)))?;
    fs::write(&lockfile, &lock_json)
        .map_err(|error| format!("failed to write {}: {error}", path_string(&lockfile)))?;

    if json {
        print_json(CliOutputMetadataV0::new("omena-cli.lock-update"), &lock)?;
    } else {
        println!(
            "omena.lock updated: {} SIF entr{} recorded at {}",
            lock.entries.len(),
            if lock.entries.len() == 1 { "y" } else { "ies" },
            path_string(&lockfile)
        );
    }

    Ok(())
}

fn lock_add(
    lockfile: PathBuf,
    package: String,
    sif_paths: Vec<PathBuf>,
    json: bool,
) -> Result<(), String> {
    if sif_paths.is_empty() {
        return Err("omena lock add requires at least one --sif <path>".to_string());
    }

    let existing = read_lockfile_or_empty(&lockfile)?;
    if existing
        .entries
        .iter()
        .any(|entry| lock_entry_matches_package_selector(entry, &package))
    {
        return Err(format!(
            "omena.lock already contains entries for '{package}'; use `omena lock update {package}` to refresh them"
        ));
    }

    let mut entries = existing.entries;
    entries.extend(filter_lock_entries_for_package(
        build_lock_entries_from_sif_paths(&lockfile, &sif_paths)?,
        &package,
    )?);
    let lock = OmenaLockV1::new(entries);
    let lock_json = write_omena_lock_json_v1(&lock)
        .map_err(|error| format!("failed to serialize {}: {error}", path_string(&lockfile)))?;
    fs::write(&lockfile, &lock_json)
        .map_err(|error| format!("failed to write {}: {error}", path_string(&lockfile)))?;

    let added_count = lock
        .entries
        .iter()
        .filter(|entry| lock_entry_matches_package_selector(entry, &package))
        .count();
    if json {
        print_json(CliOutputMetadataV0::new("omena-cli.lock-add"), &lock)?;
    } else {
        println!(
            "omena.lock added '{package}': {} SIF entr{} recorded at {}",
            added_count,
            if added_count == 1 { "y" } else { "ies" },
            path_string(&lockfile)
        );
    }

    Ok(())
}

fn lock_fetch_provenance(
    lockfile: PathBuf,
    package: String,
    npm_metadata: PathBuf,
    json: bool,
) -> Result<(), String> {
    let mut lock = read_lockfile_or_empty(&lockfile)?;
    let metadata_source = read_source(&npm_metadata)?;
    let references =
        collect_omena_sif_npm_provenance_attestation_references_v1(&metadata_source)
            .map_err(|error| format!("failed to parse {}: {error}", path_string(&npm_metadata)))?;
    if references.is_empty() {
        return Err(format!(
            "npm metadata {} does not contain dist.attestations.provenance",
            path_string(&npm_metadata)
        ));
    }

    let mut matched_count = 0usize;
    let mut added_reference_count = 0usize;
    for entry in &mut lock.entries {
        if !lock_entry_matches_package_selector(entry, &package) {
            continue;
        }
        matched_count += 1;
        added_reference_count += apply_omena_sif_npm_provenance_references_to_lock_entry_v1(
            entry,
            references.as_slice(),
        );
    }
    if matched_count == 0 {
        return Err(format!(
            "omena.lock contains no entries for package or canonical URL selector '{package}'"
        ));
    }

    let lock_json = write_omena_lock_json_v1(&lock)
        .map_err(|error| format!("failed to serialize {}: {error}", path_string(&lockfile)))?;
    fs::write(&lockfile, &lock_json)
        .map_err(|error| format!("failed to write {}: {error}", path_string(&lockfile)))?;

    if json {
        print_json(
            CliOutputMetadataV0::new("omena-cli.lock-fetch-provenance"),
            &lock,
        )?;
    } else {
        println!(
            "omena.lock provenance updated: {matched_count} entr{} matched, {added_reference_count} reference{} added",
            if matched_count == 1 { "y" } else { "ies" },
            if added_reference_count == 1 { "" } else { "s" },
        );
    }

    Ok(())
}

fn lock_record_verification(
    lockfile: PathBuf,
    package: String,
    verification: PathBuf,
    artifact: Option<PathBuf>,
    json: bool,
) -> Result<(), String> {
    let mut lock = read_lockfile_or_empty(&lockfile)?;
    let verification_source = read_source(&verification)?;
    let report = read_omena_sif_attestation_verification_report_json_v1(&verification_source)
        .map_err(|error| format!("failed to parse {}: {error}", path_string(&verification)))?;

    let mut matched_count = 0usize;
    let mut applied_count = 0usize;
    for entry in &mut lock.entries {
        if !lock_entry_matches_package_selector(entry, &package) {
            continue;
        }
        matched_count += 1;
        let mut updated_entry = entry.clone();
        let applied = apply_omena_sif_attestation_verification_report_to_lock_entry_v1(
            &mut updated_entry,
            &report,
        )
        .map_err(|error| {
            format!(
                "attestation verification report {} rejected for {}: {error}",
                path_string(&verification),
                entry.canonical_url
            )
        })?;
        if applied {
            if report.verified_trust_tier == omena_sif::OmenaSifTrustTierV1::T3 {
                let artifact = artifact.as_ref().ok_or_else(|| {
                    "lock record-verification with verifiedTrustTier t3 requires --artifact to be the matching SIF JSON".to_string()
                })?;
                let artifact_bytes = fs::read(artifact).map_err(|error| {
                    format!("failed to read {}: {error}", path_string(artifact))
                })?;
                let binding = read_verified_t3_attestation_artifact_binding(
                    artifact,
                    artifact_bytes.as_slice(),
                )?;
                validate_verified_t3_attestation_artifact_binding(entry, &binding)?;
                validate_verified_t3_attestation_statement_binding(
                    entry,
                    &binding,
                    report.attestation_statement.as_ref(),
                )?;
            }
            *entry = updated_entry;
            applied_count += 1;
        }
    }
    if matched_count == 0 {
        return Err(format!(
            "omena.lock contains no entries for package or canonical URL selector '{package}'"
        ));
    }
    if applied_count == 0 {
        return Err(format!(
            "attestation verification report {} did not match any selected lock entry subject",
            path_string(&verification)
        ));
    }

    let lock_json = write_omena_lock_json_v1(&lock)
        .map_err(|error| format!("failed to serialize {}: {error}", path_string(&lockfile)))?;
    fs::write(&lockfile, &lock_json)
        .map_err(|error| format!("failed to write {}: {error}", path_string(&lockfile)))?;

    if json {
        print_json(
            CliOutputMetadataV0::new("omena-cli.lock-record-verification"),
            &lock,
        )?;
    } else {
        println!(
            "omena.lock verification updated: {applied_count} entr{} recorded from {}",
            if applied_count == 1 { "y" } else { "ies" },
            path_string(&verification)
        );
    }

    Ok(())
}

struct LockVerifyAttestationInput {
    lockfile: PathBuf,
    package: String,
    artifact: PathBuf,
    bundle: PathBuf,
    reference: String,
    kind: String,
    verified_tier: String,
    identity: Option<String>,
    issuer: String,
    statement_policy: AttestationStatementPolicy,
    json: bool,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct AttestationStatementPolicy {
    pub(crate) statement_type: Option<String>,
    pub(crate) predicate_type: Option<String>,
    pub(crate) source_repository: Option<String>,
    pub(crate) source_ref: Option<String>,
    pub(crate) source_commit: Option<String>,
    pub(crate) builder_id: Option<String>,
    pub(crate) build_type: Option<String>,
    pub(crate) subject_names: Vec<String>,
    pub(crate) subject_digests: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct AttestationStatementSubjectDigestPolicy {
    name: String,
    algorithm: String,
    digest: String,
}

impl AttestationStatementPolicy {
    fn is_empty(&self) -> bool {
        self.statement_type.is_none()
            && self.predicate_type.is_none()
            && self.source_repository.is_none()
            && self.source_ref.is_none()
            && self.source_commit.is_none()
            && self.builder_id.is_none()
            && self.build_type.is_none()
            && self.subject_names.is_empty()
            && self.subject_digests.is_empty()
    }
}

fn lock_verify_attestation(input: LockVerifyAttestationInput) -> Result<(), String> {
    let LockVerifyAttestationInput {
        lockfile,
        package,
        artifact,
        bundle,
        reference,
        kind,
        verified_tier,
        identity,
        issuer,
        statement_policy,
        json,
    } = input;

    let verified_trust_tier = parse_lock_trust_tier(Some(verified_tier.as_str()))?
        .ok_or_else(|| "lock verify-attestation requires --verified-tier".to_string())?;
    if verified_trust_tier < omena_sif::OmenaSifTrustTierV1::T2 {
        return Err(format!(
            "lock verify-attestation --verified-tier must be t2 or t3, got {}",
            verified_trust_tier.as_str()
        ));
    }
    validate_attestation_policy_for_verified_tier(
        kind.as_str(),
        verified_trust_tier,
        Some(issuer.as_str()),
        identity.as_deref(),
    )?;

    let mut lock = read_lockfile_or_empty(&lockfile)?;
    let artifact_bytes = fs::read(&artifact)
        .map_err(|error| format!("failed to read {}: {error}", path_string(&artifact)))?;
    let bundle_source = read_source(&bundle)?;
    let sigstore_bundle = sigstore_verify::types::Bundle::from_json(&bundle_source)
        .map_err(|error| format!("failed to parse {}: {error}", path_string(&bundle)))?;
    let trusted_root = sigstore_verify::trust_root::TrustedRoot::from_json(
        sigstore_verify::trust_root::SIGSTORE_PRODUCTION_TRUSTED_ROOT,
    )
    .map_err(|error| format!("failed to load Sigstore production trust root: {error}"))?;

    let mut policy = sigstore_verify::VerificationPolicy::default();
    if let Some(identity) = identity.as_ref() {
        policy = policy.require_identity(identity.clone());
    }
    policy = policy.require_issuer(issuer.clone());
    let verification_result = sigstore_verify::verify(
        artifact_bytes.as_slice(),
        &sigstore_bundle,
        &policy,
        &trusted_root,
    )
    .map_err(|error| {
        format!(
            "sigstore verification failed for {} with {}: {error}",
            path_string(&artifact),
            path_string(&bundle)
        )
    })?;

    if !verification_result.success {
        return Err(format!(
            "sigstore verification did not succeed for {} with {}",
            path_string(&artifact),
            path_string(&bundle)
        ));
    }
    let t3_artifact_binding = if verified_trust_tier == omena_sif::OmenaSifTrustTierV1::T3 {
        Some(read_verified_t3_attestation_artifact_binding(
            &artifact,
            artifact_bytes.as_slice(),
        )?)
    } else {
        None
    };
    let attestation_statement = extract_verified_attestation_statement(
        &sigstore_bundle,
        &statement_policy,
        attestation_kind_requires_provenance_statement(kind.as_str()),
    )?;

    let mut matched_count = 0usize;
    let mut applied_count = 0usize;
    for entry in &mut lock.entries {
        if !lock_entry_matches_package_selector(entry, &package) {
            continue;
        }
        matched_count += 1;
        if let Some(binding) = t3_artifact_binding.as_ref() {
            validate_verified_t3_attestation_artifact_binding(entry, binding)?;
            validate_verified_t3_attestation_statement_binding(
                entry,
                binding,
                attestation_statement.as_ref(),
            )?;
        }
        let report = OmenaSifAttestationVerificationReportV1 {
            schema_version: OMENA_SIF_ATTESTATION_VERIFICATION_REPORT_SCHEMA_VERSION_V1.to_string(),
            product: OMENA_SIF_ATTESTATION_VERIFICATION_REPORT_PRODUCT_V1.to_string(),
            verified: verification_result.success,
            kind: kind.clone(),
            reference: reference.clone(),
            verifier: "sigstore-verify".to_string(),
            verified_trust_tier,
            verified_tlog_integrated_time: verification_result.integrated_time,
            sigstore_verification_policy: Some(OmenaSifSigstoreVerificationPolicyV1 {
                trusted_root: "sigstore-production-trusted-root".to_string(),
                transparency_log: policy.verify_tlog,
                timestamp: policy.verify_timestamp,
                certificate_chain: policy.verify_certificate,
                signed_certificate_timestamp: policy.verify_sct,
            }),
            certificate_issuer: Some(issuer.clone()),
            certificate_identity: identity.clone(),
            attestation_statement: attestation_statement.clone(),
            subject_canonical_url: entry.canonical_url.clone(),
            subject_sif_hash: entry.sif_hash.clone(),
        };
        let applied =
            apply_omena_sif_attestation_verification_report_to_lock_entry_v1(entry, &report)
                .map_err(|error| {
                    format!(
                        "sigstore verification evidence {} rejected for {}: {error}",
                        path_string(&bundle),
                        entry.canonical_url
                    )
                })?;
        if applied {
            applied_count += 1;
        }
    }
    if matched_count == 0 {
        return Err(format!(
            "omena.lock contains no entries for package or canonical URL selector '{package}'"
        ));
    }
    if applied_count == 0 {
        return Err(format!(
            "sigstore verification evidence {} did not match any selected lock entry subject",
            path_string(&bundle)
        ));
    }

    let lock_json = write_omena_lock_json_v1(&lock)
        .map_err(|error| format!("failed to serialize {}: {error}", path_string(&lockfile)))?;
    fs::write(&lockfile, &lock_json)
        .map_err(|error| format!("failed to write {}: {error}", path_string(&lockfile)))?;

    if json {
        print_json(
            CliOutputMetadataV0::new("omena-cli.lock-verify-attestation"),
            &lock,
        )?;
    } else {
        println!(
            "omena.lock sigstore verification updated: {applied_count} entr{} recorded from {}",
            if applied_count == 1 { "y" } else { "ies" },
            path_string(&bundle)
        );
    }

    Ok(())
}

pub(crate) fn extract_verified_attestation_statement(
    bundle: &sigstore_verify::types::Bundle,
    policy: &AttestationStatementPolicy,
    require_provenance_statement: bool,
) -> Result<Option<OmenaSifAttestationStatementV1>, String> {
    let sigstore_verify::types::SignatureContent::DsseEnvelope(envelope) = &bundle.content else {
        if policy.is_empty() && !require_provenance_statement {
            return Ok(None);
        }
        return Err(
            "lock verify-attestation provenance evidence requires a DSSE in-toto provenance bundle"
                .to_string(),
        );
    };
    if envelope.payload_type != "application/vnd.in-toto+json" {
        if policy.is_empty() && !require_provenance_statement {
            return Ok(None);
        }
        return Err(format!(
            "lock verify-attestation provenance evidence requires payloadType application/vnd.in-toto+json, got {}",
            envelope.payload_type
        ));
    }
    let payload = envelope.decode_payload();
    let statement: serde_json::Value = serde_json::from_slice(&payload)
        .map_err(|error| format!("failed to parse verified in-toto statement payload: {error}"))?;
    let statement = summarize_verified_attestation_statement(&statement);
    require_statement_policy_matches(&statement, policy)?;
    Ok(Some(statement))
}

fn attestation_kind_requires_provenance_statement(kind: &str) -> bool {
    kind.starts_with("npm-provenance.") || kind.starts_with("omena-toolchain.")
}

pub(crate) fn summarize_verified_attestation_statement(
    statement: &serde_json::Value,
) -> OmenaSifAttestationStatementV1 {
    let (repository_from_config, ref_from_config) = statement
        .pointer("/predicate/invocation/configSource/uri")
        .and_then(|value| value.as_str())
        .map(split_git_source_uri)
        .unwrap_or((None, None));
    let (repository_from_material, ref_from_material) = statement
        .pointer("/predicate/materials")
        .and_then(|value| value.as_array())
        .and_then(|materials| {
            materials
                .iter()
                .filter_map(|material| material.get("uri").and_then(|value| value.as_str()))
                .map(split_git_source_uri)
                .find(|(repository, _)| repository.is_some())
        })
        .unwrap_or((None, None));
    let source_repository = first_owned_string([
        statement_string(
            statement,
            "/predicate/buildDefinition/externalParameters/workflow/repository",
        ),
        statement_string(
            statement,
            "/predicate/invocation/environment/GITHUB_REPOSITORY",
        )
        .map(|repository| github_repository_url(repository.as_str())),
        repository_from_config,
        repository_from_material,
    ]);
    let source_ref = first_owned_string([
        statement_string(
            statement,
            "/predicate/buildDefinition/externalParameters/workflow/ref",
        ),
        statement_string(statement, "/predicate/invocation/environment/GITHUB_REF"),
        ref_from_config,
        ref_from_material,
    ]);
    let source_commit = first_owned_string([
        statement
            .pointer("/predicate/buildDefinition/resolvedDependencies")
            .and_then(|value| value.as_array())
            .and_then(|dependencies| {
                dependencies.iter().find_map(|dependency| {
                    dependency
                        .pointer("/digest/gitCommit")
                        .and_then(|value| value.as_str())
                        .map(ToOwned::to_owned)
                })
            }),
        statement_string(statement, "/predicate/invocation/configSource/digest/sha1"),
        statement
            .pointer("/predicate/materials")
            .and_then(|value| value.as_array())
            .and_then(|materials| {
                materials.iter().find_map(|material| {
                    material
                        .pointer("/digest/sha1")
                        .and_then(|value| value.as_str())
                        .map(ToOwned::to_owned)
                })
            }),
    ]);
    let subject_digests = statement
        .pointer("/subject")
        .and_then(|value| value.as_array())
        .map(|subjects| {
            let mut result = Vec::new();
            for subject in subjects {
                let Some(name) = subject.get("name").and_then(|value| value.as_str()) else {
                    continue;
                };
                let Some(digests) = subject.get("digest").and_then(|value| value.as_object())
                else {
                    continue;
                };
                for (algorithm, digest) in digests {
                    if let Some(digest) = digest.as_str() {
                        result.push(OmenaSifAttestationSubjectDigestV1 {
                            name: name.to_string(),
                            algorithm: algorithm.to_string(),
                            digest: digest.to_string(),
                        });
                    }
                }
            }
            result
        })
        .unwrap_or_default();

    OmenaSifAttestationStatementV1 {
        statement_type: statement_string(statement, "/_type"),
        predicate_type: statement_string(statement, "/predicateType"),
        source_repository,
        source_ref,
        source_commit,
        builder_id: first_owned_string([
            statement_string(statement, "/predicate/runDetails/builder/id"),
            statement_string(statement, "/predicate/builder/id"),
        ]),
        build_type: first_owned_string([
            statement_string(statement, "/predicate/buildDefinition/buildType"),
            statement_string(statement, "/predicate/buildType"),
        ]),
        subject_names: statement
            .pointer("/subject")
            .and_then(|value| value.as_array())
            .map(|subjects| {
                subjects
                    .iter()
                    .filter_map(|subject| {
                        subject
                            .get("name")
                            .and_then(|value| value.as_str())
                            .map(ToOwned::to_owned)
                    })
                    .collect()
            })
            .unwrap_or_default(),
        subject_digests,
    }
}

fn parse_attestation_statement_subject_digest_policy(
    input: &str,
) -> Result<AttestationStatementSubjectDigestPolicy, String> {
    let input = input.trim();
    let Some((name, digest_with_algorithm)) = input.split_once('=') else {
        return Err(
            "lock verify-attestation --statement-subject-digest must use name=algorithm:digest"
                .to_string(),
        );
    };
    let Some((algorithm, digest)) = digest_with_algorithm.split_once(':') else {
        return Err(
            "lock verify-attestation --statement-subject-digest must use name=algorithm:digest"
                .to_string(),
        );
    };
    let name = name.trim();
    let algorithm = algorithm.trim();
    let digest = digest.trim();
    if name.is_empty() || algorithm.is_empty() || digest.is_empty() {
        return Err(
            "lock verify-attestation --statement-subject-digest must include non-empty name, algorithm, and digest"
                .to_string(),
        );
    }
    Ok(AttestationStatementSubjectDigestPolicy {
        name: name.to_string(),
        algorithm: algorithm.to_string(),
        digest: digest.to_string(),
    })
}

pub(crate) fn require_statement_policy_matches(
    statement: &OmenaSifAttestationStatementV1,
    policy: &AttestationStatementPolicy,
) -> Result<(), String> {
    for (field, flag, expected, observed) in [
        (
            "statementType",
            "--statement-type",
            policy.statement_type.as_ref(),
            statement.statement_type.as_ref(),
        ),
        (
            "predicateType",
            "--statement-predicate-type",
            policy.predicate_type.as_ref(),
            statement.predicate_type.as_ref(),
        ),
        (
            "sourceRepository",
            "--statement-source-repository",
            policy.source_repository.as_ref(),
            statement.source_repository.as_ref(),
        ),
        (
            "sourceRef",
            "--statement-source-ref",
            policy.source_ref.as_ref(),
            statement.source_ref.as_ref(),
        ),
        (
            "sourceCommit",
            "--statement-source-commit",
            policy.source_commit.as_ref(),
            statement.source_commit.as_ref(),
        ),
        (
            "builderId",
            "--statement-builder-id",
            policy.builder_id.as_ref(),
            statement.builder_id.as_ref(),
        ),
        (
            "buildType",
            "--statement-build-type",
            policy.build_type.as_ref(),
            statement.build_type.as_ref(),
        ),
    ] {
        if let Some(expected) = expected {
            if expected.trim().is_empty() {
                return Err(format!("lock verify-attestation {flag} must not be empty"));
            }
            match observed {
                Some(observed) if observed == expected => {}
                Some(observed) => {
                    return Err(format!(
                        "verified attestation statement {field} mismatch: expected {expected}, got {observed}"
                    ));
                }
                None => {
                    return Err(format!(
                        "verified attestation statement missing required {field}: expected {expected}"
                    ));
                }
            }
        }
    }
    for expected in &policy.subject_names {
        if expected.trim().is_empty() {
            return Err(
                "lock verify-attestation --statement-subject-name must not be empty".to_string(),
            );
        }
        if !statement
            .subject_names
            .iter()
            .any(|subject| subject == expected)
        {
            return Err(format!(
                "verified attestation statement missing required subjectName: expected {expected}"
            ));
        }
    }
    for expected in &policy.subject_digests {
        let expected = parse_attestation_statement_subject_digest_policy(expected)?;
        if !statement.subject_digests.iter().any(|observed| {
            observed.name == expected.name
                && observed.algorithm == expected.algorithm
                && observed.digest == expected.digest
        }) {
            return Err(format!(
                "verified attestation statement missing required subjectDigest: expected {}={}:{}",
                expected.name, expected.algorithm, expected.digest
            ));
        }
    }
    Ok(())
}

fn statement_string(statement: &serde_json::Value, pointer: &str) -> Option<String> {
    statement
        .pointer(pointer)
        .and_then(|value| value.as_str())
        .filter(|value| !value.trim().is_empty())
        .map(ToOwned::to_owned)
}

fn first_owned_string(values: impl IntoIterator<Item = Option<String>>) -> Option<String> {
    values
        .into_iter()
        .flatten()
        .find(|value| !value.trim().is_empty())
}

fn split_git_source_uri(uri: &str) -> (Option<String>, Option<String>) {
    let source = uri.strip_prefix("git+").unwrap_or(uri);
    let Some((repository, source_ref)) = source.rsplit_once('@') else {
        return (Some(source.to_string()), None);
    };
    (Some(repository.to_string()), Some(source_ref.to_string()))
}

fn github_repository_url(repository: &str) -> String {
    if repository.starts_with("http://") || repository.starts_with("https://") {
        repository.to_string()
    } else {
        format!("https://github.com/{repository}")
    }
}

pub(crate) struct VerifiedT3AttestationArtifactBinding {
    pub(crate) canonical_url: String,
    pub(crate) sif_hash: omena_sif::OmenaSifDigestV1,
    pub(crate) artifact_sha256: String,
}

fn read_verified_t3_attestation_artifact_binding(
    artifact: &Path,
    artifact_bytes: &[u8],
) -> Result<VerifiedT3AttestationArtifactBinding, String> {
    let artifact_source = std::str::from_utf8(artifact_bytes).map_err(|error| {
        format!(
            "lock verify-attestation --verified-tier t3 requires --artifact to be the SIF JSON for the selected lock entry; {} is not UTF-8 JSON: {error}",
            path_string(artifact)
        )
    })?;
    let sif = read_omena_sif_json_v1(artifact_source).map_err(|error| {
        format!(
            "lock verify-attestation --verified-tier t3 requires --artifact to be the SIF JSON for the selected lock entry; failed to parse {}: {error}",
            path_string(artifact)
        )
    })?;
    let sif_hash = compute_omena_sif_artifact_hash_v1(&sif).map_err(|error| {
        format!(
            "lock verify-attestation --verified-tier t3 failed to hash SIF artifact {}: {error}",
            path_string(artifact)
        )
    })?;
    Ok(VerifiedT3AttestationArtifactBinding {
        canonical_url: sif.canonical_url,
        sif_hash,
        artifact_sha256: sha256_hex(artifact_bytes),
    })
}

pub(crate) fn validate_verified_t3_attestation_artifact_binding(
    entry: &omena_sif::OmenaLockSifEntryV1,
    binding: &VerifiedT3AttestationArtifactBinding,
) -> Result<(), String> {
    if binding.canonical_url != entry.canonical_url {
        return Err(format!(
            "lock verify-attestation --verified-tier t3 requires --artifact to match the selected SIF subject; artifact canonical URL {} does not match lock entry {}",
            binding.canonical_url, entry.canonical_url
        ));
    }
    if binding.sif_hash != entry.sif_hash {
        return Err(format!(
            "lock verify-attestation --verified-tier t3 requires --artifact to match the selected SIF subject; artifact hash {} does not match lock entry {}",
            binding.sif_hash.as_str(),
            entry.sif_hash.as_str()
        ));
    }
    Ok(())
}

pub(crate) fn validate_verified_t3_attestation_statement_binding(
    entry: &omena_sif::OmenaLockSifEntryV1,
    binding: &VerifiedT3AttestationArtifactBinding,
    statement: Option<&OmenaSifAttestationStatementV1>,
) -> Result<(), String> {
    let statement = statement.ok_or_else(|| {
        "lock verify-attestation --verified-tier t3 requires a signed SIF provenance statement"
            .to_string()
    })?;
    if !statement
        .subject_names
        .iter()
        .any(|subject| subject == &entry.sif_path)
    {
        return Err(format!(
            "lock verify-attestation --verified-tier t3 requires the signed provenance statement subjectNames to include selected SIF path {}",
            entry.sif_path
        ));
    }
    if !statement.subject_digests.iter().any(|digest| {
        digest.name == entry.sif_path
            && digest.algorithm.eq_ignore_ascii_case("sha256")
            && digest.digest.eq_ignore_ascii_case(&binding.artifact_sha256)
    }) {
        return Err(format!(
            "lock verify-attestation --verified-tier t3 requires the signed provenance statement subjectDigests to bind {} to sha256:{}",
            entry.sif_path, binding.artifact_sha256
        ));
    }
    Ok(())
}

pub(crate) fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

pub(crate) fn validate_attestation_policy_for_verified_tier(
    kind: &str,
    verified_trust_tier: omena_sif::OmenaSifTrustTierV1,
    certificate_issuer: Option<&str>,
    certificate_identity: Option<&str>,
) -> Result<(), String> {
    if verified_trust_tier == omena_sif::OmenaSifTrustTierV1::T3 {
        if !kind.starts_with("omena-toolchain.") {
            return Err(format!(
                "lock verify-attestation --verified-tier t3 requires --kind omena-toolchain.*, got {kind}"
            ));
        }
        if certificate_issuer.is_none_or(|issuer| issuer.trim().is_empty()) {
            return Err("lock verify-attestation --verified-tier t3 requires --issuer".to_string());
        }
        if certificate_identity.is_none_or(|identity| identity.trim().is_empty()) {
            return Err(
                "lock verify-attestation --verified-tier t3 requires --identity".to_string(),
            );
        }
    }
    Ok(())
}

fn build_lock_entries_from_sif_paths(
    lockfile: &Path,
    sif_paths: &[PathBuf],
) -> Result<Vec<omena_sif::OmenaLockSifEntryV1>, String> {
    let mut entries = Vec::with_capacity(sif_paths.len());
    for sif_path in sif_paths {
        let sif_json = read_source(sif_path)?;
        let sif = read_omena_sif_json_v1(&sif_json)
            .map_err(|error| format!("failed to parse SIF {}: {error}", path_string(sif_path)))?;
        let entry_path = relativize_lock_sif_path(lockfile, sif_path);
        let entry = build_omena_lock_sif_entry_v1(entry_path, &sif).map_err(|error| {
            format!(
                "failed to build lock entry for {}: {error}",
                path_string(sif_path)
            )
        })?;
        entries.push(entry);
    }
    Ok(entries)
}

fn read_lockfile_or_empty(lockfile: &Path) -> Result<OmenaLockV1, String> {
    if !lockfile.exists() {
        return Ok(OmenaLockV1::new(Vec::new()));
    }
    let lockfile_source = read_source(lockfile)?;
    read_omena_lock_json_v1(&lockfile_source)
        .map_err(|error| format!("failed to parse {}: {error}", path_string(lockfile)))
}

fn filter_lock_entries_for_package(
    entries: Vec<omena_sif::OmenaLockSifEntryV1>,
    package: &str,
) -> Result<Vec<omena_sif::OmenaLockSifEntryV1>, String> {
    let filtered = entries
        .into_iter()
        .filter(|entry| lock_entry_matches_package_selector(entry, package))
        .collect::<Vec<_>>();
    if filtered.is_empty() {
        return Err(format!(
            "no provided SIF entries match package or canonical URL selector '{package}'"
        ));
    }
    Ok(filtered)
}

fn lock_entry_matches_package_selector(
    entry: &omena_sif::OmenaLockSifEntryV1,
    selector: &str,
) -> bool {
    let canonical_url = entry
        .canonical_url
        .strip_prefix("pkg:")
        .unwrap_or(&entry.canonical_url);
    let selector = selector.strip_prefix("pkg:").unwrap_or(selector);
    canonical_url == selector || canonical_url.starts_with(&format!("{selector}/"))
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct LockStatusReportV1 {
    product: &'static str,
    lockfile: String,
    present: bool,
    lockfile_version: Option<String>,
    omena_min_version: Option<String>,
    running_omena_version: &'static str,
    entry_count: usize,
    t0_entry_count: usize,
    t1_entry_count: usize,
    t2_entry_count: usize,
    t3_entry_count: usize,
}

fn lock_status(lockfile: PathBuf, json: bool) -> Result<(), String> {
    let report = if lockfile.exists() {
        let lockfile_source = read_source(&lockfile)?;
        let lock = read_omena_lock_json_v1(&lockfile_source)
            .map_err(|error| format!("failed to parse {}: {error}", path_string(&lockfile)))?;
        LockStatusReportV1 {
            product: "omena-cli.lock-status",
            lockfile: path_string(&lockfile),
            present: true,
            lockfile_version: Some(lock.lockfile_version.clone()),
            omena_min_version: lock.omena_min_version.clone(),
            running_omena_version: env!("CARGO_PKG_VERSION"),
            entry_count: lock.entries.len(),
            t0_entry_count: lock
                .entries
                .iter()
                .filter(|entry| entry.trust_tier == omena_sif::OmenaSifTrustTierV1::T0)
                .count(),
            t1_entry_count: lock
                .entries
                .iter()
                .filter(|entry| entry.trust_tier == omena_sif::OmenaSifTrustTierV1::T1)
                .count(),
            t2_entry_count: lock
                .entries
                .iter()
                .filter(|entry| entry.trust_tier == omena_sif::OmenaSifTrustTierV1::T2)
                .count(),
            t3_entry_count: lock
                .entries
                .iter()
                .filter(|entry| entry.trust_tier == omena_sif::OmenaSifTrustTierV1::T3)
                .count(),
        }
    } else {
        LockStatusReportV1 {
            product: "omena-cli.lock-status",
            lockfile: path_string(&lockfile),
            present: false,
            lockfile_version: None,
            omena_min_version: None,
            running_omena_version: env!("CARGO_PKG_VERSION"),
            entry_count: 0,
            t0_entry_count: 0,
            t1_entry_count: 0,
            t2_entry_count: 0,
            t3_entry_count: 0,
        }
    };

    if json {
        print_json(CliOutputMetadataV0::new("omena-cli.lock-status"), &report)?;
    } else if report.present {
        println!(
            "omena.lock status: {} entr{} at {} (requires omena >= {})",
            report.entry_count,
            if report.entry_count == 1 { "y" } else { "ies" },
            report.lockfile,
            report.omena_min_version.as_deref().unwrap_or("unspecified")
        );
    } else {
        println!(
            "omena.lock status: no lockfile found at {}",
            report.lockfile
        );
    }
    Ok(())
}

/// Store `sif_path` so it round-trips through [`resolve_lock_relative_path`]:
/// drop the lockfile's parent prefix when `sif_path` lives under it, otherwise
/// keep the path as authored (already relative to the lockfile location).
fn relativize_lock_sif_path(lockfile: &Path, sif_path: &Path) -> String {
    let parent = lockfile
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty());
    if let Some(parent) = parent
        && let Ok(stripped) = sif_path.strip_prefix(parent)
    {
        return path_string(stripped);
    }
    path_string(sif_path)
}

fn lock_verify(
    lockfile: PathBuf,
    source_paths: Vec<PathBuf>,
    frozen: bool,
    tier: Option<String>,
    json: bool,
) -> Result<(), String> {
    if !frozen {
        return Err("omena lock verify currently requires --frozen".to_string());
    }

    let lockfile_source = read_source(&lockfile)?;
    let lock = read_omena_lock_json_v1(&lockfile_source)
        .map_err(|error| format!("failed to parse {}: {error}", path_string(&lockfile)))?;
    let mut report = verify_omena_lock_frozen_v1(&lock, |entry| {
        let sif_path = resolve_lock_relative_path(&lockfile, &entry.sif_path);
        read_source(&sif_path)
    });
    let source_coverage_issues =
        collect_lock_source_coverage_issues(&lock, source_paths.as_slice())?;
    if !source_coverage_issues.is_empty() {
        report.verified = false;
        report.issues.extend(source_coverage_issues);
    }
    if let Some(minimum_tier) = parse_lock_trust_tier(tier.as_deref())? {
        let tier_issues = collect_lock_trust_tier_issues(&lock, minimum_tier);
        if !tier_issues.is_empty() {
            report.verified = false;
            report.issues.extend(tier_issues);
        }
    }

    if json {
        print_json(CliOutputMetadataV0::new("omena-cli.lock-verify"), &report)?;
    } else if report.verified {
        println!(
            "omena.lock frozen verification passed: {} SIF entr{} checked",
            report.entries_checked,
            if report.entries_checked == 1 {
                "y"
            } else {
                "ies"
            }
        );
    } else {
        println!(
            "omena.lock frozen verification failed: {} issue{}",
            report.issues.len(),
            if report.issues.len() == 1 { "" } else { "s" }
        );
        for issue in &report.issues {
            println!("{} {}: {}", issue.code, issue.sif_path, issue.message);
        }
    }

    if !report.verified {
        return Err("omena.lock frozen verification failed".to_string());
    }
    Ok(())
}

fn parse_lock_trust_tier(
    tier: Option<&str>,
) -> Result<Option<omena_sif::OmenaSifTrustTierV1>, String> {
    let Some(tier) = tier else {
        return Ok(None);
    };
    match tier {
        "t0" | "T0" => Ok(Some(omena_sif::OmenaSifTrustTierV1::T0)),
        "t1" | "T1" => Ok(Some(omena_sif::OmenaSifTrustTierV1::T1)),
        "t2" | "T2" => Ok(Some(omena_sif::OmenaSifTrustTierV1::T2)),
        "t3" | "T3" => Ok(Some(omena_sif::OmenaSifTrustTierV1::T3)),
        _ => Err(format!(
            "unsupported omena.lock trust tier '{tier}'; expected t0, t1, t2, or t3"
        )),
    }
}

pub(crate) fn collect_lock_trust_tier_issues(
    lock: &OmenaLockV1,
    minimum_tier: omena_sif::OmenaSifTrustTierV1,
) -> Vec<OmenaLockVerificationIssueV1> {
    let mut issues = Vec::new();
    for entry in &lock.entries {
        if entry.trust_tier < minimum_tier {
            issues.push(OmenaLockVerificationIssueV1 {
                canonical_url: entry.canonical_url.clone(),
                sif_path: entry.sif_path.clone(),
                code: "trustTierBelowMinimum".to_string(),
                message: format!(
                    "lock entry trust tier {} is below required {}",
                    entry.trust_tier.as_str(),
                    minimum_tier.as_str()
                ),
            });
            continue;
        }
        if minimum_tier >= omena_sif::OmenaSifTrustTierV1::T2 {
            let mut candidate_count = 0usize;
            let mut invalid_reasons = Vec::new();
            let mut has_valid_evidence = false;
            for verification in &entry.attestation_verifications {
                if verification.verified_trust_tier < minimum_tier {
                    continue;
                }
                candidate_count += 1;
                match omena_sif::validate_omena_sif_lock_entry_attestation_verification_v1(
                    entry,
                    verification,
                ) {
                    Ok(()) => {
                        has_valid_evidence = true;
                        break;
                    }
                    Err(error) => invalid_reasons.push(error),
                }
            }
            if has_valid_evidence {
                continue;
            }
            let (code, message) = if candidate_count == 0 {
                (
                    "attestationVerificationMissing",
                    format!(
                        "lock entry trust tier {} must carry verified attestation evidence for required {}",
                        entry.trust_tier.as_str(),
                        minimum_tier.as_str()
                    ),
                )
            } else {
                (
                    "attestationVerificationInvalid",
                    format!(
                        "lock entry trust tier {} has attestation evidence for required {}, but the evidence is invalid: {}",
                        entry.trust_tier.as_str(),
                        minimum_tier.as_str(),
                        invalid_reasons
                            .first()
                            .map_or("unknown validation failure", String::as_str)
                    ),
                )
            };
            issues.push(OmenaLockVerificationIssueV1 {
                canonical_url: entry.canonical_url.clone(),
                sif_path: entry.sif_path.clone(),
                code: code.to_string(),
                message,
            });
        }
    }
    issues
}

pub(crate) fn collect_lock_source_coverage_issues(
    lock: &OmenaLockV1,
    source_paths: &[PathBuf],
) -> Result<Vec<OmenaLockVerificationIssueV1>, String> {
    if source_paths.is_empty() {
        return Ok(Vec::new());
    }

    let locked_sources = lock
        .entries
        .iter()
        .map(|entry| entry.canonical_url.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    let mut missing = std::collections::BTreeSet::new();
    for source_path in source_paths {
        let style_source = read_source(source_path)?;
        let style_path = path_string(source_path);
        let Some(module_sources) =
            summarize_omena_query_sass_module_sources(&style_path, &style_source)
        else {
            continue;
        };
        for module_source in module_sources
            .module_use_edges
            .iter()
            .map(|edge| edge.source.as_str())
            .chain(
                module_sources
                    .module_forward_sources
                    .iter()
                    .map(String::as_str),
            )
        {
            if !lock_source_requires_sif(module_source) {
                continue;
            }
            if locked_sources.contains(module_source) {
                continue;
            }
            missing.insert((module_source.to_string(), style_path.clone()));
        }
    }

    Ok(missing
        .into_iter()
        .map(|(canonical_url, style_path)| OmenaLockVerificationIssueV1 {
            canonical_url: canonical_url.clone(),
            sif_path: style_path.clone(),
            code: "sourceSifMissingFromLock".to_string(),
            message: format!(
                "source {} references external Sass module '{}' but omena.lock has no matching SIF entry",
                style_path, canonical_url
            ),
        })
        .collect())
}

fn lock_source_requires_sif(source: &str) -> bool {
    if source.starts_with("sass:") {
        return false;
    }
    if source.starts_with('.') || source.starts_with('/') || source.starts_with("file://") {
        return false;
    }
    true
}

pub(crate) fn resolve_lock_relative_path(lockfile: &Path, entry_path: &str) -> PathBuf {
    let entry_path = Path::new(entry_path);
    if entry_path.is_absolute() {
        return entry_path.to_path_buf();
    }
    lockfile
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(entry_path)
}
