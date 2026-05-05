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
use std::path::{Component, Path, PathBuf};

use engine_style_parser::{
    AtRuleKind, ParserByteSpanV0, ParserPositionV0, ParserRangeV0, StyleLanguage, Stylesheet,
    SyntaxNodePayload, parse_style_module, summarize_css_modules_intermediate,
};
use omena_abstract_value::{
    AbstractValueDomainSummaryV0, ClassValueFlowAnalysisV0, ClassValueFlowIncrementalAnalysisV0,
    analyze_class_value_flow_incremental_with_database, summarize_omena_abstract_value_domain,
};
use omena_bridge::{
    DesignTokenExternalDeclarationCandidateScopeV0, DesignTokenWorkspaceDeclarationFactV0,
    StyleSemanticGraphSummaryV0, collect_omena_bridge_design_token_workspace_declarations,
    summarize_omena_bridge_style_semantic_graph_for_path_with_scoped_workspace_declarations,
    summarize_omena_bridge_style_semantic_graph_from_source,
};
use omena_incremental::OmenaIncrementalDatabaseV0;
use omena_resolver::{
    OmenaResolverSourceResolutionRuntimeIndexV0,
    summarize_omena_resolver_canonical_producer_signal, summarize_omena_resolver_query_fragments,
    summarize_omena_resolver_source_resolution_runtime,
};
use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryBoundarySummaryV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub query_engine_name: &'static str,
    pub input_version: String,
    pub abstract_value_domain: AbstractValueDomainSummaryV0,
    pub selected_query_adapter_capabilities: SelectedQueryAdapterCapabilitiesV0,
    pub delegated_fragment_products: Vec<&'static str>,
    pub expression_semantics_query_count: usize,
    pub source_resolution_query_count: usize,
    pub selector_usage_query_count: usize,
    pub total_query_count: usize,
    pub ready_surfaces: Vec<&'static str>,
    pub cme_coupled_surfaces: Vec<&'static str>,
    pub next_decoupling_targets: Vec<&'static str>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryFragmentBundleV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub input_version: String,
    pub expression_semantics: ExpressionSemanticsQueryFragmentsV0,
    pub source_resolution: SourceResolutionQueryFragmentsV0,
    pub selector_usage: SelectorUsageQueryFragmentsV0,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SelectedQueryAdapterCapabilitiesV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub default_candidate_backend: &'static str,
    pub backend_kinds: Vec<SelectedQueryBackendCapabilityV0>,
    pub runner_commands: Vec<SelectedQueryRunnerCommandV0>,
    pub expression_semantics_payload_contracts: Vec<&'static str>,
    pub required_input_contracts: Vec<&'static str>,
    pub adapter_readiness: Vec<&'static str>,
    pub routing_status: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SelectedQueryBackendCapabilityV0 {
    pub backend_kind: &'static str,
    pub source_resolution: bool,
    pub expression_semantics: bool,
    pub selector_usage: bool,
    pub style_semantic_graph: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SelectedQueryRunnerCommandV0 {
    pub surface: &'static str,
    pub command: &'static str,
    pub input_contract: &'static str,
    pub output_product: &'static str,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryStyleSemanticGraphBatchOutputV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub graphs: Vec<OmenaQueryStyleSemanticGraphBatchEntryV0>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryStyleSemanticGraphBatchEntryV0 {
    pub style_path: String,
    pub graph: Option<StyleSemanticGraphSummaryV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryStyleDocumentSummaryV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub language: &'static str,
    pub selector_names: Vec<String>,
    pub custom_property_decl_names: Vec<String>,
    pub custom_property_ref_names: Vec<String>,
    pub sass_module_use_sources: Vec<String>,
    pub sass_module_forward_sources: Vec<String>,
    pub diagnostic_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryStyleHoverCandidateV0 {
    pub kind: &'static str,
    pub name: String,
    pub range: ParserRangeV0,
    pub source: &'static str,
    pub namespace: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryStyleHoverCandidatesV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub language: &'static str,
    pub candidates: Vec<OmenaQueryStyleHoverCandidateV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryStyleHoverRenderPartsV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub snippet: String,
    pub value: Option<String>,
    pub signature: Option<String>,
    pub render_source: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryStyleDiagnosticV0 {
    pub code: &'static str,
    pub range: ParserRangeV0,
    pub message: String,
    pub create_custom_property: Option<OmenaQueryCreateCustomPropertyActionV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryCreateCustomPropertyActionV0 {
    pub uri: String,
    pub range: ParserRangeV0,
    pub new_text: String,
    pub property_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQuerySassModuleUseEdgeV0 {
    pub source: String,
    pub namespace_kind: &'static str,
    pub namespace: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQuerySassModuleSourcesV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub module_use_edges: Vec<OmenaQuerySassModuleUseEdgeV0>,
    pub module_forward_sources: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OmenaQueryStylePackageManifestV0 {
    pub package_json_path: String,
    pub package_json_source: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryExpressionDomainIncrementalFlowAnalysisV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub input_version: String,
    pub revision: u64,
    pub graph_count: usize,
    pub dirty_graph_count: usize,
    pub reused_graph_count: usize,
    pub analyses: Vec<OmenaQueryExpressionDomainIncrementalFlowAnalysisEntryV0>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OmenaQueryExpressionDomainIncrementalFlowAnalysisEntryV0 {
    pub graph_id: String,
    pub file_path: String,
    pub analysis: ClassValueFlowIncrementalAnalysisV0,
}

#[derive(Default)]
pub struct OmenaQueryExpressionDomainFlowRuntimeV0 {
    revision: u64,
    databases_by_graph_id: BTreeMap<String, OmenaIncrementalDatabaseV0>,
    previous_analyses_by_graph_id: BTreeMap<String, ClassValueFlowAnalysisV0>,
}

pub fn summarize_omena_query_boundary(input: &EngineInputV2) -> OmenaQueryBoundarySummaryV0 {
    let fragment_bundle = summarize_omena_query_fragment_bundle(input);
    let expression_semantics_query_count = fragment_bundle.expression_semantics.fragments.len();
    let source_resolution_query_count = fragment_bundle.source_resolution.fragments.len();
    let selector_usage_query_count = fragment_bundle.selector_usage.fragments.len();

    OmenaQueryBoundarySummaryV0 {
        schema_version: "0",
        product: "omena-query.boundary",
        query_engine_name: "omena-query",
        input_version: input.version.clone(),
        abstract_value_domain: summarize_omena_abstract_value_domain(),
        selected_query_adapter_capabilities:
            summarize_omena_query_selected_query_adapter_capabilities(),
        delegated_fragment_products: vec![
            "engine-input-producers.expression-semantics-query-fragments",
            "engine-input-producers.source-resolution-query-fragments",
            "omena-resolver.boundary",
            "omena-resolver.source-resolution-runtime-index",
            "engine-input-producers.selector-usage-query-fragments",
            "engine-input-producers.expression-domain-flow-analysis",
            "engine-input-producers.expression-domain-control-flow-analysis",
            "omena-query.expression-domain-incremental-flow-analysis",
        ],
        expression_semantics_query_count,
        source_resolution_query_count,
        selector_usage_query_count,
        total_query_count: expression_semantics_query_count
            + source_resolution_query_count
            + selector_usage_query_count,
        ready_surfaces: vec![
            "queryFragmentBundle",
            "abstractValueProjectionContract",
            "sourceResolutionResolverBoundary",
            "sourceResolutionRuntimeIndex",
            "expressionDomainFlowAnalysisBoundary",
            "expressionDomainControlFlowAnalysisBoundary",
            "expressionDomainSalsaRuntime",
            "styleHoverRenderParts",
            "styleMissingCustomPropertyDiagnostics",
            "queryBoundarySummary",
        ],
        cme_coupled_surfaces: vec!["EngineInputV2", "producerQueryFragments"],
        next_decoupling_targets: vec!["queryEvaluationRuntime", "selectedQueryBackendAdapter"],
    }
}

pub fn summarize_omena_query_selected_query_adapter_capabilities()
-> SelectedQueryAdapterCapabilitiesV0 {
    SelectedQueryAdapterCapabilitiesV0 {
        schema_version: "0",
        product: "omena-query.selected-query-adapter-capabilities",
        default_candidate_backend: "rust-selected-query",
        backend_kinds: vec![
            SelectedQueryBackendCapabilityV0 {
                backend_kind: "typescript-current",
                source_resolution: false,
                expression_semantics: false,
                selector_usage: false,
                style_semantic_graph: false,
            },
            SelectedQueryBackendCapabilityV0 {
                backend_kind: "rust-source-resolution",
                source_resolution: true,
                expression_semantics: false,
                selector_usage: false,
                style_semantic_graph: false,
            },
            SelectedQueryBackendCapabilityV0 {
                backend_kind: "rust-expression-semantics",
                source_resolution: false,
                expression_semantics: true,
                selector_usage: false,
                style_semantic_graph: false,
            },
            SelectedQueryBackendCapabilityV0 {
                backend_kind: "rust-selector-usage",
                source_resolution: false,
                expression_semantics: false,
                selector_usage: true,
                style_semantic_graph: false,
            },
            SelectedQueryBackendCapabilityV0 {
                backend_kind: "rust-selected-query",
                source_resolution: true,
                expression_semantics: true,
                selector_usage: true,
                style_semantic_graph: true,
            },
        ],
        runner_commands: vec![
            SelectedQueryRunnerCommandV0 {
                surface: "sourceResolution",
                command: "input-source-resolution-canonical-producer",
                input_contract: "EngineInputV2",
                output_product: "engine-input-producers.source-resolution-canonical-producer",
            },
            SelectedQueryRunnerCommandV0 {
                surface: "sourceResolutionRuntime",
                command: "input-omena-resolver-source-resolution-runtime",
                input_contract: "EngineInputV2",
                output_product: "omena-resolver.source-resolution-runtime-index",
            },
            SelectedQueryRunnerCommandV0 {
                surface: "expressionSemantics",
                command: "input-expression-semantics-canonical-producer",
                input_contract: "EngineInputV2",
                output_product: "engine-input-producers.expression-semantics-canonical-producer",
            },
            SelectedQueryRunnerCommandV0 {
                surface: "expressionDomainFlowAnalysis",
                command: "input-expression-domain-flow-analysis",
                input_contract: "EngineInputV2",
                output_product: "engine-input-producers.expression-domain-flow-analysis",
            },
            SelectedQueryRunnerCommandV0 {
                surface: "expressionDomainControlFlowAnalysis",
                command: "input-expression-domain-control-flow-analysis",
                input_contract: "EngineInputV2",
                output_product: "engine-input-producers.expression-domain-control-flow-analysis",
            },
            SelectedQueryRunnerCommandV0 {
                surface: "expressionDomainIncrementalFlowAnalysis",
                command: "input-expression-domain-incremental-flow-analysis",
                input_contract: "EngineInputV2 + OmenaQueryExpressionDomainFlowRuntimeV0",
                output_product: "omena-query.expression-domain-incremental-flow-analysis",
            },
            SelectedQueryRunnerCommandV0 {
                surface: "selectorUsage",
                command: "input-selector-usage-canonical-producer",
                input_contract: "EngineInputV2",
                output_product: "engine-input-producers.selector-usage-canonical-producer",
            },
            SelectedQueryRunnerCommandV0 {
                surface: "styleSemanticGraph",
                command: "style-semantic-graph",
                input_contract: "StyleSemanticGraphInputV0",
                output_product: "omena-semantic.style-semantic-graph",
            },
            SelectedQueryRunnerCommandV0 {
                surface: "styleSemanticGraphBatch",
                command: "style-semantic-graph-batch",
                input_contract: "StyleSemanticGraphBatchInputV0",
                output_product: "omena-semantic.style-semantic-graph-batch",
            },
        ],
        expression_semantics_payload_contracts: vec!["valueDomainKind", "valueDomainDerivation"],
        required_input_contracts: vec![
            "EngineInputV2",
            "StyleSemanticGraphInputV0",
            "StyleSemanticGraphBatchInputV0",
        ],
        adapter_readiness: vec![
            "backendCapabilityMatrix",
            "canonicalProducerWrapperBoundary",
            "styleSemanticGraphBridgeBoundary",
            "runnerCommandContract",
            "fragmentBundleBoundary",
            "sourceResolutionRuntimeIndex",
            "expressionSemanticsDerivationPayload",
            "expressionDomainFlowAnalysisRunner",
            "expressionDomainControlFlowAnalysisRunner",
            "expressionDomainSalsaRuntime",
        ],
        routing_status: "declaredOnly",
    }
}

pub fn summarize_omena_query_fragment_bundle(input: &EngineInputV2) -> OmenaQueryFragmentBundleV0 {
    OmenaQueryFragmentBundleV0 {
        schema_version: "0",
        product: "omena-query.fragment-bundle",
        input_version: input.version.clone(),
        expression_semantics: summarize_omena_query_expression_semantics_query_fragments(input),
        source_resolution: summarize_omena_query_source_resolution_query_fragments(input),
        selector_usage: summarize_omena_query_selector_usage_query_fragments(input),
    }
}

pub fn summarize_omena_query_expression_semantics_query_fragments(
    input: &EngineInputV2,
) -> ExpressionSemanticsQueryFragmentsV0 {
    summarize_expression_semantics_query_fragments_input(input)
}

pub fn summarize_omena_query_expression_domain_flow_analysis(
    input: &EngineInputV2,
) -> ExpressionDomainFlowAnalysisV0 {
    summarize_expression_domain_flow_analysis_input(input)
}

pub fn summarize_omena_query_expression_domain_control_flow_analysis(
    input: &EngineInputV2,
) -> ExpressionDomainControlFlowAnalysisV0 {
    summarize_expression_domain_control_flow_analysis_input(input)
}

pub fn summarize_omena_query_expression_domain_incremental_flow_analysis(
    input: &EngineInputV2,
    runtime: &mut OmenaQueryExpressionDomainFlowRuntimeV0,
) -> OmenaQueryExpressionDomainIncrementalFlowAnalysisV0 {
    runtime.analyze_input(input)
}

impl OmenaQueryExpressionDomainFlowRuntimeV0 {
    pub fn revision(&self) -> u64 {
        self.revision
    }

    pub fn graph_count(&self) -> usize {
        self.databases_by_graph_id.len()
    }

    pub fn analyze_input(
        &mut self,
        input: &EngineInputV2,
    ) -> OmenaQueryExpressionDomainIncrementalFlowAnalysisV0 {
        self.revision += 1;
        let revision = self.revision;
        let flow_graphs = collect_expression_domain_flow_graphs(input);
        let live_graph_ids = flow_graphs
            .iter()
            .map(|entry| entry.graph_id.clone())
            .collect::<BTreeSet<_>>();

        self.databases_by_graph_id
            .retain(|graph_id, _| live_graph_ids.contains(graph_id));
        self.previous_analyses_by_graph_id
            .retain(|graph_id, _| live_graph_ids.contains(graph_id));

        let analyses = flow_graphs
            .into_iter()
            .map(|entry| {
                let database = self
                    .databases_by_graph_id
                    .entry(entry.graph_id.clone())
                    .or_default();
                let previous_analysis = self.previous_analyses_by_graph_id.get(&entry.graph_id);
                let analysis = analyze_class_value_flow_incremental_with_database(
                    &entry.graph,
                    database,
                    previous_analysis,
                    revision,
                );
                self.previous_analyses_by_graph_id
                    .insert(entry.graph_id.clone(), analysis.analysis.clone());

                OmenaQueryExpressionDomainIncrementalFlowAnalysisEntryV0 {
                    graph_id: entry.graph_id,
                    file_path: entry.file_path,
                    analysis,
                }
            })
            .collect::<Vec<_>>();

        let dirty_graph_count = analyses
            .iter()
            .filter(|entry| entry.analysis.incremental_plan.dirty_node_count > 0)
            .count();
        let reused_graph_count = analyses
            .iter()
            .filter(|entry| entry.analysis.reused_previous_analysis)
            .count();

        OmenaQueryExpressionDomainIncrementalFlowAnalysisV0 {
            schema_version: "0",
            product: "omena-query.expression-domain-incremental-flow-analysis",
            input_version: input.version.clone(),
            revision,
            graph_count: analyses.len(),
            dirty_graph_count,
            reused_graph_count,
            analyses,
        }
    }
}

pub fn summarize_omena_query_source_resolution_query_fragments(
    input: &EngineInputV2,
) -> SourceResolutionQueryFragmentsV0 {
    summarize_omena_resolver_query_fragments(input)
}

pub fn summarize_omena_query_selector_usage_query_fragments(
    input: &EngineInputV2,
) -> SelectorUsageQueryFragmentsV0 {
    summarize_selector_usage_query_fragments_input(input)
}

pub fn summarize_omena_query_source_resolution_canonical_producer_signal(
    input: &EngineInputV2,
) -> SourceResolutionCanonicalProducerSignalV0 {
    summarize_omena_resolver_canonical_producer_signal(input)
}

pub fn summarize_omena_query_source_resolution_runtime(
    input: &EngineInputV2,
) -> OmenaResolverSourceResolutionRuntimeIndexV0 {
    summarize_omena_resolver_source_resolution_runtime(input)
}

pub fn summarize_omena_query_expression_semantics_canonical_producer_signal(
    input: &EngineInputV2,
) -> ExpressionSemanticsCanonicalProducerSignalV0 {
    summarize_expression_semantics_canonical_producer_signal_input(input)
}

pub fn summarize_omena_query_selector_usage_canonical_producer_signal(
    input: &EngineInputV2,
) -> SelectorUsageCanonicalProducerSignalV0 {
    summarize_selector_usage_canonical_producer_signal_input(input)
}

pub fn summarize_omena_query_style_semantic_graph_from_source(
    style_path: &str,
    style_source: &str,
    input: &EngineInputV2,
) -> Option<StyleSemanticGraphSummaryV0> {
    summarize_omena_bridge_style_semantic_graph_from_source(style_path, style_source, input)
}

pub fn summarize_omena_query_style_document(
    style_path: &str,
    style_source: &str,
) -> Option<OmenaQueryStyleDocumentSummaryV0> {
    let sheet = parse_style_module(style_path, style_source)?;
    let index = summarize_css_modules_intermediate(&sheet);
    Some(OmenaQueryStyleDocumentSummaryV0 {
        schema_version: "0",
        product: "omena-query.style-document-summary",
        language: style_language_label(sheet.language),
        selector_names: index.selectors.names,
        custom_property_decl_names: index.custom_properties.decl_names,
        custom_property_ref_names: index.custom_properties.ref_names,
        sass_module_use_sources: index.sass.module_use_sources,
        sass_module_forward_sources: index.sass.module_forward_sources,
        diagnostic_count: sheet.diagnostics.len(),
    })
}

pub fn summarize_omena_query_style_hover_candidates(
    style_path: &str,
    style_source: &str,
) -> Option<OmenaQueryStyleHoverCandidatesV0> {
    let sheet = parse_style_module(style_path, style_source)?;
    let index = summarize_css_modules_intermediate(&sheet);
    let mut seen = BTreeSet::new();
    let mut candidates = Vec::new();
    collect_style_selector_hover_candidates_from_parser_facts(
        index.selectors.definition_facts.as_slice(),
        &mut seen,
        &mut candidates,
    );
    collect_custom_property_hover_candidates(
        sheet.source.as_str(),
        index.custom_properties.decl_facts.as_slice(),
        index.custom_properties.ref_names.as_slice(),
        &mut seen,
        &mut candidates,
    );
    collect_sass_symbol_hover_candidates(
        index.sass.symbol_decl_facts.as_slice(),
        index.sass.selector_symbol_facts.as_slice(),
        &mut seen,
        &mut candidates,
    );
    collect_sass_partial_evaluator_selector_candidates(
        sheet.source.as_str(),
        sheet.nodes.as_slice(),
        &mut seen,
        &mut candidates,
    );
    candidates.sort();
    Some(OmenaQueryStyleHoverCandidatesV0 {
        schema_version: "0",
        product: "omena-query.style-hover-candidates",
        language: style_language_label(sheet.language),
        candidates,
    })
}

pub fn summarize_omena_query_style_hover_render_parts(
    source: &str,
    kind: &str,
    name: &str,
    position: ParserPositionV0,
) -> OmenaQueryStyleHoverRenderPartsV0 {
    let mut parts = OmenaQueryStyleHoverRenderPartsV0 {
        schema_version: "0",
        product: "omena-query.style-hover-render-parts",
        snippet: String::new(),
        value: None,
        signature: None,
        render_source: "lineSnippet",
    };

    match kind {
        "selector" => {
            parts.snippet = rule_snippet_around_position(source, position).unwrap_or_else(|| {
                parts.render_source = "selectorFallback";
                format!(".{name} {{ ... }}")
            });
            if parts.render_source != "selectorFallback" {
                parts.render_source = "ruleSnippet";
            }
        }
        "customPropertyReference" | "customPropertyDeclaration" => {
            parts.snippet = line_snippet_at_position(source, position).unwrap_or_default();
        }
        kind if is_sass_symbol_candidate_kind(kind) => {
            parts.snippet = line_snippet_at_position(source, position).unwrap_or_default();
            if sass_symbol_kind_from_candidate_kind(kind) == Some("variable")
                && is_sass_symbol_declaration_kind(kind)
            {
                parts.value = sass_variable_value_from_declaration_line(parts.snippet.as_str());
            } else if matches!(
                sass_symbol_kind_from_candidate_kind(kind),
                Some("mixin" | "function")
            ) && is_sass_symbol_declaration_kind(kind)
                && let Some((signature, snippet)) =
                    sass_callable_definition_render_parts(source, position)
            {
                parts.signature = Some(signature);
                parts.snippet = snippet;
                parts.render_source = "callableBlockSnippet";
            }
        }
        _ => {
            parts.snippet = name.to_string();
            parts.render_source = "candidateNameFallback";
        }
    }

    parts
}

pub fn summarize_omena_query_missing_custom_property_diagnostics(
    style_uri: &str,
    source: &str,
    candidates: &[OmenaQueryStyleHoverCandidateV0],
) -> Vec<OmenaQueryStyleDiagnosticV0> {
    let declaration_names = candidates
        .iter()
        .filter(|candidate| candidate.kind == "customPropertyDeclaration")
        .map(|candidate| candidate.name.as_str())
        .collect::<BTreeSet<_>>();
    if declaration_names.is_empty() {
        return Vec::new();
    }

    let insertion_range = end_of_source_range(source);
    candidates
        .iter()
        .filter(|candidate| {
            candidate.kind == "customPropertyReference"
                && !declaration_names.contains(candidate.name.as_str())
        })
        .map(|candidate| OmenaQueryStyleDiagnosticV0 {
            code: "missingCustomProperty",
            range: candidate.range,
            message: format!(
                "CSS custom property '{}' not found in indexed style tokens.",
                candidate.name
            ),
            create_custom_property: Some(OmenaQueryCreateCustomPropertyActionV0 {
                uri: style_uri.to_string(),
                range: insertion_range,
                new_text: format!("\n\n:root {{\n  {}: ;\n}}\n", candidate.name),
                property_name: candidate.name.clone(),
            }),
        })
        .collect()
}

pub fn summarize_omena_query_sass_module_sources(
    style_path: &str,
    style_source: &str,
) -> Option<OmenaQuerySassModuleSourcesV0> {
    let sheet = parse_style_module(style_path, style_source)?;
    let index = summarize_css_modules_intermediate(&sheet);
    Some(OmenaQuerySassModuleSourcesV0 {
        schema_version: "0",
        product: "omena-query.sass-module-sources",
        module_use_edges: index
            .sass
            .module_use_edges
            .into_iter()
            .map(|edge| OmenaQuerySassModuleUseEdgeV0 {
                source: edge.source,
                namespace_kind: edge.namespace_kind,
                namespace: edge.namespace,
            })
            .collect(),
        module_forward_sources: index.sass.module_forward_sources,
    })
}

pub fn summarize_omena_query_style_semantic_graph_batch_from_sources<'a>(
    styles: impl IntoIterator<Item = (&'a str, &'a str)>,
    input: &EngineInputV2,
) -> OmenaQueryStyleSemanticGraphBatchOutputV0 {
    summarize_omena_query_style_semantic_graph_batch_from_sources_with_package_manifests(
        styles,
        input,
        &[],
    )
}

pub fn summarize_omena_query_style_semantic_graph_batch_from_sources_with_package_manifests<'a>(
    styles: impl IntoIterator<Item = (&'a str, &'a str)>,
    input: &EngineInputV2,
    package_manifests: &[OmenaQueryStylePackageManifestV0],
) -> OmenaQueryStyleSemanticGraphBatchOutputV0 {
    let style_sources = styles.into_iter().collect::<Vec<_>>();
    let parsed_styles = style_sources
        .iter()
        .filter_map(|(style_path, style_source)| {
            parse_style_module(style_path, style_source)
                .map(|sheet| ((*style_path).to_string(), sheet))
        })
        .collect::<Vec<_>>();
    let workspace_declarations = parsed_styles
        .iter()
        .flat_map(|(style_path, sheet)| {
            collect_omena_bridge_design_token_workspace_declarations(style_path, sheet)
        })
        .collect::<Vec<_>>();
    let graphs = style_sources
        .into_iter()
        .map(
            |(style_path, _style_source)| OmenaQueryStyleSemanticGraphBatchEntryV0 {
                style_path: style_path.to_string(),
                graph: parsed_style_by_path(&parsed_styles, style_path).map(|sheet| {
                    let import_reachable_declarations =
                        filter_import_reachable_design_token_workspace_declarations(
                            style_path,
                            &parsed_styles,
                            &workspace_declarations,
                            package_manifests,
                        );
                    summarize_omena_bridge_style_semantic_graph_for_path_with_scoped_workspace_declarations(
                        sheet,
                        input,
                        Some(style_path),
                        &import_reachable_declarations,
                        DesignTokenExternalDeclarationCandidateScopeV0::CrossFileImportGraph,
                    )
                }),
            },
        )
        .collect::<Vec<_>>();

    OmenaQueryStyleSemanticGraphBatchOutputV0 {
        schema_version: "0",
        product: "omena-semantic.style-semantic-graph-batch",
        graphs,
    }
}

fn parsed_style_by_path<'a>(
    parsed_styles: &'a [(String, Stylesheet)],
    style_path: &str,
) -> Option<&'a Stylesheet> {
    parsed_styles
        .iter()
        .find(|(parsed_style_path, _sheet)| parsed_style_path == style_path)
        .map(|(_style_path, sheet)| sheet)
}

fn filter_import_reachable_design_token_workspace_declarations(
    target_style_path: &str,
    parsed_styles: &[(String, Stylesheet)],
    workspace_declarations: &[DesignTokenWorkspaceDeclarationFactV0],
    package_manifests: &[OmenaQueryStylePackageManifestV0],
) -> Vec<DesignTokenWorkspaceDeclarationFactV0> {
    let reachable_style_paths = collect_import_reachable_style_path_metadata(
        target_style_path,
        parsed_styles,
        package_manifests,
    );
    workspace_declarations
        .iter()
        .filter_map(|declaration| {
            if declaration.file_path == target_style_path {
                return Some(declaration.clone());
            }
            let reachability = reachable_style_paths.get(declaration.file_path.as_str())?;
            let mut declaration = declaration.clone();
            declaration.import_graph_distance = Some(reachability.distance);
            declaration.import_graph_order = Some(reachability.order);
            Some(declaration)
        })
        .collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ImportReachability {
    distance: usize,
    order: usize,
}

fn collect_import_reachable_style_path_metadata(
    target_style_path: &str,
    parsed_styles: &[(String, Stylesheet)],
    package_manifests: &[OmenaQueryStylePackageManifestV0],
) -> BTreeMap<String, ImportReachability> {
    let mut reachable_style_paths = BTreeMap::new();
    let available_style_paths = parsed_styles
        .iter()
        .map(|(style_path, _sheet)| style_path.as_str())
        .collect::<BTreeSet<_>>();
    let mut pending_style_paths = collect_import_reachable_direct_style_paths(
        target_style_path,
        parsed_styles,
        &available_style_paths,
        package_manifests,
    )
    .into_iter()
    .map(|style_path| (style_path, 1usize))
    .collect::<VecDeque<_>>();
    let style_by_path = parsed_styles
        .iter()
        .map(|(style_path, sheet)| (style_path.as_str(), sheet))
        .collect::<BTreeMap<_, _>>();
    let mut visit_order = 0usize;

    while let Some((style_path, distance)) = pending_style_paths.pop_front() {
        if style_path == target_style_path || reachable_style_paths.contains_key(&style_path) {
            continue;
        }
        reachable_style_paths.insert(
            style_path.clone(),
            ImportReachability {
                distance,
                order: visit_order,
            },
        );
        visit_order += 1;

        let Some(sheet) = style_by_path.get(style_path.as_str()) else {
            continue;
        };
        for source in collect_sass_module_sources(sheet) {
            if let Some(next_style_path) = resolve_style_module_source(
                &style_path,
                &source,
                &available_style_paths,
                package_manifests,
            ) {
                pending_style_paths.push_back((next_style_path, distance + 1));
            }
        }
    }

    reachable_style_paths
}

fn collect_import_reachable_direct_style_paths(
    target_style_path: &str,
    parsed_styles: &[(String, Stylesheet)],
    available_style_paths: &BTreeSet<&str>,
    package_manifests: &[OmenaQueryStylePackageManifestV0],
) -> Vec<String> {
    let Some(target_sheet) = parsed_style_by_path(parsed_styles, target_style_path) else {
        return Vec::new();
    };
    collect_sass_module_sources(target_sheet)
        .into_iter()
        .filter_map(|source| {
            resolve_style_module_source(
                target_style_path,
                &source,
                available_style_paths,
                package_manifests,
            )
        })
        .collect()
}

fn collect_sass_module_sources(sheet: &Stylesheet) -> Vec<String> {
    let summary = summarize_css_modules_intermediate(sheet);
    let mut sources = Vec::new();
    for edge in summary.sass.module_use_edges {
        push_unique_string(&mut sources, edge.source);
    }
    for source in summary.sass.module_forward_sources {
        push_unique_string(&mut sources, source);
    }
    for source in summary.sass.module_import_sources {
        push_unique_string(&mut sources, source);
    }
    sources
}

fn resolve_style_module_source(
    from_style_path: &str,
    source: &str,
    available_style_paths: &BTreeSet<&str>,
    package_manifests: &[OmenaQueryStylePackageManifestV0],
) -> Option<String> {
    if source.starts_with("sass:")
        || source.starts_with("http://")
        || source.starts_with("https://")
    {
        return None;
    }

    style_module_source_candidates(from_style_path, source, package_manifests)
        .into_iter()
        .find(|candidate| available_style_paths.contains(candidate.as_str()))
}

fn style_module_source_candidates(
    from_style_path: &str,
    source: &str,
    package_manifests: &[OmenaQueryStylePackageManifestV0],
) -> Vec<String> {
    let source_path = Path::new(source);
    let base_path = if source_path.is_absolute() {
        PathBuf::from(source)
    } else {
        Path::new(from_style_path)
            .parent()
            .map(|parent| parent.join(source))
            .unwrap_or_else(|| PathBuf::from(source))
    };
    let mut candidates = Vec::new();
    push_style_module_path_candidates(
        &mut candidates,
        base_path,
        source_path.extension().is_none(),
    );
    for package_manifest_base_path in
        package_manifest_style_module_base_candidates(from_style_path, source, package_manifests)
    {
        push_style_module_path_candidates(&mut candidates, package_manifest_base_path, true);
    }
    for package_base_path in package_style_module_base_candidates(from_style_path, source) {
        push_style_module_path_candidates(&mut candidates, package_base_path, true);
    }

    candidates
}

fn push_style_module_path_candidates(
    candidates: &mut Vec<String>,
    base_path: PathBuf,
    include_extension_variants: bool,
) {
    push_style_path_candidate(candidates, base_path.clone());
    push_partial_style_path_candidate(candidates, &base_path);

    if !include_extension_variants {
        return;
    }

    for extension in [
        ".module.scss",
        ".module.css",
        ".module.less",
        ".scss",
        ".css",
        ".less",
    ] {
        let candidate = PathBuf::from(format!("{}{}", base_path.display(), extension));
        push_style_path_candidate(candidates, candidate.clone());
        push_partial_style_path_candidate(candidates, &candidate);
    }
}

fn package_style_module_base_candidates(from_style_path: &str, source: &str) -> Vec<PathBuf> {
    let Some(package_source) = parse_package_style_source(source) else {
        return Vec::new();
    };
    let Some(from_dir) = Path::new(from_style_path).parent() else {
        return Vec::new();
    };
    let mut candidates = Vec::new();
    let mut current_dir = Some(from_dir);
    while let Some(dir) = current_dir {
        let package_root = dir.join("node_modules").join(package_source.package_name);
        let package_entry = match package_source.subpath {
            Some(subpath) => package_root.join(subpath),
            None => package_root.clone(),
        };
        push_unique_pathbuf(&mut candidates, package_entry.clone());
        if let Some(subpath) = package_source.subpath {
            push_unique_pathbuf(&mut candidates, package_root.join("src").join(subpath));
        } else {
            push_unique_pathbuf(&mut candidates, package_root.join("index"));
            push_unique_pathbuf(&mut candidates, package_root.join("src").join("index"));
        }
        current_dir = dir.parent();
    }
    candidates
}

fn package_manifest_style_module_base_candidates(
    from_style_path: &str,
    source: &str,
    package_manifests: &[OmenaQueryStylePackageManifestV0],
) -> Vec<PathBuf> {
    let Some(package_source) = parse_package_style_source(source) else {
        return Vec::new();
    };
    let Some(from_dir) = Path::new(from_style_path).parent() else {
        return Vec::new();
    };
    let manifest_by_package_dir = package_manifests
        .iter()
        .map(|manifest| {
            (
                package_dir_from_package_json_path(&manifest.package_json_path),
                manifest.package_json_source.as_str(),
            )
        })
        .collect::<BTreeMap<_, _>>();

    let mut candidates = Vec::new();
    let mut current_dir = Some(from_dir);
    while let Some(dir) = current_dir {
        let package_root = dir.join("node_modules").join(package_source.package_name);
        let package_root_key = normalize_style_path(package_root.clone());
        if let Some(package_json_source) = manifest_by_package_dir.get(&package_root_key)
            && let Some(entry) =
                read_package_manifest_style_entry(package_json_source, package_source.subpath)
        {
            push_unique_pathbuf(&mut candidates, package_root.join(entry));
        }
        current_dir = dir.parent();
    }
    candidates
}

fn package_dir_from_package_json_path(package_json_path: &str) -> String {
    Path::new(package_json_path)
        .parent()
        .map(|path| normalize_style_path(path.to_path_buf()))
        .unwrap_or_default()
}

fn read_package_manifest_style_entry(
    package_json_source: &str,
    subpath: Option<&str>,
) -> Option<PathBuf> {
    let package_json = serde_json::from_str::<serde_json::Value>(package_json_source).ok()?;
    let package_object = package_json.as_object()?;
    let entry = if let Some(subpath) = subpath {
        read_package_export_subpath_entry(package_object.get("exports"), subpath)
    } else {
        read_package_json_string_field(package_object, "sass")
            .or_else(|| read_package_json_string_field(package_object, "scss"))
            .or_else(|| read_package_json_string_field(package_object, "style"))
            .or_else(|| read_package_export_entry(package_object.get("exports")))
    }?;
    Some(PathBuf::from(normalize_package_json_entry(&entry)))
}

fn read_package_export_subpath_entry(
    exports_value: Option<&serde_json::Value>,
    subpath: &str,
) -> Option<String> {
    let exports_object = exports_value?.as_object()?;
    for key in package_export_subpath_keys(subpath) {
        if let Some(entry) = read_package_export_entry(exports_object.get(&key)) {
            return Some(entry);
        }
    }
    for (key, export_value) in exports_object {
        let Some(pattern_match) = match_package_export_subpath_pattern(key, subpath) else {
            continue;
        };
        let Some(entry) = read_package_export_entry(Some(export_value)) else {
            continue;
        };
        return Some(substitute_package_export_pattern(&entry, &pattern_match));
    }
    None
}

fn package_export_subpath_keys(subpath: &str) -> Vec<String> {
    let normalized = subpath
        .trim_start_matches("./")
        .trim_start_matches('/')
        .to_string();
    vec![
        format!("./{normalized}"),
        format!("./{normalized}.scss"),
        format!("./{normalized}.sass"),
        format!("./{normalized}.css"),
    ]
}

fn match_package_export_subpath_pattern(pattern_key: &str, subpath: &str) -> Option<String> {
    let normalized_pattern = pattern_key.trim_start_matches("./").trim_start_matches('/');
    let (prefix, suffix) = normalized_pattern.split_once('*')?;
    if suffix.contains('*') {
        return None;
    }

    for candidate_key in package_export_subpath_keys(subpath) {
        let normalized_candidate = candidate_key
            .trim_start_matches("./")
            .trim_start_matches('/')
            .to_string();
        if !normalized_candidate.starts_with(prefix) || !normalized_candidate.ends_with(suffix) {
            continue;
        }
        return Some(
            normalized_candidate[prefix.len()..normalized_candidate.len() - suffix.len()]
                .to_string(),
        );
    }
    None
}

fn substitute_package_export_pattern(entry: &str, pattern_match: &str) -> String {
    if entry.contains('*') {
        entry.replace('*', pattern_match)
    } else {
        entry.to_string()
    }
}

fn read_package_export_entry(exports_value: Option<&serde_json::Value>) -> Option<String> {
    let exports_value = exports_value?;
    if let Some(entry) = exports_value.as_str() {
        return Some(entry.to_string());
    }
    if let Some(entries) = exports_value.as_array() {
        for entry_value in entries {
            if let Some(entry) = read_package_export_entry(Some(entry_value)) {
                return Some(entry);
            }
        }
        return None;
    }
    let exports_object = exports_value.as_object()?;
    if let Some(root_entry) = read_package_export_entry(exports_object.get(".")) {
        return Some(root_entry);
    }
    for key in ["sass", "scss", "style", "default", "import", "require"] {
        if let Some(entry) = read_package_export_entry(exports_object.get(key)) {
            return Some(entry);
        }
    }
    None
}

fn read_package_json_string_field(
    package_object: &serde_json::Map<String, serde_json::Value>,
    key: &str,
) -> Option<String> {
    package_object
        .get(key)
        .and_then(|value| value.as_str())
        .map(ToString::to_string)
}

fn normalize_package_json_entry(entry: &str) -> String {
    entry
        .trim_start_matches("./")
        .trim_start_matches('/')
        .to_string()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PackageStyleSource<'a> {
    package_name: &'a str,
    subpath: Option<&'a str>,
}

fn parse_package_style_source(source: &str) -> Option<PackageStyleSource<'_>> {
    if source.starts_with('.')
        || source.starts_with('/')
        || source.starts_with("sass:")
        || source.starts_with("http://")
        || source.starts_with("https://")
    {
        return None;
    }

    if source.starts_with('@') {
        let mut segments = source.splitn(3, '/');
        let scope = segments.next()?;
        let package = segments.next()?;
        if scope.len() <= 1 || package.is_empty() {
            return None;
        }
        let package_name_end = scope.len() + 1 + package.len();
        let package_name = &source[..package_name_end];
        let subpath = segments.next().filter(|subpath| !subpath.is_empty());
        return Some(PackageStyleSource {
            package_name,
            subpath,
        });
    }

    let mut segments = source.splitn(2, '/');
    let package_name = segments.next()?;
    if package_name.is_empty() {
        return None;
    }
    let subpath = segments.next().filter(|subpath| !subpath.is_empty());
    Some(PackageStyleSource {
        package_name,
        subpath,
    })
}

fn push_unique_pathbuf(candidates: &mut Vec<PathBuf>, value: PathBuf) {
    if !candidates.contains(&value) {
        candidates.push(value);
    }
}

fn push_partial_style_path_candidate(candidates: &mut Vec<String>, path: &Path) {
    let Some(file_name) = path.file_name().and_then(|file_name| file_name.to_str()) else {
        return;
    };
    if file_name.starts_with('_') {
        return;
    }
    let mut partial_path = path.to_path_buf();
    partial_path.set_file_name(format!("_{file_name}"));
    push_style_path_candidate(candidates, partial_path);
}

fn push_style_path_candidate(candidates: &mut Vec<String>, path: PathBuf) {
    let candidate = normalize_style_path(path);
    if !candidates.contains(&candidate) {
        candidates.push(candidate);
    }
}

fn normalize_style_path(path: PathBuf) -> String {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            Component::Normal(part) => normalized.push(part),
            Component::RootDir | Component::Prefix(_) => normalized.push(component.as_os_str()),
        }
    }
    normalized.to_string_lossy().replace('\\', "/")
}

fn collect_style_selector_hover_candidates_from_parser_facts(
    definition_facts: &[engine_style_parser::ParserIndexSelectorDefinitionFactV0],
    seen: &mut BTreeSet<(usize, usize, String)>,
    candidates: &mut Vec<OmenaQueryStyleHoverCandidateV0>,
) {
    for fact in definition_facts {
        if seen.insert((fact.byte_span.start, fact.byte_span.end, fact.name.clone())) {
            candidates.push(OmenaQueryStyleHoverCandidateV0 {
                kind: "selector",
                name: fact.name.clone(),
                range: fact.range,
                source: "engineStyleParserSelectorDefinitionFacts",
                namespace: None,
            });
        }
    }
}

fn collect_custom_property_hover_candidates(
    source: &str,
    decl_facts: &[engine_style_parser::ParserIndexCustomPropertyDeclFactV0],
    ref_names: &[String],
    seen: &mut BTreeSet<(usize, usize, String)>,
    candidates: &mut Vec<OmenaQueryStyleHoverCandidateV0>,
) {
    for fact in decl_facts {
        if seen.insert((fact.byte_span.start, fact.byte_span.end, fact.name.clone())) {
            candidates.push(OmenaQueryStyleHoverCandidateV0 {
                kind: "customPropertyDeclaration",
                name: fact.name.clone(),
                range: fact.range,
                source: "openedStyleDocumentIndex",
                namespace: None,
            });
        }
    }

    for name in ref_names {
        for byte_span in custom_property_ref_byte_spans(source, name) {
            if seen.insert((byte_span.start, byte_span.end, name.clone())) {
                candidates.push(OmenaQueryStyleHoverCandidateV0 {
                    kind: "customPropertyReference",
                    name: name.clone(),
                    range: parser_range_for_byte_span(source, byte_span),
                    source: "openedStyleDocumentIndex",
                    namespace: None,
                });
            }
        }
    }
}

fn collect_sass_symbol_hover_candidates(
    decl_facts: &[engine_style_parser::ParserIndexSassSymbolDeclFactV0],
    ref_facts: &[engine_style_parser::ParserIndexSassSelectorSymbolFactV0],
    seen: &mut BTreeSet<(usize, usize, String)>,
    candidates: &mut Vec<OmenaQueryStyleHoverCandidateV0>,
) {
    for fact in decl_facts {
        if seen.insert((
            fact.byte_span.start,
            fact.byte_span.end,
            format!("{}:{}", fact.symbol_kind, fact.name),
        )) {
            candidates.push(OmenaQueryStyleHoverCandidateV0 {
                kind: sass_symbol_declaration_candidate_kind(fact.symbol_kind),
                name: fact.name.clone(),
                range: fact.range,
                source: "engineStyleParserSassSymbolFacts",
                namespace: None,
            });
        }
    }

    for fact in ref_facts {
        if seen.insert((
            fact.byte_span.start,
            fact.byte_span.end,
            format!(
                "{}:{}:{}",
                fact.symbol_kind,
                fact.namespace.as_deref().unwrap_or_default(),
                fact.name
            ),
        )) {
            candidates.push(OmenaQueryStyleHoverCandidateV0 {
                kind: sass_symbol_reference_candidate_kind(fact.symbol_kind, fact.role),
                name: fact.name.clone(),
                range: fact.range,
                source: "engineStyleParserSassSymbolFacts",
                namespace: fact.namespace.clone(),
            });
        }
    }
}

fn collect_sass_partial_evaluator_selector_candidates(
    source: &str,
    nodes: &[engine_style_parser::SyntaxNode],
    seen: &mut BTreeSet<(usize, usize, String)>,
    candidates: &mut Vec<OmenaQueryStyleHoverCandidateV0>,
) {
    for node in nodes {
        if let Some(SyntaxNodePayload::AtRule(at_rule)) = &node.payload
            && at_rule.kind == AtRuleKind::Include
        {
            let range_span = ParserByteSpanV0 {
                start: node.header_span.unwrap_or(node.span).start,
                end: node.header_span.unwrap_or(node.span).end,
            };
            for selector_name in infer_sass_include_generated_selector_names(&at_rule.params) {
                if seen.insert((range_span.start, range_span.end, selector_name.clone())) {
                    candidates.push(OmenaQueryStyleHoverCandidateV0 {
                        kind: "selector",
                        name: selector_name,
                        range: parser_range_for_byte_span(source, range_span),
                        source: "sassPartialEvaluatorGeneratedSelectors",
                        namespace: None,
                    });
                }
            }
        }
        collect_sass_partial_evaluator_selector_candidates(
            source,
            &node.children,
            seen,
            candidates,
        );
    }
}

fn infer_sass_include_generated_selector_names(params: &str) -> Vec<String> {
    let Some(prefix) = sass_named_argument_string_value(params, "prefix") else {
        return Vec::new();
    };
    if prefix.is_empty() || !prefix.chars().all(is_css_identifier_continue) {
        return Vec::new();
    }
    let mut selectors = sass_first_map_string_keys(params)
        .into_iter()
        .filter(|key| !key.is_empty() && key.chars().all(is_css_identifier_continue))
        .map(|key| format!("{prefix}-{key}"))
        .collect::<Vec<_>>();
    selectors.sort();
    selectors.dedup();
    selectors
}

fn sass_named_argument_string_value(params: &str, name: &str) -> Option<String> {
    let needle = format!("${name}");
    let mut cursor = 0usize;
    while let Some(relative_match) = params[cursor..].find(needle.as_str()) {
        let name_start = cursor + relative_match;
        let name_end = name_start + needle.len();
        if !sass_identifier_boundary(params, name_start, name_end) {
            cursor = name_end;
            continue;
        }
        let colon_offset = skip_ascii_whitespace(params, name_end);
        if params.as_bytes().get(colon_offset) != Some(&b':') {
            cursor = name_end;
            continue;
        }
        let value_start = skip_ascii_whitespace(params, colon_offset + 1);
        return sass_string_literal_value(params, value_start).map(|(value, _)| value);
    }
    None
}

fn sass_first_map_string_keys(params: &str) -> Vec<String> {
    let mut cursor = 0usize;
    while cursor < params.len() {
        let Some(open_relative) = params[cursor..].find('(') else {
            break;
        };
        let open = cursor + open_relative;
        let Some(close) = matching_style_block_end(params, open, b'(', b')') else {
            break;
        };
        let keys = sass_map_string_keys(params, open + 1, close);
        if !keys.is_empty() {
            return keys;
        }
        cursor = open + 1;
    }
    Vec::new()
}

fn sass_map_string_keys(params: &str, start: usize, end: usize) -> Vec<String> {
    split_top_level_style_segments(params, start, end, b',')
        .into_iter()
        .filter_map(|(entry_start, entry_end)| {
            let key_start = skip_ascii_whitespace(params, entry_start);
            let (key, key_end) = sass_string_literal_value(params, key_start)?;
            let colon_offset = skip_ascii_whitespace(params, key_end);
            (colon_offset < entry_end && params.as_bytes().get(colon_offset) == Some(&b':'))
                .then_some(key)
        })
        .collect()
}

fn sass_string_literal_value(source: &str, quote_offset: usize) -> Option<(String, usize)> {
    let quote = source.as_bytes().get(quote_offset).copied()?;
    if !matches!(quote, b'\'' | b'"') {
        return None;
    }
    let literal_end = skip_style_string_literal(source, quote_offset, source.len())?;
    let value_end = literal_end.saturating_sub(1);
    source
        .get(quote_offset + 1..value_end)
        .map(|value| (value.to_string(), literal_end))
}

fn sass_identifier_boundary(source: &str, start: usize, end: usize) -> bool {
    let before = source
        .get(..start)
        .and_then(|prefix| prefix.chars().next_back())
        .is_none_or(|ch| !is_css_identifier_continue(ch) && ch != '$');
    let after = source
        .get(end..)
        .and_then(|suffix| suffix.chars().next())
        .is_none_or(|ch| !is_css_identifier_continue(ch));
    before && after
}

fn sass_symbol_declaration_candidate_kind(symbol_kind: &str) -> &'static str {
    match symbol_kind {
        "variable" => "sassVariableDeclaration",
        "mixin" => "sassMixinDeclaration",
        "function" => "sassFunctionDeclaration",
        _ => "sassSymbolDeclaration",
    }
}

fn is_sass_symbol_candidate_kind(kind: &str) -> bool {
    sass_symbol_kind_from_candidate_kind(kind).is_some()
}

fn is_sass_symbol_declaration_kind(kind: &str) -> bool {
    matches!(
        kind,
        "sassVariableDeclaration"
            | "sassMixinDeclaration"
            | "sassFunctionDeclaration"
            | "sassSymbolDeclaration"
    )
}

fn sass_symbol_kind_from_candidate_kind(kind: &str) -> Option<&'static str> {
    match kind {
        "sassVariableDeclaration" | "sassVariableReference" => Some("variable"),
        "sassMixinDeclaration" | "sassMixinInclude" | "sassMixinReference" => Some("mixin"),
        "sassFunctionDeclaration" | "sassFunctionCall" | "sassFunctionReference" => {
            Some("function")
        }
        "sassSymbolDeclaration" | "sassSymbolReference" => Some("symbol"),
        _ => None,
    }
}

fn sass_symbol_reference_candidate_kind(symbol_kind: &str, role: &str) -> &'static str {
    match (symbol_kind, role) {
        ("variable", _) => "sassVariableReference",
        ("mixin", "include") => "sassMixinInclude",
        ("function", "call") => "sassFunctionCall",
        ("mixin", _) => "sassMixinReference",
        ("function", _) => "sassFunctionReference",
        _ => "sassSymbolReference",
    }
}

fn sass_variable_value_from_declaration_line(line: &str) -> Option<String> {
    let (_, value) = line.split_once(':')?;
    let value = value
        .trim()
        .trim_end_matches(';')
        .trim()
        .trim_end_matches("!default")
        .trim();
    (!value.is_empty()).then(|| value.to_string())
}

fn sass_callable_definition_render_parts(
    source: &str,
    position: ParserPositionV0,
) -> Option<(String, String)> {
    let line_start = byte_offset_for_parser_position(
        source,
        ParserPositionV0 {
            line: position.line,
            character: 0,
        },
    )?;
    let open_brace = source[line_start..].find('{')? + line_start;
    let close_brace = matching_style_block_end(source, open_brace, b'{', b'}')?;
    let signature = source[line_start..open_brace].trim().to_string();
    let body = source[open_brace + 1..close_brace].trim();
    if signature.is_empty() || body.is_empty() {
        return None;
    }
    Some((signature, trim_hover_snippet(body)))
}

fn rule_snippet_around_position(source: &str, position: ParserPositionV0) -> Option<String> {
    let line_start = byte_offset_for_parser_position(
        source,
        ParserPositionV0 {
            line: position.line,
            character: 0,
        },
    )?;
    let open_brace = source[line_start..].find('{')? + line_start;
    let mut depth = 0usize;
    let mut cursor = open_brace;
    while cursor < source.len() {
        match source.as_bytes().get(cursor).copied()? {
            b'{' => depth += 1,
            b'}' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    let snippet = source[line_start..=cursor].trim();
                    return Some(trim_hover_snippet(snippet));
                }
            }
            _ => {}
        }
        cursor = advance_style_scan_cursor(source, cursor, source.len());
    }
    None
}

fn line_snippet_at_position(source: &str, position: ParserPositionV0) -> Option<String> {
    let line_start = byte_offset_for_parser_position(
        source,
        ParserPositionV0 {
            line: position.line,
            character: 0,
        },
    )?;
    let line_end = source[line_start..]
        .find('\n')
        .map(|offset| line_start + offset)
        .unwrap_or(source.len());
    Some(source[line_start..line_end].trim().to_string())
}

fn trim_hover_snippet(snippet: &str) -> String {
    const MAX_SNIPPET_LEN: usize = 1200;
    if snippet.len() <= MAX_SNIPPET_LEN {
        return snippet.to_string();
    }
    let end = char_boundary_floor(snippet, MAX_SNIPPET_LEN);
    format!("{}...", snippet[..end].trim_end())
}

fn custom_property_ref_byte_spans(source: &str, name: &str) -> Vec<ParserByteSpanV0> {
    let mut spans = Vec::new();
    let mut search_offset = 0usize;

    while let Some(relative_match) = source[search_offset..].find(name) {
        let name_start = search_offset + relative_match;
        let name_end = name_start + name.len();
        if source[..name_start].trim_end().ends_with("var(")
            && is_selector_name_boundary(source, name_end)
        {
            spans.push(ParserByteSpanV0 {
                start: name_start,
                end: name_end,
            });
        }
        search_offset += relative_match + name.len();
    }

    spans
}

fn is_selector_name_boundary(source: &str, byte_offset: usize) -> bool {
    source[byte_offset..]
        .chars()
        .next()
        .is_none_or(|ch| !is_css_identifier_continue(ch))
}

fn is_css_identifier_continue(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_')
}

fn parser_range_for_byte_span(source: &str, span: ParserByteSpanV0) -> ParserRangeV0 {
    ParserRangeV0 {
        start: parser_position_for_byte_offset(source, span.start),
        end: parser_position_for_byte_offset(source, span.end),
    }
}

fn end_of_source_range(source: &str) -> ParserRangeV0 {
    let position = parser_position_for_byte_offset(source, source.len());
    ParserRangeV0 {
        start: position,
        end: position,
    }
}

fn parser_position_for_byte_offset(source: &str, offset: usize) -> ParserPositionV0 {
    let clamped_offset = offset.min(source.len());
    let mut line = 0usize;
    let mut character = 0usize;

    for (byte_index, ch) in source.char_indices() {
        if byte_index >= clamped_offset {
            break;
        }
        if ch == '\n' {
            line += 1;
            character = 0;
        } else {
            character += ch.len_utf16();
        }
    }

    ParserPositionV0 { line, character }
}

fn byte_offset_for_parser_position(source: &str, position: ParserPositionV0) -> Option<usize> {
    let mut current_line = 0usize;
    let mut current_character = 0usize;

    if position.line == 0 && position.character == 0 {
        return Some(0);
    }

    for (byte_index, ch) in source.char_indices() {
        if current_line == position.line && current_character == position.character {
            return Some(byte_index);
        }
        if ch == '\n' {
            current_line += 1;
            current_character = 0;
            if current_line == position.line && position.character == 0 {
                return Some(byte_index + ch.len_utf8());
            }
        } else if current_line == position.line {
            current_character += ch.len_utf16();
        }
    }

    (current_line == position.line && current_character == position.character)
        .then_some(source.len())
}

fn skip_ascii_whitespace(source: &str, mut offset: usize) -> usize {
    while source
        .as_bytes()
        .get(offset)
        .is_some_and(u8::is_ascii_whitespace)
    {
        offset += 1;
    }
    offset
}

fn matching_style_block_end(
    source: &str,
    open_offset: usize,
    open: u8,
    close: u8,
) -> Option<usize> {
    if source.as_bytes().get(open_offset) != Some(&open) {
        return None;
    }
    let mut cursor = advance_style_scan_cursor(source, open_offset, source.len());
    let mut depth = 1usize;
    while cursor < source.len() {
        match source.as_bytes().get(cursor).copied()? {
            b'\'' | b'"' | b'`' => {
                cursor = skip_style_string_literal(source, cursor, source.len())?;
            }
            byte if byte == open => {
                depth += 1;
                cursor = advance_style_scan_cursor(source, cursor, source.len());
            }
            byte if byte == close => {
                depth -= 1;
                if depth == 0 {
                    return Some(cursor);
                }
                cursor = advance_style_scan_cursor(source, cursor, source.len());
            }
            _ => cursor = advance_style_scan_cursor(source, cursor, source.len()),
        }
    }
    None
}

fn split_top_level_style_segments(
    source: &str,
    start: usize,
    end: usize,
    delimiter: u8,
) -> Vec<(usize, usize)> {
    let mut segments = Vec::new();
    let end = char_boundary_floor(source, end);
    let mut segment_start = char_boundary_ceil(source, start).min(end);
    let mut cursor = segment_start;
    let mut depth = 0usize;
    while cursor < end {
        match source.as_bytes().get(cursor).copied() {
            Some(b'\'' | b'"' | b'`') => {
                cursor = skip_style_string_literal(source, cursor, end).unwrap_or(end);
            }
            Some(b'(' | b'[' | b'{') => {
                depth += 1;
                cursor = advance_style_scan_cursor(source, cursor, end);
            }
            Some(b')' | b']' | b'}') => {
                depth = depth.saturating_sub(1);
                cursor = advance_style_scan_cursor(source, cursor, end);
            }
            Some(byte) if byte == delimiter && depth == 0 => {
                segments.push((segment_start, cursor));
                cursor = advance_style_scan_cursor(source, cursor, end);
                segment_start = cursor;
            }
            Some(_) => cursor = advance_style_scan_cursor(source, cursor, end),
            None => break,
        }
    }
    if segment_start <= end {
        segments.push((segment_start, end));
    }
    segments
}

fn skip_style_string_literal(source: &str, quote_offset: usize, limit: usize) -> Option<usize> {
    let quote = source.as_bytes().get(quote_offset).copied()?;
    let limit = char_boundary_floor(source, limit);
    let mut cursor = quote_offset + 1;
    while cursor < limit {
        let byte = source.as_bytes().get(cursor).copied()?;
        if byte == b'\\' {
            cursor = advance_style_escaped_char(source, cursor, limit);
            continue;
        }
        if byte == quote {
            return Some(cursor + 1);
        }
        cursor = advance_style_scan_cursor(source, cursor, limit);
    }
    None
}

fn advance_style_escaped_char(source: &str, slash_offset: usize, limit: usize) -> usize {
    let after_slash = advance_style_scan_cursor(source, slash_offset, limit);
    advance_style_scan_cursor(source, after_slash, limit)
}

fn advance_style_scan_cursor(source: &str, cursor: usize, limit: usize) -> usize {
    let cursor = char_boundary_ceil(source, cursor);
    let limit = char_boundary_floor(source, limit);
    if cursor >= limit {
        return limit;
    }
    char_boundary_ceil(source, cursor + 1).min(limit)
}

fn char_boundary_floor(source: &str, index: usize) -> usize {
    let mut index = index.min(source.len());
    while index > 0 && !source.is_char_boundary(index) {
        index -= 1;
    }
    index
}

fn char_boundary_ceil(source: &str, index: usize) -> usize {
    let mut index = index.min(source.len());
    while index < source.len() && !source.is_char_boundary(index) {
        index += 1;
    }
    index
}

fn style_language_label(language: StyleLanguage) -> &'static str {
    match language {
        StyleLanguage::Css => "css",
        StyleLanguage::Scss => "scss",
        StyleLanguage::Less => "less",
    }
}

fn push_unique_string(values: &mut Vec<String>, value: String) {
    if !values.contains(&value) {
        values.push(value);
    }
}

#[cfg(test)]
mod tests {
    use engine_input_producers::{
        ClassExpressionInputV2, EngineInputV2, PositionV2, RangeV2, SourceAnalysisInputV2,
        SourceDocumentV2, StringTypeFactsV2, StyleAnalysisInputV2, StyleDocumentV2,
        StyleSelectorV2, TypeFactEntryV2,
    };

    use super::{
        OmenaQueryExpressionDomainFlowRuntimeV0, OmenaQueryStylePackageManifestV0,
        SelectedQueryAdapterCapabilitiesV0, summarize_omena_query_boundary,
        summarize_omena_query_expression_domain_control_flow_analysis,
        summarize_omena_query_expression_domain_flow_analysis,
        summarize_omena_query_expression_domain_incremental_flow_analysis,
        summarize_omena_query_expression_semantics_canonical_producer_signal,
        summarize_omena_query_expression_semantics_query_fragments,
        summarize_omena_query_fragment_bundle,
        summarize_omena_query_selected_query_adapter_capabilities,
        summarize_omena_query_selector_usage_canonical_producer_signal,
        summarize_omena_query_selector_usage_query_fragments,
        summarize_omena_query_source_resolution_canonical_producer_signal,
        summarize_omena_query_source_resolution_query_fragments,
        summarize_omena_query_source_resolution_runtime,
        summarize_omena_query_style_semantic_graph_batch_from_sources,
        summarize_omena_query_style_semantic_graph_batch_from_sources_with_package_manifests,
        summarize_omena_query_style_semantic_graph_from_source,
    };

    #[test]
    fn summarizes_query_boundary_over_producer_fragments() {
        let input = sample_input();
        let summary = summarize_omena_query_boundary(&input);

        assert_eq!(summary.schema_version, "0");
        assert_eq!(summary.product, "omena-query.boundary");
        assert_eq!(summary.query_engine_name, "omena-query");
        assert_eq!(summary.input_version, "2");
        assert_eq!(
            summary.abstract_value_domain.product,
            "omena-abstract-value.domain"
        );
        assert_eq!(
            summary.selected_query_adapter_capabilities.product,
            "omena-query.selected-query-adapter-capabilities"
        );
        assert_eq!(summary.expression_semantics_query_count, 2);
        assert_eq!(summary.source_resolution_query_count, 2);
        assert_eq!(summary.selector_usage_query_count, 2);
        assert_eq!(summary.total_query_count, 6);
        assert!(
            summary
                .ready_surfaces
                .contains(&"abstractValueProjectionContract")
        );
        assert!(
            summary
                .ready_surfaces
                .contains(&"sourceResolutionResolverBoundary")
        );
        assert!(
            summary
                .delegated_fragment_products
                .contains(&"omena-resolver.boundary")
        );
        assert!(
            summary
                .delegated_fragment_products
                .contains(&"omena-resolver.source-resolution-runtime-index")
        );
        assert!(
            summary
                .delegated_fragment_products
                .contains(&"engine-input-producers.expression-domain-flow-analysis")
        );
        assert!(
            summary
                .delegated_fragment_products
                .contains(&"engine-input-producers.expression-domain-control-flow-analysis")
        );
        assert!(
            summary
                .delegated_fragment_products
                .contains(&"omena-query.expression-domain-incremental-flow-analysis")
        );
        assert!(
            summary
                .ready_surfaces
                .contains(&"expressionDomainFlowAnalysisBoundary")
        );
        assert!(
            summary
                .ready_surfaces
                .contains(&"expressionDomainControlFlowAnalysisBoundary")
        );
        assert!(
            summary
                .ready_surfaces
                .contains(&"expressionDomainSalsaRuntime")
        );
        assert!(
            summary
                .ready_surfaces
                .contains(&"sourceResolutionRuntimeIndex")
        );
        assert!(
            summary
                .cme_coupled_surfaces
                .contains(&"producerQueryFragments")
        );
    }

    #[test]
    fn bundles_expression_source_and_selector_query_fragments() {
        let input = sample_input();
        let bundle = summarize_omena_query_fragment_bundle(&input);

        assert_eq!(bundle.schema_version, "0");
        assert_eq!(bundle.product, "omena-query.fragment-bundle");
        assert_eq!(bundle.input_version, "2");
        assert_eq!(bundle.expression_semantics.fragments.len(), 2);
        assert_eq!(bundle.expression_semantics.fragments[0].query_id, "expr-1");
        assert_eq!(bundle.source_resolution.fragments.len(), 2);
        assert_eq!(bundle.source_resolution.fragments[1].query_id, "expr-2");
        assert_eq!(bundle.selector_usage.fragments.len(), 2);
        assert_eq!(bundle.selector_usage.fragments[0].query_id, "btn-active");

        let expression = summarize_omena_query_expression_semantics_query_fragments(&input);
        let source = summarize_omena_query_source_resolution_query_fragments(&input);
        let selector = summarize_omena_query_selector_usage_query_fragments(&input);

        assert_eq!(expression.schema_version, "0");
        assert_eq!(source.schema_version, "0");
        assert_eq!(selector.schema_version, "0");
        assert_eq!(expression.input_version, "2");
        assert_eq!(source.input_version, "2");
        assert_eq!(selector.input_version, "2");
        assert_eq!(
            expression.fragments.len(),
            bundle.expression_semantics.fragments.len()
        );
        assert_eq!(
            source.fragments.len(),
            bundle.source_resolution.fragments.len()
        );
        assert_eq!(
            selector.fragments.len(),
            bundle.selector_usage.fragments.len()
        );
    }

    #[test]
    fn declares_selected_query_adapter_capabilities_without_flipping_runtime_routing() {
        let summary = summarize_omena_query_selected_query_adapter_capabilities();

        assert_eq!(summary.schema_version, "0");
        assert_eq!(
            summary.product,
            "omena-query.selected-query-adapter-capabilities"
        );
        assert_eq!(summary.default_candidate_backend, "rust-selected-query");
        assert_eq!(summary.routing_status, "declaredOnly");

        let unified = backend(&summary, "rust-selected-query");
        assert!(unified.is_some());
        let Some(unified) = unified else {
            return;
        };
        assert!(unified.source_resolution);
        assert!(unified.expression_semantics);
        assert!(unified.selector_usage);
        assert!(unified.style_semantic_graph);

        let source_only = backend(&summary, "rust-source-resolution");
        assert!(source_only.is_some());
        let Some(source_only) = source_only else {
            return;
        };
        assert!(source_only.source_resolution);
        assert!(!source_only.expression_semantics);
        assert!(!source_only.selector_usage);
        assert!(!source_only.style_semantic_graph);

        assert!(
            summary
                .runner_commands
                .iter()
                .any(|command| command.command == "input-omena-resolver-source-resolution-runtime")
        );
        assert!(
            summary
                .runner_commands
                .iter()
                .any(|command| command.command == "input-expression-domain-flow-analysis")
        );
        assert!(
            summary.runner_commands.iter().any(|command| {
                command.command == "input-expression-domain-control-flow-analysis"
            })
        );
        assert!(summary.runner_commands.iter().any(|command| {
            command.command == "input-expression-domain-incremental-flow-analysis"
        }));
        assert!(
            summary
                .runner_commands
                .iter()
                .any(|command| command.command == "style-semantic-graph-batch")
        );
        assert!(
            summary
                .expression_semantics_payload_contracts
                .contains(&"valueDomainDerivation")
        );
        assert!(summary.adapter_readiness.contains(&"runnerCommandContract"));
        assert!(
            summary
                .adapter_readiness
                .contains(&"canonicalProducerWrapperBoundary")
        );
        assert!(
            summary
                .adapter_readiness
                .contains(&"styleSemanticGraphBridgeBoundary")
        );
        assert!(
            summary
                .adapter_readiness
                .contains(&"expressionDomainFlowAnalysisRunner")
        );
        assert!(
            summary
                .adapter_readiness
                .contains(&"expressionDomainControlFlowAnalysisRunner")
        );
        assert!(
            summary
                .adapter_readiness
                .contains(&"expressionDomainSalsaRuntime")
        );
        assert!(
            summary
                .adapter_readiness
                .contains(&"sourceResolutionRuntimeIndex")
        );
    }

    #[test]
    fn owns_expression_domain_flow_analysis_wrapper_without_changing_product() {
        let input = sample_input();
        let summary = summarize_omena_query_expression_domain_flow_analysis(&input);

        assert_eq!(summary.schema_version, "0");
        assert_eq!(
            summary.product,
            "engine-input-producers.expression-domain-flow-analysis"
        );
        assert_eq!(summary.input_version, "2");
        assert_eq!(summary.analyses.len(), 2);
        assert!(
            summary
                .analyses
                .iter()
                .all(|entry| entry.analysis.product == "omena-abstract-value.flow-analysis")
        );
        assert!(
            summary
                .analyses
                .iter()
                .all(|entry| entry.analysis.converged)
        );
    }

    #[test]
    fn owns_expression_domain_control_flow_analysis_wrapper_without_changing_product() {
        let input = sample_input();
        let summary = summarize_omena_query_expression_domain_control_flow_analysis(&input);

        assert_eq!(summary.schema_version, "0");
        assert_eq!(
            summary.product,
            "engine-input-producers.expression-domain-control-flow-analysis"
        );
        assert_eq!(summary.input_version, "2");
        assert_eq!(summary.analyses.len(), 2);
        assert!(
            summary
                .analyses
                .iter()
                .all(|entry| entry.analysis.product
                    == "omena-abstract-value.control-flow-analysis")
        );
    }

    #[test]
    fn reuses_expression_domain_flow_analysis_through_salsa_runtime() {
        let input = sample_input();
        let mut runtime = OmenaQueryExpressionDomainFlowRuntimeV0::default();

        let first =
            summarize_omena_query_expression_domain_incremental_flow_analysis(&input, &mut runtime);
        assert_eq!(
            first.product,
            "omena-query.expression-domain-incremental-flow-analysis"
        );
        assert_eq!(first.revision, 1);
        assert_eq!(first.graph_count, 2);
        assert_eq!(first.dirty_graph_count, 2);
        assert_eq!(first.reused_graph_count, 0);
        assert_eq!(runtime.graph_count(), 2);

        let second =
            summarize_omena_query_expression_domain_incremental_flow_analysis(&input, &mut runtime);
        assert_eq!(second.revision, 2);
        assert_eq!(second.graph_count, 2);
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
    fn owns_selected_query_canonical_producer_wrappers_without_changing_products() {
        let input = sample_input();

        let source = summarize_omena_query_source_resolution_canonical_producer_signal(&input);
        assert_eq!(source.schema_version, "0");
        assert_eq!(source.input_version, "2");
        assert_eq!(source.canonical_bundle.query_fragments.len(), 2);
        assert_eq!(source.evaluator_candidates.results.len(), 2);

        let expression =
            summarize_omena_query_expression_semantics_canonical_producer_signal(&input);
        assert_eq!(expression.schema_version, "0");
        assert_eq!(expression.input_version, "2");
        assert_eq!(expression.canonical_bundle.query_fragments.len(), 2);
        assert_eq!(expression.evaluator_candidates.results.len(), 2);
        assert_eq!(
            expression.evaluator_candidates.results[0]
                .payload
                .value_domain_derivation
                .product,
            "omena-abstract-value.reduced-class-value-derivation"
        );
        assert_eq!(
            expression.evaluator_candidates.results[0]
                .payload
                .value_domain_derivation
                .reduced_kind,
            "prefixSuffix"
        );

        let selector = summarize_omena_query_selector_usage_canonical_producer_signal(&input);
        assert_eq!(selector.schema_version, "0");
        assert_eq!(selector.input_version, "2");
        assert_eq!(selector.canonical_bundle.query_fragments.len(), 2);
        assert_eq!(selector.evaluator_candidates.results.len(), 2);
    }

    #[test]
    fn owns_source_resolution_runtime_index_wrapper() {
        let input = sample_input();
        let runtime_index = summarize_omena_query_source_resolution_runtime(&input);

        assert_eq!(
            runtime_index.product,
            "omena-resolver.source-resolution-runtime-index"
        );
        assert_eq!(runtime_index.expression_count, 2);
        assert_eq!(runtime_index.resolved_expression_count, 2);
        assert_eq!(runtime_index.unresolved_expression_count, 0);
        assert!(
            runtime_index
                .entries
                .iter()
                .any(|entry| entry.expression_id == "expr-1"
                    && entry.selector_names == ["btn-active"])
        );
    }

    #[test]
    fn owns_style_semantic_graph_adapter_boundary_without_changing_graph_product() {
        let input = sample_input();
        let graph = summarize_omena_query_style_semantic_graph_from_source(
            "/tmp/App.module.scss",
            ".btn-active { color: red; }",
            &input,
        );
        assert!(graph.is_some());
        let Some(graph) = graph else {
            return;
        };
        assert_eq!(graph.schema_version, "0");
        assert_eq!(graph.product, "omena-semantic.style-semantic-graph");
        assert_eq!(graph.selector_identity_engine.canonical_ids.len(), 1);

        let batch = summarize_omena_query_style_semantic_graph_batch_from_sources(
            [
                ("/tmp/App.module.scss", ".btn-active { color: red; }"),
                ("/tmp/Card.module.scss", ".card-header { color: blue; }"),
            ],
            &input,
        );
        assert_eq!(batch.schema_version, "0");
        assert_eq!(batch.product, "omena-semantic.style-semantic-graph-batch");
        assert_eq!(batch.graphs.len(), 2);
        assert_eq!(batch.graphs[0].style_path, "/tmp/App.module.scss");
        assert!(batch.graphs[0].graph.is_some());
        assert!(batch.graphs[1].graph.is_some());
    }

    #[test]
    fn style_semantic_graph_batch_feeds_workspace_design_token_candidates() {
        let input = sample_input();
        let batch = summarize_omena_query_style_semantic_graph_batch_from_sources(
            [
                ("/tmp/tokens.module.scss", ":root { --brand: red; }"),
                ("/tmp/theme.module.scss", "@forward \"./tokens\";"),
                ("/tmp/unrelated.module.scss", ":root { --brand: blue; }"),
                (
                    "/tmp/App.module.scss",
                    "@use \"./theme\";\n.button { color: var(--brand); }",
                ),
            ],
            &input,
        );

        let app_graph = batch
            .graphs
            .iter()
            .find(|entry| entry.style_path == "/tmp/App.module.scss")
            .and_then(|entry| entry.graph.as_ref());
        assert!(app_graph.is_some());
        let Some(app_graph) = app_graph else {
            return;
        };
        let design_tokens = &app_graph.design_token_semantics;

        assert_eq!(
            design_tokens.status,
            "cross-file-import-cascade-ranking-seed"
        );
        assert_eq!(
            design_tokens.resolution_scope,
            "cross-file-import-candidate"
        );
        assert!(
            design_tokens
                .capabilities
                .workspace_cascade_candidate_signal_ready
        );
        assert!(design_tokens.capabilities.cross_file_import_graph_ready);
        assert_eq!(
            design_tokens
                .resolution_signal
                .cross_file_declaration_fact_count,
            1
        );
        assert_eq!(
            design_tokens
                .resolution_signal
                .workspace_occurrence_resolved_reference_count,
            1
        );
        assert_eq!(
            design_tokens
                .cascade_ranking_signal
                .cross_file_candidate_declaration_count,
            1
        );
        assert_eq!(
            design_tokens
                .cascade_ranking_signal
                .cross_file_winner_declaration_count,
            1
        );
        assert_eq!(
            design_tokens.cascade_ranking_signal.ranked_references[0]
                .winner_declaration_file_path
                .as_deref(),
            Some("/tmp/tokens.module.scss")
        );
        let winner_range =
            design_tokens.cascade_ranking_signal.ranked_references[0].winner_declaration_range;
        assert_eq!(winner_range.map(|range| range.start.line), Some(0));
        assert_eq!(winner_range.map(|range| range.start.character), Some(8));
        assert_eq!(design_tokens.declaration_candidates.len(), 1);
        let declaration_candidate = &design_tokens.declaration_candidates[0];
        assert_eq!(declaration_candidate.name, "--brand");
        assert_eq!(declaration_candidate.file_path, "/tmp/tokens.module.scss");
        assert_eq!(
            declaration_candidate.candidate_scope,
            "cross-file-import-candidate"
        );
        assert!(declaration_candidate.import_graph_distance.is_some());
        assert_eq!(
            design_tokens.cascade_ranking_signal.ranked_references[0]
                .cross_file_candidate_declaration_count,
            1
        );
    }

    #[test]
    fn style_semantic_graph_batch_prefers_nearer_import_graph_token_candidates() {
        let input = sample_input();
        let batch = summarize_omena_query_style_semantic_graph_batch_from_sources(
            [
                ("/tmp/a-direct.module.scss", ":root { --brand: direct; }"),
                ("/tmp/mid.module.scss", "@forward \"./z-transitive\";"),
                (
                    "/tmp/z-transitive.module.scss",
                    ":root { --brand: transitive; }",
                ),
                (
                    "/tmp/App.module.scss",
                    "@use \"./a-direct\";\n@use \"./mid\";\n.button { color: var(--brand); }",
                ),
            ],
            &input,
        );

        let app_graph = batch
            .graphs
            .iter()
            .find(|entry| entry.style_path == "/tmp/App.module.scss")
            .and_then(|entry| entry.graph.as_ref());
        assert!(app_graph.is_some());
        let Some(app_graph) = app_graph else {
            return;
        };
        let ranked_reference = &app_graph
            .design_token_semantics
            .cascade_ranking_signal
            .ranked_references[0];

        assert_eq!(
            ranked_reference.winner_declaration_file_path.as_deref(),
            Some("/tmp/a-direct.module.scss")
        );
        assert_eq!(ranked_reference.winner_import_graph_distance, Some(1));
        assert_eq!(ranked_reference.winner_import_graph_order, Some(0));
        assert_eq!(ranked_reference.cross_file_candidate_declaration_count, 2);
        assert_eq!(ranked_reference.cross_file_shadowed_declaration_count, 1);
    }

    #[test]
    fn style_semantic_graph_batch_resolves_package_root_forward_chain_token_candidates() {
        let input = sample_input();
        let batch = summarize_omena_query_style_semantic_graph_batch_from_sources(
            [
                (
                    "/fake/workspace/node_modules/@design/tokens/src/index.scss",
                    "@forward \"./colors\";",
                ),
                (
                    "/fake/workspace/node_modules/@design/tokens/src/_colors.scss",
                    ":root { --brand: package; }",
                ),
                (
                    "/fake/workspace/src/_utils.scss",
                    "@forward \"@design/tokens\" as ds_*;",
                ),
                (
                    "/fake/workspace/src/App.module.scss",
                    "@use \"./utils\";\n.button { color: var(--brand); }",
                ),
            ],
            &input,
        );

        let app_graph = batch
            .graphs
            .iter()
            .find(|entry| entry.style_path == "/fake/workspace/src/App.module.scss")
            .and_then(|entry| entry.graph.as_ref());
        assert!(app_graph.is_some());
        let Some(app_graph) = app_graph else {
            return;
        };
        let ranked_reference = &app_graph
            .design_token_semantics
            .cascade_ranking_signal
            .ranked_references[0];

        assert_eq!(
            ranked_reference.winner_declaration_file_path.as_deref(),
            Some("/fake/workspace/node_modules/@design/tokens/src/_colors.scss")
        );
        assert_eq!(ranked_reference.winner_import_graph_distance, Some(3));
        assert_eq!(ranked_reference.cross_file_candidate_declaration_count, 1);
    }

    #[test]
    fn style_semantic_graph_batch_resolves_package_manifest_style_exports() {
        let input = sample_input();
        let batch =
            summarize_omena_query_style_semantic_graph_batch_from_sources_with_package_manifests(
                [
                    (
                        "/fake/workspace/node_modules/@design/tokens/dist/theme.css",
                        ":root { --brand: package; }",
                    ),
                    (
                        "/fake/workspace/src/App.module.scss",
                        "@use \"@design/tokens/theme\";\n.button { color: var(--brand); }",
                    ),
                ],
                &input,
                &[OmenaQueryStylePackageManifestV0 {
                    package_json_path: "/fake/workspace/node_modules/@design/tokens/package.json"
                        .to_string(),
                    package_json_source: r#"{"exports":{"./theme":{"style":"./dist/theme.css"}}}"#
                        .to_string(),
                }],
            );

        let app_graph = batch
            .graphs
            .iter()
            .find(|entry| entry.style_path == "/fake/workspace/src/App.module.scss")
            .and_then(|entry| entry.graph.as_ref());
        assert!(app_graph.is_some());
        let Some(app_graph) = app_graph else {
            return;
        };
        let ranked_reference = &app_graph
            .design_token_semantics
            .cascade_ranking_signal
            .ranked_references[0];

        assert_eq!(
            ranked_reference.winner_declaration_file_path.as_deref(),
            Some("/fake/workspace/node_modules/@design/tokens/dist/theme.css")
        );
        assert_eq!(ranked_reference.winner_import_graph_distance, Some(1));
        assert_eq!(ranked_reference.cross_file_candidate_declaration_count, 1);
        let declaration_candidate = &app_graph.design_token_semantics.declaration_candidates[0];
        assert_eq!(declaration_candidate.name, "--brand");
        assert_eq!(
            declaration_candidate.file_path,
            "/fake/workspace/node_modules/@design/tokens/dist/theme.css"
        );
        assert_eq!(
            declaration_candidate.candidate_scope,
            "cross-file-import-candidate"
        );
        assert_eq!(declaration_candidate.import_graph_distance, Some(1));
    }

    #[test]
    fn style_hover_candidates_are_query_owned() {
        let candidates = super::summarize_omena_query_style_hover_candidates(
            "Component.module.scss",
            r#"
@mixin variants($prefix, $map) {
  @each $name, $value in $map {
    .#{$prefix}-#{$name} { color: $value; }
  }
}

$accent: red;
.button { color: var(--brand); }
:root { --brand: blue; }
@include variants($prefix: "tone", $map: ("warm": red));
"#,
        );
        assert!(candidates.is_some());
        let Some(candidates) = candidates else {
            return;
        };

        assert_eq!(candidates.product, "omena-query.style-hover-candidates");
        assert!(
            candidates
                .candidates
                .iter()
                .any(|candidate| candidate.kind == "selector" && candidate.name == "button")
        );
        assert!(candidates.candidates.iter().any(|candidate| {
            candidate.kind == "customPropertyReference" && candidate.name == "--brand"
        }));
        assert!(candidates.candidates.iter().any(|candidate| {
            candidate.kind == "sassVariableDeclaration" && candidate.name == "accent"
        }));
        assert!(
            candidates
                .candidates
                .iter()
                .any(
                    |candidate| candidate.source == "sassPartialEvaluatorGeneratedSelectors"
                        && candidate.name == "tone-warm"
                )
        );
    }

    #[test]
    fn style_hover_render_parts_are_query_owned() {
        let source = r#"$accent: red !default;
@mixin tone($color) {
  color: $color;
}
.button { color: var(--brand); }
"#;

        let variable = super::summarize_omena_query_style_hover_render_parts(
            source,
            "sassVariableDeclaration",
            "accent",
            engine_style_parser::ParserPositionV0 {
                line: 0,
                character: 1,
            },
        );
        assert_eq!(variable.product, "omena-query.style-hover-render-parts");
        assert_eq!(variable.value.as_deref(), Some("red"));
        assert_eq!(variable.snippet, "$accent: red !default;");

        let mixin = super::summarize_omena_query_style_hover_render_parts(
            source,
            "sassMixinDeclaration",
            "tone",
            engine_style_parser::ParserPositionV0 {
                line: 1,
                character: 7,
            },
        );
        assert_eq!(mixin.signature.as_deref(), Some("@mixin tone($color)"));
        assert_eq!(mixin.snippet, "color: $color;");
        assert_eq!(mixin.render_source, "callableBlockSnippet");

        let selector = super::summarize_omena_query_style_hover_render_parts(
            source,
            "selector",
            "button",
            engine_style_parser::ParserPositionV0 {
                line: 4,
                character: 1,
            },
        );
        assert_eq!(selector.snippet, ".button { color: var(--brand); }");
        assert_eq!(selector.render_source, "ruleSnippet");
    }

    #[test]
    fn missing_custom_property_diagnostics_are_query_owned() {
        let source = ":root { --brand: red; }\n.alert { color: var(--missing); }";
        let candidates =
            super::summarize_omena_query_style_hover_candidates("Component.module.scss", source);
        assert!(candidates.is_some());
        let Some(candidates) = candidates else {
            return;
        };

        let diagnostics = super::summarize_omena_query_missing_custom_property_diagnostics(
            "file:///workspace/src/Component.module.scss",
            source,
            candidates.candidates.as_slice(),
        );

        assert_eq!(diagnostics.len(), 1);
        let diagnostic = &diagnostics[0];
        assert_eq!(diagnostic.code, "missingCustomProperty");
        assert_eq!(
            diagnostic.message,
            "CSS custom property '--missing' not found in indexed style tokens."
        );
        assert_eq!(
            diagnostic.range,
            engine_style_parser::ParserRangeV0 {
                start: engine_style_parser::ParserPositionV0 {
                    line: 1,
                    character: 20,
                },
                end: engine_style_parser::ParserPositionV0 {
                    line: 1,
                    character: 29,
                },
            }
        );
        assert_eq!(
            diagnostic
                .create_custom_property
                .as_ref()
                .map(|action| action.new_text.as_str()),
            Some("\n\n:root {\n  --missing: ;\n}\n")
        );
        assert_eq!(
            diagnostic
                .create_custom_property
                .as_ref()
                .map(|action| action.range),
            Some(engine_style_parser::ParserRangeV0 {
                start: engine_style_parser::ParserPositionV0 {
                    line: 1,
                    character: 33,
                },
                end: engine_style_parser::ParserPositionV0 {
                    line: 1,
                    character: 33,
                },
            })
        );
    }

    #[test]
    fn sass_module_sources_are_query_owned() {
        let sources = super::summarize_omena_query_sass_module_sources(
            "Component.module.scss",
            r#"
@use "./tokens" as tokens;
@use "./reset" as *;
@forward "./theme";
"#,
        );
        assert!(sources.is_some());
        let Some(sources) = sources else {
            return;
        };

        assert_eq!(sources.product, "omena-query.sass-module-sources");
        assert!(sources.module_use_edges.iter().any(|edge| {
            edge.source == "./tokens"
                && edge.namespace.as_deref() == Some("tokens")
                && edge.namespace_kind == "alias"
        }));
        assert!(sources.module_use_edges.iter().any(|edge| {
            edge.source == "./reset"
                && edge.namespace.is_none()
                && edge.namespace_kind == "wildcard"
        }));
        assert_eq!(sources.module_forward_sources, vec!["./theme".to_string()]);
    }

    fn backend<'a>(
        summary: &'a SelectedQueryAdapterCapabilitiesV0,
        backend_kind: &str,
    ) -> Option<&'a super::SelectedQueryBackendCapabilityV0> {
        summary
            .backend_kinds
            .iter()
            .find(|backend| backend.backend_kind == backend_kind)
    }

    fn sample_input() -> EngineInputV2 {
        EngineInputV2 {
            version: "2".to_string(),
            sources: vec![SourceAnalysisInputV2 {
                document: SourceDocumentV2 {
                    class_expressions: vec![
                        ClassExpressionInputV2 {
                            id: "expr-1".to_string(),
                            kind: "symbolRef".to_string(),
                            scss_module_path: "/tmp/App.module.scss".to_string(),
                            range: range(4, 12, 4, 16),
                            class_name: None,
                            root_binding_decl_id: Some("decl-1".to_string()),
                            access_path: None,
                        },
                        ClassExpressionInputV2 {
                            id: "expr-2".to_string(),
                            kind: "styleAccess".to_string(),
                            scss_module_path: "/tmp/Card.module.scss".to_string(),
                            range: range(6, 9, 6, 20),
                            class_name: Some("card-header".to_string()),
                            root_binding_decl_id: None,
                            access_path: Some(vec!["card".to_string(), "header".to_string()]),
                        },
                    ],
                },
            }],
            styles: vec![
                StyleAnalysisInputV2 {
                    file_path: "/tmp/App.module.scss".to_string(),
                    document: StyleDocumentV2 {
                        selectors: vec![StyleSelectorV2 {
                            name: "btn-active".to_string(),
                            view_kind: "canonical".to_string(),
                            canonical_name: Some("btn-active".to_string()),
                            range: range(1, 1, 1, 12),
                            nested_safety: Some("safe".to_string()),
                            composes: None,
                            bem_suffix: None,
                        }],
                    },
                },
                StyleAnalysisInputV2 {
                    file_path: "/tmp/Card.module.scss".to_string(),
                    document: StyleDocumentV2 {
                        selectors: vec![StyleSelectorV2 {
                            name: "card-header".to_string(),
                            view_kind: "canonical".to_string(),
                            canonical_name: Some("card-header".to_string()),
                            range: range(3, 1, 3, 13),
                            nested_safety: Some("unsafe".to_string()),
                            composes: None,
                            bem_suffix: None,
                        }],
                    },
                },
            ],
            type_facts: vec![
                TypeFactEntryV2 {
                    file_path: "/tmp/App.tsx".to_string(),
                    expression_id: "expr-1".to_string(),
                    facts: StringTypeFactsV2 {
                        kind: "constrained".to_string(),
                        constraint_kind: Some("prefixSuffix".to_string()),
                        values: None,
                        prefix: Some("btn-".to_string()),
                        suffix: Some("-active".to_string()),
                        min_len: Some(10),
                        max_len: None,
                        char_must: None,
                        char_may: None,
                        may_include_other_chars: None,
                    },
                },
                TypeFactEntryV2 {
                    file_path: "/tmp/Card.tsx".to_string(),
                    expression_id: "expr-2".to_string(),
                    facts: StringTypeFactsV2 {
                        kind: "finiteSet".to_string(),
                        constraint_kind: None,
                        values: Some(vec!["card-header".to_string(), "card-body".to_string()]),
                        prefix: None,
                        suffix: None,
                        min_len: None,
                        max_len: None,
                        char_must: None,
                        char_may: None,
                        may_include_other_chars: None,
                    },
                },
            ],
        }
    }

    fn range(
        start_line: usize,
        start_character: usize,
        end_line: usize,
        end_character: usize,
    ) -> RangeV2 {
        RangeV2 {
            start: PositionV2 {
                line: start_line,
                character: start_character,
            },
            end: PositionV2 {
                line: end_line,
                character: end_character,
            },
        }
    }
}
