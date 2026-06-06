//! Sass Interface File v1 contracts for Omena CSS.
//!
//! SIF v1 is the local, deterministic artifact that lets resolver and
//! diagnostic layers stop treating external Sass modules as blind
//! `externalIgnored` gaps. This crate is deliberately pure data plus hashing:
//! it performs no Sass evaluation, package execution, filesystem traversal, or
//! network access.

use serde::{Deserialize, Serialize};
use serde_json::Value;

mod generator;

pub use generator::*;

pub const OMENA_SIF_VERSION_V1: &str = "1";
pub const OMENA_LOCK_CURRENT_MIN_VERSION_V1: &str = env!("CARGO_PKG_VERSION");
pub const OMENA_SIF_HASH_ALGORITHM_V1: &str = "blake3";
pub const OMENA_SIF_ATTESTATION_VERIFICATION_REPORT_PRODUCT_V1: &str =
    "omena-sif.attestation-verification-report";
pub const OMENA_SIF_ATTESTATION_VERIFICATION_REPORT_SCHEMA_VERSION_V1: &str = "1";
pub const OMENA_SIF_V1_SCHEMA_JSON: &str = include_str!("../schema/sif-v1.schema.json");
pub const OMENA_LOCK_V1_SCHEMA_JSON: &str = include_str!("../schema/lock-v1.schema.json");
pub const OMENA_SIF_ATTESTATION_VERIFICATION_REPORT_V1_SCHEMA_JSON: &str =
    include_str!("../schema/attestation-verification-report-v1.schema.json");

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct OmenaSifDigestV1(String);

impl OmenaSifDigestV1 {
    pub fn as_str(&self) -> &str {
        &self.0
    }

    fn from_blake3_bytes(bytes: &[u8]) -> Self {
        let hash = blake3::hash(bytes);
        Self(format!("{OMENA_SIF_HASH_ALGORITHM_V1}:{}", hash.to_hex()))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaSifV1 {
    pub sif_version: String,
    pub canonical_url: String,
    pub generator: OmenaSifGeneratorV1,
    pub source: OmenaSifSourceV1,
    pub exports: OmenaSifExportsV1,
    pub dependencies: Vec<OmenaSifDependencyInterfaceHashV1>,
    pub fingerprints: OmenaSifFingerprintChainV1,
}

impl OmenaSifV1 {
    pub fn from_static_exports(
        canonical_url: impl Into<String>,
        generator: OmenaSifGeneratorV1,
        source: OmenaSifSourceV1,
        exports: OmenaSifExportsV1,
        dependencies: Vec<OmenaSifDependencyInterfaceHashV1>,
        source_bytes: &[u8],
    ) -> Result<Self, serde_json::Error> {
        let fingerprints = compute_omena_sif_fingerprint_chain_v1(
            source_bytes,
            generator.toolchain_id.as_str(),
            &exports,
            &dependencies,
        )?;
        Ok(Self {
            sif_version: OMENA_SIF_VERSION_V1.to_string(),
            canonical_url: canonical_url.into(),
            generator,
            source,
            exports,
            dependencies: sorted_omena_sif_dependencies_v1(dependencies),
            fingerprints,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaSifGeneratorV1 {
    pub name: String,
    pub version: String,
    pub toolchain_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaSifSourceV1 {
    pub syntax: OmenaSifSourceSyntaxV1,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum OmenaSifSourceSyntaxV1 {
    Css,
    Scss,
    Sass,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaSifExportsV1 {
    pub variables: Vec<OmenaSifVariableExportV1>,
    pub mixins: Vec<OmenaSifCallableExportV1>,
    pub functions: Vec<OmenaSifCallableExportV1>,
    pub placeholders: Vec<OmenaSifPlaceholderExportV1>,
    pub forwards: Vec<OmenaSifForwardExportV1>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaSifVariableExportV1 {
    pub name: String,
    pub defaulted: bool,
    pub value_repr: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaSifCallableExportV1 {
    pub name: String,
    pub parameters: Vec<OmenaSifParameterV1>,
    pub accepts_content: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaSifParameterV1 {
    pub name: String,
    pub default_value_repr: Option<String>,
    pub variadic: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaSifPlaceholderExportV1 {
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaSifForwardExportV1 {
    pub canonical_url: String,
    pub prefix: Option<String>,
    pub show: Vec<String>,
    pub hide: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaSifDependencyInterfaceHashV1 {
    pub canonical_url: String,
    pub interface_hash: OmenaSifDigestV1,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaSifFingerprintChainV1 {
    pub hash_algorithm: String,
    pub leaf_hash: OmenaSifDigestV1,
    pub interface_hash: OmenaSifDigestV1,
    pub transitive_hash: OmenaSifDigestV1,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaLockV1 {
    pub lockfile_version: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub omena_min_version: Option<String>,
    pub entries: Vec<OmenaLockSifEntryV1>,
}

impl OmenaLockV1 {
    pub fn new(entries: Vec<OmenaLockSifEntryV1>) -> Self {
        Self::new_with_min_version(entries, Some(OMENA_LOCK_CURRENT_MIN_VERSION_V1.to_string()))
    }

    pub fn new_with_min_version(
        entries: Vec<OmenaLockSifEntryV1>,
        omena_min_version: Option<String>,
    ) -> Self {
        Self {
            lockfile_version: OMENA_SIF_VERSION_V1.to_string(),
            omena_min_version,
            entries: sorted_omena_lock_entries_v1(entries),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaLockSifEntryV1 {
    pub canonical_url: String,
    pub sif_path: String,
    pub sif_hash: OmenaSifDigestV1,
    pub interface_hash: OmenaSifDigestV1,
    pub transitive_hash: OmenaSifDigestV1,
    #[serde(default = "default_omena_sif_trust_tier_v1")]
    pub trust_tier: OmenaSifTrustTierV1,
    #[serde(default)]
    pub attestation_references: Vec<OmenaSifAttestationReferenceV1>,
    #[serde(default)]
    pub attestation_verifications: Vec<OmenaSifAttestationVerificationV1>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum OmenaSifTrustTierV1 {
    #[serde(rename = "t0")]
    T0,
    #[default]
    #[serde(rename = "t1")]
    T1,
    #[serde(rename = "t2")]
    T2,
    #[serde(rename = "t3")]
    T3,
}

impl OmenaSifTrustTierV1 {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::T0 => "t0",
            Self::T1 => "t1",
            Self::T2 => "t2",
            Self::T3 => "t3",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaSifAttestationReferenceV1 {
    pub kind: String,
    pub reference: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaSifAttestationVerificationV1 {
    pub kind: String,
    pub reference: String,
    pub verifier: String,
    pub verified_trust_tier: OmenaSifTrustTierV1,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub verified_tlog_integrated_time: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sigstore_verification_policy: Option<OmenaSifSigstoreVerificationPolicyV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub certificate_issuer: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub certificate_identity: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaSifSigstoreVerificationPolicyV1 {
    pub trusted_root: String,
    pub transparency_log: bool,
    pub timestamp: bool,
    pub certificate_chain: bool,
    pub signed_certificate_timestamp: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaSifAttestationVerificationReportV1 {
    pub schema_version: String,
    pub product: String,
    pub verified: bool,
    pub kind: String,
    pub reference: String,
    pub verifier: String,
    pub verified_trust_tier: OmenaSifTrustTierV1,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub verified_tlog_integrated_time: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sigstore_verification_policy: Option<OmenaSifSigstoreVerificationPolicyV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub certificate_issuer: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub certificate_identity: Option<String>,
    pub subject_canonical_url: String,
    pub subject_sif_hash: OmenaSifDigestV1,
}

pub fn read_omena_sif_attestation_verification_report_json_v1(
    source: &str,
) -> Result<OmenaSifAttestationVerificationReportV1, serde_json::Error> {
    serde_json::from_str(source)
}

pub fn apply_omena_sif_attestation_verification_report_to_lock_entry_v1(
    entry: &mut OmenaLockSifEntryV1,
    report: &OmenaSifAttestationVerificationReportV1,
) -> Result<bool, String> {
    validate_omena_sif_attestation_verification_report_v1(report)?;
    if !report.verified {
        return Err("attestation verification report is not marked verified".to_string());
    }
    if report.verified_trust_tier < OmenaSifTrustTierV1::T2 {
        return Err(format!(
            "attestation verification report tier {} cannot satisfy enforced provenance",
            report.verified_trust_tier.as_str()
        ));
    }
    if entry.canonical_url != report.subject_canonical_url {
        return Ok(false);
    }
    if entry.sif_hash != report.subject_sif_hash {
        return Err(format!(
            "attestation verification report subject hash {} does not match lock entry {}",
            report.subject_sif_hash.as_str(),
            entry.sif_hash.as_str()
        ));
    }
    if !lock_entry_has_compatible_attestation_reference_v1(
        entry,
        report.kind.as_str(),
        report.reference.as_str(),
    ) {
        return Err(format!(
            "attestation verification report reference '{}' was not recorded with a compatible kind for lock entry {}",
            report.reference, entry.canonical_url
        ));
    }

    let verification = OmenaSifAttestationVerificationV1 {
        kind: report.kind.clone(),
        reference: report.reference.clone(),
        verifier: report.verifier.clone(),
        verified_trust_tier: report.verified_trust_tier,
        verified_tlog_integrated_time: report.verified_tlog_integrated_time,
        sigstore_verification_policy: report.sigstore_verification_policy.clone(),
        certificate_issuer: report.certificate_issuer.clone(),
        certificate_identity: report.certificate_identity.clone(),
    };
    let mut verifications = entry
        .attestation_verifications
        .iter()
        .cloned()
        .collect::<std::collections::BTreeSet<_>>();
    verifications.insert(verification);
    entry.attestation_verifications = verifications.into_iter().collect();
    if entry.trust_tier < report.verified_trust_tier {
        entry.trust_tier = report.verified_trust_tier;
    }
    Ok(true)
}

fn validate_omena_sif_attestation_verification_report_v1(
    report: &OmenaSifAttestationVerificationReportV1,
) -> Result<(), String> {
    if report.schema_version != OMENA_SIF_ATTESTATION_VERIFICATION_REPORT_SCHEMA_VERSION_V1 {
        return Err(format!(
            "unsupported attestation verification report schemaVersion '{}'; expected {}",
            report.schema_version, OMENA_SIF_ATTESTATION_VERIFICATION_REPORT_SCHEMA_VERSION_V1
        ));
    }
    if report.product != OMENA_SIF_ATTESTATION_VERIFICATION_REPORT_PRODUCT_V1 {
        return Err(format!(
            "unsupported attestation verification report product '{}'; expected {}",
            report.product, OMENA_SIF_ATTESTATION_VERIFICATION_REPORT_PRODUCT_V1
        ));
    }
    for (field, value) in [
        ("kind", report.kind.as_str()),
        ("reference", report.reference.as_str()),
        ("verifier", report.verifier.as_str()),
        ("subjectCanonicalUrl", report.subject_canonical_url.as_str()),
    ] {
        if value.trim().is_empty() {
            return Err(format!(
                "attestation verification report field {field} must not be empty"
            ));
        }
    }
    for (field, value) in [
        ("certificateIssuer", report.certificate_issuer.as_ref()),
        ("certificateIdentity", report.certificate_identity.as_ref()),
    ] {
        if value.is_some_and(|value| value.trim().is_empty()) {
            return Err(format!(
                "attestation verification report field {field} must not be empty when present"
            ));
        }
    }
    if report
        .verified_tlog_integrated_time
        .is_some_and(|integrated_time| integrated_time <= 0)
    {
        return Err(
            "attestation verification report field verifiedTlogIntegratedTime must be positive when present"
                .to_string(),
        );
    }
    if let Some(policy) = report.sigstore_verification_policy.as_ref() {
        validate_sigstore_verification_policy_v1(policy)?;
    }
    validate_attestation_verification_for_trust_tier_v1(&OmenaSifAttestationVerificationV1 {
        kind: report.kind.clone(),
        reference: report.reference.clone(),
        verifier: report.verifier.clone(),
        verified_trust_tier: report.verified_trust_tier,
        verified_tlog_integrated_time: report.verified_tlog_integrated_time,
        sigstore_verification_policy: report.sigstore_verification_policy.clone(),
        certificate_issuer: report.certificate_issuer.clone(),
        certificate_identity: report.certificate_identity.clone(),
    })?;
    Ok(())
}

fn validate_sigstore_verification_policy_v1(
    policy: &OmenaSifSigstoreVerificationPolicyV1,
) -> Result<(), String> {
    if policy.trusted_root.trim().is_empty() {
        return Err(
            "attestation verification report sigstoreVerificationPolicy.trustedRoot must not be empty"
                .to_string(),
        );
    }
    if !policy.transparency_log
        || !policy.timestamp
        || !policy.certificate_chain
        || !policy.signed_certificate_timestamp
    {
        return Err(
            "attestation verification report sigstoreVerificationPolicy must require transparency log, timestamp, certificate chain, and signed certificate timestamp verification"
                .to_string(),
        );
    }
    Ok(())
}

fn validate_attestation_verification_for_trust_tier_v1(
    verification: &OmenaSifAttestationVerificationV1,
) -> Result<(), String> {
    for (field, value) in [
        ("kind", verification.kind.as_str()),
        ("reference", verification.reference.as_str()),
        ("verifier", verification.verifier.as_str()),
    ] {
        if value.trim().is_empty() {
            return Err(format!(
                "attestation verification field {field} must not be empty"
            ));
        }
    }
    if let Some(policy) = verification.sigstore_verification_policy.as_ref() {
        validate_sigstore_verification_policy_v1(policy)?;
    } else if verification.verifier == "sigstore-verify" {
        return Err(
            "attestation verification from sigstore-verify requires sigstoreVerificationPolicy"
                .to_string(),
        );
    }
    for (field, value) in [
        (
            "certificateIssuer",
            verification.certificate_issuer.as_ref(),
        ),
        (
            "certificateIdentity",
            verification.certificate_identity.as_ref(),
        ),
    ] {
        if value.is_some_and(|value| value.trim().is_empty()) {
            return Err(format!(
                "attestation verification field {field} must not be empty when present"
            ));
        }
    }
    if verification.verifier == "sigstore-verify"
        && verification
            .certificate_issuer
            .as_deref()
            .is_none_or(|issuer| issuer.trim().is_empty())
    {
        return Err(
            "attestation verification from sigstore-verify requires certificateIssuer".to_string(),
        );
    }
    if verification.verifier == "sigstore-verify"
        && verification.verified_tlog_integrated_time.is_none()
    {
        return Err(
            "attestation verification from sigstore-verify requires verifiedTlogIntegratedTime"
                .to_string(),
        );
    }
    if verification.verified_trust_tier == OmenaSifTrustTierV1::T3 {
        if !verification.kind.starts_with("omena-toolchain.") {
            return Err(format!(
                "attestation verification tier t3 requires kind omena-toolchain.*, got {}",
                verification.kind
            ));
        }
        if verification
            .certificate_issuer
            .as_deref()
            .is_none_or(|issuer| issuer.trim().is_empty())
        {
            return Err("attestation verification tier t3 requires certificateIssuer".to_string());
        }
        if verification
            .certificate_identity
            .as_deref()
            .is_none_or(|identity| identity.trim().is_empty())
        {
            return Err(
                "attestation verification tier t3 requires certificateIdentity".to_string(),
            );
        }
    }
    Ok(())
}

pub fn validate_omena_sif_attestation_verification_v1(
    verification: &OmenaSifAttestationVerificationV1,
) -> Result<(), String> {
    validate_attestation_verification_for_trust_tier_v1(verification)
}

pub fn validate_omena_sif_lock_entry_attestation_verification_v1(
    entry: &OmenaLockSifEntryV1,
    verification: &OmenaSifAttestationVerificationV1,
) -> Result<(), String> {
    validate_attestation_verification_for_trust_tier_v1(verification)?;
    if lock_entry_has_compatible_attestation_reference_v1(
        entry,
        verification.kind.as_str(),
        verification.reference.as_str(),
    ) {
        Ok(())
    } else {
        Err(format!(
            "attestation verification reference '{}' is not backed by a compatible attestation reference",
            verification.reference
        ))
    }
}

fn lock_entry_has_compatible_attestation_reference_v1(
    entry: &OmenaLockSifEntryV1,
    verification_kind: &str,
    verification_reference: &str,
) -> bool {
    entry.attestation_references.iter().any(|reference| {
        reference.reference == verification_reference
            && attestation_reference_kind_satisfies_verification_kind_v1(
                reference.kind.as_str(),
                verification_kind,
            )
    })
}

fn attestation_reference_kind_satisfies_verification_kind_v1(
    reference_kind: &str,
    verification_kind: &str,
) -> bool {
    if reference_kind == verification_kind {
        return true;
    }
    if verification_kind.starts_with("npm-provenance.") {
        return matches!(reference_kind, "npm-provenance" | "npm-provenance.url");
    }
    if verification_kind.starts_with("omena-toolchain.") {
        return reference_kind == "sigstore-bundle"
            || reference_kind.starts_with("omena-toolchain.");
    }
    false
}

pub fn collect_omena_sif_npm_provenance_attestation_references_v1(
    source: &str,
) -> Result<Vec<OmenaSifAttestationReferenceV1>, serde_json::Error> {
    let value = serde_json::from_str::<Value>(source)?;
    let mut references = std::collections::BTreeSet::new();
    collect_npm_metadata_provenance_references_v1(&value, &mut references);
    Ok(references.into_iter().collect())
}

pub fn apply_omena_sif_npm_provenance_references_to_lock_entry_v1(
    entry: &mut OmenaLockSifEntryV1,
    references: &[OmenaSifAttestationReferenceV1],
) -> usize {
    let mut existing = entry
        .attestation_references
        .iter()
        .cloned()
        .collect::<std::collections::BTreeSet<_>>();
    let before = existing.len();
    existing.extend(references.iter().cloned());
    entry.attestation_references = existing.into_iter().collect();
    entry.attestation_references.len().saturating_sub(before)
}

pub fn omena_lock_entry_has_verified_attestation_for_tier_v1(
    entry: &OmenaLockSifEntryV1,
    minimum_tier: OmenaSifTrustTierV1,
) -> bool {
    minimum_tier <= OmenaSifTrustTierV1::T1
        || entry.attestation_verifications.iter().any(|verification| {
            verification.verified_trust_tier >= minimum_tier
                && validate_omena_sif_lock_entry_attestation_verification_v1(entry, verification)
                    .is_ok()
        })
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaSifProvenanceAdvisoryReportV1 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub enforcement: &'static str,
    pub network_access: &'static str,
    pub entries: Vec<OmenaSifProvenanceAdvisoryEntryV1>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaSifProvenanceAdvisoryEntryV1 {
    pub canonical_url: String,
    pub trust_tier: OmenaSifTrustTierV1,
    pub attestation_reference_count: usize,
    pub attestation_verification_count: usize,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub attestation_verification_policies: Vec<OmenaSifAttestationVerificationPolicyV1>,
    pub advisory_message: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaSifAttestationVerificationPolicyV1 {
    pub kind: String,
    pub reference: String,
    pub verifier: String,
    pub verified_trust_tier: OmenaSifTrustTierV1,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verified_tlog_integrated_time: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sigstore_verification_policy: Option<OmenaSifSigstoreVerificationPolicyV1>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub certificate_issuer: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub certificate_identity: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaLockVerificationReportV1 {
    pub lockfile_version: String,
    pub frozen: bool,
    pub verified: bool,
    pub entries_checked: usize,
    pub issues: Vec<OmenaLockVerificationIssueV1>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaLockVerificationIssueV1 {
    pub canonical_url: String,
    pub sif_path: String,
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
struct OmenaSifInterfaceHashInputV1<'a> {
    sif_version: &'a str,
    toolchain_id: &'a str,
    exports: &'a OmenaSifExportsV1,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
struct OmenaSifTransitiveHashInputV1<'a> {
    sif_version: &'a str,
    interface_hash: &'a OmenaSifDigestV1,
    dependencies: Vec<OmenaSifDependencyInterfaceHashV1>,
}

pub fn compute_omena_sif_leaf_hash_v1(source_bytes: &[u8]) -> OmenaSifDigestV1 {
    OmenaSifDigestV1::from_blake3_bytes(source_bytes)
}

pub fn compute_omena_sif_interface_hash_v1(
    toolchain_id: &str,
    exports: &OmenaSifExportsV1,
) -> Result<OmenaSifDigestV1, serde_json::Error> {
    let input = OmenaSifInterfaceHashInputV1 {
        sif_version: OMENA_SIF_VERSION_V1,
        toolchain_id,
        exports,
    };
    let canonical_bytes = write_omena_canonical_json_bytes_v1(&input)?;
    Ok(OmenaSifDigestV1::from_blake3_bytes(&canonical_bytes))
}

pub fn compute_omena_sif_transitive_hash_v1(
    interface_hash: &OmenaSifDigestV1,
    dependencies: &[OmenaSifDependencyInterfaceHashV1],
) -> Result<OmenaSifDigestV1, serde_json::Error> {
    let input = OmenaSifTransitiveHashInputV1 {
        sif_version: OMENA_SIF_VERSION_V1,
        interface_hash,
        dependencies: sorted_omena_sif_dependencies_v1(dependencies.to_vec()),
    };
    let canonical_bytes = write_omena_canonical_json_bytes_v1(&input)?;
    Ok(OmenaSifDigestV1::from_blake3_bytes(&canonical_bytes))
}

pub fn compute_omena_sif_fingerprint_chain_v1(
    source_bytes: &[u8],
    toolchain_id: &str,
    exports: &OmenaSifExportsV1,
    dependencies: &[OmenaSifDependencyInterfaceHashV1],
) -> Result<OmenaSifFingerprintChainV1, serde_json::Error> {
    let leaf_hash = compute_omena_sif_leaf_hash_v1(source_bytes);
    let interface_hash = compute_omena_sif_interface_hash_v1(toolchain_id, exports)?;
    let transitive_hash = compute_omena_sif_transitive_hash_v1(&interface_hash, dependencies)?;

    Ok(OmenaSifFingerprintChainV1 {
        hash_algorithm: OMENA_SIF_HASH_ALGORITHM_V1.to_string(),
        leaf_hash,
        interface_hash,
        transitive_hash,
    })
}

pub fn compute_omena_sif_artifact_hash_v1(
    sif: &OmenaSifV1,
) -> Result<OmenaSifDigestV1, serde_json::Error> {
    let canonical_bytes = write_omena_sif_json_v1(sif)?.into_bytes();
    Ok(OmenaSifDigestV1::from_blake3_bytes(&canonical_bytes))
}

pub fn build_omena_lock_sif_entry_v1(
    sif_path: impl Into<String>,
    sif: &OmenaSifV1,
) -> Result<OmenaLockSifEntryV1, serde_json::Error> {
    Ok(OmenaLockSifEntryV1 {
        canonical_url: sif.canonical_url.clone(),
        sif_path: sif_path.into(),
        sif_hash: compute_omena_sif_artifact_hash_v1(sif)?,
        interface_hash: sif.fingerprints.interface_hash.clone(),
        transitive_hash: sif.fingerprints.transitive_hash.clone(),
        trust_tier: OmenaSifTrustTierV1::T1,
        attestation_references: Vec::new(),
        attestation_verifications: Vec::new(),
    })
}

pub fn summarize_omena_sif_provenance_advisory_v1(
    lock: &OmenaLockV1,
) -> OmenaSifProvenanceAdvisoryReportV1 {
    OmenaSifProvenanceAdvisoryReportV1 {
        schema_version: "0",
        product: "omena-sif.provenance-advisory",
        enforcement: provenance_advisory_enforcement(lock),
        network_access: "none",
        entries: lock
            .entries
            .iter()
            .map(|entry| OmenaSifProvenanceAdvisoryEntryV1 {
                canonical_url: entry.canonical_url.clone(),
                trust_tier: entry.trust_tier,
                attestation_reference_count: entry.attestation_references.len(),
                attestation_verification_count: entry.attestation_verifications.len(),
                attestation_verification_policies: summarize_attestation_verification_policies(
                    entry,
                ),
                advisory_message: provenance_advisory_message(entry),
            })
            .collect(),
    }
}

fn summarize_attestation_verification_policies(
    entry: &OmenaLockSifEntryV1,
) -> Vec<OmenaSifAttestationVerificationPolicyV1> {
    entry
        .attestation_verifications
        .iter()
        .map(|verification| OmenaSifAttestationVerificationPolicyV1 {
            kind: verification.kind.clone(),
            reference: verification.reference.clone(),
            verifier: verification.verifier.clone(),
            verified_trust_tier: verification.verified_trust_tier,
            verified_tlog_integrated_time: verification.verified_tlog_integrated_time,
            sigstore_verification_policy: verification.sigstore_verification_policy.clone(),
            certificate_issuer: verification.certificate_issuer.clone(),
            certificate_identity: verification.certificate_identity.clone(),
        })
        .collect()
}

fn provenance_advisory_enforcement(lock: &OmenaLockV1) -> &'static str {
    if lock
        .entries
        .iter()
        .any(|entry| !entry.attestation_verifications.is_empty())
    {
        "lockVerifyTier2Tier3WhenRequested"
    } else {
        "referenceOnlyAdvisory"
    }
}

pub fn read_omena_lock_json_v1(source: &str) -> Result<OmenaLockV1, serde_json::Error> {
    serde_json::from_str(source)
}

pub fn write_omena_lock_json_v1(lock: &OmenaLockV1) -> Result<String, serde_json::Error> {
    write_omena_canonical_json_string_v1(&OmenaLockV1::new_with_min_version(
        lock.entries.clone(),
        lock.omena_min_version.clone(),
    ))
}

pub fn verify_omena_lock_frozen_v1<F>(
    lock: &OmenaLockV1,
    load_sif_json: F,
) -> OmenaLockVerificationReportV1
where
    F: FnMut(&OmenaLockSifEntryV1) -> Result<String, String>,
{
    verify_omena_lock_frozen_with_runtime_version_v1(
        lock,
        OMENA_LOCK_CURRENT_MIN_VERSION_V1,
        load_sif_json,
    )
}

pub fn verify_omena_lock_frozen_with_runtime_version_v1<F>(
    lock: &OmenaLockV1,
    runtime_version: &str,
    mut load_sif_json: F,
) -> OmenaLockVerificationReportV1
where
    F: FnMut(&OmenaLockSifEntryV1) -> Result<String, String>,
{
    let mut issues = Vec::new();

    if let Some(required_version) = lock.omena_min_version.as_deref() {
        match compare_omena_semver_core_v1(runtime_version, required_version) {
            Some(std::cmp::Ordering::Less) => issues.push(OmenaLockVerificationIssueV1 {
                canonical_url: "omena.lock".to_string(),
                sif_path: "omena.lock".to_string(),
                code: "omenaMinVersionUnsupported".to_string(),
                message: format!(
                    "omena.lock requires omena >= {required_version}, but the running binary is {runtime_version}"
                ),
            }),
            Some(_) => {}
            None if runtime_version != required_version => issues.push(OmenaLockVerificationIssueV1 {
                canonical_url: "omena.lock".to_string(),
                sif_path: "omena.lock".to_string(),
                code: "omenaMinVersionUnparseable".to_string(),
                message: format!(
                    "omena.lock omenaMinVersion '{required_version}' cannot be compared with running binary version '{runtime_version}'"
                ),
            }),
            None => {}
        }
    }

    for entry in &lock.entries {
        let sif_json = match load_sif_json(entry) {
            Ok(sif_json) => sif_json,
            Err(error) => {
                push_omena_lock_issue_v1(&mut issues, entry, "loadFailed", error);
                continue;
            }
        };
        let sif = match read_omena_sif_json_v1(&sif_json) {
            Ok(sif) => sif,
            Err(error) => {
                push_omena_lock_issue_v1(
                    &mut issues,
                    entry,
                    "parseFailed",
                    format!("failed to parse SIF JSON: {error}"),
                );
                continue;
            }
        };
        let actual_sif_hash = match compute_omena_sif_artifact_hash_v1(&sif) {
            Ok(hash) => hash,
            Err(error) => {
                push_omena_lock_issue_v1(
                    &mut issues,
                    entry,
                    "hashFailed",
                    format!("failed to hash canonical SIF JSON: {error}"),
                );
                continue;
            }
        };

        if sif.canonical_url != entry.canonical_url {
            push_omena_lock_issue_v1(
                &mut issues,
                entry,
                "canonicalUrlMismatch",
                format!(
                    "lock expected {}, SIF declared {}",
                    entry.canonical_url, sif.canonical_url
                ),
            );
        }
        if actual_sif_hash != entry.sif_hash {
            push_omena_lock_issue_v1(
                &mut issues,
                entry,
                "sifHashMismatch",
                format!(
                    "lock expected {}, SIF canonical artifact hash is {}",
                    entry.sif_hash.as_str(),
                    actual_sif_hash.as_str()
                ),
            );
        }
        if sif.fingerprints.interface_hash != entry.interface_hash {
            push_omena_lock_issue_v1(
                &mut issues,
                entry,
                "interfaceHashMismatch",
                format!(
                    "lock expected {}, SIF interface hash is {}",
                    entry.interface_hash.as_str(),
                    sif.fingerprints.interface_hash.as_str()
                ),
            );
        }
        if sif.fingerprints.transitive_hash != entry.transitive_hash {
            push_omena_lock_issue_v1(
                &mut issues,
                entry,
                "transitiveHashMismatch",
                format!(
                    "lock expected {}, SIF transitive hash is {}",
                    entry.transitive_hash.as_str(),
                    sif.fingerprints.transitive_hash.as_str()
                ),
            );
        }
    }

    OmenaLockVerificationReportV1 {
        lockfile_version: lock.lockfile_version.clone(),
        frozen: true,
        verified: issues.is_empty(),
        entries_checked: lock.entries.len(),
        issues,
    }
}

fn compare_omena_semver_core_v1(left: &str, right: &str) -> Option<std::cmp::Ordering> {
    let left = parse_omena_semver_core_v1(left)?;
    let right = parse_omena_semver_core_v1(right)?;
    Some(left.cmp(&right))
}

fn parse_omena_semver_core_v1(version: &str) -> Option<(u64, u64, u64)> {
    let core = version.split(['-', '+']).next().unwrap_or(version);
    let mut parts = core.split('.');
    let major = parts.next()?.parse::<u64>().ok()?;
    let minor = parts.next().unwrap_or("0").parse::<u64>().ok()?;
    let patch = parts.next().unwrap_or("0").parse::<u64>().ok()?;
    if parts.next().is_some() {
        return None;
    }
    Some((major, minor, patch))
}

pub fn read_omena_sif_json_v1(source: &str) -> Result<OmenaSifV1, serde_json::Error> {
    serde_json::from_str(source)
}

pub fn write_omena_sif_json_v1(sif: &OmenaSifV1) -> Result<String, serde_json::Error> {
    write_omena_canonical_json_string_v1(sif)
}

pub fn write_omena_canonical_json_bytes_v1<T: Serialize>(
    value: &T,
) -> Result<Vec<u8>, serde_json::Error> {
    Ok(write_omena_canonical_json_string_v1(value)?.into_bytes())
}

pub fn write_omena_canonical_json_string_v1<T: Serialize>(
    value: &T,
) -> Result<String, serde_json::Error> {
    let value = serde_json::to_value(value)?;
    let mut output = String::new();
    write_canonical_json_value_v1(&value, &mut output)?;
    Ok(output)
}

fn sorted_omena_sif_dependencies_v1(
    mut dependencies: Vec<OmenaSifDependencyInterfaceHashV1>,
) -> Vec<OmenaSifDependencyInterfaceHashV1> {
    dependencies.sort_by(|left, right| {
        left.canonical_url
            .cmp(&right.canonical_url)
            .then(left.interface_hash.cmp(&right.interface_hash))
    });
    dependencies
}

fn sorted_omena_lock_entries_v1(mut entries: Vec<OmenaLockSifEntryV1>) -> Vec<OmenaLockSifEntryV1> {
    entries.sort_by(|left, right| {
        left.canonical_url
            .cmp(&right.canonical_url)
            .then(left.sif_path.cmp(&right.sif_path))
    });
    entries
}

fn default_omena_sif_trust_tier_v1() -> OmenaSifTrustTierV1 {
    OmenaSifTrustTierV1::T1
}

fn provenance_advisory_message(entry: &OmenaLockSifEntryV1) -> &'static str {
    match entry.trust_tier {
        OmenaSifTrustTierV1::T0 => {
            "No enforced provenance verification is available for this SIF entry."
        }
        OmenaSifTrustTierV1::T1 => "T1 local lock verification is the enforced trust path.",
        OmenaSifTrustTierV1::T2 | OmenaSifTrustTierV1::T3
            if entry.attestation_verifications.is_empty() =>
        {
            "T2/T3 trust tiers require verified attestation evidence; references alone are advisory."
        }
        OmenaSifTrustTierV1::T2 | OmenaSifTrustTierV1::T3 => {
            "Verified attestation evidence is recorded for this trust tier."
        }
    }
}

fn collect_npm_metadata_provenance_references_v1(
    value: &Value,
    references: &mut std::collections::BTreeSet<OmenaSifAttestationReferenceV1>,
) {
    if let Some(provenance) = value.pointer("/dist/attestations/provenance") {
        collect_npm_provenance_value_references_v1(provenance, references);
    }
    if let Some(provenance) = value.pointer("/attestations/provenance") {
        collect_npm_provenance_value_references_v1(provenance, references);
    }
    if let Some(versions) = value.get("versions").and_then(Value::as_object) {
        for version in versions.values() {
            collect_npm_metadata_provenance_references_v1(version, references);
        }
    }
}

fn collect_npm_provenance_value_references_v1(
    value: &Value,
    references: &mut std::collections::BTreeSet<OmenaSifAttestationReferenceV1>,
) {
    match value {
        Value::String(reference) if !reference.trim().is_empty() => {
            references.insert(OmenaSifAttestationReferenceV1 {
                kind: "npm-provenance".to_string(),
                reference: reference.trim().to_string(),
            });
        }
        Value::Array(values) => {
            for value in values {
                collect_npm_provenance_value_references_v1(value, references);
            }
        }
        Value::Object(object) => {
            let before = references.len();
            for key in ["url", "uri", "reference", "bundle", "provenance"] {
                if let Some(reference) = object.get(key).and_then(Value::as_str)
                    && !reference.trim().is_empty()
                {
                    references.insert(OmenaSifAttestationReferenceV1 {
                        kind: format!("npm-provenance.{key}"),
                        reference: reference.trim().to_string(),
                    });
                }
            }
            if references.len() == before
                && let Ok(reference) = serde_json::to_string(value)
            {
                references.insert(OmenaSifAttestationReferenceV1 {
                    kind: "npm-provenance.object".to_string(),
                    reference,
                });
            }
        }
        _ => {}
    }
}

fn push_omena_lock_issue_v1(
    issues: &mut Vec<OmenaLockVerificationIssueV1>,
    entry: &OmenaLockSifEntryV1,
    code: &str,
    message: String,
) {
    issues.push(OmenaLockVerificationIssueV1 {
        canonical_url: entry.canonical_url.clone(),
        sif_path: entry.sif_path.clone(),
        code: code.to_string(),
        message,
    });
}

fn write_canonical_json_value_v1(
    value: &Value,
    output: &mut String,
) -> Result<(), serde_json::Error> {
    match value {
        Value::Null => output.push_str("null"),
        Value::Bool(value) => output.push_str(if *value { "true" } else { "false" }),
        Value::Number(value) => output.push_str(&value.to_string()),
        Value::String(value) => output.push_str(&serde_json::to_string(value)?),
        Value::Array(values) => {
            output.push('[');
            for (index, value) in values.iter().enumerate() {
                if index > 0 {
                    output.push(',');
                }
                write_canonical_json_value_v1(value, output)?;
            }
            output.push(']');
        }
        Value::Object(map) => {
            output.push('{');
            let mut entries: Vec<_> = map.iter().collect();
            entries.sort_by_key(|(key, _)| *key);
            for (index, (key, value)) in entries.into_iter().enumerate() {
                if index > 0 {
                    output.push(',');
                }
                output.push_str(&serde_json::to_string(key)?);
                output.push(':');
                write_canonical_json_value_v1(value, output)?;
            }
            output.push('}');
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::collections::BTreeMap;

    #[test]
    fn schema_file_is_valid_json() -> Result<(), serde_json::Error> {
        let schema: Value = serde_json::from_str(OMENA_SIF_V1_SCHEMA_JSON)?;
        assert_eq!(
            schema.get("title").and_then(Value::as_str),
            Some("Omena Sass Interface File v1")
        );
        Ok(())
    }

    #[test]
    fn lock_schema_file_is_valid_json() -> Result<(), String> {
        let schema: Value =
            serde_json::from_str(OMENA_LOCK_V1_SCHEMA_JSON).map_err(|error| error.to_string())?;
        assert_eq!(
            schema.get("title").and_then(Value::as_str),
            Some("Omena Lockfile v1")
        );
        let verification_properties = schema
            .pointer("/$defs/attestationVerification/properties")
            .and_then(Value::as_object)
            .ok_or_else(|| {
                "lock schema must define attestation verification properties".to_string()
            })?;
        for field in [
            "verifiedTlogIntegratedTime",
            "sigstoreVerificationPolicy",
            "certificateIssuer",
            "certificateIdentity",
        ] {
            assert!(
                verification_properties.contains_key(field),
                "lock schema must preserve attestation verification field {field}"
            );
        }
        let all_of = schema
            .pointer("/$defs/attestationVerification/allOf")
            .and_then(Value::as_array)
            .ok_or_else(|| {
                "lock schema must encode conditional verification requirements".to_string()
            })?;
        assert!(
            all_of.iter().any(|rule| {
                rule.pointer("/if/properties/verifier/const")
                    .and_then(Value::as_str)
                    == Some("sigstore-verify")
                    && rule
                        .pointer("/then/required")
                        .and_then(Value::as_array)
                        .is_some_and(|required| {
                            required
                                .contains(&Value::String("sigstoreVerificationPolicy".to_string()))
                                && required
                                    .contains(&Value::String("certificateIssuer".to_string()))
                                && required.contains(&Value::String(
                                    "verifiedTlogIntegratedTime".to_string(),
                                ))
                        })
            }),
            "lock schema sigstore-verify evidence must require policy, issuer, and log time"
        );
        Ok(())
    }

    #[test]
    fn attestation_verification_report_schema_file_is_valid_json() -> Result<(), String> {
        let schema: Value =
            serde_json::from_str(OMENA_SIF_ATTESTATION_VERIFICATION_REPORT_V1_SCHEMA_JSON)
                .map_err(|error| error.to_string())?;
        assert_eq!(
            schema.get("title").and_then(Value::as_str),
            Some("Omena SIF Attestation Verification Report v1")
        );
        assert_eq!(
            schema.pointer("/properties/product/const"),
            Some(&Value::String(
                OMENA_SIF_ATTESTATION_VERIFICATION_REPORT_PRODUCT_V1.to_string()
            ))
        );
        assert_eq!(
            schema.pointer("/properties/schemaVersion/const"),
            Some(&Value::String(
                OMENA_SIF_ATTESTATION_VERIFICATION_REPORT_SCHEMA_VERSION_V1.to_string()
            ))
        );
        let all_of = schema
            .pointer("/allOf")
            .and_then(Value::as_array)
            .ok_or_else(|| {
                "attestation report schema must encode conditional policy requirements".to_string()
            })?;
        assert!(
            all_of.iter().any(|rule| {
                rule.pointer("/if/properties/verifier/const")
                    .and_then(Value::as_str)
                    == Some("sigstore-verify")
                    && rule
                        .pointer("/then/required")
                        .and_then(Value::as_array)
                        .is_some_and(|required| {
                            required
                                .contains(&Value::String("sigstoreVerificationPolicy".to_string()))
                                && required
                                    .contains(&Value::String("certificateIssuer".to_string()))
                                && required.contains(&Value::String(
                                    "verifiedTlogIntegratedTime".to_string(),
                                ))
                        })
            }),
            "sigstore-verify reports must require policy, issuer, and log time"
        );
        assert!(
            all_of.iter().any(|rule| {
                rule.pointer("/if/properties/verifiedTrustTier/const")
                    .and_then(Value::as_str)
                    == Some("t3")
                    && rule
                        .pointer("/then/properties/kind/pattern")
                        .and_then(Value::as_str)
                        == Some("^omena-toolchain\\.")
            }),
            "t3 reports must require omena-toolchain evidence"
        );
        Ok(())
    }

    #[test]
    fn canonical_json_sorts_object_keys_recursively() -> Result<(), serde_json::Error> {
        let value = json!({
            "z": 1,
            "a": {
                "d": true,
                "b": ["x", { "y": null, "c": 2 }]
            }
        });

        assert_eq!(
            write_omena_canonical_json_string_v1(&value)?,
            r#"{"a":{"b":["x",{"c":2,"y":null}],"d":true},"z":1}"#
        );
        Ok(())
    }

    #[test]
    fn leaf_hash_tracks_source_bytes_with_algorithm_tag() {
        let first = compute_omena_sif_leaf_hash_v1(b"$color: red;");
        let second = compute_omena_sif_leaf_hash_v1(b"$color: blue;");

        assert_ne!(first, second);
        assert!(first.as_str().starts_with("blake3:"));
        assert_eq!(first.as_str().len(), "blake3:".len() + 64);
    }

    #[test]
    fn interface_hash_ignores_source_bytes_and_urls() -> Result<(), serde_json::Error> {
        let exports = fixture_exports();
        let first = compute_omena_sif_interface_hash_v1("omena-sifgen@0.1", &exports)?;
        let second = OmenaSifV1::from_static_exports(
            "pkg:a/_tokens.scss",
            fixture_generator(),
            fixture_source(),
            exports.clone(),
            Vec::new(),
            b"$color: red;",
        )?
        .fingerprints
        .interface_hash;
        let third = OmenaSifV1::from_static_exports(
            "pkg:b/_tokens.scss",
            fixture_generator(),
            fixture_source(),
            exports,
            Vec::new(),
            b"$color: blue;",
        )?
        .fingerprints
        .interface_hash;

        assert_eq!(first, second);
        assert_eq!(second, third);
        Ok(())
    }

    #[test]
    fn transitive_hash_sorts_dependencies_before_hashing() -> Result<(), serde_json::Error> {
        let interface_hash = OmenaSifDigestV1::from_blake3_bytes(b"self-interface");
        let first = vec![
            OmenaSifDependencyInterfaceHashV1 {
                canonical_url: "pkg:z/_index.scss".to_string(),
                interface_hash: OmenaSifDigestV1::from_blake3_bytes(b"z"),
            },
            OmenaSifDependencyInterfaceHashV1 {
                canonical_url: "pkg:a/_index.scss".to_string(),
                interface_hash: OmenaSifDigestV1::from_blake3_bytes(b"a"),
            },
        ];
        let second = vec![first[1].clone(), first[0].clone()];

        assert_eq!(
            compute_omena_sif_transitive_hash_v1(&interface_hash, &first)?,
            compute_omena_sif_transitive_hash_v1(&interface_hash, &second)?
        );
        Ok(())
    }

    #[test]
    fn sif_json_round_trips_through_canonical_writer() -> Result<(), serde_json::Error> {
        let sif = fixture_sif("pkg:design-system/_tokens.scss", b"$color: red !default;")?;

        let json = write_omena_sif_json_v1(&sif)?;
        let decoded = read_omena_sif_json_v1(&json)?;

        assert_eq!(decoded, sif);
        assert!(json.starts_with(r#"{"canonicalUrl":"pkg:design-system/_tokens.scss","#));
        Ok(())
    }

    #[test]
    fn sif_artifact_hash_tracks_canonical_sif_json() -> Result<(), serde_json::Error> {
        let first = fixture_sif("pkg:design-system/_tokens.scss", b"$color: red !default;")?;
        let mut second = first.clone();
        second.source.syntax = OmenaSifSourceSyntaxV1::Css;

        assert_ne!(
            compute_omena_sif_artifact_hash_v1(&first)?,
            compute_omena_sif_artifact_hash_v1(&second)?
        );
        Ok(())
    }

    #[test]
    fn lock_json_writer_sorts_entries_deterministically() -> Result<(), serde_json::Error> {
        let first = fixture_sif("pkg:z/_tokens.scss", b"$z: red;")?;
        let second = fixture_sif("pkg:a/_tokens.scss", b"$a: red;")?;
        let lock = OmenaLockV1 {
            lockfile_version: OMENA_SIF_VERSION_V1.to_string(),
            omena_min_version: Some("0.2.0".to_string()),
            entries: vec![
                build_omena_lock_sif_entry_v1("sif/z.sif.json", &first)?,
                build_omena_lock_sif_entry_v1("sif/a.sif.json", &second)?,
            ],
        };

        let json = write_omena_lock_json_v1(&lock)?;
        assert!(json.contains(r#""lockfileVersion":"1""#));
        assert!(json.contains(r#""omenaMinVersion":"0.2.0""#));
        assert!(
            json.find("pkg:a/_tokens.scss") < json.find("pkg:z/_tokens.scss"),
            "{json}"
        );
        Ok(())
    }

    #[test]
    fn lock_entry_defaults_old_json_to_t1_without_attestations() -> Result<(), serde_json::Error> {
        let digest = "blake3:0000000000000000000000000000000000000000000000000000000000000000";
        let lock_json = format!(
            r#"{{
                "lockfileVersion": "1",
                "entries": [{{
                    "canonicalUrl": "pkg:design-system/_tokens.scss",
                    "sifPath": "sif/design-system.sif.json",
                    "sifHash": "{digest}",
                    "interfaceHash": "{digest}",
                    "transitiveHash": "{digest}"
                }}]
            }}"#
        );

        let lock = read_omena_lock_json_v1(&lock_json)?;
        assert!(lock.omena_min_version.is_none());
        assert_eq!(lock.entries[0].trust_tier, OmenaSifTrustTierV1::T1);
        assert!(lock.entries[0].attestation_references.is_empty());
        assert!(lock.entries[0].attestation_verifications.is_empty());
        Ok(())
    }

    #[test]
    fn lock_frozen_verification_fails_when_min_version_exceeds_runtime()
    -> Result<(), serde_json::Error> {
        let sif = fixture_sif("pkg:design-system/_tokens.scss", b"$color: red !default;")?;
        let sif_json = write_omena_sif_json_v1(&sif)?;
        let lock = OmenaLockV1::new_with_min_version(
            vec![build_omena_lock_sif_entry_v1(
                "sif/design-system.sif.json",
                &sif,
            )?],
            Some("999.0.0".to_string()),
        );

        let report = verify_omena_lock_frozen_with_runtime_version_v1(&lock, "0.2.0", |_entry| {
            Ok(sif_json.clone())
        });

        assert!(!report.verified, "{report:?}");
        assert!(
            report
                .issues
                .iter()
                .any(|issue| issue.code == "omenaMinVersionUnsupported"),
            "{report:?}"
        );
        Ok(())
    }

    #[test]
    fn provenance_advisory_report_marks_reference_only_lock_as_advisory()
    -> Result<(), serde_json::Error> {
        let sif = fixture_sif("pkg:design-system/_tokens.scss", b"$color: red !default;")?;
        let mut entry = build_omena_lock_sif_entry_v1("sif/design-system.sif.json", &sif)?;
        entry.trust_tier = OmenaSifTrustTierV1::T3;
        entry
            .attestation_references
            .push(OmenaSifAttestationReferenceV1 {
                kind: "sigstore-bundle".to_string(),
                reference: "sif/design-system.sigstore.json".to_string(),
            });
        let lock = OmenaLockV1::new(vec![entry]);

        let report = summarize_omena_sif_provenance_advisory_v1(&lock);

        assert_eq!(report.enforcement, "referenceOnlyAdvisory");
        assert_eq!(report.network_access, "none");
        assert_eq!(report.entries[0].trust_tier, OmenaSifTrustTierV1::T3);
        assert_eq!(report.entries[0].attestation_reference_count, 1);
        assert_eq!(report.entries[0].attestation_verification_count, 0);
        assert_eq!(
            report.entries[0].advisory_message,
            "T2/T3 trust tiers require verified attestation evidence; references alone are advisory."
        );
        Ok(())
    }

    #[test]
    fn provenance_advisory_report_marks_verified_evidence_as_enforceable()
    -> Result<(), serde_json::Error> {
        let sif = fixture_sif("pkg:design-system/_tokens.scss", b"$color: red !default;")?;
        let mut entry = build_omena_lock_sif_entry_v1("sif/design-system.sif.json", &sif)?;
        entry.trust_tier = OmenaSifTrustTierV1::T2;
        let attestation_reference =
            "https://registry.npmjs.org/-/npm/v1/attestations/design-system@1.0.0/provenance"
                .to_string();
        entry
            .attestation_references
            .push(OmenaSifAttestationReferenceV1 {
                kind: "npm-provenance.url".to_string(),
                reference: attestation_reference.clone(),
            });
        entry
            .attestation_verifications
            .push(OmenaSifAttestationVerificationV1 {
                kind: "npm-provenance.sigstore".to_string(),
                reference: attestation_reference,
                verifier: "offline-sigstore-verifier".to_string(),
                verified_trust_tier: OmenaSifTrustTierV1::T2,
                verified_tlog_integrated_time: Some(1_717_000_000),
                sigstore_verification_policy: Some(OmenaSifSigstoreVerificationPolicyV1 {
                    trusted_root: "sigstore-production-trusted-root".to_string(),
                    transparency_log: true,
                    timestamp: true,
                    certificate_chain: true,
                    signed_certificate_timestamp: true,
                }),
                certificate_issuer: Some("https://token.actions.githubusercontent.com".to_string()),
                certificate_identity: Some(
                    "https://github.com/omenien/omena-css/.github/workflows/release.yml@refs/tags/v1.0.0"
                        .to_string(),
                ),
            });
        let lock = OmenaLockV1::new(vec![entry]);

        let report = summarize_omena_sif_provenance_advisory_v1(&lock);

        assert_eq!(report.enforcement, "lockVerifyTier2Tier3WhenRequested");
        assert_eq!(
            report.entries[0].attestation_verification_policies[0].verified_tlog_integrated_time,
            Some(1_717_000_000)
        );
        assert_eq!(
            report.entries[0].attestation_verification_policies[0]
                .sigstore_verification_policy
                .as_ref()
                .map(|policy| policy.trusted_root.as_str()),
            Some("sigstore-production-trusted-root")
        );
        assert_eq!(report.entries[0].attestation_verification_count, 1);
        assert_eq!(report.entries[0].attestation_verification_policies.len(), 1);
        assert_eq!(
            report.entries[0].attestation_verification_policies[0]
                .certificate_issuer
                .as_deref(),
            Some("https://token.actions.githubusercontent.com")
        );
        assert_eq!(
            report.entries[0].attestation_verification_policies[0]
                .certificate_identity
                .as_deref(),
            Some(
                "https://github.com/omenien/omena-css/.github/workflows/release.yml@refs/tags/v1.0.0"
            )
        );
        assert_eq!(
            report.entries[0].advisory_message,
            "Verified attestation evidence is recorded for this trust tier."
        );
        Ok(())
    }

    #[test]
    fn npm_provenance_metadata_references_do_not_upgrade_without_verification()
    -> Result<(), serde_json::Error> {
        let metadata = json!({
            "name": "design-system",
            "version": "1.0.0",
            "dist": {
                "attestations": {
                    "provenance": [
                        "https://registry.npmjs.org/-/npm/v1/attestations/design-system@1.0.0/provenance",
                        {"url": "https://registry.npmjs.org/-/npm/v1/attestations/design-system@1.0.0/bundle"}
                    ]
                }
            }
        });
        let references = collect_omena_sif_npm_provenance_attestation_references_v1(
            &serde_json::to_string(&metadata)?,
        )?;
        assert_eq!(references.len(), 2, "{references:?}");

        let sif = fixture_sif("pkg:design-system/_tokens.scss", b"$color: red !default;")?;
        let mut entry = build_omena_lock_sif_entry_v1("sif/design-system.sif.json", &sif)?;
        let added =
            apply_omena_sif_npm_provenance_references_to_lock_entry_v1(&mut entry, &references);

        assert_eq!(added, 2);
        assert_eq!(entry.trust_tier, OmenaSifTrustTierV1::T1);
        assert_eq!(entry.attestation_references.len(), 2);
        assert!(entry.attestation_verifications.is_empty());
        Ok(())
    }

    #[test]
    fn verified_attestation_evidence_controls_trust_tier_gate() -> Result<(), serde_json::Error> {
        let sif = fixture_sif("pkg:design-system/_tokens.scss", b"$color: red !default;")?;
        let mut entry = build_omena_lock_sif_entry_v1("sif/design-system.sif.json", &sif)?;
        entry.trust_tier = OmenaSifTrustTierV1::T2;
        let attestation_reference =
            "https://registry.npmjs.org/-/npm/v1/attestations/design-system@1.0.0/provenance"
                .to_string();
        entry
            .attestation_references
            .push(OmenaSifAttestationReferenceV1 {
                kind: "npm-provenance.url".to_string(),
                reference: attestation_reference.clone(),
            });

        assert!(!omena_lock_entry_has_verified_attestation_for_tier_v1(
            &entry,
            OmenaSifTrustTierV1::T2
        ));

        entry
            .attestation_verifications
            .push(OmenaSifAttestationVerificationV1 {
                kind: "npm-provenance.sigstore".to_string(),
                reference: attestation_reference.clone(),
                verifier: "omena-sif-test-fixture".to_string(),
                verified_trust_tier: OmenaSifTrustTierV1::T2,
                verified_tlog_integrated_time: None,
                sigstore_verification_policy: None,
                certificate_issuer: None,
                certificate_identity: None,
            });

        assert!(omena_lock_entry_has_verified_attestation_for_tier_v1(
            &entry,
            OmenaSifTrustTierV1::T2
        ));
        assert!(!omena_lock_entry_has_verified_attestation_for_tier_v1(
            &entry,
            OmenaSifTrustTierV1::T3
        ));

        let mut sigstore_entry = build_omena_lock_sif_entry_v1("sif/design-system.sif.json", &sif)?;
        sigstore_entry.trust_tier = OmenaSifTrustTierV1::T2;
        sigstore_entry
            .attestation_references
            .push(OmenaSifAttestationReferenceV1 {
                kind: "npm-provenance.url".to_string(),
                reference: attestation_reference.clone(),
            });
        sigstore_entry
            .attestation_verifications
            .push(OmenaSifAttestationVerificationV1 {
                kind: "npm-provenance.sigstore".to_string(),
                reference: attestation_reference.clone(),
                verifier: "sigstore-verify".to_string(),
                verified_trust_tier: OmenaSifTrustTierV1::T2,
                verified_tlog_integrated_time: Some(1_717_000_000),
                sigstore_verification_policy: None,
                certificate_issuer: Some("https://github.com/login/oauth".to_string()),
                certificate_identity: None,
            });
        assert!(!omena_lock_entry_has_verified_attestation_for_tier_v1(
            &sigstore_entry,
            OmenaSifTrustTierV1::T2
        ));
        sigstore_entry.attestation_verifications[0].sigstore_verification_policy =
            Some(OmenaSifSigstoreVerificationPolicyV1 {
                trusted_root: "sigstore-production-trusted-root".to_string(),
                transparency_log: true,
                timestamp: true,
                certificate_chain: true,
                signed_certificate_timestamp: true,
            });
        sigstore_entry.attestation_verifications[0].certificate_issuer = None;
        assert!(!omena_lock_entry_has_verified_attestation_for_tier_v1(
            &sigstore_entry,
            OmenaSifTrustTierV1::T2
        ));
        sigstore_entry.attestation_verifications[0].certificate_issuer =
            Some("https://github.com/login/oauth".to_string());
        assert!(omena_lock_entry_has_verified_attestation_for_tier_v1(
            &sigstore_entry,
            OmenaSifTrustTierV1::T2
        ));

        entry.trust_tier = OmenaSifTrustTierV1::T3;
        entry
            .attestation_verifications
            .push(OmenaSifAttestationVerificationV1 {
                kind: "omena-toolchain.sigstore".to_string(),
                reference: attestation_reference,
                verifier: "omena-sif-test-fixture".to_string(),
                verified_trust_tier: OmenaSifTrustTierV1::T3,
                verified_tlog_integrated_time: None,
                sigstore_verification_policy: None,
                certificate_issuer: Some("https://github.com/login/oauth".to_string()),
                certificate_identity: None,
            });
        assert!(!omena_lock_entry_has_verified_attestation_for_tier_v1(
            &entry,
            OmenaSifTrustTierV1::T3
        ));

        let mut toolchain_entry =
            build_omena_lock_sif_entry_v1("sif/design-system.sif.json", &sif)?;
        toolchain_entry.trust_tier = OmenaSifTrustTierV1::T3;
        toolchain_entry
            .attestation_references
            .push(OmenaSifAttestationReferenceV1 {
                kind: "sigstore-bundle".to_string(),
                reference: "sif/design-system.sigstore.json".to_string(),
            });
        toolchain_entry
            .attestation_verifications
            .push(OmenaSifAttestationVerificationV1 {
                kind: "omena-toolchain.sigstore".to_string(),
                reference: "sif/design-system.sigstore.json".to_string(),
                verifier: "omena-sif-test-fixture".to_string(),
                verified_trust_tier: OmenaSifTrustTierV1::T3,
                verified_tlog_integrated_time: None,
                sigstore_verification_policy: None,
                certificate_issuer: Some("https://github.com/login/oauth".to_string()),
                certificate_identity: Some("https://github.com/omenien/omena-css/.github/workflows/sif-keyless-attestation.yml@refs/heads/master".to_string()),
            });
        assert!(omena_lock_entry_has_verified_attestation_for_tier_v1(
            &toolchain_entry,
            OmenaSifTrustTierV1::T3
        ));
        Ok(())
    }

    #[test]
    fn verified_attestation_report_upgrades_matching_lock_entry() -> Result<(), String> {
        let sif = fixture_sif("pkg:design-system/_tokens.scss", b"$color: red !default;")
            .map_err(|error| error.to_string())?;
        let mut entry = build_omena_lock_sif_entry_v1("sif/design-system.sif.json", &sif)
            .map_err(|error| error.to_string())?;
        let provenance_reference =
            "https://registry.npmjs.org/-/npm/v1/attestations/design-system@1.0.0/provenance";
        entry
            .attestation_references
            .push(OmenaSifAttestationReferenceV1 {
                kind: "npm-provenance.url".to_string(),
                reference: provenance_reference.to_string(),
            });
        let report = read_omena_sif_attestation_verification_report_json_v1(
            &json!({
                "schemaVersion": "1",
                "product": "omena-sif.attestation-verification-report",
                "verified": true,
                "kind": "npm-provenance.sigstore",
                "reference": "https://registry.npmjs.org/-/npm/v1/attestations/design-system@1.0.0/provenance",
                "verifier": "offline-sigstore-verifier",
                "verifiedTrustTier": "t2",
                "subjectCanonicalUrl": entry.canonical_url.as_str(),
                "subjectSifHash": entry.sif_hash.as_str()
            })
            .to_string(),
        )
        .map_err(|error| error.to_string())?;

        let applied =
            apply_omena_sif_attestation_verification_report_to_lock_entry_v1(&mut entry, &report)?;

        assert!(applied);
        assert_eq!(entry.trust_tier, OmenaSifTrustTierV1::T2);
        assert_eq!(entry.attestation_verifications.len(), 1);
        assert_eq!(entry.attestation_verifications[0].certificate_issuer, None);
        assert_eq!(
            entry.attestation_verifications[0].certificate_identity,
            None
        );
        assert!(omena_lock_entry_has_verified_attestation_for_tier_v1(
            &entry,
            OmenaSifTrustTierV1::T2
        ));
        Ok(())
    }

    #[test]
    fn verified_attestation_report_rejects_unrecorded_reference() -> Result<(), String> {
        let sif = fixture_sif("pkg:design-system/_tokens.scss", b"$color: red !default;")
            .map_err(|error| error.to_string())?;
        let mut entry = build_omena_lock_sif_entry_v1("sif/design-system.sif.json", &sif)
            .map_err(|error| error.to_string())?;
        let report = read_omena_sif_attestation_verification_report_json_v1(
            &json!({
                "schemaVersion": "1",
                "product": "omena-sif.attestation-verification-report",
                "verified": true,
                "kind": "npm-provenance.sigstore",
                "reference": "https://registry.npmjs.org/-/npm/v1/attestations/design-system@1.0.0/provenance",
                "verifier": "offline-sigstore-verifier",
                "verifiedTrustTier": "t2",
                "subjectCanonicalUrl": entry.canonical_url.as_str(),
                "subjectSifHash": entry.sif_hash.as_str()
            })
            .to_string(),
        )
        .map_err(|error| error.to_string())?;

        let result =
            apply_omena_sif_attestation_verification_report_to_lock_entry_v1(&mut entry, &report);

        assert!(
            matches!(result.as_ref(), Err(message) if message.contains("was not recorded")),
            "{result:?}"
        );
        assert!(entry.attestation_verifications.is_empty());
        Ok(())
    }

    #[test]
    fn verified_attestation_report_t3_requires_omena_toolchain_kind() -> Result<(), String> {
        let sif = fixture_sif("pkg:design-system/_tokens.scss", b"$color: red !default;")
            .map_err(|error| error.to_string())?;
        let mut entry = build_omena_lock_sif_entry_v1("sif/design-system.sif.json", &sif)
            .map_err(|error| error.to_string())?;
        let provenance_reference =
            "https://registry.npmjs.org/-/npm/v1/attestations/design-system@1.0.0/provenance";
        entry
            .attestation_references
            .push(OmenaSifAttestationReferenceV1 {
                kind: "npm-provenance.url".to_string(),
                reference: provenance_reference.to_string(),
            });
        let report = read_omena_sif_attestation_verification_report_json_v1(
            &json!({
                "schemaVersion": "1",
                "product": "omena-sif.attestation-verification-report",
                "verified": true,
                "kind": "npm-provenance.sigstore",
                "reference": provenance_reference,
                "verifier": "offline-sigstore-verifier",
                "verifiedTrustTier": "t3",
                "subjectCanonicalUrl": entry.canonical_url.as_str(),
                "subjectSifHash": entry.sif_hash.as_str()
            })
            .to_string(),
        )
        .map_err(|error| error.to_string())?;

        let result =
            apply_omena_sif_attestation_verification_report_to_lock_entry_v1(&mut entry, &report);

        assert!(
            matches!(
                result.as_ref(),
                Err(message) if message.contains("tier t3 requires kind omena-toolchain.*")
            ),
            "{result:?}"
        );
        assert!(entry.attestation_verifications.is_empty());
        assert_eq!(entry.trust_tier, OmenaSifTrustTierV1::T1);
        Ok(())
    }

    #[test]
    fn verified_attestation_report_t3_requires_certificate_identity() -> Result<(), String> {
        let sif = fixture_sif("pkg:design-system/_tokens.scss", b"$color: red !default;")
            .map_err(|error| error.to_string())?;
        let mut entry = build_omena_lock_sif_entry_v1("sif/design-system.sif.json", &sif)
            .map_err(|error| error.to_string())?;
        let provenance_reference = "sif/design-system.sigstore.json";
        entry
            .attestation_references
            .push(OmenaSifAttestationReferenceV1 {
                kind: "sigstore-bundle".to_string(),
                reference: provenance_reference.to_string(),
            });
        let report = read_omena_sif_attestation_verification_report_json_v1(
            &json!({
                "schemaVersion": "1",
                "product": "omena-sif.attestation-verification-report",
                "verified": true,
                "kind": "omena-toolchain.sigstore",
                "reference": provenance_reference,
                "verifier": "offline-sigstore-verifier",
                "verifiedTrustTier": "t3",
                "certificateIssuer": "https://github.com/login/oauth",
                "subjectCanonicalUrl": entry.canonical_url.as_str(),
                "subjectSifHash": entry.sif_hash.as_str()
            })
            .to_string(),
        )
        .map_err(|error| error.to_string())?;

        let result =
            apply_omena_sif_attestation_verification_report_to_lock_entry_v1(&mut entry, &report);

        assert!(
            matches!(
                result.as_ref(),
                Err(message) if message.contains("tier t3 requires certificateIdentity")
            ),
            "{result:?}"
        );
        assert!(entry.attestation_verifications.is_empty());
        assert_eq!(entry.trust_tier, OmenaSifTrustTierV1::T1);
        Ok(())
    }

    #[test]
    fn verified_attestation_report_rejects_incompatible_reference_kind() -> Result<(), String> {
        let sif = fixture_sif("pkg:design-system/_tokens.scss", b"$color: red !default;")
            .map_err(|error| error.to_string())?;
        let mut entry = build_omena_lock_sif_entry_v1("sif/design-system.sif.json", &sif)
            .map_err(|error| error.to_string())?;
        let provenance_reference =
            "https://registry.npmjs.org/-/npm/v1/attestations/design-system@1.0.0/provenance";
        entry
            .attestation_references
            .push(OmenaSifAttestationReferenceV1 {
                kind: "npm-provenance.url".to_string(),
                reference: provenance_reference.to_string(),
            });
        let report = read_omena_sif_attestation_verification_report_json_v1(
            &json!({
                "schemaVersion": "1",
                "product": "omena-sif.attestation-verification-report",
                "verified": true,
                "kind": "omena-toolchain.sigstore",
                "reference": provenance_reference,
                "verifier": "offline-sigstore-verifier",
                "verifiedTrustTier": "t3",
                "certificateIssuer": "https://github.com/login/oauth",
                "certificateIdentity": "https://github.com/omenien/omena-css/.github/workflows/sif-keyless-attestation.yml@refs/heads/master",
                "subjectCanonicalUrl": entry.canonical_url.as_str(),
                "subjectSifHash": entry.sif_hash.as_str()
            })
            .to_string(),
        )
        .map_err(|error| error.to_string())?;

        let result =
            apply_omena_sif_attestation_verification_report_to_lock_entry_v1(&mut entry, &report);

        assert!(
            matches!(
                result.as_ref(),
                Err(message) if message.contains("compatible kind")
            ),
            "{result:?}"
        );
        assert!(entry.attestation_verifications.is_empty());
        assert_eq!(entry.trust_tier, OmenaSifTrustTierV1::T1);
        Ok(())
    }

    #[test]
    fn verified_attestation_report_rejects_unverified_or_mismatched_subject() -> Result<(), String>
    {
        let sif = fixture_sif("pkg:design-system/_tokens.scss", b"$color: red !default;")
            .map_err(|error| error.to_string())?;
        let mut entry = build_omena_lock_sif_entry_v1("sif/design-system.sif.json", &sif)
            .map_err(|error| error.to_string())?;
        let changed_sif = fixture_sif("pkg:design-system/_tokens.scss", b"$color: blue !default;")
            .map_err(|error| error.to_string())?;
        let changed_entry =
            build_omena_lock_sif_entry_v1("sif/design-system.changed.sif.json", &changed_sif)
                .map_err(|error| error.to_string())?;
        let mut report = OmenaSifAttestationVerificationReportV1 {
            schema_version: "1".to_string(),
            product: "omena-sif.attestation-verification-report".to_string(),
            verified: true,
            kind: "npm-provenance.sigstore".to_string(),
            reference:
                "https://registry.npmjs.org/-/npm/v1/attestations/design-system@1.0.0/provenance"
                    .to_string(),
            verifier: "offline-sigstore-verifier".to_string(),
            verified_trust_tier: OmenaSifTrustTierV1::T2,
            verified_tlog_integrated_time: None,
            sigstore_verification_policy: None,
            certificate_issuer: None,
            certificate_identity: None,
            subject_canonical_url: entry.canonical_url.clone(),
            subject_sif_hash: changed_entry.sif_hash,
        };

        let hash_mismatch =
            apply_omena_sif_attestation_verification_report_to_lock_entry_v1(&mut entry, &report);
        assert!(hash_mismatch.is_err(), "{hash_mismatch:?}");

        report.subject_canonical_url = "pkg:other/_tokens.scss".to_string();
        let unmatched =
            apply_omena_sif_attestation_verification_report_to_lock_entry_v1(&mut entry, &report)?;
        assert!(!unmatched);

        report.subject_canonical_url = entry.canonical_url.clone();
        report.subject_sif_hash = entry.sif_hash.clone();
        report.verified = false;
        let unverified =
            apply_omena_sif_attestation_verification_report_to_lock_entry_v1(&mut entry, &report);
        assert!(unverified.is_err(), "{unverified:?}");
        Ok(())
    }

    #[test]
    fn verified_attestation_report_rejects_wrong_contract_or_empty_evidence() -> Result<(), String>
    {
        let sif = fixture_sif("pkg:design-system/_tokens.scss", b"$color: red !default;")
            .map_err(|error| error.to_string())?;
        let mut entry = build_omena_lock_sif_entry_v1("sif/design-system.sif.json", &sif)
            .map_err(|error| error.to_string())?;
        let mut report = OmenaSifAttestationVerificationReportV1 {
            schema_version: "0".to_string(),
            product: "omena-sif.attestation-verification-report".to_string(),
            verified: true,
            kind: "npm-provenance.sigstore".to_string(),
            reference:
                "https://registry.npmjs.org/-/npm/v1/attestations/design-system@1.0.0/provenance"
                    .to_string(),
            verifier: "offline-sigstore-verifier".to_string(),
            verified_trust_tier: OmenaSifTrustTierV1::T2,
            verified_tlog_integrated_time: None,
            sigstore_verification_policy: None,
            certificate_issuer: Some("https://token.actions.githubusercontent.com".to_string()),
            certificate_identity: Some(
                "https://github.com/omenien/omena-css/.github/workflows/release.yml@refs/tags/v1.0.0"
                    .to_string(),
            ),
            subject_canonical_url: entry.canonical_url.clone(),
            subject_sif_hash: entry.sif_hash.clone(),
        };

        let bad_schema =
            apply_omena_sif_attestation_verification_report_to_lock_entry_v1(&mut entry, &report);
        assert!(bad_schema.is_err(), "{bad_schema:?}");

        report.schema_version = "1".to_string();
        report.product = "other-product".to_string();
        let bad_product =
            apply_omena_sif_attestation_verification_report_to_lock_entry_v1(&mut entry, &report);
        assert!(bad_product.is_err(), "{bad_product:?}");

        report.product = "omena-sif.attestation-verification-report".to_string();
        report.verifier.clear();
        let empty_verifier =
            apply_omena_sif_attestation_verification_report_to_lock_entry_v1(&mut entry, &report);
        assert!(empty_verifier.is_err(), "{empty_verifier:?}");

        report.verifier = "offline-sigstore-verifier".to_string();
        report.verified_tlog_integrated_time = Some(0);
        let bad_integrated_time =
            apply_omena_sif_attestation_verification_report_to_lock_entry_v1(&mut entry, &report);
        assert!(bad_integrated_time.is_err(), "{bad_integrated_time:?}");

        report.verified_tlog_integrated_time = None;
        report.sigstore_verification_policy = Some(OmenaSifSigstoreVerificationPolicyV1 {
            trusted_root: "sigstore-production-trusted-root".to_string(),
            transparency_log: false,
            timestamp: true,
            certificate_chain: true,
            signed_certificate_timestamp: true,
        });
        let weak_sigstore_policy =
            apply_omena_sif_attestation_verification_report_to_lock_entry_v1(&mut entry, &report);
        assert!(weak_sigstore_policy.is_err(), "{weak_sigstore_policy:?}");

        report.sigstore_verification_policy = None;
        report.certificate_issuer = Some(" ".to_string());
        let empty_issuer =
            apply_omena_sif_attestation_verification_report_to_lock_entry_v1(&mut entry, &report);
        assert!(empty_issuer.is_err(), "{empty_issuer:?}");

        report.certificate_issuer = None;
        report.verifier = "sigstore-verify".to_string();
        report.sigstore_verification_policy = Some(OmenaSifSigstoreVerificationPolicyV1 {
            trusted_root: "sigstore-production-trusted-root".to_string(),
            transparency_log: true,
            timestamp: true,
            certificate_chain: true,
            signed_certificate_timestamp: true,
        });
        report.verified_tlog_integrated_time = Some(1_717_000_000);
        let sigstore_without_issuer =
            apply_omena_sif_attestation_verification_report_to_lock_entry_v1(&mut entry, &report);
        assert!(
            matches!(
                sigstore_without_issuer.as_ref(),
                Err(message) if message.contains("requires certificateIssuer")
            ),
            "{sigstore_without_issuer:?}"
        );
        report.certificate_issuer = Some("https://token.actions.githubusercontent.com".to_string());
        report.verified_tlog_integrated_time = None;
        let sigstore_without_log_time =
            apply_omena_sif_attestation_verification_report_to_lock_entry_v1(&mut entry, &report);
        assert!(
            matches!(
                sigstore_without_log_time.as_ref(),
                Err(message) if message.contains("requires verifiedTlogIntegratedTime")
            ),
            "{sigstore_without_log_time:?}"
        );
        Ok(())
    }

    #[test]
    fn lock_frozen_verification_passes_for_matching_sifs() -> Result<(), serde_json::Error> {
        let sif = fixture_sif("pkg:design-system/_tokens.scss", b"$color: red !default;")?;
        let sif_json = write_omena_sif_json_v1(&sif)?;
        let lock = OmenaLockV1::new(vec![build_omena_lock_sif_entry_v1(
            "sif/design-system.sif.json",
            &sif,
        )?]);
        let mut files = BTreeMap::new();
        files.insert("sif/design-system.sif.json".to_string(), sif_json);

        let report = verify_omena_lock_frozen_v1(&lock, |entry| {
            files
                .get(&entry.sif_path)
                .cloned()
                .ok_or_else(|| format!("missing {}", entry.sif_path))
        });

        assert!(report.verified, "{report:?}");
        assert_eq!(report.entries_checked, 1);
        assert!(report.issues.is_empty());
        Ok(())
    }

    #[test]
    fn lock_frozen_verification_fails_for_changed_sif() -> Result<(), serde_json::Error> {
        let locked_sif = fixture_sif("pkg:design-system/_tokens.scss", b"$color: red !default;")?;
        let changed_sif = fixture_sif("pkg:design-system/_tokens.scss", b"$color: blue !default;")?;
        let changed_json = write_omena_sif_json_v1(&changed_sif)?;
        let lock = OmenaLockV1::new(vec![build_omena_lock_sif_entry_v1(
            "sif/design-system.sif.json",
            &locked_sif,
        )?]);

        let report = verify_omena_lock_frozen_v1(&lock, |_entry| Ok(changed_json.clone()));

        assert!(!report.verified, "{report:?}");
        assert!(
            report
                .issues
                .iter()
                .any(|issue| issue.code == "sifHashMismatch"),
            "{report:?}"
        );
        Ok(())
    }

    fn fixture_generator() -> OmenaSifGeneratorV1 {
        OmenaSifGeneratorV1 {
            name: "omena-sifgen".to_string(),
            version: "0.1.0".to_string(),
            toolchain_id: "omena-sifgen@0.1".to_string(),
        }
    }

    fn fixture_source() -> OmenaSifSourceV1 {
        OmenaSifSourceV1 {
            syntax: OmenaSifSourceSyntaxV1::Scss,
        }
    }

    fn fixture_sif(
        canonical_url: &str,
        source_bytes: &[u8],
    ) -> Result<OmenaSifV1, serde_json::Error> {
        OmenaSifV1::from_static_exports(
            canonical_url,
            fixture_generator(),
            fixture_source(),
            fixture_exports(),
            Vec::new(),
            source_bytes,
        )
    }

    fn fixture_exports() -> OmenaSifExportsV1 {
        OmenaSifExportsV1 {
            variables: vec![OmenaSifVariableExportV1 {
                name: "$color".to_string(),
                defaulted: true,
                value_repr: Some("red".to_string()),
            }],
            mixins: vec![OmenaSifCallableExportV1 {
                name: "button".to_string(),
                parameters: vec![OmenaSifParameterV1 {
                    name: "$variant".to_string(),
                    default_value_repr: Some("primary".to_string()),
                    variadic: false,
                }],
                accepts_content: true,
            }],
            functions: Vec::new(),
            placeholders: vec![OmenaSifPlaceholderExportV1 {
                name: "%surface".to_string(),
            }],
            forwards: vec![OmenaSifForwardExportV1 {
                canonical_url: "pkg:design-system/_colors.scss".to_string(),
                prefix: None,
                show: vec!["$color".to_string()],
                hide: Vec::new(),
            }],
        }
    }
}
