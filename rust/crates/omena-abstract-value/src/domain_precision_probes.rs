use super::*;

fn assert_emitted_axes(probe_id: &str, actual: AnalysisPrecisionV1, expected: AnalysisPrecisionV1) {
    eprintln!(
        "OMENA_PRECISION_VECTOR {}",
        serde_json::json!({
            "probeId": probe_id,
            "testId": std::thread::current().name().unwrap_or(""),
            "fixtureFile": file!(),
            "fixtureSource": include_str!("domain_precision_probes.rs"),
            "actualAxes": actual,
            "expectedAxes": expected,
        })
    );
    assert_eq!(actual, expected, "emitted precision vector {probe_id}");
}

#[test]
fn witness_preserves_unresolved_axes() -> Result<(), String> {
    let value = AbstractClassValueV0::FiniteSet {
        values: vec!["card".to_string(), "panel".to_string()],
    };
    let external = OmenaAbstractValuePrecisionWitnessV0 {
        direction: OmenaAbstractValueCoverageDirectionV0::SupersetOfProducible,
        basis: OmenaAbstractValuePrecisionBasisV0::ClosedSetEnumeration,
        authority_digest: Some("fixture-authority".to_string()),
    };
    let witness = closed_world_precision_witness_from_class_value(&value, Some(&external))
        .ok_or("closed set witness missing")?;
    let receiver = AnalysisPrecisionV1::from_axes_for_tests(
        ValueDomainPrecisionV1::Unknown,
        FlowPrecisionV1::KLimitedCallSiteFlow,
        ContextPrecisionV1::SameFile,
        ProviderCompletenessV1::Unresolved,
        WorldAssumptionV1::Unknown,
        RevisionIdentityV1::StaleTypeFact,
    );
    assert_emitted_axes(
        "witness-preserves-unresolved-axes",
        witness.apply_to(receiver),
        AnalysisPrecisionV1::from_axes_for_tests(
            ValueDomainPrecisionV1::Unknown,
            FlowPrecisionV1::KLimitedCallSiteFlow,
            ContextPrecisionV1::SameFile,
            ProviderCompletenessV1::Unresolved,
            WorldAssumptionV1::Unknown,
            RevisionIdentityV1::StaleTypeFact,
        ),
    );
    Ok(())
}

#[test]
fn unwitnessed_finite_set_keeps_open_world() -> Result<(), String> {
    let value = AbstractClassValueV0::FiniteSet {
        values: vec!["card".to_string(), "panel".to_string()],
    };
    assert_emitted_axes(
        "unwitnessed-finite-set-keeps-open-world",
        analysis_precision_from_class_value_with_witness(&value, None),
        AnalysisPrecisionV1::from_axes_for_tests(
            ValueDomainPrecisionV1::ClosedClassValueSet,
            FlowPrecisionV1::RepresentationBound,
            ContextPrecisionV1::ValueLocal,
            ProviderCompletenessV1::Partial,
            WorldAssumptionV1::Open,
            RevisionIdentityV1::Current,
        ),
    );
    Ok(())
}
