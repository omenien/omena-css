#[cfg(not(target_family = "wasm"))]
pub(crate) const OMENA_SIF_KEYLESS_CERTIFICATE_ISSUER: &str =
    "https://token.actions.githubusercontent.com";
#[cfg(not(target_family = "wasm"))]
pub(crate) const OMENA_SIF_KEYLESS_CERTIFICATE_IDENTITY: &str = "https://github.com/omenien/omena-css/.github/workflows/sif-keyless-attestation.yml@refs/heads/master";

#[cfg(not(target_family = "wasm"))]
pub(crate) fn verify_omena_external_sif_keyless_bundle(
    artifact_bytes: &[u8],
    bundle_bytes: &[u8],
    expected_bundle_sha256: &str,
) -> Result<(), String> {
    use sha2::{Digest, Sha256};
    use std::fmt::Write as _;

    let mut observed_bundle_sha256 = String::with_capacity(64);
    for byte in Sha256::digest(bundle_bytes) {
        write!(&mut observed_bundle_sha256, "{byte:02x}")
            .map_err(|error| format!("failed to encode Sigstore bundle digest: {error}"))?;
    }
    if observed_bundle_sha256 != expected_bundle_sha256 {
        return Err(format!(
            "Sigstore bundle content digest mismatch: expected {expected_bundle_sha256}, observed {observed_bundle_sha256}"
        ));
    }
    let bundle_source = std::str::from_utf8(bundle_bytes)
        .map_err(|error| format!("Sigstore bundle is not UTF-8 JSON: {error}"))?;
    let bundle = sigstore_verify::types::Bundle::from_json(bundle_source)
        .map_err(|error| format!("failed to parse Sigstore bundle: {error}"))?;
    if !bundle.has_inclusion_proof() {
        return Err("Sigstore bundle has no Rekor inclusion proof".to_string());
    }
    let trusted_root = sigstore_verify::trust_root::TrustedRoot::from_json(
        sigstore_verify::trust_root::SIGSTORE_PRODUCTION_TRUSTED_ROOT,
    )
    .map_err(|error| format!("failed to load Sigstore production trust root: {error}"))?;
    let policy = sigstore_verify::VerificationPolicy::default()
        .require_identity(OMENA_SIF_KEYLESS_CERTIFICATE_IDENTITY)
        .require_issuer(OMENA_SIF_KEYLESS_CERTIFICATE_ISSUER);
    sigstore_verify::verify(artifact_bytes, &bundle, &policy, &trusted_root)
        .map(|_| ())
        .map_err(|error| format!("offline Sigstore verification failed: {error}"))
}

#[cfg(target_family = "wasm")]
pub(crate) fn verify_omena_external_sif_keyless_bundle(
    _artifact_bytes: &[u8],
    _bundle_bytes: &[u8],
    _expected_bundle_sha256: &str,
) -> Result<(), String> {
    Err("offline Sigstore verification is unavailable in wasm builds".to_string())
}

#[cfg(test)]
mod tests {
    use super::verify_omena_external_sif_keyless_bundle;

    const FIXTURE_BUNDLE_SHA256: &str =
        "9b8793695c37482b03c41a72d188d7f815086ac1ad7c9df58013d507ba5af3ec";

    #[test]
    fn workflow_keyless_bundle_verifies_against_external_trust_root() {
        let artifact = include_str!("../tests/fixtures/keyless-attested-shard.sif.json");
        let bundle = include_bytes!("../tests/fixtures/keyless-attested-shard.sigstore.json");
        let result = verify_omena_external_sif_keyless_bundle(
            artifact.trim_end().as_bytes(),
            bundle,
            FIXTURE_BUNDLE_SHA256,
        );
        assert!(result.is_ok(), "{result:?}");
    }

    #[test]
    fn workflow_keyless_bundle_rejects_changed_sif_bytes() {
        let artifact = include_str!("../tests/fixtures/keyless-attested-shard.sif.json");
        let bundle = include_bytes!("../tests/fixtures/keyless-attested-shard.sigstore.json");
        let mut poisoned = artifact.trim_end().as_bytes().to_vec();
        poisoned.push(b' ');
        let result = verify_omena_external_sif_keyless_bundle(
            poisoned.as_slice(),
            bundle,
            FIXTURE_BUNDLE_SHA256,
        );
        assert!(result.is_err(), "changed SIF bytes unexpectedly verified");
    }
}
