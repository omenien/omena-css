use crate::{
    TransformExecutionContextV0, TransformPassDispatchKindV0, default_transform_pass_registry,
    execute_transform_passes_incremental_with_database,
    execute_transform_passes_on_source_with_dialect_and_context, plan_transform_passes,
    run_transform_fuzz_seed_corpus, summarize_omena_transform_passes_boundary,
    summarize_structural_ir_shadow_equivalence_v0, transform_pass_incremental_graph_input,
};
use omena_incremental::{IncrementalRevisionV0, OmenaIncrementalDatabaseV0};
use omena_parser::StyleDialect;
use omena_transform_cst::{
    TRANSFORM_PASS_CATALOG_LEN, TransformPassClassV0, TransformPassKind, all_transform_pass_kinds,
    default_transform_pass_contracts,
};

#[test]
fn registry_covers_full_transform_catalog() {
    let boundary = summarize_omena_transform_passes_boundary();

    assert_eq!(boundary.schema_version, "0");
    assert_eq!(boundary.product, "omena-transform-passes.boundary");
    assert_eq!(boundary.pass_count, TRANSFORM_PASS_CATALOG_LEN);
    assert!(boundary.full_catalog_registered);
    assert_eq!(boundary.semantic_aware_pass_count, 14);
    assert!(boundary.cascade_aware_pass_count >= 9);
    assert_eq!(boundary.structural_pass_count, 21);
    assert_eq!(boundary.text_local_pass_count, 20);
    assert_eq!(boundary.module_evaluation_pass_count, 2);
    assert!(boundary.planner_enforces_dag_edges);
    assert!(boundary.planner_uses_pass_descriptors);
    assert!(!boundary.ordinal_has_execution_semantics);
    assert!(boundary.execution_runtime_ready);
    assert!(boundary.incremental_execution_runtime_ready);
    assert_eq!(
        boundary.module_evaluation_native_output_marker,
        "nativeEditOutput"
    );
    assert!(boundary.module_evaluation_requires_native_product_output);
    assert!(boundary.module_evaluation_requires_oracle_readiness);
    assert!(boundary.module_evaluation_legacy_output_is_oracle_only);
    assert!(boundary.module_evaluation_preserves_source_without_native_output);
    let mutation_pass_ids = default_transform_pass_contracts()
        .into_iter()
        .filter(|contract| contract.executes_mutation)
        .map(|contract| contract.id)
        .collect::<Vec<_>>();
    assert_eq!(boundary.implemented_mutation_pass_ids, mutation_pass_ids);
    assert_eq!(
        boundary.implemented_mutation_pass_ids.len(),
        TRANSFORM_PASS_CATALOG_LEN
    );
    assert!(boundary.registry_entries.iter().any(|entry| {
        entry.contract.kind == TransformPassKind::TreeShakeClass
            && entry.descriptor.kind == TransformPassKind::TreeShakeClass
            && entry.descriptor.pass_class == TransformPassClassV0::Structural
            && entry.module_family == "semantic-reachability"
    }));
    assert!(boundary.registry_entries.iter().any(|entry| {
        entry.contract.kind == TransformPassKind::StalePrefixRemoval
            && entry.descriptor.kind == TransformPassKind::StalePrefixRemoval
            && entry.descriptor.pass_class == TransformPassClassV0::TextLocal
            && entry.module_family == "target-lowering"
            && entry.contract.read_model == omena_transform_cst::TransformPassReadModel::TargetData
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
fn pass_registry_subsumes_contracts_and_descriptors() {
    let registry = default_transform_pass_registry();
    let boundary = summarize_omena_transform_passes_boundary();

    assert_eq!(registry.schema_version, "0");
    assert_eq!(registry.product, "omena-transform-passes.pass-registry");
    assert_eq!(registry.entries.len(), TRANSFORM_PASS_CATALOG_LEN);
    assert_eq!(boundary.registry_entries, registry.entries);
    assert!(registry.entries.iter().all(|entry| {
        entry.contract.kind == entry.descriptor.kind && entry.contract.id == entry.descriptor.id
    }));
    assert!(registry.entries.iter().all(|entry| {
        let expected_dispatch_kind = match entry.contract.kind {
            TransformPassKind::ImportInline
            | TransformPassKind::ResolveCssModulesComposes
            | TransformPassKind::DesignTokenRouting
            | TransformPassKind::HashCssModuleClassNames => {
                TransformPassDispatchKindV0::ModuleEvaluationOrEgressHandler
            }
            _ => match entry.descriptor.pass_class {
                TransformPassClassV0::TextLocal => {
                    TransformPassDispatchKindV0::TextLocalSliceRewrite
                }
                TransformPassClassV0::Structural => TransformPassDispatchKindV0::StructuralHandler,
                TransformPassClassV0::ModuleEvaluation => {
                    TransformPassDispatchKindV0::ModuleEvaluationOrEgressHandler
                }
                TransformPassClassV0::Emission => TransformPassDispatchKindV0::EmissionBoundary,
            },
        };
        entry.dispatch_kind == expected_dispatch_kind
    }));
    assert!(
        registry
            .entries
            .iter()
            .filter(|entry| entry.descriptor.pass_class == TransformPassClassV0::Emission)
            .all(|entry| entry.contract.kind == TransformPassKind::PrintCss)
    );
}

#[test]
fn structural_ir_shadow_report_covers_structural_ir_paths() {
    let report = summarize_structural_ir_shadow_equivalence_v0();

    assert_eq!(
        report.product,
        "omena-transform-passes.structural-ir-shadow-equivalence"
    );
    assert_eq!(
        report.compared_pass_ids,
        vec![
            "container-static-eval",
            "dead-media-branch-removal",
            "dead-supports-branch-removal",
            "empty-rule-removal",
            "layer-flatten",
            "media-static-eval",
            "nesting-unwrap",
            "rule-deduplication",
            "rule-merging",
            "scope-flatten",
            "selector-merging",
            "supports-static-eval",
            "tree-shake-class",
            "tree-shake-custom-property",
            "tree-shake-keyframes",
            "tree-shake-value"
        ]
    );
    assert_eq!(
        report.compared_fields,
        vec![
            "canonicalCssBytes",
            "selectorSet",
            "declarationSet",
            "cascadeOutcome",
            "mutationSpanRanges",
            "mutationCount",
            "semanticRemovals"
        ]
    );
    assert_eq!(report.fixture_count, 23);
    assert!(report.all_fields_match, "{report:#?}");
    assert!(report.reports.iter().all(|fixture| {
        fixture.all_fields_match
            && fixture.string_path_mutation_count == fixture.ir_path_mutation_count
    }));
}

#[test]
fn planner_uses_descriptor_order_without_pass_ordinals() -> Result<(), String> {
    let planner_source = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("runtime")
            .join("planner.rs"),
    )
    .map_err(|err| format!("planner source should be readable: {err:?}"))?;

    assert!(
        !planner_source.contains(".ordinal()"),
        "execution planner must not use TransformPassKind::ordinal as an ordering tiebreak"
    );
    assert!(planner_source.contains("descriptor.phase"));
    assert!(planner_source.contains("descriptor.phase_order"));
    assert!(planner_source.contains("descriptor.depends_on"));

    let plan = plan_transform_passes(&[
        TransformPassKind::WhitespaceStrip,
        TransformPassKind::PrintCss,
        TransformPassKind::CalcReduction,
        TransformPassKind::StaticVarSubstitution,
    ]);
    assert_eq!(
        plan.build_profile.pass_ids,
        vec![
            "custom-property-static-resolve",
            "calc-reduction",
            "whitespace-strip",
            "print-css"
        ]
    );
    assert_eq!(plan.build_profile.pass_ids, plan.ordered_pass_ids);
    Ok(())
}

#[test]
fn contract_execution_phases_preserve_full_catalog_ordering() {
    let requested = all_transform_pass_kinds();
    let plan = plan_transform_passes(&requested);

    assert_eq!(plan.violated_dag_edge_count, 0);
    assert!(plan.all_requested_registered);
    assert_eq!(
        plan.ordered_pass_ids,
        vec![
            "import-inline",
            "scss-module-evaluate",
            "less-module-evaluate",
            "composes-resolution",
            "value-resolution",
            "custom-property-static-resolve",
            "tree-shake-class",
            "tree-shake-keyframes",
            "tree-shake-value",
            "tree-shake-custom-property",
            "dead-media-branch-removal",
            "dead-supports-branch-removal",
            "design-token-routing",
            "light-dark-lowering",
            "color-mix-lowering",
            "oklch-oklab-lowering",
            "color-function-lowering",
            "logical-to-physical",
            "nesting-unwrap",
            "css-modules-class-hashing",
            "scope-flatten",
            "layer-flatten",
            "supports-static-eval",
            "media-static-eval",
            "relative-color-lowering",
            "container-static-eval",
            "native-css-static-eval",
            "vendor-prefixing",
            "stale-prefix-removal",
            "selector-is-where-compression",
            "shorthand-combining",
            "rule-deduplication",
            "rule-merging",
            "calc-reduction",
            "comment-strip",
            "empty-rule-removal",
            "number-compression",
            "unit-normalization",
            "color-compression",
            "url-quote-strip",
            "string-quote-normalize",
            "selector-merging",
            "whitespace-strip",
            "print-css",
        ]
    );
}

#[test]
fn mutation_contracts_have_executor_outcomes() {
    let context = TransformExecutionContextV0::default();
    let source = ".used { color: #ffffff; margin: 0px; user-select: none; }";

    for contract in default_transform_pass_contracts()
        .into_iter()
        .filter(|contract| contract.executes_mutation)
    {
        let summary = execute_transform_passes_on_source_with_dialect_and_context(
            source,
            StyleDialect::Css,
            &[contract.kind],
            &context,
        );
        let outcome = summary
            .outcomes
            .iter()
            .find(|outcome| outcome.pass_id == contract.id);

        assert!(
            outcome.is_some(),
            "missing executor outcome for {}",
            contract.id
        );
        assert_ne!(
            outcome.map(|outcome| outcome.detail),
            Some("unknown pass id in execution plan"),
            "executor fell through to unknown-pass outcome for {}",
            contract.id
        );
    }
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
fn planner_respects_minify_structural_ordering_edges() {
    let plan = plan_transform_passes(&[
        TransformPassKind::PrintCss,
        TransformPassKind::WhitespaceStrip,
        TransformPassKind::EmptyRuleRemoval,
        TransformPassKind::SelectorMerging,
        TransformPassKind::RuleMerging,
        TransformPassKind::RuleDeduplication,
        TransformPassKind::ShorthandCombining,
        TransformPassKind::ColorCompression,
        TransformPassKind::NumberCompression,
        TransformPassKind::CommentStrip,
        TransformPassKind::CalcReduction,
    ]);

    assert_eq!(plan.violated_dag_edge_count, 0);
    assert_before(
        &plan.ordered_pass_ids,
        "shorthand-combining",
        "rule-merging",
    );
    assert_before(
        &plan.ordered_pass_ids,
        "shorthand-combining",
        "selector-merging",
    );
    assert_before(
        &plan.ordered_pass_ids,
        "comment-strip",
        "empty-rule-removal",
    );
    assert_before(
        &plan.ordered_pass_ids,
        "selector-merging",
        "whitespace-strip",
    );
    assert_eq!(plan.ordered_pass_ids.last(), Some(&"print-css"));
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

fn assert_before(pass_ids: &[&'static str], before: &'static str, after: &'static str) {
    let before_index = pass_ids.iter().position(|pass_id| *pass_id == before);
    let after_index = pass_ids.iter().position(|pass_id| *pass_id == after);
    assert!(
        before_index
            .zip(after_index)
            .is_some_and(|(before_index, after_index)| before_index < after_index),
        "expected {before} before {after}, got {pass_ids:?}"
    );
}
