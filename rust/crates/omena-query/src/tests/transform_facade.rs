use super::*;
use crate::{
    OmenaQueryBundlePlanInputV0, OmenaQueryTransformExecutionContextV0,
    attach_omena_query_consumer_build_source_map_v3_with_sources_and_resolution_inputs,
    run_omena_query_bundle, summarize_omena_query_bundle_code_split_source_map_v3,
    summarize_omena_query_bundle_code_split_workspace_plan,
};
use omena_query_transform_runner::{
    TRANSFORM_PASS_CATALOG_LEN, all_transform_pass_kinds,
    with_transform_pass_sort_ordinal_overrides_for_test,
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
        supports_relative_color: true,
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
        enable_container_static_eval: false,
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
fn transform_plan_order_is_invariant_under_reversed_sort_ordinals() {
    let source = r#"
@import "./tokens.css";
@value primary from "./colors.module.css";
.card {
  composes: reset from "./reset.module.css";
  color: light-dark(red, blue);
  margin-inline-start: 1rem;
  & .title { color: color-mix(in srgb, red 50%, blue); }
}
@layer theme { .card { color: oklch(60% 0.2 20); } }
@supports not (display: grid) { .fallback { display: block; } }
@media not all { .dead { color: red; } }
"#;
    let target_support = OmenaQueryTargetFeatureSupportV0 {
        vendor_prefix_required: true,
        supports_light_dark: false,
        supports_color_mix: false,
        supports_oklch_oklab: false,
        supports_color_function: false,
        supports_relative_color: false,
        supports_logical_properties: false,
        supports_css_nesting: false,
        supports_css_scope: false,
        supports_cascade_layers: false,
    };
    let target_options = OmenaQueryTargetTransformOptionsV0 {
        allow_logical_to_physical: true,
        allow_scope_flatten: true,
        allow_layer_flatten: true,
        enable_supports_static_eval: true,
        enable_media_static_eval: true,
        enable_container_static_eval: true,
        drop_dark_mode_media_queries: true,
    };

    let baseline = summarize_omena_query_transform_plan_from_source(
        "Card.module.scss",
        source,
        "legacy-webview",
        target_support,
        target_options,
        default_omena_query_transform_print_options(),
    );
    let mut reversed_ordinals = [0u8; TRANSFORM_PASS_CATALOG_LEN];
    for kind in all_transform_pass_kinds() {
        reversed_ordinals[(kind.ordinal() - 1) as usize] =
            (TRANSFORM_PASS_CATALOG_LEN as u8 + 1).saturating_sub(kind.ordinal());
    }
    let permuted = with_transform_pass_sort_ordinal_overrides_for_test(reversed_ordinals, || {
        summarize_omena_query_transform_plan_from_source(
            "Card.module.scss",
            source,
            "legacy-webview",
            target_support,
            target_options,
            default_omena_query_transform_print_options(),
        )
    });

    assert!(baseline.bundle.required_pass_ids.len() >= 3);
    assert!(baseline.target.required_pass_ids.len() >= 8);
    assert_eq!(
        baseline.combined_pass_ids,
        baseline.combined_plan.ordered_pass_ids
    );
    assert_eq!(
        permuted.combined_pass_ids,
        permuted.combined_plan.ordered_pass_ids
    );
    assert_eq!(permuted.combined_pass_ids, baseline.combined_pass_ids);
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
        ..Default::default()
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
fn bundle_code_split_workspace_plan_surfaces_entry_config_and_shared_boundaries()
-> Result<(), String> {
    let sources = vec![
        OmenaQueryStyleSourceInputV0 {
            style_path: "src/app.css".to_string(),
            style_source: r#"@import "./theme/tokens.css"; .app { color: green; }"#.to_string(),
        },
        OmenaQueryStyleSourceInputV0 {
            style_path: "src/admin.css".to_string(),
            style_source: r#"@import "./theme/tokens.css"; .admin { color: green; }"#.to_string(),
        },
        OmenaQueryStyleSourceInputV0 {
            style_path: "src/theme/tokens.css".to_string(),
            style_source: ".token { color: blue; }".to_string(),
        },
    ];
    let plan = summarize_omena_query_bundle_code_split_workspace_plan(
        "src/app.css",
        &["src/admin.css".to_string()],
        &sources,
        &OmenaQueryStyleResolutionInputsV0::default(),
    )?;
    let output_for = |source_path: &str| {
        plan.outputs
            .iter()
            .find(|output| output.source_path == source_path)
            .ok_or_else(|| format!("missing output for {source_path}"))
    };

    assert_eq!(plan.product, "omena-query.bundle-code-split-workspace-plan");
    assert_eq!(plan.configured_entry_count, 1);
    assert_eq!(plan.shared_boundary_count, 1);
    assert!(plan.ready_surfaces.contains(&"bundleCodeSplitPlan"));
    assert!(plan.ready_surfaces.contains(&"bundleCodeSplitBoundaryPlan"));
    assert!(plan.ready_surfaces.contains(&"bundleCodeSplitEntryConfig"));
    assert!(
        plan.ready_surfaces
            .contains(&"bundleCodeSplitSharedChunkPlan")
    );

    assert_eq!(output_for("src/app.css")?.split_boundary, "entry");
    assert!(output_for("src/app.css")?.is_entry);
    assert_eq!(output_for("src/admin.css")?.split_boundary, "entryConfig");
    assert!(output_for("src/admin.css")?.is_entry);
    let shared = output_for("src/theme/tokens.css")?;
    assert_eq!(shared.split_boundary, "shared");
    assert!(!shared.is_entry);
    assert_eq!(
        shared.reachable_from_entries,
        vec!["src/admin.css".to_string(), "src/app.css".to_string()]
    );
    Ok(())
}

#[test]
fn bundle_operation_facade_matches_consumer_build_source_map() -> Result<(), String> {
    let sources = vec![
        OmenaQueryStyleSourceInputV0 {
            style_path: "src/app.css".to_string(),
            style_source: r#"@import "./theme/tokens.css"; .app { color: green; }"#.to_string(),
        },
        OmenaQueryStyleSourceInputV0 {
            style_path: "src/theme/tokens.css".to_string(),
            style_source: ".token { color: blue; }".to_string(),
        },
    ];
    let pass_ids = vec!["import-inline".to_string(), "print-css".to_string()];
    let context = OmenaQueryTransformExecutionContextV0::default();
    let resolution_inputs = OmenaQueryStyleResolutionInputsV0::default();

    let artifact = run_omena_query_bundle(OmenaQueryBundlePlanInputV0 {
        target_style_path: "src/app.css",
        style_sources: &sources,
        source_map_sources: &sources,
        requested_pass_ids: &pass_ids,
        context: &context,
        resolution_inputs: &resolution_inputs,
        asset_rewrites: Vec::new(),
        bundle_entry_style_paths: &[],
    })?;
    let mut summary =
        execute_omena_query_consumer_build_style_sources_with_context_and_resolution_inputs(
            "src/app.css",
            &sources,
            &pass_ids,
            &context,
            &resolution_inputs,
        )?;
    attach_omena_query_consumer_build_source_map_v3_with_sources_and_resolution_inputs(
        &mut summary,
        &sources,
        &resolution_inputs,
    );

    assert_eq!(artifact.product, "omena-query.bundle-artifact");
    assert_eq!(artifact.output_css, summary.execution.output_css);
    let summary_source_map = summary
        .source_map_v3
        .ok_or_else(|| "consumer summary should carry a source map".to_string())?;
    assert_eq!(artifact.source_map_v3, summary_source_map);
    assert_eq!(artifact.per_pass_provenance, artifact.execution.outcomes);
    assert!(artifact.ready_surfaces.contains(&"bundleOperationFacade"));
    assert!(
        artifact
            .code_split_outputs
            .iter()
            .any(|output| output.source_path == "src/theme/tokens.css")
    );
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
        supports_relative_color: true,
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
        enable_container_static_eval: false,
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
fn exposes_target_stale_prefix_removal_egg_witnesses_from_source_execution() {
    let source = ".a { -webkit-user-select: none; user-select: none; }";
    let summary = summarize_omena_query_transform_plan_from_source(
        "Button.css",
        source,
        "modern",
        OmenaQueryTargetFeatureSupportV0 {
            vendor_prefix_required: false,
            supports_light_dark: true,
            supports_color_mix: true,
            supports_oklch_oklab: true,
            supports_color_function: true,
            supports_relative_color: true,
            supports_logical_properties: true,
            supports_css_nesting: true,
            supports_css_scope: true,
            supports_cascade_layers: true,
        },
        OmenaQueryTargetTransformOptionsV0 {
            allow_logical_to_physical: false,
            allow_scope_flatten: false,
            allow_layer_flatten: false,
            enable_supports_static_eval: false,
            enable_media_static_eval: false,
            enable_container_static_eval: false,
            drop_dark_mode_media_queries: false,
        },
        default_omena_query_transform_print_options(),
    );

    assert!(
        summary
            .target
            .planned_pass_ids
            .contains(&"stale-prefix-removal")
    );
    assert!(summary.combined_pass_ids.contains(&"stale-prefix-removal"));
    assert!(!summary.execution.output_css.contains("-webkit-user-select"));
    assert!(summary.execution.output_css.contains("user-select: none"));
    assert!(summary.egg.planned_pass_ids.is_empty());
    assert!(summary.egg_witnesses.iter().any(|witness| {
        witness.pass_id == "stale-prefix-removal"
            && witness.source_kind == "stalePrefixExactPeer"
            && witness.css_before == "-webkit-user-select: none;"
            && witness.css_after == "user-select: none;"
            && witness.execution.accepted
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
            enable_container_static_eval: false,
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
        enable_container_static_eval: false,
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
fn exposes_native_css_static_eval_execution_runner_from_source() {
    let source = r#"@function --gap(--size <length>: 1rem) returns <length> { result: var(--size); } .card { gap: --gap(2rem); display: if(supports(display: grid): grid; else: block); margin: if(media(width >= 1px): 1rem; else: 2rem); } @when supports(display: grid) { .grid { display: grid; } } @else { .fallback { display: block; } }"#;
    let summary = execute_omena_query_transform_passes_from_source(
        "Native.css",
        source,
        &[
            "native-css-static-eval".to_string(),
            "print-css".to_string(),
        ],
    );

    assert_eq!(summary.product, "omena-query.transform-execute");
    assert_eq!(summary.unknown_pass_ids, Vec::<String>::new());
    assert_eq!(summary.execution.mutation_count, 3);
    assert!(summary.execution.output_css.contains("gap: 2rem"));
    assert!(summary.execution.output_css.contains("display: grid"));
    assert!(
        summary
            .execution
            .output_css
            .contains(".grid { display: grid; }")
    );
    assert!(
        summary
            .execution
            .output_css
            .contains("margin: if(media(width >= 1px): 1rem; else: 2rem)")
    );
    assert!(!summary.execution.output_css.contains("--gap(2rem)"));
    assert!(!summary.execution.output_css.contains("@when"));
    assert!(!summary.execution.output_css.contains(".fallback"));
    assert!(
        !summary
            .execution
            .output_css
            .contains("display: if(supports")
    );
    assert_eq!(
        summary.execution.executed_pass_ids,
        vec!["native-css-static-eval", "print-css"]
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

    assert_eq!(passes.len(), 44);
    assert!(passes.iter().any(|pass| pass.id == "whitespace-strip"));
    assert!(
        passes
            .iter()
            .any(|pass| pass.id == "relative-color-lowering")
    );
    assert!(passes.iter().any(|pass| pass.id == "container-static-eval"));
    assert!(
        passes
            .iter()
            .any(|pass| pass.id == "native-css-static-eval")
    );
    assert!(passes.iter().any(|pass| {
        pass.id == "native-css-static-eval"
            && pass.explicit_opt_in_required
            && pass.dialect_restriction == Some("css-only")
            && pass.spec_snapshot == Some("css-values-5-if-css-mixins-1-function-ed-2026-06-22")
            && pass.opt_in_policy
                == Some("explicit-pass-id-required-default-consumer-build-excludes")
    }));
    assert!(passes.iter().any(|pass| pass.id == "print-css"));
}
