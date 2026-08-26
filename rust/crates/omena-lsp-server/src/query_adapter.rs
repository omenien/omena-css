use crate::{LspStyleHoverCandidate, LspStyleHoverCandidatesResult, LspTextDocumentState};
use omena_query::{
    OmenaQuerySourceSelectorCandidateV0, OmenaQuerySourceSelectorReferenceCandidateV0,
    OmenaQueryStyleHoverCandidateV0, OmenaQueryStyleSelectorDefinitionV0, ParserPositionV0,
    ParserRangeV0, summarize_omena_query_style_hover_candidates,
};
use omena_syntax::ident::{AuthoredPropertyTextV0, PropertyNameV0};

fn lsp_custom_property_key(
    kind: &str,
    name: &str,
) -> Option<omena_syntax::ident::CanonicalCustomPropertyNameV0> {
    kind.starts_with("customProperty")
        .then(|| PropertyNameV0::canonical_custom_key(name))
}

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
        selector_key: candidate.selector_key,
        property_key: candidate.property_key,
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
        selector_key: candidate.selector_key.clone(),
        property_key: candidate.property_key.clone(),
        range: candidate.range,
        source: candidate.source,
        namespace: candidate.namespace.clone(),
    }
}

pub(crate) fn query_source_selector_candidate_from_lsp(
    candidate: &LspStyleHoverCandidate,
) -> OmenaQuerySourceSelectorCandidateV0 {
    let mut name = String::new();
    let _ = omena_syntax::ident::render_authored(&candidate.name, &mut name);
    OmenaQuerySourceSelectorCandidateV0 {
        kind: candidate.kind,
        name,
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
        selector_key: Some(
            omena_syntax::ident::ClassNameV0::new(candidate.name.as_str()).canonical_key(),
        ),
        property_key: lsp_custom_property_key(candidate.kind, candidate.name.as_str()),
        name: AuthoredPropertyTextV0::new(candidate.name),
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
    let mut name = String::new();
    let _ = omena_syntax::ident::render_authored(&definition.name, &mut name);
    OmenaQueryStyleSelectorDefinitionV0 {
        uri: uri.to_string(),
        name,
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
    let mut name = String::new();
    let _ = omena_syntax::ident::render_authored(&candidate.name, &mut name);
    OmenaQuerySourceSelectorReferenceCandidateV0 {
        uri: document.uri.clone(),
        kind: candidate.kind,
        name,
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
