use crate::{OmenaSifDigestV1, OmenaSifTrustTierV1, write_omena_canonical_json_string_v1};
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const OMENA_SIF_SHARD_TRUST_ENVELOPE_SCHEMA_VERSION_V1: &str = "1";
pub const OMENA_SIF_SHARD_TRUST_ENVELOPE_PRODUCT_V1: &str = "omena-sif.shard-trust-envelope";
pub const OMENA_SIF_SHARD_SIGNATURE_ALGORITHM_VERSION_V1: &str = "sigstore-bundle-v1";

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

fn is_omena_sif_digest_v1(value: &str) -> bool {
    let Some(hex) = value.strip_prefix("blake3:") else {
        return false;
    };
    hex.len() == 64
        && hex
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}
