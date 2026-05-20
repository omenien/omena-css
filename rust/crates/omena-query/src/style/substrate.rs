use super::*;
use std::collections::{BTreeMap, BTreeSet};

pub fn summarize_omena_query_fast_facts(style_path: &str, style_source: &str) -> FastFactsV0 {
    let dialect = omena_parser_dialect_for_style_path(style_path);
    let facts = collect_omena_query_omena_parser_style_facts_raw(style_source, dialect);
    let custom_property_count = facts
        .variables
        .iter()
        .filter(|fact| {
            matches!(
                fact.kind,
                ParsedVariableFactKind::CustomPropertyDeclaration
                    | ParsedVariableFactKind::CustomPropertyReference
            )
        })
        .count();

    FastFactsV0 {
        schema_version: "0",
        product: "omena-query.fast-facts",
        tier: "fastFactsV0",
        style_path: style_path.to_string(),
        language: omena_parser_style_dialect_label(dialect),
        selector_count: facts.selectors.len(),
        custom_property_count,
        sass_symbol_count: facts.sass_symbols.len(),
        module_edge_count: facts.sass_module_edges.len(),
        parser_error_count: facts.error_count,
    }
}

pub fn summarize_omena_query_analyzed_graph(
    style_path: &str,
    style_source: &str,
) -> AnalyzedGraphV0 {
    let dialect = omena_parser_dialect_for_style_path(style_path);
    let facts = collect_omena_query_omena_parser_style_facts_raw(style_source, dialect);
    let fast_facts = summarize_omena_query_fast_facts(style_path, style_source);
    let edge_count = facts.css_module_composes_edges.len()
        + facts.css_module_value_import_edges.len()
        + facts.css_module_value_definition_edges.len()
        + facts.icss_import_edges.len()
        + facts.icss_export_edges.len()
        + facts.sass_module_edges.len()
        + facts.animations.len();

    AnalyzedGraphV0 {
        schema_version: "0",
        product: "omena-query.analyzed-graph",
        tier: "analyzedGraphV0",
        style_path: style_path.to_string(),
        node_count: facts.selectors.len() + facts.variables.len() + facts.sass_symbols.len(),
        edge_count,
        cycle_count: 0,
        graph_kinds: vec![
            "selectorFacts",
            "customPropertyFacts",
            "cssModulesFacts",
            "sassModuleFacts",
        ],
        fast_facts,
    }
}

pub fn summarize_omena_query_custom_property_annotations(
    style_path: &str,
    style_source: &str,
) -> OmenaQueryCustomPropertyAnnotationSummaryV0 {
    let dialect = omena_parser_dialect_for_style_path(style_path);
    let facts = collect_omena_query_omena_parser_style_facts_raw(style_source, dialect);
    let mut declarations_by_name = BTreeMap::<String, usize>::new();
    let mut references_by_name = BTreeMap::<String, usize>::new();

    for fact in facts.variables {
        match fact.kind {
            ParsedVariableFactKind::CustomPropertyDeclaration => {
                *declarations_by_name.entry(fact.name).or_default() += 1;
            }
            ParsedVariableFactKind::CustomPropertyReference => {
                *references_by_name.entry(fact.name).or_default() += 1;
            }
            _ => {}
        }
    }

    let names = declarations_by_name
        .keys()
        .chain(references_by_name.keys())
        .cloned()
        .collect::<BTreeSet<_>>();
    let annotations = names
        .into_iter()
        .map(|name| {
            let declaration_count = declarations_by_name.get(&name).copied().unwrap_or(0);
            let reference_count = references_by_name.get(&name).copied().unwrap_or(0);
            OmenaQueryCustomPropertyAnnotationV0 {
                name,
                declaration_count,
                reference_count,
                annotation_kind: match (declaration_count > 0, reference_count > 0) {
                    (true, true) => "declarationAndReference",
                    (true, false) => "declaration",
                    (false, true) => "reference",
                    (false, false) => "empty",
                },
                participates_in_fixed_point: declaration_count > 0 && reference_count > 0,
            }
        })
        .collect::<Vec<_>>();

    OmenaQueryCustomPropertyAnnotationSummaryV0 {
        schema_version: "0",
        product: "omena-query.custom-property-annotations",
        style_path: style_path.to_string(),
        annotation_count: annotations.len(),
        annotations,
    }
}
