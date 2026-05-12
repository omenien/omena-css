use super::*;

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

pub fn summarize_omena_query_expression_domain_selector_projection(
    input: &EngineInputV2,
) -> OmenaQueryExpressionDomainSelectorProjectionV0 {
    let style_selectors_by_path = style_selector_universe_by_path(input);
    let expression_targets = expression_target_style_paths(input);
    let flow_analysis = summarize_omena_query_expression_domain_flow_analysis(input);
    let mut projections = Vec::new();

    for graph in flow_analysis.analyses {
        for node in graph.analysis.nodes {
            let target_style_paths = target_style_paths_for_flow_node(
                node.id.as_str(),
                node.predecessor_ids.as_slice(),
                &expression_targets,
            );
            let selector_universe = selector_universe_for_targets(
                target_style_paths.as_slice(),
                &style_selectors_by_path,
            );
            let projection = project_abstract_value_selectors(&node.value, &selector_universe);
            projections.push(OmenaQueryExpressionDomainSelectorProjectionEntryV0 {
                graph_id: graph.graph_id.clone(),
                file_path: graph.file_path.clone(),
                node_id: node.id,
                target_style_paths,
                value_kind: node.value_kind,
                reduced_product: summarize_reduced_class_value_product(&node.value),
                selector_names: projection.selector_names,
                certainty: projection.certainty,
            });
        }
    }

    OmenaQueryExpressionDomainSelectorProjectionV0 {
        schema_version: "0",
        product: "omena-query.expression-domain-selector-projection",
        input_version: input.version.clone(),
        projection_count: projections.len(),
        projections,
    }
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

fn expression_target_style_paths(input: &EngineInputV2) -> BTreeMap<String, String> {
    input
        .sources
        .iter()
        .flat_map(|source| source.document.class_expressions.iter())
        .map(|expression| (expression.id.clone(), expression.scss_module_path.clone()))
        .collect()
}

fn style_selector_universe_by_path(input: &EngineInputV2) -> BTreeMap<String, Vec<String>> {
    input
        .styles
        .iter()
        .map(|style| {
            let selector_names = style
                .document
                .selectors
                .iter()
                .map(|selector| {
                    selector
                        .canonical_name
                        .clone()
                        .unwrap_or_else(|| selector.name.clone())
                })
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect::<Vec<_>>();
            (style.file_path.clone(), selector_names)
        })
        .collect()
}

fn target_style_paths_for_flow_node(
    node_id: &str,
    predecessor_ids: &[String],
    expression_targets: &BTreeMap<String, String>,
) -> Vec<String> {
    let mut targets = BTreeSet::new();
    if let Some(target) = expression_targets.get(node_id) {
        targets.insert(target.clone());
    }
    for predecessor_id in predecessor_ids {
        if let Some(target) = expression_targets.get(predecessor_id) {
            targets.insert(target.clone());
        }
    }
    targets.into_iter().collect()
}

fn selector_universe_for_targets(
    target_style_paths: &[String],
    style_selectors_by_path: &BTreeMap<String, Vec<String>>,
) -> Vec<String> {
    let mut selectors = BTreeSet::new();
    if target_style_paths.is_empty() {
        for selector_names in style_selectors_by_path.values() {
            selectors.extend(selector_names.iter().cloned());
        }
    } else {
        for target_style_path in target_style_paths {
            if let Some(selector_names) = style_selectors_by_path.get(target_style_path) {
                selectors.extend(selector_names.iter().cloned());
            }
        }
    }
    selectors.into_iter().collect()
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
