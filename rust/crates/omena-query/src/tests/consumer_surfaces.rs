use crate::{
    OmenaQueryStyleSourceInputV0, OmenaQueryTargetTransformOptionsV0,
    OmenaQueryTransformExecutionContextV0, OmenaQueryTransformModuleEvaluationV0,
    execute_omena_query_consumer_build_style_source,
    execute_omena_query_consumer_build_style_source_for_target_query,
    execute_omena_query_consumer_build_style_source_for_target_query_with_context_and_options,
    execute_omena_query_consumer_build_style_source_for_target_query_with_options,
    execute_omena_query_consumer_build_style_sources_with_context,
    summarize_omena_query_consumer_check_style_source,
};
use std::{fs, path::PathBuf, time::SystemTime};

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
fn target_query_build_emits_expanded_vendor_prefix_matrix() {
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
    assert!(summary.execution.output_css.contains("display: -ms-grid"));
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
            .contains("-ms-transform: translateX(1px)")
    );
    assert!(summary.execution.output_css.contains("-webkit-columns: 2"));
    assert!(summary.execution.output_css.contains("-moz-columns: 2"));
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
            .contains("-webkit-box-align: start")
    );
    assert!(
        summary
            .execution
            .output_css
            .contains("-ms-flex-pack: justify")
    );
    assert!(
        summary
            .execution
            .output_css
            .contains("-webkit-box-direction: reverse")
    );
    assert!(summary.execution.output_css.contains("-ms-flex-wrap: wrap"));
    assert!(
        summary
            .execution
            .output_css
            .contains("@supports ((display: grid) or (display: -ms-grid))")
    );
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
            native_replacements: Vec::new(),
            native_edits: Vec::new(),
            oracle: None,
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
