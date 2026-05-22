use super::*;

pub fn is_omena_query_sass_symbol_candidate_kind(kind: &str) -> bool {
    omena_query_sass_symbol_kind_from_candidate_kind(kind).is_some()
}

pub fn is_omena_query_sass_symbol_reference_kind(kind: &str) -> bool {
    matches!(
        kind,
        "sassVariableReference"
            | "sassMixinInclude"
            | "sassFunctionCall"
            | "sassMixinReference"
            | "sassFunctionReference"
            | "sassSymbolReference"
    )
}

pub fn is_omena_query_sass_symbol_declaration_kind(kind: &str) -> bool {
    matches!(
        kind,
        "sassVariableDeclaration"
            | "sassMixinDeclaration"
            | "sassFunctionDeclaration"
            | "sassSymbolDeclaration"
    )
}

pub fn omena_query_sass_symbol_kind_from_candidate_kind(kind: &str) -> Option<&'static str> {
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

pub fn omena_query_sass_symbol_target_matches(
    candidate_kind: &str,
    candidate_name: &str,
    candidate_namespace: Option<&str>,
    target_kind: &str,
    target_name: &str,
    target_namespace: Option<&str>,
) -> bool {
    candidate_name == target_name
        && candidate_namespace == target_namespace
        && omena_query_sass_symbol_kind_from_candidate_kind(candidate_kind)
            == omena_query_sass_symbol_kind_from_candidate_kind(target_kind)
}

pub fn resolve_omena_query_sass_symbol_declarations(
    candidates: &[OmenaQueryStyleHoverCandidateV0],
    symbol_kind: &str,
    name: &str,
) -> Vec<OmenaQueryStyleHoverCandidateV0> {
    candidates
        .iter()
        .filter(|target| {
            is_omena_query_sass_symbol_declaration_kind(target.kind)
                && omena_query_sass_symbol_kind_from_candidate_kind(target.kind)
                    == Some(symbol_kind)
                && target.name == name
        })
        .cloned()
        .collect()
}

pub fn resolve_omena_query_sass_module_use_sources_for_candidate(
    sources: &OmenaQuerySassModuleSourcesV0,
    namespace: Option<&str>,
) -> Vec<String> {
    let mut selected = sources
        .module_use_edges
        .iter()
        .filter(|edge| {
            if let Some(namespace) = namespace {
                edge.namespace.as_deref() == Some(namespace)
            } else {
                edge.namespace_kind == "wildcard"
            }
        })
        .filter(|edge| !is_sass_builtin_module_source(edge.source.as_str()))
        .map(|edge| edge.source.clone())
        .collect::<Vec<_>>();
    selected.sort();
    selected.dedup();
    selected
}

pub fn resolve_omena_query_sass_forward_sources(
    sources: &OmenaQuerySassModuleSourcesV0,
) -> Vec<String> {
    let mut selected = sources
        .module_forward_sources
        .iter()
        .filter(|source| !is_sass_builtin_module_source(source.as_str()))
        .cloned()
        .collect::<Vec<_>>();
    selected.sort();
    selected.dedup();
    selected
}

pub fn summarize_omena_query_sass_module_sources(
    style_path: &str,
    style_source: &str,
) -> Option<OmenaQuerySassModuleSourcesV0> {
    let facts = collect_omena_query_omena_parser_style_facts_raw(
        style_source,
        omena_parser_dialect_for_style_path(style_path),
    );
    let mut module_use_edges = Vec::new();
    let mut module_forward_sources = BTreeSet::new();
    for edge in facts.sass_module_edges {
        match edge.kind {
            ParsedSassModuleEdgeFactKind::Use => {
                module_use_edges.push(OmenaQuerySassModuleUseEdgeV0 {
                    source: edge.source,
                    namespace_kind: edge.namespace_kind.unwrap_or("default"),
                    namespace: edge.namespace,
                });
            }
            ParsedSassModuleEdgeFactKind::Forward => {
                module_forward_sources.insert(edge.source);
            }
            ParsedSassModuleEdgeFactKind::Import => {
                module_use_edges.push(OmenaQuerySassModuleUseEdgeV0 {
                    source: edge.source,
                    namespace_kind: "wildcard",
                    namespace: None,
                });
            }
        }
    }
    Some(OmenaQuerySassModuleSourcesV0 {
        schema_version: "0",
        product: "omena-query.sass-module-sources",
        module_use_edges,
        module_forward_sources: module_forward_sources.into_iter().collect(),
    })
}
