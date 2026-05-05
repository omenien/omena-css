use crate::{LspStyleHoverCandidate, LspStyleHoverCandidatesResult, LspTextDocumentState};
use omena_query::{
    OmenaQuerySourceSelectorCandidateV0, OmenaQuerySourceSelectorReferenceEditTargetV0,
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

pub(crate) fn query_source_selector_reference_edit_target(
    document: &LspTextDocumentState,
    candidate: &LspStyleHoverCandidate,
) -> OmenaQuerySourceSelectorReferenceEditTargetV0 {
    OmenaQuerySourceSelectorReferenceEditTargetV0 {
        uri: document.uri.clone(),
        name: candidate.name.clone(),
        range: candidate.range,
        target_style_uri: candidate.target_style_uri.clone(),
    }
}

pub(crate) fn query_definition_identity(
    uri: &str,
    name: &str,
    range: ParserRangeV0,
) -> (String, String, usize, usize, usize, usize) {
    (
        uri.to_string(),
        name.to_string(),
        range.start.line,
        range.start.character,
        range.end.line,
        range.end.character,
    )
}
