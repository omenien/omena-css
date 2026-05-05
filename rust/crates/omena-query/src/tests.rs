use engine_input_producers::{
    ClassExpressionInputV2, EngineInputV2, PositionV2, RangeV2, SourceAnalysisInputV2,
    SourceDocumentV2, StringTypeFactsV2, StyleAnalysisInputV2, StyleDocumentV2, StyleSelectorV2,
    TypeFactEntryV2,
};
use omena_abstract_value::SelectorProjectionCertaintyV0;

use super::{
    OmenaQueryExpressionDomainFlowRuntimeV0, OmenaQueryStylePackageManifestV0,
    SelectedQueryAdapterCapabilitiesV0, summarize_omena_query_boundary,
    summarize_omena_query_expression_domain_control_flow_analysis,
    summarize_omena_query_expression_domain_flow_analysis,
    summarize_omena_query_expression_domain_incremental_flow_analysis,
    summarize_omena_query_expression_domain_selector_projection,
    summarize_omena_query_expression_semantics_canonical_producer_signal,
    summarize_omena_query_expression_semantics_query_fragments,
    summarize_omena_query_fragment_bundle, summarize_omena_query_omena_parser_style_facts,
    summarize_omena_query_selected_query_adapter_capabilities,
    summarize_omena_query_selector_usage_canonical_producer_signal,
    summarize_omena_query_selector_usage_query_fragments,
    summarize_omena_query_source_resolution_canonical_producer_signal,
    summarize_omena_query_source_resolution_query_fragments,
    summarize_omena_query_source_resolution_runtime,
    summarize_omena_query_style_semantic_graph_batch_from_sources,
    summarize_omena_query_style_semantic_graph_batch_from_sources_with_package_manifests,
    summarize_omena_query_style_semantic_graph_from_source,
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
            .contains(&"sourceResolutionRuntimeIndex")
    );
    assert!(
        summary
            .ready_surfaces
            .contains(&"omenaParserStyleFactExtraction")
    );
    assert!(summary.ready_surfaces.contains(&"readCascadeAtPosition"));
    assert!(
        summary
            .cme_coupled_surfaces
            .contains(&"producerQueryFragments")
    );
}

#[test]
fn exposes_omena_parser_style_fact_surface() {
    let summary = summarize_omena_query_omena_parser_style_facts(
        "@use \"tokens\"; $gap: 1rem; .card#main { --space: $gap; }",
        omena_parser::StyleDialect::Scss,
    );

    assert_eq!(summary.schema_version, "0");
    assert_eq!(summary.product, "omena-query.omena-parser-style-facts");
    assert_eq!(summary.dialect, "scss");
    assert_eq!(summary.class_selector_names, vec!["card"]);
    assert_eq!(summary.id_selector_names, vec!["main"]);
    assert!(summary.variable_names.contains(&"$gap".to_string()));
    assert!(
        summary
            .custom_property_names
            .contains(&"--space".to_string())
    );
    assert_eq!(summary.at_rule_names, vec!["@use"]);
    assert_eq!(summary.parser_error_count, 0);
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
        merge.selector_names,
        vec![
            "btn-primary--active".to_string(),
            "btn-secondary--active".to_string()
        ]
    );
    assert_eq!(merge.certainty, SelectorProjectionCertaintyV0::Inferred);
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

$accent: red;
.button { color: var(--brand); }
:root { --brand: blue; }
@include variants($prefix: "tone", $map: ("warm": red));
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
        engine_style_parser::ParserPositionV0 {
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
        engine_style_parser::ParserPositionV0 {
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
        engine_style_parser::ParserPositionV0 {
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
        engine_style_parser::ParserRangeV0 {
            start: engine_style_parser::ParserPositionV0 {
                line: 1,
                character: 20,
            },
            end: engine_style_parser::ParserPositionV0 {
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
        Some(engine_style_parser::ParserRangeV0 {
            start: engine_style_parser::ParserPositionV0 {
                line: 1,
                character: 33,
            },
            end: engine_style_parser::ParserPositionV0 {
                line: 1,
                character: 33,
            },
        })
    );
}

#[test]
fn read_cascade_at_position_is_query_owned() {
    let source = ":root { --surface: white; }\n:root { --surface: black; }\n.button { color: var(--surface); }\n";
    let cascade = super::read_omena_query_cascade_at_position(
        "Component.module.css",
        source,
        &sample_input(),
        engine_style_parser::ParserPositionV0 {
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
    assert_eq!(cascade.candidate_declaration_count, 2);
    assert_eq!(cascade.shadowed_declaration_source_orders, vec![0]);

    let no_reference = super::read_omena_query_cascade_at_position(
        "Component.module.css",
        source,
        &sample_input(),
        engine_style_parser::ParserPositionV0 {
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
fn missing_selector_diagnostics_are_query_owned() {
    let diagnostic = super::summarize_omena_query_missing_selector_diagnostic(
        "file:///workspace/src/App.module.scss",
        ".root {\n}\n",
        "missing",
        engine_style_parser::ParserRangeV0 {
            start: engine_style_parser::ParserPositionV0 {
                line: 2,
                character: 18,
            },
            end: engine_style_parser::ParserPositionV0 {
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
        Some(engine_style_parser::ParserRangeV0 {
            start: engine_style_parser::ParserPositionV0 {
                line: 2,
                character: 0,
            },
            end: engine_style_parser::ParserPositionV0 {
                line: 2,
                character: 0,
            },
        })
    );
}

#[test]
fn source_provider_candidate_resolution_is_query_owned() {
    let source_range = engine_style_parser::ParserRangeV0 {
        start: engine_style_parser::ParserPositionV0 {
            line: 0,
            character: 0,
        },
        end: engine_style_parser::ParserPositionV0 {
            line: 0,
            character: 4,
        },
    };
    let definition_range = engine_style_parser::ParserRangeV0 {
        start: engine_style_parser::ParserPositionV0 {
            line: 1,
            character: 1,
        },
        end: engine_style_parser::ParserPositionV0 {
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
    let source_range = engine_style_parser::ParserRangeV0 {
        start: engine_style_parser::ParserPositionV0 {
            line: 3,
            character: 16,
        },
        end: engine_style_parser::ParserPositionV0 {
            line: 3,
            character: 20,
        },
    };
    let definition_range = engine_style_parser::ParserRangeV0 {
        start: engine_style_parser::ParserPositionV0 {
            line: 0,
            character: 1,
        },
        end: engine_style_parser::ParserPositionV0 {
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
    assert_eq!(
        super::resolve_omena_query_sass_module_use_sources_for_candidate(&sources, None),
        vec!["./reset".to_string()]
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
