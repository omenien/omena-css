use std::collections::{BTreeMap, BTreeSet, VecDeque};

use omena_cross_file_summary::{
    HypergraphClosureMode, HypergraphClosurePath, collect_directed_graph_cycles,
    collect_hypergraph_transitive_closure_paths,
    collect_hypergraph_transitive_closure_paths_with_mode,
};
use omena_resolver::canonicalize_omena_resolver_style_identity_path;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SassModuleGraphEdgeFactV0 {
    pub from_style_path: String,
    pub edge_kind: &'static str,
    pub source: String,
    pub rule_ordinal: usize,
    pub namespace_kind: Option<&'static str>,
    pub namespace: Option<String>,
    pub forward_prefix: Option<String>,
    pub visibility_filter_kind: Option<&'static str>,
    pub visibility_filter_names: Vec<String>,
    pub resolved_style_path: Option<String>,
    pub status: &'static str,
    pub configuration_signature: String,
    pub configuration_variable_count: usize,
    pub invalid_configuration_variable_names: Vec<String>,
    pub module_instance_identity_key: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SassModuleGraphClosureSummaryV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub status: &'static str,
    pub module_edge_count: usize,
    pub graph_closure_edge_count: usize,
    pub cycle_count: usize,
    pub graph_closure_edges: Vec<SassModuleGraphClosureEdgeV0>,
    pub cycles: Vec<SassModuleCycleV0>,
    pub capabilities: SassModuleGraphClosureCapabilitiesV0,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SassModuleGraphClosureCapabilitiesV0 {
    pub semantic_layer_owned: bool,
    pub graph_closure_ready: bool,
    pub cycle_detection_ready: bool,
    pub namespace_show_hide_filter_ready: bool,
    pub configured_module_instance_identity_ready: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SassModuleGraphClosureEdgeV0 {
    pub from_style_path: String,
    pub target_style_path: String,
    pub edge_kind: &'static str,
    pub depth: usize,
    pub path: Vec<String>,
    pub namespace_kind: Option<&'static str>,
    pub namespace: Option<String>,
    pub forward_prefix: Option<String>,
    pub visibility_filter_kind: Option<&'static str>,
    pub visibility_filter_names: Vec<String>,
    pub configuration_signature: String,
    pub configuration_variable_count: usize,
    pub invalid_configuration_variable_names: Vec<String>,
    pub module_instance_identity_key: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SassModuleCycleV0 {
    pub path: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StyleImportReachabilityEdgeFactV0 {
    pub from_style_path: String,
    pub target_style_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StyleImportReachabilitySummaryV0 {
    pub schema_version: &'static str,
    pub product: &'static str,
    pub status: &'static str,
    pub target_style_path: String,
    pub edge_count: usize,
    pub reachable_style_path_count: usize,
    pub reachable_style_paths: Vec<StyleImportReachabilityFactV0>,
    pub capabilities: StyleImportReachabilityCapabilitiesV0,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StyleImportReachabilityFactV0 {
    pub style_path: String,
    pub distance: usize,
    pub order: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StyleImportReachabilityCapabilitiesV0 {
    pub semantic_layer_owned: bool,
    pub transitive_reachability_ready: bool,
    pub stable_distance_ready: bool,
    pub stable_order_ready: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct SassModuleUseConfigurationRequestV0<'a> {
    pub from_style_path: &'a str,
    pub rule_ordinal: usize,
}

#[derive(Debug, Clone, Copy)]
pub struct SassModuleForwardConfigurationRequestV0<'a> {
    pub from_style_path: &'a str,
    pub target_style_path: &'a str,
    pub rule_ordinal: usize,
    pub inherited_variable_overrides: &'a BTreeMap<String, String>,
    pub forward_prefix: Option<&'a str>,
    pub visibility_filter_kind: Option<&'static str>,
    pub visibility_filter_names: &'a [String],
    pub configurable_names: &'a BTreeSet<String>,
}

pub trait SassModuleGraphConfigurationResolverV0 {
    fn use_variable_overrides(
        &self,
        request: SassModuleUseConfigurationRequestV0<'_>,
    ) -> BTreeMap<String, String>;

    fn forward_effective_variable_overrides(
        &self,
        request: SassModuleForwardConfigurationRequestV0<'_>,
    ) -> BTreeMap<String, String>;

    fn configurable_names(&self, target_style_path: &str) -> BTreeSet<String>;
}

pub fn summarize_sass_module_configuration_signature(
    variable_overrides: &BTreeMap<String, String>,
) -> String {
    if variable_overrides.is_empty() {
        return "with:none".to_string();
    }
    let mut key = String::from("with");
    for (name, value) in variable_overrides {
        key.push('|');
        key.push_str(name.len().to_string().as_str());
        key.push(':');
        key.push_str(name);
        key.push('=');
        key.push_str(value.len().to_string().as_str());
        key.push(':');
        key.push_str(value);
    }
    key
}

pub fn summarize_sass_module_instance_identity_key(
    style_path: &str,
    variable_overrides: &BTreeMap<String, String>,
) -> String {
    let canonical_path = canonicalize_omena_resolver_style_identity_path(style_path);
    let mut key = format!("path:{}:{canonical_path}", canonical_path.len());
    key.push('|');
    key.push_str(summarize_sass_module_configuration_signature(variable_overrides).as_str());
    key
}

pub fn summarize_sass_module_graph_closure(
    edges: &[SassModuleGraphEdgeFactV0],
    configuration_resolver: &impl SassModuleGraphConfigurationResolverV0,
) -> SassModuleGraphClosureSummaryV0 {
    let (graph_closure_edges, cycles) =
        summarize_sass_module_graph_closure_edges(edges, configuration_resolver);
    SassModuleGraphClosureSummaryV0 {
        schema_version: "0",
        product: "omena-semantic.sass-module-graph-closure",
        status: "semanticLayerOwnedClosure",
        module_edge_count: edges.len(),
        graph_closure_edge_count: graph_closure_edges.len(),
        cycle_count: cycles.len(),
        graph_closure_edges,
        cycles,
        capabilities: SassModuleGraphClosureCapabilitiesV0 {
            semantic_layer_owned: true,
            graph_closure_ready: true,
            cycle_detection_ready: true,
            namespace_show_hide_filter_ready: true,
            configured_module_instance_identity_ready: true,
        },
    }
}

pub fn summarize_style_import_reachability(
    target_style_path: &str,
    edges: &[StyleImportReachabilityEdgeFactV0],
) -> StyleImportReachabilitySummaryV0 {
    let mut graph = BTreeMap::<String, BTreeSet<String>>::new();
    for edge in edges {
        graph
            .entry(edge.from_style_path.clone())
            .or_default()
            .insert(edge.target_style_path.clone());
    }

    let (closure_paths, _) =
        collect_hypergraph_transitive_closure_paths(&graph, |style_path: &String| {
            style_path.clone()
        });
    let mut seen = BTreeSet::new();
    let mut reachable_style_paths = Vec::new();
    for path in closure_paths
        .into_iter()
        .filter(|path| path.origin == target_style_path)
    {
        if path.target == target_style_path || !seen.insert(path.target.clone()) {
            continue;
        }
        let order = reachable_style_paths.len();
        reachable_style_paths.push(StyleImportReachabilityFactV0 {
            style_path: path.target,
            distance: path.depth,
            order,
        });
    }

    StyleImportReachabilitySummaryV0 {
        schema_version: "0",
        product: "omena-semantic.style-import-reachability",
        status: "semanticLayerOwnedReachability",
        target_style_path: target_style_path.to_string(),
        edge_count: edges.len(),
        reachable_style_path_count: reachable_style_paths.len(),
        reachable_style_paths,
        capabilities: StyleImportReachabilityCapabilitiesV0 {
            semantic_layer_owned: true,
            transitive_reachability_ready: true,
            stable_distance_ready: true,
            stable_order_ready: true,
        },
    }
}

fn summarize_sass_module_graph_closure_edges(
    edges: &[SassModuleGraphEdgeFactV0],
    configuration_resolver: &impl SassModuleGraphConfigurationResolverV0,
) -> (Vec<SassModuleGraphClosureEdgeV0>, Vec<SassModuleCycleV0>) {
    let mut resolved_edges = edges
        .iter()
        .filter(|edge| edge.status == "resolved" && edge.resolved_style_path.is_some())
        .collect::<Vec<_>>();
    resolved_edges.sort_by_key(|edge| {
        (
            edge.from_style_path.clone(),
            edge.resolved_style_path.clone().unwrap_or_default(),
            edge.edge_kind,
            edge.rule_ordinal,
            edge.source.clone(),
        )
    });

    let mut graph = BTreeMap::<String, BTreeSet<String>>::new();
    let mut metadata_by_step =
        BTreeMap::<(String, String), Vec<SassModuleGraphClosureStepMetadata>>::new();
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
            .or_default()
            .push(SassModuleGraphClosureStepMetadata::from(edge));
    }

    let cycle_paths = collect_directed_graph_cycles(&graph);

    if test_force_rawallpaths_closure() {
        let (closure_paths, _) = collect_hypergraph_transitive_closure_paths_with_mode(
            &graph,
            &mut |style_path: &String| style_path.clone(),
            HypergraphClosureMode::RawAllPaths,
        );
        let closure_edges = sass_module_graph_closure_edges_from_paths(
            closure_paths,
            &metadata_by_step,
            configuration_resolver,
        );
        return finalize_sass_module_graph_closure(closure_edges, cycle_paths);
    }

    let (mut closure_edges, capped) = collect_sass_module_graph_closure_edges_via_worklist(
        &graph,
        &metadata_by_step,
        configuration_resolver,
        SASS_MODULE_CLOSURE_STATE_CAP,
    );
    if capped {
        let (closure_paths, _) =
            collect_hypergraph_transitive_closure_paths(&graph, |style_path: &String| {
                style_path.clone()
            });
        closure_edges = sass_module_graph_closure_edges_from_paths(
            closure_paths,
            &metadata_by_step,
            configuration_resolver,
        );
    }
    finalize_sass_module_graph_closure(closure_edges, cycle_paths)
}

fn finalize_sass_module_graph_closure(
    mut closure_edges: Vec<SassModuleGraphClosureEdgeV0>,
    cycle_paths: Vec<Vec<String>>,
) -> (Vec<SassModuleGraphClosureEdgeV0>, Vec<SassModuleCycleV0>) {
    let mut cycles = cycle_paths
        .into_iter()
        .map(|path| SassModuleCycleV0 { path })
        .collect::<Vec<_>>();
    closure_edges.sort_by_key(|edge| {
        (
            edge.from_style_path.clone(),
            edge.depth,
            edge.target_style_path.clone(),
            edge.edge_kind,
            edge.configuration_signature.clone(),
            edge.module_instance_identity_key
                .clone()
                .unwrap_or_default(),
            edge.path.clone(),
        )
    });
    closure_edges.dedup();
    cycles.sort_by_key(|cycle| cycle.path.clone());
    (closure_edges, cycles)
}

#[derive(Debug, Clone)]
struct SassModuleGraphClosureStepMetadata {
    rule_ordinal: usize,
    edge_kind: &'static str,
    namespace_kind: Option<&'static str>,
    namespace: Option<String>,
    forward_prefix: Option<String>,
    visibility_filter_kind: Option<&'static str>,
    visibility_filter_names: Vec<String>,
    configuration_signature: String,
    configuration_variable_count: usize,
    invalid_configuration_variable_names: Vec<String>,
    module_instance_identity_key: Option<String>,
}

impl From<&SassModuleGraphEdgeFactV0> for SassModuleGraphClosureStepMetadata {
    fn from(edge: &SassModuleGraphEdgeFactV0) -> Self {
        Self {
            rule_ordinal: edge.rule_ordinal,
            edge_kind: edge.edge_kind,
            namespace_kind: edge.namespace_kind,
            namespace: edge.namespace.clone(),
            forward_prefix: edge.forward_prefix.clone(),
            visibility_filter_kind: edge.visibility_filter_kind,
            visibility_filter_names: edge.visibility_filter_names.clone(),
            configuration_signature: edge.configuration_signature.clone(),
            configuration_variable_count: edge.configuration_variable_count,
            invalid_configuration_variable_names: edge.invalid_configuration_variable_names.clone(),
            module_instance_identity_key: edge.module_instance_identity_key.clone(),
        }
    }
}

const SASS_MODULE_CLOSURE_STATE_CAP: usize = 1 << 16;

thread_local! {
    static FORCE_RAWALLPATHS_CLOSURE: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

fn test_force_rawallpaths_closure() -> bool {
    FORCE_RAWALLPATHS_CLOSURE.with(std::cell::Cell::get)
}

pub fn with_sass_module_rawallpaths_closure_for_test<R>(body: impl FnOnce() -> R) -> R {
    FORCE_RAWALLPATHS_CLOSURE.with(|cell| cell.set(true));
    let result = body();
    FORCE_RAWALLPATHS_CLOSURE.with(|cell| cell.set(false));
    result
}

fn sass_module_graph_closure_edge(
    origin: &str,
    target: &str,
    depth: usize,
    path: Vec<String>,
    metadata: SassModuleGraphClosureStepMetadata,
) -> SassModuleGraphClosureEdgeV0 {
    SassModuleGraphClosureEdgeV0 {
        from_style_path: origin.to_string(),
        target_style_path: target.to_string(),
        edge_kind: metadata.edge_kind,
        depth,
        path,
        namespace_kind: metadata.namespace_kind,
        namespace: metadata.namespace,
        forward_prefix: metadata.forward_prefix,
        visibility_filter_kind: metadata.visibility_filter_kind,
        visibility_filter_names: metadata.visibility_filter_names,
        configuration_signature: metadata.configuration_signature,
        configuration_variable_count: metadata.configuration_variable_count,
        invalid_configuration_variable_names: metadata.invalid_configuration_variable_names,
        module_instance_identity_key: metadata.module_instance_identity_key,
    }
}

fn sass_module_graph_closure_edges_from_paths(
    closure_paths: Vec<HypergraphClosurePath<String>>,
    metadata_by_step: &BTreeMap<(String, String), Vec<SassModuleGraphClosureStepMetadata>>,
    configuration_resolver: &impl SassModuleGraphConfigurationResolverV0,
) -> Vec<SassModuleGraphClosureEdgeV0> {
    closure_paths
        .into_iter()
        .flat_map(
            |HypergraphClosurePath {
                 origin,
                 target,
                 depth,
                 path_labels,
             }| {
                derive_sass_module_graph_closure_path_metadata(
                    path_labels.as_slice(),
                    metadata_by_step,
                    configuration_resolver,
                )
                .into_iter()
                .map(move |metadata| {
                    sass_module_graph_closure_edge(
                        &origin,
                        &target,
                        depth,
                        path_labels.clone(),
                        metadata,
                    )
                })
                .collect::<Vec<_>>()
            },
        )
        .collect()
}

fn collect_sass_module_graph_closure_edges_via_worklist(
    graph: &BTreeMap<String, BTreeSet<String>>,
    metadata_by_step: &BTreeMap<(String, String), Vec<SassModuleGraphClosureStepMetadata>>,
    configuration_resolver: &impl SassModuleGraphConfigurationResolverV0,
    per_origin_state_cap: usize,
) -> (Vec<SassModuleGraphClosureEdgeV0>, bool) {
    let mut edges = Vec::new();
    for origin in graph.keys() {
        let mut visited = BTreeSet::<(String, BTreeMap<String, String>)>::new();
        let mut pending = VecDeque::<(String, BTreeMap<String, String>, usize, Vec<String>)>::new();
        visited.insert((origin.clone(), BTreeMap::new()));
        pending.push_back((origin.clone(), BTreeMap::new(), 0, vec![origin.clone()]));
        let mut state_count = 0usize;
        while let Some((node, inherited_overrides, depth, path)) = pending.pop_front() {
            state_count += 1;
            if state_count > per_origin_state_cap {
                return (edges, true);
            }
            let Some(targets) = graph.get(node.as_str()) else {
                continue;
            };
            for target in targets {
                if path.contains(target) {
                    continue;
                }
                let Some(step_metadata) = metadata_by_step.get(&(node.clone(), target.clone()))
                else {
                    continue;
                };
                for metadata in step_metadata {
                    let variable_overrides =
                        derive_sass_module_graph_closure_step_variable_overrides(
                            node.as_str(),
                            target.as_str(),
                            metadata,
                            &inherited_overrides,
                            configuration_resolver,
                        );
                    let applied = apply_sass_module_graph_closure_step_configuration(
                        metadata.clone(),
                        target.as_str(),
                        variable_overrides.clone(),
                        configuration_resolver,
                    );
                    let mut edge_path = path.clone();
                    edge_path.push(target.clone());
                    edges.push(sass_module_graph_closure_edge(
                        origin,
                        target,
                        depth + 1,
                        edge_path.clone(),
                        applied,
                    ));
                    let next_state = (target.clone(), variable_overrides);
                    if visited.insert(next_state.clone()) {
                        pending.push_back((target.clone(), next_state.1, depth + 1, edge_path));
                    }
                }
            }
        }
    }
    (edges, false)
}

fn derive_sass_module_graph_closure_path_metadata(
    path_labels: &[String],
    metadata_by_step: &BTreeMap<(String, String), Vec<SassModuleGraphClosureStepMetadata>>,
    configuration_resolver: &impl SassModuleGraphConfigurationResolverV0,
) -> Vec<SassModuleGraphClosureStepMetadata> {
    let mut states = vec![(BTreeMap::<String, String>::new(), None)];

    for step in path_labels.windows(2) {
        let Some(from_style_path) = step.first() else {
            return Vec::new();
        };
        let Some(target_style_path) = step.get(1) else {
            return Vec::new();
        };
        let Some(step_metadata) =
            metadata_by_step.get(&(from_style_path.clone(), target_style_path.clone()))
        else {
            return Vec::new();
        };
        let mut next_states = Vec::new();
        for (inherited_variable_overrides, _) in &states {
            for metadata in step_metadata {
                let variable_overrides = derive_sass_module_graph_closure_step_variable_overrides(
                    from_style_path,
                    target_style_path,
                    metadata,
                    inherited_variable_overrides,
                    configuration_resolver,
                );
                let applied_metadata = apply_sass_module_graph_closure_step_configuration(
                    metadata.clone(),
                    target_style_path,
                    variable_overrides.clone(),
                    configuration_resolver,
                );
                next_states.push((variable_overrides, Some(applied_metadata)));
            }
        }
        states = next_states;
    }

    states
        .into_iter()
        .filter_map(|(_, metadata)| metadata)
        .collect()
}

fn derive_sass_module_graph_closure_step_variable_overrides(
    from_style_path: &str,
    target_style_path: &str,
    metadata: &SassModuleGraphClosureStepMetadata,
    inherited_variable_overrides: &BTreeMap<String, String>,
    configuration_resolver: &impl SassModuleGraphConfigurationResolverV0,
) -> BTreeMap<String, String> {
    match metadata.edge_kind {
        "sassForward" => {
            let configurable_names = configuration_resolver.configurable_names(target_style_path);
            configuration_resolver.forward_effective_variable_overrides(
                SassModuleForwardConfigurationRequestV0 {
                    from_style_path,
                    target_style_path,
                    rule_ordinal: metadata.rule_ordinal,
                    inherited_variable_overrides,
                    forward_prefix: metadata.forward_prefix.as_deref(),
                    visibility_filter_kind: metadata.visibility_filter_kind,
                    visibility_filter_names: &metadata.visibility_filter_names,
                    configurable_names: &configurable_names,
                },
            )
        }
        "sassUse" => {
            configuration_resolver.use_variable_overrides(SassModuleUseConfigurationRequestV0 {
                from_style_path,
                rule_ordinal: metadata.rule_ordinal,
            })
        }
        _ => BTreeMap::new(),
    }
}

fn apply_sass_module_graph_closure_step_configuration(
    mut metadata: SassModuleGraphClosureStepMetadata,
    target_style_path: &str,
    variable_overrides: BTreeMap<String, String>,
    configuration_resolver: &impl SassModuleGraphConfigurationResolverV0,
) -> SassModuleGraphClosureStepMetadata {
    let configurable_names = configuration_resolver.configurable_names(target_style_path);
    metadata.invalid_configuration_variable_names = variable_overrides
        .keys()
        .filter(|name| !configurable_names.contains(*name))
        .cloned()
        .collect();
    metadata.configuration_signature =
        summarize_sass_module_configuration_signature(&variable_overrides);
    metadata.configuration_variable_count = variable_overrides.len();
    metadata.module_instance_identity_key = Some(summarize_sass_module_instance_identity_key(
        target_style_path,
        &variable_overrides,
    ));
    metadata
}
