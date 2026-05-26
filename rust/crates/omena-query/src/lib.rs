use std::collections::{BTreeMap, BTreeSet};

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
use omena_parser::{
    ParsedSassModuleEdgeFactKind, ParsedSassSymbolFactKind, ParsedSelectorFactKind,
    ParsedVariableFactKind,
};
pub use omena_parser::{
    ParserByteSpanV0, ParserPositionV0, ParserRangeV0, StyleDialect as OmenaParserStyleDialect,
    StyleLanguage,
};
pub use omena_query_core::EngineInputV2 as OmenaQueryEngineInputV2;
pub use omena_query_core::*;
pub use omena_query_transform_runner::{
    CustomPropertyLeastFixedPointSummaryV0 as OmenaQueryCustomPropertyLeastFixedPointSummaryV0,
    TargetFeatureSupportV0 as OmenaQueryTargetFeatureSupportV0,
    TargetTransformOptionsV0 as OmenaQueryTargetTransformOptionsV0,
    TransformClassNameRewriteV0 as OmenaQueryTransformClassNameRewriteV0,
    TransformCssModuleComposesResolutionV0 as OmenaQueryTransformCssModuleComposesResolutionV0,
    TransformCssModuleValueResolutionV0 as OmenaQueryTransformCssModuleValueResolutionV0,
    TransformDesignTokenRouteV0 as OmenaQueryTransformDesignTokenRouteV0,
    TransformExecutionContextV0 as OmenaQueryTransformExecutionContextV0,
    TransformExecutionSummaryV0 as OmenaQueryTransformExecutionSummaryV0,
    TransformImportInlineV0 as OmenaQueryTransformImportInlineV0,
    TransformModuleEvaluationV0 as OmenaQueryTransformModuleEvaluationV0, TransformPrintArtifactV0,
    TransformPrintMode as OmenaQueryTransformPrintMode,
    TransformPrintOptionsV0 as OmenaQueryTransformPrintOptionsV0,
    TransformTargetQueryPlanV0 as OmenaQueryTransformTargetQueryPlanV0,
    conservative_target_options as conservative_omena_query_target_options,
    default_print_options as default_omena_query_transform_print_options,
    modern_feature_support as modern_omena_query_target_feature_support,
};
use omena_query_transform_runner::{
    EggRewriteSourceWitnessV0, TransformBundleSourceSummaryV0, TransformEggPlanV0,
    TransformExecutionContextV0, TransformExecutionSummaryV0, TransformPassKind,
    TransformPassPlanV0, TransformTargetPlanV0, all_transform_pass_kinds,
    execute_egg_rewrite_witnesses_for_css_source,
    execute_transform_passes_on_source_with_dialect_and_context,
    plan_egg_rewrite_passes_for_source, plan_target_transforms, plan_target_transforms_from_query,
    plan_transform_passes, print_transform_execution_artifact_with_dialect_and_source,
    summarize_omena_transform_bundle_from_source,
    summarize_static_css_custom_property_fixed_point_from_source,
};
use omena_resolver::{
    OmenaResolverBoundaryStateV0, OmenaResolverStylePackageManifestV0,
    canonicalize_omena_resolver_style_identity_path, resolve_omena_resolver_style_module_source,
    summarize_omena_resolver_style_module_resolution,
};
use omena_semantic::StyleContextIndexV0;
use serde::{Deserialize, Serialize};

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
