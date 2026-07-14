use std::{fs, path::PathBuf};

use omena_query::{
    OmenaQuerySassModuleCrossFileResolutionCapabilitiesV0, OmenaQuerySassModuleCycleV0,
    OmenaQuerySassModuleEdgeResolutionV0, OmenaQuerySassModuleGraphClosureEdgeV0,
    summarize_omena_query_sass_module_cross_file_resolution_for_workspace,
};
use serde::Serialize;

use crate::{
    commands::SassCommand,
    config::find_omena_config_for_path,
    io::{read_package_manifests, read_style_sources},
    lint::discover_workspace_files,
    output::{CliOutputMetadataV0, print_json},
    paths::path_string,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
struct SassModuleGraphViewV0 {
    schema_version: &'static str,
    product: &'static str,
    workspace_root: String,
    selected_module: Option<String>,
    style_count: usize,
    module_edge_count: usize,
    resolved_module_edge_count: usize,
    unresolved_module_edge_count: usize,
    external_module_edge_count: usize,
    configured_module_instance_count: usize,
    visibility_filter_count: usize,
    graph_closure_edge_count: usize,
    cycle_count: usize,
    edges: Vec<OmenaQuerySassModuleEdgeResolutionV0>,
    graph_closure_edges: Vec<OmenaQuerySassModuleGraphClosureEdgeV0>,
    cycles: Vec<OmenaQuerySassModuleCycleV0>,
    capabilities: OmenaQuerySassModuleCrossFileResolutionCapabilitiesV0,
}

pub(crate) fn sass_command(command: SassCommand) -> Result<(), String> {
    match command {
        SassCommand::Graph { root, module, json } => sass_graph(root, module, json),
    }
}

fn sass_graph(root: Option<PathBuf>, module: Option<PathBuf>, json: bool) -> Result<(), String> {
    let view = build_sass_graph_view(root, module)?;
    let loaded_config = find_omena_config_for_path(PathBuf::from(&view.workspace_root).as_path())?;
    if json {
        print_json(
            CliOutputMetadataV0::new("omena-cli.sass.graph").with_config_content_digest(
                loaded_config
                    .as_ref()
                    .map(|loaded| loaded.config_content_digest.as_ref()),
            ),
            &view,
        )?;
    } else {
        println!(
            "Sass module graph: {} edge(s), {} unresolved, {} visibility filter(s)",
            view.module_edge_count, view.unresolved_module_edge_count, view.visibility_filter_count
        );
        for edge in &view.edges {
            println!(
                "{} {} '{}' -> {} [{}]{}{}",
                edge.from_style_path,
                edge.edge_kind,
                edge.source,
                edge.resolved_style_path.as_deref().unwrap_or("unresolved"),
                edge.status,
                render_namespace(edge),
                render_visibility(edge),
            );
        }
    }
    Ok(())
}

fn build_sass_graph_view(
    root: Option<PathBuf>,
    module: Option<PathBuf>,
) -> Result<SassModuleGraphViewV0, String> {
    let workspace_input = root.unwrap_or_else(|| {
        module
            .as_deref()
            .and_then(|path| path.parent())
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."))
    });
    let workspace_root = fs::canonicalize(&workspace_input).map_err(|error| {
        format!(
            "failed to resolve Sass workspace {}: {error}",
            path_string(workspace_input.as_path())
        )
    })?;
    let selected_module = module
        .as_deref()
        .map(fs::canonicalize)
        .transpose()
        .map_err(|error| format!("failed to resolve selected Sass module: {error}"))?
        .map(|path| path_string(path.as_path()));
    if let Some(selected) = selected_module.as_deref()
        && !selected.starts_with(path_string(workspace_root.as_path()).as_str())
    {
        return Err(format!(
            "selected Sass module {selected} is outside workspace {}",
            path_string(workspace_root.as_path())
        ));
    }

    let files = discover_workspace_files(workspace_root.as_path())?;
    let style_sources = read_style_sources(files.style_paths.as_slice())?;
    if style_sources.is_empty() {
        return Err(format!(
            "no CSS-family sources found under {}",
            path_string(workspace_root.as_path())
        ));
    }
    if let Some(selected) = selected_module.as_deref()
        && !style_sources
            .iter()
            .any(|source| source.style_path == selected)
    {
        return Err(format!(
            "selected Sass module {selected} is not in the workspace graph"
        ));
    }
    let package_manifests = read_package_manifests(files.package_manifest_paths.as_slice())?;
    let resolution = summarize_omena_query_sass_module_cross_file_resolution_for_workspace(
        style_sources.as_slice(),
        package_manifests.as_slice(),
        &[],
        &[],
    );
    let edges = resolution
        .edges
        .into_iter()
        .filter(|edge| {
            selected_module
                .as_ref()
                .is_none_or(|selected| &edge.from_style_path == selected)
        })
        .collect::<Vec<_>>();
    let graph_closure_edges = resolution
        .graph_closure_edges
        .into_iter()
        .filter(|edge| {
            selected_module
                .as_ref()
                .is_none_or(|selected| &edge.from_style_path == selected)
        })
        .collect::<Vec<_>>();
    let cycles = resolution
        .cycles
        .into_iter()
        .filter(|cycle| {
            selected_module
                .as_ref()
                .is_none_or(|selected| cycle.path.iter().any(|path| path == selected))
        })
        .collect::<Vec<_>>();

    Ok(SassModuleGraphViewV0 {
        schema_version: "0",
        product: "omena-cli.sass.graph",
        workspace_root: path_string(workspace_root.as_path()),
        selected_module: selected_module.clone(),
        style_count: selected_module
            .as_ref()
            .map_or(resolution.style_count, |_| 1),
        module_edge_count: edges.len(),
        resolved_module_edge_count: edges
            .iter()
            .filter(|edge| edge.status == "resolved")
            .count(),
        unresolved_module_edge_count: edges
            .iter()
            .filter(|edge| edge.status == "unresolved")
            .count(),
        external_module_edge_count: edges
            .iter()
            .filter(|edge| edge.status == "external")
            .count(),
        configured_module_instance_count: edges
            .iter()
            .filter(|edge| edge.module_instance_identity_key.is_some())
            .count(),
        visibility_filter_count: edges
            .iter()
            .filter(|edge| edge.visibility_filter_kind.is_some())
            .count(),
        graph_closure_edge_count: graph_closure_edges.len(),
        cycle_count: cycles.len(),
        edges,
        graph_closure_edges,
        cycles,
        capabilities: resolution.capabilities,
    })
}

fn render_namespace(edge: &OmenaQuerySassModuleEdgeResolutionV0) -> String {
    edge.namespace
        .as_deref()
        .map_or_else(String::new, |namespace| format!(" namespace={namespace}"))
}

fn render_visibility(edge: &OmenaQuerySassModuleEdgeResolutionV0) -> String {
    edge.visibility_filter_kind
        .map_or_else(String::new, |kind| {
            format!(" {kind}={}", edge.visibility_filter_names.join(","))
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn graph_view_preserves_namespace_visibility_and_unresolved_edges() -> Result<(), String> {
        let root = temp_dir("sass-graph-view");
        fs::create_dir_all(&root).map_err(|error| error.to_string())?;
        let entry = root.join("entry.scss");
        fs::write(
            &entry,
            "@use './tokens' as theme;\n@forward './public' show $brand;\n@use './missing';",
        )
        .map_err(|error| error.to_string())?;
        fs::write(root.join("_tokens.scss"), "$space: 1rem;").map_err(|error| error.to_string())?;
        fs::write(root.join("_public.scss"), "$brand: red;").map_err(|error| error.to_string())?;

        let view = build_sass_graph_view(Some(root.clone()), Some(entry.clone()))?;
        assert_eq!(view.module_edge_count, 3);
        assert_eq!(view.resolved_module_edge_count, 2);
        assert_eq!(view.unresolved_module_edge_count, 1);
        assert_eq!(view.visibility_filter_count, 1);
        assert!(
            view.edges
                .iter()
                .any(|edge| edge.namespace.as_deref() == Some("theme"))
        );
        assert!(view.edges.iter().any(|edge| {
            edge.visibility_filter_kind == Some("show") && edge.visibility_filter_names == ["brand"]
        }));
        assert!(view.capabilities.namespace_show_hide_filter_ready);
        fs::remove_dir_all(root).map_err(|error| error.to_string())?;
        Ok(())
    }

    fn temp_dir(label: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        std::env::temp_dir().join(format!("omena-{label}-{}-{nonce}", std::process::id()))
    }
}
