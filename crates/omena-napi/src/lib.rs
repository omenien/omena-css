//! Node native bindings for the Omena CSS parser and transform surface.

use napi_derive::napi;
use omena_parser::{StyleDialect, dialect_for_path, parse, summarize_omena_parser_style_facts};
use omena_transform_cst::{TransformPassKind, all_transform_pass_kinds};
use omena_transform_passes::{
    TransformExecutionSummaryV0, execute_transform_passes_on_source_with_dialect,
};
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaNapiCheckSummaryV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub path: String,
    pub dialect: &'static str,
    pub token_count: usize,
    pub parser_error_count: usize,
    pub class_selector_count: usize,
    pub custom_property_count: usize,
    pub keyframe_count: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaNapiBuildSummaryV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub path: String,
    pub dialect: &'static str,
    pub requested_pass_ids: Vec<String>,
    pub unknown_pass_ids: Vec<String>,
    pub execution: TransformExecutionSummaryV0,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaNapiPassSummaryV0 {
    pub id: &'static str,
    pub title: &'static str,
    pub reads_semantic_graph: bool,
    pub reads_cascade_model: bool,
}

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
    let dialect = dialect_for_path(path);
    let parse_result = parse(source, dialect);
    let style_facts = summarize_omena_parser_style_facts(source, dialect);

    OmenaNapiCheckSummaryV0 {
        schema_version: "0",
        product: "omena-napi.check-style-source",
        path: path.to_string(),
        dialect: dialect_label(dialect),
        token_count: parse_result.token_count(),
        parser_error_count: parse_result.errors().len(),
        class_selector_count: style_facts.class_selector_names.len(),
        custom_property_count: style_facts.custom_property_names.len(),
        keyframe_count: style_facts.keyframe_names.len(),
    }
}

pub fn build_style_source_summary(
    source: &str,
    path: &str,
    pass_ids: &[String],
) -> OmenaNapiBuildSummaryV0 {
    let path = effective_path(path);
    let dialect = dialect_for_path(path);
    let mut requested_passes = Vec::new();
    let mut unknown_pass_ids = Vec::new();

    if pass_ids.is_empty() {
        requested_passes.extend(all_transform_pass_kinds());
    } else {
        for pass_id in pass_ids {
            match transform_pass_kind_from_id(pass_id) {
                Some(pass) => requested_passes.push(pass),
                None => unknown_pass_ids.push(pass_id.clone()),
            }
        }
    }

    let execution =
        execute_transform_passes_on_source_with_dialect(source, dialect, &requested_passes);

    OmenaNapiBuildSummaryV0 {
        schema_version: "0",
        product: "omena-napi.build-style-source",
        path: path.to_string(),
        dialect: dialect_label(dialect),
        requested_pass_ids: pass_ids.to_vec(),
        unknown_pass_ids,
        execution,
    }
}

pub fn list_transform_pass_summaries() -> Vec<OmenaNapiPassSummaryV0> {
    all_transform_pass_kinds()
        .into_iter()
        .map(|kind| OmenaNapiPassSummaryV0 {
            id: kind.id(),
            title: kind.title(),
            reads_semantic_graph: kind.reads_semantic_graph(),
            reads_cascade_model: kind.reads_cascade_model(),
        })
        .collect()
}

fn to_json_string<T: Serialize>(value: &T) -> napi::Result<String> {
    serde_json::to_string(value).map_err(|error| {
        napi::Error::from_reason(format!("failed to serialize Omena CSS result: {error}"))
    })
}

fn transform_pass_kind_from_id(pass_id: &str) -> Option<TransformPassKind> {
    all_transform_pass_kinds()
        .into_iter()
        .find(|kind| kind.id() == pass_id)
}

fn effective_path(path: &str) -> &str {
    if path.trim().is_empty() {
        "style.css"
    } else {
        path
    }
}

fn dialect_label(dialect: StyleDialect) -> &'static str {
    match dialect {
        StyleDialect::Css => "css",
        StyleDialect::Scss => "scss",
        StyleDialect::Sass => "sass",
        StyleDialect::Less => "less",
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

        assert_eq!(summary.product, "omena-napi.check-style-source");
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

        assert_eq!(summary.product, "omena-napi.build-style-source");
        assert!(summary.unknown_pass_ids.is_empty());
        assert!(summary.execution.output_css.contains("#fff"));
    }

    #[test]
    fn serializes_public_json_for_node_clients() -> napi::Result<()> {
        let json = check_style_source_json(".card {}".to_string(), "fixture.css".to_string())
            .map_err(|error| napi::Error::from_reason(format!("{error:?}")))?;

        assert!(json.contains("\"product\":\"omena-napi.check-style-source\""));
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
