use engine_input_producers::{EngineInputV2, StyleAnalysisInputV2, StyleDocumentV2};
use omena_abstract_value::SelectorProjectionCertaintyV0;

use super::{
    OmenaQueryExpressionDomainFlowRuntimeV0, OmenaQueryStylePackageManifestV0, ParserPositionV0,
    ParserRangeV0, execute_omena_query_consumer_build_style_source,
    execute_omena_query_consumer_build_style_source_for_target_query,
    execute_omena_query_consumer_build_style_source_for_target_query_with_context_and_options,
    execute_omena_query_consumer_build_style_source_for_target_query_with_options,
    execute_omena_query_consumer_build_style_source_with_engine_input_context,
    execute_omena_query_consumer_build_style_sources,
    execute_omena_query_consumer_build_style_sources_for_target_query_with_context_and_options,
    execute_omena_query_consumer_build_style_sources_with_context,
    execute_omena_query_transform_passes_from_source, list_omena_query_transform_pass_summaries,
    summarize_omena_query_analyzed_graph, summarize_omena_query_boundary,
    summarize_omena_query_consumer_check_style_source,
    summarize_omena_query_custom_property_annotations, summarize_omena_query_evaluation_runtime,
    summarize_omena_query_expression_domain_call_site_flow_analysis,
    summarize_omena_query_expression_domain_control_flow_analysis,
    summarize_omena_query_expression_domain_flow_analysis,
    summarize_omena_query_expression_domain_incremental_flow_analysis,
    summarize_omena_query_expression_domain_provenance_explanations,
    summarize_omena_query_expression_domain_reduced_product_iteration,
    summarize_omena_query_expression_domain_selector_projection,
    summarize_omena_query_expression_semantics_canonical_producer_signal,
    summarize_omena_query_expression_semantics_query_fragments, summarize_omena_query_fast_facts,
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
    summarize_omena_query_transform_context_from_engine_input,
    summarize_omena_query_transform_context_from_sources,
    summarize_omena_query_transform_plan_from_source,
    summarize_omena_query_transform_plan_from_target_query,
};
use crate::{
    OmenaQueryCompletionCandidateV0, OmenaQuerySourceSelectorReferenceCandidateV0,
    OmenaQuerySourceSelectorReferenceEditTargetV0, OmenaQueryStyleSelectorDefinitionV0,
    OmenaQueryStyleSourceInputV0, OmenaQueryTargetFeatureSupportV0,
    OmenaQueryTargetTransformOptionsV0, OmenaQueryTransformExecutionContextV0,
    OmenaQueryTransformModuleEvaluationV0, OmenaQueryTransformPrintMode,
    OmenaQueryTransformPrintOptionsV0, check_omena_query_schema_version,
    default_omena_query_transform_print_options, modern_omena_query_target_feature_support,
    summarize_omena_query_schema_version_policy,
};

mod cross_file_summary;
mod source_surfaces;
mod style_diagnostics;
mod style_semantic_graph;
mod stylesheet_evaluation;
mod support;

use support::{
    backend, reduced_product_iteration_input, reduced_product_projection_input, sample_input,
    style_selector,
};

#[test]
fn summarizes_query_boundary_over_producer_fragments() {
    let input = sample_input();
    let summary = summarize_omena_query_boundary(&input);

    assert_eq!(summary.schema_version, "0");
    assert_eq!(summary.product, "omena-query.boundary");
    assert_eq!(summary.query_engine_name, "omena-query");
    assert_eq!(
        summary.schema_version_policy.product,
        "omena-query.schema-version-policy"
    );
    assert_eq!(summary.schema_version_policy.current_version, "0");
    assert_eq!(summary.schema_version_policy.current_version_label, "V0");
    assert_eq!(summary.schema_version_policy.accepted_versions, vec!["0"]);
    assert_eq!(
        summary.schema_version_policy.missing_version_policy,
        "rejectMissingSchemaVersionOnExternalInputs"
    );
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
            .contains(&"engine-input-producers.expression-domain-call-site-flow-analysis")
    );
    assert!(
        summary
            .delegated_fragment_products
            .contains(&"engine-input-producers.expression-domain-provenance-explanations")
    );
    assert!(
        summary
            .delegated_fragment_products
            .contains(&"engine-input-producers.expression-domain-reduced-product-iteration")
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
    assert!(summary.ready_surfaces.contains(&"fastFactsV0"));
    assert!(summary.ready_surfaces.contains(&"analyzedGraphV0"));
    assert!(
        summary
            .ready_surfaces
            .contains(&"customPropertyAnnotations")
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
fn exposes_fast_facts_analyzed_graph_and_custom_property_annotations() {
    let source = r#"
      @use "tokens";
      :root { --surface: #fff; }
      .card { color: var(--surface); }
      .card:hover { color: var(--accent); }
    "#;

    let fast_facts = summarize_omena_query_fast_facts("Card.module.scss", source);
    assert_eq!(fast_facts.schema_version, "0");
    assert_eq!(fast_facts.product, "omena-query.fast-facts");
    assert_eq!(fast_facts.tier, "fastFactsV0");
    assert_eq!(fast_facts.language, "scss");
    assert_eq!(fast_facts.selector_count, 2);
    assert_eq!(fast_facts.custom_property_count, 3);
    assert_eq!(fast_facts.module_edge_count, 1);

    let graph = summarize_omena_query_analyzed_graph("Card.module.scss", source);
    assert_eq!(graph.product, "omena-query.analyzed-graph");
    assert_eq!(graph.tier, "analyzedGraphV0");
    assert_eq!(graph.fast_facts.selector_count, fast_facts.selector_count);
    assert!(graph.graph_kinds.contains(&"customPropertyFacts"));
    assert!(graph.node_count >= fast_facts.selector_count);

    let annotations = summarize_omena_query_custom_property_annotations("Card.module.scss", source);
    assert_eq!(
        annotations.product,
        "omena-query.custom-property-annotations"
    );
    assert_eq!(annotations.annotation_count, 2);
    assert!(annotations.annotations.iter().any(|annotation| {
        annotation.name == "--surface"
            && annotation.declaration_count == 1
            && annotation.reference_count == 1
            && annotation.annotation_kind == "declarationAndReference"
            && annotation.participates_in_fixed_point
    }));
    assert!(annotations.annotations.iter().any(|annotation| {
        annotation.name == "--accent"
            && annotation.declaration_count == 0
            && annotation.reference_count == 1
            && annotation.annotation_kind == "reference"
            && !annotation.participates_in_fixed_point
    }));
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
    assert!(summary.ready_surfaces.contains(&"cascadeProofObligations"));
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
fn exposes_transform_plan_minified_print_mode() {
    let source = "/* remove */ .button { color: red; margin: 0px; }";
    let summary = summarize_omena_query_transform_plan_from_source(
        "Button.module.css",
        source,
        "modern",
        modern_omena_query_target_feature_support(),
        OmenaQueryTargetTransformOptionsV0::default(),
        OmenaQueryTransformPrintOptionsV0 {
            mode: OmenaQueryTransformPrintMode::Minified,
            include_source_map: true,
        },
    );

    assert_eq!(summary.product, "omena-query.transform-plan");
    assert_eq!(summary.execution.output_css, source);
    assert_eq!(summary.print.css, ".button{color:red;margin:0px}");
    assert!(summary.print.provenance_preserved);
    assert!(!summary.print.source_map_segments.is_empty());
    assert!(
        summary
            .print
            .source_map_segments
            .iter()
            .all(|segment| segment.generated_end <= summary.print.css.len())
    );
}

#[test]
fn transform_plan_keeps_plain_css_imports_out_of_scss_evaluator() {
    let source = r#"@import "./tokens.css"; .button { color: red; }"#;
    let summary = summarize_omena_query_transform_plan_from_source(
        "App.css",
        source,
        "modern",
        modern_omena_query_target_feature_support(),
        OmenaQueryTargetTransformOptionsV0::default(),
        default_omena_query_transform_print_options(),
    );

    assert_eq!(summary.product, "omena-query.transform-plan");
    assert!(summary.bundle.import_inline_required);
    assert!(!summary.bundle.module_evaluation_required);
    assert_eq!(summary.bundle.required_pass_ids, vec!["import-inline"]);
    assert!(!summary.combined_pass_ids.contains(&"scss-module-evaluate"));
    assert!(summary.combined_pass_ids.contains(&"import-inline"));
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
fn exposes_transform_execution_cascade_proof_obligations_from_source() {
    let source = r#".a { margin-top: 1px; margin-right: 2px; margin-bottom: 1px; margin-left: 2px; }
@supports (display: grid) { .grid { display: grid; } }
"#;
    let summary = execute_omena_query_transform_passes_from_source(
        "Button.module.css",
        source,
        &[
            "shorthand-combining".to_string(),
            "supports-static-eval".to_string(),
            "print-css".to_string(),
        ],
    );

    assert_eq!(
        summary.execution.cascade_proof_obligations.product,
        "omena-transform-passes.cascade-proof-obligations"
    );
    assert_eq!(
        summary.execution.cascade_proof_obligations.obligation_count,
        2
    );
    assert_eq!(
        summary.execution.cascade_proof_obligations.accepted_count,
        2
    );
    assert!(
        summary
            .execution
            .cascade_proof_obligations
            .checked_pass_ids
            .contains(&"shorthand-combining")
    );
    assert!(
        summary
            .execution
            .cascade_proof_obligations
            .checked_pass_ids
            .contains(&"supports-static-eval")
    );
    assert!(
        summary
            .execution
            .cascade_proof_obligations
            .obligations
            .iter()
            .any(|obligation| {
                obligation.proof_product == "omena-cascade.shorthand-combination-proof"
            })
    );
    assert!(
        summary
            .execution
            .cascade_proof_obligations
            .obligations
            .iter()
            .any(|obligation| obligation.proof_product == "omena-cascade.supports-static-eval")
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
fn derives_transform_context_with_transitive_design_token_routes() {
    let summary = summarize_omena_query_transform_context_from_sources(
        "Button.module.css",
        [
            (
                "Button.module.css",
                r#"@import "./tokens.css"; .button { color: var(--alias); }"#,
            ),
            (
                "tokens.css",
                r#":root { --alias: var(--brand); --brand: red; }"#,
            ),
        ],
        &[],
    );

    assert_eq!(summary.design_token_route_count, 2);
    assert_eq!(
        summary
            .context
            .design_token_routes
            .iter()
            .map(|route| (route.token_name.as_str(), route.routed_value.as_str()))
            .collect::<Vec<_>>(),
        vec![("--alias", "var(--brand)"), ("--brand", "red")]
    );
}

#[test]
fn derives_transform_context_with_transitive_import_inlines() {
    let summary = summarize_omena_query_transform_context_from_sources(
        "/tmp/App.css",
        [
            ("/tmp/base.css", ".base { color: red; }"),
            (
                "/tmp/tokens.css",
                r#"@import "./base.css"; .token { color: blue; }"#,
            ),
            (
                "/tmp/App.css",
                r#"@import "./tokens.css"; .app { color: green; }"#,
            ),
        ],
        &[],
    );

    assert_eq!(summary.import_inline_count, 1);
    assert_eq!(
        summary.context.import_inlines[0].import_source,
        "./tokens.css"
    );
    assert_eq!(
        summary.context.import_inlines[0].replacement_css,
        ".base { color: red; } .token { color: blue; }"
    );
}

#[test]
fn derives_unique_class_rewrites_for_repeated_escaped_selectors() {
    let summary = summarize_omena_query_transform_context_from_sources(
        "Escaped.module.css",
        [(
            "Escaped.module.css",
            r#".foo\:bar { color: red; } :local(.foo\:bar) { color: blue; } :global(.foo\:bar) .foo\:bar { color: green; } .hex\3A bar { color: purple; } .hex\:bar { color: cyan; }"#,
        )],
        &[],
    );

    assert_eq!(summary.class_name_rewrite_count, 2);
    assert_eq!(
        summary
            .context
            .class_name_rewrites
            .iter()
            .map(|rewrite| {
                (
                    rewrite.original_name.as_str(),
                    rewrite.rewritten_name.as_str(),
                )
            })
            .collect::<Vec<_>>(),
        vec![
            (r#"foo\:bar"#, "_foo_bar_0"),
            (r#"hex\3A bar"#, "_hex_bar_1")
        ]
    );
}

#[test]
fn transform_context_hashes_only_final_stem_css_module_paths() {
    let summary = summarize_omena_query_transform_context_from_sources(
        "Button.module.test.css",
        [(
            "Button.module.test.css",
            ".button { color: red; } .base { color: blue; }",
        )],
        &[],
    );
    let windows_summary = summarize_omena_query_transform_context_from_sources(
        r#"components\Card.MODULE.SCSS"#,
        [(r#"components\Card.MODULE.SCSS"#, ".card { color: red; }")],
        &[],
    );

    assert_eq!(summary.class_name_rewrite_count, 0);
    assert!(summary.context.class_name_rewrites.is_empty());
    assert_eq!(windows_summary.class_name_rewrite_count, 1);
    assert_eq!(
        windows_summary.context.class_name_rewrites[0].original_name,
        "card"
    );
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
fn consumer_build_style_sources_routes_transitive_design_token_aliases()
-> Result<(), Box<dyn std::error::Error>> {
    let sources = vec![
        OmenaQueryStyleSourceInputV0 {
            style_path: "Button.module.css".to_string(),
            style_source: r#"@import "./tokens.css"; .button { color: var(--alias); }"#.to_string(),
        },
        OmenaQueryStyleSourceInputV0 {
            style_path: "tokens.css".to_string(),
            style_source: ":root { --alias: var(--brand); --brand: red; }".to_string(),
        },
    ];
    let summary = execute_omena_query_consumer_build_style_sources(
        "Button.module.css",
        &sources,
        &[
            "import-inline".to_string(),
            "design-token-routing".to_string(),
            "print-css".to_string(),
        ],
        &[],
    )?;

    assert_eq!(summary.product, "omena-query.consumer-build-style-source");
    assert_eq!(summary.execution.design_token_routes.len(), 2);
    assert!(
        summary
            .execution
            .design_token_routes
            .iter()
            .any(|route| route.token_name == "--alias" && route.routed_value == "var(--brand)")
    );
    assert!(
        summary
            .execution
            .design_token_routes
            .iter()
            .any(|route| route.token_name == "--brand" && route.routed_value == "red")
    );
    assert!(!summary.execution.output_css.contains("@import"));
    assert!(
        summary
            .execution
            .output_css
            .contains(":root { --alias: red; --brand: red; }")
    );
    assert!(summary.execution.output_css.contains("color: red"));
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
fn derives_import_aware_static_stylesheet_module_evaluation() {
    let scss_summary = summarize_omena_query_transform_context_from_sources(
        "/tmp/App.module.scss",
        [
            ("/tmp/tokens.scss", "$brand: red; .base { color: blue; }"),
            (
                "/tmp/App.module.scss",
                r#"@import "./tokens.scss"; .button { color: $brand; }"#,
            ),
        ],
        &[],
    );
    let less_summary = summarize_omena_query_transform_context_from_sources(
        "/tmp/App.module.less",
        [
            ("/tmp/tokens.less", "@brand: red; .base { color: blue; }"),
            (
                "/tmp/App.module.less",
                r#"@import "./tokens.less"; .button { color: @brand; }"#,
            ),
        ],
        &[],
    );

    assert_eq!(scss_summary.import_inline_count, 1);
    assert_eq!(less_summary.import_inline_count, 1);
    assert_eq!(
        scss_summary
            .context
            .scss_module_evaluation
            .as_ref()
            .map(|evaluation| evaluation.evaluated_css.as_str()),
        Some(" .base { color: blue; } .button { color: red; }")
    );
    assert_eq!(
        less_summary
            .context
            .less_module_evaluation
            .as_ref()
            .map(|evaluation| evaluation.evaluated_css.as_str()),
        Some(" .base { color: blue; } .button { color: red; }")
    );
}

#[test]
fn derives_scss_use_aware_static_stylesheet_module_evaluation() {
    let summary = summarize_omena_query_transform_context_from_sources(
        "/tmp/App.module.scss",
        [
            (
                "/tmp/tokens.scss",
                "$brand: red; $gap: 8px; .base { color: blue; }",
            ),
            (
                "/tmp/App.module.scss",
                r#"@use "./tokens" as tokens; .button { color: tokens.$brand; margin: tokens.$gap; }"#,
            ),
        ],
        &[],
    );

    assert_eq!(
        summary
            .context
            .scss_module_evaluation
            .as_ref()
            .map(|evaluation| evaluation.evaluated_css.as_str()),
        Some("  .base { color: blue; } .button { color: red; margin: 8px; }")
    );
}

#[test]
fn derives_scss_use_aware_static_stylesheet_module_evaluation_without_duplicate_css_side_effects() {
    let summary = summarize_omena_query_transform_context_from_sources(
        "/tmp/App.module.scss",
        [
            ("/tmp/tokens.scss", "$brand: red; .base { color: $brand; }"),
            (
                "/tmp/App.module.scss",
                r#"@use "./tokens" as a; @use "./tokens" as b; .button { color: a.$brand; border-color: b.$brand; }"#,
            ),
        ],
        &[],
    );

    assert_eq!(
        summary
            .context
            .scss_module_evaluation
            .as_ref()
            .map(|evaluation| evaluation.evaluated_css.as_str()),
        Some(" .base { color: red; }  .button { color: red; border-color: red; }")
    );
}

#[test]
fn derives_wildcard_scss_use_aware_static_stylesheet_module_evaluation() {
    let summary = summarize_omena_query_transform_context_from_sources(
        "/tmp/App.module.scss",
        [
            (
                "/tmp/tokens.scss",
                "$brand: red; $gap: 8px; .base { color: blue; }",
            ),
            (
                "/tmp/App.module.scss",
                r#"@use "./tokens" as *; .button { color: $brand; margin: $gap; }"#,
            ),
        ],
        &[],
    );

    assert_eq!(
        summary
            .context
            .scss_module_evaluation
            .as_ref()
            .map(|evaluation| evaluation.evaluated_css.as_str()),
        Some("  .base { color: blue; } .button { color: red; margin: 8px; }")
    );
}

#[test]
fn derives_forwarded_scss_use_aware_static_stylesheet_module_evaluation() {
    let summary = summarize_omena_query_transform_context_from_sources(
        "/tmp/App.module.scss",
        [
            (
                "/tmp/tokens.scss",
                "$brand: red; $gap: 8px; .base { color: blue; }",
            ),
            ("/tmp/theme.scss", r#"@forward "./tokens" show $brand;"#),
            (
                "/tmp/App.module.scss",
                r#"@use "./theme" as theme; .button { color: theme.$brand; }"#,
            ),
        ],
        &[],
    );

    assert_eq!(
        summary
            .context
            .scss_module_evaluation
            .as_ref()
            .map(|evaluation| evaluation.evaluated_css.as_str()),
        Some("  .base { color: blue; } .button { color: red; }")
    );
}

#[test]
fn derives_forwarded_scss_module_evaluation_without_duplicate_css_side_effects() {
    let summary = summarize_omena_query_transform_context_from_sources(
        "/tmp/App.module.scss",
        [
            ("/tmp/tokens.scss", "$brand: red; .base { color: $brand; }"),
            (
                "/tmp/theme.scss",
                r#"@forward "./tokens"; @forward "./tokens";"#,
            ),
            (
                "/tmp/App.module.scss",
                r#"@use "./theme" as theme; .button { color: theme.$brand; }"#,
            ),
        ],
        &[],
    );

    assert_eq!(
        summary
            .context
            .scss_module_evaluation
            .as_ref()
            .map(|evaluation| evaluation.evaluated_css.as_str()),
        Some(" .base { color: red; }  .button { color: red; }")
    );
}

#[test]
fn derives_configured_scss_use_aware_static_stylesheet_module_evaluation() {
    let summary = summarize_omena_query_transform_context_from_sources(
        "/tmp/App.module.scss",
        [
            (
                "/tmp/tokens.scss",
                "$brand: blue !default; .base { color: $brand; }",
            ),
            (
                "/tmp/App.module.scss",
                r#"@use "./tokens" as tokens with ($brand: red); .button { color: tokens.$brand; }"#,
            ),
        ],
        &[],
    );

    assert_eq!(
        summary
            .context
            .scss_module_evaluation
            .as_ref()
            .map(|evaluation| evaluation.evaluated_css.as_str()),
        Some("  .base { color: red; } .button { color: red; }")
    );
}

#[test]
fn derives_configured_scss_forward_aware_static_stylesheet_module_evaluation() {
    let summary = summarize_omena_query_transform_context_from_sources(
        "/tmp/App.module.scss",
        [
            (
                "/tmp/tokens.scss",
                "$brand: blue !default; .base { color: $brand; }",
            ),
            (
                "/tmp/theme.scss",
                r#"@forward "./tokens" with ($brand: red);"#,
            ),
            (
                "/tmp/App.module.scss",
                r#"@use "./theme" as theme; .button { color: theme.$brand; }"#,
            ),
        ],
        &[],
    );

    assert_eq!(
        summary
            .context
            .scss_module_evaluation
            .as_ref()
            .map(|evaluation| evaluation.evaluated_css.as_str()),
        Some("  .base { color: red; } .button { color: red; }")
    );
}

#[test]
fn shares_configured_scss_module_instances_across_transitive_consumers() {
    let summary = summarize_omena_query_transform_context_from_sources(
        "/tmp/App.module.scss",
        [
            (
                "/tmp/tokens.scss",
                "$brand: blue !default; .base { color: $brand; }",
            ),
            (
                "/tmp/theme-a.scss",
                r#"@forward "./tokens" with ($brand: red);"#,
            ),
            (
                "/tmp/theme-b.scss",
                r#"@forward "./tokens" with ($brand: red);"#,
            ),
            (
                "/tmp/App.module.scss",
                r#"@use "./theme-a" as a; @use "./theme-b" as b; .button { color: a.$brand; border-color: b.$brand; }"#,
            ),
        ],
        &[],
    );

    let evaluated_css = summary
        .context
        .scss_module_evaluation
        .as_ref()
        .map(|evaluation| evaluation.evaluated_css.as_str())
        .unwrap_or_default();
    assert_eq!(evaluated_css.matches(".base { color: red; }").count(), 1);
    assert!(evaluated_css.contains(".button { color: red; border-color: red; }"));
}

#[test]
fn separates_configured_scss_module_instances_by_configuration_signature() {
    let summary = summarize_omena_query_transform_context_from_sources(
        "/tmp/App.module.scss",
        [
            (
                "/tmp/tokens.scss",
                "$brand: blue !default; .base { color: $brand; }",
            ),
            (
                "/tmp/theme-red.scss",
                r#"@forward "./tokens" with ($brand: red);"#,
            ),
            (
                "/tmp/theme-blue.scss",
                r#"@forward "./tokens" with ($brand: blue);"#,
            ),
            (
                "/tmp/App.module.scss",
                r#"@use "./theme-red" as redTheme; @use "./theme-blue" as blueTheme; .button { color: redTheme.$brand; border-color: blueTheme.$brand; }"#,
            ),
        ],
        &[],
    );

    let evaluated_css = summary
        .context
        .scss_module_evaluation
        .as_ref()
        .map(|evaluation| evaluation.evaluated_css.as_str())
        .unwrap_or_default();
    assert_eq!(evaluated_css.matches(".base { color: red; }").count(), 1);
    assert_eq!(evaluated_css.matches(".base { color: blue; }").count(), 1);
    assert!(evaluated_css.contains(".button { color: red; border-color: blue; }"));
}

#[test]
fn derives_prefixed_scss_forward_aware_static_stylesheet_module_evaluation() {
    let summary = summarize_omena_query_transform_context_from_sources(
        "/tmp/App.module.scss",
        [
            (
                "/tmp/tokens.scss",
                "$brand: red; $gap: 8px; .base { color: $brand; }",
            ),
            (
                "/tmp/theme.scss",
                r#"@forward "./tokens" as token-* show $token-brand;"#,
            ),
            (
                "/tmp/App.module.scss",
                r#"@use "./theme" as theme; .button { color: theme.$token-brand; }"#,
            ),
        ],
        &[],
    );

    assert_eq!(
        summary
            .context
            .scss_module_evaluation
            .as_ref()
            .map(|evaluation| evaluation.evaluated_css.as_str()),
        Some("  .base { color: red; } .button { color: red; }")
    );
}

#[test]
fn derives_prefixed_scss_forward_hide_filters_after_prefixing() {
    let summary = summarize_omena_query_transform_context_from_sources(
        "/tmp/App.module.scss",
        [
            (
                "/tmp/tokens.scss",
                "$brand: red; $gap: 8px; .base { color: $brand; }",
            ),
            (
                "/tmp/theme.scss",
                r#"@forward "./tokens" as token-* hide $token-gap;"#,
            ),
            (
                "/tmp/App.module.scss",
                r#"@use "./theme" as theme; .button { color: theme.$token-brand; margin: theme.$token-gap; }"#,
            ),
        ],
        &[],
    );

    assert_eq!(
        summary
            .context
            .scss_module_evaluation
            .as_ref()
            .map(|evaluation| evaluation.evaluated_css.as_str()),
        Some("  .base { color: red; } .button { color: red; margin: theme.$token-gap; }")
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
fn derives_transform_context_with_transitive_cross_file_value_resolutions() {
    let summary = summarize_omena_query_transform_context_from_sources(
        "/tmp/App.module.css",
        [
            ("/tmp/base.module.css", "@value primary: #fff;"),
            (
                "/tmp/tokens.module.css",
                r#"@value primary from "./base.module.css"; @value alias: primary;"#,
            ),
            (
                "/tmp/App.module.css",
                r#"@value alias as brand from "./tokens.module.css"; .btn { color: brand; }"#,
            ),
        ],
        &[],
    );

    assert_eq!(summary.product, "omena-query.transform-context");
    assert_eq!(summary.css_module_value_resolution_count, 1);
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
        vec![("brand", "#fff")]
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
fn consumer_build_resolves_transitive_cross_file_css_modules_values_through_query_context()
-> Result<(), Box<dyn std::error::Error>> {
    let summary = execute_omena_query_consumer_build_style_sources(
        "/tmp/App.module.css",
        &[
            OmenaQueryStyleSourceInputV0 {
                style_path: "/tmp/base.module.css".to_string(),
                style_source: "@value primary: #fff;".to_string(),
            },
            OmenaQueryStyleSourceInputV0 {
                style_path: "/tmp/tokens.module.css".to_string(),
                style_source: r#"@value primary from "./base.module.css"; @value alias: primary;"#
                    .to_string(),
            },
            OmenaQueryStyleSourceInputV0 {
                style_path: "/tmp/App.module.css".to_string(),
                style_source:
                    r#"@value alias as brand from "./tokens.module.css"; .btn { color: brand; }"#
                        .to_string(),
            },
        ],
        &["value-resolution".to_string(), "print-css".to_string()],
        &[],
    )?;

    assert_eq!(summary.product, "omena-query.consumer-build-style-source");
    assert_eq!(summary.execution.output_css, r#" .btn { color: #fff; }"#);
    assert_eq!(summary.execution.mutation_count, 2);
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
    assert_eq!(
        summary.schema_version_policy.product,
        "omena-query.schema-version-policy"
    );
    assert!(
        summary
            .schema_version_policy
            .migration_policy
            .contains(&"breaking payload changes require a new numeric schemaVersion and explicit migration adapter")
    );
    assert!(summary.schema_version_checks.iter().any(|check| {
        check.requested_version.as_deref() == Some("0")
            && check.status == "current"
            && check.accepted
    }));
    assert!(summary.schema_version_checks.iter().any(|check| {
        check.requested_version.as_deref() == Some("V0")
            && check.status == "labelOnlyVersionRejected"
            && !check.accepted
    }));
    assert!(summary.schema_version_checks.iter().any(|check| {
        check.requested_version.as_deref() == Some("1")
            && check.status == "unsupportedVersion"
            && !check.accepted
    }));
    assert!(summary.schema_version_checks.iter().any(|check| {
        check.requested_version.is_none() && check.status == "missingVersion" && !check.accepted
    }));

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
            command.command == "input-expression-domain-call-site-flow-analysis"
        })
    );
    assert!(
        summary.runner_commands.iter().any(|command| {
            command.command == "input-expression-domain-provenance-explanations"
        })
    );
    assert!(
        summary.runner_commands.iter().any(|command| {
            command.command == "input-expression-domain-reduced-product-iteration"
        })
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
            .contains(&"expressionDomainCallSiteFlowAnalysisRunner")
    );
    assert!(
        summary
            .adapter_readiness
            .contains(&"expressionDomainProvenanceExplanationRunner")
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
fn classifies_omena_query_schema_versions_before_execution() {
    let policy = summarize_omena_query_schema_version_policy();
    assert_eq!(policy.schema_version, "0");
    assert_eq!(policy.accepted_versions, vec!["0"]);
    assert!(policy.deprecated_versions.is_empty());
    assert_eq!(
        policy.rejected_version_policy,
        "rejectUnknownVersionsBeforeExecution"
    );
    assert_eq!(
        policy.compatibility_gate,
        "rust/omena-query/adapter-capabilities"
    );

    let current = check_omena_query_schema_version(Some("0"));
    assert!(current.accepted);
    assert_eq!(current.status, "current");
    assert_eq!(current.migration_action, "executeCurrentFacade");

    let label = check_omena_query_schema_version(Some("V0"));
    assert!(!label.accepted);
    assert_eq!(label.status, "labelOnlyVersionRejected");
    assert_eq!(label.migration_action, "sendNumericSchemaVersion");

    let future = check_omena_query_schema_version(Some("1"));
    assert!(!future.accepted);
    assert_eq!(future.status, "unsupportedVersion");
    assert_eq!(future.migration_action, "rejectBeforeExecution");

    let missing = check_omena_query_schema_version(None);
    assert!(!missing.accepted);
    assert_eq!(missing.status, "missingVersion");
    assert_eq!(missing.migration_action, "rejectBeforeExecution");
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
    assert_eq!(
        batch.cross_file_summary.product,
        "omena-query.cross-file-summary"
    );
    assert_eq!(batch.cross_file_summary.status, "summaryEdgeSeed");
    assert_eq!(batch.cross_file_summary.summary_edge_count, 15);
    assert_eq!(batch.cross_file_summary.summary_hash.len(), 16);
    assert!(batch.cross_file_summary.edges.iter().all(|edge| {
        edge.linear_provenance.semiring_identifier() == "lin01"
            && edge.linear_provenance.semiring_identifier == "lin01"
            && edge.linear_provenance.labels() == edge.provenance
    }));
    assert!(
        batch
            .cross_file_summary
            .capabilities
            .css_modules_composes_edges_ready
    );
    assert!(
        batch
            .cross_file_summary
            .capabilities
            .css_modules_value_edges_ready
    );
    assert!(
        batch
            .cross_file_summary
            .capabilities
            .css_modules_icss_edges_ready
    );
    assert!(
        batch
            .cross_file_summary
            .capabilities
            .linear_provenance_ready
    );

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
    let composes_summary = batch
        .cross_file_summary
        .edges
        .iter()
        .find(|edge| edge.edge_kind == "cssModulesComposesImport");
    assert!(composes_summary.is_some());
    let Some(composes_summary) = composes_summary else {
        return;
    };
    assert_eq!(
        composes_summary.target_path.as_deref(),
        Some("/tmp/base.module.scss")
    );
    assert_eq!(
        composes_summary.source.as_deref(),
        Some("./base.module.scss")
    );
    assert_eq!(composes_summary.target_names, vec!["base"]);
    assert_eq!(
        composes_summary.provenance,
        vec![
            "omena-query.css-modules-cross-file-resolution",
            "omena-parser.css-module-composes-facts",
        ]
    );
    assert_eq!(
        composes_summary.linear_provenance.labels(),
        composes_summary.provenance
    );
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
    assert!(batch.cross_file_summary.edges.iter().any(|edge| {
        edge.edge_kind == "cssModulesComposesClosure"
            && edge.from_kind == "style"
            && edge.from_path == "/tmp/App.module.scss"
            && edge.target_kind == Some("style")
            && edge.target_path.as_deref() == Some("/tmp/base.module.scss")
            && edge.owner_selector_name.as_deref() == Some("btn")
            && edge.remote_name.as_deref() == Some("foundation")
            && edge.status == "reachable"
    }));

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
fn cross_file_summary_edges_are_equivalent_to_resolution_products() {
    let input = sample_input();
    let style_sources = [
        (
            "/tmp/base.module.scss",
            ".foundation { display: block; } .base { composes: foundation; --brand: red; }",
        ),
        (
            "/tmp/tokens.module.scss",
            "@value primary: red; :export { raw: red; exported: raw; }",
        ),
        ("/tmp/_palette.scss", "$tone: red;"),
        ("/tmp/_theme.scss", "@forward \"./palette\" show $tone;"),
        (
            "/tmp/App.module.scss",
            "@use \"./theme\"; @value primary as localPrimary from \"./tokens.module.scss\"; @value accent: localPrimary; :import(\"./tokens.module.scss\") { imported: exported; } :export { forwarded: imported; } .btn { composes: base from \"./base.module.scss\"; color: var(--brand); }",
        ),
    ];
    let batch =
        summarize_omena_query_style_semantic_graph_batch_from_sources(style_sources, &input);
    let summary = &batch.cross_file_summary;

    let custom_property_reference_count = style_sources
        .iter()
        .filter_map(|(path, source)| summarize_omena_query_style_document(path, source))
        .map(|summary| summary.custom_property_ref_names.len())
        .sum::<usize>();
    let expected_summary_edge_count = batch.css_modules_resolution.edges.len()
        + batch.css_modules_resolution.composes_closure_edges.len()
        + batch.css_modules_resolution.value_closure_edges.len()
        + batch.css_modules_resolution.icss_closure_edges.len()
        + batch.sass_module_resolution.edges.len()
        + batch.sass_module_resolution.graph_closure_edges.len()
        + custom_property_reference_count;

    assert_eq!(summary.summary_edge_count, expected_summary_edge_count);
    assert_eq!(summary.edges.len(), expected_summary_edge_count);

    for edge in &batch.css_modules_resolution.edges {
        let edge_kind = match edge.import_kind {
            "composes" => "cssModulesComposesImport",
            "value" => "cssModulesValueImport",
            "icss" => "cssModulesIcssImport",
            _ => "cssModulesImport",
        };
        assert!(
            summary.edges.iter().any(|summary_edge| {
                summary_edge.edge_kind == edge_kind
                    && summary_edge.from_path == edge.from_style_path
                    && summary_edge.source.as_deref() == Some(edge.source.as_str())
                    && summary_edge.target_path == edge.resolved_style_path
                    && summary_edge.target_names == edge.imported_names
                    && summary_edge.status == edge.status
            }),
            "missing CSS Modules import summary edge for {edge_kind} {}",
            edge.source
        );
    }

    for edge in &batch.css_modules_resolution.composes_closure_edges {
        assert!(summary.edges.iter().any(|summary_edge| {
            summary_edge.edge_kind == "cssModulesComposesClosure"
                && summary_edge.from_path == edge.from_style_path
                && summary_edge.target_path.as_deref() == Some(edge.target_style_path.as_str())
                && summary_edge.owner_selector_name.as_deref()
                    == Some(edge.owner_selector_name.as_str())
                && summary_edge.remote_name.as_deref() == Some(edge.target_selector_name.as_str())
                && summary_edge.target_names == vec![edge.target_selector_name.clone()]
                && summary_edge.status == "reachable"
        }));
    }

    for edge in &batch.css_modules_resolution.value_closure_edges {
        assert!(summary.edges.iter().any(|summary_edge| {
            summary_edge.edge_kind == "cssModulesValueClosure"
                && summary_edge.from_path == edge.from_style_path
                && summary_edge.target_path.as_deref() == Some(edge.target_style_path.as_str())
                && summary_edge.local_name.as_deref() == Some(edge.value_name.as_str())
                && summary_edge.remote_name.as_deref() == Some(edge.target_value_name.as_str())
                && summary_edge.target_names == vec![edge.target_value_name.clone()]
                && summary_edge.status == "reachable"
        }));
    }

    for edge in &batch.css_modules_resolution.icss_closure_edges {
        assert!(summary.edges.iter().any(|summary_edge| {
            summary_edge.edge_kind == "cssModulesIcssClosure"
                && summary_edge.from_path == edge.from_style_path
                && summary_edge.target_path.as_deref() == Some(edge.target_style_path.as_str())
                && summary_edge.local_name.as_deref() == Some(edge.name.as_str())
                && summary_edge.remote_name.as_deref() == Some(edge.target_name.as_str())
                && summary_edge.target_names == vec![edge.target_name.clone()]
                && summary_edge.status == "reachable"
        }));
    }

    for edge in &batch.sass_module_resolution.edges {
        assert!(summary.edges.iter().any(|summary_edge| {
            summary_edge.edge_kind == edge.edge_kind
                && summary_edge.from_path == edge.from_style_path
                && summary_edge.source.as_deref() == Some(edge.source.as_str())
                && summary_edge.target_path == edge.resolved_style_path
                && summary_edge.local_name == edge.namespace
                && summary_edge.remote_name == edge.forward_prefix
                && summary_edge.target_names == edge.visibility_filter_names
                && summary_edge.status == edge.status
        }));
    }

    for edge in &batch.sass_module_resolution.graph_closure_edges {
        assert!(summary.edges.iter().any(|summary_edge| {
            summary_edge.edge_kind == "sassModuleGraphClosure"
                && summary_edge.from_path == edge.from_style_path
                && summary_edge.target_path.as_deref() == Some(edge.target_style_path.as_str())
                && summary_edge.local_name == edge.namespace
                && summary_edge.remote_name == edge.forward_prefix
                && summary_edge.target_names == edge.visibility_filter_names
                && summary_edge.status == "reachable"
        }));
    }

    assert_eq!(
        summary
            .edges
            .iter()
            .filter(|edge| edge.edge_kind == "styleDesignTokenReference")
            .count(),
        custom_property_reference_count
    );
    assert!(summary.edges.iter().all(|edge| {
        edge.linear_provenance.semiring_identifier() == "lin01"
            && edge.linear_provenance.labels() == edge.provenance
    }));
}

#[test]
fn style_semantic_graph_batch_cross_file_summary_hash_tracks_edge_changes() {
    let input = sample_input();
    let baseline = summarize_omena_query_style_semantic_graph_batch_from_sources(
        [
            ("/tmp/base.module.scss", ".base { display: block; }"),
            (
                "/tmp/App.module.scss",
                ".btn { composes: base from \"./base.module.scss\"; }",
            ),
        ],
        &input,
    );
    let changed = summarize_omena_query_style_semantic_graph_batch_from_sources(
        [
            ("/tmp/base.module.scss", ".base { display: block; }"),
            (
                "/tmp/App.module.scss",
                ".btn { composes: base from \"./missing.module.scss\"; }",
            ),
        ],
        &input,
    );

    assert_ne!(
        baseline.cross_file_summary.summary_hash,
        changed.cross_file_summary.summary_hash
    );
    assert!(baseline.cross_file_summary.edges.iter().any(|edge| {
        edge.edge_kind == "cssModulesComposesImport"
            && edge.target_path.as_deref() == Some("/tmp/base.module.scss")
            && edge.status == "resolved"
    }));
    assert!(changed.cross_file_summary.edges.iter().any(|edge| {
        edge.edge_kind == "cssModulesComposesImport"
            && edge.target_path.is_none()
            && edge.status == "unresolvedSource"
    }));
}

#[test]
fn cross_file_summary_linear_provenance_serializes_as_strict_superset()
-> Result<(), serde_json::Error> {
    let input = sample_input();
    let batch = summarize_omena_query_style_semantic_graph_batch_from_sources(
        [
            ("/tmp/base.module.scss", ".base { display: block; }"),
            (
                "/tmp/App.module.scss",
                ".btn { composes: base from \"./base.module.scss\"; }",
            ),
        ],
        &input,
    );
    let edge = batch
        .cross_file_summary
        .edges
        .iter()
        .find(|edge| edge.edge_kind == "cssModulesComposesImport")
        .expect("expected CSS Modules composes summary edge");
    let serialized = serde_json::to_value(edge)?;
    let legacy_labels = serialized
        .pointer("/provenance")
        .and_then(|value| value.as_array())
        .expect("legacy provenance vector must stay serialized")
        .iter()
        .map(|value| value.as_str().expect("provenance labels are strings"))
        .collect::<Vec<_>>();
    let typed_labels = serialized
        .pointer("/linearProvenance/terms")
        .and_then(|value| value.as_array())
        .expect("typed linear provenance terms must be serialized")
        .iter()
        .map(|value| {
            value
                .pointer("/label")
                .and_then(|label| label.as_str())
                .expect("linear provenance labels are strings")
        })
        .collect::<Vec<_>>();

    assert_eq!(legacy_labels, edge.provenance);
    assert_eq!(typed_labels, legacy_labels);
    assert_eq!(
        serialized
            .pointer("/linearProvenance/product")
            .and_then(|value| value.as_str()),
        Some("omena-abstract-value.linear-provenance")
    );
    assert_eq!(
        serialized
            .pointer("/linearProvenance/semiringIdentifier")
            .and_then(|value| value.as_str()),
        Some("lin01")
    );
    assert_eq!(
        serialized
            .pointer("/linearProvenance/termCount")
            .and_then(|value| value.as_u64()),
        Some(edge.provenance.len() as u64)
    );
    assert!(
        serialized
            .pointer("/linearProvenance/terms")
            .and_then(|value| value.as_array())
            .expect("typed linear provenance terms must be serialized")
            .iter()
            .all(|value| value
                .pointer("/coefficient")
                .and_then(|coefficient| coefficient.as_u64())
                == Some(1))
    );
    Ok(())
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
        &[],
    );
    assert_eq!(source_completion.context_kind, "sourceCssModuleTarget");
    assert_eq!(source_completion.item_count, 1);
    assert_eq!(source_completion.items[0].label, "root");
    assert_eq!(
        source_completion.items[0].ranking_source,
        "targetAndPrefixNarrowing"
    );
    assert!(
        source_completion
            .ready_surfaces
            .contains(&"bridgeAwareSelectorCompletion")
    );
    Ok(())
}

#[test]
fn source_completion_ranking_prefers_value_domain_projection() {
    let candidates = [
        OmenaQueryCompletionCandidateV0 {
            file_uri: "file:///workspace/src/Component.module.scss".to_string(),
            name: "item--large".to_string(),
            kind: "selector",
            range: ParserRangeV0 {
                start: ParserPositionV0 {
                    line: 0,
                    character: 1,
                },
                end: ParserPositionV0 {
                    line: 0,
                    character: 12,
                },
            },
            source: "omenaQueryStyleHoverCandidates",
        },
        OmenaQueryCompletionCandidateV0 {
            file_uri: "file:///workspace/src/Component.module.scss".to_string(),
            name: "item--primary".to_string(),
            kind: "selector",
            range: ParserRangeV0 {
                start: ParserPositionV0 {
                    line: 1,
                    character: 1,
                },
                end: ParserPositionV0 {
                    line: 1,
                    character: 14,
                },
            },
            source: "omenaQueryStyleHoverCandidates",
        },
        OmenaQueryCompletionCandidateV0 {
            file_uri: "file:///workspace/src/Component.module.scss".to_string(),
            name: "item--secondary".to_string(),
            kind: "selector",
            range: ParserRangeV0 {
                start: ParserPositionV0 {
                    line: 2,
                    character: 1,
                },
                end: ParserPositionV0 {
                    line: 2,
                    character: 16,
                },
            },
            source: "omenaQueryStyleHoverCandidates",
        },
    ];
    let completion = super::summarize_omena_query_source_completion_at_position(
        "file:///workspace/src/App.tsx",
        ParserPositionV0 {
            line: 0,
            character: 42,
        },
        &candidates,
        Some("file:///workspace/src/Component.module.scss"),
        Some("item--"),
        &["item--secondary".to_string(), "item--primary".to_string()],
    );

    assert_eq!(completion.context_kind, "sourceCssModuleValueDomainTarget");
    assert_eq!(
        completion
            .items
            .iter()
            .map(|item| item.label.as_str())
            .collect::<Vec<_>>(),
        vec!["item--primary", "item--secondary", "item--large"]
    );
    assert_eq!(
        completion.items[0].ranking_source,
        "valueDomainSelectorProjection"
    );
    assert!(
        completion
            .ready_surfaces
            .contains(&"valueDomainAwareSelectorCompletion")
    );
}

#[test]
fn completion_ranking_uses_query_facts() -> Result<(), &'static str> {
    let source =
        ":root { --alpha: red; }\n.card { --zeta: blue; color: var(--); }\n.next { --omega: red; }";
    let candidates =
        super::summarize_omena_query_style_hover_candidates("Component.module.scss", source)
            .ok_or("style candidates")?;

    let completion = super::summarize_omena_query_style_completion_at_position(
        "file:///workspace/src/Component.module.scss",
        source,
        ParserPositionV0 {
            line: 1,
            character: 35,
        },
        candidates.candidates.as_slice(),
    );

    assert_eq!(completion.context_kind, "styleCustomPropertyReference");
    assert_eq!(
        completion
            .items
            .iter()
            .map(|item| item.label.as_str())
            .collect::<Vec<_>>(),
        vec!["--zeta", "--alpha", "--omega"]
    );
    assert_eq!(
        completion.items[0].ranking_source,
        "sameFileSourceOrderCascade"
    );
    assert!(completion.items[0].sort_text.starts_with("00-"));
    Ok(())
}

#[test]
fn style_extract_code_actions_are_query_owned() {
    let source = ".button { color: #ff0000; margin: 1rem; }";
    let plan = super::summarize_omena_query_style_extract_code_actions(
        "file:///workspace/src/App.module.scss",
        source,
        ParserRangeV0 {
            start: ParserPositionV0 {
                line: 0,
                character: 17,
            },
            end: ParserPositionV0 {
                line: 0,
                character: 24,
            },
        },
    );

    assert_eq!(plan.product, "omena-query.code-actions");
    assert_eq!(plan.file_kind, "style");
    assert_eq!(plan.action_count, 2);
    assert_eq!(
        plan.actions
            .iter()
            .map(|action| action.title.as_str())
            .collect::<Vec<_>>(),
        vec![
            "Extract CSS custom property '--extracted-color'",
            "Extract @value 'extractedColor'",
        ]
    );
    assert_eq!(plan.actions[0].kind, "refactor.extract");
    assert_eq!(plan.actions[0].source, "omenaQueryStyleExtractCodeActions");
    assert_eq!(
        plan.actions[0]
            .edits
            .iter()
            .map(|edit| edit.new_text.as_str())
            .collect::<Vec<_>>(),
        vec![
            ":root {\n  --extracted-color: #ff0000;\n}\n\n",
            "var(--extracted-color)"
        ]
    );
    assert_eq!(
        plan.actions[1]
            .edits
            .iter()
            .map(|edit| edit.new_text.as_str())
            .collect::<Vec<_>>(),
        vec!["@value extractedColor: #ff0000;\n\n", "extractedColor"]
    );
    assert!(plan.ready_surfaces.contains(&"styleExtractRefactorActions"));
}

#[test]
fn style_inline_code_actions_are_query_owned() {
    let source = ".button {\n  composes: base;\n  color: red;\n}\n.base {\n  color: blue;\n  margin: 1rem;\n}";
    let style_uri = "file:///workspace/src/App.module.scss";
    let plan = super::summarize_omena_query_style_inline_code_actions(
        style_uri,
        &[OmenaQueryStyleSourceInputV0 {
            style_path: style_uri.to_string(),
            style_source: source.to_string(),
        }],
        ParserRangeV0 {
            start: ParserPositionV0 {
                line: 1,
                character: 12,
            },
            end: ParserPositionV0 {
                line: 1,
                character: 16,
            },
        },
        &[],
    );

    assert_eq!(plan.product, "omena-query.code-actions");
    assert_eq!(plan.file_kind, "style");
    assert_eq!(plan.action_count, 1);
    assert_eq!(plan.actions[0].title, "Inline composed class 'base'");
    assert_eq!(plan.actions[0].kind, "refactor.inline");
    assert_eq!(plan.actions[0].source, "omenaQueryStyleInlineCodeActions");
    assert_eq!(
        plan.actions[0].edits[0].range,
        ParserRangeV0 {
            start: ParserPositionV0 {
                line: 1,
                character: 2,
            },
            end: ParserPositionV0 {
                line: 1,
                character: 17,
            },
        }
    );
    assert_eq!(
        plan.actions[0].edits[0].new_text,
        "color: blue;\n  margin: 1rem;"
    );
    assert!(plan.ready_surfaces.contains(&"styleInlineRefactorActions"));
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
fn read_cascade_at_position_uses_exact_conditional_context() {
    let source = r#":root { --surface: base; }
@media (min-width: 40rem) {
  :root { --surface: wide; }
  .button { color: var(--surface); }
}
@media (max-width: 20rem) {
  :root { --surface: narrow; }
}
"#;
    let cascade = super::read_omena_query_cascade_at_position(
        "Component.module.css",
        source,
        &sample_input(),
        ParserPositionV0 {
            line: 3,
            character: 25,
        },
    );
    assert!(cascade.is_some());
    let Some(cascade) = cascade else {
        return;
    };

    assert_eq!(cascade.status, "resolved");
    assert_eq!(cascade.reference_name.as_deref(), Some("--surface"));
    assert_eq!(cascade.winner_declaration_source_order, Some(1));
    assert_eq!(cascade.candidate_declaration_count, 2);
    assert_eq!(cascade.shadowed_declaration_source_orders, vec![0]);
    assert_eq!(
        cascade
            .reference_custom_property_fixed_point_value
            .as_deref(),
        Some("wide")
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
    let linear_provenance = diagnostic.linear_provenance();
    assert_eq!(
        linear_provenance.labels(),
        vec![
            "omena-query.source-syntax-index",
            "omena-query.style-selector-definitions"
        ]
    );
}

#[cfg(unix)]
#[test]
fn query_resolves_symlinked_package_style_uri_to_canonical_identity()
-> Result<(), Box<dyn std::error::Error>> {
    let root = std::env::temp_dir().join(format!(
        "omena_query_symlinked_package_identity_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos()
    ));
    let source = root.join("src/App.module.scss");
    let real_package = root.join(".pnpm/@design+tokens@1.0.0/node_modules/@design/tokens");
    let linked_scope = root.join("node_modules/@design");
    let linked_package = linked_scope.join("tokens");
    let style = real_package.join("src/index.scss");
    std::fs::create_dir_all(
        source
            .parent()
            .ok_or_else(|| std::io::Error::other("source"))?,
    )?;
    std::fs::create_dir_all(
        style
            .parent()
            .ok_or_else(|| std::io::Error::other("style"))?,
    )?;
    std::fs::create_dir_all(linked_scope.as_path())?;
    std::fs::write(&source, r#"@use "@design/tokens" as tokens;"#)?;
    std::fs::write(
        real_package.join("package.json"),
        r#"{"sass":"src/index.scss"}"#,
    )?;
    std::fs::write(&style, "$brand: #fff;")?;
    std::os::unix::fs::symlink(real_package.as_path(), linked_package.as_path())?;

    let resolved_uri = super::resolve_omena_query_style_uri_for_specifier(
        test_file_uri(source.as_path()).as_str(),
        Some(test_file_uri(root.as_path()).as_str()),
        "@design/tokens",
    );
    let expected_uri = test_file_uri(std::fs::canonicalize(style)?.as_path());

    assert_eq!(resolved_uri.as_deref(), Some(expected_uri.as_str()));
    let _ = std::fs::remove_dir_all(root);
    Ok(())
}

#[test]
fn query_resolves_vite_bundler_alias_style_uri() -> Result<(), Box<dyn std::error::Error>> {
    let root = std::env::temp_dir().join(format!(
        "omena_query_vite_alias_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos()
    ));
    let source = root.join("src/App.tsx");
    let style = root.join("src/styles/Button.module.scss");
    std::fs::create_dir_all(
        style
            .parent()
            .ok_or_else(|| std::io::Error::other("style"))?,
    )?;
    std::fs::write(&source, "")?;
    std::fs::write(&style, ".button { color: red; }")?;
    std::fs::write(
        root.join("vite.config.ts"),
        r#"export default { resolve: { alias: { "@styles": "./src/styles" } } };"#,
    )?;

    let resolved_uri = super::resolve_omena_query_style_uri_for_specifier(
        test_file_uri(source.as_path()).as_str(),
        Some(test_file_uri(root.as_path()).as_str()),
        "@styles/Button.module.scss",
    );
    let expected_uri = test_file_uri(std::fs::canonicalize(style)?.as_path());

    assert_eq!(resolved_uri.as_deref(), Some(expected_uri.as_str()));
    let _ = std::fs::remove_dir_all(root);
    Ok(())
}

fn test_file_uri(path: &std::path::Path) -> String {
    format!("file://{}", path.to_string_lossy())
}
