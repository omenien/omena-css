use crate::protocol::{file_uri_to_path, path_to_file_uri, style_language_label};
use crate::{LspShellState, lsp_text_document_state};
use omena_query::StyleLanguage;
use std::{fs, path::Path, time::Instant};

const WORKSPACE_INDEX_FILE_LIMIT: usize = 512;
const WORKSPACE_STYLE_INDEX_DIR_LIMIT: usize = 2048;
const WORKSPACE_STYLE_INDEX_TIME_BUDGET_MS: u128 = 50;

pub(crate) fn index_workspace_style_files(state: &mut LspShellState) {
    let mut budget = WorkspaceStyleIndexBudget::with_defaults();
    index_workspace_style_files_with_budget(state, &mut budget);
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
    for entry in entries.flatten() {
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
