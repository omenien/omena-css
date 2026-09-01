use crate::{
    ClassExpressionInputV2, EngineInputV2, OmenaQueryBundleEmissionPathV0,
    OmenaQueryBundlePlanInputV0, OmenaQueryClosedWorldBlockerV0, OmenaQueryClosedWorldOutcomeV0,
    OmenaQueryConsumerBuildOptionsV0, OmenaQueryExplainAvailabilityV0,
    OmenaQueryExplainFactValueV0, OmenaQueryExplainInputV0, OmenaQueryExplainSymbolKindV0,
    OmenaQueryModuleReachabilityAttributionReportV0, OmenaQueryStyleResolutionInputsV0,
    OmenaQueryStyleSourceInputV0, OmenaQueryTargetTransformOptionsV0,
    OmenaQueryTransformExecutionContextV0, OmenaQueryTransformStrictPolicyReasonV0, PositionV2,
    RangeV2, SourceAnalysisInputV2, SourceDocumentV2, StringTypeFactsV2, StyleAnalysisInputV2,
    StyleDocumentV2, StyleSelectorV2, TypeFactEntryV2,
    derive_omena_query_module_reachability_from_engine_input,
    execute_omena_query_consumer_build_style_source_with_context,
    execute_omena_query_consumer_build_style_source_with_engine_input_context,
    execute_omena_query_consumer_build_style_sources,
    execute_omena_query_consumer_build_style_sources_for_target_query_with_context_and_options,
    execute_omena_query_consumer_build_style_sources_with_context, explain_omena_query,
    explain_omena_query_tree_shake_for_module,
    run_omena_query_bundle_with_execution_scope_evidence_and_options,
    run_omena_query_bundle_with_module_reachability_and_execution_scope_evidence_and_options,
    run_omena_query_bundle_with_module_reachability_and_options,
    summarize_omena_query_expression_domain_selector_projection_with_precision,
};
use omena_parser::{ModuleIdV0, ModuleInstanceKeyV0};

#[allow(deprecated)]
fn legacy_bundle_options() -> OmenaQueryConsumerBuildOptionsV0 {
    OmenaQueryConsumerBuildOptionsV0 {
        bundle_emission_path: OmenaQueryBundleEmissionPathV0::legacy_compatibility(),
        ..OmenaQueryConsumerBuildOptionsV0::default()
    }
}

#[test]
fn consumer_build_inlines_transitive_workspace_imports() -> Result<(), Box<dyn std::error::Error>> {
    let sources = vec![
        OmenaQueryStyleSourceInputV0 {
            style_path: "/tmp/base.css".to_string(),
            style_source: ".base { color: red; }".to_string(),
        },
        OmenaQueryStyleSourceInputV0 {
            style_path: "/tmp/tokens.css".to_string(),
            style_source: r#"@import "./base.css"; .token { color: blue; }"#.to_string(),
        },
        OmenaQueryStyleSourceInputV0 {
            style_path: "/tmp/App.css".to_string(),
            style_source: r#"@import "./tokens.css"; .app { color: green; }"#.to_string(),
        },
    ];
    let summary = execute_omena_query_consumer_build_style_sources(
        "/tmp/App.css",
        &sources,
        &["import-inline".to_string(), "print-css".to_string()],
        &[],
    )?;

    assert_eq!(summary.product, "omena-query.consumer-build-style-source");
    assert_eq!(
        summary.execution.output_css,
        ".base { color: red; } .token { color: blue; } .app { color: green; }"
    );
    assert!(!summary.execution.output_css.contains("@import"));
    assert_eq!(summary.execution.mutation_count, 1);
    assert_eq!(
        summary.execution.css_import_inlines[0].replacement_css,
        ".base { color: red; } .token { color: blue; }"
    );
    Ok(())
}

#[test]
fn closed_world_request_open_world_downgrades_and_skips_tree_shake() {
    let summary = execute_omena_query_consumer_build_style_source_with_context(
        "Button.module.css",
        ".used { color: blue; } .dead { color: red; }",
        &["tree-shake-class".to_string()],
        &OmenaQueryTransformExecutionContextV0::default(),
    );

    assert!(summary.ready_surfaces.contains(&"openWorldSnapshot"));
    assert!(summary.open_world_snapshot.is_some());
    assert!(
        summary
            .open_world_snapshot
            .as_ref()
            .is_some_and(|snapshot| snapshot
                .reason()
                .contains("closed-world bundle unavailable"))
    );
    assert!(
        summary
            .execution
            .planned_only_pass_ids
            .contains(&"tree-shake-class")
    );
    assert!(
        !summary
            .execution
            .executed_pass_ids
            .contains(&"tree-shake-class")
    );
    assert_eq!(summary.semantic_removal_count, 0);
    assert!(summary.execution.output_css.contains(".dead"));
}

#[test]
fn closed_world_boundary_request_open_world_downgrades_and_skips_tree_shake() {
    let input = EngineInputV2 {
        version: "2".to_string(),
        sources: Vec::new(),
        styles: Vec::new(),
        type_facts: Vec::new(),
    };
    let summary = execute_omena_query_consumer_build_style_source_with_engine_input_context(
        "Button.module.css",
        ".used { color: blue; } .dead { color: red; }",
        &["tree-shake-class".to_string()],
        &input,
        true,
    );

    assert!(summary.ready_surfaces.contains(&"openWorldSnapshot"));
    assert!(summary.open_world_snapshot.is_some());
    assert!(
        summary
            .execution
            .planned_only_pass_ids
            .contains(&"tree-shake-class")
    );
    assert!(
        !summary
            .execution
            .executed_pass_ids
            .contains(&"tree-shake-class")
    );
    assert_eq!(summary.semantic_removal_count, 0);
    assert!(summary.execution.output_css.contains(".dead"));
    assert!(
        summary
            .ready_surfaces
            .contains(&"semanticReachabilityTransformContext")
    );
}

#[test]
fn workspace_bundle_failure_downgrades_without_context_reconstruction() {
    let sources = vec![OmenaQueryStyleSourceInputV0 {
        style_path: "Button.module.css".to_string(),
        style_source: r#"@import "./missing.css"; .used { color: blue; } .dead { color: red; }"#
            .to_string(),
    }];
    let context = OmenaQueryTransformExecutionContextV0 {
        reachable_class_names: vec!["used".to_string()],
        ..OmenaQueryTransformExecutionContextV0::default()
    };
    let summary_result = execute_omena_query_consumer_build_style_sources_with_context(
        "Button.module.css",
        &sources,
        &["tree-shake-class".to_string()],
        &context,
        &[],
    );
    assert!(summary_result.is_ok());
    let Ok(summary) = summary_result else {
        return;
    };

    assert!(summary.ready_surfaces.contains(&"openWorldSnapshot"));
    assert!(summary.open_world_snapshot.is_some());
    assert!(
        summary
            .execution
            .planned_only_pass_ids
            .contains(&"tree-shake-class")
    );
    assert!(
        !summary
            .execution
            .executed_pass_ids
            .contains(&"tree-shake-class")
    );
    assert_eq!(summary.semantic_removal_count, 0);
    assert!(summary.execution.output_css.contains(".dead"));
}

#[test]
fn consumer_build_executes_tree_shaking_with_context_closed_world_bundle() {
    let sources = vec![OmenaQueryStyleSourceInputV0 {
        style_path: "Button.module.css".to_string(),
        style_source: ".used { color: blue; } .dead { color: red; }".to_string(),
    }];
    let context = OmenaQueryTransformExecutionContextV0 {
        reachable_class_names: vec!["used".to_string()],
        ..OmenaQueryTransformExecutionContextV0::default()
    };
    let summary_result = execute_omena_query_consumer_build_style_sources_with_context(
        "Button.module.css",
        &sources,
        &["tree-shake-class".to_string()],
        &context,
        &[],
    );
    assert!(summary_result.is_ok());
    let Ok(summary) = summary_result else {
        return;
    };

    assert!(
        summary
            .execution
            .executed_pass_ids
            .contains(&"tree-shake-class")
    );
    assert!(summary.execution.planned_only_pass_ids.is_empty());
    assert_eq!(summary.semantic_removal_count, 1);
    assert!(!summary.execution.semantic_removals.is_empty());
    assert!(!summary.execution.output_css.contains(".dead"));
    assert!(summary.execution.output_css.contains(".used"));
}

#[test]
fn consumer_build_executes_composes_reachability_with_context_closed_world_bundle() {
    let sources = vec![OmenaQueryStyleSourceInputV0 {
        style_path: "Button.module.css".to_string(),
        style_source: r#".button { composes: base utility; color: red; } .base { color: blue; } .utility { animation: spin 1s; color: var(--brand); } .dead { color: black; } @keyframes spin { to { opacity: 1; } } @keyframes ghost { to { opacity: 0; } } :root { --brand: red; --dead: blue; }"#
            .to_string(),
    }];
    let context = OmenaQueryTransformExecutionContextV0 {
        reachable_class_names: vec!["button".to_string()],
        ..OmenaQueryTransformExecutionContextV0::default()
    };
    let summary_result = execute_omena_query_consumer_build_style_sources_with_context(
        "Button.module.css",
        &sources,
        &[
            "tree-shake-class".to_string(),
            "tree-shake-keyframes".to_string(),
            "tree-shake-custom-property".to_string(),
        ],
        &context,
        &[],
    );
    assert!(summary_result.is_ok());
    let Ok(summary) = summary_result else {
        return;
    };

    assert!(summary.execution.output_css.contains(".button"));
    assert!(summary.execution.output_css.contains(".base"));
    assert!(summary.execution.output_css.contains(".utility"));
    assert!(summary.execution.output_css.contains("@keyframes spin"));
    assert!(summary.execution.output_css.contains("--brand: red"));
    assert!(!summary.execution.output_css.contains(".dead"));
    assert!(!summary.execution.output_css.contains("@keyframes ghost"));
    assert!(!summary.execution.output_css.contains("--dead: blue"));
    assert!(summary.execution.planned_only_pass_ids.is_empty());
    assert!(!summary.execution.semantic_removals.is_empty());
}

#[test]
fn consumer_build_executes_value_tree_shaking_with_context_closed_world_bundle() {
    let sources = vec![OmenaQueryStyleSourceInputV0 {
        style_path: "Button.module.css".to_string(),
        style_source: r#"@value used: red; @value dead: blue; @value ghost: green; @keyframes pulse { to { color: used; } } @keyframes ghost { to { color: ghost; } } .button { animation: pulse 1s; }"#.to_string(),
    }];
    let context = OmenaQueryTransformExecutionContextV0 {
        reachable_class_names: vec!["button".to_string()],
        ..OmenaQueryTransformExecutionContextV0::default()
    };
    let summary_result = execute_omena_query_consumer_build_style_sources_with_context(
        "Button.module.css",
        &sources,
        &[
            "tree-shake-keyframes".to_string(),
            "tree-shake-value".to_string(),
        ],
        &context,
        &[],
    );
    assert!(summary_result.is_ok());
    let Ok(summary) = summary_result else {
        return;
    };

    assert!(summary.execution.output_css.contains("@value used: red;"));
    assert!(summary.execution.output_css.contains("color: used;"));
    assert!(!summary.execution.output_css.contains("@value dead:"));
    assert!(!summary.execution.output_css.contains("@value ghost:"));
    assert!(!summary.execution.output_css.contains("@keyframes ghost"));
    assert!(summary.execution.planned_only_pass_ids.is_empty());
    assert!(!summary.execution.semantic_removals.is_empty());
}

#[test]
fn consumer_build_executes_semantic_tree_shaking_with_context_closed_world_bundle() {
    let sources = vec![OmenaQueryStyleSourceInputV0 {
        style_path: "Button.module.css".to_string(),
        style_source: r#"@value liveValue: red; @value deadValue: orange; @keyframes liveSpin { to { opacity: 1; } } @keyframes deadSpin { to { opacity: 0; } } :root { --live: blue; --dead: gray; } .used { color: liveValue; border-color: var(--live); animation: liveSpin 1s; } .dead { color: deadValue; background: var(--dead); animation: deadSpin 1s; }"#
            .to_string(),
    }];
    let context = OmenaQueryTransformExecutionContextV0 {
        reachable_class_names: vec!["used".to_string()],
        ..OmenaQueryTransformExecutionContextV0::default()
    };
    let summary_result = execute_omena_query_consumer_build_style_sources_with_context(
        "Button.module.css",
        &sources,
        &[
            "tree-shake-keyframes".to_string(),
            "tree-shake-value".to_string(),
            "tree-shake-custom-property".to_string(),
        ],
        &context,
        &[],
    );
    assert!(summary_result.is_ok());
    let Ok(summary) = summary_result else {
        return;
    };

    assert!(summary.execution.output_css.contains("@value liveValue:"));
    assert!(summary.execution.output_css.contains("@keyframes liveSpin"));
    assert!(summary.execution.output_css.contains("--live: blue"));
    assert!(!summary.execution.output_css.contains("@value deadValue:"));
    assert!(!summary.execution.output_css.contains("@keyframes deadSpin"));
    assert!(!summary.execution.output_css.contains("--dead: gray"));
    assert!(summary.execution.output_css.contains(".dead"));
    assert!(summary.execution.planned_only_pass_ids.is_empty());
    assert!(!summary.execution.semantic_removals.is_empty());
}

#[test]
fn target_query_build_derives_workspace_context_for_bundle_passes() {
    let sources = vec![
        OmenaQueryStyleSourceInputV0 {
            style_path: "Button.module.css".to_string(),
            style_source:
                r#"@import "./tokens.css"; .button { direction: ltr; composes: base; margin-inline-start: 1rem; } .base { color: blue; }"#
                    .to_string(),
        },
        OmenaQueryStyleSourceInputV0 {
            style_path: "tokens.css".to_string(),
            style_source: ":root { --brand: red; }".to_string(),
        },
    ];
    let summary_result =
        execute_omena_query_consumer_build_style_sources_for_target_query_with_context_and_options(
            "Button.module.css",
            &sources,
            "ie 11",
            &OmenaQueryTransformExecutionContextV0::default(),
            OmenaQueryTargetTransformOptionsV0 {
                allow_logical_to_physical: true,
                allow_scope_flatten: false,
                allow_layer_flatten: false,
                enable_supports_static_eval: false,
                enable_media_static_eval: false,
                enable_container_static_eval: false,
                drop_dark_mode_media_queries: false,
            },
            &[],
        );
    assert!(summary_result.is_ok());
    let Ok(summary) = summary_result else {
        return;
    };

    assert!(
        summary
            .ready_surfaces
            .contains(&"multiSourceTransformContextProducer")
    );
    assert!(
        summary
            .execution
            .executed_pass_ids
            .contains(&"logical-to-physical")
    );
    assert!(!summary.execution.output_css.contains("@import"));
    assert!(!summary.execution.output_css.contains("composes:"));
    assert!(summary.execution.output_css.contains("margin-left"));
}

#[test]
fn acyclic_automaton_reachability_satisfies_the_tree_shake_precision_floor() -> Result<(), String> {
    let class_names = (0..12)
        .map(|index| format!("utility-{index:02}"))
        .collect::<Vec<_>>();
    let style_path = "Utilities.module.css";
    let style_source = class_names
        .iter()
        .map(|name| format!(".{name} {{ color: blue; }}"))
        .chain([".dead { color: red; }".to_string()])
        .collect::<Vec<_>>()
        .join(" ");
    let mut selector_names = class_names.clone();
    selector_names.push("dead".to_string());
    let input = EngineInputV2 {
        version: "2".to_string(),
        sources: vec![SourceAnalysisInputV2 {
            document: SourceDocumentV2 {
                class_expressions: vec![ClassExpressionInputV2 {
                    id: "utilities".to_string(),
                    kind: "symbolRef".to_string(),
                    scss_module_path: style_path.to_string(),
                    range: fixture_range(),
                    class_name: None,
                    root_binding_decl_id: Some("utilities-binding".to_string()),
                    access_path: None,
                }],
            },
        }],
        styles: vec![StyleAnalysisInputV2 {
            file_path: style_path.to_string(),
            source: Some(style_source.clone()),
            document: StyleDocumentV2 {
                selectors: selector_names
                    .iter()
                    .map(|name| StyleSelectorV2 {
                        name: name.clone(),
                        view_kind: "canonical".to_string(),
                        canonical_name: Some(name.clone()),
                        range: fixture_range(),
                        nested_safety: Some("safe".to_string()),
                        composes: None,
                        bem_suffix: None,
                    })
                    .collect(),
            },
        }],
        type_facts: vec![TypeFactEntryV2 {
            file_path: "Utilities.tsx".to_string(),
            expression_id: "utilities".to_string(),
            facts: StringTypeFactsV2 {
                kind: "finiteSet".to_string(),
                constraint_kind: None,
                values: Some(class_names.clone()),
                prefix: None,
                suffix: None,
                min_len: None,
                max_len: None,
                char_must: None,
                char_may: None,
                may_include_other_chars: None,
                provenance: None,
            },
            control_flow_graph: None,
        }],
    };

    let (_, precisions) =
        summarize_omena_query_expression_domain_selector_projection_with_precision(&input);
    let observed_precision = precisions
        .iter()
        .find(|entry| entry.node_id == "utilities")
        .map(|entry| omena_query_core::fact_precision_from_precision_axes(&entry.precision));
    assert_eq!(observed_precision, Some(crate::FactPrecision::Conservative));

    let summary = execute_omena_query_consumer_build_style_source_with_engine_input_context(
        style_path,
        &style_source,
        &["tree-shake-class".to_string()],
        &input,
        true,
    );
    assert!(
        summary
            .execution
            .executed_pass_ids
            .contains(&"tree-shake-class")
    );
    assert!(!summary.execution.output_css.contains(".dead"));
    for class_name in &class_names {
        assert!(
            summary
                .execution
                .output_css
                .contains(&format!(".{class_name}"))
        );
    }

    let calibration_report: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../omena-precision-calibration-report.json"
    ))
    .map_err(|error| format!("precision calibration report should be valid JSON: {error}"))?;
    let removed_class_names = summary
        .execution
        .semantic_removals
        .iter()
        .filter(|removal| removal.symbol_kind == "class")
        .map(|removal| removal.name.clone())
        .collect::<Vec<_>>();
    assert_eq!(
        calibration_report["cases"][0],
        serde_json::json!({
            "caseId": "acyclicAutomatonClassReachability",
            "inputClassCount": 12,
            "representation": "automaton",
            "witnessDirection": "supersetOfProducible",
            "witnessBasis": "acyclicExact",
            "previousPrecision": "heuristic",
            "currentPrecision": observed_precision,
            "closedWorldBundleAvailable": summary.ready_surfaces.contains(&"closedWorldBundle"),
            "requiredPrecision": "conservative",
            "previousOutcome": "blocked",
            "currentOutcome": "executed",
            "removedClassNames": removed_class_names,
            "retainedClassNames": class_names,
        })
    );
    Ok(())
}

#[test]
fn module_reachability_preserves_projection_union_without_flattening_ownership()
-> Result<(), String> {
    let entry_path = "src/entry.module.css";
    let dependency_path = "src/dependency.module.css";
    let entry_source =
        "@import \"./dependency.module.css\"; .shared { color: red; } .entry-dead { color: tan; }";
    let dependency_source = ".shared { padding: 8px; } .dependency-own { color: blue; } .dependency-dead { color: gray; }";
    let input = module_reachability_input(
        &[
            ("entry-ref", entry_path, "shared"),
            ("dependency-ref", dependency_path, "dependency-own"),
        ],
        &[
            (entry_path, entry_source, &["shared", "entry-dead"]),
            (
                dependency_path,
                dependency_source,
                &["shared", "dependency-own", "dependency-dead"],
            ),
        ],
    );
    let reachability =
        derive_omena_query_module_reachability_from_engine_input(&input, entry_path, true);

    assert_eq!(reachability.projection_summary_evaluation_count(), 1);
    assert_eq!(
        reachability.module_attribution(entry_path).class_names(),
        &["shared".to_string()]
    );
    assert_eq!(
        reachability
            .module_attribution(dependency_path)
            .class_names(),
        &["dependency-own".to_string()]
    );

    let style_sources = vec![
        OmenaQueryStyleSourceInputV0 {
            style_path: entry_path.to_string(),
            style_source: entry_source.to_string(),
        },
        OmenaQueryStyleSourceInputV0 {
            style_path: dependency_path.to_string(),
            style_source: dependency_source.to_string(),
        },
    ];
    let passes = vec!["tree-shake-class".to_string()];
    let context = OmenaQueryTransformExecutionContextV0::default();
    let resolution_inputs = OmenaQueryStyleResolutionInputsV0::default();
    let result =
        run_omena_query_bundle_with_module_reachability_and_execution_scope_evidence_and_options(
            OmenaQueryBundlePlanInputV0 {
                target_style_path: entry_path,
                style_sources: &style_sources,
                source_map_sources: &style_sources,
                requested_pass_ids: &passes,
                context: &context,
                resolution_inputs: &resolution_inputs,
                asset_rewrites: Vec::new(),
                bundle_entry_style_paths: &[],
            },
            &[],
            &OmenaQueryConsumerBuildOptionsV0 {
                bundle_emission_path: OmenaQueryBundleEmissionPathV0::LinkedOrder,
                ..OmenaQueryConsumerBuildOptionsV0::default()
            },
            &reachability,
        )?;
    let report = result
        .reachability_attribution
        .as_ref()
        .ok_or_else(|| "module-attributed run should retain reachability evidence".to_string())?;
    assert_eq!(report.projection_summary_evaluation_count(), 1);
    assert_eq!(
        report.projected_class_names(),
        report.attributed_class_names()
    );
    assert_eq!(report.flat_class_names(), report.attributed_class_names());
    assert!(report.lost_class_names().is_empty());
    assert!(report.unmatched_target_style_paths().is_empty());

    let bundle = result
        .bundle_result
        .closed_world_outcome
        .bundle()
        .ok_or_else(|| "module-attributed bundle should close".to_string())?;
    let entry = ModuleInstanceKeyV0::unconfigured(ModuleIdV0::new(entry_path));
    let dependency = ModuleInstanceKeyV0::unconfigured(ModuleIdV0::new(dependency_path));
    assert_eq!(
        bundle
            .reachability()
            .symbols_for_module(&entry)
            .ok_or_else(|| "entry module reachability should exist".to_string())?
            .class_names(),
        &["shared".to_string()]
    );
    let dependency_symbols = bundle
        .reachability()
        .symbols_for_module(&dependency)
        .ok_or_else(|| "dependency module reachability should exist".to_string())?;
    assert_eq!(
        dependency_symbols.class_names(),
        &["dependency-own".to_string()]
    );
    assert_ne!(
        dependency_symbols.class_names(),
        bundle.reachability().class_names()
    );
    assert_eq!(
        bundle.reachability().class_names(),
        &["dependency-own".to_string(), "shared".to_string()]
    );
    let output_css = &result.bundle_result.artifact.output_css;
    let flat_explanation = explain_omena_query(OmenaQueryExplainInputV0::TreeShake {
        bundle,
        symbol_kind: OmenaQueryExplainSymbolKindV0::Class,
        symbol_name: "shared",
    });
    assert!(matches!(
        flat_explanation.primary_fact().value(),
        OmenaQueryExplainFactValueV0::ReachabilityMembership { reachable: true }
    ));
    assert!(flat_explanation.supporting_facts().iter().any(|fact| {
        matches!(
            fact.value(),
            OmenaQueryExplainFactValueV0::ProvenanceLabel { label }
                if label == "moduleOwnershipUnobserved"
        )
    }));

    let dependency_shared = explain_omena_query_tree_shake_for_module(
        bundle,
        &dependency,
        OmenaQueryExplainSymbolKindV0::Class,
        "shared",
    );
    assert_eq!(dependency_shared.reachable(), Some(false));
    assert!(dependency_shared.flat_reachable());
    assert!(!dependency_shared.emission_guard_retained());
    assert_eq!(
        dependency_shared.ownership_digest(),
        bundle.module_qualified_ownership_digest()
    );
    assert_eq!(
        dependency_shared.provenance_labels(),
        &["moduleQualifiedOwnershipObserved"]
    );
    assert!(output_css.contains("padding: 8px"));

    let dependency_owned = explain_omena_query_tree_shake_for_module(
        bundle,
        &dependency,
        OmenaQueryExplainSymbolKindV0::Class,
        "dependency-own",
    );
    assert_eq!(dependency_owned.reachable(), Some(true));
    assert!(!dependency_owned.emission_guard_retained());
    assert!(output_css.contains("color: blue"));
    assert!(output_css.contains("color: tan"));
    assert!(output_css.contains("color: gray"));

    let execution_scope = result
        .execution_scope
        .as_ref()
        .ok_or_else(|| "linked-order run should retain bundle execution evidence".to_string())?;
    assert_eq!(
        execution_scope
            .bundle_execution
            .aggregate_closed_world_refusal_count,
        2
    );
    for module_execution in &execution_scope.bundle_execution.module_executions {
        assert!(matches!(
            module_execution
                .execution
                .closed_world_admission
                .refusal_reasons
                .as_slice(),
            [event]
                if event.module_instance.as_ref() == Some(&module_execution.module_instance)
                    && matches!(
                        event.reasons.as_slice(),
                        [OmenaQueryTransformStrictPolicyReasonV0::OwnershipNotSeparable {
                            token,
                            module_paths,
                        }] if token == "shared" && module_paths == &[
                            dependency_path.to_string(),
                            entry_path.to_string(),
                        ]
                    )
        ));
    }

    let unknown = explain_omena_query_tree_shake_for_module(
        bundle,
        &ModuleInstanceKeyV0::unconfigured(ModuleIdV0::new("src/unknown.module.css")),
        OmenaQueryExplainSymbolKindV0::Class,
        "shared",
    );
    assert_eq!(
        unknown.availability(),
        OmenaQueryExplainAvailabilityV0::NotFound
    );
    assert_eq!(unknown.reachable(), None);
    assert!(!unknown.emission_guard_retained());
    Ok(())
}

#[test]
fn workspace_reachability_outside_bundle_sources_does_not_block_entry_build() -> Result<(), String>
{
    let entry_path = "src/a.module.css";
    let outside_path = "src/b.module.css";
    let entry_source = ".a-live { padding: 1px; } .a-dead { padding: 2px; }";
    let outside_source = ".b-live { color: blue; }";
    let input = module_reachability_input(
        &[
            ("entry-ref", entry_path, "a-live"),
            ("outside-ref", outside_path, "b-live"),
        ],
        &[
            (entry_path, entry_source, &["a-live", "a-dead"]),
            (outside_path, outside_source, &["b-live"]),
        ],
    );
    let reachability =
        derive_omena_query_module_reachability_from_engine_input(&input, entry_path, true);
    let style_sources = vec![OmenaQueryStyleSourceInputV0 {
        style_path: entry_path.to_string(),
        style_source: entry_source.to_string(),
    }];
    let result = run_omena_query_bundle_with_module_reachability_and_options(
        OmenaQueryBundlePlanInputV0 {
            target_style_path: entry_path,
            style_sources: &style_sources,
            source_map_sources: &style_sources,
            requested_pass_ids: &["tree-shake-class".to_string()],
            context: &OmenaQueryTransformExecutionContextV0::default(),
            resolution_inputs: &OmenaQueryStyleResolutionInputsV0::default(),
            asset_rewrites: Vec::new(),
            bundle_entry_style_paths: &[],
        },
        &[],
        &OmenaQueryConsumerBuildOptionsV0::default(),
        &reachability,
    )?;

    assert_eq!(
        result.reachability_attribution().flat_class_names(),
        &["a-live".to_string()]
    );
    assert!(
        result
            .reachability_attribution()
            .lost_class_names()
            .is_empty()
    );
    assert!(
        result
            .bundle_result()
            .artifact
            .output_css
            .contains("padding: 1px")
    );
    assert!(
        !result
            .bundle_result()
            .artifact
            .output_css
            .contains("padding: 2px")
    );
    Ok(())
}

#[test]
fn workspace_reachability_does_not_hide_missing_dependency_blocker() -> Result<(), String> {
    let entry_path = "src/entry.module.css";
    let missing_path = "src/missing.module.css";
    let entry_source =
        "@import \"./missing.module.css\"; .entry-live { color: red; } .entry-dead { color: tan; }";
    let missing_source = ".dependency-live { color: blue; }";
    let input = module_reachability_input(
        &[
            ("entry-ref", entry_path, "entry-live"),
            ("dependency-ref", missing_path, "dependency-live"),
        ],
        &[
            (entry_path, entry_source, &["entry-live", "entry-dead"]),
            (missing_path, missing_source, &["dependency-live"]),
        ],
    );
    let reachability =
        derive_omena_query_module_reachability_from_engine_input(&input, entry_path, true);
    let style_sources = vec![OmenaQueryStyleSourceInputV0 {
        style_path: entry_path.to_string(),
        style_source: entry_source.to_string(),
    }];
    let result = run_omena_query_bundle_with_module_reachability_and_options(
        OmenaQueryBundlePlanInputV0 {
            target_style_path: entry_path,
            style_sources: &style_sources,
            source_map_sources: &style_sources,
            requested_pass_ids: &["tree-shake-class".to_string()],
            context: &OmenaQueryTransformExecutionContextV0::default(),
            resolution_inputs: &OmenaQueryStyleResolutionInputsV0::default(),
            asset_rewrites: Vec::new(),
            bundle_entry_style_paths: &[],
        },
        &[],
        &legacy_bundle_options(),
        &reachability,
    )?;

    assert!(matches!(
        &result.bundle_result().closed_world_outcome,
        OmenaQueryClosedWorldOutcomeV0::Open { blockers }
            if blockers.iter().any(|blocker| matches!(
                blocker,
                OmenaQueryClosedWorldBlockerV0::MissingDependency {
                    source_path,
                    import_source,
                } if source_path == entry_path && import_source == "./missing.module.css"
            ))
    ));
    assert!(
        result
            .bundle_result()
            .artifact
            .output_css
            .contains("entry-dead")
    );
    Ok(())
}

#[test]
fn synthetic_attribution_input_is_excluded_by_the_product_domain() {
    let entry_path = "src/entry.module.css";
    let outside_path = "src/outside.module.css";
    let input = module_reachability_input(
        &[("outside-ref", outside_path, "outside-live")],
        &[
            (entry_path, ".entry { color: red; }", &["entry"]),
            (
                outside_path,
                ".outside-live { color: blue; }",
                &["outside-live"],
            ),
        ],
    );
    let reachability =
        derive_omena_query_module_reachability_from_engine_input(&input, entry_path, true);
    assert!(
        reachability
            .flat_class_names_for_style_paths([entry_path], &["outside-live".to_string()])
            .is_empty(),
        "the product domain filter must not manufacture this synthetic input"
    );
}

#[test]
fn production_attribution_domains_assign_every_admitted_name() -> Result<(), String> {
    let cases = [
        (
            "normalized duplicate paths",
            module_reachability_input(
                &[("entry-ref", "src/entry.module.css", "entry-live")],
                &[
                    ("src/entry.module.css", ".entry-live {}", &["entry-live"]),
                    ("src/./entry.module.css", ".entry-live {}", &["entry-live"]),
                ],
            ),
            "src/entry.module.css",
            vec!["src/entry.module.css"],
        ),
        (
            "ambiguous suffix",
            module_reachability_input(
                &[
                    (
                        "first-ref",
                        "/workspace/first/Button.module.css",
                        "first-live",
                    ),
                    (
                        "second-ref",
                        "/workspace/second/Button.module.css",
                        "second-live",
                    ),
                ],
                &[
                    (
                        "/workspace/first/Button.module.css",
                        ".first-live {}",
                        &["first-live"],
                    ),
                    (
                        "/workspace/second/Button.module.css",
                        ".second-live {}",
                        &["second-live"],
                    ),
                ],
            ),
            "/workspace/first/Button.module.css",
            vec!["Button.module.css"],
        ),
        (
            "case-insensitive owner",
            module_reachability_input(
                &[("entry-ref", "src/entry.module.css", "entry-live")],
                &[("SRC/Entry.module.css", ".entry-live {}", &["entry-live"])],
            ),
            "src/entry.module.css",
            vec!["src/entry.module.css"],
        ),
        (
            "reference without a declared owner",
            module_reachability_input(
                &[("missing-ref", "src/missing.module.css", "missing-live")],
                &[("src/entry.module.css", ".entry-live {}", &["entry-live"])],
            ),
            "src/entry.module.css",
            vec!["src/entry.module.css"],
        ),
    ];

    for (label, input, target_style_path, build_style_paths) in cases {
        let reachability = derive_omena_query_module_reachability_from_engine_input(
            &input,
            target_style_path,
            true,
        );
        let flat_class_names = reachability.flat_class_names_for_style_paths(
            build_style_paths.iter().copied(),
            reachability.projected_class_names(),
        );
        let report = OmenaQueryModuleReachabilityAttributionReportV0::from_style_paths(
            &reachability,
            build_style_paths.iter().copied(),
            flat_class_names.as_slice(),
        );
        assert!(
            report.lost_class_names().is_empty(),
            "{label} left admitted names without an attribution entry"
        );
        assert_eq!(
            report.flat_class_names(),
            report.attributed_class_names(),
            "{label} changed the admitted-name set during placement"
        );
    }

    Ok(())
}

#[test]
fn equivalent_class_spellings_have_one_attribution_identity() {
    let style_path = "src/entry.module.css";
    let input = module_reachability_input(
        &[("entry-ref", style_path, "card")],
        &[(style_path, r#".c\61 rd {}"#, &[r#"c\61 rd"#])],
    );
    let reachability =
        derive_omena_query_module_reachability_from_engine_input(&input, style_path, true);
    let flat_class_names =
        reachability.flat_class_names_for_style_paths([style_path], &["card".to_string()]);
    let report = OmenaQueryModuleReachabilityAttributionReportV0::from_style_paths(
        &reachability,
        [style_path],
        flat_class_names.as_slice(),
    );

    assert!(report.lost_class_names().is_empty());
    assert_eq!(report.attributed_class_names().len(), 1);
    assert!(
        omena_syntax::ident::ClassNameV0::new(&report.attributed_class_names()[0])
            .same_as(&omena_syntax::ident::ClassNameV0::new("card"))
    );
}

#[test]
fn ambiguous_build_path_keeps_every_matching_owner_in_the_attribution_domain() {
    let first_style_path = "/workspace/first/Button.module.css";
    let second_style_path = "/workspace/second/Button.module.css";
    let input = module_reachability_input(
        &[
            ("first-ref", first_style_path, "first-live"),
            ("second-ref", second_style_path, "second-live"),
        ],
        &[
            (first_style_path, ".first-live {}", &["first-live"]),
            (second_style_path, ".second-live {}", &["second-live"]),
        ],
    );
    let reachability =
        derive_omena_query_module_reachability_from_engine_input(&input, first_style_path, true);

    assert_eq!(
        reachability.flat_class_names_for_style_paths(
            ["Button.module.css"],
            &["first-live".to_string(), "second-live".to_string()],
        ),
        vec!["first-live".to_string(), "second-live".to_string()]
    );
}

#[test]
fn missing_target_source_precedes_attribution_domain_validation() {
    let entry_path = "src/entry.module.css";
    let input = module_reachability_input(
        &[("entry-ref", entry_path, "entry-live")],
        &[(entry_path, ".entry-live {}", &["entry-live"])],
    );
    let reachability =
        derive_omena_query_module_reachability_from_engine_input(&input, entry_path, true);
    let result = run_omena_query_bundle_with_module_reachability_and_options(
        OmenaQueryBundlePlanInputV0 {
            target_style_path: entry_path,
            style_sources: &[],
            source_map_sources: &[],
            requested_pass_ids: &["tree-shake-class".to_string()],
            context: &OmenaQueryTransformExecutionContextV0::default(),
            resolution_inputs: &OmenaQueryStyleResolutionInputsV0::default(),
            asset_rewrites: Vec::new(),
            bundle_entry_style_paths: &[],
        },
        &[],
        &OmenaQueryConsumerBuildOptionsV0::default(),
        &reachability,
    );

    assert_eq!(
        result,
        Err(
            "module-attributed bundle target style path \"src/entry.module.css\" was not found in workspace style sources"
                .to_string()
        )
    );
}

#[test]
fn composes_closure_is_partitioned_to_the_declaring_module() -> Result<(), String> {
    let entry_path = "src/entry.module.css";
    let base_path = "src/base.module.css";
    let entry_source = "@import \"./base.module.css\"; \
        .card { composes: base from \"./base.module.css\"; color: red; }";
    let base_source =
        ".base { padding: 2px; } .base-live { color: blue; } .base-dead { color: gray; }";
    let input = module_reachability_input(
        &[
            ("entry-card", entry_path, "card"),
            ("base-live", base_path, "base-live"),
        ],
        &[
            (entry_path, entry_source, &["card"]),
            (base_path, base_source, &["base", "base-live", "base-dead"]),
        ],
    );
    let reachability =
        derive_omena_query_module_reachability_from_engine_input(&input, entry_path, true);
    let style_sources = vec![
        OmenaQueryStyleSourceInputV0 {
            style_path: entry_path.to_string(),
            style_source: entry_source.to_string(),
        },
        OmenaQueryStyleSourceInputV0 {
            style_path: base_path.to_string(),
            style_source: base_source.to_string(),
        },
    ];
    let result = run_omena_query_bundle_with_module_reachability_and_options(
        OmenaQueryBundlePlanInputV0 {
            target_style_path: entry_path,
            style_sources: &style_sources,
            source_map_sources: &style_sources,
            requested_pass_ids: &["tree-shake-class".to_string()],
            context: &OmenaQueryTransformExecutionContextV0::default(),
            resolution_inputs: &OmenaQueryStyleResolutionInputsV0::default(),
            asset_rewrites: Vec::new(),
            bundle_entry_style_paths: &[],
        },
        &[],
        &OmenaQueryConsumerBuildOptionsV0 {
            bundle_emission_path: OmenaQueryBundleEmissionPathV0::LinkedOrder,
            ..OmenaQueryConsumerBuildOptionsV0::default()
        },
        &reachability,
    )?;

    let report = result.reachability_attribution();
    assert_eq!(
        report.flat_class_names(),
        &[
            "base".to_string(),
            "base-live".to_string(),
            "card".to_string()
        ]
    );
    assert_eq!(report.flat_class_names(), report.attributed_class_names());
    assert!(report.lost_class_names().is_empty());
    assert_eq!(
        report
            .entry_for_style_path(entry_path)
            .ok_or_else(|| "entry attribution should exist".to_string())?
            .class_names(),
        &["card".to_string()]
    );
    assert_eq!(
        report
            .entry_for_style_path(base_path)
            .ok_or_else(|| "base attribution should exist".to_string())?
            .class_names(),
        &["base".to_string(), "base-live".to_string()]
    );
    let output_css = &result.bundle_result().artifact.output_css;
    assert!(output_css.contains("padding: 2px"));
    assert!(output_css.contains("color: blue"));
    assert!(!output_css.contains("color: gray"));
    Ok(())
}

#[test]
fn composes_closure_survives_equivalent_engine_and_build_path_forms() -> Result<(), String> {
    let engine_entry_path = "/workspace/src/entry.module.css";
    let engine_base_path = "/workspace/src/base.module.css";
    let build_entry_path = "src/entry.module.css";
    let build_base_path = "src/base.module.css";
    let entry_source = "@import \"./base.module.css\"; \
        .card { composes: base from \"./base.module.css\"; color: red; }";
    let base_source =
        ".base { padding: 2px; } .base-live { color: blue; } .base-dead { color: gray; }";
    let input = module_reachability_input(
        &[
            ("entry-card", engine_entry_path, "card"),
            ("base-live", engine_base_path, "base-live"),
        ],
        &[
            (engine_entry_path, entry_source, &["card"]),
            (
                engine_base_path,
                base_source,
                &["base", "base-live", "base-dead"],
            ),
        ],
    );
    let reachability =
        derive_omena_query_module_reachability_from_engine_input(&input, build_entry_path, true);
    let style_sources = vec![
        OmenaQueryStyleSourceInputV0 {
            style_path: build_entry_path.to_string(),
            style_source: entry_source.to_string(),
        },
        OmenaQueryStyleSourceInputV0 {
            style_path: build_base_path.to_string(),
            style_source: base_source.to_string(),
        },
    ];
    let result = run_omena_query_bundle_with_module_reachability_and_options(
        OmenaQueryBundlePlanInputV0 {
            target_style_path: build_entry_path,
            style_sources: &style_sources,
            source_map_sources: &style_sources,
            requested_pass_ids: &["tree-shake-class".to_string()],
            context: &OmenaQueryTransformExecutionContextV0::default(),
            resolution_inputs: &OmenaQueryStyleResolutionInputsV0::default(),
            asset_rewrites: Vec::new(),
            bundle_entry_style_paths: &[],
        },
        &[],
        &OmenaQueryConsumerBuildOptionsV0 {
            bundle_emission_path: OmenaQueryBundleEmissionPathV0::LinkedOrder,
            ..OmenaQueryConsumerBuildOptionsV0::default()
        },
        &reachability,
    )?;

    assert!(
        result
            .bundle_result()
            .artifact
            .output_css
            .contains("padding: 2px")
    );
    assert!(
        result
            .reachability_attribution()
            .unmatched_target_style_paths()
            .is_empty()
    );
    Ok(())
}

#[test]
fn unattributed_projection_fans_out_to_every_style_module() {
    let first_path = "src/first.module.css";
    let second_path = "src/second.module.css";
    let input = EngineInputV2 {
        version: "2".to_string(),
        sources: Vec::new(),
        styles: vec![
            style_input(first_path, ".shared { color: red; }", &["shared"]),
            style_input(second_path, ".shared { color: blue; }", &["shared"]),
        ],
        type_facts: vec![exact_type_fact("unattributed-ref", "shared")],
    };
    let reachability =
        derive_omena_query_module_reachability_from_engine_input(&input, first_path, true);

    for style_path in [first_path, second_path] {
        let attribution = reachability.module_attribution(style_path);
        assert!(attribution.was_attempted());
        assert_eq!(attribution.targeted_projection_count(), 0);
        assert!(attribution.unattributed_projection_count() > 0);
        assert_eq!(attribution.class_names(), &["shared".to_string()]);
    }
}

#[test]
fn module_reachability_resolves_equivalent_style_path_spellings() {
    let style_path = "/workspace/src/Button.module.css";
    let input_style_path = "/workspace/src/nested/../Button.module.css";
    let input = module_reachability_input(
        &[
            (
                "current-directory-ref",
                "/workspace/src/./Button.module.css",
                "current-directory",
            ),
            (
                "parent-directory-ref",
                "/workspace/src/nested/../Button.module.css",
                "parent-directory",
            ),
            (
                "case-insensitive-ref",
                "/WORKSPACE/SRC/BUTTON.MODULE.CSS",
                "case-insensitive",
            ),
            (
                "workspace-relative-ref",
                "src/Button.module.css",
                "workspace-relative",
            ),
        ],
        &[(
            input_style_path,
            ".current-directory {} .parent-directory {} .case-insensitive {} .workspace-relative {}",
            &[
                "current-directory",
                "parent-directory",
                "case-insensitive",
                "workspace-relative",
            ],
        )],
    );
    let reachability =
        derive_omena_query_module_reachability_from_engine_input(&input, style_path, true);
    let attribution = reachability.module_attribution(style_path);

    assert_eq!(
        attribution.class_names(),
        &[
            "case-insensitive".to_string(),
            "current-directory".to_string(),
            "parent-directory".to_string(),
            "workspace-relative".to_string(),
        ]
    );
    assert_eq!(attribution.targeted_projection_count(), 4);
    assert_eq!(attribution.unattributed_projection_count(), 0);
}

#[test]
fn ambiguous_relative_style_path_fans_out_instead_of_choosing_an_owner() {
    let first_style_path = "/workspace/first/Button.module.css";
    let second_style_path = "/workspace/second/Button.module.css";
    let input = module_reachability_input(
        &[("ambiguous-ref", "Button.module.css", "shared")],
        &[
            (first_style_path, ".shared {}", &["shared"]),
            (second_style_path, ".shared {}", &["shared"]),
        ],
    );
    let reachability =
        derive_omena_query_module_reachability_from_engine_input(&input, first_style_path, true);

    for style_path in [first_style_path, second_style_path] {
        let attribution = reachability.module_attribution(style_path);
        assert_eq!(attribution.class_names(), &["shared".to_string()]);
        assert_eq!(attribution.unattributed_projection_count(), 1);
    }
}

#[test]
fn attributed_empty_projection_removes_unreachable_parse_derived_names() -> Result<(), String> {
    let fixtures: &[(&str, &str, &str, &[&str])] = &[
        (
            "single-class",
            "src/retained.module.css",
            ".retained { color: red; }",
            &["retained"],
        ),
        (
            "two-classes",
            "src/pair.module.css",
            ".first { color: red; } .second { color: blue; }",
            &["first", "second"],
        ),
    ];

    for (fixture_id, style_path, style_source, selector_names) in fixtures {
        let input = module_reachability_input(
            &[("missing-ref", style_path, "missing")],
            &[(style_path, style_source, selector_names)],
        );
        let reachability =
            derive_omena_query_module_reachability_from_engine_input(&input, style_path, true);
        let attribution = reachability.module_attribution(style_path);
        assert!(attribution.was_attempted());
        assert!(attribution.class_names().is_empty());

        let style_sources = vec![OmenaQueryStyleSourceInputV0 {
            style_path: (*style_path).to_string(),
            style_source: (*style_source).to_string(),
        }];
        let passes = vec!["tree-shake-class".to_string()];
        let context = OmenaQueryTransformExecutionContextV0::default();
        let resolution_inputs = OmenaQueryStyleResolutionInputsV0::default();
        let input = || OmenaQueryBundlePlanInputV0 {
            target_style_path: style_path,
            style_sources: &style_sources,
            source_map_sources: &style_sources,
            requested_pass_ids: &passes,
            context: &context,
            resolution_inputs: &resolution_inputs,
            asset_rewrites: Vec::new(),
            bundle_entry_style_paths: &[],
        };
        let options = OmenaQueryConsumerBuildOptionsV0 {
            bundle_emission_path: OmenaQueryBundleEmissionPathV0::LinkedOrder,
            ..OmenaQueryConsumerBuildOptionsV0::default()
        };
        let conservative = run_omena_query_bundle_with_execution_scope_evidence_and_options(
            input(),
            &[],
            &options,
        )?;
        let analyzed = run_omena_query_bundle_with_module_reachability_and_options(
            input(),
            &[],
            &options,
            &reachability,
        )?;
        let conservative_css = &conservative.bundle_result.artifact.output_css;
        let analyzed_css = &analyzed.bundle_result().artifact.output_css;

        assert_eq!(
            analyzed
                .reachability_attribution()
                .attributed_empty_module_count(),
            1
        );
        assert!(
            analyzed
                .bundle_result()
                .closed_world_outcome
                .bundle()
                .is_some()
        );
        assert!(
            conservative
                .bundle_result
                .closed_world_outcome
                .bundle()
                .is_some()
        );
        assert_eq!(conservative_css, style_source);
        assert_ne!(analyzed_css, conservative_css);
        assert!(
            selector_names
                .iter()
                .all(|name| !analyzed_css.contains(name))
        );
        assert!(
            analyzed
                .bundle_result()
                .artifact
                .execution
                .semantic_removals
                .len()
                >= selector_names.len()
        );
        assert!(
            conservative
                .bundle_result
                .artifact
                .execution
                .semantic_removals
                .is_empty()
        );

        eprintln!(
            "REACHABILITY_CORPUS_CELL={}",
            serde_json::json!({
                "fixtureId": fixture_id,
                "state": "analyzed",
                "cause": null,
                "closedWorldOutcome": "closed",
                "semanticRemovalCount": analyzed
                    .bundle_result()
                    .artifact
                    .execution
                    .semantic_removals
                    .len(),
                "outputCss": analyzed_css,
                "productBytesEqualToConservative": analyzed_css == conservative_css,
            })
        );
        eprintln!(
            "REACHABILITY_CORPUS_CELL={}",
            serde_json::json!({
                "fixtureId": fixture_id,
                "state": "unanalyzed",
                "cause": "inputNotProvided",
                "closedWorldOutcome": "closed",
                "semanticRemovalCount": conservative
                    .bundle_result
                    .artifact
                    .execution
                    .semantic_removals
                    .len(),
                "outputCss": conservative_css,
                "productBytesEqualToConservative": true,
            })
        );
    }
    Ok(())
}

#[test]
#[cfg(feature = "test-support")]
fn module_reachability_producers_are_hoisted_for_two_module_bundle() -> Result<(), String> {
    assert_module_reachability_producer_counts(
        "src/entry.module.css",
        &[
            (
                "src/entry.module.css",
                "@import \"./dependency.module.css\"; .shared { color: red; }",
                &["shared"],
            ),
            (
                "src/dependency.module.css",
                ".dependency-own { color: blue; }",
                &["dependency-own"],
            ),
        ],
        &[
            ("entry-ref", "src/entry.module.css", "shared"),
            (
                "dependency-ref",
                "src/dependency.module.css",
                "dependency-own",
            ),
        ],
    )
}

#[test]
#[cfg(feature = "test-support")]
fn module_reachability_producers_are_hoisted_for_three_module_bundle() -> Result<(), String> {
    assert_module_reachability_producer_counts(
        "src/entry.module.css",
        &[
            (
                "src/entry.module.css",
                "@import \"./middle.module.css\"; .entry { color: red; }",
                &["entry"],
            ),
            (
                "src/middle.module.css",
                "@import \"./leaf.module.css\"; .middle { color: blue; }",
                &["middle"],
            ),
            ("src/leaf.module.css", ".leaf { color: green; }", &["leaf"]),
        ],
        &[
            ("entry-ref", "src/entry.module.css", "entry"),
            ("middle-ref", "src/middle.module.css", "middle"),
            ("leaf-ref", "src/leaf.module.css", "leaf"),
        ],
    )
}

#[cfg(feature = "test-support")]
fn assert_module_reachability_producer_counts(
    entry_path: &str,
    styles: &[(&str, &str, &[&str])],
    references: &[(&str, &str, &str)],
) -> Result<(), String> {
    let input = module_reachability_input(references, styles);
    let style_sources = styles
        .iter()
        .map(
            |(style_path, style_source, _)| OmenaQueryStyleSourceInputV0 {
                style_path: (*style_path).to_string(),
                style_source: (*style_source).to_string(),
            },
        )
        .collect::<Vec<_>>();
    let passes = vec!["tree-shake-class".to_string()];
    let context = OmenaQueryTransformExecutionContextV0::default();
    let resolution_inputs = OmenaQueryStyleResolutionInputsV0::default();
    omena_query_core::reset_selector_projection_evaluation_count_for_test();
    omena_parser::reset_closed_world_bundle_construction_count_for_test();
    let reachability =
        derive_omena_query_module_reachability_from_engine_input(&input, entry_path, true);
    let result = run_omena_query_bundle_with_module_reachability_and_options(
        OmenaQueryBundlePlanInputV0 {
            target_style_path: entry_path,
            style_sources: &style_sources,
            source_map_sources: &style_sources,
            requested_pass_ids: &passes,
            context: &context,
            resolution_inputs: &resolution_inputs,
            asset_rewrites: Vec::new(),
            bundle_entry_style_paths: &[],
        },
        &[],
        &OmenaQueryConsumerBuildOptionsV0 {
            bundle_emission_path: OmenaQueryBundleEmissionPathV0::LinkedOrder,
            ..OmenaQueryConsumerBuildOptionsV0::default()
        },
        &reachability,
    )?;

    let projection_summary_evaluation_count =
        omena_query_core::selector_projection_evaluation_count_for_test();
    let closed_world_bundle_construction_count =
        omena_parser::closed_world_bundle_construction_count_for_test();
    assert_eq!(
        projection_summary_evaluation_count, 1,
        "selector projection summary must be produced once per bundle run"
    );
    assert_eq!(
        closed_world_bundle_construction_count, 1,
        "the closed-world bundle and qualified index must be built once, not rebuilt per module"
    );
    assert_eq!(
        result
            .bundle_result()
            .closed_world_outcome
            .bundle()
            .map(|bundle| bundle.linked_modules().len()),
        Some(styles.len())
    );
    println!(
        "OMENA_QUERY_MODULE_REACHABILITY_HOIST path={entry_path} \
         projectionSummaryEvaluationCount={projection_summary_evaluation_count} \
         closedWorldBundleConstructionCount={closed_world_bundle_construction_count}"
    );
    Ok(())
}

fn module_reachability_input(
    references: &[(&str, &str, &str)],
    styles: &[(&str, &str, &[&str])],
) -> EngineInputV2 {
    EngineInputV2 {
        version: "2".to_string(),
        sources: references
            .iter()
            .map(|(id, style_path, class_name)| SourceAnalysisInputV2 {
                document: SourceDocumentV2 {
                    class_expressions: vec![ClassExpressionInputV2 {
                        id: (*id).to_string(),
                        kind: "styleAccess".to_string(),
                        scss_module_path: (*style_path).to_string(),
                        range: fixture_range(),
                        class_name: Some((*class_name).to_string()),
                        root_binding_decl_id: None,
                        access_path: Some(vec![(*class_name).to_string()]),
                    }],
                },
            })
            .collect(),
        styles: styles
            .iter()
            .map(|(path, source, selectors)| style_input(path, source, selectors))
            .collect(),
        type_facts: references
            .iter()
            .map(|(id, _, class_name)| exact_type_fact(id, class_name))
            .collect(),
    }
}

fn style_input(path: &str, source: &str, selectors: &[&str]) -> StyleAnalysisInputV2 {
    StyleAnalysisInputV2 {
        file_path: path.to_string(),
        source: Some(source.to_string()),
        document: StyleDocumentV2 {
            selectors: selectors
                .iter()
                .map(|name| StyleSelectorV2 {
                    name: (*name).to_string(),
                    view_kind: "canonical".to_string(),
                    canonical_name: Some((*name).to_string()),
                    range: fixture_range(),
                    nested_safety: Some("safe".to_string()),
                    composes: None,
                    bem_suffix: None,
                })
                .collect(),
        },
    }
}

fn exact_type_fact(expression_id: &str, class_name: &str) -> TypeFactEntryV2 {
    TypeFactEntryV2 {
        file_path: "src/references.tsx".to_string(),
        expression_id: expression_id.to_string(),
        facts: StringTypeFactsV2 {
            kind: "exact".to_string(),
            constraint_kind: None,
            values: Some(vec![class_name.to_string()]),
            prefix: None,
            suffix: None,
            min_len: None,
            max_len: None,
            char_must: None,
            char_may: None,
            may_include_other_chars: None,
            provenance: None,
        },
        control_flow_graph: None,
    }
}

fn fixture_range() -> RangeV2 {
    RangeV2 {
        start: PositionV2 {
            line: 0,
            character: 0,
        },
        end: PositionV2 {
            line: 0,
            character: 1,
        },
    }
}
