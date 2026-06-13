use super::shared::*;

pub fn summarize_omena_query_missing_custom_property_diagnostics(
    style_uri: &str,
    source: &str,
    candidates: &[OmenaQueryStyleHoverCandidateV0],
) -> Vec<OmenaQueryStyleDiagnosticV0> {
    let declaration_names = candidates
        .iter()
        .filter(|candidate| candidate.kind == "customPropertyDeclaration")
        .map(|candidate| candidate.name.as_str())
        .collect::<BTreeSet<_>>();
    if declaration_names.is_empty() {
        return Vec::new();
    }

    // `var(--x, fallback)` references cannot be "missing" in any observable way — the
    // fallback guarantees a value — so suppress the lint per-reference. The fallback fact
    // range and the candidate range both derive from the same parser byte span via
    // `parser_range_for_byte_span`, so matching on the rendered range scopes the suppression
    // to the exact `var()` argument (a nested fallback-less `var(--b)` in
    // `var(--a, var(--b))` stays a live candidate).
    let dialect = omena_parser_dialect_for_style_path(style_uri);
    let facts = collect_omena_query_omena_parser_style_facts_raw(source, dialect);
    let fallback_ranges = facts
        .variables
        .iter()
        .filter(|fact| {
            fact.kind == ParsedVariableFactKind::CustomPropertyReference && fact.has_fallback
        })
        .map(|fact| {
            let byte_span = ParserByteSpanV0 {
                start: u32::from(fact.range.start()) as usize,
                end: u32::from(fact.range.end()) as usize,
            };
            (
                fact.name.clone(),
                parser_range_for_byte_span(source, byte_span),
            )
        })
        .collect::<BTreeSet<_>>();

    let insertion_range = end_of_source_range(source);
    candidates
        .iter()
        .filter(|candidate| {
            candidate.kind == "customPropertyReference"
                && !declaration_names.contains(candidate.name.as_str())
                && !fallback_ranges.contains(&(candidate.name.clone(), candidate.range))
        })
        .map(|candidate| OmenaQueryStyleDiagnosticV0 {
            code: "missingCustomProperty",
            severity: "warning",
            provenance: vec![
                "omena-parser.custom-property-facts",
                "omena-query.style-diagnostics",
            ],
            range: candidate.range,
            message: format!(
                "CSS custom property '{}' not found in indexed style tokens.",
                candidate.name
            ),
            tags: Vec::new(),
            create_custom_property: Some(OmenaQueryCreateCustomPropertyActionV0 {
                uri: style_uri.to_string(),
                range: insertion_range,
                new_text: format!("\n\n:root {{\n  {}: ;\n}}\n", candidate.name),
                property_name: candidate.name.clone(),
            }),
            cascade_narrowing: None,
            cascade_confidence: None,
            polynomial_provenance: None,
            cross_file_scc: None,
        })
        .collect()
}

pub fn summarize_omena_query_cascade_aware_style_diagnostics(
    style_uri: &str,
    source: &str,
    candidates: &[OmenaQueryStyleHoverCandidateV0],
) -> Vec<OmenaQueryStyleDiagnosticV0> {
    summarize_omena_query_cascade_aware_style_diagnostics_with_deep_analysis(
        style_uri, source, candidates, false,
    )
}

/// Cascade-aware diagnostics with an explicit opt-in deep-analysis switch. With
/// `deep_analysis == false` (the default surface) only the product cascade gate
/// diagnostics are emitted; `deep_analysis == true` additionally surfaces the
/// rg-flow / categorical theory hints, deduplicated against `circularVar`.
pub fn summarize_omena_query_cascade_aware_style_diagnostics_with_deep_analysis(
    style_uri: &str,
    source: &str,
    candidates: &[OmenaQueryStyleHoverCandidateV0],
    deep_analysis: bool,
) -> Vec<OmenaQueryStyleDiagnosticV0> {
    let declarations_by_name = candidates
        .iter()
        .filter(|candidate| candidate.kind == "customPropertyDeclaration")
        .map(|candidate| (candidate.name.as_str(), candidate.range))
        .collect::<BTreeMap<_, _>>();

    let dialect = omena_parser_dialect_for_style_path(style_uri);
    let mut diagnostics =
        summarize_static_css_custom_property_fixed_point_from_source(source, dialect)
            .entries
            .into_iter()
            .filter(|entry| entry.guaranteed_invalid)
            .filter_map(|entry| {
                declarations_by_name
                    .get(entry.name.as_str())
                    .copied()
                    .map(|range| OmenaQueryStyleDiagnosticV0 {
                        code: "guaranteedInvalidCustomProperty",
                        severity: "warning",
                        provenance: vec![
                            "omena-transform-passes.custom-property-lfp",
                            "omena-query.cascade-aware-diagnostics",
                        ],
                        range,
                        message: format!(
                            "CSS custom property '{}' resolves to the guaranteed-invalid value.",
                            entry.name
                        ),
                        tags: Vec::new(),
                        create_custom_property: None,
                        cascade_narrowing: None,
                        cascade_confidence: None,
                        polynomial_provenance: None,
                        cross_file_scc: None,
                    })
            })
            .collect::<Vec<_>>();

    diagnostics.extend(
        summarize_query_cascade_checker_diagnostics_with_deep_analysis(
            style_uri,
            source,
            deep_analysis,
        ),
    );

    diagnostics
}

pub fn summarize_omena_query_missing_keyframes_diagnostics(
    style_uri: &str,
    source: &str,
) -> Vec<OmenaQueryStyleDiagnosticV0> {
    let dialect = omena_parser_dialect_for_style_path(style_uri);
    let facts = collect_omena_query_omena_parser_style_facts_raw(source, dialect);
    let declared_keyframes = facts
        .animations
        .iter()
        .filter(|animation| animation.kind == ParsedAnimationFactKind::KeyframesDeclaration)
        .map(|animation| animation.name.clone())
        .collect::<BTreeSet<_>>();
    let mut emitted = BTreeSet::new();

    facts
        .animations
        .into_iter()
        .filter(|animation| animation.kind == ParsedAnimationFactKind::AnimationNameReference)
        .filter(|animation| !declared_keyframes.contains(animation.name.as_str()))
        .filter_map(|animation| {
            let start: u32 = animation.range.start().into();
            let end: u32 = animation.range.end().into();
            let byte_span = ParserByteSpanV0 {
                start: start as usize,
                end: end as usize,
            };
            if !emitted.insert((animation.name.clone(), byte_span.start, byte_span.end)) {
                return None;
            }
            Some((animation, parser_range_for_byte_span(source, byte_span)))
        })
        .map(|(animation, range)| OmenaQueryStyleDiagnosticV0 {
            code: "missingKeyframes",
            severity: "warning",
            provenance: vec![
                "omena-parser.animation-facts",
                "omena-query.style-diagnostics",
            ],
            range,
            message: format!("@keyframes '{}' not found in this file.", animation.name),
            tags: Vec::new(),
            create_custom_property: None,
            cascade_narrowing: None,
            cascade_confidence: None,
            polynomial_provenance: None,
            cross_file_scc: None,
        })
        .collect()
}
