use crate::protocol::{
    file_uri_to_path, is_style_document_uri, normalize_path, path_to_file_uri, style_language_label,
};
use crate::source_document_cache::{
    load_source_document_index_sidecar, source_document_text_hash,
    store_source_document_index_sidecar,
};
use crate::{
    LspShellState, LspTextDocumentState, LspWorkspaceFolderState, lsp_text_document_state,
    lsp_text_document_state_with_source_syntax_index,
};
use omena_query::{OmenaQueryStyleResolutionInputsV0, StyleLanguage};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
    time::Instant,
};

const WORKSPACE_INDEX_FILE_LIMIT: usize = 512;
const WORKSPACE_STYLE_INDEX_DIR_LIMIT: usize = 2048;
const WORKSPACE_STYLE_INDEX_TIME_BUDGET_MS: u128 = 50;

pub(crate) fn index_workspace_style_files(state: &mut LspShellState) {
    let mut budget = WorkspaceStyleIndexBudget::with_defaults();
    index_workspace_style_files_with_budget(state, &mut budget);
    crate::admit_foreign_style_dependencies_for_indexed_style_documents(state);
    crate::refresh_external_sifs_for_state(state);
}

#[derive(Debug, Clone)]
pub struct LspWorkspaceIndexJobV0 {
    pub revision: u64,
    pub progress_token: Option<String>,
    pub folders: Vec<LspWorkspaceFolderState>,
    pub resolution_inputs_by_workspace_uri: BTreeMap<String, OmenaQueryStyleResolutionInputsV0>,
    pub indexed_document_hashes_by_uri: BTreeMap<String, String>,
    pub open_document_uris: BTreeSet<String>,
    pub pending_file_uris: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct LspWorkspaceIndexResultV0 {
    pub revision: u64,
    pub progress_token: Option<String>,
    pub documents: Vec<LspTextDocumentState>,
    pub pending_file_uris: Vec<String>,
    pub indexed_count: usize,
    pub pending_file_count: usize,
    pub exhausted: bool,
}

pub fn prepare_background_workspace_index_job(state: &mut LspShellState) -> LspWorkspaceIndexJobV0 {
    state.workspace_index_revision = state.workspace_index_revision.saturating_add(1);
    LspWorkspaceIndexJobV0 {
        revision: state.workspace_index_revision,
        progress_token: None,
        folders: state.workspace_runtime_registry.folder_snapshots(),
        resolution_inputs_by_workspace_uri: state
            .resolution
            .workspace_style_resolution_inputs
            .clone(),
        indexed_document_hashes_by_uri: state
            .documents
            .values()
            .map(|document| (document.uri.clone(), document.text_hash.clone()))
            .collect(),
        open_document_uris: state
            .open_document_uris
            .iter()
            .flat_map(|file_id| {
                [
                    state
                        .document_for_file_id(*file_id)
                        .map(|document| document.uri.clone()),
                    state
                        .document_storage_uri_for_file_id(*file_id)
                        .map(ToString::to_string),
                ]
                .into_iter()
                .flatten()
            })
            .collect(),
        pending_file_uris: Vec::new(),
    }
}

pub fn prepare_background_workspace_index_continuation_job(
    state: &mut LspShellState,
    pending_file_uris: Vec<String>,
) -> LspWorkspaceIndexJobV0 {
    let mut job = prepare_background_workspace_index_job(state);
    job.pending_file_uris = pending_file_uris;
    job
}

pub fn collect_background_workspace_index(
    job: LspWorkspaceIndexJobV0,
) -> LspWorkspaceIndexResultV0 {
    let mut documents = Vec::new();
    let candidate_uris = if job.pending_file_uris.is_empty() {
        collect_workspace_index_candidate_uris(&job)
    } else {
        job.pending_file_uris.clone()
    };
    let mut budget = WorkspaceStyleIndexBudget::with_defaults();
    let pending_file_uris = collect_workspace_index_documents_from_candidates(
        &job,
        candidate_uris,
        &mut budget,
        &mut documents,
    );
    let indexed_count = documents.len();
    let pending_file_count = pending_file_uris.len();
    LspWorkspaceIndexResultV0 {
        revision: job.revision,
        progress_token: job.progress_token,
        documents,
        pending_file_uris,
        indexed_count,
        pending_file_count,
        exhausted: budget.exhausted,
    }
}

pub fn apply_background_workspace_index_result(
    state: &mut LspShellState,
    result: LspWorkspaceIndexResultV0,
) -> bool {
    if result.revision != state.workspace_index_revision {
        return false;
    }
    let mut indexed_style_uris = Vec::new();
    for document in result.documents {
        if state.has_open_document_uri(document.uri.as_str()) {
            continue;
        }
        let Some(current_owner_uri) = state
            .workspace_runtime_registry
            .resolve_owner_uri(document.uri.as_str())
        else {
            continue;
        };
        if document.workspace_folder_uri.as_deref() != Some(current_owner_uri.as_str()) {
            continue;
        }
        let uri = document.uri.clone();
        if is_style_document_uri(uri.as_str()) {
            indexed_style_uris.push(uri.clone());
        }
        state.insert_document(uri.as_str(), document);
    }
    crate::admit_foreign_style_dependencies_for_style_uris(state, indexed_style_uris);
    crate::refresh_external_sifs_for_state(state);
    state.workspace_index_pending_file_count = result.pending_file_count;
    if result.exhausted {
        state.workspace_style_index_exhausted_count += 1;
    }
    true
}

pub(crate) fn index_workspace_style_files_with_budget(
    state: &mut LspShellState,
    budget: &mut WorkspaceStyleIndexBudget,
) {
    let folders = state.workspace_runtime_registry.folder_snapshots();
    for folder in folders {
        if budget.should_stop() {
            break;
        }
        let Some(path) = file_uri_to_path(folder.uri.as_str()) else {
            continue;
        };
        index_workspace_style_files_from_dir(state, folder.uri.as_str(), path.as_path(), budget);
    }
    if budget.exhausted {
        state.workspace_style_index_exhausted_count += 1;
    }
}

fn index_workspace_style_files_from_dir(
    state: &mut LspShellState,
    workspace_folder_uri: &str,
    dir: &Path,
    budget: &mut WorkspaceStyleIndexBudget,
) {
    if budget.should_stop() || should_skip_workspace_index_dir(dir) {
        return;
    }
    budget.consume_dir();
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    let mut entries = entries.flatten().collect::<Vec<_>>();
    entries.sort_by_key(|entry| entry.path());
    for entry in entries {
        if budget.should_stop() {
            return;
        }
        let path = entry.path();
        if path.is_dir() {
            index_workspace_style_files_from_dir(
                state,
                workspace_folder_uri,
                path.as_path(),
                budget,
            );
            continue;
        }
        let Some(language_id) = workspace_index_language_id_for_path(path.as_path()) else {
            continue;
        };
        let uri = path_to_file_uri(path.as_path());
        if state.contains_document_uri(uri.as_str()) {
            continue;
        }
        let Ok(text) = fs::read_to_string(path.as_path()) else {
            continue;
        };
        let workspace_owner_uri = state
            .workspace_runtime_registry
            .resolve_owner_uri(uri.as_str())
            .unwrap_or_else(|| workspace_folder_uri.to_string());
        let resolution_inputs = state
            .resolution
            .workspace_style_resolution_inputs
            .get(workspace_owner_uri.as_str())
            .cloned()
            .unwrap_or_default();
        state.insert_document(
            uri.as_str(),
            lsp_text_document_state(
                uri.clone(),
                Some(workspace_owner_uri),
                language_id,
                0,
                text,
                &resolution_inputs,
            ),
        );
        budget.consume_indexed_file();
    }
}

fn collect_workspace_index_candidate_uris(job: &LspWorkspaceIndexJobV0) -> Vec<String> {
    let mut uris = Vec::new();
    for folder in &job.folders {
        let Some(path) = file_uri_to_path(folder.uri.as_str()) else {
            continue;
        };
        collect_workspace_index_candidate_uris_from_dir(job, path.as_path(), uris.as_mut());
    }
    uris.sort();
    uris.dedup();
    uris
}

fn collect_workspace_index_candidate_uris_from_dir(
    job: &LspWorkspaceIndexJobV0,
    dir: &Path,
    uris: &mut Vec<String>,
) {
    if should_skip_workspace_index_dir(dir) {
        return;
    }
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    let mut entries = entries.flatten().collect::<Vec<_>>();
    entries.sort_by_key(|entry| entry.path());
    for entry in entries {
        let path = entry.path();
        if path.is_dir() {
            collect_workspace_index_candidate_uris_from_dir(job, path.as_path(), uris);
            continue;
        }
        if workspace_index_language_id_for_path(path.as_path()).is_none() {
            continue;
        }
        let uri = path_to_file_uri(path.as_path());
        if job.open_document_uris.contains(uri.as_str())
            || job
                .indexed_document_hashes_by_uri
                .contains_key(uri.as_str())
        {
            continue;
        }
        uris.push(uri);
    }
}

fn collect_workspace_index_documents_from_candidates(
    job: &LspWorkspaceIndexJobV0,
    candidate_uris: Vec<String>,
    budget: &mut WorkspaceStyleIndexBudget,
    documents: &mut Vec<LspTextDocumentState>,
) -> Vec<String> {
    let mut pending_file_uris = Vec::new();
    for uri in candidate_uris {
        if job.open_document_uris.contains(uri.as_str())
            || job
                .indexed_document_hashes_by_uri
                .contains_key(uri.as_str())
        {
            continue;
        }
        if budget.should_stop() {
            pending_file_uris.push(uri);
            continue;
        }
        let Some(path) = file_uri_to_path(uri.as_str()) else {
            continue;
        };
        let Some(language_id) = workspace_index_language_id_for_path(path.as_path()) else {
            continue;
        };
        let Ok(text) = fs::read_to_string(path.as_path()) else {
            continue;
        };
        let text_hash = source_document_text_hash(text.as_str());
        let workspace_owner_uri = resolve_background_workspace_owner_uri(job, uri.as_str())
            .or_else(|| {
                job.folders
                    .iter()
                    .find(|folder| uri.starts_with(folder.uri.as_str()))
                    .map(|folder| folder.uri.clone())
            });
        let resolution_inputs = workspace_owner_uri
            .as_ref()
            .and_then(|owner_uri| {
                job.resolution_inputs_by_workspace_uri
                    .get(owner_uri.as_str())
            })
            .cloned()
            .unwrap_or_default();
        let document = if !is_style_document_uri(uri.as_str())
            && let Some(sidecar) = load_source_document_index_sidecar(
                workspace_owner_uri.as_deref(),
                uri.as_str(),
                language_id.as_str(),
                text_hash.as_str(),
                &resolution_inputs,
            ) {
            lsp_text_document_state_with_source_syntax_index(
                uri,
                workspace_owner_uri,
                language_id,
                0,
                text,
                sidecar.source_syntax_index,
                sidecar.has_unresolved_style_import,
            )
        } else {
            let document = lsp_text_document_state(
                uri,
                workspace_owner_uri,
                language_id,
                0,
                text,
                &resolution_inputs,
            );
            if !is_style_document_uri(document.uri.as_str()) {
                store_source_document_index_sidecar(
                    document.workspace_folder_uri.as_deref(),
                    document.uri.as_str(),
                    document.language_id.as_str(),
                    document.text_hash.as_str(),
                    &resolution_inputs,
                    &document.source_syntax_index,
                    document.has_unresolved_style_import,
                );
            }
            document
        };
        documents.push(document);
        budget.consume_indexed_file();
    }
    pending_file_uris
}

fn resolve_background_workspace_owner_uri(
    job: &LspWorkspaceIndexJobV0,
    document_uri: &str,
) -> Option<String> {
    let document_path = file_uri_to_path(document_uri).map(normalize_path);
    job.folders
        .iter()
        .filter_map(|folder| {
            background_workspace_owner_score(folder, document_uri, document_path.as_deref())
                .map(|score| (score, folder.uri.clone()))
        })
        .max_by_key(|(score, _)| *score)
        .map(|(_, uri)| uri)
}

fn background_workspace_owner_score(
    folder: &LspWorkspaceFolderState,
    document_uri: &str,
    document_path: Option<&Path>,
) -> Option<(u8, usize, usize)> {
    if let (Some(root_path), Some(document_path)) = (
        file_uri_to_path(folder.uri.as_str()).map(normalize_path),
        document_path,
    ) && !root_path.as_os_str().is_empty()
        && (document_path == root_path || document_path.starts_with(root_path.as_path()))
    {
        return Some((1, root_path.components().count(), folder.uri.len()));
    }

    if document_uri == folder.uri
        || document_uri
            .strip_prefix(folder.uri.as_str())
            .is_some_and(|suffix| suffix.starts_with('/'))
    {
        return Some((0, 0, folder.uri.len()));
    }

    None
}

pub(crate) struct WorkspaceStyleIndexBudget {
    remaining_files: usize,
    remaining_dirs: usize,
    started_at: Instant,
    time_budget_ms: u128,
    pub(crate) exhausted: bool,
}

impl WorkspaceStyleIndexBudget {
    pub(crate) fn with_defaults() -> Self {
        Self::with_limits(
            WORKSPACE_INDEX_FILE_LIMIT,
            WORKSPACE_STYLE_INDEX_DIR_LIMIT,
            WORKSPACE_STYLE_INDEX_TIME_BUDGET_MS,
        )
    }

    pub(crate) fn with_limits(
        remaining_files: usize,
        remaining_dirs: usize,
        time_budget_ms: u128,
    ) -> Self {
        Self {
            remaining_files,
            remaining_dirs,
            started_at: Instant::now(),
            time_budget_ms,
            exhausted: false,
        }
    }

    fn should_stop(&mut self) -> bool {
        if self.remaining_files == 0
            || self.remaining_dirs == 0
            || self.started_at.elapsed().as_millis() >= self.time_budget_ms
        {
            self.exhausted = true;
            return true;
        }
        false
    }

    fn consume_dir(&mut self) {
        self.remaining_dirs = self.remaining_dirs.saturating_sub(1);
    }

    fn consume_indexed_file(&mut self) {
        self.remaining_files = self.remaining_files.saturating_sub(1);
    }
}

fn should_skip_workspace_index_dir(dir: &Path) -> bool {
    dir.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| {
            matches!(
                name,
                ".cache"
                    | ".git"
                    | ".next"
                    | ".turbo"
                    | "build"
                    | "coverage"
                    | "node_modules"
                    | "out"
                    | "target"
            )
        })
}

fn workspace_index_language_id_for_path(path: &Path) -> Option<String> {
    if let Some(language) = StyleLanguage::from_module_path(path.to_string_lossy().as_ref()) {
        return Some(style_language_label(language).to_string());
    }
    source_language_id_for_path(path).map(str::to_string)
}

pub(crate) fn workspace_index_language_id_for_uri(uri: &str) -> Option<String> {
    let path = file_uri_to_path(uri)?;
    workspace_index_language_id_for_path(path.as_path())
}

fn source_language_id_for_path(path: &Path) -> Option<&'static str> {
    let file_name = path.file_name()?.to_str()?.to_ascii_lowercase();
    if file_name.ends_with(".d.ts") {
        return Some("typescript");
    }
    if file_name.ends_with(".html.eex") {
        return Some("html-eex");
    }
    match path.extension()?.to_str()?.to_ascii_lowercase().as_str() {
        "ts" | "mts" | "cts" => Some("typescript"),
        "tsx" => Some("typescriptreact"),
        "js" | "mjs" | "cjs" => Some("javascript"),
        "jsx" => Some("javascriptreact"),
        "vue" => Some("vue"),
        "html" => Some("html"),
        "svelte" => Some("svelte"),
        "astro" => Some("astro"),
        "md" => Some("markdown"),
        "mdx" => Some("mdx"),
        "liquid" => Some("liquid"),
        "twig" => Some("twig"),
        "njk" => Some("nunjucks"),
        "hbs" => Some("handlebars"),
        "erb" => Some("erb"),
        "ejs" => Some("ejs"),
        "heex" => Some("heex"),
        _ => None,
    }
}
