//! Browser-side in-memory bindings for the Omena CSS parser and transform surface.

use omena_query::{
    OmenaQueryCascadeAtPositionV0 as OmenaWasmCascadeAtPositionV0,
    OmenaQueryCompletionAtPositionV0 as OmenaWasmCompletionAtPositionV0,
    OmenaQueryConsumerBuildSummaryV0 as OmenaWasmBuildSummaryV0,
    OmenaQueryConsumerCheckSummaryV0 as OmenaWasmCheckSummaryV0,
    OmenaQueryEngineInputV2 as OmenaWasmEngineInputV2, OmenaQueryExpressionDomainFlowRuntimeV0,
    OmenaQueryExpressionDomainIncrementalFlowAnalysisV0 as OmenaWasmExpressionDomainIncrementalFlowAnalysisV0,
    OmenaQueryExpressionDomainSelectorProjectionV0 as OmenaWasmExpressionDomainSelectorProjectionV0,
    OmenaQuerySourceDiagnosticsForFileV0 as OmenaWasmSourceDiagnosticsForFileV0,
    OmenaQuerySourceMissingSelectorDiagnosticCandidateV0 as OmenaWasmSourceMissingSelectorDiagnosticCandidateV0,
    OmenaQueryStyleContextIndexV0 as OmenaWasmStyleContextIndexV0,
    OmenaQueryStyleDiagnosticsForFileV0 as OmenaWasmStyleDiagnosticsForFileV0,
    OmenaQueryStyleHoverCandidatesV0 as OmenaWasmStyleHoverCandidatesV0,
    OmenaQueryStylePackageManifestV0 as OmenaWasmStylePackageManifestV0,
    OmenaQueryStyleSourceInputV0 as OmenaWasmStyleSourceInputV0,
    OmenaQueryTargetTransformOptionsV0 as OmenaWasmTargetTransformOptionsV0,
    OmenaQueryTransformContextFromEngineInputSummaryV0 as OmenaWasmTransformContextFromEngineInputSummaryV0,
    OmenaQueryTransformExecutionContextV0 as OmenaWasmTransformExecutionContextV0,
    OmenaQueryTransformPassSummaryV0 as OmenaWasmPassSummaryV0, ParserPositionV0,
    conservative_omena_query_target_options, execute_omena_query_consumer_build_style_source,
    execute_omena_query_consumer_build_style_source_for_target_query,
    execute_omena_query_consumer_build_style_source_for_target_query_with_context_and_options,
    execute_omena_query_consumer_build_style_source_for_target_query_with_options,
    execute_omena_query_consumer_build_style_source_with_context,
    execute_omena_query_consumer_build_style_source_with_engine_input_context,
    execute_omena_query_consumer_build_style_sources_for_target_query_with_context_and_options,
    execute_omena_query_consumer_build_style_sources_with_context,
    list_omena_query_transform_pass_summaries, read_omena_query_cascade_at_position,
    read_omena_query_style_context_index, summarize_omena_query_consumer_check_style_source,
    summarize_omena_query_expression_domain_incremental_flow_analysis,
    summarize_omena_query_expression_domain_selector_projection,
    summarize_omena_query_source_diagnostics_for_file,
    summarize_omena_query_style_completion_at_position,
    summarize_omena_query_style_diagnostics_for_file, summarize_omena_query_style_hover_candidates,
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

#[wasm_bindgen(js_name = expressionDomainIncrementalFlow)]
pub fn expression_domain_incremental_flow(input: JsValue) -> Result<JsValue, JsValue> {
    let input = parse_engine_input_value(input)?;
    let mut runtime = OmenaQueryExpressionDomainFlowRuntimeV0::default();
    to_js_value(&expression_domain_incremental_flow_analysis_summary(
        &input,
        &mut runtime,
    ))
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

#[wasm_bindgen(js_name = readCascadeAtPosition)]
pub fn read_cascade_at_position(
    source: &str,
    path: &str,
    line: u32,
    character: u32,
    input: JsValue,
) -> Result<JsValue, JsValue> {
    let input = parse_optional_engine_input_value(input)?;
    to_js_value(&read_cascade_at_position_summary(
        source,
        path,
        line as usize,
        character as usize,
        &input,
    ))
}

#[wasm_bindgen(js_name = readStyleContextIndex)]
pub fn read_style_context_index(
    source: &str,
    path: &str,
    input: JsValue,
) -> Result<JsValue, JsValue> {
    let input = parse_optional_engine_input_value(input)?;
    to_js_value(&read_style_context_index_summary(source, path, &input))
}

#[wasm_bindgen(js_name = readStyleDiagnostics)]
pub fn read_style_diagnostics(source: &str, path: &str) -> Result<JsValue, JsValue> {
    to_js_value(&read_style_diagnostics_summary(source, path)?)
}

#[wasm_bindgen(js_name = readStyleHoverCandidates)]
pub fn read_style_hover_candidates(source: &str, path: &str) -> Result<JsValue, JsValue> {
    to_js_value(&read_style_hover_candidates_summary(source, path)?)
}

#[wasm_bindgen(js_name = readStyleCompletionAtPosition)]
pub fn read_style_completion_at_position(
    source: &str,
    path: &str,
    line: u32,
    character: u32,
) -> Result<JsValue, JsValue> {
    to_js_value(&read_style_completion_at_position_summary(
        source,
        path,
        line as usize,
        character as usize,
    )?)
}

#[wasm_bindgen(js_name = readSourceDiagnostics)]
pub fn read_source_diagnostics(source_uri: &str, candidates: JsValue) -> Result<JsValue, JsValue> {
    let candidates = parse_source_diagnostic_candidates_value(candidates)?;
    to_js_value(&read_source_diagnostics_summary(
        source_uri,
        candidates.as_slice(),
    ))
}

#[wasm_bindgen(js_name = ExpressionDomainFlowRuntime)]
pub struct OmenaWasmExpressionDomainFlowRuntimeV0 {
    inner: OmenaQueryExpressionDomainFlowRuntimeV0,
}

#[wasm_bindgen(js_class = ExpressionDomainFlowRuntime)]
impl OmenaWasmExpressionDomainFlowRuntimeV0 {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: OmenaQueryExpressionDomainFlowRuntimeV0::default(),
        }
    }

    #[wasm_bindgen(js_name = analyze)]
    pub fn analyze(&mut self, input: JsValue) -> Result<JsValue, JsValue> {
        let input = parse_engine_input_value(input)?;
        to_js_value(&self.analyze_summary(&input))
    }
}

impl OmenaWasmExpressionDomainFlowRuntimeV0 {
    pub fn analyze_summary(
        &mut self,
        input: &OmenaWasmEngineInputV2,
    ) -> OmenaWasmExpressionDomainIncrementalFlowAnalysisV0 {
        expression_domain_incremental_flow_analysis_summary(input, &mut self.inner)
    }
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

pub fn expression_domain_incremental_flow_analysis_summary(
    input: &OmenaWasmEngineInputV2,
    runtime: &mut OmenaQueryExpressionDomainFlowRuntimeV0,
) -> OmenaWasmExpressionDomainIncrementalFlowAnalysisV0 {
    summarize_omena_query_expression_domain_incremental_flow_analysis(input, runtime)
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

pub fn read_cascade_at_position_summary(
    source: &str,
    path: &str,
    line: usize,
    character: usize,
    input: &OmenaWasmEngineInputV2,
) -> Option<OmenaWasmCascadeAtPositionV0> {
    let path = effective_path(path);
    read_omena_query_cascade_at_position(path, source, input, ParserPositionV0 { line, character })
}

pub fn read_style_context_index_summary(
    source: &str,
    path: &str,
    input: &OmenaWasmEngineInputV2,
) -> Option<OmenaWasmStyleContextIndexV0> {
    let path = effective_path(path);
    read_omena_query_style_context_index(path, source, input)
}

pub fn read_style_diagnostics_summary(
    source: &str,
    path: &str,
) -> Result<OmenaWasmStyleDiagnosticsForFileV0, JsValue> {
    let path = effective_path(path);
    let candidates = summarize_omena_query_style_hover_candidates(path, source)
        .ok_or_else(|| JsValue::from_str(&format!("failed to read style candidates for {path}")))?;
    Ok(summarize_omena_query_style_diagnostics_for_file(
        path,
        source,
        candidates.candidates.as_slice(),
    ))
}

pub fn read_style_hover_candidates_summary(
    source: &str,
    path: &str,
) -> Result<OmenaWasmStyleHoverCandidatesV0, JsValue> {
    let path = effective_path(path);
    summarize_omena_query_style_hover_candidates(path, source)
        .ok_or_else(|| JsValue::from_str(&format!("failed to read style candidates for {path}")))
}

pub fn read_style_completion_at_position_summary(
    source: &str,
    path: &str,
    line: usize,
    character: usize,
) -> Result<OmenaWasmCompletionAtPositionV0, JsValue> {
    let path = effective_path(path);
    let candidates = read_style_hover_candidates_summary(source, path)?;
    Ok(summarize_omena_query_style_completion_at_position(
        path,
        source,
        ParserPositionV0 { line, character },
        candidates.candidates.as_slice(),
    ))
}

pub fn read_source_diagnostics_summary(
    source_uri: &str,
    candidates: &[OmenaWasmSourceMissingSelectorDiagnosticCandidateV0],
) -> OmenaWasmSourceDiagnosticsForFileV0 {
    summarize_omena_query_source_diagnostics_for_file(source_uri, candidates)
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

fn parse_source_diagnostic_candidates_value(
    value: JsValue,
) -> Result<Vec<OmenaWasmSourceMissingSelectorDiagnosticCandidateV0>, JsValue> {
    serde_wasm_bindgen::from_value(value).map_err(|error| {
        JsValue::from_str(&format!(
            "failed to parse source diagnostic candidates value: {error}"
        ))
    })
}

fn parse_optional_engine_input_value(value: JsValue) -> Result<OmenaWasmEngineInputV2, JsValue> {
    if value.is_null() || value.is_undefined() {
        return Ok(empty_engine_input());
    }
    parse_engine_input_value(value)
}

fn empty_engine_input() -> OmenaWasmEngineInputV2 {
    OmenaWasmEngineInputV2 {
        version: "2".to_string(),
        sources: Vec::new(),
        styles: Vec::new(),
        type_facts: Vec::new(),
    }
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
    fn reads_cascade_lfp_for_browser_clients() {
        let input = empty_engine_input();
        let summary = read_cascade_at_position_summary(
            ":root { --known: #2563eb; }\n.button { color: var(--known); }\n",
            "fixture.module.css",
            1,
            24,
            &input,
        )
        .expect("cascade summary should be available");

        assert_eq!(summary.product, "omena-query.read-cascade-at-position");
        assert_eq!(summary.status, "resolved");
        assert_eq!(summary.reference_name.as_deref(), Some("--known"));
        assert_eq!(
            summary.referenced_declaration_computed_value.as_deref(),
            Some("#2563eb")
        );
        assert_eq!(
            summary.reference_custom_property_fixed_point_status,
            Some("fixedPointStable")
        );
        assert_eq!(
            summary
                .reference_custom_property_fixed_point_value
                .as_deref(),
            Some("#2563eb")
        );
    }

    #[test]
    fn reads_style_context_index_for_browser_clients() {
        let input = empty_engine_input();
        let summary = read_style_context_index_summary(
            "@layer components { @container card (min-width: 20rem) { .card { color: red; } } }",
            "fixture.module.css",
            &input,
        )
        .expect("context index summary should be available");

        assert_eq!(summary.product, "omena-query.style-context-index");
        assert_eq!(summary.context_index.layer_index.block_layers.len(), 1);
        assert_eq!(
            summary.context_index.container_index.named_container_count,
            1
        );
    }

    #[test]
    fn reads_style_diagnostics_for_browser_clients() {
        let summary = read_style_diagnostics_summary(
            ":root { --known: #2563eb; }\n.button { color: var(--missing); animation: fade 1s; }\n",
            "fixture.module.css",
        )
        .expect("style diagnostics should be available");

        assert_eq!(summary.product, "omena-query.diagnostics-for-file");
        assert_eq!(summary.file_kind, "style");
        assert!(
            summary
                .ready_surfaces
                .contains(&"missingCustomPropertyDiagnostics")
        );
        assert!(
            summary
                .ready_surfaces
                .contains(&"missingKeyframesDiagnostics")
        );
        assert!(
            summary
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "missingCustomProperty")
        );
        assert!(
            summary
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "missingKeyframes")
        );
    }

    #[test]
    fn reads_style_hover_and_completion_for_browser_clients() {
        let source = ":root { --brand: #2563eb; }\n.button { color: var(--); }\n";
        let hover = read_style_hover_candidates_summary(source, "fixture.module.css")
            .expect("style hover candidates should be available");

        assert_eq!(hover.product, "omena-query.style-hover-candidates");
        assert!(
            hover
                .candidates
                .iter()
                .any(|candidate| candidate.name == "--brand")
        );

        let completion =
            read_style_completion_at_position_summary(source, "fixture.module.css", 1, 23)
                .expect("style completion should be available");

        assert_eq!(completion.product, "omena-query.completion-at");
        assert!(completion.items.iter().any(|item| item.label == "--brand"));
    }

    #[test]
    fn reads_source_diagnostics_for_browser_clients() {
        let candidates = serde_json::from_str::<
            Vec<OmenaWasmSourceMissingSelectorDiagnosticCandidateV0>,
        >(source_diagnostic_candidates_json());
        assert!(candidates.is_ok());
        let Ok(candidates) = candidates else {
            return;
        };
        let summary =
            read_source_diagnostics_summary("file:///workspace/src/App.tsx", candidates.as_slice());

        assert_eq!(summary.product, "omena-query.diagnostics-for-file");
        assert_eq!(summary.file_kind, "source");
        assert_eq!(summary.diagnostic_count, 1);
        assert_eq!(summary.diagnostics[0].code, "missingSelector");
        assert!(summary.ready_surfaces.contains(&"crossLanguageDiagnostics"));
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
    fn reuses_expression_domain_flow_runtime_for_browser_clients() {
        let input = serde_json::from_str::<OmenaWasmEngineInputV2>(
            reduced_product_projection_engine_input_json(),
        );
        assert!(input.is_ok());
        let Ok(input) = input else {
            return;
        };
        let mut runtime = OmenaWasmExpressionDomainFlowRuntimeV0::new();

        let first = runtime.analyze_summary(&input);
        assert_eq!(
            first.product,
            "omena-query.expression-domain-incremental-flow-analysis"
        );
        assert_eq!(first.revision, 1);
        assert_eq!(first.reused_graph_count, 0);

        let second = runtime.analyze_summary(&input);
        assert_eq!(second.revision, 2);
        assert_eq!(second.dirty_graph_count, 0);
        assert_eq!(second.reused_graph_count, 1);
        assert!(
            second
                .analyses
                .iter()
                .all(|entry| entry.analysis.reused_previous_analysis)
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

    fn source_diagnostic_candidates_json() -> &'static str {
        r#"[
          {
            "targetStyleUri": "file:///workspace/src/App.module.css",
            "targetStyleSource": ".root {\n}\n",
            "selectorName": "missing",
            "sourceReferenceRange": {
              "start": { "line": 2, "character": 18 },
              "end": { "line": 2, "character": 25 }
            }
          }
        ]"#
    }
}
