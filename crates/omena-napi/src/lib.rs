//! Node native bindings for the Omena CSS parser and transform surface.

use napi_derive::napi;
use omena_query::{
    OmenaQueryConsumerBuildSummaryV0 as OmenaNapiBuildSummaryV0,
    OmenaQueryConsumerCheckSummaryV0 as OmenaNapiCheckSummaryV0,
    OmenaQueryTransformPassSummaryV0 as OmenaNapiPassSummaryV0,
    execute_omena_query_consumer_build_style_source, list_omena_query_transform_pass_summaries,
    summarize_omena_query_consumer_check_style_source,
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

pub fn list_transform_pass_summaries() -> Vec<OmenaNapiPassSummaryV0> {
    list_omena_query_transform_pass_summaries()
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
