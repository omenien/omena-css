use super::*;
use crate::workspace_occurrence_cache::{
    evict_workspace_occurrence_shard, load_workspace_occurrence_shard,
    record_workspace_occurrence_shadow_mismatch, store_workspace_occurrence_shard,
    workspace_occurrence_dependency_digest, workspace_occurrence_shadow_asserts_on_mismatch,
    workspace_occurrence_shard_should_shadow,
};
use serde::Serialize;
#[cfg(test)]
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::sync::atomic::AtomicU64;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct StyleSymbolOccurrenceDocumentReadV1 {
    uri: String,
    text_hash: String,
    foreign_package_identity: Option<String>,
}

#[derive(Debug, Clone)]
pub(crate) struct StyleSymbolOccurrenceReadSetV1 {
    pub(crate) dependency_digest: Option<String>,
    pub(crate) dependency_document_uris: BTreeSet<String>,
}

pub(crate) fn style_symbol_workspace_occurrences_for_document(
    state: &dyn LspQueryReadView,
    document: &LspTextDocumentState,
    workspace_folder_uri: Option<&str>,
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
            record_workspace_occurrence_shadow_verification(document.uri.as_str());
            let fresh = extract_fresh_style_symbol_workspace_occurrences_for_document(
                state,
                document,
                workspace_folder_uri,
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
    record_workspace_occurrence_extractor_rebuild(document.uri.as_str());
    let workspace_occurrences = extract_style_symbol_workspace_occurrences_for_document(
        state,
        document,
        workspace_folder_uri,
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
        workspace_occurrences.as_slice(),
    );
    workspace_occurrences
}

pub(crate) fn style_symbol_occurrence_read_set(
    state: &dyn LspQueryReadView,
    document: &LspTextDocumentState,
) -> StyleSymbolOccurrenceReadSetV1 {
    #[cfg(test)]
    record_workspace_occurrence_read_set_recomputation(document.uri.as_str());
    let dependency_documents = style_symbol_occurrence_dependency_documents(state, document);
    let dependency_document_uris = dependency_documents
        .iter()
        .map(|dependency| dependency.uri.clone())
        .collect();
    StyleSymbolOccurrenceReadSetV1 {
        dependency_digest: workspace_occurrence_dependency_digest(&(
            dependency_documents,
            state.query_resolution().external_sifs.as_slice(),
        )),
        dependency_document_uris,
    }
}

fn style_symbol_occurrence_dependency_documents(
    state: &dyn LspQueryReadView,
    document: &LspTextDocumentState,
) -> Vec<StyleSymbolOccurrenceDocumentReadV1> {
    let mut target_uris = BTreeSet::new();
    let candidates = crate::collect_style_hover_candidates(document.uri.as_str(), &document.text)
        .map(|(_, candidates)| candidates)
        .unwrap_or_default();
    for candidate in &candidates {
        if !is_sass_symbol_reference_kind(candidate.kind) {
            continue;
        }
        target_uris.extend(sass_module_target_uris_for_candidate(
            state, document, candidate,
        ));
    }
    let mut visited = BTreeSet::new();
    let mut reads = Vec::new();
    for target_uri in target_uris {
        collect_style_symbol_occurrence_dependency_document(
            state,
            target_uri.as_str(),
            &mut visited,
            &mut reads,
        );
    }
    reads.sort_by(|left, right| left.uri.cmp(&right.uri));
    reads
}

fn collect_style_symbol_occurrence_dependency_document(
    state: &dyn LspQueryReadView,
    target_uri: &str,
    visited: &mut BTreeSet<String>,
    reads: &mut Vec<StyleSymbolOccurrenceDocumentReadV1>,
) {
    if !visited.insert(target_uri.to_string()) {
        return;
    }
    let target_document = state
        .document(target_uri)
        .cloned()
        .or_else(|| style_document_from_disk_for_uri(state, target_uri));
    let Some(target_document) = target_document else {
        return;
    };
    let foreign_package_identity =
        crate::foreign_style_identity::foreign_sass_package_identity_for_uri(state, target_uri)
            .map(|identity| {
                format!(
                    "{}@{}/{}",
                    identity.package_name, identity.version, identity.subpath
                )
            });
    reads.push(StyleSymbolOccurrenceDocumentReadV1 {
        uri: target_document.uri.clone(),
        text_hash: target_document.text_hash.clone(),
        foreign_package_identity,
    });
    for forward_edge in sass_forward_edges_for_document(&target_document) {
        let Some(forward_uri) = resolve_lsp_style_uri_for_specifier(
            state,
            &target_document,
            forward_edge.source.as_str(),
        ) else {
            continue;
        };
        collect_style_symbol_occurrence_dependency_document(
            state,
            forward_uri.as_str(),
            visited,
            reads,
        );
    }
}

pub(crate) fn extract_fresh_style_symbol_workspace_occurrences_for_document(
    state: &dyn LspQueryReadView,
    document: &LspTextDocumentState,
    workspace_folder_uri: Option<&str>,
) -> Vec<OmenaWorkspaceOccurrenceV0> {
    let Some((_, candidates)) =
        crate::collect_style_hover_candidates(document.uri.as_str(), document.text.as_str())
    else {
        return Vec::new();
    };
    let mut fresh_document = document.clone();
    fresh_document.style_candidates = candidates;
    extract_style_symbol_workspace_occurrences_for_document(
        state,
        &fresh_document,
        workspace_folder_uri,
    )
}

#[cfg(test)]
thread_local! {
    static WORKSPACE_OCCURRENCE_EXTRACTOR_REBUILDS: std::cell::RefCell<BTreeMap<String, u64>> = const { std::cell::RefCell::new(BTreeMap::new()) };
    static WORKSPACE_OCCURRENCE_SHADOW_VERIFICATIONS: std::cell::RefCell<BTreeMap<String, u64>> = const { std::cell::RefCell::new(BTreeMap::new()) };
    static WORKSPACE_OCCURRENCE_READ_SET_RECOMPUTATIONS: std::cell::RefCell<BTreeMap<String, u64>> = const { std::cell::RefCell::new(BTreeMap::new()) };
}

#[cfg(test)]
pub(crate) fn record_workspace_occurrence_extractor_rebuild(document_uri: &str) {
    WORKSPACE_OCCURRENCE_EXTRACTOR_REBUILDS.with(|counts| {
        let mut counts = counts.borrow_mut();
        *counts.entry(document_uri.to_string()).or_default() += 1;
    });
}

#[cfg(test)]
pub(crate) fn record_workspace_occurrence_shadow_verification(document_uri: &str) {
    WORKSPACE_OCCURRENCE_SHADOW_VERIFICATIONS.with(|counts| {
        let mut counts = counts.borrow_mut();
        *counts.entry(document_uri.to_string()).or_default() += 1;
    });
}

#[cfg(test)]
fn record_workspace_occurrence_read_set_recomputation(document_uri: &str) {
    WORKSPACE_OCCURRENCE_READ_SET_RECOMPUTATIONS.with(|counts| {
        let mut counts = counts.borrow_mut();
        *counts.entry(document_uri.to_string()).or_default() += 1;
    });
}

#[cfg(test)]
pub(crate) fn reset_workspace_occurrence_extractor_counters_for_test() {
    WORKSPACE_OCCURRENCE_EXTRACTOR_REBUILDS.with(|counts| counts.borrow_mut().clear());
    WORKSPACE_OCCURRENCE_SHADOW_VERIFICATIONS.with(|counts| counts.borrow_mut().clear());
    WORKSPACE_OCCURRENCE_READ_SET_RECOMPUTATIONS.with(|counts| counts.borrow_mut().clear());
}

#[cfg(test)]
pub(crate) fn workspace_occurrence_extractor_rebuild_count_for_test(document_uri: &str) -> u64 {
    WORKSPACE_OCCURRENCE_EXTRACTOR_REBUILDS.with(|counts| {
        counts
            .borrow()
            .get(document_uri)
            .copied()
            .unwrap_or_default()
    })
}

#[cfg(test)]
pub(crate) fn workspace_occurrence_shadow_verification_count_for_test(document_uri: &str) -> u64 {
    WORKSPACE_OCCURRENCE_SHADOW_VERIFICATIONS.with(|counts| {
        counts
            .borrow()
            .get(document_uri)
            .copied()
            .unwrap_or_default()
    })
}

#[cfg(test)]
pub(crate) fn workspace_occurrence_shadow_verification_total_for_test() -> u64 {
    WORKSPACE_OCCURRENCE_SHADOW_VERIFICATIONS.with(|counts| counts.borrow().values().sum())
}

#[cfg(test)]
pub(crate) fn workspace_occurrence_read_set_recomputation_count_for_test(
    document_uri: &str,
) -> u64 {
    WORKSPACE_OCCURRENCE_READ_SET_RECOMPUTATIONS.with(|counts| {
        counts
            .borrow()
            .get(document_uri)
            .copied()
            .unwrap_or_default()
    })
}
