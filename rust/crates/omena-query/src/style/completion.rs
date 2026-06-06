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
    let mut ranked_items = candidates
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
            let (sort_text, ranking_source) =
                style_completion_ranking(context_kind, position, candidate, label.as_str());
            let documentation = style_completion_documentation(source, context_kind, candidate);
            Some(OmenaQueryCompletionItemV0 {
                insert_text: label.clone(),
                label,
                sort_text,
                detail,
                documentation,
                item_kind,
                ranking_source,
                source: "omenaQueryCompletionAtPosition",
            })
        })
        .collect::<Vec<_>>();
    ranked_items.sort_by_key(|item| (item.sort_text.clone(), item.label.clone()));
    let mut emitted_labels = BTreeSet::new();
    let items = ranked_items
        .into_iter()
        .filter(|item| emitted_labels.insert(item.label.clone()))
        .collect::<Vec<_>>();

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
    preferred_selector_names: &[String],
) -> OmenaQueryCompletionAtPositionV0 {
    let preferred_selectors = preferred_selector_names
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
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
            let (sort_text, ranking_source) = source_completion_ranking(
                candidate,
                target_style_uri,
                value_prefix,
                preferred_selectors.contains(candidate.name.as_str()),
            );
            Some(OmenaQueryCompletionItemV0 {
                label: candidate.name.clone(),
                insert_text: candidate.name.clone(),
                sort_text,
                detail: "CSS Module selector",
                documentation: candidate.documentation.clone(),
                item_kind: "cssModuleSelector",
                ranking_source,
                source: "omenaQueryCompletionAtPosition",
            })
        })
        .collect::<Vec<_>>();
    items.sort_by_key(|item| (item.sort_text.clone(), item.label.clone()));

    OmenaQueryCompletionAtPositionV0 {
        schema_version: "0",
        product: "omena-query.completion-at",
        file_uri: source_uri.to_string(),
        file_kind: "source",
        query_position: position,
        context_kind: if target_style_uri.is_some() {
            if preferred_selectors.is_empty() {
                "sourceCssModuleTarget"
            } else {
                "sourceCssModuleValueDomainTarget"
            }
        } else {
            "sourceClassToken"
        },
        prefix: value_prefix.map(ToString::to_string),
        is_incomplete: false,
        item_count: items.len(),
        items,
        ready_surfaces: if preferred_selectors.is_empty() {
            vec!["sourceCompletionAt", "bridgeAwareSelectorCompletion"]
        } else {
            vec![
                "sourceCompletionAt",
                "bridgeAwareSelectorCompletion",
                "valueDomainAwareSelectorCompletion",
            ]
        },
    }
}

fn style_completion_ranking(
    context_kind: &str,
    position: ParserPositionV0,
    candidate: &OmenaQueryStyleHoverCandidateV0,
    label: &str,
) -> (String, &'static str) {
    if context_kind == "styleCustomPropertyReference"
        && candidate.kind == "customPropertyDeclaration"
    {
        let query_key = completion_position_order_key(position);
        let candidate_key = completion_position_order_key(candidate.range.start);
        let (group, distance) = if candidate_key <= query_key {
            (0usize, query_key.saturating_sub(candidate_key))
        } else {
            (1usize, candidate_key.saturating_sub(query_key))
        };
        return (
            format!("{group:02}-{distance:012}-{label}"),
            "sameFileSourceOrderCascade",
        );
    }

    (format!("50-{label}"), "label")
}

fn style_completion_documentation(
    source: &str,
    context_kind: &str,
    candidate: &OmenaQueryStyleHoverCandidateV0,
) -> Option<String> {
    if context_kind != "styleDocument" || candidate.kind != "selector" {
        return None;
    }

    summarize_omena_query_style_completion_candidate_documentation(
        source,
        candidate.kind,
        candidate.name.as_str(),
        candidate.range.start,
    )
}

pub fn summarize_omena_query_style_completion_candidate_documentation(
    source: &str,
    candidate_kind: &str,
    candidate_name: &str,
    candidate_position: ParserPositionV0,
) -> Option<String> {
    if candidate_kind != "selector" {
        return None;
    }
    let render_parts = summarize_omena_query_style_hover_render_parts(
        source,
        candidate_kind,
        candidate_name,
        candidate_position,
    );
    render_property_value_narrowings_markdown(&render_parts.property_value_narrowings)
}

#[allow(clippy::too_many_arguments)]
pub fn summarize_omena_query_style_completion_candidate_documentation_for_workspace_file(
    target_style_path: &str,
    style_sources: &[OmenaQueryStyleSourceInputV0],
    package_manifests: &[OmenaQueryStylePackageManifestV0],
    resolution_inputs: &OmenaQueryStyleResolutionInputsV0,
    candidate_kind: &str,
    candidate_name: &str,
    candidate_position: ParserPositionV0,
) -> Option<String> {
    if candidate_kind != "selector" {
        return None;
    }
    let render_parts = summarize_omena_query_style_hover_render_parts_for_workspace_file(
        target_style_path,
        style_sources,
        package_manifests,
        resolution_inputs,
        candidate_kind,
        candidate_name,
        candidate_position,
    )?;
    render_property_value_narrowings_markdown(&render_parts.property_value_narrowings)
}

fn render_property_value_narrowings_markdown(
    narrowings: &[AbstractPropertyValueNarrowingV0],
) -> Option<String> {
    if narrowings.is_empty() {
        return None;
    }

    let lines = narrowings
        .iter()
        .take(6)
        .map(|narrowing| {
            format!(
                "- `{}`: {}{}",
                narrowing.property_name,
                render_abstract_property_value(&narrowing.value),
                render_property_value_narrowing_context(narrowing)
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    Some(format!("Cascade narrowed values:\n{lines}"))
}

fn render_abstract_property_value(value: &AbstractPropertyValueV0) -> String {
    match value {
        AbstractPropertyValueV0::Bottom { .. } => "`<bottom>`".to_string(),
        AbstractPropertyValueV0::Exact { value, .. } => format!("`{value}`"),
        AbstractPropertyValueV0::FiniteSet { values, .. } => values
            .iter()
            .map(|value| format!("`{value}`"))
            .collect::<Vec<_>>()
            .join(" | "),
        AbstractPropertyValueV0::CustomPropertyReference {
            custom_property_name,
            ..
        } => {
            format!("`var({custom_property_name})`")
        }
        AbstractPropertyValueV0::Top { .. } => "`<top>`".to_string(),
    }
}

fn render_property_value_narrowing_context(narrowing: &AbstractPropertyValueNarrowingV0) -> String {
    let mut context = Vec::new();
    if !narrowing.requested_condition_context.is_empty() {
        context.push(narrowing.requested_condition_context.join(" / "));
    }
    if let Some(layer_name) = narrowing.requested_layer_name.as_deref() {
        context.push(format!("@layer {layer_name}"));
    } else if narrowing.requested_layer_scope == "exactLayer" {
        context.push("unlayered".to_string());
    }

    if context.is_empty() {
        String::new()
    } else {
        format!(" ({})", context.join(", "))
    }
}

fn source_completion_ranking(
    candidate: &OmenaQueryCompletionCandidateV0,
    target_style_uri: Option<&str>,
    value_prefix: Option<&str>,
    preferred_by_value_domain: bool,
) -> (String, &'static str) {
    if preferred_by_value_domain {
        return (
            format!("00-00-{}", candidate.name),
            "valueDomainSelectorProjection",
        );
    }

    let target_rank =
        usize::from(target_style_uri.is_none_or(|target| candidate.file_uri != target));
    let prefix_rank = usize::from(value_prefix.is_none());
    (
        format!("10-{target_rank:02}-{prefix_rank:02}-{}", candidate.name),
        if target_style_uri.is_some() || value_prefix.is_some() {
            "targetAndPrefixNarrowing"
        } else {
            "label"
        },
    )
}

fn completion_position_order_key(position: ParserPositionV0) -> usize {
    position
        .line
        .saturating_mul(1_000_000)
        .saturating_add(position.character)
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
    preferred_selector_names: &[String],
) -> OmenaQueryCompletionAtPositionV0 {
    let resolution_inputs = OmenaQueryStyleResolutionInputsV0::default();
    let candidates =
        collect_omena_query_completion_candidates(style_sources, &[], &resolution_inputs);
    summarize_omena_query_source_completion_at_position(
        source_path,
        position,
        candidates.as_slice(),
        target_style_uri,
        value_prefix,
        preferred_selector_names,
    )
}

#[allow(clippy::too_many_arguments)]
pub fn summarize_omena_query_source_completion_for_workspace_file_with_resolution_inputs(
    source_path: &str,
    position: ParserPositionV0,
    style_sources: &[OmenaQueryStyleSourceInputV0],
    package_manifests: &[OmenaQueryStylePackageManifestV0],
    resolution_inputs: &OmenaQueryStyleResolutionInputsV0,
    target_style_uri: Option<&str>,
    value_prefix: Option<&str>,
    preferred_selector_names: &[String],
) -> OmenaQueryCompletionAtPositionV0 {
    let candidates = collect_omena_query_completion_candidates(
        style_sources,
        package_manifests,
        resolution_inputs,
    );
    summarize_omena_query_source_completion_at_position(
        source_path,
        position,
        candidates.as_slice(),
        target_style_uri,
        value_prefix,
        preferred_selector_names,
    )
}

fn collect_omena_query_completion_candidates(
    style_sources: &[OmenaQueryStyleSourceInputV0],
    package_manifests: &[OmenaQueryStylePackageManifestV0],
    resolution_inputs: &OmenaQueryStyleResolutionInputsV0,
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
            if candidate.kind != "selector" {
                return None;
            }
            let documentation =
                summarize_omena_query_style_completion_candidate_documentation_for_workspace_file(
                    source.style_path.as_str(),
                    style_sources,
                    package_manifests,
                    resolution_inputs,
                    candidate.kind,
                    candidate.name.as_str(),
                    candidate.range.start,
                )
                .or_else(|| {
                    summarize_omena_query_style_completion_candidate_documentation(
                        source.style_source.as_str(),
                        candidate.kind,
                        candidate.name.as_str(),
                        candidate.range.start,
                    )
                });
            Some(OmenaQueryCompletionCandidateV0 {
                file_uri: source.style_path.clone(),
                name: candidate.name,
                kind: "selector",
                range: candidate.range,
                source: "omenaQueryStyleHoverCandidates",
                documentation,
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
