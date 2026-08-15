use crate::{
    OmenaSifDigestV1, OmenaSifTrustTierV1, OmenaSifV1, compute_omena_sif_artifact_hash_v1,
    write_omena_canonical_json_string_v1,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const OMENA_SIF_SHARD_TRUST_ENVELOPE_SCHEMA_VERSION_V1: &str = "1";
pub const OMENA_SIF_SHARD_TRUST_ENVELOPE_PRODUCT_V1: &str = "omena-sif.shard-trust-envelope";
pub const OMENA_SIF_SHARD_SIGNATURE_ALGORITHM_VERSION_V1: &str = "sigstore-bundle-v1";
pub const OMENA_SIF_SHARD_RECORDED_VERDICT_SCHEMA_VERSION_V1: &str = "1";
pub const OMENA_SIF_SHARD_RECORDED_VERDICT_PRODUCT_V1: &str = "omena-sif.shard-recorded-verdict";
pub const OMENA_SIF_SHARD_RECORDED_VERDICT_ADDRESS_PRODUCT_V1: &str =
    "omena-sif.shard-recorded-verdict-address";
pub const OMENA_SIF_SHARD_VERDICT_DIR_V1: &str = "sif-verdicts-v1";
pub const OMENA_SIF_SHARD_VERIFICATION_OWNER_V1: &str = "omena-cli.lock-provenance";
pub const OMENA_SIF_PUBLISHED_ATTESTATION_SUBJECT_SCHEMA_VERSION_V1: &str = "1";
pub const OMENA_SIF_PUBLISHED_ATTESTATION_SUBJECT_PRODUCT_V1: &str =
    "omena-sif.published-attestation-subject";

/// Canonical bytes signed by the Omena release workflow for elevated provenance.
///
/// This descriptor binds the resource identity and advisory tier to the exact
/// SIF artifact hash. The SIF itself remains a deterministic, verifier-free
/// data format suitable for wasm consumers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaSifPublishedAttestationSubjectV1 {
    pub schema_version: String,
    pub product: String,
    pub canonical_url: String,
    pub trust_tier: OmenaSifTrustTierV1,
    pub sif_hash: OmenaSifDigestV1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OmenaSifPublishedAttestationSubjectErrorV1 {
    MalformedJson,
    MalformedSubject,
    UnsupportedSchemaVersion,
    UnsupportedProduct,
    EmptyCanonicalUrl,
    InsufficientTrustTier,
    MalformedSifHash,
}

impl OmenaSifPublishedAttestationSubjectErrorV1 {
    pub fn code(self) -> &'static str {
        match self {
            Self::MalformedJson => "malformedJson",
            Self::MalformedSubject => "malformedSubject",
            Self::UnsupportedSchemaVersion => "unsupportedSchemaVersion",
            Self::UnsupportedProduct => "unsupportedProduct",
            Self::EmptyCanonicalUrl => "emptyCanonicalUrl",
            Self::InsufficientTrustTier => "insufficientTrustTier",
            Self::MalformedSifHash => "malformedSifHash",
        }
    }
}

impl std::fmt::Display for OmenaSifPublishedAttestationSubjectErrorV1 {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.code())
    }
}

impl std::error::Error for OmenaSifPublishedAttestationSubjectErrorV1 {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaSifShardTrustEnvelopeV1 {
    pub schema_version: String,
    pub product: String,
    pub trust_tier: OmenaSifTrustTierV1,
    pub payload_digest: OmenaSifDigestV1,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signature: Option<OmenaSifShardSignatureV1>,
    pub lock_binding: OmenaSifShardLockBindingV1,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaSifShardSignatureV1 {
    pub algorithm_version: String,
    pub reference: String,
    pub signed_payload_digest: OmenaSifDigestV1,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaSifShardLockBindingV1 {
    pub canonical_url: String,
    pub sif_hash: OmenaSifDigestV1,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaSifShardRecordedVerdictV1 {
    pub schema_version: String,
    pub product: String,
    pub verification_owner: String,
    pub canonical_url: String,
    pub sif_hash: OmenaSifDigestV1,
    pub trust_tier: OmenaSifTrustTierV1,
    pub signature: OmenaSifShardSignatureV1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OmenaSifShardTrustEnvelopeErrorV1 {
    MalformedJson,
    MalformedEnvelope,
    UnsupportedSchemaVersion,
    UnsupportedProduct,
    InvalidTrustTier,
    MalformedPayloadDigest,
    MissingSignature,
    UnsupportedSignatureAlgorithm,
    EmptySignatureReference,
    MalformedSignedPayloadDigest,
    SignedPayloadDigestMismatch,
    EmptyCanonicalUrl,
    MalformedSifHash,
}

impl OmenaSifShardTrustEnvelopeErrorV1 {
    pub fn code(self) -> &'static str {
        match self {
            Self::MalformedJson => "malformedJson",
            Self::MalformedEnvelope => "malformedEnvelope",
            Self::UnsupportedSchemaVersion => "unsupportedSchemaVersion",
            Self::UnsupportedProduct => "unsupportedProduct",
            Self::InvalidTrustTier => "invalidTrustTier",
            Self::MalformedPayloadDigest => "malformedPayloadDigest",
            Self::MissingSignature => "missingSignature",
            Self::UnsupportedSignatureAlgorithm => "unsupportedSignatureAlgorithm",
            Self::EmptySignatureReference => "emptySignatureReference",
            Self::MalformedSignedPayloadDigest => "malformedSignedPayloadDigest",
            Self::SignedPayloadDigestMismatch => "signedPayloadDigestMismatch",
            Self::EmptyCanonicalUrl => "emptyCanonicalUrl",
            Self::MalformedSifHash => "malformedSifHash",
        }
    }
}

impl std::fmt::Display for OmenaSifShardTrustEnvelopeErrorV1 {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.code())
    }
}

impl std::error::Error for OmenaSifShardTrustEnvelopeErrorV1 {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OmenaSifShardRecordedVerdictErrorV1 {
    MalformedJson,
    MalformedVerdict,
    UnsupportedSchemaVersion,
    UnsupportedProduct,
    UnsupportedVerificationOwner,
    EmptyCanonicalUrl,
    MalformedSifHash,
    InsufficientTrustTier,
    UnsupportedSignatureAlgorithm,
    EmptySignatureReference,
    MalformedSignedPayloadDigest,
    SignedSifDigestMismatch,
}

impl OmenaSifShardRecordedVerdictErrorV1 {
    pub fn code(self) -> &'static str {
        match self {
            Self::MalformedJson => "malformedJson",
            Self::MalformedVerdict => "malformedVerdict",
            Self::UnsupportedSchemaVersion => "unsupportedSchemaVersion",
            Self::UnsupportedProduct => "unsupportedProduct",
            Self::UnsupportedVerificationOwner => "unsupportedVerificationOwner",
            Self::EmptyCanonicalUrl => "emptyCanonicalUrl",
            Self::MalformedSifHash => "malformedSifHash",
            Self::InsufficientTrustTier => "insufficientTrustTier",
            Self::UnsupportedSignatureAlgorithm => "unsupportedSignatureAlgorithm",
            Self::EmptySignatureReference => "emptySignatureReference",
            Self::MalformedSignedPayloadDigest => "malformedSignedPayloadDigest",
            Self::SignedSifDigestMismatch => "signedSifDigestMismatch",
        }
    }
}

impl std::fmt::Display for OmenaSifShardRecordedVerdictErrorV1 {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.code())
    }
}

impl std::error::Error for OmenaSifShardRecordedVerdictErrorV1 {}

pub fn build_omena_sif_published_attestation_subject_v1(
    sif: &OmenaSifV1,
    trust_tier: OmenaSifTrustTierV1,
) -> Result<OmenaSifPublishedAttestationSubjectV1, serde_json::Error> {
    Ok(OmenaSifPublishedAttestationSubjectV1 {
        schema_version: OMENA_SIF_PUBLISHED_ATTESTATION_SUBJECT_SCHEMA_VERSION_V1.to_string(),
        product: OMENA_SIF_PUBLISHED_ATTESTATION_SUBJECT_PRODUCT_V1.to_string(),
        canonical_url: sif.canonical_url.clone(),
        trust_tier,
        sif_hash: compute_omena_sif_artifact_hash_v1(sif)?,
    })
}

pub fn read_omena_sif_published_attestation_subject_json_v1(
    source: &str,
) -> Result<OmenaSifPublishedAttestationSubjectV1, OmenaSifPublishedAttestationSubjectErrorV1> {
    let value = serde_json::from_str::<Value>(source)
        .map_err(|_| OmenaSifPublishedAttestationSubjectErrorV1::MalformedJson)?;
    if !matches!(
        value.get("trustTier").and_then(Value::as_str),
        Some("t0" | "t1" | "t2" | "t3")
    ) {
        return Err(OmenaSifPublishedAttestationSubjectErrorV1::MalformedSubject);
    }
    let subject = serde_json::from_value::<OmenaSifPublishedAttestationSubjectV1>(value)
        .map_err(|_| OmenaSifPublishedAttestationSubjectErrorV1::MalformedSubject)?;
    validate_omena_sif_published_attestation_subject_v1(&subject)?;
    Ok(subject)
}

pub fn write_omena_sif_published_attestation_subject_json_v1(
    subject: &OmenaSifPublishedAttestationSubjectV1,
) -> Result<String, serde_json::Error> {
    write_omena_canonical_json_string_v1(subject)
}

pub fn validate_omena_sif_published_attestation_subject_v1(
    subject: &OmenaSifPublishedAttestationSubjectV1,
) -> Result<(), OmenaSifPublishedAttestationSubjectErrorV1> {
    if subject.schema_version != OMENA_SIF_PUBLISHED_ATTESTATION_SUBJECT_SCHEMA_VERSION_V1 {
        return Err(OmenaSifPublishedAttestationSubjectErrorV1::UnsupportedSchemaVersion);
    }
    if subject.product != OMENA_SIF_PUBLISHED_ATTESTATION_SUBJECT_PRODUCT_V1 {
        return Err(OmenaSifPublishedAttestationSubjectErrorV1::UnsupportedProduct);
    }
    if subject.canonical_url.trim().is_empty() {
        return Err(OmenaSifPublishedAttestationSubjectErrorV1::EmptyCanonicalUrl);
    }
    if subject.trust_tier < OmenaSifTrustTierV1::T2 {
        return Err(OmenaSifPublishedAttestationSubjectErrorV1::InsufficientTrustTier);
    }
    if !is_omena_sif_digest_v1(subject.sif_hash.as_str()) {
        return Err(OmenaSifPublishedAttestationSubjectErrorV1::MalformedSifHash);
    }
    Ok(())
}

pub fn read_omena_sif_shard_trust_envelope_json_v1(
    source: &str,
) -> Result<OmenaSifShardTrustEnvelopeV1, OmenaSifShardTrustEnvelopeErrorV1> {
    let value = serde_json::from_str::<Value>(source)
        .map_err(|_| OmenaSifShardTrustEnvelopeErrorV1::MalformedJson)?;
    if !matches!(
        value.get("trustTier").and_then(Value::as_str),
        Some("t0" | "t1" | "t2" | "t3")
    ) {
        return Err(OmenaSifShardTrustEnvelopeErrorV1::InvalidTrustTier);
    }
    let envelope = serde_json::from_value::<OmenaSifShardTrustEnvelopeV1>(value)
        .map_err(|_| OmenaSifShardTrustEnvelopeErrorV1::MalformedEnvelope)?;
    validate_omena_sif_shard_trust_envelope_v1(&envelope)?;
    Ok(envelope)
}

pub fn write_omena_sif_shard_trust_envelope_json_v1(
    envelope: &OmenaSifShardTrustEnvelopeV1,
) -> Result<String, serde_json::Error> {
    write_omena_canonical_json_string_v1(envelope)
}

pub fn validate_omena_sif_shard_trust_envelope_v1(
    envelope: &OmenaSifShardTrustEnvelopeV1,
) -> Result<(), OmenaSifShardTrustEnvelopeErrorV1> {
    if envelope.schema_version != OMENA_SIF_SHARD_TRUST_ENVELOPE_SCHEMA_VERSION_V1 {
        return Err(OmenaSifShardTrustEnvelopeErrorV1::UnsupportedSchemaVersion);
    }
    if envelope.product != OMENA_SIF_SHARD_TRUST_ENVELOPE_PRODUCT_V1 {
        return Err(OmenaSifShardTrustEnvelopeErrorV1::UnsupportedProduct);
    }
    if !is_omena_sif_digest_v1(envelope.payload_digest.as_str()) {
        return Err(OmenaSifShardTrustEnvelopeErrorV1::MalformedPayloadDigest);
    }
    if envelope.lock_binding.canonical_url.trim().is_empty() {
        return Err(OmenaSifShardTrustEnvelopeErrorV1::EmptyCanonicalUrl);
    }
    if !is_omena_sif_digest_v1(envelope.lock_binding.sif_hash.as_str()) {
        return Err(OmenaSifShardTrustEnvelopeErrorV1::MalformedSifHash);
    }
    let Some(signature) = envelope.signature.as_ref() else {
        if envelope.trust_tier >= OmenaSifTrustTierV1::T2 {
            return Err(OmenaSifShardTrustEnvelopeErrorV1::MissingSignature);
        }
        return Ok(());
    };
    if signature.algorithm_version != OMENA_SIF_SHARD_SIGNATURE_ALGORITHM_VERSION_V1 {
        return Err(OmenaSifShardTrustEnvelopeErrorV1::UnsupportedSignatureAlgorithm);
    }
    if signature.reference.trim().is_empty() {
        return Err(OmenaSifShardTrustEnvelopeErrorV1::EmptySignatureReference);
    }
    if !is_omena_sif_digest_v1(signature.signed_payload_digest.as_str()) {
        return Err(OmenaSifShardTrustEnvelopeErrorV1::MalformedSignedPayloadDigest);
    }
    if signature.signed_payload_digest != envelope.payload_digest {
        return Err(OmenaSifShardTrustEnvelopeErrorV1::SignedPayloadDigestMismatch);
    }
    Ok(())
}

pub fn compute_omena_sif_shard_recorded_verdict_address_v1(
    canonical_url: &str,
    sif_hash: &OmenaSifDigestV1,
) -> Result<OmenaSifDigestV1, serde_json::Error> {
    crate::compute_omena_stable_cache_shard_address_v1(
        OMENA_SIF_SHARD_RECORDED_VERDICT_ADDRESS_PRODUCT_V1,
        &[canonical_url, sif_hash.as_str()],
    )
}

pub fn read_omena_sif_shard_recorded_verdict_json_v1(
    source: &str,
) -> Result<OmenaSifShardRecordedVerdictV1, OmenaSifShardRecordedVerdictErrorV1> {
    let verdict = serde_json::from_str::<OmenaSifShardRecordedVerdictV1>(source)
        .map_err(|_| OmenaSifShardRecordedVerdictErrorV1::MalformedJson)?;
    validate_omena_sif_shard_recorded_verdict_v1(&verdict)?;
    Ok(verdict)
}

pub fn write_omena_sif_shard_recorded_verdict_json_v1(
    verdict: &OmenaSifShardRecordedVerdictV1,
) -> Result<String, serde_json::Error> {
    write_omena_canonical_json_string_v1(verdict)
}

pub fn validate_omena_sif_shard_recorded_verdict_v1(
    verdict: &OmenaSifShardRecordedVerdictV1,
) -> Result<(), OmenaSifShardRecordedVerdictErrorV1> {
    if verdict.schema_version != OMENA_SIF_SHARD_RECORDED_VERDICT_SCHEMA_VERSION_V1 {
        return Err(OmenaSifShardRecordedVerdictErrorV1::UnsupportedSchemaVersion);
    }
    if verdict.product != OMENA_SIF_SHARD_RECORDED_VERDICT_PRODUCT_V1 {
        return Err(OmenaSifShardRecordedVerdictErrorV1::UnsupportedProduct);
    }
    if verdict.verification_owner != OMENA_SIF_SHARD_VERIFICATION_OWNER_V1 {
        return Err(OmenaSifShardRecordedVerdictErrorV1::UnsupportedVerificationOwner);
    }
    if verdict.canonical_url.trim().is_empty() {
        return Err(OmenaSifShardRecordedVerdictErrorV1::EmptyCanonicalUrl);
    }
    if !is_omena_sif_digest_v1(verdict.sif_hash.as_str()) {
        return Err(OmenaSifShardRecordedVerdictErrorV1::MalformedSifHash);
    }
    if verdict.trust_tier < OmenaSifTrustTierV1::T2 {
        return Err(OmenaSifShardRecordedVerdictErrorV1::InsufficientTrustTier);
    }
    if verdict.signature.algorithm_version != OMENA_SIF_SHARD_SIGNATURE_ALGORITHM_VERSION_V1 {
        return Err(OmenaSifShardRecordedVerdictErrorV1::UnsupportedSignatureAlgorithm);
    }
    if verdict.signature.reference.trim().is_empty() {
        return Err(OmenaSifShardRecordedVerdictErrorV1::EmptySignatureReference);
    }
    if !is_omena_sif_digest_v1(verdict.signature.signed_payload_digest.as_str()) {
        return Err(OmenaSifShardRecordedVerdictErrorV1::MalformedSignedPayloadDigest);
    }
    if verdict.signature.signed_payload_digest != verdict.sif_hash {
        return Err(OmenaSifShardRecordedVerdictErrorV1::SignedSifDigestMismatch);
    }
    Ok(())
}

fn is_omena_sif_digest_v1(value: &str) -> bool {
    let Some(hex) = value.strip_prefix("blake3:") else {
        return false;
    };
    hex.len() == 64
        && hex
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}
