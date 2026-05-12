use super::*;

pub fn summarize_omena_resolver_query_fragments(
    input: &EngineInputV2,
) -> SourceResolutionQueryFragmentsV0 {
    summarize_source_resolution_query_fragments_input(input)
}

pub fn summarize_omena_resolver_canonical_producer_signal(
    input: &EngineInputV2,
) -> SourceResolutionCanonicalProducerSignalV0 {
    summarize_source_resolution_canonical_producer_signal_input(input)
}

pub fn summarize_omena_resolver_source_resolution_runtime(
    input: &EngineInputV2,
) -> OmenaResolverSourceResolutionRuntimeIndexV0 {
    let canonical_signal = summarize_omena_resolver_canonical_producer_signal(input);
    let mut candidates_by_expression = BTreeMap::<String, SourceResolutionCandidateV0>::new();

    for candidate in canonical_signal.canonical_bundle.candidates {
        candidates_by_expression.insert(candidate.expression_id.clone(), candidate);
    }

    let entries = canonical_signal
        .canonical_bundle
        .query_fragments
        .iter()
        .map(|fragment| {
            runtime_source_resolution_entry_from_fragment(
                fragment,
                candidates_by_expression.get(&fragment.expression_id),
            )
        })
        .collect::<Vec<_>>();
    let resolved_expression_count = entries
        .iter()
        .filter(|entry| entry.can_resolve_source_expression)
        .count();
    let unresolved_expression_count = entries.len() - resolved_expression_count;
    let mut blocking_gaps = Vec::new();

    if entries.is_empty() {
        blocking_gaps.push("emptySourceResolutionRuntimeIndex");
    }
    if unresolved_expression_count > 0 {
        blocking_gaps.push("unresolvedSourceExpressions");
    }

    OmenaResolverSourceResolutionRuntimeIndexV0 {
        schema_version: "0",
        product: "omena-resolver.source-resolution-runtime-index",
        input_product: "engine-input-producers.source-resolution-canonical-producer",
        input_version: canonical_signal.input_version,
        expression_count: entries.len(),
        resolved_expression_count,
        unresolved_expression_count,
        blocking_gaps,
        entries,
    }
}

pub fn query_omena_resolver_source_expression(
    runtime_index: &OmenaResolverSourceResolutionRuntimeIndexV0,
    expression_id: &str,
) -> Option<OmenaResolverSourceResolutionRuntimeEntryV0> {
    runtime_index
        .entries
        .iter()
        .find(|entry| entry.expression_id == expression_id)
        .cloned()
}

fn runtime_source_resolution_entry_from_fragment(
    fragment: &SourceResolutionQueryFragmentV0,
    candidate: Option<&SourceResolutionCandidateV0>,
) -> OmenaResolverSourceResolutionRuntimeEntryV0 {
    let selector_names = candidate
        .map(|candidate| candidate.selector_names.clone())
        .unwrap_or_default();
    let finite_values = candidate.and_then(|candidate| candidate.finite_values.clone());
    let has_selector_match = !selector_names.is_empty();
    let has_finite_values = finite_values
        .as_ref()
        .is_some_and(|values| !values.is_empty());

    OmenaResolverSourceResolutionRuntimeEntryV0 {
        query_id: fragment.query_id.clone(),
        expression_id: fragment.expression_id.clone(),
        expression_kind: fragment.expression_kind.clone(),
        style_file_path: fragment.style_file_path.clone(),
        selector_names,
        finite_values,
        selector_certainty: candidate
            .map(|candidate| candidate.selector_certainty.clone())
            .unwrap_or_else(|| "unresolved".to_string()),
        value_certainty: candidate.and_then(|candidate| candidate.value_certainty.clone()),
        selector_certainty_shape_kind: candidate
            .map(|candidate| candidate.selector_certainty_shape_kind.clone())
            .unwrap_or_else(|| "missingTypeFacts".to_string()),
        value_certainty_shape_kind: candidate
            .map(|candidate| candidate.value_certainty_shape_kind.clone())
            .unwrap_or_else(|| "missingTypeFacts".to_string()),
        has_selector_match,
        has_finite_values,
        can_resolve_source_expression: has_selector_match,
        status: if has_selector_match {
            "resolved"
        } else if candidate.is_some() {
            "unresolvedSelectorSet"
        } else {
            "missingTypeFacts"
        },
    }
}
