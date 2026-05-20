use crate::protocol::{file_uri_to_path, path_to_file_uri, style_language_label};
use crate::{LspShellState, lsp_text_document_state};
use omena_query::StyleLanguage;
use std::{fs, path::Path, time::Instant};

const WORKSPACE_STYLE_INDEX_LIMIT: usize = 512;
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
        if !is_indexable_style_path(path.as_path()) {
            continue;
        }
        let uri = path_to_file_uri(path.as_path());
        if state.documents.contains_key(uri.as_str()) {
            continue;
        }
        let Ok(text) = fs::read_to_string(path.as_path()) else {
            continue;
        };
        let workspace_owner_uri = state
            .workspace_runtime_registry
            .resolve_owner_uri(uri.as_str())
            .unwrap_or_else(|| workspace_folder_uri.to_string());
        state.documents.insert(
            uri.clone(),
            lsp_text_document_state(
                uri.clone(),
                Some(workspace_owner_uri),
                StyleLanguage::from_module_path(uri.as_str())
                    .map(style_language_label)
                    .unwrap_or("unknown")
                    .to_string(),
                0,
                text,
            ),
        );
        budget.consume_style_file();
    }
}

pub(crate) struct WorkspaceStyleIndexBudget {
    remaining_style_files: usize,
    remaining_dirs: usize,
    started_at: Instant,
    time_budget_ms: u128,
    pub(crate) exhausted: bool,
}

impl WorkspaceStyleIndexBudget {
    pub(crate) fn with_defaults() -> Self {
        Self::with_limits(
            WORKSPACE_STYLE_INDEX_LIMIT,
            WORKSPACE_STYLE_INDEX_DIR_LIMIT,
            WORKSPACE_STYLE_INDEX_TIME_BUDGET_MS,
        )
    }

    pub(crate) fn with_limits(
        remaining_style_files: usize,
        remaining_dirs: usize,
        time_budget_ms: u128,
    ) -> Self {
        Self {
            remaining_style_files,
            remaining_dirs,
            started_at: Instant::now(),
            time_budget_ms,
            exhausted: false,
        }
    }

    fn should_stop(&mut self) -> bool {
        if self.remaining_style_files == 0
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

    fn consume_style_file(&mut self) {
        self.remaining_style_files = self.remaining_style_files.saturating_sub(1);
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
                    | "dist"
                    | "node_modules"
                    | "out"
                    | "target"
            )
        })
}

fn is_indexable_style_path(path: &Path) -> bool {
    StyleLanguage::from_module_path(path.to_string_lossy().as_ref()).is_some()
}
