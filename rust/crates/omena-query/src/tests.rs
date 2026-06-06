#[cfg(feature = "lawvere-trace")]
use super::execute_omena_query_transform_passes_from_source_with_lawvere_trace;
use super::{
    EngineInputV2, IncrementalGraphInputV0, IncrementalNodeInputV0, IncrementalRevisionV0,
    OmenaQueryCanonicalFormInput, OmenaQueryExpressionDomainFlowRuntimeV0,
    OmenaQueryStylePackageManifestV0, ParserPositionV0, SelectorProjectionCertaintyV0,
    StyleAnalysisInputV2, StyleDocumentV2, attach_omena_query_consumer_build_bundle_summary,
    attach_omena_query_consumer_build_source_map_v3,
    attach_omena_query_consumer_build_source_map_v3_with_sources,
    execute_omena_query_consumer_build_style_source,
    execute_omena_query_consumer_build_style_source_with_engine_input_context,
    execute_omena_query_consumer_build_style_sources,
    execute_omena_query_consumer_build_style_sources_with_context,
    execute_omena_query_consumer_build_style_sources_with_context_and_resolution_inputs,
    execute_omena_query_transform_passes_from_source, list_omena_query_transform_pass_summaries,
    plan_incremental_computation_with_priority_inputs, snapshot_from_graph_input,
    summarize_omena_query_analyzed_graph, summarize_omena_query_boundary,
    summarize_omena_query_canonical_form, summarize_omena_query_custom_property_annotations,
    summarize_omena_query_design_system_minimum_description,
    summarize_omena_query_expression_domain_call_site_flow_analysis,
    summarize_omena_query_expression_domain_control_flow_analysis,
    summarize_omena_query_expression_domain_flow_analysis,
    summarize_omena_query_expression_domain_incremental_flow_analysis,
    summarize_omena_query_expression_domain_provenance_explanations,
    summarize_omena_query_expression_domain_reduced_product_iteration,
    summarize_omena_query_expression_domain_selector_projection,
    summarize_omena_query_expression_semantics_query_fragments, summarize_omena_query_fast_facts,
    summarize_omena_query_fragment_bundle,
    summarize_omena_query_omena_parser_css_modules_intermediate,
    summarize_omena_query_omena_parser_lex, summarize_omena_query_omena_parser_style_facts,
    summarize_omena_query_selector_usage_query_fragments,
    summarize_omena_query_source_resolution_query_fragments, summarize_omena_query_style_document,
    summarize_omena_query_style_edit_distance,
    summarize_omena_query_style_edit_distance_cascade_margin_bridge,
    summarize_omena_query_transform_context_from_engine_input,
    summarize_omena_query_transform_context_from_sources,
    summarize_omena_query_transform_context_from_sources_with_resolution_inputs,
    summarize_omena_query_transform_plan_from_source,
    summarize_omena_query_transform_plan_from_target_query,
};
use crate::{
    ModelClassV0, OmenaQueryStyleResolutionInputsV0, OmenaQueryStyleSourceInputV0,
    OmenaQueryTargetFeatureSupportV0, OmenaQueryTargetTransformOptionsV0,
    OmenaQueryTransformDesignTokenRouteV0, OmenaQueryTransformExecutionContextV0,
    OmenaQueryTransformPrintMode, OmenaQueryTransformPrintOptionsV0,
    OmenaQueryTsconfigPathMappingV0, default_omena_query_transform_print_options,
    modern_omena_query_target_feature_support,
    summarize_omena_query_sass_module_cross_file_resolution_for_workspace,
};
use omena_cascade::{CascadeKey, CascadeLevel, CascadeMarginV0, LayerRank, Specificity};

mod cascade_queries;
mod consumer_reachability;
mod consumer_surfaces;
mod cross_file_summary;
mod dynamic_classname;
mod expression_domain;
mod provider_queries;
mod runtime_contracts;
mod source_surfaces;
mod style_diagnostics;
mod style_semantic_graph;
mod stylesheet_evaluation;
mod support;
mod transform_facade;

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
            .ready_surfaces
            .contains(&"polynomialProvenanceDiagnostics")
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
            .contains(&"omena-abstract-value.polynomial-provenance")
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
            .contains(&"omena-query-checker-orchestrator.cascade-gate")
    );
    assert!(
        summary
            .delegated_fragment_products
            .contains(&"omena-query-transform-runner.boundary")
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
    assert!(
        summary
            .ready_surfaces
            .contains(&"queryCheckerOrchestratorBoundary")
    );
    assert!(
        summary
            .ready_surfaces
            .contains(&"queryTransformRunnerBoundary")
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
fn summarizes_design_system_minimum_description_in_bits() {
    let summary = summarize_omena_query_design_system_minimum_description(
        "file:///workspace/tokens.module.css",
        "len42-sum100",
        3,
        5,
        // 4 distinct value symbols observed 4/4/4/4 times: uniform, max entropy.
        &[4, 4, 4, 4],
    );

    assert_eq!(summary.schema_version, "0");
    assert_eq!(
        summary.product,
        "omena-query.design-system-minimum-description"
    );
    assert_eq!(summary.unit, "bit");
    assert_eq!(summary.layer_marker, "mdl-bits");
    assert_eq!(summary.feature_gate, "mdl");
    assert_eq!(
        summary.model_bits + summary.residual_bits,
        summary.total_bits
    );
    assert_eq!(summary.semiring_instance, "tropical");
    // 4-symbol alphabet -> log2(4)=2 bits/rule * 3 rules = 6 model bits.
    assert_eq!(summary.model_bits, 6.0);
    // Uniform 4-way distribution -> H=2 bits * 5 observations = 10 residual bits.
    assert_eq!(summary.residual_bits, 10.0);
    assert_eq!(summary.model_class, ModelClassV0::TwoPartMultinomial);
}

#[test]
fn mdl_total_bits_is_not_the_count_sum_and_tracks_distribution_entropy() {
    // Two inputs with IDENTICAL rule_count + observation_count but different value
    // distributions. If MDL were the degenerate `rule_count + observation_count`
    // sum, both would equal 12 and these assertions would fail. The real entropy/
    // log code length distinguishes them.
    let uniform = summarize_omena_query_design_system_minimum_description(
        "file:///workspace/uniform.module.css",
        "hash",
        8,
        4,
        // Maximum-entropy 4-way uniform value distribution.
        &[3, 3, 3, 3],
    );
    let peaked = summarize_omena_query_design_system_minimum_description(
        "file:///workspace/peaked.module.css",
        "hash",
        8,
        4,
        // Same alphabet size + same totals, but one dominant symbol: low entropy.
        &[9, 1, 1, 1],
    );

    // Same alphabet size => same model_bits; only residual entropy differs.
    assert_eq!(uniform.model_bits, peaked.model_bits);
    // Degenerate count-sum would be 8 + 4 = 12 for both. The real code is not.
    assert_ne!(uniform.total_bits, 12.0);
    assert_ne!(peaked.total_bits, 12.0);
    // The more-compressible (peaked) distribution costs strictly fewer bits.
    assert!(peaked.total_bits < uniform.total_bits);
}

#[test]
fn summarizes_mdl_canonical_form_as_strict_superset_contract() {
    let form = summarize_omena_query_canonical_form(OmenaQueryCanonicalFormInput {
        pass_id: "selector-is-where-compression",
        before: ".a:is(.b){}".into(),
        canonical_after: ".a.b{}".into(),
        fallback_after: ".a.b{}".into(),
        mdl_bits: 3.0,
        ast_size_bits: 5.0,
        iteration_count: 2,
        eclass_count: 4,
        enode_count: 8,
    });

    assert_eq!(form.schema_version, "0");
    assert_eq!(form.product, "omena-query.canonical-form");
    assert_eq!(form.layer_marker, "mdl-bits");
    assert_eq!(form.feature_gate, "mdl");
    assert_eq!(form.unit, "bit");
    assert!(form.canonical_matches_fallback);
    assert_eq!(form.bits_saved_vs_fallback, 2.0);
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
fn exposes_style_edit_distance_and_cascade_margin_bridge_witness() {
    let baseline = r#"
      .card { color: red; }
    "#;
    let changed = r#"
      @use "tokens";
      .card { color: red; }
      .cardPrimary { color: blue; }
    "#;

    let distance = summarize_omena_query_style_edit_distance(
        "Card.module.scss",
        baseline,
        "Card.module.scss",
        changed,
    );
    assert_eq!(distance.schema_version, "0");
    assert_eq!(distance.product, "omena-query.style-edit-distance");
    assert_eq!(distance.tier, "fastFactsAnalyzedGraphEditDistanceV0");
    assert_eq!(
        distance.metric_kind,
        "absoluteCountDeltaOverFastFactsAndAnalyzedGraph"
    );
    assert_eq!(distance.claim_level, "researchStagedMetricSubstrate");
    assert!(!distance.public_safety_claim_ready);
    assert_eq!(distance.selector_delta, 1);
    assert_eq!(distance.module_edge_delta, 1);
    assert!(distance.total_distance >= 2);

    let winner_key = CascadeKey::new(
        CascadeLevel::AuthorNormal,
        LayerRank(0),
        1,
        Specificity::new(0, 1, 0),
        2,
    );
    let challenger_key = CascadeKey::new(
        CascadeLevel::AuthorNormal,
        LayerRank(0),
        1,
        Specificity::new(0, 1, 0),
        1,
    );
    let margin = CascadeMarginV0 {
        schema_version: "0",
        product: "omena-cascade.margin",
        margin_kind: "lexicographicCascadeKeyDelta",
        winner_declaration_id: "later".to_string(),
        challenger_declaration_id: Some("earlier".to_string()),
        dominant_axis: "sourceOrder",
        signed_distance: 1,
        winner_key,
        challenger_key: Some(challenger_key),
        calibration_stage: "schemaOnlyUncalibrated",
        public_safety_claim_ready: false,
    };

    let bridge =
        summarize_omena_query_style_edit_distance_cascade_margin_bridge(&distance, &margin);
    assert_eq!(
        bridge.product,
        "omena-query.style-edit-distance-cascade-margin-bridge"
    );
    assert_eq!(bridge.bridge_kind, "checkedEmpiricalLipschitzWitness");
    assert_eq!(bridge.claim_level, "fixtureWitnessOnly");
    assert!(!bridge.theorem_claimed);
    assert_eq!(bridge.lipschitz_constant_name, "K_A");
    assert_eq!(bridge.lipschitz_constant, Some(1));
    assert_eq!(bridge.cascade_margin_abs_distance, 1);
    assert!(bridge.checked);
    assert!(!bridge.public_safety_claim_ready);
    assert_eq!(
        bridge.incremental_priority_input.product,
        "omena-incremental.edit-distance-priority-input"
    );
    assert_eq!(
        bridge.incremental_priority_input.feature_gate,
        "incremental-edit-distance-priority-v0"
    );
    assert_eq!(
        bridge.incremental_priority_input.claim_level,
        "fixtureWitnessMetricInput"
    );
    assert!(!bridge.incremental_priority_input.theorem_claimed);
    assert_eq!(
        bridge.incremental_priority_input.node_id,
        "Card.module.scss"
    );
    assert_eq!(
        bridge.incremental_priority_input.edit_distance_total,
        distance.total_distance
    );

    let previous = IncrementalGraphInputV0 {
        revision: IncrementalRevisionV0 { value: 1 },
        nodes: vec![IncrementalNodeInputV0 {
            id: "Card.module.scss".to_string(),
            digest: "card:v1".to_string(),
            dependency_ids: Vec::new(),
        }],
    };
    let previous_snapshot = snapshot_from_graph_input(&previous);
    let next = IncrementalGraphInputV0 {
        revision: IncrementalRevisionV0 { value: 2 },
        nodes: vec![IncrementalNodeInputV0 {
            id: "Card.module.scss".to_string(),
            digest: "card:v2".to_string(),
            dependency_ids: Vec::new(),
        }],
    };
    let plan = plan_incremental_computation_with_priority_inputs(
        &next,
        Some(&previous_snapshot),
        std::slice::from_ref(&bridge.incremental_priority_input),
    );
    assert_eq!(
        plan.invalidation_priority_plan.product,
        "omena-incremental.invalidation-priority-plan"
    );
    assert_eq!(plan.invalidation_priority_plan.metric_consumed_count, 1);
    assert_eq!(
        plan.invalidation_priority_plan.prioritized_dirty_node_ids,
        vec!["Card.module.scss".to_string()]
    );
    assert_eq!(
        plan.invalidation_priority_plan.entries[0].priority_kind,
        "editDistanceCascadeMarginWeighted"
    );
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
        design_token_routes: vec![OmenaQueryTransformDesignTokenRouteV0 {
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
fn sass_module_resolution_flags_non_default_configured_variables() -> Result<(), String> {
    let sources = vec![
        OmenaQueryStyleSourceInputV0 {
            style_path: "/tmp/tokens.scss".to_string(),
            style_source: "$brand: blue; .base { color: $brand; }".to_string(),
        },
        OmenaQueryStyleSourceInputV0 {
            style_path: "/tmp/App.module.scss".to_string(),
            style_source:
                r#"@use "./tokens" as tokens with ($brand: red); .button { color: tokens.$brand; }"#
                    .to_string(),
        },
    ];

    let resolution = summarize_omena_query_sass_module_cross_file_resolution_for_workspace(
        sources.as_slice(),
        &[],
        &[],
        &[],
    );
    let edge = resolution
        .edges
        .iter()
        .find(|edge| edge.from_style_path == "/tmp/App.module.scss")
        .ok_or_else(|| format!("App should have a resolved Sass module edge: {resolution:?}"))?;

    assert_eq!(
        edge.invalid_configuration_variable_names,
        vec!["brand".to_string()]
    );
    Ok(())
}

#[test]
fn sass_module_resolution_tracks_repeated_source_configuration_per_rule() -> Result<(), String> {
    let sources = vec![
        OmenaQueryStyleSourceInputV0 {
            style_path: "/tmp/tokens.scss".to_string(),
            style_source: "$brand: blue !default; .base { color: $brand; }".to_string(),
        },
        OmenaQueryStyleSourceInputV0 {
            style_path: "/tmp/App.module.scss".to_string(),
            style_source: r#"@use "./tokens" as redTokens with ($brand: red); @use "./tokens" as blueTokens with ($brand: blue);"#.to_string(),
        },
    ];

    let resolution = summarize_omena_query_sass_module_cross_file_resolution_for_workspace(
        sources.as_slice(),
        &[],
        &[],
        &[],
    );
    let signatures = resolution
        .edges
        .iter()
        .filter(|edge| edge.from_style_path == "/tmp/App.module.scss" && edge.source == "./tokens")
        .map(|edge| edge.configuration_signature.as_str())
        .collect::<Vec<_>>();
    let rule_ordinals = resolution
        .edges
        .iter()
        .filter(|edge| edge.from_style_path == "/tmp/App.module.scss" && edge.source == "./tokens")
        .map(|edge| edge.rule_ordinal)
        .collect::<Vec<_>>();

    assert!(
        signatures
            .iter()
            .any(|signature| signature.contains("brand=3:red")),
        "{resolution:?}"
    );
    assert!(
        signatures
            .iter()
            .any(|signature| signature.contains("brand=4:blue")),
        "{resolution:?}"
    );
    assert_eq!(rule_ordinals, vec![0, 1], "{resolution:?}");
    Ok(())
}

#[test]
fn sass_module_resolution_tracks_repeated_forward_configuration_per_rule() -> Result<(), String> {
    let sources = vec![
        OmenaQueryStyleSourceInputV0 {
            style_path: "/tmp/tokens.scss".to_string(),
            style_source: "$brand: blue !default; .base { color: $brand; }".to_string(),
        },
        OmenaQueryStyleSourceInputV0 {
            style_path: "/tmp/theme.scss".to_string(),
            style_source: r#"@forward "./tokens" with ($brand: red); @forward "./tokens" with ($brand: blue);"#.to_string(),
        },
        OmenaQueryStyleSourceInputV0 {
            style_path: "/tmp/App.module.scss".to_string(),
            style_source: r#"@use "./theme" as theme;"#.to_string(),
        },
    ];

    let resolution = summarize_omena_query_sass_module_cross_file_resolution_for_workspace(
        sources.as_slice(),
        &[],
        &[],
        &[],
    );
    let signatures = resolution
        .edges
        .iter()
        .filter(|edge| edge.from_style_path == "/tmp/theme.scss" && edge.source == "./tokens")
        .map(|edge| edge.configuration_signature.as_str())
        .collect::<Vec<_>>();
    let rule_ordinals = resolution
        .edges
        .iter()
        .filter(|edge| edge.from_style_path == "/tmp/theme.scss" && edge.source == "./tokens")
        .map(|edge| edge.rule_ordinal)
        .collect::<Vec<_>>();

    assert!(
        signatures
            .iter()
            .any(|signature| signature.contains("brand=3:red")),
        "{resolution:?}"
    );
    assert!(
        signatures
            .iter()
            .any(|signature| signature.contains("brand=4:blue")),
        "{resolution:?}"
    );
    assert_eq!(rule_ordinals, vec![0, 1], "{resolution:?}");
    Ok(())
}

#[test]
fn sass_module_graph_closure_preserves_parallel_forward_configurations() -> Result<(), String> {
    let sources = vec![
        OmenaQueryStyleSourceInputV0 {
            style_path: "/tmp/tokens.scss".to_string(),
            style_source: "$brand: blue !default; .base { color: $brand; }".to_string(),
        },
        OmenaQueryStyleSourceInputV0 {
            style_path: "/tmp/theme.scss".to_string(),
            style_source: r#"@forward "./tokens" with ($brand: red); @forward "./tokens" with ($brand: blue);"#.to_string(),
        },
        OmenaQueryStyleSourceInputV0 {
            style_path: "/tmp/App.module.scss".to_string(),
            style_source: r#"@use "./theme" as theme;"#.to_string(),
        },
    ];

    let resolution = summarize_omena_query_sass_module_cross_file_resolution_for_workspace(
        sources.as_slice(),
        &[],
        &[],
        &[],
    );
    let signatures = resolution
        .graph_closure_edges
        .iter()
        .filter(|edge| {
            edge.from_style_path == "/tmp/App.module.scss"
                && edge.target_style_path == "/tmp/tokens.scss"
        })
        .map(|edge| edge.configuration_signature.as_str())
        .collect::<Vec<_>>();

    assert!(
        signatures
            .iter()
            .any(|signature| signature.contains("brand=3:red")),
        "{resolution:?}"
    );
    assert!(
        signatures
            .iter()
            .any(|signature| signature.contains("brand=4:blue")),
        "{resolution:?}"
    );
    Ok(())
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
fn applies_scss_use_configuration_to_forwarded_module_instance() {
    let summary = summarize_omena_query_transform_context_from_sources(
        "/tmp/App.module.scss",
        [
            (
                "/tmp/tokens.scss",
                "$brand: blue !default; .base { color: $brand; }",
            ),
            ("/tmp/theme.scss", r#"@forward "./tokens";"#),
            (
                "/tmp/App.module.scss",
                r#"@use "./theme" as theme with ($brand: red); .button { color: theme.$brand; }"#,
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
    assert!(!evaluated_css.contains(".base { color: blue; }"));
    assert!(evaluated_css.contains(".button { color: red; }"));
}

#[test]
fn applies_scss_use_configuration_to_prefixed_forwarded_module_instance() {
    let summary = summarize_omena_query_transform_context_from_sources(
        "/tmp/App.module.scss",
        [
            (
                "/tmp/tokens.scss",
                "$brand: blue !default; .base { color: $brand; }",
            ),
            ("/tmp/theme.scss", r#"@forward "./tokens" as token-*;"#),
            (
                "/tmp/App.module.scss",
                r#"@use "./theme" as theme with ($token-brand: red); .button { color: theme.$token-brand; }"#,
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
    assert!(!evaluated_css.contains(".base { color: blue; }"));
    assert!(evaluated_css.contains(".button { color: red; }"));
}

#[test]
fn applies_forward_default_configuration_until_downstream_override() {
    let no_override_summary = summarize_omena_query_transform_context_from_sources(
        "/tmp/App.module.scss",
        [
            (
                "/tmp/tokens.scss",
                "$brand: blue !default; .base { color: $brand; }",
            ),
            (
                "/tmp/theme.scss",
                r#"@forward "./tokens" with ($brand: red !default);"#,
            ),
            (
                "/tmp/App.module.scss",
                r#"@use "./theme" as theme; .button { color: theme.$brand; }"#,
            ),
        ],
        &[],
    );
    let override_summary = summarize_omena_query_transform_context_from_sources(
        "/tmp/App.module.scss",
        [
            (
                "/tmp/tokens.scss",
                "$brand: blue !default; .base { color: $brand; }",
            ),
            (
                "/tmp/theme.scss",
                r#"@forward "./tokens" with ($brand: red !default);"#,
            ),
            (
                "/tmp/App.module.scss",
                r#"@use "./theme" as theme with ($brand: green); .button { color: theme.$brand; }"#,
            ),
        ],
        &[],
    );

    let no_override_css = no_override_summary
        .context
        .scss_module_evaluation
        .as_ref()
        .map(|evaluation| evaluation.evaluated_css.as_str())
        .unwrap_or_default();
    assert!(no_override_css.contains(".base { color: red; }"));
    assert!(no_override_css.contains(".button { color: red; }"));

    let override_css = override_summary
        .context
        .scss_module_evaluation
        .as_ref()
        .map(|evaluation| evaluation.evaluated_css.as_str())
        .unwrap_or_default();
    assert!(override_css.contains(".base { color: green; }"));
    assert!(override_css.contains(".button { color: green; }"));
    assert!(!override_css.contains(".base { color: red; }"));
}

#[test]
fn sass_module_resolution_flags_downstream_configuration_after_non_default_forward()
-> Result<(), String> {
    let sources = vec![
        OmenaQueryStyleSourceInputV0 {
            style_path: "/tmp/tokens.scss".to_string(),
            style_source: "$brand: blue !default; .base { color: $brand; }".to_string(),
        },
        OmenaQueryStyleSourceInputV0 {
            style_path: "/tmp/theme.scss".to_string(),
            style_source: r#"@forward "./tokens" with ($brand: red);"#.to_string(),
        },
        OmenaQueryStyleSourceInputV0 {
            style_path: "/tmp/App.module.scss".to_string(),
            style_source: r#"@use "./theme" as theme with ($brand: green);"#.to_string(),
        },
    ];

    let resolution = summarize_omena_query_sass_module_cross_file_resolution_for_workspace(
        sources.as_slice(),
        &[],
        &[],
        &[],
    );
    let edge = resolution
        .edges
        .iter()
        .find(|edge| edge.from_style_path == "/tmp/App.module.scss")
        .ok_or_else(|| format!("App should have a resolved Sass module edge: {resolution:?}"))?;

    assert_eq!(
        edge.invalid_configuration_variable_names,
        vec!["brand".to_string()]
    );
    Ok(())
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
fn resolves_transform_context_style_references_through_tsconfig_aliases() {
    let resolution_inputs = OmenaQueryStyleResolutionInputsV0 {
        package_manifests: vec![],
        tsconfig_path_mappings: vec![OmenaQueryTsconfigPathMappingV0 {
            base_path: "/workspace".to_string(),
            pattern: "@styles/*".to_string(),
            target_patterns: vec!["src/styles/*".to_string()],
        }],
        bundler_path_mappings: vec![],
    };

    let scss_summary = summarize_omena_query_transform_context_from_sources_with_resolution_inputs(
        "/workspace/src/components/App.module.scss",
        [
            (
                "/workspace/src/styles/_tokens.scss",
                "$brand: blue !default; .base { color: $brand; }",
            ),
            (
                "/workspace/src/components/App.module.scss",
                r#"@use "@styles/tokens" as tokens with ($brand: red); .button { color: tokens.$brand; }"#,
            ),
        ],
        &resolution_inputs,
    );
    let evaluated_css = scss_summary
        .context
        .scss_module_evaluation
        .as_ref()
        .map(|evaluation| evaluation.evaluated_css.as_str())
        .unwrap_or_default();
    assert!(evaluated_css.contains(".base { color: red; }"));
    assert!(evaluated_css.contains(".button { color: red; }"));
    assert!(!evaluated_css.contains(".base { color: blue; }"));

    let value_summary = summarize_omena_query_transform_context_from_sources_with_resolution_inputs(
        "/workspace/src/components/Values.module.css",
        [
            (
                "/workspace/src/styles/tokens.module.css",
                "@value primary: #fff;",
            ),
            (
                "/workspace/src/components/Values.module.css",
                r#"@value primary as brand from "@styles/tokens.module.css"; .btn { color: brand; }"#,
            ),
        ],
        &resolution_inputs,
    );
    assert_eq!(
        value_summary
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

    let composes_summary =
        summarize_omena_query_transform_context_from_sources_with_resolution_inputs(
            "/workspace/src/components/Composes.module.scss",
            [
                (
                    "/workspace/src/styles/base.module.scss",
                    ".foundation { display: block; } .base { composes: foundation; color: red; }",
                ),
                (
                    "/workspace/src/components/Composes.module.scss",
                    r#".btn { composes: base from "@styles/base.module.scss"; color: blue; }"#,
                ),
            ],
            &resolution_inputs,
        );
    assert_eq!(composes_summary.css_module_composes_resolution_count, 1);
    assert_eq!(
        composes_summary.context.css_module_composes_resolutions[0].local_class_name,
        "btn"
    );
    assert_eq!(
        composes_summary.context.css_module_composes_resolutions[0].exported_class_names,
        vec!["base", "btn", "foundation"]
    );
}

#[test]
fn shares_preconfigured_scss_module_instance_with_unconfigured_transitive_forward() {
    let summary = summarize_omena_query_transform_context_from_sources(
        "/tmp/App.module.scss",
        [
            (
                "/tmp/tokens.scss",
                "$brand: blue !default; .base { color: $brand; }",
            ),
            ("/tmp/theme.scss", r#"@forward "./tokens";"#),
            (
                "/tmp/App.module.scss",
                r#"@use "./tokens" as tokens with ($brand: red); @use "./theme" as theme; .button { color: tokens.$brand; border-color: theme.$brand; }"#,
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
    assert!(!evaluated_css.contains(".base { color: blue; }"));
    assert!(evaluated_css.contains(".button { color: red; border-color: red; }"));
}

#[test]
fn preserves_unconfigured_scss_module_instance_before_later_configuration() {
    let summary = summarize_omena_query_transform_context_from_sources(
        "/tmp/App.module.scss",
        [
            (
                "/tmp/tokens.scss",
                "$brand: blue !default; .base { color: $brand; }",
            ),
            ("/tmp/theme.scss", r#"@forward "./tokens";"#),
            (
                "/tmp/App.module.scss",
                r#"@use "./theme" as theme; @use "./tokens" as tokens with ($brand: red); .button { color: tokens.$brand; border-color: theme.$brand; }"#,
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
    assert_eq!(evaluated_css.matches(".base { color: blue; }").count(), 1);
    assert!(!evaluated_css.contains(".base { color: red; }"));
    assert!(evaluated_css.contains(r#"@use "./tokens" as tokens with ($brand: red)"#));
    assert!(evaluated_css.contains(".button { color: tokens.$brand; border-color: blue; }"));
}

#[test]
fn preserves_conflicting_scss_module_configuration_boundary() {
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
    assert!(!evaluated_css.contains(".base { color: blue; }"));
    assert!(evaluated_css.contains(r#"@use "./theme-blue" as blueTheme"#));
    assert!(evaluated_css.contains(".button { color: red; border-color: blueTheme.$brand; }"));
}

#[test]
fn preserves_conflicting_scss_module_configuration_for_repeated_source_use() {
    let summary = summarize_omena_query_transform_context_from_sources(
        "/tmp/App.module.scss",
        [
            (
                "/tmp/tokens.scss",
                "$brand: blue !default; .base { color: $brand; }",
            ),
            (
                "/tmp/App.module.scss",
                r#"@use "./tokens" as redTokens with ($brand: red); @use "./tokens" as blueTokens with ($brand: blue); .button { color: redTokens.$brand; border-color: blueTokens.$brand; }"#,
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
    assert!(!evaluated_css.contains(".base { color: blue; }"));
    assert!(
        evaluated_css.contains(r#"@use "./tokens" as blueTokens with ($brand: blue)"#),
        "{evaluated_css}"
    );
    assert!(evaluated_css.contains(".button { color: red; border-color: blueTokens.$brand; }"));
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
fn consumer_build_resolves_css_modules_values_through_tsconfig_aliases()
-> Result<(), Box<dyn std::error::Error>> {
    let resolution_inputs = OmenaQueryStyleResolutionInputsV0 {
        package_manifests: vec![],
        tsconfig_path_mappings: vec![OmenaQueryTsconfigPathMappingV0 {
            base_path: "/workspace".to_string(),
            pattern: "@styles/*".to_string(),
            target_patterns: vec!["src/styles/*".to_string()],
        }],
        bundler_path_mappings: vec![],
    };
    let summary = execute_omena_query_consumer_build_style_sources_with_context_and_resolution_inputs(
        "/workspace/src/components/App.module.css",
        &[
            OmenaQueryStyleSourceInputV0 {
                style_path: "/workspace/src/styles/tokens.module.css".to_string(),
                style_source: "@value primary: #fff; @value gap: 8px;".to_string(),
            },
            OmenaQueryStyleSourceInputV0 {
                style_path: "/workspace/src/components/App.module.css".to_string(),
                style_source: r#"@value primary as brand, gap from "@styles/tokens.module.css"; .btn { color: brand; margin: gap; }"#.to_string(),
            },
        ],
        &["value-resolution".to_string(), "print-css".to_string()],
        &OmenaQueryTransformExecutionContextV0::default(),
        &resolution_inputs,
    )?;

    assert_eq!(
        summary.execution.output_css,
        r#" .btn { color: #fff; margin: 8px; }"#
    );
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
