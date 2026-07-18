use crate::{
    ClassExpressionInputV2, EngineInputV2, OmenaQueryStyleSourceInputV0,
    OmenaQueryTargetTransformOptionsV0, OmenaQueryTransformExecutionContextV0, PositionV2, RangeV2,
    SourceAnalysisInputV2, SourceDocumentV2, StringTypeFactsV2, StyleAnalysisInputV2,
    StyleDocumentV2, StyleSelectorV2, TypeFactEntryV2,
    execute_omena_query_consumer_build_style_source_with_context,
    execute_omena_query_consumer_build_style_source_with_engine_input_context,
    execute_omena_query_consumer_build_style_sources,
    execute_omena_query_consumer_build_style_sources_for_target_query_with_context_and_options,
    execute_omena_query_consumer_build_style_sources_with_context,
    summarize_omena_query_expression_domain_selector_projection_with_precision,
};

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
fn acyclic_automaton_reachability_satisfies_the_tree_shake_precision_floor() {
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
        .map(|entry| entry.precision);
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
    .expect("precision calibration report should be valid JSON");
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
