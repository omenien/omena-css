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
        .require_identity(omena_sif::OMENA_SIF_PUBLISHED_ATTESTATION_CERTIFICATE_IDENTITY_V1)
        .require_issuer(omena_sif::OMENA_SIF_PUBLISHED_ATTESTATION_CERTIFICATE_ISSUER_V1);
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
        "0c99e37ac1b1d3cbfd677416a74218c9a1ca8e28c3aac95c7614549f3b3b0ce1";

    #[test]
    fn published_subject_bundle_verifies_against_external_trust_root() {
        let artifact = include_str!("../tests/fixtures/published-sif-attestation.subject.json");
        let bundle = include_bytes!("../tests/fixtures/published-sif-attestation.sigstore.json");
        let result = verify_omena_external_sif_keyless_bundle(
            artifact.trim_end().as_bytes(),
            bundle,
            FIXTURE_BUNDLE_SHA256,
        );
        assert!(result.is_ok(), "{result:?}");
    }

    #[test]
    fn published_subject_bundle_rejects_changed_identity_or_tier_bytes() {
        let artifact = include_str!("../tests/fixtures/published-sif-attestation.subject.json");
        let bundle = include_bytes!("../tests/fixtures/published-sif-attestation.sigstore.json");
        let mut poisoned = artifact.trim_end().as_bytes().to_vec();
        poisoned.push(b' ');
        let result = verify_omena_external_sif_keyless_bundle(
            poisoned.as_slice(),
            bundle,
            FIXTURE_BUNDLE_SHA256,
        );
        assert!(
            result.is_err(),
            "changed subject bytes unexpectedly verified"
        );
    }
}
