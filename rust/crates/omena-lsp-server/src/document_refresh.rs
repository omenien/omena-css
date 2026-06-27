use std::{
    collections::{BTreeSet, VecDeque},
    fs,
    path::Path,
};

#[cfg(test)]
use std::cell::Cell;

use omena_query::{
    StyleLanguage, summarize_omena_query_sass_module_sources, summarize_omena_query_style_document,
};

use crate::{
    LspDocumentOrigin, LspShellState, LspStyleDocumentSummary, LspTextDocumentState,
    foreign_style_identity::{is_foreign_style_document_uri, node_modules_package_for_path},
    lsp_text_document_state,
    protocol::{
        file_uri_to_path, is_style_document_uri, style_language_label, workspace_folder_compatible,
    },
    query_reuse::refresh_document_reusable_indexes,
    refresh_external_sifs_for_bridge_source_delta, refresh_external_sifs_for_state,
    refresh_source_type_fact_candidates_for_document, refresh_workspace_resolution_inputs,
    refresh_workspace_resolution_inputs_for_uri, resolution_inputs_for_workspace_uri,
    resolve_lsp_style_uri_for_specifier, resolve_workspace_folder_uri,
};

pub(crate) fn refresh_source_indexes_for_resolution_config_change(
    state: &mut LspShellState,
    config_uri: &str,
) {
    refresh_workspace_resolution_inputs_for_uri(state, config_uri);
    refresh_external_sifs_for_state(state);
    let workspace_folder_uri = resolve_workspace_folder_uri(state, config_uri);
    let source_uris = state
        .documents
        .values()
        .filter(|document| !is_style_document_uri(document.uri.as_str()))
        .filter(|document| {
            workspace_folder_uri.as_deref().is_none_or(|workspace_uri| {
                workspace_folder_compatible(Some(workspace_uri), document)
            })
        })
        .map(|document| document.uri.clone())
        .collect::<Vec<_>>();
    for source_uri in source_uris {
        let resolution_inputs = state
            .document(source_uri.as_str())
            .map(|document| {
                resolution_inputs_for_workspace_uri(state, document.workspace_folder_uri.as_deref())
            })
            .unwrap_or_else(|| {
                resolution_inputs_for_workspace_uri(state, workspace_folder_uri.as_deref())
            });
        if let Some(document) = state.document_mut(source_uri.as_str()) {
            refresh_document_reusable_indexes(document, &resolution_inputs);
        }
        refresh_source_type_fact_candidates_for_document(state, source_uri.as_str());
    }
}

pub(crate) fn refresh_source_indexes_for_resolution_settings_change(state: &mut LspShellState) {
    refresh_workspace_resolution_inputs(state);
    refresh_external_sifs_for_state(state);
    let source_uris = state
        .documents
        .values()
        .filter(|document| !is_style_document_uri(document.uri.as_str()))
        .map(|document| document.uri.clone())
        .collect::<Vec<_>>();
    for source_uri in source_uris {
        let resolution_inputs = state
            .document(source_uri.as_str())
            .map(|document| {
                resolution_inputs_for_workspace_uri(state, document.workspace_folder_uri.as_deref())
            })
            .unwrap_or_else(|| resolution_inputs_for_workspace_uri(state, None));
        if let Some(document) = state.document_mut(source_uri.as_str()) {
            refresh_document_reusable_indexes(document, &resolution_inputs);
        }
        refresh_source_type_fact_candidates_for_document(state, source_uri.as_str());
    }
}

pub(crate) fn is_resolution_config_document_uri(uri: &str) -> bool {
    let Some(path) = file_uri_to_path(uri) else {
        return false;
    };
    if is_package_manager_install_state_path(path.as_path()) {
        return true;
    }
    let Some(file_name) = path.file_name().and_then(|value| value.to_str()) else {
        return false;
    };
    file_name == "package.json"
        || file_name == "omena.lock"
        || file_name == "pnpm-lock.yaml"
        || file_name == "package-lock.json"
        || file_name == "yarn.lock"
        || file_name == "bun.lock"
        || file_name == "bun.lockb"
        || file_name == ".modules.yaml"
        || file_name.ends_with(".sif.json")
        || file_name == "jsconfig.json"
        || (file_name.starts_with("tsconfig") && file_name.ends_with(".json"))
        || matches!(
            file_name,
            "vite.config.ts"
                | "vite.config.mts"
                | "vite.config.cts"
                | "vite.config.js"
                | "vite.config.mjs"
                | "vite.config.cjs"
                | "webpack.config.ts"
                | "webpack.config.mts"
                | "webpack.config.cts"
                | "webpack.config.js"
                | "webpack.config.mjs"
                | "webpack.config.cjs"
        )
}

fn is_package_manager_install_state_path(path: &Path) -> bool {
    // The package-ROOT path (the `node_modules/<scope>/<pkg>` symlink itself) is what a
    // pnpm install / symlink-retarget touches. `node_modules_package_for_path` normalizes
    // that root case to subpath = "." (never ""), so matching `is_empty()` alone was dead
    // code; match the normalized "." sentinel and "" defensively.
    node_modules_package_for_path(path)
        .is_some_and(|(_, _, subpath)| subpath.is_empty() || subpath == ".")
}

pub(crate) fn ensure_style_document_loaded_from_disk(state: &mut LspShellState, uri: &str) -> bool {
    if state.contains_document_uri(uri) {
        return true;
    }
    reload_indexed_style_document_from_disk(state, uri)
}

pub(crate) fn reload_indexed_style_document_from_disk(
    state: &mut LspShellState,
    uri: &str,
) -> bool {
    let Some(path) = file_uri_to_path(uri) else {
        return false;
    };
    let Ok(text) = fs::read_to_string(path) else {
        return false;
    };
    let workspace_folder_uri = resolve_workspace_folder_uri(state, uri);
    let resolution_inputs =
        resolution_inputs_for_workspace_uri(state, workspace_folder_uri.as_deref());
    state.insert_document(
        uri,
        lsp_text_document_state(
            uri.to_string(),
            workspace_folder_uri,
            StyleLanguage::from_module_path(uri)
                .map(style_language_label)
                .unwrap_or("unknown")
                .to_string(),
            0,
            text,
            &resolution_inputs,
        ),
    );
    true
}

pub(crate) fn reload_indexed_source_document_from_disk(
    state: &mut LspShellState,
    uri: &str,
) -> bool {
    let Some(path) = file_uri_to_path(uri) else {
        return false;
    };
    let Some(language_id) = crate::workspace_index_language_id_for_uri(uri) else {
        return false;
    };
    if is_style_document_uri(uri) {
        return false;
    }
    let Ok(text) = fs::read_to_string(path) else {
        return false;
    };
    let workspace_folder_uri = resolve_workspace_folder_uri(state, uri);
    let resolution_inputs =
        resolution_inputs_for_workspace_uri(state, workspace_folder_uri.as_deref());
    state.insert_document(
        uri,
        lsp_text_document_state(
            uri.to_string(),
            workspace_folder_uri,
            language_id,
            0,
            text,
            &resolution_inputs,
        ),
    );
    true
}

const FOREIGN_STYLE_DEPENDENCY_ADMISSION_LIMIT: usize = 512;

#[cfg(test)]
thread_local! {
    static FOREIGN_STYLE_DEPENDENCY_SCAN_COUNT: Cell<usize> = const { Cell::new(0) };
}

#[cfg(test)]
pub(crate) fn reset_foreign_style_dependency_scan_count_for_test() {
    FOREIGN_STYLE_DEPENDENCY_SCAN_COUNT.with(|count| count.set(0));
}

#[cfg(test)]
pub(crate) fn foreign_style_dependency_scan_count_for_test() -> usize {
    FOREIGN_STYLE_DEPENDENCY_SCAN_COUNT.with(Cell::get)
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct StyleExternalDependencySnapshot {
    bridge_sources: Vec<String>,
    foreign_dependency_uris: Vec<String>,
}

pub(crate) fn style_external_dependency_snapshot(
    state: &LspShellState,
    uri: &str,
) -> StyleExternalDependencySnapshot {
    let Some(document) = state.document(uri) else {
        return StyleExternalDependencySnapshot::default();
    };
    let Some(summary) = document.style_summary.as_ref() else {
        return StyleExternalDependencySnapshot::default();
    };

    let mut bridge_sources = BTreeSet::new();
    let mut foreign_dependency_uris = BTreeSet::new();
    for source in summary
        .sass_module_use_sources
        .iter()
        .map(String::as_str)
        .chain(
            summary
                .sass_module_forward_sources
                .iter()
                .map(String::as_str),
        )
    {
        if source.starts_with("file://") {
            bridge_sources.insert(source.to_string());
        }
        if let Some(uri) = resolve_lsp_style_uri_for_specifier(state, document, source)
            && is_foreign_style_document_uri(uri.as_str())
        {
            bridge_sources.insert(source.to_string());
            foreign_dependency_uris.insert(uri);
        }
    }

    StyleExternalDependencySnapshot {
        bridge_sources: bridge_sources.into_iter().collect(),
        foreign_dependency_uris: foreign_dependency_uris.into_iter().collect(),
    }
}

pub(crate) fn refresh_style_external_inputs_for_document_event(
    state: &mut LspShellState,
    uri: &str,
    previous: Option<StyleExternalDependencySnapshot>,
) {
    let previous = previous.unwrap_or_default();
    let next = style_external_dependency_snapshot(state, uri);
    if previous == next {
        return;
    }

    if !previous.foreign_dependency_uris.is_empty() || !next.foreign_dependency_uris.is_empty() {
        admit_foreign_style_dependencies_for_style_uri(state, uri);
    }

    if previous.bridge_sources != next.bridge_sources {
        refresh_external_sifs_for_bridge_source_delta(
            state,
            previous.bridge_sources.as_slice(),
            next.bridge_sources.as_slice(),
        );
    }
}

pub(crate) fn refresh_style_external_inputs_after_document_removal(
    state: &mut LspShellState,
    previous: StyleExternalDependencySnapshot,
) {
    if !previous.bridge_sources.is_empty() {
        refresh_external_sifs_for_bridge_source_delta(
            state,
            previous.bridge_sources.as_slice(),
            &[],
        );
    }
}

pub(crate) fn admit_foreign_style_dependencies_for_indexed_style_documents(
    state: &mut LspShellState,
) {
    let style_uris = state
        .documents
        .values()
        .filter(|document| is_style_document_uri(document.uri.as_str()))
        .map(|document| document.uri.clone())
        .collect::<Vec<_>>();
    let _ = admit_foreign_style_dependencies_for_style_uris(state, style_uris);
}

pub(crate) fn admit_foreign_style_dependencies_for_style_uri(state: &mut LspShellState, uri: &str) {
    let _ = admit_foreign_style_dependencies_for_style_uris(state, vec![uri.to_string()]);
}

pub(crate) fn admit_foreign_style_dependencies_for_style_uris(
    state: &mut LspShellState,
    style_uris: Vec<String>,
) -> Vec<String> {
    let mut queue = style_uris.into_iter().collect::<VecDeque<_>>();
    let mut visited = BTreeSet::new();
    let mut admitted_uris = Vec::new();
    let mut admitted = 0usize;
    while let Some(current_uri) = queue.pop_front() {
        if admitted >= FOREIGN_STYLE_DEPENDENCY_ADMISSION_LIMIT
            || !visited.insert(current_uri.clone())
        {
            continue;
        }
        let dependency_uris = state
            .document(current_uri.as_str())
            .map(|document| style_module_dependency_target_uris(state, document))
            .unwrap_or_default();
        for dependency_uri in dependency_uris {
            if admitted >= FOREIGN_STYLE_DEPENDENCY_ADMISSION_LIMIT {
                break;
            }
            if !is_foreign_style_document_uri(dependency_uri.as_str()) {
                continue;
            }
            if !state.contains_document_uri(dependency_uri.as_str())
                && reload_indexed_style_document_from_disk(state, dependency_uri.as_str())
            {
                admitted += 1;
                admitted_uris.push(dependency_uri.clone());
            }
            if state
                .document(dependency_uri.as_str())
                .is_some_and(|document| document.origin == LspDocumentOrigin::Foreign)
            {
                queue.push_back(dependency_uri);
            }
        }
    }
    admitted_uris
}

fn style_module_dependency_target_uris(
    state: &LspShellState,
    document: &LspTextDocumentState,
) -> Vec<String> {
    #[cfg(test)]
    FOREIGN_STYLE_DEPENDENCY_SCAN_COUNT.with(|count| count.set(count.get().saturating_add(1)));
    let Some(sources) =
        summarize_omena_query_sass_module_sources(document.uri.as_str(), document.text.as_str())
    else {
        return Vec::new();
    };
    let mut uris = Vec::new();
    let module_sources = sources
        .module_use_edges
        .iter()
        .map(|edge| edge.source.as_str())
        .chain(sources.module_forward_sources.iter().map(String::as_str));
    for source in module_sources {
        if let Some(uri) = resolve_lsp_style_uri_for_specifier(state, document, source) {
            uris.push(uri);
        }
    }
    uris.sort();
    uris.dedup();
    uris
}

pub(crate) fn refresh_source_indexes_for_style_document_change(
    state: &mut LspShellState,
    style_uri: &str,
) {
    let workspace_folder_uri = state
        .document(style_uri)
        .and_then(|document| document.workspace_folder_uri.clone())
        .or_else(|| resolve_workspace_folder_uri(state, style_uri));
    let source_uris = state
        .documents
        .values()
        .filter(|document| !is_style_document_uri(document.uri.as_str()))
        .filter(|document| {
            workspace_folder_uri.as_deref().is_none_or(|workspace_uri| {
                workspace_folder_compatible(Some(workspace_uri), document)
            })
        })
        .map(|document| document.uri.clone())
        .collect::<Vec<_>>();
    for source_uri in source_uris {
        let resolution_inputs = state
            .document(source_uri.as_str())
            .map(|document| {
                resolution_inputs_for_workspace_uri(state, document.workspace_folder_uri.as_deref())
            })
            .unwrap_or_else(|| {
                resolution_inputs_for_workspace_uri(state, workspace_folder_uri.as_deref())
            });
        if let Some(document) = state.document_mut(source_uri.as_str()) {
            refresh_document_reusable_indexes(document, &resolution_inputs);
        }
        refresh_source_type_fact_candidates_for_document(state, source_uri.as_str());
    }
}

pub(crate) fn summarize_style_document(
    uri: &str,
    text: Option<&str>,
) -> Option<LspStyleDocumentSummary> {
    let text = text?;
    let summary = summarize_omena_query_style_document(uri, text)?;
    Some(LspStyleDocumentSummary {
        language: summary.language,
        selector_names: summary.selector_names,
        custom_property_decl_names: summary.custom_property_decl_names,
        custom_property_ref_names: summary.custom_property_ref_names,
        sass_module_use_sources: summary.sass_module_use_sources,
        sass_module_forward_sources: summary.sass_module_forward_sources,
        diagnostic_count: summary.diagnostic_count,
    })
}
