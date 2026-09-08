use super::*;

#[cfg(test)]
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
fn incremental_flow_keeps_expression_context() -> Result<(), String> {
    let input = EngineInputV2 {
        version: "precision-fixture".to_string(),
        sources: vec![],
        styles: vec![],
        type_facts: vec![],
    };
    let mut runtime = OmenaQueryExpressionDomainFlowRuntimeV0::default();
    let result = summarize_omena_query_expression_domain_incremental_flow_analysis_result(
        &input,
        &mut runtime,
    );
    assert_emitted_axes(
        "incremental-flow-keeps-expression-context",
        result.precision.axes,
        AnalysisPrecisionV1::from_axes_for_tests(
            ValueDomainPrecisionV1::ClassValueFlow,
            FlowPrecisionV1::IncrementalDataflow,
            ContextPrecisionV1::PerExpressionGraph,
            ProviderCompletenessV1::Complete,
            WorldAssumptionV1::Closed,
            RevisionIdentityV1::ExpressionDomainFlowRuntime,
        ),
    );
    Ok(())
}
