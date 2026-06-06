use crate::{
    TransformExecutionContextV0, execute_transform_passes_incremental_with_database,
    plan_transform_passes, run_transform_fuzz_seed_corpus,
    summarize_omena_transform_passes_boundary, transform_pass_incremental_graph_input,
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
fn planner_respects_value_and_var_resolution_before_static_branch_evaluation() {
    let plan = plan_transform_passes(&[
        TransformPassKind::MediaStaticEval,
        TransformPassKind::SupportsStaticEval,
        TransformPassKind::StaticVarSubstitution,
        TransformPassKind::ValueResolution,
        TransformPassKind::PrintCss,
    ]);

    assert_eq!(plan.violated_dag_edge_count, 0);
    assert!(plan.all_requested_registered);
    assert_eq!(
        plan.ordered_pass_ids,
        vec![
            "value-resolution",
            "custom-property-static-resolve",
            "supports-static-eval",
            "media-static-eval",
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
fn planner_respects_class_tree_shake_before_hash_edges() {
    let plan = plan_transform_passes(&[
        TransformPassKind::HashCssModuleClassNames,
        TransformPassKind::TreeShakeClass,
        TransformPassKind::PrintCss,
    ]);

    assert_eq!(plan.violated_dag_edge_count, 0);
    assert_eq!(
        plan.ordered_pass_ids,
        vec!["tree-shake-class", "css-modules-class-hashing", "print-css"]
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
