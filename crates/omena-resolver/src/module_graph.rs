use super::*;

#[derive(Debug, Default)]
struct ModuleGraphAccumulator {
    source_expression_ids: BTreeSet<String>,
    source_expression_kinds: BTreeSet<String>,
    type_fact_expression_ids: BTreeSet<String>,
    selector_names: BTreeSet<String>,
    canonical_selector_names: BTreeSet<String>,
    has_source_input: bool,
    has_style_input: bool,
    has_type_fact_input: bool,
}

pub fn summarize_omena_resolver_module_graph_index(
    input: &EngineInputV2,
) -> OmenaResolverModuleGraphSummaryV0 {
    let mut modules = BTreeMap::<String, ModuleGraphAccumulator>::new();
    let mut expression_to_style_path = BTreeMap::<String, String>::new();
    let mut source_expression_edge_count = 0usize;
    let mut type_fact_edge_count = 0usize;
    let mut selector_count = 0usize;
    let mut unresolved_type_fact_expression_ids = BTreeSet::<String>::new();

    for source in &input.sources {
        for expression in &source.document.class_expressions {
            source_expression_edge_count += 1;
            expression_to_style_path
                .insert(expression.id.clone(), expression.scss_module_path.clone());
            let module = modules
                .entry(expression.scss_module_path.clone())
                .or_default();
            module.has_source_input = true;
            module.source_expression_ids.insert(expression.id.clone());
            module
                .source_expression_kinds
                .insert(expression.kind.clone());
        }
    }

    for style in &input.styles {
        let module = modules.entry(style.file_path.clone()).or_default();
        module.has_style_input = true;
        for selector in &style.document.selectors {
            selector_count += 1;
            module.selector_names.insert(selector.name.clone());
            if let Some(canonical_name) = &selector.canonical_name {
                module
                    .canonical_selector_names
                    .insert(canonical_name.clone());
            }
        }
    }

    for type_fact in &input.type_facts {
        if let Some(style_file_path) = expression_to_style_path.get(&type_fact.expression_id) {
            type_fact_edge_count += 1;
            let module = modules.entry(style_file_path.clone()).or_default();
            module.has_type_fact_input = true;
            module
                .type_fact_expression_ids
                .insert(type_fact.expression_id.clone());
        } else {
            unresolved_type_fact_expression_ids.insert(type_fact.expression_id.clone());
        }
    }

    let modules = modules
        .into_iter()
        .map(
            |(style_file_path, module)| OmenaResolverModuleGraphModuleV0 {
                style_file_path,
                source_expression_ids: module.source_expression_ids.into_iter().collect(),
                source_expression_kinds: module.source_expression_kinds.into_iter().collect(),
                type_fact_expression_ids: module.type_fact_expression_ids.into_iter().collect(),
                selector_names: module.selector_names.into_iter().collect(),
                canonical_selector_names: module.canonical_selector_names.into_iter().collect(),
                has_source_input: module.has_source_input,
                has_style_input: module.has_style_input,
                has_type_fact_input: module.has_type_fact_input,
            },
        )
        .collect::<Vec<_>>();
    let unresolved_type_fact_expression_ids = unresolved_type_fact_expression_ids
        .into_iter()
        .collect::<Vec<_>>();

    OmenaResolverModuleGraphSummaryV0 {
        schema_version: "0".to_string(),
        product: "omena-resolver.module-graph-index".to_string(),
        input_version: input.version.clone(),
        module_count: modules.len(),
        source_expression_edge_count,
        type_fact_edge_count,
        selector_count,
        unresolved_type_fact_count: unresolved_type_fact_expression_ids.len(),
        modules,
        unresolved_type_fact_expression_ids,
    }
}
