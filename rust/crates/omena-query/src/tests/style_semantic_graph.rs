use crate::{
    OmenaQueryStylePackageManifestV0,
    summarize_omena_query_style_semantic_graph_batch_from_sources,
    summarize_omena_query_style_semantic_graph_batch_from_sources_with_package_manifests,
    summarize_omena_query_style_semantic_graph_from_source,
};

use super::support::sample_input;

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

    assert_eq!(
        batch.cross_file_summary.product,
        "omena-query.cross-file-summary"
    );
    assert_eq!(
        batch.cross_file_summary.summary_edge_count,
        resolution.module_edge_count + resolution.graph_closure_edge_count
    );
    assert!(
        batch
            .cross_file_summary
            .capabilities
            .sass_module_edges_ready
    );
    assert!(
        batch
            .cross_file_summary
            .capabilities
            .stable_summary_hash_ready
    );
    assert!(batch.cross_file_summary.edges.iter().any(|edge| {
        edge.edge_kind == "sassForward"
            && edge.from_kind == "style"
            && edge.from_path == "/tmp/_tokens.scss"
            && edge.target_kind == Some("style")
            && edge.target_path.as_deref() == Some("/tmp/_palette.scss")
            && edge.source.as_deref() == Some("./palette")
            && edge.provenance
                == vec![
                    "omena-query.sass-module-cross-file-resolution",
                    "omena-parser.sass-module-facts",
                ]
            && edge.linear_provenance.labels() == edge.provenance
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
