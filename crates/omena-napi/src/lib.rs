//! Node native bindings for the Omena CSS parser and transform surface.

use napi_derive::napi;
use omena_query::{
    OmenaQueryCascadeAtPositionV0 as OmenaNapiCascadeAtPositionV0,
    OmenaQueryCompletionAtPositionV0 as OmenaNapiCompletionAtPositionV0,
    OmenaQueryConsumerBuildSummaryV0 as OmenaNapiBuildSummaryV0,
    OmenaQueryConsumerCheckSummaryV0 as OmenaNapiCheckSummaryV0,
    OmenaQueryEngineInputV2 as OmenaNapiEngineInputV2, OmenaQueryExpressionDomainFlowRuntimeV0,
    OmenaQueryExpressionDomainIncrementalFlowAnalysisV0 as OmenaNapiExpressionDomainIncrementalFlowAnalysisV0,
    OmenaQueryExpressionDomainSelectorProjectionV0 as OmenaNapiExpressionDomainSelectorProjectionV0,
    OmenaQuerySourceDiagnosticsForFileV0 as OmenaNapiSourceDiagnosticsForFileV0,
    OmenaQuerySourceMissingSelectorDiagnosticCandidateV0 as OmenaNapiSourceMissingSelectorDiagnosticCandidateV0,
    OmenaQueryStyleContextIndexV0 as OmenaNapiStyleContextIndexV0,
    OmenaQueryStyleDiagnosticsForFileV0 as OmenaNapiStyleDiagnosticsForFileV0,
    OmenaQueryStyleHoverCandidatesV0 as OmenaNapiStyleHoverCandidatesV0,
    OmenaQueryStylePackageManifestV0 as OmenaNapiStylePackageManifestV0,
    OmenaQueryStyleSourceInputV0 as OmenaNapiStyleSourceInputV0,
    OmenaQueryTargetTransformOptionsV0 as OmenaNapiTargetTransformOptionsV0,
    OmenaQueryTransformContextFromEngineInputSummaryV0 as OmenaNapiTransformContextFromEngineInputSummaryV0,
    OmenaQueryTransformExecutionContextV0 as OmenaNapiTransformExecutionContextV0,
    OmenaQueryTransformPassSummaryV0 as OmenaNapiPassSummaryV0, ParserPositionV0,
    execute_omena_query_consumer_build_style_source,
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
    summarize_omena_query_style_diagnostics_for_file,
    summarize_omena_query_style_diagnostics_for_workspace_file,
    summarize_omena_query_style_hover_candidates,
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

#[napi(js_name = "expressionDomainIncrementalFlowJson")]
pub fn expression_domain_incremental_flow_json(input_json: String) -> napi::Result<String> {
    let input = parse_engine_input_json(&input_json)?;
    let mut runtime = OmenaQueryExpressionDomainFlowRuntimeV0::default();
    to_json_string(&expression_domain_incremental_flow_analysis_summary(
        &input,
        &mut runtime,
    ))
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

#[napi(js_name = "readCascadeAtPositionJson")]
pub fn read_cascade_at_position_json(
    source: String,
    path: String,
    line: u32,
    character: u32,
    input_json: String,
) -> napi::Result<String> {
    let input = parse_optional_engine_input_json(&input_json)?;
    to_json_string(&read_cascade_at_position_summary(
        &source,
        &path,
        line as usize,
        character as usize,
        &input,
    ))
}

#[napi(js_name = "readStyleContextIndexJson")]
pub fn read_style_context_index_json(
    source: String,
    path: String,
    input_json: String,
) -> napi::Result<String> {
    let input = parse_optional_engine_input_json(&input_json)?;
    to_json_string(&read_style_context_index_summary(&source, &path, &input))
}

#[napi(js_name = "readStyleDiagnosticsJson")]
pub fn read_style_diagnostics_json(source: String, path: String) -> napi::Result<String> {
    to_json_string(&read_style_diagnostics_summary(&source, &path)?)
}

#[napi(js_name = "readWorkspaceStyleDiagnosticsJson")]
pub fn read_workspace_style_diagnostics_json(
    target_path: String,
    sources_json: String,
    package_manifests_json: String,
) -> napi::Result<String> {
    let sources = parse_style_sources_json(&sources_json)?;
    let package_manifests = parse_package_manifests_json(&package_manifests_json)?;
    to_json_string(&read_workspace_style_diagnostics_summary(
        &target_path,
        &sources,
        &package_manifests,
    )?)
}

#[napi(js_name = "readStyleHoverCandidatesJson")]
pub fn read_style_hover_candidates_json(source: String, path: String) -> napi::Result<String> {
    to_json_string(&read_style_hover_candidates_summary(&source, &path)?)
}

#[napi(js_name = "readStyleCompletionAtPositionJson")]
pub fn read_style_completion_at_position_json(
    source: String,
    path: String,
    line: u32,
    character: u32,
) -> napi::Result<String> {
    to_json_string(&read_style_completion_at_position_summary(
        &source,
        &path,
        line as usize,
        character as usize,
    )?)
}

#[napi(js_name = "readSourceDiagnosticsJson")]
pub fn read_source_diagnostics_json(
    source_uri: String,
    candidates_json: String,
) -> napi::Result<String> {
    let candidates = parse_source_diagnostic_candidates_json(&candidates_json)?;
    to_json_string(&read_source_diagnostics_summary(
        &source_uri,
        candidates.as_slice(),
    ))
}

#[napi(js_name = "ExpressionDomainFlowRuntime")]
pub struct OmenaNapiExpressionDomainFlowRuntimeV0 {
    inner: OmenaQueryExpressionDomainFlowRuntimeV0,
}

#[napi]
impl OmenaNapiExpressionDomainFlowRuntimeV0 {
    #[napi(constructor)]
    pub fn new() -> Self {
        Self {
            inner: OmenaQueryExpressionDomainFlowRuntimeV0::default(),
        }
    }

    #[napi(js_name = "analyzeJson")]
    pub fn analyze_json(&mut self, input_json: String) -> napi::Result<String> {
        let input = parse_engine_input_json(&input_json)?;
        to_json_string(&expression_domain_incremental_flow_analysis_summary(
            &input,
            &mut self.inner,
        ))
    }
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

pub fn expression_domain_incremental_flow_analysis_summary(
    input: &OmenaNapiEngineInputV2,
    runtime: &mut OmenaQueryExpressionDomainFlowRuntimeV0,
) -> OmenaNapiExpressionDomainIncrementalFlowAnalysisV0 {
    summarize_omena_query_expression_domain_incremental_flow_analysis(input, runtime)
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

pub fn read_cascade_at_position_summary(
    source: &str,
    path: &str,
    line: usize,
    character: usize,
    input: &OmenaNapiEngineInputV2,
) -> Option<OmenaNapiCascadeAtPositionV0> {
    let path = effective_path(path);
    read_omena_query_cascade_at_position(path, source, input, ParserPositionV0 { line, character })
}

pub fn read_style_context_index_summary(
    source: &str,
    path: &str,
    input: &OmenaNapiEngineInputV2,
) -> Option<OmenaNapiStyleContextIndexV0> {
    let path = effective_path(path);
    read_omena_query_style_context_index(path, source, input)
}

pub fn read_style_diagnostics_summary(
    source: &str,
    path: &str,
) -> napi::Result<OmenaNapiStyleDiagnosticsForFileV0> {
    let path = effective_path(path);
    let candidates =
        summarize_omena_query_style_hover_candidates(path, source).ok_or_else(|| {
            napi::Error::from_reason(format!("failed to read style candidates for {path}"))
        })?;
    Ok(summarize_omena_query_style_diagnostics_for_file(
        path,
        source,
        candidates.candidates.as_slice(),
    ))
}

pub fn read_workspace_style_diagnostics_summary(
    target_path: &str,
    sources: &[OmenaNapiStyleSourceInputV0],
    package_manifests: &[OmenaNapiStylePackageManifestV0],
) -> napi::Result<OmenaNapiStyleDiagnosticsForFileV0> {
    let target_path = effective_path(target_path);
    summarize_omena_query_style_diagnostics_for_workspace_file(
        target_path,
        sources,
        package_manifests,
    )
    .ok_or_else(|| {
        napi::Error::from_reason(format!(
            "failed to read workspace style diagnostics for {target_path}"
        ))
    })
}

pub fn read_style_hover_candidates_summary(
    source: &str,
    path: &str,
) -> napi::Result<OmenaNapiStyleHoverCandidatesV0> {
    let path = effective_path(path);
    summarize_omena_query_style_hover_candidates(path, source).ok_or_else(|| {
        napi::Error::from_reason(format!("failed to read style candidates for {path}"))
    })
}

pub fn read_style_completion_at_position_summary(
    source: &str,
    path: &str,
    line: usize,
    character: usize,
) -> napi::Result<OmenaNapiCompletionAtPositionV0> {
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
    candidates: &[OmenaNapiSourceMissingSelectorDiagnosticCandidateV0],
) -> OmenaNapiSourceDiagnosticsForFileV0 {
    summarize_omena_query_source_diagnostics_for_file(source_uri, candidates)
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

fn parse_source_diagnostic_candidates_json(
    candidates_json: &str,
) -> napi::Result<Vec<OmenaNapiSourceMissingSelectorDiagnosticCandidateV0>> {
    serde_json::from_str(candidates_json).map_err(|error| {
        napi::Error::from_reason(format!(
            "failed to parse source diagnostic candidates JSON: {error}"
        ))
    })
}

fn parse_optional_engine_input_json(input_json: &str) -> napi::Result<OmenaNapiEngineInputV2> {
    if input_json.trim().is_empty() {
        return Ok(empty_engine_input());
    }
    parse_engine_input_json(input_json)
}

fn empty_engine_input() -> OmenaNapiEngineInputV2 {
    OmenaNapiEngineInputV2 {
        version: "2".to_string(),
        sources: Vec::new(),
        styles: Vec::new(),
        type_facts: Vec::new(),
    }
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
    fn reads_cascade_lfp_for_node_clients() {
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
    fn reads_style_context_index_for_node_clients() {
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
    fn reads_style_diagnostics_for_node_clients() {
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
                .ready_surfaces
                .contains(&"missingSassSymbolDiagnostics")
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
    fn reads_workspace_style_diagnostics_for_node_clients() {
        let sources = vec![
            OmenaNapiStyleSourceInputV0 {
                style_path: "/workspace/src/App.module.css".to_string(),
                style_source: r#".button { composes: missing from "./Base.module.css"; }
@value absent from "./Tokens.module.css";"#
                    .to_string(),
            },
            OmenaNapiStyleSourceInputV0 {
                style_path: "/workspace/src/Base.module.css".to_string(),
                style_source: ".base { color: blue; }".to_string(),
            },
            OmenaNapiStyleSourceInputV0 {
                style_path: "/workspace/src/Tokens.module.css".to_string(),
                style_source: "@value accent: blue;".to_string(),
            },
        ];
        let summary = read_workspace_style_diagnostics_summary(
            "/workspace/src/App.module.css",
            &sources,
            &[],
        )
        .expect("workspace diagnostics should be available");

        assert!(
            summary
                .ready_surfaces
                .contains(&"cssModulesComposesResolutionDiagnostics")
        );
        assert!(
            summary
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "missingComposedSelector")
        );
        assert!(
            summary
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "missingImportedValue")
        );
    }

    #[test]
    fn reads_style_hover_and_completion_for_node_clients() {
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
    fn reads_source_diagnostics_for_node_clients() {
        let candidates =
            parse_source_diagnostic_candidates_json(source_diagnostic_candidates_json())
                .expect("source diagnostic candidates should parse");
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
    fn serializes_cascade_lfp_for_node_clients() -> napi::Result<()> {
        let json = read_cascade_at_position_json(
            ":root { --known: #2563eb; }\n.button { color: var(--known); }\n".to_string(),
            "fixture.module.css".to_string(),
            1,
            24,
            String::new(),
        )
        .map_err(|error| napi::Error::from_reason(format!("{error:?}")))?;

        assert!(json.contains("\"product\":\"omena-query.read-cascade-at-position\""));
        assert!(json.contains("\"referenceName\":\"--known\""));
        assert!(json.contains("\"referenceCustomPropertyFixedPointValue\":\"#2563eb\""));
        Ok(())
    }

    #[test]
    fn serializes_style_context_index_for_node_clients() -> napi::Result<()> {
        let json = read_style_context_index_json(
            "@layer components { @container card (min-width: 20rem) { .card { color: red; } } }"
                .to_string(),
            "fixture.module.css".to_string(),
            String::new(),
        )
        .map_err(|error| napi::Error::from_reason(format!("{error:?}")))?;

        assert!(json.contains("\"product\":\"omena-query.style-context-index\""));
        assert!(json.contains("\"contextIndexSource\":\"omena-semantic.style-context-index\""));
        assert!(json.contains("\"namedContainerCount\":1"));
        Ok(())
    }

    #[test]
    fn serializes_style_diagnostics_for_node_clients() -> napi::Result<()> {
        let json = read_style_diagnostics_json(
            ":root { --known: #2563eb; }\n.button { color: var(--missing); }\n".to_string(),
            "fixture.module.css".to_string(),
        )
        .map_err(|error| napi::Error::from_reason(format!("{error:?}")))?;

        assert!(json.contains("\"product\":\"omena-query.diagnostics-for-file\""));
        assert!(json.contains("\"fileKind\":\"style\""));
        assert!(json.contains("\"code\":\"missingCustomProperty\""));
        Ok(())
    }

    #[test]
    fn serializes_workspace_style_diagnostics_for_node_clients() -> napi::Result<()> {
        let sources = r#"[
{"stylePath":"/workspace/src/App.module.css","styleSource":".button { composes: missing from \"./Base.module.css\"; }"},
{"stylePath":"/workspace/src/Base.module.css","styleSource":".base { color: blue; }"}
]"#;
        let json = read_workspace_style_diagnostics_json(
            "/workspace/src/App.module.css".to_string(),
            sources.to_string(),
            String::new(),
        )
        .map_err(|error| napi::Error::from_reason(format!("{error:?}")))?;

        assert!(json.contains("\"product\":\"omena-query.diagnostics-for-file\""));
        assert!(json.contains("\"code\":\"missingComposedSelector\""));
        Ok(())
    }

    #[test]
    fn serializes_style_hover_and_completion_for_node_clients() -> napi::Result<()> {
        let source = ":root { --brand: #2563eb; }\n.button { color: var(--); }\n".to_string();
        let hover_json =
            read_style_hover_candidates_json(source.clone(), "fixture.module.css".to_string())
                .map_err(|error| napi::Error::from_reason(format!("{error:?}")))?;
        assert!(hover_json.contains("\"product\":\"omena-query.style-hover-candidates\""));
        assert!(hover_json.contains("\"name\":\"--brand\""));

        let completion_json =
            read_style_completion_at_position_json(source, "fixture.module.css".to_string(), 1, 23)
                .map_err(|error| napi::Error::from_reason(format!("{error:?}")))?;
        assert!(completion_json.contains("\"product\":\"omena-query.completion-at\""));
        assert!(completion_json.contains("\"label\":\"--brand\""));
        Ok(())
    }

    #[test]
    fn serializes_source_diagnostics_for_node_clients() -> napi::Result<()> {
        let json = read_source_diagnostics_json(
            "file:///workspace/src/App.tsx".to_string(),
            source_diagnostic_candidates_json().to_string(),
        )
        .map_err(|error| napi::Error::from_reason(format!("{error:?}")))?;

        assert!(json.contains("\"product\":\"omena-query.diagnostics-for-file\""));
        assert!(json.contains("\"fileKind\":\"source\""));
        assert!(json.contains("\"code\":\"missingSelector\""));
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
    fn reuses_expression_domain_flow_runtime_for_node_clients() -> napi::Result<()> {
        let mut runtime = OmenaNapiExpressionDomainFlowRuntimeV0::new();
        let input_json = reduced_product_projection_engine_input_json().to_string();

        let first_json = runtime.analyze_json(input_json.clone())?;
        assert!(
            first_json.contains(
                "\"product\":\"omena-query.expression-domain-incremental-flow-analysis\""
            )
        );
        assert!(first_json.contains("\"revision\":1"));
        assert!(first_json.contains("\"reusedGraphCount\":0"));

        let second_json = runtime.analyze_json(input_json)?;
        assert!(second_json.contains("\"revision\":2"));
        assert!(second_json.contains("\"dirtyGraphCount\":0"));
        assert!(second_json.contains("\"reusedGraphCount\":1"));
        assert!(second_json.contains("\"reusedPreviousAnalysis\":true"));
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
