use std::collections::BTreeSet;

use super::*;

pub fn summarize_omena_query_style_completion_at_position(
    style_uri: &str,
    source: &str,
    position: ParserPositionV0,
    candidates: &[OmenaQueryStyleHoverCandidateV0],
) -> OmenaQueryCompletionAtPositionV0 {
    let (context_kind, prefix) =
        style_completion_context_at_position(source, position).unwrap_or(("styleDocument", None));
    let mut emitted_labels = BTreeSet::new();
    let mut items = candidates
        .iter()
        .filter_map(|candidate| {
            let (label, detail, item_kind) = match candidate.kind {
                "selector" if context_kind == "styleDocument" => (
                    format!(".{}", candidate.name),
                    "CSS Module selector",
                    "cssModuleSelector",
                ),
                "customPropertyDeclaration" => (
                    candidate.name.clone(),
                    "CSS custom property",
                    "cssCustomProperty",
                ),
                _ => return None,
            };
            if prefix
                .as_deref()
                .is_some_and(|prefix| !label.starts_with(prefix))
            {
                return None;
            }
            if !emitted_labels.insert(label.clone()) {
                return None;
            }
            Some(OmenaQueryCompletionItemV0 {
                insert_text: label.clone(),
                label,
                detail,
                item_kind,
                source: "omenaQueryCompletionAtPosition",
            })
        })
        .collect::<Vec<_>>();
    items.sort_by_key(|item| item.label.clone());

    OmenaQueryCompletionAtPositionV0 {
        schema_version: "0",
        product: "omena-query.completion-at",
        file_uri: style_uri.to_string(),
        file_kind: "style",
        query_position: position,
        context_kind,
        prefix,
        is_incomplete: false,
        item_count: items.len(),
        items,
        ready_surfaces: vec!["styleCompletionAt"],
    }
}

pub fn summarize_omena_query_source_completion_at_position(
    source_uri: &str,
    position: ParserPositionV0,
    candidates: &[OmenaQueryCompletionCandidateV0],
    target_style_uri: Option<&str>,
    value_prefix: Option<&str>,
) -> OmenaQueryCompletionAtPositionV0 {
    let mut emitted_labels = BTreeSet::new();
    let mut items = candidates
        .iter()
        .filter(|candidate| candidate.kind == "selector")
        .filter(|candidate| {
            target_style_uri.is_none_or(|target_uri| candidate.file_uri == target_uri)
        })
        .filter(|candidate| value_prefix.is_none_or(|prefix| candidate.name.starts_with(prefix)))
        .filter_map(|candidate| {
            if !emitted_labels.insert(candidate.name.clone()) {
                return None;
            }
            Some(OmenaQueryCompletionItemV0 {
                label: candidate.name.clone(),
                insert_text: candidate.name.clone(),
                detail: "CSS Module selector",
                item_kind: "cssModuleSelector",
                source: "omenaQueryCompletionAtPosition",
            })
        })
        .collect::<Vec<_>>();
    items.sort_by_key(|item| item.label.clone());

    OmenaQueryCompletionAtPositionV0 {
        schema_version: "0",
        product: "omena-query.completion-at",
        file_uri: source_uri.to_string(),
        file_kind: "source",
        query_position: position,
        context_kind: if target_style_uri.is_some() {
            "sourceCssModuleTarget"
        } else {
            "sourceClassToken"
        },
        prefix: value_prefix.map(ToString::to_string),
        is_incomplete: false,
        item_count: items.len(),
        items,
        ready_surfaces: vec!["sourceCompletionAt", "bridgeAwareSelectorCompletion"],
    }
}

pub fn summarize_omena_query_style_completion_for_workspace_file(
    target_style_path: &str,
    style_source: &str,
    position: ParserPositionV0,
) -> OmenaQueryCompletionAtPositionV0 {
    let candidates = summarize_omena_query_style_hover_candidates(target_style_path, style_source)
        .map(|summary| summary.candidates)
        .unwrap_or_default();
    summarize_omena_query_style_completion_at_position(
        target_style_path,
        style_source,
        position,
        candidates.as_slice(),
    )
}

pub fn summarize_omena_query_source_completion_for_workspace_file(
    source_path: &str,
    position: ParserPositionV0,
    style_sources: &[OmenaQueryStyleSourceInputV0],
    target_style_uri: Option<&str>,
    value_prefix: Option<&str>,
) -> OmenaQueryCompletionAtPositionV0 {
    let candidates = collect_omena_query_completion_candidates(style_sources);
    summarize_omena_query_source_completion_at_position(
        source_path,
        position,
        candidates.as_slice(),
        target_style_uri,
        value_prefix,
    )
}

fn collect_omena_query_completion_candidates(
    style_sources: &[OmenaQueryStyleSourceInputV0],
) -> Vec<OmenaQueryCompletionCandidateV0> {
    let mut candidates = Vec::new();
    for source in style_sources {
        let Some(summary) = summarize_omena_query_style_hover_candidates(
            source.style_path.as_str(),
            source.style_source.as_str(),
        ) else {
            continue;
        };
        candidates.extend(summary.candidates.into_iter().filter_map(|candidate| {
            (candidate.kind == "selector").then(|| OmenaQueryCompletionCandidateV0 {
                file_uri: source.style_path.clone(),
                name: candidate.name,
                kind: "selector",
                range: candidate.range,
                source: "omenaQueryStyleHoverCandidates",
            })
        }));
    }
    candidates.sort_by_key(|candidate| {
        (
            candidate.file_uri.clone(),
            candidate.range.start.line,
            candidate.range.start.character,
            candidate.name.clone(),
        )
    });
    candidates.dedup_by(|left, right| {
        left.file_uri == right.file_uri && left.name == right.name && left.range == right.range
    });
    candidates
}
