use crate::{LspStyleHoverCandidate, LspStyleHoverCandidatesResult, LspTextDocumentState};
use omena_query::{
    OmenaQuerySourceSelectorCandidateV0, OmenaQuerySourceSelectorReferenceCandidateV0,
    OmenaQueryStyleHoverCandidateV0, OmenaQueryStyleSelectorDefinitionV0, ParserPositionV0,
    ParserRangeV0, summarize_omena_query_style_hover_candidates,
};

pub(crate) fn empty_style_hover_candidates_result(
    document_uri: String,
    workspace_folder_uri: Option<String>,
    query_position: Option<ParserPositionV0>,
) -> LspStyleHoverCandidatesResult {
    LspStyleHoverCandidatesResult {
        schema_version: "0",
        product: "omena-lsp-server.style-hover-candidates",
        document_uri,
        workspace_folder_uri,
        language: None,
        query_position,
        candidate_count: 0,
        candidates: Vec::new(),
    }
}

pub(crate) fn collect_style_hover_candidates(
    uri: &str,
    text: &str,
) -> Option<(&'static str, Vec<LspStyleHoverCandidate>)> {
    let summary = summarize_omena_query_style_hover_candidates(uri, text)?;
    Some((
        summary.language,
        summary
            .candidates
            .into_iter()
            .map(lsp_style_hover_candidate_from_query)
            .collect(),
    ))
}

pub(crate) fn lsp_style_hover_candidate_from_query(
    candidate: OmenaQueryStyleHoverCandidateV0,
) -> LspStyleHoverCandidate {
    LspStyleHoverCandidate {
        kind: candidate.kind,
        name: candidate.name,
        range: candidate.range,
        source: candidate.source,
        target_style_uri: None,
        namespace: candidate.namespace,
    }
}

pub(crate) fn query_style_hover_candidate_from_lsp(
    candidate: &LspStyleHoverCandidate,
) -> OmenaQueryStyleHoverCandidateV0 {
    OmenaQueryStyleHoverCandidateV0 {
        kind: candidate.kind,
        name: candidate.name.clone(),
        range: candidate.range,
        source: candidate.source,
        namespace: candidate.namespace.clone(),
    }
}

pub(crate) fn query_source_selector_candidate_from_lsp(
    candidate: &LspStyleHoverCandidate,
) -> OmenaQuerySourceSelectorCandidateV0 {
    OmenaQuerySourceSelectorCandidateV0 {
        kind: candidate.kind,
        name: candidate.name.clone(),
        range: candidate.range,
        source: candidate.source,
        target_style_uri: candidate.target_style_uri.clone(),
    }
}

pub(crate) fn query_source_selector_candidate_for_matching(
    candidate: &LspStyleHoverCandidate,
) -> OmenaQuerySourceSelectorCandidateV0 {
    let mut query_candidate = query_source_selector_candidate_from_lsp(candidate);
    query_candidate.target_style_uri =
        canonical_query_target_style_uri(query_candidate.target_style_uri);
    query_candidate
}

pub(crate) fn lsp_source_selector_candidate_from_query(
    candidate: OmenaQuerySourceSelectorCandidateV0,
) -> LspStyleHoverCandidate {
    LspStyleHoverCandidate {
        kind: candidate.kind,
        name: candidate.name,
        range: candidate.range,
        source: candidate.source,
        target_style_uri: candidate.target_style_uri,
        namespace: None,
    }
}

pub(crate) fn query_style_selector_definition(
    uri: &str,
    definition: &LspStyleHoverCandidate,
) -> OmenaQueryStyleSelectorDefinitionV0 {
    OmenaQueryStyleSelectorDefinitionV0 {
        uri: uri.to_string(),
        name: definition.name.clone(),
        range: definition.range,
    }
}

pub(crate) fn query_style_selector_definition_for_matching(
    uri: &str,
    definition: &LspStyleHoverCandidate,
) -> OmenaQueryStyleSelectorDefinitionV0 {
    let mut query_definition = query_style_selector_definition(uri, definition);
    query_definition.uri = canonical_query_uri(uri);
    query_definition
}

pub(crate) fn query_source_selector_reference_candidate(
    document: &LspTextDocumentState,
    candidate: &LspStyleHoverCandidate,
) -> OmenaQuerySourceSelectorReferenceCandidateV0 {
    OmenaQuerySourceSelectorReferenceCandidateV0 {
        uri: document.uri.clone(),
        kind: candidate.kind,
        name: candidate.name.clone(),
        range: candidate.range,
        source: candidate.source,
        target_style_uri: candidate.target_style_uri.clone(),
    }
}

pub(crate) fn query_source_selector_reference_candidate_for_matching(
    document: &LspTextDocumentState,
    candidate: &LspStyleHoverCandidate,
) -> OmenaQuerySourceSelectorReferenceCandidateV0 {
    let mut reference = query_source_selector_reference_candidate(document, candidate);
    reference.target_style_uri = canonical_query_target_style_uri(reference.target_style_uri);
    reference
}

pub(crate) fn query_definition_identity(
    uri: &str,
    name: &str,
    range: ParserRangeV0,
) -> (String, String, usize, usize, usize, usize) {
    (
        canonical_query_uri(uri),
        name.to_string(),
        range.start.line,
        range.start.character,
        range.end.line,
        range.end.character,
    )
}

pub(crate) fn query_target_style_uri_for_matching(uri: Option<&str>) -> Option<String> {
    uri.map(canonical_query_uri)
}

fn canonical_query_target_style_uri(uri: Option<String>) -> Option<String> {
    uri.map(|uri| canonical_query_uri(uri.as_str()))
}

fn canonical_query_uri(uri: &str) -> String {
    crate::protocol::canonical_file_uri(uri).unwrap_or_else(|| uri.to_string())
}
