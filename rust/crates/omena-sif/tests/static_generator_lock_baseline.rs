use omena_sif::{
    OmenaLockV1, OmenaSifGeneratorV1, OmenaSifSourceSyntaxV1, OmenaSifSourceV1,
    OmenaSifStaticGeneratorInputV1, OmenaSifV1, build_omena_lock_sif_entry_v1,
    compute_omena_sif_artifact_hash_v1, generate_static_omena_sif_v1, parse_static_sass_exports_v1,
    read_omena_lock_json_v1, verify_omena_lock_frozen_v1, write_omena_sif_json_v1,
};

const SOURCE: &str = include_str!("fixtures/static-generator-hash-contract.scss");
const LOCK: &str = include_str!("fixtures/static-generator-lock.json");

#[test]
fn frozen_lock_tracks_the_static_generator_identity() -> Result<(), serde_json::Error> {
    let sif = generate_static_omena_sif_v1(OmenaSifStaticGeneratorInputV1 {
        canonical_url: "pkg:fixture/tokens.scss",
        source: SOURCE,
        syntax: OmenaSifSourceSyntaxV1::Scss,
    })?;
    let sif_json = write_omena_sif_json_v1(&sif)?;
    let lock = read_omena_lock_json_v1(LOCK)?;
    let report = verify_omena_lock_frozen_v1(&lock, |_entry| Ok(sif_json.clone()));

    assert!(report.verified, "{:#?}", report.issues);
    assert_eq!(report.entries_checked, 1);
    Ok(())
}

#[test]
fn generator_identity_rotation_rekeys_the_interface_chain_without_rehashing_source()
-> Result<(), serde_json::Error> {
    let current = generate_static_omena_sif_v1(OmenaSifStaticGeneratorInputV1 {
        canonical_url: "pkg:fixture/tokens.scss",
        source: SOURCE,
        syntax: OmenaSifSourceSyntaxV1::Scss,
    })?;
    let legacy = OmenaSifV1::from_static_exports(
        "pkg:fixture/tokens.scss",
        OmenaSifGeneratorV1 {
            name: "omena-sifgen-static".to_string(),
            version: "0.1.0".to_string(),
            toolchain_id: "omena-sifgen-static@0.1.0".to_string(),
        },
        OmenaSifSourceV1 {
            syntax: OmenaSifSourceSyntaxV1::Scss,
        },
        parse_static_sass_exports_v1(SOURCE),
        Vec::new(),
        SOURCE.as_bytes(),
    )?;

    assert_eq!(
        current.fingerprints.leaf_hash,
        legacy.fingerprints.leaf_hash
    );
    assert_ne!(
        current.fingerprints.interface_hash,
        legacy.fingerprints.interface_hash
    );
    assert_ne!(
        current.fingerprints.transitive_hash,
        legacy.fingerprints.transitive_hash
    );
    assert_ne!(
        compute_omena_sif_artifact_hash_v1(&current)?,
        compute_omena_sif_artifact_hash_v1(&legacy)?
    );

    let legacy_lock = OmenaLockV1::new(vec![build_omena_lock_sif_entry_v1(
        "tokens.sif.json",
        &legacy,
    )?]);
    let current_json = write_omena_sif_json_v1(&current)?;
    let report = verify_omena_lock_frozen_v1(&legacy_lock, |_entry| Ok(current_json.clone()));
    let issue_codes = report
        .issues
        .iter()
        .map(|issue| issue.code.as_str())
        .collect::<Vec<_>>();

    assert!(!report.verified);
    assert!(issue_codes.contains(&"sifHashMismatch"));
    assert!(issue_codes.contains(&"interfaceHashMismatch"));
    assert!(issue_codes.contains(&"transitiveHashMismatch"));
    Ok(())
}
