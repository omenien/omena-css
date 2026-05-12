//! Browser-side in-memory bindings for the Omena CSS parser and transform surface.

use omena_query::{
    OmenaQueryConsumerBuildSummaryV0 as OmenaWasmBuildSummaryV0,
    OmenaQueryConsumerCheckSummaryV0 as OmenaWasmCheckSummaryV0,
    OmenaQueryEngineInputV2 as OmenaWasmEngineInputV2,
    OmenaQueryExpressionDomainSelectorProjectionV0 as OmenaWasmExpressionDomainSelectorProjectionV0,
    OmenaQueryStylePackageManifestV0 as OmenaWasmStylePackageManifestV0,
    OmenaQueryStyleSourceInputV0 as OmenaWasmStyleSourceInputV0,
    OmenaQueryTargetTransformOptionsV0 as OmenaWasmTargetTransformOptionsV0,
    OmenaQueryTransformContextFromEngineInputSummaryV0 as OmenaWasmTransformContextFromEngineInputSummaryV0,
    OmenaQueryTransformExecutionContextV0 as OmenaWasmTransformExecutionContextV0,
    OmenaQueryTransformPassSummaryV0 as OmenaWasmPassSummaryV0,
    conservative_omena_query_target_options, execute_omena_query_consumer_build_style_source,
    execute_omena_query_consumer_build_style_source_for_target_query,
    execute_omena_query_consumer_build_style_source_for_target_query_with_context_and_options,
    execute_omena_query_consumer_build_style_source_for_target_query_with_options,
    execute_omena_query_consumer_build_style_source_with_context,
    execute_omena_query_consumer_build_style_source_with_engine_input_context,
    execute_omena_query_consumer_build_style_sources_for_target_query_with_context_and_options,
    execute_omena_query_consumer_build_style_sources_with_context,
    list_omena_query_transform_pass_summaries, summarize_omena_query_consumer_check_style_source,
    summarize_omena_query_expression_domain_selector_projection,
    summarize_omena_query_transform_context_from_engine_input,
};
use serde::Serialize;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(js_name = checkStyleSource)]
pub fn check_style_source(source: &str, path: &str) -> Result<JsValue, JsValue> {
    to_js_value(&check_style_source_summary(source, path))
}

#[wasm_bindgen(js_name = buildStyleSource)]
pub fn build_style_source(source: &str, path: &str, pass_ids: JsValue) -> Result<JsValue, JsValue> {
    let pass_ids = parse_pass_ids_value(pass_ids)?;
    to_js_value(&build_style_source_summary(source, path, &pass_ids))
}

#[wasm_bindgen(js_name = buildStyleSourceWithContext)]
pub fn build_style_source_with_context(
    source: &str,
    path: &str,
    pass_ids: JsValue,
    context: JsValue,
) -> Result<JsValue, JsValue> {
    let pass_ids = parse_pass_ids_value(pass_ids)?;
    let context = parse_context_value(context)?;
    to_js_value(&build_style_source_with_context_summary(
        source, path, &pass_ids, &context,
    ))
}

#[wasm_bindgen(js_name = buildStyleSourceWithEngineInputContext)]
pub fn build_style_source_with_engine_input_context(
    source: &str,
    path: &str,
    pass_ids: JsValue,
    input: JsValue,
    closed_style_world: bool,
) -> Result<JsValue, JsValue> {
    let pass_ids = parse_pass_ids_value(pass_ids)?;
    let input = parse_engine_input_value(input)?;
    to_js_value(&build_style_source_with_engine_input_context_summary(
        source,
        path,
        &pass_ids,
        &input,
        closed_style_world,
    ))
}

#[wasm_bindgen(js_name = buildStyleSourceForTargetQuery)]
pub fn build_style_source_for_target_query(
    source: &str,
    path: &str,
    target_query: &str,
) -> Result<JsValue, JsValue> {
    to_js_value(&build_style_source_for_target_query_summary(
        source,
        path,
        target_query,
    ))
}

#[wasm_bindgen(js_name = buildStyleSourceForTargetQueryWithOptions)]
pub fn build_style_source_for_target_query_with_options(
    source: &str,
    path: &str,
    target_query: &str,
    target_options: JsValue,
) -> Result<JsValue, JsValue> {
    let target_options = parse_target_options_value(target_options)?;
    to_js_value(&build_style_source_for_target_query_with_options_summary(
        source,
        path,
        target_query,
        target_options,
    ))
}

#[wasm_bindgen(js_name = buildStyleSourceForTargetQueryWithContext)]
pub fn build_style_source_for_target_query_with_context(
    source: &str,
    path: &str,
    target_query: &str,
    target_options: JsValue,
    context: JsValue,
) -> Result<JsValue, JsValue> {
    let target_options = parse_target_options_value(target_options)?;
    let context = parse_context_value(context)?;
    to_js_value(&build_style_source_for_target_query_with_context_summary(
        source,
        path,
        target_query,
        target_options,
        &context,
    ))
}

#[wasm_bindgen(js_name = buildStyleSourcesWithContext)]
pub fn build_style_sources_with_context(
    target_path: &str,
    sources: JsValue,
    pass_ids: JsValue,
    context: JsValue,
    package_manifests: JsValue,
) -> Result<JsValue, JsValue> {
    let sources = parse_style_sources_value(sources)?;
    let pass_ids = parse_pass_ids_value(pass_ids)?;
    let context = parse_context_value(context)?;
    let package_manifests = parse_package_manifests_value(package_manifests)?;
    let summary = build_style_sources_with_context_summary(
        target_path,
        &sources,
        &pass_ids,
        &context,
        &package_manifests,
    )?;
    to_js_value(&summary)
}

#[wasm_bindgen(js_name = buildStyleSourcesForTargetQueryWithContext)]
pub fn build_style_sources_for_target_query_with_context(
    target_path: &str,
    sources: JsValue,
    target_query: &str,
    target_options: JsValue,
    context: JsValue,
    package_manifests: JsValue,
) -> Result<JsValue, JsValue> {
    let sources = parse_style_sources_value(sources)?;
    let target_options = parse_target_options_value(target_options)?;
    let context = parse_context_value(context)?;
    let package_manifests = parse_package_manifests_value(package_manifests)?;
    let summary = build_style_sources_for_target_query_with_context_summary(
        target_path,
        &sources,
        target_query,
        target_options,
        &context,
        &package_manifests,
    )?;
    to_js_value(&summary)
}

#[wasm_bindgen(js_name = listTransformPasses)]
pub fn list_transform_passes() -> Result<JsValue, JsValue> {
    to_js_value(&list_transform_pass_summaries())
}

#[wasm_bindgen(js_name = expressionDomainSelectorProjection)]
pub fn expression_domain_selector_projection(input: JsValue) -> Result<JsValue, JsValue> {
    let input = parse_engine_input_value(input)?;
    to_js_value(&expression_domain_selector_projection_summary(&input))
}

#[wasm_bindgen(js_name = transformContextFromEngineInput)]
pub fn transform_context_from_engine_input(
    input: JsValue,
    target_path: &str,
    closed_style_world: bool,
) -> Result<JsValue, JsValue> {
    let input = parse_engine_input_value(input)?;
    to_js_value(&transform_context_from_engine_input_summary(
        &input,
        target_path,
        closed_style_world,
    ))
}

pub fn check_style_source_summary(source: &str, path: &str) -> OmenaWasmCheckSummaryV0 {
    let path = effective_path(path);
    summarize_omena_query_consumer_check_style_source(path, source)
}

pub fn build_style_source_summary(
    source: &str,
    path: &str,
    pass_ids: &[String],
) -> OmenaWasmBuildSummaryV0 {
    let path = effective_path(path);
    execute_omena_query_consumer_build_style_source(path, source, pass_ids)
}

pub fn build_style_source_with_context_summary(
    source: &str,
    path: &str,
    pass_ids: &[String],
    context: &OmenaWasmTransformExecutionContextV0,
) -> OmenaWasmBuildSummaryV0 {
    let path = effective_path(path);
    execute_omena_query_consumer_build_style_source_with_context(path, source, pass_ids, context)
}

pub fn build_style_source_with_engine_input_context_summary(
    source: &str,
    path: &str,
    pass_ids: &[String],
    input: &OmenaWasmEngineInputV2,
    closed_style_world: bool,
) -> OmenaWasmBuildSummaryV0 {
    let path = effective_path(path);
    execute_omena_query_consumer_build_style_source_with_engine_input_context(
        path,
        source,
        pass_ids,
        input,
        closed_style_world,
    )
}

pub fn build_style_source_for_target_query_summary(
    source: &str,
    path: &str,
    target_query: &str,
) -> OmenaWasmBuildSummaryV0 {
    let path = effective_path(path);
    execute_omena_query_consumer_build_style_source_for_target_query(path, source, target_query)
}

pub fn build_style_source_for_target_query_with_options_summary(
    source: &str,
    path: &str,
    target_query: &str,
    target_options: OmenaWasmTargetTransformOptionsV0,
) -> OmenaWasmBuildSummaryV0 {
    let path = effective_path(path);
    execute_omena_query_consumer_build_style_source_for_target_query_with_options(
        path,
        source,
        target_query,
        target_options,
    )
}

pub fn build_style_source_for_target_query_with_context_summary(
    source: &str,
    path: &str,
    target_query: &str,
    target_options: OmenaWasmTargetTransformOptionsV0,
    context: &OmenaWasmTransformExecutionContextV0,
) -> OmenaWasmBuildSummaryV0 {
    let path = effective_path(path);
    execute_omena_query_consumer_build_style_source_for_target_query_with_context_and_options(
        path,
        source,
        target_query,
        context,
        target_options,
    )
}

pub fn build_style_sources_with_context_summary(
    target_path: &str,
    sources: &[OmenaWasmStyleSourceInputV0],
    pass_ids: &[String],
    context: &OmenaWasmTransformExecutionContextV0,
    package_manifests: &[OmenaWasmStylePackageManifestV0],
) -> Result<OmenaWasmBuildSummaryV0, JsValue> {
    execute_omena_query_consumer_build_style_sources_with_context(
        target_path,
        sources,
        pass_ids,
        context,
        package_manifests,
    )
    .map_err(|error| JsValue::from_str(&error))
}

pub fn build_style_sources_for_target_query_with_context_summary(
    target_path: &str,
    sources: &[OmenaWasmStyleSourceInputV0],
    target_query: &str,
    target_options: OmenaWasmTargetTransformOptionsV0,
    context: &OmenaWasmTransformExecutionContextV0,
    package_manifests: &[OmenaWasmStylePackageManifestV0],
) -> Result<OmenaWasmBuildSummaryV0, JsValue> {
    execute_omena_query_consumer_build_style_sources_for_target_query_with_context_and_options(
        target_path,
        sources,
        target_query,
        context,
        target_options,
        package_manifests,
    )
    .map_err(|error| JsValue::from_str(&error))
}

pub fn list_transform_pass_summaries() -> Vec<OmenaWasmPassSummaryV0> {
    list_omena_query_transform_pass_summaries()
}

pub fn expression_domain_selector_projection_summary(
    input: &OmenaWasmEngineInputV2,
) -> OmenaWasmExpressionDomainSelectorProjectionV0 {
    summarize_omena_query_expression_domain_selector_projection(input)
}

pub fn transform_context_from_engine_input_summary(
    input: &OmenaWasmEngineInputV2,
    target_path: &str,
    closed_style_world: bool,
) -> OmenaWasmTransformContextFromEngineInputSummaryV0 {
    summarize_omena_query_transform_context_from_engine_input(
        input,
        target_path,
        closed_style_world,
    )
}

fn parse_pass_ids_value(value: JsValue) -> Result<Vec<String>, JsValue> {
    if value.is_null() || value.is_undefined() {
        return Ok(Vec::new());
    }

    serde_wasm_bindgen::from_value(value).map_err(|error| {
        JsValue::from_str(&format!(
            "passIds must be an array of transform pass id strings: {error}"
        ))
    })
}

fn parse_target_options_value(
    value: JsValue,
) -> Result<OmenaWasmTargetTransformOptionsV0, JsValue> {
    if value.is_null() || value.is_undefined() {
        return Ok(conservative_omena_query_target_options());
    }

    serde_wasm_bindgen::from_value(value).map_err(|error| {
        JsValue::from_str(&format!(
            "targetOptions must be an object with camelCase target transform option booleans: {error}"
        ))
    })
}

fn parse_context_value(value: JsValue) -> Result<OmenaWasmTransformExecutionContextV0, JsValue> {
    if value.is_null() || value.is_undefined() {
        return Ok(OmenaWasmTransformExecutionContextV0::default());
    }

    serde_wasm_bindgen::from_value(value).map_err(|error| {
        JsValue::from_str(&format!(
            "context must be a TransformExecutionContextV0-compatible object: {error}"
        ))
    })
}

fn parse_style_sources_value(value: JsValue) -> Result<Vec<OmenaWasmStyleSourceInputV0>, JsValue> {
    serde_wasm_bindgen::from_value(value).map_err(|error| {
        JsValue::from_str(&format!(
            "sources must be an array of {{stylePath, styleSource}} objects: {error}"
        ))
    })
}

fn parse_package_manifests_value(
    value: JsValue,
) -> Result<Vec<OmenaWasmStylePackageManifestV0>, JsValue> {
    if value.is_null() || value.is_undefined() {
        return Ok(Vec::new());
    }

    serde_wasm_bindgen::from_value(value).map_err(|error| {
        JsValue::from_str(&format!(
            "packageManifests must be an array of package manifest objects: {error}"
        ))
    })
}

fn parse_engine_input_value(value: JsValue) -> Result<OmenaWasmEngineInputV2, JsValue> {
    serde_wasm_bindgen::from_value(value)
        .map_err(|error| JsValue::from_str(&format!("failed to parse engine input: {error}")))
}

fn to_js_value<T: Serialize>(value: &T) -> Result<JsValue, JsValue> {
    serde_wasm_bindgen::to_value(value)
        .map_err(|error| JsValue::from_str(&format!("failed to serialize result: {error}")))
}

fn effective_path(path: &str) -> &str {
    if path.trim().is_empty() {
        "style.css"
    } else {
        path
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_parser_facts_for_browser_source() {
        let summary = check_style_source_summary(
            ".card { color: red; }\n:root { --brand: blue; }",
            "fixture.module.css",
        );

        assert_eq!(summary.product, "omena-query.consumer-check-style-source");
        assert_eq!(summary.style_path, "fixture.module.css");
        assert_eq!(summary.dialect, "css");
        assert_eq!(summary.parser_error_count, 0);
        assert_eq!(summary.class_selector_count, 1);
        assert_eq!(summary.custom_property_count, 1);
    }

    #[test]
    fn builds_css_with_requested_passes() {
        let pass_ids = vec![
            "whitespace-strip".to_string(),
            "color-compression".to_string(),
        ];
        let summary =
            build_style_source_summary(".card { color: #ffffff; }", "fixture.css", &pass_ids);

        assert_eq!(summary.product, "omena-query.consumer-build-style-source");
        assert!(summary.unknown_pass_ids.is_empty());
        assert!(summary.execution.output_css.contains("#fff"));
    }

    #[test]
    fn builds_css_from_target_query_for_browser_clients() {
        let summary = build_style_source_for_target_query_summary(
            ".card { display: flex; color: light-dark(#000, #fff); }",
            "fixture.css",
            "ie 11",
        );

        assert_eq!(summary.product, "omena-query.consumer-build-style-source");
        assert!(summary.unknown_pass_ids.is_empty());
        assert!(summary.target_query.is_some());
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
    }

    #[test]
    fn builds_css_from_target_query_options_for_browser_clients() {
        let summary = build_style_source_for_target_query_with_options_summary(
            ".card { margin-inline: 1rem; }",
            "fixture.css",
            "ie 11",
            OmenaWasmTargetTransformOptionsV0 {
                allow_logical_to_physical: true,
                allow_scope_flatten: false,
                allow_layer_flatten: false,
                enable_supports_static_eval: false,
                enable_media_static_eval: false,
            },
        );

        assert!(summary.unknown_pass_ids.is_empty());
        assert!(
            summary
                .requested_pass_ids
                .iter()
                .any(|pass_id| pass_id == "logical-to-physical")
        );
    }

    #[test]
    fn builds_css_from_evaluator_context_for_browser_clients() {
        let context = OmenaWasmTransformExecutionContextV0 {
            scss_module_evaluation: Some(omena_query::OmenaQueryTransformModuleEvaluationV0 {
                evaluator: "dart-sass-compatible".to_string(),
                evaluated_css: ".card { color: red; }".to_string(),
            }),
            ..OmenaWasmTransformExecutionContextV0::default()
        };
        let summary = build_style_source_for_target_query_with_context_summary(
            "$brand: red; .card { color: $brand; }",
            "fixture.module.scss",
            "ie 11",
            OmenaWasmTargetTransformOptionsV0 {
                allow_logical_to_physical: false,
                allow_scope_flatten: false,
                allow_layer_flatten: false,
                enable_supports_static_eval: false,
                enable_media_static_eval: false,
            },
            &context,
        );

        assert!(
            summary
                .execution
                .executed_pass_ids
                .contains(&"scss-module-evaluate")
        );
        assert!(summary.execution.output_css.contains("._card_0"));
    }

    #[test]
    fn builds_workspace_sources_for_browser_clients() {
        let sources = vec![
            OmenaWasmStyleSourceInputV0 {
                style_path: "Button.module.css".to_string(),
                style_source:
                    r#"@import "./tokens.css"; .button { composes: base; color: var(--brand); } .base { color: blue; }"#
                        .to_string(),
            },
            OmenaWasmStyleSourceInputV0 {
                style_path: "tokens.css".to_string(),
                style_source: ":root { --brand: red; }".to_string(),
            },
        ];
        let pass_ids = vec![
            "import-inline".to_string(),
            "composes-resolution".to_string(),
        ];
        let summary_result = build_style_sources_with_context_summary(
            "Button.module.css",
            &sources,
            &pass_ids,
            &OmenaWasmTransformExecutionContextV0::default(),
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
        assert!(!summary.execution.output_css.contains("@import"));
        assert!(!summary.execution.output_css.contains("composes:"));
    }

    #[test]
    fn builds_css_from_engine_input_context_for_browser_clients() {
        let input = serde_json::from_str::<OmenaWasmEngineInputV2>(
            reduced_product_projection_engine_input_json(),
        );
        assert!(input.is_ok());
        let Ok(input) = input else {
            return;
        };
        let pass_ids = vec!["tree-shake-class".to_string()];
        let summary = build_style_source_with_engine_input_context_summary(
            r#".btn-primary--active { color: red; } .btn-secondary--active { color: blue; } .card-active { color: gray; }"#,
            "/tmp/App.module.scss",
            &pass_ids,
            &input,
            true,
        );

        assert!(
            summary
                .execution
                .output_css
                .contains(".btn-primary--active")
        );
        assert!(
            summary
                .execution
                .output_css
                .contains(".btn-secondary--active")
        );
        assert!(!summary.execution.output_css.contains(".card-active"));
        assert!(
            summary
                .execution
                .executed_pass_ids
                .contains(&"tree-shake-class")
        );
        assert!(
            summary
                .ready_surfaces
                .contains(&"semanticReachabilityTransformContext")
        );
    }

    #[test]
    fn builds_css_from_engine_input_style_sources_for_browser_clients() {
        let input = serde_json::from_str::<OmenaWasmEngineInputV2>(
            workspace_style_source_engine_input_json(),
        );
        assert!(input.is_ok());
        let Ok(input) = input else {
            return;
        };
        let pass_ids = vec![
            "import-inline".to_string(),
            "composes-resolution".to_string(),
        ];
        let summary = build_style_source_with_engine_input_context_summary(
            r#"@import "./tokens.css" supports(display: grid); .button { composes: base; color: var(--brand); } .base { color: blue; }"#,
            "/tmp/Button.module.css",
            &pass_ids,
            &input,
            false,
        );

        assert!(
            summary
                .ready_surfaces
                .contains(&"semanticReachabilityTransformContext")
        );
        assert!(
            summary
                .execution
                .output_css
                .contains("@supports (display: grid) { :root { --brand: red; } }")
        );
        assert!(!summary.execution.output_css.contains("@import"));
        assert!(!summary.execution.output_css.contains("composes:"));

        let context =
            transform_context_from_engine_input_summary(&input, "/tmp/Button.module.css", false);
        assert_eq!(context.style_source_count, 2);
        assert_eq!(context.import_inline_count, 1);
    }

    #[test]
    fn exposes_transform_context_reachability_sources_for_browser_clients() {
        let input = serde_json::from_str::<OmenaWasmEngineInputV2>(
            reduced_product_projection_engine_input_json(),
        );
        assert!(input.is_ok());
        let Ok(input) = input else {
            return;
        };
        let summary =
            transform_context_from_engine_input_summary(&input, "/tmp/App.module.scss", true);

        assert_eq!(
            summary.product,
            "omena-query.transform-context-from-engine-input"
        );
        assert_eq!(summary.selected_projection_count, 3);
        assert!(
            summary
                .reachability_sources
                .iter()
                .any(|source| source.node_id == "file-merge")
        );
    }

    #[test]
    fn reports_unknown_passes_without_failing_known_execution() {
        let pass_ids = vec!["whitespace-strip".to_string(), "unknown-pass".to_string()];
        let summary = build_style_source_summary(".card { color: red; }", "fixture.css", &pass_ids);

        assert_eq!(summary.unknown_pass_ids, vec!["unknown-pass"]);
        assert!(
            summary
                .execution
                .executed_pass_ids
                .contains(&"whitespace-strip")
        );
    }

    #[test]
    fn lists_transform_passes_for_browser_clients() {
        let passes = list_transform_pass_summaries();

        assert_eq!(passes.len(), 40);
        assert!(passes.iter().any(|pass| pass.id == "whitespace-strip"));
    }

    fn reduced_product_projection_engine_input_json() -> &'static str {
        r#"{
          "version": "2",
          "sources": [
            {
              "document": {
                "classExpressions": [
                  {
                    "id": "expr-primary",
                    "kind": "symbolRef",
                    "scssModulePath": "/tmp/App.module.scss",
                    "range": {
                      "start": { "line": 4, "character": 12 },
                      "end": { "line": 4, "character": 16 }
                    },
                    "className": null,
                    "rootBindingDeclId": "decl-primary",
                    "accessPath": null
                  },
                  {
                    "id": "expr-secondary",
                    "kind": "symbolRef",
                    "scssModulePath": "/tmp/App.module.scss",
                    "range": {
                      "start": { "line": 5, "character": 12 },
                      "end": { "line": 5, "character": 16 }
                    },
                    "className": null,
                    "rootBindingDeclId": "decl-secondary",
                    "accessPath": null
                  }
                ]
              }
            }
          ],
          "styles": [
            {
              "filePath": "/tmp/App.module.scss",
              "document": {
                "selectors": [
                  {
                    "name": "btn-primary--active",
                    "viewKind": "canonical",
                    "canonicalName": "btn-primary--active",
                    "range": {
                      "start": { "line": 1, "character": 1 },
                      "end": { "line": 1, "character": 21 }
                    },
                    "nestedSafety": "safe",
                    "composes": null,
                    "bemSuffix": null
                  },
                  {
                    "name": "btn-secondary--active",
                    "viewKind": "canonical",
                    "canonicalName": "btn-secondary--active",
                    "range": {
                      "start": { "line": 2, "character": 1 },
                      "end": { "line": 2, "character": 23 }
                    },
                    "nestedSafety": "safe",
                    "composes": null,
                    "bemSuffix": null
                  },
                  {
                    "name": "card-active",
                    "viewKind": "canonical",
                    "canonicalName": "card-active",
                    "range": {
                      "start": { "line": 3, "character": 1 },
                      "end": { "line": 3, "character": 12 }
                    },
                    "nestedSafety": "safe",
                    "composes": null,
                    "bemSuffix": null
                  }
                ]
              }
            }
          ],
          "typeFacts": [
            {
              "filePath": "/tmp/App.tsx",
              "expressionId": "expr-primary",
              "facts": {
                "kind": "constrained",
                "constraintKind": "prefixSuffix",
                "values": null,
                "prefix": "btn-primary-",
                "suffix": "-active",
                "minLen": 19,
                "maxLen": null,
                "charMust": null,
                "charMay": null,
                "mayIncludeOtherChars": null
              }
            },
            {
              "filePath": "/tmp/App.tsx",
              "expressionId": "expr-secondary",
              "facts": {
                "kind": "constrained",
                "constraintKind": "prefixSuffix",
                "values": null,
                "prefix": "btn-secondary-",
                "suffix": "-active",
                "minLen": 21,
                "maxLen": null,
                "charMust": null,
                "charMay": null,
                "mayIncludeOtherChars": null
              }
            }
          ]
        }"#
    }

    fn workspace_style_source_engine_input_json() -> &'static str {
        r#"{
          "version": "2",
          "sources": [],
          "styles": [
            {
              "filePath": "/tmp/Button.module.css",
              "source": "@import \"./tokens.css\" supports(display: grid); .button { composes: base; color: var(--brand); } .base { color: blue; }",
              "document": {
                "selectors": [
                  {
                    "name": "button",
                    "viewKind": "canonical",
                    "canonicalName": "button",
                    "range": {
                      "start": { "line": 1, "character": 1 },
                      "end": { "line": 1, "character": 7 }
                    },
                    "nestedSafety": "safe",
                    "composes": null,
                    "bemSuffix": null
                  },
                  {
                    "name": "base",
                    "viewKind": "canonical",
                    "canonicalName": "base",
                    "range": {
                      "start": { "line": 1, "character": 50 },
                      "end": { "line": 1, "character": 54 }
                    },
                    "nestedSafety": "safe",
                    "composes": null,
                    "bemSuffix": null
                  }
                ]
              }
            },
            {
              "filePath": "/tmp/tokens.css",
              "source": ":root { --brand: red; }",
              "document": { "selectors": [] }
            }
          ],
          "typeFacts": []
        }"#
    }
}
