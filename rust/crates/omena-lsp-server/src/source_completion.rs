use omena_query::{
    OmenaQueryCompletionItemV0,
    OmenaQuerySourceDomainClassReferenceFactV0 as SourceDomainClassReferenceFact,
    OmenaQuerySourceSelectorReferenceMatchKindV0 as SourceSelectorReferenceMatchKind,
    OmenaQuerySourceSelectorReferenceSurfaceV0 as SourceSelectorReferenceSurface,
    OmenaQuerySourceSyntaxIndexV0 as SourceSyntaxIndex,
    OmenaQueryStyleIntelligenceSnapshotV0 as StyleIntelligenceSnapshot, ParserByteSpanV0,
    ParserPositionV0, ParserRangeV0, omena_query_style_intelligence_completions_at_offset,
};

use crate::{
    LspShellState,
    protocol::{
        byte_offset_for_parser_position, file_uri_equivalent, is_css_identifier_continue,
        parser_range_contains_position, parser_range_for_byte_span,
    },
    source_selector_candidates_at_position,
    state::LspTextDocumentState,
};

pub(crate) struct SourceCompletionContext {
    pub(crate) target_style_uri: Option<String>,
    pub(crate) value_prefix: Option<String>,
    pub(crate) preferred_selector_names: Vec<String>,
    pub(crate) domain_option_names: Vec<String>,
}

pub(crate) fn source_completion_context_at_position(
    state: &LspShellState,
    document: &LspTextDocumentState,
    position: ParserPositionV0,
) -> Option<SourceCompletionContext> {
    let offset = byte_offset_for_parser_position(document.text.as_str(), position)?;
    if let Some(reference) = document
        .source_syntax_index
        .domain_class_references
        .iter()
        .find(|reference| offset >= reference.byte_span.start && offset <= reference.byte_span.end)
    {
        return Some(SourceCompletionContext {
            target_style_uri: None,
            value_prefix: source_completion_prefix_from_span(
                document.text.as_str(),
                reference.byte_span,
                offset,
            ),
            preferred_selector_names: Vec::new(),
            domain_option_names: source_domain_reference_option_names(
                &document.source_syntax_index,
                reference,
            ),
        });
    }
    if let Some(target) = document
        .source_syntax_index
        .type_fact_targets
        .iter()
        .find(|target| offset >= target.byte_span.start && offset <= target.byte_span.end)
    {
        return Some(SourceCompletionContext {
            target_style_uri: target.target_style_uri.clone(),
            value_prefix: (!target.prefix.is_empty()).then(|| target.prefix.clone()),
            preferred_selector_names: source_completion_value_domain_selectors_for_target(
                document,
                target.byte_span,
                target.target_style_uri.as_deref(),
            ),
            domain_option_names: Vec::new(),
        });
    }
    if let Some(candidate) = source_selector_candidates_at_position(state, document, position)
        .into_iter()
        .find(|candidate| {
            candidate.kind == "sourceSelectorReference"
                || candidate.kind == "sourceSelectorPrefixReference"
        })
        && let Some(span) = byte_span_for_parser_range(document.text.as_str(), candidate.range)
    {
        return Some(SourceCompletionContext {
            target_style_uri: candidate.target_style_uri.clone(),
            value_prefix: source_completion_prefix_for_terminal_offset(
                document.text.as_str(),
                span,
                offset,
            ),
            preferred_selector_names: Vec::new(),
            domain_option_names: Vec::new(),
        });
    }
    if let Some(access) = document
        .source_syntax_index
        .style_property_accesses
        .iter()
        .find(|access| offset >= access.byte_span.start && offset <= access.byte_span.end)
    {
        let target_style_uri = access
            .target_style_uri
            .clone()
            .or_else(|| source_completion_target_uri_for_span(document, access.byte_span));
        return Some(SourceCompletionContext {
            target_style_uri,
            value_prefix: source_completion_prefix_for_terminal_offset(
                document.text.as_str(),
                access.byte_span,
                offset,
            ),
            preferred_selector_names: Vec::new(),
            domain_option_names: Vec::new(),
        });
    }
    if let Some(reference) = document
        .source_syntax_index
        .selector_references
        .iter()
        .find(|reference| offset >= reference.byte_span.start && offset <= reference.byte_span.end)
    {
        let target_style_uri = reference
            .target_style_uri
            .clone()
            .or_else(|| source_completion_target_uri_for_span(document, reference.byte_span));
        return Some(SourceCompletionContext {
            target_style_uri,
            value_prefix: source_completion_prefix_for_terminal_offset(
                document.text.as_str(),
                reference.byte_span,
                offset,
            ),
            preferred_selector_names: Vec::new(),
            domain_option_names: Vec::new(),
        });
    }
    if document
        .source_syntax_index
        .class_string_literals
        .iter()
        .any(|span| offset >= span.start && offset <= span.end)
    {
        let span = document
            .source_syntax_index
            .class_string_literals
            .iter()
            .find(|span| offset >= span.start && offset <= span.end)
            .copied()?;
        return Some(SourceCompletionContext {
            target_style_uri: None,
            value_prefix: source_completion_class_token_prefix_from_span(
                document.text.as_str(),
                span,
                offset,
            ),
            preferred_selector_names: Vec::new(),
            domain_option_names: Vec::new(),
        });
    }
    None
}

pub(crate) fn source_domain_option_completion_items(
    option_names: &[String],
    value_prefix: Option<&str>,
) -> Vec<OmenaQueryCompletionItemV0> {
    let mut items = option_names
        .iter()
        .filter(|option| value_prefix.is_none_or(|prefix| option.starts_with(prefix)))
        .map(|option| OmenaQueryCompletionItemV0 {
            label: option.clone(),
            insert_text: option.clone(),
            sort_text: format!("00-{option}"),
            detail: "Class value option",
            documentation: None,
            item_kind: "classValueOption",
            ranking_source: "classValueUniverseProvider",
            source: "omenaLspSourceCompletion",
        })
        .collect::<Vec<_>>();
    items.sort_by_key(|item| item.label.clone());
    items.dedup_by(|left, right| left.label == right.label);
    items
}

pub(crate) fn source_domain_reference_option_names(
    index: &SourceSyntaxIndex,
    reference: &SourceDomainClassReferenceFact,
) -> Vec<String> {
    let snapshot = StyleIntelligenceSnapshot::new(index);
    let mut options =
        omena_query_style_intelligence_completions_at_offset(&snapshot, reference.byte_span.start)
            .into_iter()
            .map(|completion| completion.label)
            .collect::<Vec<_>>();
    options.sort();
    options.dedup();
    options
}

fn source_completion_target_uri_for_span(
    document: &LspTextDocumentState,
    span: ParserByteSpanV0,
) -> Option<String> {
    let range = parser_range_for_byte_span(document.text.as_str(), span);
    document
        .source_selector_candidates
        .iter()
        .find(|candidate| {
            candidate.range == range
                || parser_range_contains_position(&candidate.range, range.start)
                || parser_range_contains_position(&candidate.range, range.end)
        })
        .and_then(|candidate| candidate.target_style_uri.clone())
}

fn byte_span_for_parser_range(source: &str, range: ParserRangeV0) -> Option<ParserByteSpanV0> {
    Some(ParserByteSpanV0 {
        start: byte_offset_for_parser_position(source, range.start)?,
        end: byte_offset_for_parser_position(source, range.end)?,
    })
}

fn source_completion_value_domain_selectors_for_target(
    document: &LspTextDocumentState,
    byte_span: ParserByteSpanV0,
    target_style_uri: Option<&str>,
) -> Vec<String> {
    let mut selectors = document
        .source_syntax_index
        .selector_references
        .iter()
        .filter(|reference| {
            reference.byte_span == byte_span
                || (reference.surface
                    == SourceSelectorReferenceSurface::OmenaTsgoTypeFactProjection
                    && reference.byte_span.start <= byte_span.start
                    && reference.byte_span.end >= byte_span.end)
        })
        .filter(|reference| reference.match_kind == SourceSelectorReferenceMatchKind::Exact)
        .filter(|reference| {
            target_style_uri.is_none_or(|target_uri| {
                reference
                    .target_style_uri
                    .as_deref()
                    .is_some_and(|reference_uri| file_uri_equivalent(reference_uri, target_uri))
            })
        })
        .filter_map(|reference| reference.selector_name.clone())
        .collect::<Vec<_>>();
    selectors.sort();
    selectors.dedup();
    selectors
}

fn source_completion_prefix_for_terminal_offset(
    source: &str,
    span: ParserByteSpanV0,
    offset: usize,
) -> Option<String> {
    (offset >= span.end).then(|| source_completion_prefix_from_span(source, span, offset))?
}

fn source_completion_prefix_from_span(
    source: &str,
    span: ParserByteSpanV0,
    offset: usize,
) -> Option<String> {
    let end = offset.min(span.end);
    if end < span.start {
        return None;
    }
    let prefix = source.get(span.start..end)?;
    if prefix.is_empty() {
        return None;
    }
    if prefix.chars().all(is_css_identifier_continue) {
        Some(prefix.to_string())
    } else {
        None
    }
}

fn source_completion_class_token_prefix_from_span(
    source: &str,
    span: ParserByteSpanV0,
    offset: usize,
) -> Option<String> {
    let end = offset.min(span.end);
    if end < span.start {
        return None;
    }
    let prefix = source.get(span.start..end)?;
    let token = prefix
        .rsplit(|ch: char| ch.is_ascii_whitespace())
        .next()
        .unwrap_or_default();
    if token.is_empty() {
        return None;
    }
    if token.chars().all(is_css_identifier_continue) {
        Some(token.to_string())
    } else {
        None
    }
}
