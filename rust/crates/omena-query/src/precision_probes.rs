use crate::*;

fn assert_emitted_axes(probe_id: &str, actual: AnalysisPrecisionV1, expected: AnalysisPrecisionV1) {
    eprintln!(
        "OMENA_PRECISION_VECTOR {}",
        serde_json::json!({
            "probeId": probe_id,
            "testId": std::thread::current().name().unwrap_or(""),
            "fixtureFile": file!(),
            "fixtureSource": include_str!("precision_probes.rs"),
            "actualAxes": actual,
            "expectedAxes": expected,
        })
    );
    assert_eq!(actual, expected, "emitted precision vector {probe_id}");
}

#[test]
fn source_diagnostic_keeps_input_revision() -> Result<(), String> {
    let result = crate::types::source_diagnostic_precision(
        ValueDomainPrecisionV1::ClassValueResolution,
        FlowPrecisionV1::SourceControlFlow,
        ContextPrecisionV1::SameFile,
        1,
        false,
    );
    assert_emitted_axes(
        "source-diagnostic-keeps-input-revision",
        result.axes,
        AnalysisPrecisionV1::from_axes_for_tests(
            ValueDomainPrecisionV1::ClassValueResolution,
            FlowPrecisionV1::SourceControlFlow,
            ContextPrecisionV1::SameFile,
            ProviderCompletenessV1::Unresolved,
            WorldAssumptionV1::Open,
            RevisionIdentityV1::QuerySourceDiagnosticsInput,
        ),
    );
    Ok(())
}

#[test]
fn missing_source_capture_keeps_provider_unresolved() -> Result<(), String> {
    let result = resolve_omena_query_source_precision_for_source(
        "file:///workspace/Example.tsx",
        "export const value = 1;",
        Some("typescript"),
        "missing",
        0,
    );
    assert_emitted_axes(
        "missing-source-capture-keeps-provider-unresolved",
        result.precision.axes,
        AnalysisPrecisionV1::from_axes_for_tests(
            ValueDomainPrecisionV1::ClassValueResolution,
            FlowPrecisionV1::SourceControlFlow,
            ContextPrecisionV1::SameFile,
            ProviderCompletenessV1::Unresolved,
            WorldAssumptionV1::Open,
            RevisionIdentityV1::QuerySourceDiagnosticsInput,
        ),
    );
    Ok(())
}
