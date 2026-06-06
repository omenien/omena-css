use super::*;
use crate::{
    OmenaQueryTransformExecutionContextV0,
    attach_omena_query_consumer_build_source_map_v3_with_sources_and_resolution_inputs,
    summarize_omena_query_bundle_code_split_source_map_v3,
};

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
fn consumer_build_summary_can_attach_source_map_v3() -> Result<(), String> {
    let source = "/* remove */ .button { color: red; }\n.card { color: blue; }";
    let mut summary = execute_omena_query_consumer_build_style_source(
        "Button.module.css",
        source,
        &[
            "comment-strip".to_string(),
            "whitespace-strip".to_string(),
            "print-css".to_string(),
        ],
    );

    attach_omena_query_consumer_build_source_map_v3(&mut summary, source);

    let source_map = summary
        .source_map_v3
        .as_ref()
        .ok_or_else(|| "consumer build should attach Source Map V3 on request".to_string())?;
    assert_eq!(source_map.version, 3);
    assert_eq!(source_map.file, "Button.module.css");
    assert_eq!(source_map.sources, vec!["Button.module.css"]);
    assert_eq!(source_map.sources_content, vec![source]);
    assert!(!source_map.mappings.is_empty());
    assert!(source_map.x_omena_segment_count > 0);
    assert!(summary.ready_surfaces.contains(&"sourceMapV3Serializer"));
    Ok(())
}

#[test]
fn consumer_build_source_map_v3_preserves_bundle_import_origins() -> Result<(), String> {
    let sources = vec![
        OmenaQueryStyleSourceInputV0 {
            style_path: "src/App.css".to_string(),
            style_source: r#"@import "./theme/tokens.css"; .app { color: green; }"#.to_string(),
        },
        OmenaQueryStyleSourceInputV0 {
            style_path: "src/theme/tokens.css".to_string(),
            style_source: r#"@import "./base.css"; .token { color: blue; }"#.to_string(),
        },
        OmenaQueryStyleSourceInputV0 {
            style_path: "src/theme/base.css".to_string(),
            style_source: ".base { color: red; }".to_string(),
        },
    ];
    let mut summary = execute_omena_query_consumer_build_style_sources(
        "src/App.css",
        &sources,
        &["import-inline".to_string(), "print-css".to_string()],
        &[],
    )?;

    attach_omena_query_consumer_build_bundle_summary(&mut summary, &sources[0].style_source);
    attach_omena_query_consumer_build_source_map_v3_with_sources(&mut summary, &sources, &[]);

    assert!(
        summary
            .execution
            .output_css
            .contains(".base { color: red; }")
    );
    assert!(
        summary
            .execution
            .output_css
            .contains(".token { color: blue; }")
    );
    assert!(!summary.execution.output_css.contains("@import"));
    let source_map = summary
        .source_map_v3
        .as_ref()
        .ok_or_else(|| "consumer build should attach bundle-aware Source Map V3".to_string())?;
    assert!(source_map.sources.contains(&"src/App.css".to_string()));
    assert!(
        source_map
            .sources
            .contains(&"src/theme/tokens.css".to_string())
    );
    assert!(
        source_map
            .sources
            .contains(&"src/theme/base.css".to_string())
    );
    assert!(
        source_map
            .sources
            .iter()
            .position(|source| source == "src/theme/tokens.css")
            .and_then(|index| source_map.sources_content.get(index))
            .is_some_and(|content| content == r#"@import "./base.css"; .token { color: blue; }"#)
    );
    assert!(
        source_map
            .sources
            .iter()
            .position(|source| source == "src/theme/base.css")
            .and_then(|index| source_map.sources_content.get(index))
            .is_some_and(|content| content == ".base { color: red; }")
    );
    assert!(source_map.x_omena_pass_ids.contains(&"import-inline"));
    assert!(
        summary
            .ready_surfaces
            .contains(&"bundleSourceMapOriginChain")
    );
    Ok(())
}

#[test]
fn consumer_build_source_map_v3_preserves_alias_resolved_import_origins() -> Result<(), String> {
    let sources = vec![
        OmenaQueryStyleSourceInputV0 {
            style_path: "/workspace/src/App.css".to_string(),
            style_source: r#"@import "@styles/tokens.css"; .app { color: green; }"#.to_string(),
        },
        OmenaQueryStyleSourceInputV0 {
            style_path: "/workspace/src/styles/tokens.css".to_string(),
            style_source: ".token { color: blue; }".to_string(),
        },
    ];
    let resolution_inputs = OmenaQueryStyleResolutionInputsV0 {
        package_manifests: vec![],
        tsconfig_path_mappings: vec![OmenaQueryTsconfigPathMappingV0 {
            base_path: "/workspace".to_string(),
            pattern: "@styles/*".to_string(),
            target_patterns: vec!["src/styles/*".to_string()],
        }],
        bundler_path_mappings: vec![],
    };
    let mut summary =
        execute_omena_query_consumer_build_style_sources_with_context_and_resolution_inputs(
            "/workspace/src/App.css",
            &sources,
            &["import-inline".to_string(), "print-css".to_string()],
            &OmenaQueryTransformExecutionContextV0::default(),
            &resolution_inputs,
        )?;

    attach_omena_query_consumer_build_source_map_v3_with_sources_and_resolution_inputs(
        &mut summary,
        &sources,
        &resolution_inputs,
    );

    assert!(
        summary
            .execution
            .output_css
            .contains(".token { color: blue; }")
    );
    assert!(!summary.execution.output_css.contains("@import"));
    let source_map = summary.source_map_v3.as_ref().ok_or_else(|| {
        "consumer build should attach alias-aware bundle Source Map V3".to_string()
    })?;
    assert!(
        source_map
            .sources
            .contains(&"/workspace/src/styles/tokens.css".to_string())
    );
    assert!(
        source_map
            .sources
            .iter()
            .position(|source| source == "/workspace/src/styles/tokens.css")
            .and_then(|index| source_map.sources_content.get(index))
            .is_some_and(|content| content == ".token { color: blue; }")
    );
    assert!(
        summary
            .ready_surfaces
            .contains(&"bundleSourceMapOriginChain")
    );
    Ok(())
}

#[test]
fn bundle_code_split_source_map_preserves_split_origin() {
    let source = r#"@import "./base.css"; .token { color: blue; }"#;
    let generated = r#"@import "theme-base-css-1.css"; .token { color: blue; }"#;
    let source_map = summarize_omena_query_bundle_code_split_source_map_v3(
        "theme-tokens-css-1.css",
        generated,
        "src/theme/tokens.css",
        source,
    );

    assert_eq!(source_map.version, 3);
    assert_eq!(source_map.file, "theme-tokens-css-1.css");
    assert_eq!(source_map.sources, vec!["src/theme/tokens.css"]);
    assert_eq!(source_map.sources_content, vec![source]);
    assert!(!source_map.mappings.is_empty());
    assert_eq!(source_map.x_omena_segment_count, 1);
    assert_eq!(source_map.x_omena_pass_ids, vec!["code-split-emission"]);
}

#[test]
fn consumer_build_summary_can_attach_bundle_asset_urls() -> Result<(), String> {
    let source = r#".button { background-image: url("./assets/icon.svg"); }"#;
    let mut summary = execute_omena_query_consumer_build_style_source(
        "src/Button.module.css",
        source,
        &["print-css".to_string()],
    );

    attach_omena_query_consumer_build_bundle_summary(&mut summary, source);

    let bundle = summary
        .bundle
        .as_ref()
        .ok_or_else(|| "consumer build should attach bundle summary on request".to_string())?;
    assert_eq!(bundle.asset_urls.len(), 1);
    assert_eq!(bundle.asset_urls[0].normalized_url, "./assets/icon.svg");
    assert_eq!(
        bundle.asset_urls[0].resolved_path.as_deref(),
        Some("src/assets/icon.svg")
    );
    assert!(bundle.code_splitting_required);
    assert_eq!(bundle.code_split_chunks.len(), 2);
    assert!(summary.ready_surfaces.contains(&"bundleAssetUrlResolution"));
    assert!(summary.ready_surfaces.contains(&"bundleCodeSplitPlan"));
    Ok(())
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

#[cfg(feature = "lawvere-trace")]
#[test]
fn transform_execute_lawvere_trace_is_explicit_opt_in_product_lane() {
    let source = r#".a { color : red ; /* remove */ content : "x y" ; }"#;
    let requested_pass_ids = vec![
        "comment-strip".to_string(),
        "whitespace-strip".to_string(),
        "print-css".to_string(),
    ];

    let summary = execute_omena_query_transform_passes_from_source_with_lawvere_trace(
        "Button.css",
        source,
        &requested_pass_ids,
    );

    assert_eq!(
        summary.product,
        "omena-query.transform-execute-lawvere-trace"
    );
    assert_eq!(
        summary.product_scope,
        "explicitOptInLawvereTraceProductLane"
    );
    assert!(!summary.default_product_mechanism);
    assert!(!summary.global_transform_theorem_claimed);
    assert_eq!(
        summary.execution.execution.output_css,
        r#".a{color:red;content:"x y"}"#
    );
    assert_eq!(summary.lawvere_trace.product, "omena-lawvere.model-trace");
    assert_eq!(
        summary.lawvere_trace.ordered_pass_ids,
        summary.execution.execution.ordered_pass_ids
    );
    assert_eq!(summary.parallel_plan.scheduler_status, "scaffoldOnly");
    assert!(!summary.parallel_plan.executor_consumes_plan);
    assert_eq!(summary.reorderability_certificates.len(), 1);
    assert_eq!(summary.differential_witnesses.len(), 1);
    assert_eq!(
        summary.reorderability_certificates[0].commute_witness,
        "differentialCommutativityCorpus"
    );
    assert!(summary.reorderability_certificates[0].accepted);
    assert_eq!(summary.differential_witnesses[0].fixture_count, 1);
    assert_eq!(summary.differential_witnesses[0].mismatch_count, 0);
    assert!(
        summary
            .ready_surfaces
            .contains(&"lawvereDifferentialReorderabilityCertificate")
    );
}

#[cfg(feature = "lawvere-trace")]
#[test]
fn transform_execute_lawvere_trace_exposes_rejected_differential_witness() {
    let source = r#".a { & .b { color: red; } } .a .b { color: red; }"#;
    let requested_pass_ids = vec![
        "rule-deduplication".to_string(),
        "nesting-unwrap".to_string(),
    ];

    let summary = execute_omena_query_transform_passes_from_source_with_lawvere_trace(
        "Nested.css",
        source,
        &requested_pass_ids,
    );

    assert_eq!(
        summary.product_scope,
        "explicitOptInLawvereTraceProductLane"
    );
    assert!(!summary.default_product_mechanism);
    assert!(!summary.global_transform_theorem_claimed);
    assert_eq!(summary.reorderability_certificates.len(), 1);
    assert_eq!(summary.differential_witnesses.len(), 1);
    assert_eq!(
        summary.reorderability_certificates[0].differential_mismatch_count,
        1
    );
    assert!(!summary.reorderability_certificates[0].accepted);
    assert_eq!(summary.differential_witnesses[0].fixture_count, 1);
    assert_eq!(summary.differential_witnesses[0].equal_fixture_count, 0);
    assert_eq!(summary.differential_witnesses[0].mismatch_count, 1);
    assert!(!summary.differential_witnesses[0].accepted);
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
fn lists_transform_pass_summaries_from_query() {
    let passes = list_omena_query_transform_pass_summaries();

    assert_eq!(passes.len(), 40);
    assert!(passes.iter().any(|pass| pass.id == "whitespace-strip"));
    assert!(passes.iter().any(|pass| pass.id == "print-css"));
}
