use std::{
    collections::{BTreeMap, BTreeSet},
    sync::{Arc, atomic::AtomicU64},
};

use omena_query::{
    OmenaQuerySourceSelectorOccurrenceIndexV0, OmenaQuerySourceSelectorOccurrenceV0,
    OmenaQueryStyleSelectorDefinitionV0, OmenaWorkspaceMonikerInput,
    OmenaWorkspaceOccurrenceRoleV0, OmenaWorkspaceOccurrenceSurfaceV0, OmenaWorkspaceOccurrenceV0,
    omena_workspace_moniker, resolve_omena_query_source_candidate_selector_names,
    summarize_omena_query_workspace_occurrence_index_from_occurrences,
};

use crate::{
    LspQueryReadView, collect_source_selector_reference_candidates, document_has_style_index,
    occurrence_mapping::{
        source_selector_occurrence_from_workspace_occurrence_for_lsp,
        workspace_occurrence_from_source_selector_occurrence_for_lsp,
        workspace_occurrence_kind_from_source_reference_kind_for_lsp,
    },
    protocol::{file_uri_equivalent, is_style_document_uri, workspace_folder_compatible},
    query_source_selector_reference_candidate_for_matching,
    query_style_selector_definition_for_matching, resolution_inputs_for_workspace_uri,
    state::{
        LspFileId, LspSourceSelectorOccurrenceDocumentKey, LspTextDocumentState,
        LspWorkspaceOccurrenceDocumentMemoEntry, LspWorkspaceOccurrenceIndexMemo,
    },
    style_selector_definitions_from_open_documents, style_selector_definitions_from_uri,
    style_symbol_provider::style_symbol_occurrence_read_set,
    style_symbol_workspace_occurrences_for_document,
    workspace_occurrence_cache::{
        evict_workspace_occurrence_shard, load_workspace_occurrence_shard,
        record_workspace_occurrence_shadow_mismatch, store_workspace_occurrence_shard,
        workspace_occurrence_dependency_digest, workspace_occurrence_shadow_asserts_on_mismatch,
        workspace_occurrence_shard_should_shadow,
    },
};

#[derive(Debug, Clone)]
pub(crate) struct WorkspaceOccurrenceIndexes {
    pub(crate) definitions: Vec<OmenaQueryStyleSelectorDefinitionV0>,
    pub(crate) source_selector_index: Arc<OmenaQuerySourceSelectorOccurrenceIndexV0>,
    pub(crate) workspace_index: Arc<omena_query::OmenaWorkspaceOccurrenceIndexV0>,
}

pub(crate) fn workspace_occurrence_indexes_from_documents(
    state: &dyn LspQueryReadView,
    workspace_folder_uri: Option<&str>,
) -> WorkspaceOccurrenceIndexes {
    let source_documents = occurrence_documents(state, workspace_folder_uri, false);
    let style_documents = occurrence_documents(state, workspace_folder_uri, true);
    let source_document_keys = source_documents
        .iter()
        .map(|(_, document)| occurrence_document_key(document))
        .collect::<Vec<_>>();
    let style_document_keys = style_documents
        .iter()
        .map(|(_, document)| occurrence_document_key(document))
        .collect::<Vec<_>>();
    let memo_workspace_folder_uri = workspace_folder_uri.map(str::to_string);
    let environment_digest = workspace_occurrence_dependency_digest(&(
        &state.query_resolution().external_sifs,
        &resolution_inputs_for_workspace_uri(state, workspace_folder_uri),
    ));
    let memo_revision_digest = workspace_occurrence_dependency_digest(&(
        &memo_workspace_folder_uri,
        &environment_digest,
        &source_document_keys,
        &style_document_keys,
    ));
    let memo_revision_should_shadow = memo_revision_digest
        .as_deref()
        .is_some_and(workspace_occurrence_shard_should_shadow);
    let prior_memo = {
        let memo = state.workspace_occurrence_index_memo_lock();
        if let Some(memo) = memo.as_ref()
            && memo.workspace_folder_uri == memo_workspace_folder_uri
            && memo.environment_digest == environment_digest
            && memo.source_document_keys == source_document_keys
            && memo.style_document_keys == style_document_keys
            && (!memo_revision_should_shadow
                || memo.shadow_verified_revision_digest == memo_revision_digest)
        {
            return WorkspaceOccurrenceIndexes {
                definitions: memo.definitions.clone(),
                source_selector_index: Arc::clone(&memo.source_selector_index),
                workspace_index: Arc::clone(&memo.workspace_index),
            };
        }
        memo.clone()
    };
    let verify_reused_entries = memo_revision_should_shadow
        && prior_memo
            .as_ref()
            .is_some_and(|memo| memo.shadow_verified_revision_digest != memo_revision_digest);
    let environment_matches = prior_memo.as_ref().is_some_and(|memo| {
        memo.workspace_folder_uri == memo_workspace_folder_uri
            && memo.environment_digest == environment_digest
    });
    let shadow_mismatch_count = prior_memo
        .as_ref()
        .map(|memo| Arc::clone(&memo.shadow_mismatch_count))
        .unwrap_or_else(|| Arc::new(AtomicU64::new(0)));
    let changed_document_uris = changed_occurrence_document_uris(
        prior_memo.as_ref(),
        source_document_keys.as_slice(),
        style_document_keys.as_slice(),
    );
    let rebuild_started = std::time::Instant::now();
    let definitions =
        style_selector_definitions_from_open_documents(state, "", workspace_folder_uri)
            .iter()
            .map(|(uri, definition)| query_style_selector_definition_for_matching(uri, definition))
            .collect::<Vec<_>>();
    let mut workspace_occurrences = Vec::new();
    let mut source_occurrences = Vec::new();
    let mut document_entries = BTreeMap::new();
    for (file_id, document) in source_documents {
        let document_key = occurrence_document_key(document);
        let prior_entry = environment_matches
            .then(|| prior_memo.as_ref()?.document_entries.get(file_id))
            .flatten();
        let reusable_entry = prior_entry.filter(|entry| {
            entry.document_key == document_key
                && entry
                    .dependency_document_uris
                    .is_disjoint(&changed_document_uris)
        });
        let (document_occurrences, dependency_document_uris, dependency_digest) =
            if let Some(entry) = reusable_entry {
                #[cfg(test)]
                record_workspace_occurrence_memo_hit(document.uri.as_str());
                (
                    workspace_occurrence_memo_value(
                        document,
                        entry,
                        shadow_mismatch_count.as_ref(),
                        verify_reused_entries,
                        || {
                            let read_set = source_selector_occurrence_read_set(
                                state,
                                document,
                                definitions.as_slice(),
                            );
                            extract_shadow_source_selector_workspace_occurrences_for_document(
                                state,
                                document,
                                read_set.definitions.as_slice(),
                            )
                        },
                    ),
                    entry.dependency_document_uris.clone(),
                    entry.dependency_digest.clone(),
                )
            } else {
                let read_set =
                    source_selector_occurrence_read_set(state, document, definitions.as_slice());
                let document_occurrences =
                    cached_source_selector_workspace_occurrences_for_document(
                        state,
                        document,
                        workspace_folder_uri,
                        read_set.definitions.as_slice(),
                        read_set.dependency_digest.as_deref(),
                        shadow_mismatch_count.as_ref(),
                    );
                (
                    document_occurrences,
                    read_set.dependency_document_uris,
                    read_set.dependency_digest,
                )
            };
        workspace_occurrences.extend(document_occurrences.clone());
        source_occurrences.extend(
            document_occurrences
                .iter()
                .cloned()
                .filter_map(source_selector_occurrence_from_workspace_occurrence_for_lsp),
        );
        document_entries.insert(
            *file_id,
            LspWorkspaceOccurrenceDocumentMemoEntry {
                document_key,
                dependency_document_uris,
                dependency_digest,
                occurrences: document_occurrences,
            },
        );
    }
    source_occurrences.sort();
    source_occurrences.dedup();
    let source_phase_ms = rebuild_started.elapsed().as_millis();
    for (file_id, document) in style_documents {
        let document_key = occurrence_document_key(document);
        let prior_entry = environment_matches
            .then(|| prior_memo.as_ref()?.document_entries.get(file_id))
            .flatten();
        let reusable_entry = prior_entry.filter(|entry| {
            entry.document_key == document_key
                && entry
                    .dependency_document_uris
                    .is_disjoint(&changed_document_uris)
        });
        let (document_occurrences, dependency_document_uris, dependency_digest) = if let Some(
            entry,
        ) =
            reusable_entry
        {
            #[cfg(test)]
            record_workspace_occurrence_memo_hit(document.uri.as_str());
            (
                workspace_occurrence_memo_value(
                    document,
                    entry,
                    shadow_mismatch_count.as_ref(),
                    verify_reused_entries,
                    || {
                        crate::style_symbol_provider::extract_fresh_style_symbol_workspace_occurrences_for_document(
                                state,
                                document,
                                workspace_folder_uri,
                            )
                    },
                ),
                entry.dependency_document_uris.clone(),
                entry.dependency_digest.clone(),
            )
        } else {
            let read_set = style_symbol_occurrence_read_set(state, document);
            let occurrences = style_symbol_workspace_occurrences_for_document(
                state,
                document,
                workspace_folder_uri,
                read_set.dependency_digest.as_deref(),
                shadow_mismatch_count.as_ref(),
            );
            (
                occurrences,
                read_set.dependency_document_uris,
                read_set.dependency_digest,
            )
        };
        workspace_occurrences.extend(document_occurrences.clone());
        document_entries.insert(
            *file_id,
            LspWorkspaceOccurrenceDocumentMemoEntry {
                document_key,
                dependency_document_uris,
                dependency_digest,
                occurrences: document_occurrences,
            },
        );
    }
    workspace_occurrences.sort();
    workspace_occurrences.dedup();
    let style_phase_ms = rebuild_started.elapsed().as_millis() - source_phase_ms;
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
    *state.workspace_occurrence_index_memo_lock() = Some(LspWorkspaceOccurrenceIndexMemo {
        workspace_folder_uri: memo_workspace_folder_uri,
        environment_digest,
        source_document_keys,
        style_document_keys,
        document_entries,
        shadow_verified_revision_digest: verify_reused_entries
            .then_some(memo_revision_digest)
            .flatten(),
        definitions: definitions.clone(),
        source_selector_index: Arc::clone(&index),
        workspace_index: Arc::clone(&workspace_index),
        shadow_mismatch_count,
    });
    crate::loop_trace!(
        "occ-index-rebuild source_ms={} style_ms={} aggregate_ms={} total_ms={}",
        source_phase_ms,
        style_phase_ms,
        rebuild_started.elapsed().as_millis() - source_phase_ms - style_phase_ms,
        rebuild_started.elapsed().as_millis()
    );
    WorkspaceOccurrenceIndexes {
        definitions,
        source_selector_index: index,
        workspace_index,
    }
}

#[derive(Debug, Clone)]
struct SourceSelectorOccurrenceReadSetV1 {
    definitions: Vec<OmenaQueryStyleSelectorDefinitionV0>,
    dependency_document_uris: BTreeSet<String>,
    dependency_digest: Option<String>,
}

fn source_selector_occurrence_read_set(
    state: &dyn LspQueryReadView,
    document: &LspTextDocumentState,
    workspace_definitions: &[OmenaQueryStyleSelectorDefinitionV0],
) -> SourceSelectorOccurrenceReadSetV1 {
    let mut target_uris = document
        .source_syntax_index
        .imported_style_bindings
        .iter()
        .map(|binding| binding.style_uri.clone())
        .chain(
            document
                .source_selector_candidates
                .iter()
                .filter_map(|candidate| candidate.target_style_uri.clone()),
        )
        .collect::<BTreeSet<_>>();
    let has_unscoped_prefix = document.source_selector_candidates.iter().any(|candidate| {
        candidate.kind == "sourceSelectorPrefixReference" && candidate.target_style_uri.is_none()
    });
    let requires_workspace_fallback = has_unscoped_prefix;
    let mut definitions = if requires_workspace_fallback {
        target_uris.extend(
            occurrence_documents(state, document.workspace_folder_uri.as_deref(), true)
                .into_iter()
                .map(|(_, style_document)| style_document.uri.clone()),
        );
        target_uris.extend(
            workspace_definitions
                .iter()
                .map(|definition| definition.uri.clone()),
        );
        workspace_definitions.to_vec()
    } else {
        workspace_definitions
            .iter()
            .filter(|definition| {
                target_uris.iter().any(|target_uri| {
                    file_uri_equivalent(target_uri.as_str(), definition.uri.as_str())
                })
            })
            .cloned()
            .collect::<Vec<_>>()
    };
    for target_uri in &target_uris {
        if definitions
            .iter()
            .any(|definition| file_uri_equivalent(target_uri.as_str(), definition.uri.as_str()))
        {
            continue;
        }
        definitions.extend(
            style_selector_definitions_from_uri(state, target_uri.as_str())
                .iter()
                .map(|(uri, definition)| {
                    query_style_selector_definition_for_matching(uri, definition)
                }),
        );
    }
    definitions.sort_by(|left, right| {
        (
            left.uri.as_str(),
            left.range.start.line,
            left.range.start.character,
            left.name.as_str(),
        )
            .cmp(&(
                right.uri.as_str(),
                right.range.start.line,
                right.range.start.character,
                right.name.as_str(),
            ))
    });
    definitions.dedup();
    let dependency_digest = workspace_occurrence_dependency_digest(&definitions);
    let dependency_document_uris = target_uris
        .iter()
        .map(|target_uri| {
            state
                .query_documents()
                .values()
                .find(|candidate| file_uri_equivalent(target_uri.as_str(), candidate.uri.as_str()))
                .map(|candidate| candidate.uri.clone())
                .unwrap_or_else(|| target_uri.clone())
        })
        .collect();
    SourceSelectorOccurrenceReadSetV1 {
        definitions,
        dependency_document_uris,
        dependency_digest,
    }
}

#[cfg(test)]
pub(crate) fn source_selector_occurrence_read_set_digest_for_test(
    state: &dyn LspQueryReadView,
    document: &LspTextDocumentState,
    workspace_definitions: &[OmenaQueryStyleSelectorDefinitionV0],
) -> Option<String> {
    source_selector_occurrence_read_set(state, document, workspace_definitions).dependency_digest
}

fn workspace_occurrence_memo_value(
    document: &LspTextDocumentState,
    entry: &LspWorkspaceOccurrenceDocumentMemoEntry,
    shadow_mismatch_count: &AtomicU64,
    verify_revision: bool,
    fresh: impl FnOnce() -> Vec<OmenaWorkspaceOccurrenceV0>,
) -> Vec<OmenaWorkspaceOccurrenceV0> {
    if !verify_revision {
        return entry.occurrences.clone();
    }
    #[cfg(test)]
    crate::style_symbol_provider::record_workspace_occurrence_shadow_verification(
        document.uri.as_str(),
    );
    let fresh = fresh();
    let cached_bytes = serde_json::to_vec(&entry.occurrences).map_err(|error| error.to_string());
    let fresh_bytes = serde_json::to_vec(&fresh).map_err(|error| error.to_string());
    let matches = cached_bytes.is_ok() && fresh_bytes.is_ok() && cached_bytes == fresh_bytes;
    if matches {
        return entry.occurrences.clone();
    }
    record_workspace_occurrence_shadow_mismatch(shadow_mismatch_count, document.uri.as_str());
    if workspace_occurrence_shadow_asserts_on_mismatch() {
        assert!(
            matches,
            "workspace occurrence RAM memo mismatch: document_uri={} cached_bytes={:?} fresh_bytes={:?}",
            document.uri,
            cached_bytes.as_ref().map(Vec::len),
            fresh_bytes.as_ref().map(Vec::len),
        );
    }
    fresh
}

pub(crate) fn source_selector_occurrence_index_from_open_documents(
    state: &dyn LspQueryReadView,
    workspace_folder_uri: Option<&str>,
) -> WorkspaceOccurrenceIndexes {
    workspace_occurrence_indexes_from_documents(state, workspace_folder_uri)
}

fn occurrence_documents<'a>(
    state: &'a dyn LspQueryReadView,
    workspace_folder_uri: Option<&str>,
    style_documents: bool,
) -> Vec<(&'a LspFileId, &'a LspTextDocumentState)> {
    state
        .query_documents()
        .iter()
        .filter(|(_, document)| workspace_folder_compatible(workspace_folder_uri, document))
        .filter(|(_, document)| {
            if style_documents {
                document_has_style_index(document)
            } else {
                !is_style_document_uri(document.uri.as_str())
            }
        })
        .map(|(file_id, document)| (file_id, document.as_ref()))
        .collect()
}

fn occurrence_document_key(
    document: &LspTextDocumentState,
) -> LspSourceSelectorOccurrenceDocumentKey {
    LspSourceSelectorOccurrenceDocumentKey {
        uri: document.uri.clone(),
        workspace_folder_uri: document.workspace_folder_uri.clone(),
        language_id: document.language_id.clone(),
        version: document.version,
        text_hash: document.text_hash.clone(),
    }
}

fn changed_occurrence_document_uris(
    prior_memo: Option<&LspWorkspaceOccurrenceIndexMemo>,
    source_document_keys: &[LspSourceSelectorOccurrenceDocumentKey],
    style_document_keys: &[LspSourceSelectorOccurrenceDocumentKey],
) -> BTreeSet<String> {
    let current = source_document_keys
        .iter()
        .chain(style_document_keys)
        .map(|key| (key.uri.as_str(), key))
        .collect::<BTreeMap<_, _>>();
    let prior = prior_memo
        .into_iter()
        .flat_map(|memo| {
            memo.source_document_keys
                .iter()
                .chain(&memo.style_document_keys)
        })
        .map(|key| (key.uri.as_str(), key))
        .collect::<BTreeMap<_, _>>();
    current
        .keys()
        .chain(prior.keys())
        .filter(|uri| current.get(**uri) != prior.get(**uri))
        .map(|uri| (*uri).to_string())
        .collect()
}

fn cached_source_selector_workspace_occurrences_for_document(
    state: &dyn LspQueryReadView,
    document: &LspTextDocumentState,
    workspace_folder_uri: Option<&str>,
    definitions: &[OmenaQueryStyleSelectorDefinitionV0],
    dependency_digest: Option<&str>,
    shadow_mismatch_count: &AtomicU64,
) -> Vec<OmenaWorkspaceOccurrenceV0> {
    let resolution_inputs =
        resolution_inputs_for_workspace_uri(state, document.workspace_folder_uri.as_deref());
    if let Some(shard) = load_workspace_occurrence_shard(
        &state.query_resolution().cache_storage,
        document.workspace_folder_uri.as_deref(),
        workspace_folder_uri,
        document.uri.as_str(),
        document.language_id.as_str(),
        document.text_hash.as_str(),
        dependency_digest,
        &resolution_inputs,
    ) {
        if workspace_occurrence_shard_should_shadow(shard.key.as_str()) {
            #[cfg(test)]
            crate::style_symbol_provider::record_workspace_occurrence_shadow_verification(
                document.uri.as_str(),
            );
            let fresh = extract_shadow_source_selector_workspace_occurrences_for_document(
                state,
                document,
                definitions,
            );
            let cached_bytes =
                serde_json::to_vec(&shard.occurrences).map_err(|error| error.to_string());
            let fresh_bytes = serde_json::to_vec(&fresh).map_err(|error| error.to_string());
            let matches =
                cached_bytes.is_ok() && fresh_bytes.is_ok() && cached_bytes == fresh_bytes;
            if !matches {
                record_workspace_occurrence_shadow_mismatch(
                    shadow_mismatch_count,
                    document.uri.as_str(),
                );
                if workspace_occurrence_shadow_asserts_on_mismatch() {
                    assert!(
                        matches,
                        "workspace occurrence shadow mismatch: document_uri={} document_workspace_folder_uri={:?} workspace_folder_uri={workspace_folder_uri:?} language_id={} text_hash={} dependency_digest={dependency_digest:?} shard_key={} cached_bytes={:?} fresh_bytes={:?}",
                        document.uri,
                        document.workspace_folder_uri,
                        document.language_id,
                        document.text_hash,
                        shard.key,
                        cached_bytes.as_ref().map(Vec::len),
                        fresh_bytes.as_ref().map(Vec::len),
                    );
                }
                evict_workspace_occurrence_shard(
                    &state.query_resolution().cache_storage,
                    document.workspace_folder_uri.as_deref(),
                    document.uri.as_str(),
                    document.language_id.as_str(),
                );
                store_workspace_occurrence_shard(
                    &state.query_resolution().cache_storage,
                    document.workspace_folder_uri.as_deref(),
                    workspace_folder_uri,
                    document.uri.as_str(),
                    document.language_id.as_str(),
                    document.text_hash.as_str(),
                    dependency_digest,
                    &resolution_inputs,
                    fresh.as_slice(),
                );
                return fresh;
            }
        }
        return shard.occurrences;
    }

    #[cfg(test)]
    crate::style_symbol_provider::record_workspace_occurrence_extractor_rebuild(
        document.uri.as_str(),
    );
    let occurrences = extract_fresh_source_selector_workspace_occurrences_for_document(
        state,
        document,
        definitions,
    );
    store_workspace_occurrence_shard(
        &state.query_resolution().cache_storage,
        document.workspace_folder_uri.as_deref(),
        workspace_folder_uri,
        document.uri.as_str(),
        document.language_id.as_str(),
        document.text_hash.as_str(),
        dependency_digest,
        &resolution_inputs,
        occurrences.as_slice(),
    );
    occurrences
}

fn extract_fresh_source_selector_workspace_occurrences_for_document(
    state: &dyn LspQueryReadView,
    document: &LspTextDocumentState,
    definitions: &[OmenaQueryStyleSelectorDefinitionV0],
) -> Vec<OmenaWorkspaceOccurrenceV0> {
    source_selector_workspace_occurrences_for_document(state, document, definitions)
}

fn extract_shadow_source_selector_workspace_occurrences_for_document(
    state: &dyn LspQueryReadView,
    document: &LspTextDocumentState,
    definitions: &[OmenaQueryStyleSelectorDefinitionV0],
) -> Vec<OmenaWorkspaceOccurrenceV0> {
    let mut fresh_document = document.clone();
    fresh_document.source_selector_candidates =
        crate::source_syntax_index::source_selector_candidates_from_index(
            &fresh_document,
            &document.source_syntax_index,
        );
    source_selector_workspace_occurrences_for_document(state, &fresh_document, definitions)
}

fn source_selector_workspace_occurrences_for_document(
    state: &dyn LspQueryReadView,
    document: &LspTextDocumentState,
    definitions: &[OmenaQueryStyleSelectorDefinitionV0],
) -> Vec<OmenaWorkspaceOccurrenceV0> {
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
    occurrences
}

#[cfg(test)]
thread_local! {
    static WORKSPACE_OCCURRENCE_MEMO_HITS: std::cell::RefCell<BTreeMap<String, u64>> = const { std::cell::RefCell::new(BTreeMap::new()) };
}

#[cfg(test)]
fn record_workspace_occurrence_memo_hit(document_uri: &str) {
    WORKSPACE_OCCURRENCE_MEMO_HITS.with(|hits| {
        *hits
            .borrow_mut()
            .entry(document_uri.to_string())
            .or_default() += 1;
    });
}

#[cfg(test)]
pub(crate) fn reset_workspace_occurrence_memo_hit_counts_for_test() {
    WORKSPACE_OCCURRENCE_MEMO_HITS.with(|hits| hits.borrow_mut().clear());
}

#[cfg(test)]
pub(crate) fn workspace_occurrence_memo_hit_count_for_test(document_uri: &str) -> u64 {
    WORKSPACE_OCCURRENCE_MEMO_HITS
        .with(|hits| hits.borrow().get(document_uri).copied().unwrap_or_default())
}
