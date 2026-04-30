use engine_input_producers::{
    EngineInputV2, ExpressionDomainFlowAnalysisV0, ExpressionSemanticsCanonicalProducerSignalV0,
    ExpressionSemanticsQueryFragmentsV0, SelectorUsageCanonicalProducerSignalV0,
    SelectorUsageQueryFragmentsV0, SourceResolutionCanonicalProducerSignalV0,
    SourceResolutionQueryFragmentsV0, collect_expression_domain_flow_graphs,
    summarize_expression_domain_flow_analysis_input,
    summarize_expression_semantics_canonical_producer_signal_input,
    summarize_expression_semantics_query_fragments_input,
    summarize_selector_usage_canonical_producer_signal_input,
    summarize_selector_usage_query_fragments_input,
};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::path::{Component, Path, PathBuf};

use engine_style_parser::{Stylesheet, parse_style_module, summarize_css_modules_intermediate};
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
            "expressionDomainSalsaRuntime",
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

pub fn summarize_omena_query_style_semantic_graph_batch_from_sources<'a>(
    styles: impl IntoIterator<Item = (&'a str, &'a str)>,
    input: &EngineInputV2,
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
) -> Vec<DesignTokenWorkspaceDeclarationFactV0> {
    let reachable_style_paths =
        collect_import_reachable_style_path_metadata(target_style_path, parsed_styles);
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
            if let Some(next_style_path) =
                resolve_style_module_source(&style_path, &source, &available_style_paths)
            {
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
) -> Vec<String> {
    let Some(target_sheet) = parsed_style_by_path(parsed_styles, target_style_path) else {
        return Vec::new();
    };
    collect_sass_module_sources(target_sheet)
        .into_iter()
        .filter_map(|source| {
            resolve_style_module_source(target_style_path, &source, available_style_paths)
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
) -> Option<String> {
    if source.starts_with("sass:")
        || source.starts_with("http://")
        || source.starts_with("https://")
    {
        return None;
    }

    style_module_source_candidates(from_style_path, source)
        .into_iter()
        .find(|candidate| available_style_paths.contains(candidate.as_str()))
}

fn style_module_source_candidates(from_style_path: &str, source: &str) -> Vec<String> {
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
        OmenaQueryExpressionDomainFlowRuntimeV0, SelectedQueryAdapterCapabilitiesV0,
        summarize_omena_query_boundary, summarize_omena_query_expression_domain_flow_analysis,
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
