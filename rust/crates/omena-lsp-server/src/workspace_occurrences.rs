use std::{collections::BTreeSet, sync::Arc};

use omena_query::{
    OmenaQuerySourceSelectorOccurrenceIndexV0, OmenaQuerySourceSelectorOccurrenceV0,
    OmenaQueryStyleSelectorDefinitionV0, OmenaWorkspaceMonikerInput,
    OmenaWorkspaceOccurrenceRoleV0, OmenaWorkspaceOccurrenceSurfaceV0, OmenaWorkspaceOccurrenceV0,
    omena_workspace_moniker, resolve_omena_query_source_candidate_selector_names,
    summarize_omena_query_workspace_occurrence_index_from_occurrences,
};

use crate::{
    LspShellState, collect_source_selector_reference_candidates, document_has_style_index,
    occurrence_mapping::{
        source_selector_occurrence_from_workspace_occurrence_for_lsp,
        style_symbol_occurrence_from_workspace_occurrence_for_lsp,
        workspace_occurrence_from_source_selector_occurrence_for_lsp,
        workspace_occurrence_kind_from_source_reference_kind_for_lsp,
    },
    protocol::{is_style_document_uri, workspace_folder_compatible},
    query_source_selector_reference_candidate_for_matching,
    query_style_selector_definition_for_matching, resolution_inputs_for_workspace_uri,
    state::{
        LspSourceSelectorOccurrenceDocumentKey, LspTextDocumentState,
        LspWorkspaceOccurrenceIndexMemo,
    },
    store_source_selector_occurrence_sidecar, store_style_symbol_occurrence_sidecar,
    style_selector_definitions_from_open_documents,
    style_symbol_workspace_occurrences_for_document,
    workspace_occurrence_cache::{
        load_workspace_occurrence_shard, store_workspace_occurrence_shard,
        workspace_occurrence_dependency_digest,
    },
};

#[derive(Debug, Clone)]
pub(crate) struct WorkspaceOccurrenceIndexes {
    pub(crate) definitions: Vec<OmenaQueryStyleSelectorDefinitionV0>,
    pub(crate) source_selector_index: Arc<OmenaQuerySourceSelectorOccurrenceIndexV0>,
    pub(crate) workspace_index: Arc<omena_query::OmenaWorkspaceOccurrenceIndexV0>,
}

pub(crate) fn workspace_occurrence_indexes_from_documents(
    state: &LspShellState,
    workspace_folder_uri: Option<&str>,
) -> WorkspaceOccurrenceIndexes {
    let source_document_keys =
        source_selector_occurrence_document_keys(state, workspace_folder_uri);
    let style_document_keys = style_symbol_occurrence_document_keys(state, workspace_folder_uri);
    let memo_workspace_folder_uri = workspace_folder_uri.map(str::to_string);
    if let Some(memo) = state.workspace_occurrence_index_memo_lock().as_ref()
        && memo.workspace_folder_uri == memo_workspace_folder_uri
        && memo.source_document_keys == source_document_keys
        && memo.style_document_keys == style_document_keys
    {
        return WorkspaceOccurrenceIndexes {
            definitions: memo.definitions.clone(),
            source_selector_index: Arc::clone(&memo.source_selector_index),
            workspace_index: Arc::clone(&memo.workspace_index),
        };
    }
    let definitions =
        style_selector_definitions_from_open_documents(state, "", workspace_folder_uri)
            .iter()
            .map(|(uri, definition)| query_style_selector_definition_for_matching(uri, definition))
            .collect::<Vec<_>>();
    let definitions_digest = workspace_occurrence_dependency_digest(&definitions);
    let mut workspace_occurrences = Vec::new();
    let mut source_occurrences = state
        .documents
        .values()
        .filter(|document| !is_style_document_uri(document.uri.as_str()))
        .filter(|document| workspace_folder_compatible(workspace_folder_uri, document))
        .flat_map(|document| {
            let document_occurrences = source_selector_workspace_occurrences_for_document(
                state,
                document,
                definitions.as_slice(),
                definitions_digest.as_deref(),
            );
            workspace_occurrences.extend(document_occurrences.clone());
            document_occurrences
                .into_iter()
                .filter_map(source_selector_occurrence_from_workspace_occurrence_for_lsp)
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    source_occurrences.sort();
    source_occurrences.dedup();
    let style_dependency_digest = workspace_occurrence_dependency_digest(&(
        style_document_keys.as_slice(),
        &state.resolution.external_sifs,
    ));
    for document in state
        .documents
        .values()
        .filter(|document| document_has_style_index(document))
        .filter(|document| workspace_folder_compatible(workspace_folder_uri, document))
    {
        workspace_occurrences.extend(style_symbol_workspace_occurrences_for_document(
            state,
            document,
            workspace_folder_uri,
            style_dependency_digest.as_deref(),
        ));
    }
    workspace_occurrences.sort();
    workspace_occurrences.dedup();
    let workspace_index = Arc::new(
        summarize_omena_query_workspace_occurrence_index_from_occurrences(
            workspace_occurrences.as_slice(),
            vec![
                "workspaceOccurrenceIndex",
                "sourceSelectorOccurrenceIndex",
                "workspaceWideSelectorReferences",
                "workspaceWideSelectorRename",
                "styleSymbolReferences",
                "styleSymbolRename",
                "workspaceOccurrencePerFileShard",
            ],
        ),
    );
    let moniker_count = source_occurrences
        .iter()
        .map(|occurrence| occurrence.moniker.as_str())
        .collect::<BTreeSet<_>>()
        .len();
    let index = OmenaQuerySourceSelectorOccurrenceIndexV0 {
        schema_version: "0",
        product: "omena-query.source-selector-occurrence-index",
        moniker_count,
        occurrence_count: source_occurrences.len(),
        workspace_index: workspace_index.as_ref().clone(),
        occurrences: source_occurrences,
        ready_surfaces: vec![
            "sourceSelectorOccurrenceIndex",
            "workspaceWideSelectorReferences",
            "workspaceWideSelectorRename",
            "workspaceOccurrencePerFileShard",
        ],
    };
    let index = Arc::new(index);
    store_source_selector_occurrence_sidecar(
        state,
        workspace_folder_uri,
        source_document_keys.as_slice(),
        definitions.as_slice(),
        &index,
    );
    let style_occurrences = workspace_index
        .by_moniker
        .values()
        .flat_map(|occurrences| occurrences.iter())
        .filter_map(style_symbol_occurrence_from_workspace_occurrence_for_lsp)
        .collect::<Vec<_>>();
    store_style_symbol_occurrence_sidecar(
        state,
        workspace_folder_uri,
        style_document_keys.as_slice(),
        style_occurrences.as_slice(),
    );
    *state.workspace_occurrence_index_memo_lock() = Some(LspWorkspaceOccurrenceIndexMemo {
        workspace_folder_uri: memo_workspace_folder_uri,
        source_document_keys,
        style_document_keys,
        definitions: definitions.clone(),
        source_selector_index: Arc::clone(&index),
        workspace_index: Arc::clone(&workspace_index),
    });
    WorkspaceOccurrenceIndexes {
        definitions,
        source_selector_index: index,
        workspace_index,
    }
}

pub(crate) fn source_selector_occurrence_index_from_open_documents(
    state: &LspShellState,
    workspace_folder_uri: Option<&str>,
) -> WorkspaceOccurrenceIndexes {
    workspace_occurrence_indexes_from_documents(state, workspace_folder_uri)
}

pub(crate) fn source_selector_occurrence_document_keys(
    state: &LspShellState,
    workspace_folder_uri: Option<&str>,
) -> Vec<LspSourceSelectorOccurrenceDocumentKey> {
    state
        .documents
        .values()
        .filter(|document| workspace_folder_compatible(workspace_folder_uri, document))
        .map(|document| LspSourceSelectorOccurrenceDocumentKey {
            uri: document.uri.clone(),
            workspace_folder_uri: document.workspace_folder_uri.clone(),
            language_id: document.language_id.clone(),
            version: document.version,
            text_hash: document.text_hash.clone(),
        })
        .collect()
}

fn source_selector_workspace_occurrences_for_document(
    state: &LspShellState,
    document: &LspTextDocumentState,
    definitions: &[OmenaQueryStyleSelectorDefinitionV0],
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

    let mut occurrences = Vec::new();
    for candidate in collect_source_selector_reference_candidates(state, document) {
        let reference =
            query_source_selector_reference_candidate_for_matching(document, &candidate);
        let reference_candidate = omena_query::OmenaQuerySourceSelectorCandidateV0 {
            kind: reference.kind,
            name: reference.name.clone(),
            range: reference.range,
            source: reference.source,
            target_style_uri: reference.target_style_uri.clone(),
        };
        for selector_name in resolve_omena_query_source_candidate_selector_names(
            &reference_candidate,
            definitions,
            reference.target_style_uri.as_deref(),
        ) {
            let source_occurrence = OmenaQuerySourceSelectorOccurrenceV0 {
                moniker: omena_workspace_moniker(OmenaWorkspaceMonikerInput::CssModuleSelector {
                    target_style_uri: reference.target_style_uri.as_deref(),
                    selector_name: selector_name.as_str(),
                }),
                uri: reference.uri.clone(),
                selector_name: selector_name.clone(),
                range: reference.range,
                kind: workspace_occurrence_kind_from_source_reference_kind_for_lsp(reference.kind),
                role: OmenaWorkspaceOccurrenceRoleV0::Reference,
                source: OmenaWorkspaceOccurrenceSurfaceV0::OmenaQuerySourceSyntaxIndex,
                target_style_uri: reference.target_style_uri.clone(),
                rename_target: reference.kind == "sourceSelectorReference"
                    && reference.name == selector_name,
            };
            occurrences.push(
                workspace_occurrence_from_source_selector_occurrence_for_lsp(&source_occurrence),
            );
        }
    }
    occurrences.sort();
    occurrences.dedup();
    store_workspace_occurrence_shard(
        document.workspace_folder_uri.as_deref(),
        document.uri.as_str(),
        document.language_id.as_str(),
        document.text_hash.as_str(),
        dependency_digest,
        &resolution_inputs,
        occurrences.as_slice(),
    );
    occurrences
}

pub(crate) fn style_symbol_occurrence_document_keys(
    state: &LspShellState,
    workspace_folder_uri: Option<&str>,
) -> Vec<LspSourceSelectorOccurrenceDocumentKey> {
    state
        .documents
        .values()
        .filter(|document| document_has_style_index(document))
        .filter(|document| workspace_folder_compatible(workspace_folder_uri, document))
        .map(|document| LspSourceSelectorOccurrenceDocumentKey {
            uri: document.uri.clone(),
            workspace_folder_uri: document.workspace_folder_uri.clone(),
            language_id: document.language_id.clone(),
            version: document.version,
            text_hash: document.text_hash.clone(),
        })
        .collect()
}
