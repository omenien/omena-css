//! Node native bindings for the Omena CSS parser and transform surface.

use napi_derive::napi;
use omena_query::{
    OmenaQueryConsumerBuildSummaryV0 as OmenaNapiBuildSummaryV0,
    OmenaQueryConsumerCheckSummaryV0 as OmenaNapiCheckSummaryV0,
    OmenaQueryTargetTransformOptionsV0 as OmenaNapiTargetTransformOptionsV0,
    OmenaQueryTransformExecutionContextV0 as OmenaNapiTransformExecutionContextV0,
    OmenaQueryTransformPassSummaryV0 as OmenaNapiPassSummaryV0,
    execute_omena_query_consumer_build_style_source,
    execute_omena_query_consumer_build_style_source_for_target_query,
    execute_omena_query_consumer_build_style_source_for_target_query_with_context_and_options,
    execute_omena_query_consumer_build_style_source_for_target_query_with_options,
    execute_omena_query_consumer_build_style_source_with_context,
    list_omena_query_transform_pass_summaries, summarize_omena_query_consumer_check_style_source,
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

#[napi(js_name = "listTransformPassesJson")]
pub fn list_transform_passes_json() -> napi::Result<String> {
    to_json_string(&list_transform_pass_summaries())
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

pub fn list_transform_pass_summaries() -> Vec<OmenaNapiPassSummaryV0> {
    list_omena_query_transform_pass_summaries()
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
    fn serializes_public_json_for_node_clients() -> napi::Result<()> {
        let json = check_style_source_json(".card {}".to_string(), "fixture.css".to_string())
            .map_err(|error| napi::Error::from_reason(format!("{error:?}")))?;

        assert!(json.contains("\"product\":\"omena-query.consumer-check-style-source\""));
        assert!(json.contains("\"classSelectorCount\":1"));
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
}
