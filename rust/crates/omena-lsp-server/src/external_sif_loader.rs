use crate::protocol::{file_uri_to_path, is_style_document_uri, normalize_path};
use crate::{LspShellState, LspTextDocumentState};
use omena_query::{
    OmenaQueryBridgeExternalSifResolutionV0, OmenaQueryExternalSifInputV0,
    OmenaQueryStyleResolutionInputsV0, OmenaQueryStyleSourceInputV0,
    resolve_omena_query_bridge_external_sifs_for_seed_pairs,
    resolve_omena_query_bridge_external_sifs_for_style_sources,
};
use omena_sif::{read_omena_lock_json_v1, read_omena_sif_json_v1};
use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone)]
pub struct LspExternalSifRefreshDocumentV0 {
    pub uri: String,
    pub workspace_folder_uri: Option<String>,
    pub text: String,
}

#[derive(Debug, Clone)]
pub struct LspExternalSifRefreshJobV0 {
    pub revision: u64,
    pub lockfiles: Vec<PathBuf>,
    pub documents: Vec<LspExternalSifRefreshDocumentV0>,
    pub package_manifests: Vec<omena_query::OmenaQueryStylePackageManifestV0>,
    pub resolution_inputs_by_workspace_uri:
        std::collections::BTreeMap<String, OmenaQueryStyleResolutionInputsV0>,
}

#[derive(Debug, Clone)]
pub struct LspExternalSifRefreshResultV0 {
    pub revision: u64,
    pub external_sifs: Vec<OmenaQueryExternalSifInputV0>,
    pub bridge_external_sif_urls: BTreeSet<String>,
    pub lock_read_count: usize,
    pub bridge_generation_count: usize,
}

pub(crate) fn refresh_external_sifs_for_state(state: &mut LspShellState) {
    if state.external_sif_refresh_deferred {
        mark_external_sif_refresh_dirty(state);
        return;
    }
    refresh_external_sifs_for_state_immediate(state);
}

fn refresh_external_sifs_for_state_immediate(state: &mut LspShellState) {
    let mut external_sifs = Vec::new();
    let mut covered = BTreeSet::new();

    for lockfile in workspace_lockfiles(state).iter() {
        state.external_sif_lock_read_count = state.external_sif_lock_read_count.saturating_add(1);
        if let Ok(lock_sifs) = read_lock_external_sifs(lockfile.as_path()) {
            extend_unique_external_sifs(&mut external_sifs, &mut covered, lock_sifs);
        }
    }

    let bridge_result = resolve_in_process_external_sifs_for_lsp(state, &covered);
    state.external_sif_bridge_generation_count = state
        .external_sif_bridge_generation_count
        .saturating_add(bridge_result.generation_count);
    extend_unique_external_sifs(
        &mut external_sifs,
        &mut covered,
        bridge_result.external_sifs,
    );

    if state.resolution.external_sifs != external_sifs {
        state.resolution.external_sifs = external_sifs;
        invalidate_external_sif_dependents(state);
    }
    state.resolution.bridge_external_sif_urls = bridge_result.bridge_urls.into_iter().collect();
}

pub(crate) fn refresh_external_sifs_for_bridge_source_delta(
    state: &mut LspShellState,
    previous_sources: &[String],
    next_sources: &[String],
) {
    if state.external_sif_refresh_deferred {
        mark_external_sif_refresh_dirty(state);
        return;
    }
    let previous_sources = previous_sources.iter().cloned().collect::<BTreeSet<_>>();
    let next_sources = next_sources.iter().cloned().collect::<BTreeSet<_>>();
    if previous_sources == next_sources {
        return;
    }
    if previous_sources
        .iter()
        .chain(next_sources.iter())
        .any(|source| !source.starts_with("file://"))
    {
        refresh_external_sifs_for_state(state);
        return;
    }

    let active_bridge_sources = active_bridge_sources_from_documents(state);
    let mut changed = false;
    let mut remove_urls = BTreeSet::new();
    for source in previous_sources.difference(&next_sources) {
        if active_bridge_sources.contains(source) {
            continue;
        }
        collect_bridge_sif_urls_for_sources(std::iter::once(source.as_str()), &BTreeSet::new())
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
    for source in next_sources.difference(&previous_sources) {
        if state
            .resolution
            .bridge_external_sif_urls
            .contains(source.as_str())
        {
            continue;
        }
        let bridge_result =
            resolve_bridge_external_sifs_for_sources(std::iter::once(source.as_str()), &covered);
        let before_len = state.resolution.external_sifs.len();
        extend_unique_external_sifs(
            &mut state.resolution.external_sifs,
            &mut covered,
            bridge_result.external_sifs,
        );
        state
            .resolution
            .bridge_external_sif_urls
            .extend(bridge_result.bridge_urls);
        changed |= before_len != state.resolution.external_sifs.len();
        state.external_sif_bridge_generation_count = state
            .external_sif_bridge_generation_count
            .saturating_add(bridge_result.generation_count);
    }

    if changed {
        invalidate_external_sif_dependents(state);
    }
}

pub(crate) fn bridge_sources_for_style_uris(
    state: &LspShellState,
    style_uris: &[String],
) -> Vec<String> {
    let mut sources = BTreeSet::new();
    for uri in style_uris {
        let Some(document) = state.document(uri.as_str()) else {
            continue;
        };
        collect_bridge_sources_from_style_document(document, &mut sources);
    }
    sources.into_iter().collect()
}

pub fn enable_deferred_external_sif_refresh(state: &mut LspShellState) {
    state.external_sif_refresh_deferred = true;
    mark_external_sif_refresh_dirty(state);
}

pub fn prepare_deferred_external_sif_refresh_job(
    state: &mut LspShellState,
) -> Option<LspExternalSifRefreshJobV0> {
    if !state.external_sif_refresh_deferred || !state.external_sif_refresh_dirty {
        return None;
    }
    state.external_sif_refresh_dirty = false;
    Some(LspExternalSifRefreshJobV0 {
        revision: state.external_sif_refresh_revision,
        lockfiles: workspace_lockfiles(state),
        documents: state
            .documents
            .values()
            .map(AsRef::as_ref)
            .filter(|document| is_style_document_uri(document.uri.as_str()))
            .map(|document| LspExternalSifRefreshDocumentV0 {
                uri: document.uri.clone(),
                workspace_folder_uri: document.workspace_folder_uri.clone(),
                text: document.text.clone(),
            })
            .collect(),
        package_manifests: state.resolution.package_manifests.clone(),
        resolution_inputs_by_workspace_uri: state
            .resolution
            .workspace_style_resolution_inputs
            .clone(),
    })
}

pub fn collect_deferred_external_sif_refresh(
    job: LspExternalSifRefreshJobV0,
) -> LspExternalSifRefreshResultV0 {
    let mut external_sifs = Vec::new();
    let mut covered = BTreeSet::new();
    let mut lock_read_count = 0usize;

    for lockfile in job.lockfiles.iter() {
        lock_read_count = lock_read_count.saturating_add(1);
        if let Ok(lock_sifs) = read_lock_external_sifs(lockfile.as_path()) {
            extend_unique_external_sifs(&mut external_sifs, &mut covered, lock_sifs);
        }
    }

    let bridge_result = resolve_external_sifs_for_refresh_documents(
        job.documents.as_slice(),
        external_sifs.as_slice(),
        job.package_manifests.as_slice(),
        &job.resolution_inputs_by_workspace_uri,
    );
    extend_unique_external_sifs(
        &mut external_sifs,
        &mut covered,
        bridge_result.external_sifs,
    );

    LspExternalSifRefreshResultV0 {
        revision: job.revision,
        external_sifs,
        bridge_external_sif_urls: bridge_result.bridge_urls.into_iter().collect(),
        lock_read_count,
        bridge_generation_count: bridge_result.generation_count,
    }
}

pub fn apply_deferred_external_sif_refresh_result(
    state: &mut LspShellState,
    result: LspExternalSifRefreshResultV0,
) -> bool {
    if result.revision != state.external_sif_refresh_revision {
        return false;
    }
    state.external_sif_lock_read_count = state
        .external_sif_lock_read_count
        .saturating_add(result.lock_read_count);
    state.external_sif_bridge_generation_count = state
        .external_sif_bridge_generation_count
        .saturating_add(result.bridge_generation_count);
    let changed = state.resolution.external_sifs != result.external_sifs;
    if changed {
        state.resolution.external_sifs = result.external_sifs;
        invalidate_external_sif_dependents(state);
    }
    state.resolution.bridge_external_sif_urls = result.bridge_external_sif_urls;
    changed
}

fn mark_external_sif_refresh_dirty(state: &mut LspShellState) {
    state.external_sif_refresh_revision = state.external_sif_refresh_revision.saturating_add(1);
    state.external_sif_refresh_dirty = true;
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

fn resolve_in_process_external_sifs_for_lsp(
    state: &LspShellState,
    existing_covered: &BTreeSet<String>,
) -> OmenaQueryBridgeExternalSifResolutionV0 {
    let mut existing_inputs = state
        .resolution
        .external_sifs
        .iter()
        .filter(|input| {
            existing_covered.contains(input.canonical_url.as_str())
                || existing_covered.contains(input.sif.canonical_url.as_str())
        })
        .cloned()
        .collect::<Vec<_>>();
    let mut combined = OmenaQueryBridgeExternalSifResolutionV0::default();
    let mut bridge_urls = BTreeSet::new();

    for document in state.documents.values().map(AsRef::as_ref) {
        if !is_style_document_uri(document.uri.as_str()) {
            continue;
        }
        let source = OmenaQueryStyleSourceInputV0 {
            style_path: document.uri.clone(),
            style_source: document.text.clone(),
        };
        let resolution_inputs =
            resolution_inputs_for_document(state, document.workspace_folder_uri.as_deref());
        let result = resolve_omena_query_bridge_external_sifs_for_style_sources(
            std::slice::from_ref(&source),
            existing_inputs.as_slice(),
            &resolution_inputs,
        );
        combined.generation_count = combined
            .generation_count
            .saturating_add(result.generation_count);
        bridge_urls.extend(result.bridge_urls);
        for external_sif in result.external_sifs {
            existing_inputs.push(external_sif.clone());
            combined.external_sifs.push(external_sif);
        }
    }

    combined.bridge_urls = bridge_urls.into_iter().collect();
    combined
}

fn resolve_external_sifs_for_refresh_documents(
    documents: &[LspExternalSifRefreshDocumentV0],
    existing_external_sifs: &[OmenaQueryExternalSifInputV0],
    package_manifests: &[omena_query::OmenaQueryStylePackageManifestV0],
    resolution_inputs_by_workspace_uri: &std::collections::BTreeMap<
        String,
        OmenaQueryStyleResolutionInputsV0,
    >,
) -> OmenaQueryBridgeExternalSifResolutionV0 {
    let mut existing_inputs = existing_external_sifs.to_vec();
    let mut combined = OmenaQueryBridgeExternalSifResolutionV0::default();
    let mut bridge_urls = BTreeSet::new();

    for document in documents {
        let source = OmenaQueryStyleSourceInputV0 {
            style_path: document.uri.clone(),
            style_source: document.text.clone(),
        };
        let resolution_inputs = document
            .workspace_folder_uri
            .as_deref()
            .and_then(|uri| resolution_inputs_by_workspace_uri.get(uri))
            .cloned()
            .unwrap_or_else(|| OmenaQueryStyleResolutionInputsV0 {
                package_manifests: package_manifests.to_vec(),
                ..OmenaQueryStyleResolutionInputsV0::default()
            });
        let result = resolve_omena_query_bridge_external_sifs_for_style_sources(
            std::slice::from_ref(&source),
            existing_inputs.as_slice(),
            &resolution_inputs,
        );
        combined.generation_count = combined
            .generation_count
            .saturating_add(result.generation_count);
        bridge_urls.extend(result.bridge_urls);
        for external_sif in result.external_sifs {
            existing_inputs.push(external_sif.clone());
            combined.external_sifs.push(external_sif);
        }
    }

    combined.bridge_urls = bridge_urls.into_iter().collect();
    combined
}

fn resolve_bridge_external_sifs_for_sources<'a>(
    sources: impl Iterator<Item = &'a str>,
    existing_covered: &BTreeSet<String>,
) -> OmenaQueryBridgeExternalSifResolutionV0 {
    resolve_omena_query_bridge_external_sifs_for_seed_pairs(
        sources
            .filter(|source| source.starts_with("file://") && !existing_covered.contains(*source))
            .map(|source| (source.to_string(), source.to_string())),
        &[],
        &OmenaQueryStyleResolutionInputsV0::default(),
    )
}

fn collect_bridge_sif_urls_for_sources<'a>(
    sources: impl Iterator<Item = &'a str>,
    existing_covered: &BTreeSet<String>,
) -> BTreeSet<String> {
    resolve_bridge_external_sifs_for_sources(sources, existing_covered)
        .bridge_urls
        .into_iter()
        .collect()
}

fn resolution_inputs_for_document(
    state: &LspShellState,
    workspace_folder_uri: Option<&str>,
) -> OmenaQueryStyleResolutionInputsV0 {
    workspace_folder_uri
        .and_then(|uri| {
            state
                .resolution
                .workspace_style_resolution_inputs
                .get(uri)
                .cloned()
        })
        .unwrap_or_else(|| OmenaQueryStyleResolutionInputsV0 {
            package_manifests: state.resolution.package_manifests.clone(),
            ..OmenaQueryStyleResolutionInputsV0::default()
        })
}

fn active_bridge_sources_from_documents(state: &LspShellState) -> BTreeSet<String> {
    let mut sources = BTreeSet::new();
    for document in state.documents.values() {
        collect_bridge_sources_from_style_document(document, &mut sources);
    }
    sources
}

fn collect_bridge_sources_from_style_document(
    document: &LspTextDocumentState,
    sources: &mut BTreeSet<String>,
) {
    let Some(summary) = document.style_summary.as_ref() else {
        return;
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
