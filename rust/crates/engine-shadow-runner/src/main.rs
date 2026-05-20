use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::io::{self, BufRead, Read, Write};

use engine_input_producers::{
    ConstraintDetailCounts, EngineInputV2, summarize_expression_domain_candidates_input,
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
    AbstractClassValueV0, CompositeClassValueInputV0, bottom_class_value,
    char_inclusion_class_value, composite_class_value, exact_class_value, finite_set_class_value,
    prefix_class_value, prefix_suffix_class_value, suffix_class_value, top_class_value,
};
use omena_checker::{
    OmenaCheckerCascadeEvaluationV0, OmenaCheckerCascadeInputV0,
    OmenaCheckerDynamicClassDomainInputV0, OmenaCheckerMTierEvaluationV0,
    evaluate_omena_checker_cascade_rules, evaluate_omena_checker_m_tier_rules,
};
use omena_query::{
    OmenaParserStyleDialect, OmenaQueryExpressionDomainFlowRuntimeV0,
    OmenaQuerySourceDocumentInputV0, OmenaQueryStylePackageManifestV0,
    OmenaQueryStyleSourceInputV0, OmenaQueryTargetFeatureSupportV0,
    OmenaQueryTargetTransformOptionsV0, OmenaQueryTransformExecutionContextV0, ParserPositionV0,
    default_omena_query_transform_print_options,
    execute_omena_query_consumer_build_style_sources_for_target_query_with_context_and_options,
    execute_omena_query_consumer_build_style_sources_with_context,
    execute_omena_query_transform_passes_from_source_with_context,
    list_omena_query_transform_pass_summaries, read_omena_query_cascade_at_position,
    read_omena_query_style_context_index, summarize_omena_query_boundary,
    summarize_omena_query_consumer_check_style_source, summarize_omena_query_evaluation_runtime,
    summarize_omena_query_expression_domain_control_flow_analysis,
    summarize_omena_query_expression_domain_flow_analysis,
    summarize_omena_query_expression_domain_incremental_flow_analysis,
    summarize_omena_query_expression_domain_selector_projection,
    summarize_omena_query_expression_semantics_canonical_producer_signal,
    summarize_omena_query_expression_semantics_query_fragments,
    summarize_omena_query_omena_parser_css_modules_intermediate,
    summarize_omena_query_omena_parser_lex, summarize_omena_query_omena_parser_style_facts,
    summarize_omena_query_refs_for_workspace_class,
    summarize_omena_query_rename_plan_for_workspace_class,
    summarize_omena_query_selected_query_adapter_capabilities,
    summarize_omena_query_selector_usage_canonical_producer_signal,
    summarize_omena_query_selector_usage_query_fragments,
    summarize_omena_query_source_completion_for_workspace_file,
    summarize_omena_query_source_diagnostics_for_workspace_file,
    summarize_omena_query_source_resolution_canonical_producer_signal,
    summarize_omena_query_source_resolution_query_fragments,
    summarize_omena_query_source_resolution_runtime,
    summarize_omena_query_style_completion_for_workspace_file,
    summarize_omena_query_style_diagnostics_for_workspace_file,
    summarize_omena_query_style_semantic_graph_batch_from_sources_with_package_manifests,
    summarize_omena_query_style_semantic_graph_from_source,
    summarize_omena_query_transform_context_from_engine_input,
    summarize_omena_query_transform_context_from_sources,
    summarize_omena_query_transform_plan_from_source_with_context,
    summarize_omena_query_transform_plan_from_target_query_with_context,
};
use omena_resolver::{
    OmenaResolverModuleGraphSummaryV0, OmenaResolverStylePackageManifestV0,
    OmenaResolverTsconfigPathMappingV0, summarize_omena_resolver_boundary,
    summarize_omena_resolver_module_graph_index, summarize_omena_resolver_runtime_query_boundary,
    summarize_omena_resolver_specifier_resolution_runtime,
    summarize_omena_resolver_style_module_resolution_with_tsconfig_paths,
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
struct EngineOutputV2 {
    query_results: Vec<QueryResultV2>,
    rewrite_plans: Vec<serde_json::Value>,
    checker_report: CheckerReportV1,
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
struct StyleDiagnosticsForFileInputV0 {
    target_style_path: String,
    styles: Vec<OmenaQueryStyleSourceInputV0>,
    #[serde(default)]
    source_documents: Vec<OmenaQuerySourceDocumentInputV0>,
    #[serde(default)]
    package_manifests: Vec<OmenaQueryStylePackageManifestV0>,
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

#[derive(Debug, Deserialize)]
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

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TransformPlanTargetFeatureSupportInputV0 {
    vendor_prefix_required: bool,
    supports_light_dark: bool,
    supports_color_mix: bool,
    supports_oklch_oklab: bool,
    supports_color_function: bool,
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
    tsconfig_path_mappings: Vec<OmenaResolverTsconfigPathMappingInputV0>,
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
struct OmenaCheckerCascadeEvaluationRunnerOutputV0 {
    schema_version: &'static str,
    product: &'static str,
    declaration_count: usize,
    custom_property_count: usize,
    evaluation_count: usize,
    rule_code_names: Vec<&'static str>,
    evaluations: Vec<OmenaCheckerCascadeEvaluationV0>,
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

#[derive(Debug, Deserialize)]
#[serde(tag = "kind")]
enum QueryResultV2 {
    #[serde(rename = "expression-semantics")]
    ExpressionSemantics {
        #[serde(rename = "queryId")]
        query_id: String,
        payload: ExpressionSemanticsPayloadV2,
    },
    #[serde(rename = "source-expression-resolution")]
    SourceExpressionResolution {
        #[serde(rename = "queryId")]
        query_id: String,
        payload: SourceExpressionResolutionPayloadV2,
    },
    #[serde(rename = "selector-usage")]
    SelectorUsage {
        #[serde(rename = "queryId")]
        query_id: String,
        payload: SelectorUsagePayloadV2,
    },
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ExpressionSemanticsPayloadV2 {
    value_domain_kind: String,
    value_constraint_kind: Option<String>,
    value_prefix: Option<String>,
    value_suffix: Option<String>,
    value_min_len: Option<usize>,
    value_max_len: Option<usize>,
    value_char_must: Option<String>,
    value_char_may: Option<String>,
    value_may_include_other_chars: Option<bool>,
    value_certainty_shape_kind: Option<String>,
    selector_certainty_shape_kind: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SourceExpressionResolutionPayloadV2 {
    value_certainty_shape_kind: Option<String>,
    value_certainty_constraint_kind: Option<String>,
    value_prefix: Option<String>,
    value_suffix: Option<String>,
    value_min_len: Option<usize>,
    value_max_len: Option<usize>,
    value_char_must: Option<String>,
    value_char_may: Option<String>,
    value_may_include_other_chars: Option<bool>,
    selector_certainty_shape_kind: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SelectorUsagePayloadV2 {
    total_references: usize,
    direct_reference_count: usize,
    editable_direct_reference_count: usize,
    exact_reference_count: usize,
    inferred_or_better_reference_count: usize,
    has_expanded_references: bool,
    has_style_dependency_references: bool,
    has_any_references: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CheckerReportV1 {
    version: String,
    findings: Vec<CheckerFindingRecordV1>,
    summary: CheckerReportSummaryV1,
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
            let Some(summary) = summarize_omena_query_style_diagnostics_for_workspace_file(
                &input.target_style_path,
                &input.styles,
                &input.source_documents,
                &input.package_manifests,
            ) else {
                return Err("unsupported style module path".into());
            };
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
                    summarize_omena_query_style_completion_for_workspace_file(
                        &input.file_uri,
                        &style_source,
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
            let package_manifests = input
                .package_manifests
                .iter()
                .map(|manifest| OmenaQueryStylePackageManifestV0 {
                    package_json_path: manifest.package_json_path.clone(),
                    package_json_source: manifest.package_json_source.clone(),
                })
                .collect::<Vec<_>>();
            let output =
                summarize_omena_query_style_semantic_graph_batch_from_sources_with_package_manifests(
                input
                    .styles
                    .iter()
                    .map(|style| (style.style_path.as_str(), style.style_source.as_str())),
                &input.engine_input,
                &package_manifests,
            );
            serde_json::to_writer_pretty(io::stdout(), &output)?;
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
            let tsconfig_path_mappings = input
                .tsconfig_path_mappings
                .iter()
                .map(|mapping| OmenaResolverTsconfigPathMappingV0 {
                    base_path: mapping.base_path.clone(),
                    pattern: mapping.pattern.clone(),
                    target_patterns: mapping.target_patterns.clone(),
                })
                .collect::<Vec<_>>();
            let summary = summarize_omena_resolver_style_module_resolution_with_tsconfig_paths(
                &input.from_style_path,
                &input.source,
                &available_style_paths,
                &package_manifests,
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
            let tsconfig_path_mappings = input
                .tsconfig_path_mappings
                .iter()
                .map(|mapping| OmenaResolverTsconfigPathMappingV0 {
                    base_path: mapping.base_path.clone(),
                    pattern: mapping.pattern.clone(),
                    target_patterns: mapping.target_patterns.clone(),
                })
                .collect::<Vec<_>>();
            let summary = summarize_omena_resolver_specifier_resolution_runtime(
                &input.from_style_path,
                &input.sources,
                &available_style_paths,
                &package_manifests,
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
        Some("omena-checker-cascade-evaluations") => {
            let input: OmenaCheckerCascadeInputV0 = serde_json::from_str(&stdin)?;
            let summary = summarize_omena_checker_cascade_evaluations(input);
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
        "style-semantic-graph-batch" => {
            let input: StyleSemanticGraphBatchInputV0 = serde_json::from_value(input)?;
            let package_manifests = input
                .package_manifests
                .iter()
                .map(|manifest| OmenaQueryStylePackageManifestV0 {
                    package_json_path: manifest.package_json_path.clone(),
                    package_json_source: manifest.package_json_source.clone(),
                })
                .collect::<Vec<_>>();
            Ok(serde_json::to_value(
                summarize_omena_query_style_semantic_graph_batch_from_sources_with_package_manifests(
                    input
                        .styles
                        .iter()
                        .map(|style| (style.style_path.as_str(), style.style_source.as_str())),
                    &input.engine_input,
                    &package_manifests,
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
                summarize_omena_resolver_style_module_resolution_with_tsconfig_paths(
                    &input.from_style_path,
                    &input.source,
                    &available_style_paths,
                    &package_manifests,
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
                summarize_omena_resolver_specifier_resolution_runtime(
                    &input.from_style_path,
                    &input.sources,
                    &available_style_paths,
                    &package_manifests,
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
        "omena-checker-cascade-evaluations" => {
            let input: OmenaCheckerCascadeInputV0 = serde_json::from_value(input)?;
            Ok(serde_json::to_value(
                summarize_omena_checker_cascade_evaluations(input),
            )?)
        }
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
            QueryResultV2::ExpressionSemantics { query_id, payload } => {
                *query_kind_counts
                    .entry("expression-semantics".to_string())
                    .or_insert(0) += 1;
                expression_semantics_ids.insert(query_id.clone());
                *expression_value_domain_kinds
                    .entry(payload.value_domain_kind.clone())
                    .or_insert(0) += 1;

                if let Some(constraint_kind) = &payload.value_constraint_kind {
                    *expression_value_constraint_kinds
                        .entry(constraint_kind.clone())
                        .or_insert(0) += 1;
                }
                collect_constraint_detail_counts(
                    &mut expression_constraint_detail_counts,
                    ConstraintDetailInput {
                        prefix: payload.value_prefix.as_ref(),
                        suffix: payload.value_suffix.as_ref(),
                        min_len: payload.value_min_len,
                        max_len: payload.value_max_len,
                        char_must: payload.value_char_must.as_ref(),
                        char_may: payload.value_char_may.as_ref(),
                        may_include_other_chars: payload.value_may_include_other_chars,
                    },
                );

                if let Some(shape_kind) = &payload.value_certainty_shape_kind {
                    *expression_value_certainty_shapes
                        .entry(shape_kind.clone())
                        .or_insert(0) += 1;
                }

                if let Some(shape_kind) = &payload.selector_certainty_shape_kind {
                    *expression_selector_certainty_shapes
                        .entry(shape_kind.clone())
                        .or_insert(0) += 1;
                }
            }
            QueryResultV2::SourceExpressionResolution { query_id, payload } => {
                *query_kind_counts
                    .entry("source-expression-resolution".to_string())
                    .or_insert(0) += 1;
                resolution_ids.insert(query_id.clone());

                if let Some(constraint_kind) = &payload.value_certainty_constraint_kind {
                    *resolution_value_constraint_kinds
                        .entry(constraint_kind.clone())
                        .or_insert(0) += 1;
                }
                collect_constraint_detail_counts(
                    &mut resolution_constraint_detail_counts,
                    ConstraintDetailInput {
                        prefix: payload.value_prefix.as_ref(),
                        suffix: payload.value_suffix.as_ref(),
                        min_len: payload.value_min_len,
                        max_len: payload.value_max_len,
                        char_must: payload.value_char_must.as_ref(),
                        char_may: payload.value_char_may.as_ref(),
                        may_include_other_chars: payload.value_may_include_other_chars,
                    },
                );

                if let Some(shape_kind) = &payload.value_certainty_shape_kind {
                    *resolution_value_certainty_shapes
                        .entry(shape_kind.clone())
                        .or_insert(0) += 1;
                }

                if let Some(shape_kind) = &payload.selector_certainty_shape_kind {
                    *resolution_selector_certainty_shapes
                        .entry(shape_kind.clone())
                        .or_insert(0) += 1;
                }
            }
            QueryResultV2::SelectorUsage { query_id, payload } => {
                *query_kind_counts
                    .entry("selector-usage".to_string())
                    .or_insert(0) += 1;
                selector_usage_ids.insert(query_id.clone());

                selector_usage_total_references += payload.total_references;
                selector_usage_direct_references += payload.direct_reference_count;
                selector_usage_editable_direct_references +=
                    payload.editable_direct_reference_count;
                selector_usage_exact_references += payload.exact_reference_count;
                selector_usage_inferred_or_better_references +=
                    payload.inferred_or_better_reference_count;

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
        checker_warning_count: output.checker_report.summary.warnings,
        checker_hint_count: output.checker_report.summary.hints,
        checker_total_findings: output.checker_report.summary.total,
    }
}

fn summarize_checker_style_recovery_canonical_candidate(
    payload: ShadowPayloadV0,
) -> CheckerStyleRecoveryCanonicalCandidateBundleV0 {
    let input_version = payload.input.version.to_string();
    let report = payload.output.checker_report;
    let mut code_counts = BTreeMap::new();
    let mut file_paths = std::collections::BTreeSet::new();
    let mut findings = Vec::new();
    let mut warnings = 0usize;
    let mut hints = 0usize;

    for finding in report.findings {
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
        report_version: report.version,
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
    let mut code_counts = BTreeMap::new();
    let mut file_paths = std::collections::BTreeSet::new();
    let mut findings = Vec::new();
    let mut warnings = 0usize;
    let mut hints = 0usize;

    for finding in report.findings {
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
        report_version: report.version,
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
    let mut code_counts = BTreeMap::new();
    let mut file_paths = std::collections::BTreeSet::new();
    let mut findings = Vec::new();
    let mut warnings = 0usize;
    let mut hints = 0usize;

    for finding in report.findings {
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
        report_version: report.version,
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
