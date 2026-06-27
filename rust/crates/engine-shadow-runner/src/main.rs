use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::io::{self, BufRead, Read, Write};

use engine_input_producers::{
    ConstraintDetailCounts, EngineInputV2,
    engine_contract_v2_idl_generated::{
        CertaintyShapeKindV2Json, CheckerReportJsonV1Json, EngineOutputV2Json as EngineOutputV2,
        QueryResultV2Json as QueryResultV2, StringConstraintKindV2Json, ValueDomainKindV2Json,
    },
    summarize_expression_domain_candidates_input,
    summarize_expression_domain_canonical_candidate_bundle_input,
    summarize_expression_domain_canonical_producer_signal_input,
    summarize_expression_domain_evaluator_candidates_input,
    summarize_expression_domain_fragments_input, summarize_expression_domain_plan_input,
    summarize_expression_semantics_candidates_input,
    summarize_expression_semantics_canonical_candidate_bundle_input,
    summarize_expression_semantics_evaluator_candidates_input,
    summarize_expression_semantics_fragments_input,
    summarize_expression_semantics_match_fragments_input, summarize_query_plan_input,
    summarize_selector_usage_candidates_input,
    summarize_selector_usage_canonical_candidate_bundle_input,
    summarize_selector_usage_evaluator_candidates_input, summarize_selector_usage_fragments_input,
    summarize_selector_usage_plan_input, summarize_semantic_canonical_candidate_bundle_input,
    summarize_semantic_canonical_producer_signal_input,
    summarize_semantic_evaluator_candidates_input, summarize_source_resolution_candidates_input,
    summarize_source_resolution_canonical_candidate_bundle_input,
    summarize_source_resolution_evaluator_candidates_input,
    summarize_source_resolution_fragments_input, summarize_source_resolution_match_fragments_input,
    summarize_source_resolution_plan_input, summarize_source_side_canonical_candidate_bundle_input,
    summarize_source_side_canonical_producer_signal_input,
    summarize_source_side_evaluator_candidates_input, summarize_type_fact_input,
};
use omena_abstract_value::{
    AbstractClassValueV0, ClassValueFlowGraphV0, ClassValueFlowNodeV0, ClassValueFlowTransferV0,
    CompositeClassValueInputV0, ExternalStringTypeFactsV0, KLimitedCallSiteFlowInputV0,
    abstract_class_value_kind, analyze_k_limited_call_site_flows, bottom_class_value,
    char_inclusion_class_value, composite_class_value, exact_class_value, finite_set_class_value,
    prefix_class_value, prefix_suffix_class_value, suffix_class_value, top_class_value,
};
use omena_cascade::{
    CascadeDeclaration, CascadeKey, CascadeLevel, CascadeOutcome, CascadeProof, CascadeValue,
    LayerRank, ModuleRank, Specificity,
};
use omena_checker::{
    OmenaCheckerCascadeEvaluationV0, OmenaCheckerCascadeInputV0,
    OmenaCheckerCategoricalEvaluationV0, OmenaCheckerCategoricalInputV0,
    OmenaCheckerCategoricalPrimitiveRolePairInputV0, OmenaCheckerCategoricalRoleMappingInputV0,
    OmenaCheckerDynamicClassDomainInputV0, OmenaCheckerGrnEvaluationV0, OmenaCheckerGrnInputV0,
    OmenaCheckerMTierEvaluationV0, OmenaCheckerMdlEvaluationV0, OmenaCheckerMdlInputV0,
    OmenaCheckerMdlSummaryInputV0, OmenaCheckerReplicaEnsembleEvaluationV0,
    OmenaCheckerReplicaEnsembleInputV0, OmenaCheckerReplicaEnsembleReportInputV0,
    OmenaCheckerRgFlowCouplingInputV0, OmenaCheckerRgFlowCouplingSpaceInputV0,
    OmenaCheckerRgFlowEvaluationV0, OmenaCheckerRgFlowInputV0, OmenaCheckerSmtEvaluationV0,
    OmenaCheckerSmtInputV0, OmenaCheckerStreamingIfdsEvaluationV0,
    OmenaCheckerStreamingIfdsInputV0, OmenaCheckerStreamingIfdsReportInputV0,
    evaluate_omena_checker_cascade_rules, evaluate_omena_checker_categorical_rules,
    evaluate_omena_checker_grn_rules, evaluate_omena_checker_m_tier_rules,
    evaluate_omena_checker_mdl_rules, evaluate_omena_checker_replica_ensemble_rules,
    evaluate_omena_checker_rg_flow_rules, evaluate_omena_checker_smt_rules,
    evaluate_omena_checker_streaming_ifds_rules,
    summarize_omena_checker_rule_enforcement_coverage_v0,
};
use omena_ensemble::{
    ModuleGraphEdgeV0, ModuleGraphV0, OutcomeMode, REPLICA_ENSEMBLE_FEATURE_GATE_V0,
    REPLICA_ENSEMBLE_LAYER_MARKER_V0, REPLICA_ENSEMBLE_SCHEMA_VERSION_V0, ReplicaSiteOutcomeV0,
    ReplicaSnapshotV0, ReportOptionsV0, ReportRecommendation,
    build_cross_file_inconsistency_report, site,
};
use omena_query::{
    OmenaParserStyleDialect, OmenaQueryCodeActionPlanV0, OmenaQueryCrossFileSummaryV0,
    OmenaQueryExpressionDomainFlowRuntimeV0, OmenaQueryExternalModuleModeV0,
    OmenaQueryExternalSifInputV0, OmenaQuerySourceDocumentInputV0, OmenaQueryStyleMemoHostV0,
    OmenaQueryStylePackageManifestV0, OmenaQueryStyleResolutionInputsV0,
    OmenaQueryStyleSourceInputV0, OmenaQueryTargetFeatureSupportV0,
    OmenaQueryTargetTransformOptionsV0, OmenaQueryTransformExecutionContextV0, ParserPositionV0,
    UnifiedHypergraphEdgeKindV0, UnifiedHypergraphHyperedgeV0,
    default_omena_query_transform_print_options,
    execute_omena_query_consumer_build_style_sources_for_target_query_with_context_and_options,
    execute_omena_query_consumer_build_style_sources_with_context,
    execute_omena_query_transform_passes_from_source_with_context,
    list_omena_query_transform_pass_summaries, read_omena_query_cascade_at_position,
    read_omena_query_style_context_index, summarize_omena_query_boundary,
    summarize_omena_query_consumer_check_style_source,
    summarize_omena_query_design_system_minimum_description,
    summarize_omena_query_evaluation_runtime,
    summarize_omena_query_expression_domain_call_site_flow_analysis,
    summarize_omena_query_expression_domain_control_flow_analysis,
    summarize_omena_query_expression_domain_flow_analysis,
    summarize_omena_query_expression_domain_incremental_flow_analysis,
    summarize_omena_query_expression_domain_provenance_explanations,
    summarize_omena_query_expression_domain_reduced_product_iteration,
    summarize_omena_query_expression_domain_selector_projection,
    summarize_omena_query_expression_semantics_canonical_producer_signal,
    summarize_omena_query_expression_semantics_query_fragments,
    summarize_omena_query_native_css_evaluator_from_engine_input,
    summarize_omena_query_omena_parser_css_modules_intermediate,
    summarize_omena_query_omena_parser_lex, summarize_omena_query_omena_parser_style_facts,
    summarize_omena_query_refs_for_workspace_class,
    summarize_omena_query_rename_plan_for_workspace_class,
    summarize_omena_query_scss_evaluator_control_flow_from_engine_input,
    summarize_omena_query_scss_evaluator_control_flow_oracle_corpus,
    summarize_omena_query_selected_query_adapter_capabilities,
    summarize_omena_query_selector_usage_canonical_producer_signal,
    summarize_omena_query_selector_usage_query_fragments,
    summarize_omena_query_source_completion_for_workspace_file,
    summarize_omena_query_source_diagnostics_for_workspace_file,
    summarize_omena_query_source_resolution_canonical_producer_signal,
    summarize_omena_query_source_resolution_query_fragments,
    summarize_omena_query_source_resolution_runtime,
    summarize_omena_query_static_lif_exports_from_engine_input,
    summarize_omena_query_static_stylesheet_evaluator_from_engine_input,
    summarize_omena_query_static_stylesheet_evaluator_oracle_corpus,
    summarize_omena_query_style_completion_for_workspace_file_with_resolution_inputs,
    summarize_omena_query_style_extract_code_actions,
    summarize_omena_query_style_inline_code_actions,
    summarize_omena_query_style_insight_code_actions,
    summarize_omena_query_style_semantic_graph_from_source,
    summarize_omena_query_transform_context_from_engine_input,
    summarize_omena_query_transform_context_from_sources,
    summarize_omena_query_transform_plan_from_source_with_context,
    summarize_omena_query_transform_plan_from_target_query_with_context,
};
use omena_resolver::{
    OmenaResolverBundlerPathAliasMappingV0, OmenaResolverModuleGraphSummaryV0,
    OmenaResolverStylePackageManifestV0, OmenaResolverTsconfigPathMappingV0,
    summarize_omena_resolver_boundary, summarize_omena_resolver_module_graph_index,
    summarize_omena_resolver_runtime_query_boundary,
    summarize_omena_resolver_specifier_resolution_runtime_with_path_mappings,
    summarize_omena_resolver_style_module_resolution_with_path_mappings,
};
use omena_streaming_ifds::{
    PolylogDynamicConnectivityBackendV0, run_streaming_ifds_exact_v0,
    streaming_ifds_event_input_v0, streaming_ifds_summary_cache_entry_v0,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ShadowPayloadV0 {
    input: EngineInputV2,
    output: EngineOutputV2,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct StyleSemanticGraphInputV0 {
    style_path: String,
    style_source: String,
    engine_input: EngineInputV2,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ReadCascadeAtPositionInputV0 {
    style_path: String,
    style_source: String,
    engine_input: EngineInputV2,
    position: ParserPositionInputV0,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ReadStyleContextIndexInputV0 {
    style_path: String,
    style_source: String,
    engine_input: EngineInputV2,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TargetStyleEngineInputV0 {
    target_style_path: String,
    engine_input: EngineInputV2,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct StyleDiagnosticsForFileInputV0 {
    target_style_path: String,
    styles: Vec<OmenaQueryStyleSourceInputV0>,
    #[serde(default)]
    source_documents: Vec<OmenaQuerySourceDocumentInputV0>,
    #[serde(default)]
    package_manifests: Vec<OmenaQueryStylePackageManifestV0>,
    #[serde(default)]
    classname_transform: Option<String>,
    #[serde(default)]
    external_sifs: Vec<OmenaQueryExternalSifInputV0>,
    #[serde(default)]
    external_mode: Option<String>,
}

fn style_diagnostics_external_mode_from_wire(
    external_mode: Option<&str>,
) -> OmenaQueryExternalModuleModeV0 {
    match external_mode {
        Some("sif") => OmenaQueryExternalModuleModeV0::Sif,
        _ => OmenaQueryExternalModuleModeV0::Ignored,
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SourceDiagnosticsForFileInputV0 {
    source_path: String,
    source_source: String,
    styles: Vec<OmenaQueryStyleSourceInputV0>,
    #[serde(default)]
    package_manifests: Vec<OmenaQueryStylePackageManifestV0>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CompletionAtInputV0 {
    file_uri: String,
    file_kind: CompletionFileKindV0,
    position: ParserPositionInputV0,
    #[serde(default)]
    style_source: Option<String>,
    #[serde(default)]
    styles: Vec<OmenaQueryStyleSourceInputV0>,
    #[serde(default)]
    target_style_uri: Option<String>,
    #[serde(default)]
    value_prefix: Option<String>,
    #[serde(default)]
    preferred_selector_names: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct StyleCodeActionsInputV0 {
    style_uri: String,
    style_source: String,
    range: ParserRangeInputV0,
    #[serde(default)]
    styles: Vec<OmenaQueryStyleSourceInputV0>,
    #[serde(default)]
    package_manifests: Vec<OmenaQueryStylePackageManifestV0>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
enum CompletionFileKindV0 {
    Style,
    Source,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RefsForClassInputV0 {
    selector_name: String,
    #[serde(default)]
    target_style_uri: Option<String>,
    include_declaration: bool,
    styles: Vec<OmenaQueryStyleSourceInputV0>,
    #[serde(default)]
    source_documents: Vec<OmenaQuerySourceDocumentInputV0>,
    #[serde(default)]
    package_manifests: Vec<OmenaQueryStylePackageManifestV0>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RenamePlanInputV0 {
    selector_name: String,
    new_name: String,
    #[serde(default)]
    target_style_uri: Option<String>,
    styles: Vec<OmenaQueryStyleSourceInputV0>,
    #[serde(default)]
    source_documents: Vec<OmenaQuerySourceDocumentInputV0>,
    #[serde(default)]
    package_manifests: Vec<OmenaQueryStylePackageManifestV0>,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ParserPositionInputV0 {
    line: usize,
    character: usize,
}

impl From<ParserPositionInputV0> for ParserPositionV0 {
    fn from(position: ParserPositionInputV0) -> Self {
        Self {
            line: position.line,
            character: position.character,
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ParserRangeInputV0 {
    start: ParserPositionInputV0,
    end: ParserPositionInputV0,
}

impl From<ParserRangeInputV0> for omena_query::ParserRangeV0 {
    fn from(range: ParserRangeInputV0) -> Self {
        Self {
            start: range.start.into(),
            end: range.end.into(),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct StyleSemanticGraphBatchStyleInputV0 {
    style_path: String,
    style_source: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct StyleSemanticGraphPackageManifestInputV0 {
    package_json_path: String,
    package_json_source: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct StyleSemanticGraphBatchInputV0 {
    styles: Vec<StyleSemanticGraphBatchStyleInputV0>,
    #[serde(default)]
    package_manifests: Vec<StyleSemanticGraphPackageManifestInputV0>,
    engine_input: EngineInputV2,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WorkspaceCrossFileSummaryInputV0 {
    styles: Vec<OmenaQueryStyleSourceInputV0>,
    #[serde(default)]
    source_documents: Vec<OmenaQuerySourceDocumentInputV0>,
    #[serde(default)]
    package_manifests: Vec<OmenaQueryStylePackageManifestV0>,
}

fn summarize_workspace_cross_file_summary_from_committed_selector(
    input: &WorkspaceCrossFileSummaryInputV0,
) -> Result<OmenaQueryCrossFileSummaryV0, Box<dyn std::error::Error>> {
    let mut host = OmenaQueryStyleMemoHostV0::new();
    let external_sifs = Vec::<OmenaQueryExternalSifInputV0>::new();
    let resolution_inputs = OmenaQueryStyleResolutionInputsV0::default();
    let Some(selector) = host.workspace_revision_selector(
        input.styles.as_slice(),
        input.source_documents.as_slice(),
        input.package_manifests.as_slice(),
        external_sifs.as_slice(),
        &resolution_inputs,
    ) else {
        return Err("failed to commit workspace cross-file summary".into());
    };

    Ok(selector.workspace_cross_file_summary().clone())
}

fn summarize_omena_query_style_semantic_graph_batch_from_sources_with_committed_selector(
    input: &StyleSemanticGraphBatchInputV0,
) -> Result<omena_query::OmenaQueryStyleSemanticGraphBatchOutputV0, Box<dyn std::error::Error>> {
    let styles = input
        .styles
        .iter()
        .map(|style| OmenaQueryStyleSourceInputV0 {
            style_path: style.style_path.clone(),
            style_source: style.style_source.clone(),
        })
        .collect::<Vec<_>>();
    let package_manifests = input
        .package_manifests
        .iter()
        .map(|manifest| OmenaQueryStylePackageManifestV0 {
            package_json_path: manifest.package_json_path.clone(),
            package_json_source: manifest.package_json_source.clone(),
        })
        .collect::<Vec<_>>();
    let mut host = OmenaQueryStyleMemoHostV0::new();
    let external_sifs = Vec::<OmenaQueryExternalSifInputV0>::new();
    let resolution_inputs = OmenaQueryStyleResolutionInputsV0::default();
    let Some(selector) = host.workspace_revision_selector(
        styles.as_slice(),
        &[],
        package_manifests.as_slice(),
        external_sifs.as_slice(),
        &resolution_inputs,
    ) else {
        return Err("failed to commit style semantic graph batch".into());
    };

    Ok(selector.style_semantic_graph_batch(&input.engine_input, package_manifests.as_slice()))
}

fn summarize_style_diagnostics_from_committed_selector(
    input: &StyleDiagnosticsForFileInputV0,
) -> Result<omena_query::OmenaQueryStyleDiagnosticsForFileV0, Box<dyn std::error::Error>> {
    if input.classname_transform.is_some() {
        return Err(
            "committed selector style diagnostics do not support classname transforms".into(),
        );
    }

    let mut host = OmenaQueryStyleMemoHostV0::new();
    let resolution_inputs = OmenaQueryStyleResolutionInputsV0::default();
    let Some(selector) = host.workspace_revision_selector(
        input.styles.as_slice(),
        input.source_documents.as_slice(),
        input.package_manifests.as_slice(),
        input.external_sifs.as_slice(),
        &resolution_inputs,
    ) else {
        return Err("failed to commit workspace style diagnostics".into());
    };

    let external_mode = style_diagnostics_external_mode_from_wire(input.external_mode.as_deref());
    selector
        .workspace_style_diagnostics_with_external_mode(&input.target_style_path, external_mode)
        .ok_or_else(|| "unsupported style module path".into())
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TransformPlanInputV0 {
    style_path: String,
    style_source: String,
    #[serde(default = "default_transform_target_label")]
    target_label: String,
    #[serde(default)]
    target_query: Option<String>,
    #[serde(default)]
    target_support: TransformPlanTargetFeatureSupportInputV0,
    target_options: TransformPlanTargetOptionsInputV0,
    #[serde(default)]
    transform_context: OmenaQueryTransformExecutionContextV0,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TransformContextInputV0 {
    target_style_path: String,
    styles: Vec<StyleSemanticGraphBatchStyleInputV0>,
    #[serde(default)]
    package_manifests: Vec<StyleSemanticGraphPackageManifestInputV0>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TransformExecuteInputV0 {
    style_path: String,
    style_source: String,
    requested_pass_ids: Vec<String>,
    #[serde(default)]
    transform_context: OmenaQueryTransformExecutionContextV0,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TransformContextFromEngineInputV0 {
    engine_input: EngineInputV2,
    target_style_path: String,
    closed_style_world: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ConsumerStyleSourceInputV0 {
    style_path: String,
    style_source: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ConsumerStyleSourceBuildInputV0 {
    style_path: String,
    style_source: String,
    #[serde(default)]
    requested_pass_ids: Vec<String>,
    #[serde(default)]
    target_query: Option<String>,
    #[serde(default)]
    target_options: TransformPlanTargetOptionsInputV0,
    #[serde(default)]
    transform_context: OmenaQueryTransformExecutionContextV0,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ConsumerStyleSourcesBuildInputV0 {
    target_style_path: String,
    styles: Vec<OmenaQueryStyleSourceInputV0>,
    #[serde(default)]
    requested_pass_ids: Vec<String>,
    #[serde(default)]
    target_query: Option<String>,
    #[serde(default)]
    target_options: TransformPlanTargetOptionsInputV0,
    #[serde(default)]
    transform_context: OmenaQueryTransformExecutionContextV0,
    #[serde(default)]
    package_manifests: Vec<OmenaQueryStylePackageManifestV0>,
}

fn default_transform_target_label() -> String {
    "explicit-feature-matrix".to_string()
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TransformPlanTargetFeatureSupportInputV0 {
    vendor_prefix_required: bool,
    supports_light_dark: bool,
    supports_color_mix: bool,
    supports_oklch_oklab: bool,
    supports_color_function: bool,
    // NOTE: defaulted so existing payloads omitting it map to "supported"
    // (no spurious relative-color lowering) and still deserialize.
    #[serde(default = "default_true")]
    supports_relative_color: bool,
    supports_logical_properties: bool,
    supports_css_nesting: bool,
    supports_css_scope: bool,
    supports_cascade_layers: bool,
}

impl Default for TransformPlanTargetFeatureSupportInputV0 {
    fn default() -> Self {
        Self {
            vendor_prefix_required: false,
            supports_light_dark: true,
            supports_color_mix: true,
            supports_oklch_oklab: true,
            supports_color_function: true,
            supports_relative_color: true,
            supports_logical_properties: true,
            supports_css_nesting: true,
            supports_css_scope: true,
            supports_cascade_layers: true,
        }
    }
}

impl From<TransformPlanTargetFeatureSupportInputV0> for OmenaQueryTargetFeatureSupportV0 {
    fn from(input: TransformPlanTargetFeatureSupportInputV0) -> Self {
        Self {
            vendor_prefix_required: input.vendor_prefix_required,
            supports_light_dark: input.supports_light_dark,
            supports_color_mix: input.supports_color_mix,
            supports_oklch_oklab: input.supports_oklch_oklab,
            supports_color_function: input.supports_color_function,
            supports_relative_color: input.supports_relative_color,
            supports_logical_properties: input.supports_logical_properties,
            supports_css_nesting: input.supports_css_nesting,
            supports_css_scope: input.supports_css_scope,
            supports_cascade_layers: input.supports_cascade_layers,
        }
    }
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TransformPlanTargetOptionsInputV0 {
    allow_logical_to_physical: bool,
    allow_scope_flatten: bool,
    allow_layer_flatten: bool,
    enable_supports_static_eval: bool,
    enable_media_static_eval: bool,
    #[serde(default)]
    drop_dark_mode_media_queries: bool,
}

impl From<TransformPlanTargetOptionsInputV0> for OmenaQueryTargetTransformOptionsV0 {
    fn from(input: TransformPlanTargetOptionsInputV0) -> Self {
        Self {
            allow_logical_to_physical: input.allow_logical_to_physical,
            allow_scope_flatten: input.allow_scope_flatten,
            allow_layer_flatten: input.allow_layer_flatten,
            enable_supports_static_eval: input.enable_supports_static_eval,
            enable_media_static_eval: input.enable_media_static_eval,
            enable_container_static_eval: false,
            drop_dark_mode_media_queries: input.drop_dark_mode_media_queries,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct OmenaResolverStyleModuleResolutionInputV0 {
    from_style_path: String,
    source: String,
    #[serde(default)]
    available_style_paths: Vec<String>,
    #[serde(default)]
    package_manifests: Vec<OmenaResolverStylePackageManifestInputV0>,
    #[serde(default)]
    bundler_path_mappings: Vec<OmenaResolverBundlerPathAliasMappingInputV0>,
    #[serde(default)]
    tsconfig_path_mappings: Vec<OmenaResolverTsconfigPathMappingInputV0>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct OmenaResolverStylePackageManifestInputV0 {
    package_json_path: String,
    package_json_source: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct OmenaResolverSpecifierResolutionRuntimeInputV0 {
    from_style_path: String,
    #[serde(default)]
    sources: Vec<String>,
    #[serde(default)]
    available_style_paths: Vec<String>,
    #[serde(default)]
    package_manifests: Vec<OmenaResolverStylePackageManifestInputV0>,
    #[serde(default)]
    bundler_path_mappings: Vec<OmenaResolverBundlerPathAliasMappingInputV0>,
    #[serde(default)]
    tsconfig_path_mappings: Vec<OmenaResolverTsconfigPathMappingInputV0>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct OmenaResolverBundlerPathAliasMappingInputV0 {
    pattern: String,
    target_path: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct OmenaResolverTsconfigPathMappingInputV0 {
    base_path: String,
    pattern: String,
    #[serde(default)]
    target_patterns: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct OmenaParserStyleFactsInputV0 {
    style_source: String,
    dialect: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct OmenaParserCssModulesIntermediateInputV0 {
    style_source: String,
    dialect: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct OmenaParserLexInputV0 {
    style_source: String,
    dialect: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct OmenaCheckerMTierEvaluationInputV0 {
    abstract_value: OmenaCheckerAbstractClassValueInputV0,
    selector_universe: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct OmenaCheckerKLimitedFlowMTierEvaluationInputV0 {
    max_context_depth: usize,
    selector_universe: Vec<String>,
    contexts: Vec<OmenaCheckerKLimitedFlowContextInputV0>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct OmenaCheckerKLimitedFlowContextInputV0 {
    callee_key: String,
    call_site_stack: Vec<String>,
    value: OmenaCheckerAbstractClassValueInputV0,
}

#[derive(Debug, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
enum OmenaCheckerAbstractClassValueInputV0 {
    Bottom,
    Exact {
        value: String,
    },
    FiniteSet {
        values: Vec<String>,
    },
    Prefix {
        prefix: String,
    },
    Suffix {
        suffix: String,
    },
    PrefixSuffix {
        prefix: String,
        suffix: String,
        min_length: Option<usize>,
    },
    CharInclusion {
        #[serde(default)]
        must_chars: String,
        #[serde(default)]
        may_chars: String,
        #[serde(default)]
        may_include_other_chars: bool,
    },
    Composite {
        #[serde(default)]
        prefix: Option<String>,
        #[serde(default)]
        suffix: Option<String>,
        #[serde(default)]
        min_length: Option<usize>,
        #[serde(default)]
        must_chars: String,
        #[serde(default)]
        may_chars: String,
        #[serde(default)]
        may_include_other_chars: bool,
    },
    Top,
}

impl OmenaCheckerAbstractClassValueInputV0 {
    fn into_abstract_class_value(self) -> AbstractClassValueV0 {
        match self {
            Self::Bottom => bottom_class_value(),
            Self::Exact { value } => exact_class_value(value),
            Self::FiniteSet { values } => finite_set_class_value(values),
            Self::Prefix { prefix } => prefix_class_value(prefix, None),
            Self::Suffix { suffix } => suffix_class_value(suffix, None),
            Self::PrefixSuffix {
                prefix,
                suffix,
                min_length,
            } => prefix_suffix_class_value(prefix, suffix, min_length, None),
            Self::CharInclusion {
                must_chars,
                may_chars,
                may_include_other_chars,
            } => char_inclusion_class_value(must_chars, may_chars, None, may_include_other_chars),
            Self::Composite {
                prefix,
                suffix,
                min_length,
                must_chars,
                may_chars,
                may_include_other_chars,
            } => composite_class_value(CompositeClassValueInputV0 {
                prefix,
                suffix,
                min_length,
                must_chars,
                may_chars,
                may_include_other_chars,
                provenance: None,
            }),
            Self::Top => top_class_value(),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct OmenaCheckerMTierEvaluationRunnerOutputV0 {
    schema_version: &'static str,
    product: &'static str,
    selector_universe_count: usize,
    evaluation_count: usize,
    evaluations: Vec<OmenaCheckerMTierEvaluationV0>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct OmenaCheckerKLimitedFlowMTierEvaluationRunnerOutputV0 {
    schema_version: &'static str,
    product: &'static str,
    flow_product: &'static str,
    context_sensitivity: String,
    max_context_depth: usize,
    selector_universe_count: usize,
    context_count: usize,
    evaluation_count: usize,
    contexts: Vec<OmenaCheckerKLimitedFlowMTierContextOutputV0>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct OmenaCheckerKLimitedFlowMTierContextOutputV0 {
    callee_key: String,
    call_site_stack: Vec<String>,
    context_key: String,
    exit_value_kind: &'static str,
    exit_value: AbstractClassValueV0,
    evaluation_count: usize,
    rule_code_names: Vec<&'static str>,
    evaluations: Vec<OmenaCheckerMTierEvaluationV0>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct OmenaCheckerCascadeEvaluationRunnerOutputV0 {
    schema_version: &'static str,
    product: &'static str,
    declaration_count: usize,
    custom_property_count: usize,
    evaluation_count: usize,
    rule_code_names: Vec<&'static str>,
    evaluations: Vec<OmenaCheckerCascadeEvaluationV0>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct OmenaCheckerGrnEvaluationRunnerOutputV0 {
    schema_version: &'static str,
    product: &'static str,
    vertex_count: usize,
    evaluation_count: usize,
    rule_code_names: Vec<&'static str>,
    evaluations: Vec<OmenaCheckerGrnEvaluationV0>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct OmenaCheckerSmtEvaluationRunnerOutputV0 {
    schema_version: &'static str,
    product: &'static str,
    obligation_count: usize,
    evaluation_count: usize,
    rule_code_names: Vec<&'static str>,
    evaluations: Vec<OmenaCheckerSmtEvaluationV0>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct OmenaCheckerMdlEvaluationInputV0 {
    source_uri: String,
    source_hash: String,
    rule_count: usize,
    observation_count: usize,
    /// Empirical value-symbol frequency histogram. The runner computes the real
    /// entropy/log MDL from these model inputs; it does NOT accept total_bits.
    value_frequencies: Vec<usize>,
    budget_bits: f64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct OmenaCheckerMdlEvaluationRunnerOutputV0 {
    schema_version: &'static str,
    product: &'static str,
    source_uri: String,
    total_bits: f64,
    budget_bits: f64,
    evaluation_count: usize,
    rule_code_names: Vec<&'static str>,
    evaluations: Vec<OmenaCheckerMdlEvaluationV0>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct OmenaCheckerStreamingIfdsEvaluationInputV0 {
    update_id: String,
    start_node_id: String,
    hyperedges: Vec<StreamingIfdsHyperedgeInputV0>,
    events: Vec<StreamingIfdsEventRunnerInputV0>,
    /// Prior streaming summary fact keys (`node_id|value-key`) carried from an
    /// earlier revision. When present, the incremental path reuses prior facts
    /// outside the dirty region; a stale key (a fact the current graph no longer
    /// produces) makes the incremental result diverge from the batch oracle.
    #[serde(default)]
    previous_fact_keys: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct StreamingIfdsHyperedgeInputV0 {
    hyperedge_id: String,
    from: String,
    to: String,
    #[serde(default)]
    edge_kind: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct StreamingIfdsEventRunnerInputV0 {
    event_id: String,
    revision: u64,
    node_id: String,
    value: OmenaCheckerAbstractClassValueInputV0,
    #[serde(default)]
    refinement_context_digest: Option<u64>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct OmenaCheckerStreamingIfdsEvaluationRunnerOutputV0 {
    schema_version: &'static str,
    product: &'static str,
    report_product: &'static str,
    event_count: usize,
    output_fact_count: usize,
    precision_parity_with_batch: bool,
    evaluation_count: usize,
    rule_code_names: Vec<&'static str>,
    evaluations: Vec<OmenaCheckerStreamingIfdsEvaluationV0>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct OmenaCheckerRgFlowEvaluationInputV0 {
    flows: Vec<OmenaCheckerRgFlowCouplingRunnerInputV0>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct OmenaCheckerRgFlowCouplingRunnerInputV0 {
    workspace_path: String,
    before: OmenaCheckerRgFlowCouplingSpaceInputV0,
    after: OmenaCheckerRgFlowCouplingSpaceInputV0,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct OmenaCheckerRgFlowEvaluationRunnerOutputV0 {
    schema_version: &'static str,
    product: &'static str,
    flow_count: usize,
    evaluation_count: usize,
    rule_code_names: Vec<&'static str>,
    evaluations: Vec<OmenaCheckerRgFlowEvaluationV0>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct OmenaCheckerReplicaEnsembleEvaluationInputV0 {
    workspace_root: String,
    replicas: Vec<ReplicaEnsembleSnapshotRunnerInputV0>,
    graph_edges: Vec<ReplicaEnsembleGraphEdgeRunnerInputV0>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ReplicaEnsembleSnapshotRunnerInputV0 {
    path: String,
    winners: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ReplicaEnsembleGraphEdgeRunnerInputV0 {
    from_module: String,
    to_module: String,
    #[serde(default)]
    edge_kind: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct OmenaCheckerReplicaEnsembleEvaluationRunnerOutputV0 {
    schema_version: &'static str,
    product: &'static str,
    report_product: &'static str,
    workspace_root: String,
    replica_count: usize,
    pair_count: usize,
    mean_q: f64,
    variance_q: f64,
    recommendation: &'static str,
    top_disagreement_pair_count: usize,
    evaluation_count: usize,
    rule_code_names: Vec<&'static str>,
    evaluations: Vec<OmenaCheckerReplicaEnsembleEvaluationV0>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct OmenaCheckerCategoricalEvaluationInputV0 {
    mappings: Vec<OmenaCheckerCategoricalRoleMappingRunnerInputV0>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct OmenaCheckerCategoricalRoleMappingRunnerInputV0 {
    mapping_id: String,
    primitive_role_pairs: Vec<OmenaCheckerCategoricalPrimitiveRolePairRunnerInputV0>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct OmenaCheckerCategoricalPrimitiveRolePairRunnerInputV0 {
    primitive_name: String,
    categorical_role: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct OmenaCheckerCategoricalEvaluationRunnerOutputV0 {
    schema_version: &'static str,
    product: &'static str,
    mapping_count: usize,
    evaluation_count: usize,
    rule_code_names: Vec<&'static str>,
    evaluations: Vec<OmenaCheckerCategoricalEvaluationV0>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct EngineShadowRunnerDaemonRequestV0 {
    id: serde_json::Value,
    command: String,
    input: serde_json::Value,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct EngineShadowRunnerDaemonResponseV0 {
    schema_version: &'static str,
    id: serde_json::Value,
    ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct CheckerReportSummaryV1 {
    warnings: usize,
    hints: usize,
    total: usize,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CheckerFindingRecordV1 {
    file_path: String,
    category: String,
    code: String,
    severity: String,
    range: RangeV0,
    message: String,
    analysis_reason: Option<String>,
    value_certainty_shape_label: Option<String>,
    value_domain_derivation: Option<ValueDomainDerivationV0>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
struct ValueDomainDerivationV0 {
    schema_version: String,
    product: String,
    input_fact_kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    input_constraint_kind: Option<String>,
    input_value_count: usize,
    reduced_kind: String,
    steps: Vec<ValueDomainDerivationStepV0>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
struct ValueDomainDerivationStepV0 {
    operation: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    input_kind: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    refinement_kind: Option<String>,
    result_kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    result_provenance: Option<String>,
    reason: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
struct RangeV0 {
    start: PositionV0,
    end: PositionV0,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
struct PositionV0 {
    line: usize,
    character: usize,
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
struct CheckerStyleRecoveryFindingV0 {
    file_path: String,
    code: String,
    severity: String,
    range: RangeV0,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    analysis_reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    value_certainty_shape_label: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CheckerStyleRecoveryCanonicalCandidateBundleV0 {
    schema_version: &'static str,
    input_version: String,
    report_version: String,
    bundle: &'static str,
    distinct_file_count: usize,
    code_counts: BTreeMap<String, usize>,
    summary: CheckerReportSummaryV1,
    findings: Vec<CheckerStyleRecoveryFindingV0>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CheckerStyleRecoveryCanonicalProducerGateV0 {
    canonical_candidate_command: &'static str,
    canonical_producer_command: &'static str,
    consumer_boundary_command: &'static str,
    bounded_checker_lane_command: &'static str,
    promotion_review_command: &'static str,
    promotion_evidence_command: &'static str,
    broader_rust_lane_command: &'static str,
    release_gate_readiness_command: &'static str,
    release_gate_shadow_command: &'static str,
    release_gate_shadow_review_command: &'static str,
    release_bundle_command: &'static str,
    minimum_bounded_lane_count_for_rust_lane_bundle: usize,
    minimum_bounded_lane_count_for_rust_release_bundle: usize,
    minimum_successful_shadow_runs_for_rust_release_bundle: usize,
    checker_bundle: &'static str,
    release_gate_stage: &'static str,
    included_in_rust_lane_bundle: bool,
    included_in_rust_release_bundle: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CheckerStyleRecoveryCanonicalProducerSignalV0 {
    schema_version: &'static str,
    input_version: String,
    canonical_candidate: CheckerStyleRecoveryCanonicalCandidateBundleV0,
    bounded_checker_gate: CheckerStyleRecoveryCanonicalProducerGateV0,
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
struct CheckerSourceMissingFindingV0 {
    file_path: String,
    code: String,
    severity: String,
    range: RangeV0,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    analysis_reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    value_certainty_shape_label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    value_domain_derivation: Option<ValueDomainDerivationV0>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CheckerSourceMissingCanonicalCandidateBundleV0 {
    schema_version: &'static str,
    input_version: String,
    report_version: String,
    bundle: &'static str,
    distinct_file_count: usize,
    code_counts: BTreeMap<String, usize>,
    summary: CheckerReportSummaryV1,
    findings: Vec<CheckerSourceMissingFindingV0>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CheckerSourceMissingCanonicalProducerGateV0 {
    canonical_candidate_command: &'static str,
    canonical_producer_command: &'static str,
    consumer_boundary_command: &'static str,
    bounded_checker_lane_command: &'static str,
    promotion_review_command: &'static str,
    promotion_evidence_command: &'static str,
    broader_rust_lane_command: &'static str,
    release_gate_readiness_command: &'static str,
    release_gate_shadow_command: &'static str,
    release_gate_shadow_review_command: &'static str,
    release_bundle_command: &'static str,
    minimum_bounded_lane_count_for_rust_lane_bundle: usize,
    minimum_bounded_lane_count_for_rust_release_bundle: usize,
    minimum_successful_shadow_runs_for_rust_release_bundle: usize,
    checker_bundle: &'static str,
    release_gate_stage: &'static str,
    included_in_rust_lane_bundle: bool,
    included_in_rust_release_bundle: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CheckerSourceMissingFlowEvidenceV0 {
    schema_version: &'static str,
    product: &'static str,
    input_version: String,
    graph_count: usize,
    node_count: usize,
    converged_graph_count: usize,
    unconverged_graph_count: usize,
    max_iteration_count: usize,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CheckerSourceMissingCanonicalProducerSignalV0 {
    schema_version: &'static str,
    input_version: String,
    canonical_candidate: CheckerSourceMissingCanonicalCandidateBundleV0,
    flow_evidence: CheckerSourceMissingFlowEvidenceV0,
    bounded_checker_gate: CheckerSourceMissingCanonicalProducerGateV0,
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "camelCase")]
struct CheckerStyleUnusedFindingV0 {
    file_path: String,
    code: String,
    severity: String,
    range: RangeV0,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    analysis_reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    value_certainty_shape_label: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CheckerStyleUnusedCanonicalCandidateBundleV0 {
    schema_version: &'static str,
    input_version: String,
    report_version: String,
    bundle: &'static str,
    distinct_file_count: usize,
    code_counts: BTreeMap<String, usize>,
    summary: CheckerReportSummaryV1,
    findings: Vec<CheckerStyleUnusedFindingV0>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CheckerStyleUnusedCanonicalProducerGateV0 {
    canonical_candidate_command: &'static str,
    canonical_producer_command: &'static str,
    consumer_boundary_command: &'static str,
    bounded_checker_lane_command: &'static str,
    promotion_review_command: &'static str,
    promotion_evidence_command: &'static str,
    broader_rust_lane_command: &'static str,
    release_gate_readiness_command: &'static str,
    release_gate_shadow_command: &'static str,
    release_gate_shadow_review_command: &'static str,
    release_bundle_command: &'static str,
    minimum_bounded_lane_count_for_rust_lane_bundle: usize,
    minimum_bounded_lane_count_for_rust_release_bundle: usize,
    minimum_successful_shadow_runs_for_rust_release_bundle: usize,
    checker_bundle: &'static str,
    release_gate_stage: &'static str,
    included_in_rust_lane_bundle: bool,
    included_in_rust_release_bundle: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CheckerStyleUnusedCanonicalProducerSignalV0 {
    schema_version: &'static str,
    input_version: String,
    canonical_candidate: CheckerStyleUnusedCanonicalCandidateBundleV0,
    bounded_checker_gate: CheckerStyleUnusedCanonicalProducerGateV0,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ShadowSummaryV0 {
    schema_version: &'static str,
    input_version: String,
    source_count: usize,
    style_count: usize,
    type_fact_count: usize,
    distinct_fact_files: usize,
    by_kind: BTreeMap<String, usize>,
    constrained_kinds: BTreeMap<String, usize>,
    finite_value_count: usize,
    query_result_count: usize,
    query_kind_counts: BTreeMap<String, usize>,
    expression_value_domain_kinds: BTreeMap<String, usize>,
    expression_value_constraint_kinds: BTreeMap<String, usize>,
    expression_constraint_detail_counts: ConstraintDetailCounts,
    expression_value_certainty_shapes: BTreeMap<String, usize>,
    expression_selector_certainty_shapes: BTreeMap<String, usize>,
    resolution_value_constraint_kinds: BTreeMap<String, usize>,
    resolution_constraint_detail_counts: ConstraintDetailCounts,
    resolution_value_certainty_shapes: BTreeMap<String, usize>,
    resolution_selector_certainty_shapes: BTreeMap<String, usize>,
    selector_usage_referenced_count: usize,
    selector_usage_unreferenced_count: usize,
    selector_usage_total_references: usize,
    selector_usage_direct_references: usize,
    selector_usage_editable_direct_references: usize,
    selector_usage_exact_references: usize,
    selector_usage_inferred_or_better_references: usize,
    selector_usage_expanded_count: usize,
    selector_usage_style_dependency_count: usize,
    expected_expression_semantics_count: usize,
    expected_source_expression_resolution_count: usize,
    expected_selector_usage_count: usize,
    expected_total_query_count: usize,
    matched_expression_query_pairs: usize,
    missing_expression_semantics_count: usize,
    missing_source_expression_resolution_count: usize,
    unexpected_expression_semantics_count: usize,
    unexpected_source_expression_resolution_count: usize,
    matched_selector_usage_count: usize,
    missing_selector_usage_count: usize,
    unexpected_selector_usage_count: usize,
    rewrite_plan_count: usize,
    checker_warning_count: usize,
    checker_hint_count: usize,
    checker_total_findings: usize,
}

fn run_style_code_actions_facade(input: StyleCodeActionsInputV0) -> OmenaQueryCodeActionPlanV0 {
    let mut styles = input.styles;
    if !styles
        .iter()
        .any(|style| style.style_path == input.style_uri)
    {
        styles.push(OmenaQueryStyleSourceInputV0 {
            style_path: input.style_uri.clone(),
            style_source: input.style_source.clone(),
        });
    }
    let range = input.range.into();
    let inline = summarize_omena_query_style_inline_code_actions(
        &input.style_uri,
        &styles,
        range,
        input.package_manifests.as_slice(),
    );
    if inline.action_count > 0 {
        return inline;
    }
    let insight = summarize_omena_query_style_insight_code_actions(
        &input.style_uri,
        &input.style_source,
        range,
    );
    if insight.action_count > 0 {
        return insight;
    }
    summarize_omena_query_style_extract_code_actions(&input.style_uri, &input.style_source, range)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mode = env::args().nth(1);
    if mode.as_deref() == Some("--daemon") {
        return run_daemon();
    }

    let mut stdin = String::new();
    io::stdin().read_to_string(&mut stdin)?;

    match mode.as_deref() {
        None => {
            let payload: ShadowPayloadV0 = serde_json::from_str(&stdin)?;
            let summary = summarize(payload);
            serde_json::to_writer_pretty(io::stdout(), &summary)?;
        }
        Some("input-type-facts") => {
            let input: EngineInputV2 = serde_json::from_str(&stdin)?;
            let summary = summarize_type_fact_input(&input);
            serde_json::to_writer_pretty(io::stdout(), &summary)?;
        }
        Some("input-query-plan") => {
            let input: EngineInputV2 = serde_json::from_str(&stdin)?;
            let summary = summarize_query_plan_input(&input);
            serde_json::to_writer_pretty(io::stdout(), &summary)?;
        }
        Some("input-expression-domains") => {
            let input: EngineInputV2 = serde_json::from_str(&stdin)?;
            let summary = summarize_expression_domain_plan_input(&input);
            serde_json::to_writer_pretty(io::stdout(), &summary)?;
        }
        Some("input-expression-domain-fragments") => {
            let input: EngineInputV2 = serde_json::from_str(&stdin)?;
            let summary = summarize_expression_domain_fragments_input(&input);
            serde_json::to_writer_pretty(io::stdout(), &summary)?;
        }
        Some("input-expression-domain-candidates") => {
            let input: EngineInputV2 = serde_json::from_str(&stdin)?;
            let summary = summarize_expression_domain_candidates_input(&input);
            serde_json::to_writer_pretty(io::stdout(), &summary)?;
        }
        Some("input-expression-domain-canonical-candidate") => {
            let input: EngineInputV2 = serde_json::from_str(&stdin)?;
            let summary = summarize_expression_domain_canonical_candidate_bundle_input(&input);
            serde_json::to_writer_pretty(io::stdout(), &summary)?;
        }
        Some("input-expression-domain-evaluator-candidates") => {
            let input: EngineInputV2 = serde_json::from_str(&stdin)?;
            let summary = summarize_expression_domain_evaluator_candidates_input(&input);
            serde_json::to_writer_pretty(io::stdout(), &summary)?;
        }
        Some("input-expression-domain-flow-analysis") => {
            let input: EngineInputV2 = serde_json::from_str(&stdin)?;
            let summary = summarize_omena_query_expression_domain_flow_analysis(&input);
            serde_json::to_writer_pretty(io::stdout(), &summary)?;
        }
        Some("input-expression-domain-control-flow-analysis") => {
            let input: EngineInputV2 = serde_json::from_str(&stdin)?;
            let summary = summarize_omena_query_expression_domain_control_flow_analysis(&input);
            serde_json::to_writer_pretty(io::stdout(), &summary)?;
        }
        Some("input-expression-domain-call-site-flow-analysis") => {
            let input: EngineInputV2 = serde_json::from_str(&stdin)?;
            let summary = summarize_omena_query_expression_domain_call_site_flow_analysis(&input);
            serde_json::to_writer_pretty(io::stdout(), &summary)?;
        }
        Some("input-expression-domain-provenance-explanations") => {
            let input: EngineInputV2 = serde_json::from_str(&stdin)?;
            let summary = summarize_omena_query_expression_domain_provenance_explanations(&input);
            serde_json::to_writer_pretty(io::stdout(), &summary)?;
        }
        Some("input-expression-domain-reduced-product-iteration") => {
            let input: EngineInputV2 = serde_json::from_str(&stdin)?;
            let summary = summarize_omena_query_expression_domain_reduced_product_iteration(&input);
            serde_json::to_writer_pretty(io::stdout(), &summary)?;
        }
        Some("input-expression-domain-incremental-flow-analysis") => {
            let input: EngineInputV2 = serde_json::from_str(&stdin)?;
            let mut runtime = OmenaQueryExpressionDomainFlowRuntimeV0::default();
            let summary = summarize_omena_query_expression_domain_incremental_flow_analysis(
                &input,
                &mut runtime,
            );
            serde_json::to_writer_pretty(io::stdout(), &summary)?;
        }
        Some("input-expression-domain-selector-projection") => {
            let input: EngineInputV2 = serde_json::from_str(&stdin)?;
            let summary = summarize_omena_query_expression_domain_selector_projection(&input);
            serde_json::to_writer_pretty(io::stdout(), &summary)?;
        }
        Some("input-expression-domain-canonical-producer") => {
            let input: EngineInputV2 = serde_json::from_str(&stdin)?;
            let summary = summarize_expression_domain_canonical_producer_signal_input(&input);
            serde_json::to_writer_pretty(io::stdout(), &summary)?;
        }
        Some("input-selector-usage-plan") => {
            let input: EngineInputV2 = serde_json::from_str(&stdin)?;
            let summary = summarize_selector_usage_plan_input(&input);
            serde_json::to_writer_pretty(io::stdout(), &summary)?;
        }
        Some("input-selector-usage-fragments") => {
            let input: EngineInputV2 = serde_json::from_str(&stdin)?;
            let summary = summarize_selector_usage_fragments_input(&input);
            serde_json::to_writer_pretty(io::stdout(), &summary)?;
        }
        Some("input-selector-usage-query-fragments") => {
            let input: EngineInputV2 = serde_json::from_str(&stdin)?;
            let summary = summarize_omena_query_selector_usage_query_fragments(&input);
            serde_json::to_writer_pretty(io::stdout(), &summary)?;
        }
        Some("input-selector-usage-candidates") => {
            let input: EngineInputV2 = serde_json::from_str(&stdin)?;
            let summary = summarize_selector_usage_candidates_input(&input);
            serde_json::to_writer_pretty(io::stdout(), &summary)?;
        }
        Some("input-selector-usage-evaluator-candidates") => {
            let input: EngineInputV2 = serde_json::from_str(&stdin)?;
            let summary = summarize_selector_usage_evaluator_candidates_input(&input);
            serde_json::to_writer_pretty(io::stdout(), &summary)?;
        }
        Some("input-selector-usage-canonical-candidate") => {
            let input: EngineInputV2 = serde_json::from_str(&stdin)?;
            let summary = summarize_selector_usage_canonical_candidate_bundle_input(&input);
            serde_json::to_writer_pretty(io::stdout(), &summary)?;
        }
        Some("input-selector-usage-canonical-producer") => {
            let input: EngineInputV2 = serde_json::from_str(&stdin)?;
            let summary = summarize_omena_query_selector_usage_canonical_producer_signal(&input);
            serde_json::to_writer_pretty(io::stdout(), &summary)?;
        }
        Some("input-source-resolution-plan") => {
            let input: EngineInputV2 = serde_json::from_str(&stdin)?;
            let summary = summarize_source_resolution_plan_input(&input);
            serde_json::to_writer_pretty(io::stdout(), &summary)?;
        }
        Some("input-expression-semantics-fragments") => {
            let input: EngineInputV2 = serde_json::from_str(&stdin)?;
            let summary = summarize_expression_semantics_fragments_input(&input);
            serde_json::to_writer_pretty(io::stdout(), &summary)?;
        }
        Some("input-expression-semantics-candidates") => {
            let input: EngineInputV2 = serde_json::from_str(&stdin)?;
            let summary = summarize_expression_semantics_candidates_input(&input);
            serde_json::to_writer_pretty(io::stdout(), &summary)?;
        }
        Some("input-expression-semantics-evaluator-candidates") => {
            let input: EngineInputV2 = serde_json::from_str(&stdin)?;
            let summary = summarize_expression_semantics_evaluator_candidates_input(&input);
            serde_json::to_writer_pretty(io::stdout(), &summary)?;
        }
        Some("input-expression-semantics-canonical-candidate") => {
            let input: EngineInputV2 = serde_json::from_str(&stdin)?;
            let summary = summarize_expression_semantics_canonical_candidate_bundle_input(&input);
            serde_json::to_writer_pretty(io::stdout(), &summary)?;
        }
        Some("input-expression-semantics-canonical-producer") => {
            let input: EngineInputV2 = serde_json::from_str(&stdin)?;
            let summary =
                summarize_omena_query_expression_semantics_canonical_producer_signal(&input);
            serde_json::to_writer_pretty(io::stdout(), &summary)?;
        }
        Some("input-source-side-canonical-producer") => {
            let input: EngineInputV2 = serde_json::from_str(&stdin)?;
            let summary = summarize_source_side_canonical_producer_signal_input(&input);
            serde_json::to_writer_pretty(io::stdout(), &summary)?;
        }
        Some("input-source-side-canonical-candidate") => {
            let input: EngineInputV2 = serde_json::from_str(&stdin)?;
            let summary = summarize_source_side_canonical_candidate_bundle_input(&input);
            serde_json::to_writer_pretty(io::stdout(), &summary)?;
        }
        Some("input-source-side-evaluator-candidates") => {
            let input: EngineInputV2 = serde_json::from_str(&stdin)?;
            let summary = summarize_source_side_evaluator_candidates_input(&input);
            serde_json::to_writer_pretty(io::stdout(), &summary)?;
        }
        Some("input-semantic-canonical-candidate") => {
            let input: EngineInputV2 = serde_json::from_str(&stdin)?;
            let summary = summarize_semantic_canonical_candidate_bundle_input(&input);
            serde_json::to_writer_pretty(io::stdout(), &summary)?;
        }
        Some("input-semantic-evaluator-candidates") => {
            let input: EngineInputV2 = serde_json::from_str(&stdin)?;
            let summary = summarize_semantic_evaluator_candidates_input(&input);
            serde_json::to_writer_pretty(io::stdout(), &summary)?;
        }
        Some("input-semantic-canonical-producer") => {
            let input: EngineInputV2 = serde_json::from_str(&stdin)?;
            let summary = summarize_semantic_canonical_producer_signal_input(&input);
            serde_json::to_writer_pretty(io::stdout(), &summary)?;
        }
        Some("omena-parser-style-facts") => {
            let input: OmenaParserStyleFactsInputV0 = serde_json::from_str(&stdin)?;
            let dialect = parse_omena_parser_style_dialect(input.dialect.as_str())?;
            let summary =
                summarize_omena_query_omena_parser_style_facts(&input.style_source, dialect);
            serde_json::to_writer_pretty(io::stdout(), &summary)?;
        }
        Some("omena-parser-css-modules-intermediate") => {
            let input: OmenaParserCssModulesIntermediateInputV0 = serde_json::from_str(&stdin)?;
            let dialect = parse_omena_parser_style_dialect(input.dialect.as_str())?;
            let summary = summarize_omena_query_omena_parser_css_modules_intermediate(
                &input.style_source,
                dialect,
            );
            serde_json::to_writer_pretty(io::stdout(), &summary)?;
        }
        Some("omena-parser-lex") => {
            let input: OmenaParserLexInputV0 = serde_json::from_str(&stdin)?;
            let dialect = parse_omena_parser_style_dialect(input.dialect.as_str())?;
            let summary = summarize_omena_query_omena_parser_lex(&input.style_source, dialect);
            serde_json::to_writer_pretty(io::stdout(), &summary)?;
        }
        Some("input-scss-evaluator-control-flow") => {
            let input: TargetStyleEngineInputV0 = serde_json::from_str(&stdin)?;
            let Some(summary) = summarize_omena_query_scss_evaluator_control_flow_from_engine_input(
                &input.engine_input,
                &input.target_style_path,
            ) else {
                return Err(format!(
                    "target style source not found in EngineInputV2 for {}",
                    input.target_style_path
                )
                .into());
            };
            serde_json::to_writer_pretty(io::stdout(), &summary)?;
        }
        Some("input-scss-evaluator-control-flow-oracle-corpus") => {
            let summary = summarize_omena_query_scss_evaluator_control_flow_oracle_corpus();
            serde_json::to_writer_pretty(io::stdout(), &summary)?;
        }
        Some("input-native-css-evaluator") => {
            let input: TargetStyleEngineInputV0 = serde_json::from_str(&stdin)?;
            let Some(summary) = summarize_omena_query_native_css_evaluator_from_engine_input(
                &input.engine_input,
                &input.target_style_path,
            ) else {
                return Err(format!(
                    "target style source not found in EngineInputV2 for {}",
                    input.target_style_path
                )
                .into());
            };
            serde_json::to_writer_pretty(io::stdout(), &summary)?;
        }
        Some("input-static-stylesheet-evaluator-oracle-corpus") => {
            let summary = summarize_omena_query_static_stylesheet_evaluator_oracle_corpus();
            serde_json::to_writer_pretty(io::stdout(), &summary)?;
        }
        Some("input-static-stylesheet-evaluator") => {
            let input: TargetStyleEngineInputV0 = serde_json::from_str(&stdin)?;
            let Some(summary) = summarize_omena_query_static_stylesheet_evaluator_from_engine_input(
                &input.engine_input,
                &input.target_style_path,
            ) else {
                return Err(format!(
                    "target style source not found in EngineInputV2 for {}",
                    input.target_style_path
                )
                .into());
            };
            serde_json::to_writer_pretty(io::stdout(), &summary)?;
        }
        Some("input-static-lif-exports") => {
            let input: TargetStyleEngineInputV0 = serde_json::from_str(&stdin)?;
            let Some(summary) = summarize_omena_query_static_lif_exports_from_engine_input(
                &input.engine_input,
                &input.target_style_path,
            ) else {
                return Err(format!(
                    "target style source not found in EngineInputV2 for {}",
                    input.target_style_path
                )
                .into());
            };
            serde_json::to_writer_pretty(io::stdout(), &summary)?;
        }
        Some("style-semantic-graph") => {
            let input: StyleSemanticGraphInputV0 = serde_json::from_str(&stdin)?;
            let Some(summary) = summarize_omena_query_style_semantic_graph_from_source(
                &input.style_path,
                &input.style_source,
                &input.engine_input,
            ) else {
                return Err("unsupported style module path".into());
            };
            serde_json::to_writer_pretty(io::stdout(), &summary)?;
        }
        Some("read-cascade-at-position") => {
            let input: ReadCascadeAtPositionInputV0 = serde_json::from_str(&stdin)?;
            let Some(summary) = read_omena_query_cascade_at_position(
                &input.style_path,
                &input.style_source,
                &input.engine_input,
                input.position.into(),
            ) else {
                return Err("unsupported style module path".into());
            };
            serde_json::to_writer_pretty(io::stdout(), &summary)?;
        }
        Some("style-diagnostics-for-file") => {
            let input: StyleDiagnosticsForFileInputV0 = serde_json::from_str(&stdin)?;
            let summary = summarize_style_diagnostics_from_committed_selector(&input)?;
            serde_json::to_writer_pretty(io::stdout(), &summary)?;
        }
        Some("source-diagnostics-for-file") => {
            let input: SourceDiagnosticsForFileInputV0 = serde_json::from_str(&stdin)?;
            let summary = summarize_omena_query_source_diagnostics_for_workspace_file(
                &input.source_path,
                &input.source_source,
                &input.styles,
                &input.package_manifests,
            );
            serde_json::to_writer_pretty(io::stdout(), &summary)?;
        }
        Some("completion-at") => {
            let input: CompletionAtInputV0 = serde_json::from_str(&stdin)?;
            let position = input.position.into();
            let summary = match input.file_kind {
                CompletionFileKindV0::Style => {
                    let style_source = input
                        .style_source
                        .or_else(|| {
                            input
                                .styles
                                .iter()
                                .find(|source| source.style_path == input.file_uri)
                                .map(|source| source.style_source.clone())
                        })
                        .ok_or("missing style source for completion-at style request")?;
                    let mut styles = input.styles;
                    if !styles
                        .iter()
                        .any(|source| source.style_path == input.file_uri)
                    {
                        styles.push(OmenaQueryStyleSourceInputV0 {
                            style_path: input.file_uri.clone(),
                            style_source,
                        });
                    }
                    summarize_omena_query_style_completion_for_workspace_file_with_resolution_inputs(
                        &input.file_uri,
                        styles.as_slice(),
                        &[],
                        &[],
                        &OmenaQueryStyleResolutionInputsV0::default(),
                        position,
                    )
                }
                CompletionFileKindV0::Source => {
                    summarize_omena_query_source_completion_for_workspace_file(
                        &input.file_uri,
                        position,
                        &input.styles,
                        input.target_style_uri.as_deref(),
                        input.value_prefix.as_deref(),
                        input.preferred_selector_names.as_slice(),
                    )
                }
            };
            serde_json::to_writer_pretty(io::stdout(), &summary)?;
        }
        Some("style-code-actions") => {
            let input: StyleCodeActionsInputV0 = serde_json::from_str(&stdin)?;
            let summary = run_style_code_actions_facade(input);
            serde_json::to_writer_pretty(io::stdout(), &summary)?;
        }
        Some("refs-for-class") => {
            let input: RefsForClassInputV0 = serde_json::from_str(&stdin)?;
            let summary = summarize_omena_query_refs_for_workspace_class(
                &input.selector_name,
                input.target_style_uri.as_deref(),
                input.include_declaration,
                &input.styles,
                &input.source_documents,
                &input.package_manifests,
            );
            serde_json::to_writer_pretty(io::stdout(), &summary)?;
        }
        Some("rename-plan") => {
            let input: RenamePlanInputV0 = serde_json::from_str(&stdin)?;
            let summary = summarize_omena_query_rename_plan_for_workspace_class(
                &input.selector_name,
                &input.new_name,
                input.target_style_uri.as_deref(),
                &input.styles,
                &input.source_documents,
                &input.package_manifests,
            );
            serde_json::to_writer_pretty(io::stdout(), &summary)?;
        }
        Some("read-style-context-index") => {
            let input: ReadStyleContextIndexInputV0 = serde_json::from_str(&stdin)?;
            let Some(summary) = read_omena_query_style_context_index(
                &input.style_path,
                &input.style_source,
                &input.engine_input,
            ) else {
                return Err("unsupported style module path".into());
            };
            serde_json::to_writer_pretty(io::stdout(), &summary)?;
        }
        Some("style-semantic-graph-batch") => {
            let input: StyleSemanticGraphBatchInputV0 = serde_json::from_str(&stdin)?;
            let output =
                summarize_omena_query_style_semantic_graph_batch_from_sources_with_committed_selector(
                    &input,
                )?;
            serde_json::to_writer_pretty(io::stdout(), &output)?;
        }
        Some("workspace-cross-file-summary") => {
            let input: WorkspaceCrossFileSummaryInputV0 = serde_json::from_str(&stdin)?;
            let summary = summarize_workspace_cross_file_summary_from_committed_selector(&input)?;
            serde_json::to_writer_pretty(io::stdout(), &summary)?;
        }
        Some("transform-plan") => {
            let input: TransformPlanInputV0 = serde_json::from_str(&stdin)?;
            let output = if let Some(target_query) = input.target_query.as_deref() {
                summarize_omena_query_transform_plan_from_target_query_with_context(
                    &input.style_path,
                    &input.style_source,
                    target_query,
                    input.target_options.into(),
                    default_omena_query_transform_print_options(),
                    &input.transform_context,
                )
            } else {
                summarize_omena_query_transform_plan_from_source_with_context(
                    &input.style_path,
                    &input.style_source,
                    &input.target_label,
                    input.target_support.into(),
                    input.target_options.into(),
                    default_omena_query_transform_print_options(),
                    &input.transform_context,
                )
            };
            serde_json::to_writer_pretty(io::stdout(), &output)?;
        }
        Some("transform-context") => {
            let input: TransformContextInputV0 = serde_json::from_str(&stdin)?;
            let package_manifests = input
                .package_manifests
                .iter()
                .map(|manifest| OmenaQueryStylePackageManifestV0 {
                    package_json_path: manifest.package_json_path.clone(),
                    package_json_source: manifest.package_json_source.clone(),
                })
                .collect::<Vec<_>>();
            let output = summarize_omena_query_transform_context_from_sources(
                &input.target_style_path,
                input
                    .styles
                    .iter()
                    .map(|style| (style.style_path.as_str(), style.style_source.as_str())),
                &package_manifests,
            );
            serde_json::to_writer_pretty(io::stdout(), &output)?;
        }
        Some("transform-context-from-engine-input") => {
            let input: TransformContextFromEngineInputV0 = serde_json::from_str(&stdin)?;
            let output = summarize_omena_query_transform_context_from_engine_input(
                &input.engine_input,
                &input.target_style_path,
                input.closed_style_world,
            );
            serde_json::to_writer_pretty(io::stdout(), &output)?;
        }
        Some("transform-execute") => {
            let input: TransformExecuteInputV0 = serde_json::from_str(&stdin)?;
            let output = execute_omena_query_transform_passes_from_source_with_context(
                &input.style_path,
                &input.style_source,
                &input.requested_pass_ids,
                &input.transform_context,
            );
            serde_json::to_writer_pretty(io::stdout(), &output)?;
        }
        Some("consumer-check-style-source") => {
            let input: ConsumerStyleSourceInputV0 = serde_json::from_str(&stdin)?;
            let output = summarize_omena_query_consumer_check_style_source(
                &input.style_path,
                &input.style_source,
            );
            serde_json::to_writer_pretty(io::stdout(), &output)?;
        }
        Some("consumer-build-style-source") => {
            let input: ConsumerStyleSourceBuildInputV0 = serde_json::from_str(&stdin)?;
            let output = if let Some(target_query) = input.target_query.as_deref() {
                execute_omena_query_consumer_build_style_sources_for_target_query_with_context_and_options(
                    &input.style_path,
                    &[OmenaQueryStyleSourceInputV0 {
                        style_path: input.style_path.clone(),
                        style_source: input.style_source.clone(),
                    }],
                    target_query,
                    &input.transform_context,
                    input.target_options.into(),
                    &[],
                )
                .map_err(|message| io::Error::new(io::ErrorKind::InvalidInput, message))?
            } else {
                execute_omena_query_consumer_build_style_sources_with_context(
                    &input.style_path,
                    &[OmenaQueryStyleSourceInputV0 {
                        style_path: input.style_path.clone(),
                        style_source: input.style_source.clone(),
                    }],
                    &input.requested_pass_ids,
                    &input.transform_context,
                    &[],
                )
                .map_err(|message| io::Error::new(io::ErrorKind::InvalidInput, message))?
            };
            serde_json::to_writer_pretty(io::stdout(), &output)?;
        }
        Some("consumer-build-style-sources") => {
            let input: ConsumerStyleSourcesBuildInputV0 = serde_json::from_str(&stdin)?;
            let output = if let Some(target_query) = input.target_query.as_deref() {
                execute_omena_query_consumer_build_style_sources_for_target_query_with_context_and_options(
                    &input.target_style_path,
                    &input.styles,
                    target_query,
                    &input.transform_context,
                    input.target_options.into(),
                    &input.package_manifests,
                )
                .map_err(|message| io::Error::new(io::ErrorKind::InvalidInput, message))?
            } else {
                execute_omena_query_consumer_build_style_sources_with_context(
                    &input.target_style_path,
                    &input.styles,
                    &input.requested_pass_ids,
                    &input.transform_context,
                    &input.package_manifests,
                )
                .map_err(|message| io::Error::new(io::ErrorKind::InvalidInput, message))?
            };
            serde_json::to_writer_pretty(io::stdout(), &output)?;
        }
        Some("consumer-transform-pass-list") => {
            let output = list_omena_query_transform_pass_summaries();
            serde_json::to_writer_pretty(io::stdout(), &output)?;
        }
        Some("input-expression-semantics-query-fragments") => {
            let input: EngineInputV2 = serde_json::from_str(&stdin)?;
            let summary = summarize_omena_query_expression_semantics_query_fragments(&input);
            serde_json::to_writer_pretty(io::stdout(), &summary)?;
        }
        Some("input-expression-semantics-match-fragments") => {
            let input: EngineInputV2 = serde_json::from_str(&stdin)?;
            let summary = summarize_expression_semantics_match_fragments_input(&input);
            serde_json::to_writer_pretty(io::stdout(), &summary)?;
        }
        Some("input-source-resolution-fragments") => {
            let input: EngineInputV2 = serde_json::from_str(&stdin)?;
            let summary = summarize_source_resolution_fragments_input(&input);
            serde_json::to_writer_pretty(io::stdout(), &summary)?;
        }
        Some("input-source-resolution-candidates") => {
            let input: EngineInputV2 = serde_json::from_str(&stdin)?;
            let summary = summarize_source_resolution_candidates_input(&input);
            serde_json::to_writer_pretty(io::stdout(), &summary)?;
        }
        Some("input-source-resolution-evaluator-candidates") => {
            let input: EngineInputV2 = serde_json::from_str(&stdin)?;
            let summary = summarize_source_resolution_evaluator_candidates_input(&input);
            serde_json::to_writer_pretty(io::stdout(), &summary)?;
        }
        Some("input-source-resolution-canonical-candidate") => {
            let input: EngineInputV2 = serde_json::from_str(&stdin)?;
            let summary = summarize_source_resolution_canonical_candidate_bundle_input(&input);
            serde_json::to_writer_pretty(io::stdout(), &summary)?;
        }
        Some("input-source-resolution-canonical-producer") => {
            let input: EngineInputV2 = serde_json::from_str(&stdin)?;
            let summary = summarize_omena_query_source_resolution_canonical_producer_signal(&input);
            serde_json::to_writer_pretty(io::stdout(), &summary)?;
        }
        Some("input-omena-query-boundary") => {
            let input: EngineInputV2 = serde_json::from_str(&stdin)?;
            let summary = summarize_omena_query_boundary(&input);
            serde_json::to_writer_pretty(io::stdout(), &summary)?;
        }
        Some("input-omena-query-evaluation-runtime") => {
            let input: EngineInputV2 = serde_json::from_str(&stdin)?;
            let mut runtime = OmenaQueryExpressionDomainFlowRuntimeV0::default();
            let summary = summarize_omena_query_evaluation_runtime(&input, &mut runtime);
            serde_json::to_writer_pretty(io::stdout(), &summary)?;
        }
        Some("input-omena-resolver-boundary") => {
            let input: EngineInputV2 = serde_json::from_str(&stdin)?;
            let summary = summarize_omena_resolver_boundary(&input);
            serde_json::to_writer_pretty(io::stdout(), &summary)?;
        }
        Some("input-omena-resolver-module-graph") => {
            let input: EngineInputV2 = serde_json::from_str(&stdin)?;
            let summary = summarize_omena_resolver_module_graph_index(&input);
            serde_json::to_writer_pretty(io::stdout(), &summary)?;
        }
        Some("input-omena-resolver-source-resolution-runtime") => {
            let input: EngineInputV2 = serde_json::from_str(&stdin)?;
            let summary = summarize_omena_query_source_resolution_runtime(&input);
            serde_json::to_writer_pretty(io::stdout(), &summary)?;
        }
        Some("omena-resolver-runtime-query-boundary") => {
            let module_graph: OmenaResolverModuleGraphSummaryV0 = serde_json::from_str(&stdin)?;
            let summary = summarize_omena_resolver_runtime_query_boundary(&module_graph);
            serde_json::to_writer_pretty(io::stdout(), &summary)?;
        }
        Some("omena-resolver-style-module-resolution") => {
            let input: OmenaResolverStyleModuleResolutionInputV0 = serde_json::from_str(&stdin)?;
            let available_style_paths = input
                .available_style_paths
                .iter()
                .map(String::as_str)
                .collect::<BTreeSet<_>>();
            let package_manifests = input
                .package_manifests
                .iter()
                .map(|manifest| OmenaResolverStylePackageManifestV0 {
                    package_json_path: manifest.package_json_path.clone(),
                    package_json_source: manifest.package_json_source.clone(),
                })
                .collect::<Vec<_>>();
            let bundler_path_mappings = input
                .bundler_path_mappings
                .iter()
                .map(|mapping| OmenaResolverBundlerPathAliasMappingV0 {
                    pattern: mapping.pattern.clone(),
                    target_path: mapping.target_path.clone(),
                })
                .collect::<Vec<_>>();
            let tsconfig_path_mappings = input
                .tsconfig_path_mappings
                .iter()
                .map(|mapping| OmenaResolverTsconfigPathMappingV0 {
                    base_path: mapping.base_path.clone(),
                    pattern: mapping.pattern.clone(),
                    target_patterns: mapping.target_patterns.clone(),
                })
                .collect::<Vec<_>>();
            let summary = summarize_omena_resolver_style_module_resolution_with_path_mappings(
                &input.from_style_path,
                &input.source,
                &available_style_paths,
                &package_manifests,
                &bundler_path_mappings,
                &tsconfig_path_mappings,
            );
            serde_json::to_writer_pretty(io::stdout(), &summary)?;
        }
        Some("omena-resolver-specifier-resolution-runtime") => {
            let input: OmenaResolverSpecifierResolutionRuntimeInputV0 =
                serde_json::from_str(&stdin)?;
            let available_style_paths = input
                .available_style_paths
                .iter()
                .map(String::as_str)
                .collect::<BTreeSet<_>>();
            let package_manifests = input
                .package_manifests
                .iter()
                .map(|manifest| OmenaResolverStylePackageManifestV0 {
                    package_json_path: manifest.package_json_path.clone(),
                    package_json_source: manifest.package_json_source.clone(),
                })
                .collect::<Vec<_>>();
            let bundler_path_mappings = input
                .bundler_path_mappings
                .iter()
                .map(|mapping| OmenaResolverBundlerPathAliasMappingV0 {
                    pattern: mapping.pattern.clone(),
                    target_path: mapping.target_path.clone(),
                })
                .collect::<Vec<_>>();
            let tsconfig_path_mappings = input
                .tsconfig_path_mappings
                .iter()
                .map(|mapping| OmenaResolverTsconfigPathMappingV0 {
                    base_path: mapping.base_path.clone(),
                    pattern: mapping.pattern.clone(),
                    target_patterns: mapping.target_patterns.clone(),
                })
                .collect::<Vec<_>>();
            let summary = summarize_omena_resolver_specifier_resolution_runtime_with_path_mappings(
                &input.from_style_path,
                &input.sources,
                &available_style_paths,
                &package_manifests,
                &bundler_path_mappings,
                &tsconfig_path_mappings,
            );
            serde_json::to_writer_pretty(io::stdout(), &summary)?;
        }
        Some("omena-query-selected-query-adapter-capabilities") => {
            let summary = summarize_omena_query_selected_query_adapter_capabilities();
            serde_json::to_writer_pretty(io::stdout(), &summary)?;
        }
        Some("omena-checker-m-tier-evaluations") => {
            let input: OmenaCheckerMTierEvaluationInputV0 = serde_json::from_str(&stdin)?;
            let summary = summarize_omena_checker_m_tier_evaluations(input);
            serde_json::to_writer_pretty(io::stdout(), &summary)?;
        }
        Some("omena-checker-k-limited-flow-m-tier-evaluations") => {
            let input: OmenaCheckerKLimitedFlowMTierEvaluationInputV0 =
                serde_json::from_str(&stdin)?;
            let summary = summarize_omena_checker_k_limited_flow_m_tier_evaluations(input);
            serde_json::to_writer_pretty(io::stdout(), &summary)?;
        }
        Some("omena-checker-cascade-evaluations") => {
            let input: OmenaCheckerCascadeInputV0 = serde_json::from_str(&stdin)?;
            let summary = summarize_omena_checker_cascade_evaluations(input);
            serde_json::to_writer_pretty(io::stdout(), &summary)?;
        }
        Some("omena-checker-grn-evaluations") => {
            let input: OmenaCheckerGrnInputV0 = serde_json::from_str(&stdin)?;
            let summary = summarize_omena_checker_grn_evaluations(input);
            serde_json::to_writer_pretty(io::stdout(), &summary)?;
        }
        Some("omena-checker-smt-evaluations") => {
            let input: OmenaCheckerSmtInputV0 = serde_json::from_str(&stdin)?;
            let summary = summarize_omena_checker_smt_evaluations(input);
            serde_json::to_writer_pretty(io::stdout(), &summary)?;
        }
        Some("omena-checker-mdl-evaluations") => {
            let input: OmenaCheckerMdlEvaluationInputV0 = serde_json::from_str(&stdin)?;
            let summary = summarize_omena_checker_mdl_evaluations(input);
            serde_json::to_writer_pretty(io::stdout(), &summary)?;
        }
        Some("omena-checker-streaming-ifds-evaluations") => {
            let input: OmenaCheckerStreamingIfdsEvaluationInputV0 = serde_json::from_str(&stdin)?;
            let summary = summarize_omena_checker_streaming_ifds_evaluations(input);
            serde_json::to_writer_pretty(io::stdout(), &summary)?;
        }
        Some("omena-checker-rg-flow-evaluations") => {
            let input: OmenaCheckerRgFlowEvaluationInputV0 = serde_json::from_str(&stdin)?;
            let summary = summarize_omena_checker_rg_flow_evaluations(input);
            serde_json::to_writer_pretty(io::stdout(), &summary)?;
        }
        Some("omena-checker-replica-ensemble-evaluations") => {
            let input: OmenaCheckerReplicaEnsembleEvaluationInputV0 = serde_json::from_str(&stdin)?;
            let summary = summarize_omena_checker_replica_ensemble_evaluations(input);
            serde_json::to_writer_pretty(io::stdout(), &summary)?;
        }
        Some("omena-checker-categorical-evaluations") => {
            let input: OmenaCheckerCategoricalEvaluationInputV0 = serde_json::from_str(&stdin)?;
            let summary = summarize_omena_checker_categorical_evaluations(input);
            serde_json::to_writer_pretty(io::stdout(), &summary)?;
        }
        Some("omena-checker-rule-enforcement-coverage") => {
            let summary = summarize_omena_checker_rule_enforcement_coverage_v0();
            serde_json::to_writer_pretty(io::stdout(), &summary)?;
        }
        Some("input-source-resolution-match-fragments") => {
            let input: EngineInputV2 = serde_json::from_str(&stdin)?;
            let summary = summarize_source_resolution_match_fragments_input(&input);
            serde_json::to_writer_pretty(io::stdout(), &summary)?;
        }
        Some("input-source-resolution-query-fragments") => {
            let input: EngineInputV2 = serde_json::from_str(&stdin)?;
            let summary = summarize_omena_query_source_resolution_query_fragments(&input);
            serde_json::to_writer_pretty(io::stdout(), &summary)?;
        }
        Some("output-checker-style-recovery-canonical-candidate") => {
            let payload: ShadowPayloadV0 = serde_json::from_str(&stdin)?;
            let summary = summarize_checker_style_recovery_canonical_candidate(payload);
            serde_json::to_writer_pretty(io::stdout(), &summary)?;
        }
        Some("output-checker-style-recovery-canonical-producer") => {
            let payload: ShadowPayloadV0 = serde_json::from_str(&stdin)?;
            let summary = summarize_checker_style_recovery_canonical_producer(payload);
            serde_json::to_writer_pretty(io::stdout(), &summary)?;
        }
        Some("output-checker-style-unused-canonical-candidate") => {
            let payload: ShadowPayloadV0 = serde_json::from_str(&stdin)?;
            let summary = summarize_checker_style_unused_canonical_candidate(payload);
            serde_json::to_writer_pretty(io::stdout(), &summary)?;
        }
        Some("output-checker-style-unused-canonical-producer") => {
            let payload: ShadowPayloadV0 = serde_json::from_str(&stdin)?;
            let summary = summarize_checker_style_unused_canonical_producer(payload);
            serde_json::to_writer_pretty(io::stdout(), &summary)?;
        }
        Some("output-checker-source-missing-canonical-candidate") => {
            let payload: ShadowPayloadV0 = serde_json::from_str(&stdin)?;
            let summary = summarize_checker_source_missing_canonical_candidate(payload);
            serde_json::to_writer_pretty(io::stdout(), &summary)?;
        }
        Some("output-checker-source-missing-canonical-producer") => {
            let payload: ShadowPayloadV0 = serde_json::from_str(&stdin)?;
            let summary = summarize_checker_source_missing_canonical_producer(payload);
            serde_json::to_writer_pretty(io::stdout(), &summary)?;
        }
        Some(other) => {
            return Err(format!("unsupported engine-shadow-runner mode: {other}").into());
        }
    }

    Ok(())
}

fn run_daemon() -> Result<(), Box<dyn std::error::Error>> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut expression_domain_runtime = OmenaQueryExpressionDomainFlowRuntimeV0::default();

    for line in stdin.lock().lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }

        let response = match serde_json::from_str::<EngineShadowRunnerDaemonRequestV0>(&line) {
            Ok(request) => {
                let id = request.id.clone();
                match run_daemon_selected_query_command(
                    &request.command,
                    request.input,
                    &mut expression_domain_runtime,
                ) {
                    Ok(result) => EngineShadowRunnerDaemonResponseV0 {
                        schema_version: "0",
                        id,
                        ok: true,
                        result: Some(result),
                        error: None,
                    },
                    Err(error) => EngineShadowRunnerDaemonResponseV0 {
                        schema_version: "0",
                        id,
                        ok: false,
                        result: None,
                        error: Some(error.to_string()),
                    },
                }
            }
            Err(error) => EngineShadowRunnerDaemonResponseV0 {
                schema_version: "0",
                id: serde_json::Value::Null,
                ok: false,
                result: None,
                error: Some(error.to_string()),
            },
        };

        serde_json::to_writer(&mut stdout, &response)?;
        stdout.write_all(b"\n")?;
        stdout.flush()?;
    }

    Ok(())
}

fn parse_omena_parser_style_dialect(
    dialect: &str,
) -> Result<OmenaParserStyleDialect, Box<dyn std::error::Error>> {
    match dialect {
        "css" => Ok(OmenaParserStyleDialect::Css),
        "scss" => Ok(OmenaParserStyleDialect::Scss),
        "sass" => Ok(OmenaParserStyleDialect::Sass),
        "less" => Ok(OmenaParserStyleDialect::Less),
        other => Err(format!("unsupported omena parser style dialect: {other}").into()),
    }
}

fn run_daemon_selected_query_command(
    command: &str,
    input: serde_json::Value,
    expression_domain_runtime: &mut OmenaQueryExpressionDomainFlowRuntimeV0,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    match command {
        "input-omena-query-evaluation-runtime" => {
            let input: EngineInputV2 = serde_json::from_value(input)?;
            Ok(serde_json::to_value(
                summarize_omena_query_evaluation_runtime(&input, expression_domain_runtime),
            )?)
        }
        "input-source-resolution-canonical-producer" => {
            let input: EngineInputV2 = serde_json::from_value(input)?;
            Ok(serde_json::to_value(
                summarize_omena_query_source_resolution_canonical_producer_signal(&input),
            )?)
        }
        "input-expression-semantics-canonical-producer" => {
            let input: EngineInputV2 = serde_json::from_value(input)?;
            Ok(serde_json::to_value(
                summarize_omena_query_expression_semantics_canonical_producer_signal(&input),
            )?)
        }
        "input-expression-domain-incremental-flow-analysis" => {
            let input: EngineInputV2 = serde_json::from_value(input)?;
            Ok(serde_json::to_value(
                summarize_omena_query_expression_domain_incremental_flow_analysis(
                    &input,
                    expression_domain_runtime,
                ),
            )?)
        }
        "input-expression-domain-selector-projection" => {
            let input: EngineInputV2 = serde_json::from_value(input)?;
            Ok(serde_json::to_value(
                summarize_omena_query_expression_domain_selector_projection(&input),
            )?)
        }
        "input-expression-domain-control-flow-analysis" => {
            let input: EngineInputV2 = serde_json::from_value(input)?;
            Ok(serde_json::to_value(
                summarize_omena_query_expression_domain_control_flow_analysis(&input),
            )?)
        }
        "input-expression-domain-provenance-explanations" => {
            let input: EngineInputV2 = serde_json::from_value(input)?;
            Ok(serde_json::to_value(
                summarize_omena_query_expression_domain_provenance_explanations(&input),
            )?)
        }
        "input-expression-domain-reduced-product-iteration" => {
            let input: EngineInputV2 = serde_json::from_value(input)?;
            Ok(serde_json::to_value(
                summarize_omena_query_expression_domain_reduced_product_iteration(&input),
            )?)
        }
        "input-selector-usage-canonical-producer" => {
            let input: EngineInputV2 = serde_json::from_value(input)?;
            Ok(serde_json::to_value(
                summarize_omena_query_selector_usage_canonical_producer_signal(&input),
            )?)
        }
        "omena-parser-style-facts" => {
            let input: OmenaParserStyleFactsInputV0 = serde_json::from_value(input)?;
            let dialect = parse_omena_parser_style_dialect(input.dialect.as_str())?;
            Ok(serde_json::to_value(
                summarize_omena_query_omena_parser_style_facts(&input.style_source, dialect),
            )?)
        }
        "omena-parser-css-modules-intermediate" => {
            let input: OmenaParserCssModulesIntermediateInputV0 = serde_json::from_value(input)?;
            let dialect = parse_omena_parser_style_dialect(input.dialect.as_str())?;
            Ok(serde_json::to_value(
                summarize_omena_query_omena_parser_css_modules_intermediate(
                    &input.style_source,
                    dialect,
                ),
            )?)
        }
        "omena-parser-lex" => {
            let input: OmenaParserLexInputV0 = serde_json::from_value(input)?;
            let dialect = parse_omena_parser_style_dialect(input.dialect.as_str())?;
            Ok(serde_json::to_value(
                summarize_omena_query_omena_parser_lex(&input.style_source, dialect),
            )?)
        }
        "input-scss-evaluator-control-flow" => {
            let input: TargetStyleEngineInputV0 = serde_json::from_value(input)?;
            let Some(summary) = summarize_omena_query_scss_evaluator_control_flow_from_engine_input(
                &input.engine_input,
                &input.target_style_path,
            ) else {
                return Err(format!(
                    "target style source not found in EngineInputV2 for {}",
                    input.target_style_path
                )
                .into());
            };
            Ok(serde_json::to_value(summary)?)
        }
        "input-scss-evaluator-control-flow-oracle-corpus" => Ok(serde_json::to_value(
            summarize_omena_query_scss_evaluator_control_flow_oracle_corpus(),
        )?),
        "input-native-css-evaluator" => {
            let input: TargetStyleEngineInputV0 = serde_json::from_value(input)?;
            let Some(summary) = summarize_omena_query_native_css_evaluator_from_engine_input(
                &input.engine_input,
                &input.target_style_path,
            ) else {
                return Err(format!(
                    "target style source not found in EngineInputV2 for {}",
                    input.target_style_path
                )
                .into());
            };
            Ok(serde_json::to_value(summary)?)
        }
        "input-static-stylesheet-evaluator-oracle-corpus" => Ok(serde_json::to_value(
            summarize_omena_query_static_stylesheet_evaluator_oracle_corpus(),
        )?),
        "input-static-stylesheet-evaluator" => {
            let input: TargetStyleEngineInputV0 = serde_json::from_value(input)?;
            let Some(summary) = summarize_omena_query_static_stylesheet_evaluator_from_engine_input(
                &input.engine_input,
                &input.target_style_path,
            ) else {
                return Err(format!(
                    "target style source not found in EngineInputV2 for {}",
                    input.target_style_path
                )
                .into());
            };
            Ok(serde_json::to_value(summary)?)
        }
        "input-static-lif-exports" => {
            let input: TargetStyleEngineInputV0 = serde_json::from_value(input)?;
            let Some(summary) = summarize_omena_query_static_lif_exports_from_engine_input(
                &input.engine_input,
                &input.target_style_path,
            ) else {
                return Err(format!(
                    "target style source not found in EngineInputV2 for {}",
                    input.target_style_path
                )
                .into());
            };
            Ok(serde_json::to_value(summary)?)
        }
        "style-semantic-graph" => {
            let input: StyleSemanticGraphInputV0 = serde_json::from_value(input)?;
            let Some(summary) = summarize_omena_query_style_semantic_graph_from_source(
                &input.style_path,
                &input.style_source,
                &input.engine_input,
            ) else {
                return Err("unsupported style module path".into());
            };
            Ok(serde_json::to_value(summary)?)
        }
        "read-cascade-at-position" => {
            let input: ReadCascadeAtPositionInputV0 = serde_json::from_value(input)?;
            let Some(summary) = read_omena_query_cascade_at_position(
                &input.style_path,
                &input.style_source,
                &input.engine_input,
                input.position.into(),
            ) else {
                return Err("unsupported style module path".into());
            };
            Ok(serde_json::to_value(summary)?)
        }
        "style-code-actions" => {
            let input: StyleCodeActionsInputV0 = serde_json::from_value(input)?;
            Ok(serde_json::to_value(run_style_code_actions_facade(input))?)
        }
        "style-semantic-graph-batch" => {
            let input: StyleSemanticGraphBatchInputV0 = serde_json::from_value(input)?;
            Ok(serde_json::to_value(
                summarize_omena_query_style_semantic_graph_batch_from_sources_with_committed_selector(
                    &input,
                )?,
            )?)
        }
        "workspace-cross-file-summary" => {
            let input: WorkspaceCrossFileSummaryInputV0 = serde_json::from_value(input)?;
            Ok(serde_json::to_value(
                summarize_workspace_cross_file_summary_from_committed_selector(&input)?,
            )?)
        }
        "style-diagnostics-for-file" => {
            let input: StyleDiagnosticsForFileInputV0 = serde_json::from_value(input)?;
            Ok(serde_json::to_value(
                summarize_style_diagnostics_from_committed_selector(&input)?,
            )?)
        }
        "source-diagnostics-for-file" => {
            let input: SourceDiagnosticsForFileInputV0 = serde_json::from_value(input)?;
            Ok(serde_json::to_value(
                summarize_omena_query_source_diagnostics_for_workspace_file(
                    &input.source_path,
                    &input.source_source,
                    &input.styles,
                    &input.package_manifests,
                ),
            )?)
        }
        "transform-plan" => {
            let input: TransformPlanInputV0 = serde_json::from_value(input)?;
            let output = if let Some(target_query) = input.target_query.as_deref() {
                summarize_omena_query_transform_plan_from_target_query_with_context(
                    &input.style_path,
                    &input.style_source,
                    target_query,
                    input.target_options.into(),
                    default_omena_query_transform_print_options(),
                    &input.transform_context,
                )
            } else {
                summarize_omena_query_transform_plan_from_source_with_context(
                    &input.style_path,
                    &input.style_source,
                    &input.target_label,
                    input.target_support.into(),
                    input.target_options.into(),
                    default_omena_query_transform_print_options(),
                    &input.transform_context,
                )
            };
            Ok(serde_json::to_value(output)?)
        }
        "transform-context" => {
            let input: TransformContextInputV0 = serde_json::from_value(input)?;
            let package_manifests = input
                .package_manifests
                .iter()
                .map(|manifest| OmenaQueryStylePackageManifestV0 {
                    package_json_path: manifest.package_json_path.clone(),
                    package_json_source: manifest.package_json_source.clone(),
                })
                .collect::<Vec<_>>();
            Ok(serde_json::to_value(
                summarize_omena_query_transform_context_from_sources(
                    &input.target_style_path,
                    input
                        .styles
                        .iter()
                        .map(|style| (style.style_path.as_str(), style.style_source.as_str())),
                    &package_manifests,
                ),
            )?)
        }
        "transform-execute" => {
            let input: TransformExecuteInputV0 = serde_json::from_value(input)?;
            Ok(serde_json::to_value(
                execute_omena_query_transform_passes_from_source_with_context(
                    &input.style_path,
                    &input.style_source,
                    &input.requested_pass_ids,
                    &input.transform_context,
                ),
            )?)
        }
        "omena-resolver-style-module-resolution" => {
            let input: OmenaResolverStyleModuleResolutionInputV0 = serde_json::from_value(input)?;
            let available_style_paths = input
                .available_style_paths
                .iter()
                .map(String::as_str)
                .collect::<BTreeSet<_>>();
            let package_manifests = input
                .package_manifests
                .iter()
                .map(|manifest| OmenaResolverStylePackageManifestV0 {
                    package_json_path: manifest.package_json_path.clone(),
                    package_json_source: manifest.package_json_source.clone(),
                })
                .collect::<Vec<_>>();
            let bundler_path_mappings = input
                .bundler_path_mappings
                .iter()
                .map(|mapping| OmenaResolverBundlerPathAliasMappingV0 {
                    pattern: mapping.pattern.clone(),
                    target_path: mapping.target_path.clone(),
                })
                .collect::<Vec<_>>();
            let tsconfig_path_mappings = input
                .tsconfig_path_mappings
                .iter()
                .map(|mapping| OmenaResolverTsconfigPathMappingV0 {
                    base_path: mapping.base_path.clone(),
                    pattern: mapping.pattern.clone(),
                    target_patterns: mapping.target_patterns.clone(),
                })
                .collect::<Vec<_>>();
            Ok(serde_json::to_value(
                summarize_omena_resolver_style_module_resolution_with_path_mappings(
                    &input.from_style_path,
                    &input.source,
                    &available_style_paths,
                    &package_manifests,
                    &bundler_path_mappings,
                    &tsconfig_path_mappings,
                ),
            )?)
        }
        "omena-resolver-specifier-resolution-runtime" => {
            let input: OmenaResolverSpecifierResolutionRuntimeInputV0 =
                serde_json::from_value(input)?;
            let available_style_paths = input
                .available_style_paths
                .iter()
                .map(String::as_str)
                .collect::<BTreeSet<_>>();
            let package_manifests = input
                .package_manifests
                .iter()
                .map(|manifest| OmenaResolverStylePackageManifestV0 {
                    package_json_path: manifest.package_json_path.clone(),
                    package_json_source: manifest.package_json_source.clone(),
                })
                .collect::<Vec<_>>();
            let bundler_path_mappings = input
                .bundler_path_mappings
                .iter()
                .map(|mapping| OmenaResolverBundlerPathAliasMappingV0 {
                    pattern: mapping.pattern.clone(),
                    target_path: mapping.target_path.clone(),
                })
                .collect::<Vec<_>>();
            let tsconfig_path_mappings = input
                .tsconfig_path_mappings
                .iter()
                .map(|mapping| OmenaResolverTsconfigPathMappingV0 {
                    base_path: mapping.base_path.clone(),
                    pattern: mapping.pattern.clone(),
                    target_patterns: mapping.target_patterns.clone(),
                })
                .collect::<Vec<_>>();
            Ok(serde_json::to_value(
                summarize_omena_resolver_specifier_resolution_runtime_with_path_mappings(
                    &input.from_style_path,
                    &input.sources,
                    &available_style_paths,
                    &package_manifests,
                    &bundler_path_mappings,
                    &tsconfig_path_mappings,
                ),
            )?)
        }
        "omena-checker-m-tier-evaluations" => {
            let input: OmenaCheckerMTierEvaluationInputV0 = serde_json::from_value(input)?;
            Ok(serde_json::to_value(
                summarize_omena_checker_m_tier_evaluations(input),
            )?)
        }
        "omena-checker-k-limited-flow-m-tier-evaluations" => {
            let input: OmenaCheckerKLimitedFlowMTierEvaluationInputV0 =
                serde_json::from_value(input)?;
            Ok(serde_json::to_value(
                summarize_omena_checker_k_limited_flow_m_tier_evaluations(input),
            )?)
        }
        "omena-checker-cascade-evaluations" => {
            let input: OmenaCheckerCascadeInputV0 = serde_json::from_value(input)?;
            Ok(serde_json::to_value(
                summarize_omena_checker_cascade_evaluations(input),
            )?)
        }
        "omena-checker-grn-evaluations" => {
            let input: OmenaCheckerGrnInputV0 = serde_json::from_value(input)?;
            Ok(serde_json::to_value(
                summarize_omena_checker_grn_evaluations(input),
            )?)
        }
        "omena-checker-smt-evaluations" => {
            let input: OmenaCheckerSmtInputV0 = serde_json::from_value(input)?;
            Ok(serde_json::to_value(
                summarize_omena_checker_smt_evaluations(input),
            )?)
        }
        "omena-checker-mdl-evaluations" => {
            let input: OmenaCheckerMdlEvaluationInputV0 = serde_json::from_value(input)?;
            Ok(serde_json::to_value(
                summarize_omena_checker_mdl_evaluations(input),
            )?)
        }
        "omena-checker-streaming-ifds-evaluations" => {
            let input: OmenaCheckerStreamingIfdsEvaluationInputV0 = serde_json::from_value(input)?;
            Ok(serde_json::to_value(
                summarize_omena_checker_streaming_ifds_evaluations(input),
            )?)
        }
        "omena-checker-rg-flow-evaluations" => {
            let input: OmenaCheckerRgFlowEvaluationInputV0 = serde_json::from_value(input)?;
            Ok(serde_json::to_value(
                summarize_omena_checker_rg_flow_evaluations(input),
            )?)
        }
        "omena-checker-replica-ensemble-evaluations" => {
            let input: OmenaCheckerReplicaEnsembleEvaluationInputV0 =
                serde_json::from_value(input)?;
            Ok(serde_json::to_value(
                summarize_omena_checker_replica_ensemble_evaluations(input),
            )?)
        }
        "omena-checker-categorical-evaluations" => {
            let input: OmenaCheckerCategoricalEvaluationInputV0 = serde_json::from_value(input)?;
            Ok(serde_json::to_value(
                summarize_omena_checker_categorical_evaluations(input),
            )?)
        }
        "omena-checker-rule-enforcement-coverage" => Ok(serde_json::to_value(
            summarize_omena_checker_rule_enforcement_coverage_v0(),
        )?),
        other => Err(format!("unsupported engine-shadow-runner daemon command: {other}").into()),
    }
}

fn summarize_omena_checker_m_tier_evaluations(
    input: OmenaCheckerMTierEvaluationInputV0,
) -> OmenaCheckerMTierEvaluationRunnerOutputV0 {
    let selector_universe_count = input.selector_universe.len();
    let evaluations = evaluate_omena_checker_m_tier_rules(OmenaCheckerDynamicClassDomainInputV0 {
        abstract_value: input.abstract_value.into_abstract_class_value(),
        selector_universe: input.selector_universe,
    });

    OmenaCheckerMTierEvaluationRunnerOutputV0 {
        schema_version: "0",
        product: "omena-checker.m-tier-evaluations",
        selector_universe_count,
        evaluation_count: evaluations.len(),
        evaluations,
    }
}

fn summarize_omena_checker_k_limited_flow_m_tier_evaluations(
    input: OmenaCheckerKLimitedFlowMTierEvaluationInputV0,
) -> OmenaCheckerKLimitedFlowMTierEvaluationRunnerOutputV0 {
    let selector_universe_count = input.selector_universe.len();
    let flow_inputs = input
        .contexts
        .iter()
        .map(|context| KLimitedCallSiteFlowInputV0 {
            callee_key: context.callee_key.clone(),
            call_site_stack: context.call_site_stack.clone(),
            graph: checker_k_limited_flow_graph(context),
            exit_node_id: "exit".to_string(),
        })
        .collect::<Vec<_>>();
    let flow = analyze_k_limited_call_site_flows(&flow_inputs, input.max_context_depth);
    let contexts = flow
        .entries
        .into_iter()
        .map(|entry| {
            let evaluations =
                evaluate_omena_checker_m_tier_rules(OmenaCheckerDynamicClassDomainInputV0 {
                    abstract_value: entry.exit_value.clone(),
                    selector_universe: input.selector_universe.clone(),
                });
            let rule_code_names = evaluations
                .iter()
                .map(|evaluation| evaluation.rule_code_name)
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect::<Vec<_>>();

            OmenaCheckerKLimitedFlowMTierContextOutputV0 {
                callee_key: entry.callee_key,
                call_site_stack: entry.call_site_stack,
                context_key: entry.context_key,
                exit_value_kind: abstract_class_value_kind(&entry.exit_value),
                exit_value: entry.exit_value,
                evaluation_count: evaluations.len(),
                rule_code_names,
                evaluations,
            }
        })
        .collect::<Vec<_>>();
    let evaluation_count = contexts
        .iter()
        .map(|context| context.evaluation_count)
        .sum();

    OmenaCheckerKLimitedFlowMTierEvaluationRunnerOutputV0 {
        schema_version: "0",
        product: "omena-checker.k-limited-flow-m-tier-evaluations",
        flow_product: flow.product,
        context_sensitivity: flow.context_sensitivity,
        max_context_depth: flow.max_context_depth,
        selector_universe_count,
        context_count: contexts.len(),
        evaluation_count,
        contexts,
    }
}

fn checker_k_limited_flow_graph(
    context: &OmenaCheckerKLimitedFlowContextInputV0,
) -> ClassValueFlowGraphV0 {
    ClassValueFlowGraphV0 {
        context_key: None,
        nodes: vec![ClassValueFlowNodeV0 {
            id: "exit".to_string(),
            predecessors: Vec::new(),
            transfer: ClassValueFlowTransferV0::AssignFacts(checker_external_facts_from_value(
                &context.value,
            )),
        }],
    }
}

fn checker_external_facts_from_value(
    value: &OmenaCheckerAbstractClassValueInputV0,
) -> ExternalStringTypeFactsV0 {
    match value {
        OmenaCheckerAbstractClassValueInputV0::Exact { value } => {
            let mut facts = external_string_type_facts("exact");
            facts.values = Some(vec![value.clone()]);
            facts
        }
        OmenaCheckerAbstractClassValueInputV0::FiniteSet { values } => {
            let mut facts = external_string_type_facts("finiteSet");
            facts.values = Some(values.clone());
            facts
        }
        OmenaCheckerAbstractClassValueInputV0::Prefix { prefix } => {
            let mut facts = external_string_type_facts("constrained");
            facts.constraint_kind = Some("prefix".to_string());
            facts.prefix = Some(prefix.clone());
            facts
        }
        OmenaCheckerAbstractClassValueInputV0::Suffix { suffix } => {
            let mut facts = external_string_type_facts("constrained");
            facts.constraint_kind = Some("suffix".to_string());
            facts.suffix = Some(suffix.clone());
            facts
        }
        OmenaCheckerAbstractClassValueInputV0::PrefixSuffix {
            prefix,
            suffix,
            min_length,
        } => {
            let mut facts = external_string_type_facts("constrained");
            facts.constraint_kind = Some("prefixSuffix".to_string());
            facts.prefix = Some(prefix.clone());
            facts.suffix = Some(suffix.clone());
            facts.min_len = *min_length;
            facts
        }
        OmenaCheckerAbstractClassValueInputV0::CharInclusion {
            must_chars,
            may_chars,
            may_include_other_chars,
        } => {
            let mut facts = external_string_type_facts("constrained");
            facts.constraint_kind = Some("charInclusion".to_string());
            facts.char_must = Some(must_chars.clone());
            facts.char_may = Some(may_chars.clone());
            facts.may_include_other_chars = Some(*may_include_other_chars);
            facts
        }
        OmenaCheckerAbstractClassValueInputV0::Composite {
            prefix,
            suffix,
            min_length,
            must_chars,
            may_chars,
            may_include_other_chars,
        } => {
            let mut facts = external_string_type_facts("constrained");
            facts.constraint_kind = Some("composite".to_string());
            facts.prefix = prefix.clone();
            facts.suffix = suffix.clone();
            facts.min_len = *min_length;
            facts.char_must = Some(must_chars.clone());
            facts.char_may = Some(may_chars.clone());
            facts.may_include_other_chars = Some(*may_include_other_chars);
            facts
        }
        OmenaCheckerAbstractClassValueInputV0::Bottom => {
            let mut facts = external_string_type_facts("finiteSet");
            facts.values = Some(Vec::new());
            facts
        }
        OmenaCheckerAbstractClassValueInputV0::Top => external_string_type_facts("top"),
    }
}

fn external_string_type_facts(kind: impl Into<String>) -> ExternalStringTypeFactsV0 {
    ExternalStringTypeFactsV0 {
        kind: kind.into(),
        constraint_kind: None,
        values: None,
        prefix: None,
        suffix: None,
        min_len: None,
        max_len: None,
        char_must: None,
        char_may: None,
        may_include_other_chars: None,
    }
}

fn summarize_omena_checker_cascade_evaluations(
    input: OmenaCheckerCascadeInputV0,
) -> OmenaCheckerCascadeEvaluationRunnerOutputV0 {
    let declaration_count = input.declarations.len();
    let custom_property_count = input.custom_properties.len();
    let evaluations = evaluate_omena_checker_cascade_rules(input);
    let rule_code_names = evaluations
        .iter()
        .map(|evaluation| evaluation.rule_code_name)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();

    OmenaCheckerCascadeEvaluationRunnerOutputV0 {
        schema_version: "0",
        product: "omena-checker.cascade-evaluations",
        declaration_count,
        custom_property_count,
        evaluation_count: evaluations.len(),
        rule_code_names,
        evaluations,
    }
}

fn summarize_omena_checker_grn_evaluations(
    input: OmenaCheckerGrnInputV0,
) -> OmenaCheckerGrnEvaluationRunnerOutputV0 {
    let vertex_count = input.vertices.len();
    let evaluations = evaluate_omena_checker_grn_rules(input);
    let rule_code_names = evaluations
        .iter()
        .map(|evaluation| evaluation.rule_code_name)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();

    OmenaCheckerGrnEvaluationRunnerOutputV0 {
        schema_version: "0",
        product: "omena-checker.grn-evaluations",
        vertex_count,
        evaluation_count: evaluations.len(),
        rule_code_names,
        evaluations,
    }
}

fn summarize_omena_checker_smt_evaluations(
    input: OmenaCheckerSmtInputV0,
) -> OmenaCheckerSmtEvaluationRunnerOutputV0 {
    let obligation_count = input.obligations.len();
    let evaluations = evaluate_omena_checker_smt_rules(input);
    let rule_code_names = evaluations
        .iter()
        .map(|evaluation| evaluation.rule_code_name)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();

    OmenaCheckerSmtEvaluationRunnerOutputV0 {
        schema_version: "0",
        product: "omena-checker.smt-evaluations",
        obligation_count,
        evaluation_count: evaluations.len(),
        rule_code_names,
        evaluations,
    }
}

fn summarize_omena_checker_mdl_evaluations(
    input: OmenaCheckerMdlEvaluationInputV0,
) -> OmenaCheckerMdlEvaluationRunnerOutputV0 {
    let mdl_summary = summarize_omena_query_design_system_minimum_description(
        input.source_uri.clone(),
        input.source_hash,
        input.rule_count,
        input.observation_count,
        &input.value_frequencies,
    );
    let evaluations = evaluate_omena_checker_mdl_rules(OmenaCheckerMdlInputV0 {
        summaries: vec![OmenaCheckerMdlSummaryInputV0 {
            source_uri: input.source_uri.clone(),
            total_bits: mdl_summary.total_bits,
            budget_bits: input.budget_bits,
        }],
    });
    let rule_code_names = evaluations
        .iter()
        .map(|evaluation| evaluation.rule_code_name)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();

    OmenaCheckerMdlEvaluationRunnerOutputV0 {
        schema_version: "0",
        product: "omena-checker.mdl-evaluations",
        source_uri: input.source_uri,
        total_bits: mdl_summary.total_bits,
        budget_bits: input.budget_bits,
        evaluation_count: evaluations.len(),
        rule_code_names,
        evaluations,
    }
}

fn summarize_omena_checker_streaming_ifds_evaluations(
    input: OmenaCheckerStreamingIfdsEvaluationInputV0,
) -> OmenaCheckerStreamingIfdsEvaluationRunnerOutputV0 {
    let hyperedges = input
        .hyperedges
        .into_iter()
        .map(streaming_ifds_hyperedge)
        .collect::<Vec<_>>();
    let events = input
        .events
        .into_iter()
        .map(|event| {
            streaming_ifds_event_input_v0(
                event.event_id,
                event.revision,
                event.node_id,
                event.value.into_abstract_class_value(),
                event.refinement_context_digest,
            )
        })
        .collect::<Vec<_>>();
    let previous_cache = if input.previous_fact_keys.is_empty() {
        Vec::new()
    } else {
        vec![streaming_ifds_summary_cache_entry_v0(
            input.start_node_id.clone(),
            Vec::new(),
            input.previous_fact_keys.clone(),
            true,
        )]
    };
    let report = run_streaming_ifds_exact_v0(
        input.update_id.clone(),
        input.start_node_id,
        &hyperedges,
        &events,
        &PolylogDynamicConnectivityBackendV0::default(),
        (!previous_cache.is_empty()).then_some(previous_cache.as_slice()),
    );
    let evaluations =
        evaluate_omena_checker_streaming_ifds_rules(OmenaCheckerStreamingIfdsInputV0 {
            reports: vec![OmenaCheckerStreamingIfdsReportInputV0 {
                report_id: input.update_id,
                precision_parity_with_batch: report.precision_parity_with_batch,
                fallback_to_batch: report.fallback_to_batch,
            }],
        });
    let rule_code_names = evaluations
        .iter()
        .map(|evaluation| evaluation.rule_code_name)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();

    OmenaCheckerStreamingIfdsEvaluationRunnerOutputV0 {
        schema_version: "0",
        product: "omena-checker.streaming-ifds-evaluations",
        report_product: report.product,
        event_count: report.event_count,
        output_fact_count: report.output_fact_count,
        precision_parity_with_batch: report.precision_parity_with_batch,
        evaluation_count: evaluations.len(),
        rule_code_names,
        evaluations,
    }
}

fn summarize_omena_checker_rg_flow_evaluations(
    input: OmenaCheckerRgFlowEvaluationInputV0,
) -> OmenaCheckerRgFlowEvaluationRunnerOutputV0 {
    let flow_count = input.flows.len();
    let checker_input = OmenaCheckerRgFlowInputV0 {
        flows: input
            .flows
            .into_iter()
            .map(|flow| OmenaCheckerRgFlowCouplingInputV0 {
                workspace_path: flow.workspace_path,
                before: flow.before,
                after: flow.after,
            })
            .collect(),
    };
    let evaluations = evaluate_omena_checker_rg_flow_rules(checker_input);
    let rule_code_names = evaluations
        .iter()
        .map(|evaluation| evaluation.rule_code_name)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();

    OmenaCheckerRgFlowEvaluationRunnerOutputV0 {
        schema_version: "0",
        product: "omena-checker.rg-flow-evaluations",
        flow_count,
        evaluation_count: evaluations.len(),
        rule_code_names,
        evaluations,
    }
}

fn summarize_omena_checker_categorical_evaluations(
    input: OmenaCheckerCategoricalEvaluationInputV0,
) -> OmenaCheckerCategoricalEvaluationRunnerOutputV0 {
    let mapping_count = input.mappings.len();
    let checker_input = OmenaCheckerCategoricalInputV0 {
        mappings: input
            .mappings
            .into_iter()
            .map(|mapping| OmenaCheckerCategoricalRoleMappingInputV0 {
                mapping_id: mapping.mapping_id,
                primitive_role_pairs: mapping
                    .primitive_role_pairs
                    .into_iter()
                    .map(|pair| OmenaCheckerCategoricalPrimitiveRolePairInputV0 {
                        primitive_name: pair.primitive_name,
                        categorical_role: pair.categorical_role,
                    })
                    .collect(),
            })
            .collect(),
    };
    let evaluations = evaluate_omena_checker_categorical_rules(checker_input);
    let rule_code_names = evaluations
        .iter()
        .map(|evaluation| evaluation.rule_code_name)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();

    OmenaCheckerCategoricalEvaluationRunnerOutputV0 {
        schema_version: "0",
        product: "omena-checker.categorical-evaluations",
        mapping_count,
        evaluation_count: evaluations.len(),
        rule_code_names,
        evaluations,
    }
}

fn summarize_omena_checker_replica_ensemble_evaluations(
    input: OmenaCheckerReplicaEnsembleEvaluationInputV0,
) -> OmenaCheckerReplicaEnsembleEvaluationRunnerOutputV0 {
    let replicas = input
        .replicas
        .into_iter()
        .map(replica_ensemble_snapshot)
        .collect::<Vec<_>>();
    let graph =
        replica_ensemble_module_graph(input.workspace_root.as_str(), &replicas, input.graph_edges);
    let report = build_cross_file_inconsistency_report(
        input.workspace_root.as_str(),
        replicas,
        &graph,
        OutcomeMode::DefiniteOnly,
        ReportOptionsV0::default(),
        None,
    );
    let recommendation = replica_ensemble_recommendation_name(report.recommendation);
    let top_disagreement_pair_count = report.top_disagreement_pairs.len();
    let evaluations =
        evaluate_omena_checker_replica_ensemble_rules(OmenaCheckerReplicaEnsembleInputV0 {
            reports: vec![OmenaCheckerReplicaEnsembleReportInputV0 {
                workspace_root: input.workspace_root.clone(),
                recommendation: recommendation.to_string(),
                mean_q: report.distribution.mean_q,
                variance_q: report.distribution.variance_q,
                top_disagreement_pair_count,
                mechanism_scope: report.mechanism_scope.to_string(),
                product_surface: report.product_surface.to_string(),
                default_product_decision_mechanism: report.default_product_decision_mechanism,
            }],
        });
    let rule_code_names = evaluations
        .iter()
        .map(|evaluation| evaluation.rule_code_name)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();

    OmenaCheckerReplicaEnsembleEvaluationRunnerOutputV0 {
        schema_version: "0",
        product: "omena-checker.replica-ensemble-evaluations",
        report_product: report.product,
        workspace_root: input.workspace_root,
        replica_count: report.distribution.replica_count,
        pair_count: report.distribution.pair_count,
        mean_q: report.distribution.mean_q,
        variance_q: report.distribution.variance_q,
        recommendation,
        top_disagreement_pair_count,
        evaluation_count: evaluations.len(),
        rule_code_names,
        evaluations,
    }
}

fn replica_ensemble_snapshot(input: ReplicaEnsembleSnapshotRunnerInputV0) -> ReplicaSnapshotV0 {
    let sites = input
        .winners
        .into_iter()
        .enumerate()
        .map(|(index, winner)| ReplicaSiteOutcomeV0 {
            schema_version: REPLICA_ENSEMBLE_SCHEMA_VERSION_V0,
            product: "omena-ensemble.replica-site-outcome",
            layer_marker: REPLICA_ENSEMBLE_LAYER_MARKER_V0,
            feature_gate: REPLICA_ENSEMBLE_FEATURE_GATE_V0,
            site: site(format!(".item-{index}"), "color"),
            outcome: replica_ensemble_definite_outcome(&winner, index as u32),
            provenance: None,
        })
        .collect();

    ReplicaSnapshotV0 {
        schema_version: REPLICA_ENSEMBLE_SCHEMA_VERSION_V0,
        product: "omena-ensemble.replica-snapshot",
        layer_marker: REPLICA_ENSEMBLE_LAYER_MARKER_V0,
        feature_gate: REPLICA_ENSEMBLE_FEATURE_GATE_V0,
        path: input.path,
        sites,
    }
}

fn replica_ensemble_module_graph(
    workspace_root: &str,
    replicas: &[ReplicaSnapshotV0],
    edges: Vec<ReplicaEnsembleGraphEdgeRunnerInputV0>,
) -> ModuleGraphV0 {
    ModuleGraphV0 {
        schema_version: REPLICA_ENSEMBLE_SCHEMA_VERSION_V0,
        product: "omena-ensemble.module-graph",
        layer_marker: REPLICA_ENSEMBLE_LAYER_MARKER_V0,
        feature_gate: REPLICA_ENSEMBLE_FEATURE_GATE_V0,
        workspace_root: workspace_root.to_string(),
        nodes: replicas
            .iter()
            .map(|replica| replica.path.clone())
            .collect(),
        edges: edges
            .into_iter()
            .map(|edge| ModuleGraphEdgeV0 {
                schema_version: REPLICA_ENSEMBLE_SCHEMA_VERSION_V0,
                product: "omena-ensemble.module-graph-edge",
                layer_marker: REPLICA_ENSEMBLE_LAYER_MARKER_V0,
                feature_gate: REPLICA_ENSEMBLE_FEATURE_GATE_V0,
                from_module: edge.from_module,
                to_module: edge.to_module,
                edge_kind: replica_ensemble_edge_kind(edge.edge_kind.as_deref()),
            })
            .collect(),
    }
}

fn replica_ensemble_definite_outcome(winner: &str, source_order: u32) -> CascadeOutcome {
    let declaration = CascadeDeclaration {
        id: winner.to_string(),
        property: "color".to_string(),
        value: CascadeValue::Literal(winner.to_string()),
        key: CascadeKey {
            level: CascadeLevel::AuthorNormal,
            layer_rank: LayerRank(0),
            scope_proximity: 0,
            specificity: Specificity {
                ids: 0,
                classes: 1,
                elements: 0,
            },
            module_rank: ModuleRank::ZERO,
            source_order,
        },
    };
    CascadeOutcome::Definite {
        proof: Box::new(CascadeProof::from_declaration(&declaration)),
        winner: declaration,
        also_considered: Vec::new(),
    }
}

fn replica_ensemble_edge_kind(value: Option<&str>) -> &'static str {
    match value {
        Some("composes") => "composes",
        Some("analyzedGraphEdge") => "analyzedGraphEdge",
        Some("fixture") | None => "fixture",
        Some(_) => "fixture",
    }
}

fn replica_ensemble_recommendation_name(recommendation: ReportRecommendation) -> &'static str {
    match recommendation {
        ReportRecommendation::NoActionNeeded => "noActionNeeded",
        ReportRecommendation::InvestigateRsbBroken => "investigateRsbBroken",
        ReportRecommendation::UndetectablePhase => "undetectablePhase",
    }
}

fn streaming_ifds_hyperedge(input: StreamingIfdsHyperedgeInputV0) -> UnifiedHypergraphHyperedgeV0 {
    let edge_kind = streaming_ifds_edge_kind(input.edge_kind.as_deref());
    UnifiedHypergraphHyperedgeV0 {
        schema_version: "0",
        product: "omena-query.unified-hypergraph-edge",
        layer_marker: "hypergraph-ifds",
        feature_gate: "hypergraph-ifds",
        source_summary_edge_id: input.hyperedge_id.clone(),
        source_edge_kind: edge_kind.as_wire_label(),
        source_status: "resolved",
        order_significant_tail: edge_kind.is_order_significant(),
        hyperedge_id: input.hyperedge_id,
        edge_kind,
        tail_node_ids: vec![input.from],
        head_node_id: input.to,
    }
}

fn streaming_ifds_edge_kind(value: Option<&str>) -> UnifiedHypergraphEdgeKindV0 {
    match value {
        Some("composesLocal") => UnifiedHypergraphEdgeKindV0::ComposesLocal,
        Some("composesGlobal") => UnifiedHypergraphEdgeKindV0::ComposesGlobal,
        Some("composesExternal") => UnifiedHypergraphEdgeKindV0::ComposesExternal,
        Some("sassUse") => UnifiedHypergraphEdgeKindV0::SassUse,
        Some("sassForward") => UnifiedHypergraphEdgeKindV0::SassForward,
        Some("sassImport") => UnifiedHypergraphEdgeKindV0::SassImport,
        Some("lessImport") => UnifiedHypergraphEdgeKindV0::LessImport,
        Some("lessModuleGraphClosure") => UnifiedHypergraphEdgeKindV0::LessModuleGraphClosure,
        Some("value") => UnifiedHypergraphEdgeKindV0::Value,
        Some("icss") => UnifiedHypergraphEdgeKindV0::Icss,
        Some("foreignReference") => UnifiedHypergraphEdgeKindV0::ForeignReference,
        _ => UnifiedHypergraphEdgeKindV0::ForeignReference,
    }
}

fn summarize(payload: ShadowPayloadV0) -> ShadowSummaryV0 {
    let mut query_kind_counts = BTreeMap::new();
    let mut expression_value_domain_kinds = BTreeMap::new();
    let mut expression_value_constraint_kinds = BTreeMap::new();
    let mut expression_constraint_detail_counts = ConstraintDetailCounts::default();
    let mut expression_value_certainty_shapes = BTreeMap::new();
    let mut expression_selector_certainty_shapes = BTreeMap::new();
    let mut resolution_value_constraint_kinds = BTreeMap::new();
    let mut resolution_constraint_detail_counts = ConstraintDetailCounts::default();
    let mut resolution_value_certainty_shapes = BTreeMap::new();
    let mut resolution_selector_certainty_shapes = BTreeMap::new();
    let mut selector_usage_referenced_count = 0usize;
    let mut selector_usage_unreferenced_count = 0usize;
    let mut selector_usage_total_references = 0usize;
    let mut selector_usage_direct_references = 0usize;
    let mut selector_usage_editable_direct_references = 0usize;
    let mut selector_usage_exact_references = 0usize;
    let mut selector_usage_inferred_or_better_references = 0usize;
    let mut selector_usage_expanded_count = 0usize;
    let mut selector_usage_style_dependency_count = 0usize;
    let input = payload.input;
    let output = payload.output;
    let type_fact_summary = summarize_type_fact_input(&input);
    let expected_expression_ids: std::collections::BTreeSet<String> = input
        .sources
        .iter()
        .flat_map(|source| source.document.class_expressions.iter())
        .map(|expression| expression.id.clone())
        .collect();
    let expected_selector_usage_ids: std::collections::BTreeSet<String> = input
        .styles
        .iter()
        .flat_map(|style| style.document.selectors.iter())
        .filter(|selector| selector.view_kind == "canonical")
        .filter_map(|selector| selector.canonical_name.as_ref())
        .map(|name| name.to_string())
        .collect();
    let mut expression_semantics_ids = std::collections::BTreeSet::new();
    let mut resolution_ids = std::collections::BTreeSet::new();
    let mut selector_usage_ids = std::collections::BTreeSet::new();
    let expected_expression_semantics_count: usize = input
        .sources
        .iter()
        .map(|source| source.document.class_expressions.len())
        .sum();
    let expected_source_expression_resolution_count = expected_expression_semantics_count;
    let expected_selector_usage_count: usize = input
        .styles
        .iter()
        .map(|style| {
            style
                .document
                .selectors
                .iter()
                .filter(|selector| selector.view_kind == "canonical")
                .count()
        })
        .sum();
    let expected_total_query_count = expected_expression_semantics_count
        + expected_source_expression_resolution_count
        + expected_selector_usage_count;

    for query in &output.query_results {
        match query {
            QueryResultV2::ExpressionSemantics {
                query_id, payload, ..
            } => {
                *query_kind_counts
                    .entry("expression-semantics".to_string())
                    .or_insert(0) += 1;
                expression_semantics_ids.insert(query_id.clone());
                *expression_value_domain_kinds
                    .entry(value_domain_kind_string(&payload.value_domain_kind))
                    .or_insert(0) += 1;

                if let Some(constraint_kind) = &payload.value_constraint_kind {
                    *expression_value_constraint_kinds
                        .entry(string_constraint_kind_string(constraint_kind))
                        .or_insert(0) += 1;
                }
                collect_constraint_detail_counts(
                    &mut expression_constraint_detail_counts,
                    ConstraintDetailInput {
                        prefix: payload.value_prefix.as_ref(),
                        suffix: payload.value_suffix.as_ref(),
                        min_len: optional_nonnegative_i32_to_usize(
                            "expression valueMinLen",
                            payload.value_min_len,
                        ),
                        max_len: optional_nonnegative_i32_to_usize(
                            "expression valueMaxLen",
                            payload.value_max_len,
                        ),
                        char_must: payload.value_char_must.as_ref(),
                        char_may: payload.value_char_may.as_ref(),
                        may_include_other_chars: payload.value_may_include_other_chars,
                    },
                );

                if let Some(shape_kind) = &payload.value_certainty_shape_kind {
                    *expression_value_certainty_shapes
                        .entry(certainty_shape_kind_string(shape_kind))
                        .or_insert(0) += 1;
                }

                if let Some(shape_kind) = &payload.selector_certainty_shape_kind {
                    *expression_selector_certainty_shapes
                        .entry(certainty_shape_kind_string(shape_kind))
                        .or_insert(0) += 1;
                }
            }
            QueryResultV2::SourceExpressionResolution {
                query_id, payload, ..
            } => {
                *query_kind_counts
                    .entry("source-expression-resolution".to_string())
                    .or_insert(0) += 1;
                resolution_ids.insert(query_id.clone());

                if let Some(constraint_kind) = &payload.value_certainty_constraint_kind {
                    *resolution_value_constraint_kinds
                        .entry(string_constraint_kind_string(constraint_kind))
                        .or_insert(0) += 1;
                }
                collect_constraint_detail_counts(
                    &mut resolution_constraint_detail_counts,
                    ConstraintDetailInput {
                        prefix: payload.value_prefix.as_ref(),
                        suffix: payload.value_suffix.as_ref(),
                        min_len: optional_nonnegative_i32_to_usize(
                            "source resolution valueMinLen",
                            payload.value_min_len,
                        ),
                        max_len: optional_nonnegative_i32_to_usize(
                            "source resolution valueMaxLen",
                            payload.value_max_len,
                        ),
                        char_must: payload.value_char_must.as_ref(),
                        char_may: payload.value_char_may.as_ref(),
                        may_include_other_chars: payload.value_may_include_other_chars,
                    },
                );

                if let Some(shape_kind) = &payload.value_certainty_shape_kind {
                    *resolution_value_certainty_shapes
                        .entry(certainty_shape_kind_string(shape_kind))
                        .or_insert(0) += 1;
                }

                if let Some(shape_kind) = &payload.selector_certainty_shape_kind {
                    *resolution_selector_certainty_shapes
                        .entry(certainty_shape_kind_string(shape_kind))
                        .or_insert(0) += 1;
                }
            }
            QueryResultV2::SelectorUsage {
                query_id, payload, ..
            } => {
                *query_kind_counts
                    .entry("selector-usage".to_string())
                    .or_insert(0) += 1;
                selector_usage_ids.insert(query_id.clone());

                selector_usage_total_references +=
                    nonnegative_i32_to_usize("selector totalReferences", payload.total_references);
                selector_usage_direct_references += nonnegative_i32_to_usize(
                    "selector directReferenceCount",
                    payload.direct_reference_count,
                );
                selector_usage_editable_direct_references += nonnegative_i32_to_usize(
                    "selector editableDirectReferenceCount",
                    payload.editable_direct_reference_count,
                );
                selector_usage_exact_references += nonnegative_i32_to_usize(
                    "selector exactReferenceCount",
                    payload.exact_reference_count,
                );
                selector_usage_inferred_or_better_references += nonnegative_i32_to_usize(
                    "selector inferredOrBetterReferenceCount",
                    payload.inferred_or_better_reference_count,
                );

                if payload.has_expanded_references {
                    selector_usage_expanded_count += 1;
                }
                if payload.has_style_dependency_references {
                    selector_usage_style_dependency_count += 1;
                }
                if payload.has_any_references {
                    selector_usage_referenced_count += 1;
                } else {
                    selector_usage_unreferenced_count += 1;
                }
            }
        }
    }

    let matched_expression_query_pairs = expected_expression_ids
        .iter()
        .filter(|id| expression_semantics_ids.contains(*id) && resolution_ids.contains(*id))
        .count();
    let missing_expression_semantics_count = expected_expression_ids
        .iter()
        .filter(|id| !expression_semantics_ids.contains(*id))
        .count();
    let missing_source_expression_resolution_count = expected_expression_ids
        .iter()
        .filter(|id| !resolution_ids.contains(*id))
        .count();
    let unexpected_expression_semantics_count = expression_semantics_ids
        .iter()
        .filter(|id| !expected_expression_ids.contains(*id))
        .count();
    let unexpected_source_expression_resolution_count = resolution_ids
        .iter()
        .filter(|id| !expected_expression_ids.contains(*id))
        .count();
    let matched_selector_usage_count = expected_selector_usage_ids
        .iter()
        .filter(|id| selector_usage_ids.contains(*id))
        .count();
    let missing_selector_usage_count = expected_selector_usage_ids
        .iter()
        .filter(|id| !selector_usage_ids.contains(*id))
        .count();
    let unexpected_selector_usage_count = selector_usage_ids
        .iter()
        .filter(|id| !expected_selector_usage_ids.contains(*id))
        .count();

    ShadowSummaryV0 {
        schema_version: "0",
        input_version: type_fact_summary.input_version,
        source_count: input.sources.len(),
        style_count: input.styles.len(),
        type_fact_count: type_fact_summary.type_fact_count,
        distinct_fact_files: type_fact_summary.distinct_fact_files,
        by_kind: type_fact_summary.by_kind,
        constrained_kinds: type_fact_summary.constrained_kinds,
        finite_value_count: type_fact_summary.finite_value_count,
        query_result_count: output.query_results.len(),
        query_kind_counts,
        expression_value_domain_kinds,
        expression_value_constraint_kinds,
        expression_constraint_detail_counts,
        expression_value_certainty_shapes,
        expression_selector_certainty_shapes,
        resolution_value_constraint_kinds,
        resolution_constraint_detail_counts,
        resolution_value_certainty_shapes,
        resolution_selector_certainty_shapes,
        selector_usage_referenced_count,
        selector_usage_unreferenced_count,
        selector_usage_total_references,
        selector_usage_direct_references,
        selector_usage_editable_direct_references,
        selector_usage_exact_references,
        selector_usage_inferred_or_better_references,
        selector_usage_expanded_count,
        selector_usage_style_dependency_count,
        expected_expression_semantics_count,
        expected_source_expression_resolution_count,
        expected_selector_usage_count,
        expected_total_query_count,
        matched_expression_query_pairs,
        missing_expression_semantics_count,
        missing_source_expression_resolution_count,
        unexpected_expression_semantics_count,
        unexpected_source_expression_resolution_count,
        matched_selector_usage_count,
        missing_selector_usage_count,
        unexpected_selector_usage_count,
        rewrite_plan_count: output.rewrite_plans.len(),
        checker_warning_count: checker_summary_count(&output.checker_report, "warnings"),
        checker_hint_count: checker_summary_count(&output.checker_report, "hints"),
        checker_total_findings: checker_summary_count(&output.checker_report, "total"),
    }
}

fn summarize_checker_style_recovery_canonical_candidate(
    payload: ShadowPayloadV0,
) -> CheckerStyleRecoveryCanonicalCandidateBundleV0 {
    let input_version = payload.input.version.to_string();
    let report = payload.output.checker_report;
    let report_version = report.version.clone();
    let mut code_counts = BTreeMap::new();
    let mut file_paths = std::collections::BTreeSet::new();
    let mut findings = Vec::new();
    let mut warnings = 0usize;
    let mut hints = 0usize;

    for finding in checker_findings(report.findings) {
        if finding.category != "style" {
            continue;
        }
        if !matches!(
            finding.code.as_str(),
            "missing-composed-module"
                | "missing-composed-selector"
                | "missing-value-module"
                | "missing-imported-value"
                | "missing-keyframes"
                | "missing-sass-symbol"
        ) {
            continue;
        }

        *code_counts.entry(finding.code.clone()).or_insert(0) += 1;
        file_paths.insert(finding.file_path.clone());
        if finding.severity == "warning" {
            warnings += 1;
        }
        if finding.severity == "hint" {
            hints += 1;
        }
        findings.push(CheckerStyleRecoveryFindingV0 {
            file_path: finding.file_path,
            code: finding.code,
            severity: finding.severity,
            range: finding.range,
            message: finding.message,
            analysis_reason: finding.analysis_reason,
            value_certainty_shape_label: finding.value_certainty_shape_label,
        });
    }

    findings.sort();

    CheckerStyleRecoveryCanonicalCandidateBundleV0 {
        schema_version: "0",
        input_version,
        report_version,
        bundle: "style-recovery",
        distinct_file_count: file_paths.len(),
        code_counts,
        summary: CheckerReportSummaryV1 {
            warnings,
            hints,
            total: findings.len(),
        },
        findings,
    }
}

fn summarize_checker_style_recovery_canonical_producer(
    payload: ShadowPayloadV0,
) -> CheckerStyleRecoveryCanonicalProducerSignalV0 {
    let canonical_candidate = summarize_checker_style_recovery_canonical_candidate(payload);
    CheckerStyleRecoveryCanonicalProducerSignalV0 {
        schema_version: "0",
        input_version: canonical_candidate.input_version.clone(),
        canonical_candidate,
        bounded_checker_gate: CheckerStyleRecoveryCanonicalProducerGateV0 {
            canonical_candidate_command: "pnpm check:rust-checker-style-recovery-canonical-candidate",
            canonical_producer_command: "pnpm check:rust-checker-style-recovery-canonical-producer",
            consumer_boundary_command: "pnpm check:rust-checker-style-recovery-consumer-boundary",
            bounded_checker_lane_command: "pnpm check:rust-checker-bounded-lanes",
            promotion_review_command: "pnpm check:rust-checker-promotion-review",
            promotion_evidence_command: "pnpm check:rust-checker-promotion-evidence",
            broader_rust_lane_command: "pnpm check:rust-lane-bundle",
            release_gate_readiness_command: "pnpm check:rust-checker-release-gate-readiness",
            release_gate_shadow_command: "pnpm check:rust-checker-release-gate-shadow",
            release_gate_shadow_review_command: "pnpm check:rust-checker-release-gate-shadow-review",
            release_bundle_command: "pnpm check:rust-release-bundle",
            minimum_bounded_lane_count_for_rust_lane_bundle: 3,
            minimum_bounded_lane_count_for_rust_release_bundle: 3,
            minimum_successful_shadow_runs_for_rust_release_bundle: 3,
            checker_bundle: "style-recovery",
            release_gate_stage: "enforced",
            included_in_rust_lane_bundle: true,
            included_in_rust_release_bundle: true,
        },
    }
}

fn summarize_checker_source_missing_canonical_candidate(
    payload: ShadowPayloadV0,
) -> CheckerSourceMissingCanonicalCandidateBundleV0 {
    let input_version = payload.input.version.clone();
    let report = payload.output.checker_report;
    let report_version = report.version.clone();
    let mut code_counts = BTreeMap::new();
    let mut file_paths = std::collections::BTreeSet::new();
    let mut findings = Vec::new();
    let mut warnings = 0usize;
    let mut hints = 0usize;

    for finding in checker_findings(report.findings) {
        if finding.category != "source" {
            continue;
        }
        if !matches!(
            finding.code.as_str(),
            "missing-module"
                | "missing-static-class"
                | "missing-template-prefix"
                | "missing-resolved-class-values"
                | "missing-resolved-class-domain"
        ) {
            continue;
        }

        *code_counts.entry(finding.code.clone()).or_insert(0) += 1;
        file_paths.insert(finding.file_path.clone());
        if finding.severity == "warning" {
            warnings += 1;
        }
        if finding.severity == "hint" {
            hints += 1;
        }
        findings.push(CheckerSourceMissingFindingV0 {
            file_path: finding.file_path,
            code: finding.code,
            severity: finding.severity,
            range: finding.range,
            message: finding.message,
            analysis_reason: finding.analysis_reason,
            value_certainty_shape_label: finding.value_certainty_shape_label,
            value_domain_derivation: finding.value_domain_derivation,
        });
    }

    findings.sort();

    CheckerSourceMissingCanonicalCandidateBundleV0 {
        schema_version: "0",
        input_version,
        report_version,
        bundle: "source-missing",
        distinct_file_count: file_paths.len(),
        code_counts,
        summary: CheckerReportSummaryV1 {
            warnings,
            hints,
            total: findings.len(),
        },
        findings,
    }
}

fn summarize_checker_source_missing_canonical_producer(
    payload: ShadowPayloadV0,
) -> CheckerSourceMissingCanonicalProducerSignalV0 {
    let flow_summary = summarize_omena_query_expression_domain_flow_analysis(&payload.input);
    let graph_count = flow_summary.analyses.len();
    let node_count = flow_summary
        .analyses
        .iter()
        .map(|entry| entry.analysis.nodes.len())
        .sum::<usize>();
    let converged_graph_count = flow_summary
        .analyses
        .iter()
        .filter(|entry| entry.analysis.converged)
        .count();
    let max_iteration_count = flow_summary
        .analyses
        .iter()
        .map(|entry| entry.analysis.iteration_count)
        .max()
        .unwrap_or(0);
    let canonical_candidate = summarize_checker_source_missing_canonical_candidate(payload);
    CheckerSourceMissingCanonicalProducerSignalV0 {
        schema_version: "0",
        input_version: canonical_candidate.input_version.clone(),
        canonical_candidate,
        flow_evidence: CheckerSourceMissingFlowEvidenceV0 {
            schema_version: "0",
            product: "engine-input-producers.expression-domain-flow-analysis",
            input_version: flow_summary.input_version,
            graph_count,
            node_count,
            converged_graph_count,
            unconverged_graph_count: graph_count - converged_graph_count,
            max_iteration_count,
        },
        bounded_checker_gate: CheckerSourceMissingCanonicalProducerGateV0 {
            canonical_candidate_command: "pnpm check:rust-checker-source-missing-canonical-candidate",
            canonical_producer_command: "pnpm check:rust-checker-source-missing-canonical-producer",
            consumer_boundary_command: "pnpm check:rust-checker-source-missing-consumer-boundary",
            bounded_checker_lane_command: "pnpm check:rust-checker-bounded-lanes",
            promotion_review_command: "pnpm check:rust-checker-promotion-review",
            promotion_evidence_command: "pnpm check:rust-checker-promotion-evidence",
            broader_rust_lane_command: "pnpm check:rust-lane-bundle",
            release_gate_readiness_command: "pnpm check:rust-checker-release-gate-readiness",
            release_gate_shadow_command: "pnpm check:rust-checker-release-gate-shadow",
            release_gate_shadow_review_command: "pnpm check:rust-checker-release-gate-shadow-review",
            release_bundle_command: "pnpm check:rust-release-bundle",
            minimum_bounded_lane_count_for_rust_lane_bundle: 3,
            minimum_bounded_lane_count_for_rust_release_bundle: 3,
            minimum_successful_shadow_runs_for_rust_release_bundle: 3,
            checker_bundle: "source-missing",
            release_gate_stage: "enforced",
            included_in_rust_lane_bundle: true,
            included_in_rust_release_bundle: true,
        },
    }
}

fn summarize_checker_style_unused_canonical_candidate(
    payload: ShadowPayloadV0,
) -> CheckerStyleUnusedCanonicalCandidateBundleV0 {
    let input_version = payload.input.version.clone();
    let report = payload.output.checker_report;
    let report_version = report.version.clone();
    let mut code_counts = BTreeMap::new();
    let mut file_paths = std::collections::BTreeSet::new();
    let mut findings = Vec::new();
    let mut warnings = 0usize;
    let mut hints = 0usize;

    for finding in checker_findings(report.findings) {
        if finding.category != "style" || finding.code != "unused-selector" {
            continue;
        }

        *code_counts.entry(finding.code.clone()).or_insert(0) += 1;
        file_paths.insert(finding.file_path.clone());
        if finding.severity == "warning" {
            warnings += 1;
        }
        if finding.severity == "hint" {
            hints += 1;
        }
        findings.push(CheckerStyleUnusedFindingV0 {
            file_path: finding.file_path,
            code: finding.code,
            severity: finding.severity,
            range: finding.range,
            message: finding.message,
            analysis_reason: finding.analysis_reason,
            value_certainty_shape_label: finding.value_certainty_shape_label,
        });
    }

    findings.sort();

    CheckerStyleUnusedCanonicalCandidateBundleV0 {
        schema_version: "0",
        input_version,
        report_version,
        bundle: "style-unused",
        distinct_file_count: file_paths.len(),
        code_counts,
        summary: CheckerReportSummaryV1 {
            warnings,
            hints,
            total: findings.len(),
        },
        findings,
    }
}

fn summarize_checker_style_unused_canonical_producer(
    payload: ShadowPayloadV0,
) -> CheckerStyleUnusedCanonicalProducerSignalV0 {
    let canonical_candidate = summarize_checker_style_unused_canonical_candidate(payload);
    CheckerStyleUnusedCanonicalProducerSignalV0 {
        schema_version: "0",
        input_version: canonical_candidate.input_version.clone(),
        canonical_candidate,
        bounded_checker_gate: CheckerStyleUnusedCanonicalProducerGateV0 {
            canonical_candidate_command: "pnpm check:rust-checker-style-unused-canonical-candidate",
            canonical_producer_command: "pnpm check:rust-checker-style-unused-canonical-producer",
            consumer_boundary_command: "pnpm check:rust-checker-style-unused-consumer-boundary",
            bounded_checker_lane_command: "pnpm check:rust-checker-bounded-lanes",
            promotion_review_command: "pnpm check:rust-checker-promotion-review",
            promotion_evidence_command: "pnpm check:rust-checker-promotion-evidence",
            broader_rust_lane_command: "pnpm check:rust-lane-bundle",
            release_gate_readiness_command: "pnpm check:rust-checker-release-gate-readiness",
            release_gate_shadow_command: "pnpm check:rust-checker-release-gate-shadow",
            release_gate_shadow_review_command: "pnpm check:rust-checker-release-gate-shadow-review",
            release_bundle_command: "pnpm check:rust-release-bundle",
            minimum_bounded_lane_count_for_rust_lane_bundle: 3,
            minimum_bounded_lane_count_for_rust_release_bundle: 3,
            minimum_successful_shadow_runs_for_rust_release_bundle: 3,
            checker_bundle: "style-unused",
            release_gate_stage: "enforced",
            included_in_rust_lane_bundle: true,
            included_in_rust_release_bundle: true,
        },
    }
}

fn optional_nonnegative_i32_to_usize(label: &str, value: Option<i32>) -> Option<usize> {
    value.map(|inner| nonnegative_i32_to_usize(label, inner))
}

fn nonnegative_i32_to_usize(_label: &str, value: i32) -> usize {
    usize::try_from(value).unwrap_or(0)
}

fn checker_summary_count(report: &CheckerReportJsonV1Json, field: &str) -> usize {
    let Some(raw_value) = report.summary.get(field) else {
        return 0;
    };
    let Some(count) = raw_value.as_u64() else {
        return 0;
    };
    usize::try_from(count).unwrap_or(usize::MAX)
}

fn checker_findings(values: Vec<serde_json::Value>) -> Vec<CheckerFindingRecordV1> {
    values
        .into_iter()
        .filter_map(|value| serde_json::from_value(value).ok())
        .collect()
}

fn value_domain_kind_string(value: &ValueDomainKindV2Json) -> String {
    match value {
        ValueDomainKindV2Json::None => "none",
        ValueDomainKindV2Json::Exact => "exact",
        ValueDomainKindV2Json::FiniteSet => "finiteSet",
        ValueDomainKindV2Json::Constrained => "constrained",
        ValueDomainKindV2Json::Top => "top",
    }
    .to_string()
}

fn string_constraint_kind_string(value: &StringConstraintKindV2Json) -> String {
    match value {
        StringConstraintKindV2Json::Prefix => "prefix",
        StringConstraintKindV2Json::Suffix => "suffix",
        StringConstraintKindV2Json::PrefixSuffix => "prefixSuffix",
        StringConstraintKindV2Json::CharInclusion => "charInclusion",
        StringConstraintKindV2Json::Composite => "composite",
    }
    .to_string()
}

fn certainty_shape_kind_string(value: &CertaintyShapeKindV2Json) -> String {
    match value {
        CertaintyShapeKindV2Json::Exact => "exact",
        CertaintyShapeKindV2Json::BoundedFinite => "boundedFinite",
        CertaintyShapeKindV2Json::Constrained => "constrained",
        CertaintyShapeKindV2Json::Unknown => "unknown",
    }
    .to_string()
}

fn collect_constraint_detail_counts(
    counts: &mut ConstraintDetailCounts,
    details: ConstraintDetailInput<'_>,
) {
    if details.prefix.is_some() {
        counts.prefix_count += 1;
    }
    if details.suffix.is_some() {
        counts.suffix_count += 1;
    }
    if let Some(value) = details.min_len {
        counts.min_len_count += 1;
        counts.min_len_sum += value;
    }
    if let Some(value) = details.max_len {
        counts.max_len_count += 1;
        counts.max_len_sum += value;
    }
    if let Some(value) = details.char_must {
        counts.char_must_count += 1;
        counts.char_must_len_sum += value.len();
    }
    if let Some(value) = details.char_may {
        counts.char_may_count += 1;
        counts.char_may_len_sum += value.len();
    }
    if details.may_include_other_chars == Some(true) {
        counts.may_include_other_chars_count += 1;
    }
}

struct ConstraintDetailInput<'a> {
    prefix: Option<&'a String>,
    suffix: Option<&'a String>,
    min_len: Option<usize>,
    max_len: Option<usize>,
    char_must: Option<&'a String>,
    char_may: Option<&'a String>,
    may_include_other_chars: Option<bool>,
}
