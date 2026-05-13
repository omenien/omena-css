pub use engine_input_producers::EngineInputV2 as OmenaQueryEngineInputV2;
use engine_input_producers::{
    EngineInputV2, ExpressionDomainControlFlowAnalysisV0, ExpressionDomainFlowAnalysisV0,
    ExpressionSemanticsCanonicalProducerSignalV0, ExpressionSemanticsQueryFragmentsV0,
    SelectorUsageCanonicalProducerSignalV0, SelectorUsageQueryFragmentsV0,
    SourceResolutionCanonicalProducerSignalV0, SourceResolutionQueryFragmentsV0,
    collect_expression_domain_flow_graphs, summarize_expression_domain_control_flow_analysis_input,
    summarize_expression_domain_flow_analysis_input,
    summarize_expression_semantics_canonical_producer_signal_input,
    summarize_expression_semantics_query_fragments_input,
    summarize_selector_usage_canonical_producer_signal_input,
    summarize_selector_usage_query_fragments_input,
};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

use omena_abstract_value::{
    AbstractValueDomainSummaryV0, ClassValueFlowAnalysisV0, ClassValueFlowIncrementalAnalysisV0,
    ReducedClassValueProductV0, SelectorProjectionCertaintyV0,
    analyze_class_value_flow_incremental_with_database, project_abstract_value_selectors,
    summarize_omena_abstract_value_domain, summarize_reduced_class_value_product,
};
use omena_bridge::{
    DesignTokenExternalDeclarationCandidateScopeV0, DesignTokenWorkspaceDeclarationFactV0,
    StyleSemanticGraphSummaryV0,
    collect_omena_bridge_design_token_workspace_declarations_from_source,
    summarize_omena_bridge_style_semantic_graph_from_source,
    summarize_omena_bridge_style_semantic_graph_from_source_with_scoped_workspace_declarations,
};
pub use omena_bridge::{
    SourceImportDeclarationSummaryV0 as OmenaQuerySourceImportDeclarationSummaryV0,
    SourceImportDeclarationV0 as OmenaQuerySourceImportDeclarationV0,
    SourceImportedStyleBindingV0 as OmenaQuerySourceImportedStyleBindingV0,
    SourceSelectorReferenceFactV0 as OmenaQuerySourceSelectorReferenceFactV0,
    SourceSelectorReferenceMatchKindV0 as OmenaQuerySourceSelectorReferenceMatchKindV0,
    SourceSyntaxIndexV0 as OmenaQuerySourceSyntaxIndexV0,
    SourceTypeFactTargetV0 as OmenaQuerySourceTypeFactTargetV0,
};
use omena_incremental::OmenaIncrementalDatabaseV0;
use omena_parser::{
    ParsedAnimationFactKind, ParsedCssModuleComposesEdgeKind, ParsedCssModuleComposesFactKind,
    ParsedCssModuleValueFactKind, ParsedIcssFactKind, ParsedSassModuleEdgeFactKind,
    ParsedSassSymbolFactKind, ParsedSelectorFactKind, ParsedVariableFactKind, collect_style_facts,
    parse,
};
pub use omena_parser::{
    ParserByteSpanV0, ParserPositionV0, ParserRangeV0, StyleDialect as OmenaParserStyleDialect,
    StyleLanguage,
};
use omena_resolver::{
    OmenaResolverSourceResolutionRuntimeIndexV0, OmenaResolverStylePackageManifestV0,
    resolve_omena_resolver_style_module_source, summarize_omena_resolver_canonical_producer_signal,
    summarize_omena_resolver_query_fragments, summarize_omena_resolver_source_resolution_runtime,
    summarize_omena_resolver_style_module_resolution,
};
use omena_semantic::StyleContextIndexV0;
use omena_transform_bundle::{
    TransformBundleSourceSummaryV0, summarize_omena_transform_bundle_from_source,
};
use omena_transform_cst::{TransformPassKind, all_transform_pass_kinds};
use omena_transform_egg::{
    EggRewriteSourceWitnessV0, TransformEggPlanV0, execute_egg_rewrite_witnesses_for_css_source,
    plan_egg_rewrite_passes_for_source,
};
pub use omena_transform_passes::{
    CustomPropertyLeastFixedPointSummaryV0 as OmenaQueryCustomPropertyLeastFixedPointSummaryV0,
    TransformClassNameRewriteV0 as OmenaQueryTransformClassNameRewriteV0,
    TransformCssModuleComposesResolutionV0 as OmenaQueryTransformCssModuleComposesResolutionV0,
    TransformDesignTokenRouteV0 as OmenaQueryTransformDesignTokenRouteV0,
    TransformExecutionContextV0 as OmenaQueryTransformExecutionContextV0,
    TransformExecutionSummaryV0 as OmenaQueryTransformExecutionSummaryV0,
    TransformImportInlineV0 as OmenaQueryTransformImportInlineV0,
    TransformModuleEvaluationV0 as OmenaQueryTransformModuleEvaluationV0,
};
use omena_transform_passes::{
    TransformExecutionContextV0, TransformExecutionSummaryV0, TransformPassPlanV0,
    execute_transform_passes_on_source_with_dialect_and_context, plan_transform_passes,
    summarize_static_css_custom_property_fixed_point_from_source,
};
use omena_transform_print::print_transform_execution_artifact_with_dialect;
pub use omena_transform_print::{
    TransformPrintArtifactV0, TransformPrintOptionsV0 as OmenaQueryTransformPrintOptionsV0,
    default_print_options as default_omena_query_transform_print_options,
};
pub use omena_transform_target::{
    TargetFeatureSupportV0 as OmenaQueryTargetFeatureSupportV0,
    TargetTransformOptionsV0 as OmenaQueryTargetTransformOptionsV0,
    TransformTargetQueryPlanV0 as OmenaQueryTransformTargetQueryPlanV0,
    conservative_target_options as conservative_omena_query_target_options,
    modern_feature_support as modern_omena_query_target_feature_support,
};
use omena_transform_target::{
    TransformTargetPlanV0, plan_target_transforms, plan_target_transforms_from_query,
};
use serde::{Deserialize, Serialize};

mod boundary;
mod fragments;
mod source;
mod style;
#[cfg(test)]
mod tests;
mod types;

pub use boundary::*;
pub use fragments::*;
pub use source::*;
pub use style::*;
pub use types::*;
