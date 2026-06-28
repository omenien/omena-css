use std::collections::{BTreeMap, BTreeSet};

use omena_bridge::{
    DesignTokenExternalDeclarationCandidateScopeV0, DesignTokenWorkspaceDeclarationFactV0,
    StyleSemanticGraphSummaryV0,
    collect_omena_bridge_design_token_workspace_declarations_from_source,
    summarize_omena_bridge_style_semantic_graph_from_source,
    summarize_omena_bridge_style_semantic_graph_from_source_with_scoped_workspace_declarations,
};
pub use omena_bridge::{
    OmenaBridgeExternalSifCacheContextV0 as OmenaQueryExternalSifCacheContextV0,
    generate_omena_bridge_sif_for_resolved_style_path,
    generate_omena_bridge_sif_for_resolved_style_path_with_cache_context,
};
pub use omena_bridge::{
    SourceBindingDeclFactV0 as OmenaQuerySourceBindingDeclFactV0,
    SourceBindingIndexV0 as OmenaQuerySourceBindingIndexV0,
    SourceBindingScopeFactV0 as OmenaQuerySourceBindingScopeFactV0,
    SourceBindingStyleImportFactV0 as OmenaQuerySourceBindingStyleImportFactV0,
    SourceClassExpressionNodeFactV0 as OmenaQuerySourceClassExpressionNodeFactV0,
    SourceClassUtilityBindingFactV0 as OmenaQuerySourceClassUtilityBindingFactV0,
    SourceClassValueUniverseAxisV0 as OmenaQuerySourceClassValueUniverseAxisV0,
    SourceClassValueUniverseEntryV0 as OmenaQuerySourceClassValueUniverseEntryV0,
    SourceClassnamesBindUtilityBindingFactV0 as OmenaQuerySourceClassnamesBindUtilityBindingFactV0,
    SourceControlFlowGraphCaptureV0 as OmenaQuerySourceControlFlowGraphCaptureV0,
    SourceDeclaresStyleImportFactV0 as OmenaQuerySourceDeclaresStyleImportFactV0,
    SourceDeclaresUtilityBindingFactV0 as OmenaQuerySourceDeclaresUtilityBindingFactV0,
    SourceDomainClassReferenceFactV0 as OmenaQuerySourceDomainClassReferenceFactV0,
    SourceExpressionTargetsModuleFactV0 as OmenaQuerySourceExpressionTargetsModuleFactV0,
    SourceFlowBlockGraphSnapshotV0 as OmenaQuerySourceFlowBlockGraphSnapshotV0,
    SourceFlowBlockSnapshotV0 as OmenaQuerySourceFlowBlockSnapshotV0,
    SourceImportDeclarationSummaryV0 as OmenaQuerySourceImportDeclarationSummaryV0,
    SourceImportDeclarationV0 as OmenaQuerySourceImportDeclarationV0,
    SourceImportedStyleBindingV0 as OmenaQuerySourceImportedStyleBindingV0,
    SourceInlineStyleDeclarationFactV0 as OmenaQuerySourceInlineStyleDeclarationFactV0,
    SourceLanguageParserBoundarySummaryV0 as OmenaQuerySourceLanguageParserBoundarySummaryV0,
    SourceLanguageParserDescriptorV0 as OmenaQuerySourceLanguageParserDescriptorV0,
    SourceScopeContainsDeclFactV0 as OmenaQuerySourceScopeContainsDeclFactV0,
    SourceScopeParentFactV0 as OmenaQuerySourceScopeParentFactV0,
    SourceSelectorReferenceFactV0 as OmenaQuerySourceSelectorReferenceFactV0,
    SourceSelectorReferenceMatchKindV0 as OmenaQuerySourceSelectorReferenceMatchKindV0,
    SourceStyleAccessUsesStyleImportFactV0 as OmenaQuerySourceStyleAccessUsesStyleImportFactV0,
    SourceStyleImportResolvesModuleFactV0 as OmenaQuerySourceStyleImportResolvesModuleFactV0,
    SourceStylePropertyAccessFactV0 as OmenaQuerySourceStylePropertyAccessFactV0,
    SourceSymbolRefUsesDeclFactV0 as OmenaQuerySourceSymbolRefUsesDeclFactV0,
    SourceSyntaxIndexV0 as OmenaQuerySourceSyntaxIndexV0,
    SourceTypeFactProviderUnavailableFactV0 as OmenaQuerySourceTypeFactProviderUnavailableFactV0,
    SourceTypeFactTargetV0 as OmenaQuerySourceTypeFactTargetV0,
    SourceUtilityUsesStyleImportFactV0 as OmenaQuerySourceUtilityUsesStyleImportFactV0,
    summarize_omena_bridge_source_language_parser_boundary_v0 as summarize_omena_query_source_language_parser_boundary_v0,
};
pub type OmenaQuerySourceTypeFactControlFlowGraphV0 =
    omena_bridge::OmenaBridgeSourceTypeFactControlFlowGraphV0;
pub use omena_cross_file_summary::*;
pub use omena_parser::{
    ParseTreeNodeV0 as OmenaQueryParseTreeNodeV0, ParserByteSpanV0, ParserPositionV0,
    ParserRangeV0, StyleDialect as OmenaParserStyleDialect, StyleLanguage,
};
use omena_parser::{
    ParsedSassModuleEdgeFactKind, ParsedSassSymbolFactKind, ParsedSelectorFactKind,
    ParsedVariableFactKind,
};

pub fn parse_style_document_typed_v0(
    source: &str,
    dialect: OmenaParserStyleDialect,
) -> OmenaQueryParseTreeNodeV0 {
    omena_cst_typed::parse_style_document_typed_v0(source, dialect)
}

pub fn syntax_node_id_for_omena_query_style_source(
    style_source: &str,
    dialect: OmenaParserStyleDialect,
) -> String {
    let parsed = omena_parser::parse(style_source, dialect);
    omena_parser::syntax_node_id(&parsed.syntax())
        .as_str()
        .to_string()
}
use omena_query_checker_orchestrator::{
    CATEGORICAL_FEATURE_GATE_V0, CATEGORICAL_LAYER_MARKER_V0, CATEGORICAL_SCHEMA_VERSION_V0,
    DesignSystemEdgeKindCountV0, DesignSystemProjectSummaryInputV0,
    build_omena_query_checker_design_system_model_from_project_summary_v0,
    compare_omena_query_checker_design_system_models_for_invariant_v0,
};
pub use omena_query_checker_orchestrator::{
    DesignSystemInvariantSummaryV0 as OmenaQueryCategoricalDesignSystemInvariantSummaryV0,
    DesignSystemModelV0 as OmenaQueryCategoricalDesignSystemModelV0,
};
pub use omena_query_core::EngineInputV2 as OmenaQueryEngineInputV2;
pub use omena_query_core::*;
pub use omena_query_transform_runner::{
    CustomPropertyLeastFixedPointSummaryV0 as OmenaQueryCustomPropertyLeastFixedPointSummaryV0,
    TargetFeatureSupportV0 as OmenaQueryTargetFeatureSupportV0,
    TargetTransformOptionsV0 as OmenaQueryTargetTransformOptionsV0,
    TransformBundleAssetUrlRewriteSummaryV0 as OmenaQueryTransformBundleAssetUrlRewriteSummaryV0,
    TransformBundleEdgeKind,
    TransformBundleSourceSummaryV0 as OmenaQueryTransformBundleSourceSummaryV0,
    TransformClassNameRewriteV0 as OmenaQueryTransformClassNameRewriteV0,
    TransformCssModuleComposesResolutionV0 as OmenaQueryTransformCssModuleComposesResolutionV0,
    TransformCssModuleValueResolutionV0 as OmenaQueryTransformCssModuleValueResolutionV0,
    TransformDesignTokenRouteV0 as OmenaQueryTransformDesignTokenRouteV0,
    TransformExecutionContextV0 as OmenaQueryTransformExecutionContextV0,
    TransformExecutionSummaryV0 as OmenaQueryTransformExecutionSummaryV0,
    TransformImportInlineV0 as OmenaQueryTransformImportInlineV0,
    TransformModuleEvaluationNativeEditV0 as OmenaQueryTransformModuleEvaluationNativeEditV0,
    TransformModuleEvaluationNativeReplacementV0 as OmenaQueryTransformModuleEvaluationNativeReplacementV0,
    TransformModuleEvaluationOracleV0 as OmenaQueryTransformModuleEvaluationOracleV0,
    TransformModuleEvaluationV0 as OmenaQueryTransformModuleEvaluationV0,
    TransformPassExecutionOutcomeV0 as OmenaQueryTransformPassExecutionOutcomeV0,
    TransformPrintArtifactV0, TransformPrintMode as OmenaQueryTransformPrintMode,
    TransformPrintOptionsV0 as OmenaQueryTransformPrintOptionsV0,
    TransformSourceMapCompositionV0 as OmenaQueryTransformSourceMapCompositionV0,
    TransformSourceMapV3V0 as OmenaQueryTransformSourceMapV3V0,
    TransformTargetQueryPlanV0 as OmenaQueryTransformTargetQueryPlanV0,
    TransformVendorPrefixPolicyV0 as OmenaQueryTransformVendorPrefixPolicyV0,
    compose_transform_source_map_v3_with_upstream_map as compose_omena_query_transform_source_map_v3_with_upstream_map,
    conservative_target_options as conservative_omena_query_target_options,
    default_print_options as default_omena_query_transform_print_options,
    modern_feature_support as modern_omena_query_target_feature_support,
    rewrite_omena_transform_bundle_asset_urls_in_source,
    summarize_omena_transform_bundle_from_source,
};
use omena_query_transform_runner::{
    EggRewriteSourceWitnessV0, TransformBundleAssetUrlRewriteSummaryV0,
    TransformBundleSourceSummaryV0, TransformEggPlanV0, TransformExecutionContextV0,
    TransformExecutionSummaryV0, TransformPassExecutionOutcomeV0, TransformPassKind,
    TransformPassPlanV0, TransformSourceMapSegmentV0, TransformTargetPlanV0,
    all_transform_pass_kinds, execute_egg_rewrite_witnesses_for_css_source,
    execute_transform_passes_on_source_with_dialect_and_context,
    plan_egg_rewrite_passes_for_source, plan_target_transforms, plan_target_transforms_from_query,
    plan_transform_passes, print_transform_execution_artifact_with_dialect_and_source,
    serialize_transform_source_map_v3_with_source_contents,
    summarize_static_css_custom_property_fixed_point_from_source, transform_source_map_point,
};
#[cfg(feature = "lawvere-trace")]
pub use omena_query_transform_runner::{
    LawvereDifferentialCommutativityWitnessV0 as OmenaQueryLawvereDifferentialCommutativityWitnessV0,
    LawvereModelTraceV0 as OmenaQueryLawvereModelTraceV0,
    ReorderabilityCertificateV0 as OmenaQueryLawvereReorderabilityCertificateV0,
    TransformPassParallelPlanV0 as OmenaQueryLawvereTransformPassParallelPlanV0,
};
#[cfg(feature = "lawvere-trace")]
use omena_query_transform_runner::{
    evaluate_lawvere_reorderability_with_differential_corpus,
    execute_transform_passes_on_source_with_lawvere_trace_and_dialect,
    plan_transform_passes_parallel_lawvere_layers,
};
use omena_resolver::{
    OmenaResolverBoundaryStateKindV0, OmenaResolverBoundaryStateV0, OmenaResolverBoundaryTopV0,
    OmenaResolverBundlerPathAliasMappingV0, OmenaResolverCanonicalUrlV0,
    OmenaResolverStyleModuleConfirmationOptionsV0, OmenaResolverStyleModuleDiskCandidateIdentityV0,
    OmenaResolverStylePackageManifestV0, OmenaResolverTsconfigPathMappingV0,
    omena_resolver_boundary_state_for_unresolved_reference_v0,
    resolve_omena_resolver_style_module_source,
    summarize_omena_resolver_style_module_resolution_with_confirmation_inputs,
    summarize_omena_resolver_style_module_resolution_with_load_path_roots,
};
pub use omena_resolver::{
    OmenaResolverStyleResolutionPolicyStepV0 as OmenaQueryStyleResolutionPolicyStepV0,
    OmenaResolverStyleResolutionPolicyV0 as OmenaQueryStyleResolutionPolicyV0,
    summarize_omena_resolver_style_resolution_policy_v0 as summarize_omena_query_style_resolution_policy_v0,
};
pub use omena_scss_eval::{
    OmenaScssEvalCallReturnIrSummaryV0 as OmenaQueryScssEvalCallReturnIrSummaryV0,
    OmenaScssEvalControlFlowIrSummaryV0 as OmenaQueryScssEvalControlFlowIrSummaryV0,
    OmenaScssEvalControlFlowPruneReachabilityV0 as OmenaQueryScssEvalControlFlowPruneReachabilityV0,
    OmenaScssEvalControlFlowValueAnalysisV0 as OmenaQueryScssEvalControlFlowValueAnalysisV0,
    OmenaScssEvalNativeCssFunctionCallEvaluationSurfaceV0 as OmenaQueryScssEvalNativeCssFunctionCallEvaluationSurfaceV0,
    OmenaScssEvalNativeCssFunctionSurfaceV0 as OmenaQueryScssEvalNativeCssFunctionSurfaceV0,
    OmenaScssEvalNativeCssIfFunctionDecisionSurfaceV0 as OmenaQueryScssEvalNativeCssIfFunctionDecisionSurfaceV0,
    OmenaScssEvalNativeCssStaticEditPlanV0 as OmenaQueryScssEvalNativeCssStaticEditPlanV0,
};
use omena_semantic::StyleContextIndexV0;
use serde::{Deserialize, Serialize};

macro_rules! omena_query_evidence_graph_provenance {
    ($($label:expr),* $(,)?) => {
        crate::types::project_omena_query_provenance_from_evidence_graph(&[$($label),*])
    };
}

mod boundary;
mod source;
mod style;
#[cfg(test)]
mod tests;
mod types;

pub use boundary::*;
pub use source::*;
pub use style::*;
pub use types::*;
