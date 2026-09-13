// Authored product fixtures and mutations. The accepted identities and fixture digests
// are independent of both the authority JSON and the compiled fixture's own envelope.
export interface PrecisionExerciseCase {
  readonly id: string;
  readonly pointId: string;
  readonly producerPath: string;
  readonly owningCrate: string;
  readonly fixtureFile: string;
  readonly fixtureSha256: string;
  readonly testPath: string;
  readonly mutation: {
    readonly id: string;
    readonly sourcePath: string;
    readonly from: string;
    readonly to: string;
  };
}

export const PRECISION_EXERCISE_CASES: readonly PrecisionExerciseCase[] = [
  {
    id: "witness-preserves-unresolved-axes",
    pointId:
      "analysisPrecisionConstructor:rust/crates/omena-abstract-value/src/domain.rs:apply_to:1",
    producerPath: "crate::domain::OmenaClosedWorldPrecisionWitnessV1::apply_to",
    owningCrate: "omena-abstract-value",
    fixtureFile: "rust/crates/omena-abstract-value/src/domain_precision_probes.rs",
    fixtureSha256: "cd19068ac1b9b0d3e9a6e8b4e2ad61d93644046f733114ba7a295d36b68a13d4",
    testPath: "domain::precision_probe_tests::witness_preserves_unresolved_axes",
    mutation: {
      id: "witness-preserves-unresolved-axes",
      sourcePath: "rust/crates/omena-abstract-value/src/domain.rs",
      from: "(ProviderCompletenessV1::Unresolved | ProviderCompletenessV1::Unknown, _)",
      to: "(ProviderCompletenessV1::Unknown, _)",
    },
  },
  {
    id: "unwitnessed-finite-set-keeps-open-world",
    pointId:
      "analysisPrecisionConstructor:rust/crates/omena-abstract-value/src/domain.rs:analysis_precision_from_class_value_with_witness:1",
    producerPath: "crate::domain::analysis_precision_from_class_value_with_witness",
    owningCrate: "omena-abstract-value",
    fixtureFile: "rust/crates/omena-abstract-value/src/domain_precision_probes.rs",
    fixtureSha256: "cd19068ac1b9b0d3e9a6e8b4e2ad61d93644046f733114ba7a295d36b68a13d4",
    testPath: "domain::precision_probe_tests::unwitnessed_finite_set_keeps_open_world",
    mutation: {
      id: "unwitnessed-finite-set-keeps-open-world",
      sourcePath: "rust/crates/omena-abstract-value/src/domain.rs",
      from: "AbstractClassValueV0::FiniteSet { .. } => (\n            ValueDomainPrecisionV1::ClosedClassValueSet,\n            ProviderCompletenessV1::Partial,\n            WorldAssumptionV1::Open,",
      to: "AbstractClassValueV0::FiniteSet { .. } => (\n            ValueDomainPrecisionV1::ClosedClassValueSet,\n            ProviderCompletenessV1::Partial,\n            WorldAssumptionV1::Closed,",
    },
  },
  {
    id: "incremental-flow-keeps-expression-context",
    pointId:
      "analysisPrecisionConstructor:rust/crates/omena-query-core/src/lib.rs:summarize_omena_query_expression_domain_incremental_flow_analysis_result:1",
    producerPath: "crate::summarize_omena_query_expression_domain_incremental_flow_analysis_result",
    owningCrate: "omena-query-core",
    fixtureFile: "rust/crates/omena-query-core/src/precision_probes.rs",
    fixtureSha256: "396f9b1346f83743145f41dcffef33a675198b9e716b1090d6a666fc58060e9f",
    testPath: "precision_probe_tests::incremental_flow_keeps_expression_context",
    mutation: {
      id: "incremental-flow-keeps-expression-context",
      sourcePath: "rust/crates/omena-query-core/src/lib.rs",
      from: "FlowPrecisionV1::from_dataflow_mode(complete_dataflow),\n                ContextPrecisionV1::PerExpressionGraph,",
      to: "FlowPrecisionV1::from_dataflow_mode(complete_dataflow),\n                ContextPrecisionV1::ContextInsensitive,",
    },
  },
  {
    id: "provider-without-target-keeps-unresolved-axes",
    pointId:
      "analysisPrecisionConstructor:rust/crates/omena-bridge/src/style_intelligence.rs:provider_analysis_precision:1",
    producerPath: "crate::style_intelligence::provider_analysis_precision",
    owningCrate: "omena-bridge",
    fixtureFile: "rust/crates/omena-bridge/src/style_intelligence_precision_probes.rs",
    fixtureSha256: "28e61974026b1f4284ab13bc4d1ca3c92c07febf8391e108b4ac7f5c3a7ddb86",
    testPath:
      "style_intelligence::precision_probe_tests::provider_without_target_keeps_unresolved_axes",
    mutation: {
      id: "provider-without-target-keeps-unresolved-axes",
      sourcePath: "rust/crates/omena-bridge/src/style_intelligence.rs",
      from: "let unresolved_provider_count = if import_target_count == 0 { 1 } else { 0 };",
      to: "let unresolved_provider_count = if import_target_count == 0 { 0 } else { 0 };",
    },
  },
  {
    id: "source-diagnostic-keeps-input-revision",
    pointId:
      "analysisPrecisionConstructor:rust/crates/omena-query/src/types.rs:source_diagnostic_precision:1",
    producerPath: "crate::types::source_diagnostic_precision",
    owningCrate: "omena-query",
    fixtureFile: "rust/crates/omena-query/src/precision_probes.rs",
    fixtureSha256: "90b17e066c8c391aa020b478879aa1d3a5f5f35655b06736b15b953be5ed6857",
    testPath: "precision_probe_tests::source_diagnostic_keeps_input_revision",
    mutation: {
      id: "source-diagnostic-keeps-input-revision",
      sourcePath: "rust/crates/omena-query/src/types.rs",
      from: "WorldAssumptionV1::from_closed_world(closed_world),\n        RevisionIdentityV1::QuerySourceDiagnosticsInput,",
      to: "WorldAssumptionV1::from_closed_world(closed_world),\n        RevisionIdentityV1::Current,",
    },
  },
  {
    id: "missing-source-capture-keeps-provider-unresolved",
    pointId:
      "sourceDiagnosticArgumentSite:rust/crates/omena-query/src/source.rs:resolve_omena_query_source_precision_for_source:1",
    producerPath: "crate::source::resolve_omena_query_source_precision_for_source",
    owningCrate: "omena-query",
    fixtureFile: "rust/crates/omena-query/src/precision_probes.rs",
    fixtureSha256: "90b17e066c8c391aa020b478879aa1d3a5f5f35655b06736b15b953be5ed6857",
    testPath: "precision_probe_tests::missing_source_capture_keeps_provider_unresolved",
    mutation: {
      id: "missing-source-capture-keeps-provider-unresolved",
      sourcePath: "rust/crates/omena-query/src/source.rs",
      from: "ContextPrecisionV1::SameFile,\n        usize::from(capture.is_none()),",
      to: "ContextPrecisionV1::SameFile,\n        0,",
    },
  },
  {
    id: "unresolved-cascade-keeps-provider-unresolved",
    pointId:
      "analysisPrecisionConstructor:rust/crates/omena-query/src/style/cascade_position.rs:cascade_at_position_analysis_result:1",
    producerPath: "crate::style::cascade_position::cascade_at_position_analysis_result",
    owningCrate: "omena-query",
    fixtureFile: "rust/crates/omena-query/src/style/cascade_precision_probes.rs",
    fixtureSha256: "da3fd0fd1d9b56723908f9eb9ceca411a51bc52d673b7956d68403017fa95a76",
    testPath:
      "style::cascade_position::precision_probe_tests::unresolved_cascade_keeps_provider_unresolved",
    mutation: {
      id: "unresolved-cascade-keeps-provider-unresolved",
      sourcePath: "rust/crates/omena-query/src/style/cascade_position.rs",
      from: 'let unresolved_provider_count = usize::from(value.status == "unresolved");',
      to: "let unresolved_provider_count = 0;",
    },
  },
  {
    id: "missing-selector-keeps-reference-context",
    pointId:
      "sourceDiagnosticArgumentSite:rust/crates/omena-query/src/style/source_refs.rs:summarize_omena_query_missing_selector_diagnostic_with_insertion_range:1",
    producerPath:
      "crate::style::source_refs::summarize_omena_query_missing_selector_diagnostic_with_insertion_range",
    owningCrate: "omena-query",
    fixtureFile: "rust/crates/omena-query/src/style/source_refs_precision_probes.rs",
    fixtureSha256: "4d904bc4ae0184ccf46a90b66e1df98901d8cfdfc560b1ff3069eeadae123bb3",
    testPath: "style::source_refs::precision_probe_tests::missing_selector_keeps_reference_context",
    mutation: {
      id: "missing-selector-keeps-reference-context",
      sourcePath: "rust/crates/omena-query/src/style/source_refs.rs",
      from: "FlowPrecisionV1::SourceSyntaxIndex,\n            ContextPrecisionV1::PerSourceReference,",
      to: "FlowPrecisionV1::SourceSyntaxIndex,\n            ContextPrecisionV1::ContextInsensitive,",
    },
  },
  {
    id: "global-class-fallthrough-keeps-open-world",
    pointId:
      "sourceDiagnosticArgumentSite:rust/crates/omena-query/src/style/source_refs.rs:summarize_omena_query_global_class_fallthrough_diagnostic:1",
    producerPath:
      "crate::style::source_refs::summarize_omena_query_global_class_fallthrough_diagnostic",
    owningCrate: "omena-query",
    fixtureFile: "rust/crates/omena-query/src/style/source_refs_precision_probes.rs",
    fixtureSha256: "4d904bc4ae0184ccf46a90b66e1df98901d8cfdfc560b1ff3069eeadae123bb3",
    testPath:
      "style::source_refs::precision_probe_tests::global_class_fallthrough_keeps_open_world",
    mutation: {
      id: "global-class-fallthrough-keeps-open-world",
      sourcePath: "rust/crates/omena-query/src/style/source_refs.rs",
      from: "FlowPrecisionV1::GlobalClassUniverse,\n            ContextPrecisionV1::PerSourceReference,\n            0,\n            false,",
      to: "FlowPrecisionV1::GlobalClassUniverse,\n            ContextPrecisionV1::PerSourceReference,\n            0,\n            true,",
    },
  },
  {
    id: "missing-style-import-keeps-provider-unresolved",
    pointId:
      "sourceDiagnosticArgumentSite:rust/crates/omena-query/src/style/source_refs.rs:summarize_omena_query_source_diagnostics_for_workspace_file_with_resolution_inputs_and_context_depth:1",
    producerPath:
      "crate::style::source_refs::summarize_omena_query_source_diagnostics_for_workspace_file_with_resolution_inputs_and_context_depth",
    owningCrate: "omena-query",
    fixtureFile: "rust/crates/omena-query/src/style/source_refs_precision_probes.rs",
    fixtureSha256: "4d904bc4ae0184ccf46a90b66e1df98901d8cfdfc560b1ff3069eeadae123bb3",
    testPath:
      "style::source_refs::precision_probe_tests::missing_style_import_keeps_provider_unresolved",
    mutation: {
      id: "missing-style-import-keeps-provider-unresolved",
      sourcePath: "rust/crates/omena-query/src/style/source_refs.rs",
      from: "FlowPrecisionV1::SourceImportResolution,\n                    ContextPrecisionV1::PerImportSpecifier,\n                    1,",
      to: "FlowPrecisionV1::SourceImportResolution,\n                    ContextPrecisionV1::PerImportSpecifier,\n                    0,",
    },
  },
  {
    id: "unavailable-type-provider-keeps-provider-unresolved",
    pointId:
      "sourceDiagnosticArgumentSite:rust/crates/omena-query/src/style/source_refs.rs:summarize_omena_query_type_fact_provider_unavailable_diagnostics:1",
    producerPath:
      "crate::style::source_refs::summarize_omena_query_type_fact_provider_unavailable_diagnostics",
    owningCrate: "omena-query",
    fixtureFile: "rust/crates/omena-query/src/style/source_refs_precision_probes.rs",
    fixtureSha256: "4d904bc4ae0184ccf46a90b66e1df98901d8cfdfc560b1ff3069eeadae123bb3",
    testPath:
      "style::source_refs::precision_probe_tests::unavailable_type_provider_keeps_provider_unresolved",
    mutation: {
      id: "unavailable-type-provider-keeps-provider-unresolved",
      sourcePath: "rust/crates/omena-query/src/style/source_refs.rs",
      from: "FlowPrecisionV1::TypeOracleProviderUnavailable,\n            ContextPrecisionV1::PerTypeFactTarget,\n            1,",
      to: "FlowPrecisionV1::TypeOracleProviderUnavailable,\n            ContextPrecisionV1::PerTypeFactTarget,\n            0,",
    },
  },
  {
    id: "domain-class-reference-keeps-domain-context",
    pointId:
      "sourceDiagnosticArgumentSite:rust/crates/omena-query/src/style/source_refs.rs:summarize_omena_query_domain_class_reference_diagnostics:1",
    producerPath:
      "crate::style::source_refs::summarize_omena_query_domain_class_reference_diagnostics",
    owningCrate: "omena-query",
    fixtureFile: "rust/crates/omena-query/src/style/source_refs_precision_probes.rs",
    fixtureSha256: "4d904bc4ae0184ccf46a90b66e1df98901d8cfdfc560b1ff3069eeadae123bb3",
    testPath:
      "style::source_refs::precision_probe_tests::domain_class_reference_keeps_domain_context",
    mutation: {
      id: "domain-class-reference-keeps-domain-context",
      sourcePath: "rust/crates/omena-query/src/style/source_refs.rs",
      from: "FlowPrecisionV1::SourceDomainReference,\n                ContextPrecisionV1::PerDomainAxis,",
      to: "FlowPrecisionV1::SourceDomainReference,\n                ContextPrecisionV1::ContextInsensitive,",
    },
  },
  {
    id: "unresolved-class-reference-keeps-reference-context",
    pointId:
      "sourceDiagnosticArgumentSite:rust/crates/omena-query/src/style/source_refs.rs:summarize_omena_query_unresolved_source_reference_diagnostic:1",
    producerPath:
      "crate::style::source_refs::summarize_omena_query_unresolved_source_reference_diagnostic",
    owningCrate: "omena-query",
    fixtureFile: "rust/crates/omena-query/src/style/source_refs_precision_probes.rs",
    fixtureSha256: "4d904bc4ae0184ccf46a90b66e1df98901d8cfdfc560b1ff3069eeadae123bb3",
    testPath:
      "style::source_refs::precision_probe_tests::unresolved_class_reference_keeps_reference_context",
    mutation: {
      id: "unresolved-class-reference-keeps-reference-context",
      sourcePath: "rust/crates/omena-query/src/style/source_refs.rs",
      from: "_ => ContextPrecisionV1::PerSourceReference,\n            },\n            usize::from(value_domain_size == 0),",
      to: "_ => ContextPrecisionV1::ContextInsensitive,\n            },\n            usize::from(value_domain_size == 0),",
    },
  },
  {
    id: "dynamic-classname-keeps-context-depth",
    pointId:
      "sourceDiagnosticArgumentSite:rust/crates/omena-query/src/style/dynamic_classname.rs:collect_omena_query_dynamic_classname_m_tier_diagnostics:1",
    producerPath:
      "crate::style::dynamic_classname::collect_omena_query_dynamic_classname_m_tier_diagnostics",
    owningCrate: "omena-query",
    fixtureFile: "rust/crates/omena-query/src/style/dynamic_classname_precision_probes.rs",
    fixtureSha256: "cb82c5113aabcd4ed511eaf88f2d267a9b749326752fd16746f0ea3f7bc60e89",
    testPath:
      "style::dynamic_classname::precision_probe_tests::dynamic_classname_keeps_context_depth",
    mutation: {
      id: "dynamic-classname-keeps-context-depth",
      sourcePath: "rust/crates/omena-query/src/style/dynamic_classname.rs",
      from: "FlowPrecisionV1::KLimitedCallSiteFlow,\n                        ContextPrecisionV1::from_max_context_depth(max_context_depth),",
      to: "FlowPrecisionV1::KLimitedCallSiteFlow,\n                        ContextPrecisionV1::ContextInsensitive,",
    },
  },
];

// This output loses the six axes when converted to FactPrecision. Exercising the
// aggregate value cannot certify the emitted vector without a separate product change.
export const PRECISION_UNOBSERVABLE_POINTS = [
  "analysisPrecisionConstructor:rust/crates/omena-query/src/style/diagnostics/source_usage.rs:summarize_omena_query_css_modules_export_usage:1",
] as const;
