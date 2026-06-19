use omena_query::{
    OmenaQuerySourceSyntaxIndexV0 as SourceSyntaxIndex, OmenaQueryStyleResolutionInputsV0,
};
use omena_sif::compute_omena_sif_leaf_hash_v1;

use crate::{
    LspDocumentOrigin, LspTextDocumentState, is_foreign_style_document_uri,
    query_reuse::refresh_document_reusable_indexes,
    source_syntax_index::source_selector_candidates_from_index,
};

pub(crate) fn lsp_text_document_state(
    uri: String,
    workspace_folder_uri: Option<String>,
    language_id: String,
    version: i64,
    text: String,
    resolution_inputs: &OmenaQueryStyleResolutionInputsV0,
) -> LspTextDocumentState {
    let origin = lsp_document_origin_for_uri(uri.as_str());
    let mut document = LspTextDocumentState {
        uri,
        origin,
        workspace_folder_uri,
        language_id,
        version,
        text,
        text_hash: String::new(),
        style_summary: None,
        diagnostics_schedule_count: 0,
        optimizing_tier_feedback: None,
        style_candidates: Vec::new(),
        source_syntax_index: SourceSyntaxIndex::default(),
        has_unresolved_style_import: false,
        source_selector_candidates: Vec::new(),
    };
    refresh_document_reusable_indexes(&mut document, resolution_inputs);
    document
}

pub(crate) fn lsp_text_document_state_with_source_syntax_index(
    uri: String,
    workspace_folder_uri: Option<String>,
    language_id: String,
    version: i64,
    text: String,
    source_syntax_index: SourceSyntaxIndex,
    has_unresolved_style_import: bool,
) -> LspTextDocumentState {
    let origin = lsp_document_origin_for_uri(uri.as_str());
    let mut document = LspTextDocumentState {
        uri,
        origin,
        workspace_folder_uri,
        language_id,
        version,
        text,
        text_hash: String::new(),
        style_summary: None,
        diagnostics_schedule_count: 0,
        optimizing_tier_feedback: None,
        style_candidates: Vec::new(),
        source_syntax_index: SourceSyntaxIndex::default(),
        has_unresolved_style_import,
        source_selector_candidates: Vec::new(),
    };
    document.text_hash = compute_omena_sif_leaf_hash_v1(document.text.as_bytes())
        .as_str()
        .to_string();
    document.source_selector_candidates =
        source_selector_candidates_from_index(&document, &source_syntax_index);
    document.source_syntax_index = source_syntax_index;
    document
}

fn lsp_document_origin_for_uri(uri: &str) -> LspDocumentOrigin {
    if is_foreign_style_document_uri(uri) {
        LspDocumentOrigin::Foreign
    } else {
        LspDocumentOrigin::Local
    }
}
