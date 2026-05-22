use engine_input_producers::{
    ClassExpressionInputV2, EngineInputV2, PositionV2, RangeV2, SourceAnalysisInputV2,
    SourceDocumentV2, StringTypeFactsV2, StyleAnalysisInputV2, StyleDocumentV2, StyleSelectorV2,
    TypeFactEntryV2,
};
use std::collections::BTreeSet;
#[cfg(unix)]
use std::{fs, path::PathBuf, time::SystemTime};

use super::{
    OmenaResolverBundlerPathAliasMappingV0, OmenaResolverStylePackageManifestV0,
    OmenaResolverTsconfigPathMappingV0, query_omena_resolver_runtime_module,
    query_omena_resolver_source_expression, summarize_omena_resolver_boundary,
    summarize_omena_resolver_canonical_producer_signal,
    summarize_omena_resolver_module_graph_index, summarize_omena_resolver_query_fragments,
    summarize_omena_resolver_runtime_query_boundary,
    summarize_omena_resolver_source_resolution_runtime,
    summarize_omena_resolver_specifier_resolution_runtime,
    summarize_omena_resolver_specifier_resolution_runtime_with_path_mappings,
    summarize_omena_resolver_style_module_resolution,
    summarize_omena_resolver_style_module_resolution_with_path_mappings,
    summarize_omena_resolver_style_module_resolution_with_tsconfig_paths,
};

#[path = "tests/bundler.rs"]
mod bundler;
#[path = "tests/filesystem.rs"]
mod filesystem;
#[path = "tests/package.rs"]
mod package;
#[path = "tests/tsconfig.rs"]
mod tsconfig;

#[test]
fn summarizes_resolver_boundary_over_source_resolution_products() {
    let input = sample_input();
    let summary = summarize_omena_resolver_boundary(&input);

    assert_eq!(summary.schema_version, "0");
    assert_eq!(summary.product, "omena-resolver.boundary");
    assert_eq!(summary.resolver_name, "omena-resolver");
    assert_eq!(summary.input_version, "2");
    assert_eq!(summary.source_resolution_query_count, 2);
    assert_eq!(summary.source_resolution_candidate_count, 2);
    assert_eq!(summary.source_resolution_evaluator_candidate_count, 2);
    assert_eq!(summary.module_graph_module_count, 2);
    assert_eq!(summary.module_graph_source_expression_edge_count, 2);
    assert_eq!(summary.runtime_query_module_count, 2);
    assert_eq!(summary.runtime_query_ready_module_count, 2);
    assert_eq!(summary.source_resolution_runtime_expression_count, 2);
    assert_eq!(
        summary.source_resolution_runtime_resolved_expression_count,
        2
    );
    assert!(
        summary
            .delegated_source_resolution_products
            .contains(&"engine-input-producers.source-resolution-canonical-producer")
    );
    assert!(
        summary
            .resolver_owned_products
            .contains(&"omena-resolver.module-graph-index")
    );
    assert!(
        summary
            .resolver_owned_products
            .contains(&"omena-resolver.runtime-query-boundary")
    );
    assert!(
        summary
            .resolver_owned_products
            .contains(&"omena-resolver.source-resolution-runtime-index")
    );
    assert!(
        summary
            .resolver_owned_products
            .contains(&"omena-resolver.style-module-resolution")
    );
    assert!(
        summary
            .resolver_owned_products
            .contains(&"omena-resolver.specifier-resolution-runtime")
    );
    assert!(summary.ready_surfaces.contains(&"resolverModuleGraphIndex"));
    assert!(
        summary
            .ready_surfaces
            .contains(&"resolverRuntimeQueryBoundary")
    );
    assert!(
        summary
            .ready_surfaces
            .contains(&"resolverSourceResolutionRuntimeIndex")
    );
    assert!(
        summary
            .ready_surfaces
            .contains(&"resolverStyleModuleResolution")
    );
    assert!(
        summary
            .ready_surfaces
            .contains(&"resolverSpecifierResolutionRuntime")
    );
    assert!(
        summary
            .ready_surfaces
            .contains(&"resolverBundlerPathAliasMapping")
    );
    assert!(
        summary
            .ready_surfaces
            .contains(&"resolverTsconfigPathMapping")
    );
    assert!(
        !summary
            .next_decoupling_targets
            .contains(&"tsconfigPathMapping")
    );
}

#[test]
fn summarizes_specifier_resolution_runtime_for_style_batches() {
    let available_style_paths = BTreeSet::from([
        "/fake/workspace/src/styles/Button.module.scss",
        "/fake/workspace/node_modules/@design/tokens/dist/theme.css",
    ]);
    let runtime = summarize_omena_resolver_specifier_resolution_runtime(
        "/fake/workspace/src/components/App.tsx",
        &[
            "@styles/Button".to_string(),
            "@design/tokens/theme".to_string(),
            "sass:map".to_string(),
            "./Missing".to_string(),
        ],
        &available_style_paths,
        &[OmenaResolverStylePackageManifestV0 {
            package_json_path: "/fake/workspace/node_modules/@design/tokens/package.json"
                .to_string(),
            package_json_source: r#"{"exports":{"./theme":{"style":"./dist/theme.css"}}}"#
                .to_string(),
        }],
        &[OmenaResolverTsconfigPathMappingV0 {
            base_path: "/fake/workspace".to_string(),
            pattern: "@styles/*".to_string(),
            target_patterns: vec!["src/styles/*".to_string()],
        }],
    );

    assert_eq!(
        runtime.product,
        "omena-resolver.specifier-resolution-runtime"
    );
    assert_eq!(runtime.specifier_count, 4);
    assert_eq!(runtime.resolved_specifier_count, 2);
    assert_eq!(runtime.external_specifier_count, 1);
    assert_eq!(runtime.unresolved_specifier_count, 1);
    assert!(
        runtime
            .ready_surfaces
            .contains(&"specifierResolutionRuntime")
    );
    assert!(
        runtime
            .entries
            .iter()
            .any(|entry| entry.source == "@styles/Button"
                && entry.status == "resolved"
                && entry.resolution_kind == "tsconfigPathStyleModule")
    );
    assert!(
        runtime
            .entries
            .iter()
            .any(|entry| entry.source == "@design/tokens/theme"
                && entry.status == "resolved"
                && entry.resolution_kind == "packageStyleModule")
    );
    assert!(
        runtime
            .entries
            .iter()
            .any(|entry| entry.source == "sass:map" && entry.status == "external")
    );
    assert!(
        runtime
            .entries
            .iter()
            .any(|entry| entry.source == "./Missing" && entry.status == "unresolved")
    );
}

#[test]
fn builds_resolver_module_graph_index_from_engine_input() {
    let input = sample_input();
    let summary = summarize_omena_resolver_module_graph_index(&input);

    assert_eq!(summary.schema_version, "0");
    assert_eq!(summary.product, "omena-resolver.module-graph-index");
    assert_eq!(summary.input_version, "2");
    assert_eq!(summary.module_count, 2);
    assert_eq!(summary.source_expression_edge_count, 2);
    assert_eq!(summary.type_fact_edge_count, 2);
    assert_eq!(summary.selector_count, 2);
    assert_eq!(summary.unresolved_type_fact_count, 0);
    assert!(summary.unresolved_type_fact_expression_ids.is_empty());

    let app = summary
        .modules
        .iter()
        .find(|module| module.style_file_path == "/tmp/App.module.scss");
    assert!(app.is_some());
    let Some(app) = app else {
        return;
    };
    assert_eq!(app.source_expression_ids, ["expr-1"]);
    assert_eq!(app.source_expression_kinds, ["symbolRef"]);
    assert_eq!(app.type_fact_expression_ids, ["expr-1"]);
    assert_eq!(app.selector_names, ["btn-active"]);
    assert_eq!(app.canonical_selector_names, ["btn-active"]);
    assert!(app.has_source_input);
    assert!(app.has_style_input);
    assert!(app.has_type_fact_input);

    let card = summary
        .modules
        .iter()
        .find(|module| module.style_file_path == "/tmp/Card.module.scss");
    assert!(card.is_some());
    let Some(card) = card else {
        return;
    };
    assert_eq!(card.source_expression_ids, ["expr-2"]);
    assert_eq!(card.source_expression_kinds, ["styleAccess"]);
    assert_eq!(card.type_fact_expression_ids, ["expr-2"]);
    assert_eq!(card.selector_names, ["card-header"]);
    assert_eq!(card.canonical_selector_names, ["card-header"]);
}

#[test]
fn exposes_runtime_query_boundary_from_module_graph_index() {
    let input = sample_input();
    let module_graph = summarize_omena_resolver_module_graph_index(&input);
    let runtime_query = summarize_omena_resolver_runtime_query_boundary(&module_graph);

    assert_eq!(runtime_query.schema_version, "0");
    assert_eq!(
        runtime_query.product,
        "omena-resolver.runtime-query-boundary"
    );
    assert_eq!(
        runtime_query.input_product,
        "omena-resolver.module-graph-index"
    );
    assert_eq!(runtime_query.input_version, "2");
    assert_eq!(runtime_query.module_query_count, 2);
    assert_eq!(runtime_query.fully_resolvable_module_count, 2);
    assert_eq!(runtime_query.source_only_module_count, 0);
    assert_eq!(runtime_query.style_only_module_count, 0);
    assert_eq!(runtime_query.unresolved_type_fact_count, 0);
    assert!(runtime_query.blocking_gaps.is_empty());
    assert!(
        runtime_query
            .runtime_capabilities
            .contains(&"moduleLookupByStylePath")
    );

    let app = query_omena_resolver_runtime_module(&module_graph, "/tmp/App.module.scss");
    assert!(app.is_some());
    let Some(app) = app else {
        return;
    };
    assert_eq!(app.status, "ready");
    assert!(app.can_resolve_source_expressions);
    assert!(app.can_check_type_fact_edges);
    assert!(app.can_query_selector_names);
    assert_eq!(app.source_expression_ids, ["expr-1"]);
    assert_eq!(app.selector_names, ["btn-active"]);
}

#[test]
fn builds_source_resolution_runtime_index_from_canonical_candidates() {
    let input = sample_input();
    let runtime_index = summarize_omena_resolver_source_resolution_runtime(&input);

    assert_eq!(runtime_index.schema_version, "0");
    assert_eq!(
        runtime_index.product,
        "omena-resolver.source-resolution-runtime-index"
    );
    assert_eq!(
        runtime_index.input_product,
        "engine-input-producers.source-resolution-canonical-producer"
    );
    assert_eq!(runtime_index.input_version, "2");
    assert_eq!(runtime_index.expression_count, 2);
    assert_eq!(runtime_index.resolved_expression_count, 2);
    assert_eq!(runtime_index.unresolved_expression_count, 0);
    assert!(runtime_index.blocking_gaps.is_empty());

    let app = query_omena_resolver_source_expression(&runtime_index, "expr-1");
    assert!(app.is_some());
    let Some(app) = app else {
        return;
    };
    assert_eq!(app.query_id, "expr-1");
    assert_eq!(app.expression_kind, "symbolRef");
    assert_eq!(app.style_file_path, "/tmp/App.module.scss");
    assert_eq!(app.selector_names, ["btn-active"]);
    assert_eq!(app.selector_certainty, "inferred");
    assert_eq!(app.selector_certainty_shape_kind, "constrained");
    assert_eq!(app.value_certainty_shape_kind, "constrained");
    assert!(app.has_selector_match);
    assert!(!app.has_finite_values);
    assert!(app.can_resolve_source_expression);
    assert_eq!(app.status, "resolved");

    let card = query_omena_resolver_source_expression(&runtime_index, "expr-2");
    assert!(card.is_some());
    let Some(card) = card else {
        return;
    };
    assert_eq!(card.selector_names, ["card-header"]);
    assert_eq!(
        card.finite_values,
        Some(vec!["card-header".to_string(), "card-body".to_string()])
    );
    assert!(card.has_finite_values);
}

#[test]
fn exposes_stable_query_fragment_and_canonical_producer_wrappers() {
    let input = sample_input();

    let query_fragments = summarize_omena_resolver_query_fragments(&input);
    assert_eq!(query_fragments.schema_version, "0");
    assert_eq!(query_fragments.input_version, "2");
    assert_eq!(query_fragments.fragments.len(), 2);
    assert_eq!(query_fragments.fragments[0].query_id, "expr-1");
    assert_eq!(
        query_fragments.fragments[1].style_file_path,
        "/tmp/Card.module.scss"
    );

    let canonical_signal = summarize_omena_resolver_canonical_producer_signal(&input);
    assert_eq!(canonical_signal.schema_version, "0");
    assert_eq!(canonical_signal.input_version, "2");
    assert_eq!(canonical_signal.canonical_bundle.query_fragments.len(), 2);
    assert_eq!(canonical_signal.canonical_bundle.candidates.len(), 2);
    assert_eq!(canonical_signal.evaluator_candidates.results.len(), 2);
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

#[cfg(unix)]
fn temp_dir(prefix: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let suffix = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)?
        .as_nanos();
    let path = std::env::temp_dir().join(format!("{prefix}_{suffix}"));
    fs::create_dir_all(path.as_path())?;
    Ok(path)
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
