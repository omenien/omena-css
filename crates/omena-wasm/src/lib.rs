//! Browser-side in-memory bindings for the Omena CSS parser and transform surface.

use omena_parser::{StyleDialect, dialect_for_path, parse, summarize_omena_parser_style_facts};
use omena_transform_cst::{TransformPassKind, all_transform_pass_kinds};
use omena_transform_passes::{
    TransformExecutionSummaryV0, execute_transform_passes_on_source_with_dialect,
};
use serde::Serialize;
use wasm_bindgen::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaWasmCheckSummaryV0 {
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
pub struct OmenaWasmBuildSummaryV0 {
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
pub struct OmenaWasmPassSummaryV0 {
    pub id: &'static str,
    pub title: &'static str,
    pub reads_semantic_graph: bool,
    pub reads_cascade_model: bool,
}

#[wasm_bindgen(js_name = checkStyleSource)]
pub fn check_style_source(source: &str, path: &str) -> Result<JsValue, JsValue> {
    to_js_value(&check_style_source_summary(source, path))
}

#[wasm_bindgen(js_name = buildStyleSource)]
pub fn build_style_source(source: &str, path: &str, pass_ids: JsValue) -> Result<JsValue, JsValue> {
    let pass_ids = parse_pass_ids_value(pass_ids)?;
    to_js_value(&build_style_source_summary(source, path, &pass_ids))
}

#[wasm_bindgen(js_name = listTransformPasses)]
pub fn list_transform_passes() -> Result<JsValue, JsValue> {
    to_js_value(&list_transform_pass_summaries())
}

pub fn check_style_source_summary(source: &str, path: &str) -> OmenaWasmCheckSummaryV0 {
    let path = effective_path(path);
    let dialect = dialect_for_path(path);
    let parse_result = parse(source, dialect);
    let style_facts = summarize_omena_parser_style_facts(source, dialect);

    OmenaWasmCheckSummaryV0 {
        schema_version: "0",
        product: "omena-wasm.check-style-source",
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
) -> OmenaWasmBuildSummaryV0 {
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

    OmenaWasmBuildSummaryV0 {
        schema_version: "0",
        product: "omena-wasm.build-style-source",
        path: path.to_string(),
        dialect: dialect_label(dialect),
        requested_pass_ids: pass_ids.to_vec(),
        unknown_pass_ids,
        execution,
    }
}

pub fn list_transform_pass_summaries() -> Vec<OmenaWasmPassSummaryV0> {
    all_transform_pass_kinds()
        .into_iter()
        .map(|kind| OmenaWasmPassSummaryV0 {
            id: kind.id(),
            title: kind.title(),
            reads_semantic_graph: kind.reads_semantic_graph(),
            reads_cascade_model: kind.reads_cascade_model(),
        })
        .collect()
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

fn to_js_value<T: Serialize>(value: &T) -> Result<JsValue, JsValue> {
    serde_wasm_bindgen::to_value(value)
        .map_err(|error| JsValue::from_str(&format!("failed to serialize result: {error}")))
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
    fn reports_parser_facts_for_browser_source() {
        let summary = check_style_source_summary(
            ".card { color: red; }\n:root { --brand: blue; }",
            "fixture.module.css",
        );

        assert_eq!(summary.product, "omena-wasm.check-style-source");
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

        assert_eq!(summary.product, "omena-wasm.build-style-source");
        assert!(summary.unknown_pass_ids.is_empty());
        assert!(summary.execution.output_css.contains("#fff"));
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
}
