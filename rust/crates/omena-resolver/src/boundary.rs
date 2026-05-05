use super::*;

pub fn summarize_omena_resolver_boundary(input: &EngineInputV2) -> OmenaResolverBoundarySummaryV0 {
    let canonical_signal = summarize_omena_resolver_canonical_producer_signal(input);
    let module_graph = summarize_omena_resolver_module_graph_index(input);
    let runtime_query = summarize_omena_resolver_runtime_query_boundary(&module_graph);
    let source_resolution_runtime = summarize_omena_resolver_source_resolution_runtime(input);

    OmenaResolverBoundarySummaryV0 {
        schema_version: "0",
        product: "omena-resolver.boundary",
        resolver_name: "omena-resolver",
        input_version: input.version.clone(),
        delegated_source_resolution_products: vec![
            "engine-input-producers.source-resolution-query-fragments",
            "engine-input-producers.source-resolution-canonical-producer",
        ],
        resolver_owned_products: vec![
            "omena-resolver.module-graph-index",
            "omena-resolver.runtime-query-boundary",
            "omena-resolver.source-resolution-runtime-index",
        ],
        source_resolution_query_count: canonical_signal.canonical_bundle.query_fragments.len(),
        source_resolution_candidate_count: canonical_signal.canonical_bundle.candidates.len(),
        source_resolution_evaluator_candidate_count: canonical_signal
            .evaluator_candidates
            .results
            .len(),
        module_graph_module_count: module_graph.module_count,
        module_graph_source_expression_edge_count: module_graph.source_expression_edge_count,
        runtime_query_module_count: runtime_query.module_query_count,
        runtime_query_ready_module_count: runtime_query.fully_resolvable_module_count,
        source_resolution_runtime_expression_count: source_resolution_runtime.expression_count,
        source_resolution_runtime_resolved_expression_count: source_resolution_runtime
            .resolved_expression_count,
        ready_surfaces: vec![
            "resolverBoundarySummary",
            "resolverModuleGraphIndex",
            "resolverRuntimeQueryBoundary",
            "resolverSourceResolutionRuntimeIndex",
            "sourceResolutionQueryFragments",
            "sourceResolutionCanonicalProducerSignal",
        ],
        cme_coupled_surfaces: vec!["EngineInputV2", "producerSourceResolutionRows"],
        next_decoupling_targets: vec!["specifierResolutionRuntime", "tsconfigPathMapping"],
    }
}
