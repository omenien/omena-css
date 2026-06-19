use std::{collections::BTreeSet, fs, path::Path};

use omena_query::load_omena_query_workspace_style_resolution_inputs;
use omena_sif::compute_omena_sif_leaf_hash_v1;
use serde_json::{Value, json};

use crate::{
    external_sif_loader::refresh_external_sifs_for_state,
    protocol::{file_uri_to_path, normalize_path},
    state::LspShellState,
};

pub(crate) fn initialize_workspace_folders(state: &mut LspShellState, params: Option<&Value>) {
    state.workspace_runtime_registry.clear();
    state.client_supports_work_done_progress = params
        .and_then(|value| value.pointer("/capabilities/window/workDoneProgress"))
        .and_then(Value::as_bool)
        .unwrap_or(false);
    if let Some(folders) = params
        .and_then(|value| value.get("workspaceFolders"))
        .and_then(Value::as_array)
    {
        for folder in folders {
            insert_workspace_folder(state, folder);
        }
        refresh_workspace_resolution_inputs(state);
        refresh_external_sifs_for_state(state);
        return;
    }

    if let Some(root_uri) = params
        .and_then(|value| value.get("rootUri"))
        .and_then(Value::as_str)
    {
        state
            .workspace_runtime_registry
            .insert(root_uri.to_string(), root_uri.to_string());
    }
    refresh_workspace_resolution_inputs(state);
    refresh_external_sifs_for_state(state);
}

pub(crate) fn refresh_workspace_resolution_inputs(state: &mut LspShellState) {
    let configured_package_manifests = state.resolution.package_manifests.clone();
    let workspace_uris = state
        .workspace_runtime_registry
        .folder_snapshots()
        .into_iter()
        .map(|folder| folder.uri)
        .collect::<BTreeSet<_>>();
    state
        .resolution
        .workspace_style_resolution_inputs
        .retain(|workspace_uri, _| workspace_uris.contains(workspace_uri));
    for workspace_uri in workspace_uris {
        let inputs = load_lsp_workspace_style_resolution_inputs(
            Some(workspace_uri.as_str()),
            configured_package_manifests.as_slice(),
        );
        state
            .resolution
            .workspace_style_resolution_inputs
            .insert(workspace_uri, inputs);
    }
}

pub(crate) fn refresh_workspace_resolution_inputs_for_uri(state: &mut LspShellState, uri: &str) {
    let Some(workspace_uri) = resolve_workspace_folder_uri(state, uri) else {
        return;
    };
    let inputs = load_lsp_workspace_style_resolution_inputs(
        Some(workspace_uri.as_str()),
        state.resolution.package_manifests.as_slice(),
    );
    state
        .resolution
        .workspace_style_resolution_inputs
        .insert(workspace_uri, inputs);
}

pub(crate) fn load_lsp_workspace_style_resolution_inputs(
    workspace_folder_uri: Option<&str>,
    configured_package_manifests: &[omena_query::OmenaQueryStylePackageManifestV0],
) -> omena_query::OmenaQueryStyleResolutionInputsV0 {
    let mut inputs = load_omena_query_workspace_style_resolution_inputs(
        workspace_folder_uri,
        configured_package_manifests,
    );
    inputs.external_sif_cache_fingerprint =
        workspace_folder_uri.and_then(external_sif_cache_fingerprint_for_workspace_uri);
    inputs
}

fn external_sif_cache_fingerprint_for_workspace_uri(workspace_folder_uri: &str) -> Option<String> {
    const METADATA_SCAN_LIMIT: usize = 2048;
    let root = file_uri_to_path(workspace_folder_uri)?;
    let root = normalize_path(root);
    let mut identities = Vec::new();
    for relative in [
        "omena.lock",
        "pnpm-lock.yaml",
        "package-lock.json",
        "yarn.lock",
        "bun.lock",
        "bun.lockb",
        "node_modules/.modules.yaml",
    ] {
        push_file_identity(&mut identities, root.join(relative).as_path());
    }
    collect_node_modules_package_link_identities(
        root.join("node_modules").as_path(),
        &mut identities,
        METADATA_SCAN_LIMIT,
    );
    if identities.is_empty() {
        return None;
    }
    let value = json!({
        "schemaVersion": "0",
        "product": "omena-lsp.external-sif-cache-freshness",
        "workspaceRoot": root.to_string_lossy(),
        "identities": identities,
    });
    let bytes = serde_json::to_vec(&value).ok()?;
    Some(
        compute_omena_sif_leaf_hash_v1(bytes.as_slice())
            .as_str()
            .to_string(),
    )
}

fn push_file_identity(output: &mut Vec<String>, path: &Path) {
    let Ok(metadata) = fs::symlink_metadata(path) else {
        return;
    };
    let modified = metadata
        .modified()
        .ok()
        .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|duration| format!("{}.{:09}", duration.as_secs(), duration.subsec_nanos()))
        .unwrap_or_else(|| "unknownMtime".to_string());
    let file_type = if metadata.file_type().is_symlink() {
        "symlink"
    } else if metadata.is_dir() {
        "dir"
    } else if metadata.is_file() {
        "file"
    } else {
        "other"
    };
    let target = if metadata.file_type().is_symlink() {
        fs::read_link(path)
            .ok()
            .map(|target| target.to_string_lossy().to_string())
            .unwrap_or_else(|| "unknownTarget".to_string())
    } else {
        String::new()
    };
    output.push(format!(
        "{}|{file_type}|len{}|mtime{modified}|target{target}",
        normalize_path(path.to_path_buf()).to_string_lossy(),
        metadata.len()
    ));
}

fn collect_node_modules_package_link_identities(
    node_modules: &Path,
    output: &mut Vec<String>,
    limit: usize,
) {
    let Ok(entries) = fs::read_dir(node_modules) else {
        return;
    };
    let mut seen = 0usize;
    for entry in entries.flatten() {
        if seen >= limit {
            break;
        }
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if name.starts_with('.') {
            continue;
        }
        if name.starts_with('@') && path.is_dir() {
            let Ok(scoped_entries) = fs::read_dir(path.as_path()) else {
                continue;
            };
            for scoped_entry in scoped_entries.flatten() {
                if seen >= limit {
                    break;
                }
                push_file_identity(output, scoped_entry.path().as_path());
                seen = seen.saturating_add(1);
            }
            continue;
        }
        push_file_identity(output, path.as_path());
        seen = seen.saturating_add(1);
    }
}

pub(crate) fn resolution_inputs_for_workspace_uri(
    state: &LspShellState,
    workspace_folder_uri: Option<&str>,
) -> omena_query::OmenaQueryStyleResolutionInputsV0 {
    workspace_folder_uri
        .and_then(|workspace_uri| {
            state
                .resolution
                .workspace_style_resolution_inputs
                .get(workspace_uri)
        })
        .cloned()
        .unwrap_or_else(|| {
            load_lsp_workspace_style_resolution_inputs(
                workspace_folder_uri,
                state.resolution.package_manifests.as_slice(),
            )
        })
}

pub(crate) fn insert_workspace_folder(state: &mut LspShellState, folder: &Value) {
    let Some(uri) = folder.get("uri").and_then(Value::as_str) else {
        return;
    };
    state.workspace_runtime_registry.insert(
        uri.to_string(),
        folder
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or(uri)
            .to_string(),
    );
}

pub(crate) fn refresh_document_workspace_owners(state: &mut LspShellState) {
    let workspace_runtime_registry = state.workspace_runtime_registry.clone();
    for document in state.documents.values_mut() {
        let document = std::sync::Arc::make_mut(document);
        document.workspace_folder_uri =
            workspace_runtime_registry.resolve_owner_uri(document.uri.as_str());
    }
}

pub(crate) fn resolve_workspace_folder_uri(
    state: &LspShellState,
    document_uri: &str,
) -> Option<String> {
    state
        .workspace_runtime_registry
        .resolve_owner_uri(document_uri)
}
