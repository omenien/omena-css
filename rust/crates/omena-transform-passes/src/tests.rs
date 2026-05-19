use super::{
    TransformClassNameRewriteV0, TransformCssModuleComposesResolutionV0,
    TransformCssModuleValueResolutionV0, TransformDesignTokenRouteV0, TransformExecutionContextV0,
    TransformImportInlineV0, TransformModuleEvaluationV0, TransformPassRuntimeStatus,
    execute_transform_passes_incremental_with_database, execute_transform_passes_on_source,
    execute_transform_passes_on_source_with_dialect,
    execute_transform_passes_on_source_with_dialect_and_context, plan_transform_passes,
    run_transform_fuzz_seed_corpus, summarize_omena_transform_passes_boundary,
    transform_pass_incremental_graph_input,
};
use omena_incremental::{IncrementalRevisionV0, OmenaIncrementalDatabaseV0};
use omena_parser::StyleDialect;
use omena_transform_cst::{TRANSFORM_PASS_CATALOG_LEN, TransformPassKind};

#[test]
fn registry_covers_full_transform_catalog() {
    let boundary = summarize_omena_transform_passes_boundary();

    assert_eq!(boundary.schema_version, "0");
    assert_eq!(boundary.product, "omena-transform-passes.boundary");
    assert_eq!(boundary.pass_count, TRANSFORM_PASS_CATALOG_LEN);
    assert!(boundary.full_catalog_registered);
    assert_eq!(boundary.semantic_aware_pass_count, 14);
    assert!(boundary.cascade_aware_pass_count >= 9);
    assert!(boundary.planner_enforces_dag_edges);
    assert!(boundary.execution_runtime_ready);
    assert!(boundary.incremental_execution_runtime_ready);
    assert_eq!(
        boundary.implemented_mutation_pass_ids,
        vec![
            "whitespace-strip",
            "comment-strip",
            "number-compression",
            "unit-normalization",
            "color-compression",
            "url-quote-strip",
            "string-quote-normalize",
            "selector-is-where-compression",
            "shorthand-combining",
            "rule-deduplication",
            "rule-merging",
            "selector-merging",
            "empty-rule-removal",
            "vendor-prefixing",
            "light-dark-lowering",
            "color-mix-lowering",
            "oklch-oklab-lowering",
            "color-function-lowering",
            "logical-to-physical",
            "nesting-unwrap",
            "scope-flatten",
            "layer-flatten",
            "supports-static-eval",
            "media-static-eval",
            "dead-media-branch-removal",
            "dead-supports-branch-removal",
            "import-inline",
            "scss-module-evaluate",
            "less-module-evaluate",
            "value-resolution",
            "custom-property-static-resolve",
            "composes-resolution",
            "css-modules-class-hashing",
            "tree-shake-class",
            "tree-shake-keyframes",
            "tree-shake-value",
            "tree-shake-custom-property",
            "design-token-routing",
            "calc-reduction",
            "print-css"
        ]
    );
    assert!(boundary.registry_entries.iter().any(|entry| {
        entry.contract.kind == TransformPassKind::TreeShakeClass
            && entry.module_family == "semantic-reachability"
    }));
    assert!(
        !boundary
            .next_surfaces
            .contains(&"transformContextProducers")
    );
    assert!(
        !boundary
            .next_surfaces
            .contains(&"provenanceSourceSpanMapping")
    );
    assert!(!boundary.next_surfaces.contains(&"transformSalsaQueries"));
    assert!(!boundary.next_surfaces.contains(&"sourceMapSpanPrecision"));
}

#[test]
fn incremental_transform_graph_tracks_source_context_plan_and_pass_dependencies() {
    let context = TransformExecutionContextV0 {
        reachable_class_names: vec!["used".to_string()],
        ..TransformExecutionContextV0::default()
    };
    let graph = transform_pass_incremental_graph_input(
        ".used { color: red; }",
        StyleDialect::Css,
        &[
            TransformPassKind::TreeShakeClass,
            TransformPassKind::PrintCss,
        ],
        &context,
        IncrementalRevisionV0 { value: 1 },
    );

    assert_eq!(graph.revision.value, 1);
    assert!(graph.nodes.iter().any(|node| node.id == "transform:source"));
    assert!(
        graph
            .nodes
            .iter()
            .any(|node| node.id == "transform:context")
    );
    assert!(graph.nodes.iter().any(|node| node.id == "transform:plan"));
    let execution_node = graph
        .nodes
        .iter()
        .find(|node| node.id == "transform:execution");
    assert!(execution_node.is_some());
    if let Some(execution_node) = execution_node {
        assert!(
            execution_node
                .dependency_ids
                .contains(&"transform:pass:print-css".to_string())
        );
    }
}

#[test]
fn incremental_transform_execution_reuses_clean_salsa_database_plan() {
    let mut incremental_database = OmenaIncrementalDatabaseV0::default();
    let context = TransformExecutionContextV0::default();
    let requested = [TransformPassKind::CommentStrip, TransformPassKind::PrintCss];
    let first = execute_transform_passes_incremental_with_database(
        ".button { /* keep no comment */ color: red; }",
        StyleDialect::Css,
        &requested,
        &context,
        &mut incremental_database,
        None,
        IncrementalRevisionV0 { value: 1 },
    );

    assert_eq!(
        first.product,
        "omena-transform-passes.incremental-execution"
    );
    assert_eq!(first.incremental_engine, "omena-incremental");
    assert!(!first.reused_previous_execution);
    assert!(first.incremental_plan.dirty_node_count > 0);
    assert!(first.ready_surfaces.contains(&"transformSalsaQueries"));

    let reused = execute_transform_passes_incremental_with_database(
        ".button { /* keep no comment */ color: red; }",
        StyleDialect::Css,
        &requested,
        &context,
        &mut incremental_database,
        Some(&first.execution),
        IncrementalRevisionV0 { value: 2 },
    );

    assert!(reused.reused_previous_execution);
    assert_eq!(reused.incremental_plan.dirty_node_count, 0);
    assert_eq!(reused.execution.output_css, first.execution.output_css);

    let changed = execute_transform_passes_incremental_with_database(
        ".button { /* changed */ color: blue; }",
        StyleDialect::Css,
        &requested,
        &context,
        &mut incremental_database,
        Some(&reused.execution),
        IncrementalRevisionV0 { value: 3 },
    );

    assert!(!changed.reused_previous_execution);
    assert!(changed.incremental_plan.changed_input_count >= 1);
    assert!(changed.execution.output_css.contains("blue"));
}

#[test]
fn fuzz_seed_corpus_preserves_transform_cascade_safe_invariants() {
    let report = run_transform_fuzz_seed_corpus();

    assert_eq!(report.product, "omena-transform-passes.fuzz-seed-corpus");
    assert_eq!(report.failed_count, 0);
    assert_eq!(report.passed_count, report.case_count);
    assert!(
        report
            .results
            .iter()
            .all(|result| result.output_error_count == 0)
    );
    assert!(
        report
            .results
            .iter()
            .any(|result| !result.executed_pass_ids.is_empty())
    );
}

#[test]
fn planner_respects_var_before_calc_before_print_edges() {
    let plan = plan_transform_passes(&[
        TransformPassKind::PrintCss,
        TransformPassKind::CalcReduction,
        TransformPassKind::StaticVarSubstitution,
    ]);

    assert_eq!(plan.violated_dag_edge_count, 0);
    assert!(plan.all_requested_registered);
    assert_eq!(
        plan.ordered_pass_ids,
        vec![
            "custom-property-static-resolve",
            "calc-reduction",
            "print-css"
        ]
    );
}

#[test]
fn planner_respects_composes_before_hash_before_selector_merge_edges() {
    let plan = plan_transform_passes(&[
        TransformPassKind::SelectorMerging,
        TransformPassKind::HashCssModuleClassNames,
        TransformPassKind::ResolveCssModulesComposes,
    ]);

    assert_eq!(plan.violated_dag_edge_count, 0);
    assert_eq!(
        plan.ordered_pass_ids,
        vec![
            "composes-resolution",
            "css-modules-class-hashing",
            "selector-merging"
        ]
    );
}

#[test]
fn planner_respects_block_canonicalization_before_selector_merge_edges() {
    let plan = plan_transform_passes(&[
        TransformPassKind::SelectorMerging,
        TransformPassKind::ColorCompression,
        TransformPassKind::UnitNormalization,
        TransformPassKind::CalcReduction,
        TransformPassKind::WhitespaceStrip,
        TransformPassKind::PrintCss,
    ]);

    assert_eq!(plan.violated_dag_edge_count, 0);
    assert_eq!(
        plan.ordered_pass_ids,
        vec![
            "calc-reduction",
            "unit-normalization",
            "color-compression",
            "selector-merging",
            "whitespace-strip",
            "print-css"
        ]
    );
}

#[test]
fn planner_respects_nesting_before_hash_edges() {
    let plan = plan_transform_passes(&[
        TransformPassKind::HashCssModuleClassNames,
        TransformPassKind::NestingUnwrap,
        TransformPassKind::PrintCss,
    ]);

    assert_eq!(plan.violated_dag_edge_count, 0);
    assert_eq!(
        plan.ordered_pass_ids,
        vec!["nesting-unwrap", "css-modules-class-hashing", "print-css"]
    );
}

#[test]
fn planner_respects_comment_strip_before_empty_rule_removal_edge() {
    let plan = plan_transform_passes(&[
        TransformPassKind::EmptyRuleRemoval,
        TransformPassKind::CommentStrip,
        TransformPassKind::PrintCss,
    ]);

    assert_eq!(plan.violated_dag_edge_count, 0);
    assert_eq!(
        plan.ordered_pass_ids,
        vec!["comment-strip", "empty-rule-removal", "print-css"]
    );
}

#[test]
fn planner_respects_semantic_tree_shaking_before_empty_rule_removal_edges() {
    let plan = plan_transform_passes(&[
        TransformPassKind::EmptyRuleRemoval,
        TransformPassKind::TreeShakeCustomProperty,
        TransformPassKind::TreeShakeValue,
        TransformPassKind::TreeShakeKeyframes,
        TransformPassKind::TreeShakeClass,
        TransformPassKind::PrintCss,
    ]);

    assert_eq!(plan.violated_dag_edge_count, 0);
    assert_eq!(
        plan.ordered_pass_ids,
        vec![
            "tree-shake-class",
            "tree-shake-keyframes",
            "tree-shake-value",
            "tree-shake-custom-property",
            "empty-rule-removal",
            "print-css"
        ]
    );
}

#[test]
fn execution_runtime_inlines_imports_from_explicit_replacements() {
    let source = r#"@import "./tokens.css"; @import url(./theme.css); @import "./conditional.css" layer(theme) supports(display: grid) screen and (min-width: 40rem); .button { color: var(--brand); }"#;
    let context = TransformExecutionContextV0 {
        import_inlines: vec![
            TransformImportInlineV0 {
                import_source: "./tokens.css".to_string(),
                replacement_css: r#":root { --brand: red; }"#.to_string(),
            },
            TransformImportInlineV0 {
                import_source: "./theme.css".to_string(),
                replacement_css: r#"@media screen { .theme { color: blue; } }"#.to_string(),
            },
            TransformImportInlineV0 {
                import_source: "./conditional.css".to_string(),
                replacement_css: r#".conditional { color: green; }"#.to_string(),
            },
        ],
        ..TransformExecutionContextV0::default()
    };
    let execution = execute_transform_passes_on_source_with_dialect_and_context(
        source,
        StyleDialect::Css,
        &[TransformPassKind::ImportInline, TransformPassKind::PrintCss],
        &context,
    );

    assert_eq!(execution.mutation_count, 3);
    assert_eq!(
        execution.output_css,
        r#":root { --brand: red; } @media screen { .theme { color: blue; } } @media screen and (min-width: 40rem) { @supports (display: grid) { @layer theme { .conditional { color: green; } } } } .button { color: var(--brand); }"#
    );
    assert_eq!(
        execution.css_import_inlines,
        vec![
            TransformImportInlineV0 {
                import_source: "./tokens.css".to_string(),
                replacement_css: r#":root { --brand: red; }"#.to_string(),
            },
            TransformImportInlineV0 {
                import_source: "./theme.css".to_string(),
                replacement_css: r#"@media screen { .theme { color: blue; } }"#.to_string(),
            },
            TransformImportInlineV0 {
                import_source: "./conditional.css".to_string(),
                replacement_css: r#".conditional { color: green; }"#.to_string(),
            },
        ]
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["import-inline", "print-css"]
    );
}

#[test]
fn execution_runtime_inlines_less_imports_with_options() {
    let source = r#"@import (reference) "./tokens.less"; .button { color: @brand; }"#;
    let context = TransformExecutionContextV0 {
        import_inlines: vec![TransformImportInlineV0 {
            import_source: "./tokens.less".to_string(),
            replacement_css: r#"@brand: red; .base { color: @brand; }"#.to_string(),
        }],
        ..TransformExecutionContextV0::default()
    };
    let execution = execute_transform_passes_on_source_with_dialect_and_context(
        source,
        StyleDialect::Less,
        &[TransformPassKind::ImportInline, TransformPassKind::PrintCss],
        &context,
    );

    assert_eq!(execution.mutation_count, 1);
    assert_eq!(
        execution.output_css,
        r#"@brand: red; .button { color: @brand; }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["import-inline", "print-css"]
    );
}

#[test]
fn execution_runtime_inlines_less_imports_once_by_default() {
    let source =
        r#"@import "./tokens.less"; @import (once) "./tokens.less"; .button { color: @brand; }"#;
    let context = TransformExecutionContextV0 {
        import_inlines: vec![TransformImportInlineV0 {
            import_source: "./tokens.less".to_string(),
            replacement_css: r#"@brand: red; .base { color: @brand; }"#.to_string(),
        }],
        ..TransformExecutionContextV0::default()
    };
    let execution = execute_transform_passes_on_source_with_dialect_and_context(
        source,
        StyleDialect::Less,
        &[TransformPassKind::ImportInline, TransformPassKind::PrintCss],
        &context,
    );

    assert_eq!(execution.mutation_count, 2);
    assert_eq!(
        execution.output_css,
        r#"@brand: red; .base { color: @brand; }  .button { color: @brand; }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["import-inline", "print-css"]
    );
}

#[test]
fn execution_runtime_honors_less_multiple_imports() {
    let source = r#"@import "./tokens.less"; @import (multiple) "./tokens.less"; .button { color: @brand; }"#;
    let context = TransformExecutionContextV0 {
        import_inlines: vec![TransformImportInlineV0 {
            import_source: "./tokens.less".to_string(),
            replacement_css: r#"@brand: red; .base { color: @brand; }"#.to_string(),
        }],
        ..TransformExecutionContextV0::default()
    };
    let execution = execute_transform_passes_on_source_with_dialect_and_context(
        source,
        StyleDialect::Less,
        &[TransformPassKind::ImportInline, TransformPassKind::PrintCss],
        &context,
    );

    assert_eq!(execution.mutation_count, 2);
    assert_eq!(
        execution.output_css,
        r#"@brand: red; .base { color: @brand; } @brand: red; .base { color: @brand; } .button { color: @brand; }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["import-inline", "print-css"]
    );
}

#[test]
fn execution_runtime_removes_missing_optional_less_imports() {
    let source = r#"@import (optional) "./missing.less"; .button { color: red; }"#;
    let execution = execute_transform_passes_on_source_with_dialect_and_context(
        source,
        StyleDialect::Less,
        &[TransformPassKind::ImportInline, TransformPassKind::PrintCss],
        &TransformExecutionContextV0::default(),
    );

    assert_eq!(execution.mutation_count, 1);
    assert_eq!(execution.output_css, r#" .button { color: red; }"#);
    assert_eq!(
        execution.executed_pass_ids,
        vec!["import-inline", "print-css"]
    );
}

#[test]
fn execution_runtime_preserves_less_inline_imports_as_literal_css() {
    let source = r#"@import (inline) "./tokens.less"; .button { color: blue; }"#;
    let context = TransformExecutionContextV0 {
        import_inlines: vec![TransformImportInlineV0 {
            import_source: "./tokens.less".to_string(),
            replacement_css: r#"@brand: red; .base { color: @brand; }"#.to_string(),
        }],
        ..TransformExecutionContextV0::default()
    };
    let execution = execute_transform_passes_on_source_with_dialect_and_context(
        source,
        StyleDialect::Less,
        &[TransformPassKind::ImportInline, TransformPassKind::PrintCss],
        &context,
    );

    assert_eq!(execution.mutation_count, 1);
    assert_eq!(
        execution.output_css,
        r#"@brand: red; .base { color: @brand; } .button { color: blue; }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["import-inline", "print-css"]
    );
}

#[test]
fn execution_runtime_preserves_less_css_imports_as_css_imports() {
    let source = r#"@import (css) "./tokens.less" screen; .button { color: blue; }"#;
    let context = TransformExecutionContextV0 {
        import_inlines: vec![TransformImportInlineV0 {
            import_source: "./tokens.less".to_string(),
            replacement_css: r#"@brand: red; .base { color: @brand; }"#.to_string(),
        }],
        ..TransformExecutionContextV0::default()
    };
    let execution = execute_transform_passes_on_source_with_dialect_and_context(
        source,
        StyleDialect::Less,
        &[TransformPassKind::ImportInline, TransformPassKind::PrintCss],
        &context,
    );

    assert_eq!(execution.mutation_count, 1);
    assert_eq!(
        execution.output_css,
        r#"@import "./tokens.less" screen; .button { color: blue; }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["import-inline", "print-css"]
    );
}

#[test]
fn execution_runtime_leaves_unknown_less_import_options_untouched() {
    let source = r#"@import (plugin) "./tokens.less"; .button { color: blue; }"#;
    let context = TransformExecutionContextV0 {
        import_inlines: vec![TransformImportInlineV0 {
            import_source: "./tokens.less".to_string(),
            replacement_css: r#"@brand: red;"#.to_string(),
        }],
        ..TransformExecutionContextV0::default()
    };
    let execution = execute_transform_passes_on_source_with_dialect_and_context(
        source,
        StyleDialect::Less,
        &[TransformPassKind::ImportInline, TransformPassKind::PrintCss],
        &context,
    );

    assert_eq!(execution.mutation_count, 0);
    assert_eq!(execution.output_css, source);
    assert_eq!(
        execution.executed_pass_ids,
        vec!["import-inline", "print-css"]
    );
}

#[test]
fn execution_runtime_applies_explicit_scss_module_evaluation() {
    let source = r#"$brand: red; .button { color: $brand; }"#;
    let context = TransformExecutionContextV0 {
        scss_module_evaluation: Some(TransformModuleEvaluationV0 {
            evaluator: "dart-sass-compatible".to_string(),
            evaluated_css: ".button { color: red; }".to_string(),
        }),
        ..TransformExecutionContextV0::default()
    };
    let execution = execute_transform_passes_on_source_with_dialect_and_context(
        source,
        StyleDialect::Scss,
        &[
            TransformPassKind::ScssModuleEvaluate,
            TransformPassKind::PrintCss,
        ],
        &context,
    );

    assert_eq!(execution.mutation_count, 1);
    assert_eq!(execution.output_css, ".button { color: red; }");
    assert_eq!(
        execution.css_module_evaluation,
        Some(TransformModuleEvaluationV0 {
            evaluator: "dart-sass-compatible".to_string(),
            evaluated_css: ".button { color: red; }".to_string(),
        })
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["scss-module-evaluate", "print-css"]
    );
}

#[test]
fn execution_runtime_applies_explicit_less_module_evaluation() {
    let source = r#"@brand: red; .button { color: @brand; }"#;
    let context = TransformExecutionContextV0 {
        less_module_evaluation: Some(TransformModuleEvaluationV0 {
            evaluator: "less-js-compatible".to_string(),
            evaluated_css: ".button { color: red; }".to_string(),
        }),
        ..TransformExecutionContextV0::default()
    };
    let execution = execute_transform_passes_on_source_with_dialect_and_context(
        source,
        StyleDialect::Less,
        &[
            TransformPassKind::LessModuleEvaluate,
            TransformPassKind::PrintCss,
        ],
        &context,
    );

    assert_eq!(execution.mutation_count, 1);
    assert_eq!(execution.output_css, ".button { color: red; }");
    assert_eq!(
        execution.css_module_evaluation,
        Some(TransformModuleEvaluationV0 {
            evaluator: "less-js-compatible".to_string(),
            evaluated_css: ".button { color: red; }".to_string(),
        })
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["less-module-evaluate", "print-css"]
    );
}

#[test]
fn execution_runtime_resolves_css_module_composes_with_export_set() {
    let source = r#".button { composes: base from "./base.module.css"; color: red; } .button:hover { color: blue; } .card, .panel { composes: shared; color: green; } :local(.card) { composes: shared; color: yellow; } :local(.card, .panel) { composes: shared; color: purple; } :local { .button { composes: base; color: navy; } } :global { .button { composes: base; color: pink; } } @media (min-width: 1px) { .button { composes: base; color: black; } }"#;
    let context = TransformExecutionContextV0 {
        css_module_composes_resolutions: vec![
            TransformCssModuleComposesResolutionV0 {
                local_class_name: "button".to_string(),
                exported_class_names: vec!["button".to_string(), "base".to_string()],
            },
            TransformCssModuleComposesResolutionV0 {
                local_class_name: "card".to_string(),
                exported_class_names: vec!["card".to_string(), "shared".to_string()],
            },
            TransformCssModuleComposesResolutionV0 {
                local_class_name: "panel".to_string(),
                exported_class_names: vec!["panel".to_string(), "shared".to_string()],
            },
        ],
        ..TransformExecutionContextV0::default()
    };
    let execution = execute_transform_passes_on_source_with_dialect_and_context(
        source,
        StyleDialect::Css,
        &[
            TransformPassKind::ResolveCssModulesComposes,
            TransformPassKind::PrintCss,
        ],
        &context,
    );

    assert_eq!(execution.mutation_count, 6);
    assert_eq!(
        execution.output_css,
        r#".button {  color: red; } .button:hover { color: blue; } .card, .panel {  color: green; } :local(.card) {  color: yellow; } :local(.card, .panel) {  color: purple; } :local { .button {  color: navy; } } :global { .button { composes: base; color: pink; } } @media (min-width: 1px) { .button {  color: black; } }"#
    );
    assert_eq!(
        execution.css_module_composes_exports,
        vec![
            TransformCssModuleComposesResolutionV0 {
                local_class_name: "button".to_string(),
                exported_class_names: vec!["button".to_string(), "base".to_string()],
            },
            TransformCssModuleComposesResolutionV0 {
                local_class_name: "card".to_string(),
                exported_class_names: vec!["card".to_string(), "shared".to_string()],
            },
            TransformCssModuleComposesResolutionV0 {
                local_class_name: "panel".to_string(),
                exported_class_names: vec!["panel".to_string(), "shared".to_string()],
            },
        ]
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["composes-resolution", "print-css"]
    );
}

#[test]
fn execution_runtime_routes_design_tokens_from_bridge_context() {
    let source = r#"@property --registered { syntax: "<color>"; inherits: false; initial-value: var(--pkg-brand); } @keyframes pulse { to { color: var(--pkg-border); } } .button { color: var(--pkg-brand); background: var(--pkg-brand, blue); border: 1px solid var(--pkg-border); box-shadow: 0 0 1px var(--unsafe); --local: var(--pkg-brand); } @media screen { .button { outline-color: var(--pkg-brand); } }"#;
    let context = TransformExecutionContextV0 {
        design_token_routes: vec![
            TransformDesignTokenRouteV0 {
                token_name: "--pkg-brand".to_string(),
                routed_value: "var(--theme-brand)".to_string(),
            },
            TransformDesignTokenRouteV0 {
                token_name: "--pkg-border".to_string(),
                routed_value: "#123456".to_string(),
            },
            TransformDesignTokenRouteV0 {
                token_name: "--unsafe".to_string(),
                routed_value: "red; color: blue".to_string(),
            },
        ],
        ..TransformExecutionContextV0::default()
    };
    let execution = execute_transform_passes_on_source_with_dialect_and_context(
        source,
        StyleDialect::Css,
        &[
            TransformPassKind::DesignTokenRouting,
            TransformPassKind::PrintCss,
        ],
        &context,
    );

    assert_eq!(execution.mutation_count, 7);
    assert_eq!(
        execution.output_css,
        r#"@property --registered { syntax: "<color>"; inherits: false; initial-value: var(--theme-brand); } @keyframes pulse { to { color: #123456; } } .button { color: var(--theme-brand); background: var(--theme-brand, blue); border: 1px solid #123456; box-shadow: 0 0 1px var(--unsafe); --local: var(--theme-brand); } @media screen { .button { outline-color: var(--theme-brand); } }"#
    );
    assert_eq!(execution.design_token_routes, context.design_token_routes);
    assert_eq!(
        execution.executed_pass_ids,
        vec!["design-token-routing", "print-css"]
    );
}

#[test]
fn execution_runtime_routes_design_tokens_in_supported_at_rule_preludes() {
    let source = r#"@custom-media --wide (min-width: var(--pkg-breakpoint)); @container card style(--theme: var(--pkg-theme)) { .button { color: var(--pkg-brand); } } @supports (color: var(--pkg-brand)) { .button { border-color: currentColor; } } @media (min-width: var(--pkg-breakpoint)) { .button { color: red; } }"#;
    let context = TransformExecutionContextV0 {
        design_token_routes: vec![
            TransformDesignTokenRouteV0 {
                token_name: "--pkg-theme".to_string(),
                routed_value: "var(--theme-mode)".to_string(),
            },
            TransformDesignTokenRouteV0 {
                token_name: "--pkg-brand".to_string(),
                routed_value: "#123456".to_string(),
            },
            TransformDesignTokenRouteV0 {
                token_name: "--pkg-breakpoint".to_string(),
                routed_value: "40rem".to_string(),
            },
        ],
        ..TransformExecutionContextV0::default()
    };
    let execution = execute_transform_passes_on_source_with_dialect_and_context(
        source,
        StyleDialect::Css,
        &[
            TransformPassKind::DesignTokenRouting,
            TransformPassKind::PrintCss,
        ],
        &context,
    );

    assert_eq!(execution.mutation_count, 5);
    assert_eq!(
        execution.output_css,
        r#"@custom-media --wide (min-width: 40rem); @container card style(--theme: var(--theme-mode)) { .button { color: #123456; } } @supports (color: #123456) { .button { border-color: currentColor; } } @media (min-width: 40rem) { .button { color: red; } }"#
    );
}

#[test]
fn execution_runtime_routes_design_tokens_inside_custom_property_aliases() {
    let source = r#":root { --pkg-brand: var(--pkg-brand, black); --alias: var(--pkg-brand); --bridge: var(--pkg-border); } .button { color: var(--alias); }"#;
    let context = TransformExecutionContextV0 {
        design_token_routes: vec![
            TransformDesignTokenRouteV0 {
                token_name: "--pkg-brand".to_string(),
                routed_value: "var(--theme-brand)".to_string(),
            },
            TransformDesignTokenRouteV0 {
                token_name: "--pkg-border".to_string(),
                routed_value: "#123456".to_string(),
            },
        ],
        ..TransformExecutionContextV0::default()
    };
    let execution = execute_transform_passes_on_source_with_dialect_and_context(
        source,
        StyleDialect::Css,
        &[
            TransformPassKind::DesignTokenRouting,
            TransformPassKind::PrintCss,
        ],
        &context,
    );

    assert_eq!(execution.mutation_count, 2);
    assert_eq!(
        execution.output_css,
        r#":root { --pkg-brand: var(--pkg-brand, black); --alias: var(--theme-brand); --bridge: #123456; } .button { color: var(--alias); }"#
    );
}

#[test]
fn execution_runtime_recovers_design_token_routing_after_malformed_var() {
    let source =
        r#".button { color: var(--pkg-brand); box-shadow: 0 0 var(--pkg-border) var(--broken; }"#;
    let context = TransformExecutionContextV0 {
        design_token_routes: vec![
            TransformDesignTokenRouteV0 {
                token_name: "--pkg-brand".to_string(),
                routed_value: "var(--theme-brand)".to_string(),
            },
            TransformDesignTokenRouteV0 {
                token_name: "--pkg-border".to_string(),
                routed_value: "#123456".to_string(),
            },
        ],
        ..TransformExecutionContextV0::default()
    };
    let execution = execute_transform_passes_on_source_with_dialect_and_context(
        source,
        StyleDialect::Css,
        &[
            TransformPassKind::DesignTokenRouting,
            TransformPassKind::PrintCss,
        ],
        &context,
    );

    assert_eq!(execution.mutation_count, 2);
    assert_eq!(
        execution.output_css,
        r#".button { color: var(--theme-brand); box-shadow: 0 0 #123456 var(--broken; }"#
    );
}

#[test]
fn execution_runtime_applies_comment_strip_without_touching_strings() {
    let source = r#".a { color: red; /* remove */ content: "/* keep */"; }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::CommentStrip,
            TransformPassKind::HashCssModuleClassNames,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.product, "omena-transform-passes.execution");
    assert_eq!(execution.mutation_count, 1);
    assert_eq!(
        execution.output_css,
        r#".a { color: red;  content: "/* keep */"; }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["comment-strip", "print-css"]
    );
    assert_eq!(
        execution.planned_only_pass_ids,
        vec!["css-modules-class-hashing"]
    );
    assert!(execution.provenance_preserved);
    assert_eq!(execution.pass_plan.violated_dag_edge_count, 0);
    assert!(execution.outcomes.iter().any(|outcome| {
        outcome.pass_id == "comment-strip"
            && outcome.status == TransformPassRuntimeStatus::Applied
            && outcome.mutation_count == 1
    }));
    assert!(execution.outcomes.iter().any(|outcome| {
        outcome.pass_id == "css-modules-class-hashing"
            && outcome.status == TransformPassRuntimeStatus::PlannedOnly
    }));
    assert_eq!(
        execution.provenance_derivation_forest.product,
        "omena-transform-passes.provenance-derivation-forest"
    );
    assert_eq!(execution.provenance_derivation_forest.root_count, 1);
    assert_eq!(
        execution.provenance_derivation_forest.node_count,
        execution.outcomes.len()
    );
    let comment_node = execution
        .provenance_derivation_forest
        .nodes
        .iter()
        .find(|node| node.pass_id == "comment-strip");
    assert!(
        comment_node.is_some(),
        "comment strip provenance node should exist"
    );
    let Some(comment_node) = comment_node else {
        return;
    };
    assert_eq!(comment_node.status, TransformPassRuntimeStatus::Applied);
    assert_eq!(comment_node.mutation_count, 1);
    assert_eq!(comment_node.mutation_spans.len(), 1);
    assert_eq!(comment_node.source_span_start, 17);
    assert!(comment_node.source_span_end < comment_node.input_byte_len);
    assert_eq!(comment_node.generated_span_start, 17);
    assert_eq!(comment_node.generated_span_end, 17);
    assert_eq!(
        execution.provenance_derivation_forest.nodes[0].parent_index,
        None
    );
    for (index, node) in execution
        .provenance_derivation_forest
        .nodes
        .iter()
        .enumerate()
        .skip(1)
    {
        assert_eq!(node.parent_index, Some(index - 1));
    }
}

#[test]
fn execution_runtime_rewrites_css_module_class_names_with_identity_map() {
    let source = r#".button { composes: base utility global(reset); color: red; } .base, .utility { color: blue; } .button:hover { color: green; } .button :global(.external) { color: purple; } :global(.root) .button { color: orange; } :global(.standalone) { color: teal; } :global { .global-block { color: silver; } } :local(.button) { color: navy; } :local { .button { color: maroon; } } @media (min-width: 1px) { .button { color: black; } }"#;
    let context = TransformExecutionContextV0 {
        class_name_rewrites: vec![
            TransformClassNameRewriteV0 {
                original_name: "button".to_string(),
                rewritten_name: "_button_abc123".to_string(),
            },
            TransformClassNameRewriteV0 {
                original_name: "base".to_string(),
                rewritten_name: "_base_def456".to_string(),
            },
            TransformClassNameRewriteV0 {
                original_name: "utility".to_string(),
                rewritten_name: "_utility_ghi789".to_string(),
            },
            TransformClassNameRewriteV0 {
                original_name: "external".to_string(),
                rewritten_name: "_external_global".to_string(),
            },
            TransformClassNameRewriteV0 {
                original_name: "root".to_string(),
                rewritten_name: "_root_global".to_string(),
            },
            TransformClassNameRewriteV0 {
                original_name: "global-block".to_string(),
                rewritten_name: "_global_block_should_not_apply".to_string(),
            },
            TransformClassNameRewriteV0 {
                original_name: "reset".to_string(),
                rewritten_name: "_reset_should_not_apply".to_string(),
            },
        ],
        ..TransformExecutionContextV0::default()
    };
    let execution = execute_transform_passes_on_source_with_dialect_and_context(
        source,
        StyleDialect::Css,
        &[
            TransformPassKind::HashCssModuleClassNames,
            TransformPassKind::PrintCss,
        ],
        &context,
    );

    assert_eq!(execution.mutation_count, 14);
    assert_eq!(
        execution.output_css,
        r#"._button_abc123{ composes: _base_def456 _utility_ghi789 reset; color: red; } ._base_def456, ._utility_ghi789{ color: blue; } ._button_abc123:hover{ color: green; } ._button_abc123 .external{ color: purple; } .root ._button_abc123{ color: orange; } .standalone{ color: teal; }  .global-block { color: silver; }  ._button_abc123{ color: navy; }  ._button_abc123{ color: maroon; }  @media (min-width: 1px) { ._button_abc123{ color: black; } }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["css-modules-class-hashing", "print-css"]
    );
}

#[test]
fn execution_runtime_hashes_escaped_css_module_class_selectors() {
    let source = r#".foo\:bar { color: red; } :local(.foo\:bar) { color: blue; } :global(.foo\:bar) .foo\:bar { color: green; }"#;
    let context = TransformExecutionContextV0 {
        class_name_rewrites: vec![TransformClassNameRewriteV0 {
            original_name: "foo:bar".to_string(),
            rewritten_name: "_foo_bar_0".to_string(),
        }],
        ..TransformExecutionContextV0::default()
    };
    let execution = execute_transform_passes_on_source_with_dialect_and_context(
        source,
        StyleDialect::Css,
        &[
            TransformPassKind::HashCssModuleClassNames,
            TransformPassKind::PrintCss,
        ],
        &context,
    );

    assert_eq!(execution.mutation_count, 3);
    assert_eq!(
        execution.output_css,
        r#"._foo_bar_0{ color: red; } ._foo_bar_0{ color: blue; } .foo\:bar ._foo_bar_0{ color: green; }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["css-modules-class-hashing", "print-css"]
    );
}

#[test]
fn execution_runtime_hashes_nested_css_module_selectors_after_unwrap() {
    let source = r#".item { color: red; &--primary { color: blue; } & .body { color: green; } }"#;
    let context = TransformExecutionContextV0 {
        class_name_rewrites: vec![
            TransformClassNameRewriteV0 {
                original_name: "item".to_string(),
                rewritten_name: "_item_0".to_string(),
            },
            TransformClassNameRewriteV0 {
                original_name: "item--primary".to_string(),
                rewritten_name: "_item--primary_1".to_string(),
            },
            TransformClassNameRewriteV0 {
                original_name: "body".to_string(),
                rewritten_name: "_body_2".to_string(),
            },
        ],
        ..TransformExecutionContextV0::default()
    };
    let execution = execute_transform_passes_on_source_with_dialect_and_context(
        source,
        StyleDialect::Scss,
        &[
            TransformPassKind::HashCssModuleClassNames,
            TransformPassKind::NestingUnwrap,
            TransformPassKind::PrintCss,
        ],
        &context,
    );

    assert_eq!(
        execution.ordered_pass_ids,
        vec!["nesting-unwrap", "css-modules-class-hashing", "print-css"]
    );
    assert!(execution.output_css.contains("._item_0{ color: red; }"));
    assert!(
        execution
            .output_css
            .contains("._item--primary_1{ color: blue; }")
    );
    assert!(
        execution
            .output_css
            .contains("._item_0 ._body_2{ color: green; }")
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["nesting-unwrap", "css-modules-class-hashing", "print-css"]
    );
}

#[test]
fn execution_runtime_applies_conservative_whitespace_normalization() {
    let source = r#".a , .b { color : red ; content: "x y"; }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::WhitespaceStrip,
            TransformPassKind::CommentStrip,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 11);
    assert_eq!(execution.output_css, r#".a,.b{color:red;content:"x y"}"#);
    assert_eq!(
        execution.executed_pass_ids,
        vec!["whitespace-strip", "comment-strip", "print-css"]
    );
}

#[test]
fn execution_runtime_compresses_numeric_tokens_only() {
    let source = r#".a { width: 0.50rem; opacity: 000.50; margin: -0.25px 10.00%; scale: 1.0E+03; flex-grow: 1e+00; z-index: 001; order: +001; translate: 0e+3px; rotate: -0deg; content: "0.50 1.0E+03"; }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::NumberCompression,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 10);
    assert_eq!(
        execution.output_css,
        r#".a { width: .5rem; opacity: .5; margin: -.25px 10%; scale: 1e3; flex-grow: 1; z-index: 1; order: 1; translate: 0px; rotate: 0deg; content: "0.50 1.0E+03"; }"#
    );
}

#[test]
fn execution_runtime_normalizes_zero_length_units_with_property_context() {
    let source = r#".a { margin: 0px 0.0rem -0em; border: 0px solid #000; border-top: 0px solid #000; border-top-width: 0PX; border-radius: -0em; border-spacing: 0px 0px; letter-spacing: 0px; word-spacing: 0px; scroll-margin-inline: 0rem; outline: 0px solid #000; outline-width: 0pt; outline-offset: 0px; text-decoration: underline 0px #000; text-indent: 0px; line-height: 0em; stroke-width: 0px; stroke-dasharray: 0px; stroke-dashoffset: 0px; tab-size: 0px; vertical-align: 0px; perspective: 0px; border-image-width: 0px; flex-basis: 0px; grid-template-columns: 0px 1FR; grid-auto-rows: 0px; font-size: 0px; rotate: 1TURN; animation-delay: 200MS; transition-duration: .05s; transition-delay: 0ms; --x: 0PX; width: 10PX; }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::UnitNormalization,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 34);
    assert_eq!(
        execution.output_css,
        r#".a { margin: 0 0 0; border: 0 solid #000; border-top: 0 solid #000; border-top-width: 0; border-radius: 0; border-spacing: 0 0; letter-spacing: 0; word-spacing: 0; scroll-margin-inline: 0; outline: 0 solid #000; outline-width: 0; outline-offset: 0px; text-decoration: underline 0 #000; text-indent: 0; line-height: 0; stroke-width: 0; stroke-dasharray: 0; stroke-dashoffset: 0; tab-size: 0; vertical-align: 0; perspective: 0; border-image-width: 0; flex-basis: 0; grid-template-columns: 0 1fr; grid-auto-rows: 0; font-size: 0; rotate: 1turn; animation-delay: .2s; transition-duration: 50ms; transition-delay: 0s; --x: 0PX; width: 10px; }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["unit-normalization", "print-css"]
    );
}

#[test]
fn execution_runtime_removes_adjacent_duplicate_unit_declarations() {
    let source = r#".a { tab-size: 0px; tab-size: 0; width: 0px; width: 0; opacity: 100%; opacity: 1; color: red; color: red; }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::UnitNormalization,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 5);
    assert_eq!(
        execution.output_css,
        r#".a { tab-size: 0;  width: 0;  opacity: 1; opacity: 1; color: red; color: red; }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["unit-normalization", "print-css"]
    );
}

#[test]
fn execution_runtime_normalizes_safe_zero_percent_position_values() {
    let source = r#".a { background-position: 0% 0%; background-size: auto auto; mask-position: 0% 0%; perspective-origin: 0% 0%; transform-origin: 0% 0%; object-position: 0% 0%; width: 0%; opacity: 0%; }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::UnitNormalization,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 10);
    assert_eq!(
        execution.output_css,
        r#".a { background-position: 0 0; background-size: auto; mask-position: 0 0; perspective-origin: 0 0; transform-origin: 0 0; object-position: 0% 0%; width: 0%; opacity: 0; }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["unit-normalization", "print-css"]
    );
}

#[test]
fn execution_runtime_normalizes_shorter_opacity_percentages() {
    let source = r#".a { opacity: 50%; } .b { opacity: 100%; } .c { opacity: 5%; } .d { opacity: 150%; } .e { width: 50%; } .f { fill-opacity: 100%; stroke-opacity: 50%; flood-opacity: 0%; stop-opacity: 5%; }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::UnitNormalization,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 4);
    assert_eq!(
        execution.output_css,
        r#".a { opacity: .5; } .b { opacity: 1; } .c { opacity: 5%; } .d { opacity: 150%; } .e { width: 50%; } .f { fill-opacity: 1; stroke-opacity: .5; flood-opacity: 0%; stop-opacity: 5%; }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["unit-normalization", "print-css"]
    );
}

#[test]
fn execution_runtime_normalizes_static_aspect_ratio_spacing() {
    let source = r#".a { aspect-ratio: 16 / 9; } .b { aspect-ratio: auto 4 / 3; } .c { aspect-ratio: var(--ratio); }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::UnitNormalization,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 2);
    assert_eq!(
        execution.output_css,
        r#".a { aspect-ratio: 16/9; } .b { aspect-ratio: auto 4/3; } .c { aspect-ratio: var(--ratio); }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["unit-normalization", "print-css"]
    );
}

#[test]
fn execution_runtime_normalizes_static_center_position_values() {
    let source = r#".a { background-position: center center; transform-origin: center; mask-position: CENTER CENTER; mask-position-x: center; mask-position-y: CENTER; object-position: center center; } .b { background-position: left center; transform-origin: center top; mask-position: bottom right; mask-position-x: right; } .c { background-position: 0% 50%; mask-position: 100% 50%; -webkit-mask-position: 50% 50%; transform-origin: 50% 0%; }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::UnitNormalization,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 13);
    assert_eq!(
        execution.output_css,
        r#".a { background-position: 50%; transform-origin: 50%; mask-position: 50%; mask-position-x: 50%; mask-position-y: 50%; object-position: center center; } .b { background-position: 0; transform-origin: top; mask-position: 100% 100%; mask-position-x: right; } .c { background-position: 0%; mask-position: 100%; -webkit-mask-position: 50%; transform-origin: 50% 0; }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["unit-normalization", "print-css"]
    );
}

#[test]
fn execution_runtime_normalizes_zero_transform_function_units() {
    let source = r#".a { transform: rotate(0deg) rotateX(-0turn) translate(0px) skew(0deg); } .b { rotate: 0deg; transform: rotate(1deg) translate(1px); }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::UnitNormalization,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 1);
    assert_eq!(
        execution.output_css,
        r#".a { transform: rotate(0)rotateX(0)translate(0)skew(0deg); } .b { rotate: 0deg; transform: rotate(1deg) translate(1px); }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["unit-normalization", "print-css"]
    );
}

#[test]
fn execution_runtime_normalizes_repeated_transform_scale_values() {
    let source = r#".a { transform: scale(1, 1) scale(2, 2) scale(.5, .5) scale(1, 2); } .b { transform: scale(var(--x), var(--x)); }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::UnitNormalization,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 1);
    assert_eq!(
        execution.output_css,
        r#".a { transform: scale(1)scale(2)scale(.5)scaleY(2); } .b { transform: scale(var(--x), var(--x)); }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["unit-normalization", "print-css"]
    );
}

#[test]
fn execution_runtime_normalizes_static_3d_transform_axes() {
    let source = r#".a { transform: scale(2, 1) scale3d(1, 1, 1) scale3d(2, 3, 1) scale3d(1, 1, 2) rotate3d(1, 0, 0, 0deg) rotate3d(0, 1, 0, 1turn) rotate3d(0, 0, 1, 10deg) translate3d(0px, 0px, 0px) translate3d(1px, 0px, 0px) translate3d(0px, 1px, 0px) translate3d(0px, 0px, 1px) translate3d(1px, 2px, 0px); }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::UnitNormalization,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 1);
    assert_eq!(
        execution.output_css,
        r#".a { transform: scaleX(2)scale(1)scale(2,3)scaleZ(2)rotateX(0)rotateY(1turn)rotate(10deg)translate(0,0)translate(1px)translateY(1px)translateZ(1px)translate(1px,2px); }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["unit-normalization", "print-css"]
    );
}

#[test]
fn execution_runtime_normalizes_zero_transform_axis_lengths() {
    let source = r#".a { transform: translateX(0px) translateY(-0%) translateZ(0em) translate(0px, 0%) perspective(0px); } .b { transform: translateX(1px) translate(0px, 1px); }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::UnitNormalization,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 1);
    assert_eq!(
        execution.output_css,
        r#".a { transform: translate(0)translateY(0)translateZ(0)translate(0)perspective(0); } .b { transform: translateX(1px) translate(0px, 1px); }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["unit-normalization", "print-css"]
    );
}

#[test]
fn execution_runtime_normalizes_static_transform_tail_zeros() {
    let source = r#".a { transform: translate(1px, 0px) skew(0deg, 0deg) skewX(0deg) skewY(-0turn); } .b { transform: translate(1px, 2px) skew(1deg, 2deg); }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::UnitNormalization,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 1);
    assert_eq!(
        execution.output_css,
        r#".a { transform: translate(1px)skew(0deg)skew(0)skewY(0); } .b { transform: translate(1px, 2px) skew(1deg, 2deg); }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["unit-normalization", "print-css"]
    );
}

#[test]
fn execution_runtime_normalizes_static_filter_default_functions() {
    let source = r#".a { filter: opacity(100%) brightness(1) contrast(+1) saturate(0100%) blur(0px) hue-rotate(-0deg); } .b { backdrop-filter: opacity(.5) blur(1px); } .c { -webkit-filter: opacity(1.0); } .d { filter: drop-shadow(red 0px 0px 0px); } .e { filter: drop-shadow(1px 2px 0px #000); }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::UnitNormalization,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 5);
    assert_eq!(
        execution.output_css,
        r#".a { filter: opacity()brightness()contrast()saturate()blur()hue-rotate(); } .b { backdrop-filter: opacity(.5)blur(1px); } .c { -webkit-filter: opacity(); } .d { filter: drop-shadow(0 0 red); } .e { filter: drop-shadow(1px 2px #000); }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["unit-normalization", "print-css"]
    );
}

#[test]
fn execution_runtime_normalizes_individual_transform_properties() {
    let source = r#".t0 { translate: 0px 0% 0px; } .t1 { translate: 1px 0px; } .t2 { translate: 0px 1px; } .t3 { translate: 1px 2px 0px; } .t4 { translate: 0px 0px 1px; } .s0 { scale: 1 1; } .s1 { scale: 2 2; } .s2 { scale: 1 2; } .s3 { scale: 2 3 1; } .s4 { scale: 1 1 2; } .s5 { scale: 1 1 1; } .s6 { scale: 50% 50%; } .r0 { rotate: z 0deg; } .r1 { rotate: 0 0 1 10deg; } .r2 { rotate: 1 0 0 .500turn; } .r3 { rotate: 0 1 0 10.0deg; } .r4 { rotate: 0rad; }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::UnitNormalization,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 21);
    assert_eq!(
        execution.output_css,
        r#".t0 { translate: 0; } .t1 { translate: 1px; } .t2 { translate: 0 1px; } .t3 { translate: 1px 2px; } .t4 { translate: 0 0 1px; } .s0 { scale: 1; } .s1 { scale: 2; } .s2 { scale: 1 2; } .s3 { scale: 2 3; } .s4 { scale: 1 1 2; } .s5 { scale: 1; } .s6 { scale: .5; } .r0 { rotate: 0deg; } .r1 { rotate: 10deg; } .r2 { rotate: x .5turn; } .r3 { rotate: y 10deg; } .r4 { rotate: 0deg; }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["unit-normalization", "print-css"]
    );
}

#[test]
fn execution_runtime_normalizes_static_shadow_zero_lengths() {
    let source = r#".a { box-shadow: 0px 0px 0px #000; } .b { box-shadow: inset 1px 2px 0px 0px #000; } .c { text-shadow: 1px 2px 0px #000; } .d { box-shadow: 1px 2px 0px 5px #000; }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::UnitNormalization,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 10);
    assert_eq!(
        execution.output_css,
        r#".a { box-shadow: 0 0 #000; } .b { box-shadow: inset 1px 2px #000; } .c { text-shadow: 1px 2px #000; } .d { box-shadow: 1px 2px 0 5px #000; }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["unit-normalization", "print-css"]
    );
}

#[test]
fn execution_runtime_compresses_static_declaration_colors_only() {
    let source = r#".a { color: #FFFFFF; box-shadow: 0 0 #AABBCC, 0 0 blue; border: 1px solid black; font-family: blue; background: url(blue.svg); background-color: rgb(255 0 0); border-color: rgb(0, 128, 0); outline-color: rgb(50% 50% 50%); text-emphasis-color: rgb(128 0 128); text-decoration-color: hsl(240 100% 50%); caret-color: hsl(0, 0%, 0%); fill: hwb(0 0% 0%); stroke: hwb(120 0% 50%); column-rule-color: hwb(0 100% 0%); flood-color: white; lighting-color: black; stop-color: blue; scrollbar-color: hsl(.5TURN 100% 50%); border-block-color: hwb(200GRAD 0% 0%); border-left-color: rgb(255 0 0 / 100%); border-right-color: hsl(120 100% 25% / 1); border-top-color: hwb(240 0% 0% / 100%); background: linear-gradient(rgb(255 0 0), hsl(240 100% 50%)); filter: drop-shadow(0 0 1px hwb(0 100% 0%)); border-bottom-color: rgb(255 0 0 / .5); accent-color: hsl(0 0% 0% / 50%); --brand: rgb(255 0 0); } .alpha { color: #FFFFFFFF; background-color: #ffff; border-color: #00000000; outline-color: rgba(255, 0, 0, 1); text-decoration-color: hsla(240, 100%, 50%, 100%); accent-color: rgba(255, 0, 0, .5); text-shadow: 0 0 hsla(240, 100%, 50%, 50%); column-rule-color: hwb(0 0% 0% / 50%); fill: transparent; box-shadow: 0 0 transparent; } #FFFFFF { color: red; }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::ColorCompression,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 35);
    assert_eq!(
        execution.output_css,
        r#".a { color: #fff; box-shadow: 0 0 #abc, 0 0 #00f; border: 1px solid #000; font-family: blue; background: url(blue.svg); background-color: red; border-color: green; outline-color: gray; text-emphasis-color: purple; text-decoration-color: #00f; caret-color: #000; fill: red; stroke: green; column-rule-color: #fff; flood-color: #fff; lighting-color: #000; stop-color: #00f; scrollbar-color: #0ff; border-block-color: #0ff; border-left-color: red; border-right-color: green; border-top-color: #00f; background: linear-gradient(red, #00f); filter: drop-shadow(0 0 1px #fff); border-bottom-color: #ff000080; accent-color: #00000080; --brand: rgb(255 0 0); } .alpha { color: #fff; background-color: #fff; border-color: #0000; outline-color: red; text-decoration-color: #00f; accent-color: #ff000080; text-shadow: 0 0 #0000ff80; column-rule-color: #ff000080; fill: #0000; box-shadow: 0 0 #0000; } #FFFFFF { color: red; }"#
    );
}

#[test]
fn execution_runtime_compresses_default_linear_gradient_directions() {
    let source = r#".a { background: linear-gradient(to bottom, red, blue); background-image: repeating-linear-gradient(180deg, white, black); list-style-image: linear-gradient(0.5turn, red, blue); mask-image: linear-gradient(200grad, red, blue); border-image-source: linear-gradient(to top, red, blue); } .b { background: radial-gradient(circle at center, red, blue); } .c { background: radial-gradient(ellipse at center, red, blue); } .d { background: conic-gradient(from 0deg, red, blue); } .e { background: repeating-conic-gradient(from 0turn, red, blue); } .f { background: linear-gradient(0deg, red 10%, blue 90%); background-image: repeating-linear-gradient(0turn, white, black); }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::ColorCompression,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 11);
    assert_eq!(
        execution.output_css,
        r#".a { background: linear-gradient(red,#00f); background-image: repeating-linear-gradient(#fff,#000); list-style-image: linear-gradient(red,#00f); mask-image: linear-gradient(red,#00f); border-image-source: linear-gradient(#00f,red); } .b { background: radial-gradient(circle,red,#00f); } .c { background: radial-gradient(red,#00f); } .d { background: conic-gradient(red,#00f); } .e { background: repeating-conic-gradient(red,#00f); } .f { background: linear-gradient(#00f 10%,red 90%); background-image: repeating-linear-gradient(#000,#fff); }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["color-compression", "print-css"]
    );
}

#[test]
fn execution_runtime_compresses_hex_colors_to_shorter_named_colors() {
    let source = r#".card { color: #ff0000; outline-color: #808080; background: #0000ff; border-color: #FFFFFF; box-shadow: 0 0 1px rebeccapurple; text-shadow: 0 0 1px aliceblue; caret-color: darkgray; accent-color: #d2b48c; fill: LightGoldenRodYellow; column-rule-color: currentcolor; }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::ColorCompression,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 9);
    assert_eq!(
        execution.output_css,
        r#".card { color: red; outline-color: gray; background: #00f; border-color: #fff; box-shadow: 0 0 1px #639; text-shadow: 0 0 1px #f0f8ff; caret-color: #a9a9a9; accent-color: tan; fill: #fafad2; column-rule-color: currentcolor; }"#
    );
}

#[test]
fn execution_runtime_keeps_column_rule_color_case() {
    let source = r#".a { column-rule: medium none currentcolor; column-rule-color: currentcolor; color: currentcolor; }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::ColorCompression,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 1);
    assert_eq!(
        execution.output_css,
        r#".a { column-rule: medium none currentcolor; column-rule-color: currentcolor; color: currentColor; }"#
    );
}

#[test]
fn execution_runtime_removes_adjacent_duplicate_color_declarations_after_compression() {
    let source = r#".a { color: rgb(255 0 0); color: rgb(255 0 0 / 100%); background: blue; background: #0000FF; } .b { color: red; margin: 1px; color: red; } .important { color: red !important; color: red !important; }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::ColorCompression,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 6);
    assert_eq!(
        execution.output_css,
        r#".a { color: red;  background: #00f;  } .b { color: red; margin: 1px; color: red; } .important { color: red !important; color: red !important; }"#
    );
}

#[test]
fn execution_runtime_preserves_minified_declaration_shape_for_value_replacements() {
    let source = ".a{background:blue}.b{margin:calc(2rem + 3rem)}.c{width:calc(2px * 3);height:calc(6px / 2)}";
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::ColorCompression,
            TransformPassKind::CalcReduction,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 4);
    assert_eq!(
        execution.output_css,
        ".a{background:#00f}.b{margin:5rem}.c{width:6px;height:3px}"
    );
}

#[test]
fn execution_runtime_strips_safe_url_quotes_only() {
    let source = r#".a { background: url("img/icon.svg"); mask: url("has space.svg"); content: "url(\"keep\")"; }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::UrlQuoteStrip,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 1);
    assert_eq!(
        execution.output_css,
        r#".a { background: url(img/icon.svg); mask: url("has space.svg"); content: "url(\"keep\")"; }"#
    );
}

#[test]
fn execution_runtime_normalizes_safe_strings_without_rewriting_semantic_strings() {
    let source = r#".a { font-family: 'Demo'; content: 'has "quote"'; background: url('asset.svg'); } .b { font-family: "serif"; } .c { font-family: "Open Sans", "Helvetica Neue", "system-ui"; } .d { font-family: "--brand"; }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::StringQuoteNormalize,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 4);
    assert_eq!(
        execution.output_css,
        r#".a { font-family: Demo; content: 'has "quote"'; background: url("asset.svg"); } .b { font-family: "serif"; } .c { font-family: Open Sans,Helvetica Neue,"system-ui"; } .d { font-family: --brand; }"#
    );
}

#[test]
fn execution_runtime_normalizes_static_font_longhand_keywords() {
    let source = r#".a { font-weight: normal; font-stretch: normal; } .b { font-weight: bold; font-stretch: condensed; } .c { font-weight: bolder; font-stretch: 80%; } .d { font-stretch: 100%; color: red; font-stretch: 50%; font-weight: normal; font-weight: 700; } .important { font-stretch: 100% !important; font-stretch: 50%; } .bad { font-stretch: 100%; font-stretch: bad; }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::StringQuoteNormalize,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 7);
    assert_eq!(
        execution.output_css,
        r#".a { font-weight: 400; font-stretch: 100%; } .b { font-weight: 700; font-stretch: 75%; } .c { font-weight: bolder; font-stretch: 80%; } .d {  color: red; font-stretch: 50%;  font-weight: 700; } .important { font-stretch: 100% !important; font-stretch: 50%; } .bad { font-stretch: 100%; font-stretch: bad; }"#
    );
}

#[test]
fn execution_runtime_combines_static_font_longhands() {
    let source = r#".a { font-style: normal; font-variant-caps: normal; font-weight: normal; font-stretch: normal; font-size: 16px; line-height: normal; font-family: Arial; } .b { font-style: normal; font-variant-caps: normal; font-weight: bold; font-stretch: condensed; font-size: 16px; line-height: 1.5; font-family: Arial, sans-serif; } .c { font-style: italic; font-variant-caps: small-caps; font-weight: bold; font-stretch: condensed; font-size: 1rem; line-height: 120%; font-family: "Open Sans", serif; } .d { font-style: normal !important; font-variant-caps: normal !important; font-weight: normal !important; font-stretch: normal !important; font-size: 16px !important; line-height: normal !important; font-family: Arial !important; }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::StringQuoteNormalize,
            TransformPassKind::ShorthandCombining,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 4);
    assert_eq!(
        execution.output_css,
        r#".a { font: 16px Arial; } .b { font: 700 75% 16px/1.5 Arial,sans-serif; } .c { font: italic small-caps 700 75% 1rem/120% Open Sans,serif; } .d { font: 16px Arial!important; }"#
    );
}

#[test]
fn execution_runtime_normalizes_static_display_multi_keywords() {
    let source = r#".a { display: block flow; } .b { display: inline flow; } .c { display: block flow-root; } .d { display: inline flow-root; } .e { display: inline flex; } .f { display: block grid; } .g { display: list-item block flow; } .h { display: block ruby; }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::StringQuoteNormalize,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 7);
    assert_eq!(
        execution.output_css,
        r#".a { display: block; } .b { display: inline; } .c { display: flow-root; } .d { display: inline-block; } .e { display: inline-flex; } .f { display: grid; } .g { display: list-item; } .h { display: block ruby; }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["string-quote-normalize", "print-css"]
    );
}

#[test]
fn execution_runtime_compresses_specificity_safe_is_where_selectors() {
    let source = r#".a:is(.ready) { color: red; } .b:where(.x, .x) { color: blue; } .c:where(.y) { color: green; } .d:is(:is(.u, .v), .u) { color: orange; } .g:is(.p, .q):hover { color: lime; } .e, .e, .f { color: purple; } .w:where(:where(.one, .two), .one) { color: teal; } @media (min-width: 1px) { .m, .m, .n { color: black; } } @supports (display: grid) { .s, .s { display: grid; } }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::SelectorIsWhereCompression,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 9);
    assert_eq!(
        execution.output_css,
        r#".a.ready { color: red; } .b:where(.x) { color: blue; } .c:where(.y) { color: green; } .d.u, .d.v { color: orange; } .g.p:hover, .g.q:hover { color: lime; } .e, .f { color: purple; } .w:where(.one,.two) { color: teal; } @media (min-width: 1px) { .m, .n { color: black; } } @supports (display: grid) { .s { display: grid; } }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["selector-is-where-compression", "print-css"]
    );
}

#[test]
fn execution_runtime_normalizes_keyframe_selector_aliases() {
    let source = r#"@keyframes fade { from { opacity: 0; } 100% { opacity: 1; } 50%, TO { opacity: .5; } } @-webkit-keyframes spin { FROM { transform: rotate(0deg); } }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::SelectorIsWhereCompression,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 4);
    assert_eq!(
        execution.output_css,
        r#"@keyframes fade { 0%{ opacity: 0; } to{ opacity: 1; } 50%,to{ opacity: .5; } } @-webkit-keyframes spin { 0%{ transform: rotate(0deg); } }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["selector-is-where-compression", "print-css"]
    );
}

#[test]
fn execution_runtime_removes_only_plain_empty_rules() {
    let source = r#".empty { } @media (min-width: 1px) { .nested { } } .outer { .inner { } } .with-comment { /* keep */ } .filled { color: red; }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::EmptyRuleRemoval,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 5);
    assert_eq!(
        execution.output_css,
        r#"   .with-comment { /* keep */ } .filled { color: red; }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["empty-rule-removal", "print-css"]
    );
}

#[test]
fn execution_runtime_removes_comment_only_rules_after_comment_strip() {
    let source = r#".empty { } @media (min-width: 1px) { .nested { } .filled { color: red; } } .outer { .inner { } } .with-comment { /* remove after comment strip */ } .filled { color: red; }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::EmptyRuleRemoval,
            TransformPassKind::CommentStrip,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(
        execution.ordered_pass_ids,
        vec!["comment-strip", "empty-rule-removal", "print-css"]
    );
    assert_eq!(execution.mutation_count, 6);
    assert_eq!(
        execution.output_css,
        r#" @media (min-width: 1px) {  .filled { color: red; } }   .filled { color: red; }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["comment-strip", "empty-rule-removal", "print-css"]
    );
}

#[test]
fn execution_runtime_preserves_empty_keyframe_frames() {
    let source = r#"@keyframes fade { 0% {} to { opacity: 1 } } .empty{}"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::EmptyRuleRemoval,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 1);
    assert_eq!(
        execution.output_css,
        r#"@keyframes fade { 0% {} to { opacity: 1 } } "#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["empty-rule-removal", "print-css"]
    );
}

#[test]
fn execution_runtime_combines_adjacent_box_longhands_with_cascade_proof() {
    let source = r#".a { margin-top: 1px; margin-right: 2px; margin-bottom: 1px; margin-left: 2px; border-top-color: red; border-right-color: blue; border-bottom-color: red; border-left-color: blue; border-top-width: 1px; border-right-width: 2px; border-bottom-width: 3px; border-left-width: 2px; padding-top: 1px; color: red; padding-right: 2px; padding-bottom: 3px; padding-left: 4px; }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::ShorthandCombining,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 3);
    assert_eq!(
        execution.output_css,
        r#".a { margin: 1px 2px; border-color: red blue; border-width: 1px 2px 3px; padding-top: 1px; color: red; padding-right: 2px; padding-bottom: 3px; padding-left: 4px; }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["shorthand-combining", "print-css"]
    );
}

#[test]
fn execution_runtime_compresses_box_shorthand_values() {
    let source = r#".a { margin: 1px 1px 1px 1px; padding: 1px 2px 3px 2px; border-color: red blue red blue; border-width: 1px 1px; border-style: solid solid solid solid; border-image-slice: 100% 100% 100% 100%; border-image-width: 1 1 1 1; border-image-outset: 0 0 0 0; border: medium none currentColor; border-top: currentColor medium none; outline: medium none currentColor; } .important { margin: 1px 1px 1px 1px !important; border: medium none currentColor !important; }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::ShorthandCombining,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 11);
    assert_eq!(
        execution.output_css,
        r#".a { margin: 1px; padding: 1px 2px 3px; border-color: red blue; border-width: 1px; border-style: solid; border-image-slice: 100%; border-image-width: 1; border-image-outset: 0; border: none; border-top: none; outline: none; } .important { margin: 1px 1px 1px 1px !important; border: medium none currentColor !important; }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["shorthand-combining", "print-css"]
    );
}

#[test]
fn execution_runtime_compresses_border_image_longhands() {
    let source = r#".a { border-image-source: url(a.png); border-image-slice: 10; border-image-width: 1; border-image-outset: 0; border-image-repeat: stretch; } .b { border-image-source: linear-gradient(red,#00f); border-image-slice: 10 20; border-image-width: auto; border-image-outset: 1; border-image-repeat: round; } .c { border-image-source: none; border-image-slice: 10; border-image-width: 1; border-image-outset: 0; border-image-repeat: stretch; } .d { border-image-source: url(a.png); border-image-slice: 10 fill; border-image-width: 2; border-image-outset: 0; border-image-repeat: round space; } .invalid { border-image-source: url(a.png); border-image-slice: 10; border-image-width: fill; border-image-outset: 0; border-image-repeat: stretch; } .default { border-image-source: none; border-image-slice: 100%; border-image-width: 1; border-image-outset: 0; border-image-repeat: stretch; }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::ShorthandCombining,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 4);
    assert_eq!(
        execution.output_css,
        r#".a { border-image: url(a.png) 10; } .b { border-image: linear-gradient(red,#00f) 10 20/auto/1 round; } .c { border-image: 10; } .d { border-image: url(a.png) 10 fill/2 round space; } .invalid { border-image-source: url(a.png); border-image-slice: 10; border-image-width: fill; border-image-outset: 0; border-image-repeat: stretch; } .default { border-image-source: none; border-image-slice: 100%; border-image-width: 1; border-image-outset: 0; border-image-repeat: stretch; }"#
    );
}

#[test]
fn execution_runtime_compresses_overflow_and_background_repeat_shorthands() {
    let source = r#".a { overflow-x: visible; overflow-y: visible; background-repeat: repeat repeat; } .b { overflow-x: hidden; color: red; overflow-y: hidden; background-repeat: round space; } .c { background-repeat: Repeat Repeat; } .d { overflow: hidden hidden; background-repeat: repeat no-repeat; } .e { overflow: visible visible; background-repeat: no-repeat repeat; } .f { overflow-x: auto; overflow-y: hidden; } .g { overflow-y: scroll; overflow-x: clip; } .h { overflow: AUTO HIDDEN; } .pos { background-position-x: left; background-position-y: top; } .pos-center { background-position-x: center; background-position-y: center; } .pos-reverse { background-position-y: top; background-position-x: center; } .pos-important { background-position-x: left !important; background-position-y: top !important; } .important { overflow-x: auto !important; overflow-y: auto !important; background-repeat: no-repeat no-repeat !important; }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::ShorthandCombining,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 14);
    assert_eq!(
        execution.output_css,
        r#".a { overflow: visible; background-repeat: repeat; } .b { overflow-x: hidden; color: red; overflow-y: hidden; background-repeat: round space; } .c { background-repeat: repeat; } .d { overflow: hidden; background-repeat: repeat-x; } .e { overflow: visible; background-repeat: repeat-y; } .f { overflow: auto hidden; } .g { overflow: clip scroll; } .h { overflow: auto hidden; } .pos { background-position: 0 0; } .pos-center { background-position: 50%; } .pos-reverse { background-position: top; } .pos-important { background-position: 0 0!important; } .important { overflow-x: auto !important; overflow-y: auto !important; background-repeat: no-repeat no-repeat !important; }"#
    );
}

#[test]
fn execution_runtime_compresses_place_axis_shorthands() {
    let source = r#".items { align-items: stretch; justify-items: stretch; } .content { align-content: center; justify-content: center; } .self { justify-self: end; align-self: start; } .important { align-items: start !important; justify-items: end !important; } .mixed { align-items: first baseline; justify-items: center; } .legacy { justify-items: legacy left; align-items: normal; } .safe { align-self: safe center; justify-self: unsafe end; } .content-multi { align-content: space-between; justify-content: first baseline; } .content-shorthand { place-content: normal normal; } .items-stretch { place-items: stretch stretch; } .self-auto { place-self: auto auto; }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::ShorthandCombining,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 9);
    assert_eq!(
        execution.output_css,
        r#".items { place-items: stretch stretch; } .content { place-content: center; } .self { place-self: start end; } .important { place-items: start end!important; } .mixed { place-items: baseline center; } .legacy { place-items: normal legacy left; } .safe { place-self: safe center unsafe end; } .content-multi { align-content: space-between; justify-content: first baseline; } .content-shorthand { place-content: normal; } .items-stretch { place-items: stretch stretch; } .self-auto { place-self: auto; }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["shorthand-combining", "print-css"]
    );
}

#[test]
fn execution_runtime_compresses_gap_axis_shorthands() {
    let source = r#".a { row-gap: 1px; column-gap: 1px; } .b { gap: 2px 2px; } .c { column-gap: 2px; row-gap: 1px; } .important { row-gap: 1px !important; column-gap: 2px !important; } .mixed { row-gap: calc(1px + 1px); column-gap: 2px; }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::ShorthandCombining,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 5);
    assert_eq!(
        execution.output_css,
        r#".a { gap: 1px; } .b { gap: 2px; } .c { gap: 1px 2px; } .important { gap: 1px 2px!important; } .mixed { gap: calc(1px + 1px) 2px; }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["shorthand-combining", "print-css"]
    );
}

#[test]
fn execution_runtime_compresses_scroll_box_shorthands() {
    let source = r#".a { scroll-margin-top: 1px; scroll-margin-right: 2px; scroll-margin-bottom: 1px; scroll-margin-left: 2px; } .b { scroll-padding-top: 1px; scroll-padding-right: 1px; scroll-padding-bottom: 1px; scroll-padding-left: 1px; } .c { scroll-margin: 3px 3px; } .important { scroll-margin-top: 1px !important; scroll-margin-right: 2px !important; scroll-margin-bottom: 1px !important; scroll-margin-left: 2px !important; }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::ShorthandCombining,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 3);
    assert_eq!(
        execution.output_css,
        r#".a { scroll-margin: 1px 2px; } .b { scroll-padding: 1px; } .c { scroll-margin: 3px; } .important { scroll-margin-top: 1px !important; scroll-margin-right: 2px !important; scroll-margin-bottom: 1px !important; scroll-margin-left: 2px !important; }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["shorthand-combining", "print-css"]
    );
}

#[test]
fn execution_runtime_compresses_text_decoration_shorthands() {
    let source = r#".a { text-decoration-line: underline; text-decoration-style: solid; text-decoration-color: currentcolor; text-decoration-thickness: auto; } .b { text-decoration: underline solid red auto; } .c { text-decoration-line: underline; text-decoration-style: wavy; text-decoration-color: red; text-decoration-thickness: 1px; } .important { text-decoration-line: underline !important; text-decoration-style: solid !important; text-decoration-color: currentcolor !important; text-decoration-thickness: auto !important; } .mixed { text-decoration-line: underline overline; text-decoration-style: solid; text-decoration-color: currentcolor; text-decoration-thickness: auto; } .em-a { text-emphasis-style: none; text-emphasis-color: currentcolor; } .em-b { text-emphasis-style: filled dot; text-emphasis-color: red; } .em-c { text-emphasis-style: open sesame !important; text-emphasis-color: currentcolor !important; } .pos-a { text-emphasis-position: over right; } .pos-b { text-emphasis-position: left under; } .pos-c { text-emphasis-position: over left; }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::ShorthandCombining,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 10);
    assert_eq!(
        execution.output_css,
        r#".a { text-decoration: underline; } .b { text-decoration: underline red; } .c { text-decoration: underline 1px wavy red; } .important { text-decoration: underline!important; } .mixed { text-decoration: underline overline; } .em-a { text-emphasis: none; } .em-b { text-emphasis: dot red; } .em-c { text-emphasis: open sesame!important; } .pos-a { text-emphasis-position: over; } .pos-b { text-emphasis-position: under left; } .pos-c { text-emphasis-position: over left; }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["shorthand-combining", "print-css"]
    );
}

#[test]
fn execution_runtime_compresses_logical_axis_shorthands() {
    let source = r#".a { padding-block-start: 1px; padding-block-end: 1px; } .b { margin-inline-start: 1px; margin-inline-end: 2px; } .c { inset-block-end: 2px; inset-block-start: 1px; } .d { border-block-start-color: red; border-block-end-color: red; } .e { border-inline-start-width: 1px; border-inline-end-width: 2px; } .f { scroll-margin-block-start: 1px; scroll-margin-block-end: 1px; } .g { scroll-padding-inline-end: 2px; scroll-padding-inline-start: 1px; } .h { inset-block-start: 1px; inset-inline-end: 2px; inset-block-end: 1px; inset-inline-start: 2px; } .border-all { border-block-start-width: 1px; border-block-end-width: 1px; border-inline-start-width: 1px; border-inline-end-width: 1px; } .important { padding-block-start: 1px !important; padding-block-end: 2px !important; } .mixed { padding-block-start: calc(1px + 1px); padding-block-end: 2px; }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::ShorthandCombining,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 11);
    assert_eq!(
        execution.output_css,
        r#".a { padding-block: 1px; } .b { margin-inline: 1px 2px; } .c { inset-block: 1px 2px; } .d { border-block-color: red; } .e { border-inline-width: 1px 2px; } .f { scroll-margin-block: 1px; } .g { scroll-padding-inline: 1px 2px; } .h { inset-block: 1px; inset-inline: 2px; } .border-all { border-width: 1px; } .important { padding-block: 1px 2px!important; } .mixed { padding-block: calc(1px + 1px) 2px; }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["shorthand-combining", "print-css"]
    );
}

#[test]
fn execution_runtime_compresses_line_style_shorthands() {
    let source = r#".a { border-top-width: 1px; border-top-style: solid; border-top-color: red; } .b { border-width: medium; border-style: none; border-color: currentcolor; } .c { outline-width: medium; outline-style: solid; outline-color: currentcolor; } .d { outline-width: 1px; outline-style: none; outline-color: red; } .e { border-inline-width: medium !important; border-inline-style: none !important; border-inline-color: currentcolor !important; } .f { border-color: red; border-style: solid; border-width: 1px; } .g { border-top: 1px solid red; border-right: 1px solid red; border-bottom: 1px solid red; border-left: 1px solid red; } .mixed { border-top-width: 1px; color: blue; border-top-style: solid; border-top-color: red; }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::ShorthandCombining,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 7);
    assert_eq!(
        execution.output_css,
        r#".a { border-top: 1px solid red; } .b { border: none; } .c { outline: solid; } .d { outline: 1px red; } .e { border-inline: none!important; } .f { border: 1px solid red; } .g { border: 1px solid red; } .mixed { border-top-width: 1px; color: blue; border-top-style: solid; border-top-color: red; }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["shorthand-combining", "print-css"]
    );
}

#[test]
fn execution_runtime_compresses_logical_border_line_shorthands() {
    let source = r#".a { border-block-start-width: 1px; border-block-start-style: solid; border-block-start-color: red; } .b { border-block-start: 1px solid red; border-block-end: 1px solid red; } .c { border-block-start-width: 1px; border-block-start-style: solid; border-block-start-color: red; border-block-end-width: 1px; border-block-end-style: solid; border-block-end-color: red; } .d { border-inline-end: 1px solid red; border-inline-start: 1px solid red; } .e { border-inline-end-width: medium !important; border-inline-end-style: none !important; border-inline-end-color: currentcolor !important; border-inline-start-width: medium !important; border-inline-start-style: none !important; border-inline-start-color: currentcolor !important; } .different { border-block-start: 1px solid red; border-block-end: 2px solid red; }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::ShorthandCombining,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 5);
    assert_eq!(
        execution.output_css,
        r#".a { border-block-start: 1px solid red; } .b { border-block: 1px solid red; } .c { border-block: 1px solid red; } .d { border-inline: 1px solid red; } .e { border-inline: none!important; } .different { border-block-start: 1px solid red; border-block-end: 2px solid red; }"#
    );
}

#[test]
fn execution_runtime_compresses_repeated_axis_shorthand_values() {
    let source = r#".a { mask-repeat: repeat repeat; -webkit-mask-repeat: no-repeat no-repeat; background-repeat: space round; } .b { border-spacing: 1px 1px; } .c { scroll-padding-inline: 1px 1px; scroll-margin-block: 1px 2px; } .d { padding-inline: 2px 2px; margin-block: 1px 2px; } .e { border-block-color: red red; border-inline-width: 1px 1px; } .f { background-repeat: repeat no-repeat; mask-repeat: no-repeat repeat; -webkit-mask-repeat: repeat no-repeat; }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::ShorthandCombining,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 10);
    assert_eq!(
        execution.output_css,
        r#".a { mask-repeat: repeat; -webkit-mask-repeat: no-repeat; background-repeat: space round; } .b { border-spacing: 1px; } .c { scroll-padding-inline: 1px; scroll-margin-block: 1px 2px; } .d { padding-inline: 2px; margin-block: 1px 2px; } .e { border-block-color: red; border-inline-width: 1px; } .f { background-repeat: repeat-x; mask-repeat: repeat-y; -webkit-mask-repeat: repeat-x; }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["shorthand-combining", "print-css"]
    );
}

#[test]
fn execution_runtime_compresses_static_flex_shorthands() {
    let source = r#".a { flex: 0 1 auto; } .b { flex: 1 1 0%; } .c { flex: 2 1 0%; } .d { flex: 1 2 0%; } .e { flex: var(--flex); } .f { flex: 0 0 auto; } .g { flex-flow: row nowrap; } .h { flex-flow: row wrap; } .i { flex-flow: nowrap row; } .j { flex-direction: row; flex-wrap: nowrap; } .k { flex-wrap: wrap; flex-direction: column; } .l { flex-direction: row !important; flex-wrap: nowrap !important; } .m { flex-basis: 0%; flex: 1 1 0%; } .n { flex-basis: 0% !important; flex: 1; } .o { flex-grow: 1; flex-shrink: 1; flex: 2 1 0%; }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::ShorthandCombining,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 16);
    assert_eq!(
        execution.output_css,
        r#".a { flex: 0 auto; } .b { flex: 1; } .c { flex: 2; } .d { flex: 1 2; } .e { flex: var(--flex); } .f { flex: none; } .g { flex-flow: row; } .h { flex-flow: wrap; } .i { flex-flow: row; } .j { flex-flow: row; } .k { flex-flow: column wrap; } .l { flex-flow: row!important; } .m {  flex: 1; } .n { flex-basis: 0% !important; flex: 1; } .o {   flex: 2; }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["shorthand-combining", "print-css"]
    );
}

#[test]
fn execution_runtime_compresses_static_motion_shorthands() {
    let source = r#".a { transition: all 0s ease 0s; } .b { transition: opacity 0s linear .1s; } .c { transition: opacity 0s ease 0s, color .2s ease 0s; } .d { animation: none 0s ease 0s 1 normal none running; } .e { animation: 0s ease 0s 1 normal none running fade; } .f { animation: fade .2s ease 0s 1 normal none running; } .g { transition-property: all; transition-duration: 0s; transition-timing-function: ease; transition-delay: 0s; } .h { transition-property: opacity; transition-duration: .2s; transition-timing-function: ease; transition-delay: 0s; } .i { transition-property: all !important; transition-duration: 0s !important; transition-timing-function: ease !important; transition-delay: 0s !important; }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::ShorthandCombining,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 7);
    assert_eq!(
        execution.output_css,
        r#".a { transition: all; } .b { transition: opacity 0s linear .1s; } .c { transition: opacity,color .2s; } .d { animation: none; } .e { animation: fade; } .f { animation: fade .2s ease 0s 1 normal none running; } .g { transition: all; } .h { transition: opacity .2s; } .i { transition: all!important; }"#
    );
}

#[test]
fn execution_runtime_compresses_border_radius_shorthands() {
    let source = r#".a { border-radius: 1px 1px 1px 1px; border-top-left-radius: 1px; border-top-right-radius: 2px; border-bottom-right-radius: 1px; border-bottom-left-radius: 2px; } .b { border-radius: 1px / 2px; border-top-left-radius: 1px 2px; border-top-right-radius: 2px; border-bottom-right-radius: 1px; border-bottom-left-radius: 2px; } .c { border-radius: 1px 1px 1px 1px / 2px 2px 2px 2px; } .d { border-radius: 1px 2px 1px 2px / 3px 4px 3px 4px; } .e { border-radius: 1px / 1px; }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::ShorthandCombining,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 7);
    assert_eq!(
        execution.output_css,
        r#".a { border-radius: 1px; border-radius: 1px 2px; } .b { border-radius: 1px/2px; border-radius: 1px 2px/2px 2px 1px; } .c { border-radius: 1px/2px; } .d { border-radius: 1px 2px/3px 4px; } .e { border-radius: 1px; }"#
    );
}

#[test]
fn execution_runtime_compresses_inset_shorthands() {
    let source = r#".a { inset: 1px 2px 1px 2px; top: 1px; right: 2px; bottom: 1px; left: 2px; } .b { top: 1px; color: red; right: 2px; bottom: 1px; left: 2px; } .important { top: 1px !important; right: 2px !important; bottom: 1px !important; left: 2px !important; }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::ShorthandCombining,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 2);
    assert_eq!(
        execution.output_css,
        r#".a { inset: 1px 2px; inset: 1px 2px; } .b { top: 1px; color: red; right: 2px; bottom: 1px; left: 2px; } .important { top: 1px !important; right: 2px !important; bottom: 1px !important; left: 2px !important; }"#
    );
}

#[test]
fn execution_runtime_compresses_list_style_shorthands() {
    let source = r#".a { list-style: disc outside none; list-style-type: none; list-style-position: outside; list-style-image: none; } .b { list-style-type: decimal; list-style-position: inside; list-style-image: none; } .c { list-style-type: disc; color: red; list-style-position: outside; list-style-image: none; } .d { list-style: none outside none; } .e { list-style: url(icon.svg) outside none; } .f { list-style: NONE OUTSIDE NONE; } .important { list-style-type: none !important; list-style-position: outside !important; list-style-image: none !important; }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::ShorthandCombining,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 6);
    assert_eq!(
        execution.output_css,
        r#".a { list-style: outside; list-style: none; } .b { list-style: inside decimal; } .c { list-style-type: disc; color: red; list-style-position: outside; list-style-image: none; } .d { list-style: none; } .e { list-style: url(icon.svg) none; } .f { list-style: none; } .important { list-style-type: none !important; list-style-position: outside !important; list-style-image: none !important; }"#
    );
}

#[test]
fn execution_runtime_rewrites_declaration_values_inside_group_rules() {
    let source = r#"@media (min-width: 1px) { .a { width: calc(1px + 1px); margin: 1px 1px 1px 1px; color: blue; } } @supports (display: grid) { .b { color: blue; } }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::ShorthandCombining,
            TransformPassKind::CalcReduction,
            TransformPassKind::ColorCompression,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 4);
    assert_eq!(
        execution.output_css,
        r#"@media (min-width: 1px) { .a { width: 2px; margin: 1px; color: #00f; } } @supports (display: grid) { .b { color: #00f; } }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec![
            "shorthand-combining",
            "calc-reduction",
            "color-compression",
            "print-css"
        ]
    );
}

#[test]
fn execution_runtime_removes_cascade_safe_duplicate_rules() {
    let source = r#".a { color: red; } .b { color: red; } .a { color: blue; } .a { color: red; } @media (min-width: 1px) { .m { color: red; } .x { color: blue; } .m { color: red; } } @media (max-width: 1px) { .m { color: red; } }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::RuleDeduplication,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 2);
    assert_eq!(
        execution.output_css,
        r#" .b { color: red; } .a { color: blue; } .a { color: red; } @media (min-width: 1px) {  .x { color: blue; } .m { color: red; } } @media (max-width: 1px) { .m { color: red; } }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["rule-deduplication", "print-css"]
    );
}

#[test]
fn execution_runtime_merges_adjacent_same_selector_rules_only() {
    let source = r#".a { color: red; } .a { background: blue; } .a { outline: 0; } .b { color: red; } .a { border: 0; } @media (min-width: 1px) { .m { color: red; } .m { background: blue; } }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[TransformPassKind::RuleMerging, TransformPassKind::PrintCss],
    );

    assert_eq!(execution.mutation_count, 2);
    assert_eq!(
        execution.output_css,
        r#".a { color: red; background: blue; outline: 0; } .b { color: red; } .a { border: 0; } @media (min-width: 1px) { .m { color: red; background: blue; } }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["rule-merging", "print-css"]
    );
}

#[test]
fn execution_runtime_preserves_declaration_boundaries_when_merging_semicolonless_rules() {
    let source = r#".b{color:red}.b{background:blue} @media (min-width: 1px) { .m { color: red } .m { background: blue } }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[TransformPassKind::RuleMerging, TransformPassKind::PrintCss],
    );

    assert_eq!(execution.mutation_count, 2);
    assert_eq!(
        execution.output_css,
        r#".b { color:red; background:blue; } @media (min-width: 1px) { .m { color: red; background: blue; } }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["rule-merging", "print-css"]
    );
}

#[test]
fn execution_runtime_merges_adjacent_same_conditional_wrappers() {
    let source = r#"@media (prefers-color-scheme: dark) { .card { color: white; } } @media (prefers-color-scheme: dark) { .card .title { color: #ddd; } } @supports (display: grid) { .grid { display: grid; } } @supports (display: flex) { .flex { display: flex; } } @supports (display: flex) { .flex .child { display: flex; } }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[TransformPassKind::RuleMerging, TransformPassKind::PrintCss],
    );

    assert_eq!(execution.mutation_count, 2);
    assert_eq!(
        execution.output_css,
        r#"@media (prefers-color-scheme: dark) { .card { color: white; } .card .title { color: #ddd; } } @supports (display: grid) { .grid { display: grid; } } @supports (display: flex) { .flex { display: flex; } .flex .child { display: flex; } }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["rule-merging", "print-css"]
    );
}

#[test]
fn execution_runtime_merges_adjacent_same_block_selectors_only() {
    let source = r#".a { color: red; } .b { color: red; } .c { color: red; } .d { color: blue; } .e { color: red; } .x{color:red;}.y{color:red} @media (min-width: 1px) { .m { color: black; } .n { color: black; } }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::SelectorMerging,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 3);
    assert_eq!(
        execution.output_css,
        r#".a, .b, .c { color: red; } .d { color: blue; } .e, .x, .y { color: red; } @media (min-width: 1px) { .m, .n { color: black; } }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["selector-merging", "print-css"]
    );
}

#[test]
fn execution_runtime_adds_conservative_vendor_prefixes_when_absent() {
    let source = r#".a { user-select: none; -webkit-appearance: none; appearance: none; backdrop-filter: blur(2px); } .flex { display: flex; position: sticky; } .inline { display: -webkit-inline-box; display: inline-flex; } .extra { text-size-adjust: 100%; mask-image: linear-gradient(red, blue); hyphens: auto; } .print { print-color-adjust: exact; -webkit-mask-size: cover; mask-size: cover; } @keyframes fade { from { opacity: 0; } to { opacity: 1; } } @-webkit-keyframes spin { from { opacity: 0; } } @keyframes spin { from { opacity: 0; } }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::VendorPrefixing,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 15);
    assert_eq!(
        execution.output_css,
        r#".a { -webkit-user-select: none; -moz-user-select: none; -ms-user-select: none; user-select: none; -webkit-appearance: none; -moz-appearance: none; appearance: none; -webkit-backdrop-filter: blur(2px); backdrop-filter: blur(2px); } .flex { display: -webkit-box; display: -ms-flexbox; display: flex; position: -webkit-sticky; position: sticky; } .inline { display: -webkit-inline-box; display: -ms-inline-flexbox; display: inline-flex; } .extra { -webkit-text-size-adjust: 100%; text-size-adjust: 100%; -webkit-mask-image: linear-gradient(red, blue); mask-image: linear-gradient(red, blue); -webkit-hyphens: auto; -ms-hyphens: auto; hyphens: auto; } .print { -webkit-print-color-adjust: exact; print-color-adjust: exact; -webkit-mask-size: cover; mask-size: cover; } @-webkit-keyframes fade { from { opacity: 0; } to { opacity: 1; } } @keyframes fade { from { opacity: 0; } to { opacity: 1; } } @-webkit-keyframes spin { from { opacity: 0; } } @keyframes spin { from { opacity: 0; } }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["vendor-prefixing", "print-css"]
    );
}

#[test]
fn execution_runtime_lowers_whole_value_light_dark_declarations() {
    let source = r#".card { color: light-dark(#000, #fff); background: linear-gradient(light-dark(red, blue), white); border: 1px solid light-dark(red, blue); box-shadow: 0 0 1px light-dark(black, white); }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::LightDarkLowering,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 4);
    assert_eq!(
        execution.output_css,
        r#".card { color: #000; background: linear-gradient(red, white); border: 1px solid red; box-shadow: 0 0 1px black; } @media (prefers-color-scheme: dark) { .card { color: #fff; } } @media (prefers-color-scheme: dark) { .card { background: linear-gradient(blue, white); } } @media (prefers-color-scheme: dark) { .card { border: 1px solid blue; } } @media (prefers-color-scheme: dark) { .card { box-shadow: 0 0 1px white; } }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["light-dark-lowering", "print-css"]
    );
}

#[test]
fn execution_runtime_lowers_static_srgb_color_mix_declarations() {
    let source = r#".card { color: color-mix(in srgb, red 50%, blue 50%); background-color: color-mix(in srgb, #000, #fff 25%); outline-color: color-mix(in srgb, rgb(255 0 0) 25%, hsl(240 100% 50%) 75%); text-decoration-color: color-mix(in srgb, hwb(120 0% 50%) 40%, white 60%); caret-color: color-mix(in srgb, black 12.5%, white 87.5%); background: linear-gradient(color-mix(in srgb, red 25%, blue 75%), white); accent-color: color-mix(in srgb, red 25%, blue 25%); fill: color-mix(in srgb, red 75%, blue 75%); stroke: color-mix(in srgb, red 0%, blue 0%); border: 1px solid color-mix(in srgb, red, blue); box-shadow: 0 0 1px color-mix(in srgb, red, blue); column-rule: 1px solid color-mix(in srgb, red, blue); border-color: color-mix(in oklab, red, blue); }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::ColorMixLowering,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 11);
    assert_eq!(
        execution.output_css,
        r#".card { color: rgb(128 0 128); background-color: rgb(64 64 64); outline-color: rgb(64 0 191); text-decoration-color: rgb(153 204 153); caret-color: rgb(223 223 223); background: linear-gradient(rgb(64 0 191), white); accent-color: rgb(128 0 128 / .5); fill: rgb(128 0 128); stroke: color-mix(in srgb, red 0%, blue 0%); border: 1px solid rgb(128 0 128); box-shadow: 0 0 1px rgb(128 0 128); column-rule: 1px solid rgb(128 0 128); border-color: color-mix(in oklab, red, blue); }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["color-mix-lowering", "print-css"]
    );
}

#[test]
fn execution_runtime_lowers_alpha_aware_srgb_color_mix_declarations() {
    let source = r#".card { color: color-mix(in srgb, 50% red, transparent 50%); background-color: color-mix(in srgb, 25% rgb(100% 0% 0% / .7), rgb(0% 100% 0% / .2)); outline-color: color-mix(in srgb, rgb(100% 0% 0% / .7) 20%, 60% rgb(0% 100% 0% / .2)); border-color: color-mix(in srgb, 50% #ff000080, 50% blue); }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::ColorMixLowering,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 4);
    assert_eq!(
        execution.output_css,
        r#".card { color: rgb(255 0 0 / .5); background-color: rgb(137 118 0 / .325); outline-color: rgb(137 118 0 / .26); border-color: rgb(85 0 170 / .75098); }"#
    );
}

#[test]
fn execution_runtime_lowers_linear_srgb_color_mix_declarations() {
    let source = r#".card { color: color-mix(in srgb-linear, red 50%, blue 50%); background-color: color-mix(in srgb-linear, 50% red, transparent 50%); outline-color: color-mix(in srgb-linear, 25% rgb(100% 0% 0% / .7), rgb(0% 100% 0% / .2)); border-color: color-mix(in srgb-linear, 50% #ff000080, 50% blue); }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::ColorMixLowering,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 4);
    assert_eq!(
        execution.output_css,
        r#".card { color: rgb(188 0 188); background-color: rgb(255 0 0 / .5); outline-color: rgb(194 181 0 / .325); border-color: rgb(156 0 213 / .75098); }"#
    );
}

#[test]
fn execution_runtime_lowers_in_gamut_oklab_oklch_declarations() {
    let source = r#".card { color: oklab(1 0 0); background-color: oklch(0% 0 0deg); outline-color: oklch(0% 0 0.5TURN); background: linear-gradient(oklch(0% 0 0deg), white); accent-color: oklch(0% 0 0deg / .5); box-shadow: 0 0 1px oklch(0% 0 0deg); column-rule: 1px solid oklab(1 0 0); border-color: oklch(70% 0.4 40deg); }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::OklchOklabLowering,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 7);
    assert_eq!(
        execution.output_css,
        r#".card { color: rgb(255 255 255); background-color: rgb(0 0 0); outline-color: rgb(0 0 0); background: linear-gradient(rgb(0 0 0), white); accent-color: rgb(0 0 0 / .5); box-shadow: 0 0 1px rgb(0 0 0); column-rule: 1px solid rgb(255 255 255); border-color: oklch(70% 0.4 40deg); }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["oklch-oklab-lowering", "print-css"]
    );
}

#[test]
fn execution_runtime_lowers_static_srgb_color_function_declarations() {
    let source = r#".card { color: color(srgb 1 0 0); background-color: color(srgb 50% 25% 0% / 100%); outline-color: color(srgb 0 0 1 / 1); fill: color(display-p3 0.5 0.5 0.5 / 100%); background: linear-gradient(color(srgb 1 0 0), white); accent-color: color(srgb 1 0 0 / .5); box-shadow: 0 0 1px color(srgb 0 0 1); column-rule: 1px solid color(srgb 1 0 0); text-shadow: 0 0 1px color(srgb-linear 0.5 0 0.5); scrollbar-color: color(srgb-linear 1 0 0 / 50%) white; border-color: color(display-p3 1 0 0); }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::ColorFunctionLowering,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 10);
    assert_eq!(
        execution.output_css,
        r#".card { color: rgb(255 0 0); background-color: rgb(128 64 0); outline-color: rgb(0 0 255); fill: rgb(128 128 128); background: linear-gradient(rgb(255 0 0), white); accent-color: rgb(255 0 0 / .5); box-shadow: 0 0 1px rgb(0 0 255); column-rule: 1px solid rgb(255 0 0); text-shadow: 0 0 1px rgb(188 0 188); scrollbar-color: rgb(255 0 0 / .5) white; border-color: color(display-p3 1 0 0); }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["color-function-lowering", "print-css"]
    );
}

#[test]
fn execution_runtime_lowers_logical_properties_only_with_static_direction() {
    let source = r#".ltr { direction: ltr; margin-inline-start: 1px; padding-inline-end: 2px; inline-size: 10rem; margin-inline: 1px 2px; padding-inline: calc(1rem + 1px) 3px; border-inline-color: red blue; margin-block: 4px 5px; padding-block-start: 6px; border-block-color: green yellow; border-block: 1px solid blue; inset-block-end: 7px; border-start-start-radius: 1px; border-start-end-radius: 2px; border-end-start-radius: 3px; border-end-end-radius: 4px; } .unknown { margin-inline-start: 1px; } .rtl { direction: rtl; writing-mode: horizontal-tb; inset-inline-start: 3px; border-inline-end-color: red; inset-inline: 4px 5px; border-inline: 1px solid red; border-inline-start: 2px dashed blue; border-start-start-radius: 5px; border-start-end-radius: 6px; border-end-start-radius: 7px; border-end-end-radius: 8px; }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::LogicalToPhysical,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 24);
    assert_eq!(
        execution.output_css,
        r#".ltr { direction: ltr; margin-left: 1px; padding-right: 2px; width: 10rem; margin-left: 1px; margin-right: 2px; padding-left: calc(1rem + 1px); padding-right: 3px; border-left-color: red; border-right-color: blue; margin-top: 4px; margin-bottom: 5px; padding-top: 6px; border-top-color: green; border-bottom-color: yellow; border-top: 1px solid blue; border-bottom: 1px solid blue; bottom: 7px; border-top-left-radius: 1px; border-top-right-radius: 2px; border-bottom-left-radius: 3px; border-bottom-right-radius: 4px; } .unknown { margin-inline-start: 1px; } .rtl { direction: rtl; writing-mode: horizontal-tb; right: 3px; border-left-color: red; right: 4px; left: 5px; border-right: 1px solid red; border-left: 1px solid red; border-right: 2px dashed blue; border-top-right-radius: 5px; border-top-left-radius: 6px; border-bottom-right-radius: 7px; border-bottom-left-radius: 8px; }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["logical-to-physical", "print-css"]
    );
}

#[test]
fn execution_runtime_lowers_vertical_logical_properties_with_static_axes() {
    let source = r#".vrl { writing-mode: vertical-rl; direction: ltr; margin-block-start: 1px; margin-block-end: 2px; margin-inline-start: 3px; margin-inline-end: 4px; block-size: 10px; inline-size: 20px; border-start-start-radius: 1px; border-end-end-radius: 2px; inset-block: 5px 6px; padding-inline: 7px 8px; } .vlr-rtl { writing-mode: vertical-lr; direction: rtl; inset-inline-start: 9px; border-start-end-radius: 3px; } .sideways { writing-mode: sideways-rl; direction: ltr; margin-inline-start: 1px; }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::LogicalToPhysical,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 12);
    assert_eq!(
        execution.output_css,
        r#".vrl { writing-mode: vertical-rl; direction: ltr; margin-right: 1px; margin-left: 2px; margin-top: 3px; margin-bottom: 4px; width: 10px; height: 20px; border-top-right-radius: 1px; border-bottom-left-radius: 2px; right: 5px; left: 6px; padding-top: 7px; padding-bottom: 8px; } .vlr-rtl { writing-mode: vertical-lr; direction: rtl; bottom: 9px; border-top-left-radius: 3px; } .sideways { writing-mode: sideways-rl; direction: ltr; margin-inline-start: 1px; }"#
    );
}

#[test]
fn execution_runtime_unwraps_simple_single_depth_nesting() {
    let source = r#".card { color: red; & .title { color: blue; } &:hover { color: green; } } .comma, .skip { & .x { color: red; } }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::NestingUnwrap,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 2);
    assert_eq!(
        execution.output_css,
        r#".card { color: red; } .card .title { color: blue; } .card:hover { color: green; } .comma .x, .skip .x { color: red; }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["nesting-unwrap", "print-css"]
    );
}

#[test]
fn execution_runtime_unwraps_selector_list_nesting_without_splitting_function_commas() {
    let source = r#".card:is(.active, .selected), .panel { &:hover, &--open { color: red; } }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::NestingUnwrap,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 1);
    assert_eq!(
        execution.output_css,
        r#".card:is(.active, .selected):hover, .card:is(.active, .selected)--open, .panel:hover, .panel--open { color: red; }"#
    );
}

#[test]
fn execution_runtime_unwraps_nested_rule_descendants() {
    let source = r#".card { color: red; & .title { font-weight: bold; &:hover { color: blue; } .icon, &__icon { color: green; } } }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::NestingUnwrap,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 1);
    assert_eq!(
        execution.output_css,
        r#".card { color: red; } .card .title { font-weight: bold; } .card .title:hover { color: blue; } .card .title .icon, .card .title__icon { color: green; }"#
    );
}

#[test]
fn execution_runtime_unwraps_explicit_nest_at_rules() {
    let source = r#".card { color: red; @nest .theme & { color: blue; & .title { color: green; } } @nest &:is(:hover, :focus) { color: purple; } }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::NestingUnwrap,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 1);
    assert_eq!(
        execution.output_css,
        r#".card { color: red; } .theme .card { color: blue; } .theme .card .title { color: green; } .card:is(:hover, :focus) { color: purple; }"#
    );
}

#[test]
fn execution_runtime_bubbles_nested_conditional_group_rules() {
    let source = r#".card { color: red; @media (min-width: 40rem) { color: blue; &:hover { color: green; } } @supports (display: grid) { & .title { display: grid; } } }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::NestingUnwrap,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 1);
    assert_eq!(
        execution.output_css,
        r#".card { color: red; } @media (min-width: 40rem) { .card { color: blue; } .card:hover { color: green; } } @supports (display: grid) { .card .title { display: grid; } }"#
    );
}

#[test]
fn execution_runtime_flattens_only_root_scope_proof_candidates() {
    let source =
        r#"@scope (:root) { .card { color: red; } } @scope (.theme) { .title { color: blue; } }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[TransformPassKind::ScopeFlatten, TransformPassKind::PrintCss],
    );

    assert_eq!(execution.mutation_count, 0);
    assert_eq!(execution.output_css, source);

    let accepted = execute_transform_passes_on_source(
        r#"@scope (:root) { .card { color: red; } }"#,
        &[TransformPassKind::ScopeFlatten, TransformPassKind::PrintCss],
    );
    assert_eq!(accepted.mutation_count, 1);
    assert_eq!(accepted.output_css, r#".card { color: red; }"#);
    assert_eq!(
        accepted.executed_pass_ids,
        vec!["scope-flatten", "print-css"]
    );
}

#[test]
fn execution_runtime_flattens_layers_only_with_closed_bundle_context() {
    let source = r#"@layer theme { .card { color: red; } }"#;
    let planned = execute_transform_passes_on_source(
        source,
        &[TransformPassKind::LayerFlatten, TransformPassKind::PrintCss],
    );
    assert_eq!(planned.output_css, source);
    assert_eq!(planned.planned_only_pass_ids, vec!["layer-flatten"]);

    let context = TransformExecutionContextV0 {
        closed_style_world: true,
        ..TransformExecutionContextV0::default()
    };
    let execution = execute_transform_passes_on_source_with_dialect_and_context(
        source,
        StyleDialect::Css,
        &[TransformPassKind::LayerFlatten, TransformPassKind::PrintCss],
        &context,
    );
    assert_eq!(execution.mutation_count, 1);
    assert_eq!(execution.output_css, r#".card { color: red; }"#);
    assert_eq!(
        execution.executed_pass_ids,
        vec!["layer-flatten", "print-css"]
    );
}

#[test]
fn execution_runtime_evaluates_literal_media_branches() {
    let source = r#"@media all { .a { color: red; } } @media not all { .b { color: blue; } } @media (max-width: 0px) { .zero { color: red; } } @media not (max-width: 0px) { .not-zero { color: lime; } } @media not all and (max-width: 0px) { .not-impossible { color: teal; } } @media all and (max-width: 0px) { .dead-and { color: red; } } @media (min-width: 10px) and (max-width: 5px) { .impossible { color: red; } } @media (min-width: calc(4px + 4px)) and (max-width: 5px) { .impossible-calc { color: red; } } @media not all, (max-width: 0px) { .dead-list { color: blue; } } @media all, screen { .list-true { color: purple; } } @media screen, (max-width: 0px) { .unknown-list { color: orange; } } @media screen { .c { color: green; } } @supports (display: grid) { @media all { @media all { .d { color: black; } } } }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::MediaStaticEval,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 13);
    assert_eq!(
        execution.output_css,
        r#".a { color: red; }   .not-zero { color: lime; } .not-impossible { color: teal; }     .list-true { color: purple; } @media screen, (width<=0px) { .unknown-list { color: orange; } } @media screen { .c { color: green; } } @supports (display: grid) { .d { color: black; } }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["media-static-eval", "print-css"]
    );
}

#[test]
fn execution_runtime_evaluates_media_or_disjunctions() {
    let source = r#"@media (max-width: 0px) or all { .live { color: red; } } @media (max-width: 0px) or (height<=0px) { .dead { color: blue; } } @media screen or (max-width: 0px) { .unknown { color: green; } }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::MediaStaticEval,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 3);
    assert_eq!(
        execution.output_css,
        r#".live { color: red; }  @media screen or (width<=0px) { .unknown { color: green; } }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["media-static-eval", "print-css"]
    );
}

#[test]
fn execution_runtime_evaluates_strict_media_range_comparisons() {
    let source = r#"@media (width > 10px) and (width < 5px) { .dead { color: red; } } @media (width > 10px) and (width < 10px) { .dead-strict { color: blue; } } @media (10px <= width) and (width <= 10px) { .maybe-point { color: green; } } @media (height < 0px) { .negative { color: orange; } } @media (0px < width) { .live { color: purple; } }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::MediaStaticEval,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 6);
    assert_eq!(
        execution.output_css,
        r#"  @media (width>=10px) and (width<=10px) { .maybe-point { color: green; } }  @media (width>0px) { .live { color: purple; } }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["media-static-eval", "print-css"]
    );
}

#[test]
fn execution_runtime_evaluates_chained_media_range_comparisons() {
    let source = r#"@media (400px < width < 800px) { .fluid { color: red; } } @media (800px < width < 400px) { .dead { color: blue; } } @media (10px <= width <= 10px) { .point { color: green; } } @media (10px < width <= 10px) { .dead-strict { color: orange; } } @media (100px > height > 20px) { .reverse { color: purple; } }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::MediaStaticEval,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 7);
    assert_eq!(
        execution.output_css,
        r#"@media (width>400px) and (width<800px) { .fluid { color: red; } }  @media (width>=10px) and (width<=10px) { .point { color: green; } }  @media (height<100px) and (height>20px) { .reverse { color: purple; } }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["media-static-eval", "print-css"]
    );
}

#[test]
fn execution_runtime_evaluates_media_range_equality_comparisons() {
    let source = r#"@media (width = 10px) { .point { color: red; } } @media (10px = height) { .reverse-point { color: blue; } } @media (width = 10px) and (width > 10px) { .dead-high { color: green; } } @media (height = 20px) and (height < 20px) { .dead-low { color: orange; } }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::MediaStaticEval,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 4);
    assert_eq!(
        execution.output_css,
        r#"@media (width=10px) { .point { color: red; } } @media (height=10px) { .reverse-point { color: blue; } }  "#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["media-static-eval", "print-css"]
    );
}

#[test]
fn execution_runtime_normalizes_simple_media_range_features() {
    let source = r#"@media screen and (min-width: 1px) and (max-width: 10px) { .a { color: red; } } @media (min-height: 2rem) { .b { color: blue; } } @media (min-width: calc(1px + 1px)) { .c { color: green; } } @media (max-height: clamp(1rem, 2rem, 3rem)) { .d { color: orange; } }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::MediaStaticEval,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 4);
    assert_eq!(
        execution.output_css,
        r#"@media screen and (width>=1px) and (width<=10px) { .a { color: red; } } @media (height>=2rem) { .b { color: blue; } } @media (width>=2px) { .c { color: green; } } @media (height<=2rem) { .d { color: orange; } }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["media-static-eval", "print-css"]
    );
}

#[test]
fn execution_runtime_evaluates_simple_supports_branches_with_cascade_witness() {
    let source = r#"@supports (display: grid) { .a { display: grid; } } @supports not (display: grid) { .b { display: block; } } @supports (display: grid) and (color: red) { .c { color: red; } } @supports (display: grid) or (selector(:has(*))) { .or { display: grid; } } @supports ((display: grid) or (display: -ms-grid)) and (color: red) { .grouped { display: grid; } } @supports not ((display: -ms-grid) or (-ms-ime-align: auto)) { .not-grouped { display: grid; } } @supports not ((display: grid) or (display: -ms-grid)) { .not-dead { display: grid; } } @media all { @supports (display: grid) { @supports (display: grid) { .d { display: grid; } } } }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::SupportsStaticEval,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 9);
    assert_eq!(
        execution.output_css,
        r#".a { display: grid; }  .c { color: red; } .or { display: grid; } .grouped { display: grid; } .not-grouped { display: grid; }  @media all { .d { display: grid; } }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["supports-static-eval", "print-css"]
    );
}

#[test]
fn execution_runtime_evaluates_selector_supports_branches_with_cascade_witness() {
    let source = r#"@supports selector(:has(*)) { .has { color: red; } } @supports not selector(:has(*)) { .not-has { color: blue; } } @supports selector(:-ms-input-placeholder) { .ms { color: green; } } @supports not selector(:-ms-input-placeholder) { .not-ms { color: purple; } }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::SupportsStaticEval,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 4);
    assert_eq!(
        execution.output_css,
        r#".has { color: red; }   .not-ms { color: purple; }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["supports-static-eval", "print-css"]
    );
}

#[test]
fn execution_runtime_evaluates_function_value_supports_branches_with_cascade_witness() {
    let source = r#"@supports (color: color(display-p3 1 0 0)) { .p3 { color: red; } } @supports not (background-image: linear-gradient(red, blue)) { .not-gradient { color: blue; } } @supports (width: min(10px, 20px)) and (display: grid) { .math { color: green; } } @supports (color: color(display-p3 1 0 0) { .malformed { color: orange; } }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::SupportsStaticEval,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 3);
    assert_eq!(
        execution.output_css,
        r#".p3 { color: red; }  .math { color: green; } @supports (color: color(display-p3 1 0 0) { .malformed { color: orange; } }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["supports-static-eval", "print-css"]
    );
}

#[test]
fn execution_runtime_evaluates_font_supports_branches_with_cascade_witness() {
    let source = r#"@supports font-tech(color-COLRv1) { .color-font { color: red; } } @supports not font-format(woff2) { .not-woff2 { color: blue; } } @supports font-format(embedded-opentype) { .eot { color: green; } } @supports not font-tech(-ms-color) { .not-ms { color: purple; } }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::SupportsStaticEval,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 4);
    assert_eq!(
        execution.output_css,
        r#".color-font { color: red; }   .not-ms { color: purple; }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["supports-static-eval", "print-css"]
    );
}

#[test]
fn execution_runtime_reduces_simple_same_unit_calc_values() {
    let source = r#".card { width: calc(1px + 2px); height: calc(10rem - 2rem); margin: calc(1px + 2rem); padding: calc(2px + 3px + 4px); margin-block-start: calc(10px - 3px - 2px); color: calc(1 + 2); gap: calc(.5rem+.25rem); inset: calc(1px - -2px); letter-spacing: calc(2px * 1); border-width: calc(1 * 3px); z-index: calc(4 / 1); scale: calc(3 * 0); box-shadow: 0 0 calc(1px + 2px) red; transform: translate(calc(10px - 2px), calc(1rem + 1rem)); min-width: min(10px, 4px); max-width: max(1rem, 2rem); block-size: min(2em, 1rem); opacity: max(.2, .5); outline-width: calc((2px * 3)); flex-basis: calc(2px * 3 * 4); inline-size: min(10px, max(2px, 4px)); line-height: clamp(.1, .5, .9); stroke-width: abs(-2px); order: sign(-10px); top: round(nearest, 10px, 3px); right: round(up, 10px, 3px); bottom: round(down, 10px, 3px); left: round(to-zero, 10px, 3px); translate: round(10px, 6px); rotate: round(nearest, 5px, 2px); margin-left: mod(10px, 3px); margin-right: rem(10px, 4px); perspective: mod(-10px, 3px); border-spacing: hypot(3px, 4px); flex-grow: hypot(3, 4); margin-bottom: hypot(3px, 4rem); }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::CalcReduction,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 31);
    assert_eq!(
        execution.output_css,
        r#".card { width: 3px; height: 8rem; margin: calc(1px + 2rem); padding: 9px; margin-block-start: 5px; color: 3; gap: 0.75rem; inset: 3px; letter-spacing: 2px; border-width: 3px; z-index: 4; scale: 0; box-shadow: 0 0 3px red; transform: translate(8px, 2rem); min-width: 4px; max-width: 2rem; block-size: min(2em, 1rem); opacity: 0.5; outline-width: 6px; flex-basis: 24px; inline-size: 4px; line-height: 0.5; stroke-width: 2px; order: -1; top: 9px; right: 12px; bottom: 9px; left: 9px; translate: 12px; rotate: round(nearest, 5px, 2px); margin-left: 1px; margin-right: 2px; perspective: mod(-10px, 3px); border-spacing: 5px; flex-grow: 5; margin-bottom: hypot(3px, 4rem); }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["calc-reduction", "print-css"]
    );
}

#[test]
fn execution_runtime_resolves_unique_static_root_custom_properties() {
    let source = r#":root { --brand: red; --gap: 2rem; --shadow: 0 0 var(--gap); --alias: var(--brand); --dynamic: var(--alias); --fallback: var(--missing, blue); --dup: red; --dup: blue; --cycle-a: var(--cycle-b); --cycle-b: var(--cycle-a); } @property --registered { syntax: "<color>"; inherits: false; initial-value: var(--brand); } @keyframes pulse { to { color: var(--brand); } } .card { color: var(--brand); margin: var(--gap); border-color: var(--missing, blue); background: var(--dup); outline-color: var(--dynamic); text-decoration-color: var(--fallback); caret-color: var(--cycle-a, green); box-shadow: var(--shadow); filter: drop-shadow(var(--missing, blue) 0 0); } @media screen { .card { color: var(--dynamic); } }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::StaticVarSubstitution,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 11);
    assert_eq!(
        execution.output_css,
        r#":root { --brand: red; --gap: 2rem; --shadow: 0 0 var(--gap); --alias: var(--brand); --dynamic: var(--alias); --fallback: var(--missing, blue); --dup: red; --dup: blue; --cycle-a: var(--cycle-b); --cycle-b: var(--cycle-a); } @property --registered { syntax: "<color>"; inherits: false; initial-value: red; } @keyframes pulse { to { color: red; } } .card { color: red; margin: 2rem; border-color: blue; background: var(--dup); outline-color: red; text-decoration-color: blue; caret-color: green; box-shadow: 0 0 2rem; filter: drop-shadow(blue 0 0); } @media screen { .card { color: red; } }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["custom-property-static-resolve", "print-css"]
    );
}

#[test]
fn execution_runtime_recovers_static_custom_property_substitution_after_malformed_var() {
    let source = r#":root { --brand: red; --gap: 2rem; } .card { border: 1px solid var(--brand) var(--broken; box-shadow: 0 0 var(--gap) var(--also-broken; }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::StaticVarSubstitution,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 2);
    assert_eq!(
        execution.output_css,
        r#":root { --brand: red; --gap: 2rem; } .card { border: 1px solid red var(--broken; box-shadow: 0 0 2rem var(--also-broken; }"#
    );
}

#[test]
fn execution_runtime_recovers_static_custom_property_env_after_malformed_var() {
    let source = r#":root { --gap: 2rem; --shadow: 0 0 var(--gap) var(--broken; } .card { box-shadow: var(--shadow); }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::StaticVarSubstitution,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 1);
    assert_eq!(
        execution.output_css,
        r#":root { --gap: 2rem; --shadow: 0 0 var(--gap) var(--broken; } .card { box-shadow: 0 0 2rem var(--broken; }"#
    );
}

#[test]
fn execution_runtime_keeps_shadowed_custom_properties_unresolved() {
    let source = r#":root { --brand: red; --gap: 2rem; --tone: red; --tone: blue !important; } .card { --brand: blue; color: var(--brand); margin: var(--gap); border-color: var(--tone); } .other { color: var(--brand); }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::StaticVarSubstitution,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 1);
    assert_eq!(
        execution.output_css,
        r#":root { --brand: red; --gap: 2rem; --tone: red; --tone: blue !important; } .card { --brand: blue; color: var(--brand); margin: 2rem; border-color: var(--tone); } .other { color: var(--brand); }"#
    );
}

#[test]
fn execution_runtime_resolves_unique_property_initial_values() {
    let source = r#"@property --brand { syntax: "<color>"; inherits: false; initial-value: red; } @property --shadowed { syntax: "<color>"; inherits: false; initial-value: green; } @property --dynamic { syntax: "<color>"; inherits: false; initial-value: teal; } @property --dup { syntax: "<color>"; inherits: false; initial-value: blue; } @property --dup { syntax: "<color>"; inherits: false; initial-value: purple; } :root { --dynamic: env(theme-color); } .card { --shadowed: orange; color: var(--brand); background: var(--shadowed); border-color: var(--dup); outline-color: var(--dynamic); }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::StaticVarSubstitution,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 1);
    assert_eq!(
        execution.output_css,
        r#"@property --brand { syntax: "<color>"; inherits: false; initial-value: red; } @property --shadowed { syntax: "<color>"; inherits: false; initial-value: green; } @property --dynamic { syntax: "<color>"; inherits: false; initial-value: teal; } @property --dup { syntax: "<color>"; inherits: false; initial-value: blue; } @property --dup { syntax: "<color>"; inherits: false; initial-value: purple; } :root { --dynamic: env(theme-color); } .card { --shadowed: orange; color: red; background: var(--shadowed); border-color: var(--dup); outline-color: var(--dynamic); }"#
    );
}

#[test]
fn execution_runtime_resolves_static_local_css_modules_values() {
    let source = r#"@value primary: #fff; @value spacing: 8px; @value alias: primary; @value shadow: 0 0 4px primary; @value bp: 40rem; @value wide: 80rem; @value width: 1px; @value modulePath: "./tokens.module.css"; @value dup: red; @value dup: blue; .btn { color: primary; margin: spacing spacing; background: alias; box-shadow: shadow; border-color: dup; } @media screen and (min-width: bp) and (width >= wide) and (bp <= width <= wide) { .btn { color: primary; } } @container card (inline-size >= wide) { .btn { margin: spacing; } }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::ValueResolution,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 18);
    assert_eq!(
        execution.output_css,
        r#"       @value modulePath: "./tokens.module.css"; @value dup: red; @value dup: blue; .btn { color: #fff; margin: 8px 8px; background: #fff; box-shadow: 0 0 4px #fff; border-color: dup; } @media screen and (min-width: 40rem) and (width >= 80rem) and (40rem <= width <= 80rem) { .btn { color: #fff; } } @container card (inline-size >= 80rem) { .btn { margin: 8px; } }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["value-resolution", "print-css"]
    );
}

#[test]
fn execution_runtime_resolves_imported_static_css_modules_values_from_context() {
    let source = r#"@value primary as brand, gap, tone from "./tokens.module.css"; @custom-media --gap (min-width: gap); .btn { color: brand; margin: gap; border-color: tone; } @media (min-width: gap) { .btn { color: brand; } } @supports (width: gap) { .btn { color: brand; } }"#;
    let context = TransformExecutionContextV0 {
        css_module_value_resolutions: vec![
            TransformCssModuleValueResolutionV0 {
                local_name: "brand".to_string(),
                resolved_value: "#fff".to_string(),
            },
            TransformCssModuleValueResolutionV0 {
                local_name: "gap".to_string(),
                resolved_value: "8px".to_string(),
            },
        ],
        ..TransformExecutionContextV0::default()
    };
    let execution = execute_transform_passes_on_source_with_dialect_and_context(
        source,
        StyleDialect::Css,
        &[
            TransformPassKind::ValueResolution,
            TransformPassKind::PrintCss,
        ],
        &context,
    );

    assert_eq!(execution.mutation_count, 8);
    assert_eq!(
        execution.output_css,
        r#"@value tone from "./tokens.module.css"; @custom-media --gap (min-width: 8px); .btn { color: #fff; margin: 8px; border-color: tone; } @media (min-width: 8px) { .btn { color: #fff; } } @supports (width: 8px) { .btn { color: #fff; } }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["value-resolution", "print-css"]
    );
}

#[test]
fn execution_runtime_removes_dead_branches_through_semantic_pass_surfaces() {
    let source = r#"@media not all { .dead { color: red; } } @supports (display: grid) { .grid { display: grid; } } @supports (display: -ms-grid) { .ms { display: -ms-grid; } } @supports (display: grid) and (color: red) { .conjunction { color: red; } }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::DeadMediaBranchRemoval,
            TransformPassKind::DeadSupportsBranchRemoval,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.mutation_count, 4);
    assert_eq!(
        execution.output_css,
        r#" .grid { display: grid; }  .conjunction { color: red; }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec![
            "dead-media-branch-removal",
            "dead-supports-branch-removal",
            "print-css"
        ]
    );
}

#[test]
fn execution_runtime_removes_dark_media_branches_with_workspace_context() {
    let source = r#"@media (prefers-color-scheme: dark) { .dark { color: white; } } @media (prefers-color-scheme: light) { .light { color: black; } } @media screen and (prefers-color-scheme: dark) { .screen-dark { color: white; } }"#;
    let context = TransformExecutionContextV0 {
        drop_dark_mode_media_queries: true,
        ..TransformExecutionContextV0::default()
    };
    let execution = execute_transform_passes_on_source_with_dialect_and_context(
        source,
        StyleDialect::Css,
        &[
            TransformPassKind::DeadMediaBranchRemoval,
            TransformPassKind::PrintCss,
        ],
        &context,
    );

    assert_eq!(execution.mutation_count, 2);
    assert_eq!(
        execution.output_css,
        r#" @media (prefers-color-scheme: light) { .light { color: black; } } "#
    );
    assert!(!execution.output_css.contains("prefers-color-scheme: dark"));
}

#[test]
fn execution_runtime_keeps_keyframe_tree_shaking_planned_without_closed_world_context() {
    let source = r#"@keyframes unused { to { opacity: 1; } } .btn { color: red; }"#;
    let execution = execute_transform_passes_on_source(
        source,
        &[
            TransformPassKind::TreeShakeKeyframes,
            TransformPassKind::PrintCss,
        ],
    );

    assert_eq!(execution.output_css, source);
    assert_eq!(execution.mutation_count, 0);
    assert_eq!(execution.executed_pass_ids, vec!["print-css"]);
    assert_eq!(
        execution.planned_only_pass_ids,
        vec!["tree-shake-keyframes"]
    );
}

#[test]
fn execution_runtime_tree_shakes_keyframes_with_closed_world_context() {
    let source = r#"@-webkit-keyframes fade { to { opacity: 1; } } @keyframes fade { to { opacity: 1; } } @-webkit-keyframes spin { to { transform: rotate(1turn); } } @keyframes spin { to { transform: rotate(1turn); } } @-webkit-keyframes dead { to { opacity: 0; } } @keyframes dead { to { opacity: 0; } } @keyframes ghost { to { opacity: .5; } } .btn { animation: 1s ease fade; } .dead-ref { animation: ghost 1s ease; }"#;
    let context = TransformExecutionContextV0 {
        closed_style_world: true,
        reachable_class_names: vec!["btn".to_string()],
        reachable_keyframe_names: vec!["spin".to_string()],
        ..TransformExecutionContextV0::default()
    };
    let execution = execute_transform_passes_on_source_with_dialect_and_context(
        source,
        StyleDialect::Css,
        &[
            TransformPassKind::TreeShakeKeyframes,
            TransformPassKind::PrintCss,
        ],
        &context,
    );

    assert_eq!(execution.mutation_count, 3);
    assert_eq!(
        execution.output_css,
        r#"@-webkit-keyframes fade { to { opacity: 1; } } @keyframes fade { to { opacity: 1; } } @-webkit-keyframes spin { to { transform: rotate(1turn); } } @keyframes spin { to { transform: rotate(1turn); } }    .btn { animation: 1s ease fade; } .dead-ref { animation: ghost 1s ease; }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["tree-shake-keyframes", "print-css"]
    );
    assert_eq!(execution.semantic_removals.len(), 3);
    assert!(
        execution
            .semantic_removals
            .iter()
            .all(|removal| removal.symbol_kind == "keyframes")
    );
    assert!(
        execution
            .semantic_removals
            .iter()
            .any(|removal| removal.name == "dead" && removal.pass_id == "tree-shake-keyframes")
    );
    assert!(
        execution.semantic_removals.iter().any(|removal| {
            removal.name == "ghost" && removal.pass_id == "tree-shake-keyframes"
        })
    );
}

#[test]
fn execution_runtime_tree_shakes_quoted_keyframes_with_closed_world_context() {
    let source = r#"@keyframes "slide" { to { opacity: 1; } } @keyframes "fade in" { to { opacity: 1; } } @keyframes "ghost" { to { opacity: 0; } } .btn { animation-name: "slide"; } .alt { animation: "slide" 1s ease; } .space { animation: "fade in" 1s ease; }"#;
    let context = TransformExecutionContextV0 {
        closed_style_world: true,
        ..TransformExecutionContextV0::default()
    };
    let execution = execute_transform_passes_on_source_with_dialect_and_context(
        source,
        StyleDialect::Css,
        &[
            TransformPassKind::TreeShakeKeyframes,
            TransformPassKind::PrintCss,
        ],
        &context,
    );

    assert_eq!(execution.mutation_count, 1);
    assert_eq!(
        execution.output_css,
        r#"@keyframes "slide" { to { opacity: 1; } } @keyframes "fade in" { to { opacity: 1; } }  .btn { animation-name: "slide"; } .alt { animation: "slide" 1s ease; } .space { animation: "fade in" 1s ease; }"#
    );
    assert_eq!(
        execution.executed_pass_ids,
        vec!["tree-shake-keyframes", "print-css"]
    );
}

#[test]
fn execution_runtime_tree_shakes_escaped_keyframes_with_closed_world_context() {
    let source = r#"@keyframes spin\:fast { to { opacity: 1; } } @keyframes hex\3A fast { to { opacity: .5; } } @keyframes dead { to { opacity: 0; } } .btn { animation: spin\:fast 1s ease; } .dead-ref { animation: dead 1s ease; }"#;
    let context = TransformExecutionContextV0 {
        closed_style_world: true,
        reachable_class_names: vec!["btn".to_string()],
        reachable_keyframe_names: vec!["hex:fast".to_string()],
        ..TransformExecutionContextV0::default()
    };
    let execution = execute_transform_passes_on_source_with_dialect_and_context(
        source,
        StyleDialect::Css,
        &[
            TransformPassKind::TreeShakeKeyframes,
            TransformPassKind::PrintCss,
        ],
        &context,
    );

    assert!(execution.output_css.contains("@keyframes spin\\:fast"));
    assert!(execution.output_css.contains("@keyframes hex\\3A fast"));
    assert!(!execution.output_css.contains("@keyframes dead"));
    assert!(
        execution
            .output_css
            .contains(".btn { animation: spin\\:fast 1s ease; }")
    );
    assert_eq!(execution.mutation_count, 1);
    assert_eq!(
        execution.executed_pass_ids,
        vec!["tree-shake-keyframes", "print-css"]
    );
    assert!(
        execution
            .semantic_removals
            .iter()
            .any(|removal| { removal.name == "dead" && removal.pass_id == "tree-shake-keyframes" })
    );
}

#[test]
fn execution_runtime_tree_shakes_class_owned_rules_with_closed_world_context() {
    let source = r#".used { color: red; } .dead { color: blue; } .dead:hover { color: green; } button.other-dead { color: black; } .also-dead, .other-dead { color: black; } .used, .dead-mixed { color: cyan; } .used .child { color: purple; } :global(.external) { color: gray; } :global { .global-block { color: silver; } } .dead :global(.external) { color: pink; } :global(.root) .dead-global { color: lime; } :local(.dead-local) { color: brown; } @media (min-width: 1px) { .media-dead { color: orange; } .used { color: brown; } }"#;
    let context = TransformExecutionContextV0 {
        closed_style_world: true,
        reachable_class_names: vec!["used".to_string()],
        ..TransformExecutionContextV0::default()
    };
    let execution = execute_transform_passes_on_source_with_dialect_and_context(
        source,
        StyleDialect::Css,
        &[
            TransformPassKind::TreeShakeClass,
            TransformPassKind::PrintCss,
        ],
        &context,
    );

    assert_eq!(execution.mutation_count, 9);
    assert!(execution.output_css.contains(".used { color: red; }"));
    assert!(execution.output_css.contains(".used { color: cyan; }"));
    assert!(
        execution
            .output_css
            .contains(".used .child { color: purple; }")
    );
    assert!(
        execution
            .output_css
            .contains("@media (min-width: 1px) {  .used { color: brown; } }")
    );
    assert!(
        execution
            .output_css
            .contains(":global(.external) { color: gray; }")
    );
    assert!(
        execution
            .output_css
            .contains(":global { .global-block { color: silver; } }")
    );
    assert!(!execution.output_css.contains(".dead {"));
    assert!(!execution.output_css.contains(".dead:hover"));
    assert!(!execution.output_css.contains(".dead :global"));
    assert!(!execution.output_css.contains(".dead-global"));
    assert!(!execution.output_css.contains(".dead-local"));
    assert!(!execution.output_css.contains("button.other-dead"));
    assert!(!execution.output_css.contains(".also-dead"));
    assert!(!execution.output_css.contains(".other-dead"));
    assert!(!execution.output_css.contains(".dead-mixed"));
    assert!(!execution.output_css.contains(".media-dead"));
    assert_eq!(
        execution.executed_pass_ids,
        vec!["tree-shake-class", "print-css"]
    );
    assert_eq!(execution.semantic_removals.len(), 9);
    assert!(execution.semantic_removals.iter().any(|removal| {
        removal.symbol_kind == "class"
            && removal.name == "also-dead,other-dead"
            && removal.pass_id == "tree-shake-class"
            && removal
                .derivation_steps
                .contains(&"symbolNotMarkedReachable")
    }));
}

#[test]
fn execution_runtime_tree_shakes_escaped_class_owned_rules_with_closed_world_context() {
    let source = r#".foo\:bar { color: red; } .dead { color: blue; } .foo\:bar:hover { color: green; } .dead, .foo\:bar { color: cyan; } .hex\3A bar { color: purple; } .hex-dead { color: black; }"#;
    let context = TransformExecutionContextV0 {
        closed_style_world: true,
        reachable_class_names: vec!["foo:bar".to_string(), "hex:bar".to_string()],
        ..TransformExecutionContextV0::default()
    };
    let execution = execute_transform_passes_on_source_with_dialect_and_context(
        source,
        StyleDialect::Css,
        &[
            TransformPassKind::TreeShakeClass,
            TransformPassKind::PrintCss,
        ],
        &context,
    );

    assert!(
        execution
            .output_css
            .contains(r#".foo\:bar { color: red; }"#)
    );
    assert!(
        execution
            .output_css
            .contains(r#".foo\:bar:hover { color: green; }"#)
    );
    assert!(
        execution
            .output_css
            .contains(r#".foo\:bar { color: cyan; }"#)
    );
    assert!(
        execution
            .output_css
            .contains(r#".hex\3A bar { color: purple; }"#)
    );
    assert!(!execution.output_css.contains(".dead {"));
    assert!(!execution.output_css.contains(".dead,"));
    assert!(!execution.output_css.contains(".hex-dead"));
    assert_eq!(
        execution.executed_pass_ids,
        vec!["tree-shake-class", "print-css"]
    );
    assert_eq!(execution.semantic_removals.len(), 3);
    assert!(
        execution
            .semantic_removals
            .iter()
            .any(|removal| { removal.pass_id == "tree-shake-class" && removal.name == "dead" })
    );
}

#[test]
fn execution_runtime_keeps_composed_classes_reachable_during_tree_shaking() {
    let source = r#".button { composes: base; color: red; } .base { color: blue; } .utility { animation: spin 1s; color: var(--brand); } .dead { color: black; } @keyframes spin { to { opacity: 1; } } @keyframes ghost { to { opacity: 0; } } :root { --brand: red; --dead: blue; }"#;
    let context = TransformExecutionContextV0 {
        closed_style_world: true,
        reachable_class_names: vec!["button".to_string()],
        css_module_composes_resolutions: vec![TransformCssModuleComposesResolutionV0 {
            local_class_name: "button".to_string(),
            exported_class_names: vec![
                "button".to_string(),
                "base".to_string(),
                "utility".to_string(),
            ],
        }],
        ..TransformExecutionContextV0::default()
    };
    let execution = execute_transform_passes_on_source_with_dialect_and_context(
        source,
        StyleDialect::Css,
        &[
            TransformPassKind::TreeShakeClass,
            TransformPassKind::TreeShakeKeyframes,
            TransformPassKind::TreeShakeCustomProperty,
            TransformPassKind::PrintCss,
        ],
        &context,
    );

    assert!(execution.output_css.contains(".button"));
    assert!(execution.output_css.contains(".base"));
    assert!(execution.output_css.contains(".utility"));
    assert!(execution.output_css.contains("@keyframes spin"));
    assert!(execution.output_css.contains("--brand: red"));
    assert!(!execution.output_css.contains(".dead"));
    assert!(!execution.output_css.contains("@keyframes ghost"));
    assert!(!execution.output_css.contains("--dead: blue"));
    assert!(
        execution
            .semantic_removals
            .iter()
            .any(|removal| removal.pass_id == "tree-shake-class" && removal.name == "dead")
    );
}

#[test]
fn execution_runtime_tree_shakes_local_values_with_closed_world_context() {
    let source = r#"@value used: red; @value dead: blue; @value alias: used; @value shadow: 0 0 4px used; @value bp: 40rem; @value deadAlias: dead; @value deadShadow: 0 0 4px dead; @value deadBp: 50rem; @value deadFromRule: orange; @value deadExpr: calc(1rem + 2px); .btn { color: used; background: alias; box-shadow: shadow; } .dead { color: deadFromRule; } @media (min-width: bp) { .btn { color: red; } } @media (min-width: deadBp) { .dead { color: dead; } }"#;
    let context = TransformExecutionContextV0 {
        closed_style_world: true,
        reachable_class_names: vec!["btn".to_string()],
        ..TransformExecutionContextV0::default()
    };
    let execution = execute_transform_passes_on_source_with_dialect_and_context(
        source,
        StyleDialect::Css,
        &[
            TransformPassKind::TreeShakeValue,
            TransformPassKind::PrintCss,
        ],
        &context,
    );

    assert_eq!(execution.mutation_count, 6);
    assert!(execution.output_css.contains("@value used: red;"));
    assert!(execution.output_css.contains("@value alias: used;"));
    assert!(
        execution
            .output_css
            .contains("@value shadow: 0 0 4px used;")
    );
    assert!(execution.output_css.contains("@value bp: 40rem;"));
    assert!(execution.output_css.contains("box-shadow: shadow;"));
    assert!(execution.output_css.contains("@media (min-width: bp)"));
    assert!(!execution.output_css.contains("@value dead:"));
    assert!(!execution.output_css.contains("@value deadAlias:"));
    assert!(!execution.output_css.contains("@value deadShadow:"));
    assert!(!execution.output_css.contains("@value deadBp:"));
    assert!(!execution.output_css.contains("@value deadFromRule:"));
    assert!(!execution.output_css.contains("@value deadExpr:"));
    assert_eq!(
        execution.executed_pass_ids,
        vec!["tree-shake-value", "print-css"]
    );
    assert_eq!(
        execution
            .semantic_removals
            .iter()
            .map(|removal| removal.name.as_str())
            .collect::<Vec<_>>(),
        vec![
            "dead",
            "deadAlias",
            "deadShadow",
            "deadBp",
            "deadFromRule",
            "deadExpr"
        ]
    );
}

#[test]
fn execution_runtime_keeps_values_used_by_reachable_keyframes() {
    let source = r#"@value used: red; @value dead: blue; @value ghost: green; @keyframes pulse { to { color: used; } } @keyframes ghost { to { color: ghost; } } .btn { animation: pulse 1s; }"#;
    let context = TransformExecutionContextV0 {
        closed_style_world: true,
        reachable_class_names: vec!["btn".to_string()],
        ..TransformExecutionContextV0::default()
    };
    let execution = execute_transform_passes_on_source_with_dialect_and_context(
        source,
        StyleDialect::Css,
        &[
            TransformPassKind::TreeShakeKeyframes,
            TransformPassKind::TreeShakeValue,
            TransformPassKind::PrintCss,
        ],
        &context,
    );

    assert_eq!(execution.mutation_count, 3);
    assert!(execution.output_css.contains("@value used: red;"));
    assert!(execution.output_css.contains("color: used;"));
    assert!(!execution.output_css.contains("@value dead:"));
    assert!(!execution.output_css.contains("@value ghost:"));
    assert!(!execution.output_css.contains("@keyframes ghost"));
    assert_eq!(
        execution
            .semantic_removals
            .iter()
            .map(|removal| (removal.pass_id, removal.symbol_kind, removal.name.as_str()))
            .collect::<Vec<_>>(),
        vec![
            ("tree-shake-keyframes", "keyframes", "ghost"),
            ("tree-shake-value", "cssModuleValue", "dead"),
            ("tree-shake-value", "cssModuleValue", "ghost")
        ]
    );
}

#[test]
fn execution_runtime_keeps_values_used_by_dynamically_reachable_keyframes() {
    let source = r#"@value used: red; @value dead: blue; @keyframes pulse { to { color: used; } } @keyframes ghost { to { color: dead; } } .btn { animation: var(--motion-name); }"#;
    let context = TransformExecutionContextV0 {
        closed_style_world: true,
        reachable_class_names: vec!["btn".to_string()],
        ..TransformExecutionContextV0::default()
    };
    let execution = execute_transform_passes_on_source_with_dialect_and_context(
        source,
        StyleDialect::Css,
        &[
            TransformPassKind::TreeShakeKeyframes,
            TransformPassKind::TreeShakeValue,
            TransformPassKind::PrintCss,
        ],
        &context,
    );

    assert_eq!(execution.mutation_count, 0);
    assert!(execution.output_css.contains("@value used: red;"));
    assert!(execution.output_css.contains("@value dead: blue;"));
    assert!(execution.output_css.contains("@keyframes pulse"));
    assert!(execution.output_css.contains("@keyframes ghost"));
    assert!(execution.semantic_removals.is_empty());
}

#[test]
fn execution_runtime_keeps_values_used_by_explicit_reachable_keyframes() {
    let source = r#"@value ghost: green; @value dead: blue; @keyframes ghost { to { color: ghost; } } @keyframes dead { to { color: dead; } } .btn { color: red; }"#;
    let context = TransformExecutionContextV0 {
        closed_style_world: true,
        reachable_class_names: vec!["btn".to_string()],
        reachable_keyframe_names: vec!["ghost".to_string()],
        ..TransformExecutionContextV0::default()
    };
    let execution = execute_transform_passes_on_source_with_dialect_and_context(
        source,
        StyleDialect::Css,
        &[
            TransformPassKind::TreeShakeKeyframes,
            TransformPassKind::TreeShakeValue,
            TransformPassKind::PrintCss,
        ],
        &context,
    );

    assert_eq!(execution.mutation_count, 2);
    assert!(execution.output_css.contains("@value ghost: green;"));
    assert!(execution.output_css.contains("@keyframes ghost"));
    assert!(!execution.output_css.contains("@value dead:"));
    assert!(!execution.output_css.contains("@keyframes dead"));
    assert_eq!(
        execution
            .semantic_removals
            .iter()
            .map(|removal| (removal.pass_id, removal.symbol_kind, removal.name.as_str()))
            .collect::<Vec<_>>(),
        vec![
            ("tree-shake-keyframes", "keyframes", "dead"),
            ("tree-shake-value", "cssModuleValue", "dead")
        ]
    );
}

#[test]
fn execution_runtime_tree_shakes_at_rule_prelude_non_value_identifiers() {
    let source = r#"@value screen: 1px; @value width: 1px; @value bp: 40rem; @value wide: 80rem; @value theme: dark; @media screen and (min-width: bp) and (bp <= width <= wide) { .btn { color: red; } } @container card style(--mode: theme) { .btn { color: blue; } }"#;
    let context = TransformExecutionContextV0 {
        closed_style_world: true,
        reachable_class_names: vec!["btn".to_string()],
        ..TransformExecutionContextV0::default()
    };
    let execution = execute_transform_passes_on_source_with_dialect_and_context(
        source,
        StyleDialect::Css,
        &[
            TransformPassKind::TreeShakeValue,
            TransformPassKind::PrintCss,
        ],
        &context,
    );

    assert_eq!(execution.mutation_count, 2);
    assert!(!execution.output_css.contains("@value screen:"));
    assert!(!execution.output_css.contains("@value width:"));
    assert!(execution.output_css.contains("@value bp: 40rem;"));
    assert!(execution.output_css.contains("@value wide: 80rem;"));
    assert!(execution.output_css.contains("@value theme: dark;"));
    assert!(
        execution
            .output_css
            .contains("@media screen and (min-width: bp) and (bp <= width <= wide)")
    );
    assert!(
        execution
            .output_css
            .contains("@container card style(--mode: theme)")
    );
    assert_eq!(
        execution
            .semantic_removals
            .iter()
            .map(|removal| removal.name.as_str())
            .collect::<Vec<_>>(),
        vec!["screen", "width"]
    );
}

#[test]
fn execution_runtime_tree_shakes_imported_values_with_closed_world_context() {
    let source = r#"@value used, dead, ghost from "./tokens.module.css"; @value local: used; .btn { color: local; } .dead { color: dead; }"#;
    let context = TransformExecutionContextV0 {
        closed_style_world: true,
        reachable_class_names: vec!["btn".to_string()],
        ..TransformExecutionContextV0::default()
    };
    let execution = execute_transform_passes_on_source_with_dialect_and_context(
        source,
        StyleDialect::Css,
        &[
            TransformPassKind::TreeShakeValue,
            TransformPassKind::PrintCss,
        ],
        &context,
    );

    assert_eq!(execution.mutation_count, 2);
    assert!(
        execution
            .output_css
            .contains(r#"@value used from "./tokens.module.css";"#)
    );
    assert!(execution.output_css.contains("@value local: used;"));
    assert!(!execution.output_css.contains("dead, ghost from"));
    assert!(execution.output_css.contains(".btn { color: local; }"));
    assert!(execution.output_css.contains(".dead { color: dead; }"));
    assert_eq!(
        execution
            .semantic_removals
            .iter()
            .map(|removal| removal.name.as_str())
            .collect::<Vec<_>>(),
        vec!["dead", "ghost"]
    );
}

#[test]
fn execution_runtime_tree_shakes_icss_exports_with_closed_world_context() {
    let source = r#"@value primary: red; @value shadow: 0 0 primary; @value dead: blue; :export { public-color: shadow; dead-public: dead; } .btn { color: red; }"#;
    let context = TransformExecutionContextV0 {
        closed_style_world: true,
        reachable_class_names: vec!["btn".to_string()],
        reachable_value_names: vec!["public-color".to_string()],
        ..TransformExecutionContextV0::default()
    };
    let execution = execute_transform_passes_on_source_with_dialect_and_context(
        source,
        StyleDialect::Css,
        &[
            TransformPassKind::TreeShakeValue,
            TransformPassKind::PrintCss,
        ],
        &context,
    );

    assert_eq!(execution.mutation_count, 2);
    assert!(execution.output_css.contains("@value primary: red;"));
    assert!(execution.output_css.contains("@value shadow: 0 0 primary;"));
    assert!(
        execution
            .output_css
            .contains(":export { public-color: shadow;")
    );
    assert!(!execution.output_css.contains("@value dead:"));
    assert!(!execution.output_css.contains("dead-public: dead"));
    assert_eq!(
        execution
            .semantic_removals
            .iter()
            .map(|removal| (removal.symbol_kind, removal.name.as_str()))
            .collect::<Vec<_>>(),
        vec![
            ("cssModuleValue", "dead"),
            ("cssModuleIcssExport", "dead-public")
        ]
    );
}

#[test]
fn execution_runtime_tree_shakes_custom_properties_with_closed_world_context() {
    let source = r#":root { --used: VAR(--alias); --alias: red; --dead: VAR(--dead-dep); --dead-dep: blue; --string-only: orange; --dead-from-rule: black; color: VAR(--used); content: "var(--string-only)"; } .btn { color: var(--external); } .dead { color: var(--dead-from-rule); }"#;
    let context = TransformExecutionContextV0 {
        closed_style_world: true,
        reachable_class_names: vec!["btn".to_string()],
        reachable_custom_property_names: vec!["--external".to_string()],
        ..TransformExecutionContextV0::default()
    };
    let execution = execute_transform_passes_on_source_with_dialect_and_context(
        source,
        StyleDialect::Css,
        &[
            TransformPassKind::TreeShakeCustomProperty,
            TransformPassKind::PrintCss,
        ],
        &context,
    );

    assert_eq!(execution.mutation_count, 4);
    assert!(execution.output_css.contains("--used: VAR(--alias);"));
    assert!(execution.output_css.contains("--alias: red;"));
    assert!(execution.output_css.contains("color: VAR(--used);"));
    assert!(
        execution
            .output_css
            .contains(r#"content: "var(--string-only)";"#)
    );
    assert!(execution.output_css.contains("color: var(--external);"));
    assert!(!execution.output_css.contains("--dead:"));
    assert!(!execution.output_css.contains("--dead-dep:"));
    assert!(!execution.output_css.contains("--string-only:"));
    assert!(!execution.output_css.contains("--dead-from-rule:"));
    assert_eq!(
        execution.executed_pass_ids,
        vec!["tree-shake-custom-property", "print-css"]
    );
    assert_eq!(
        execution
            .semantic_removals
            .iter()
            .map(|removal| (removal.symbol_kind, removal.name.as_str()))
            .collect::<Vec<_>>(),
        vec![
            ("customProperty", "--dead"),
            ("customProperty", "--dead-dep"),
            ("customProperty", "--string-only"),
            ("customProperty", "--dead-from-rule")
        ]
    );
}

#[test]
fn execution_runtime_tree_shakes_custom_property_icss_exports_with_closed_world_context() {
    let source = r#":root { --brand: red; --dead: blue; } :export { brand: var(--brand); dead: var(--dead); }"#;
    let context = TransformExecutionContextV0 {
        closed_style_world: true,
        reachable_custom_property_names: vec!["brand".to_string()],
        ..TransformExecutionContextV0::default()
    };
    let execution = execute_transform_passes_on_source_with_dialect_and_context(
        source,
        StyleDialect::Css,
        &[
            TransformPassKind::TreeShakeCustomProperty,
            TransformPassKind::PrintCss,
        ],
        &context,
    );

    assert_eq!(execution.mutation_count, 2);
    assert!(execution.output_css.contains("--brand: red;"));
    assert!(execution.output_css.contains("brand: var(--brand);"));
    assert!(!execution.output_css.contains("--dead: blue;"));
    assert!(!execution.output_css.contains("dead: var(--dead);"));
    assert_eq!(
        execution
            .semantic_removals
            .iter()
            .map(|removal| (removal.symbol_kind, removal.name.as_str()))
            .collect::<Vec<_>>(),
        vec![
            ("customPropertyIcssExport", "dead"),
            ("customProperty", "--dead")
        ]
    );

    let all_unreachable = execute_transform_passes_on_source_with_dialect_and_context(
        r#":root { --dead: blue; } :export { dead: var(--dead); }"#,
        StyleDialect::Css,
        &[
            TransformPassKind::TreeShakeCustomProperty,
            TransformPassKind::PrintCss,
        ],
        &TransformExecutionContextV0 {
            closed_style_world: true,
            ..TransformExecutionContextV0::default()
        },
    );

    assert_eq!(all_unreachable.mutation_count, 2);
    assert!(!all_unreachable.output_css.contains(":export"));
    assert!(!all_unreachable.output_css.contains("--dead: blue;"));

    let css_name_root = execute_transform_passes_on_source_with_dialect_and_context(
        source,
        StyleDialect::Css,
        &[
            TransformPassKind::TreeShakeCustomProperty,
            TransformPassKind::PrintCss,
        ],
        &TransformExecutionContextV0 {
            closed_style_world: true,
            reachable_custom_property_names: vec!["--brand".to_string()],
            ..TransformExecutionContextV0::default()
        },
    );

    assert_eq!(css_name_root.mutation_count, 2);
    assert!(css_name_root.output_css.contains("--brand: red;"));
    assert!(css_name_root.output_css.contains("brand: var(--brand);"));
    assert!(!css_name_root.output_css.contains("--dead: blue;"));
    assert!(!css_name_root.output_css.contains("dead: var(--dead);"));
}

#[test]
fn execution_runtime_ignores_unreachable_custom_property_dependencies() {
    let source = r#":root { --used: var(--dep); --dep: red; --ghost: blue; } .btn { color: var(--used); } .dead { --used: var(--ghost); color: var(--ghost); }"#;
    let context = TransformExecutionContextV0 {
        closed_style_world: true,
        reachable_class_names: vec!["btn".to_string()],
        ..TransformExecutionContextV0::default()
    };
    let execution = execute_transform_passes_on_source_with_dialect_and_context(
        source,
        StyleDialect::Css,
        &[
            TransformPassKind::TreeShakeCustomProperty,
            TransformPassKind::PrintCss,
        ],
        &context,
    );

    assert_eq!(execution.mutation_count, 2);
    assert!(execution.output_css.contains("--used: var(--dep);"));
    assert!(execution.output_css.contains("--dep: red;"));
    assert!(!execution.output_css.contains("--ghost: blue;"));
    assert!(!execution.output_css.contains("--used: var(--ghost);"));
    assert!(execution.output_css.contains("color: var(--ghost);"));
    assert_eq!(
        execution
            .semantic_removals
            .iter()
            .map(|removal| (removal.symbol_kind, removal.name.as_str()))
            .collect::<Vec<_>>(),
        vec![("customProperty", "--ghost"), ("customProperty", "--used")]
    );
}

#[test]
fn execution_runtime_ignores_malformed_var_in_unreachable_custom_property_rules() {
    let source = r#":root { --used: red; --dead: blue; } .btn { color: var(--used); } .dead { color: var(--broken; --other: var(--also-broken; }"#;
    let context = TransformExecutionContextV0 {
        closed_style_world: true,
        reachable_class_names: vec!["btn".to_string()],
        ..TransformExecutionContextV0::default()
    };
    let execution = execute_transform_passes_on_source_with_dialect_and_context(
        source,
        StyleDialect::Css,
        &[
            TransformPassKind::TreeShakeCustomProperty,
            TransformPassKind::PrintCss,
        ],
        &context,
    );

    assert_eq!(execution.mutation_count, 2);
    assert!(execution.output_css.contains("--used: red;"));
    assert!(!execution.output_css.contains("--dead: blue;"));
    assert!(execution.output_css.contains("color: var(--broken;"));
    assert!(!execution.output_css.contains("--other: var(--also-broken;"));
    assert_eq!(
        execution
            .semantic_removals
            .iter()
            .map(|removal| (removal.symbol_kind, removal.name.as_str()))
            .collect::<Vec<_>>(),
        vec![("customProperty", "--dead"), ("customProperty", "--other")]
    );
}

#[test]
fn execution_runtime_recovers_custom_property_tree_shaking_after_reachable_malformed_var() {
    let source = r#":root { --used: red; --dead: blue; } .btn { color: var(--used); outline: var(--broken; }"#;
    let context = TransformExecutionContextV0 {
        closed_style_world: true,
        reachable_class_names: vec!["btn".to_string()],
        ..TransformExecutionContextV0::default()
    };
    let execution = execute_transform_passes_on_source_with_dialect_and_context(
        source,
        StyleDialect::Css,
        &[
            TransformPassKind::TreeShakeCustomProperty,
            TransformPassKind::PrintCss,
        ],
        &context,
    );

    assert_eq!(execution.mutation_count, 1);
    assert!(execution.output_css.contains("--used: red;"));
    assert!(execution.output_css.contains("color: var(--used);"));
    assert!(execution.output_css.contains("outline: var(--broken;"));
    assert!(!execution.output_css.contains("--dead: blue;"));
    assert_eq!(
        execution
            .semantic_removals
            .iter()
            .map(|removal| (removal.symbol_kind, removal.name.as_str()))
            .collect::<Vec<_>>(),
        vec![("customProperty", "--dead")]
    );
}

#[test]
fn execution_runtime_ignores_dead_keyframe_custom_property_dependencies() {
    let source = r#":root { --used: red; --ghost: blue; } .btn { animation: live 1s; } @keyframes live { to { color: var(--used); } } @keyframes ghost { to { --used: var(--ghost); color: var(--ghost); } }"#;
    let context = TransformExecutionContextV0 {
        closed_style_world: true,
        reachable_class_names: vec!["btn".to_string()],
        ..TransformExecutionContextV0::default()
    };
    let execution = execute_transform_passes_on_source_with_dialect_and_context(
        source,
        StyleDialect::Css,
        &[
            TransformPassKind::TreeShakeCustomProperty,
            TransformPassKind::PrintCss,
        ],
        &context,
    );

    assert_eq!(execution.mutation_count, 2);
    assert!(execution.output_css.contains("--used: red;"));
    assert!(execution.output_css.contains("color: var(--used);"));
    assert!(execution.output_css.contains("@keyframes ghost"));
    assert!(execution.output_css.contains("color: var(--ghost);"));
    assert!(!execution.output_css.contains("--ghost: blue;"));
    assert!(!execution.output_css.contains("--used: var(--ghost);"));
    assert_eq!(
        execution
            .semantic_removals
            .iter()
            .map(|removal| (removal.symbol_kind, removal.name.as_str()))
            .collect::<Vec<_>>(),
        vec![("customProperty", "--ghost"), ("customProperty", "--used")]
    );
}

#[test]
fn execution_runtime_tree_shakes_custom_property_registrations_with_closed_world_context() {
    let source = r#"@property --used { syntax: "<color>"; inherits: false; initial-value: red; } @property --dead { syntax: "<color>"; inherits: false; initial-value: blue; } :root { --used: red; --dead: blue; } .btn { color: var(--used); } .dead { color: var(--dead); }"#;
    let context = TransformExecutionContextV0 {
        closed_style_world: true,
        reachable_class_names: vec!["btn".to_string()],
        ..TransformExecutionContextV0::default()
    };
    let execution = execute_transform_passes_on_source_with_dialect_and_context(
        source,
        StyleDialect::Css,
        &[
            TransformPassKind::TreeShakeCustomProperty,
            TransformPassKind::PrintCss,
        ],
        &context,
    );

    assert_eq!(execution.mutation_count, 2);
    assert!(execution.output_css.contains("@property --used"));
    assert!(execution.output_css.contains("--used: red"));
    assert!(execution.output_css.contains("color: var(--used);"));
    assert!(!execution.output_css.contains("@property --dead"));
    assert!(!execution.output_css.contains("--dead: blue"));
    assert_eq!(
        execution
            .semantic_removals
            .iter()
            .map(|removal| (removal.symbol_kind, removal.name.as_str()))
            .collect::<Vec<_>>(),
        vec![
            ("customPropertyRegistration", "--dead"),
            ("customProperty", "--dead")
        ]
    );
}

#[test]
fn execution_runtime_keeps_registration_initial_value_custom_property_dependencies() {
    let source = r#"@property --used { syntax: "<color>"; inherits: false; initial-value: var(--registered-dep); } @property --dead { syntax: "<color>"; inherits: false; initial-value: var(--dead-dep); } :root { --registered-dep: red; --dead-dep: blue; --ghost: orange; } .btn { color: var(--used); }"#;
    let context = TransformExecutionContextV0 {
        closed_style_world: true,
        reachable_class_names: vec!["btn".to_string()],
        ..TransformExecutionContextV0::default()
    };
    let execution = execute_transform_passes_on_source_with_dialect_and_context(
        source,
        StyleDialect::Css,
        &[
            TransformPassKind::TreeShakeCustomProperty,
            TransformPassKind::PrintCss,
        ],
        &context,
    );

    assert_eq!(execution.mutation_count, 3);
    assert!(execution.output_css.contains("@property --used"));
    assert!(execution.output_css.contains("--registered-dep: red;"));
    assert!(execution.output_css.contains("color: var(--used);"));
    assert!(!execution.output_css.contains("@property --dead"));
    assert!(!execution.output_css.contains("--dead-dep: blue"));
    assert!(!execution.output_css.contains("--ghost: orange"));
    assert_eq!(
        execution
            .semantic_removals
            .iter()
            .map(|removal| (removal.symbol_kind, removal.name.as_str()))
            .collect::<Vec<_>>(),
        vec![
            ("customPropertyRegistration", "--dead"),
            ("customProperty", "--dead-dep"),
            ("customProperty", "--ghost")
        ]
    );
}

#[test]
fn execution_runtime_keeps_container_style_query_custom_property_roots() {
    let source = r#"@property --theme { syntax: "<custom-ident>"; inherits: true; initial-value: light; } @property --dead { syntax: "<custom-ident>"; inherits: true; initial-value: off; } :root { --theme: dark; --dead: off; } @container card style(--theme: dark) { .btn { color: white; } } @container card style(--dead: off) { .dead { color: black; } }"#;
    let context = TransformExecutionContextV0 {
        closed_style_world: true,
        reachable_class_names: vec!["btn".to_string()],
        ..TransformExecutionContextV0::default()
    };
    let execution = execute_transform_passes_on_source_with_dialect_and_context(
        source,
        StyleDialect::Css,
        &[
            TransformPassKind::TreeShakeCustomProperty,
            TransformPassKind::PrintCss,
        ],
        &context,
    );

    assert_eq!(execution.mutation_count, 2);
    assert!(execution.output_css.contains("@property --theme"));
    assert!(execution.output_css.contains("--theme: dark;"));
    assert!(
        execution
            .output_css
            .contains("@container card style(--theme: dark)")
    );
    assert!(!execution.output_css.contains("@property --dead"));
    assert!(
        !execution
            .output_css
            .contains(":root { --theme: dark; --dead: off;")
    );
    assert_eq!(
        execution
            .semantic_removals
            .iter()
            .map(|removal| (removal.symbol_kind, removal.name.as_str()))
            .collect::<Vec<_>>(),
        vec![
            ("customPropertyRegistration", "--dead"),
            ("customProperty", "--dead")
        ]
    );
}

#[test]
fn execution_runtime_uses_dialect_lexer_for_scss_silent_comments() {
    let source = ".a { // remove\n  color: red;\n  content: \"// keep\";\n}";
    let execution = execute_transform_passes_on_source_with_dialect(
        source,
        StyleDialect::Scss,
        &[TransformPassKind::CommentStrip],
    );

    assert_eq!(execution.mutation_count, 1);
    assert_eq!(
        execution.output_css,
        ".a { \n  color: red;\n  content: \"// keep\";\n}"
    );
}
