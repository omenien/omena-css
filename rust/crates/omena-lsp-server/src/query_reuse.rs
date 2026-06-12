use crate::{
    LspShellState, LspStyleDocumentSummary, LspStyleHoverCandidate, LspTextDocumentState,
    build_source_syntax_index, collect_source_imports, collect_style_hover_candidates,
    protocol::{
        byte_offset_for_parser_position, is_style_document_uri, parser_range_for_byte_span,
    },
    source_selector_candidates_from_index,
    state::LspCascadeNarrowingSubstrateMemo,
    summarize_style_document,
};
use omena_query::{
    OmenaQueryStyleCascadeNarrowingSubstrateV0, OmenaQueryStyleResolutionInputsV0,
    OmenaQueryStyleSourceInputV0, ParserByteSpanV0,
    collect_omena_query_style_cascade_narrowing_substrate,
};
use omena_sif::compute_omena_sif_leaf_hash_v1;
use serde::Serialize;
use std::sync::Arc;

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
            "workspaceStyleResolutionInputs",
            "styleDocumentSummary",
            "styleHoverCandidates",
            "optimizingTierFeedback",
            "sourceSyntaxIndex",
            "sourceSelectorCandidates",
            "sourceTypeFactCache",
            "sourceDocumentIndexSidecar",
            "sourceSelectorOccurrenceSidecar",
            "cascadeNarrowingSubstrate",
        ],
        invalidation_policy: vec![
            "refreshOnDocumentOpen",
            "refreshOnDocumentContentChange",
            "refreshOnWorkspaceFileReload",
            "refreshOnResolutionConfigChange",
            "refreshOnResolutionSettingsChange",
            "rebuildCascadeNarrowingSubstrateOnInputContentMismatch",
            "rebuildSourceTypeFactCacheOnContentConfigOrWorkspaceSourceMismatch",
            "rebuildSourceDocumentIndexSidecarOnTextResolutionOrLanguageMismatch",
            "rebuildSourceSelectorOccurrenceSidecarOnDocumentKeyMismatch",
        ],
        request_path_policy: vec![
            "noPackageManifestOrConfigReadOnProviderRequest",
            "noRawSourceRescanOnProviderRequest",
            "noStyleSelectorRescanOnProviderRequest",
            "typeFactRefreshConsumesCacheBeforeTsgoTransport",
            "providerRequestsConsumeDocumentIndexes",
        ],
    }
}

/// Get-or-build the cascade-narrowing substrate for this exact narrowing input set
/// (rfcs#63 E-ii). A hit returns the memoized substrate (zero re-collections); a miss
/// rebuilds and replaces the memo. The compare is exact content equality — cheap next
/// to the collection pass it avoids — so reuse can never serve stale narrowing inputs.
///
/// RFC 0009 Pillar A (rfcs#67): the memo slot is shared between the loop and
/// dispatched query workers, so the mutex is held ONLY to compare and to store —
/// never across the collection pass. Two threads racing the same miss both build
/// (work duplicated, never wrong) and the last store wins; the next caller with
/// the same inputs hits whichever copy landed last.
pub(crate) fn cascade_narrowing_substrate_for_style_sources(
    state: &LspShellState,
    style_sources: &[OmenaQueryStyleSourceInputV0],
    resolution_inputs: &OmenaQueryStyleResolutionInputsV0,
) -> Arc<OmenaQueryStyleCascadeNarrowingSubstrateV0> {
    let package_manifests = state.resolution.package_manifests.as_slice();
    {
        let memo = state.cascade_narrowing_substrate_memo_lock();
        if let Some(memo) = memo.as_ref()
            && memo.style_sources.as_slice() == style_sources
            && memo.package_manifests.as_slice() == package_manifests
            && &memo.resolution_inputs == resolution_inputs
        {
            return Arc::clone(&memo.substrate);
        }
    }
    let substrate = Arc::new(collect_omena_query_style_cascade_narrowing_substrate(
        style_sources,
        package_manifests,
        resolution_inputs,
    ));
    *state.cascade_narrowing_substrate_memo_lock() = Some(LspCascadeNarrowingSubstrateMemo {
        style_sources: style_sources.to_vec(),
        package_manifests: package_manifests.to_vec(),
        resolution_inputs: resolution_inputs.clone(),
        substrate: Arc::clone(&substrate),
    });
    substrate
}

pub(crate) fn refresh_document_reusable_indexes(
    document: &mut LspTextDocumentState,
    resolution_inputs: &OmenaQueryStyleResolutionInputsV0,
) {
    document.optimizing_tier_feedback = None;
    document.text_hash = compute_omena_sif_leaf_hash_v1(document.text.as_bytes())
        .as_str()
        .to_string();
    if is_style_document_uri(document.uri.as_str()) {
        document.style_summary =
            summarize_style_document(document.uri.as_str(), Some(document.text.as_str()));
        document.style_candidates =
            collect_style_hover_candidates(document.uri.as_str(), document.text.as_str())
                .map(|(_, candidates)| candidates)
                .unwrap_or_default();
    } else if let Some((summary, candidates)) = collect_vue_embedded_module_style_indexes(document)
    {
        document.style_summary = Some(summary);
        document.style_candidates = candidates;
    } else {
        document.style_summary = None;
        document.style_candidates = Vec::new();
    }
    let source_syntax_index = build_source_syntax_index(document, resolution_inputs);
    document.has_unresolved_style_import =
        collect_source_imports(document, resolution_inputs).has_unresolved_style_import;
    document.source_selector_candidates =
        source_selector_candidates_from_index(document, &source_syntax_index);
    document.source_syntax_index = source_syntax_index;
}

fn collect_vue_embedded_module_style_indexes(
    document: &LspTextDocumentState,
) -> Option<(LspStyleDocumentSummary, Vec<LspStyleHoverCandidate>)> {
    let embedded = embedded_vue_module_style(document)?;
    let summary =
        summarize_style_document(embedded.virtual_uri.as_str(), Some(embedded.style_source))?;
    let (_, candidates) =
        collect_style_hover_candidates(embedded.virtual_uri.as_str(), embedded.style_source)?;
    let candidates = candidates
        .into_iter()
        .filter_map(|mut candidate| {
            candidate.range = embedded_range_to_document_range(
                document.text.as_str(),
                embedded.style_source,
                embedded.content_start,
                candidate.range,
            )?;
            Some(candidate)
        })
        .collect();
    Some((summary, candidates))
}

struct EmbeddedVueModuleStyle<'a> {
    virtual_uri: String,
    style_source: &'a str,
    content_start: usize,
}

fn embedded_vue_module_style(
    document: &LspTextDocumentState,
) -> Option<EmbeddedVueModuleStyle<'_>> {
    if document.language_id != "vue" && !document.uri.ends_with(".vue") {
        return None;
    }

    let lower = document.text.to_ascii_lowercase();
    let mut cursor = 0usize;
    while let Some(relative_start) = lower[cursor..].find("<style") {
        let tag_start = cursor + relative_start;
        let relative_tag_end = lower[tag_start..].find('>')?;
        let tag_end = tag_start + relative_tag_end + 1;
        let tag = &lower[tag_start..tag_end];
        let close_start = lower[tag_end..].find("</style>")? + tag_end;
        let content_start = tag_end;
        let content_end = close_start;
        if tag.contains("module") {
            return Some(EmbeddedVueModuleStyle {
                virtual_uri: format!(
                    "{}{}",
                    document.uri,
                    vue_embedded_style_virtual_extension(tag)
                ),
                style_source: &document.text[content_start..content_end],
                content_start,
            });
        }
        cursor = close_start + "</style>".len();
    }
    None
}

fn vue_embedded_style_virtual_extension(tag: &str) -> &'static str {
    if tag.contains("lang=\"scss\"")
        || tag.contains("lang='scss'")
        || tag.contains("lang=scss")
        || tag.contains("lang=\"sass\"")
        || tag.contains("lang='sass'")
        || tag.contains("lang=sass")
    {
        ".module.scss"
    } else if tag.contains("lang=\"less\"")
        || tag.contains("lang='less'")
        || tag.contains("lang=less")
    {
        ".module.less"
    } else {
        ".module.css"
    }
}

fn embedded_range_to_document_range(
    document_source: &str,
    embedded_source: &str,
    content_start: usize,
    range: omena_query::ParserRangeV0,
) -> Option<omena_query::ParserRangeV0> {
    let start = content_start + byte_offset_for_parser_position(embedded_source, range.start)?;
    let end = content_start + byte_offset_for_parser_position(embedded_source, range.end)?;
    Some(parser_range_for_byte_span(
        document_source,
        ParserByteSpanV0 { start, end },
    ))
}
