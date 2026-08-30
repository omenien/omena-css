use crate::{
    LspQueryReadView, LspStyleDocumentSummary, LspStyleHoverCandidate, LspTextDocumentState,
    build_source_syntax_index_with_type_fact_attempts, collect_source_imports,
    collect_style_hover_candidates,
    protocol::{
        byte_offset_for_parser_position_with_line_index, file_uri_to_path, is_style_document_uri,
        parser_range_for_byte_span_with_line_index,
    },
    source_selector_candidates_from_index,
    source_type_facts::initial_source_type_fact_tier_attempts,
    state::LspCascadeNarrowingSubstrateMemo,
    summarize_style_document,
};
#[cfg(not(feature = "salsa-style-diagnostics"))]
use omena_query::collect_omena_query_style_cascade_narrowing_substrate_with_external_sifs;
use omena_query::{
    OmenaQueryStyleCascadeNarrowingSubstrateV0, OmenaQueryStyleResolutionInputsV0,
    OmenaQueryStyleSourceInputV0, ParserByteSpanV0, append_omena_query_utility_class_intelligence,
    load_omena_query_workspace_utility_class_intelligence,
    summarize_omena_query_source_binding_index_for_source_language,
};
use omena_sif::compute_omena_sif_leaf_hash_v1;
use omena_syntax::OmenaLineIndexV0;
use serde::Serialize;
use std::sync::Arc;

pub const STYLE_HOVER_INDEX_MAX_SOURCE_BYTES_V0: usize = 64 * 1024;

static OVERSIZED_STYLE_HOVER_INDEX_SKIP_COUNT: std::sync::atomic::AtomicU64 =
    std::sync::atomic::AtomicU64::new(0);

#[cfg(test)]
static OVERSIZED_STYLE_HOVER_INDEX_MEASUREMENT_LOCK: std::sync::Mutex<()> =
    std::sync::Mutex::new(());

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
            "cascadeNarrowingSubstrate",
            "visibleSassSymbolCompletionSubstrate",
        ],
        invalidation_policy: vec![
            "refreshOnDocumentOpen",
            "refreshOnDocumentContentChange",
            "refreshOnWorkspaceFileReload",
            "refreshOnResolutionConfigChange",
            "refreshOnResolutionSettingsChange",
            "rebuildCascadeNarrowingSubstrateOnInputContentMismatch",
            "validateSourceTypeFactCacheAgainstCurrentContentEnvironmentBinaryAndWorkspaceInputs",
            "rebuildSourceDocumentIndexSidecarOnTextResolutionOrLanguageMismatch",
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
    state: &dyn LspQueryReadView,
    style_sources: &[OmenaQueryStyleSourceInputV0],
    resolution_inputs: &OmenaQueryStyleResolutionInputsV0,
) -> Arc<OmenaQueryStyleCascadeNarrowingSubstrateV0> {
    let package_manifests = effective_style_package_manifests(state, resolution_inputs);
    let external_sifs = state.query_resolution().external_sifs.as_slice();
    {
        let memo = state.cascade_narrowing_substrate_memo_lock();
        if let Some(memo) = memo.as_ref()
            && memo.style_sources.as_slice() == style_sources
            && memo.package_manifests == package_manifests
            && memo.external_sifs.as_slice() == external_sifs
            && &memo.resolution_inputs == resolution_inputs
        {
            return Arc::clone(&memo.substrate);
        }
    }
    let substrate = Arc::new(collect_cascade_narrowing_substrate(
        style_sources,
        package_manifests.as_slice(),
        external_sifs,
        resolution_inputs,
    ));
    *state.cascade_narrowing_substrate_memo_lock() = Some(LspCascadeNarrowingSubstrateMemo {
        style_sources: style_sources.to_vec(),
        package_manifests,
        external_sifs: external_sifs.to_vec(),
        resolution_inputs: resolution_inputs.clone(),
        substrate: Arc::clone(&substrate),
    });
    substrate
}

fn collect_cascade_narrowing_substrate(
    style_sources: &[OmenaQueryStyleSourceInputV0],
    package_manifests: &[omena_query::OmenaQueryStylePackageManifestV0],
    external_sifs: &[omena_query::OmenaQueryExternalSifInputV0],
    resolution_inputs: &OmenaQueryStyleResolutionInputsV0,
) -> OmenaQueryStyleCascadeNarrowingSubstrateV0 {
    omena_query::collect_omena_query_style_cascade_narrowing_substrate_with_external_sifs(
        style_sources,
        package_manifests,
        external_sifs,
        resolution_inputs,
    )
}

pub(crate) fn effective_style_package_manifests(
    state: &dyn LspQueryReadView,
    resolution_inputs: &OmenaQueryStyleResolutionInputsV0,
) -> Vec<omena_query::OmenaQueryStylePackageManifestV0> {
    let mut package_manifests = state.query_resolution().package_manifests.clone();
    package_manifests.extend(resolution_inputs.package_manifests.clone());
    package_manifests.sort_by(|left, right| {
        (
            left.package_json_path.as_str(),
            left.package_json_source.as_str(),
        )
            .cmp(&(
                right.package_json_path.as_str(),
                right.package_json_source.as_str(),
            ))
    });
    package_manifests.dedup();
    package_manifests
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
        document.style_candidates = if style_hover_index_is_oversized(document.text.len()) {
            record_oversized_style_hover_index_skip(document.uri.as_str(), document.text.len());
            Vec::new()
        } else {
            collect_style_hover_candidates(document.uri.as_str(), document.text.as_str())
                .map(|(_, candidates)| candidates)
                .unwrap_or_default()
        };
    } else if let Some((summary, candidates)) = collect_vue_embedded_module_style_indexes(document)
    {
        document.style_summary = Some(summary);
        document.style_candidates = candidates;
    } else {
        document.style_summary = None;
        document.style_candidates = Vec::new();
    }
    let source_index_build =
        build_source_syntax_index_with_type_fact_attempts(document, resolution_inputs);
    let mut source_syntax_index = source_index_build.source_syntax_index;
    if !is_style_document_uri(document.uri.as_str())
        && let Some(workspace_root) = document
            .workspace_folder_uri
            .as_deref()
            .and_then(file_uri_to_path)
    {
        let utility_intelligence =
            load_omena_query_workspace_utility_class_intelligence(workspace_root.as_path(), None);
        append_omena_query_utility_class_intelligence(
            &mut source_syntax_index,
            document.text.as_str(),
            &utility_intelligence,
        );
    }
    let source_imports = collect_source_imports(document, resolution_inputs);
    document.has_unresolved_style_import = source_imports.has_unresolved_style_import;
    if is_style_document_uri(document.uri.as_str()) {
        document.source_module_specifiers.clear();
        document.source_module_specifier_index_complete = true;
    } else {
        document.source_module_specifiers =
            summarize_omena_query_source_binding_index_for_source_language(
                document.uri.as_str(),
                document.text.as_str(),
                Some(document.language_id.as_str()),
                source_imports.style_import_resolutions,
            )
            .module_specifiers
            .into_iter()
            .map(|fact| fact.specifier)
            .collect();
        document.source_module_specifier_index_complete = true;
    }
    document.source_selector_candidates =
        source_selector_candidates_from_index(document, &source_syntax_index);
    document.source_syntax_index = source_syntax_index;
    document.source_type_fact_lexical_attempts = source_index_build.type_fact_attempts;
    document.source_type_fact_tier_attempts = initial_source_type_fact_tier_attempts(
        document.source_type_fact_lexical_attempts.as_slice(),
    );
    document.source_type_fact_selector_references.clear();
    document.source_type_fact_retired_prefix_references.clear();
}

fn collect_vue_embedded_module_style_indexes(
    document: &LspTextDocumentState,
) -> Option<(LspStyleDocumentSummary, Vec<LspStyleHoverCandidate>)> {
    let embedded = embedded_vue_module_style(document)?;
    let summary =
        summarize_style_document(embedded.virtual_uri.as_str(), Some(embedded.style_source))?;
    if style_hover_index_is_oversized(embedded.style_source.len()) {
        record_oversized_style_hover_index_skip(
            embedded.virtual_uri.as_str(),
            embedded.style_source.len(),
        );
        return Some((summary, Vec::new()));
    }
    let (_, candidates) =
        collect_style_hover_candidates(embedded.virtual_uri.as_str(), embedded.style_source)?;
    let embedded_line_index = OmenaLineIndexV0::new(embedded.style_source);
    let document_line_index = OmenaLineIndexV0::new(document.text.as_str());
    let candidates = candidates
        .into_iter()
        .filter_map(|mut candidate| {
            candidate.range = embedded_range_to_document_range(
                document.text.as_str(),
                &document_line_index,
                embedded.style_source,
                &embedded_line_index,
                embedded.content_start,
                candidate.range,
            )?;
            Some(candidate)
        })
        .collect();
    Some((summary, candidates))
}

fn style_hover_index_is_oversized(source_bytes: usize) -> bool {
    source_bytes > STYLE_HOVER_INDEX_MAX_SOURCE_BYTES_V0
}

fn record_oversized_style_hover_index_skip(uri: &str, source_bytes: usize) {
    let skip_count = OVERSIZED_STYLE_HOVER_INDEX_SKIP_COUNT
        .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        .saturating_add(1);
    eprintln!(
        "{}",
        serde_json::json!({
            "schemaVersion": "0",
            "product": "omena-lsp-server.style-hover-index-policy",
            "outcome": "skipped",
            "reason": "source-byte-limit",
            "sourceUri": uri,
            "sourceBytes": source_bytes,
            "maximumSourceBytes": STYLE_HOVER_INDEX_MAX_SOURCE_BYTES_V0,
            "oversizedSkipCount": skip_count,
        })
    );
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
    document_line_index: &OmenaLineIndexV0,
    embedded_source: &str,
    embedded_line_index: &OmenaLineIndexV0,
    content_start: usize,
    range: omena_query::ParserRangeV0,
) -> Option<omena_query::ParserRangeV0> {
    let start = content_start
        + byte_offset_for_parser_position_with_line_index(
            embedded_source,
            embedded_line_index,
            range.start,
        )?;
    let end = content_start
        + byte_offset_for_parser_position_with_line_index(
            embedded_source,
            embedded_line_index,
            range.end,
        )?;
    Some(parser_range_for_byte_span_with_line_index(
        document_source,
        document_line_index,
        ParserByteSpanV0 { start, end },
    ))
}

#[cfg(test)]
mod oversized_style_hover_index_tests {
    use super::*;

    #[test]
    fn ordinary_style_source_keeps_hover_indexing_enabled() {
        let _guard = OVERSIZED_STYLE_HOVER_INDEX_MEASUREMENT_LOCK
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        OVERSIZED_STYLE_HOVER_INDEX_SKIP_COUNT.store(0, std::sync::atomic::Ordering::SeqCst);
        let document = crate::lsp_text_document_state(
            "file:///workspace/card.module.css".to_string(),
            Some("file:///workspace".to_string()),
            "css".to_string(),
            1,
            ".card { color: red; }\n".to_string(),
            &OmenaQueryStyleResolutionInputsV0::default(),
        );
        assert_eq!(document.style_candidates.len(), 1);
        assert_eq!(
            OVERSIZED_STYLE_HOVER_INDEX_SKIP_COUNT.load(std::sync::atomic::Ordering::SeqCst),
            0
        );
    }

    #[test]
    fn one_megabyte_style_source_skips_hover_indexing_with_a_counted_policy() {
        let _guard = OVERSIZED_STYLE_HOVER_INDEX_MEASUREMENT_LOCK
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        OVERSIZED_STYLE_HOVER_INDEX_SKIP_COUNT.store(0, std::sync::atomic::Ordering::SeqCst);
        let rule = ".oversized { color: red; }\n";
        let source = rule.repeat((1024 * 1024 / rule.len()) + 1);
        assert!(source.len() >= 1024 * 1024);
        let started = std::time::Instant::now();
        let document = crate::lsp_text_document_state(
            "file:///workspace/oversized.module.css".to_string(),
            Some("file:///workspace".to_string()),
            "css".to_string(),
            1,
            source,
            &OmenaQueryStyleResolutionInputsV0::default(),
        );
        let elapsed = started.elapsed();
        assert!(document.style_summary.is_some());
        assert!(document.style_candidates.is_empty());
        assert_eq!(
            OVERSIZED_STYLE_HOVER_INDEX_SKIP_COUNT.load(std::sync::atomic::Ordering::SeqCst),
            1
        );
        println!(
            "{{\"schemaVersion\":\"0\",\"product\":\"omena-lsp-server.style-hover-index-measurement\",\"fixtureKind\":\"styleDocument\",\"sourceBytes\":{},\"maximumSourceBytes\":{},\"hoverIndexAttempted\":false,\"oversizedSkipCount\":1,\"elapsedMicros\":{}}}",
            document.text.len(),
            STYLE_HOVER_INDEX_MAX_SOURCE_BYTES_V0,
            elapsed.as_micros()
        );
    }

    #[test]
    fn one_megabyte_vue_module_style_skips_hover_indexing_with_a_counted_policy() {
        let _guard = OVERSIZED_STYLE_HOVER_INDEX_MEASUREMENT_LOCK
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        OVERSIZED_STYLE_HOVER_INDEX_SKIP_COUNT.store(0, std::sync::atomic::Ordering::SeqCst);
        let rule = ".embedded { color: blue; }\n";
        let embedded_source = rule.repeat((1024 * 1024 / rule.len()) + 1);
        assert!(embedded_source.len() >= 1024 * 1024);
        let embedded_source_bytes = embedded_source.len();
        let source =
            format!("<template><div /></template>\n<style module>\n{embedded_source}</style>\n");
        let started = std::time::Instant::now();
        let document = crate::lsp_text_document_state(
            "file:///workspace/Oversized.vue".to_string(),
            Some("file:///workspace".to_string()),
            "vue".to_string(),
            1,
            source,
            &OmenaQueryStyleResolutionInputsV0::default(),
        );
        let elapsed = started.elapsed();
        assert!(document.style_summary.is_some());
        assert!(document.style_candidates.is_empty());
        assert_eq!(
            OVERSIZED_STYLE_HOVER_INDEX_SKIP_COUNT.load(std::sync::atomic::Ordering::SeqCst),
            1
        );
        println!(
            "{{\"schemaVersion\":\"0\",\"product\":\"omena-lsp-server.style-hover-index-measurement\",\"fixtureKind\":\"vueEmbeddedStyleModule\",\"documentBytes\":{},\"embeddedSourceBytes\":{},\"maximumSourceBytes\":{},\"hoverIndexAttempted\":false,\"oversizedSkipCount\":1,\"elapsedMicros\":{}}}",
            document.text.len(),
            embedded_source_bytes,
            STYLE_HOVER_INDEX_MAX_SOURCE_BYTES_V0,
            elapsed.as_micros()
        );
    }
}
