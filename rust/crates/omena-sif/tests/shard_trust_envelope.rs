use omena_sif::{
    OMENA_SIF_SHARD_SIGNATURE_ALGORITHM_VERSION_V1, OMENA_SIF_SHARD_TRUST_ENVELOPE_PRODUCT_V1,
    OMENA_SIF_SHARD_TRUST_ENVELOPE_SCHEMA_VERSION_V1,
    OMENA_SIF_SHARD_TRUST_ENVELOPE_V1_SCHEMA_JSON, OmenaSifShardLockBindingV1,
    OmenaSifShardSignatureV1, OmenaSifShardTrustEnvelopeErrorV1, OmenaSifShardTrustEnvelopeV1,
    OmenaSifTrustTierV1, compute_omena_sif_leaf_hash_v1,
    read_omena_sif_shard_trust_envelope_json_v1, validate_omena_sif_shard_trust_envelope_v1,
    write_omena_sif_shard_trust_envelope_json_v1,
};
use serde_json::{Value, json};

const COMMITTED_ENVELOPE: &str = include_str!("fixtures/shard-trust-envelope-v1.json");

#[test]
fn committed_shard_trust_envelope_remains_cross_fixture_compatible()
-> Result<(), Box<dyn std::error::Error>> {
    let envelope = read_omena_sif_shard_trust_envelope_json_v1(COMMITTED_ENVELOPE)?;
    assert_eq!(envelope.trust_tier, OmenaSifTrustTierV1::T3);
    assert_eq!(
        envelope
            .signature
            .as_ref()
            .map(|signature| signature.algorithm_version.as_str()),
        Some(OMENA_SIF_SHARD_SIGNATURE_ALGORITHM_VERSION_V1)
    );
    let canonical = write_omena_sif_shard_trust_envelope_json_v1(&envelope)?;
    assert_eq!(
        serde_json::from_str::<Value>(&canonical)?,
        serde_json::from_str::<Value>(COMMITTED_ENVELOPE)?
    );
    Ok(())
}

#[test]
fn shard_trust_envelope_round_trip_binds_the_existing_payload_digest()
-> Result<(), Box<dyn std::error::Error>> {
    let payload_digest = compute_omena_sif_leaf_hash_v1(b"committed shard payload bytes");
    let envelope = OmenaSifShardTrustEnvelopeV1 {
        schema_version: OMENA_SIF_SHARD_TRUST_ENVELOPE_SCHEMA_VERSION_V1.to_string(),
        product: OMENA_SIF_SHARD_TRUST_ENVELOPE_PRODUCT_V1.to_string(),
        trust_tier: OmenaSifTrustTierV1::T3,
        payload_digest: payload_digest.clone(),
        signature: Some(OmenaSifShardSignatureV1 {
            algorithm_version: OMENA_SIF_SHARD_SIGNATURE_ALGORITHM_VERSION_V1.to_string(),
            reference: "sif-shards/batch.sigstore.json".to_string(),
            signed_payload_digest: payload_digest,
        }),
        lock_binding: OmenaSifShardLockBindingV1 {
            canonical_url: "pkg:design-system/_tokens.scss".to_string(),
            sif_hash: compute_omena_sif_leaf_hash_v1(b"committed SIF bytes"),
        },
    };
    validate_omena_sif_shard_trust_envelope_v1(&envelope)?;
    let encoded = write_omena_sif_shard_trust_envelope_json_v1(&envelope)?;
    assert_eq!(
        read_omena_sif_shard_trust_envelope_json_v1(encoded.as_str())?,
        envelope
    );
    Ok(())
}

#[test]
fn shard_trust_envelope_fixture_mutations_have_typed_refusals()
-> Result<(), Box<dyn std::error::Error>> {
    assert_fixture_mutation_error(
        |value| {
            value["payloadDigest"] =
                json!("blake3:1123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef");
        },
        OmenaSifShardTrustEnvelopeErrorV1::SignedPayloadDigestMismatch,
    )?;
    assert_fixture_mutation_error(
        |value| value["trustTier"] = json!("T3"),
        OmenaSifShardTrustEnvelopeErrorV1::InvalidTrustTier,
    )?;
    assert_fixture_mutation_error(
        |value| value["signature"]["algorithmVersion"] = json!("sigstore-bundle-v2"),
        OmenaSifShardTrustEnvelopeErrorV1::UnsupportedSignatureAlgorithm,
    )?;
    Ok(())
}

#[test]
fn shard_trust_envelope_reuses_the_lock_plane_tier_vocabulary()
-> Result<(), Box<dyn std::error::Error>> {
    let serialized_tiers = [
        OmenaSifTrustTierV1::T0,
        OmenaSifTrustTierV1::T1,
        OmenaSifTrustTierV1::T2,
        OmenaSifTrustTierV1::T3,
    ]
    .map(serde_json::to_value)
    .into_iter()
    .collect::<Result<Vec<_>, _>>()?;
    assert_eq!(
        serialized_tiers,
        vec![json!("t0"), json!("t1"), json!("t2"), json!("t3")]
    );
    let schema = serde_json::from_str::<Value>(OMENA_SIF_SHARD_TRUST_ENVELOPE_V1_SCHEMA_JSON)?;
    assert_eq!(
        schema["properties"]["trustTier"]["enum"],
        json!(["t0", "t1", "t2", "t3"])
    );
    Ok(())
}

fn assert_fixture_mutation_error(
    mutate: impl FnOnce(&mut Value),
    expected: OmenaSifShardTrustEnvelopeErrorV1,
) -> Result<(), serde_json::Error> {
    let mut value = serde_json::from_str::<Value>(COMMITTED_ENVELOPE)?;
    mutate(&mut value);
    let source = serde_json::to_string(&value)?;
    assert_eq!(
        read_omena_sif_shard_trust_envelope_json_v1(source.as_str()),
        Err(expected),
        "fixture mutation must have a typed refusal"
    );
    Ok(())
}
