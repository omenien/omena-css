use omena_sif::{
    OmenaSifSourceSyntaxV1, OmenaSifStaticGeneratorInputV1, generate_static_omena_sif_v1,
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
