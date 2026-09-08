use super::*;

#[cfg(test)]
fn assert_emitted_axes(probe_id: &str, actual: AnalysisPrecisionV1, expected: AnalysisPrecisionV1) {
    eprintln!(
        "OMENA_PRECISION_VECTOR {}",
        serde_json::json!({
            "probeId": probe_id,
            "testId": std::thread::current().name().unwrap_or(""),
            "fixtureFile": file!(),
            "fixtureSource": include_str!("source_refs_precision_probes.rs"),
            "actualAxes": actual,
            "expectedAxes": expected,
        })
    );
    assert_eq!(actual, expected, "emitted precision vector {probe_id}");
}

#[test]
fn missing_selector_keeps_reference_context() -> Result<(), String> {
    let diagnostic = summarize_omena_query_missing_selector_diagnostic_with_insertion_range(
        "file:///workspace/Example.module.css",
        "",
        "ghost",
        ParserRangeV0::default(),
        ParserRangeV0::default(),
    );
    let precision = diagnostic
        .precision
        .ok_or("missing selector precision absent")?;
    assert_emitted_axes(
        "missing-selector-keeps-reference-context",
        precision.axes,
        AnalysisPrecisionV1::from_axes_for_tests(
            ValueDomainPrecisionV1::ClassValueResolution,
            FlowPrecisionV1::SourceSyntaxIndex,
            ContextPrecisionV1::PerSourceReference,
            ProviderCompletenessV1::Complete,
            WorldAssumptionV1::Closed,
            RevisionIdentityV1::QuerySourceDiagnosticsInput,
        ),
    );
    Ok(())
}

#[test]
fn global_class_fallthrough_keeps_open_world() -> Result<(), String> {
    let diagnostic = summarize_omena_query_global_class_fallthrough_diagnostic(
        "ghost",
        "file:///workspace/global.css",
        "file:///workspace/Example.module.css",
        "",
        ParserRangeV0::default(),
    );
    let precision = diagnostic.precision.ok_or("fallthrough precision absent")?;
    assert_emitted_axes(
        "global-class-fallthrough-keeps-open-world",
        precision.axes,
        AnalysisPrecisionV1::from_axes_for_tests(
            ValueDomainPrecisionV1::ClassValueResolution,
            FlowPrecisionV1::GlobalClassUniverse,
            ContextPrecisionV1::PerSourceReference,
            ProviderCompletenessV1::Complete,
            WorldAssumptionV1::Open,
            RevisionIdentityV1::QuerySourceDiagnosticsInput,
        ),
    );
    Ok(())
}

#[test]
fn missing_style_import_keeps_provider_unresolved() -> Result<(), String> {
    let resolution_inputs = OmenaQueryStyleResolutionInputsV0::default();
    let resolver_identity_index = build_omena_resolver_style_module_confirmation_identity_index(
        &BTreeSet::new(),
        resolution_inputs.disk_style_path_identities.as_slice(),
    );
    let report = summarize_omena_query_source_diagnostics_for_workspace_file_with_resolution_inputs_and_context_depth(
        "file:///workspace/Example.tsx", "import styles from './Missing.module.css';", &[], &[],
        &resolution_inputs, Some(&resolver_identity_index), 2,
    );
    let diagnostic = report
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code == "missing-module")
        .ok_or("missing module diagnostic absent")?;
    let precision = diagnostic
        .precision
        .as_ref()
        .ok_or("missing module precision absent")?;
    assert_emitted_axes(
        "missing-style-import-keeps-provider-unresolved",
        precision.axes,
        AnalysisPrecisionV1::from_axes_for_tests(
            ValueDomainPrecisionV1::StyleModuleResolution,
            FlowPrecisionV1::SourceImportResolution,
            ContextPrecisionV1::PerImportSpecifier,
            ProviderCompletenessV1::Unresolved,
            WorldAssumptionV1::Open,
            RevisionIdentityV1::QuerySourceDiagnosticsInput,
        ),
    );
    Ok(())
}

#[test]
fn unavailable_type_provider_keeps_provider_unresolved() -> Result<(), String> {
    let source = "unknown";
    let index = OmenaQuerySourceSyntaxIndexV0 {
        type_fact_provider_unavailable: vec![
            omena_bridge::SourceTypeFactProviderUnavailableFactV0 {
                byte_span: ParserByteSpanV0 {
                    start: 0,
                    end: source.len(),
                },
                expression_id: "missing".to_string(),
                target_style_uri: None,
                provider_id: "tsgo",
                reason: "unresolvable",
            },
        ],
        ..Default::default()
    };
    let diagnostics =
        summarize_omena_query_type_fact_provider_unavailable_diagnostics(source, &index);
    let precision = diagnostics
        .first()
        .and_then(|diagnostic| diagnostic.precision.as_ref())
        .ok_or("provider diagnostic precision absent")?;
    assert_emitted_axes(
        "unavailable-type-provider-keeps-provider-unresolved",
        precision.axes,
        AnalysisPrecisionV1::from_axes_for_tests(
            ValueDomainPrecisionV1::Unknown,
            FlowPrecisionV1::TypeOracleProviderUnavailable,
            ContextPrecisionV1::PerTypeFactTarget,
            ProviderCompletenessV1::Unresolved,
            WorldAssumptionV1::Open,
            RevisionIdentityV1::QuerySourceDiagnosticsInput,
        ),
    );
    Ok(())
}

#[test]
fn domain_class_reference_keeps_domain_context() -> Result<(), String> {
    let source = "ghost";
    let span = ParserByteSpanV0 {
        start: 0,
        end: source.len(),
    };
    let index = OmenaQuerySourceSyntaxIndexV0 {
        class_value_universes: vec![omena_bridge::SourceClassValueUniverseEntryV0 {
            plugin_id: "fixture",
            domain: "variant",
            owner_name: "card".to_string(),
            class_names: vec![],
            axes: vec![omena_bridge::SourceClassValueUniverseAxisV0 {
                axis_name: "size".to_string(),
                values: vec!["small".to_string()],
            }],
            patterns: vec![],
            unresolved: vec![],
            byte_span: span,
        }],
        domain_class_references: vec![omena_bridge::SourceDomainClassReferenceFactV0 {
            byte_span: span,
            plugin_id: "fixture",
            domain: "variant",
            owner_name: "card".to_string(),
            axis_name: "size".to_string(),
            option_name: Some("ghost".to_string()),
            prefix: None,
        }],
        ..Default::default()
    };
    let diagnostics = summarize_omena_query_domain_class_reference_diagnostics(source, &index);
    let precision = diagnostics
        .first()
        .and_then(|diagnostic| diagnostic.precision.as_ref())
        .ok_or("domain diagnostic precision absent")?;
    assert_emitted_axes(
        "domain-class-reference-keeps-domain-context",
        precision.axes,
        AnalysisPrecisionV1::from_axes_for_tests(
            ValueDomainPrecisionV1::ClassValueUniverse,
            FlowPrecisionV1::SourceDomainReference,
            ContextPrecisionV1::PerDomainAxis,
            ProviderCompletenessV1::Complete,
            WorldAssumptionV1::Closed,
            RevisionIdentityV1::QuerySourceDiagnosticsInput,
        ),
    );
    Ok(())
}

#[test]
fn unresolved_class_reference_keeps_reference_context() -> Result<(), String> {
    let source = "ghost";
    let reference = OmenaQuerySourceSelectorReferenceFactV0 {
        byte_span: ParserByteSpanV0 {
            start: 0,
            end: source.len(),
        },
        selector_name: None,
        match_kind: OmenaQuerySourceSelectorReferenceMatchKindV0::Exact,
        target_style_uri: None,
        surface: OmenaQuerySourceSelectorReferenceSurfaceV0::OmenaQuerySourceSyntaxIndex,
    };
    let diagnostic = summarize_omena_query_unresolved_source_reference_diagnostic(
        source,
        &omena_query_line_index(source),
        &reference,
        source,
        None,
        &[],
        0,
    );
    let precision = diagnostic
        .precision
        .ok_or("unresolved reference precision absent")?;
    assert_emitted_axes(
        "unresolved-class-reference-keeps-reference-context",
        precision.axes,
        AnalysisPrecisionV1::from_axes_for_tests(
            ValueDomainPrecisionV1::ClassValueResolution,
            FlowPrecisionV1::SourceSelectorReference,
            ContextPrecisionV1::PerSourceReference,
            ProviderCompletenessV1::Unresolved,
            WorldAssumptionV1::Open,
            RevisionIdentityV1::QuerySourceDiagnosticsInput,
        ),
    );
    Ok(())
}
