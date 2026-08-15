use serde_json::Value;

#[test]
fn npm_provenance_fixture_registry_pins_typed_ingestion_and_absence()
-> Result<(), Box<dyn std::error::Error>> {
    let registry = serde_json::from_str::<Value>(include_str!(
        "fixtures/npm-provenance-metadata-registry-v1.json"
    ))?;
    assert_eq!(
        registry.pointer("/schemaVersion").and_then(Value::as_str),
        Some("1")
    );
    let cases = registry
        .pointer("/cases")
        .and_then(Value::as_array)
        .ok_or("fixture registry cases")?;
    let mut present = 0_usize;
    let mut absent = 0_usize;
    let mut refused = 0_usize;
    for case in cases {
        let id = case
            .pointer("/id")
            .and_then(Value::as_str)
            .ok_or("case id")?;
        let selector = case
            .pointer("/packageSelector")
            .and_then(Value::as_str)
            .ok_or("package selector")?;
        let source = case
            .pointer("/source")
            .and_then(Value::as_str)
            .ok_or("metadata source")?;
        let result = omena_sif::ingest_omena_sif_npm_provenance_metadata_v1(source, selector);
        if let Some(expected_code) = case.pointer("/expectedErrorCode").and_then(Value::as_str) {
            let error = match result {
                Err(error) => error,
                Ok(references) => {
                    return Err(format!("{id}: expected refusal, got {references:?}").into());
                }
            };
            assert_eq!(error.code(), expected_code, "{id}");
            refused += 1;
            continue;
        }
        let references = result.map_err(|error| format!("{id}: {error}"))?;
        let expected_count = case
            .pointer("/expectedReferenceCount")
            .and_then(Value::as_u64)
            .ok_or("expected reference count")? as usize;
        assert_eq!(references.len(), expected_count, "{id}");
        match case.pointer("/expectedDisposition").and_then(Value::as_str) {
            Some("present") => {
                assert!(!references.is_empty(), "{id}");
                present += 1;
            }
            Some("absent") => {
                assert!(references.is_empty(), "{id}");
                absent += 1;
            }
            disposition => {
                return Err(format!("{id}: unexpected disposition {disposition:?}").into());
            }
        }
    }
    assert_eq!((present, absent, refused), (2, 1, 10));
    Ok(())
}
