use crate::protocol::{file_uri_to_path, is_style_document_uri, normalize_path};
use crate::{LspShellState, LspTextDocumentState};
use omena_query::{
    OmenaQueryExternalSifInputV0, generate_omena_bridge_sif_for_resolved_style_path,
    resolve_omena_query_style_uri_for_specifier, summarize_omena_query_sass_module_sources,
};
use omena_sif::{read_omena_lock_json_v1, read_omena_sif_json_v1};
use std::{
    collections::{BTreeSet, VecDeque},
    fs,
    path::{Path, PathBuf},
};

pub(crate) fn refresh_external_sifs_for_state(state: &mut LspShellState) {
    let mut external_sifs = Vec::new();
    let mut covered = BTreeSet::new();

    for lockfile in workspace_lockfiles(state).iter() {
        state.external_sif_lock_read_count = state.external_sif_lock_read_count.saturating_add(1);
        if let Ok(lock_sifs) = read_lock_external_sifs(lockfile.as_path()) {
            extend_unique_external_sifs(&mut external_sifs, &mut covered, lock_sifs);
        }
    }

    let mut bridge_generation_count = 0usize;
    let (bridge_sifs, bridge_urls) = resolve_in_process_external_sifs_for_lsp(
        state.documents.values().map(AsRef::as_ref),
        &covered,
        &mut bridge_generation_count,
    );
    state.external_sif_bridge_generation_count = state
        .external_sif_bridge_generation_count
        .saturating_add(bridge_generation_count);
    extend_unique_external_sifs(&mut external_sifs, &mut covered, bridge_sifs);

    if state.resolution.external_sifs != external_sifs {
        state.resolution.external_sifs = external_sifs;
        invalidate_external_sif_dependents(state);
    }
    state.resolution.bridge_external_sif_urls = bridge_urls;
}

pub(crate) fn refresh_external_sifs_for_bridge_source_delta(
    state: &mut LspShellState,
    previous_sources: &[String],
    next_sources: &[String],
) {
    let previous_sources = previous_sources.iter().cloned().collect::<BTreeSet<_>>();
    let next_sources = next_sources.iter().cloned().collect::<BTreeSet<_>>();
    if previous_sources == next_sources {
        return;
    }

    let active_bridge_sources = active_bridge_sources_from_documents(state);
    let mut changed = false;
    let mut remove_urls = BTreeSet::new();
    for source in previous_sources.difference(&next_sources) {
        if active_bridge_sources.contains(source) {
            continue;
        }
        let mut ignored_generation_count = 0usize;
        collect_bridge_sif_urls_for_sources(
            std::iter::once(source.as_str()),
            &BTreeSet::new(),
            &mut ignored_generation_count,
        )
        .into_iter()
        .for_each(|url| {
            remove_urls.insert(url);
        });
    }

    if !remove_urls.is_empty() {
        let before_len = state.resolution.external_sifs.len();
        state.resolution.external_sifs.retain(|input| {
            !state
                .resolution
                .bridge_external_sif_urls
                .contains(input.canonical_url.as_str())
                || !remove_urls.contains(input.canonical_url.as_str())
        });
        state
            .resolution
            .bridge_external_sif_urls
            .retain(|url| !remove_urls.contains(url.as_str()));
        changed |= before_len != state.resolution.external_sifs.len();
    }

    let mut covered = covered_external_sif_urls(state.resolution.external_sifs.as_slice());
    let mut bridge_generation_count = 0usize;
    for source in next_sources.difference(&previous_sources) {
        if state
            .resolution
            .bridge_external_sif_urls
            .contains(source.as_str())
        {
            continue;
        }
        let (bridge_sifs, bridge_urls) = resolve_bridge_external_sifs_for_sources(
            std::iter::once(source.as_str()),
            &covered,
            &mut bridge_generation_count,
        );
        let before_len = state.resolution.external_sifs.len();
        extend_unique_external_sifs(
            &mut state.resolution.external_sifs,
            &mut covered,
            bridge_sifs,
        );
        state
            .resolution
            .bridge_external_sif_urls
            .extend(bridge_urls);
        changed |= before_len != state.resolution.external_sifs.len();
    }

    state.external_sif_bridge_generation_count = state
        .external_sif_bridge_generation_count
        .saturating_add(bridge_generation_count);
    if changed {
        invalidate_external_sif_dependents(state);
    }
}

fn workspace_lockfiles(state: &LspShellState) -> Vec<PathBuf> {
    let mut lockfiles = BTreeSet::new();
    for folder in state.workspace_runtime_registry.folder_snapshots() {
        let Some(root) = file_uri_to_path(folder.uri.as_str()).map(normalize_path) else {
            continue;
        };
        if let Some(lockfile) = discover_omena_lockfile_for_workspace_root(root.as_path()) {
            lockfiles.insert(lockfile);
        }
    }
    lockfiles.into_iter().collect()
}

fn discover_omena_lockfile_for_workspace_root(root: &Path) -> Option<PathBuf> {
    let mut current = Some(root);
    while let Some(directory) = current {
        let candidate = directory.join("omena.lock");
        if candidate.exists() {
            return Some(normalize_path(candidate));
        }
        current = directory.parent();
    }
    None
}

fn read_lock_external_sifs(lockfile: &Path) -> Result<Vec<OmenaQueryExternalSifInputV0>, String> {
    let lockfile_source = fs::read_to_string(lockfile)
        .map_err(|error| format!("failed to read {}: {error}", lockfile.display()))?;
    let lock = read_omena_lock_json_v1(lockfile_source.as_str())
        .map_err(|error| format!("failed to parse {}: {error}", lockfile.display()))?;
    lock.entries
        .iter()
        .map(|entry| {
            let sif_path = resolve_lock_relative_path(lockfile, entry.sif_path.as_str());
            let sif_json = fs::read_to_string(sif_path.as_path())
                .map_err(|error| format!("failed to read {}: {error}", sif_path.display()))?;
            let sif = read_omena_sif_json_v1(sif_json.as_str())
                .map_err(|error| format!("failed to parse SIF {}: {error}", sif_path.display()))?;
            if sif.canonical_url != entry.canonical_url {
                return Err(format!(
                    "lock entry {} points to SIF {} with canonicalUrl {}",
                    entry.canonical_url,
                    sif_path.display(),
                    sif.canonical_url
                ));
            }
            Ok(OmenaQueryExternalSifInputV0 {
                canonical_url: entry.canonical_url.clone(),
                sif,
            })
        })
        .collect()
}

fn resolve_lock_relative_path(lockfile: &Path, entry_path: &str) -> PathBuf {
    let path = PathBuf::from(entry_path);
    if path.is_absolute() {
        return normalize_path(path);
    }
    normalize_path(
        lockfile
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join(path),
    )
}

fn extend_unique_external_sifs(
    output: &mut Vec<OmenaQueryExternalSifInputV0>,
    covered: &mut BTreeSet<String>,
    candidates: Vec<OmenaQueryExternalSifInputV0>,
) {
    for candidate in candidates {
        if covered.insert(candidate.canonical_url.clone()) {
            covered.insert(candidate.sif.canonical_url.clone());
            output.push(candidate);
        }
    }
}

fn resolve_in_process_external_sifs_for_lsp<'a>(
    documents: impl Iterator<Item = &'a LspTextDocumentState>,
    existing_covered: &BTreeSet<String>,
    bridge_generation_count: &mut usize,
) -> (Vec<OmenaQueryExternalSifInputV0>, BTreeSet<String>) {
    let sources = documents
        .flat_map(|document| {
            if !is_style_document_uri(document.uri.as_str()) {
                return Vec::new();
            }
            let Some(module_sources) = summarize_omena_query_sass_module_sources(
                document.uri.as_str(),
                document.text.as_str(),
            ) else {
                return Vec::new();
            };
            module_sources
                .module_use_edges
                .iter()
                .map(|edge| edge.source.clone())
                .chain(module_sources.module_forward_sources)
                .filter(|source| source.starts_with("file://"))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    resolve_bridge_external_sifs_for_sources(
        sources.iter().map(String::as_str),
        existing_covered,
        bridge_generation_count,
    )
}

fn resolve_bridge_external_sifs_for_sources<'a>(
    sources: impl Iterator<Item = &'a str>,
    existing_covered: &BTreeSet<String>,
    bridge_generation_count: &mut usize,
) -> (Vec<OmenaQueryExternalSifInputV0>, BTreeSet<String>) {
    let mut covered = existing_covered.clone();
    let mut bridge_urls = BTreeSet::new();
    let mut resolved = Vec::new();
    let mut worklist = VecDeque::new();

    for edge_source in sources {
        if !edge_source.starts_with("file://") || !covered.insert(edge_source.to_string()) {
            continue;
        }
        bridge_urls.insert(edge_source.to_string());
        if let Ok(sif) = generate_omena_bridge_sif_for_resolved_style_path(edge_source) {
            *bridge_generation_count = (*bridge_generation_count).saturating_add(1);
            covered.insert(sif.canonical_url.clone());
            bridge_urls.insert(sif.canonical_url.clone());
            worklist.push_back(sif.clone());
            resolved.push(OmenaQueryExternalSifInputV0 {
                canonical_url: edge_source.to_string(),
                sif,
            });
        }
    }

    while let Some(sif) = worklist.pop_front() {
        let base_file_uri = sif.canonical_url.clone();
        for forward in &sif.exports.forwards {
            let specifier = forward.canonical_url.as_str();
            if specifier.starts_with("sass:")
                || specifier.starts_with("http://")
                || specifier.starts_with("https://")
            {
                continue;
            }
            let Some(child_url) = resolve_omena_query_style_uri_for_specifier(
                base_file_uri.as_str(),
                None,
                specifier,
            ) else {
                continue;
            };
            if !covered.insert(child_url.clone()) {
                continue;
            }
            bridge_urls.insert(child_url.clone());
            if let Ok(child) = generate_omena_bridge_sif_for_resolved_style_path(child_url.as_str())
            {
                *bridge_generation_count = (*bridge_generation_count).saturating_add(1);
                bridge_urls.insert(child.canonical_url.clone());
                worklist.push_back(child.clone());
                resolved.push(OmenaQueryExternalSifInputV0 {
                    canonical_url: child_url,
                    sif: child,
                });
            }
        }
    }

    (resolved, bridge_urls)
}

fn collect_bridge_sif_urls_for_sources<'a>(
    sources: impl Iterator<Item = &'a str>,
    existing_covered: &BTreeSet<String>,
    bridge_generation_count: &mut usize,
) -> BTreeSet<String> {
    let (_, bridge_urls) = resolve_bridge_external_sifs_for_sources(
        sources,
        existing_covered,
        bridge_generation_count,
    );
    bridge_urls
}

fn active_bridge_sources_from_documents(state: &LspShellState) -> BTreeSet<String> {
    let mut sources = BTreeSet::new();
    for document in state.documents.values() {
        let Some(summary) = document.style_summary.as_ref() else {
            continue;
        };
        let edge_sources = summary
            .sass_module_use_sources
            .iter()
            .map(String::as_str)
            .chain(
                summary
                    .sass_module_forward_sources
                    .iter()
                    .map(String::as_str),
            );
        for edge_source in edge_sources {
            if edge_source.starts_with("file://") {
                sources.insert(edge_source.to_string());
            }
        }
    }
    sources
}

fn covered_external_sif_urls(inputs: &[OmenaQueryExternalSifInputV0]) -> BTreeSet<String> {
    inputs
        .iter()
        .flat_map(|input| [input.canonical_url.clone(), input.sif.canonical_url.clone()])
        .collect()
}

fn invalidate_external_sif_dependents(state: &mut LspShellState) {
    state.workspace_occurrence_index_memo.replace(None);
    if let Ok(mut memo) = state.cascade_narrowing_substrate_memo.lock() {
        *memo = None;
    }
}
