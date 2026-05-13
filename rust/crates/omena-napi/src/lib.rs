//! Node native bindings for the Omena CSS parser and transform surface.

use napi_derive::napi;
use omena_query::{
    OmenaQueryConsumerBuildSummaryV0 as OmenaNapiBuildSummaryV0,
    OmenaQueryConsumerCheckSummaryV0 as OmenaNapiCheckSummaryV0,
    OmenaQueryEngineInputV2 as OmenaNapiEngineInputV2,
    OmenaQueryExpressionDomainSelectorProjectionV0 as OmenaNapiExpressionDomainSelectorProjectionV0,
    OmenaQueryStylePackageManifestV0 as OmenaNapiStylePackageManifestV0,
    OmenaQueryStyleSourceInputV0 as OmenaNapiStyleSourceInputV0,
    OmenaQueryTargetTransformOptionsV0 as OmenaNapiTargetTransformOptionsV0,
    OmenaQueryTransformContextFromEngineInputSummaryV0 as OmenaNapiTransformContextFromEngineInputSummaryV0,
    OmenaQueryTransformExecutionContextV0 as OmenaNapiTransformExecutionContextV0,
    OmenaQueryTransformPassSummaryV0 as OmenaNapiPassSummaryV0,
    execute_omena_query_consumer_build_style_source,
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

#[napi(js_name = "checkStyleSourceJson")]
pub fn check_style_source_json(source: String, path: String) -> napi::Result<String> {
    to_json_string(&check_style_source_summary(&source, &path))
}

#[napi(js_name = "buildStyleSourceJson")]
pub fn build_style_source_json(
    source: String,
    path: String,
    pass_ids: Vec<String>,
) -> napi::Result<String> {
    to_json_string(&build_style_source_summary(&source, &path, &pass_ids))
}

#[napi(js_name = "buildStyleSourceWithContextJson")]
pub fn build_style_source_with_context_json(
    source: String,
    path: String,
    pass_ids: Vec<String>,
    context_json: String,
) -> napi::Result<String> {
    let context = parse_context_json(&context_json)?;
    to_json_string(&build_style_source_with_context_summary(
        &source, &path, &pass_ids, &context,
    ))
}

#[napi(js_name = "buildStyleSourceWithEngineInputContextJson")]
pub fn build_style_source_with_engine_input_context_json(
    source: String,
    path: String,
    pass_ids: Vec<String>,
    input_json: String,
    closed_style_world: bool,
) -> napi::Result<String> {
    let input = parse_engine_input_json(&input_json)?;
    to_json_string(&build_style_source_with_engine_input_context_summary(
        &source,
        &path,
        &pass_ids,
        &input,
        closed_style_world,
    ))
}

#[napi(js_name = "buildStyleSourceForTargetQueryJson")]
pub fn build_style_source_for_target_query_json(
    source: String,
    path: String,
    target_query: String,
) -> napi::Result<String> {
    to_json_string(&build_style_source_for_target_query_summary(
        &source,
        &path,
        &target_query,
    ))
}

#[napi(js_name = "buildStyleSourceForTargetQueryWithOptionsJson")]
pub fn build_style_source_for_target_query_with_options_json(
    source: String,
    path: String,
    target_query: String,
    target_options_json: String,
) -> napi::Result<String> {
    let target_options = parse_target_options_json(&target_options_json)?;
    to_json_string(&build_style_source_for_target_query_with_options_summary(
        &source,
        &path,
        &target_query,
        target_options,
    ))
}

#[napi(js_name = "buildStyleSourceForTargetQueryWithContextJson")]
pub fn build_style_source_for_target_query_with_context_json(
    source: String,
    path: String,
    target_query: String,
    target_options_json: String,
    context_json: String,
) -> napi::Result<String> {
    let target_options = parse_target_options_json(&target_options_json)?;
    let context = parse_context_json(&context_json)?;
    to_json_string(&build_style_source_for_target_query_with_context_summary(
        &source,
        &path,
        &target_query,
        target_options,
        &context,
    ))
}

#[napi(js_name = "buildStyleSourcesWithContextJson")]
pub fn build_style_sources_with_context_json(
    target_path: String,
    sources_json: String,
    pass_ids: Vec<String>,
    context_json: String,
    package_manifests_json: String,
) -> napi::Result<String> {
    let sources = parse_style_sources_json(&sources_json)?;
    let context = parse_context_json(&context_json)?;
    let package_manifests = parse_package_manifests_json(&package_manifests_json)?;
    to_json_string(&build_style_sources_with_context_summary(
        &target_path,
        &sources,
        &pass_ids,
        &context,
        &package_manifests,
    )?)
}

#[napi(js_name = "buildStyleSourcesForTargetQueryWithContextJson")]
pub fn build_style_sources_for_target_query_with_context_json(
    target_path: String,
    sources_json: String,
    target_query: String,
    target_options_json: String,
    context_json: String,
    package_manifests_json: String,
) -> napi::Result<String> {
    let sources = parse_style_sources_json(&sources_json)?;
    let target_options = parse_target_options_json(&target_options_json)?;
    let context = parse_context_json(&context_json)?;
    let package_manifests = parse_package_manifests_json(&package_manifests_json)?;
    to_json_string(&build_style_sources_for_target_query_with_context_summary(
        &target_path,
        &sources,
        &target_query,
        target_options,
        &context,
        &package_manifests,
    )?)
}

#[napi(js_name = "listTransformPassesJson")]
pub fn list_transform_passes_json() -> napi::Result<String> {
    to_json_string(&list_transform_pass_summaries())
}

#[napi(js_name = "expressionDomainSelectorProjectionJson")]
pub fn expression_domain_selector_projection_json(input_json: String) -> napi::Result<String> {
    let input = parse_engine_input_json(&input_json)?;
    to_json_string(&expression_domain_selector_projection_summary(&input))
}

#[napi(js_name = "transformContextFromEngineInputJson")]
pub fn transform_context_from_engine_input_json(
    input_json: String,
    target_path: String,
    closed_style_world: bool,
) -> napi::Result<String> {
    let input = parse_engine_input_json(&input_json)?;
    to_json_string(&transform_context_from_engine_input_summary(
        &input,
        &target_path,
        closed_style_world,
    ))
}

pub fn check_style_source_summary(source: &str, path: &str) -> OmenaNapiCheckSummaryV0 {
    let path = effective_path(path);
    summarize_omena_query_consumer_check_style_source(path, source)
}

pub fn build_style_source_summary(
    source: &str,
    path: &str,
    pass_ids: &[String],
) -> OmenaNapiBuildSummaryV0 {
    let path = effective_path(path);
    execute_omena_query_consumer_build_style_source(path, source, pass_ids)
}

pub fn build_style_source_with_context_summary(
    source: &str,
    path: &str,
    pass_ids: &[String],
    context: &OmenaNapiTransformExecutionContextV0,
) -> OmenaNapiBuildSummaryV0 {
    let path = effective_path(path);
    execute_omena_query_consumer_build_style_source_with_context(path, source, pass_ids, context)
}

pub fn build_style_source_with_engine_input_context_summary(
    source: &str,
    path: &str,
    pass_ids: &[String],
    input: &OmenaNapiEngineInputV2,
    closed_style_world: bool,
) -> OmenaNapiBuildSummaryV0 {
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
) -> OmenaNapiBuildSummaryV0 {
    let path = effective_path(path);
    execute_omena_query_consumer_build_style_source_for_target_query(path, source, target_query)
}

pub fn build_style_source_for_target_query_with_options_summary(
    source: &str,
    path: &str,
    target_query: &str,
    target_options: OmenaNapiTargetTransformOptionsV0,
) -> OmenaNapiBuildSummaryV0 {
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
    target_options: OmenaNapiTargetTransformOptionsV0,
    context: &OmenaNapiTransformExecutionContextV0,
) -> OmenaNapiBuildSummaryV0 {
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
    sources: &[OmenaNapiStyleSourceInputV0],
    pass_ids: &[String],
    context: &OmenaNapiTransformExecutionContextV0,
    package_manifests: &[OmenaNapiStylePackageManifestV0],
) -> napi::Result<OmenaNapiBuildSummaryV0> {
    execute_omena_query_consumer_build_style_sources_with_context(
        target_path,
        sources,
        pass_ids,
        context,
        package_manifests,
    )
    .map_err(napi::Error::from_reason)
}

pub fn build_style_sources_for_target_query_with_context_summary(
    target_path: &str,
    sources: &[OmenaNapiStyleSourceInputV0],
    target_query: &str,
    target_options: OmenaNapiTargetTransformOptionsV0,
    context: &OmenaNapiTransformExecutionContextV0,
    package_manifests: &[OmenaNapiStylePackageManifestV0],
) -> napi::Result<OmenaNapiBuildSummaryV0> {
    execute_omena_query_consumer_build_style_sources_for_target_query_with_context_and_options(
        target_path,
        sources,
        target_query,
        context,
        target_options,
        package_manifests,
    )
    .map_err(napi::Error::from_reason)
}

pub fn list_transform_pass_summaries() -> Vec<OmenaNapiPassSummaryV0> {
    list_omena_query_transform_pass_summaries()
}

pub fn expression_domain_selector_projection_summary(
    input: &OmenaNapiEngineInputV2,
) -> OmenaNapiExpressionDomainSelectorProjectionV0 {
    summarize_omena_query_expression_domain_selector_projection(input)
}

pub fn transform_context_from_engine_input_summary(
    input: &OmenaNapiEngineInputV2,
    target_path: &str,
    closed_style_world: bool,
) -> OmenaNapiTransformContextFromEngineInputSummaryV0 {
    summarize_omena_query_transform_context_from_engine_input(
        input,
        target_path,
        closed_style_world,
    )
}

fn parse_target_options_json(
    target_options_json: &str,
) -> napi::Result<OmenaNapiTargetTransformOptionsV0> {
    serde_json::from_str(target_options_json).map_err(|error| {
        napi::Error::from_reason(format!("failed to parse target options JSON: {error}"))
    })
}

fn parse_context_json(context_json: &str) -> napi::Result<OmenaNapiTransformExecutionContextV0> {
    serde_json::from_str(context_json).map_err(|error| {
        napi::Error::from_reason(format!("failed to parse transform context JSON: {error}"))
    })
}

fn parse_style_sources_json(sources_json: &str) -> napi::Result<Vec<OmenaNapiStyleSourceInputV0>> {
    serde_json::from_str(sources_json).map_err(|error| {
        napi::Error::from_reason(format!("failed to parse style sources JSON: {error}"))
    })
}

fn parse_package_manifests_json(
    package_manifests_json: &str,
) -> napi::Result<Vec<OmenaNapiStylePackageManifestV0>> {
    if package_manifests_json.trim().is_empty() {
        return Ok(Vec::new());
    }
    serde_json::from_str(package_manifests_json).map_err(|error| {
        napi::Error::from_reason(format!("failed to parse package manifests JSON: {error}"))
    })
}

fn parse_engine_input_json(input_json: &str) -> napi::Result<OmenaNapiEngineInputV2> {
    serde_json::from_str(input_json).map_err(|error| {
        napi::Error::from_reason(format!("failed to parse engine input JSON: {error}"))
    })
}

fn to_json_string<T: Serialize>(value: &T) -> napi::Result<String> {
    serde_json::to_string(value).map_err(|error| {
        napi::Error::from_reason(format!("failed to serialize Omena CSS result: {error}"))
    })
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
    fn reports_parser_facts_for_node_source() {
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
    fn builds_css_from_target_query_for_node_clients() {
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
    fn builds_css_from_target_query_options_for_node_clients() {
        let summary = build_style_source_for_target_query_with_options_summary(
            ".card { margin-inline: 1rem; }",
            "fixture.css",
            "ie 11",
            OmenaNapiTargetTransformOptionsV0 {
                allow_logical_to_physical: true,
                allow_scope_flatten: false,
                allow_layer_flatten: false,
                enable_supports_static_eval: false,
                enable_media_static_eval: false,
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
    }

    #[test]
    fn builds_css_from_evaluator_context_for_node_clients() {
        let context = OmenaNapiTransformExecutionContextV0 {
            scss_module_evaluation: Some(omena_query::OmenaQueryTransformModuleEvaluationV0 {
                evaluator: "dart-sass-compatible".to_string(),
                evaluated_css: ".card { color: red; }".to_string(),
            }),
            ..OmenaNapiTransformExecutionContextV0::default()
        };
        let summary = build_style_source_for_target_query_with_context_summary(
            "$brand: red; .card { color: $brand; }",
            "fixture.module.scss",
            "ie 11",
            OmenaNapiTargetTransformOptionsV0 {
                allow_logical_to_physical: false,
                allow_scope_flatten: false,
                allow_layer_flatten: false,
                enable_supports_static_eval: false,
                enable_media_static_eval: false,
                drop_dark_mode_media_queries: false,
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
    fn builds_workspace_sources_for_node_clients() {
        let sources = vec![
            OmenaNapiStyleSourceInputV0 {
                style_path: "Button.module.css".to_string(),
                style_source:
                    r#"@import "./tokens.css"; .button { composes: base; color: var(--brand); } .base { color: blue; }"#
                        .to_string(),
            },
            OmenaNapiStyleSourceInputV0 {
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
            &OmenaNapiTransformExecutionContextV0::default(),
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
    fn serializes_public_json_for_node_clients() -> napi::Result<()> {
        let json = check_style_source_json(".card {}".to_string(), "fixture.css".to_string())
            .map_err(|error| napi::Error::from_reason(format!("{error:?}")))?;

        assert!(json.contains("\"product\":\"omena-query.consumer-check-style-source\""));
        assert!(json.contains("\"classSelectorCount\":1"));
        Ok(())
    }

    #[test]
    fn serializes_expression_domain_reduced_product_projection_for_node_clients() -> napi::Result<()>
    {
        let json = expression_domain_selector_projection_json(
            reduced_product_projection_engine_input_json().to_string(),
        )
        .map_err(|error| napi::Error::from_reason(format!("{error:?}")))?;

        assert!(json.contains("\"product\":\"omena-query.expression-domain-selector-projection\""));
        assert!(json.contains("\"reducedProduct\""));
        assert!(json.contains("\"sourceValueKind\":\"composite\""));
        assert!(json.contains("\"prefix\":\"btn-\""));
        assert!(json.contains("\"suffix\":\"-active\""));
        Ok(())
    }

    #[test]
    fn serializes_transform_context_reachability_sources_for_node_clients() -> napi::Result<()> {
        let json = transform_context_from_engine_input_json(
            reduced_product_projection_engine_input_json().to_string(),
            "/tmp/App.module.scss".to_string(),
            true,
        )
        .map_err(|error| napi::Error::from_reason(format!("{error:?}")))?;

        assert!(json.contains("\"product\":\"omena-query.transform-context-from-engine-input\""));
        assert!(json.contains("\"selectedProjectionCount\":3"));
        assert!(json.contains("\"reachabilitySources\""));
        assert!(json.contains("\"nodeId\":\"file-merge\""));
        assert!(json.contains("\"btn-primary--active\""));
        Ok(())
    }

    #[test]
    fn builds_css_from_engine_input_context_for_node_clients() -> napi::Result<()> {
        let input = parse_engine_input_json(reduced_product_projection_engine_input_json())?;
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
        assert_eq!(summary.semantic_removal_count, 1);
        assert_eq!(summary.execution.semantic_removals.len(), 1);
        assert_eq!(summary.execution.semantic_removals[0].name, "card-active");
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
        Ok(())
    }

    #[test]
    fn builds_css_from_engine_input_style_sources_for_node_clients() -> napi::Result<()> {
        let input = parse_engine_input_json(workspace_style_source_engine_input_json())?;
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
        Ok(())
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
    fn lists_transform_passes_for_node_clients() {
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
                      "end": { "line": 1, "character": 20 }
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
                      "end": { "line": 2, "character": 22 }
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
