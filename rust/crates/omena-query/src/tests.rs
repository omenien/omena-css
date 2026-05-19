use engine_input_producers::{
    ClassExpressionInputV2, EngineInputV2, PositionV2, RangeV2, SourceAnalysisInputV2,
    SourceDocumentV2, StringTypeFactsV2, StyleAnalysisInputV2, StyleDocumentV2, StyleSelectorV2,
    TypeFactEntryV2,
};
use omena_abstract_value::SelectorProjectionCertaintyV0;

use super::{
    OmenaQueryExpressionDomainFlowRuntimeV0, OmenaQueryStylePackageManifestV0, ParserPositionV0,
    ParserRangeV0, SelectedQueryAdapterCapabilitiesV0,
    execute_omena_query_consumer_build_style_source,
    execute_omena_query_consumer_build_style_source_for_target_query,
    execute_omena_query_consumer_build_style_source_for_target_query_with_context_and_options,
    execute_omena_query_consumer_build_style_source_for_target_query_with_options,
    execute_omena_query_consumer_build_style_source_with_engine_input_context,
    execute_omena_query_consumer_build_style_sources,
    execute_omena_query_consumer_build_style_sources_for_target_query_with_context_and_options,
    execute_omena_query_consumer_build_style_sources_with_context,
    execute_omena_query_transform_passes_from_source, list_omena_query_transform_pass_summaries,
    summarize_omena_query_boundary, summarize_omena_query_consumer_check_style_source,
    summarize_omena_query_evaluation_runtime,
    summarize_omena_query_expression_domain_control_flow_analysis,
    summarize_omena_query_expression_domain_flow_analysis,
    summarize_omena_query_expression_domain_incremental_flow_analysis,
    summarize_omena_query_expression_domain_selector_projection,
    summarize_omena_query_expression_semantics_canonical_producer_signal,
    summarize_omena_query_expression_semantics_query_fragments,
    summarize_omena_query_fragment_bundle,
    summarize_omena_query_omena_parser_css_modules_intermediate,
    summarize_omena_query_omena_parser_lex, summarize_omena_query_omena_parser_style_facts,
    summarize_omena_query_selected_query_adapter_capabilities,
    summarize_omena_query_selector_usage_canonical_producer_signal,
    summarize_omena_query_selector_usage_query_fragments,
    summarize_omena_query_source_resolution_canonical_producer_signal,
    summarize_omena_query_source_resolution_query_fragments,
    summarize_omena_query_source_resolution_runtime, summarize_omena_query_style_document,
    summarize_omena_query_style_semantic_graph_batch_from_sources,
    summarize_omena_query_style_semantic_graph_batch_from_sources_with_package_manifests,
    summarize_omena_query_style_semantic_graph_from_source,
    summarize_omena_query_transform_context_from_engine_input,
    summarize_omena_query_transform_context_from_sources,
    summarize_omena_query_transform_plan_from_source,
    summarize_omena_query_transform_plan_from_target_query,
};
use crate::{
    OmenaQueryCompletionCandidateV0, OmenaQuerySourceDocumentInputV0,
    OmenaQuerySourceMissingSelectorDiagnosticCandidateV0,
    OmenaQuerySourceSelectorReferenceCandidateV0, OmenaQuerySourceSelectorReferenceEditTargetV0,
    OmenaQueryStyleSelectorDefinitionV0, OmenaQueryStyleSourceInputV0,
    OmenaQueryTargetFeatureSupportV0, OmenaQueryTargetTransformOptionsV0,
    OmenaQueryTransformExecutionContextV0, OmenaQueryTransformModuleEvaluationV0,
    default_omena_query_transform_print_options, modern_omena_query_target_feature_support,
};

#[test]
fn summarizes_query_boundary_over_producer_fragments() {
    let input = sample_input();
    let summary = summarize_omena_query_boundary(&input);

    assert_eq!(summary.schema_version, "0");
    assert_eq!(summary.product, "omena-query.boundary");
    assert_eq!(summary.query_engine_name, "omena-query");
    assert_eq!(summary.input_version, "2");
    assert_eq!(
        summary.abstract_value_domain.product,
        "omena-abstract-value.domain"
    );
    assert_eq!(
        summary.selected_query_adapter_capabilities.product,
        "omena-query.selected-query-adapter-capabilities"
    );
    assert_eq!(summary.expression_semantics_query_count, 2);
    assert_eq!(summary.source_resolution_query_count, 2);
    assert_eq!(summary.selector_usage_query_count, 2);
    assert_eq!(summary.total_query_count, 6);
    assert!(
        summary
            .ready_surfaces
            .contains(&"abstractValueProjectionContract")
    );
    assert!(
        summary
            .ready_surfaces
            .contains(&"abstractValueReducedProductAlgebra")
    );
    assert!(
        summary
            .abstract_value_domain
            .reduced_product_operations
            .contains(&"matchesString")
    );
    assert!(
        summary
            .ready_surfaces
            .contains(&"sourceResolutionResolverBoundary")
    );
    assert!(
        summary
            .delegated_fragment_products
            .contains(&"omena-resolver.boundary")
    );
    assert!(
        summary
            .delegated_fragment_products
            .contains(&"omena-resolver.source-resolution-runtime-index")
    );
    assert!(
        summary
            .delegated_fragment_products
            .contains(&"engine-input-producers.expression-domain-flow-analysis")
    );
    assert!(
        summary
            .delegated_fragment_products
            .contains(&"engine-input-producers.expression-domain-control-flow-analysis")
    );
    assert!(
        summary
            .delegated_fragment_products
            .contains(&"omena-query.expression-domain-incremental-flow-analysis")
    );
    assert!(
        summary
            .delegated_fragment_products
            .contains(&"omena-query.expression-domain-selector-projection")
    );
    assert!(
        summary
            .delegated_fragment_products
            .contains(&"omena-parser.style-facts")
    );
    assert!(
        summary
            .delegated_fragment_products
            .contains(&"omena-transform-bundle.source")
    );
    assert!(
        summary
            .delegated_fragment_products
            .contains(&"omena-transform-passes.plan")
    );
    assert!(
        summary
            .delegated_fragment_products
            .contains(&"omena-transform-egg.execution")
    );
    assert!(
        summary
            .delegated_fragment_products
            .contains(&"omena-query.consumer-check-style-source")
    );
    assert!(
        summary
            .delegated_fragment_products
            .contains(&"omena-query.consumer-build-style-source")
    );
    assert!(
        summary
            .ready_surfaces
            .contains(&"expressionDomainFlowAnalysisBoundary")
    );
    assert!(
        summary
            .ready_surfaces
            .contains(&"expressionDomainControlFlowAnalysisBoundary")
    );
    assert!(
        summary
            .ready_surfaces
            .contains(&"expressionDomainSalsaRuntime")
    );
    assert!(
        summary
            .ready_surfaces
            .contains(&"expressionDomainSelectorProjection")
    );
    assert!(
        summary
            .ready_surfaces
            .contains(&"transformEggExecutionWitnesses")
    );
    assert!(
        summary
            .ready_surfaces
            .contains(&"sourceResolutionRuntimeIndex")
    );
    assert!(
        summary
            .ready_surfaces
            .contains(&"omenaParserStyleFactExtraction")
    );
    assert!(summary.ready_surfaces.contains(&"transformPlanFacade"));
    assert!(
        summary
            .ready_surfaces
            .contains(&"transformExecutionRuntime")
    );
    assert!(summary.ready_surfaces.contains(&"consumerCheckFacade"));
    assert!(summary.ready_surfaces.contains(&"consumerBuildFacade"));
    assert!(
        summary
            .ready_surfaces
            .contains(&"consumerTransformPassListFacade")
    );
    assert!(summary.ready_surfaces.contains(&"readCascadeAtPosition"));
    assert!(
        summary
            .ready_surfaces
            .contains(&"selectedQueryBackendAdapter")
    );
    assert!(summary.ready_surfaces.contains(&"queryEvaluationRuntime"));
    assert!(
        summary
            .ready_surfaces
            .contains(&"omenaParserStyleDocumentSummary")
    );
    assert!(
        summary
            .ready_surfaces
            .contains(&"omenaParserPublicContractTypes")
    );
    assert!(summary.next_decoupling_targets.is_empty());
    assert!(
        summary
            .cme_coupled_surfaces
            .contains(&"producerQueryFragments")
    );
}

#[test]
fn exposes_omena_parser_style_fact_surface() {
    let summary = summarize_omena_query_omena_parser_style_facts(
        "@use \"tokens\"; @value primary: #fff; @value accent: primary; @value secondary as localSecondary from \"./tokens.module.scss\"; :export { primary: #fff; forwarded: imported; } :import(\"./tokens.css\") { imported: primary; } @keyframes fade { to { opacity: 1; } } $gap: 1rem; %surface { color: red; } .card#main { composes: base utility from \"./base.module.scss\"; --space: $gap; animation: 1s ease-in fade; }",
        omena_parser::StyleDialect::Scss,
    );

    assert_eq!(summary.schema_version, "0");
    assert_eq!(summary.product, "omena-query.omena-parser-style-facts");
    assert_eq!(summary.dialect, "scss");
    assert_eq!(summary.class_selector_names, vec!["card"]);
    assert_eq!(summary.id_selector_names, vec!["main"]);
    assert_eq!(summary.placeholder_selector_names, vec!["surface"]);
    assert_eq!(summary.keyframe_names, vec!["fade"]);
    assert_eq!(summary.animation_reference_names, vec!["fade"]);
    assert_eq!(
        summary.css_module_value_definition_names,
        vec!["accent", "localSecondary", "primary"]
    );
    assert_eq!(
        summary.css_module_value_reference_names,
        vec!["primary", "secondary"]
    );
    assert_eq!(
        summary.css_module_value_import_sources,
        vec!["./tokens.module.scss"]
    );
    assert_eq!(summary.css_module_value_import_edges.len(), 1);
    assert_eq!(
        summary.css_module_value_import_edges[0].remote_name,
        "secondary"
    );
    assert_eq!(
        summary.css_module_value_import_edges[0].local_name,
        "localSecondary"
    );
    assert_eq!(
        summary.css_module_value_import_edges[0].import_source,
        "./tokens.module.scss"
    );
    assert_eq!(summary.css_module_value_definition_edges.len(), 1);
    assert_eq!(
        summary.css_module_value_definition_edges[0].definition_name,
        "accent"
    );
    assert_eq!(
        summary.css_module_value_definition_edges[0].reference_names,
        vec!["primary"]
    );
    assert_eq!(
        summary.css_module_composes_target_names,
        vec!["base", "utility"]
    );
    assert_eq!(
        summary.css_module_composes_import_sources,
        vec!["./base.module.scss"]
    );
    assert_eq!(summary.css_module_composes_edges.len(), 1);
    assert_eq!(summary.css_module_composes_edges[0].kind, "external");
    assert_eq!(
        summary.css_module_composes_edges[0].owner_selector_names,
        vec!["card"]
    );
    assert_eq!(
        summary.css_module_composes_edges[0].target_names,
        vec!["base", "utility"]
    );
    assert_eq!(
        summary.css_module_composes_edges[0]
            .import_source
            .as_deref(),
        Some("./base.module.scss")
    );
    assert_eq!(summary.icss_export_names, vec!["forwarded", "primary"]);
    assert_eq!(summary.icss_import_local_names, vec!["imported"]);
    assert_eq!(summary.icss_import_remote_names, vec!["primary"]);
    assert_eq!(summary.icss_import_sources, vec!["./tokens.css"]);
    assert_eq!(summary.icss_import_edges.len(), 1);
    assert_eq!(summary.icss_import_edges[0].local_name, "imported");
    assert_eq!(summary.icss_import_edges[0].remote_name, "primary");
    assert_eq!(summary.icss_import_edges[0].import_source, "./tokens.css");
    assert_eq!(summary.icss_export_edges.len(), 1);
    assert_eq!(summary.icss_export_edges[0].export_name, "forwarded");
    assert_eq!(
        summary.icss_export_edges[0].reference_names,
        vec!["imported"]
    );
    assert!(summary.variable_names.contains(&"$gap".to_string()));
    assert!(
        summary
            .custom_property_names
            .contains(&"--space".to_string())
    );
    assert_eq!(
        summary.at_rule_names,
        vec!["@use", "@value", "@value", "@value", "@keyframes"]
    );
    assert_eq!(summary.parser_error_count, 0);
}

#[test]
fn exposes_omena_parser_css_modules_intermediate_surface() -> Result<(), serde_json::Error> {
    let summary = summarize_omena_query_omena_parser_css_modules_intermediate(
        "@value primary: #fff; .card { color: primary; }",
        omena_parser::StyleDialect::Css,
    );
    let summary = serde_json::to_value(summary)?;

    assert_eq!(summary["schemaVersion"], "0");
    assert_eq!(summary["language"], "css");
    assert_eq!(summary["selectors"]["names"], serde_json::json!(["card"]));
    assert_eq!(
        summary["values"]["declNames"],
        serde_json::json!(["primary"])
    );
    assert_eq!(
        summary["values"]["selectorsWithRefsNames"],
        serde_json::json!(["card"])
    );
    Ok(())
}

#[test]
fn exposes_omena_parser_lex_surface() {
    let summary = summarize_omena_query_omena_parser_lex(
        ".card { color: red; }",
        omena_parser::StyleDialect::Css,
    );

    assert_eq!(summary.schema_version, "0");
    assert_eq!(summary.product, "omena-parser.lex-result");
    assert_eq!(summary.dialect, "css");
    assert_eq!(summary.parser_error_count, 0);
    assert!(summary.tokens.len() >= 8);
    assert!(summary.tokens.iter().any(|token| token.text == "card"));
}

#[test]
fn exposes_transform_plan_facade_from_source() {
    let source = r#"
@use "./tokens" as tokens;
@value primary from "./colors.module.css";
.button {
  composes: reset from "./reset.module.css";
  color: tokens.$brand;
}
"#;
    let target_support = OmenaQueryTargetFeatureSupportV0 {
        vendor_prefix_required: true,
        supports_light_dark: false,
        supports_color_mix: true,
        supports_oklch_oklab: true,
        supports_color_function: true,
        supports_logical_properties: true,
        supports_css_nesting: false,
        supports_css_scope: true,
        supports_cascade_layers: true,
    };
    let target_options = OmenaQueryTargetTransformOptionsV0 {
        allow_logical_to_physical: false,
        allow_scope_flatten: false,
        allow_layer_flatten: false,
        enable_supports_static_eval: false,
        enable_media_static_eval: false,
        drop_dark_mode_media_queries: false,
    };

    let summary = summarize_omena_query_transform_plan_from_source(
        "Button.module.scss",
        source,
        "legacy-webview",
        target_support,
        target_options,
        default_omena_query_transform_print_options(),
    );

    assert_eq!(summary.product, "omena-query.transform-plan");
    assert_eq!(summary.dialect, "scss");
    assert_eq!(summary.target_query, None);
    assert!(summary.bundle.required_pass_ids.contains(&"import-inline"));
    assert!(
        summary
            .bundle
            .required_pass_ids
            .contains(&"composes-resolution")
    );
    assert!(
        summary
            .target
            .required_pass_ids
            .contains(&"light-dark-lowering")
    );
    assert!(summary.target.required_pass_ids.contains(&"nesting-unwrap"));
    assert!(summary.combined_pass_ids.contains(&"print-css"));
    assert_eq!(summary.combined_violated_dag_edge_count, 0);
    assert_eq!(summary.print.css, source);
    assert_eq!(summary.print.css, summary.execution.output_css);
    assert_eq!(
        summary.execution.product,
        "omena-transform-passes.execution"
    );
    assert_eq!(summary.execution.output_css, source);
    assert_eq!(
        summary.execution.executed_pass_ids,
        vec![
            "value-resolution",
            "light-dark-lowering",
            "nesting-unwrap",
            "vendor-prefixing",
            "print-css"
        ]
    );
    assert!(
        summary
            .execution
            .planned_only_pass_ids
            .contains(&"css-modules-class-hashing")
    );
    assert_eq!(summary.execution.pass_plan.violated_dag_edge_count, 0);
}

#[test]
fn exposes_transform_plan_egg_witnesses_from_source_execution() {
    let source = ".a:is(.ready) { width: calc(7 + 0); } .b:is(.x, .x) { color: red; } .c:where(.y, .y) { color: blue; }";
    let target_support = OmenaQueryTargetFeatureSupportV0 {
        vendor_prefix_required: false,
        supports_light_dark: true,
        supports_color_mix: true,
        supports_oklch_oklab: true,
        supports_color_function: true,
        supports_logical_properties: true,
        supports_css_nesting: true,
        supports_css_scope: true,
        supports_cascade_layers: true,
    };
    let target_options = OmenaQueryTargetTransformOptionsV0 {
        allow_logical_to_physical: false,
        allow_scope_flatten: false,
        allow_layer_flatten: false,
        enable_supports_static_eval: false,
        enable_media_static_eval: false,
        drop_dark_mode_media_queries: false,
    };

    let summary = summarize_omena_query_transform_plan_from_source(
        "Button.css",
        source,
        "modern",
        target_support,
        target_options,
        default_omena_query_transform_print_options(),
    );

    assert_eq!(
        summary.egg.planned_pass_ids,
        vec!["selector-is-where-compression", "calc-reduction"]
    );
    assert!(
        summary
            .ready_surfaces
            .contains(&"transformEggExecutionWitnesses")
    );
    assert_eq!(summary.egg_witnesses.len(), 4);
    assert!(
        summary
            .egg_witnesses
            .iter()
            .all(|witness| witness.execution.accepted)
    );
    assert!(summary.execution.output_css.contains(".a.ready"));
    assert!(summary.execution.output_css.contains(".b.x"));
    assert!(summary.execution.output_css.contains(".c:where(.y)"));
    assert!(summary.execution.output_css.contains("width: 7"));
    assert!(
        summary.egg_witnesses.iter().any(|witness| {
            witness.source_kind == "selectorIsDedup" && witness.css_after == ".x"
        })
    );
    assert!(summary.egg_witnesses.iter().any(|witness| {
        witness.source_kind == "selectorWhereDedup" && witness.css_after == ":where(.y)"
    }));
}

#[test]
fn exposes_transform_plan_custom_property_fixed_point() {
    let source = r#":root { --brand: red; --alias: var(--brand); --shadow: 0 0 var(--alias); --cycle-a: var(--cycle-b); --cycle-b: var(--cycle-a); } .card { color: var(--alias); box-shadow: var(--shadow); }"#;
    let summary = summarize_omena_query_transform_plan_from_source(
        "tokens.css",
        source,
        "modern",
        modern_omena_query_target_feature_support(),
        OmenaQueryTargetTransformOptionsV0 {
            allow_logical_to_physical: false,
            allow_scope_flatten: false,
            allow_layer_flatten: false,
            enable_supports_static_eval: false,
            enable_media_static_eval: false,
            drop_dark_mode_media_queries: false,
        },
        default_omena_query_transform_print_options(),
    );

    assert!(
        summary
            .ready_surfaces
            .contains(&"customPropertyLeastFixedPoint")
    );
    assert_eq!(summary.custom_property_fixed_point.input_count, 5);
    assert_eq!(summary.custom_property_fixed_point.resolved_count, 3);
    assert_eq!(
        summary.custom_property_fixed_point.guaranteed_invalid_count,
        2
    );
    assert!(
        summary
            .custom_property_fixed_point
            .entries
            .iter()
            .any(|entry| entry.name == "--alias" && entry.changed)
    );
    assert!(
        summary
            .custom_property_fixed_point
            .entries
            .iter()
            .any(|entry| entry.name == "--shadow" && entry.changed)
    );
}

#[test]
fn exposes_transform_plan_facade_from_browserslist_target_query() {
    let source = ".button { display: flex; color: light-dark(#000, #fff); }";
    let target_options = OmenaQueryTargetTransformOptionsV0 {
        allow_logical_to_physical: true,
        allow_scope_flatten: true,
        allow_layer_flatten: true,
        enable_supports_static_eval: false,
        enable_media_static_eval: false,
        drop_dark_mode_media_queries: false,
    };

    let summary = summarize_omena_query_transform_plan_from_target_query(
        "Button.module.css",
        source,
        "ie 11",
        target_options,
        default_omena_query_transform_print_options(),
    );

    assert!(summary.target_query.is_some());
    let Some(target_query) = summary.target_query.as_ref() else {
        return;
    };
    assert_eq!(target_query.profile_id, "browserslist-resolved");
    assert_eq!(target_query.resolved_targets, vec!["ie 11"]);
    assert_eq!(target_query.resolution_error, None);
    assert_eq!(summary.target, target_query.transform_plan);
    assert!(
        summary
            .target
            .required_pass_ids
            .contains(&"vendor-prefixing")
    );
    assert!(
        summary
            .target
            .required_pass_ids
            .contains(&"light-dark-lowering")
    );
    assert_eq!(summary.combined_violated_dag_edge_count, 0);
}

#[test]
fn exposes_transform_execution_runner_from_source() {
    let source = r#".a { color: red; /* remove */ content: "/* keep */"; }"#;
    let summary = execute_omena_query_transform_passes_from_source(
        "Button.module.css",
        source,
        &[
            "comment-strip".to_string(),
            "print-css".to_string(),
            "unknown-transform-pass".to_string(),
        ],
    );

    assert_eq!(summary.product, "omena-query.transform-execute");
    assert_eq!(summary.style_path, "Button.module.css");
    assert_eq!(summary.unknown_pass_ids, vec!["unknown-transform-pass"]);
    assert_eq!(
        summary.execution.product,
        "omena-transform-passes.execution"
    );
    assert_eq!(summary.execution.mutation_count, 1);
    assert_eq!(
        summary.execution.output_css,
        r#".a { color: red;  content: "/* keep */"; }"#
    );
    assert_eq!(
        summary.execution.executed_pass_ids,
        vec!["comment-strip", "print-css"]
    );
    assert!(
        summary
            .ready_surfaces
            .contains(&"transformExecutionRuntime")
    );
}

#[test]
fn exposes_consumer_check_facade_from_query() {
    let summary = summarize_omena_query_consumer_check_style_source(
        "Button.module.scss",
        ".card { color: red; }\n:root { --brand: blue; }",
    );

    assert_eq!(summary.product, "omena-query.consumer-check-style-source");
    assert_eq!(summary.style_path, "Button.module.scss");
    assert_eq!(summary.dialect, "scss");
    assert_eq!(summary.parser_error_count, 0);
    assert_eq!(summary.class_selector_count, 1);
    assert_eq!(summary.custom_property_count, 1);
    assert!(summary.ready_surfaces.contains(&"consumerCheckFacade"));
}

#[test]
fn exposes_consumer_build_facade_from_query() {
    let pass_ids = vec![
        "color-compression".to_string(),
        "unknown-transform-pass".to_string(),
    ];
    let summary = execute_omena_query_consumer_build_style_source(
        "Button.module.css",
        ".card { color: #ffffff; }",
        &pass_ids,
    );

    assert_eq!(summary.product, "omena-query.consumer-build-style-source");
    assert_eq!(summary.dialect, "css");
    assert_eq!(summary.requested_pass_ids, pass_ids);
    assert_eq!(summary.target_query, None);
    assert_eq!(summary.unknown_pass_ids, vec!["unknown-transform-pass"]);
    assert!(summary.execution.output_css.contains("#fff"));
    assert!(summary.ready_surfaces.contains(&"consumerBuildFacade"));
}

#[test]
fn consumer_build_derives_single_source_transform_context() {
    let summary = execute_omena_query_consumer_build_style_source(
        "Button.module.css",
        ".button { composes: base; color: red; } .base { color: blue; }",
        &[
            "composes-resolution".to_string(),
            "css-modules-class-hashing".to_string(),
            "print-css".to_string(),
        ],
    );

    assert!(
        summary
            .ready_surfaces
            .contains(&"singleSourceTransformContextProducer")
    );
    assert!(
        summary
            .execution
            .executed_pass_ids
            .contains(&"composes-resolution")
    );
    assert!(
        summary
            .execution
            .executed_pass_ids
            .contains(&"css-modules-class-hashing")
    );
    assert!(
        !summary
            .execution
            .planned_only_pass_ids
            .contains(&"composes-resolution")
    );
    assert!(
        !summary
            .execution
            .planned_only_pass_ids
            .contains(&"css-modules-class-hashing")
    );
    assert!(!summary.execution.output_css.contains("composes:"));
    assert!(summary.execution.output_css.contains("._button_0"));
}

#[test]
fn exposes_consumer_build_facade_from_target_query() {
    let summary = execute_omena_query_consumer_build_style_source_for_target_query(
        "Button.module.css",
        ".card { display: flex; color: light-dark(#000, #fff); }",
        "ie 11",
    );

    assert_eq!(summary.product, "omena-query.consumer-build-style-source");
    assert_eq!(summary.dialect, "css");
    assert!(summary.unknown_pass_ids.is_empty());
    assert!(summary.target_query.is_some());
    let Some(target_query) = summary.target_query.as_ref() else {
        return;
    };
    assert_eq!(target_query.profile_id, "browserslist-resolved");
    assert_eq!(target_query.resolved_targets, vec!["ie 11"]);
    assert!(
        summary
            .requested_pass_ids
            .iter()
            .any(|pass_id| pass_id == "vendor-prefixing")
    );
    assert!(
        summary
            .requested_pass_ids
            .iter()
            .any(|pass_id| pass_id == "light-dark-lowering")
    );
    assert!(
        summary
            .execution
            .executed_pass_ids
            .contains(&"css-modules-class-hashing")
    );
    assert!(
        !summary
            .execution
            .planned_only_pass_ids
            .contains(&"css-modules-class-hashing")
    );
    assert!(summary.execution.output_css.contains("._card_0"));
    assert!(summary.ready_surfaces.contains(&"targetQueryBuildFacade"));
}

#[test]
fn exposes_consumer_build_facade_from_target_query_options() {
    let summary = execute_omena_query_consumer_build_style_source_for_target_query_with_options(
        "Button.module.css",
        ".card { margin-inline: 1rem; @scope (.card) { & { color: red; } } }",
        "ie 11",
        OmenaQueryTargetTransformOptionsV0 {
            allow_logical_to_physical: true,
            allow_scope_flatten: true,
            allow_layer_flatten: true,
            enable_supports_static_eval: true,
            enable_media_static_eval: true,
            drop_dark_mode_media_queries: false,
        },
    );

    assert!(summary.unknown_pass_ids.is_empty());
    assert!(
        summary
            .requested_pass_ids
            .iter()
            .any(|pass_id| pass_id == "logical-to-physical")
    );
    assert!(
        summary
            .requested_pass_ids
            .iter()
            .any(|pass_id| pass_id == "scope-flatten")
    );
    assert!(
        summary
            .requested_pass_ids
            .iter()
            .any(|pass_id| pass_id == "supports-static-eval")
    );
}

#[test]
fn target_query_options_drop_dark_media_branches_through_execution_context() {
    let summary = execute_omena_query_consumer_build_style_source_for_target_query_with_options(
        "Theme.css",
        r#"@media (prefers-color-scheme: dark) { .dark { color: white; } } @media (prefers-color-scheme: light) { .light { color: black; } }"#,
        "modern",
        OmenaQueryTargetTransformOptionsV0 {
            allow_logical_to_physical: false,
            allow_scope_flatten: false,
            allow_layer_flatten: false,
            enable_supports_static_eval: false,
            enable_media_static_eval: false,
            drop_dark_mode_media_queries: true,
        },
    );

    assert!(
        summary
            .requested_pass_ids
            .contains(&"dead-media-branch-removal".to_string())
    );
    assert!(
        !summary
            .execution
            .output_css
            .contains("prefers-color-scheme: dark")
    );
    assert!(
        summary
            .execution
            .output_css
            .contains("prefers-color-scheme: light")
    );
}

#[test]
fn consumer_build_accepts_explicit_scss_evaluator_context() {
    let context = OmenaQueryTransformExecutionContextV0 {
        scss_module_evaluation: Some(OmenaQueryTransformModuleEvaluationV0 {
            evaluator: "dart-sass-compatible".to_string(),
            evaluated_css: ".button { color: red; }".to_string(),
        }),
        ..OmenaQueryTransformExecutionContextV0::default()
    };
    let summary =
        execute_omena_query_consumer_build_style_source_for_target_query_with_context_and_options(
            "Button.module.scss",
            "$brand: red; .button { color: $brand; }",
            "ie 11",
            &context,
            OmenaQueryTargetTransformOptionsV0 {
                allow_logical_to_physical: false,
                allow_scope_flatten: false,
                allow_layer_flatten: false,
                enable_supports_static_eval: false,
                enable_media_static_eval: false,
                drop_dark_mode_media_queries: false,
            },
        );

    assert!(
        summary
            .execution
            .executed_pass_ids
            .contains(&"scss-module-evaluate")
    );
    assert!(
        !summary
            .execution
            .planned_only_pass_ids
            .contains(&"scss-module-evaluate")
    );
    assert!(summary.execution.output_css.contains("color: red"));
    assert!(summary.execution.output_css.contains("._button_0"));
}

#[test]
fn consumer_build_derives_static_scss_evaluator_context() {
    let summary = execute_omena_query_consumer_build_style_source(
        "Button.module.scss",
        "$brand: red; .button { color: $brand; }",
        &[
            "scss-module-evaluate".to_string(),
            "css-modules-class-hashing".to_string(),
            "print-css".to_string(),
        ],
    );

    assert!(
        summary
            .execution
            .executed_pass_ids
            .contains(&"scss-module-evaluate")
    );
    assert!(
        !summary
            .execution
            .planned_only_pass_ids
            .contains(&"scss-module-evaluate")
    );
    assert!(summary.execution.output_css.contains("color: red"));
    assert!(summary.execution.output_css.contains("._button_0"));
    assert_eq!(
        summary
            .execution
            .css_module_evaluation
            .as_ref()
            .map(|evaluation| evaluation.evaluator.as_str()),
        Some("omena-query-static-scss-variable-evaluator")
    );
}

#[test]
fn consumer_build_derives_static_less_evaluator_context() {
    let summary = execute_omena_query_consumer_build_style_source(
        "Button.module.less",
        "@brand: red; .button { color: @brand; }",
        &[
            "less-module-evaluate".to_string(),
            "css-modules-class-hashing".to_string(),
            "print-css".to_string(),
        ],
    );

    assert!(
        summary
            .execution
            .executed_pass_ids
            .contains(&"less-module-evaluate")
    );
    assert!(
        !summary
            .execution
            .planned_only_pass_ids
            .contains(&"less-module-evaluate")
    );
    assert!(summary.execution.output_css.contains("color: red"));
    assert!(summary.execution.output_css.contains("._button_0"));
    assert_eq!(
        summary
            .execution
            .css_module_evaluation
            .as_ref()
            .map(|evaluation| evaluation.evaluator.as_str()),
        Some("omena-query-static-less-variable-evaluator")
    );
}

#[test]
fn consumer_build_derives_workspace_context_for_import_inline_and_composes() {
    let sources = vec![
        OmenaQueryStyleSourceInputV0 {
            style_path: "Button.module.css".to_string(),
            style_source:
                r#"@import "./tokens.css" supports(display: grid) screen and (min-width: 40rem); .button { composes: base; color: var(--brand); } .base { color: blue; }"#
                    .to_string(),
        },
        OmenaQueryStyleSourceInputV0 {
            style_path: "tokens.css".to_string(),
            style_source: ":root { --brand: red; }".to_string(),
        },
    ];
    let pass_ids = vec![
        "import-inline".to_string(),
        "composes-resolution".to_string(),
    ];
    let summary_result = execute_omena_query_consumer_build_style_sources_with_context(
        "Button.module.css",
        &sources,
        &pass_ids,
        &OmenaQueryTransformExecutionContextV0::default(),
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
            .contains(&"import-inline")
    );
    assert!(
        summary
            .execution
            .executed_pass_ids
            .contains(&"composes-resolution")
    );
    assert!(
        !summary
            .execution
            .planned_only_pass_ids
            .contains(&"import-inline")
    );
    assert!(
        !summary
            .execution
            .planned_only_pass_ids
            .contains(&"composes-resolution")
    );
    assert!(summary.execution.output_css.contains("--brand: red"));
    assert!(
        summary
            .execution
            .output_css
            .contains("@media screen and (min-width: 40rem) { @supports (display: grid) { :root { --brand: red; } } }")
    );
    assert!(!summary.execution.output_css.contains("@import"));
    assert!(!summary.execution.output_css.contains("composes:"));
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

#[test]
fn lists_transform_pass_summaries_from_query() {
    let passes = list_omena_query_transform_pass_summaries();

    assert_eq!(passes.len(), 40);
    assert!(passes.iter().any(|pass| pass.id == "whitespace-strip"));
    assert!(passes.iter().any(|pass| pass.id == "print-css"));
}

#[test]
fn style_document_summary_is_omena_parser_owned() {
    let summary = summarize_omena_query_style_document(
        "Component.module.scss",
        r#"
@use "./tokens" as tokens;
@forward "./theme";
.card { --brand: #fff; color: var(--brand); }
"#,
    );
    assert!(summary.is_some());
    let Some(summary) = summary else {
        return;
    };

    assert_eq!(summary.product, "omena-query.style-document-summary");
    assert_eq!(summary.language, "scss");
    assert_eq!(summary.selector_names, vec!["card"]);
    assert_eq!(summary.custom_property_decl_names, vec!["--brand"]);
    assert_eq!(summary.custom_property_ref_names, vec!["--brand"]);
    assert_eq!(summary.sass_module_use_sources, vec!["./tokens"]);
    assert_eq!(summary.sass_module_forward_sources, vec!["./theme"]);
    assert_eq!(summary.diagnostic_count, 0);
}

#[test]
fn reads_style_context_index_from_query_boundary() {
    let summary = super::read_omena_query_style_context_index(
        "Component.module.scss",
        r#"
@layer reset, components;
@layer components {
  @container card (min-width: 20rem) {
    @scope (.card) {
      .card { color: red; }
    }
  }
}
"#,
        &sample_input(),
    );
    assert!(summary.is_some());
    let Some(summary) = summary else {
        return;
    };

    assert_eq!(summary.product, "omena-query.style-context-index");
    assert_eq!(
        summary.context_index_source,
        "omena-semantic.style-context-index"
    );
    assert_eq!(summary.context_index.layer_index.named_layer_count, 2);
    assert_eq!(
        summary.context_index.container_index.named_container_count,
        1
    );
    assert_eq!(summary.context_index.scope_index.scopes.len(), 1);
    assert!(
        summary
            .context_index
            .container_index
            .selector_memberships
            .iter()
            .any(|membership| membership.selector_name == "card")
    );
}

#[test]
fn exposes_omena_parser_sass_symbol_fact_surface() {
    let summary = summarize_omena_query_omena_parser_style_facts(
        "@use \"./tokens\" as tokens; @forward \"./theme\" show tone; @import \"legacy\"; @mixin tone($color) { color: $color; } @function double($x) { @return $x * 2; } .card { @include tone(red); width: double(2px); }",
        omena_parser::StyleDialect::Scss,
    );

    assert_eq!(
        summary.sass_symbol_declaration_names,
        vec!["color", "double", "tone", "x"]
    );
    assert_eq!(
        summary.sass_symbol_reference_names,
        vec!["color", "double", "tone", "x"]
    );
    assert!(summary.sass_symbol_facts.iter().any(|fact| {
        fact.kind == "sassMixinDeclaration" && fact.name == "tone" && fact.role == "declaration"
    }));
    assert!(summary.sass_symbol_facts.iter().any(|fact| {
        fact.kind == "sassMixinInclude" && fact.name == "tone" && fact.role == "include"
    }));
    assert!(summary.sass_symbol_facts.iter().any(|fact| {
        fact.kind == "sassFunctionDeclaration"
            && fact.name == "double"
            && fact.role == "declaration"
    }));
    assert!(summary.sass_symbol_facts.iter().any(|fact| {
        fact.kind == "sassFunctionCall" && fact.name == "double" && fact.role == "call"
    }));
    assert_eq!(summary.sass_symbol_resolution.resolution_scope, "same-file");
    assert_eq!(summary.sass_symbol_resolution.declaration_count, 4);
    assert_eq!(summary.sass_symbol_resolution.reference_count, 4);
    assert_eq!(summary.sass_symbol_resolution.resolved_reference_count, 4);
    assert_eq!(summary.sass_symbol_resolution.unresolved_reference_count, 0);
    assert!(
        summary
            .sass_symbol_resolution
            .capabilities
            .same_file_lexical_resolution_ready
    );
    assert!(summary.sass_symbol_resolution.edges.iter().any(|edge| {
        edge.symbol_kind == "mixin"
            && edge.name == "tone"
            && edge.reference_kind == "sassMixinInclude"
            && edge.declaration_kind == Some("sassMixinDeclaration")
            && edge.status == "resolved"
    }));
    assert!(summary.sass_symbol_resolution.edges.iter().any(|edge| {
        edge.symbol_kind == "function"
            && edge.name == "double"
            && edge.reference_kind == "sassFunctionCall"
            && edge.declaration_kind == Some("sassFunctionDeclaration")
            && edge.status == "resolved"
    }));
    assert_eq!(summary.sass_module_use_sources, vec!["./tokens"]);
    assert_eq!(summary.sass_module_forward_sources, vec!["./theme"]);
    assert_eq!(summary.sass_module_import_sources, vec!["legacy"]);
    assert!(summary.sass_module_edges.iter().any(|edge| {
        edge.kind == "sassForward"
            && edge.source == "./theme"
            && edge.visibility_filter_kind == Some("show")
            && edge.visibility_filter_names == vec!["tone"]
    }));
    assert!(summary.sass_module_edges.iter().any(|edge| {
        edge.kind == "sassUse"
            && edge.source == "./tokens"
            && edge.namespace_kind == Some("alias")
            && edge.namespace.as_deref() == Some("tokens")
    }));
}

#[test]
fn exposes_omena_parser_unresolved_sass_symbol_resolution() {
    let summary = summarize_omena_query_omena_parser_style_facts(
        ".card { color: $missing; @include absent; }",
        omena_parser::StyleDialect::Scss,
    );

    assert_eq!(summary.sass_symbol_resolution.declaration_count, 0);
    assert_eq!(summary.sass_symbol_resolution.reference_count, 2);
    assert_eq!(summary.sass_symbol_resolution.resolved_reference_count, 0);
    assert_eq!(summary.sass_symbol_resolution.unresolved_reference_count, 2);
    assert!(
        summary
            .sass_symbol_resolution
            .capabilities
            .unresolved_reference_reporting_ready
    );
}

#[test]
fn exposes_omena_parser_namespaced_sass_symbol_fact_surface() {
    let summary = summarize_omena_query_omena_parser_style_facts(
        r#"@use "./tokens" as tokens; .card { color: tokens.$brand; @include tokens.tone(red); width: tokens.double(2px); }"#,
        omena_parser::StyleDialect::Scss,
    );

    assert_eq!(
        summary.sass_symbol_reference_names,
        vec!["brand", "double", "tone"]
    );
    assert!(summary.sass_symbol_facts.iter().any(|fact| {
        fact.kind == "sassVariableReference"
            && fact.name == "brand"
            && fact.role == "reference"
            && fact.namespace.as_deref() == Some("tokens")
    }));
    assert!(summary.sass_symbol_facts.iter().any(|fact| {
        fact.kind == "sassMixinInclude"
            && fact.name == "tone"
            && fact.role == "include"
            && fact.namespace.as_deref() == Some("tokens")
    }));
    assert!(summary.sass_symbol_facts.iter().any(|fact| {
        fact.kind == "sassFunctionCall"
            && fact.name == "double"
            && fact.role == "call"
            && fact.namespace.as_deref() == Some("tokens")
    }));
    assert_eq!(summary.sass_symbol_resolution.declaration_count, 0);
    assert_eq!(summary.sass_symbol_resolution.reference_count, 3);
    assert_eq!(summary.sass_symbol_resolution.resolved_reference_count, 0);
    assert_eq!(summary.sass_symbol_resolution.unresolved_reference_count, 3);
    assert!(summary.sass_symbol_resolution.edges.iter().all(|edge| {
        edge.namespace.as_deref() == Some("tokens") && edge.status == "unresolved"
    }));
}

#[test]
fn bundles_expression_source_and_selector_query_fragments() {
    let input = sample_input();
    let bundle = summarize_omena_query_fragment_bundle(&input);

    assert_eq!(bundle.schema_version, "0");
    assert_eq!(bundle.product, "omena-query.fragment-bundle");
    assert_eq!(bundle.input_version, "2");
    assert_eq!(bundle.expression_semantics.fragments.len(), 2);
    assert_eq!(bundle.expression_semantics.fragments[0].query_id, "expr-1");
    assert_eq!(bundle.source_resolution.fragments.len(), 2);
    assert_eq!(bundle.source_resolution.fragments[1].query_id, "expr-2");
    assert_eq!(bundle.selector_usage.fragments.len(), 2);
    assert_eq!(bundle.selector_usage.fragments[0].query_id, "btn-active");

    let expression = summarize_omena_query_expression_semantics_query_fragments(&input);
    let source = summarize_omena_query_source_resolution_query_fragments(&input);
    let selector = summarize_omena_query_selector_usage_query_fragments(&input);

    assert_eq!(expression.schema_version, "0");
    assert_eq!(source.schema_version, "0");
    assert_eq!(selector.schema_version, "0");
    assert_eq!(expression.input_version, "2");
    assert_eq!(source.input_version, "2");
    assert_eq!(selector.input_version, "2");
    assert_eq!(
        expression.fragments.len(),
        bundle.expression_semantics.fragments.len()
    );
    assert_eq!(
        source.fragments.len(),
        bundle.source_resolution.fragments.len()
    );
    assert_eq!(
        selector.fragments.len(),
        bundle.selector_usage.fragments.len()
    );
}

#[test]
fn derives_transform_context_from_workspace_sources() {
    let summary = summarize_omena_query_transform_context_from_sources(
        "Button.module.css",
        [
            (
                "Button.module.css",
                r#"@import "./tokens.css"; .button { composes: base; color: var(--brand); } .base { color: blue; }"#,
            ),
            ("tokens.css", r#":root { --brand: red; }"#),
        ],
        &[],
    );

    assert_eq!(summary.product, "omena-query.transform-context");
    assert_eq!(summary.target_style_path, "Button.module.css");
    assert_eq!(summary.style_count, 2);
    assert_eq!(summary.import_inline_count, 1);
    assert_eq!(summary.class_name_rewrite_count, 2);
    assert_eq!(summary.css_module_composes_resolution_count, 1);
    assert_eq!(summary.css_module_value_resolution_count, 0);
    assert_eq!(summary.design_token_route_count, 1);
    assert_eq!(summary.reachable_class_name_count, 0);
    assert_eq!(summary.reachable_keyframe_name_count, 0);
    assert_eq!(summary.reachable_value_name_count, 0);
    assert_eq!(summary.reachable_custom_property_name_count, 0);
    assert!(!summary.context.closed_style_world);
    assert_eq!(summary.context.reachable_class_names, Vec::<String>::new());
    assert_eq!(
        summary.context.reachable_custom_property_names,
        Vec::<String>::new()
    );
    assert_eq!(
        summary.context.import_inlines[0].import_source,
        "./tokens.css"
    );
    assert_eq!(summary.context.design_token_routes[0].token_name, "--brand");
    assert_eq!(summary.context.design_token_routes[0].routed_value, "red");
    assert!(summary.ready_surfaces.contains(&"designTokenRouteProducer"));
    assert_eq!(
        summary.context.import_inlines[0].replacement_css,
        ":root { --brand: red; }"
    );
    assert_eq!(
        summary
            .context
            .class_name_rewrites
            .iter()
            .map(|rewrite| rewrite.original_name.as_str())
            .collect::<Vec<_>>(),
        vec!["button", "base"]
    );
    assert_eq!(
        summary.context.css_module_composes_resolutions[0].exported_class_names,
        vec!["base", "button"]
    );
    assert!(summary.ready_surfaces.contains(&"transformContextProducer"));
}

#[test]
fn explicit_context_extends_query_derived_transform_context()
-> Result<(), Box<dyn std::error::Error>> {
    let sources = vec![
        OmenaQueryStyleSourceInputV0 {
            style_path: "Button.module.css".to_string(),
            style_source: r#"@import "./tokens.css"; .button { color: var(--brand); background: var(--external); }"#.to_string(),
        },
        OmenaQueryStyleSourceInputV0 {
            style_path: "tokens.css".to_string(),
            style_source: ":root { --brand: red; }".to_string(),
        },
    ];
    let context = OmenaQueryTransformExecutionContextV0 {
        design_token_routes: vec![omena_transform_passes::TransformDesignTokenRouteV0 {
            token_name: "--external".to_string(),
            routed_value: "blue".to_string(),
        }],
        ..OmenaQueryTransformExecutionContextV0::default()
    };
    let summary = execute_omena_query_consumer_build_style_sources_with_context(
        "Button.module.css",
        &sources,
        &[
            "import-inline".to_string(),
            "design-token-routing".to_string(),
            "print-css".to_string(),
        ],
        &context,
        &[],
    )?;

    assert_eq!(summary.product, "omena-query.consumer-build-style-source");
    assert_eq!(summary.execution.design_token_routes.len(), 2);
    assert!(
        summary
            .execution
            .design_token_routes
            .iter()
            .any(|route| route.token_name == "--brand" && route.routed_value == "red")
    );
    assert!(
        summary
            .execution
            .design_token_routes
            .iter()
            .any(|route| route.token_name == "--external" && route.routed_value == "blue")
    );
    assert!(!summary.execution.output_css.contains("@import"));
    assert!(
        summary
            .execution
            .output_css
            .contains(":root { --brand: red; }")
    );
    assert!(summary.execution.output_css.contains("color: red"));
    assert!(summary.execution.output_css.contains("background: blue"));
    Ok(())
}

#[test]
fn derives_transform_context_with_static_stylesheet_module_evaluation() {
    let scss_summary = summarize_omena_query_transform_context_from_sources(
        "Button.module.scss",
        [(
            "Button.module.scss",
            "$brand: red; $accent: $brand; .button { color: $accent; }",
        )],
        &[],
    );
    let less_summary = summarize_omena_query_transform_context_from_sources(
        "Button.module.less",
        [(
            "Button.module.less",
            "@brand: red; @accent: @brand; .button { color: @accent; }",
        )],
        &[],
    );

    assert_eq!(
        scss_summary
            .context
            .scss_module_evaluation
            .as_ref()
            .map(|evaluation| evaluation.evaluated_css.as_str()),
        Some("  .button { color: red; }")
    );
    assert_eq!(
        less_summary
            .context
            .less_module_evaluation
            .as_ref()
            .map(|evaluation| evaluation.evaluated_css.as_str()),
        Some("  .button { color: red; }")
    );
    assert!(
        scss_summary
            .ready_surfaces
            .contains(&"stylesheetModuleEvaluationProducer")
    );
    assert!(
        less_summary
            .ready_surfaces
            .contains(&"stylesheetModuleEvaluationProducer")
    );

    let declaration_only_summary = summarize_omena_query_transform_context_from_sources(
        "Tokens.module.scss",
        [(
            "Tokens.module.scss",
            "$unused: 1px; .button { color: red; }",
        )],
        &[],
    );
    assert_eq!(
        declaration_only_summary
            .context
            .scss_module_evaluation
            .as_ref()
            .map(|evaluation| evaluation.evaluated_css.as_str()),
        Some(" .button { color: red; }")
    );

    let forward_reference_summary = summarize_omena_query_transform_context_from_sources(
        "Forward.module.scss",
        [(
            "Forward.module.scss",
            "$accent: $brand; $brand: red; .button { color: $accent; }",
        )],
        &[],
    );
    assert!(
        forward_reference_summary
            .context
            .scss_module_evaluation
            .is_none()
    );
}

#[test]
fn derives_transform_context_with_cross_file_value_resolutions() {
    let summary = summarize_omena_query_transform_context_from_sources(
        "/tmp/App.module.css",
        [
            (
                "/tmp/tokens.module.css",
                "@value primary: #fff; @value gap: 8px; @value alias: primary;",
            ),
            (
                "/tmp/App.module.css",
                r#"@value primary as brand, gap, alias from "./tokens.module.css"; .btn { color: brand; margin: gap; border-color: alias; }"#,
            ),
        ],
        &[],
    );

    assert_eq!(summary.product, "omena-query.transform-context");
    assert_eq!(summary.css_module_value_resolution_count, 3);
    assert_eq!(
        summary
            .context
            .css_module_value_resolutions
            .iter()
            .map(|resolution| {
                (
                    resolution.local_name.as_str(),
                    resolution.resolved_value.as_str(),
                )
            })
            .collect::<Vec<_>>(),
        vec![("alias", "#fff"), ("brand", "#fff"), ("gap", "8px")]
    );
    assert!(
        summary
            .ready_surfaces
            .contains(&"cssModuleValueResolutionProducer")
    );
}

#[test]
fn consumer_build_resolves_cross_file_css_modules_values_through_query_context()
-> Result<(), Box<dyn std::error::Error>> {
    let summary = execute_omena_query_consumer_build_style_sources(
        "/tmp/App.module.css",
        &[
            OmenaQueryStyleSourceInputV0 {
                style_path: "/tmp/tokens.module.css".to_string(),
                style_source: "@value primary: #fff; @value gap: 8px;".to_string(),
            },
            OmenaQueryStyleSourceInputV0 {
                style_path: "/tmp/App.module.css".to_string(),
                style_source: r#"@value primary as brand, gap from "./tokens.module.css"; .btn { color: brand; margin: gap; } @media (min-width: gap) { .btn { color: brand; } }"#.to_string(),
            },
        ],
        &["value-resolution".to_string(), "print-css".to_string()],
        &[],
    )?;

    assert_eq!(summary.product, "omena-query.consumer-build-style-source");
    assert_eq!(
        summary.execution.output_css,
        r#" .btn { color: #fff; margin: 8px; } @media (min-width: 8px) { .btn { color: #fff; } }"#
    );
    assert_eq!(summary.execution.mutation_count, 5);
    assert!(
        summary
            .ready_surfaces
            .contains(&"multiSourceTransformContextProducer")
    );
    Ok(())
}

#[test]
fn derives_transform_context_with_cross_file_composes_closure() {
    let summary = summarize_omena_query_transform_context_from_sources(
        "/tmp/App.module.scss",
        [
            (
                "/tmp/base.module.scss",
                ".foundation { display: block; } .base { composes: foundation; color: red; }",
            ),
            (
                "/tmp/App.module.scss",
                r#".btn { composes: base from "./base.module.scss"; color: blue; }"#,
            ),
        ],
        &[],
    );

    assert_eq!(summary.css_module_composes_resolution_count, 1);
    assert_eq!(
        summary.context.css_module_composes_resolutions[0].local_class_name,
        "btn"
    );
    assert_eq!(
        summary.context.css_module_composes_resolutions[0].exported_class_names,
        vec!["base", "btn", "foundation"]
    );
}

#[test]
fn derives_transform_context_from_package_manifest_style_exports() {
    let summary = summarize_omena_query_transform_context_from_sources(
        "/fake/workspace/src/App.module.css",
        [
            (
                "/fake/workspace/src/App.module.css",
                r#"@import "@design/tokens/theme"; .button { color: var(--brand); }"#,
            ),
            (
                "/fake/workspace/node_modules/@design/tokens/dist/theme.css",
                ":root { --brand: package; }",
            ),
        ],
        &[OmenaQueryStylePackageManifestV0 {
            package_json_path: "/fake/workspace/node_modules/@design/tokens/package.json"
                .to_string(),
            package_json_source: r#"{"exports":{"./theme":{"style":"./dist/theme.css"}}}"#
                .to_string(),
        }],
    );

    assert_eq!(summary.import_inline_count, 1);
    assert_eq!(summary.design_token_route_count, 1);
    assert_eq!(
        summary.context.import_inlines[0].import_source,
        "@design/tokens/theme"
    );
    assert_eq!(
        summary.context.import_inlines[0].replacement_css,
        ":root { --brand: package; }"
    );
    assert_eq!(summary.context.design_token_routes[0].token_name, "--brand");
    assert_eq!(
        summary.context.design_token_routes[0].routed_value,
        "package"
    );
}

#[test]
fn declares_runtime_backed_selected_query_adapter_capabilities() {
    let summary = summarize_omena_query_selected_query_adapter_capabilities();

    assert_eq!(summary.schema_version, "0");
    assert_eq!(
        summary.product,
        "omena-query.selected-query-adapter-capabilities"
    );
    assert_eq!(summary.default_candidate_backend, "rust-selected-query");
    assert_eq!(summary.routing_status, "runtimeBacked");

    let unified = backend(&summary, "rust-selected-query");
    assert!(unified.is_some());
    let Some(unified) = unified else {
        return;
    };
    assert!(unified.source_resolution);
    assert!(unified.expression_semantics);
    assert!(unified.selector_usage);
    assert!(unified.style_semantic_graph);

    let source_only = backend(&summary, "rust-source-resolution");
    assert!(source_only.is_some());
    let Some(source_only) = source_only else {
        return;
    };
    assert!(source_only.source_resolution);
    assert!(!source_only.expression_semantics);
    assert!(!source_only.selector_usage);
    assert!(!source_only.style_semantic_graph);

    assert!(
        summary
            .runner_commands
            .iter()
            .any(|command| command.command == "input-omena-query-evaluation-runtime")
    );
    assert!(
        summary
            .runner_commands
            .iter()
            .any(|command| command.command == "input-omena-resolver-source-resolution-runtime")
    );
    assert!(
        summary
            .runner_commands
            .iter()
            .any(|command| command.command == "input-expression-domain-flow-analysis")
    );
    assert!(
        summary
            .runner_commands
            .iter()
            .any(|command| { command.command == "input-expression-domain-control-flow-analysis" })
    );
    assert!(
        summary.runner_commands.iter().any(|command| {
            command.command == "input-expression-domain-incremental-flow-analysis"
        })
    );
    assert!(
        summary
            .runner_commands
            .iter()
            .any(|command| command.command == "input-expression-domain-selector-projection")
    );
    assert!(
        summary
            .runner_commands
            .iter()
            .any(|command| command.command == "style-semantic-graph-batch")
    );
    assert!(
        summary
            .runner_commands
            .iter()
            .any(|command| command.command == "read-cascade-at-position")
    );
    assert!(
        summary
            .runner_commands
            .iter()
            .any(|command| command.command == "transform-plan")
    );
    assert!(summary.runner_commands.iter().any(|command| {
        command.command == "transform-context-from-engine-input"
            && command.output_product == "omena-query.transform-context-from-engine-input"
    }));
    assert!(
        summary
            .expression_semantics_payload_contracts
            .contains(&"valueDomainDerivation")
    );
    assert!(
        summary
            .expression_semantics_payload_contracts
            .contains(&"valueDomainProvenanceTree")
    );
    assert!(summary.adapter_readiness.contains(&"runnerCommandContract"));
    assert!(
        summary
            .adapter_readiness
            .contains(&"canonicalProducerWrapperBoundary")
    );
    assert!(
        summary
            .adapter_readiness
            .contains(&"styleSemanticGraphBridgeBoundary")
    );
    assert!(
        summary
            .adapter_readiness
            .contains(&"expressionDomainFlowAnalysisRunner")
    );
    assert!(
        summary
            .adapter_readiness
            .contains(&"expressionDomainControlFlowAnalysisRunner")
    );
    assert!(
        summary
            .adapter_readiness
            .contains(&"expressionDomainSalsaRuntime")
    );
    assert!(
        summary
            .adapter_readiness
            .contains(&"expressionDomainSelectorProjection")
    );
    assert!(
        summary
            .adapter_readiness
            .contains(&"sourceResolutionRuntimeIndex")
    );
    assert!(summary.adapter_readiness.contains(&"readCascadeAtPosition"));
    assert!(summary.adapter_readiness.contains(&"transformPlanRunner"));
    assert!(
        summary
            .adapter_readiness
            .contains(&"transformEggExecutionWitnesses")
    );
    assert!(
        summary
            .adapter_readiness
            .contains(&"semanticReachabilityTransformContext")
    );
    assert!(
        summary
            .adapter_readiness
            .contains(&"queryEvaluationRuntime")
    );
}

#[test]
fn summarizes_query_evaluation_runtime_without_legacy_parser_coupling() {
    let input = sample_input();
    let mut runtime = OmenaQueryExpressionDomainFlowRuntimeV0::default();

    let first = summarize_omena_query_evaluation_runtime(&input, &mut runtime);
    assert_eq!(first.schema_version, "0");
    assert_eq!(first.product, "omena-query.evaluation-runtime");
    assert_eq!(first.input_version, "2");
    assert_eq!(
        first.selected_query_adapter_capabilities.routing_status,
        "runtimeBacked"
    );
    assert!(
        first
            .runtime_products
            .contains(&"omena-resolver.source-resolution-runtime-index")
    );
    assert!(
        first
            .runtime_products
            .contains(&"omena-query.expression-domain-incremental-flow-analysis")
    );
    assert!(
        first
            .runtime_products
            .contains(&"omena-query.style-document-summary")
    );
    assert_eq!(first.source_resolution_expression_count, 2);
    assert_eq!(first.source_resolution_unresolved_expression_count, 0);
    assert_eq!(first.expression_domain_revision, 1);
    assert_eq!(first.expression_domain_graph_count, 2);
    assert_eq!(first.expression_domain_dirty_graph_count, 2);
    assert_eq!(first.expression_domain_reused_graph_count, 0);
    assert_eq!(
        first.style_document_summary_source,
        "omena-parser.style-facts"
    );
    assert!(
        first
            .ready_surfaces
            .contains(&"selectedQueryBackendAdapter")
    );
    assert!(
        first
            .ready_surfaces
            .contains(&"sourceResolutionRuntimeIndex")
    );
    assert!(
        first
            .ready_surfaces
            .contains(&"expressionDomainSalsaRuntime")
    );
    assert!(
        first
            .ready_surfaces
            .contains(&"omenaParserStyleDocumentSummary")
    );
    assert!(
        first
            .ready_surfaces
            .contains(&"omenaParserPublicContractTypes")
    );
    assert!(
        first
            .retired_couplings
            .contains(&"engineStyleParserStyleDocumentSummary")
    );
    assert!(
        first
            .retired_couplings
            .contains(&"engineStyleParserQueryPublicTypes")
    );

    let second = summarize_omena_query_evaluation_runtime(&input, &mut runtime);
    assert_eq!(second.expression_domain_revision, 2);
    assert_eq!(second.expression_domain_dirty_graph_count, 0);
    assert_eq!(second.expression_domain_reused_graph_count, 2);
}

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

#[test]
fn owns_selected_query_canonical_producer_wrappers_without_changing_products() {
    let input = sample_input();

    let source = summarize_omena_query_source_resolution_canonical_producer_signal(&input);
    assert_eq!(source.schema_version, "0");
    assert_eq!(source.input_version, "2");
    assert_eq!(source.canonical_bundle.query_fragments.len(), 2);
    assert_eq!(source.evaluator_candidates.results.len(), 2);

    let expression = summarize_omena_query_expression_semantics_canonical_producer_signal(&input);
    assert_eq!(expression.schema_version, "0");
    assert_eq!(expression.input_version, "2");
    assert_eq!(expression.canonical_bundle.query_fragments.len(), 2);
    assert_eq!(expression.evaluator_candidates.results.len(), 2);
    assert_eq!(
        expression.evaluator_candidates.results[0]
            .payload
            .value_domain_derivation
            .product,
        "omena-abstract-value.reduced-class-value-derivation"
    );
    assert_eq!(
        expression.evaluator_candidates.results[0]
            .payload
            .value_domain_derivation
            .reduced_kind,
        "prefixSuffix"
    );
    assert_eq!(
        expression.evaluator_candidates.results[0]
            .payload
            .value_domain_provenance_tree
            .root
            .operation,
        "constraintDomain"
    );

    let selector = summarize_omena_query_selector_usage_canonical_producer_signal(&input);
    assert_eq!(selector.schema_version, "0");
    assert_eq!(selector.input_version, "2");
    assert_eq!(selector.canonical_bundle.query_fragments.len(), 2);
    assert_eq!(selector.evaluator_candidates.results.len(), 2);
}

#[test]
fn owns_source_resolution_runtime_index_wrapper() {
    let input = sample_input();
    let runtime_index = summarize_omena_query_source_resolution_runtime(&input);

    assert_eq!(
        runtime_index.product,
        "omena-resolver.source-resolution-runtime-index"
    );
    assert_eq!(runtime_index.expression_count, 2);
    assert_eq!(runtime_index.resolved_expression_count, 2);
    assert_eq!(runtime_index.unresolved_expression_count, 0);
    assert!(
        runtime_index
            .entries
            .iter()
            .any(|entry| entry.expression_id == "expr-1" && entry.selector_names == ["btn-active"])
    );
}

#[test]
fn owns_style_semantic_graph_adapter_boundary_without_changing_graph_product() {
    let input = sample_input();
    let graph = summarize_omena_query_style_semantic_graph_from_source(
        "/tmp/App.module.scss",
        ".btn-active { color: red; }",
        &input,
    );
    assert!(graph.is_some());
    let Some(graph) = graph else {
        return;
    };
    assert_eq!(graph.schema_version, "0");
    assert_eq!(graph.product, "omena-semantic.style-semantic-graph");
    assert_eq!(graph.selector_identity_engine.canonical_ids.len(), 1);

    let batch = summarize_omena_query_style_semantic_graph_batch_from_sources(
        [
            ("/tmp/App.module.scss", ".btn-active { color: red; }"),
            ("/tmp/Card.module.scss", ".card-header { color: blue; }"),
        ],
        &input,
    );
    assert_eq!(batch.schema_version, "0");
    assert_eq!(batch.product, "omena-semantic.style-semantic-graph-batch");
    assert_eq!(batch.graphs.len(), 2);
    assert_eq!(batch.graphs[0].style_path, "/tmp/App.module.scss");
    assert!(batch.graphs[0].graph.is_some());
    assert!(batch.graphs[1].graph.is_some());
}

#[test]
fn style_semantic_graph_adapter_exposes_css_modules_semantic_seed() {
    let input = sample_input();
    let graph = summarize_omena_query_style_semantic_graph_from_source(
        "/tmp/App.module.scss",
        "@value primary: #fff; @value accent: primary; :export { primary: #fff; } .btn { composes: base from \"./base.module.scss\"; }",
        &input,
    );
    assert!(graph.is_some());
    let Some(graph) = graph else {
        return;
    };

    assert_eq!(
        graph.css_modules_semantics.product,
        "omena-semantic.css-modules-semantics"
    );
    assert_eq!(graph.css_modules_semantics.status, "parserFactSeed");
    assert_eq!(graph.css_modules_semantics.class_export_names, vec!["btn"]);
    assert_eq!(
        graph.css_modules_semantics.composes_target_names,
        vec!["base"]
    );
    assert_eq!(
        graph.css_modules_semantics.composes_import_sources,
        vec!["./base.module.scss"]
    );
    assert_eq!(
        graph.css_modules_semantics.value_definition_names,
        vec!["accent", "primary"]
    );
    assert_eq!(
        graph.css_modules_semantics.value_reference_names,
        vec!["primary"]
    );
    assert_eq!(
        graph.css_modules_semantics.icss_export_names,
        vec!["primary"]
    );
    assert!(
        graph
            .css_modules_semantics
            .capabilities
            .per_file_symbol_summary_ready
    );
    assert!(
        !graph
            .css_modules_semantics
            .capabilities
            .cross_file_resolution_ready
    );
}

#[test]
fn style_semantic_graph_batch_feeds_workspace_design_token_candidates() {
    let input = sample_input();
    let batch = summarize_omena_query_style_semantic_graph_batch_from_sources(
        [
            ("/tmp/tokens.module.scss", ":root { --brand: red; }"),
            ("/tmp/theme.module.scss", "@forward \"./tokens\";"),
            ("/tmp/unrelated.module.scss", ":root { --brand: blue; }"),
            (
                "/tmp/App.module.scss",
                "@use \"./theme\";\n.button { color: var(--brand); }",
            ),
        ],
        &input,
    );

    let app_graph = batch
        .graphs
        .iter()
        .find(|entry| entry.style_path == "/tmp/App.module.scss")
        .and_then(|entry| entry.graph.as_ref());
    assert!(app_graph.is_some());
    let Some(app_graph) = app_graph else {
        return;
    };
    let design_tokens = &app_graph.design_token_semantics;

    assert_eq!(
        design_tokens.status,
        "cross-file-import-cascade-ranking-seed"
    );
    assert_eq!(
        design_tokens.resolution_scope,
        "cross-file-import-candidate"
    );
    assert!(
        design_tokens
            .capabilities
            .workspace_cascade_candidate_signal_ready
    );
    assert!(design_tokens.capabilities.cross_file_import_graph_ready);
    assert_eq!(
        design_tokens
            .resolution_signal
            .cross_file_declaration_fact_count,
        1
    );
    assert_eq!(
        design_tokens
            .resolution_signal
            .workspace_occurrence_resolved_reference_count,
        1
    );
    assert_eq!(
        design_tokens
            .cascade_ranking_signal
            .cross_file_candidate_declaration_count,
        1
    );
    assert_eq!(
        design_tokens
            .cascade_ranking_signal
            .cross_file_winner_declaration_count,
        1
    );
    assert_eq!(
        design_tokens.cascade_ranking_signal.ranked_references[0]
            .winner_declaration_file_path
            .as_deref(),
        Some("/tmp/tokens.module.scss")
    );
    let winner_range =
        design_tokens.cascade_ranking_signal.ranked_references[0].winner_declaration_range;
    assert_eq!(winner_range.map(|range| range.start.line), Some(0));
    assert_eq!(winner_range.map(|range| range.start.character), Some(8));
    assert_eq!(design_tokens.declaration_candidates.len(), 1);
    let declaration_candidate = &design_tokens.declaration_candidates[0];
    assert_eq!(declaration_candidate.name, "--brand");
    assert_eq!(declaration_candidate.file_path, "/tmp/tokens.module.scss");
    assert_eq!(
        declaration_candidate.candidate_scope,
        "cross-file-import-candidate"
    );
    assert!(declaration_candidate.import_graph_distance.is_some());
    assert_eq!(
        design_tokens.cascade_ranking_signal.ranked_references[0]
            .cross_file_candidate_declaration_count,
        1
    );
}

#[test]
fn style_semantic_graph_batch_prefers_nearer_import_graph_token_candidates() {
    let input = sample_input();
    let batch = summarize_omena_query_style_semantic_graph_batch_from_sources(
        [
            ("/tmp/a-direct.module.scss", ":root { --brand: direct; }"),
            ("/tmp/mid.module.scss", "@forward \"./z-transitive\";"),
            (
                "/tmp/z-transitive.module.scss",
                ":root { --brand: transitive; }",
            ),
            (
                "/tmp/App.module.scss",
                "@use \"./a-direct\";\n@use \"./mid\";\n.button { color: var(--brand); }",
            ),
        ],
        &input,
    );

    let app_graph = batch
        .graphs
        .iter()
        .find(|entry| entry.style_path == "/tmp/App.module.scss")
        .and_then(|entry| entry.graph.as_ref());
    assert!(app_graph.is_some());
    let Some(app_graph) = app_graph else {
        return;
    };
    let ranked_reference = &app_graph
        .design_token_semantics
        .cascade_ranking_signal
        .ranked_references[0];

    assert_eq!(
        ranked_reference.winner_declaration_file_path.as_deref(),
        Some("/tmp/a-direct.module.scss")
    );
    assert_eq!(ranked_reference.winner_import_graph_distance, Some(1));
    assert_eq!(ranked_reference.winner_import_graph_order, Some(0));
    assert_eq!(ranked_reference.cross_file_candidate_declaration_count, 2);
    assert_eq!(ranked_reference.cross_file_shadowed_declaration_count, 1);
}

#[test]
fn style_semantic_graph_batch_resolves_package_root_forward_chain_token_candidates() {
    let input = sample_input();
    let batch = summarize_omena_query_style_semantic_graph_batch_from_sources(
        [
            (
                "/fake/workspace/node_modules/@design/tokens/src/index.scss",
                "@forward \"./colors\";",
            ),
            (
                "/fake/workspace/node_modules/@design/tokens/src/_colors.scss",
                ":root { --brand: package; }",
            ),
            (
                "/fake/workspace/src/_utils.scss",
                "@forward \"@design/tokens\" as ds_*;",
            ),
            (
                "/fake/workspace/src/App.module.scss",
                "@use \"./utils\";\n.button { color: var(--brand); }",
            ),
        ],
        &input,
    );

    let app_graph = batch
        .graphs
        .iter()
        .find(|entry| entry.style_path == "/fake/workspace/src/App.module.scss")
        .and_then(|entry| entry.graph.as_ref());
    assert!(app_graph.is_some());
    let Some(app_graph) = app_graph else {
        return;
    };
    let ranked_reference = &app_graph
        .design_token_semantics
        .cascade_ranking_signal
        .ranked_references[0];

    assert_eq!(
        ranked_reference.winner_declaration_file_path.as_deref(),
        Some("/fake/workspace/node_modules/@design/tokens/src/_colors.scss")
    );
    assert_eq!(ranked_reference.winner_import_graph_distance, Some(3));
    assert_eq!(ranked_reference.cross_file_candidate_declaration_count, 1);
    assert_eq!(
        batch.sass_module_resolution.product,
        "omena-query.sass-module-cross-file-resolution"
    );
    assert_eq!(batch.sass_module_resolution.module_edge_count, 3);
    assert_eq!(batch.sass_module_resolution.resolved_module_edge_count, 3);
    assert_eq!(batch.sass_module_resolution.unresolved_module_edge_count, 0);
    assert!(
        batch
            .sass_module_resolution
            .capabilities
            .omena_parser_module_edge_consumption_ready
    );
    assert!(batch.sass_module_resolution.edges.iter().any(|edge| {
        edge.from_style_path == "/fake/workspace/src/_utils.scss"
            && edge.edge_kind == "sassForward"
            && edge.source == "@design/tokens"
            && edge.resolved_style_path.as_deref()
                == Some("/fake/workspace/node_modules/@design/tokens/src/index.scss")
            && edge.status == "resolved"
    }));
}

#[test]
fn style_semantic_graph_batch_resolves_sass_module_graph_closure_and_filters() {
    let input = sample_input();
    let batch = summarize_omena_query_style_semantic_graph_batch_from_sources(
        [
            ("/tmp/_palette.scss", "$brand: red; @mixin tone {}"),
            (
                "/tmp/_tokens.scss",
                "@forward \"./palette\" show $brand, tone;",
            ),
            (
                "/tmp/App.module.scss",
                "@use \"./tokens\" as tokens; .button { color: tokens.$brand; }",
            ),
        ],
        &input,
    );
    let resolution = &batch.sass_module_resolution;

    assert_eq!(resolution.status, "moduleGraphClosureResolved");
    assert_eq!(resolution.module_edge_count, 2);
    assert_eq!(resolution.resolved_module_edge_count, 2);
    assert_eq!(resolution.unresolved_module_edge_count, 0);
    assert_eq!(resolution.graph_closure_edge_count, 3);
    assert_eq!(resolution.cycle_count, 0);
    assert_eq!(resolution.visibility_filter_count, 1);
    assert!(resolution.capabilities.graph_closure_ready);
    assert!(resolution.capabilities.cycle_detection_ready);
    assert!(resolution.capabilities.namespace_show_hide_filter_ready);
    assert!(resolution.next_priorities.is_empty());
    assert!(resolution.edges.iter().any(|edge| {
        edge.from_style_path == "/tmp/_tokens.scss"
            && edge.edge_kind == "sassForward"
            && edge.source == "./palette"
            && edge.visibility_filter_kind == Some("show")
            && edge.visibility_filter_names == vec!["brand", "tone"]
            && edge.resolved_style_path.as_deref() == Some("/tmp/_palette.scss")
    }));
    assert!(resolution.graph_closure_edges.iter().any(|edge| {
        edge.from_style_path == "/tmp/App.module.scss"
            && edge.target_style_path == "/tmp/_palette.scss"
            && edge.depth == 2
            && edge.path
                == vec![
                    "/tmp/App.module.scss".to_string(),
                    "/tmp/_tokens.scss".to_string(),
                    "/tmp/_palette.scss".to_string(),
                ]
    }));
}

#[test]
fn style_semantic_graph_batch_detects_sass_module_cycles() {
    let input = sample_input();
    let batch = summarize_omena_query_style_semantic_graph_batch_from_sources(
        [
            ("/tmp/_a.scss", "@use \"./b\";"),
            ("/tmp/_b.scss", "@use \"./a\";"),
        ],
        &input,
    );
    let resolution = &batch.sass_module_resolution;

    assert_eq!(resolution.module_edge_count, 2);
    assert_eq!(resolution.resolved_module_edge_count, 2);
    assert_eq!(resolution.cycle_count, 2);
    assert!(resolution.cycles.iter().any(|cycle| {
        cycle.path
            == vec![
                "/tmp/_a.scss".to_string(),
                "/tmp/_b.scss".to_string(),
                "/tmp/_a.scss".to_string(),
            ]
    }));
    assert!(resolution.capabilities.cycle_detection_ready);
}

#[test]
fn style_semantic_graph_batch_resolves_css_modules_import_seed_edges() {
    let input = sample_input();
    let batch = summarize_omena_query_style_semantic_graph_batch_from_sources(
        [
            (
                "/tmp/base.module.scss",
                ".foundation { display: block; } .base { composes: foundation; color: red; }",
            ),
            (
                "/tmp/tokens.module.scss",
                "@value primary: red; :export { raw: red; exported: raw; }",
            ),
            (
                "/tmp/App.module.scss",
                "@value primary as localPrimary from \"./tokens.module.scss\"; @value accent: localPrimary; :import(\"./tokens.module.scss\") { imported: exported; } :export { forwarded: imported; } .btn { composes: base from \"./base.module.scss\"; color: accent; }",
            ),
        ],
        &input,
    );

    assert_eq!(
        batch.css_modules_resolution.product,
        "omena-query.css-modules-cross-file-resolution"
    );
    assert_eq!(
        batch.css_modules_resolution.status,
        "icssExportImportClosureSeed"
    );
    assert_eq!(batch.css_modules_resolution.import_edge_count, 3);
    assert_eq!(batch.css_modules_resolution.resolved_import_edge_count, 3);
    assert_eq!(batch.css_modules_resolution.unresolved_import_edge_count, 0);
    assert_eq!(batch.css_modules_resolution.matched_name_count, 3);
    assert_eq!(batch.css_modules_resolution.composes_closure_edge_count, 3);
    assert_eq!(batch.css_modules_resolution.value_closure_edge_count, 3);
    assert_eq!(batch.css_modules_resolution.icss_closure_edge_count, 6);
    assert_eq!(batch.css_modules_resolution.composes_cycle_count, 0);
    assert_eq!(batch.css_modules_resolution.value_cycle_count, 0);
    assert_eq!(batch.css_modules_resolution.icss_cycle_count, 0);

    let composes = batch
        .css_modules_resolution
        .edges
        .iter()
        .find(|edge| edge.import_kind == "composes");
    assert!(composes.is_some());
    let Some(composes) = composes else {
        return;
    };
    assert_eq!(composes.status, "resolved");
    assert_eq!(
        composes.resolved_style_path.as_deref(),
        Some("/tmp/base.module.scss")
    );
    assert_eq!(composes.imported_names, vec!["base"]);
    assert_eq!(composes.exported_names, vec!["foundation", "base"]);
    assert_eq!(composes.matched_names, vec!["base"]);
    let transitive_composes = batch
        .css_modules_resolution
        .composes_closure_edges
        .iter()
        .find(|edge| {
            edge.owner_selector_name == "btn" && edge.target_selector_name == "foundation"
        });
    assert!(transitive_composes.is_some());
    let Some(transitive_composes) = transitive_composes else {
        return;
    };
    assert_eq!(transitive_composes.depth, 2);
    assert_eq!(
        transitive_composes.path,
        vec![
            "/tmp/App.module.scss#btn",
            "/tmp/base.module.scss#base",
            "/tmp/base.module.scss#foundation"
        ]
    );

    let transitive_value = batch
        .css_modules_resolution
        .value_closure_edges
        .iter()
        .find(|edge| edge.value_name == "accent" && edge.target_value_name == "primary");
    assert!(transitive_value.is_some());
    let Some(transitive_value) = transitive_value else {
        return;
    };
    assert_eq!(transitive_value.depth, 2);
    assert_eq!(
        transitive_value.path,
        vec![
            "/tmp/App.module.scss#accent",
            "/tmp/App.module.scss#localPrimary",
            "/tmp/tokens.module.scss#primary"
        ]
    );

    let value = batch
        .css_modules_resolution
        .edges
        .iter()
        .find(|edge| edge.import_kind == "value");
    assert!(value.is_some());
    let Some(value) = value else {
        return;
    };
    assert_eq!(value.status, "resolved");
    assert_eq!(
        value.resolved_style_path.as_deref(),
        Some("/tmp/tokens.module.scss")
    );
    assert_eq!(value.imported_names, vec!["primary"]);
    assert_eq!(value.exported_names, vec!["primary"]);
    assert_eq!(value.matched_names, vec!["primary"]);

    let icss = batch
        .css_modules_resolution
        .edges
        .iter()
        .find(|edge| edge.import_kind == "icss");
    assert!(icss.is_some());
    let Some(icss) = icss else {
        return;
    };
    assert_eq!(icss.status, "resolved");
    assert_eq!(icss.imported_names, vec!["exported"]);
    assert_eq!(icss.exported_names, vec!["exported", "raw"]);
    assert_eq!(icss.matched_names, vec!["exported"]);
    let transitive_icss = batch
        .css_modules_resolution
        .icss_closure_edges
        .iter()
        .find(|edge| edge.name == "forwarded" && edge.target_name == "raw");
    assert!(transitive_icss.is_some());
    let Some(transitive_icss) = transitive_icss else {
        return;
    };
    assert_eq!(transitive_icss.depth, 3);
    assert_eq!(
        transitive_icss.path,
        vec![
            "/tmp/App.module.scss#forwarded",
            "/tmp/App.module.scss#imported",
            "/tmp/tokens.module.scss#exported",
            "/tmp/tokens.module.scss#raw"
        ]
    );
    assert!(
        batch
            .css_modules_resolution
            .capabilities
            .transitive_closure_ready
    );
    assert!(
        batch
            .css_modules_resolution
            .capabilities
            .value_graph_closure_ready
    );
    assert!(
        batch
            .css_modules_resolution
            .capabilities
            .icss_export_import_closure_ready
    );
    assert!(
        batch
            .css_modules_resolution
            .capabilities
            .cycle_detection_ready
    );
}

#[test]
fn style_semantic_graph_batch_detects_css_modules_composes_cycles() {
    let input = sample_input();
    let batch = summarize_omena_query_style_semantic_graph_batch_from_sources(
        [(
            "/tmp/cycle.module.scss",
            ".a { composes: b; } .b { composes: a; }",
        )],
        &input,
    );

    assert_eq!(batch.css_modules_resolution.import_edge_count, 0);
    assert_eq!(batch.css_modules_resolution.composes_cycle_count, 1);
    assert_eq!(batch.css_modules_resolution.value_cycle_count, 0);
    assert_eq!(
        batch.css_modules_resolution.cycles[0].path,
        vec![
            "/tmp/cycle.module.scss#a",
            "/tmp/cycle.module.scss#b",
            "/tmp/cycle.module.scss#a"
        ]
    );
}

#[test]
fn style_semantic_graph_batch_detects_css_modules_value_cycles() {
    let input = sample_input();
    let batch = summarize_omena_query_style_semantic_graph_batch_from_sources(
        [("/tmp/value-cycle.module.scss", "@value a: b; @value b: a;")],
        &input,
    );

    assert_eq!(batch.css_modules_resolution.value_cycle_count, 1);
    assert_eq!(
        batch.css_modules_resolution.cycles[0].path,
        vec![
            "/tmp/value-cycle.module.scss#a",
            "/tmp/value-cycle.module.scss#b",
            "/tmp/value-cycle.module.scss#a"
        ]
    );
}

#[test]
fn style_semantic_graph_batch_detects_css_modules_icss_cycles() {
    let input = sample_input();
    let batch = summarize_omena_query_style_semantic_graph_batch_from_sources(
        [("/tmp/icss-cycle.module.scss", ":export { a: b; b: a; }")],
        &input,
    );

    assert_eq!(batch.css_modules_resolution.icss_cycle_count, 1);
    assert_eq!(
        batch.css_modules_resolution.cycles[0].path,
        vec![
            "/tmp/icss-cycle.module.scss#a",
            "/tmp/icss-cycle.module.scss#b",
            "/tmp/icss-cycle.module.scss#a"
        ]
    );
}

#[test]
fn style_semantic_graph_batch_resolves_package_manifest_style_exports() {
    let input = sample_input();
    let batch =
        summarize_omena_query_style_semantic_graph_batch_from_sources_with_package_manifests(
            [
                (
                    "/fake/workspace/node_modules/@design/tokens/dist/theme.css",
                    ":root { --brand: package; }",
                ),
                (
                    "/fake/workspace/src/App.module.scss",
                    "@use \"@design/tokens/theme\";\n.button { color: var(--brand); }",
                ),
            ],
            &input,
            &[OmenaQueryStylePackageManifestV0 {
                package_json_path: "/fake/workspace/node_modules/@design/tokens/package.json"
                    .to_string(),
                package_json_source: r#"{"exports":{"./theme":{"style":"./dist/theme.css"}}}"#
                    .to_string(),
            }],
        );

    let app_graph = batch
        .graphs
        .iter()
        .find(|entry| entry.style_path == "/fake/workspace/src/App.module.scss")
        .and_then(|entry| entry.graph.as_ref());
    assert!(app_graph.is_some());
    let Some(app_graph) = app_graph else {
        return;
    };
    let ranked_reference = &app_graph
        .design_token_semantics
        .cascade_ranking_signal
        .ranked_references[0];

    assert_eq!(
        ranked_reference.winner_declaration_file_path.as_deref(),
        Some("/fake/workspace/node_modules/@design/tokens/dist/theme.css")
    );
    assert_eq!(ranked_reference.winner_import_graph_distance, Some(1));
    assert_eq!(ranked_reference.cross_file_candidate_declaration_count, 1);
    let declaration_candidate = &app_graph.design_token_semantics.declaration_candidates[0];
    assert_eq!(declaration_candidate.name, "--brand");
    assert_eq!(
        declaration_candidate.file_path,
        "/fake/workspace/node_modules/@design/tokens/dist/theme.css"
    );
    assert_eq!(
        declaration_candidate.candidate_scope,
        "cross-file-import-candidate"
    );
    assert_eq!(declaration_candidate.import_graph_distance, Some(1));
}

#[test]
fn style_hover_candidates_are_query_owned() {
    let candidates = super::summarize_omena_query_style_hover_candidates(
        "Component.module.scss",
        r#"
@mixin variants($prefix, $map) {
  @each $name, $value in $map {
.#{$prefix}-#{$name} { color: $value; }
  }
}

@use "./tokens" as tokens;
$accent: red;
.button { color: var(--brand); }
:root { --brand: blue; }
@include variants($prefix: "tone", $map: ("warm": red));
.alert { color: tokens.$brand; @include tokens.tone(red); width: tokens.double(2px); }
"#,
    );
    assert!(candidates.is_some());
    let Some(candidates) = candidates else {
        return;
    };

    assert_eq!(candidates.product, "omena-query.style-hover-candidates");
    assert!(
        candidates
            .candidates
            .iter()
            .any(|candidate| candidate.kind == "selector" && candidate.name == "button")
    );
    assert!(candidates.candidates.iter().any(|candidate| {
        candidate.kind == "customPropertyReference" && candidate.name == "--brand"
    }));
    assert!(candidates.candidates.iter().any(|candidate| {
        candidate.kind == "sassVariableDeclaration" && candidate.name == "accent"
    }));
    assert!(candidates.candidates.iter().any(|candidate| {
        candidate.kind == "sassVariableReference"
            && candidate.name == "brand"
            && candidate.namespace.as_deref() == Some("tokens")
    }));
    assert!(candidates.candidates.iter().any(|candidate| {
        candidate.kind == "sassMixinInclude"
            && candidate.name == "tone"
            && candidate.namespace.as_deref() == Some("tokens")
    }));
    assert!(candidates.candidates.iter().any(|candidate| {
        candidate.kind == "sassFunctionCall"
            && candidate.name == "double"
            && candidate.namespace.as_deref() == Some("tokens")
    }));
    assert!(
        candidates
            .candidates
            .iter()
            .any(
                |candidate| candidate.source == "sassPartialEvaluatorGeneratedSelectors"
                    && candidate.name == "tone-warm"
            )
    );
}

#[test]
fn style_hover_render_parts_are_query_owned() {
    let source = r#"$accent: red !default;
@mixin tone($color) {
  color: $color;
}
.button { color: var(--brand); }
"#;

    let variable = super::summarize_omena_query_style_hover_render_parts(
        source,
        "sassVariableDeclaration",
        "accent",
        ParserPositionV0 {
            line: 0,
            character: 1,
        },
    );
    assert_eq!(variable.product, "omena-query.style-hover-render-parts");
    assert_eq!(variable.value.as_deref(), Some("red"));
    assert_eq!(variable.snippet, "$accent: red !default;");

    let mixin = super::summarize_omena_query_style_hover_render_parts(
        source,
        "sassMixinDeclaration",
        "tone",
        ParserPositionV0 {
            line: 1,
            character: 7,
        },
    );
    assert_eq!(mixin.signature.as_deref(), Some("@mixin tone($color)"));
    assert_eq!(mixin.snippet, "color: $color;");
    assert_eq!(mixin.render_source, "callableBlockSnippet");

    let selector = super::summarize_omena_query_style_hover_render_parts(
        source,
        "selector",
        "button",
        ParserPositionV0 {
            line: 4,
            character: 1,
        },
    );
    assert_eq!(selector.snippet, ".button { color: var(--brand); }");
    assert_eq!(selector.render_source, "ruleSnippet");
}

#[test]
fn missing_custom_property_diagnostics_are_query_owned() {
    let source = ":root { --brand: red; }\n.alert { color: var(--missing); }";
    let candidates =
        super::summarize_omena_query_style_hover_candidates("Component.module.scss", source);
    assert!(candidates.is_some());
    let Some(candidates) = candidates else {
        return;
    };

    let diagnostics = super::summarize_omena_query_missing_custom_property_diagnostics(
        "file:///workspace/src/Component.module.scss",
        source,
        candidates.candidates.as_slice(),
    );

    assert_eq!(diagnostics.len(), 1);
    let diagnostic = &diagnostics[0];
    assert_eq!(diagnostic.code, "missingCustomProperty");
    assert_eq!(
        diagnostic.message,
        "CSS custom property '--missing' not found in indexed style tokens."
    );
    assert_eq!(
        diagnostic.range,
        ParserRangeV0 {
            start: ParserPositionV0 {
                line: 1,
                character: 20,
            },
            end: ParserPositionV0 {
                line: 1,
                character: 29,
            },
        }
    );
    assert_eq!(
        diagnostic
            .create_custom_property
            .as_ref()
            .map(|action| action.new_text.as_str()),
        Some("\n\n:root {\n  --missing: ;\n}\n")
    );
    assert_eq!(
        diagnostic
            .create_custom_property
            .as_ref()
            .map(|action| action.range),
        Some(ParserRangeV0 {
            start: ParserPositionV0 {
                line: 1,
                character: 33,
            },
            end: ParserPositionV0 {
                line: 1,
                character: 33,
            },
        })
    );

    let summary = super::summarize_omena_query_style_diagnostics_for_file(
        "file:///workspace/src/App.module.scss",
        source,
        candidates.candidates.as_slice(),
    );
    assert_eq!(summary.product, "omena-query.diagnostics-for-file");
    assert_eq!(summary.file_kind, "style");
    assert_eq!(summary.diagnostic_count, 1);
    assert_eq!(summary.diagnostics[0].code, "missingCustomProperty");
    assert!(
        summary
            .ready_surfaces
            .contains(&"missingCustomPropertyDiagnostics")
    );
}

#[test]
fn style_diagnostics_for_file_include_cascade_aware_lints() -> Result<(), &'static str> {
    let source = r#"
@layer base {
  .btn { color: red; }
  .dead { border-color: red; }
}
@layer overrides {
  .btn { color: blue; }
  .dead { border-color: blue; }
}
:root {
  --cycle-a: var(--cycle-b);
  --cycle-b: var(--cycle-a);
  --bad: var(--missing);
}
.card { color: var(--bad); }
.tie { color: red; color: green; }
"#;
    let candidates =
        super::summarize_omena_query_style_hover_candidates("Component.module.scss", source)
            .ok_or("style candidates")?;

    let diagnostics = super::summarize_omena_query_style_diagnostics_for_file(
        "file:///workspace/src/Component.module.scss",
        source,
        candidates.candidates.as_slice(),
    );

    assert_eq!(diagnostics.product, "omena-query.diagnostics-for-file");
    assert!(
        diagnostics
            .ready_surfaces
            .contains(&"cascadeAwareDiagnostics")
    );
    assert_eq!(
        diagnostics
            .diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.code == "guaranteedInvalidCustomProperty")
            .count(),
        3
    );
    let diagnostic_codes = diagnostics
        .diagnostics
        .iter()
        .map(|diagnostic| diagnostic.code)
        .collect::<std::collections::BTreeSet<_>>();
    assert!(diagnostic_codes.contains("unreachableDeclaration"));
    assert!(diagnostic_codes.contains("deadCascadeLayer"));
    assert!(diagnostic_codes.contains("iacvtProne"));
    assert!(diagnostic_codes.contains("circularVar"));
    assert!(diagnostic_codes.contains("unspecifiedCascadeTie"));
    Ok(())
}

#[test]
fn style_diagnostics_collect_uppercase_and_fallback_var_references() -> Result<(), &'static str> {
    let source = r#"
:root {
  --cycle-a: VAR(--missing, var(--cycle-b));
  --cycle-b: var(--cycle-a);
}
.card { color: var(--cycle-a); }
"#;
    let candidates =
        super::summarize_omena_query_style_hover_candidates("Component.module.scss", source)
            .ok_or("style candidates")?;

    let diagnostics = super::summarize_omena_query_style_diagnostics_for_file(
        "file:///workspace/src/Component.module.scss",
        source,
        candidates.candidates.as_slice(),
    );
    let diagnostic_codes = diagnostics
        .diagnostics
        .iter()
        .map(|diagnostic| diagnostic.code)
        .collect::<std::collections::BTreeSet<_>>();

    assert!(diagnostic_codes.contains("circularVar"));
    assert!(
        diagnostics
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "circularVar"
                && diagnostic.message == "Custom property dependency graph contains a cycle.")
    );
    Ok(())
}

#[test]
fn style_diagnostics_for_file_include_keyframes_resolution_lints() -> Result<(), &'static str> {
    let source = ".button { animation: fade 1s ease; }\n@keyframes spin { to { opacity: 1; } }";
    let candidates =
        super::summarize_omena_query_style_hover_candidates("Component.module.css", source)
            .ok_or("style candidates")?;

    let diagnostics = super::summarize_omena_query_style_diagnostics_for_file(
        "file:///workspace/src/Component.module.css",
        source,
        candidates.candidates.as_slice(),
    );

    assert!(
        diagnostics
            .ready_surfaces
            .contains(&"missingKeyframesDiagnostics")
    );
    let keyframes_diagnostics = diagnostics
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.code == "missingKeyframes")
        .collect::<Vec<_>>();
    assert_eq!(keyframes_diagnostics.len(), 1);
    assert_eq!(
        keyframes_diagnostics[0].message,
        "@keyframes 'fade' not found in this file."
    );
    Ok(())
}

#[test]
fn style_diagnostics_for_file_include_same_file_sass_symbol_lints() -> Result<(), &'static str> {
    let source = "$known: 1rem;\n@mixin raised() { box-shadow: 0 0 $known; }\n.button { color: $missing; @include absent; }";
    let candidates =
        super::summarize_omena_query_style_hover_candidates("Component.module.scss", source)
            .ok_or("style candidates")?;

    let diagnostics = super::summarize_omena_query_style_diagnostics_for_file(
        "file:///workspace/src/Component.module.scss",
        source,
        candidates.candidates.as_slice(),
    );

    assert!(
        diagnostics
            .ready_surfaces
            .contains(&"missingSassSymbolDiagnostics")
    );
    let messages = diagnostics
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.code == "missingSassSymbol")
        .map(|diagnostic| diagnostic.message.as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        messages,
        vec![
            "Sass variable '$missing' not found in this file.",
            "Sass mixin '@mixin absent' not found in this file.",
        ]
    );
    Ok(())
}

#[test]
fn style_diagnostics_for_workspace_file_include_css_modules_resolution_lints()
-> Result<(), &'static str> {
    let sources = vec![
        OmenaQueryStyleSourceInputV0 {
            style_path: "/workspace/src/Component.module.css".to_string(),
            style_source: r#".button { composes: missingLocal; }
.missingModule { composes: root from "./Missing.module.css"; }
.external { composes: ghost from "./Base.module.css"; }
@value primary from "./MissingTokens.module.css";
@value absent from "./Tokens.module.css";"#
                .to_string(),
        },
        OmenaQueryStyleSourceInputV0 {
            style_path: "/workspace/src/Base.module.css".to_string(),
            style_source: ".base { color: blue; }".to_string(),
        },
        OmenaQueryStyleSourceInputV0 {
            style_path: "/workspace/src/Tokens.module.css".to_string(),
            style_source: "@value accent: blue;".to_string(),
        },
    ];

    let diagnostics = super::summarize_omena_query_style_diagnostics_for_workspace_file(
        "/workspace/src/Component.module.css",
        sources.as_slice(),
        &[],
        &[],
    )
    .ok_or("workspace style diagnostics")?;

    assert!(
        diagnostics
            .ready_surfaces
            .contains(&"cssModulesComposesResolutionDiagnostics")
    );
    assert!(
        diagnostics
            .ready_surfaces
            .contains(&"cssModulesValueResolutionDiagnostics")
    );
    let messages = diagnostics
        .diagnostics
        .iter()
        .map(|diagnostic| (diagnostic.code, diagnostic.message.as_str()))
        .collect::<Vec<_>>();
    assert!(messages.contains(&(
        "missingComposedSelector",
        "Selector '.missingLocal' not found in this file for composes.",
    )));
    assert!(messages.contains(&(
        "missingComposedModule",
        "Cannot resolve composed CSS Module './Missing.module.css'.",
    )));
    assert!(messages.contains(&(
        "missingComposedSelector",
        "Selector '.ghost' not found in composed module './Base.module.css'.",
    )));
    assert!(messages.contains(&(
        "missingValueModule",
        "Cannot resolve imported @value module './MissingTokens.module.css'.",
    )));
    assert!(messages.contains(&(
        "missingImportedValue",
        "@value 'absent' not found in './Tokens.module.css'.",
    )));
    Ok(())
}

#[test]
fn style_diagnostics_for_workspace_file_include_unused_selector_lints() -> Result<(), &'static str>
{
    let sources = vec![OmenaQueryStyleSourceInputV0 {
        style_path: "/workspace/src/App.module.css".to_string(),
        style_source:
            ".used { color: red; }\n.ghost { color: blue; }\n.composed { composes: used; }"
                .to_string(),
    }];
    let source_documents = vec![OmenaQuerySourceDocumentInputV0 {
        source_path: "/workspace/src/App.tsx".to_string(),
        source_source: r#"import styles from "./App.module.css";
export function App() {
  return <div className={styles.composed}>hi</div>;
}"#
        .to_string(),
    }];

    let diagnostics = super::summarize_omena_query_style_diagnostics_for_workspace_file(
        "/workspace/src/App.module.css",
        sources.as_slice(),
        source_documents.as_slice(),
        &[],
    )
    .ok_or("workspace style diagnostics")?;

    assert!(
        diagnostics
            .ready_surfaces
            .contains(&"unusedSelectorDiagnostics")
    );
    let unused = diagnostics
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.code == "unusedSelector")
        .map(|diagnostic| diagnostic.message.as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        unused,
        vec!["Selector '.ghost' is declared but never used."]
    );
    Ok(())
}

#[test]
fn completion_at_position_is_query_owned_for_style_and_source() -> Result<(), &'static str> {
    let source = ":root { --brand: red; }\n.root { color: var(--br); }\n.row { display: flex; }";
    let candidates =
        super::summarize_omena_query_style_hover_candidates("Component.module.scss", source)
            .ok_or("style candidates")?;

    let style_completion = super::summarize_omena_query_style_completion_at_position(
        "file:///workspace/src/Component.module.scss",
        source,
        ParserPositionV0 {
            line: 1,
            character: 23,
        },
        candidates.candidates.as_slice(),
    );
    assert_eq!(style_completion.product, "omena-query.completion-at");
    assert_eq!(
        style_completion.context_kind,
        "styleCustomPropertyReference"
    );
    assert_eq!(style_completion.prefix.as_deref(), Some("--br"));
    assert_eq!(
        style_completion
            .items
            .iter()
            .map(|item| item.label.as_str())
            .collect::<Vec<_>>(),
        vec!["--brand"]
    );

    let source_completion = super::summarize_omena_query_source_completion_at_position(
        "file:///workspace/src/App.tsx",
        ParserPositionV0 {
            line: 1,
            character: 22,
        },
        &[
            OmenaQueryCompletionCandidateV0 {
                file_uri: "file:///workspace/src/Component.module.scss".to_string(),
                name: "root".to_string(),
                kind: "selector",
                range: ParserRangeV0 {
                    start: ParserPositionV0 {
                        line: 1,
                        character: 1,
                    },
                    end: ParserPositionV0 {
                        line: 1,
                        character: 5,
                    },
                },
                source: "omenaQueryStyleHoverCandidates",
            },
            OmenaQueryCompletionCandidateV0 {
                file_uri: "file:///workspace/src/Other.module.scss".to_string(),
                name: "rootOther".to_string(),
                kind: "selector",
                range: ParserRangeV0 {
                    start: ParserPositionV0 {
                        line: 0,
                        character: 1,
                    },
                    end: ParserPositionV0 {
                        line: 0,
                        character: 10,
                    },
                },
                source: "omenaQueryStyleHoverCandidates",
            },
        ],
        Some("file:///workspace/src/Component.module.scss"),
        Some("ro"),
    );
    assert_eq!(source_completion.context_kind, "sourceCssModuleTarget");
    assert_eq!(source_completion.item_count, 1);
    assert_eq!(source_completion.items[0].label, "root");
    assert!(
        source_completion
            .ready_surfaces
            .contains(&"bridgeAwareSelectorCompletion")
    );
    Ok(())
}

#[test]
fn refs_for_class_is_query_owned_and_workspace_scoped() {
    let definition = OmenaQueryStyleSelectorDefinitionV0 {
        uri: "file:///workspace/src/Component.module.scss".to_string(),
        name: "root".to_string(),
        range: ParserRangeV0 {
            start: ParserPositionV0 {
                line: 0,
                character: 1,
            },
            end: ParserPositionV0 {
                line: 0,
                character: 5,
            },
        },
    };
    let references = vec![
        OmenaQuerySourceSelectorReferenceCandidateV0 {
            uri: "file:///workspace/src/App.tsx".to_string(),
            kind: "sourceSelectorReference",
            name: "root".to_string(),
            range: ParserRangeV0 {
                start: ParserPositionV0 {
                    line: 1,
                    character: 31,
                },
                end: ParserPositionV0 {
                    line: 1,
                    character: 35,
                },
            },
            source: "omenaQuerySourceSyntaxIndex",
            target_style_uri: Some("file:///workspace/src/Component.module.scss".to_string()),
        },
        OmenaQuerySourceSelectorReferenceCandidateV0 {
            uri: "file:///workspace/src/Other.tsx".to_string(),
            kind: "sourceSelectorReference",
            name: "root".to_string(),
            range: ParserRangeV0 {
                start: ParserPositionV0 {
                    line: 1,
                    character: 31,
                },
                end: ParserPositionV0 {
                    line: 1,
                    character: 35,
                },
            },
            source: "omenaQuerySourceSyntaxIndex",
            target_style_uri: Some("file:///workspace/src/Other.module.scss".to_string()),
        },
    ];

    let refs = super::summarize_omena_query_refs_for_class(
        "root",
        Some("file:///workspace/src/Component.module.scss"),
        true,
        &[definition],
        references.as_slice(),
    );
    assert_eq!(refs.product, "omena-query.refs-for-class");
    assert_eq!(refs.location_count, 2);
    assert_eq!(refs.locations[0].role, "definition");
    assert_eq!(refs.locations[1].role, "reference");
    assert_eq!(refs.locations[1].uri, "file:///workspace/src/App.tsx");
    assert!(
        refs.ready_surfaces
            .contains(&"workspaceWideSelectorReferences")
    );
}

#[test]
fn rename_plan_is_query_owned_and_workspace_scoped() {
    let definition = OmenaQueryStyleSelectorDefinitionV0 {
        uri: "file:///workspace/src/Component.module.scss".to_string(),
        name: "root".to_string(),
        range: ParserRangeV0 {
            start: ParserPositionV0 {
                line: 0,
                character: 1,
            },
            end: ParserPositionV0 {
                line: 0,
                character: 5,
            },
        },
    };
    let reference = OmenaQuerySourceSelectorReferenceEditTargetV0 {
        uri: "file:///workspace/src/App.tsx".to_string(),
        name: "root".to_string(),
        range: ParserRangeV0 {
            start: ParserPositionV0 {
                line: 1,
                character: 31,
            },
            end: ParserPositionV0 {
                line: 1,
                character: 35,
            },
        },
        target_style_uri: Some("file:///workspace/src/Component.module.scss".to_string()),
    };

    let plan = super::summarize_omena_query_rename_plan(
        "root",
        "button",
        Some("file:///workspace/src/Component.module.scss"),
        &[definition],
        &[reference],
    );
    assert_eq!(plan.product, "omena-query.rename-plan");
    assert_eq!(plan.edit_count, 2);
    assert_eq!(plan.edits[0].new_text, "button");
    assert_eq!(
        plan.edits
            .iter()
            .map(|edit| edit.uri.as_str())
            .collect::<Vec<_>>(),
        vec![
            "file:///workspace/src/App.tsx",
            "file:///workspace/src/Component.module.scss"
        ]
    );
    assert!(plan.ready_surfaces.contains(&"workspaceWideSelectorRename"));
}

#[test]
fn read_cascade_at_position_is_query_owned() {
    let source = ":root { --surface: white; }\n:root { --surface: black; }\n.button { color: var(--surface); }\n";
    let cascade = super::read_omena_query_cascade_at_position(
        "Component.module.css",
        source,
        &sample_input(),
        ParserPositionV0 {
            line: 2,
            character: 24,
        },
    );
    assert!(cascade.is_some());
    let Some(cascade) = cascade else {
        return;
    };

    assert_eq!(cascade.product, "omena-query.read-cascade-at-position");
    assert_eq!(cascade.status, "resolved");
    assert_eq!(cascade.cascade_engine, "omena-cascade");
    assert_eq!(cascade.reference_name.as_deref(), Some("--surface"));
    assert_eq!(cascade.winner_declaration_source_order, Some(1));
    assert_eq!(cascade.winner_declaration_layer_rank, Some(1));
    assert_eq!(cascade.candidate_declaration_count, 2);
    assert_eq!(cascade.shadowed_declaration_source_orders, vec![0]);
    assert_eq!(
        cascade.referenced_declaration_property.as_deref(),
        Some("color")
    );
    assert_eq!(
        cascade.referenced_declaration_value.as_deref(),
        Some("var(--surface)")
    );
    assert_eq!(
        cascade.referenced_declaration_computed_value_status,
        Some("resolved")
    );
    assert_eq!(
        cascade.referenced_declaration_computed_value.as_deref(),
        Some("black")
    );
    assert!(!cascade.referenced_declaration_invalid_at_computed_value_time);
    assert_eq!(cascade.custom_property_fixed_point_iteration_count, 1);
    assert_eq!(
        cascade.custom_property_fixed_point_guaranteed_invalid_count,
        0
    );
    assert_eq!(
        cascade.reference_custom_property_fixed_point_status,
        Some("fixedPointStable")
    );
    assert_eq!(
        cascade
            .reference_custom_property_fixed_point_value
            .as_deref(),
        Some("black")
    );
    assert!(
        cascade
            .referenced_declaration_computed_value_derivation_steps
            .contains(&"computedValueResolved")
    );

    let no_reference = super::read_omena_query_cascade_at_position(
        "Component.module.css",
        source,
        &sample_input(),
        ParserPositionV0 {
            line: 0,
            character: 1,
        },
    );
    assert!(no_reference.is_some());
    assert_eq!(
        no_reference.map(|cascade| cascade.status),
        Some("noCustomPropertyReference")
    );
}

#[test]
fn read_cascade_at_position_uses_layer_ranked_lfp_winner() {
    let source = r#".button { --surface: unlayered; }
@layer components {
  .button {
    --surface: layered;
    color: var(--surface);
  }
}
"#;
    let cascade = super::read_omena_query_cascade_at_position(
        "Component.module.css",
        source,
        &sample_input(),
        ParserPositionV0 {
            line: 4,
            character: 15,
        },
    );
    assert!(cascade.is_some());
    let Some(cascade) = cascade else {
        return;
    };

    assert_eq!(cascade.status, "resolved");
    assert_eq!(cascade.reference_name.as_deref(), Some("--surface"));
    assert_eq!(cascade.winner_declaration_source_order, Some(0));
    assert_eq!(cascade.winner_declaration_layer_rank, Some(2));
    assert_eq!(
        cascade
            .reference_custom_property_fixed_point_value
            .as_deref(),
        Some("unlayered")
    );
}

#[test]
fn read_cascade_at_position_reports_iacvt_seed() {
    let source = ":root { --a: var(--b); --b: var(--a); }\n.button { color: var(--a); }\n";
    let cascade = super::read_omena_query_cascade_at_position(
        "Component.module.css",
        source,
        &sample_input(),
        ParserPositionV0 {
            line: 1,
            character: 22,
        },
    );
    assert!(cascade.is_some());
    let Some(cascade) = cascade else {
        return;
    };

    assert_eq!(cascade.status, "resolved");
    assert_eq!(cascade.reference_name.as_deref(), Some("--a"));
    assert_eq!(
        cascade.referenced_declaration_computed_value_status,
        Some("invalidAtComputedValueTime")
    );
    assert_eq!(
        cascade.referenced_declaration_computed_value.as_deref(),
        Some("canvastext")
    );
    assert!(cascade.referenced_declaration_invalid_at_computed_value_time);
    assert!(cascade.custom_property_fixed_point_iteration_count >= 2);
    assert_eq!(
        cascade.custom_property_fixed_point_guaranteed_invalid_count,
        2
    );
    assert_eq!(
        cascade.reference_custom_property_fixed_point_status,
        Some("guaranteedInvalid")
    );
    assert_eq!(
        cascade
            .reference_custom_property_fixed_point_value
            .as_deref(),
        Some("guaranteed-invalid")
    );
    assert!(
        cascade
            .referenced_declaration_computed_value_derivation_steps
            .contains(&"invalidAtComputedValueTimeFallsBackAsUnset")
    );
}

#[test]
fn missing_selector_diagnostics_are_query_owned() {
    let diagnostic = super::summarize_omena_query_missing_selector_diagnostic(
        "file:///workspace/src/App.module.scss",
        ".root {\n}\n",
        "missing",
        ParserRangeV0 {
            start: ParserPositionV0 {
                line: 2,
                character: 18,
            },
            end: ParserPositionV0 {
                line: 2,
                character: 25,
            },
        },
    );

    assert_eq!(diagnostic.code, "missingSelector");
    assert_eq!(
        diagnostic.message,
        "CSS Module selector '.missing' not found in indexed style tokens."
    );
    assert_eq!(
        diagnostic
            .create_selector
            .as_ref()
            .map(|action| action.new_text.as_str()),
        Some("\n\n.missing {\n}\n")
    );
    assert_eq!(
        diagnostic
            .create_selector
            .as_ref()
            .map(|action| action.range),
        Some(ParserRangeV0 {
            start: ParserPositionV0 {
                line: 2,
                character: 0,
            },
            end: ParserPositionV0 {
                line: 2,
                character: 0,
            },
        })
    );
}

#[test]
fn source_diagnostics_for_file_are_query_owned() {
    let diagnostics = super::summarize_omena_query_source_diagnostics_for_file(
        "file:///workspace/src/App.tsx",
        &[OmenaQuerySourceMissingSelectorDiagnosticCandidateV0 {
            target_style_uri: "file:///workspace/src/App.module.scss".to_string(),
            target_style_source: ".root {\n}\n".to_string(),
            selector_name: "missing".to_string(),
            source_reference_range: ParserRangeV0 {
                start: ParserPositionV0 {
                    line: 2,
                    character: 18,
                },
                end: ParserPositionV0 {
                    line: 2,
                    character: 25,
                },
            },
        }],
    );

    assert_eq!(diagnostics.product, "omena-query.diagnostics-for-file");
    assert_eq!(diagnostics.file_kind, "source");
    assert_eq!(diagnostics.diagnostic_count, 1);
    assert_eq!(diagnostics.diagnostics[0].code, "missingSelector");
    assert!(
        diagnostics
            .ready_surfaces
            .contains(&"crossLanguageDiagnostics")
    );
}

#[test]
fn source_diagnostics_for_workspace_file_are_query_owned() {
    let diagnostics = super::summarize_omena_query_source_diagnostics_for_workspace_file(
        "/workspace/src/App.tsx",
        r#"import bind from "classnames/bind";
import styles from "./App.module.scss";
import missing from "./Missing.module.scss";
const cx = bind.bind(styles);
const variant = Math.random() > 0.5 ? "chip" : "ghost";
const dynamicPrefix = "lost-" + suffix;
export function App({ suffix }) {
  return <div className={cx("ghost", variant, dynamicPrefix, `empty-${suffix}`)} data-x={styles.ghost} />;
}"#,
        &[OmenaQueryStyleSourceInputV0 {
            style_path: "/workspace/src/App.module.scss".to_string(),
            style_source: ".root {}\n.chip {}\n".to_string(),
        }],
        &[],
    );

    let codes = diagnostics
        .diagnostics
        .iter()
        .map(|diagnostic| diagnostic.code)
        .collect::<Vec<_>>();
    assert_eq!(diagnostics.product, "omena-query.diagnostics-for-file");
    assert_eq!(diagnostics.file_kind, "source");
    assert!(codes.contains(&"missingModule"));
    assert!(codes.contains(&"missingStaticClass"));
    assert!(codes.contains(&"missingResolvedClassValues"));
    assert!(codes.contains(&"missingResolvedClassDomain"));
    assert!(codes.contains(&"missingTemplatePrefix"));
    assert!(
        diagnostics
            .ready_surfaces
            .contains(&"sourceResolvedClassDiagnostics")
    );
}

#[test]
fn source_provider_candidate_resolution_is_query_owned() {
    let source_range = ParserRangeV0 {
        start: ParserPositionV0 {
            line: 0,
            character: 0,
        },
        end: ParserPositionV0 {
            line: 0,
            character: 4,
        },
    };
    let definition_range = ParserRangeV0 {
        start: ParserPositionV0 {
            line: 1,
            character: 1,
        },
        end: ParserPositionV0 {
            line: 1,
            character: 5,
        },
    };

    let resolution = super::resolve_omena_query_source_provider_candidates(
        vec![
            super::OmenaQuerySourceSelectorCandidateV0 {
                kind: "sourceSelectorReference",
                name: "root".to_string(),
                range: source_range,
                source: "omenaQuerySourceSyntaxIndex",
                target_style_uri: Some("file:///workspace/src/App.module.scss".to_string()),
            },
            super::OmenaQuerySourceSelectorCandidateV0 {
                kind: "sourceSelectorPrefixReference",
                name: "btn-".to_string(),
                range: source_range,
                source: "omenaQuerySourceSyntaxIndex",
                target_style_uri: Some("file:///workspace/src/App.module.scss".to_string()),
            },
            super::OmenaQuerySourceSelectorCandidateV0 {
                kind: "sourceSelectorReference",
                name: "ghost".to_string(),
                range: source_range,
                source: "omenaQuerySourceSyntaxIndex",
                target_style_uri: Some("file:///workspace/src/Other.module.scss".to_string()),
            },
        ],
        &[
            super::OmenaQueryStyleSelectorDefinitionV0 {
                uri: "file:///workspace/src/App.module.scss".to_string(),
                name: "root".to_string(),
                range: definition_range,
            },
            super::OmenaQueryStyleSelectorDefinitionV0 {
                uri: "file:///workspace/src/App.module.scss".to_string(),
                name: "btn-primary".to_string(),
                range: definition_range,
            },
        ],
    );

    assert_eq!(
        resolution
            .matched
            .iter()
            .map(|candidate| candidate.name.as_str())
            .collect::<Vec<_>>(),
        vec!["btn-", "root"]
    );
    assert_eq!(
        resolution
            .unresolved
            .iter()
            .map(|candidate| candidate.name.as_str())
            .collect::<Vec<_>>(),
        vec!["ghost"]
    );

    let prefix_candidate = &resolution.matched[0];
    let definitions = vec![
        super::OmenaQueryStyleSelectorDefinitionV0 {
            uri: "file:///workspace/src/App.module.scss".to_string(),
            name: "root".to_string(),
            range: definition_range,
        },
        super::OmenaQueryStyleSelectorDefinitionV0 {
            uri: "file:///workspace/src/App.module.scss".to_string(),
            name: "btn-primary".to_string(),
            range: definition_range,
        },
    ];
    assert_eq!(
        super::resolve_omena_query_source_candidate_selector_names(
            prefix_candidate,
            definitions.as_slice(),
            None,
        ),
        vec!["btn-primary".to_string()]
    );
    assert_eq!(
        super::resolve_omena_query_style_selector_definitions_for_source_candidate(
            prefix_candidate,
            definitions.as_slice(),
        )
        .into_iter()
        .map(|definition| definition.name)
        .collect::<Vec<_>>(),
        vec!["btn-primary".to_string()]
    );
}

#[test]
fn source_candidate_matching_normalizes_percent_encoded_file_uris() {
    let source_range = ParserRangeV0 {
        start: ParserPositionV0 {
            line: 0,
            character: 0,
        },
        end: ParserPositionV0 {
            line: 0,
            character: 4,
        },
    };
    let definition_range = ParserRangeV0 {
        start: ParserPositionV0 {
            line: 1,
            character: 1,
        },
        end: ParserPositionV0 {
            line: 1,
            character: 5,
        },
    };
    let candidate = super::OmenaQuerySourceSelectorCandidateV0 {
        kind: "sourceSelectorPrefixReference",
        name: "btn-".to_string(),
        range: source_range,
        source: "omenaQuerySourceSyntaxIndex",
        target_style_uri: Some(
            "file:///workspace/app/%28marketing%29/Button.module.scss".to_string(),
        ),
    };
    let definitions = vec![super::OmenaQueryStyleSelectorDefinitionV0 {
        uri: "file:///workspace/app/(marketing)/Button.module.scss".to_string(),
        name: "btn-primary".to_string(),
        range: definition_range,
    }];

    assert_eq!(
        super::resolve_omena_query_source_candidate_selector_names(
            &candidate,
            definitions.as_slice(),
            None,
        ),
        vec!["btn-primary".to_string()]
    );
}

#[test]
fn source_syntax_index_adapter_is_query_owned_without_changing_product() {
    let style_uri = super::resolve_omena_query_style_uri_for_specifier(
        "file:///workspace/src/Button.tsx",
        Some("file:///workspace"),
        "./Button.module.scss",
    );
    assert_eq!(
        style_uri.as_deref(),
        Some("file:///workspace/src/Button.module.scss")
    );
    let style_uri = style_uri.unwrap_or_default();
    assert_eq!(style_uri, "file:///workspace/src/Button.module.scss");

    let import_summary = super::summarize_omena_query_source_import_declarations(
        "import styles from './Button.module.scss';",
    );
    assert_eq!(import_summary.import_count, 1);
    assert_eq!(import_summary.imports[0].binding, "styles");

    let source = "import styles from './Button.module.scss';\nconst el = styles.root;\n";
    let mut index = super::summarize_omena_query_source_syntax_index(
        source,
        vec![super::OmenaQuerySourceImportedStyleBindingV0 {
            binding: "styles".to_string(),
            style_uri,
        }],
        Vec::new(),
    );
    assert_eq!(index.product, "omena-bridge.source-syntax-index");
    assert_eq!(index.selector_references.len(), 1);
    let reference = &index.selector_references[0];
    assert_eq!(
        &source[reference.byte_span.start..reference.byte_span.end],
        "root"
    );

    super::canonicalize_omena_query_source_selector_references(&mut index.selector_references);
    assert_eq!(index.selector_references.len(), 1);
}

#[test]
fn selector_rename_edit_planning_is_query_owned() {
    let source_range = ParserRangeV0 {
        start: ParserPositionV0 {
            line: 3,
            character: 16,
        },
        end: ParserPositionV0 {
            line: 3,
            character: 20,
        },
    };
    let definition_range = ParserRangeV0 {
        start: ParserPositionV0 {
            line: 0,
            character: 1,
        },
        end: ParserPositionV0 {
            line: 0,
            character: 5,
        },
    };

    let edits = super::resolve_omena_query_selector_rename_edits(
        "root",
        ".shell",
        Some("file:///workspace/src/App.module.scss"),
        &[super::OmenaQueryStyleSelectorDefinitionV0 {
            uri: "file:///workspace/src/App.module.scss".to_string(),
            name: "root".to_string(),
            range: definition_range,
        }],
        &[super::OmenaQuerySourceSelectorReferenceEditTargetV0 {
            uri: "file:///workspace/src/App.tsx".to_string(),
            name: "root".to_string(),
            range: source_range,
            target_style_uri: Some("file:///workspace/src/App.module.scss".to_string()),
        }],
    );

    assert_eq!(
        edits
            .iter()
            .map(|edit| (edit.uri.as_str(), edit.new_text.as_str()))
            .collect::<Vec<_>>(),
        vec![
            ("file:///workspace/src/App.module.scss", "shell"),
            ("file:///workspace/src/App.tsx", "shell"),
        ]
    );
}

#[test]
fn sass_symbol_matching_is_query_owned() {
    let source = "$accent: red;\n.button { color: $accent; }\n";
    let Some(candidates) =
        super::summarize_omena_query_style_hover_candidates("Component.module.scss", source)
    else {
        return;
    };

    assert!(super::is_omena_query_sass_symbol_candidate_kind(
        "sassVariableDeclaration"
    ));
    assert!(super::is_omena_query_sass_symbol_reference_kind(
        "sassVariableReference"
    ));
    assert_eq!(
        super::omena_query_sass_symbol_kind_from_candidate_kind("sassVariableReference"),
        Some("variable")
    );
    assert!(super::omena_query_sass_symbol_target_matches(
        "sassVariableReference",
        "accent",
        None,
        "sassVariableDeclaration",
        "accent",
        None,
    ));

    let declarations = super::resolve_omena_query_sass_symbol_declarations(
        candidates.candidates.as_slice(),
        "variable",
        "accent",
    );
    assert_eq!(declarations.len(), 1);
    assert_eq!(declarations[0].kind, "sassVariableDeclaration");
}

#[test]
fn sass_module_sources_are_query_owned() {
    let sources = super::summarize_omena_query_sass_module_sources(
        "Component.module.scss",
        r#"
@use "./tokens" as tokens;
@use "./reset" as *;
@use "sass:map";
@import "./legacy";
@forward "./theme";
@forward "sass:color";
"#,
    );
    assert!(sources.is_some());
    let Some(sources) = sources else {
        return;
    };

    assert_eq!(sources.product, "omena-query.sass-module-sources");
    assert!(sources.module_use_edges.iter().any(|edge| {
        edge.source == "./tokens"
            && edge.namespace.as_deref() == Some("tokens")
            && edge.namespace_kind == "alias"
    }));
    assert!(sources.module_use_edges.iter().any(|edge| {
        edge.source == "./reset" && edge.namespace.is_none() && edge.namespace_kind == "wildcard"
    }));
    assert!(sources.module_use_edges.iter().any(|edge| {
        edge.source == "./legacy" && edge.namespace.is_none() && edge.namespace_kind == "wildcard"
    }));
    assert_eq!(
        super::resolve_omena_query_sass_module_use_sources_for_candidate(&sources, None),
        vec!["./legacy".to_string(), "./reset".to_string()]
    );
    assert_eq!(
        super::resolve_omena_query_sass_module_use_sources_for_candidate(&sources, Some("tokens"),),
        vec!["./tokens".to_string()]
    );
    assert_eq!(
        super::resolve_omena_query_sass_forward_sources(&sources),
        vec!["./theme".to_string()]
    );
}

fn backend<'a>(
    summary: &'a SelectedQueryAdapterCapabilitiesV0,
    backend_kind: &str,
) -> Option<&'a super::SelectedQueryBackendCapabilityV0> {
    summary
        .backend_kinds
        .iter()
        .find(|backend| backend.backend_kind == backend_kind)
}

fn sample_input() -> EngineInputV2 {
    EngineInputV2 {
        version: "2".to_string(),
        sources: vec![SourceAnalysisInputV2 {
            document: SourceDocumentV2 {
                class_expressions: vec![
                    ClassExpressionInputV2 {
                        id: "expr-1".to_string(),
                        kind: "symbolRef".to_string(),
                        scss_module_path: "/tmp/App.module.scss".to_string(),
                        range: range(4, 12, 4, 16),
                        class_name: None,
                        root_binding_decl_id: Some("decl-1".to_string()),
                        access_path: None,
                    },
                    ClassExpressionInputV2 {
                        id: "expr-2".to_string(),
                        kind: "styleAccess".to_string(),
                        scss_module_path: "/tmp/Card.module.scss".to_string(),
                        range: range(6, 9, 6, 20),
                        class_name: Some("card-header".to_string()),
                        root_binding_decl_id: None,
                        access_path: Some(vec!["card".to_string(), "header".to_string()]),
                    },
                ],
            },
        }],
        styles: vec![
            StyleAnalysisInputV2 {
                file_path: "/tmp/App.module.scss".to_string(),
                source: None,
                document: StyleDocumentV2 {
                    selectors: vec![StyleSelectorV2 {
                        name: "btn-active".to_string(),
                        view_kind: "canonical".to_string(),
                        canonical_name: Some("btn-active".to_string()),
                        range: range(1, 1, 1, 12),
                        nested_safety: Some("safe".to_string()),
                        composes: None,
                        bem_suffix: None,
                    }],
                },
            },
            StyleAnalysisInputV2 {
                file_path: "/tmp/Card.module.scss".to_string(),
                source: None,
                document: StyleDocumentV2 {
                    selectors: vec![StyleSelectorV2 {
                        name: "card-header".to_string(),
                        view_kind: "canonical".to_string(),
                        canonical_name: Some("card-header".to_string()),
                        range: range(3, 1, 3, 13),
                        nested_safety: Some("unsafe".to_string()),
                        composes: None,
                        bem_suffix: None,
                    }],
                },
            },
        ],
        type_facts: vec![
            TypeFactEntryV2 {
                file_path: "/tmp/App.tsx".to_string(),
                expression_id: "expr-1".to_string(),
                facts: StringTypeFactsV2 {
                    kind: "constrained".to_string(),
                    constraint_kind: Some("prefixSuffix".to_string()),
                    values: None,
                    prefix: Some("btn-".to_string()),
                    suffix: Some("-active".to_string()),
                    min_len: Some(10),
                    max_len: None,
                    char_must: None,
                    char_may: None,
                    may_include_other_chars: None,
                },
            },
            TypeFactEntryV2 {
                file_path: "/tmp/Card.tsx".to_string(),
                expression_id: "expr-2".to_string(),
                facts: StringTypeFactsV2 {
                    kind: "finiteSet".to_string(),
                    constraint_kind: None,
                    values: Some(vec!["card-header".to_string(), "card-body".to_string()]),
                    prefix: None,
                    suffix: None,
                    min_len: None,
                    max_len: None,
                    char_must: None,
                    char_may: None,
                    may_include_other_chars: None,
                },
            },
        ],
    }
}

fn reduced_product_projection_input() -> EngineInputV2 {
    EngineInputV2 {
        version: "2".to_string(),
        sources: vec![SourceAnalysisInputV2 {
            document: SourceDocumentV2 {
                class_expressions: vec![
                    ClassExpressionInputV2 {
                        id: "expr-primary".to_string(),
                        kind: "symbolRef".to_string(),
                        scss_module_path: "/tmp/App.module.scss".to_string(),
                        range: range(4, 12, 4, 16),
                        class_name: None,
                        root_binding_decl_id: Some("decl-primary".to_string()),
                        access_path: None,
                    },
                    ClassExpressionInputV2 {
                        id: "expr-secondary".to_string(),
                        kind: "symbolRef".to_string(),
                        scss_module_path: "/tmp/App.module.scss".to_string(),
                        range: range(5, 12, 5, 16),
                        class_name: None,
                        root_binding_decl_id: Some("decl-secondary".to_string()),
                        access_path: None,
                    },
                ],
            },
        }],
        styles: vec![StyleAnalysisInputV2 {
            file_path: "/tmp/App.module.scss".to_string(),
            source: None,
            document: StyleDocumentV2 {
                selectors: vec![
                    style_selector("btn--active"),
                    style_selector("btn-primary--active"),
                    style_selector("btn-secondary--active"),
                    style_selector("card-active"),
                ],
            },
        }],
        type_facts: vec![
            TypeFactEntryV2 {
                file_path: "/tmp/App.tsx".to_string(),
                expression_id: "expr-primary".to_string(),
                facts: StringTypeFactsV2 {
                    kind: "constrained".to_string(),
                    constraint_kind: Some("prefixSuffix".to_string()),
                    values: None,
                    prefix: Some("btn-primary-".to_string()),
                    suffix: Some("-active".to_string()),
                    min_len: Some("btn-primary--active".len()),
                    max_len: None,
                    char_must: None,
                    char_may: None,
                    may_include_other_chars: None,
                },
            },
            TypeFactEntryV2 {
                file_path: "/tmp/App.tsx".to_string(),
                expression_id: "expr-secondary".to_string(),
                facts: StringTypeFactsV2 {
                    kind: "constrained".to_string(),
                    constraint_kind: Some("prefixSuffix".to_string()),
                    values: None,
                    prefix: Some("btn-secondary-".to_string()),
                    suffix: Some("-active".to_string()),
                    min_len: Some("btn-secondary--active".len()),
                    max_len: None,
                    char_must: None,
                    char_may: None,
                    may_include_other_chars: None,
                },
            },
        ],
    }
}

fn style_selector(name: &str) -> StyleSelectorV2 {
    StyleSelectorV2 {
        name: name.to_string(),
        view_kind: "canonical".to_string(),
        canonical_name: Some(name.to_string()),
        range: range(1, 1, 1, 1 + name.len()),
        nested_safety: Some("safe".to_string()),
        composes: None,
        bem_suffix: None,
    }
}

fn range(
    start_line: usize,
    start_character: usize,
    end_line: usize,
    end_character: usize,
) -> RangeV2 {
    RangeV2 {
        start: PositionV2 {
            line: start_line,
            character: start_character,
        },
        end: PositionV2 {
            line: end_line,
            character: end_character,
        },
    }
}
