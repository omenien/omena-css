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

use engine_style_parser::{
    AtRuleKind, Stylesheet, SyntaxNodePayload, parse_style_module,
    summarize_css_modules_intermediate,
};
pub use engine_style_parser::{ParserByteSpanV0, ParserPositionV0, ParserRangeV0, StyleLanguage};
use omena_abstract_value::{
    AbstractValueDomainSummaryV0, ClassValueFlowAnalysisV0, ClassValueFlowIncrementalAnalysisV0,
    SelectorProjectionCertaintyV0, analyze_class_value_flow_incremental_with_database,
    project_abstract_value_selectors, summarize_omena_abstract_value_domain,
};
use omena_bridge::{
    DesignTokenExternalDeclarationCandidateScopeV0, DesignTokenWorkspaceDeclarationFactV0,
    StyleSemanticGraphSummaryV0, collect_omena_bridge_design_token_workspace_declarations,
    summarize_omena_bridge_style_semantic_graph_for_path_with_scoped_workspace_declarations,
    summarize_omena_bridge_style_semantic_graph_from_source,
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
pub use omena_parser::StyleDialect as OmenaParserStyleDialect;
use omena_parser::{
    ParsedAnimationFactKind, ParsedCssModuleComposesEdgeKind, ParsedCssModuleComposesFactKind,
    ParsedCssModuleValueFactKind, ParsedIcssFactKind, ParsedSassModuleEdgeFactKind,
    ParsedSassSymbolFactKind, ParsedSelectorFactKind, ParsedVariableFactKind, collect_style_facts,
};
use omena_resolver::{
    OmenaResolverSourceResolutionRuntimeIndexV0, OmenaResolverStylePackageManifestV0,
    resolve_omena_resolver_style_module_source, summarize_omena_resolver_canonical_producer_signal,
    summarize_omena_resolver_query_fragments, summarize_omena_resolver_source_resolution_runtime,
    summarize_omena_resolver_style_module_resolution,
};
use serde::Serialize;

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
