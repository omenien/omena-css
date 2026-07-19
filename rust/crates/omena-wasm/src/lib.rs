//! Browser-side in-memory bindings for the Omena CSS parser and transform surface.

mod sdk_workspace;

pub use sdk_workspace::OmenaWasmWorkspaceV0;

use omena_query::{
    OmenaBundlerHostResolveModuleRequestV0, OmenaParserStyleDialect,
    OmenaQueryBuildVerificationProfileV0, OmenaQueryBundleArtifactV0 as OmenaWasmBundleArtifactV0,
    OmenaQueryBundleWithEvidenceV0 as OmenaWasmBundleWithEvidenceV0,
    OmenaQueryCascadeAtPositionV0 as OmenaWasmCascadeAtPositionV0,
    OmenaQueryCompletionAtPositionV0 as OmenaWasmCompletionAtPositionV0,
    OmenaQueryConsumerBuildOptionsV0, OmenaQueryConsumerBuildSummaryV0 as OmenaWasmBuildSummaryV0,
    OmenaQueryConsumerCheckSummaryV0 as OmenaWasmCheckSummaryV0,
    OmenaQueryEngineInputV2 as OmenaWasmEngineInputV2, OmenaQueryExpressionDomainFlowRuntimeV0,
    OmenaQueryExpressionDomainIncrementalFlowAnalysisV0 as OmenaWasmExpressionDomainIncrementalFlowAnalysisV0,
    OmenaQueryExpressionDomainSelectorProjectionV0 as OmenaWasmExpressionDomainSelectorProjectionV0,
    OmenaQueryExternalModuleModeV0 as OmenaWasmExternalModuleModeV0,
    OmenaQueryExternalSifInputV0 as OmenaWasmExternalSifInputV0,
    OmenaQueryParseTreeNodeV0 as OmenaWasmParseTreeNodeV0,
    OmenaQuerySourceBindingIndexV0 as OmenaWasmSourceBindingIndexV0,
    OmenaQuerySourceDiagnosticsForFileV0 as OmenaWasmSourceDiagnosticsForFileV0,
    OmenaQuerySourceDocumentInputV0 as OmenaWasmSourceDocumentInputV0,
    OmenaQuerySourceImportedStyleBindingV0 as OmenaWasmSourceImportedStyleBindingV0,
    OmenaQuerySourceMissingSelectorDiagnosticCandidateV0 as OmenaWasmSourceMissingSelectorDiagnosticCandidateV0,
    OmenaQuerySourceSyntaxIndexV0 as OmenaWasmSourceSyntaxIndexV0,
    OmenaQuerySourceTypeFactControlFlowGraphV0 as OmenaWasmSourceTypeFactControlFlowGraphV0,
    OmenaQueryStyleContextIndexV0 as OmenaWasmStyleContextIndexV0,
    OmenaQueryStyleDiagnosticsForFileV0 as OmenaWasmStyleDiagnosticsForFileV0,
    OmenaQueryStyleHoverCandidatesV0 as OmenaWasmStyleHoverCandidatesV0, OmenaQueryStyleMemoHostV0,
    OmenaQueryStylePackageManifestV0 as OmenaWasmStylePackageManifestV0,
    OmenaQueryStyleResolutionInputsV0, OmenaQueryStyleSourceInputV0 as OmenaWasmStyleSourceInputV0,
    OmenaQueryTargetTransformOptionsV0 as OmenaWasmTargetTransformOptionsV0,
    OmenaQueryTransformBundleSourceSummaryV0 as OmenaWasmTransformBundleSourceSummaryV0,
    OmenaQueryTransformContextFromEngineInputSummaryV0 as OmenaWasmTransformContextFromEngineInputSummaryV0,
    OmenaQueryTransformExecutionContextV0 as OmenaWasmTransformExecutionContextV0,
    OmenaQueryTransformPassSummaryV0 as OmenaWasmPassSummaryV0, ParserPositionV0,
    attach_omena_query_consumer_build_source_map_v3_with_sources,
    conservative_omena_query_target_options, current_omena_bundler_host_capabilities_v0,
    execute_omena_query_consumer_build_style_source,
    execute_omena_query_consumer_build_style_source_for_target_query,
    execute_omena_query_consumer_build_style_source_for_target_query_with_context_and_options,
    execute_omena_query_consumer_build_style_source_for_target_query_with_options,
    execute_omena_query_consumer_build_style_source_with_context,
    execute_omena_query_consumer_build_style_source_with_context_and_options,
    execute_omena_query_consumer_build_style_source_with_engine_input_context,
    execute_omena_query_consumer_build_style_sources_for_target_query_with_context_and_options,
    execute_omena_query_consumer_build_style_sources_with_context,
    list_omena_query_transform_pass_summaries, parse_style_document_typed_v0,
    read_omena_query_cascade_at_position, read_omena_query_style_context_index,
    resolve_omena_bundler_host_module_v0, run_omena_query_bundle_for_style_sources_with_context,
    run_omena_query_bundle_with_evidence_for_style_sources_with_context,
    semantic_omena_query_minify_build_profile, summarize_omena_query_consumer_check_style_source,
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
use wasm_bindgen::prelude::*;

#[wasm_bindgen(js_name = bundlerHostCapabilities)]
pub fn bundler_host_capabilities() -> Result<JsValue, JsValue> {
    to_json_compatible_js_value(&current_omena_bundler_host_capabilities_v0())
}

#[wasm_bindgen(js_name = resolveCssModuleForBundlerHost)]
pub fn resolve_css_module_for_bundler_host(request: JsValue) -> Result<JsValue, JsValue> {
    let request = serde_wasm_bindgen::from_value::<OmenaBundlerHostResolveModuleRequestV0>(request)
        .map_err(|error| JsValue::from_str(&format!("invalid bundler host request: {error}")))?;
    to_json_compatible_js_value(&resolve_omena_bundler_host_module_v0(request))
}

#[wasm_bindgen(js_name = checkStyleSource)]
pub fn check_style_source(source: &str, path: &str) -> Result<JsValue, JsValue> {
    to_js_value(&check_style_source_summary(source, path))
}

#[wasm_bindgen(js_name = parseStylesheet)]
pub fn parse_stylesheet(source: &str, path: &str) -> Result<JsValue, JsValue> {
    to_js_value(&parse_stylesheet_summary(source, path))
}

#[wasm_bindgen(js_name = buildStyleSource)]
pub fn build_style_source(source: &str, path: &str, pass_ids: JsValue) -> Result<JsValue, JsValue> {
    let pass_ids = parse_pass_ids_value(pass_ids)?;
    to_js_value(&build_style_source_summary(source, path, &pass_ids))
}

#[wasm_bindgen(js_name = buildStyleSourceWithVerificationProfile)]
pub fn build_style_source_with_verification_profile(
    source: &str,
    path: &str,
    pass_ids: JsValue,
    strict_verification: bool,
) -> Result<JsValue, JsValue> {
    let pass_ids = parse_pass_ids_value(pass_ids)?;
    to_js_value(&build_style_source_with_verification_summary(
        source,
        path,
        &pass_ids,
        strict_verification,
    ))
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

#[wasm_bindgen(js_name = bundleStyleSourcesWithContext)]
pub fn bundle_style_sources_with_context(
    target_path: &str,
    sources: JsValue,
    pass_ids: JsValue,
    context: JsValue,
    package_manifests: JsValue,
    bundle_entry_style_paths: JsValue,
) -> Result<JsValue, JsValue> {
    let sources = parse_style_sources_value(sources)?;
    let pass_ids = parse_pass_ids_value(pass_ids)?;
    let context = parse_context_value(context)?;
    let package_manifests = parse_package_manifests_value(package_manifests)?;
    let bundle_entry_style_paths =
        parse_string_array_value(bundle_entry_style_paths, "bundleEntryStylePaths")?;
    let artifact = bundle_style_sources_with_context_evidence(
        target_path,
        &sources,
        &pass_ids,
        &context,
        &package_manifests,
        &bundle_entry_style_paths,
    )?;
    to_js_value(&artifact)
}

#[wasm_bindgen(js_name = buildStyleSourcesMinifiedWithContext)]
pub fn build_style_sources_minified_with_context(
    target_path: &str,
    sources: JsValue,
    context: JsValue,
    package_manifests: JsValue,
) -> Result<JsValue, JsValue> {
    let sources = parse_style_sources_value(sources)?;
    let context = parse_context_value(context)?;
    let package_manifests = parse_package_manifests_value(package_manifests)?;
    let summary = build_style_sources_with_context_summary(
        target_path,
        &sources,
        &minify_pass_ids(),
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

#[wasm_bindgen(js_name = summarizeTransformBundleFromSource)]
pub fn summarize_transform_bundle_from_source(
    source: &str,
    path: &str,
) -> Result<JsValue, JsValue> {
    to_js_value(&summarize_transform_bundle_from_source_summary(
        source, path,
    ))
}

fn minify_pass_ids() -> Vec<String> {
    semantic_omena_query_minify_build_profile()
        .pass_ids
        .into_iter()
        .map(str::to_string)
        .collect()
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

#[wasm_bindgen(js_name = readWorkspaceStyleDiagnostics)]
pub fn read_workspace_style_diagnostics(
    target_path: &str,
    sources: JsValue,
    source_documents: JsValue,
    package_manifests: JsValue,
    external_sifs: JsValue,
    external_mode: Option<String>,
) -> Result<JsValue, JsValue> {
    let sources = parse_style_sources_value(sources)?;
    let source_documents = parse_source_documents_value(source_documents)?;
    let package_manifests = parse_package_manifests_value(package_manifests)?;
    let external_sifs = parse_external_sifs_value(external_sifs)?;
    to_js_value(&read_workspace_style_diagnostics_summary(
        target_path,
        &sources,
        &source_documents,
        &package_manifests,
        &external_sifs,
        external_mode.as_deref(),
    )?)
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

#[wasm_bindgen(js_name = readWorkspaceSourceDiagnostics)]
pub fn read_workspace_source_diagnostics(
    source_uri: &str,
    source: &str,
    style_sources: JsValue,
    package_manifests: JsValue,
) -> Result<JsValue, JsValue> {
    let style_sources = parse_style_sources_value(style_sources)?;
    let package_manifests = parse_package_manifests_value(package_manifests)?;
    to_js_value(&read_workspace_source_diagnostics_summary(
        source_uri,
        source,
        &style_sources,
        &package_manifests,
    ))
}

#[wasm_bindgen(js_name = readSourceSyntaxIndex)]
pub fn read_source_syntax_index(
    source_path: &str,
    source: &str,
    source_language: Option<String>,
    imported_style_bindings: JsValue,
    classnames_bind_bindings: JsValue,
) -> Result<JsValue, JsValue> {
    let imported_style_bindings =
        parse_source_imported_style_bindings_value(imported_style_bindings)?;
    let classnames_bind_bindings = parse_classnames_bind_bindings_value(classnames_bind_bindings)?;
    to_js_value(&read_source_syntax_index_summary(
        source_path,
        source,
        source_language.as_deref(),
        imported_style_bindings,
        classnames_bind_bindings,
    ))
}

#[wasm_bindgen(js_name = readSourceBindingIndex)]
pub fn read_source_binding_index(
    source_path: &str,
    source: &str,
    source_language: Option<String>,
    imported_style_bindings: JsValue,
    classnames_bind_bindings: JsValue,
) -> Result<JsValue, JsValue> {
    let imported_style_bindings =
        parse_source_imported_style_bindings_value(imported_style_bindings)?;
    let classnames_bind_bindings = parse_classnames_bind_bindings_value(classnames_bind_bindings)?;
    to_js_value(&read_source_binding_index_summary(
        source_path,
        source,
        source_language.as_deref(),
        imported_style_bindings,
        classnames_bind_bindings,
    ))
}

#[wasm_bindgen(js_name = readSourceTypeFactControlFlowGraph)]
pub fn read_source_type_fact_control_flow_graph(
    source_path: &str,
    source: &str,
    source_language: Option<String>,
    variable_name: &str,
    reference_byte_offset: u32,
) -> Result<JsValue, JsValue> {
    to_js_value(&read_source_type_fact_control_flow_graph_summary(
        source_path,
        source,
        source_language.as_deref(),
        variable_name,
        reference_byte_offset as usize,
    ))
}

#[wasm_bindgen(js_name = ExpressionDomainFlowRuntime)]
pub struct OmenaWasmExpressionDomainFlowRuntimeV0 {
    inner: OmenaQueryExpressionDomainFlowRuntimeV0,
}

impl Default for OmenaWasmExpressionDomainFlowRuntimeV0 {
    fn default() -> Self {
        Self::new()
    }
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

pub fn parse_stylesheet_summary(source: &str, path: &str) -> OmenaWasmParseTreeNodeV0 {
    parse_style_document_typed_v0(source, infer_style_dialect(effective_path(path)))
}

pub fn build_style_source_summary(
    source: &str,
    path: &str,
    pass_ids: &[String],
) -> OmenaWasmBuildSummaryV0 {
    let path = effective_path(path);
    execute_omena_query_consumer_build_style_source(path, source, pass_ids)
}

pub fn build_style_source_with_verification_summary(
    source: &str,
    path: &str,
    pass_ids: &[String],
    strict_verification: bool,
) -> OmenaWasmBuildSummaryV0 {
    let path = effective_path(path);
    execute_omena_query_consumer_build_style_source_with_context_and_options(
        path,
        source,
        pass_ids,
        &OmenaWasmTransformExecutionContextV0::default(),
        &OmenaQueryConsumerBuildOptionsV0 {
            verification_profile: if strict_verification {
                OmenaQueryBuildVerificationProfileV0::Strict
            } else {
                OmenaQueryBuildVerificationProfileV0::Descriptive
            },
            ..OmenaQueryConsumerBuildOptionsV0::default()
        },
    )
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
    let mut summary = execute_omena_query_consumer_build_style_sources_with_context(
        target_path,
        sources,
        pass_ids,
        context,
        package_manifests,
    )
    .map_err(|error| JsValue::from_str(&error))?;
    attach_omena_query_consumer_build_source_map_v3_with_sources(
        &mut summary,
        sources,
        package_manifests,
    );
    Ok(summary)
}

pub fn bundle_style_sources_with_context_summary(
    target_path: &str,
    sources: &[OmenaWasmStyleSourceInputV0],
    pass_ids: &[String],
    context: &OmenaWasmTransformExecutionContextV0,
    package_manifests: &[OmenaWasmStylePackageManifestV0],
    bundle_entry_style_paths: &[String],
) -> Result<OmenaWasmBundleArtifactV0, JsValue> {
    run_omena_query_bundle_for_style_sources_with_context(
        target_path,
        sources,
        pass_ids,
        context,
        package_manifests,
        bundle_entry_style_paths,
    )
    .map_err(|error| JsValue::from_str(&error))
}

pub fn bundle_style_sources_with_context_evidence(
    target_path: &str,
    sources: &[OmenaWasmStyleSourceInputV0],
    pass_ids: &[String],
    context: &OmenaWasmTransformExecutionContextV0,
    package_manifests: &[OmenaWasmStylePackageManifestV0],
    bundle_entry_style_paths: &[String],
) -> Result<OmenaWasmBundleWithEvidenceV0, JsValue> {
    run_omena_query_bundle_with_evidence_for_style_sources_with_context(
        target_path,
        sources,
        pass_ids,
        context,
        package_manifests,
        bundle_entry_style_paths,
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
    let mut summary =
        execute_omena_query_consumer_build_style_sources_for_target_query_with_context_and_options(
            target_path,
            sources,
            target_query,
            context,
            target_options,
            package_manifests,
        )
        .map_err(|error| JsValue::from_str(&error))?;
    attach_omena_query_consumer_build_source_map_v3_with_sources(
        &mut summary,
        sources,
        package_manifests,
    );
    Ok(summary)
}

pub fn list_transform_pass_summaries() -> Vec<OmenaWasmPassSummaryV0> {
    list_omena_query_transform_pass_summaries()
}

pub fn summarize_transform_bundle_from_source_summary(
    source: &str,
    path: &str,
) -> OmenaWasmTransformBundleSourceSummaryV0 {
    let path = effective_path(path);
    summarize_omena_transform_bundle_from_source(path, source, infer_style_dialect(path))
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

pub fn read_workspace_style_diagnostics_summary(
    target_path: &str,
    sources: &[OmenaWasmStyleSourceInputV0],
    source_documents: &[OmenaWasmSourceDocumentInputV0],
    package_manifests: &[OmenaWasmStylePackageManifestV0],
    external_sifs: &[OmenaWasmExternalSifInputV0],
    external_mode: Option<&str>,
) -> Result<OmenaWasmStyleDiagnosticsForFileV0, JsValue> {
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
        JsValue::from_str(&format!(
            "failed to read workspace style diagnostics for {target_path}"
        ))
    })
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

pub fn read_workspace_source_diagnostics_summary(
    source_uri: &str,
    source: &str,
    style_sources: &[OmenaWasmStyleSourceInputV0],
    package_manifests: &[OmenaWasmStylePackageManifestV0],
) -> OmenaWasmSourceDiagnosticsForFileV0 {
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
    imported_style_bindings: Vec<OmenaWasmSourceImportedStyleBindingV0>,
    classnames_bind_bindings: Vec<String>,
) -> OmenaWasmSourceSyntaxIndexV0 {
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
    imported_style_bindings: Vec<OmenaWasmSourceImportedStyleBindingV0>,
    classnames_bind_bindings: Vec<String>,
) -> OmenaWasmSourceBindingIndexV0 {
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
) -> Option<OmenaWasmSourceTypeFactControlFlowGraphV0> {
    summarize_omena_query_source_type_fact_control_flow_graph_for_source_language(
        source_path,
        source,
        source_language,
        variable_name,
        reference_byte_offset,
    )
}

fn parse_pass_ids_value(value: JsValue) -> Result<Vec<String>, JsValue> {
    if value.is_null() || value.is_undefined() {
        return Ok(Vec::new());
    }

    parse_string_array_value(value, "passIds")
}

fn parse_string_array_value(value: JsValue, field_name: &str) -> Result<Vec<String>, JsValue> {
    serde_wasm_bindgen::from_value(value).map_err(|error| {
        JsValue::from_str(&format!(
            "{field_name} must be an array of strings: {error}"
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

fn parse_source_documents_value(
    value: JsValue,
) -> Result<Vec<OmenaWasmSourceDocumentInputV0>, JsValue> {
    if value.is_null() || value.is_undefined() {
        return Ok(Vec::new());
    }

    serde_wasm_bindgen::from_value(value).map_err(|error| {
        JsValue::from_str(&format!(
            "sourceDocuments must be an array of {{sourcePath, sourceSource}} objects: {error}"
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

fn parse_external_sifs_value(value: JsValue) -> Result<Vec<OmenaWasmExternalSifInputV0>, JsValue> {
    if value.is_null() || value.is_undefined() {
        return Ok(Vec::new());
    }

    serde_wasm_bindgen::from_value(value).map_err(|error| {
        JsValue::from_str(&format!(
            "externalSifs must be an array of {{canonicalUrl, sif}} objects: {error}"
        ))
    })
}

fn parse_external_module_mode(
    external_mode: Option<&str>,
) -> Result<OmenaWasmExternalModuleModeV0, JsValue> {
    match external_mode {
        None => Ok(OmenaWasmExternalModuleModeV0::Ignored),
        Some("ignored") => Ok(OmenaWasmExternalModuleModeV0::Ignored),
        Some("sif") => Ok(OmenaWasmExternalModuleModeV0::Sif),
        Some(other) => Err(JsValue::from_str(&format!(
            "unsupported external mode '{other}'; expected ignored or sif"
        ))),
    }
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

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SourceImportedStyleBindingInputV0 {
    binding: String,
    style_uri: String,
}

fn parse_source_imported_style_bindings_value(
    value: JsValue,
) -> Result<Vec<OmenaWasmSourceImportedStyleBindingV0>, JsValue> {
    if value.is_null() || value.is_undefined() {
        return Ok(Vec::new());
    }
    let inputs = serde_wasm_bindgen::from_value::<Vec<SourceImportedStyleBindingInputV0>>(value)
        .map_err(|error| {
            JsValue::from_str(&format!(
                "source imported style bindings must be an array of {{binding, styleUri}} objects: {error}"
            ))
        })?;
    Ok(inputs
        .into_iter()
        .map(|input| OmenaWasmSourceImportedStyleBindingV0 {
            binding: input.binding,
            style_uri: input.style_uri,
        })
        .collect())
}

fn parse_classnames_bind_bindings_value(value: JsValue) -> Result<Vec<String>, JsValue> {
    if value.is_null() || value.is_undefined() {
        return Ok(Vec::new());
    }
    parse_string_array_value(value, "classnamesBindBindings")
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

fn to_json_compatible_js_value<T: Serialize>(value: &T) -> Result<JsValue, JsValue> {
    value
        .serialize(&serde_wasm_bindgen::Serializer::json_compatible())
        .map_err(|error| JsValue::from_str(&format!("failed to serialize result: {error}")))
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

    #[cfg(target_arch = "wasm32")]
    #[test]
    fn accepts_absent_js_values_for_optional_browser_inputs() {
        assert!(parse_pass_ids_value(JsValue::NULL).unwrap().is_empty());
        assert_eq!(
            parse_target_options_value(JsValue::NULL).unwrap(),
            conservative_omena_query_target_options()
        );
        assert_eq!(
            parse_context_value(JsValue::NULL).unwrap(),
            OmenaWasmTransformExecutionContextV0::default()
        );
        assert!(
            parse_source_documents_value(JsValue::NULL)
                .unwrap()
                .is_empty()
        );
        assert!(
            parse_package_manifests_value(JsValue::NULL)
                .unwrap()
                .is_empty()
        );
        let empty_input = parse_optional_engine_input_value(JsValue::NULL).unwrap();
        assert_eq!(empty_input.version, "2");
        assert!(empty_input.sources.is_empty());
        assert!(empty_input.styles.is_empty());
        assert!(empty_input.type_facts.is_empty());
    }

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
    fn parses_stylesheet_tree_for_browser_clients_through_query() {
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
    }

    #[test]
    fn summarizes_bundle_plan_for_browser_clients() {
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
    fn reads_cascade_lfp_for_browser_clients() -> Result<(), String> {
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
    fn reads_style_context_index_for_browser_clients() -> Result<(), String> {
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
    fn reads_style_diagnostics_for_browser_clients() -> Result<(), String> {
        let summary = read_style_diagnostics_summary(
            ":root { --known: #2563eb; }\n.button { color: var(--missing); animation: fade 1s; }\n",
            "fixture.module.css",
        )
        .map_err(|_| "style diagnostics should be available".to_string())?;

        assert_eq!(summary.schema_version, "0");
        assert_eq!(summary.product, "omena-query.diagnostics-for-file");
        assert_eq!(summary.file_uri, "fixture.module.css");
        assert_eq!(summary.file_kind, "style");
        assert_eq!(summary.diagnostic_count, summary.diagnostics.len());
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
        assert!(
            summary
                .diagnostics
                .iter()
                .all(|diagnostic| !diagnostic.provenance.is_empty())
        );
        Ok(())
    }

    #[test]
    fn reads_workspace_style_diagnostics_for_browser_clients() -> Result<(), String> {
        let sources = vec![
            OmenaWasmStyleSourceInputV0 {
                style_path: "/workspace/src/App.module.css".to_string(),
                style_source: r#".button { composes: missing from "./Base.module.css"; }
@value absent from "./Tokens.module.css";"#
                    .to_string(),
            },
            OmenaWasmStyleSourceInputV0 {
                style_path: "/workspace/src/Base.module.css".to_string(),
                style_source: ".base { color: blue; }".to_string(),
            },
            OmenaWasmStyleSourceInputV0 {
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
    fn carries_external_sif_mode_in_workspace_diagnostics_for_browser_clients() -> Result<(), String>
    {
        let sources = vec![OmenaWasmStyleSourceInputV0 {
            style_path: "/tmp/App.module.scss".to_string(),
            style_source: r#"@use "https://cdn.example/tokens.scss" as remote;
.button { color: remote.$brand; }"#
                .to_string(),
        }];

        // Default (external_mode = None) is byte-for-byte the Ignored surface: no SIF boundary.
        let ignored = read_workspace_style_diagnostics_summary(
            "/tmp/App.module.scss",
            &sources,
            &[],
            &[],
            &[],
            None,
        )
        .map_err(|_| "ignored workspace diagnostics should be available".to_string())?;
        assert!(
            ignored
                .diagnostics
                .iter()
                .all(|diagnostic| diagnostic.code != "missingExternalSif")
        );

        // external_mode = "sif" with empty external SIFs surfaces the missing-boundary diagnostic.
        let sif = read_workspace_style_diagnostics_summary(
            "/tmp/App.module.scss",
            &sources,
            &[],
            &[],
            &[],
            Some("sif"),
        )
        .map_err(|_| "sif workspace diagnostics should be available".to_string())?;
        assert!(
            sif.ready_surfaces
                .contains(&"externalSifBoundaryDiagnostics")
        );
        assert!(
            sif.diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "missingExternalSif")
        );
        Ok(())
    }

    #[test]
    fn reads_style_hover_and_completion_for_browser_clients() -> Result<(), String> {
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

        assert_eq!(summary.schema_version, "0");
        assert_eq!(summary.product, "omena-query.diagnostics-for-file");
        assert_eq!(summary.file_uri, "file:///workspace/src/App.tsx");
        assert_eq!(summary.file_kind, "source");
        assert_eq!(summary.diagnostic_count, 1);
        assert_eq!(summary.diagnostic_count, summary.diagnostics.len());
        assert_eq!(summary.diagnostics[0].code, "missingSelector");
        assert!(!summary.diagnostics[0].provenance.is_empty());
        assert!(summary.ready_surfaces.contains(&"crossLanguageDiagnostics"));
    }

    #[test]
    fn reads_source_frontend_indexes_for_browser_clients() {
        let imported_style_bindings = vec![OmenaWasmSourceImportedStyleBindingV0 {
            binding: "styles".to_string(),
            style_uri: "file:///workspace/src/Card.module.scss".to_string(),
        }];
        let classnames_bind_bindings = vec!["cn".to_string()];
        let syntax = read_source_syntax_index_summary(
            "/workspace/src/Card.tsx",
            source_frontend_fixture(),
            Some("typescriptreact"),
            imported_style_bindings.clone(),
            classnames_bind_bindings.clone(),
        );
        let binding = read_source_binding_index_summary(
            "/workspace/src/Card.tsx",
            source_frontend_fixture(),
            Some("typescriptreact"),
            imported_style_bindings,
            classnames_bind_bindings,
        );

        assert_eq!(syntax.product, "omena-bridge.source-syntax-index");
        assert!(!syntax.style_property_accesses.is_empty());
        assert!(syntax.selector_references.iter().any(|reference| {
            reference.target_style_uri.as_deref() == Some("file:///workspace/src/Card.module.scss")
        }));
        assert_eq!(binding.product, "omena-bridge.source-binding-index");
        assert!(
            binding
                .classnames_bind_utility_bindings
                .iter()
                .any(|utility| utility.local_name == "cx" && utility.styles_local_name == "styles")
        );
        assert!(
            binding
                .symbol_ref_uses_decls
                .iter()
                .any(|reference| reference.root_name == "size")
        );
    }

    #[test]
    fn reads_source_type_fact_cfg_for_browser_clients() -> Result<(), String> {
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
        let Some(reference) = source.rfind("size") else {
            return Err("fixture contains size reference".to_string());
        };
        let Some(graph) = read_source_type_fact_control_flow_graph_summary(
            "/workspace/src/Card.tsx",
            source.as_str(),
            Some("typescriptreact"),
            "size",
            reference,
        ) else {
            return Err("fixture should produce a source TypeFact CFG".to_string());
        };

        assert_eq!(graph.entry_block_id, "entry");
        assert!(graph.blocks.iter().any(|block| block.kind == "branch"));
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
    fn reads_workspace_source_diagnostics_for_browser_clients() {
        let sources = vec![OmenaWasmStyleSourceInputV0 {
            style_path: "/workspace/src/App.module.scss".to_string(),
            style_source: ".chip {}".to_string(),
        }];
        let summary = read_workspace_source_diagnostics_summary(
            "/workspace/src/App.tsx",
            r#"import bind from "classnames/bind";
import styles from "./App.module.scss";
const cx = bind.bind(styles);
const variant = Math.random() > 0.5 ? "chip" : "ghost";
export function App() {
  return <div className={cx(variant)} />;
}
"#,
            &sources,
            &[],
        );

        assert_eq!(summary.product, "omena-query.diagnostics-for-file");
        assert_eq!(summary.file_kind, "source");
        assert!(
            summary
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "missingResolvedClassValues")
        );
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
        assert_eq!(summary.requested_pass_ids, pass_ids);
        assert_eq!(summary.effective_pass_ids, pass_ids);
        assert!(summary.unknown_pass_ids.is_empty());
        assert!(summary.execution.output_css.contains("#fff"));
    }

    #[test]
    fn strict_build_surfaces_typed_refusal() {
        let pass_ids = vec!["rule-merging".to_string()];
        let summary = build_style_source_with_verification_summary(
            ".card { color: red; }",
            "fixture.css",
            &pass_ids,
            true,
        );

        assert_eq!(
            summary.execution.strict_policy.profile_id.as_deref(),
            Some("strict-verification")
        );
        assert_eq!(summary.execution.strict_policy.refused_count, 1);
        assert_eq!(summary.execution.strict_policy.rolled_back_count, 0);
        assert_eq!(
            summary.execution.strict_policy.refusal_reasons[0].pass_id,
            "rule-merging"
        );
        assert_eq!(summary.execution.output_css, ".card { color: red; }");
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
    fn builds_css_from_evaluator_context_for_browser_clients() {
        let context = OmenaWasmTransformExecutionContextV0 {
            scss_module_evaluation: Some(omena_query::OmenaQueryTransformModuleEvaluationV0 {
                evaluator: "dart-sass-compatible".to_string(),
                product_output_source: Some("nativeEditOutput".to_string()),
                evaluated_css: ".card { color: red; }".to_string(),
                native_edit_output: Some(".card { color: red; }".to_string()),
                native_replacements: Vec::new(),
                native_edits: Vec::new(),
                oracle: None,
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
        assert!(summary.ready_surfaces.contains(&"sourceMapV3Serializer"));
        assert!(summary.source_map_v3.is_some());
        assert!(!summary.execution.output_css.contains("@import"));
        assert!(!summary.execution.output_css.contains("composes:"));
    }

    #[test]
    fn bundles_workspace_sources_for_browser_clients() {
        let sources = vec![
            OmenaWasmStyleSourceInputV0 {
                style_path: "Button.module.css".to_string(),
                style_source: r#"@import "./tokens.css"; .button { color: var(--brand); }"#
                    .to_string(),
            },
            OmenaWasmStyleSourceInputV0 {
                style_path: "tokens.css".to_string(),
                style_source: ":root { --brand: red; }".to_string(),
            },
        ];
        let pass_ids = vec!["import-inline".to_string(), "print-css".to_string()];
        let artifact_result = bundle_style_sources_with_context_summary(
            "Button.module.css",
            &sources,
            &pass_ids,
            &OmenaWasmTransformExecutionContextV0::default(),
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
        let typed_result = bundle_style_sources_with_context_evidence(
            "Button.module.css",
            &sources,
            &pass_ids,
            &OmenaWasmTransformExecutionContextV0::default(),
            &[],
            &["Button.module.css".to_string()],
        );
        assert!(typed_result.is_ok());
        let Ok(typed_result) = typed_result else {
            return;
        };
        assert!(matches!(
            typed_result.closed_world_outcome,
            omena_query::OmenaQueryClosedWorldOutcomeV0::Closed { .. }
        ));
        assert_eq!(typed_result.evidence.outcome_status, "closed");
        assert!(
            typed_result
                .evidence
                .gates
                .iter()
                .any(|gate| gate.name == "closedWorldAdmission" && gate.passed)
        );
    }

    #[test]
    fn browser_minify_preset_matches_the_semantic_profile_authority() {
        let authority_pass_ids = semantic_omena_query_minify_build_profile()
            .pass_ids
            .into_iter()
            .map(str::to_string)
            .collect::<Vec<_>>();

        assert_eq!(minify_pass_ids(), authority_pass_ids);
    }

    #[test]
    fn builds_minified_workspace_sources_for_browser_clients() {
        let sources = vec![OmenaWasmStyleSourceInputV0 {
            style_path: "Card.module.css".to_string(),
            style_source:
                ".card { color: #ffffff; margin-top: 1px; margin-right: 2px; margin-bottom: 3px; margin-left: 4px; } .empty {}"
                    .to_string(),
        }];
        let summary_result = build_style_sources_with_context_summary(
            "Card.module.css",
            &sources,
            &minify_pass_ids(),
            &OmenaWasmTransformExecutionContextV0::default(),
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
    fn blocks_heuristic_engine_input_tree_shaking_for_browser_clients() {
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
        assert!(summary.execution.output_css.contains(".card-active"));
        assert_eq!(summary.semantic_removal_count, 0);
        assert!(summary.execution.semantic_removals.is_empty());
        assert!(
            summary
                .execution
                .planned_only_pass_ids
                .contains(&"tree-shake-class")
        );
        let decision = serde_json::to_value(&summary.execution.decisions[0]);
        assert!(decision.is_ok());
        let Ok(decision) = decision else {
            return;
        };
        assert_eq!(decision["kind"], "blocked");
        assert_eq!(decision["reason"]["kind"], "precisionBelowFloor");
        assert_eq!(decision["reason"]["required"], "conservative");
        assert_eq!(decision["reason"]["observed"], "heuristic");
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
        assert_eq!(summary.selected_projection_count, 2);
        assert!(
            summary
                .reachability_sources
                .iter()
                .all(|source| source.node_id != "file-merge")
        );
        assert!(
            summary
                .reachability_sources
                .iter()
                .any(|source| source.node_id == "expr-primary"
                    && source.selector_names == vec!["btn-primary--active".to_string()])
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
        assert_eq!(second.reused_graph_count, 2);
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
