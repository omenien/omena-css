use std::{fs, path::PathBuf};

use omena_query::{
    OmenaQuerySassModuleCrossFileResolutionCapabilitiesV0, OmenaQuerySassModuleCycleV0,
    OmenaQuerySassModuleEdgeResolutionV0, OmenaQuerySassModuleGraphClosureEdgeV0,
    summarize_omena_query_sass_module_cross_file_resolution_for_workspace,
    summarize_omena_query_sass_unsupported_ledger_view_v0,
};
use omena_sif::{
    OmenaSifStructuralChangeKindV0, read_omena_sif_json_v1, summarize_omena_sif_structural_diff_v0,
};
use serde::Serialize;

use crate::{
    commands::SassCommand,
    config::find_omena_config_for_path,
    io::{read_package_manifests, read_source, read_style_sources},
    lint::discover_workspace_files,
    output::{CliOutputMetadataV0, print_json},
    paths::path_string,
};

mod compile;

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
        SassCommand::Diff { old, new, json } => sass_diff(old, new, json),
        SassCommand::Compile {
            entry,
            output,
            json,
        } => compile::sass_compile(entry, output, json),
        SassCommand::Unsupported { json } => sass_unsupported(json),
    }
}

fn sass_unsupported(json: bool) -> Result<(), String> {
    let view = summarize_omena_query_sass_unsupported_ledger_view_v0().map_err(|error| {
        format!("failed to read the canonical Sass unsupported ledger: {error}")
    })?;
    if json {
        print_json(
            CliOutputMetadataV0::new("omena-cli.sass.unsupported"),
            &view,
        )?;
    } else {
        println!(
            "Unsupported Sass evaluation sites: {} total, {} linked, {} named gaps",
            view.surface_record_count, view.linked_site_count, view.named_gap_site_count
        );
        for record in &view.records {
            let line = record.current_line.unwrap_or(record.ledger_line_hint);
            let coverage = if record.linked_fixture_ids.is_empty() {
                record.gap.as_deref().unwrap_or("coverage gap is unnamed")
            } else {
                "linked imported fixture"
            };
            println!("{}:{line}: {} - {coverage}", record.file, record.reason);
        }
    }
    if !view.summary_view_ready {
        return Err("canonical Sass unsupported ledger projection is out of date".to_string());
    }
    Ok(())
}

fn sass_diff(old_path: PathBuf, new_path: PathBuf, json: bool) -> Result<(), String> {
    let old_source = read_source(old_path.as_path())?;
    let new_source = read_source(new_path.as_path())?;
    let old = read_omena_sif_json_v1(old_source.as_str()).map_err(|error| {
        format!(
            "failed to parse previous SIF {}: {error}",
            path_string(old_path.as_path())
        )
    })?;
    let new = read_omena_sif_json_v1(new_source.as_str()).map_err(|error| {
        format!(
            "failed to parse candidate SIF {}: {error}",
            path_string(new_path.as_path())
        )
    })?;
    let report = summarize_omena_sif_structural_diff_v0(&old, &new)
        .map_err(|error| format!("failed to compare SIF exports: {error}"))?;
    if json {
        print_json(CliOutputMetadataV0::new("omena-cli.sass.diff"), &report)?;
    } else {
        println!(
            "Sass interface diff: {} breaking, {} added, {} unchanged",
            report.breaking_change_count, report.added_count, report.unchanged_count
        );
        for change in &report.changes {
            let polarity = match change.change_kind {
                OmenaSifStructuralChangeKindV0::Removed => "removed",
                OmenaSifStructuralChangeKindV0::Changed => "changed",
                OmenaSifStructuralChangeKindV0::VisibilityNarrowed => "visibility-narrowed",
                OmenaSifStructuralChangeKindV0::Added => "added",
            };
            println!("{polarity}: {:?} {}", change.export_kind, change.identity);
        }
    }
    if report.breaking {
        return Err(format!(
            "Sass interface compatibility check found {} breaking change(s)",
            report.breaking_change_count
        ));
    }
    Ok(())
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
    use omena_sif::{
        OmenaSifExportsV1, OmenaSifGeneratorV1, OmenaSifSourceSyntaxV1, OmenaSifSourceV1,
        OmenaSifV1, OmenaSifVariableExportV1, write_omena_sif_json_v1,
    };
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

    #[test]
    fn structural_diff_exit_tracks_breaking_changes() -> Result<(), String> {
        let root = temp_dir("sass-structural-diff");
        fs::create_dir_all(&root).map_err(|error| error.to_string())?;
        let old_path = root.join("old.sif.json");
        let same_path = root.join("same.sif.json");
        let breaking_path = root.join("breaking.sif.json");
        let old = sif_with_variables(&["brand"])?;
        let breaking = sif_with_variables(&[])?;
        fs::write(
            &old_path,
            write_omena_sif_json_v1(&old).map_err(|error| error.to_string())?,
        )
        .map_err(|error| error.to_string())?;
        fs::copy(&old_path, &same_path).map_err(|error| error.to_string())?;
        fs::write(
            &breaking_path,
            write_omena_sif_json_v1(&breaking).map_err(|error| error.to_string())?,
        )
        .map_err(|error| error.to_string())?;

        sass_diff(old_path.clone(), same_path, true)?;
        let Err(error) = sass_diff(old_path, breaking_path, true) else {
            return Err("removing an exported variable must produce a breaking exit".to_string());
        };
        assert!(error.contains("1 breaking change"));
        fs::remove_dir_all(root).map_err(|error| error.to_string())?;
        Ok(())
    }

    fn sif_with_variables(names: &[&str]) -> Result<OmenaSifV1, String> {
        OmenaSifV1::from_static_exports(
            "pkg:test",
            OmenaSifGeneratorV1 {
                name: "fixture".to_string(),
                version: "1".to_string(),
                toolchain_id: "fixture@1".to_string(),
            },
            OmenaSifSourceV1 {
                syntax: OmenaSifSourceSyntaxV1::Scss,
            },
            OmenaSifExportsV1 {
                variables: names
                    .iter()
                    .map(|name| OmenaSifVariableExportV1 {
                        name: (*name).to_string(),
                        defaulted: false,
                        value_repr: None,
                    })
                    .collect(),
                ..OmenaSifExportsV1::default()
            },
            Vec::new(),
            b"fixture",
        )
        .map_err(|error| error.to_string())
    }

    fn temp_dir(label: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        std::env::temp_dir().join(format!("omena-{label}-{}-{nonce}", std::process::id()))
    }
}
