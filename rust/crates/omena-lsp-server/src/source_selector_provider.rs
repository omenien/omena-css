use std::collections::BTreeSet;

use omena_query::{
    ParserPositionV0, resolve_omena_query_source_provider_candidates,
    resolve_omena_query_style_selector_definitions_for_source_candidate,
};
use serde_json::Value;

use crate::{
    LspQueryReadView, LspStyleHoverCandidate, LspTextDocumentState, document_uri_from_params,
    lsp_position_from_params,
    protocol::{
        file_uri_equivalent, is_style_document_uri, parser_range_contains_position,
        workspace_folder_compatible,
    },
    query_adapter::{
        lsp_source_selector_candidate_from_query, query_definition_identity,
        query_source_selector_candidate_for_matching, query_style_selector_definition_for_matching,
    },
    style_hover_candidates_for_document, style_hover_candidates_for_uri,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SourceProviderCandidateResolution {
    pub(crate) matched: Vec<LspStyleHoverCandidate>,
    pub(crate) unresolved: Vec<LspStyleHoverCandidate>,
}

pub(crate) fn source_selector_candidate_for_params(
    state: &dyn LspQueryReadView,
    params: Option<&Value>,
) -> Option<(String, LspStyleHoverCandidate)> {
    let document_uri = document_uri_from_params(params);
    let position = lsp_position_from_params(params)?;
    let document = state.document(document_uri.as_str())?;
    if is_style_document_uri(document.uri.as_str()) {
        return None;
    }
    source_selector_candidate_at_position(state, document, position)
        .map(|candidate| (document_uri, candidate))
}

pub(crate) fn source_selector_candidate_at_position(
    state: &dyn LspQueryReadView,
    document: &LspTextDocumentState,
    position: ParserPositionV0,
) -> Option<LspStyleHoverCandidate> {
    source_selector_candidates_at_position(state, document, position)
        .into_iter()
        .next()
}

pub(crate) fn source_selector_candidates_at_position(
    state: &dyn LspQueryReadView,
    document: &LspTextDocumentState,
    position: ParserPositionV0,
) -> Vec<LspStyleHoverCandidate> {
    collect_source_selector_reference_candidates(state, document)
        .into_iter()
        .filter(|candidate| parser_range_contains_position(&candidate.range, position))
        .collect()
}

pub(crate) fn collect_source_selector_reference_candidates(
    state: &dyn LspQueryReadView,
    document: &LspTextDocumentState,
) -> Vec<LspStyleHoverCandidate> {
    resolve_source_provider_candidates(state, document).matched
}

pub(crate) fn resolve_source_provider_candidates(
    state: &dyn LspQueryReadView,
    document: &LspTextDocumentState,
) -> SourceProviderCandidateResolution {
    let source_candidates = collect_source_class_reference_candidates(document);
    let mut definitions = style_selector_definitions_from_open_documents(
        state,
        "",
        document.workspace_folder_uri.as_deref(),
    );
    for candidate in &source_candidates {
        if let Some(target_uri) = candidate.target_style_uri.as_deref()
            && !definitions
                .iter()
                .any(|(uri, _)| file_uri_equivalent(uri, target_uri))
        {
            definitions.extend(style_selector_definitions_from_uri(state, target_uri));
        }
    }
    let query_definitions = definitions
        .iter()
        .map(|(uri, definition)| query_style_selector_definition_for_matching(uri, definition))
        .collect::<Vec<_>>();
    let resolution = resolve_omena_query_source_provider_candidates(
        source_candidates
            .iter()
            .map(query_source_selector_candidate_for_matching)
            .collect(),
        query_definitions.as_slice(),
    );

    SourceProviderCandidateResolution {
        matched: resolution
            .matched
            .into_iter()
            .map(lsp_source_selector_candidate_from_query)
            .collect(),
        unresolved: resolution
            .unresolved
            .into_iter()
            .map(lsp_source_selector_candidate_from_query)
            .collect(),
    }
}

fn collect_source_class_reference_candidates(
    document: &LspTextDocumentState,
) -> Vec<LspStyleHoverCandidate> {
    document.source_selector_candidates.clone()
}

pub(crate) fn document_has_style_index(document: &LspTextDocumentState) -> bool {
    is_style_document_uri(document.uri.as_str()) || document.style_summary.is_some()
}

pub(crate) fn style_selector_definitions_from_open_documents(
    state: &dyn LspQueryReadView,
    selector_name: &str,
    workspace_folder_uri: Option<&str>,
) -> Vec<(String, LspStyleHoverCandidate)> {
    let mut definitions = Vec::new();
    for document in state.query_documents().values() {
        if !document_has_style_index(document)
            || !workspace_folder_compatible(workspace_folder_uri, document)
        {
            continue;
        }
        let Some((_, candidates)) = style_hover_candidates_for_document(document) else {
            continue;
        };
        definitions.extend(
            candidates
                .into_iter()
                .filter(|candidate| {
                    candidate.kind == "selector"
                        && (selector_name.is_empty() || candidate.name == selector_name)
                })
                .map(|candidate| (document.uri.clone(), candidate)),
        );
    }
    definitions.sort_by_key(|(uri, candidate)| {
        (
            uri.clone(),
            candidate.range.start.line,
            candidate.range.start.character,
        )
    });
    definitions
}

pub(crate) fn style_selector_definitions_from_uri(
    state: &dyn LspQueryReadView,
    uri: &str,
) -> Vec<(String, LspStyleHoverCandidate)> {
    style_hover_candidates_for_uri(state, uri)
        .map(|(_, candidates)| {
            candidates
                .into_iter()
                .filter(|candidate| candidate.kind == "selector")
                .map(|candidate| (uri.to_string(), candidate))
                .collect()
        })
        .unwrap_or_default()
}

fn style_selector_definitions_for_source_candidate(
    state: &dyn LspQueryReadView,
    candidate: &LspStyleHoverCandidate,
    workspace_folder_uri: Option<&str>,
) -> Vec<(String, LspStyleHoverCandidate)> {
    let mut definitions = style_selector_definitions_from_open_documents(
        state,
        source_candidate_definition_lookup_name(candidate),
        workspace_folder_uri,
    );
    if let Some(target_uri) = candidate.target_style_uri.as_deref()
        && !definitions
            .iter()
            .any(|(uri, _)| file_uri_equivalent(uri, target_uri))
    {
        definitions.extend(style_selector_definitions_from_uri(state, target_uri));
    }
    let query_definitions = definitions
        .iter()
        .map(|(uri, definition)| query_style_selector_definition_for_matching(uri, definition))
        .collect::<Vec<_>>();
    let matched_identities = resolve_omena_query_style_selector_definitions_for_source_candidate(
        &query_source_selector_candidate_for_matching(candidate),
        query_definitions.as_slice(),
    )
    .into_iter()
    .map(|definition| {
        query_definition_identity(
            definition.uri.as_str(),
            definition.name.as_str(),
            definition.range,
        )
    })
    .collect::<BTreeSet<_>>();

    definitions
        .into_iter()
        .filter(|(uri, definition)| {
            matched_identities.contains(&query_definition_identity(
                uri.as_str(),
                definition.name.as_str(),
                definition.range,
            ))
        })
        .collect()
}

pub(crate) fn style_selector_definitions_for_source_candidates(
    state: &dyn LspQueryReadView,
    candidates: &[LspStyleHoverCandidate],
    workspace_folder_uri: Option<&str>,
) -> Vec<(String, LspStyleHoverCandidate)> {
    let mut definitions = candidates
        .iter()
        .flat_map(|candidate| {
            style_selector_definitions_for_source_candidate(state, candidate, workspace_folder_uri)
        })
        .collect::<Vec<_>>();
    definitions.sort_by_key(|(uri, definition)| {
        (
            uri.clone(),
            definition.range.start.line,
            definition.range.start.character,
            definition.name.clone(),
        )
    });
    definitions.dedup_by(|left, right| {
        left.0 == right.0 && left.1.name == right.1.name && left.1.range == right.1.range
    });
    definitions
}

fn source_candidate_definition_lookup_name(candidate: &LspStyleHoverCandidate) -> &str {
    if candidate.kind == "sourceSelectorPrefixReference" {
        ""
    } else {
        candidate.name.as_str()
    }
}

pub(crate) fn first_style_document_for_workspace<'a>(
    state: &'a dyn LspQueryReadView,
    workspace_folder_uri: Option<&str>,
) -> Option<(String, &'a LspTextDocumentState)> {
    state
        .query_documents()
        .values()
        .filter(|document| document_has_style_index(document))
        .filter(|document| workspace_folder_compatible(workspace_folder_uri, document))
        .map(|document| (document.uri.clone(), document.as_ref()))
        .next()
}
