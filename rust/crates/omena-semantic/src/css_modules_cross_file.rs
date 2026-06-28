//! CSS Modules cross-file closure and resolution summaries for semantic consumers.

use std::collections::{BTreeMap, BTreeSet};

use omena_cross_file_summary::{
    HypergraphClosurePath, collect_hypergraph_transitive_closure_paths,
};
use omena_resolver::{
    OmenaResolverStylePackageManifestV0, resolve_omena_resolver_style_module_source,
};
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CssModulesCrossFileStyleFactsV0 {
    pub style_path: String,
    pub class_selector_names: Vec<String>,
    pub css_module_value_definition_names: Vec<String>,
    pub css_module_value_import_edges: Vec<CssModulesValueImportEdgeFactV0>,
    pub css_module_value_definition_edges: Vec<CssModulesValueDefinitionEdgeFactV0>,
    pub css_module_composes_edges: Vec<CssModulesComposesEdgeFactV0>,
    pub icss_export_names: Vec<String>,
    pub icss_import_edges: Vec<CssModulesIcssImportEdgeFactV0>,
    pub icss_export_edges: Vec<CssModulesIcssExportEdgeFactV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CssModulesValueImportEdgeFactV0 {
    pub remote_name: String,
    pub local_name: String,
    pub import_source: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CssModulesValueDefinitionEdgeFactV0 {
    pub definition_name: String,
    pub reference_names: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CssModulesComposesEdgeFactV0 {
    pub kind: &'static str,
    pub owner_selector_names: Vec<String>,
    pub target_names: Vec<String>,
    pub import_source: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CssModulesIcssImportEdgeFactV0 {
    pub local_name: String,
    pub remote_name: String,
    pub import_source: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CssModulesIcssExportEdgeFactV0 {
    pub export_name: String,
    pub reference_names: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CssModulesCrossFileClosureSummaryV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub status: &'static str,
    pub style_count: usize,
    pub composes_closure_edge_count: usize,
    pub value_closure_edge_count: usize,
    pub icss_closure_edge_count: usize,
    pub composes_cycle_count: usize,
    pub value_cycle_count: usize,
    pub icss_cycle_count: usize,
    pub composes_closure_edges: Vec<CssModulesComposesClosureEdgeV0>,
    pub value_closure_edges: Vec<CssModulesValueClosureEdgeV0>,
    pub icss_closure_edges: Vec<CssModulesIcssClosureEdgeV0>,
    pub cycles: Vec<CssModulesCycleV0>,
    pub capabilities: CssModulesCrossFileClosureCapabilitiesV0,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CssModulesCrossFileClosureCapabilitiesV0 {
    pub semantic_layer_owned: bool,
    pub composes_closure_ready: bool,
    pub value_graph_closure_ready: bool,
    pub icss_export_import_closure_ready: bool,
    pub cycle_detection_ready: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CssModulesCrossFileResolutionSummaryV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub status: &'static str,
    pub resolution_scope: &'static str,
    pub style_count: usize,
    pub import_edge_count: usize,
    pub resolved_import_edge_count: usize,
    pub unresolved_import_edge_count: usize,
    pub matched_name_count: usize,
    pub edges: Vec<CssModulesImportEdgeResolutionV0>,
    pub composes_closure_edge_count: usize,
    pub value_closure_edge_count: usize,
    pub icss_closure_edge_count: usize,
    pub composes_cycle_count: usize,
    pub value_cycle_count: usize,
    pub icss_cycle_count: usize,
    pub composes_closure_edges: Vec<CssModulesComposesClosureEdgeV0>,
    pub value_closure_edges: Vec<CssModulesValueClosureEdgeV0>,
    pub icss_closure_edges: Vec<CssModulesIcssClosureEdgeV0>,
    pub cycles: Vec<CssModulesCycleV0>,
    pub capabilities: CssModulesCrossFileResolutionCapabilitiesV0,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CssModulesImportEdgeResolutionV0 {
    pub from_style_path: String,
    pub import_kind: &'static str,
    pub source: String,
    pub resolved_style_path: Option<String>,
    pub status: &'static str,
    pub import_graph_distance: Option<usize>,
    pub import_graph_order: Option<usize>,
    pub imported_names: Vec<String>,
    pub exported_names: Vec<String>,
    pub matched_names: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CssModulesCrossFileResolutionCapabilitiesV0 {
    pub semantic_layer_owned: bool,
    pub import_source_resolution_ready: bool,
    pub composes_name_match_ready: bool,
    pub value_name_match_ready: bool,
    pub icss_name_match_ready: bool,
    pub transitive_closure_ready: bool,
    pub value_graph_closure_ready: bool,
    pub icss_export_import_closure_ready: bool,
    pub cycle_detection_ready: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CssModulesComposesClosureEdgeV0 {
    pub from_style_path: String,
    pub owner_selector_name: String,
    pub target_style_path: String,
    pub target_selector_name: String,
    pub depth: usize,
    pub path: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CssModulesValueClosureEdgeV0 {
    pub from_style_path: String,
    pub value_name: String,
    pub target_style_path: String,
    pub target_value_name: String,
    pub depth: usize,
    pub path: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CssModulesIcssClosureEdgeV0 {
    pub from_style_path: String,
    pub name: String,
    pub target_style_path: String,
    pub target_name: String,
    pub depth: usize,
    pub path: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CssModulesCycleV0 {
    pub kind: &'static str,
    pub path: Vec<String>,
}

pub fn summarize_css_modules_cross_file_closure(
    style_facts: &[CssModulesCrossFileStyleFactsV0],
    package_manifests: &[OmenaResolverStylePackageManifestV0],
) -> CssModulesCrossFileClosureSummaryV0 {
    let available_style_paths = style_facts
        .iter()
        .map(|entry| entry.style_path.as_str())
        .collect::<BTreeSet<_>>();
    let facts_by_path = style_facts
        .iter()
        .map(|entry| (entry.style_path.as_str(), entry))
        .collect::<BTreeMap<_, _>>();
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

    CssModulesCrossFileClosureSummaryV0 {
        schema_version: "0",
        product: "omena-semantic.css-modules-cross-file-closure",
        status: "semanticLayerOwnedClosure",
        style_count: style_facts.len(),
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
        capabilities: CssModulesCrossFileClosureCapabilitiesV0 {
            semantic_layer_owned: true,
            composes_closure_ready: true,
            value_graph_closure_ready: true,
            icss_export_import_closure_ready: true,
            cycle_detection_ready: true,
        },
    }
}

pub fn summarize_css_modules_cross_file_resolution(
    style_facts: &[CssModulesCrossFileStyleFactsV0],
    style_import_edges: &[crate::sass_module_graph::StyleImportReachabilityEdgeFactV0],
    package_manifests: &[OmenaResolverStylePackageManifestV0],
) -> CssModulesCrossFileResolutionSummaryV0 {
    let available_style_paths = style_facts
        .iter()
        .map(|entry| entry.style_path.as_str())
        .collect::<BTreeSet<_>>();
    let facts_by_path = style_facts
        .iter()
        .map(|entry| (entry.style_path.as_str(), entry))
        .collect::<BTreeMap<_, _>>();
    let mut edges = Vec::new();

    for entry in style_facts {
        let style_path = entry.style_path.as_str();
        let reachable =
            collect_import_reachable_style_path_metadata(style_path, style_import_edges);

        for edge in &entry.css_module_composes_edges {
            let Some(source) = edge.import_source.as_deref() else {
                continue;
            };
            edges.push(resolve_css_modules_import_edge(
                style_path,
                "composes",
                source,
                edge.target_names.as_slice(),
                &available_style_paths,
                &facts_by_path,
                &reachable,
                package_manifests,
                |target| target.class_selector_names.as_slice(),
            ));
        }

        for edge in &entry.css_module_value_import_edges {
            edges.push(resolve_css_modules_import_edge(
                style_path,
                "value",
                edge.import_source.as_str(),
                std::slice::from_ref(&edge.remote_name),
                &available_style_paths,
                &facts_by_path,
                &reachable,
                package_manifests,
                |target| target.css_module_value_definition_names.as_slice(),
            ));
        }

        for edge in &entry.icss_import_edges {
            edges.push(resolve_css_modules_import_edge(
                style_path,
                "icss",
                edge.import_source.as_str(),
                std::slice::from_ref(&edge.remote_name),
                &available_style_paths,
                &facts_by_path,
                &reachable,
                package_manifests,
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
    let closure_summary = summarize_css_modules_cross_file_closure(style_facts, package_manifests);
    let resolved_import_edge_count = edges
        .iter()
        .filter(|edge| edge.resolved_style_path.is_some())
        .count();
    let matched_name_count = edges
        .iter()
        .map(|edge| edge.matched_names.len())
        .sum::<usize>();

    CssModulesCrossFileResolutionSummaryV0 {
        schema_version: "0",
        product: "omena-semantic.css-modules-cross-file-resolution",
        status: "semanticLayerOwnedResolution",
        resolution_scope: "batchImportGraph",
        style_count: style_facts.len(),
        import_edge_count: edges.len(),
        resolved_import_edge_count,
        unresolved_import_edge_count: edges.len() - resolved_import_edge_count,
        matched_name_count,
        edges,
        composes_closure_edge_count: closure_summary.composes_closure_edge_count,
        value_closure_edge_count: closure_summary.value_closure_edge_count,
        icss_closure_edge_count: closure_summary.icss_closure_edge_count,
        composes_cycle_count: closure_summary.composes_cycle_count,
        value_cycle_count: closure_summary.value_cycle_count,
        icss_cycle_count: closure_summary.icss_cycle_count,
        composes_closure_edges: closure_summary.composes_closure_edges,
        value_closure_edges: closure_summary.value_closure_edges,
        icss_closure_edges: closure_summary.icss_closure_edges,
        cycles: closure_summary.cycles,
        capabilities: CssModulesCrossFileResolutionCapabilitiesV0 {
            semantic_layer_owned: true,
            import_source_resolution_ready: true,
            composes_name_match_ready: true,
            value_name_match_ready: true,
            icss_name_match_ready: true,
            transitive_closure_ready: true,
            value_graph_closure_ready: true,
            icss_export_import_closure_ready: true,
            cycle_detection_ready: true,
        },
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ImportReachability {
    distance: usize,
    order: usize,
}

fn collect_import_reachable_style_path_metadata(
    target_style_path: &str,
    style_import_edges: &[crate::sass_module_graph::StyleImportReachabilityEdgeFactV0],
) -> BTreeMap<String, ImportReachability> {
    crate::sass_module_graph::summarize_style_import_reachability(
        target_style_path,
        style_import_edges,
    )
    .reachable_style_paths
    .into_iter()
    .map(|fact| {
        (
            fact.style_path,
            ImportReachability {
                distance: fact.distance,
                order: fact.order,
            },
        )
    })
    .collect()
}

#[allow(clippy::too_many_arguments)]
fn resolve_css_modules_import_edge(
    from_style_path: &str,
    import_kind: &'static str,
    source: &str,
    imported_names: &[String],
    available_style_paths: &BTreeSet<&str>,
    facts_by_path: &BTreeMap<&str, &CssModulesCrossFileStyleFactsV0>,
    reachable: &BTreeMap<String, ImportReachability>,
    package_manifests: &[OmenaResolverStylePackageManifestV0],
    exported_names_for_kind: fn(&CssModulesCrossFileStyleFactsV0) -> &[String],
) -> CssModulesImportEdgeResolutionV0 {
    let resolved_style_path = resolve_omena_resolver_style_module_source(
        from_style_path,
        source,
        available_style_paths,
        package_manifests,
    );
    let reachability = resolved_style_path
        .as_ref()
        .and_then(|style_path| reachable.get(style_path));
    let exported_names = resolved_style_path
        .as_deref()
        .and_then(|style_path| facts_by_path.get(style_path))
        .map(|facts| exported_names_for_kind(facts).to_vec())
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

    CssModulesImportEdgeResolutionV0 {
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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct CssModulesComposesNode {
    style_path: String,
    selector_name: String,
}

fn summarize_css_modules_composes_closure(
    facts_by_path: &BTreeMap<&str, &CssModulesCrossFileStyleFactsV0>,
    available_style_paths: &BTreeSet<&str>,
    package_manifests: &[OmenaResolverStylePackageManifestV0],
) -> (Vec<CssModulesComposesClosureEdgeV0>, Vec<CssModulesCycleV0>) {
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
             }| CssModulesComposesClosureEdgeV0 {
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
        .map(|path| CssModulesCycleV0 {
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
    facts_by_path: &BTreeMap<&str, &CssModulesCrossFileStyleFactsV0>,
    available_style_paths: &BTreeSet<&str>,
    package_manifests: &[OmenaResolverStylePackageManifestV0],
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
                    resolve_omena_resolver_style_module_source(
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
    facts_by_path: &BTreeMap<&str, &CssModulesCrossFileStyleFactsV0>,
    available_style_paths: &BTreeSet<&str>,
    package_manifests: &[OmenaResolverStylePackageManifestV0],
) -> (Vec<CssModulesValueClosureEdgeV0>, Vec<CssModulesCycleV0>) {
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
             }| CssModulesValueClosureEdgeV0 {
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
        .map(|path| CssModulesCycleV0 {
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
    facts_by_path: &BTreeMap<&str, &CssModulesCrossFileStyleFactsV0>,
    available_style_paths: &BTreeSet<&str>,
    package_manifests: &[OmenaResolverStylePackageManifestV0],
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
            let Some(target_style_path) = resolve_omena_resolver_style_module_source(
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
    facts_by_path: &BTreeMap<&str, &CssModulesCrossFileStyleFactsV0>,
    available_style_paths: &BTreeSet<&str>,
    package_manifests: &[OmenaResolverStylePackageManifestV0],
) -> (Vec<CssModulesIcssClosureEdgeV0>, Vec<CssModulesCycleV0>) {
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
             }| CssModulesIcssClosureEdgeV0 {
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
        .map(|path| CssModulesCycleV0 { kind: "icss", path })
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
    facts_by_path: &BTreeMap<&str, &CssModulesCrossFileStyleFactsV0>,
    available_style_paths: &BTreeSet<&str>,
    package_manifests: &[OmenaResolverStylePackageManifestV0],
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
            let Some(target_style_path) = resolve_omena_resolver_style_module_source(
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
