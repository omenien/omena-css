use omena_query::{OmenaQuerySourceDocumentInputV0, OmenaQueryStyleSourceInputV0};

use crate::{
    LspQueryReadView,
    protocol::{is_style_document_uri, workspace_folder_compatible},
};

pub(crate) fn style_sources_from_open_documents(
    state: &dyn LspQueryReadView,
    workspace_folder_uri: Option<&str>,
    required_document_uri: Option<&str>,
) -> Vec<OmenaQueryStyleSourceInputV0> {
    let mut sources = state
        .query_documents()
        .values()
        .filter(|document| {
            is_style_document_uri(document.uri.as_str())
                && workspace_folder_compatible(workspace_folder_uri, document)
        })
        .map(|document| OmenaQueryStyleSourceInputV0 {
            style_path: document.uri.clone(),
            style_source: document.text.clone(),
        })
        .collect::<Vec<_>>();
    if let Some(required_document_uri) = required_document_uri
        && !sources
            .iter()
            .any(|source| source.style_path == required_document_uri)
        && let Some(document) = state.document(required_document_uri)
    {
        sources.push(OmenaQueryStyleSourceInputV0 {
            style_path: document.uri.clone(),
            style_source: document.text.clone(),
        });
    }
    sources
}

#[cfg(feature = "salsa-style-diagnostics")]
pub(crate) fn style_path_inputs_from_open_documents(
    state: &dyn LspQueryReadView,
    workspace_folder_uri: Option<&str>,
    required_document_uri: Option<&str>,
) -> Vec<OmenaQueryStyleSourceInputV0> {
    let mut sources = state
        .query_documents()
        .values()
        .filter(|document| {
            is_style_document_uri(document.uri.as_str())
                && workspace_folder_compatible(workspace_folder_uri, document)
        })
        .map(|document| OmenaQueryStyleSourceInputV0 {
            style_path: document.uri.clone(),
            style_source: String::new(),
        })
        .collect::<Vec<_>>();
    if let Some(required_document_uri) = required_document_uri
        && !sources
            .iter()
            .any(|source| source.style_path == required_document_uri)
        && let Some(document) = state.document(required_document_uri)
    {
        sources.push(OmenaQueryStyleSourceInputV0 {
            style_path: document.uri.clone(),
            style_source: String::new(),
        });
    }
    sources
}

pub(crate) fn source_documents_from_open_documents(
    state: &dyn LspQueryReadView,
    workspace_folder_uri: Option<&str>,
) -> Vec<OmenaQuerySourceDocumentInputV0> {
    state
        .query_documents()
        .values()
        .filter(|document| {
            !is_style_document_uri(document.uri.as_str())
                && workspace_folder_compatible(workspace_folder_uri, document)
        })
        .map(|document| OmenaQuerySourceDocumentInputV0 {
            source_path: document.uri.clone(),
            source_source: document.text.clone(),
            source_syntax_index: Some(document.source_syntax_index.clone()),
            has_unresolved_style_import: document.has_unresolved_style_import,
        })
        .collect()
}

pub(crate) fn style_sources_for_hover_render(
    state: &dyn LspQueryReadView,
    workspace_folder_uri: Option<&str>,
    document_uri: &str,
    source: &str,
) -> Vec<OmenaQueryStyleSourceInputV0> {
    let mut style_sources =
        style_sources_from_open_documents(state, workspace_folder_uri, Some(document_uri));
    if !style_sources
        .iter()
        .any(|style_source| style_source.style_path == document_uri)
    {
        style_sources.push(OmenaQueryStyleSourceInputV0 {
            style_path: document_uri.to_string(),
            style_source: source.to_string(),
        });
    }
    style_sources
}
