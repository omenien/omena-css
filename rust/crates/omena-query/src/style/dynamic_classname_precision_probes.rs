use super::*;

#[cfg(test)]
fn assert_emitted_axes(probe_id: &str, actual: AnalysisPrecisionV1, expected: AnalysisPrecisionV1) {
    eprintln!(
        "OMENA_PRECISION_VECTOR {}",
        serde_json::json!({
            "probeId": probe_id,
            "testId": std::thread::current().name().unwrap_or(""),
            "fixtureFile": file!(),
            "fixtureSource": include_str!("dynamic_classname_precision_probes.rs"),
            "actualAxes": actual,
            "expectedAxes": expected,
        })
    );
    assert_eq!(actual, expected, "emitted precision vector {probe_id}");
}

#[test]
fn dynamic_classname_keeps_context_depth() -> Result<(), String> {
    let sites = vec![OmenaQueryDynamicClassnameCallSiteV0 {
        callee_key: "classForVariant".to_string(),
        call_site_stack: vec!["render".to_string(), "className".to_string()],
        exit_value: OmenaQueryDynamicClassValueInputV0::Exact {
            value: "ghost".to_string(),
        },
        reference_range: ParserRangeV0::default(),
    }];
    let diagnostics =
        collect_omena_query_dynamic_classname_m_tier_diagnostics(&sites, &["card".to_string()], 2);
    let precision = diagnostics
        .first()
        .and_then(|diagnostic| diagnostic.precision.as_ref())
        .ok_or("dynamic classname precision absent")?;
    assert_emitted_axes(
        "dynamic-classname-keeps-context-depth",
        precision.axes,
        AnalysisPrecisionV1::from_axes_for_tests(
            ValueDomainPrecisionV1::ClassValueFlow,
            FlowPrecisionV1::KLimitedCallSiteFlow,
            ContextPrecisionV1::KLimitedCallSite,
            ProviderCompletenessV1::Complete,
            WorldAssumptionV1::Closed,
            RevisionIdentityV1::QuerySourceDiagnosticsInput,
        ),
    );
    Ok(())
}
