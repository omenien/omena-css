use super::*;
use omena_parser::{ParsedSassIncludeFact, ParsedSelectorFact, ParsedVariableFact};

mod cascade_position;
mod code_actions;
mod completion;
mod cross_file_hypergraph;
mod cross_file_summary;
mod diagnostic_suppressions;
mod diagnostics;
mod dynamic_classname;
mod insights;
mod parser_facade;
mod sass;
mod source_refs;
mod stylesheet_evaluation;
mod substrate;
mod transform;

pub use cascade_position::*;
pub use code_actions::*;
pub use completion::*;
#[cfg(feature = "hypergraph-ifds")]
pub use cross_file_hypergraph::*;
use cross_file_hypergraph::{
    HypergraphClosureMode, HypergraphClosurePath, collect_hypergraph_transitive_closure_paths,
    collect_hypergraph_transitive_closure_paths_with_mode,
};
use cross_file_summary::summarize_omena_query_cross_file_summary;
pub use cross_file_summary::{
    summarize_omena_query_categorical_design_system_cross_project_summary,
    summarize_omena_query_m4_axis_c_readiness,
    summarize_omena_query_source_selector_reference_cross_file_summary,
    summarize_omena_query_workspace_cross_file_summary,
};
pub use diagnostics::*;
pub use dynamic_classname::*;
pub use insights::*;
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
pub use sass::*;
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
    let sass_module_resolution = summarize_sass_module_cross_file_resolution(
        &style_fact_entries,
        package_manifests,
        &[],
        &[],
    );
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

pub fn summarize_omena_query_sass_module_cross_file_resolution_for_workspace(
    style_sources: &[OmenaQueryStyleSourceInputV0],
    package_manifests: &[OmenaQueryStylePackageManifestV0],
    bundler_path_mappings: &[OmenaResolverBundlerPathAliasMappingV0],
    tsconfig_path_mappings: &[OmenaResolverTsconfigPathMappingV0],
) -> OmenaQuerySassModuleCrossFileResolutionV0 {
    let style_source_refs = style_sources
        .iter()
        .map(|source| (source.style_path.as_str(), source.style_source.as_str()))
        .collect::<Vec<_>>();
    let style_fact_entries = collect_omena_query_style_fact_entries(style_source_refs.as_slice());
    summarize_sass_module_cross_file_resolution(
        &style_fact_entries,
        package_manifests,
        bundler_path_mappings,
        tsconfig_path_mappings,
    )
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

/// Derive the load-path roots to try when joining a load-path-rooted `@use` (dart-sass
/// `--load-path`). Each in-graph style file contributes its ancestor directories: a path-shaped
/// specifier `src/scss/design-system.scss` is then joinable under any root `<R>` for which
/// `<R>/src/scss/design-system.scss` is itself in-graph. The resolver accepts only such existing
/// candidates, so over-collecting roots cannot fabricate a spurious edge. (RFC-0007-I, #49)
fn collect_load_path_roots(available_style_paths: &BTreeSet<&str>) -> Vec<String> {
    let mut roots = BTreeSet::new();
    for path in available_style_paths {
        let mut current = *path;
        // Walk up the directory chain on the normalized `/` separator. Style paths flowing
        // through the query layer are already forward-slash normalized by the resolver.
        while let Some(parent_end) = current.rfind('/') {
            if parent_end == 0 {
                // Keep the filesystem root (`/`) as a candidate load-path root.
                roots.insert("/".to_string());
                break;
            }
            let parent = &current[..parent_end];
            if !roots.insert(parent.to_string()) {
                // This ancestor (and therefore all of its ancestors) is already recorded.
                break;
            }
            current = parent;
        }
    }
    roots.into_iter().collect()
}

fn summarize_sass_module_cross_file_resolution(
    style_fact_entries: &[OmenaQueryStyleFactEntry],
    package_manifests: &[OmenaQueryStylePackageManifestV0],
    bundler_path_mappings: &[OmenaResolverBundlerPathAliasMappingV0],
    tsconfig_path_mappings: &[OmenaResolverTsconfigPathMappingV0],
) -> OmenaQuerySassModuleCrossFileResolutionV0 {
    let available_style_paths = style_fact_entries
        .iter()
        .map(|entry| entry.style_path.as_str())
        .collect::<BTreeSet<_>>();
    // Load-path roots are the ancestor directories of the in-graph style files. A
    // load-path-rooted `@use 'src/scss/design-system.scss'` (dart-sass `--load-path`) is joined
    // only when `<root>/src/scss/design-system.scss` is itself an in-graph file, so deriving
    // roots from `available_style_paths` keeps the join sound without new configuration input,
    // and never shadows the file-relative or bare-package routes. (RFC-0007-I, #49)
    let load_path_roots = collect_load_path_roots(&available_style_paths);
    let load_path_root_refs = load_path_roots
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>();
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
            let resolution = summarize_omena_resolver_style_module_resolution_with_load_path_roots(
                entry.style_path.as_str(),
                edge.source.as_str(),
                &available_style_paths,
                &resolver_package_manifests,
                bundler_path_mappings,
                tsconfig_path_mappings,
                &load_path_root_refs,
            );
            let status = if resolution.resolution_kind == "externalIgnored" {
                "external"
            } else if resolution.resolved_style_path.is_some() {
                "resolved"
            } else {
                "unresolved"
            };
            let resolved_style_path = resolution.resolved_style_path;
            let symlink_chain_link_count = resolution.symlink_chain.link_count;
            let symlink_chain_links = resolution
                .symlink_chain
                .links
                .into_iter()
                .map(|link| OmenaQuerySymlinkChainLinkV0 {
                    link_path: link.link_path,
                    target_path: link.target_path,
                    target_was_absolute: link.target_was_absolute,
                })
                .collect::<Vec<_>>();
            let configuration_evidence =
                transform::derive_static_scss_module_resolution_configuration_evidence(
                    entry.style_source.as_str(),
                    edge.kind,
                    edge.source.as_str(),
                    resolved_style_path.as_deref(),
                );
            edges.push(OmenaQuerySassModuleEdgeResolutionV0 {
                from_style_path: entry.style_path.clone(),
                edge_kind: edge.kind,
                source: edge.source.clone(),
                namespace_kind: edge.namespace_kind,
                namespace: edge.namespace.clone(),
                forward_prefix: edge.forward_prefix.clone(),
                visibility_filter_kind: edge.visibility_filter_kind,
                visibility_filter_names: edge.visibility_filter_names.clone(),
                resolved_style_path,
                status,
                resolution_kind: resolution.resolution_kind,
                candidate_count: resolution.candidate_count,
                symlink_chain_link_count,
                symlink_chain_links,
                configuration_signature: configuration_evidence.configuration_signature,
                configuration_variable_count: configuration_evidence.configuration_variable_count,
                module_instance_identity_key: configuration_evidence.module_instance_identity_key,
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
    let symlink_chain_edge_count = edges
        .iter()
        .filter(|edge| edge.symlink_chain_link_count > 0)
        .count();
    let symlink_chain_link_count = edges.iter().map(|edge| edge.symlink_chain_link_count).sum();
    let configured_module_instance_count = edges
        .iter()
        .filter(|edge| edge.module_instance_identity_key.is_some())
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
        symlink_chain_edge_count,
        symlink_chain_link_count,
        configured_module_instance_count,
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
            configured_module_instance_identity_ready: true,
            symlink_chain_metadata_ready: true,
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
    let mut resolved_edges = edges
        .iter()
        .filter(|edge| edge.status == "resolved" && edge.resolved_style_path.is_some())
        .collect::<Vec<_>>();
    resolved_edges.sort_by_key(|edge| {
        (
            edge.from_style_path.clone(),
            edge.resolved_style_path.clone().unwrap_or_default(),
            edge.edge_kind,
            edge.source.clone(),
        )
    });

    let mut graph = BTreeMap::<String, BTreeSet<String>>::new();
    let mut metadata_by_step =
        BTreeMap::<(String, String), SassModuleGraphClosureStepMetadata>::new();
    for edge in resolved_edges {
        let Some(target_style_path) = edge.resolved_style_path.clone() else {
            continue;
        };
        graph
            .entry(edge.from_style_path.clone())
            .or_default()
            .insert(target_style_path.clone());
        metadata_by_step
            .entry((edge.from_style_path.clone(), target_style_path))
            .or_insert_with(|| SassModuleGraphClosureStepMetadata::from(edge));
    }

    let (closure_paths, cycle_paths) = collect_hypergraph_transitive_closure_paths_with_mode(
        &graph,
        &mut |style_path: &String| style_path.clone(),
        HypergraphClosureMode::RawAllPaths,
    );
    let mut closure_edges = closure_paths
        .into_iter()
        .filter_map(
            |HypergraphClosurePath {
                 origin,
                 target,
                 depth,
                 path_labels,
             }| {
                let last_hop_from = path_labels.iter().rev().nth(1)?.clone();
                let metadata = metadata_by_step
                    .get(&(last_hop_from, target.clone()))?
                    .clone();
                Some(OmenaQuerySassModuleGraphClosureEdgeV0 {
                    from_style_path: origin,
                    target_style_path: target,
                    edge_kind: metadata.edge_kind,
                    depth,
                    path: path_labels,
                    namespace_kind: metadata.namespace_kind,
                    namespace: metadata.namespace,
                    forward_prefix: metadata.forward_prefix,
                    visibility_filter_kind: metadata.visibility_filter_kind,
                    visibility_filter_names: metadata.visibility_filter_names,
                    configuration_signature: metadata.configuration_signature,
                    configuration_variable_count: metadata.configuration_variable_count,
                    module_instance_identity_key: metadata.module_instance_identity_key,
                })
            },
        )
        .collect::<Vec<_>>();
    let mut cycles = cycle_paths
        .into_iter()
        .map(|path| OmenaQuerySassModuleCycleV0 { path })
        .collect::<Vec<_>>();
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

#[derive(Debug, Clone)]
struct SassModuleGraphClosureStepMetadata {
    edge_kind: &'static str,
    namespace_kind: Option<&'static str>,
    namespace: Option<String>,
    forward_prefix: Option<String>,
    visibility_filter_kind: Option<&'static str>,
    visibility_filter_names: Vec<String>,
    configuration_signature: String,
    configuration_variable_count: usize,
    module_instance_identity_key: Option<String>,
}

impl From<&OmenaQuerySassModuleEdgeResolutionV0> for SassModuleGraphClosureStepMetadata {
    fn from(edge: &OmenaQuerySassModuleEdgeResolutionV0) -> Self {
        Self {
            edge_kind: edge.edge_kind,
            namespace_kind: edge.namespace_kind,
            namespace: edge.namespace.clone(),
            forward_prefix: edge.forward_prefix.clone(),
            visibility_filter_kind: edge.visibility_filter_kind,
            visibility_filter_names: edge.visibility_filter_names.clone(),
            configuration_signature: edge.configuration_signature.clone(),
            configuration_variable_count: edge.configuration_variable_count,
            module_instance_identity_key: edge.module_instance_identity_key.clone(),
        }
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
    let (closure_paths, cycle_paths) =
        collect_hypergraph_transitive_closure_paths(&graph, css_modules_composes_node_label);
    let mut closure_edges = closure_paths
        .into_iter()
        .map(
            |HypergraphClosurePath {
                 origin,
                 target,
                 depth,
                 path_labels,
             }| OmenaQueryCssModulesComposesClosureEdgeV0 {
                from_style_path: origin.style_path,
                owner_selector_name: origin.selector_name,
                target_style_path: target.style_path,
                target_selector_name: target.selector_name,
                depth,
                path: path_labels,
            },
        )
        .collect::<Vec<_>>();
    let mut cycles = cycle_paths
        .into_iter()
        .map(|path| OmenaQueryCssModulesCycleV0 {
            kind: "composes",
            path,
        })
        .collect::<Vec<_>>();

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
    let (closure_paths, cycle_paths) =
        collect_hypergraph_transitive_closure_paths(&graph, css_modules_value_node_label);
    let mut closure_edges = closure_paths
        .into_iter()
        .map(
            |HypergraphClosurePath {
                 origin,
                 target,
                 depth,
                 path_labels,
             }| OmenaQueryCssModulesValueClosureEdgeV0 {
                from_style_path: origin.style_path,
                value_name: origin.value_name,
                target_style_path: target.style_path,
                target_value_name: target.value_name,
                depth,
                path: path_labels,
            },
        )
        .collect::<Vec<_>>();
    let mut cycles = cycle_paths
        .into_iter()
        .map(|path| OmenaQueryCssModulesCycleV0 {
            kind: "value",
            path,
        })
        .collect::<Vec<_>>();

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
    let (closure_paths, cycle_paths) =
        collect_hypergraph_transitive_closure_paths(&graph, css_modules_icss_node_label);
    let mut closure_edges = closure_paths
        .into_iter()
        .map(
            |HypergraphClosurePath {
                 origin,
                 target,
                 depth,
                 path_labels,
             }| OmenaQueryCssModulesIcssClosureEdgeV0 {
                from_style_path: origin.style_path,
                name: origin.name,
                target_style_path: target.style_path,
                target_name: target.name,
                depth,
                path: path_labels,
            },
        )
        .collect::<Vec<_>>();
    let mut cycles = cycle_paths
        .into_iter()
        .map(|path| OmenaQueryCssModulesCycleV0 { kind: "icss", path })
        .collect::<Vec<_>>();

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
    let available_style_paths = style_fact_entries
        .iter()
        .map(|entry| entry.style_path.as_str())
        .collect::<BTreeSet<_>>();
    let mut graph = BTreeMap::<String, BTreeSet<String>>::new();
    for entry in style_fact_entries {
        let targets = collect_sass_module_sources_from_facts(&entry.facts)
            .into_iter()
            .filter_map(|source| {
                resolve_style_module_source(
                    entry.style_path.as_str(),
                    &source,
                    &available_style_paths,
                    package_manifests,
                )
            })
            .collect::<BTreeSet<_>>();
        if !targets.is_empty() {
            graph.insert(entry.style_path.clone(), targets);
        }
    }

    let (closure_paths, _) =
        collect_hypergraph_transitive_closure_paths(&graph, |style_path: &String| {
            style_path.clone()
        });
    let mut reachable_style_paths = BTreeMap::new();
    let mut visit_order = 0usize;

    for path in closure_paths
        .into_iter()
        .filter(|path| path.origin == target_style_path)
    {
        if path.target == target_style_path || reachable_style_paths.contains_key(&path.target) {
            continue;
        }
        reachable_style_paths.insert(
            path.target.clone(),
            ImportReachability {
                distance: path.depth,
                order: visit_order,
            },
        );
        visit_order += 1;
    }

    reachable_style_paths
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

/// Alias-aware style-module resolution: the same routing as `resolve_style_module_source`, plus
/// tsconfig/bundler path-mapping resolution so a workspace-alias specifier (`@/styles/a.module.scss`)
/// resolves when the workspace's `paths`/`alias` config is wired in. RFC-0007-J (#50): the
/// unused-selector usage collector must use this so it agrees with the reference/goto path, which
/// already resolves aliases — otherwise an alias import leaves every selector dimmed `unusedSelector`.
fn resolve_style_module_source_with_path_mappings(
    from_style_path: &str,
    source: &str,
    available_style_paths: &BTreeSet<&str>,
    package_manifests: &[OmenaQueryStylePackageManifestV0],
    bundler_path_mappings: &[OmenaResolverBundlerPathAliasMappingV0],
    tsconfig_path_mappings: &[OmenaResolverTsconfigPathMappingV0],
) -> Option<String> {
    let resolver_package_manifests = package_manifests
        .iter()
        .map(|manifest| OmenaResolverStylePackageManifestV0 {
            package_json_path: manifest.package_json_path.clone(),
            package_json_source: manifest.package_json_source.clone(),
        })
        .collect::<Vec<_>>();
    resolve_omena_resolver_style_module_source_with_path_mappings(
        from_style_path,
        source,
        available_style_paths,
        &resolver_package_manifests,
        bundler_path_mappings,
        tsconfig_path_mappings,
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
