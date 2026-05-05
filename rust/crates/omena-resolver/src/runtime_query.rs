use super::*;

pub fn summarize_omena_resolver_runtime_query_boundary(
    module_graph: &OmenaResolverModuleGraphSummaryV0,
) -> OmenaResolverRuntimeQueryBoundarySummaryV0 {
    let module_queries = module_graph
        .modules
        .iter()
        .map(runtime_module_query_from_graph_module)
        .collect::<Vec<_>>();
    let fully_resolvable_module_count = module_queries
        .iter()
        .filter(|module| module.status == "ready")
        .count();
    let source_only_module_count = module_graph
        .modules
        .iter()
        .filter(|module| module.has_source_input && !module.has_style_input)
        .count();
    let style_only_module_count = module_graph
        .modules
        .iter()
        .filter(|module| module.has_style_input && !module.has_source_input)
        .count();
    let mut blocking_gaps = Vec::new();

    if module_graph.module_count == 0 {
        blocking_gaps.push("emptyModuleGraph");
    }
    if fully_resolvable_module_count < module_graph.module_count {
        blocking_gaps.push("partialModuleCoverage");
    }
    if module_graph.unresolved_type_fact_count > 0 {
        blocking_gaps.push("unresolvedTypeFactEdges");
    }

    OmenaResolverRuntimeQueryBoundarySummaryV0 {
        schema_version: "0",
        product: "omena-resolver.runtime-query-boundary",
        input_product: module_graph.product.clone(),
        input_version: module_graph.input_version.clone(),
        module_query_count: module_queries.len(),
        fully_resolvable_module_count,
        source_only_module_count,
        style_only_module_count,
        unresolved_type_fact_count: module_graph.unresolved_type_fact_count,
        runtime_capabilities: vec![
            "moduleLookupByStylePath",
            "sourceExpressionEdgeLookup",
            "typeFactEdgeLookup",
            "selectorNameLookup",
        ],
        blocking_gaps,
        module_queries,
    }
}

pub fn query_omena_resolver_runtime_module(
    module_graph: &OmenaResolverModuleGraphSummaryV0,
    style_file_path: &str,
) -> Option<OmenaResolverRuntimeModuleQueryV0> {
    module_graph
        .modules
        .iter()
        .find(|module| module.style_file_path == style_file_path)
        .map(runtime_module_query_from_graph_module)
}

fn runtime_module_query_from_graph_module(
    module: &OmenaResolverModuleGraphModuleV0,
) -> OmenaResolverRuntimeModuleQueryV0 {
    OmenaResolverRuntimeModuleQueryV0 {
        style_file_path: module.style_file_path.clone(),
        source_expression_ids: module.source_expression_ids.clone(),
        type_fact_expression_ids: module.type_fact_expression_ids.clone(),
        selector_names: module.selector_names.clone(),
        canonical_selector_names: module.canonical_selector_names.clone(),
        can_resolve_source_expressions: module.has_source_input && module.has_style_input,
        can_check_type_fact_edges: module.has_source_input && module.has_type_fact_input,
        can_query_selector_names: module.has_style_input,
        status: module_runtime_status(module),
    }
}

fn module_runtime_status(module: &OmenaResolverModuleGraphModuleV0) -> &'static str {
    if module.has_source_input && module.has_style_input && module.has_type_fact_input {
        "ready"
    } else if module.has_source_input && !module.has_style_input {
        "sourceOnly"
    } else if module.has_style_input && !module.has_source_input {
        "styleOnly"
    } else {
        "partial"
    }
}
