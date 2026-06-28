//! Node native bindings for the Omena CSS parser and transform surface.

use napi_derive::napi;
use omena_query::{
    OmenaParserStyleDialect, OmenaQueryBundleArtifactV0 as OmenaNapiBundleArtifactV0,
    OmenaQueryCascadeAtPositionV0 as OmenaNapiCascadeAtPositionV0,
    OmenaQueryCompletionAtPositionV0 as OmenaNapiCompletionAtPositionV0,
    OmenaQueryConsumerBuildSummaryV0 as OmenaNapiBuildSummaryV0,
    OmenaQueryConsumerCheckSummaryV0 as OmenaNapiCheckSummaryV0,
    OmenaQueryEngineInputV2 as OmenaNapiEngineInputV2, OmenaQueryExpressionDomainFlowRuntimeV0,
    OmenaQueryExpressionDomainIncrementalFlowAnalysisV0 as OmenaNapiExpressionDomainIncrementalFlowAnalysisV0,
    OmenaQueryExpressionDomainSelectorProjectionV0 as OmenaNapiExpressionDomainSelectorProjectionV0,
    OmenaQueryExternalModuleModeV0 as OmenaNapiExternalModuleModeV0,
    OmenaQueryExternalSifInputV0 as OmenaNapiExternalSifInputV0,
    OmenaQueryParseTreeNodeV0 as OmenaNapiParseTreeNodeV0,
    OmenaQuerySourceBindingIndexV0 as OmenaNapiSourceBindingIndexV0,
    OmenaQuerySourceDiagnosticsForFileV0 as OmenaNapiSourceDiagnosticsForFileV0,
    OmenaQuerySourceDocumentInputV0 as OmenaNapiSourceDocumentInputV0,
    OmenaQuerySourceImportedStyleBindingV0 as OmenaNapiSourceImportedStyleBindingV0,
    OmenaQuerySourceMissingSelectorDiagnosticCandidateV0 as OmenaNapiSourceMissingSelectorDiagnosticCandidateV0,
    OmenaQuerySourceSyntaxIndexV0 as OmenaNapiSourceSyntaxIndexV0,
    OmenaQuerySourceTypeFactControlFlowGraphV0 as OmenaNapiSourceTypeFactControlFlowGraphV0,
    OmenaQueryStyleContextIndexV0 as OmenaNapiStyleContextIndexV0,
    OmenaQueryStyleDiagnosticsForFileV0 as OmenaNapiStyleDiagnosticsForFileV0,
    OmenaQueryStyleHoverCandidatesV0 as OmenaNapiStyleHoverCandidatesV0, OmenaQueryStyleMemoHostV0,
    OmenaQueryStylePackageManifestV0 as OmenaNapiStylePackageManifestV0,
    OmenaQueryStyleResolutionInputsV0, OmenaQueryStyleSourceInputV0 as OmenaNapiStyleSourceInputV0,
    OmenaQueryTargetTransformOptionsV0 as OmenaNapiTargetTransformOptionsV0,
    OmenaQueryTransformBundleSourceSummaryV0 as OmenaNapiTransformBundleSourceSummaryV0,
    OmenaQueryTransformContextFromEngineInputSummaryV0 as OmenaNapiTransformContextFromEngineInputSummaryV0,
    OmenaQueryTransformExecutionContextV0 as OmenaNapiTransformExecutionContextV0,
    OmenaQueryTransformPassSummaryV0 as OmenaNapiPassSummaryV0, ParserPositionV0,
    attach_omena_query_consumer_build_source_map_v3_with_sources,
    conservative_omena_query_target_options, execute_omena_query_consumer_build_style_source,
    execute_omena_query_consumer_build_style_source_for_target_query,
    execute_omena_query_consumer_build_style_source_for_target_query_with_context_and_options,
    execute_omena_query_consumer_build_style_source_for_target_query_with_options,
    execute_omena_query_consumer_build_style_source_with_context,
    execute_omena_query_consumer_build_style_source_with_engine_input_context,
    execute_omena_query_consumer_build_style_sources_for_target_query_with_context_and_options,
    execute_omena_query_consumer_build_style_sources_with_context,
    list_omena_query_transform_pass_summaries, parse_style_document_typed_v0,
    read_omena_query_cascade_at_position, read_omena_query_style_context_index,
    run_omena_query_bundle_for_style_sources_with_context,
    summarize_omena_query_consumer_check_style_source,
    summarize_omena_query_expression_domain_incremental_flow_analysis,
    summarize_omena_query_expression_domain_selector_projection,
    summarize_omena_query_source_binding_index_for_source_language,
    summarize_omena_query_source_diagnostics_for_file,
    summarize_omena_query_source_diagnostics_for_workspace_file,
    summarize_omena_query_source_syntax_index_for_source_language,
    summarize_omena_query_source_type_fact_control_flow_graph_for_source_language,
    summarize_omena_query_style_completion_at_position,
    summarize_omena_query_style_diagnostics_for_file, summarize_omena_query_style_hover_candidates,
    summarize_omena_query_transform_context_from_engine_input,
    summarize_omena_transform_bundle_from_source,
};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[napi(js_name = "checkStyleSourceJson")]
pub fn check_style_source_json(source: String, path: String) -> napi::Result<String> {
    to_json_string(&check_style_source_summary(&source, &path))
}

#[napi(js_name = "parseStylesheetJson")]
pub fn parse_stylesheet_json(source: String, path: String) -> napi::Result<String> {
    to_json_string(&parse_stylesheet_summary(&source, &path))
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

#[napi(js_name = "bundleStyleSourcesWithContextJson")]
pub fn bundle_style_sources_with_context_json(
    target_path: String,
    sources_json: String,
    pass_ids: Vec<String>,
    context_json: String,
    package_manifests_json: String,
    bundle_entry_style_paths: Vec<String>,
) -> napi::Result<String> {
    let sources = parse_style_sources_json(&sources_json)?;
    let context = parse_context_json(&context_json)?;
    let package_manifests = parse_package_manifests_json(&package_manifests_json)?;
    to_json_string(&bundle_style_sources_with_context_summary(
        &target_path,
        &sources,
        &pass_ids,
        &context,
        &package_manifests,
        &bundle_entry_style_paths,
    )?)
}

#[napi(js_name = "buildStyleSourcesMinifiedWithContextJson")]
pub fn build_style_sources_minified_with_context_json(
    target_path: String,
    sources_json: String,
    context_json: String,
    package_manifests_json: String,
) -> napi::Result<String> {
    let sources = parse_style_sources_json(&sources_json)?;
    let context = parse_context_json(&context_json)?;
    let package_manifests = parse_package_manifests_json(&package_manifests_json)?;
    to_json_string(&build_style_sources_with_context_summary(
        &target_path,
        &sources,
        &minify_pass_ids(),
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

#[napi(js_name = "summarizeTransformBundleFromSourceJson")]
pub fn summarize_transform_bundle_from_source_json(
    source: String,
    path: String,
) -> napi::Result<String> {
    to_json_string(&summarize_transform_bundle_from_source_summary(
        &source, &path,
    ))
}

fn minify_pass_ids() -> Vec<String> {
    [
        "comment-strip",
        "whitespace-strip",
        "number-compression",
        "color-compression",
        "shorthand-combining",
        "rule-deduplication",
        "rule-merging",
        "selector-merging",
        "empty-rule-removal",
        "calc-reduction",
        "print-css",
    ]
    .iter()
    .map(|pass_id| (*pass_id).to_string())
    .collect()
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
    source_documents_json: String,
    package_manifests_json: String,
    external_sifs_json: Option<String>,
    external_mode: Option<String>,
) -> napi::Result<String> {
    let sources = parse_style_sources_json(&sources_json)?;
    let source_documents = parse_source_documents_json(&source_documents_json)?;
    let package_manifests = parse_package_manifests_json(&package_manifests_json)?;
    let external_sifs = parse_external_sifs_json(external_sifs_json.as_deref())?;
    to_json_string(&read_workspace_style_diagnostics_summary(
        &target_path,
        &sources,
        &source_documents,
        &package_manifests,
        &external_sifs,
        external_mode.as_deref(),
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

#[napi(js_name = "readWorkspaceSourceDiagnosticsJson")]
pub fn read_workspace_source_diagnostics_json(
    source_uri: String,
    source: String,
    sources_json: String,
    package_manifests_json: String,
) -> napi::Result<String> {
    let sources = parse_style_sources_json(&sources_json)?;
    let package_manifests = parse_package_manifests_json(&package_manifests_json)?;
    to_json_string(&read_workspace_source_diagnostics_summary(
        &source_uri,
        &source,
        &sources,
        &package_manifests,
    ))
}

#[napi(js_name = "readSourceSyntaxIndexJson")]
pub fn read_source_syntax_index_json(
    source_path: String,
    source: String,
    source_language: Option<String>,
    imported_style_bindings_json: String,
    classnames_bind_bindings_json: String,
) -> napi::Result<String> {
    let imported_style_bindings =
        parse_source_imported_style_bindings_json(&imported_style_bindings_json)?;
    let classnames_bind_bindings =
        parse_classnames_bind_bindings_json(&classnames_bind_bindings_json)?;
    to_json_string(&read_source_syntax_index_summary(
        &source_path,
        &source,
        source_language.as_deref(),
        imported_style_bindings,
        classnames_bind_bindings,
    ))
}

#[napi(js_name = "readSourceBindingIndexJson")]
pub fn read_source_binding_index_json(
    source_path: String,
    source: String,
    source_language: Option<String>,
    imported_style_bindings_json: String,
    classnames_bind_bindings_json: String,
) -> napi::Result<String> {
    let imported_style_bindings =
        parse_source_imported_style_bindings_json(&imported_style_bindings_json)?;
    let classnames_bind_bindings =
        parse_classnames_bind_bindings_json(&classnames_bind_bindings_json)?;
    to_json_string(&read_source_binding_index_summary(
        &source_path,
        &source,
        source_language.as_deref(),
        imported_style_bindings,
        classnames_bind_bindings,
    ))
}

#[napi(js_name = "readSourceTypeFactControlFlowGraphJson")]
pub fn read_source_type_fact_control_flow_graph_json(
    source_path: String,
    source: String,
    source_language: Option<String>,
    variable_name: String,
    reference_byte_offset: u32,
) -> napi::Result<String> {
    to_json_string(&read_source_type_fact_control_flow_graph_summary(
        &source_path,
        &source,
        source_language.as_deref(),
        &variable_name,
        reference_byte_offset as usize,
    ))
}

#[napi(js_name = "ExpressionDomainFlowRuntime")]
pub struct OmenaNapiExpressionDomainFlowRuntimeV0 {
    inner: OmenaQueryExpressionDomainFlowRuntimeV0,
}

impl Default for OmenaNapiExpressionDomainFlowRuntimeV0 {
    fn default() -> Self {
        Self::new()
    }
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

pub fn parse_stylesheet_summary(source: &str, path: &str) -> OmenaNapiParseTreeNodeV0 {
    parse_style_document_typed_v0(source, infer_style_dialect(effective_path(path)))
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
    let mut summary = execute_omena_query_consumer_build_style_sources_with_context(
        target_path,
        sources,
        pass_ids,
        context,
        package_manifests,
    )
    .map_err(napi::Error::from_reason)?;
    attach_omena_query_consumer_build_source_map_v3_with_sources(
        &mut summary,
        sources,
        package_manifests,
    );
    Ok(summary)
}

pub fn bundle_style_sources_with_context_summary(
    target_path: &str,
    sources: &[OmenaNapiStyleSourceInputV0],
    pass_ids: &[String],
    context: &OmenaNapiTransformExecutionContextV0,
    package_manifests: &[OmenaNapiStylePackageManifestV0],
    bundle_entry_style_paths: &[String],
) -> napi::Result<OmenaNapiBundleArtifactV0> {
    run_omena_query_bundle_for_style_sources_with_context(
        target_path,
        sources,
        pass_ids,
        context,
        package_manifests,
        bundle_entry_style_paths,
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
    let mut summary =
        execute_omena_query_consumer_build_style_sources_for_target_query_with_context_and_options(
            target_path,
            sources,
            target_query,
            context,
            target_options,
            package_manifests,
        )
        .map_err(napi::Error::from_reason)?;
    attach_omena_query_consumer_build_source_map_v3_with_sources(
        &mut summary,
        sources,
        package_manifests,
    );
    Ok(summary)
}

pub fn list_transform_pass_summaries() -> Vec<OmenaNapiPassSummaryV0> {
    list_omena_query_transform_pass_summaries()
}

pub fn summarize_transform_bundle_from_source_summary(
    source: &str,
    path: &str,
) -> OmenaNapiTransformBundleSourceSummaryV0 {
    let path = effective_path(path);
    summarize_omena_transform_bundle_from_source(path, source, infer_style_dialect(path))
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
    source_documents: &[OmenaNapiSourceDocumentInputV0],
    package_manifests: &[OmenaNapiStylePackageManifestV0],
    external_sifs: &[OmenaNapiExternalSifInputV0],
    external_mode: Option<&str>,
) -> napi::Result<OmenaNapiStyleDiagnosticsForFileV0> {
    let target_path = effective_path(target_path);
    let external_mode = parse_external_module_mode(external_mode)?;
    let resolution_inputs = OmenaQueryStyleResolutionInputsV0 {
        package_manifests: package_manifests.to_vec(),
        ..Default::default()
    };
    let mut host = OmenaQueryStyleMemoHostV0::new();
    host.workspace_revision_selector(
        sources,
        source_documents,
        package_manifests,
        external_sifs,
        &resolution_inputs,
    )
    .and_then(|selector| {
        selector.workspace_style_diagnostics_with_external_mode(target_path, external_mode)
    })
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

pub fn read_workspace_source_diagnostics_summary(
    source_uri: &str,
    source: &str,
    style_sources: &[OmenaNapiStyleSourceInputV0],
    package_manifests: &[OmenaNapiStylePackageManifestV0],
) -> OmenaNapiSourceDiagnosticsForFileV0 {
    summarize_omena_query_source_diagnostics_for_workspace_file(
        source_uri,
        source,
        style_sources,
        package_manifests,
    )
}

pub fn read_source_syntax_index_summary(
    source_path: &str,
    source: &str,
    source_language: Option<&str>,
    imported_style_bindings: Vec<OmenaNapiSourceImportedStyleBindingV0>,
    classnames_bind_bindings: Vec<String>,
) -> OmenaNapiSourceSyntaxIndexV0 {
    summarize_omena_query_source_syntax_index_for_source_language(
        source_path,
        source,
        source_language,
        imported_style_bindings,
        classnames_bind_bindings,
    )
}

pub fn read_source_binding_index_summary(
    source_path: &str,
    source: &str,
    source_language: Option<&str>,
    imported_style_bindings: Vec<OmenaNapiSourceImportedStyleBindingV0>,
    classnames_bind_bindings: Vec<String>,
) -> OmenaNapiSourceBindingIndexV0 {
    summarize_omena_query_source_binding_index_for_source_language(
        source_path,
        source,
        source_language,
        imported_style_bindings,
        classnames_bind_bindings,
    )
}

pub fn read_source_type_fact_control_flow_graph_summary(
    source_path: &str,
    source: &str,
    source_language: Option<&str>,
    variable_name: &str,
    reference_byte_offset: usize,
) -> Option<OmenaNapiSourceTypeFactControlFlowGraphV0> {
    summarize_omena_query_source_type_fact_control_flow_graph_for_source_language(
        source_path,
        source,
        source_language,
        variable_name,
        reference_byte_offset,
    )
}

fn parse_target_options_json(
    target_options_json: &str,
) -> napi::Result<OmenaNapiTargetTransformOptionsV0> {
    if json_argument_is_absent(target_options_json) {
        return Ok(conservative_omena_query_target_options());
    }
    serde_json::from_str(target_options_json).map_err(|error| {
        napi::Error::from_reason(format!("failed to parse target options JSON: {error}"))
    })
}

fn parse_context_json(context_json: &str) -> napi::Result<OmenaNapiTransformExecutionContextV0> {
    if json_argument_is_absent(context_json) {
        return Ok(OmenaNapiTransformExecutionContextV0::default());
    }
    serde_json::from_str(context_json).map_err(|error| {
        napi::Error::from_reason(format!("failed to parse transform context JSON: {error}"))
    })
}

fn parse_style_sources_json(sources_json: &str) -> napi::Result<Vec<OmenaNapiStyleSourceInputV0>> {
    serde_json::from_str(sources_json).map_err(|error| {
        napi::Error::from_reason(format!("failed to parse style sources JSON: {error}"))
    })
}

fn parse_source_documents_json(
    source_documents_json: &str,
) -> napi::Result<Vec<OmenaNapiSourceDocumentInputV0>> {
    if json_argument_is_absent(source_documents_json) {
        return Ok(Vec::new());
    }
    serde_json::from_str(source_documents_json).map_err(|error| {
        napi::Error::from_reason(format!("failed to parse source documents JSON: {error}"))
    })
}

fn parse_package_manifests_json(
    package_manifests_json: &str,
) -> napi::Result<Vec<OmenaNapiStylePackageManifestV0>> {
    if json_argument_is_absent(package_manifests_json) {
        return Ok(Vec::new());
    }
    serde_json::from_str(package_manifests_json).map_err(|error| {
        napi::Error::from_reason(format!("failed to parse package manifests JSON: {error}"))
    })
}

fn parse_external_sifs_json(
    external_sifs_json: Option<&str>,
) -> napi::Result<Vec<OmenaNapiExternalSifInputV0>> {
    let Some(external_sifs_json) = external_sifs_json else {
        return Ok(Vec::new());
    };
    if json_argument_is_absent(external_sifs_json) {
        return Ok(Vec::new());
    }
    serde_json::from_str(external_sifs_json).map_err(|error| {
        napi::Error::from_reason(format!("failed to parse external SIFs JSON: {error}"))
    })
}

fn parse_external_module_mode(
    external_mode: Option<&str>,
) -> napi::Result<OmenaNapiExternalModuleModeV0> {
    match external_mode {
        None => Ok(OmenaNapiExternalModuleModeV0::Ignored),
        Some(mode) if json_argument_is_absent(mode) => Ok(OmenaNapiExternalModuleModeV0::Ignored),
        Some("ignored") => Ok(OmenaNapiExternalModuleModeV0::Ignored),
        Some("sif") => Ok(OmenaNapiExternalModuleModeV0::Sif),
        Some(other) => Err(napi::Error::from_reason(format!(
            "unsupported external mode '{other}'; expected ignored or sif"
        ))),
    }
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

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SourceImportedStyleBindingInputV0 {
    binding: String,
    style_uri: String,
}

fn parse_source_imported_style_bindings_json(
    input_json: &str,
) -> napi::Result<Vec<OmenaNapiSourceImportedStyleBindingV0>> {
    if json_argument_is_absent(input_json) {
        return Ok(Vec::new());
    }
    let inputs = serde_json::from_str::<Vec<SourceImportedStyleBindingInputV0>>(input_json)
        .map_err(|error| {
            napi::Error::from_reason(format!(
                "failed to parse source imported style bindings JSON: {error}"
            ))
        })?;
    Ok(inputs
        .into_iter()
        .map(|input| OmenaNapiSourceImportedStyleBindingV0 {
            binding: input.binding,
            style_uri: input.style_uri,
        })
        .collect())
}

fn parse_classnames_bind_bindings_json(input_json: &str) -> napi::Result<Vec<String>> {
    if json_argument_is_absent(input_json) {
        return Ok(Vec::new());
    }
    serde_json::from_str(input_json).map_err(|error| {
        napi::Error::from_reason(format!(
            "failed to parse classnames bind bindings JSON: {error}"
        ))
    })
}

fn parse_optional_engine_input_json(input_json: &str) -> napi::Result<OmenaNapiEngineInputV2> {
    if json_argument_is_absent(input_json) {
        return Ok(empty_engine_input());
    }
    parse_engine_input_json(input_json)
}

fn json_argument_is_absent(input_json: &str) -> bool {
    let trimmed = input_json.trim();
    trimmed.is_empty()
        || trimmed.eq_ignore_ascii_case("null")
        || trimmed.eq_ignore_ascii_case("undefined")
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

fn infer_style_dialect(path: &str) -> OmenaParserStyleDialect {
    match Path::new(path)
        .extension()
        .and_then(|extension| extension.to_str())
    {
        Some("scss") => OmenaParserStyleDialect::Scss,
        Some("sass") => OmenaParserStyleDialect::Sass,
        Some("less") => OmenaParserStyleDialect::Less,
        _ => OmenaParserStyleDialect::Css,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_query_diagnostics_json_shape(
        json: &str,
        expected_file_kind: &str,
        expected_code: &str,
    ) -> napi::Result<()> {
        let value = serde_json::from_str::<serde_json::Value>(json)
            .map_err(|error| napi::Error::from_reason(format!("diagnostics JSON: {error}")))?;
        assert_eq!(
            value.get("schemaVersion").and_then(|value| value.as_str()),
            Some("0")
        );
        assert_eq!(
            value.get("product").and_then(|value| value.as_str()),
            Some("omena-query.diagnostics-for-file")
        );
        assert_eq!(
            value.get("fileKind").and_then(|value| value.as_str()),
            Some(expected_file_kind)
        );
        assert!(
            value
                .get("fileUri")
                .and_then(|value| value.as_str())
                .is_some_and(|value| !value.is_empty())
        );
        let diagnostics = value
            .get("diagnostics")
            .and_then(|value| value.as_array())
            .ok_or_else(|| {
                napi::Error::from_reason("diagnostics must be serialized as an array")
            })?;
        assert_eq!(
            value
                .get("diagnosticCount")
                .and_then(|value| value.as_u64()),
            Some(diagnostics.len() as u64)
        );
        assert!(
            value
                .get("readySurfaces")
                .and_then(|value| value.as_array())
                .is_some_and(|surfaces| !surfaces.is_empty())
        );
        assert!(
            diagnostics.iter().any(|diagnostic| diagnostic
                .get("code")
                .and_then(|value| value.as_str())
                == Some(expected_code)),
            "diagnostics must include {expected_code}: {json}"
        );
        assert!(
            diagnostics
                .iter()
                .all(|diagnostic| diagnostic.get("category").is_none())
        );
        assert!(diagnostics.iter().all(|diagnostic| {
            diagnostic
                .get("provenance")
                .and_then(|value| value.as_array())
                .is_some_and(|provenance| !provenance.is_empty())
        }));
        Ok(())
    }

    #[test]
    fn accepts_absent_json_sentinels_for_optional_node_inputs() -> napi::Result<()> {
        assert_eq!(
            parse_target_options_json("null")?,
            conservative_omena_query_target_options()
        );
        assert_eq!(
            parse_context_json("undefined")?,
            OmenaNapiTransformExecutionContextV0::default()
        );
        assert!(parse_source_documents_json("null")?.is_empty());
        assert!(parse_package_manifests_json("undefined")?.is_empty());
        let empty_input = parse_optional_engine_input_json("null")?;
        assert_eq!(empty_input.version, "2");
        assert!(empty_input.sources.is_empty());
        assert!(empty_input.styles.is_empty());
        assert!(empty_input.type_facts.is_empty());
        assert!(json_argument_is_absent("  null  "));
        Ok(())
    }

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
    fn parses_stylesheet_tree_for_node_clients_through_query() -> napi::Result<()> {
        let summary = parse_stylesheet_summary(".card { color: red; }", "fixture.module.css");
        let query =
            parse_style_document_typed_v0(".card { color: red; }", OmenaParserStyleDialect::Css);

        assert_eq!(summary, query);
        assert_eq!(summary.kind, "Root");
        assert!(
            summary
                .children
                .iter()
                .any(|child| child.kind == "Stylesheet")
        );

        let json = parse_stylesheet_json(
            ".card { color: red; }".to_string(),
            "fixture.module.css".to_string(),
        )?;
        assert!(json.contains("\"kind\":\"Root\""));
        assert!(json.contains("\"kind\":\"Stylesheet\""));
        Ok(())
    }

    #[test]
    fn summarizes_bundle_plan_for_node_clients() {
        let summary = summarize_transform_bundle_from_source_summary(
            r#"@use "./tokens" as tokens;
@import "./base.css";
@value primary from "./colors.module.css";
.card { composes: reset from "./reset.module.css"; color: tokens.$brand; }"#,
            "Button.module.scss",
        );

        assert_eq!(summary.product, "omena-transform-bundle.source");
        assert_eq!(summary.dialect, "scss");
        assert!(summary.planned_pass_ids.contains(&"import-inline"));
        assert!(summary.planned_pass_ids.contains(&"scss-module-evaluate"));
    }

    #[test]
    fn reads_cascade_lfp_for_node_clients() -> Result<(), String> {
        let input = empty_engine_input();
        let Some(summary) = read_cascade_at_position_summary(
            ":root { --known: #2563eb; }\n.button { color: var(--known); }\n",
            "fixture.module.css",
            1,
            24,
            &input,
        ) else {
            return Err("cascade summary should be available".to_string());
        };

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
        Ok(())
    }

    #[test]
    fn reads_style_context_index_for_node_clients() -> Result<(), String> {
        let input = empty_engine_input();
        let Some(summary) = read_style_context_index_summary(
            "@layer components { @container card (min-width: 20rem) { .card { color: red; } } }",
            "fixture.module.css",
            &input,
        ) else {
            return Err("context index summary should be available".to_string());
        };

        assert_eq!(summary.product, "omena-query.style-context-index");
        assert_eq!(summary.context_index.layer_index.block_layers.len(), 1);
        assert_eq!(
            summary.context_index.container_index.named_container_count,
            1
        );
        Ok(())
    }

    #[test]
    fn reads_style_diagnostics_for_node_clients() -> Result<(), String> {
        let summary = read_style_diagnostics_summary(
            ":root { --known: #2563eb; }\n.button { color: var(--missing); animation: fade 1s; }\n",
            "fixture.module.css",
        )
        .map_err(|_| "style diagnostics should be available".to_string())?;

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
        Ok(())
    }

    #[test]
    fn reads_workspace_style_diagnostics_for_node_clients() -> Result<(), String> {
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
            &[],
            &[],
            None,
        )
        .map_err(|_| "workspace diagnostics should be available".to_string())?;

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
        Ok(())
    }

    #[test]
    fn reads_style_hover_and_completion_for_node_clients() -> Result<(), String> {
        let source = ":root { --brand: #2563eb; }\n.button { color: var(--); }\n";
        let hover = read_style_hover_candidates_summary(source, "fixture.module.css")
            .map_err(|_| "style hover candidates should be available".to_string())?;

        assert_eq!(hover.product, "omena-query.style-hover-candidates");
        assert!(
            hover
                .candidates
                .iter()
                .any(|candidate| candidate.name == "--brand")
        );

        let completion =
            read_style_completion_at_position_summary(source, "fixture.module.css", 1, 23)
                .map_err(|_| "style completion should be available".to_string())?;

        assert_eq!(completion.product, "omena-query.completion-at");
        assert!(completion.items.iter().any(|item| item.label == "--brand"));
        Ok(())
    }

    #[test]
    fn reads_source_diagnostics_for_node_clients() -> Result<(), String> {
        let candidates =
            parse_source_diagnostic_candidates_json(source_diagnostic_candidates_json())
                .map_err(|_| "source diagnostic candidates should parse".to_string())?;
        let summary =
            read_source_diagnostics_summary("file:///workspace/src/App.tsx", candidates.as_slice());

        assert_eq!(summary.product, "omena-query.diagnostics-for-file");
        assert_eq!(summary.file_kind, "source");
        assert_eq!(summary.diagnostic_count, 1);
        assert_eq!(summary.diagnostics[0].code, "missingSelector");
        assert!(summary.ready_surfaces.contains(&"crossLanguageDiagnostics"));
        Ok(())
    }

    #[test]
    fn serializes_source_frontend_indexes_for_node_clients() -> napi::Result<()> {
        let imported_style_bindings = serde_json::json!([
            {
                "binding": "styles",
                "styleUri": "file:///workspace/src/Card.module.scss"
            }
        ])
        .to_string();
        let classnames_bind_bindings = serde_json::json!(["cn"]).to_string();
        let syntax_json = read_source_syntax_index_json(
            "/workspace/src/Card.tsx".to_string(),
            source_frontend_fixture().to_string(),
            Some("typescriptreact".to_string()),
            imported_style_bindings.clone(),
            classnames_bind_bindings.clone(),
        )?;
        let binding_json = read_source_binding_index_json(
            "/workspace/src/Card.tsx".to_string(),
            source_frontend_fixture().to_string(),
            Some("typescriptreact".to_string()),
            imported_style_bindings,
            classnames_bind_bindings,
        )?;
        let syntax = serde_json::from_str::<serde_json::Value>(&syntax_json)
            .map_err(|error| napi::Error::from_reason(error.to_string()))?;
        let binding = serde_json::from_str::<serde_json::Value>(&binding_json)
            .map_err(|error| napi::Error::from_reason(error.to_string()))?;

        assert_eq!(syntax["product"], "omena-bridge.source-syntax-index");
        assert!(
            syntax["stylePropertyAccesses"]
                .as_array()
                .is_some_and(|accesses| !accesses.is_empty())
        );
        assert!(
            syntax["selectorReferences"]
                .as_array()
                .is_some_and(|references| references
                    .iter()
                    .any(|reference| reference["targetStyleUri"]
                        == "file:///workspace/src/Card.module.scss"))
        );
        assert_eq!(binding["product"], "omena-bridge.source-binding-index");
        assert!(
            binding["classnamesBindUtilityBindings"]
                .as_array()
                .is_some_and(|bindings| bindings
                    .iter()
                    .any(|binding| binding["localName"] == "cx"
                        && binding["stylesLocalName"] == "styles"))
        );
        assert!(
            binding["symbolRefUsesDecls"]
                .as_array()
                .is_some_and(|references| references
                    .iter()
                    .any(|reference| reference["rootName"] == "size"))
        );
        Ok(())
    }

    #[test]
    fn serializes_source_type_fact_cfg_for_node_clients() -> napi::Result<()> {
        let source = [
            "export function Card({ active }: { active: boolean }) {",
            "  let size = \"card\";",
            "  if (active) {",
            "    size = \"card--active\";",
            "  }",
            "  return <div className={size} />;",
            "}",
            "",
        ]
        .join("\n");
        let reference = source
            .rfind("size")
            .ok_or_else(|| napi::Error::from_reason("fixture contains size reference"))?;
        let json = read_source_type_fact_control_flow_graph_json(
            "/workspace/src/Card.tsx".to_string(),
            source,
            Some("typescriptreact".to_string()),
            "size".to_string(),
            reference as u32,
        )?;
        let value = serde_json::from_str::<serde_json::Value>(&json)
            .map_err(|error| napi::Error::from_reason(error.to_string()))?;

        assert_eq!(value["entryBlockId"], "entry");
        assert!(
            value["blocks"]
                .as_array()
                .is_some_and(|blocks| blocks.iter().any(|block| block["kind"] == "branch"))
        );
        Ok(())
    }

    fn source_frontend_fixture() -> &'static str {
        r#"import cn from "classnames/bind";
import styles from "./Card.module.scss";

const cx = cn.bind(styles);

export function Card({ active }: { active: boolean }) {
  let size = "card";
  if (active) {
    size = "card--active";
  }
  return <div className={cx("card", size, styles.icon)} />;
}
"#
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
    }

    #[test]
    fn builds_css_from_evaluator_context_for_node_clients() {
        let context = OmenaNapiTransformExecutionContextV0 {
            scss_module_evaluation: Some(omena_query::OmenaQueryTransformModuleEvaluationV0 {
                evaluator: "dart-sass-compatible".to_string(),
                product_output_source: Some("nativeEditOutput".to_string()),
                evaluated_css: ".card { color: red; }".to_string(),
                native_edit_output: Some(".card { color: red; }".to_string()),
                native_replacements: Vec::new(),
                native_edits: Vec::new(),
                oracle: None,
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
                enable_container_static_eval: false,
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
        assert!(summary.ready_surfaces.contains(&"sourceMapV3Serializer"));
        assert!(summary.source_map_v3.is_some());
        assert!(!summary.execution.output_css.contains("@import"));
        assert!(!summary.execution.output_css.contains("composes:"));
    }

    #[test]
    fn bundles_workspace_sources_for_node_clients() {
        let sources = vec![
            OmenaNapiStyleSourceInputV0 {
                style_path: "Button.module.css".to_string(),
                style_source: r#"@import "./tokens.css"; .button { color: var(--brand); }"#
                    .to_string(),
            },
            OmenaNapiStyleSourceInputV0 {
                style_path: "tokens.css".to_string(),
                style_source: ":root { --brand: red; }".to_string(),
            },
        ];
        let pass_ids = vec!["import-inline".to_string(), "print-css".to_string()];
        let artifact_result = bundle_style_sources_with_context_summary(
            "Button.module.css",
            &sources,
            &pass_ids,
            &OmenaNapiTransformExecutionContextV0::default(),
            &[],
            &[],
        );

        assert!(artifact_result.is_ok());
        let Ok(artifact) = artifact_result else {
            return;
        };
        assert_eq!(artifact.product, "omena-query.bundle-artifact");
        assert!(artifact.ready_surfaces.contains(&"bundleOperationFacade"));
        assert!(artifact.source_map_v3.sources.len() >= 2);
        assert_eq!(artifact.per_pass_provenance, artifact.execution.outcomes);
        assert!(!artifact.output_css.contains("@import"));
    }

    #[test]
    fn builds_minified_workspace_sources_for_node_clients() {
        let sources = vec![OmenaNapiStyleSourceInputV0 {
            style_path: "Card.module.css".to_string(),
            style_source:
                ".card { color: #ffffff; margin-top: 1px; margin-right: 2px; margin-bottom: 3px; margin-left: 4px; } .empty {}"
                    .to_string(),
        }];
        let summary_result = build_style_sources_with_context_summary(
            "Card.module.css",
            &sources,
            &minify_pass_ids(),
            &OmenaNapiTransformExecutionContextV0::default(),
            &[],
        );

        assert!(summary_result.is_ok());
        let Ok(summary) = summary_result else {
            return;
        };
        assert!(
            summary
                .requested_pass_ids
                .iter()
                .any(|pass_id| pass_id == "number-compression")
        );
        assert!(
            summary
                .requested_pass_ids
                .iter()
                .any(|pass_id| pass_id == "color-compression")
        );
        assert!(summary.execution.output_css.contains("#fff"));
        assert!(!summary.execution.output_css.contains(".empty"));
        assert!(summary.ready_surfaces.contains(&"sourceMapV3Serializer"));
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

        assert_query_diagnostics_json_shape(&json, "style", "missingCustomProperty")?;
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
            String::new(),
            None,
            None,
        )
        .map_err(|error| napi::Error::from_reason(format!("{error:?}")))?;

        assert_query_diagnostics_json_shape(&json, "style", "missingComposedSelector")?;
        Ok(())
    }

    #[test]
    fn carries_external_sif_mode_in_workspace_diagnostics_for_node_clients() -> napi::Result<()> {
        let sources = r#"[
{"stylePath":"/tmp/App.module.scss","styleSource":"@use \"https://cdn.example/tokens.scss\" as remote;\n.button { color: remote.$brand; }"}
]"#;

        // Default (no external params) is byte-for-byte the Ignored surface: no SIF boundary.
        let ignored = read_workspace_style_diagnostics_json(
            "/tmp/App.module.scss".to_string(),
            sources.to_string(),
            String::new(),
            String::new(),
            None,
            None,
        )
        .map_err(|error| napi::Error::from_reason(format!("{error:?}")))?;
        assert!(!ignored.contains("\"code\":\"missingExternalSif\""));

        // external_mode = "sif" with empty external SIFs surfaces the missing-boundary diagnostic.
        let sif = read_workspace_style_diagnostics_json(
            "/tmp/App.module.scss".to_string(),
            sources.to_string(),
            String::new(),
            String::new(),
            None,
            Some("sif".to_string()),
        )
        .map_err(|error| napi::Error::from_reason(format!("{error:?}")))?;
        assert!(sif.contains("\"code\":\"missingExternalSif\""));
        assert!(sif.contains("externalSifBoundaryDiagnostics"));
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

        assert_query_diagnostics_json_shape(&json, "source", "missingSelector")?;
        Ok(())
    }

    #[test]
    fn serializes_workspace_source_diagnostics_for_node_clients() -> napi::Result<()> {
        let json = read_workspace_source_diagnostics_json(
            "/workspace/src/App.tsx".to_string(),
            r#"import bind from "classnames/bind";
import styles from "./App.module.scss";
const cx = bind.bind(styles);
const variant = Math.random() > 0.5 ? "chip" : "ghost";
export function App() {
  return <div className={cx(variant)} />;
}
"#
            .to_string(),
            r#"[
              {
                "stylePath": "/workspace/src/App.module.scss",
                "styleSource": ".chip {}"
              }
            ]"#
            .to_string(),
            "[]".to_string(),
        )
        .map_err(|error| napi::Error::from_reason(format!("{error:?}")))?;

        assert_query_diagnostics_json_shape(&json, "source", "missingResolvedClassValues")?;
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
        assert!(json.contains("\"sourceValueKind\":\"prefixSuffix\""));
        assert!(json.contains("\"prefix\":\"btn-primary-\""));
        assert!(json.contains("\"prefix\":\"btn-secondary-\""));
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
        assert!(second_json.contains("\"reusedGraphCount\":2"));
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
        assert!(json.contains("\"selectedProjectionCount\":2"));
        assert!(json.contains("\"reachabilitySources\""));
        assert!(!json.contains("\"nodeId\":\"file-merge\""));
        assert!(json.contains("\"nodeId\":\"expr-primary\""));
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

        assert!(passes.len() >= 41);
        assert!(passes.iter().any(|pass| pass.id == "whitespace-strip"));
    }

    fn reduced_product_projection_engine_input_json() -> &'static str {
        r#"{
          "version": "2",
          "workspace": {
            "root": "/tmp",
            "classnameTransform": "asIs",
            "settingsKey": "fixture"
          },
          "sources": [
            {
              "filePath": "/tmp/App.tsx",
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
          "workspace": {
            "root": "/tmp",
            "classnameTransform": "asIs",
            "settingsKey": "fixture"
          },
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
