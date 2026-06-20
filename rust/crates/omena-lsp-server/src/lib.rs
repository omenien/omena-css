mod boundary;
mod diagnostics_follow_up;
mod diagnostics_scheduler;
mod disk_cache;
mod document_events;
mod document_state;
mod engine_input_params;
mod external_sif_loader;
mod external_sif_symbols;
mod foreign_style_identity;
mod frame_aware_refresh;
mod lsp_output;
mod message_loop;
mod occurrence_mapping;
mod open_document_inputs;
#[cfg(feature = "parallel-style-diagnostics")]
mod parallel_style_wave;
mod protocol;
mod query_adapter;
mod query_reuse;
mod settings;
mod source_completion;
mod source_diagnostics;
mod source_document_cache;
mod source_occurrence_cache;
mod source_selector_provider;
mod source_syntax_index;
mod source_type_fact_cache;
mod source_type_facts;
mod state;
mod streaming_ifds_diagnostics;
mod style_hover_markdown;
mod style_symbol_monikers;
mod style_symbol_occurrence_cache;
mod workspace_index;
mod workspace_occurrence_cache;
mod workspace_occurrences;
mod workspace_resolution;
mod workspace_runtime_registry;

pub use boundary::*;
pub use diagnostics_follow_up::*;
use disk_cache::disk_diagnostics_cache_slot_for_resolve;
pub(crate) use document_events::{
    did_change_text_document, did_change_watched_files, did_change_workspace_folders,
    did_close_text_document, did_open_text_document,
};
pub(crate) use document_state::{
    lsp_text_document_state, lsp_text_document_state_with_source_syntax_index,
};
use engine_input_params::query_engine_input_from_params;
pub use external_sif_loader::{
    LspExternalSifRefreshJobV0, LspExternalSifRefreshResultV0,
    apply_deferred_external_sif_refresh_result, collect_deferred_external_sif_refresh,
    enable_deferred_external_sif_refresh, prepare_deferred_external_sif_refresh_job,
};
pub(crate) use external_sif_loader::{
    refresh_external_sifs_for_bridge_source_delta, refresh_external_sifs_for_state,
};
use external_sif_symbols::external_sif_sass_symbol_definition_location;
pub(crate) use external_sif_symbols::{
    ExternalSifSassSymbolTarget, external_sif_sass_symbol_target_for_candidate,
};
use foreign_style_identity::{is_foreign_style_document_uri, node_modules_package_for_path};
pub use frame_aware_refresh::*;
pub use lsp_output::*;
#[cfg(test)]
pub(crate) use message_loop::current_time_millis;
pub use message_loop::{
    LspLoopTurnV0, LspQueryDispatchV0, dispatched_query_internal_error_response,
    handle_lsp_message, handle_lsp_message_outputs, handle_lsp_message_scheduled_outputs,
    handle_lsp_message_scheduled_outputs_or_dispatch, resolve_dispatched_query_response,
    workspace_index_progress_end_output,
};
use occurrence_mapping::{
    style_symbol_occurrence_for_candidate, workspace_occurrence_from_style_symbol_occurrence,
    workspace_occurrence_matches_target_style,
};
#[cfg(feature = "salsa-style-diagnostics")]
use omena_query::summarize_omena_query_target_unresolved_sass_import_diagnostics_for_workspace_paths;
use omena_query::{
    OmenaParserStyleDialect, OmenaQueryCodeActionV0, OmenaQueryCompletionCandidateV0,
    OmenaQueryCompletionItemV0, OmenaQuerySourceDocumentInputV0,
    OmenaQuerySourceDomainClassReferenceFactV0 as SourceDomainClassReferenceFact,
    OmenaQueryStyleDiagnosticV0, OmenaQueryStylePackageManifestV0, OmenaQueryStyleSourceInputV0,
    OmenaWorkspaceOccurrenceFamilyV0, OmenaWorkspaceOccurrenceIndexV0,
    OmenaWorkspaceOccurrenceRoleV0, OmenaWorkspaceOccurrenceV0, ParserPositionV0, ParserRangeV0,
    StyleLanguage, is_omena_query_sass_symbol_candidate_kind as is_sass_symbol_candidate_kind,
    is_omena_query_sass_symbol_declaration_kind as is_sass_symbol_declaration_kind,
    is_omena_query_sass_symbol_reference_kind as is_sass_symbol_reference_kind,
    occurrences_for_monikers,
    omena_query_sass_symbol_kind_from_candidate_kind as sass_symbol_kind_from_candidate_kind,
    read_omena_query_cascade_at_position_with_categorical_evidence,
    read_omena_query_style_context_index, resolve_omena_query_sass_forward_sources,
    resolve_omena_query_sass_module_use_sources_for_candidate,
    resolve_omena_query_sass_symbol_declarations,
    resolve_omena_query_source_candidate_selector_names,
    resolve_omena_query_style_uri_for_specifier_with_resolution_inputs,
    summarize_omena_query_omena_parser_style_facts,
    summarize_omena_query_refs_for_class_from_occurrence_index,
    summarize_omena_query_rename_plan_from_occurrence_index,
    summarize_omena_query_sass_module_sources, summarize_omena_query_source_completion_at_position,
    summarize_omena_query_style_completion_candidate_documentation,
    summarize_omena_query_style_completion_candidate_documentation_for_workspace_file_with_substrate,
    summarize_omena_query_style_completion_for_workspace_file_with_substrate,
    summarize_omena_query_style_diagnostics_for_file,
    summarize_omena_query_style_diagnostics_for_file_with_deep_analysis,
    summarize_omena_query_style_document,
    summarize_omena_query_style_hover_render_parts_for_hover_position,
    summarize_omena_query_style_hover_render_parts_for_workspace_file_hover_position_with_substrate,
    summarize_omena_query_style_refactor_code_actions,
};
#[cfg(not(feature = "salsa-style-diagnostics"))]
use omena_query::{
    OmenaQueryExternalModuleModeV0,
    summarize_omena_query_style_diagnostics_for_workspace_file_with_external_mode_and_sifs_and_resolution_inputs,
};
#[cfg(test)]
pub(crate) use omena_query::{
    OmenaQueryExternalSifInputV0, OmenaQuerySourceImportedStyleBindingV0 as ImportedStyleBinding,
    OmenaQuerySourceSelectorReferenceFactV0 as SourceSelectorReferenceFact,
    OmenaQuerySourceSelectorReferenceMatchKindV0 as SourceSelectorReferenceMatchKind,
    OmenaQuerySourceSyntaxIndexV0 as SourceSyntaxIndex, ParserByteSpanV0,
};
#[cfg(test)]
pub(crate) use omena_tsgo_client::{TsgoResolvedTypeV0, TsgoTypeFactResultEntryV0};
#[cfg(feature = "salsa-style-diagnostics")]
pub(crate) use open_document_inputs::style_path_inputs_from_open_documents;
pub(crate) use open_document_inputs::{
    source_documents_from_open_documents, style_sources_for_hover_render,
    style_sources_from_open_documents,
};
use protocol::*;
use query_adapter::*;
use query_reuse::{
    cascade_narrowing_substrate_for_style_sources, effective_style_package_manifests,
    refresh_document_reusable_indexes,
};
use serde_json::{Value, json};
pub(crate) use settings::{
    apply_diagnostic_settings, apply_feature_settings, apply_resolution_settings,
};
use source_completion::{
    source_completion_context_at_position, source_domain_option_completion_items,
    source_domain_reference_option_names,
};
pub(crate) use source_diagnostics::{
    finish_source_diagnostics_value, prepare_deferred_source_diagnostics_for_uri,
    resolve_source_diagnostics_for_uri,
};
use source_occurrence_cache::store_source_selector_occurrence_sidecar;
pub(crate) use source_selector_provider::{
    collect_source_selector_reference_candidates, document_has_style_index,
    first_style_document_for_workspace, resolve_source_provider_candidates,
    source_selector_candidate_at_position, source_selector_candidate_for_params,
    source_selector_candidates_at_position, style_selector_definitions_for_source_candidates,
    style_selector_definitions_from_open_documents, style_selector_definitions_from_uri,
};
pub(crate) use source_syntax_index::{
    build_source_syntax_index, collect_source_imports, source_selector_candidates_from_index,
};
#[cfg(test)]
pub(crate) use source_type_facts::apply_source_type_fact_results_to_document;
pub(crate) use source_type_facts::refresh_source_type_fact_candidates_for_document;
pub use state::*;
use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    fs,
    path::Path,
    sync::Arc,
};
use streaming_ifds_diagnostics::summarize_cross_file_streaming_reachability_diagnostics_for_lsp;
use style_hover_markdown::render_style_hover_candidate_markdown_from_parts;
use style_symbol_monikers::{
    render_external_sif_sass_symbol_hover_markdown, style_custom_property_moniker,
    style_external_sif_sass_symbol_moniker, style_sass_symbol_moniker_for_document,
    style_sass_symbol_moniker_for_uri, style_symbol_monikers_for_candidate,
    style_symbol_role_for_candidate, style_unresolved_sass_symbol_moniker,
};
use style_symbol_occurrence_cache::store_style_symbol_occurrence_sidecar;
pub(crate) use workspace_index::index_workspace_style_files;
pub(crate) use workspace_index::workspace_index_language_id_for_uri;
pub use workspace_index::{
    LspWorkspaceIndexJobV0, LspWorkspaceIndexResultV0, apply_background_workspace_index_result,
    collect_background_workspace_index, prepare_background_workspace_index_continuation_job,
    prepare_background_workspace_index_job,
};
#[cfg(test)]
pub(crate) use workspace_index::{
    WorkspaceStyleIndexBudget, index_workspace_style_files_with_budget,
};
use workspace_occurrence_cache::{
    load_workspace_occurrence_shard, store_workspace_occurrence_shard,
};
#[cfg(test)]
use workspace_occurrences::{
    source_selector_occurrence_document_keys, style_symbol_occurrence_document_keys,
};
use workspace_occurrences::{
    source_selector_occurrence_index_from_open_documents,
    workspace_occurrence_indexes_from_documents,
};
#[cfg(test)]
pub(crate) use workspace_resolution::load_lsp_workspace_style_resolution_inputs;
pub(crate) use workspace_resolution::{
    initialize_workspace_folders, insert_workspace_folder, refresh_document_workspace_owners,
    refresh_workspace_resolution_inputs, refresh_workspace_resolution_inputs_for_uri,
    resolution_inputs_for_workspace_uri, resolve_workspace_folder_uri,
};

pub const NODE_TEXT_DOCUMENT_SYNC_KIND: u8 = 2;
pub const DEBUG_STATE_REQUEST: &str = "omena/rustLspState";
pub const RUNTIME_LOOP_PROBE_REQUEST: &str = "omena/runtimeLoopProbe";
pub const STYLE_HOVER_CANDIDATES_REQUEST: &str = "omena/rustStyleHoverCandidates";
pub const STYLE_DIAGNOSTICS_REQUEST: &str = "omena/rustStyleDiagnostics";
pub const SOURCE_DIAGNOSTICS_REQUEST: &str = "omena/rustSourceDiagnostics";
pub const CASCADE_AT_POSITION_REQUEST: &str = "omena/rustCascadeAtPosition";
pub const STYLE_CONTEXT_INDEX_REQUEST: &str = "omena/rustStyleContextIndex";
pub const EXPLAIN_HOVER_TRACE_REQUEST: &str = "omena/explainHoverTrace";
const CANCEL_REQUEST_METHOD: &str = "$/cancelRequest";
const REQUEST_CANCELLED_ERROR_CODE: i32 = -32800;
// Cascade docs cost a whole-corpus narrowing analysis per completion item; only the
// top-ranked items an editor list actually shows get them, so completion latency
// stays independent of the workspace selector count.
const SOURCE_COMPLETION_DOCUMENTATION_BUDGET: usize = 12;

#[cfg(feature = "test-support")]
pub mod test_support {
    use std::path::Path;

    pub fn file_uri_equivalent(left: &str, right: &str) -> bool {
        crate::protocol::file_uri_equivalent(left, right)
    }

    pub fn path_to_file_uri(path: &Path) -> String {
        crate::protocol::path_to_file_uri(path)
    }
}

fn refresh_source_indexes_for_resolution_config_change(
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
    // code — the package-root watched event never fired an external-SIF refresh. Match the
    // normalized "." sentinel (and "" defensively).
    node_modules_package_for_path(path)
        .is_some_and(|(_, _, subpath)| subpath.is_empty() || subpath == ".")
}

pub(crate) fn ensure_style_document_loaded_from_disk(state: &mut LspShellState, uri: &str) -> bool {
    if state.contains_document_uri(uri) {
        return true;
    }
    reload_indexed_style_document_from_disk(state, uri)
}

fn reload_indexed_style_document_from_disk(state: &mut LspShellState, uri: &str) -> bool {
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

fn reload_indexed_source_document_from_disk(state: &mut LspShellState, uri: &str) -> bool {
    let Some(path) = file_uri_to_path(uri) else {
        return false;
    };
    let Some(language_id) = workspace_index_language_id_for_uri(uri) else {
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

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct StyleExternalDependencySnapshot {
    bridge_sources: Vec<String>,
    foreign_dependency_uris: Vec<String>,
}

fn style_external_dependency_snapshot(
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

fn refresh_style_external_inputs_for_document_event(
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

fn refresh_style_external_inputs_after_document_removal(
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
    admit_foreign_style_dependencies_for_style_uris(state, style_uris);
}

fn admit_foreign_style_dependencies_for_style_uri(state: &mut LspShellState, uri: &str) {
    admit_foreign_style_dependencies_for_style_uris(state, vec![uri.to_string()]);
}

fn admit_foreign_style_dependencies_for_style_uris(
    state: &mut LspShellState,
    style_uris: Vec<String>,
) {
    let mut queue = style_uris.into_iter().collect::<VecDeque<_>>();
    let mut visited = BTreeSet::new();
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
            }
            if state
                .document(dependency_uri.as_str())
                .is_some_and(|document| document.origin == LspDocumentOrigin::Foreign)
            {
                queue.push_back(dependency_uri);
            }
        }
    }
}

fn style_module_dependency_target_uris(
    state: &LspShellState,
    document: &LspTextDocumentState,
) -> Vec<String> {
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

fn refresh_source_indexes_for_style_document_change(state: &mut LspShellState, style_uri: &str) {
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

fn summarize_style_document(uri: &str, text: Option<&str>) -> Option<LspStyleDocumentSummary> {
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

pub fn resolve_style_hover_candidates(
    state: &LspShellState,
    params: Option<&Value>,
) -> LspStyleHoverCandidatesResult {
    let document_uri = document_uri_from_params(params);
    let query_position = lsp_position_from_params(params);
    let Some(document) = state.document(&document_uri) else {
        return empty_style_hover_candidates_result(document_uri, None, query_position);
    };

    let Some((language, mut candidates)) = style_hover_candidates_for_document(document) else {
        return empty_style_hover_candidates_result(
            document_uri,
            document.workspace_folder_uri.clone(),
            query_position,
        );
    };

    if let Some(position) = query_position {
        candidates.retain(|candidate| parser_range_contains_position(&candidate.range, position));
    }

    LspStyleHoverCandidatesResult {
        schema_version: "0",
        product: "omena-lsp-server.style-hover-candidates",
        document_uri,
        workspace_folder_uri: document.workspace_folder_uri.clone(),
        language: Some(language),
        query_position,
        candidate_count: candidates.len(),
        candidates,
    }
}

fn style_hover_candidates_for_document(
    document: &LspTextDocumentState,
) -> Option<(&'static str, Vec<LspStyleHoverCandidate>)> {
    let summary = document.style_summary.as_ref()?;
    Some((summary.language, document.style_candidates.clone()))
}

fn style_text_for_uri(state: &LspShellState, uri: &str) -> Option<String> {
    state
        .document(uri)
        .map(|document| document.text.clone())
        .or_else(|| fs::read_to_string(file_uri_to_path(uri)?).ok())
}

fn style_hover_candidates_for_uri(
    state: &LspShellState,
    uri: &str,
) -> Option<(&'static str, Vec<LspStyleHoverCandidate>)> {
    if let Some(document) = state.document(uri) {
        return style_hover_candidates_for_document(document);
    }
    let text = style_text_for_uri(state, uri)?;
    collect_style_hover_candidates(uri, text.as_str())
}

fn resolve_lsp_definition(state: &LspShellState, params: Option<&Value>) -> Value {
    let document_uri = document_uri_from_params(params);
    let Some(position) = lsp_position_from_params(params) else {
        return Value::Null;
    };
    let Some(document) = state.document(&document_uri) else {
        return Value::Null;
    };
    if !is_style_document_uri(document.uri.as_str()) {
        return resolve_source_lsp_definition(state, document, position);
    }

    let Some((_, candidates)) = style_hover_candidates_for_document(document) else {
        return Value::Null;
    };
    let Some(candidate) = candidates
        .iter()
        .find(|candidate| parser_range_contains_position(&candidate.range, position))
    else {
        return Value::Null;
    };
    if is_sass_symbol_reference_kind(candidate.kind) {
        let definitions = sass_symbol_definitions_for_candidate(state, document, candidate);
        if !definitions.is_empty() {
            return json!(
                definitions
                    .into_iter()
                    .map(|(uri, definition)| json!({ "uri": uri, "range": definition.range }))
                    .collect::<Vec<_>>()
            );
        }
        if let Some(location) =
            external_sif_sass_symbol_definition_location(state, document, candidate)
        {
            return json!([location]);
        }
        return Value::Null;
    }
    if candidate.kind == "customPropertyReference" {
        let definitions =
            style_symbol_definition_locations_from_documents(state, document, candidate);
        if !definitions.is_empty() {
            return json!(definitions);
        }
    }

    let target = candidate;

    json!([
        {
            "uri": document.uri.as_str(),
            "range": target.range,
        },
    ])
}

fn resolve_lsp_references(state: &LspShellState, params: Option<&Value>) -> Value {
    let document_uri = document_uri_from_params(params);
    let Some(position) = lsp_position_from_params(params) else {
        return Value::Null;
    };
    let Some(document) = state.document(&document_uri) else {
        return Value::Null;
    };
    if !is_style_document_uri(document.uri.as_str()) {
        return resolve_source_lsp_references(state, document, position, params);
    }

    let Some((_, candidates)) = style_hover_candidates_for_document(document) else {
        return Value::Null;
    };
    let Some(candidate) = candidates
        .iter()
        .find(|candidate| parser_range_contains_position(&candidate.range, position))
    else {
        return Value::Null;
    };
    let include_declaration = include_declaration_from_params(params);
    let mut locations: Vec<Value> = if candidate.kind.starts_with("customProperty")
        || is_sass_symbol_candidate_kind(candidate.kind)
    {
        style_symbol_reference_locations_from_documents(
            state,
            document,
            candidate,
            include_declaration,
        )
    } else if candidate.kind == "selector" {
        let mut locations = if include_declaration {
            vec![json!({ "uri": document.uri.as_str(), "range": candidate.range })]
        } else {
            Vec::new()
        };
        locations.extend(selector_reference_locations_from_open_documents(
            state,
            candidate.name.as_str(),
            document.workspace_folder_uri.as_deref(),
            Some(document.uri.as_str()),
        ));
        locations
    } else if include_declaration {
        vec![json!({ "uri": document.uri.as_str(), "range": candidate.range })]
    } else {
        Vec::new()
    };

    locations.sort_by_key(|location| {
        let line = location
            .pointer("/range/start/line")
            .and_then(Value::as_u64)
            .unwrap_or_default();
        let character = location
            .pointer("/range/start/character")
            .and_then(Value::as_u64)
            .unwrap_or_default();
        (line, character)
    });
    json!(locations)
}

fn resolve_lsp_completion(state: &LspShellState, params: Option<&Value>) -> Value {
    let document_uri = document_uri_from_params(params);
    let Some(document) = state.document(&document_uri) else {
        return Value::Null;
    };
    if !is_style_document_uri(document.uri.as_str()) {
        return resolve_source_lsp_completion(state, document, params);
    }

    let Some(position) = lsp_position_from_params(params) else {
        return Value::Null;
    };
    let style_sources = style_sources_from_open_documents(
        state,
        document.workspace_folder_uri.as_deref(),
        Some(document.uri.as_str()),
    );
    let resolution_inputs =
        resolution_inputs_for_workspace_uri(state, document.workspace_folder_uri.as_deref());
    let package_manifests = effective_style_package_manifests(state, &resolution_inputs);
    let narrowing_substrate = cascade_narrowing_substrate_for_style_sources(
        state,
        style_sources.as_slice(),
        &resolution_inputs,
    );
    let completion = summarize_omena_query_style_completion_for_workspace_file_with_substrate(
        document.uri.as_str(),
        style_sources.as_slice(),
        package_manifests.as_slice(),
        state.resolution.external_sifs.as_slice(),
        &resolution_inputs,
        &narrowing_substrate,
        position,
    );
    let provider_feedback =
        current_provider_tier_feedback_data(document, "textDocument/completion");
    let items: Vec<Value> = completion
        .items
        .into_iter()
        .map(|item| {
            lsp_completion_item_from_query(completion.file_kind, item, provider_feedback.as_ref())
        })
        .collect();

    json!({
        "isIncomplete": false,
        "items": items,
    })
}

fn lsp_completion_item_from_query(
    file_kind: &str,
    item: OmenaQueryCompletionItemV0,
    provider_feedback: Option<&Value>,
) -> Value {
    let kind = match (file_kind, item.item_kind) {
        ("style", "cssModuleSelector") => 7,
        (_, "cssModuleSelector") | (_, "cssCustomProperty") => 10,
        _ => 1,
    };
    let mut completion_item = json!({
        "label": item.label,
        "kind": kind,
        "sortText": item.sort_text,
        "detail": item.detail,
        "insertText": item.insert_text,
        "data": {
            "source": item.source,
            "rankingSource": item.ranking_source,
        },
    });
    if let Some(documentation) = item.documentation {
        completion_item["documentation"] = json!({
            "kind": "markdown",
            "value": documentation,
        });
    }
    attach_provider_tier_feedback(&mut completion_item, provider_feedback);
    completion_item
}

fn resolve_style_diagnostics(state: &LspShellState, params: Option<&Value>) -> Value {
    let document_uri = document_uri_from_params(params);
    resolve_style_diagnostics_for_uri(state, document_uri.as_str())
}

fn resolve_source_diagnostics(state: &LspShellState, params: Option<&Value>) -> Value {
    let document_uri = document_uri_from_params(params);
    resolve_source_diagnostics_for_uri(state, document_uri.as_str())
}

fn resolve_cascade_at_position(state: &LspShellState, params: Option<&Value>) -> Value {
    let document_uri = document_uri_from_params(params);
    let Some(position) = lsp_position_from_params(params) else {
        return Value::Null;
    };
    let Some(document) = state.document(&document_uri) else {
        return Value::Null;
    };
    if !is_style_document_uri(document.uri.as_str()) {
        return Value::Null;
    }
    let Some(engine_input) = query_engine_input_from_params(params) else {
        return Value::Null;
    };

    let include_categorical_evidence = params
        .and_then(|value| value.get("context"))
        .and_then(|value| value.get("includeCategoricalEvidence"))
        .and_then(Value::as_bool)
        .unwrap_or(false);

    read_omena_query_cascade_at_position_with_categorical_evidence(
        document.uri.as_str(),
        document.text.as_str(),
        &engine_input,
        position,
        include_categorical_evidence,
    )
    .map(|result| json!(result))
    .unwrap_or(Value::Null)
}

fn resolve_style_context_index(state: &LspShellState, params: Option<&Value>) -> Value {
    let document_uri = document_uri_from_params(params);
    let Some(document) = state.document(&document_uri) else {
        return Value::Null;
    };
    if !is_style_document_uri(document.uri.as_str()) {
        return Value::Null;
    }
    let Some(engine_input) = query_engine_input_from_params(params) else {
        return Value::Null;
    };

    read_omena_query_style_context_index(
        document.uri.as_str(),
        document.text.as_str(),
        &engine_input,
    )
    .map(|result| json!(result))
    .unwrap_or(Value::Null)
}

fn resolve_document_diagnostics_for_uri(state: &LspShellState, document_uri: &str) -> Value {
    if is_style_document_uri(document_uri) {
        resolve_style_diagnostics_for_uri(state, document_uri)
    } else {
        resolve_source_diagnostics_for_uri(state, document_uri)
    }
}

fn resolve_style_diagnostics_for_uri(state: &LspShellState, document_uri: &str) -> Value {
    let Some(document) = state.document(document_uri) else {
        return json!([]);
    };
    let Some((_, candidates)) = style_hover_candidates_for_document(document) else {
        return json!([]);
    };

    let query_candidates = candidates
        .iter()
        .map(query_style_hover_candidate_from_lsp)
        .collect::<Vec<_>>();
    let style_sources = style_sources_from_open_documents(
        state,
        document.workspace_folder_uri.as_deref(),
        Some(document.uri.as_str()),
    );
    let source_documents =
        source_documents_from_open_documents(state, document.workspace_folder_uri.as_deref());
    let external_sifs = state.resolution.external_sifs.as_slice();
    // RFC-0007-J (#50): pass the workspace's tsconfig/bundler path mappings so the unused-selector
    // usage collector resolves alias style imports (`@/styles/...`) the same way the reference/goto
    // path does — otherwise an alias import dims every selector as `unusedSelector`.
    let resolution_inputs =
        resolution_inputs_for_workspace_uri(state, document.workspace_folder_uri.as_deref());
    // RFC 0009 Pillar C (rfcs#66) stage 1: the persistent content-addressed
    // shard cache. The composite key chains the FULL input surface gathered
    // above (target path, every style source, every source document, package
    // manifests, external SIFs, resolution inputs, diagnostics settings) plus
    // crate/schema/arm versions, so a shard can only serve when a recompute
    // would be byte-identical by construction. Misses fall through to the
    // compute below and persist write-behind; everything is fail-soft and
    // killable via OMENA_LSP_DISK_CACHE=off.
    let disk_cache_slot = disk_diagnostics_cache_slot_for_resolve(
        state,
        document.workspace_folder_uri.as_deref(),
        document.uri.as_str(),
        style_sources.as_slice(),
        source_documents.as_slice(),
        external_sifs,
        &resolution_inputs,
    );
    if let Some(cached_diagnostics) = disk_cache_slot.as_ref().and_then(|slot| slot.load()) {
        return cached_diagnostics;
    }
    // RFC 0009 Pillar B (rfcs#65): the workspace entry point runs through the
    // salsa-memoized host (input diff-sync + tracked query) so an unchanged
    // corpus revalidates instead of recomputing. `--no-default-features`
    // preserves the straight-line call; byte-identity between the two is
    // enforced by omena-diff-test's salsaMemoizedVsFromScratchEquivalence
    // gate. Both arms use query-level per-edge external classification.
    #[cfg(feature = "salsa-style-diagnostics")]
    let workspace_diagnostics_summary = {
        let mut host_slot = state.style_memo_host.borrow_mut();
        let host = host_slot.get_or_insert_with(omena_query::OmenaQueryStyleMemoHostV0::new);
        host.workspace_style_diagnostics(
            document.uri.as_str(),
            style_sources.as_slice(),
            source_documents.as_slice(),
            state.resolution.package_manifests.as_slice(),
            external_sifs,
            &resolution_inputs,
        )
    };
    #[cfg(not(feature = "salsa-style-diagnostics"))]
    let workspace_diagnostics_summary =
        summarize_omena_query_style_diagnostics_for_workspace_file_with_external_mode_and_sifs_and_resolution_inputs(
            document.uri.as_str(),
            style_sources.as_slice(),
            source_documents.as_slice(),
            state.resolution.package_manifests.as_slice(),
            None,
            OmenaQueryExternalModuleModeV0::Auto,
            external_sifs,
            &resolution_inputs,
        );
    let diagnostics = finish_style_diagnostics_value(
        &LspStyleDiagnosticsRenderInputsV0 {
            document_uri: document.uri.as_str(),
            document_text: document.text.as_str(),
            query_candidates: query_candidates.as_slice(),
            style_sources: style_sources.as_slice(),
            source_documents: source_documents.as_slice(),
            package_manifests: state.resolution.package_manifests.as_slice(),
            deep_analysis: state.diagnostics.deep_analysis,
            configured_severity: state.diagnostics.severity,
        },
        workspace_diagnostics_summary,
    );
    // RFC 0009 Pillar C (rfcs#66): write-behind after the compute. Fail-soft —
    // io errors are swallowed and a session breaker stops retrying hot.
    if let Some(slot) = disk_cache_slot.as_ref() {
        slot.store_write_behind(state, &diagnostics);
    }
    diagnostics
}

/// The full argument surface of [`finish_style_diagnostics_value`]: plain
/// `Send` data only, by design — no `&LspShellState`.
pub(crate) struct LspStyleDiagnosticsRenderInputsV0<'inputs> {
    pub(crate) document_uri: &'inputs str,
    pub(crate) document_text: &'inputs str,
    pub(crate) query_candidates: &'inputs [omena_query::OmenaQueryStyleHoverCandidateV0],
    pub(crate) style_sources: &'inputs [OmenaQueryStyleSourceInputV0],
    pub(crate) source_documents: &'inputs [OmenaQuerySourceDocumentInputV0],
    pub(crate) package_manifests: &'inputs [OmenaQueryStylePackageManifestV0],
    pub(crate) deep_analysis: bool,
    pub(crate) configured_severity: u8,
}

#[cfg(feature = "salsa-style-diagnostics")]
impl LspOwnedStyleDiagnosticsRenderInputsV0 {
    fn borrowed(&self) -> LspStyleDiagnosticsRenderInputsV0<'_> {
        LspStyleDiagnosticsRenderInputsV0 {
            document_uri: self.document_uri.as_str(),
            document_text: self.document_text.as_str(),
            query_candidates: self.query_candidates.as_slice(),
            style_sources: self.style_sources.as_slice(),
            source_documents: self.source_documents.as_slice(),
            package_manifests: self.package_manifests.as_slice(),
            deep_analysis: self.deep_analysis,
            configured_severity: self.configured_severity,
        }
    }
}

pub(crate) fn prepare_deferred_style_diagnostics_for_uri(
    state: &LspShellState,
    document_uri: &str,
    tier_plan: DiagnosticsPipelineTierPlanV0,
) -> Option<(Value, LspDeferredDiagnosticsDispatchV0)> {
    #[cfg(not(feature = "salsa-style-diagnostics"))]
    {
        let _ = (state, document_uri, tier_plan);
        None
    }
    #[cfg(feature = "salsa-style-diagnostics")]
    {
        let document = state.document(document_uri)?;
        let (_, candidates) = style_hover_candidates_for_document(document)?;
        let query_candidates = candidates
            .iter()
            .map(query_style_hover_candidate_from_lsp)
            .collect::<Vec<_>>();
        let style_paths = style_path_inputs_from_open_documents(
            state,
            document.workspace_folder_uri.as_deref(),
            Some(document.uri.as_str()),
        );

        let mut baseline_summary = summarize_omena_query_style_diagnostics_for_file(
            document.uri.as_str(),
            document.text.as_str(),
            query_candidates.as_slice(),
        );
        baseline_summary.diagnostics.extend(
            summarize_omena_query_target_unresolved_sass_import_diagnostics_for_workspace_paths(
                document.uri.as_str(),
                document.text.as_str(),
                style_paths.as_slice(),
                state.resolution.package_manifests.as_slice(),
            ),
        );
        baseline_summary.diagnostic_count = baseline_summary.diagnostics.len();
        let baseline_render_inputs = LspStyleDiagnosticsRenderInputsV0 {
            document_uri: document.uri.as_str(),
            document_text: document.text.as_str(),
            query_candidates: query_candidates.as_slice(),
            style_sources: style_paths.as_slice(),
            source_documents: &[],
            package_manifests: state.resolution.package_manifests.as_slice(),
            deep_analysis: state.diagnostics.deep_analysis,
            configured_severity: state.diagnostics.severity,
        };
        let baseline_diagnostics =
            render_style_diagnostics_summary_value(&baseline_render_inputs, baseline_summary);
        let dispatch = LspDeferredDiagnosticsDispatchV0 {
            uri: document_uri.to_string(),
            coalesce_key: String::new(),
            tier_plan,
            render_inputs: DeferredDiagnosticsRenderInputsV0::StyleSnapshot(Box::new(
                state.query_snapshot(),
            )),
        };
        Some((baseline_diagnostics, dispatch))
    }
}

#[cfg(feature = "salsa-style-diagnostics")]
fn owned_style_diagnostics_render_inputs_for_uri(
    state: &LspShellState,
    document_uri: &str,
) -> Option<LspOwnedStyleDiagnosticsRenderInputsV0> {
    let document = state.document(document_uri)?;
    let (_, candidates) = style_hover_candidates_for_document(document)?;
    let query_candidates = candidates
        .iter()
        .map(query_style_hover_candidate_from_lsp)
        .collect::<Vec<_>>();
    let style_sources = style_sources_from_open_documents(
        state,
        document.workspace_folder_uri.as_deref(),
        Some(document.uri.as_str()),
    );
    let source_documents =
        source_documents_from_open_documents(state, document.workspace_folder_uri.as_deref());
    let resolution_inputs =
        resolution_inputs_for_workspace_uri(state, document.workspace_folder_uri.as_deref());
    Some(LspOwnedStyleDiagnosticsRenderInputsV0 {
        document_uri: document.uri.clone(),
        document_text: document.text.clone(),
        query_candidates,
        style_sources,
        source_documents,
        package_manifests: state.resolution.package_manifests.clone(),
        external_sifs: state.resolution.external_sifs.clone(),
        resolution_inputs,
        deep_analysis: state.diagnostics.deep_analysis,
        configured_severity: state.diagnostics.severity,
    })
}

#[cfg(feature = "salsa-style-diagnostics")]
pub fn resolve_deferred_diagnostics_notification(
    host: &mut omena_query::OmenaQueryStyleMemoHostV0,
    dispatch: &LspDeferredDiagnosticsDispatchV0,
) -> Value {
    let diagnostics = match &dispatch.render_inputs {
        DeferredDiagnosticsRenderInputsV0::StyleSnapshot(snapshot) => {
            let Some(inputs) = owned_style_diagnostics_render_inputs_for_uri(
                snapshot.shell_state(),
                &dispatch.uri,
            ) else {
                return diagnostics_scheduler::deferred_full_diagnostics_notification(
                    dispatch.uri.as_str(),
                    json!([]),
                    dispatch.tier_plan,
                );
            };
            let workspace_summary = host.workspace_style_diagnostics(
                inputs.document_uri.as_str(),
                inputs.style_sources.as_slice(),
                inputs.source_documents.as_slice(),
                inputs.package_manifests.as_slice(),
                inputs.external_sifs.as_slice(),
                &inputs.resolution_inputs,
            );
            finish_style_diagnostics_value(&inputs.borrowed(), workspace_summary)
        }
        DeferredDiagnosticsRenderInputsV0::Source(inputs) => {
            finish_source_diagnostics_value(&inputs.borrowed())
        }
    };
    diagnostics_scheduler::deferred_full_diagnostics_notification(
        dispatch.uri.as_str(),
        diagnostics,
        dispatch.tier_plan,
    )
}

/// RFC 0009 Pillar F (rfcs#68): the worker-safe tail of the style
/// diagnostics pipeline — per-file fallback summarize, streaming-IFDS
/// extend, opt-in deep analysis, severity mapping and LSP JSON rendering.
/// Pure of its arguments, so the serial resolve and the parallel wave share
/// ONE implementation and cannot drift byte-wise.
pub(crate) fn finish_style_diagnostics_value(
    inputs: &LspStyleDiagnosticsRenderInputsV0<'_>,
    workspace_diagnostics_summary: Option<omena_query::OmenaQueryStyleDiagnosticsForFileV0>,
) -> Value {
    let mut diagnostics_summary = workspace_diagnostics_summary.unwrap_or_else(|| {
        summarize_omena_query_style_diagnostics_for_file(
            inputs.document_uri,
            inputs.document_text,
            inputs.query_candidates,
        )
    });
    diagnostics_summary.diagnostics.extend(
        summarize_cross_file_streaming_reachability_diagnostics_for_lsp(
            inputs.document_uri,
            inputs.style_sources,
            inputs.source_documents,
            inputs.package_manifests,
        ),
    );
    if inputs.deep_analysis {
        diagnostics_summary
            .diagnostics
            .extend(summarize_lsp_opt_in_deep_analysis_diagnostics(
                inputs.document_uri,
                inputs.document_text,
                inputs.query_candidates,
            ));
    }
    diagnostics_summary.diagnostic_count = diagnostics_summary.diagnostics.len();
    render_style_diagnostics_summary_value(inputs, diagnostics_summary)
}

fn render_style_diagnostics_summary_value(
    inputs: &LspStyleDiagnosticsRenderInputsV0<'_>,
    diagnostics_summary: omena_query::OmenaQueryStyleDiagnosticsForFileV0,
) -> Value {
    let diagnostics = diagnostics_summary
        .diagnostics
        .into_iter()
        .map(|diagnostic| {
            let tags = diagnostic.tags;
            let query_severity = diagnostic.severity;
            let mut data = serde_json::Map::new();
            data.insert("querySeverity".to_string(), json!(query_severity));
            data.insert("provenance".to_string(), json!(diagnostic.provenance));
            if let Some(create_custom_property) = diagnostic.create_custom_property {
                data.insert(
                    "createCustomProperty".to_string(),
                    json!(create_custom_property),
                );
            }
            if let Some(cascade_narrowing) = diagnostic.cascade_narrowing {
                if let Some(runtime_state) = cascade_narrowing.runtime_state.as_ref() {
                    data.insert("runtimeState".to_string(), json!(runtime_state));
                }
                data.insert("cascadeNarrowing".to_string(), json!(cascade_narrowing));
            }
            if let Some(cascade_confidence) = diagnostic.cascade_confidence {
                data.insert("cascadeConfidence".to_string(), json!(cascade_confidence));
            }
            if let Some(polynomial_provenance) = diagnostic.polynomial_provenance {
                data.insert(
                    "polynomialProvenance".to_string(),
                    json!(polynomial_provenance),
                );
            }
            if let Some(cross_file_scc) = diagnostic.cross_file_scc {
                data.insert("crossFileScc".to_string(), json!(cross_file_scc));
            }

            let mut lsp_diagnostic = json!({
                "range": diagnostic.range,
                "severity": lsp_diagnostic_severity(query_severity, inputs.configured_severity),
                "code": diagnostic.code,
                "source": "omena-css",
                "message": diagnostic.message,
                "data": Value::Object(data),
            });
            if !tags.is_empty() {
                lsp_diagnostic["tags"] = json!(tags);
            }
            lsp_diagnostic
        })
        .collect::<Vec<_>>();

    json!(diagnostics)
}

fn summarize_lsp_opt_in_deep_analysis_diagnostics(
    document_uri: &str,
    text: &str,
    candidates: &[omena_query::OmenaQueryStyleHoverCandidateV0],
) -> Vec<OmenaQueryStyleDiagnosticV0> {
    summarize_omena_query_style_diagnostics_for_file_with_deep_analysis(
        document_uri,
        text,
        candidates,
        true,
    )
    .diagnostics
    .into_iter()
    .filter(|diagnostic| {
        matches!(
            diagnostic.code,
            "rgFlowRelevantOperator"
                | "categoricalCascadeEvidenceInconsistency"
                | "cascadeSmtViolation"
        )
    })
    .collect()
}

fn lsp_diagnostic_severity(query_severity: &str, configured_severity: u8) -> u8 {
    if configured_severity != 2 {
        return configured_severity;
    }
    match query_severity {
        "error" => 1,
        "warning" => 2,
        "information" => 3,
        "hint" => 4,
        _ => configured_severity,
    }
}

fn resolve_lsp_code_actions(state: &LspShellState, params: Option<&Value>) -> Value {
    let diagnostics = params
        .and_then(|value| value.get("context"))
        .and_then(|value| value.get("diagnostics"))
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or(&[]);

    let mut actions: Vec<Value> = diagnostics
        .iter()
        .enumerate()
        .filter_map(|(index, diagnostic)| {
            let payload = diagnostic
                .pointer("/data/createCustomProperty")
                .and_then(Value::as_object)?;
            let uri = payload.get("uri").and_then(Value::as_str)?;
            let range = payload.get("range")?;
            let new_text = payload.get("newText").and_then(Value::as_str)?;
            let property_name = payload.get("propertyName").and_then(Value::as_str)?;
            let mut changes = serde_json::Map::new();
            changes.insert(
                uri.to_string(),
                json!([
                    {
                        "range": range,
                        "newText": new_text,
                    },
                ]),
            );

            Some(json!({
                "title": format!("Add '{}' to {}", property_name, file_label_from_uri(uri)),
                "kind": "quickfix",
                "diagnostics": [diagnostic],
                "edit": {
                    "changes": Value::Object(changes),
                },
                "data": {
                    "source": "omenaQueryStyleDiagnosticsForFile",
                    "diagnosticIndex": index,
                },
            }))
        })
        .chain(diagnostics.iter().enumerate().filter_map(|(index, diagnostic)| {
            let payload = diagnostic
                .pointer("/data/createSelector")
                .and_then(Value::as_object)?;
            let uri = payload.get("uri").and_then(Value::as_str)?;
            let range = payload.get("range")?;
            let new_text = payload.get("newText").and_then(Value::as_str)?;
            let selector_name = payload.get("selectorName").and_then(Value::as_str)?;
            let mut changes = serde_json::Map::new();
            changes.insert(
                uri.to_string(),
                json!([
                    {
                        "range": range,
                        "newText": new_text,
                    },
                ]),
            );

            Some(json!({
                "title": format!("Add '.{}' to {}", selector_name, file_label_from_uri(uri)),
                "kind": "quickfix",
                "diagnostics": [diagnostic],
                "edit": {
                    "changes": Value::Object(changes),
                },
                "data": {
                    "source": "omenaQuerySourceSyntaxIndex",
                    "diagnosticIndex": index,
                },
            }))
        }))
        .collect();

    actions.extend(resolve_lsp_suppression_code_actions(
        state,
        params,
        diagnostics,
    ));

    if diagnostics.is_empty() {
        actions.extend(resolve_lsp_refactor_code_actions(state, params));
    }

    if actions.is_empty() {
        Value::Null
    } else {
        json!(actions)
    }
}

fn resolve_lsp_suppression_code_actions(
    state: &LspShellState,
    params: Option<&Value>,
    diagnostics: &[Value],
) -> Vec<Value> {
    let document_uri = document_uri_from_params(params);
    let Some(document) = state.document(document_uri.as_str()) else {
        return Vec::new();
    };
    if !is_style_document_uri(document.uri.as_str()) {
        return Vec::new();
    }

    let mut actions = Vec::new();
    for (index, diagnostic) in diagnostics.iter().enumerate() {
        let Some(code) = diagnostic.get("code").and_then(Value::as_str) else {
            continue;
        };
        let Some(line) = diagnostic
            .pointer("/range/start/line")
            .and_then(Value::as_u64)
            .and_then(|line| usize::try_from(line).ok())
        else {
            continue;
        };
        let character = diagnostic
            .pointer("/range/start/character")
            .and_then(Value::as_u64)
            .and_then(|character| usize::try_from(character).ok())
            .unwrap_or(0);

        let indent = source_line_indent(document.text.as_str(), line);
        let insert_range = json!({
            "start": {
                "line": line,
                "character": 0,
            },
            "end": {
                "line": line,
                "character": 0,
            },
        });
        let mut changes = serde_json::Map::new();
        changes.insert(
            document.uri.clone(),
            json!([
                {
                    "range": insert_range,
                    "newText": format!(
                        "{indent}/* omena-ignore-next-line {code} [reason: 'TODO'] */\n"
                    ),
                },
            ]),
        );

        actions.push(json!({
            "title": "Suppress this diagnostic on the next line",
            "kind": "quickfix",
            "diagnostics": [diagnostic],
            "edit": {
                "changes": Value::Object(changes),
            },
            "data": {
                "source": "omenaLspDiagnosticSuppressionCodeAction",
                "diagnosticIndex": index,
                "code": code,
            },
        }));

        if let Some(block_line) =
            enclosing_style_block_open_line(document.text.as_str(), line, character)
        {
            let block_indent = source_line_indent(document.text.as_str(), block_line);
            let block_insert_range = json!({
                "start": {
                    "line": block_line,
                    "character": 0,
                },
                "end": {
                    "line": block_line,
                    "character": 0,
                },
            });
            let mut block_changes = serde_json::Map::new();
            block_changes.insert(
                document.uri.clone(),
                json!([
                    {
                        "range": block_insert_range,
                        "newText": format!(
                            "{block_indent}/* omena-ignore {code} [reason: 'TODO'] */\n"
                        ),
                    },
                ]),
            );

            actions.push(json!({
                "title": "Suppress diagnostics in this block",
                "kind": "quickfix",
                "diagnostics": [diagnostic],
                "edit": {
                    "changes": Value::Object(block_changes),
                },
                "data": {
                    "source": "omenaLspDiagnosticSuppressionCodeAction",
                    "diagnosticIndex": index,
                    "code": code,
                    "scope": "block",
                },
            }));
        }
    }
    actions
}

fn source_line_indent(source: &str, line: usize) -> String {
    source
        .lines()
        .nth(line)
        .map(|text| {
            text.chars()
                .take_while(|character| character.is_whitespace())
                .collect()
        })
        .unwrap_or_default()
}

fn enclosing_style_block_open_line(source: &str, line: usize, character: usize) -> Option<usize> {
    let offset =
        protocol::byte_offset_for_parser_position(source, ParserPositionV0 { line, character })?;
    let prefix = source.get(..offset)?;
    let mut block_stack = Vec::new();
    let mut current_line = 0usize;
    let mut quote: Option<char> = None;
    let mut in_block_comment = false;
    let mut characters = prefix.chars().peekable();

    while let Some(character) = characters.next() {
        if character == '\n' {
            current_line += 1;
            continue;
        }
        if in_block_comment {
            if character == '*' && characters.peek() == Some(&'/') {
                characters.next();
                in_block_comment = false;
            }
            continue;
        }
        if let Some(quote_character) = quote {
            if character == '\\' {
                if characters.peek().is_some() {
                    characters.next();
                }
            } else if character == quote_character {
                quote = None;
            }
            continue;
        }
        if character == '/' && characters.peek() == Some(&'*') {
            characters.next();
            in_block_comment = true;
            continue;
        }
        match character {
            '"' | '\'' => quote = Some(character),
            '{' => block_stack.push(current_line),
            '}' => {
                block_stack.pop();
            }
            _ => {}
        }
    }

    block_stack.last().copied()
}

fn resolve_lsp_refactor_code_actions(state: &LspShellState, params: Option<&Value>) -> Vec<Value> {
    let document_uri = document_uri_from_params(params);
    let Some(document) = state.document(document_uri.as_str()) else {
        return Vec::new();
    };
    if !is_style_document_uri(document.uri.as_str()) {
        return Vec::new();
    }
    let Some(range) = params
        .and_then(|value| value.get("range"))
        .and_then(lsp_range_from_value)
    else {
        return Vec::new();
    };

    let style_sources = style_sources_from_open_documents(
        state,
        document.workspace_folder_uri.as_deref(),
        Some(document.uri.as_str()),
    );
    let actions = summarize_omena_query_style_refactor_code_actions(
        document.uri.as_str(),
        style_sources.as_slice(),
        document.text.as_str(),
        range,
        &[],
    )
    .actions;
    render_omena_query_lsp_code_actions(actions)
}

fn render_omena_query_lsp_code_actions(actions: Vec<OmenaQueryCodeActionV0>) -> Vec<Value> {
    actions
        .into_iter()
        .enumerate()
        .map(|(index, action)| {
            let mut changes_by_uri = BTreeMap::<String, Vec<Value>>::new();
            for edit in action.edits {
                changes_by_uri.entry(edit.uri).or_default().push(json!({
                    "range": edit.range,
                    "newText": edit.new_text,
                }));
            }

            let changes = changes_by_uri
                .into_iter()
                .map(|(uri, edits)| (uri, Value::Array(edits)))
                .collect::<serde_json::Map<_, _>>();

            json!({
                "title": action.title,
                "kind": action.kind,
                "edit": {
                    "changes": Value::Object(changes),
                },
                "data": {
                    "source": action.source,
                    "actionIndex": index,
                },
            })
        })
        .collect()
}

fn resolve_lsp_code_lens(state: &LspShellState, params: Option<&Value>) -> Value {
    let document_uri = document_uri_from_params(params);
    let Some(document) = state.document(document_uri.as_str()) else {
        return Value::Null;
    };
    let Some((_, candidates)) = style_hover_candidates_for_document(document) else {
        return Value::Null;
    };

    let mut lenses = Vec::new();
    let mut emitted_selectors = BTreeSet::new();
    let reference_locations_by_name = selector_reference_locations_by_name_from_open_documents(
        state,
        document.workspace_folder_uri.as_deref(),
        Some(document.uri.as_str()),
    );
    for candidate in candidates
        .iter()
        .filter(|candidate| candidate.kind == "selector")
    {
        if !emitted_selectors.insert(candidate.name.as_str()) {
            continue;
        }
        let locations = reference_locations_by_name
            .get(candidate.name.as_str())
            .cloned()
            .unwrap_or_default();
        if locations.is_empty() {
            continue;
        }
        let position = candidate.range.start;
        lenses.push(json!({
            "range": {
                "start": position,
                "end": position,
            },
            "command": {
                "title": reference_lens_title(locations.len()),
                "command": "editor.action.showReferences",
                "arguments": [
                    document.uri.as_str(),
                    position,
                    locations,
                ],
            },
        }));
    }
    lenses.sort_by_key(lsp_range_start_sort_key);

    if lenses.is_empty() {
        Value::Null
    } else {
        json!(lenses)
    }
}

fn selector_reference_locations_from_open_documents(
    state: &LspShellState,
    selector_name: &str,
    workspace_folder_uri: Option<&str>,
    target_style_uri: Option<&str>,
) -> Vec<Value> {
    let occurrence_index =
        source_selector_occurrence_index_from_open_documents(state, workspace_folder_uri);
    let query_target_style_uri = query_target_style_uri_for_matching(target_style_uri);
    summarize_omena_query_refs_for_class_from_occurrence_index(
        selector_name,
        query_target_style_uri.as_deref(),
        false,
        occurrence_index.definitions.as_slice(),
        &occurrence_index.source_selector_index,
    )
    .locations
    .into_iter()
    .map(|location| json!({ "uri": location.uri, "range": location.range }))
    .collect()
}

fn selector_reference_locations_by_name_from_open_documents(
    state: &LspShellState,
    workspace_folder_uri: Option<&str>,
    target_style_uri: Option<&str>,
) -> BTreeMap<String, Vec<Value>> {
    let mut locations_by_name: BTreeMap<String, Vec<Value>> = BTreeMap::new();
    let occurrence_index =
        source_selector_occurrence_index_from_open_documents(state, workspace_folder_uri);
    let query_target_style_uri = query_target_style_uri_for_matching(target_style_uri);
    for occurrence in occurrence_index
        .workspace_index
        .by_moniker
        .values()
        .flat_map(|occurrences| occurrences.iter())
    {
        if occurrence.family != Some(OmenaWorkspaceOccurrenceFamilyV0::CssModuleSelector)
            || !workspace_occurrence_matches_target_style(
                occurrence,
                query_target_style_uri.as_deref(),
            )
        {
            continue;
        }
        locations_by_name
            .entry(occurrence.name.clone())
            .or_default()
            .push(json!({
                "uri": occurrence.uri,
                "range": occurrence.range,
            }));
    }
    for locations in locations_by_name.values_mut() {
        locations.sort_by_key(location_sort_key);
        locations
            .dedup_by(|left, right| location_identity_key(left) == location_identity_key(right));
    }
    locations_by_name
}

fn style_symbol_definition_locations_from_documents(
    state: &LspShellState,
    document: &LspTextDocumentState,
    candidate: &LspStyleHoverCandidate,
) -> Vec<Value> {
    let monikers = style_symbol_monikers_for_candidate(state, document, candidate);
    let occurrence_index = style_symbol_occurrence_index_from_documents(
        state,
        document.workspace_folder_uri.as_deref(),
    );
    let mut locations = occurrences_for_monikers(&occurrence_index, &monikers)
        .into_iter()
        .filter(|occurrence| occurrence.role == OmenaWorkspaceOccurrenceRoleV0::Definition)
        .map(|occurrence| {
            json!({
                "uri": occurrence.uri,
                "range": occurrence.range,
            })
        })
        .collect::<Vec<_>>();
    locations.sort_by_key(location_sort_key);
    locations.dedup_by(|left, right| location_identity_key(left) == location_identity_key(right));
    locations
}

fn style_symbol_reference_locations_from_documents(
    state: &LspShellState,
    document: &LspTextDocumentState,
    candidate: &LspStyleHoverCandidate,
    include_declaration: bool,
) -> Vec<Value> {
    let monikers = style_symbol_monikers_for_candidate(state, document, candidate);
    let occurrence_index = style_symbol_occurrence_index_from_documents(
        state,
        document.workspace_folder_uri.as_deref(),
    );
    let mut locations = occurrences_for_monikers(&occurrence_index, &monikers)
        .into_iter()
        .filter(|occurrence| {
            occurrence.role == OmenaWorkspaceOccurrenceRoleV0::Reference
                || (include_declaration
                    && occurrence.role == OmenaWorkspaceOccurrenceRoleV0::Definition)
        })
        .map(|occurrence| {
            json!({
                "uri": occurrence.uri,
                "range": occurrence.range,
            })
        })
        .collect::<Vec<_>>();
    if include_declaration && is_sass_symbol_candidate_kind(candidate.kind) {
        locations.extend(
            sass_symbol_definitions_for_candidate(state, document, candidate)
                .into_iter()
                .map(|(uri, definition)| {
                    json!({
                        "uri": uri,
                        "range": definition.range,
                    })
                }),
        );
    }
    locations.sort_by_key(location_sort_key);
    locations.dedup_by(|left, right| location_identity_key(left) == location_identity_key(right));
    locations
}

fn resolve_style_symbol_rename(
    state: &LspShellState,
    document: &LspTextDocumentState,
    candidate: &LspStyleHoverCandidate,
    new_name: &str,
) -> Value {
    let monikers = style_symbol_monikers_for_candidate(state, document, candidate);
    let occurrence_index = style_symbol_occurrence_index_from_documents(
        state,
        document.workspace_folder_uri.as_deref(),
    );
    let mut seen = BTreeSet::new();
    let mut changes: BTreeMap<String, Vec<Value>> = BTreeMap::new();
    for occurrence in occurrences_for_monikers(&occurrence_index, &monikers) {
        if !occurrence.rename_target {
            continue;
        }
        let key = (
            occurrence.uri.clone(),
            occurrence.range.start.line,
            occurrence.range.start.character,
            occurrence.range.end.line,
            occurrence.range.end.character,
        );
        if !seen.insert(key) {
            continue;
        }
        let edit_uri = external_document_uri_for_query_uri(state, occurrence.uri.as_str());
        changes.entry(edit_uri).or_default().push(json!({
            "range": occurrence.range,
            "newText": new_name,
        }));
    }

    if changes.is_empty() {
        return Value::Null;
    }
    for edits in changes.values_mut() {
        edits.sort_by_key(lsp_range_start_sort_key);
    }
    json!({
        "changes": Value::Object(changes.into_iter().map(|(uri, edits)| (uri, json!(edits))).collect()),
    })
}

fn style_symbol_occurrence_index_from_documents(
    state: &LspShellState,
    workspace_folder_uri: Option<&str>,
) -> Arc<OmenaWorkspaceOccurrenceIndexV0> {
    workspace_occurrence_indexes_from_documents(state, workspace_folder_uri).workspace_index
}

fn style_symbol_workspace_occurrences_for_document(
    state: &LspShellState,
    document: &LspTextDocumentState,
    workspace_folder_uri: Option<&str>,
    dependency_digest: Option<&str>,
) -> Vec<OmenaWorkspaceOccurrenceV0> {
    let resolution_inputs =
        resolution_inputs_for_workspace_uri(state, document.workspace_folder_uri.as_deref());
    if let Some(shard) = load_workspace_occurrence_shard(
        document.workspace_folder_uri.as_deref(),
        document.uri.as_str(),
        document.language_id.as_str(),
        document.text_hash.as_str(),
        dependency_digest,
        &resolution_inputs,
    ) {
        return shard.occurrences;
    }

    let mut style_occurrences = Vec::new();
    let Some((_, candidates)) = style_hover_candidates_for_document(document) else {
        return Vec::new();
    };
    for candidate in candidates {
        if candidate.kind.starts_with("customProperty") {
            style_occurrences.push(style_symbol_occurrence_for_candidate(
                style_custom_property_moniker(workspace_folder_uri, candidate.name.as_str()),
                document.uri.as_str(),
                &candidate,
                "customProperty",
                style_symbol_role_for_candidate(&candidate),
            ));
            continue;
        }
        if !is_sass_symbol_candidate_kind(candidate.kind) {
            continue;
        }
        if is_sass_symbol_declaration_kind(candidate.kind) {
            style_occurrences.push(style_symbol_occurrence_for_candidate(
                style_sass_symbol_moniker_for_document(state, document, &candidate),
                document.uri.as_str(),
                &candidate,
                sass_symbol_kind_from_candidate_kind(candidate.kind).unwrap_or("symbol"),
                "definition",
            ));
            continue;
        }
        let definitions = sass_symbol_definitions_for_candidate(state, document, &candidate);
        if definitions.is_empty() {
            let moniker = if let Some(target) =
                external_sif_sass_symbol_target_for_candidate(state, document, &candidate)
            {
                style_external_sif_sass_symbol_moniker(&target)
            } else {
                style_unresolved_sass_symbol_moniker(workspace_folder_uri, &candidate)
            };
            style_occurrences.push(style_symbol_occurrence_for_candidate(
                moniker,
                document.uri.as_str(),
                &candidate,
                sass_symbol_kind_from_candidate_kind(candidate.kind).unwrap_or("symbol"),
                "reference",
            ));
            continue;
        }
        for (definition_uri, definition) in definitions {
            style_occurrences.push(style_symbol_occurrence_for_candidate(
                style_sass_symbol_moniker_for_uri(state, definition_uri.as_str(), &definition),
                document.uri.as_str(),
                &candidate,
                sass_symbol_kind_from_candidate_kind(candidate.kind).unwrap_or("symbol"),
                "reference",
            ));
        }
    }
    style_occurrences.sort();
    style_occurrences.dedup();
    let workspace_occurrences = style_occurrences
        .iter()
        .map(|occurrence| workspace_occurrence_from_style_symbol_occurrence(document, occurrence))
        .collect::<Vec<_>>();
    store_workspace_occurrence_shard(
        document.workspace_folder_uri.as_deref(),
        document.uri.as_str(),
        document.language_id.as_str(),
        document.text_hash.as_str(),
        dependency_digest,
        &resolution_inputs,
        workspace_occurrences.as_slice(),
    );
    workspace_occurrences
}

fn source_candidate_selector_names(
    candidate: &LspStyleHoverCandidate,
    definitions: &[(String, LspStyleHoverCandidate)],
    target_style_uri: Option<&str>,
) -> Vec<String> {
    let query_definitions = definitions
        .iter()
        .map(|(uri, definition)| query_style_selector_definition_for_matching(uri, definition))
        .collect::<Vec<_>>();
    let query_target_style_uri = query_target_style_uri_for_matching(target_style_uri);
    resolve_omena_query_source_candidate_selector_names(
        &query_source_selector_candidate_for_matching(candidate),
        query_definitions.as_slice(),
        query_target_style_uri.as_deref(),
    )
}

fn sass_symbol_definitions_for_candidate(
    state: &LspShellState,
    document: &LspTextDocumentState,
    candidate: &LspStyleHoverCandidate,
) -> Vec<(String, LspStyleHoverCandidate)> {
    let Some(symbol_kind) = sass_symbol_kind_from_candidate_kind(candidate.kind) else {
        return Vec::new();
    };
    if is_sass_symbol_declaration_kind(candidate.kind) {
        return vec![(document.uri.clone(), candidate.clone())];
    }

    let mut definitions = if candidate.namespace.is_none() {
        sass_symbol_declarations_in_document(document, symbol_kind, candidate)
    } else {
        Vec::new()
    };
    if candidate.namespace.is_none() && !definitions.is_empty() {
        return definitions;
    }

    for target_uri in sass_module_target_uris_for_candidate(state, document, candidate) {
        definitions.extend(sass_symbol_declarations_for_uri(
            state,
            target_uri.as_str(),
            symbol_kind,
            candidate,
        ));
    }
    definitions.sort_by_key(|(uri, target)| {
        (
            uri.clone(),
            target.range.start.line,
            target.range.start.character,
        )
    });
    definitions.dedup_by(|left, right| {
        left.0 == right.0
            && left.1.kind == right.1.kind
            && left.1.name == right.1.name
            && left.1.range == right.1.range
    });
    definitions
}

fn sass_symbol_declarations_for_uri(
    state: &LspShellState,
    target_uri: &str,
    symbol_kind: &str,
    candidate: &LspStyleHoverCandidate,
) -> Vec<(String, LspStyleHoverCandidate)> {
    sass_symbol_declarations_for_uri_with_visited(
        state,
        target_uri,
        symbol_kind,
        candidate,
        &mut BTreeSet::new(),
    )
}

fn sass_symbol_declarations_for_uri_with_visited(
    state: &LspShellState,
    target_uri: &str,
    symbol_kind: &str,
    candidate: &LspStyleHoverCandidate,
    visited: &mut BTreeSet<String>,
) -> Vec<(String, LspStyleHoverCandidate)> {
    if let Some(target_document) = state.document(target_uri) {
        return sass_symbol_declarations_with_forwards(
            state,
            target_document,
            symbol_kind,
            candidate,
            visited,
        );
    }
    let Some(target_document) = style_document_from_disk_for_uri(state, target_uri) else {
        return Vec::new();
    };
    sass_symbol_declarations_with_forwards(state, &target_document, symbol_kind, candidate, visited)
}

fn style_document_from_disk_for_uri(
    state: &LspShellState,
    uri: &str,
) -> Option<LspTextDocumentState> {
    let text = style_text_for_uri(state, uri)?;
    let workspace_folder_uri = resolve_workspace_folder_uri(state, uri);
    let resolution_inputs =
        resolution_inputs_for_workspace_uri(state, workspace_folder_uri.as_deref());
    Some(lsp_text_document_state(
        uri.to_string(),
        workspace_folder_uri,
        StyleLanguage::from_module_path(uri)
            .map(style_language_label)
            .unwrap_or("unknown")
            .to_string(),
        0,
        text,
        &resolution_inputs,
    ))
}

fn sass_symbol_declarations_in_document(
    document: &LspTextDocumentState,
    symbol_kind: &str,
    candidate: &LspStyleHoverCandidate,
) -> Vec<(String, LspStyleHoverCandidate)> {
    let query_candidates = document
        .style_candidates
        .iter()
        .map(query_style_hover_candidate_from_lsp)
        .collect::<Vec<_>>();
    resolve_omena_query_sass_symbol_declarations(
        query_candidates.as_slice(),
        symbol_kind,
        candidate.name.as_str(),
    )
    .into_iter()
    .map(lsp_style_hover_candidate_from_query)
    .map(|target| (document.uri.clone(), target))
    .collect()
}

fn sass_module_target_uris_for_candidate(
    state: &LspShellState,
    document: &LspTextDocumentState,
    candidate: &LspStyleHoverCandidate,
) -> Vec<String> {
    let Some(sources) =
        summarize_omena_query_sass_module_sources(document.uri.as_str(), document.text.as_str())
    else {
        return Vec::new();
    };
    let mut uris = Vec::new();
    for source in resolve_omena_query_sass_module_use_sources_for_candidate(
        &sources,
        candidate.namespace.as_deref(),
    ) {
        if let Some(uri) = resolve_lsp_style_uri_for_specifier(state, document, source.as_str()) {
            uris.push(uri);
        }
    }
    for forward_source in resolve_omena_query_sass_forward_sources(&sources) {
        if let Some(uri) =
            resolve_lsp_style_uri_for_specifier(state, document, forward_source.as_str())
        {
            uris.push(uri.clone());
            if let Some(target_document) = state.document(uri.as_str()) {
                uris.extend(sass_forward_module_target_uris(
                    state,
                    target_document,
                    &mut BTreeSet::new(),
                ));
            }
        }
    }
    uris.sort();
    uris.dedup();
    uris
}

fn sass_symbol_declarations_with_forwards(
    state: &LspShellState,
    document: &LspTextDocumentState,
    symbol_kind: &str,
    candidate: &LspStyleHoverCandidate,
    visited: &mut BTreeSet<String>,
) -> Vec<(String, LspStyleHoverCandidate)> {
    if !visited.insert(document.uri.clone()) {
        return Vec::new();
    }
    let mut definitions = sass_symbol_declarations_in_document(document, symbol_kind, candidate);
    if summarize_omena_query_sass_module_sources(document.uri.as_str(), document.text.as_str())
        .is_none()
    {
        return definitions;
    }
    for forward_edge in sass_forward_edges_for_document(document) {
        let Some(uri) =
            resolve_lsp_style_uri_for_specifier(state, document, forward_edge.source.as_str())
        else {
            continue;
        };
        let Some(target_candidate) =
            forward_edge.private_candidate_for_forwarded_public_candidate(candidate)
        else {
            continue;
        };
        definitions.extend(sass_symbol_declarations_for_uri_with_visited(
            state,
            uri.as_str(),
            symbol_kind,
            &target_candidate,
            visited,
        ));
    }
    definitions
}

fn sass_forward_module_target_uris(
    state: &LspShellState,
    document: &LspTextDocumentState,
    visited: &mut BTreeSet<String>,
) -> Vec<String> {
    if !visited.insert(document.uri.clone()) {
        return Vec::new();
    }
    let Some(sources) =
        summarize_omena_query_sass_module_sources(document.uri.as_str(), document.text.as_str())
    else {
        return Vec::new();
    };
    let mut uris = Vec::new();
    for forward_source in resolve_omena_query_sass_forward_sources(&sources) {
        if let Some(uri) =
            resolve_lsp_style_uri_for_specifier(state, document, forward_source.as_str())
        {
            uris.push(uri.clone());
        }
    }
    uris.sort();
    uris.dedup();
    uris
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SassForwardEdgeForLsp {
    source: String,
    forward_prefix: Option<String>,
}

impl SassForwardEdgeForLsp {
    fn private_candidate_for_forwarded_public_candidate(
        &self,
        candidate: &LspStyleHoverCandidate,
    ) -> Option<LspStyleHoverCandidate> {
        let private_name =
            unapply_sass_forward_prefix(self.forward_prefix.as_deref(), &candidate.name)?;
        let mut target = candidate.clone();
        target.name = private_name;
        target.namespace = None;
        Some(target)
    }
}

fn sass_forward_edges_for_document(document: &LspTextDocumentState) -> Vec<SassForwardEdgeForLsp> {
    let facts = summarize_omena_query_omena_parser_style_facts(
        document.text.as_str(),
        query_style_dialect_for_uri(document.uri.as_str()),
    );
    facts
        .sass_module_edges
        .into_iter()
        .filter(|edge| edge.kind == "sassForward")
        .map(|edge| SassForwardEdgeForLsp {
            source: edge.source,
            forward_prefix: edge.forward_prefix,
        })
        .collect()
}

fn unapply_sass_forward_prefix(prefix: Option<&str>, exposed_name: &str) -> Option<String> {
    let Some(prefix) = prefix else {
        return Some(exposed_name.to_string());
    };
    if let Some(star_offset) = prefix.find('*') {
        let before = prefix.get(..star_offset).unwrap_or_default();
        let after = prefix
            .get(star_offset + '*'.len_utf8()..)
            .unwrap_or_default();
        let without_before = exposed_name.strip_prefix(before)?;
        let without_after = if after.is_empty() {
            without_before
        } else {
            without_before.strip_suffix(after)?
        };
        return Some(without_after.to_string());
    }
    exposed_name
        .strip_prefix(prefix)
        .map(str::to_string)
        .filter(|name| !name.is_empty())
}

fn query_style_dialect_for_uri(uri: &str) -> OmenaParserStyleDialect {
    let lower = uri.to_ascii_lowercase();
    if lower.ends_with(".sass") || lower.ends_with(".sass?module") {
        OmenaParserStyleDialect::Sass
    } else if lower.ends_with(".scss") || lower.ends_with(".module.scss") {
        OmenaParserStyleDialect::Scss
    } else if lower.ends_with(".less") || lower.ends_with(".module.less") {
        OmenaParserStyleDialect::Less
    } else {
        OmenaParserStyleDialect::Css
    }
}

fn resolve_lsp_style_uri_for_specifier(
    state: &LspShellState,
    document: &LspTextDocumentState,
    specifier: &str,
) -> Option<String> {
    let resolution_inputs =
        resolution_inputs_for_workspace_uri(state, document.workspace_folder_uri.as_deref());
    resolve_omena_query_style_uri_for_specifier_with_resolution_inputs(
        document.uri.as_str(),
        document.workspace_folder_uri.as_deref(),
        specifier,
        &resolution_inputs,
    )
}

fn reference_lens_title(count: usize) -> String {
    if count == 1 {
        "1 reference".to_string()
    } else {
        format!("{count} references")
    }
}

fn resolve_lsp_prepare_rename(state: &LspShellState, params: Option<&Value>) -> Value {
    if let Some((_, candidate)) = source_selector_candidate_for_params(state, params) {
        return json!({
            "range": candidate.range,
            "placeholder": candidate.name,
        });
    }

    let Some((_, candidate, _)) = style_candidates_for_params(state, params) else {
        return Value::Null;
    };

    json!({
        "range": candidate.range,
        "placeholder": rename_placeholder(&candidate),
    })
}

fn resolve_lsp_rename(state: &LspShellState, params: Option<&Value>) -> Value {
    let Some(new_name) = params
        .and_then(|value| value.get("newName"))
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
    else {
        return Value::Null;
    };
    if let Some((document_uri, candidate)) = source_selector_candidate_for_params(state, params) {
        let workspace_folder_uri = state
            .document(document_uri.as_str())
            .and_then(|document| document.workspace_folder_uri.as_deref());
        return resolve_selector_rename(
            state,
            workspace_folder_uri,
            candidate.target_style_uri.as_deref(),
            candidate.name.as_str(),
            new_name,
        );
    }

    let Some((document_uri, candidate, _candidates)) = style_candidates_for_params(state, params)
    else {
        return Value::Null;
    };

    if candidate.kind == "selector" {
        let workspace_folder_uri = state
            .document(document_uri.as_str())
            .and_then(|document| document.workspace_folder_uri.as_deref());
        return resolve_selector_rename(
            state,
            workspace_folder_uri,
            Some(document_uri.as_str()),
            candidate.name.as_str(),
            new_name,
        );
    }

    if candidate.kind.starts_with("customProperty") || is_sass_symbol_candidate_kind(candidate.kind)
    {
        let Some(document) = state.document(document_uri.as_str()) else {
            return Value::Null;
        };
        return resolve_style_symbol_rename(state, document, &candidate, new_name);
    }

    Value::Null
}

fn style_candidates_for_params(
    state: &LspShellState,
    params: Option<&Value>,
) -> Option<(String, LspStyleHoverCandidate, Vec<LspStyleHoverCandidate>)> {
    let document_uri = document_uri_from_params(params);
    let position = lsp_position_from_params(params)?;
    let document = state.document(document_uri.as_str())?;
    let (_, candidates) = style_hover_candidates_for_document(document)?;
    let candidate = candidates
        .iter()
        .find(|candidate| parser_range_contains_position(&candidate.range, position))?
        .clone();
    Some((document_uri, candidate, candidates))
}

fn rename_placeholder(candidate: &LspStyleHoverCandidate) -> &str {
    candidate.name.as_str()
}

fn resolve_lsp_hover(state: &LspShellState, params: Option<&Value>) -> Value {
    let document_uri = document_uri_from_params(params);
    if let Some(document) = state.document(document_uri.as_str())
        && !is_style_document_uri(document.uri.as_str())
    {
        return resolve_source_lsp_hover(state, document, params);
    }

    let candidates = resolve_style_hover_candidates(state, params);
    let Some(candidate) = candidates.candidates.first() else {
        return Value::Null;
    };
    let Some(document) = state.document(document_uri.as_str()) else {
        return Value::Null;
    };
    if is_sass_symbol_reference_kind(candidate.kind)
        && let Some((target_uri, target)) =
            sass_symbol_definitions_for_candidate(state, document, candidate)
                .into_iter()
                .next()
        && let Some(target_text) = style_text_for_uri(state, target_uri.as_str())
    {
        let mut response = json!({
            "contents": {
                "kind": "markdown",
                "value": render_style_hover_candidate_markdown_for_workspace(
                    state,
                    target_uri.as_str(),
                    target_text.as_str(),
                    &target,
                ),
            },
            "range": candidate.range,
        });
        if let Some(target_document) = state.document(target_uri.as_str()) {
            let provider_feedback =
                current_provider_tier_feedback_data(target_document, "textDocument/hover");
            attach_provider_tier_feedback(&mut response, provider_feedback.as_ref());
        }
        return response;
    }
    if is_sass_symbol_reference_kind(candidate.kind)
        && let Some(target) =
            external_sif_sass_symbol_target_for_candidate(state, document, candidate)
    {
        return json!({
            "contents": {
                "kind": "markdown",
                "value": render_external_sif_sass_symbol_hover_markdown(&target),
            },
            "range": candidate.range,
        });
    }

    let mut response = json!({
        "contents": {
            "kind": "markdown",
            "value": render_style_hover_candidate_markdown_for_workspace(
                state,
                document.uri.as_str(),
                document.text.as_str(),
                candidate,
            ),
        },
        "range": candidate.range,
    });
    let provider_feedback = current_provider_tier_feedback_data(document, "textDocument/hover");
    attach_provider_tier_feedback(&mut response, provider_feedback.as_ref());
    response
}

fn resolve_lsp_hover_trace(state: &LspShellState, params: Option<&Value>) -> Value {
    let document_uri = document_uri_from_params(params);
    let Some(position) = lsp_position_from_params(params) else {
        return empty_hover_trace(document_uri, None, "unknown", None, "missingPosition");
    };
    let Some(document) = state.document(document_uri.as_str()) else {
        return empty_hover_trace(
            document_uri,
            None,
            "unknown",
            Some(position),
            "documentNotIndexed",
        );
    };

    if is_style_document_uri(document.uri.as_str()) {
        return resolve_style_lsp_hover_trace(state, document, position);
    }
    resolve_source_lsp_hover_trace(state, document, position)
}

fn resolve_style_lsp_hover_trace(
    state: &LspShellState,
    document: &LspTextDocumentState,
    position: ParserPositionV0,
) -> Value {
    let Some((language, candidates)) = style_hover_candidates_for_document(document) else {
        return empty_hover_trace(
            document.uri.clone(),
            document.workspace_folder_uri.clone(),
            "style",
            Some(position),
            "styleDocumentNotIndexed",
        );
    };
    let matched = candidates
        .iter()
        .filter(|candidate| parser_range_contains_position(&candidate.range, position))
        .cloned()
        .collect::<Vec<_>>();
    let Some(candidate) = matched.first() else {
        return json!({
            "schemaVersion": "0",
            "product": "omena-lsp-server.explain-hover-trace",
            "documentUri": document.uri.as_str(),
            "workspaceFolderUri": document.workspace_folder_uri.as_deref(),
            "fileKind": "style",
            "language": language,
            "queryPosition": position,
            "matched": false,
            "reason": "noStyleCandidateAtPosition",
            "candidateCount": 0,
            "definitionCount": 0,
            "candidates": [],
            "definitions": [],
            "resolutionPath": ["styleHoverCandidates"],
            "readySurfaces": ["explainHoverTraceRpc", "styleHoverCandidates"],
        });
    };
    let definitions =
        style_hover_trace_definitions(state, document, candidate, candidates.as_slice());
    let rendered_markdown =
        render_source_hover_definitions_markdown(state, definitions.as_slice()).unwrap_or_default();

    json!({
        "schemaVersion": "0",
        "product": "omena-lsp-server.explain-hover-trace",
        "documentUri": document.uri.as_str(),
        "workspaceFolderUri": document.workspace_folder_uri.as_deref(),
        "fileKind": "style",
        "language": language,
        "queryPosition": position,
        "matched": true,
        "reason": "styleCandidateResolved",
        "candidateCount": matched.len(),
        "definitionCount": definitions.len(),
        "candidates": matched,
        "definitions": hover_trace_definition_values(definitions.as_slice()),
        "renderedMarkdown": rendered_markdown,
        "resolutionPath": ["styleHoverCandidates", "styleDefinitionResolver", "hoverMarkdownRenderer"],
        "readySurfaces": ["explainHoverTraceRpc", "styleHoverCandidates", "hoverMarkdownRenderer"],
    })
}

fn resolve_source_lsp_hover_trace(
    state: &LspShellState,
    document: &LspTextDocumentState,
    position: ParserPositionV0,
) -> Value {
    if let Some(trace) = source_domain_reference_trace_at_position(document, position) {
        return trace;
    }

    let resolution = resolve_source_provider_candidates(state, document);
    let matched = resolution
        .matched
        .into_iter()
        .filter(|candidate| parser_range_contains_position(&candidate.range, position))
        .collect::<Vec<_>>();
    let unresolved = resolution
        .unresolved
        .into_iter()
        .filter(|candidate| parser_range_contains_position(&candidate.range, position))
        .collect::<Vec<_>>();
    if matched.is_empty() && unresolved.is_empty() {
        return empty_hover_trace(
            document.uri.clone(),
            document.workspace_folder_uri.clone(),
            "source",
            Some(position),
            "noSourceCandidateAtPosition",
        );
    }

    let definitions = style_selector_definitions_for_source_candidates(
        state,
        matched.as_slice(),
        document.workspace_folder_uri.as_deref(),
    );
    let rendered_markdown =
        render_source_hover_definitions_markdown(state, definitions.as_slice()).unwrap_or_default();

    json!({
        "schemaVersion": "0",
        "product": "omena-lsp-server.explain-hover-trace",
        "documentUri": document.uri.as_str(),
        "workspaceFolderUri": document.workspace_folder_uri.as_deref(),
        "fileKind": "source",
        "languageId": document.language_id.as_str(),
        "queryPosition": position,
        "matched": !matched.is_empty(),
        "reason": if matched.is_empty() { "sourceCandidateUnresolved" } else { "sourceCandidateResolved" },
        "matchedCandidateCount": matched.len(),
        "unresolvedCandidateCount": unresolved.len(),
        "definitionCount": definitions.len(),
        "candidates": matched,
        "unresolvedCandidates": unresolved,
        "definitions": hover_trace_definition_values(definitions.as_slice()),
        "renderedMarkdown": rendered_markdown,
        "resolutionPath": ["sourceSyntaxIndex", "sourceProviderCandidateResolution", "styleSelectorDefinitionResolver", "hoverMarkdownRenderer"],
        "readySurfaces": ["explainHoverTraceRpc", "sourceSyntaxIndex", "sourceProviderCandidateResolution", "hoverMarkdownRenderer"],
    })
}

fn empty_hover_trace(
    document_uri: String,
    workspace_folder_uri: Option<String>,
    file_kind: &'static str,
    query_position: Option<ParserPositionV0>,
    reason: &'static str,
) -> Value {
    json!({
        "schemaVersion": "0",
        "product": "omena-lsp-server.explain-hover-trace",
        "documentUri": document_uri,
        "workspaceFolderUri": workspace_folder_uri,
        "fileKind": file_kind,
        "queryPosition": query_position,
        "matched": false,
        "reason": reason,
        "candidateCount": 0,
        "definitionCount": 0,
        "candidates": [],
        "definitions": [],
        "resolutionPath": [],
        "readySurfaces": ["explainHoverTraceRpc"],
    })
}

fn style_hover_trace_definitions(
    state: &LspShellState,
    document: &LspTextDocumentState,
    candidate: &LspStyleHoverCandidate,
    candidates: &[LspStyleHoverCandidate],
) -> Vec<(String, LspStyleHoverCandidate)> {
    if is_sass_symbol_reference_kind(candidate.kind) {
        return sass_symbol_definitions_for_candidate(state, document, candidate);
    }
    if candidate.kind == "customPropertyReference"
        && let Some(target) = candidates.iter().find(|target| {
            target.kind == "customPropertyDeclaration" && target.name == candidate.name
        })
    {
        return vec![(document.uri.clone(), target.clone())];
    }
    vec![(document.uri.clone(), candidate.clone())]
}

fn hover_trace_definition_values(definitions: &[(String, LspStyleHoverCandidate)]) -> Vec<Value> {
    definitions
        .iter()
        .map(|(uri, definition)| {
            json!({
                "uri": uri,
                "kind": definition.kind,
                "name": definition.name,
                "range": definition.range,
                "source": definition.source,
                "targetStyleUri": definition.target_style_uri,
                "namespace": definition.namespace,
            })
        })
        .collect()
}

fn resolve_source_lsp_hover(
    state: &LspShellState,
    document: &LspTextDocumentState,
    params: Option<&Value>,
) -> Value {
    let Some(position) = lsp_position_from_params(params) else {
        return Value::Null;
    };
    if let Some((range, value)) = source_domain_reference_hover_at_position(document, position) {
        return json!({
            "contents": {
                "kind": "markdown",
                "value": value,
            },
            "range": range,
        });
    }
    let candidates = source_selector_candidates_at_position(state, document, position);
    let Some(candidate) = candidates.first() else {
        return Value::Null;
    };
    let definitions = style_selector_definitions_for_source_candidates(
        state,
        candidates.as_slice(),
        document.workspace_folder_uri.as_deref(),
    );
    let value = render_source_hover_definitions_markdown(state, definitions.as_slice())
        .unwrap_or_else(|| format!("**`.{}`**", candidate.name));

    let mut response = json!({
        "contents": {
            "kind": "markdown",
            "value": value,
        },
        "range": candidate.range,
    });
    let provider_feedback =
        provider_tier_feedback_for_hover_definitions(state, definitions.as_slice());
    attach_provider_tier_feedback(&mut response, provider_feedback.as_ref());
    response
}

fn source_domain_reference_at_position(
    document: &LspTextDocumentState,
    position: ParserPositionV0,
) -> Option<&SourceDomainClassReferenceFact> {
    let offset = byte_offset_for_parser_position(document.text.as_str(), position)?;
    document
        .source_syntax_index
        .domain_class_references
        .iter()
        .find(|reference| offset >= reference.byte_span.start && offset <= reference.byte_span.end)
}

fn source_domain_reference_trace_at_position(
    document: &LspTextDocumentState,
    position: ParserPositionV0,
) -> Option<Value> {
    let reference = source_domain_reference_at_position(document, position)?;
    let options = source_domain_reference_option_names(&document.source_syntax_index, reference);
    let current = source_domain_reference_current_option(reference);
    let validity = source_domain_reference_validity(reference, options.as_slice());
    let range = parser_range_for_byte_span(document.text.as_str(), reference.byte_span);

    Some(json!({
        "schemaVersion": "0",
        "product": "omena-lsp-server.explain-hover-trace",
        "documentUri": document.uri.as_str(),
        "workspaceFolderUri": document.workspace_folder_uri.as_deref(),
        "fileKind": "source",
        "languageId": document.language_id.as_str(),
        "queryPosition": position,
        "matched": true,
        "reason": "domainClassReferenceResolved",
        "hoverKind": "domainClassReference",
        "range": range,
        "sourceOwner": reference.owner_name,
        "domain": reference.domain,
        "axisName": reference.axis_name,
        "optionName": reference.option_name,
        "prefix": reference.prefix,
        "currentOption": current,
        "validity": validity,
        "knownOptions": options,
        "candidateCount": 1,
        "definitionCount": 0,
        "candidates": [],
        "definitions": [],
        "renderedMarkdown": render_source_domain_reference_hover_text(reference, options.as_slice()),
        "resolutionPath": ["sourceSyntaxIndex", "classValueUniverseProvider", "sourceDomainReferenceHover"],
        "readySurfaces": ["explainHoverTraceRpc", "sourceSyntaxIndex", "classValueUniverseProvider"],
    }))
}

fn source_domain_reference_hover_at_position(
    document: &LspTextDocumentState,
    position: ParserPositionV0,
) -> Option<(ParserRangeV0, String)> {
    let reference = source_domain_reference_at_position(document, position)?;
    let options = source_domain_reference_option_names(&document.source_syntax_index, reference);
    Some((
        parser_range_for_byte_span(document.text.as_str(), reference.byte_span),
        render_source_domain_reference_hover_text(reference, options.as_slice()),
    ))
}

fn source_domain_reference_current_option(reference: &SourceDomainClassReferenceFact) -> &str {
    reference
        .option_name
        .as_deref()
        .or(reference.prefix.as_deref())
        .unwrap_or("*")
}

fn source_domain_reference_validity(
    reference: &SourceDomainClassReferenceFact,
    options: &[String],
) -> &'static str {
    reference
        .option_name
        .as_ref()
        .map(|option| {
            if options.iter().any(|known| known == option) {
                "known option"
            } else {
                "unknown option"
            }
        })
        .unwrap_or("prefix option")
}

fn render_source_domain_reference_hover_text(
    reference: &SourceDomainClassReferenceFact,
    options: &[String],
) -> String {
    let current = reference
        .option_name
        .as_deref()
        .or(reference.prefix.as_deref())
        .unwrap_or("*");
    let validity = source_domain_reference_validity(reference, options);
    let known_options = if options.is_empty() {
        "No known options indexed.".to_string()
    } else {
        format!("Known options: `{}`.", options.join("`, `"))
    };
    format!(
        "**`{}.{}.{}`**\n\n{} from `{}`.\n\n{}",
        reference.owner_name,
        reference.axis_name,
        current,
        validity,
        reference.domain,
        known_options
    )
}

fn resolve_source_lsp_definition(
    state: &LspShellState,
    document: &LspTextDocumentState,
    position: ParserPositionV0,
) -> Value {
    let candidates = source_selector_candidates_at_position(state, document, position);
    if candidates.is_empty() {
        return Value::Null;
    };
    let definitions = style_selector_definitions_for_source_candidates(
        state,
        candidates.as_slice(),
        document.workspace_folder_uri.as_deref(),
    );
    if definitions.is_empty() {
        return Value::Null;
    }

    json!(
        definitions
            .into_iter()
            .map(|(uri, definition)| json!({ "uri": uri, "range": definition.range }))
            .collect::<Vec<_>>()
    )
}

fn resolve_source_lsp_references(
    state: &LspShellState,
    document: &LspTextDocumentState,
    position: ParserPositionV0,
    params: Option<&Value>,
) -> Value {
    let candidates = source_selector_candidates_at_position(state, document, position);
    if candidates.is_empty() {
        return Value::Null;
    };
    let include_declaration = include_declaration_from_params(params);
    let mut locations = Vec::new();
    if include_declaration {
        locations.extend(
            style_selector_definitions_for_source_candidates(
                state,
                candidates.as_slice(),
                document.workspace_folder_uri.as_deref(),
            )
            .into_iter()
            .map(|(uri, definition)| json!({ "uri": uri, "range": definition.range })),
        );
    }
    for candidate in candidates {
        if candidate.kind == "sourceSelectorPrefixReference" {
            let definitions = style_selector_definitions_from_open_documents(
                state,
                "",
                document.workspace_folder_uri.as_deref(),
            );
            for selector_name in source_candidate_selector_names(
                &candidate,
                definitions.as_slice(),
                candidate.target_style_uri.as_deref(),
            ) {
                locations.extend(selector_reference_locations_from_open_documents(
                    state,
                    selector_name.as_str(),
                    document.workspace_folder_uri.as_deref(),
                    candidate.target_style_uri.as_deref(),
                ));
            }
        } else {
            locations.extend(selector_reference_locations_from_open_documents(
                state,
                candidate.name.as_str(),
                document.workspace_folder_uri.as_deref(),
                candidate.target_style_uri.as_deref(),
            ));
        }
    }
    locations.sort_by_key(location_sort_key);
    locations.dedup();

    if locations.is_empty() {
        Value::Null
    } else {
        json!(locations)
    }
}

fn resolve_source_lsp_completion(
    state: &LspShellState,
    document: &LspTextDocumentState,
    params: Option<&Value>,
) -> Value {
    let Some(position) = lsp_position_from_params(params) else {
        return Value::Null;
    };
    let Some(context) = source_completion_context_at_position(state, document, position) else {
        return Value::Null;
    };
    if !context.domain_option_names.is_empty() {
        let items = source_domain_option_completion_items(
            context.domain_option_names.as_slice(),
            context.value_prefix.as_deref(),
        )
        .into_iter()
        .map(|item| lsp_completion_item_from_query("source", item, None))
        .collect::<Vec<_>>();
        return json!({
            "isIncomplete": false,
            "items": items,
        });
    }
    let inferred_target_style_uri = context.target_style_uri.clone().or_else(|| {
        source_selector_candidate_at_position(state, document, position)
            .and_then(|candidate| candidate.target_style_uri)
    });
    let target_style_uri = inferred_target_style_uri
        .as_deref()
        .map(|uri| external_document_uri_for_query_uri(state, uri));
    let style_sources = style_sources_from_open_documents(
        state,
        document.workspace_folder_uri.as_deref(),
        target_style_uri.as_deref(),
    );
    let resolution_inputs =
        resolution_inputs_for_workspace_uri(state, document.workspace_folder_uri.as_deref());

    let definitions = style_selector_definitions_from_open_documents(
        state,
        "",
        document.workspace_folder_uri.as_deref(),
    )
    .into_iter()
    .filter(|(uri, _)| {
        target_style_uri
            .as_deref()
            .is_none_or(|target_uri| file_uri_equivalent(target_uri, uri))
    })
    .collect::<Vec<_>>();
    let candidates = definitions
        .iter()
        .map(|(uri, definition)| {
            let file_uri = target_style_uri
                .as_deref()
                .filter(|target_uri| file_uri_equivalent(target_uri, uri.as_str()))
                .map(ToString::to_string)
                .unwrap_or_else(|| uri.clone());
            OmenaQueryCompletionCandidateV0 {
                file_uri,
                name: definition.name.clone(),
                kind: definition.kind,
                range: definition.range,
                source: definition.source,
                documentation: None,
            }
        })
        .collect::<Vec<_>>();
    let mut completion = summarize_omena_query_source_completion_at_position(
        document.uri.as_str(),
        position,
        candidates.as_slice(),
        target_style_uri.as_deref(),
        context.value_prefix.as_deref(),
        context.preferred_selector_names.as_slice(),
    );
    // Cascade documentation is attached lazily AFTER ranking/dedup and only for the
    // top-ranked items a completion list actually surfaces. The name-independent
    // narrowing inputs come from the memoized substrate (rfcs#63 E-ii) — fetched once
    // per request, reused across requests while the corpus is unchanged — so the
    // per-candidate work is the cheap per-name filter, not a whole-corpus collection.
    let mut narrowing_substrate = None;
    for item in completion
        .items
        .iter_mut()
        .take(SOURCE_COMPLETION_DOCUMENTATION_BUDGET)
    {
        if item.item_kind != "cssModuleSelector" || item.documentation.is_some() {
            continue;
        }
        let Some((uri, definition)) = definitions
            .iter()
            .find(|(_, definition)| definition.kind == "selector" && definition.name == item.label)
        else {
            continue;
        };
        let narrowing_substrate = narrowing_substrate.get_or_insert_with(|| {
            cascade_narrowing_substrate_for_style_sources(
                state,
                style_sources.as_slice(),
                &resolution_inputs,
            )
        });
        item.documentation = style_text_for_uri(state, uri.as_str()).and_then(|style_text| {
            summarize_omena_query_style_completion_candidate_documentation_for_workspace_file_with_substrate(
                uri.as_str(),
                style_sources.as_slice(),
                narrowing_substrate,
                definition.kind,
                definition.name.as_str(),
                definition.range.start,
            )
            .or_else(|| {
                summarize_omena_query_style_completion_candidate_documentation(
                    style_text.as_str(),
                    definition.kind,
                    definition.name.as_str(),
                    definition.range.start,
                )
            })
        });
    }
    let completion = completion;
    let provider_feedback = target_style_uri
        .as_deref()
        .and_then(|uri| state.document(uri))
        .and_then(|target_document| {
            current_provider_tier_feedback_data(target_document, "textDocument/completion")
        });
    let items: Vec<Value> = completion
        .items
        .into_iter()
        .map(|item| {
            lsp_completion_item_from_query(completion.file_kind, item, provider_feedback.as_ref())
        })
        .collect();

    json!({
        "isIncomplete": false,
        "items": items,
    })
}

fn current_provider_tier_feedback_data(
    document: &LspTextDocumentState,
    provider: &'static str,
) -> Option<Value> {
    let feedback = document
        .optimizing_tier_feedback
        .as_ref()
        .filter(|feedback| feedback.document_version == document.version)?;
    Some(json!({
        "product": "omena-lsp-server.provider-tier-feedback",
        "source": feedback.product,
        "provider": provider,
        "policy": feedback.policy,
        "consumer": "hoverCompletionProviderRequest",
        "tier": feedback.analyzed_graph.tier,
        "feedback": "analyzedGraphV0HotStylePrewarm",
        "documentVersion": feedback.document_version,
        "nodeCount": feedback.analyzed_graph.node_count,
        "edgeCount": feedback.analyzed_graph.edge_count,
    }))
}

fn provider_tier_feedback_for_hover_definitions(
    state: &LspShellState,
    definitions: &[(String, LspStyleHoverCandidate)],
) -> Option<Value> {
    definitions.iter().find_map(|(uri, _)| {
        state.document(uri.as_str()).and_then(|document| {
            current_provider_tier_feedback_data(document, "textDocument/hover")
        })
    })
}

fn attach_provider_tier_feedback(response: &mut Value, provider_feedback: Option<&Value>) {
    let Some(provider_feedback) = provider_feedback else {
        return;
    };
    let Some(response_object) = response.as_object_mut() else {
        return;
    };
    let data = response_object.entry("data").or_insert_with(|| json!({}));
    if let Some(data_object) = data.as_object_mut() {
        data_object.insert(
            "providerTierFeedback".to_string(),
            provider_feedback.clone(),
        );
    }
}

fn render_source_hover_definitions_markdown(
    state: &LspShellState,
    definitions: &[(String, LspStyleHoverCandidate)],
) -> Option<String> {
    let parts = definitions
        .iter()
        .filter_map(|(uri, definition)| {
            style_text_for_uri(state, uri).map(|text| {
                render_style_hover_candidate_markdown_for_workspace(
                    state,
                    uri.as_str(),
                    text.as_str(),
                    definition,
                )
            })
        })
        .collect::<Vec<_>>();
    if parts.is_empty() {
        None
    } else {
        Some(parts.join("\n\n---\n\n"))
    }
}

fn resolve_selector_rename(
    state: &LspShellState,
    workspace_folder_uri: Option<&str>,
    target_style_uri: Option<&str>,
    selector_name: &str,
    new_name: &str,
) -> Value {
    let occurrence_index =
        source_selector_occurrence_index_from_open_documents(state, workspace_folder_uri);
    let query_target_style_uri = query_target_style_uri_for_matching(target_style_uri);
    let rename_plan = summarize_omena_query_rename_plan_from_occurrence_index(
        selector_name,
        new_name,
        query_target_style_uri.as_deref(),
        occurrence_index.definitions.as_slice(),
        &occurrence_index.source_selector_index,
    );
    if rename_plan.edits.is_empty() {
        return Value::Null;
    }

    let mut changes: BTreeMap<String, Vec<Value>> = BTreeMap::new();
    for edit in rename_plan.edits {
        let edit_uri = external_document_uri_for_query_uri(state, edit.uri.as_str());
        changes.entry(edit_uri).or_default().push(json!({
            "range": edit.range,
            "newText": edit.new_text,
        }));
    }
    for edits in changes.values_mut() {
        edits.sort_by_key(|edit| {
            let line = edit
                .pointer("/range/start/line")
                .and_then(Value::as_u64)
                .unwrap_or_default();
            let character = edit
                .pointer("/range/start/character")
                .and_then(Value::as_u64)
                .unwrap_or_default();
            (line, character)
        });
    }

    let mut response_changes = serde_json::Map::new();
    for (uri, edits) in changes {
        response_changes.insert(uri, json!(edits));
    }
    json!({
        "changes": Value::Object(response_changes),
    })
}

fn external_document_uri_for_query_uri(state: &LspShellState, uri: &str) -> String {
    state
        .document(uri)
        .map(|document| document.uri.clone())
        .unwrap_or_else(|| uri.to_string())
}

fn render_style_hover_candidate_markdown_for_workspace(
    state: &LspShellState,
    document_uri: &str,
    source: &str,
    candidate: &LspStyleHoverCandidate,
) -> String {
    let workspace_folder_uri = state
        .document(document_uri)
        .and_then(|document| document.workspace_folder_uri.clone())
        .or_else(|| resolve_workspace_folder_uri(state, document_uri));
    let style_sources = style_sources_for_hover_render(
        state,
        workspace_folder_uri.as_deref(),
        document_uri,
        source,
    );
    let resolution_inputs =
        resolution_inputs_for_workspace_uri(state, workspace_folder_uri.as_deref());
    let narrowing_substrate = cascade_narrowing_substrate_for_style_sources(
        state,
        style_sources.as_slice(),
        &resolution_inputs,
    );
    let render_parts =
        summarize_omena_query_style_hover_render_parts_for_workspace_file_hover_position_with_substrate(
            document_uri,
            style_sources.as_slice(),
            &narrowing_substrate,
            candidate.kind,
            candidate.name.as_str(),
            candidate.range.start,
        )
        .unwrap_or_else(|| {
            summarize_omena_query_style_hover_render_parts_for_hover_position(
                source,
                candidate.kind,
                candidate.name.as_str(),
                candidate.range.start,
            )
        });
    render_style_hover_candidate_markdown_from_parts(document_uri, candidate, &render_parts)
}

#[cfg(test)]
mod tests;
