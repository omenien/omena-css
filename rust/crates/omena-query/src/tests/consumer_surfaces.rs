use crate::{
    OmenaQueryStyleSourceInputV0, OmenaQueryTargetTransformOptionsV0,
    OmenaQueryTransformExecutionContextV0, OmenaQueryTransformModuleEvaluationNativeEditV0,
    OmenaQueryTransformModuleEvaluationOracleV0, OmenaQueryTransformModuleEvaluationV0,
    execute_omena_query_consumer_build_style_source,
    execute_omena_query_consumer_build_style_source_for_target_query,
    execute_omena_query_consumer_build_style_source_for_target_query_with_context_and_options,
    execute_omena_query_consumer_build_style_source_for_target_query_with_context_options_and_additional_passes,
    execute_omena_query_consumer_build_style_source_for_target_query_with_options,
    execute_omena_query_consumer_build_style_sources_for_target_query_with_context_and_options,
    execute_omena_query_consumer_build_style_sources_with_context,
    summarize_omena_query_consumer_check_style_source,
};
use std::collections::BTreeSet;
use std::{fs, path::PathBuf, time::SystemTime};

#[test]
fn exposes_consumer_check_facade_from_query() {
    let summary = summarize_omena_query_consumer_check_style_source(
        "Button.module.scss",
        "@keyframes fade { to { opacity: 1; } }\n.card { color: red; animation: fade 1s; }\n:root { --brand: blue; color: var(--brand); }",
    );

    assert_eq!(summary.product, "omena-query.consumer-check-style-source");
    assert_eq!(summary.style_path, "Button.module.scss");
    assert_eq!(summary.dialect, "scss");
    assert_eq!(summary.parser_error_count, 0);
    assert_eq!(summary.class_selector_count, 1);
    assert_eq!(summary.custom_property_count, 1);
    assert_eq!(summary.keyframe_count, 1);
    assert!(summary.ready_surfaces.contains(&"consumerCheckFacade"));
    assert!(
        summary
            .ready_surfaces
            .contains(&"semanticRuntimeIndexFacts")
    );
}

#[test]
fn consumer_check_keeps_parser_fallback_for_extensionless_style_path() {
    let summary = summarize_omena_query_consumer_check_style_source(
        "virtual-style",
        ".card { --brand: blue; color: var(--brand); }",
    );

    assert_eq!(summary.class_selector_count, 1);
    assert_eq!(summary.custom_property_count, 1);
    assert!(summary.ready_surfaces.contains(&"parserFactSummary"));
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
fn consumer_build_keeps_native_css_static_eval_explicit_opt_in() {
    let source = ".card { display: if(supports(display: grid): grid; else: block); }";
    let default_summary =
        execute_omena_query_consumer_build_style_source("Button.module.css", source, &[]);

    assert!(
        !default_summary
            .execution
            .executed_pass_ids
            .contains(&"native-css-static-eval")
    );
    assert!(default_summary.execution.output_css.contains("if("));

    let explicit_summary = execute_omena_query_consumer_build_style_source(
        "Button.module.css",
        source,
        &[
            "native-css-static-eval".to_string(),
            "print-css".to_string(),
        ],
    );

    assert!(
        explicit_summary
            .execution
            .executed_pass_ids
            .contains(&"native-css-static-eval")
    );
    assert!(
        explicit_summary
            .execution
            .output_css
            .contains("display: grid")
    );
    assert!(!explicit_summary.execution.output_css.contains("if("));
}

#[test]
fn consumer_build_reports_the_effective_default_plan_that_executes() {
    let summary = execute_omena_query_consumer_build_style_source(
        "Baseline.module.css",
        ".a { color: #ffffff; margin: 0px; } .a { color: #ffffff; margin: 0px; }",
        &[],
    );

    assert!(summary.requested_pass_ids.is_empty());
    assert!(!summary.effective_pass_ids.is_empty());
    for closed_world_pass in [
        "layer-flatten",
        "tree-shake-class",
        "tree-shake-keyframes",
        "tree-shake-value",
        "tree-shake-custom-property",
    ] {
        assert!(
            !summary
                .effective_pass_ids
                .iter()
                .any(|id| id == closed_world_pass)
        );
    }
    let effective = summary
        .effective_pass_ids
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let planned = summary
        .execution
        .ordered_pass_ids
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    assert_eq!(planned, effective);
    assert_eq!(
        summary.execution.requested_pass_ids,
        summary
            .effective_pass_ids
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>()
    );
    assert_eq!(summary.execution.output_css, "._a_0{color:#fff;margin:0}");
    assert!(summary.open_world_snapshot.is_none());
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
fn target_query_build_emits_ms_vendor_prefixes_for_ie_target_only() {
    let summary = execute_omena_query_consumer_build_style_source_for_target_query(
        "Grid.module.css",
        ".card { display: grid; transform: translateX(1px); columns: 2; touch-action: manipulation; } .flex { display: flex; align-items: flex-start; justify-content: space-between; flex-direction: row-reverse; flex-wrap: wrap; } @supports (display: grid) { .query { display: grid; } }",
        "ie 11",
    );

    assert!(summary.unknown_pass_ids.is_empty());
    assert!(
        summary
            .requested_pass_ids
            .iter()
            .any(|pass_id| pass_id == "vendor-prefixing")
    );
    assert!(summary.target_query.is_some());
    let Some(target_query) = summary.target_query.as_ref() else {
        return;
    };
    assert_eq!(target_query.resolved_targets, vec!["ie 11"]);
    assert!(target_query.support.vendor_prefix_required);
    assert!(target_query.vendor_prefix_policy.is_some());
    let Some(policy) = target_query.vendor_prefix_policy else {
        return;
    };
    assert!(!policy.webkit);
    assert!(!policy.moz);
    assert!(policy.ms);
    assert!(summary.execution.output_css.contains("display: -ms-grid"));
    assert!(
        summary
            .execution
            .output_css
            .contains("-ms-transform: translateX(1px)")
    );
    assert!(
        summary
            .execution
            .output_css
            .contains("-ms-touch-action: manipulation")
    );
    assert!(
        summary
            .execution
            .output_css
            .contains("-ms-flex-pack: justify")
    );
    assert!(summary.execution.output_css.contains("-ms-flex-wrap: wrap"));
    assert!(!summary.execution.output_css.contains("-webkit-"));
    assert!(!summary.execution.output_css.contains("-moz-"));
    assert!(
        summary
            .execution
            .output_css
            .contains("@supports ((display: grid) or (display: -ms-grid))")
    );
    assert!(summary.ready_surfaces.contains(&"targetQueryBuildFacade"));
}

#[test]
fn target_query_build_emits_webkit_vendor_prefixes_without_ms_for_webkit_target() {
    let summary = execute_omena_query_consumer_build_style_source_for_target_query(
        "Motion.module.css",
        ".card { transform: translateX(1px); user-select: none; } @supports (transform: translateX(1px)) { .query { transform: translateX(1px); } } @keyframes fade { to { opacity: 1; } }",
        "chrome 40",
    );

    assert!(summary.unknown_pass_ids.is_empty());
    assert!(
        summary
            .requested_pass_ids
            .iter()
            .any(|pass_id| pass_id == "vendor-prefixing")
    );
    assert!(summary.target_query.is_some());
    let Some(target_query) = summary.target_query.as_ref() else {
        return;
    };
    assert_eq!(target_query.resolved_targets, vec!["chrome 40"]);
    assert!(target_query.support.vendor_prefix_required);
    assert!(target_query.vendor_prefix_policy.is_some());
    let Some(policy) = target_query.vendor_prefix_policy else {
        return;
    };
    assert!(policy.webkit);
    assert!(!policy.moz);
    assert!(!policy.ms);
    assert!(
        summary
            .execution
            .output_css
            .contains("-webkit-transform: translateX(1px)")
    );
    assert!(
        summary
            .execution
            .output_css
            .contains("@-webkit-keyframes fade")
    );
    assert!(!summary.execution.output_css.contains("-ms-"));
    assert!(!summary.execution.output_css.contains("-moz-"));
    assert!(summary.execution.output_css.contains(
        "@supports ((transform: translateX(1px)) or (-webkit-transform: translateX(1px)))"
    ));
}

#[test]
fn target_query_build_composes_prefixing_with_minification() {
    let source = "/* note */ .card { display: flex; } .empty {}";
    let minify_passes = vec![
        "comment-strip".to_string(),
        "empty-rule-removal".to_string(),
        "whitespace-strip".to_string(),
    ];
    let legacy = execute_omena_query_consumer_build_style_source_for_target_query_with_context_options_and_additional_passes(
        "Theme.css",
        source,
        "ie 11",
        &OmenaQueryTransformExecutionContextV0::default(),
        OmenaQueryTargetTransformOptionsV0::default(),
        &minify_passes,
    );
    let modern = execute_omena_query_consumer_build_style_source_for_target_query_with_context_options_and_additional_passes(
        "Theme.css",
        source,
        "chrome 123",
        &OmenaQueryTransformExecutionContextV0::default(),
        OmenaQueryTargetTransformOptionsV0::default(),
        &minify_passes,
    );

    for pass_id in [
        "vendor-prefixing",
        "comment-strip",
        "empty-rule-removal",
        "whitespace-strip",
    ] {
        assert!(
            legacy
                .requested_pass_ids
                .iter()
                .any(|requested| requested == pass_id),
            "legacy target should compose pass {pass_id}"
        );
        assert!(legacy.execution.executed_pass_ids.contains(&pass_id));
    }
    assert!(legacy.execution.output_css.contains("display:-ms-flexbox"));
    assert!(!legacy.execution.output_css.contains("/* note */"));
    assert!(!legacy.execution.output_css.contains(".empty"));
    let recorded_mutation_count = legacy
        .execution
        .decisions
        .iter()
        .map(|decision| decision.compatibility_outcome().mutation_count)
        .sum::<usize>();
    assert_eq!(recorded_mutation_count, legacy.execution.mutation_count);
    assert!(legacy.execution.decisions.iter().any(|decision| {
        let outcome = decision.compatibility_outcome();
        outcome.pass_id == "vendor-prefixing" && outcome.mutation_count > 0
    }));
    assert!(
        legacy
            .execution
            .semantic_preservation_telemetry
            .observed_pass_count
            > 0
    );
    assert_eq!(
        legacy
            .execution
            .semantic_preservation_telemetry
            .observed_pass_count,
        legacy
            .execution
            .semantic_preservation_telemetry
            .preserved_pass_count
    );
    assert_eq!(
        legacy
            .execution
            .semantic_preservation_telemetry
            .blocked_pass_count,
        0
    );

    assert!(
        !modern
            .requested_pass_ids
            .iter()
            .any(|requested| requested == "vendor-prefixing")
    );
    assert!(
        modern
            .requested_pass_ids
            .contains(&"whitespace-strip".to_string())
    );
    assert!(!modern.execution.output_css.contains("-ms-"));
}

#[test]
fn target_query_build_lowers_static_colors_only_for_unsupported_targets() {
    let source = ".card { color: light-dark(#000, #fff); background: color-mix(in srgb, red 50%, blue 50%); border-color: rgb(from red r g b); }";
    let minify_passes = vec!["whitespace-strip".to_string()];
    let legacy = execute_omena_query_consumer_build_style_source_for_target_query_with_context_options_and_additional_passes(
        "Theme.css",
        source,
        "ie 11",
        &OmenaQueryTransformExecutionContextV0::default(),
        OmenaQueryTargetTransformOptionsV0::default(),
        &minify_passes,
    );
    let modern = execute_omena_query_consumer_build_style_source_for_target_query_with_context_options_and_additional_passes(
        "Theme.css",
        source,
        "chrome 123",
        &OmenaQueryTransformExecutionContextV0::default(),
        OmenaQueryTargetTransformOptionsV0::default(),
        &minify_passes,
    );

    for pass_id in [
        "light-dark-lowering",
        "color-mix-lowering",
        "relative-color-lowering",
    ] {
        assert!(legacy.execution.executed_pass_ids.contains(&pass_id));
        assert!(legacy.execution.decisions.iter().any(|decision| {
            let outcome = decision.compatibility_outcome();
            outcome.pass_id == pass_id && outcome.mutation_count > 0
        }));
        assert!(
            !modern
                .requested_pass_ids
                .iter()
                .any(|requested| requested == pass_id),
            "supported target should not request pass {pass_id}"
        );
    }
    assert!(!legacy.execution.output_css.contains("light-dark("));
    assert!(!legacy.execution.output_css.contains("color-mix("));
    assert!(!legacy.execution.output_css.contains("rgb(from"));
    assert!(legacy.execution.output_css.contains("@media"));
    assert!(legacy.execution.output_css.contains("rgb(128 0 128)"));
    assert!(
        legacy
            .execution
            .output_css
            .contains("border-color:rgb(255 0 0)")
    );
    assert!(
        legacy
            .execution
            .semantic_preservation_telemetry
            .observed_pass_count
            > 0
    );
    assert_eq!(
        legacy
            .execution
            .semantic_preservation_telemetry
            .observed_pass_count,
        legacy
            .execution
            .semantic_preservation_telemetry
            .preserved_pass_count
    );
    assert_eq!(
        legacy
            .execution
            .semantic_preservation_telemetry
            .blocked_pass_count,
        0
    );
    assert!(modern.execution.output_css.contains("light-dark("));
    assert!(modern.execution.output_css.contains("color-mix("));
    assert!(modern.execution.output_css.contains("rgb(from"));
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
            enable_container_static_eval: false,
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
fn chrome_122_target_query_build_preserves_light_dark_supports_guard() {
    let summary = execute_omena_query_consumer_build_style_source_for_target_query_with_options(
        "Theme.css",
        r#"@supports (color: light-dark(#000, #fff)) { .theme { color: red; } }"#,
        "chrome 122",
        OmenaQueryTargetTransformOptionsV0 {
            allow_logical_to_physical: false,
            allow_scope_flatten: false,
            allow_layer_flatten: false,
            enable_supports_static_eval: true,
            enable_media_static_eval: false,
            enable_container_static_eval: false,
            drop_dark_mode_media_queries: false,
        },
    );

    assert!(summary.target_query.is_some());
    let Some(target_query) = summary.target_query.as_ref() else {
        return;
    };
    assert!(!target_query.support.supports_light_dark);
    assert!(
        summary
            .requested_pass_ids
            .iter()
            .any(|pass_id| pass_id == "supports-static-eval")
    );
    assert!(summary.execution.output_css.contains("@supports"));
    assert!(summary.execution.output_css.contains("light-dark"));
    assert!(
        summary
            .execution
            .output_css
            .contains(".theme { color: red; }")
    );
}

#[test]
fn chrome_123_target_query_build_strips_light_dark_supports_guard() {
    let summary = execute_omena_query_consumer_build_style_source_for_target_query_with_options(
        "Theme.css",
        r#"@supports (color: light-dark(#000, #fff)) { .theme { color: red; } }"#,
        "chrome 123",
        OmenaQueryTargetTransformOptionsV0 {
            allow_logical_to_physical: false,
            allow_scope_flatten: false,
            allow_layer_flatten: false,
            enable_supports_static_eval: true,
            enable_media_static_eval: false,
            enable_container_static_eval: false,
            drop_dark_mode_media_queries: false,
        },
    );

    assert!(summary.target_query.is_some());
    let Some(target_query) = summary.target_query.as_ref() else {
        return;
    };
    assert!(target_query.support.supports_light_dark);
    assert!(!summary.execution.output_css.contains("@supports"));
    assert!(
        summary
            .execution
            .output_css
            .contains(".theme { color: red; }")
    );
}

#[test]
fn multi_source_target_query_build_preserves_light_dark_supports_guard() {
    let sources = vec![
        OmenaQueryStyleSourceInputV0 {
            style_path: "Theme.css".to_string(),
            style_source: r#"@supports (color: light-dark(#000, #fff)) { .theme { color: red; } }"#
                .to_string(),
        },
        OmenaQueryStyleSourceInputV0 {
            style_path: "tokens.css".to_string(),
            style_source: ":root { --brand: red; }".to_string(),
        },
    ];
    let summary_result =
        execute_omena_query_consumer_build_style_sources_for_target_query_with_context_and_options(
            "Theme.css",
            &sources,
            "chrome 122",
            &OmenaQueryTransformExecutionContextV0::default(),
            OmenaQueryTargetTransformOptionsV0 {
                allow_logical_to_physical: false,
                allow_scope_flatten: false,
                allow_layer_flatten: false,
                enable_supports_static_eval: true,
                enable_media_static_eval: false,
                enable_container_static_eval: false,
                drop_dark_mode_media_queries: false,
            },
            &[],
        );
    assert!(summary_result.is_ok());
    let Ok(summary) = summary_result else {
        return;
    };

    assert!(summary.target_query.is_some());
    let Some(target_query) = summary.target_query.as_ref() else {
        return;
    };
    assert!(!target_query.support.supports_light_dark);
    assert!(summary.execution.output_css.contains("@supports"));
    assert!(summary.execution.output_css.contains("light-dark"));
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
            enable_container_static_eval: false,
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
fn target_query_layer_flatten_uses_constructed_closed_world_bundle() {
    let summary = execute_omena_query_consumer_build_style_source_for_target_query_with_options(
        "Theme.css",
        r#"@layer theme { .card { color: red; } }"#,
        "ie 11",
        OmenaQueryTargetTransformOptionsV0 {
            allow_logical_to_physical: false,
            allow_scope_flatten: false,
            allow_layer_flatten: true,
            enable_supports_static_eval: false,
            enable_media_static_eval: false,
            enable_container_static_eval: false,
            drop_dark_mode_media_queries: false,
        },
    );

    assert!(
        summary
            .requested_pass_ids
            .contains(&"layer-flatten".to_string())
    );
    assert!(
        summary
            .execution
            .executed_pass_ids
            .contains(&"layer-flatten")
    );
    assert!(
        !summary
            .execution
            .planned_only_pass_ids
            .contains(&"layer-flatten")
    );
    assert!(!summary.execution.output_css.contains("@layer"));
    assert!(summary.open_world_snapshot.is_none());
    assert!(summary.ready_surfaces.contains(&"closedWorldBundle"));
    assert!(summary.ready_surfaces.contains(&"targetQueryBuildFacade"));
}

#[test]
fn consumer_build_accepts_explicit_scss_evaluator_context() {
    let source = ".button { color: $brand; }";
    let context = OmenaQueryTransformExecutionContextV0 {
        scss_module_evaluation: Some(OmenaQueryTransformModuleEvaluationV0 {
            evaluator: "dart-sass-compatible".to_string(),
            product_output_source: Some("nativeEditOutput".to_string()),
            evaluated_css: ".button { color: red; }".to_string(),
            native_edit_output: Some(".button { color: red; }".to_string()),
            native_replacements: Vec::new(),
            native_edits: vec![native_module_evaluation_edit(source, "$brand", "red")],
            oracle: Some(oracle_allowing_native_output()),
        }),
        ..OmenaQueryTransformExecutionContextV0::default()
    };
    let summary =
        execute_omena_query_consumer_build_style_source_for_target_query_with_context_and_options(
            "Button.module.scss",
            source,
            "ie 11",
            &context,
            OmenaQueryTargetTransformOptionsV0 {
                allow_logical_to_physical: false,
                allow_scope_flatten: false,
                allow_layer_flatten: false,
                enable_supports_static_eval: false,
                enable_media_static_eval: false,
                enable_container_static_eval: false,
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
fn consumer_build_preserves_source_when_scss_evaluator_native_edits_diverge() {
    let source = ".button { color: $brand; }";
    let context = OmenaQueryTransformExecutionContextV0 {
        scss_module_evaluation: Some(OmenaQueryTransformModuleEvaluationV0 {
            evaluator: "dart-sass-compatible".to_string(),
            product_output_source: None,
            evaluated_css: ".button { color: red; }".to_string(),
            native_edit_output: None,
            native_replacements: Vec::new(),
            native_edits: vec![native_module_evaluation_edit(source, "$brand", "blue")],
            oracle: None,
        }),
        ..OmenaQueryTransformExecutionContextV0::default()
    };
    let summary =
        execute_omena_query_consumer_build_style_source_for_target_query_with_context_and_options(
            "Button.module.scss",
            source,
            "ie 11",
            &context,
            OmenaQueryTargetTransformOptionsV0 {
                allow_logical_to_physical: false,
                allow_scope_flatten: false,
                allow_layer_flatten: false,
                enable_supports_static_eval: false,
                enable_media_static_eval: false,
                enable_container_static_eval: false,
                drop_dark_mode_media_queries: false,
            },
        );

    assert!(summary.execution.output_css.contains("color: $brand"));
    assert!(!summary.execution.output_css.contains("color: red"));
    assert!(!summary.execution.output_css.contains("color: blue"));
    assert_eq!(
        summary
            .execution
            .outcomes
            .iter()
            .find(|outcome| outcome.pass_id == "scss-module-evaluate")
            .map(|outcome| outcome.detail),
        Some(
            "preserved SCSS source because native evaluator edits did not match the oracle boundary"
        )
    );
}

fn native_module_evaluation_edit(
    source: &str,
    needle: &str,
    replacement: &str,
) -> OmenaQueryTransformModuleEvaluationNativeEditV0 {
    let start = source.find(needle).unwrap_or(source.len());
    assert!(
        start < source.len(),
        "test fixture missing native edit needle: {needle}"
    );
    OmenaQueryTransformModuleEvaluationNativeEditV0 {
        start,
        end: start + needle.len(),
        replacement: replacement.to_string(),
        edit_kind: "valueReplacement".to_string(),
        abstract_value: None,
        abstract_value_kind: None,
    }
}

fn oracle_allowing_native_output() -> OmenaQueryTransformModuleEvaluationOracleV0 {
    OmenaQueryTransformModuleEvaluationOracleV0 {
        mode: "oracleOnly".to_string(),
        product_output_source: "legacyEvaluatedCss".to_string(),
        divergence_count: 0,
        all_legacy_declaration_values_preserved: true,
        ..OmenaQueryTransformModuleEvaluationOracleV0::default()
    }
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
fn consumer_build_derives_nested_next_alias_context_for_composes()
-> Result<(), Box<dyn std::error::Error>> {
    let root = temp_dir("omena_query_transform_next_alias")?;
    let app_dir = root.join("apps/web");
    let source_path = app_dir.join("src/App.module.scss");
    let base_path = app_dir.join("src/styles/base.module.scss");
    fs::create_dir_all(
        base_path
            .parent()
            .ok_or_else(|| std::io::Error::other("base style parent"))?,
    )?;
    fs::write(
        app_dir.join("next.config.mjs"),
        r#"export default { resolve: { alias: { "@styles": "./src/styles" } } };"#,
    )?;
    let sources = vec![
        OmenaQueryStyleSourceInputV0 {
            style_path: source_path.to_string_lossy().to_string(),
            style_source:
                r#".button { composes: base from "@styles/base.module.scss"; color: red; }"#
                    .to_string(),
        },
        OmenaQueryStyleSourceInputV0 {
            style_path: base_path.to_string_lossy().to_string(),
            style_source: ".base { color: blue; }".to_string(),
        },
    ];
    let pass_ids = vec!["composes-resolution".to_string()];

    let summary = execute_omena_query_consumer_build_style_sources_with_context(
        source_path.to_string_lossy().as_ref(),
        &sources,
        &pass_ids,
        &OmenaQueryTransformExecutionContextV0::default(),
        &[],
    )?;

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
            .contains(&"composes-resolution")
    );
    assert!(!summary.execution.output_css.contains("composes:"));
    let _ = fs::remove_dir_all(root);
    Ok(())
}

fn temp_dir(prefix: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let suffix = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)?
        .as_nanos();
    let path = std::env::temp_dir().join(format!("{prefix}-{suffix}"));
    fs::create_dir_all(path.as_path())?;
    Ok(path)
}
