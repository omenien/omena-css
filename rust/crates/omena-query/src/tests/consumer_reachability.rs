use crate::{
    OmenaQueryStyleSourceInputV0, OmenaQueryTargetTransformOptionsV0,
    OmenaQueryTransformExecutionContextV0, execute_omena_query_consumer_build_style_sources,
    execute_omena_query_consumer_build_style_sources_for_target_query_with_context_and_options,
    execute_omena_query_consumer_build_style_sources_with_context,
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
fn consumer_build_requires_explicit_reachability_for_tree_shaking() {
    let sources = vec![OmenaQueryStyleSourceInputV0 {
        style_path: "Button.module.css".to_string(),
        style_source: ".used { color: blue; } .dead { color: red; }".to_string(),
    }];
    let context = OmenaQueryTransformExecutionContextV0 {
        closed_style_world: true,
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
    assert_eq!(summary.semantic_removal_count, 1);
    assert_eq!(summary.execution.semantic_removals.len(), 1);
    assert_eq!(summary.execution.semantic_removals[0].symbol_kind, "class");
    assert_eq!(summary.execution.semantic_removals[0].name, "dead");
    assert_eq!(
        summary.execution.semantic_removals[0].derivation_steps,
        vec![
            "closedStyleWorld",
            "reachableRootSetComputed",
            "symbolNotMarkedReachable",
            "sourceRangeRemoved",
        ]
    );
    assert!(!summary.execution.output_css.contains(".dead"));
    assert!(summary.execution.output_css.contains(".used"));
}

#[test]
fn consumer_build_extends_reachability_through_css_modules_composes() {
    let sources = vec![OmenaQueryStyleSourceInputV0 {
        style_path: "Button.module.css".to_string(),
        style_source: r#".button { composes: base utility; color: red; } .base { color: blue; } .utility { animation: spin 1s; color: var(--brand); } .dead { color: black; } @keyframes spin { to { opacity: 1; } } @keyframes ghost { to { opacity: 0; } } :root { --brand: red; --dead: blue; }"#
            .to_string(),
    }];
    let context = OmenaQueryTransformExecutionContextV0 {
        closed_style_world: true,
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
    assert_eq!(
        summary
            .execution
            .semantic_removals
            .iter()
            .map(|removal| (removal.pass_id, removal.name.as_str()))
            .collect::<Vec<_>>(),
        vec![
            ("tree-shake-class", "dead"),
            ("tree-shake-keyframes", "ghost"),
            ("tree-shake-custom-property", "--dead"),
        ]
    );
}

#[test]
fn consumer_build_keeps_css_modules_values_used_by_reachable_keyframes() {
    let sources = vec![OmenaQueryStyleSourceInputV0 {
        style_path: "Button.module.css".to_string(),
        style_source: r#"@value used: red; @value dead: blue; @value ghost: green; @keyframes pulse { to { color: used; } } @keyframes ghost { to { color: ghost; } } .button { animation: pulse 1s; }"#.to_string(),
    }];
    let context = OmenaQueryTransformExecutionContextV0 {
        closed_style_world: true,
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
    assert_eq!(
        summary
            .execution
            .semantic_removals
            .iter()
            .map(|removal| (removal.pass_id, removal.symbol_kind, removal.name.as_str()))
            .collect::<Vec<_>>(),
        vec![
            ("tree-shake-keyframes", "keyframes", "ghost"),
            ("tree-shake-value", "cssModuleValue", "dead"),
            ("tree-shake-value", "cssModuleValue", "ghost"),
        ]
    );
}

#[test]
fn consumer_build_scopes_semantic_tree_shaking_to_reachable_class_rules() {
    let sources = vec![OmenaQueryStyleSourceInputV0 {
        style_path: "Button.module.css".to_string(),
        style_source: r#"@value liveValue: red; @value deadValue: orange; @keyframes liveSpin { to { opacity: 1; } } @keyframes deadSpin { to { opacity: 0; } } :root { --live: blue; --dead: gray; } .used { color: liveValue; border-color: var(--live); animation: liveSpin 1s; } .dead { color: deadValue; background: var(--dead); animation: deadSpin 1s; }"#
            .to_string(),
    }];
    let context = OmenaQueryTransformExecutionContextV0 {
        closed_style_world: true,
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
    assert!(
        summary
            .execution
            .output_css
            .contains(".dead { color: deadValue;")
    );
    assert_eq!(
        summary
            .execution
            .semantic_removals
            .iter()
            .map(|removal| (removal.pass_id, removal.name.as_str()))
            .collect::<Vec<_>>(),
        vec![
            ("tree-shake-keyframes", "deadSpin"),
            ("tree-shake-value", "deadValue"),
            ("tree-shake-custom-property", "--dead"),
        ]
    );
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
