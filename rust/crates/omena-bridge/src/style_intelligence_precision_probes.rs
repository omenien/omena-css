use super::*;

#[cfg(test)]
fn assert_emitted_axes(probe_id: &str, actual: AnalysisPrecisionV1, expected: AnalysisPrecisionV1) {
    eprintln!(
        "OMENA_PRECISION_VECTOR {}",
        serde_json::json!({
            "probeId": probe_id,
            "testId": std::thread::current().name().unwrap_or(""),
            "fixtureFile": file!(),
            "fixtureSource": include_str!("style_intelligence_precision_probes.rs"),
            "actualAxes": actual,
            "expectedAxes": expected,
        })
    );
    assert_eq!(actual, expected, "emitted precision vector {probe_id}");
}

#[cfg(test)]
#[test]
fn provider_without_target_keeps_unresolved_axes() -> Result<(), String> {
    assert_emitted_axes(
        "provider-without-target-keeps-unresolved-axes",
        crate::style_intelligence::provider_analysis_precision(0),
        AnalysisPrecisionV1::from_axes_for_tests(
            ValueDomainPrecisionV1::StyleModuleResolution,
            FlowPrecisionV1::ProviderObservation,
            ContextPrecisionV1::ProviderScoped,
            ProviderCompletenessV1::Unresolved,
            WorldAssumptionV1::Open,
            RevisionIdentityV1::Current,
        ),
    );
    Ok(())
}
