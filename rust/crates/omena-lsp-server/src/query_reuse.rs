use crate::{
    LspTextDocumentState, build_source_syntax_index, collect_style_hover_candidates,
    protocol::is_style_document_uri, source_selector_candidates_from_index,
    summarize_style_document,
};
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RustQueryReuseBoundaryV0 {
    pub product: &'static str,
    pub owner: &'static str,
    pub reuse_model: &'static str,
    pub cached_surfaces: Vec<&'static str>,
    pub invalidation_policy: Vec<&'static str>,
    pub request_path_policy: Vec<&'static str>,
}

pub fn rust_query_reuse_contract() -> RustQueryReuseBoundaryV0 {
    RustQueryReuseBoundaryV0 {
        product: "omena-lsp-server.query-reuse",
        owner: "omena-lsp-server/documentQueryReuse",
        reuse_model: "documentRevisionOwnedReusableIndexes",
        cached_surfaces: vec![
            "styleDocumentSummary",
            "styleHoverCandidates",
            "sourceSyntaxIndex",
            "sourceSelectorCandidates",
        ],
        invalidation_policy: vec![
            "refreshOnDocumentOpen",
            "refreshOnDocumentContentChange",
            "refreshOnWorkspaceFileReload",
            "refreshOnResolutionConfigChange",
        ],
        request_path_policy: vec![
            "noRawSourceRescanOnProviderRequest",
            "noStyleSelectorRescanOnProviderRequest",
            "providerRequestsConsumeDocumentIndexes",
        ],
    }
}

pub(crate) fn refresh_document_reusable_indexes(document: &mut LspTextDocumentState) {
    if is_style_document_uri(document.uri.as_str()) {
        document.style_summary =
            summarize_style_document(document.uri.as_str(), Some(document.text.as_str()));
        document.style_candidates =
            collect_style_hover_candidates(document.uri.as_str(), document.text.as_str())
                .map(|(_, candidates)| candidates)
                .unwrap_or_default();
    } else {
        document.style_summary = None;
        document.style_candidates = Vec::new();
    }
    let source_syntax_index = build_source_syntax_index(document);
    document.source_selector_candidates =
        source_selector_candidates_from_index(document, &source_syntax_index);
    document.source_syntax_index = source_syntax_index;
}
