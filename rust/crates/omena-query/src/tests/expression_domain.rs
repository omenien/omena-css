use super::*;

#[test]
fn owns_expression_domain_flow_analysis_wrapper_without_changing_product() {
    let input = sample_input();
    let summary = summarize_omena_query_expression_domain_flow_analysis(&input);

    assert_eq!(summary.schema_version, "0");
    assert_eq!(
        summary.product,
        "engine-input-producers.expression-domain-flow-analysis"
    );
    assert_eq!(summary.input_version, "2");
    assert_eq!(summary.analyses.len(), 2);
    assert!(
        summary
            .analyses
            .iter()
            .all(|entry| entry.analysis.product == "omena-abstract-value.flow-analysis")
    );
    assert!(
        summary
            .analyses
            .iter()
            .all(|entry| entry.analysis.converged)
    );
}

#[test]
fn owns_expression_domain_control_flow_analysis_wrapper_without_changing_product() {
    let input = sample_input();
    let summary = summarize_omena_query_expression_domain_control_flow_analysis(&input);

    assert_eq!(summary.schema_version, "0");
    assert_eq!(
        summary.product,
        "engine-input-producers.expression-domain-control-flow-analysis"
    );
    assert_eq!(summary.input_version, "2");
    assert_eq!(summary.analyses.len(), 2);
    assert!(
        summary
            .analyses
            .iter()
            .all(|entry| entry.analysis.product == "omena-abstract-value.control-flow-analysis")
    );
}

#[test]
fn owns_expression_domain_call_site_flow_analysis_wrapper_without_changing_product() {
    let input = sample_input();
    let summary = summarize_omena_query_expression_domain_call_site_flow_analysis(&input);

    assert_eq!(summary.schema_version, "0");
    assert_eq!(
        summary.product,
        "engine-input-producers.expression-domain-call-site-flow-analysis"
    );
    assert_eq!(summary.input_version, "2");
    assert_eq!(summary.zero_cfa.context_sensitivity, "0-cfa");
    assert_eq!(summary.one_cfa.context_sensitivity, "1-cfa");
    assert_eq!(summary.zero_cfa.max_context_depth, 0);
    assert_eq!(summary.one_cfa.max_context_depth, 1);
    assert_eq!(summary.zero_cfa.call_site_count, 2);
    assert_eq!(summary.one_cfa.call_site_count, 2);
    assert!(
        summary
            .zero_cfa
            .entries
            .iter()
            .all(|entry| entry.context_key == "expression-domain-class-value@<root>")
    );
    assert_ne!(
        summary.one_cfa.entries[0].context_key,
        summary.one_cfa.entries[1].context_key
    );
}

#[test]
fn owns_expression_domain_provenance_explanations_wrapper_without_changing_product() {
    let input = sample_input();
    let summary = summarize_omena_query_expression_domain_provenance_explanations(&input);

    assert_eq!(summary.schema_version, "0");
    assert_eq!(
        summary.product,
        "engine-input-producers.expression-domain-provenance-explanations"
    );
    assert_eq!(summary.input_version, "2");
    assert_eq!(summary.explanation_count, 2);
    assert_eq!(summary.explanations[0].expression_id, "expr-1");
    assert_eq!(summary.explanations[0].reduced_kind, "prefixSuffix");
    assert_eq!(
        summary.explanations[0].derivation.product,
        "omena-abstract-value.reduced-class-value-derivation"
    );
    assert_eq!(
        summary.explanations[0].provenance_tree.product,
        "omena-abstract-value.provenance-tree"
    );
}

#[test]
fn owns_expression_domain_reduced_product_iteration_wrapper_without_changing_product() {
    let input = reduced_product_iteration_input();
    let summary = summarize_omena_query_expression_domain_reduced_product_iteration(&input);

    assert_eq!(summary.schema_version, "0");
    assert_eq!(
        summary.product,
        "engine-input-producers.expression-domain-reduced-product-iteration"
    );
    assert_eq!(summary.input_version, "2");
    assert_eq!(summary.iteration_count, 1);
    assert_eq!(summary.iterations[0].expression_id, "expr-reduced");
    assert_eq!(summary.iterations[0].axis_constraint_count, 3);
    assert_eq!(summary.iterations[0].iteration.input_count, 3);
    assert_eq!(summary.iterations[0].iteration.result_kind, "composite");
    assert!(summary.iterations[0].iteration.converged);
    assert!(summary.iterations[0].iteration.monotone_witness_valid);
}

#[test]
fn reuses_expression_domain_flow_analysis_through_salsa_runtime() {
    let input = sample_input();
    let mut runtime = OmenaQueryExpressionDomainFlowRuntimeV0::default();

    let first =
        summarize_omena_query_expression_domain_incremental_flow_analysis(&input, &mut runtime);
    assert_eq!(
        first.product,
        "omena-query.expression-domain-incremental-flow-analysis"
    );
    assert_eq!(first.revision, 1);
    assert_eq!(first.graph_count, 2);
    assert_eq!(first.dirty_graph_count, 2);
    assert_eq!(first.reused_graph_count, 0);
    assert_eq!(runtime.graph_count(), 2);

    let second =
        summarize_omena_query_expression_domain_incremental_flow_analysis(&input, &mut runtime);
    assert_eq!(second.revision, 2);
    assert_eq!(second.graph_count, 2);
    assert_eq!(second.dirty_graph_count, 0);
    assert_eq!(second.reused_graph_count, 2);
    assert!(
        second
            .analyses
            .iter()
            .all(|entry| entry.analysis.reused_previous_analysis)
    );
}

#[test]
fn projects_reduced_product_flow_to_target_style_selectors() {
    let input = reduced_product_projection_input();
    let summary = summarize_omena_query_expression_domain_selector_projection(&input);

    assert_eq!(summary.schema_version, "0");
    assert_eq!(
        summary.product,
        "omena-query.expression-domain-selector-projection"
    );
    assert_eq!(summary.input_version, "2");
    assert_eq!(summary.projection_count, 3);

    let merge = summary
        .projections
        .iter()
        .find(|projection| projection.node_id == "file-merge");
    assert!(merge.is_some());
    let Some(merge) = merge else {
        return;
    };
    assert_eq!(merge.graph_id, "/tmp/App.tsx:expression-domain-flow");
    assert_eq!(merge.file_path, "/tmp/App.tsx");
    assert_eq!(
        merge.target_style_paths,
        vec!["/tmp/App.module.scss".to_string()]
    );
    assert_eq!(merge.value_kind, "composite");
    assert_eq!(
        merge
            .reduced_product
            .as_ref()
            .map(|product| product.source_value_kind),
        Some("composite")
    );
    assert_eq!(
        merge
            .reduced_product
            .as_ref()
            .and_then(|product| product.prefix.as_ref())
            .map(|axis| axis.prefix.as_str()),
        Some("btn-")
    );
    assert_eq!(
        merge
            .reduced_product
            .as_ref()
            .and_then(|product| product.suffix.as_ref())
            .map(|axis| axis.suffix.as_str()),
        Some("-active")
    );
    assert_eq!(
        merge.selector_names,
        vec![
            "btn-primary--active".to_string(),
            "btn-secondary--active".to_string()
        ]
    );
    assert_eq!(merge.certainty, SelectorProjectionCertaintyV0::Inferred);
}

#[test]
fn builds_semantic_reachability_transform_context_from_expression_projection() {
    let input = reduced_product_projection_input();
    let context_summary = summarize_omena_query_transform_context_from_engine_input(
        &input,
        "/tmp/App.module.scss",
        true,
    );

    assert_eq!(
        context_summary.product,
        "omena-query.transform-context-from-engine-input"
    );
    assert!(context_summary.closed_style_world);
    assert_eq!(context_summary.projection_count, 3);
    assert_eq!(context_summary.selected_projection_count, 3);
    assert_eq!(context_summary.reachable_class_name_count, 2);
    assert_eq!(context_summary.reachability_sources.len(), 3);
    let merge_source = context_summary
        .reachability_sources
        .iter()
        .find(|source| source.node_id == "file-merge");
    assert!(merge_source.is_some());
    let Some(merge_source) = merge_source else {
        return;
    };
    assert_eq!(
        merge_source.selector_names,
        vec![
            "btn-primary--active".to_string(),
            "btn-secondary--active".to_string()
        ]
    );
    assert_eq!(
        context_summary.context.reachable_class_names,
        vec![
            "btn-primary--active".to_string(),
            "btn-secondary--active".to_string()
        ]
    );
    assert!(
        context_summary
            .ready_surfaces
            .contains(&"semanticReachabilityTransformContext")
    );

    let source = r#".btn-primary--active { color: red; } .btn-secondary--active { color: blue; } .btn--active { color: purple; } .card-active { color: gray; }"#;
    let build = execute_omena_query_consumer_build_style_source_with_engine_input_context(
        "/tmp/App.module.scss",
        source,
        &["tree-shake-class".to_string()],
        &input,
        true,
    );

    assert!(build.execution.output_css.contains(".btn-primary--active"));
    assert!(
        build
            .execution
            .output_css
            .contains(".btn-secondary--active")
    );
    assert!(!build.execution.output_css.contains(".btn--active"));
    assert!(!build.execution.output_css.contains(".card-active"));
    assert!(
        build
            .execution
            .executed_pass_ids
            .contains(&"tree-shake-class")
    );
    assert!(
        build
            .ready_surfaces
            .contains(&"semanticReachabilityTransformContext")
    );
    assert!(
        build
            .ready_surfaces
            .contains(&"expressionDomainSelectorProjection")
    );
}

#[test]
fn engine_input_transform_context_consumes_style_sources_for_workspace_context() {
    let source = r#"@import "./tokens.css" supports(display: grid); .button { composes: base; color: var(--brand); } .base { color: blue; }"#;
    let input = EngineInputV2 {
        version: "2".to_string(),
        sources: Vec::new(),
        styles: vec![
            StyleAnalysisInputV2 {
                file_path: "/tmp/Button.module.css".to_string(),
                source: Some(source.to_string()),
                document: StyleDocumentV2 {
                    selectors: vec![style_selector("button"), style_selector("base")],
                },
            },
            StyleAnalysisInputV2 {
                file_path: "/tmp/tokens.css".to_string(),
                source: Some(":root { --brand: red; }".to_string()),
                document: StyleDocumentV2 {
                    selectors: Vec::new(),
                },
            },
        ],
        type_facts: Vec::new(),
    };

    let context_summary = summarize_omena_query_transform_context_from_engine_input(
        &input,
        "/tmp/Button.module.css",
        false,
    );
    assert_eq!(context_summary.style_source_count, 2);
    assert_eq!(context_summary.import_inline_count, 1);
    assert_eq!(context_summary.class_name_rewrite_count, 2);
    assert_eq!(context_summary.css_module_composes_resolution_count, 1);
    assert_eq!(context_summary.css_module_value_resolution_count, 0);
    assert_eq!(context_summary.design_token_route_count, 1);
    assert!(
        context_summary
            .ready_surfaces
            .contains(&"engineInputStyleSourceTransformContext")
    );

    let build = execute_omena_query_consumer_build_style_source_with_engine_input_context(
        "/tmp/Button.module.css",
        source,
        &[
            "import-inline".to_string(),
            "composes-resolution".to_string(),
        ],
        &input,
        false,
    );

    assert!(build.execution.executed_pass_ids.contains(&"import-inline"));
    assert!(
        build
            .execution
            .executed_pass_ids
            .contains(&"composes-resolution")
    );
    assert!(
        build
            .execution
            .output_css
            .contains("@supports (display: grid) { :root { --brand: red; } }")
    );
    assert!(!build.execution.output_css.contains("@import"));
    assert!(!build.execution.output_css.contains("composes:"));
}
