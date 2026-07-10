use super::*;
use crate::FactPrecision;

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
    assert!(summary.analyses.is_empty());
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
fn wraps_expression_domain_runtime_in_revision_aligned_analysis_result() -> Result<(), String> {
    let input = sample_input();
    let mut runtime = OmenaQueryExpressionDomainFlowRuntimeV0::default();

    let first = summarize_omena_query_expression_domain_incremental_flow_analysis_result(
        &input,
        &mut runtime,
    );
    let first_json = serde_json::to_value(&first)
        .map_err(|err| format!("analysis result should serialize: {err:?}"))?;

    assert_eq!(first.product, "omena-query.analysis-result");
    assert_eq!(first.revision, 1);
    assert_eq!(first.value.revision, first.revision);
    assert_eq!(first.precision.value_domain, "classValueFlow");
    assert_eq!(
        first.precision.revision_axis,
        "OmenaQueryExpressionDomainFlowRuntimeV0.revision"
    );
    assert!(
        first
            .provenance
            .contains(&"omena-query-core.expression-domain-runtime".to_string())
    );
    assert_eq!(first_json["schemaVersion"], "0");
    assert_eq!(
        first_json["precision"]["flowSensitivity"],
        "incrementalDataflow"
    );
    assert_eq!(first_json["value"]["revision"], first_json["revision"]);

    let roundtrip: OmenaQueryAnalysisResultV0<String> = serde_json::from_value(serde_json::json!({
        "schemaVersion": "0",
        "product": "omena-query.analysis-result",
        "value": "class-value-flow",
        "precision": {
            "product": "omena-query.analysis-precision",
            "valueDomain": "classValueFlow",
            "flowSensitivity": "incrementalDataflow",
            "contextSensitivity": "perExpressionGraph",
            "revisionAxis": "OmenaQueryExpressionDomainFlowRuntimeV0.revision"
        },
        "provenance": ["omena-query-core.expression-domain-runtime"],
        "revision": 7
    }))
    .map_err(|err| format!("generic analysis result should deserialize: {err:?}"))?;
    assert_eq!(roundtrip.value, "class-value-flow");
    assert_eq!(roundtrip.revision, 7);

    let second = summarize_omena_query_expression_domain_incremental_flow_analysis_result(
        &input,
        &mut runtime,
    );
    assert_eq!(second.revision, 2);
    assert_eq!(second.value.revision, 2);
    assert_eq!(runtime.revision(), 2);
    Ok(())
}

#[test]
fn projects_reduced_product_flow_to_target_style_selectors() {
    let input = reduced_product_projection_input();
    let (summary, precisions) =
        summarize_omena_query_expression_domain_selector_projection_with_precision(&input);

    assert_eq!(summary.schema_version, "0");
    assert_eq!(
        summary.product,
        "omena-query.expression-domain-selector-projection"
    );
    assert_eq!(summary.input_version, "2");
    assert_eq!(summary.projection_count, 2);
    assert_eq!(precisions.len(), summary.projection_count);
    assert_eq!(
        precisions
            .iter()
            .find(|precision| precision.node_id == "expr-primary")
            .map(|precision| precision.precision),
        Some(FactPrecision::Heuristic)
    );
    assert!(
        summary
            .projections
            .iter()
            .all(|projection| projection.node_id != "file-merge")
    );

    let primary = summary
        .projections
        .iter()
        .find(|projection| projection.node_id == "expr-primary");
    assert!(primary.is_some());
    let Some(primary) = primary else {
        return;
    };
    assert_eq!(
        primary.graph_id,
        "/tmp/App.tsx:expr-primary:expression-domain-flow"
    );
    assert_eq!(primary.file_path, "/tmp/App.tsx");
    assert_eq!(
        primary.target_style_paths,
        vec!["/tmp/App.module.scss".to_string()]
    );
    assert_eq!(primary.value_kind, "prefixSuffix");
    assert_eq!(
        primary
            .reduced_product
            .as_ref()
            .map(|product| product.source_value_kind),
        Some("prefixSuffix")
    );
    assert_eq!(
        primary
            .reduced_product
            .as_ref()
            .and_then(|product| product.prefix.as_ref())
            .map(|axis| axis.prefix.as_str()),
        Some("btn-primary-")
    );
    assert_eq!(
        primary
            .reduced_product
            .as_ref()
            .and_then(|product| product.suffix.as_ref())
            .map(|axis| axis.suffix.as_str()),
        Some("-active")
    );
    assert_eq!(
        primary.selector_names,
        vec!["btn-primary--active".to_string()]
    );
    assert_eq!(primary.certainty, SelectorProjectionCertaintyV0::Inferred);

    let secondary = summary
        .projections
        .iter()
        .find(|projection| projection.node_id == "expr-secondary");
    assert!(secondary.is_some());
    let Some(secondary) = secondary else {
        return;
    };
    assert_eq!(
        secondary.selector_names,
        vec!["btn-secondary--active".to_string()]
    );
    assert_eq!(secondary.certainty, SelectorProjectionCertaintyV0::Inferred);
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
    assert!(context_summary.closed_world_requested);
    assert_eq!(context_summary.projection_count, 2);
    assert_eq!(context_summary.selected_projection_count, 2);
    assert_eq!(context_summary.reachable_class_name_count, 2);
    assert_eq!(context_summary.reachability_sources.len(), 2);
    assert!(
        context_summary
            .reachability_sources
            .iter()
            .all(|source| source.node_id != "file-merge")
    );
    assert_eq!(
        context_summary
            .reachability_sources
            .iter()
            .flat_map(|source| source.selector_names.iter().cloned())
            .collect::<Vec<_>>(),
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

    assert_eq!(build.execution.output_css, source);
    assert_eq!(build.execution.mutation_count, 0);
    assert!(
        build
            .execution
            .planned_only_pass_ids
            .contains(&"tree-shake-class")
    );
    let decision = serde_json::to_value(
        build
            .execution
            .decisions
            .first()
            .expect("tree-shake decision should be present"),
    )
    .expect("tree-shake decision should serialize");
    assert_eq!(decision["kind"], "blocked");
    assert_eq!(decision["reason"]["kind"], "precisionBelowFloor");
    assert_eq!(decision["reason"]["required"], "conservative");
    assert_eq!(decision["reason"]["observed"], "heuristic");
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
