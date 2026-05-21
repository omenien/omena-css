use super::*;
use omena_parser::{ParsedSassIncludeFact, ParsedSelectorFact, ParsedVariableFact};

mod cascade_position;
mod code_actions;
mod completion;
mod cross_file_summary;
mod diagnostics;
mod parser_facade;
mod source_refs;
mod stylesheet_evaluation;
mod substrate;
mod transform;

pub use cascade_position::*;
pub use code_actions::*;
pub use completion::*;
use cross_file_summary::summarize_omena_query_cross_file_summary;
pub use cross_file_summary::{
    summarize_omena_query_source_selector_reference_cross_file_summary,
    summarize_omena_query_workspace_cross_file_summary,
};
pub use diagnostics::*;
use parser_facade::{
    collect_omena_query_omena_parser_style_facts_raw, omena_parser_dialect_for_style_path,
    omena_parser_style_dialect_label, omena_query_sass_symbol_fact_kind_is_declaration,
    omena_query_sass_symbol_fact_kind_is_reference,
};
pub use parser_facade::{
    summarize_omena_query_omena_parser_css_modules_intermediate,
    summarize_omena_query_omena_parser_lex, summarize_omena_query_omena_parser_style_facts,
    summarize_omena_query_style_document,
};
pub use source_refs::*;
pub use substrate::*;
pub use transform::*;

mod cascade_checker;

pub fn summarize_omena_query_style_semantic_graph_from_source(
    style_path: &str,
    style_source: &str,
    input: &EngineInputV2,
) -> Option<StyleSemanticGraphSummaryV0> {
    summarize_omena_bridge_style_semantic_graph_from_source(style_path, style_source, input)
}

pub fn read_omena_query_style_context_index(
    style_path: &str,
    style_source: &str,
    input: &EngineInputV2,
) -> Option<OmenaQueryStyleContextIndexV0> {
    let graph =
        summarize_omena_query_style_semantic_graph_from_source(style_path, style_source, input)?;
    Some(OmenaQueryStyleContextIndexV0 {
        schema_version: "0",
        product: "omena-query.style-context-index",
        style_path: style_path.to_string(),
        language: graph.language,
        context_index_source: graph.semantic_facts.context_index.product,
        context_index: graph.semantic_facts.context_index,
    })
}

pub fn summarize_omena_query_style_hover_candidates(
    style_path: &str,
    style_source: &str,
) -> Option<OmenaQueryStyleHoverCandidatesV0> {
    let dialect = omena_parser_dialect_for_style_path(style_path);
    let facts = collect_omena_query_omena_parser_style_facts_raw(style_source, dialect);
    let mut seen = BTreeSet::new();
    let mut candidates = Vec::new();
    collect_style_selector_hover_candidates_from_omena_parser_facts(
        style_source,
        facts.selectors.as_slice(),
        &mut seen,
        &mut candidates,
    );
    collect_custom_property_hover_candidates_from_omena_parser_facts(
        style_source,
        facts.variables.as_slice(),
        &mut seen,
        &mut candidates,
    );
    collect_sass_symbol_hover_candidates_from_omena_parser_facts(
        style_source,
        facts.sass_symbols.as_slice(),
        &mut seen,
        &mut candidates,
    );
    collect_sass_partial_evaluator_selector_candidates_from_omena_parser_facts(
        style_source,
        facts.sass_includes.as_slice(),
        &mut seen,
        &mut candidates,
    );
    candidates.sort();
    Some(OmenaQueryStyleHoverCandidatesV0 {
        schema_version: "0",
        product: "omena-query.style-hover-candidates",
        language: omena_parser_style_dialect_label(dialect),
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

fn source_reference_text_selector_name(source: &str, span: ParserByteSpanV0) -> Option<String> {
    let text = source.get(span.start..span.end)?;
    if text.is_empty() {
        return None;
    }
    text.chars()
        .all(is_css_identifier_continue)
        .then(|| text.to_string())
}

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
    let style_fact_entries = collect_omena_query_style_fact_entries(style_sources.as_slice());
    let workspace_declarations = style_fact_entries
        .iter()
        .flat_map(|entry| {
            collect_omena_bridge_design_token_workspace_declarations_from_source(
                entry.style_path.as_str(),
                entry.style_source.as_str(),
            )
        })
        .collect::<Vec<_>>();
    let css_modules_resolution =
        summarize_css_modules_cross_file_resolution(&style_fact_entries, package_manifests);
    let sass_module_resolution =
        summarize_sass_module_cross_file_resolution(&style_fact_entries, package_manifests);
    let cross_file_summary = summarize_omena_query_cross_file_summary(
        &style_fact_entries,
        &css_modules_resolution,
        &sass_module_resolution,
    );
    let graphs = style_sources
        .into_iter()
        .map(
            |(style_path, style_source)| OmenaQueryStyleSemanticGraphBatchEntryV0 {
                style_path: style_path.to_string(),
                graph: {
                    let import_reachable_declarations =
                        filter_import_reachable_design_token_workspace_declarations(
                            style_path,
                            &style_fact_entries,
                            &workspace_declarations,
                            package_manifests,
                        );
                    summarize_omena_bridge_style_semantic_graph_from_source_with_scoped_workspace_declarations(
                        style_path,
                        style_source,
                        input,
                        &import_reachable_declarations,
                        DesignTokenExternalDeclarationCandidateScopeV0::CrossFileImportGraph,
                    )
                },
            },
        )
        .collect::<Vec<_>>();

    OmenaQueryStyleSemanticGraphBatchOutputV0 {
        schema_version: "0",
        product: "omena-semantic.style-semantic-graph-batch",
        cross_file_summary,
        css_modules_resolution,
        sass_module_resolution,
        graphs,
    }
}

struct OmenaQueryStyleFactEntry {
    style_path: String,
    style_source: String,
    facts: OmenaQueryOmenaParserStyleFactsV0,
}

fn collect_omena_query_style_fact_entries(
    style_sources: &[(&str, &str)],
) -> Vec<OmenaQueryStyleFactEntry> {
    style_sources
        .iter()
        .map(|(style_path, style_source)| OmenaQueryStyleFactEntry {
            style_path: (*style_path).to_string(),
            style_source: (*style_source).to_string(),
            facts: summarize_omena_query_omena_parser_style_facts(
                style_source,
                omena_parser_dialect_for_style_path(style_path),
            ),
        })
        .collect()
}

fn summarize_sass_module_cross_file_resolution(
    style_fact_entries: &[OmenaQueryStyleFactEntry],
    package_manifests: &[OmenaQueryStylePackageManifestV0],
) -> OmenaQuerySassModuleCrossFileResolutionV0 {
    let available_style_paths = style_fact_entries
        .iter()
        .map(|entry| entry.style_path.as_str())
        .collect::<BTreeSet<_>>();
    let resolver_package_manifests = package_manifests
        .iter()
        .map(|manifest| OmenaResolverStylePackageManifestV0 {
            package_json_path: manifest.package_json_path.clone(),
            package_json_source: manifest.package_json_source.clone(),
        })
        .collect::<Vec<_>>();
    let mut edges = Vec::new();

    for entry in style_fact_entries {
        for edge in &entry.facts.sass_module_edges {
            let resolution = summarize_omena_resolver_style_module_resolution(
                entry.style_path.as_str(),
                edge.source.as_str(),
                &available_style_paths,
                &resolver_package_manifests,
            );
            let status = if resolution.resolution_kind == "externalIgnored" {
                "external"
            } else if resolution.resolved_style_path.is_some() {
                "resolved"
            } else {
                "unresolved"
            };
            edges.push(OmenaQuerySassModuleEdgeResolutionV0 {
                from_style_path: entry.style_path.clone(),
                edge_kind: edge.kind,
                source: edge.source.clone(),
                namespace_kind: edge.namespace_kind,
                namespace: edge.namespace.clone(),
                forward_prefix: edge.forward_prefix.clone(),
                visibility_filter_kind: edge.visibility_filter_kind,
                visibility_filter_names: edge.visibility_filter_names.clone(),
                resolved_style_path: resolution.resolved_style_path,
                status,
                resolution_kind: resolution.resolution_kind,
                candidate_count: resolution.candidate_count,
            });
        }
    }

    edges.sort_by_key(|edge| {
        (
            edge.from_style_path.clone(),
            edge.edge_kind,
            edge.source.clone(),
        )
    });
    let resolved_module_edge_count = edges
        .iter()
        .filter(|edge| edge.status == "resolved")
        .count();
    let external_module_edge_count = edges
        .iter()
        .filter(|edge| edge.status == "external")
        .count();
    let unresolved_module_edge_count = edges
        .len()
        .saturating_sub(resolved_module_edge_count + external_module_edge_count);
    let (graph_closure_edges, cycles) = summarize_sass_module_graph_closure(&edges);
    let visibility_filter_count = edges
        .iter()
        .filter(|edge| edge.visibility_filter_kind.is_some())
        .count();

    OmenaQuerySassModuleCrossFileResolutionV0 {
        schema_version: "0",
        product: "omena-query.sass-module-cross-file-resolution",
        status: "moduleGraphClosureResolved",
        resolution_scope: "batchModuleGraph",
        style_count: style_fact_entries.len(),
        module_edge_count: edges.len(),
        resolved_module_edge_count,
        unresolved_module_edge_count,
        external_module_edge_count,
        edges,
        graph_closure_edge_count: graph_closure_edges.len(),
        cycle_count: cycles.len(),
        visibility_filter_count,
        graph_closure_edges,
        cycles,
        capabilities: OmenaQuerySassModuleCrossFileResolutionCapabilitiesV0 {
            omena_parser_module_edge_consumption_ready: true,
            resolver_backed_source_resolution_ready: true,
            package_manifest_resolution_ready: true,
            external_module_filtering_ready: true,
            graph_closure_ready: true,
            cycle_detection_ready: true,
            namespace_show_hide_filter_ready: true,
        },
        next_priorities: Vec::new(),
    }
}

fn summarize_sass_module_graph_closure(
    edges: &[OmenaQuerySassModuleEdgeResolutionV0],
) -> (
    Vec<OmenaQuerySassModuleGraphClosureEdgeV0>,
    Vec<OmenaQuerySassModuleCycleV0>,
) {
    let mut adjacency: BTreeMap<&str, Vec<&OmenaQuerySassModuleEdgeResolutionV0>> = BTreeMap::new();
    for edge in edges {
        if edge.status != "resolved" {
            continue;
        }
        if edge.resolved_style_path.is_none() {
            continue;
        }
        adjacency
            .entry(edge.from_style_path.as_str())
            .or_default()
            .push(edge);
    }
    for outgoing in adjacency.values_mut() {
        outgoing.sort_by_key(|edge| {
            (
                edge.resolved_style_path.clone().unwrap_or_default(),
                edge.edge_kind,
                edge.source.clone(),
            )
        });
    }

    let mut collector = SassModuleGraphClosureCollector::new(&adjacency);
    for origin in adjacency.keys() {
        let mut path = vec![(*origin).to_string()];
        collector.collect(origin, origin, &mut path);
    }

    let (mut closure_edges, mut cycles) = collector.finish();
    closure_edges.sort_by_key(|edge| {
        (
            edge.from_style_path.clone(),
            edge.depth,
            edge.target_style_path.clone(),
            edge.edge_kind,
            edge.path.clone(),
        )
    });
    cycles.sort_by_key(|cycle| cycle.path.clone());
    (closure_edges, cycles)
}

struct SassModuleGraphClosureCollector<'a> {
    adjacency: &'a BTreeMap<&'a str, Vec<&'a OmenaQuerySassModuleEdgeResolutionV0>>,
    seen_edges: BTreeSet<(String, String, Vec<String>)>,
    closure_edges: Vec<OmenaQuerySassModuleGraphClosureEdgeV0>,
    seen_cycles: BTreeSet<Vec<String>>,
    cycles: Vec<OmenaQuerySassModuleCycleV0>,
}

impl<'a> SassModuleGraphClosureCollector<'a> {
    fn new(
        adjacency: &'a BTreeMap<&'a str, Vec<&'a OmenaQuerySassModuleEdgeResolutionV0>>,
    ) -> Self {
        Self {
            adjacency,
            seen_edges: BTreeSet::new(),
            closure_edges: Vec::new(),
            seen_cycles: BTreeSet::new(),
            cycles: Vec::new(),
        }
    }

    fn collect(&mut self, origin: &str, current: &str, path: &mut Vec<String>) {
        let Some(outgoing) = self.adjacency.get(current) else {
            return;
        };
        for edge in outgoing {
            let Some(target) = edge.resolved_style_path.as_deref() else {
                continue;
            };
            let mut next_path = path.clone();
            next_path.push(target.to_string());
            if path.iter().any(|segment| segment == target) {
                if self.seen_cycles.insert(next_path.clone()) {
                    self.cycles
                        .push(OmenaQuerySassModuleCycleV0 { path: next_path });
                }
                continue;
            }
            if self
                .seen_edges
                .insert((origin.to_string(), target.to_string(), next_path.clone()))
            {
                self.closure_edges
                    .push(OmenaQuerySassModuleGraphClosureEdgeV0 {
                        from_style_path: origin.to_string(),
                        target_style_path: target.to_string(),
                        edge_kind: edge.edge_kind,
                        depth: next_path.len().saturating_sub(1),
                        path: next_path.clone(),
                        namespace_kind: edge.namespace_kind,
                        namespace: edge.namespace.clone(),
                        forward_prefix: edge.forward_prefix.clone(),
                        visibility_filter_kind: edge.visibility_filter_kind,
                        visibility_filter_names: edge.visibility_filter_names.clone(),
                    });
            }
            path.push(target.to_string());
            self.collect(origin, target, path);
            path.pop();
        }
    }

    fn finish(
        self,
    ) -> (
        Vec<OmenaQuerySassModuleGraphClosureEdgeV0>,
        Vec<OmenaQuerySassModuleCycleV0>,
    ) {
        (self.closure_edges, self.cycles)
    }
}

fn summarize_css_modules_cross_file_resolution(
    style_fact_entries: &[OmenaQueryStyleFactEntry],
    package_manifests: &[OmenaQueryStylePackageManifestV0],
) -> OmenaQueryCssModulesCrossFileResolutionV0 {
    let available_style_paths = style_fact_entries
        .iter()
        .map(|entry| entry.style_path.as_str())
        .collect::<BTreeSet<_>>();
    let facts_by_path = style_fact_entries
        .iter()
        .map(|entry| (entry.style_path.as_str(), entry.facts.clone()))
        .collect::<BTreeMap<_, _>>();
    let mut edges = Vec::new();

    for entry in style_fact_entries {
        let style_path = entry.style_path.as_str();
        let Some(facts) = facts_by_path.get(style_path) else {
            continue;
        };
        let reachable = collect_import_reachable_style_path_metadata(
            style_path,
            style_fact_entries,
            package_manifests,
        );
        let context = CssModulesResolutionBatchContext {
            available_style_paths: &available_style_paths,
            facts_by_path: &facts_by_path,
            reachable: &reachable,
            package_manifests,
        };

        for edge in &facts.css_module_composes_edges {
            let Some(source) = edge.import_source.as_deref() else {
                continue;
            };
            edges.push(resolve_css_modules_import_edge(
                style_path,
                "composes",
                source,
                edge.target_names.as_slice(),
                &context,
                |target| target.class_selector_names.as_slice(),
            ));
        }

        for edge in &facts.css_module_value_import_edges {
            edges.push(resolve_css_modules_import_edge(
                style_path,
                "value",
                edge.import_source.as_str(),
                std::slice::from_ref(&edge.remote_name),
                &context,
                |target| target.css_module_value_definition_names.as_slice(),
            ));
        }

        for edge in &facts.icss_import_edges {
            edges.push(resolve_css_modules_import_edge(
                style_path,
                "icss",
                edge.import_source.as_str(),
                std::slice::from_ref(&edge.remote_name),
                &context,
                |target| target.icss_export_names.as_slice(),
            ));
        }
    }

    edges.sort_by_key(|edge| {
        (
            edge.from_style_path.clone(),
            edge.import_kind,
            edge.source.clone(),
        )
    });
    let (composes_closure_edges, cycles) = summarize_css_modules_composes_closure(
        &facts_by_path,
        &available_style_paths,
        package_manifests,
    );
    let (value_closure_edges, value_cycles) = summarize_css_modules_value_closure(
        &facts_by_path,
        &available_style_paths,
        package_manifests,
    );
    let (icss_closure_edges, icss_cycles) = summarize_css_modules_icss_closure(
        &facts_by_path,
        &available_style_paths,
        package_manifests,
    );
    let composes_cycle_count = cycles.len();
    let value_cycle_count = value_cycles.len();
    let icss_cycle_count = icss_cycles.len();
    let mut cycles = cycles;
    cycles.extend(value_cycles);
    cycles.extend(icss_cycles);
    cycles.sort_by_key(|cycle| (cycle.kind, cycle.path.clone()));
    let resolved_import_edge_count = edges
        .iter()
        .filter(|edge| edge.resolved_style_path.is_some())
        .count();
    let matched_name_count = edges
        .iter()
        .map(|edge| edge.matched_names.len())
        .sum::<usize>();

    OmenaQueryCssModulesCrossFileResolutionV0 {
        schema_version: "0",
        product: "omena-query.css-modules-cross-file-resolution",
        status: "icssExportImportClosureSeed",
        resolution_scope: "batchImportGraph",
        style_count: style_fact_entries.len(),
        import_edge_count: edges.len(),
        resolved_import_edge_count,
        unresolved_import_edge_count: edges.len() - resolved_import_edge_count,
        matched_name_count,
        edges,
        composes_closure_edge_count: composes_closure_edges.len(),
        value_closure_edge_count: value_closure_edges.len(),
        icss_closure_edge_count: icss_closure_edges.len(),
        composes_cycle_count,
        value_cycle_count,
        icss_cycle_count,
        composes_closure_edges,
        value_closure_edges,
        icss_closure_edges,
        cycles,
        capabilities: OmenaQueryCssModulesCrossFileResolutionCapabilitiesV0 {
            import_source_resolution_ready: true,
            composes_name_match_ready: true,
            value_name_match_ready: true,
            icss_name_match_ready: true,
            transitive_closure_ready: true,
            value_graph_closure_ready: true,
            icss_export_import_closure_ready: true,
            cycle_detection_ready: true,
        },
        next_priorities: vec![],
    }
}

struct CssModulesResolutionBatchContext<'a> {
    available_style_paths: &'a BTreeSet<&'a str>,
    facts_by_path: &'a BTreeMap<&'a str, OmenaQueryOmenaParserStyleFactsV0>,
    reachable: &'a BTreeMap<String, ImportReachability>,
    package_manifests: &'a [OmenaQueryStylePackageManifestV0],
}

fn resolve_css_modules_import_edge(
    from_style_path: &str,
    import_kind: &'static str,
    source: &str,
    imported_names: &[String],
    context: &CssModulesResolutionBatchContext<'_>,
    exported_names_for_kind: fn(&OmenaQueryOmenaParserStyleFactsV0) -> &[String],
) -> OmenaQueryCssModulesImportEdgeResolutionV0 {
    let resolved_style_path = resolve_style_module_source(
        from_style_path,
        source,
        context.available_style_paths,
        context.package_manifests,
    );
    let reachability = resolved_style_path
        .as_ref()
        .and_then(|style_path| context.reachable.get(style_path));
    let exported_names = resolved_style_path
        .as_deref()
        .and_then(|style_path| context.facts_by_path.get(style_path))
        .map(exported_names_for_kind)
        .map(|names| names.to_vec())
        .unwrap_or_default();
    let imported_names = sorted_unique_strings(imported_names);
    let matched_names =
        sorted_name_intersection(imported_names.as_slice(), exported_names.as_slice());
    let status = if resolved_style_path.is_none() {
        "unresolvedSource"
    } else if imported_names.is_empty() {
        "resolvedSource"
    } else if matched_names.is_empty() {
        "resolvedSourceNoNameMatch"
    } else {
        "resolved"
    };

    OmenaQueryCssModulesImportEdgeResolutionV0 {
        from_style_path: from_style_path.to_string(),
        import_kind,
        source: source.to_string(),
        resolved_style_path,
        status,
        import_graph_distance: reachability.map(|reachability| reachability.distance),
        import_graph_order: reachability.map(|reachability| reachability.order),
        imported_names,
        exported_names,
        matched_names,
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct CssModulesComposesNode {
    style_path: String,
    selector_name: String,
}

fn summarize_css_modules_composes_closure(
    facts_by_path: &BTreeMap<&str, OmenaQueryOmenaParserStyleFactsV0>,
    available_style_paths: &BTreeSet<&str>,
    package_manifests: &[OmenaQueryStylePackageManifestV0],
) -> (
    Vec<OmenaQueryCssModulesComposesClosureEdgeV0>,
    Vec<OmenaQueryCssModulesCycleV0>,
) {
    let graph = collect_css_modules_composes_adjacency(
        facts_by_path,
        available_style_paths,
        package_manifests,
    );
    let mut closure_edges = Vec::new();
    let mut cycles = Vec::new();
    let mut seen_cycles = BTreeSet::new();

    for start in graph.keys() {
        let mut visited = BTreeSet::new();
        let mut pending = VecDeque::from([(start.clone(), vec![start.clone()])]);

        while let Some((current, path)) = pending.pop_front() {
            let Some(targets) = graph.get(&current) else {
                continue;
            };
            for target in targets {
                if let Some(cycle_start) = path.iter().position(|node| node == target) {
                    let mut cycle_path = path[cycle_start..].to_vec();
                    cycle_path.push(target.clone());
                    let cycle_labels = canonical_directed_cycle_labels(&cycle_path);
                    if !cycle_labels.is_empty() && seen_cycles.insert(cycle_labels.clone()) {
                        cycles.push(OmenaQueryCssModulesCycleV0 {
                            kind: "composes",
                            path: cycle_labels,
                        });
                    }
                    continue;
                }

                if !visited.insert(target.clone()) {
                    continue;
                }

                let mut edge_path = path.clone();
                edge_path.push(target.clone());
                closure_edges.push(OmenaQueryCssModulesComposesClosureEdgeV0 {
                    from_style_path: start.style_path.clone(),
                    owner_selector_name: start.selector_name.clone(),
                    target_style_path: target.style_path.clone(),
                    target_selector_name: target.selector_name.clone(),
                    depth: edge_path.len().saturating_sub(1),
                    path: edge_path
                        .iter()
                        .map(css_modules_composes_node_label)
                        .collect(),
                });
                pending.push_back((target.clone(), edge_path));
            }
        }
    }

    closure_edges.sort_by_key(|edge| {
        (
            edge.from_style_path.clone(),
            edge.owner_selector_name.clone(),
            edge.depth,
            edge.target_style_path.clone(),
            edge.target_selector_name.clone(),
        )
    });
    cycles.sort_by_key(|cycle| cycle.path.clone());
    (closure_edges, cycles)
}

fn collect_css_modules_composes_adjacency(
    facts_by_path: &BTreeMap<&str, OmenaQueryOmenaParserStyleFactsV0>,
    available_style_paths: &BTreeSet<&str>,
    package_manifests: &[OmenaQueryStylePackageManifestV0],
) -> BTreeMap<CssModulesComposesNode, BTreeSet<CssModulesComposesNode>> {
    let mut graph = BTreeMap::new();
    for (style_path, facts) in facts_by_path {
        let class_names = facts
            .class_selector_names
            .iter()
            .map(String::as_str)
            .collect::<BTreeSet<_>>();
        for edge in &facts.css_module_composes_edges {
            if edge.kind == "global" {
                continue;
            }
            let target_style_path = if edge.kind == "external" {
                edge.import_source.as_deref().and_then(|source| {
                    resolve_style_module_source(
                        style_path,
                        source,
                        available_style_paths,
                        package_manifests,
                    )
                })
            } else {
                Some((*style_path).to_string())
            };
            let Some(target_style_path) = target_style_path else {
                continue;
            };
            let target_class_names = if target_style_path == *style_path {
                class_names.clone()
            } else {
                facts_by_path
                    .get(target_style_path.as_str())
                    .map(|facts| {
                        facts
                            .class_selector_names
                            .iter()
                            .map(String::as_str)
                            .collect::<BTreeSet<_>>()
                    })
                    .unwrap_or_default()
            };
            for owner_selector_name in &edge.owner_selector_names {
                if !class_names.contains(owner_selector_name.as_str()) {
                    continue;
                }
                let owner = CssModulesComposesNode {
                    style_path: (*style_path).to_string(),
                    selector_name: owner_selector_name.clone(),
                };
                for target_selector_name in &edge.target_names {
                    if !target_class_names.contains(target_selector_name.as_str()) {
                        continue;
                    }
                    graph
                        .entry(owner.clone())
                        .or_insert_with(BTreeSet::new)
                        .insert(CssModulesComposesNode {
                            style_path: target_style_path.clone(),
                            selector_name: target_selector_name.clone(),
                        });
                }
            }
        }
    }
    graph
}

fn css_modules_composes_node_label(node: &CssModulesComposesNode) -> String {
    format!("{}#{}", node.style_path, node.selector_name)
}

fn canonical_directed_cycle_labels(path: &[CssModulesComposesNode]) -> Vec<String> {
    let mut labels = path
        .iter()
        .map(css_modules_composes_node_label)
        .collect::<Vec<_>>();
    if labels.len() > 1 && labels.first() == labels.last() {
        labels.pop();
    }
    if labels.is_empty() {
        return Vec::new();
    }

    let mut best = labels.clone();
    for offset in 1..labels.len() {
        let mut rotated = labels[offset..].to_vec();
        rotated.extend_from_slice(&labels[..offset]);
        if rotated < best {
            best = rotated;
        }
    }
    best.push(best[0].clone());
    best
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct CssModulesValueNode {
    style_path: String,
    value_name: String,
}

fn summarize_css_modules_value_closure(
    facts_by_path: &BTreeMap<&str, OmenaQueryOmenaParserStyleFactsV0>,
    available_style_paths: &BTreeSet<&str>,
    package_manifests: &[OmenaQueryStylePackageManifestV0],
) -> (
    Vec<OmenaQueryCssModulesValueClosureEdgeV0>,
    Vec<OmenaQueryCssModulesCycleV0>,
) {
    let graph = collect_css_modules_value_adjacency(
        facts_by_path,
        available_style_paths,
        package_manifests,
    );
    let mut closure_edges = Vec::new();
    let mut cycles = Vec::new();
    let mut seen_cycles = BTreeSet::new();

    for start in graph.keys() {
        let mut visited = BTreeSet::new();
        let mut pending = VecDeque::from([(start.clone(), vec![start.clone()])]);

        while let Some((current, path)) = pending.pop_front() {
            let Some(targets) = graph.get(&current) else {
                continue;
            };
            for target in targets {
                if let Some(cycle_start) = path.iter().position(|node| node == target) {
                    let mut cycle_path = path[cycle_start..].to_vec();
                    cycle_path.push(target.clone());
                    let cycle_labels = canonical_directed_value_cycle_labels(&cycle_path);
                    if !cycle_labels.is_empty() && seen_cycles.insert(cycle_labels.clone()) {
                        cycles.push(OmenaQueryCssModulesCycleV0 {
                            kind: "value",
                            path: cycle_labels,
                        });
                    }
                    continue;
                }

                if !visited.insert(target.clone()) {
                    continue;
                }

                let mut edge_path = path.clone();
                edge_path.push(target.clone());
                closure_edges.push(OmenaQueryCssModulesValueClosureEdgeV0 {
                    from_style_path: start.style_path.clone(),
                    value_name: start.value_name.clone(),
                    target_style_path: target.style_path.clone(),
                    target_value_name: target.value_name.clone(),
                    depth: edge_path.len().saturating_sub(1),
                    path: edge_path.iter().map(css_modules_value_node_label).collect(),
                });
                pending.push_back((target.clone(), edge_path));
            }
        }
    }

    closure_edges.sort_by_key(|edge| {
        (
            edge.from_style_path.clone(),
            edge.value_name.clone(),
            edge.depth,
            edge.target_style_path.clone(),
            edge.target_value_name.clone(),
        )
    });
    cycles.sort_by_key(|cycle| cycle.path.clone());
    (closure_edges, cycles)
}

fn collect_css_modules_value_adjacency(
    facts_by_path: &BTreeMap<&str, OmenaQueryOmenaParserStyleFactsV0>,
    available_style_paths: &BTreeSet<&str>,
    package_manifests: &[OmenaQueryStylePackageManifestV0],
) -> BTreeMap<CssModulesValueNode, BTreeSet<CssModulesValueNode>> {
    let mut graph = BTreeMap::new();
    for (style_path, facts) in facts_by_path {
        let local_value_names = facts
            .css_module_value_definition_names
            .iter()
            .chain(
                facts
                    .css_module_value_import_edges
                    .iter()
                    .map(|edge| &edge.local_name),
            )
            .map(String::as_str)
            .collect::<BTreeSet<_>>();
        for edge in &facts.css_module_value_definition_edges {
            if !local_value_names.contains(edge.definition_name.as_str()) {
                continue;
            }
            let owner = CssModulesValueNode {
                style_path: (*style_path).to_string(),
                value_name: edge.definition_name.clone(),
            };
            for reference_name in &edge.reference_names {
                if !local_value_names.contains(reference_name.as_str()) {
                    continue;
                }
                graph
                    .entry(owner.clone())
                    .or_insert_with(BTreeSet::new)
                    .insert(CssModulesValueNode {
                        style_path: (*style_path).to_string(),
                        value_name: reference_name.clone(),
                    });
            }
        }

        for edge in &facts.css_module_value_import_edges {
            let Some(target_style_path) = resolve_style_module_source(
                style_path,
                edge.import_source.as_str(),
                available_style_paths,
                package_manifests,
            ) else {
                continue;
            };
            let Some(target_facts) = facts_by_path.get(target_style_path.as_str()) else {
                continue;
            };
            if !target_facts
                .css_module_value_definition_names
                .iter()
                .any(|name| name == &edge.remote_name)
            {
                continue;
            }
            graph
                .entry(CssModulesValueNode {
                    style_path: (*style_path).to_string(),
                    value_name: edge.local_name.clone(),
                })
                .or_insert_with(BTreeSet::new)
                .insert(CssModulesValueNode {
                    style_path: target_style_path,
                    value_name: edge.remote_name.clone(),
                });
        }
    }
    graph
}

fn css_modules_value_node_label(node: &CssModulesValueNode) -> String {
    format!("{}#{}", node.style_path, node.value_name)
}

fn canonical_directed_value_cycle_labels(path: &[CssModulesValueNode]) -> Vec<String> {
    let mut labels = path
        .iter()
        .map(css_modules_value_node_label)
        .collect::<Vec<_>>();
    if labels.len() > 1 && labels.first() == labels.last() {
        labels.pop();
    }
    if labels.is_empty() {
        return Vec::new();
    }

    let mut best = labels.clone();
    for offset in 1..labels.len() {
        let mut rotated = labels[offset..].to_vec();
        rotated.extend_from_slice(&labels[..offset]);
        if rotated < best {
            best = rotated;
        }
    }
    best.push(best[0].clone());
    best
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct CssModulesIcssNode {
    style_path: String,
    name: String,
}

fn summarize_css_modules_icss_closure(
    facts_by_path: &BTreeMap<&str, OmenaQueryOmenaParserStyleFactsV0>,
    available_style_paths: &BTreeSet<&str>,
    package_manifests: &[OmenaQueryStylePackageManifestV0],
) -> (
    Vec<OmenaQueryCssModulesIcssClosureEdgeV0>,
    Vec<OmenaQueryCssModulesCycleV0>,
) {
    let graph =
        collect_css_modules_icss_adjacency(facts_by_path, available_style_paths, package_manifests);
    let mut closure_edges = Vec::new();
    let mut cycles = Vec::new();
    let mut seen_cycles = BTreeSet::new();

    for start in graph.keys() {
        let mut visited = BTreeSet::new();
        let mut pending = VecDeque::from([(start.clone(), vec![start.clone()])]);

        while let Some((current, path)) = pending.pop_front() {
            let Some(targets) = graph.get(&current) else {
                continue;
            };
            for target in targets {
                if let Some(cycle_start) = path.iter().position(|node| node == target) {
                    let mut cycle_path = path[cycle_start..].to_vec();
                    cycle_path.push(target.clone());
                    let cycle_labels = canonical_directed_icss_cycle_labels(&cycle_path);
                    if !cycle_labels.is_empty() && seen_cycles.insert(cycle_labels.clone()) {
                        cycles.push(OmenaQueryCssModulesCycleV0 {
                            kind: "icss",
                            path: cycle_labels,
                        });
                    }
                    continue;
                }

                if !visited.insert(target.clone()) {
                    continue;
                }

                let mut edge_path = path.clone();
                edge_path.push(target.clone());
                closure_edges.push(OmenaQueryCssModulesIcssClosureEdgeV0 {
                    from_style_path: start.style_path.clone(),
                    name: start.name.clone(),
                    target_style_path: target.style_path.clone(),
                    target_name: target.name.clone(),
                    depth: edge_path.len().saturating_sub(1),
                    path: edge_path.iter().map(css_modules_icss_node_label).collect(),
                });
                pending.push_back((target.clone(), edge_path));
            }
        }
    }

    closure_edges.sort_by_key(|edge| {
        (
            edge.from_style_path.clone(),
            edge.name.clone(),
            edge.depth,
            edge.target_style_path.clone(),
            edge.target_name.clone(),
        )
    });
    cycles.sort_by_key(|cycle| cycle.path.clone());
    (closure_edges, cycles)
}

fn collect_css_modules_icss_adjacency(
    facts_by_path: &BTreeMap<&str, OmenaQueryOmenaParserStyleFactsV0>,
    available_style_paths: &BTreeSet<&str>,
    package_manifests: &[OmenaQueryStylePackageManifestV0],
) -> BTreeMap<CssModulesIcssNode, BTreeSet<CssModulesIcssNode>> {
    let mut graph = BTreeMap::new();
    for (style_path, facts) in facts_by_path {
        let local_names = facts
            .icss_export_names
            .iter()
            .chain(facts.icss_import_edges.iter().map(|edge| &edge.local_name))
            .map(String::as_str)
            .collect::<BTreeSet<_>>();
        for edge in &facts.icss_export_edges {
            if !local_names.contains(edge.export_name.as_str()) {
                continue;
            }
            let owner = CssModulesIcssNode {
                style_path: (*style_path).to_string(),
                name: edge.export_name.clone(),
            };
            for reference_name in &edge.reference_names {
                if !local_names.contains(reference_name.as_str()) {
                    continue;
                }
                graph
                    .entry(owner.clone())
                    .or_insert_with(BTreeSet::new)
                    .insert(CssModulesIcssNode {
                        style_path: (*style_path).to_string(),
                        name: reference_name.clone(),
                    });
            }
        }

        for edge in &facts.icss_import_edges {
            let Some(target_style_path) = resolve_style_module_source(
                style_path,
                edge.import_source.as_str(),
                available_style_paths,
                package_manifests,
            ) else {
                continue;
            };
            let Some(target_facts) = facts_by_path.get(target_style_path.as_str()) else {
                continue;
            };
            if !target_facts
                .icss_export_names
                .iter()
                .any(|name| name == &edge.remote_name)
            {
                continue;
            }
            graph
                .entry(CssModulesIcssNode {
                    style_path: (*style_path).to_string(),
                    name: edge.local_name.clone(),
                })
                .or_insert_with(BTreeSet::new)
                .insert(CssModulesIcssNode {
                    style_path: target_style_path,
                    name: edge.remote_name.clone(),
                });
        }
    }
    graph
}

fn css_modules_icss_node_label(node: &CssModulesIcssNode) -> String {
    format!("{}#{}", node.style_path, node.name)
}

fn canonical_directed_icss_cycle_labels(path: &[CssModulesIcssNode]) -> Vec<String> {
    let mut labels = path
        .iter()
        .map(css_modules_icss_node_label)
        .collect::<Vec<_>>();
    if labels.len() > 1 && labels.first() == labels.last() {
        labels.pop();
    }
    if labels.is_empty() {
        return Vec::new();
    }

    let mut best = labels.clone();
    for offset in 1..labels.len() {
        let mut rotated = labels[offset..].to_vec();
        rotated.extend_from_slice(&labels[..offset]);
        if rotated < best {
            best = rotated;
        }
    }
    best.push(best[0].clone());
    best
}

fn filter_import_reachable_design_token_workspace_declarations(
    target_style_path: &str,
    style_fact_entries: &[OmenaQueryStyleFactEntry],
    workspace_declarations: &[DesignTokenWorkspaceDeclarationFactV0],
    package_manifests: &[OmenaQueryStylePackageManifestV0],
) -> Vec<DesignTokenWorkspaceDeclarationFactV0> {
    let reachable_style_paths = collect_import_reachable_style_path_metadata(
        target_style_path,
        style_fact_entries,
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
    style_fact_entries: &[OmenaQueryStyleFactEntry],
    package_manifests: &[OmenaQueryStylePackageManifestV0],
) -> BTreeMap<String, ImportReachability> {
    let mut reachable_style_paths = BTreeMap::new();
    let available_style_paths = style_fact_entries
        .iter()
        .map(|entry| entry.style_path.as_str())
        .collect::<BTreeSet<_>>();
    let mut pending_style_paths = collect_import_reachable_direct_style_paths(
        target_style_path,
        style_fact_entries,
        &available_style_paths,
        package_manifests,
    )
    .into_iter()
    .map(|style_path| (style_path, 1usize))
    .collect::<VecDeque<_>>();
    let facts_by_path = style_fact_entries
        .iter()
        .map(|entry| (entry.style_path.as_str(), &entry.facts))
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

        let Some(facts) = facts_by_path.get(style_path.as_str()) else {
            continue;
        };
        for source in collect_sass_module_sources_from_facts(facts) {
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
    style_fact_entries: &[OmenaQueryStyleFactEntry],
    available_style_paths: &BTreeSet<&str>,
    package_manifests: &[OmenaQueryStylePackageManifestV0],
) -> Vec<String> {
    let Some(target_facts) = style_fact_entries
        .iter()
        .find(|entry| entry.style_path == target_style_path)
        .map(|entry| &entry.facts)
    else {
        return Vec::new();
    };
    collect_sass_module_sources_from_facts(target_facts)
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

fn collect_sass_module_sources_from_facts(
    facts: &OmenaQueryOmenaParserStyleFactsV0,
) -> Vec<String> {
    let mut sources = facts
        .sass_module_edges
        .iter()
        .map(|edge| edge.source.clone())
        .collect::<Vec<_>>();
    sources.sort();
    sources.dedup();
    sources
}

fn resolve_style_module_source(
    from_style_path: &str,
    source: &str,
    available_style_paths: &BTreeSet<&str>,
    package_manifests: &[OmenaQueryStylePackageManifestV0],
) -> Option<String> {
    let resolver_package_manifests = package_manifests
        .iter()
        .map(|manifest| OmenaResolverStylePackageManifestV0 {
            package_json_path: manifest.package_json_path.clone(),
            package_json_source: manifest.package_json_source.clone(),
        })
        .collect::<Vec<_>>();
    resolve_omena_resolver_style_module_source(
        from_style_path,
        source,
        available_style_paths,
        &resolver_package_manifests,
    )
}

fn collect_style_selector_hover_candidates_from_omena_parser_facts(
    source: &str,
    definition_facts: &[ParsedSelectorFact],
    seen: &mut BTreeSet<(usize, usize, String)>,
    candidates: &mut Vec<OmenaQueryStyleHoverCandidateV0>,
) {
    for fact in definition_facts {
        if fact.kind != ParsedSelectorFactKind::Class {
            continue;
        }
        let start: u32 = fact.range.start().into();
        let end: u32 = fact.range.end().into();
        let byte_span = ParserByteSpanV0 {
            start: start as usize,
            end: end as usize,
        };
        if seen.insert((byte_span.start, byte_span.end, fact.name.clone())) {
            candidates.push(OmenaQueryStyleHoverCandidateV0 {
                kind: "selector",
                name: fact.name.clone(),
                range: parser_range_for_byte_span(source, byte_span),
                source: "omenaParserSelectorFacts",
                namespace: None,
            });
        }
    }
}

fn collect_custom_property_hover_candidates_from_omena_parser_facts(
    source: &str,
    variable_facts: &[ParsedVariableFact],
    seen: &mut BTreeSet<(usize, usize, String)>,
    candidates: &mut Vec<OmenaQueryStyleHoverCandidateV0>,
) {
    for fact in variable_facts {
        let kind = match fact.kind {
            ParsedVariableFactKind::CustomPropertyDeclaration => "customPropertyDeclaration",
            ParsedVariableFactKind::CustomPropertyReference => "customPropertyReference",
            _ => continue,
        };
        let start: u32 = fact.range.start().into();
        let end: u32 = fact.range.end().into();
        let byte_span = ParserByteSpanV0 {
            start: start as usize,
            end: end as usize,
        };
        if seen.insert((byte_span.start, byte_span.end, fact.name.clone())) {
            candidates.push(OmenaQueryStyleHoverCandidateV0 {
                kind,
                name: fact.name.clone(),
                range: parser_range_for_byte_span(source, byte_span),
                source: "omenaParserVariableFacts",
                namespace: None,
            });
        }
    }
}

fn collect_sass_symbol_hover_candidates_from_omena_parser_facts(
    source: &str,
    symbol_facts: &[omena_parser::ParsedSassSymbolFact],
    seen: &mut BTreeSet<(usize, usize, String)>,
    candidates: &mut Vec<OmenaQueryStyleHoverCandidateV0>,
) {
    for fact in symbol_facts {
        let kind = match fact.kind {
            ParsedSassSymbolFactKind::VariableDeclaration
            | ParsedSassSymbolFactKind::MixinDeclaration
            | ParsedSassSymbolFactKind::FunctionDeclaration => {
                sass_symbol_declaration_candidate_kind(fact.symbol_kind)
            }
            ParsedSassSymbolFactKind::VariableReference
            | ParsedSassSymbolFactKind::MixinInclude
            | ParsedSassSymbolFactKind::FunctionCall => {
                sass_symbol_reference_candidate_kind(fact.symbol_kind, fact.role)
            }
        };
        let start: u32 = fact.range.start().into();
        let end: u32 = fact.range.end().into();
        let byte_span = ParserByteSpanV0 {
            start: start as usize,
            end: end as usize,
        };
        if seen.insert((
            byte_span.start,
            byte_span.end,
            format!(
                "{}:{}:{}",
                fact.symbol_kind,
                fact.namespace.as_deref().unwrap_or_default(),
                fact.name
            ),
        )) {
            candidates.push(OmenaQueryStyleHoverCandidateV0 {
                kind,
                name: fact.name.clone(),
                range: parser_range_for_byte_span(source, byte_span),
                source: "omenaParserSassSymbolFacts",
                namespace: fact.namespace.clone(),
            });
        }
    }
}

fn collect_sass_partial_evaluator_selector_candidates_from_omena_parser_facts(
    source: &str,
    includes: &[ParsedSassIncludeFact],
    seen: &mut BTreeSet<(usize, usize, String)>,
    candidates: &mut Vec<OmenaQueryStyleHoverCandidateV0>,
) {
    for include in includes {
        let start: u32 = include.range.start().into();
        let end: u32 = include.range.end().into();
        let range_span = ParserByteSpanV0 {
            start: start as usize,
            end: end as usize,
        };
        for selector_name in infer_sass_include_generated_selector_names(&include.params) {
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

fn style_completion_context_at_position(
    source: &str,
    position: ParserPositionV0,
) -> Option<(&'static str, Option<String>)> {
    let cursor = byte_offset_for_parser_position(source, position)?;
    let line_start = byte_offset_for_parser_position(
        source,
        ParserPositionV0 {
            line: position.line,
            character: 0,
        },
    )?;
    let line_prefix = source.get(line_start..cursor)?;
    if let Some(var_start) = line_prefix.rfind("var(") {
        let var_prefix = &line_prefix[var_start + "var(".len()..];
        if !var_prefix.contains(')') {
            let prefix = var_prefix
                .rsplit(|ch: char| ch == ',' || ch.is_ascii_whitespace())
                .next()
                .unwrap_or_default();
            let prefix = (!prefix.is_empty()).then(|| prefix.to_string());
            return Some(("styleCustomPropertyReference", prefix));
        }
    }

    Some(("styleDocument", None))
}

fn trim_hover_snippet(snippet: &str) -> String {
    const MAX_SNIPPET_LEN: usize = 1200;
    if snippet.len() <= MAX_SNIPPET_LEN {
        return snippet.to_string();
    }
    let end = char_boundary_floor(snippet, MAX_SNIPPET_LEN);
    format!("{}...", snippet[..end].trim_end())
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

fn push_omena_query_ready_surface(ready_surfaces: &mut Vec<&'static str>, surface: &'static str) {
    if !ready_surfaces.contains(&surface) {
        ready_surfaces.push(surface);
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

fn is_sass_builtin_module_source(source: &str) -> bool {
    source.starts_with("sass:")
}

fn format_query_sass_symbol_label(symbol_kind: &str, name: &str) -> String {
    match symbol_kind {
        "variable" => format!("Sass variable '${name}'"),
        "mixin" => format!("Sass mixin '@mixin {name}'"),
        "function" => format!("Sass function '{name}()'"),
        _ => format!("Sass symbol '{name}'"),
    }
}

fn sorted_unique_strings(values: &[String]) -> Vec<String> {
    values
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn sorted_name_intersection(left: &[String], right: &[String]) -> Vec<String> {
    let right = right.iter().map(String::as_str).collect::<BTreeSet<_>>();
    left.iter()
        .filter(|name| right.contains(name.as_str()))
        .cloned()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}
