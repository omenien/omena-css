use super::*;

#[cfg(test)]
fn assert_emitted_axes(probe_id: &str, actual: AnalysisPrecisionV1, expected: AnalysisPrecisionV1) {
    eprintln!(
        "OMENA_PRECISION_VECTOR {}",
        serde_json::json!({
            "probeId": probe_id,
            "testId": std::thread::current().name().unwrap_or(""),
            "fixtureFile": file!(),
            "fixtureSource": include_str!("cascade_precision_probes.rs"),
            "actualAxes": actual,
            "expectedAxes": expected,
        })
    );
    assert_eq!(actual, expected, "emitted precision vector {probe_id}");
}

#[test]
fn unresolved_cascade_keeps_provider_unresolved() -> Result<(), String> {
    let input = EngineInputV2 {
        version: "precision-fixture".to_string(),
        sources: vec![],
        styles: vec![],
        type_facts: vec![],
    };
    let mut value = read_omena_query_cascade_at_position(
        "file:///workspace/Example.css",
        ".card { color: red; }",
        &input,
        ParserPositionV0::default(),
    )
    .ok_or("cascade fixture missing")?;
    value.status = "unresolved";
    let result = cascade_at_position_analysis_result(value, 1);
    assert_emitted_axes(
        "unresolved-cascade-keeps-provider-unresolved",
        result.precision.axes,
        AnalysisPrecisionV1::from_axes_for_tests(
            ValueDomainPrecisionV1::CascadeAtPosition,
            FlowPrecisionV1::PositionScopedCascade,
            ContextPrecisionV1::StyleSemanticGraph,
            ProviderCompletenessV1::Unresolved,
            WorldAssumptionV1::Open,
            RevisionIdentityV1::EvaluationRuntimeExpressionDomain,
        ),
    );
    Ok(())
}
