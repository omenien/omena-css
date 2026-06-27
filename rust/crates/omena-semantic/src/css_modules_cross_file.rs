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
